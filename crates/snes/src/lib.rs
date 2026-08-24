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
pub mod cpu_timeline;
mod cycle_spc700;
pub mod dma;
pub mod input;
pub mod loader;
pub mod ppu;
pub mod snes;
pub mod snes9x_apu_clock;
pub mod snes9x_apu_timing;
pub mod tracing;

pub use cpu_step::{cpu_run_opcode, cpu_run_opcode_timed, CpuInstructionTiming};

pub use cart::{Cart, CartType};
pub use cpu::CpuState;
pub use cpu_timeline::{
    CpuBusEvent, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
    CpuTimelineDeadlineAdvance, CpuTimelineEvent, HDMA_INIT_CYCLE, HDMA_START_CYCLE,
    MASTER_CYCLES_PER_SCANLINE, NMI_SCANLINE, NTSC_FIELD_MASTER_CYCLES, NTSC_SCANLINES_PER_FIELD,
    SHORT_SCANLINE_END_CYCLE, SHORT_SCANLINE_MISSING_MASTER_CYCLES,
    SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES, SNES9X_NMI_GENERAL_DMA_DELAY_MASTER_CYCLES,
    WRAM_REFRESH_CYCLE, WRAM_REFRESH_STALL_MASTER_CYCLES,
};
pub use dma::{DmaChannel, DmaState};
pub use input::InputState;
pub use loader::{load_rom, LoadRomError};
pub use ppu::PpuState;
pub use snes::{Snes, WRAM_SIZE};
pub use snes9x_apu_clock::{
    Snes9xApuClockCheckpoint, Snes9xApuClockError, Snes9xApuClockState,
    SNES9X_NTSC_APU_CLOCK_DENOMINATOR, SNES9X_NTSC_APU_CLOCK_NUMERATOR,
};
pub use snes9x_apu_timing::{Snes9xApuTiming, Snes9xApuTimingCheckpoint, Snes9xApuTimingError};
