// Methods ported from zelda3/src/ancilla.c and included inside ZeldaState.

use super::*;
use crate::types::{
    abs16, abs8, sign16, sign8, AncillaRadialProjection, PairU8, Point16U, ProjectSpeedRet,
    SpriteHitBox,
};
use crate::zelda_rtl::sprite::SpriteSpawnInfo;

const ANCILLA_Z_SUBPIXEL_PLAYER: usize = 0x02a8;
const ANCILLA_TILE_ATTR_PLAYER: usize = 0x03e4;
const ANCILLA_ALLOC_ROTATE_PLAYER: usize = 0x03c4;
const ANCILLA_S_PLAYER: usize = 0x03a9;
const ANCILLA_T_PLAYER: usize = 0x03d5;
const ANCILLA_R_PLAYER: usize = 0x03ea;
const DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER: usize = 0x0646;
const ANCILLA_INTERACTIVE_RESET_FLAG: usize = 0x02f3;
const SPRITE_TILETYPE_ANCILLA: usize = 0x0fa5;
const CURRENT_AREA_OF_PLAYER_ANCILLA: usize = 0x0700;
const BOOMERANG_TEMP_Y: usize = 0x0399;
const BOOMERANG_TEMP_X: usize = 0x039b;
// Single-use coordinate scratch for arrow setup; NES_Ver2 aliases are broader shared work RAM.
const SCRATCH_0_ANCILLA: usize = 0x0072;
const SCRATCH_1_ANCILLA: usize = 0x0074;
const INDEX_OF_INTERACTING_TILE_ANCILLA: usize = 0x0076;
const SPRITE_IGNORE_PROJECTILE_ANCILLA: usize = 0x0ba0;
const REPULSESPARK_FLOOR_STATUS_ANCILLA: usize = 0x0b68;
const REPULSESPARK_TIMER_ANCILLA: usize = 0x0fac;
const REPULSESPARK_X_LO_ANCILLA: usize = 0x0fad;
const REPULSESPARK_Y_LO_ANCILLA: usize = 0x0fae;
const REPULSESPARK_ANIM_DELAY_ANCILLA: usize = 0x0faf;
const SPRITE_FLAGS_ANCILLA: usize = 0x0b6b;
const DAMAGE_TYPE_DETERMINER_ANCILLA: usize = 0x0cf2;
const SPRITE_B_ANCILLA: usize = 0x0da0;
const SPRITE_C_ANCILLA: usize = 0x0db0;
const SPRITE_BUMP_DAMAGE_ANCILLA: usize = 0x0cd2;
const SPRITE_HEALTH_ANCILLA: usize = 0x0e50;
const SPRITE_HEAD_DIR_ANCILLA: usize = 0x0eb0;
const SPRITE_F_ANCILLA: usize = 0x0ea0;
const SPRITE_G_ANCILLA: usize = 0x0ed0;
const SPRITE_DELAY_AUX2_ANCILLA: usize = 0x0e10;
const SPRITE_DELAY_AUX3_ANCILLA: usize = 0x0ee0;
const SPRITE_HIT_TIMER_ANCILLA: usize = 0x0ef0;
const SPRITE_Y_RECOIL_ANCILLA: usize = 0x0f30;
const SPRITE_OAM_FLAGS_ANCILLA: usize = 0x0f50;
const GARNISH_ACTIVE_ANCILLA: usize = 0x0fb4;
const GARNISH_Y_LO_ANCILLA: usize = 0x1f81e;
const GARNISH_X_LO_ANCILLA: usize = 0x1f83c;
const GARNISH_Y_HI_ANCILLA: usize = 0x1f85a;
const GARNISH_X_HI_ANCILLA: usize = 0x1f878;
const GARNISH_SPRITE_ANCILLA: usize = 0x1f8b4;
const GARNISH_COUNTDOWN_ANCILLA: usize = 0x1f90e;
const DOOR_DEBRIS_DIRECTION: usize = 0x073c;
const SWORDBEAM_TEMP_X: usize = 0x1580e;
const SWORDBEAM_TEMP_Y: usize = 0x15810;
const TAGALONG_Y_LO_ANCILLA: usize = 0x1a00;
const TAGALONG_Y_HI_ANCILLA: usize = 0x1a14;
const TAGALONG_X_LO_ANCILLA: usize = 0x1a28;
const TAGALONG_X_HI_ANCILLA: usize = 0x1a3c;
const MILESTONE_ITEM_GFX_SWAP_COUNTDOWN: usize = 0x04c2;
const TRIGGER_SPECIAL_ENTRANCE_ANCILLA: usize = 0x04c6;
const MAGIC_SPELL_PLAYER_LOCK_FLAG: usize = 0x0325;

const BOMBOS_PANNED_SFX_BITS: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
const BOMBOS_BLAST_POSITION_TABLE: [u8; 72] = [
    0xb6, 0x5d, 0xa1, 0x30, 0x69, 0xb5, 0xa3, 0x24, 0x96, 0xac, 0x73, 0x5f, 0x92, 0x48, 0x52, 0x81,
    0x39, 0x95, 0x7f, 0x20, 0x88, 0x5d, 0x34, 0x98, 0xbc, 0xd2, 0x51, 0x77, 0xa2, 0x47, 0x94, 0xb2,
    0x34, 0xda, 0x30, 0x62, 0x9f, 0x76, 0x51, 0x46, 0x98, 0x5c, 0x9b, 0x61, 0x58, 0x95, 0x4c, 0xba,
    0x7e, 0xcb, 0x12, 0xd0, 0x70, 0xa6, 0x46, 0xbf, 0x40, 0x50, 0x7e, 0x8c, 0x2d, 0x61, 0xac, 0x88,
    0x20, 0x6a, 0x72, 0x5f, 0xd2, 0x28, 0x52, 0x80,
];

#[derive(Clone, Copy)]
struct SignedOffset {
    y: i8,
    x: i8,
}

#[derive(Clone, Copy)]
struct UnsignedOffset {
    y: u16,
    x: u16,
}

#[derive(Clone, Copy)]
struct OamTileAttrs {
    char: u8,
    flags: u8,
}

#[derive(Clone, Copy)]
struct QuakeBoltSprite {
    x: i8,
    y: i8,
    flags: u8,
}

macro_rules! quake_bolt_sprites {
    ($($x:literal, $y:literal, $flags:literal),+ $(,)?) => {
        [$(QuakeBoltSprite { x: $x, y: $y, flags: $flags as u8 },)+]
    };
}

macro_rules! signed_offsets {
    ($($y:literal, $x:literal),+ $(,)?) => {
        [$(SignedOffset { y: $y, x: $x },)+]
    };
}

macro_rules! oam_tile_attrs {
    ($($char:literal, $flags:literal),+ $(,)?) => {
        [$(OamTileAttrs { char: $char, flags: $flags },)+]
    };
}

const QUAKE_BOLT_TARGET_PHASES: [u8; 5] = [0x17, 0x16, 0x17, 0x16, 0x10];
const QUAKE_GROUND_BOLT_CHARS: [u8; 15] = [
    0x40, 0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 0x60, 0x62, 0x64, 0x66, 0x68, 0x6a, 0x63,
];
const QUAKE_INITIAL_BOLT_SPRITES: [QuakeBoltSprite; 151] = quake_bolt_sprites![
    0, -16, 0, 0, -16, 1, 0, -16, 2, 0, -16, 3, 0, -16, 67, 0, -16, 66, 0, -16, 65, 0, -16, 64, 0,
    -16, 64, 14, -8, 132, 29, -8, 68, 13, -7, 132, 31, -7, 68, 47, -4, 132, 49, -11, 6, 63, -5, 68,
    47, -4, 132, 36, -17, 8, 49, -11, 6, 63, -5, 68, 78, 4, 8, 22, -31, 8, 36, -17, 8, 78, 4, 8,
    93, 20, 8, 7, -46, 8, 23, -45, 72, 22, -31, 8, 93, 20, 8, 93, 36, 72, -7, -61, 8, 37, -59, 72,
    7, -46, 8, 23, -45, 72, 93, 36, 72, 93, 52, 8, -22, -75, 8, 47, -74, 1, -8, -61, 8, 36, -60,
    72, 93, 52, 8, 108, 67, 8, -37, -90, 8, -22, -75, 8, 47, -74, 1, 59, -62, 129, 108, 67, 8, 121,
    80, 8, -44, -104, 201, -37, -90, 8, 73, -74, 72, 59, -62, 129, 121, 80, 8, -44, -120, 9, -44,
    -104, 201, 87, -89, 72, 73, -74, 72, -44, -120, 9, 102, -104, 72, 87, -89, 72, 102, -104, 72,
    87, -89, 72, 112, -116, 72, 102, -104, 72, 112, -116, 72, -13, -16, 0, -13, -16, 1, -13, -16,
    2, -13, -16, 3, -11, -16, 67, -11, -16, 66, -11, -16, 65, -11, -16, 64, -24, -10, 4, -38, -18,
    8, -24, -10, 4, -40, -7, 196, -45, -33, 201, -38, -18, 8, -57, -7, 4, -40, -7, 196, -48, -45,
    7, -45, -33, 201, -57, -7, 4, -71, 2, 72, -48, -45, 6, -71, 2, 72, -70, 18, 8, -48, -45, 5,
    -70, 18, 8, -56, 33, 8, -48, -45, 7, -54, 34, 8, -54, 49, 136, -48, -45, 6, -54, 49, 136, -69,
    64, 136, -48, -45, 7, -69, 64, 136, -85, 73, 196, -48, -45, 5, -101, 73, 4, -85, 73, 196, -60,
    -53, 8, -48, -45, 6, -101, 73, 4, -116, 77, 196, -75, -67, 8, -60, -53, 8, -128, 76, 4, -116,
    77, 196, -90, -82, 8, -75, -67, 8, -128, 76, 4, -105, -97, 8, -90, -82, 8, -120, -111, 8, -105,
    -97, 8, -120, -111, 8, 0, -5, 10, 0, -5, 11, 2, -3, 12, 1, -3, 13, 0, -3, 141, 1, -3, 140, 1,
    -3, 139, 1, -3, 138, -6, 12, 137, -6, 12, 137, -10, 28, 201, -10, 28, 73, -8, 44, 137, -8, 44,
    137, -10, 56, 2, -10, 56, 2, -23, 70, 72, 5, 70, 8, -23, 70, 72, 5, 70, 8, -38, 85, 72, 19, 85,
    8, -38, 85, 72, 19, 85, 8, -52, 99, 72, 33, 101, 8, -52, 99, 72, 33, 101, 8, -66, 113, 72, 47,
    115, 8, -66, 113, 72, 47, 115, 8,
];
const QUAKE_SPREAD_BOLT_SPRITES: [QuakeBoltSprite; 104] = quake_bolt_sprites![
    -96, 112, 32, -96, 112, 33, -96, 112, 102, -96, 112, 34, -96, 112, 35, -96, 112, 99, -96, 112,
    98, -96, 112, 38, -96, 112, 39, -86, 124, 40, -86, 124, 40, -72, -117, 40, -72, -117, 40, -59,
    -102, 161, -59, -102, 161, -44, -116, 104, -44, -116, 104, -29, 126, 104, -29, 126, 104, -19,
    125, 197, -112, 96, 42, -112, 96, 43, -112, 96, 44, -112, 96, 45, -119, 82, 41, -112, 96, 42,
    -123, 66, 233, -119, 82, 41, -121, 50, 41, -123, 66, 233, 126, 34, 40, -115, 34, 104, -121, 50,
    41, -106, 18, 169, 111, 19, 40, 126, 34, 40, -115, 34, 104, -100, 2, 104, 102, 4, 233, -106,
    18, 169, 111, 19, 40, -91, -14, 169, 95, -11, 40, -100, 2, 104, 102, 4, 233, 96, 112, 96, 96,
    112, 97, 96, 112, 38, 96, 112, 98, 96, 112, 99, 96, 112, 35, 96, 112, 34, 96, 112, 102, 85,
    111, 232, 96, 112, 103, 70, 104, 36, 85, 111, 232, 70, 104, 36, 54, 108, 228, 40, 100, 40, 38,
    107, 36, 54, 108, 228, 25, 85, 40, 40, 100, 40, 38, 107, 36, 22, 110, 228, 11, 70, 40, 25, 85,
    40, 7, 108, 36, 22, 110, 228, 11, 70, 40, 7, 108, 36, 112, 112, 42, 112, 112, 43, 112, 112, 44,
    112, 112, 45, 112, 112, 42, 108, 125, 41, 108, 125, 41, 114, -116, 40, 114, -116, 40, 124,
    -100, 41, 124, -100, 41, 123, -84, 233, 123, -84, 233, 117, -74, 228, -124, -69, 40, 117, -74,
    228, -124, -69, 40, 103, -67, 104, -110, -54, 40, 103, -67, 104, -110, -54, 40, 95, -52, 105,
    -102, -39, 41, 95, -52, 105, -102, -39, 41, 96, -36, 233, -102, -24, 233, 96, -36, 233, -102,
    -24, 233, -123, -14, 41, -115, -14, 46, 49, -12, 40,
];
const QUAKE_INITIAL_BOLT_FRAME_RANGES: [u8; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 17, 21, 25, 30, 36, 42, 48, 53, 57, 60, 62, 64, 65, 66,
    67, 68, 69, 70, 71, 72, 74, 77, 81, 85, 88, 91, 94, 97, 100, 103, 107, 111, 114, 116, 118, 119,
    120, 121, 122, 123, 124, 125, 126, 128, 130, 132, 134, 137, 141, 145, 149, 151,
];
const QUAKE_SPREAD_BOLT_FRAME_RANGES: [u8; 56] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 18, 19, 20, 21, 22, 23, 24, 26, 28, 30, 33, 37, 41,
    45, 46, 47, 48, 49, 50, 51, 52, 53, 55, 57, 59, 62, 66, 70, 72, 73, 74, 75, 76, 78, 80, 82, 84,
    87, 91, 95, 99, 101, 104,
];
const RECEIVE_ITEM_MILESTONE_FRAME_TIMERS: [u8; 3] = [9, 5, 5];
const RECEIVE_ITEM_MILESTONE_GFX_SOURCES: [u8; 3] = [0x24, 0x25, 0x26];
const RECEIVE_ITEM_CRYSTAL_FRAME_SEQUENCE: [u8; 3] = [5, 1, 4];
const RECEIVE_ITEM_MESSAGES: [i16; 76] = [
    -1, 0x70, 0x77, 0x52, -1, 0x78, 0x78, 0x62, 0x61, 0x66, 0x69, 0x53, 0x52, 0x56, -1, 0x64, 0x63,
    0x65, 0x51, 0x54, 0x67, 0x68, 0x6b, 0x77, 0x79, 0x55, 0x6e, 0x58, 0x6d, 0x5d, 0x57, 0x5e, -1,
    0x74, 0x75, 0x76, -1, 0x5f, 0x158, -1, 0x6a, 0x5c, 0x8f, 0x71, 0x72, 0x73, 0x71, 0x72, 0x73,
    0x6a, 0x6c, 0x60, -1, -1, -1, 0x59, 0x84, 0x5a, -1, -1, -1, -1, -1, 0x159, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 0xdb, 0x67, 0x7c,
];
const RECEIVE_ITEM_SPECIAL_MESSAGES: [i16; 2] = [0x5b, 0x83];
const RECEIVE_ITEM_HEART_PIECE_MESSAGES: [i16; 4] = [-1, 0x155, 0x156, 0x157];
const BOMB_PHASE_TIMERS: [u8; 11] = [0xa0, 6, 4, 4, 4, 4, 4, 6, 6, 6, 6];
const BOMB_DRAW_FRAME_STARTS: [u8; 12] = [0, 1, 2, 3, 2, 3, 4, 5, 6, 7, 8, 9];
const BOMB_DRAW_FRAME_COUNTS: [u8; 11] = [1, 4, 4, 4, 4, 4, 5, 4, 6, 6, 6];

const ANCILLA_DRAW_SPRITE_COUNTS: [u8; 68] = [
    0, 8, 0x0c, 0x10, 0x10, 4, 0x10, 0x18, 8, 8, 8, 0, 0x14, 0, 0x10, 0x28, 0x18, 0x10, 0x10, 0x10,
    0x10, 0x0c, 8, 8, 0x50, 0, 0x10, 8, 0x40, 0, 0x0c, 0x24, 0x10, 0x0c, 8, 0x10, 0x10, 4, 0x0c,
    0x1c, 0, 0x10, 0x14, 0x14, 0x10, 8, 0x20, 0x10, 0x10, 0x10, 4, 0, 0x80, 0x10, 4, 0x30, 0x14,
    0x10, 0, 0x10, 0, 0, 8, 0, 0x10, 8, 0x78, 0x80,
];

const RECEIVE_ITEM_GRAPHICS: [u8; 76] = [
    6, 0x18, 0x18, 0x18, 0x2d, 0x20, 0x2e, 9, 9, 0x0a, 8, 5, 0x10, 0x0b, 0x2c, 0x1b, 0x1a, 0x1c,
    0x14, 0x19, 0x0c, 7, 0x1d, 0x2f, 7, 0x15, 0x12, 0x0d, 0x0d, 0x0e, 0x11, 0x17, 0x28, 0x27, 4, 4,
    0x0f, 0x16, 3, 0x13, 1, 0x1e, 0x10, 0, 0, 0, 0, 0, 0, 0x30, 0x22, 0x21, 0x24, 0x24, 0x24, 0x23,
    0x23, 0x23, 0x29, 0x2a, 0x2c, 0x2b, 3, 3, 0x34, 0x35, 0x31, 0x33, 2, 0x32, 0x36, 0x37, 0x2c, 6,
    0x0c, 0x38,
];
const RECEIVE_ITEM_OAM_EXT_SIZES: [u8; 76] = [
    0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
];
const WISH_POND_ITEM_OAM_FLAGS: [u8; 76] = [
    5, 0xff, 5, 5, 5, 5, 5, 1, 2, 1, 1, 1, 2, 2, 2, 4, 4, 4, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1, 4, 4, 2,
    1, 6, 1, 2, 1, 2, 2, 1, 2, 2, 4, 1, 1, 4, 2, 1, 4, 2, 2, 4, 4, 4, 2, 1, 4, 1, 2, 2, 1, 2, 2, 1,
    1, 4, 4, 1, 2, 2, 4, 4, 4, 2, 5, 2, 1,
];
const TRAVEL_BIRD_DMA_TILE_OFFSETS: [u8; 4] = [0, 0x20, 0x40, 0xe0];
const TRAVEL_BIRD_DRAW_X_OFFSETS: [i8; 3] = [0, -9, -9];
const TRAVEL_BIRD_DRAW_Y_OFFSETS: [i8; 3] = [0, 12, 20];
const TRAVEL_BIRD_DRAW_CHARS: [u8; 3] = [0x0e, 0, 2];
const TRAVEL_BIRD_DRAW_FLAGS: [u8; 3] = [0x22, 0x2e, 0x2e];

const ANCILLA_OVERWORLD_AREA_BASE_X: [u16; 64] = [
    0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00,
    0, 0x200, 0x400, 0x600, 0x800, 0xa00, 0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00,
    0xc00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0x200, 0x400, 0x600, 0x800, 0xa00,
    0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00,
    0xa00, 0xe00,
];

const ANCILLA_OVERWORLD_AREA_BASE_Y: [u16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x200, 0, 0, 0, 0, 0x200, 0x400, 0x400, 0x400, 0x400, 0x400,
    0x400, 0x400, 0x400, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600,
    0x800, 0x600, 0x600, 0x800, 0x600, 0x600, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00,
    0xa00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xe00, 0xe00,
    0xe00, 0xc00, 0xc00, 0xe00,
];

const MAGIC_POWDER_FRAME_TIMERS: [u8; 40] = [
    13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 10, 11, 12, 0, 1, 2, 3, 4, 5, 6, 16, 17, 18, 0, 1, 2, 3, 4, 5,
    6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6,
];

#[rustfmt::skip]
const ANCILLA_TILE_COLLISION_ATTRS: [u8; 256] = [
    0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4,
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 3, 3, 3,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

#[rustfmt::skip]
const ANCILLA_TILE_COLLISION_ATTRS_LAYER0: [u8; 256] = [
    0, 1, 0, 3, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4,
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 3, 3, 3,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

const SLOPED_TILE_HEIGHTS: [u8; 32] = [
    7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0,
];

const FIRE_ROD_SPARK_X_VELOCITIES: [i8; 12] = [0, 0, -40, 40, 0, 0, -48, 48, 0, 0, -64, 64];
const FIRE_ROD_SPARK_Y_VELOCITIES: [i8; 12] = [-40, 40, 0, 0, -48, 48, 0, 0, -64, 64, 0, 0];

struct CheckPlayerCollOut {
    r4: u16,
    r6: u16,
    r8: u16,
    r10: u16,
}

struct AncillaOamInfo {
    x: u8,
    y: u8,
    flags: u8,
}

impl ZeldaState {
    fn replay_ancilla_trace_enabled(&self) -> bool {
        if std::env::var_os("ZELDA3_REPLAY_ANCILLA_TRACE").is_none() {
            return false;
        }
        std::env::var("ZELDA3_REPLAY_ANCILLA_TRACE_FRAME")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .is_none_or(|target| self.state_recorder.replay_frame_counter == target)
    }

    pub(super) fn ancilla_main(&mut self) {
        self.ancilla_weapon_tink();
        self.ancilla_execute_all();
    }

    fn ancilla_weapon_tink(&mut self) {
        if self.garnish_state_view().repulsespark_timer() == 0 {
            return;
        }
        self.sprite_system_view_mut().set_alert_flag(2);
        let anim_delay = self
            .garnish_state_view_mut()
            .decrement_repulsespark_anim_delay();
        if sign8(anim_delay) {
            self.garnish_state_view_mut().decrement_repulsespark_timer();
            self.garnish_state_view_mut().set_repulsespark_anim_delay(1);
        }

        if self.oam_state_view().has_sprite_sorting() {
            if self.garnish_state_view().repulsespark_floor_status() != 0 {
                self.oam_allocate_from_region_f(0x10);
            } else {
                self.oam_allocate_from_region_d(0x10);
            }
        } else {
            self.oam_allocate_from_region_a(0x10);
        }

        let x = self
            .garnish_state_view()
            .repulsespark_x_lo()
            .wrapping_sub(self.world_scroll().bg2_x_low());
        let y = self
            .garnish_state_view()
            .repulsespark_y_lo()
            .wrapping_sub(self.world_scroll().bg2_y_low());
        if x >= 0xf8 || y >= 0xf0 {
            self.garnish_state_view_mut().clear_repulsespark_timer();
            return;
        }

        let oam = self.oam_state_view().current_pointer_usize();
        let oam_idx = (oam - OAM_BUF) / 4;
        const REPULSE_SPARK_FLAGS: [u8; 4] = [0x22, 0x12, 0x22, 0x22];
        let flags =
            REPULSE_SPARK_FLAGS[self.garnish_state_view().repulsespark_floor_status() as usize];
        if self.garnish_state_view().repulsespark_timer() >= 3 {
            self.set_oam_plain(
                oam_idx,
                x,
                y,
                if self.garnish_state_view().repulsespark_timer() < 9 {
                    0x92
                } else {
                    0x80
                },
                flags,
                0,
            );
            return;
        }

        const REPULSE_SPARK_CHAR: [u8; 3] = [0x93, 0x82, 0x81];
        let c = REPULSE_SPARK_CHAR[self.garnish_state_view().repulsespark_timer() as usize];
        self.set_oam_plain(oam_idx, x.wrapping_sub(4), y.wrapping_sub(4), c, flags, 0);
        self.set_oam_plain(
            oam_idx + 1,
            x.wrapping_add(4),
            y.wrapping_sub(4),
            c,
            flags | 0x40,
            0,
        );
        self.set_oam_plain(
            oam_idx + 2,
            x.wrapping_sub(4),
            y.wrapping_add(4),
            c,
            flags | 0x80,
            0,
        );
        self.set_oam_plain(
            oam_idx + 3,
            x.wrapping_add(4),
            y.wrapping_add(4),
            c,
            flags | 0xc0,
            0,
        );
    }

    fn ancilla_empty(&mut self, _k: usize) {}

    fn ancilla_unused_14(&mut self, _k: usize) {
        // Ancilla_Unused_14 is an assert-only dispatch slot in the C port.
        panic!("Ancilla_Unused_14");
    }

    fn ancilla_unused_25(&mut self, _k: usize) {
        // Ancilla_Unused_25 is an assert-only dispatch slot in the C port.
        panic!("Ancilla_Unused_25");
    }

    fn ancilla_execute_all(&mut self) {
        for i in (0..10).rev() {
            self.sprite_system_view_mut().set_cur_object_index(i as u8);
            let ty = self.ancilla_slot_view(i).ancilla_type();
            if ty != 0 {
                let ancilla = self.ancilla_slot_view(i);
                self.replay_trace_ram_watch(&format!(
                    "ancilla-before-execute-one ancilla={i} type=0x{:02x} timer=0x{:02x} item=0x{:02x} work3=0x{:02x} floor=0x{:02x} num={}",
                    ty,
                    ancilla.timer(),
                    ancilla.item_to_link(),
                    ancilla.work_byte_3(),
                    ancilla.floor(),
                    ancilla.num_sprites(),
                ));
                self.ancilla_execute_one(ty, i);
                let ancilla = self.ancilla_slot_view(i);
                self.replay_trace_ram_watch(&format!(
                    "ancilla-after-execute-one ancilla={i} type=0x{:02x} timer=0x{:02x} item=0x{:02x} work3=0x{:02x} floor=0x{:02x} num={}",
                    self.ancilla_slot_view(i).ancilla_type(),
                    ancilla.timer(),
                    ancilla.item_to_link(),
                    ancilla.work_byte_3(),
                    ancilla.floor(),
                    ancilla.num_sprites(),
                ));
            }
        }
    }

    fn ancilla_execute_one(&mut self, ty: u8, k: usize) {
        if k < 6 {
            let num_sprites = self.ancilla_slot_view(k).num_sprites();
            let idx = self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, num_sprites);
            self.ancilla_slot_view_mut(k).set_oam_index(idx as u8);
        }

        if self.frame_state().submodule == 0 && self.ancilla_slot_view(k).timer() != 0 {
            self.ancilla_slot_view_mut(k).tick_timer();
        }

        match ty {
            0x01 => self.ancilla01_somaria_bullet(k),
            0x02 => self.ancilla02_fire_rod_shot(k),
            0x03 => self.ancilla_empty(k),
            0x04 => self.ancilla04_beam_hit(k),
            0x05 => self.ancilla05_boomerang(k),
            0x06 => self.ancilla06_wall_hit(k),
            0x07 => self.ancilla07_bomb(k),
            0x08 => self.ancilla08_door_debris(k),
            0x09 => self.ancilla09_arrow(k),
            0x0a => self.ancilla0_a_arrow_in_the_wall(k),
            0x0b => self.ancilla0_b_ice_rod_shot(k),
            0x0c => self.ancilla_sword_beam(k),
            0x0d => self.ancilla0_d_spin_attack_full_charge_spark(k),
            0x0e..=0x10 => self.ancilla33_blast_wall_explosion(k),
            0x11 => self.ancilla11_ice_rod_wall_hit(k),
            0x12 => self.ancilla33_blast_wall_explosion(k),
            0x13 => self.ancilla13_ice_rod_sparkle(k),
            0x14 => self.ancilla_unused_14(k),
            0x15 => self.ancilla15_jump_splash(k),
            0x16 => self.ancilla16_hit_stars(k),
            0x17 => self.ancilla17_shovel_dirt(k),
            0x18 => self.ancilla18_ether_spell(k),
            0x19 => self.ancilla19_bombos_spell(k),
            0x1a => self.ancilla1_a_powder_dust(k),
            0x1b => self.ancilla_sword_wall_hit(k),
            0x1c => self.ancilla1_c_quake_spell(k),
            0x1d => self.ancilla1_d_screen_shake(k),
            0x1e => self.ancilla1_e_dash_dust(k),
            0x1f => self.ancilla1_f_hookshot(k),
            0x20 => self.ancilla20_blanket(k),
            0x21 => self.ancilla21_snore(k),
            0x22 => self.ancilla22_item_receipt(k),
            0x23 => self.ancilla23_link_poof(k),
            0x24 => self.ancilla24_gravestone(k),
            0x25 => self.ancilla_unused_25(k),
            0x26 => self.ancilla26_sword_swing_sparkle(k),
            0x27 => self.ancilla27_duck(k),
            0x28 => self.ancilla28_wish_pond_item(k),
            0x29 => self.ancilla29_milestone_item_receipt(k),
            0x2a => self.ancilla2_a_spin_attack_sparkle_a(k),
            0x2b => self.ancilla2_b_spin_attack_sparkle_b(k),
            0x2c => self.ancilla2_c_somaria_block(k),
            0x2d => self.ancilla2_d_somaria_block_fizz(k),
            0x2e => self.ancilla2_e_somaria_block_fission(k),
            0x2f => self.ancilla2_f_lamp_flame(k),
            0x30 => self.ancilla30_byrna_windup_spark(k),
            0x31 => self.ancilla31_byrna_spark(k),
            0x32 => self.ancilla32_blast_wall_fireball(k),
            0x33 => self.ancilla33_blast_wall_explosion(k),
            0x34 => self.ancilla34_skull_woods_fire(k),
            0x35 => self.ancilla35_master_sword_receipt(k),
            0x36 => self.ancilla36_flute(k),
            0x37 => self.ancilla37_weathervane_explosion(k),
            0x39 => self.ancilla39_somaria_platform_poof(k),
            0x38 => self.ancilla38_cutscene_duck(k),
            0x3a => self.ancilla3_a_big_bomb_explosion(k),
            0x3b => self.ancilla3_b_sword_up_sparkle(k),
            0x3c => self.ancilla3_c_spin_attack_charge_sparkle(k),
            0x3d => self.ancilla3_d_item_splash(k),
            0x3e => self.ancilla_rising_crystal(k),
            0x3f => self.ancilla3_f_bush_poof(k),
            0x40 => self.ancilla40_dwarf_poof(k),
            0x41 => self.ancilla41_waterfall_splash(k),
            0x42 => self.ancilla42_happiness_pond_rupees(k),
            0x43 => self.ancilla43_ganons_tower_cutscene(k),
            _ => {}
        }
    }

    pub(super) fn ancilla_add_blanket(&mut self, a: u8) {
        let k = 0;
        let floor = self.player_state_view().lower_level_state();
        let mirror_floor = self.player_state_view().lower_level_mirror_state();
        {
            let mut blanket = self.ancilla_slot_view_mut(k);
            blanket.set_ancilla_type(a);
            blanket.set_num_sprites(ANCILLA_DRAW_SPRITE_COUNTS[a as usize]);
            blanket.set_floor(floor);
            blanket.set_floor2(mirror_floor);
            blanket.set_object_priority(0);
        }
        self.ancilla_set_xy(k, 0x0938, 0x2162);
    }

    pub(super) fn ancilla_add_cape_poof(&mut self, ty: u8, limit: u8) {
        if let Some(k) = self.ancilla_add_simple(ty, limit) {
            self.player_state_view_mut().set_transforming();
            self.player_state_view_mut().set_direction_lock_bits(1);
            self.player_state_view_mut().set_direction(0);
            self.player_state_view_mut().set_last_direction(0);
            {
                let mut cape_poof = self.ancilla_slot_view_mut(k);
                cape_poof.set_step(1);
                cape_poof.set_item_to_link(0);
                cape_poof.set_aux_timer(7);
            }
            let x = self.player_state_view().x();
            let y = self.player_state_view().y().wrapping_add(4);
            self.ancilla_set_xy(k, x, y);
        }
    }

    pub(super) fn ancilla_add_hit_stars(&mut self, a: u8, y: u8) {
        const SHOVEL_HIT_STARS_OFFSET: [SignedOffset; 6] = [
            SignedOffset { y: 21, x: -11 },
            SignedOffset { y: 21, x: 11 },
            SignedOffset { y: 3, x: -6 },
            SignedOffset { y: 21, x: 5 },
            SignedOffset { y: 16, x: -14 },
            SignedOffset { y: 16, x: 14 },
        ];
        const SHOVEL_HIT_STARS_X2: [i8; 6] = [-3, 19, 2, 13, -6, 22];

        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_aux_timer(2);
                ancilla.set_work_byte_3(1);
                ancilla.set_item_to_link(0);
                ancilla.set_y_velocity(0);
                ancilla.set_x_velocity(0);
            }

            let mut j = a;
            let player = self.player_state_view();
            if player.has_item_in_hand() {
                j = (player.facing() >> 1).wrapping_add(2);
            } else if player.has_position_mode() {
                j = if player.facing() != 4 { 1 } else { 0 };
            }

            self.ancilla_slot_view_mut(k).set_step(j);
            let j = j as usize;
            let link_x = self.player_state_view().x();
            let link_y = self.player_state_view().y();
            let t = link_x.wrapping_add(SHOVEL_HIT_STARS_X2[j] as i16 as u16);
            let value = t as u8;
            self.ancilla_slot_view_mut(k).set_a(value);
            let value = (t >> 8) as u8;
            self.ancilla_slot_view_mut(k).set_b(value);
            let offset = SHOVEL_HIT_STARS_OFFSET[j];
            self.ancilla_set_xy(
                k,
                link_x.wrapping_add(offset.x as i16 as u16),
                link_y.wrapping_add(offset.y as i16 as u16),
            );
        }
    }

    pub(super) fn ancilla_add_fire_rod_shot(&mut self, type_: u8, _y: u8) {
        const FIRE_ROD_X: [i8; 4] = [0, 0, -8, 16];
        const FIRE_ROD_Y: [i8; 4] = [-8, 16, 3, 3];
        const FIRE_ROD_XVEL: [i8; 4] = [0, 0, -64, 64];
        const FIRE_ROD_YVEL: [i8; 4] = [-64, 64, 0, 0];

        let y = 1;
        let Some(mut j) = self.ancilla_alloc_init(type_, y) else {
            if type_ != 1 {
                self.refund_magic(0);
            }
            return;
        };

        if type_ != 1 {
            self.ancilla_sfx2_near(0x0e);
        }

        let mut i = self.player_state_view().facing_index();
        {
            let mut ancilla = self.ancilla_slot_view_mut(j);
            ancilla.set_ancilla_type(type_);
            ancilla.set_num_sprites(ANCILLA_DRAW_SPRITE_COUNTS[type_ as usize]);
            ancilla.set_object_priority(0);
            ancilla.set_u(0);
            ancilla.set_step(0);
            ancilla.set_timer(3);
            ancilla.set_item_to_link(0);
            ancilla.set_direction(i as u8);
        }

        if self.ancilla_check_initial_tile_a(j) < 0 {
            self.ancilla_set_xy(
                j,
                self.player_state_view()
                    .x()
                    .wrapping_add(FIRE_ROD_X[i] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(FIRE_ROD_Y[i] as i16 as u16),
            );
            if type_ != 1 {
                let mut ancilla = self.ancilla_slot_view_mut(j);
                ancilla.set_x_velocity(FIRE_ROD_XVEL[i] as u8);
                ancilla.set_y_velocity(FIRE_ROD_YVEL[i] as u8);
            } else {
                i += self.inventory_items().sword_type().wrapping_sub(2) as usize * 4;
                let mut ancilla = self.ancilla_slot_view_mut(j);
                ancilla.set_x_velocity(FIRE_ROD_SPARK_X_VELOCITIES[i] as u8);
                ancilla.set_y_velocity(FIRE_ROD_SPARK_Y_VELOCITIES[i] as u8);
            }
            let floor = self.player_state_view().lower_level_state();
            let mirror_floor = self.player_state_view().lower_level_mirror_state();
            let mut ancilla = self.ancilla_slot_view_mut(j);
            ancilla.set_floor(floor);
            ancilla.set_floor2(mirror_floor);
        } else if type_ == 1 {
            let mut ancilla = self.ancilla_slot_view_mut(j);
            ancilla.set_ancilla_type(4);
            ancilla.set_timer(7);
            ancilla.set_num_sprites(16);
        } else {
            let mut ancilla = self.ancilla_slot_view_mut(j);
            ancilla.set_step(1);
            ancilla.set_timer(31);
            ancilla.set_num_sprites(8);
            j = self.player_state_view().facing_index();
            self.ancilla_sfx2_pan(j, 0x2a);
        }
    }

    pub(super) fn ancilla_add_falling_prize(&mut self, a: u8, item_idx: u8, yv: u8) -> i32 {
        const FALLING_ITEM_TYPE: [u8; 7] = [0x10, 0x37, 0x39, 0x38, 0x26, 0x0f, 0x20];
        const FALLING_ITEM_G: [u8; 7] = [0x40, 0, 0, 0, 0, 0xff, 0];
        const FALLING_ITEM_X: [u16; 7] = [0x78, 0x78, 0x78, 0x78, 0x78, 0x80, 0x78];
        const FALLING_ITEM_Y: [u16; 7] = [0x48, 0x78, 0x78, 0x78, 0x78, 0x68, 0x78];
        const FALLING_ITEM_Z: [u8; 7] = [0x60, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];

        self.player_state_view_mut()
            .set_receive_item_index(item_idx);
        let Some(k) = self.ancilla_add_simple(a, yv) else {
            return -1;
        };
        let item_type = FALLING_ITEM_TYPE[item_idx as usize];
        if item_type == 0x10 || item_type == 0x0f {
            self.DecodeAnimatedSpriteTile_variable(RECEIVE_ITEM_GRAPHICS[item_type as usize]);
        }

        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_item_to_link(item_type);
            ancilla.set_z_velocity((-48i8) as u8);
            ancilla.set_y_velocity(0);
            ancilla.set_x_velocity(0);
            ancilla.set_z(FALLING_ITEM_Z[item_idx as usize]);
            ancilla.set_step(0);
            ancilla.set_aux_timer(9);
            ancilla.set_work_byte_3(0);
            ancilla.set_l(0);
            ancilla.set_g(FALLING_ITEM_G[item_idx as usize]);
        }
        self.player_state_view_mut()
            .set_receive_item_index(item_type);

        let (x, y) = if item_idx != 0 && item_idx != 5 {
            if self.save_progress_view().palace_index_x2() == 20 {
                (
                    (self.player_state_view().x() & 0xff00) | 0x0100,
                    (self.player_state_view().y() & 0xff00) | 0x0100,
                )
            } else {
                (
                    FALLING_ITEM_X[item_idx as usize].wrapping_add(self.world_scroll().bg2_x()),
                    FALLING_ITEM_Y[item_idx as usize].wrapping_add(self.world_scroll().bg2_y()),
                )
            }
        } else {
            (
                self.player_state_view().x(),
                FALLING_ITEM_Y[item_idx as usize].wrapping_add(self.world_scroll().bg2_y()),
            )
        };
        self.ancilla_set_xy(k, x, y);
        k as i32
    }

    pub(super) fn add_sword_beam(&mut self, y: u8) {
        const SWORD_BEAM_X: [i8; 4] = [-8, -10, -22, 4];
        const SWORD_BEAM_Y: [i8; 4] = [-24, 8, -6, -6];
        const SWORD_BEAM_S: [i8; 4] = [-8, -8, -8, 8];
        const SWORD_BEAM_TRAILING_ANGLES: [u8; 16] = [
            0x21, 0x1d, 0x19, 0x15, 3, 0x3e, 0x3a, 0x36, 0x12, 0x0e, 0x0a, 6, 0x31, 0x2d, 0x29,
            0x25,
        ];
        const SWORD_BEAM_YVEL: [i8; 4] = [-64, 64, 0, 0];
        const SWORD_BEAM_XVEL: [i8; 4] = [0, 0, -64, 64];

        let Some(k) = self.ancilla_add_simple(0x0c, y) else {
            return;
        };
        let mut j = self.player_state_view().facing() as usize * 2;
        self.effect_angle_scratch_view_mut()
            .set_angles4(&SWORD_BEAM_TRAILING_ANGLES, j);
        self.effect_angle_scratch_view_mut()
            .set_trailing_angle(SWORD_BEAM_TRAILING_ANGLES[j + 3]);
        self.effect_angle_scratch_view_mut().set_radial_radius(14);
        j = self.player_state_view().facing_index();
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_l(0);
            ancilla.set_g(0);
            ancilla.set_work_byte_1(0);
            ancilla.set_aux_timer(2);
            ancilla.set_work_byte_3(8);
            ancilla.set_step(0);
            ancilla.set_item_to_link(0x4c);
            ancilla.set_direction(j as u8);
            ancilla.set_y_velocity(SWORD_BEAM_YVEL[j] as u8);
            ancilla.set_x_velocity(SWORD_BEAM_XVEL[j] as u8);
            ancilla.set_s_player(SWORD_BEAM_S[j] as u8);
        }

        let swordbeam_temp_y = self.player_state_view().y().wrapping_add(12);
        let swordbeam_temp_x = self.player_state_view().x().wrapping_add(8);
        self.ether_orbit_view_mut()
            .set_swordbeam_temp(swordbeam_temp_x, swordbeam_temp_y);

        if self.ancilla_check_initial_tile_a(k) >= 0 {
            self.ancilla_set_xy(
                k,
                swordbeam_temp_x.wrapping_add(SWORD_BEAM_X[j] as i16 as u16),
                swordbeam_temp_y.wrapping_add(SWORD_BEAM_Y[j] as i16 as u16),
            );
            self.set_sound_effect_2_with_ancilla_pan(k, 1);
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_ancilla_type(4);
            ancilla.set_timer(7);
            ancilla.set_num_sprites(16);
        }
    }

    pub(super) fn ancilla_spawn_sword_charge_sparkle(&mut self) {
        const SWORD_CHARGE_SPARKLE_A: [u8; 4] = [0, 0, 7, 7];
        const SWORD_CHARGE_SPARKLE_B: [u8; 4] = [0x70, 0x70, 0, 0];
        const SWORD_CHARGE_SPARKLE_X: [u8; 4] = [0, 3, 4, 5];
        const SWORD_CHARGE_SPARKLE_Y: [u8; 4] = [5, 12, 8, 8];

        let Some(k) = self.ancilla_alloc_high() else {
            return;
        };
        {
            let mut sparkle = self.ancilla_slot_view_mut(k);
            sparkle.set_ancilla_type(0x3c);
            sparkle.set_item_to_link(0);
            sparkle.set_timer(4);
        }
        let floor = self.player_state_view().lower_level_state();
        self.ancilla_slot_view_mut(k).set_floor(floor);
        let j = self.player_state_view().facing_index();
        let mut x = 0i8;
        let mut y = 0i8;
        let m0 = SWORD_CHARGE_SPARKLE_A[j];
        if m0 == 0 {
            y = (self.player_state_view().spin_attack_step_counter() >> 2) as i8;
            if j == 0 {
                y = -y;
            }
        }
        let m1 = SWORD_CHARGE_SPARKLE_B[j];
        if m1 == 0 {
            x = (self.player_state_view().spin_attack_step_counter() >> 2) as i8;
            if j == 2 {
                x = -x;
            }
        }
        let r = self.get_random_number();
        let dst_x = self
            .player_state_view()
            .x()
            .wrapping_add(x as i16 as u16)
            .wrapping_add(SWORD_CHARGE_SPARKLE_X[j] as u16)
            .wrapping_add(((r & m1) >> 4) as u16);
        let dst_y = self
            .player_state_view()
            .y()
            .wrapping_add(y as i16 as u16)
            .wrapping_add(SWORD_CHARGE_SPARKLE_Y[j] as u16)
            .wrapping_add((r & m0) as u16);
        if self.replay_ancilla_trace_enabled() {
            println!(
                "ancilla-trace kind=spawn-charge abs={} fc=0x{:02x} dst={} rng=0x{:02x} j={} off=0x{:02x}/0x{:02x} mask=0x{:02x}/0x{:02x} xy=0x{:04x}/0x{:04x} link=0x{:04x}/0x{:04x} face=0x{:02x} spin=0x{:02x} speed=0x{:02x}/0x{:02x} type=0x{:02x} timer=0x{:02x} floor=0x{:02x}",
                self.state_recorder.replay_frame_counter,
                self.frame_state().frame_counter,
                k,
                r,
                j,
                x as u8,
                y as u8,
                m1,
                m0,
                dst_x,
                dst_y,
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.player_state_view().facing(),
                self.player_state_view().spin_attack_step_counter(),
                self.player_state_view().actual_x_velocity(),
                self.player_state_view().actual_y_velocity(),
                self.ancilla_slot_view(k).ancilla_type(),
                self.ancilla_slot_view(k).timer(),
                self.ancilla_slot_view(k).floor(),
            );
        }
        self.ancilla_set_xy(k, dst_x, dst_y);
    }

    pub(super) fn ancilla_add_sword_charge_sparkle_from_ancilla(&mut self, source: usize) {
        let Some(k) = self.ancilla_alloc_high() else {
            return;
        };
        {
            let mut sparkle = self.ancilla_slot_view_mut(k);
            sparkle.set_ancilla_type(60);
            sparkle.set_item_to_link(0);
            sparkle.set_timer(4);
        }
        let floor = self.player_state_view().lower_level_state();
        self.ancilla_slot_view_mut(k).set_floor(floor);

        let rand = self.get_random_number();
        let mut z = self.ancilla_slot_view(source).z();
        if z >= 0xf8 {
            z = 0;
        }
        self.ancilla_set_xy(
            k,
            self.ancilla_get_x(source)
                .wrapping_add(2)
                .wrapping_add(u16::from(rand >> 5)),
            self.ancilla_get_y(source)
                .wrapping_sub(2)
                .wrapping_sub(u16::from(z))
                .wrapping_add(u16::from(rand & 0x0f)),
        );
    }

    fn add_dashing_dust_ex(&mut self, a: u8, y: u8, flag: u8) {
        const ADD_DASHING_DUST_X: [i8; 4] = [4, 4, 6, 0];
        const ADD_DASHING_DUST_Y: [i8; 4] = [20, 4, 16, 16];
        if let Some(k) = self.ancilla_add_simple(a, y) {
            let j = self.player_state_view().facing_index();
            {
                let mut dust = self.ancilla_slot_view_mut(k);
                dust.set_step(flag);
                dust.set_item_to_link(0);
                dust.set_timer(3);
                dust.set_direction(j as u8);
            }
            if flag == 0 {
                self.ancilla_set_xy(
                    k,
                    self.player_state_view().x(),
                    self.player_state_view().y().wrapping_add(20),
                );
            } else {
                self.ancilla_set_xy(
                    k,
                    self.player_state_view()
                        .x()
                        .wrapping_add(ADD_DASHING_DUST_X[j] as i16 as u16),
                    self.player_state_view()
                        .y()
                        .wrapping_add(ADD_DASHING_DUST_Y[j] as i16 as u16),
                );
            }
        }
    }

    pub(super) fn ancilla_add_dash_dust(&mut self, a: u8, y: u8) {
        self.add_dashing_dust_ex(a, y, 1);
    }

    pub(super) fn ancilla_add_dash_dust_charging(&mut self, a: u8, y: u8) {
        self.add_dashing_dust_ex(a, y, 0);
    }

    fn ancilla_add_blast_wall_fireball(&mut self, _a: u8, _y: u8, r4: usize) {
        const BLAST_WALL_FIREBALL_VELOCITY: [SignedOffset; 16] = [
            SignedOffset { y: -64, x: 0 },
            SignedOffset { y: -22, x: 42 },
            SignedOffset { y: -38, x: 38 },
            SignedOffset { y: -42, x: 22 },
            SignedOffset { y: 0, x: 64 },
            SignedOffset { y: 22, x: 42 },
            SignedOffset { y: 38, x: 38 },
            SignedOffset { y: 42, x: 22 },
            SignedOffset { y: 64, x: 0 },
            SignedOffset { y: 22, x: -42 },
            SignedOffset { y: 38, x: -38 },
            SignedOffset { y: 42, x: -22 },
            SignedOffset { y: 0, x: -64 },
            SignedOffset { y: -22, x: -42 },
            SignedOffset { y: -38, x: -38 },
            SignedOffset { y: -42, x: -22 },
        ];

        for k in (5..=10).rev() {
            if self.ancilla_slot_view(k).ancilla_type() == 0 {
                let floor = self.player_state_view().lower_level_state();
                {
                    let mut ancilla = self.ancilla_slot_view_mut(k);
                    ancilla.set_ancilla_type(0x32);
                    ancilla.set_floor(floor);
                }
                self.blast_wall_fireball_view_mut(k).set_timer(16);
                let j = (self.frame_state().frame_counter & 15) as usize;
                let velocity = BLAST_WALL_FIREBALL_VELOCITY[j];
                {
                    let mut ancilla = self.ancilla_slot_view_mut(k);
                    ancilla.set_y_velocity(velocity.y as u8);
                    ancilla.set_x_velocity(velocity.x as u8);
                }
                self.ancilla_set_xy(
                    k,
                    self.blast_wall_fragment_view(r4).x().wrapping_add(16),
                    self.blast_wall_fragment_view(r4).y().wrapping_add(8),
                );
                return;
            }
        }
    }

    pub(super) fn ancilla_add_arrow(
        &mut self,
        a: u8,
        ax: u8,
        ay: u8,
        xcoord: u16,
        ycoord: u16,
    ) -> i32 {
        const SHOOT_BOW_X: [i8; 4] = [4, 4, 0, 4];
        const SHOOT_BOW_Y: [i8; 4] = [-4, 3, 4, 4];
        const SHOOT_BOW_XVEL: [i8; 4] = [0, 0, -48, 48];
        const SHOOT_BOW_YVEL: [i8; 4] = [-48, 48, 0, 0];

        self.tile_detect_position_view_mut()
            .set_interaction_scratch_y(ycoord);
        self.tile_detect_position_view_mut()
            .set_interaction_scratch_x(xcoord);
        self.tile_detect_position_view_mut()
            .set_interacting_tile_low(ax);

        if self.ancilla_add_check_for_presence(a) {
            return -1;
        }

        let k = self.ancilla_add_arrow_find_slot(a, ay);

        if k >= 0 {
            let k = k as usize;
            self.set_sound_effect_1_with_link_pan(7);
            let j = (ax >> 1) as usize;
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_h(0);
                ancilla.set_item_to_link(8);
                ancilla.set_direction(j as u8 | 4);
                ancilla.set_y_velocity(SHOOT_BOW_YVEL[j] as u8);
                ancilla.set_x_velocity(SHOOT_BOW_XVEL[j] as u8);
            }
            self.ancilla_set_xy(
                k,
                xcoord.wrapping_add(SHOOT_BOW_X[j] as i16 as u16),
                ycoord
                    .wrapping_add(8)
                    .wrapping_add(SHOOT_BOW_Y[j] as i16 as u16),
            );
        }
        k
    }

    fn ancilla_add_arrow_find_slot(&mut self, type_: u8, ay: u8) -> i32 {
        let mut n = 0;
        for k in (0..=4).rev() {
            if self.ancilla_slot_view(k).ancilla_type() == 10 {
                n += 1;
            }
        }

        let mut k = -1;
        if n != ay.wrapping_add(1) {
            for i in (0..=4).rev() {
                if self.ancilla_slot_view(i).ancilla_type() == 0 {
                    k = i as i32;
                    break;
                }
            }
        } else {
            loop {
                let rotate = self
                    .sprite_system_view_mut()
                    .decrement_ancilla_alloc_rotate();
                if sign8(rotate) {
                    self.sprite_system_view_mut().set_ancilla_alloc_rotate(4);
                }
                k = self.sprite_system_view().ancilla_alloc_rotate() as i32;
                if self.ancilla_slot_view(k as usize).ancilla_type() == 10 {
                    break;
                }
            }
        }

        if k >= 0 {
            let k = k as usize;
            let floor = self.player_state_view().lower_level_state();
            let mirror_floor = self.player_state_view().lower_level_mirror_state();
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_ancilla_type(type_);
                ancilla.set_floor(floor);
                ancilla.set_floor2(mirror_floor);
                ancilla.set_y_velocity(0);
                ancilla.set_x_velocity(0);
                ancilla.set_object_priority(0);
                ancilla.set_u(0);
                ancilla.set_num_sprites(ANCILLA_DRAW_SPRITE_COUNTS[type_ as usize]);
            }
        }
        k
    }

    fn add_bird_common(&mut self, k: usize) {
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_y_velocity(0);
            ancilla.set_x_velocity(56);
            ancilla.set_item_to_link(0);
        }
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_aux_timer(1);
            ancilla.set_work_byte_3(3);
        }
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_k(0);
            ancilla.set_g(0);
        }

        let xt: u16 = if self.enhanced_features_view().has(1) {
            0x40
        } else {
            0
        };
        self.ancilla_set_xy(
            k,
            self.world_scroll()
                .bg2_x()
                .wrapping_sub(16)
                .wrapping_sub(xt),
            self.player_state_view().y().wrapping_sub(8),
        );
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
        for j in (0..16).rev() {
            if (((j as u8 ^ self.frame_state().frame_counter) & 3)
                | self.sprite_slot_view(j).hit_timer()
                | self.sprite_slot_view(j).ignore_projectile())
                != 0
            {
                continue;
            }
            if self.sprite_slot_view(j).floor() != self.ancilla_slot_view(k).floor()
                || self.sprite_slot_view(j).state() < 9
            {
                continue;
            }
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
            self.sprite_setup_hit_box(j, &mut hb);
            if !self.check_if_hit_boxes_overlap(&hb) {
                continue;
            }
            if self.sprite_slot_view(j).sprite_type() == 0x92 && self.sprite_slot_view(j).c() >= 3 {
                continue;
            }
            self.ancilla_check_damage_to_sprite(j, self.ancilla_slot_view(k).ancilla_type());
            let pt = self.ancilla_project_reflexive_speed_onto_sprite(
                j,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                64,
            );
            let value = 0u8.wrapping_sub(pt.x);
            self.sprite_slot_view_mut(j).set_x_recoil(value);
            let value = 0u8.wrapping_sub(pt.y);
            self.sprite_slot_view_mut(j).set_y_recoil(value);
        }
    }

    fn bomb_check_sprite_and_player_damage(&mut self, k: usize) {
        const BOMB_DMG_SPEED: [u8; 16] = [
            32, 32, 32, 32, 32, 32, 28, 28, 28, 28, 28, 28, 24, 24, 24, 24,
        ];
        const BOMB_DMG_ZVEL: [u8; 16] = [16, 16, 16, 16, 16, 16, 12, 12, 12, 12, 8, 8, 8, 8, 8, 8];
        const BOMB_DMG_DELAY: [u8; 16] = [
            32, 32, 32, 32, 32, 32, 24, 24, 24, 24, 24, 24, 16, 16, 16, 16,
        ];
        const BOMB_DMG_TO_LINK: [u8; 3] = [8, 4, 2];

        let bomb_phase = self.ancilla_slot_view(k).item_to_link();
        if bomb_phase == 0 || bomb_phase >= 9 {
            return;
        }
        self.bomb_check_sprite_damage(k);
        if self.player_state_view().sprite_damage_disable_timer() != 0 {
            if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize
                && self.player_state_view().is_lifting_or_carrying()
            {
                self.player_state_view_mut()
                    .clear_lifting_or_carrying_state();
                self.player_state_view_mut().clear_direction_lock();
            }
            return;
        }

        if self.player_state_view().has_auxiliary_state()
            || self.player_state_view().incapacitated_timer() != 0
            || self.ancilla_slot_view(k).floor() != self.player_state_view().lower_level_state()
        {
            return;
        }

        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
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
        let pt = self.bomb_project_speed_towards_player(k, x, y, BOMB_DMG_SPEED[j]);
        if self.player_state_view().blink_countdown() != 0
            || self.player_state_view().has_menu_block_flag(2)
        {
            return;
        }
        self.player_state_view_mut()
            .set_actual_velocity_xy(pt.x, pt.y);
        self.player_state_view_mut()
            .set_actual_z_velocity_and_copy(BOMB_DMG_ZVEL[j]);
        self.player_state_view_mut()
            .set_incapacitated_timer(BOMB_DMG_DELAY[j]);
        self.player_state_view_mut().set_auxiliary_state(1);
        self.player_state_view_mut().set_blink_countdown(58);
        if self.dungeon_savegame_state().savegame_state_bits() & 0x8000 == 0 {
            let armor = self.inventory_items().armor() as usize;
            self.player_state_view_mut()
                .set_given_damage(BOMB_DMG_TO_LINK[armor]);
        }
    }

    fn ancilla07_bomb(&mut self, k: usize) {
        if self.frame_state().submodule != 0 {
            if self.frame_state().submodule == 8 || self.frame_state().submodule == 16 {
                self.ancilla_handle_lift_logic(k);
            } else if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize
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
        let mut flag = self.ancilla_check_tile_collision_class2(k);

        if self.world_location_state().is_indoors()
            && self.ancilla_slot_view(k).l() != 0
            && self.ancilla_slot_view(k).tile_attribute() == 0x1c
        {
            let value = 1;
            self.ancilla_slot_view_mut(k).set_t_player(value);
        }

        loop {
            if flag
                && (!self.player_state_view().is_lifting_or_carrying()
                    || self.player_state_view().has_picking_throw_state())
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
            } else if !(k + 1 == self.player_state_view().ancilla_pickup_flag() as usize
                && self.player_state_view().is_lifting_or_carrying())
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
                    if self.dungeon_state_view().header_collision() != 3 {
                        if self.ancilla_slot_view(k).floor() == 0
                            && self.ancilla_slot_view(k).z() != 0
                            && self.ancilla_slot_view(k).z() != 0xff
                        {
                            self.ancilla_slot_view_mut(k).set_floor(1);
                        }
                    } else {
                        old_y = self
                            .ancilla_get_y(k)
                            .wrapping_add(self.dungeon_moving_floor().floor_y_velocity());
                        self.ancilla_set_x(
                            k,
                            self.ancilla_get_x(k)
                                .wrapping_add(self.dungeon_moving_floor().floor_x_velocity()),
                        );
                    }
                } else if a == 0x20 || (a & 0xf0) == 0xb0 && a != 0xb6 && a != 0xbc {
                    if !self.player_state_view().is_lifting_or_carrying() {
                        if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize {
                            self.player_state_view_mut().clear_ancilla_pickup_flag();
                        }
                        if self.ancilla_slot_view(k).timer() == 0 {
                            self.ancilla_slot_view_mut(k).clear();
                            return;
                        }
                    }
                } else if a == 8 {
                    if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize {
                        self.player_state_view_mut().clear_ancilla_pickup_flag();
                    }
                    if self.ancilla_slot_view(k).timer() == 0 {
                        self.ancilla_set_y(k, self.ancilla_get_y(k).wrapping_sub(24));
                        self.ancilla_transmute_to_splash(k);
                        return;
                    }
                } else if matches!(a, 0x68 | 0x69 | 0x6a | 0x6b) {
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
                if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize {
                    self.player_state_view_mut().clear_ancilla_pickup_flag();
                    if self.player_state_view().is_lifting_or_carrying() {
                        self.player_state_view_mut().clear_state_bits();
                        self.player_state_view_mut().clear_direction_lock();
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
            self.door_debris_view_mut().set_x_word(k, 0);
            self.bomb_check_for_destructibles(
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                k as u8,
            );
            if self.door_debris_view().x_word(k) != 0 {
                self.ancilla_slot_view_mut(k).set_step(1);
            }
        }
        self.bomb_draw(k);
    }

    fn boomerang_cheat_when_no_ones_looking(&self, k: usize, pt: &mut ProjectSpeedRet) {
        let x = self
            .player_state_view()
            .x()
            .wrapping_sub(self.ancilla_get_x(k))
            .wrapping_add(0xf0);
        let y = self
            .player_state_view()
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
        if self.messaging_state_view().effect_index() & 3 != 0 {
            let t = x
                .wrapping_add(if self.messaging_state_view().effect_index() & 1 != 0 {
                    16
                } else {
                    0
                })
                .wrapping_sub(self.world_scroll().bg2_x());
            if t >= 0x100 {
                return true;
            }
        }
        if self.messaging_state_view().effect_index() & 12 != 0 {
            let t = y
                .wrapping_add(if self.messaging_state_view().effect_index() & 4 != 0 {
                    16
                } else {
                    0
                })
                .wrapping_sub(self.world_scroll().bg2_y());
            if t >= 0xe2 {
                return true;
            }
        }
        false
    }

    fn boomerang_stop_off_screen(&mut self, k: usize) {
        let x = self.ancilla_get_x(k).wrapping_add(8);
        let y = self.ancilla_get_y(k).wrapping_add(8);
        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        if x >= link_x && x < link_x.wrapping_add(16) && y >= link_y && y < link_y.wrapping_add(24)
        {
            self.boomerang_terminate(k);
        }
    }

    fn boomerang_terminate(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).clear();
        self.minigame_state_view_mut()
            .clear_flag_boomerang_in_place();
        if self.player_state_view().item_in_hand_has(0x80) {
            self.player_state_view_mut().clear_item_in_hand();
            self.player_state_view_mut()
                .clear_button_mask_b_y_bits(0x40);
            if self.player_state_view().button_mask_b_y() & 0x80 == 0 {
                self.player_state_view_mut().clear_direction_lock_bits(1);
            }
        }
    }

    fn ancilla05_boomerang(&mut self, k: usize) {
        const BOOMERANG_X0: [i8; 8] = [0, 0, -8, 8, 8, 8, -8, -8];
        const BOOMERANG_Y0: [i8; 8] = [-16, 6, 0, 0, -8, 8, -8, 8];
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        for j in (0..=4).rev() {
            if self.ancilla_slot_view(j).ancilla_type() == 0x22 {
                self.boomerang_draw(k);
                return;
            }
        }
        if self.frame_state().submodule != 0 {
            self.boomerang_draw(k);
            return;
        }

        if self.frame_state().frame_counter & 7 == 0 {
            self.ancilla_sfx2_pan(k, 9);
        }

        if self.ancilla_slot_view(k).aux_timer() == 0 {
            if self.player_state_view().button_b_frames() < 9
                && self.player_state_view().action_handler_timer() == 0
            {
                if self.player_state_view().is_bunny_mirror()
                    || self.player_state_view().has_auxiliary_state()
                    || !self.player_state_view().has_item_in_hand()
                        && self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES)
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
                self.player_state_view()
                    .x()
                    .wrapping_add(BOOMERANG_X0[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(8)
                    .wrapping_add(BOOMERANG_Y0[j] as i16 as u16),
            );
            self.ancilla_slot_view_mut(k).add_aux_timer(1);
        }

        if self.ancilla_slot_view(k).g() != 0 && self.frame_state().frame_counter & 1 == 0 {
            self.ancilla_add_sword_charge_sparkle(k);
        }

        if self.ancilla_slot_view(k).item_to_link() != 0 {
            if self.ancilla_slot_view(k).k() != 0 {
                self.ancilla_slot_view_mut(k).advance_k();
            }
            let link_y = self.player_state_view().y();
            self.ancilla_slot_view_mut(k).set_a_word(link_y);
            self.player_state_view_mut().set_y(link_y.wrapping_add(8));
            let speed = self.ancilla_slot_view(k).h();
            let mut pt = self.ancilla_project_speed_towards_player(k, speed);
            self.boomerang_cheat_when_no_ones_looking(k, &mut pt);
            {
                let mut boomerang = self.ancilla_slot_view_mut(k);
                boomerang.set_x_velocity(pt.x);
                boomerang.set_y_velocity(pt.y);
            }
            let y = self.ancilla_slot_view(k).a_word();
            self.player_state_view_mut().set_y(y);
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
        if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
            && k == 4
            && self.frame_state().frame_counter >= 140
            && self.frame_state().frame_counter <= 210
        {
            let boomerang = self.ancilla_slot_view(k);
            eprintln!(
                "R boomerang-tick fc={} k={} x={:04x} y={:04x} xv={:02x} yv={:02x} step={:02x} aux={:02x} item={:02x} K={:02x} H={:02x} dir={:02x} work23={:02x} link={:04x}/{:04x} hook={:02x}",
                self.frame_state().frame_counter,
                k,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                boomerang.x_velocity(),
                boomerang.y_velocity(),
                boomerang.step(),
                boomerang.aux_timer(),
                boomerang.item_to_link(),
                boomerang.k(),
                boomerang.h(),
                boomerang.direction(),
                self.ancilla_slot_view(k).work_byte_23(),
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.messaging_state_view().effect_index(),
            );
        }
        let hit_spr = self.ancilla_check_sprite_collision(k);
        let trace_pre_item = self.ancilla_slot_view(k).item_to_link();
        let trace_pre_step = self.ancilla_slot_view(k).step();
        let trace_pre_k = self.ancilla_slot_view(k).k();

        if self.ancilla_slot_view(k).item_to_link() == 0 {
            if hit_spr.is_some() {
                let item_to_link = self.ancilla_slot_view(k).item_to_link() ^ 1;
                self.ancilla_slot_view_mut(k).set_item_to_link(item_to_link);
                if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
                    && k == 4
                    && self.frame_state().frame_counter >= 130
                    && self.frame_state().frame_counter <= 150
                {
                    eprintln!(
                        "R boomerang-branch fc={} reason=hit hit={} pre_item={:02x} item={:02x} pre_step={:02x} step={:02x} preK={:02x} K={:02x} x={:04x} y={:04x}",
                        self.frame_state().frame_counter,
                        hit_spr.unwrap(),
                        trace_pre_item,
                        item_to_link,
                        trace_pre_step,
                        self.ancilla_slot_view(k).step(),
                        trace_pre_k,
                        self.ancilla_slot_view(k).k(),
                        self.ancilla_get_x(k),
                        self.ancilla_get_y(k),
                    );
                }
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
                if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
                    && k == 4
                    && self.frame_state().frame_counter >= 130
                    && self.frame_state().frame_counter <= 150
                {
                    eprintln!(
                        "R boomerang-branch fc={} reason=tile attr={:02x} pre_item={:02x} item={:02x} pre_step={:02x} step={:02x} preK={:02x} K={:02x} x={:04x} y={:04x}",
                        self.frame_state().frame_counter,
                        self.ancilla_slot_view(k).tile_attribute(),
                        trace_pre_item,
                        item_to_link,
                        trace_pre_step,
                        self.ancilla_slot_view(k).step(),
                        trace_pre_k,
                        self.ancilla_slot_view(k).k(),
                        self.ancilla_get_x(k),
                        self.ancilla_get_y(k),
                    );
                }
            } else {
                let reached_edge = self.boomerang_screen_edge(k);
                if !reached_edge {
                    self.ancilla_slot_view_mut(k).retreat_step();
                }
                if reached_edge || self.ancilla_slot_view(k).step() == 0 {
                    let item_to_link = self.ancilla_slot_view(k).item_to_link() ^ 1;
                    self.ancilla_slot_view_mut(k).set_item_to_link(item_to_link);
                    if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
                        && k == 4
                        && self.frame_state().frame_counter >= 130
                        && self.frame_state().frame_counter <= 150
                    {
                        eprintln!(
                            "R boomerang-branch fc={} reason=edge-step pre_item={:02x} item={:02x} pre_step={:02x} step={:02x} preK={:02x} K={:02x} x={:04x} y={:04x}",
                            self.frame_state().frame_counter,
                            trace_pre_item,
                            item_to_link,
                            trace_pre_step,
                            self.ancilla_slot_view(k).step(),
                            trace_pre_k,
                            self.ancilla_slot_view(k).k(),
                            self.ancilla_get_x(k),
                            self.ancilla_get_y(k),
                        );
                    }
                } else if self.ancilla_slot_view(k).step() < 5 {
                    self.ancilla_slot_view_mut(k).retreat_k();
                    if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
                        && k == 4
                        && self.frame_state().frame_counter >= 130
                        && self.frame_state().frame_counter <= 150
                    {
                        eprintln!(
                            "R boomerang-branch fc={} reason=outbound pre_item={:02x} item={:02x} pre_step={:02x} step={:02x} preK={:02x} K={:02x} x={:04x} y={:04x}",
                            self.frame_state().frame_counter,
                            trace_pre_item,
                            self.ancilla_slot_view(k).item_to_link(),
                            trace_pre_step,
                            self.ancilla_slot_view(k).step(),
                            trace_pre_k,
                            self.ancilla_slot_view(k).k(),
                            self.ancilla_get_x(k),
                            self.ancilla_get_y(k),
                        );
                    }
                } else if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
                    && k == 4
                    && self.frame_state().frame_counter >= 130
                    && self.frame_state().frame_counter <= 150
                {
                    eprintln!(
                        "R boomerang-branch fc={} reason=outbound pre_item={:02x} item={:02x} pre_step={:02x} step={:02x} preK={:02x} K={:02x} x={:04x} y={:04x}",
                        self.frame_state().frame_counter,
                        trace_pre_item,
                        self.ancilla_slot_view(k).item_to_link(),
                        trace_pre_step,
                        self.ancilla_slot_view(k).step(),
                        trace_pre_k,
                        self.ancilla_slot_view(k).k(),
                        self.ancilla_get_x(k),
                        self.ancilla_get_y(k),
                    );
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

    fn ancilla01_somaria_bullet(&mut self, k: usize) {
        const SOMARIAN_BLAST_MASK: [u8; 6] = [7, 3, 1, 0, 0, 0];

        if self.frame_state().submodule == 0 {
            if self.frame_state().frame_counter
                & SOMARIAN_BLAST_MASK[self.ancilla_slot_view(k).step() as usize]
                == 0
            {
                self.ancilla_move_x(k);
                self.ancilla_move_y(k);
            }
            if self.ancilla_slot_view(k).timer() == 0 {
                self.ancilla_slot_view_mut(k).set_timer(3);
                let mut a = self.ancilla_slot_view_mut(k).advance_step();
                if a >= 6 {
                    a = 4;
                }
                self.ancilla_slot_view_mut(k).set_step(a);
            }
            if self.ancilla_check_sprite_collision(k).is_some()
                || self.ancilla_check_tile_collision_staggered(k) != 0
            {
                let mut bullet = self.ancilla_slot_view_mut(k);
                bullet.set_ancilla_type(4);
                bullet.set_timer(7);
                bullet.set_num_sprites(16);
            }
        }
        self.somarian_blast_draw(k);
    }

    fn bomb_get_displacement_from_link(&self, k: usize) -> i32 {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        (((abs16(self.player_state_view().x().wrapping_add(8).wrapping_sub(x))
            + abs16(
                self.player_state_view()
                    .y()
                    .wrapping_add(12)
                    .wrapping_sub(y),
            ))
            & 0xfc)
            >> 2) as i32
    }

    fn hookshot_check_proximity_to_link(&self, x: i32, y: i32) -> bool {
        abs16(
            self.player_state_view()
                .y()
                .wrapping_sub(self.world_scroll().bg2_y())
                .wrapping_add(12)
                .wrapping_sub(y as u16)
                .wrapping_sub(4),
        ) < 12
            && abs16(
                self.player_state_view()
                    .x()
                    .wrapping_sub(self.world_scroll().bg2_x())
                    .wrapping_add(8)
                    .wrapping_sub(x as u16)
                    .wrapping_sub(4),
            ) < 12
    }

    fn hookshot_should_i_even_bother_with_tiles(&self, k: usize) -> bool {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        if self.world_location_state().is_outdoors() {
            let area = (self.world_region().current_area_of_player() >> 1) as usize;
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
                || (y & 0x200) != (self.player_state_view().y() & 0x200)
        } else {
            (x & 0x1ff) < 4
                || (x & 0x1ff) >= 0x1f0
                || (x & 0x200) != (self.player_state_view().x() & 0x200)
        }
    }

    fn boomerang_draw(&mut self, k: usize) {
        const BOOMERANG_FLAGS: [u8; 8] = [0xa4, 0xe4, 0x64, 0x24, 0xa2, 0xe2, 0x62, 0x22];
        const BOOMERANG_DRAW_OFFSET: [SignedOffset; 4] = [
            SignedOffset { y: 2, x: -2 },
            SignedOffset { y: 2, x: 2 },
            SignedOffset { y: -2, x: 2 },
            SignedOffset { y: -2, x: -2 },
        ];
        const BOOMERANG_DRAW_OAM_IDX: [u16; 2] = [0x180, 0xd0];
        const BOOMERANG_FRAME_RESET_BY_TYPE: [u8; 2] = [3, 2];
        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);

        if self.ancilla_slot_view(k).item_to_link() != 0 {
            let floor = self.player_state_view().lower_level_state();
            self.ancilla_slot_view_mut(k).set_floor(floor);
            const TAGALONG_LAYER_BITS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
            let priority =
                (TAGALONG_LAYER_BITS[self.player_state_view().lower_level_state() as usize] as u16)
                    << 8;
            self.oam_state_view_mut().set_priority_word(priority);
        }

        if self.ancilla_slot_view(k).object_priority() != 0 {
            self.oam_state_view_mut().set_priority_word(0x3000);
        }

        if self.frame_state().submodule == 0 && self.ancilla_slot_view(k).aux_timer() != 0 {
            if sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3()) {
                let frame_reset =
                    BOOMERANG_FRAME_RESET_BY_TYPE[self.ancilla_slot_view(k).g() as usize];
                self.ancilla_slot_view_mut(k).set_work_byte_3(frame_reset);
                let delta = if self.ancilla_slot_view(k).s_player() != 0 {
                    0xff
                } else {
                    1
                };
                self.ancilla_slot_view_mut(k).add_work_byte_1_mod4(delta);
            }
        }

        let j = self.ancilla_slot_view(k).work_byte_1() as usize;
        let offset = BOOMERANG_DRAW_OFFSET[j];
        let x = info_x.wrapping_add(offset.x as i16 as u16);
        let y = info_y.wrapping_add(offset.y as i16 as u16);
        if self.ancilla_slot_view(k).aux_timer() == 0 {
            let i = BOOMERANG_DRAW_OAM_IDX[self.oam_state_view().sprite_sorting_offset_index()];
            self.oam_state_view_mut()
                .set_current_extended_pointer((i >> 2) + 0xa20);
            self.oam_state_view_mut().set_current_pointer(i + 0x800);
        }
        self.ancilla_set_oam_safe(
            self.oam_state_view().current_pointer_usize(),
            x,
            y,
            0x26,
            (BOOMERANG_FLAGS[self.ancilla_slot_view(k).g() as usize * 4 + j] & !0x30)
                | self.oam_state_view().priority_high(),
            2,
        );
    }

    fn ancilla06_wall_hit(&mut self, k: usize) {
        if sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3()) {
            let t = self.ancilla_slot_view_mut(k).advance_item_to_link();
            if t == 5 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
            self.ancilla_slot_view_mut(k).set_work_byte_3(1);
        }
        self.wall_hit_draw(k);
    }

    fn ancilla_sword_wall_hit(&mut self, k: usize) {
        self.sprite_system_view_mut().set_alert_flag(3);
        if sign8(self.ancilla_slot_view_mut(k).tick_aux_timer()) {
            let t = self.ancilla_slot_view_mut(k).advance_item_to_link();
            if t == 8 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
            self.ancilla_slot_view_mut(k).set_aux_timer(1);
        }
        self.wall_hit_draw(k);
    }

    fn ancilla1_d_screen_shake(&mut self, k: usize) {
        if self.frame_state().submodule == 0 {
            let item_to_link = self.ancilla_slot_view(k).item_to_link().wrapping_sub(1);
            self.ancilla_slot_view_mut(k).set_item_to_link(item_to_link);
            if sign8(item_to_link) {
                self.world_scroll_mut().set_bg1_x_offset(0);
                self.world_scroll_mut().set_bg1_y_offset(0);
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
            let offs = self.dash_tremor_twiddle_offset(k);
            let j = self.ancilla_slot_view(k).direction();
            if j == 0 {
                self.world_scroll_mut().set_bg1_x_offset(offs as u16);
                self.player_state_view_mut()
                    .add_movement_velocity_delta(offs as u16, 0);
            } else {
                self.world_scroll_mut().set_bg1_y_offset(offs as u16);
                self.player_state_view_mut()
                    .add_movement_velocity_delta(0, offs as u16);
            }
        }
        self.sprite_system_view_mut().set_alert_flag(3);
    }

    fn ancilla1_e_dash_dust(&mut self, k: usize) {
        if self.ancilla_slot_view(k).step() != 0 {
            self.dash_dust_motive(k);
            return;
        }
        if self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).set_timer(3);
            let item_to_link = self.ancilla_slot_view_mut(k).advance_item_to_link();
            if item_to_link == 5 {
                return;
            }
            if item_to_link == 6 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }
        if self.ancilla_slot_view(k).item_to_link() == 5 {
            return;
        }

        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();

        const DASH_DUST_DRAW_X1: [i8; 4] = [0, 0, 4, -4];
        const DASH_DUST_DRAW_X: [i16; 30] = [
            10, 5, -1, 0, 10, 5, 0, 5, -1, 0, -1, -1, 9, -1, -1, 10, 5, -1, 0, 10, 5, 0, 5, -1, 0,
            -1, -1, 9, -1, -1,
        ];
        const DASH_DUST_DRAW_Y: [i16; 30] = [
            -2, 0, -1, -3, -2, 0, -3, 0, -1, -3, -1, -1, -2, -1, -1, -2, 0, -1, -3, -2, 0, -3, 0,
            -1, -3, -1, -1, -2, -1, -1,
        ];
        const DASH_DUST_DRAW_CHAR: [u8; 30] = [
            0xcf, 0xa9, 0xff, 0xa9, 0xdf, 0xcf, 0xcf, 0xdf, 0xff, 0xdf, 0xff, 0xff, 0xa9, 0xff,
            0xff, 0xcf, 0xcf, 0xff, 0xcf, 0xdf, 0xcf, 0xcf, 0xdf, 0xff, 0xdf, 0xff, 0xff, 0xcf,
            0xff, 0xff,
        ];
        let r12 = DASH_DUST_DRAW_X1[self.player_state_view().facing_index()] as i16;
        let mut t = 3
            * (self.ancilla_slot_view(k).item_to_link() as usize
                + if self.player_state_view().water_ripple_or_grass_state() == 1 {
                    5
                } else {
                    0
                });

        for _ in (0..=2).rev() {
            if DASH_DUST_DRAW_CHAR[t] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    info_x
                        .wrapping_add(r12 as u16)
                        .wrapping_add(DASH_DUST_DRAW_X[t] as u16),
                    info_y.wrapping_add(DASH_DUST_DRAW_Y[t] as u16),
                    DASH_DUST_DRAW_CHAR[t],
                    4 | self.oam_state_view().priority_high(),
                    0,
                );
                oam += 4;
            }
            t += 1;
        }
    }

    fn dash_dust_motive(&mut self, k: usize) {
        const MOTIVE_DASH_DUST_DRAW_CHAR: [u8; 3] = [0xa9, 0xcf, 0xdf];
        if self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).set_timer(3);
            let item_to_link = self.ancilla_slot_view_mut(k).advance_item_to_link();
            if item_to_link == 3 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }
        if self.player_state_view().facing() == 2 {
            self.oam_allocate_from_region_b(4);
        }
        let frame = self.ancilla_slot_view(k).item_to_link() as usize;
        if frame >= MOTIVE_DASH_DUST_DRAW_CHAR.len() {
            self.ancilla_slot_view_mut(k).clear();
            return;
        }
        let (x, y) = self.ancilla_prep_oam_coord(k);
        self.ancilla_set_oam(
            self.oam_state_view().current_pointer_usize(),
            x,
            y,
            MOTIVE_DASH_DUST_DRAW_CHAR[frame],
            4 | self.oam_state_view().priority_high(),
            0,
        );
    }

    fn wall_hit_draw(&mut self, k: usize) {
        const WALL_HIT_X: [i8; 32] = [
            -4, 0, 0, 0, -4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -4, 0, 0, 0, -4, 0,
            0, 0, -8, 0, 0, 0,
        ];
        const WALL_HIT_Y: [i8; 32] = [
            -4, 0, 0, 0, -4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -4, 0, 0, 0, -4, 0,
            0, 0, -8, 0, 0, 0,
        ];
        const WALL_HIT_CHAR: [u8; 32] = [
            0x80, 0, 0, 0, 0x92, 0, 0, 0, 0x81, 0x81, 0x81, 0x81, 0x82, 0x82, 0x82, 0x82, 0x93,
            0x93, 0x93, 0x93, 0x92, 0, 0, 0, 0xb9, 0, 0, 0, 0x90, 0x90, 0, 0,
        ];
        const WALL_HIT_FLAGS: [u8; 32] = [
            0x32, 0, 0, 0, 0x32, 0, 0, 0, 0x32, 0x72, 0xb2, 0xf2, 0x32, 0x72, 0xb2, 0xf2, 0x32,
            0x72, 0xb2, 0xf2, 0x32, 0, 0, 0, 0x72, 0, 0, 0, 0x32, 0xf2, 0, 0,
        ];
        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        let mut t = self.ancilla_slot_view(k).item_to_link() as usize * 4;

        let mut oam = self.oam_state_view().current_pointer_usize();
        for _ in (0..=3).rev() {
            if WALL_HIT_CHAR[t] != 0 {
                self.ancilla_set_oam(
                    oam,
                    info_x.wrapping_add(WALL_HIT_X[t] as i16 as u16),
                    info_y.wrapping_add(WALL_HIT_Y[t] as i16 as u16),
                    WALL_HIT_CHAR[t],
                    (WALL_HIT_FLAGS[t] & !0x30) | self.oam_state_view().priority_high(),
                    0,
                );
                oam += 4;
            }
            oam = self.ancilla_allocate_oam_from_custom_region(oam);
            t += 1;
        }
    }

    fn ancilla08_door_debris(&mut self, k: usize) {
        self.door_debris_draw(k);
        let work_byte_26 = self.ancilla_slot_view(k).work_byte_26().wrapping_sub(1);
        self.ancilla_slot_view_mut(k).set_work_byte_26(work_byte_26);
        if sign8(work_byte_26) {
            let mut debris = self.ancilla_slot_view_mut(k);
            debris.set_work_byte_26(7);
            let frame = debris.advance_work_byte_25();
            if frame == 4 {
                debris.clear();
            }
        }
    }

    fn door_debris_draw(&mut self, k: usize) {
        const DOOR_DEBRIS_OFFSET: [UnsignedOffset; 32] = [
            UnsignedOffset { y: 4, x: 7 },
            UnsignedOffset { y: 3, x: 17 },
            UnsignedOffset { y: 8, x: 8 },
            UnsignedOffset { y: 7, x: 17 },
            UnsignedOffset { y: 11, x: 7 },
            UnsignedOffset { y: 10, x: 16 },
            UnsignedOffset { y: 16, x: 7 },
            UnsignedOffset { y: 17, x: 17 },
            UnsignedOffset { y: 20, x: 7 },
            UnsignedOffset { y: 21, x: 17 },
            UnsignedOffset { y: 16, x: 8 },
            UnsignedOffset { y: 17, x: 17 },
            UnsignedOffset { y: 13, x: 7 },
            UnsignedOffset { y: 14, x: 16 },
            UnsignedOffset { y: 8, x: 7 },
            UnsignedOffset { y: 7, x: 17 },
            UnsignedOffset { y: 7, x: 4 },
            UnsignedOffset { y: 17, x: 3 },
            UnsignedOffset { y: 8, x: 8 },
            UnsignedOffset { y: 17, x: 7 },
            UnsignedOffset { y: 7, x: 11 },
            UnsignedOffset { y: 16, x: 10 },
            UnsignedOffset { y: 7, x: 16 },
            UnsignedOffset { y: 17, x: 17 },
            UnsignedOffset { y: 7, x: 20 },
            UnsignedOffset { y: 17, x: 21 },
            UnsignedOffset { y: 8, x: 16 },
            UnsignedOffset { y: 17, x: 17 },
            UnsignedOffset { y: 7, x: 13 },
            UnsignedOffset { y: 16, x: 14 },
            UnsignedOffset { y: 7, x: 8 },
            UnsignedOffset { y: 17, x: 7 },
        ];
        const DOOR_DEBRIS_TILE: [OamTileAttrs; 32] = [
            OamTileAttrs {
                char: 0x5e,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0xe0,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0xa0,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0xe0,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0xe0,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0xa0,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0xe0,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0x20,
            },
            OamTileAttrs {
                char: 0x5e,
                flags: 0xe0,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x60,
            },
            OamTileAttrs {
                char: 0x4f,
                flags: 0x60,
            },
        ];

        self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let y = self
            .door_debris_view()
            .y_word(k)
            .wrapping_sub(self.world_scroll().bg2_y());
        let x = self
            .door_debris_view()
            .x_word(k)
            .wrapping_sub(self.world_scroll().bg2_x());
        let j = self.ancilla_slot_view(k).work_byte_25() as usize
            + self.door_debris_view().direction(k) as usize * 4;

        for i in 0..2 {
            let t = j * 2 + i;
            let offset = DOOR_DEBRIS_OFFSET[t];
            let tile = DOOR_DEBRIS_TILE[t];
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(offset.x),
                y.wrapping_add(offset.y),
                tile.char,
                (tile.flags & 0xc0) | self.oam_state_view().priority_high(),
                0,
            );
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
        }
    }

    fn ancilla_add_boomerang_wall_clink(&mut self, k: usize) {
        const BOOMERANG_WALL_HIT_X: [i8; 8] = [8, 8, 0, 10, 12, 8, 4, 0];
        const BOOMERANG_WALL_HIT_Y: [i8; 8] = [0, 8, 8, 8, 4, 8, 12, 8];
        const BOOMERANG_WALL_HIT_OFFSET_INDEX: [u8; 16] =
            [0, 6, 4, 0, 2, 10, 12, 0, 0, 8, 14, 0, 0, 0, 0, 0];
        let temp_x = self.ancilla_get_x(k);
        let temp_y = self.ancilla_get_y(k);
        self.minigame_state_view_mut().set_boomerang_temp_x(temp_x);
        self.minigame_state_view_mut().set_boomerang_temp_y(temp_y);
        if let Some(k) = self.ancilla_add_ancilla(6, 1) {
            {
                let mut wall_clink = self.ancilla_slot_view_mut(k);
                wall_clink.set_item_to_link(0);
                wall_clink.set_work_byte_3(1);
            }
            let j = (BOOMERANG_WALL_HIT_OFFSET_INDEX
                [self.messaging_state_view().effect_index() as usize]
                >> 1) as usize;
            self.ancilla_set_xy(
                k,
                self.minigame_state_view()
                    .boomerang_temp_x()
                    .wrapping_add(BOOMERANG_WALL_HIT_X[j] as i16 as u16),
                self.minigame_state_view()
                    .boomerang_temp_y()
                    .wrapping_add(BOOMERANG_WALL_HIT_Y[j] as i16 as u16),
            );
        }
    }

    pub(super) fn call_for_duck_indoors(&mut self) {
        self.ancilla_sfx2_near(0x13);
        self.ancilla_add_duck_take_off(0x27, 4);
    }

    pub(super) fn ancilla_add_duck_take_off(&mut self, a: u8, y: u8) {
        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ancilla_slot_view_mut(k).set_timer(0x78);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_l(value);
            {
                let mut duck = self.ancilla_slot_view_mut(k);
                duck.set_z_velocity(0);
                duck.set_z(0);
                duck.set_step(0);
            }
            self.add_bird_common(k);
        }
    }

    fn ancilla30_byrna_windup_spark(&mut self, k: usize) {
        const INITIAL_CANE_SPARK_X: [i8; 16] =
            [3, 1, 0, 0, 13, 16, 12, 12, 24, 7, -4, -10, -8, 9, 22, 26];
        const INITIAL_CANE_SPARK_Y: [i8; 16] =
            [5, 0, -3, -6, -8, -3, 12, 28, 5, 0, 8, 16, 5, 0, 8, 16];
        const INITIAL_CANE_SPARK_DRAW_X: [i8; 16] =
            [-4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0];
        const INITIAL_CANE_SPARK_DRAW_Y: [i8; 16] =
            [-4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0];
        const INITIAL_CANE_SPARK_DRAW_CHAR: [u8; 16] = [
            0x92, 0xff, 0xff, 0xff, 0x8c, 0x8c, 0x8c, 0x8c, 0xd6, 0xd6, 0xd6, 0xd6, 0x93, 0x93,
            0x93, 0x93,
        ];
        const INITIAL_CANE_SPARK_DRAW_FLAGS: [u8; 16] = [
            0x22, 0xff, 0xff, 0xff, 0x22, 0x62, 0xa2, 0xe2, 0x24, 0x64, 0xa4, 0xe4, 0x22, 0x62,
            0xa2, 0xe2,
        ];

        if self.frame_state().submodule == 0 {
            self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(self.ancilla_slot_view(k).aux_timer()) {
                let value = 1;
                self.ancilla_slot_view_mut(k).set_aux_timer(value);
                if self.ancilla_slot_view_mut(k).advance_item_to_link() == 17 {
                    self.byrna_windup_spark_transmute_to_normal(k);
                    return;
                }
            }
        }
        if self.ancilla_slot_view(k).item_to_link() == 0 {
            return;
        }

        let mut j = self.player_state_view().action_handler_timer();
        if j == 2 {
            let mut a = self.ancilla_slot_view(k).work_byte_3().wrapping_sub(1);
            if sign8(a) {
                a = 0;
                j = 3;
            }
            let value = a;
            self.ancilla_slot_view_mut(k).set_work_byte_3(value);
        }
        let j = j.wrapping_add(self.player_state_view().facing().wrapping_mul(2)) as usize;
        self.ancilla_set_xy(
            k,
            self.player_state_view()
                .x()
                .wrapping_add(INITIAL_CANE_SPARK_X[j] as i16 as u16),
            self.player_state_view()
                .y()
                .wrapping_add(INITIAL_CANE_SPARK_Y[j] as i16 as u16),
        );
        let (x, y) = self.ancilla_prep_oam_coord(k);

        let a = self.ancilla_slot_view(k).item_to_link().wrapping_sub(1) & 0x0f;
        let mut j = 0usize;
        if a != 0 {
            j = 4 * if a != 15 { ((a & 1) + 1) as usize } else { 3 };
        }

        let mut oam = self.oam_state_view().current_pointer_usize();
        for _ in 0..4 {
            if INITIAL_CANE_SPARK_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(INITIAL_CANE_SPARK_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(INITIAL_CANE_SPARK_DRAW_Y[j] as i16 as u16),
                    INITIAL_CANE_SPARK_DRAW_CHAR[j],
                    INITIAL_CANE_SPARK_DRAW_FLAGS[j] & !0x30
                        | self.oam_state_view().priority_high(),
                    0,
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn byrna_windup_spark_transmute_to_normal(&mut self, k: usize) {
        const CANE_SPARK_TRAILING_ANGLES: [u8; 16] = [
            0x34, 0x33, 0x32, 0x31, 0x16, 0x15, 0x14, 0x13, 0x2a, 0x29, 0x28, 0x27, 0x10, 0x0f,
            0x0e, 0x0d,
        ];
        self.ancilla_slot_view_mut(k).set_ancilla_type(0x31);
        let j = (self.player_state_view().facing() << 1) as usize;
        self.effect_angle_scratch_view_mut()
            .set_angles4(&CANE_SPARK_TRAILING_ANGLES, j);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_g(value);
        {
            let mut spark = self.ancilla_slot_view_mut(k);
            spark.set_aux_timer(0x17);
            spark.set_item_to_link(0);
            spark.set_work_byte_3(8);
            spark.set_step(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        let value = 2;
        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
        self.ancilla_slot_view_mut(k).set_timer(21);
        self.effect_angle_scratch_view_mut().set_radial_radius(20);
        self.ancilla_sfx3_near(0x30);
        self.ancilla31_byrna_spark(k);
    }

    fn ancilla31_byrna_spark(&mut self, k: usize) {
        const CANE_SPARK_MAGIC: [u8; 3] = [4, 2, 1];
        const CANE_SPARK_CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];

        let mut flags = 2;
        if self.frame_state().submodule == 0 {
            if self.player_state_view().current_item_y() != 13 {
                self.kill_byrna_spark(k);
                return;
            }
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.ancilla_slot_view_mut(k).tick_aux_timer();
            if self.ancilla_slot_view(k).aux_timer() == 0 {
                let value = 1;
                self.ancilla_slot_view_mut(k).set_aux_timer(value);
                let magic_cost =
                    CANE_SPARK_MAGIC[self.player_state_view().magic_consumption_level() as usize];
                let r0 = self
                    .player_state_view()
                    .magic_power()
                    .wrapping_sub(magic_cost);
                if self.player_state_view().magic_power() == 0 || r0 >= 0x80 {
                    self.kill_byrna_spark(k);
                    return;
                }

                self.ancilla_slot_view_mut(k).subtract_g(1);
                if sign8(self.ancilla_slot_view(k).g()) {
                    let value = 0x17;
                    self.ancilla_slot_view_mut(k).set_g(value);
                    self.player_resources_view_mut().set_magic_power(r0);
                }
                if self.player_state_view().filtered_joypad_h() & 0x40 != 0 {
                    self.kill_byrna_spark(k);
                    return;
                }
            }
            if self.ancilla_slot_view(k).step() != 3 {
                let a = self.ancilla_slot_view_mut(k).advance_item_to_link();
                let value = if a >= 4 {
                    3
                } else if a == 2 {
                    1
                } else if a == 3 {
                    2
                } else {
                    0
                };
                self.ancilla_slot_view_mut(k).set_step(value);
            }
            self.ancilla_slot_view_mut(k).subtract_work_byte_1(1);
            if sign8(self.ancilla_slot_view(k).work_byte_1()) {
                let value = 2;
                self.ancilla_slot_view_mut(k).set_work_byte_1(value);
                flags = 4;
            }
        }

        let mut z = self.player_state_view().z() as u8 as i8 as i16;
        if z == -1 {
            z = 0;
        }
        let swordbeam_temp_y = self
            .player_state_view()
            .y()
            .wrapping_add(12)
            .wrapping_sub(z as u16);
        let swordbeam_temp_x = self.player_state_view().x().wrapping_add(8);
        self.ether_orbit_view_mut()
            .set_swordbeam_temp(swordbeam_temp_x, swordbeam_temp_y);
        if self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).set_timer(21);
            self.ancilla_sfx3_near(0x30);
        }
        let mut oam = self.oam_state_view().current_pointer_usize();
        let mut i = self.ancilla_slot_view(k).step() as usize;
        loop {
            let angle = if self.frame_state().submodule == 0 {
                self.effect_angle_scratch_view_mut().add_angle_mod64(i, 3)
            } else {
                self.effect_angle_scratch_view().angle(i)
            };
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                angle,
                self.effect_angle_scratch_view().radial_radius(),
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                CANE_SPARK_CHAR[i],
                flags | self.oam_state_view().priority_high(),
                0,
            );
            self.ancilla_set_xy(
                k,
                pt.x.wrapping_add(self.world_scroll().bg2_x()),
                pt.y.wrapping_add(self.world_scroll().bg2_y()),
            );
            self.ancilla_slot_view_mut(k).set_direction(0);
            self.ancilla_check_sprite_collision(k);
            oam += 4;
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    fn kill_byrna_spark(&mut self, k: usize) {
        self.player_state_view_mut()
            .clear_sprite_damage_disable_timer();
        self.ancilla_slot_view_mut(k).clear();
        self.player_state_view_mut().clear_given_damage();
    }

    pub(super) fn configure_revival_ancillae(&mut self) {
        self.player_state_view_mut().set_link_dma_staging_index(80);
        let mut k = 0usize;

        {
            let mut revival = self.ancilla_slot_view_mut(k);
            revival.set_work_byte_3(64);
            revival.set_step(0);
            revival.set_z_velocity(8);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        let value = 5;
        self.ancilla_slot_view_mut(k).set_g(value);
        self.ancilla_slot_view_mut(k).set_item_to_link(0);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_k(value);
        self.ancilla_set_xy(
            k,
            self.player_state_view().x(),
            self.player_state_view().y(),
        );
        self.ancilla_slot_view_mut(k).set_z(0);
        k += 1;

        self.ancilla_slot_view_mut(k).set_z(0);
        {
            let mut revival = self.ancilla_slot_view_mut(k);
            revival.set_work_byte_3(240);
            revival.set_step(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_k(value);
        k += 1;

        self.ancilla_slot_view_mut(k).set_item_to_link(2);
        {
            let mut revival = self.ancilla_slot_view_mut(k);
            revival.set_aux_timer(3);
            revival.set_work_byte_3(8);
            revival.set_step(0);
        }
        self.ancilla_slot_view_mut(k).set_direction(3);
        let item_to_link = self.ancilla_slot_view(k).item_to_link() as usize;
        self.ancilla_slot_view_mut(k)
            .set_work_byte_25(MAGIC_POWDER_FRAME_TIMERS[30 + item_to_link]);

        self.ancilla_set_xy(
            k,
            self.player_state_view().x().wrapping_add(20),
            self.player_state_view().y().wrapping_add(2),
        );
    }

    pub(super) fn ancilla_add_bunny_poof(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.player_state_view_mut().set_visibility_status(0x0c);
            let effect = if !self.player_state_view().is_bunny_mirror() {
                0x14
            } else {
                0x15
            };
            self.set_sound_effect_1_with_link_pan(effect);
            {
                let mut poof = self.ancilla_slot_view_mut(k);
                poof.set_step(0);
                poof.set_item_to_link(0);
                poof.set_aux_timer(7);
            }
            self.ancilla_set_xy(
                k,
                self.player_state_view().x(),
                self.player_state_view().y().wrapping_add(4),
            );
        }
    }

    pub(super) fn ancilla_add_dwarf_poof(&mut self, ain: u8, yin: u8) {
        let Some(k) = self.ancilla_add_ancilla(ain, yin) else {
            return;
        };
        let effect = if self.follower_state_view().indicator() == 8 {
            0x14
        } else {
            0x15
        };
        self.set_sound_effect_1_with_link_pan(effect);

        {
            let mut poof = self.ancilla_slot_view_mut(k);
            poof.set_item_to_link(0);
            poof.set_step(0);
            poof.set_aux_timer(7);
        }
        self.follower_state_view_mut().set_appearance_none_flag(1);
        let j = self.follower_state_view().data_index() as usize;
        let x = self.tagalong_slot_view(j).x();
        let y = self.tagalong_slot_view(j).y();
        self.ancilla_set_xy(k, x, y.wrapping_add(4));
    }

    pub(super) fn ancilla_add_bush_poof(&mut self, x: u16, y: u16) {
        if !self.player_state_view().item_in_hand_has(0x40) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(0x3f, 4) {
            let mut poof = self.ancilla_slot_view_mut(k);
            poof.set_item_to_link(0);
            poof.set_timer(7);
            self.set_sound_effect_1_with_link_pan(21);
            self.ancilla_set_xy(k, x, y.wrapping_sub(2));
        }
    }

    pub(super) fn ancilla_add_victory_spin(&mut self) {
        if self.inventory_items().sword_type().wrapping_add(1) & 0xfe != 0 {
            if let Some(k) = self.ancilla_add_ancilla(0x3b, 0) {
                let mut spin = self.ancilla_slot_view_mut(k);
                spin.set_item_to_link(0);
                spin.set_work_byte_3(1);
                spin.set_aux_timer(34);
            }
        }
    }

    pub(super) fn ancilla_add_magic_powder(&mut self, a: u8, y: u8) {
        const MAGIC_POWER_X: [i8; 4] = [-2, -2, -12, 12];
        const MAGIC_POWER_Y: [i8; 4] = [0, 20, 16, 16];
        const MAGIC_POWER_X1: [i8; 4] = [10, 10, -8, 28];
        const MAGIC_POWER_Y1: [i8; 4] = [1, 40, 22, 22];

        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut powder = self.ancilla_slot_view_mut(k);
                powder.set_item_to_link(0);
                powder.set_z(0);
                powder.set_aux_timer(1);
            }
            self.player_state_view_mut().set_link_dma_staging_index(80);
            let j = self.player_state_view().facing_index();
            self.ancilla_slot_view_mut(k).set_direction(j as u8);
            let value = MAGIC_POWDER_FRAME_TIMERS[j * 10];
            self.ancilla_slot_view_mut(k).set_work_byte_25(value);
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(MAGIC_POWER_X[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(MAGIC_POWER_Y[j] as i16 as u16),
            );
            self.ancilla_check_tile_collision(k);
            let value = self.ancilla_slot_view(k).tile_attribute();
            self.dungeon_torch_mut().set_attr(value);
            if self.player_state_view().current_item_active() == 9 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
            self.set_sound_effect_1_with_link_pan(0x0d);
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(MAGIC_POWER_X1[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(MAGIC_POWER_Y1[j] as i16 as u16),
            );
        }
    }

    pub(super) fn ancilla_add_wall_tap_spark(&mut self, a: u8, y: u8) {
        const WALL_TAP_SPARK_X: [i8; 4] = [11, 10, -12, 29];
        const WALL_TAP_SPARK_Y: [i8; 4] = [-4, 32, 17, 17];
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut spark = self.ancilla_slot_view_mut(k);
                spark.set_item_to_link(5);
                spark.set_aux_timer(1);
            }
            let i = self.player_state_view().facing_index();
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(WALL_TAP_SPARK_X[i] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(WALL_TAP_SPARK_Y[i] as i16 as u16),
            );
        }
    }

    pub(super) fn ancilla_add_lamp_flame(&mut self, a: u8, y: u8) {
        const LAMP_FLAME_X: [i8; 4] = [0, 0, -20, 18];
        const LAMP_FLAME_Y: [i8; 4] = [-16, 24, 4, 4];
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            let j = self.player_state_view().facing_index();
            {
                let mut flame = self.ancilla_slot_view_mut(k);
                flame.set_item_to_link(0);
                flame.set_aux_timer(0);
                flame.set_timer(23);
                flame.set_direction(j as u8);
            }
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(LAMP_FLAME_X[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(LAMP_FLAME_Y[j] as i16 as u16),
            );
            self.set_sound_effect_1_with_ancilla_pan(k, 42);
        }
    }

    pub(super) fn ancilla_add_ms_cutscene(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut cutscene = self.ancilla_slot_view_mut(k);
                cutscene.set_item_to_link(0);
                cutscene.set_aux_timer(2);
                cutscene.set_timer(64);
            }
            self.ancilla_set_xy(
                k,
                self.player_state_view().x().wrapping_add(8),
                self.player_state_view().y().wrapping_sub(8),
            );
        }
    }

    pub(super) fn ancilla_add_dash_tremor(&mut self, a: u8, y: u8) {
        const ADD_DASH_TREMOR_DIR: [u8; 4] = [2, 2, 0, 0];
        const DASH_TREMOR_COORD_LIMITS: [u8; 2] = [0x80, 0x78];

        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ancilla_slot_view_mut(k).set_item_to_link(16);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_l(value);
            let mut j = self.player_state_view().facing_index();
            j = ADD_DASH_TREMOR_DIR[j] as usize;
            self.ancilla_slot_view_mut(k).set_direction(j as u8);
            let y = self
                .player_state_view()
                .y()
                .wrapping_sub(self.world_scroll().bg2_y()) as u8;
            let x = self
                .player_state_view()
                .x()
                .wrapping_sub(self.world_scroll().bg2_x()) as u8;
            let coord = if j != 0 { y } else { x };
            self.ancilla_set_y(
                k,
                if coord < DASH_TREMOR_COORD_LIMITS[j >> 1] {
                    3
                } else {
                    (-3i8) as u16
                },
            );
        }
    }

    fn ancilla_add_hookshot_wall_clink(&mut self, kin: usize, a: u8, y: u8) {
        const HOOKSHOT_WALL_HIT_X: [i8; 8] = [8, 8, 0, 10, 12, 8, 4, 0];
        const HOOKSHOT_WALL_HIT_Y: [i8; 8] = [0, 8, 8, 8, 4, 8, 12, 8];

        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut wall_clink = self.ancilla_slot_view_mut(k);
                wall_clink.set_item_to_link(0);
                wall_clink.set_work_byte_3(1);
            }
            let j = self.ancilla_slot_view(kin).direction() as usize;
            self.ancilla_set_xy(
                k,
                self.ancilla_get_x(kin)
                    .wrapping_add(HOOKSHOT_WALL_HIT_X[j] as i16 as u16),
                self.ancilla_get_y(kin)
                    .wrapping_add(HOOKSHOT_WALL_HIT_Y[j] as i16 as u16),
            );
        }
    }

    pub(super) fn ancilla_add_quake_spell(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut quake = self.ancilla_slot_view_mut(k);
                quake.set_step(0);
                quake.set_item_to_link(0);
                quake.set_timer(2);
            }
            self.set_chr_halfslot_request(13);
            self.system_signals_view_mut().set_sound_effect_1(0x35);
            for i in 0..5 {
                self.quake_bolt_view_mut(i).set_phase(0);
            }
            self.quake_spell_scratch_view_mut().set_active_bolt_limit(0);
            for i in 0..5 {
                self.quake_bolt_view_mut(i).set_timer(1);
            }
            self.player_state_view_mut()
                .set_custom_spell_animation_active();
            let quake_origin_y = self.player_state_view().y().wrapping_add(26);
            let quake_origin_x = self.player_state_view().x().wrapping_add(8);
            self.quake_spell_scratch_view_mut()
                .set_origin(quake_origin_x, quake_origin_y);
            self.quake_spell_scratch_view_mut().set_screen_shake_y(3);
        }
    }

    pub(super) fn ancilla_add_ether_spell(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut ether = self.ancilla_slot_view_mut(k);
                ether.set_item_to_link(0);
                ether.set_work_byte_25(0);
                ether.set_step(0);
                ether.set_aux_timer(2);
                ether.set_work_byte_3(3);
                ether.set_y_velocity(127);
            }
            self.player_state_view_mut()
                .set_custom_spell_animation_active();
            self.ether_orbit_view_mut().set_radius(40);
            self.set_chr_halfslot_request(9);
            self.ether_orbit_view_mut().set_spin_countdown(0x40);
            self.set_sound_effect_2_with_link_pan(0x26);
            for i in 0..8 {
                self.ether_orbit_view_mut().set_angle(i, (i * 8) as u8);
            }
            let ether_y = self.player_state_view().y();
            let ether_x = self.player_state_view().x();
            self.ether_orbit_view_mut()
                .set_orb_position(ether_x, ether_y);
            let y = self.world_scroll().bg2_y().wrapping_sub(16);
            self.ether_orbit_view_mut()
                .initialize_beam_adjusted_y(y & 0x00f0);
            let ether_y2 = ether_y.wrapping_sub(16);
            self.ether_orbit_view_mut().set_beam_y(ether_y2);
            self.ether_orbit_view_mut()
                .set_orbit_position(ether_x.wrapping_add(8), ether_y2.wrapping_add(0x24));
            self.ancilla_set_xy(k, ether_x, y);
        }
    }

    fn ancilla18_ether_spell(&mut self, k: usize) {
        if self.frame_state().submodule != 0 {
            return;
        }

        if self.ancilla_slot_view(k).step() != 0 {
            let flag = if self.player_state_view().spin_animation_step_counter() == 0 {
                self.ancilla_slot_view_mut(k).advance_work_byte_4() & 4 == 0
            } else {
                self.player_state_view().spin_animation_step_counter() == 11
            };
            if flag {
                self.palette_electro_themed_gear();
                self.filter_majorly_whiten_bg();
            } else {
                self.load_actual_gear_palettes();
                self.palette_restore_bg_from_flash();
            }
        }

        if self.ancilla_slot_view(k).step() == 2 {
            if sign8(self.ancilla_slot_view_mut(k).tick_aux_timer()) {
                let mut ether = self.ancilla_slot_view_mut(k);
                ether.set_aux_timer(2);
                if ether.advance_item_to_link() == 2 {
                    ether.retreat_item_to_link();
                    ether.set_x_velocity(16);
                    ether.set_step(3);
                }
            }
            self.ancilla_slot_view_mut(k).add_x_velocity(1);
            self.ether_spell_handle_radial_spin(k);
            return;
        }

        if sign8(self.ancilla_slot_view_mut(k).tick_aux_timer()) {
            let mut ether = self.ancilla_slot_view_mut(k);
            ether.set_aux_timer(2);
            ether.toggle_item_to_link_bit0();
        }
        match self.ancilla_slot_view(k).step() {
            0 => self.ether_spell_handle_lightning_stroke(k),
            1 => self.ether_spell_handle_orb_pulse(k),
            3 => self.ether_spell_handle_radial_spin(k),
            4 => {
                if self.ether_orbit_view_mut().tick_spin_countdown() == 0 {
                    self.ancilla_slot_view_mut(k).set_step(5);
                }
                self.ether_spell_handle_radial_spin(k);
            }
            _ => {
                let mut vel = self.ancilla_slot_view(k).x_velocity().wrapping_add(0x10);
                if sign8(vel) {
                    vel = 0x7f;
                }
                self.ancilla_slot_view_mut(k).set_x_velocity(vel);
                self.ether_spell_handle_radial_spin(k);
            }
        }
    }

    fn ether_spell_handle_lightning_stroke(&mut self, k: usize) {
        self.ancilla_move_y(k);
        let y = self.ancilla_get_y(k);

        if self.ether_orbit_view().beam_top_bucket() != (y & 0xf0) as u8 {
            self.ether_orbit_view_mut()
                .set_beam_top_bucket((y & 0xf0) as u8);
            self.ancilla_slot_view_mut(k).advance_work_byte_25();
        }
        if y < 0xe000
            && self.ether_orbit_view().beam_y() < 0xe000
            && self.ether_orbit_view().beam_y() <= y
        {
            self.ancilla_slot_view_mut(k).set_step(1);
        }
        self.ancilla_draw_ether_blitz(k);
    }

    fn ether_spell_handle_orb_pulse(&mut self, k: usize) {
        if !sign8(self.ancilla_slot_view(k).work_byte_25()) {
            if !sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3()) {
                self.ancilla_draw_ether_blitz(k);
                return;
            }
            self.ancilla_slot_view_mut(k).set_work_byte_3(3);
            if !sign8(self.ancilla_slot_view_mut(k).retreat_work_byte_25()) {
                self.ancilla_draw_ether_blitz(k);
                return;
            }
            self.ancilla_slot_view_mut(k).set_work_byte_3(9);
        }
        if sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3()) {
            {
                let mut ether = self.ancilla_slot_view_mut(k);
                ether.set_step(2);
                ether.set_y_velocity(0);
                ether.set_x_velocity(16);
                ether.set_item_to_link(0);
                ether.set_aux_timer(2);
            }
            if self.player_state_view().spin_animation_step_counter() != 0 {
                self.medallion_check_sprite_damage(k);
            }
        }
        let oam = self.oam_state_view().current_pointer_usize();
        self.ancilla_draw_ether_orb(k, oam);
    }

    fn ether_spell_handle_radial_spin(&mut self, k: usize) {
        if self.ancilla_slot_view(k).step() == 4 {
            if self.frame_state().frame_counter & 7 == 0 {
                self.system_signals_view_mut().set_sound_effect_2(0x2a);
            } else if self.frame_state().frame_counter & 7 == 4 {
                self.system_signals_view_mut().set_sound_effect_2(0xaa);
            } else if self.frame_state().frame_counter & 7 == 7 {
                self.system_signals_view_mut().set_sound_effect_2(0x6a);
            }
        } else {
            let radius = self.ether_orbit_view().radius();
            self.ancilla_slot_view_mut(k).set_x(u16::from(radius));
            self.ancilla_move_x(k);
            let radius = self.ancilla_slot_view(k).x() as u8;
            self.ether_orbit_view_mut().set_radius(radius);
            if self.ether_orbit_view().radius() == 0x40 {
                self.ancilla_slot_view_mut(k).set_step(4);
            }
        }

        let sb = self.ancilla_slot_view(k).step();
        let sa = self.ancilla_slot_view(k).item_to_link() as usize;
        let mut oam = self.oam_state_view().current_pointer_usize();
        for i in (0..=7).rev() {
            let mut angle = self.ether_orbit_view().angle(i);
            if sb != 2 && sb != 5 {
                angle = self.ether_orbit_view_mut().advance_angle(i);
            }
            let arp = self.ancilla_get_radial_projection(angle, self.ether_orbit_view().radius());
            if sb != 2 {
                oam = self.ancilla_draw_ether_blitz_ball(oam, &arp, sa);
            } else {
                oam = self.ancilla_draw_ether_blitz_segment(oam, &arp, sa, i);
            }
        }
        if self.ether_orbit_view().radius() < 0xf0 {
            let oam = self.oam_state_view().current_pointer_usize();
            for i in 0..8 {
                if self.oam_state_view().entry_y(oam + i * 4) != 0xf0 {
                    return;
                }
            }
        }

        self.ancilla_slot_view_mut(k).clear();
        self.set_chr_halfslot_request(1);
        self.player_state_view_mut().clear_spin_attack_sound_latch();
        self.player_state_view_mut().clear_state_for_spin_attack();
        self.player_state_view_mut()
            .clear_spin_animation_step_counter();
        self.player_state_view_mut().clear_direction_lock();
        self.clear_modal_pause_flag();

        if self.world_location_state().overworld_screen_index() == 0x70
            && self.overworld_event_info_view().event_info(0x70) & 0x20 == 0
            && self.ancilla_check_for_entrance_trigger(2)
        {
            self.set_special_entrance_trigger(3);
            self.set_subsubmodule(0);
            self.scratch_word_view_mut()
                .clear_module_transition_counter();
        }

        if self.player_state_view().handler_state() != 25 {
            self.player_state_view_mut().clear_handler_state();
            self.player_state_view_mut().set_spin_attack_delay_timer(0);
            let button_mask_b_y = if self.player_state_view().button_b_frames() != 0 {
                self.player_state_view().joypad1h_last() & 0x80
            } else {
                0
            };
            self.player_state_view_mut()
                .set_button_mask_b_y(button_mask_b_y);
        }
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().clear_magic_spell_player_lock();
        self.load_actual_gear_palettes();
        self.palette_restore_bg_and_hud();
    }

    pub(super) fn ancilla_add_bombos_spell(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_add_ancilla_bank08(a, y) else {
            return;
        };
        for i in 0..10 {
            self.bombos_fire_column_view_mut(i).set_phase(0);
            self.bombos_fire_column_view_mut(i).set_timer(3);
        }
        for i in 0..8 {
            self.bombos_blast_view_mut(i).set_phase(0);
            self.bombos_blast_view_mut(i).set_timer(3);
        }
        self.bombos_spell_scratch_view_mut().set_mode(0);
        self.bombos_spell_scratch_view_mut()
            .set_blast_release_locked(false);
        self.bombos_spell_scratch_view_mut()
            .set_blast_release_countdown(0x80);
        self.bombos_fire_column_view_mut(0).set_radial_angle(0x10);
        self.set_chr_halfslot_request(11);
        self.player_state_view_mut()
            .set_custom_spell_animation_active();
        {
            let mut bombos = self.ancilla_slot_view_mut(k);
            bombos.set_step(0);
            bombos.set_item_to_link(0);
        }
        self.ancilla_sfx2_near(0x2a);

        let mut t = self.asset_u8(72, self.frame_state().frame_counter as usize);
        t = if t < 0xe0 { t } else { t & 0x7f };
        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        self.bombos_spell_scratch_view_mut().set_blast_position(
            0,
            (link_x & !0xff) | u16::from(t),
            (link_y & !0xff) | u16::from(t),
        );

        const BOMBOS_Y_DELTA: [i16; 4] = [16, 24, -128, -16];
        const BOMBOS_X_DELTA: [i16; 4] = [-16, -128, 0, 128];

        for i in 0..1 {
            let bombos_x_coord2 = link_x.wrapping_add(BOMBOS_X_DELTA[i] as u16);
            let bombos_y_coord2 = link_y.wrapping_add(BOMBOS_Y_DELTA[i] as u16);
            self.bombos_spell_scratch_view_mut()
                .set_fire_column_seed_position(i, bombos_x_coord2, bombos_y_coord2);
            self.bombos_spell_scratch_view_mut()
                .set_fire_column_radius(16);
            let arp = self
                .ancilla_get_radial_projection(self.bombos_fire_column_view(i).radial_angle(), 16);
            let x = (if arp.r6 != 0 {
                -(arp.r4 as i32)
            } else {
                arp.r4 as i32
            }) + i32::from(bombos_x_coord2);
            let y = (if arp.r2 != 0 {
                -(arp.r0 as i32)
            } else {
                arp.r0 as i32
            }) + i32::from(bombos_y_coord2);
            self.bombos_fire_column_view_mut(i)
                .set_position(x as u16, y as u16);
        }
    }

    fn ancilla19_bombos_spell(&mut self, k: usize) {
        if self.bombos_spell_scratch_view().mode() == 0 {
            if self.frame_state().submodule == 0 {
                self.bombos_spell_control_fire_columns(k);
                return;
            }
            for i in (0..=9).rev() {
                self.ancilla_draw_bombos_fire_column(i);
            }
        } else if self.bombos_spell_scratch_view().mode() != 2 {
            if self.frame_state().submodule == 0 {
                self.bombos_spell_finish_fire_columns(k);
                return;
            }
            for i in (0..=9).rev() {
                self.ancilla_draw_bombos_fire_column(i);
            }
        } else {
            if self.frame_state().submodule == 0 {
                self.bombos_spell_control_blasting(k);
                return;
            }
            let mut i = self.ancilla_slot_view(k).step() as i32;
            loop {
                self.ancilla_draw_bombos_blast(i as usize);
                i -= 1;
                if i < 0 {
                    break;
                }
            }
        }
    }

    fn bombos_spell_control_fire_columns(&mut self, k: usize) {
        let sa = self.ancilla_slot_view(k).item_to_link();
        let mut sb = self.ancilla_slot_view(k).step();

        let mut i = sb as i32;
        loop {
            let ui = i as usize;
            if self.bombos_fire_column_view(ui).phase() != 13 {
                let timer = self.bombos_fire_column_view_mut(ui).tick_timer();
                if sign8(timer) {
                    self.bombos_fire_column_view_mut(ui).set_timer(3);
                    let phase = self.bombos_fire_column_view_mut(ui).advance_phase();
                    if phase != 13 {
                        if phase == 2 && sa == 0 {
                            let j = if sb == 9 {
                                let mut found: Option<usize> = None;
                                for candidate in (0..=9).rev() {
                                    if self.bombos_fire_column_view(candidate).phase() == 13 {
                                        self.bombos_fire_column_view_mut(candidate).set_phase(0);
                                        found = Some(candidate);
                                        break;
                                    }
                                }
                                found.unwrap_or(9)
                            } else {
                                sb = if sb.wrapping_add(1) != 10 {
                                    sb.wrapping_add(1)
                                } else {
                                    9
                                };
                                sb as usize
                            };
                            let radius = self
                                .bombos_spell_scratch_view_mut()
                                .grow_fire_column_radius(3, 207);
                            let angle =
                                self.bombos_fire_column_view_mut(0).add_radial_angle(6) & 0x3f;
                            let arp = self.ancilla_get_radial_projection(angle, radius);
                            let x = (if arp.r6 != 0 {
                                -(arp.r4 as i32)
                            } else {
                                arp.r4 as i32
                            }) + i32::from(
                                self.bombos_spell_scratch_view().fire_column_seed_x(0),
                            );
                            let y = (if arp.r2 != 0 {
                                -(arp.r0 as i32)
                            } else {
                                arp.r0 as i32
                            }) + i32::from(
                                self.bombos_spell_scratch_view().fire_column_seed_y(0),
                            );
                            self.bombos_fire_column_view_mut(j)
                                .set_position(x as u16, y as u16);

                            let t = (x as u16)
                                .wrapping_sub(self.world_scroll().bg2_x())
                                .wrapping_add(8);
                            if t < 256 {
                                self.system_signals_view_mut().set_sound_effect_1(
                                    BOMBOS_PANNED_SFX_BITS[(t >> 5) as usize] | 0x2a,
                                );
                            }
                        }
                    } else {
                        i -= 1;
                        if i < 0 {
                            break;
                        }
                        continue;
                    }
                }
                self.ancilla_draw_bombos_fire_column(ui);
            }

            i -= 1;
            if i < 0 {
                break;
            }
        }
        if self.bombos_fire_column_view(0).radial_angle() >= 0x80 {
            self.bombos_spell_scratch_view_mut().set_mode(1);
        }
        self.ancilla_slot_view_mut(k).set_step(sb);
    }

    fn bombos_spell_finish_fire_columns(&mut self, kk: usize) {
        let mut k = self.ancilla_slot_view(kk).step() as i32;
        loop {
            let uk = k as usize;
            let timer = self.bombos_fire_column_view_mut(uk).tick_timer();
            if sign8(timer) {
                self.bombos_fire_column_view_mut(uk).set_timer(3);
                let phase = self.bombos_fire_column_view_mut(uk).advance_phase();
                if phase >= 13 {
                    self.bombos_fire_column_view_mut(uk).set_phase(13);
                }
            }
            self.ancilla_draw_bombos_fire_column(uk);
            k -= 1;
            if k < 0 {
                break;
            }
        }
        for k in (0..=9).rev() {
            if self.bombos_fire_column_view(k).phase() != 13 {
                return;
            }
        }
        self.bombos_spell_scratch_view_mut().set_mode(2);
        self.medallion_check_sprite_damage(kk);
        self.ancilla_slot_view_mut(kk).set_step(0);
    }

    fn bombos_spell_control_blasting(&mut self, kk: usize) {
        let mut k = self.ancilla_slot_view(kk).step() as i32;
        let mut sb = k;
        while k >= 0 {
            let uk = k as usize;
            if self.bombos_blast_view(uk).phase() != 8 {
                let timer = self.bombos_blast_view_mut(uk).tick_timer();
                if sign8(timer) {
                    self.bombos_blast_view_mut(uk).set_timer(3);
                    let phase = self.bombos_blast_view_mut(uk).advance_phase();
                    if phase == 1 && !self.bombos_spell_scratch_view().blast_release_locked() {
                        let mut j = sb;
                        if j != 15 {
                            sb += 1;
                            j = sb;
                        } else {
                            while j >= 0 && self.bombos_blast_view(j as usize).phase() != 8 {
                                j -= 1;
                            }
                        }
                        let uj = j as usize;
                        self.bombos_blast_view_mut(uj).set_phase(0);
                        self.bombos_blast_view_mut(uj).set_timer(3);

                        let idx = (self.frame_state().frame_counter & 0x3f) as usize;
                        let y = u16::from(BOMBOS_BLAST_POSITION_TABLE[idx]);
                        let x = u16::from(BOMBOS_BLAST_POSITION_TABLE[idx + 3]);
                        let bg2vofs_copy2 = self.world_scroll().bg2_y();
                        let bg2hofs_copy2 = self.world_scroll().bg2_x();
                        self.bombos_spell_scratch_view_mut().set_blast_position(
                            uj,
                            x.wrapping_add(bg2hofs_copy2),
                            y.wrapping_add(bg2vofs_copy2),
                        );
                        let bombos_x = self.bombos_spell_scratch_view().blast_x(uj);
                        self.system_signals_view_mut().set_sound_effect_1(
                            0x0c | BOMBOS_PANNED_SFX_BITS[((bombos_x >> 5) & 7) as usize],
                        );
                    }
                }
            }
            self.ancilla_draw_bombos_blast(uk);
            k -= 1;
        }

        for j in (0..=15).rev() {
            if self.bombos_blast_view(j).phase() != 8 {
                self.ancilla_slot_view_mut(kk).set_step(sb as u8);
                if self
                    .bombos_spell_scratch_view_mut()
                    .tick_blast_release_countdown()
                    == 0
                {
                    self.bombos_spell_scratch_view_mut()
                        .set_blast_release_countdown(1);
                    self.bombos_spell_scratch_view_mut()
                        .set_blast_release_locked(true);
                }
                return;
            }
        }
        self.ancilla_slot_view_mut(kk).clear();
        self.set_chr_halfslot_request(1);
        self.player_state_view_mut().clear_spin_attack_sound_latch();
        self.player_state_view_mut().clear_state_for_spin_attack();
        self.player_state_view_mut()
            .clear_spin_animation_step_counter();
        self.player_state_view_mut().clear_direction_lock();
        self.clear_modal_pause_flag();
        if self.player_state_view().handler_state() != 26 {
            self.player_state_view_mut().clear_handler_state();
            self.player_state_view_mut().set_spin_attack_delay_timer(0);
            let button_mask_b_y = if self.player_state_view().button_b_frames() != 0 {
                self.player_state_view().joypad1h_last() & 0x80
            } else {
                0
            };
            self.player_state_view_mut()
                .set_button_mask_b_y(button_mask_b_y);
        }
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().clear_magic_spell_player_lock();

        if self
            .bombos_spell_scratch_view_mut()
            .tick_blast_release_countdown()
            == 0
        {
            self.bombos_spell_scratch_view_mut()
                .set_blast_release_countdown(1);
            self.bombos_spell_scratch_view_mut()
                .set_blast_release_locked(true);
        }
    }

    pub(super) fn ancilla_add_gt_cutscene(&mut self) {
        if self.player_state_view().is_lifting_or_carrying()
            || self.player_state_view().has_auxiliary_state()
            || self.player_resources_view().crystal_flags() & 0x7f != 0x7f
            || self.overworld_event_info_view().event_info(0x43) & 0x20 != 0
        {
            return;
        }

        self.ancilla_terminate_sparkle_objects_for_ancilla();

        if self.ancilla_add_check_for_presence(0x43) {
            return;
        }

        let Some(k) = self.ancilla_add_ancilla(0x43, 4) else {
            return;
        };

        for i in (0..=15).rev() {
            if self.sprite_slot_view(i).sprite_type() == 0x37 {
                let value = 0;
                self.sprite_slot_view_mut(i).set_state(value);
            }
        }

        for i in (0..=0x17).rev() {
            self.tower_seal_sparkle_view_mut(i).set_phase(0xff);
        }
        self.DecodeAnimatedSpriteTile_variable(0x28);
        self.palette_buffer_view_mut().set_sp6r_indoors(4);
        self.palette_buffer_view_mut()
            .select_overworld_aux_palette_offset();
        self.palette_load_sprite_environment_dungeon();
        self.system_signals_view_mut().increment_cgram_update_flag();
        self.player_state_view_mut().immobilize();
        let value = 0;
        self.ancilla_slot_view_mut(k).set_y_subpixel(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_x_subpixel(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_step(value);
        self.tower_seal_scratch_view_mut().set_wait_countdown(240);
        self.tower_seal_scratch_view_mut().set_ring_radius(0);

        self.tower_seal_orbit_view_mut(0).set_angle(0);
        self.tower_seal_orbit_view_mut(1).set_angle(10);
        self.tower_seal_orbit_view_mut(2).set_angle(22);
        self.tower_seal_orbit_view_mut(3).set_angle(32);
        self.tower_seal_orbit_view_mut(4).set_angle(42);
        self.tower_seal_orbit_view_mut(5).set_angle(54);

        self.ancilla_set_xy(
            k,
            self.player_state_view().x(),
            self.player_state_view().y().wrapping_sub(16),
        );
    }

    fn ancilla_terminate_sparkle_objects_for_ancilla(&mut self) {
        for i in (0..=4).rev() {
            let t = self.ancilla_slot_view(i).ancilla_type();
            if t == 0x2a
                || t == 0x2b
                || t == 0x30
                || t == 0x31
                || t == 0x18
                || t == 0x19
                || t == 0x0c
            {
                let value = 0;
                self.ancilla_slot_view_mut(i).set_ancilla_type(value);
            }
        }
    }

    pub(super) fn ancilla_add_blast_wall(&mut self) {
        const BLAST_WALL_FRAGMENT_X_OFFSET: [i8; 4] = [-16, 16, 0, 0];
        const BLAST_WALL_FRAGMENT_Y_OFFSET: [i8; 4] = [0, 0, -16, 16];
        const BLAST_WALL_FRAGMENT_OFFSET: [SignedOffset; 8] =
            signed_offsets![-8, 0, -8, 16, 16, 0, 16, 16, 0, -8, 16, -8, 0, 16, 16, 16,];

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
        self.player_state_view_mut().clear_ancilla_pickup_flag();
        self.player_state_view_mut().clear_state_bits();
        self.player_state_view_mut().clear_direction_lock();
        self.ancilla_slot_view_mut(0).set_k(0);
        let player_floor = self.player_state_view().lower_level_state();
        self.ancilla_slot_view_mut(0).set_floor(player_floor);
        let value = player_floor;
        self.ancilla_slot_view_mut(1).set_floor(value);
        let value = self.player_state_view().lower_level_mirror_state();
        self.ancilla_slot_view_mut(0).set_floor2(value);
        self.blast_wall_scratch_view_mut().clear_entry_state();
        self.blast_wall_explosion_view_mut(1).set_timer(0);
        self.blast_wall_explosion_view_mut(1).set_phase(0);
        self.blast_wall_scratch_view_mut().clear_secondary_state();
        self.blast_wall_explosion_view_mut(0).set_phase(1);
        self.player_state_view_mut()
            .set_custom_spell_animation_active();
        self.blast_wall_explosion_view_mut(0).set_timer(3);

        let mut j = self.blast_wall_scratch_view().direction() as usize;
        let (blast_wall_center_x, blast_wall_center_y) =
            self.blast_wall_scratch_view_mut().offset_center(
                BLAST_WALL_FRAGMENT_Y_OFFSET[j],
                BLAST_WALL_FRAGMENT_X_OFFSET[j],
            );
        j = if j < 4 { 4 } else { 0 };
        for k in (0..=3).rev() {
            let offset = BLAST_WALL_FRAGMENT_OFFSET[j];
            let y = blast_wall_center_y.wrapping_add(offset.y as i16 as u16);
            let x = blast_wall_center_x.wrapping_add(offset.x as i16 as u16);
            self.blast_wall_fragment_view_mut(k).set_position(x, y);
            let x = x.wrapping_sub(self.world_scroll().bg2_x());
            if x < 256 {
                self.system_signals_view_mut()
                    .set_sound_effect_1(BOMBOS_PANNED_SFX_BITS[(x >> 5) as usize] | 0x0c);
            }
            j += 1;
        }
    }

    pub(super) fn add_bird_travel_something(&mut self, a: u8, y: u8) {
        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_simple(a, y) {
            self.player_state_view_mut().clear_handler_state();
            self.player_state_view_mut().set_speed_setting(0);
            self.player_state_view_mut()
                .clear_button_mask_b_y_bits(0x81);
            self.player_state_view_mut().clear_button_b_frames();
            self.player_state_view_mut().set_spin_attack_delay_timer(0);
            self.player_state_view_mut().clear_direction_lock_bits(1);
            let value = 1;
            self.ancilla_slot_view_mut(k).set_l(value);

            let enhanced_bird_travel = self.enhanced_features_view().has(1);
            let mut bird = self.ancilla_slot_view_mut(k);
            if enhanced_bird_travel {
                bird.set_z_velocity(58);
                bird.set_z((-105i8) as u8);
            } else {
                bird.set_z_velocity(40);
                bird.set_z((-51i8) as u8);
            }
            bird.set_step(2);
            self.add_bird_common(k);
        }
    }

    fn ancilla_add_check_for_presence(&self, a: u8) -> bool {
        (0..=5)
            .rev()
            .any(|k| self.ancilla_slot_view(k).ancilla_type() == a)
    }

    pub(super) fn add_happiness_pond_rupees(&mut self, arg: u8) {
        let Some(_) = self.ancilla_add_simple(0x42, 9) else {
            return;
        };
        self.set_sound_effect_2_with_link_pan(0x13);
        self.DecodeAnimatedSpriteTile_variable(0x24);
        self.player_state_view_mut().enter_item_hold_pose();

        for i in 0..10 {
            self.happiness_pond_rupee_view_mut(i).clear();
        }

        const HAPPINESS_POND_START: [i8; 4] = [0, 4, 4, 9];
        const HAPPINESS_POND_END: [i8; 4] = [-1, 0, -1, -1];
        const HAPPINESS_POND_XVEL: [i8; 10] = [0, -12, -6, 6, 12, -9, -5, 0, 5, 9];
        const HAPPINESS_POND_YVEL: [i8; 10] = [-40, -40, -40, -40, -40, -32, -32, -32, -32, -32];
        const HAPPINESS_POND_ZVEL: [i8; 10] = [20, 20, 20, 20, 20, 16, 16, 16, 16, 16];

        let mut j = HAPPINESS_POND_START[arg as usize];
        let j_end = HAPPINESS_POND_END[arg as usize];
        let mut k = 9usize;
        loop {
            let x = self.player_state_view().x().wrapping_add(4);
            let y = self.player_state_view().y().wrapping_sub(12);
            self.happiness_pond_rupee_view_mut(k).initialize(
                x,
                y,
                HAPPINESS_POND_XVEL[j as usize] as u8,
                HAPPINESS_POND_YVEL[j as usize] as u8,
                HAPPINESS_POND_ZVEL[j as usize] as u8,
            );
            if k == 0 {
                break;
            }
            k -= 1;
            j -= 1;
            if j == j_end {
                break;
            }
        }
    }

    pub(super) fn ancilla_add_snoring(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_simple(a, y) else {
            return;
        };
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_item_to_link(0);
            ancilla.set_y_velocity((-8i8) as u8);
            ancilla.set_x_velocity(8);
            ancilla.set_aux_timer(7);
            ancilla.set_step(255);
        }
        self.ancilla_set_xy(
            k,
            self.player_state_view().x().wrapping_add(16),
            self.player_state_view().y().wrapping_add(4),
        );
    }

    pub(super) fn ancilla_add_bomb(&mut self, a: u8, y: u8) {
        const BOMB_PLACE_X0: [i8; 4] = [8, 8, 0, 16];
        const BOMB_PLACE_Y0: [i8; 4] = [0, 24, 12, 12];
        const BOMB_PLACE_X1: [i8; 4] = [8, 8, -6, 22];
        const BOMB_PLACE_Y1: [i8; 4] = [4, 28, 12, 12];

        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return;
        };
        if self.player_resources_view().bombs() == 0 {
            self.ancilla_slot_view_mut(k).clear();
            return;
        }

        if self.player_resources_view_mut().decrement_bombs() == 0 {
            self.hud_refresh_icon();
        }

        let value = 0;

        self.ancilla_slot_view_mut(k).set_r(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        let direction = self.player_state_view().facing() >> 1;
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
                self.player_state_view()
                    .x()
                    .wrapping_add(BOMB_PLACE_X0[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(BOMB_PLACE_Y0[j] as i16 as u16),
            );
        } else {
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(BOMB_PLACE_X1[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(BOMB_PLACE_Y1[j] as i16 as u16),
            );
        }
        self.set_sound_effect_1_with_link_pan(0x0b);
    }

    pub(super) fn ancilla_add_boomerang(&mut self, a: u8, y: u8) -> u8 {
        const BOOMERANG_SPEED_BY_TYPE: [u8; 4] = [0x20, 0x18, 0x30, 0x28];
        const BOOMERANG_INITIAL_STEP_BY_TYPE: [u8; 2] = [0x20, 0x60];
        const BOOMERANG_FRAME_RESET_BY_TYPE: [u8; 2] = [3, 2];
        const BOOMERANG_DIRECTION_BITS: [u8; 4] = [8, 4, 2, 1];
        const BOOMERANG_INPUT_MASKS: [u8; 8] = [8, 4, 2, 1, 9, 5, 10, 6];
        const BOOMERANG_INITIAL_SPIN_BY_INPUT: [u8; 8] = [2, 3, 3, 2, 2, 3, 3, 3];
        const BOOMERANG_INITIAL_Y_OFFSET: [i8; 8] = [-10, -8, -9, -9, -10, -8, -9, -9];
        const BOOMERANG_INITIAL_X_OFFSET: [i8; 8] = [-10, 11, 8, -8, -10, 11, 8, -8];
        const BOOMERANG_CHARGED_Y_OFFSET: [i8; 8] = [-16, 6, 0, 0, -8, 8, -8, 8];
        const BOOMERANG_CHARGED_X_OFFSET: [i8; 8] = [0, 0, -8, 8, 8, 8, -8, -8];

        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return 0;
        };
        let value = 0;
        self.ancilla_slot_view_mut(k).set_k(value);
        {
            let mut boomerang = self.ancilla_slot_view_mut(k);
            boomerang.set_aux_timer(0);
            boomerang.set_item_to_link(0);
            boomerang.set_z(0);
        }
        let value = self.ancilla_slot_view(k).num_sprites();
        self.ancilla_slot_view_mut(k).set_l(value);
        self.minigame_state_view_mut()
            .set_flag_boomerang_in_place(1);
        let mut j = self.inventory_items().boomerang().wrapping_sub(1) as usize;
        let value = j as u8;
        self.ancilla_slot_view_mut(k).set_g(value);
        {
            let mut boomerang = self.ancilla_slot_view_mut(k);
            boomerang.set_step(BOOMERANG_INITIAL_STEP_BY_TYPE[j]);
            boomerang.set_work_byte_3(BOOMERANG_FRAME_RESET_BY_TYPE[j]);
        }

        let s = self.ancilla_slot_view(k).g() as usize * 2
            + if self.player_state_view().joypad1h_last() & 0x0c != 0
                && self.player_state_view().joypad1h_last() & 3 != 0
            {
                1
            } else {
                0
            };
        let r0 = BOOMERANG_SPEED_BY_TYPE[s];
        let value = r0;
        self.ancilla_slot_view_mut(k).set_h(value);

        let r1 = if self.player_state_view().joypad1h_last() & 0x0f != 0 {
            self.player_state_view().joypad1h_last() & 0x0f
        } else {
            BOOMERANG_DIRECTION_BITS[self.player_state_view().facing_index()]
        };
        self.messaging_state_view_mut().clear_effect_index();

        if r1 & 0x0c != 0 {
            let y_velocity = if r1 & 8 != 0 { (-(r0 as i8)) as u8 } else { r0 };
            self.ancilla_slot_view_mut(k).set_y_velocity(y_velocity);
            let i = if sign8(y_velocity) { 0 } else { 1 };
            self.ancilla_slot_view_mut(k).set_direction(i);
            self.messaging_state_view_mut()
                .set_effect_index(BOOMERANG_DIRECTION_BITS[i as usize]);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_s_player(value);

        if r1 & 3 != 0 {
            if r1 & 2 == 0 {
                let value = 1;
                self.ancilla_slot_view_mut(k).set_s_player(value);
            }
            let x_velocity = if r1 & 2 != 0 { (-(r0 as i8)) as u8 } else { r0 };
            self.ancilla_slot_view_mut(k).set_x_velocity(x_velocity);
            let i = if sign8(x_velocity) { 2 } else { 3 };
            self.ancilla_slot_view_mut(k).set_direction(i);
            self.messaging_state_view_mut()
                .or_effect_index(BOOMERANG_DIRECTION_BITS[i as usize]);
        }

        j = BOOMERANG_INPUT_MASKS
            .iter()
            .position(|&v| v == r1)
            .unwrap_or(0);
        let value = BOOMERANG_INITIAL_SPIN_BY_INPUT[j];
        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
        let value = (j << 1) as u8;
        self.ancilla_slot_view_mut(k).set_work_byte_23(value);
        if self.player_state_view().button_b_frames() >= 9 {
            self.ancilla_slot_view_mut(k).add_aux_timer(1);
        } else if s != 0 || self.player_state_view().joypad1h_last() & 0x0f == 0 {
            j = self.player_state_view().facing_index();
        }

        let s = self.ancilla_check_initial_tile_a(k);
        if s < 0 {
            if self.ancilla_slot_view(k).aux_timer() != 0 {
                self.ancilla_set_xy(
                    k,
                    self.player_state_view()
                        .x()
                        .wrapping_add(BOOMERANG_CHARGED_X_OFFSET[j] as i16 as u16),
                    self.player_state_view()
                        .y()
                        .wrapping_add(8)
                        .wrapping_add(BOOMERANG_CHARGED_Y_OFFSET[j] as i16 as u16),
                );
            } else {
                self.ancilla_set_xy(
                    k,
                    self.player_state_view()
                        .x()
                        .wrapping_add(BOOMERANG_INITIAL_X_OFFSET[j] as i16 as u16),
                    self.player_state_view()
                        .y()
                        .wrapping_add(8)
                        .wrapping_add(BOOMERANG_INITIAL_Y_OFFSET[j] as i16 as u16),
                );
            }
        } else {
            self.ancilla_slot_view_mut(k).clear();
            self.minigame_state_view_mut()
                .clear_flag_boomerang_in_place();
            let effect = if self.ancilla_slot_view(k).tile_attribute() != 0xf0 {
                5
            } else {
                6
            };
            self.set_sound_effect_1_with_ancilla_pan(k, effect);
            self.ancilla_add_boomerang_wall_clink(k);
        }
        if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
            && k == 4
            && self.frame_state().frame_counter >= 140
            && self.frame_state().frame_counter <= 210
        {
            let boomerang = self.ancilla_slot_view(k);
            eprintln!(
                "R boomerang-add fc={} k={} s={} type=0x{:02x} x={:04x} y={:04x} xv={:02x} yv={:02x} step={:02x} aux={:02x} item={:02x} K={:02x} dir={:02x} work23={:02x} link={:04x}/{:04x} joy={:02x} bframes={:02x}",
                self.frame_state().frame_counter,
                k,
                s,
                boomerang.ancilla_type(),
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                boomerang.x_velocity(),
                boomerang.y_velocity(),
                self.ancilla_slot_view(k).step(),
                self.ancilla_slot_view(k).aux_timer(),
                boomerang.item_to_link(),
                self.ancilla_slot_view(k).k(),
                boomerang.direction(),
                self.ancilla_slot_view(k).work_byte_23(),
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.player_state_view().joypad1h_last(),
                self.player_state_view().button_b_frames(),
            );
        }
        s as u8
    }

    pub(super) fn ancilla_add_tossed_pond_item(&mut self, a: u8, xin: u8, yin: u8) {
        const WISH_POND_ITEM_X: [u8; 76] = [
            4, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 4, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 2, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 0, 4, 0, 0, 0, 4, 0, 0,
        ];
        const WISH_POND_ITEM_Y: [i8; 76] = [
            -13, -13, -13, -13, -13, -12, -12, -13, -13, -12, -12, -12, -10, -12, -12, -12, -12,
            -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12,
            -12, -13, -12, -12, -12, -12, -12, -12, -10, -12, -12, -12, -12, -12, -12, -12, -12,
            -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12,
            -12, -12, -12, -12, -12, -13, -12, -12,
        ];

        self.player_state_view_mut().set_receive_item_index(xin);
        if let Some(k) = self.ancilla_add_ancilla(a, yin) {
            self.set_sound_effect_2_with_link_pan(0x13);
            let sb = RECEIVE_ITEM_GRAPHICS[xin as usize];
            if sb != 0xff {
                if sb == 0x20 {
                    self.DecompressShieldGraphics();
                }
                self.DecodeAnimatedSpriteTile_variable(sb);
            } else {
                self.DecodeAnimatedSpriteTile_variable(0);
            }
            if sb == 6 {
                self.DecompressSwordGraphics();
            }

            self.player_state_view_mut().enter_item_hold_pose();
            let receive_item = self.player_state_view().receive_item_index();
            {
                let mut item = self.ancilla_slot_view_mut(k);
                item.set_z_velocity(20);
                item.set_y_velocity((-40i8) as u8);
                item.set_x_velocity(0);
                item.set_z(0);
                item.set_timer(16);
                item.set_item_to_link(receive_item);
            }
            self.ancilla_set_xy(
                k,
                self.player_state_view().x().wrapping_add(
                    WISH_POND_ITEM_X[self.player_state_view().receive_item_index() as usize] as u16,
                ),
                self.player_state_view().y().wrapping_add(
                    WISH_POND_ITEM_Y[self.player_state_view().receive_item_index() as usize] as i16
                        as u16,
                ),
            );
        }
    }

    fn ancilla_add_cutscene_duck(&mut self, a: u8, y: u8) {
        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut duck = self.ancilla_slot_view_mut(k);
                duck.set_direction(2);
                duck.set_work_byte_3(3);
                duck.set_step(0);
                duck.set_aux_timer(32);
                duck.set_item_to_link(116);
                duck.set_z_velocity(0);
                duck.set_z(0);
            }
            let value = 0;
            self.ancilla_slot_view_mut(k).set_l(value);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_s_player(value);
            self.ancilla_set_xy(k, 0x0200, 0x0788);
        }
    }

    pub(super) fn ancilla_add_exploding_weather_vane(&mut self, a: u8, y: u8) {
        const WEATHERVANE_DEBRIS_X_VELOCITY: [i8; 12] =
            [8, 10, 9, 4, 11, 12, -10, -8, 4, -6, -10, -4];
        const WEATHERVANE_DEBRIS_Y_VELOCITY: [i8; 12] =
            [20, 22, 20, 20, 22, 20, 20, 22, 20, 22, 20, 20];
        const WEATHERVANE_DEBRIS_START_Y: [u8; 12] = [
            0xb0, 0xa3, 0xa0, 0xa2, 0xa0, 0xa8, 0xa0, 0xa0, 0xa8, 0xa1, 0xb0, 0xa0,
        ];
        const WEATHERVANE_DEBRIS_START_X: [u8; 12] = [0, 2, 4, 6, 3, 8, 14, 8, 12, 7, 10, 8];
        const WEATHERVANE_DEBRIS_CHAR: [u8; 12] = [48, 18, 32, 20, 22, 24, 32, 20, 24, 22, 20, 32];

        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return;
        };

        {
            let mut weather_vane = self.ancilla_slot_view_mut(k);
            weather_vane.set_aux_timer(10);
            weather_vane.set_step(0);
            weather_vane.set_work_byte_3(0);
        }
        let value = 128;
        self.ancilla_slot_view_mut(k).set_g(value);
        self.system_signals_view_mut().set_sound_effect_1(0);
        self.system_signals_view_mut().set_music_control(0xf2);
        self.system_signals_view_mut()
            .set_ambient_sound_effect(0x17);

        self.set_weather_vane_music_latch(0);
        self.set_weather_vane_countdown(0x0280);

        for i in (0..=11).rev() {
            self.weather_vane_debris_view_mut(i).initialize(
                u16::from(WEATHERVANE_DEBRIS_START_X[i]) | 0x0200,
                u16::from(WEATHERVANE_DEBRIS_START_Y[i]) | 0x0700,
                WEATHERVANE_DEBRIS_X_VELOCITY[i] as u8,
                0,
                WEATHERVANE_DEBRIS_Y_VELOCITY[i] as u8,
                WEATHERVANE_DEBRIS_CHAR[i],
                (i & 1) as u8,
            );
        }
    }

    fn ancilla_add_super_bomb_explosion(&mut self, a: u8, y: u8) -> i32 {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return -1;
        };
        let value = 0;
        self.ancilla_slot_view_mut(k).set_r(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        {
            let mut explosion = self.ancilla_slot_view_mut(k);
            explosion.set_step(0);
            explosion.set_work_byte_25(0);
            explosion.set_work_byte_3(BOMB_PHASE_TIMERS[1]);
            explosion.set_item_to_link(1);
        }
        let j = self.follower_state_view().data_index() as usize;
        let y = self.tagalong_slot_view(j).y();
        let x = self.tagalong_slot_view(j).x();
        self.ancilla_set_xy(k, x.wrapping_add(8), y.wrapping_add(16));
        k as i32
    }

    pub(super) fn ancilla_add_somaria_block(&mut self, ty: u8, y: u8) -> Option<usize> {
        let k = self.ancilla_add_add_ancilla_bank08(ty, y)?;
        for j in (0..=4).rev() {
            if j == k || self.ancilla_slot_view(j).ancilla_type() != 0x2c {
                continue;
            }
            if j == self
                .player_state_view()
                .ancilla_pickup_flag()
                .wrapping_sub(1) as usize
            {
                self.player_state_view_mut().clear_ancilla_pickup_flag();
            }
            self.ancilla_add_exploding_somaria_block(j);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            self.dungeon_state_view_mut()
                .clear_somaria_block_switch_counter();
            if self.player_state_view().speed_setting() == 0x12 {
                self.player_state_view_mut().clear_defense_flags();
                self.player_state_view_mut().set_speed_setting(0);
            }
            return Some(k);
        }

        self.ancilla_sfx3_near(0x2a);
        {
            let mut block = self.ancilla_slot_view_mut(k);
            block.set_step(0);
            block.set_y_velocity(0);
            block.set_x_velocity(0);
            block.set_item_to_link(0);
            block.set_aux_timer(0);
            block.set_work_byte_3(0);
            block.set_timer(18);
            block.set_z(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_h(value);
        let value = 12;
        self.ancilla_slot_view_mut(k).set_g(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_k(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_r(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_4(value);
        let value = 9;
        self.ancilla_slot_view_mut(k).set_s_player(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_t_player(value);
        let direction = self.player_state_view().facing() >> 1;
        self.ancilla_slot_view_mut(k).set_direction(direction);
        if self.ancilla_check_initial_tile_collision_class2(k) {
            self.ancilla_set_xy(
                k,
                self.player_state_view().x().wrapping_add(8),
                self.player_state_view().y().wrapping_add(16),
            );
        } else {
            const CANE_OF_SOMARIA_Y: [i8; 4] = [-8, 31, 17, 17];
            const CANE_OF_SOMARIA_X: [i8; 4] = [8, 8, -8, 23];
            let j = self.player_state_view().facing_index();
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(CANE_OF_SOMARIA_X[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(CANE_OF_SOMARIA_Y[j] as i16 as u16),
            );
            self.somaria_block_check_for_transit_tile(k);
        }
        Some(k)
    }

    pub(super) fn ancilla_get_y(&self, k: usize) -> u16 {
        self.ancilla_slot_view(k).y()
    }

    pub(super) fn ancilla_get_x(&self, k: usize) -> u16 {
        self.ancilla_slot_view(k).x()
    }

    fn ancilla_project_reflexive_speed_onto_sprite(
        &mut self,
        k: usize,
        x: u16,
        y: u16,
        vel: u8,
    ) -> ProjectSpeedRet {
        let old_x = self.player_state_view().x();
        let old_y = self.player_state_view().y();
        self.player_state_view_mut().set_x(x);
        self.player_state_view_mut().set_y(y);
        let pt = self.sprite_project_speed_towards_link(k, vel);
        self.player_state_view_mut().set_x(old_x);
        self.player_state_view_mut().set_y(old_y);
        pt
    }

    pub(super) fn ancilla_terminate_select_interactives(&mut self, mut y: u8) -> u8 {
        for i in (0..=5).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x3e {
                y = i as u8;
            } else if self.ancilla_slot_view(i).ancilla_type() == 0x2c {
                self.dungeon_state_view_mut()
                    .clear_somaria_block_switch_counter();
                if self.player_state_view().defense_flags() & 0x80 != 0 {
                    self.player_state_view_mut().clear_defense_flags();
                    self.player_state_view_mut().set_speed_setting(0);
                }
            }

            if sign8(self.player_state_view().state_bits()) {
                if i + 1 != self.player_state_view().ancilla_pickup_flag() as usize {
                    let value = 0;
                    self.ancilla_slot_view_mut(i).set_ancilla_type(value);
                }
            } else {
                if i + 1 == self.player_state_view().ancilla_pickup_flag() as usize {
                    self.player_state_view_mut().clear_ancilla_pickup_flag();
                }
                let value = 0;
                self.ancilla_slot_view_mut(i).set_ancilla_type(value);
            }
        }

        if self.player_state_view().position_mode_has(0x10) {
            self.player_state_view_mut().set_incapacitated_timer(0);
            self.player_state_view_mut().clear_position_mode();
        }
        self.player_state_view_mut().clear_flute_countdown();
        self.follower_state_view_mut().clear_event_flags();
        self.player_state_view_mut()
            .clear_ancilla_interactive_reset_flag();
        self.minigame_state_view_mut()
            .clear_flag_boomerang_in_place();
        self.minigame_state_view_mut()
            .clear_is_archer_or_shovel_game();
        self.player_state_view_mut()
            .clear_sprite_damage_disable_timer();
        self.player_state_view_mut()
            .clear_player_special_draw_flag();
        self.player_state_view_mut().clear_electrocute_on_touch();
        if self.player_state_view().handler_state() == 19 {
            self.player_state_view_mut().clear_handler_state();
            self.player_state_view_mut()
                .clear_button_mask_b_y_bits(0x40);
            self.player_state_view_mut().clear_direction_lock_bits(1);
            self.player_state_view_mut().clear_position_mode_bits(4);
            self.player_state_view_mut().clear_hookshot_interlock();
        }
        y
    }

    pub(super) fn ancilla_allocate_oam_from_region_a_or_d_or_f(
        &mut self,
        k: usize,
        size: u8,
    ) -> u16 {
        if self.oam_state_view().has_sprite_sorting() {
            if self.ancilla_slot_view(k).floor() != 0 {
                self.oam_allocate_from_region_f(size)
            } else {
                self.oam_allocate_from_region_d(size)
            }
        } else {
            self.oam_allocate_from_region_a(size)
        }
    }

    pub(super) fn ancilla_prep_oam_coord(&mut self, k: usize) -> (u16, u16) {
        const TAGALONG_LAYER_BITS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
        let floor = self.ancilla_slot_view(k).floor() as usize;
        self.oam_state_view_mut()
            .set_priority_word((TAGALONG_LAYER_BITS[floor] as u16) << 8);
        (
            self.ancilla_x(k).wrapping_sub(self.world_scroll().bg2_x()),
            self.ancilla_y(k).wrapping_sub(self.world_scroll().bg2_y()),
        )
    }

    pub(super) fn ancilla_move_x(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).move_x();
    }

    pub(super) fn ancilla_move_y(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).move_y();
    }

    pub(super) fn ancilla_move_z(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).move_z();
    }

    fn ancilla02_fire_rod_shot(&mut self, k: usize) {
        if self.ancilla_slot_view(k).step() == 0 {
            if self.frame_state().submodule == 0 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_l(value);
                self.ancilla_move_x(k);
                self.ancilla_move_y(k);
                let mut coll = self.ancilla_check_sprite_collision(k).is_some();
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
                let value = self.ancilla_slot_view(k).l();
                self.dungeon_torch_mut().set_attr(value);
                if self.dungeon_torch_state().torch_attr() & 0xf0 == 0xc0 {
                    self.dungeon_light_torch();
                } else {
                    let value = self.ancilla_slot_view(k).tile_attribute();
                    self.dungeon_torch_mut().set_attr(value);
                    if self.dungeon_torch_state().torch_attr() & 0xf0 == 0xc0 {
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
            let oam = self.oam_state_view().current_pointer_usize();
            if self.ancilla_slot_view(k).timer() == 0 {
                let old_type = self.ancilla_slot_view(k).ancilla_type();
                self.ancilla_slot_view_mut(k).clear();
                if old_type != 0x2f
                    && self.world_location_state().overworld_screen_index() == 64
                    && self.ancilla_slot_view(k).tile_attribute() == 0x43
                {
                    self.fire_rod_shot_become_skull_woods_fire(k);
                }
                return;
            }
            let j = self.ancilla_slot_view(k).timer() >> 3;
            if j != 0 {
                const FIRE_SHOT_DRAW_CHAR: [u8; 3] = [0xa2, 0xa0, 0x8e];
                self.ancilla_set_oam_plain(
                    oam,
                    info.x as u16,
                    info.y as u16,
                    FIRE_SHOT_DRAW_CHAR[j as usize - 1],
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
        if self.world_location_state().is_indoors()
            || self.world_location_state().overworld_screen_index() & 0x40 == 0
        {
            return;
        }

        self.ancilla_slot_view_mut(0).set_ancilla_type(0x34);
        for i in 1..=5 {
            self.ancilla_slot_view_mut(i).clear();
        }
        self.minigame_state_view_mut()
            .clear_flag_boomerang_in_place();
        self.ancilla_slot_view_mut(0)
            .set_num_sprites(ANCILLA_DRAW_SPRITE_COUNTS[0x34]);
        self.skull_woods_fire_view_mut(0).set_phase(253);
        self.skull_woods_fire_view_mut(1).set_phase(254);
        self.skull_woods_fire_view_mut(2).set_phase(255);
        self.skull_woods_fire_view_mut(3).set_phase(0);
        self.skull_woods_fire_scratch_view_mut()
            .clear_entrance_opening_started();
        for i in 0..4 {
            self.skull_woods_fire_view_mut(i).set_timer(5);
        }
        self.ancilla_slot_view_mut(0).set_aux_timer(5);
        self.skull_woods_fire_scratch_view_mut()
            .set_inner_position(0x0098, 0x0100);
        self.skull_woods_fire_scratch_view_mut()
            .set_outer_position(0x0098, 0x0100);
        self.set_special_entrance_trigger(2);
        self.set_subsubmodule(0);
        self.scratch_word_view_mut()
            .clear_module_transition_counter();
        let value = self.player_state_view().lower_level_state();
        self.ancilla_slot_view_mut(0).set_floor(value);
        let value = self.player_state_view().lower_level_mirror_state();
        self.ancilla_slot_view_mut(0).set_floor2(value);
        self.ancilla_slot_view_mut(0).set_item_to_link(0);
        self.ancilla_slot_view_mut(0).set_step(0);
    }

    fn ancilla0_b_ice_rod_shot(&mut self, k: usize) {
        if self.frame_state().submodule == 0 {
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
                if self.ancilla_check_sprite_collision(k).is_some()
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

    fn ancilla09_arrow(&mut self, k: usize) {
        const ARROW_Y: [i8; 4] = [-4, 2, 0, 0];
        const ARROW_X: [i8; 4] = [0, 0, -4, 4];

        if self.frame_state().submodule != 0 {
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
        if self.inventory_items().has_silver_arrows() && self.frame_state().frame_counter & 1 == 0 {
            self.ancilla_add_silver_arrow_sparkle(k);
        }
        let value = 255;
        self.ancilla_slot_view_mut(k).set_s_player(value);
        let j;
        if let Some(sprite) = self.ancilla_check_sprite_collision(k) {
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
                    self.system_signals_view_mut().set_sound_effect_2(0x2d);
                    let value = 0x80;
                    self.sprite_slot_view_mut(sprite).set_delay_aux2(value);
                    self.sprite_slot_view_mut(0).set_delay_aux4(128);
                    if self.archery_game_view().hit_counter() < 9 {
                        self.archery_game_view_mut().increment_hit_counter();
                    }
                    let value = self.archery_game_view().hit_counter();
                    self.sprite_slot_view_mut(sprite).set_b(value);
                    let value = self.sprite_slot_view(sprite).g().wrapping_add(1);
                    self.sprite_slot_view_mut(sprite).set_g(value);
                } else {
                    let value = 4;
                    self.sprite_slot_view_mut(sprite).set_delay_aux3(value);
                    self.archery_game_view_mut().clear_hit_counter();
                }
            } else {
                self.archery_game_view_mut().clear_hit_counter();
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
                        .wrapping_add(ARROW_X[dir] as i16 as u16),
                    self.ancilla_get_y(k)
                        .wrapping_add(ARROW_Y[dir] as i16 as u16),
                );
                self.archery_game_view_mut().clear_hit_counter();
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
                .wrapping_add(self.world_scroll().bg1_x_low())
                .wrapping_sub(self.world_scroll().bg2_x_low());
            self.ancilla_slot_view_mut(k).set_x_low(value);
            let value = self
                .ancilla_slot_view(k)
                .y_low()
                .wrapping_add(self.world_scroll().bg1_y_low())
                .wrapping_sub(self.world_scroll().bg2_y_low());
            self.ancilla_slot_view_mut(k).set_y_low(value);
        }
        self.arrow_draw(k);
    }

    fn ancilla_sword_beam(&mut self, k: usize) {
        const SWORD_BEAM_YVEL2: [i8; 4] = [0, 0, -6, -6];
        const SWORD_BEAM_XVEL2: [i8; 4] = [-8, -10, 0, 0];
        const SWORD_BEAM_CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];
        const SWORD_BEAM_CHAR2: [u8; 3] = [0xb7, 0x80, 0x83];

        let mut flags = 2;

        if self.frame_state().submodule == 0 {
            self.ancilla_set_xy(
                k,
                self.ether_orbit_view().swordbeam_temp_x(),
                self.ether_orbit_view().swordbeam_temp_y(),
            );
            self.ancilla_move_x(k);
            self.ancilla_move_y(k);
            let x = self.ancilla_get_x(k);
            let y = self.ancilla_get_y(k);
            self.ether_orbit_view_mut().set_swordbeam_temp(x, y);

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
                        .wrapping_add(SWORD_BEAM_XVEL2[j] as i16 as u16),
                    self.ancilla_get_y(k)
                        .wrapping_add(SWORD_BEAM_YVEL2[j] as i16 as u16),
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

        let oam_org = self.oam_state_view().current_pointer_usize();
        let mut oam = oam_org;
        let s = self.ancilla_slot_view(k).s_player();
        for i in (0..=3).rev() {
            let angle = if self.frame_state().submodule == 0 {
                self.effect_angle_scratch_view_mut().add_angle_mod64(i, s)
            } else {
                self.effect_angle_scratch_view().angle(i)
            };
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                angle,
                self.effect_angle_scratch_view().radial_radius(),
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                SWORD_BEAM_CHAR[i],
                flags | self.oam_state_view().priority_high(),
                0,
            );
            oam += 4;
        }

        if self.frame_state().submodule == 0 {
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
                self.effect_angle_scratch_view_mut()
                    .add_trailing_angle_mod64(s);
            }
        }

        let t = self.ancilla_slot_view(k).work_byte_1();
        if t != 3 {
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                self.effect_angle_scratch_view().trailing_angle(),
                self.effect_angle_scratch_view().radial_radius(),
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                SWORD_BEAM_CHAR2[t as usize],
                4 | self.oam_state_view().priority_high(),
                0,
            );
        }

        self.ancilla_sword_beam_check_offscreen(k, oam_org);
    }

    fn ancilla_sword_beam_check_offscreen(&mut self, k: usize, oam_org: usize) {
        for i in 0..4 {
            if self.oam_state_view().entry_y(oam_org + i * 4) != 0xf0 {
                return;
            }
        }
        self.ancilla_slot_view_mut(k).clear();
    }

    fn ancilla0_d_spin_attack_full_charge_spark(&mut self, k: usize) {
        const SWORD_FULL_CHARGE_SPARK_Y: [i8; 4] = [-8, 27, 12, 12];
        const SWORD_FULL_CHARGE_SPARK_X: [i8; 4] = [4, 4, -13, 20];
        const SWORD_FULL_CHARGE_SPARK_FLAGS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];

        let value = self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 4) as u8;

        self.ancilla_slot_view_mut(k).set_oam_index(value);

        if self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).clear();
            return;
        }

        let j = self.player_state_view().facing_index();

        let x = self
            .player_state_view()
            .x()
            .wrapping_add(SWORD_FULL_CHARGE_SPARK_X[j] as i16 as u16)
            .wrapping_sub(self.world_scroll().bg2_x());
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(SWORD_FULL_CHARGE_SPARK_Y[j] as i16 as u16)
            .wrapping_sub(self.world_scroll().bg2_y());

        let flags = SWORD_FULL_CHARGE_SPARK_FLAGS[self.ancilla_slot_view(k).floor() as usize];
        self.oam_state_view_mut()
            .set_priority_word((flags as u16) << 8);
        let oam = self.oam_state_view().current_pointer_usize();
        self.ancilla_set_oam(oam, x, y, 0xd7, flags | 2, 0);
    }

    fn ancilla20_blanket(&mut self, k: usize) {
        const BEDSPREAD_CHAR: [u8; 8] = [0x0a, 0x0a, 0x0a, 0x0a, 0x0c, 0x0c, 0x0a, 0x0a];
        const BEDSPREAD_FLAGS: [u8; 8] = [0, 0x60, 0xa0, 0xe0, 0, 0x60, 0xa0, 0xe0];

        let (mut x, mut y) = self.ancilla_prep_oam_coord(k);

        if self.player_state_view().opening_pose() == 0 {
            self.oam_allocate_from_region_b(0x10);
        } else {
            self.oam_allocate_from_region_a(0x10);
        }

        let mut oam = self.oam_state_view().current_pointer_usize();
        let mut j = if self.player_state_view().opening_pose() != 0 {
            4
        } else {
            0
        };
        for i in (0..=3).rev() {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                BEDSPREAD_CHAR[j],
                BEDSPREAD_FLAGS[j] | 0x0d | self.oam_state_view().priority_high(),
                2,
            );
            x = x.wrapping_add(16);
            if i == 2 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(8);
            }
            j += 1;
            oam += 4;
        }
    }

    fn ancilla21_snore(&mut self, k: usize) {
        const BEDSPREAD_DMA: [u8; 3] = [0x44, 0x43, 0x42];

        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = self.ancilla_slot_view(k).item_to_link();
            let mut snore = self.ancilla_slot_view_mut(k);
            if item_to_link != 2 {
                snore.advance_item_to_link();
            }
            snore.set_aux_timer(7);
        }

        let step = self.ancilla_slot_view(k).step();
        let x_velocity = self.ancilla_slot_view_mut(k).add_x_velocity(step);
        if abs8(x_velocity) >= 8 {
            self.ancilla_slot_view_mut(k)
                .set_step((-(step as i8)) as u8);
        }

        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
        if self.ancilla_y(k) <= self.player_state_view().y().wrapping_sub(24) {
            self.ancilla_slot_view_mut(k).clear();
        }

        let dma_staging_index = BEDSPREAD_DMA[self.ancilla_slot_view(k).item_to_link() as usize];
        self.player_state_view_mut()
            .set_link_dma_staging_index(dma_staging_index);
        let (x, y) = self.ancilla_prep_oam_coord(k);
        self.ancilla_set_oam(
            self.oam_state_view().current_pointer_usize(),
            x,
            y,
            9,
            0x24,
            0,
        );
    }

    fn ancilla23_link_poof(&mut self, k: usize) {
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = {
                let mut poof = self.ancilla_slot_view_mut(k);
                poof.set_aux_timer(7);
                poof.advance_item_to_link()
            };
            if item_to_link == 3 {
                self.ancilla_slot_view_mut(k).clear();
                self.player_state_view_mut().clear_transforming();
                self.player_state_view_mut().clear_direction_lock();
                if self.ancilla_slot_view(k).step() == 0 {
                    self.player_state_view_mut().clear_animation_step();
                    self.player_state_view_mut().set_visibility_status(0);
                    let bunny = if self.world_location_state().overworld_screen_index() & 0x40 != 0
                    {
                        1
                    } else {
                        0
                    };
                    self.player_state_view_mut().set_bunny_state(bunny);
                    if self.player_state_view().is_bunny() {
                        self.LoadGearPalettes_bunny();
                    } else {
                        self.LoadActualGearPalettes();
                    }
                }
                return;
            }
        }
        self.morph_poof_draw(k);
    }

    fn ancilla24_gravestone(&mut self, k: usize) {
        const ANCILLA_GRAVESTONE_CHAR: [u8; 4] = [0xc8, 0xc8, 0xd8, 0xd8];
        const ANCILLA_GRAVESTONE_FLAGS: [u8; 4] = [0, 0x40, 0, 0x40];

        let (mut x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.oam_allocate_from_region_b(16);
        let mut oam = self.oam_state_view().current_pointer_usize();
        for i in 0..4 {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ANCILLA_GRAVESTONE_CHAR[i],
                ANCILLA_GRAVESTONE_FLAGS[i] | 0x3d,
                2,
            );
            x = x.wrapping_add(16);
            if i == 1 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(8);
            }
            oam += 4;
        }
    }

    fn ancilla34_skull_woods_fire(&mut self, k: usize) {
        const SKULL_WOODS_FIRE_DRAW_Y: [i8; 4] = [0, 0, 0, -3];
        const SKULL_WOODS_FIRE_DRAW_CHAR: [u8; 4] = [0x8e, 0xa0, 0xa2, 0xa4];
        const SKULL_WOODS_FIRE_DRAW_EXT: [u8; 4] = [2, 2, 2, 0];
        const SKULL_WOODS_FIRE_DRAW2_X: [i8; 24] = [
            -13, -21, -10, -1, -1, -1, -16, -27, -4, -16, -6, -25, -16, -27, -4, -16, -6, -25, -13,
            -5, -27, -11, -22, -3,
        ];
        const SKULL_WOODS_FIRE_DRAW2_Y: [i8; 24] = [
            -31, -24, -22, -1, -1, -1, -37, -32, -32, -23, -16, -14, -37, -32, -32, -23, -16, -14,
            -35, -29, -28, -20, -13, -11,
        ];
        const SKULL_WOODS_FIRE_DRAW2_CHAR: [u8; 24] = [
            0x86, 0x86, 0x86, 0xff, 0xff, 0xff, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x8a, 0x8a,
            0x8a, 0x8a, 0x8a, 0x8a, 0x9b, 0x9b, 0x9b, 0x9b, 0x9b, 0x9b,
        ];
        const SKULL_WOODS_FIRE_DRAW2_FLAGS: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80, 0x40, 0x40, 0x80, 0x40, 0,
        ];
        const SKULL_WOODS_FIRE_DRAW2_EXT: [u8; 24] = [
            2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0,
        ];

        if self
            .skull_woods_fire_scratch_view()
            .has_started_entrance_opening()
            && self.ancilla_slot_view(k).item_to_link() != 4
            && sign8(self.ancilla_slot_view_mut(k).tick_aux_timer())
        {
            let mut fire = self.ancilla_slot_view_mut(k);
            fire.set_aux_timer(5);
            fire.advance_item_to_link();
        }

        let mut oam = self.oam_state_view().current_pointer_usize();
        for i in (0..=3).rev() {
            let timer = self.skull_woods_fire_view_mut(i).tick_timer();
            if sign8(timer) {
                self.skull_woods_fire_view_mut(i).set_timer(5);
                if self.skull_woods_fire_view(i).phase() != 128 {
                    let phase = self.skull_woods_fire_view_mut(i).advance_phase();
                    if phase == 0 || phase == 4 {
                        self.skull_woods_fire_view_mut(i).set_phase(0);
                        let inner_y = self.skull_woods_fire_scratch_view_mut().retreat_inner_y(8);
                        if inner_y < 200
                            && !self
                                .skull_woods_fire_scratch_view()
                                .has_started_entrance_opening()
                        {
                            self.skull_woods_fire_scratch_view_mut()
                                .set_entrance_opening_started();
                            let pan =
                                (0x98u16.wrapping_sub(self.world_scroll().bg2_x()) as u8) >> 5;
                            self.system_signals_view_mut()
                                .set_sound_effect_1(BOMBOS_PANNED_SFX_BITS[pan as usize] | 0x0c);
                        }
                        if inner_y < 168 {
                            self.skull_woods_fire_view_mut(i).set_phase(128);
                        }
                        let inner_x = self.skull_woods_fire_scratch_view().inner_x();
                        self.skull_woods_fire_view_mut(i)
                            .set_position(inner_x, inner_y);
                        if self.system_signals_view().sound_effect_1() == 0 {
                            let pan = (self
                                .skull_woods_fire_scratch_view()
                                .inner_x()
                                .wrapping_sub(self.world_scroll().bg2_x())
                                as u8)
                                >> 5;
                            self.system_signals_view_mut()
                                .set_sound_effect_1(BOMBOS_PANNED_SFX_BITS[pan as usize] | 0x2a);
                        }
                    }
                }
            }

            if !self.skull_woods_fire_view(i).is_finished() {
                let j = self.skull_woods_fire_view(i).phase() as usize;
                let x = self
                    .skull_woods_fire_view(i)
                    .x()
                    .wrapping_sub(self.world_scroll().bg2_x());
                let y = self
                    .skull_woods_fire_view(i)
                    .y()
                    .wrapping_sub(self.world_scroll().bg2_y())
                    .wrapping_add(SKULL_WOODS_FIRE_DRAW_Y[j] as i16 as u16);
                self.ancilla_set_oam(
                    oam,
                    x,
                    y,
                    SKULL_WOODS_FIRE_DRAW_CHAR[j],
                    0x32,
                    SKULL_WOODS_FIRE_DRAW_EXT[j],
                );
                oam += 4;
                if SKULL_WOODS_FIRE_DRAW_EXT[j] != 2 {
                    self.ancilla_set_oam(
                        oam,
                        x.wrapping_add(8),
                        y,
                        SKULL_WOODS_FIRE_DRAW_CHAR[j].wrapping_add(1),
                        0x32,
                        SKULL_WOODS_FIRE_DRAW_EXT[j],
                    );
                    oam += 4;
                }
            }
        }

        let mut i = 3i32;
        while self.skull_woods_fire_view(i as usize).is_finished() {
            i -= 1;
            if i < 0 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }

        let item_to_link = self.ancilla_slot_view(k).item_to_link();
        if !self
            .skull_woods_fire_scratch_view()
            .has_started_entrance_opening()
            || item_to_link == 4
        {
            return;
        }

        let mut j = item_to_link as usize * 6;
        for _ in 0..6 {
            if SKULL_WOODS_FIRE_DRAW2_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    168u16
                        .wrapping_sub(self.world_scroll().bg2_x())
                        .wrapping_add(SKULL_WOODS_FIRE_DRAW2_X[j] as i16 as u16),
                    200u16
                        .wrapping_sub(self.world_scroll().bg2_y())
                        .wrapping_add(SKULL_WOODS_FIRE_DRAW2_Y[j] as i16 as u16),
                    SKULL_WOODS_FIRE_DRAW2_CHAR[j],
                    SKULL_WOODS_FIRE_DRAW2_FLAGS[j] | 0x32,
                    SKULL_WOODS_FIRE_DRAW2_EXT[j],
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn morph_poof_draw(&mut self, k: usize) {
        const MORPH_POOF_OFFSET: [SignedOffset; 12] = signed_offsets![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 0, 8, 8, -4, -4, -4, 12, 12, -4, 12, 12,
        ];
        const MORPH_POOF_FLAGS: [u8; 12] = [
            0, 0xff, 0xff, 0xff, 0x40, 0, 0xc0, 0x80, 0, 0x40, 0x80, 0xc0,
        ];
        const MORPH_POOF_CHAR: [u8; 3] = [0x86, 0xa9, 0x9b];
        const MORPH_POOF_EXT: [u8; 3] = [2, 0, 0];
        if self.oam_state_view().has_sprite_sorting()
            && self.ancilla_slot_view(k).floor() != 0
            && (self.minigame_state_view().flag_boomerang_in_place() == 0
                || self.frame_state().frame_counter & 1 == 0)
        {
            self.oam_state_view_mut().set_current_pointer(0x08d0);
            self.oam_state_view_mut()
                .set_current_extended_pointer(0x0a20 + (0x0d0 >> 2));
        }
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let j = self.ancilla_slot_view(k).item_to_link() as usize;
        let ext = MORPH_POOF_EXT[j];
        let chr = MORPH_POOF_CHAR[j];
        for i in 0..4 {
            let offset = MORPH_POOF_OFFSET[j * 4 + i];
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(offset.x as i16 as u16),
                y.wrapping_add(offset.y as i16 as u16),
                chr,
                MORPH_POOF_FLAGS[j * 4 + i] | 4 | self.oam_state_view().priority_high(),
                ext,
            );
            if ext == 2 {
                break;
            }
            oam += 4;
        }
    }

    fn ancilla40_dwarf_poof(&mut self, k: usize) {
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = {
                let mut poof = self.ancilla_slot_view_mut(k);
                poof.set_aux_timer(7);
                poof.advance_item_to_link()
            };
            if item_to_link == 3 {
                self.ancilla_slot_view_mut(k).clear();
                self.follower_state_view_mut().set_appearance_none_flag(0);
                return;
            }
        }
        self.morph_poof_draw(k);
    }

    fn ancilla3_f_bush_poof(&mut self, k: usize) {
        const BUSH_POOF_DRAW_X: [i8; 16] = [0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, -2, 10, -2, 10];
        const BUSH_POOF_DRAW_Y: [i8; 16] = [0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, -2, -2, 10, 10];
        const BUSH_POOF_DRAW_CHAR: [u8; 16] = [
            0x86, 0x87, 0x96, 0x97, 0xa9, 0xa9, 0xa9, 0xa9, 0x8a, 0x8b, 0x9a, 0x9b, 0x9b, 0x9b,
            0x9b, 0x9b,
        ];
        const BUSH_POOF_DRAW_FLAGS: [u8; 16] = [
            0, 0, 0, 0, 0, 0x40, 0x80, 0xc0, 0, 0, 0, 0, 0xc0, 0x80, 0x40, 0,
        ];

        if self.ancilla_slot_view(k).timer() == 0 {
            let item_to_link = {
                let mut poof = self.ancilla_slot_view_mut(k);
                poof.set_timer(7);
                poof.advance_item_to_link()
            };
            if item_to_link == 4 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }
        self.oam_get_buffer_position(0x10, 4);
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();

        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 4;
        for _ in 0..4 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(BUSH_POOF_DRAW_X[j] as i16 as u16),
                y.wrapping_add(BUSH_POOF_DRAW_Y[j] as i16 as u16),
                BUSH_POOF_DRAW_CHAR[j],
                BUSH_POOF_DRAW_FLAGS[j] | 4 | self.oam_state_view().priority_high(),
                0,
            );
            j += 1;
            oam += 4;
        }
    }

    fn ancilla26_sword_swing_sparkle(&mut self, k: usize) {
        const SWORD_SWING_SPARKLE_X: [i8; 48] = [
            5, 10, -1, 5, 10, -4, 5, 10, -4, -4, -1, -1, 0, 5, -1, 0, 5, 14, 0, 5, 14, 14, -1, -1,
            -23, -27, -1, -23, -27, -22, -23, -27, -22, -22, -1, -1, 32, 35, -1, 32, 35, 30, 32,
            35, 30, 30, -1, -1,
        ];
        const SWORD_SWING_SPARKLE_Y: [i8; 48] = [
            -22, -18, -1, -22, -18, -17, -22, -18, -17, -17, -1, -1, 35, 40, -1, 35, 40, 37, 35,
            40, 37, 37, -1, -1, 2, 7, -1, 2, 7, 19, 2, 7, 19, 19, -1, -1, 2, 7, -1, 2, 7, 19, 2, 7,
            19, 19, -1, -1,
        ];
        const SWORD_SWING_SPARKLE_CHAR: [u8; 48] = [
            0xb7, 0xb7, 0xff, 0x80, 0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7,
            0xff, 0x80, 0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7, 0xff, 0x80,
            0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7, 0xff, 0x80, 0x80, 0xb7,
            0x83, 0x83, 0x80, 0x83, 0xff, 0xff,
        ];
        const SWORD_SWING_SPARKLE_FLAGS: [u8; 48] = [
            0, 0, 0xff, 0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff, 0, 0, 0xff, 0, 0, 0, 0x80, 0x80,
            0, 0x80, 0xff, 0xff, 0, 0, 0xff, 0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff, 0, 0, 0xff,
            0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff,
        ];
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = {
                let mut sparkle = self.ancilla_slot_view_mut(k);
                sparkle.set_aux_timer(0);
                sparkle.advance_item_to_link()
            };
            if item_to_link == 4 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }
        self.ancilla_set_xy(
            k,
            self.player_state_view().x(),
            self.player_state_view().y(),
        );

        let (x, y) = self.ancilla_prep_oam_coord(k);

        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 3
            + self.ancilla_slot_view(k).direction() as usize * 12;

        let mut oam = self.oam_state_view().current_pointer_usize();
        for _ in (0..=2).rev() {
            let chr = SWORD_SWING_SPARKLE_CHAR[j];
            if chr != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(SWORD_SWING_SPARKLE_X[j] as i16 as u16),
                    y.wrapping_add(SWORD_SWING_SPARKLE_Y[j] as i16 as u16),
                    chr,
                    SWORD_SWING_SPARKLE_FLAGS[j] | 0x4 | self.oam_state_view().priority_high(),
                    0,
                );
            }
            j += 1;
            oam += 4;
        }
    }

    fn ancilla2_d_somaria_block_fizz(&mut self, k: usize) {
        const SOMARIA_BLOCK_FIZZLE_X: [i8; 6] = [-4, -1, -8, 0, -6, -2];
        const SOMARIA_BLOCK_FIZZLE_Y: [i8; 6] = [-4, -1, -4, -4, -4, -4];
        const SOMARIA_BLOCK_FIZZLE_CHAR: [u8; 6] = [0x92, 0xff, 0xf9, 0xf9, 0xf9, 0xf9];
        const SOMARIA_BLOCK_FIZZLE_FLAGS: [u8; 6] = [6, 0xff, 0x86, 0xc6, 0x86, 0xc6];

        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if (aux_timer as i8) < 0 {
            let item_to_link = {
                let mut fizzle = self.ancilla_slot_view_mut(k);
                fizzle.set_aux_timer(3);
                fizzle.advance_item_to_link()
            };
            if item_to_link == 3 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let mut z = self.ancilla_slot_view(k).z();
        if z == 0xff {
            z = 0;
        }
        let y = y.wrapping_sub(z as i8 as i16 as u16);
        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 2;
        for _ in 0..2 {
            if SOMARIA_BLOCK_FIZZLE_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(SOMARIA_BLOCK_FIZZLE_X[j] as i16 as u16),
                    y.wrapping_add(SOMARIA_BLOCK_FIZZLE_Y[j] as i16 as u16),
                    SOMARIA_BLOCK_FIZZLE_CHAR[j],
                    SOMARIA_BLOCK_FIZZLE_FLAGS[j] & !0x30 | self.oam_state_view().priority_high(),
                    0,
                );
            }
            j += 1;
            oam += 4;
        }
    }

    fn ancilla39_somaria_platform_poof(&mut self, k: usize) {
        const SOMARIAN_PLATFORM_POOF_DIRECTION_BY_OPEN_SIDE: [u8; 4] = [1, 0, 3, 2];
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if (aux_timer as i8) >= 0 {
            return;
        }
        self.ancilla_slot_view_mut(k).clear();
        let x = self.ancilla_get_x(k) & !7 | 4;
        let y = self.ancilla_get_y(k) & !7 | 4;
        let floor = self.ancilla_slot_view(k).floor();
        if let Some(j) = self.sprite_spawn_dynamically_for_ancilla(k, 0xed) {
            self.player_state_view_mut().clear_somaria_platform_state();
            self.sprite_set_x(j, x);
            self.sprite_set_y(j, y);

            let pos = (((x & 0x01f8) >> 3) + ((y & 0x01f8) << 3)) as usize
                + if floor >= 1 { 0x1000 } else { 0 };

            let mut t = 0usize;
            if self
                .dungeon_bg2_attributes()
                .bg2_attr(pos.wrapping_sub(0x40))
                & 0xf0
                != 0xb0
            {
                t += 1;
                if self.dungeon_bg2_attributes().bg2_attr(pos + 0x40) & 0xf0 != 0xb0 {
                    t += 1;
                    if self.dungeon_bg2_attributes().bg2_attr(pos.wrapping_sub(1)) & 0xf0 != 0xb0 {
                        t += 1;
                    }
                }
            }
            let value = SOMARIAN_PLATFORM_POOF_DIRECTION_BY_OPEN_SIDE[t];
            self.sprite_slot_view_mut(j).set_direction(value);
            self.sprite_slot_view_mut(j).set_floor(0);
        } else {
            self.ancilla_draw_somaria_block(k);
        }
    }

    fn ancilla3_a_big_bomb_explosion(&mut self, k: usize) {
        const SUPER_BOMB_EXPLODE_X: [i8; 9] = [0, -16, 0, 16, -24, 24, -16, 0, 16];
        const SUPER_BOMB_EXPLODE_Y: [i8; 9] = [0, -16, -24, -16, 0, 0, 16, 24, 16];
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        if self.frame_state().submodule == 0 {
            if self.ancilla_slot_view_mut(k).tick_work_byte_3() == 0 {
                let bomb_phase = self.ancilla_slot_view_mut(k).advance_item_to_link();
                if bomb_phase == 2 {
                    self.ancilla_sfx2_pan(k, 0x0c);
                }
                if bomb_phase == 11 {
                    self.ancilla_slot_view_mut(k).clear();
                    return;
                }
                self.ancilla_slot_view_mut(k)
                    .set_work_byte_3(BOMB_PHASE_TIMERS[bomb_phase as usize]);
            }
        }

        self.oam_state_view_mut().set_priority_word(0x3000);
        let bomb_phase = self.ancilla_slot_view(k).item_to_link() as usize;
        let numframes = BOMB_DRAW_FRAME_COUNTS[bomb_phase] as usize;
        let j = BOMB_DRAW_FRAME_STARTS[bomb_phase] as usize * 6;
        self.ancilla_slot_view_mut(k).set_step((j * 2) as u8);

        let mut yy = 0usize;
        for i in (0..=8).rev() {
            let x = self
                .ancilla_get_x(k)
                .wrapping_add(SUPER_BOMB_EXPLODE_X[i] as i16 as u16)
                .wrapping_sub(self.world_scroll().bg2_x());
            let y = self
                .ancilla_get_y(k)
                .wrapping_add(SUPER_BOMB_EXPLODE_Y[i] as i16 as u16)
                .wrapping_sub(self.world_scroll().bg2_y());
            if x < 256 && y < 256 {
                self.ancilla_allocate_oam_from_region_a_or_d_or_f((j * 2) as usize, 0x18);
                let base_oam = self.oam_state_view().current_pointer_usize();
                let oam = base_oam + yy;
                let next_oam = self.ancilla_draw_explosion(oam, j, 0, numframes, 0x32, x, y);
                yy += next_oam - oam;
            }
        }

        if self.ancilla_slot_view(k).item_to_link() == 3
            && self.ancilla_slot_view(k).work_byte_3() == 1
        {
            let old = if self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES) {
                self.follower_state_view().indicator()
            } else {
                0
            };
            self.follower_state_view_mut().set_indicator(13);
            self.bomb_check_for_destructibles(self.ancilla_get_x(k), self.ancilla_get_y(k), 0);
            self.follower_state_view_mut().set_indicator(old);
        }
    }

    fn ancilla3_b_sword_up_sparkle(&mut self, k: usize) {
        const ANCILLA_VICTORY_SPARKLE_X: [i8; 16] =
            [16, 0, 0, 0, 8, 16, 8, 16, 9, 15, 0, 0, 12, 0, 0, 0];
        const ANCILLA_VICTORY_SPARKLE_Y: [i8; 16] =
            [-7, 0, 0, 0, -11, -11, -3, -3, -7, -7, 0, 0, -7, 0, 0, 0];
        const ANCILLA_VICTORY_SPARKLE_CHAR: [u8; 16] = [
            0x92, 0xff, 0xff, 0xff, 0x93, 0x93, 0x93, 0x93, 0xf9, 0xf9, 0xff, 0xff, 0x80, 0xff,
            0xff, 0xff,
        ];
        const ANCILLA_VICTORY_SPARKLE_FLAGS: [u8; 16] = [
            0, 0xff, 0xff, 0xff, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0xff, 0xff, 0, 0xff, 0xff, 0xff,
        ];

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
        let mut oam = self.oam_state_view().current_pointer_usize();
        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 4;
        for _ in 0..4 {
            if ANCILLA_VICTORY_SPARKLE_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    self.player_state_view()
                        .x()
                        .wrapping_add(ANCILLA_VICTORY_SPARKLE_X[j] as i16 as u16)
                        .wrapping_sub(self.world_scroll().bg2_x()),
                    self.player_state_view()
                        .y()
                        .wrapping_add(ANCILLA_VICTORY_SPARKLE_Y[j] as i16 as u16)
                        .wrapping_sub(self.world_scroll().bg2_y()),
                    ANCILLA_VICTORY_SPARKLE_CHAR[j],
                    ANCILLA_VICTORY_SPARKLE_FLAGS[j] | 4 | self.oam_state_view().priority_high(),
                    0,
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn spin_spark_draw(&mut self, k: usize, offs: i32) {
        const INITIAL_SPIN_SPARK_CHAR: [u8; 32] = [
            0x92, 0xff, 0xff, 0xff, 0x8c, 0x8c, 0x8c, 0x8c, 0xd6, 0xd6, 0xd6, 0xd6, 0x93, 0x93,
            0x93, 0x93, 0xd6, 0xd6, 0xd6, 0xd6, 0xd7, 0xff, 0xff, 0xff, 0x80, 0xff, 0xff, 0xff,
            0x22, 0xff, 0xff, 0xff,
        ];
        const INITIAL_SPIN_SPARK_FLAGS: [u8; 29] = [
            0x22, 0xff, 0xff, 0xff, 0x22, 0x62, 0xa2, 0xe2, 0x24, 0x64, 0xa4, 0xe4, 0x22, 0x62,
            0xa2, 0xe2, 0x22, 0x62, 0xa2, 0xe2, 0x22, 0xff, 0xff, 0xff, 0x22, 0xff, 0xff, 0xff,
            0xfc,
        ];
        const INITIAL_SPIN_SPARK_Y: [i8; 29] = [
            -4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -4, 0, 0, 0, -4,
            0, 0, 0, -4,
        ];
        const INITIAL_SPIN_SPARK_X: [i16; 29] = [
            -4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -4, 0, 0, 0, -4,
            0, 0, 0, 0x11a5,
        ];

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let mut t = (i32::from(self.ancilla_slot_view(k).item_to_link()) + offs) * 4;
        assert!(t < 32);
        for _ in 0..4 {
            let idx = t as usize;
            if INITIAL_SPIN_SPARK_CHAR[idx] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(INITIAL_SPIN_SPARK_X[idx] as u16),
                    y.wrapping_add(INITIAL_SPIN_SPARK_Y[idx] as i16 as u16),
                    INITIAL_SPIN_SPARK_CHAR[idx],
                    INITIAL_SPIN_SPARK_FLAGS[idx] & !0x30 | self.oam_state_view().priority_high(),
                    0,
                );
                oam += 4;
            }
            t += 1;
        }
    }

    fn ancilla2_a_spin_attack_sparkle_a(&mut self, k: usize) {
        const INITIAL_SPIN_SPARK_TIMER: [u8; 6] = [4, 2, 3, 3, 2, 1];

        if self.frame_state().submodule == 0 {
            let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(aux_timer) {
                self.ancilla_slot_view_mut(k).set_aux_timer(0);
                if self.ancilla_slot_view(k).timer() == 0 {
                    let j = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
                    {
                        let mut sparkle = self.ancilla_slot_view_mut(k);
                        sparkle.set_item_to_link(j);
                        sparkle.set_timer(INITIAL_SPIN_SPARK_TIMER[j as usize]);
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
        const TRANSMUTE_SPIN_SPARK_ARR: [u8; 16] = [
            0x21, 0x20, 0x1f, 0x1e, 3, 2, 1, 0, 0x12, 0x11, 0x10, 0x0f, 0x31, 0x30, 0x2f, 0x2e,
        ];
        const TRANSMUTE_SPIN_SPARK_X: [i8; 4] = [-3, 21, 25, -8];
        const TRANSMUTE_SPIN_SPARK_Y: [i8; 4] = [28, -2, 24, 6];

        let mut j = self.player_state_view().facing() as usize * 2;
        self.effect_angle_scratch_view_mut()
            .set_angles4(&TRANSMUTE_SPIN_SPARK_ARR, j);
        self.effect_angle_scratch_view_mut()
            .set_trailing_angle(TRANSMUTE_SPIN_SPARK_ARR[j + 3]);
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
        self.effect_angle_scratch_view_mut().set_radial_radius(20);

        let swordbeam_temp_x = self.player_state_view().x().wrapping_add(8);
        let swordbeam_temp_y = self.player_state_view().y().wrapping_add(12);
        self.ether_orbit_view_mut()
            .set_swordbeam_temp(swordbeam_temp_x, swordbeam_temp_y);

        j = self.player_state_view().facing_index();
        self.ancilla_set_xy(
            k,
            self.player_state_view()
                .x()
                .wrapping_add(TRANSMUTE_SPIN_SPARK_X[j] as i16 as u16),
            self.player_state_view()
                .y()
                .wrapping_add(TRANSMUTE_SPIN_SPARK_Y[j] as i16 as u16),
        );
        self.ancilla2_b_spin_attack_sparkle_b(k);
    }

    fn ancilla2_b_spin_attack_sparkle_b(&mut self, k: usize) {
        const SPIN_SPARK_CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];

        if self.ancilla_slot_view(k).l() != 0 {
            self.spin_attack_sparkle_b_closer(k);
            return;
        }
        let mut flags = 2;
        if self.frame_state().submodule == 0 {
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

        let oam_org = self.oam_state_view().current_pointer_usize();
        let mut oam = oam_org;
        let mut i = self.ancilla_slot_view(k).step() as usize;
        loop {
            let angle = if self.frame_state().submodule == 0 {
                self.effect_angle_scratch_view_mut().add_angle_mod64(i, 4)
            } else {
                self.effect_angle_scratch_view().angle(i)
            };
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                angle,
                self.effect_angle_scratch_view().radial_radius(),
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                SPIN_SPARK_CHAR[i],
                flags | self.oam_state_view().priority_high(),
                0,
            );
            oam += 4;
            if i == 0 {
                break;
            }
            i -= 1;
        }

        if self.frame_state().submodule == 0 {
            let work_byte_3 = self.ancilla_slot_view_mut(k).tick_work_byte_3();
            if !sign8(work_byte_3) {
                if self.ancilla_slot_view(k).item_to_link() == 7 {
                    let value = 1;
                    self.oam_state_view_mut()
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
                self.effect_angle_scratch_view_mut()
                    .add_trailing_angle_mod64(9);
            }
        }

        let t = self.ancilla_slot_view(k).work_byte_1();
        if t != 3 {
            const SPIN_SPARK_CHAR2: [u8; 3] = [0xb7, 0x80, 0x83];
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                self.effect_angle_scratch_view().trailing_angle(),
                self.effect_angle_scratch_view().radial_radius(),
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                SPIN_SPARK_CHAR2[t as usize],
                4 | self.oam_state_view().priority_high(),
                0,
            );
        }
        if self.ancilla_slot_view(k).item_to_link() == 7 {
            let value = 1;
            self.oam_state_view_mut()
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

    fn ancilla35_master_sword_receipt(&mut self, k: usize) {
        const SWORD_CEREMONY_X: [i8; 8] = [-1, 8, -1, 8, 0, 7, 0, 7];
        const SWORD_CEREMONY_Y: [i8; 8] = [1, 1, 9, 9, 1, 1, 9, 9];
        const SWORD_CEREMONY_CHAR: [u8; 8] = [0x86, 0x86, 0x96, 0x96, 0x87, 0x87, 0x97, 0x97];
        const SWORD_CEREMONY_FLAGS: [u8; 8] = [1, 0x41, 1, 0x41, 1, 0x41, 1, 0x41];

        if self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).clear();
            return;
        }
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = if self.ancilla_slot_view(k).item_to_link() == 2 {
                0
            } else {
                self.ancilla_slot_view(k).item_to_link().wrapping_add(1)
            };
            self.ancilla_slot_view_mut(k).set_item_to_link(item_to_link);
        }

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let item = self.ancilla_slot_view(k).item_to_link();
        if item == 0 {
            return;
        }

        let mut j = item.wrapping_sub(1) as usize * 4;
        for _ in 0..4 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(SWORD_CEREMONY_X[j] as i16 as u16),
                y.wrapping_add(SWORD_CEREMONY_Y[j] as i16 as u16),
                SWORD_CEREMONY_CHAR[j],
                SWORD_CEREMONY_FLAGS[j] & !0x30 | 4 | self.oam_state_view().priority_high(),
                0,
            );
            j += 1;
            oam += 4;
        }
    }

    fn ancilla36_flute(&mut self, k: usize) {
        const FLUTE_VELS: [u8; 4] = [0x18, 0x10, 0x0a, 0];

        if self.frame_state().submodule == 0 {
            if self.ancilla_slot_view(k).step() != 3 {
                self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
                self.ancilla_move_x(k);
                self.ancilla_move_z(k);
                let z = self.ancilla_slot_view(k).z();
                if sign8(z) || z >= 0xf0 {
                    let step = self.ancilla_slot_view_mut(k).advance_step();
                    let z_velocity = FLUTE_VELS[step as usize];
                    let mut flute = self.ancilla_slot_view_mut(k);
                    flute.set_z_velocity(z_velocity);
                    flute.set_z(0);
                }
            } else if self.ancilla_check_link_collision(k, 2)
                && !self.player_state_view().has_hookshot_interlock()
                && !self.player_state_view().has_auxiliary_state()
            {
                self.ancilla_slot_view_mut(k).clear();
                self.player_state_view_mut().set_item_receipt_method(0);
                self.link_receive_item(0x14, 0);
                return;
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let oam = self.oam_state_view().current_pointer_usize();
        self.ancilla_set_oam(
            oam,
            x,
            y.wrapping_sub(self.ancilla_slot_view(k).z() as i8 as i16 as u16),
            0x24,
            self.oam_state_view().priority_high() | 4,
            2,
        );
        if self.oam_state_view().entry_y(oam) == 0xf0 {
            self.ancilla_slot_view_mut(k).clear();
        }
    }

    fn ancilla37_weathervane_explosion(&mut self, k: usize) {
        if self.tick_weather_vane_countdown() != 0 {
            return;
        }
        self.set_weather_vane_countdown(1);
        if self.weather_vane_music_latch() == 0 {
            self.set_weather_vane_music_latch(1);
            self.system_signals_view_mut().set_music_control(0xf3);
        }
        if self.ancilla_slot_view_mut(k).tick_g() != 0 {
            return;
        }
        self.ancilla_slot_view_mut(k).set_g(1);
        if self.ancilla_slot_view(k).work_byte_3() == 0 {
            self.ancilla_slot_view_mut(k).advance_work_byte_3();
            self.ancilla_sfx2_near(0x0c);
        }
        if self.ancilla_slot_view(k).step() == 0 {
            let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(aux_timer) {
                self.ancilla_slot_view_mut(k).set_step(1);
                self.overworld_alter_weathervane();
                self.ancilla_add_cutscene_duck(0x38, 0);
            }
        }
        self.set_weather_vane_source_slot(k as u8);
        self.reset_weather_vane_oam_offset();
        for i in (0..=11).rev() {
            if self.weather_vane_debris_view(i).is_finished() {
                continue;
            }
            let draw_state = self.weather_vane_debris_view_mut(i).tick_animation();
            let z_velocity = self.weather_vane_debris_view_mut(i).tick_z_velocity();

            let mut debris = self.weather_vane_debris_view(i).snapshot();
            debris.z_velocity = z_velocity;
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_item_to_link(draw_state);
                ancilla.set_y(debris.y);
                ancilla.set_x(debris.x);
                ancilla.set_z(debris.z);
                ancilla.set_y_velocity(debris.y_velocity);
                ancilla.set_x_velocity(debris.x_velocity);
                ancilla.set_z_velocity(debris.z_velocity);
            }

            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            self.ancilla_move_z(k);

            let landed_z = self.ancilla_slot_view(k).z();
            self.ancilla_draw_weathervane_explosion_wood_debris(k);
            self.weather_vane_debris_view_mut(i)
                .mark_finished_if_landed(landed_z);
            let ancilla = self.ancilla_slot_view(k);
            let debris_y = ancilla.y();
            let debris_x = ancilla.x();
            let debris_z = ancilla.z();
            self.weather_vane_debris_view_mut(i)
                .save_position(debris_x, debris_y, debris_z);
        }
        for i in (0..=11).rev() {
            if !self.weather_vane_debris_view(i).is_finished() {
                return;
            }
        }
        self.ancilla_slot_view_mut(k).clear();
    }

    fn ancilla2_c_somaria_block(&mut self, k: usize) {
        const SOMARIAN_BLOCK_COLL_X: [i8; 12] = [0, 0, -8, 8, 0, 0, 0, 0, 8, -8, -8, 8];
        const SOMARIAN_BLOCK_COLL_Y: [i8; 12] = [-8, 8, 0, 0, 0, 0, 0, 0, -8, 8, -8, 8];

        if !sign8(self.ancilla_slot_view_mut(k).tick_g()) {
            return;
        }
        self.ancilla_slot_view_mut(k).set_g(0);

        if self.ancilla_slot_view(k).h() == 0 {
            if matches!(self.frame_state().submodule, 0 | 8 | 16) {
                self.ancilla_handle_lift_logic(k);
            } else if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize
                && self.ancilla_slot_view(k).k() != 0
            {
                if self.ancilla_slot_view(k).k() != 3 {
                    self.ancilla_latch_link_coordinates(k, 3);
                    self.ancilla_latch_altitude_above_link(k);
                    self.ancilla_slot_view_mut(k).set_k(3);
                }
                self.ancilla_latch_carried_position(k);
            }
            if self.world_location_state().is_indoors() {
                if self.ancilla_slot_view(k).k() == 0
                    && !self.player_state_view().is_lifting_or_carrying()
                    && (self.ancilla_slot_view(k).z() == 0 || self.ancilla_slot_view(k).z() == 0xff)
                {
                    if self.player_state_view().somaria_block_bg_check_flag() != 0 {
                        let mut j = (self.frame_state().frame_counter & 3) as usize;
                        loop {
                            let bak = self.ancilla_slot_view(k).object_priority();
                            let x = self
                                .ancilla_get_x(k)
                                .wrapping_add(SOMARIAN_BLOCK_COLL_X[j] as i16 as u16);
                            let y = self
                                .ancilla_get_y(k)
                                .wrapping_add(SOMARIAN_BLOCK_COLL_Y[j] as i16 as u16);
                            self.ancilla_check_tile_collision_targeted(k, x, y);
                            self.ancilla_slot_view_mut(k).set_object_priority(bak);
                            if matches!(self.ancilla_slot_view(k).tile_attribute(), 0xb6 | 0xbc) {
                                self.ancilla_set_xy(k, x, y);
                                self.ancilla_add_somaria_platform_poof(k);
                                if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize
                                {
                                    self.player_state_view_mut().clear_ancilla_pickup_flag();
                                }
                                return;
                            }
                            j += 4;
                            if j >= 12 {
                                break;
                            }
                        }
                    } else if !self.somaria_block_check_for_switch(k)
                        && (self.ancilla_slot_view(k).z() == 0
                            || self.ancilla_slot_view(k).z() == 0xff)
                    {
                        self.dungeon_state_view_mut()
                            .increment_somaria_block_switch_counter();
                    }
                } else if self.player_state_view().ancilla_pickup_flag() == k as u8 + 1 {
                    self.dungeon_state_view_mut()
                        .clear_somaria_block_switch_counter();
                }
            }
        } else if self.world_location_state().is_indoors()
            && self.player_state_view().ancilla_pickup_flag() == k as u8 + 1
        {
            self.dungeon_state_view_mut()
                .clear_somaria_block_switch_counter();
        }

        let mut old_y = self.ancilla_latch_y_coord_to_z(k);
        let s1a = self.ancilla_slot_view(k).direction();
        let mut s1b = self.ancilla_slot_view(k).object_priority();
        self.ancilla_slot_view_mut(k).set_object_priority(0);
        let mut flag = self.ancilla_check_tile_collision_class2(k);

        if self.world_location_state().is_indoors()
            && self.ancilla_slot_view(k).l() != 0
            && self.ancilla_slot_view(k).tile_attribute() == 0x1c
        {
            let value = 1;
            self.ancilla_slot_view_mut(k).set_t_player(value);
        }

        loop {
            if flag
                && (!self.player_state_view().is_lifting_or_carrying()
                    || self.player_state_view().has_picking_throw_state())
            {
                if s1b == 0
                    && self.ancilla_slot_view(k).work_byte_4() == 0
                    && self.ancilla_slot_view(k).z() != 0
                {
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
            } else if !self.player_state_view().is_lifting_or_carrying()
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
                    if self.dungeon_state_view().header_collision() != 3 {
                        if self.ancilla_slot_view(k).floor() == 0
                            && self.ancilla_slot_view(k).z() != 0
                            && self.ancilla_slot_view(k).z() != 0xff
                        {
                            self.ancilla_slot_view_mut(k).set_floor(1);
                        }
                    } else {
                        old_y = self
                            .ancilla_get_y(k)
                            .wrapping_add(self.dungeon_moving_floor().floor_y_velocity());
                        self.ancilla_set_x(
                            k,
                            self.ancilla_get_x(k)
                                .wrapping_add(self.dungeon_moving_floor().floor_x_velocity()),
                        );
                    }
                } else if a == 0x20 || (a & 0xf0) == 0xb0 && a != 0xb6 && a != 0xbc {
                    if !self.player_state_view().is_lifting_or_carrying() {
                        if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize {
                            self.player_state_view_mut().clear_ancilla_pickup_flag();
                        }
                        if self.ancilla_slot_view(k).timer() == 0 {
                            if self.player_state_view().speed_setting() == 18 {
                                self.player_state_view_mut().set_speed_setting(0);
                                self.player_state_view_mut().clear_defense_flags();
                            }
                            self.ancilla_slot_view_mut(k).clear();
                            return;
                        }
                    }
                } else if a == 8 {
                    if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize {
                        self.player_state_view_mut().clear_ancilla_pickup_flag();
                    }
                    if self.ancilla_slot_view(k).timer() == 0 {
                        self.ancilla_set_y(k, self.ancilla_get_y(k).wrapping_sub(24));
                        self.ancilla_transmute_to_splash(k);
                        return;
                    }
                } else if matches!(a, 0x68 | 0x69 | 0x6a | 0x6b) {
                    self.ancilla_apply_conveyor(k);
                    old_y = self.ancilla_get_y(k);
                } else {
                    let timer =
                        if (self.ancilla_slot_view(k).l() | self.ancilla_slot_view(k).h()) != 0 {
                            0
                        } else {
                            2
                        };
                    self.ancilla_slot_view_mut(k).set_timer(timer);
                }
            }
            break;
        }

        s1b |= self.ancilla_slot_view(k).object_priority();

        if !self.player_state_view().is_lifting_or_carrying() {
            if self.ancilla_slot_view_mut(k).tick_s_player() == 0 {
                self.ancilla_slot_view_mut(k).set_s_player(1);
                self.ancilla_slot_view_mut(k).set_object_priority(0);
                if self.ancilla_check_basic_sprite_collision(k).is_some() {
                    self.ancilla_slot_view_mut(k).set_s_player(7);
                    if self.ancilla_slot_view_mut(k).advance_step() == 5 {
                        self.somaria_block_fizzle_away(k);
                        return;
                    }
                }
            }
        }
        self.ancilla_set_y(k, old_y);
        self.ancilla_slot_view_mut(k).set_direction(s1a);
        self.ancilla_slot_view_mut(k).set_object_priority(s1b);

        self.ancilla_draw_somaria_block(k);
    }

    fn quake_spell_shake_screen(&mut self, _k: usize) {
        let shake_y = self.quake_spell_scratch_view_mut().invert_screen_shake_y();
        self.world_scroll_mut().set_bg1_y_offset(shake_y);
        self.player_state_view_mut()
            .add_y_velocity_delta(shake_y as u8);
    }

    fn ancilla1_c_quake_spell(&mut self, k: usize) {
        if self.frame_state().submodule != 0 {
            if self.quake_bolt_view(4).phase() != QUAKE_BOLT_TARGET_PHASES[4] {
                self.ancilla_draw_quake_initial_bolts(k);
            }
            return;
        }
        if self.ancilla_slot_view(k).step() != 2 {
            self.quake_spell_shake_screen(k);
            self.quake_spell_control_bolts(k);
            self.quake_spell_spread_bolts(k);
            return;
        }
        self.medallion_check_sprite_damage(k);
        self.prepare_apply_rumble_to_sprites();
        self.ancilla_slot_view_mut(k).clear();
        self.player_state_view_mut().clear_handler_state();
        self.set_chr_halfslot_request(1);
        self.player_state_view_mut().clear_spin_attack_sound_latch();
        self.player_state_view_mut().clear_state_for_spin_attack();
        self.player_state_view_mut()
            .clear_spin_animation_step_counter();
        self.player_state_view_mut().clear_direction_lock();
        self.player_state_view_mut().set_spin_attack_delay_timer(0);
        self.clear_modal_pause_flag();
        self.world_scroll_mut().set_bg1_x_offset(0);
        self.world_scroll_mut().set_bg1_y_offset(0);
        if self.world_location_state().overworld_screen_index() == 0x47
            && self.overworld_event_info_view().event_info(0x47) & 0x20 == 0
            && self.ancilla_check_for_entrance_trigger(3)
        {
            self.set_special_entrance_trigger(4);
            self.set_subsubmodule(0);
            self.scratch_word_view_mut()
                .clear_module_transition_counter();
        }
        let button_mask_b_y = if self.player_state_view().button_b_frames() != 0 {
            self.player_state_view().joypad1h_last() & 0x80
        } else {
            0
        };
        self.player_state_view_mut()
            .set_button_mask_b_y(button_mask_b_y);
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().clear_magic_spell_player_lock();
    }

    fn quake_spell_control_bolts(&mut self, k: usize) {
        let pending_step = self.ancilla_slot_view(k).step();
        self.quake_spell_scratch_view_mut()
            .set_pending_step(pending_step);
        let mut j = self.quake_spell_scratch_view().active_bolt_limit() as i32;
        loop {
            let uj = j as usize;
            if self.quake_bolt_view(uj).phase() != QUAKE_BOLT_TARGET_PHASES[uj] {
                let timer = self.quake_bolt_view_mut(uj).tick_timer();
                if sign8(timer) {
                    self.quake_bolt_view_mut(uj).set_timer(1);
                    let phase = self.quake_bolt_view_mut(uj).advance_phase();
                    if phase != QUAKE_BOLT_TARGET_PHASES[uj] {
                        if j == 0 && phase == 2 {
                            self.ancilla_sfx2_near(0x0c);
                            self.quake_spell_scratch_view_mut().set_active_bolt_limit(1);
                        } else if j == 1 && phase == 2 {
                            self.quake_spell_scratch_view_mut().set_active_bolt_limit(4);
                        } else if j == 4 && phase == 7 {
                            self.quake_spell_scratch_view_mut().set_pending_step(1);
                        }
                        self.ancilla_draw_quake_initial_bolts(uj);
                    }
                } else {
                    self.ancilla_draw_quake_initial_bolts(uj);
                }
            }
            j -= 1;
            if j < 0 {
                break;
            }
        }
        let step = self.quake_spell_scratch_view().pending_step();
        self.ancilla_slot_view_mut(k).set_step(step);
    }

    fn ancilla_draw_quake_initial_bolts(&mut self, k: usize) {
        const QUAKE_GROUND_BOLT_OAM_STARTS: [u8; 5] = [0, 0x18, 0, 0x18, 0x2f];

        let t = self
            .quake_bolt_view(k)
            .phase()
            .wrapping_add(QUAKE_GROUND_BOLT_OAM_STARTS[k]) as usize;
        let mut oam = self.oam_state_view().current_pointer_usize();
        let idx = QUAKE_INITIAL_BOLT_FRAME_RANGES[t] as usize;
        let end = QUAKE_INITIAL_BOLT_FRAME_RANGES[t + 1] as usize;
        for item_idx in idx..end {
            let sprite = QUAKE_INITIAL_BOLT_SPRITES[item_idx];
            let x = self
                .quake_spell_scratch_view()
                .origin_x()
                .wrapping_add(sprite.x as u16)
                .wrapping_sub(self.world_scroll().bg2_x());
            let y = self
                .quake_spell_scratch_view()
                .origin_y()
                .wrapping_add(sprite.y as u16)
                .wrapping_sub(self.world_scroll().bg2_y());

            let mut xval = self.oam_state_view().entry_x(oam);
            let mut yval = 0xf0;
            if x < 256 && y < 256 {
                xval = x as u8;
                if y < 0xf0 {
                    yval = y as u8;
                }
            }
            self.oam_state_view_mut().write_entry(
                oam,
                xval,
                yval,
                QUAKE_GROUND_BOLT_CHARS[(sprite.flags & 0x0f) as usize],
                (sprite.flags & 0xc0) | 0x3c,
            );
            let value = 2;
            self.oam_state_view_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            oam += 4;
            let cur = self.oam_state_view().current_pointer().wrapping_add(4);
            let ext = self
                .oam_state_view()
                .current_extended_pointer()
                .wrapping_add(1);
            self.oam_state_view_mut().set_current_pointer(cur);
            self.oam_state_view_mut().set_current_extended_pointer(ext);
        }
    }

    fn quake_spell_spread_bolts(&mut self, k: usize) {
        if self.ancilla_slot_view(k).step() != 1 {
            return;
        }
        if self.ancilla_slot_view(k).timer() == 0 {
            let mut quake = self.ancilla_slot_view_mut(k);
            quake.set_timer(2);
            let spread_phase = quake.advance_item_to_link();
            if spread_phase == 55 {
                quake.set_step(2);
                return;
            }
        }
        let t = self.ancilla_slot_view(k).item_to_link() as usize;
        let idx = QUAKE_SPREAD_BOLT_FRAME_RANGES[t] as usize;
        let end = QUAKE_SPREAD_BOLT_FRAME_RANGES[t + 1] as usize;
        let mut oam = self.oam_state_view().current_pointer_usize();
        for item_idx in idx..end {
            let sprite = QUAKE_SPREAD_BOLT_SPRITES[item_idx];
            self.oam_state_view_mut().write_entry(
                oam,
                sprite.x as u8,
                sprite.y as u8,
                QUAKE_GROUND_BOLT_CHARS[(sprite.flags & 0x0f) as usize],
                (sprite.flags & 0xc0) | 0x3c,
            );
            let value = (sprite.flags >> 4) & 3;
            self.oam_state_view_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            let cur = self.oam_state_view().current_pointer().wrapping_add(4);
            let ext = self
                .oam_state_view()
                .current_extended_pointer()
                .wrapping_add(1);
            self.oam_state_view_mut().set_current_pointer(cur);
            self.oam_state_view_mut().set_current_extended_pointer(ext);
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
        }
    }

    fn ancilla1_f_hookshot(&mut self, k: usize) {
        const HOOKSHOT_MOVE_X: [i8; 4] = [0, 0, 8, -8];
        const HOOKSHOT_MOVE_Y: [i8; 4] = [8, -9, 0, 0];
        const HOOKSHOT_DRAW_FLAGS: [u8; 12] =
            [0, 0, 0xff, 0x80, 0x80, 0xff, 0x40, 0xff, 0x40, 0, 0xff, 0];
        const HOOKSHOT_DRAW_CHAR: [u8; 12] =
            [9, 0x0a, 0xff, 9, 0x0a, 0xff, 9, 0xff, 0x0a, 9, 0xff, 0x0a];

        if self.frame_state().submodule == 0 {
            if self.ancilla_slot_view(k).timer() == 0 {
                self.ancilla_slot_view_mut(k).set_timer(7);
                self.ancilla_sfx2_pan(k, 0x0a);
            }

            if !self.player_state_view().has_hookshot_interlock() {
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
                            && self.ancilla_check_sprite_collision(k).is_some()
                            && self.ancilla_slot_view(k).step() == 0
                        {
                            let mut hookshot = self.ancilla_slot_view_mut(k);
                            hookshot.set_step(1);
                            hookshot.negate_y_velocity();
                            hookshot.negate_x_velocity();
                        }

                        self.hookshot_check_tile_collision(k as i32);

                        let mut r0 = 0u8;
                        let contact = if self.world_location_state().is_indoors() {
                            if self.ancilla_slot_view(k).direction() & 2 == 0 {
                                r0 = (self.tile_detect_position_view().vertical_ledge()
                                    | (self.tile_detect_position_view().vertical_ledge() >> 4))
                                    & 3;
                            } else {
                                r0 = self.tile_detect_position_view().horizontal_ledge() & 3;
                            }
                            r0 != 0
                        } else {
                            (self.tile_detect_position_view().horizontal_ledge() & 3
                                | self.tile_detect_position_view().vertical_ledge()
                                | self.tile_detect_position_view().diagonal_ledge_tiles())
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
                                        != self.tile_detect_position_view().interacting_tile_low())
                            {
                                self.ancilla_slot_view_mut(k).set_g(2);
                                let l = self.ancilla_slot_view_mut(k).retreat_l();
                                if sign8(l) {
                                    self.ancilla_slot_view_mut(k).set_l(0);
                                }
                            } else {
                                let interacting_tile =
                                    self.tile_detect_position_view().interacting_tile_low();
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
                                    self.tile_detect_position_view().collision_bits();
                                let blocked = (((collision_bits >> 4)
                                    | collision_bits
                                    | self.tile_detect_position_view().stair_tile() as u16
                                    | self.tile_detect_position_view().slope_collision_bits())
                                    & 3)
                                    != 0;
                                if blocked && self.ancilla_slot_view(k).step() == 0 {
                                    let mut hookshot = self.ancilla_slot_view_mut(k);
                                    hookshot.set_step(1);
                                    hookshot.negate_y_velocity();
                                    hookshot.negate_x_velocity();
                                    if self.tile_detect_position_view().misc_tiles() & 3 == 0 {
                                        self.ancilla_add_hookshot_wall_clink(k, 6, 1);
                                        self.ancilla_sfx2_pan(
                                            k,
                                            if self.tile_detect_position_view().misc_tiles() & 0x30
                                                != 0
                                            {
                                                6
                                            } else {
                                                5
                                            },
                                        );
                                    }
                                }

                                if self.tile_detect_position_view().misc_tiles() & 3 != 0 {
                                    if self.ancilla_slot_view(k).item_to_link() < 4 {
                                        self.ancilla_slot_view_mut(k).clear();
                                        return;
                                    }
                                    self.player_state_view_mut().set_hookshot_interlock(1);
                                    self.messaging_state_view_mut().set_effect_index(k as u8);
                                }
                            }
                        }
                    }
                }
            }
        }

        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        if self.ancilla_slot_view(k).l() != 0 {
            self.oam_state_view_mut().set_priority_word(0x3000);
        }
        let mut oam = self.oam_state_view().current_pointer_usize();

        let mut j = self.ancilla_slot_view(k).direction() as usize * 3;
        let mut x = info_x;
        let mut y = info_y;
        for i in (0..=2).rev() {
            if HOOKSHOT_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x,
                    y,
                    HOOKSHOT_DRAW_CHAR[j],
                    HOOKSHOT_DRAW_FLAGS[j] | 2 | self.oam_state_view().priority_high(),
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
        if HOOKSHOT_MOVE_Y[j] == 0 {
            y = y.wrapping_add(4);
        }
        if HOOKSHOT_MOVE_X[j] == 0 {
            x = x.wrapping_add(4);
        }
        loop {
            if HOOKSHOT_MOVE_Y[j] != 0 {
                y = y.wrapping_add((HOOKSHOT_MOVE_Y[j] as i32 + r10) as i16 as u16);
            }
            if HOOKSHOT_MOVE_X[j] != 0 {
                x = x.wrapping_add((HOOKSHOT_MOVE_X[j] as i32 + r10) as i16 as u16);
            }
            if !self.hookshot_check_proximity_to_link(x as i32, y as i32) {
                self.ancilla_set_oam(
                    oam,
                    x,
                    y,
                    0x19,
                    (self.frame_state().frame_counter & 2) << 6
                        | 2
                        | self.oam_state_view().priority_high(),
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

    fn ancilla_draw_ether_blitz_ball(
        &mut self,
        oam: usize,
        arp: &AncillaRadialProjection,
        s: usize,
    ) -> usize {
        const ETHER_BLITZ_BALL_CHAR: [u8; 2] = [0x68, 0x6a];

        let x = self
            .ether_orbit_view()
            .orbit_x()
            .wrapping_add(if arp.r6 != 0 {
                0u16.wrapping_sub(arp.r4 as u16)
            } else {
                arp.r4 as u16
            })
            .wrapping_sub(8)
            .wrapping_sub(self.world_scroll().bg2_x());
        let y = self
            .ether_orbit_view()
            .orbit_y()
            .wrapping_add(if arp.r2 != 0 {
                0u16.wrapping_sub(arp.r0 as u16)
            } else {
                arp.r0 as u16
            })
            .wrapping_sub(8)
            .wrapping_sub(self.world_scroll().bg2_y());
        self.ancilla_set_oam(oam, x, y, ETHER_BLITZ_BALL_CHAR[s], 0x3c, 2);
        self.ancilla_allocate_oam_from_custom_region(oam + 4)
    }

    fn ancilla_draw_ether_blitz_segment(
        &mut self,
        oam: usize,
        arp: &AncillaRadialProjection,
        s: usize,
        k: usize,
    ) -> usize {
        const ETHER_SPLITTING_BLITZ_SEGMENT_X: [i8; 16] = [
            -8, -16, -24, -16, -8, 0, 8, -16, -8, -16, -24, -16, -8, 0, 8, 0,
        ];
        const ETHER_SPLITTING_BLITZ_SEGMENT_Y: [i8; 16] = [
            8, 0, -8, -16, -24, -16, -8, -16, 8, 0, -8, -16, -24, -16, -8, 0,
        ];
        const ETHER_SPLITTING_BLITZ_SEGMENT_CHAR: [u8; 32] = [
            0x40, 0x42, 0x66, 0x64, 0x62, 0x60, 0x64, 0x66, 0x42, 0x40, 0x66, 0x64, 0x60, 0x62,
            0x64, 0x66, 0x68, 0x42, 0x68, 0x64, 0x68, 0x60, 0x68, 0x64, 0x68, 0x40, 0x68, 0x66,
            0x68, 0x62, 0x68, 0x64,
        ];
        const ETHER_SPLITTING_BLITZ_SEGMENT_FLAGS: [u8; 32] = [
            0x3c, 0x3c, 0xfc, 0xfc, 0x3c, 0x3c, 0xbc, 0xbc, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c,
            0x7c, 0x7c, 0x3c, 0x7c, 0x3c, 0x3c, 0x3c, 0xbc, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0xfc,
            0x3c, 0xbc, 0x3c, 0xbc,
        ];

        let x = if arp.r6 != 0 {
            0u16.wrapping_sub(arp.r4 as u16)
        } else {
            arp.r4 as u16
        };
        let y = if arp.r2 != 0 {
            0u16.wrapping_sub(arp.r0 as u16)
        } else {
            arp.r0 as u16
        };
        let t = s * 8 + k;
        let base_x = x
            .wrapping_add(self.ether_orbit_view().orbit_x())
            .wrapping_sub(8)
            .wrapping_sub(self.world_scroll().bg2_x());
        let base_y = y
            .wrapping_add(self.ether_orbit_view().orbit_y())
            .wrapping_sub(8)
            .wrapping_sub(self.world_scroll().bg2_y());
        self.ancilla_set_oam(
            oam,
            base_x,
            base_y,
            ETHER_SPLITTING_BLITZ_SEGMENT_CHAR[t * 2],
            ETHER_SPLITTING_BLITZ_SEGMENT_FLAGS[t * 2],
            2,
        );
        self.ancilla_set_oam(
            oam + 4,
            x.wrapping_add(self.ether_orbit_view().orbit_x())
                .wrapping_add(ETHER_SPLITTING_BLITZ_SEGMENT_X[t] as i16 as u16)
                .wrapping_sub(self.world_scroll().bg2_x()),
            y.wrapping_add(self.ether_orbit_view().orbit_y())
                .wrapping_add(ETHER_SPLITTING_BLITZ_SEGMENT_Y[t] as i16 as u16)
                .wrapping_sub(self.world_scroll().bg2_y()),
            ETHER_SPLITTING_BLITZ_SEGMENT_CHAR[t * 2 + 1],
            ETHER_SPLITTING_BLITZ_SEGMENT_FLAGS[t * 2 + 1],
            2,
        );
        self.ancilla_allocate_oam_from_custom_region(oam + 8)
    }

    fn ancilla_draw_ether_blitz(&mut self, k: usize) {
        const ETHER_BLITZ_ORB_FLAGS: [u8; 8] = [0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c];
        const ETHER_BLITZ_SEGMENT_CHAR: [u8; 4] = [0x40, 0x42, 0x44, 0x46];

        let (x, mut y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let t = self.ancilla_slot_view(k).item_to_link() as usize;
        let mut i = self.ancilla_slot_view(k).work_byte_25();
        let mut m = 0usize;
        loop {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ETHER_BLITZ_SEGMENT_CHAR[t * 2 + m],
                ETHER_BLITZ_ORB_FLAGS[0] | self.oam_state_view().priority_high(),
                2,
            );
            y = y.wrapping_sub(16);
            oam += 4;
            m ^= 1;
            i = i.wrapping_sub(1);
            if sign8(i) {
                break;
            }
        }
        if self.ancilla_slot_view(k).step() == 1 {
            self.ancilla_draw_ether_orb(k, oam);
        }
    }

    fn ancilla_draw_ether_orb(&mut self, k: usize, mut oam: usize) {
        const ETHER_BLITZ_ORB_CHAR: [u8; 8] = [0x48, 0x48, 0x4a, 0x4a, 0x4c, 0x4c, 0x4e, 0x4e];
        const ETHER_BLITZ_ORB_FLAGS: [u8; 8] = [0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c];

        let mut y = self
            .ether_orbit_view()
            .orb_y()
            .wrapping_sub(1)
            .wrapping_sub(self.world_scroll().bg2_y());
        let mut x = self
            .ether_orbit_view()
            .orb_x()
            .wrapping_sub(8)
            .wrapping_sub(self.world_scroll().bg2_x());
        let t = self.ancilla_slot_view(k).item_to_link() as usize * 4;

        for i in 0..4 {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ETHER_BLITZ_ORB_CHAR[t + i],
                ETHER_BLITZ_ORB_FLAGS[t + i],
                2,
            );
            oam += 4;
            oam = self.ancilla_allocate_oam_from_custom_region(oam);
            x = x.wrapping_add(16);
            if i == 1 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(16);
            }
        }
    }

    fn ancilla_draw_bombos_fire_column(&mut self, kk: usize) {
        const BOMBOS_SPELL_FIRE_COLUMN_X: [i8; 39] = [
            0, -1, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, -1, 1, -1, -1, 2, -1, -1,
        ];
        const BOMBOS_SPELL_FIRE_COLUMN_Y: [i8; 39] = [
            0, -1, -1, 0, -4, -1, 0, -8, -1, 0, -12, -1, 0, -16, -1, 0, -4, -20, 0, -8, -24, 0,
            -12, -28, 0, -16, -32, 0, -16, -32, -18, -34, -1, -35, -1, -1, -36, -1, -1,
        ];
        const BOMBOS_SPELL_FIRE_COLUMN_FLAGS: [u8; 39] = [
            0x3c, 0xff, 0xff, 0x3c, 0x3c, 0xff, 0x3c, 0x3c, 0xff, 0x7c, 0x7c, 0xff, 0x3c, 0x7c,
            0xff, 0x3c, 0x3c, 0x3c, 0xbc, 0x3c, 0x3c, 0x7c, 0x3c, 0x3c, 0x3c, 0x3c, 0x7c, 0x3c,
            0x3c, 0x3c, 0x3c, 0x3c, 0xff, 0x3c, 0xff, 0xff, 0x3c, 0xff, 0xff,
        ];
        const BOMBOS_SPELL_FIRE_COLUMN_CHAR: [u8; 39] = [
            0x40, 0xff, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44,
            0xff, 0x40, 0x46, 0x44, 0x4a, 0x4a, 0x48, 0x4c, 0x4c, 0x4a, 0x4e, 0x4c, 0x4a, 0x4e,
            0x6a, 0x4c, 0x4e, 0x68, 0xff, 0x6a, 0xff, 0xff, 0x4e, 0xff, 0xff,
        ];

        self.ancilla_allocate_oam_from_region_a_or_d_or_f(kk, 0x10);
        let mut oam = self.oam_state_view().current_pointer_usize();
        for _ in 0..1 {
            let mut k = self.bombos_fire_column_view(kk).phase() as usize;
            if k == 13 {
                continue;
            }
            k = k * 3 + 2;
            for _ in 0..3 {
                if BOMBOS_SPELL_FIRE_COLUMN_CHAR[k] != 0xff {
                    let x = self.bombos_fire_column_view(kk).x();
                    let y = self.bombos_fire_column_view(kk).y();
                    self.ancilla_set_oam(
                        oam,
                        x.wrapping_add(BOMBOS_SPELL_FIRE_COLUMN_X[k] as i16 as u16)
                            .wrapping_sub(self.world_scroll().bg2_x()),
                        y.wrapping_add(BOMBOS_SPELL_FIRE_COLUMN_Y[k] as i16 as u16)
                            .wrapping_sub(self.world_scroll().bg2_y()),
                        BOMBOS_SPELL_FIRE_COLUMN_CHAR[k],
                        BOMBOS_SPELL_FIRE_COLUMN_FLAGS[k],
                        2,
                    );
                    oam += 4;
                }
                oam = self.ancilla_allocate_oam_from_custom_region(oam);
                k = k.wrapping_sub(1);
            }
        }
    }

    fn ancilla_draw_bombos_blast(&mut self, k: usize) {
        const BOMBOS_SPELL_DRAW_BLAST_X: [i8; 32] = [
            -8, -1, -1, -1, -12, -4, -12, -4, -16, 0, -16, 0, -16, 0, -16, 0, -17, 1, -17, 1, -19,
            3, -19, 3, -19, 3, -19, 3, -19, 3, -19, 3,
        ];
        const BOMBOS_SPELL_DRAW_BLAST_Y: [i8; 32] = [
            -8, -1, -1, -1, -12, -12, -4, -4, -16, -16, 0, 0, -16, -16, 0, 0, -17, -17, 1, 1, -19,
            -19, 3, 3, -19, -19, 3, 3, -19, -19, 3, 3,
        ];
        const BOMBOS_SPELL_DRAW_BLAST_FLAGS: [u8; 32] = [
            0x3c, 0xff, 0xff, 0xff, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c,
            0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc,
            0x3c, 0x7c, 0xbc, 0xfc,
        ];
        const BOMBOS_SPELL_DRAW_BLAST_CHAR: [u8; 32] = [
            0x60, 0xff, 0xff, 0xff, 0x62, 0x62, 0x62, 0x62, 0x64, 0x64, 0x64, 0x64, 0x66, 0x66,
            0x66, 0x66, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x6a, 0x6a, 0x6a, 0x6a,
            0x4e, 0x4e, 0x4e, 0x4e,
        ];

        let x = self.bombos_spell_scratch_view().blast_x(k);
        let y = self.bombos_spell_scratch_view().blast_y(k);
        if self.bombos_blast_view(k).phase() == 8 {
            return;
        }

        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 0x10);
        let mut oam = self.oam_state_view().current_pointer_usize();

        let mut t = self.bombos_blast_view(k).phase() as usize * 4 + 3;
        for _ in 0..4 {
            if BOMBOS_SPELL_DRAW_BLAST_CHAR[t] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(BOMBOS_SPELL_DRAW_BLAST_X[t] as i16 as u16)
                        .wrapping_sub(self.world_scroll().bg2_x()),
                    y.wrapping_add(BOMBOS_SPELL_DRAW_BLAST_Y[t] as i16 as u16)
                        .wrapping_sub(self.world_scroll().bg2_y()),
                    BOMBOS_SPELL_DRAW_BLAST_CHAR[t],
                    BOMBOS_SPELL_DRAW_BLAST_FLAGS[t],
                    2,
                );
                oam += 4;
            }
            oam = self.ancilla_allocate_oam_from_custom_region(oam);
            t = t.wrapping_sub(1);
        }
    }

    fn ancilla27_duck(&mut self, k: usize) {
        if self.frame_state().submodule == 0 {
            if self.ancilla_slot_view(k).timer() != 0 {
                let xt: u16 = if self.enhanced_features_view().has(1) {
                    0x40
                } else {
                    0
                };
                self.ancilla_set_xy(
                    k,
                    self.world_scroll()
                        .bg2_x()
                        .wrapping_sub(16)
                        .wrapping_sub(xt),
                    self.player_state_view().y().wrapping_sub(8),
                );
                return;
            }

            self.ancilla_slot_view_mut(k).subtract_g(1);
            if sign8(self.ancilla_slot_view(k).g()) {
                let value = 0x28;
                self.ancilla_slot_view_mut(k).set_g(value);
                self.ancilla_sfx3_pan(k, 0x1e);
            }

            if self.ancilla_slot_view(k).l() != 0 || self.ancilla_slot_view(k).step() != 0 {
                if self.ancilla_slot_view(k).l() == 0 && self.ancilla_slot_view(k).step() != 0 {
                    self.increment_modal_pause_flag();
                }
                self.ancilla_slot_view_mut(k).tick_z_velocity();
                self.ancilla_move_z(k);
            }
            self.ancilla_move_x(k);

            if self.ancilla_slot_view(k).l() != 0 {
                let x = self.ancilla_get_x(k);
                if self.ancilla_slot_view(k).step() != 0 {
                    self.increment_modal_pause_flag();
                }
                if !sign16(x) && x >= self.player_state_view().x() {
                    if self.ancilla_slot_view(k).step() != 0 {
                        let value = 0;
                        self.ancilla_slot_view_mut(k).set_step(value);
                        self.player_state_view_mut().set_visibility_status(0);
                        self.follower_state_view_mut().set_appearance_none_flag(0);
                        self.player_state_view_mut().clear_item_hold_pose();
                        let value = 0;
                        self.ancilla_slot_view_mut(k).set_y_velocity(value);
                        self.player_state_view_mut().clear_immobilized();
                        self.player_state_view_mut()
                            .clear_sprite_damage_disable_timer();
                        self.player_state_view_mut()
                            .clear_player_special_draw_flag();
                        self.player_state_view_mut().set_blink_countdown(144);
                        if !((self.follower_state_view().indicator() == 12
                            || self.follower_state_view().indicator() == 13)
                            && self.follower_state_view().dropped() != 0)
                        {
                            self.follower_initialize();
                        }
                    }
                } else if self.player_state_view().x().wrapping_sub(x) < 48 {
                    self.draw_duck(k, 3);
                    return;
                }
            } else if self.ancilla_check_link_collision(k, 1)
                && self.frame_state().main_module != 15
            {
                if self.world_location_state().is_outdoors() {
                    if self.player_state_view().handler_state() == 8
                        || self.player_state_view().handler_state() == 9
                        || self.player_state_view().handler_state() == 10
                        || self.player_state_view().near_pit_state_is(2)
                        || (self.player_state_view().item_hold_pose()
                            | self.player_state_view().hookshot_interlock()
                            | self.player_state_view().force_hold_sword_up_state()
                            | self.player_state_view().sprite_damage_disable_timer())
                            != 0
                        || self.player_state_view().is_lifting_or_carrying()
                    {
                        self.draw_duck_default(k);
                        return;
                    }
                    for i in (0..5).rev() {
                        let a = self.ancilla_slot_view(i).ancilla_type();
                        if a == 0x2a || a == 0x1f || a == 0x30 || a == 0x31 || a == 0x41 {
                            let value = 0;
                            self.ancilla_slot_view_mut(i).set_ancilla_type(value);
                        }
                    }
                    if self.follower_state_view().indicator() == 9 {
                        self.follower_state_view_mut().set_indicator(0);
                        self.follower_state_view_mut().set_appearance_none_flag(0);
                    }
                }
                {
                    let mut player = self.player_state_view_mut();
                    player.clear_state_bits();
                    player.clear_picking_throw_state();
                }

                self.world_scroll_mut().set_bg1_x_offset(0);
                self.world_scroll_mut().set_bg1_y_offset(0);
                self.link_reset_properties_a();
                self.player_state_view_mut().clear_deep_water_state();
                self.player_state_view_mut()
                    .clear_pull_for_rupees_sprite_need();
                self.player_state_view_mut().set_visibility_status(12);
                self.player_state_view_mut().clear_handler_state();
                self.player_state_view_mut().set_item_hold_pose(1);
                self.player_state_view_mut().immobilize();
                self.player_state_view_mut()
                    .set_sprite_damage_disable_timer(1);
                self.follower_state_view_mut().set_appearance_none_flag(1);
                let value = 2;
                self.ancilla_slot_view_mut(k).set_step(value);
                self.increment_modal_pause_flag();
                self.player_state_view_mut().clear_given_damage();
                if self.world_location_state().is_indoors() {
                    let value = self.world_location_state().indoor_flag;
                    self.player_state_view_mut()
                        .set_player_special_draw_flag(value);
                }
            }
        }
        self.draw_duck_default(k);
    }

    fn draw_duck_default(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).tick_work_byte_3();
        if sign8(self.ancilla_slot_view(k).work_byte_3()) {
            let value = 3;
            self.ancilla_slot_view_mut(k).set_work_byte_3(value);
            self.ancilla_slot_view_mut(k).add_k(1);
            if self.ancilla_slot_view(k).k() == 3 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_k(value);
            }
        }
        self.draw_duck(k, self.ancilla_slot_view(k).k());
    }

    fn draw_duck(&mut self, k: usize, j: u8) {
        self.world_transient_mut()
            .set_flag_travel_bird(TRAVEL_BIRD_DMA_TILE_OFFSETS[j as usize]);

        let (x, y) = self.ancilla_prep_oam_coord(k);

        let mut oam = self.oam_state_view().current_pointer_usize();
        let z = if self.ancilla_slot_view(k).z() != 0 {
            self.ancilla_slot_view(k).z() as i8 as i16 as u16
        } else {
            0
        };
        let n = self.ancilla_slot_view(k).step() as usize + 1;
        for i in 0..n {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(TRAVEL_BIRD_DRAW_X_OFFSETS[i] as i16 as u16),
                y.wrapping_add(z)
                    .wrapping_add(TRAVEL_BIRD_DRAW_Y_OFFSETS[i] as i16 as u16),
                TRAVEL_BIRD_DRAW_CHARS[i],
                TRAVEL_BIRD_DRAW_FLAGS[i] | 0x30,
                2,
            );
            oam += 4;
        }

        self.ancilla_draw_shadow(oam, 1, x, y.wrapping_add(28), 0x30);
        oam += 8;
        if self.ancilla_slot_view(k).step() != 0 {
            self.ancilla_draw_shadow(oam, 1, x.wrapping_sub(7), y.wrapping_add(28), 0x30);
        }

        if !sign16(x) && x >= 0x0130 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            if self.ancilla_slot_view(k).l() == 0 && self.ancilla_slot_view(k).step() != 0 {
                let main_module = self.frame_state().main_module;
                self.set_submodule(10);
                self.set_saved_module_for_menu(main_module);
                self.set_main_module(14);
            }
        }
    }

    fn ancilla_draw_weathervane_explosion_wood_debris(&mut self, k: usize) {
        const WEATHERVANE_EXPLODE_CHAR: [u8; 2] = [0x4e, 0x4f];

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let y = y.wrapping_sub(self.ancilla_slot_view(k).z() as i8 as i16 as u16);
        let i = self.ancilla_slot_view(k).item_to_link();
        if sign8(i) {
            return;
        }
        let oam = self.oam_state_view().current_pointer_usize()
            + ((self.weather_vane_state().oam_offset >> 2) as usize) * 4;
        self.ancilla_set_oam(oam, x, y, WEATHERVANE_EXPLODE_CHAR[i as usize], 0x3c, 0);
        self.advance_weather_vane_oam_offset(4);
    }

    fn ancilla38_cutscene_duck(&mut self, k: usize) {
        const TRAVEL_BIRD_INTRO_FLAGS_BY_DIRECTION: [u8; 2] = [0x40, 0];
        const TRAVEL_BIRD_INTRO_X_SPEED_LIMITS: [u8; 2] = [28, 60];

        if self.frame_state().frame_counter & 31 == 0 {
            self.ancilla_sfx3_pan(k, 0x1e);
        }

        if sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3()) {
            let mut duck = self.ancilla_slot_view_mut(k);
            duck.set_work_byte_3(3);
            duck.toggle_k_bit0();
        }

        if self.ancilla_slot_view_mut(k).tick_aux_timer() == 0 {
            self.ancilla_slot_view_mut(k).set_aux_timer(1);
            if self.ancilla_slot_view(k).l() == 0 {
                let item_to_link = self.ancilla_slot_view_mut(k).retreat_item_to_link();
                if !sign8(item_to_link) {
                    let z_delta = if self.ancilla_slot_view(k).step() != 0 {
                        1
                    } else {
                        (-1i8) as u8
                    };
                    self.ancilla_slot_view_mut(k).add_z_velocity(z_delta);
                    if abs8(self.ancilla_slot_view(k).z_velocity()) >= 12 {
                        let step = self.ancilla_slot_view(k).step() ^ 1;
                        self.ancilla_slot_view_mut(k).set_step(step);
                    }
                    self.ancilla38_cutscene_duck_after_stuff(k);
                    return;
                }
                let mut duck = self.ancilla_slot_view_mut(k);
                duck.set_item_to_link(0);
                duck.set_step(0);
                duck.set_x_velocity(TRAVEL_BIRD_INTRO_X_SPEED_LIMITS[0]);
                duck.set_z_velocity((-16i8) as u8);
                duck.advance_l();
                duck.set_step(3);
            }
            let x_delta = if self.ancilla_slot_view(k).step() & 1 == 0 {
                1
            } else {
                (-1i8) as u8
            };
            let x_velocity = self.ancilla_slot_view_mut(k).add_x_velocity(x_delta);
            let absx = abs8(x_velocity);
            if absx == 0 {
                let l = self.ancilla_slot_view_mut(k).advance_l();
                if l == 7 {
                    self.ancilla_slot_view_mut(k).set_s_player(1);
                }
            }
            if absx
                >= TRAVEL_BIRD_INTRO_X_SPEED_LIMITS[self.ancilla_slot_view(k).s_player() as usize]
            {
                let step = self.ancilla_slot_view(k).step() ^ 3;
                self.ancilla_slot_view_mut(k).set_step(step);
            }
            let direction = if sign8(self.ancilla_slot_view(k).x_velocity()) {
                2
            } else {
                3
            };
            self.ancilla_slot_view_mut(k).set_direction(direction);
            let t = TRAVEL_BIRD_INTRO_X_SPEED_LIMITS[self.ancilla_slot_view(k).s_player() as usize]
                .wrapping_sub(absx)
                >> 1;
            let z_velocity = if self.ancilla_slot_view(k).step() & 2 != 0 {
                0u8.wrapping_sub(t)
            } else {
                t
            };
            self.ancilla_slot_view_mut(k).set_z_velocity(z_velocity);
        }
        self.ancilla38_cutscene_duck_after_stuff(k);
    }

    fn ancilla38_cutscene_duck_after_stuff(&mut self, k: usize) {
        const TRAVEL_BIRD_INTRO_FLAGS_BY_DIRECTION: [u8; 2] = [0x40, 0];

        self.ancilla_move_x(k);
        self.ancilla_move_z(k);
        let value = TRAVEL_BIRD_DMA_TILE_OFFSETS[self.ancilla_slot_view(k).k() as usize + 1];
        self.set_travel_bird_tile_offset(value);
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let oam = self.oam_state_view().current_pointer_usize();
        self.ancilla_set_oam(
            oam,
            x.wrapping_add(TRAVEL_BIRD_DRAW_X_OFFSETS[0] as i16 as u16),
            y.wrapping_add(self.ancilla_slot_view(k).z() as i8 as i16 as u16)
                .wrapping_add(TRAVEL_BIRD_DRAW_Y_OFFSETS[0] as i16 as u16),
            TRAVEL_BIRD_DRAW_CHARS[0],
            TRAVEL_BIRD_DRAW_FLAGS[0]
                | 0x30
                | TRAVEL_BIRD_INTRO_FLAGS_BY_DIRECTION
                    [(self.ancilla_slot_view(k).direction() & 1) as usize],
            2,
        );
        self.ancilla_draw_shadow(oam + 4, 1, x, y.wrapping_add(48), 0x30);
        if !sign16(x) && x >= 248 {
            self.ancilla_slot_view_mut(k).clear();
            self.set_submodule(0);
            self.inventory_items_mut().set_flute(3);
        }
    }

    fn ancilla16_hit_stars(&mut self, k: usize) {
        const ANCILLA_HIT_STARS_CHAR: [u8; 2] = [0x90, 0x91];

        self.ancilla_slot_view_mut(k).tick_work_byte_3();
        if !sign8(self.ancilla_slot_view(k).work_byte_3()) {
            return;
        }

        let value = 0;

        self.ancilla_slot_view_mut(k).set_work_byte_3(value);
        if self.frame_state().submodule == 0 {
            self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(self.ancilla_slot_view(k).aux_timer()) {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_aux_timer(value);
                let value = 1;
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
            }
            if self.ancilla_slot_view(k).item_to_link() != 0 {
                self.ancilla_slot_view_mut(k).subtract_y_velocity(4);
                let value = self.ancilla_slot_view(k).y_velocity();
                self.ancilla_slot_view_mut(k).set_x_velocity(value);
                if self.ancilla_slot_view(k).y_velocity() < 232 {
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                    return;
                }
                self.ancilla_move_y(k);
                self.ancilla_move_x(k);
            }
        }
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let ax = self.ancilla_get_x(k);
        let tt = u16::from(self.ancilla_slot_view(k).a())
            | (u16::from(self.ancilla_slot_view(k).b()) << 8);
        let r8 = tt
            .wrapping_mul(2)
            .wrapping_sub(ax)
            .wrapping_sub(8)
            .wrapping_sub(self.world_scroll().bg2_x());

        if self.ancilla_slot_view(k).step() == 2 {
            self.ancilla_allocate_oam_from_region_b_or_e(8);
        }

        let mut oam = self.oam_state_view().current_pointer_usize();
        let mut x = x;
        let mut flags = 0;
        for _ in (0..=1).rev() {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ANCILLA_HIT_STARS_CHAR[self.ancilla_slot_view(k).item_to_link() as usize],
                self.oam_state_view().priority_high() | 4 | flags,
                0,
            );
            flags = 0x40;
            x = (x & 0xff00) | (r8 & 0x00ff);
            oam = self.hit_stars_update_oam_buffer_position(oam + 4);
        }
    }

    fn ancilla17_shovel_dirt(&mut self, k: usize) {
        const SHOVEL_DIRT_XY: [i8; 8] = [18, -13, -9, 4, 18, 13, -9, -11];
        const SHOVEL_DIRT_CHAR: [u8; 2] = [0x40, 0x50];

        let (mut x, mut y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        if self.ancilla_slot_view(k).timer() == 0 {
            let value = 8;
            self.ancilla_slot_view_mut(k).set_timer(value);
            self.ancilla_slot_view_mut(k).add_item_to_link(1);
            if self.ancilla_slot_view(k).item_to_link() == 2 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            }
        }
        let b = self.ancilla_slot_view(k).item_to_link() as usize;
        let j = b + if self.player_state_view().facing() == 4 {
            0
        } else {
            2
        };
        x = x.wrapping_add(SHOVEL_DIRT_XY[j * 2 + 1] as i16 as u16);
        y = y.wrapping_add(SHOVEL_DIRT_XY[j * 2] as i16 as u16);
        for i in 0..2 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add((i * 8) as u16),
                y,
                SHOVEL_DIRT_CHAR[b].wrapping_add(i as u8),
                4 | self.oam_state_view().priority_high(),
                0,
            );
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
        }
    }

    fn ancilla_magic_powder_draw(&mut self, k: usize) {
        const MAGIC_POWDER_DRAW_X: [i8; 76] = [
            -5, -12, 2, -9, -7, -10, -6, -2, -6, -12, 1, -6, -6, -12, 1, -6, -6, -12, 1, -6, -6,
            -12, 1, -6, -6, -12, 1, -6, -17, -23, -14, -19, -11, -18, -9, -13, -4, -13, -1, -8, -3,
            -9, 0, -5, -3, -10, -1, -5, -4, -13, -1, -8, -3, -9, 0, -5, -3, -10, -1, -5, -3, -13,
            -1, -8, 9, 15, 6, 11, 3, 10, 1, 5, -4, 5, -7, 0,
        ];
        const MAGIC_POWDER_DRAW_Y: [i8; 76] = [
            -20, -15, -13, -7, -18, -13, -13, -13, -20, -13, -13, -8, -20, -13, -13, -8, -19, -12,
            -12, -7, -18, -11, -11, -6, -17, -10, -10, -5, -16, -14, -12, -9, -17, -14, -12, -8,
            -18, -14, -13, -6, -33, -31, -29, -26, -28, -25, -23, -19, -22, -18, -17, -10, -2, 0,
            2, 5, -9, -6, -4, 0, -16, -12, -11, -4, -16, -14, -12, -9, -17, -14, -12, -8, -18, -14,
            -13, -6,
        ];
        const MAGIC_POWDER_DRAW_CHAR: [u8; 19] =
            [9, 10, 10, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9];
        const MAGIC_POWDER_DRAW_FLAGS: [u8; 76] = [
            0x68, 0x24, 0xa2, 0x28, 0x68, 0xe2, 0x28, 0xa4, 0x68, 0xe2, 0xa4, 0x28, 0x22, 0xa4,
            0xe8, 0x62, 0x24, 0xa8, 0xe2, 0x64, 0x28, 0xa2, 0xe4, 0x68, 0x22, 0xa4, 0xe8, 0x62,
            0xe2, 0xa4, 0xe8, 0x64, 0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4,
            0xe8, 0x64, 0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4, 0xe8, 0x64,
            0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4, 0xe8, 0x64, 0xe8, 0xa8,
            0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68,
        ];

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let b = self.ancilla_slot_view(k).work_byte_25() as usize;
        let mut j = b * 4;
        for _ in 0..4 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(MAGIC_POWDER_DRAW_X[j] as i16 as u16),
                y.wrapping_add(MAGIC_POWDER_DRAW_Y[j] as i16 as u16),
                MAGIC_POWDER_DRAW_CHAR[b],
                MAGIC_POWDER_DRAW_FLAGS[j] & !0x30 | self.oam_state_view().priority_high(),
                0,
            );
            oam += 4;
            j += 1;
        }
    }

    fn ancilla1_a_powder_dust(&mut self, k: usize) {
        if self.frame_state().submodule == 0 {
            self.powder_apply_damage_to_sprites(k);
            self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(self.ancilla_slot_view(k).aux_timer()) {
                let value = 1;
                self.ancilla_slot_view_mut(k).set_aux_timer(value);
                let j = self.ancilla_slot_view(k).direction() as usize;
                if self.ancilla_slot_view(k).item_to_link() == 9 {
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                    self.dungeon_torch_mut().clear_attr();
                    return;
                }
                let value = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                let value = MAGIC_POWDER_FRAME_TIMERS
                    [self.ancilla_slot_view(k).item_to_link() as usize + j * 10];
                self.ancilla_slot_view_mut(k).set_work_byte_25(value);
            }
        }
        self.ancilla_allocate_oam_from_region_b_or_e(self.ancilla_slot_view(k).num_sprites());
        self.ancilla_magic_powder_draw(k);
    }

    fn powder_apply_damage_to_sprites(&mut self, k: usize) {
        for j in (0..16).rev() {
            if ((self.frame_state().frame_counter ^ j as u8) & 3) != 0
                || self.sprite_slot_view(j).state() != 9
                || (self.sprite_slot_view(j).bump_damage() & 0x20) != 0
            {
                continue;
            }

            let mut hb = self.ancilla_setup_basic_hit_box(k);
            self.sprite_setup_hit_box(j, &mut hb);
            if !self.check_if_hit_boxes_overlap(&hb) {
                continue;
            }

            let mut a = self.sprite_slot_view(j).sprite_type();
            if a != 0x0b
                || {
                    a = self.world_location_state().indoor_flag;
                    a == 0
                }
                || {
                    a = self.dungeon_state_view().room_index2().wrapping_sub(1);
                    a != 0
                }
            {
                if a != 0x0d {
                    self.ancilla_check_damage_to_sprite_preset(j, 10);
                    continue;
                }
                if self.sprite_slot_view(j).head_direction() != 0 {
                    continue;
                }
            }
            let value = 1;
            self.sprite_slot_view_mut(j).set_head_direction(value);
            self.sprite_spawn_poof_garnish_for_ancilla(j);
        }
    }

    fn garnish_alloc_force_for_ancilla(&self) -> usize {
        (0..30)
            .rev()
            .find(|&k| self.garnish_slot_view(k).is_empty())
            .unwrap_or(0)
    }

    fn sprite_spawn_poof_garnish_for_ancilla(&mut self, j: usize) {
        let k = self.garnish_alloc_force_for_ancilla();
        let value = 10;
        self.garnish_slot_view_mut(k).set_garnish_type(value);
        self.garnish_state_view_mut().set_active_type(10);
        let value = self.sprite_slot_view(j).x_low();
        self.garnish_slot_view_mut(k).set_x_low(value);
        let value = self.sprite_slot_view(j).x_high();
        self.garnish_slot_view_mut(k).set_x_high(value);
        let y = self.sprite_get_y(j).wrapping_add(16);
        let value = y as u8;
        self.garnish_slot_view_mut(k).set_y_low(value);
        let value = (y >> 8) as u8;
        self.garnish_slot_view_mut(k).set_y_high(value);
        let value = self.sprite_slot_view(j).floor();
        self.garnish_slot_view_mut(k).set_sprite(value);
        let value = 15;
        self.garnish_slot_view_mut(k).set_countdown(value);
    }

    fn wish_pond_item_draw(&mut self, k: usize) {
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);

        if self.ancilla_slot_view(k).item_to_link() == 1 {
            let value = 5;
            self.ancilla_slot_view_mut(k).set_work_byte_4(value);
        }

        let oam = self.ancilla_receive_item_draw(
            k,
            x,
            y.wrapping_sub(self.ancilla_slot_view(k).z() as i8 as i16 as u16),
        );

        if !self.player_state_view().picking_throw_state_has(2)
            || (!sign8(self.ancilla_slot_view(k).z_velocity())
                && self.ancilla_slot_view(k).z_velocity() >= 2)
        {
            return;
        }

        let xx = self.asset_u8(71, self.ancilla_slot_view(k).item_to_link() as usize);
        self.ancilla_draw_shadow(
            oam,
            if xx == 2 { 1 } else { 2 },
            x.wrapping_sub(if xx == 2 { 0 } else { 4 }),
            y.wrapping_add(40),
            self.oam_state_view().priority_high(),
        );
    }

    fn ancilla_receive_item_draw(&mut self, k: usize, x: u16, y: u16) -> usize {
        let mut oam = self.oam_state_view().current_pointer_usize();
        let j = self.ancilla_slot_view(k).item_to_link() as usize;
        let mut a = WISH_POND_ITEM_OAM_FLAGS[j];
        if sign8(a) {
            a = self.ancilla_slot_view(k).work_byte_4();
        }
        self.ancilla_set_oam(
            oam,
            x,
            y,
            0x24,
            a.wrapping_mul(2) | 0x30,
            RECEIVE_ITEM_OAM_EXT_SIZES[j],
        );
        oam += 4;
        if RECEIVE_ITEM_OAM_EXT_SIZES[j] == 0 {
            self.ancilla_set_oam(oam, x, y.wrapping_add(8), 0x34, a.wrapping_mul(2) | 0x30, 0);
            oam += 4;
        }
        oam
    }

    fn item_receipt_transmute_to_rising_crystal(&mut self, k: usize) {
        let value = 0x3e;
        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_y_velocity(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_x_velocity(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_y_subpixel(value);
        self.ancilla_rising_crystal(k);
    }

    fn ancilla22_item_receipt(&mut self, k: usize) {
        if self.player_state_view().immobilized_flag() != 2 {
            if self.frame_state().submodule != 0
                && self.frame_state().submodule != 43
                && self.frame_state().submodule != 9
            {
                if self.frame_state().submodule == 2 {
                    let value = 16;
                    self.ancilla_slot_view_mut(k).set_timer(value);
                }
            } else {
                self.increment_modal_pause_flag();

                if self.ancilla_slot_view(k).step() != 0 && self.ancilla_slot_view(k).step() != 3 {
                    let value = self.ancilla_slot_view(k).aux_timer().wrapping_sub(1);
                    self.ancilla_slot_view_mut(k).set_aux_timer(value);
                    if sign8(self.ancilla_slot_view(k).aux_timer()) {
                        self.ancilla22_item_receipt_finish(k);
                        return;
                    }
                    if self.ancilla_slot_view(k).aux_timer() == 0 {
                        self.ancilla22_item_receipt_show_message(k);
                    } else {
                        if self.ancilla_slot_view(k).aux_timer() == 40
                            && self.ancilla_slot_view(k).step() != 2
                            && (self.ancilla_add_rupees(k)
                                || self.ancilla_slot_view(k).item_to_link() != 0x17)
                        {
                            self.ancilla_sfx3_near(0x0f);
                        }
                        self.ancilla22_item_receipt_move_label_b(k);
                    }
                } else if self.ancilla_slot_view(k).item_to_link() == 1
                    && self.ancilla_slot_view(k).step() != 2
                {
                    if self.ancilla_slot_view(k).timer() == 0 {
                        self.ancilla22_item_receipt_label_a(k);
                        return;
                    }
                    if self.ancilla_slot_view(k).timer() == 17 {
                        self.start_shared_message_timer(0x0df3);
                        self.follower_state_view_mut().set_indicator(0x0e);
                        self.ancilla22_item_receipt_show_message(k);
                    }
                } else {
                    let value = self.ancilla_slot_view(k).aux_timer().wrapping_sub(1);
                    self.ancilla_slot_view_mut(k).set_aux_timer(value);
                    let a = self.ancilla_slot_view(k).aux_timer();
                    if a == 0 {
                        self.ancilla22_item_receipt_label_a(k);
                        return;
                    }
                    if a == 1 {
                        let item = self.ancilla_slot_view(k).item_to_link();
                        if (item == 0x37 || item == 0x38 || item == 0x39)
                            && self.zelda_read_apui00() != 0
                        {
                            let value = self.ancilla_slot_view(k).aux_timer().wrapping_add(1);
                            self.ancilla_slot_view_mut(k).set_aux_timer(value);
                        } else {
                            self.ancilla22_item_receipt_show_message(k);
                        }
                    }
                }
            }
        }

        self.ancilla22_item_receipt_draw_and_update(k);
    }

    fn ancilla22_item_receipt_label_a(&mut self, k: usize) {
        if self.ancilla_slot_view(k).item_to_link() == 1 && self.ancilla_slot_view(k).step() == 0 {
            self.system_signals_view_mut().set_ambient_sound_effect(5);
            self.system_signals_view_mut().set_music_control(2);
        }
        let handler_state = if self.player_state_view().is_in_deep_water() {
            4
        } else {
            0
        };
        self.player_state_view_mut()
            .set_handler_state(handler_state);
        self.player_state_view_mut().set_receive_item_index(0);
        self.player_state_view_mut().clear_item_hold_pose();
        self.player_state_view_mut()
            .clear_sprite_damage_disable_timer();
        self.ancilla_add_rupees(k);
        self.ancilla22_item_receipt_finish(k);
    }

    fn ancilla22_item_receipt_finish(&mut self, k: usize) {
        self.player_state_view_mut().set_item_receipt_method(0);
        let a = self.ancilla_slot_view(k).item_to_link();
        if a == 23 && self.player_resources_view().heart_pieces() == 0 {
            self.link_receive_item(0x26, 0);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            self.clear_modal_pause_flag();
            return;
        }

        if a == 0x26 || a == 0x3f {
            if self.player_resources_view().health_capacity() != 0xa0 {
                let capacity = self
                    .player_resources_view_mut()
                    .increment_health_capacity_by(8);
                let filler = capacity.wrapping_sub(self.player_resources_view().current_health());
                self.player_resources_view_mut()
                    .increment_heart_filler_by(filler);
                self.ancilla_sfx3_near(0x0d);
            }
        } else if a == 0x3e {
            self.player_state_view_mut().clear_immobilized();
            if self.player_resources_view().health_capacity() != 0xa0 {
                self.player_resources_view_mut()
                    .increment_health_capacity_by(8);
                self.player_resources_view_mut()
                    .increment_heart_filler_by(8);
                self.ancilla_sfx3_near(0x0d);
            }
        } else if a == 0x42 {
            self.player_resources_view_mut()
                .increment_heart_filler_by(8);
        } else if a == 0x45 {
            self.player_resources_view_mut()
                .increment_magic_filler_by(16);
        } else if a == 0x22 || a == 0x23 {
            self.Palette_Load_LinkArmorAndGloves();
        }

        let value = 0;

        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        self.clear_modal_pause_flag();
        let a = self.ancilla_slot_view(k).item_to_link();
        if self.ancilla_slot_view(k).step() == 3 && a != 0x10 && a != 0x26 && a != 0x0f && a != 0x20
        {
            self.prepare_dungeon_exit_from_boss_fight();
        }

        if self.ancilla_slot_view(k).step() != 2 {
            self.player_state_view_mut().clear_immobilized();
        }
    }

    fn ancilla22_item_receipt_show_message(&mut self, k: usize) {
        if self.world_location_state().is_indoors() {
            let room = self.world_location_state().dungeon_room;
            if room == 0x00ff
                || room == 0x010f
                || room == 0x0110
                || room == 0x0112
                || room == 0x011f
            {
                self.ancilla22_item_receipt_move_label_b(k);
                return;
            }
        }
        let item = self.ancilla_slot_view(k).item_to_link() as usize;
        let mut msg = -1i16;
        if self.ancilla_slot_view(k).item_to_link() == 0x38
            || self.ancilla_slot_view(k).item_to_link() == 0x39
        {
            if self.player_resources_view().pendant_flags() & 7 == 7 {
                msg = RECEIVE_ITEM_SPECIAL_MESSAGES[item - 0x38];
            } else {
                msg = RECEIVE_ITEM_MESSAGES[item];
            }
        } else if self.ancilla_slot_view(k).step() != 2 {
            if self.ancilla_slot_view(k).item_to_link() == 0x17 {
                msg = RECEIVE_ITEM_HEART_PIECE_MESSAGES
                    [self.player_resources_view().heart_pieces() as usize];
            } else {
                msg = RECEIVE_ITEM_MESSAGES[item];
            }
        }
        if msg != -1 {
            self.dialogue_message_index_view_mut().set_value(msg as u16);
            if msg == 0x70 {
                self.system_signals_view_mut().set_ambient_sound_effect(9);
            }
            self.main_show_text_message();
        }
    }

    fn ancilla22_item_receipt_move_label_b(&mut self, k: usize) {
        if self.ancilla_slot_view(k).aux_timer() >= 24 {
            let a = self.ancilla_slot_view(k).y_velocity().wrapping_sub(1);
            if a >= 248 {
                let value = a;
                self.ancilla_slot_view_mut(k).set_y_velocity(value);
            }
            self.ancilla_move_y(k);
        }
    }

    fn ancilla22_item_receipt_draw_and_update(&mut self, k: usize) {
        if self.ancilla_slot_view(k).item_to_link() == 0x20 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_z(value);
            self.ancilla_add_occasional_sparkle(k);
            if self.zelda_read_apui00() == 0 {
                self.system_signals_view_mut().set_music_control(0x1a);
                self.item_receipt_transmute_to_rising_crystal(k);
                return;
            }
        } else if self.ancilla_slot_view(k).item_to_link() == 1 {
            let value = RECEIVE_ITEM_CRYSTAL_FRAME_SEQUENCE[0];
            self.ancilla_slot_view_mut(k).set_work_byte_4(value);
            if self.ancilla_slot_view(k).step() != 2 {
                if self.ancilla_slot_view(k).timer() < 16 {
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_work_byte_1(value);
                    let value = RECEIVE_ITEM_CRYSTAL_FRAME_SEQUENCE[0];
                    self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                } else {
                    self.ancilla_slot_view_mut(k).tick_work_byte_3();
                    if sign8(self.ancilla_slot_view(k).work_byte_3()) {
                        let value = 2;
                        self.ancilla_slot_view_mut(k).set_work_byte_3(value);
                        let mut a = self.ancilla_slot_view(k).work_byte_1().wrapping_add(1);
                        if a == 3 {
                            a = 0;
                        }
                        let value = a;
                        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
                        let value = RECEIVE_ITEM_CRYSTAL_FRAME_SEQUENCE[a as usize];
                        self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                    }
                }
            }
        }

        if self.ancilla_slot_view(k).item_to_link() == 0x34
            || self.ancilla_slot_view(k).item_to_link() == 0x35
            || self.ancilla_slot_view(k).item_to_link() == 0x36
        {
            self.ancilla_slot_view_mut(k).tick_work_byte_3();
            if sign8(self.ancilla_slot_view(k).work_byte_3()) {
                let mut a = self.ancilla_slot_view(k).work_byte_1().wrapping_add(1);
                if a == 3 {
                    a = 0;
                }
                let value = a;
                self.ancilla_slot_view_mut(k).set_work_byte_1(value);
                let value = RECEIVE_ITEM_MILESTONE_FRAME_TIMERS[a as usize];
                self.ancilla_slot_view_mut(k).set_work_byte_3(value);
                self.WriteTo4BPPBuffer_at_7F4000(RECEIVE_ITEM_MILESTONE_GFX_SOURCES[a as usize]);
            }
        }
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.ancilla_receive_item_draw(k, x, y);
    }

    fn ancilla_rising_crystal(&mut self, k: usize) {
        const DUNGEON_CRYSTAL_PENDANT_BIT: [u8; 13] = [0, 0, 4, 2, 0, 16, 2, 1, 64, 4, 1, 32, 8];

        let value = 0;

        self.ancilla_slot_view_mut(k).set_z(value);
        self.ancilla_add_occasional_sparkle(k);
        let mut yy = self.ancilla_slot_view(k).y_velocity().wrapping_sub(1);
        if yy < 0xf0 {
            yy = 0xf0;
        }
        let value = yy;
        self.ancilla_slot_view_mut(k).set_y_velocity(value);
        self.ancilla_move_y(k);

        let y = self
            .ancilla_get_y(k)
            .wrapping_sub(self.ppu_scroll_copy_view().bg2_v_copy());
        if y < 0x49 {
            self.ancilla_set_y(
                k,
                0x49u16.wrapping_add(self.ppu_scroll_copy_view().bg2_v_copy()),
            );
            if self.frame_state().submodule == 0 {
                let i = (self.save_progress_view().palace_index_x2() >> 1) as usize;
                self.player_resources_view_mut()
                    .add_crystal_flags(DUNGEON_CRYSTAL_PENDANT_BIT[i]);
                self.set_submodule(0x18);
                self.set_subsubmodule(0);
                self.palette_buffer_view_mut()
                    .clear_aux_visible_subpalettes();
                self.palette_filter_view_mut().set_countdown_word(0);
                self.palette_filter_view_mut()
                    .set_darkening_or_lightening_screen_word(0);
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.ancilla_receive_item_draw(k, x, y);
    }

    fn ancilla29_milestone_item_receipt(&mut self, k: usize) {
        if self.ancilla_slot_view(k).item_to_link() != 0x10
            && self.ancilla_slot_view(k).item_to_link() != 0x0f
        {
            let dung_savegame_state_bits = self.dungeon_savegame_state().savegame_state_bits();
            if dung_savegame_state_bits & 0x4000 != 0 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            }

            if dung_savegame_state_bits & 0x8000 == 0 {
                return;
            }

            if self.world_transient().milestone_item_gfx_swap_countdown() != 0 {
                if self.world_transient().milestone_item_gfx_swap_countdown() == 1 {
                    if self.ancilla_slot_view(k).item_to_link() == 0x20 {
                        self.system_signals_view_mut()
                            .set_ambient_sound_effect(0x0f);
                        self.DecodeAnimatedSpriteTile_variable(0x28);
                    } else {
                        self.DecodeAnimatedSpriteTile_variable(0x23);
                    }
                }
                self.world_transient_mut()
                    .decrement_milestone_item_gfx_swap_countdown();
                return;
            }
            if self.ancilla_slot_view(k).work_byte_3() == 0
                && self.ancilla_slot_view(k).item_to_link() == 0x20
            {
                let value = 1;
                self.ancilla_slot_view_mut(k).set_work_byte_3(value);
                self.palette_buffer_view_mut().set_sp6r_indoors(4);
                self.palette_buffer_view_mut()
                    .select_overworld_aux_palette_offset();
                self.Palette_Load_SpriteEnvironment_Dungeon();
                self.system_signals_view_mut().increment_cgram_update_flag();
            }
        } else if self.ancilla_slot_view(k).g() != 0 {
            self.ancilla_slot_view_mut(k).subtract_g(1);
            return;
        }

        if self.ancilla_slot_view(k).item_to_link() == 0x20 {
            self.ancilla_add_occasional_sparkle(k);
        }

        if self.frame_state().submodule == 0 {
            if self.ancilla_slot_view(k).z() < 24
                && self.ancilla_check_link_collision(k, 2)
                && !self.player_state_view().has_hookshot_interlock()
                && !self.player_state_view().has_auxiliary_state()
            {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                if self.player_state_view().handler_state() == 25
                    || self.player_state_view().handler_state() == 26
                {
                    self.player_state_view_mut().clear_custom_spell_animation();
                    self.player_state_view_mut().clear_force_hold_sword_up();
                    self.player_state_view_mut().clear_handler_state();
                }
                self.player_state_view_mut().set_item_receipt_method(3);
                self.link_receive_item(self.ancilla_slot_view(k).item_to_link(), 0);
                return;
            }

            if self.ancilla_slot_view(k).step() != 2 {
                if self.ancilla_slot_view(k).step() != 0 {
                    self.ancilla_slot_view_mut(k).tick_z_velocity();
                }
                self.ancilla_move_z(k);
                if self.ancilla_slot_view(k).z() >= 0xf8 {
                    self.ancilla_slot_view_mut(k).add_step(1);
                    let value = 0x18;
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_z(value);
                }
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let oam = self.ancilla_receive_item_draw(
            k,
            x,
            y.wrapping_sub(self.ancilla_slot_view(k).z() as u16),
        );

        let aux_timer = self.ancilla_slot_view(k).aux_timer().wrapping_sub(1);
        let value = aux_timer;
        self.ancilla_slot_view_mut(k).set_aux_timer(value);
        if sign8(aux_timer) {
            let value = 9;
            self.ancilla_slot_view_mut(k).set_aux_timer(value);
            self.ancilla_slot_view_mut(k).add_l(1);
            if self.ancilla_slot_view(k).l() == 3 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_l(value);
            }
        }

        let t = if self.ancilla_slot_view(k).z() == 0 {
            if self.world_location_state().dungeon_room == 6 {
                self.ancilla_slot_view(k).l().wrapping_add(4)
            } else {
                0
            }
        } else if self.ancilla_slot_view(k).z() < 0x20 {
            1
        } else {
            2
        };
        self.ancilla_draw_shadow(oam, t as usize, x, y.wrapping_add(12), 0x20);
    }

    fn ancilla28_wish_pond_item(&mut self, k: usize) {
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 0x10);

        if self.frame_state().submodule == 0 && self.ancilla_slot_view(k).timer() == 0 {
            let mut player = self.player_state_view_mut();
            player.set_picking_throw_state(2);
            player.clear_state_bits();
            self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
            self.ancilla_move_z(k);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            if sign8(self.ancilla_slot_view(k).z()) && self.ancilla_slot_view(k).z() < 228 {
                let value = 228;
                self.ancilla_slot_view_mut(k).set_z(value);
                let j = self.ancilla_slot_view(k).item_to_link() as usize;
                self.ancilla_set_xy(
                    k,
                    self.ancilla_get_x(k)
                        .wrapping_add(if self.asset_u8(71, j) != 0 { 8 } else { 4 }),
                    self.ancilla_get_y(k).wrapping_add(18),
                );
                self.ancilla_transmute_to_splash(k);
                return;
            }
        }
        self.wish_pond_item_draw(k);
    }

    fn ancilla42_happiness_pond_rupees(&mut self, k: usize) {
        let mut player = self.player_state_view_mut();
        player.set_picking_throw_state(2);
        player.clear_state_bits();
        for i in (0..=9).rev() {
            if self.happiness_pond_rupee_view(i).is_active() {
                self.hapiness_pond_rupees_execute_rupee(k, i);
                if self.happiness_pond_rupee_view(i).step() == 2 {
                    self.happiness_pond_rupee_view_mut(i).clear();
                }
            }
        }
        for i in (0..=9).rev() {
            if self.happiness_pond_rupee_view(i).is_active() {
                return;
            }
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
    }

    fn hapiness_pond_rupees_execute_rupee(&mut self, k: usize, i: usize) {
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 0x10);
        self.hapiness_pond_rupees_get_state(k, i);

        if self.ancilla_slot_view(k).step() != 0 {
            if self.frame_state().submodule == 0 && self.ancilla_slot_view(k).timer() == 0 {
                let value = 6;
                self.ancilla_slot_view_mut(k).set_timer(value);
                let value = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                if self.ancilla_slot_view(k).item_to_link() == 5 {
                    self.ancilla_slot_view_mut(k).add_step(1);
                } else {
                    self.object_splash_draw(k);
                }
            } else {
                self.object_splash_draw(k);
            }
        } else if self.frame_state().submodule == 0 && self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            self.ancilla_move_z(k);
            if sign8(self.ancilla_slot_view(k).z()) && self.ancilla_slot_view(k).z() < 0xe4 {
                let value = 0xe4;
                self.ancilla_slot_view_mut(k).set_z(value);
                self.ancilla_set_xy(
                    k,
                    self.ancilla_get_x(k).wrapping_sub(4),
                    self.ancilla_get_y(k).wrapping_add(30),
                );
                let value = 0;
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                let value = 6;
                self.ancilla_slot_view_mut(k).set_timer(value);
                self.ancilla_sfx2_pan(k, 0x28);
                self.ancilla_slot_view_mut(k).add_step(1);
                self.object_splash_draw(k);
            } else {
                let value = 2;
                self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_floor(value);
                self.wish_pond_item_draw(k);
            }
        } else {
            let value = 2;
            self.ancilla_slot_view_mut(k).set_work_byte_4(value);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_floor(value);
            self.wish_pond_item_draw(k);
        }
        self.hapiness_pond_rupees_save_state(i, k);
    }

    fn hapiness_pond_rupees_get_state(&mut self, j: usize, k: usize) {
        let pond = self.happiness_pond_rupee_view(k).snapshot();
        let mut ancilla = self.ancilla_slot_view_mut(j);
        ancilla.set_y_low(pond.y_low);
        ancilla.set_y_high(pond.y_high);
        ancilla.set_x_low(pond.x_low);
        ancilla.set_x_high(pond.x_high);
        ancilla.set_z(pond.z);
        ancilla.set_y_velocity(pond.y_velocity);
        ancilla.set_x_velocity(pond.x_velocity);
        ancilla.set_z_velocity(pond.z_velocity);
        ancilla.set_y_subpixel(pond.y_subpixel);
        ancilla.set_x_subpixel(pond.x_subpixel);
        ancilla.set_z_subpixel(pond.z_subpixel);
        ancilla.set_item_to_link(pond.item_to_link);
        ancilla.set_step(pond.step);
        ancilla.set_timer(pond.timer);
    }

    fn hapiness_pond_rupees_save_state(&mut self, k: usize, j: usize) {
        let ancilla = self.ancilla_slot_view(j);
        let snapshot = crate::game_state::HappinessPondRupeeState {
            y_low: ancilla.y_low(),
            y_high: ancilla.y_high(),
            x_low: ancilla.x_low(),
            x_high: ancilla.x_high(),
            z: ancilla.z(),
            y_velocity: ancilla.y_velocity(),
            x_velocity: ancilla.x_velocity(),
            z_velocity: ancilla.z_velocity(),
            y_subpixel: ancilla.y_subpixel(),
            x_subpixel: ancilla.x_subpixel(),
            z_subpixel: ancilla.z_subpixel(),
            item_to_link: ancilla.item_to_link(),
            timer: ancilla.timer(),
            step: ancilla.step(),
        };
        self.happiness_pond_rupee_view_mut(k)
            .store_snapshot(snapshot);
    }

    fn ancilla3_c_spin_attack_charge_sparkle(&mut self, k: usize) {
        const SWORD_CHARGE_SPARK_CHAR: [u8; 3] = [0xb7, 0x80, 0x83];
        const SWORD_CHARGE_SPARK_FLAGS: [u8; 3] = [4, 4, 0x84];

        if self.frame_state().submodule == 0 && self.ancilla_slot_view(k).timer() == 0 {
            let value = 4;
            self.ancilla_slot_view_mut(k).set_timer(value);
            self.ancilla_slot_view_mut(k).add_item_to_link(1);
            if self.ancilla_slot_view(k).item_to_link() == 3 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            }
        }
        let value = self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 4) as u8;
        self.ancilla_slot_view_mut(k).set_oam_index(value);
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let j = self.ancilla_slot_view(k).item_to_link() as usize;
        self.ancilla_set_oam(
            self.oam_state_view().current_pointer_usize(),
            x,
            y,
            SWORD_CHARGE_SPARK_CHAR[j],
            SWORD_CHARGE_SPARK_FLAGS[j] | self.oam_state_view().priority_high(),
            0,
        );
    }

    fn ancilla2_e_somaria_block_fission(&mut self, k: usize) {
        const SOMARIAN_BLOCK_DIVIDE_X: [i8; 16] =
            [-8, 0, -8, 0, -10, -10, 2, 2, -8, 0, -8, 0, -12, -12, 4, 4];
        const SOMARIAN_BLOCK_DIVIDE_Y: [i8; 16] =
            [-10, -10, 2, 2, -8, 0, -8, 0, -12, -12, 4, 4, -8, 0, -8, 0];
        const SOMARIAN_BLOCK_DIVIDE_CHAR: [u8; 16] = [
            0xc6, 0xc6, 0xc6, 0xc6, 0xc4, 0xc4, 0xc4, 0xc4, 0xd2, 0xd2, 0xd2, 0xd2, 0xc5, 0xc5,
            0xc5, 0xc5,
        ];
        const SOMARIAN_BLOCK_DIVIDE_FLAGS: [u8; 16] = [
            0xc6, 0x86, 0x46, 6, 0x46, 0xc6, 6, 0x86, 0xc6, 0x86, 0x46, 6, 0x46, 0xc6, 6, 0x86,
        ];

        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if (aux_timer as i8) < 0 {
            let item_to_link = {
                let mut fission = self.ancilla_slot_view_mut(k);
                fission.set_aux_timer(3);
                fission.advance_item_to_link()
            };
            if item_to_link == 2 {
                self.ancilla_slot_view_mut(k).clear();
                self.somaria_block_spawn_bullets(k);
                return;
            }
        }
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();

        let z = self.ancilla_slot_view(k).z().wrapping_add(
            if self.ancilla_slot_view(k).k() == 3 && self.player_state_view().z() as u8 != 0xff {
                self.player_state_view().z() as u8
            } else {
                0
            },
        );
        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 8;
        for _ in 0..8 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(SOMARIAN_BLOCK_DIVIDE_X[j] as i16 as u16),
                y.wrapping_add(SOMARIAN_BLOCK_DIVIDE_Y[j] as i16 as u16)
                    .wrapping_sub(z as i8 as i16 as u16),
                SOMARIAN_BLOCK_DIVIDE_CHAR[j],
                SOMARIAN_BLOCK_DIVIDE_FLAGS[j] & !0x30 | self.oam_state_view().priority_high(),
                0,
            );
            j += 1;
            oam += 4;
        }
    }

    fn ancilla2_f_lamp_flame(&mut self, k: usize) {
        const LAMP_FLAME_DRAW_CHAR: [u8; 12] = [
            0x9c, 0x9c, 0xff, 0xff, 0xa4, 0xa5, 0xb2, 0xb3, 0xe3, 0xf3, 0xff, 0xff,
        ];
        const LAMP_FLAME_DRAW_Y: [i8; 12] = [-3, 0, 0, 0, 0, 0, 8, 8, 0, 8, 0, 0];
        const LAMP_FLAME_DRAW_X: [i8; 12] = [4, 10, 0, 0, 1, 9, 2, 7, 4, 4, 0, 0];

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        if self.ancilla_slot_view(k).timer() == 0 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            return;
        }
        let mut j = ((self.ancilla_slot_view(k).timer() & 0xf8) >> 1) as usize;
        loop {
            if LAMP_FLAME_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(LAMP_FLAME_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(LAMP_FLAME_DRAW_Y[j] as i16 as u16),
                    LAMP_FLAME_DRAW_CHAR[j],
                    self.oam_state_view().priority_high() | 2,
                    0,
                );
                oam += 4;
            }
            j += 1;
            if j & 3 == 0 {
                break;
            }
        }
    }

    fn ancilla41_waterfall_splash(&mut self, k: usize) {
        const WATERFALL_SPLASH_X: [i8; 8] = [0, 0, -4, 4, -7, 7, -9, 17];
        const WATERFALL_SPLASH_Y: [i8; 8] = [-4, 0, -5, -5, -3, -3, 12, 12];
        const WATERFALL_SPLASH_CHAR: [u8; 8] = [0xc0, 0xff, 0xac, 0xac, 0xae, 0xae, 0xbf, 0xbf];
        const WATERFALL_SPLASH_FLAGS: [u8; 8] = [0x84, 0xff, 0x84, 0xc4, 0x84, 0xc4, 0x84, 0xc4];
        const WATERFALL_SPLASH_EXT: [u8; 8] = [2, 0xff, 2, 2, 2, 2, 0, 0];

        if !self.ancilla_check_for_entrance_trigger(if self.world_location_state().is_indoors() {
            0
        } else {
            1
        }) {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            return;
        }

        if self.frame_state().submodule == 0 && self.frame_state().frame_counter & 7 == 0 {
            self.ancilla_sfx2_near(0x1c);
        }

        self.player_state_view_mut()
            .set_water_ripple_or_grass_state(1);
        self.player_state_view_mut()
            .subtract_animation_step_if_at_least(6, 6);

        if self.ancilla_slot_view(k).timer() == 0 {
            let mut splash = self.ancilla_slot_view_mut(k);
            splash.set_timer(2);
            let item_to_link = splash.advance_item_to_link();
            splash.set_item_to_link(item_to_link & 3);
        }

        if self.world_location_state().is_indoors() && self.player_state_view().y_low() < 0x38 {
            self.ancilla_set_y(k, 0x0d38);
        } else {
            self.ancilla_set_y(k, self.player_state_view().y());
        }
        self.ancilla_set_x(k, self.player_state_view().x());

        let (x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let z = self.player_state_view().z() as u8;
        y = y.wrapping_sub(if sign8(z) { 0 } else { z } as u16);

        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 2;
        for _ in 0..2 {
            if WATERFALL_SPLASH_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(WATERFALL_SPLASH_X[j] as i16 as u16),
                    y.wrapping_add(WATERFALL_SPLASH_Y[j] as i16 as u16),
                    WATERFALL_SPLASH_CHAR[j],
                    WATERFALL_SPLASH_FLAGS[j] | 0x30,
                    WATERFALL_SPLASH_EXT[j],
                );
            }
            j += 1;
            oam += 4;
        }
    }

    fn ancilla3_d_item_splash(&mut self, k: usize) {
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 8);
        if self.frame_state().submodule == 0 && self.ancilla_slot_view(k).timer() == 0 {
            let mut splash = self.ancilla_slot_view_mut(k);
            splash.set_timer(6);
            if splash.advance_item_to_link() == 5 {
                splash.clear();
                return;
            }
        }
        self.object_splash_draw(k);
    }

    fn ancilla15_jump_splash(&mut self, k: usize) {
        const ANCILLA_JUMP_SPLASH_CHAR: [u8; 2] = [0xac, 0xae];

        if self.frame_state().submodule == 0 {
            let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(aux_timer) {
                let mut splash = self.ancilla_slot_view_mut(k);
                splash.set_aux_timer(0);
                splash.set_item_to_link(1);
            }
            if self.ancilla_slot_view(k).item_to_link() != 0 {
                let y_velocity = self.ancilla_slot_view(k).y_velocity().wrapping_sub(4);
                {
                    let mut splash = self.ancilla_slot_view_mut(k);
                    splash.set_y_velocity(y_velocity);
                    splash.set_x_velocity(y_velocity);
                }
                if y_velocity < 232 {
                    self.ancilla_slot_view_mut(k).clear();
                    if (self.player_state_view().is_bunny_mirror()
                        || self.player_state_view().handler_state() == 4)
                        && self.player_state_view().is_in_deep_water()
                    {
                        self.check_ability_to_swim();
                    }
                    return;
                }
                self.ancilla_move_x(k);
                self.ancilla_move_y(k);
            }
        }

        let (mut x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let ax = self.ancilla_get_x(k);
        let x8 = self
            .player_state_view()
            .x()
            .wrapping_mul(2)
            .wrapping_sub(ax)
            .wrapping_sub(self.world_scroll().bg2_x());
        let x6 = ax
            .wrapping_add(12)
            .wrapping_sub(self.world_scroll().bg2_x());
        let j = self.ancilla_slot_view(k).item_to_link() as usize;
        let mut flags = 0;
        for _ in 0..2 {
            self.ancilla_set_oam(oam, x, y, ANCILLA_JUMP_SPLASH_CHAR[j], 0x24 | flags, 2);
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
            x = x8;
            flags = 0x40;
        }
        self.ancilla_set_oam(oam, x6, y, 0xc0, 0x24, if j == 1 { 1 } else { 2 });
    }

    fn ancilla04_beam_hit(&mut self, k: usize) {
        const BEAM_HIT_X: [i8; 16] = [-12, 20, -12, 20, -8, 16, -8, 16, -4, 12, -4, 12, 0, 8, 0, 8];
        const BEAM_HIT_Y: [i8; 16] = [-12, -12, 20, 20, -8, -8, 16, 16, -4, -4, 12, 12, 0, 0, 8, 8];
        const BEAM_HIT_CHAR: [u8; 16] = [
            0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x54, 0x54,
            0x54, 0x54,
        ];
        const BEAM_HIT_FLAGS: [u8; 16] = [
            0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0, 0x40, 0x80, 0xc0,
        ];

        let Some(info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };
        if self.ancilla_slot_view(k).timer() == 0 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            return;
        }

        let mut oam = self.oam_state_view().current_pointer_usize();
        let j = (self.ancilla_slot_view(k).timer() >> 1) as usize;
        let ancilla_x = self.ancilla_get_x(k);
        let ancilla_y = self.ancilla_get_y(k);
        let r7 = ancilla_x.wrapping_sub(self.world_scroll().bg2_x()) as u8;
        let r6 = ancilla_y.wrapping_sub(self.world_scroll().bg2_y()) as u8;
        for i in (0..=3).rev() {
            let m = j * 4 + i;
            let x = info.x.wrapping_add(BEAM_HIT_X[m] as u8);
            let y = info.y.wrapping_add(BEAM_HIT_Y[m] as u8);
            let x_adj = ancilla_x
                .wrapping_add(x.wrapping_sub(r7) as i8 as i16 as u16)
                .wrapping_sub(self.world_scroll().bg2_x());
            let y_adj = ancilla_y
                .wrapping_add(y.wrapping_sub(r6) as i8 as i16 as u16)
                .wrapping_sub(self.world_scroll().bg2_y())
                .wrapping_add(0x10);
            self.oam_state_view_mut().write_entry(
                oam,
                x,
                if y_adj >= 0x100 { 0xf0 } else { y },
                BEAM_HIT_CHAR[m].wrapping_add(0x82),
                BEAM_HIT_FLAGS[m] | 2 | info.flags,
            );
            let value = if x_adj >= 0x100 { 1 } else { 0 };
            self.oam_state_view_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            oam += 4;
        }
    }

    fn ancilla13_ice_rod_sparkle(&mut self, k: usize) {
        const ICE_SHOT_SPARKLE_X: [u8; 16] = [2, 7, 6, 1, 1, 7, 7, 1, 0, 7, 8, 1, 4, 9, 4, 0xff];
        const ICE_SHOT_SPARKLE_Y: [u8; 16] = [2, 3, 8, 7, 1, 1, 7, 7, 1, 0, 7, 8, 0xff, 4, 9, 4];
        const ICE_SHOT_SPARKLE_CHAR: [u8; 16] = [
            0x83, 0x83, 0x83, 0x83, 0xb6, 0x80, 0xb6, 0x80, 0xb7, 0xb6, 0xb7, 0xb6, 0xb7, 0xb6,
            0xb7, 0xb6,
        ];

        if self.ancilla_slot_view(k).timer() == 0 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        }
        if self.frame_state().submodule == 0 {
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

        if self.oam_state_view().has_sprite_sorting() {
            if self.ancilla_slot_view(k).floor() != 0 {
                self.oam_allocate_from_region_e(0x10);
            } else {
                self.oam_allocate_from_region_d(0x10);
            }
        } else {
            self.oam_allocate_from_region_a(0x10);
        }

        let mut oam = self.oam_state_view().current_pointer_usize();
        j = (self.ancilla_slot_view(k).timer() & 0x1c) as i32;
        for i in (0..=3).rev() {
            let n = i + j as usize;
            self.oam_state_view_mut().write_entry(
                oam,
                info.x.wrapping_add(ICE_SHOT_SPARKLE_X[n]),
                info.y.wrapping_add(ICE_SHOT_SPARKLE_Y[n]),
                ICE_SHOT_SPARKLE_CHAR[n],
                info.flags | 4,
            );
            let value = 0;
            self.oam_state_view_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            oam += 4;
        }
    }

    fn ancilla_add_ice_rod_sparkle(&mut self, k: usize) {
        const ICE_SHOT_SPARKLE_XVEL: [i8; 4] = [0, 0, -4, 4];
        const ICE_SHOT_SPARKLE_YVEL: [i8; 4] = [-4, 4, 0, 0];

        if self.frame_state().submodule != 0 {
            return;
        }
        self.ancilla_slot_view_mut(k).subtract_work_byte_4(1);
        if !sign8(self.ancilla_slot_view(k).work_byte_4()) {
            return;
        }

        let value = 5;

        self.ancilla_slot_view_mut(k).set_work_byte_4(value);
        if let Some(j) = self.ancilla_alloc_high() {
            let value = 0x13;
            self.ancilla_slot_view_mut(j).set_ancilla_type(value);
            let value = 15;
            self.ancilla_slot_view_mut(j).set_timer(value);

            let i = self.ancilla_slot_view(k).direction() as usize;
            {
                let mut sparkle = self.ancilla_slot_view_mut(j);
                sparkle.set_x_velocity(ICE_SHOT_SPARKLE_XVEL[i] as u8);
                sparkle.set_y_velocity(ICE_SHOT_SPARKLE_YVEL[i] as u8);
            }

            let value = self.ancilla_slot_view(k).x_low();

            self.ancilla_slot_view_mut(j).set_x_low(value);
            let value = self.ancilla_slot_view(k).y_low();
            self.ancilla_slot_view_mut(j).set_y_low(value);
            let value = self.ancilla_slot_view(k).floor();
            self.ancilla_slot_view_mut(j).set_floor(value);
            let value = 0;
            self.ancilla_slot_view_mut(j).set_num_sprites(value);
        }
    }

    pub(super) fn ancilla_add_simple(&mut self, ty: u8, limit: u8) -> Option<usize> {
        self.ancilla_add_ancilla(ty, limit)
    }

    fn ancilla_add_ancilla(&mut self, a: u8, y: u8) -> Option<usize> {
        let k = self.ancilla_alloc_init(a, y)?;
        let value = a;
        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        let value = self.player_state_view().lower_level_state();
        self.ancilla_slot_view_mut(k).set_floor(value);
        let value = self.player_state_view().lower_level_mirror_state();
        self.ancilla_slot_view_mut(k).set_floor2(value);
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_y_velocity(0);
            ancilla.set_x_velocity(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_object_priority(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_u(value);
        let value = ANCILLA_DRAW_SPRITE_COUNTS[a as usize];
        self.ancilla_slot_view_mut(k).set_num_sprites(value);
        Some(k)
    }

    fn ancilla_alloc_high(&self) -> Option<usize> {
        (0..=9)
            .rev()
            .find(|&k| self.ancilla_slot_view(k).ancilla_type() == 0)
    }

    pub(super) fn ancilla_alloc_init(&mut self, ty: u8, limit: u8) -> Option<usize> {
        if self.system_signals_view().bugs_fixed() >= BUGFIX_POLY_RENDERER {
            self.tile_detect_position_view_mut()
                .set_collision_bits(u16::from(limit.wrapping_add(1)));
        }

        let n = (0..5)
            .filter(|&i| self.ancilla_slot_view(i).ancilla_type() == ty)
            .count();
        if limit as usize + 1 == n {
            return None;
        }

        let start = if ty == 7 || ty == 8 {
            limit as usize
        } else {
            4
        };
        for j in (0..=start).rev() {
            if self.ancilla_slot_view(j).ancilla_type() == 0 {
                return Some(j);
            }
        }

        let mut k = self.sprite_system_view().ancilla_alloc_rotate() as i8;
        loop {
            k -= 1;
            if k < 0 {
                k = limit as i8;
            }
            let old_type = self.ancilla_slot_view(k as usize).ancilla_type();
            if old_type == 0x3c || old_type == 0x13 || old_type == 0x0a {
                self.sprite_system_view_mut()
                    .set_ancilla_alloc_rotate(k as u8);
                return Some(k as usize);
            }
            if k == 0 {
                break;
            }
        }
        self.sprite_system_view_mut().clear_ancilla_alloc_rotate();
        None
    }

    fn ancilla_add_add_ancilla_bank08(&mut self, ty: u8, y: u8) -> Option<usize> {
        self.ancilla_add_simple(ty, y)
    }

    pub(super) fn ancilla_check_link_collision(&self, k: usize, j: usize) -> bool {
        self.ancilla_check_link_collision_out(k, j).is_some()
    }

    fn ancilla_check_link_collision_out(&self, k: usize, j: usize) -> Option<CheckPlayerCollOut> {
        const YOFFS: [u16; 5] = [0, 8, 8, 8, 0];
        const XOFFS: [u16; 5] = [0, 8, 8, 8, 0];
        const H: [u16; 5] = [20, 20, 8, 28, 14];
        const W: [u16; 5] = [20, 3, 8, 24, 14];
        const LINK_YOFFS: [u16; 5] = [12, 12, 12, 12, 12];
        const LINK_XOFFS: [u16; 5] = [8, 8, 8, 12, 8];

        let y = self
            .ancilla_y(k)
            .wrapping_add(YOFFS[j])
            .wrapping_add(self.ancilla_slot_view(k).z() as i8 as i16 as u16);
        let x = self.ancilla_x(k).wrapping_add(XOFFS[j]);
        let r4 = self
            .player_state_view()
            .y()
            .wrapping_add(LINK_YOFFS[j])
            .wrapping_sub(y);
        let r6 = self
            .player_state_view()
            .x()
            .wrapping_add(LINK_XOFFS[j])
            .wrapping_sub(x);
        let r8 = abs16(r4);
        let r10 = abs16(r6);
        if r8 < H[j] && r10 < W[j] {
            Some(CheckPlayerCollOut { r4, r6, r8, r10 })
        } else {
            None
        }
    }

    fn ancilla_check_tile_collision(&mut self, k: usize) -> u8 {
        if self.world_location_state().is_outdoors()
            && self.ancilla_slot_view(k).object_priority() != 0
        {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_tile_attribute(value);
            return 0;
        }
        if self.dungeon_state_view().header_collision() == 0 {
            return self.ancilla_check_tile_collision_one_floor(k) as u8;
        }

        let mut x = 0u16;
        let mut y = 0u16;
        if self.dungeon_state_view().header_collision() < 3 {
            x = self
                .world_scroll()
                .bg1_x()
                .wrapping_sub(self.world_scroll().bg2_x());
            y = self
                .world_scroll()
                .bg1_y()
                .wrapping_sub(self.world_scroll().bg2_y());
        }
        let oldx = self.ancilla_get_x(k);
        let oldy = self.ancilla_get_y(k);
        self.ancilla_set_xy(k, oldx.wrapping_add(x), oldy.wrapping_add(y));
        let value = 1;
        self.ancilla_slot_view_mut(k).set_floor(value);
        let b = self.ancilla_check_tile_collision_one_floor(k) as u8;
        let value = 0;
        self.ancilla_slot_view_mut(k).set_floor(value);
        self.ancilla_set_xy(k, oldx, oldy);
        (b << 1) | self.ancilla_check_tile_collision_one_floor(k) as u8
    }

    fn ancilla_check_tile_collision_staggered(&mut self, k: usize) -> u8 {
        if (self.frame_state().frame_counter ^ k as u8) & 1 != 0 {
            self.ancilla_check_tile_collision(k)
        } else {
            0
        }
    }

    fn ancilla_check_tile_collision_one_floor(&mut self, k: usize) -> bool {
        const CHECK_TILE_COLL0_X: [i8; 20] = [
            8, 8, 0, 16, 4, 4, 0, 16, 4, 4, 4, 12, 12, 12, 4, 12, 0, 0, 0, 0,
        ];
        const CHECK_TILE_COLL0_Y: [i8; 20] = [
            0, 16, 5, 5, 0, 16, 4, 4, 4, 12, 5, 5, 4, 12, 12, 12, 0, 0, 0, 0,
        ];
        let j = self.ancilla_slot_view(k).direction() as usize;
        let x = self
            .ancilla_get_x(k)
            .wrapping_add(CHECK_TILE_COLL0_X[j] as i16 as u16);
        let y = self
            .ancilla_get_y(k)
            .wrapping_add(CHECK_TILE_COLL0_Y[j] as i16 as u16);
        self.ancilla_check_tile_collision_targeted(k, x, y)
    }

    fn ancilla_check_initial_tile_a(&mut self, k: usize) -> i32 {
        const YOFFS_HB: [i8; 12] = [8, 0, -8, 8, 16, 24, 8, 8, 8, 8, 8, 8];
        const XOFFS_HB: [i8; 12] = [0, 0, 0, 0, 0, 0, 0, -8, -16, 0, 8, 16];

        let mut j = self.ancilla_slot_view(k).direction() as usize * 3;
        let mut i = 2i32;
        loop {
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(XOFFS_HB[j] as i16 as u16);
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(YOFFS_HB[j] as i16 as u16);
            self.ancilla_set_xy(k, x, y);
            if self.ancilla_check_tile_collision(k) != 0 {
                break;
            }
            i -= 1;
            if i < 0 {
                break;
            }
            j += 1;
        }
        i
    }

    fn ancilla_return_if_outside_bounds(&mut self, k: usize) -> Option<AncillaOamInfo> {
        const ANCILLA_FLOOR_FLAGS: [u8; 2] = [0x20, 0x10];
        let info = AncillaOamInfo {
            x: self
                .ancilla_slot_view(k)
                .x_low()
                .wrapping_sub(self.world_scroll().bg2_x_low()),
            y: self
                .ancilla_slot_view(k)
                .y_low()
                .wrapping_sub(self.world_scroll().bg2_y_low()),
            flags: ANCILLA_FLOOR_FLAGS[self.ancilla_slot_view(k).floor() as usize],
        };
        if info.x >= 0xf4 || info.y >= 0xf0 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            None
        } else {
            Some(info)
        }
    }

    fn ancilla_apply_conveyor(&mut self, k: usize) {
        const ANCILLA_BELT_XVEL: [i8; 4] = [0, 0, -8, 8];
        const ANCILLA_BELT_YVEL: [i8; 4] = [-8, 8, 0, 0];
        let j = self
            .ancilla_slot_view(k)
            .tile_attribute()
            .wrapping_sub(0x68) as usize;
        let value = ANCILLA_BELT_YVEL[j] as u8;
        self.ancilla_slot_view_mut(k).set_y_velocity(value);
        let value = ANCILLA_BELT_XVEL[j] as u8;
        self.ancilla_slot_view_mut(k).set_x_velocity(value);
        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
    }

    fn ancilla_project_speed_towards_player(&self, k: usize, mut vel: u8) -> ProjectSpeedRet {
        if vel == 0 {
            return ProjectSpeedRet {
                x: 0,
                y: 0,
                xdiff: 0,
                ydiff: 0,
            };
        }
        let below = self.ancilla_is_below_link(k);
        let mut r12 = if (below.b as i8).is_negative() {
            0u8.wrapping_sub(below.b)
        } else {
            below.b
        };

        let right = self.ancilla_is_right_of_link(k);
        let mut r13 = if (right.b as i8).is_negative() {
            0u8.wrapping_sub(right.b)
        } else {
            right.b
        };
        let mut swapped = false;
        if r13 < r12 {
            swapped = true;
            std::mem::swap(&mut r12, &mut r13);
        }
        let mut xvel = vel;
        let mut yvel = 0u8;
        let mut t = 0u8;
        loop {
            t = t.wrapping_add(r12);
            if t >= r13 {
                t = t.wrapping_sub(r13);
                yvel = yvel.wrapping_add(1);
            }
            vel = vel.wrapping_sub(1);
            if vel == 0 {
                break;
            }
        }
        if swapped {
            std::mem::swap(&mut xvel, &mut yvel);
        }
        ProjectSpeedRet {
            x: if right.a != 0 {
                0u8.wrapping_sub(xvel)
            } else {
                xvel
            },
            y: if below.a != 0 {
                0u8.wrapping_sub(yvel)
            } else {
                yvel
            },
            xdiff: right.b,
            ydiff: below.b,
        }
    }

    fn ancilla_get_radial_projection(&self, a: u8, r8: u8) -> AncillaRadialProjection {
        const RADIAL_PROJECTION_PRIMARY_MAGNITUDE: [u8; 64] = [
            255, 254, 251, 244, 236, 225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49,
            74, 97, 120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254, 251, 244, 236,
            225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49, 74, 97, 120, 142, 162,
            181, 197, 212, 225, 236, 244, 251, 254,
        ];
        const RADIAL_PROJECTION_SECONDARY_MAGNITUDE: [u8; 64] = [
            0, 25, 49, 74, 97, 120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254,
            251, 244, 236, 225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49, 74, 97,
            120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254, 251, 244, 236, 225,
            212, 197, 181, 162, 142, 120, 97, 74, 49, 25,
        ];
        const RADIAL_PROJECTION_PRIMARY_SIGN: [u8; 64] = [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        const RADIAL_PROJECTION_SECONDARY_SIGN: [u8; 64] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        let a = a as usize;
        let p0 = u16::from(RADIAL_PROJECTION_PRIMARY_MAGNITUDE[a]) * u16::from(r8);
        let p1 = u16::from(RADIAL_PROJECTION_SECONDARY_MAGNITUDE[a]) * u16::from(r8);
        AncillaRadialProjection {
            r0: ((p0 >> 8) + ((p0 >> 7) & 1)) as u8,
            r2: RADIAL_PROJECTION_PRIMARY_SIGN[a],
            r4: ((p1 >> 8) + ((p1 >> 7) & 1)) as u8,
            r6: RADIAL_PROJECTION_SECONDARY_SIGN[a],
        }
    }

    fn sparkle_prep_oam_from_radial(&self, p: AncillaRadialProjection) -> Point16U {
        Point16U {
            y: self
                .ether_orbit_view()
                .swordbeam_temp_y()
                .wrapping_add(if p.r2 != 0 {
                    -(p.r0 as i16)
                } else {
                    p.r0 as i16
                } as u16)
                .wrapping_sub(4)
                .wrapping_sub(self.world_scroll().bg2_y()),
            x: self
                .ether_orbit_view()
                .swordbeam_temp_x()
                .wrapping_add(if p.r6 != 0 {
                    -(p.r4 as i16)
                } else {
                    p.r4 as i16
                } as u16)
                .wrapping_sub(4)
                .wrapping_sub(self.world_scroll().bg2_x()),
        }
    }

    fn ancilla_is_right_of_link(&self, k: usize) -> PairU8 {
        let x = self
            .player_state_view()
            .x()
            .wrapping_sub(self.ancilla_get_x(k));
        PairU8 {
            a: u8::from((x as i16).is_negative()),
            b: x as u8,
        }
    }

    fn ancilla_is_below_link(&self, k: usize) -> PairU8 {
        let y = self
            .player_state_view()
            .y()
            .wrapping_sub(self.ancilla_get_y(k));
        PairU8 {
            a: u8::from((y as i16).is_negative()),
            b: y as u8,
        }
    }

    fn ancilla_transmute_to_splash(&mut self, k: usize) {
        let value = 0x3d;
        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_item_to_link(value);
        let value = 6;
        self.ancilla_slot_view_mut(k).set_timer(value);
        self.ancilla_set_xy(
            k,
            self.ancilla_get_x(k).wrapping_sub(8),
            self.ancilla_get_y(k).wrapping_add(12),
        );
        self.ancilla_sfx2_pan(k, 0x28);
        self.ancilla3_d_item_splash(k);
    }

    fn object_splash_draw(&mut self, k: usize) {
        const OBJECT_SPLASH_DRAW_X: [i8; 10] = [0, 0, 0, 0, 11, -3, 15, -7, 15, -7];
        const OBJECT_SPLASH_DRAW_Y: [i8; 10] = [0, 0, -6, 0, -13, -8, -17, -4, -17, -4];
        const OBJECT_SPLASH_DRAW_CHAR: [u8; 10] =
            [0xc0, 0xff, 0xe7, 0xff, 0xaf, 0xbf, 0x80, 0x80, 0x83, 0x83];
        const OBJECT_SPLASH_DRAW_FLAGS: [u8; 10] = [0, 0xff, 0, 0xff, 0x40, 0, 0x40, 0, 0xc0, 0x80];
        const OBJECT_SPLASH_DRAW_EXT: [u8; 10] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0];
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 2;
        for _ in 0..2 {
            if OBJECT_SPLASH_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(OBJECT_SPLASH_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(OBJECT_SPLASH_DRAW_Y[j] as i16 as u16),
                    OBJECT_SPLASH_DRAW_CHAR[j],
                    OBJECT_SPLASH_DRAW_FLAGS[j] | 0x24,
                    OBJECT_SPLASH_DRAW_EXT[j],
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn ancilla_handle_lift_logic(&mut self, k: usize) {
        const ANCILLA_LIFTABLE_DELAY: [u8; 3] = [16, 8, 9];

        if self.ancilla_slot_view(k).r() != 0 {
            self.ancilla_handle_lift_logic_label_6(k);
            return;
        }
        if self.ancilla_slot_view(k).l() == 0 {
            if self.player_state_view().ancilla_pickup_flag() == 0 {
                if self.ancilla_handle_lift_logic_clear_pickup_item(k, &ANCILLA_LIFTABLE_DELAY) {
                    return;
                }
            } else {
                if self.player_state_view().ancilla_pickup_flag() != k as u8 + 1 {
                    return;
                }
                if (self.player_state_view().sprite_damage_disable_timer() == 0
                    && self.player_state_view().incapacitated_timer() != 0)
                    || self.player_state_view().player_special_draw_flag() != 0
                    || self.player_state_view().is_in_auxiliary_state(1)
                {
                    let value = 1;
                    self.ancilla_slot_view_mut(k).set_r(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                    self.player_state_view_mut().clear_ancilla_pickup_flag();
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                    self.ancilla_handle_lift_logic_label_6(k);
                    return;
                }
                if !self.player_state_view().is_lifting_or_carrying() {
                    if self.ancilla_handle_lift_logic_clear_pickup_item(k, &ANCILLA_LIFTABLE_DELAY)
                    {
                        return;
                    }
                } else {
                    let mut j = self.ancilla_slot_view(k).k();
                    if !self.player_state_view().picking_throw_state_has(2)
                        && self.player_state_view().ancilla_pickup_flag() != 0
                        && j != 3
                    {
                        if j == 0 && self.ancilla_slot_view(k).aux_timer() == 16 {
                            self.ancilla_sfx2_pan(k, 0x1d);
                        }
                        let value = self.ancilla_slot_view(k).aux_timer().wrapping_sub(1);
                        self.ancilla_slot_view_mut(k).set_aux_timer(value);
                        if (self.ancilla_slot_view(k).aux_timer() as i8).is_negative() {
                            j = j.wrapping_add(1);
                            let value = j;
                            self.ancilla_slot_view_mut(k).set_k(value);
                            let value = if j == 3 {
                                (-2i8) as u8
                            } else {
                                ANCILLA_LIFTABLE_DELAY[j as usize]
                            };
                            self.ancilla_slot_view_mut(k).set_aux_timer(value);
                            if j == 3 {
                                self.ancilla_latch_altitude_above_link(k);
                                return;
                            }
                        }
                        self.ancilla_latch_link_coordinates(k, j as usize);
                        return;
                    }
                    if j != 3 {
                        return;
                    }

                    if !self.player_state_view().picking_throw_state_has(2)
                        && (self.frame_state().submodule != 0
                            || ((self.player_state_view().filtered_joypad_l()
                                | self.player_state_view().filtered_joypad_h())
                                & 0x80)
                                == 0)
                    {
                        if self.ancilla_slot_view(k).item_to_link() != 0 {
                            return;
                        }
                        if self.player_state_view().near_pit_state_at_least(2) {
                            self.player_state_view_mut().set_speed_setting(0);
                            if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize {
                                self.player_state_view_mut().clear_ancilla_pickup_flag();
                                let value = 0;
                                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                            }
                            return;
                        }
                        if !self.player_state_view().is_in_deep_water()
                            && !self.player_state_view().is_bunny_mirror()
                        {
                            self.ancilla_latch_carried_position(k);
                            return;
                        }
                        self.player_state_view_mut().clear_state_bits();
                    }
                    const ANCILLA_LIFTABLE_YVEL: [i8; 4] = [-32, 32, 0, 0];
                    const ANCILLA_LIFTABLE_XVEL: [i8; 4] = [0, 0, -32, 32];
                    let j = self.player_state_view().facing_index();
                    let value = j as u8;
                    self.ancilla_slot_view_mut(k).set_direction(value);
                    {
                        let mut liftable = self.ancilla_slot_view_mut(k);
                        liftable.set_z_velocity(24);
                        liftable.set_y_velocity(ANCILLA_LIFTABLE_YVEL[j] as u8);
                        liftable.set_x_velocity(ANCILLA_LIFTABLE_XVEL[j] as u8);
                    }
                    self.player_state_view_mut().set_picking_throw_state(2);
                    let value = 1;
                    self.ancilla_slot_view_mut(k).set_l(value);
                    self.player_state_view_mut().clear_ancilla_pickup_flag();
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_k(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_object_priority(value);
                    self.ancilla_sfx3_pan(k, 0x13);
                }
            }
        }

        if self.ancilla_slot_view(k).item_to_link() == 0 {
            self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            let old_z = self.ancilla_slot_view(k).z();
            self.ancilla_move_z(k);
            let z = self.ancilla_slot_view(k).z();
            if self.ancilla_slot_view(k).work_byte_4() != 0
                && self.ancilla_slot_view(k).direction() == 1
                && !(z as i8).is_negative()
            {
                self.ancilla_set_y(
                    k,
                    self.ancilla_get_y(k)
                        .wrapping_add(z.wrapping_sub(old_z) as i8 as i16 as u16),
                );
            }
            if !(z as i8).is_negative() || z == 0xff {
                return;
            }
            self.ancilla_slot_view_mut(k).set_z(0);
            self.ancilla_sfx2_pan(k, 0x21);
            self.ancilla_slot_view_mut(k).add_l(1);
            if self.ancilla_slot_view(k).l() != 3 {
                let mut liftable = self.ancilla_slot_view_mut(k);
                let y_velocity = ((liftable.y_velocity() as i8) / 2) as u8;
                let x_velocity = ((liftable.x_velocity() as i8) / 2) as u8;
                liftable.set_y_velocity(y_velocity);
                liftable.set_x_velocity(x_velocity);
                liftable.set_z_velocity(16);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_work_byte_4(value);
            } else {
                self.ancilla_slot_view_mut(k).set_z(0);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_l(value);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                self.player_state_view_mut().set_speed_setting(0);
                {
                    let mut liftable = self.ancilla_slot_view_mut(k);
                    liftable.set_y_velocity(0);
                    liftable.set_x_velocity(0);
                    liftable.set_z_velocity(0);
                }
                if self.ancilla_slot_view(k).t_player() != 0 {
                    let value = self.ancilla_slot_view(k).t_player();
                    self.ancilla_slot_view_mut(k).set_floor(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_t_player(value);
                }
            }
        }
    }

    fn ancilla_handle_lift_logic_clear_pickup_item(
        &mut self,
        k: usize,
        liftable_delay: &[u8; 3],
    ) -> bool {
        self.player_state_view_mut().clear_ancilla_pickup_flag();
        if self.ancilla_slot_view(k).item_to_link() != 0
            || self.player_state_view().has_action_state()
        {
            return true;
        }
        let Some(coll) = self.ancilla_check_link_collision_out(k, 0) else {
            return true;
        };
        if self.ancilla_slot_view(k).floor() != self.player_state_view().lower_level_state() {
            return true;
        }
        if coll.r8 >= 16 || coll.r10 >= 12 {
            let j = if coll.r8 >= coll.r10 {
                if (coll.r4 as i16).is_negative() {
                    1
                } else {
                    0
                }
            } else if (coll.r6 as i16).is_negative() {
                3
            } else {
                2
            };
            if j * 2 != self.player_state_view().facing() {
                return true;
            }
        }
        self.player_state_view_mut()
            .set_ancilla_pickup_flag(k as u8 + 1);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_k(value);
        let value = liftable_delay[0];
        self.ancilla_slot_view_mut(k).set_aux_timer(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        self.ancilla_slot_view_mut(k).set_z(0);
        true
    }

    fn ancilla_handle_lift_logic_label_6(&mut self, k: usize) {
        if self.ancilla_slot_view(k).item_to_link() != 0 {
            return;
        }
        if self.ancilla_slot_view(k).k() == 3 {
            self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
            self.ancilla_move_z(k);
            if self.ancilla_slot_view(k).z() != 0 && self.ancilla_slot_view(k).z() < 252 {
                return;
            }
            let value = 0;
            self.ancilla_slot_view_mut(k).set_z(value);
            self.ancilla_slot_view_mut(k).add_r(1);
            if self.ancilla_slot_view(k).r() != 3 {
                let value = 24;
                self.ancilla_slot_view_mut(k).set_z_velocity(value);
                return;
            }
            let value = 0;
            self.ancilla_slot_view_mut(k).set_k(value);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_r(value);
        self.player_state_view_mut().set_speed_setting(0);
    }

    fn ancilla_latch_altitude_above_link(&mut self, k: usize) {
        let value = 17;
        self.ancilla_slot_view_mut(k).set_z(value);
        self.ancilla_set_y(k, self.ancilla_get_y(k).wrapping_add(17));
        let value = 0;
        self.ancilla_slot_view_mut(k).set_object_priority(value);
    }

    fn ancilla_latch_link_coordinates(&mut self, k: usize, mut j: usize) {
        const ANCILLA_FUNC3_X: [i8; 12] = [8, 8, -4, 20, 8, 8, 8, 8, 8, 8, 8, 8];
        const ANCILLA_FUNC3_Y: [i8; 12] = [16, 8, 4, 4, 8, 2, -1, -1, 2, 2, -1, -1];
        j = j * 4 + self.player_state_view().facing_index();
        self.ancilla_set_xy(
            k,
            self.player_state_view()
                .x()
                .wrapping_add(ANCILLA_FUNC3_X[j] as i16 as u16),
            self.player_state_view()
                .y()
                .wrapping_add(ANCILLA_FUNC3_Y[j] as i16 as u16),
        );
    }

    fn ancilla_latch_carried_position(&mut self, k: usize) {
        const ANCILLA_FUNC2_Y: [i8; 6] = [-2, -1, 0, -2, -1, 0];
        self.player_state_view_mut().set_speed_setting(12);
        let value = self.player_state_view().lower_level_state();
        self.ancilla_slot_view_mut(k).set_floor(value);
        let value = self.player_state_view().lower_level_mirror_state();
        self.ancilla_slot_view_mut(k).set_floor2(value);
        let mut z = self.player_state_view().z();
        if z == 0xffff {
            z = 0;
        }
        self.ancilla_set_xy(
            k,
            self.player_state_view().x().wrapping_add(8),
            self.player_state_view()
                .y()
                .wrapping_sub(z)
                .wrapping_add(18)
                .wrapping_add(
                    ANCILLA_FUNC2_Y[self.player_state_view().animation_step_index()] as i16 as u16,
                ),
        );
    }

    fn ancilla_latch_y_coord_to_z(&mut self, k: usize) -> u16 {
        let y = self.ancilla_get_y(k);
        let z = self.ancilla_slot_view(k).z();
        if self.ancilla_slot_view(k).direction() == 1 && z != 0xff {
            self.ancilla_set_y(k, y.wrapping_sub(z as i8 as i16 as u16));
        }
        y
    }

    pub(super) fn ancilla_check_tile_collision_class2(&mut self, k: usize) -> bool {
        if self.dungeon_state_view().header_collision() == 0 {
            return self.ancilla_check_tile_collision_class2_inner(k);
        }

        let mut x = 0u16;
        let mut y = 0u16;
        if self.dungeon_state_view().header_collision() < 3 {
            x = self
                .world_scroll()
                .bg1_x()
                .wrapping_sub(self.world_scroll().bg2_x());
            y = self
                .world_scroll()
                .bg1_y()
                .wrapping_sub(self.world_scroll().bg2_y());
        }

        let oldx = self.ancilla_x(k);
        let oldy = self.ancilla_y(k);
        self.ancilla_set_xy(k, oldx.wrapping_add(x), oldy.wrapping_add(y));
        let value = 1;
        self.ancilla_slot_view_mut(k).set_floor(value);
        let b = self.ancilla_check_tile_collision_class2_inner(k);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_floor(value);
        self.ancilla_set_xy(k, oldx, oldy);
        b | self.ancilla_check_tile_collision_class2_inner(k)
    }

    fn ancilla_check_tile_collision_class2_inner(&mut self, k: usize) -> bool {
        const Y: [i8; 4] = [-8, 8, 0, 0];
        const X: [i8; 4] = [0, 0, -8, 8];

        let dir = self.ancilla_slot_view(k).direction() as usize;
        let mut x = self.ancilla_x(k).wrapping_add(X[dir] as i16 as u16);
        let y = self.ancilla_y(k).wrapping_add(Y[dir] as i16 as u16);

        if y.wrapping_sub(self.world_scroll().bg2_y()) >= 224
            || x.wrapping_sub(self.world_scroll().bg2_x()) >= 256
        {
            return false;
        }

        let tile_attr = if self.world_location_state().is_outdoors() {
            x >>= 3;
            self.overworld_get_tile_attribute_at_location(x, y)
        } else {
            self.get_tile_attribute_for_ancilla(self.ancilla_slot_view(k).floor(), x, y)
        };

        let value = tile_attr;

        self.ancilla_slot_view_mut(k).set_tile_attribute(value);
        if tile_attr == 3 && self.ancilla_slot_view(k).floor2() != 0 {
            return false;
        }

        match ANCILLA_TILE_COLLISION_ATTRS[tile_attr as usize] {
            0 => false,
            2 => self.entity_check_sloped_tile_collision_for_ancilla(x, y),
            3 => self.ancilla_slot_view(k).floor2() != 0,
            4 => {
                if self.ancilla_slot_view(k).floor2() != 0 {
                    true
                } else {
                    let value = 1;
                    self.ancilla_slot_view_mut(k).set_object_priority(value);
                    false
                }
            }
            _ => true,
        }
    }

    fn ancilla_check_initial_tile_collision_class2(&mut self, k: usize) -> bool {
        const INITIAL_TILE_COLL_Y: [i16; 9] = [15, 16, 28, 24, 12, 12, 12, 12, 8];
        const INITIAL_TILE_COLL_X: [i16; 9] = [8, 8, 8, 8, -1, 0, 17, 16, 0x4b8b];
        let mut j = self.ancilla_slot_view(k).direction() as usize * 2;
        for _ in (0..=2).rev() {
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(INITIAL_TILE_COLL_X[j] as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(INITIAL_TILE_COLL_Y[j] as u16),
            );
            if self.ancilla_check_tile_collision_class2(k) {
                return true;
            }
            j += 1;
        }
        false
    }

    fn ancilla_check_tile_collision_targeted(&mut self, k: usize, mut x: u16, y: u16) -> bool {
        let trace_x = x;
        let trace_y = y;
        if y.wrapping_sub(self.world_scroll().bg2_y()) >= 224
            || x.wrapping_sub(self.world_scroll().bg2_x()) >= 256
        {
            if std::env::var_os("ZELDA3_TRACE_TILE_COLL").is_some()
                && k == 4
                && self.frame_state().frame_counter >= 140
                && self.frame_state().frame_counter <= 150
            {
                eprintln!(
                    "R tile-target fc={} k={} offscreen x={:04x} y={:04x} bg2={:04x}/{:04x} floor={:02x} type={:02x}",
                    self.frame_state().frame_counter,
                    k,
                    trace_x,
                    trace_y,
                    self.world_scroll().bg2_x(),
                    self.world_scroll().bg2_y(),
                    self.ancilla_slot_view(k).floor(),
                    self.ancilla_slot_view(k).ancilla_type(),
                );
            }
            return false;
        }
        let tile_attr = if self.world_location_state().is_outdoors() {
            x >>= 3;
            self.overworld_get_tile_attribute_at_location(x, y)
        } else {
            self.get_tile_attribute_for_ancilla(self.ancilla_slot_view(k).floor(), x, y)
        };

        let value = tile_attr;

        self.ancilla_slot_view_mut(k).set_tile_attribute(value);
        if tile_attr == 3 && self.ancilla_slot_view(k).floor2() != 0 {
            return false;
        }

        let mut t = ANCILLA_TILE_COLLISION_ATTRS_LAYER0[tile_attr as usize];
        if self.ancilla_slot_view(k).ancilla_type() == 2 && tile_attr & 0xf0 == 0xc0 {
            t = 0;
        }
        if std::env::var_os("ZELDA3_TRACE_TILE_COLL").is_some()
            && k == 4
            && self.frame_state().frame_counter >= 140
            && self.frame_state().frame_counter <= 150
        {
            eprintln!(
                "R tile-target fc={} k={} x={:04x} y={:04x} lookup={:04x}/{:04x} floor={:02x} floor2={:02x} obj={:02x} type={:02x} attr={:02x} t={:02x} u={:02x} indoors={:02x} hdr={:02x} bg1={:04x}/{:04x} bg2={:04x}/{:04x}",
                self.frame_state().frame_counter,
                k,
                trace_x,
                trace_y,
                x,
                y,
                self.ancilla_slot_view(k).floor(),
                self.ancilla_slot_view(k).floor2(),
                self.ancilla_slot_view(k).object_priority(),
                self.ancilla_slot_view(k).ancilla_type(),
                tile_attr,
                t,
                self.ancilla_slot_view(k).u(),
                self.world_location_state().indoor_flag,
                self.dungeon_state_view().header_collision(),
                self.world_scroll().bg1_x(),
                self.world_scroll().bg1_y(),
                self.world_scroll().bg2_x(),
                self.world_scroll().bg2_y(),
            );
        }

        if self.ancilla_slot_view(k).object_priority() == 0 {
            if t == 0 {
                return false;
            }
            if t == 1 {
                self.sprite_system_view_mut().set_alert_flag(3);
                return true;
            }
            if t == 2 {
                return self.entity_check_sloped_tile_collision_for_ancilla(x, y);
            }
            if t == 3 {
                if self.ancilla_slot_view(k).floor2() != 0 {
                    self.sprite_system_view_mut().set_alert_flag(3);
                    return true;
                }
                return false;
            }
        }
        self.ancilla_slot_view_mut(k).subtract_u(1);
        if (self.ancilla_slot_view(k).u() as i8) < 0 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_u(value);
            if t == 4 {
                let value = 6;
                self.ancilla_slot_view_mut(k).set_u(value);
                self.ancilla_slot_view_mut(k).xor_object_priority(1);
            }
        }
        false
    }

    fn somaria_block_check_for_transit_tile(&mut self, k: usize) {
        const SOMARIA_TRANSIT_LINE_X: [i8; 12] = [-8, 0, 8, -8, 0, 8, -16, -16, -16, 16, 16, 16];
        const SOMARIA_TRANSIT_LINE_Y: [i8; 12] = [-16, -16, -16, 16, 16, 16, -8, 0, 8, -8, 0, 8];
        if self.player_state_view().somaria_block_bg_check_flag() == 0 {
            return;
        }
        for j in (0..=11).rev() {
            let x = self
                .ancilla_get_x(k)
                .wrapping_add(SOMARIA_TRANSIT_LINE_X[j] as i16 as u16);
            let y = self
                .ancilla_get_y(k)
                .wrapping_add(SOMARIA_TRANSIT_LINE_Y[j] as i16 as u16);
            let bak = self.ancilla_slot_view(k).object_priority();
            self.ancilla_check_tile_collision_targeted(k, x, y);
            let value = bak;
            self.ancilla_slot_view_mut(k).set_object_priority(value);
            if matches!(self.ancilla_slot_view(k).tile_attribute(), 0xb6 | 0xbc) {
                self.ancilla_set_xy(k, x, y);
                self.ancilla_add_somaria_platform_poof(k);
                return;
            }
        }
    }

    fn ancilla_add_somaria_platform_poof(&mut self, k: usize) {
        {
            let mut poof = self.ancilla_slot_view_mut(k);
            poof.set_ancilla_type(0x39);
            poof.set_aux_timer(7);
        }
        for j in (0..=15).rev() {
            if self.sprite_slot_view(j).sprite_type() == 0xed {
                let value = 0;
                self.sprite_slot_view_mut(j).set_state(value);
                self.player_state_view_mut().clear_somaria_platform_state();
            }
        }
        self.player_tile_detect_nearby();
    }

    fn ancilla_add_exploding_somaria_block(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).set_ancilla_type(0x2e);
        let value = ANCILLA_DRAW_SPRITE_COUNTS[0x2e];
        self.ancilla_slot_view_mut(k).set_num_sprites(value);
        {
            let mut block = self.ancilla_slot_view_mut(k);
            block.set_aux_timer(3);
            block.set_step(0);
            block.set_item_to_link(0);
            block.set_work_byte_3(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_r(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_object_priority(value);
        self.dungeon_state_view_mut()
            .clear_somaria_block_switch_counter();
        self.set_sound_effect_2_with_ancilla_pan(k, 1);
    }

    pub(super) fn ancilla_add_charged_spin_attack_sparkle(&mut self) {
        for k in (0..10).rev() {
            if self.ancilla_slot_view(k).ancilla_type() == 0
                || self.ancilla_slot_view(k).ancilla_type() == 0x3c
            {
                self.ancilla_slot_view_mut(k).set_ancilla_type(13);
                let value = self.player_state_view().lower_level_state();
                self.ancilla_slot_view_mut(k).set_floor(value);
                self.ancilla_slot_view_mut(k).set_timer(6);
                break;
            }
        }
    }

    pub(super) fn ancilla_add_sword_swing_sparkle(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return;
        };
        let direction = self.player_state_view().facing() >> 1;
        {
            let mut sparkle = self.ancilla_slot_view_mut(k);
            sparkle.set_item_to_link(0);
            sparkle.set_aux_timer(1);
            sparkle.set_direction(direction);
        }
        self.ancilla_set_xy(
            k,
            self.player_state_view().x(),
            self.player_state_view().y(),
        );
    }

    pub(super) fn ancilla_add_spin_attack_init_spark(&mut self, a: u8, x: u8, y: u8) {
        const SPIN_ATTACK_START_SPARKLE_Y: [i8; 4] = [32, -8, 10, 20];
        const SPIN_ATTACK_START_SPARKLE_X: [i8; 4] = [10, 7, 28, -10];

        let k = self.ancilla_add_ancilla(a, y);
        for i in (0..=4).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x31 {
                let value = 0;
                self.ancilla_slot_view_mut(i).set_ancilla_type(value);
            }
        }
        let j = self.player_state_view().facing_index();
        let spark_x = self
            .player_state_view()
            .x()
            .wrapping_add(SPIN_ATTACK_START_SPARKLE_X[j] as i16 as u16);
        let spark_y = self
            .player_state_view()
            .y()
            .wrapping_add(SPIN_ATTACK_START_SPARKLE_Y[j] as i16 as u16);
        let Some(k) = k else {
            // C writes through k = -1 on allocation failure; preserve those
            // aliasing writes explicitly instead of silently returning.
            self.ancilla_spawn_scratch_view_mut()
                .write_failed_spin_sparkle(x, spark_x, spark_y);
            return;
        };
        {
            let mut sparkle = self.ancilla_slot_view_mut(k);
            sparkle.set_item_to_link(0);
            sparkle.set_step(x);
            sparkle.set_timer(4);
            sparkle.set_aux_timer(3);
        }
        self.ancilla_set_xy(k, spark_x, spark_y);
    }

    fn ancilla_add_sword_charge_sparkle(&mut self, k: usize) {
        let mut j = 9usize;
        while self.ancilla_slot_view(j).ancilla_type() != 0 {
            if j == 0 {
                return;
            }
            j -= 1;
        }
        self.ancilla_slot_view_mut(j).set_ancilla_type(60);
        let value = self.player_state_view().lower_level_state();
        self.ancilla_slot_view_mut(j).set_floor(value);
        {
            let mut sparkle = self.ancilla_slot_view_mut(j);
            sparkle.set_item_to_link(0);
            sparkle.set_timer(4);
        }

        let rand = self.get_random_number();

        let mut z = self.ancilla_slot_view(k).z();
        if z >= 0xf8 {
            z = 0;
        }
        let dst_x = self
            .ancilla_get_x(k)
            .wrapping_add(2)
            .wrapping_add((rand >> 5) as u16);
        let dst_y = self
            .ancilla_get_y(k)
            .wrapping_sub(2)
            .wrapping_sub(z as u16)
            .wrapping_add((rand & 0xf) as u16);
        if self.replay_ancilla_trace_enabled() {
            println!(
                "ancilla-trace kind=child-charge abs={} fc=0x{:02x} src={} dst={} rng=0x{:02x} base=0x{:04x}/0x{:04x} z=0x{:02x} xy=0x{:04x}/0x{:04x} type=0x{:02x} timer=0x{:02x} floor=0x{:02x} link=0x{:04x}/0x{:04x} face=0x{:02x} spin=0x{:02x} speed=0x{:02x}/0x{:02x}",
                self.state_recorder.replay_frame_counter,
                self.frame_state().frame_counter,
                k,
                j,
                rand,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                z,
                dst_x,
                dst_y,
                self.ancilla_slot_view(j).ancilla_type(),
                self.ancilla_slot_view(j).timer(),
                self.ancilla_slot_view(j).floor(),
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.player_state_view().facing(),
                self.player_state_view().spin_attack_step_counter(),
                self.player_state_view().actual_x_velocity(),
                self.player_state_view().actual_y_velocity(),
            );
        }
        self.ancilla_set_xy(j, dst_x, dst_y);
    }

    fn ancilla_add_silver_arrow_sparkle(&mut self, kin: usize) {
        const SILVER_ARROW_SPARKLE_X: [i8; 4] = [-4, -4, 0, 2];
        const SILVER_ARROW_SPARKLE_Y: [i8; 4] = [0, 2, -4, -4];

        if let Some(k) = self.ancilla_alloc_high() {
            {
                let mut sparkle = self.ancilla_slot_view_mut(k);
                sparkle.set_ancilla_type(0x3c);
                sparkle.set_item_to_link(0);
                sparkle.set_timer(4);
            }
            let value = self.player_state_view().lower_level_state();
            self.ancilla_slot_view_mut(k).set_floor(value);
            let m = self.get_random_number();
            let j = (self.ancilla_slot_view(kin).direction() & 3) as usize;
            self.ancilla_set_xy(
                k,
                self.ancilla_get_x(kin)
                    .wrapping_add(SILVER_ARROW_SPARKLE_X[j] as i16 as u16)
                    .wrapping_add(((m >> 4) & 7) as u16),
                self.ancilla_get_y(kin)
                    .wrapping_add(SILVER_ARROW_SPARKLE_Y[j] as i16 as u16)
                    .wrapping_add((m & 7) as u16),
            );
        }
    }

    pub(super) fn ancilla_add_ice_rod_shot(&mut self, a: u8, y: u8) {
        const ICE_ROD_X: [i8; 4] = [0, 0, -20, 20];
        const ICE_ROD_Y: [i8; 4] = [-16, 24, 8, 8];
        const ICE_ROD_XVEL: [i8; 4] = [0, 0, -48, 48];
        const ICE_ROD_YVEL: [i8; 4] = [-48, 48, 0, 0];

        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            self.refund_magic(0);
            return;
        };
        self.set_sound_effect_1_with_link_pan(15);
        let value = 1;
        self.ancilla_slot_view_mut(k).set_l(value);
        let j = self.player_state_view().facing_index();
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_step(0);
            ancilla.set_work_byte_25(0);
            ancilla.set_item_to_link(255);
            ancilla.set_aux_timer(3);
            ancilla.set_work_byte_3(6);
            ancilla.set_direction(j as u8);
            ancilla.set_y_velocity(ICE_ROD_YVEL[j] as u8);
            ancilla.set_x_velocity(ICE_ROD_XVEL[j] as u8);
        }

        if self.ancilla_check_initial_tile_a(k) < 0 {
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(ICE_ROD_X[j] as i16 as u16);
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(ICE_ROD_Y[j] as i16 as u16);

            if (x.wrapping_sub(self.world_scroll().bg2_x())
                | y.wrapping_sub(self.world_scroll().bg2_y()))
                & 0xff00
                != 0
            {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            }
            self.ancilla_set_xy(k, x, y);
        } else {
            self.ancilla_slot_view_mut(k).set_ancilla_type(0x11);
            let value = ANCILLA_DRAW_SPRITE_COUNTS[0x11];
            self.ancilla_slot_view_mut(k).set_num_sprites(value);
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_item_to_link(0);
            ancilla.set_aux_timer(4);
        }
    }

    #[track_caller]
    pub(super) fn ancilla_add_splash(&mut self, a: u8, y: u8) -> bool {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            if std::env::var_os("ZELDA3_REPLAY_SPLASH_TRACE").is_some() {
                let caller = std::panic::Location::caller();
                println!(
                    "splash-trace abs={} fc=0x{:02x} a=0x{:02x} yarg=0x{:02x} slot=-1 caller={}:{} link=0x{:04x}/0x{:04x} state=0x{:02x} deep=0x{:04x} inwater=0x{:02x} indoors={} lower=0x{:02x} aux=0x{:02x} z=0x{:02x} vz=0x{:02x} tile=0x{:04x} normal=0x{:04x} joy=0x{:02x}/0x{:02x}",
                    self.state_recorder.replay_frame_counter,
                    self.frame_state().frame_counter,
                    a,
                    y,
                    caller.file(),
                    caller.line(),
                    self.player_state_view().x(),
                    self.player_state_view().y(),
                    self.player_state_view().handler_state(),
                    self.tile_detect_position_view().deepwater(),
                    self.player_state_view().deep_water_state(),
                    self.world_location_state().indoor_flag,
                    self.player_state_view().lower_level_state(),
                    self.player_state_view().auxiliary_state(),
                    self.player_state_view().z(),
                    self.player_state_view().actual_z_velocity(),
                    self.tile_detect_position_view().tile_type(),
                    self.tile_detect_position_view().normal_tiles(),
                    self.player_state_view().joypad1h_last(),
                    self.player_state_view().joypad1l_last(),
                );
            }
            return true;
        };
        if std::env::var_os("ZELDA3_REPLAY_SPLASH_TRACE").is_some() {
            let caller = std::panic::Location::caller();
            println!(
                "splash-trace abs={} fc=0x{:02x} a=0x{:02x} yarg=0x{:02x} slot={} caller={}:{} link=0x{:04x}/0x{:04x} state=0x{:02x} deep=0x{:04x} inwater=0x{:02x} indoors={} lower=0x{:02x} aux=0x{:02x} z=0x{:02x} vz=0x{:02x} tile=0x{:04x} normal=0x{:04x} joy=0x{:02x}/0x{:02x}",
                self.state_recorder.replay_frame_counter,
                self.frame_state().frame_counter,
                a,
                y,
                k,
                caller.file(),
                caller.line(),
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.player_state_view().handler_state(),
                self.tile_detect_position_view().deepwater(),
                self.player_state_view().deep_water_state(),
                self.world_location_state().indoor_flag,
                self.player_state_view().lower_level_state(),
                self.player_state_view().auxiliary_state(),
                self.player_state_view().z(),
                self.player_state_view().actual_z_velocity(),
                self.tile_detect_position_view().tile_type(),
                self.tile_detect_position_view().normal_tiles(),
                self.player_state_view().joypad1h_last(),
                self.player_state_view().joypad1l_last(),
            );
        }
        self.set_sound_effect_1_with_link_pan(0x24);
        {
            let mut splash = self.ancilla_slot_view_mut(k);
            splash.set_item_to_link(0);
            splash.set_aux_timer(2);
        }
        if self.world_location_state().is_indoors() && !self.player_state_view().is_in_deep_water()
        {
            self.player_state_view_mut().set_lower_level_state(0);
        }
        self.ancilla_set_xy(
            k,
            self.player_state_view().x().wrapping_sub(11),
            self.player_state_view().y().wrapping_add(8),
        );
        false
    }

    pub(super) fn ancilla_add_grave_stone(&mut self, ain: u8, yin: u8) {
        const MOVE_GRAVESTONE_Y: [u16; 8] =
            [0x550, 0x540, 0x530, 0x520, 0x500, 0x4e0, 0x4c0, 0x4b0];
        const MOVE_GRAVESTONE_X: [u16; 15] = [
            0x8b0, 0x8f0, 0x910, 0x950, 0x970, 0x9a0, 0x850, 0x870, 0x8b0, 0x8f0, 0x920, 0x950,
            0x880, 0x990, 0x840,
        ];
        const MOVE_GRAVESTONE_Y1: [u16; 15] = [
            0x540, 0x530, 0x530, 0x530, 0x520, 0x520, 0x510, 0x510, 0x4f0, 0x4f0, 0x4f0, 0x4f0,
            0x4d0, 0x4b0, 0x4a0,
        ];
        const MOVE_GRAVESTONE_X1: [u16; 15] = [
            0x8b0, 0x8f0, 0x910, 0x950, 0x970, 0x9a0, 0x850, 0x870, 0x8b0, 0x8f0, 0x920, 0x950,
            0x880, 0x990, 0x840,
        ];
        const MOVE_GRAVESTONE_POS: [u16; 15] = [
            0xa16, 0x99e, 0x9a2, 0x9aa, 0x92e, 0x934, 0x88a, 0x88e, 0x796, 0x79e, 0x7a4, 0x7aa,
            0x690, 0x5b2, 0x508,
        ];
        const MOVE_GRAVESTONE_CTR: [u8; 15] = [
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x38,
            0x58,
        ];
        const MOVE_GRAVESTONE_IDX: [u8; 9] = [0, 1, 4, 6, 8, 12, 13, 14, 15];

        let Some(k) = self.ancilla_add_ancilla(ain, yin) else {
            return;
        };
        let link_y = self.player_state_view().y();
        let t = if link_y & 0x0f < 7 {
            link_y
        } else {
            link_y.wrapping_add(16)
        } & !0x0f;

        let mut i = 7usize;
        while MOVE_GRAVESTONE_Y[i] != t {
            if i == 0 {
                self.ancilla_slot_view_mut(k).set_ancilla_type(0);
                return;
            }
            i -= 1;
        }

        let mut j = MOVE_GRAVESTONE_IDX[i] as usize;
        let end = MOVE_GRAVESTONE_IDX[i + 1] as usize;
        loop {
            let x = MOVE_GRAVESTONE_X[j];
            let link_x = self.player_state_view().x();
            if x < link_x && x.wrapping_add(15) >= link_x {
                if (j == 13) == !self.player_state_view().is_running() {
                    break;
                }

                let pos = MOVE_GRAVESTONE_POS[j];
                self.dungeon_state_view_mut()
                    .set_big_rock_starting_address(pos);
                self.dungeon_state_view_mut()
                    .set_door_open_counter(MOVE_GRAVESTONE_CTR[j] as u16);
                if self.dungeon_state_view().door_open_counter_low() == 0x58 {
                    self.set_sound_effect_2_with_link_pan(0x1b);
                } else if self.dungeon_state_view().door_open_counter_low() == 0x38 {
                    let screen = self.world_location_state().overworld_screen_index() as usize;
                    self.overworld_event_info_view_mut()
                        .set_event_bits(screen, 0x20);
                    self.set_sound_effect_2_with_link_pan(0x1b);
                }

                let debris = pos.wrapping_sub(0x80);
                self.door_debris_view_mut()
                    .set_y_low_and_x_low_from_word(k, debris);

                self.Overworld_DoMapUpdate32x32_B();

                if self.system_signals_view().sound_effect_2() & 0x3f != 0x1b {
                    self.set_sound_effect_1_with_link_pan(0x22);
                }

                let yy = MOVE_GRAVESTONE_Y1[j];
                let xx = MOVE_GRAVESTONE_X1[j];
                self.player_state_view_mut().set_defense_flags(4);
                self.player_state_view_mut().set_hookshot_grave_latch();
                let ancilla_a = yy.wrapping_sub(18);
                let value = ancilla_a as u8;
                self.ancilla_slot_view_mut(k).set_a(value);
                let value = (ancilla_a >> 8) as u8;
                self.ancilla_slot_view_mut(k).set_b(value);
                self.ancilla_set_xy(k, xx, yy.wrapping_sub(2));
                return;
            }
            j += 1;
            if j == end {
                break;
            }
        }
        self.ancilla_slot_view_mut(k).set_ancilla_type(0);
    }

    pub(super) fn ancilla_add_waterfall_splash(&mut self) {
        if self.ancilla_add_check_for_presence(0x41) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(0x41, 4) {
            let mut splash = self.ancilla_slot_view_mut(k);
            splash.set_timer(2);
            splash.set_item_to_link(0);
        }
    }

    pub(super) fn ancilla_add_door_debris(&mut self) -> i32 {
        let Some(k) = self.ancilla_add_ancilla(8, 1) else {
            return -1;
        };
        {
            let mut debris = self.ancilla_slot_view_mut(k);
            debris.set_work_byte_25(0);
            debris.set_work_byte_26(7);
        }
        k as i32
    }

    fn ancilla_add_occasional_sparkle(&mut self, k: usize) {
        if self.frame_state().frame_counter & 7 == 0 {
            self.ancilla_add_sword_charge_sparkle(k);
        }
    }

    fn ancilla43_ganons_tower_cutscene(&mut self, k: usize) {
        let mut oam = self.oam_state_view().current_pointer_usize();
        let mut draw_ring = true;

        if self.ancilla_slot_view(k).step() == 0 {
            let yy = self.ancilla_slot_view(k).y_velocity().wrapping_sub(1);
            let value = if yy < 0xf0 { 0xf0 } else { yy };
            self.ancilla_slot_view_mut(k).set_y_velocity(value);
            self.ancilla_move_y(k);
            let x = self.ancilla_get_x(k);
            let y = self.ancilla_get_y(k);
            let bg2vofs = self.ppu_scroll_copy_view().bg2_v_copy();
            if y.wrapping_sub(bg2vofs) < 0x38 {
                self.tower_seal_scratch_view_mut().set_center(
                    x.wrapping_add(8),
                    0x38u16.wrapping_add(8).wrapping_add(bg2vofs),
                );
                self.ancilla_set_y(k, 0x38u16.wrapping_add(bg2vofs));
                self.ancilla_slot_view_mut(k).add_step(1);
                self.system_signals_view_mut().set_ambient_sound_effect(5);
                self.system_signals_view_mut().set_music_control(0xf1);
                self.dialogue_message_index_view_mut().set_value(0x013b);
                self.main_show_text_message();
                draw_ring = false;
            } else if self.frame_state().submodule == 0 {
                draw_ring = false;
            }
        }

        if draw_ring {
            if self.ancilla_slot_view(k).step() == 1 && self.frame_state().submodule == 0 {
                let value = 16;
                self.ancilla_slot_view_mut(k).set_x_velocity(value);
                let bak0 = self.ancilla_slot_view(k).x_low();
                let bak1 = self.ancilla_slot_view(k).x_high();
                let value = self.tower_seal_scratch_view().ring_radius();
                self.ancilla_slot_view_mut(k).set_x_low(value);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_x_high(value);
                self.ancilla_move_x(k);
                let radius = self.ancilla_slot_view(k).x_low();
                self.tower_seal_scratch_view_mut().set_ring_radius(radius);
                let value = bak0;
                self.ancilla_slot_view_mut(k).set_x_low(value);
                let value = bak1;
                self.ancilla_slot_view_mut(k).set_x_high(value);
                if self.tower_seal_scratch_view().ring_radius() >= 48 {
                    self.tower_seal_scratch_view_mut().set_ring_radius(48);
                    self.ancilla_slot_view_mut(k).add_step(1);
                }
            }

            if self.frame_state().submodule == 0
                && self.ancilla_slot_view(k).step() != 0
                && self.ancilla_slot_view(k).step() != 1
            {
                if self.ancilla_slot_view(k).step() == 2 {
                    if self.tower_seal_scratch_view_mut().tick_wait_countdown() == 0 {
                        self.set_special_entrance_trigger(5);
                        self.set_subsubmodule(0);
                        self.scratch_word_view_mut()
                            .clear_module_transition_counter();
                        self.ancilla_slot_view_mut(k).add_step(1);
                    }
                } else {
                    let value = 48;
                    self.ancilla_slot_view_mut(k).set_x_velocity(value);
                    let bak0 = self.ancilla_slot_view(k).x_low();
                    let bak1 = self.ancilla_slot_view(k).x_high();
                    let value = self.tower_seal_scratch_view().ring_radius();
                    self.ancilla_slot_view_mut(k).set_x_low(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_x_high(value);
                    self.ancilla_move_x(k);
                    let radius = self.ancilla_slot_view(k).x_low();
                    self.tower_seal_scratch_view_mut().set_ring_radius(radius);
                    let value = bak0;
                    self.ancilla_slot_view_mut(k).set_x_low(value);
                    let value = bak1;
                    self.ancilla_slot_view_mut(k).set_x_high(value);
                    if self.tower_seal_scratch_view().ring_radius() >= 240 {
                        self.palette_buffer_view_mut().set_sp6r_indoors(0);
                        self.palette_buffer_view_mut()
                            .select_overworld_aux_palette_offset();
                        self.Palette_Load_SpriteEnvironment_Dungeon();
                        self.system_signals_view_mut().increment_cgram_update_flag();
                        let value = 0;
                        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                        return;
                    }
                }
            }

            let astep = self.ancilla_slot_view(k).step();
            if astep != 0 {
                oam = self.gt_cutscene_sparkle_a_lot(oam);
            }

            for j in (0..=6).rev() {
                if self.frame_state().submodule == 0
                    && astep != 1
                    && self.frame_state().frame_counter & 1 == 0
                {
                    self.tower_seal_orbit_view_mut(j).advance_angle_mod64();
                }
                let arp = self.ancilla_get_radial_projection(
                    self.tower_seal_orbit_view(j).angle(),
                    self.tower_seal_scratch_view().ring_radius(),
                );
                let x = (if arp.r6 != 0 {
                    -(arp.r4 as i32)
                } else {
                    arp.r4 as i32
                }) + i32::from(self.tower_seal_scratch_view().center_x())
                    - 8
                    - i32::from(self.ppu_scroll_copy_view().bg2_h_copy());
                let y = (if arp.r2 != 0 {
                    -(arp.r0 as i32)
                } else {
                    arp.r0 as i32
                }) + i32::from(self.tower_seal_scratch_view().center_y())
                    - 8
                    - i32::from(self.ppu_scroll_copy_view().bg2_v_copy());

                self.tower_seal_orbit_view_mut(j)
                    .set_base_sparkle_position(x as u16, y as u16);

                self.ancilla_draw_gt_cutscene_crystal(oam, x as u16, y as u16);
                oam += 4;
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.tower_seal_orbit_view_mut(7)
            .set_base_sparkle_position(x, y);

        self.ancilla_draw_gt_cutscene_crystal(oam, x, y);

        if self.ancilla_slot_view(k).step() == 0 {
            self.ancilla_add_occasional_sparkle(k);
        } else if self.frame_state().submodule == 0 {
            self.gt_cutscene_activate_sparkle();
        }
    }

    fn ancilla_draw_gt_cutscene_crystal(&mut self, oam: usize, x: u16, y: u16) {
        self.ancilla_set_oam_safe(oam, x, y, 0x24, 0x3c, 2);
    }

    fn fire_shot_draw(&mut self, k: usize) {
        const FIRE_SHOT_DRAW_X2: [u8; 16] = [7, 0, 8, 0, 8, 4, 0, 0, 2, 8, 0, 0, 1, 4, 9, 0];
        const FIRE_SHOT_DRAW_Y2: [u8; 16] = [1, 4, 9, 0, 7, 0, 8, 0, 8, 4, 0, 0, 2, 8, 0, 0];
        const FIRE_SHOT_DRAW_CHAR2: [u8; 3] = [0x8d, 0x9d, 0x9c];

        let Some(mut info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };
        if self.ancilla_slot_view(k).object_priority() != 0 {
            info.flags |= 0x30;
        }

        let mut oam = self.oam_state_view().current_pointer_usize();
        let j = (self.ancilla_slot_view(k).item_to_link() & 0x0c) as usize;
        for i in (0..=2).rev() {
            self.ancilla_set_oam_plain(
                oam,
                u16::from(info.x.wrapping_add(FIRE_SHOT_DRAW_X2[j + i])),
                u16::from(info.y.wrapping_add(FIRE_SHOT_DRAW_Y2[j + i])),
                FIRE_SHOT_DRAW_CHAR2[i],
                info.flags | 2,
                0,
            );
            oam += 4;
        }
    }

    fn ice_shot_spread_draw(&mut self, k: usize) {
        const ICE_SHOT_SPREAD_TILE: [OamTileAttrs; 8] = oam_tile_attrs![
            0xcf, 0x24, 0xcf, 0x24, 0xcf, 0x24, 0xcf, 0x24, 0xdf, 0x24, 0xdf, 0x24, 0xdf, 0x24,
            0xdf, 0x24,
        ];
        const ICE_SHOT_SPREAD_OFFSET: [SignedOffset; 8] =
            signed_offsets![0, 0, 0, 8, 8, 0, 8, 8, -8, -8, -8, 16, 16, -8, 16, 16,];

        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(
            k,
            self.ancilla_slot_view(k).num_sprites(),
        );
        let mut oam = self.oam_state_view().current_pointer_usize();
        let oam_org = oam;
        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 4;
        for _ in 0..4 {
            let offset = ICE_SHOT_SPREAD_OFFSET[j];
            let tile = ICE_SHOT_SPREAD_TILE[j];
            let y = info_y.wrapping_add(offset.y as i16 as u16);
            let x = info_x.wrapping_add(offset.x as i16 as u16);
            let mut yv = 0xf0;
            if x < 256 && y < 256 {
                self.oam_state_view_mut().set_entry_x(oam, x as u8);
                if y < 224 {
                    yv = y as u8;
                }
            }
            self.oam_state_view_mut().set_entry_y(oam, yv);
            self.oam_state_view_mut().set_entry_char(oam, tile.char);
            let flags = tile.flags & !0x30 | self.oam_state_view().priority_high();
            self.oam_state_view_mut().set_entry_flags(oam, flags);
            let value = 0;
            self.oam_state_view_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
            j += 1;
        }
        if self.oam_state_view().entry_y(oam_org) == 0xf0
            && self.oam_state_view().entry_y(oam_org + 4) == 0xf0
        {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        }
    }

    fn ancilla11_ice_rod_wall_hit(&mut self, k: usize) {
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

    fn ancilla0_a_arrow_in_the_wall(&mut self, k: usize) {
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
        if self.frame_state().submodule == 0 {
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

    fn somarian_blast_draw(&mut self, k: usize) {
        const SOMARIAN_BLAST_FLAGS: [u8; 2] = [2, 6];
        const SOMARIAN_BLAST_DRAW_X0: [i8; 24] = [
            0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const SOMARIAN_BLAST_DRAW_X1: [i8; 24] = [
            8, 8, 8, 8, 4, 4, 8, 8, 8, 8, 4, 4, 0, 0, 0, 0, 8, 8, 0, 0, 0, 0, 8, 8,
        ];
        const SOMARIAN_BLAST_DRAW_Y0: [u8; 24] = [
            0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 4, 4,
        ];
        const SOMARIAN_BLAST_DRAW_Y1: [u8; 24] = [
            0, 0, 0, 0, 8, 8, 0x80, 0, 0, 0, 8, 8, 0x80, 8, 8, 8, 4, 4, 0x80, 8, 8, 8, 4, 4,
        ];
        const SOMARIAN_BLAST_DRAW_FLAGS0: [u8; 24] = [
            0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0xc0, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0xc0, 0, 0, 0, 0, 0, 0x80,
        ];
        const SOMARIAN_BLAST_DRAW_FLAGS1: [u8; 24] = [
            0x80, 0x80, 0x80, 0x80, 0x80, 0xc0, 0, 0, 0, 0, 0, 0x40, 0xc0, 0xc0, 0xc0, 0xc0, 0x40,
            0xc0, 0x80, 0x80, 0x80, 0x80, 0, 0x80,
        ];
        const SOMARIAN_BLAST_DRAW_CHAR0: [u8; 24] = [
            0x50, 0x50, 0x44, 0x44, 0x52, 0x52, 0x50, 0x50, 0x44, 0x44, 0x51, 0x51, 0x43, 0x43,
            0x42, 0x42, 0x41, 0x41, 0x43, 0x43, 0x42, 0x42, 0x40, 0x40,
        ];
        const SOMARIAN_BLAST_DRAW_CHAR1: [u8; 24] = [
            0x50, 0x50, 0x44, 0x44, 0x51, 0x51, 0x50, 0x50, 0x44, 0x44, 0x52, 0x52, 0x43, 0x43,
            0x42, 0x42, 0x40, 0x40, 0x43, 0x43, 0x42, 0x42, 0x41, 0x41,
        ];

        let Some(mut info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };
        info.flags |= SOMARIAN_BLAST_FLAGS[self.ancilla_slot_view(k).item_to_link() as usize];
        if self.ancilla_slot_view(k).object_priority() != 0 {
            info.flags |= 0x30;
        }
        let oam = self.oam_state_view().current_pointer_usize();
        let j = self.ancilla_slot_view(k).direction() as usize * 6
            + self.ancilla_slot_view(k).step() as usize;
        self.oam_state_view_mut().write_entry(
            oam,
            info.x.wrapping_add(SOMARIAN_BLAST_DRAW_X0[j] as u8),
            if sign8(SOMARIAN_BLAST_DRAW_Y0[j]) {
                0xf0
            } else {
                info.y.wrapping_add(SOMARIAN_BLAST_DRAW_Y0[j])
            },
            0x82u8.wrapping_add(SOMARIAN_BLAST_DRAW_CHAR0[j]),
            info.flags | SOMARIAN_BLAST_DRAW_FLAGS0[j],
        );
        let value = 0;
        self.oam_state_view_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
        self.oam_state_view_mut().write_entry(
            oam + 4,
            info.x.wrapping_add(SOMARIAN_BLAST_DRAW_X1[j] as u8),
            if sign8(SOMARIAN_BLAST_DRAW_Y1[j]) {
                0xf0
            } else {
                info.y.wrapping_add(SOMARIAN_BLAST_DRAW_Y1[j])
            },
            0x82u8.wrapping_add(SOMARIAN_BLAST_DRAW_CHAR1[j]),
            info.flags | SOMARIAN_BLAST_DRAW_FLAGS1[j],
        );
        let value = 0;
        self.oam_state_view_mut()
            .set_extended_byte((oam - OAM_BUF) / 4 + 1, value);
    }

    fn arrow_draw(&mut self, k: usize) {
        const ARROW_DRAW_CHAR: [u8; 48] = [
            0x2b, 0x2a, 0x2a, 0x2b, 0x3d, 0x3a, 0x3a, 0x3d, 0x2b, 0xff, 0x2b, 0xff, 0x3d, 0xff,
            0x3d, 0xff, 0x3c, 0x2c, 0x3c, 0x2a, 0x3c, 0x2c, 0x3c, 0x2a, 0x2c, 0x3c, 0x2a, 0x3c,
            0x2c, 0x3c, 0x2a, 0x3c, 0x3b, 0x2d, 0x3b, 0x3a, 0x3b, 0x2d, 0x3b, 0x3a, 0x2d, 0x3b,
            0x3a, 0x3b, 0x2d, 0x3b, 0x3a, 0x3b,
        ];
        const ARROW_DRAW_FLAGS: [u8; 48] = [
            0xa4, 0xa4, 0x24, 0x24, 0x64, 0x64, 0x24, 0x24, 0xa4, 0xff, 0x24, 0xff, 0x64, 0xff,
            0x24, 0xff, 0xa4, 0xa4, 0xa4, 0xa4, 0xa4, 0xe4, 0xa4, 0xa4, 0x24, 0x24, 0x24, 0x24,
            0x64, 0x24, 0x24, 0x24, 0x64, 0x64, 0x64, 0xe4, 0x64, 0xe4, 0x64, 0xe4, 0x24, 0x24,
            0x24, 0xa4, 0xa4, 0x24, 0x24, 0xa4,
        ];
        const ARROW_DRAW_Y: [i8; 48] = [
            0, 8, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0,
            8, 0, 8, -1, -1, 0, 0, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 0, 0,
        ];
        const ARROW_DRAW_X: [i8; 48] = [
            0, 0, 0, 0, 0, 8, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, -2, 0, 0, 1, 1, 0, 0,
            -2, -1, 0, 0, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8,
        ];

        let (mut x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        if self.ancilla_slot_view(k).object_priority() != 0 {
            self.oam_state_view_mut().set_priority_high(0x30);
        }
        if self.ancilla_slot_view(k).h() != 0 {
            x = x.wrapping_add(
                self.world_scroll()
                    .bg2_y()
                    .wrapping_sub(self.world_scroll().bg1_y()),
            );
            y = y.wrapping_add(
                self.world_scroll()
                    .bg2_x()
                    .wrapping_sub(self.world_scroll().bg1_x()),
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

        let mut oam = self.oam_state_view().current_pointer_usize();
        let oam_org = oam;
        let flags = if self.inventory_items().has_silver_arrows() {
            2
        } else {
            4
        };
        for _ in 0..2 {
            if ARROW_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(ARROW_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(ARROW_DRAW_Y[j] as i16 as u16),
                    ARROW_DRAW_CHAR[j],
                    ARROW_DRAW_FLAGS[j] & !0x3e | flags | self.oam_state_view().priority_high(),
                    0,
                );
                oam += 4;
            }
            j += 1;
        }

        if self.oam_state_view().entry_y(oam_org) == 0xf0
            && self.oam_state_view().entry_y(oam_org + 4) == 0xf0
        {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        }
    }

    fn revival_fairy_monitor_hp(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        if (self.player_resources_view().current_health()
            == self.player_resources_view().health_capacity()
            || self.player_resources_view().current_health() == 0x38)
            && !self.hud_state_view().is_doing_heart_animation()
        {
            if self.player_state_view().is_in_deep_water() {
                self.player_state_view_mut().set_swim_direction_flags(4);
                self.player_state_view_mut().set_handler_state(4);
            } else if self.player_state_view().is_bunny() {
                self.player_state_view_mut().set_handler_state(23);
                self.player_state_view_mut().set_bunny_state(1);
                if self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES) {
                    self.LoadGearPalettes_bunny();
                }
            } else {
                self.player_state_view_mut().clear_handler_state();
            }
            self.player_state_view_mut().clear_auxiliary_state();
            self.player_state_view_mut().clear_faint_animation_active();
            self.player_state_view_mut().clear_item_action_step_var();
            self.player_state_view_mut().set_y_button_action_step(0);
            let mut player = self.player_state_view_mut();
            player.set_z(0);
            player.set_incapacitated_timer(0);
            for i in 0..5 {
                let value = 0;
                self.ancilla_slot_view_mut(i).set_ancilla_type(value);
            }
            return;
        }

        let k = 1;
        if self.ancilla_slot_view(k).step() == 0 {
            self.ancilla_slot_view_mut(k).tick_work_byte_3();
            if self.ancilla_slot_view(k).work_byte_3() == 0 {
                self.ancilla_slot_view_mut(k).add_work_byte_3(1);
                let value = 4;
                self.ancilla_slot_view_mut(k).set_z_velocity(value);
                self.ancilla_move_z(k);
                if self.ancilla_slot_view(k).z() >= 16 {
                    self.ancilla_slot_view_mut(k).add_step(1);
                    let value = 2;
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                }
            }
        } else {
            self.ancilla_slot_view_mut(k).tick_k();
            if sign8(self.ancilla_slot_view(k).k()) {
                let value = 32;
                self.ancilla_slot_view_mut(k).set_k(value);
                let value = 0u8.wrapping_sub(self.ancilla_slot_view(k).z_velocity());
                self.ancilla_slot_view_mut(k).set_z_velocity(value);
            }
            self.ancilla_move_z(k);
        }
        let z = self.ancilla_slot_view(k).z() as u16;
        self.player_state_view_mut().set_z(z);
    }

    fn revival_fairy_dust(&mut self) {
        let k = 2;
        if self.ancilla_slot_view(0).step() == 0 || self.ancilla_slot_view(k).step() == 2 {
            return;
        }
        self.ancilla_slot_view_mut(k).tick_work_byte_3();
        if !sign8(self.ancilla_slot_view(k).work_byte_3()) {
            return;
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_3(value);
        if !self.oam_state_view().has_sprite_sorting() {
            self.oam_allocate_from_region_a(16);
        } else {
            self.oam_allocate_from_region_d(16);
        }
        self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(self.ancilla_slot_view(k).aux_timer()) {
            let value = 3;
            self.ancilla_slot_view_mut(k).set_aux_timer(value);
            if self.ancilla_slot_view(k).item_to_link() == 9 {
                let value = 32;
                self.ancilla_slot_view_mut(k).set_work_byte_3(value);
                self.ancilla_slot_view_mut(k).add_step(1);
                let value = 2;
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                return;
            }
            self.ancilla_slot_view_mut(k).add_item_to_link(1);
            let value =
                MAGIC_POWDER_FRAME_TIMERS[30 + self.ancilla_slot_view(k).item_to_link() as usize];
            self.ancilla_slot_view_mut(k).set_work_byte_25(value);
        }
        self.ancilla_magic_powder_draw(k);
    }

    pub(super) fn revival_fairy_main(&mut self) {
        const REVIVAL_FAIRY_STEP_TIMERS: [u8; 2] = [0, 0x90];
        const REVIVAL_FAIRY_TILE_CHARS: [u8; 5] = [0x4b, 0x4d, 0x49, 0x47, 0x49];

        let k = 0;
        let skip_draw = match self.ancilla_slot_view(k).step() {
            0 => {
                self.ancilla_slot_view_mut(k).tick_work_byte_3();
                if self.ancilla_slot_view(k).work_byte_3() == 0 {
                    self.ancilla_slot_view_mut(k).add_step(1);
                    let value =
                        REVIVAL_FAIRY_STEP_TIMERS[self.ancilla_slot_view(k).step() as usize];
                    self.ancilla_slot_view_mut(k).set_work_byte_3(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_k(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                } else {
                    self.ancilla_move_z(k);
                }
                false
            }
            1 => {
                self.ancilla_slot_view_mut(k).tick_work_byte_3();
                if self.ancilla_slot_view(k).work_byte_3() == 0 {
                    self.ancilla_slot_view_mut(k).add_step(1);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_x_velocity(value);
                } else {
                    if self.ancilla_slot_view(k).work_byte_3() == 0x4f
                        || self.ancilla_slot_view(k).work_byte_3() == 0x8f
                    {
                        self.ancilla_slot_view_mut(k).add_l(1);
                        self.ancilla_sfx2_pan(k, 0x31);
                    }
                    if self.ancilla_slot_view(k).l() != 0 {
                        self.ancilla_slot_view_mut(k).subtract_g(1);
                        if sign8(self.ancilla_slot_view(k).g()) {
                            let value = 5;
                            self.ancilla_slot_view_mut(k).set_g(value);
                            let value = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
                            self.ancilla_slot_view_mut(k).set_item_to_link(value);
                            if self.ancilla_slot_view(k).item_to_link() == 3 {
                                let value = 0;
                                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                                let value = 0;
                                self.ancilla_slot_view_mut(k).set_l(value);
                            }
                        }
                    }
                    let value = self.ancilla_slot_view(k).z_velocity().wrapping_add(
                        if self.ancilla_slot_view(k).k() != 0 {
                            1
                        } else {
                            0xff
                        },
                    );
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                    if abs8(self.ancilla_slot_view(k).z_velocity()) == 8 {
                        self.ancilla_slot_view_mut(k).toggle_k_bit0();
                    }
                    self.ancilla_move_z(k);
                }
                false
            }
            2 => {
                if self.ancilla_slot_view(k).z_velocity() < 24 {
                    self.ancilla_slot_view_mut(k).add_z_velocity(1);
                }
                if self.ancilla_slot_view(k).x_velocity() < 16 {
                    self.ancilla_slot_view_mut(k).add_x_velocity(1);
                }
                self.ancilla_move_x(k);
                self.ancilla_move_z(k);
                false
            }
            3 => true,
            _ => false,
        };

        if !skip_draw {
            self.oam_allocate_from_region_c(12);
            let (x, y) = self.ancilla_prep_oam_coord(k);
            let oam = self.oam_state_view().current_pointer_usize();
            let mut t =
                if self.ancilla_slot_view(k).step() == 1 && self.ancilla_slot_view(k).l() != 0 {
                    self.ancilla_slot_view(k).item_to_link().wrapping_add(1)
                } else {
                    0
                };
            if t != 0 {
                t = t.wrapping_add(1);
            } else {
                t = (self.frame_state().frame_counter >> 2) & 1;
            }
            self.ancilla_set_oam(
                oam,
                x,
                y.wrapping_sub(self.ancilla_slot_view(k).z() as i8 as i16 as u16),
                REVIVAL_FAIRY_TILE_CHARS[t as usize],
                0x74,
                2,
            );
            if self.oam_state_view().entry_y(oam) == 0xf0 {
                let value = 3;
                self.ancilla_slot_view_mut(k).set_step(value);
                self.increment_submodule();
                let main_screen_layers = self.ppu_scroll_copy_view().mapbak_tm();
                self.set_main_screen_layers(main_screen_layers);
            }
        }

        self.revival_fairy_dust();
        self.revival_fairy_monitor_hp();
    }

    fn gt_cutscene_activate_sparkle(&mut self) {
        for k in (0..=0x17).rev() {
            if self.tower_seal_sparkle_view(k).is_free() {
                self.tower_seal_sparkle_view_mut(k).set_phase(0);
                self.tower_seal_sparkle_view_mut(k).set_timer(4);
                let r = self.get_random_number();
                let base = k & 7;
                let (mut x, mut y) = self
                    .tower_seal_sparkle_view_mut(k)
                    .base_sparkle_position(base);
                x = x.wrapping_add((r >> 4) as u16);
                y = y.wrapping_add((r & 0x0f) as u16);
                self.tower_seal_sparkle_view_mut(k).set_position(x, y);
                return;
            }
        }
    }

    fn gt_cutscene_sparkle_a_lot(&mut self, mut oam: usize) -> usize {
        const SWORD_CHARGE_SPARK_CHAR: [u8; 3] = [0xb7, 0x80, 0x83];
        const SWORD_CHARGE_SPARK_FLAGS: [u8; 3] = [4, 4, 0x84];

        for k in (0..=0x17).rev() {
            if self.tower_seal_sparkle_view(k).is_free() {
                continue;
            }

            let timer = self.tower_seal_sparkle_view_mut(k).tick_timer();
            if sign8(timer) {
                self.tower_seal_sparkle_view_mut(k).set_timer(4);
                let phase = self.tower_seal_sparkle_view_mut(k).advance_phase();
                if phase == 3 {
                    self.tower_seal_sparkle_view_mut(k).set_phase(0xff);
                    continue;
                }
            }

            let x = self.tower_seal_sparkle_view(k).x();
            let y = self.tower_seal_sparkle_view(k).y();
            let j = self.tower_seal_sparkle_view(k).phase() as usize;
            self.ancilla_set_oam(
                oam,
                x,
                y,
                SWORD_CHARGE_SPARK_CHAR[j],
                SWORD_CHARGE_SPARK_FLAGS[j] | 0x30,
                0,
            );
            oam += 4;
        }
        oam
    }

    fn ancilla_add_rupees(&mut self, k: usize) -> bool {
        const RUPEE_GIFT_AMOUNTS: [u16; 5] = [1, 5, 20, 100, 50];
        let a = self.ancilla_slot_view(k).item_to_link();
        let amount = if (0x34..=0x36).contains(&a) {
            RUPEE_GIFT_AMOUNTS[(a - 0x34) as usize]
        } else if a == 0x40 || a == 0x41 {
            RUPEE_GIFT_AMOUNTS[(a - 0x40 + 3) as usize]
        } else if a == 0x46 {
            300
        } else if a == 0x47 {
            20
        } else {
            return false;
        };
        let rupees = self
            .player_resources_view()
            .rupees_goal()
            .wrapping_add(amount);
        self.player_resources_view_mut().set_rupees_goal(rupees);
        true
    }

    fn somaria_block_spawn_bullets(&mut self, k: usize) {
        const SPAWN_CENTRIFUGAL_QUAD_X: [i8; 4] = [-8, -8, -9, -4];
        const SPAWN_CENTRIFUGAL_QUAD_Y: [i8; 4] = [-15, -4, -8, -8];

        let z = if self.ancilla_slot_view(k).z() == 0xff {
            0
        } else {
            self.ancilla_slot_view(k).z()
        };
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k).wrapping_sub(z as u16);

        for i in (0..=3).rev() {
            if let Some(j) = self.ancilla_alloc_init(1, 4) {
                let value = 1;
                self.ancilla_slot_view_mut(j).set_ancilla_type(value);
                let value = ANCILLA_DRAW_SPRITE_COUNTS[1];
                self.ancilla_slot_view_mut(j).set_num_sprites(value);
                let value = 4;
                self.ancilla_slot_view_mut(j).set_step(value);
                let value = 0;
                self.ancilla_slot_view_mut(j).set_item_to_link(value);
                let value = 0;
                self.ancilla_slot_view_mut(j).set_object_priority(value);
                let value = i as u8;
                self.ancilla_slot_view_mut(j).set_direction(value);
                self.ancilla_set_xy(
                    j,
                    x.wrapping_add(SPAWN_CENTRIFUGAL_QUAD_X[i] as i16 as u16),
                    y.wrapping_add(SPAWN_CENTRIFUGAL_QUAD_Y[i] as i16 as u16),
                );
                self.ancilla_terminate_if_offscreen(j);
                let value = FIRE_ROD_SPARK_X_VELOCITIES[i] as u8;
                self.ancilla_slot_view_mut(j).set_x_velocity(value);
                let value = FIRE_ROD_SPARK_Y_VELOCITIES[i] as u8;
                self.ancilla_slot_view_mut(j).set_y_velocity(value);
                let value = self.ancilla_slot_view(k).floor();
                self.ancilla_slot_view_mut(j).set_floor(value);
                let value = self.player_state_view().lower_level_mirror_state();
                self.ancilla_slot_view_mut(j).set_floor2(value);
            }
        }
        self.temp_counter_view_mut().set(0xff);
    }

    fn ancilla_terminate_if_offscreen(&mut self, j: usize) {
        let xt: u16 = if self.enhanced_features_view().has(1) {
            0x40
        } else {
            0
        };
        let x = self
            .ancilla_get_x(j)
            .wrapping_sub(self.world_scroll().bg2_x())
            .wrapping_add(xt);
        let y = self
            .ancilla_get_y(j)
            .wrapping_sub(self.world_scroll().bg2_y());
        if x >= 244 + xt * 2 || y >= 240 {
            let value = 0;
            self.ancilla_slot_view_mut(j).set_ancilla_type(value);
        }
    }

    fn ancilla_draw_somaria_block(&mut self, k: usize) {
        const SOMARIAN_BLOCK_DRAW_X: [i8; 12] = [-8, 0, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        const SOMARIAN_BLOCK_DRAW_Y: [i8; 12] = [-8, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        const SOMARIAN_BLOCK_DRAW_FLAGS: [u8; 12] = [
            0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0,
        ];

        if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize
            && self.player_state_view().is_lifting_or_carrying()
            && self.ancilla_slot_view(k).k() != 3
            && self.player_state_view().facing() == 0
        {
            self.ancilla_allocate_oam_from_region_b_or_e(self.ancilla_slot_view(k).num_sprites());
        } else if self.oam_state_view().has_sprite_sorting()
            && self.ancilla_slot_view(k).floor() != 0
            && (self.ancilla_slot_view(k).l() != 0
                || k + 1 == self.player_state_view().ancilla_pickup_flag() as usize
                    && self.player_state_view().is_lifting_or_carrying())
        {
            self.oam_state_view_mut().set_current_pointer(0x08d0);
            self.oam_state_view_mut()
                .set_current_extended_pointer(0x0a20 + 0x34);
        }

        let (x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        let oam_org = self.oam_state_view().current_pointer_usize();
        let mut oam = oam_org;
        let z = self.ancilla_slot_view(k).z() as i8;
        if z != 0
            && z != -1
            && self.ancilla_slot_view(k).k() != 3
            && self.ancilla_slot_view(k).object_priority() != 0
        {
            self.oam_state_view_mut().set_priority_word(0x3000);
        }
        y = y.wrapping_sub(z as i16 as u16);
        let mut j = self.ancilla_slot_view(k).work_byte_1() as usize * 4;
        for _ in 0..4 {
            self.ancilla_set_oam_safe(
                oam,
                x.wrapping_add(SOMARIAN_BLOCK_DRAW_X[j] as i16 as u16),
                y.wrapping_add(SOMARIAN_BLOCK_DRAW_Y[j] as i16 as u16),
                0xe9,
                SOMARIAN_BLOCK_DRAW_FLAGS[j] & !0x30 | 2 | self.oam_state_view().priority_high(),
                0,
            );
            j += 1;
            oam += 4;
        }

        if self.somarian_block_check_empty(oam_org) {
            self.dungeon_state_view_mut()
                .clear_somaria_block_switch_counter();
            self.ancilla_slot_view_mut(k).set_ancilla_type(0);
            if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize {
                self.player_state_view_mut().clear_ancilla_pickup_flag();
                if self.player_state_view().is_lifting_or_carrying() {
                    self.player_state_view_mut().clear_state_bits();
                }
            }
        }
    }

    fn somaria_block_check_for_switch(&mut self, k: usize) -> bool {
        const SOMARIAN_BLOCK_CHECK_COVER_X: [i8; 4] = [0, 0, -4, 4];
        const SOMARIAN_BLOCK_CHECK_COVER_Y: [i8; 4] = [-4, 4, 0, 0];
        self.dungeon_state_view_mut()
            .clear_somaria_block_switch_counter();
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_24(value);
        for j in (0..=3).rev() {
            let y = self
                .ancilla_get_y(k)
                .wrapping_add(SOMARIAN_BLOCK_CHECK_COVER_Y[j] as i16 as u16);
            let x = self
                .ancilla_get_x(k)
                .wrapping_add(SOMARIAN_BLOCK_CHECK_COVER_X[j] as i16 as u16);
            let bak = self.ancilla_slot_view(k).object_priority();
            self.ancilla_check_tile_collision_targeted(k, x, y);
            let value = bak;
            self.ancilla_slot_view_mut(k).set_object_priority(value);
            if matches!(
                self.ancilla_slot_view(k).tile_attribute(),
                0x23 | 0x24 | 0x25 | 0x3b
            ) {
                self.ancilla_slot_view_mut(k).add_work_byte_24(1);
            }
        }
        self.ancilla_slot_view(k).work_byte_24() != 4
    }

    fn somaria_block_fizzle_away(&mut self, k: usize) {
        if self.player_state_view().speed_setting() == 18 {
            self.player_state_view_mut().clear_defense_flags();
            self.player_state_view_mut().set_speed_setting(0);
        }
        self.dungeon_state_view_mut()
            .clear_somaria_block_switch_counter();
        {
            let mut block = self.ancilla_slot_view_mut(k);
            block.set_ancilla_type(0x2d);
            block.set_aux_timer(0);
            block.set_step(0);
            block.set_item_to_link(0);
            block.set_work_byte_3(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_r(value);
        if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize {
            self.player_state_view_mut().clear_ancilla_pickup_flag();
            self.player_state_view_mut()
                .keep_only_lifting_or_carrying_state();
        }
        self.ancilla2_d_somaria_block_fizz(k);
    }

    fn ancilla_setup_basic_hit_box(&self, k: usize) -> SpriteHitBox {
        let x = self.ancilla_get_x(k).wrapping_sub(8);
        let y = self
            .ancilla_get_y(k)
            .wrapping_sub(8)
            .wrapping_sub(self.ancilla_slot_view(k).z() as u16);
        SpriteHitBox {
            r0_xlo: x as u8,
            r8_xhi: (x >> 8) as u8,
            r1_ylo: y as u8,
            r9_yhi: (y >> 8) as u8,
            r2: 15,
            r3: 15,
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        }
    }

    fn ancilla_setup_hit_box(&self, k: usize) -> SpriteHitBox {
        const ANCILLA_HIT_BOX_X: [i8; 12] = [4, 4, 4, 4, 3, 3, 2, 11, -16, -16, -1, -8];
        const ANCILLA_HIT_BOX_Y: [i8; 12] = [4, 4, 4, 4, 2, 11, 3, 3, -1, -8, -16, -16];
        const ANCILLA_HIT_BOX_W: [u8; 12] = [8, 8, 8, 8, 1, 1, 1, 1, 32, 32, 8, 8];
        const ANCILLA_HIT_BOX_H: [u8; 12] = [8, 8, 8, 8, 1, 1, 1, 1, 8, 8, 32, 32];
        let mut j = self.ancilla_slot_view(k).direction() as usize;
        if self.ancilla_slot_view(k).ancilla_type() == 0x0c {
            j |= 8;
        }
        let x = self
            .ancilla_get_x(k)
            .wrapping_add(ANCILLA_HIT_BOX_X[j] as i16 as u16);
        let y = self
            .ancilla_get_y(k)
            .wrapping_add(ANCILLA_HIT_BOX_Y[j] as i16 as u16);
        SpriteHitBox {
            r0_xlo: x as u8,
            r8_xhi: (x >> 8) as u8,
            r1_ylo: y as u8,
            r9_yhi: (y >> 8) as u8,
            r2: ANCILLA_HIT_BOX_W[j],
            r3: ANCILLA_HIT_BOX_H[j],
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        }
    }

    fn somarian_block_check_empty(&self, oam: usize) -> bool {
        for i in 0..4 {
            if self.oam_state_view().entry_y(oam + i * 4) == 0xf0 {
                continue;
            }
            for i in 0..4 {
                if self.oam_state_view().extended_byte((oam - OAM_BUF) / 4 + i) & 1 == 0 {
                    return false;
                }
            }
            break;
        }
        true
    }

    fn ancilla_prep_adjusted_oam_coord(&mut self, k: usize) -> (u16, u16) {
        const TAGALONG_LAYER_BITS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
        let floor = self.ancilla_slot_view(k).floor() as usize;
        self.oam_state_view_mut()
            .set_priority_word((TAGALONG_LAYER_BITS[floor] as u16) << 8);
        (
            self.ancilla_get_x(k)
                .wrapping_sub(self.ppu_scroll_copy_view().bg2_h_copy()),
            self.ancilla_get_y(k)
                .wrapping_sub(self.ppu_scroll_copy_view().bg2_v_copy()),
        )
    }

    fn ancilla_allocate_oam_from_region_b_or_e(&mut self, size: u8) {
        if !self.oam_state_view().has_sprite_sorting() {
            self.oam_allocate_from_region_b(size);
        } else {
            self.oam_allocate_from_region_e(size);
        }
    }

    fn ancilla_allocate_oam_from_custom_region(&mut self, oam: usize) -> usize {
        let mut a = oam;
        if self.oam_state_view().has_sprite_sorting() {
            if a < 0x900 {
                if a < 0x8e0 {
                    return oam;
                }
                a = 0x820;
            } else {
                if a < 0x9d0 {
                    return oam;
                }
                a = 0x940;
            }
        } else {
            if a < 0x990 {
                return oam;
            }
            a = 0x820;
        }
        self.oam_state_view_mut().set_current_pointer(a as u16);
        self.oam_state_view_mut()
            .set_current_extended_pointer((((a - 0x800) >> 2) + 0xa20) as u16);
        self.oam_state_view().current_pointer_usize()
    }

    fn hit_stars_update_oam_buffer_position(&mut self, oam: usize) -> usize {
        let mut oam = oam;
        if !self.oam_state_view().has_sprite_sorting() && oam >= 0x9d0 {
            self.oam_state_view_mut().set_current_pointer(0x820);
            self.oam_state_view_mut()
                .set_current_extended_pointer(0xa20 + (0x20 >> 2));
            oam = self.oam_state_view().current_pointer_usize();
        }
        oam
    }

    fn ancilla_check_for_entrance_trigger(&self, what: usize) -> bool {
        const ENTRANCE_TRIGGER_BASE_Y: [u16; 4] = [0x0d40, 0x0210, 0x0cfc, 0x0100];
        const ENTRANCE_TRIGGER_BASE_X: [u16; 4] = [0x0d80, 0x0e68, 0x0130, 0x0f10];
        const ENTRANCE_TRIGGER_SIZE_Y: [u16; 4] = [11, 32, 16, 12];
        const ENTRANCE_TRIGGER_SIZE_X: [u16; 4] = [16, 16, 16, 16];

        abs16(
            self.player_state_view()
                .y()
                .wrapping_add(12)
                .wrapping_sub(ENTRANCE_TRIGGER_BASE_Y[what]),
        ) < ENTRANCE_TRIGGER_SIZE_Y[what]
            && abs16(
                self.player_state_view()
                    .x()
                    .wrapping_add(8)
                    .wrapping_sub(ENTRANCE_TRIGGER_BASE_X[what]),
            ) < ENTRANCE_TRIGGER_SIZE_X[what]
    }

    fn game_over_text_draw(&mut self) {
        const GAME_OVER_TEXT_CHARS: [u8; 16] = [
            0x40, 0x50, 0x41, 0x51, 0x42, 0x52, 0x43, 0x53, 0x44, 0x54, 0x45, 0x55, 0x43, 0x53,
            0x46, 0x56,
        ];

        self.oam_state_view_mut().set_current_pointer(0x0800);
        self.oam_state_view_mut()
            .set_current_extended_pointer(0x0a20);
        let mut oam = self.oam_state_view().current_pointer_usize();
        let mut k = self.minigame_state_view().flag_boomerang_in_place() as i32;
        loop {
            let j = k as usize * 2;
            let x = self.ancilla_get_x(k as usize);
            self.ancilla_set_oam(oam, x, 0x57, GAME_OVER_TEXT_CHARS[j], 0x3c, 0);
            self.ancilla_set_oam(oam + 4, x, 0x5f, GAME_OVER_TEXT_CHARS[j + 1], 0x3c, 0);
            oam += 8;
            k -= 1;
            if k < 0 {
                break;
            }
        }
    }

    fn ancilla_draw_shadow(&mut self, oam: usize, k: usize, mut x: u16, y: u16, pal: u8) {
        const ANCILLA_DRAW_SHADOW_CHAR: [u8; 14] = [
            0x6c, 0x6c, 0x28, 0x28, 0x38, 0xff, 0xc8, 0xc8, 0xd8, 0xd8, 0xd9, 0xd9, 0xda, 0xda,
        ];
        const ANCILLA_DRAW_SHADOW_FLAGS: [u8; 14] = [
            0x28, 0x68, 0x28, 0x68, 0x28, 0xff, 0x22, 0x22, 0x24, 0x64, 0x24, 0x64, 0x24, 0x64,
        ];

        if k == 2 {
            x = x.wrapping_add(4);
        }
        self.ancilla_set_oam_safe(
            oam,
            x,
            y,
            ANCILLA_DRAW_SHADOW_CHAR[k * 2],
            ANCILLA_DRAW_SHADOW_FLAGS[k * 2] & !0x30 | pal,
            0,
        );
        let ch = ANCILLA_DRAW_SHADOW_CHAR[k * 2 + 1];
        if ch != 0xff {
            x = x.wrapping_add(8);
            self.ancilla_set_oam_safe(
                oam + 4,
                x,
                y,
                ch,
                ANCILLA_DRAW_SHADOW_FLAGS[k * 2 + 1] & !0x30 | pal,
                0,
            );
        }
    }

    fn sprite_spawn_dynamically_for_ancilla(&mut self, k: usize, sprite: u8) -> Option<usize> {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, sprite, &mut info);
        if j >= 0 {
            Some(j as usize)
        } else {
            None
        }
    }

    fn sprite_place_rupulse_spark_2_for_ancilla(&mut self, k: usize) {
        let x = self
            .sprite_get_x(k)
            .wrapping_sub(self.world_scroll().bg2_x());
        let y = self
            .sprite_get_y(k)
            .wrapping_sub(self.world_scroll().bg2_y());
        if x & !0xff != 0 || y & !0xff != 0 {
            return;
        }
        let spark_x = self.sprite_slot_view(k).x_low();
        let spark_y = self.sprite_slot_view(k).y_low();
        let spark_floor = self.sprite_slot_view(k).floor();
        self.garnish_state_view_mut().set_repulsespark_x_lo(spark_x);
        self.garnish_state_view_mut().set_repulsespark_y_lo(spark_y);
        self.garnish_state_view_mut().set_repulsespark_timer(5);
        self.garnish_state_view_mut()
            .set_repulsespark_floor_status(spark_floor);
    }

    fn sprite_place_weapon_tink_for_ancilla(&mut self, k: usize) {
        if self.garnish_state_view().repulsespark_timer() != 0 {
            return;
        }
        self.sprite_sfx_queue_sfx2_with_pan(k, 5);
        self.sprite_place_rupulse_spark_2_for_ancilla(k);
    }

    fn sprite_create_deflected_arrow(&mut self, k: usize) {
        let value = 0;
        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        if let Some(j) = self.sprite_spawn_dynamically_for_ancilla(k, 0x1b) {
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
            let value = self.player_state_view().lower_level_state();
            self.sprite_slot_view_mut(j).set_floor(value);
            self.sprite_place_weapon_tink_for_ancilla(j);
        }
    }

    fn ancilla_check_damage_to_sprite(&mut self, k: usize, ty: u8) {
        if !sign8(self.sprite_slot_view(k).hit_timer()) {
            self.ancilla_check_damage_to_sprite_aggressive(k, ty);
        }
    }

    fn ancilla_check_damage_to_sprite_aggressive(&mut self, k: usize, ty: u8) {
        const ANCILLA_DAMAGE: [u8; 57] = [
            6, 1, 11, 0, 0, 0, 0, 8, 0, 6, 0, 12, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 14, 13, 0, 0,
            15, 0, 0, 7, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 11, 0, 1, 1, 1, 1, 1, 1, 1,
            1,
        ];

        let mut dmg = ANCILLA_DAMAGE[ty as usize];
        if dmg == 6 && self.inventory_items().has_upgraded_bow() {
            if self.sprite_slot_view(k).sprite_type() == 0xd7 {
                let value = 32;
                self.sprite_slot_view_mut(k).set_delay_aux4(value);
            }
            dmg = 9;
        }
        self.ancilla_check_damage_to_sprite_preset(k, dmg);
    }

    fn medallion_check_sprite_damage(&mut self, k: usize) {
        let ancilla_type = self.ancilla_slot_view(k).ancilla_type();
        self.temp_counter_view_mut().set(ancilla_type);
        for j in (0..16).rev() {
            if self.sprite_slot_view(j).state() >= 9
                && (self.sprite_slot_view(j).ignore_projectile() | self.sprite_slot_view(j).pause())
                    == 0
            {
                self.ancilla_check_damage_to_sprite_aggressive(j, self.temp_counter_view().value());
            }
        }
    }

    fn sprite_func15_for_ancilla(&mut self, k: usize, a: u8) {
        self.sprite_battle_view_mut().set_damage_type_determiner(a);
        self.sprite_apply_calculated_damage_for_ancilla(k, if a == 8 { 0x35 } else { 0x20 });
    }

    fn sprite_apply_calculated_damage_for_ancilla(&mut self, k: usize, a: u8) {
        const ENEMY_DAMAGES: [u8; 128] = [
            0, 1, 32, 255, 252, 251, 0, 0, 0, 2, 64, 4, 0, 0, 0, 0, 0, 4, 64, 2, 3, 0, 0, 0, 0, 8,
            64, 4, 0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 4, 64, 16, 0,
            0, 0, 0, 0, 255, 64, 255, 252, 251, 0, 0, 0, 4, 64, 255, 252, 251, 32, 0, 0, 100, 24,
            100, 0, 0, 0, 0, 0, 249, 250, 255, 100, 0, 0, 0, 0, 8, 64, 253, 4, 16, 0, 0, 0, 8, 64,
            254, 4, 0, 0, 0, 0, 16, 64, 253, 0, 0, 0, 0, 0, 254, 64, 16, 0, 0, 0, 0, 0, 32, 64,
            255, 0, 0, 0, 250,
        ];

        if self.sprite_slot_view(k).flags3() & 0x40 != 0
            || self.sprite_slot_view(k).sprite_type() >= 0xd8
        {
            return;
        }
        let damage_type = self.sprite_battle_view().damage_type_determiner() as usize;
        let enemy_damage_index = self.sprite_slot_view(k).sprite_type() as usize * 16 + damage_type;
        let dmg = ENEMY_DAMAGES[damage_type * 8
            | self
                .enemy_damage_subclass_table_view()
                .entry(enemy_damage_index) as usize];
        self.sprite_give_damage_for_ancilla(k, dmg, a);
    }

    fn sprite_give_damage_for_ancilla(&mut self, k: usize, dmg: u8, r0_hit_timer: u8) {
        if dmg == 249 {
            self.sprite_func18_for_ancilla(k, 0xe3);
            return;
        }
        if dmg == 250 {
            self.sprite_func18_for_ancilla(k, 0x8f);
            let value = 2;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 32;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            let value = 8;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_f(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_health(value);
            let value = 1;
            self.sprite_slot_view_mut(k).set_bump_damage(value);
            let value = 1;
            self.sprite_slot_view_mut(k).set_flags5(value);
            return;
        }
        if dmg >= self.sprite_slot_view(k).incoming_damage() {
            self.sprite_slot_view_mut(k).set_incoming_damage(dmg);
        }
        if dmg == 0 {
            if self.sprite_battle_view().damage_type_determiner() != 10 {
                if self.sprite_slot_view(k).flags() & 4 != 0 {
                    self.sprite_set_damage_stun_for_ancilla(k);
                    return;
                }
                self.player_state_view_mut().clear_sword_delay_timer();
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            return;
        }
        if dmg >= 254 && self.sprite_slot_view(k).state() == 11 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            return;
        }
        if self.sprite_slot_view(k).sprite_type() == 0x9a
            && self.sprite_slot_view(k).incoming_damage() < 0xf0
        {
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 4;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 15;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
            return;
        }
        if self.sprite_slot_view(k).sprite_type() == 0x1b {
            self.sprite_sfx_queue_sfx2_with_pan(k, 5);
            self.sprite_schedule_for_breakage_for_ancilla(k);
            self.sprite_place_weapon_tink_for_ancilla(k);
            return;
        }
        let value = r0_hit_timer;
        self.sprite_slot_view_mut(k).set_hit_timer(value);
        if self.sprite_slot_view(k).sprite_type() != 0x92 || self.sprite_slot_view(k).c() >= 3 {
            let sfx = if self.sprite_slot_view(k).flags() & 2 != 0 {
                0x21
            } else if self.sprite_slot_view(k).flags5() & 0x10 != 0 {
                0x1c
            } else {
                8
            };
            self.set_sound_effect_2_with_sprite_pan(k, sfx);
        }
        self.sprite_set_damage_stun_for_ancilla(k);
    }

    fn sprite_set_damage_stun_for_ancilla(&mut self, k: usize) {
        let ty = self.sprite_slot_view(k).sprite_type();
        let value = if self.sprite_battle_view().damage_type_determiner() >= 13 {
            0
        } else if ty == 9 {
            20
        } else if ty == 0x53 || ty == 0x18 {
            11
        } else {
            15
        };
        self.sprite_slot_view_mut(k).set_f(value);
    }

    fn sprite_schedule_for_breakage_for_ancilla(&mut self, k: usize) {
        let value = 31;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let value = 6;
        self.sprite_slot_view_mut(k).set_state(value);
        self.sprite_slot_view_mut(k).add_flags2(4);
    }

    fn sprite_func18_for_ancilla(&mut self, k: usize, new_type: u8) {
        let value = new_type;
        self.sprite_slot_view_mut(k).set_sprite_type(value);
        self.sprite_prep_load_properties(k);
        self.system_signals_view_mut().set_sound_effect_2(0);
    }

    fn ancilla_check_sprite_collision(&mut self, k: usize) -> Option<usize> {
        for j in (0..16).rev() {
            if (self.ancilla_slot_view(k).ancilla_type() == 9
                || self.ancilla_slot_view(k).ancilla_type() == 0x1f
                || (((j as u8 ^ self.frame_state().frame_counter) & 3)
                    | self.sprite_slot_view(j).pause())
                    == 0)
                && self.sprite_slot_view(j).state() >= 9
                && (self.sprite_slot_view(j).deflection_bits() & 2 != 0
                    || self.ancilla_slot_view(k).object_priority() == 0)
                && self.ancilla_slot_view(k).floor() == self.sprite_slot_view(j).floor()
                && self.ancilla_check_sprite_collision_single(k, j)
            {
                return Some(j);
            }
        }
        None
    }

    fn ancilla_check_sprite_collision_single(&mut self, k: usize, j: usize) -> bool {
        let mut hb = self.ancilla_setup_hit_box(k);
        self.sprite_setup_hit_box(j, &mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if std::env::var_os("ZELDA3_TRACE_ANCILLA_COLL").is_some()
            && j == 2
            && self.frame_state().frame_counter >= 160
            && self.frame_state().frame_counter <= 210
        {
            eprintln!(
                "R ancilla-coll fc={} k={} atype=0x{:02x} j={} stype=0x{:02x} overlap={} ax={:04x} ay={:04x} az={:02x} hb={:02x}/{:02x} {:02x}/{:02x} size={:02x}/{:02x} spr={:02x}/{:02x} {:02x}/{:02x} ssize={:02x}/{:02x} dir={:02x} hit={:02x} pause={:02x} floor={:02x}/{:02x}",
                self.frame_state().frame_counter,
                k,
                self.ancilla_slot_view(k).ancilla_type(),
                j,
                self.sprite_slot_view(j).sprite_type(),
                overlap,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                self.ancilla_slot_view(k).z(),
                hb.r0_xlo,
                hb.r8_xhi,
                hb.r1_ylo,
                hb.r9_yhi,
                hb.r2,
                hb.r3,
                hb.r4_spr_xlo,
                hb.r10_spr_xhi,
                hb.r5_spr_ylo,
                hb.r11_spr_yhi,
                hb.r6_spr_xsize,
                hb.r7_spr_ysize,
                self.ancilla_slot_view(k).direction(),
                self.sprite_slot_view(j).hit_timer(),
                self.sprite_slot_view(j).pause(),
                self.ancilla_slot_view(k).floor(),
                self.sprite_slot_view(j).floor(),
            );
        }
        if !overlap {
            return false;
        }

        let mut return_value = true;
        if self.sprite_slot_view(j).flags() & 8 != 0
            && self.ancilla_slot_view(k).ancilla_type() == 9
        {
            if self.sprite_slot_view(j).sprite_type() != 0x1b {
                self.sprite_create_deflected_arrow(k);
                return false;
            }
            if !self.inventory_items().has_upgraded_bow() {
                self.sprite_create_deflected_arrow(k);
            } else {
                return_value = false;
            }
        }

        let mut return_true_set_alert = false;
        if self.sprite_slot_view(j).deflection_bits() & 0x10 != 0 {
            const ANCILLA_CHECK_SPRITE_COLL_DIR: [u8; 4] = [2, 3, 0, 1];
            self.ancilla_slot_view_mut(k).and_direction(3);
            if self.ancilla_slot_view(k).direction()
                == ANCILLA_CHECK_SPRITE_COLL_DIR[self.ancilla_slot_view(k).direction() as usize]
            {
                return_true_set_alert = true;
            }
        }

        if !return_true_set_alert
            && (self.ancilla_slot_view(k).ancilla_type() == 5
                || self.ancilla_slot_view(k).ancilla_type() == 0x1f)
        {
            let skip = self.ancilla_slot_view(k).ancilla_type() == 0x1f
                && self.sprite_slot_view(j).sprite_type() == 0x8d;
            if !skip && self.sprite_slot_view(j).hit_timer() != 0 {
                return_true_set_alert = true;
            } else if skip || self.sprite_slot_view(j).deflection_bits() & 2 != 0 {
                let value = k as u8 + 1;
                self.sprite_slot_view_mut(j).set_b(value);
                let value = self.ancilla_slot_view(k).ancilla_type();
                self.sprite_slot_view_mut(j).set_draw_work_byte_2(value);
                return_true_set_alert = true;
            }
        }

        if !return_true_set_alert && self.sprite_slot_view(j).ignore_projectile() == 0 {
            const ANCILLA_CHECK_SPRITE_COLL_RECOIL_X: [u8; 4] = [0, 0, 0xc0, 0x40];
            const ANCILLA_CHECK_SPRITE_COLL_RECOIL_Y: [u8; 4] = [0xc0, 0x40, 0, 0];
            if self.sprite_slot_view(j).sprite_type() == 0x92 && self.sprite_slot_view(j).c() < 3 {
                return_true_set_alert = true;
            } else {
                let i = (self.ancilla_slot_view(k).direction() & 3) as usize;
                let value = ANCILLA_CHECK_SPRITE_COLL_RECOIL_X[i];
                self.sprite_slot_view_mut(j).set_x_recoil(value);
                let value = ANCILLA_CHECK_SPRITE_COLL_RECOIL_Y[i];
                self.sprite_slot_view_mut(j).set_y_recoil(value);
                self.sprite_workspace_view_mut()
                    .set_shared_scratch_a(k as u8);
                self.ancilla_check_damage_to_sprite(j, self.ancilla_slot_view(k).ancilla_type());
                return_true_set_alert = true;
            }
        } else if !return_true_set_alert {
            return false;
        }

        if return_true_set_alert {
            let value = self.ancilla_slot_view(k).ancilla_type();
            self.sprite_slot_view_mut(j).set_draw_work_byte_2(value);
            self.sprite_system_view_mut().set_alert_flag(3);
            return return_value;
        }
        false
    }

    fn ancilla_check_basic_sprite_collision(&mut self, k: usize) -> Option<usize> {
        for j in (0..16).rev() {
            if (((j as u8 ^ self.frame_state().frame_counter) & 3)
                | self.sprite_slot_view(j).pause()
                | self.sprite_slot_view(j).hit_timer())
                != 0
            {
                continue;
            }
            if self.sprite_slot_view(j).state() < 9
                || (self.sprite_slot_view(j).deflection_bits() & 2 == 0
                    && self.ancilla_slot_view(k).object_priority() != 0)
                || self.ancilla_slot_view(k).floor() != self.sprite_slot_view(j).floor()
                || self.ancilla_slot_view(k).ancilla_type() == 0x2c
                    && (self.sprite_slot_view(j).sprite_type() == 0x1e
                        || self.sprite_slot_view(j).sprite_type() == 0x90)
            {
                continue;
            }
            if self.ancilla_check_basic_sprite_collision_single(k, j) {
                return Some(j);
            }
        }
        None
    }

    fn ancilla_check_basic_sprite_collision_single(&mut self, k: usize, j: usize) -> bool {
        let mut hb = self.ancilla_setup_basic_hit_box(k);
        self.sprite_setup_hit_box(j, &mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if std::env::var_os("ZELDA3_TRACE_ANCILLA_COLL").is_some()
            && j == 2
            && self.frame_state().frame_counter >= 160
            && self.frame_state().frame_counter <= 210
        {
            eprintln!(
                "R ancilla-basic-coll fc={} k={} atype=0x{:02x} j={} stype=0x{:02x} overlap={} ax={:04x} ay={:04x} az={:02x} hb={:02x}/{:02x} {:02x}/{:02x} size={:02x}/{:02x} spr={:02x}/{:02x} {:02x}/{:02x} ssize={:02x}/{:02x} dir={:02x} hit={:02x} pause={:02x} floor={:02x}/{:02x}",
                self.frame_state().frame_counter,
                k,
                self.ancilla_slot_view(k).ancilla_type(),
                j,
                self.sprite_slot_view(j).sprite_type(),
                overlap,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                self.ancilla_slot_view(k).z(),
                hb.r0_xlo,
                hb.r8_xhi,
                hb.r1_ylo,
                hb.r9_yhi,
                hb.r2,
                hb.r3,
                hb.r4_spr_xlo,
                hb.r10_spr_xhi,
                hb.r5_spr_ylo,
                hb.r11_spr_yhi,
                hb.r6_spr_xsize,
                hb.r7_spr_ysize,
                self.ancilla_slot_view(k).direction(),
                self.sprite_slot_view(j).hit_timer(),
                self.sprite_slot_view(j).pause(),
                self.ancilla_slot_view(k).floor(),
                self.sprite_slot_view(j).floor(),
            );
        }
        if !overlap {
            return false;
        }
        if self.sprite_slot_view(j).sprite_type() == 0x92 && self.sprite_slot_view(j).c() < 3 {
            return true;
        }
        if self.sprite_slot_view(j).sprite_type() == 0x80
            && self.sprite_slot_view(j).delay_aux4() == 0
        {
            let value = 24;
            self.sprite_slot_view_mut(j).set_delay_aux4(value);
            self.sprite_slot_view_mut(j).xor_direction(1);
        }
        if self.sprite_slot_view(j).ignore_projectile() != 0 {
            return false;
        }

        let x = self.ancilla_get_x(k).wrapping_sub(8);
        let y = self
            .ancilla_get_y(k)
            .wrapping_sub(8)
            .wrapping_sub(self.ancilla_slot_view(k).z() as u16);
        let pt = self.sprite_project_speed_towards_location(j, x, y, 80);
        let value = !pt.y;
        self.sprite_slot_view_mut(j).set_y_recoil(value);
        let value = !pt.x;
        self.sprite_slot_view_mut(j).set_x_recoil(value);
        self.ancilla_check_damage_to_sprite(j, self.ancilla_slot_view(k).ancilla_type());
        true
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
            if self.system_signals_view().sound_effect_1() & 0x3f == 0x0b
                || self.system_signals_view().sound_effect_1() & 0x3f == 0x21
            {
                self.set_sound_effect_1_with_ancilla_pan(k, 0x28);
            }
        } else if self.ancilla_slot_view(k).tile_attribute() == 0x40 {
            r10 = 3;
        }

        if self.ancilla_slot_view(k).z() >= 2 && self.ancilla_slot_view(k).z() < 252 {
            r10 = 2;
        }
        if k + 1 == self.player_state_view().ancilla_pickup_flag() as usize
            && self.player_state_view().is_lifting_or_carrying()
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
        const BOMB_DRAW_EXPLOSION_OFFSET: [SignedOffset; 54] = signed_offsets![
            -8, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -8, -8, -8, 0, 0, -8, 0, 0, 0, 0, 0, 0, -16, -16,
            -16, 0, 0, -16, 0, 0, 0, 0, 0, 0, -16, -16, -16, 0, 0, -16, 0, 0, 0, 0, 0, 0, -8, -8,
            -21, -22, -21, 8, 9, -22, 9, 8, 0, 0, -6, -15, 0, -1, -16, -2, -8, -7, 0, 0, 0, 0, -9,
            -4, -21, -5, -12, -18, -11, 7, 0, -15, 4, -2, -9, -4, -22, -5, -13, -20, -11, 8, 1,
            -16, 5, -2, -20, 4, -12, -19, -9, 16, -5, -2, 2, -9, 10, 6,
        ];
        const BOMB_DRAW_EXPLOSION_TILE: [OamTileAttrs; 54] = oam_tile_attrs![
            0x6e, 0x26, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x8c, 0x22,
            0x8c, 0x62, 0x8c, 0xa2, 0x8c, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x84, 0x22, 0x84, 0x62,
            0x84, 0xa2, 0x84, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x88, 0x22, 0x88, 0x62, 0x88, 0xa2,
            0x88, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x86, 0x22, 0x88, 0x22, 0x88, 0x62, 0x88, 0xa2,
            0x88, 0xe2, 0xff, 0xff, 0x86, 0x22, 0x86, 0x62, 0x86, 0xe2, 0x86, 0xe2, 0xff, 0xff,
            0xff, 0xff, 0x86, 0xe2, 0x86, 0x22, 0x86, 0x22, 0x86, 0x62, 0x86, 0xa2, 0x86, 0xa2,
            0x8a, 0xa2, 0x8a, 0x62, 0x8a, 0x22, 0x8a, 0x62, 0x8a, 0x62, 0x8a, 0xe2, 0x9b, 0x22,
            0x9b, 0xa2, 0x9b, 0x62, 0x9b, 0xe2, 0x9b, 0xa2, 0x9b, 0x22,
        ];
        const BOMB_DRAW_EXPLOSION_EXT: [u8; 54] = [
            2, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2,
            1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0,
        ];

        let base_frame = frame;
        loop {
            let tile = BOMB_DRAW_EXPLOSION_TILE[frame];
            if tile.char != 0xff {
                let i = idx + base_frame;
                let offset = BOMB_DRAW_EXPLOSION_OFFSET[i];
                self.ancilla_set_oam_safe(
                    oam,
                    x.wrapping_add(offset.x as i16 as u16),
                    y.wrapping_add(offset.y as i16 as u16),
                    tile.char,
                    tile.flags & !0x3e | self.oam_state_view().priority_high() | r11,
                    BOMB_DRAW_EXPLOSION_EXT[frame],
                );
                oam += 4;
            }
            frame += 1;
            idx += 1;
            if idx == idx_end {
                break;
            }
        }
        oam
    }

    fn bomb_draw(&mut self, k: usize) {
        let (x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        let z = self.ancilla_slot_view(k).z() as i8;
        if z != 0
            && z != -1
            && self.ancilla_slot_view(k).k() != 3
            && self.ancilla_slot_view(k).object_priority() != 0
        {
            self.oam_state_view_mut().set_priority_word(0x3000);
        }
        y = y.wrapping_sub(z as i16 as u16);
        let j =
            BOMB_DRAW_FRAME_STARTS[self.ancilla_slot_view(k).item_to_link() as usize] as usize * 6;

        let mut r11 = 2;
        if self.ancilla_slot_view(k).item_to_link() == 0 {
            r11 = if self.ancilla_slot_view(k).work_byte_3() < 0x20 {
                self.ancilla_slot_view(k).work_byte_3() & 0x0e
            } else {
                4
            };
        }

        if self.ancilla_slot_view(k).item_to_link() == 0 {
            if self.ancilla_slot_view(k).l() == 0
                && (self.sprite_slot_view(0).sprite_type() == 0x92
                    || k + 1 == self.player_state_view().ancilla_pickup_flag() as usize)
                && (!self.player_state_view().is_lifting_or_carrying()
                    || self.ancilla_slot_view(k).k() != 3 && self.player_state_view().facing() == 0)
            {
                self.ancilla_allocate_oam_from_region_b_or_e(12);
            } else if self.oam_state_view().has_sprite_sorting()
                && self.ancilla_slot_view(k).floor() != 0
                && (self.ancilla_slot_view(k).l() != 0
                    || k + 1 == self.player_state_view().ancilla_pickup_flag() as usize
                        && self.player_state_view().is_lifting_or_carrying())
            {
                self.oam_state_view_mut()
                    .set_current_pointer(0x0800 + 0x34 * 4);
                self.oam_state_view_mut()
                    .set_current_extended_pointer(0x0a20 + 0x34);
            }
        }

        let oam_org = self.oam_state_view().current_pointer_usize();
        let numframes =
            BOMB_DRAW_FRAME_COUNTS[self.ancilla_slot_view(k).item_to_link() as usize] as usize;
        let mut oam = oam_org;
        if self.ancilla_slot_view(k).item_to_link() == 0
            && (self.ancilla_slot_view(k).tile_attribute() == 9
                || self.ancilla_slot_view(k).tile_attribute() == 0x40)
        {
            oam += 8;
        }

        self.ancilla_draw_explosion(oam, j, 0, numframes, r11, x, y);
        oam += numframes * 4;

        let mut pt = Point16U { x, y };
        if let Some(r10) = self.bomb_check_underside_sprite_status(k, &mut pt) {
            if oam != oam_org + 4 {
                oam = oam_org;
            }
            self.ancilla_draw_shadow(
                oam,
                r10 as usize,
                pt.x,
                pt.y,
                self.oam_state_view().priority_high(),
            );
        }
    }

    fn ancilla32_blast_wall_fireball(&mut self, k: usize) {
        const BLAST_WALL_FIREBALL_CHAR: [u8; 3] = [0x9d, 0x9c, 0x8d];

        if self.frame_state().submodule == 0 {
            self.ancilla_slot_view_mut(k).add_item_to_link(2);
            let value = self
                .ancilla_slot_view(k)
                .y_velocity()
                .wrapping_add(self.ancilla_slot_view(k).item_to_link());
            self.ancilla_slot_view_mut(k).set_y_velocity(value);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            let timer = self.blast_wall_fireball_view_mut(k).tick_timer();
            if sign8(timer) {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            }
        }

        if self.oam_state_view().has_sprite_sorting() {
            self.oam_allocate_from_region_d(4);
        } else {
            self.oam_allocate_from_region_a(4);
        }

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let timer = self.blast_wall_fireball_view(k).timer();
        let j = if timer & 8 != 0 {
            0
        } else if timer & 4 != 0 {
            1
        } else {
            2
        };
        self.ancilla_set_oam(
            self.oam_state_view().current_pointer_usize(),
            x,
            y,
            BLAST_WALL_FIREBALL_CHAR[j],
            0x22,
            0,
        );
    }

    fn ancilla33_blast_wall_explosion(&mut self, k: usize) {
        if self.frame_state().submodule == 0 {
            if self.blast_wall_explosion_view(k).phase() != 0 {
                let timer = self.blast_wall_explosion_view_mut(k).tick_timer();
                if timer == 0 {
                    let phase = self.blast_wall_explosion_view_mut(k).advance_phase();
                    if phase != 0 && phase < 9 {
                        self.ancilla_add_blast_wall_fireball(0x32, 10, k * 4);
                    }
                    if phase == 11 {
                        self.blast_wall_explosion_view_mut(k).set_phase(0);
                        self.blast_wall_explosion_view_mut(k).set_timer(0);
                    } else {
                        self.blast_wall_explosion_view_mut(k).set_timer(3);
                    }
                }
            } else {
                let k = k ^ 1;
                if self.blast_wall_explosion_view(k).phase() == 6
                    && self.blast_wall_explosion_view(k).timer() == 2
                    && self.ancilla_slot_view(0).item_to_link().wrapping_add(1) < 7
                {
                    self.ancilla_slot_view_mut(0).advance_item_to_link();
                    self.blast_wall_explosion_view_mut(k).set_phase(1);
                    self.blast_wall_explosion_view_mut(k).set_timer(3);
                    for i in (0..=3).rev() {
                        let mut arr = [0i8, 0i8];
                        let j = if self.blast_wall_scratch_view().direction() < 4 {
                            1
                        } else {
                            0
                        };
                        arr[j] = if i & 2 != 0 { -13 } else { 13 };
                        let j = k * 4 + i;
                        let (x, _y) = self
                            .blast_wall_fragment_view_mut(j)
                            .offset(arr[1] as i16, arr[0] as i16);
                        let x = x.wrapping_sub(self.world_scroll().bg2_x());
                        if x < 256 {
                            self.system_signals_view_mut().set_sound_effect_1(
                                BOMBOS_PANNED_SFX_BITS[(x >> 5) as usize] | 0x0c,
                            );
                        }
                    }
                }
            }
        }

        let k = self.ancilla_slot_view(0).k() as usize;
        if self.blast_wall_explosion_view(k).phase() != 0 {
            let first_i = if k == 1 { 7 } else { 3 };
            for i in (first_i - 3..=first_i).rev() {
                self.ancilla_draw_blast_wall_blast(
                    k,
                    self.blast_wall_fragment_view(i).x(),
                    self.blast_wall_fragment_view(i).y(),
                );
            }
        }
        if self.ancilla_slot_view(0).item_to_link() == 6
            && self.blast_wall_explosion_view(0).phase() == 0
            && self.blast_wall_explosion_view(1).phase() == 0
        {
            self.ancilla_slot_view_mut(0).set_ancilla_type(0);
            let value = 0;
            self.ancilla_slot_view_mut(1).set_ancilla_type(value);
            self.player_state_view_mut().clear_custom_spell_animation();
        }
    }

    fn ancilla_draw_blast_wall_blast(&mut self, k: usize, x: u16, y: u16) {
        self.oam_state_view_mut().set_priority_word(0x3000);
        if self.oam_state_view().has_sprite_sorting() {
            self.oam_allocate_from_region_d(0x18);
        } else {
            self.oam_allocate_from_region_a(0x18);
        }
        let oam = self.oam_state_view().current_pointer_usize();
        let i = self.blast_wall_explosion_view(k).phase() as usize;
        self.ancilla_draw_explosion(
            oam,
            BOMB_DRAW_FRAME_STARTS[i] as usize * 6,
            0,
            BOMB_DRAW_FRAME_COUNTS[i] as usize,
            0x32,
            x.wrapping_sub(self.world_scroll().bg2_x()),
            y.wrapping_sub(self.world_scroll().bg2_y()),
        );
    }

    pub(crate) fn ancilla_calculate_sfx_pan(&self, k: usize) -> u8 {
        Self::calculate_sfx_pan_with_scroll(self.ancilla_get_x(k), self.world_scroll().bg2_x())
    }

    fn get_tile_attribute_for_ancilla(&mut self, floor: u8, mut x: u16, y: u16) -> u8 {
        let tiletype = if self.world_location_state().is_indoors() {
            let mut t = if floor >= 1 { 0x1000 } else { 0 };
            t += ((x & 0x01f8) >> 3) as usize;
            t += ((y & 0x01f8) << 3) as usize;
            self.dungeon_bg2_attributes().bg2_attr(t)
        } else {
            x >>= 3;
            self.overworld_get_tile_attribute_at_location(x, y)
        };
        self.sprite_workspace_view_mut().set_tile_type(tiletype);
        tiletype
    }

    fn entity_check_sloped_tile_collision_for_ancilla(&self, x: u16, y: u16) -> bool {
        let a = (y & 7) as u8;
        let r6 = self.sprite_workspace_view().tile_type().wrapping_sub(0x10);
        if r6 >= 4 {
            return true;
        }
        let b = SLOPED_TILE_HEIGHTS[(r6 as usize) * 8 + (x as usize & 7)];
        if r6 < 2 {
            b >= a
        } else {
            a >= b
        }
    }

    fn ancilla_set_oam(&mut self, oam: usize, x: u16, y: u16, charnum: u8, flags: u8, mut big: u8) {
        let mut yval = 0xf0;
        let xt: u16 = if self.enhanced_features_view().has(1) {
            0x40
        } else {
            0
        };
        if x.wrapping_add(xt) < 256 + xt * 2 && y < 256 {
            big |= ((x >> 8) as u8) & 1;
            self.oam_state_view_mut().set_entry_x(oam, x as u8);
            if y < 0xf0 {
                yval = y as u8;
            }
        }
        self.oam_state_view_mut().set_entry_y(oam, yval);
        self.oam_state_view_mut().set_entry_char(oam, charnum);
        self.oam_state_view_mut().set_entry_flags(oam, flags);
        let value = big;
        self.oam_state_view_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
    }

    fn ancilla_set_oam_plain(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.oam_state_view_mut()
            .write_entry(oam, x as u8, y as u8, charnum, flags);
        let value = big;
        self.oam_state_view_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
    }

    fn ancilla_set_oam_safe(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        mut big: u8,
    ) {
        let mut yval = 0xf0;
        self.oam_state_view_mut().set_entry_x(oam, x as u8);
        let xt: u16 = if self.enhanced_features_view().has(1) {
            0x48
        } else {
            0
        };
        if x.wrapping_add(0x80) < 0x180 + xt {
            big |= ((x >> 8) as u8) & 1;
            if y.wrapping_add(0x10) < 0x100 {
                yval = y as u8;
            }
        }
        self.oam_state_view_mut().set_entry_y(oam, yval);
        self.oam_state_view_mut().set_entry_char(oam, charnum);
        self.oam_state_view_mut().set_entry_flags(oam, flags);
        let value = big;
        self.oam_state_view_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
    }

    pub(super) fn ancilla_sfx2_pan(&mut self, k: usize, sfx: u8) {
        self.system_signals_view_mut().set_raw_sfx_pan_value(sfx);
        let out = sfx | self.ancilla_calculate_sfx_pan(k);
        self.system_signals_view_mut().set_sound_effect_1(out);
        self.replay_trace_sfx("ancilla_sfx2_pan", Some(k), sfx, out);
    }

    pub(super) fn ancilla_sfx1_pan(&mut self, k: usize, sfx: u8) {
        self.system_signals_view_mut().set_raw_sfx_pan_value(sfx);
        let out = sfx | self.ancilla_calculate_sfx_pan(k);
        self.system_signals_view_mut().set_ambient_sound_effect(out);
        self.replay_trace_sfx("ancilla_sfx1_pan", Some(k), sfx, out);
    }

    pub(super) fn ancilla_sfx3_pan(&mut self, k: usize, sfx: u8) {
        self.system_signals_view_mut().set_raw_sfx_pan_value(sfx);
        let out = sfx | self.ancilla_calculate_sfx_pan(k);
        self.system_signals_view_mut().set_sound_effect_2(out);
        self.replay_trace_sfx("ancilla_sfx3_pan", Some(k), sfx, out);
    }

    pub(super) fn ancilla_set_xy(&mut self, k: usize, x: u16, y: u16) {
        self.ancilla_set_x(k, x);
        self.ancilla_set_y(k, y);
    }

    fn dash_tremor_twiddle_offset(&mut self, k: usize) -> i32 {
        let j = self.ancilla_slot_view(k).direction();
        let y = 0u16.wrapping_sub(self.ancilla_get_y(k));
        self.ancilla_set_y(k, y);
        if self.world_location_state().is_indoors() {
            return y as i32;
        }
        if j == 2 {
            let start = self.room_bounds_view().packed_top().wrapping_add(1);
            let end = self.room_bounds_view().packed_bottom().wrapping_sub(1);
            let a = y.wrapping_add(self.world_scroll().bg2_y());
            if a <= start || a >= end {
                0
            } else {
                y as i32
            }
        } else {
            let start = self.room_bounds_view().packed_left().wrapping_add(1);
            let end = self.room_bounds_view().packed_right().wrapping_sub(1);
            let a = y.wrapping_add(self.world_scroll().bg2_x());
            if a <= start || a >= end {
                0
            } else {
                y as i32
            }
        }
    }

    fn ancilla_set_x(&mut self, k: usize, x: u16) {
        self.ancilla_slot_view_mut(k).set_x(x);
    }

    fn ancilla_set_y(&mut self, k: usize, y: u16) {
        self.ancilla_slot_view_mut(k).set_y(y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dash_dust_motive_expires_out_of_range_frame() {
        let mut state = ZeldaState::new();
        state.ancilla_slot_view_mut(0).set_ancilla_type(0x1e);
        state.ram[ANCILLA_TIMER] = 1;
        state.ram[ANCILLA_ITEM_TO_LINK] = 3;

        state.dash_dust_motive(0);

        assert_eq!(state.ancilla_slot_view(0).ancilla_type(), 0);
    }
}
