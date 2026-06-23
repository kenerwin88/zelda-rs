use super::sprite::DrawMultipleData;
use crate::types::sign8;

// ---------------------------------------------------------------------------
// File-local RAM offsets. The canonical zelda_rtl.rs already exports the
// frequently-shared ones; these are the remaining `variables.h` addresses
// referenced by SpriteDraw_*. Local names use `_DRAW` so they cannot clash
// with constants other sprite_main_* modules declare for their own use.
// ---------------------------------------------------------------------------

// variables.h:672 — sprite_ignore_projectile.
pub(super) const SPRITE_IGNORE_PROJECTILE_DRAW: usize = 0x0ba0;
// variables.h:721..722 — sprite_B / sprite_C live in pages 0xDA0..0xDC0.
pub(super) const SPRITE_B_DRAW: usize = 0x0da0;
pub(super) const SPRITE_C_DRAW: usize = 0x0db0;
// variables.h:727..728 — sprite_delay_aux1 / aux2.
pub(super) const SPRITE_DELAY_AUX1_DRAW: usize = 0x0e00;
pub(super) const SPRITE_DELAY_AUX2_DRAW: usize = 0x0e10;
// variables.h:737 — sprite_F.
pub(super) const SPRITE_F_DRAW: usize = 0x0ea0;
// variables.h:739..742 — sprite_anim_clock / sprite_G / sprite_delay_aux3 /
// sprite_hit_timer.
pub(super) const SPRITE_ANIM_CLOCK_DRAW: usize = 0x0ec0;
pub(super) const SPRITE_G_DRAW: usize = 0x0ed0;
pub(super) const SPRITE_DELAY_AUX3_DRAW: usize = 0x0ee0;
// variables.h:743 — sprite_y_recoil.
pub(super) const SPRITE_Y_RECOIL_DRAW: usize = 0x0f30;
// variables.h:776 — light/dark world flag.
pub(super) const IS_IN_DARK_WORLD_DRAW: usize = 0x0fff;
// variables.h:758..761 — repulsespark_*.
pub(super) const REPULSESPARK_TIMER_DRAW: usize = 0x0fac;
pub(super) const REPULSESPARK_X_LO_DRAW: usize = 0x0fad;
pub(super) const REPULSESPARK_Y_LO_DRAW: usize = 0x0fae;
// variables.h:755 — sprite_tiletype.
pub(super) const SPRITE_TILETYPE_DRAW: usize = 0x0fa5;
// overlord.c stores the active overlord slot here before sprite code consumes it.
pub(super) const ACTIVE_OVERLORD_INDEX_DRAW: usize = 0x0fde;
// variables.h:766..768 — garnish_active / tmp_counter / shared draw scratch.
pub(super) const GARNISH_ACTIVE_DRAW: usize = 0x0fb4;
pub(super) const SPRITE_DRAW_WORK_Y_OR_FLAGS: usize = 0x0fb6;
// variables.h:910 — sram_progress_indicator_3.
pub(super) const SRAM_PROGRESS_INDICATOR_3_DRAW: usize = 0x0f3c9;
// hud.c private table mirrored for bomb shop purchase gating.
pub(super) const MAX_BOMBS_FOR_LEVEL_DRAW: [u8; 8] = [10, 15, 20, 25, 30, 35, 40, 50];
// variables.h:1203..1217 — garnish_* tables (paged at 0x1F800+).
pub(super) const GARNISH_TYPE_DRAW: usize = 0x1f800;
pub(super) const GARNISH_Y_LO_DRAW: usize = 0x1f81e;
pub(super) const GARNISH_X_LO_DRAW: usize = 0x1f83c;
pub(super) const GARNISH_Y_HI_DRAW: usize = 0x1f85a;
pub(super) const GARNISH_X_HI_DRAW: usize = 0x1f878;
pub(super) const GARNISH_COUNTDOWN_DRAW: usize = 0x1f90e;
pub(super) const GARNISH_SPRITE_DRAW: usize = 0x1f92c;
pub(super) const GARNISH_FLOOR_DRAW: usize = 0x1f968;
pub(super) const GARNISH_OAM_FLAGS_DRAW: usize = 0x1f9fe;
// variables.h:690 — activate_bomb_trap_overlord.
pub(super) const ACTIVATE_BOMB_TRAP_OVERLORD_DRAW: usize = 0x0cf4;
// variables.h:1208 — sprite_I.
pub(super) const SPRITE_I_DRAW: usize = 0x1f9c2;
// variables.h:1241..1242 — chainchomp history buffer aliases the moldorm pages.
// variables.h:679..683 — ancilla_*.
pub(super) const ANCILLA_Y_LO_DRAW: usize = 0x0bfa;
pub(super) const ANCILLA_X_LO_DRAW: usize = 0x0c04;
pub(super) const ANCILLA_Y_HI_DRAW: usize = 0x0c0e;
pub(super) const ANCILLA_X_HI_DRAW: usize = 0x0c18;
pub(super) const ANCILLA_X_VEL_DRAW: usize = 0x0c2c;
pub(super) const ANCILLA_Y_VEL_DRAW: usize = 0x0c22;
pub(super) const ANCILLA_Z_DRAW: usize = 0x29e;
// variables.h:179 — sound_effect_1.
// variables.h:535 — minigame_credits.
pub(super) const MINIGAME_CREDITS_DRAW: usize = 0x04c4;
// variables.h:741 — flag_overworld_area_did_change.
pub(super) const FLAG_OVERWORLD_AREA_DID_CHANGE_DRAW: usize = 0x0abf;
// sprite_main.c local scratch words.
// variables.h:488 — enhanced_features0.
pub(super) const ENHANCED_FEATURES0_DRAW: usize = 0x064c;
pub(super) const FEATURE_MISC_BUG_FIXES_DRAW: u32 = 4096;
pub(super) const FEATURE_GAME_CHANGING_BUG_FIXES_DRAW: u32 = 16384;
// hud.h:10 — kHudItem_BookMudora.
pub(super) const HUD_ITEM_BOOK_MUDORA_DRAW: u8 = 15;
// hud.h:9 — kHudItem_Flute.
pub(super) const HUD_ITEM_FLUTE_DRAW: u8 = 13;
// hud.h:7 — kHudItem_Mushroom.
pub(super) const HUD_ITEM_MUSHROOM_DRAW: u8 = 5;
// player.h:30 — kPlayerState_OpeningDesertPalace.
pub(super) const PLAYER_STATE_OPENING_DESERT_PALACE_DRAW: u8 = 27;
// variables.h:1071 — link_item_bombos_medallion.
pub(super) const LINK_ITEM_BOMBOS_MEDALLION_DRAW: usize = 0x0f347;
pub(super) const LINK_ITEM_QUAKE_MEDALLION_DRAW: usize = 0x0f349;
// ---------------------------------------------------------------------------
// kSinusLookupTable from sprite_main.c:338 — 256-entry sine half-wave used
// by ChainBallSin / HelmasaurSin. Verbatim from the C source.
// ---------------------------------------------------------------------------
pub(super) const SHARED_SINE_LOOKUP_TABLE: [u16; 256] = [
    0, 3, 6, 9, 12, 15, 18, 21, 25, 28, 31, 34, 37, 40, 40, 46, 49, 53, 56, 59, 62, 65, 68, 71, 74,
    77, 80, 83, 86, 89, 92, 95, 97, 100, 103, 106, 109, 112, 115, 117, 120, 123, 126, 128, 131,
    134, 136, 139, 142, 144, 147, 149, 152, 155, 157, 159, 162, 164, 167, 169, 171, 174, 176, 178,
    181, 183, 185, 187, 189, 191, 193, 195, 197, 199, 201, 203, 205, 207, 209, 211, 212, 214, 216,
    217, 219, 221, 222, 224, 225, 227, 228, 230, 231, 232, 234, 235, 236, 237, 238, 239, 241, 242,
    243, 244, 244, 245, 246, 247, 248, 249, 249, 250, 251, 251, 252, 252, 253, 253, 254, 254, 254,
    255, 255, 255, 255, 255, 255, 255, 256, 255, 255, 255, 255, 255, 255, 255, 254, 254, 254, 253,
    253, 252, 252, 251, 251, 250, 249, 249, 248, 247, 246, 245, 244, 244, 243, 242, 241, 239, 238,
    237, 236, 235, 234, 232, 231, 230, 228, 227, 225, 224, 222, 221, 219, 217, 216, 214, 212, 211,
    209, 207, 205, 203, 201, 199, 197, 195, 193, 191, 189, 187, 185, 183, 181, 178, 176, 174, 171,
    169, 167, 164, 162, 159, 157, 155, 152, 149, 147, 144, 142, 139, 136, 134, 131, 128, 126, 123,
    120, 117, 115, 112, 109, 106, 103, 100, 97, 95, 92, 89, 86, 83, 80, 77, 74, 71, 68, 65, 62, 59,
    56, 53, 49, 46, 43, 40, 37, 34, 31, 28, 25, 21, 18, 15, 12, 9, 6, 3,
];

