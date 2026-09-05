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

// Shared boss and enemy draw tables promoted from sprite_main_draw.rs method bodies.
pub(super) const ALTAR_ZELDA_WARP_DRAW_FRAMES: [DrawMultipleData; 10] = [
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x0480,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x0480,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x04b7,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x04b7,
        ext: 0,
    },
    DrawMultipleData {
        x: -6,
        y: 0,
        char_flags: 0x0524,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 0,
        char_flags: 0x4524,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0524,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4524,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x05c6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x05c6,
        ext: 2,
    },
];

pub(super) const ALTAR_ZELDA_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x0103,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x0104,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x0100,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x0101,
        ext: 2,
    },
];

pub(super) const LEVITATE_GFX: [u8; 4] = [2, 0, 3, 0];

pub(super) const AGAHNIM_SHADOW_X_OFFSETS: [i8; 6] = [0, 10, 8, 0, -10, -10];

pub(super) const AGAHNIM_SHADOW_Y_OFFSETS: [i8; 6] = [-9, -2, -2, -9, -2, -2];

pub(super) const CLONE_DASH_X_VELOCITIES: [i8; 2] = [32, -32];

pub(super) const CLONE_FLAGS3_BY_SLOT: [u8; 2] = [9, 11];

pub(super) const ATTACK_POSE_OFFSETS_BY_DELAY: [u8; 16] =
    [0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0];

pub(super) const HEAD_TURN_STEPS_BY_DELAY: [u8; 16] =
    [0, 0, 0, 0, 0, 0, 0, 6, 5, 4, 3, 2, 1, 0, 0, 0];

pub(super) const HEAD_TURN_DIRECTION_BASES: [u8; 6] = [30, 24, 12, 0, 6, 18];

pub(super) const TELEPORT_TARGET_X_LOW: [u8; 16] = [
    0x38, 0x38, 0x38, 0x58, 0x78, 0x98, 0xb8, 0xb8, 0xb8, 0x98, 0x58, 0x58, 0x60, 0x90, 0x98, 0x78,
];

pub(super) const TELEPORT_TARGET_Y_LOW: [u8; 16] = [
    0xb8, 0x78, 0x58, 0x48, 0x48, 0x48, 0x58, 0x78, 0xb8, 0xb8, 0xb8, 0x90, 0x70, 0x70, 0x90, 0xa0,
];

pub(super) const ENERGY_BALL_GRAPHICS: [u8; 16] = [2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, 0];

pub(super) const GIANT_MOLDORM_X_VELOCITIES: [i8; 32] = [
    24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9, 0, 9, 17, 22, 36, 33, 25, 13, 0, -13, -25,
    -33, -36, -33, -25, -13, 0, 13, 25, 33,
];

pub(super) const GIANT_MOLDORM_Y_VELOCITIES: [i8; 32] = [
    0, 9, 17, 22, 24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9, 0, 13, 25, 33, 36, 33, 25, 13,
    0, -13, -25, -33, -36, -33, -25, -13,
];

pub(super) const GIANT_MOLDORM_NEXT_DIRECTIONS: [u8; 16] =
    [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7];

pub(super) const GIANT_MOLDORM_SEG_A_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4084,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40a4,
        ext: 2,
    },
];

pub(super) const GIANT_MOLDORM_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(super) const GIANT_MOLDORM_HEAD_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4080,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x40a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40a0,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -6,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: -6,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 6,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 6,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -6,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: -6,
        char_flags: 0x4080,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 6,
        char_flags: 0x40a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 6,
        char_flags: 0x40a0,
        ext: 2,
    },
];

pub(super) const GIANT_MOLDORM_EYE_X_OFFSETS: [i16; 16] = [
    16, 15, 12, 6, 0, -6, -12, -13, -16, -13, -12, -6, 0, 6, 12, 15,
];

pub(super) const GIANT_MOLDORM_EYE_Y_OFFSETS: [i16; 16] = [
    0, 6, 12, 15, 16, 15, 12, 6, 0, -6, -12, -13, -16, -13, -12, -6,
];

pub(super) const GIANT_MOLDORM_EYE_CHARS: [u8; 16] = [
    0xaa, 0xaa, 0xa8, 0xa8, 0x8a, 0x8a, 0xa8, 0xa8, 0xaa, 0xaa, 0xa8, 0xa8, 0x8a, 0x8a, 0xa8, 0xa8,
];

pub(super) const GIANT_MOLDORM_EYE_FLAGS: [u8; 16] = [
    0, 0, 0, 0, 0x80, 0x80, 0x40, 0x40, 0x40, 0x40, 0xc0, 0xc0, 0, 0, 0x80, 0x80,
];

pub(super) const DRAW_FOUR_AROUND_ONE_DRAW_FRAMES: [DrawMultipleData; 30] = [
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

pub(super) const HELMASAUR_KING_MASK_X_OFFSETS: [i8; 2] = [-3, 11];

pub(super) const HELMASAUR_KING_MASK_CHARS: [u8; 8] =
    [0xce, 0xcf, 0xde, 0xde, 0xde, 0xde, 0xcf, 0xce];

pub(super) const HELMASAUR_KING_MASK_FLAGS: [u8; 2] = [0x3b, 0x7b];

pub(super) const HELMASAUR_KING_DRAW_D_DRAW_FRAMES: [DrawMultipleData; 19] = [
    DrawMultipleData {
        x: -24,
        y: -32,
        char_flags: 0x0b80,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -32,
        char_flags: 0x0b82,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -32,
        char_flags: 0x4b82,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: -32,
        char_flags: 0x4b80,
        ext: 2,
    },
    DrawMultipleData {
        x: -24,
        y: -16,
        char_flags: 0x0b84,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -16,
        char_flags: 0x0b86,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -16,
        char_flags: 0x4b86,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: -16,
        char_flags: 0x4b84,
        ext: 2,
    },
    DrawMultipleData {
        x: -24,
        y: 0,
        char_flags: 0x0b88,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0b8a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4b8a,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: 0,
        char_flags: 0x4b88,
        ext: 2,
    },
    DrawMultipleData {
        x: -24,
        y: 16,
        char_flags: 0x0b8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 16,
        char_flags: 0x0b8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 16,
        char_flags: 0x4b8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: 16,
        char_flags: 0x4b8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 32,
        char_flags: 0x0ba0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 32,
        char_flags: 0x4ba0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -40,
        char_flags: 0x0bac,
        ext: 2,
    },
];

pub(super) const HELMASAUR_KING_EXPLOSION_X_OFFSETS: [i8; 4] = [-28, -28, 28, 28];

pub(super) const HELMASAUR_KING_EXPLOSION_Y_OFFSETS: [i8; 4] = [-28, 4, -28, 4];

pub(super) const HELMASAUR_KING_EXPLOSION_CHARS: [u8; 4] = [0xa2, 0xa6, 0xa2, 0xa6];

pub(super) const HELMASAUR_KING_EXPLOSION_FLAGS: [u8; 4] = [0xb, 0xb, 0x4b, 0x4b];

pub(super) const HELMASAUR_KING_FIREBALL_Y_OFFSETS: [u8; 32] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 9, 8, 7, 6,
    5, 4, 3, 2, 1,
];

pub(super) const HELMASAUR_MASK_DRAW_FRAMES: [DrawMultipleData; 24] = [
    DrawMultipleData {
        x: -16,
        y: -5,
        char_flags: 0x0dae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x0dc0,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: -5,
        char_flags: 0x4dae,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: 11,
        char_flags: 0x0dc2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0dc4,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 11,
        char_flags: 0x4dc2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 27,
        char_flags: 0x0dc6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 27,
        char_flags: 0x4dc6,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: -5,
        char_flags: 0x0dae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x0dc0,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: -5,
        char_flags: 0x4dae,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: 11,
        char_flags: 0x0dc8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0dc4,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 11,
        char_flags: 0x4dc2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 27,
        char_flags: 0x0dc6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 27,
        char_flags: 0x4dc6,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: -5,
        char_flags: 0x0dae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x0dc0,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: -5,
        char_flags: 0x4dae,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: 11,
        char_flags: 0x0dc8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0dc4,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 11,
        char_flags: 0x4dc8,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 27,
        char_flags: 0x0dc6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 27,
        char_flags: 0x4dc6,
        ext: 2,
    },
];

pub(super) const STALFOS_KNIGHT_HEAD_CHARS: [u8; 4] = [0x66, 0x66, 0x46, 0x46];

pub(super) const STALFOS_KNIGHT_HEAD_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

pub(super) const SHOP_KEEPER_ITEM_WITH_PRICE_DRAW_FRAMES: [DrawMultipleData; 35] = [
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

pub(super) const BEAMOS_DRAW_Y_OFFSETS: [i8; 2] = [-16, 0];

pub(super) const BEAMOS_DRAW_CHARS: [u8; 2] = [0x48, 0x68];

pub(super) const CRAB_DRAW_X_OFFSETS: [i16; 4] = [-8, 8, -8, 8];

pub(super) const CRAB_DRAW_CHARS: [u8; 4] = [0x8e, 0x8e, 0xae, 0xae];

pub(super) const CRAB_DRAW_FLAGS: [u8; 4] = [0, 0x40, 0, 0x40];

pub(super) const POE_DRAW_X_OFFSETS: [i8; 2] = [9, -1];

pub(super) const POE_DRAW_CHARS: [u8; 4] = [0x7c, 0x80, 0xb7, 0x80];

pub(super) const BARI_BURST_X_VELOCITIES: [u8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, 0xf8, 0xf5, 0xf2, 0xf0, 0xf2, 0xf5, 0xf8,
];

pub(super) const BARI_BURST_Y_VELOCITIES: [u8; 16] = [
    0xf0, 0xf2, 0xf5, 0xf8, 0, 8, 11, 14, 16, 14, 11, 8, 0, 0xf7, 0xf5, 0xf2,
];

pub(super) const BARI_GRAPHICS: [u8; 2] = [0, 3];

pub(super) const RED_BARI_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const HARD_HAT_BEETLE_DRAW_FRAMES: [DrawMultipleData; 4] = [
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

pub(super) const ENEMY_ARROW_X_VELOCITIES: [u8; 8] = [0, 0, 16, 16, 0, 0, 0xf0, 0xf0];

pub(super) const ENEMY_ARROW_Y_VELOCITIES: [u8; 8] = [16, 16, 0, 0, 0xf0, 0xf0, 0, 0];

pub(super) const ENEMY_ARROW_DIRECTIONS: [u8; 4] = [0, 2, 1, 3];

pub(super) const ENEMY_ARROW_DRAW_X_OFFSETS: [i16; 8] = [-8, 0, 0, 8, 0, 0, 0, 0];

pub(super) const ENEMY_ARROW_DRAW_Y_OFFSETS: [i16; 8] = [0, 0, 0, 0, -8, 0, 0, 8];

pub(super) const ENEMY_ARROW_DRAW_CHARS: [u8; 32] = [
    0x3a, 0x3d, 0x3d, 0x3a, 0x2a, 0x2b, 0x2b, 0x2a, 0x7c, 0x6c, 0x6c, 0x7c, 0x7b, 0x6b, 0x6b, 0x7b,
    0x3a, 0x3b, 0x3b, 0x3a, 0x2a, 0x3c, 0x3c, 0x2a, 0x81, 0x80, 0x80, 0x81, 0x91, 0x90, 0x90, 0x91,
];

pub(super) const ENEMY_ARROW_DRAW_FLAGS: [u8; 32] = [
    8, 8, 0x48, 0x48, 8, 8, 0x88, 0x88, 9, 0x49, 9, 0x49, 9, 0x89, 9, 0x89, 8, 0x88, 0xc8, 0x48, 8,
    8, 0x88, 0x88, 0x49, 0x49, 9, 9, 0x89, 0x89, 9, 9,
];

pub(super) const OCTOROCK_DRAW_X_OFFSETS: [i8; 9] = [8, 0, 4, 8, 0, 4, 9, -1, 4];

pub(super) const OCTOROCK_DRAW_Y_OFFSETS: [i8; 9] = [6, 6, 9, 6, 6, 9, 6, 6, 9];

pub(super) const OCTOROCK_DRAW_CHARS: [u8; 9] =
    [0xbb, 0xbb, 0xba, 0xab, 0xab, 0xaa, 0xa9, 0xa9, 0xb9];

pub(super) const OCTOROCK_DRAW_FLAGS: [u8; 9] =
    [0x65, 0x25, 0x25, 0x65, 0x25, 0x25, 0x65, 0x25, 0x25];

pub(super) const FLUTE_BOY_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -1,
        y: -1,
        char_flags: 0x0abe,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: -1,
        y: -1,
        char_flags: 0x0abe,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0abf,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: -1,
        y: -1,
        char_flags: 0x0abe,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: -1,
        y: -1,
        char_flags: 0x0abe,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0abf,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
];

pub(super) const FLUTE_AARDVARK_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: -16,
        char_flags: 0x06e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x06c8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -16,
        char_flags: 0x06e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x06ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -16,
        char_flags: 0x06e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x06ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -16,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00dc,
        ext: 2,
    },
];

