//! Shared tables for attract.rs helpers.

// ---------------------------------------------------------------------------
// File-level attract draw tables.
// ---------------------------------------------------------------------------

pub(in crate::zelda_rtl) const SOLDIER_DRAW1_CHAR: [u8; 4] = [0x42, 0x42, 0x40, 0x44];

pub(in crate::zelda_rtl) const SOLDIER_DRAW1_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

pub(in crate::zelda_rtl) const SOLDIER_DRAW1_YD: [i8; 26] = [
    7, 8, 7, 8, 8, 7, 8, 7, 8, 7, 8, 8, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
];

pub(in crate::zelda_rtl) const SOLDIER_DRAW2_XD: [i8; 104] = [
    -4, 4, 10, 10, -4, 4, 10, 10, -4, 4, 10, 10, -4, 4, 10, 10, -4, -4, 0, 0, -4, -4, 0, 0, -3, -3,
    0, 0, -3, -3, -4, 4, -3, -3, -4, 4, -3, -3, -4, 4, -3, -3, -4, 4, 12, 12, 0, 0, 12, 12, 0, 0,
    11, 11, 0, 0, -4, 4, 0, 0, -4, 4, 0, 0, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -4, 4,
    0, 0, -4, 4, 0, 0, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(in crate::zelda_rtl) const SOLDIER_DRAW2_YD: [i8; 104] = [
    0, 0, 2, 10, 0, 0, 2, 10, 0, 0, 1, 9, 0, 0, 2, 10, -2, 6, 1, 1, -2, 6, 2, 2, -2, 6, 1, 1, -5,
    3, 0, 0, -4, 4, 0, 0, -4, 4, 0, 0, -5, 3, 0, 0, -2, 6, 1, 1, -2, 6, 2, 2, -2, 6, 1, 1, 0, 0, 8,
    8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8,
    8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8,
];

pub(in crate::zelda_rtl) const SOLDIER_DRAW2_CHAR: [u8; 104] = [
    0x48, 0x49, 0x6d, 0x7d, 0x49, 0x48, 0x6d, 0x7d, 0x46, 0x46, 0x6d, 0x7d, 0x4b, 0x46, 0x6d, 0x7d,
    0x4d, 0x5d, 0x4e, 0x4e, 0x4d, 0x5d, 0x60, 0x60, 0x4d, 0x5d, 0x62, 0x62, 0x6d, 0x7d, 0x64, 0x64,
    0x6d, 0x7d, 0x66, 0x67, 0x6d, 0x7d, 0x67, 0x66, 0x6d, 0x7d, 0x64, 0x69, 0x4d, 0x5d, 0x4e, 0x4e,
    0x4d, 0x5d, 0x60, 0x60, 0x4d, 0x5d, 0x62, 0x62, 2, 3, 0x20, 0x20, 2, 0x0c, 0x20, 0x20, 2, 0x0c,
    0x20, 0x20, 8, 8, 0x20, 0x20, 0x0e, 0x0e, 0x20, 0x20, 0x0e, 0x0e, 0x20, 0x20, 5, 6, 0x20, 0x20,
    0x22, 6, 0x20, 0x20, 0x22, 6, 0x20, 0x20, 8, 8, 0x20, 0x20, 0x0e, 0x0e, 0x20, 0x20, 0x0e, 0x0e,
    0x20, 0x20,
];

pub(in crate::zelda_rtl) const SOLDIER_DRAW2_FLAGS: [u8; 104] = [
    0, 0, 0, 0, 0x40, 0x40, 0, 0, 0, 0x40, 0, 0, 0, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x40, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40,
];

pub(in crate::zelda_rtl) const SOLDIER_DRAW2_BIG: [u8; 104] = [
    2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2,
    0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2,
];

pub(in crate::zelda_rtl) const SOLDIER_DRAW2_OAM_IDX: [u8; 4] = [12, 12, 12, 4];

pub(in crate::zelda_rtl) type AttractOamInfo = (i8, i8, u8, u8, u8);

pub(in crate::zelda_rtl) const SOLDIER_DRAW3_XD: [i8; 28] = [
    -3, -3, -4, -4, -4, -4, -4, -4, -11, -3, -11, -3, -16, -8, 12, 12, 12, 12, 12, 12, 12, 12, 21,
    13, 21, 13, 24, 16,
];

pub(in crate::zelda_rtl) const SOLDIER_DRAW3_YD: [i8; 28] = [
    11, 19, 11, 19, 10, 18, 14, 22, 8, 8, 8, 8, 6, 6, -10, -2, -9, -1, -9, -1, -16, -8, 8, 8, 8, 8,
    6, 6,
];

pub(in crate::zelda_rtl) const SOLDIER_DRAW3_CHAR: [u8; 28] = [
    0x7b, 0x6b, 0x7b, 0x6b, 0x7b, 0x6b, 0x7b, 0x6b, 0x6c, 0x7c, 0x6c, 0x7c, 0x6c, 0x7c, 0x6b, 0x7b,
    0x6b, 0x7b, 0x6b, 0x7b, 0x6b, 0x7b, 0x6c, 0x7c, 0x6c, 0x7c, 0x6c, 0x7c,
];

pub(in crate::zelda_rtl) const SOLDIER_DRAW3_FLAGS: [u8; 28] = [
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40,
];

pub(in crate::zelda_rtl) const SOLDIER_DRAW3_OAM_IDX: [u8; 4] = [4, 4, 4, 20];

pub(in crate::zelda_rtl) const SOLDIER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

// ---------------------------------------------------------------------------
// Promoted attract method-local tables. Names retain the owning helper so
// generic C table names stay readable at callsites.
// ---------------------------------------------------------------------------

pub(in crate::zelda_rtl) const ATTRACT_MAP_ZOOM_HDMA_BASES: [u16; 240] = [
    375, 374, 373, 373, 372, 371, 371, 370, 369, 369, 368, 367, 367, 366, 365, 365, 364, 363, 363,
    361, 361, 360, 359, 359, 358, 357, 357, 356, 355, 355, 354, 354, 353, 352, 352, 351, 351, 350,
    349, 349, 348, 348, 347, 346, 346, 345, 345, 344, 343, 343, 342, 342, 341, 341, 340, 339, 339,
    338, 338, 337, 337, 336, 335, 335, 334, 334, 333, 333, 332, 332, 331, 331, 330, 330, 328, 327,
    327, 326, 326, 325, 325, 324, 324, 323, 323, 322, 322, 321, 321, 320, 320, 319, 319, 318, 318,
    317, 317, 316, 316, 315, 315, 314, 314, 313, 313, 312, 312, 311, 311, 310, 310, 309, 309, 309,
    308, 308, 307, 307, 306, 306, 305, 305, 304, 304, 303, 303, 303, 302, 302, 301, 301, 300, 300,
    299, 299, 299, 298, 298, 297, 297, 295, 295, 294, 294, 294, 293, 293, 292, 292, 292, 291, 291,
    290, 290, 289, 289, 289, 288, 288, 287, 287, 287, 286, 286, 285, 285, 285, 284, 284, 283, 283,
    283, 282, 282, 281, 281, 281, 280, 280, 279, 279, 279, 278, 278, 278, 277, 277, 276, 276, 276,
    275, 275, 275, 274, 274, 273, 273, 273, 272, 272, 272, 271, 271, 271, 270, 270, 269, 269, 269,
    268, 268, 268, 267, 267, 267, 266, 266, 266, 265, 265, 265, 264, 264, 264, 263, 263, 262, 262,
    262, 261, 261, 261, 260, 260, 260, 259, 259, 259, 258, 258,
];

pub(in crate::zelda_rtl) const ATTRACT_THRONE_ROOM_SPRITE_ENTRIES: [(i8, i8, u8, u8, u8); 10] = [
    (16, 16, 0x2a, 0x7b, 2),
    (0, 16, 0x2a, 0x3b, 2),
    (16, 0, 0x0a, 0x7b, 2),
    (0, 0, 0x0a, 0x3b, 2),
    (0, 0, 0x0c, 0x31, 2),
    (16, 0, 0x0e, 0x31, 2),
    (32, 0, 0x0c, 0x71, 2),
    (0, 16, 0x2c, 0x31, 2),
    (16, 16, 0x2e, 0x31, 2),
    (32, 16, 0x2c, 0x71, 2),
];

pub(in crate::zelda_rtl) const ATTRACT_THRONE_ROOM_SPRITE_ENTRY_STARTS: [usize; 3] = [0, 4, 10];

pub(in crate::zelda_rtl) const ATTRACT_THRONE_ROOM_X_BASES: [u8; 2] = [80, 104];

pub(in crate::zelda_rtl) const ATTRACT_THRONE_ROOM_Y_BASES: [i16; 2] = [88, 32];

pub(in crate::zelda_rtl) const ATTRACT_DRAMATIZE_AGAHNIM_ALTAR_SOLDIER_X: [u16; 6] =
    [48, 192, 48, 192, 80, 160];

pub(in crate::zelda_rtl) const ATTRACT_DRAMATIZE_AGAHNIM_ALTAR_SOLDIER_Y: [u16; 6] =
    [112, 112, 152, 152, 192, 192];

pub(in crate::zelda_rtl) const ATTRACT_DRAMATIZE_AGAHNIM_ALTAR_SOLDIER_DIR: [u8; 6] =
    [0, 1, 0, 1, 3, 3];

pub(in crate::zelda_rtl) const ATTRACT_DRAMATIZE_AGAHNIM_ALTAR_SOLDIER_FLAGS: [u8; 6] =
    [9, 9, 9, 9, 7, 9];

pub(in crate::zelda_rtl) const ATTRACT_AGAHNIM_ALTAR_MAIDEN_CORE_ENTRIES: [(i8, i8, u8, u8, u8);
    4] = [
    (0, 0, 0x03, 0x3d, 2),
    (8, 0, 0x04, 0x3d, 2),
    (0, 0, 0x00, 0x3d, 2),
    (8, 0, 0x01, 0x3d, 2),
];

pub(in crate::zelda_rtl) const ATTRACT_AGAHNIM_ALTAR_MAIDEN_X_BASE_OFFSETS: [u8; 8] =
    [4, 4, 3, 3, 2, 2, 1, 0];

pub(in crate::zelda_rtl) const ATTRACT_AGAHNIM_ALTAR_MAIDEN_SHIMMER_ENTRIES: [(
    i8,
    i8,
    u8,
    u8,
    u8,
); 16] = [
    (0, 0, 0x6c, 0x38, 2),
    (0, 0, 0x6c, 0x38, 2),
    (0, 0, 0x6c, 0x38, 2),
    (0, 0, 0x6c, 0x38, 2),
    (0, 0, 0x6c, 0x38, 2),
    (2, 0, 0x6c, 0x38, 2),
    (0, 0, 0x6c, 0x38, 2),
    (2, 0, 0x6c, 0x38, 2),
    (0, 0, 0x6c, 0x38, 2),
    (4, 0, 0x6c, 0x38, 2),
    (0, 0, 0x6c, 0x38, 2),
    (4, 0, 0x6c, 0x38, 2),
    (0, 0, 0x6c, 0x38, 2),
    (6, 0, 0x6c, 0x38, 2),
    (0, 0, 0x6c, 0x38, 2),
    (8, 0, 0x6c, 0x38, 2),
];

pub(in crate::zelda_rtl) const ATTRACT_AGAHNIM_ALTAR_MAIDEN_WARP_ENTRIES: [(i8, i8, u8, u8, u8);
    48] = [
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x82, 0x3b, 2),
    (16, 0, 0x82, 0x7b, 2),
    (0, 16, 0xa2, 0x3b, 2),
    (16, 16, 0xa2, 0x7b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x80, 0x3b, 2),
    (16, 0, 0x82, 0x7b, 2),
    (0, 16, 0xa0, 0x3b, 2),
    (16, 16, 0xa2, 0x7b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x82, 0x3b, 2),
    (16, 0, 0x82, 0x7b, 2),
    (0, 16, 0xa2, 0x3b, 2),
    (16, 16, 0xa2, 0x7b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x82, 0x3b, 2),
    (16, 0, 0x80, 0x7b, 2),
    (0, 16, 0xa2, 0x3b, 2),
    (16, 16, 0xa0, 0x7b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x82, 0x3b, 2),
    (16, 0, 0x82, 0x7b, 2),
    (0, 16, 0xa2, 0x3b, 2),
    (16, 16, 0xa2, 0x7b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x80, 0x3b, 2),
    (16, 0, 0x82, 0x7b, 2),
    (0, 16, 0xa0, 0x3b, 2),
    (16, 16, 0xa2, 0x7b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x82, 0x3b, 2),
    (16, 0, 0x82, 0x7b, 2),
    (0, 16, 0xa2, 0x3b, 2),
    (16, 16, 0xa2, 0x7b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x80, 0x3b, 2),
    (16, 0, 0x80, 0x7b, 2),
    (0, 16, 0xa0, 0x3b, 2),
    (16, 16, 0xa0, 0x7b, 2),
];