// ---------------------------------------------------------------------------
// kLargeShadow_Dmd from sprite_main.c:373 — 15-entry shadow table used by
// SpriteDraw_BigShadow.
// ---------------------------------------------------------------------------
pub(super) const LARGE_SHADOW_DRAW_FRAMES: [DrawMultipleData; 15] = [
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

// ---------------------------------------------------------------------------
// kChainBallTrooperHead_* from sprite_main.c:293-294.
// kFlailTrooperBody_* from sprite_main.c:295-336.
// kFlailTrooperWeapon_* from sprite_main.c:356-362.
// ---------------------------------------------------------------------------
pub(super) const CHAIN_BALL_TROOPER_HEAD_CHARS: [u8; 4] = [2, 2, 0, 4];
pub(super) const CHAIN_BALL_TROOPER_HEAD_FLAGS: [u8; 4] = [0x40, 0, 0, 0];
pub(super) const CHAIN_BALL_TROOPER_BODY_CHAR_BY_STATE: [u8; 4] = [0x0d, 0x60, 0x22, 0x10];
pub(super) const FLAIL_TROOPER_GRAPHICS: [u8; 32] = [
    0x10, 0x11, 0x12, 0x13, 0x10, 0x11, 0x12, 0x13, 6, 7, 8, 9, 6, 7, 8, 9, 0, 1, 2, 3, 0, 1, 4, 5,
    0x0a, 0x0b, 0x0c, 0x0d, 0x0a, 0x0b, 0x0e, 0x0f,
];
pub(super) const FLAIL_TROOPER_BODY_X_OFFSETS: [i8; 72] = [
    -4, 4, 12, -4, 4, 13, -4, 4, 13, -4, 4, 13, -4, 4, 13, -4, 4, 13, 0, 0, 4, 0, 0, 5, 0, 0, 6, 0,
    0, 4, -4, 4, -6, -4, 4, -5, -4, 4, -5, -4, 4, -6, -4, 4, -5, -4, 4, -6, 0, 0, 4, 0, 0, 3, 0, 0,
    2, 0, 0, 4, 0, 0, 0, 0, 0, 0, -4, 4, 4, -4, 4, 4,
];
pub(super) const FLAIL_TROOPER_BODY_Y_OFFSETS: [i8; 72] = [
    0, 0, -4, 0, 0, -4, 0, 0, -3, 0, 0, -2, 0, 0, -3, 0, 0, -2, 0, 0, 1, 0, 0, 1, 0, 0, 2, 0, 0, 2,
    0, 0, -2, 0, 0, -2, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, 1, 0, 0, 1, 0, 0, 2, 0, 0, 2,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub(super) const FLAIL_TROOPER_BODY_CHARS: [u8; 72] = [
    0x46, 6, 0x2f, 0x46, 6, 0x2f, 0x48, 0xd, 0x2f, 0x48, 0xd, 0x2f, 0x49, 0xc, 0x2f, 0x49, 0xc,
    0x2f, 8, 8, 0x2f, 8, 8, 0x2f, 0x22, 0x22, 0x2f, 0x22, 0x22, 0x2f, 0xa, 0x64, 0x2f, 0xa, 0x64,
    0x2f, 0x2c, 0x67, 0x2f, 0x2c, 0x67, 0x2f, 0x2d, 0x66, 0x2f, 0x2d, 0x66, 0x2f, 8, 8, 0x2f, 8, 8,
    0x2f, 0x22, 0x22, 0x2f, 0x22, 0x22, 0x2f, 0x62, 0x62, 0x62, 0x62, 0x62, 0x62, 0x46, 0x4b, 0x4b,
    0x69, 0x64, 0x64,
];
pub(super) const FLAIL_TROOPER_BODY_FLAGS: [u8; 72] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x40, 0x40, 0, 0x40, 0x40, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0,
    0, 0, 0x40, 0x40, 0, 0x40, 0x40,
];
pub(super) const FLAIL_TROOPER_BODY_SIZES: [u8; 72] = [
    2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2,
    0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 0, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2,
];
pub(super) const FLAIL_TROOPER_BODY_SEGMENT_COUNTS: [u8; 24] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1,
];
pub(super) const FLAIL_TROOPER_BODY_SPRITE_OFFSETS: [u8; 24] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8,
];
pub(super) const FLAIL_TROOPER_CHAIN_SEGMENT_SCALES: [u8; 4] = [0x33, 0x66, 0x99, 0xcc];
pub(super) const FLAIL_TROOPER_CHAIN_RADIUS_BY_FRAME: [u8; 32] = [
    0x10, 0x12, 0x14, 0x16, 0x18, 0x1a, 0x1c, 0x1e, 0x20, 0x22, 0x24, 0x26, 0x28, 0x2a, 0x2c, 0x2e,
    0x30, 0x2e, 0x2c, 0x2a, 0x28, 0x26, 0x24, 0x22, 0x20, 0x1e, 0x1c, 0x1a, 0x18, 0x16, 0x14, 0x12,
];
pub(super) const FLAIL_TROOPER_WEAPON_X_CENTER_BY_DIRECTION: [i8; 4] = [4, 4, 12, -5];
pub(super) const FLAIL_TROOPER_WEAPON_Y_CENTER_BY_DIRECTION: [i8; 4] = [-2, -2, -6, -4];
pub(super) const FLAIL_TROOPER_ATTACK_DIRECTIONS: [u8; 4] = [3, 1, 2, 0];
pub(super) const ROPE_GRAPHICS: [u8; 8] = [0, 0, 2, 3, 2, 3, 1, 1];
pub(super) const ROPE_OAM_FLAGS: [u8; 8] = [0, 0x40, 0, 0, 0x40, 0x40, 0, 0x40];
pub(super) const ROPE_FAST_GFX_BY_DIRECTION: [u8; 8] = [4, 5, 2, 3, 0, 1, 6, 7];
pub(super) const ROPE_X_VELOCITIES: [i8; 8] = [8, -8, 0, 0, 16, -16, 0, 0];
pub(super) const ROPE_Y_VELOCITIES: [i8; 8] = [0, 0, 8, -8, 0, 0, 0x10, -0x10];
pub(super) const ROPE_IDLE_GFX_BY_DIRECTION: [u8; 4] = [2, 3, 1, 0];
pub(super) const RECRUIT_X_VELOCITIES: [u8; 8] = [12, 244, 0, 0, 18, 238, 0, 0];
pub(super) const RECRUIT_Y_VELOCITIES: [u8; 8] = [0, 0, 12, 244, 0, 0, 18, 238];
pub(super) const RECRUIT_GRAPHICS: [u8; 8] = [0, 2, 4, 6, 1, 3, 5, 7];
pub(super) const ZORA_SURFACING_GRAPHICS: [u8; 16] =
    [4, 3, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 0, 0];
