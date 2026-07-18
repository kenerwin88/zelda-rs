//! SNES emulator core. Port of the C sources under `zelda3/snes/`.
//!
//! Used as the verification oracle: the C codebase runs the original ROM
//! through this emulator each frame and `memcmp`s WRAM/SRAM/VRAM against
//! the native re-implementation. We preserve that behavior so the Rust
//! port can be validated module-by-module against the original game.

#![allow(dead_code)]

pub mod apu;
pub mod cart;
pub mod consts;
pub mod cpu;
pub mod cpu_step;
mod cycle_spc700;
pub mod dma;
pub mod input;
pub mod loader;
pub mod ppu;
pub mod snes;
pub mod tracing;

pub use cpu_step::cpu_run_opcode;

pub use cart::{Cart, CartType};
pub use cpu::CpuState;
pub use dma::{DmaChannel, DmaState};
pub use input::InputState;
pub use loader::{load_rom, LoadRomError};
pub use ppu::PpuState;
pub use snes::{Snes, WRAM_SIZE};
