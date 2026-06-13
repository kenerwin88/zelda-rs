// Methods ported from zelda3/src/ending.c and included inside ZeldaState.

use super::sprite::{DrawMultipleData, PrepOamCoordsRet};
use super::*;
use crate::types::sign8;
use crate::zelda_rtl::misc::DUNG_ANIMATED_TILES;
use crate::zelda_rtl::sprite::SpriteSpawnInfo;

const POLYHEDRAL_PALETTE: [u16; 8] = [0, 0x14d, 0x1b0, 0x1f3, 0x256, 0x279, 0x2fd, 0x35f];
const FEATURE_DIM_ENDING_FLASHES: u32 = 65536;
const ENDING_SCENE_ENTRANCES: [u16; 16] = [
    0x1000, 2, 0x1002, 0x1012, 0x1004, 0x1006, 0x1010, 0x1014, 0x100a, 0x1016, 0x5d, 0x64, 0x100e,
    0x1008, 0x1018, 0x180,
];
const ENDING_SPRITE_PACKS: [u8; 17] = [
    0x28, 0x46, 0x27, 0x2e, 0x2b, 0x2b, 0xe, 0x2c, 0x1a, 0x29, 0x47, 0x28, 0x27, 0x28, 0x2a, 0x28,
    0x2d,
];
const ENDING_SPRITE_PALETTES: [u8; 17] = [
    1, 0x40, 1, 4, 1, 1, 1, 0x11, 1, 1, 0x47, 0x40, 1, 1, 1, 1, 1,
];
const ENDING_SCENE_SCROLL_TARGET_Y: [u16; 16] = [
    0x6f2, 0x210, 0x72c, 0xc00, 0x10c, 0xa9b, 0x10, 0x510, 0x89, 0xa8e, 0x222c, 0x2510, 0x826,
    0x5c, 0x20a, 0x30,
];
const ENDING_SCENE_SCROLL_TARGET_X: [u16; 16] = [
    0x77f, 0x480, 0x193, 0xaa, 0x878, 0x847, 0x4fd, 0xc57, 0x40f, 0x478, 0xa00, 0x200, 0x201,
    0xaa1, 0x26f, 0,
];
const ENDING_SCENE_SCROLL_Y_VELOCITIES: [i8; 16] =
    [-1, -1, 1, -1, 1, 1, 0, 1, 0, -1, -1, 0, 0, 0, 1, -1];
const ENDING_SCENE_SCROLL_X_VELOCITIES: [i8; 16] =
    [0, 0, -1, 0, 0, -1, 1, 0, -1, 0, 0, 0, 1, -1, 1, 0];
const BG2HOFS: usize = 0x210f;
const OVERWORLD_SCROLL_UP_COUNTER: usize = 0x624;
const OVERWORLD_SCROLL_DOWN_COUNTER: usize = 0x626;
const OVERWORLD_SCROLL_LEFT_COUNTER: usize = 0x628;
const OVERWORLD_SCROLL_RIGHT_COUNTER: usize = 0x62a;
const POLY_THREAD_RAM_START: usize = 0x1f00;
const POLY_THREAD_RAM_LEN: usize = 0x100;
const POLY_THREAD_INIT_BYTES: usize = 0x1f32;
type IntroSpriteEnt = (i8, i8, u8, u8, u8);

const ENDING_SPRITE_X_OFFSETS: [u16; 85] = [
    0x1e0, 0x200, 0x1ed, 0x203, 0x1da, 0x216, 0x1c8, 0x228, 0x1c0, 0x1e0, 0x208, 0x228, 0xf8, 0xf0,
    0x278, 0x298, 0x1e0, 0x200, 0x220, 0x288, 0x1e2, 0xe0, 0x150, 0xe8, 0x168, 0x128, 0x170, 0x170,
    0x335, 0x335, 0x300, 0xb8, 0xce, 0xac, 0xc4, 0x3b0, 0x390, 0x3d0, 0xf8, 0xc8, 0x80, 0xf8, 0xf8,
    0xf8, 0xf8, 0xf8, 0xe8, 0xf8, 0xd8, 0xf8, 0xc8, 0x108, 0x70, 0x70, 0x70, 0x68, 0x88, 0x70,
    0x40, 0x70, 0x4f, 0x61, 0x37, 0x79, 0xc8, 0x278, 0x258, 0x1d8, 0x1c8, 0x188, 0x270, 0x180,
    0x2e8, 0x270, 0x270, 0x2a0, 0x2a0, 0x2a4, 0x2fc, 0x76, 0x73, 0x76, 0x0, 0xd0, 0x80,
];
const ENDING_SPRITE_Y_OFFSETS: [u16; 85] = [
    0x158, 0x158, 0x138, 0x138, 0x140, 0x140, 0x150, 0x150, 0x120, 0x120, 0x120, 0x120, 0x60, 0x37,
    0xc2, 0xc2, 0x16b, 0x16c, 0x16b, 0xb8, 0x16b, 0x80, 0x60, 0x146, 0x146, 0x1c6, 0x70, 0x70,
    0x128, 0x128, 0x16f, 0xf5, 0xfc, 0x10d, 0x10d, 0x40, 0x40, 0x40, 0x150, 0x158, 0xf4, 0x120,
    0x120, 0x120, 0x120, 0x120, 0x108, 0x100, 0xd8, 0xd8, 0xf0, 0xf0, 0x3c, 0x3c, 0x3c, 0x90, 0x80,
    0x3c, 0x16c, 0x16c, 0x174, 0x174, 0x175, 0x175, 0x250, 0x2b0, 0x2b0, 0x2a0, 0x2b0, 0x2b0,
    0x2b8, 0xd8, 0x24b, 0x1b0, 0x1c8, 0x1c8, 0x1b0, 0x230, 0x230, 0x8b, 0x83, 0x85, 0x2c, 0xf8,
    0x100,
];
const ENDING_SCENE_SPRITE_RANGES: [usize; 17] = [
    0, 12, 14, 21, 28, 31, 35, 38, 40, 41, 52, 58, 64, 71, 72, 79, 85,
];

type DrawMultipleDataEnding = (i8, i8, u16, u8);

