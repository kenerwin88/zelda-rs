//! Native cycle models of original routines: the master-cycle cost the ROM
//! spends in a routine, computed from the same inputs the routine consumes,
//! without executing ROM code. Each model is priced instruction by
//! instruction from the routine's disassembly and verified against the
//! shadow CPU (`rom_cpu_timing`) over every input the cartridge holds, so a
//! ROM-less build can schedule exactly what a ROM-loaded build measures.
//! See docs/parity/romless-exact-play.md.

pub(crate) mod decompress;
pub(crate) mod text_buffer;
pub(crate) mod vwf;