pub(super) const DUST_CLOUD_DRAW_FRAMES: [DrawMultipleData; 24] = [
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

pub(super) const DUST_CLOUD_GRAPHICS: [u8; 9] = [0, 1, 2, 3, 4, 5, 1, 0, 0xff];

pub(super) const LANDMINE_DRAW_FRAMES: [DrawMultipleData; 2] = [
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

pub(super) const ARMOS_DRAW_FRAMES: [DrawMultipleData; 2] = [
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

pub(super) const ARMOS_KNIGHT_DRAW_X_OFFSETS: [i8; 24] = [
    -8, 8, -8, 8, -10, 10, -10, 10, -10, 10, -10, 10, -12, 12, -12, 12, -14, 14, -14, 14, -16, 24,
    -16, 24,
];

pub(super) const ARMOS_KNIGHT_DRAW_Y_OFFSETS: [i8; 24] = [
    -8, -8, 8, 8, -10, -10, 10, 10, -10, -10, 10, 10, -12, -12, 12, 12, -14, -14, 14, 14, -16, -16,
    24, 24,
];

pub(super) const ARMOS_KNIGHT_DRAW_CHARS: [u8; 24] = [
    0xc0, 0xc2, 0xe0, 0xe2, 0xc0, 0xc2, 0xe0, 0xe2, 0xc4, 0xc4, 0xc4, 0xc4, 0xc6, 0xc6, 0xc6, 0xc6,
    0xc8, 0xc8, 0xc8, 0xc8, 0xd8, 0xd8, 0xd8, 0xd8,
];

pub(super) const ARMOS_KNIGHT_DRAW_FLAGS: [u8; 24] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0x40, 0,
    0xc0, 0x80,
];

pub(super) const ARMOS_KNIGHT_DRAW_SIZES: [u8; 24] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0,
];

pub(super) const BOULDER_DRAW_FRAMES: [DrawMultipleData; 16] = [
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

pub(super) const FLAME_DRAW_FRAMES: [DrawMultipleData; 12] = [
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

pub(super) const ENERGY_BALL_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const WIZZBEAM_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const FREEZOR_DRAW_FRAMES0: [DrawMultipleData; 28] = [
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

pub(super) const FREEZOR_DRAW_FRAMES1: [DrawMultipleData; 8] = [
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

pub(super) const ROPA_DRAW_FRAMES: [DrawMultipleData; 12] = [
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

pub(super) const ZAZAK_DRAW_CHARS: [u8; 8] = [0x82, 0x82, 0x80, 0x84, 0x88, 0x88, 0x86, 0x84];

pub(super) const ZAZAK_DRAW_FLAGS: [u8; 8] = [0x40, 0, 0, 0, 0x40, 0, 0, 0];

pub(super) const ZAZAK_DRAW_FRAMES: [DrawMultipleData; 24] = [
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

pub(super) const LEEVER_DRAW_COUNTS: [u8; 14] = [1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1];

pub(super) const LEEVER_DRAW_X_OFFSETS: [i8; 56] = [
    2, 6, 6, 6, 0, 8, 8, 8, 0, 8, 8, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 0, 0, 8, 0, 0, 0, 8, 0, 0, 0, 8,
    0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(super) const LEEVER_DRAW_Y_OFFSETS: [i8; 56] = [
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 5, 5, 8, 8, 5, 5, 8, 8, 2, 2, 8, 8, 1, 1, 8, 8, 0, 0, 8, 8,
    -1, -1, 8, 8, 8, -2, -2, 0, 8, -2, -2, 0, 8, -2, -2, 0, 8, -2, -2, 0, 8, -2, -2, 0,
];

pub(super) const LEEVER_DRAW_CHARS: [u8; 56] = [
    0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x38, 0x38, 0x38, 0x38, 8, 9, 0x28, 0x28, 8, 9,
    0xd9, 0xd9, 8, 8, 0xd8, 0xd8, 8, 8, 0xda, 0xda, 6, 6, 0xd9, 0xd9, 0x26, 0x26, 0xd8, 0xd8, 0x6c,
    6, 6, 0, 0x6c, 0x26, 0x26, 0, 0x6c, 6, 6, 0, 0x6c, 0x26, 0x26, 0, 0x6c, 8, 8, 0,
];

pub(super) const LEEVER_DRAW_FLAGS: [u8; 56] = [
    1, 0x41, 0x41, 0x41, 1, 0x41, 0x41, 0x41, 1, 0x41, 0x41, 0x41, 1, 1, 1, 0x41, 1, 1, 0, 0x40, 1,
    1, 0, 0x40, 1, 1, 0, 0x40, 1, 1, 0, 0x40, 0, 1, 0, 0x40, 6, 0x41, 0x41, 0, 6, 0x41, 0x41, 0, 6,
    1, 1, 0, 6, 1, 1, 0, 6, 1, 1, 0,
];

pub(super) const LEEVER_DRAW_SIZES: [u8; 56] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 2, 2, 2, 0, 2, 2, 2, 0, 2, 2, 2, 0, 2, 2, 2, 0,
];

pub(super) const PENGATOR_DRAW_FRAMES0: [DrawMultipleData; 40] = [
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

pub(super) const PENGATOR_DRAW_FRAMES1: [DrawMultipleData; 4] = [
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

pub(super) const SPIKE_ROLLER_DRAW_X_OFFSETS: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0,
    0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70,
];

pub(super) const SPIKE_ROLLER_DRAW_Y_OFFSETS: [u8; 32] = [
    0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(super) const SPIKE_ROLLER_DRAW_CHARS: [u8; 32] = [
    0x8e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e, 0x8e, 0x8e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e, 0x9e, 0x8e,
    0x88, 0x89, 0x89, 0x89, 0x89, 0x89, 0x89, 0x88, 0x88, 0x89, 0x89, 0x89, 0x89, 0x89, 0x89, 0x88,
];

pub(super) const SPIKE_ROLLER_DRAW_FLAGS: [u8; 32] = [
    0, 0, 0, 0x80, 0, 0, 0, 0x80, 0x40, 0x40, 0x40, 0xc0, 0x40, 0x40, 0x40, 0xc0, 0, 0, 0, 0x40, 0,
    0, 0, 0x40, 0x80, 0x80, 0x80, 0xc0, 0x80, 0x80, 0x80, 0xc0,
];

pub(super) const MEDALLION_TABLET_DRAW_FRAMES: [DrawMultipleData; 20] = [
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

// Shared world, item, and enemy draw tables promoted from sprite_main_draw.rs method bodies.
pub(super) const SURFACE_XY: [i8; 8] = [-32, -24, -16, -8, 8, 16, 24, 32];

pub(super) const MOVABLE_STATUE_DIRECTIONS: [u8; 4] = [4, 6, 0, 2];

pub(super) const MOVABLE_STATUE_JOYPAD_MASKS: [u8; 4] = [1, 2, 4, 8];

pub(super) const MOVABLE_STATUE_X_VELOCITIES: [u8; 4] = [0xf0, 16, 0, 0];

pub(super) const MOVABLE_STATUE_Y_VELOCITIES: [u8; 4] = [0, 0, 0xf0, 16];

pub(super) const MOVABLE_STATUE_SWITCH_X_OFFSETS: [u8; 4] = [3, 12, 3, 12];

pub(super) const MOVABLE_STATUE_SWITCH_Y_OFFSETS: [u8; 4] = [3, 3, 12, 12];

pub(super) const MOVABLE_STATUE_DRAW_FRAMES: [DrawMultipleData; 3] = [
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

pub(super) const MOVABLE_MANTLE_X_OFFSETS: [u8; 6] = [0, 0x10, 0x20, 0, 0x10, 0x20];

pub(super) const MOVABLE_MANTLE_Y_OFFSETS: [u8; 6] = [0, 0, 0, 0x10, 0x10, 0x10];

pub(super) const MOVABLE_MANTLE_CHARS: [u8; 6] = [0x0c, 0x0e, 0x0c, 0x2c, 0x2e, 0x2c];

pub(super) const MOVABLE_MANTLE_FLAGS: [u8; 6] = [0x31, 0x31, 0x71, 0x31, 0x31, 0x71];

pub(super) const FISH_DRAW_FRAMES: [DrawMultipleData; 16] = [
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

pub(super) const FISH_DRAW_FRAMES2: [DrawMultipleData; 9] = [
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

pub(super) const CHIMNEY_SMOKE_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const VULTURE_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const RAVEN_ASCEND_TIMERS: [u8; 2] = [16, 248];

pub(super) const MAGIC_POWDER_DRAW_FRAMES: [DrawMultipleData; 2] = [
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

pub(super) const GREEN_POTION_ITEM_DRAW_FRAMES: [DrawMultipleData; 3] = [
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

pub(super) const BLUE_POTION_ITEM_DRAW_FRAMES: [DrawMultipleData; 4] = [
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

pub(super) const RED_POTION_ITEM_DRAW_FRAMES: [DrawMultipleData; 4] = [
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

pub(super) const UP_PULL_ANIMATION_DELAYS: [u8; 10] = [8, 24, 4, 4, 4, 4, 4, 4, 2, 10];

pub(super) const UP_PULL_PLAYER_ACTION_STATES: [u8; 10] = [6, 7, 8, 8, 8, 8, 8, 9, 9, 9];

pub(super) const DOWN_PULL_ANIMATION_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5];

pub(super) const DOWN_PULL_PLAYER_ACTION_STATES: [u8; 12] = [1, 1, 2, 2, 3, 3, 1, 1, 4, 4, 5, 5];

pub(super) const BAD_PULL_DOWN_SWITCH_X_OFFSETS: [i8; 5] = [-4, 12, 0, -4, 4];

pub(super) const BAD_PULL_DOWN_SWITCH_Y_OFFSETS: [i8; 5] = [-3, -3, 0, 5, 5];

pub(super) const BAD_PULL_DOWN_SWITCH_DRAW_CHARS: [u8; 5] = [0xd2, 0xd2, 0xc4, 0xe4, 0xe4];

pub(super) const BAD_PULL_DOWN_SWITCH_DRAW_FLAGS: [u8; 5] = [0x40, 0, 0, 0x40, 0];

pub(super) const BAD_PULL_DOWN_SWITCH_DRAW_SIZES: [u8; 5] = [0, 0, 2, 2, 2];

pub(super) const BAD_PULL_SWITCH_CENTER_Y_OFFSETS: [u8; 6] = [0, 1, 2, 3, 4, 5];

pub(super) const BAD_PULL_UP_SWITCH_CHARS: [u8; 2] = [0xa2, 0xa4];

pub(super) const BAD_PULL_SWITCH_TOP_Y_OFFSETS: [u8; 6] = [0, 1, 2, 3, 4, 5];

pub(super) const GOOD_PULL_SWITCH_BOTTOM_Y_OFFSETS: [u8; 14] =
    [1, 1, 2, 3, 2, 3, 4, 5, 6, 7, 6, 7, 7, 7];

pub(super) const BUG_NET_KID_DRAW_FRAMES: [DrawMultipleData; 18] = [
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x0027,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 6,
        char_flags: 0x040a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 6,
        char_flags: 0x440a,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 14,
        char_flags: 0x840a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 14,
        char_flags: 0xc40a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 6,
        char_flags: 0x040a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 6,
        char_flags: 0x440a,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 14,
        char_flags: 0x840a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 14,
        char_flags: 0xc40a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x002e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x002e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x040a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x440a,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 14,
        char_flags: 0x840a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 14,
        char_flags: 0xc40a,
        ext: 2,
    },
];

pub(super) const BOMBER_DRAW_FRAMES: [DrawMultipleData; 22] = [
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

pub(super) const BUMPER_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const FAKE_SWORD_DRAW_FRAMES: [DrawMultipleData; 2] = [
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

pub(super) const SWAMOLA_TARGET_X_OFFSETS: [i8; 9] = [0, 0, 32, 32, 32, 0, -32, -32, -32];

pub(super) const SWAMOLA_TARGET_Y_OFFSETS: [i8; 9] = [0, -32, -32, 0, 32, 32, 32, 0, -32];

pub(super) const ZORO_X_VELOCITIES: [i8; 4] = [16, -16, 0, 0];

pub(super) const WIZZROBE_BEAM_XY_VELOCITIES: [i8; 6] = [32, -32, 0, 0, 32, -32];

pub(super) const STAL_DRAW_FRAMES: [DrawMultipleData; 6] = [
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

pub(super) const RABBIT_BEAM_GFX: [u8; 6] = [0xd7, 0xd7, 0xd7, 0x91, 0x91, 0x91];

pub(super) const HEART_PIECE_MESSAGES: [u16; 4] = [0x158, 0x155, 0x156, 0x157];

pub(super) const GIBO_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(super) const GIBO_ALT_OAM_FLAGS: [u8; 2] = [11, 7];

pub(super) const GIBO_DRAW_FRAMES: [DrawMultipleData; 32] = [
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

pub(super) const LASER_EYE_DRAW_FRAMES: [DrawMultipleData; 24] = [
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

pub(super) const GIBDO_DRAW_FRAMES: [DrawMultipleData; 24] = [
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

pub(super) const FIREBAT_DRAW_X_OFFSETS: [i8; 2] = [-8, 8];

pub(super) const FIREBAT_DRAW_CHARS: [u8; 7] = [0x88, 0x88, 0x8a, 0x8c, 0x68, 0xaa, 0xa8];

pub(super) const FIREBAT_DRAW_FLAGS: [u8; 14] = [
    0, 0xc0, 0x80, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40,
];

pub(super) const FIRE_PHLEGM_DRAW_FRAMES: [DrawMultipleData; 16] = [
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

pub(super) const FLYING_TILE_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const BULLY_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x46e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x46e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x46e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x46c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x06e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x06e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x06e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x06c4,
        ext: 2,
    },
];

pub(super) const RUNNING_MAN_RECOIL_X_VELOCITIES: [i8; 2] = [-24, 24];

pub(super) const TALKING_TREE_EYE_BASE_X_OFFSETS: [i8; 2] = [9, -9];

pub(super) const DASH_TREE_TOP_GRID_CHAR_FLAGS: [u16; 16] = [
    0x3100, 0x3102, 0x7102, 0x7100, 0x3120, 0x3122, 0x7122, 0x7120, 0x3104, 0x3106, 0x7106, 0x7104,
    0x3124, 0x3126, 0x7126, 0x7124,
];

pub(super) const OCTOROK_SPIT_POSE_BY_DELAY: [u8; 20] =
    [0, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 1, 1, 0];

pub(super) const FOUR_WAY_OCTOROK_SPIT_POSE_BY_DELAY: [u8; 10] = [2, 2, 2, 2, 2, 2, 2, 2, 1, 0];

pub(super) const HINOX_RANDOM_DIRECTIONS: [u8; 8] = [2, 3, 3, 2, 0, 1, 1, 0];

pub(super) const HINOX_BOMB_X_OFFSETS: [i8; 4] = [8, -8, -13, 13];

pub(super) const HINOX_BOMB_Y_OFFSETS: [i8; 4] = [-11, -11, -16, -16];

pub(super) const HINOX_BOMB_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(super) const HINOX_BOMB_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(super) const ANTIFAIRY_CIRCLE_VELOCITY_TARGETS: [u8; 2] = [18, (-18i8) as u8];

pub(super) const VULTURE_GRAPHICS: [u8; 4] = [1, 2, 3, 2];

pub(super) const DEAD_ROCK_GRAPHICS: [u8; 9] = [0, 1, 0, 1, 2, 2, 3, 3, 4];

pub(super) const DEAD_ROCK_OAM_FLAGS: [u8; 9] = [0x40, 0x40, 0, 0, 0, 0x40, 0, 0x40, 0];

pub(super) const DEAD_ROCK_X_VELOCITIES: [u8; 4] = [32, 0xe0, 0, 0];

pub(super) const DEAD_ROCK_Y_VELOCITIES: [u8; 4] = [0, 0, 32, 0xe0];

pub(super) const CHOMP_Z_VELOCITY_TARGETS: [i8; 2] = [8, -8];

pub(super) const CHOMP_X_VELOCITY_TARGETS: [i8; 4] = [16, -16, 28, -28];

pub(super) const CRYSTAL_SWITCH_PAL: [u8; 2] = [2, 4];

pub(super) const RAT_GRAPHICS_BY_ANIM_STATE: [u8; 16] =
    [0, 0, 3, 3, 1, 2, 4, 5, 1, 2, 4, 5, 0, 0, 3, 3];

pub(super) const RAT_IDLE_ANIM_STATES: [u8; 8] = [10, 11, 6, 7, 2, 3, 14, 15];

pub(super) const RAT_RUN_ANIM_STATES: [u8; 8] = [8, 9, 4, 5, 0, 1, 12, 13];

pub(super) const RAT_WALL_TURN_DIRECTIONS: [u8; 4] = [2, 3, 1, 0];

pub(super) const KEESE_ATTACK_START_PHASES: [i8; 4] = [2, 10, 6, 14];

pub(super) const KEESE_ATTACK_X_VELOCITIES: [i8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
];

pub(super) const KEESE_ATTACK_Y_VELOCITIES: [i8; 16] = [
    -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
];

pub(super) const WARP_VORTEX_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(super) const CUCCO_CALM_CIRCLE_X_VELOCITIES: [i8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
];

pub(super) const CUCCO_CALM_CIRCLE_Y_VELOCITIES: [i8; 16] = [
    -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
];

pub(super) const WALKING_ZORA_SPAWN_X_OFFSETS: [i8; 4] = [8, -8, 0, 0];

pub(super) const WALKING_ZORA_SPAWN_Y_OFFSETS: [i8; 4] = [0, 0, 8, -8];

pub(super) const WALKING_ZORA_SPAWN_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(super) const WALKING_ZORA_SPAWN_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(super) const SOLDIER_B_STEP_X_VELOCITIES: [i8; 8] = [1, 1, -1, -1, -1, -1, 1, 1];

pub(super) const SOLDIER_B_STEP_Y_VELOCITIES: [i8; 8] = [-1, 1, 1, -1, -1, 1, 1, -1];

pub(super) const SOLDIER_B_FAST_X_VELOCITIES: [i8; 8] = [8, 0, -8, 0, -8, 0, 8, 0];

pub(super) const SOLDIER_B_FAST_Y_VELOCITIES: [i8; 8] = [0, 8, 0, -8, 0, 8, 0, -8];

pub(super) const ZAZAK_DIR2: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];

pub(super) const DESERT_BARRIER_X_VELOCITY_TARGETS: [u8; 2] = [16, (-16i8) as u8];

pub(super) const ALTAR_ZELDA_HEAD_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

// Remaining Rat, Keese, and Soldier-B behavior tables promoted from sprite_main_draw.rs.
pub(super) const RAT_OAM_FLIP_BY_ANIM_STATE: [u8; 16] = [
    0, 0x40, 0, 0x40, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x80, 0xc0, 0x80, 0xc0,
];

pub(super) const KEESE_ORBIT_DIRECTION_STEPS: [i8; 2] = [1, -1];

pub(super) const SOLDIER_B_FAST_COLLISION_MASKS: [u8; 8] = [1, 4, 2, 8, 2, 4, 1, 8];

pub(super) const SOLDIER_B_COLLISION_MASKS: [u8; 8] = [8, 1, 4, 2, 8, 2, 4, 1];

pub(super) const SOLDIER_B_FAST_NEXT_DIRECTIONS: [u8; 8] = [1, 2, 3, 0, 5, 6, 7, 4];

pub(super) const SOLDIER_B_NEXT_DIRECTIONS: [u8; 8] = [3, 0, 1, 2, 7, 4, 5, 6];

// Generic DrawMultipleData tables promoted from sprite_main_draw.rs draw helpers.
pub(super) const SPRITE_CATFISH_SPLASH_OF_WATER_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const SWAMOLA_RIPPLES_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const WALL_MASTER_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const WIZZROBE_DRAW_FRAMES: [DrawMultipleData; 24] = [
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

pub(super) const STALFOS_BONE_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const STALFOS_DRAW_FRAMES: [DrawMultipleData; 36] = [
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

pub(super) const STALFOS_KNIGHT_DRAW_FRAMES: [DrawMultipleData; 35] = [
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

pub(super) const TRIDENT_DRAW_FRAMES: [DrawMultipleData; 50] = [
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

pub(super) const BABUSU_DRAW_FRAMES: [DrawMultipleData; 40] = [
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

pub(super) const LADY_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x00e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x00e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x00e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x40e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x40e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40e6,
        ext: 2,
    },
];

pub(super) const YOUNG_SNITCH_LADY_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0026,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0026,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0024,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0024,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0028,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0028,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x00e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4028,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x4028,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40e6,
        ext: 2,
    },
];

pub(super) const CUKEMAN_DRAW_FRAMES: [DrawMultipleData; 18] = [
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

pub(super) const SNAP_DRAGON_DRAW_FRAMES: [DrawMultipleData; 32] = [
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

pub(super) const LYNEL_DRAW_FRAMES: [DrawMultipleData; 33] = [
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

pub(super) const GORIYA_DRAW_FRAMES_2: [DrawMultipleData; 3] = [
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

pub(super) const GORIYA_DRAW_FRAMES: [DrawMultipleData; 32] = [
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

pub(super) const KYAMERON_DRAW_FRAMES: [DrawMultipleData; 28] = [
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

pub(super) const HOBO_DRAW_FRAMES: [DrawMultipleData; 12] = [
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00ab,
        ext: 0,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: -11,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00ab,
        ext: 0,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
];

pub(super) const RUNNING_MAN_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x002c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x08ee,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x002c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x48ee,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x002a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x08ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x002a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x48ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x002e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x08cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x002e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x08ce,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x402e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x48cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x402e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x48ce,
        ext: 2,
    },
];

pub(super) const ELDER_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x00a0,
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
        y: -8,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40a4,
        ext: 2,
    },
];

pub(super) const SHOPKEEPER_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0c00,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0c10,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0c00,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4c10,
        ext: 2,
    },
];

pub(super) const FLUTE_BOY_FATHER_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -6,
        char_flags: 0x0086,
        ext: 2,
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
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0088,
        ext: 2,
    },
];

pub(super) const FLUTE_BOY_OSTRICH_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x0081,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x00a3,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x0081,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x00a1,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x0081,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x0083,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -7,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -7,
        char_flags: 0x0081,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 9,
        char_flags: 0x00a3,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 9,
        char_flags: 0x00a4,
        ext: 2,
    },
];

pub(super) const OLD_MOUNTAIN_MAN_DRAW_FRAMES_0: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x00ae,
        ext: 2,
    },
];

pub(super) const OLD_MOUNTAIN_MAN_DRAW_FRAMES_1: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: 0,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: 1,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: 0,
        char_flags: 0x4120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: 1,
        char_flags: 0x4120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x4122,
        ext: 2,
    },
];

pub(super) const INN_KEEPER_DRAW_FRAMES: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ca,
        ext: 2,
    },
];

pub(super) const ELDER_WIFE_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 5,
        char_flags: 0x0028,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 5,
        char_flags: 0x4028,
        ext: 2,
    },
];

pub(super) const MIDDLE_AGED_MAN_DRAW_FRAMES: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00ea,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ec,
        ext: 2,
    },
];

pub(super) const BLIND_HIDEOUT_GUY_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x000c,
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
        y: -8,
        char_flags: 0x000c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40ca,
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
        char_flags: 0x00ca,
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
        char_flags: 0x40ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x000e,
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
        y: -8,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x400e,
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
        y: -8,
        char_flags: 0x400e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40ca,
        ext: 2,
    },
];

pub(super) const SWEEPING_LADY_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x008e,
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
        y: -8,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x008c,
        ext: 2,
    },
];

pub(super) const FORTUNE_TELLER_DRAW_FRAMES: [DrawMultipleData; 12] = [
    DrawMultipleData {
        x: 0,
        y: -48,
        char_flags: 0x000c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -32,
        char_flags: 0x002c,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -32,
        char_flags: 0x402c,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -48,
        char_flags: 0x000a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -32,
        char_flags: 0x002a,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -32,
        char_flags: 0x402a,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -40,
        char_flags: 0x0066,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -40,
        char_flags: 0x4066,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -40,
        char_flags: 0x0066,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -40,
        char_flags: 0x0068,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -40,
        char_flags: 0x4068,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -40,
        char_flags: 0x0068,
        ext: 2,
    },
];

pub(super) const MAZE_GAME_GUY_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
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
        char_flags: 0x0020,
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
        char_flags: 0x0020,
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
        char_flags: 0x0020,
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
        char_flags: 0x0020,
        ext: 2,
    },
];

