//! Overworld enemy, aquatic creature, and wildlife draw and behavior tables.

use crate::zelda_rtl::sprite::DrawMultipleData;
pub(in crate::zelda_rtl::sprite_main_draw) const FEATURE_GAME_CHANGING_BUG_FIXES_DRAW: u32 = 16384;
pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_SURFACING_GRAPHICS: [u8; 16] =
    [4, 3, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 0, 0];
pub(in crate::zelda_rtl::sprite_main_draw) const THROWABLE_SCENERY_CHARS: [u8; 12] = [
    0x42, 0x44, 0x46, 0, 0x46, 0x44, 0x42, 0x44, 0x44, 0, 0x46, 0x44,
];
pub(in crate::zelda_rtl::sprite_main_draw) const THROWABLE_SCENERY_FLAGS: [u8; 9] = [0x0c, 0x0c, 0x0c, 0, 0, 0, 0xb0, 0x08, 0xb4];

pub(in crate::zelda_rtl::sprite_main_draw) const WATER_RIPPLE_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: 0,
        y: 10,
        char_flags: 0x01d8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 10,
        char_flags: 0x41d8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 10,
        char_flags: 0x01d9,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 10,
        char_flags: 0x41d9,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 10,
        char_flags: 0x01da,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 10,
        char_flags: 0x41da,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const WATER_RIPPLE_FRAME_INDICES: [u8; 4] = [0, 1, 2, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const BOMBER_PELLET_DRAW_FRAMES: [DrawMultipleData; 15] = [
    DrawMultipleData {
        x: -11,
        y: 0,
        char_flags: 0x019b,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0xc19b,
        ext: 0,
    },
    DrawMultipleData {
        x: 6,
        y: 6,
        char_flags: 0x419b,
        ext: 0,
    },
    DrawMultipleData {
        x: -15,
        y: -6,
        char_flags: 0x018a,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -14,
        char_flags: 0x018a,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: 0,
        char_flags: 0x018a,
        ext: 2,
    },
    DrawMultipleData {
        x: -15,
        y: -6,
        char_flags: 0x0186,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -14,
        char_flags: 0x0186,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: 0,
        char_flags: 0x0186,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x0186,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x0186,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x0186,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x01aa,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x01aa,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x01aa,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const PIKIT_TONGUE_LENGTH_MULTIPLIERS: [u8; 4] = [0x33, 0x66, 0x99, 0xcc];

pub(in crate::zelda_rtl::sprite_main_draw) const PIKIT_TONGUE_DRAW_CHARS: [u8; 8] =
    [0xee, 0xfd, 0xed, 0xfd, 0xee, 0xfd, 0xed, 0xfd];

pub(in crate::zelda_rtl::sprite_main_draw) const PIKIT_TONGUE_DRAW_FLAGS: [u8; 8] = [0, 0, 0, 0x40, 0x40, 0xc0, 0x80, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const PIKIT_GRABBED_ITEM_X_OFFSETS: [i8; 20] = [
    -4, 4, -4, 4, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, -4, 4, -4, 4,
];

pub(in crate::zelda_rtl::sprite_main_draw) const PIKIT_GRABBED_ITEM_Y_OFFSETS: [i8; 20] = [
    -4, -4, 4, 4, -4, -4, 4, 4, -4, -4, 4, 4, -4, -4, 4, 4, -4, -4, 4, 4,
];

pub(in crate::zelda_rtl::sprite_main_draw) const PIKIT_GRABBED_ITEM_CHARS: [u8; 20] = [
    0x6e, 0x6f, 0x7e, 0x7f, 0x63, 0x7c, 0x73, 0x7c, 0xb, 0x7c, 0x1b, 0x7c, 0xec, 0xf9, 0xfc, 0xf9,
    0xea, 0xeb, 0xfa, 0xfb,
];

pub(in crate::zelda_rtl::sprite_main_draw) const PIKIT_GRABBED_ITEM_FLAGS: [u8; 5] = [0x24, 0x24, 0x28, 0x29, 0x2f];

pub(in crate::zelda_rtl::sprite_main_draw) const CRAB_DRAW_X_OFFSETS: [i16; 4] = [-8, 8, -8, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const CRAB_DRAW_CHARS: [u8; 4] = [0x8e, 0x8e, 0xae, 0xae];

pub(in crate::zelda_rtl::sprite_main_draw) const CRAB_DRAW_FLAGS: [u8; 4] = [0, 0x40, 0, 0x40];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOROCK_DRAW_X_OFFSETS: [i8; 9] = [8, 0, 4, 8, 0, 4, 9, -1, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOROCK_DRAW_Y_OFFSETS: [i8; 9] = [6, 6, 9, 6, 6, 9, 6, 6, 9];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOROCK_DRAW_CHARS: [u8; 9] =
    [0xbb, 0xbb, 0xba, 0xab, 0xab, 0xaa, 0xa9, 0xa9, 0xb9];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOROCK_DRAW_FLAGS: [u8; 9] =
    [0x65, 0x25, 0x25, 0x65, 0x25, 0x25, 0x65, 0x25, 0x25];

pub(in crate::zelda_rtl::sprite_main_draw) const BOULDER_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x01cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x01ce,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x01ec,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x01ee,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x41ce,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x41cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x41ee,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x41ec,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0xc1ee,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0xc1ec,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0xc1ce,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc1cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x81ec,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x81ee,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x81cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x81ce,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const ZAZAK_DRAW_CHARS: [u8; 8] = [0x82, 0x82, 0x80, 0x84, 0x88, 0x88, 0x86, 0x84];

pub(in crate::zelda_rtl::sprite_main_draw) const ZAZAK_DRAW_FLAGS: [u8; 8] = [0x40, 0, 0, 0, 0x40, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const ZAZAK_DRAW_FRAMES: [DrawMultipleData; 24] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0008,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00a1,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0008,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 1,
        char_flags: 0x40a1,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 1,
        char_flags: 0x40a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x00a3,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 1,
        char_flags: 0x40a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 1,
        char_flags: 0x40a3,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x000c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x000c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x400c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x400c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40a8,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const LEEVER_DRAW_COUNTS: [u8; 14] = [1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const LEEVER_DRAW_X_OFFSETS: [i8; 56] = [
    2, 6, 6, 6, 0, 8, 8, 8, 0, 8, 8, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 0, 0, 8, 0, 0, 0, 8, 0, 0, 0, 8,
    0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const LEEVER_DRAW_Y_OFFSETS: [i8; 56] = [
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 5, 5, 8, 8, 5, 5, 8, 8, 2, 2, 8, 8, 1, 1, 8, 8, 0, 0, 8, 8,
    -1, -1, 8, 8, 8, -2, -2, 0, 8, -2, -2, 0, 8, -2, -2, 0, 8, -2, -2, 0, 8, -2, -2, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const LEEVER_DRAW_CHARS: [u8; 56] = [
    0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x38, 0x38, 0x38, 0x38, 8, 9, 0x28, 0x28, 8, 9,
    0xd9, 0xd9, 8, 8, 0xd8, 0xd8, 8, 8, 0xda, 0xda, 6, 6, 0xd9, 0xd9, 0x26, 0x26, 0xd8, 0xd8, 0x6c,
    6, 6, 0, 0x6c, 0x26, 0x26, 0, 0x6c, 6, 6, 0, 0x6c, 0x26, 0x26, 0, 0x6c, 8, 8, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const LEEVER_DRAW_FLAGS: [u8; 56] = [
    1, 0x41, 0x41, 0x41, 1, 0x41, 0x41, 0x41, 1, 0x41, 0x41, 0x41, 1, 1, 1, 0x41, 1, 1, 0, 0x40, 1,
    1, 0, 0x40, 1, 1, 0, 0x40, 1, 1, 0, 0x40, 0, 1, 0, 0x40, 6, 0x41, 0x41, 0, 6, 0x41, 0x41, 0, 6,
    1, 1, 0, 6, 1, 1, 0, 6, 1, 1, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const LEEVER_DRAW_SIZES: [u8; 56] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 2, 2, 2, 0, 2, 2, 2, 0, 2, 2, 2, 0, 2, 2, 2, 0,
];

// Shared world, item, and enemy draw tables promoted from sprite_main_draw.rs method bodies.
pub(in crate::zelda_rtl::sprite_main_draw) const SURFACE_XY: [i8; 8] = [-32, -24, -16, -8, 8, 16, 24, 32];

pub(in crate::zelda_rtl::sprite_main_draw) const FISH_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x045e,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x045f,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x845e,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x845f,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x445f,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x445e,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0xc45f,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0xc45e,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0461,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0471,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4461,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x4471,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x8471,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x8461,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0xc471,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0xc461,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FISH_DRAW_FRAMES2: [DrawMultipleData; 9] = [
    DrawMultipleData {
        x: -2,
        y: 11,
        char_flags: 0x0438,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0438,
        ext: 0,
    },
    DrawMultipleData {
        x: 2,
        y: 11,
        char_flags: 0x0438,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: 11,
        char_flags: 0x0438,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0438,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: 11,
        char_flags: 0x0438,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0438,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0438,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0438,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const CHIMNEY_SMOKE_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0086,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x0087,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0096,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x0097,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: 1,
        char_flags: 0x0086,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: 1,
        char_flags: 0x0087,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: 7,
        char_flags: 0x0096,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: 7,
        char_flags: 0x0097,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const VULTURE_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4080,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4084,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const RAVEN_ASCEND_TIMERS: [u8; 2] = [16, 248];

pub(in crate::zelda_rtl::sprite_main_draw) const BOMBER_DRAW_FRAMES: [DrawMultipleData; 22] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40c6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40c6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c4,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40c0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40c2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40e0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40e2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e6,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SWAMOLA_TARGET_X_OFFSETS: [i8; 9] = [0, 0, 32, 32, 32, 0, -32, -32, -32];

pub(in crate::zelda_rtl::sprite_main_draw) const SWAMOLA_TARGET_Y_OFFSETS: [i8; 9] = [0, -32, -32, 0, 32, 32, 32, 0, -32];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOROK_SPIT_POSE_BY_DELAY: [u8; 20] =
    [0, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 1, 1, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const FOUR_WAY_OCTOROK_SPIT_POSE_BY_DELAY: [u8; 10] = [2, 2, 2, 2, 2, 2, 2, 2, 1, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const HINOX_RANDOM_DIRECTIONS: [u8; 8] = [2, 3, 3, 2, 0, 1, 1, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const HINOX_BOMB_X_OFFSETS: [i8; 4] = [8, -8, -13, 13];

pub(in crate::zelda_rtl::sprite_main_draw) const HINOX_BOMB_Y_OFFSETS: [i8; 4] = [-11, -11, -16, -16];

pub(in crate::zelda_rtl::sprite_main_draw) const HINOX_BOMB_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const HINOX_BOMB_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(in crate::zelda_rtl::sprite_main_draw) const VULTURE_GRAPHICS: [u8; 4] = [1, 2, 3, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const DEAD_ROCK_GRAPHICS: [u8; 9] = [0, 1, 0, 1, 2, 2, 3, 3, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const DEAD_ROCK_OAM_FLAGS: [u8; 9] = [0x40, 0x40, 0, 0, 0, 0x40, 0, 0x40, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const DEAD_ROCK_X_VELOCITIES: [u8; 4] = [32, 0xe0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const DEAD_ROCK_Y_VELOCITIES: [u8; 4] = [0, 0, 32, 0xe0];

pub(in crate::zelda_rtl::sprite_main_draw) const CUCCO_CALM_CIRCLE_X_VELOCITIES: [i8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const CUCCO_CALM_CIRCLE_Y_VELOCITIES: [i8; 16] = [
    -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ALTAR_ZELDA_HEAD_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

// Generic DrawMultipleData tables promoted from sprite_main_draw.rs draw helpers.
pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_CATFISH_SPLASH_OF_WATER_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: -8,
        y: -4,
        char_flags: 0x0080,
        ext: 0,
    },
    DrawMultipleData {
        x: 18,
        y: -7,
        char_flags: 0x0080,
        ext: 0,
    },
    DrawMultipleData {
        x: -5,
        y: -2,
        char_flags: 0x00bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 15,
        y: -4,
        char_flags: 0x40af,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x00e7,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x00e7,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x00c0,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SWAMOLA_RIPPLES_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00d8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x40d8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00d9,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x40d9,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00da,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x40da,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x00d9,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x40d9,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const CUKEMAN_DRAW_FRAMES: [DrawMultipleData; 18] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x01f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: 0,
        char_flags: 0x41f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 7,
        char_flags: 0x07e0,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: 2,
        char_flags: 0x01f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 6,
        y: 1,
        char_flags: 0x41f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x07e0,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: 1,
        char_flags: 0x01f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 2,
        char_flags: 0x41f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x07e0,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: 0,
        char_flags: 0x01f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 10,
        y: 0,
        char_flags: 0x41f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 7,
        char_flags: 0x07e0,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x01f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x41f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 6,
        char_flags: 0x07e0,
        ext: 0,
    },
    DrawMultipleData {
        x: -5,
        y: 0,
        char_flags: 0x01f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x41f3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x07e0,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SNAP_DRAGON_DRAW_FRAMES: [DrawMultipleData; 32] = [
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x008f,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: -8,
        char_flags: 0x009f,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x008d,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x002b,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: -8,
        char_flags: 0x003b,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x0028,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x0029,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x003c,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x003d,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00ab,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x003e,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x003f,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x00ad,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00ae,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x409f,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x408f,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x408d,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x408c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x403b,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x402b,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x4029,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x4028,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x403d,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: -8,
        char_flags: 0x403c,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x40ab,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x40aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x403f,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: -8,
        char_flags: 0x403e,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x40ae,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x40ad,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const LYNEL_DRAW_FRAMES: [DrawMultipleData; 33] = [
    DrawMultipleData {
        x: -5,
        y: -11,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x00e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00e5,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: -10,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x00e7,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: -11,
        char_flags: 0x00c8,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x00e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00e5,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: -11,
        char_flags: 0x40cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x40e5,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x40e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: -10,
        char_flags: 0x40cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x40e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x40e7,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: -11,
        char_flags: 0x40c8,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x40e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x40e7,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x00ce,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x00ea,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x00ce,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x40eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x40ea,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x00ca,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x40eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -14,
        char_flags: 0x00c6,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x00ed,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00ee,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -14,
        char_flags: 0x00c6,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x40ee,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x40ed,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const KYAMERON_DRAW_FRAMES: [DrawMultipleData; 28] = [
    DrawMultipleData {
        x: 1,
        y: 8,
        char_flags: 0x00b4,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: 8,
        char_flags: 0x00b5,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -3,
        char_flags: 0x0086,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -13,
        char_flags: 0x80a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: 8,
        char_flags: 0x00b4,
        ext: 0,
    },
    DrawMultipleData {
        x: 6,
        y: 8,
        char_flags: 0x00b5,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -6,
        char_flags: 0x0096,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -20,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -1,
        char_flags: 0x0096,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -27,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -27,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -27,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -6,
        char_flags: 0x01df,
        ext: 0,
    },
    DrawMultipleData {
        x: 14,
        y: -6,
        char_flags: 0x41df,
        ext: 0,
    },
    DrawMultipleData {
        x: -6,
        y: 14,
        char_flags: 0x81df,
        ext: 0,
    },
    DrawMultipleData {
        x: 14,
        y: 14,
        char_flags: 0xc1df,
        ext: 0,
    },
    DrawMultipleData {
        x: -6,
        y: -6,
        char_flags: 0x0096,
        ext: 0,
    },
    DrawMultipleData {
        x: 14,
        y: -6,
        char_flags: 0x4096,
        ext: 0,
    },
    DrawMultipleData {
        x: -6,
        y: 14,
        char_flags: 0x8096,
        ext: 0,
    },
    DrawMultipleData {
        x: 14,
        y: 14,
        char_flags: 0xc096,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x018d,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: -4,
        char_flags: 0x418d,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 12,
        char_flags: 0x818d,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 12,
        char_flags: 0xc18d,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x018d,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x418d,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x818d,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc18d,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const GREAT_CATFISH_DRAW_FRAMES: [DrawMultipleData; 28] = [
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x008d,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x008d,
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
        char_flags: 0x008d,
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
        char_flags: 0x009d,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -4,
        char_flags: 0x408d,
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
        char_flags: 0x409d,
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
        char_flags: 0xc09d,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -4,
        char_flags: 0xc09c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0xc08d,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0xc08c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0xc09d,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0xc09c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 4,
        char_flags: 0xc09d,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0xc09c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x00bd,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40bd,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40bd,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40bd,
        ext: 0,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4086,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const PIKIT_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ca,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00ce,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40ce,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const MOBLIN_DRAW_FRAMES: [DrawMultipleData; 48] = [
    DrawMultipleData {
        x: -2,
        y: 3,
        char_flags: 0x8091,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: 11,
        char_flags: 0x8090,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: 7,
        char_flags: 0x8091,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: 15,
        char_flags: 0x8090,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x408a,
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
        x: 11,
        y: -5,
        char_flags: 0x0090,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: 3,
        char_flags: 0x0091,
        ext: 0,
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
        char_flags: 0x40a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 11,
        y: -8,
        char_flags: 0x0090,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: 0,
        char_flags: 0x0091,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x0080,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0081,
        ext: 0,
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
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: -9,
        y: 6,
        char_flags: 0x0080,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: 6,
        char_flags: 0x0081,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
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
        x: 12,
        y: 8,
        char_flags: 0x4080,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x4081,
        ext: 0,
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
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 17,
        y: 6,
        char_flags: 0x4080,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: 6,
        char_flags: 0x4081,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
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
        x: -3,
        y: -5,
        char_flags: 0x8091,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 3,
        char_flags: 0x8090,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 11,
        y: -11,
        char_flags: 0x0090,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: -3,
        char_flags: 0x0091,
        ext: 0,
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
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: -3,
        char_flags: 0x0080,
        ext: 0,
    },
    DrawMultipleData {
        x: 6,
        y: -3,
        char_flags: 0x0081,
        ext: 0,
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
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 10,
        y: -3,
        char_flags: 0x4080,
        ext: 0,
    },
    DrawMultipleData {
        x: 2,
        y: -3,
        char_flags: 0x4081,
        ext: 0,
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
        char_flags: 0x40a2,
        ext: 2,
    },
];

// ---------------------------------------------------------------------------
// Promoted sprite_main_draw method-local tables. Names retain their owning
// helper to keep generic C table names such as X/Y/FLAGS readable at callsites.
// ---------------------------------------------------------------------------

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_DRAW_X_OFFSETS: [i8; 26] = [
    4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -4, 11, 0, 4, -8, 18, -8, 18,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_DRAW_Y_OFFSETS: [i8; 26] = [
    4, 4, 0, 0, 0, 0, 0, -3, 0, -3, -3, -3, -3, -3, -3, -3, -6, -6, -8, -9, -3, 5, -10, -11, -10,
    -11,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_DRAW_CHARS: [u8; 26] = [
    0xa8, 0xa8, 0x88, 0x88, 0x88, 0x88, 0x88, 0xa4, 0x88, 0xa4, 0xa4, 0xa4, 0xa6, 0xa6, 0xa4, 0xc0,
    0x8a, 0x8a, 0xae, 0xaf, 0xa6, 0x8d, 0xcf, 0xcf, 0xdf, 0xdf,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_DRAW_FLAGS: [u8; 26] = [
    0x25, 0x25, 0x25, 0x25, 0xe5, 0xe5, 0x25, 0x20, 0xe5, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x24,
    0x25, 0x25, 0x24, 0x64, 0x20, 0x26, 0x24, 0x64, 0x24, 0x64,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_DRAW_BIG: [u8; 26] = [
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 2, 0, 0, 0, 0, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_FIREBALL_OFFSETS: [u8; 4] = [3, 2, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_FIREBALL_X_OFFSETS: [i8; 4] = [4, 4, -4, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_FIREBALL_Y_OFFSETS: [i8; 4] = [0, 16, 8, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_FIREBALL_WIDTHS: [u8; 4] = [8, 8, 4, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_FIREBALL_HEIGHTS: [u8; 4] = [4, 4, 8, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_ZORA_MAIN_ATTACK_GFX: [u8; 8] = [5, 5, 6, 10, 6, 5, 5, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_ZORA_MAIN_SUBMERGE_GFX: [u8; 12] = [12, 11, 9, 8, 7, 0, 0, 0, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_SPAWN_BIG_SPLASH_X_OFFSETS: [i8; 8] = [-8, -5, 4, 13, 16, 13, 4, -5];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_SPAWN_BIG_SPLASH_Y_OFFSETS: [i8; 8] = [4, -5, -8, -5, 4, 13, 16, 13];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_SPAWN_BIG_SPLASH_LOCAL_X_VELOCITIES: [i8; 8] = [-8, -6, 0, 6, 8, 6, 0, -6];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_SPAWN_BIG_SPLASH_LOCAL_Y_VELOCITIES: [i8; 8] = [0, -6, -8, -6, 0, 6, 8, 6];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_KING_DRAW_X_OFFSETS_1: [i8; 8] = [-23, 23, 23, 23, -20, -15, 13, 18];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_KING_DRAW_Y_OFFSETS_1: [i8; 8] = [-8, -8, -8, -8, -7, 0, 0, -7];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_KING_DRAW_CHARS_1: [u8; 8] = [0xae, 0xae, 0xae, 0xae, 0xac, 0xac, 0xac, 0xac];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_KING_DRAW_FLAGS_1: [u8; 8] = [0, 0x40, 0x40, 0x40, 0, 0, 0x40, 0x40];

pub(in crate::zelda_rtl::sprite_main_draw) const WALKING_ZORA_DRAW_CHARS: [u8; 4] = [0xce, 0xce, 0xa4, 0xee];

pub(in crate::zelda_rtl::sprite_main_draw) const WALKING_ZORA_DRAW_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const WALKING_ZORA_DRAW_CHARS_2: [u8; 8] =
    [0xcc, 0xec, 0xcc, 0xec, 0xe8, 0xe8, 0xca, 0xca];

pub(in crate::zelda_rtl::sprite_main_draw) const WALKING_ZORA_DRAW_FLAGS_2: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0x40, 0, 0x40];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_CF_SWAMOLA_TARGET_DIR: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const SWAMOLA_DRAW_LOCAL_GRAPHICS: [u8; 16] =
    [7, 6, 5, 4, 3, 4, 5, 6, 7, 6, 5, 4, 3, 4, 5, 6];

pub(in crate::zelda_rtl::sprite_main_draw) const SWAMOLA_DRAW_SECONDARY_GRAPHICS: [u8; 4] = [0, 0, 1, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SWAMOLA_DRAW_FLAGS: [u8; 16] = [
    0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SWAMOLA_DRAW_HIST_OFFS: [u8; 4] = [8, 16, 22, 26];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_D0_LYNEL_ATTACK_GFX: [u8; 4] = [5, 2, 8, 10];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_D0_LYNEL_LOCAL_GRAPHICS: [u8; 8] = [3, 0, 6, 9, 4, 1, 7, 10];

pub(in crate::zelda_rtl::sprite_main_draw) const BUZZ_BLOB_DRAW_X_OFFSETS: [u16; 3] = [0, 8, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const BUZZ_BLOB_DRAW_Y_OFFSETS: [i16; 3] = [-8, -8, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const BUZZ_BLOB_DRAW_CHARS: [u8; 18] = [
    0xf0, 0xf0, 0xe1, 0, 0, 0xce, 0, 0, 0xce, 0xe3, 0xe3, 0xca, 0xe4, 0xe5, 0xcc, 0xe5, 0xe4, 0xcc,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BUZZ_BLOB_DRAW_FL: [u8; 18] = [
    0, 0x40, 0, 0, 0, 0, 0, 0, 0x40, 0, 0x40, 0, 0, 0, 0, 0x40, 0x40, 0x40,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BUZZ_BLOB_DRAW_EXT: [u8; 3] = [0, 0, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const KYAMERON_DRAW_FLAGS: [u8; 12] = [0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0xc0, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const PIROGUSU_DRAW_FLAGS: [u8; 28] = [
    0, 0x80, 0x40, 0, 0, 0, 0, 0x80, 0x80, 0xc0, 0x40, 0x40, 0, 0x40, 0x80, 0xc0, 0x40, 0xc0, 0,
    0x80, 0, 0x40, 0x80, 0xc0, 0x40, 0xc0, 0, 0x80,
];

pub(in crate::zelda_rtl::sprite_main_draw) const PIROGUSU_DRAW_LOCAL_GRAPHICS: [u8; 28] = [
    0, 0, 1, 1, 2, 3, 4, 3, 2, 3, 4, 3, 5, 5, 5, 5, 7, 7, 7, 7, 6, 6, 6, 6, 8, 8, 8, 8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_HOARDER_COVERED_LOCAL_GRAPHICS: [u8; 4] = [3, 4, 5, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_HOARDER_COVERED_LOCAL_X_VELOCITIES: [i8; 4] = [-12, 12, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_HOARDER_COVERED_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, -12, 12];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_HOARDER_FRANTIC_LOCAL_GRAPHICS: [u8; 4] = [0, 1, 0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_HOARDER_FRANTIC_OAM_FLAGS: [u8; 4] = [0, 0, 0x40, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_HOARDER_FRANTIC_LOCAL_X_VELOCITIES: [i8; 4] = [-16, 16, -16, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_HOARDER_FRANTIC_LOCAL_Y_VELOCITIES: [i8; 4] = [-16, -16, 16, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const COVERED_RUPEE_CRAB_DRAW_Y_OFFSETS: [i8; 12] =
    [0, 0, 0, -3, 0, -5, 0, -6, 0, -6, 0, -6];

pub(in crate::zelda_rtl::sprite_main_draw) const COVERED_RUPEE_CRAB_DRAW_CHARS: [u8; 12] = [
    0x44, 0x44, 0xe8, 0x44, 0xe8, 0x44, 0xe6, 0x44, 0xe8, 0x44, 0xe6, 0x44,
];

pub(in crate::zelda_rtl::sprite_main_draw) const COVERED_RUPEE_CRAB_DRAW_FL: [u8; 12] =
    [0, 0x0c, 3, 0x0c, 3, 0x0c, 3, 0x0c, 3, 0x0c, 0x43, 0x0c];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_0_F_OCTOBALLOON_Z: [u8; 8] = [16, 17, 18, 19, 20, 19, 18, 17];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_0_D_BUZZBLOB_LOCAL_GRAPHICS: [u8; 4] = [0, 1, 0, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_0_D_BUZZBLOB_OBJ_PRIO: [u8; 4] = [10, 2, 8, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_08_OCTOROK_NEXT_DIR: [u8; 4] = [2, 3, 1, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_08_OCTOROK_DIRECTIONS: [u8; 4] = [3, 2, 0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_08_OCTOROK_LOCAL_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_08_OCTOROK_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_08_OCTOROK_OAM_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_0_E_SNAPDRAGON_DELAY: [u8; 4] = [0x20, 0x30, 0x40, 0x50];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_0_E_SNAPDRAGON_LOCAL_GRAPHICS: [u8; 4] = [4, 0, 6, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_0_E_SNAPDRAGON_LOCAL_X_VELOCITIES: [i8; 8] =
    [8, -8, 8, -8, 16, -16, 16, -16];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_0_E_SNAPDRAGON_LOCAL_Y_VELOCITIES: [i8; 8] =
    [8, 8, -8, -8, 16, 16, -16, -16];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_11_HINOX_WALK_GFX: [u8; 4] = [6, 4, 0, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_11_HINOX_LOCAL_GRAPHICS: [u8; 8] = [11, 10, 8, 9, 7, 5, 1, 3];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_20_SLUGGULA_LOCAL_GRAPHICS: [u8; 8] = [0, 1, 0, 1, 2, 3, 4, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_20_SLUGGULA_OAM_FLAGS: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_20_SLUGGULA_XYVEL: [i8; 6] = [16, -16, 0, 0, 16, -16];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_4_D_TOPPO_X_OFFS: [i8; 4] = [-32, 32, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_4_D_TOPPO_Y_OFFS: [i8; 4] = [0, 0, -32, 32];

pub(in crate::zelda_rtl::sprite_main_draw) const TOPPO_DRAW_X_OFFSETS: [i8; 15] = [0, 8, 8, 0, 8, 8, 0, 0, 8, 0, 0, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const TOPPO_DRAW_Y_OFFSETS: [i8; 15] = [8, 8, 8, 8, 8, 8, 0, 8, 8, 0, 0, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const TOPPO_DRAW_CHARS: [u8; 15] = [
    0xc8, 0xc8, 0xc8, 0xca, 0xca, 0xca, 0xc0, 0xc8, 0xc8, 0xc2, 0xc2, 0xc2, 0xc2, 0xc2, 0xc2,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TOPPO_DRAW_FLAGS: [u8; 15] = [
    0, 0x40, 0x40, 0, 0x40, 0x40, 0, 0, 0x40, 0, 0, 0, 0x40, 0x40, 0x40,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TOPPO_DRAW_BIG: [u8; 15] = [0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 2, 2, 2, 2, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_4_C_GELDMAN_EMERGE_GFX: [u8; 8] = [3, 2, 0, 0, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_4_C_GELDMAN_PURSUE_GFX: [u8; 2] = [4, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_4_C_GELDMAN_SUBMERGE_GFX: [u8; 5] = [0, 1, 2, 3, 3];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_58_CRAB_LOCAL_X_VELOCITIES: [i8; 4] = [28, -28, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_58_CRAB_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 12, -12];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_71_LEEVER_EMERGE_GFX: [u8; 16] =
    [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 2, 1, 2, 1, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_71_LEEVER_ATTACK_GFX: [u8; 4] = [9, 10, 11, 12];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_71_LEEVER_ATTACK_SPD: [u8; 2] = [12, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_71_LEEVER_SUBMERGE_GFX: [u8; 16] =
    [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 2, 1, 2, 1, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_12_MOBLIN_LOCAL_X_VELOCITIES: [i8; 4] = [16, -16, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_12_MOBLIN_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 16, -16];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_12_MOBLIN_DELAY: [u8; 4] = [0x10, 0x20, 0x30, 0x40];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_12_MOBLIN_SECONDARY_GRAPHICS: [u8; 8] = [11, 10, 8, 9, 7, 5, 0, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_12_MOBLIN_DIRS: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_12_MOBLIN_LOCAL_GRAPHICS: [u8; 4] = [6, 4, 0, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOBALLOON_DRAW_X_OFFSETS: [i8; 12] = [-4, 4, -4, 4, -8, 8, -8, 8, -4, 4, -4, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOBALLOON_DRAW_Y_OFFSETS: [i8; 12] = [-4, -4, 4, 4, -8, -8, 8, 8, -4, -4, 4, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOBALLOON_DRAW_CHARS: [u8; 12] = [
    0x8c, 0x8c, 0x9c, 0x9c, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86,
];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOBALLOON_DRAW_FL: [u8; 12] =
    [0, 0x40, 0, 0x40, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0];

pub(in crate::zelda_rtl::sprite_main_draw) const MOBLIN_DRAW_OBJ_OFFS: [u8; 12] = [2, 2, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const MOBLIN_DRAW_HEAD_CHAR: [u8; 4] = [0x88, 0x88, 0x86, 0x84];

pub(in crate::zelda_rtl::sprite_main_draw) const TEKTITE_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00c8,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40c8,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40ca,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00ea,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40ea,
        ext: 2,
    },
];

// ---------------------------------------------------------------------------
// Promoted SpriteDraw method-local tables. Names retain the owning helper so
// generic C table names stay readable at callsites.
// ---------------------------------------------------------------------------

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_52_KING_ZORA_SURFACING_GFX: [u8; 16] =
    [0, 0, 0, 3, 9, 8, 7, 6, 9, 8, 7, 6, 5, 4, 5, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_52_KING_ZORA_DIALOGUE_GFX: [u8; 8] = [0, 0, 1, 2, 1, 2, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_52_KING_ZORA_SUBMERGE_GFX: [u8; 21] = [
    12, 12, 12, 12, 12, 12, 11, 11, 11, 11, 11, 10, 10, 10, 10, 3, 3, 3, 3, 3, 3,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_KING_DRAW_X_OFFSETS_0: [i8; 52] = [
    -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, 0, 0, 0, 0, 0, 0, 0, 0, -8, 8, -8, 8,
    -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -9, 9, -9, 9, -10, 10, -10, 10, -11, 11, -11, 11,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_KING_DRAW_Y_OFFSETS_0: [i8; 52] = [
    -18, -18, -2, -2, -18, -18, -2, -2, -18, -18, -2, -2, -12, -12, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0,
    -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -5, -5, 5, 5, -5, -5, 5, 5, -5, -5, 5,
    5,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_KING_DRAW_CHARS_0: [u8; 52] = [
    0xc0, 0xc0, 0xe0, 0xe0, 0xc2, 0xea, 0xe2, 0xe2, 0xea, 0xc2, 0xe2, 0xe2, 0xc0, 0xc0, 0xe4, 0xe6,
    0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0xc4, 0xc6, 0xe4, 0xe6, 0xc6, 0xc4, 0xe6, 0xe4,
    0xe6, 0xe4, 0xc6, 0xc4, 0xe4, 0xe6, 0xc4, 0xc6, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88,
    0x88, 0x88, 0x88, 0x88,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ZORA_KING_DRAW_FLAGS_0: [u8; 52] = [
    0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 5, 5, 5, 5, 5, 5, 0xc5, 0xc5,
    0xc5, 0xc5, 5, 5, 5, 5, 0x45, 0x45, 0x45, 0x45, 0xc5, 0xc5, 0xc5, 0xc5, 0x85, 0x85, 0x85, 0x85,
    4, 0x44, 0x84, 0xc4, 4, 0x44, 0x84, 0xc4, 4, 0x44, 0x84, 0xc4,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_CATFISH_QUAKE_MEDALLION_Z_VELOCITIES: [u8; 4] = [0x20, 0x10, 8, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const CATFISH_BIG_FISH_EMERGE_GFX: [u8; 16] =
    [1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 0, 0, 0, 0];

// $1D:E0BF through E0D3. Index 20 reaches the first instruction byte
// following the named table when the outdoor splash clears carry.
pub(in crate::zelda_rtl::sprite_main_draw) const CATFISH_BIG_FISH_CONVERSATE_GFX: [u8; 21] = [
    0, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6, 6, 0xbd,
];

// GreatCatfish_Draw computes E240 + (BD - 1) * 32 = F9C0. Preserve
// the source's full 16-bit coordinates and eighth-byte extended attributes.
pub(in crate::zelda_rtl::sprite_main_draw) const GREAT_CATFISH_SPLASH_CARRY_DRAW: [[u8; 8]; 4] = [
    [0xf0, 0x0d, 0xc9, 0x07, 0xd0, 0x03, 0x20, 0x4e],
    [0xfa, 0xbd, 0xf0, 0x0d, 0xd0, 0x03, 0xfe, 0x80],
    [0x0d, 0x60, 0x01, 0x02, 0x03, 0x01, 0x03, 0x01],
    [0x02, 0x03, 0x0d, 0x0d, 0x0d, 0x0b, 0x0b, 0x06],
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_CF_SWAMOLA_Z_ACCEL: [i8; 2] = [2, -2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_CF_SWAMOLA_Z_VEL_TARGET: [i8; 2] = [12, -12];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_D0_LYNEL_X_TARGETS: [i8; 4] = [-96, 96, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_D0_LYNEL_Y_TARGETS: [i8; 4] = [8, 8, -96, 112];