const END_SEQUENCE_DRAW_FRAMES0: [DrawMultipleDataEnding; 12] = [
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

const END_SEQUENCE_DRAW_FRAMES1: [DrawMultipleDataEnding; 6] = [
    (14, -7, 0x0d48, 2),
    (0, -6, 0x0944, 2),
    (0, 0, 0x094e, 2),
    (13, -14, 0x0d48, 2),
    (0, -8, 0x0944, 2),
    (0, 0, 0x0946, 2),
];

const END_SEQUENCE_DRAW_FRAMES2: [DrawMultipleDataEnding; 16] = [
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

const END_SEQUENCE_DRAW_FRAMES3: [DrawMultipleDataEnding; 12] = [
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

const END_SEQUENCE_DRAW_FRAMES4: [DrawMultipleDataEnding; 8] = [
    (10, 8, 0x8a32, 0),
    (10, 16, 0x8a22, 0),
    (0, -10, 0x0800, 2),
    (0, 0, 0x082c, 2),
    (10, -14, 0x0a22, 0),
    (10, -6, 0x0a32, 0),
    (0, -10, 0x082a, 2),
    (0, 0, 0x0828, 2),
];

const END_SEQUENCE_DRAW_FRAMES5: [DrawMultipleDataEnding; 10] = [
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

const END_SEQUENCE_DRAW_FRAMES6: [DrawMultipleDataEnding; 3] =
    [(-6, -2, 0x0706, 2), (0, -9, 0x090e, 2), (0, -1, 0x0908, 2)];

const END_SEQUENCE_DRAW_FRAMES7: [DrawMultipleDataEnding; 10] = [
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

const END_SEQUENCE_DRAW_FRAMES8: [DrawMultipleDataEnding; 1] = [(0, -19, 0x39af, 0)];

const END_SEQUENCE_DRAW_FRAMES9: [DrawMultipleDataEnding; 4] = [
    (-16, -24, 0x3704, 2),
    (-16, -16, 0x3764, 2),
    (-16, -24, 0x3762, 2),
    (-16, -16, 0x3764, 2),
];

const END_SEQUENCE_DRAW_FRAMES10: [DrawMultipleDataEnding; 4] = [
    (0, 0, 0x0c0c, 2),
    (0, 0, 0x0c0a, 2),
    (0, 0, 0x0cc5, 2),
    (0, 0, 0x0ce1, 2),
];

const END_SEQUENCE_DRAW_FRAMES11: [DrawMultipleDataEnding; 6] = [
    (1, 4, 0x002a, 0),
    (1, 12, 0x003a, 0),
    (4, 0, 0x0026, 2),
    (0, 9, 0x0024, 2),
    (8, 9, 0x4024, 2),
    (4, 20, 0x016c, 2),
];

const END_SEQUENCE_DRAW_FRAMES12: [DrawMultipleDataEnding; 21] = [
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

const END_SEQUENCE_DRAW_FRAMES13: [DrawMultipleDataEnding; 16] = [
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

const END_SEQUENCE_DRAW_FRAMES14: [DrawMultipleDataEnding; 6] = [
    (0, 0, 0, 0),
    (0, 0, 0x34c7, 0),
    (0, 0, 0x3480, 0),
    (0, 0, 0x34b6, 0),
    (0, 0, 0x34b7, 0),
    (0, 0, 0x34a6, 0),
];

const END_SEQUENCE_DRAW_FRAMES15: [DrawMultipleDataEnding; 6] = [
    (-3, 17, 0x002b, 0),
    (-3, 25, 0x003b, 0),
    (0, 0, 0x000e, 2),
    (16, 0, 0x400e, 2),
    (0, 16, 0x002e, 2),
    (16, 16, 0x402e, 2),
];

const END_SEQUENCE_DRAW_FRAMES16: [DrawMultipleDataEnding; 3] =
    [(8, 5, 0x0a04, 2), (0, 16, 0x0806, 2), (16, 16, 0x4806, 2)];

const END_SEQUENCE_DRAW_FRAMES17: [DrawMultipleDataEnding; 2] =
    [(0, 0, 0x0000, 2), (0, 11, 0x0002, 2)];

const END_SEQUENCE_DRAW_FRAMES18: [DrawMultipleDataEnding; 2] =
    [(0, 0, 0x000e, 2), (0, 64, 0x006c, 2)];

const END_SEQUENCE_DRAW_FRAMES19: [DrawMultipleDataEnding; 8] = [
    (0, 0, 0x0882, 2),
    (0, 7, 0x0a4e, 2),
    (0, 0, 0x4880, 2),
    (0, 7, 0x0a4e, 2),
    (0, 0, 0x0882, 2),
    (0, 7, 0x0a4e, 2),
    (0, 0, 0x0880, 2),
    (0, 7, 0x0a4e, 2),
];

const END_SEQUENCE_DRAW_FRAMES20: [DrawMultipleDataEnding; 6] = [
    (-4, 1, 0x0c68, 0),
    (0, -8, 0x0c40, 2),
    (0, 1, 0x0c42, 2),
    (-4, 1, 0x0c78, 0),
    (0, -8, 0x0c40, 2),
    (0, 1, 0x0c42, 2),
];

const END_SEQUENCE_DRAW_FRAMES21: [DrawMultipleDataEnding; 6] = [
    (8, 5, 0x0679, 0),
    (0, -10, 0x088e, 2),
    (0, 0, 0x066e, 2),
    (0, -10, 0x088e, 2),
    (0, -10, 0x088e, 2),
    (0, 0, 0x066e, 2),
];

const END_SEQUENCE_DRAW_FRAMES22: [DrawMultipleDataEnding; 6] = [
    (11, -3, 0x0869, 0),
    (0, -12, 0x0804, 2),
    (0, 0, 0x0860, 2),
    (10, -3, 0x0867, 0),
    (0, -12, 0x0804, 2),
    (0, 0, 0x0860, 2),
];

const END_SEQUENCE_DRAW_FRAMES23: [DrawMultipleDataEnding; 6] = [
    (-2, 1, 0x0868, 0),
    (0, -8, 0x08c0, 2),
    (0, 0, 0x08c2, 2),
    (-3, 1, 0x0878, 0),
    (0, -8, 0x08c0, 2),
    (0, 0, 0x08c2, 2),
];

const END_SEQUENCE_DRAW_FRAMES24: [DrawMultipleDataEnding; 4] = [
    (0, -10, 0x084c, 2),
    (0, 0, 0x0a6c, 2),
    (0, -9, 0x084c, 2),
    (0, 0, 0x0aa8, 2),
];

const END_SEQUENCE_DRAW_FRAMES25: [DrawMultipleDataEnding; 4] = [
    (0, -7, 0x084a, 2),
    (0, 0, 0x0c6a, 2),
    (0, -7, 0x084a, 2),
    (0, 0, 0x0ca6, 2),
];

const END_SEQUENCE_DRAW_FRAMES26: [DrawMultipleDataEnding; 12] = [
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

const END_SEQUENCE_DRAW_FRAMES27: [DrawMultipleDataEnding; 6] = [
    (0, -4, 0x30aa, 2),
    (0, -4, 0x30aa, 2),
    (-4, -8, 0x3090, 0),
    (12, -8, 0x7090, 0),
    (-6, -10, 0x3091, 0),
    (14, -10, 0x7091, 0),
];

const END_SEQUENCE_DRAW_FRAMES28: [DrawMultipleDataEnding; 8] = [
    (0, 0, 0x0722, 2),
    (0, -8, 0x09c2, 2),
    (0, 0, 0x4722, 2),
    (0, -8, 0x09c2, 2),
    (0, -9, 0x09c4, 2),
    (0, 0, 0x0722, 2),
    (0, -9, 0x0924, 2),
    (0, 0, 0x0722, 2),
];

const END_SEQUENCE_DRAW_FRAMES29: [DrawMultipleDataEnding; 3] = [
    (-16, -12, 0x3f08, 2),
    (0, -12, 0x3f20, 2),
    (16, -12, 0x3f20, 2),
];

const END_SEQUENCE_DRAW_FRAMES30: [DrawMultipleDataEnding; 1] = [(0, 0, 0x0086, 2)];

const END_SEQUENCE_DRAW_FRAMES31: [DrawMultipleDataEnding; 1] = [(0, 0, 0x8060, 2)];

const END_SEQUENCE_DRAW_FRAME_SETS: [&[DrawMultipleDataEnding]; 32] = [
    &END_SEQUENCE_DRAW_FRAMES0,
    &END_SEQUENCE_DRAW_FRAMES1,
    &END_SEQUENCE_DRAW_FRAMES2,
    &END_SEQUENCE_DRAW_FRAMES3,
    &END_SEQUENCE_DRAW_FRAMES4,
    &END_SEQUENCE_DRAW_FRAMES5,
    &END_SEQUENCE_DRAW_FRAMES6,
    &END_SEQUENCE_DRAW_FRAMES7,
    &END_SEQUENCE_DRAW_FRAMES8,
    &END_SEQUENCE_DRAW_FRAMES9,
    &END_SEQUENCE_DRAW_FRAMES10,
    &END_SEQUENCE_DRAW_FRAMES11,
    &END_SEQUENCE_DRAW_FRAMES12,
    &END_SEQUENCE_DRAW_FRAMES13,
    &END_SEQUENCE_DRAW_FRAMES14,
    &END_SEQUENCE_DRAW_FRAMES15,
    &END_SEQUENCE_DRAW_FRAMES16,
    &END_SEQUENCE_DRAW_FRAMES17,
    &END_SEQUENCE_DRAW_FRAMES18,
    &END_SEQUENCE_DRAW_FRAMES19,
    &END_SEQUENCE_DRAW_FRAMES20,
    &END_SEQUENCE_DRAW_FRAMES21,
    &END_SEQUENCE_DRAW_FRAMES22,
    &END_SEQUENCE_DRAW_FRAMES23,
    &END_SEQUENCE_DRAW_FRAMES24,
    &END_SEQUENCE_DRAW_FRAMES25,
    &END_SEQUENCE_DRAW_FRAMES26,
    &END_SEQUENCE_DRAW_FRAMES27,
    &END_SEQUENCE_DRAW_FRAMES28,
    &END_SEQUENCE_DRAW_FRAMES29,
    &END_SEQUENCE_DRAW_FRAMES30,
    &END_SEQUENCE_DRAW_FRAMES31,
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
        self.palette_buffer_view_mut().set_sp5l(pal2);
        self.palette_buffer_view_mut().set_sp6l(pal3);
    }

    fn CallForDuckIndoors(&mut self) {
        self.call_for_duck_indoors();
    }

    fn Sprite_SpawnBatCrashCutscene(&mut self) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(0, 0x37, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_slot_view_mut(j).set_y_velocity(0);
            self.sprite_slot_view_mut(j).set_b(0);
            self.sprite_slot_view_mut(j).set_direction(0);
            self.sprite_slot_view_mut(j).set_floor(0);
            self.sprite_slot_view_mut(j).set_subtype2(1);
            self.sprite_slot_view_mut(j).set_flags2(1);
            self.sprite_slot_view_mut(j).set_flags3(1);
            self.sprite_slot_view_mut(j).set_oam_flags(1);
            self.sprite_slot_view_mut(j).set_x_low(204);
            self.sprite_slot_view_mut(j).set_x_high(7);
            self.sprite_slot_view_mut(j).set_y_low(50);
            self.sprite_slot_view_mut(j).set_y_high(6);
            self.sprite_slot_view_mut(j).set_deflection_bits(128);
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
        let y = if y.wrapping_add(0x10) < 0x100 {
            y as u8
        } else {
            0xf0
        };
        self.oam_state_view_mut()
            .write_entry(oam, x as u8, y, charnum, flags);
        let value = big | ((x >> 8) as u8 & 1);
        self.oam_state_view_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
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
        self.palette_filter_view_mut()
            .set_color_window_selection(0x82);
        let k = (self.frame_state().submodule >> 1) as usize;
        self.set_dungeon_room(ENDING_SCENE_ENTRANCES[k]);
        if k != 6 && k != 15 {
            self.LoadOverworldFromDungeon();
        } else {
            self.Overworld_EnterSpecialArea();
        }
        self.system_signals_view_mut().set_music_control(0);
        self.system_signals_view_mut().set_ambient_sound_effect(0);
        let t = self.world_location_state().overworld_screen_index() & !0x40;
        self.DecompressAnimatedOverworldTiles(if t == 3 || t == 5 || t == 7 {
            0x58
        } else {
            0x5a
        });
        let k = (self.frame_state().submodule >> 1) as usize;
        self.sprite_system_view_mut()
            .set_graphics_index(ENDING_SPRITE_PACKS[k]);
        let sprpal = ENDING_SPRITE_PALETTES[k];
        self.palette_buffer_view_mut().set_hud_palette(1);
        self.initialize_tilesets();
        self.OverworldLoadScreensPaletteSet();
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(self.world_location_state().overworld_screen_index()),
            sprpal,
        );
        self.palette_load_hud();
        if self.frame_state().submodule == 0 {
            self.TransferFontToVRAM();
        }
        self.overworld_load_palettes_inner();
        self.Overworld_SetFixedColAndScroll();
        if self.world_location_state().overworld_screen_index() >= 128 {
            self.Palette_SetOwBgColor();
        }
        self.set_bg_mode(9);
        self.increment_subsubmodule();
    }

    pub(super) fn credits_load_scene_overworld_overlay(&mut self) {
        self.Overworld_LoadOverlays2();
        self.system_signals_view_mut().set_music_control(0);
        self.system_signals_view_mut().set_ambient_sound_effect(0);
        self.decrement_submodule();
        self.increment_subsubmodule();
    }

    pub(super) fn credits_load_scene_overworld_load_map(&mut self) {
        self.Overworld_LoadAndBuildScreen();
        self.credits_prep_and_load_sprites();
        self.ending_scratch_view_mut().clear_primary_word();
        self.set_subsubmodule(0);
    }

    pub(super) fn credits_operate_scrolling_and_tile_map(&mut self) {
        self.credits_handle_camera_scroll_control();
        if self.has_screen_transition_direction_bits() {
            self.OverworldHandleMapScroll();
        }
    }

    pub(super) fn credits_load_cool_background(&mut self) {
        self.sprite_system_view_mut().set_main_tile_theme(33);
        self.sprite_system_view_mut().set_aux_tile_theme(59);
        self.sprite_system_view_mut().set_graphics_index(45);
        self.initialize_tilesets();
        self.set_overworld_screen(0x5b);
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(self.world_location_state().overworld_screen_index()),
            0x13,
        );
        self.palette_buffer_view_mut()
            .set_overworld_palette_aux2_hi(3);
        self.palette_load_ow_bg2();
        self.overworld_copy_palettes_to_cache();
        self.Overworld_LoadOverlays2();
        self.world_scroll_mut().set_bg1_y_low(0);
        self.world_scroll_mut().set_bg1_x_low(0);
        self.decrement_submodule();
    }

    pub(super) fn credits_load_scene_dungeon(&mut self) {
        self.enable_force_blank();
        self.erase_tile_maps_normal();
        let i = (self.frame_state().submodule >> 1) as usize;
        self.world_region_mut()
            .set_which_entrance(ENDING_SCENE_ENTRANCES[i]);
        self.Dungeon_LoadEntrance();
        self.dungeon_state_view_mut().clear_lit_torches();
        self.dungeon_state_view_mut()
            .clear_dungeon_dark_with_lantern();
        self.Dungeon_LoadAndDrawRoom();
        self.decompress_animated_dungeon_tiles(
            DUNG_ANIMATED_TILES[self.sprite_system_view().main_tile_theme() as usize] as usize,
        );
        self.sprite_system_view_mut()
            .set_graphics_index(ENDING_SPRITE_PACKS[i]);
        self.apply_dung_pal_info(ENDING_SPRITE_PALETTES[i] & 0x3f);
        self.sprite_system_view_mut()
            .set_misc_sprites_graphics_index(10);
        self.initialize_tilesets();
        self.palette_buffer_view_mut().set_sp6r_indoors(10);
        self.Dungeon_LoadPalettes();
        self.set_bg_mode(9);
        self.ending_scratch_view_mut().clear_primary_word();
        self.set_screen_brightness(0);
        self.increment_submodule();
        self.credits_prep_and_load_sprites();
    }

    pub(super) fn module18_ganon_emerges(&mut self) {
        let hofs2 = self.world_scroll().bg2_x();
        let vofs2 = self.world_scroll().bg2_y();
        let hofs1 = self.world_scroll().bg1_x();
        let vofs1 = self.world_scroll().bg1_y();
        let bg1_x_offset = self.world_scroll().bg1_x_offset();
        let bg1_y_offset = self.world_scroll().bg1_y_offset();
        self.ppu_scroll_copy_view_mut().set_bg1_bg2_live_and_copy(
            hofs2.wrapping_add(bg1_x_offset),
            vofs2.wrapping_add(bg1_y_offset),
            hofs1.wrapping_add(bg1_x_offset),
            vofs1.wrapping_add(bg1_y_offset),
        );
        self.sprite_main();
        self.world_scroll_mut().set_bg1_y(vofs1);
        self.world_scroll_mut().set_bg1_x(hofs1);
        self.world_scroll_mut().set_bg2_y(vofs2);
        self.world_scroll_mut().set_bg2_x(hofs2);
        match self.overworld_map_state() {
            0 => {
                self.dungeon_handle_layer_effect();
                self.CallForDuckIndoors();
                self.SaveDungeonKeys();
                self.increment_overworld_map_state();
                self.player_state_view_mut().increment_immobilized_flag();
            }
            1 => {
                self.dungeon_handle_layer_effect();
                if self.frame_state().submodule == 10 {
                    self.set_overworld_screen(91);
                    self.set_indoor_flag(0);
                    self.set_main_module(24);
                    self.set_submodule(0);
                    self.set_overworld_map_state(2);
                }
            }
            2 => {
                self.dungeon_handle_layer_effect();
                self.decrement_screen_brightness();
                if self.display_state().screen_brightness == 0 {
                    self.enable_force_blank();
                    self.increment_overworld_map_state();
                    self.hud_rebuild_indoor();
                    self.player_state_view_mut().clear_movement_velocity();
                }
            }
            3 => {
                self.set_birdtravel_status(8);
                self.clear_bird_travel_stop_status(1);
                self.FluteMenu_LoadSelectedScreen();
                self.LoadOWMusicIfNeeded();
                self.system_signals_view_mut().set_music_control(9);
            }
            4 => {
                self.Overworld_LoadOverlayAndMap();
                self.set_subsubmodule(0);
            }
            5 => {
                self.increment_screen_brightness();
                if self.display_state().screen_brightness == 15 {
                    self.dungeon_savegame_state_mut()
                        .clear_savegame_state_bits();
                    self.clear_modal_pause_flag();
                    self.Sprite_SpawnBatCrashCutscene();
                    self.player_state_view_mut().set_facing(2);
                    self.set_saved_module_for_menu(9);
                    self.set_indoor_flag(0);
                    self.increment_overworld_map_state();
                    self.set_subsubmodule(128);
                    self.save_progress_view_mut().set_palace_index_x2(255);
                }
            }
            6 => {}
            7 => {
                self.decrement_subsubmodule();
                if self.frame_state().subsubmodule == 0 {
                    self.increment_overworld_map_state();
                }
            }
            8 => self.BirdTravel_Finish_Doit(),
            _ => {}
        }
        self.link_oam_main();
    }

    pub(super) fn module19_triforce_room(&mut self) {
        match self.frame_state().subsubmodule {
            0 => {
                self.link_reset_properties_a();
                self.player_state_view_mut()
                    .set_last_direction_moved_towards(0);
                self.system_signals_view_mut().set_music_control(0xf1);
                self.reset_transition_props_and_advance_reset_interface();
            }
            1 => {
                self.conditional_mosaic_control();
                self.apply_palette_filter_bounce();
            }
            2 => {
                self.enable_force_blank();
                self.load_credits_songs();
                self.set_dungeon_room(0x189);
                self.erase_tile_maps_normal();
                self.Palette_RevertTranslucencySwap();
                self.Overworld_EnterSpecialArea();
                self.Overworld_LoadOverlays2();
                self.increment_subsubmodule();
                self.set_main_module(25);
                self.set_submodule(0);
            }
            3 => {
                self.sprite_system_view_mut().set_main_tile_theme(36);
                self.sprite_system_view_mut().set_graphics_index(125);
                self.sprite_system_view_mut().set_aux_tile_theme(81);
                self.initialize_tilesets();
                self.Overworld_LoadAreaPalettesEx(4);
                self.Overworld_LoadPalettes(14, 0);
                self.SpecialOverworld_CopyPalettesToCache();
                self.increment_subsubmodule();
            }
            4 => {
                let bak0 = self.frame_state().subsubmodule;
                self.Module08_02_LoadAndAdvance();
                self.set_subsubmodule(bak0.wrapping_add(1));
                self.set_screen_brightness(15);
                self.palette_filter_view_mut().set_countdown(31);
                self.clear_mosaic_target_level();
                self.ppu_scroll_copy_view_mut().set_bg1_h_high(1);
                self.palette_filter_view_mut().set_color_window_selection(2);
                self.palette_filter_view_mut().set_color_math_control(50);
                self.set_mosaic_level(240);
                {
                    let mut player = self.player_state_view_mut();
                    player.set_y_low(236);
                    player.set_x_low(120);
                    player.set_lower_level_state(2);
                }
                self.system_signals_view_mut().set_music_control(32);
                self.set_main_module(25);
                self.set_submodule(0);
            }
            5 => {
                self.player_state_view_mut()
                    .set_direction_and_last_direction(8);
                self.player_state_view_mut().set_facing(0);
                if self.player_state_view().y_low() < 192 {
                    self.player_state_view_mut()
                        .set_direction_and_last_direction(0);
                    self.player_state_view_mut().clear_animation_step();
                    self.increment_subsubmodule();
                }
            }
            6 => {
                if self.palette_filter_view().countdown() & 1 == 0
                    && self.display_state().mosaic_level != 0
                {
                    self.decrement_mosaic_level_by(0x10);
                }
                self.set_bg_mode(9);
                self.set_mosaic_copy_from_level_or(7);
                self.apply_palette_filter_bounce();
            }
            7 => {
                self.triforce_room_prep_gfx_slot_for_poly();
                self.dialogue_message_index_view_mut().set_value(0x173);
                self.main_show_text_message();
                self.RenderText();
                self.ending_scratch_view_mut().set_primary_low(0x80);
                self.set_main_module(25);
                self.increment_subsubmodule();
            }
            8 | 10 => {
                self.advance_polyhedral();
                if self.frame_state().subsubmodule == 11 {
                    self.system_signals_view_mut().set_music_control(33);
                    self.set_main_module(25);
                    self.player_state_view_mut()
                        .set_direction_and_last_direction(0);
                    self.increment_submodule();
                }
            }
            9 => {
                self.advance_polyhedral();
                self.RenderText();
                if self.frame_state().submodule == 0 {
                    self.set_overworld_map_state(0);
                    self.set_main_module(25);
                    self.increment_subsubmodule();
                }
            }
            11 => {
                self.advance_polyhedral();
                self.triforce_room_link_approach_triforce();
                if self.frame_state().subsubmodule == 12 {
                    self.player_state_view_mut()
                        .set_direction_and_last_direction(0);
                }
            }
            12 => {
                self.advance_polyhedral();
                if self.ending_scratch_view_mut().decrement_primary_low() == 0 {
                    self.Palette_AnimGetMasterSword2();
                    self.increment_submodule();
                }
            }
            13 => {
                self.advance_polyhedral();
                self.PaletteFilter_BlindingWhiteTriforce();
                if self.palette_filter_view().darkening_or_lightening_screen() == 255 {
                    self.increment_subsubmodule();
                }
            }
            14 => {
                self.decrement_screen_brightness();
                if self.display_state().screen_brightness == 0 {
                    self.set_main_module(26);
                    self.set_submodule(0);
                    self.set_subsubmodule(0);
                    self.set_irq_control_flag(0xff);
                    self.deactivate_nmi_thread();
                    self.clear_pending_polyhedral_update();
                    self.save_progress_view_mut().set_dark_world_state(0);
                }
            }
            _ => {}
        }
        self.ppu_scroll_copy_view_mut().copy_live_to_ppu_copy();
        if self.frame_state().subsubmodule < 7 || self.frame_state().subsubmodule >= 11 {
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
        const INTRO_CLEAR_BLOCK_BASE: usize = 0x2000;
        const INTRO_CLEAR_BLOCK_STRIDE: usize = 0x2000;

        let mut i = self.ending_scratch_view().primary_word();
        let r18 = self.ending_scratch_view().secondary_word();
        loop {
            for j in 0..15 {
                self.native_ram_bridge_view_mut().set_word_at(
                    INTRO_CLEAR_BLOCK_BASE + i as usize + j * INTRO_CLEAR_BLOCK_STRIDE,
                    0,
                );
            }
            i = i.wrapping_sub(2);
            if i == r18 {
                break;
            }
        }
        self.ending_scratch_view_mut().set_primary_word(i);
        self.ending_scratch_view_mut()
            .set_secondary_word(i.wrapping_sub(0x400));
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
        let x = self.intro_actor_view(k).x();
        let y = self.intro_actor_view(k).y();
        let mut oam = self.allocate_intro_sprite_oam_entries(entries.len());
        for &(x_off, y_off, charnum, flags, ext) in entries {
            let obj_x = x.wrapping_add((x_off as i16).wrapping_add(x_delta) as u16);
            let obj_y = y.wrapping_add((y_off as i16).wrapping_add(y_delta) as u16);
            self.set_oam_helper0_at(oam, obj_x, obj_y, charnum, flags, ext);
            oam += 4;
        }
    }

    pub(super) fn AnimateSceneSprite_MoveTriangle(&mut self, k: usize) {
        self.animate_scene_sprite_move_triangle(k);
    }

    pub(super) fn triforce_room_prep_gfx_slot_for_poly(&mut self) {
        self.sprite_system_view_mut()
            .set_misc_sprites_graphics_index(8);
        self.load_common_sprites();
        self.intro_init_gfx_helper();
        self.intro_actor_view_mut(0).set_init_phase(1);
        self.intro_actor_view_mut(1).set_init_phase(1);
        self.intro_actor_view_mut(2).set_init_phase(1);
        self.intro_actor_view_mut(0).set_subtype(4);
        self.intro_actor_view_mut(1).set_subtype(5);
        self.intro_actor_view_mut(2).set_subtype(6);
        self.set_screen_brightness(15);
        self.increment_submodule();
    }

    pub(super) fn credits_initialize_polyhedral(&mut self) {
        self.sprite_system_view_mut()
            .set_misc_sprites_graphics_index(8);
        self.load_common_sprites();
        self.intro_init_gfx_helper();
        self.poly_runtime_mut().clear_config1();
        for k in 0..3 {
            self.intro_actor_view_mut(k).set_init_phase(1);
            self.intro_actor_view_mut(k).set_subtype(7);
        }
        self.set_screen_brightness(15);
        self.increment_submodule();
    }

    pub(super) fn advance_polyhedral(&mut self) {
        self.triforce_room_handle_poly();
        self.scene_animate_every_sprite();
    }

    pub(super) fn triforce_room_handle_poly(&mut self) {
        self.activate_nmi_thread();
        self.pause_intro_triangle_motion();
        if self.attract_scene().intro_did_run_step() != 0 {
            return;
        }
        match self.attract_scene().intro_step_index() {
            0 => {
                self.poly_runtime_mut().subtract_config1(2);
                if self.poly_runtime().config1() < 2 {
                    self.poly_runtime_mut().clear_config1();
                    self.attract_scene_mut().increment_intro_step_index();
                    self.increment_subsubmodule();
                }
                if self.frame_state().subsubmodule >= 10 {
                    self.attract_scene_mut().increment_intro_step_index();
                    self.intro_actor_view_mut(1).set_y_velocity(5);
                }
                self.poly_runtime_mut().add_angle_b(2);
                self.poly_runtime_mut().add_angle_a(1);
            }
            1 => {
                if self.frame_state().subsubmodule >= 10 {
                    self.attract_scene_mut().increment_intro_step_index();
                    self.intro_actor_view_mut(1).set_y_velocity(5);
                }
                self.poly_runtime_mut().add_angle_b(2);
                self.poly_runtime_mut().add_angle_a(1);
            }
            2 => {
                self.start_triforce_countdown(0x1c0);
                if self.poly_runtime().config1() < 128 {
                    self.poly_runtime_mut().increment_config1();
                } else if (self.poly_runtime().angle_b().wrapping_sub(10) & 0x7f) >= 92
                    && self.poly_runtime().angle_a().wrapping_sub(11) >= 220
                {
                    self.poly_runtime_mut().clear_angles();
                    self.increment_subsubmodule();
                    self.attract_scene_mut().increment_intro_step_index();
                    self.system_signals_view_mut().set_sound_effect_1(44);
                    self.palette_buffer_view_mut().set_main_color(0xd7, 0x7fff);
                    self.system_signals_view_mut().increment_cgram_update_flag();
                    self.attract_scene_mut().set_intro_step_timer(6);
                    break_triforce_handle_poly(self);
                    return;
                }
                self.poly_runtime_mut().add_angle_b(5);
                self.poly_runtime_mut().add_angle_a(3);
            }
            3 => {
                self.attract_scene_mut().decrement_intro_step_timer();
                if self.attract_scene().intro_step_timer() == 0 {
                    self.palette_buffer_view_mut()
                        .set_main_color(0xd7, POLYHEDRAL_PALETTE[7]);
                    self.system_signals_view_mut().increment_cgram_update_flag();
                    self.attract_scene_mut().increment_intro_step_index();
                }
            }
            _ => {}
        }
        self.attract_scene_mut().mark_intro_did_run_step();
        self.resume_intro_triangle_motion();
        self.attract_scene_mut().increment_intro_frame_counter();
    }

    pub(super) fn credits_animate_the_triangles(&mut self) {
        self.attract_scene_mut().increment_intro_frame_counter();
        self.activate_nmi_thread();
        if self.attract_scene().intro_did_run_step() == 0 {
            self.poly_runtime_mut().add_angle_b(3);
            self.poly_runtime_mut().add_angle_a(1);
            self.attract_scene_mut().mark_intro_did_run_step();
        }
        self.scene_animate_every_sprite();
    }

    pub(super) fn initialize_scene_sprite_triforce_room_triangle(&mut self, k: usize) {
        const X: [i16; 3] = [0x4e, 0x5f, 0x72];
        const Y: [i16; 3] = [0x9c, 0x9c, 0x9c];
        const LOCAL_X_VELOCITIES: [i8; 3] = [-2, 0, 2];
        const LOCAL_Y_VELOCITIES: [i8; 3] = [4, -4, 4];
        self.write_intro_x(k, X[k]);
        self.write_intro_y(k, Y[k]);
        self.intro_actor_view_mut(k)
            .set_x_velocity(LOCAL_X_VELOCITIES[k] as u8);
        self.intro_actor_view_mut(k)
            .set_y_velocity(LOCAL_Y_VELOCITIES[k] as u8);
        self.intro_actor_view_mut(k).increment_init_phase();
    }

    pub(super) fn intro_sprite_type_b_456(&mut self, k: usize) {
        self.intro_copy_sprite_type4_to_oam(k);
        if self.intro_scene_state().triangle_motion_is_paused() {
            return;
        }
        self.animate_scene_sprite_move_triangle(k);
        match self.attract_scene().intro_step_index() {
            0 => {
                const XACC: [i8; 3] = [-1, 0, 1];
                const YACC: [i8; 3] = [-1, -1, -1];
                if self.attract_scene().intro_frame_counter() & 7 == 0 {
                    self.intro_actor_view_mut(k).add_x_velocity(XACC[k] as u8);
                }
                if self.attract_scene().intro_frame_counter() & 3 == 0 {
                    self.intro_actor_view_mut(k).add_y_velocity(YACC[k] as u8);
                }
            }
            1 => {
                self.intro_actor_view_mut(k).set_x_velocity(0);
                self.intro_actor_view_mut(k).set_y_velocity(0);
            }
            2 => {
                const XFINAL: [u8; 3] = [0x59, 0x5f, 0x67];
                const YFINAL: [u8; 3] = [0x74, 0x68, 0x74];
                if self.attract_scene().intro_frame_counter() & 3 == 0 {
                    self.animate_triforce_room_triangle_handle_contracting(k);
                }
                if XFINAL[k] == self.intro_actor_view(k).x_low() {
                    self.intro_actor_view_mut(k).set_x_velocity(0);
                }
                if YFINAL[k] == self.intro_actor_view(k).y_low() {
                    self.intro_actor_view_mut(k).set_y_velocity(0);
                }
            }
            3 | 4 => {
                const YFINAL2: [u8; 3] = [0x72, 0x66, 0x72];
                let ctr = self.intro_scene_state().triforce_countdown;
                if ctr == 0 {
                    self.intro_actor_view_mut(k).set_y_low(YFINAL2[k]);
                } else {
                    self.decrement_triforce_countdown();
                }
            }
            _ => {}
        }
    }

    pub(super) fn animate_triforce_room_triangle_handle_contracting(&mut self, k: usize) {
        const XFINAL: [u8; 3] = [0x59, 0x5f, 0x67];
        const YFINAL: [u8; 3] = [0x74, 0x68, 0x74];
        let xv = self.intro_actor_view(k).x_velocity().wrapping_add(
            if self.intro_actor_view(k).x_low() <= XFINAL[k] {
                1
            } else {
                0xff
            },
        );
        self.intro_actor_view_mut(k).set_x_velocity(match xv {
            0x11 => 0x10,
            0xef => 0xf0,
            _ => xv,
        });
        let yv = self.intro_actor_view(k).y_velocity().wrapping_add(
            if self.intro_actor_view(k).y_low() <= YFINAL[k] {
                1
            } else {
                0xff
            },
        );
        self.intro_actor_view_mut(k).set_y_velocity(match yv {
            0x11 => 0x10,
            0xef => 0xf0,
            _ => yv,
        });
    }

    pub(super) fn initialize_scene_sprite_credits_triangle(&mut self, k: usize) {
        const X: [u8; 3] = [0x29, 0x5f, 0x97];
        const Y: [u8; 3] = [0x70, 0x20, 0x70];
        self.intro_actor_view_mut(k).set_x(i16::from(X[k]));
        self.intro_actor_view_mut(k).set_y(i16::from(Y[k]));
        self.intro_actor_view_mut(k).increment_init_phase();
    }

    pub(super) fn animate_scene_sprite_credits_triangle(&mut self, k: usize) {
        const XACC: [i8; 3] = [-1, 0, 1];
        const YACC: [i8; 3] = [1, -1, 1];
        self.load_triforce_sprite_palette();
        self.intro_copy_sprite_type4_to_oam(k);
        self.animate_scene_sprite_move_triangle(k);
        if self.frame_state().submodule != 36 {
            self.intro_actor_view_mut(k).set_state(0);
            return;
        }
        if self.intro_actor_view(k).state() != 80 {
            self.intro_actor_view_mut(k).increment_state();
            self.intro_actor_view_mut(k).add_x_velocity(XACC[k] as u8);
            self.intro_actor_view_mut(k).add_y_velocity(YACC[k] as u8);
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
        self.oam_state_view_mut().init_credits_region_base();
        match self.frame_state().submodule {
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
        match self.frame_state().subsubmodule {
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
            self.sprite_slot_view_mut(k).set_state(0);
            self.sprite_slot_view_mut(k).set_flags5(0);
            self.sprite_slot_view_mut(k).set_deflection_bits(0);
        }
        let scene = (self.frame_state().submodule >> 1) as usize;
        match scene {
            2 => {
                self.sprite_slot_view_mut(6).set_y_velocity((-16i8) as u8);
                self.init_ending_sprites_overworld(scene);
            }
            3 => {
                self.sprite_slot_view_mut(5).set_a(22);
                self.sprite_slot_view_mut(0).set_y_velocity((-16i8) as u8);
                self.sprite_slot_view_mut(1).set_y_velocity(16);
                self.sprite_slot_view_mut(1).set_head_direction(1);
                for j in (0..=2).rev() {
                    self.sprite_slot_view_mut(2 + j).set_sprite_type(0x57);
                    self.sprite_slot_view_mut(2 + j).set_oam_flags(0x31);
                }
                self.init_ending_sprites_overworld(scene);
            }
            6 => {
                self.sprite_slot_view_mut(0).set_delay_main(255);
                self.sprite_slot_view_mut(1).set_delay_main(255);
                self.sprite_slot_view_mut(2).set_delay_main(255);
                self.init_ending_sprites_overworld(scene);
            }
            7 => {
                self.sprite_slot_view_mut(1).set_delay_main(255);
                self.init_ending_sprites_overworld(scene);
            }
            9 => {
                for j in (0..=4).rev() {
                    self.sprite_slot_view_mut(j).set_delay_main((j * 19) as u8);
                    self.sprite_slot_view_mut(j).set_state(0);
                }
                self.sprite_slot_view_mut(5).set_sprite_type(0x2e);
                for j in (0..=1).rev() {
                    self.sprite_slot_view_mut(7 + j).set_sprite_type(0x9f);
                    self.sprite_slot_view_mut(9 + j).set_sprite_type(0xa0);
                    self.sprite_slot_view_mut(7 + j).set_flags2(1);
                    self.sprite_slot_view_mut(9 + j).set_flags2(2);
                    self.sprite_slot_view_mut(7 + j).set_flags3(0x10);
                    self.sprite_slot_view_mut(9 + j).set_flags3(0x10);
                }
                self.init_ending_sprites_overworld(scene);
            }
            10 => {
                self.sprite_slot_view_mut(1).set_delay_main(0x10);
                self.sprite_slot_view_mut(2).set_delay_main(0x20);
                self.sprite_slot_view_mut(3).set_oam_flags(8);
                self.sprite_slot_view_mut(4).set_oam_flags(8);
                self.init_ending_sprites_dungeon(scene);
            }
            11 => {
                self.sprite_slot_view_mut(4).set_oam_flags(0x79);
                self.sprite_slot_view_mut(5).set_oam_flags(0x39);
                self.sprite_slot_view_mut(1).set_direction(1);
                self.sprite_slot_view_mut(1).set_a(4);
                self.init_ending_sprites_dungeon(scene);
            }
            12 => {
                for j in (0..=1).rev() {
                    self.sprite_slot_view_mut(j + 3).set_oam_flags(0x39);
                    self.sprite_slot_view_mut(j + 3).set_sprite_type(0x0b);
                    self.sprite_slot_view_mut(j + 3).set_flags3(0x10);
                    self.sprite_slot_view_mut(j + 3).set_flags2(1);
                }
                self.sprite_slot_view_mut(5).set_sprite_type(0x2a);
                self.sprite_slot_view_mut(6).set_sprite_type(0x79);
                self.sprite_slot_view_mut(6).set_ai_state(1);
                self.sprite_slot_view_mut(6).set_z(5);
                self.init_ending_sprites_overworld(scene);
            }
            14 => {
                self.sprite_slot_view_mut(5).set_y_velocity((-16i8) as u8);
                self.sprite_slot_view_mut(6).set_y_velocity(16);
                self.sprite_slot_view_mut(6).set_head_direction(1);
                self.sprite_slot_view_mut(0).set_a(8);
                for j in (0..=3).rev() {
                    self.sprite_slot_view_mut(1 + j).set_y_velocity(4);
                }
                self.init_ending_sprites_overworld(scene);
            }
            15 => {
                self.sprite_slot_view_mut(4).set_c(2);
                self.sprite_slot_view_mut(5).set_y_velocity(8);
                self.sprite_slot_view_mut(1).set_delay_main(0x13);
                self.sprite_slot_view_mut(4).set_delay_main(0x40);
                self.init_ending_sprites_overworld(scene);
            }
            0 | 4 | 5 | 8 | 13 => self.init_ending_sprites_overworld(scene),
            1 => self.init_ending_sprites_dungeon(scene),
            _ => {}
        }
    }

    fn init_ending_sprites_overworld(&mut self, scene: usize) {
        let idx = ENDING_SCENE_SPRITE_RANGES[scene];
        let num = ENDING_SCENE_SPRITE_RANGES[scene + 1] - idx;
        let area = self.world_region().overworld_area_index();
        let base_x = area.wrapping_shl(9) & 0x0f00;
        let base_y = area.wrapping_shl(6) & 0x0e00;
        for k in (0..num).rev() {
            self.garnish_state_view_mut().set_sprcoll_x_size(0xffff);
            self.garnish_state_view_mut().set_sprcoll_y_size(0xffff);
            let x = base_x.wrapping_add(ENDING_SPRITE_X_OFFSETS[idx + k]);
            let y = base_y.wrapping_add(ENDING_SPRITE_Y_OFFSETS[idx + k]);
            self.sprite_slot_view_mut(k).set_x_low(x as u8);
            self.sprite_slot_view_mut(k).set_x_high((x >> 8) as u8);
            self.sprite_slot_view_mut(k).set_y_low(y as u8);
            self.sprite_slot_view_mut(k).set_y_high((y >> 8) as u8);
        }
    }

    fn init_ending_sprites_dungeon(&mut self, scene: usize) {
        let idx = ENDING_SCENE_SPRITE_RANGES[scene];
        let num = ENDING_SCENE_SPRITE_RANGES[scene + 1] - idx;
        let room = self.dungeon_room_tracking().room_index2_word();
        self.sprite_workspace_view_mut()
            .set_room_origin_y_high(((room >> 3) as u8) & 0xfe);
        self.sprite_workspace_view_mut()
            .set_room_origin_x_high(((room & 15) << 1) as u8);
        for k in (0..num).rev() {
            self.garnish_state_view_mut().set_sprcoll_x_size(0xffff);
            self.garnish_state_view_mut().set_sprcoll_y_size(0xffff);
            let x = ((self.sprite_workspace_view().room_origin_x_high() as u16) << 8)
                .wrapping_add(ENDING_SPRITE_X_OFFSETS[idx + k]);
            let y = ((self.sprite_workspace_view().room_origin_y_high() as u16) << 8)
                .wrapping_add(ENDING_SPRITE_Y_OFFSETS[idx + k]);
            self.sprite_slot_view_mut(k).set_x_low(x as u8);
            self.sprite_slot_view_mut(k).set_x_high((x >> 8) as u8);
            self.sprite_slot_view_mut(k).set_y_low(y as u8);
            self.sprite_slot_view_mut(k).set_y_high((y >> 8) as u8);
        }
    }

    pub(super) fn credits_scroll_scene_overworld(&mut self) {
        for k in (0..16).rev() {
            if self.sprite_slot_view(k).delay_main() != 0 {
                let delay = self.sprite_slot_view(k).delay_main().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_main(delay);
            }
        }
        let i = (self.frame_state().submodule >> 1) as usize;
        self.player_state_view_mut().clear_movement_velocity();
        let r16 = self.ending_scratch_view().primary_word();
        if r16 >= 0x40 && r16 & 1 == 0 {
            if self.world_scroll().bg2_y() != ENDING_SCENE_SCROLL_TARGET_Y[i] {
                self.player_state_view_mut()
                    .set_y_velocity(ENDING_SCENE_SCROLL_Y_VELOCITIES[i] as u8);
            }
            if self.world_scroll().bg2_x() != ENDING_SCENE_SCROLL_TARGET_X[i] {
                self.player_state_view_mut()
                    .set_x_velocity(ENDING_SCENE_SCROLL_X_VELOCITIES[i] as u8);
            }
        }
        self.credits_operate_scrolling_and_tile_map();
        self.credits_handle_scene_fade();
    }

    pub(super) fn credits_scroll_scene_dungeon(&mut self) {
        for k in (0..16).rev() {
            if self.sprite_slot_view(k).delay_main() != 0 {
                let delay = self.sprite_slot_view(k).delay_main().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_main(delay);
            }
        }
        let i = (self.frame_state().submodule >> 1) as usize;
        let r16 = self.ending_scratch_view().primary_word();
        if r16 >= 0x40 && r16 & 1 == 0 {
            if self.world_scroll().bg2_y() != ENDING_SCENE_SCROLL_TARGET_Y[i] {
                self.ppu_scroll_copy_view_mut()
                    .add_bg2_v_copy2_signed(ENDING_SCENE_SCROLL_Y_VELOCITIES[i]);
            }
            if self.world_scroll().bg2_x() != ENDING_SCENE_SCROLL_TARGET_X[i] {
                self.ppu_scroll_copy_view_mut()
                    .add_bg2_h_copy2_signed(ENDING_SCENE_SCROLL_X_VELOCITIES[i]);
            }
        }
        self.credits_handle_scene_fade();
    }

    pub(super) fn credits_handle_scene_fade(&mut self) {
        const CREDITS_SCENE_FADE_SCROLL_LIMITS: [u16; 16] = [
            0x300, 0x280, 0x250, 0x2e0, 0x280, 0x250, 0x2c0, 0x2c0, 0x250, 0x250, 0x280, 0x250,
            0x480, 0x400, 0x250, 0x500,
        ];
        const CREDITS_CASE0_SPRITE_CHARS: [u8; 12] = [
            0x1e, 0x20, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x16, 0x16, 0x16, 0x16,
        ];
        const CREDITS_CASE0_SPRITE_GFX: [u8; 12] = [6, 3, 2, 2, 2, 2, 2, 2, 6, 6, 6, 6];
        const CASE0_OAM_FLAGS: [u8; 12] = [
            0x3b, 0x31, 0x3d, 0x3f, 0x39, 0x3b, 0x37, 0x3d, 0x39, 0x37, 0x37, 0x39,
        ];

        let i = (self.frame_state().submodule >> 1) as usize;
        let r16 = self.ending_scratch_view().primary_word();
        match i {
            0 => {
                for k in (8..=11).rev() {
                    self.sprite_slot_view_mut(k)
                        .set_oam_flags(CASE0_OAM_FLAGS[k]);
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_CASE0_SPRITE_GFX[k],
                        CREDITS_CASE0_SPRITE_CHARS[k],
                    );
                }
                for k in (2..=7).rev() {
                    let oam_flags =
                        CASE0_OAM_FLAGS[k] | ((self.frame_state().frame_counter << 2) & 0x40);
                    self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_CASE0_SPRITE_GFX[k],
                        CREDITS_CASE0_SPRITE_CHARS[k],
                    );
                }
                for k in (0..=1).rev() {
                    self.sprite_slot_view_mut(k)
                        .set_oam_flags(CASE0_OAM_FLAGS[k]);
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_CASE0_SPRITE_GFX[k],
                        CREDITS_CASE0_SPRITE_CHARS[k],
                    );
                }
            }
            1 => {
                self.credits_sprite_draw_single(0, 3, 12);
                self.credits_sprite_draw_draw_shadow(0);
                let k = 1;
                self.sprite_slot_view_mut(k).set_sprite_type(0x73);
                self.sprite_slot_view_mut(k).set_oam_flags(0x27);
                self.sprite_slot_view_mut(k).set_e(2);
                self.credits_sprite_draw_preexisting_sprite_draw(k, 16);
            }
            2 => {
                const CREDITS_CASE2_BIRD_FLAG_FRAMES: [u8; 2] = [0x20, 0x40];
                const CREDITS_CASE2_BIRD_OAM_VELOCITY_OFFSETS: [i8; 2] = [16, -16];
                const CREDITS_CASE2_SPRITE_CHARS: [u8; 5] = [0x28, 0x2a, 0x2c, 0x2e, 0x2c];
                const CREDITS_CASE2_SPRITE_GFX: [u8; 5] = [3, 3, 3, 3, 3];
                const CASE2_DELAY: [u8; 2] = [0x30, 0x10];
                let bird_frame_idx = ((self.frame_state().frame_counter >> 2) & 1) as usize;
                self.world_transient_mut()
                    .set_flag_travel_bird(CREDITS_CASE2_BIRD_FLAG_FRAMES[bird_frame_idx]);
                let mut k = 6usize;
                let j = ((self.sprite_slot_view(k).x_velocity() >> 7) & 1) as usize;
                let oam_flags = self
                    .sprite_slot_view(k)
                    .x_velocity()
                    .wrapping_add(CREDITS_CASE2_BIRD_OAM_VELOCITY_OFFSETS[j] as u8)
                    >> 1
                    & 0x40
                    | 0x32;
                self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                self.credits_sprite_draw_single(k, 2, 0x24);
                self.credits_sprite_draw_circling_birds(k);
                k -= 1;
                self.sprite_slot_view_mut(k).set_oam_flags(0x31);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = self.sprite_slot_view(k).a() as usize;
                    let a = self.sprite_slot_view(k).a() ^ 1;
                    self.sprite_slot_view_mut(k).set_a(a);
                    self.sprite_slot_view_mut(k).set_delay_main(CASE2_DELAY[j]);
                    let graphics = self.sprite_slot_view(k).graphics().wrapping_add(1) & 3;
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                }
                self.credits_sprite_draw_single(k, 2, 0x26);
                k -= 1;
                loop {
                    if self.frame_state().frame_counter & 15 == 0 {
                        let graphics = self.sprite_slot_view(k).graphics() ^ 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                    }
                    self.sprite_slot_view_mut(k).set_oam_flags(0x31);
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_CASE2_SPRITE_GFX[k],
                        CREDITS_CASE2_SPRITE_CHARS[k],
                    );
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
                        self.sprite_slot_view_mut(k).set_sprite_type(1);
                        self.sprite_slot_view_mut(k).set_oam_flags(0x0b);
                        self.credits_sprite_draw_set_shadow_prop(k, 2);
                        self.sprite_slot_view_mut(k).set_z(48);
                        let j = ((self.frame_state().frame_counter.wrapping_add(if k != 0 {
                            0x5f
                        } else {
                            0x7d
                        })) >> 2
                            & 3) as usize;
                        self.sprite_slot_view_mut(k).set_graphics(CASE3_GFX[j]);
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
                const CREDITS_CASE4_SPRITE_CHARS: [u8; 2] = [0x30, 0x32];
                const CREDITS_CASE4_SPRITE_GFX: [u8; 2] = [2, 2];
                const CASE4_CTR: [u16; 2] = [0x20, 0];
                const CASE4_XYVEL: [i8; 10] = [0, -12, -16, -12, 0, 12, 16, 12, 0, -12];
                const CASE4_DELAYVEL: [u8; 24] = [
                    0x3b, 0x14, 0x1e, 0x1d, 0x2c, 0x2b, 0x42, 0x20, 0x27, 0x28, 0x2e, 0x38, 0x3a,
                    0x4c, 0x32, 0x44, 0x2e, 0x2f, 0x1e, 0x28, 0x47, 0x35, 0x32, 0x30,
                ];
                let mut k = 2usize;
                self.sprite_slot_view_mut(k).set_oam_flags(0x35);
                self.credits_sprite_draw_single(k, 1, 0x3c);
                k -= 1;
                loop {
                    let oam_flags =
                        self.sprite_slot_view(k).x_velocity().wrapping_sub(1) >> 1 & 0x40 ^ 0x71;
                    self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                    let graphics = self.frame_state().frame_counter >> 3 & 1;
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                    if r16 >= CASE4_CTR[k] && self.sprite_slot_view(k).delay_main() == 0 {
                        let a = CASE4_DELAYVEL[self.sprite_slot_view(k).a() as usize];
                        self.sprite_slot_view_mut(k).set_delay_main(a & 0xf8);
                        self.sprite_slot_view_mut(k)
                            .set_y_velocity(CASE4_XYVEL[((a & 7) + 2) as usize] as u8);
                        self.sprite_slot_view_mut(k)
                            .set_x_velocity(CASE4_XYVEL[(a & 7) as usize] as u8);
                        self.sprite_slot_view_mut(k).increment_a();
                    }
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_CASE4_SPRITE_GFX[k],
                        CREDITS_CASE4_SPRITE_CHARS[k],
                    );
                    self.end_sequence_draw_shadow2(k);
                    self.sprite_move_xy(k);
                    if k == 0 {
                        break;
                    }
                    k -= 1;
                }
            }
            5 => {
                const CREDITS_CASE5_SHIELD_DMA_GFX: [u8; 2] = [0, 4];
                const CREDITS_CASE5_LINK_DMA_GFX: [u16; 2] = [0x0a, 0x224];
                const CREDITS_CASE5_SPRITE_CHARS: [u8; 2] = [10, 14];
                if r16 == 0x200 {
                    self.system_signals_view_mut().set_sound_effect_1(1);
                } else if r16 == 0x208 {
                    self.system_signals_view_mut().set_sound_effect_1(0x2c);
                }
                if r16.wrapping_sub(0x208) < 0x30 {
                    self.credits_sprite_draw_add_sparkle(2, 10, r16.wrapping_sub(0x208) as u8);
                }
                let mut k = 3usize;
                if r16 >= 0x200 {
                    self.sprite_slot_view_mut(k).set_graphics(1);
                }
                self.sprite_slot_view_mut(k).set_oam_flags(0x31);
                self.credits_sprite_draw_single(k, 4, 8);
                self.end_sequence_draw_shadow2(k);
                let j = self.sprite_slot_view(k).graphics() as usize;
                k -= 1;
                self.sprite_slot_view_mut(k).set_graphics(j as u8);
                {
                    let mut player = self.player_state_view_mut();
                    player.set_sword_dma_graphics_index(0);
                    player.set_shield_dma_graphics_index(CREDITS_CASE5_SHIELD_DMA_GFX[j]);
                }
                self.sprite_slot_view_mut(k).set_oam_flags(0x30);
                self.player_state_view_mut()
                    .set_link_dma_graphics_index_word(CREDITS_CASE5_LINK_DMA_GFX[j]);
                self.credits_sprite_draw_single(k, 5, CREDITS_CASE5_SPRITE_CHARS[j]);
                self.end_sequence_draw_shadow2(k);
            }
            6 => {
                const SPR_TYPE: [u8; 3] = [0x52, 0x55, 0x55];
                const OAM_SIZE: [u8; 3] = [0x20, 8, 8];
                const STATE: [u8; 3] = [3, 1, 1];
                const LOCAL_GRAPHICS: [u8; 6] = [0, 5, 5, 1, 6, 6];
                let idx = ENDING_SCENE_SPRITE_RANGES[i];
                let num = ENDING_SCENE_SPRITE_RANGES[i + 1] - idx;
                for k in (0..num).rev() {
                    self.sprite_system_view_mut().set_cur_object_index(k as u8);
                    self.sprite_slot_view_mut(k).set_sprite_type(SPR_TYPE[k]);
                    self.oam_allocate_from_region_a(OAM_SIZE[k]);
                    self.sprite_slot_view_mut(k).set_ai_state(STATE[k]);
                    let j = if r16 >= 0x26f { k + 3 } else { k };
                    if r16 == 0x26f {
                        self.system_signals_view_mut().set_sound_effect_2(0x21);
                    }
                    self.sprite_slot_view_mut(k).set_graphics(LOCAL_GRAPHICS[j]);
                    self.sprite_slot_view_mut(k).set_oam_flags(0x33);
                    self.sprite_get_16_bit_coords_ending(k);
                    self.sprite_active_main_ending(k);
                }
            }
            7 => {
                let mut k = 1usize;
                self.credits_sprite_draw_set_shadow_prop(k, 2);
                self.sprite_slot_view_mut(k).set_sprite_type(0xe9);
                self.oam_allocate_from_region_a(0x0c);
                self.sprite_slot_view_mut(k).set_oam_flags(0x37);
                self.sprite_get_16_bit_coords_ending(k);
                if self.frame_state().frame_counter & 15 == 0 {
                    let graphics = self.sprite_slot_view(k).graphics() ^ 1;
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                }
                self.sprite_active_main_ending(k);
                if r16 >= 0x180 {
                    self.sprite_slot_view_mut(k).set_y_velocity(4);
                    if self.sprite_slot_view(k).y_low() != 0x7c {
                        self.sprite_move_xy(k);
                    }
                }
                k -= 1;
                self.sprite_slot_view_mut(k).set_sprite_type(0x36);
                self.oam_allocate_from_region_a(0x18);
                self.sprite_slot_view_mut(k).set_oam_flags(0x39);
                self.sprite_get_16_bit_coords_ending(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    const GFX_STEP: [i8; 2] = [1, -1];
                    self.sprite_slot_view_mut(k).set_delay_main(4);
                    let graphics = self
                        .sprite_slot_view(k)
                        .graphics()
                        .wrapping_add(GFX_STEP[((r16 >> 9) & 1) as usize] as u8)
                        & 7;
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                }
                self.sprite_active_main_ending(k);
            }
            8 => {
                let k = 0usize;
                self.sprite_slot_view_mut(k).set_sprite_type(0x2c);
                self.oam_allocate_from_region_a(0x2c);
                self.sprite_slot_view_mut(k).set_oam_flags(0x3b);
                self.sprite_get_16_bit_coords_ending(k);
                let graphics = if r16 < 0x1c0 {
                    ((r16 >> 5) & 1) as u8
                } else {
                    2
                };
                self.sprite_slot_view_mut(k).set_graphics(graphics);
                self.sprite_active_main_ending(k);
            }
            9 => {
                let mut k = 0usize;
                while k < 5 {
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        self.sprite_slot_view_mut(k).set_delay_main(96);
                        self.sprite_slot_view_mut(k).set_state(96);
                        self.sprite_slot_view_mut(k).set_x_velocity(0);
                        self.sprite_slot_view_mut(k).set_x_low(238);
                        self.sprite_slot_view_mut(k).set_x_high(4);
                        self.sprite_slot_view_mut(k).set_y_low(24);
                        self.sprite_slot_view_mut(k).set_y_high(11);
                    }
                    if self.sprite_slot_view(k).state() != 0 {
                        self.sprite_slot_view_mut(k).set_y_velocity((-8i8) as u8);
                        self.sprite_move_xy(k);
                        if self.frame_state().frame_counter & 1 == 0 {
                            let delta =
                                if ((self.frame_state().frame_counter >> 5) ^ k as u8) & 1 != 0 {
                                    -1i8
                                } else {
                                    1i8
                                };
                            self.sprite_slot_view_mut(k).add_x_velocity(delta as u8);
                        }
                        self.credits_sprite_draw_single(k, 1, 0x10);
                    }
                    k += 1;
                }
                loop {
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        const DELAY1: [u8; 4] = [16, 14, 16, 18];
                        const DELAY2: [u8; 4] = [20, 48, 20, 20];
                        let a = self.sprite_slot_view(k).a() as usize;
                        let delay = if k == 5 { DELAY1[a] } else { DELAY2[a] };
                        self.sprite_slot_view_mut(k).set_delay_main(delay);
                        let next_a = self.sprite_slot_view(k).a().wrapping_add(1) & 3;
                        self.sprite_slot_view_mut(k).set_a(next_a);
                        let graphics = self.sprite_slot_view(k).graphics() ^ 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                    }
                    if k == 5 {
                        self.sprite_slot_view_mut(k).set_oam_flags(0x31);
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
                    const CREDITS_CASE8_RUN_SPRITE_CHARS: [u8; 4] = [8, 8, 12, 12];
                    self.sprite_slot_view_mut(k).set_oam_flags(OAM_FLAGS[k - 7]);
                    self.sprite_slot_view_mut(k).set_direction(D[k - 7]);
                    self.credits_sprite_draw_activate_and_run_sprite(
                        k,
                        CREDITS_CASE8_RUN_SPRITE_CHARS[k - 7],
                    );
                    k += 1;
                }
            }
            10 => {
                const WISH_POND_X: [u8; 8] = [0, 4, 8, 12, 16, 20, 24, 0];
                const WISH_POND_Y: [u8; 8] = [0, 8, 16, 24, 32, 40, 4, 36];
                let k = 5usize;
                self.sprite_get_16_bit_coords_ending(k);
                if self.sprite_slot_view(k).pause() == 0 {
                    let xb = WISH_POND_X[(self.get_random_number() & 7) as usize]
                        .wrapping_add(self.sprite_workspace_view().current_sprite_x() as u8);
                    let yb = WISH_POND_Y[(self.get_random_number() & 7) as usize]
                        .wrapping_add(self.sprite_workspace_view().current_sprite_y() as u8);
                    self.credits_sprite_draw_add_sparkle(3, xb, yb);
                }
                for k in 3..5 {
                    if self.sprite_slot_view(k).delay_aux1() != 0 {
                        let delay = self.sprite_slot_view(k).delay_aux1().wrapping_sub(1);
                        self.sprite_slot_view_mut(k).set_delay_aux1(delay);
                    }
                    self.sprite_slot_view_mut(k).set_sprite_type(0xe3);
                    self.credits_sprite_draw_set_shadow_prop(k, 1);
                    self.credits_sprite_draw_activate_and_run_sprite(k, 8);
                }
                self.sprite_slot_view_mut(k).set_sprite_type(0x72);
                self.sprite_slot_view_mut(k).set_oam_flags(0x3b);
                self.sprite_slot_view_mut(k).set_state(9);
                self.sprite_slot_view_mut(k).set_b(9);
                self.credits_sprite_draw_preexisting_sprite_draw(k, 0x30);
            }
            11 => {
                if r16 >= 0x170 {
                    for k in 4..6 {
                        self.credits_sprite_draw_single(k, 1, 0x3e);
                    }
                    let k = 0usize;
                    self.sprite_slot_view_mut(k).set_oam_flags(0x39);
                    if r16 < 0x1c0 {
                        self.sprite_slot_view_mut(k).set_graphics(2);
                    } else if self.sprite_slot_view(k).delay_main() == 0 {
                        self.sprite_slot_view_mut(k).set_delay_main(0x20);
                        let graphics = (self.sprite_slot_view(k).graphics() ^ 1) & 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                    }
                    self.credits_sprite_draw_single(k, 4, 6);
                } else {
                    const LOCAL_GRAPHICS: [u8; 16] =
                        [1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0];
                    for k in 0..2 {
                        self.sprite_slot_view_mut(k).set_sprite_type(0x1a);
                        self.sprite_slot_view_mut(k).set_oam_flags(0x39);
                        self.credits_sprite_draw_set_shadow_prop(k, 2);
                        let bak0 = self.frame_state().main_module;
                        self.credits_sprite_draw_activate_and_run_sprite(k, 0x0c);
                        self.set_main_module(bak0);
                        if self.sprite_slot_view(k).b() == 15 && self.sprite_slot_view(k).a() == 4 {
                            self.sprite_slot_view_mut(k + 2).set_delay_main(15);
                        }
                        let j = self.sprite_slot_view(k + 2).delay_main();
                        if j != 0 {
                            self.sprite_slot_view_mut(k + 2).set_oam_flags(2);
                            self.sprite_slot_view_mut(k + 2)
                                .set_graphics(LOCAL_GRAPHICS[j as usize]);
                            self.credits_sprite_draw_single(k + 2, 2, 0x36);
                        }
                    }
                }
            }
            12 => {
                let mut k = 6usize;
                let graphics = self.frame_state().frame_counter & 1;
                self.sprite_slot_view_mut(k).set_graphics(graphics);
                if self.sprite_slot_view(k).graphics() == 0 {
                    let x_delta = if sign8(self.sprite_slot_view(k).x_low().wrapping_sub(0x80)) {
                        1
                    } else {
                        0xff
                    };
                    let y_delta = if sign8(self.sprite_slot_view(k).y_low().wrapping_sub(0xb0)) {
                        1
                    } else {
                        0xff
                    };
                    self.sprite_slot_view_mut(k).add_x_velocity(x_delta);
                    self.sprite_slot_view_mut(k).add_y_velocity(y_delta);
                    self.sprite_move_xy(k);
                }
                let oam_flags = (self.sprite_slot_view(k).x_velocity() >> 1 & 0x40) ^ 0x7e;
                self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                self.sprite_slot_view_mut(k).set_flags2(1);
                self.sprite_slot_view_mut(k).set_flags3(0x30);
                self.sprite_slot_view_mut(k).set_z(16);
                self.credits_sprite_draw_preexisting_sprite_draw(k, 8);
                k -= 1;
                self.sprite_slot_view_mut(k).set_oam_flags(0x37);
                self.credits_sprite_draw_set_shadow_prop(k, 2);
                self.credits_sprite_draw_activate_and_run_sprite(k, 12);
                k -= 1;
                self.credits_sprite_draw_activate_and_run_sprite(k, 8);
                k -= 1;
                self.credits_sprite_draw_activate_and_run_sprite(k, 8);
                k -= 1;
                loop {
                    const CREDITS_CASE12_SPRITE_GFX: [u8; 3] = [3, 3, 8];
                    const Z: [u8; 15] = [2, 4, 5, 6, 6, 7, 7, 7, 7, 6, 6, 5, 4, 2, 0];
                    self.credits_sprite_draw_single(k, CREDITS_CASE12_SPRITE_GFX[k], (k * 2) as u8);
                    if k == 0 {
                        self.ending_func2(k, 0x30);
                    } else if k & !1 != 0 {
                        let graphics = self.frame_state().frame_counter >> 3 & 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                    } else {
                        let j = (self.frame_state().frame_counter & 0x1f) as usize;
                        if j < 0x0f {
                            self.sprite_slot_view_mut(k).set_z(Z[j]);
                        }
                        self.sprite_slot_view_mut(k)
                            .set_graphics(if j < 0x0f { 1 } else { 0 });
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
                    self.sprite_slot_view_mut(k).set_x_velocity((-4i8) as u8);
                }
                let graphics = self.frame_state().frame_counter >> 4 & 1;
                self.sprite_slot_view_mut(k).set_graphics(graphics);
                if self.sprite_slot_view(k).x_low() == 56 {
                    self.sprite_slot_view_mut(k).set_x_velocity(0);
                    let graphics = self.sprite_slot_view(k).graphics().wrapping_add(2);
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                }
                self.credits_sprite_draw_single(k, 3, 0x34);
                self.sprite_move_xy(k);
            }
            14 => {
                const CREDITS_CASE14_BIRD_GFX_FRAMES: [u8; 4] = [0, 1, 0, 2];
                const CREDITS_CASE14_TIMING_THRESHOLDS: [u8; 5] = [2, 8, 32, 32, 8];
                let mut k = 6usize;
                while k != 0 {
                    if k >= 5 {
                        self.sprite_slot_view_mut(k).set_sprite_type(0);
                        self.credits_sprite_draw_set_shadow_prop(k, 1);
                        let graphics =
                            (self.frame_state().frame_counter.wrapping_add(0x4a) & 8) >> 3;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                        self.sprite_slot_view_mut(k).set_z(32);
                        self.credits_sprite_draw_circling_birds(k);
                        let oam_flags = (self.sprite_slot_view(k).x_velocity() >> 1 & 0x40) ^ 0x0f;
                        self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 8);
                    } else {
                        self.sprite_slot_view_mut(k).set_sprite_type(0x0d);
                        if k == 1 {
                            self.sprite_slot_view_mut(k).set_head_direction(0x0d);
                        }
                        self.credits_sprite_draw_set_shadow_prop(k, 3);
                        self.sprite_slot_view_mut(k).set_oam_flags(0x2b);
                        let mut a = self.sprite_slot_view(k).delay_main();
                        if a == 0 {
                            a = 0xc0;
                            self.sprite_slot_view_mut(k).set_delay_main(a);
                        }
                        a >>= 1;
                        if a == 0 {
                            self.sprite_slot_view_mut(k).set_y_velocity(0);
                            self.sprite_slot_view_mut(k).set_x_velocity(0);
                        } else if a < CREDITS_CASE14_TIMING_THRESHOLDS[k]
                            && self.frame_state().frame_counter & 3 == 0
                            && self.sprite_slot_view(k).y_velocity() != 0
                        {
                            let mut v = self.sprite_slot_view(k).y_velocity().wrapping_sub(1);
                            self.sprite_slot_view_mut(k).set_y_velocity(v);
                            v = v.wrapping_sub(4);
                            if k < 3 {
                                v = (0u8).wrapping_sub(v);
                            }
                            self.sprite_slot_view_mut(k).set_x_velocity(v);
                        }
                        self.sprite_move_xy(k);
                        let graphics = CREDITS_CASE14_BIRD_GFX_FRAMES
                            [((self.frame_state().frame_counter >> 3) & 3) as usize];
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
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
                let sparkle_idx = sparkle_table[self.frame_state().frame_counter as usize] & 3;
                self.credits_sprite_draw_add_sparkle(
                    2,
                    X[sparkle_idx as usize],
                    Y[sparkle_idx as usize],
                );
                let mut k = 2usize;
                self.sprite_slot_view_mut(k).set_sprite_type(0x62);
                self.sprite_slot_view_mut(k).set_oam_flags(0x39);
                self.credits_sprite_draw_preexisting_sprite_draw(k, 0x18);
                let mut j = 1u8;
                loop {
                    k += 1;
                    if self.sprite_slot_view(k).delay_aux1() != 0 {
                        let delay = self.sprite_slot_view(k).delay_aux1().wrapping_sub(1);
                        self.sprite_slot_view_mut(k).set_delay_aux1(delay);
                    }
                    let oam_flags =
                        (self.sprite_slot_view(k).x_velocity() >> 1 & 0x40) ^ OAM_FLAGS[j as usize];
                    self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        self.sprite_slot_view_mut(k).set_delay_main(128);
                        self.sprite_slot_view_mut(k).set_a(0);
                    }
                    if self.sprite_slot_view(k).a() == 0 {
                        let graphics = (self.frame_state().frame_counter >> 2 & 1) + 2;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                        self.credits_sprite_draw_move_squirrel(k);
                    } else if self.sprite_slot_view(k).delay_aux1() == 0 {
                        if self.sprite_slot_view(k).b() == 8 {
                            self.sprite_slot_view_mut(k).set_b(0);
                        }
                        let b = self.sprite_slot_view(k).b() & 7;
                        self.sprite_slot_view_mut(k)
                            .set_delay_aux1(DELAY[b as usize]);
                        let graphics = (self.sprite_slot_view(k).graphics() & 1) ^ 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                        let b = self.sprite_slot_view(k).b().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_b(b);
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

        let k = (self.frame_state().submodule >> 1) as usize;
        let r16 = self.ending_scratch_view().primary_word();
        if r16 >= CREDITS_SCENE_FADE_SCROLL_LIMITS[k] {
            if r16 & 1 == 0 {
                self.decrement_screen_brightness();
                if self.display_state().screen_brightness == 0 {
                    self.increment_submodule();
                } else {
                    self.ending_scratch_view_mut()
                        .set_primary_word(r16.wrapping_add(1));
                }
            } else {
                self.ending_scratch_view_mut()
                    .set_primary_word(r16.wrapping_add(1));
            }
        } else {
            if r16 & 1 == 0 && self.display_state().screen_brightness != 15 {
                self.increment_screen_brightness();
            }
            self.ending_scratch_view_mut()
                .set_primary_word(r16.wrapping_add(1));
        }
        self.ppu_scroll_copy_view_mut().copy_live_to_ppu_copy();
    }

    pub(super) fn credits_sprite_draw_draw_shadow(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_oam_flags(0x30);
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
        const ENDING_SPRITE_ANIMATION_STEPS: [i8; 28] = [
            0, 0, 1, 0, 1, 0, 2, 3, 0, 2, 0, 1, 0, 1, 0, 1, 2, 3, 4, 5, 6, 3, 0, -1, -1, -1, 2, 3,
        ];
        self.sprite_slot_view_mut(k).set_oam_flags(ain);
        self.end_sequence_draw_shadow2(k);
        let mut j = self.sprite_slot_view(k).a();
        if self.sprite_slot_view(k).delay_main() == 0 {
            j = j.wrapping_add(1);
            if j == 8 {
                j = 6;
            } else if j == 22 {
                j = 21;
            } else if j == 28 {
                j = 27;
            }
            self.sprite_slot_view_mut(k).set_a(j);
            self.sprite_slot_view_mut(k)
                .set_delay_main(DELAY[j.wrapping_sub(1) as usize]);
        }
        let a = ENDING_SPRITE_ANIMATION_STEPS[j as usize];
        let graphics = if a == -1 {
            self.frame_state().frame_counter >> 3 & 1
        } else {
            a as u8
        };
        self.sprite_slot_view_mut(k).set_graphics(graphics);
        if (j < 5 || (10..15).contains(&j)) && self.frame_state().frame_counter & 1 == 0 {
            let y_low = self.sprite_slot_view(k).y_low().wrapping_add(1);
            self.sprite_slot_view_mut(k).set_y_low(y_low);
        }
    }

    pub(super) fn credits_sprite_draw_activate_and_run_sprite(&mut self, k: usize, a: u8) {
        self.sprite_system_view_mut().set_cur_object_index(k as u8);
        self.oam_allocate_from_region_a(a);
        self.sprite_get_16_bit_coords_ending(k);
        let bak0 = self.frame_state().submodule;
        self.set_submodule(0);
        self.sprite_slot_view_mut(k).set_state(9);
        self.sprite_active_main_ending(k);
        self.set_submodule(bak0);
    }

    pub(super) fn credits_sprite_draw_preexisting_sprite_draw(&mut self, k: usize, a: u8) {
        self.oam_allocate_from_region_a(a);
        self.sprite_system_view_mut().set_cur_object_index(k as u8);
        self.sprite_get_16_bit_coords_ending(k);
        self.sprite_active_main_ending(k);
    }

    pub(super) fn credits_sprite_draw_single(&mut self, k: usize, a: u8, j: u8) {
        self.oam_allocate_from_region_a(a.wrapping_mul(4));
        self.sprite_get_16_bit_coords_ending(k);
        let entries = END_SEQUENCE_DRAW_FRAME_SETS[(j >> 1) as usize];
        let start = a as usize * self.sprite_slot_view(k).graphics() as usize;
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
        self.sprite_slot_view_mut(k).set_flags2(a);
        self.sprite_slot_view_mut(k).set_flags3(16);
    }

    pub(super) fn credits_sprite_draw_add_sparkle(&mut self, j_count: usize, xb: u8, yb: u8) {
        const DELAY: [u8; 6] = [32, 4, 4, 4, 5, 6];
        self.sprite_slot_view_mut(0).set_c(j_count as u8);
        for k in 0..j_count {
            let mut j = self.sprite_slot_view(k).graphics();
            if self.sprite_slot_view(k).delay_main() == 0 {
                j = j.wrapping_add(1);
                if j >= 6 {
                    self.sprite_slot_view_mut(k).set_x_low(xb);
                    self.sprite_slot_view_mut(k).set_y_low(yb);
                    j = 0;
                }
                self.sprite_slot_view_mut(k).set_graphics(j);
                self.sprite_slot_view_mut(k)
                    .set_delay_main(DELAY[j as usize]);
            }
            if j != 0 {
                self.credits_sprite_draw_single(k, 1, 0x1c);
            }
        }
    }

    pub(super) fn credits_sprite_draw_walk_link_away_from_pedestal(&mut self, k: usize) {
        const DMA: [u16; 8] = [0x16c, 0x16e, 0x170, 0x172, 0x16c, 0x174, 0x176, 0x178];
        if self.sprite_slot_view(k).delay_main() == 0 {
            let graphics = self.sprite_slot_view(k).graphics().wrapping_add(1) & 7;
            self.sprite_slot_view_mut(k).set_graphics(graphics);
            self.sprite_slot_view_mut(k).set_delay_main(4);
        }
        let dma = DMA[self.sprite_slot_view(k).graphics() as usize];
        self.player_state_view_mut()
            .set_link_dma_graphics_index_word(dma);
        self.sprite_slot_view_mut(k).set_oam_flags(32);
        self.credits_sprite_draw_single(k, 2, 26);
        self.end_sequence_draw_shadow2(k);
        self.sprite_move_xy(k);
    }

    pub(super) fn credits_sprite_draw_move_squirrel(&mut self, k: usize) {
        const LOCAL_X_VELOCITIES: [i8; 4] = [32, 24, -32, -24];
        const LOCAL_Y_VELOCITIES: [i8; 4] = [8, -8, -8, 8];
        if self.sprite_slot_view(k).delay_main() < 64 {
            let c = self.sprite_slot_view(k).c().wrapping_add(1) & 3;
            self.sprite_slot_view_mut(k).set_c(c);
            self.sprite_slot_view_mut(k).increment_a();
        } else {
            let j = self.sprite_slot_view(k).c() as usize;
            self.sprite_slot_view_mut(k)
                .set_x_velocity(LOCAL_X_VELOCITIES[j] as u8);
            self.sprite_slot_view_mut(k)
                .set_y_velocity(LOCAL_Y_VELOCITIES[j] as u8);
            self.sprite_move_xy(k);
        }
    }

    pub(super) fn credits_sprite_draw_circling_birds(&mut self, k: usize) {
        const TARGET_X: [i8; 2] = [0x20, -0x20];
        const TARGET_Y: [i8; 2] = [0x10, -0x10];
        let j = self.sprite_slot_view(k).direction() & 1;
        let x_velocity = self
            .sprite_slot_view(k)
            .x_velocity()
            .wrapping_add(if j != 0 { 0xff } else { 1 });
        self.sprite_slot_view_mut(k).set_x_velocity(x_velocity);
        if self.sprite_slot_view(k).x_velocity() == TARGET_X[j as usize] as u8 {
            self.sprite_slot_view_mut(k).increment_direction();
        }
        if self.frame_state().frame_counter & 1 == 0 {
            let j = self.sprite_slot_view(k).head_direction() & 1;
            let y_velocity = self
                .sprite_slot_view(k)
                .y_velocity()
                .wrapping_add(if j != 0 { 0xff } else { 1 });
            self.sprite_slot_view_mut(k).set_y_velocity(y_velocity);
            if self.sprite_slot_view(k).y_velocity() == TARGET_Y[j as usize] as u8 {
                let head_direction = self.sprite_slot_view(k).head_direction().wrapping_add(1);
                self.sprite_slot_view_mut(k)
                    .set_head_direction(head_direction);
            }
        }
        self.sprite_move_xy(k);
    }

    pub(super) fn credits_handle_camera_scroll_control(&mut self) {
        if self.player_state_view().y_velocity() != 0 {
            let y_vel = self.player_state_view().y_velocity_signed();
            self.ppu_scroll_copy_view_mut()
                .add_bg2_v_copy2_signed(y_vel);
            let which_axis = if y_vel < 0 { 0 } else { 1 };
            let other_axis = if y_vel < 0 { 1 } else { 0 };
            let mut value = self
                .world_camera_boundaries()
                .overworld_scroll_counter_for_axis(which_axis)
                .wrapping_add(y_vel.unsigned_abs() as u16);
            if (value as i16).wrapping_sub(0x10) >= 0 {
                value = value.wrapping_sub(0x10);
                let bits =
                    self.screen_transition_direction_bits_word() | if y_vel < 0 { 8 } else { 4 };
                self.set_screen_transition_direction_bits_word(bits);
            }
            self.world_camera_boundaries_mut()
                .set_overworld_scroll_counter_for_axis(which_axis, value);
            self.world_camera_boundaries_mut()
                .set_overworld_scroll_counter_for_axis(other_axis, 0u16.wrapping_sub(value));
            let mut r4 = y_vel as i16 as u16;
            self.set_overworld_vertical_scroll_delta(r4);
            let oi = self.world_region().overlay_index();
            if oi != 0x97 && oi != 0x9d {
                let subp;
                if oi == 0xb5 || oi == 0xbe {
                    subp = (r4 & 3) << 14;
                    r4 = ((r4 as i16) >> 2) as u16;
                } else {
                    subp = (r4 & 1) << 15;
                    r4 = ((r4 as i16) >> 1) as u16;
                }
                self.ppu_scroll_copy_view_mut()
                    .add_bg1_v_copy2_subpixel(subp, r4);
            }
        }
        if self.player_state_view().x_velocity() != 0 {
            let x_vel = self.player_state_view().x_velocity_signed();
            self.ppu_scroll_copy_view_mut()
                .add_bg2_h_copy2_signed(x_vel);
            let which_axis = if x_vel < 0 { 2 } else { 3 };
            let other_axis = if x_vel < 0 { 3 } else { 2 };
            let mut value = self
                .world_camera_boundaries()
                .overworld_scroll_counter_for_axis(which_axis)
                .wrapping_add(x_vel.unsigned_abs() as u16);
            if (value as i16).wrapping_sub(0x10) >= 0 {
                value = value.wrapping_sub(0x10);
                let bits =
                    self.screen_transition_direction_bits_word() | if x_vel < 0 { 2 } else { 1 };
                self.set_screen_transition_direction_bits_word(bits);
            }
            self.world_camera_boundaries_mut()
                .set_overworld_scroll_counter_for_axis(which_axis, value);
            self.world_camera_boundaries_mut()
                .set_overworld_scroll_counter_for_axis(other_axis, 0u16.wrapping_sub(value));
            let mut r4 = x_vel as i16 as u16;
            self.set_overworld_horizontal_scroll_delta(r4);
            let oi = self.world_region().overlay_index();
            if oi != 0x97 && oi != 0x9d && r4 != 0 {
                let subp;
                if oi == 0x95 || oi == 0x9e {
                    subp = (r4 & 3) << 14;
                    r4 = ((r4 as i16) >> 2) as u16;
                } else {
                    subp = (r4 & 1) << 15;
                    r4 = ((r4 as i16) >> 1) as u16;
                }
                self.ppu_scroll_copy_view_mut()
                    .add_bg1_h_copy2_subpixel(subp, r4);
            }
        }
        if self.world_region().overlay_index() == 0x9c {
            self.ppu_scroll_copy_view_mut()
                .subtract_bg1_v_copy2_subpixel(0x2000, 0);
            let bg1v = self
                .world_scroll()
                .bg1_y()
                .wrapping_add(self.overworld_vertical_scroll_delta());
            self.world_scroll_mut().set_bg1_y(bg1v);
            self.ppu_scroll_copy_view_mut()
                .copy_bg2_h_live_to_bg1_h_live();
        } else if self.world_region().overlay_index() == 0x97
            || self.world_region().overlay_index() == 0x9d
        {
            self.ppu_scroll_copy_view_mut()
                .add_bg1_v_copy2_subpixel(0x2000, 0);
            self.ppu_scroll_copy_view_mut()
                .add_bg1_h_copy2_subpixel(0x2000, 0);
        }
        if self.world_location_state().dungeon_room == 0x181 {
            let bg2v = self.world_scroll().bg2_y() | 0x100;
            self.world_scroll_mut().set_bg1_y(bg2v);
            self.ppu_scroll_copy_view_mut()
                .copy_bg2_h_live_to_bg1_h_live();
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
        self.set_screen_brightness(128);
        self.palette_buffer_view_mut()
            .select_overworld_aux_palette_offset();
        self.palette_buffer_view_mut().set_hud_palette(1);
        self.palette_load_hud();
        self.system_signals_view_mut().increment_cgram_update_flag();
        self.save_progress_view_mut()
            .set_death_count_for_palace(4, 0);
        let palace_13 = self
            .save_progress_view()
            .death_count_for_palace(13)
            .wrapping_add(self.save_progress_view().pending_death_save_counter());
        self.save_progress_view_mut()
            .set_death_count_for_palace(13, palace_13);
        let mut sum = palace_13;
        for i in (0..=12).rev() {
            sum = sum.wrapping_add(self.save_progress_view().death_count_for_palace(i));
        }
        self.save_progress_view_mut()
            .set_total_death_save_counter(sum);
        self.save_progress_view_mut()
            .clear_pending_death_save_counter();
        let health =
            HEALTH_AFTER_DEATH[(self.player_resources_view().health_capacity() >> 3) as usize];
        self.player_resources_view_mut().set_current_health(health);
        self.save_progress_view_mut().set_dark_world_state(0x40);
        self.SaveGameFile();
        self.palette_buffer_view_mut().set_aux_color(38, 0);
        self.palette_buffer_view_mut().set_main_color(38, 0);
        self.palette_buffer_view_mut().set_aux_color(0, 0);
        self.palette_buffer_view_mut().set_main_color(0, 0);
        self.set_main_screen_layers(0x16);
        self.set_sub_screen_layers(0);
        self.ending_scratch_view_mut().set_primary_word(0x6800);
        self.ending_scratch_view_mut().set_secondary_word(0);
        self.clear_ending_palace_death_count_digit_step();
        self.world_scroll_mut().set_bg2_y((-0x48i16) as u16);
        self.world_scroll_mut().set_bg2_x(0x90);
        self.ppu_scroll_copy_view_mut().set_bg3_v_copy2(0);
        self.ppu_scroll_copy_view_mut().set_bg3_h_copy2(0);
        self.credits_add_next_attribution();
        self.system_signals_view_mut().set_music_control(0x22);
        self.palette_filter_view_mut().set_color_window_selection(0);
        self.palette_filter_view_mut().set_color_math_control(162);
        self.zelda_ppu_write(0x2108, 0x13);
        self.palette_filter_view_mut().set_fixed_color_red(0x3f);
        self.palette_filter_view_mut().set_fixed_color_green(0x5f);
        self.palette_filter_view_mut().set_fixed_color_blue(0x9f);
        self.set_subsubmodule(64);
        self.set_screen_brightness(0);
        self.hdma_setup(0, 0xebd53, 0x42, 0, BG2HOFS as u8, 0);
        self.set_hdma_enable_mask(0x80);
        self.ppu_scroll_copy_view_mut().copy_live_to_ppu_copy();
    }

    pub(super) fn credits_fade_out_fixed_col(&mut self) {
        self.decrement_subsubmodule();
        if self.frame_state().subsubmodule == 0 {
            self.set_subsubmodule(16);
            if self.palette_filter_view().fixed_color_red() != 32 {
                self.palette_filter_view_mut().subtract_fixed_color_red(1);
            } else if self.palette_filter_view().fixed_color_green() != 64 {
                self.palette_filter_view_mut().subtract_fixed_color_green(1);
            } else if self.palette_filter_view().fixed_color_blue() != 128 {
                self.palette_filter_view_mut().subtract_fixed_color_blue(1);
            }
        }
    }

    pub(super) fn credits_fade_color_and_begin_animating(&mut self) {
        self.credits_fade_out_fixed_col();
        self.set_core_update_disable_flag(1);
        self.credits_animate_the_triangles();
        if self.frame_state().frame_counter & 3 == 0 {
            let bg2 = self.world_scroll().bg2_x().wrapping_add(1);
            self.world_scroll_mut().set_bg2_x(bg2);
            if bg2 == 0x0c00 {
                self.zelda_ppu_write(0x2108, 0x13);
            }
            let a1 = bg2 >> 1;
            let a0 = a1.wrapping_add(bg2);
            self.room_bounds_view_mut()
                .set_packed_bounds(a0, a0 >> 1, a1, a1 >> 1);
            if self.ppu_scroll_copy_view().bg3_v_copy2() == 3288 {
                self.ending_scratch_view_mut().set_primary_word(0x80);
                self.increment_submodule();
            } else {
                self.ppu_scroll_copy_view_mut().add_bg3_v_copy2_signed(1);
                let bg3v = self.ppu_scroll_copy_view().bg3_v_copy2();
                if bg3v & 7 == 0 {
                    self.ending_scratch_view_mut().set_secondary_word(bg3v >> 3);
                    self.credits_add_next_attribution();
                }
            }
        }
        self.ppu_scroll_copy_view_mut().copy_live_to_ppu_copy();
    }

    pub(super) fn credits_add_next_attribution(&mut self) {
        const ATTRIBUTION_PALACE_ORDER: [usize; 14] =
            [1, 0, 2, 3, 10, 6, 5, 8, 11, 9, 7, 12, 13, 15];
        const DIGITS_SCROLL_Y: [u16; 14] = [
            0x290, 0x298, 0x2a0, 0x2a8, 0x2b0, 0x2ba, 0x2c2, 0x2ca, 0x2d2, 0x2da, 0x2e2, 0x2ea,
            0x2f2, 0x310,
        ];
        const DIGIT_CHAR: [u16; 2] = [0x3ce6, 0x3cf6];
        let mut dst = self.display_state().current_vram_upload_data_address();
        let mut r16 = self.ending_scratch_view().primary_word();

        self.write_vram_upload_absolute_word(dst, r16.swap_bytes());
        self.write_vram_upload_absolute_word(dst + 2, 0x3e40);
        let blank_tile = self.ending_asset_u16(76, 159);
        self.write_vram_upload_absolute_word(dst + 4, blank_tile);
        dst += 6;

        let r18 = self.ending_scratch_view().secondary_word() as usize;
        if r18 < 394 {
            let text_off = self.ending_asset_u16(75, r18) as usize;
            let text = self
                .asset_raw(74)
                .unwrap_or_else(|| panic!("missing ending asset 74"))
                .to_vec();
            if text[text_off] != 0xff {
                let addr_delta = text[text_off] as u16;
                let n = text[text_off + 1];
                self.write_vram_upload_absolute_word(
                    dst,
                    r16.wrapping_add(addr_delta).swap_bytes(),
                );
                self.write_vram_upload_absolute_word(dst + 2, (n as u16).swap_bytes());
                dst += 4;
                let count = ((n.wrapping_add(1)) >> 1) as usize;
                for q in 0..count {
                    let ch = text[text_off + 2 + q] as usize;
                    let tile = self.ending_asset_u16(76, ch);
                    self.write_vram_upload_absolute_word(dst, tile);
                    dst += 2;
                }
            }

            let credits = self.ending_credit_state();
            let which_idx = credits.palace_death_count_index();
            if credits.should_write_digit_for_scroll_y(
                (r18 as u16).wrapping_mul(2),
                DIGITS_SCROLL_Y[which_idx],
            ) {
                let t = DIGIT_CHAR[credits.digit_tile_base_index()];
                self.set_ending_death_count_digit_tile_base(t);
                self.write_vram_upload_absolute_word(dst, r16.wrapping_add(0x19).swap_bytes());
                self.write_vram_upload_absolute_word(dst + 2, 0x0500);
                let palace = ATTRIBUTION_PALACE_ORDER[which_idx];
                let mut deaths = self.save_progress_view().death_count_for_palace(palace);
                if deaths >= 1000 {
                    deaths = 999;
                }
                self.write_vram_upload_absolute_word(dst + 8, t.wrapping_add(deaths % 10));
                deaths /= 10;
                self.write_vram_upload_absolute_word(dst + 6, t.wrapping_add(deaths % 10));
                deaths /= 10;
                self.write_vram_upload_absolute_word(dst + 4, t.wrapping_add(deaths));
                dst += 10;
                self.advance_ending_palace_death_count_digit_step();
            }
        }

        r16 = r16.wrapping_add(0x20);
        if r16 & 0x3ff == 0 {
            r16 = (r16 & 0x6800) ^ 0x800;
        }
        self.ending_scratch_view_mut().set_primary_word(r16);
        let upload_base = self.display_state().vram_upload_buffer_base();
        self.set_vram_upload_cursor((dst - upload_base) as u16);
        self.write_vram_upload_absolute_byte(dst, 0xff);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn credits_add_ending_sequence_text(&mut self) {
        let mut dst = self.display_state().vram_upload_buffer_base();
        self.write_vram_upload_absolute_word(dst, 0x0060);
        self.write_vram_upload_absolute_word(dst + 2, 0xfe47);
        let blank_tile = self.ending_asset_u16(76, 159);
        self.write_vram_upload_absolute_word(dst + 4, blank_tile);
        dst += 6;

        let scene = (self.frame_state().submodule >> 1) as usize;
        let mut curo = self.ending_asset_u16(77, scene) as usize;
        let endo = self.ending_asset_u16(77, scene + 1) as usize;
        let data = self
            .asset_raw(78)
            .unwrap_or_else(|| panic!("missing ending asset 78"))
            .to_vec();
        while curo != endo {
            let a = u16::from_le_bytes([data[curo], data[curo + 1]]);
            let b = u16::from_le_bytes([data[curo + 2], data[curo + 3]]);
            self.write_vram_upload_absolute_word(dst, a);
            self.write_vram_upload_absolute_word(dst + 2, b);
            let m = ((b >> 9) & 0x7f) as usize;
            dst += 4;
            curo += 4;
            for _ in 0..=m {
                let ch = data[curo] as usize;
                let tile = self.ending_asset_u16(76, ch);
                self.write_vram_upload_absolute_word(dst, tile);
                dst += 2;
                curo += 1;
            }
        }
        let upload_base = self.display_state().vram_upload_buffer_base();
        self.set_vram_upload_cursor((dst - upload_base) as u16);
        self.write_vram_upload_absolute_byte(dst, 0xff);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn credits_brighten_triangles(&mut self) {
        if self.frame_state().frame_counter & 15 == 0 {
            self.increment_screen_brightness();
            if self.display_state().screen_brightness == 15 {
                self.increment_submodule();
            }
        }
        self.credits_animate_the_triangles();
    }

    pub(super) fn credits_stop_credits_scroll(&mut self) {
        if self.ending_scratch_view_mut().decrement_primary_low() == 0 {
            self.palette_filter_view_mut()
                .set_darkening_or_lightening_screen_word(0);
            self.palette_filter_view_mut().set_countdown_word(0);
            self.set_mosaic_target_level_word(0x1f);
            self.increment_submodule();
            self.ending_scratch_view_mut().set_primary_word(0x00c0);
            self.ending_scratch_view_mut().set_secondary_word(0);
        }
        self.credits_animate_the_triangles();
    }

    pub(super) fn credits_fade_and_disperse_triangles(&mut self) {
        self.ending_scratch_view_mut().decrement_primary_low();
        if self.ending_scratch_view().secondary_low() == 0 {
            self.apply_palette_filter_bounce();
            if self.palette_filter_view().countdown() != 0 {
                self.credits_animate_the_triangles();
                return;
            }
            self.ending_scratch_view_mut().increment_secondary_low();
        }
        if self.ending_scratch_view().primary_low() != 0 {
            self.credits_animate_the_triangles();
            return;
        }
        self.increment_submodule();
        self.PaletteFilter_WishPonds_Inner();
    }

    pub(super) fn credits_fade_in_the_end(&mut self) {
        if self.frame_state().frame_counter & 7 == 0 {
            self.PaletteFilter_SP5F();
            if self.palette_filter_view().countdown() == 0 {
                self.increment_submodule();
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
        self.poly_runtime_mut().set_config1(156);
        self.poly_runtime_mut().set_color_mode(1);
        self.activate_nmi_thread();
        self.attract_scene_mut().mark_intro_did_run_step();
        self.poly_runtime_mut().set_base_x(32);
        self.poly_runtime_mut().set_base_y(32);
        self.poly_runtime_mut().set_shape_depth_bias_low(32);
        self.poly_runtime_mut().set_model(0);
        self.poly_runtime_mut().set_angle_a(16);
        self.set_sub_screen_layers(0);
        self.set_main_screen_layers(0x16);
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

fn break_triforce_handle_poly(state: &mut ZeldaState) {
    state.attract_scene_mut().mark_intro_did_run_step();
    state.resume_intro_triangle_motion();
    state.attract_scene_mut().increment_intro_frame_counter();
}

impl ZeldaState {
    pub(super) fn fade_music_and_reset_sram_mirror(&mut self) {
        self.set_irq_control_flag(0xff);
        self.set_main_screen_layers(0x15);
        self.set_sub_screen_layers(0);
        self.set_indoor_flag(0);
        self.system_signals_view_mut().set_music_control(0xf1);
        self.set_backdrop_color_black();
        self.player_state_view_mut()
            .clear_link_state_block_for_ending();
        self.save_progress_view_mut().clear_dungeon_info();
        self.set_main_module(1);
        self.system_signals_view_mut().set_restart_check_flag(1);
        self.set_submodule(0);
    }

    pub(super) fn load_triforce_sprite_palette(&mut self) {
        const POLYHEDRAL_PALETTE: [u16; 8] =
            [0, 0x014d, 0x01b0, 0x01f3, 0x0256, 0x0279, 0x02fd, 0x035f];
        for (i, color) in POLYHEDRAL_PALETTE.iter().enumerate() {
            self.palette_buffer_view_mut()
                .set_main_color(0xd0 + i, *color);
        }
        self.system_signals_view_mut().increment_cgram_update_flag();
    }

    pub(super) fn module00_intro(&mut self) {
        let skip_at = if self
            .enhanced_features_view()
            .has(FEATURES0_SKIP_INTRO_ON_KEYPRESS)
        {
            4
        } else {
            8
        };
        if self.frame_state().submodule >= skip_at
            && (((self.player_state_view().filtered_joypad_l() & 0xc0)
                | self.player_state_view().filtered_joypad_h())
                & 0xd0)
                != 0
        {
            self.fade_music_and_reset_sram_mirror();
            return;
        }

        match self.frame_state().submodule {
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
        if self.frame_state().frame_counter & 1 == 0 {
            return;
        }
        self.palette_fade_intro_one_step();
        if self.palette_filter_view().countdown() == 0 {
            self.set_subsubmodule(42);
            self.increment_submodule();
            self.intro_setup_sword_and_intro_flash();
        } else if self.palette_filter_view().countdown() == 13 {
            self.set_main_screen_layers(0x15);
            self.set_sub_screen_layers(0);
        }
    }

    pub(super) fn intro_setup_sword_and_intro_flash(&mut self) {
        self.intro_sword_view_mut().reset_sword_state();
        self.intro_periodic_sword_and_intro_flash();
    }

    pub(super) fn intro_sword_coming_down(&mut self) {
        self.intro_handle_all_triforce_animations();
        self.attract_scene_mut().clear_intro_did_run_step();
        self.deactivate_nmi_thread();
        self.intro_periodic_sword_and_intro_flash();
        self.decrement_subsubmodule();
        if self.frame_state().subsubmodule == 0 {
            self.increment_submodule();
            self.palette_filter_view_mut().set_color_window_selection(2);
            self.palette_filter_view_mut().set_color_math_control(0x22);
            self.palette_filter_view_mut().set_countdown_word(31);
            self.set_sub_screen_layers(2);
        }
    }

    pub(super) fn intro_fade_in_bg(&mut self) {
        self.intro_periodic_sword_and_intro_flash();
        self.intro_handle_all_triforce_animations();
        if self.palette_filter_view().countdown() != 0 {
            if self.frame_state().frame_counter & 1 != 0 {
                self.palette_fade_intro2();
            }
        } else if (((self.player_state_view().filtered_joypad_l() & 0xc0)
            | self.player_state_view().filtered_joypad_h())
            & 0xd0)
            != 0
        {
            self.fade_music_and_reset_sram_mirror();
        } else {
            self.decrement_subsubmodule();
            if self.frame_state().subsubmodule == 0 {
                self.increment_submodule();
            }
        }
    }

    pub(super) fn intro_wait_player(&mut self) {
        self.intro_handle_all_triforce_animations();
        self.attract_scene_mut().clear_intro_did_run_step();
        self.deactivate_nmi_thread();
        self.intro_periodic_sword_and_intro_flash();
        self.decrement_subsubmodule();
        if self.frame_state().subsubmodule == 0 {
            self.increment_submodule();
            self.set_main_module(20);
            self.set_submodule(0);
            self.player_state_view_mut().set_x(0);
        }
    }

    pub(super) fn intro_periodic_sword_and_intro_flash(&mut self) {
        if self.intro_sword_view().sparkle_timer() != 0 {
            self.intro_sword_view_mut().decrement_sparkle_timer();
        }
        self.set_backdrop_color_black();
        if self.attract_scene().intro_palette_flash_count() != 0 {
            if self.attract_scene().intro_palette_flash_count() & 3 != 0 {
                let flash = if self
                    .enhanced_features_view()
                    .has(FEATURE_DIM_ENDING_FLASHES)
                {
                    0x05
                } else {
                    0x1f
                };
                let flash_component = self.intro_sword_view().flash_rgb_channel();
                self.palette_filter_view_mut()
                    .or_fixed_color_component(flash_component, flash);
                self.intro_sword_view_mut().cycle_flash_rgb_channel();
            }
            self.attract_scene_mut()
                .decrement_intro_palette_flash_count();
        }

        const CHARS: [u8; 10] = [0, 2, 0x20, 0x22, 4, 6, 8, 0x0a, 0x0c, 0x0e];
        const XS: [u8; 10] = [0x40, 0x40, 0x30, 0x50, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40];
        const YS: [u16; 10] = [0x10, 0x20, 0x28, 0x28, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80];
        let sword_y = self.intro_sword_view().ypos();
        for j in (0..10).rev() {
            let y = sword_y.wrapping_add(YS[j]);
            let visible_y = if y & 0xff00 != 0 { 0xf8u8 } else { y as u8 }.wrapping_sub(8);
            self.set_oam_plain(0x52 + j, XS[j], visible_y, CHARS[j], 0x21, 2);
        }

        if sword_y != 30 {
            if sword_y == 0xffbe {
                self.system_signals_view_mut().set_sound_effect_1(1);
            } else if sword_y == 14 {
                self.intro_sword_view_mut().set_flash_rgb_channel_word(0);
                self.attract_scene_mut().set_intro_palette_flash_count(0x20);
                self.system_signals_view_mut().set_sound_effect_1(0x2c);
            }
            self.intro_sword_view_mut()
                .set_ypos(sword_y.wrapping_add(16));
        }

        match self.intro_sword_view().anim_phase() {
            0 => {
                if self.attract_scene().intro_palette_flash_count() == 0
                    && self.intro_sword_view().ypos() == 30
                {
                    self.intro_sword_view_mut().advance_anim_step();
                }
            }
            1 => {
                const INTRO_SWORD_SPARKLE_TIMERS: [u8; 8] = [4, 4, 6, 6, 6, 4, 4, 0];
                const SPARKLE_CHARS: [u8; 7] = [0x28, 0x37, 0x27, 0x36, 0x27, 0x37, 0x28];
                if self.intro_sword_view().sparkle_timer() == 0 {
                    if self
                        .intro_sword_view_mut()
                        .decrement_sparkle_step_check_negative()
                    {
                        self.intro_sword_view_mut().set_sparkle_step(0);
                        self.intro_sword_view_mut().set_sparkle_timer(2);
                        self.intro_sword_view_mut().advance_anim_step();
                        return;
                    }
                    let timer =
                        INTRO_SWORD_SPARKLE_TIMERS[self.intro_sword_view().sparkle_step() as usize];
                    self.intro_sword_view_mut().set_sparkle_timer(timer);
                }
                self.set_oam_plain(
                    0x50,
                    0x44,
                    0x43,
                    SPARKLE_CHARS[self.intro_sword_view().sparkle_step() as usize],
                    0x25,
                    0,
                );
            }
            2 => {
                const SPARKLE_CHARS: [u8; 8] = [0x26, 0x20, 0x24, 0x34, 0x25, 0x20, 0x35, 0x20];
                let k = self.intro_sword_view().sparkle_step() as usize;
                if k >= 7 {
                    return;
                }
                let y_base = self.intro_sword_view().sparkle_y_offset().min(0x4f);
                let y = y_base
                    .wrapping_add(self.intro_sword_view().ypos() as u8)
                    .wrapping_add(0x31);
                self.set_oam_plain(0x50, 0x42, y, SPARKLE_CHARS[k], 0x23, 0);
                self.set_oam_plain(0x51, 0x42, y.wrapping_add(8), SPARKLE_CHARS[k + 1], 0x23, 0);
                if self.intro_sword_view().sparkle_timer() == 0 {
                    self.intro_sword_view_mut().advance_sparkle_y_offset();
                    if matches!(
                        self.intro_sword_view().sparkle_y_offset(),
                        0x04 | 0x48 | 0x4c | 0x58
                    ) {
                        let step = self.intro_sword_view().sparkle_step().wrapping_add(2);
                        self.intro_sword_view_mut().set_sparkle_step(step);
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn intro_init(&mut self) {
        self.intro_setup_screen();
        self.set_screen_brightness(15);
        self.set_subsubmodule(0);
        self.intro_startup_delay = 0;
        self.system_signals_view_mut().increment_cgram_update_flag();
        self.increment_submodule();
        self.system_signals_view_mut().set_sound_effect_2(10);
        self.intro_init_continue();
    }

    pub(super) fn intro_setup_screen(&mut self) {
        self.set_core_update_disable_flag(0x80);
        self.enable_force_blank();
        self.set_main_screen_layers(16);
        self.set_sub_screen_layers(0);
        self.intro_initialize_background_settings();
        self.palette_filter_view_mut()
            .set_color_window_selection(0x20);
        self.set_chr_halfslot_request(20);
        self.graphics_load_chr_half_slot();
        self.clear_chr_halfslot_request();
        self.LoadOWMusicIfNeeded();

        for i in 0..17 {
            self.palette_buffer_view_mut()
                .set_main_color(144 + i, 0x7fff);
            self.ppu.vram[0x27f0 + i] = 0;
        }

        self.ending_scratch_view_mut().set_primary_word(0x1ffe);
        self.ending_scratch_view_mut().set_secondary_word(0x1bfe);
    }

    pub(super) fn intro_initialize_background_settings(&mut self) {
        self.set_bg_mode(9);
        self.set_mosaic_copy(0);
        self.zelda_ppu_write(0x2107, 0x13);
        self.zelda_ppu_write(0x2108, 0x03);
        self.zelda_ppu_write(0x2109, 0x63);
        self.palette_filter_view_mut().set_color_math_control(32);
        self.palette_filter_view_mut().set_fixed_color_red(32);
        self.palette_filter_view_mut().set_fixed_color_green(64);
        self.palette_filter_view_mut().set_fixed_color_blue(128);
    }

    pub(super) fn intro_init_continue(&mut self) {
        self.intro_display_logo();
        let t = self.frame_state().subsubmodule;
        self.increment_subsubmodule();
        match t {
            0..=7 => self.intro_clear1kb_blocks_of_wram(),
            8 => self.intro_load_text_pointers_and_palettes(),
            9 => self.load_item_gfx_into_wram_4bpp_buffer(),
            10 => self.load_follower_graphics(),
            _ => {
                self.decrement_screen_brightness();
                if self.display_state().screen_brightness == 0 {
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
        self.sprite_system_view_mut().set_main_tile_theme(35);
        self.sprite_system_view_mut().set_graphics_index(125);
        self.sprite_system_view_mut().set_aux_tile_theme(81);
        self.sprite_system_view_mut()
            .set_misc_sprites_graphics_index(8);
        self.load_default_graphics();
        self.initialize_tilesets();
        self.decompress_animated_dungeon_tiles(0x5d);
        self.palette_buffer_view_mut()
            .set_bg_tile_animation_countdown(2);
        self.set_overworld_screen(0);
        self.palette_buffer_view_mut().set_palette_main_indoors(0);
        self.palette_buffer_view_mut()
            .set_overworld_palette_aux3_lo(0);
        self.ending_scratch_view_mut().set_primary_word(0);
        self.ending_scratch_view_mut().set_secondary_word(0);
        self.palette_filter_view_mut()
            .set_darkening_or_lightening_screen_word(2);
        self.palette_filter_view_mut().set_countdown_word(31);
        self.clear_mosaic_target_level();
        self.increment_submodule();
    }

    pub(super) fn intro_initialize_triforce_poly_thread(&mut self) {
        self.sprite_system_view_mut()
            .set_misc_sprites_graphics_index(8);
        self.load_common_sprites();
        self.intro_init_gfx_helper();
        self.intro_actor_view_mut(0).set_init_phase(1);
        self.intro_actor_view_mut(1).set_init_phase(1);
        self.intro_actor_view_mut(2).set_init_phase(1);
        self.intro_actor_view_mut(0).set_subtype(0);
        self.intro_actor_view_mut(1).set_subtype(0);
        self.intro_actor_view_mut(2).set_subtype(0);
        self.intro_actor_view_mut(4).set_init_phase(1);
        self.intro_actor_view_mut(4).set_subtype(2);
        self.set_screen_brightness(15);
        self.increment_submodule();
    }

    pub(super) fn intro_init_gfx_helper(&mut self) {
        self.polyhedral_initialize_thread();
        self.load_triforce_sprite_palette();
        self.set_vertical_irq_trigger(0x90);
        self.poly_runtime_mut().set_config1(0xff);
        self.poly_runtime_mut().set_base_x(32);
        self.poly_runtime_mut().set_base_y(32);
        self.poly_runtime_mut().set_shape_depth_bias_low(32);
        self.poly_runtime_mut().set_angle_a(0xa0);
        self.poly_runtime_mut().set_angle_b(0x60);
        self.poly_runtime_mut().set_color_mode(1);
        self.poly_runtime_mut().set_model(1);
        self.activate_nmi_thread();
        self.attract_scene_mut().mark_intro_did_run_step();
        if self.rom_startup_timing() {
            self.intro_poly_upload_delay = configured_intro_thread_start_delay();
            self.intro_sprite_animation_start_delay =
                configured_intro_sprite_animation_start_delay();
        }
        self.attract_scene_mut().clear_intro_step_state_block();
        if self.rom_startup_timing() {
            for _ in 0..configured_intro_poly_bootstrap_steps() {
                self.intro_run_step();
            }
        }
    }

    pub(super) fn polyhedral_initialize_thread(&mut self) {
        const POLY_THREAD_INIT: [u8; 13] = [9, 0, 0x1f, 0, 0, 0, 0, 0, 0, 0x30, 0x1d, 0xf8, 9];
        self.fill_ram(POLY_THREAD_RAM_START, POLY_THREAD_RAM_LEN, 0);
        self.set_nmi_thread_stack_pointer(0x1f31);
        self.copy_to_ram(POLY_THREAD_INIT_BYTES, &POLY_THREAD_INIT);
    }

    pub(super) fn intro_handle_all_triforce_animations(&mut self) {
        if self.rom_startup_timing() && self.intro_sprite_animation_start_delay != 0 {
            self.intro_sprite_animation_start_delay =
                self.intro_sprite_animation_start_delay.saturating_sub(1);
            self.intro_animate_triforce();
            return;
        }
        self.attract_scene_mut().increment_intro_frame_counter();
        self.intro_animate_triforce();
        self.scene_animate_every_sprite();
    }

    pub(super) fn intro_animate_triforce(&mut self) {
        self.activate_nmi_thread();
        if self.rom_startup_timing() && self.intro_memory_darken_frame_delay == 0 {
            if self.intro_poly_upload_delay != 0 {
                self.intro_poly_upload_delay = self.intro_poly_upload_delay.saturating_sub(1);
                self.attract_scene_mut().mark_intro_did_run_step();
                return;
            }
            if self.bsnes_hold_intro_step_this_frame {
                self.attract_scene_mut().mark_intro_did_run_step();
                return;
            }
            self.intro_run_step();
            self.attract_scene_mut().mark_intro_did_run_step();
            return;
        }
        if self.attract_scene().intro_did_run_step() == 0 {
            self.intro_run_step();
            self.attract_scene_mut().mark_intro_did_run_step();
        }
    }

    pub(super) fn intro_run_step(&mut self) {
        match self.attract_scene().intro_step_index() {
            0 => {
                self.attract_scene_mut().increment_intro_step_timer();
                if self.attract_scene().intro_step_timer() == 64 {
                    self.attract_scene_mut().increment_intro_step_index();
                }
                self.poly_runtime_mut().add_angle_b(5);
                self.poly_runtime_mut().add_angle_a(3);
            }
            1 => {
                if self.poly_runtime().config1() < 2 {
                    self.poly_runtime_mut().clear_config1();
                    self.attract_scene_mut().increment_intro_step_index();
                    self.attract_scene_mut().set_intro_step_timer(64);
                    return;
                }
                self.poly_runtime_mut().subtract_config1(2);
                self.poly_runtime_mut().add_angle_b(5);
                self.poly_runtime_mut().add_angle_a(3);
                if self.poly_runtime().config1() < 225 {
                    self.set_submodule(4);
                }
                if self.poly_runtime().config1() == 113 {
                    self.system_signals_view_mut().set_music_control(1);
                }
            }
            2 => {
                self.attract_scene_mut().decrement_intro_step_timer();
                if self.attract_scene().intro_step_timer() == 0 {
                    self.attract_scene_mut().increment_intro_step_index();
                } else {
                    self.poly_runtime_mut().add_angle_b(5);
                    self.poly_runtime_mut().add_angle_a(3);
                }
            }
            3 => {
                if self.poly_runtime().angle_b() >= 250 && self.poly_runtime().angle_a() >= 252 {
                    self.attract_scene_mut().increment_intro_step_index();
                    self.attract_scene_mut().set_intro_step_timer(32);
                } else {
                    self.poly_runtime_mut().add_angle_b(5);
                    self.poly_runtime_mut().add_angle_a(3);
                }
            }
            4 => {
                self.poly_runtime_mut().set_angle_b(0);
                self.poly_runtime_mut().set_angle_a(0);
                self.attract_scene_mut().decrement_intro_step_timer();
                if self.attract_scene().intro_step_timer() == 0 {
                    self.attract_scene_mut().increment_intro_step_index();
                    self.intro_actor_view_mut(5).set_init_phase(1);
                    self.intro_actor_view_mut(5).set_subtype(3);
                    self.set_main_screen_layers(0x10);
                    self.set_sub_screen_layers(5);
                    self.palette_filter_view_mut().set_color_window_selection(2);
                    self.palette_filter_view_mut().set_color_math_control(0x31);
                    self.set_subsubmodule(0);
                    self.system_signals_view_mut().increment_cgram_update_flag();
                    self.set_bg_vram_load_mode(3);
                    self.increment_submodule();
                }
            }
            _ => {}
        }
    }

    pub(super) fn scene_animate_every_sprite(&mut self) {
        self.reset_intro_sprite_oam_cursor();
        for k in (0..8).rev() {
            self.intro_anim_one_obj(k);
        }
    }

    pub(super) fn intro_anim_one_obj(&mut self, k: usize) {
        match self.intro_actor_view(k).init_phase() {
            1 => match self.intro_actor_view(k).subtype() {
                0 => self.intro_sprite_type_a_0(k),
                1 => self.exit_0_cca90(k),
                2 => self.initialize_scene_sprite_copyright(k),
                3 => self.initialize_scene_sprite_sparkle(k),
                4 | 5 | 6 => self.initialize_scene_sprite_triforce_room_triangle(k),
                7 => self.initialize_scene_sprite_credits_triangle(k),
                _ => {}
            },
            2 => match self.intro_actor_view(k).subtype() {
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
        const LOCAL_X_VELOCITIES: [i8; 3] = [1, 0, -1];
        const LOCAL_Y_VELOCITIES: [i8; 3] = [-1, 1, -1];
        self.write_intro_x(k, X[k]);
        self.write_intro_y(k, Y[k]);
        self.intro_actor_view_mut(k)
            .set_x_velocity(LOCAL_X_VELOCITIES[k] as u8);
        self.intro_actor_view_mut(k)
            .set_y_velocity(LOCAL_Y_VELOCITIES[k] as u8);
        self.intro_actor_view_mut(k).increment_init_phase();
    }

    pub(super) fn initialize_scene_sprite_copyright(&mut self, k: usize) {
        self.write_intro_x(k, 76);
        self.write_intro_y(k, 184);
        self.intro_actor_view_mut(k).increment_init_phase();
    }

    pub(super) fn intro_sprite_type_b_0(&mut self, k: usize) {
        self.animate_scene_sprite_draw_triangle(k);
        self.animate_scene_sprite_move_triangle(k);
        if self.attract_scene().intro_step_index() != 5 {
            if self.attract_scene().intro_frame_counter() & 31 == 0 {
                const LOCAL_X_VELOCITIES: [i8; 3] = [1, 0, -1];
                const LOCAL_Y_VELOCITIES: [i8; 3] = [-1, 1, -1];
                self.intro_actor_view_mut(k)
                    .add_x_velocity(LOCAL_X_VELOCITIES[k] as u8);
                self.intro_actor_view_mut(k)
                    .add_y_velocity(LOCAL_Y_VELOCITIES[k] as u8);
            }
            const X_LIMIT: [u8; 3] = [75, 95, 117];
            const Y_LIMIT: [u8; 3] = [88, 48, 88];
            if self.intro_actor_view(k).x_low() == X_LIMIT[k] {
                self.intro_actor_view_mut(k).set_x_velocity(0);
            }
            if self.intro_actor_view(k).y_low() == Y_LIMIT[k] {
                self.intro_actor_view_mut(k).set_y_velocity(0);
            }
        } else {
            self.intro_actor_view_mut(k).set_x_velocity(0);
            self.intro_actor_view_mut(k).set_y_velocity(0);
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
        let j = (self.attract_scene().intro_frame_counter() >> 5 & 3) as usize;
        self.intro_actor_view_mut(k).set_x(i16::from(X[j]));
        self.intro_actor_view_mut(k).set_y(i16::from(Y[j]));
        self.intro_actor_view_mut(k).increment_init_phase();
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

        let state = self.intro_actor_view(k).state();
        if state < 4 {
            self.animate_scene_sprite_add_objects_to_oam_buffer(
                k,
                &ENTS[state as usize..state as usize + 1],
            );
        }

        let next_state = STATE[(self.attract_scene().intro_frame_counter() >> 2 & 7) as usize];
        self.intro_actor_view_mut(k).set_state(next_state);
        let j = (self.attract_scene().intro_frame_counter() >> 5 & 3) as usize;
        self.intro_actor_view_mut(k).set_x_low(X[j]);
        self.intro_actor_view_mut(k).set_y_low(Y[j]);
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
        if self.intro_actor_view(k).x_velocity() != 0 {
            self.intro_actor_view_mut(k).move_x();
        }
        if self.intro_actor_view(k).y_velocity() != 0 {
            self.intro_actor_view_mut(k).move_y();
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