pub(super) const DRINKING_GUY_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: 8,
        y: 2,
        char_flags: 0x00ae,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0822,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0006,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: 0,
        char_flags: 0x00af,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0822,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0006,
        ext: 2,
    },
];

pub(super) const TALKING_TREE_DRAW_FRAMES: [DrawMultipleData; 12] = [
    DrawMultipleData {
        x: 1,
        y: -1,
        char_flags: 0x00e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: 7,
        char_flags: 0x00f8,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: -1,
        char_flags: 0x40e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: 7,
        char_flags: 0x40f8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -1,
        char_flags: 0x00e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 7,
        char_flags: 0x00f8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -1,
        char_flags: 0x40e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x40f8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 7,
        char_flags: 0x00f8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x40f8,
        ext: 0,
    },
];

pub(super) const DIGGING_GAME_GUY_DRAW_FRAMES: [DrawMultipleData; 9] = [
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

pub(super) const BOMB_SHOP_ENTITY_DRAW_FRAMES: [DrawMultipleData; 6] = [
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

pub(super) const STORY_TELLER_1_DRAW_FRAMES: [DrawMultipleData; 10] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a4a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4a6e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a24,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4a24,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0804,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4804,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a6a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a6c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a0e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a2e,
        ext: 2,
    },
];

pub(super) const SMITHY_FROG_DRAW_FRAMES: [DrawMultipleData; 1] = [DrawMultipleData {
    x: 0,
    y: 0,
    char_flags: 0x00c8,
    ext: 2,
}];

pub(super) const SMITHY_SPARK_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: 0,
        y: 3,
        char_flags: 0x41aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -1,
        char_flags: 0x41aa,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x0190,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 0,
        char_flags: 0x4190,
        ext: 0,
    },
    DrawMultipleData {
        x: -5,
        y: -2,
        char_flags: 0x0191,
        ext: 0,
    },
    DrawMultipleData {
        x: 13,
        y: -2,
        char_flags: 0x0191,
        ext: 0,
    },
];

