// Methods ported from zelda3/src/ending.c and included inside ZeldaState.

use super::sprite::{DrawMultipleData, PrepOamCoordsRet};
use super::*;
use crate::types::sign8;
use crate::zelda_rtl::misc::DUNG_ANIMATED_TILES;
use crate::zelda_rtl::sprite::SpriteSpawnInfo;

const K_POLYHEDRAL_PALETTE: [u16; 8] = [0, 0x14d, 0x1b0, 0x1f3, 0x256, 0x279, 0x2fd, 0x35f];
const K_FEATURES0_DIM_FLASHES_ENDING: u32 = 65536;
const K_ENDING_TAB1: [u16; 16] = [
    0x1000, 2, 0x1002, 0x1012, 0x1004, 0x1006, 0x1010, 0x1014, 0x100a, 0x1016, 0x5d, 0x64, 0x100e,
    0x1008, 0x1018, 0x180,
];
const K_ENDING_SPRITE_PACK: [u8; 17] = [
    0x28, 0x46, 0x27, 0x2e, 0x2b, 0x2b, 0xe, 0x2c, 0x1a, 0x29, 0x47, 0x28, 0x27, 0x28, 0x2a, 0x28,
    0x2d,
];
const K_ENDING_SPRITE_PAL: [u8; 17] = [
    1, 0x40, 1, 4, 1, 1, 1, 0x11, 1, 1, 0x47, 0x40, 1, 1, 1, 1, 1,
];
const K_ENDING1_TARGET_SCROLL_Y: [u16; 16] = [
    0x6f2, 0x210, 0x72c, 0xc00, 0x10c, 0xa9b, 0x10, 0x510, 0x89, 0xa8e, 0x222c, 0x2510, 0x826,
    0x5c, 0x20a, 0x30,
];
const K_ENDING1_TARGET_SCROLL_X: [u16; 16] = [
    0x77f, 0x480, 0x193, 0xaa, 0x878, 0x847, 0x4fd, 0xc57, 0x40f, 0x478, 0xa00, 0x200, 0x201,
    0xaa1, 0x26f, 0,
];
const K_ENDING1_YVEL: [i8; 16] = [-1, -1, 1, -1, 1, 1, 0, 1, 0, -1, -1, 0, 0, 0, 1, -1];
const K_ENDING1_XVEL: [i8; 16] = [0, 0, -1, 0, 0, -1, 1, 0, -1, 0, 0, 0, 1, -1, 1, 0];
const INTRO_WANT_DOUBLE_RET: usize = 0x1e02;
const TRIFORCE_CTR: usize = 0x1e0c;
const ENDING_WHICH_DUNG: usize = 0x0cc;
const ENDING_CREDIT_DIGIT_CHAR: usize = 0x0ce;
const MAPBAK_PALETTE: usize = 0x1dd80;
const SPRITE_B_ENDING: usize = 0x0da0;
const SPRITE_C: usize = 0x0db0;
const SPRITE_DELAY_AUX1: usize = 0x0e00;
const OAM_REGION_BASE: usize = 0x0fe0;
const BG2HOFS: usize = 0x210f;
const FLAG_WHICH_MUSIC_TYPE: usize = 0x136;
const OVERLAY_INDEX: usize = 0x08c;
const OVERWORLD_SCROLL_UP_COUNTER: usize = 0x624;
const OVERWORLD_SCROLL_DOWN_COUNTER: usize = 0x626;
const OVERWORLD_SCROLL_LEFT_COUNTER: usize = 0x628;
const OVERWORLD_SCROLL_RIGHT_COUNTER: usize = 0x62a;
const OVERWORLD_AREA_INDEX: usize = 0x40a;
const DEATHS_PER_PALACE: usize = 0xf3e7;
const DEATH_SAVE_COUNTER: usize = 0xf403;
const POLY_THREAD_RAM_START: usize = 0x1f00;
const POLY_THREAD_RAM_LEN: usize = 0x100;
const POLY_THREAD_INIT_BYTES: usize = 0x1f32;
const DEATH_VAR2: usize = 0xf405;
type IntroSpriteEnt = (i8, i8, u8, u8, u8);

const K_ENDING_SPRITES_X: [u16; 85] = [
    0x1e0, 0x200, 0x1ed, 0x203, 0x1da, 0x216, 0x1c8, 0x228, 0x1c0, 0x1e0, 0x208, 0x228, 0xf8, 0xf0,
    0x278, 0x298, 0x1e0, 0x200, 0x220, 0x288, 0x1e2, 0xe0, 0x150, 0xe8, 0x168, 0x128, 0x170, 0x170,
    0x335, 0x335, 0x300, 0xb8, 0xce, 0xac, 0xc4, 0x3b0, 0x390, 0x3d0, 0xf8, 0xc8, 0x80, 0xf8, 0xf8,
    0xf8, 0xf8, 0xf8, 0xe8, 0xf8, 0xd8, 0xf8, 0xc8, 0x108, 0x70, 0x70, 0x70, 0x68, 0x88, 0x70,
    0x40, 0x70, 0x4f, 0x61, 0x37, 0x79, 0xc8, 0x278, 0x258, 0x1d8, 0x1c8, 0x188, 0x270, 0x180,
    0x2e8, 0x270, 0x270, 0x2a0, 0x2a0, 0x2a4, 0x2fc, 0x76, 0x73, 0x76, 0x0, 0xd0, 0x80,
];
const K_ENDING_SPRITES_Y: [u16; 85] = [
    0x158, 0x158, 0x138, 0x138, 0x140, 0x140, 0x150, 0x150, 0x120, 0x120, 0x120, 0x120, 0x60, 0x37,
    0xc2, 0xc2, 0x16b, 0x16c, 0x16b, 0xb8, 0x16b, 0x80, 0x60, 0x146, 0x146, 0x1c6, 0x70, 0x70,
    0x128, 0x128, 0x16f, 0xf5, 0xfc, 0x10d, 0x10d, 0x40, 0x40, 0x40, 0x150, 0x158, 0xf4, 0x120,
    0x120, 0x120, 0x120, 0x120, 0x108, 0x100, 0xd8, 0xd8, 0xf0, 0xf0, 0x3c, 0x3c, 0x3c, 0x90, 0x80,
    0x3c, 0x16c, 0x16c, 0x174, 0x174, 0x175, 0x175, 0x250, 0x2b0, 0x2b0, 0x2a0, 0x2b0, 0x2b0,
    0x2b8, 0xd8, 0x24b, 0x1b0, 0x1c8, 0x1c8, 0x1b0, 0x230, 0x230, 0x8b, 0x83, 0x85, 0x2c, 0xf8,
    0x100,
];
const K_ENDING_SPRITES_IDX: [usize; 17] = [
    0, 12, 14, 21, 28, 31, 35, 38, 40, 41, 52, 58, 64, 71, 72, 79, 85,
];

type DrawMultipleDataEnding = (i8, i8, u16, u8);

const K_END_SEQUENCE_DMD0: [DrawMultipleDataEnding; 12] = [
    (0, -8, 0x072a, 2),
    (0, -8, 0x072a, 2),
    (0, 0, 0x4fca, 2),
    (0, -8, 0x072a, 2),
    (0, -8, 0x072a, 2),
    (0, 0, 0x0fca, 2),
    (-2, 0, 0x0f77, 0),
    (0, -8, 0x072a, 2),
    (0, 0, 0x4fca, 2),
    (-3, 0, 0x0f66, 0),
    (0, -8, 0x072a, 2),
    (0, 0, 0x4fca, 2),
];

const K_END_SEQUENCE_DMD1: [DrawMultipleDataEnding; 6] = [
    (14, -7, 0x0d48, 2),
    (0, -6, 0x0944, 2),
    (0, 0, 0x094e, 2),
    (13, -14, 0x0d48, 2),
    (0, -8, 0x0944, 2),
    (0, 0, 0x0946, 2),
];

const K_END_SEQUENCE_DMD2: [DrawMultipleDataEnding; 16] = [
    (-2, -16, 0x3d78, 0),
    (0, -24, 0x3d24, 2),
    (0, -16, 0x3dc2, 2),
    (61, -16, 0x3777, 0),
    (64, -24, 0x37c4, 2),
    (64, -16, 0x77ca, 2),
    (0, -6, 0x326c, 2),
    (64, -6, 0x326c, 2),
    (-2, -16, 0x3d68, 0),
    (0, -24, 0x3d24, 2),
    (0, -16, 0x3dc2, 2),
    (61, -16, 0x3766, 0),
    (64, -24, 0x37c4, 2),
    (64, -16, 0x77ca, 2),
    (0, -6, 0x326c, 2),
    (64, -6, 0x326c, 2),
];

const K_END_SEQUENCE_DMD3: [DrawMultipleDataEnding; 12] = [
    (0, 0, 0x0022, 2),
    (48, 0, 0x0064, 2),
    (0, 10, 0x016c, 2),
    (48, 10, 0x016c, 2),
    (0, 0, 0x0064, 2),
    (48, 0, 0x0022, 2),
    (0, 10, 0x016c, 2),
    (48, 10, 0x016c, 2),
    (0, 0, 0x0064, 2),
    (48, 0, 0x0064, 2),
    (0, 10, 0x016c, 2),
    (48, 10, 0x016c, 2),
];

const K_END_SEQUENCE_DMD4: [DrawMultipleDataEnding; 8] = [
    (10, 8, 0x8a32, 0),
    (10, 16, 0x8a22, 0),
    (0, -10, 0x0800, 2),
    (0, 0, 0x082c, 2),
    (10, -14, 0x0a22, 0),
    (10, -6, 0x0a32, 0),
    (0, -10, 0x082a, 2),
    (0, 0, 0x0828, 2),
];

const K_END_SEQUENCE_DMD5: [DrawMultipleDataEnding; 10] = [
    (10, 16, 0x8a05, 0),
    (10, 8, 0x8a15, 0),
    (-4, 2, 0x0a07, 2),
    (0, -7, 0x0e00, 2),
    (0, 1, 0x0e02, 2),
    (10, -20, 0x0a05, 0),
    (10, -12, 0x0a15, 0),
    (-7, 1, 0x4a07, 2),
    (0, -7, 0x0e00, 2),
    (0, 1, 0x0e02, 2),
];

const K_END_SEQUENCE_DMD6: [DrawMultipleDataEnding; 3] =
    [(-6, -2, 0x0706, 2), (0, -9, 0x090e, 2), (0, -1, 0x0908, 2)];

const K_END_SEQUENCE_DMD7: [DrawMultipleDataEnding; 10] = [
    (0, -10, 0x082a, 2),
    (0, 0, 0x0828, 2),
    (10, 16, 0x8a05, 0),
    (10, 8, 0x8a15, 0),
    (-4, 2, 0x0a07, 2),
    (0, -7, 0x0e00, 2),
    (0, 1, 0x0e02, 2),
    (10, -20, 0x0a05, 0),
    (10, -12, 0x0a15, 0),
    (-7, 1, 0x4a07, 2),
];

const K_END_SEQUENCE_DMD8: [DrawMultipleDataEnding; 1] = [(0, -19, 0x39af, 0)];

const K_END_SEQUENCE_DMD9: [DrawMultipleDataEnding; 4] = [
    (-16, -24, 0x3704, 2),
    (-16, -16, 0x3764, 2),
    (-16, -24, 0x3762, 2),
    (-16, -16, 0x3764, 2),
];

const K_END_SEQUENCE_DMD10: [DrawMultipleDataEnding; 4] = [
    (0, 0, 0x0c0c, 2),
    (0, 0, 0x0c0a, 2),
    (0, 0, 0x0cc5, 2),
    (0, 0, 0x0ce1, 2),
];

const K_END_SEQUENCE_DMD11: [DrawMultipleDataEnding; 6] = [
    (1, 4, 0x002a, 0),
    (1, 12, 0x003a, 0),
    (4, 0, 0x0026, 2),
    (0, 9, 0x0024, 2),
    (8, 9, 0x4024, 2),
    (4, 20, 0x016c, 2),
];

const K_END_SEQUENCE_DMD12: [DrawMultipleDataEnding; 21] = [
    (0, -7, 0x0d00, 2),
    (0, -7, 0x0d00, 2),
    (0, 0, 0x0d06, 2),
    (0, -7, 0x0d00, 2),
    (0, -7, 0x0d00, 2),
    (0, 0, 0x4d06, 2),
    (0, -8, 0x0d00, 2),
    (0, -8, 0x0d00, 2),
    (0, 0, 0x0d20, 2),
    (0, -8, 0x0d02, 2),
    (0, -8, 0x0d02, 2),
    (0, 0, 0x0d2c, 2),
    (-3, 0, 0x0d2f, 0),
    (0, -7, 0x0d02, 2),
    (0, 0, 0x0d2c, 2),
    (-5, 2, 0x0d2f, 0),
    (0, -8, 0x0d02, 2),
    (0, 0, 0x0d2c, 2),
    (-5, 2, 0x0d3f, 0),
    (0, -8, 0x0d02, 2),
    (0, 0, 0x0d2c, 2),
];

const K_END_SEQUENCE_DMD13: [DrawMultipleDataEnding; 16] = [
    (0, -7, 0x0e00, 2),
    (0, 1, 0x4e02, 2),
    (0, -8, 0x0e00, 2),
    (0, 1, 0x0e02, 2),
    (0, -9, 0x0e00, 2),
    (0, 1, 0x0e02, 2),
    (0, -7, 0x0e00, 2),
    (0, 1, 0x0e02, 2),
    (0, -7, 0x0e00, 2),
    (0, 1, 0x4e02, 2),
    (0, -8, 0x0e00, 2),
    (0, 1, 0x4e02, 2),
    (0, -9, 0x0e00, 2),
    (0, 1, 0x4e02, 2),
    (0, -7, 0x0e00, 2),
    (0, 1, 0x4e02, 2),
];

const K_END_SEQUENCE_DMD14: [DrawMultipleDataEnding; 6] = [
    (0, 0, 0, 0),
    (0, 0, 0x34c7, 0),
    (0, 0, 0x3480, 0),
    (0, 0, 0x34b6, 0),
    (0, 0, 0x34b7, 0),
    (0, 0, 0x34a6, 0),
];

const K_END_SEQUENCE_DMD15: [DrawMultipleDataEnding; 6] = [
    (-3, 17, 0x002b, 0),
    (-3, 25, 0x003b, 0),
    (0, 0, 0x000e, 2),
    (16, 0, 0x400e, 2),
    (0, 16, 0x002e, 2),
    (16, 16, 0x402e, 2),
];

const K_END_SEQUENCE_DMD16: [DrawMultipleDataEnding; 3] =
    [(8, 5, 0x0a04, 2), (0, 16, 0x0806, 2), (16, 16, 0x4806, 2)];

const K_END_SEQUENCE_DMD17: [DrawMultipleDataEnding; 2] = [(0, 0, 0x0000, 2), (0, 11, 0x0002, 2)];

const K_END_SEQUENCE_DMD18: [DrawMultipleDataEnding; 2] = [(0, 0, 0x000e, 2), (0, 64, 0x006c, 2)];

const K_END_SEQUENCE_DMD19: [DrawMultipleDataEnding; 8] = [
    (0, 0, 0x0882, 2),
    (0, 7, 0x0a4e, 2),
    (0, 0, 0x4880, 2),
    (0, 7, 0x0a4e, 2),
    (0, 0, 0x0882, 2),
    (0, 7, 0x0a4e, 2),
    (0, 0, 0x0880, 2),
    (0, 7, 0x0a4e, 2),
];

const K_END_SEQUENCE_DMD20: [DrawMultipleDataEnding; 6] = [
    (-4, 1, 0x0c68, 0),
    (0, -8, 0x0c40, 2),
    (0, 1, 0x0c42, 2),
    (-4, 1, 0x0c78, 0),
    (0, -8, 0x0c40, 2),
    (0, 1, 0x0c42, 2),
];

const K_END_SEQUENCE_DMD21: [DrawMultipleDataEnding; 6] = [
    (8, 5, 0x0679, 0),
    (0, -10, 0x088e, 2),
    (0, 0, 0x066e, 2),
    (0, -10, 0x088e, 2),
    (0, -10, 0x088e, 2),
    (0, 0, 0x066e, 2),
];

const K_END_SEQUENCE_DMD22: [DrawMultipleDataEnding; 6] = [
    (11, -3, 0x0869, 0),
    (0, -12, 0x0804, 2),
    (0, 0, 0x0860, 2),
    (10, -3, 0x0867, 0),
    (0, -12, 0x0804, 2),
    (0, 0, 0x0860, 2),
];

const K_END_SEQUENCE_DMD23: [DrawMultipleDataEnding; 6] = [
    (-2, 1, 0x0868, 0),
    (0, -8, 0x08c0, 2),
    (0, 0, 0x08c2, 2),
    (-3, 1, 0x0878, 0),
    (0, -8, 0x08c0, 2),
    (0, 0, 0x08c2, 2),
];

const K_END_SEQUENCE_DMD24: [DrawMultipleDataEnding; 4] = [
    (0, -10, 0x084c, 2),
    (0, 0, 0x0a6c, 2),
    (0, -9, 0x084c, 2),
    (0, 0, 0x0aa8, 2),
];

const K_END_SEQUENCE_DMD25: [DrawMultipleDataEnding; 4] = [
    (0, -7, 0x084a, 2),
    (0, 0, 0x0c6a, 2),
    (0, -7, 0x084a, 2),
    (0, 0, 0x0ca6, 2),
];

const K_END_SEQUENCE_DMD26: [DrawMultipleDataEnding; 12] = [
    (-18, -24, 0x39a4, 2),
    (-16, -16, 0x39a8, 2),
    (-18, -24, 0x39a4, 2),
    (-18, -24, 0x39a4, 2),
    (-16, -16, 0x39a6, 2),
    (-18, -24, 0x39a4, 2),
    (-6, -17, 0x392d, 0),
    (-16, -24, 0x39a0, 2),
    (-16, -16, 0x39aa, 2),
    (-5, -17, 0x392c, 0),
    (-16, -24, 0x39a0, 2),
    (-16, -16, 0x39aa, 2),
];

const K_END_SEQUENCE_DMD27: [DrawMultipleDataEnding; 6] = [
    (0, -4, 0x30aa, 2),
    (0, -4, 0x30aa, 2),
    (-4, -8, 0x3090, 0),
    (12, -8, 0x7090, 0),
    (-6, -10, 0x3091, 0),
    (14, -10, 0x7091, 0),
];

const K_END_SEQUENCE_DMD28: [DrawMultipleDataEnding; 8] = [
    (0, 0, 0x0722, 2),
    (0, -8, 0x09c2, 2),
    (0, 0, 0x4722, 2),
    (0, -8, 0x09c2, 2),
    (0, -9, 0x09c4, 2),
    (0, 0, 0x0722, 2),
    (0, -9, 0x0924, 2),
    (0, 0, 0x0722, 2),
];

const K_END_SEQUENCE_DMD29: [DrawMultipleDataEnding; 3] = [
    (-16, -12, 0x3f08, 2),
    (0, -12, 0x3f20, 2),
    (16, -12, 0x3f20, 2),
];

const K_END_SEQUENCE_DMD30: [DrawMultipleDataEnding; 1] = [(0, 0, 0x0086, 2)];

const K_END_SEQUENCE_DMD31: [DrawMultipleDataEnding; 1] = [(0, 0, 0x8060, 2)];

