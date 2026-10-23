//! Dungeon enemy, soldier, and trooper draw and behavior tables.

use crate::zelda_rtl::sprite::DrawMultipleData;

// ---------------------------------------------------------------------------
// kChainBallTrooperHead_* from sprite_main.c:293-294.
// kFlailTrooperBody_* from sprite_main.c:295-336.
// kFlailTrooperWeapon_* from sprite_main.c:356-362.
// ---------------------------------------------------------------------------
pub(in crate::zelda_rtl::sprite_main_draw) const CHAIN_BALL_TROOPER_HEAD_CHARS: [u8; 4] = [2, 2, 0, 4];
pub(in crate::zelda_rtl::sprite_main_draw) const CHAIN_BALL_TROOPER_HEAD_FLAGS: [u8; 4] = [0x40, 0, 0, 0];
pub(in crate::zelda_rtl::sprite_main_draw) const CHAIN_BALL_TROOPER_BODY_CHAR_BY_STATE: [u8; 4] = [0x0d, 0x60, 0x22, 0x10];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_GRAPHICS: [u8; 32] = [
    0x10, 0x11, 0x12, 0x13, 0x10, 0x11, 0x12, 0x13, 6, 7, 8, 9, 6, 7, 8, 9, 0, 1, 2, 3, 0, 1, 4, 5,
    0x0a, 0x0b, 0x0c, 0x0d, 0x0a, 0x0b, 0x0e, 0x0f,
];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_BODY_X_OFFSETS: [i8; 72] = [
    -4, 4, 12, -4, 4, 13, -4, 4, 13, -4, 4, 13, -4, 4, 13, -4, 4, 13, 0, 0, 4, 0, 0, 5, 0, 0, 6, 0,
    0, 4, -4, 4, -6, -4, 4, -5, -4, 4, -5, -4, 4, -6, -4, 4, -5, -4, 4, -6, 0, 0, 4, 0, 0, 3, 0, 0,
    2, 0, 0, 4, 0, 0, 0, 0, 0, 0, -4, 4, 4, -4, 4, 4,
];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_BODY_Y_OFFSETS: [i8; 72] = [
    0, 0, -4, 0, 0, -4, 0, 0, -3, 0, 0, -2, 0, 0, -3, 0, 0, -2, 0, 0, 1, 0, 0, 1, 0, 0, 2, 0, 0, 2,
    0, 0, -2, 0, 0, -2, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, 1, 0, 0, 1, 0, 0, 2, 0, 0, 2,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_BODY_CHARS: [u8; 72] = [
    0x46, 6, 0x2f, 0x46, 6, 0x2f, 0x48, 0xd, 0x2f, 0x48, 0xd, 0x2f, 0x49, 0xc, 0x2f, 0x49, 0xc,
    0x2f, 8, 8, 0x2f, 8, 8, 0x2f, 0x22, 0x22, 0x2f, 0x22, 0x22, 0x2f, 0xa, 0x64, 0x2f, 0xa, 0x64,
    0x2f, 0x2c, 0x67, 0x2f, 0x2c, 0x67, 0x2f, 0x2d, 0x66, 0x2f, 0x2d, 0x66, 0x2f, 8, 8, 0x2f, 8, 8,
    0x2f, 0x22, 0x22, 0x2f, 0x22, 0x22, 0x2f, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x46, 0x4b, 0x4b,
    0x69, 0x64, 0x64,
];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_BODY_FLAGS: [u8; 72] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x40, 0x40, 0, 0x40, 0x40, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0,
    0, 0, 0x40, 0x40, 0, 0x40, 0x40,
];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_BODY_SIZES: [u8; 72] = [
    2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2,
    0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2,
];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_BODY_SEGMENT_COUNTS: [u8; 24] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1,
];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_BODY_SPRITE_OFFSETS: [u8; 24] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8,
];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_CHAIN_SEGMENT_SCALES: [u8; 4] = [0x33, 0x66, 0x99, 0xcc];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_CHAIN_RADIUS_BY_FRAME: [u8; 32] = [
    0x10, 0x12, 0x14, 0x16, 0x18, 0x1a, 0x1c, 0x1e, 0x20, 0x22, 0x24, 0x26, 0x28, 0x2a, 0x2c, 0x2e,
    0x30, 0x2e, 0x2c, 0x2a, 0x28, 0x26, 0x24, 0x22, 0x20, 0x1e, 0x1c, 0x1a, 0x18, 0x16, 0x14, 0x12,
];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_WEAPON_X_CENTER_BY_DIRECTION: [i8; 4] = [4, 4, 12, -5];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_WEAPON_Y_CENTER_BY_DIRECTION: [i8; 4] = [-2, -2, -6, -4];
pub(in crate::zelda_rtl::sprite_main_draw) const FLAIL_TROOPER_ATTACK_DIRECTIONS: [u8; 4] = [3, 1, 2, 0];
pub(in crate::zelda_rtl::sprite_main_draw) const ROPE_GRAPHICS: [u8; 8] = [0, 0, 2, 3, 2, 3, 1, 1];
pub(in crate::zelda_rtl::sprite_main_draw) const ROPE_OAM_FLAGS: [u8; 8] = [0, 0x40, 0, 0, 0x40, 0x40, 0, 0x40];
pub(in crate::zelda_rtl::sprite_main_draw) const ROPE_FAST_GFX_BY_DIRECTION: [u8; 8] = [4, 5, 2, 3, 0, 1, 6, 7];
pub(in crate::zelda_rtl::sprite_main_draw) const ROPE_X_VELOCITIES: [i8; 8] = [8, -8, 0, 0, 16, -16, 0, 0];
pub(in crate::zelda_rtl::sprite_main_draw) const ROPE_Y_VELOCITIES: [i8; 8] = [0, 0, 8, -8, 0, 0, 0x10, -0x10];
pub(in crate::zelda_rtl::sprite_main_draw) const ROPE_IDLE_GFX_BY_DIRECTION: [u8; 4] = [2, 3, 1, 0];
pub(in crate::zelda_rtl::sprite_main_draw) const RECRUIT_X_VELOCITIES: [u8; 8] = [12, 244, 0, 0, 18, 238, 0, 0];
pub(in crate::zelda_rtl::sprite_main_draw) const RECRUIT_Y_VELOCITIES: [u8; 8] = [0, 0, 12, 244, 0, 0, 18, 238];

