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

// ---------------------------------------------------------------------------
// Promoted ending method-local tables. Names retain the owning helper so
// generic C table names stay readable at callsites.
// ---------------------------------------------------------------------------

pub(super) const INTRO_COPY_SPRITE_TYPE4_TO_OAM_LEFT_ENTRIES: [(i8, i8, u8, u8, u8); 16] = [
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

pub(super) const INTRO_COPY_SPRITE_TYPE4_TO_OAM_RIGHT_ENTRIES: [(i8, i8, u8, u8, u8); 16] = [
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

pub(super) const INITIALIZE_SCENE_SPRITE_TRIFORCE_ROOM_TRIANGLE_X_OFFSETS: [i16; 3] =
    [0x4e, 0x5f, 0x72];

pub(super) const INITIALIZE_SCENE_SPRITE_TRIFORCE_ROOM_TRIANGLE_Y_OFFSETS: [i16; 3] =
    [0x9c, 0x9c, 0x9c];

pub(super) const INITIALIZE_SCENE_SPRITE_TRIFORCE_ROOM_TRIANGLE_X_VELOCITIES: [i8; 3] = [-2, 0, 2];

pub(super) const INITIALIZE_SCENE_SPRITE_TRIFORCE_ROOM_TRIANGLE_Y_VELOCITIES: [i8; 3] = [4, -4, 4];

pub(super) const ANIMATE_TRIFORCE_ROOM_TRIANGLE_HANDLE_CONTRACTING_FINAL_X: [u8; 3] =
    [0x59, 0x5f, 0x67];

pub(super) const ANIMATE_TRIFORCE_ROOM_TRIANGLE_HANDLE_CONTRACTING_FINAL_Y: [u8; 3] =
    [0x74, 0x68, 0x74];

pub(super) const INITIALIZE_SCENE_SPRITE_CREDITS_TRIANGLE_X_OFFSETS: [u8; 3] = [0x29, 0x5f, 0x97];

pub(super) const INITIALIZE_SCENE_SPRITE_CREDITS_TRIANGLE_Y_OFFSETS: [u8; 3] = [0x70, 0x20, 0x70];

pub(super) const ANIMATE_SCENE_SPRITE_CREDITS_TRIANGLE_X_ACCELERATION: [i8; 3] = [-1, 0, 1];

pub(super) const ANIMATE_SCENE_SPRITE_CREDITS_TRIANGLE_Y_ACCELERATION: [i8; 3] = [1, -1, 1];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_SCENE_FADE_SCROLL_LIMITS: [u16; 16] = [
    0x300, 0x280, 0x250, 0x2e0, 0x280, 0x250, 0x2c0, 0x2c0, 0x250, 0x250, 0x280, 0x250, 0x480,
    0x400, 0x250, 0x500,
];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE0_SPRITE_CHARS: [u8; 12] = [
    0x1e, 0x20, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x16, 0x16, 0x16, 0x16,
];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE0_SPRITE_GFX: [u8; 12] =
    [6, 3, 2, 2, 2, 2, 2, 2, 6, 6, 6, 6];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CASE0_OAM_FLAGS: [u8; 12] = [
    0x3b, 0x31, 0x3d, 0x3f, 0x39, 0x3b, 0x37, 0x3d, 0x39, 0x37, 0x37, 0x39,
];

pub(super) const ENDING_FUNC2_DELAYS: [u8; 27] = [
    10, 10, 10, 10, 20, 8, 8, 0, 255, 12, 12, 12, 12, 12, 12, 30, 8, 4, 4, 4, 0, 0, 255, 255, 144,
    4, 0,
];

pub(super) const ENDING_FUNC2_ENDING_SPRITE_ANIMATION_STEPS: [i8; 28] = [
    0, 0, 1, 0, 1, 0, 2, 3, 0, 2, 0, 1, 0, 1, 0, 1, 2, 3, 4, 5, 6, 3, 0, -1, -1, -1, 2, 3,
];

pub(super) const CREDITS_SPRITE_DRAW_ADD_SPARKLE_DELAYS: [u8; 6] = [32, 4, 4, 4, 5, 6];

pub(super) const CREDITS_SPRITE_DRAW_WALK_LINK_AWAY_FROM_PEDESTAL_DMA_SOURCES: [u16; 8] =
    [0x16c, 0x16e, 0x170, 0x172, 0x16c, 0x174, 0x176, 0x178];

pub(super) const CREDITS_SPRITE_DRAW_MOVE_SQUIRREL_X_VELOCITIES: [i8; 4] = [32, 24, -32, -24];

pub(super) const CREDITS_SPRITE_DRAW_MOVE_SQUIRREL_Y_VELOCITIES: [i8; 4] = [8, -8, -8, 8];

pub(super) const CREDITS_SPRITE_DRAW_CIRCLING_BIRDS_TARGET_X_OFFSETS: [i8; 2] = [0x20, -0x20];

pub(super) const CREDITS_SPRITE_DRAW_CIRCLING_BIRDS_TARGET_Y_OFFSETS: [i8; 2] = [0x10, -0x10];

pub(super) const END_SEQUENCE_32_HEALTH_AFTER_DEATH: [u8; 21] = [
    0x18, 0x18, 0x18, 0x18, 0x18, 0x20, 0x20, 0x28, 0x28, 0x30, 0x30, 0x38, 0x38, 0x38, 0x40, 0x40,
    0x40, 0x48, 0x48, 0x48, 0x50,
];

pub(super) const CREDITS_ADD_NEXT_ATTRIBUTION_ATTRIBUTION_PALACE_ORDER: [usize; 14] =
    [1, 0, 2, 3, 10, 6, 5, 8, 11, 9, 7, 12, 13, 15];

pub(super) const CREDITS_ADD_NEXT_ATTRIBUTION_DIGITS_SCROLL_Y: [u16; 14] = [
    0x290, 0x298, 0x2a0, 0x2a8, 0x2b0, 0x2ba, 0x2c2, 0x2ca, 0x2d2, 0x2da, 0x2e2, 0x2ea, 0x2f2,
    0x310,
];

pub(super) const CREDITS_ADD_NEXT_ATTRIBUTION_DIGIT_CHARS: [u16; 2] = [0x3ce6, 0x3cf6];

pub(super) const INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_CHARS: [u8; 10] =
    [0, 2, 0x20, 0x22, 4, 6, 8, 0x0a, 0x0c, 0x0e];

pub(super) const INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_X_OFFSETS: [u8; 10] =
    [0x40, 0x40, 0x30, 0x50, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40];

pub(super) const INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_Y_OFFSETS: [u16; 10] =
    [0x10, 0x20, 0x28, 0x28, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80];

pub(super) const INTRO_SPRITE_TYPE_A_0_X_OFFSETS: [i16; 3] = [-38, 95, 230];

pub(super) const INTRO_SPRITE_TYPE_A_0_Y_OFFSETS: [i16; 3] = [200, -67, 200];

pub(super) const INTRO_SPRITE_TYPE_A_0_X_VELOCITIES: [i8; 3] = [1, 0, -1];

pub(super) const INTRO_SPRITE_TYPE_A_0_Y_VELOCITIES: [i8; 3] = [-1, 1, -1];

pub(super) const ANIMATE_SCENE_SPRITE_COPYRIGHT_SPRITE_ENTRIES: [(i8, i8, u8, u8, u8); 13] = [
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

pub(super) const INITIALIZE_SCENE_SPRITE_SPARKLE_X_OFFSETS: [u8; 4] = [0xc2, 0x98, 0x6f, 0x34];

pub(super) const INITIALIZE_SCENE_SPRITE_SPARKLE_Y_OFFSETS: [u8; 4] = [0x7c, 0x54, 0x7c, 0x57];

pub(super) const ANIMATE_SCENE_SPRITE_SPARKLE_SPRITE_ENTRIES: [(i8, i8, u8, u8, u8); 4] = [
    (0, 0, 0x80, 0x34, 0),
    (0, 0, 0xb7, 0x34, 0),
    (-4, -3, 0x64, 0x38, 2),
    (-4, -3, 0x62, 0x34, 2),
];

pub(super) const ANIMATE_SCENE_SPRITE_SPARKLE_STATES: [u8; 8] = [0, 1, 2, 3, 2, 1, 0xff, 0xff];

pub(super) const ANIMATE_SCENE_SPRITE_DRAW_TRIANGLE_LEFT_ENTRIES: [(i8, i8, u8, u8, u8); 16] = [
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

pub(super) const ANIMATE_SCENE_SPRITE_DRAW_TRIANGLE_RIGHT_ENTRIES: [(i8, i8, u8, u8, u8); 16] = [
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

pub(super) const INTRO_DISPLAY_LOGO_INTRO_LOGO_X: [u8; 4] = [0x60, 0x70, 0x80, 0x88];

pub(super) const INTRO_DISPLAY_LOGO_INTRO_LOGO_TILE: [u8; 4] = [0x69, 0x6b, 0x6d, 0x6e];

// ---------------------------------------------------------------------------
// Promoted nested ending tables from credits and intro match arms.
// ---------------------------------------------------------------------------

pub(super) const INTRO_SPRITE_TYPE_B_456_Y_ACCELERATION: [i8; 3] = [-1, -1, -1];

pub(super) const INTRO_SPRITE_TYPE_B_456_FINAL_Y2: [u8; 3] = [0x72, 0x66, 0x72];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE2_BIRD_FLAG_FRAMES: [u8; 2] = [0x20, 0x40];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE2_BIRD_OAM_VELOCITY_OFFSETS: [i8; 2] =
    [16, -16];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE2_SPRITE_CHARS: [u8; 5] =
    [0x28, 0x2a, 0x2c, 0x2e, 0x2c];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE2_SPRITE_GFX: [u8; 5] = [3, 3, 3, 3, 3];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CASE2_DELAYS: [u8; 2] = [0x30, 0x10];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CASE3_GRAPHICS: [u8; 4] = [1, 2, 3, 2];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE4_SPRITE_CHARS: [u8; 2] = [0x30, 0x32];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE4_SPRITE_GFX: [u8; 2] = [2, 2];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CASE4_COUNTERS: [u16; 2] = [0x20, 0];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CASE4_XY_VELOCITIES: [i8; 10] =
    [0, -12, -16, -12, 0, 12, 16, 12, 0, -12];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CASE4_DELAY_VELOCITIES: [u8; 24] = [
    0x3b, 0x14, 0x1e, 0x1d, 0x2c, 0x2b, 0x42, 0x20, 0x27, 0x28, 0x2e, 0x38, 0x3a, 0x4c, 0x32, 0x44,
    0x2e, 0x2f, 0x1e, 0x28, 0x47, 0x35, 0x32, 0x30,
];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE5_SHIELD_DMA_GFX: [u8; 2] = [0, 4];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE5_LINK_DMA_GFX: [u16; 2] = [0x0a, 0x224];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE5_SPRITE_CHARS: [u8; 2] = [10, 14];

pub(super) const CREDITS_HANDLE_SCENE_FADE_SPRITE_TYPES: [u8; 3] = [0x52, 0x55, 0x55];

pub(super) const CREDITS_HANDLE_SCENE_FADE_OAM_SIZES: [u8; 3] = [0x20, 8, 8];

pub(super) const CREDITS_HANDLE_SCENE_FADE_STATES: [u8; 3] = [3, 1, 1];

pub(super) const CREDITS_HANDLE_SCENE_FADE_GRAPHICS: [u8; 6] = [0, 5, 5, 1, 6, 6];

pub(super) const CREDITS_HANDLE_SCENE_FADE_GRAPHICS_STEPS: [i8; 2] = [1, -1];

pub(super) const CREDITS_HANDLE_SCENE_FADE_DELAYS1: [u8; 4] = [16, 14, 16, 18];

pub(super) const CREDITS_HANDLE_SCENE_FADE_DELAYS2: [u8; 4] = [20, 48, 20, 20];

pub(super) const CREDITS_HANDLE_SCENE_FADE_DIRECTIONS: [u8; 4] = [0, 1, 0, 1];

pub(super) const CREDITS_HANDLE_SCENE_FADE_OAM_FLAGS: [u8; 4] = [55, 55, 59, 61];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE8_RUN_SPRITE_CHARS: [u8; 4] = [8, 8, 12, 12];

pub(super) const CREDITS_HANDLE_SCENE_FADE_WISH_POND_X_OFFSETS: [u8; 8] =
    [0, 4, 8, 12, 16, 20, 24, 0];

pub(super) const CREDITS_HANDLE_SCENE_FADE_WISH_POND_Y_OFFSETS: [u8; 8] =
    [0, 8, 16, 24, 32, 40, 4, 36];

pub(super) const CREDITS_HANDLE_SCENE_FADE_GRAPHICS_2: [u8; 16] =
    [1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE12_SPRITE_GFX: [u8; 3] = [3, 3, 8];

pub(super) const CREDITS_HANDLE_SCENE_FADE_Z_OFFSETS: [u8; 15] =
    [2, 4, 5, 6, 6, 7, 7, 7, 7, 6, 6, 5, 4, 2, 0];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE14_BIRD_GFX_FRAMES: [u8; 4] = [0, 1, 0, 2];

pub(super) const CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE14_TIMING_THRESHOLDS: [u8; 5] =
    [2, 8, 32, 32, 8];

pub(super) const CREDITS_HANDLE_SCENE_FADE_X_OFFSETS: [u8; 4] = [0x76, 0x73, 0x71, 0x78];

pub(super) const CREDITS_HANDLE_SCENE_FADE_Y_OFFSETS: [u8; 4] = [0x8b, 0x83, 0x8d, 0x85];

pub(super) const CREDITS_HANDLE_SCENE_FADE_DELAYS: [u8; 8] = [6, 6, 6, 6, 6, 6, 10, 8];

pub(super) const CREDITS_HANDLE_SCENE_FADE_OAM_FLAGS_2: [u8; 4] = [0x61, 0x61, 0x3b, 0x39];

pub(super) const INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_INTRO_SWORD_SPARKLE_TIMERS: [u8; 8] =
    [4, 4, 6, 6, 6, 4, 4, 0];

pub(super) const INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_SPARKLE_CHARS: [u8; 7] =
    [0x28, 0x37, 0x27, 0x36, 0x27, 0x37, 0x28];

pub(super) const INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_SPARKLE_CHARS_2: [u8; 8] =
    [0x26, 0x20, 0x24, 0x34, 0x25, 0x20, 0x35, 0x20];

pub(super) const INTRO_SPRITE_TYPE_B_0_X_LIMITS: [u8; 3] = [75, 95, 117];

pub(super) const INTRO_SPRITE_TYPE_B_0_Y_LIMITS: [u8; 3] = [88, 48, 88];