const K_END_SEQUENCE_DMDS: [&[DrawMultipleDataEnding]; 32] = [
    &K_END_SEQUENCE_DMD0,
    &K_END_SEQUENCE_DMD1,
    &K_END_SEQUENCE_DMD2,
    &K_END_SEQUENCE_DMD3,
    &K_END_SEQUENCE_DMD4,
    &K_END_SEQUENCE_DMD5,
    &K_END_SEQUENCE_DMD6,
    &K_END_SEQUENCE_DMD7,
    &K_END_SEQUENCE_DMD8,
    &K_END_SEQUENCE_DMD9,
    &K_END_SEQUENCE_DMD10,
    &K_END_SEQUENCE_DMD11,
    &K_END_SEQUENCE_DMD12,
    &K_END_SEQUENCE_DMD13,
    &K_END_SEQUENCE_DMD14,
    &K_END_SEQUENCE_DMD15,
    &K_END_SEQUENCE_DMD16,
    &K_END_SEQUENCE_DMD17,
    &K_END_SEQUENCE_DMD18,
    &K_END_SEQUENCE_DMD19,
    &K_END_SEQUENCE_DMD20,
    &K_END_SEQUENCE_DMD21,
    &K_END_SEQUENCE_DMD22,
    &K_END_SEQUENCE_DMD23,
    &K_END_SEQUENCE_DMD24,
    &K_END_SEQUENCE_DMD25,
    &K_END_SEQUENCE_DMD26,
    &K_END_SEQUENCE_DMD27,
    &K_END_SEQUENCE_DMD28,
    &K_END_SEQUENCE_DMD29,
    &K_END_SEQUENCE_DMD30,
    &K_END_SEQUENCE_DMD31,
];

const DUNG_PAL_INFOS_ENDING: [(u8, u8, u8, u8); 41] = [
    (0, 0, 3, 1),
    (2, 0, 3, 1),
    (4, 0, 10, 1),
    (6, 0, 1, 7),
    (10, 2, 2, 7),
    (4, 4, 3, 10),
    (12, 5, 8, 20),
    (14, 0, 3, 10),
    (2, 0, 15, 20),
    (10, 2, 0, 7),
    (2, 0, 15, 12),
    (6, 0, 6, 7),
    (0, 0, 14, 18),
    (18, 5, 5, 11),
    (18, 0, 2, 12),
    (16, 5, 10, 7),
    (16, 0, 16, 12),
    (22, 7, 2, 7),
    (22, 0, 7, 15),
    (8, 0, 4, 12),
    (8, 0, 4, 9),
    (4, 0, 3, 1),
    (20, 0, 4, 4),
    (20, 0, 20, 12),
    (24, 5, 7, 11),
    (24, 6, 16, 12),
    (26, 5, 8, 20),
    (26, 2, 0, 7),
    (6, 0, 3, 10),
    (28, 0, 3, 1),
    (30, 0, 11, 17),
    (4, 0, 11, 17),
    (14, 0, 0, 2),
    (32, 8, 19, 13),
    (10, 0, 3, 10),
    (20, 0, 4, 4),
    (26, 2, 2, 7),
    (26, 10, 0, 0),
    (0, 0, 3, 2),
    (14, 0, 3, 7),
    (26, 5, 5, 11),
];

impl ZeldaState {
    fn apply_dung_pal_info(&mut self, idx: u8) {
        let (_, _, pal2, pal3) = DUNG_PAL_INFOS_ENDING[idx as usize];
        self.ram[PALETTE_SP5L] = pal2;
        self.ram[PALETTE_SP6L] = pal3;
    }

    fn CallForDuckIndoors(&mut self) {
        self.call_for_duck_indoors();
    }