pub(super) const QUARREL_BROS_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -12,
        char_flags: 0x0004,
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
        y: -11,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x400a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -12,
        char_flags: 0x0004,
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
        y: -11,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x400a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -12,
        char_flags: 0x0008,
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
        y: -11,
        char_flags: 0x0008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x400a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -12,
        char_flags: 0x4008,
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
        y: -11,
        char_flags: 0x4008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x400a,
        ext: 2,
    },
];

pub(super) const LUMBERJACKS_DRAW_FRAMES: [DrawMultipleData; 33] = [
    DrawMultipleData {
        x: -23,
        y: 5,
        char_flags: 0x02be,
        ext: 0,
    },
    DrawMultipleData {
        x: -15,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: -7,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 17,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 25,
        y: 5,
        char_flags: 0x42be,
        ext: 0,
    },
    DrawMultipleData {
        x: -32,
        y: -8,
        char_flags: 0x40a8,
        ext: 2,
    },
    DrawMultipleData {
        x: -32,
        y: 4,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 30,
        y: -8,
        char_flags: 0x00a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 31,
        y: 4,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 5,
        char_flags: 0x02be,
        ext: 0,
    },
    DrawMultipleData {
        x: -11,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 13,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 21,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 29,
        y: 5,
        char_flags: 0x42be,
        ext: 0,
    },
    DrawMultipleData {
        x: -31,
        y: -8,
        char_flags: 0x40a8,
        ext: 2,
    },
    DrawMultipleData {
        x: -32,
        y: 4,
        char_flags: 0x40a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 31,
        y: -8,
        char_flags: 0x00a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 31,
        y: 4,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 5,
        char_flags: 0x02be,
        ext: 0,
    },
    DrawMultipleData {
        x: -11,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 13,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 21,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 29,
        y: 5,
        char_flags: 0x42be,
        ext: 0,
    },
    DrawMultipleData {
        x: -32,
        y: -8,
        char_flags: 0x400e,
        ext: 2,
    },
    DrawMultipleData {
        x: -32,
        y: 4,
        char_flags: 0x40a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 32,
        y: -8,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: 31,
        y: 4,
        char_flags: 0x00a6,
        ext: 2,
    },
];

pub(super) const GREAT_CATFISH_DRAW_FRAMES: [DrawMultipleData; 28] = [
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

pub(super) const BIG_FAERIE_DRAW_FRAMES: [DrawMultipleData; 16] = [
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

pub(super) const SPIKE_TRAP_DRAW_FRAMES: [DrawMultipleData; 4] = [
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

pub(super) const DESERT_BARRIER_DRAW_FRAMES: [DrawMultipleData; 4] = [
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

pub(super) const SAGE_MANTLE_DRAW_FRAMES: [DrawMultipleData; 4] = [
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

pub(super) const TROUGH_BOY_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0882,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0882,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4880,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0880,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
];

pub(super) const RETREAT_BAT_DRAW_FRAMES: [DrawMultipleData; 18] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x044b,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: -4,
        char_flags: 0x045b,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: -4,
        char_flags: 0x0464,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: -4,
        char_flags: 0x0449,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -9,
        char_flags: 0x046c,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -9,
        char_flags: 0x446c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -7,
        char_flags: 0x044c,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -7,
        char_flags: 0x444c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -9,
        char_flags: 0x0444,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -9,
        char_flags: 0x4444,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0462,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4462,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -7,
        char_flags: 0x0460,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -7,
        char_flags: 0x4460,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x044e,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x444e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x046e,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 16,
        char_flags: 0x446e,
        ext: 2,
    },
];

pub(super) const GANON_BAT_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0560,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4560,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0562,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4562,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0544,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4544,
        ext: 2,
    },
];

pub(super) const EVIL_BARRIER_DRAW_FRAMES: [DrawMultipleData; 45] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: 3,
        char_flags: 0x00ca,
        ext: 0,
    },
    DrawMultipleData {
        x: -29,
        y: 11,
        char_flags: 0x00da,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 3,
        char_flags: 0x40ca,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40da,
        ext: 0,
    },
    DrawMultipleData {
        x: -24,
        y: -2,
        char_flags: 0x00e6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -2,
        char_flags: 0x00e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -2,
        char_flags: 0x40e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: -2,
        char_flags: 0x40e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: 3,
        char_flags: 0x00cb,
        ext: 0,
    },
    DrawMultipleData {
        x: -29,
        y: 11,
        char_flags: 0x00db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 3,
        char_flags: 0x40cb,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: 3,
        char_flags: 0x00cb,
        ext: 0,
    },
    DrawMultipleData {
        x: -29,
        y: 11,
        char_flags: 0x00db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 3,
        char_flags: 0x40cb,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: -24,
        y: -2,
        char_flags: 0x80e6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -2,
        char_flags: 0x80e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -2,
        char_flags: 0xc0e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: -2,
        char_flags: 0xc0e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: 3,
        char_flags: 0x00ca,
        ext: 0,
    },
    DrawMultipleData {
        x: -29,
        y: 11,
        char_flags: 0x00da,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 3,
        char_flags: 0x40ca,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40da,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: 3,
        char_flags: 0x00cb,
        ext: 0,
    },
    DrawMultipleData {
        x: -29,
        y: 11,
        char_flags: 0x00db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 3,
        char_flags: 0x40cb,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
];

pub(super) const CHATTY_AGAHNIM_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0b82,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4b82,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x0ba2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4ba2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0b80,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4b80,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x0ba0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4ba0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0b80,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4b82,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x0ba0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4ba2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0b82,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4b80,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x0ba2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4ba0,
        ext: 2,
    },
];

pub(super) const FAERIE_QUEEN_DRAW_FRAMES: [DrawMultipleData; 20] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x40e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x40e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x40e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x00eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 16,
        char_flags: 0x40eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 32,
        char_flags: 0x00ed,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 32,
        char_flags: 0x40ed,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ef,
        ext: 0,
    },
    DrawMultipleData {
        x: 24,
        y: 0,
        char_flags: 0x40ef,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x00ff,
        ext: 0,
    },
    DrawMultipleData {
        x: 24,
        y: 8,
        char_flags: 0x40ff,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x40e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x00eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 16,
        char_flags: 0x40eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 32,
        char_flags: 0x00ed,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 32,
        char_flags: 0x40ed,
        ext: 2,
    },
];

pub(super) const CRYSTAL_MAIDEN_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x4120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x4120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x4122,
        ext: 2,
    },
];

pub(super) const KHOLDSTARE_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: -7,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: -7,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: 7,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: 7,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: -7,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: -7,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: 7,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: 7,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00a6,
        ext: 2,
    },
];

pub(super) const EYEGORE_DRAW_FRAMES: [DrawMultipleData; 48] = [
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

pub(super) const PIKIT_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const MOBLIN_DRAW_FRAMES: [DrawMultipleData; 48] = [
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

pub(super) const ZORA_DRAW_X_OFFSETS: [i8; 26] = [
    4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -4, 11, 0, 4, -8, 18, -8, 18,
];

pub(super) const ZORA_DRAW_Y_OFFSETS: [i8; 26] = [
    4, 4, 0, 0, 0, 0, 0, -3, 0, -3, -3, -3, -3, -3, -3, -3, -6, -6, -8, -9, -3, 5, -10, -11, -10,
    -11,
];

pub(super) const ZORA_DRAW_CHARS: [u8; 26] = [
    0xa8, 0xa8, 0x88, 0x88, 0x88, 0x88, 0x88, 0xa4, 0x88, 0xa4, 0xa4, 0xa4, 0xa6, 0xa6, 0xa4, 0xc0,
    0x8a, 0x8a, 0xae, 0xaf, 0xa6, 0x8d, 0xcf, 0xcf, 0xdf, 0xdf,
];

pub(super) const ZORA_DRAW_FLAGS: [u8; 26] = [
    0x25, 0x25, 0x25, 0x25, 0xe5, 0xe5, 0x25, 0x20, 0xe5, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x24,
    0x25, 0x25, 0x24, 0x64, 0x20, 0x26, 0x24, 0x64, 0x24, 0x64,
];

pub(super) const ZORA_DRAW_BIG: [u8; 26] = [
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 2, 0, 0, 0, 0, 0,
];

pub(super) const SPRITE_FIREBALL_OFFSETS: [u8; 4] = [3, 2, 0, 0];

pub(super) const SPRITE_FIREBALL_X_OFFSETS: [i8; 4] = [4, 4, -4, 16];

pub(super) const SPRITE_FIREBALL_Y_OFFSETS: [i8; 4] = [0, 16, 8, 8];

pub(super) const SPRITE_FIREBALL_WIDTHS: [u8; 4] = [8, 8, 4, 4];

pub(super) const SPRITE_FIREBALL_HEIGHTS: [u8; 4] = [4, 4, 8, 8];

pub(super) const SPRITE_ZORA_MAIN_ATTACK_GFX: [u8; 8] = [5, 5, 6, 10, 6, 5, 5, 5];

pub(super) const SPRITE_ZORA_MAIN_SUBMERGE_GFX: [u8; 12] = [12, 11, 9, 8, 7, 0, 0, 0, 0, 0, 0, 0];

pub(super) const SPRITE_SPAWN_BIG_SPLASH_X_OFFSETS: [i8; 8] = [-8, -5, 4, 13, 16, 13, 4, -5];

pub(super) const SPRITE_SPAWN_BIG_SPLASH_Y_OFFSETS: [i8; 8] = [4, -5, -8, -5, 4, 13, 16, 13];

pub(super) const SPRITE_SPAWN_BIG_SPLASH_LOCAL_X_VELOCITIES: [i8; 8] = [-8, -6, 0, 6, 8, 6, 0, -6];

pub(super) const SPRITE_SPAWN_BIG_SPLASH_LOCAL_Y_VELOCITIES: [i8; 8] = [0, -6, -8, -6, 0, 6, 8, 6];

pub(super) const ZORA_KING_DRAW_X_OFFSETS_1: [i8; 8] = [-23, 23, 23, 23, -20, -15, 13, 18];

pub(super) const ZORA_KING_DRAW_Y_OFFSETS_1: [i8; 8] = [-8, -8, -8, -8, -7, 0, 0, -7];

pub(super) const ZORA_KING_DRAW_CHARS_1: [u8; 8] = [0xae, 0xae, 0xae, 0xae, 0xac, 0xac, 0xac, 0xac];

pub(super) const ZORA_KING_DRAW_FLAGS_1: [u8; 8] = [0, 0x40, 0x40, 0x40, 0, 0, 0x40, 0x40];

pub(super) const WALKING_ZORA_DRAW_CHARS: [u8; 4] = [0xce, 0xce, 0xa4, 0xee];

pub(super) const WALKING_ZORA_DRAW_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

pub(super) const WALKING_ZORA_DRAW_CHARS_2: [u8; 8] =
    [0xcc, 0xec, 0xcc, 0xec, 0xe8, 0xe8, 0xca, 0xca];

pub(super) const WALKING_ZORA_DRAW_FLAGS_2: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0x40, 0, 0x40];

pub(super) const AGAHNIM_DRAW_X_OFFSETS_0: [i8; 72] = [
    -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8,
    8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -6, 6,
    -6, 6, -8, 8, -8, 8, -6, 6, -6, 6, 0, 8, 0, 8, -8, 8, -8, 8,
];

pub(super) const AGAHNIM_DRAW_Y_OFFSETS_0: [i8; 72] = [
    -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8,
    8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -6, -6,
    6, 6, -8, -8, 8, 8, -6, -6, 6, 6, 0, 0, 8, 8, 8, 8, 8, 8,
];

pub(super) const AGAHNIM_DRAW_CHARS_0: [u8; 72] = [
    0x82, 0x82, 0xa2, 0xa2, 0x80, 0x80, 0xa0, 0xa0, 0x84, 0x84, 0xa4, 0xa4, 0x86, 0x86, 0xa6, 0xa6,
    0x88, 0x8a, 0xa8, 0xaa, 0x8c, 0x8e, 0xac, 0xae, 0xc4, 0xc2, 0xe4, 0xe6, 0xc0, 0xc2, 0xe0, 0xe2,
    0x8a, 0x88, 0xaa, 0xa8, 0x8e, 0x8c, 0xae, 0xac, 0xc2, 0xc4, 0xe6, 0xe4, 0xc2, 0xc0, 0xe2, 0xe0,
    0xec, 0xec, 0xec, 0xec, 0xec, 0xec, 0xec, 0xec, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee,
    0xdf, 0xdf, 0xdf, 0xdf, 0x40, 0x42, 0x40, 0x42,
];

pub(super) const AGAHNIM_DRAW_FLAGS_0: [u8; 72] = [
    0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40,
    0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0, 0, 0,
];

pub(super) const AGAHNIM_DRAW_X_OFFSETS_1: [i8; 72] = [
    -7, 15, -11, 11, -11, 11, -8, 8, -4, 4, 0, 0, -10, -1, -14, -5, -14, -5, -12, -7, -10, -7, -10,
    -10, 16, 8, 12, 4, 12, 4, 10, 6, 9, 7, 8, 8, -6, -6, -10, -10, -10, -10, -10, -10, -10, -10,
    -10, -10, 14, 14, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, -7, 15, -11, 11, -11, 11, -8, 8, -4,
    4, 0, 0,
];

pub(super) const AGAHNIM_DRAW_Y_OFFSETS_1: [i8; 72] = [
    -5, -5, -9, -9, -9, -9, -9, -9, -9, -9, -9, -9, -3, 9, -7, 5, -7, 5, -5, 3, -3, 3, -2, -2, -3,
    9, -7, 5, -7, 5, -5, 3, -3, 3, -2, -2, -3, 9, -7, 5, -7, 5, -5, 3, -3, 3, -2, -2, -3, 9, -7, 5,
    -7, 5, -5, 3, -3, 3, -2, -2, -5, -5, -9, -9, -9, -9, -9, -9, -9, -9, -9, -9,
];

pub(super) const AGAHNIM_DRAW_CHARS_1: [u8; 36] = [
    0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6,
    0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc,
    0xc6, 0xc6, 0xc6, 0xc6,
];

pub(super) const AGAHNIM_DRAW_BIG_1: [u8; 36] = [
    0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2,
    2, 2, 2, 2,
];

pub(super) const SPRITE_7_A_AGAHNIM_START_STATE: [u8; 2] = [1, 6];

pub(super) const SPRITE_7_A_AGAHNIM_GRAPHICS_0: [u8; 5] = [12, 13, 14, 15, 16];

pub(super) const SPRITE_7_A_AGAHNIM_DIRECTIONS: [u8; 25] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 0, 1, 1, 4, 4, 0, 2, 2, 4, 4, 3, 2, 2,
];

