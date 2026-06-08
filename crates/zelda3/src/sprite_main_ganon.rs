//! Ported Ganon-boss handlers from sprite_main.c.
//!
//! Each method preserves a 1:1 mapping to the C source (sprite_main.c lines
//! 14392..15075). The original C body is reproduced as a comment block above
//! each port so a reviewer can verify behavior line-by-line.
//!
//! Helpers reached from these handlers use `_for_ganon` suffixes where this
//! split module needs an explicit adapter to a shared canonical port or a
//! local table/OAM emitter.

use super::sprite::DrawMultipleData;
use super::*;
use crate::types::sign8;

// ---------------------------------------------------------------------------
// File-local RAM offsets. These mirror variables.h and are kept local because
// the global tables in zelda_rtl.rs are not exported (the other sprite_main_*
// files follow the same convention).
// ---------------------------------------------------------------------------

// variables.h:225, 727 — torch state owned by dungeon.c helpers.
const DUNG_TORCH_TIMERS_GANON: usize = 0x04f0;
const DUNG_TORCH_DATA_GANON: usize = 0x0fb40;
// variables.h:647..654 — overlord scratch.
const OVERLORD_X_LO_GANON: usize = 0x0b08;
const OVERLORD_X_HI_GANON: usize = 0x0b10;
const OVERLORD_Y_HI_GANON: usize = 0x0b20;
const OVERLORD_GEN2_GANON: usize = 0x0b30;
const OVERLORD_FLOOR_GANON: usize = 0x0b40;
// variables.h:666 — sprite_obj_prio.
const SPRITE_OBJ_PRIO_GANON: usize = 0x0b89;
// variables.h:752 — sprite_n.
// variables.h:670 — sprite_ignore_projectile.
const SPRITE_IGNORE_PROJECTILE_GANON: usize = 0x0ba0;
// variables.h sprite_bump_damage at g_ram+0xCD2.
const SPRITE_BUMP_DAMAGE_GANON: usize = 0x0cd2;
// sprite_B / sprite_C live in pages 0xD90..0xDB0.
const SPRITE_B_GANON: usize = 0x0da0;
const SPRITE_C_GANON: usize = 0x0db0;
// sprite_delay_aux1 .. aux4
const SPRITE_DELAY_AUX1_GANON: usize = 0x0e00;
const SPRITE_DELAY_AUX2_GANON: usize = 0x0e10;
// sprite_health
const SPRITE_HEALTH_GANON: usize = 0x0e50;
// sprite_anim_clock
const SPRITE_ANIM_CLOCK_GANON: usize = 0x0ec0;
// sprite_G
const SPRITE_G_GANON: usize = 0x0ed0;
// sprite_hit_timer
const SPRITE_HIT_TIMER_GANON: usize = 0x0ef0;
// tmp_counter
const TMP_COUNTER_GANON: usize = 0x0fb5;
// garnish_countdown
const GARNISH_COUNTDOWN_GANON: usize = 0x1f90e;
// swamola_target_x_lo / y_lo
const SWAMOLA_TARGET_X_LO_GANON: usize = 0x1fd5c;
const SWAMOLA_TARGET_Y_LO_GANON: usize = 0x1fd68;
// link_x_coord / link_y_coord (already in zelda_rtl.rs as LINK_X_COORD/LINK_Y_COORD)

// ---------------------------------------------------------------------------
// Static tables shared across the Ganon handlers (sprite_main.c:402..464).
// ---------------------------------------------------------------------------

const K_GANON_G_FUNC2: [u8; 16] = [8, 7, 6, 5, 4, 3, 2, 1, 8, 7, 6, 5, 4, 3, 2, 1];