pub(in crate::zelda_rtl) const ATTRACT_MAIDEN_WARP_CASE1_SPRITE_ENTRIES: [(i8, i8, u8, u8, u8);
    28] = [
    (0, 0, 0xce, 0x35, 0),
    (28, 0, 0xce, 0x35, 0),
    (-2, 3, 0x26, 0x75, 0),
    (30, 3, 0x26, 0x35, 0),
    (-2, 11, 0x36, 0x75, 0),
    (30, 11, 0x36, 0x35, 0),
    (0, 16, 0x26, 0x75, 0),
    (28, 16, 0x26, 0x35, 0),
    (0, 24, 0x36, 0x75, 0),
    (28, 24, 0x36, 0x35, 0),
    (2, 16, 0x20, 0x35, 2),
    (18, 16, 0x20, 0x75, 2),
    (2, 32, 0x20, 0xb5, 2),
    (18, 32, 0x20, 0xf5, 2),
    (0, 0, 0xce, 0x37, 0),
    (28, 0, 0xce, 0x37, 0),
    (-2, 3, 0x26, 0x77, 0),
    (30, 3, 0x26, 0x37, 0),
    (-2, 11, 0x36, 0x77, 0),
    (30, 11, 0x36, 0x37, 0),
    (0, 16, 0x26, 0x77, 0),
    (28, 16, 0x26, 0x37, 0),
    (0, 24, 0x36, 0x77, 0),
    (28, 24, 0x36, 0x37, 0),
    (2, 16, 0x22, 0x37, 2),
    (18, 16, 0x22, 0x77, 2),
    (2, 32, 0x22, 0xb7, 2),
    (18, 32, 0x22, 0xf7, 2),
];