pub(super) const ABSORB_BIG_KEY_MASKS_DRAW: [u16; 2] = [0x40, 0x20];
pub(super) const HEART_REFILL_X_ACCELERATIONS: [i8; 2] = [1, -1];
pub(super) const HEART_REFILL_X_VELOCITY_TARGETS: [i8; 2] = [10, -10];
pub(super) const PLAYER_DAMAGE_CARRY_MASK_DRAW: u8 = 1;
pub(super) const PLAYER_DAMAGE_NONZERO_MASK_DRAW: u8 = 2;

// kThrowableScenery_DrawLarge_* from sprite_main.c:137-141.
pub(super) const THROWABLE_SCENERY_LARGE_X_OFFSETS: [i16; 4] = [-8, 8, -8, 8];
pub(super) const THROWABLE_SCENERY_LARGE_Y_OFFSETS: [i16; 4] = [-14, -14, 2, 2];
pub(super) const THROWABLE_SCENERY_LARGE_DRAW_FLAGS: [u8; 4] = [0, 0x40, 0x80, 0xc0];
pub(super) const THROWABLE_SCENERY_LARGE_EXTRA_X_OFFSETS: [i16; 3] = [-6, 0, 6];
pub(super) const THROWABLE_SCENERY_LARGE_OAM_FLAGS: [u8; 2] = [0xc, 0];
pub(super) const THROWABLE_SCENERY_CHARS: [u8; 12] = [
    0x42, 0x44, 0x46, 0, 0x46, 0x44, 0x42, 0x44, 0x44, 0, 0x46, 0x44,
];
pub(super) const THROWABLE_SCENERY_FLAGS: [u8; 9] = [0x0c, 0x0c, 0x0c, 0, 0, 0, 0xb0, 0x08, 0xb4];
pub(super) const SCATTER_DEBRIS_X_OFFSETS: [i16; 4] = [-8, 8, -8, 8];
pub(super) const SCATTER_DEBRIS_Y_OFFSETS: [i16; 4] = [-8, -8, 8, 8];