pub(super) const SPRITE_7_A_AGAHNIM_GRAPHICS_1: [u8; 6] = [2, 10, 8, 0, 4, 6];

pub(super) const SPRITE_7_A_AGAHNIM_SECONDARY_GRAPHICS: [u8; 5] = [16, 15, 14, 13, 12];

pub(super) const SPRITE_7_A_AGAHNIM_GRAPHICS_3: [u8; 7] = [0, 8, 10, 2, 2, 6, 4];

pub(super) const KING_HELMASAUR_OPERATE_TAIL_MULTIPLIERS: [u8; 32] = [
    0xff, 0xf0, 0xe0, 0xd0, 0xc0, 0xb0, 0xa0, 0x90, 0x80, 0x70, 0x60, 0x50, 0x40, 0x30, 0x20, 0x10,
    0xff, 0xf8, 0xf0, 0xe8, 0xe0, 0xd8, 0xd0, 0xc8, 0xbc, 0xb0, 0xa0, 0x90, 0x70, 0x40, 0x20, 0x10,
];

pub(super) const KING_HELMASAUR_OPERATE_TAIL_MULT_B: [u8; 16] = [
    0xff, 0xf0, 0xe0, 0xd0, 0xc0, 0xb0, 0xa0, 0x90, 0x80, 0x70, 0x60, 0x50, 0x40, 0x30, 0x20, 0x10,
];

pub(super) const SPRITE_SPAWN_PROBE_ALWAYS_LOCAL_X_VELOCITIES: [i8; 64] = [
    -16, -16, -16, -16, -16, -16, -16, -16, -16, -14, -12, -10, -8, -6, -4, -2, 0, 2, 4, 6, 8, 10,
    12, 14, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 14, 12, 10, 8, 6, 4, 2,
    0, -2, -4, -6, -8, -10, -12, -14, -16, -16, -16, -16, -16, -16, -16, -16, -16,
];

pub(super) const SPRITE_SPAWN_PROBE_ALWAYS_LOCAL_Y_VELOCITIES: [i8; 64] = [
    0, 2, 4, 6, 8, 10, 12, 14, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 14,
    12, 10, 8, 6, 4, 2, 0, -2, -4, -6, -8, -10, -12, -14, -16, -16, -16, -16, -16, -16, -16, -16,
    -16, -16, -16, -16, -16, -16, -16, -16, -16, -14, -12, -10, -8, -6, -4, -2, 0,
];

pub(super) const SPRITE_70_KING_HELMASAUR_FIREBALL_CHARS: [u8; 3] = [0xcc, 0xcc, 0xca];

pub(super) const SPRITE_70_KING_HELMASAUR_FIREBALL_FLAGS: [u8; 2] = [0x33, 0x73];

pub(super) const SPRITE_70_KING_HELMASAUR_FIREBALL_LOCAL_GRAPHICS: [u8; 4] = [2, 2, 1, 0];

pub(super) const SPRITE_BEAMOS_LASER_HIT_X_OFFSETS: [i8; 4] = [-4, 4, -4, 4];

pub(super) const SPRITE_BEAMOS_LASER_HIT_Y_OFFSETS: [i8; 4] = [-4, -4, 4, 4];

pub(super) const SPRITE_BEAMOS_LASER_HIT_FLAGS: [u8; 4] = [6, 0x46, 0x86, 0xc6];

pub(super) const BOMBOS_TABLET_MESSAGES: [u16; 2] = [0x10d, 0x10f];

pub(super) const ETHER_TABLET_MESSAGES: [u16; 2] = [0x10d, 0x10e];

pub(super) const PULL_SWITCH_FACING_DOWN_YOFFS: [u8; 12] =
    [9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14];

pub(super) const BAD_PULL_DOWN_SWITCH_DRAW_BAD_PULL_SWITCH_Y_OFFSET_INDEX_BY_GRAPHICS: [u8; 12] =
    [0, 0, 1, 1, 2, 2, 3, 3, 4, 5, 5, 5];

pub(super) const BAD_PULL_UP_SWITCH_DRAW_BAD_PULL_SWITCH_Y_OFFSET_INDEX_BY_GRAPHICS: [u8; 12] =
    [0, 0, 1, 1, 2, 2, 3, 3, 4, 5, 5, 5];

pub(super) const SPRITE_93_BUMPER_VELOCITIES: [i8; 4] = [0, 2, -2, 0];

pub(super) const SPRITE_BAT_CRASH_X_POSITIONS: [u16; 4] = [0x07dc, 0x07f0, 0x0820, 0x0818];

pub(super) const SPRITE_BAT_CRASH_Y_POSITIONS: [u16; 4] = [0x062e, 0x0636, 0x0630, 0x05e0];

pub(super) const SPRITE_BAT_CRASH_DELAY: [u8; 5] = [4, 3, 4, 6, 0];

pub(super) const BAT_CRASH_SPAWN_DEBRIS_X_OFFSETS: [i8; 30] = [
    -8, 0, 8, 16, 24, 32, -8, 0, 8, 16, 24, 32, -8, 0, 8, 16, 24, 32, -8, 0, 8, 16, 24, 32, -8, 0,
    8, 16, 24, 32,
];

pub(super) const BAT_CRASH_SPAWN_DEBRIS_Y_OFFSETS: [i8; 30] = [
    0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x20, 0x20, 0x20, 0x20,
    0x20, 0x20, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
];

pub(super) const BAT_CRASH_SPAWN_DEBRIS_LOCAL_X_VELOCITIES: [i8; 30] = [
    -30, -25, -8, 8, 25, 30, -50, -45, -20, 20, 45, 50, -50, -35, -25, 25, 35, 50, -45, -50, -60,
    60, 50, 45, -30, -35, -40, 40, 35, 30,
];

pub(super) const BAT_CRASH_SPAWN_DEBRIS_LOCAL_Y_VELOCITIES: [i8; 30] = [
    2, 5, 10, 10, 5, 2, 5, 20, 30, 30, 20, 5, 10, 30, 40, 40, 30, 10, -20, -40, -60, -60, -40, -20,
    -10, -20, -40, -40, -20, -10,
];

pub(super) const SPRITE_D4_LANDMINE_OAM_FLAGS: [u8; 4] = [4, 2, 8, 2];

pub(super) const SPRITE_CF_SWAMOLA_TARGET_DIR: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

pub(super) const SWAMOLA_DRAW_LOCAL_GRAPHICS: [u8; 16] =
    [7, 6, 5, 4, 3, 4, 5, 6, 7, 6, 5, 4, 3, 4, 5, 6];

pub(super) const SWAMOLA_DRAW_SECONDARY_GRAPHICS: [u8; 4] = [0, 0, 1, 2];

pub(super) const SWAMOLA_DRAW_FLAGS: [u8; 16] = [
    0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40,
];

pub(super) const SWAMOLA_DRAW_HIST_OFFS: [u8; 4] = [8, 16, 22, 26];

pub(super) const STALFOS_DRAW_CHARS: [u8; 4] = [2, 2, 0, 4];

pub(super) const STALFOS_DRAW_FLAGS: [u8; 4] = [0x70, 0x30, 0x30, 0x30];

pub(super) const TRIDENT_DRAW_X_OFFSETS: [i8; 5] = [24, -16, 0, 16, -8];

pub(super) const TRIDENT_DRAW_Y_OFFSETS: [i8; 5] = [4, 4, 16, 21, 19];

pub(super) const TUTORIAL_SOLDIER_DRAW_X_OFFSETS: [i16; 20] = [
    4, 0, -6, -6, 2, 0, 0, -7, -7, -7, 0, 0, 0x0f, 0x0f, 0x0f, 6, 0x0e, -4, 4, 0,
];

pub(super) const TUTORIAL_SOLDIER_DRAW_Y_OFFSETS: [i16; 20] = [
    0, -10, -4, 12, 12, 0, -9, -11, -3, 5, 0, -9, -11, -3, 5, -11, 5, 0, 0, -9,
];

pub(super) const TUTORIAL_SOLDIER_DRAW_CHARS: [u8; 20] = [
    0x46, 0x40, 0, 0x28, 0x29, 0x4e, 0x42, 0x39, 0x2a, 0x3a, 0x4e, 0x42, 0x39, 0x2a, 0x3a, 0x26,
    0x38, 0x64, 0x64, 0x44,
];

pub(super) const TUTORIAL_SOLDIER_DRAW_FLAGS: [u8; 20] = [
    0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0, 0x40, 0,
];

pub(super) const TUTORIAL_SOLDIER_DRAW_BIG: [u8; 20] =
    [2, 2, 2, 0, 0, 2, 2, 0, 0, 0, 2, 2, 0, 0, 0, 2, 0, 2, 2, 2];

pub(super) const SPRITE_D3_STAL_STAL_GFX: [u8; 5] = [2, 2, 1, 0, 1];

pub(super) const SPRITE_D0_LYNEL_ATTACK_GFX: [u8; 4] = [5, 2, 8, 10];

pub(super) const SPRITE_D0_LYNEL_LOCAL_GRAPHICS: [u8; 8] = [3, 0, 6, 9, 4, 1, 7, 10];

pub(super) const BUZZ_BLOB_DRAW_X_OFFSETS: [u16; 3] = [0, 8, 0];

pub(super) const BUZZ_BLOB_DRAW_Y_OFFSETS: [i16; 3] = [-8, -8, 0];

pub(super) const BUZZ_BLOB_DRAW_CHARS: [u8; 18] = [
    0xf0, 0xf0, 0xe1, 0, 0, 0xce, 0, 0, 0xce, 0xe3, 0xe3, 0xca, 0xe4, 0xe5, 0xcc, 0xe5, 0xe4, 0xcc,
];

pub(super) const BUZZ_BLOB_DRAW_FL: [u8; 18] = [
    0, 0x40, 0, 0, 0, 0, 0, 0, 0x40, 0, 0x40, 0, 0, 0, 0, 0x40, 0x40, 0x40,
];

pub(super) const BUZZ_BLOB_DRAW_EXT: [u8; 3] = [0, 0, 2];

pub(super) const SPRITE_OLD_SNITCH_LADY_XD: [i8; 2] = [-32, 32];

pub(super) const SPRITE_OLD_SNITCH_LADY_LOCAL_X_VELOCITIES: [i8; 4] = [0, 0, -9, 9];

pub(super) const SPRITE_OLD_SNITCH_LADY_LOCAL_Y_VELOCITIES: [i8; 4] = [-9, 9, 0, 0];

pub(super) const GORIYA_DRAW_OFFSETS: [usize; 11] = [0, 4, 8, 12, 16, 20, 24, 26, 28, 30, 32];

pub(super) const KYAMERON_DRAW_FLAGS: [u8; 12] = [0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0xc0, 0x80];

pub(super) const PIROGUSU_DRAW_FLAGS: [u8; 28] = [
    0, 0x80, 0x40, 0, 0, 0, 0, 0x80, 0x80, 0xc0, 0x40, 0x40, 0, 0x40, 0x80, 0xc0, 0x40, 0xc0, 0,
    0x80, 0, 0x40, 0x80, 0xc0, 0x40, 0xc0, 0, 0x80,
];

pub(super) const PIROGUSU_DRAW_LOCAL_GRAPHICS: [u8; 28] = [
    0, 0, 1, 1, 2, 3, 4, 3, 2, 3, 4, 3, 5, 5, 5, 5, 7, 7, 7, 7, 6, 6, 6, 6, 8, 8, 8, 8,
];

pub(super) const SPRITE_HOBO_SMOKE_OAM_FLAGS: [u8; 4] = [0, 64, 128, 192];

pub(super) const SPRITE_RUNNING_MAN_LOCAL_X_VELOCITIES: [i8; 4] = [0, 0, -54, 54];

pub(super) const SPRITE_RUNNING_MAN_LOCAL_Y_VELOCITIES: [i8; 4] = [-54, 54, 0, 0];

pub(super) const SPRITE_RUNNING_MAN_DIRECTIONS: [i8; 4] = [3, 1, 3, -1];

pub(super) const SPRITE_RUNNING_MAN_A: [u8; 4] = [120, 24, 128, 3];

