pub(super) const POLYHEDRAL_PALETTE: [u16; 8] =
    [0, 0x14d, 0x1b0, 0x1f3, 0x256, 0x279, 0x2fd, 0x35f];
pub(super) const FEATURE_DIM_ENDING_FLASHES: u32 = 65536;
pub(super) const ENDING_SCENE_ENTRANCES: [u16; 16] = [
    0x1000, 2, 0x1002, 0x1012, 0x1004, 0x1006, 0x1010, 0x1014, 0x100a, 0x1016, 0x5d, 0x64, 0x100e,
    0x1008, 0x1018, 0x180,
];
pub(super) const ENDING_SPRITE_PACKS: [u8; 17] = [
    0x28, 0x46, 0x27, 0x2e, 0x2b, 0x2b, 0xe, 0x2c, 0x1a, 0x29, 0x47, 0x28, 0x27, 0x28, 0x2a, 0x28,
    0x2d,
];
pub(super) const ENDING_SPRITE_PALETTES: [u8; 17] = [
    1, 0x40, 1, 4, 1, 1, 1, 0x11, 1, 1, 0x47, 0x40, 1, 1, 1, 1, 1,
];
pub(super) const ENDING_SCENE_SCROLL_TARGET_Y: [u16; 16] = [
    0x6f2, 0x210, 0x72c, 0xc00, 0x10c, 0xa9b, 0x10, 0x510, 0x89, 0xa8e, 0x222c, 0x2510, 0x826,
    0x5c, 0x20a, 0x30,
];
pub(super) const ENDING_SCENE_SCROLL_TARGET_X: [u16; 16] = [
    0x77f, 0x480, 0x193, 0xaa, 0x878, 0x847, 0x4fd, 0xc57, 0x40f, 0x478, 0xa00, 0x200, 0x201,
    0xaa1, 0x26f, 0,
];
pub(super) const ENDING_SCENE_SCROLL_Y_VELOCITIES: [i8; 16] =
    [-1, -1, 1, -1, 1, 1, 0, 1, 0, -1, -1, 0, 0, 0, 1, -1];
pub(super) const ENDING_SCENE_SCROLL_X_VELOCITIES: [i8; 16] =
    [0, 0, -1, 0, 0, -1, 1, 0, -1, 0, 0, 0, 1, -1, 1, 0];
pub(super) const BG2HOFS: usize = 0x210f;
pub(super) const OVERWORLD_SCROLL_UP_COUNTER: usize = 0x624;
pub(super) const OVERWORLD_SCROLL_DOWN_COUNTER: usize = 0x626;
pub(super) const OVERWORLD_SCROLL_LEFT_COUNTER: usize = 0x628;
pub(super) const OVERWORLD_SCROLL_RIGHT_COUNTER: usize = 0x62a;
pub(super) type IntroSpriteEnt = (i8, i8, u8, u8, u8);

pub(super) const ENDING_SPRITE_X_OFFSETS: [u16; 85] = [
    0x1e0, 0x200, 0x1ed, 0x203, 0x1da, 0x216, 0x1c8, 0x228, 0x1c0, 0x1e0, 0x208, 0x228, 0xf8, 0xf0,
    0x278, 0x298, 0x1e0, 0x200, 0x220, 0x288, 0x1e2, 0xe0, 0x150, 0xe8, 0x168, 0x128, 0x170, 0x170,
    0x335, 0x335, 0x300, 0xb8, 0xce, 0xac, 0xc4, 0x3b0, 0x390, 0x3d0, 0xf8, 0xc8, 0x80, 0xf8, 0xf8,
    0xf8, 0xf8, 0xf8, 0xe8, 0xf8, 0xd8, 0xf8, 0xc8, 0x108, 0x70, 0x70, 0x70, 0x68, 0x88, 0x70,
    0x40, 0x70, 0x4f, 0x61, 0x37, 0x79, 0xc8, 0x278, 0x258, 0x1d8, 0x1c8, 0x188, 0x270, 0x180,
    0x2e8, 0x270, 0x270, 0x2a0, 0x2a0, 0x2a4, 0x2fc, 0x76, 0x73, 0x76, 0x0, 0xd0, 0x80,
];
pub(super) const ENDING_SPRITE_Y_OFFSETS: [u16; 85] = [
    0x158, 0x158, 0x138, 0x138, 0x140, 0x140, 0x150, 0x150, 0x120, 0x120, 0x120, 0x120, 0x60, 0x37,
    0xc2, 0xc2, 0x16b, 0x16c, 0x16b, 0xb8, 0x16b, 0x80, 0x60, 0x146, 0x146, 0x1c6, 0x70, 0x70,
    0x128, 0x128, 0x16f, 0xf5, 0xfc, 0x10d, 0x10d, 0x40, 0x40, 0x40, 0x150, 0x158, 0xf4, 0x120,
    0x120, 0x120, 0x120, 0x120, 0x108, 0x100, 0xd8, 0xd8, 0xf0, 0xf0, 0x3c, 0x3c, 0x3c, 0x90, 0x80,
    0x3c, 0x16c, 0x16c, 0x174, 0x174, 0x175, 0x175, 0x250, 0x2b0, 0x2b0, 0x2a0, 0x2b0, 0x2b0,
    0x2b8, 0xd8, 0x24b, 0x1b0, 0x1c8, 0x1c8, 0x1b0, 0x230, 0x230, 0x8b, 0x83, 0x85, 0x2c, 0xf8,
    0x100,
];
pub(super) const ENDING_SCENE_SPRITE_RANGES: [usize; 17] = [
    0, 12, 14, 21, 28, 31, 35, 38, 40, 41, 52, 58, 64, 71, 72, 79, 85,
];