    fn Sprite_SpawnBatCrashCutscene(&mut self) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(0, 0x37, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.ram[SPRITE_Y_VEL + j] = 0;
            self.ram[SPRITE_B_ENDING + j] = 0;
            self.ram[SPRITE_D + j] = 0;
            self.ram[SPRITE_FLOOR + j] = 0;
            self.ram[SPRITE_SUBTYPE2 + j] = 1;
            self.ram[SPRITE_FLAGS2 + j] = 1;
            self.ram[SPRITE_FLAGS3 + j] = 1;
            self.ram[SPRITE_OAM_FLAGS + j] = 1;
            self.ram[SPRITE_X_LO + j] = 204;
            self.ram[SPRITE_X_HI + j] = 7;
            self.ram[SPRITE_Y_LO + j] = 50;
            self.ram[SPRITE_Y_HI + j] = 6;
            self.ram[SPRITE_DEFL_BITS + j] = 128;
        }
    }

    fn sprite_get_16_bit_coords_ending(&mut self, k: usize) {
        self.sprite_get16_bit_coords(k);
    }

    fn sprite_active_main_ending(&mut self, k: usize) {
        self.sprite_active_main(k);
    }

    fn ending_asset_u16(&self, asset: usize, index: usize) -> u16 {
        let data = self
            .asset_raw(asset)
            .unwrap_or_else(|| panic!("missing ending asset {asset}"));
        read_word_from_slice(data, index * 2)
    }

    fn set_oam_helper0_addr(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.ram[oam] = x as u8;
        self.ram[oam + 1] = if y.wrapping_add(0x10) < 0x100 {
            y as u8
        } else {
            0xf0
        };
        self.ram[oam + 2] = charnum;
        self.ram[oam + 3] = flags;
        self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4] = big | ((x >> 8) as u8 & 1);
    }

    pub(super) fn Intro_SetupScreen(&mut self) {
        self.intro_setup_screen();
    }

    pub(super) fn Intro_LoadTextPointersAndPalettes(&mut self) {
        self.intro_load_text_pointers_and_palettes();
    }

    pub(super) fn credits_load_scene_overworld_prep_gfx(&mut self) {
        self.enable_force_blank();
        self.erase_tile_maps_normal();
        self.ram[CGWSEL_COPY] = 0x82;
        let k = (self.frame_control_view().submodule() >> 1) as usize;
        write_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX, K_ENDING_TAB1[k]);
        if k != 6 && k != 15 {
            self.LoadOverworldFromDungeon();
        } else {
            self.Overworld_EnterSpecialArea();
        }
        self.ram[MUSIC_CONTROL] = 0;
        self.ram[SOUND_EFFECT_AMBIENT] = 0;
        let t = self.ram[OVERWORLD_SCREEN_INDEX] & !0x40;
        self.DecompressAnimatedOverworldTiles(if t == 3 || t == 5 || t == 7 {
            0x58
        } else {
            0x5a
        });
        let k = (self.frame_control_view().submodule() >> 1) as usize;
        self.ram[SPRITE_GRAPHICS_INDEX] = K_ENDING_SPRITE_PACK[k];
        let sprpal = K_ENDING_SPRITE_PAL[k];
        self.ram[HUD_PALETTE] = 1;
        self.initialize_tilesets();
        self.OverworldLoadScreensPaletteSet();
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(self.ram[OVERWORLD_SCREEN_INDEX]),
            sprpal,
        );
        self.palette_load_hud();
        if self.frame_control_view().submodule() == 0 {
            self.TransferFontToVRAM();
        }
        self.overworld_load_palettes_inner();
        self.Overworld_SetFixedColAndScroll();
        if self.ram[OVERWORLD_SCREEN_INDEX] >= 128 {
            self.Palette_SetOwBgColor();
        }
        self.ram[BGMODE_COPY] = 9;
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn credits_load_scene_overworld_overlay(&mut self) {
        self.Overworld_LoadOverlays2();
        self.ram[MUSIC_CONTROL] = 0;
        self.ram[SOUND_EFFECT_AMBIENT] = 0;
        self.frame_control_view_mut().decrement_submodule();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn credits_load_scene_overworld_load_map(&mut self) {
        self.Overworld_LoadAndBuildScreen();
        self.credits_prep_and_load_sprites();
        write_le_u16(&mut self.ram, R16, 0);
        self.frame_control_view_mut().set_subsubmodule(0);
    }

    pub(super) fn credits_operate_scrolling_and_tile_map(&mut self) {
        self.credits_handle_camera_scroll_control();
        if self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS2] != 0 {
            self.OverworldHandleMapScroll();
        }
    }

    pub(super) fn credits_load_cool_background(&mut self) {
        self.ram[MAIN_TILE_THEME_INDEX] = 33;
        self.ram[AUX_TILE_THEME_INDEX] = 59;
        self.ram[SPRITE_GRAPHICS_INDEX] = 45;
        self.initialize_tilesets();
        self.ram[OVERWORLD_SCREEN_INDEX] = 0x5b;
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(self.ram[OVERWORLD_SCREEN_INDEX]),
            0x13,
        );
        self.ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = 3;
        self.palette_load_ow_bg2();
        self.overworld_copy_palettes_to_cache();
        self.Overworld_LoadOverlays2();
        self.ram[BG1VOFS_COPY2] = 0;
        self.ram[BG1HOFS_COPY2] = 0;
        self.frame_control_view_mut().decrement_submodule();
    }

    pub(super) fn credits_load_scene_dungeon(&mut self) {
        self.enable_force_blank();
        self.erase_tile_maps_normal();
        let i = (self.frame_control_view().submodule() >> 1) as usize;
        write_le_u16(&mut self.ram, WHICH_ENTRANCE, K_ENDING_TAB1[i]);
        self.Dungeon_LoadEntrance();
        self.ram[DUNG_NUM_LIT_TORCHES] = 0;
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 0;
        self.Dungeon_LoadAndDrawRoom();
        self.decompress_animated_dungeon_tiles(
            DUNG_ANIMATED_TILES[self.ram[MAIN_TILE_THEME_INDEX] as usize] as usize,
        );
        self.ram[SPRITE_GRAPHICS_INDEX] = K_ENDING_SPRITE_PACK[i];
        self.apply_dung_pal_info(K_ENDING_SPRITE_PAL[i] & 0x3f);
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = 10;
        self.initialize_tilesets();
        self.ram[PALETTE_SP6R_INDOORS] = 10;
        self.Dungeon_LoadPalettes();
        self.ram[BGMODE_COPY] = 9;
        write_le_u16(&mut self.ram, R16, 0);
        self.ram[INIDISP_COPY] = 0;
        self.frame_control_view_mut().increment_submodule();
        self.credits_prep_and_load_sprites();
    }

    pub(super) fn module18_ganon_emerges(&mut self) {
        let hofs2 = read_le_u16(&self.ram, BG2HOFS_COPY2);
        let vofs2 = read_le_u16(&self.ram, BG2VOFS_COPY2);
        let hofs1 = read_le_u16(&self.ram, BG1HOFS_COPY2);
        let vofs1 = read_le_u16(&self.ram, BG1VOFS_COPY2);
        let bg1_x_offset = read_le_u16(&self.ram, BG1_X_OFFSET);
        let bg1_y_offset = read_le_u16(&self.ram, BG1_Y_OFFSET);
        write_le_u16(
            &mut self.ram,
            BG2HOFS_COPY2,
            hofs2.wrapping_add(bg1_x_offset),
        );
        copy_le_u16(&mut self.ram, BG2HOFS_COPY, BG2HOFS_COPY2);
        write_le_u16(
            &mut self.ram,
            BG2VOFS_COPY2,
            vofs2.wrapping_add(bg1_y_offset),
        );
        copy_le_u16(&mut self.ram, BG2VOFS_COPY, BG2VOFS_COPY2);
        write_le_u16(
            &mut self.ram,
            BG1HOFS_COPY2,
            hofs1.wrapping_add(bg1_x_offset),
        );
        copy_le_u16(&mut self.ram, BG1HOFS_COPY, BG1HOFS_COPY2);
        write_le_u16(
            &mut self.ram,
            BG1VOFS_COPY2,
            vofs1.wrapping_add(bg1_y_offset),
        );
        copy_le_u16(&mut self.ram, BG1VOFS_COPY, BG1VOFS_COPY2);
        self.sprite_main();
        write_le_u16(&mut self.ram, BG1VOFS_COPY2, vofs1);
        write_le_u16(&mut self.ram, BG1HOFS_COPY2, hofs1);
        write_le_u16(&mut self.ram, BG2VOFS_COPY2, vofs2);
        write_le_u16(&mut self.ram, BG2HOFS_COPY2, hofs2);
        match self.ram[OVERWORLD_MAP_STATE] {
            0 => {
                self.dungeon_handle_layer_effect();
                self.CallForDuckIndoors();
                self.SaveDungeonKeys();
                self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
                self.ram[FLAG_IS_LINK_IMMOBILIZED] =
                    self.ram[FLAG_IS_LINK_IMMOBILIZED].wrapping_add(1);
            }
            1 => {
                self.dungeon_handle_layer_effect();
                if self.frame_control_view().submodule() == 10 {
                    self.ram[OVERWORLD_SCREEN_INDEX] = 91;
                    self.ram[PLAYER_IS_INDOORS] = 0;
                    self.frame_control_view_mut().set_main_module(24);
                    self.frame_control_view_mut().set_submodule(0);
                    self.ram[OVERWORLD_MAP_STATE] = 2;
                }
            }
            2 => {
                self.dungeon_handle_layer_effect();
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
                if self.ram[INIDISP_COPY] == 0 {
                    self.enable_force_blank();
                    self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
                    self.hud_rebuild_indoor();
                    self.ram[LINK_X_VEL] = 0;
                    self.ram[LINK_Y_VEL] = 0;
                }
            }
            3 => {
                self.ram[BIRDTRAVEL_STATUS] = 8;
                self.ram[BIRDTRAVEL_STATUS + 1] = 0;
                self.FluteMenu_LoadSelectedScreen();
                self.LoadOWMusicIfNeeded();
                self.ram[MUSIC_CONTROL] = 9;
            }
            4 => {
                self.Overworld_LoadOverlayAndMap();
                self.frame_control_view_mut().set_subsubmodule(0);
            }
            5 => {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_add(1);
                if self.ram[INIDISP_COPY] == 15 {
                    write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, 0);
                    self.ram[FLAG_UNK1] = 0;
                    self.Sprite_SpawnBatCrashCutscene();
                    self.ram[LINK_DIRECTION_FACING] = 2;
                    self.ram[SAVED_MODULE_FOR_MENU] = 9;
                    self.ram[PLAYER_IS_INDOORS] = 0;
                    self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
                    self.frame_control_view_mut().set_subsubmodule(128);
                    self.ram[CUR_PALACE_INDEX_X2] = 255;
                }
            }
            6 => {}
            7 => {
                self.frame_control_view_mut().decrement_subsubmodule();
                if self.frame_control_view().subsubmodule() == 0 {
                    self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
                }
            }
            8 => self.BirdTravel_Finish_Doit(),
            _ => {}
        }
        self.link_oam_main();
    }

    pub(super) fn module19_triforce_room(&mut self) {
        match self.frame_control_view().subsubmodule() {
            0 => {
                self.link_reset_properties_a();
                self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = 0;
                self.ram[MUSIC_CONTROL] = 0xf1;
                self.reset_transition_props_and_advance_reset_interface();
            }
            1 => {
                self.conditional_mosaic_control();
                self.apply_palette_filter_bounce();
            }
            2 => {
                self.enable_force_blank();
                self.load_credits_songs();
                write_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX, 0x189);
                self.erase_tile_maps_normal();
                self.Palette_RevertTranslucencySwap();
                self.Overworld_EnterSpecialArea();
                self.Overworld_LoadOverlays2();
                self.frame_control_view_mut().increment_subsubmodule();
                self.frame_control_view_mut().set_main_module(25);
                self.frame_control_view_mut().set_submodule(0);
            }
            3 => {
                self.ram[MAIN_TILE_THEME_INDEX] = 36;
                self.ram[SPRITE_GRAPHICS_INDEX] = 125;
                self.ram[AUX_TILE_THEME_INDEX] = 81;
                self.initialize_tilesets();
                self.Overworld_LoadAreaPalettesEx(4);
                self.Overworld_LoadPalettes(14, 0);
                self.SpecialOverworld_CopyPalettesToCache();
                self.frame_control_view_mut().increment_subsubmodule();
            }
            4 => {
                let bak0 = self.frame_control_view().subsubmodule();
                self.Module08_02_LoadAndAdvance();
                self.frame_control_view_mut()
                    .set_subsubmodule(bak0.wrapping_add(1));
                self.ram[INIDISP_COPY] = 15;
                self.ram[PALETTE_FILTER_COUNTDOWN] = 31;
                self.ram[MOSAIC_TARGET_LEVEL] = 0;
                self.ram[BG1HOFS_COPY2 + 1] = 1;
                self.ram[CGWSEL_COPY] = 2;
                self.ram[CGADSUB_COPY] = 50;
                self.ram[MOSAIC_LEVEL] = 240;
                self.ram[LINK_Y_COORD] = 236;
                self.ram[LINK_X_COORD] = 120;
                self.ram[LINK_IS_ON_LOWER_LEVEL] = 2;
                self.ram[MUSIC_CONTROL] = 32;
                self.frame_control_view_mut().set_main_module(25);
                self.frame_control_view_mut().set_submodule(0);
            }
            5 => {
                self.ram[LINK_DIRECTION] = 8;
                self.ram[LINK_DIRECTION_LAST] = 8;
                self.ram[LINK_DIRECTION_FACING] = 0;
                if self.ram[LINK_Y_COORD] < 192 {
                    self.ram[LINK_DIRECTION] = 0;
                    self.ram[LINK_DIRECTION_LAST] = 0;
                    self.ram[LINK_ANIMATION_STEPS] = 0;
                    self.frame_control_view_mut().increment_subsubmodule();
                }
            }
            6 => {
                if self.ram[PALETTE_FILTER_COUNTDOWN] & 1 == 0 && self.ram[MOSAIC_LEVEL] != 0 {
                    self.ram[MOSAIC_LEVEL] = self.ram[MOSAIC_LEVEL].wrapping_sub(0x10);
                }
                self.ram[BGMODE_COPY] = 9;
                self.ram[MOSAIC_COPY] = self.ram[MOSAIC_LEVEL] | 7;
                self.apply_palette_filter_bounce();
            }
            7 => {
                self.triforce_room_prep_gfx_slot_for_poly();
                write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x173);
                self.main_show_text_message();
                self.RenderText();
                self.ram[R16] = 0x80;
                self.frame_control_view_mut().set_main_module(25);
                self.frame_control_view_mut().increment_subsubmodule();
            }
            8 | 10 => {
                self.advance_polyhedral();
                if self.frame_control_view().subsubmodule() == 11 {
                    self.ram[MUSIC_CONTROL] = 33;
                    self.frame_control_view_mut().set_main_module(25);
                    self.ram[LINK_DIRECTION] = 0;
                    self.ram[LINK_DIRECTION_LAST] = 0;
                    self.frame_control_view_mut().increment_submodule();
                }
            }
            9 => {
                self.advance_polyhedral();
                self.RenderText();
                if self.frame_control_view().submodule() == 0 {
                    self.ram[OVERWORLD_MAP_STATE] = 0;
                    self.frame_control_view_mut().set_main_module(25);
                    self.frame_control_view_mut().increment_subsubmodule();
                }
            }
            11 => {
                self.advance_polyhedral();
                self.triforce_room_link_approach_triforce();
                if self.frame_control_view().subsubmodule() == 12 {
                    self.ram[LINK_DIRECTION] = 0;
                    self.ram[LINK_DIRECTION_LAST] = 0;
                }
            }
            12 => {
                self.advance_polyhedral();
                self.ram[R16] = self.ram[R16].wrapping_sub(1);
                if self.ram[R16] == 0 {
                    self.Palette_AnimGetMasterSword2();
                    self.frame_control_view_mut().increment_submodule();
                }
            }
            13 => {
                self.advance_polyhedral();
                self.PaletteFilter_BlindingWhiteTriforce();
                if self.ram[DARKENING_OR_LIGHTENING_SCREEN] == 255 {
                    self.frame_control_view_mut().increment_subsubmodule();
                }
            }
            14 => {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
                if self.ram[INIDISP_COPY] == 0 {
                    self.frame_control_view_mut().set_main_module(26);
                    self.frame_control_view_mut().set_submodule(0);
                    self.frame_control_view_mut().set_subsubmodule(0);
                    self.ram[IRQ_FLAG] = 0xff;
                    self.ram[IS_NMI_THREAD_ACTIVE] = 0;
                    self.ram[NMI_FLAG_UPDATE_POLYHEDRAL] = 0;
                    self.ram[SAVEGAME_IS_DARKWORLD] = 0;
                }
            }
            _ => {}
        }
        copy_le_u16(&mut self.ram, BG1HOFS_COPY, BG1HOFS_COPY2);
        copy_le_u16(&mut self.ram, BG1VOFS_COPY, BG1VOFS_COPY2);
        copy_le_u16(&mut self.ram, BG2HOFS_COPY, BG2HOFS_COPY2);
        copy_le_u16(&mut self.ram, BG2VOFS_COPY, BG2VOFS_COPY2);
        if self.frame_control_view().subsubmodule() < 7
            || self.frame_control_view().subsubmodule() >= 11
        {
            self.link_handle_velocity();
            self.link_handle_moving_animation_full_long_entry();
        }
        self.link_oam_main();
    }

    pub(super) fn Intro_InitializeBackgroundSettings(&mut self) {
        self.intro_initialize_background_settings();
    }

    pub(super) fn Polyhedral_InitializeThread(&mut self) {
        self.polyhedral_initialize_thread();
    }

    pub(super) fn Module00_Intro(&mut self) {
        self.module00_intro();
    }

    pub(super) fn Intro_Init(&mut self) {
        self.intro_init();
    }

    pub(super) fn Intro_Init_Continue(&mut self) {
        self.intro_init_continue();
    }

    pub(super) fn intro_clear1kb_blocks_of_wram(&mut self) {
        let mut i = read_le_u16(&self.ram, R16);
        let r18 = read_le_u16(&self.ram, R18);
        loop {
            for j in 0..15 {
                write_le_u16(&mut self.ram, DUNG_BG2 + i as usize + j * DUNG_BG2, 0);
            }
            i = i.wrapping_sub(2);
            if i == r18 {
                break;
            }
        }
        write_le_u16(&mut self.ram, R16, i);
        write_le_u16(&mut self.ram, R18, i.wrapping_sub(0x400));
    }

    pub(super) fn Intro_InitializeMemory_darken(&mut self) {
        self.intro_initialize_memory_darken();
    }

    pub(super) fn IntroZeldaFadein(&mut self) {
        self.intro_zelda_fadein();
    }

    pub(super) fn Intro_FadeInBg(&mut self) {
        self.intro_fade_in_bg();
    }

    pub(super) fn Intro_SwordComingDown(&mut self) {
        self.intro_sword_coming_down();
    }

    pub(super) fn Intro_WaitPlayer(&mut self) {
        self.intro_wait_player();
    }

    pub(super) fn FadeMusicAndResetSRAMMirror(&mut self) {
        self.fade_music_and_reset_sram_mirror();
    }

    pub(super) fn Intro_InitializeTriforcePolyThread(&mut self) {
        self.intro_initialize_triforce_poly_thread();
    }

    pub(super) fn Intro_InitGfx_Helper(&mut self) {
        self.intro_init_gfx_helper();
    }

    pub(super) fn LoadTriforceSpritePalette(&mut self) {
        self.load_triforce_sprite_palette();
    }

    pub(super) fn Intro_HandleAllTriforceAnimations(&mut self) {
        self.intro_handle_all_triforce_animations();
    }

    pub(super) fn Scene_AnimateEverySprite(&mut self) {
        self.scene_animate_every_sprite();
    }

    pub(super) fn Intro_AnimateTriforce(&mut self) {
        self.intro_animate_triforce();
    }

    pub(super) fn Intro_RunStep(&mut self) {
        self.intro_run_step();
    }

    pub(super) fn Intro_AnimOneObj(&mut self, k: usize) {
        self.intro_anim_one_obj(k);
    }

    pub(super) fn Intro_SpriteType_A_0(&mut self, k: usize) {
        self.intro_sprite_type_a_0(k);
    }

    pub(super) fn Intro_SpriteType_B_0(&mut self, k: usize) {
        self.intro_sprite_type_b_0(k);
    }

    pub(super) fn AnimateSceneSprite_DrawTriangle(&mut self, k: usize) {
        self.animate_scene_sprite_draw_triangle(k);
    }

    pub(super) fn intro_copy_sprite_type4_to_oam(&mut self, k: usize) {
        const LEFT: [(i8, i8, u8, u8, u8); 16] = [
            (0, 0, 0x80, 0x2b, 2),
            (16, 0, 0x82, 0x2b, 2),
            (32, 0, 0x84, 0x2b, 2),
            (48, 0, 0x86, 0x2b, 2),
            (0, 16, 0xa0, 0x2b, 2),
            (16, 16, 0xa2, 0x2b, 2),
            (32, 16, 0xa4, 0x2b, 2),
            (48, 16, 0xa6, 0x2b, 2),
            (0, 32, 0x88, 0x2b, 2),
            (16, 32, 0x8a, 0x2b, 2),
            (32, 32, 0x8c, 0x2b, 2),
            (48, 32, 0x8e, 0x2b, 2),
            (0, 48, 0xa8, 0x2b, 2),
            (16, 48, 0xaa, 0x2b, 2),
            (32, 48, 0xac, 0x2b, 2),
            (48, 48, 0xae, 0x2b, 2),
        ];
        const RIGHT: [(i8, i8, u8, u8, u8); 16] = [
            (48, 0, 0x80, 0x6b, 2),
            (32, 0, 0x82, 0x6b, 2),
            (16, 0, 0x84, 0x6b, 2),
            (0, 0, 0x86, 0x6b, 2),
            (48, 16, 0xa0, 0x6b, 2),
            (32, 16, 0xa2, 0x6b, 2),
            (16, 16, 0xa4, 0x6b, 2),
            (0, 16, 0xa6, 0x6b, 2),
            (48, 32, 0x88, 0x6b, 2),
            (32, 32, 0x8a, 0x6b, 2),
            (16, 32, 0x8c, 0x6b, 2),
            (0, 32, 0x8e, 0x6b, 2),
            (48, 48, 0xa8, 0x6b, 2),
            (32, 48, 0xaa, 0x6b, 2),
            (16, 48, 0xac, 0x6b, 2),
            (0, 48, 0xae, 0x6b, 2),
        ];
        self.animate_scene_sprite_add_objects_to_oam_buffer(k, if k == 2 { &RIGHT } else { &LEFT });
    }

    pub(super) fn exit_0_cca90(&mut self, _k: usize) {}

    pub(super) fn InitializeSceneSprite_Copyright(&mut self, k: usize) {
        self.initialize_scene_sprite_copyright(k);
    }

    pub(super) fn AnimateSceneSprite_Copyright(&mut self, k: usize) {
        self.animate_scene_sprite_copyright(k);
    }

    pub(super) fn InitializeSceneSprite_Sparkle(&mut self, k: usize) {
        self.initialize_scene_sprite_sparkle(k);
    }

    pub(super) fn AnimateSceneSprite_Sparkle(&mut self, k: usize) {
        self.animate_scene_sprite_sparkle(k);
    }

    #[rustfmt::skip]
    pub(super) fn animate_scene_sprite_add_objects_to_oam_buffer(&mut self, k: usize, entries: &[IntroSpriteEnt]) {
        self.animate_scene_sprite_add_objects_to_oam_buffer_with_offset(k, entries, 0, 0);
    }

    #[rustfmt::skip]
    fn animate_scene_sprite_add_objects_to_oam_buffer_with_offset(&mut self, k: usize, entries: &[IntroSpriteEnt], x_delta: i16, y_delta: i16) {
        let x = self.ram[INTRO_X_LO + k] as u16 | ((self.ram[INTRO_X_HI + k] as u16) << 8);
        let y = self.ram[INTRO_Y_LO + k] as u16 | ((self.ram[INTRO_Y_HI + k] as u16) << 8);
        let mut oam = read_le_u16(&self.ram, INTRO_SPRITE_ALLOC) as usize;
        for &(x_off, y_off, charnum, flags, ext) in entries {
            let obj_x = x.wrapping_add((x_off as i16).wrapping_add(x_delta) as u16);
            let obj_y = y.wrapping_add((y_off as i16).wrapping_add(y_delta) as u16);
            self.set_oam_helper0_at(oam, obj_x, obj_y, charnum, flags, ext);
            oam += 4;
        }
        write_le_u16(&mut self.ram, INTRO_SPRITE_ALLOC, oam as u16);
    }

    pub(super) fn AnimateSceneSprite_MoveTriangle(&mut self, k: usize) {
        self.animate_scene_sprite_move_triangle(k);
    }

    pub(super) fn triforce_room_prep_gfx_slot_for_poly(&mut self) {
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = 8;
        self.load_common_sprites();
        self.intro_init_gfx_helper();
        self.ram[INTRO_SPRITE_IS_INITED] = 1;
        self.ram[INTRO_SPRITE_IS_INITED + 1] = 1;
        self.ram[INTRO_SPRITE_IS_INITED + 2] = 1;
        self.ram[INTRO_SPRITE_SUBTYPE] = 4;
        self.ram[INTRO_SPRITE_SUBTYPE + 1] = 5;
        self.ram[INTRO_SPRITE_SUBTYPE + 2] = 6;
        self.ram[INIDISP_COPY] = 15;
        self.frame_control_view_mut().increment_submodule();
    }

    pub(super) fn credits_initialize_polyhedral(&mut self) {
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = 8;
        self.load_common_sprites();
        self.intro_init_gfx_helper();
        self.ram[POLY_CONFIG1] = 0;
        for k in 0..3 {
            self.ram[INTRO_SPRITE_IS_INITED + k] = 1;
            self.ram[INTRO_SPRITE_SUBTYPE + k] = 7;
        }
        self.ram[INIDISP_COPY] = 15;
        self.frame_control_view_mut().increment_submodule();
    }

    pub(super) fn advance_polyhedral(&mut self) {
        self.triforce_room_handle_poly();
        self.scene_animate_every_sprite();
    }

    pub(super) fn triforce_room_handle_poly(&mut self) {
        self.ram[IS_NMI_THREAD_ACTIVE] = 1;
        self.ram[INTRO_WANT_DOUBLE_RET] = 1;
        if self.ram[INTRO_DID_RUN_STEP] != 0 {
            return;
        }
        match self.ram[INTRO_STEP_INDEX] {
            0 => {
                self.ram[POLY_CONFIG1] = self.ram[POLY_CONFIG1].wrapping_sub(2);
                if self.ram[POLY_CONFIG1] < 2 {
                    self.ram[POLY_CONFIG1] = 0;
                    self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
                    self.frame_control_view_mut().increment_subsubmodule();
                }
                if self.frame_control_view().subsubmodule() >= 10 {
                    self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
                    self.ram[INTRO_Y_VEL + 1] = 5;
                }
                self.ram[POLY_B] = self.ram[POLY_B].wrapping_add(2);
                self.ram[POLY_A] = self.ram[POLY_A].wrapping_add(1);
            }
            1 => {
                if self.frame_control_view().subsubmodule() >= 10 {
                    self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
                    self.ram[INTRO_Y_VEL + 1] = 5;
                }
                self.ram[POLY_B] = self.ram[POLY_B].wrapping_add(2);
                self.ram[POLY_A] = self.ram[POLY_A].wrapping_add(1);
            }
            2 => {
                write_le_u16(&mut self.ram, TRIFORCE_CTR, 0x1c0);
                if self.ram[POLY_CONFIG1] < 128 {
                    self.ram[POLY_CONFIG1] = self.ram[POLY_CONFIG1].wrapping_add(1);
                } else if (self.ram[POLY_B].wrapping_sub(10) & 0x7f) >= 92
                    && self.ram[POLY_A].wrapping_sub(11) >= 220
                {
                    self.ram[POLY_A] = 0;
                    self.ram[POLY_B] = 0;
                    self.frame_control_view_mut().increment_subsubmodule();
                    self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
                    self.ram[SOUND_EFFECT_1] = 44;
                    write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0xd7 * 2, 0x7fff);
                    self.ram[FLAG_UPDATE_CGRAM_IN_NMI] =
                        self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
                    self.ram[INTRO_STEP_TIMER] = 6;
                    break_triforce_handle_poly(self);
                    return;
                }
                self.ram[POLY_B] = self.ram[POLY_B].wrapping_add(5);
                self.ram[POLY_A] = self.ram[POLY_A].wrapping_add(3);
            }
            3 => {
                self.ram[INTRO_STEP_TIMER] = self.ram[INTRO_STEP_TIMER].wrapping_sub(1);
                if self.ram[INTRO_STEP_TIMER] == 0 {
                    write_le_u16(
                        &mut self.ram,
                        MAIN_PALETTE_BUFFER + 0xd7 * 2,
                        K_POLYHEDRAL_PALETTE[7],
                    );
                    self.ram[FLAG_UPDATE_CGRAM_IN_NMI] =
                        self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
                    self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
                }
            }
            _ => {}
        }
        self.ram[INTRO_DID_RUN_STEP] = 1;
        self.ram[INTRO_WANT_DOUBLE_RET] = 0;
        self.ram[INTRO_FRAME_CTR] = self.ram[INTRO_FRAME_CTR].wrapping_add(1);
    }

    pub(super) fn credits_animate_the_triangles(&mut self) {
        self.ram[INTRO_FRAME_CTR] = self.ram[INTRO_FRAME_CTR].wrapping_add(1);
        self.ram[IS_NMI_THREAD_ACTIVE] = 1;
        if self.ram[INTRO_DID_RUN_STEP] == 0 {
            self.ram[POLY_B] = self.ram[POLY_B].wrapping_add(3);
            self.ram[POLY_A] = self.ram[POLY_A].wrapping_add(1);
            self.ram[INTRO_DID_RUN_STEP] = 1;
        }
        self.scene_animate_every_sprite();
    }

    pub(super) fn initialize_scene_sprite_triforce_room_triangle(&mut self, k: usize) {
        const X: [i16; 3] = [0x4e, 0x5f, 0x72];
        const Y: [i16; 3] = [0x9c, 0x9c, 0x9c];
        const XVEL: [i8; 3] = [-2, 0, 2];
        const YVEL: [i8; 3] = [4, -4, 4];
        self.write_intro_x(k, X[k]);
        self.write_intro_y(k, Y[k]);
        self.ram[INTRO_X_VEL + k] = XVEL[k] as u8;
        self.ram[INTRO_Y_VEL + k] = YVEL[k] as u8;
        self.ram[INTRO_SPRITE_IS_INITED + k] = self.ram[INTRO_SPRITE_IS_INITED + k].wrapping_add(1);
    }

    pub(super) fn intro_sprite_type_b_456(&mut self, k: usize) {
        self.intro_copy_sprite_type4_to_oam(k);
        if self.ram[INTRO_WANT_DOUBLE_RET] != 0 {
            return;
        }
        self.animate_scene_sprite_move_triangle(k);
        match self.ram[INTRO_STEP_INDEX] {
            0 => {
                const XACC: [i8; 3] = [-1, 0, 1];
                const YACC: [i8; 3] = [-1, -1, -1];
                if self.ram[INTRO_FRAME_CTR] & 7 == 0 {
                    self.ram[INTRO_X_VEL + k] =
                        self.ram[INTRO_X_VEL + k].wrapping_add(XACC[k] as u8);
                }
                if self.ram[INTRO_FRAME_CTR] & 3 == 0 {
                    self.ram[INTRO_Y_VEL + k] =
                        self.ram[INTRO_Y_VEL + k].wrapping_add(YACC[k] as u8);
                }
            }
            1 => {
                self.ram[INTRO_X_VEL + k] = 0;
                self.ram[INTRO_Y_VEL + k] = 0;
            }
            2 => {
                const XFINAL: [u8; 3] = [0x59, 0x5f, 0x67];
                const YFINAL: [u8; 3] = [0x74, 0x68, 0x74];
                if self.ram[INTRO_FRAME_CTR] & 3 == 0 {
                    self.animate_triforce_room_triangle_handle_contracting(k);
                }
                if XFINAL[k] == self.ram[INTRO_X_LO + k] {
                    self.ram[INTRO_X_VEL + k] = 0;
                }
                if YFINAL[k] == self.ram[INTRO_Y_LO + k] {
                    self.ram[INTRO_Y_VEL + k] = 0;
                }
            }
            3 | 4 => {
                const YFINAL2: [u8; 3] = [0x72, 0x66, 0x72];
                let ctr = read_le_u16(&self.ram, TRIFORCE_CTR);
                if ctr == 0 {
                    self.ram[INTRO_Y_LO + k] = YFINAL2[k];
                } else {
                    write_le_u16(&mut self.ram, TRIFORCE_CTR, ctr.wrapping_sub(1));
                }
            }
            _ => {}
        }
    }

    pub(super) fn animate_triforce_room_triangle_handle_contracting(&mut self, k: usize) {
        const XFINAL: [u8; 3] = [0x59, 0x5f, 0x67];
        const YFINAL: [u8; 3] = [0x74, 0x68, 0x74];
        let xv = self.ram[INTRO_X_VEL + k].wrapping_add(if self.ram[INTRO_X_LO + k] <= XFINAL[k] {
            1
        } else {
            0xff
        });
        self.ram[INTRO_X_VEL + k] = match xv {
            0x11 => 0x10,
            0xef => 0xf0,
            _ => xv,
        };
        let yv = self.ram[INTRO_Y_VEL + k].wrapping_add(if self.ram[INTRO_Y_LO + k] <= YFINAL[k] {
            1
        } else {
            0xff
        });
        self.ram[INTRO_Y_VEL + k] = match yv {
            0x11 => 0x10,
            0xef => 0xf0,
            _ => yv,
        };
    }

    pub(super) fn initialize_scene_sprite_credits_triangle(&mut self, k: usize) {
        const X: [u8; 3] = [0x29, 0x5f, 0x97];
        const Y: [u8; 3] = [0x70, 0x20, 0x70];
        self.ram[INTRO_X_LO + k] = X[k];
        self.ram[INTRO_X_HI + k] = 0;
        self.ram[INTRO_Y_LO + k] = Y[k];
        self.ram[INTRO_Y_HI + k] = 0;
        self.ram[INTRO_SPRITE_IS_INITED + k] = self.ram[INTRO_SPRITE_IS_INITED + k].wrapping_add(1);
    }

    pub(super) fn animate_scene_sprite_credits_triangle(&mut self, k: usize) {
        const XACC: [i8; 3] = [-1, 0, 1];
        const YACC: [i8; 3] = [1, -1, 1];
        self.load_triforce_sprite_palette();
        self.intro_copy_sprite_type4_to_oam(k);
        self.animate_scene_sprite_move_triangle(k);
        if self.frame_control_view().submodule() != 36 {
            self.ram[INTRO_SPRITE_STATE + k] = 0;
            return;
        }
        if self.ram[INTRO_SPRITE_STATE + k] != 80 {
            self.ram[INTRO_SPRITE_STATE + k] = self.ram[INTRO_SPRITE_STATE + k].wrapping_add(1);
            self.ram[INTRO_X_VEL + k] = self.ram[INTRO_X_VEL + k].wrapping_add(XACC[k] as u8);
            self.ram[INTRO_Y_VEL + k] = self.ram[INTRO_Y_VEL + k].wrapping_add(YACC[k] as u8);
        }
    }

    pub(super) fn Intro_DisplayLogo(&mut self) {
        self.intro_display_logo();
    }

    pub(super) fn Intro_SetupSwordAndIntroFlash(&mut self) {
        self.intro_setup_sword_and_intro_flash();
    }

    pub(super) fn Intro_PeriodicSwordAndIntroFlash(&mut self) {
        self.intro_periodic_sword_and_intro_flash();
    }

    pub(super) fn module1_a_credits(&mut self) {
        write_le_u16(&mut self.ram, OAM_REGION_BASE, 0x30);
        write_le_u16(&mut self.ram, OAM_REGION_BASE + 2, 0x1d0);
        write_le_u16(&mut self.ram, OAM_REGION_BASE + 4, 0);
        match self.frame_control_view().submodule() {
            0 | 4 | 6 | 8 | 10 | 12 | 14 | 16 | 18 | 24 | 26 | 28 | 30 => {
                self.credits_load_next_scene_overworld()
            }
            2 | 20 | 22 => self.credits_load_next_scene_dungeon(),
            1 | 5 | 7 | 9 | 11 | 13 | 15 | 17 | 19 | 25 | 27 | 29 | 31 => {
                self.credits_scroll_scene_overworld()
            }
            3 | 21 | 23 => self.credits_scroll_scene_dungeon(),
            32 => self.end_sequence_32(),
            33 => self.credits_brighten_triangles(),
            34 => self.credits_fade_color_and_begin_animating(),
            35 => self.credits_stop_credits_scroll(),
            36 => self.credits_fade_and_disperse_triangles(),
            37 => self.credits_fade_in_the_end(),
            38 => self.credits_hang_forever(),
            _ => {}
        }
    }

    pub(super) fn credits_load_next_scene_overworld(&mut self) {
        match self.frame_control_view().subsubmodule() {
            0 => self.credits_load_scene_overworld_prep_gfx(),
            1 => self.credits_load_scene_overworld_overlay(),
            2 => self.credits_load_scene_overworld_load_map(),
            _ => {}
        }
        self.credits_add_ending_sequence_text();
    }

    pub(super) fn credits_load_next_scene_dungeon(&mut self) {
        self.credits_load_scene_dungeon();
        self.credits_add_ending_sequence_text();
    }

    pub(super) fn credits_prep_and_load_sprites(&mut self) {
        for k in (0..16).rev() {
            self.sprite_prep_reset_properties(k);
            self.ram[SPRITE_STATE + k] = 0;
            self.ram[SPRITE_FLAGS5 + k] = 0;
            self.ram[SPRITE_DEFL_BITS + k] = 0;
        }
        let scene = (self.frame_control_view().submodule() >> 1) as usize;
        match scene {
            2 => {
                self.ram[SPRITE_Y_VEL + 6] = (-16i8) as u8;
                self.init_ending_sprites_overworld(scene);
            }
            3 => {
                self.ram[SPRITE_A + 5] = 22;
                self.ram[SPRITE_Y_VEL] = (-16i8) as u8;
                self.ram[SPRITE_Y_VEL + 1] = 16;
                self.ram[SPRITE_HEAD_DIR + 1] = 1;
                for j in (0..=2).rev() {
                    self.ram[SPRITE_TYPE + 2 + j] = 0x57;
                    self.ram[SPRITE_OAM_FLAGS + 2 + j] = 0x31;
                }
                self.init_ending_sprites_overworld(scene);
            }
            6 => {
                self.ram[SPRITE_DELAY_MAIN] = 255;
                self.ram[SPRITE_DELAY_MAIN + 1] = 255;
                self.ram[SPRITE_DELAY_MAIN + 2] = 255;
                self.init_ending_sprites_overworld(scene);
            }
            7 => {
                self.ram[SPRITE_DELAY_MAIN + 1] = 255;
                self.init_ending_sprites_overworld(scene);
            }
            9 => {
                for j in (0..=4).rev() {
                    self.ram[SPRITE_DELAY_MAIN + j] = (j * 19) as u8;
                    self.ram[SPRITE_STATE + j] = 0;
                }
                self.ram[SPRITE_TYPE + 5] = 0x2e;
                for j in (0..=1).rev() {
                    self.ram[SPRITE_TYPE + 7 + j] = 0x9f;
                    self.ram[SPRITE_TYPE + 9 + j] = 0xa0;
                    self.ram[SPRITE_FLAGS2 + 7 + j] = 1;
                    self.ram[SPRITE_FLAGS2 + 9 + j] = 2;
                    self.ram[SPRITE_FLAGS3 + 7 + j] = 0x10;
                    self.ram[SPRITE_FLAGS3 + 9 + j] = 0x10;
                }
                self.init_ending_sprites_overworld(scene);
            }
            10 => {
                self.ram[SPRITE_DELAY_MAIN + 1] = 0x10;
                self.ram[SPRITE_DELAY_MAIN + 2] = 0x20;
                self.ram[SPRITE_OAM_FLAGS + 3] = 8;
                self.ram[SPRITE_OAM_FLAGS + 4] = 8;
                self.init_ending_sprites_dungeon(scene);
            }
            11 => {
                self.ram[SPRITE_OAM_FLAGS + 4] = 0x79;
                self.ram[SPRITE_OAM_FLAGS + 5] = 0x39;
                self.ram[SPRITE_D + 1] = 1;
                self.ram[SPRITE_A + 1] = 4;
                self.init_ending_sprites_dungeon(scene);
            }
            12 => {
                for j in (0..=1).rev() {
                    self.ram[SPRITE_OAM_FLAGS + j + 3] = 0x39;
                    self.ram[SPRITE_TYPE + j + 3] = 0x0b;
                    self.ram[SPRITE_FLAGS3 + j + 3] = 0x10;
                    self.ram[SPRITE_FLAGS2 + j + 3] = 1;
                }
                self.ram[SPRITE_TYPE + 5] = 0x2a;
                self.ram[SPRITE_TYPE + 6] = 0x79;
                self.ram[SPRITE_AI_STATE + 6] = 1;
                self.ram[SPRITE_Z + 6] = 5;
                self.init_ending_sprites_overworld(scene);
            }
            14 => {
                self.ram[SPRITE_Y_VEL + 5] = (-16i8) as u8;
                self.ram[SPRITE_Y_VEL + 6] = 16;
                self.ram[SPRITE_HEAD_DIR + 6] = 1;
                self.ram[SPRITE_A] = 8;
                for j in (0..=3).rev() {
                    self.ram[SPRITE_Y_VEL + 1 + j] = 4;
                }
                self.init_ending_sprites_overworld(scene);
            }
            15 => {
                self.ram[SPRITE_C + 4] = 2;
                self.ram[SPRITE_Y_VEL + 5] = 8;
                self.ram[SPRITE_DELAY_MAIN + 1] = 0x13;
                self.ram[SPRITE_DELAY_MAIN + 4] = 0x40;
                self.init_ending_sprites_overworld(scene);
            }
            0 | 4 | 5 | 8 | 13 => self.init_ending_sprites_overworld(scene),
            1 => self.init_ending_sprites_dungeon(scene),
            _ => {}
        }
    }

    fn init_ending_sprites_overworld(&mut self, scene: usize) {
        let idx = K_ENDING_SPRITES_IDX[scene];
        let num = K_ENDING_SPRITES_IDX[scene + 1] - idx;
        let area = read_le_u16(&self.ram, OVERWORLD_AREA_INDEX);
        let base_x = area.wrapping_shl(9) & 0x0f00;
        let base_y = area.wrapping_shl(6) & 0x0e00;
        for k in (0..num).rev() {
            write_le_u16(&mut self.ram, SPRCOLL_X_SIZE, 0xffff);
            write_le_u16(&mut self.ram, SPRCOLL_Y_SIZE, 0xffff);
            let x = base_x.wrapping_add(K_ENDING_SPRITES_X[idx + k]);
            let y = base_y.wrapping_add(K_ENDING_SPRITES_Y[idx + k]);
            self.ram[SPRITE_X_LO + k] = x as u8;
            self.ram[SPRITE_X_HI + k] = (x >> 8) as u8;
            self.ram[SPRITE_Y_LO + k] = y as u8;
            self.ram[SPRITE_Y_HI + k] = (y >> 8) as u8;
        }
    }

    fn init_ending_sprites_dungeon(&mut self, scene: usize) {
        let idx = K_ENDING_SPRITES_IDX[scene];
        let num = K_ENDING_SPRITES_IDX[scene + 1] - idx;
        let room = read_le_u16(&self.ram, DUNGEON_ROOM_INDEX2);
        self.ram[SPRITE_ROOM_ORIGIN_Y_HI] = ((room >> 3) as u8) & 0xfe;
        self.ram[SPRITE_ROOM_ORIGIN_X_HI] = ((room & 15) << 1) as u8;
        for k in (0..num).rev() {
            write_le_u16(&mut self.ram, SPRCOLL_X_SIZE, 0xffff);
            write_le_u16(&mut self.ram, SPRCOLL_Y_SIZE, 0xffff);
            let x = ((self.ram[SPRITE_ROOM_ORIGIN_X_HI] as u16) << 8)
                .wrapping_add(K_ENDING_SPRITES_X[idx + k]);
            let y = ((self.ram[SPRITE_ROOM_ORIGIN_Y_HI] as u16) << 8)
                .wrapping_add(K_ENDING_SPRITES_Y[idx + k]);
            self.ram[SPRITE_X_LO + k] = x as u8;
            self.ram[SPRITE_X_HI + k] = (x >> 8) as u8;
            self.ram[SPRITE_Y_LO + k] = y as u8;
            self.ram[SPRITE_Y_HI + k] = (y >> 8) as u8;
        }
    }

    pub(super) fn credits_scroll_scene_overworld(&mut self) {
        for k in (0..16).rev() {
            if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                self.ram[SPRITE_DELAY_MAIN + k] = self.ram[SPRITE_DELAY_MAIN + k].wrapping_sub(1);
            }
        }
        let i = (self.frame_control_view().submodule() >> 1) as usize;
        self.ram[LINK_X_VEL] = 0;
        self.ram[LINK_Y_VEL] = 0;
        let r16 = read_le_u16(&self.ram, R16);
        if r16 >= 0x40 && r16 & 1 == 0 {
            if read_le_u16(&self.ram, BG2VOFS_COPY2) != K_ENDING1_TARGET_SCROLL_Y[i] {
                self.ram[LINK_Y_VEL] = K_ENDING1_YVEL[i] as u8;
            }
            if read_le_u16(&self.ram, BG2HOFS_COPY2) != K_ENDING1_TARGET_SCROLL_X[i] {
                self.ram[LINK_X_VEL] = K_ENDING1_XVEL[i] as u8;
            }
        }
        self.credits_operate_scrolling_and_tile_map();
        self.credits_handle_scene_fade();
    }

    pub(super) fn credits_scroll_scene_dungeon(&mut self) {
        for k in (0..16).rev() {
            if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                self.ram[SPRITE_DELAY_MAIN + k] = self.ram[SPRITE_DELAY_MAIN + k].wrapping_sub(1);
            }
        }
        let i = (self.frame_control_view().submodule() >> 1) as usize;
        let r16 = read_le_u16(&self.ram, R16);
        if r16 >= 0x40 && r16 & 1 == 0 {
            if read_le_u16(&self.ram, BG2VOFS_COPY2) != K_ENDING1_TARGET_SCROLL_Y[i] {
                add_i8_to_word(&mut self.ram, BG2VOFS_COPY2, K_ENDING1_YVEL[i]);
            }
            if read_le_u16(&self.ram, BG2HOFS_COPY2) != K_ENDING1_TARGET_SCROLL_X[i] {
                add_i8_to_word(&mut self.ram, BG2HOFS_COPY2, K_ENDING1_XVEL[i]);
            }
        }
        self.credits_handle_scene_fade();
    }

    pub(super) fn credits_handle_scene_fade(&mut self) {
        const TAB0: [u16; 16] = [
            0x300, 0x280, 0x250, 0x2e0, 0x280, 0x250, 0x2c0, 0x2c0, 0x250, 0x250, 0x280, 0x250,
            0x480, 0x400, 0x250, 0x500,
        ];
        const CASE0_TAB1: [u8; 12] = [
            0x1e, 0x20, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x16, 0x16, 0x16, 0x16,
        ];
        const CASE0_TAB0: [u8; 12] = [6, 3, 2, 2, 2, 2, 2, 2, 6, 6, 6, 6];
        const CASE0_OAM_FLAGS: [u8; 12] = [
            0x3b, 0x31, 0x3d, 0x3f, 0x39, 0x3b, 0x37, 0x3d, 0x39, 0x37, 0x37, 0x39,
        ];

        let i = (self.frame_control_view().submodule() >> 1) as usize;
        let r16 = read_le_u16(&self.ram, R16);
        match i {
            0 => {
                for k in (8..=11).rev() {
                    self.ram[SPRITE_OAM_FLAGS + k] = CASE0_OAM_FLAGS[k];
                    self.credits_sprite_draw_single(k, CASE0_TAB0[k], CASE0_TAB1[k]);
                }
                for k in (2..=7).rev() {
                    self.ram[SPRITE_OAM_FLAGS + k] =
                        CASE0_OAM_FLAGS[k] | ((self.ram[FRAME_COUNTER] << 2) & 0x40);
                    self.credits_sprite_draw_single(k, CASE0_TAB0[k], CASE0_TAB1[k]);
                }
                for k in (0..=1).rev() {
                    self.ram[SPRITE_OAM_FLAGS + k] = CASE0_OAM_FLAGS[k];
                    self.credits_sprite_draw_single(k, CASE0_TAB0[k], CASE0_TAB1[k]);
                }
            }
            1 => {
                self.credits_sprite_draw_single(0, 3, 12);
                self.credits_sprite_draw_draw_shadow(0);
                let k = 1;
                self.ram[SPRITE_TYPE + k] = 0x73;
                self.ram[SPRITE_OAM_FLAGS + k] = 0x27;
                self.ram[SPRITE_E + k] = 2;
                self.credits_sprite_draw_preexisting_sprite_draw(k, 16);
            }
            2 => {
                const CASE2_TAB0: [u8; 2] = [0x20, 0x40];
                const CASE2_TAB1: [i8; 2] = [16, -16];
                const CASE2_TAB2: [u8; 5] = [0x28, 0x2a, 0x2c, 0x2e, 0x2c];
                const CASE2_TAB3: [u8; 5] = [3, 3, 3, 3, 3];
                const CASE2_DELAY: [u8; 2] = [0x30, 0x10];
                self.ram[FLAG_TRAVEL_BIRD] =
                    CASE2_TAB0[((self.ram[FRAME_COUNTER] >> 2) & 1) as usize];
                let mut k = 6usize;
                let j = ((self.ram[SPRITE_X_VEL + k] >> 7) & 1) as usize;
                self.ram[SPRITE_OAM_FLAGS + k] =
                    self.ram[SPRITE_X_VEL + k].wrapping_add(CASE2_TAB1[j] as u8) >> 1 & 0x40 | 0x32;
                self.credits_sprite_draw_single(k, 2, 0x24);
                self.credits_sprite_draw_circling_birds(k);
                k -= 1;
                self.ram[SPRITE_OAM_FLAGS + k] = 0x31;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    let j = self.ram[SPRITE_A + k] as usize;
                    self.ram[SPRITE_A + k] ^= 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = CASE2_DELAY[j];
                    self.ram[SPRITE_GRAPHICS + k] =
                        self.ram[SPRITE_GRAPHICS + k].wrapping_add(1) & 3;
                }
                self.credits_sprite_draw_single(k, 2, 0x26);
                k -= 1;
                loop {
                    if self.ram[FRAME_COUNTER] & 15 == 0 {
                        self.ram[SPRITE_GRAPHICS + k] ^= 1;
                    }
                    self.ram[SPRITE_OAM_FLAGS + k] = 0x31;
                    self.credits_sprite_draw_single(k, CASE2_TAB3[k], CASE2_TAB2[k]);
                    self.end_sequence_draw_shadow2(k);
                    if k == 0 {
                        break;
                    }
                    k -= 1;
                }
            }
            3 => {
                const CASE3_GFX: [u8; 4] = [1, 2, 3, 2];
                let mut k = 0usize;
                while k < 5 {
                    if k < 2 {
                        self.ram[SPRITE_TYPE + k] = 1;
                        self.ram[SPRITE_OAM_FLAGS + k] = 0x0b;
                        self.credits_sprite_draw_set_shadow_prop(k, 2);
                        self.ram[SPRITE_Z + k] = 48;
                        let j = ((self.ram[FRAME_COUNTER].wrapping_add(if k != 0 {
                            0x5f
                        } else {
                            0x7d
                        })) >> 2
                            & 3) as usize;
                        self.ram[SPRITE_GRAPHICS + k] = CASE3_GFX[j];
                        self.credits_sprite_draw_circling_birds(k);
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 12);
                    } else {
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 16);
                    }
                    k += 1;
                }
                self.credits_sprite_draw_single(k, 2, 0x38);
                self.ending_func2(k, 0x30);
                k += 1;
                self.credits_sprite_draw_single(k, 3, 0x3a);
            }
            4 => {
                const CASE4_TAB1: [u8; 2] = [0x30, 0x32];
                const CASE4_TAB0: [u8; 2] = [2, 2];
                const CASE4_CTR: [u16; 2] = [0x20, 0];
                const CASE4_XYVEL: [i8; 10] = [0, -12, -16, -12, 0, 12, 16, 12, 0, -12];
                const CASE4_DELAYVEL: [u8; 24] = [
                    0x3b, 0x14, 0x1e, 0x1d, 0x2c, 0x2b, 0x42, 0x20, 0x27, 0x28, 0x2e, 0x38, 0x3a,
                    0x4c, 0x32, 0x44, 0x2e, 0x2f, 0x1e, 0x28, 0x47, 0x35, 0x32, 0x30,
                ];
                let mut k = 2usize;
                self.ram[SPRITE_OAM_FLAGS + k] = 0x35;
                self.credits_sprite_draw_single(k, 1, 0x3c);
                k -= 1;
                loop {
                    self.ram[SPRITE_OAM_FLAGS + k] =
                        self.ram[SPRITE_X_VEL + k].wrapping_sub(1) >> 1 & 0x40 ^ 0x71;
                    self.ram[SPRITE_GRAPHICS + k] = self.ram[FRAME_COUNTER] >> 3 & 1;
                    if r16 >= CASE4_CTR[k] && self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                        let a = CASE4_DELAYVEL[self.ram[SPRITE_A + k] as usize];
                        self.ram[SPRITE_DELAY_MAIN + k] = a & 0xf8;
                        self.ram[SPRITE_Y_VEL + k] = CASE4_XYVEL[((a & 7) + 2) as usize] as u8;
                        self.ram[SPRITE_X_VEL + k] = CASE4_XYVEL[(a & 7) as usize] as u8;
                        self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
                    }
                    self.credits_sprite_draw_single(k, CASE4_TAB0[k], CASE4_TAB1[k]);
                    self.end_sequence_draw_shadow2(k);
                    self.sprite_move_xy(k);
                    if k == 0 {
                        break;
                    }
                    k -= 1;
                }
            }
            5 => {
                const CASE5_TAB0: [u8; 2] = [0, 4];
                const CASE5_TAB1: [u16; 2] = [0x0a, 0x224];
                const CASE5_TAB2: [u8; 2] = [10, 14];
                if r16 == 0x200 {
                    self.ram[SOUND_EFFECT_1] = 1;
                } else if r16 == 0x208 {
                    self.ram[SOUND_EFFECT_1] = 0x2c;
                }
                if r16.wrapping_sub(0x208) < 0x30 {
                    self.credits_sprite_draw_add_sparkle(2, 10, r16.wrapping_sub(0x208) as u8);
                }
                let mut k = 3usize;
                if r16 >= 0x200 {
                    self.ram[SPRITE_GRAPHICS + k] = 1;
                }
                self.ram[SPRITE_OAM_FLAGS + k] = 0x31;
                self.credits_sprite_draw_single(k, 4, 8);
                self.end_sequence_draw_shadow2(k);
                let j = self.ram[SPRITE_GRAPHICS + k] as usize;
                k -= 1;
                self.ram[SPRITE_GRAPHICS + k] = j as u8;
                self.ram[LINK_DMA_SWORD_GRAPHICS_INDEX] = 0;
                self.ram[LINK_DMA_SHIELD_GRAPHICS_INDEX] = CASE5_TAB0[j];
                self.ram[SPRITE_OAM_FLAGS + k] = 0x30;
                write_le_u16(&mut self.ram, LINK_DMA_GRAPHICS_INDEX, CASE5_TAB1[j]);
                self.credits_sprite_draw_single(k, 5, CASE5_TAB2[j]);
                self.end_sequence_draw_shadow2(k);
            }
            6 => {
                const SPR_TYPE: [u8; 3] = [0x52, 0x55, 0x55];
                const OAM_SIZE: [u8; 3] = [0x20, 8, 8];
                const STATE: [u8; 3] = [3, 1, 1];
                const GFX: [u8; 6] = [0, 5, 5, 1, 6, 6];
                let idx = K_ENDING_SPRITES_IDX[i];
                let num = K_ENDING_SPRITES_IDX[i + 1] - idx;
                for k in (0..num).rev() {
                    self.ram[CUR_OBJECT_INDEX] = k as u8;
                    self.ram[SPRITE_TYPE + k] = SPR_TYPE[k];
                    self.oam_allocate_from_region_a(OAM_SIZE[k]);
                    self.ram[SPRITE_AI_STATE + k] = STATE[k];
                    let j = if r16 >= 0x26f { k + 3 } else { k };
                    if r16 == 0x26f {
                        self.ram[SOUND_EFFECT_2] = 0x21;
                    }
                    self.ram[SPRITE_GRAPHICS + k] = GFX[j];
                    self.ram[SPRITE_OAM_FLAGS + k] = 0x33;
                    self.sprite_get_16_bit_coords_ending(k);
                    self.sprite_active_main_ending(k);
                }
            }
            7 => {
                let mut k = 1usize;
                self.credits_sprite_draw_set_shadow_prop(k, 2);
                self.ram[SPRITE_TYPE + k] = 0xe9;
                self.oam_allocate_from_region_a(0x0c);
                self.ram[SPRITE_OAM_FLAGS + k] = 0x37;
                self.sprite_get_16_bit_coords_ending(k);
                if self.ram[FRAME_COUNTER] & 15 == 0 {
                    self.ram[SPRITE_GRAPHICS + k] ^= 1;
                }
                self.sprite_active_main_ending(k);
                if r16 >= 0x180 {
                    self.ram[SPRITE_Y_VEL + k] = 4;
                    if self.ram[SPRITE_Y_LO + k] != 0x7c {
                        self.sprite_move_xy(k);
                    }
                }
                k -= 1;
                self.ram[SPRITE_TYPE + k] = 0x36;
                self.oam_allocate_from_region_a(0x18);
                self.ram[SPRITE_OAM_FLAGS + k] = 0x39;
                self.sprite_get_16_bit_coords_ending(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    const GFX_STEP: [i8; 2] = [1, -1];
                    self.ram[SPRITE_DELAY_MAIN + k] = 4;
                    self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_GRAPHICS + k]
                        .wrapping_add(GFX_STEP[((r16 >> 9) & 1) as usize] as u8)
                        & 7;
                }
                self.sprite_active_main_ending(k);
            }
            8 => {
                let k = 0usize;
                self.ram[SPRITE_TYPE + k] = 0x2c;
                self.oam_allocate_from_region_a(0x2c);
                self.ram[SPRITE_OAM_FLAGS + k] = 0x3b;
                self.sprite_get_16_bit_coords_ending(k);
                self.ram[SPRITE_GRAPHICS + k] = if r16 < 0x1c0 {
                    ((r16 >> 5) & 1) as u8
                } else {
                    2
                };
                self.sprite_active_main_ending(k);
            }
            9 => {
                let mut k = 0usize;
                while k < 5 {
                    if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                        self.ram[SPRITE_DELAY_MAIN + k] = 96;
                        self.ram[SPRITE_STATE + k] = 96;
                        self.ram[SPRITE_X_VEL + k] = 0;
                        self.ram[SPRITE_X_LO + k] = 238;
                        self.ram[SPRITE_X_HI + k] = 4;
                        self.ram[SPRITE_Y_LO + k] = 24;
                        self.ram[SPRITE_Y_HI + k] = 11;
                    }
                    if self.ram[SPRITE_STATE + k] != 0 {
                        self.ram[SPRITE_Y_VEL + k] = (-8i8) as u8;
                        self.sprite_move_xy(k);
                        if self.ram[FRAME_COUNTER] & 1 == 0 {
                            let delta = if ((self.ram[FRAME_COUNTER] >> 5) ^ k as u8) & 1 != 0 {
                                -1i8
                            } else {
                                1i8
                            };
                            self.ram[SPRITE_X_VEL + k] =
                                self.ram[SPRITE_X_VEL + k].wrapping_add(delta as u8);
                        }
                        self.credits_sprite_draw_single(k, 1, 0x10);
                    }
                    k += 1;
                }
                loop {
                    if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                        const DELAY1: [u8; 4] = [16, 14, 16, 18];
                        const DELAY2: [u8; 4] = [20, 48, 20, 20];
                        let a = self.ram[SPRITE_A + k] as usize;
                        self.ram[SPRITE_DELAY_MAIN + k] =
                            if k == 5 { DELAY1[a] } else { DELAY2[a] };
                        self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1) & 3;
                        self.ram[SPRITE_GRAPHICS + k] ^= 1;
                    }
                    if k == 5 {
                        self.ram[SPRITE_OAM_FLAGS + k] = 0x31;
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 0x10);
                        k += 1;
                    } else {
                        self.credits_sprite_draw_single(k, 2, 0x12);
                        k += 1;
                        break;
                    }
                }
                while k != 11 {
                    const D: [u8; 4] = [0, 1, 0, 1];
                    const OAM_FLAGS: [u8; 4] = [55, 55, 59, 61];
                    const TAB: [u8; 4] = [8, 8, 12, 12];
                    self.ram[SPRITE_OAM_FLAGS + k] = OAM_FLAGS[k - 7];
                    self.ram[SPRITE_D + k] = D[k - 7];
                    self.credits_sprite_draw_activate_and_run_sprite(k, TAB[k - 7]);
                    k += 1;
                }
            }
            10 => {
                const WISH_POND_X: [u8; 8] = [0, 4, 8, 12, 16, 20, 24, 0];
                const WISH_POND_Y: [u8; 8] = [0, 8, 16, 24, 32, 40, 4, 36];
                let k = 5usize;
                self.sprite_get_16_bit_coords_ending(k);
                if self.ram[SPRITE_PAUSE + k] == 0 {
                    let xb = WISH_POND_X[(self.get_random_number() & 7) as usize]
                        .wrapping_add(read_le_u16(&self.ram, CUR_SPRITE_X) as u8);
                    let yb = WISH_POND_Y[(self.get_random_number() & 7) as usize]
                        .wrapping_add(read_le_u16(&self.ram, CUR_SPRITE_Y) as u8);
                    self.credits_sprite_draw_add_sparkle(3, xb, yb);
                }
                for k in 3..5 {
                    if self.ram[SPRITE_DELAY_AUX1 + k] != 0 {
                        self.ram[SPRITE_DELAY_AUX1 + k] =
                            self.ram[SPRITE_DELAY_AUX1 + k].wrapping_sub(1);
                    }
                    self.ram[SPRITE_TYPE + k] = 0xe3;
                    self.credits_sprite_draw_set_shadow_prop(k, 1);
                    self.credits_sprite_draw_activate_and_run_sprite(k, 8);
                }
                self.ram[SPRITE_TYPE + k] = 0x72;
                self.ram[SPRITE_OAM_FLAGS + k] = 0x3b;
                self.ram[SPRITE_STATE + k] = 9;
                self.ram[SPRITE_B_ENDING + k] = 9;
                self.credits_sprite_draw_preexisting_sprite_draw(k, 0x30);
            }
            11 => {
                if r16 >= 0x170 {
                    for k in 4..6 {
                        self.credits_sprite_draw_single(k, 1, 0x3e);
                    }
                    let k = 0usize;
                    self.ram[SPRITE_OAM_FLAGS + k] = 0x39;
                    if r16 < 0x1c0 {
                        self.ram[SPRITE_GRAPHICS + k] = 2;
                    } else if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                        self.ram[SPRITE_DELAY_MAIN + k] = 0x20;
                        self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_GRAPHICS + k] ^ 1) & 1;
                    }
                    self.credits_sprite_draw_single(k, 4, 6);
                } else {
                    const GFX: [u8; 16] = [1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0];
                    for k in 0..2 {
                        self.ram[SPRITE_TYPE + k] = 0x1a;
                        self.ram[SPRITE_OAM_FLAGS + k] = 0x39;
                        self.credits_sprite_draw_set_shadow_prop(k, 2);
                        let bak0 = self.frame_control_view().main_module();
                        self.credits_sprite_draw_activate_and_run_sprite(k, 0x0c);
                        self.frame_control_view_mut().set_main_module(bak0);
                        if self.ram[SPRITE_B_ENDING + k] == 15 && self.ram[SPRITE_A + k] == 4 {
                            self.ram[SPRITE_DELAY_MAIN + k + 2] = 15;
                        }
                        let j = self.ram[SPRITE_DELAY_MAIN + k + 2];
                        if j != 0 {
                            self.ram[SPRITE_OAM_FLAGS + k + 2] = 2;
                            self.ram[SPRITE_GRAPHICS + k + 2] = GFX[j as usize];
                            self.credits_sprite_draw_single(k + 2, 2, 0x36);
                        }
                    }
                }
            }
            12 => {
                let mut k = 6usize;
                self.ram[SPRITE_GRAPHICS + k] = self.ram[FRAME_COUNTER] & 1;
                if self.ram[SPRITE_GRAPHICS + k] == 0 {
                    self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_add(
                        if sign8(self.ram[SPRITE_X_LO + k].wrapping_sub(0x80)) {
                            1
                        } else {
                            0xff
                        },
                    );
                    self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_add(
                        if sign8(self.ram[SPRITE_Y_LO + k].wrapping_sub(0xb0)) {
                            1
                        } else {
                            0xff
                        },
                    );
                    self.sprite_move_xy(k);
                }
                self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_X_VEL + k] >> 1 & 0x40) ^ 0x7e;
                self.ram[SPRITE_FLAGS2 + k] = 1;
                self.ram[SPRITE_FLAGS3 + k] = 0x30;
                self.ram[SPRITE_Z + k] = 16;
                self.credits_sprite_draw_preexisting_sprite_draw(k, 8);
                k -= 1;
                self.ram[SPRITE_OAM_FLAGS + k] = 0x37;
                self.credits_sprite_draw_set_shadow_prop(k, 2);
                self.credits_sprite_draw_activate_and_run_sprite(k, 12);
                k -= 1;
                self.credits_sprite_draw_activate_and_run_sprite(k, 8);
                k -= 1;
                self.credits_sprite_draw_activate_and_run_sprite(k, 8);
                k -= 1;
                loop {
                    const TAB: [u8; 3] = [3, 3, 8];
                    const Z: [u8; 15] = [2, 4, 5, 6, 6, 7, 7, 7, 7, 6, 6, 5, 4, 2, 0];
                    self.credits_sprite_draw_single(k, TAB[k], (k * 2) as u8);
                    if k == 0 {
                        self.ending_func2(k, 0x30);
                    } else if k & !1 != 0 {
                        self.ram[SPRITE_GRAPHICS + k] = self.ram[FRAME_COUNTER] >> 3 & 1;
                    } else {
                        let j = (self.ram[FRAME_COUNTER] & 0x1f) as usize;
                        if j < 0x0f {
                            self.ram[SPRITE_Z + k] = Z[j];
                        }
                        self.ram[SPRITE_GRAPHICS + k] = if j < 0x0f { 1 } else { 0 };
                        self.credits_sprite_draw_draw_shadow(k);
                    }
                    if k == 0 {
                        break;
                    }
                    k -= 1;
                }
            }
            13 => {
                let k = 0usize;
                if r16 == 0x200 {
                    self.ram[SPRITE_X_VEL + k] = (-4i8) as u8;
                }
                self.ram[SPRITE_GRAPHICS + k] = self.ram[FRAME_COUNTER] >> 4 & 1;
                if self.ram[SPRITE_X_LO + k] == 56 {
                    self.ram[SPRITE_X_VEL + k] = 0;
                    self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_GRAPHICS + k].wrapping_add(2);
                }
                self.credits_sprite_draw_single(k, 3, 0x34);
                self.sprite_move_xy(k);
            }
            14 => {
                const TAB1: [u8; 4] = [0, 1, 0, 2];
                const TAB0: [u8; 5] = [2, 8, 32, 32, 8];
                let mut k = 6usize;
                while k != 0 {
                    if k >= 5 {
                        self.ram[SPRITE_TYPE + k] = 0;
                        self.credits_sprite_draw_set_shadow_prop(k, 1);
                        self.ram[SPRITE_GRAPHICS + k] =
                            (self.ram[FRAME_COUNTER].wrapping_add(0x4a) & 8) >> 3;
                        self.ram[SPRITE_Z + k] = 32;
                        self.credits_sprite_draw_circling_birds(k);
                        self.ram[SPRITE_OAM_FLAGS + k] =
                            (self.ram[SPRITE_X_VEL + k] >> 1 & 0x40) ^ 0x0f;
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 8);
                    } else {
                        self.ram[SPRITE_TYPE + k] = 0x0d;
                        if k == 1 {
                            self.ram[SPRITE_HEAD_DIR + k] = 0x0d;
                        }
                        self.credits_sprite_draw_set_shadow_prop(k, 3);
                        self.ram[SPRITE_OAM_FLAGS + k] = 0x2b;
                        let mut a = self.ram[SPRITE_DELAY_MAIN + k];
                        if a == 0 {
                            a = 0xc0;
                            self.ram[SPRITE_DELAY_MAIN + k] = a;
                        }
                        a >>= 1;
                        if a == 0 {
                            self.ram[SPRITE_Y_VEL + k] = 0;
                            self.ram[SPRITE_X_VEL + k] = 0;
                        } else if a < TAB0[k]
                            && self.ram[FRAME_COUNTER] & 3 == 0
                            && self.ram[SPRITE_Y_VEL + k] != 0
                        {
                            let mut v = self.ram[SPRITE_Y_VEL + k].wrapping_sub(1);
                            self.ram[SPRITE_Y_VEL + k] = v;
                            v = v.wrapping_sub(4);
                            if k < 3 {
                                v = (0u8).wrapping_sub(v);
                            }
                            self.ram[SPRITE_X_VEL + k] = v;
                        }
                        self.sprite_move_xy(k);
                        self.ram[SPRITE_GRAPHICS + k] =
                            TAB1[((self.ram[FRAME_COUNTER] >> 3) & 3) as usize];
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 16);
                    }
                    k -= 1;
                }
                self.credits_sprite_draw_single(k, 3, 0x18);
                self.ending_func2(k, 0x20);
            }
            15 => {
                const X: [u8; 4] = [0x76, 0x73, 0x71, 0x78];
                const Y: [u8; 4] = [0x8b, 0x83, 0x8d, 0x85];
                const DELAY: [u8; 8] = [6, 6, 6, 6, 6, 6, 10, 8];
                const OAM_FLAGS: [u8; 4] = [0x61, 0x61, 0x3b, 0x39];
                let sparkle_table = self
                    .asset_raw(73)
                    .unwrap_or_else(|| panic!("missing ending asset 73"));
                let sparkle_idx = sparkle_table[self.ram[FRAME_COUNTER] as usize] & 3;
                self.credits_sprite_draw_add_sparkle(
                    2,
                    X[sparkle_idx as usize],
                    Y[sparkle_idx as usize],
                );
                let mut k = 2usize;
                self.ram[SPRITE_TYPE + k] = 0x62;
                self.ram[SPRITE_OAM_FLAGS + k] = 0x39;
                self.credits_sprite_draw_preexisting_sprite_draw(k, 0x18);
                let mut j = 1u8;
                loop {
                    k += 1;
                    if self.ram[SPRITE_DELAY_AUX1 + k] != 0 {
                        self.ram[SPRITE_DELAY_AUX1 + k] =
                            self.ram[SPRITE_DELAY_AUX1 + k].wrapping_sub(1);
                    }
                    self.ram[SPRITE_OAM_FLAGS + k] =
                        (self.ram[SPRITE_X_VEL + k] >> 1 & 0x40) ^ OAM_FLAGS[j as usize];
                    if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                        self.ram[SPRITE_DELAY_MAIN + k] = 128;
                        self.ram[SPRITE_A + k] = 0;
                    }
                    if self.ram[SPRITE_A + k] == 0 {
                        self.ram[SPRITE_GRAPHICS + k] = (self.ram[FRAME_COUNTER] >> 2 & 1) + 2;
                        self.credits_sprite_draw_move_squirrel(k);
                    } else if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
                        if self.ram[SPRITE_B_ENDING + k] == 8 {
                            self.ram[SPRITE_B_ENDING + k] = 0;
                        }
                        let b = self.ram[SPRITE_B_ENDING + k] & 7;
                        self.ram[SPRITE_DELAY_AUX1 + k] = DELAY[b as usize];
                        self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_GRAPHICS + k] & 1) ^ 1;
                        self.ram[SPRITE_B_ENDING + k] =
                            self.ram[SPRITE_B_ENDING + k].wrapping_add(1);
                    }
                    self.credits_sprite_draw_single(k, 1, 20);
                    self.end_sequence_draw_shadow2(k);
                    if j == 0 {
                        break;
                    }
                    j -= 1;
                }
                self.credits_sprite_draw_walk_link_away_from_pedestal(k + 1);
            }
            _ => {}
        }

        let k = (self.frame_control_view().submodule() >> 1) as usize;
        let r16 = read_le_u16(&self.ram, R16);
        if r16 >= TAB0[k] {
            if r16 & 1 == 0 {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
                if self.ram[INIDISP_COPY] == 0 {
                    self.frame_control_view_mut().increment_submodule();
                } else {
                    write_le_u16(&mut self.ram, R16, r16.wrapping_add(1));
                }
            } else {
                write_le_u16(&mut self.ram, R16, r16.wrapping_add(1));
            }
        } else {
            if r16 & 1 == 0 && self.ram[INIDISP_COPY] != 15 {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_add(1);
            }
            write_le_u16(&mut self.ram, R16, r16.wrapping_add(1));
        }
        copy_word(&mut self.ram, BG2HOFS_COPY, BG2HOFS_COPY2);
        copy_word(&mut self.ram, BG2VOFS_COPY, BG2VOFS_COPY2);
        copy_word(&mut self.ram, BG1HOFS_COPY, BG1HOFS_COPY2);
        copy_word(&mut self.ram, BG1VOFS_COPY, BG1VOFS_COPY2);
    }

    pub(super) fn credits_sprite_draw_draw_shadow(&mut self, k: usize) {
        self.ram[SPRITE_OAM_FLAGS + k] = 0x30;
        self.credits_sprite_draw_set_shadow_prop(k, 0);
        self.oam_allocate_from_region_a(4);
        let mut info = self.ending_coords;
        self.sprite_draw_shadow_custom(k, &mut info, 10);
        self.ending_coords = info;
    }

    pub(super) fn end_sequence_draw_shadow2(&mut self, k: usize) {
        self.credits_sprite_draw_set_shadow_prop(k, 0);
        self.oam_allocate_from_region_a(4);
        let mut info = self.ending_coords;
        self.sprite_draw_shadow_custom(k, &mut info, 10);
        self.ending_coords = info;
    }

    pub(super) fn ending_func2(&mut self, k: usize, ain: u8) {
        const DELAY: [u8; 27] = [
            10, 10, 10, 10, 20, 8, 8, 0, 255, 12, 12, 12, 12, 12, 12, 30, 8, 4, 4, 4, 0, 0, 255,
            255, 144, 4, 0,
        ];
        const TAB0: [i8; 28] = [
            0, 0, 1, 0, 1, 0, 2, 3, 0, 2, 0, 1, 0, 1, 0, 1, 2, 3, 4, 5, 6, 3, 0, -1, -1, -1, 2, 3,
        ];
        self.ram[SPRITE_OAM_FLAGS + k] = ain;
        self.end_sequence_draw_shadow2(k);
        let mut j = self.ram[SPRITE_A + k];
        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            j = j.wrapping_add(1);
            if j == 8 {
                j = 6;
            } else if j == 22 {
                j = 21;
            } else if j == 28 {
                j = 27;
            }
            self.ram[SPRITE_A + k] = j;
            self.ram[SPRITE_DELAY_MAIN + k] = DELAY[j.wrapping_sub(1) as usize];
        }
        let a = TAB0[j as usize];
        self.ram[SPRITE_GRAPHICS + k] = if a == -1 {
            self.ram[FRAME_COUNTER] >> 3 & 1
        } else {
            a as u8
        };
        if (j < 5 || (10..15).contains(&j)) && self.ram[FRAME_COUNTER] & 1 == 0 {
            self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(1);
        }
    }

    pub(super) fn credits_sprite_draw_activate_and_run_sprite(&mut self, k: usize, a: u8) {
        self.ram[CUR_OBJECT_INDEX] = k as u8;
        self.oam_allocate_from_region_a(a);
        self.sprite_get_16_bit_coords_ending(k);
        let bak0 = self.frame_control_view().submodule();
        self.frame_control_view_mut().set_submodule(0);
        self.ram[SPRITE_STATE + k] = 9;
        self.sprite_active_main_ending(k);
        self.frame_control_view_mut().set_submodule(bak0);
    }

    pub(super) fn credits_sprite_draw_preexisting_sprite_draw(&mut self, k: usize, a: u8) {
        self.oam_allocate_from_region_a(a);
        self.ram[CUR_OBJECT_INDEX] = k as u8;
        self.sprite_get_16_bit_coords_ending(k);
        self.sprite_active_main_ending(k);
    }

    pub(super) fn credits_sprite_draw_single(&mut self, k: usize, a: u8, j: u8) {
        self.oam_allocate_from_region_a(a.wrapping_mul(4));
        self.sprite_get_16_bit_coords_ending(k);
        let entries = K_END_SEQUENCE_DMDS[(j >> 1) as usize];
        let start = a as usize * self.ram[SPRITE_GRAPHICS + k] as usize;
        let dmd: Vec<DrawMultipleData> = entries[start..start + a as usize]
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect();
        let mut info = PrepOamCoordsRet::default();
        self.sprite_draw_multiple(k, &dmd, Some(&mut info));
        self.ending_coords = info;
    }

    pub(super) fn credits_sprite_draw_set_shadow_prop(&mut self, k: usize, a: u8) {
        self.ram[SPRITE_FLAGS2 + k] = a;
        self.ram[SPRITE_FLAGS3 + k] = 16;
    }

    pub(super) fn credits_sprite_draw_add_sparkle(&mut self, j_count: usize, xb: u8, yb: u8) {
        const DELAY: [u8; 6] = [32, 4, 4, 4, 5, 6];
        self.ram[SPRITE_C] = j_count as u8;
        for k in 0..j_count {
            let mut j = self.ram[SPRITE_GRAPHICS + k];
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                j = j.wrapping_add(1);
                if j >= 6 {
                    self.ram[SPRITE_X_LO + k] = xb;
                    self.ram[SPRITE_Y_LO + k] = yb;
                    j = 0;
                }
                self.ram[SPRITE_GRAPHICS + k] = j;
                self.ram[SPRITE_DELAY_MAIN + k] = DELAY[j as usize];
            }
            if j != 0 {
                self.credits_sprite_draw_single(k, 1, 0x1c);
            }
        }
    }

    pub(super) fn credits_sprite_draw_walk_link_away_from_pedestal(&mut self, k: usize) {
        const DMA: [u16; 8] = [0x16c, 0x16e, 0x170, 0x172, 0x16c, 0x174, 0x176, 0x178];
        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_GRAPHICS + k].wrapping_add(1) & 7;
            self.ram[SPRITE_DELAY_MAIN + k] = 4;
        }
        let dma = DMA[self.ram[SPRITE_GRAPHICS + k] as usize];
        write_le_u16(&mut self.ram, LINK_DMA_GRAPHICS_INDEX, dma);
        self.ram[SPRITE_OAM_FLAGS + k] = 32;
        self.credits_sprite_draw_single(k, 2, 26);
        self.end_sequence_draw_shadow2(k);
        self.sprite_move_xy(k);
    }

    pub(super) fn credits_sprite_draw_move_squirrel(&mut self, k: usize) {
        const XVEL: [i8; 4] = [32, 24, -32, -24];
        const YVEL: [i8; 4] = [8, -8, -8, 8];
        if self.ram[SPRITE_DELAY_MAIN + k] < 64 {
            self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1) & 3;
            self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
        } else {
            let j = self.ram[SPRITE_C + k] as usize;
            self.ram[SPRITE_X_VEL + k] = XVEL[j] as u8;
            self.ram[SPRITE_Y_VEL + k] = YVEL[j] as u8;
            self.sprite_move_xy(k);
        }
    }

    pub(super) fn credits_sprite_draw_circling_birds(&mut self, k: usize) {
        const TARGET_X: [i8; 2] = [0x20, -0x20];
        const TARGET_Y: [i8; 2] = [0x10, -0x10];
        let j = self.ram[SPRITE_D + k] & 1;
        self.ram[SPRITE_X_VEL + k] =
            self.ram[SPRITE_X_VEL + k].wrapping_add(if j != 0 { 0xff } else { 1 });
        if self.ram[SPRITE_X_VEL + k] == TARGET_X[j as usize] as u8 {
            self.ram[SPRITE_D + k] = self.ram[SPRITE_D + k].wrapping_add(1);
        }
        if self.ram[FRAME_COUNTER] & 1 == 0 {
            let j = self.ram[SPRITE_HEAD_DIR + k] & 1;
            self.ram[SPRITE_Y_VEL + k] =
                self.ram[SPRITE_Y_VEL + k].wrapping_add(if j != 0 { 0xff } else { 1 });
            if self.ram[SPRITE_Y_VEL + k] == TARGET_Y[j as usize] as u8 {
                self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_HEAD_DIR + k].wrapping_add(1);
            }
        }
        self.sprite_move_xy(k);
    }

    pub(super) fn credits_handle_camera_scroll_control(&mut self) {
        if self.ram[LINK_Y_VEL] != 0 {
            let y_vel = self.ram[LINK_Y_VEL] as i8;
            add_i8_to_word(&mut self.ram, BG2VOFS_COPY2, y_vel);
            let which = if y_vel < 0 {
                OVERWORLD_SCROLL_UP_COUNTER
            } else {
                OVERWORLD_SCROLL_DOWN_COUNTER
            };
            let other = if y_vel < 0 {
                OVERWORLD_SCROLL_DOWN_COUNTER
            } else {
                OVERWORLD_SCROLL_UP_COUNTER
            };
            let mut value = read_le_u16(&self.ram, which).wrapping_add(y_vel.unsigned_abs() as u16);
            if (value as i16).wrapping_sub(0x10) >= 0 {
                value = value.wrapping_sub(0x10);
                let bits = read_le_u16(&self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2)
                    | if y_vel < 0 { 8 } else { 4 };
                write_le_u16(&mut self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, bits);
            }
            write_le_u16(&mut self.ram, which, value);
            write_le_u16(&mut self.ram, other, 0u16.wrapping_sub(value));
            let mut r4 = y_vel as i16 as u16;
            write_le_u16(&mut self.ram, OVERWORLD_SCROLL_DELTA, r4);
            let oi = self.ram[OVERLAY_INDEX];
            if oi != 0x97 && oi != 0x9d {
                let subp;
                if oi == 0xb5 || oi == 0xbe {
                    subp = (r4 & 3) << 14;
                    r4 = ((r4 as i16) >> 2) as u16;
                } else {
                    subp = (r4 & 1) << 15;
                    r4 = ((r4 as i16) >> 1) as u16;
                }
                add_bg1_subpixel(&mut self.ram, BG1VOFS_SUBPIXEL, BG1VOFS_COPY2, subp, r4);
            }
        }
        if self.ram[LINK_X_VEL] != 0 {
            let x_vel = self.ram[LINK_X_VEL] as i8;
            add_i8_to_word(&mut self.ram, BG2HOFS_COPY2, x_vel);
            let which = if x_vel < 0 {
                OVERWORLD_SCROLL_LEFT_COUNTER
            } else {
                OVERWORLD_SCROLL_RIGHT_COUNTER
            };
            let other = if x_vel < 0 {
                OVERWORLD_SCROLL_RIGHT_COUNTER
            } else {
                OVERWORLD_SCROLL_LEFT_COUNTER
            };
            let mut value = read_le_u16(&self.ram, which).wrapping_add(x_vel.unsigned_abs() as u16);
            if (value as i16).wrapping_sub(0x10) >= 0 {
                value = value.wrapping_sub(0x10);
                let bits = read_le_u16(&self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2)
                    | if x_vel < 0 { 2 } else { 1 };
                write_le_u16(&mut self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, bits);
            }
            write_le_u16(&mut self.ram, which, value);
            write_le_u16(&mut self.ram, other, 0u16.wrapping_sub(value));
            let mut r4 = x_vel as i16 as u16;
            write_le_u16(&mut self.ram, OVERWORLD_SCROLL_DELTA + 1, r4);
            let oi = self.ram[OVERLAY_INDEX];
            if oi != 0x97 && oi != 0x9d && r4 != 0 {
                let subp;
                if oi == 0x95 || oi == 0x9e {
                    subp = (r4 & 3) << 14;
                    r4 = ((r4 as i16) >> 2) as u16;
                } else {
                    subp = (r4 & 1) << 15;
                    r4 = ((r4 as i16) >> 1) as u16;
                }
                add_bg1_subpixel(&mut self.ram, BG1HOFS_SUBPIXEL, BG1HOFS_COPY2, subp, r4);
            }
        }
        if self.ram[OVERLAY_INDEX] == 0x9c {
            sub_bg1_subpixel(&mut self.ram, BG1VOFS_SUBPIXEL, BG1VOFS_COPY2, 0x2000, 0);
            let bg1v = read_le_u16(&self.ram, BG1VOFS_COPY2)
                .wrapping_add(read_le_u16(&self.ram, OVERWORLD_SCROLL_DELTA));
            write_le_u16(&mut self.ram, BG1VOFS_COPY2, bg1v);
            copy_le_u16(&mut self.ram, BG1HOFS_COPY2, BG2HOFS_COPY2);
        } else if self.ram[OVERLAY_INDEX] == 0x97 || self.ram[OVERLAY_INDEX] == 0x9d {
            add_bg1_subpixel(&mut self.ram, BG1VOFS_SUBPIXEL, BG1VOFS_COPY2, 0x2000, 0);
            add_bg1_subpixel(&mut self.ram, BG1HOFS_SUBPIXEL, BG1HOFS_COPY2, 0x2000, 0);
        }
        if self.world_state_view().dungeon_room() == 0x181 {
            let bg2v = read_le_u16(&self.ram, BG2VOFS_COPY2) | 0x100;
            write_le_u16(&mut self.ram, BG1VOFS_COPY2, bg2v);
            copy_le_u16(&mut self.ram, BG1HOFS_COPY2, BG2HOFS_COPY2);
        }
    }

    pub(super) fn end_sequence_32(&mut self) {
        const HEALTH_AFTER_DEATH: [u8; 21] = [
            0x18, 0x18, 0x18, 0x18, 0x18, 0x20, 0x20, 0x28, 0x28, 0x30, 0x30, 0x38, 0x38, 0x38,
            0x40, 0x40, 0x40, 0x48, 0x48, 0x48, 0x50,
        ];
        self.enable_force_blank();
        self.erase_tile_maps_triforce();
        self.transfer_font_to_vram();
        self.credits_load_cool_background();
        self.credits_initialize_polyhedral();
        self.ram[INIDISP_COPY] = 128;
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x200);
        self.ram[HUD_PALETTE] = 1;
        self.palette_load_hud();
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        write_le_u16(&mut self.ram, DEATHS_PER_PALACE + 4 * 2, 0);
        let palace_13 = read_le_u16(&self.ram, DEATHS_PER_PALACE + 13 * 2)
            .wrapping_add(read_le_u16(&self.ram, DEATH_SAVE_COUNTER));
        write_le_u16(&mut self.ram, DEATHS_PER_PALACE + 13 * 2, palace_13);
        let mut sum = palace_13;
        for i in (0..=12).rev() {
            sum = sum.wrapping_add(read_le_u16(&self.ram, DEATHS_PER_PALACE + i * 2));
        }
        write_le_u16(&mut self.ram, DEATH_VAR2, sum);
        write_le_u16(&mut self.ram, DEATH_SAVE_COUNTER, 0);
        self.ram[LINK_HEALTH_CURRENT] =
            HEALTH_AFTER_DEATH[(self.ram[LINK_HEALTH_CAPACITY] >> 3) as usize];
        self.ram[SAVEGAME_IS_DARKWORLD] = 0x40;
        self.SaveGameFile();
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + 38 * 2, 0);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 38 * 2, 0);
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER, 0);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, 0);
        self.ram[TM_COPY] = 0x16;
        self.ram[TS_COPY] = 0;
        write_le_u16(&mut self.ram, R16, 0x6800);
        write_le_u16(&mut self.ram, R18, 0);
        write_le_u16(&mut self.ram, ENDING_WHICH_DUNG, 0);
        write_le_u16(&mut self.ram, BG2VOFS_COPY2, (-0x48i16) as u16);
        write_le_u16(&mut self.ram, BG2HOFS_COPY2, 0x90);
        write_le_u16(&mut self.ram, BG3VOFS_COPY2, 0);
        write_le_u16(&mut self.ram, BG3HOFS_COPY2, 0);
        self.credits_add_next_attribution();
        self.ram[MUSIC_CONTROL] = 0x22;
        self.ram[CGWSEL_COPY] = 0;
        self.ram[CGADSUB_COPY] = 162;
        self.zelda_ppu_write(0x2108, 0x13);
        self.ram[COLDATA_COPY0] = 0x3f;
        self.ram[COLDATA_COPY1] = 0x5f;
        self.ram[COLDATA_COPY2] = 0x9f;
        self.frame_control_view_mut().set_subsubmodule(64);
        self.ram[INIDISP_COPY] = 0;
        self.hdma_setup(0, 0xebd53, 0x42, 0, BG2HOFS as u8, 0);
        self.ram[HDMAEN_COPY] = 0x80;
        copy_le_u16(&mut self.ram, BG2HOFS_COPY, BG2HOFS_COPY2);
        copy_le_u16(&mut self.ram, BG2VOFS_COPY, BG2VOFS_COPY2);
        copy_le_u16(&mut self.ram, BG1HOFS_COPY, BG1HOFS_COPY2);
        copy_le_u16(&mut self.ram, BG1VOFS_COPY, BG1VOFS_COPY2);
    }

    pub(super) fn credits_fade_out_fixed_col(&mut self) {
        self.frame_control_view_mut().decrement_subsubmodule();
        if self.frame_control_view().subsubmodule() == 0 {
            self.frame_control_view_mut().set_subsubmodule(16);
            if self.ram[COLDATA_COPY0] != 32 {
                self.ram[COLDATA_COPY0] = self.ram[COLDATA_COPY0].wrapping_sub(1);
            } else if self.ram[COLDATA_COPY1] != 64 {
                self.ram[COLDATA_COPY1] = self.ram[COLDATA_COPY1].wrapping_sub(1);
            } else if self.ram[COLDATA_COPY2] != 128 {
                self.ram[COLDATA_COPY2] = self.ram[COLDATA_COPY2].wrapping_sub(1);
            }
        }
    }

    pub(super) fn credits_fade_color_and_begin_animating(&mut self) {
        self.credits_fade_out_fixed_col();
        self.ram[NMI_DISABLE_CORE_UPDATES] = 1;
        self.credits_animate_the_triangles();
        if self.ram[FRAME_COUNTER] & 3 == 0 {
            let bg2 = read_le_u16(&self.ram, BG2HOFS_COPY2).wrapping_add(1);
            write_le_u16(&mut self.ram, BG2HOFS_COPY2, bg2);
            if bg2 == 0x0c00 {
                self.zelda_ppu_write(0x2108, 0x13);
            }
            let a1 = bg2 >> 1;
            let a0 = a1.wrapping_add(bg2);
            write_le_u16(&mut self.ram, ROOM_BOUNDS_Y, a0);
            write_le_u16(&mut self.ram, ROOM_BOUNDS_Y + 2, a0 >> 1);
            write_le_u16(&mut self.ram, ROOM_BOUNDS_Y + 4, a1);
            write_le_u16(&mut self.ram, ROOM_BOUNDS_Y + 6, a1 >> 1);
            if read_le_u16(&self.ram, BG3VOFS_COPY2) == 3288 {
                write_le_u16(&mut self.ram, R16, 0x80);
                self.frame_control_view_mut().increment_submodule();
            } else {
                add_i8_to_word(&mut self.ram, BG3VOFS_COPY2, 1);
                let bg3v = read_le_u16(&self.ram, BG3VOFS_COPY2);
                if bg3v & 7 == 0 {
                    write_le_u16(&mut self.ram, R18, bg3v >> 3);
                    self.credits_add_next_attribution();
                }
            }
        }
        copy_le_u16(&mut self.ram, BG2HOFS_COPY, BG2HOFS_COPY2);
        copy_le_u16(&mut self.ram, BG2VOFS_COPY, BG2VOFS_COPY2);
        copy_le_u16(&mut self.ram, BG1HOFS_COPY, BG1HOFS_COPY2);
        copy_le_u16(&mut self.ram, BG1VOFS_COPY, BG1VOFS_COPY2);
    }

    pub(super) fn credits_add_next_attribution(&mut self) {
        const TAB2: [usize; 14] = [1, 0, 2, 3, 10, 6, 5, 8, 11, 9, 7, 12, 13, 15];
        const DIGITS_SCROLL_Y: [u16; 14] = [
            0x290, 0x298, 0x2a0, 0x2a8, 0x2b0, 0x2ba, 0x2c2, 0x2ca, 0x2d2, 0x2da, 0x2e2, 0x2ea,
            0x2f2, 0x310,
        ];
        const DIGIT_CHAR: [u16; 2] = [0x3ce6, 0x3cf6];
        let mut dst = VRAM_UPLOAD_DATA + read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET) as usize;
        let mut r16 = read_le_u16(&self.ram, R16);

        write_le_u16(&mut self.ram, dst, r16.swap_bytes());
        write_le_u16(&mut self.ram, dst + 2, 0x3e40);
        let blank_tile = self.ending_asset_u16(76, 159);
        write_le_u16(&mut self.ram, dst + 4, blank_tile);
        dst += 6;

        let r18 = read_le_u16(&self.ram, R18) as usize;
        if r18 < 394 {
            let text_off = self.ending_asset_u16(75, r18) as usize;
            let text = self
                .asset_raw(74)
                .unwrap_or_else(|| panic!("missing ending asset 74"))
                .to_vec();
            if text[text_off] != 0xff {
                let addr_delta = text[text_off] as u16;
                let n = text[text_off + 1];
                write_le_u16(
                    &mut self.ram,
                    dst,
                    r16.wrapping_add(addr_delta).swap_bytes(),
                );
                write_le_u16(&mut self.ram, dst + 2, (n as u16).swap_bytes());
                dst += 4;
                let count = ((n.wrapping_add(1)) >> 1) as usize;
                for q in 0..count {
                    let ch = text[text_off + 2 + q] as usize;
                    let tile = self.ending_asset_u16(76, ch);
                    write_le_u16(&mut self.ram, dst, tile);
                    dst += 2;
                }
            }

            let mut which = read_le_u16(&self.ram, ENDING_WHICH_DUNG);
            let which_idx = (which >> 1) as usize;
            if (which & 1) != 0 || (r18 as u16).wrapping_mul(2) == DIGITS_SCROLL_Y[which_idx] {
                let t = DIGIT_CHAR[(which & 1) as usize];
                write_le_u16(&mut self.ram, ENDING_CREDIT_DIGIT_CHAR, t);
                write_le_u16(&mut self.ram, dst, r16.wrapping_add(0x19).swap_bytes());
                write_le_u16(&mut self.ram, dst + 2, 0x0500);
                let palace = TAB2[which_idx];
                let mut deaths = read_le_u16(&self.ram, DEATHS_PER_PALACE + palace * 2);
                if deaths >= 1000 {
                    deaths = 999;
                }
                write_le_u16(&mut self.ram, dst + 8, t.wrapping_add(deaths % 10));
                deaths /= 10;
                write_le_u16(&mut self.ram, dst + 6, t.wrapping_add(deaths % 10));
                deaths /= 10;
                write_le_u16(&mut self.ram, dst + 4, t.wrapping_add(deaths));
                dst += 10;
                which = which.wrapping_add(1);
                write_le_u16(&mut self.ram, ENDING_WHICH_DUNG, which);
            }
        }

        r16 = r16.wrapping_add(0x20);
        if r16 & 0x3ff == 0 {
            r16 = (r16 & 0x6800) ^ 0x800;
        }
        write_le_u16(&mut self.ram, R16, r16);
        write_le_u16(
            &mut self.ram,
            VRAM_UPLOAD_OFFSET,
            (dst - VRAM_UPLOAD_DATA) as u16,
        );
        self.ram[dst] = 0xff;
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn credits_add_ending_sequence_text(&mut self) {
        let mut dst = VRAM_UPLOAD_DATA;
        write_le_u16(&mut self.ram, dst, 0x0060);
        write_le_u16(&mut self.ram, dst + 2, 0xfe47);
        let blank_tile = self.ending_asset_u16(76, 159);
        write_le_u16(&mut self.ram, dst + 4, blank_tile);
        dst += 6;

        let scene = (self.frame_control_view().submodule() >> 1) as usize;
        let mut curo = self.ending_asset_u16(77, scene) as usize;
        let endo = self.ending_asset_u16(77, scene + 1) as usize;
        let data = self
            .asset_raw(78)
            .unwrap_or_else(|| panic!("missing ending asset 78"))
            .to_vec();
        while curo != endo {
            let a = u16::from_le_bytes([data[curo], data[curo + 1]]);
            let b = u16::from_le_bytes([data[curo + 2], data[curo + 3]]);
            write_le_u16(&mut self.ram, dst, a);
            write_le_u16(&mut self.ram, dst + 2, b);
            let m = ((b >> 9) & 0x7f) as usize;
            dst += 4;
            curo += 4;
            for _ in 0..=m {
                let ch = data[curo] as usize;
                let tile = self.ending_asset_u16(76, ch);
                write_le_u16(&mut self.ram, dst, tile);
                dst += 2;
                curo += 1;
            }
        }
        write_le_u16(
            &mut self.ram,
            VRAM_UPLOAD_OFFSET,
            (dst - VRAM_UPLOAD_DATA) as u16,
        );
        self.ram[dst] = 0xff;
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn credits_brighten_triangles(&mut self) {
        if self.ram[FRAME_COUNTER] & 15 == 0 {
            self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_add(1);
            if self.ram[INIDISP_COPY] == 15 {
                self.frame_control_view_mut().increment_submodule();
            }
        }
        self.credits_animate_the_triangles();
    }

    pub(super) fn credits_stop_credits_scroll(&mut self) {
        self.ram[R16] = self.ram[R16].wrapping_sub(1);
        if self.ram[R16] == 0 {
            write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, 0);
            write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
            write_le_u16(&mut self.ram, MOSAIC_TARGET_LEVEL, 0x1f);
            self.frame_control_view_mut().increment_submodule();
            write_le_u16(&mut self.ram, R16, 0x00c0);
            write_le_u16(&mut self.ram, R18, 0);
        }
        self.credits_animate_the_triangles();
    }

    pub(super) fn credits_fade_and_disperse_triangles(&mut self) {
        self.ram[R16] = self.ram[R16].wrapping_sub(1);
        if self.ram[R18] == 0 {
            self.apply_palette_filter_bounce();
            if self.ram[PALETTE_FILTER_COUNTDOWN] != 0 {
                self.credits_animate_the_triangles();
                return;
            }
            self.ram[R18] = self.ram[R18].wrapping_add(1);
        }
        if self.ram[R16] != 0 {
            self.credits_animate_the_triangles();
            return;
        }
        self.frame_control_view_mut().increment_submodule();
        self.PaletteFilter_WishPonds_Inner();
    }

    pub(super) fn credits_fade_in_the_end(&mut self) {
        if self.ram[FRAME_COUNTER] & 7 == 0 {
            self.PaletteFilter_SP5F();
            if self.ram[PALETTE_FILTER_COUNTDOWN] == 0 {
                self.frame_control_view_mut().increment_submodule();
            }
        }
        self.credits_hang_forever();
    }

    pub(super) fn credits_hang_forever(&mut self) {
        self.set_oam_plain(0, 0xa0, 0xb8, 0x00, 0x3b, 2);
        self.set_oam_plain(1, 0xb0, 0xb8, 0x02, 0x3b, 2);
        self.set_oam_plain(2, 0xc0, 0xb8, 0x04, 0x3b, 2);
        self.set_oam_plain(3, 0xd0, 0xb8, 0x06, 0x3b, 2);
    }

    pub(super) fn crystal_cutscene_initialize_polyhedral(&mut self) {
        self.ram[POLY_CONFIG1] = 156;
        self.ram[POLY_CONFIG_COLOR_MODE] = 1;
        self.ram[IS_NMI_THREAD_ACTIVE] = 1;
        self.ram[INTRO_DID_RUN_STEP] = 1;
        self.ram[POLY_BASE_X] = 32;
        self.ram[POLY_BASE_Y] = 32;
        self.ram[POLY_VAR1] = 32;
        self.ram[POLY_WHICH_MODEL] = 0;
        self.ram[POLY_A] = 16;
        self.ram[TS_COPY] = 0;
        self.ram[TM_COPY] = 0x16;
    }
}

