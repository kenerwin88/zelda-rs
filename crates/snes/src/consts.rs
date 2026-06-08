//! Build-time constants shared across the SNES emulator and the game.
//! Mirrors `zelda3/src/types.h`.

pub const ENABLE_LARGE_SCREEN: bool = true;
pub const PPU_EXTRA_LEFT_RIGHT: usize = if ENABLE_LARGE_SCREEN { 96 } else { 0 };
pub const PPU_X_PIXELS: usize = 256 + PPU_EXTRA_LEFT_RIGHT * 2;