pub(in crate::zelda_rtl) const ATTRACT_MAIDEN_WARP_CASE1_ENTRY_COUNTS: [usize; 8] =
    [2, 2, 2, 6, 6, 10, 10, 14];

pub(in crate::zelda_rtl) const ATTRACT_MAIDEN_WARP_CASE2_ENTRY_COUNTS: [usize; 8] =
    [4, 4, 8, 8, 12, 12, 14, 14];

pub(in crate::zelda_rtl) const ATTRACT_MAIDEN_WARP_CASE2_SPRITE_ENTRIES: [(i8, i8, u8, u8, u8);
    28] = [
    (0, 0, 0xce, 0x35, 0),
    (28, 0, 0xce, 0x35, 0),
    (-2, 3, 0x26, 0x75, 0),
    (30, 3, 0x26, 0x35, 0),
    (-2, 11, 0x36, 0x75, 0),
    (30, 11, 0x36, 0x35, 0),
    (0, 16, 0x26, 0x75, 0),
    (28, 16, 0x26, 0x35, 0),
    (0, 24, 0x36, 0x75, 0),
    (28, 24, 0x36, 0x35, 0),
    (2, 16, 0x20, 0x35, 2),
    (18, 16, 0x20, 0x75, 2),
    (2, 32, 0x20, 0xb5, 2),
    (18, 32, 0x20, 0xf5, 2),
    (0, 0, 0xce, 0x37, 0),
    (28, 0, 0xce, 0x37, 0),
    (-2, 3, 0x26, 0x77, 0),
    (30, 3, 0x26, 0x37, 0),
    (-2, 11, 0x36, 0x77, 0),
    (30, 11, 0x36, 0x37, 0),
    (0, 16, 0x26, 0x77, 0),
    (28, 16, 0x26, 0x37, 0),
    (0, 24, 0x36, 0x77, 0),
    (28, 24, 0x36, 0x37, 0),
    (2, 16, 0x22, 0x37, 2),
    (18, 16, 0x22, 0x77, 2),
    (2, 32, 0x22, 0xb7, 2),
    (18, 32, 0x22, 0xf7, 2),
];

