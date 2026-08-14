use snes::{CpuInstructionTiming, DmaState, PpuState, Snes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RomCpuCheckpoint {
    pub(crate) entry_pc: u32,
    pub(crate) stop_pc: u32,
    pub(crate) a: u16,
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) sp: u16,
    pub(crate) dp: u16,
    pub(crate) db: u8,
    pub(crate) carry: bool,
    pub(crate) zero: bool,
    pub(crate) overflow: bool,
    pub(crate) negative: bool,
    pub(crate) interrupt_disable: bool,
    pub(crate) decimal: bool,
    pub(crate) accumulator_is_8_bit: bool,
    pub(crate) index_is_8_bit: bool,
    pub(crate) emulation: bool,
    pub(crate) stack_address: u16,
    pub(crate) stack_bytes: &'static [u8],
}

/// Read/write-isolated execution of a translated routine's original ROM path.
///
/// The clone supplies instruction ordering and cycle timing only. Its WRAM,
/// SRAM, PPU, and DMA mutations are discarded, leaving the translated Rust
/// implementation as the sole owner of game state.
pub(crate) struct RomCpuTimingRun {
    shadow: Snes,
    stop_pc: u32,
}

impl RomCpuTimingRun {
    pub(crate) fn new(
        rom: &[u8],
        ram: &[u8],
        sram: &[u8],
        ppu: &PpuState,
        dma: &DmaState,
        checkpoint: RomCpuCheckpoint,
    ) -> Result<Self, String> {
        let mut shadow = Snes::new();
        snes::load_rom(&mut shadow, rom).map_err(|error| error.to_string())?;
        shadow.ram.copy_from_slice(ram);
        shadow.cart.ram.copy_from_slice(sram);
        shadow.ppu = ppu.clone();
        shadow.dma = dma.clone();

        shadow.cpu.a = checkpoint.a;
        shadow.cpu.x = checkpoint.x;
        shadow.cpu.y = checkpoint.y;
        shadow.cpu.sp = checkpoint.sp;
        shadow.cpu.pc = checkpoint.entry_pc as u16;
        shadow.cpu.dp = checkpoint.dp;
        shadow.cpu.k = (checkpoint.entry_pc >> 16) as u8;
        shadow.cpu.db = checkpoint.db;
        shadow.cpu.c = checkpoint.carry;
        shadow.cpu.z = checkpoint.zero;
        shadow.cpu.v = checkpoint.overflow;
        shadow.cpu.n = checkpoint.negative;
        shadow.cpu.i = checkpoint.interrupt_disable;
        shadow.cpu.d = checkpoint.decimal;
        shadow.cpu.mf = checkpoint.accumulator_is_8_bit;
        shadow.cpu.xf = checkpoint.index_is_8_bit;
        shadow.cpu.e = checkpoint.emulation;

        let stack_start = usize::from(checkpoint.stack_address);
        let stack_end = stack_start + checkpoint.stack_bytes.len();
        shadow.ram[stack_start..stack_end].copy_from_slice(checkpoint.stack_bytes);

        Ok(Self {
            shadow,
            stop_pc: checkpoint.stop_pc,
        })
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.pc() == self.stop_pc
    }

    pub(crate) fn pc(&self) -> u32 {
        (u32::from(self.shadow.cpu.k) << 16) | u32::from(self.shadow.cpu.pc)
    }

    pub(crate) fn set_raster_position(&mut self, scanline: u16, master_cycle: u16) {
        self.shadow.v_pos = scanline;
        self.shadow.h_pos = master_cycle;
    }

    pub(crate) fn step(&mut self) -> CpuInstructionTiming {
        snes::cpu_run_opcode_timed(&mut self.shadow)
    }
}
