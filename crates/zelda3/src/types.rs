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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    (v << 8) | (v >> 8)
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
pub static WW_CUR_FRAME: AtomicU32 = AtomicU32::new(0);

pub fn ww_set_cur_frame(frame: u32) {
    if WW_ADDR.load(Ordering::Relaxed) == usize::MAX {
        // lazy one-time env init (usize::MAX sentinel also means "not yet checked")
        let a = std::env::var("ZELDA3_WW_ADDR")
            .ok()
            .and_then(|s| usize::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok());
        if let Some(a) = a {
            WW_ADDR.store(a, Ordering::Relaxed);
            if let Some(f) = std::env::var("ZELDA3_WW_FRAME").ok().and_then(|s| s.trim().parse().ok()) {
                WW_FRAME.store(f, Ordering::Relaxed);
            }
        } else {
            WW_ADDR.store(usize::MAX - 1, Ordering::Relaxed); // mark "checked, disabled"
        }
    }
    WW_CUR_FRAME.store(frame, Ordering::Relaxed);
}

#[inline]
fn ww_enabled_hit(offset: usize, len: usize) -> bool {
    let a = WW_ADDR.load(Ordering::Relaxed);
    if a >= usize::MAX - 1 || !(offset <= a && a < offset + len) {
        return false;
    }
    let wf = WW_FRAME.load(Ordering::Relaxed);
    wf == u32::MAX || wf == WW_CUR_FRAME.load(Ordering::Relaxed)
}

/// Call from any non-write_le_u16 write path (slice copies, byte writes) to report
/// hits on the watched address.
#[track_caller]
pub fn ww_check(offset: usize, len: usize, descr: &str, value: u32) {
    if ww_enabled_hit(offset, len) {
        eprintln!(
            "[WW] f={} {} off=0x{:05x} len={} val=0x{:x} caller={}",
            WW_CUR_FRAME.load(Ordering::Relaxed),
            descr,
            offset,
            len,
            value,
            std::panic::Location::caller()
        );
    }
}

#[track_caller]
pub fn write_le_u16(bytes: &mut [u8], offset: usize, value: u16) {
    if ww_enabled_hit(offset, 2) {
        eprintln!(
            "[WW] f={} write_le_u16 off=0x{:05x} val=0x{:04x} caller={}",
            WW_CUR_FRAME.load(Ordering::Relaxed),
            offset,
            value,
            std::panic::Location::caller()
        );
    }
    let [lo, hi] = value.to_le_bytes();
    bytes[offset] = lo;
    bytes[offset + 1] = hi;
}

pub fn xy(x: usize, y: usize) -> usize {
    y * 64 + x
}