pub(super) type DrawMultipleDataEnding = (i8, i8, u16, u8);

pub(super) const END_SEQUENCE_DRAW_FRAMES0: [DrawMultipleDataEnding; 12] = [
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

pub(super) const END_SEQUENCE_DRAW_FRAMES1: [DrawMultipleDataEnding; 6] = [
    (14, -7, 0x0d48, 2),
    (0, -6, 0x0944, 2),
    (0, 0, 0x094e, 2),
    (13, -14, 0x0d48, 2),
    (0, -8, 0x0944, 2),
    (0, 0, 0x0946, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES2: [DrawMultipleDataEnding; 16] = [
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

pub(super) const END_SEQUENCE_DRAW_FRAMES3: [DrawMultipleDataEnding; 12] = [
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

pub(super) const END_SEQUENCE_DRAW_FRAMES4: [DrawMultipleDataEnding; 8] = [
    (10, 8, 0x8a32, 0),
    (10, 16, 0x8a22, 0),
    (0, -10, 0x0800, 2),
    (0, 0, 0x082c, 2),
    (10, -14, 0x0a22, 0),
    (10, -6, 0x0a32, 0),
    (0, -10, 0x082a, 2),
    (0, 0, 0x0828, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES5: [DrawMultipleDataEnding; 10] = [
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

pub(super) const END_SEQUENCE_DRAW_FRAMES6: [DrawMultipleDataEnding; 3] =
    [(-6, -2, 0x0706, 2), (0, -9, 0x090e, 2), (0, -1, 0x0908, 2)];

pub(super) const END_SEQUENCE_DRAW_FRAMES7: [DrawMultipleDataEnding; 10] = [
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

pub(super) const END_SEQUENCE_DRAW_FRAMES8: [DrawMultipleDataEnding; 1] = [(0, -19, 0x39af, 0)];

pub(super) const END_SEQUENCE_DRAW_FRAMES9: [DrawMultipleDataEnding; 4] = [
    (-16, -24, 0x3704, 2),
    (-16, -16, 0x3764, 2),
    (-16, -24, 0x3762, 2),
    (-16, -16, 0x3764, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES10: [DrawMultipleDataEnding; 4] = [
    (0, 0, 0x0c0c, 2),
    (0, 0, 0x0c0a, 2),
    (0, 0, 0x0cc5, 2),
    (0, 0, 0x0ce1, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES11: [DrawMultipleDataEnding; 6] = [
    (1, 4, 0x002a, 0),
    (1, 12, 0x003a, 0),
    (4, 0, 0x0026, 2),
    (0, 9, 0x0024, 2),
    (8, 9, 0x4024, 2),
    (4, 20, 0x016c, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES12: [DrawMultipleDataEnding; 21] = [
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

pub(super) const END_SEQUENCE_DRAW_FRAMES13: [DrawMultipleDataEnding; 16] = [
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

pub(super) const END_SEQUENCE_DRAW_FRAMES14: [DrawMultipleDataEnding; 6] = [
    (0, 0, 0, 0),
    (0, 0, 0x34c7, 0),
    (0, 0, 0x3480, 0),
    (0, 0, 0x34b6, 0),
    (0, 0, 0x34b7, 0),
    (0, 0, 0x34a6, 0),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES15: [DrawMultipleDataEnding; 6] = [
    (-3, 17, 0x002b, 0),
    (-3, 25, 0x003b, 0),
    (0, 0, 0x000e, 2),
    (16, 0, 0x400e, 2),
    (0, 16, 0x002e, 2),
    (16, 16, 0x402e, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES16: [DrawMultipleDataEnding; 3] =
    [(8, 5, 0x0a04, 2), (0, 16, 0x0806, 2), (16, 16, 0x4806, 2)];

pub(super) const END_SEQUENCE_DRAW_FRAMES17: [DrawMultipleDataEnding; 2] =
    [(0, 0, 0x0000, 2), (0, 11, 0x0002, 2)];

pub(super) const END_SEQUENCE_DRAW_FRAMES18: [DrawMultipleDataEnding; 2] =
    [(0, 0, 0x000e, 2), (0, 64, 0x006c, 2)];

pub(super) const END_SEQUENCE_DRAW_FRAMES19: [DrawMultipleDataEnding; 8] = [
    (0, 0, 0x0882, 2),
    (0, 7, 0x0a4e, 2),
    (0, 0, 0x4880, 2),
    (0, 7, 0x0a4e, 2),
    (0, 0, 0x0882, 2),
    (0, 7, 0x0a4e, 2),
    (0, 0, 0x0880, 2),
    (0, 7, 0x0a4e, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES20: [DrawMultipleDataEnding; 6] = [
    (-4, 1, 0x0c68, 0),
    (0, -8, 0x0c40, 2),
    (0, 1, 0x0c42, 2),
    (-4, 1, 0x0c78, 0),
    (0, -8, 0x0c40, 2),
    (0, 1, 0x0c42, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES21: [DrawMultipleDataEnding; 6] = [
    (8, 5, 0x0679, 0),
    (0, -10, 0x088e, 2),
    (0, 0, 0x066e, 2),
    (0, -10, 0x088e, 2),
    (0, -10, 0x088e, 2),
    (0, 0, 0x066e, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES22: [DrawMultipleDataEnding; 6] = [
    (11, -3, 0x0869, 0),
    (0, -12, 0x0804, 2),
    (0, 0, 0x0860, 2),
    (10, -3, 0x0867, 0),
    (0, -12, 0x0804, 2),
    (0, 0, 0x0860, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES23: [DrawMultipleDataEnding; 6] = [
    (-2, 1, 0x0868, 0),
    (0, -8, 0x08c0, 2),
    (0, 0, 0x08c2, 2),
    (-3, 1, 0x0878, 0),
    (0, -8, 0x08c0, 2),
    (0, 0, 0x08c2, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES24: [DrawMultipleDataEnding; 4] = [
    (0, -10, 0x084c, 2),
    (0, 0, 0x0a6c, 2),
    (0, -9, 0x084c, 2),
    (0, 0, 0x0aa8, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES25: [DrawMultipleDataEnding; 4] = [
    (0, -7, 0x084a, 2),
    (0, 0, 0x0c6a, 2),
    (0, -7, 0x084a, 2),
    (0, 0, 0x0ca6, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES26: [DrawMultipleDataEnding; 12] = [
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

pub(super) const END_SEQUENCE_DRAW_FRAMES27: [DrawMultipleDataEnding; 6] = [
    (0, -4, 0x30aa, 2),
    (0, -4, 0x30aa, 2),
    (-4, -8, 0x3090, 0),
    (12, -8, 0x7090, 0),
    (-6, -10, 0x3091, 0),
    (14, -10, 0x7091, 0),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES28: [DrawMultipleDataEnding; 8] = [
    (0, 0, 0x0722, 2),
    (0, -8, 0x09c2, 2),
    (0, 0, 0x4722, 2),
    (0, -8, 0x09c2, 2),
    (0, -9, 0x09c4, 2),
    (0, 0, 0x0722, 2),
    (0, -9, 0x0924, 2),
    (0, 0, 0x0722, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES29: [DrawMultipleDataEnding; 3] = [
    (-16, -12, 0x3f08, 2),
    (0, -12, 0x3f20, 2),
    (16, -12, 0x3f20, 2),
];

pub(super) const END_SEQUENCE_DRAW_FRAMES30: [DrawMultipleDataEnding; 1] = [(0, 0, 0x0086, 2)];

pub(super) const END_SEQUENCE_DRAW_FRAMES31: [DrawMultipleDataEnding; 1] = [(0, 0, 0x8060, 2)];

pub(super) const END_SEQUENCE_DRAW_FRAME_SETS: [&[DrawMultipleDataEnding]; 32] = [
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

pub(super) const DUNG_PAL_INFOS_ENDING: [(u8, u8, u8, u8); 41] = [
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