fn add_i8_to_word(ram: &mut [u8], offset: usize, value: i8) {
    let next = read_le_u16(ram, offset).wrapping_add(value as i16 as u16);
    write_le_u16(ram, offset, next);
}

fn copy_word(ram: &mut [u8], dst: usize, src: usize) {
    let value = read_le_u16(ram, src);
    write_le_u16(ram, dst, value);
}

fn add_bg1_subpixel(ram: &mut [u8], subpixel_addr: usize, copy_addr: usize, subp: u16, r4: u16) {
    let tmp =
        (read_le_u16(ram, subpixel_addr) as u32) | ((read_le_u16(ram, copy_addr) as u32) << 16);
    let tmp = tmp.wrapping_add((subp as u32) | ((r4 as u32) << 16));
    write_le_u16(ram, subpixel_addr, tmp as u16);
    write_le_u16(ram, copy_addr, (tmp >> 16) as u16);
}

fn sub_bg1_subpixel(ram: &mut [u8], subpixel_addr: usize, copy_addr: usize, subp: u16, r4: u16) {
    let tmp =
        (read_le_u16(ram, subpixel_addr) as u32) | ((read_le_u16(ram, copy_addr) as u32) << 16);
    let tmp = tmp.wrapping_sub((subp as u32) | ((r4 as u32) << 16));
    write_le_u16(ram, subpixel_addr, tmp as u16);
    write_le_u16(ram, copy_addr, (tmp >> 16) as u16);
}

