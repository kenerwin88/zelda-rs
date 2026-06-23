//! Shared Blind boss draw, movement, and projectile tables.

use super::sprite::DrawMultipleData;

// Tables shared by Blind head drawing (from sprite_main.c lines 400-401).
pub(super) const BLIND_HEAD_DRAW_CHARS: [u8; 16] = [
    0x86, 0x86, 0x84, 0x82, 0x80, 0x82, 0x84, 0x86, 0x86, 0x86, 0x88, 0x8a, 0x8c, 0x8a, 0x88, 0x86,
];
pub(super) const BLIND_HEAD_DRAW_FLAGS: [u8; 16] = [
    0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0,
];

// kBlindPoof_Dmd from sprite_main.c:15819.
pub(super) const BLIND_POOF_DRAW_FRAMES: [DrawMultipleData; 37] = [
    DrawMultipleData {
        x: -16,
        y: -20,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -11,
        y: -28,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -23,
        y: -26,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -17,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -20,
        y: -13,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: -37,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -27,
        y: -31,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -10,
        y: -28,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: -28,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -20,
        y: -27,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -27,
        y: -17,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -17,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: -13,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -18,
        y: -37,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: -33,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -32,
        y: -32,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -23,
        y: -31,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -15,
        y: -24,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -23,
        y: -31,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -15,
        y: -24,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: -22,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: -22,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: -14,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -12,
        y: -32,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -26,
        y: -29,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -22,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: -20,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -26,
        y: -29,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -22,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: -20,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -17,
        y: -27,
        char_flags: 0x059b,
        ext: 0,
    },
    DrawMultipleData {
        x: -10,
        y: -26,
        char_flags: 0x059b,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -22,
        char_flags: 0x459b,
        ext: 0,
    },
    DrawMultipleData {
        x: -19,
        y: -16,
        char_flags: 0x459b,
        ext: 0,
    },
    DrawMultipleData {
        x: -6,
        y: -12,
        char_flags: 0x059b,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 13,
        char_flags: 0x0b20,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 23,
        char_flags: 0x0b22,
        ext: 2,
    },
];

// kBlind_Dmd from sprite_main.c:15863.
pub(super) const BLIND_DRAW_FRAMES: [DrawMultipleData; 105] = [
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: -15,
        y: 5,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 23,
        char_flags: 0x0ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 23,
        char_flags: 0x4ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a8a,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: -1,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -11,
        y: 9,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 23,
        char_flags: 0x0ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 23,
        char_flags: 0x4ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a88,
        ext: 2,
    },
    DrawMultipleData {
        x: 10,
        y: -2,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a84,
        ext: 2,
    },
    DrawMultipleData {
        x: 13,
        y: 8,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -10,
        y: -2,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 23,
        char_flags: 0x0ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 23,
        char_flags: 0x4ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a82,
        ext: 2,
    },
    DrawMultipleData {
        x: 18,
        y: 4,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -15,
        y: -1,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 23,
        char_flags: 0x0ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 23,
        char_flags: 0x4ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a80,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a80,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a80,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 9,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 9,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 2,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 16,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 16,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 20,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 23,
        char_flags: 0x0a8c,
        ext: 2,
    },
];

pub(super) const BLIND_LASER_GRAPHICS_BY_HEAD_DIR: [u8; 16] =
    [7, 7, 8, 9, 10, 9, 8, 7, 7, 7, 8, 9, 10, 9, 8, 7];

pub(super) const BLIND_LASER_OAM_FLAGS_BY_HEAD_DIR: [u8; 16] = [
    0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80,
];

pub(super) const BLIND_HEAD_X_POSITION_LIMITS: [u8; 2] = [0x98, 0x58];

pub(super) const BLIND_HEAD_Y_POSITION_LIMITS: [u8; 2] = [0xb0, 0x50];

pub(super) const BLIND_HEAD_Y_VELOCITY_LIMITS: [i8; 2] = [24, -24];

pub(super) const BLIND_HEAD_X_VELOCITY_LIMITS: [i8; 2] = [32, -32];

pub(super) const BLIND_DEFEAT_GRAPHICS_SEQUENCE: [u8; 7] = [20, 19, 18, 17, 16, 15, 15];

pub(super) const BLIND_OSCILLATION_Y_VELOCITY_TARGETS: [i8; 2] = [18, -18];

pub(super) const BLIND_OSCILLATION_X_VELOCITY_TARGETS: [i8; 2] = [24, -24];

pub(super) const BLIND_OSCILLATION_X_POSITION_TARGETS: [u8; 2] = [164, 76];

pub(super) const BLIND_SWITCH_WALL_Y_VELOCITY_TARGETS: [i8; 2] = [64, -64];

pub(super) const BLIND_SWITCH_WALL_Y_POSITION_TARGETS: [u8; 2] = [0x90, 0x50];

pub(super) const BLIND_WHIRL_AROUND_GRAPHICS_TARGETS: [u8; 2] = [0, 9];

pub(super) const BLIND_BEHIND_CURTAIN_GRAPHICS: [u8; 4] = [14, 13, 12, 10];

pub(super) const BLIND_REROBE_GRAPHICS: [u8; 5] = [10, 11, 12, 13, 14];

pub(super) const BLIND_ROBE_ANIMATION_GRAPHICS: [u8; 8] = [7, 8, 9, 8, 0, 1, 2, 1];

pub(super) const BLIND_FIREBALL_X_VELOCITIES_BY_HEAD_DIR: [i8; 16] = [
    -32, -28, -24, -16, 0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28,
];

pub(super) const BLIND_FIREBALL_Y_VELOCITIES_BY_HEAD_DIR: [i8; 16] = [
    0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16,
];

pub(super) const BLIND_HEAD_DIRECTION_BASES: [u8; 17] =
    [0, 1, 2, 3, 4, 3, 2, 1, 0, 15, 14, 13, 12, 13, 14, 15, 0];

pub(super) const BLIND_HEAD_FRAME_BY_DISTANCE: [u8; 8] = [0, 1, 1, 2, 2, 3, 3, 4];

pub(super) const BLIND_LASER_X_VELOCITIES_BY_HEAD_DIR: [i8; 16] =
    [-8, -8, -8, -4, 0, 4, 8, 8, 8, 8, 8, 4, 0, -4, -8, -8];

pub(super) const BLIND_LASER_Y_VELOCITIES_BY_HEAD_DIR: [i8; 16] =
    [0, 0, 4, 8, 8, 8, 4, 0, 0, 0, -4, -8, -8, -8, -4, 0];

pub(super) const BLIND_POOF_DRAW_FRAME_STARTS: [u8; 8] = [0, 1, 5, 13, 23, 30, 35, 37];

pub(super) const BLIND_HEAD_OAM_OFFSETS_BY_GRAPHICS: [u8; 10] = [4, 4, 4, 5, 5, 0, 0, 0, 0, 0];