pub(in crate::zelda_rtl) const ATTRACT_MAIDEN_WARP_CASE3_SPRITE_ENTRIES: [(i8, i8, u8, u8, u8); 3] = [
    (0, 0, 0xc6, 0x3d, 2),
    (0, 0, 0x24, 0x35, 2),
    (16, 0, 0x24, 0x75, 2),
];

pub(in crate::zelda_rtl) const ATTRACT_MAIDEN_WARP_CASE3_X_BASES: [u8; 2] = [0x78, 0x70];

pub(in crate::zelda_rtl) const ATTRACT_DRAMATIZE_PRISON_ZELDA_ANIMATION_FRAMES: [u8; 16] =
    [0, 1, 2, 3, 4, 5, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1];

pub(in crate::zelda_rtl) const ATTRACT_DRAMATIZE_PRISON_SOLDIER_X: [i16; 2] = [32, -12];

pub(in crate::zelda_rtl) const ATTRACT_DRAMATIZE_PRISON_SOLDIER_Y: [u16; 2] = [24, 24];

pub(in crate::zelda_rtl) const ATTRACT_DRAMATIZE_PRISON_SOLDIER_DIR: [u8; 2] = [1, 1];

pub(in crate::zelda_rtl) const ATTRACT_DRAMATIZE_PRISON_SOLDIER_FLAGS: [u8; 2] = [9, 7];

