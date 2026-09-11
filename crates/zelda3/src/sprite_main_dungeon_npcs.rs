//! Ported Priest / Thief / Kiki / Cucco / Smithy handlers from sprite_main.c.
//!
//! Each method preserves a 1:1 mapping to the C source (sprite_main.c lines
//! 1493, 5473..5653, 6236, 9367..9468, 9990..10302, 10859..10906,
//! 12877, 13022..13027, 17330..17445, 24358..24611, 25047..25055).
//! The original C body is reproduced as a comment block immediately above
//! each port so a reviewer can verify behavior line-by-line.
//!
//! Helpers reached from these handlers route through canonical ports, with
//! local `_for_dn` adapters kept only where the split module needs a narrow
//! signature bridge.

use super::sprite::{dmd, DrawMultipleData};
use super::*;

// Local mirrors of sprite-RAM addresses that are not yet exposed through
// `zelda_rtl.rs`. The C declarations live in `src/variables.h`.
const SRAM_PROGRESS_INDICATOR_3: usize = 0x0f3c9;
// Feature flag bit (features.h:40).
const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
// hud.h:8.
const HUD_ITEM_HAMMER: u8 = 12;
const SPRITE_TYPE_UNCLE_AND_PRIEST: u8 = 0x73;

// sprite_main.c:13 — `kSpriteKeese_Tab2` (cosine wave used by Cucco_Calm).
const CUCCO_CALM_CIRCLE_X_VELOCITIES: [i8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
];
// sprite_main.c:14 — `kSpriteKeese_Tab3` (sine wave; note the `-9` at index 13
// matches the original ROM's quirky entry).
const CUCCO_CALM_CIRCLE_Y_VELOCITIES: [i8; 16] = [
    -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
];

// sprite_main.c:9445.
const CHICKEN_AVENGER: [u8; 2] = [0, 0xff];

/// Replays the two chained 16-bit `ADC` sequences at ROM $06:a7ff-$06:a843.
///
/// `GetRandomNumber` returns with a meaningful carry. The intervening STA,
/// AND, branch, and loads preserve it, so the first coordinate addition must
/// consume that carry. Its 16-bit carry-out then feeds the other coordinate,
/// exactly as the low-byte/high-byte ADC pairs do in the ROM.
fn cucco_avenger_spawn_coordinates(
    random: crate::rom_random::RomRandomResult,
    bg2_h: u16,
    bg2_v: u16,
) -> (u16, u16) {
    fn adc_u8(value: u16, addend: u8, carry: bool) -> (u16, bool) {
        let sum = u32::from(value) + u32::from(addend) + u32::from(carry);
        (sum as u16, sum > u32::from(u16::MAX))
    }

    let t = random.value();
    if t & 2 != 0 {
        let (x, carry) = adc_u8(bg2_h, t, random.carry());
        let (y, _) = adc_u8(bg2_v, CHICKEN_AVENGER[(t & 1) as usize], carry);
        (x, y)
    } else {
        let (y, carry) = adc_u8(bg2_v, t, random.carry());
        let (x, _) = adc_u8(bg2_h, CHICKEN_AVENGER[(t & 1) as usize], carry);
        (x, y)
    }
}

// sprite.c:507.
const ABSORPTION_SFX: [u8; 15] = [
    0xb, 0xa, 0xa, 0xa, 0xb, 0xb, 0xb, 0xb, 0xb, 0xb, 0xb, 0xb, 0x2f, 0x2f, 0xb,
];

// sprite_main.c:80.
const PRIEST_DRAW_FRAMES: [DrawMultipleData; 20] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0e20,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0e26,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0e20,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4e26,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0e0e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0e24,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0e0e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0e24,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0e22,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0e28,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0e22,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0e2a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4e22,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4e28,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4e22,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4e2a,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: 1,
        char_flags: 0x0e0a,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x0e0c,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: 1,
        char_flags: 0x0e0a,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x0e0c,
        ext: 2,
    },
];