// Sin-table used by GanonSin (sprite_main.c:338).
const K_SINUS_LOOKUP_TABLE_GANON: [u16; 256] = [
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

// PhantomGanon_Draw table (sprite_main.c:14393).
// Packed as (x:i8, y:i8, char:u16, big:u8).
const K_PHANTOM_GANON_DMD: [(i8, i8, u16, u8); 16] = [
    (-16, -8, 0x0d46, 2),
    (-8, -8, 0x0d47, 2),
    (8, -8, 0x4d47, 2),
    (16, -8, 0x4d46, 2),
    (-16, 8, 0x0d69, 2),
    (-8, 8, 0x0d6a, 2),
    (8, 8, 0x4d6a, 2),
    (16, 8, 0x4d69, 2),
    (-16, -8, 0x0d46, 2),
    (-8, -8, 0x0d47, 2),
    (8, -8, 0x4d47, 2),
    (16, -8, 0x4d46, 2),
    (-16, 8, 0x0d66, 2),
    (-8, 8, 0x0d67, 2),
    (8, 8, 0x4d67, 2),
    (16, 8, 0x4d66, 2),
];

// Ganon_HandleFireBatCircle (sprite_main.c:14559..14560).
const K_GANON_MATH_X: [i8; 16] = [
    0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16,
];
const K_GANON_MATH_Y: [i8; 16] = [
    32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16, 0, 16, 24, 28,
];

// Ganon_SpawnFallingTilesOverlord (sprite_main.c:14989..14991).
const K_GANON_OV_TYPE: [u8; 4] = [12, 13, 14, 15];
const K_GANON_OV_X: [u8; 4] = [0x18, 0xd8, 0xd8, 0x18];
const K_GANON_OV_Y: [u8; 4] = [0x28, 0x28, 0xd8, 0xd8];

// Ganon_Func1 (sprite_main.c:15023).
const K_GANON_GFX16_Y: [i8; 2] = [0, -16];

// Ganon_Phase1_AnimateTridentSpin (sprite_main.c:15036).
const K_GANON_GFX_FUNC2: [u8; 16] = [0, 0, 1, 1, 0, 0, 1, 1, 8, 8, 9, 9, 8, 8, 9, 9];

// Ganon_HandleAnimation_Idle (sprite_main.c:15045..15046).
const K_GANON_G_IDLE: [u8; 2] = [9, 10];
const K_GANON_GFX_IDLE: [u8; 2] = [2, 10];

// Ganon_SelectWarpLocation (sprite_main.c:15053..15058).
const K_GANON_NEXT_SUBTYPE: [u8; 32] = [
    4, 5, 6, 7, 4, 5, 6, 7, 4, 5, 6, 7, 4, 5, 6, 7, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3,
];
const K_GANON_NEXT_Y: [u8; 8] = [0x40, 0x30, 0x30, 0x40, 0xb0, 0xc0, 0xc0, 0xb0];
const K_GANON_NEXT_X: [u8; 8] = [0x30, 0x50, 0xa0, 0xc0, 0x40, 0x60, 0x90, 0xb0];

// Ganon_ShakeHead (sprite_main.c:15070..15073).
const K_GANON_HEAD_DIR: [u8; 18] = [0, 0, 0, 1, 2, 2, 2, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 16];

// Ganon_Draw (sprite_main.c:402..464, 15077..15130).
const K_GANON_DRAW_X: [i8; 204] = [
    18, -8, 8, -8, 8, -18, -18, 18, -8, 8, -8, 8, 18, -8, 8, -8, 8, -18, -18, 18, -8, 8, -8, 8, 16,
    -8, 8, -8, 8, -18, -18, 16, -8, 8, -11, 11, 16, -8, 8, -8, 8, -18, -18, 16, -8, 8, -11, 11, 16,
    -8, 8, -8, 8, -18, -18, 16, -8, 8, -11, 11, 18, -8, 8, -8, 8, -18, -18, 18, -8, 8, -8, 8, 18,
    -8, 8, -8, 8, -18, -18, 18, -8, 8, -8, 8, 18, -8, 8, -8, 8, -18, -18, 18, -8, 8, -11, 11, -8,
    8, -8, 8, -8, 8, -8, 8, -18, -18, 18, 18, -8, 8, -8, 8, -8, 8, -8, 8, -18, -18, 18, 18, -8, 8,
    -8, 8, -8, 8, -10, 10, -18, -18, 18, 18, -8, 8, -8, 8, -8, 8, -10, 10, -18, -18, 18, 18, -8, 8,
    -8, 8, -8, 8, -10, 10, -18, -18, 18, 18, -8, 8, -8, 8, -8, 8, -8, 8, -18, -18, 18, 18, -8, 8,
    -8, 8, -8, 8, -8, 8, -18, -18, 18, 18, -7, -8, 8, -8, 8, -9, 8, -14, -14, -8, 8, 8, -8, 8, -8,
    8, -18, -18, 18, 18, -8, 8, -11, 11,
];
const K_GANON_DRAW_Y: [i8; 204] = [
    -8, -16, -16, -13, -13, -9, -1, -16, 3, 3, 8, 8, -8, -16, -16, -13, -13, -9, -1, -16, 3, 3, 8,
    8, 5, -10, -10, -13, -13, -7, 1, -3, 3, 3, 8, 8, 5, -10, -10, -13, -13, -7, 1, -3, 3, 3, 8, 8,
    5, -10, -10, -13, -13, -7, 1, -3, 3, 3, 8, 8, -1, -16, -16, -13, -13, -9, -1, -9, 3, 3, 8, 8,
    -10, -16, -16, -13, -13, -18, -10, -18, 3, 3, 8, 8, 1, -10, -10, -13, -13, -7, 1, -7, 3, 3, 8,
    8, -12, -12, 4, 4, -18, -18, 10, 10, -16, -8, -4, 4, -12, -12, 4, 4, -18, -18, 10, 10, -16, -8,
    -4, 4, -12, -12, 4, 4, -12, -12, 10, 10, -4, 4, -4, 4, -12, -12, 4, 4, -12, -12, 10, 10, -4, 4,
    -4, 4, -12, -12, 4, 4, -12, -12, 10, 10, -4, 4, -4, 4, -12, -12, 4, 4, -18, -18, 10, 10, -4, 4,
    -4, 4, -12, -12, 4, 4, -18, -18, 10, 10, -16, -8, -16, -8, -7, -12, -12, 4, 4, 7, 13, -11, -4,
    -16, -16, -16, -10, -10, -13, -13, -7, -7, -7, -7, 3, 3, 8, 8,
];
const K_GANON_DRAW_CHAR: [u8; 204] = [
    0x16, 0, 0, 2, 2, 8, 0x18, 6, 0x22, 0x22, 0x20, 0x20, 0x46, 0, 0, 2, 2, 8, 0x18, 0x36, 0x22,
    0x22, 0x20, 0x20, 0x1a, 0, 0, 4, 4, 0x38, 0x48, 0x0a, 0x24, 0x24, 0x20, 0x20, 0x1a, 0x40, 0x42,
    4, 4, 0x38, 0x48, 0x0a, 0x24, 0x24, 0x20, 0x20, 0x1a, 0x42, 0x40, 4, 4, 0x38, 0x48, 0x0a, 0x24,
    0x24, 0x20, 0x20, 0x18, 0, 0, 2, 2, 8, 0x18, 8, 0x22, 0x22, 0x20, 0x20, 0x16, 0x6a, 0x6a, 0x0e,
    0x0e, 6, 0x16, 6, 0x22, 0x22, 0x20, 0x20, 0x48, 0, 0, 4, 4, 0x38, 0x48, 0x38, 0x24, 0x24, 0x20,
    0x20, 0x4e, 0x4e, 0x6e, 0x6e, 0x6c, 0x6c, 0xa2, 0xa2, 0x0c, 0x1c, 0x3c, 0x4c, 0x4e, 0x4e, 0x6e,
    0x6e, 0x6c, 0x6c, 0xa2, 0xa2, 0x3a, 0x4a, 0x3c, 0x4c, 0x84, 0x84, 0xa4, 0xa4, 0xa0, 0xa0, 0xa2,
    0xa2, 0x3c, 0x4c, 0x3c, 0x4c, 0x84, 0x84, 0xa4, 0xa4, 0x80, 0x82, 0xa2, 0xa2, 0x3c, 0x4c, 0x3c,
    0x4c, 0x84, 0x84, 0xa4, 0xa4, 0x82, 0x80, 0xa2, 0xa2, 0x3c, 0x4c, 0x3c, 0x4c, 0x4e, 0x4e, 0x6e,
    0x6e, 0x6c, 0x6c, 0xa2, 0xa2, 0x3c, 0x4c, 0x3c, 0x4c, 0x4e, 0x4e, 0x6e, 0x6e, 0x6c, 0x6c, 0xa2,
    0xa2, 0x0c, 0x1c, 0x0c, 0x1c, 0xe0, 0xc6, 0xc8, 0xe6, 0xe8, 0x20, 0x20, 8, 0x18, 0xc0, 0xc2,
    0xc2, 0, 0, 0xce, 0xce, 0xec, 0xec, 0xec, 0xec, 0xee, 0xee, 0xc4, 0xc4,
];
const K_GANON_DRAW_FLAGS: [u8; 204] = [
    0x4c, 0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x4c, 0x4c, 0x0c, 0x4c, 0x0a,
    0x4a, 0x0c, 0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x4c, 0x4c, 0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x0c, 0x4c,
    0x0a, 0x4a, 0x0c, 0x4c, 0x4c, 0x0c, 0x0c, 0x0a, 0x4a, 0x0c, 0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x4c,
    0x4c, 0x4c, 0x4c, 0x0a, 0x4a, 0x0c, 0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x4c, 0x4c, 0x0c, 0x4c, 0x0a,
    0x4a, 0x0c, 0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x4c, 0x4c, 0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x0c, 0x4c,
    0x0a, 0x4a, 0x0c, 0x4c, 0x4c, 0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x4c,
    0x0a, 0x4a, 0x0a, 0x4a, 0x0c, 0x4c, 0x0c, 0x4c, 0x0c, 0x0c, 0x4c, 0x4c, 0x0a, 0x4a, 0x0a, 0x4a,
    0x0c, 0x4c, 0x0c, 0x4c, 0x0c, 0x0c, 0x4c, 0x4c, 0x0a, 0x4a, 0x0a, 0x4a, 0x0c, 0x4c, 0x0c, 0x4c,
    0x0c, 0x0c, 0x4c, 0x4c, 0x0a, 0x4a, 0x0a, 0x4a, 0x0c, 0x0c, 0x0c, 0x4c, 0x0c, 0x0c, 0x4c, 0x4c,
    0x0a, 0x4a, 0x0a, 0x4a, 0x4c, 0x4c, 0x0c, 0x4c, 0x0c, 0x0c, 0x4c, 0x4c, 0x0a, 0x4a, 0x0a, 0x4a,
    0x0c, 0x4c, 0x0c, 0x4c, 0x0c, 0x0c, 0x4c, 0x4c, 0x0a, 0x4a, 0x0a, 0x4a, 0x0c, 0x4c, 0x0c, 0x4c,
    0x0c, 0x0c, 0x4c, 0x4c, 0x0c, 0x0a, 0x0a, 0x0a, 0x0a, 0x0c, 0x4c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c,
    0x0c, 0x4c, 0x0a, 0x4a, 0x0c, 0x0c, 0x4c, 0x4c, 0x0a, 0x4a, 0x0c, 0x4c,
];
const K_GANON_DRAW_CHAR2: [u8; 12] = [
    0x40, 0x42, 0, 0, 0x42, 0x40, 0x82, 0x80, 0xa0, 0xa0, 0x80, 0x82,
];
const K_GANON_DRAW_FLAGS2: [u8; 12] = [0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0, 0];
const K_GANON_SPR_OFFS: [u8; 17] = [1, 1, 1, 1, 1, 1, 15, 1, 4, 4, 4, 4, 4, 4, 4, 15, 15];
const K_GANON_DMD: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 16,
        y: -3,
        char_flags: 0x4c0a,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 5,
        char_flags: 0x4c1a,
        ext: 2,
    },
];
const K_GANON_LARGE_SHADOW_DMD: [DrawMultipleData; 15] = [
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
// Inline helpers from sprite_main.c:1570..1580 — GanonMult / GanonSin.
// ---------------------------------------------------------------------------

// static inline uint8 GanonMult(uint16 a, uint8 b) {
//   if (a >= 256)
//     return b;
//   int p = a * b;
//   return (p >> 8) + (p >> 7 & 1);
// }
fn ganon_mult(a: u16, b: u8) -> u8 {
    if a >= 256 {
        return b;
    }
    let p: u32 = u32::from(a) * u32::from(b);
    (((p >> 8) as u32).wrapping_add((p >> 7) & 1)) as u8
}

// static inline int8 GanonSin(uint16 a, uint8 b) {
//   uint8 t = GanonMult(kSinusLookupTable[a & 0xff], b);
//   return (a & 0x100) ? -t : t;
// }
fn ganon_sin(a: u16, b: u8) -> i8 {
    let t = ganon_mult(K_SINUS_LOOKUP_TABLE_GANON[(a & 0xff) as usize], b);
    if (a & 0x100) != 0 {
        (0i8).wrapping_sub(t as i8)
    } else {
        t as i8
    }
}

impl ZeldaState {
    fn replay_trace_ganon_matches(&self) -> bool {
        if std::env::var_os("ZELDA3_TRACE_GANON").is_none() {
            return false;
        }
        std::env::var("ZELDA3_TRACE_GANON_FRAME")
            .ok()
            .and_then(|value| {
                let trimmed = value.trim();
                if let Some(hex) = trimmed.strip_prefix("0x") {
                    u8::from_str_radix(hex, 16).ok()
                } else {
                    trimmed.parse::<u8>().ok()
                }
            })
            .is_none_or(|frame| frame == self.ram[FRAME_COUNTER])
    }

    // void SwishEvery16Frames(int k) {  // 9d8aa9
    pub(super) fn swish_every16_frames(&mut self, k: usize) {
        if (self.ram[FRAME_COUNTER] & 15) == 0 {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x06);
        }
    }

    // void Sprite_GanonTrident(int k) {  // 9d8ab6
    pub(super) fn sprite_ganon_trident(&mut self, k: usize) {
        self.trident_draw_for_ganon(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.swish_every16_frames(k);
        self.sprite_move_xy(k);
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_sub(1);
        let j = ((self.ram[SPRITE_SUBTYPE2 + k] >> 2) & 7) as usize;
        self.ram[SPRITE_G_GANON + k] = K_GANON_G_FUNC2[j];
        if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
            if (self.ram[SPRITE_DELAY_MAIN + k] & 1) != 0 {
                return;
            }
            let pt = self.sprite_project_speed_towards_link(k, 32);
            self.sprite_approach_target_speed(k, pt.x, pt.y);
        } else {
            let x = self
                .sprite_get_x(0)
                .wrapping_add_signed(if self.ram[SPRITE_D] != 0 { -16 } else { 24 });
            let y = self.sprite_get_y(0).wrapping_sub(16);
            if self.ganon_attempt_trident_catch(x, y) {
                self.ram[SPRITE_STATE + k] = 0;
                self.ram[SPRITE_AI_STATE] = 3;
                self.ram[SPRITE_DELAY_MAIN] = 16;
            }
            let pt = self.sprite_project_speed_towards_location(k, x, y, 32);
            self.sprite_approach_target_speed(k, pt.x, pt.y);
        }
    }

    // void Sprite_FireBat_Trailer(int k) {  // 9d8b49
    pub(super) fn sprite_fire_bat_trailer(&mut self, k: usize) {
        self.fire_bat_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.fire_bat_move(k);
    }

    // void Sprite_SpiralFireBat(int k) {  // 9d8b52
    pub(super) fn sprite_spiral_fire_bat(&mut self, k: usize) {
        self.fire_bat_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let x = (u16::from(self.ram[SPRITE_B_GANON + k]) << 8) | u16::from(self.ram[SPRITE_A + k]);
        let y = (u16::from(self.ram[SPRITE_E + k]) << 8) | u16::from(self.ram[SPRITE_C_GANON + k]);
        let pt = self.sprite_project_speed_towards_location(k, x, y, 2);
        let pt2 = self.sprite_project_speed_towards_location(k, x, y, 80);
        self.ram[SPRITE_X_VEL + k] = pt2.y.wrapping_sub(pt.x);
        self.ram[SPRITE_Y_VEL + k] = 0u8.wrapping_sub(pt2.x).wrapping_sub(pt.y);
        self.fire_bat_move(k);
    }

    // void Sprite_FireBat_Launched(int k) {  // 9d8bd7
    pub(super) fn sprite_fire_bat_launched(&mut self, k: usize) {
        self.fire_bat_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_link(k);
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.get_position_relative_to_the_great_overlord_ganon(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                } else {
                    self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_DELAY_MAIN + k] >> 2) & 1;
                }
            }
            1 => {
                self.get_position_relative_to_the_great_overlord_ganon(k);
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_SUBTYPE2 + k] >> 2) & 1;
            }
            2 => {
                self.sprite_move_xy(k);
                self.ram[SPRITE_DEFL_BITS + k] = 64;
                if self.ram[SPRITE_DELAY_AUX1_GANON + k] == 0 {
                    if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                        self.fire_bat_animate(k);
                        self.fire_bat_animate(k);
                    } else {
                        let mut t = self.ram[SPRITE_DELAY_MAIN + k].wrapping_sub(1);
                        if t == 0 {
                            t = 35;
                            self.ram[SPRITE_DELAY_AUX1_GANON + k] = t;
                        }
                        self.ram[SPRITE_GRAPHICS + k] = (t >> 2) & 1;
                    }
                } else if self.ram[SPRITE_DELAY_AUX1_GANON + k] == 1 {
                    self.sprite_apply_speed_towards_link_for_ganon(k, 48);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x1e);
                    self.fire_bat_animate(k);
                    self.fire_bat_animate(k);
                } else {
                    const GFX2: [u8; 9] = [4, 4, 4, 3, 3, 3, 2, 2, 2];
                    self.ram[SPRITE_GRAPHICS + k] =
                        GFX2[(self.ram[SPRITE_DELAY_AUX1_GANON + k] >> 2) as usize];
                }
            }
            _ => {}
        }
    }

    // void Sprite_D6_Ganon(int k) {  // 9d8eb4
    pub(super) fn sprite_d6_ganon(&mut self, k: usize) {
        if self.replay_trace_ganon_matches() {
            eprintln!(
                "R ganon fc={} entry k={} ai=0x{:02x} delay=0x{:02x} health=0x{:02x} subtype=0x{:02x} d=0x{:02x} hit=0x{:02x} aux1=0x{:02x} aux2=0x{:02x} aux4=0x{:02x} x=0x{:04x} y=0x{:04x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ram[SPRITE_AI_STATE + k],
                self.ram[SPRITE_DELAY_MAIN + k],
                self.ram[SPRITE_HEALTH_GANON + k],
                self.ram[SPRITE_SUBTYPE + k],
                self.ram[SPRITE_D + k],
                self.ram[SPRITE_HIT_TIMER_GANON + k],
                self.ram[SPRITE_DELAY_AUX1_GANON + k],
                self.ram[SPRITE_DELAY_AUX2_GANON + k],
                self.ram[SPRITE_DELAY_AUX4 + k],
                self.sprite_get_x(k),
                self.sprite_get_y(k),
            );
        }
        if sign8(self.ram[SPRITE_AI_STATE + k]) {
            if self.sprite_return_if_inactive(k) {
                return;
            }
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            if (self.ram[SPRITE_DELAY_MAIN + k] & 1) == 0 {
                self.ganon_draw(k);
            }
            return;
        }

        if self.ram[SPRITE_DELAY_AUX4 + k] != 0 {
            const GFXB: [u8; 2] = [16, 10];
            self.ram[SPRITE_GRAPHICS + k] = GFXB[(self.ram[SPRITE_D + k] & 1) as usize];
        }

        if self.ram[GANON_TORCH_COUNT] == 2
            && self.ram[GANON_TORCH_COUNT] != self.ram[SPRITE_ROOM + k]
        {
            self.ram[SPRITE_DELAY_AUX1_GANON + k] = 64;
        }
        self.ram[SPRITE_ROOM + k] = self.ram[GANON_TORCH_COUNT];

        self.ganon_draw(k);
        if self.ram[SPRITE_DELAY_AUX1_GANON + k] != 0 {
            self.ram[SPRITE_GRAPHICS + k] = 15;
            self.ganon_enable_invincibility(k);
            self.sprite_check_damage_to_and_from_link(k);
            return;
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }

        if self.ram[SPRITE_DELAY_AUX2_GANON + k] == 1 {
            self.ganon_extinguish_torch_for_ganon();
        } else if self.ram[SPRITE_DELAY_AUX2_GANON + k] == 16 {
            self.ganon_extinguish_torch_adjust_translucency_for_ganon();
        }

        let pair = self.sprite_is_right_of_link(k);
        self.ram[SPRITE_HEAD_DIR + k] = if pair.b.wrapping_add(32) < 64 {
            1
        } else if pair.a != 0 {
            0
        } else {
            2
        };

        if self.ram[SPRITE_DELAY_AUX4 + k] != 0 {
            self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k] = self.ram[SPRITE_DELAY_AUX4 + k];
            if self.sprite_return_if_recoiling(k) {
                return;
            }
            self.ram[SPRITE_DELAY_MAIN + k] = 0;
            return;
        }

        if (self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k] | self.ram[FLAG_IS_LINK_IMMOBILIZED]) == 0
            && self.ram[GANON_TORCH_COUNT] == 2
        {
            self.sprite_check_damage_to_and_from_link(k);
        }
        self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k] = 0;

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = 128;
                } else if self.ram[SPRITE_DELAY_MAIN + k] == 32 {
                    self.ram[MUSIC_CONTROL] = 0x1f;
                } else if self.ram[SPRITE_DELAY_MAIN + k] == 64 {
                    write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x16f);
                    self.sprite_show_message_minimal_c();
                }
            }
            1 => {
                if self.ram[SPRITE_HEALTH_GANON + k] < 209 {
                    self.ram[SPRITE_HEALTH_GANON + k] = 208;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] < 64 {
                    if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                        self.ganon_select_warp_location(k, 5);
                    } else {
                        const GFX1: [u8; 2] = [2, 10];
                        self.ram[SPRITE_GRAPHICS + k] = GFX1[(self.ram[SPRITE_D + k] & 1) as usize];
                    }
                } else if self.ram[SPRITE_DELAY_MAIN + k] != 64 {
                    self.ganon_phase1_animate_trident_spin(k);
                } else {
                    const X1: [i8; 2] = [24, -16];
                    const Y1: [i8; 2] = [4, 4];
                    const XVEL1: [i8; 16] = [
                        32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16, 0, 16, 24, 28,
                    ];
                    const YVEL1: [i8; 16] = [
                        0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16,
                    ];
                    self.ram[SPRITE_G_GANON + k] = 0;
                    let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0xc9, &mut info);
                    assert!(
                        j >= 0,
                        "Sprite_D6_Ganon expected phase-1 trident spawn to succeed"
                    );
                    let j = j as usize;
                    let i = usize::from(self.ram[SPRITE_D + k] & 1);
                    self.sprite_set_x(j, info.r0_x.wrapping_add_signed(i16::from(X1[i])));
                    self.sprite_set_y(j, info.r2_y.wrapping_add_signed(i16::from(Y1[i])));
                    self.sprite_apply_speed_towards_link_for_ganon(k, 31);
                    let angle = Self::sprite_convert_velocity_to_angle(
                        self.ram[SPRITE_X_VEL + k],
                        self.ram[SPRITE_Y_VEL + k],
                    );
                    let vi = usize::from(angle.wrapping_sub(2) & 0x0f);
                    self.ram[SPRITE_X_VEL + j] = XVEL1[vi] as u8;
                    self.ram[SPRITE_Y_VEL + j] = YVEL1[vi] as u8;
                    self.ram[SPRITE_DELAY_MAIN + j] = 112;
                    self.ram[SPRITE_ANIM_CLOCK_GANON + j] = 2;
                    self.ram[SPRITE_OAM_FLAGS + j] = 1;
                    self.ram[SPRITE_FLAGS2 + j] = 4;
                    self.ram[SPRITE_DEFL_BITS + j] = 0x84;
                    self.ram[SPRITE_D + j] = 2;
                    self.ram[SPRITE_BUMP_DAMAGE_GANON + j] = 7;
                    self.ram[SPRITE_IGNORE_PROJECTILE_GANON + j] = 7;
                }
            }
            2 => {
                if self.ram[SPRITE_HEALTH_GANON + k] < 209 {
                    self.ram[SPRITE_HEALTH_GANON + k] = 208;
                }
                const GFX2: [u8; 2] = [0, 8];
                self.ram[SPRITE_GRAPHICS + k] = GFX2[(self.ram[SPRITE_D + k] & 1) as usize];
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                    self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k] =
                        self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k].wrapping_add(1);
                    if (self.ram[SPRITE_DELAY_MAIN + k] & 1) != 0 {
                        self.ram[SPRITE_GRAPHICS + k] = 255;
                    }
                }
            }
            3 => {
                if self.ram[SPRITE_HEALTH_GANON + k] < 209 {
                    self.ram[SPRITE_HEALTH_GANON + k] = 208;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                    self.ganon_phase1_animate_trident_spin(k);
                } else {
                    self.ram[SPRITE_AI_STATE + k] = 6;
                    self.ram[SPRITE_DELAY_MAIN + k] = 127;
                    self.ganon_handle_animation_idle(k);
                }
            }
            4 => {
                if self.ram[SPRITE_HEALTH_GANON + k] < 209 {
                    self.ram[SPRITE_HEALTH_GANON + k] = 208;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                    self.ganon_shake_head(k);
                } else {
                    self.ganon_select_warp_location(k, 5);
                }
            }
            5 | 10 | 13 | 18 => {
                if self.ram[SPRITE_AI_STATE + k] == 13 {
                    self.ram[SPRITE_HEALTH_GANON + k] = 100;
                }
                self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k] =
                    self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k].wrapping_add(1);
                let x = (u16::from(self.ram[SPRITE_X_HI + k]) << 8)
                    | u16::from(self.ram[SWAMOLA_TARGET_X_LO_GANON]);
                let y = (u16::from(self.ram[SPRITE_Y_HI + k]) << 8)
                    | u16::from(self.ram[SWAMOLA_TARGET_Y_LO_GANON]);
                if self.ganon_attempt_trident_catch(x, y) {
                    self.ram[SPRITE_D + k] = self.ram[SPRITE_SUBTYPE + k] >> 2;
                    if self.ram[SPRITE_AI_STATE + k] == 5 {
                        self.ram[SPRITE_AI_STATE + k] = 2;
                        self.ram[SPRITE_DELAY_MAIN + k] = 32;
                    } else if self.ram[SPRITE_HEALTH_GANON + k] >= 161 {
                        self.ram[SPRITE_AI_STATE + k] = 11;
                        self.ram[SPRITE_DELAY_MAIN + k] = 40;
                    } else if self.ram[SPRITE_HEALTH_GANON + k] >= 97 {
                        self.ram[SPRITE_AI_STATE + k] = 14;
                        self.ram[SPRITE_DELAY_MAIN + k] = 40;
                    } else {
                        self.ram[SPRITE_AI_STATE + k] = 17;
                        self.ram[SPRITE_DELAY_MAIN + k] = 104;
                    }
                } else {
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 32);
                    self.sprite_approach_target_speed(k, pt.x, pt.y);
                    self.sprite_move_xy(k);
                    if self.ram[SPRITE_DELAY_MAIN + k] == 0 || (self.ram[FRAME_COUNTER] & 1) != 0 {
                        self.ram[SPRITE_GRAPHICS + k] = 255;
                        return;
                    }
                    const GFX5: [u8; 2] = [2, 10];
                    self.ram[SPRITE_GRAPHICS + k] = GFX5[(self.ram[SPRITE_D + k] & 1) as usize];
                    if (self.ram[FRAME_COUNTER] & 7) == 0 {
                        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
                        let j = self.sprite_spawn_dynamically(k, 0xd6, &mut info);
                        if j >= 0 {
                            let j = j as usize;
                            self.sprite_set_spawned_coordinates(j, &info);
                            self.ram[SPRITE_IGNORE_PROJECTILE_GANON + j] = 24;
                            self.ram[SPRITE_DELAY_MAIN + j] = 24;
                            self.ram[SPRITE_AI_STATE + j] = 255;
                            self.ram[SPRITE_GRAPHICS + j] = self.ram[SPRITE_GRAPHICS + k];
                            self.ram[SPRITE_HEAD_DIR + j] = self.ram[SPRITE_HEAD_DIR + k];
                        }
                    }
                }
            }
            6 => {
                if self.ram[SPRITE_HEALTH_GANON + k] < 209 {
                    self.ram[SPRITE_HEALTH_GANON + k] = 208;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    if self.ram[SPRITE_HEALTH_GANON + k] >= 209 {
                        self.ram[SPRITE_AI_STATE + k] = 1;
                        self.ram[SPRITE_DELAY_MAIN + k] = 128;
                    } else {
                        self.ram[SPRITE_DELAY_MAIN + k] = 255;
                        self.ram[SPRITE_AI_STATE + k] = 7;
                    }
                } else {
                    self.ganon_shake_head(k);
                }
            }
            7 => {
                if self.ram[SPRITE_HEALTH_GANON + k] < 161 {
                    self.ram[SPRITE_HEALTH_GANON + k] = 160;
                }
                self.ram[OVERLORD_X_LO_GANON + 2] = 40;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 8;
                    self.ram[SPRITE_DELAY_MAIN + k] = 255;
                } else {
                    if self.ram[SPRITE_DELAY_MAIN + k] < 0xc0
                        && (self.ram[SPRITE_DELAY_MAIN + k] & 0x0f) == 0
                    {
                        self.ganon_spawn_spiral_bat(k);
                    }
                    self.ganon_phase1_animate_trident_spin(k);
                    self.ganon_handle_fire_bat_circle(k);
                }
            }
            8 => {
                const TAB2: [i8; 16] = [0, 0, 0, 0, -1, -1, -2, -1, 0, 0, 0, 0, 1, 2, 1, 1];
                const DELAY8: [u8; 8] = [0x10, 0x30, 0x50, 0x70, 0x90, 0xb0, 0xd0, 0xbd];
                if self.ram[SPRITE_HEALTH_GANON + k] < 161 {
                    self.ram[SPRITE_HEALTH_GANON + k] = 160;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 9;
                    self.ram[SPRITE_DELAY_MAIN + k] = 127;
                    self.ganon_handle_animation_idle(k);
                    for j in (1..=8usize).rev() {
                        self.ram[SPRITE_AI_STATE + j] = 2;
                        self.ram[SPRITE_DELAY_MAIN + j] = DELAY8[j - 1];
                    }
                } else {
                    let idx = ((self.ram[SPRITE_DELAY_MAIN + k] >> 4) & 15) as usize;
                    self.ram[OVERLORD_X_LO_GANON + 2] =
                        self.ram[OVERLORD_X_LO_GANON + 2].wrapping_add(TAB2[idx] as u8);
                    self.ganon_phase1_animate_trident_spin(k);
                    self.ganon_handle_fire_bat_circle(k);
                }
            }
            9 => {
                if self.ram[SPRITE_HEALTH_GANON + k] < 161 {
                    self.ram[SPRITE_HEALTH_GANON + k] = 160;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ganon_select_warp_location(k, 10);
                } else {
                    self.ganon_shake_head(k);
                }
            }
            11 => {
                self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k] =
                    self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k].wrapping_add(1);
                self.ganon_handle_animation_idle(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 255;
                    self.ram[SPRITE_AI_STATE + k] = 7;
                } else if (self.ram[SPRITE_DELAY_MAIN + k] & 1) != 0 {
                    self.ram[SPRITE_GRAPHICS + k] = 255;
                }
            }
            12 => {
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    self.ganon_select_warp_location(k, 13);
                    return;
                }
                let mut t = 0usize;
                if j < 96 {
                    t = 1;
                    if j < 72 {
                        if j == 66 {
                            self.ganon_func1(k, 3);
                        }
                        t = 2;
                    }
                }
                if self.ram[SPRITE_D + k] != 0 {
                    t += 3;
                }
                const GFX12: [u8; 6] = [5, 6, 7, 13, 14, 10];
                self.ram[SPRITE_GRAPHICS + k] = GFX12[t];
                if (self.ram[SPRITE_HIT_TIMER_GANON + k] & 127) == 1 {
                    self.ram[SPRITE_AI_STATE + k] = 15;
                    self.ram[SPRITE_Z_VEL + k] = 24;
                    self.ram[SPRITE_DELAY_MAIN + k] = 0;
                }
            }
            14 => {
                self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k] =
                    self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k].wrapping_add(1);
                self.ganon_handle_animation_idle(k);
                self.ram[SPRITE_G_GANON + k] = 0;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    if (self.get_random_number() & 1) != 0 {
                        self.ganon_select_warp_location(k, 13);
                    } else {
                        self.ram[SPRITE_DELAY_MAIN + k] = 127;
                        self.ram[SPRITE_AI_STATE + k] = 12;
                    }
                } else if (self.ram[SPRITE_DELAY_MAIN + k] & 1) != 0 {
                    self.ram[SPRITE_GRAPHICS + k] = 255;
                }
            }
            15 => {
                const GFX15: [u8; 2] = [6, 14];
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                    if self.ram[SPRITE_DELAY_MAIN + k] == 1 {
                        self.ram[SPRITE_AI_STATE + k] = 16;
                        self.ram[SPRITE_Z_VEL + k] = 160;
                        return;
                    }
                } else {
                    self.sprite_move_z(k);
                    self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(1);
                    if self.ram[SPRITE_Z_VEL + k] == 0 {
                        self.ram[SPRITE_DELAY_MAIN + k] = 32;
                    }
                }
                self.ram[SPRITE_GRAPHICS + k] = GFX15[(self.ram[SPRITE_D + k] & 1) as usize];
            }
            16 => {
                write_le_u16(&mut self.ram, BG1_Y_OFFSET, 0);
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                    if self.ram[SPRITE_DELAY_MAIN + k] == 1 {
                        self.ram[SOUND_EFFECT_AMBIENT] = 5;
                        self.ganon_select_warp_location(k, 13);
                        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
                        self.ganon_spawn_falling_tiles_overlord(k);
                        if self.ram[SPRITE_ANIM_CLOCK_GANON + k] >= 4 {
                            self.ganon_select_warp_location(k, 10);
                            self.ram[SPRITE_HEALTH_GANON + k] = 96;
                            self.ram[SPRITE_DELAY_AUX2_GANON + k] = 224;
                            write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x170);
                            self.sprite_show_message_minimal_c();
                        }
                    } else {
                        let offs: u16 = if ((self.ram[SPRITE_DELAY_MAIN + k] - 1) & 1) != 0 {
                            (-1i16) as u16
                        } else {
                            1
                        };
                        write_le_u16(&mut self.ram, BG1_Y_OFFSET, offs);
                        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
                    }
                } else {
                    const GFX16: [u8; 2] = [2, 10];
                    self.sprite_move_z(k);
                    if sign8(self.ram[SPRITE_Z + k]) {
                        self.ram[SPRITE_Z_VEL + k] = 0;
                        self.ram[SPRITE_Z + k] = 0;
                        self.ram[SPRITE_DELAY_MAIN + k] = 96;
                        self.ram[SOUND_EFFECT_AMBIENT] = 7;
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
                    }
                    self.ram[SPRITE_GRAPHICS + k] = GFX16[(self.ram[SPRITE_D + k] & 1) as usize];
                }
            }
            17 => {
                const GFX17B: [u8; 2] = [6, 14];
                const GFX17: [u8; 2] = [7, 10];
                self.ram[SPRITE_GRAPHICS + k] = GFX17B[(self.ram[SPRITE_D + k] & 1) as usize];
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ganon_select_warp_location(k, 0x12);
                    return;
                } else if self.ram[SPRITE_DELAY_MAIN + k] == 52 {
                    self.ganon_func1(k, 5);
                } else if self.ram[SPRITE_DELAY_MAIN + k] < 52 {
                    self.ram[SPRITE_GRAPHICS + k] = GFX17[(self.ram[SPRITE_D + k] & 1) as usize];
                }
                if self.ram[SPRITE_DELAY_MAIN + k] >= 72 || self.ram[SPRITE_DELAY_MAIN + k] < 40 {
                    self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k] =
                        self.ram[SPRITE_IGNORE_PROJECTILE_GANON + k].wrapping_add(1);
                    if (self.ram[SPRITE_DELAY_MAIN + k] & 1) != 0 {
                        self.ram[SPRITE_GRAPHICS + k] = 255;
                    }
                }
                self.ganon_enable_invincibility(k);
            }
            19 => {
                self.ram[SPRITE_OAM_FLAGS + k] = 5;
                self.ram[SPRITE_FLAGS + k] = 2;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_OAM_FLAGS + k] = 1;
                    self.ganon_select_warp_location(k, 18);
                    self.ram[SPRITE_TYPE + k] = 0xd6;
                    self.ram[SPRITE_HIT_TIMER_GANON + k] = 0;
                } else {
                    const GFX19: [u8; 2] = [5, 13];
                    self.ram[SPRITE_GRAPHICS + k] = GFX19[(self.ram[SPRITE_D + k] & 1) as usize];
                }
            }
            _ => {}
        }
    }

    // void PhantomGanon_Draw(int k) {  // sprite_main.c:14392
    //   static const DrawMultipleData kPhantomGanon_Dmd[16] = { ... };
    //   oam_cur_ptr = 0x950;
    //   oam_ext_cur_ptr = 0xa74;
    //   Sprite_DrawMultiple(k, &kPhantomGanon_Dmd[sprite_graphics[k] * 8], 8, NULL);
    // }
    pub(super) fn phantom_ganon_draw(&mut self, k: usize) {
        write_le_u16(&mut self.ram, OAM_CUR_PTR, 0x950);
        write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0xa74);
        let g = self.ram[SPRITE_GRAPHICS + k] as usize;
        // Sprite_DrawMultiple emits 8 OAM entries starting at index g*8.
        self.sprite_draw_multiple_for_ganon(k, &K_PHANTOM_GANON_DMD, g * 8, 8);
    }

    // void Sprite_SpawnPhantomGanon(int k) {  // 9d88a1
    pub(super) fn sprite_spawn_phantom_ganon(&mut self, k: usize) {
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc9, &mut info);
        assert!(
            j >= 0,
            "Sprite_SpawnPhantomGanon expected Sprite_SpawnDynamically to succeed"
        );
        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        self.ram[SPRITE_FLAGS2 + j] = 2;
        self.ram[SPRITE_IGNORE_PROJECTILE + j] = 2;
        self.ram[SPRITE_ANIM_CLOCK + j] = 1;
        self.ram[SPRITE_OAM_FLAGS + j] = 0;
    }

    // void Sprite_PhantomGanon(int k) {  // 9d88bc
    pub(super) fn sprite_phantom_ganon(&mut self, k: usize) {
        const GFX: [u8; 4] = [0, 1, 2, 1];
        const TARGET_XVEL: [u8; 2] = [32, (-32i8) as u8];
        const TARGET_YVEL: [u8; 2] = [16, (-16i8) as u8];

        if self.ram[SPRITE_AI_STATE + k] == 0 {
            self.phantom_ganon_draw(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_move_y(k);
            self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
            if (self.ram[SPRITE_SUBTYPE2 + k] & 31) == 0 {
                self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_sub(1);
                if self.ram[SPRITE_Y_VEL + k] == 252 {
                    let j = self.spawn_boss_poof(k);
                    assert!(
                        j >= 0,
                        "Sprite_PhantomGanon expected SpawnBossPoof to succeed"
                    );
                    let j = j as usize;
                    let y = self.sprite_get_y(j).wrapping_sub(20);
                    self.sprite_set_y(j, y);
                } else if self.ram[SPRITE_Y_VEL + k] == 251 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 255;
                    self.ram[SPRITE_Y_VEL + k] = (-4i8) as u8;
                }
            }
        } else {
            self.ganon_bat_draw(k);
            if self.ram[SPRITE_PAUSE + k] != 0 {
                self.ram[SPRITE_STATE + k] = 0;
                let bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) | 0x8000;
                write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, bits);
            }
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.ram[SPRITE_GRAPHICS + k] = GFX[usize::from((self.ram[FRAME_COUNTER] >> 2) & 3)];
            if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                if self.ram[SPRITE_DELAY_MAIN + k] < 208 {
                    let j = usize::from(self.ram[SPRITE_HEAD_DIR + k] & 1);
                    self.ram[SPRITE_Y_VEL + k] =
                        self.ram[SPRITE_Y_VEL + k].wrapping_add(if j != 0 { 0xff } else { 1 });
                    if self.ram[SPRITE_Y_VEL + k] == TARGET_YVEL[j] {
                        self.ram[SPRITE_HEAD_DIR + k] =
                            self.ram[SPRITE_HEAD_DIR + k].wrapping_add(1);
                    }
                    let j = usize::from(self.ram[SPRITE_D + k] & 1);
                    self.ram[SPRITE_X_VEL + k] =
                        self.ram[SPRITE_X_VEL + k].wrapping_add(if j != 0 { 0xff } else { 1 });
                    if self.ram[SPRITE_X_VEL + k] == TARGET_XVEL[j] {
                        self.ram[SPRITE_D + k] = self.ram[SPRITE_D + k].wrapping_add(1);
                    }
                    if self.ram[SPRITE_X_VEL + k] == 0 {
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x1e);
                    }
                }
                let x = self.player_state_view().x() & 0xff00 | 0x78;
                let y = self.player_state_view().y() & 0xff00 | 0x50;
                let pt = self.sprite_project_speed_towards_location(k, x, y, 5);
                let xvel = self.ram[SPRITE_X_VEL + k];
                let yvel = self.ram[SPRITE_Y_VEL + k];
                self.ram[SPRITE_X_VEL + k] = xvel.wrapping_add(pt.x);
                self.ram[SPRITE_Y_VEL + k] = yvel.wrapping_add(pt.y);
                self.sprite_move_xy(k);
                self.ram[SPRITE_X_VEL + k] = xvel;
                self.ram[SPRITE_Y_VEL + k] = yvel;
            } else {
                self.sprite_move_xy(k);
                if self.ram[SPRITE_X_VEL + k] != 64 {
                    self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_add(1);
                    self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_sub(1);
                }
            }
        }
    }

    // bool Ganon_AttemptTridentCatch(uint16 x, uint16 y) {  // sprite_main.c:14554
    //   return (uint16)(cur_sprite_x - x + 4) < 8 && (uint16)(cur_sprite_y - y + 4) < 8;
    // }
    pub(super) fn ganon_attempt_trident_catch(&self, x: u16, y: u16) -> bool {
        let cx = read_le_u16(&self.ram, CUR_SPRITE_X);
        let cy = read_le_u16(&self.ram, CUR_SPRITE_Y);
        cx.wrapping_sub(x).wrapping_add(4) < 8 && cy.wrapping_sub(y).wrapping_add(4) < 8
    }

    // void Ganon_HandleFireBatCircle(int k) {  // sprite_main.c:14558
    //   static const int8 kGanonMath_X[16] = { 0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16 };
    //   static const int8 kGanonMath_Y[16] = { 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16, 0, 16, 24, 28 };
    //   WORD(overlord_x_lo[0]) -= 4;
    //   for (int i = 0; i != 8; i++) {
    //     int t = WORD(overlord_x_lo[0]) + i * 64 & 0x1ff;
    //     if (sprite_ai_state[i + 1] != 2) {
    //       int j = (t >> 5) - 4 & 0xf;
    //       sprite_x_vel[i + 1] = (int8)kGanonMath_X[j] >> 2;
    //       sprite_y_vel[i + 1] = (int8)kGanonMath_Y[j] >> 2;
    //     }
    //     int x = Sprite_GetX(0) + GanonSin(t, overlord_x_lo[2]);
    //     overlord_x_hi[i + 1] = x;
    //     overlord_y_hi[i + 1] = x >> 8;
    //     int y = Sprite_GetY(0) + GanonSin(t + 0x80, overlord_x_lo[2]);
    //     overlord_gen2[i + 1] = y;
    //     overlord_floor[i + 1] = y >> 8;
    //   }
    //   tmp_counter = 8;
    // }
    pub(super) fn ganon_handle_fire_bat_circle(&mut self, _k: usize) {
        // WORD(overlord_x_lo[0]) -= 4 — 16-bit wrap.
        let w = read_le_u16(&self.ram, OVERLORD_X_LO_GANON);
        write_le_u16(&mut self.ram, OVERLORD_X_LO_GANON, w.wrapping_sub(4));

        let scale = self.ram[OVERLORD_X_LO_GANON + 2];
        let sprite0_x = self.sprite_get_x(0);
        let sprite0_y = self.sprite_get_y(0);

        for i in 0..8usize {
            let base = read_le_u16(&self.ram, OVERLORD_X_LO_GANON);
            let t: u16 = base.wrapping_add((i as u16).wrapping_mul(64)) & 0x1ff;
            if self.ram[SPRITE_AI_STATE + i + 1] != 2 {
                let j = ((t >> 5).wrapping_sub(4) & 0xf) as usize;
                // (int8)kGanonMath_X[j] >> 2 — arithmetic shift on signed.
                self.ram[SPRITE_X_VEL + i + 1] = ((K_GANON_MATH_X[j] as i8) >> 2) as u8;
                self.ram[SPRITE_Y_VEL + i + 1] = ((K_GANON_MATH_Y[j] as i8) >> 2) as u8;
            }
            // x = Sprite_GetX(0) + (int8)GanonSin(t, overlord_x_lo[2])
            // i32 to allow the negative-extend before re-casting to 16-bit / 8-bit.
            let xs = ganon_sin(t, scale) as i16;
            let x = (sprite0_x as i32).wrapping_add(xs as i32);
            self.ram[OVERLORD_X_HI_GANON + i + 1] = x as u8;
            self.ram[OVERLORD_Y_HI_GANON + i + 1] = (x >> 8) as u8;

            let ys = ganon_sin(t.wrapping_add(0x80), scale) as i16;
            let y = (sprite0_y as i32).wrapping_add(ys as i32);
            self.ram[OVERLORD_GEN2_GANON + i + 1] = y as u8;
            self.ram[OVERLORD_FLOOR_GANON + i + 1] = (y >> 8) as u8;
        }
        self.ram[TMP_COUNTER_GANON] = 8;
    }

    // void Ganon_SpawnSpiralBat(int k) {  // sprite_main.c:14582
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamicallyEx(k, 0xc9, &info, 8);
    //   if (j < 0)
    //     return;
    //   Sprite_SetSpawnedCoordinates(j, &info);
    //   sprite_anim_clock[j] = 4;
    //   sprite_oam_flags[j] = 3;
    //   sprite_flags3[j] = 0x40;
    //   sprite_flags2[j] = 1;
    //   sprite_defl_bits[j] = 0x80;
    //   sprite_y_hi[j] = 128;
    //   sprite_delay_main[j] = 48;
    //   sprite_bump_damage[j] = 7;
    //   sprite_ignore_projectile[j] = 7;
    // }
    pub(super) fn ganon_spawn_spiral_bat(&mut self, k: usize) {
        if let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_ex_for_ganon(k, 0xc9, 8) {
            self.sprite_set_spawned_coordinates_for_ganon(j, r0_x, r2_y);
            self.ram[SPRITE_ANIM_CLOCK_GANON + j] = 4;
            self.ram[SPRITE_OAM_FLAGS + j] = 3;
            self.ram[SPRITE_FLAGS3 + j] = 0x40;
            self.ram[SPRITE_FLAGS2 + j] = 1;
            self.ram[SPRITE_DEFL_BITS + j] = 0x80;
            self.ram[SPRITE_Y_HI + j] = 128;
            self.ram[SPRITE_DELAY_MAIN + j] = 48;
            self.ram[SPRITE_BUMP_DAMAGE_GANON + j] = 7;
            self.ram[SPRITE_IGNORE_PROJECTILE_GANON + j] = 7;
        }
    }

    // void Ganon_EnableInvincibility(int k) {  // sprite_main.c:14979
    //   if ((sprite_hit_timer[k] & 127) == 26) {
    //     sprite_hit_timer[k] = 0;
    //     sprite_ai_state[k] = 19;
    //     sprite_delay_main[k] = 127;
    //     sprite_type[k] = 215;
    //   }
    // }
    pub(super) fn ganon_enable_invincibility(&mut self, k: usize) {
        if (self.ram[SPRITE_HIT_TIMER_GANON + k] & 127) == 26 {
            self.ram[SPRITE_HIT_TIMER_GANON + k] = 0;
            self.ram[SPRITE_AI_STATE + k] = 19;
            self.ram[SPRITE_DELAY_MAIN + k] = 127;
            self.ram[SPRITE_TYPE + k] = 215;
        }
    }

    // void Ganon_SpawnFallingTilesOverlord(int k) {  // sprite_main.c:14988
    //   static const uint8 kGanon_Ov_Type[4] = { 12, 13, 14, 15 };
    //   static const uint8 kGanon_Ov_X[4] = { 0x18, 0xd8, 0xd8, 0x18 };
    //   static const uint8 kGanon_Ov_Y[4] = { 0x28, 0x28, 0xd8, 0xd8 };
    //   int j;
    //   for (j = 7; j >= 0 && overlord_type[j] != 0; j--);
    //   int t = sprite_anim_clock[k];
    //   if (t >= 4)
    //     return;
    //   sprite_anim_clock[k] = t + 1;
    //   overlord_type[j] = kGanon_Ov_Type[t];
    //   overlord_x_lo[j] = kGanon_Ov_X[t];
    //   overlord_x_hi[j] = link_x_coord >> 8;
    //   overlord_y_lo[j] = kGanon_Ov_Y[t];
    //   overlord_y_hi[j] = link_y_coord >> 8;
    //   overlord_gen1[j] = 0;
    //   overlord_gen2[j] = 0;
    // }
    pub(super) fn ganon_spawn_falling_tiles_overlord(&mut self, k: usize) {
        // The C loop `for (j = 7; j >= 0 && overlord_type[j] != 0; j--);`
        // walks j down until a free slot is found; if all are taken, j == -1
        // and the C code still writes — i.e. it is the boss's responsibility
        // to keep at least one slot free. We mirror that: search 7..=0.
        let mut j_i32: i32 = 7;
        while j_i32 >= 0 && self.ram[OVERLORD_TYPE + j_i32 as usize] != 0 {
            j_i32 -= 1;
        }

        let t = self.ram[SPRITE_ANIM_CLOCK_GANON + k];
        if t >= 4 {
            return;
        }
        self.ram[SPRITE_ANIM_CLOCK_GANON + k] = t.wrapping_add(1);

        let j = |base: usize| base.wrapping_add_signed(j_i32 as isize);
        let ti = t as usize;
        self.ram[j(OVERLORD_TYPE)] = K_GANON_OV_TYPE[ti];
        self.ram[j(OVERLORD_X_LO_GANON)] = K_GANON_OV_X[ti];
        // overlord_x_hi[j] = link_x_coord >> 8  — read the high byte of the 16-bit link_x_coord.
        self.ram[j(OVERLORD_X_HI_GANON)] = self.ram[LINK_X_COORD + 1];
        // overlord_y_lo[j] sits at OVERLORD_X_LO_GANON + 16 (variables.h:649).
        self.ram[j(OVERLORD_X_LO_GANON + 0x10)] = K_GANON_OV_Y[ti];
        self.ram[j(OVERLORD_Y_HI_GANON)] = self.ram[LINK_Y_COORD + 1];
        // overlord_gen1 / overlord_gen2 — clear both.
        self.ram[j(OVERLORD_GEN2_GANON.wrapping_sub(8))] = 0; // gen1 lives at 0xB28
        self.ram[j(OVERLORD_GEN2_GANON)] = 0;
    }

    // void Ganon_Func1(int k, int t) {  // sprite_main.c:15010
    //   tmp_counter = t;
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamicallyEx(k, 0xC9, &info, 8);
    //   if (j < 0)
    //     return;
    //   SpriteSfx_QueueSfx2WithPan(k, 0x2a);
    //   Sprite_SetSpawnedCoordinates(j, &info);
    //   sprite_ignore_projectile[j] = sprite_anim_clock[j] = t;
    //   sprite_oam_flags[j] = 3;
    //   sprite_flags3[j] = 0x40;
    //   sprite_flags2[j] = 0x21;
    //   sprite_defl_bits[j] = 0x40;
    //   static const int8 kGanon_Gfx16_Y[2] = { 0, -16 };
    //   Sprite_SetY(j, info.r2_y + kGanon_Gfx16_Y[sprite_D[k]]);
    //   Sprite_ApplySpeedTowardsLink(j, 32);
    //   sprite_delay_main[j] = 16;
    //   sprite_A[j] = sprite_x_lo[0];
    //   sprite_B[j] = sprite_x_hi[0];
    //   sprite_C[j] = sprite_y_lo[0];
    //   sprite_E[j] = sprite_y_hi[0];
    //   sprite_bump_damage[j] = 7;
    //   sprite_ignore_projectile[j] = 7;
    // }
    pub(super) fn ganon_func1(&mut self, k: usize, t: u8) {
        self.ram[TMP_COUNTER_GANON] = t;
        if let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_ex_for_ganon(k, 0xC9, 8) {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x2a);
            self.sprite_set_spawned_coordinates_for_ganon(j, r0_x, r2_y);
            self.ram[SPRITE_IGNORE_PROJECTILE_GANON + j] = t;
            self.ram[SPRITE_ANIM_CLOCK_GANON + j] = t;
            self.ram[SPRITE_OAM_FLAGS + j] = 3;
            self.ram[SPRITE_FLAGS3 + j] = 0x40;
            self.ram[SPRITE_FLAGS2 + j] = 0x21;
            self.ram[SPRITE_DEFL_BITS + j] = 0x40;
            let d = self.ram[SPRITE_D + k] as usize;
            let y = r2_y.wrapping_add(K_GANON_GFX16_Y[d] as i16 as u16);
            self.sprite_set_y(j, y);
            self.sprite_apply_speed_towards_link_for_ganon(j, 32);
            self.ram[SPRITE_DELAY_MAIN + j] = 16;
            self.ram[SPRITE_A + j] = self.ram[SPRITE_X_LO + 0];
            self.ram[SPRITE_B_GANON + j] = self.ram[SPRITE_X_HI + 0];
            self.ram[SPRITE_C_GANON + j] = self.ram[SPRITE_Y_LO + 0];
            self.ram[SPRITE_E + j] = self.ram[SPRITE_Y_HI + 0];
            self.ram[SPRITE_BUMP_DAMAGE_GANON + j] = 7;
            self.ram[SPRITE_IGNORE_PROJECTILE_GANON + j] = 7;
        }
    }

    // void Ganon_Phase1_AnimateTridentSpin(int k) {  // sprite_main.c:15035
    //   static const uint8 kGanon_GfxFunc2[16] = { 0, 0, 1, 1, 0, 0, 1, 1, 8, 8, 9, 9, 8, 8, 9, 9 };
    //   int j = (sprite_delay_main[k] >> 2 & 7) + (sprite_D[k] ? 8 : 0);
    //   sprite_G[k] = kGanon_G_Func2[j];
    //   sprite_graphics[k] = kGanon_GfxFunc2[j];
    //   SwishEvery16Frames(k);
    // }
    pub(super) fn ganon_phase1_animate_trident_spin(&mut self, k: usize) {
        let base = ((self.ram[SPRITE_DELAY_MAIN + k] >> 2) & 7) as usize;
        let bonus = if self.ram[SPRITE_D + k] != 0 { 8 } else { 0 };
        let j = base + bonus;
        self.ram[SPRITE_G_GANON + k] = K_GANON_G_FUNC2[j];
        self.ram[SPRITE_GRAPHICS + k] = K_GANON_GFX_FUNC2[j];
        // SwishEvery16Frames(k) — inline-ported (sprite_main.c:14416).
        // void SwishEvery16Frames(int k) {
        //   if (!(frame_counter & 15))
        //     SpriteSfx_QueueSfx3WithPan(k, 0x6);
        // }
        if (self.ram[FRAME_COUNTER] & 15) == 0 {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x6);
        }
    }

    // void Ganon_HandleAnimation_Idle(int k) {  // sprite_main.c:15044
    //   static const uint8 kGanon_G[2] = { 9, 10 };
    //   static const uint8 kGanon_Gfx[2] = { 2, 10 };
    //   sprite_G[k] = kGanon_G[sprite_D[k]];
    //   sprite_graphics[k] = kGanon_Gfx[sprite_D[k]];
    // }
    pub(super) fn ganon_handle_animation_idle(&mut self, k: usize) {
        let d = self.ram[SPRITE_D + k] as usize;
        self.ram[SPRITE_G_GANON + k] = K_GANON_G_IDLE[d];
        self.ram[SPRITE_GRAPHICS + k] = K_GANON_GFX_IDLE[d];
    }

    // void Ganon_SelectWarpLocation(int k, int a) {  // sprite_main.c:15051
    //   int j;
    //   static const uint8 kGanon_NextSubtype[32] = { ... };
    //   static const uint8 kGanon_NextY[8] = { 0x40, 0x30, 0x30, 0x40, 0xb0, 0xc0, 0xc0, 0xb0 };
    //   static const uint8 kGanon_NextX[8] = { 0x30, 0x50, 0xa0, 0xc0, 0x40, 0x60, 0x90, 0xb0 };
    //   sprite_subtype[k] = j = kGanon_NextSubtype[GetRandomNumber() & 3 | sprite_subtype[k] << 2];
    //   swamola_target_x_lo[0] = kGanon_NextX[j];
    //   swamola_target_y_lo[0] = kGanon_NextY[j];
    //   sprite_ai_state[k] = a;
    //   sprite_x_vel[k] = sprite_y_vel[k] = 0;
    //   sprite_delay_main[k] = 48;
    //   SpriteSfx_QueueSfx3WithPan(k, 0x28);
    // }
    pub(super) fn ganon_select_warp_location(&mut self, k: usize, a: u8) {
        if self.replay_trace_ganon_matches() {
            eprintln!(
                "R ganon fc={} select k={} a=0x{:02x} ai=0x{:02x} delay=0x{:02x} health=0x{:02x} subtype=0x{:02x} d=0x{:02x} target=0x{:02x}/0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                a,
                self.ram[SPRITE_AI_STATE + k],
                self.ram[SPRITE_DELAY_MAIN + k],
                self.ram[SPRITE_HEALTH_GANON + k],
                self.ram[SPRITE_SUBTYPE + k],
                self.ram[SPRITE_D + k],
                self.ram[SWAMOLA_TARGET_X_LO_GANON],
                self.ram[SWAMOLA_TARGET_Y_LO_GANON],
            );
        }
        let rnd = self.get_random_number();
        // `GetRandomNumber() & 3 | sprite_subtype[k] << 2` — note C precedence:
        // `&` binds tighter than `|`, so `(rnd & 3) | (subtype << 2)`.
        let idx = ((rnd & 3) | (self.ram[SPRITE_SUBTYPE + k] << 2)) as usize;
        let j = K_GANON_NEXT_SUBTYPE[idx & 0x1f];
        self.ram[SPRITE_SUBTYPE + k] = j;
        let ju = j as usize;
        self.ram[SWAMOLA_TARGET_X_LO_GANON] = K_GANON_NEXT_X[ju];
        self.ram[SWAMOLA_TARGET_Y_LO_GANON] = K_GANON_NEXT_Y[ju];
        self.ram[SPRITE_AI_STATE + k] = a;
        self.ram[SPRITE_X_VEL + k] = 0;
        self.ram[SPRITE_Y_VEL + k] = 0;
        self.ram[SPRITE_DELAY_MAIN + k] = 48;
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x28);
    }

    // void Ganon_ShakeHead(int k) {  // sprite_main.c:15069
    //   static const uint8 kGanon_HeadDir[18] = { ... };
    //   sprite_head_dir[k] = kGanon_HeadDir[sprite_delay_main[k] >> 3];
    // }
    pub(super) fn ganon_shake_head(&mut self, k: usize) {
        let idx = (self.ram[SPRITE_DELAY_MAIN + k] >> 3) as usize;
        // The C array is 18 entries; sprite_delay_main >> 3 in the
        // boss-fight path is always within bounds (delay <= 127 -> idx <= 15
        // before the death cases). Guard defensively at 18 to mirror C
        // out-of-bounds memory access by clamping (no panic, no UB).
        self.ram[SPRITE_HEAD_DIR + k] = K_GANON_HEAD_DIR[idx % 18];
    }

    // void Ganon_Draw(int k) {  // sprite_main.c:15077
    pub(super) fn ganon_draw(&mut self, k: usize) {
        let g = self.ram[SPRITE_GRAPHICS + k];
        if sign8(g)
            || (self.ram[SPRITE_AI_STATE + k] != 19
                && self.ram[SPRITE_DELAY_AUX4 + k] == 0
                && self.ram[GANON_TORCH_COUNT] == 0)
        {
            let _ = self.sprite_prep_oam_coord_or_double_ret(k);
            return;
        }

        self.trident_draw_for_ganon(k);

        let Some(info) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };

        self.ganon_draw_emit_body_oam_for_ganon(k, info);
        self.ganon_draw_patch_head_oam_for_ganon(k);
        if self.frame_control_view().submodule() != 0 {
            self.sprite_correct_oam_entries_for_ganon(k, 9, 0xff);
        }

        if self.ram[SPRITE_G_GANON + k] == 9 {
            write_le_u16(&mut self.ram, OAM_CUR_PTR, 0x828);
            write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0xa2a);
            self.ganon_draw_emit_g9_overlay_for_ganon(k);
        }

        let z = (self.ram[SPRITE_Z + k] as u16).wrapping_sub(1);
        let frame: u16 = if (z >> 11) > 4 { 4 } else { z >> 11 };
        let cy = read_le_u16(&self.ram, CUR_SPRITE_Y);
        write_le_u16(&mut self.ram, CUR_SPRITE_Y, cy.wrapping_add(z));
        write_le_u16(&mut self.ram, OAM_CUR_PTR, 0x9f4);
        write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0xa9d);
        let bak = self.ram[SPRITE_OAM_FLAGS + k];
        self.ram[SPRITE_OAM_FLAGS + k] = 0;
        self.ram[SPRITE_OBJ_PRIO_GANON + k] = 48;
        self.sprite_draw_large_shadow_for_ganon(k, frame as usize);
        self.ram[SPRITE_OAM_FLAGS + k] = bak;
        self.sprite_get_16bit_coords_for_ganon(k);
    }

    // -----------------------------------------------------------------
    // Local helpers (each named with `_for_ganon` suffix to keep this split
    // module's adaptation points explicit).
    // -----------------------------------------------------------------

    // Rewired to canonical Sprite_SpawnDynamicallyEx port.
    fn sprite_spawn_dynamically_ex_for_ganon(
        &mut self,
        k: usize,
        what: u8,
        range: u8,
    ) -> Option<(usize, u16, u16)> {
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, what, &mut info, range as i32);
        if j < 0 {
            None
        } else {
            Some((j as usize, info.r0_x, info.r2_y))
        }
    }

    // Rewired to canonical Sprite_SetSpawnedCoordinates port.
    fn sprite_set_spawned_coordinates_for_ganon(&mut self, j: usize, r0_x: u16, r2_y: u16) {
        let info = crate::zelda_rtl::sprite::SpriteSpawnInfo {
            r0_x,
            r2_y,
            ..Default::default()
        };
        self.sprite_set_spawned_coordinates(j, &info);
    }

    // Sprite_ApplySpeedTowardsLink — projects vel towards Link and writes
    // sprite_x_vel/y_vel. The canonical helper is not ported yet; we use
    // sprite_project_speed_towards_link (which IS ported) and copy the
    // result. This matches the C body byte-for-byte.
    fn sprite_apply_speed_towards_link_for_ganon(&mut self, j: usize, speed: u8) {
        let pt = self.sprite_project_speed_towards_link(j, speed);
        self.ram[SPRITE_X_VEL + j] = pt.x;
        self.ram[SPRITE_Y_VEL + j] = pt.y;
    }

    // Ganon_ExtinguishTorch_adjust_translucency / Ganon_ExtinguishTorch and
    // Dungeon_ExtinguishTorch live in dungeon.c. Port them locally here because
    // Sprite_D6_Ganon calls them from sprite_main.c.
    fn ganon_extinguish_torch_adjust_translucency_for_ganon(&mut self) {
        self.Palette_AssertTranslucencySwap();
        self.ram[DUNGEON_TORCH_ATTR] = 0xc0;
        self.dungeon_extinguish_torch_for_ganon();
    }

    fn ganon_extinguish_torch_for_ganon(&mut self) {
        self.ram[DUNGEON_TORCH_ATTR] = 193;
        self.dungeon_extinguish_torch_for_ganon();
    }

    fn dungeon_extinguish_torch_for_ganon(&mut self) {
        let y = ((self.ram[DUNGEON_TORCH_ATTR] & 0x0f) as usize) * 2
            + read_le_u16(&self.ram, DUNG_INDEX_OF_TORCHES_START) as usize;
        let idx = y >> 1;
        let mut r8 = read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + idx * 2) & 0x7fff;
        write_le_u16(&mut self.ram, DUNG_OBJECT_TILEMAP_POS + idx * 2, r8);

        let opos = (read_le_u16(&self.ram, DUNG_OBJECT_POS_IN_OBJDATA + idx * 2) & 0xff) >> 1;
        write_le_u16(&mut self.ram, DUNG_TORCH_DATA_GANON + opos as usize * 2, r8);

        r8 &= 0x3fff;
        self.room_draw_adjust_torch_lighting_change(r8, 0x0ec2, r8);
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;

        if self.ram[DUNG_WANT_LIGHTS_OUT] != 0 && self.ram[DUNG_NUM_LIT_TORCHES] != 0 {
            self.ram[DUNG_NUM_LIT_TORCHES] = self.ram[DUNG_NUM_LIT_TORCHES].wrapping_sub(1);
            if self.ram[DUNG_NUM_LIT_TORCHES] < 3 {
                if self.ram[DUNG_NUM_LIT_TORCHES] == 0 {
                    self.ram[TS_COPY] = 1;
                }
                const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
                self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] =
                    LIT_TORCHES_COLOR_PLUS[self.ram[DUNG_NUM_LIT_TORCHES] as usize];
                self.frame_control_view_mut().set_submodule(10);
                self.frame_control_view_mut().set_subsubmodule(0);
            }
        }

        let torch_timer = (self.ram[DUNGEON_TORCH_ATTR] & 0x0f) as usize;
        self.ram[DUNG_TORCH_TIMERS_GANON + torch_timer] = 0;
        self.ram[DUNGEON_TORCH_ATTR] = 0;
    }

    fn sprite_draw_multiple_for_ganon(
        &mut self,
        k: usize,
        dmd: &[(i8, i8, u16, u8)],
        start: usize,
        count: usize,
    ) {
        let entries: Vec<DrawMultipleData> = dmd
            .get(start..start.saturating_add(count))
            .unwrap_or(&[])
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect();
        self.sprite_draw_multiple(k, &entries, None);
    }

    // Rewired to canonical Trident_Draw port.
    fn trident_draw_for_ganon(&mut self, k: usize) {
        self.trident_draw(k);
    }

    fn ganon_draw_emit_body_oam_for_ganon(&mut self, k: usize, info: (u16, u16, u8)) {
        let (info_x, info_y, info_flags) = info;
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize + 5 * 4;
        let g = self.ram[SPRITE_GRAPHICS + k] as usize;
        for i in 0..12 {
            let j = g * 12 + i;
            let flags_mask = if (info_flags & 0x0f) >= 5 { 0xf0 } else { 0xff };
            let flags = info_flags | (K_GANON_DRAW_FLAGS[j] & flags_mask);
            let x = info_x.wrapping_add_signed(i16::from(K_GANON_DRAW_X[j])) as u8;
            let y = info_y.wrapping_add_signed(i16::from(K_GANON_DRAW_Y[j])) as u8;
            self.set_oam_plain_at_for_ganon(oam, x, y, K_GANON_DRAW_CHAR[j], flags, 2);
            oam += 4;
        }
    }

    fn ganon_draw_patch_head_oam_for_ganon(&mut self, k: usize) {
        let g = self.ram[SPRITE_GRAPHICS + k] as usize;
        let offs = K_GANON_SPR_OFFS[g];
        if offs == 15 {
            return;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize + (5 + usize::from(offs)) * 4;
        let j = usize::from(self.ram[SPRITE_HEAD_DIR + k]) * 2
            + if self.ram[SPRITE_D + k] != 0 { 6 } else { 0 };
        self.ram[oam + 2] = K_GANON_DRAW_CHAR2[j];
        self.ram[oam + 3] = (self.ram[oam + 3] & 0x3f) | K_GANON_DRAW_FLAGS2[j];
        self.ram[oam + 6] = K_GANON_DRAW_CHAR2[j + 1];
        self.ram[oam + 7] = (self.ram[oam + 7] & 0x3f) | K_GANON_DRAW_FLAGS2[j + 1];
    }

    // Rewired to canonical Sprite_CorrectOamEntries port.
    fn sprite_correct_oam_entries_for_ganon(&mut self, k: usize, count: u8, mask: u8) {
        self.sprite_correct_oam_entries(k, count as i32, mask);
    }

    fn ganon_draw_emit_g9_overlay_for_ganon(&mut self, k: usize) {
        self.sprite_draw_multiple(k, &K_GANON_DMD, None);
    }

    fn sprite_draw_large_shadow_for_ganon(&mut self, k: usize, frame: usize) {
        let base = frame * 3;
        self.sprite_draw_multiple(k, &K_GANON_LARGE_SHADOW_DMD[base..base + 3], None);
    }

    fn set_oam_plain_at_for_ganon(
        &mut self,
        oam: usize,
        x: u8,
        y: u8,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.ram[oam] = x;
        self.ram[oam + 1] = y;
        self.ram[oam + 2] = charnum;
        self.ram[oam + 3] = flags;
        let ext_index = (oam - OAM_BUF) / 4;
        self.ram[BYTEWISE_EXTENDED_OAM + ext_index] = big;
    }

    // Rewired to canonical Sprite_Get16BitCoords port.
    fn sprite_get_16bit_coords_for_ganon(&mut self, k: usize) {
        self.sprite_get16_bit_coords(k);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> ZeldaState {
        ZeldaState::new()
    }

    #[test]
    fn attempt_trident_catch_matches_8x8_window() {
        // cur_sprite_(x,y) within +/- 4 of the target should report a catch.
        let mut s = fresh_state();
        write_le_u16(&mut s.ram, CUR_SPRITE_X, 0x100);
        write_le_u16(&mut s.ram, CUR_SPRITE_Y, 0x80);
        // Same coords -> 4 + (0) wraps into window — bool true.
        assert!(s.ganon_attempt_trident_catch(0x100, 0x80));
        // 7-unit X delta should still land within (uint16)(dx + 4) < 8 -> 4+(-7)=-3 wraps high -> false.
        assert!(!s.ganon_attempt_trident_catch(0x107, 0x80));
        // 3-unit positive delta -> (uint16)(0x100-0x103+4) = 1 < 8 -> true.
        assert!(s.ganon_attempt_trident_catch(0x103, 0x80));
    }

    #[test]
    fn enable_invincibility_only_triggers_on_hit_timer_26() {
        let mut s = fresh_state();
        let k = 2;
        // Hit timer must be exactly 26 in its low 7 bits.
        s.ram[SPRITE_HIT_TIMER_GANON + k] = 26;
        s.ganon_enable_invincibility(k);
        assert_eq!(s.ram[SPRITE_HIT_TIMER_GANON + k], 0);
        assert_eq!(s.ram[SPRITE_AI_STATE + k], 19);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + k], 127);
        assert_eq!(s.ram[SPRITE_TYPE + k], 215);

        let mut s2 = fresh_state();
        s2.ram[SPRITE_HIT_TIMER_GANON + k] = 27;
        s2.ganon_enable_invincibility(k);
        // Nothing should change.
        assert_eq!(s2.ram[SPRITE_HIT_TIMER_GANON + k], 27);
        assert_eq!(s2.ram[SPRITE_AI_STATE + k], 0);
        assert_eq!(s2.ram[SPRITE_TYPE + k], 0);

        let mut s3 = fresh_state();
        // Top bit set + low 7 bits == 26 -> still triggers.
        s3.ram[SPRITE_HIT_TIMER_GANON + k] = 26 | 0x80;
        s3.ganon_enable_invincibility(k);
        assert_eq!(s3.ram[SPRITE_HIT_TIMER_GANON + k], 0);
    }

    #[test]
    fn phase1_animate_trident_spin_indexes_into_func2_tables() {
        let mut s = fresh_state();
        let k = 0;
        // delay_main = 0 -> base = 0; sprite_D = 0 -> bonus = 0 -> j = 0.
        s.ram[SPRITE_DELAY_MAIN + k] = 0;
        s.ram[SPRITE_D + k] = 0;
        s.ganon_phase1_animate_trident_spin(k);
        assert_eq!(s.ram[SPRITE_G_GANON + k], K_GANON_G_FUNC2[0]); // 8
        assert_eq!(s.ram[SPRITE_GRAPHICS + k], K_GANON_GFX_FUNC2[0]); // 0

        // delay_main = 28 (>> 2 == 7, & 7 == 7); D = 1 -> bonus = 8 -> j = 15.
        s.ram[SPRITE_DELAY_MAIN + k] = 28;
        s.ram[SPRITE_D + k] = 1;
        s.ganon_phase1_animate_trident_spin(k);
        assert_eq!(s.ram[SPRITE_G_GANON + k], K_GANON_G_FUNC2[15]); // 1
        assert_eq!(s.ram[SPRITE_GRAPHICS + k], K_GANON_GFX_FUNC2[15]); // 9
    }

    #[test]
    fn handle_animation_idle_writes_g_and_gfx_per_direction() {
        let mut s = fresh_state();
        let k = 3;
        s.ram[SPRITE_D + k] = 0;
        s.ganon_handle_animation_idle(k);
        assert_eq!(s.ram[SPRITE_G_GANON + k], 9);
        assert_eq!(s.ram[SPRITE_GRAPHICS + k], 2);

        s.ram[SPRITE_D + k] = 1;
        s.ganon_handle_animation_idle(k);
        assert_eq!(s.ram[SPRITE_G_GANON + k], 10);
        assert_eq!(s.ram[SPRITE_GRAPHICS + k], 10);
    }

    #[test]
    fn shake_head_indexes_table_by_delay_main_shift3() {
        let mut s = fresh_state();
        let k = 1;
        // delay_main 24 -> idx 3 -> K_GANON_HEAD_DIR[3] = 1.
        s.ram[SPRITE_DELAY_MAIN + k] = 24;
        s.ganon_shake_head(k);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + k], 1);
        // delay_main 0 -> idx 0 -> 0.
        s.ram[SPRITE_DELAY_MAIN + k] = 0;
        s.ganon_shake_head(k);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + k], 0);
        // delay_main 32 -> idx 4 -> 2.
        s.ram[SPRITE_DELAY_MAIN + k] = 32;
        s.ganon_shake_head(k);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + k], 2);
    }

    #[test]
    fn select_warp_location_zeroes_velocities_and_sets_targets() {
        let mut s = fresh_state();
        let k = 0;
        // Seed sprite_subtype so the (rnd & 3 | subtype<<2) index is
        // deterministic enough for an assertion: with subtype = 0, the
        // resulting index is rnd & 3 only — that picks one of the first
        // four entries of K_GANON_NEXT_SUBTYPE (which are 4,5,6,7).
        s.ram[SPRITE_SUBTYPE + k] = 0;
        // Pre-clobber vels to ensure they get zeroed.
        s.ram[SPRITE_X_VEL + k] = 5;
        s.ram[SPRITE_Y_VEL + k] = 7;
        s.ganon_select_warp_location(k, 12);
        let j = s.ram[SPRITE_SUBTYPE + k];
        assert!((4..=7).contains(&j));
        assert_eq!(s.ram[SWAMOLA_TARGET_X_LO_GANON], K_GANON_NEXT_X[j as usize]);
        assert_eq!(s.ram[SWAMOLA_TARGET_Y_LO_GANON], K_GANON_NEXT_Y[j as usize]);
        assert_eq!(s.ram[SPRITE_AI_STATE + k], 12);
        assert_eq!(s.ram[SPRITE_X_VEL + k], 0);
        assert_eq!(s.ram[SPRITE_Y_VEL + k], 0);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + k], 48);
    }

    #[test]
    fn spawn_falling_tiles_increments_anim_clock_until_four() {
        let mut s = fresh_state();
        let k = 0;
        // Pre-clear overlord slot 7 so the search succeeds.
        s.ram[OVERLORD_TYPE + 7] = 0;
        s.ram[SPRITE_ANIM_CLOCK_GANON + k] = 0;
        // Seed link coords so we can verify the high-byte copy.
        write_le_u16(&mut s.ram, LINK_X_COORD, 0x0234);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x0588);
        s.ganon_spawn_falling_tiles_overlord(k);
        assert_eq!(s.ram[SPRITE_ANIM_CLOCK_GANON + k], 1);
        assert_eq!(s.ram[OVERLORD_TYPE + 7], K_GANON_OV_TYPE[0]);
        assert_eq!(s.ram[OVERLORD_X_LO_GANON + 7], K_GANON_OV_X[0]);
        assert_eq!(s.ram[OVERLORD_X_HI_GANON + 7], 0x02);
        assert_eq!(s.ram[OVERLORD_Y_HI_GANON + 7], 0x05);

        // Advance the anim clock past 3 and ensure no further write happens.
        s.ram[SPRITE_ANIM_CLOCK_GANON + k] = 4;
        let bak = s.ram[OVERLORD_TYPE + 7];
        s.ganon_spawn_falling_tiles_overlord(k);
        assert_eq!(s.ram[OVERLORD_TYPE + 7], bak);
        assert_eq!(s.ram[SPRITE_ANIM_CLOCK_GANON + k], 4);
    }

    #[test]
    fn handle_fire_bat_circle_writes_eight_overlords_and_seeds_counter() {
        let mut s = fresh_state();
        // Seed overlord_x_lo word to a known value so we can predict t for each i.
        write_le_u16(&mut s.ram, OVERLORD_X_LO_GANON, 0x10);
        s.ram[OVERLORD_X_LO_GANON + 2] = 0; // scale = 0 -> GanonSin returns 0.
                                            // Sprite 0 at (0x80, 0x80) for predictable add.
        s.sprite_set_x(0, 0x80);
        s.sprite_set_y(0, 0x80);
        // sprite_ai_state for indices 1..=8: leave them at 0 so the velocity
        // assignments fire for every i.
        for i in 1..=8 {
            s.ram[SPRITE_AI_STATE + i] = 0;
        }

        s.ganon_handle_fire_bat_circle(0);

        // overlord_x_lo word should have decremented by 4.
        assert_eq!(
            read_le_u16(&s.ram, OVERLORD_X_LO_GANON),
            0x10u16.wrapping_sub(4)
        );
        // tmp_counter is set to 8.
        assert_eq!(s.ram[TMP_COUNTER_GANON], 8);
        // With scale = 0, GanonSin -> 0, so every overlord_x_hi[i+1] == sprite_x_lo(0) == 0x80.
        for i in 0..8 {
            assert_eq!(s.ram[OVERLORD_X_HI_GANON + i + 1], 0x80);
            assert_eq!(s.ram[OVERLORD_GEN2_GANON + i + 1], 0x80);
        }
    }

    #[test]
    fn spawn_spiral_bat_initializes_dynamic_slot_fields() {
        let mut s = fresh_state();
        let k = 0;
        // Canonical Sprite_SpawnDynamicallyEx walks j_in (8) down to 0; the
        // highest free slot in [0..=8] wins. Ensure slot 8 is free so it
        // gets picked (matching the C entry-point behavior).
        s.ram[SPRITE_STATE + 8] = 0;
        write_le_u16(&mut s.ram, CUR_SPRITE_X, 0x40);
        write_le_u16(&mut s.ram, CUR_SPRITE_Y, 0x60);
        s.ganon_spawn_spiral_bat(k);
        let j = 8;
        assert_eq!(s.ram[SPRITE_STATE + j], 9);
        assert_eq!(s.ram[SPRITE_TYPE + j], 0xc9);
        assert_eq!(s.ram[SPRITE_ANIM_CLOCK_GANON + j], 4);
        assert_eq!(s.ram[SPRITE_OAM_FLAGS + j], 3);
        assert_eq!(s.ram[SPRITE_FLAGS3 + j], 0x40);
        assert_eq!(s.ram[SPRITE_FLAGS2 + j], 1);
        assert_eq!(s.ram[SPRITE_DEFL_BITS + j], 0x80);
        assert_eq!(s.ram[SPRITE_Y_HI + j], 128);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + j], 48);
        assert_eq!(s.ram[SPRITE_BUMP_DAMAGE_GANON + j], 7);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE_GANON + j], 7);
    }
}