pub(in crate::zelda_rtl) const ATTRACT_ZELDA_PRISON_CASE0_SPRITE_ENTRIES: [(i8, i8, u8, u8, u8);
    6] = [
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x84, 0x3b, 2),
    (16, 0, 0x84, 0x7b, 2),
    (0, 16, 0xa4, 0x3b, 2),
    (16, 16, 0xa4, 0x7b, 2),
];

pub(in crate::zelda_rtl) const ATTRACT_ZELDA_PRISON_CASE1_SPRITE_ENTRIES: [(i8, i8, u8, u8, u8);
    30] = [
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x84, 0x3b, 2),
    (16, 0, 0x84, 0x7b, 2),
    (0, 16, 0xa4, 0x3b, 2),
    (16, 16, 0xa4, 0x7b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0xc4, 0x3b, 2),
    (16, 0, 0xc2, 0x3b, 2),
    (0, 16, 0xe4, 0x3b, 2),
    (16, 16, 0xe6, 0x3b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x88, 0x3b, 2),
    (16, 0, 0x8a, 0x3b, 2),
    (0, 16, 0xa8, 0x3b, 2),
    (16, 16, 0xaa, 0x3b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x82, 0x3b, 2),
    (16, 0, 0x82, 0x7b, 2),
    (0, 16, 0xa2, 0x3b, 2),
    (16, 16, 0xa2, 0x7b, 2),
    (5, 25, 0x6c, 0x38, 2),
    (11, 25, 0x6c, 0x38, 2),
    (0, 0, 0x80, 0x3b, 2),
    (16, 0, 0x80, 0x7b, 2),
    (0, 16, 0xa0, 0x3b, 2),
    (16, 16, 0xa0, 0x7b, 2),
];

pub(in crate::zelda_rtl) const SPRITE_SIMULATE_SOLDIER_GRAPHICS_BY_DIRECTION: [u8; 4] =
    [11, 4, 0, 7];

pub(in crate::zelda_rtl) const ATTRACT_BACKGROUND_TILE_PATTERN: [u16; 16] = [
    0x01a0, 0x09a6, 0x89a5, 0x01a0, 0x09a5, 0x01a0, 0x01a0, 0x89a6, 0x49a5, 0x01a0, 0x01a0, 0x49a5,
    0x01a0, 0x89a5, 0xc9a5, 0x01a0,
];

pub(in crate::zelda_rtl) const ATTRACT_BACKGROUND_CORNER_TILES: [u16; 4] =
    [0x09a1, 0x09a2, 0x09a3, 0x09a4];
