//! Scenery, switches, items, shops, projectiles, and hazard effect tables.

use crate::zelda_rtl::sprite::DrawMultipleData;
// ---------------------------------------------------------------------------
// File-local RAM offsets. The canonical zelda_rtl.rs already exports the
// frequently-shared ones; these are the remaining `variables.h` addresses
// referenced by SpriteDraw_*. Local names use `_DRAW` so they cannot clash
// with constants other sprite_main_* modules declare for their own use.
// ---------------------------------------------------------------------------

// variables.h:672 — sprite_ignore_projectile.
// variables.h:721..722 — sprite_B / sprite_C live in pages 0xDA0..0xDC0.
// variables.h:727..728 — sprite_delay_aux1 / aux2.
// variables.h:737 — sprite_F.
// variables.h:739..742 — sprite_anim_clock / sprite_G / sprite_delay_aux3 /
// sprite_hit_timer.
// variables.h:743 — sprite_y_recoil.
// variables.h:776 — light/dark world flag.
// variables.h:758..761 — repulsespark_*.
// variables.h:755 — sprite_tiletype.
// overlord.c stores the active overlord slot here before sprite code consumes it.
// variables.h:766..768 — garnish_active / tmp_counter / shared draw scratch.
// variables.h:910 — sram_progress_indicator_3.
// hud.c private table mirrored for bomb shop purchase gating.
pub(in crate::zelda_rtl::sprite_main_draw) const MAX_BOMBS_FOR_LEVEL_DRAW: [u8; 8] = [10, 15, 20, 25, 30, 35, 40, 50];
// hud.h:10 — kHudItem_BookMudora.
pub(in crate::zelda_rtl::sprite_main_draw) const HUD_ITEM_BOOK_MUDORA_DRAW: u8 = 15;
// player.h:30 — kPlayerState_OpeningDesertPalace.
pub(in crate::zelda_rtl::sprite_main_draw) const PLAYER_STATE_OPENING_DESERT_PALACE_DRAW: u8 = 27;

// ---------------------------------------------------------------------------
// kLargeShadow_Dmd from sprite_main.c:373 — 15-entry shadow table used by
// SpriteDraw_BigShadow.
// ---------------------------------------------------------------------------
pub(in crate::zelda_rtl::sprite_main_draw) const LARGE_SHADOW_DRAW_FRAMES: [DrawMultipleData; 15] = [
    DrawMultipleData {
        x: -6,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: 19,
        char_flags: 0x086c,
        ext: 2,
    },
];
pub(in crate::zelda_rtl::sprite_main_draw) const ABSORB_BIG_KEY_MASKS_DRAW: [u16; 2] = [0x40, 0x20];
pub(in crate::zelda_rtl::sprite_main_draw) const HEART_REFILL_X_ACCELERATIONS: [i8; 2] = [1, -1];
pub(in crate::zelda_rtl::sprite_main_draw) const HEART_REFILL_X_VELOCITY_TARGETS: [i8; 2] = [10, -10];
pub(in crate::zelda_rtl::sprite_main_draw) const PLAYER_DAMAGE_NONZERO_MASK_DRAW: u8 = 2;

// kThrowableScenery_DrawLarge_* from sprite_main.c:137-141.
pub(in crate::zelda_rtl::sprite_main_draw) const THROWABLE_SCENERY_LARGE_X_OFFSETS: [i16; 4] = [-8, 8, -8, 8];
pub(in crate::zelda_rtl::sprite_main_draw) const THROWABLE_SCENERY_LARGE_Y_OFFSETS: [i16; 4] = [-14, -14, 2, 2];
pub(in crate::zelda_rtl::sprite_main_draw) const THROWABLE_SCENERY_LARGE_DRAW_FLAGS: [u8; 4] = [0, 0x40, 0x80, 0xc0];
pub(in crate::zelda_rtl::sprite_main_draw) const THROWABLE_SCENERY_LARGE_EXTRA_X_OFFSETS: [i16; 3] = [-6, 0, 6];
pub(in crate::zelda_rtl::sprite_main_draw) const THROWABLE_SCENERY_LARGE_OAM_FLAGS: [u8; 2] = [0xc, 0];
pub(in crate::zelda_rtl::sprite_main_draw) const SCATTER_DEBRIS_X_OFFSETS: [i16; 4] = [-8, 8, -8, 8];
pub(in crate::zelda_rtl::sprite_main_draw) const SCATTER_DEBRIS_Y_OFFSETS: [i16; 4] = [-8, -8, 8, 8];