// ---------------------------------------------------------------------------
// Free helpers (module-local) ported from sprite_main.c inline statics.
// ---------------------------------------------------------------------------

pub(super) fn overworld_find_map16_vram_address_for_draw(addr: u16) -> u16 {
    (if addr & 0x3f >= 0x20 { 0x0400 } else { 0 })
        + (if addr & 0x0fff >= 0x0800 { 0x0800 } else { 0 })
        + (addr & 0x001f)
        + ((addr & 0x0780) >> 1)
}

/// `ChainBallMult` (sprite_main.c:1397) — saturating fixed-point multiply
/// used by SpriteDraw_BNCFlail.
pub(super) fn chain_ball_mult(a: u16, b: u8) -> u8 {
    chain_ball_mult_draw(a, b)
}

pub(super) fn chain_ball_mult_draw(a: u16, b: u8) -> u8 {
    if a >= 256 {
        return b;
    }
    let p = (a as u32) * (b as u32);
    ((p >> 8) as u8).wrapping_add(((p >> 7) & 1) as u8)
}

/// `GuruguruBarMult` (sprite_main.c:1438) — same fixed-point helper as
/// ChainBallMult, named separately in the C source.
pub(super) fn guruguru_bar_mult(a: u16, b: u8) -> u8 {
    chain_ball_mult_draw(a, b)
}