pub(super) const SPRITE_RUNNING_MAN_PLAYER_STATE_RECOIL_WALL_LOCAL: u8 = 13;

pub(super) const OLD_MOUNTAIN_MAN_DRAW_DMA: [u8; 16] = [
    0x20, 0xc0, 0x20, 0xc0, 0, 0xa0, 0, 0xa0, 0x40, 0x80, 0x40, 0x60, 0x40, 0x80, 0x40, 0x60,
];

pub(super) const WITCH_DRAW_DATA_A: [(i8, i8, u8, u8); 16] = [
    (-3, 8, 0xae, 0x00),
    (-3, 16, 0xbe, 0x00),
    (-2, 8, 0xae, 0x00),
    (-2, 16, 0xbe, 0x00),
    (-1, 8, 0xaf, 0x00),
    (-1, 16, 0xbf, 0x00),
    (0, 9, 0xaf, 0x00),
    (0, 17, 0xbf, 0x00),
    (1, 10, 0xaf, 0x00),
    (1, 18, 0xbf, 0x00),
    (0, 11, 0xaf, 0x00),
    (0, 18, 0xbf, 0x00),
    (-1, 10, 0xae, 0x00),
    (-1, 18, 0xbe, 0x00),
    (-3, 9, 0xae, 0x00),
    (-3, 17, 0xbe, 0x00),
];

pub(super) const WITCH_DRAW_DATA_B: [(i8, i8, u8, u8); 3] = [
    (0, -4, 0x80, 0x00),
    (-11, 15, 0x86, 0x04),
    (-3, 15, 0x86, 0x44),
];

pub(super) const WITCH_DRAW_DATA_C: [(i8, i8, u8, u8); 2] =
    [(0, 4, 0x84, 0x00), (0, 4, 0x82, 0x00)];

pub(super) const FORTUNE_TELLER_LIGHT_OR_DARK_WORLD_PRICES: [u8; 4] = [10, 15, 20, 30];

pub(super) const FORTUNE_TELLER_PERFORM_PSEUDO_SCIENCE_READINGS: [u8; 16] = [
    0xea, 0xeb, 0xec, 0xed, 0xee, 0xef, 0xf0, 0xf1, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
];

pub(super) const TALKING_TREE_MOUTH_SECONDARY_GRAPHICS: [u8; 4] = [0, 2, 3, 1];
/// ROM $9D:F9B0: `kTalkingTree_Gfx2` followed by the state-2 code bytes. The
/// ROM computes Y = delay>>1 but then loads the table with X, the sprite
/// slot (`LDA $F9B0,X`), so slots 4..15 read the instruction stream that
/// follows the four-entry table (route host 440910: slot 12 reads $9D).
pub(super) const TALKING_TREE_MOUTH_SECONDARY_GRAPHICS_BY_SLOT: [u8; 16] = [
    0x00, 0x02, 0x03, 0x01, 0xbd, 0xf0, 0x0d, 0x4a, 0xa8, 0xbd, 0xb0, 0xf9, 0x9d, 0xc0, 0x0d, 0xbd,
];

pub(super) const TALKING_TREE_MOUTH_MSGS_2: [u16; 2] = [0x0082, 0x007d];

pub(super) const TALKING_TREE_MOUTH_MSGS: [u16; 4] = [0x007e, 0x007f, 0x0080, 0x0081];

pub(super) const TALKING_TREE_MOUTH_SCREENS: [u8; 4] = [0x58, 0x5d, 0x72, 0x6b];

pub(super) const TALKING_TREE_MOUTH_LOCAL_GRAPHICS: [u8; 8] = [1, 2, 3, 1, 3, 1, 2, 3];

pub(super) const TALKING_TREE_MOUTH_DELAY: [u8; 8] = [13, 13, 13, 11, 11, 6, 16, 8];

pub(super) const TALKING_TREE_EYE_X_OFFSETS_1: [i8; 5] = [-2, -1, 0, 1, 2];

pub(super) const TALKING_TREE_EYE_Y_OFFSETS_1: [i8; 5] = [-1, 0, 0, 0, -1];

pub(super) const BOT_DRAW_LOCAL_GRAPHICS: [u8; 4] = [0, 1, 0, 1];

pub(super) const BOT_DRAW_FLAGS: [u8; 4] = [0, 0, 0x40, 0x40];

pub(super) const SPRITE_BOMB_SHOP_CLERK_LOCAL_GRAPHICS: [u8; 8] = [0, 1, 0, 1, 0, 1, 0, 1];

pub(super) const SPRITE_BOMB_SHOP_CLERK_DELAY: [u8; 8] = [255, 32, 255, 24, 15, 24, 255, 15];

pub(super) const SPRITE_BOMB_SHOP_HUFF_OAM_FLAGS: [u8; 4] = [4, 0x44, 0xc4, 0x84];

pub(super) const SPRITE_HOARDER_COVERED_LOCAL_GRAPHICS: [u8; 4] = [3, 4, 5, 4];

pub(super) const SPRITE_HOARDER_COVERED_LOCAL_X_VELOCITIES: [i8; 4] = [-12, 12, 0, 0];

pub(super) const SPRITE_HOARDER_COVERED_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, -12, 12];

pub(super) const SPRITE_HOARDER_FRANTIC_LOCAL_GRAPHICS: [u8; 4] = [0, 1, 0, 1];

pub(super) const SPRITE_HOARDER_FRANTIC_OAM_FLAGS: [u8; 4] = [0, 0, 0x40, 0];

pub(super) const SPRITE_HOARDER_FRANTIC_LOCAL_X_VELOCITIES: [i8; 4] = [-16, 16, -16, 16];

pub(super) const SPRITE_HOARDER_FRANTIC_LOCAL_Y_VELOCITIES: [i8; 4] = [-16, -16, 16, 16];

pub(super) const COVERED_RUPEE_CRAB_DRAW_Y_OFFSETS: [i8; 12] =
    [0, 0, 0, -3, 0, -5, 0, -6, 0, -6, 0, -6];

pub(super) const COVERED_RUPEE_CRAB_DRAW_CHARS: [u8; 12] = [
    0x44, 0x44, 0xe8, 0x44, 0xe8, 0x44, 0xe6, 0x44, 0xe8, 0x44, 0xe6, 0x44,
];

pub(super) const COVERED_RUPEE_CRAB_DRAW_FL: [u8; 12] =
    [0, 0x0c, 3, 0x0c, 3, 0x0c, 3, 0x0c, 3, 0x0c, 0x43, 0x0c];

pub(super) const GERUDO_MAN_DRAW_X_OFFSETS: [i8; 18] =
    [4, 4, 4, 4, 4, 4, -8, 8, 8, -8, 8, 8, -16, 0, 16, -16, 0, 16];

pub(super) const GERUDO_MAN_DRAW_Y_OFFSETS: [i8; 18] =
    [8, 8, 8, 8, 8, 8, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub(super) const GERUDO_MAN_DRAW_CHARS: [u8; 18] = [
    0xb8, 0xb8, 0xb8, 0xb8, 0xb8, 0xb8, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa4, 0xa2, 0xa0, 0xa0,
    0xa2, 0xa4,
];

pub(super) const GERUDO_MAN_DRAW_FL: [u8; 18] = [
    0, 0, 0, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0,
];

pub(super) const GERUDO_MAN_DRAW_BIG: [u8; 18] =
    [0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];

pub(super) const RECRUIT_DRAW_SOLDIER_CH: [u8; 4] = [0x42, 0x42, 0x40, 0x44];

pub(super) const RECRUIT_DRAW_SOLDIER_FL: [u8; 4] = [0x40, 0, 0, 0];

pub(super) const RECRUIT_DRAW_X_OFFSETS: [i16; 8] = [2, 2, -2, -2, 0, 0, 0, 0];

pub(super) const RECRUIT_DRAW_CHARS: [u8; 8] = [0x8a, 0x8c, 0x8a, 0x8c, 0x86, 0x88, 0x8e, 0xa0];

pub(super) const RECRUIT_DRAW_FL: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0, 0, 0];

pub(super) const SPRITE_LUMBERJACKS_MESSAGES: [u16; 4] = [0x012c, 0x012d, 0x012e, 0x012d];

pub(super) const FAERIE_CLOUD_DRAW_XY: [u16; 8] = [0xfff4, 0xfffa, 0, 6, 12, 18, 0, 6];

pub(super) const PSYCHO_TROOPER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

pub(super) const JAVELIN_TROOPER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

pub(super) const BUSH_JAVELIN_SOLDIER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

pub(super) const ARCHER_SOLDIER_DRAW_WEAPON_OAM_OFFS: [u8; 4] = [0, 0, 0, 16];

pub(super) const ARCHER_SOLDIER_DRAW_HEAD_OAM_OFFS: [u8; 4] = [16, 16, 16, 0];

pub(super) const ARCHER_SOLDIER_DRAW_BODY_OAM_OFFS: [u8; 4] = [20, 20, 20, 4];

pub(super) const ARCHER_SOLDIER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

pub(super) const SPRITE_48_RED_JAVELIN_GUARD_LOCAL_GRAPHICS: [u8; 4] = [12, 0, 18, 8];

pub(super) const SPRITE_48_RED_JAVELIN_GUARD_DIR_LOCK: [u8; 4] = [3, 2, 0, 1];

pub(super) const SPRITE_46_BLUE_ARCHER_LOCAL_GRAPHICS: [u8; 4] = [8, 0, 12, 5];

pub(super) const SPRITE_46_BLUE_ARCHER_DIR_LOCK: [u8; 4] = [3, 2, 0, 1];

pub(super) const CHAIN_BALL_TROOPER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

pub(super) const SPRITE_63_DEBIRANDO_PIT_OPENING_GFX: [u8; 4] = [5, 4, 3, 3];

pub(super) const SPRITE_63_DEBIRANDO_PIT_CLOSING_GFX: [u8; 4] = [3, 3, 4, 5];

pub(super) const SPRITE_64_DEBIRANDO_EMERGE_GFX: [u8; 2] = [1, 0];

pub(super) const SPRITE_64_DEBIRANDO_SUBMERGE_GFX: [u8; 2] = [0, 1];

pub(super) const DEBIRANDO_PIT_DRAW_X_OFFSETS: [i16; 24] = [
    -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, 0, 8, 0, 8, 0, 8, 0, 8, -8, 8, -8, 8,
];

pub(super) const DEBIRANDO_PIT_DRAW_Y_OFFSETS: [i16; 24] = [
    -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, -8, -8, 8, 8,
];

pub(super) const DEBIRANDO_PIT_DRAW_CHARS: [u8; 24] = [
    4, 4, 4, 4, 0x22, 0x22, 0x22, 0x22, 2, 2, 2, 2, 0x29, 0x29, 0x29, 0x29, 0x39, 0x39, 0x39, 0x39,
    0x2a, 0x2a, 0x2a, 0x2a,
];

pub(super) const DEBIRANDO_PIT_DRAW_FL: [u8; 24] = [
    0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40,
    0x80, 0xc0, 0, 0x40, 0x80, 0xc0,
];

pub(super) const DEBIRANDO_PIT_DRAW_BIG: [u8; 6] = [2, 2, 2, 0, 0, 2];

pub(super) const DEBIRANDO_DRAW_X_OFFSETS: [i8; 16] =
    [0, 8, 0, 8, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0];

pub(super) const DEBIRANDO_DRAW_Y_OFFSETS: [i8; 16] =
    [2, 2, 6, 6, -2, -2, 6, 6, -4, -4, -4, -4, -4, -4, -4, -4];

pub(super) const DEBIRANDO_DRAW_CHARS: [u8; 16] = [
    0, 0, 0xd8, 0xd8, 0, 0, 0xd9, 0xd9, 0, 0, 0, 0, 0x20, 0x20, 0x20, 0x20,
];

pub(super) const DEBIRANDO_DRAW_FL: [u8; 16] =
    [1, 0x41, 0, 0x40, 1, 1, 0, 0x40, 1, 1, 1, 1, 1, 1, 1, 1];

pub(super) const DEBIRANDO_DRAW_BIG: [u8; 16] = [0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2];

pub(super) const SPRITE_57_DESERT_STATUE_NEXT_D: [u8; 4] = [3, 2, 0, 1];

pub(super) const SPRITE_57_DESERT_STATUE_XV: [i8; 4] = [16, -16, 0, 0];

pub(super) const SPRITE_57_DESERT_STATUE_YV: [i8; 4] = [0, 0, 16, -16];

pub(super) const DASH_TREE_TOP_DRAW_X_OFFSETS: [i8; 16] =
    [10, 22, 30, 1, 34, 5, 13, 29, 0, 17, 27, 44, 15, 33, 18, 26];

pub(super) const DASH_TREE_TOP_DRAW_Y_OFFSETS: [i8; 16] =
    [0, 4, 2, 7, 10, 16, 24, 23, 34, 35, 30, 31, 46, 42, 10, 11];

pub(super) const DASH_TREE_TOP_DRAW_CHARS: [u8; 6] = [8, 8, 0x28, 0x28, 0x2a, 0x2a];

pub(super) const DASH_TREE_TOP_DRAW_FL: [u8; 6] = [0x31, 0x71, 0x31, 0x71, 0x31, 0x71];

