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
// garnish_countdown
const GARNISH_COUNTDOWN_GANON: usize = 0x1f90e;
// swamola_target_x_lo / y_lo
const SWAMOLA_TARGET_X_LO_GANON: usize = 0x1fd5c;
const SWAMOLA_TARGET_Y_LO_GANON: usize = 0x1fd68;
// Live Link position is accessed through RamPlayerStateView.

// ---------------------------------------------------------------------------
// Static tables shared across the Ganon handlers (sprite_main.c:402..464).
// ---------------------------------------------------------------------------

const GANON_SPIN_G_STATES: [u8; 16] = [8, 7, 6, 5, 4, 3, 2, 1, 8, 7, 6, 5, 4, 3, 2, 1];

// Sin-table used by GanonSin (sprite_main.c:338).
const GANON_SINE_LOOKUP_TABLE: [u16; 256] = [
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
const PHANTOM_GANON_DRAW_FRAMES: [(i8, i8, u16, u8); 16] = [
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
const GANON_FIRE_BAT_CIRCLE_X_COMPONENTS: [i8; 16] = [
    0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16,
];
const GANON_FIRE_BAT_CIRCLE_Y_COMPONENTS: [i8; 16] = [
    32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16, 0, 16, 24, 28,
];

// Ganon_SpawnFallingTilesOverlord (sprite_main.c:14989..14991).
const GANON_FALLING_TILE_OVERLORD_TYPES: [u8; 4] = [12, 13, 14, 15];
const GANON_FALLING_TILE_OVERLORD_X_LOW: [u8; 4] = [0x18, 0xd8, 0xd8, 0x18];
const GANON_FALLING_TILE_OVERLORD_Y_LOW: [u8; 4] = [0x28, 0x28, 0xd8, 0xd8];

// Ganon_Func1 (sprite_main.c:15023).
const GANON_FUNC1_16X16_Y_OFFSETS: [i8; 2] = [0, -16];

// Ganon_Phase1_AnimateTridentSpin (sprite_main.c:15036).
const GANON_TRIDENT_SPIN_GRAPHICS: [u8; 16] = [0, 0, 1, 1, 0, 0, 1, 1, 8, 8, 9, 9, 8, 8, 9, 9];

// Ganon_HandleAnimation_Idle (sprite_main.c:15045..15046).
const GANON_IDLE_G_STATES: [u8; 2] = [9, 10];
const GANON_IDLE_GRAPHICS: [u8; 2] = [2, 10];

// Ganon_SelectWarpLocation (sprite_main.c:15053..15058).
const GANON_WARP_SUBTYPES: [u8; 32] = [
    4, 5, 6, 7, 4, 5, 6, 7, 4, 5, 6, 7, 4, 5, 6, 7, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3,
];
const GANON_WARP_TARGET_Y_LOW: [u8; 8] = [0x40, 0x30, 0x30, 0x40, 0xb0, 0xc0, 0xc0, 0xb0];
const GANON_WARP_TARGET_X_LOW: [u8; 8] = [0x30, 0x50, 0xa0, 0xc0, 0x40, 0x60, 0x90, 0xb0];

// Ganon_ShakeHead (sprite_main.c:15070..15073).
const GANON_SHAKE_HEAD_DIRECTIONS: [u8; 18] =
    [0, 0, 0, 1, 2, 2, 2, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 16];

// Ganon_Draw (sprite_main.c:402..464, 15077..15130).
const GANON_DRAW_X_OFFSETS: [i8; 204] = [
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
const GANON_DRAW_Y_OFFSETS: [i8; 204] = [
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
const GANON_DRAW_CHARS: [u8; 204] = [
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
const GANON_DRAW_FLAGS: [u8; 204] = [
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
const GANON_DRAW_PATCH_CHARS: [u8; 12] = [
    0x40, 0x42, 0, 0, 0x42, 0x40, 0x82, 0x80, 0xa0, 0xa0, 0x80, 0x82,
];
const GANON_DRAW_PATCH_FLAGS: [u8; 12] = [0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0, 0];
const GANON_DRAW_OAM_START_OFFSETS: [u8; 17] =
    [1, 1, 1, 1, 1, 1, 15, 1, 4, 4, 4, 4, 4, 4, 4, 15, 15];
const GANON_DRAW_FRAMES: [DrawMultipleData; 2] = [
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
const GANON_LARGE_SHADOW_DRAW_FRAMES: [DrawMultipleData; 15] = [
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
    let t = ganon_mult(GANON_SINE_LOOKUP_TABLE[(a & 0xff) as usize], b);
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
            .is_none_or(|frame| frame == self.game_state.frame.frame_counter)
    }

    // void SwishEvery16Frames(int k) {  // 9d8aa9
    pub(super) fn swish_every16_frames(&mut self, k: usize) {
        if (self.game_state.frame.frame_counter & 15) == 0 {
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
        self.sprite_slot_mut(k).decrement_subtype2();
        let j = ((self.sprite_slot(k).subtype2() >> 2) & 7) as usize;
        self.sprite_slot_mut(k).set_g(GANON_SPIN_G_STATES[j]);
        if self.sprite_slot(k).delay_main() != 0 {
            if (self.sprite_slot(k).delay_main() & 1) != 0 {
                return;
            }
            let pt = self.sprite_project_speed_towards_link(k, 32);
            self.sprite_approach_target_speed(k, pt.x, pt.y);
        } else {
            let x =
                self.sprite_get_x(0)
                    .wrapping_add_signed(if self.sprite_slot(0).direction() != 0 {
                        -16
                    } else {
                        24
                    });
            let y = self.sprite_get_y(0).wrapping_sub(16);
            if self.ganon_attempt_trident_catch(x, y) {
                self.sprite_slot_mut(k).set_state(0);
                self.sprite_slot_mut(0).set_ai_state(3);
                self.sprite_slot_mut(0).set_delay_main(16);
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
        let x = (u16::from(self.sprite_slot(k).b()) << 8) | u16::from(self.sprite_slot(k).a());
        let y = (u16::from(self.sprite_slot(k).e()) << 8) | u16::from(self.sprite_slot(k).c());
        let pt = self.sprite_project_speed_towards_location(k, x, y, 2);
        let pt2 = self.sprite_project_speed_towards_location(k, x, y, 80);
        self.sprite_slot_mut(k)
            .set_x_velocity(pt2.y.wrapping_sub(pt.x));
        self.sprite_slot_mut(k)
            .set_y_velocity(0u8.wrapping_sub(pt2.x).wrapping_sub(pt.y));
        self.fire_bat_move(k);
    }

    // void Sprite_FireBat_Launched(int k) {  // 9d8bd7
    pub(super) fn sprite_fire_bat_launched(&mut self, k: usize) {
        self.fire_bat_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_link(k);
        match self.sprite_slot(k).ai_state() {
            0 => {
                self.get_position_relative_to_the_great_overlord_ganon(k);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                } else {
                    let graphics = (self.sprite_slot(k).delay_main() >> 2) & 1;
                    self.sprite_slot_mut(k).set_graphics(graphics);
                }
            }
            1 => {
                self.get_position_relative_to_the_great_overlord_ganon(k);
                self.sprite_slot_mut(k).increment_subtype2();
                let graphics = (self.sprite_slot(k).subtype2() >> 2) & 1;
                self.sprite_slot_mut(k).set_graphics(graphics);
            }
            2 => {
                self.sprite_move_xy(k);
                self.sprite_slot_mut(k).set_deflection_bits(64);
                if self.sprite_slot(k).delay_aux1() == 0 {
                    if self.sprite_slot(k).delay_main() == 0 {
                        self.fire_bat_animate(k);
                        self.fire_bat_animate(k);
                    } else {
                        let mut t = self.sprite_slot(k).delay_main().wrapping_sub(1);
                        if t == 0 {
                            t = 35;
                            self.sprite_slot_mut(k).set_delay_aux1(t);
                        }
                        self.sprite_slot_mut(k).set_graphics((t >> 2) & 1);
                    }
                } else if self.sprite_slot(k).delay_aux1() == 1 {
                    self.sprite_apply_speed_towards_link_for_ganon(k, 48);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x1e);
                    self.fire_bat_animate(k);
                    self.fire_bat_animate(k);
                } else {
                    const SECONDARY_GRAPHICS: [u8; 9] = [4, 4, 4, 3, 3, 3, 2, 2, 2];
                    let graphics =
                        SECONDARY_GRAPHICS[(self.sprite_slot(k).delay_aux1() >> 2) as usize];
                    self.sprite_slot_mut(k).set_graphics(graphics);
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
                self.game_state.frame.frame_counter,
                k,
                self.sprite_slot(k).ai_state(),
                self.sprite_slot(k).delay_main(),
                self.sprite_slot(k).health(),
                self.sprite_slot(k).subtype(),
                self.sprite_slot(k).direction(),
                self.sprite_slot(k).hit_timer(),
                self.sprite_slot(k).delay_aux1(),
                self.sprite_slot(k).delay_aux2(),
                self.sprite_slot(k).delay_aux4(),
                self.sprite_get_x(k),
                self.sprite_get_y(k),
            );
        }
        if sign8(self.sprite_slot(k).ai_state()) {
            if self.sprite_return_if_inactive(k) {
                return;
            }
            if self.sprite_slot(k).delay_main() == 0 {
                self.sprite_slot_mut(k).set_state(0);
            }
            if (self.sprite_slot(k).delay_main() & 1) == 0 {
                self.ganon_draw(k);
            }
            return;
        }

        if self.sprite_slot(k).delay_aux4() != 0 {
            const GFXB: [u8; 2] = [16, 10];
            let graphics = GFXB[(self.sprite_slot(k).direction() & 1) as usize];
            self.sprite_slot_mut(k).set_graphics(graphics);
        }

        if self.game_state.dungeon.torch.ganon_torch_count() == 2
            && self.game_state.dungeon.torch.ganon_torch_count() != self.sprite_slot(k).room()
        {
            self.sprite_slot_mut(k).set_delay_aux1(64);
        }
        let torch_count = self.game_state.dungeon.torch.ganon_torch_count();
        self.sprite_slot_mut(k).set_room(torch_count);

        self.ganon_draw(k);
        if self.sprite_slot(k).delay_aux1() != 0 {
            self.sprite_slot_mut(k).set_graphics(15);
            self.ganon_enable_invincibility(k);
            self.sprite_check_damage_to_and_from_link(k);
            return;
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }

        if self.sprite_slot(k).delay_aux2() == 1 {
            self.ganon_extinguish_torch_for_ganon();
        } else if self.sprite_slot(k).delay_aux2() == 16 {
            self.ganon_extinguish_torch_adjust_translucency_for_ganon();
        }

        let pair = self.sprite_is_right_of_link(k);
        let head_direction = if pair.b.wrapping_add(32) < 64 {
            1
        } else if pair.a != 0 {
            0
        } else {
            2
        };
        self.sprite_slot_mut(k).set_head_direction(head_direction);

        if self.sprite_slot(k).delay_aux4() != 0 {
            let delay_aux4 = self.sprite_slot(k).delay_aux4();
            self.sprite_slot_mut(k).set_ignore_projectile(delay_aux4);
            if self.sprite_return_if_recoiling(k) {
                return;
            }
            self.sprite_slot_mut(k).set_delay_main(0);
            return;
        }

        if (self.sprite_slot(k).ignore_projectile()
            | self.game_state.player.follower_link.immobilized_flag())
            == 0
            && self.game_state.dungeon.torch.ganon_torch_count() == 2
        {
            self.sprite_check_damage_to_and_from_link(k);
        }
        self.sprite_slot_mut(k).set_ignore_projectile(0);

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.sprite_slot_mut(k).set_delay_main(128);
                } else if self.sprite_slot(k).delay_main() == 32 {
                    self.set_music_control(0x1f);
                } else if self.sprite_slot(k).delay_main() == 64 {
                    self.dialogue_message_index_mut().set_value(0x16f);
                    self.sprite_show_message_minimal_c();
                }
            }
            1 => {
                if self.sprite_slot(k).health() < 209 {
                    self.sprite_slot_mut(k).set_health(208);
                }
                if self.sprite_slot(k).delay_main() < 64 {
                    if self.sprite_slot(k).delay_main() == 0 {
                        self.ganon_select_warp_location(k, 5);
                    } else {
                        const GFX1: [u8; 2] = [2, 10];
                        let graphics = GFX1[(self.sprite_slot(k).direction() & 1) as usize];
                        self.sprite_slot_mut(k).set_graphics(graphics);
                    }
                } else if self.sprite_slot(k).delay_main() != 64 {
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
                    self.sprite_slot_mut(k).set_g(0);
                    let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0xc9, &mut info);
                    assert!(
                        j >= 0,
                        "Sprite_D6_Ganon expected phase-1 trident spawn to succeed"
                    );
                    let j = j as usize;
                    let i = usize::from(self.sprite_slot(k).direction() & 1);
                    self.sprite_set_x(j, info.r0_x.wrapping_add_signed(i16::from(X1[i])));
                    self.sprite_set_y(j, info.r2_y.wrapping_add_signed(i16::from(Y1[i])));
                    self.sprite_apply_speed_towards_link_for_ganon(k, 31);
                    let angle = Self::sprite_convert_velocity_to_angle(
                        self.sprite_slot(k).x_velocity(),
                        self.sprite_slot(k).y_velocity(),
                    );
                    let vi = usize::from(angle.wrapping_sub(2) & 0x0f);
                    self.sprite_slot_mut(j).set_x_velocity(XVEL1[vi] as u8);
                    self.sprite_slot_mut(j).set_y_velocity(YVEL1[vi] as u8);
                    self.sprite_slot_mut(j).set_delay_main(112);
                    self.sprite_slot_mut(j).set_anim_clock(2);
                    self.sprite_slot_mut(j).set_oam_flags(1);
                    self.sprite_slot_mut(j).set_flags2(4);
                    self.sprite_slot_mut(j).set_deflection_bits(0x84);
                    self.sprite_slot_mut(j).set_direction(2);
                    self.sprite_slot_mut(j).set_bump_damage(7);
                    self.sprite_slot_mut(j).set_ignore_projectile(7);
                }
            }
            2 => {
                if self.sprite_slot(k).health() < 209 {
                    self.sprite_slot_mut(k).set_health(208);
                }
                const SECONDARY_GRAPHICS: [u8; 2] = [0, 8];
                let graphics = SECONDARY_GRAPHICS[(self.sprite_slot(k).direction() & 1) as usize];
                self.sprite_slot_mut(k).set_graphics(graphics);
                if self.sprite_slot(k).delay_main() != 0 {
                    self.sprite_slot_mut(k).increment_ignore_projectile();
                    if (self.sprite_slot(k).delay_main() & 1) != 0 {
                        self.sprite_slot_mut(k).set_graphics(255);
                    }
                }
            }
            3 => {
                if self.sprite_slot(k).health() < 209 {
                    self.sprite_slot_mut(k).set_health(208);
                }
                if self.sprite_slot(k).delay_main() != 0 {
                    self.ganon_phase1_animate_trident_spin(k);
                } else {
                    self.sprite_slot_mut(k).set_ai_state(6);
                    self.sprite_slot_mut(k).set_delay_main(127);
                    self.ganon_handle_animation_idle(k);
                }
            }
            4 => {
                if self.sprite_slot(k).health() < 209 {
                    self.sprite_slot_mut(k).set_health(208);
                }
                if self.sprite_slot(k).delay_main() != 0 {
                    self.ganon_shake_head(k);
                } else {
                    self.ganon_select_warp_location(k, 5);
                }
            }
            5 | 10 | 13 | 18 => {
                if self.sprite_slot(k).ai_state() == 13 {
                    self.sprite_slot_mut(k).set_health(100);
                }
                self.sprite_slot_mut(k).increment_ignore_projectile();
                let x = (u16::from(self.sprite_slot(k).x_high()) << 8)
                    | u16::from(
                        self.game_state
                            .effects
                            .sprite_histories
                            .swamola_target(0)
                            .x_low(),
                    );
                let y = (u16::from(self.sprite_slot(k).y_high()) << 8)
                    | u16::from(
                        self.game_state
                            .effects
                            .sprite_histories
                            .swamola_target(0)
                            .y_low(),
                    );
                if self.ganon_attempt_trident_catch(x, y) {
                    let direction = self.sprite_slot(k).subtype() >> 2;
                    self.sprite_slot_mut(k).set_direction(direction);
                    if self.sprite_slot(k).ai_state() == 5 {
                        self.sprite_slot_mut(k).set_ai_state(2);
                        self.sprite_slot_mut(k).set_delay_main(32);
                    } else if self.sprite_slot(k).health() >= 161 {
                        self.sprite_slot_mut(k).set_ai_state(11);
                        self.sprite_slot_mut(k).set_delay_main(40);
                    } else if self.sprite_slot(k).health() >= 97 {
                        self.sprite_slot_mut(k).set_ai_state(14);
                        self.sprite_slot_mut(k).set_delay_main(40);
                    } else {
                        self.sprite_slot_mut(k).set_ai_state(17);
                        self.sprite_slot_mut(k).set_delay_main(104);
                    }
                } else {
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 32);
                    self.sprite_approach_target_speed(k, pt.x, pt.y);
                    self.sprite_move_xy(k);
                    if self.sprite_slot(k).delay_main() == 0
                        || (self.game_state.frame.frame_counter & 1) != 0
                    {
                        self.sprite_slot_mut(k).set_graphics(255);
                        return;
                    }
                    const GFX5: [u8; 2] = [2, 10];
                    let graphics = GFX5[(self.sprite_slot(k).direction() & 1) as usize];
                    self.sprite_slot_mut(k).set_graphics(graphics);
                    if (self.game_state.frame.frame_counter & 7) == 0 {
                        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
                        let j = self.sprite_spawn_dynamically(k, 0xd6, &mut info);
                        if j >= 0 {
                            let j = j as usize;
                            self.sprite_set_spawned_coordinates(j, &info);
                            self.sprite_slot_mut(j).set_ignore_projectile(24);
                            self.sprite_slot_mut(j).set_delay_main(24);
                            self.sprite_slot_mut(j).set_ai_state(255);
                            let graphics = self.sprite_slot(k).graphics();
                            let head_direction = self.sprite_slot(k).head_direction();
                            self.sprite_slot_mut(j).set_graphics(graphics);
                            self.sprite_slot_mut(j).set_head_direction(head_direction);
                        }
                    }
                }
            }
            6 => {
                if self.sprite_slot(k).health() < 209 {
                    self.sprite_slot_mut(k).set_health(208);
                }
                if self.sprite_slot(k).delay_main() == 0 {
                    if self.sprite_slot(k).health() >= 209 {
                        self.sprite_slot_mut(k).set_ai_state(1);
                        self.sprite_slot_mut(k).set_delay_main(128);
                    } else {
                        self.sprite_slot_mut(k).set_delay_main(255);
                        self.sprite_slot_mut(k).set_ai_state(7);
                    }
                } else {
                    self.ganon_shake_head(k);
                }
            }
            7 => {
                if self.sprite_slot(k).health() < 161 {
                    self.sprite_slot_mut(k).set_health(160);
                }
                self.game_state
                    .sprites
                    .overlord_slots
                    .slot_mut(&mut self.ram, 2)
                    .set_x_low(40);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(8);
                    self.sprite_slot_mut(k).set_delay_main(255);
                } else {
                    if self.sprite_slot(k).delay_main() < 0xc0
                        && (self.sprite_slot(k).delay_main() & 0x0f) == 0
                    {
                        self.ganon_spawn_spiral_bat(k);
                    }
                    self.ganon_phase1_animate_trident_spin(k);
                    self.ganon_handle_fire_bat_circle(k);
                }
            }
            8 => {
                const BAT_ORBIT_RADIUS_DELTAS: [i8; 16] =
                    [0, 0, 0, 0, -1, -1, -2, -1, 0, 0, 0, 0, 1, 2, 1, 1];
                const DELAY8: [u8; 8] = [0x10, 0x30, 0x50, 0x70, 0x90, 0xb0, 0xd0, 0xbd];
                if self.sprite_slot(k).health() < 161 {
                    self.sprite_slot_mut(k).set_health(160);
                }
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(9);
                    self.sprite_slot_mut(k).set_delay_main(127);
                    self.ganon_handle_animation_idle(k);
                    for j in (1..=8usize).rev() {
                        self.sprite_slot_mut(j).set_ai_state(2);
                        self.sprite_slot_mut(j).set_delay_main(DELAY8[j - 1]);
                    }
                } else {
                    let idx = ((self.sprite_slot(k).delay_main() >> 4) & 15) as usize;
                    self.game_state
                        .sprites
                        .overlord_slots
                        .slot_mut(&mut self.ram, 2)
                        .add_x_low(BAT_ORBIT_RADIUS_DELTAS[idx] as u8);
                    self.ganon_phase1_animate_trident_spin(k);
                    self.ganon_handle_fire_bat_circle(k);
                }
            }
            9 => {
                if self.sprite_slot(k).health() < 161 {
                    self.sprite_slot_mut(k).set_health(160);
                }
                if self.sprite_slot(k).delay_main() == 0 {
                    self.ganon_select_warp_location(k, 10);
                } else {
                    self.ganon_shake_head(k);
                }
            }
            11 => {
                self.sprite_slot_mut(k).increment_ignore_projectile();
                self.ganon_handle_animation_idle(k);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_delay_main(255);
                    self.sprite_slot_mut(k).set_ai_state(7);
                } else if (self.sprite_slot(k).delay_main() & 1) != 0 {
                    self.sprite_slot_mut(k).set_graphics(255);
                }
            }
            12 => {
                let j = self.sprite_slot(k).delay_main();
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
                if self.sprite_slot(k).direction() != 0 {
                    t += 3;
                }
                const GFX12: [u8; 6] = [5, 6, 7, 13, 14, 10];
                self.sprite_slot_mut(k).set_graphics(GFX12[t]);
                if (self.sprite_slot(k).hit_timer() & 127) == 1 {
                    self.sprite_slot_mut(k).set_ai_state(15);
                    self.sprite_slot_mut(k).set_z_velocity(24);
                    self.sprite_slot_mut(k).set_delay_main(0);
                }
            }
            14 => {
                self.sprite_slot_mut(k).increment_ignore_projectile();
                self.ganon_handle_animation_idle(k);
                self.sprite_slot_mut(k).set_g(0);
                if self.sprite_slot(k).delay_main() == 0 {
                    if (self.get_random_number() & 1) != 0 {
                        self.ganon_select_warp_location(k, 13);
                    } else {
                        self.sprite_slot_mut(k).set_delay_main(127);
                        self.sprite_slot_mut(k).set_ai_state(12);
                    }
                } else if (self.sprite_slot(k).delay_main() & 1) != 0 {
                    self.sprite_slot_mut(k).set_graphics(255);
                }
            }
            15 => {
                const GFX15: [u8; 2] = [6, 14];
                if self.sprite_slot(k).delay_main() != 0 {
                    if self.sprite_slot(k).delay_main() == 1 {
                        self.sprite_slot_mut(k).set_ai_state(16);
                        self.sprite_slot_mut(k).set_z_velocity(160);
                        return;
                    }
                } else {
                    self.sprite_move_z(k);
                    self.sprite_slot_mut(k).subtract_z_velocity(1);
                    if self.sprite_slot(k).z_velocity() == 0 {
                        self.sprite_slot_mut(k).set_delay_main(32);
                    }
                }
                let graphics = GFX15[(self.sprite_slot(k).direction() & 1) as usize];
                self.sprite_slot_mut(k).set_graphics(graphics);
            }
            16 => {
                self.set_bg1_y_offset(0);
                if self.sprite_slot(k).delay_main() != 0 {
                    if self.sprite_slot(k).delay_main() == 1 {
                        self.set_ambient_sound_effect(5);
                        self.ganon_select_warp_location(k, 13);
                        self.follower_link_state_mut().clear_immobilized();
                        self.ganon_spawn_falling_tiles_overlord(k);
                        if self.sprite_slot(k).anim_clock() >= 4 {
                            self.ganon_select_warp_location(k, 10);
                            self.sprite_slot_mut(k).set_health(96);
                            self.sprite_slot_mut(k).set_delay_aux2(224);
                            self.dialogue_message_index_mut().set_value(0x170);
                            self.sprite_show_message_minimal_c();
                        }
                    } else {
                        let offs: u16 = if ((self.sprite_slot(k).delay_main() - 1) & 1) != 0 {
                            (-1i16) as u16
                        } else {
                            1
                        };
                        self.set_bg1_y_offset(offs);
                        self.follower_link_state_mut().immobilize();
                    }
                } else {
                    const GFX16: [u8; 2] = [2, 10];
                    self.sprite_move_z(k);
                    if sign8(self.sprite_slot(k).z()) {
                        self.sprite_slot_mut(k).set_z_velocity(0);
                        self.sprite_slot_mut(k).set_z(0);
                        self.sprite_slot_mut(k).set_delay_main(96);
                        self.set_ambient_sound_effect(7);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
                    }
                    let graphics = GFX16[(self.sprite_slot(k).direction() & 1) as usize];
                    self.sprite_slot_mut(k).set_graphics(graphics);
                }
            }
            17 => {
                const GFX17B: [u8; 2] = [6, 14];
                const GFX17: [u8; 2] = [7, 10];
                let graphics = GFX17B[(self.sprite_slot(k).direction() & 1) as usize];
                self.sprite_slot_mut(k).set_graphics(graphics);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.ganon_select_warp_location(k, 0x12);
                    return;
                } else if self.sprite_slot(k).delay_main() == 52 {
                    self.ganon_func1(k, 5);
                } else if self.sprite_slot(k).delay_main() < 52 {
                    let graphics = GFX17[(self.sprite_slot(k).direction() & 1) as usize];
                    self.sprite_slot_mut(k).set_graphics(graphics);
                }
                if self.sprite_slot(k).delay_main() >= 72 || self.sprite_slot(k).delay_main() < 40 {
                    self.sprite_slot_mut(k).increment_ignore_projectile();
                    if (self.sprite_slot(k).delay_main() & 1) != 0 {
                        self.sprite_slot_mut(k).set_graphics(255);
                    }
                }
                self.ganon_enable_invincibility(k);
            }
            19 => {
                self.sprite_slot_mut(k).set_oam_flags(5);
                self.sprite_slot_mut(k).set_flags(2);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_oam_flags(1);
                    self.ganon_select_warp_location(k, 18);
                    self.sprite_slot_mut(k).set_sprite_type(0xd6);
                    self.sprite_slot_mut(k).set_hit_timer(0);
                } else {
                    const GFX19: [u8; 2] = [5, 13];
                    let graphics = GFX19[(self.sprite_slot(k).direction() & 1) as usize];
                    self.sprite_slot_mut(k).set_graphics(graphics);
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
        self.oam_state_mut().set_current_pointer(0x950);
        self.oam_state_mut().set_current_extended_pointer(0xa74);
        let g = self.sprite_slot(k).graphics() as usize;
        // Sprite_DrawMultiple emits 8 OAM entries starting at index g*8.
        self.sprite_draw_multiple_for_ganon(k, &PHANTOM_GANON_DRAW_FRAMES, g * 8, 8);
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
        self.sprite_slot_mut(j).set_flags2(2);
        self.sprite_slot_mut(j).set_ignore_projectile(2);
        self.sprite_slot_mut(j).set_anim_clock(1);
        self.sprite_slot_mut(j).set_oam_flags(0);
    }

    // void Sprite_PhantomGanon(int k) {  // 9d88bc
    pub(super) fn sprite_phantom_ganon(&mut self, k: usize) {
        const LOCAL_GRAPHICS: [u8; 4] = [0, 1, 2, 1];
        const TARGET_XVEL: [u8; 2] = [32, (-32i8) as u8];
        const TARGET_YVEL: [u8; 2] = [16, (-16i8) as u8];

        if self.sprite_slot(k).ai_state() == 0 {
            self.phantom_ganon_draw(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_move_y(k);
            self.sprite_slot_mut(k).increment_subtype2();
            if (self.sprite_slot(k).subtype2() & 31) == 0 {
                let y_velocity = self.sprite_slot(k).y_velocity().wrapping_sub(1);
                self.sprite_slot_mut(k).set_y_velocity(y_velocity);
                if self.sprite_slot(k).y_velocity() == 252 {
                    let j = self.spawn_boss_poof(k);
                    assert!(
                        j >= 0,
                        "Sprite_PhantomGanon expected SpawnBossPoof to succeed"
                    );
                    let j = j as usize;
                    let y = self.sprite_get_y(j).wrapping_sub(20);
                    self.sprite_set_y(j, y);
                } else if self.sprite_slot(k).y_velocity() == 251 {
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).set_delay_main(255);
                    self.sprite_slot_mut(k).set_y_velocity((-4i8) as u8);
                }
            }
        } else {
            self.ganon_bat_draw(k);
            if self.sprite_slot(k).pause() != 0 {
                self.sprite_slot_mut(k).set_state(0);
                let bits = self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x8000;
                self.dungeon_savegame_state_mut()
                    .set_savegame_state_bits(bits);
            }
            if self.sprite_return_if_inactive(k) {
                return;
            }
            let graphics =
                LOCAL_GRAPHICS[usize::from((self.game_state.frame.frame_counter >> 2) & 3)];
            self.sprite_slot_mut(k).set_graphics(graphics);
            if self.sprite_slot(k).delay_main() != 0 {
                if self.sprite_slot(k).delay_main() < 208 {
                    let j = usize::from(self.sprite_slot(k).head_direction() & 1);
                    let y_velocity = self.sprite_slot(k).y_velocity().wrapping_add(if j != 0 {
                        0xff
                    } else {
                        1
                    });
                    self.sprite_slot_mut(k).set_y_velocity(y_velocity);
                    if self.sprite_slot(k).y_velocity() == TARGET_YVEL[j] {
                        let head_direction = self.sprite_slot(k).head_direction().wrapping_add(1);
                        self.sprite_slot_mut(k).set_head_direction(head_direction);
                    }
                    let j = usize::from(self.sprite_slot(k).direction() & 1);
                    let x_velocity = self.sprite_slot(k).x_velocity().wrapping_add(if j != 0 {
                        0xff
                    } else {
                        1
                    });
                    self.sprite_slot_mut(k).set_x_velocity(x_velocity);
                    if self.sprite_slot(k).x_velocity() == TARGET_XVEL[j] {
                        self.sprite_slot_mut(k).increment_direction();
                    }
                    if self.sprite_slot(k).x_velocity() == 0 {
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x1e);
                    }
                }
                let x = self.game_state.player.follower_link.x() & 0xff00 | 0x78;
                let y = self.game_state.player.follower_link.y() & 0xff00 | 0x50;
                let pt = self.sprite_project_speed_towards_location(k, x, y, 5);
                let xvel = self.sprite_slot(k).x_velocity();
                let yvel = self.sprite_slot(k).y_velocity();
                self.sprite_slot_mut(k)
                    .set_x_velocity(xvel.wrapping_add(pt.x));
                self.sprite_slot_mut(k)
                    .set_y_velocity(yvel.wrapping_add(pt.y));
                self.sprite_move_xy(k);
                self.sprite_slot_mut(k).set_x_velocity(xvel);
                self.sprite_slot_mut(k).set_y_velocity(yvel);
            } else {
                self.sprite_move_xy(k);
                if self.sprite_slot(k).x_velocity() != 64 {
                    self.sprite_slot_mut(k).add_x_velocity(1);
                    self.sprite_slot_mut(k).add_y_velocity((-1i8) as u8);
                }
            }
        }
    }

    // bool Ganon_AttemptTridentCatch(uint16 x, uint16 y) {  // sprite_main.c:14554
    //   return (uint16)(cur_sprite_x - x + 4) < 8 && (uint16)(cur_sprite_y - y + 4) < 8;
    // }
    pub(super) fn ganon_attempt_trident_catch(&self, x: u16, y: u16) -> bool {
        let cx = self.game_state.sprites.workspace.current_sprite_x();
        let cy = self.game_state.sprites.workspace.current_sprite_y();
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
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, 0)
            .subtract_adjacent_x_low_word(4);

        let scale = self.game_state.sprites.overlord_slots.slot(2).x_low();
        let sprite0_x = self.sprite_get_x(0);
        let sprite0_y = self.sprite_get_y(0);

        for i in 0..8usize {
            let base = self
                .game_state
                .sprites
                .overlord_slots
                .slot(0)
                .adjacent_x_low_word();
            let t: u16 = base.wrapping_add((i as u16).wrapping_mul(64)) & 0x1ff;
            if self.sprite_slot(i + 1).ai_state() != 2 {
                let j = ((t >> 5).wrapping_sub(4) & 0xf) as usize;
                // (int8)kGanonMath_X[j] >> 2 — arithmetic shift on signed.
                self.sprite_slot_mut(i + 1)
                    .set_x_velocity(((GANON_FIRE_BAT_CIRCLE_X_COMPONENTS[j] as i8) >> 2) as u8);
                self.sprite_slot_mut(i + 1)
                    .set_y_velocity(((GANON_FIRE_BAT_CIRCLE_Y_COMPONENTS[j] as i8) >> 2) as u8);
            }
            // x = Sprite_GetX(0) + (int8)GanonSin(t, overlord_x_lo[2])
            // i32 to allow the negative-extend before re-casting to 16-bit / 8-bit.
            let xs = ganon_sin(t, scale) as i16;
            let x = (sprite0_x as i32).wrapping_add(xs as i32);
            self.game_state
                .sprites
                .overlord_slots
                .slot_mut(&mut self.ram, i + 1)
                .set_circle_x(x as u16);

            let ys = ganon_sin(t.wrapping_add(0x80), scale) as i16;
            let y = (sprite0_y as i32).wrapping_add(ys as i32);
            self.game_state
                .sprites
                .overlord_slots
                .slot_mut(&mut self.ram, i + 1)
                .set_circle_y(y as u16);
        }
        self.temp_counter_mut().set(8);
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
            self.sprite_slot_mut(j).set_anim_clock(4);
            self.sprite_slot_mut(j).set_oam_flags(3);
            self.sprite_slot_mut(j).set_flags3(0x40);
            self.sprite_slot_mut(j).set_flags2(1);
            self.sprite_slot_mut(j).set_deflection_bits(0x80);
            self.sprite_slot_mut(j).set_y_high(128);
            self.sprite_slot_mut(j).set_delay_main(48);
            self.sprite_slot_mut(j).set_bump_damage(7);
            self.sprite_slot_mut(j).set_ignore_projectile(7);
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
        if (self.sprite_slot(k).hit_timer() & 127) == 26 {
            self.sprite_slot_mut(k).set_hit_timer(0);
            self.sprite_slot_mut(k).set_ai_state(19);
            self.sprite_slot_mut(k).set_delay_main(127);
            self.sprite_slot_mut(k).set_sprite_type(215);
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
        // The C loop keeps walking down past slot 0. The accessor layer keeps
        // the modernized port inside the real overlord slot range.
        let mut j_i32: i32 = 7;
        while j_i32 >= 0
            && self
                .game_state
                .sprites
                .overlord_slots
                .slot(j_i32 as usize)
                .overlord_type()
                != 0
        {
            j_i32 -= 1;
        }
        if j_i32 < 0 {
            return;
        }

        let t = self.sprite_slot(k).anim_clock();
        if t >= 4 {
            return;
        }
        self.sprite_slot_mut(k).set_anim_clock(t.wrapping_add(1));

        let j = j_i32 as usize;
        let ti = t as usize;
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, j)
            .set_overlord_type(GANON_FALLING_TILE_OVERLORD_TYPES[ti]);
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, j)
            .set_x_low(GANON_FALLING_TILE_OVERLORD_X_LOW[ti]);
        // overlord_x_hi[j] = link_x_coord >> 8  — read the high byte of the 16-bit link_x_coord.
        let x_high = self.game_state.player.follower_link.x_high();
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, j)
            .set_x_high(x_high);
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, j)
            .set_y_low(GANON_FALLING_TILE_OVERLORD_Y_LOW[ti]);
        let y_high = self.game_state.player.follower_link.y_high();
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, j)
            .set_y_high(y_high);
        // overlord_gen1 / overlord_gen2 — clear both.
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, j)
            .set_gen1(0);
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, j)
            .set_gen2(0);
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
        self.temp_counter_mut().set(t);
        if let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_ex_for_ganon(k, 0xC9, 8) {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x2a);
            self.sprite_set_spawned_coordinates_for_ganon(j, r0_x, r2_y);
            self.sprite_slot_mut(j).set_ignore_projectile(t);
            self.sprite_slot_mut(j).set_anim_clock(t);
            self.sprite_slot_mut(j).set_oam_flags(3);
            self.sprite_slot_mut(j).set_flags3(0x40);
            self.sprite_slot_mut(j).set_flags2(0x21);
            self.sprite_slot_mut(j).set_deflection_bits(0x40);
            let d = self.sprite_slot(k).direction() as usize;
            let y = r2_y.wrapping_add(GANON_FUNC1_16X16_Y_OFFSETS[d] as i16 as u16);
            self.sprite_set_y(j, y);
            self.sprite_apply_speed_towards_link_for_ganon(j, 32);
            self.sprite_slot_mut(j).set_delay_main(16);
            let sprite0 = self.sprite_slot(0);
            let x_low = sprite0.x_low();
            let x_high = sprite0.x_high();
            let y_low = sprite0.y_low();
            let y_high = sprite0.y_high();
            let mut sprite = self.sprite_slot_mut(j);
            sprite.set_a(x_low);
            sprite.set_b(x_high);
            sprite.set_c(y_low);
            sprite.set_e(y_high);
            self.sprite_slot_mut(j).set_bump_damage(7);
            self.sprite_slot_mut(j).set_ignore_projectile(7);
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
        let base = ((self.sprite_slot(k).delay_main() >> 2) & 7) as usize;
        let bonus = if self.sprite_slot(k).direction() != 0 {
            8
        } else {
            0
        };
        let j = base + bonus;
        self.sprite_slot_mut(k).set_g(GANON_SPIN_G_STATES[j]);
        self.sprite_slot_mut(k)
            .set_graphics(GANON_TRIDENT_SPIN_GRAPHICS[j]);
        // SwishEvery16Frames(k) — inline-ported (sprite_main.c:14416).
        // void SwishEvery16Frames(int k) {
        //   if (!(frame_counter & 15))
        //     SpriteSfx_QueueSfx3WithPan(k, 0x6);
        // }
        if (self.game_state.frame.frame_counter & 15) == 0 {
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
        let d = self.sprite_slot(k).direction() as usize;
        self.sprite_slot_mut(k).set_g(GANON_IDLE_G_STATES[d]);
        self.sprite_slot_mut(k).set_graphics(GANON_IDLE_GRAPHICS[d]);
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
                self.game_state.frame.frame_counter,
                k,
                a,
                self.sprite_slot(k).ai_state(),
                self.sprite_slot(k).delay_main(),
                self.sprite_slot(k).health(),
                self.sprite_slot(k).subtype(),
                self.sprite_slot(k).direction(),
                self.game_state.effects.sprite_histories.swamola_target(0).x_low(),
                self.game_state.effects.sprite_histories.swamola_target(0).y_low(),
            );
        }
        let rnd = self.get_random_number();
        // `GetRandomNumber() & 3 | sprite_subtype[k] << 2` — note C precedence:
        // `&` binds tighter than `|`, so `(rnd & 3) | (subtype << 2)`.
        let idx = ((rnd & 3) | (self.sprite_slot(k).subtype() << 2)) as usize;
        let j = GANON_WARP_SUBTYPES[idx & 0x1f];
        self.sprite_slot_mut(k).set_subtype(j);
        let ju = j as usize;
        self.swamola_target_mut(0)
            .set_x_low(GANON_WARP_TARGET_X_LOW[ju]);
        self.swamola_target_mut(0)
            .set_y_low(GANON_WARP_TARGET_Y_LOW[ju]);
        self.sprite_slot_mut(k).set_ai_state(a);
        self.sprite_slot_mut(k).set_x_velocity(0);
        self.sprite_slot_mut(k).set_y_velocity(0);
        self.sprite_slot_mut(k).set_delay_main(48);
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x28);
    }

    // void Ganon_ShakeHead(int k) {  // sprite_main.c:15069
    //   static const uint8 kGanon_HeadDir[18] = { ... };
    //   sprite_head_dir[k] = kGanon_HeadDir[sprite_delay_main[k] >> 3];
    // }
    pub(super) fn ganon_shake_head(&mut self, k: usize) {
        let idx = (self.sprite_slot(k).delay_main() >> 3) as usize;
        // The C array is 18 entries; sprite_delay_main >> 3 in the
        // boss-fight path is always within bounds (delay <= 127 -> idx <= 15
        // before the death cases). Guard defensively at 18 to mirror C
        // out-of-bounds memory access by clamping (no panic, no UB).
        self.sprite_slot_mut(k)
            .set_head_direction(GANON_SHAKE_HEAD_DIRECTIONS[idx % 18]);
    }

    // void Ganon_Draw(int k) {  // sprite_main.c:15077
    pub(super) fn ganon_draw(&mut self, k: usize) {
        let g = self.sprite_slot(k).graphics();
        if sign8(g)
            || (self.sprite_slot(k).ai_state() != 19
                && self.sprite_slot(k).delay_aux4() == 0
                && self.game_state.dungeon.torch.ganon_torch_count() == 0)
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
        if self.game_state.frame.submodule != 0 {
            self.sprite_correct_oam_entries_for_ganon(k, 9, 0xff);
        }

        if self.sprite_slot(k).g() == 9 {
            self.oam_state_mut().set_current_pointer(0x828);
            self.oam_state_mut().set_current_extended_pointer(0xa2a);
            self.ganon_draw_emit_g9_overlay_for_ganon(k);
        }

        let z = (self.sprite_slot(k).z() as u16).wrapping_sub(1);
        let frame: u16 = if (z >> 11) > 4 { 4 } else { z >> 11 };
        let cy = self.game_state.sprites.workspace.current_sprite_y();
        self.sprite_workspace_mut()
            .set_current_sprite_y(cy.wrapping_add(z));
        self.oam_state_mut().set_current_pointer(0x9f4);
        self.oam_state_mut().set_current_extended_pointer(0xa9d);
        let bak = self.sprite_slot(k).oam_flags();
        self.sprite_slot_mut(k).set_oam_flags(0);
        self.sprite_slot_mut(k).set_object_priority(48);
        self.sprite_draw_large_shadow_for_ganon(k, frame as usize);
        self.sprite_slot_mut(k).set_oam_flags(bak);
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
        self.sprite_slot_mut(j).set_x_velocity(pt.x);
        self.sprite_slot_mut(j).set_y_velocity(pt.y);
    }

    // Ganon_ExtinguishTorch_adjust_translucency / Ganon_ExtinguishTorch and
    // Dungeon_ExtinguishTorch live in dungeon.c. Port them locally here because
    // Sprite_D6_Ganon calls them from sprite_main.c.
    fn ganon_extinguish_torch_adjust_translucency_for_ganon(&mut self) {
        self.Palette_AssertTranslucencySwap();
        self.dungeon_torch_mut().set_attr(0xc0);
        self.dungeon_extinguish_torch_for_ganon();
    }

    fn ganon_extinguish_torch_for_ganon(&mut self) {
        self.dungeon_torch_mut().set_attr(193);
        self.dungeon_extinguish_torch_for_ganon();
    }

    fn dungeon_extinguish_torch_for_ganon(&mut self) {
        let y = self.game_state.dungeon.torch.attr_index() * 2
            + self.game_state.dungeon.torch.torches_start_index() as usize;
        let idx = y >> 1;
        let mut r8 = self
            .game_state
            .dungeon
            .object_tracking
            .object_tilemap_pos(idx)
            & 0x7fff;
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(idx, r8);

        let opos = (self
            .game_state
            .dungeon
            .object_tracking
            .object_pos_in_objdata(idx)
            & 0xff)
            >> 1;
        self.dungeon_torch_mut()
            .set_torch_data_word_index(opos as usize, r8);

        r8 &= 0x3fff;
        self.room_draw_adjust_torch_lighting_change(r8, 0x0ec2, r8);
        self.request_nmi_copy_packets();

        if self.game_state.dungeon.torch.wants_lights_out() != 0
            && self.game_state.dungeon.torch.lit_torches() != 0
        {
            self.dungeon_torch_mut().decrement_lit_torches();
            if self.game_state.dungeon.torch.lit_torches() < 3 {
                if self.game_state.dungeon.torch.lit_torches() == 0 {
                    self.set_sub_screen_layers(1);
                }
                const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
                let plus =
                    LIT_TORCHES_COLOR_PLUS[self.game_state.dungeon.torch.lit_torches() as usize];
                self.set_overworld_fixed_color_adjustment(plus);
                self.set_submodule(10);
                self.set_subsubmodule(0);
            }
        }

        let torch_timer = self.game_state.dungeon.torch.attr_index();
        self.dungeon_torch_mut().clear_timer(torch_timer);
        self.dungeon_torch_mut().clear_attr();
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
        let mut oam = self.game_state.oam.current_pointer_usize() + 5 * 4;
        let g = self.sprite_slot(k).graphics() as usize;
        for i in 0..12 {
            let j = g * 12 + i;
            let flags_mask = if (info_flags & 0x0f) >= 5 { 0xf0 } else { 0xff };
            let flags = info_flags | (GANON_DRAW_FLAGS[j] & flags_mask);
            let x = info_x.wrapping_add_signed(i16::from(GANON_DRAW_X_OFFSETS[j])) as u8;
            let y = info_y.wrapping_add_signed(i16::from(GANON_DRAW_Y_OFFSETS[j])) as u8;
            self.set_oam_plain_at_for_ganon(oam, x, y, GANON_DRAW_CHARS[j], flags, 2);
            oam += 4;
        }
    }

    fn ganon_draw_patch_head_oam_for_ganon(&mut self, k: usize) {
        let g = self.sprite_slot(k).graphics() as usize;
        let offs = GANON_DRAW_OAM_START_OFFSETS[g];
        if offs == 15 {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize() + (5 + usize::from(offs)) * 4;
        let j = usize::from(self.sprite_slot(k).head_direction()) * 2
            + if self.sprite_slot(k).direction() != 0 {
                6
            } else {
                0
            };
        self.oam_state_mut()
            .set_entry_char(oam, GANON_DRAW_PATCH_CHARS[j]);
        self.oam_state_mut()
            .merge_entry_flags(oam, 0x3f, GANON_DRAW_PATCH_FLAGS[j]);
        self.oam_state_mut()
            .set_entry_char(oam + 4, GANON_DRAW_PATCH_CHARS[j + 1]);
        self.oam_state_mut()
            .merge_entry_flags(oam + 4, 0x3f, GANON_DRAW_PATCH_FLAGS[j + 1]);
    }

    // Rewired to canonical Sprite_CorrectOamEntries port.
    fn sprite_correct_oam_entries_for_ganon(&mut self, k: usize, count: u8, mask: u8) {
        self.sprite_correct_oam_entries(k, count as i32, mask);
    }

    fn ganon_draw_emit_g9_overlay_for_ganon(&mut self, k: usize) {
        self.sprite_draw_multiple(k, &GANON_DRAW_FRAMES, None);
    }

    fn sprite_draw_large_shadow_for_ganon(&mut self, k: usize, frame: usize) {
        let base = frame * 3;
        self.sprite_draw_multiple(k, &GANON_LARGE_SHADOW_DRAW_FRAMES[base..base + 3], None);
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
        self.oam_state_mut().write_entry(oam, x, y, charnum, flags);
        let ext_index = (oam - OAM_BUF) / 4;
        let value = big;
        self.oam_state_mut().set_extended_byte(ext_index, value);
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
        s.sprite_workspace_mut().set_current_sprite_x(0x100);
        s.sprite_workspace_mut().set_current_sprite_y(0x80);
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
        s.sprite_slot_mut(k).set_hit_timer(26);
        s.ganon_enable_invincibility(k);
        assert_eq!(s.sprite_slot(k).hit_timer(), 0);
        assert_eq!(s.sprite_slot(k).ai_state(), 19);
        assert_eq!(s.sprite_slot(k).delay_main(), 127);
        assert_eq!(s.sprite_slot(k).sprite_type(), 215);

        let mut s2 = fresh_state();
        s2.sprite_slot_mut(k).set_hit_timer(27);
        s2.ganon_enable_invincibility(k);
        // Nothing should change.
        assert_eq!(s2.sprite_slot(k).hit_timer(), 27);
        assert_eq!(s2.sprite_slot(k).ai_state(), 0);
        assert_eq!(s2.sprite_slot(k).sprite_type(), 0);

        let mut s3 = fresh_state();
        // Top bit set + low 7 bits == 26 -> still triggers.
        s3.sprite_slot_mut(k).set_hit_timer(26 | 0x80);
        s3.ganon_enable_invincibility(k);
        assert_eq!(s3.sprite_slot(k).hit_timer(), 0);
    }

    #[test]
    fn phase1_animate_trident_spin_indexes_into_func2_tables() {
        let mut s = fresh_state();
        let k = 0;
        // delay_main = 0 -> base = 0; sprite_D = 0 -> bonus = 0 -> j = 0.
        s.sprite_slot_mut(k).set_delay_main(0);
        s.sprite_slot_mut(k).set_direction(0);
        s.ganon_phase1_animate_trident_spin(k);
        assert_eq!(s.sprite_slot(k).g(), GANON_SPIN_G_STATES[0]); // 8
        assert_eq!(s.sprite_slot(k).graphics(), GANON_TRIDENT_SPIN_GRAPHICS[0]); // 0

        // delay_main = 28 (>> 2 == 7, & 7 == 7); D = 1 -> bonus = 8 -> j = 15.
        s.sprite_slot_mut(k).set_delay_main(28);
        s.sprite_slot_mut(k).set_direction(1);
        s.ganon_phase1_animate_trident_spin(k);
        assert_eq!(s.sprite_slot(k).g(), GANON_SPIN_G_STATES[15]); // 1
        assert_eq!(s.sprite_slot(k).graphics(), GANON_TRIDENT_SPIN_GRAPHICS[15]);
        // 9
    }

    #[test]
    fn handle_animation_idle_writes_g_and_gfx_per_direction() {
        let mut s = fresh_state();
        let k = 3;
        s.sprite_slot_mut(k).set_direction(0);
        s.ganon_handle_animation_idle(k);
        assert_eq!(s.sprite_slot(k).g(), 9);
        assert_eq!(s.sprite_slot(k).graphics(), 2);

        s.sprite_slot_mut(k).set_direction(1);
        s.ganon_handle_animation_idle(k);
        assert_eq!(s.sprite_slot(k).g(), 10);
        assert_eq!(s.sprite_slot(k).graphics(), 10);
    }

    #[test]
    fn shake_head_indexes_table_by_delay_main_shift3() {
        let mut s = fresh_state();
        let k = 1;
        // delay_main 24 -> idx 3 -> GANON_SHAKE_HEAD_DIRECTIONS[3] = 1.
        s.sprite_slot_mut(k).set_delay_main(24);
        s.ganon_shake_head(k);
        assert_eq!(s.sprite_slot(k).head_direction(), 1);
        // delay_main 0 -> idx 0 -> 0.
        s.sprite_slot_mut(k).set_delay_main(0);
        s.ganon_shake_head(k);
        assert_eq!(s.sprite_slot(k).head_direction(), 0);
        // delay_main 32 -> idx 4 -> 2.
        s.sprite_slot_mut(k).set_delay_main(32);
        s.ganon_shake_head(k);
        assert_eq!(s.sprite_slot(k).head_direction(), 2);
    }

    #[test]
    fn select_warp_location_zeroes_velocities_and_sets_targets() {
        let mut s = fresh_state();
        let k = 0;
        // Seed sprite_subtype so the (rnd & 3 | subtype<<2) index is
        // deterministic enough for an assertion: with subtype = 0, the
        // resulting index is rnd & 3 only — that picks one of the first
        // four entries of GANON_WARP_SUBTYPES (which are 4,5,6,7).
        s.sprite_slot_mut(k).set_subtype(0);
        // Pre-clobber vels to ensure they get zeroed.
        s.sprite_slot_mut(k).set_x_velocity(5);
        s.sprite_slot_mut(k).set_y_velocity(7);
        s.ganon_select_warp_location(k, 12);
        let j = s.sprite_slot(k).subtype();
        assert!((4..=7).contains(&j));
        assert_eq!(
            s.ram[SWAMOLA_TARGET_X_LO_GANON],
            GANON_WARP_TARGET_X_LOW[j as usize]
        );
        assert_eq!(
            s.ram[SWAMOLA_TARGET_Y_LO_GANON],
            GANON_WARP_TARGET_Y_LOW[j as usize]
        );
        assert_eq!(s.sprite_slot(k).ai_state(), 12);
        assert_eq!(s.sprite_slot(k).x_velocity(), 0);
        assert_eq!(s.sprite_slot(k).y_velocity(), 0);
        assert_eq!(s.sprite_slot(k).delay_main(), 48);
    }

    #[test]
    fn spawn_falling_tiles_increments_anim_clock_until_four() {
        let mut s = fresh_state();
        let k = 0;
        // Pre-clear overlord slot 7 so the search succeeds.
        s.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut s.ram, 7)
            .clear();
        s.sprite_slot_mut(k).set_anim_clock(0);
        // Seed link coords so we can verify the high-byte copy.
        s.follower_link_state_mut().set_x(0x0234);
        s.follower_link_state_mut().set_y(0x0588);
        s.ganon_spawn_falling_tiles_overlord(k);
        assert_eq!(s.sprite_slot(k).anim_clock(), 1);
        assert_eq!(
            s.game_state.sprites.overlord_slots.slot(7).overlord_type(),
            GANON_FALLING_TILE_OVERLORD_TYPES[0]
        );
        assert_eq!(
            s.game_state.sprites.overlord_slots.slot(7).x_low(),
            GANON_FALLING_TILE_OVERLORD_X_LOW[0]
        );
        assert_eq!(s.game_state.sprites.overlord_slots.slot(7).x_high(), 0x02);
        assert_eq!(s.game_state.sprites.overlord_slots.slot(7).y_high(), 0x05);

        // Advance the anim clock past 3 and ensure no further write happens.
        s.sprite_slot_mut(k).set_anim_clock(4);
        let bak = s.game_state.sprites.overlord_slots.slot(7).overlord_type();
        s.ganon_spawn_falling_tiles_overlord(k);
        assert_eq!(
            s.game_state.sprites.overlord_slots.slot(7).overlord_type(),
            bak
        );
        assert_eq!(s.sprite_slot(k).anim_clock(), 4);
    }

    #[test]
    fn handle_fire_bat_circle_writes_eight_overlords_and_seeds_counter() {
        let mut s = fresh_state();
        // Seed overlord_x_lo word to a known value so we can predict t for each i.
        s.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut s.ram, 0)
            .set_adjacent_x_low_word(0x10);
        s.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut s.ram, 2)
            .set_x_low(0); // scale = 0 -> GanonSin returns 0.
                           // Sprite 0 at (0x80, 0x80) for predictable add.
        s.sprite_set_x(0, 0x80);
        s.sprite_set_y(0, 0x80);
        // sprite_ai_state for indices 1..=8: leave them at 0 so the velocity
        // assignments fire for every i.
        for i in 1..=8 {
            s.sprite_slot_mut(i).set_ai_state(0);
        }

        s.ganon_handle_fire_bat_circle(0);

        // overlord_x_lo word should have decremented by 4.
        assert_eq!(
            s.game_state
                .sprites
                .overlord_slots
                .slot(0)
                .adjacent_x_low_word(),
            0x10u16.wrapping_sub(4)
        );
        // tmp_counter is set to 8.
        assert_eq!(s.game_state.scratch_counter.value(), 8);
        // With scale = 0, GanonSin -> 0, so every overlord_x_hi[i+1] == sprite_x_lo(0) == 0x80.
        for i in 0..8 {
            assert_eq!(
                s.game_state.sprites.overlord_slots.slot(i + 1).x_high(),
                0x80
            );
            assert_eq!(s.game_state.sprites.overlord_slots.slot(i + 1).gen2(), 0x80);
        }
    }

    #[test]
    fn spawn_spiral_bat_initializes_dynamic_slot_fields() {
        let mut s = fresh_state();
        let k = 0;
        // Canonical Sprite_SpawnDynamicallyEx walks j_in (8) down to 0; the
        // highest free slot in [0..=8] wins. Ensure slot 8 is free so it
        // gets picked (matching the C entry-point behavior).
        s.sprite_slot_mut(8).set_state(0);
        s.sprite_workspace_mut().set_current_sprite_x(0x40);
        s.sprite_workspace_mut().set_current_sprite_y(0x60);
        s.ganon_spawn_spiral_bat(k);
        let j = 8;
        assert_eq!(s.sprite_slot(j).state(), 9);
        assert_eq!(s.sprite_slot(j).sprite_type(), 0xc9);
        assert_eq!(s.sprite_slot(j).anim_clock(), 4);
        assert_eq!(s.sprite_slot(j).oam_flags(), 3);
        assert_eq!(s.sprite_slot(j).flags3(), 0x40);
        assert_eq!(s.sprite_slot(j).flags2(), 1);
        assert_eq!(s.sprite_slot(j).deflection_bits(), 0x80);
        assert_eq!(s.sprite_slot(j).y_high(), 128);
        assert_eq!(s.sprite_slot(j).delay_main(), 48);
        assert_eq!(s.sprite_slot(j).bump_damage(), 7);
        assert_eq!(s.sprite_slot(j).ignore_projectile(), 7);
    }
}