/// `GuruguruBarSin` (sprite_main.c:1445) — table lookup plus signed quadrant.
pub(super) fn guruguru_bar_sin(a: u16, b: u8) -> i8 {
    let t = guruguru_bar_mult(SHARED_SINE_LOOKUP_TABLE[(a & 0xff) as usize], b);
    if (a & 0x100) != 0 {
        (0i8).wrapping_sub(t as i8)
    } else {
        t as i8
    }
}

/// `ArrgiMult` (sprite_main.c:1450) — same fixed-point helper as
/// ChainBallMult, named separately in the C source.
pub(super) fn arrgi_mult(a: u16, b: u8) -> u8 {
    chain_ball_mult_draw(a, b)
}

/// `ArrgiSin` (sprite_main.c:1457) — table lookup plus signed quadrant.
pub(in crate::zelda_rtl) fn arrgi_sin(a: u16, b: u8) -> i8 {
    let t = arrgi_mult(SHARED_SINE_LOOKUP_TABLE[(a & 0xff) as usize], b);
    if (a & 0x100) != 0 {
        (0i8).wrapping_sub(t as i8)
    } else {
        t as i8
    }
}

/// `HelmasaurMult` (sprite_main.c:1462) — same fixed-point helper as
/// ChainBallMult, named separately in the C source.
pub(super) fn helmasaur_mult(a: u16, b: u8) -> u8 {
    chain_ball_mult_draw(a, b)
}

/// `HelmasaurSin` (sprite_main.c:1469) — table lookup plus signed quadrant.
pub(super) fn helmasaur_sin(a: u16, b: u8) -> i8 {
    let t = helmasaur_mult(SHARED_SINE_LOOKUP_TABLE[(a & 0xff) as usize], b);
    if (a & 0x100) != 0 {
        (0i8).wrapping_sub(t as i8)
    } else {
        t as i8
    }
}

/// `ChainChomp_OneMult` (sprite_main.c:1514) — signed input, integer-complement
/// negative branch. The C return type is `int`, not uint8 wrapping.
pub(super) fn chain_chomp_one_mult(a: u8, b: u8) -> i32 {
    let at = if sign8(a) { 0u8.wrapping_sub(a) } else { a };
    let prod = (((at as u16) * (b as u16)) >> 8) as u8;
    if sign8(a) {
        !(prod as i32)
    } else {
        prod as i32
    }
}

/// `TrinexxMult` (sprite_main.c:1520) — signed fixed-point multiply used by
/// SpriteDraw_TrinexxRockHeadAndBody.
pub(super) fn trinexx_mult(a: u8, b: u8) -> u8 {
    trinexx_mult_draw(a, b)
}

pub(super) fn trinexx_mult_draw(a: u8, b: u8) -> u8 {
    let at = if sign8(a) { 0u8.wrapping_sub(a) } else { a };
    let p = (at as u32) * (b as u32);
    let res = ((p >> 8) as u8).wrapping_add(((p >> 7) & 1) as u8);
    if sign8(a) {
        0u8.wrapping_sub(res)
    } else {
        res
    }
}