pub(super) const RETREAT_BAT_DRAW_OFFSETS: [usize; 20] = [
    0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 6, 6, 8, 10, 12, 10, 14, 14, 14, 14,
];

pub(super) const RETREAT_BAT_DRAW_COUNT: [usize; 20] =
    [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 4, 4];

pub(super) const FAERIE_QUEEN_DRAW_X_OFFSETS: [u8; 24] = [
    0, 16, 0, 8, 16, 24, 0, 8, 16, 24, 0, 16, 0, 16, 0, 8, 16, 24, 0, 8, 16, 24, 0, 16,
];

pub(super) const FAERIE_QUEEN_DRAW_Y_OFFSETS: [u8; 24] = [
    0, 0, 16, 16, 16, 16, 24, 24, 24, 24, 32, 32, 0, 0, 16, 16, 16, 16, 24, 24, 24, 24, 32, 32,
];

pub(super) const FAERIE_QUEEN_DRAW_CHARS: [u8; 24] = [
    0xc7, 0xc7, 0xcf, 0xca, 0xca, 0xcf, 0xdf, 0xda, 0xda, 0xdf, 0xcb, 0xcb, 0xcd, 0xcd, 0xc9, 0xca,
    0xca, 0xc9, 0xd9, 0xda, 0xda, 0xd9, 0xcb, 0xcb,
];

pub(super) const FAERIE_QUEEN_DRAW_FL: [u8; 24] = [
    0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x40, 0x40, 0, 0x40, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x40,
    0x40, 0, 0x40,
];

pub(super) const FAERIE_QUEEN_DRAW_BIG: [u8; 24] = [
    2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2,
];

pub(super) const CRYSTAL_MAIDEN_DRAW_DMA: [u8; 16] = [
    0x20, 0xc0, 0x20, 0xc0, 0, 0xa0, 0, 0xa0, 0x40, 0x80, 0x40, 0x60, 0x40, 0x80, 0x40, 0x60,
];

pub(super) const SPRITE_0_F_OCTOBALLOON_Z: [u8; 8] = [16, 17, 18, 19, 20, 19, 18, 17];

pub(super) const SPRITE_0_D_BUZZBLOB_LOCAL_GRAPHICS: [u8; 4] = [0, 1, 0, 2];

pub(super) const SPRITE_0_D_BUZZBLOB_OBJ_PRIO: [u8; 4] = [10, 2, 8, 2];

pub(super) const SPRITE_08_OCTOROK_NEXT_DIR: [u8; 4] = [2, 3, 1, 0];

pub(super) const SPRITE_08_OCTOROK_DIRECTIONS: [u8; 4] = [3, 2, 0, 1];

pub(super) const SPRITE_08_OCTOROK_LOCAL_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(super) const SPRITE_08_OCTOROK_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(super) const SPRITE_08_OCTOROK_OAM_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

pub(super) const SPRITE_02_STALFOS_HEAD_OAM_FLAGS: [u8; 4] = [0, 0, 0, 0x40];

pub(super) const SPRITE_02_STALFOS_HEAD_LOCAL_GRAPHICS: [u8; 4] = [0, 1, 2, 1];

pub(super) const SPRITE_0_E_SNAPDRAGON_DELAY: [u8; 4] = [0x20, 0x30, 0x40, 0x50];

pub(super) const SPRITE_0_E_SNAPDRAGON_LOCAL_GRAPHICS: [u8; 4] = [4, 0, 6, 2];

pub(super) const SPRITE_0_E_SNAPDRAGON_LOCAL_X_VELOCITIES: [i8; 8] =
    [8, -8, 8, -8, 16, -16, 16, -16];

pub(super) const SPRITE_0_E_SNAPDRAGON_LOCAL_Y_VELOCITIES: [i8; 8] =
    [8, 8, -8, -8, 16, 16, -16, -16];

pub(super) const SPRITE_18_MINI_MOLDORM_LOCAL_X_VELOCITIES: [i8; 16] = [
    24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9, 0, 9, 17, 22,
];

pub(super) const SPRITE_18_MINI_MOLDORM_LOCAL_Y_VELOCITIES: [i8; 16] = [
    0, 9, 17, 22, 24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9,
];

pub(super) const SPRITE_18_MINI_MOLDORM_NEXT_DIR: [u8; 16] =
    [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7];

pub(super) const SPRITE_11_HINOX_WALK_GFX: [u8; 4] = [6, 4, 0, 2];

pub(super) const SPRITE_11_HINOX_LOCAL_GRAPHICS: [u8; 8] = [11, 10, 8, 9, 7, 5, 1, 3];

pub(super) const SPRITE_13_MINI_HELMASAUR_LOCAL_GRAPHICS: [u8; 8] = [3, 4, 3, 4, 2, 2, 5, 5];

pub(super) const SPRITE_13_MINI_HELMASAUR_OAM_FLAGS: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0x40, 0x40, 0];

pub(super) const SPRITE_82_ANTIFAIRY_CIRCLE_VEL: [i8; 2] = [1, -1];

pub(super) const SPRITE_7_E_FIREBAR_CLOCKWISE_INCR: [i16; 4] = [-2, 2, -1, 1];

pub(super) const SPRITE_20_SLUGGULA_LOCAL_GRAPHICS: [u8; 8] = [0, 1, 0, 1, 2, 3, 4, 5];

pub(super) const SPRITE_20_SLUGGULA_OAM_FLAGS: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0, 0, 0];

pub(super) const SPRITE_20_SLUGGULA_XYVEL: [i8; 6] = [16, -16, 0, 0, 16, -16];

pub(super) const SPRITE_19_POE_ACCEL: [i8; 4] = [1, -1, 2, -2];

pub(super) const SPRITE_19_POE_OAM_FLAGS: [u8; 2] = [0x40, 0];

pub(super) const SPRITE_19_POE_LOCAL_Y_VELOCITIES: [i8; 2] = [8, -8];

pub(super) const SPRITE_1_F_SICK_KID_LOCAL_GRAPHICS: [i8; 8] = [0, 1, 0, 1, 0, 1, 2, -1];

pub(super) const SPRITE_1_F_SICK_KID_DELAY: [u8; 7] = [8, 12, 8, 12, 8, 96, 16];

pub(super) const SPRITE_6_D_RAT_LOCAL_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(super) const SPRITE_6_D_RAT_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(super) const SPRITE_BA_WHIRLPOOL_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(super) const SPRITE_5_D_ROLLER_VERTICAL_DOWN_FIRST_XYVEL: [i8; 6] = [-16, 16, 0, 0, -16, 16];

pub(super) const SPRITE_4_D_TOPPO_X_OFFS: [i8; 4] = [-32, 32, 0, 0];

pub(super) const SPRITE_4_D_TOPPO_Y_OFFS: [i8; 4] = [0, 0, -32, 32];

pub(super) const TOPPO_DRAW_X_OFFSETS: [i8; 15] = [0, 8, 8, 0, 8, 8, 0, 0, 8, 0, 0, 0, 0, 0, 0];

pub(super) const TOPPO_DRAW_Y_OFFSETS: [i8; 15] = [8, 8, 8, 8, 8, 8, 0, 8, 8, 0, 0, 0, 0, 0, 0];

pub(super) const TOPPO_DRAW_CHARS: [u8; 15] = [
    0xc8, 0xc8, 0xc8, 0xca, 0xca, 0xca, 0xc0, 0xc8, 0xc8, 0xc2, 0xc2, 0xc2, 0xc2, 0xc2, 0xc2,
];

pub(super) const TOPPO_DRAW_FLAGS: [u8; 15] = [
    0, 0x40, 0x40, 0, 0x40, 0x40, 0, 0, 0x40, 0, 0, 0, 0x40, 0x40, 0x40,
];

pub(super) const TOPPO_DRAW_BIG: [u8; 15] = [0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 2, 2, 2, 2, 2];

pub(super) const SPRITE_4_C_GELDMAN_EMERGE_GFX: [u8; 8] = [3, 2, 0, 0, 0, 0, 0, 0];

pub(super) const SPRITE_4_C_GELDMAN_PURSUE_GFX: [u8; 2] = [4, 5];

pub(super) const SPRITE_4_C_GELDMAN_SUBMERGE_GFX: [u8; 5] = [0, 1, 2, 3, 3];

pub(super) const SPRITE_66_WALL_CANNON_VERTICAL_LEFT_LOCAL_X_VELOCITIES: [i8; 4] = [0, 0, -16, 16];

pub(super) const SPRITE_66_WALL_CANNON_VERTICAL_LEFT_LOCAL_Y_VELOCITIES: [i8; 4] = [-16, 16, 0, 0];

pub(super) const SPRITE_66_WALL_CANNON_VERTICAL_LEFT_LOCAL_GRAPHICS: [u8; 4] = [0, 0, 2, 2];

pub(super) const SPRITE_66_WALL_CANNON_VERTICAL_LEFT_OAM_FLAGS: [u8; 4] = [0x40, 0, 0, 0x80];

pub(super) const SPRITE_5_B_SPARK_CLOCKWISE_OAM_FLAGS: [u8; 4] = [0, 0x40, 0x80, 0xc0];

pub(super) const SPRITE_5_B_SPARK_CLOCKWISE_DIRECTIONS: [u8; 8] = [1, 3, 2, 0, 7, 5, 6, 4];

pub(super) const SPRITE_58_CRAB_LOCAL_X_VELOCITIES: [i8; 4] = [28, -28, 0, 0];

pub(super) const SPRITE_58_CRAB_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 12, -12];

pub(super) const SPRITE_80_FIRESNAKE_OAM_FLAGS: [u8; 4] = [0, 0x40, 0x80, 0xc0];

pub(super) const SPRITE_80_FIRESNAKE_LOCAL_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];

pub(super) const SPRITE_80_FIRESNAKE_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

pub(super) const SPRITE_87_KODONGO_FIRE_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(super) const SPRITE_87_KODONGO_FIRE_LOCAL_GRAPHICS: [u8; 32] = [
    5, 4, 3, 1, 2, 0, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0,
];

pub(super) const SPRITE_7_C_GREEN_STALFOS_DIRECTIONS: [u8; 4] = [4, 6, 0, 2];

pub(super) const SPRITE_7_C_GREEN_STALFOS_OAM_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

pub(super) const SPRITE_7_C_GREEN_STALFOS_LOCAL_GRAPHICS: [u8; 4] = [0, 0, 1, 2];

pub(super) const SPRITE_71_LEEVER_EMERGE_GFX: [u8; 16] =
    [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 2, 1, 2, 1, 0, 0];

pub(super) const SPRITE_71_LEEVER_ATTACK_GFX: [u8; 4] = [9, 10, 11, 12];

pub(super) const SPRITE_71_LEEVER_ATTACK_SPD: [u8; 2] = [12, 8];

pub(super) const SPRITE_71_LEEVER_SUBMERGE_GFX: [u8; 16] =
    [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 2, 1, 2, 1, 0, 0];

pub(super) const SPRITE_12_MOBLIN_LOCAL_X_VELOCITIES: [i8; 4] = [16, -16, 0, 0];

pub(super) const SPRITE_12_MOBLIN_LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 16, -16];

pub(super) const SPRITE_12_MOBLIN_DELAY: [u8; 4] = [0x10, 0x20, 0x30, 0x40];

pub(super) const SPRITE_12_MOBLIN_SECONDARY_GRAPHICS: [u8; 8] = [11, 10, 8, 9, 7, 5, 0, 2];

pub(super) const SPRITE_12_MOBLIN_DIRS: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];

pub(super) const SPRITE_12_MOBLIN_LOCAL_GRAPHICS: [u8; 4] = [6, 4, 0, 2];

pub(super) const OCTOBALLOON_DRAW_X_OFFSETS: [i8; 12] = [-4, 4, -4, 4, -8, 8, -8, 8, -4, 4, -4, 4];

pub(super) const OCTOBALLOON_DRAW_Y_OFFSETS: [i8; 12] = [-4, -4, 4, 4, -8, -8, 8, 8, -4, -4, 4, 4];

pub(super) const OCTOBALLOON_DRAW_CHARS: [u8; 12] = [
    0x8c, 0x8c, 0x9c, 0x9c, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86,
];

pub(super) const OCTOBALLOON_DRAW_FL: [u8; 12] =
    [0, 0x40, 0, 0x40, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0];

pub(super) const KHOLDSTARE_DRAW_X_OFFSETS: [i8; 16] =
    [8, 7, 4, 2, 0, -2, -4, -7, -8, -7, -4, -2, 0, 2, 4, 7];

pub(super) const KHOLDSTARE_DRAW_Y_OFFSETS: [i8; 16] =
    [0, 2, 4, 7, 8, 7, 4, 2, 0, -2, -4, -7, -8, -7, -4, -2];

pub(super) const KHOLDSTARE_DRAW_CHARS: [u8; 16] = [
    0xac, 0xac, 0xaa, 0x8c, 0x8c, 0x8c, 0xaa, 0xac, 0xac, 0xaa, 0xaa, 0x8c, 0x8c, 0x8c, 0xaa, 0xac,
];

pub(super) const KHOLDSTARE_DRAW_FL: [u8; 16] = [
    0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xc0, 0xc0,
];

