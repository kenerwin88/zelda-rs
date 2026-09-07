//! Small C-compatibility types and helpers from `src/types.h`.
//!
//! The game port intentionally starts close to the C source. Keeping these
//! aliases local to the game crate makes later module ports read like the
//! original code without leaking C naming into the emulator crates.

pub type Uint8 = u8;
pub type Int8 = i8;
pub type Uint16 = u16;
pub type Int16 = i16;
pub type Uint32 = u32;
pub type Int32 = i32;
pub type Uint64 = u64;
pub type Int64 = i64;
pub type Uint = u32;

pub const ENABLE_LARGE_SCREEN: bool = true;
pub const PPU_EXTRA_LEFT_RIGHT: usize = if ENABLE_LARGE_SCREEN { 96 } else { 0 };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point16U {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointU8 {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pair16U {
    pub a: u16,
    pub b: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairU8 {
    pub a: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectSpeedRet {
    pub x: u8,
    pub y: u8,
    pub xdiff: u8,
    pub ydiff: u8,
}

impl ProjectSpeedRet {
    pub(crate) const ZERO: Self = Self {
        x: 0,
        y: 0,
        xdiff: 0,
        ydiff: 0,
    };
}

/// Shared byte-wrapping projection used by sprites and ancillas.
///
/// Each caller keeps its own coordinate reads and zero-speed early return.
/// The accumulator and comparisons intentionally retain the original integer
/// algorithm; ordinary division or vector normalization changes its rounding.
pub(crate) fn project_speed_from_differences(
    mut speed: u8,
    below: PairU8,
    right: PairU8,
) -> ProjectSpeedRet {
    if speed == 0 {
        return ProjectSpeedRet::ZERO;
    }
    let mut minor_magnitude = if (below.b as i8).is_negative() {
        0u8.wrapping_sub(below.b)
    } else {
        below.b
    };

    let mut major_magnitude = if (right.b as i8).is_negative() {
        0u8.wrapping_sub(right.b)
    } else {
        right.b
    };
    let mut swapped = false;
    if major_magnitude < minor_magnitude {
        swapped = true;
        std::mem::swap(&mut minor_magnitude, &mut major_magnitude);
    }
    let mut x_velocity = speed;
    let mut y_velocity = 0u8;
    let mut accumulator = 0u8;
    loop {
        accumulator = accumulator.wrapping_add(minor_magnitude);
        if accumulator >= major_magnitude {
            accumulator = accumulator.wrapping_sub(major_magnitude);
            y_velocity = y_velocity.wrapping_add(1);
        }
        speed = speed.wrapping_sub(1);
        if speed == 0 {
            break;
        }
    }
    if swapped {
        std::mem::swap(&mut x_velocity, &mut y_velocity);
    }
    ProjectSpeedRet {
        x: if right.a != 0 {
            0u8.wrapping_sub(x_velocity)
        } else {
            x_velocity
        },
        y: if below.a != 0 {
            0u8.wrapping_sub(y_velocity)
        } else {
            y_velocity
        },
        xdiff: right.b,
        ydiff: below.b,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpriteHitBox {
    pub r0_xlo: u8,
    pub r8_xhi: u8,
    pub r1_ylo: u8,
    pub r9_yhi: u8,
    pub r2: u8,
    pub r3: u8,
    pub r4_spr_xlo: u8,
    pub r10_spr_xhi: u8,
    pub r5_spr_ylo: u8,
    pub r11_spr_yhi: u8,
    pub r6_spr_xsize: u8,
    pub r7_spr_ysize: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AncillaRadialProjection {
    pub r0: u8,
    pub r2: u8,
    pub r4: u8,
    pub r6: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OamEnt {
    pub x: u8,
    pub y: u8,
    pub charnum: u8,
    pub flags: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemBlk<'a> {
    pub ptr: &'a [u8],
}

/// 65816 16-bit `SBC` with an explicit incoming carry: returns the result and
/// the outgoing carry (set when no borrow occurred). ROM routines that omit
/// `SEC` inherit whatever carry the previous compare left behind; port those
/// sites through this helper instead of a plain subtraction.
pub fn sbc_u16(a: u16, b: u16, carry_in: bool) -> (u16, bool) {
    let subtrahend = u32::from(b) + u32::from(!carry_in);
    let result = u32::from(a).wrapping_sub(subtrahend);
    ((result & 0xffff) as u16, u32::from(a) >= subtrahend)
}

pub fn sign8(x: u8) -> bool {
    x & 0x80 != 0
}

pub fn sign16(x: u16) -> bool {
    x & 0x8000 != 0
}

pub fn abs8(t: u8) -> u8 {
    if sign8(t) {
        t.wrapping_neg()
    } else {
        t
    }
}

pub fn abs16(t: u16) -> u16 {
    if sign16(t) {
        t.wrapping_neg()
    } else {
        t
    }
}

pub fn int_min(a: i32, b: i32) -> i32 {
    if a < b {
        a
    } else {
        b
    }
}

pub fn int_max(a: i32, b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}

pub fn uint_min(a: u32, b: u32) -> u32 {
    if a < b {
        a
    } else {
        b
    }
}

pub fn uint_max(a: u32, b: u32) -> u32 {
    if a > b {
        a
    } else {
        b
    }
}

pub fn swap16(v: u16) -> u16 {
    v.rotate_right(8)
}

pub fn load24(bytes: &[u8]) -> u32 {
    assert!(bytes.len() >= 3);
    bytes[0] as u32 | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16)
}

pub fn read_le_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

// --- Write-watchpoint debug tool (env-driven) -------------------------------
// ZELDA3_WW_ADDR=0x<addr> [ZELDA3_WW_FRAME=<n>] logs every write that touches the
// address (via write_le_u16 / ww_check), with the calling site and current frame.
// Set the current frame each frame via ww_set_cur_frame(); 0xFFFFFFFF frame = all frames.
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
pub static WW_ADDR: AtomicUsize = AtomicUsize::new(usize::MAX);
pub static WW_FRAME: AtomicU32 = AtomicU32::new(u32::MAX);
/// Inclusive upper bound of the watched frame range (`ZELDA3_WW_FRAME=<lo>-<hi>`);
/// equals `WW_FRAME` when a single frame is given.
pub static WW_FRAME_HI: AtomicU32 = AtomicU32::new(u32::MAX);
pub static WW_CUR_FRAME: AtomicU32 = AtomicU32::new(0);

pub fn ww_set_cur_frame(frame: u32) {
    if WW_ADDR.load(Ordering::Relaxed) == usize::MAX {
        // lazy one-time env init (usize::MAX sentinel also means "not yet checked")
        let a = crate::debug_env::var("ZELDA3_WW_ADDR")
            .ok()
            .and_then(|s| usize::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok());
        if let Some(a) = a {
            WW_ADDR.store(a, Ordering::Relaxed);
            if let Ok(spec) = crate::debug_env::var("ZELDA3_WW_FRAME") {
                let spec = spec.trim();
                let (lo, hi) = match spec.split_once('-') {
                    Some((lo, hi)) => (lo.trim().parse().ok(), hi.trim().parse().ok()),
                    None => {
                        let f: Option<u32> = spec.parse().ok();
                        (f, f)
                    }
                };
                if let (Some(lo), Some(hi)) = (lo, hi) {
                    WW_FRAME.store(lo, Ordering::Relaxed);
                    WW_FRAME_HI.store(hi, Ordering::Relaxed);
                }
            }
        } else {
            WW_ADDR.store(usize::MAX - 1, Ordering::Relaxed); // mark "checked, disabled"
        }
    }
    WW_CUR_FRAME.store(frame, Ordering::Relaxed);
}

#[inline]
fn ww_enabled_hit(offset: usize, len: usize) -> bool {
    if !crate::debug_env::ENABLED {
        return false;
    }
    let a = WW_ADDR.load(Ordering::Relaxed);
    if a >= usize::MAX - 1 || !(offset <= a && a < offset + len) {
        return false;
    }
    let wf = WW_FRAME.load(Ordering::Relaxed);
    if wf == u32::MAX {
        return true;
    }
    let cur = WW_CUR_FRAME.load(Ordering::Relaxed);
    wf <= cur && cur <= WW_FRAME_HI.load(Ordering::Relaxed)
}

/// When `ZELDA3_WW_BACKTRACE=1`, return a full call-stack suffix to append to the
/// `[WW]` line — the ENTIRE caller chain in one run, so you never have to mark
/// `#[track_caller]` and rebuild per stack level. (Build with `RUSTFLAGS="-C
/// debuginfo=2"` for file:line + inlined frames; plain release shows function names.)
#[cold]
fn ww_backtrace_suffix() -> String {
    static BT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
    let mut s = BT.load(Ordering::Relaxed);
    if s == 0 {
        s = if crate::debug_env::var("ZELDA3_WW_BACKTRACE").is_ok() {
            2
        } else {
            1
        };
        BT.store(s, Ordering::Relaxed);
    }
    if s == 2 {
        format!(
            "\n--- WW backtrace ---\n{}--- end WW backtrace ---",
            std::backtrace::Backtrace::force_capture()
        )
    } else {
        String::new()
    }
}

/// Call from any non-write_le_u16 write path (slice copies, byte writes) to report
/// hits on the watched address.
#[track_caller]
pub fn ww_check(offset: usize, len: usize, descr: &str, value: u32) {
    if ww_enabled_hit(offset, len) {
        eprintln!(
            "[WW] f={} {} off=0x{:05x} len={} val=0x{:x} caller={}{}",
            WW_CUR_FRAME.load(Ordering::Relaxed),
            descr,
            offset,
            len,
            value,
            std::panic::Location::caller(),
            ww_backtrace_suffix(),
        );
    }
}

#[track_caller]
pub fn write_le_u16(bytes: &mut [u8], offset: usize, value: u16) {
    if ww_enabled_hit(offset, 2) {
        eprintln!(
            "[WW] f={} write_le_u16 off=0x{:05x} val=0x{:04x} caller={}{}",
            WW_CUR_FRAME.load(Ordering::Relaxed),
            offset,
            value,
            std::panic::Location::caller(),
            ww_backtrace_suffix(),
        );
    }
    let [lo, hi] = value.to_le_bytes();
    bytes[offset] = lo;
    bytes[offset + 1] = hi;
}

pub fn xy(x: usize, y: usize) -> usize {
    y * 64 + x
}

#[cfg(test)]
mod sbc_tests {
    use super::sbc_u16;

    #[test]
    fn sbc_u16_borrows_one_extra_when_carry_is_clear() {
        // Sprite_Absorbable_Main splash offset (route host 450016): centered
        // pan leaves the carry clear, so x loses 5; the x borrow-free result
        // sets the carry so y loses exactly 4.
        let (x, carry) = sbc_u16(0x1099, 4, false);
        assert_eq!(x, 0x1094);
        assert!(carry);
        let (y, carry) = sbc_u16(0x04cb, 4, carry);
        assert_eq!(y, 0x04c7);
        assert!(carry);
    }

    #[test]
    fn sbc_u16_reports_borrow_and_wraps() {
        assert_eq!(sbc_u16(4, 4, true), (0, true));
        assert_eq!(sbc_u16(4, 4, false), (0xffff, false));
        assert_eq!(sbc_u16(0, 1, true), (0xffff, false));
    }
}

#[cfg(test)]
mod projection_tests {
    use super::{project_speed_from_differences, PairU8};

    #[test]
    fn projection_matches_pre_consolidation_results_for_all_byte_differences() {
        // Frozen from ancilla_project_speed_towards_player at a0254a49, before
        // the shared helper existed. Each FNV-1a digest covers all 256 x 256
        // low-byte differences and all four independent direction-flag pairs,
        // in signs/y/x order, hashing x/y/xdiff/ydiff. Boundary speeds cover
        // zero, ties, byte overflow, and the signed-byte transition.
        let expected: &[(u8, u64)] = &[
            (0, 0xa96777069d622325),
            (1, 0xecbadd872992e419),
            (2, 0x1bf63e3939d6bf55),
            (3, 0x3c40784bf40bdc19),
            (7, 0xf9b4e4c93b205b11),
            (15, 0x15b7edb70520af81),
            (16, 0x87571ffacde4a235),
            (24, 0xea48f538f5e90635),
            (31, 0x5886f73b3671b881),
            (32, 0x37efe5667a7eeab5),
            (48, 0x538ee794e5481855),
            (63, 0x7b7bac52637fb2d1),
            (64, 0x24ffa04dfbfa9415),
            (127, 0xd0e17b5a3ccb9a61),
            (128, 0xc39e2ae4510cca65),
            (129, 0xe393d3efa3ee8919),
            (254, 0x812c1da0f43000e5),
            (255, 0x874563350f916aa1),
        ];
        for &(speed, expected_hash) in expected {
            let mut hash = 0xcbf29ce484222325u64;
            for signs in 0..4u8 {
                for y in 0..=255u8 {
                    for x in 0..=255u8 {
                        let result = project_speed_from_differences(
                            speed,
                            PairU8 { a: signs & 1, b: y },
                            PairU8 {
                                a: signs >> 1,
                                b: x,
                            },
                        );
                        for byte in [result.x, result.y, result.xdiff, result.ydiff] {
                            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                        }
                    }
                }
            }
            assert_eq!(hash, expected_hash, "projection changed at speed {speed}");
        }
    }
}