/// `TrinexxHeadMult` (sprite_main.c:1527) — same fixed-point helper as
/// ChainBallMult, named separately in the C source.
pub(super) fn trinexx_head_mult(a: u16, b: u8) -> u8 {
    chain_ball_mult_draw(a, b)
}

/// `TrinexxHeadSin` (sprite_main.c:1534) — table lookup plus signed quadrant.
pub(in crate::zelda_rtl) fn trinexx_head_sin(a: u16, b: u8) -> i8 {
    let t = trinexx_head_mult(SHARED_SINE_LOOKUP_TABLE[(a & 0xff) as usize], b);
    if (a & 0x100) != 0 {
        (0i8).wrapping_sub(t as i8)
    } else {
        t as i8
    }
}

// Shared draw tables promoted from sprite_main_draw.rs method bodies.
pub(super) const MASTER_SWORD_LIGHT_BALL_DRAW_FRAMES: [DrawMultipleData; 12] = [
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

pub(super) const BEAMOS_EYEBALL_DRAW_X_OFFSETS: [i8; 32] = [
    -1, 0, 1, 2, 3, 4, 5, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17, 15, 14, 13, 12, 11, 10, 8, 7, 5, 4,
    3, 2, 1, 0, -2,
];

pub(super) const BEAMOS_EYEBALL_DRAW_Y_OFFSETS: [i8; 32] = [
    11, 12, 13, 14, 14, 15, 15, 15, 15, 15, 15, 14, 14, 13, 12, 11, 10, 9, 8, 7, 7, 6, 6, 6, 6, 6,
    6, 7, 7, 8, 9, 10,
];

pub(super) const BEAMOS_EYEBALL_DRAW_CHARS: [u8; 32] = [
    0x5b, 0x5b, 0x5a, 0x5a, 0x4b, 0x4b, 0x4a, 0x4a, 0x4a, 0x4a, 0x4b, 0x4b, 0x5a, 0x5a, 0x5b, 0x5b,
    0x5b, 0x5b, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c, 0x5b, 0x5b, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c,
];

pub(super) const BEAMOS_EYEBALL_DRAW_FLAGS: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(super) const WATER_RIPPLE_DRAW_FRAMES: [DrawMultipleData; 6] = [
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

pub(super) const WATER_RIPPLE_FRAME_INDICES: [u8; 4] = [0, 1, 2, 1];

pub(super) const METAL_BALL_LARGE_X_OFFSETS: [i8; 4] = [-8, 8, -8, 8];

pub(super) const METAL_BALL_LARGE_Y_OFFSETS: [i8; 4] = [-8, -8, 8, 8];

pub(super) const METAL_BALL_LARGE_CHARS: [u8; 8] = [0x84, 0x88, 0x88, 0x88, 0x86, 0x88, 0x88, 0x88];

pub(super) const METAL_BALL_LARGE_FLAGS: [u8; 4] = [0, 0, 0xc0, 0x80];

pub(super) const ENEMY_BOMB_EXPLOSION_X_OFFSETS: [i8; 16] =
    [-12, 12, -12, 12, -8, 8, -8, 8, -8, 8, -8, 8, 0, 0, 0, 0];

pub(super) const ENEMY_BOMB_EXPLOSION_Y_OFFSETS: [i8; 16] =
    [-12, -12, 12, 12, -8, -8, 8, 8, -8, -8, 8, 8, 0, 0, 0, 0];

pub(super) const ENEMY_BOMB_EXPLOSION_CHARS: [u8; 16] = [
    0x88, 0x88, 0x88, 0x88, 0x8a, 0x8a, 0x8a, 0x8a, 0x84, 0x84, 0x84, 0x84, 0x86, 0x86, 0x86, 0x86,
];

pub(super) const ENEMY_BOMB_EXPLOSION_FLAGS: [u8; 16] = [
    0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0, 0, 0,
];

pub(super) const SOLDIER_THROWING_DRAW_X_OFFSETS: [i8; 16] =
    [15, 7, 17, 9, -8, 0, -10, -2, 13, 13, 13, 13, -4, -4, -4, -4];

pub(super) const SOLDIER_THROWING_DRAW_Y_OFFSETS: [i8; 16] = [
    -2, -2, -2, -2, -2, -2, -2, -2, 8, 0, 10, 2, -14, -6, -16, -8,
];

pub(super) const SOLDIER_THROWING_DRAW_CHARS: [u8; 16] = [
    0x6f, 0x7f, 0x6f, 0x7f, 0x6f, 0x7f, 0x6f, 0x7f, 0x6e, 0x7e, 0x6e, 0x7e, 0x6e, 0x7e, 0x6e, 0x7e,
];

pub(super) const SOLDIER_THROWING_DRAW_FLAGS: [u8; 16] = [
    0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0,
];

pub(super) const ARCHER_WEAPON_BASE_FRAME_BY_DIRECTION: [u8; 4] = [9, 3, 0, 6];

pub(super) const ARCHER_SOLDIER_DRAW_X_OFFSETS: [i8; 48] = [
    -1, 7, 3, 3, -1, 7, 3, 3, -1, 7, 7, 7, -5, -5, -10, -2, -4, -4, -6, 2, -5, -5, -5, -5, 6, 14,
    11, 11, 6, 14, 11, 11, 6, 14, 14, 14, 11, 11, 18, 10, 12, 12, 14, 6, 11, 11, 11, 11,
];

pub(super) const ARCHER_SOLDIER_DRAW_Y_OFFSETS: [i8; 48] = [
    7, 7, 3, 11, 6, 6, 1, 9, 7, 7, 7, 7, -2, 6, 2, 2, -2, 6, 2, 2, -2, 6, 6, 6, -6, -6, -12, -4,
    -6, -6, -9, -1, -6, -6, -6, -6, -2, 6, 2, 2, -2, 6, 2, 2, -2, 6, 6, 6,
];

pub(super) const ARCHER_SOLDIER_DRAW_CHARS: [u8; 48] = [
    0xa, 0xa, 0x2a, 0x2b, 0x1a, 0x1a, 0x2a, 0x2b, 0xa, 0xa, 0xa, 0xa, 0xb, 0xb, 0x3d, 0x3a, 0x1b,
    0x1b, 0x3d, 0x3a, 0xb, 0xb, 0xb, 0xb, 0xa, 0xa, 0x2b, 0x2a, 0xa, 0xa, 0x2b, 0x2a, 0xa, 0xa,
    0xa, 0xa, 0xb, 0xb, 0x3d, 0x3a, 0x1b, 0x1b, 0x3d, 0x3a, 0xb, 0xb, 0xb, 0xb,
];

pub(super) const ARCHER_SOLDIER_DRAW_FLAGS: [u8; 48] = [
    0xd, 0x4d, 8, 8, 0xd, 0x4d, 8, 8, 0xd, 0x4d, 0x4d, 0x4d, 0xd, 0x8d, 0x48, 0x48, 0xd, 0x8d,
    0x48, 0x48, 0xd, 0x8d, 0x8d, 0x8d, 0x8d, 0xcd, 0x88, 0x88, 0x8d, 0xcd, 0x88, 0x88, 0x8d, 0xcd,
    0xcd, 0xcd, 0x4d, 0xcd, 8, 8, 0x4d, 0xcd, 8, 8, 0x4d, 0xcd, 0xcd, 0xcd,
];

pub(super) const OCTOSTONE_DRAW_X_OFFSETS: [i8; 16] = [
    0, 8, 0, 8, -8, 16, -8, 16, -12, 20, -12, 20, -14, 22, -14, 22,
];

pub(super) const OCTOSTONE_DRAW_Y_OFFSETS: [i8; 16] = [
    0, 0, 8, 8, -8, -8, 16, 16, -12, -12, 20, 20, -14, -14, 22, 22,
];

pub(super) const OCTOSTONE_DRAW_FLAGS: [u8; 16] = [
    0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0,
];

pub(super) const BOMBER_PELLET_DRAW_FRAMES: [DrawMultipleData; 15] = [
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

pub(super) const PIKIT_TONGUE_LENGTH_MULTIPLIERS: [u8; 4] = [0x33, 0x66, 0x99, 0xcc];

pub(super) const PIKIT_TONGUE_DRAW_CHARS: [u8; 8] =
    [0xee, 0xfd, 0xed, 0xfd, 0xee, 0xfd, 0xed, 0xfd];

pub(super) const PIKIT_TONGUE_DRAW_FLAGS: [u8; 8] = [0, 0, 0, 0x40, 0x40, 0xc0, 0x80, 0x80];

pub(super) const PIKIT_GRABBED_ITEM_X_OFFSETS: [i8; 20] = [
    -4, 4, -4, 4, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, -4, 4, -4, 4,
];

pub(super) const PIKIT_GRABBED_ITEM_Y_OFFSETS: [i8; 20] = [
    -4, -4, 4, 4, -4, -4, 4, 4, -4, -4, 4, 4, -4, -4, 4, 4, -4, -4, 4, 4,
];

pub(super) const PIKIT_GRABBED_ITEM_CHARS: [u8; 20] = [
    0x6e, 0x6f, 0x7e, 0x7f, 0x63, 0x7c, 0x73, 0x7c, 0xb, 0x7c, 0x1b, 0x7c, 0xec, 0xf9, 0xfc, 0xf9,
    0xea, 0xeb, 0xfa, 0xfb,
];

pub(super) const PIKIT_GRABBED_ITEM_FLAGS: [u8; 5] = [0x24, 0x24, 0x28, 0x29, 0x2f];

pub(super) const TRINEXX_DRAW1_DRAW_FRAMES: [DrawMultipleData; 36] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x40c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x40e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00e0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0002,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x0022,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x00c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00c4,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x80c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x80c4,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x8020,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x8022,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x8000,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x8002,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0xc0e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x80e0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0xc0c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x80c0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0xc022,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0xc020,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0xc002,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc000,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x40c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x40c2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0xc0c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc0c2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x4002,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4000,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x4022,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4020,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0026,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4026,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x8026,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc026,
        ext: 2,
    },
];

pub(super) const TRINEXX_DRAW_X_OFFSETS: [i8; 35] = [
    0, 3, 9, 16, 24, 0, 2, 7, 13, 20, 0, 1, 4, 9, 15, 0, 0, 0, 0, 0, 0, -1, -4, -9, -15, 0, -2, -7,
    -13, -20, 0, -3, -9, -16, -24,
];

pub(super) const TRINEXX_DRAW_Y_OFFSETS: [u8; 35] = [
    0x18, 0x20, 0x25, 0x25, 0x21, 0x18, 0x20, 0x27, 0x2a, 0x2c, 0x18, 0x20, 0x28, 0x2f, 0x34, 0x18,
    0x21, 0x2a, 0x34, 0x3d, 0x18, 0x20, 0x28, 0x2f, 0x34, 0x18, 0x20, 0x27, 0x2a, 0x2c, 0x18, 0x20,
    0x25, 0x25, 0x21,
];

pub(super) const TRINEXX_DRAW_CHARS: [u8; 5] = [6, 0x28, 0x28, 0x2c, 0x2c];

pub(super) const TRINEXX_SCALE_MULTIPLIERS: [u8; 8] =
    [0xfc, 0xe0, 0xc0, 0xa0, 0x80, 0x60, 0x40, 0x20];

pub(super) const TRINEXX_WIGGLE_X_OFFSETS: [i8; 16] =
    [0, 2, 3, 4, 4, 4, 3, 2, 0, -2, -3, -4, -4, -4, -3, -2];

pub(super) const TRINEXX_WIGGLE_Y_OFFSETS: [i8; 16] =
    [-4, -4, -3, -2, 0, 2, 3, 4, 4, 4, 3, 2, 0, -2, -3, -4];

pub(super) const CHATTY_AGAHNIM_TELEWARP_DRAW_DATA: [(i8, i8, u8, u8); 28] = [
    (-10, -16, 0xce, 0x06),
    (18, -16, 0xce, 0x06),
    (20, -13, 0x26, 0x06),
    (20, -5, 0x36, 0x06),
    (-12, -13, 0x26, 0x46),
    (-12, -5, 0x36, 0x46),
    (18, 0, 0x26, 0x06),
    (18, 8, 0x36, 0x06),
    (-10, 0, 0x26, 0x46),
    (-10, 8, 0x36, 0x46),
    (-8, 0, 0x22, 0x06),
    (8, 0, 0x22, 0x46),
    (-8, 16, 0x22, 0x86),
    (8, 16, 0x22, 0xc6),
    (-10, -16, 0xce, 0x04),
    (18, -16, 0xce, 0x04),
    (20, -13, 0x26, 0x44),
    (20, -5, 0x36, 0x44),
    (-12, -13, 0x26, 0x04),
    (-12, -5, 0x36, 0x04),
    (18, 0, 0x26, 0x44),
    (18, 8, 0x36, 0x44),
    (-10, 0, 0x26, 0x04),
    (-10, 8, 0x36, 0x04),
    (-8, 0, 0x20, 0x04),
    (8, 0, 0x20, 0x44),
    (-8, 16, 0x20, 0x84),
    (8, 16, 0x20, 0xc4),
];

pub(super) const TELEWARP_DRAW_SIZES: [u8; 14] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2];