pub(super) const BAT_CRASH_DRAW_HARDCODED_GARBAGE_OAMS: [(i8, i8, u8, u8); 8] = [
    (104, -105, 0x57, 0x01),
    (120, -105, 0x57, 0x01),
    (-120, -105, 0x57, 0x01),
    (104, -89, 0x57, 0x01),
    (120, -89, 0x57, 0x01),
    (-120, -89, 0x57, 0x01),
    (101, -112, 0x57, 0x01),
    (-117, -112, 0x57, 0x01),
];

pub(super) const MOLDORM_DRAW_X_OFFSETS: [i8; 16] =
    [11, 10, 9, 6, 3, 0, -2, -3, -4, -3, -2, 1, 4, 7, 9, 10];

pub(super) const MOLDORM_DRAW_Y_OFFSETS: [i8; 16] =
    [4, 6, 9, 10, 11, 10, 9, 6, 3, 0, -2, -3, -4, -3, -2, 1];

pub(super) const MOLDORM_DRAW_CHARS: [u8; 3] = [0x5d, 0x62, 0x60];

pub(super) const MOLDORM_DRAW_XY: [i8; 3] = [4, 0, 0];

pub(super) const MOLDORM_DRAW_BIG: [u8; 3] = [0, 2, 2];

pub(super) const MOLDORM_DRAW_GET_OFFS: [u8; 3] = [21, 26, 0];

pub(super) const SPRITE_54_LANMOLAS_RAND_B: [u8; 8] =
    [0x58, 0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0x98];

pub(super) const SPRITE_54_LANMOLAS_RAND_C: [u8; 8] =
    [0x68, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xa8, 0x80];

pub(super) const SPRITE_54_LANMOLAS_ZVEL: [i8; 2] = [2, -2];

pub(super) const LANMOLA_DRAW_SPR_OFFS: [u8; 4] = [76, 60, 44, 28];

pub(super) const LANMOLA_DRAW_CHARS_1: [u8; 16] = [
    0xc4, 0xe2, 0xc2, 0xe0, 0xc0, 0xe0, 0xc2, 0xe2, 0xc4, 0xe2, 0xc2, 0xe0, 0xc0, 0xe0, 0xc2, 0xe2,
];

pub(super) const LANMOLA_DRAW_CHARS_0: [u8; 16] = [
    0xcc, 0xe4, 0xca, 0xe6, 0xc8, 0xe6, 0xca, 0xe4, 0xcc, 0xe4, 0xca, 0xe6, 0xc8, 0xe6, 0xca, 0xe4,
];

pub(super) const LANMOLA_DRAW_FLAGS: [u8; 16] = [
    0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40,
];

pub(super) const CHAIN_CHOMP_DRAW_LOCAL_GRAPHICS: [u8; 16] =
    [0, 1, 2, 3, 3, 3, 2, 1, 0, 0, 0, 4, 4, 4, 0, 0];

pub(super) const CHAIN_CHOMP_DRAW_OAM_FLAGS: [u8; 16] = [
    0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40,
];

pub(super) const MOBLIN_DRAW_OBJ_OFFS: [u8; 12] = [2, 2, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2];

pub(super) const MOBLIN_DRAW_HEAD_CHAR: [u8; 4] = [0x88, 0x88, 0x86, 0x84];

pub(super) const ALTAR_ZELDA_DRAW_BODY_X_OFFS: [u8; 16] =
    [4, 4, 3, 3, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];

pub(super) const ARCHERY_GAME_DRAW_PRIZE_X_OFFSETS: [i8; 5] = [-8, -8, 0, 8, 16];

pub(super) const ARCHERY_GAME_DRAW_PRIZE_Y_OFFSETS: [i8; 5] = [-24, -16, -20, -20, -20];

pub(super) const ARCHERY_GAME_DRAW_PRIZE_CHARS: [u8; 3] = [0x0b, 0x1b, 0xb6];

pub(super) const ARCHERY_GAME_DRAW_PRIZE_FLAGS: [u8; 5] = [0x38, 0x38, 0x34, 0x35, 0x35];

pub(super) const ARCHERY_GAME_DRAW_PRIZE_CHARS_3: [u8; 6] = [0x12, 0x32, 0x31, 3, 0x22, 0x33];

pub(super) const ARCHERY_GAME_DRAW_PRIZE_CHARS_4: [u8; 6] = [0x7c, 0x7c, 0x22, 2, 0x12, 0x33];

pub(super) const BUSH_SOLDIER_COMMON_DRAW_Y_OFFSETS: [i8; 14] =
    [8, 8, 8, 8, 2, 8, 0, 8, -3, 8, -3, 8, -3, 8];

pub(super) const BUSH_SOLDIER_COMMON_DRAW_CHARS: [u8; 14] = [
    0x20, 0x20, 0x20, 0x20, 0x40, 0x20, 0x40, 0x20, 0x40, 0x20, 0x42, 0x20, 0x42, 0x20,
];

pub(super) const BUSH_SOLDIER_COMMON_DRAW_FLAGS: [u8; 14] =
    [9, 3, 0x49, 0x43, 9, 3, 0x49, 0x43, 9, 3, 0x49, 0x43, 9, 3];

pub(super) const ARCHERY_GAME_GUY_DRAW_X_OFFSETS: [i8; 15] =
    [0, 0, 0, 0, 0, -5, 0, -1, -1, 0, 0, 0, 0, 1, 1];

pub(super) const ARCHERY_GAME_GUY_DRAW_Y_OFFSETS: [i8; 15] = [
    0, -10, -10, 0, -10, -3, 0, -10, -10, 0, -10, -10, 0, -10, -10,
];

pub(super) const ARCHERY_GAME_GUY_DRAW_CHARS: [u8; 15] =
    [0x26, 6, 6, 8, 6, 0x3a, 0x26, 6, 6, 0x26, 6, 6, 0x26, 6, 6];

pub(super) const ARCHERY_GAME_GUY_DRAW_FLAGS: [u8; 15] =
    [8, 6, 6, 8, 6, 8, 8, 6, 6, 8, 6, 6, 8, 6, 6];

pub(super) const ARCHERY_GAME_GUY_DRAW_BIG: [u8; 15] =
    [2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2];

pub(super) const PUSH_SWITCH_DRAW_OAM: [(i8, i8, u8, u8); 40] = [
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

pub(super) const PUSH_SWITCH_DRAW_WH: [u8; 16] = [
    8, 6, 0x10, 0x10, 0x10, 8, 0x10, 8, 0x10, 8, 0x10, 8, 0x10, 3, 0x10, 8,
];

pub(super) const TEKTITE_DRAW_FRAMES: [DrawMultipleData; 6] = [
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

pub(super) const SPRITE_52_KING_ZORA_SURFACING_GFX: [u8; 16] =
    [0, 0, 0, 3, 9, 8, 7, 6, 9, 8, 7, 6, 5, 4, 5, 4];

pub(super) const SPRITE_52_KING_ZORA_DIALOGUE_GFX: [u8; 8] = [0, 0, 1, 2, 1, 2, 0, 0];

pub(super) const SPRITE_52_KING_ZORA_SUBMERGE_GFX: [u8; 21] = [
    12, 12, 12, 12, 12, 12, 11, 11, 11, 11, 11, 10, 10, 10, 10, 3, 3, 3, 3, 3, 3,
];

pub(super) const ZORA_KING_DRAW_X_OFFSETS_0: [i8; 52] = [
    -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, 0, 0, 0, 0, 0, 0, 0, 0, -8, 8, -8, 8,
    -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -9, 9, -9, 9, -10, 10, -10, 10, -11, 11, -11, 11,
];

pub(super) const ZORA_KING_DRAW_Y_OFFSETS_0: [i8; 52] = [
    -18, -18, -2, -2, -18, -18, -2, -2, -18, -18, -2, -2, -12, -12, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0,
    -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -5, -5, 5, 5, -5, -5, 5, 5, -5, -5, 5,
    5,
];

pub(super) const ZORA_KING_DRAW_CHARS_0: [u8; 52] = [
    0xc0, 0xc0, 0xe0, 0xe0, 0xc2, 0xea, 0xe2, 0xe2, 0xea, 0xc2, 0xe2, 0xe2, 0xc0, 0xc0, 0xe4, 0xe6,
    0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0xc4, 0xc6, 0xe4, 0xe6, 0xc6, 0xc4, 0xe6, 0xe4,
    0xe6, 0xe4, 0xc6, 0xc4, 0xe4, 0xe6, 0xc4, 0xc6, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x88,
    0x88, 0x88, 0x88, 0x88,
];

pub(super) const ZORA_KING_DRAW_FLAGS_0: [u8; 52] = [
    0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 5, 5, 5, 5, 5, 5, 0xc5, 0xc5,
    0xc5, 0xc5, 5, 5, 5, 5, 0x45, 0x45, 0x45, 0x45, 0xc5, 0xc5, 0xc5, 0xc5, 0x85, 0x85, 0x85, 0x85,
    4, 0x44, 0x84, 0xc4, 4, 0x44, 0x84, 0xc4, 4, 0x44, 0x84, 0xc4,
];

pub(super) const SPRITE_CATFISH_QUAKE_MEDALLION_Z_VELOCITIES: [u8; 4] = [0x20, 0x10, 8, 0];

pub(super) const CATFISH_BIG_FISH_EMERGE_GFX: [u8; 16] =
    [1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 0, 0, 0, 0];

// $1D:E0BF through E0D3. Index 20 reaches the first instruction byte
// following the named table when the outdoor splash clears carry.
pub(super) const CATFISH_BIG_FISH_CONVERSATE_GFX: [u8; 21] = [
    0, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6, 6, 0xbd,
];

// GreatCatfish_Draw computes E240 + (BD - 1) * 32 = F9C0. Preserve
// the source's full 16-bit coordinates and eighth-byte extended attributes.
pub(super) const GREAT_CATFISH_SPLASH_CARRY_DRAW: [[u8; 8]; 4] = [
    [0xf0, 0x0d, 0xc9, 0x07, 0xd0, 0x03, 0x20, 0x4e],
    [0xfa, 0xbd, 0xf0, 0x0d, 0xd0, 0x03, 0xfe, 0x80],
    [0x0d, 0x60, 0x01, 0x02, 0x03, 0x01, 0x03, 0x01],
    [0x02, 0x03, 0x0d, 0x0d, 0x0d, 0x0b, 0x0b, 0x06],
];

pub(super) const SPRITE_23_RED_BARI_BARI_IDLE_X_VELOCITIES: [u8; 2] = [8, 0xf8];

pub(super) const SPRITE_CF_SWAMOLA_Z_ACCEL: [i8; 2] = [2, -2];

pub(super) const SPRITE_CF_SWAMOLA_Z_VEL_TARGET: [i8; 2] = [12, -12];

pub(super) const ZOL_DRAW_OAM_FLAGS: [u8; 4] = [0, 0, 0x40, 0x40];

pub(super) const ZOL_DRAW_FRAMES: [DrawMultipleData; 8] = [
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

pub(super) const SPRITE_D0_LYNEL_X_TARGETS: [i8; 4] = [-96, 96, 0, 0];

pub(super) const SPRITE_D0_LYNEL_Y_TARGETS: [i8; 4] = [8, 8, -96, 112];

pub(super) const SPRITE_HOBO_BUM_LOCAL_GRAPHICS: [u8; 7] = [0, 1, 0, 1, 0, 1, 2];

pub(super) const SPRITE_HOBO_BUM_DELAY: [u8; 7] = [6, 2, 6, 6, 2, 100, 30];

pub(super) const SPRITE_21_WATER_SWITCH_DELAY: [u8; 10] = [40, 6, 3, 3, 3, 5, 1, 1, 3, 12];

pub(super) const SPRITE_21_WATER_SWITCH_DIR: [u8; 10] = [0, 1, 2, 3, 4, 5, 5, 6, 7, 6];

pub(super) const LANMOLA_DRAW_BODY_FRAME_INDICES: [u8; 16] =
    [4, 5, 4, 5, 4, 5, 4, 5, 4, 3, 2, 2, 1, 1, 0, 0];

pub(super) const LANMOLA_DRAW_BODY_CHARS: [u8; 6] = [0xee, 0xee, 0xec, 0xec, 0xce, 0xce];

pub(super) const LANMOLA_DRAW_BODY_FLAGS: [u8; 6] = [0, 0x40, 0, 0x40, 0, 0x40];

pub(super) const LANMOLA_DRAW_SPLASH_X_OFFSETS: [i8; 8] = [-8, 8, -10, 10, -16, 16, -24, 32];

pub(super) const LANMOLA_DRAW_SPLASH_Y_OFFSETS: [i8; 8] = [0, 0, -1, -1, -1, -1, 3, 3];

pub(super) const LANMOLA_DRAW_SPLASH_CHARS: [u8; 8] =
    [0xe8, 0xe8, 0xe8, 0xe8, 0xea, 0xea, 0xea, 0xea];

pub(super) const LANMOLA_DRAW_SPLASH_FLAGS: [u8; 8] = [0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40];

pub(super) const LANMOLA_DRAW_SPLASH_SIZES: [u8; 8] = [2, 2, 2, 2, 2, 2, 0, 0];

pub(super) const BOMB_TROOPER_DRAW_ARM_X_OFFSETS: [i8; 8] = [-1, 1, 2, 0, 9, 9, -8, -8];

pub(super) const BOMB_TROOPER_DRAW_ARM_Y_OFFSETS: [i8; 8] =
    [-12, -12, -12, -12, -16, -14, -12, -14];
