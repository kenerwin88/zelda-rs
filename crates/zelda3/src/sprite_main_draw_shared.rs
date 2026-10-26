//! Shared sprite draw math, common constants, and family table re-exports.

use crate::types::sign8;

#[path = "sprite_main_draw_shared/bosses.rs"]
mod bosses;
pub(super) use bosses::*;

#[path = "sprite_main_draw_shared/objects_items_effects.rs"]
mod objects_items_effects;
pub(super) use objects_items_effects::*;

#[path = "sprite_main_draw_shared/dungeon_enemies_soldiers.rs"]
mod dungeon_enemies_soldiers;
pub(super) use dungeon_enemies_soldiers::*;

#[path = "sprite_main_draw_shared/overworld_creatures.rs"]
mod overworld_creatures;
pub(super) use overworld_creatures::*;

#[path = "sprite_main_draw_shared/npcs.rs"]
mod npcs;
pub(super) use npcs::*;

// variables.h:1203..1217 — garnish_* tables (paged at 0x1F800+).
// variables.h:690 — activate_bomb_trap_overlord.
// variables.h:1208 — sprite_I.
// variables.h:1241..1242 — chainchomp history buffer aliases the moldorm pages.
// variables.h:679..683 — ancilla_*.
// variables.h:179 — sound_effect_1.
// variables.h:535 — minigame_credits.
// variables.h:741 — flag_overworld_area_did_change.
// sprite_main.c local scratch words.
// variables.h:488 — enhanced_features0.
pub(super) const FEATURE_MISC_BUG_FIXES_DRAW: u32 = 4096;
// variables.h:1071 — link_item_bombos_medallion.
// ---------------------------------------------------------------------------
// kSinusLookupTable from sprite_main.c:338 — 256-entry sine half-wave used
// by ChainBallSin / HelmasaurSin. Verbatim from the C source.
// ---------------------------------------------------------------------------
pub(in crate::zelda_rtl) const SHARED_SINE_LOOKUP_TABLE: [u16; 256] = [
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
pub(super) const RECRUIT_GRAPHICS: [u8; 8] = [0, 2, 4, 6, 1, 3, 5, 7];
pub(super) const PLAYER_DAMAGE_CARRY_MASK_DRAW: u8 = 1;

// ---------------------------------------------------------------------------
// Free helpers (module-local) ported from sprite_main.c inline statics.
// ---------------------------------------------------------------------------

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