// Shared draw tables promoted from sprite_main_draw.rs method bodies.
pub(in crate::zelda_rtl::sprite_main_draw) const MASTER_SWORD_LIGHT_BALL_DRAW_FRAMES: [DrawMultipleData; 12] = [
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0xc082,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0x8082,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0x40a0,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0xc0a0,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0x80a0,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0x4080,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0xc080,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 4,
        char_flags: 0x8080,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const BEAMOS_EYEBALL_DRAW_X_OFFSETS: [i8; 32] = [
    -1, 0, 1, 2, 3, 4, 5, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17, 15, 14, 13, 12, 11, 10, 8, 7, 5, 4,
    3, 2, 1, 0, -2,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BEAMOS_EYEBALL_DRAW_Y_OFFSETS: [i8; 32] = [
    11, 12, 13, 14, 14, 15, 15, 15, 15, 15, 15, 14, 14, 13, 12, 11, 10, 9, 8, 7, 7, 6, 6, 6, 6, 6,
    6, 7, 7, 8, 9, 10,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BEAMOS_EYEBALL_DRAW_CHARS: [u8; 32] = [
    0x5b, 0x5b, 0x5a, 0x5a, 0x4b, 0x4b, 0x4a, 0x4a, 0x4a, 0x4a, 0x4b, 0x4b, 0x5a, 0x5a, 0x5b, 0x5b,
    0x5b, 0x5b, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c, 0x5b, 0x5b, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BEAMOS_EYEBALL_DRAW_FLAGS: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOSTONE_DRAW_X_OFFSETS: [i8; 16] = [
    0, 8, 0, 8, -8, 16, -8, 16, -12, 20, -12, 20, -14, 22, -14, 22,
];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOSTONE_DRAW_Y_OFFSETS: [i8; 16] = [
    0, 0, 8, 8, -8, -8, 16, 16, -12, -12, 20, 20, -14, -14, 22, 22,
];

pub(in crate::zelda_rtl::sprite_main_draw) const OCTOSTONE_DRAW_FLAGS: [u8; 16] = [
    0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const DRAW_FOUR_AROUND_ONE_DRAW_FRAMES: [DrawMultipleData; 30] = [
    DrawMultipleData {
        x: 4,
        y: 2,
        char_flags: 0x02e1,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -3,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: 2,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: 2,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 7,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 2,
        char_flags: 0x02e1,
        ext: 0,
    },
    DrawMultipleData {
        x: 3,
        y: -3,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: 1,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: 3,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: 7,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 2,
        char_flags: 0x02e1,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: -3,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: -1,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: 5,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: 7,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 2,
        char_flags: 0x02e1,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -2,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -2,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 6,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 6,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 2,
        char_flags: 0x02e1,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: -3,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: -1,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: 5,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: 7,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 2,
        char_flags: 0x02e1,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: -3,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: 1,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: 3,
        char_flags: 0x02e3,
        ext: 0,
    },
    DrawMultipleData {
        x: 3,
        y: 7,
        char_flags: 0x02e3,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SHOP_KEEPER_ITEM_WITH_PRICE_DRAW_FRAMES: [DrawMultipleData; 35] = [
    DrawMultipleData {
        x: -4,
        y: 16,
        char_flags: 0x0231,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 16,
        char_flags: 0x0213,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 16,
        char_flags: 0x0230,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x02c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x036c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x0213,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x0213,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 16,
        char_flags: 0x0230,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x04ce,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 12,
        char_flags: 0x0338,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 16,
        char_flags: 0x0213,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 16,
        char_flags: 0x0230,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 16,
        char_flags: 0x0230,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x08cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 12,
        char_flags: 0x0338,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x0231,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x0231,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 16,
        char_flags: 0x0230,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0329,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 11,
        char_flags: 0x0338,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 16,
        char_flags: 0x0203,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 16,
        char_flags: 0x0203,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 16,
        char_flags: 0x0230,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x04c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0338,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x0213,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x0213,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 16,
        char_flags: 0x0230,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x04e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x036c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x0231,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x0231,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 16,
        char_flags: 0x0230,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0ff4,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 11,
        char_flags: 0x0338,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const BEAMOS_DRAW_Y_OFFSETS: [i8; 2] = [-16, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const BEAMOS_DRAW_CHARS: [u8; 2] = [0x48, 0x68];

pub(in crate::zelda_rtl::sprite_main_draw) const DUST_CLOUD_DRAW_FRAMES: [DrawMultipleData; 24] = [
    DrawMultipleData {
        x: 0,
        y: -3,
        char_flags: 0x008b,
        ext: 0,
    },
    DrawMultipleData {
        x: 3,
        y: 0,
        char_flags: 0x009b,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 0,
        char_flags: 0xc08b,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 3,
        char_flags: 0xc09b,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 0,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 0,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 5,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: 0,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: 0,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 7,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x8086,
        ext: 2,
    },
    DrawMultipleData {
        x: 9,
        y: 0,
        char_flags: 0x8086,
        ext: 2,
    },
    DrawMultipleData {
        x: -9,
        y: 0,
        char_flags: 0x8086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x8086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0xc086,
        ext: 2,
    },
    DrawMultipleData {
        x: 9,
        y: 0,
        char_flags: 0xc086,
        ext: 2,
    },
    DrawMultipleData {
        x: -9,
        y: 0,
        char_flags: 0xc086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0xc086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: 0,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: 0,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 7,
        char_flags: 0x4086,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const DUST_CLOUD_GRAPHICS: [u8; 9] = [0, 1, 2, 3, 4, 5, 1, 0, 0xff];

pub(in crate::zelda_rtl::sprite_main_draw) const LANDMINE_DRAW_FRAMES: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x0070,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 4,
        char_flags: 0x4070,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FLAME_DRAW_FRAMES: [DrawMultipleData; 12] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x018e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x018e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x01a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x01a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x418e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x418e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x41a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x41a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x01a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x01a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -6,
        char_flags: 0x01a4,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -6,
        char_flags: 0x01a5,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const ENERGY_BALL_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 4,
        y: -3,
        char_flags: 0x00ce,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: 4,
        char_flags: 0x00ce,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 11,
        char_flags: 0x00ce,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 4,
        char_flags: 0x00ce,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: -1,
        char_flags: 0x00ce,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: -1,
        char_flags: 0x00ce,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: 9,
        char_flags: 0x00ce,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: 9,
        char_flags: 0x00ce,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const WIZZBEAM_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x00c5,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x80c5,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x40c5,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0xc0c5,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x40d2,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00d2,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0xc0d2,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x80d2,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const PENGATOR_DRAW_FRAMES0: [DrawMultipleData; 40] = [
    DrawMultipleData {
        x: -1,
        y: -8,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: -1,
        y: -7,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: -6,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -4,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00a3,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -8,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4088,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x408a,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: -6,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4088,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: -4,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x408a,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x40a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x40a3,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x4080,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -1,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x408e,
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
        y: 0,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4084,
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
        y: -7,
        char_flags: 0x0084,
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
        y: 0,
        char_flags: 0x408c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -6,
        char_flags: 0x4084,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40a0,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const PENGATOR_DRAW_FRAMES1: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x00b5,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 16,
        char_flags: 0x40b5,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00a5,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x40a5,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPIKE_ROLLER_DRAW_X_OFFSETS: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0,
    0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPIKE_ROLLER_DRAW_Y_OFFSETS: [u8; 32] = [
    0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPIKE_ROLLER_DRAW_CHARS: [u8; 32] = [
    0x8e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e, 0x8e, 0x8e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e, 0x8e,
    0x88, 0x89, 0x89, 0x89, 0x89, 0x89, 0x89, 0x88, 0x88, 0x89, 0x89, 0x89, 0x89, 0x89, 0x89, 0x88,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPIKE_ROLLER_DRAW_FLAGS: [u8; 32] = [
    0, 0, 0, 0x80, 0, 0, 0, 0x80, 0x40, 0x40, 0x40, 0xc0, 0x40, 0x40, 0x40, 0xc0, 0, 0, 0, 0x40, 0,
    0, 0, 0x40, 0x80, 0x80, 0x80, 0xc0, 0x80, 0x80, 0x80, 0xc0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const MEDALLION_TABLET_DRAW_FRAMES: [DrawMultipleData; 20] = [
    DrawMultipleData {
        x: -8,
        y: -16,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -16,
        char_flags: 0x408c,
        ext: 2,
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
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -13,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -13,
        char_flags: 0x408a,
        ext: 2,
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
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x408a,
        ext: 2,
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
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -4,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -4,
        char_flags: 0x408a,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40aa,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40aa,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40aa,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_STATUE_DIRECTIONS: [u8; 4] = [4, 6, 0, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_STATUE_JOYPAD_MASKS: [u8; 4] = [1, 2, 4, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_STATUE_X_VELOCITIES: [u8; 4] = [0xf0, 16, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_STATUE_Y_VELOCITIES: [u8; 4] = [0, 0, 0xf0, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_STATUE_SWITCH_X_OFFSETS: [u8; 4] = [3, 12, 3, 12];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_STATUE_SWITCH_Y_OFFSETS: [u8; 4] = [3, 3, 12, 12];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_STATUE_DRAW_FRAMES: [DrawMultipleData; 3] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00c2,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x40c2,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c0,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_MANTLE_X_OFFSETS: [u8; 6] = [0, 0x10, 0x20, 0, 0x10, 0x20];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_MANTLE_Y_OFFSETS: [u8; 6] = [0, 0, 0, 0x10, 0x10, 0x10];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_MANTLE_CHARS: [u8; 6] = [0x0c, 0x0e, 0x0c, 0x2c, 0x2e, 0x2c];

pub(in crate::zelda_rtl::sprite_main_draw) const MOVABLE_MANTLE_FLAGS: [u8; 6] = [0x31, 0x31, 0x71, 0x31, 0x31, 0x71];

pub(in crate::zelda_rtl::sprite_main_draw) const MAGIC_POWDER_DRAW_FRAMES: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x04e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x04e6,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const GREEN_POTION_ITEM_DRAW_FRAMES: [DrawMultipleData; 3] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x08c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 18,
        char_flags: 0x0a30,
        ext: 0,
    },
    DrawMultipleData {
        x: -1,
        y: 18,
        char_flags: 0x0a22,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const BLUE_POTION_ITEM_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x04c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 13,
        y: 18,
        char_flags: 0x0a30,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: 18,
        char_flags: 0x0a22,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 18,
        char_flags: 0x0a31,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const RED_POTION_ITEM_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x02c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 13,
        y: 18,
        char_flags: 0x0a30,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: 18,
        char_flags: 0x0a02,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 18,
        char_flags: 0x0a31,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const UP_PULL_ANIMATION_DELAYS: [u8; 10] = [8, 24, 4, 4, 4, 4, 4, 4, 2, 10];

pub(in crate::zelda_rtl::sprite_main_draw) const UP_PULL_PLAYER_ACTION_STATES: [u8; 10] = [6, 7, 8, 8, 8, 8, 8, 9, 9, 9];

pub(in crate::zelda_rtl::sprite_main_draw) const DOWN_PULL_ANIMATION_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const DOWN_PULL_PLAYER_ACTION_STATES: [u8; 12] = [1, 1, 2, 2, 3, 3, 1, 1, 4, 4, 5, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const BAD_PULL_DOWN_SWITCH_X_OFFSETS: [i8; 5] = [-4, 12, 0, -4, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const BAD_PULL_DOWN_SWITCH_Y_OFFSETS: [i8; 5] = [-3, -3, 0, 5, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const BAD_PULL_DOWN_SWITCH_DRAW_CHARS: [u8; 5] = [0xd2, 0xd2, 0xc4, 0xe4, 0xe4];

pub(in crate::zelda_rtl::sprite_main_draw) const BAD_PULL_DOWN_SWITCH_DRAW_FLAGS: [u8; 5] = [0x40, 0, 0, 0x40, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const BAD_PULL_DOWN_SWITCH_DRAW_SIZES: [u8; 5] = [0, 0, 2, 2, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const BAD_PULL_SWITCH_CENTER_Y_OFFSETS: [u8; 6] = [0, 1, 2, 3, 4, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const BAD_PULL_UP_SWITCH_CHARS: [u8; 2] = [0xa2, 0xa4];

pub(in crate::zelda_rtl::sprite_main_draw) const BAD_PULL_SWITCH_TOP_Y_OFFSETS: [u8; 6] = [0, 1, 2, 3, 4, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const GOOD_PULL_SWITCH_BOTTOM_Y_OFFSETS: [u8; 14] =
    [1, 1, 2, 3, 2, 3, 4, 5, 6, 7, 6, 7, 7, 7];

pub(in crate::zelda_rtl::sprite_main_draw) const BUMPER_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x00ec,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x40ec,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x80ec,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc0ec,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: -7,
        char_flags: 0x00ec,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: -7,
        char_flags: 0x40ec,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: 7,
        char_flags: 0x80ec,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: 7,
        char_flags: 0xc0ec,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FAKE_SWORD_DRAW_FRAMES: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x00f4,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x00f5,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const RABBIT_BEAM_GFX: [u8; 6] = [0xd7, 0xd7, 0xd7, 0x91, 0x91, 0x91];

pub(in crate::zelda_rtl::sprite_main_draw) const HEART_PIECE_MESSAGES: [u16; 4] = [0x158, 0x155, 0x156, 0x157];

pub(in crate::zelda_rtl::sprite_main_draw) const FIREBAT_DRAW_X_OFFSETS: [i8; 2] = [-8, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const FIREBAT_DRAW_CHARS: [u8; 7] = [0x88, 0x88, 0x8a, 0x8c, 0x68, 0xaa, 0xa8];

pub(in crate::zelda_rtl::sprite_main_draw) const FIREBAT_DRAW_FLAGS: [u8; 14] = [
    0, 0xc0, 0x80, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40,
];

pub(in crate::zelda_rtl::sprite_main_draw) const FIRE_PHLEGM_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c3,
        ext: 0,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x00c2,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x80c3,
        ext: 0,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x80c2,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40c3,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40c2,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0xc0c3,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0xc0c2,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00d4,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00d3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40d4,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x40d3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x80d4,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x80d3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0xc0d4,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0xc0d3,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FLYING_TILE_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00d3,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40d3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x80d3,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc0d3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c3,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40c3,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x80c3,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc0c3,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const DASH_TREE_TOP_GRID_CHAR_FLAGS: [u16; 16] = [
    0x3100, 0x3102, 0x7102, 0x7100, 0x3120, 0x3122, 0x7122, 0x7120, 0x3104, 0x3106, 0x7106, 0x7104,
    0x3124, 0x3126, 0x7126, 0x7124,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ANTIFAIRY_CIRCLE_VELOCITY_TARGETS: [u8; 2] = [18, (-18i8) as u8];

pub(in crate::zelda_rtl::sprite_main_draw) const CRYSTAL_SWITCH_PAL: [u8; 2] = [2, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const WARP_VORTEX_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const WALKING_ZORA_SPAWN_X_OFFSETS: [i8; 4] = [8, -8, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const WALKING_ZORA_SPAWN_Y_OFFSETS: [i8; 4] = [0, 0, 8, -8];

pub(in crate::zelda_rtl::sprite_main_draw) const WALKING_ZORA_SPAWN_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const WALKING_ZORA_SPAWN_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_B_STEP_X_VELOCITIES: [i8; 8] = [1, 1, -1, -1, -1, -1, 1, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_B_STEP_Y_VELOCITIES: [i8; 8] = [-1, 1, 1, -1, -1, 1, 1, -1];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_B_FAST_X_VELOCITIES: [i8; 8] = [8, 0, -8, 0, -8, 0, 8, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_B_FAST_Y_VELOCITIES: [i8; 8] = [0, 8, 0, -8, 0, 8, 0, -8];

pub(in crate::zelda_rtl::sprite_main_draw) const ZAZAK_DIR2: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_B_FAST_COLLISION_MASKS: [u8; 8] = [1, 4, 2, 8, 2, 4, 1, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_B_COLLISION_MASKS: [u8; 8] = [8, 1, 4, 2, 8, 2, 4, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_B_FAST_NEXT_DIRECTIONS: [u8; 8] = [1, 2, 3, 0, 5, 6, 7, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const SOLDIER_B_NEXT_DIRECTIONS: [u8; 8] = [3, 0, 1, 2, 7, 4, 5, 6];

pub(in crate::zelda_rtl::sprite_main_draw) const TRIDENT_DRAW_FRAMES: [DrawMultipleData; 50] = [
    DrawMultipleData {
        x: 10,
        y: -10,
        char_flags: 0x0864,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: -15,
        char_flags: 0x0864,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -20,
        char_flags: 0x0864,
        ext: 0,
    },
    DrawMultipleData {
        x: -5,
        y: -25,
        char_flags: 0x0864,
        ext: 0,
    },
    DrawMultipleData {
        x: -18,
        y: -38,
        char_flags: 0x0844,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -4,
        char_flags: 0x0865,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: -11,
        char_flags: 0x0865,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: -18,
        char_flags: 0x0865,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: -25,
        char_flags: 0x0865,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: -40,
        char_flags: 0x0862,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -9,
        char_flags: 0x4864,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: -14,
        char_flags: 0x4864,
        ext: 0,
    },
    DrawMultipleData {
        x: 3,
        y: -20,
        char_flags: 0x4864,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: -26,
        char_flags: 0x4864,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: -37,
        char_flags: 0x4844,
        ext: 2,
    },
    DrawMultipleData {
        x: -10,
        y: -20,
        char_flags: 0x4874,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: -20,
        char_flags: 0x4874,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: -20,
        char_flags: 0x4874,
        ext: 0,
    },
    DrawMultipleData {
        x: 11,
        y: -20,
        char_flags: 0x4874,
        ext: 0,
    },
    DrawMultipleData {
        x: 18,
        y: -23,
        char_flags: 0x4860,
        ext: 2,
    },
    DrawMultipleData {
        x: -10,
        y: -30,
        char_flags: 0xc864,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -24,
        char_flags: 0xc864,
        ext: 0,
    },
    DrawMultipleData {
        x: 2,
        y: -18,
        char_flags: 0xc864,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -12,
        char_flags: 0xc864,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: -8,
        char_flags: 0xc844,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -32,
        char_flags: 0x8865,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: -25,
        char_flags: 0x8865,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: -18,
        char_flags: 0x8865,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: -11,
        char_flags: 0x8865,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: -5,
        char_flags: 0x8862,
        ext: 2,
    },
    DrawMultipleData {
        x: 13,
        y: -30,
        char_flags: 0x8864,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -25,
        char_flags: 0x8864,
        ext: 0,
    },
    DrawMultipleData {
        x: 2,
        y: -19,
        char_flags: 0x8864,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -13,
        char_flags: 0x8864,
        ext: 0,
    },
    DrawMultipleData {
        x: -16,
        y: -9,
        char_flags: 0x8844,
        ext: 2,
    },
    DrawMultipleData {
        x: 14,
        y: -20,
        char_flags: 0x0874,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: -20,
        char_flags: 0x0874,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -20,
        char_flags: 0x0874,
        ext: 0,
    },
    DrawMultipleData {
        x: -7,
        y: -20,
        char_flags: 0x0874,
        ext: 0,
    },
    DrawMultipleData {
        x: -21,
        y: -23,
        char_flags: 0x0860,
        ext: 2,
    },
    DrawMultipleData {
        x: 13,
        y: -30,
        char_flags: 0x8864,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -25,
        char_flags: 0x8864,
        ext: 0,
    },
    DrawMultipleData {
        x: 2,
        y: -19,
        char_flags: 0x8864,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -13,
        char_flags: 0x8864,
        ext: 0,
    },
    DrawMultipleData {
        x: -16,
        y: -9,
        char_flags: 0x8844,
        ext: 2,
    },
    DrawMultipleData {
        x: -10,
        y: -30,
        char_flags: 0xc864,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -24,
        char_flags: 0xc864,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -24,
        char_flags: 0xc864,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -24,
        char_flags: 0xc864,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -24,
        char_flags: 0xc864,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const DIGGING_GAME_GUY_DRAW_FRAMES: [DrawMultipleData; 9] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0a40,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 9,
        char_flags: 0x0c56,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a42,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0a40,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a42,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a42,
        ext: 2,
    },
    DrawMultipleData {
        x: -1,
        y: -7,
        char_flags: 0x0a40,
        ext: 2,
    },
    DrawMultipleData {
        x: -1,
        y: 0,
        char_flags: 0x0a44,
        ext: 2,
    },
    DrawMultipleData {
        x: -1,
        y: 0,
        char_flags: 0x0a44,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const BOMB_SHOP_ENTITY_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a48,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a4c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x04c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x04c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x084e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x084e,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const BIG_FAERIE_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x408e,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x00ae,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x40ae,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x408c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x408a,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x00aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x40aa,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x008c,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x408c,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x40ac,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const DESERT_BARRIER_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x408e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x00ae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40ae,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SAGE_MANTLE_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x162c,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x562c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x062e,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 16,
        char_flags: 0x462e,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_SPAWN_PROBE_ALWAYS_LOCAL_X_VELOCITIES: [i8; 64] = [
    -16, -16, -16, -16, -16, -16, -16, -16, -16, -14, -12, -10, -8, -6, -4, -2, 0, 2, 4, 6, 8, 10,
    12, 14, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 14, 12, 10, 8, 6, 4, 2,
    0, -2, -4, -6, -8, -10, -12, -14, -16, -16, -16, -16, -16, -16, -16, -16, -16,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_SPAWN_PROBE_ALWAYS_LOCAL_Y_VELOCITIES: [i8; 64] = [
    0, 2, 4, 6, 8, 10, 12, 14, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 14,
    12, 10, 8, 6, 4, 2, 0, -2, -4, -6, -8, -10, -12, -14, -16, -16, -16, -16, -16, -16, -16, -16,
    -16, -16, -16, -16, -16, -16, -16, -16, -16, -14, -12, -10, -8, -6, -4, -2, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_BEAMOS_LASER_HIT_X_OFFSETS: [i8; 4] = [-4, 4, -4, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_BEAMOS_LASER_HIT_Y_OFFSETS: [i8; 4] = [-4, -4, 4, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_BEAMOS_LASER_HIT_FLAGS: [u8; 4] = [6, 0x46, 0x86, 0xc6];

pub(in crate::zelda_rtl::sprite_main_draw) const BOMBOS_TABLET_MESSAGES: [u16; 2] = [0x10d, 0x10f];

pub(in crate::zelda_rtl::sprite_main_draw) const ETHER_TABLET_MESSAGES: [u16; 2] = [0x10d, 0x10e];

pub(in crate::zelda_rtl::sprite_main_draw) const PULL_SWITCH_FACING_DOWN_YOFFS: [u8; 12] =
    [9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14];

pub(in crate::zelda_rtl::sprite_main_draw) const BAD_PULL_DOWN_SWITCH_DRAW_BAD_PULL_SWITCH_Y_OFFSET_INDEX_BY_GRAPHICS: [u8; 12] =
    [0, 0, 1, 1, 2, 2, 3, 3, 4, 5, 5, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const BAD_PULL_UP_SWITCH_DRAW_BAD_PULL_SWITCH_Y_OFFSET_INDEX_BY_GRAPHICS: [u8; 12] =
    [0, 0, 1, 1, 2, 2, 3, 3, 4, 5, 5, 5];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_93_BUMPER_VELOCITIES: [i8; 4] = [0, 2, -2, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_D4_LANDMINE_OAM_FLAGS: [u8; 4] = [4, 2, 8, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const TRIDENT_DRAW_X_OFFSETS: [i8; 5] = [24, -16, 0, 16, -8];

pub(in crate::zelda_rtl::sprite_main_draw) const TRIDENT_DRAW_Y_OFFSETS: [i8; 5] = [4, 4, 16, 21, 19];

pub(in crate::zelda_rtl::sprite_main_draw) const BOT_DRAW_LOCAL_GRAPHICS: [u8; 4] = [0, 1, 0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const BOT_DRAW_FLAGS: [u8; 4] = [0, 0, 0x40, 0x40];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_BOMB_SHOP_CLERK_LOCAL_GRAPHICS: [u8; 8] = [0, 1, 0, 1, 0, 1, 0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_BOMB_SHOP_CLERK_DELAY: [u8; 8] = [255, 32, 255, 24, 15, 24, 255, 15];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_BOMB_SHOP_HUFF_OAM_FLAGS: [u8; 4] = [4, 0x44, 0xc4, 0x84];

pub(in crate::zelda_rtl::sprite_main_draw) const FAERIE_CLOUD_DRAW_XY: [u16; 8] = [0xfff4, 0xfffa, 0, 6, 12, 18, 0, 6];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_57_DESERT_STATUE_NEXT_D: [u8; 4] = [3, 2, 0, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_57_DESERT_STATUE_XV: [i8; 4] = [16, -16, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_57_DESERT_STATUE_YV: [i8; 4] = [0, 0, 16, -16];

pub(in crate::zelda_rtl::sprite_main_draw) const DASH_TREE_TOP_DRAW_X_OFFSETS: [i8; 16] =
    [10, 22, 30, 1, 34, 5, 13, 29, 0, 17, 27, 44, 15, 33, 18, 26];

pub(in crate::zelda_rtl::sprite_main_draw) const DASH_TREE_TOP_DRAW_Y_OFFSETS: [i8; 16] =
    [0, 4, 2, 7, 10, 16, 24, 23, 34, 35, 30, 31, 46, 42, 10, 11];

pub(in crate::zelda_rtl::sprite_main_draw) const DASH_TREE_TOP_DRAW_CHARS: [u8; 6] = [8, 8, 0x28, 0x28, 0x2a, 0x2a];

pub(in crate::zelda_rtl::sprite_main_draw) const DASH_TREE_TOP_DRAW_FL: [u8; 6] = [0x31, 0x71, 0x31, 0x71, 0x31, 0x71];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_82_ANTIFAIRY_CIRCLE_VEL: [i8; 2] = [1, -1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_7_E_FIREBAR_CLOCKWISE_INCR: [i16; 4] = [-2, 2, -1, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_BA_WHIRLPOOL_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_5_D_ROLLER_VERTICAL_DOWN_FIRST_XYVEL: [i8; 6] = [-16, 16, 0, 0, -16, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_66_WALL_CANNON_VERTICAL_LEFT_LOCAL_X_VELOCITIES: [i8; 4] = [0, 0, -16, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_66_WALL_CANNON_VERTICAL_LEFT_LOCAL_Y_VELOCITIES: [i8; 4] = [-16, 16, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_66_WALL_CANNON_VERTICAL_LEFT_LOCAL_GRAPHICS: [u8; 4] = [0, 0, 2, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_66_WALL_CANNON_VERTICAL_LEFT_OAM_FLAGS: [u8; 4] = [0x40, 0, 0, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_5_B_SPARK_CLOCKWISE_OAM_FLAGS: [u8; 4] = [0, 0x40, 0x80, 0xc0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_5_B_SPARK_CLOCKWISE_DIRECTIONS: [u8; 8] = [1, 3, 2, 0, 7, 5, 6, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_80_FIRESNAKE_OAM_FLAGS: [u8; 4] = [0, 0x40, 0x80, 0xc0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_80_FIRESNAKE_LOCAL_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_80_FIRESNAKE_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_87_KODONGO_FIRE_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_87_KODONGO_FIRE_LOCAL_GRAPHICS: [u8; 32] = [
    5, 4, 3, 1, 2, 0, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_DRAW_PRIZE_X_OFFSETS: [i8; 5] = [-8, -8, 0, 8, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_DRAW_PRIZE_Y_OFFSETS: [i8; 5] = [-24, -16, -20, -20, -20];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_DRAW_PRIZE_CHARS: [u8; 3] = [0x0b, 0x1b, 0xb6];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_DRAW_PRIZE_FLAGS: [u8; 5] = [0x38, 0x38, 0x34, 0x35, 0x35];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_DRAW_PRIZE_CHARS_3: [u8; 6] = [0x12, 0x32, 0x31, 3, 0x22, 0x33];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_DRAW_PRIZE_CHARS_4: [u8; 6] = [0x7c, 0x7c, 0x22, 2, 0x12, 0x33];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_GUY_DRAW_X_OFFSETS: [i8; 15] =
    [0, 0, 0, 0, 0, -5, 0, -1, -1, 0, 0, 0, 0, 1, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_GUY_DRAW_Y_OFFSETS: [i8; 15] = [
    0, -10, -10, 0, -10, -3, 0, -10, -10, 0, -10, -10, 0, -10, -10,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_GUY_DRAW_CHARS: [u8; 15] =
    [0x26, 6, 6, 8, 6, 0x3a, 0x26, 6, 6, 0x26, 6, 6, 0x26, 6, 6];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_GUY_DRAW_FLAGS: [u8; 15] =
    [8, 6, 6, 8, 6, 8, 8, 6, 6, 8, 6, 6, 8, 6, 6];

pub(in crate::zelda_rtl::sprite_main_draw) const ARCHERY_GAME_GUY_DRAW_BIG: [u8; 15] =
    [2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const PUSH_SWITCH_DRAW_OAM: [(i8, i8, u8, u8); 40] = [
    (4, 20, 0xdc, 0x20),
    (4, 12, 0xdd, 0x20),
    (4, 12, 0xdd, 0x20),
    (4, 12, 0xdd, 0x20),
    (0, 0, 0xca, 0x20),
    (3, 12, 0xdd, 0x20),
    (3, 20, 0xdc, 0x20),
    (3, 20, 0xdc, 0x20),
    (3, 20, 0xdc, 0x20),
    (0, 0, 0xca, 0x20),
    (-8, 8, 0xea, 0x20),
    (0, 8, 0xeb, 0x20),
    (-8, 16, 0xfa, 0x20),
    (0, 16, 0xfb, 0x20),
    (0, 0, 0xca, 0x20),
    (-12, 4, 0xcc, 0x20),
    (-4, 4, 0xcd, 0x20),
    (-4, 4, 0xcd, 0x20),
    (-4, 4, 0xcd, 0x20),
    (0, 0, 0xca, 0x20),
    (-10, 4, 0xcc, 0x20),
    (-4, 4, 0xcd, 0x20),
    (-4, 4, 0xcd, 0x20),
    (-4, 4, 0xcd, 0x20),
    (0, 0, 0xca, 0x20),
    (-8, 4, 0xcc, 0x20),
    (-4, 4, 0xcd, 0x20),
    (-4, 4, 0xcd, 0x20),
    (-4, 4, 0xcd, 0x20),
    (0, 0, 0xca, 0x20),
    (4, 3, 0xe2, 0x20),
    (-6, 4, 0xcc, 0x20),
    (-4, 4, 0xcd, 0x20),
    (-4, 4, 0xcd, 0x20),
    (0, 0, 0xca, 0x20),
    (4, 3, 0xf1, 0x20),
    (-6, 4, 0xcc, 0x20),
    (-4, 4, 0xcd, 0x20),
    (-4, 4, 0xcd, 0x20),
    (0, 0, 0xca, 0x20),
];

pub(in crate::zelda_rtl::sprite_main_draw) const PUSH_SWITCH_DRAW_WH: [u8; 16] = [
    8, 6, 0x10, 0x10, 0x10, 8, 0x10, 8, 0x10, 8, 0x10, 8, 0x10, 3, 0x10, 8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_21_WATER_SWITCH_DELAY: [u8; 10] = [40, 6, 3, 3, 3, 5, 1, 1, 3, 12];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_21_WATER_SWITCH_DIR: [u8; 10] = [0, 1, 2, 3, 4, 5, 5, 6, 7, 6];