fn break_triforce_handle_poly(state: &mut ZeldaState) {
    state.ram[INTRO_DID_RUN_STEP] = 1;
    state.ram[INTRO_WANT_DOUBLE_RET] = 0;
    state.ram[INTRO_FRAME_CTR] = state.ram[INTRO_FRAME_CTR].wrapping_add(1);
}

impl ZeldaState {
    pub(super) fn fade_music_and_reset_sram_mirror(&mut self) {
        self.ram[IRQ_FLAG] = 0xff;
        self.ram[TM_COPY] = 0x15;
        self.ram[TS_COPY] = 0;
        self.ram[PLAYER_IS_INDOORS] = 0;
        self.ram[MUSIC_CONTROL] = 0xf1;
        self.set_backdrop_color_black();
        self.ram[LINK_Y_COORD..LINK_Y_COORD + 0x70].fill(0);
        self.ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + 256 * 5].fill(0);
        self.frame_control_view_mut().set_main_module(1);
        self.ram[RESTART_CHECK_FLAG] = 1;
        self.frame_control_view_mut().set_submodule(0);
    }

    pub(super) fn load_triforce_sprite_palette(&mut self) {
        const POLYHEDRAL_PALETTE: [u16; 8] =
            [0, 0x014d, 0x01b0, 0x01f3, 0x0256, 0x0279, 0x02fd, 0x035f];
        for (i, color) in POLYHEDRAL_PALETTE.iter().enumerate() {
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (0xd0 + i) * 2, *color);
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn module00_intro(&mut self) {
        let skip_at =
            if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_SKIP_INTRO_ON_KEYPRESS != 0 {
                4
            } else {
                8
            };
        if self.frame_control_view().submodule() >= skip_at
            && (((self.ram[FILTERED_JOYPAD_L] & 0xc0) | self.ram[FILTERED_JOYPAD_H]) & 0xd0) != 0
        {
            self.fade_music_and_reset_sram_mirror();
            return;
        }

        match self.frame_control_view().submodule() {
            0 => self.intro_init(),
            1 => self.intro_init_continue(),
            2 | 10 => self.intro_initialize_triforce_poly_thread(),
            3 | 4 | 9 | 11 => self.intro_handle_all_triforce_animations(),
            5 => self.intro_zelda_fadein(),
            6 => self.intro_sword_coming_down(),
            7 => self.intro_fade_in_bg(),
            8 => self.intro_wait_player(),
            _ => {}
        }
    }

    pub(super) fn intro_zelda_fadein(&mut self) {
        self.intro_handle_all_triforce_animations();
        if self.ram[FRAME_COUNTER] & 1 == 0 {
            return;
        }
        self.palette_fade_intro_one_step();
        if self.ram[PALETTE_FILTER_COUNTDOWN] == 0 {
            self.frame_control_view_mut().set_subsubmodule(42);
            self.frame_control_view_mut().increment_submodule();
            self.intro_setup_sword_and_intro_flash();
        } else if self.ram[PALETTE_FILTER_COUNTDOWN] == 13 {
            self.ram[TM_COPY] = 0x15;
            self.ram[TS_COPY] = 0;
        }
    }

    pub(super) fn intro_setup_sword_and_intro_flash(&mut self) {
        self.ram[INTRO_SWORD_19] = 7;
        self.ram[INTRO_SWORD_20] = 0;
        self.ram[INTRO_SWORD_21] = 0;
        write_le_u16(&mut self.ram, INTRO_SWORD_YPOS, (-130i16) as u16);
        self.intro_periodic_sword_and_intro_flash();
    }

    pub(super) fn intro_sword_coming_down(&mut self) {
        self.intro_handle_all_triforce_animations();
        self.ram[INTRO_DID_RUN_STEP] = 0;
        self.ram[IS_NMI_THREAD_ACTIVE] = 0;
        self.intro_periodic_sword_and_intro_flash();
        self.frame_control_view_mut().decrement_subsubmodule();
        if self.frame_control_view().subsubmodule() == 0 {
            self.frame_control_view_mut().increment_submodule();
            self.ram[CGWSEL_COPY] = 2;
            self.ram[CGADSUB_COPY] = 0x22;
            write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 31);
            self.ram[TS_COPY] = 2;
        }
    }

    pub(super) fn intro_fade_in_bg(&mut self) {
        self.intro_periodic_sword_and_intro_flash();
        self.intro_handle_all_triforce_animations();
        if self.ram[PALETTE_FILTER_COUNTDOWN] != 0 {
            if self.ram[FRAME_COUNTER] & 1 != 0 {
                self.palette_fade_intro2();
            }
        } else if (((self.ram[FILTERED_JOYPAD_L] & 0xc0) | self.ram[FILTERED_JOYPAD_H]) & 0xd0) != 0
        {
            self.fade_music_and_reset_sram_mirror();
        } else {
            self.frame_control_view_mut().decrement_subsubmodule();
            if self.frame_control_view().subsubmodule() == 0 {
                self.frame_control_view_mut().increment_submodule();
            }
        }
    }

    pub(super) fn intro_wait_player(&mut self) {
        self.intro_handle_all_triforce_animations();
        self.ram[INTRO_DID_RUN_STEP] = 0;
        self.ram[IS_NMI_THREAD_ACTIVE] = 0;
        self.intro_periodic_sword_and_intro_flash();
        self.frame_control_view_mut().decrement_subsubmodule();
        if self.frame_control_view().subsubmodule() == 0 {
            self.frame_control_view_mut().increment_submodule();
            self.frame_control_view_mut().set_main_module(20);
            self.frame_control_view_mut().set_submodule(0);
            self.ram[LINK_X_COORD] = 0;
        }
    }

    pub(super) fn intro_periodic_sword_and_intro_flash(&mut self) {
        if self.ram[INTRO_SWORD_18] != 0 {
            self.ram[INTRO_SWORD_18] = self.ram[INTRO_SWORD_18].wrapping_sub(1);
        }
        self.set_backdrop_color_black();
        if self.ram[INTRO_TIMES_PAL_FLASH] != 0 {
            if self.ram[INTRO_TIMES_PAL_FLASH] & 3 != 0 {
                let color = COLDATA_COPY0 + self.ram[INTRO_SWORD_24] as usize;
                let flash = if self.read_u32_ram(ENHANCED_FEATURES0)
                    & K_FEATURES0_DIM_FLASHES_ENDING
                    != 0
                {
                    0x05
                } else {
                    0x1f
                };
                self.ram[color] |= flash;
                self.ram[INTRO_SWORD_24] = if self.ram[INTRO_SWORD_24] == 2 {
                    0
                } else {
                    self.ram[INTRO_SWORD_24].wrapping_add(1)
                };
            }
            self.ram[INTRO_TIMES_PAL_FLASH] = self.ram[INTRO_TIMES_PAL_FLASH].wrapping_sub(1);
        }

        const CHARS: [u8; 10] = [0, 2, 0x20, 0x22, 4, 6, 8, 0x0a, 0x0c, 0x0e];
        const XS: [u8; 10] = [0x40, 0x40, 0x30, 0x50, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40];
        const YS: [u16; 10] = [0x10, 0x20, 0x28, 0x28, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80];
        let sword_y = read_le_u16(&self.ram, INTRO_SWORD_YPOS);
        for j in (0..10).rev() {
            let y = sword_y.wrapping_add(YS[j]);
            let visible_y = if y & 0xff00 != 0 { 0xf8u8 } else { y as u8 }.wrapping_sub(8);
            self.set_oam_plain(0x52 + j, XS[j], visible_y, CHARS[j], 0x21, 2);
        }

        if sword_y != 30 {
            if sword_y == 0xffbe {
                self.ram[SOUND_EFFECT_1] = 1;
            } else if sword_y == 14 {
                write_le_u16(&mut self.ram, INTRO_SWORD_24, 0);
                self.ram[INTRO_TIMES_PAL_FLASH] = 0x20;
                self.ram[SOUND_EFFECT_1] = 0x2c;
            }
            write_le_u16(&mut self.ram, INTRO_SWORD_YPOS, sword_y.wrapping_add(16));
        }

        match self.ram[INTRO_SWORD_20] >> 1 {
            0 => {
                if self.ram[INTRO_TIMES_PAL_FLASH] == 0
                    && read_le_u16(&self.ram, INTRO_SWORD_YPOS) == 30
                {
                    self.ram[INTRO_SWORD_20] = self.ram[INTRO_SWORD_20].wrapping_add(2);
                }
            }
            1 => {
                const TAB: [u8; 8] = [4, 4, 6, 6, 6, 4, 4, 0];
                const SPARKLE_CHARS: [u8; 7] = [0x28, 0x37, 0x27, 0x36, 0x27, 0x37, 0x28];
                if self.ram[INTRO_SWORD_18] == 0 {
                    self.ram[INTRO_SWORD_19] = self.ram[INTRO_SWORD_19].wrapping_sub(1);
                    if (self.ram[INTRO_SWORD_19] as i8) < 0 {
                        self.ram[INTRO_SWORD_19] = 0;
                        self.ram[INTRO_SWORD_18] = 2;
                        self.ram[INTRO_SWORD_20] = self.ram[INTRO_SWORD_20].wrapping_add(2);
                        return;
                    }
                    self.ram[INTRO_SWORD_18] = TAB[self.ram[INTRO_SWORD_19] as usize];
                }
                self.set_oam_plain(
                    0x50,
                    0x44,
                    0x43,
                    SPARKLE_CHARS[self.ram[INTRO_SWORD_19] as usize],
                    0x25,
                    0,
                );
            }
            2 => {
                const SPARKLE_CHARS: [u8; 8] = [0x26, 0x20, 0x24, 0x34, 0x25, 0x20, 0x35, 0x20];
                let k = self.ram[INTRO_SWORD_19] as usize;
                if k >= 7 {
                    return;
                }
                let y_base = self.ram[INTRO_SWORD_21].min(0x4f);
                let y = y_base
                    .wrapping_add(read_le_u16(&self.ram, INTRO_SWORD_YPOS) as u8)
                    .wrapping_add(0x31);
                self.set_oam_plain(0x50, 0x42, y, SPARKLE_CHARS[k], 0x23, 0);
                self.set_oam_plain(0x51, 0x42, y.wrapping_add(8), SPARKLE_CHARS[k + 1], 0x23, 0);
                if self.ram[INTRO_SWORD_18] == 0 {
                    self.ram[INTRO_SWORD_21] = self.ram[INTRO_SWORD_21].wrapping_add(4);
                    if matches!(self.ram[INTRO_SWORD_21], 0x04 | 0x48 | 0x4c | 0x58) {
                        self.ram[INTRO_SWORD_19] = self.ram[INTRO_SWORD_19].wrapping_add(2);
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn intro_init(&mut self) {
        self.intro_setup_screen();
        self.ram[INIDISP_COPY] = 15;
        self.frame_control_view_mut().set_subsubmodule(0);
        self.intro_startup_delay = 0;
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        self.frame_control_view_mut().increment_submodule();
        self.ram[SOUND_EFFECT_2] = 10;
        self.intro_init_continue();
    }

    pub(super) fn intro_setup_screen(&mut self) {
        self.ram[NMI_DISABLE_CORE_UPDATES] = 0x80;
        self.enable_force_blank();
        self.ram[TM_COPY] = 16;
        self.ram[TS_COPY] = 0;
        self.intro_initialize_background_settings();
        self.ram[CGWSEL_COPY] = 0x20;
        self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 20;
        self.graphics_load_chr_half_slot();
        self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 0;
        self.LoadOWMusicIfNeeded();

        for i in 0..17 {
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (144 + i) * 2, 0x7fff);
            self.ppu.vram[0x27f0 + i] = 0;
        }

        write_le_u16(&mut self.ram, R16, 0x1ffe);
        write_le_u16(&mut self.ram, R18, 0x1bfe);
    }

    pub(super) fn intro_initialize_background_settings(&mut self) {
        self.ram[BGMODE_COPY] = 9;
        self.ram[MOSAIC_COPY] = 0;
        self.zelda_ppu_write(0x2107, 0x13);
        self.zelda_ppu_write(0x2108, 0x03);
        self.zelda_ppu_write(0x2109, 0x63);
        self.ram[CGADSUB_COPY] = 32;
        self.ram[COLDATA_COPY0] = 32;
        self.ram[COLDATA_COPY1] = 64;
        self.ram[COLDATA_COPY2] = 128;
    }

    pub(super) fn intro_init_continue(&mut self) {
        self.intro_display_logo();
        let t = self.frame_control_view().subsubmodule();
        self.frame_control_view_mut().increment_subsubmodule();
        match t {
            0..=7 => self.intro_clear1kb_blocks_of_wram(),
            8 => self.intro_load_text_pointers_and_palettes(),
            9 => self.load_item_gfx_into_wram_4bpp_buffer(),
            10 => self.load_follower_graphics(),
            _ => {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
                if self.ram[INIDISP_COPY] == 0 {
                    if self.rom_startup_timing() {
                        self.enable_force_blank();
                        let delay = configured_intro_memory_darken_frame_delay();
                        if delay != 0 {
                            self.intro_memory_darken_frame_delay = delay;
                            return;
                        }
                        self.intro_initialize_memory_darken_finish();
                        return;
                    }
                    self.intro_initialize_memory_darken();
                }
            }
        }
    }

    pub(super) fn intro_load_text_pointers_and_palettes(&mut self) {
        self.Text_GenerateMessagePointers();
        self.overworld_load_all_palettes();
    }

    pub(super) fn intro_initialize_memory_darken(&mut self) {
        self.enable_force_blank();
        self.intro_initialize_memory_darken_finish();
    }

    pub(super) fn intro_initialize_memory_darken_finish(&mut self) {
        self.erase_tile_maps_normal();
        self.ram[MAIN_TILE_THEME_INDEX] = 35;
        self.ram[SPRITE_GRAPHICS_INDEX] = 125;
        self.ram[AUX_TILE_THEME_INDEX] = 81;
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = 8;
        self.load_default_graphics();
        self.initialize_tilesets();
        self.decompress_animated_dungeon_tiles(0x5d);
        write_le_u16(&mut self.ram, BG_TILE_ANIMATION_COUNTDOWN, 2);
        self.ram[OVERWORLD_SCREEN_INDEX] = 0;
        self.ram[PALETTE_MAIN_INDOORS] = 0;
        self.ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = 0;
        write_le_u16(&mut self.ram, R16, 0);
        write_le_u16(&mut self.ram, R18, 0);
        write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, 2);
        write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 31);
        self.ram[MOSAIC_TARGET_LEVEL] = 0;
        self.frame_control_view_mut().increment_submodule();
    }

    pub(super) fn intro_initialize_triforce_poly_thread(&mut self) {
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = 8;
        self.load_common_sprites();
        self.intro_init_gfx_helper();
        self.ram[INTRO_SPRITE_IS_INITED] = 1;
        self.ram[INTRO_SPRITE_IS_INITED + 1] = 1;
        self.ram[INTRO_SPRITE_IS_INITED + 2] = 1;
        self.ram[INTRO_SPRITE_SUBTYPE] = 0;
        self.ram[INTRO_SPRITE_SUBTYPE + 1] = 0;
        self.ram[INTRO_SPRITE_SUBTYPE + 2] = 0;
        self.ram[INTRO_SPRITE_IS_INITED + 4] = 1;
        self.ram[INTRO_SPRITE_SUBTYPE + 4] = 2;
        self.ram[INIDISP_COPY] = 15;
        self.frame_control_view_mut().increment_submodule();
    }

    pub(super) fn intro_init_gfx_helper(&mut self) {
        self.polyhedral_initialize_thread();
        self.load_triforce_sprite_palette();
        self.ram[VIRQ_TRIGGER] = 0x90;
        self.ram[POLY_CONFIG1] = 0xff;
        self.ram[POLY_BASE_X] = 32;
        self.ram[POLY_BASE_Y] = 32;
        self.ram[POLY_VAR1] = 32;
        self.ram[POLY_A] = 0xa0;
        self.ram[POLY_B] = 0x60;
        self.ram[POLY_CONFIG_COLOR_MODE] = 1;
        self.ram[POLY_WHICH_MODEL] = 1;
        self.ram[IS_NMI_THREAD_ACTIVE] = 1;
        self.ram[INTRO_DID_RUN_STEP] = 1;
        if self.rom_startup_timing() {
            self.intro_poly_upload_delay = configured_intro_thread_start_delay();
            self.intro_sprite_animation_start_delay =
                configured_intro_sprite_animation_start_delay();
        }
        self.ram[INTRO_STEP_INDEX..INTRO_STEP_INDEX + 7 * 16].fill(0);
        if self.rom_startup_timing() {
            for _ in 0..configured_intro_poly_bootstrap_steps() {
                self.intro_run_step();
            }
        }
    }

    pub(super) fn polyhedral_initialize_thread(&mut self) {
        const POLY_THREAD_INIT: [u8; 13] = [9, 0, 0x1f, 0, 0, 0, 0, 0, 0, 0x30, 0x1d, 0xf8, 9];
        self.fill_ram(POLY_THREAD_RAM_START, POLY_THREAD_RAM_LEN, 0);
        write_le_u16(&mut self.ram, THREAD_OTHER_STACK, 0x1f31);
        self.copy_to_ram(POLY_THREAD_INIT_BYTES, &POLY_THREAD_INIT);
    }

    pub(super) fn intro_handle_all_triforce_animations(&mut self) {
        if self.rom_startup_timing() && self.intro_sprite_animation_start_delay != 0 {
            self.intro_sprite_animation_start_delay =
                self.intro_sprite_animation_start_delay.saturating_sub(1);
            self.intro_animate_triforce();
            return;
        }
        self.ram[INTRO_FRAME_CTR] = self.ram[INTRO_FRAME_CTR].wrapping_add(1);
        self.intro_animate_triforce();
        self.scene_animate_every_sprite();
    }

    pub(super) fn intro_animate_triforce(&mut self) {
        self.ram[IS_NMI_THREAD_ACTIVE] = 1;
        if self.rom_startup_timing() && self.intro_memory_darken_frame_delay == 0 {
            if self.intro_poly_upload_delay != 0 {
                self.intro_poly_upload_delay = self.intro_poly_upload_delay.saturating_sub(1);
                self.ram[INTRO_DID_RUN_STEP] = 1;
                return;
            }
            if self.bsnes_hold_intro_step_this_frame {
                self.ram[INTRO_DID_RUN_STEP] = 1;
                return;
            }
            self.intro_run_step();
            self.ram[INTRO_DID_RUN_STEP] = 1;
            return;
        }
        if self.ram[INTRO_DID_RUN_STEP] == 0 {
            self.intro_run_step();
            self.ram[INTRO_DID_RUN_STEP] = 1;
        }
    }

    pub(super) fn intro_run_step(&mut self) {
        match self.ram[INTRO_STEP_INDEX] {
            0 => {
                self.ram[INTRO_STEP_TIMER] = self.ram[INTRO_STEP_TIMER].wrapping_add(1);
                if self.ram[INTRO_STEP_TIMER] == 64 {
                    self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
                }
                self.ram[POLY_B] = self.ram[POLY_B].wrapping_add(5);
                self.ram[POLY_A] = self.ram[POLY_A].wrapping_add(3);
            }
            1 => {
                if self.ram[POLY_CONFIG1] < 2 {
                    self.ram[POLY_CONFIG1] = 0;
                    self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
                    self.ram[INTRO_STEP_TIMER] = 64;
                    return;
                }
                self.ram[POLY_CONFIG1] = self.ram[POLY_CONFIG1].wrapping_sub(2);
                self.ram[POLY_B] = self.ram[POLY_B].wrapping_add(5);
                self.ram[POLY_A] = self.ram[POLY_A].wrapping_add(3);
                if self.ram[POLY_CONFIG1] < 225 {
                    self.frame_control_view_mut().set_submodule(4);
                }
                if self.ram[POLY_CONFIG1] == 113 {
                    self.ram[MUSIC_CONTROL] = 1;
                }
            }
            2 => {
                self.ram[INTRO_STEP_TIMER] = self.ram[INTRO_STEP_TIMER].wrapping_sub(1);
                if self.ram[INTRO_STEP_TIMER] == 0 {
                    self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
                } else {
                    self.ram[POLY_B] = self.ram[POLY_B].wrapping_add(5);
                    self.ram[POLY_A] = self.ram[POLY_A].wrapping_add(3);
                }
            }
            3 => {
                if self.ram[POLY_B] >= 250 && self.ram[POLY_A] >= 252 {
                    self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
                    self.ram[INTRO_STEP_TIMER] = 32;
                } else {
                    self.ram[POLY_B] = self.ram[POLY_B].wrapping_add(5);
                    self.ram[POLY_A] = self.ram[POLY_A].wrapping_add(3);
                }
            }
            4 => {
                self.ram[POLY_B] = 0;
                self.ram[POLY_A] = 0;
                self.ram[INTRO_STEP_TIMER] = self.ram[INTRO_STEP_TIMER].wrapping_sub(1);
                if self.ram[INTRO_STEP_TIMER] == 0 {
                    self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
                    self.ram[INTRO_SPRITE_IS_INITED + 5] = 1;
                    self.ram[INTRO_SPRITE_SUBTYPE + 5] = 3;
                    self.ram[TM_COPY] = 0x10;
                    self.ram[TS_COPY] = 5;
                    self.ram[CGWSEL_COPY] = 2;
                    self.ram[CGADSUB_COPY] = 0x31;
                    self.frame_control_view_mut().set_subsubmodule(0);
                    self.ram[FLAG_UPDATE_CGRAM_IN_NMI] =
                        self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
                    self.ram[NMI_LOAD_BG_FROM_VRAM] = 3;
                    self.frame_control_view_mut().increment_submodule();
                }
            }
            _ => {}
        }
    }

    pub(super) fn scene_animate_every_sprite(&mut self) {
        write_le_u16(&mut self.ram, INTRO_SPRITE_ALLOC, 0x0800);
        for k in (0..8).rev() {
            self.intro_anim_one_obj(k);
        }
    }

    pub(super) fn intro_anim_one_obj(&mut self, k: usize) {
        match self.ram[INTRO_SPRITE_IS_INITED + k] {
            1 => match self.ram[INTRO_SPRITE_SUBTYPE + k] {
                0 => self.intro_sprite_type_a_0(k),
                1 => self.exit_0_cca90(k),
                2 => self.initialize_scene_sprite_copyright(k),
                3 => self.initialize_scene_sprite_sparkle(k),
                4 | 5 | 6 => self.initialize_scene_sprite_triforce_room_triangle(k),
                7 => self.initialize_scene_sprite_credits_triangle(k),
                _ => {}
            },
            2 => match self.ram[INTRO_SPRITE_SUBTYPE + k] {
                0 => self.intro_sprite_type_b_0(k),
                1 => self.exit_0_cca90(k),
                2 => self.animate_scene_sprite_copyright(k),
                3 => self.animate_scene_sprite_sparkle(k),
                4 | 5 | 6 => self.intro_sprite_type_b_456(k),
                7 => self.animate_scene_sprite_credits_triangle(k),
                _ => {}
            },
            _ => {}
        }
    }

    pub(super) fn intro_sprite_type_a_0(&mut self, k: usize) {
        const X: [i16; 3] = [-38, 95, 230];
        const Y: [i16; 3] = [200, -67, 200];
        const X_VEL: [i8; 3] = [1, 0, -1];
        const Y_VEL: [i8; 3] = [-1, 1, -1];
        self.write_intro_x(k, X[k]);
        self.write_intro_y(k, Y[k]);
        self.ram[INTRO_X_VEL + k] = X_VEL[k] as u8;
        self.ram[INTRO_Y_VEL + k] = Y_VEL[k] as u8;
        self.ram[INTRO_SPRITE_IS_INITED + k] = self.ram[INTRO_SPRITE_IS_INITED + k].wrapping_add(1);
    }

    pub(super) fn initialize_scene_sprite_copyright(&mut self, k: usize) {
        self.write_intro_x(k, 76);
        self.write_intro_y(k, 184);
        self.ram[INTRO_SPRITE_IS_INITED + k] = self.ram[INTRO_SPRITE_IS_INITED + k].wrapping_add(1);
    }

    pub(super) fn intro_sprite_type_b_0(&mut self, k: usize) {
        self.animate_scene_sprite_draw_triangle(k);
        self.animate_scene_sprite_move_triangle(k);
        if self.ram[INTRO_STEP_INDEX] != 5 {
            if self.ram[INTRO_FRAME_CTR] & 31 == 0 {
                const X_VEL: [i8; 3] = [1, 0, -1];
                const Y_VEL: [i8; 3] = [-1, 1, -1];
                self.ram[INTRO_X_VEL + k] = self.ram[INTRO_X_VEL + k].wrapping_add(X_VEL[k] as u8);
                self.ram[INTRO_Y_VEL + k] = self.ram[INTRO_Y_VEL + k].wrapping_add(Y_VEL[k] as u8);
            }
            const X_LIMIT: [u8; 3] = [75, 95, 117];
            const Y_LIMIT: [u8; 3] = [88, 48, 88];
            if self.ram[INTRO_X_LO + k] == X_LIMIT[k] {
                self.ram[INTRO_X_VEL + k] = 0;
            }
            if self.ram[INTRO_Y_LO + k] == Y_LIMIT[k] {
                self.ram[INTRO_Y_VEL + k] = 0;
            }
        } else {
            self.ram[INTRO_X_VEL + k] = 0;
            self.ram[INTRO_Y_VEL + k] = 0;
        }
    }

    pub(super) fn animate_scene_sprite_copyright(&mut self, k: usize) {
        const ENTS: [(i8, i8, u8, u8, u8); 13] = [
            (0, 0, 0x40, 0x0a, 0),
            (8, 0, 0x41, 0x0a, 0),
            (16, 0, 0x42, 0x0a, 0),
            (24, 0, 0x68, 0x0a, 0),
            (32, 0, 0x41, 0x0a, 0),
            (40, 0, 0x42, 0x0a, 0),
            (48, 0, 0x43, 0x0a, 0),
            (56, 0, 0x44, 0x0a, 0),
            (64, 0, 0x50, 0x0a, 0),
            (72, 0, 0x51, 0x0a, 0),
            (80, 0, 0x52, 0x0a, 0),
            (88, 0, 0x53, 0x0a, 0),
            (96, 0, 0x54, 0x0a, 0),
        ];
        self.animate_scene_sprite_add_objects_to_oam_buffer(k, &ENTS);
    }

    pub(super) fn initialize_scene_sprite_sparkle(&mut self, k: usize) {
        const X: [u8; 4] = [0xc2, 0x98, 0x6f, 0x34];
        const Y: [u8; 4] = [0x7c, 0x54, 0x7c, 0x57];
        let j = (self.ram[INTRO_FRAME_CTR] >> 5 & 3) as usize;
        self.ram[INTRO_X_LO + k] = X[j];
        self.ram[INTRO_X_HI + k] = 0;
        self.ram[INTRO_Y_LO + k] = Y[j];
        self.ram[INTRO_Y_HI + k] = 0;
        self.ram[INTRO_SPRITE_IS_INITED + k] = self.ram[INTRO_SPRITE_IS_INITED + k].wrapping_add(1);
    }

    pub(super) fn animate_scene_sprite_sparkle(&mut self, k: usize) {
        const ENTS: [(i8, i8, u8, u8, u8); 4] = [
            (0, 0, 0x80, 0x34, 0),
            (0, 0, 0xb7, 0x34, 0),
            (-4, -3, 0x64, 0x38, 2),
            (-4, -3, 0x62, 0x34, 2),
        ];
        const X: [u8; 4] = [0xc2, 0x98, 0x6f, 0x34];
        const Y: [u8; 4] = [0x7c, 0x54, 0x7c, 0x57];
        const STATE: [u8; 8] = [0, 1, 2, 3, 2, 1, 0xff, 0xff];

        let state = self.ram[INTRO_SPRITE_STATE + k];
        if state < 4 {
            self.animate_scene_sprite_add_objects_to_oam_buffer(
                k,
                &ENTS[state as usize..state as usize + 1],
            );
        }

        self.ram[INTRO_SPRITE_STATE + k] = STATE[(self.ram[INTRO_FRAME_CTR] >> 2 & 7) as usize];
        let j = (self.ram[INTRO_FRAME_CTR] >> 5 & 3) as usize;
        self.ram[INTRO_X_LO + k] = X[j];
        self.ram[INTRO_Y_LO + k] = Y[j];
    }

    pub(super) fn animate_scene_sprite_draw_triangle(&mut self, k: usize) {
        const LEFT: [(i8, i8, u8, u8, u8); 16] = [
            (0, 0, 0x80, 0x1b, 2),
            (16, 0, 0x82, 0x1b, 2),
            (32, 0, 0x84, 0x1b, 2),
            (48, 0, 0x86, 0x1b, 2),
            (0, 16, 0xa0, 0x1b, 2),
            (16, 16, 0xa2, 0x1b, 2),
            (32, 16, 0xa4, 0x1b, 2),
            (48, 16, 0xa6, 0x1b, 2),
            (0, 32, 0x88, 0x1b, 2),
            (16, 32, 0x8a, 0x1b, 2),
            (32, 32, 0x8c, 0x1b, 2),
            (48, 32, 0x8e, 0x1b, 2),
            (0, 48, 0xa8, 0x1b, 2),
            (16, 48, 0xaa, 0x1b, 2),
            (32, 48, 0xac, 0x1b, 2),
            (48, 48, 0xae, 0x1b, 2),
        ];
        const RIGHT: [(i8, i8, u8, u8, u8); 16] = [
            (48, 0, 0x80, 0x5b, 2),
            (32, 0, 0x82, 0x5b, 2),
            (16, 0, 0x84, 0x5b, 2),
            (0, 0, 0x86, 0x5b, 2),
            (48, 16, 0xa0, 0x5b, 2),
            (32, 16, 0xa2, 0x5b, 2),
            (16, 16, 0xa4, 0x5b, 2),
            (0, 16, 0xa6, 0x5b, 2),
            (48, 32, 0x88, 0x5b, 2),
            (32, 32, 0x8a, 0x5b, 2),
            (16, 32, 0x8c, 0x5b, 2),
            (0, 32, 0x8e, 0x5b, 2),
            (48, 48, 0xa8, 0x5b, 2),
            (32, 48, 0xaa, 0x5b, 2),
            (16, 48, 0xac, 0x5b, 2),
            (0, 48, 0xae, 0x5b, 2),
        ];
        self.animate_scene_sprite_add_objects_to_oam_buffer(k, if k == 2 { &RIGHT } else { &LEFT });
    }

    pub(super) fn animate_scene_sprite_move_triangle(&mut self, k: usize) {
        if self.ram[INTRO_X_VEL + k] != 0 {
            self.move_intro_coord(k, INTRO_X_SUBPIXEL, INTRO_X_LO, INTRO_X_HI, INTRO_X_VEL);
        }
        if self.ram[INTRO_Y_VEL + k] != 0 {
            self.move_intro_coord(k, INTRO_Y_SUBPIXEL, INTRO_Y_LO, INTRO_Y_HI, INTRO_Y_VEL);
        }
    }

    pub(super) fn intro_display_logo(&mut self) {
        const INTRO_LOGO_X: [u8; 4] = [0x60, 0x70, 0x80, 0x88];
        const INTRO_LOGO_TILE: [u8; 4] = [0x69, 0x6b, 0x6d, 0x6e];

        for i in 0..4 {
            self.set_oam_plain(i, INTRO_LOGO_X[i], 0x68, INTRO_LOGO_TILE[i], 0x32, 2);
        }
    }
}