pub(in crate::zelda_rtl::sprite_main_draw) const METAL_BALL_LARGE_X_OFFSETS: [i8; 4] = [-8, 8, -8, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const METAL_BALL_LARGE_Y_OFFSETS: [i8; 4] = [-8, -8, 8, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const METAL_BALL_LARGE_CHARS: [u8; 8] = [0x84, 0x88, 0x88, 0x88, 0x86, 0x88, 0x88, 0x88];

pub(in crate::zelda_rtl::sprite_main_draw) const METAL_BALL_LARGE_FLAGS: [u8; 4] = [0, 0, 0xc0, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_BOMB_EXPLOSION_X_OFFSETS: [i8; 16] =
    [-12, 12, -12, 12, -8, 8, -8, 8, -8, 8, -8, 8, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_BOMB_EXPLOSION_Y_OFFSETS: [i8; 16] =
    [-12, -12, 12, 12, -8, -8, 8, 8, -8, -8, 8, 8, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_BOMB_EXPLOSION_CHARS: [u8; 16] = [
    0x88, 0x88, 0x88, 0x88, 0x8a, 0x8a, 0x8a, 0x8a, 0x84, 0x84, 0x84, 0x84, 0x86, 0x86, 0x86, 0x86,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_BOMB_EXPLOSION_FLAGS: [u8; 16] = [
    0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0, 0, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_THROWING_DRAW_X_OFFSETS: [i8; 16] =
    [15, 7, 17, 9, -8, 0, -10, -2, 13, 13, 13, 13, -4, -4, -4, -4];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_THROWING_DRAW_Y_OFFSETS: [i8; 16] = [
    -2, -2, -2, -2, -2, -2, -2, -2, 8, 0, 10, 2, -14, -6, -16, -8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_THROWING_DRAW_CHARS: [u8; 16] = [
    0x6f, 0x7f, 0x6f, 0x7f, 0x6f, 0x7f, 0x6f, 0x7f, 0x6e, 0x7e, 0x6e, 0x7e, 0x6e, 0x7e, 0x6e, 0x7e,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_THROWING_DRAW_FLAGS: [u8; 16] = [
    0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHER_WEAPON_BASE_FRAME_BY_DIRECTION: [u8; 4] = [9, 3, 0, 6];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHER_SOLDIER_DRAW_X_OFFSETS: [i8; 48] = [
    -1, 7, 3, 3, -1, 7, 3, 3, -1, 7, 7, 7, -5, -5, -10, -2, -4, -4, -6, 2, -5, -5, -5, -5, 6, 14,
    11, 11, 6, 14, 11, 11, 6, 14, 14, 14, 11, 11, 18, 10, 12, 12, 14, 6, 11, 11, 11, 11,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHER_SOLDIER_DRAW_Y_OFFSETS: [i8; 48] = [
    7, 7, 3, 11, 6, 6, 1, 9, 7, 7, 7, 7, -2, 6, 2, 2, -2, 6, 2, 2, -2, 6, 6, 6, -6, -6, -12, -4,
    -6, -6, -9, -1, -6, -6, -6, -6, -2, 6, 2, 2, -2, 6, 2, 2, -2, 6, 6, 6,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHER_SOLDIER_DRAW_CHARS: [u8; 48] = [
    0xa, 0xa, 0x2a, 0x2b, 0x1a, 0x1a, 0x2a, 0x2b, 0xa, 0xa, 0xa, 0xa, 0xb, 0xb, 0x3d, 0x3a, 0x1b,
    0x1b, 0x3d, 0x3a, 0xb, 0xb, 0xb, 0xb, 0xa, 0xa, 0x2b, 0x2a, 0xa, 0xa, 0x2b, 0x2a, 0xa, 0xa,
    0xa, 0xa, 0xb, 0xb, 0x3d, 0x3a, 0x1b, 0x1b, 0x3d, 0x3a, 0xb, 0xb, 0xb, 0xb,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHER_SOLDIER_DRAW_FLAGS: [u8; 48] = [
    0xd, 0x4d, 8, 8, 0xd, 0x4d, 8, 8, 0xd, 0x4d, 0x4d, 0x4d, 0xd, 0x8d, 0x48, 0x48, 0xd, 0x8d,
    0x48, 0x48, 0xd, 0x8d, 0x8d, 0x8d, 0x8d, 0xcd, 0x88, 0x88, 0x8d, 0xcd, 0x88, 0x88, 0x8d, 0xcd,
    0xcd, 0xcd, 0x4d, 0xcd, 8, 8, 0x4d, 0xcd, 8, 8, 0x4d, 0xcd, 0xcd, 0xcd,
];

pub(in crate::zelda_rtl::sprite_main_draw) const STALFOS_KNIGHT_HEAD_CHARS: [u8; 4] = [0x66, 0x66, 0x46, 0x46];

pub(in crate::zelda_rtl::sprite_main_draw) const STALFOS_KNIGHT_HEAD_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const POE_DRAW_X_OFFSETS: [i8; 2] = [9, -1];

pub(in crate::zelda_rtl::sprite_main_draw) const POE_DRAW_CHARS: [u8; 4] = [0x7c, 0x80, 0xb7, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const BARI_BURST_X_VELOCITIES: [u8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, 0xf8, 0xf5, 0xf2, 0xf0, 0xf2, 0xf5, 0xf8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BARI_BURST_Y_VELOCITIES: [u8; 16] = [
    0xf0, 0xf2, 0xf5, 0xf8, 0, 8, 11, 14, 16, 14, 11, 8, 0, 0xf7, 0xf5, 0xf2,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BARI_GRAPHICS: [u8; 2] = [0, 3];

pub(in crate::zelda_rtl::sprite_main_draw) const RED_BARI_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0022,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4022,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0032,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4032,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0023,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4023,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0033,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4033,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const HARD_HAT_BEETLE_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x0140,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 2,
        char_flags: 0x0142,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x0140,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 2,
        char_flags: 0x0144,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_ARROW_X_VELOCITIES: [u8; 8] = [0, 0, 16, 16, 0, 0, 0xf0, 0xf0];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_ARROW_Y_VELOCITIES: [u8; 8] = [16, 16, 0, 0, 0xf0, 0xf0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_ARROW_DIRECTIONS: [u8; 4] = [0, 2, 1, 3];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_ARROW_DRAW_X_OFFSETS: [i16; 8] = [-8, 0, 0, 8, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_ARROW_DRAW_Y_OFFSETS: [i16; 8] = [0, 0, 0, 0, -8, 0, 0, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_ARROW_DRAW_CHARS: [u8; 32] = [
    0x3a, 0x3d, 0x3d, 0x3a, 0x2a, 0x2b, 0x2b, 0x2a, 0x7c, 0x6c, 0x6c, 0x7c, 0x7b, 0x6b, 0x6b, 0x7b,
    0x3a, 0x3b, 0x3b, 0x3a, 0x2a, 0x3c, 0x3c, 0x2a, 0x81, 0x80, 0x80, 0x81, 0x91, 0x90, 0x90, 0x91,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ENEMY_ARROW_DRAW_FLAGS: [u8; 32] = [
    8, 8, 0x48, 0x48, 8, 8, 0x88, 0x88, 9, 0x49, 9, 0x49, 9, 0x89, 9, 0x89, 8, 0x88, 0xc8, 0x48, 8,
    8, 0x88, 0x88, 0x49, 0x49, 9, 9, 0x89, 0x89, 9, 9,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARMOS_DRAW_FRAMES: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 0,
        y: -16,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e0,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FREEZOR_DRAW_FRAMES0: [DrawMultipleData; 28] = [
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x00ab,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 11,
        char_flags: 0x40ab,
        ext: 0,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x00ba,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 11,
        char_flags: 0x00bb,
        ext: 0,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x40bb,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 11,
        char_flags: 0x40ba,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 2,
        char_flags: 0x00ae,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 2,
        char_flags: 0x40ae,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 10,
        char_flags: 0x00be,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 10,
        char_flags: 0x40be,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00af,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x40af,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 12,
        char_flags: 0x00bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 12,
        char_flags: 0x40bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x00aa,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40aa,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x00aa,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40aa,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FREEZOR_DRAW_FRAMES1: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ae,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40ae,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x00be,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40be,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: 0,
        char_flags: 0x00ae,
        ext: 0,
    },
    DrawMultipleData {
        x: 10,
        y: 0,
        char_flags: 0x40ae,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: 8,
        char_flags: 0x00be,
        ext: 0,
    },
    DrawMultipleData {
        x: 10,
        y: 8,
        char_flags: 0x40be,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const ROPA_DRAW_FRAMES: [DrawMultipleData; 12] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0026,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0027,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0036,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0037,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4027,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4026,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4037,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4036,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4008,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORO_X_VELOCITIES: [i8; 4] = [16, -16, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const WIZZROBE_BEAM_XY_VELOCITIES: [i8; 6] = [32, -32, 0, 0, 32, -32];

pub(in crate::zelda_rtl::sprite_main_draw) const STAL_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0044,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 11,
        char_flags: 0x0070,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0044,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 12,
        char_flags: 0x0070,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0044,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 13,
        char_flags: 0x0070,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const GIBO_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const GIBO_ALT_OAM_FLAGS: [u8; 2] = [11, 7];

pub(in crate::zelda_rtl::sprite_main_draw) const GIBO_DRAW_FRAMES: [DrawMultipleData; 32] = [
    DrawMultipleData {
        x: 4,
        y: -4,
        char_flags: 0x408a,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x408f,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 12,
        char_flags: 0x408e,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0x408c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -4,
        char_flags: 0x40aa,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x409f,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 12,
        char_flags: 0x409e,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: -3,
        char_flags: 0x40aa,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: -3,
        char_flags: 0x409f,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: 11,
        char_flags: 0x409e,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 3,
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: -3,
        char_flags: 0x408a,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: -3,
        char_flags: 0x408f,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: 11,
        char_flags: 0x408e,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 3,
        char_flags: 0x408c,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: -4,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 13,
        y: -4,
        char_flags: 0x008f,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 12,
        char_flags: 0x008e,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: 4,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: -4,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 13,
        y: -4,
        char_flags: 0x009f,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 12,
        char_flags: 0x009e,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: 4,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: -3,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: -3,
        char_flags: 0x009f,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: 11,
        char_flags: 0x009e,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 3,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: -3,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: -3,
        char_flags: 0x008f,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: 11,
        char_flags: 0x008e,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 3,
        char_flags: 0x008c,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const LASER_EYE_DRAW_FRAMES: [DrawMultipleData; 24] = [
    DrawMultipleData {
        x: 8,
        y: -4,
        char_flags: 0x40c8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x40d8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 12,
        char_flags: 0xc0c8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -4,
        char_flags: 0x40c9,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x40d9,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 12,
        char_flags: 0xc0c9,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x00c8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00d8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 12,
        char_flags: 0x80c8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x00c9,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00d9,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 12,
        char_flags: 0x80c9,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x00d6,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x00d7,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 8,
        char_flags: 0x40d6,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x00c6,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x00c7,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 8,
        char_flags: 0x40c6,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x80d6,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x80d7,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 0,
        char_flags: 0xc0d6,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x80c6,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x80c7,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 0,
        char_flags: 0xc0c6,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const GIBDO_DRAW_FRAMES: [DrawMultipleData; 24] = [
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x408a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x4088,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x408c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x408e,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const CHOMP_Z_VELOCITY_TARGETS: [i8; 2] = [8, -8];

pub(in crate::zelda_rtl::sprite_main_draw) const CHOMP_X_VELOCITY_TARGETS: [i8; 4] = [16, -16, 28, -28];

pub(in crate::zelda_rtl::sprite_main_draw) const RAT_GRAPHICS_BY_ANIM_STATE: [u8; 16] =
    [0, 0, 3, 3, 1, 2, 4, 5, 1, 2, 4, 5, 0, 0, 3, 3];

pub(in crate::zelda_rtl::sprite_main_draw) const RAT_IDLE_ANIM_STATES: [u8; 8] = [10, 11, 6, 7, 2, 3, 14, 15];

pub(in crate::zelda_rtl::sprite_main_draw) const RAT_RUN_ANIM_STATES: [u8; 8] = [8, 9, 4, 5, 0, 1, 12, 13];

pub(in crate::zelda_rtl::sprite_main_draw) const RAT_WALL_TURN_DIRECTIONS: [u8; 4] = [2, 3, 1, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const KEESE_ATTACK_START_PHASES: [i8; 4] = [2, 10, 6, 14];

pub(in crate::zelda_rtl::sprite_main_draw) const KEESE_ATTACK_X_VELOCITIES: [i8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const KEESE_ATTACK_Y_VELOCITIES: [i8; 16] = [
    -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
];

// Remaining Rat, Keese, and Soldier-B behavior tables promoted from sprite_main_draw.rs.
pub(in crate::zelda_rtl::sprite_main_draw) const RAT_OAM_FLIP_BY_ANIM_STATE: [u8; 16] = [
    0, 0x40, 0, 0x40, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x80, 0xc0, 0x80, 0xc0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const KEESE_ORBIT_DIRECTION_STEPS: [i8; 2] = [1, -1];

pub(in crate::zelda_rtl::sprite_main_draw) const WALL_MASTER_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x01a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: 0,
        char_flags: 0x01aa,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 16,
        char_flags: 0x01ba,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x01a8,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x01ab,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: 0,
        char_flags: 0x01af,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 16,
        char_flags: 0x01bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x01ad,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const WIZZROBE_DRAW_FRAMES: [DrawMultipleData; 24] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00b2,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00b3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00b2,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00b3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00b2,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00b3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00b2,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00b3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00b2,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00b3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x408c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00b2,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00b3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x408a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00b2,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00b3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00b2,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00b3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x008e,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const STALFOS_BONE_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: -4,
        y: -2,
        char_flags: 0x802f,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 2,
        char_flags: 0x402f,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 2,
        char_flags: 0x002f,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -2,
        char_flags: 0xc02f,
        ext: 0,
    },
    DrawMultipleData {
        x: 2,
        y: -4,
        char_flags: 0x403f,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: 4,
        char_flags: 0x803f,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: -4,
        char_flags: 0x003f,
        ext: 0,
    },
    DrawMultipleData {
        x: 2,
        y: 4,
        char_flags: 0xc03f,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const STALFOS_DRAW_FRAMES: [DrawMultipleData; 36] = [
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0006,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0006,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x4006,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x4006,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0006,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0006,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x4006,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x4006,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0002,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 5,
        char_flags: 0x002e,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0024,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0002,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x4002,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 5,
        char_flags: 0x402e,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4024,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x4002,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x400e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: -8,
        char_flags: 0x4002,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4008,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: -8,
        char_flags: 0x0002,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -6,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -6,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -6,
        char_flags: 0x0004,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const STALFOS_KNIGHT_DRAW_FRAMES: [DrawMultipleData; 35] = [
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0064,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x0061,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x0062,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: 16,
        char_flags: 0x0074,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: 16,
        char_flags: 0x4074,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -7,
        char_flags: 0x0064,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 1,
        char_flags: 0x0061,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 1,
        char_flags: 0x0062,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: 16,
        char_flags: 0x0065,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: 16,
        char_flags: 0x4065,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0048,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x0049,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x004b,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x004c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x004c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x0068,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0069,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0069,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0069,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0069,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: -7,
        char_flags: 0x4064,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 1,
        char_flags: 0x4062,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 1,
        char_flags: 0x4061,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: 16,
        char_flags: 0x0065,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: 16,
        char_flags: 0x4065,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: -8,
        char_flags: 0x4064,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x4062,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x4061,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: 16,
        char_flags: 0x0074,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: 16,
        char_flags: 0x4074,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x4049,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x4048,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x404c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x404b,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x404b,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const BABUSU_DRAW_FRAMES: [DrawMultipleData; 40] = [
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x4380,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x4380,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x43b6,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x43b6,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x43b7,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x0380,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x4380,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x03b6,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x03b7,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x03b7,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x0380,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x0380,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x8380,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x8380,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x83b6,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x83b6,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x83b7,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0380,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x8380,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x03b6,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x03b7,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x03b7,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0380,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0380,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0a4e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a5e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4a4e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4a5e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x0a6c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a6b,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x8a6c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x8a6b,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x8a4e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x8a5e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0xca4e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0xca5e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x4a6c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4a6b,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0xca6c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0xca6b,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const GORIYA_DRAW_FRAMES_2: [DrawMultipleData; 3] = [
    DrawMultipleData {
        x: 10,
        y: 4,
        char_flags: 0x4077,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: 4,
        char_flags: 0x0077,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x0076,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const GORIYA_DRAW_FRAMES: [DrawMultipleData; 32] = [
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0044,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: -8,
        char_flags: 0x4044,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x0064,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x4054,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0044,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: -8,
        char_flags: 0x4044,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x4074,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x4062,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0044,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x4044,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x0062,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: 8,
        char_flags: 0x4064,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0046,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: -8,
        char_flags: 0x4046,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x0066,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x4056,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0046,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: -8,
        char_flags: 0x4046,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x4075,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x406a,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0046,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x4046,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x006a,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: 8,
        char_flags: 0x0075,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: -8,
        char_flags: 0x004e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x006c,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: -7,
        char_flags: 0x004e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x006e,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: -8,
        char_flags: 0x404e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x406c,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: -7,
        char_flags: 0x404e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x406e,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPIKE_TRAP_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x00c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x40c4,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x80c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc0c4,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const EYEGORE_DRAW_FRAMES: [DrawMultipleData; 48] = [
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -4,
        char_flags: 0x40a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0x009c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x409c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -4,
        char_flags: 0x40a4,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0x009c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x409c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -4,
        char_flags: 0x408c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0x009c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x409c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -3,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: -3,
        char_flags: 0x408c,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 13,
        char_flags: 0x00bc,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 5,
        char_flags: 0x408a,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -3,
        char_flags: 0x008c,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -3,
        char_flags: 0x408c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 5,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: 13,
        char_flags: 0x40bc,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -3,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -3,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x40aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x40aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -3,
        char_flags: 0x40aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x40a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -3,
        char_flags: 0x40aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x40a8,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -4,
        char_flags: 0x408e,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0x009e,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x409e,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -3,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: -3,
        char_flags: 0x408e,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 13,
        char_flags: 0x00bd,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 5,
        char_flags: 0x40a0,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -3,
        char_flags: 0x008e,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -3,
        char_flags: 0x408e,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 5,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 12,
        y: 13,
        char_flags: 0x40bd,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const STALFOS_DRAW_CHARS: [u8; 4] = [2, 2, 0, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const STALFOS_DRAW_FLAGS: [u8; 4] = [0x70, 0x30, 0x30, 0x30];

pub(in crate::zelda_rtl::sprite_main_draw) const TUTORIAL_SOLDIER_DRAW_X_OFFSETS: [i16; 20] = [
    4, 0, -6, -6, 2, 0, 0, -7, -7, -7, 0, 0, 0x0f, 0x0f, 0x0f, 6, 0x0e, -4, 4, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TUTORIAL_SOLDIER_DRAW_Y_OFFSETS: [i16; 20] = [
    0, -10, -4, 12, 12, 0, -9, -11, -3, 5, 0, -9, -11, -3, 5, -11, 5, 0, 0, -9,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TUTORIAL_SOLDIER_DRAW_CHARS: [u8; 20] = [
    0x46, 0x40, 0, 0x28, 0x29, 0x4e, 0x42, 0x39, 0x2a, 0x3a, 0x4e, 0x42, 0x39, 0x2a, 0x3a, 0x26,
    0x38, 0x64, 0x64, 0x44,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TUTORIAL_SOLDIER_DRAW_FLAGS: [u8; 20] = [
    0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0, 0x40, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TUTORIAL_SOLDIER_DRAW_BIG: [u8; 20] =
    [2, 2, 2, 0, 0, 2, 2, 0, 0, 0, 2, 2, 0, 0, 0, 2, 0, 2, 2, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_D3_STAL_STAL_GFX: [u8; 5] = [2, 2, 1, 0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const GORIYA_DRAW_OFFSETS: [usize; 11] = [0, 4, 8, 12, 16, 20, 24, 26, 28, 30, 32];

pub(in crate::zelda_rtl::sprite_main_draw) const PSYCHO_TROOPER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

pub(in crate::zelda_rtl::sprite_main_draw) const JAVELIN_TROOPER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

pub(in crate::zelda_rtl::sprite_main_draw) const BUSH_JAVELIN_SOLDIER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHER_SOLDIER_DRAW_WEAPON_OAM_OFFS: [u8; 4] = [0, 0, 0, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHER_SOLDIER_DRAW_HEAD_OAM_OFFS: [u8; 4] = [16, 16, 16, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHER_SOLDIER_DRAW_BODY_OAM_OFFS: [u8; 4] = [20, 20, 20, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHER_SOLDIER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_48_RED_JAVELIN_GUARD_LOCAL_GRAPHICS: [u8; 4] = [12, 0, 18, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_48_RED_JAVELIN_GUARD_DIR_LOCK: [u8; 4] = [3, 2, 0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_46_BLUE_ARCHER_LOCAL_GRAPHICS: [u8; 4] = [8, 0, 12, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_46_BLUE_ARCHER_DIR_LOCK: [u8; 4] = [3, 2, 0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const CHAIN_BALL_TROOPER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_63_DEBIRANDO_PIT_OPENING_GFX: [u8; 4] = [5, 4, 3, 3];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_63_DEBIRANDO_PIT_CLOSING_GFX: [u8; 4] = [3, 3, 4, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_64_DEBIRANDO_EMERGE_GFX: [u8; 2] = [1, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_64_DEBIRANDO_SUBMERGE_GFX: [u8; 2] = [0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const DEBIRANDO_PIT_DRAW_X_OFFSETS: [i16; 24] = [
    -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, 0, 8, 0, 8, 0, 8, 0, 8, -8, 8, -8, 8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const DEBIRANDO_PIT_DRAW_Y_OFFSETS: [i16; 24] = [
    -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, -8, -8, 8, 8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const DEBIRANDO_PIT_DRAW_CHARS: [u8; 24] = [
    4, 4, 4, 4, 0x22, 0x22, 0x22, 0x22, 2, 2, 2, 2, 0x29, 0x29, 0x29, 0x29, 0x39, 0x39, 0x39, 0x39,
    0x2a, 0x2a, 0x2a, 0x2a,
];

pub(in crate::zelda_rtl::sprite_main_draw) const DEBIRANDO_PIT_DRAW_FL: [u8; 24] = [
    0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40,
    0x80, 0xc0, 0, 0x40, 0x80, 0xc0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const DEBIRANDO_PIT_DRAW_BIG: [u8; 6] = [2, 2, 2, 0, 0, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const DEBIRANDO_DRAW_X_OFFSETS: [i8; 16] =
    [0, 8, 0, 8, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const DEBIRANDO_DRAW_Y_OFFSETS: [i8; 16] =
    [2, 2, 6, 6, -2, -2, 6, 6, -4, -4, -4, -4, -4, -4, -4, -4];

pub(in crate::zelda_rtl::sprite_main_draw) const DEBIRANDO_DRAW_CHARS: [u8; 16] = [
    0, 0, 0xd8, 0xd8, 0, 0, 0xd9, 0xd9, 0, 0, 0, 0, 0x20, 0x20, 0x20, 0x20,
];

pub(in crate::zelda_rtl::sprite_main_draw) const DEBIRANDO_DRAW_FL: [u8; 16] =
    [1, 0x41, 0, 0x40, 1, 1, 0, 0x40, 1, 1, 1, 1, 1, 1, 1, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const DEBIRANDO_DRAW_BIG: [u8; 16] = [0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_02_STALFOS_HEAD_OAM_FLAGS: [u8; 4] = [0, 0, 0, 0x40];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_02_STALFOS_HEAD_LOCAL_GRAPHICS: [u8; 4] = [0, 1, 2, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_18_MINI_MOLDORM_LOCAL_X_VELOCITIES: [i8; 16] = [
    24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9, 0, 9, 17, 22,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_18_MINI_MOLDORM_LOCAL_Y_VELOCITIES: [i8; 16] = [
    0, 9, 17, 22, 24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_18_MINI_MOLDORM_NEXT_DIR: [u8; 16] =
    [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_13_MINI_HELMASAUR_LOCAL_GRAPHICS: [u8; 8] = [3, 4, 3, 4, 2, 2, 5, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_13_MINI_HELMASAUR_OAM_FLAGS: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0x40, 0x40, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_19_POE_ACCEL: [i8; 4] = [1, -1, 2, -2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_19_POE_OAM_FLAGS: [u8; 2] = [0x40, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_19_POE_LOCAL_Y_VELOCITIES: [i8; 2] = [8, -8];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_6_D_RAT_LOCAL_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_6_D_RAT_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_7_C_GREEN_STALFOS_DIRECTIONS: [u8; 4] = [4, 6, 0, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_7_C_GREEN_STALFOS_OAM_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_7_C_GREEN_STALFOS_LOCAL_GRAPHICS: [u8; 4] = [0, 0, 1, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const CHAIN_CHOMP_DRAW_LOCAL_GRAPHICS: [u8; 16] =
    [0, 1, 2, 3, 3, 3, 2, 1, 0, 0, 0, 4, 4, 4, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const CHAIN_CHOMP_DRAW_OAM_FLAGS: [u8; 16] = [
    0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BUSH_SOLDIER_COMMON_DRAW_Y_OFFSETS: [i8; 14] =
    [8, 8, 8, 8, 2, 8, 0, 8, -3, 8, -3, 8, -3, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const BUSH_SOLDIER_COMMON_DRAW_CHARS: [u8; 14] = [
    0x20, 0x20, 0x20, 0x20, 0x40, 0x20, 0x40, 0x20, 0x40, 0x20, 0x42, 0x20, 0x42, 0x20,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BUSH_SOLDIER_COMMON_DRAW_FLAGS: [u8; 14] =
    [9, 3, 0x49, 0x43, 9, 3, 0x49, 0x43, 9, 3, 0x49, 0x43, 9, 3];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_23_RED_BARI_BARI_IDLE_X_VELOCITIES: [u8; 2] = [8, 0xf8];

pub(in crate::zelda_rtl::sprite_main_draw) const ZOL_DRAW_OAM_FLAGS: [u8; 4] = [0, 0, 0x40, 0x40];

pub(in crate::zelda_rtl::sprite_main_draw) const ZOL_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x036c,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x036d,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0060,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x0070,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x4070,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4060,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0040,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0040,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const BOMB_TROOPER_DRAW_ARM_X_OFFSETS: [i8; 8] = [-1, 1, 2, 0, 9, 9, -8, -8];

pub(in crate::zelda_rtl::sprite_main_draw) const BOMB_TROOPER_DRAW_ARM_Y_OFFSETS: [i8; 8] =
    [-12, -12, -12, -12, -16, -14, -12, -14];