// sprite_main.c:13127.
const UNCLE_DRAW_FRAMES: [DrawMultipleData; 48] = [
    dmd(0, -10, 0x0e00, 2),
    dmd(0, 0, 0x0c06, 2),
    dmd(0, -10, 0x0e00, 2),
    dmd(0, 0, 0x0c06, 2),
    dmd(0, -10, 0x0e00, 2),
    dmd(0, 0, 0x0c06, 2),
    dmd(0, -10, 0x0e02, 2),
    dmd(0, 0, 0x0c06, 2),
    dmd(0, -10, 0x0e02, 2),
    dmd(0, 0, 0x0c06, 2),
    dmd(0, -10, 0x0e02, 2),
    dmd(0, 0, 0x0c06, 2),
    dmd(-7, 2, 0x0d07, 2),
    dmd(-7, 2, 0x0d07, 2),
    dmd(10, 12, 0x8d05, 0),
    dmd(10, 4, 0x8d15, 0),
    dmd(0, -10, 0x0e00, 2),
    dmd(0, 0, 0x0c04, 2),
    dmd(-7, 1, 0x0d07, 2),
    dmd(-7, 1, 0x0d07, 2),
    dmd(10, 13, 0x8d05, 0),
    dmd(10, 5, 0x8d15, 0),
    dmd(0, -9, 0x0e00, 2),
    dmd(0, 1, 0x4c04, 2),
    dmd(-7, 8, 0x8d05, 0),
    dmd(1, 8, 0x8d06, 0),
    dmd(0, -10, 0x0e02, 2),
    dmd(-6, -1, 0x4d07, 2),
    dmd(0, 0, 0x0c23, 2),
    dmd(0, 0, 0x0c23, 2),
    dmd(-9, 7, 0x8d05, 0),
    dmd(-1, 7, 0x8d06, 0),
    dmd(0, -9, 0x0e02, 2),
    dmd(-6, 0, 0x4d07, 2),
    dmd(0, 1, 0x0c25, 2),
    dmd(0, 1, 0x0c25, 2),
    dmd(-10, -17, 0x0d07, 2),
    dmd(15, -12, 0x8d15, 0),
    dmd(15, -4, 0x8d05, 0),
    dmd(0, -28, 0x0e08, 2),
    dmd(-8, -19, 0x0c20, 2),
    dmd(8, -19, 0x4c20, 2),
    dmd(0, -28, 0x0e08, 2),
    dmd(0, -28, 0x0e08, 2),
    dmd(-8, -19, 0x0c20, 2),
    dmd(8, -19, 0x4c20, 2),
    dmd(-8, -19, 0x0c20, 2),
    dmd(8, -19, 0x4c20, 2),
];
const UNCLE_DRAW_SWORD_DMA_INDEX: [u8; 8] = [8, 8, 0, 0, 6, 6, 0, 0];
const UNCLE_DRAW_SHIELD_DMA_INDEX: [u8; 8] = [0, 0, 0, 0, 4, 4, 0, 0x8b];
const UNCLE_ROM_DRAW_TABLE: u16 = 0xd203;
const UNCLE_ROM_DRAW_DIRECTION_STRIDE: u16 = 12 * 8;
const UNCLE_ROM_DRAW_GRAPHICS_STRIDE: u16 = 6 * 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UncleDrawSource {
    PortedTable { start: usize },
    WrappedWram { address: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UncleEquipmentDmaIndices {
    sword: u8,
    shield: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UncleDrawPlan {
    source: UncleDrawSource,
    equipment: UncleEquipmentDmaIndices,
}

// sprite_main.c:17369..17371.
const THIEF_GFX: [u8; 12] = [11, 8, 2, 5, 9, 6, 0, 3, 10, 7, 1, 4];
const THIEF_SPAWN_ITEMS: [u8; 4] = [0xd9, 0xe1, 0xdc, 0xd9];
const THIEF_SPAWN_XVEL: [i8; 6] = [0, 24, 24, 0, -24, -24];
const THIEF_DRAW_FRAMES: [DrawMultipleData; 24] = [
    DrawMultipleData {
        x: 0,
        y: -6,
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
        y: -6,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4006,
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
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0022,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4022,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0024,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0002,
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
        y: -7,
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
        y: -7,
        char_flags: 0x0002,
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
        y: -8,
        char_flags: 0x4002,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x400a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
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
        y: -7,
        char_flags: 0x4002,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x400a,
        ext: 2,
    },
];
const THIEF_DRAW_CHAR: [u8; 4] = [2, 2, 0, 4];
const THIEF_DRAW_FLAGS: [u8; 4] = [0x40, 0, 0, 0];

const ZELDA_XVEL: [i8; 4] = [0, 0, -9, 9];
const ZELDA_YVEL: [i8; 4] = [-9, 9, 0, 0];
const ZELDA_ENTERING_SANCTUARY_DELAYS: [u8; 4] = [38, 26, 44, 1];
const ZELDA_ENTERING_SANCTUARY_DIRECTIONS: [u8; 4] = [1, 3, 1, 2];
const THIEF_SPAWN_YVEL: [i8; 6] = [-32, -16, 16, 32, 16, -16];

// Returning Smithy tables (sprite_main.c:9996..9999).
const RETURNING_SMITHY_DELAY: [i8; 3] = [104, 12, 0];
const RETURNING_SMITHY_DIR: [i8; 3] = [0, 2, -1];
const RETURNING_SMITHY_XVEL: [i8; 4] = [0, 0, -13, 13];
const RETURNING_SMITHY_YVEL: [i8; 4] = [-13, 13, 0, 0];
const RETURNING_SMITHY_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4122,
        ext: 2,
    },
];
const RETURNING_SMITHY_DMA: [u8; 8] = [0xc0, 0xc0, 0xa0, 0xa0, 0x80, 0x60, 0x80, 0x60];

// Smithy_Main animation tables (sprite_main.c:10090..10091).
const SMITHY_GFX: [u8; 8] = [0, 1, 2, 3, 3, 2, 1, 0];
const SMITHY_FRAME_DURATIONS: [u8; 8] = [24, 4, 1, 16, 16, 5, 10, 16];
const SMITHY_DRAW_FRAMES: [DrawMultipleData; 20] = [
    dmd(1, 0, 0x4040, 2),
    dmd(-11, -10, 0x4060, 2),
    dmd(-1, 0, 0x0040, 2),
    dmd(11, -10, 0x0060, 2),
    dmd(1, 0, 0x4040, 2),
    dmd(-3, -14, 0x4044, 2),
    dmd(-1, 0, 0x0040, 2),
    dmd(3, -14, 0x0044, 2),
    dmd(1, 0, 0x4042, 2),
    dmd(11, -10, 0x0060, 2),
    dmd(-1, 0, 0x0042, 2),
    dmd(-11, -10, 0x4060, 2),
    dmd(1, 0, 0x4042, 2),
    dmd(13, 2, 0x4062, 2),
    dmd(-1, 0, 0x0042, 2),
    dmd(-13, 2, 0x0062, 2),
    dmd(0, 0, 0x4064, 2),
    dmd(0, 0, 0x4062, 2),
    dmd(0, 0, 0x0064, 2),
    dmd(0, 0, 0x0064, 2),
];

// Smithy_Spark animation tables (sprite_main.c:10259..10260).
const SMITHY_SPARK_GFX: [i8; 7] = [0, 1, 2, 1, 2, 1, -1];
const SMITHY_SPARK_DELAY: [i8; 6] = [4, 1, 3, 2, 1, 1];

// UncleAndSage Y-offset (sprite_main.c:10888).
const UNCLE_AND_SAGE_Y: [i16; 3] = [0, -9, 0];
// The ROM's walk-out tables have TWO entries, but Uncle_AtHouse ($85de3e)
// runs its step-advance block unconditionally — on the FINAL advance it reads
// one entry past each table. Those out-of-range ROM bytes are fixed data
// (Snes9x trace, PC $05decd/$05ded3: delay=0x02, direction=0xBD) and the
// direction 0xBD then indexes far past the velocity tables (PC $05deda/$05dee0
// observed xvel=0x90, yvel=0x0D). The third entries below ARE those ROM bytes,
// kept so WRAM stays byte-identical with the oracle (the stale slot residue is
// re-read when room 0x0104 reloads at ~frame 14661).
const UNCLE_LEAVE_HOUSE_DELAYS: [u8; 3] = [64, 224, 2];
const UNCLE_LEAVE_HOUSE_DIRECTIONS: [u8; 3] = [2, 1, 0xbd];
const UNCLE_LEAVE_HOUSE_X_VELOCITIES: [i8; 4] = [0, 0, -12, 12];
const UNCLE_LEAVE_HOUSE_Y_VELOCITIES: [i8; 4] = [-12, 12, 0, 0];
// ROM bytes at UNCLE_LEAVE_HOUSE_X/Y_VELOCITIES + 0xBD (the corrupt-direction
// lookup of the final step), observed via the instrumented Snes9x trace.
const UNCLE_LEAVE_HOUSE_OOB_X_VELOCITY: i8 = 0x90u8 as i8;
const UNCLE_LEAVE_HOUSE_OOB_Y_VELOCITY: i8 = 0x0d;
const UNCLE_WRAPPED_DEPARTURE_EQUIPMENT_DMA: UncleEquipmentDmaIndices = UncleEquipmentDmaIndices {
    sword: 0,
    shield: 6,
};

fn uncle_draw_plan(direction: u8, graphics: u8) -> Option<UncleDrawPlan> {
    if direction <= 3 {
        let equipment_index = usize::from(direction) * 2 + usize::from(graphics);
        return Some(UncleDrawPlan {
            source: UncleDrawSource::PortedTable {
                start: usize::from(direction) * 12 + usize::from(graphics) * 6,
            },
            equipment: UncleEquipmentDmaIndices {
                sword: UNCLE_DRAW_SWORD_DMA_INDEX[equipment_index],
                shield: UNCLE_DRAW_SHIELD_DMA_INDEX[equipment_index],
            },
        });
    }
    if direction != UNCLE_LEAVE_HOUSE_DIRECTIONS[2] {
        return None;
    }
    let address = UNCLE_ROM_DRAW_TABLE
        .wrapping_add(u16::from(direction) * UNCLE_ROM_DRAW_DIRECTION_STRIDE)
        .wrapping_add(u16::from(graphics) * UNCLE_ROM_DRAW_GRAPHICS_STRIDE);
    (address < 0x2000).then_some(UncleDrawPlan {
        source: UncleDrawSource::WrappedWram { address },
        equipment: UNCLE_WRAPPED_DEPARTURE_EQUIPMENT_DMA,
    })
}

// CrystalMaiden_RunCutscene message table (sprite_main.c:23297).
const CRYSTAL_MAIDEN_MSGS: [u16; 9] = [
    0x133, 0x132, 0x137, 0x134, 0x136, 0x132, 0x135, 0x138, 0x13c,
];

// Kiki_OfferEntranceService leave targets and per-state vectors
// (sprite_main.c:24470..24514).
const KIKI_LEAVE_X: [u16; 3] = [0xf4f, 0xf70, 0xf5d];
const KIKI_LEAVE_Y: [u16; 3] = [0x661, 0x64c, 0x624];
const KIKI_ZVEL: [u8; 2] = [32, 28];
const KIKI_LEAVE_Y_ACCELERATION_BY_TARGET: [i8; 3] = [2, 1, -1i8];
const KIKI_FINAL_LEAVE_HOP_DELAYS: [u8; 2] = [82, 0];
const KIKI_XVEL7: [i8; 4] = [0, 0, -9, 9];
const KIKI_YVEL7: [i8; 4] = [-9, 9, 0, 0];
const KIKI_DRAW_FRAMES1: [DrawMultipleData; 32] = [
    dmd(0, -6, 0x0020, 2),
    dmd(0, 0, 0x0022, 2),
    dmd(0, -6, 0x0020, 2),
    dmd(0, 0, 0x4022, 2),
    dmd(0, -6, 0x0020, 2),
    dmd(0, 0, 0x0022, 2),
    dmd(0, -6, 0x0020, 2),
    dmd(0, 0, 0x4022, 2),
    dmd(-1, -6, 0x0020, 2),
    dmd(0, 0, 0x0022, 2),
    dmd(-1, -6, 0x0020, 2),
    dmd(0, 0, 0x0022, 2),
    dmd(1, -6, 0x4020, 2),
    dmd(0, 0, 0x4022, 2),
    dmd(1, -6, 0x4020, 2),
    dmd(0, 0, 0x4022, 2),
    dmd(0, -6, 0x01ce, 2),
    dmd(0, 0, 0x01ee, 2),
    dmd(0, -6, 0x01ce, 2),
    dmd(0, 0, 0x01ee, 2),
    dmd(0, -6, 0x41ce, 2),
    dmd(0, 0, 0x41ee, 2),
    dmd(0, -6, 0x41ce, 2),
    dmd(0, 0, 0x41ee, 2),
    dmd(-1, -6, 0x01ce, 2),
    dmd(0, 0, 0x01ec, 2),
    dmd(-1, -6, 0x41ce, 2),
    dmd(0, 0, 0x01ec, 2),
    dmd(1, -6, 0x41ce, 2),
    dmd(0, 0, 0x41ec, 2),
    dmd(1, -6, 0x01ce, 2),
    dmd(0, 0, 0x41ec, 2),
];
const KIKI_DRAW_FRAMES2: [DrawMultipleData; 12] = [
    dmd(0, -6, 0x01ca, 0),
    dmd(8, -6, 0x41ca, 0),
    dmd(0, 2, 0x01da, 0),
    dmd(8, 2, 0x41da, 0),
    dmd(0, 10, 0x01cb, 0),
    dmd(8, 10, 0x41cb, 0),
    dmd(0, -6, 0x01db, 0),
    dmd(8, -6, 0x41db, 0),
    dmd(0, 2, 0x01cc, 0),
    dmd(8, 2, 0x41cc, 0),
    dmd(0, 10, 0x01dc, 0),
    dmd(8, 10, 0x41dd, 0),
];
const KIKI_DMA: [u8; 32] = [
    0x20, 0xc0, 0x20, 0xc0, 0, 0xa0, 0, 0xa0, 0x40, 0x80, 0x40, 0x60, 0x40, 0x80, 0x40, 0x60, 0, 0,
    0xfa, 0xff, 0x20, 0, 0, 2, 0, 0, 0, 0, 0x22, 0, 0, 2,
];

impl ZeldaState {
    pub(super) fn uncle_passage_item_receipt_starts_this_main_slice(&self) -> bool {
        if self.game_state.frame.main_module != 7 || self.game_state.frame.submodule != 0 {
            return false;
        }
        (0..16).rev().any(|k| {
            let sprite = self.sprite_slot_view(k);
            sprite.sprite_type() == SPRITE_TYPE_UNCLE_AND_PRIEST
                && sprite.e() == 0
                && sprite.subtype2() != 0
                && sprite.ai_state() == 1
                && !self.sprite_return_if_inactive(k)
        })
    }

    pub(super) fn sprite_ab_crystal_maiden(&mut self, k: usize) {
        let x = self
            .game_state
            .sprites
            .workspace
            .current_sprite_x()
            .wrapping_sub(self.game_state.dungeon.moving_floor.floor_x_offset());
        let y = self
            .game_state
            .sprites
            .workspace
            .current_sprite_y()
            .wrapping_sub(self.game_state.dungeon.moving_floor.floor_y_offset());
        self.sprite_workspace_mut().set_current_sprite_x(x);
        self.sprite_workspace_mut().set_current_sprite_y(y);

        if self.sprite_slot_view(k).ai_state() >= 3 {
            self.crystal_maiden_draw(k);
        }
        self.activate_nmi_thread();
        if crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some() {
            eprintln!(
                "[POLY] host={} maiden call: module={:02x}/{:02x} did_run_step={} e={:#x} ai={}",
                self.frame_ctr_dbg,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                self.game_state.ending.attract_scene.intro_did_run_step(),
                self.sprite_slot_view(k).e(),
                self.sprite_slot_view(k).ai_state(),
            );
        }
        if self.game_state.ending.attract_scene.intro_did_run_step() == 0 {
            self.crystal_maiden_run_cutscene(k);
            self.attract_scene_mut().mark_intro_did_run_step();
        }
    }

    pub(super) fn crystal_maiden_run_cutscene(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_e();
        self.poly_runtime_mut().add_angle_b(6);
        if self.game_state.frame.submodule != 0 {
            return;
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.set_sub_screen_layers(0);
                self.sprite_slot_view_mut(k).increment_ai_state();
            }
            1 => {
                self.set_sub_screen_layers(1);
                self.sprite_slot_view_mut(k).increment_ai_state();
            }
            2 => {
                if self.game_state.poly.runtime.config1() < 6 {
                    self.poly_runtime_mut().clear_config1();
                    self.sprite_slot_view_mut(k).increment_ai_state();
                } else {
                    self.poly_runtime_mut().subtract_config1(3);
                    if self.game_state.poly.runtime.config1() >= 64 {
                        self.ancilla_add_sword_charge_sparkle_from_ancilla(
                            self.sprite_slot_view(k).a() as usize,
                        );
                    }
                }
            }
            3 => {
                self.sprite_slot_view_mut(k).increment_ai_state();
                self.crystal_maiden_palette_filter_step(k);
            }
            4 => self.crystal_maiden_palette_filter_step(k),
            5 => {
                let mut j =
                    i32::from(self.game_state.inventory.save_progress.palace_index_x2()) - 10;
                if j == 2
                    && self
                        .game_state
                        .inventory
                        .save_progress
                        .map_icons_indicator()
                        < 7
                {
                    self.save_progress_mut().set_map_icons_indicator(7);
                }
                if j == 14
                    && (self.game_state.inventory.player_resources.crystal_flags() & 0x7f) != 0x7f
                {
                    j = 16;
                }
                self.sprite_show_message_unconditional(CRYSTAL_MAIDEN_MSGS[(j >> 1) as usize]);
                self.sprite_slot_view_mut(k).increment_ai_state();
                if (self.game_state.inventory.player_resources.crystal_flags() & 0x7f) == 0x7f {
                    self.save_progress_mut().set_map_icons_indicator(8);
                }
            }
            6 => {
                self.sprite_show_message_unconditional(0x13a);
                self.sprite_slot_view_mut(k).increment_ai_state();
            }
            7 => {
                if self.multiselect_choice().value() != 0 {
                    self.sprite_slot_view_mut(k).set_ai_state(5);
                } else {
                    self.sprite_show_message_unconditional(0x139);
                    self.sprite_slot_view_mut(k).increment_ai_state();
                }
            }
            8 => {
                self.set_sub_screen_layers(0);
                self.prepare_dungeon_exit_from_boss_fight();
                self.sprite_slot_view_mut(k).set_state(0);
            }
            _ => {}
        }
    }

    fn crystal_maiden_palette_filter_step(&mut self, k: usize) {
        if self.sprite_slot_view(k).e() & 1 == 0 {
            self.PaletteFilter_SP5F();
            if self.game_state.display.palette_filter.countdown() == 0 {
                self.sprite_slot_view_mut(k).increment_ai_state();
                self.follower_link_state_mut().immobilize();
                self.follower_link_state_mut().set_receive_item_index(0);
                self.follower_link_state_mut().clear_item_hold_pose();
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().set_facing(0);
            }
        }
    }

    pub(super) fn sprite_76_zelda(&mut self, k: usize) {
        self.crystal_maiden_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if self.sprite_track_body_to_head(k) {
            self.sprite_move_xy(k);
        }
        match self.sprite_slot_view(k).subtype2() {
            0 => self.zelda_in_cell(k),
            1 => self.zelda_entering_sanctuary(k),
            2 => self.zelda_at_sanctuary(k),
            _ => {}
        }
    }

    pub(super) fn zelda_in_cell(&mut self, k: usize) {
        let dir = self.sprite_direction_to_face_link(k) ^ 3;
        self.sprite_slot_view_mut(k).set_head_direction(dir);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if !self.sprite_check_damage_to_link_same_layer(k) {
                    return;
                }
                self.sprite_slot_view_mut(k).increment_ai_state();
                self.follower_link_state_mut().increment_immobilized_flag();
                let j = self.sprite_slot_view(k).head_direction() as usize;
                self.sprite_slot_view_mut(k)
                    .set_x_velocity(ZELDA_XVEL[j] as u8);
                self.sprite_slot_view_mut(k)
                    .set_y_velocity(ZELDA_YVEL[j] as u8);
                self.sprite_slot_view_mut(k).set_delay_main(16);
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_show_message_unconditional(0x1c);
                    self.sprite_slot_view_mut(k).set_x_velocity(0);
                    self.sprite_slot_view_mut(k).set_y_velocity(0);
                    self.set_music_control(25);
                }
                let graphics = self.game_state.frame.frame_counter >> 3 & 1;
                self.sprite_slot_view_mut(k).set_graphics(graphics);
            }
            2 => {
                self.sprite_slot_view_mut(k).increment_ai_state();
                self.sprite_show_message_unconditional(0x25);
            }
            3 => {
                if self.multiselect_choice().value() != 0 {
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                } else {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_show_message_unconditional(0x24);
                }
            }
            4 => {
                self.follower_link_state_mut().clear_immobilized();
                self.save_progress_mut().set_which_starting_point(2);
                self.SavePalaceDeaths();
                self.follower_state_mut().set_indicator(1);
                self.Dungeon_FlagRoomData_Quadrants();
                self.sprite_become_follower(k);
                self.sprite_slot_view_mut(k).set_state(0);
                self.set_music_control(16);
            }
            _ => {}
        }
    }

    pub(super) fn zelda_entering_sanctuary(&mut self, k: usize) {
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = self.sprite_slot_view(k).a() as usize;
                    if j >= 4 {
                        self.sprite_slot_view_mut(k).increment_ai_state();
                        self.sprite_slot_view_mut(k).set_head_direction(0);
                        self.sprite_slot_view_mut(k).set_direction(0);
                        self.sprite_slot_view_mut(k).set_x_velocity(0);
                        self.sprite_slot_view_mut(k).set_y_velocity(0);
                        return;
                    }
                    self.sprite_slot_view_mut(k)
                        .set_delay_main(ZELDA_ENTERING_SANCTUARY_DELAYS[j]);
                    let dir = ZELDA_ENTERING_SANCTUARY_DIRECTIONS[j];
                    self.sprite_slot_view_mut(k).set_direction(dir);
                    self.sprite_slot_view_mut(k).set_head_direction(dir);
                    self.sprite_slot_view_mut(k).increment_a();
                    self.sprite_slot_view_mut(k)
                        .set_x_velocity(ZELDA_XVEL[dir as usize] as u8);
                    self.sprite_slot_view_mut(k)
                        .set_y_velocity(ZELDA_YVEL[dir as usize] as u8);
                }
                let graphics = self.game_state.frame.frame_counter >> 3 & 1;
                self.sprite_slot_view_mut(k).set_graphics(graphics);
            }
            1 => {
                self.sprite_show_message_unconditional(0x1d);
                self.sprite_slot_view_mut(k).increment_ai_state();
                self.follower_state_mut().set_zelda_rescue_cutscene_state(2);
                self.save_progress_mut().set_which_starting_point(1);
                self.SavePalaceDeaths();
                self.save_progress_mut().set_progress_indicator(2);
                self.sprite_load_graphics_properties_light_world_only();
            }
            2 => {
                let dir = self.sprite_direction_to_face_link(k) ^ 3;
                self.sprite_slot_view_mut(k).set_head_direction(dir);
                let j = self.sprite_show_solicited_message(k, 0x1e);
                if j & 0x100 != 0 {
                    self.sprite_slot_view_mut(k).set_direction(j as u8);
                    self.sprite_slot_view_mut(k).set_head_direction(j as u8);
                }
            }
            _ => {}
        }
    }

    pub(super) fn zelda_at_sanctuary(&mut self, k: usize) {
        let dir = self.sprite_direction_to_face_link(k) ^ 3;
        self.sprite_slot_view_mut(k).set_head_direction(dir);
        let msg = if self.game_state.inventory.player_resources.pendant_flags() & 7 == 7 {
            0x27
        } else if self
            .game_state
            .inventory
            .save_progress
            .map_icons_indicator()
            >= 3
        {
            0x26
        } else {
            0x1e
        };
        let j = self.sprite_show_solicited_message(k, msg);
        if j & 0x100 != 0 {
            self.sprite_slot_view_mut(k).set_direction(j as u8);
            self.sprite_slot_view_mut(k).set_head_direction(j as u8);
            self.player_resources_mut().set_heart_filler(0xa0);
        }
    }

    // ----- Priest cluster -----------------------------------------------

    // void Sprite_73_UncleAndPriest(int k) {  // 86bfe0
    pub(super) fn sprite_73_uncle_and_priest(&mut self, k: usize) {
        // Cycle ledger: SpriteActive_Main's type-$73 table entry is the
        // bank-6 bounce Sprite_73_UncleAndPriest_bounce $06:BFE0 (JSL
        // $05:DB86, 62 ... RTS, 42), an RTS-dispatch target that charges into
        // the open Sprite_ExecuteSingle scope.
        crate::cycle_ledger::charge(62);
        {
            // Sprite_73_UncleAndPriest $05:DB86 (JSL target, m8 x8):
            // $05:DB86-DB89 PHB, PHK, PLB, JSR Sprite_UncleAndSage (118) ...
            // $05:DB8C PLB : RTL (72).
            let _long = crate::cycle_ledger::routine(0x05_db86);
            crate::cycle_ledger::charge(118);
            {
                // Sprite_UncleAndSage $05:DB8E (JSR target): LDA $E90,x (32),
                // JSL JumpTableLocal (62 + 414) into the three-entry table at
                // $05:DB95; the state handlers are its jump targets. Every
                // JumpTableLocal dispatch here follows the messaging.rs
                // convention: $00:8781's 52 cycles up to its PLY are its own
                // frame (charge_routine), the remaining 362 are the caller's.
                let _scope = crate::cycle_ledger::routine(0x05_db8e);
                crate::cycle_ledger::charge_routine(0x00_8781, 52);
                crate::cycle_ledger::charge(32 + 62 + 414 - 52);
                match self.sprite_slot_view(k).e() {
                    0 => self.sprite_uncle(k),
                    1 => self.sprite_priest(k),
                    2 => self.sprite_sanctuary_mantle(k),
                    _ => {}
                }
            }
            crate::cycle_ledger::charge(72);
        }
        crate::cycle_ledger::charge(42);
    }

    // void Sprite_Uncle(int k) {  // 85de2c
    pub(super) fn sprite_uncle(&mut self, k: usize) {
        // Cycle ledger (a jump target of Sprite_UncleAndSage): $05:DE2C JSL
        // Uncle_Draw (62), $05:DE30 JSR Sprite_ReturnIfInactive_ (46; an
        // inactive sprite double-returns through it).
        crate::cycle_ledger::charge(62);
        self.uncle_draw(k);
        crate::cycle_ledger::charge(46);
        if self.sprite_return_if_inactive_bank5(k) {
            return;
        }
        // $05:DE33 LDA $E80,x (32), JSL JumpTableLocal (62 + 414) into the
        // two-entry table at $05:DE3A.
        crate::cycle_ledger::charge_routine(0x00_8781, 52);
        crate::cycle_ledger::charge(32 + 62 + 414 - 52);
        if self.sprite_slot_view(k).subtype2() == 0 {
            self.uncle_at_house(k);
        } else {
            self.uncle_in_passage(k);
        }
    }

    // void Uncle_AtHouse(int k) {  // 85de3e
    pub(super) fn uncle_at_house(&mut self, k: usize) {
        // Cycle ledger (a jump target): $05:DE3E JSR Sprite_Move_ (46),
        // $05:DE41 LDA $D80,x (32), JSL JumpTableLocal (62 + 414) into the
        // five-entry table at $05:DE48.
        crate::cycle_ledger::charge(46);
        self.sprite_move_xy(k);
        crate::cycle_ledger::charge_routine(0x00_8781, 52);
        crate::cycle_ledger::charge(32 + 62 + 414 - 52);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                // Uncle_TriggerTelepathy $05:DE52-DE71 (380; the JSL
                // Sprite_ShowMessageUnconditional callee charges itself).
                crate::cycle_ledger::charge(380);
                self.follower_link_state_mut()
                    .set_previous_position(0x0940, 0x215a);
                self.sprite_show_message_unconditional(0x1f);
                self.sprite_slot_view_mut(k).increment_ai_state();
            }
            1 => {
                // Uncle_AwakenLink $05:DE72-DE76 LDA $1A : AND #$03 : BNE
                // (56; taken +6 into the $05:DE82 RTS, 42).
                if (self.game_state.frame.frame_counter & 3) != 0 {
                    crate::cycle_ledger::charge(56 + 6 + 42);
                    return;
                }
                // $05:DE78-DE7C LDA $9C : CMP #$20 : BEQ (56); not equal
                // runs $05:DE7E DEC $9C : DEC $9D (76) and the RTS (42).
                if self.game_state.display.palette_filter.fixed_color_red() != 32 {
                    crate::cycle_ledger::charge(56 + 56 + 76 + 42);
                    self.subtract_fixed_color_red(1);
                    self.subtract_fixed_color_green(1);
                    return;
                }
                // BEQ taken (+6) into $05:DE83-DE99 (314, RTS included).
                crate::cycle_ledger::charge(56 + 56 + 6 + 314);
                self.follower_link_state_mut().increment_opening_pose();
                self.follower_link_state_mut()
                    .increment_sleep_in_bed_state();
                self.follower_link_state_mut().set_y(0x2157);
                self.follower_link_state_mut().immobilize();
                self.sprite_slot_view_mut(k).increment_ai_state();
            }
            2 => {
                // Uncle_DeclareCurfew $05:DE9A-DEAF (290).
                crate::cycle_ledger::charge(290);
                self.sprite_show_message_unconditional(0x0d);
                self.set_music_control(3);
                self.sprite_slot_view_mut(k).set_graphics(1);
                self.sprite_slot_view_mut(k).increment_ai_state();
            }
            3 => {
                let graphics = (self.game_state.frame.frame_counter >> 3) & 1;
                // Uncle_Embark $05:DEB4 LDA $DF0,x : BNE (48; taken +6 into
                // $05:DEED while the delay runs). The step block below is
                // charged where the translation runs it; $05:DEED-DEF7
                // (162, RTS included) closes every path. The ROM stores the
                // graphics index last; the translation stores it first.
                if self.sprite_slot_view(k).delay_main() == 0 {
                    crate::cycle_ledger::charge(48);
                } else {
                    crate::cycle_ledger::charge(48 + 6);
                }
                self.sprite_slot_view_mut(k).set_graphics(graphics);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    // ROM Uncle_AtHouse ($85dec7..$85deed, Snes9x trace-verified):
                    // every delay expiry runs the step-write block — including
                    // the FINAL one, which reads one entry past the two-entry
                    // tables (delay=0x02, dir=0xBD) and the corrupt direction
                    // then reads far past the velocity tables (0x90/0x0D). The
                    // AI state additionally advances on that final expiry, so
                    // the despawn handler runs on the NEXT frame — after one
                    // Sprite_MoveXY step with the corrupt velocities has moved
                    // the slot (x 0x78->0x71). Reproducing the whole sequence
                    // keeps the retired slot's WRAM residue byte-identical with
                    // the oracle; room 0x0104 re-reads it at ~frame 14661.
                    let j = usize::from(self.sprite_slot_view(k).a());
                    // $05:DEB9 LDY $D90,x : BNE (48; taken +6 past the
                    // $05:DEBE-DEC4 y-step, 100, when the step index is
                    // nonzero), then $05:DEC7-DEE8 (410; the closing BNE is
                    // taken +6 unless the incremented index reached 3, which
                    // runs $05:DEEA INC $D80,x, 52).
                    crate::cycle_ledger::charge(
                        48 + if j == 0 { 100 } else { 6 } + 410 + if j == 2 { 52 } else { 6 },
                    );
                    self.sprite_slot_view_mut(k).increment_a();
                    if j == 0 {
                        let y_low = self.sprite_slot_view(k).y_low().wrapping_sub(2);
                        self.sprite_slot_view_mut(k).set_y_low(y_low);
                    }
                    self.sprite_slot_view_mut(k)
                        .set_delay_main(UNCLE_LEAVE_HOUSE_DELAYS[j]);
                    let dir = usize::from(UNCLE_LEAVE_HOUSE_DIRECTIONS[j]);
                    self.sprite_slot_view_mut(k).set_direction(dir as u8);
                    let (x_vel, y_vel) = match (
                        UNCLE_LEAVE_HOUSE_X_VELOCITIES.get(dir),
                        UNCLE_LEAVE_HOUSE_Y_VELOCITIES.get(dir),
                    ) {
                        (Some(&x), Some(&y)) => (x, y),
                        _ => (
                            UNCLE_LEAVE_HOUSE_OOB_X_VELOCITY,
                            UNCLE_LEAVE_HOUSE_OOB_Y_VELOCITY,
                        ),
                    };
                    self.sprite_slot_view_mut(k).set_x_velocity(x_vel as u8);
                    self.sprite_slot_view_mut(k).set_y_velocity(y_vel as u8);
                    if j == 2 {
                        self.sprite_slot_view_mut(k).increment_ai_state();
                    }
                }
                // $05:DEED-DEF7 (162): the graphics store and the RTS.
                crate::cycle_ledger::charge(162);
            }
            4 => {
                // Uncle_ApplyTelepathyFollower $05:DEF8-DF18 (360).
                crate::cycle_ledger::charge(360);
                self.follower_state_mut().set_indicator(5);
                self.start_shared_message_timer(0x0df3);
                self.save_progress_mut().or_progress_flags(0x10);
                self.sprite_slot_view_mut(k).set_state(0);
                self.follower_link_state_mut().clear_immobilized();
            }
            _ => {}
        }
    }

    // void Uncle_InPassage(int k) {  // 85df19
    pub(super) fn uncle_in_passage(&mut self, k: usize) {
        // Cycle ledger (a jump target): $05:DF19 LDA $D80,x (32), JSL
        // JumpTableLocal (62 + 414) into the two-entry table at $05:DF20.
        crate::cycle_ledger::charge_routine(0x00_8781, 52);
        crate::cycle_ledger::charge(32 + 62 + 414 - 52);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                // $05:DF26 JSL Sprite_CheckDamageToPlayerSameLayer_ : BCC
                // (78); a touch runs $05:DF2C JSL Player_HaltDashAttack_
                // (62), else the BCC is taken (+6).
                if self.sprite_check_damage_to_link_same_layer(k) {
                    crate::cycle_ledger::charge(78 + 62);
                    self.link_cancel_dash();
                } else {
                    crate::cycle_ledger::charge(78 + 6);
                }
                // $05:DF30-DF38 LDA #$0E, LDY #0, JSL Sprite_ShowMessageOnContact,
                // BCC (110); a shown message runs $05:DF3A-DF42 (108), else the
                // BCC is taken (+6); $05:DF43 RTS (42).
                crate::cycle_ledger::charge(110);
                if (self.sprite_show_message_on_contact(k, 0x0e) & 0x100) != 0 {
                    crate::cycle_ledger::charge(108 + 42);
                    self.follower_state_mut().set_indicator(0);
                    self.sprite_slot_view_mut(k).increment_ai_state();
                } else {
                    crate::cycle_ledger::charge(6 + 42);
                }
            }
            1 => {
                // Uncle_GrantEquipment $05:DF44-DF49 LDY #0, STZ $02E9, JSL
                // Link_ReceiveItem (110); the remainder is charged by
                // `complete_uncle_passage_item_receipt`, which also runs
                // when the receipt resumes after a suspension.
                crate::cycle_ledger::charge(110);
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        0,
                        0,
                        ItemReceiptCaller::UnclePassage {
                            sprite_slot: k as u8,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_uncle_passage_item_receipt(k);
            }
            _ => {}
        }
    }

    pub(super) fn complete_uncle_passage_item_receipt(&mut self, k: usize) {
        // $05:DF4D-DF6B (356, RTS included): the Uncle_GrantEquipment tail.
        crate::cycle_ledger::charge(356);
        self.sprite_slot_view_mut(k).increment_ai_state();
        self.sprite_slot_view_mut(k).set_graphics(1);
        self.save_progress_mut().set_which_starting_point(3);
        self.save_progress_mut().or_progress_flags(1);
        self.save_progress_mut().set_progress_indicator(1);
    }

    // void Uncle_Draw(int k) {  // 8dd391
    pub(super) fn uncle_draw(&mut self, k: usize) {
        // Cycle ledger: Uncle_Draw $0D:D391 (JSL target, m8 x8; X <= 15 so
        // no abs,x page crossing). $0D:D391-D3DF: PHB PHK PLB, LDA #$18, JSL
        // Oam_AllocateFromRegionB, the table-address and DMA-index setup,
        // JSL Sprite_DrawMultiple_R6, LDA $DE0,x, BEQ (1008; the callees
        // charge themselves). Direction 0 takes the BEQ (+6) to $0D:D3E9;
        // else $0D:D3E1 CMP #$03 : BEQ (32), taken (+6) for direction 3,
        // else $0D:D3E5 JSL Sprite_DrawShadow_ (62). $0D:D3E9 PLB : RTL (72).
        let _scope = crate::cycle_ledger::routine(0x0d_d391);
        crate::cycle_ledger::charge(1008);
        self.oam_allocate_from_region_b(0x18);
        let direction = self.sprite_slot_view(k).direction();
        let graphics = self.sprite_slot_view(k).graphics();

        let Some(plan) = uncle_draw_plan(direction, graphics) else {
            return;
        };
        self.follower_link_state_mut()
            .set_sword_dma_graphics_index(plan.equipment.sword);
        self.follower_link_state_mut()
            .set_shield_dma_graphics_index(plan.equipment.shield);
        let mut info = match plan.source {
            UncleDrawSource::PortedTable { start } => self.sprite_draw_multiple_from(
                k,
                &UNCLE_DRAW_FRAMES[start..start + 6],
                super::sprite::DrawMultipleEntry::R6,
            ),
            UncleDrawSource::WrappedWram { address } => {
                // The final departure step gives the ROM direction 0xbd. Its
                // 16-bit table calculation wraps from $0d:d203 to low WRAM at
                // $18e3, so the generic routine consumes six live WRAM records.
                self.sprite_draw_multiple_from_wram_records::<6>(
                    k,
                    address,
                    super::sprite::DrawMultipleEntry::R6,
                )
            }
        };
        if direction != 0 && direction != 3 {
            crate::cycle_ledger::charge(32 + 62 + 72);
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        } else if direction == 0 {
            crate::cycle_ledger::charge(6 + 72);
        } else {
            crate::cycle_ledger::charge(32 + 6 + 72);
        }
    }

    // void Sprite_SanctuaryMantle(int k) {  // 85db9b
    pub(super) fn sprite_sanctuary_mantle(&mut self, k: usize) {
        // Cycle ledger (a jump target of Sprite_UncleAndSage): $05:DB9B JSR
        // SageMantle_Draw (46), $05:DB9E JSR Sprite_ReturnIfInactive_ (46).
        crate::cycle_ledger::charge(46);
        self.sage_mantle_draw(k);
        crate::cycle_ledger::charge(46);
        if self.sprite_return_if_inactive_bank5(k) {
            return;
        }

        let mut collision = false;
        if self.sprite_slot_view(k).c() != 0 {
            // $05:DBA1 LDA $DB0,x : BNE (48, taken +6) into
            // SageMantle_SlidingRight $05:DBE3-DBEE LDA #$40, STA $D90,x, LDA
            // $D80,x, JSL JumpTableLocal (148 + 414) into the state table at
            // $05:DBEF.
            crate::cycle_ledger::charge_routine(0x00_8781, 52);
            crate::cycle_ledger::charge(48 + 6 + 148 + 414 - 52);
            self.sprite_slot_view_mut(k).set_a(0x40);
            collision = true;
        } else if self.sprite_check_damage_to_link_same_layer(k) {
            // $05:DBA6 JSL Sprite_CheckDamageToPlayerSameLayer_ : BCC (78),
            // $05:DBAC-DBB8 (202: the hookshot/dash callees and the
            // delay store), then the shared $05:DBBB-DBCC block (212: STZ
            // $E80,x, the $48/$5E stores, LDA $D80,x, JSL JumpTableLocal)
            // and JumpTableLocal (414) into the state table at $05:DBCD.
            crate::cycle_ledger::charge_routine(0x00_8781, 52);
            crate::cycle_ledger::charge(48 + 78 + 202 + 212 + 414 - 52);
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.sprite_repel_dash();
            self.sprite_slot_view_mut(k).set_delay_aux1(7);
            collision = true;
        } else if self.sprite_slot_view(k).delay_aux1() != 0 {
            // BCC taken (+6) into SageMantle_NoPlayerCollision $05:DBD3 LDA
            // $E00,x : BNE (48, taken +6) back into the shared $05:DBBB
            // block (212) and JumpTableLocal (414).
            crate::cycle_ledger::charge_routine(0x00_8781, 52);
            crate::cycle_ledger::charge(48 + 78 + 6 + 48 + 6 + 212 + 414 - 52);
            self.sprite_slot_view_mut(k).set_subtype2(0);
            self.follower_link_state_mut().set_defense_flags(0x81);
            self.follower_link_state_mut().set_speed_setting(8);
            collision = true;
        } else {
            // No collision and no delay: $05:DBD8 LDA $E80,x, JSL
            // JumpTableLocal (94 + 414) into the two-entry table at
            // $05:DBDF.
            crate::cycle_ledger::charge_routine(0x00_8781, 52);
            crate::cycle_ledger::charge(48 + 78 + 6 + 48 + 94 + 414 - 52);
        }

        if collision {
            if self.sprite_slot_view(k).c() == 0 {
                self.sprite_slot_view_mut(k).set_subtype2(0);
                self.follower_link_state_mut().set_defense_flags(0x81);
                self.follower_link_state_mut().set_speed_setting(8);
            }
            match self.sprite_slot_view(k).ai_state() {
                0 => {
                    // Sprite_SanctuaryMantle_AttemptCutscene $05:DC00-DC20
                    // (440, the JSR Sprite_DirectionToFacePlayer__ included;
                    // ends CPY #$03 : BEQ, taken +6 for direction 3, else
                    // $05:DC22 CPY #$01 : BNE, 32, taken +6 unless direction
                    // 1). Facing runs $05:DC26-DC2E (116; a count of 64 runs
                    // $05:DC30-DC37, 100, else the BCC is taken +6).
                    // $05:DC38 RTS (42).
                    crate::cycle_ledger::charge(440);
                    let x = self.sprite_get_x(k);
                    self.sprite_set_x(k, x.wrapping_add(19));
                    let dir = self.sprite_direction_to_face_link(k);
                    self.sprite_set_x(k, x);
                    crate::cycle_ledger::charge(match dir {
                        3 => 6,
                        1 => 32,
                        _ => 32 + 6,
                    });
                    if dir == 1 || dir == 3 {
                        self.sprite_slot_view_mut(k).increment_a();
                        if self.sprite_slot_view(k).a() >= 64 {
                            crate::cycle_ledger::charge(116 + 100);
                            self.sprite_slot_view_mut(k).increment_ai_state();
                            self.follower_link_state_mut().immobilize();
                        } else {
                            crate::cycle_ledger::charge(116 + 6);
                        }
                    }
                    crate::cycle_ledger::charge(42);
                }
                1 => {
                    // Sprite_SanctuaryMantle_InitializeSlide $05:DC39-DC51
                    // (334).
                    crate::cycle_ledger::charge(334);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 24);
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_slot_view_mut(k).set_delay_main(168);
                    self.sprite_slot_view_mut(k).set_x_velocity(3);
                    self.sprite_slot_view_mut(k).set_delay_aux1(2);
                }
                2 => {
                    // Sprite_SanctuaryMantle_SlideToTheRight $05:DC52-DC58
                    // JSR Sprite_Move_, LDA $DF0,x, BNE (94); an expired
                    // delay runs $05:DC5A-DC63 (150), else the BNE is taken
                    // (+6) into $05:DC64-DC69 (96).
                    crate::cycle_ledger::charge(94);
                    self.sprite_move_xy(k);
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        crate::cycle_ledger::charge(150);
                        self.follower_link_state_mut().clear_immobilized();
                        self.sprite_slot_view_mut(k).set_x_velocity(0);
                        self.sprite_slot_view_mut(k).set_c(0);
                    } else {
                        crate::cycle_ledger::charge(6 + 96);
                        self.sprite_slot_view_mut(k).set_delay_aux1(2);
                    }
                }
                _ => {}
            }
        } else {
            match self.sprite_slot_view(k).subtype2() {
                0 => {
                    // $05:DBF5-DBFF (180, RTS included).
                    crate::cycle_ledger::charge(180);
                    self.sprite_slot_view_mut(k).set_a(0);
                    self.follower_link_state_mut().clear_defense_flags();
                    self.follower_link_state_mut().set_speed_setting(0);
                    self.sprite_slot_view_mut(k).increment_subtype2();
                }
                1 => {
                    // $05:DBFF RTS (42).
                    crate::cycle_ledger::charge(42);
                }
                _ => {}
            }
        }
    }

    // void Sprite_Priest(int k) {  // 85dce6
    pub(super) fn sprite_priest(&mut self, k: usize) {
        // Cycle ledger (a jump target of Sprite_UncleAndSage): $05:DCE6 LDA
        // $D90,x : BNE (48); a zero runs $05:DCEB JSL Priest_Draw (62), else
        // the BNE is taken (+6).
        if self.sprite_slot_view(k).a() == 0 {
            crate::cycle_ledger::charge(48 + 62);
            self.priest_draw(k);
        } else {
            crate::cycle_ledger::charge(48 + 6);
        }
        // $05:DCEF JSR Sprite_ReturnIfInactive_ (46).
        crate::cycle_ledger::charge(46);
        if self.sprite_return_if_inactive_bank5(k) {
            return;
        }
        // $05:DCF2 JSL Sprite_BehaveAsBarrier (62), $05:DCF6 JSL
        // Sprite_TrackBodyToHead (62), $05:DCFA JSR Sprite_Move_ (46; the ROM
        // always calls it — the translation's guard is the C port's),
        // $05:DCFD LDA $E80,x (32), JSL JumpTableLocal (62 + 414) into the
        // three-entry table at $05:DD04.
        crate::cycle_ledger::charge(62);
        self.sprite_behave_as_barrier(k);
        crate::cycle_ledger::charge(62 + 46);
        if self.sprite_track_body_to_head(k) {
            self.sprite_move_xy(k);
        }
        crate::cycle_ledger::charge_routine(0x00_8781, 52);
        crate::cycle_ledger::charge(32 + 62 + 414 - 52);
        match self.sprite_slot_view(k).subtype2() {
            0 => self.priest_dying(k),
            1 => self.priest_run_rescue_cutscene(k),
            2 => self.priest_chillin(k),
            _ => {}
        }
    }

    // void Priest_SpawnMantle(int k) {  // sprite_main.c:5473
    //   sprite_state[15]++;
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x73, &info);
    //   sprite_state[15] = 0;
    //   sprite_flags2[j] = sprite_flags2[j] & 0xf0 | 0x3;
    //   sprite_x_lo[j] = 0xF0; sprite_x_hi[j] = 4;
    //   sprite_y_lo[j] = 0x37; sprite_y_hi[j] = 2;
    //   sprite_E[j] = 2;
    //   sprite_flags4[j] = 11;
    //   sprite_defl_bits[j] |= 0x20;
    //   sprite_subtype2[j] = 1;
    //   if (link_y_coord < Sprite_GetY(j))
    //     sprite_C[j] = 1;
    // }
    pub(super) fn priest_spawn_mantle(&mut self, k: usize) {
        let marker_state = self.sprite_slot_view(15).state().wrapping_add(1);
        self.sprite_slot_view_mut(15).set_state(marker_state);
        let j = self.spawn_sprite_dynamically(k, 0x73).map(|(j, _)| j);
        self.sprite_slot_view_mut(15).set_state(0);
        let j = j.expect("Priest_SpawnMantle expected Sprite_SpawnDynamically to succeed");
        self.sprite_slot_view_mut(j).masked_or_flags2(0xf0, 0x3);
        self.sprite_slot_view_mut(j).set_x_low(0xF0);
        self.sprite_slot_view_mut(j).set_x_high(4);
        self.sprite_slot_view_mut(j).set_y_low(0x37);
        self.sprite_slot_view_mut(j).set_y_high(2);
        self.sprite_slot_view_mut(j).set_e(2);
        self.sprite_slot_view_mut(j).set_flags4(11);
        self.sprite_slot_view_mut(j).or_deflection_bits(0x20);
        self.sprite_slot_view_mut(j).set_subtype2(1);
        let link_y = self.game_state.player.follower_link.y();
        if link_y < self.sprite_get_y(j) {
            self.sprite_slot_view_mut(j).set_c(1);
        }
    }

    // void Priest_Dying(int k) {  // sprite_main.c:5580
    //   sprite_head_dir[k] = 4;
    //   sprite_D[k] = 4;
    //   switch (sprite_ai_state[k]) {
    //   case 0:  // Priest_LyingOnGround
    //     if (Sprite_ShowSolicitedMessage(k, 0x1b) & 0x100) {
    //       sprite_ai_state[k]++;
    //       sprite_graphics[k]++;
    //       sram_progress_flags |= 0x2;
    //       sprite_delay_aux2[k] = 128;
    //     }
    //     break;
    //   case 1:  // Priest_FinalWords
    //     sprite_graphics[k] = 0;
    //     if (sprite_delay_aux2[k] == 0)
    //       sprite_ai_state[k]++;
    //     sprite_A[k] = frame_counter & 2;
    //     if (!(sprite_delay_aux2[k] & 7))
    //       SpriteSfx_QueueSfx2WithPan(k, 0x33);
    //     break;
    //   case 2:  // Priest_Die
    //     sprite_state[k] = 0;
    //     break;
    //   }
    // }
    pub(super) fn priest_dying(&mut self, k: usize) {
        // Cycle ledger (a jump target of Sprite_Priest): $05:DD0A-DD0F LDA
        // #$04, STA $EB0,x, STA $DE0,x (92), $05:DD12 LDA $D80,x, JSL
        // JumpTableLocal (94 + 414) into the three-entry table at $05:DD19.
        crate::cycle_ledger::charge_routine(0x00_8781, 52);
        crate::cycle_ledger::charge(92 + 94 + 414 - 52);
        self.sprite_slot_view_mut(k).set_head_direction(4);
        self.sprite_slot_view_mut(k).set_direction(4);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                // Priest_LyingOnGround $05:DD1F-DD27 LDA #$1B, LDY #0, JSL
                // Sprite_ShowSolicitedMessage, BCC (110); a shown message
                // runs $05:DD29-DD3D (254), else the BCC is taken (+6);
                // $05:DD3E RTS (42).
                crate::cycle_ledger::charge(110);
                if (self.sprite_show_solicited_message(k, 0x1b) & 0x100) != 0 {
                    crate::cycle_ledger::charge(254 + 42);
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_slot_view_mut(k).increment_graphics();
                    self.save_progress_mut().or_progress_flags(0x2);
                    self.sprite_slot_view_mut(k).set_delay_aux2(128);
                } else {
                    crate::cycle_ledger::charge(6 + 42);
                }
            }
            1 => {
                // Priest_FinalWords $05:DD3F-DD45 STZ $DC0,x, LDA $E10,x, BNE
                // (86); an expired timer runs $05:DD47 INC $D80,x (52), else
                // the BNE is taken (+6). $05:DD4A-DD56 (142) ends BNE $DD5E:
                // every eighth tick runs $05:DD58 LDA #$33, JSL
                // SpriteSfx_QueueSfx2WithPan (78), else it is taken (+6).
                // $05:DD5E RTS (42).
                self.sprite_slot_view_mut(k).set_graphics(0);
                if self.sprite_slot_view(k).delay_aux2() == 0 {
                    crate::cycle_ledger::charge(86 + 52);
                    self.sprite_slot_view_mut(k).increment_ai_state();
                } else {
                    crate::cycle_ledger::charge(86 + 6);
                }
                let a = self.game_state.frame.frame_counter & 2;
                self.sprite_slot_view_mut(k).set_a(a);
                if (self.sprite_slot_view(k).delay_aux2() & 7) == 0 {
                    crate::cycle_ledger::charge(142 + 78 + 42);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x33);
                } else {
                    crate::cycle_ledger::charge(142 + 6 + 42);
                }
            }
            2 => {
                // Priest_Die $05:DD5F-DD62 STZ $DD0,x : RTS (80).
                crate::cycle_ledger::charge(80);
                self.sprite_slot_view_mut(k).set_state(0);
            }
            _ => {}
        }
    }

    // void Priest_RunRescueCutscene(int k) {  // sprite_main.c:5606
    //   int j;
    //   switch (sprite_ai_state[k]) {
    //   case 0:
    //     sprite_head_dir[k] = 0;
    //     sprite_D[k] = 0;
    //     if (sprite_delay_main[k] == 0) {
    //       Sprite_ShowMessageUnconditional(0x17);
    //       sprite_ai_state[k]++;
    //       byte_7FFE01 = 1;
    //       Priest_SpawnRescuedPrincess();
    //       flag_is_link_immobilized = 1;
    //       savegame_map_icons_indicator = 1;
    //     }
    //     break;
    //   case 1:
    //     if (byte_7FFE01 == 2) {
    //       Sprite_ShowMessageUnconditional(0x18);
    //       sprite_ai_state[k]++;
    //     }
    //     break;
    //   case 2:
    //     if (choice_in_multiselect_box == 0) {
    //       sprite_ai_state[k]++;
    //       flag_is_link_immobilized = 0;
    //     } else {
    //       sprite_ai_state[k] = 1;
    //     }
    //     break;
    //   case 3:
    //     sprite_head_dir[k] = Sprite_DirectionToFaceLink(k, NULL) ^ 3;
    //     j = Sprite_ShowSolicitedMessage(k, 0x16);
    //     if (j & 0x100)
    //       sprite_D[k] = sprite_head_dir[k] = (uint8)j;
    //     break;
    //   }
    // }
    pub(super) fn priest_run_rescue_cutscene(&mut self, k: usize) {
        // Cycle ledger (a jump target of Sprite_Priest): $05:DD63 LDA $D80,x,
        // JSL JumpTableLocal (94 + 414) into the four-entry table at
        // $05:DD6A.
        crate::cycle_ledger::charge_routine(0x00_8781, 52);
        crate::cycle_ledger::charge(94 + 414 - 52);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                // $05:DD72-DD7D LDA #0, STA $EB0,x, STA $DE0,x, LDA $DF0,x,
                // BNE (140; taken +6 while the delay runs); an expired delay
                // runs $05:DD7F-DD9D (352, the JSL/JSR callees charge
                // themselves). $05:DD9E RTS (42).
                self.sprite_slot_view_mut(k).set_head_direction(0);
                self.sprite_slot_view_mut(k).set_direction(0);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    crate::cycle_ledger::charge(140 + 352 + 42);
                    self.sprite_show_message_unconditional(0x17);
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.follower_state_mut().set_zelda_rescue_cutscene_state(1);
                    self.priest_spawn_rescued_princess();
                    self.follower_link_state_mut().immobilize();
                    self.save_progress_mut().set_map_icons_indicator(1);
                } else {
                    crate::cycle_ledger::charge(140 + 6 + 42);
                }
            }
            1 => {
                // $05:DD9F-DDA5 LDA $7FFE01, CMP #$02, BNE (72); state 2 runs
                // $05:DDA7-DDB1 (146), else the BNE is taken (+6). $05:DDB2
                // RTS (42).
                if self
                    .game_state
                    .sprites
                    .follower_runtime
                    .zelda_rescue_cutscene_state()
                    == 2
                {
                    crate::cycle_ledger::charge(72 + 146 + 42);
                    self.sprite_show_message_unconditional(0x18);
                    self.sprite_slot_view_mut(k).increment_ai_state();
                } else {
                    crate::cycle_ledger::charge(72 + 6 + 42);
                }
            }
            2 => {
                // $05:DDB3 LDA $1CE8 : BNE (48); choice 0 runs $05:DDB8-DDBE
                // (126, RTS included), else the BNE is taken (+6) into
                // $05:DDBF-DDC4 (96, RTS included).
                if self.multiselect_choice().value_word() == 0 {
                    crate::cycle_ledger::charge(48 + 126);
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.follower_link_state_mut().clear_immobilized();
                } else {
                    crate::cycle_ledger::charge(48 + 6 + 96);
                    self.sprite_slot_view_mut(k).set_ai_state(1);
                }
            }
            3 => {
                // $05:DDC5-DDD6 JSR Sprite_DirectionToFacePlayer__, TYA, EOR
                // #$03, STA $EB0,x, LDA #$16, LDY #0, JSL
                // Sprite_ShowSolicitedMessage, BCC (224); a shown message
                // runs $05:DDD8-DDDD (76), else the BCC is taken (+6);
                // $05:DDDE RTS (42).
                crate::cycle_ledger::charge(224);
                let head_direction = self.sprite_direction_to_face_link(k) ^ 3;
                self.sprite_slot_view_mut(k)
                    .set_head_direction(head_direction);
                let j = self.sprite_show_solicited_message(k, 0x16);
                if (j & 0x100) != 0 {
                    crate::cycle_ledger::charge(76 + 42);
                    let v = j as u8;
                    self.sprite_slot_view_mut(k).set_direction(v);
                    self.sprite_slot_view_mut(k).set_head_direction(v);
                } else {
                    crate::cycle_ledger::charge(6 + 42);
                }
            }
            _ => {}
        }
    }

    // void Priest_Chillin(int k) {  // sprite_main.c:5644
    //   sprite_head_dir[k] = Sprite_DirectionToFaceLink(k, NULL) ^ 3;
    //   int m = (link_which_pendants & 7) == 7 ? 0x1a :
    //           savegame_map_icons_indicator >= 3 ? 0x19 : 0x16;
    //   int j = Sprite_ShowSolicitedMessage(k, m);
    //   if (j & 0x100) {
    //     sprite_D[k] = sprite_head_dir[k] = (uint8)j;
    //     link_hearts_filler = 0xa0;
    //   }
    // }
    pub(super) fn priest_chillin(&mut self, k: usize) {
        // Cycle ledger (a jump target of Sprite_Priest): Priest_Chillin
        // $05:DDE5-DDF8 JSR Sprite_DirectionToFacePlayer__, TYA, EOR #$03,
        // STA $EB0,x, LDY #0, LDA $7EF374, AND #$07, CMP #$07, BNE (218).
        // All pendants: $05:DDFA LDY #$02 : BRA (38); else the BNE is taken
        // (+6) into $05:DDFE LDA $7EF3C7, CMP #$03, BCC (72; three map icons
        // run $05:DE06 LDY #$01, 16, else the BCC is taken +6). $05:DE08-DE15
        // (196) loads the message and JSLs Sprite_ShowSolicitedMessage; a
        // shown message runs $05:DE17-DE22 (132), else the BCC is taken (+6);
        // $05:DE23 RTS (42).
        crate::cycle_ledger::charge(218);
        let head_direction = self.sprite_direction_to_face_link(k) ^ 3;
        self.sprite_slot_view_mut(k)
            .set_head_direction(head_direction);
        let m: u16 = if (self.game_state.inventory.player_resources.pendant_flags() & 7) == 7 {
            crate::cycle_ledger::charge(38);
            0x1a
        } else if self
            .game_state
            .inventory
            .save_progress
            .map_icons_indicator()
            >= 3
        {
            crate::cycle_ledger::charge(6 + 72 + 16);
            0x19
        } else {
            crate::cycle_ledger::charge(6 + 72 + 6);
            0x16
        };
        crate::cycle_ledger::charge(196);
        let j = self.sprite_show_solicited_message(k, m);
        if (j & 0x100) != 0 {
            crate::cycle_ledger::charge(132 + 42);
            let v = j as u8;
            self.sprite_slot_view_mut(k).set_direction(v);
            self.sprite_slot_view_mut(k).set_head_direction(v);
            self.player_resources_mut().set_heart_filler(0xa0);
        } else {
            crate::cycle_ledger::charge(6 + 42);
        }
    }

    // void Sprite_QuarrelBros(int k) {  // 85e013
    pub(super) fn sprite_quarrel_bros(&mut self, k: usize) {
        self.quarrel_bros_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_track_body_to_head(k);
        let head_direction = self.sprite_direction_to_face_link(k) ^ 3;
        self.sprite_slot_view_mut(k)
            .set_head_direction(head_direction);
        if (self.game_state.world.location.dungeon_room_index() & 1) == 0 {
            self.sprite_show_solicited_message(k, 0x131);
        } else if (self.game_state.dungeon.doors.opened_doors() & 0xff00) == 0 {
            self.sprite_show_solicited_message(k, 0x12f);
        } else {
            self.sprite_show_solicited_message(k, 0x130);
        }
        self.sprite_behave_as_barrier(k);
    }

    // void Priest_SpawnRescuedPrincess() {  // sprite_main.c:6236
    //   SpriteSpawnInfo info;
    //   int k = Sprite_SpawnDynamically(0, 0x76, &info);
    //   if (k < 0) return;
    //   sprite_D[k] = sprite_head_dir[k] = tagalong_layerbits[tagalong_var2] & 3;
    //   Sprite_SetX(k, link_x_coord);
    //   Sprite_SetY(k, link_y_coord);
    //   sprite_subtype2[k] = 1;
    //   follower_indicator = 0;
    //   sprite_ignore_projectile[k]++;
    //   sprite_flags4[k] = 3;
    // }
    pub(super) fn priest_spawn_rescued_princess(&mut self) {
        let Some((k, _)) = self.spawn_sprite_dynamically(0, 0x76) else {
            return;
        };
        let tag_idx = self.game_state.sprites.follower_runtime.data_index() as usize;
        let layer_bits = self.tagalong_slot(tag_idx).direction();
        self.sprite_slot_view_mut(k).set_direction(layer_bits);
        self.sprite_slot_view_mut(k).set_head_direction(layer_bits);
        let lx = self.game_state.player.follower_link.x();
        let ly = self.game_state.player.follower_link.y();
        self.sprite_set_x(k, lx);
        self.sprite_set_y(k, ly);
        self.sprite_slot_view_mut(k).set_subtype2(1);
        self.follower_state_mut().set_indicator(0);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        self.sprite_slot_view_mut(k).set_flags4(3);
    }

    // void SpritePrep_UncleAndPriest_bounce(int k) {  // sprite_main.c:10859
    //   if (BYTE(dungeon_room_index) == 18) {
    //     Priest_SpawnMantle(k);
    //     if (sram_progress_indicator >= 3)
    //       sram_progress_flags |= 2;
    //     if (sram_progress_flags & 2) {
    //       sprite_state[k] = 0;
    //       return;
    //     }
    //     sprite_E[k] = 1;
    //     sprite_flags2[k] = sprite_flags2[k] & 0xf0 | 0x2;
    //     sprite_flags4[k] = 3;
    //     int j;
    //     if (link_sword_type >= 2) {
    //       sprite_D[k] = 4; sprite_graphics[k] = 0; j = 0;
    //     } else {
    //       sprite_D[k] = sprite_head_dir[k] = Sprite_DirectionToFaceLink(k, NULL) ^ 3;
    //       if (follower_indicator == 1) {
    //         sram_progress_flags |= 0x4;
    //         save_ow_event_info[0x1b] |= 0x20;
    //         sprite_delay_main[k] = 170;
    //         j = 1;
    //       } else {
    //         j = 2;
    //       }
    //     }
    //     sprite_subtype2[k] = j;
    //     static const int16 kUncleAndSage_Y[3] = {0, -9, 0};
    //     Sprite_SetX(k, Sprite_GetX(k) - 6);
    //     Sprite_SetY(k, Sprite_GetY(k) + kUncleAndSage_Y[j]);
    //     sprite_ignore_projectile[k]++;
    //     byte_7FFE01 = 0;
    //   } else if (BYTE(dungeon_room_index) == 4) {
    //     if (!(sram_progress_flags & 0x10))
    //       sprite_x_lo[k] += 8;
    //     else
    //       sprite_state[k] = 0;
    //   } else {
    //     if (!(sram_progress_flags & 1)) {
    //       sprite_D[k] = 3;
    //       sprite_subtype2[k] = 1;
    //     } else {
    //       sprite_state[k] = 0;
    //     }
    //   }
    // }
    pub(super) fn sprite_prep_uncle_and_priest_bounce(&mut self, k: usize) {
        let room = self.game_state.world.location.dungeon_room_index();
        if room == 18 {
            self.priest_spawn_mantle(k);
            if self.game_state.inventory.save_progress.progress_indicator() >= 3 {
                self.save_progress_mut().or_progress_flags(2);
            }
            if self.game_state.inventory.save_progress.progress_flags() & 2 != 0 {
                self.sprite_slot_view_mut(k).set_state(0);
                return;
            }
            self.sprite_slot_view_mut(k).set_e(1);
            self.sprite_slot_view_mut(k).masked_or_flags2(0xf0, 0x2);
            self.sprite_slot_view_mut(k).set_flags4(3);
            let j: usize;
            if self.game_state.inventory.items.sword_type() >= 2 {
                self.sprite_slot_view_mut(k).set_direction(4);
                self.sprite_slot_view_mut(k).set_graphics(0);
                j = 0;
            } else {
                let v = self.sprite_direction_to_face_link(k) ^ 3;
                self.sprite_slot_view_mut(k).set_direction(v);
                self.sprite_slot_view_mut(k).set_head_direction(v);
                if self.game_state.sprites.follower_runtime.indicator() == 1 {
                    self.save_progress_mut().or_progress_flags(0x4);
                    self.set_overworld_event_bits(0x1b, 0x20);
                    self.sprite_slot_view_mut(k).set_delay_main(170);
                    j = 1;
                } else {
                    j = 2;
                }
            }
            self.sprite_slot_view_mut(k).set_subtype2(j as u8);
            let x = self.sprite_get_x(k);
            self.sprite_set_x(k, x.wrapping_sub(6));
            let y = self.sprite_get_y(k);
            let dy = UNCLE_AND_SAGE_Y[j] as u16;
            self.sprite_set_y(k, y.wrapping_add(dy));
            self.sprite_slot_view_mut(k).increment_ignore_projectile();
            self.follower_state_mut().set_zelda_rescue_cutscene_state(0);
        } else if room == 4 {
            if (self.game_state.inventory.save_progress.progress_flags() & 0x10) == 0 {
                let x_low = self.sprite_slot_view(k).x_low().wrapping_add(8);
                self.sprite_slot_view_mut(k).set_x_low(x_low);
            } else {
                self.sprite_slot_view_mut(k).set_state(0);
            }
        } else if (self.game_state.inventory.save_progress.progress_flags() & 1) == 0 {
            self.sprite_slot_view_mut(k).set_direction(3);
            self.sprite_slot_view_mut(k).set_subtype2(1);
        } else {
            self.sprite_slot_view_mut(k).set_state(0);
        }
    }

    // void Priest_Draw(int k) {  // sprite_main.c:13022
    //   int j = sprite_D[k] * 2 + sprite_graphics[k];
    //   PrepOamCoordsRet info;
    //   Sprite_DrawMultiplePlayerDeferred(k, kPriest_Dmd + j * 2, 2, &info);
    //   SpriteDraw_Shadow(k, &info);
    // }
    pub(super) fn priest_draw(&mut self, k: usize) {
        // Cycle ledger: Priest_Draw $0D:CF31 (JSL target, m8 x8; X <= 15 so
        // no abs,x page crossing), one straight block $0D:CF31-CF58: PHB PHK
        // PLB, the table-address setup, JSL Sprite_DrawMultiplePlayerDeferred,
        // JSL Sprite_DrawShadow_, PLB, RTL (562; the callees charge
        // themselves).
        let _scope = crate::cycle_ledger::routine(0x0d_cf31);
        crate::cycle_ledger::charge(562);
        let j = (self.sprite_slot_view(k).direction() as usize) * 2
            + self.sprite_slot_view(k).graphics() as usize;
        let base = j * 2;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &PRIEST_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // ----- Thief cluster ------------------------------------------------

    // void Sprite_C4_Thief(int k) {  // 9dc8d8
    pub(super) fn sprite_c4_thief(&mut self, k: usize) {
        self.thief_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_from_link(k);
        if self.sprite_slot_view(k).ai_state() != 3 {
            let j = self.sprite_direction_to_face_link(k);
            self.sprite_slot_view_mut(k).set_head_direction(j);
            if (j ^ self.sprite_slot_view(k).direction()) == 1 {
                self.sprite_slot_view_mut(k).set_direction(j);
            }
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.thief_check_collision_with_link(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let link_x = self.game_state.player.follower_link.x();
                    let link_y = self.game_state.player.follower_link.y();
                    let cur_x = self.game_state.sprites.workspace.current_sprite_x();
                    let cur_y = self.game_state.sprites.workspace.current_sprite_y();
                    if link_x.wrapping_sub(cur_x).wrapping_add(0x50) < 0xa0
                        && link_y.wrapping_sub(cur_y).wrapping_add(0x50) < 0xa0
                    {
                        self.sprite_slot_view_mut(k).increment_ai_state();
                        self.sprite_slot_view_mut(k).set_delay_main(16);
                    }
                }
                let graphics = THIEF_GFX[usize::from(self.sprite_slot_view(k).direction())];
                self.sprite_slot_view_mut(k).set_graphics(graphics);
            }
            1 => {
                self.thief_check_collision_with_link(k);
                let dir = self.sprite_direction_to_face_link(k);
                self.sprite_slot_view_mut(k).set_direction(dir);
                self.sprite_slot_view_mut(k).set_head_direction(dir);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                    self.sprite_slot_view_mut(k).set_delay_main(32);
                }
                self.thief_common(k);
            }
            2 => {
                self.sprite_apply_speed_towards_link(k, 18);
                if self.sprite_slot_view(k).wall_collision() == 0 {
                    self.sprite_move_xy(k);
                }
                self.sprite_check_tile_collision(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let link_x = self.game_state.player.follower_link.x();
                    let link_y = self.game_state.player.follower_link.y();
                    let cur_x = self.game_state.sprites.workspace.current_sprite_x();
                    let cur_y = self.game_state.sprites.workspace.current_sprite_y();
                    if link_x.wrapping_sub(cur_x).wrapping_add(0x50) >= 0xa0
                        || link_y.wrapping_sub(cur_y).wrapping_add(0x50) >= 0xa0
                    {
                        self.sprite_slot_view_mut(k).set_ai_state(0);
                        self.sprite_slot_view_mut(k).set_delay_main(128);
                    }
                }
                if self.sprite_check_damage_to_link(k) {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_slot_view_mut(k).set_delay_main(32);
                    self.thief_spill_items(k);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0xb);
                }
                self.thief_common(k);
            }
            3 => {
                self.thief_check_collision_with_link(k);
                let j = self.thief_scan_for_booty(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).increment_subtype2();
                    let i = 4 + self
                        .sprite_slot_view(k)
                        .direction()
                        .wrapping_add(self.sprite_slot_view(k).subtype2() & 4);
                    self.sprite_slot_view_mut(k)
                        .set_graphics(THIEF_GFX[usize::from(i)]);
                    if self.sprite_slot_view(k).wall_collision() == 0 {
                        self.sprite_move_xy(k);
                    }
                    self.sprite_check_tile_collision(k);
                    let direction = self.sprite_slot_view(k).head_direction();
                    self.sprite_slot_view_mut(k).set_direction(direction);
                }
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0 {
                    let j = usize::from(j);
                    let target_x = self.sprite_get_x(j);
                    let target_y = self.sprite_get_y(j);
                    let head_direction =
                        self.sprite_direction_to_face_location(k, target_x, target_y);
                    self.sprite_slot_view_mut(k)
                        .set_head_direction(head_direction);
                }
            }
            _ => {}
        }
    }

    fn thief_common(&mut self, k: usize) {
        if (self.game_state.frame.frame_counter & 31) == 0 {
            let direction = self.sprite_slot_view(k).head_direction();
            self.sprite_slot_view_mut(k).set_direction(direction);
        }
        self.sprite_slot_view_mut(k).increment_subtype2();
        let i = 4 + self
            .sprite_slot_view(k)
            .direction()
            .wrapping_add(self.sprite_slot_view(k).subtype2() & 4);
        self.sprite_slot_view_mut(k)
            .set_graphics(THIEF_GFX[usize::from(i)]);
    }

    // uint8 Thief_ScanForBooty(int k) {  // 9dca24
    pub(super) fn thief_scan_for_booty(&mut self, k: usize) -> u8 {
        for j in (0..=15usize).rev() {
            if self.sprite_slot_view(j).state() != 0 {
                let t = self.sprite_slot_view(j).sprite_type();
                if t == 0xdc || t == 0xe1 || t == 0xd9 {
                    self.thief_target_booty(k, j);
                    return j as u8;
                }
            }
        }
        self.sprite_slot_view_mut(k).set_ai_state(0);
        self.sprite_slot_view_mut(k).set_delay_main(64);
        0xff
    }

    // void Thief_TargetBooty(int k, int j) {  // sprite_main.c:17330
    //   if (!((k ^ frame_counter) & 3)) {
    //     ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLocation(k, Sprite_GetX(j), Sprite_GetY(j), 19);
    //     sprite_x_vel[k] = pt.x;
    //     sprite_y_vel[k] = pt.y;
    //   }
    //   for (j = 15; j >= 0; j--) {
    //     if (!((j ^ frame_counter) & 3 | sprite_delay_aux4[j]) && sprite_state[j] &&
    //         (sprite_type[j] == 0xdc || sprite_type[j] == 0xe1 || sprite_type[j] == 0xd9)) {
    //       Thief_GrabBooty(k, j);
    //     }
    //   }
    // }
    pub(super) fn thief_target_booty(&mut self, k: usize, j_in: usize) {
        let fc = self.game_state.frame.frame_counter as usize;
        if (k ^ fc) & 3 == 0 {
            let tx = self.sprite_get_x(j_in);
            let ty = self.sprite_get_y(j_in);
            let pt = self.sprite_project_speed_towards_location(k, tx, ty, 19);
            self.sprite_slot_view_mut(k).set_x_velocity(pt.x);
            self.sprite_slot_view_mut(k).set_y_velocity(pt.y);
        }
        for j in (0..=15usize).rev() {
            // Note: C uses `!((j ^ fc) & 3 | sprite_delay_aux4[j])` — `|` has
            // lower precedence than `&`, so the parens evaluate to
            // `((j^fc)&3) | aux4[j]`.
            let cond = (((j ^ fc) & 3) as u8) | self.sprite_slot_view(j).delay_aux4();
            if cond == 0 && self.sprite_slot_view(j).state() != 0 {
                let t = self.sprite_slot_view(j).sprite_type();
                if t == 0xdc || t == 0xe1 || t == 0xd9 {
                    self.thief_grab_booty(k, j);
                }
            }
        }
    }

    // void Thief_GrabBooty(int k, int j) {  // sprite_main.c:17344
    //   if ((uint16)(Sprite_GetX(j) - cur_sprite_x + 8) < 16 &&
    //       (uint16)(Sprite_GetY(j) - cur_sprite_y + 12) < 24) {
    //     sprite_state[j] = 0;
    //     int t = sprite_type[j] - 0xd8;
    //     SpriteSfx_QueueSfx3WithPan(t, kAbsorptionSfx[t]);
    //     sprite_delay_main[k] = 14;
    //   }
    // }
    pub(super) fn thief_grab_booty(&mut self, k: usize, j: usize) {
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let dx = self.sprite_get_x(j).wrapping_sub(cur_x).wrapping_add(8);
        let dy = self.sprite_get_y(j).wrapping_sub(cur_y).wrapping_add(12);
        if dx < 16 && dy < 24 {
            self.sprite_slot_view_mut(j).set_state(0);
            let t = self.sprite_slot_view(j).sprite_type().wrapping_sub(0xd8) as usize;
            // Original passes `t` (item index) to QueueSfx3WithPan; the
            // helper uses it to index `sprite_x_lo` for panning — we keep
            // the same semantics by passing the slot index `t`.
            self.sprite_sfx_queue_sfx3_with_pan(t, ABSORPTION_SFX[t]);
            self.sprite_slot_view_mut(k).set_delay_main(14);
        }
    }

    // void Thief_CheckCollisionWithLink(int k) {  // sprite_main.c:17355
    //   if (Sprite_CheckDamageToLink_same_layer(k)) {
    //     ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, 32);
    //     link_actual_vel_y = pt.y;
    //     sprite_y_recoil[k] = pt.y ^ 0xff;
    //     link_actual_vel_x = pt.x;
    //     sprite_x_recoil[k] = pt.x ^ 0xff;
    //     link_incapacitated_timer = 4;
    //     sprite_F[k] = 12;
    //     SpriteSfx_QueueSfx2WithPan(k, 0xb);
    //   }
    // }
    pub(super) fn thief_check_collision_with_link(&mut self, k: usize) {
        if self.sprite_check_damage_to_link_same_layer(k) {
            let pt = self.sprite_project_speed_towards_link(k, 32);
            self.follower_link_state_mut()
                .set_actual_velocity_xy(pt.x, pt.y);
            self.sprite_slot_view_mut(k).set_y_recoil(pt.y ^ 0xff);
            self.sprite_slot_view_mut(k).set_x_recoil(pt.x ^ 0xff);
            self.follower_link_state_mut().set_incapacitated_timer(4);
            self.sprite_slot_view_mut(k).set_f(12);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0xb);
        }
    }

    // void Thief_SpillItems(int k) {  // sprite_main.c:17368
    //   static const uint8 kThiefSpawn_Items[4] = {0xd9, 0xe1, 0xdc, 0xd9};
    //   static const int8 kThiefSpawn_Xvel[6] = {0, 24, 24, 0, -24, -24};
    //   static const int8 kThiefSpawn_Yvel[6] = {-32, -16, 16, 32, 16, -16};
    //   tmp_counter = 5;
    //   do {
    //     SPRITE_SHARED_WORK_A = GetRandomNumber() & 3;
    //     int j;
    //     if (SPRITE_SHARED_WORK_A == 1) j = link_num_arrows;
    //     else if (SPRITE_SHARED_WORK_A == 2) j = link_item_bombs;
    //     else j = link_rupees_goal;
    //     if (!j) return;
    //     SpriteSpawnInfo info;
    //     j = Sprite_SpawnDynamicallyEx(k, kThiefSpawn_Items[SPRITE_SHARED_WORK_A], &info, 7);
    //     if (j < 0) return;
    //     if (SPRITE_SHARED_WORK_A == 1) link_num_arrows--;
    //     else if (SPRITE_SHARED_WORK_A == 2) link_item_bombs--;
    //     else link_rupees_goal--;
    //     Sprite_SetX(j, link_x_coord);
    //     Sprite_SetY(j, link_y_coord);
    //     sprite_z_vel[j] = 0x18;
    //     sprite_x_vel[j] = kThiefSpawn_Xvel[tmp_counter];
    //     sprite_y_vel[j] = kThiefSpawn_Yvel[tmp_counter];
    //     sprite_delay_aux4[j] = 32;
    //     sprite_head_dir[j] = 1;
    //     sprite_stunned[j] = 255;
    //   } while (!sign8(--tmp_counter));
    // }
    pub(super) fn thief_spill_items(&mut self, k: usize) {
        self.temp_counter_mut().set(5);
        loop {
            let pick = self.get_random_number() & 3;
            self.sprite_workspace_mut().set_shared_scratch_a(pick);
            let count: u16 = if pick == 1 {
                self.game_state.inventory.player_resources.arrows() as u16
            } else if pick == 2 {
                self.game_state.inventory.player_resources.bombs() as u16
            } else {
                self.game_state.inventory.player_resources.rupees_goal()
            };
            if count == 0 {
                return;
            }
            let Some((j, _)) =
                self.spawn_sprite_dynamically_ex(k, THIEF_SPAWN_ITEMS[pick as usize], 7)
            else {
                return;
            };
            if pick == 1 {
                self.player_resources_mut().decrement_arrows();
            } else if pick == 2 {
                self.player_resources_mut().decrement_bombs();
            } else {
                let cur = self.game_state.inventory.player_resources.rupees_goal();
                self.player_resources_mut()
                    .set_rupees_goal(cur.wrapping_sub(1));
            }
            let lx = self.game_state.player.follower_link.x();
            let ly = self.game_state.player.follower_link.y();
            self.sprite_set_x(j, lx);
            self.sprite_set_y(j, ly);
            self.sprite_slot_view_mut(j).set_z_velocity(0x18);
            let tc = self.game_state.scratch_counter.value() as usize;
            self.sprite_slot_view_mut(j)
                .set_x_velocity(THIEF_SPAWN_XVEL[tc] as u8);
            self.sprite_slot_view_mut(j)
                .set_y_velocity(THIEF_SPAWN_YVEL[tc] as u8);
            self.sprite_slot_view_mut(j).set_delay_aux4(32);
            self.sprite_slot_view_mut(j).set_head_direction(1);
            self.sprite_slot_view_mut(j).set_stunned(255);
            // `--tmp_counter` then `!sign8(...)` continues while non-negative.
            let new_tc = self.temp_counter_mut().decrement();
            if (new_tc as i8) < 0 {
                break;
            }
        }
    }

    // void Thief_Draw(int k) {  // sprite_main.c:17407
    //   static const DrawMultipleData kThief_Dmd[24] = { ... };
    //   static const uint8 kThief_DrawChar[4] = {2, 2, 0, 4};
    //   static const uint8 kThief_DrawFlags[4] = {0x40, 0, 0, 0};
    //   PrepOamCoordsRet info;
    //   Sprite_DrawMultiple(k, &kThief_Dmd[sprite_graphics[k] * 2], 2, &info);
    //   if (!sprite_pause[k]) {
    //     OamEnt *oam = GetOamCurPtr();
    //     int j = sprite_head_dir[k];
    //     oam->charnum = kThief_DrawChar[j];
    //     oam->flags = (oam->flags & ~0x40) | kThief_DrawFlags[j];
    //     SpriteDraw_Shadow(k, &info);
    //   }
    // }
    pub(super) fn thief_draw(&mut self, k: usize) {
        let gfx = self.sprite_slot_view(k).graphics() as usize;
        let mut info = self.sprite_draw_multiple(k, &THIEF_DRAW_FRAMES[gfx * 2..gfx * 2 + 2]);
        if self.sprite_slot_view(k).pause() == 0 {
            self.thief_draw_apply_head_overrides_for_dn(k);
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        }
    }

    // void NiceThief_Animate(int k) {  // sprite_main.c:25047
    //   if (!(frame_counter & 3)) {
    //     sprite_graphics[k] = 2;
    //     uint8 dir = Sprite_DirectionToFaceLink(k, NULL);
    //     sprite_head_dir[k] = (dir == 3) ? 2 : dir;
    //   }
    //   Oam_AllocateDeferToPlayer(k);
    //   Thief_Draw(k);
    // }
    pub(super) fn nice_thief_animate(&mut self, k: usize) {
        if (self.game_state.frame.frame_counter & 3) == 0 {
            self.sprite_slot_view_mut(k).set_graphics(2);
            let dir = self.sprite_direction_to_face_link(k);
            self.sprite_slot_view_mut(k)
                .set_head_direction(if dir == 3 { 2 } else { dir });
        }
        self.oam_allocate_defer_to_player(k);
        self.thief_draw(k);
    }

    // ----- Kiki cluster -------------------------------------------------

    // void Kiki_LyingInwait(int k) {  // sprite_main.c:1493
    //   PrepOamCoordsRet info;
    //   Sprite_PrepOamCoord(k, &info);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   if (link_is_bunny_mirror | link_disable_sprite_damage | countdown_for_blink ||
    //       follower_indicator == 10)
    //     return;
    //   if (save_ow_event_info[BYTE(overworld_screen_index)] & 0x20)
    //     return;
    //   if (Sprite_CheckDamageToLink_same_layer(k)) {
    //     if (enhanced_features0 & kFeatures0_MiscBugFixes)
    //       follower_dropped = 0;  // defuse bomb
    //     follower_indicator = 10;
    //     tagalong_var5 = 0;
    //     LoadFollowerGraphics();
    //     Follower_Initialize();
    //   }
    // }
    pub(super) fn kiki_lying_inwait(&mut self, k: usize) {
        self.sprite_prep_oam_coord(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let gate = u8::from(self.game_state.player.follower_link.is_bunny_mirror())
            | self
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer()
            | self.game_state.player.follower_link.blink_countdown();
        if gate != 0 || self.game_state.sprites.follower_runtime.indicator() == 10 {
            return;
        }
        let scr = self.game_state.world.location.overworld_screen_index() as usize;
        if (self.game_state.world.overworld.event_info.event_info(scr) & 0x20) != 0 {
            return;
        }
        if self.sprite_check_damage_to_link_same_layer(k) {
            let features = self.game_state.enhanced_features.bits();
            if features & FEATURES0_MISC_BUG_FIXES != 0 {
                self.follower_state_mut().set_dropped(0);
            }
            self.follower_state_mut().set_indicator(10);
            self.follower_state_mut().set_appearance_none_flag(0);
            self.load_follower_graphics();
            self.follower_initialize();
        }
    }

    // void Kiki_Flee(int k) {  // sprite_main.c:24358
    //   bool flag = Kiki_Draw(k);
    //   if (Sprite_ReturnIfInactive(k)) return;
    //   if (!sprite_z[k] && (uint16)(cur_sprite_x - 0xc98) < 0xd0 &&
    //       (uint16)(cur_sprite_y - 0x6a5) < 0xd0) flag = true;
    //   if (flag) sprite_state[k] = 0;
    //   sprite_z_vel[k]-=2;
    //   Sprite_MoveXYZ(k);
    //   if (sign8(sprite_z[k])) {
    //     sprite_z[k] = 0;
    //     sprite_z_vel[k] = GetRandomNumber() & 15 | 16;
    //   }
    //   ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLocation(k, 0xcf5, 0x6fe, 16);
    //   sprite_x_vel[k] = pt.x << 1;
    //   sprite_y_vel[k] = pt.y << 1;
    //   tagalong_event_flags &= ~3;
    //   if (sign8(pt.x)) pt.x = -pt.x;
    //   if (sign8(pt.y)) pt.y = -pt.y;
    //   sprite_D[k] = (pt.x >= pt.y) ? (sprite_x_vel[k] >> 7) ^ 3
    //                                : (sprite_y_vel[k] >> 7) ^ 1;
    //   sprite_graphics[k] = frame_counter >> 3 & 1;
    // }
    pub(super) fn kiki_flee(&mut self, k: usize) {
        let mut flag = self.kiki_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let cx = self.game_state.sprites.workspace.current_sprite_x();
        let cy = self.game_state.sprites.workspace.current_sprite_y();
        if self.sprite_slot_view(k).z() == 0
            && cx.wrapping_sub(0xc98) < 0xd0
            && cy.wrapping_sub(0x6a5) < 0xd0
        {
            flag = true;
        }
        if flag {
            self.sprite_slot_view_mut(k).set_state(0);
        }
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        self.sprite_move_xyz(k);
        if (self.sprite_slot_view(k).z() as i8) < 0 {
            self.sprite_slot_view_mut(k).set_z(0);
            let z_velocity = (self.get_random_number() & 15) | 16;
            self.sprite_slot_view_mut(k).set_z_velocity(z_velocity);
        }
        let pt = self.sprite_project_speed_towards_location(k, 0xcf5, 0x6fe, 16);
        self.sprite_slot_view_mut(k)
            .set_x_velocity(pt.x.wrapping_shl(1));
        self.sprite_slot_view_mut(k)
            .set_y_velocity(pt.y.wrapping_shl(1));
        self.follower_state_mut().and_event_flags(!3);
        let mut px = pt.x as i8;
        let mut py = pt.y as i8;
        if px < 0 {
            px = px.wrapping_neg();
        }
        if py < 0 {
            py = py.wrapping_neg();
        }
        let d = if (px as u8) >= (py as u8) {
            (self.sprite_slot_view(k).x_velocity() >> 7) ^ 3
        } else {
            (self.sprite_slot_view(k).y_velocity() >> 7) ^ 1
        };
        self.sprite_slot_view_mut(k).set_direction(d);
        let graphics = (self.game_state.frame.frame_counter >> 3) & 1;
        self.sprite_slot_view_mut(k).set_graphics(graphics);
    }

    // void Kiki_OfferInitialService(int k) {  // sprite_main.c:24385
    //   if (!sign8(sprite_ai_state[k] - 2)) Kiki_Draw(k);
    //   if (Sprite_ReturnIfInactive(k)) return;
    //   Sprite_MoveXYZ(k);
    //   sprite_z_vel[k]--;
    //   if (sign8(sprite_z[k])) { sprite_z_vel[k] = 0; sprite_z[k] = 0; }
    //   sprite_graphics[k] = frame_counter >> 3 & 1;
    //   switch(sprite_ai_state[k]) {
    //   case 0: Sprite_ShowMessageUnconditional(0x11e); sprite_ai_state[k]++; break;
    //   case 1: ... ShopItem_HandleCost(10) ...
    //   case 2: { ProjectSpeedRet pt = ... 0xc45, 0x6fe, 9; ... }
    //   case 3: ... case 4: ...
    //   }
    // }
    pub(super) fn kiki_offer_initial_service(&mut self, k: usize) {
        // `!sign8(s-2)` <=> `(s - 2) as i8 >= 0` <=> `s >= 2 && s < 0x82`.
        // Since ai_state is u8 and the value is small in practice, a direct
        // signed-byte compare matches the C semantics.
        let s = self.sprite_slot_view(k).ai_state();
        if (s.wrapping_sub(2) as i8) >= 0 {
            self.kiki_draw(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xyz(k);
        self.sprite_slot_view_mut(k).decrement_z_velocity();
        if (self.sprite_slot_view(k).z() as i8) < 0 {
            self.sprite_slot_view_mut(k).set_z_velocity(0);
            self.sprite_slot_view_mut(k).set_z(0);
        }
        let graphics = (self.game_state.frame.frame_counter >> 3) & 1;
        self.sprite_slot_view_mut(k).set_graphics(graphics);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_show_message_unconditional(0x11e);
                self.sprite_slot_view_mut(k).increment_ai_state();
            }
            1 => {
                let choice = self.multiselect_choice().value_word();
                if choice == 0 && self.shop_item_handle_cost(10) {
                    self.sprite_show_message_unconditional(0x11f);
                    self.follower_state_mut().or_event_flags(3);
                    self.sprite_slot_view_mut(k).set_state(0);
                } else {
                    self.sprite_show_message_unconditional(0x120);
                    self.follower_state_mut().and_event_flags(!3);
                    self.follower_state_mut().set_indicator(0);
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.follower_link_state_mut().increment_immobilized_flag();
                }
            }
            2 => {
                self.sprite_slot_view_mut(k).increment_ai_state();
                let pt = self.sprite_project_speed_towards_location(k, 0xc45, 0x6fe, 9);
                self.sprite_slot_view_mut(k).set_y_velocity(pt.y);
                self.sprite_slot_view_mut(k).set_x_velocity(pt.x);
                self.sprite_slot_view_mut(k).set_direction((pt.x >> 7) ^ 3);
                self.sprite_slot_view_mut(k).set_delay_main(32);
            }
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_slot_view_mut(k).set_z_velocity(16);
                    self.sprite_slot_view_mut(k).set_delay_main(16);
                }
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 && self.sprite_slot_view(k).z() == 0 {
                    self.sprite_slot_view_mut(k).set_state(0);
                    self.follower_link_state_mut().clear_immobilized();
                }
            }
            _ => {}
        }
    }

    // void Kiki_OfferEntranceService(int k) {  // sprite_main.c:24440
    pub(super) fn kiki_offer_entrance_service(&mut self, k: usize) {
        self.kiki_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xyz(k);
        self.sprite_slot_view_mut(k).decrement_z_velocity();
        if (self.sprite_slot_view(k).z() as i8) < 0 {
            self.sprite_slot_view_mut(k).set_z_velocity(0);
            self.sprite_slot_view_mut(k).set_z(0);
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_show_message_unconditional(0x11b);
                self.sprite_slot_view_mut(k).increment_ai_state();
            }
            1 => {
                let choice = self.multiselect_choice().value_word();
                if choice != 0 || !self.shop_item_handle_cost(100) {
                    self.sprite_show_message_unconditional(0x11c);
                    self.sprite_slot_view_mut(k).set_subtype2(3);
                } else {
                    self.sprite_show_message_unconditional(0x11d);
                    self.follower_link_state_mut().increment_immobilized_flag();
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_slot_view_mut(k).set_direction(0);
                }
            }
            s @ (2 | 4 | 6) => {
                let graphics = (self.game_state.frame.frame_counter >> 3) & 1;
                self.sprite_slot_view_mut(k).set_graphics(graphics);
                let j = ((s >> 1) - 1) as usize;
                let dx = KIKI_LEAVE_X[j]
                    .wrapping_sub(self.sprite_slot_view(k).x_low() as u16)
                    .wrapping_add(2) as u8;
                let dy = KIKI_LEAVE_Y[j]
                    .wrapping_sub(self.sprite_slot_view(k).y_low() as u16)
                    .wrapping_add(2) as u8;
                if dx < 4 && dy < 4 {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_slot_view_mut(k).set_x_velocity(0);
                    self.sprite_slot_view_mut(k).set_y_velocity(0);
                    self.sprite_slot_view_mut(k).set_delay_aux1(32);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
                    return;
                }
                let pt = self.sprite_project_speed_towards_location(
                    k,
                    KIKI_LEAVE_X[j],
                    KIKI_LEAVE_Y[j],
                    9,
                );
                self.sprite_slot_view_mut(k).set_x_velocity(pt.x);
                self.sprite_slot_view_mut(k).set_y_velocity(pt.y);
            }
            s @ (3 | 5) => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    let old = s;
                    let new_state = old.wrapping_add(1);
                    self.sprite_slot_view_mut(k).set_ai_state(new_state);
                    // `sprite_ai_state[k]++ >> 1 & 1` reads the *pre-increment*
                    // value; reproduce that ordering exactly.
                    self.sprite_slot_view_mut(k)
                        .set_z_velocity(KIKI_ZVEL[((old >> 1) & 1) as usize]);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
                    self.sprite_slot_view_mut(k)
                        .set_direction(((new_state >> 1) & 1) | 4);
                } else {
                    self.sprite_slot_view_mut(k)
                        .set_direction(((s >> 1) & 1) | 6);
                    let graphics = (self.game_state.frame.frame_counter >> 3) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                }
            }
            7 => {
                let graphics = (self.game_state.frame.frame_counter >> 3) & 1;
                self.sprite_slot_view_mut(k).set_graphics(graphics);
                if self.sprite_slot_view(k).z() != 0 || self.sprite_slot_view(k).delay_main() != 0 {
                    return;
                }
                let j = self.sprite_slot_view(k).a() as usize;
                self.sprite_slot_view_mut(k).increment_a();
                let t = KIKI_LEAVE_Y_ACCELERATION_BY_TARGET[j];
                if t >= 0 {
                    self.sprite_slot_view_mut(k).set_direction(t as u8);
                    self.sprite_slot_view_mut(k)
                        .set_delay_main(KIKI_FINAL_LEAVE_HOP_DELAYS[j]);
                    self.sprite_slot_view_mut(k)
                        .set_x_velocity(KIKI_XVEL7[t as usize] as u8);
                    self.sprite_slot_view_mut(k)
                        .set_y_velocity(KIKI_YVEL7[t as usize] as u8);
                } else {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_slot_view_mut(k).set_x_velocity(0);
                    self.sprite_slot_view_mut(k).set_y_velocity(0);
                    self.set_special_entrance_trigger(1);
                    self.set_subsubmodule(0);
                    self.clear_entrance_sequence_counter();
                    self.sprite_slot_view_mut(k).set_direction(0);
                    self.follower_link_state_mut().clear_immobilized();
                }
            }
            8 => {
                self.sprite_slot_view_mut(k).set_direction(8);
                self.sprite_slot_view_mut(k).set_graphics(0);
                // ROM Kiki_OfferEntranceService $1E:E648: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                let z_velocity = self.get_random_number_with_carry().masked_adc(15, 16);
                self.sprite_slot_view_mut(k).set_z_velocity(z_velocity);
                self.sprite_slot_view_mut(k).increment_ai_state();
            }
            9 => {
                if (self.sprite_slot_view(k).z_velocity() as i8) < 0
                    && self.sprite_slot_view(k).z() == 0
                {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x25);
                }
            }
            10 => {}
            _ => {}
        }
    }

    // bool Kiki_Draw(int k) {  // sprite_main.c:24543
    pub(super) fn kiki_draw(&mut self, k: usize) -> bool {
        let mut info = if self.sprite_slot_view(k).direction() < 8 {
            let j = (self.sprite_slot_view(k).direction() as usize) * 2
                + self.sprite_slot_view(k).graphics() as usize;
            self.set_sprite_dma_head_pointer(KIKI_DMA[j * 2]);
            self.set_sprite_dma_body_pointer(KIKI_DMA[j * 2 + 1]);
            self.sprite_draw_multiple(k, &KIKI_DRAW_FRAMES1[j * 2..j * 2 + 2])
        } else {
            let gfx = self.sprite_slot_view(k).graphics() as usize;
            self.sprite_draw_multiple(k, &KIKI_DRAW_FRAMES2[gfx * 6..gfx * 6 + 6])
        };
        if self.sprite_slot_view(k).pause() == 0 {
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        }
        ((info.x | info.y) & 0xff00) != 0
    }

    // ----- Cucco cluster (retry: Sprite_ReturnIfLifted etc. now exist) --

    // void Cucco_Calm(int k) {  // sprite_main.c:9367
    //   if (sprite_delay_main[k] == 0) {
    //     int j = GetRandomNumber() & 0xf;
    //     sprite_x_vel[k] = kSpriteKeese_Tab2[j];
    //     sprite_y_vel[k] = kSpriteKeese_Tab3[j];
    //     sprite_delay_main[k] = (GetRandomNumber() & 0x1f) + 0x10;
    //     sprite_ai_state[k]++;
    //   }
    //   sprite_graphics[k] = 0;
    //   Sprite_ReturnIfLifted(k);
    // }
    pub(super) fn cucco_calm(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            let j = (self.get_random_number() & 0xf) as usize;
            self.sprite_slot_view_mut(k)
                .set_x_velocity(CUCCO_CALM_CIRCLE_X_VELOCITIES[j] as u8);
            self.sprite_slot_view_mut(k)
                .set_y_velocity(CUCCO_CALM_CIRCLE_Y_VELOCITIES[j] as u8);
            // ROM $06:a69d calls GetRandomNumber, then executes AND #$1f;
            // ADC #$10 without clearing carry. AND preserves the carry left by
            // the RNG routine, so consume the full typed RNG result here.
            let delay = self.get_random_number_with_carry().masked_adc(0x1f, 0x10);
            self.sprite_slot_view_mut(k).set_delay_main(delay);
            self.sprite_slot_view_mut(k).increment_ai_state();
        }
        self.sprite_slot_view_mut(k).set_graphics(0);
        self.sprite_return_if_lifted(k);
    }

    // void Chicken_Hopping(int k) {  // sprite_main.c:9379
    //   if ((k ^ frame_counter) & 1 && Cucco_DoMovement_XY(k))
    //     sprite_ai_state[k] = 0;
    //   Sprite_MoveZ(k);
    //   sprite_z_vel[k] -= 2;
    //   if (sign8(sprite_z[k])) {
    //     sprite_z[k] = 0;
    //     if (sprite_delay_main[k] == 0) {
    //       sprite_delay_main[k] = 32;
    //       sprite_ai_state[k] = 0;
    //     }
    //     sprite_z_vel[k] = 10;
    //   }
    //   Chicken_IncrSubtype2(k, 4);
    // }
    pub(super) fn chicken_hopping(&mut self, k: usize, helper_ordinal: u8) {
        if ((k as u8) ^ self.game_state.frame.frame_counter) & 1 != 0
            && self.cucco_do_movement_xy(k) != 0
        {
            self.sprite_slot_view_mut(k).set_ai_state(0);
        }
        self.sprite_move_z(k);
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        if (self.sprite_slot_view(k).z() as i8) < 0 {
            self.sprite_slot_view_mut(k).set_z(0);
            if self.sprite_slot_view(k).delay_main() == 0 {
                self.sprite_slot_view_mut(k).set_delay_main(32);
                self.sprite_slot_view_mut(k).set_ai_state(0);
            }
            self.sprite_slot_view_mut(k).set_z_velocity(10);
        }
        self.chicken_incr_subtype2_for_draw(
            k,
            4,
            helper_ordinal,
            CuccoSubtypeContinuation::Hopping,
        );
    }

    // void Cucco_Flee(int k) {  // sprite_main.c:9395
    //   Sprite_ReturnIfLifted(k);
    //   Cucco_DoMovement_XY(k);
    //   sprite_z[k] = 0;
    //   if (!((k ^ frame_counter) & 0x1f)) {
    //     ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, 16);
    //     sprite_x_vel[k] = -pt.x; sprite_y_vel[k] = -pt.y;
    //   }
    //   Chicken_IncrSubtype2(k, 5);
    //   Cucco_DrawPANIC(k);
    // }
    pub(super) fn cucco_flee(&mut self, k: usize, helper_ordinal: u8) {
        self.sprite_return_if_lifted(k);
        self.cucco_do_movement_xy(k);
        self.complete_cucco_flee_after_move_xy(k, helper_ordinal);
    }

    /// Complete `Cucco_Flee` after `Cucco_DoMovement_XY` has returned. This
    /// owns the source statements between movement and the shared subtype
    /// helper.
    pub(super) fn complete_cucco_flee_after_move_xy(&mut self, k: usize, helper_ordinal: u8) {
        self.sprite_slot_view_mut(k).set_z(0);
        let fc = self.game_state.frame.frame_counter as usize;
        if (k ^ fc) & 0x1f == 0 {
            let pt = self.sprite_project_speed_towards_link(k, 16);
            self.sprite_slot_view_mut(k)
                .set_x_velocity(pt.x.wrapping_neg());
            self.sprite_slot_view_mut(k)
                .set_y_velocity(pt.y.wrapping_neg());
        }
        if self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::AfterCuccoFleeMovement {
                slot: k as u8,
                helper_ordinal,
            })
        {
            return;
        }
        self.complete_cucco_flee_after_movement(k, helper_ordinal);
    }

    /// Resume `Cucco_Flee` after its movement/Z/retarget prefix. This begins
    /// at the pending shared helper call and therefore cannot replay motion.
    pub(super) fn complete_cucco_flee_after_movement(&mut self, k: usize, helper_ordinal: u8) {
        if self.chicken_incr_subtype2_for_draw(k, 5, helper_ordinal, CuccoSubtypeContinuation::Flee)
        {
            return;
        }
        self.cucco_draw_panic(k);
    }

    pub(super) fn advance_cucco_helper_to_subtype_checkpoint(&mut self, k: usize, completed: u8) {
        assert!((1..=5).contains(&completed));
        for _ in 0..completed {
            self.chicken_add_subtype2_for_draw(k, 1);
        }
    }

    pub(super) fn advance_cucco_helper_to_graphics(&mut self, k: usize, increments: u8) {
        self.advance_cucco_helper_to_subtype_checkpoint(k, increments);
        self.chicken_publish_graphics_for_draw(k);
    }

    // void Cucco_Carried(int k) {  // sprite_main.c:9415
    //   Sprite_MoveZ(k);
    //   if (Cucco_DoMovement_XY(k)) {
    //     sprite_x_vel[k] = -sprite_x_vel[k];
    //     sprite_y_vel[k] = -sprite_y_vel[k];
    //     Sprite_MoveXY(k);
    //     Sprite_HalveSpeed_XY(k);
    //     Sprite_HalveSpeed_XY(k);
    //     BawkBawk(k);
    //   }
    //   sprite_z_vel[k]--;
    //   if (sign8(sprite_z[k])) {
    //     sprite_z[k] = 0;
    //     sprite_ai_state[k] = 2;
    //     ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, 16);
    //     sprite_x_vel[k] = -pt.x; sprite_y_vel[k] = -pt.y;
    //     Chicken_IncrSubtype2(k, 5);
    //     Cucco_DrawPANIC(k);
    //   } else {
    //     Chicken_IncrSubtype2(k, 4);
    //   }
    // }
    pub(super) fn cucco_carried(&mut self, k: usize, helper_ordinal: u8) {
        self.sprite_move_z(k);
        if self.cucco_do_movement_xy(k) != 0 {
            self.sprite_slot_view_mut(k).negate_x_velocity();
            self.sprite_slot_view_mut(k).negate_y_velocity();
            self.sprite_move_xy(k);
            self.sprite_halve_speed_xy(k);
            self.sprite_halve_speed_xy(k);
            self.bawk_bawk(k);
        }
        self.sprite_slot_view_mut(k).decrement_z_velocity();
        if (self.sprite_slot_view(k).z() as i8) < 0 {
            self.sprite_slot_view_mut(k).set_z(0);
            self.sprite_slot_view_mut(k).set_ai_state(2);
            let pt = self.sprite_project_speed_towards_link(k, 16);
            self.sprite_slot_view_mut(k)
                .set_x_velocity(pt.x.wrapping_neg());
            self.sprite_slot_view_mut(k)
                .set_y_velocity(pt.y.wrapping_neg());
            if self.chicken_incr_subtype2_for_draw(
                k,
                5,
                helper_ordinal,
                CuccoSubtypeContinuation::CarriedLanding,
            ) {
                return;
            }
            self.cucco_draw_panic(k);
        } else {
            self.chicken_incr_subtype2_for_draw(
                k,
                4,
                helper_ordinal,
                CuccoSubtypeContinuation::CarriedAirborne,
            );
        }
    }

    // void Cucco_SummonAvenger(int k) {  // sprite_main.c:9444
    //   static const uint8 kChicken_Avenger[2] = {0, 0xff};
    //   if ((k ^ frame_counter) & 0xf | player_is_indoors) return;
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamicallyEx(k, 0xB, &info, 10);
    //   if (j < 0) return;
    //   SpriteSfx_QueueSfx3WithPan(j, 0x1e);
    //   sprite_C[j] = 1;
    //   uint8 t = GetRandomNumber();
    //   uint16 x = BG2HOFS_copy2, y = BG2VOFS_copy2;
    //   if (t & 2) x += t, y += kChicken_Avenger[t & 1];
    //   else       y += t, x += kChicken_Avenger[t & 1];
    //   Sprite_SetX(j, x); Sprite_SetY(j, y);
    //   Sprite_ApplySpeedTowardsLink(j, 32);
    //   BawkBawk(k);
    // }
    pub(super) fn cucco_summon_avenger(&mut self, k: usize) {
        let fc = self.game_state.frame.frame_counter as usize;
        // Original uses `|` (bitwise OR) — preserve early exit semantics.
        if ((k ^ fc) & 0xf) as u8 | self.game_state.world.location.indoor_flag() != 0 {
            return;
        }
        let Some((j, _)) = self.spawn_sprite_dynamically_ex(k, 0xB, 10) else {
            return;
        };
        self.sprite_sfx_queue_sfx3_with_pan(j, 0x1e);
        self.sprite_slot_view_mut(j).set_c(1);
        let random = self.get_random_number_with_carry();
        let (x, y) = cucco_avenger_spawn_coordinates(
            random,
            self.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        self.sprite_set_x(j, x);
        self.sprite_set_y(j, y);
        self.sprite_apply_speed_towards_link(j, 32);
        self.bawk_bawk(k);
    }

    // Helper: void BawkBawk(int k) {  // sprite_main.c:9466
    //   SpriteSfx_QueueSfx2WithPan(k, 0x30);
    // }
    pub(super) fn bawk_bawk(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x30);
    }

    // Helper: uint8 Cucco_DoMovement_XY(int k) {  // sprite_main.c:9439
    //   Sprite_MoveXY(k);
    //   return Sprite_CheckTileCollision(k);
    // }
    fn cucco_do_movement_xy(&mut self, k: usize) -> u8 {
        self.sprite_move_xy(k);
        self.sprite_check_tile_collision(k)
    }

    // ----- Smithy cluster -----------------------------------------------

    // void Sprite_1A_Smithy(int k) {  // sprite_main.c:9981
    pub(super) fn sprite_1_a_smithy(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.smithy_main(k),
            1 => self.smithy_spark(k),
            2 => self.smithy_frog(k),
            3 => self.smithy_homecoming(k),
            _ => {}
        }
    }

    // void Smithy_Homecoming(int k) {  // sprite_main.c:9990
    pub(super) fn smithy_homecoming(&mut self, k: usize) {
        self.returning_smithy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_move_xy(k);
                let graphics = (self.game_state.frame.frame_counter >> 3) & 1;
                self.sprite_slot_view_mut(k).set_graphics(graphics);
                if self.sprite_slot_view(k).delay_main() != 0 {
                    return;
                }
                let idx = self.sprite_slot_view(k).a() as usize;
                self.sprite_slot_view_mut(k).increment_a();
                self.sprite_slot_view_mut(k)
                    .set_delay_main(RETURNING_SMITHY_DELAY[idx] as u8);
                let dir = RETURNING_SMITHY_DIR[idx];
                if dir >= 0 {
                    let j = dir as usize;
                    self.sprite_slot_view_mut(k).set_direction(dir as u8);
                    self.sprite_slot_view_mut(k)
                        .set_x_velocity(RETURNING_SMITHY_XVEL[j] as u8);
                    self.sprite_slot_view_mut(k)
                        .set_y_velocity(RETURNING_SMITHY_YVEL[j] as u8);
                } else {
                    self.sprite_slot_view_mut(k).set_ai_state(1);
                }
            }
            1 => {
                self.sprite_behave_as_barrier(k);
                self.sprite_show_solicited_message(k, 0xe3);
                self.follower_link_state_mut().clear_immobilized();
                self.sprite_slot_view_mut(k).set_direction(1);
                self.save_progress_mut().or_progress_indicator_3(32);
            }
            _ => {}
        }
    }

    // void Smithy_Frog(int k) {  // sprite_main.c:10025
    pub(super) fn smithy_frog(&mut self, k: usize) {
        self.smithy_frog_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        self.sprite_move_z(k);
        if (self.sprite_slot_view(k).z() as i8) < 0 {
            self.sprite_slot_view_mut(k).set_z(0);
            self.sprite_slot_view_mut(k).set_z_velocity(16);
        }
        if self.sprite_slot_view(k).ai_state() == 0 {
            self.sprite_slot_view_mut(k).set_direction(1);
            if (self.sprite_show_solicited_message(k, 0xe1) & 0x100) != 0 {
                self.sprite_slot_view_mut(k).set_ai_state(1);
            }
        } else {
            self.follower_state_mut().set_indicator(7);
            self.load_follower_graphics();
            self.sprite_become_follower(k);
            self.sprite_slot_view_mut(k).set_state(0);
        }
    }

    // void ReturningSmithy_Draw(int k) {  // sprite_main.c:10048
    pub(super) fn returning_smithy_draw(&mut self, k: usize) {
        let j = (self.sprite_slot_view(k).direction() as usize) * 2
            + self.sprite_slot_view(k).graphics() as usize;
        self.set_sprite_dma_body_pointer(RETURNING_SMITHY_DMA[j]);
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &RETURNING_SMITHY_DRAW_FRAMES[j..j + 1],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // void Smithy_Main(int k) {  // sprite_main.c:10076
    pub(super) fn smithy_main(&mut self, k: usize) {
        self.smithy_draw(k);
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        self.sprite_move_z(k);
        if (self.sprite_slot_view(k).z() as i8) < 0 {
            self.sprite_slot_view_mut(k).set_z(0);
            self.sprite_slot_view_mut(k).set_z_velocity(0);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let e_idx = self.sprite_slot_view(k).e() as usize;
        let other = self.sprite_slot_view(e_idx).ai_state();
        let me = self.sprite_slot_view(k).ai_state();
        if (other == 5
            || other == 7
            || other == 9
            || me == 5
            || me == 7
            || me == 9
            || (me | other) == 0)
            && {
                let old = self.sprite_slot_view(k).b();
                self.sprite_slot_view_mut(k).set_b(old.wrapping_sub(1));
                old == 0
            }
        {
            let idx = self.sprite_slot_view(k).a() as usize;
            self.sprite_slot_view_mut(k)
                .set_a(((idx as u8).wrapping_add(1)) & 7);
            self.sprite_slot_view_mut(k).set_graphics(SMITHY_GFX[idx]);
            self.sprite_slot_view_mut(k)
                .set_b(SMITHY_FRAME_DURATIONS[idx]);
            if idx == 1 {
                self.sprite_slot_view_mut(k).set_z_velocity(16);
            }
            if idx == 3 {
                self.smithy_spawn_spark(k);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x5);
            }
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_slot_view_mut(k).set_c(0);
                if self.game_state.sprites.follower_runtime.indicator() != 8 {
                    if self.smithy_listen_for_hammer(k) {
                        self.sprite_show_message_unconditional(0xe4);
                        self.sprite_slot_view_mut(k).set_delay_aux1(96);
                        self.sprite_slot_view_mut(k).increment_c();
                    } else if (self
                        .game_state
                        .inventory
                        .save_progress
                        .progress_indicator_3()
                        & 0x20)
                        != 0
                    {
                        if (self.sprite_show_solicited_message(k, 0xd8) & 0x100) != 0 {
                            self.sprite_slot_view_mut(k).increment_ai_state();
                            self.sprite_slot_view_mut(k).increment_c();
                        }
                    } else {
                        self.sprite_show_solicited_message(k, 0xdf);
                    }
                } else if (self.game_state.player.follower_link.y() as u8) < 0xc2 {
                    self.sprite_show_message_unconditional(0xe0);
                    self.sprite_slot_view_mut(k).set_ai_state(10);
                    self.follower_link_state_mut().increment_immobilized_flag();
                }
            }
            1 => {
                if self.multiselect_choice().value_word() == 0 {
                    self.sprite_show_message_unconditional(0xd9);
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                } else {
                    self.sprite_show_message_unconditional(0xdc);
                    self.sprite_slot_view_mut(k).set_ai_state(0);
                }
            }
            2 => {
                if self.multiselect_choice().value_word() == 0 {
                    if self.game_state.inventory.items.sword_type() < 3 {
                        self.sprite_show_message_unconditional(0xda);
                        self.sprite_slot_view_mut(k).set_ai_state(3);
                    } else {
                        self.sprite_show_message_unconditional(0xdb);
                        self.sprite_slot_view_mut(k).set_ai_state(0);
                    }
                } else {
                    self.sprite_show_message_unconditional(0xdc);
                    self.sprite_slot_view_mut(k).set_ai_state(0);
                }
            }
            3 => {
                let choice = self.multiselect_choice().value_word();
                let rupees = self.game_state.inventory.player_resources.rupees_goal();
                if choice != 0 || rupees < 10 {
                    self.sprite_show_message_unconditional(0xdc);
                    self.sprite_slot_view_mut(k).set_ai_state(0);
                } else {
                    self.player_resources_mut()
                        .set_rupees_goal(rupees.wrapping_sub(10));
                    self.sprite_show_message_unconditional(0xdd);
                    let e_idx = self.sprite_slot_view(k).e() as usize;
                    self.sprite_slot_view_mut(e_idx).set_ai_state(5);
                    self.sprite_slot_view_mut(k).set_ai_state(5);
                    self.clear_flag_overworld_area_changed();
                    self.inventory_items_mut().set_sword_type(255);
                    self.save_progress_mut().or_progress_indicator_3(128);
                }
            }
            4 | 5 => {
                self.sprite_slot_view_mut(k).set_c(0);
                if self.smithy_listen_for_hammer(k) {
                    self.sprite_show_message_unconditional(0xe4);
                    self.sprite_slot_view_mut(k).set_delay_aux1(96);
                    self.sprite_slot_view_mut(k).increment_c();
                } else if self.game_state.world.region.flag_overworld_area_changed() {
                    if (self.sprite_show_solicited_message(k, 0xde) & 0x100) != 0 {
                        self.sprite_slot_view_mut(k).increment_ai_state();
                        self.sprite_slot_view_mut(k).set_graphics(4);
                    }
                } else {
                    self.sprite_show_solicited_message(k, 0xe2);
                }
            }
            6 => {
                self.sprite_slot_view_mut(k).set_ai_state(0);
                let e_idx = self.sprite_slot_view(k).e() as usize;
                self.sprite_slot_view_mut(e_idx).set_ai_state(0);
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        2,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::SmithyTemperedSword,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_smithy_tempered_sword_receipt();
            }
            7..=9 => {}
            10 => {
                if let Some((j, _)) = self.spawn_sprite_dynamically(k, 0x1a) {
                    let lx = self.game_state.player.follower_link.x();
                    let ly = self.game_state.player.follower_link.y();
                    self.sprite_set_x(j, lx);
                    self.sprite_set_y(j, ly);
                    self.sprite_slot_view_mut(j).set_subtype2(3);
                    self.sprite_slot_view_mut(j).set_ignore_projectile(3);
                }
                self.sprite_slot_view_mut(k).set_ai_state(11);
                self.follower_state_mut().set_indicator(0);
                self.sprite_slot_view_mut(k).set_graphics(4);
            }
            11 => {
                self.sprite_show_solicited_message(k, 0xe3);
            }
            _ => {}
        }
    }

    // bool Smithy_ListenForHammer(int k) {  // sprite_main.c:10212
    //   return sprite_delay_aux1[k] == 0 && hud_cur_item == kHudItem_Hammer &&
    //          (link_item_in_hand & 2) && player_handler_timer == 2 &&
    //          Sprite_CheckDamageToLink_same_layer(k);
    // }
    /// Source suffix after Smithy_GiveTemperedSword's `Link_ReceiveItem(2, 0)`.
    pub(super) fn complete_smithy_tempered_sword_receipt(&mut self) {
        self.save_progress_mut()
            .clear_progress_indicator_3_bits(0x80);
    }

    pub(super) fn smithy_listen_for_hammer(&mut self, k: usize) -> bool {
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            return false;
        }
        if self.game_state.inventory.save_progress.hud_current_item() != HUD_ITEM_HAMMER {
            return false;
        }
        if !self.game_state.player.follower_link.item_in_hand_has(2) {
            return false;
        }
        if self.game_state.player.follower_link.action_handler_timer() != 2 {
            return false;
        }
        self.sprite_check_damage_to_link_same_layer(k)
    }

    // int Smithy_SpawnDwarfPal(int k) {  // sprite_main.c:10216
    pub(super) fn smithy_spawn_dwarf_pal(&mut self, k: usize) -> i32 {
        let Some((j, _)) = self.spawn_sprite_dynamically(k, 0x1a) else {
            return -1;
        };
        let (rx, ry) = self.spawn_info_for_dn();
        self.sprite_set_x(j, rx);
        self.sprite_set_y(j, ry);
        let x_low = self.sprite_slot_view(j).x_low().wrapping_add(0x2C);
        self.sprite_slot_view_mut(j).set_x_low(x_low);
        self.sprite_slot_view_mut(j).set_direction(1);
        self.sprite_slot_view_mut(j).set_a(4);
        self.sprite_slot_view_mut(j).set_ignore_projectile(4);
        j as i32
    }

    // void Smithy_Draw(int k) {  // sprite_main.c:10230
    pub(super) fn smithy_draw(&mut self, k: usize) {
        let idx = self.sprite_slot_view(k).graphics() as usize * 4
            + self.sprite_slot_view(k).direction() as usize * 2;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &SMITHY_DRAW_FRAMES[idx..idx + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // void Smithy_Spark(int k) {  // sprite_main.c:10258
    pub(super) fn smithy_spark(&mut self, k: usize) {
        self.smithy_spark_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() != 0 {
            return;
        }
        let j = self.sprite_slot_view(k).a() as usize;
        self.sprite_slot_view_mut(k)
            .set_a(((j as u8).wrapping_add(1)) & 7);
        let g = SMITHY_SPARK_GFX[j];
        if g < 0 {
            self.sprite_slot_view_mut(k).set_state(0);
            return;
        }
        self.sprite_slot_view_mut(k).set_graphics(g as u8);
        self.sprite_slot_view_mut(k)
            .set_delay_main(SMITHY_SPARK_DELAY[j] as u8);
    }

    // void Smithy_SpawnSpark(int k) {  // sprite_main.c:10276
    pub(super) fn smithy_spawn_spark(&mut self, k: usize) {
        if let Some((j, _)) = self.spawn_sprite_dynamically(k, 0x1a) {
            let (rx, ry) = self.spawn_info_for_dn();
            self.sprite_set_x(j, rx);
            self.sprite_set_y(j, ry);
            let delta: i8 = if self.sprite_slot_view(k).direction() != 0 {
                -15
            } else {
                15
            };
            let x_low = (self.sprite_slot_view(j).x_low() as i8).wrapping_add(delta) as u8;
            let y_low = self.sprite_slot_view(j).y_low().wrapping_add(2);
            self.sprite_slot_view_mut(j).set_x_low(x_low);
            self.sprite_slot_view_mut(j).set_y_low(y_low);
            self.sprite_slot_view_mut(j).set_subtype2(1);
        }
    }

    // void Smithy_SpawnDumbBarrierSprite(int k) {  // sprite_main.c:12877
    // ----- `_for_dn` shims -----------------------------------------------
    //
    // Each shim adapts a canonical helper for use by the split-module handlers
    // above while preserving the local call signatures.

    fn spawn_info_for_dn(&self) -> (u16, u16) {
        (
            self.game_state.sprites.workspace.current_sprite_x(),
            self.game_state.sprites.workspace.current_sprite_y(),
        )
    }

    fn thief_draw_apply_head_overrides_for_dn(&mut self, k: usize) {
        let oam = self.game_state.oam.current_pointer_usize();
        let j = self.sprite_slot_view(k).head_direction() as usize;
        self.oam_state_mut().set_entry_char(oam, THIEF_DRAW_CHAR[j]);
        self.oam_state_mut()
            .merge_entry_flags(oam, !0x40, THIEF_DRAW_FLAGS[j]);
    }

}

#[cfg(test)]
#[path = "sprite_main_dungeon_npcs_tests.rs"]
mod tests;
