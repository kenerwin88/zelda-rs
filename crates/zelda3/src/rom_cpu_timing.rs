use snes::{CpuInstructionTiming, DmaState, PpuState, Snes};

const TEXT_DIALOGUE_POINTERS: usize = 0x171c0;
const ROM_DIALOGUE_MESSAGE_COUNT: usize = 398;
const ROM_DIALOGUE_SECOND_SEGMENT_INDEX: usize = 359;
const ROM_DIALOGUE_FIRST_SEGMENT: u32 = 0x1c_8000;
const ROM_DIALOGUE_SECOND_SEGMENT: u32 = 0x0e_df40;
const ROM_DIALOGUE_TERMINATOR: u8 = 0x7f;

fn lorom_offset(address: u32) -> Option<usize> {
    let address_in_bank = address as u16;
    if address_in_bank < 0x8000 {
        return None;
    }
    Some(((address as usize >> 16) & 0x7f) * 0x8000 + usize::from(address_in_bank - 0x8000))
}

const fn next_lorom_address(mut address: u32) -> u32 {
    address += 1;
    if (address as u16) < 0x8000 {
        address += 0x8000;
    }
    address
}

fn rom_dialogue_message_pointers(rom: &[u8]) -> Result<[u32; ROM_DIALOGUE_MESSAGE_COUNT], String> {
    let mut pointers = [0; ROM_DIALOGUE_MESSAGE_COUNT];
    let mut address = ROM_DIALOGUE_FIRST_SEGMENT;
    for (index, pointer) in pointers.iter_mut().enumerate() {
        if index == ROM_DIALOGUE_SECOND_SEGMENT_INDEX {
            address = ROM_DIALOGUE_SECOND_SEGMENT;
        }
        *pointer = address;
        loop {
            let offset = lorom_offset(address)
                .ok_or_else(|| format!("dialogue address ${address:06x} is outside LoROM"))?;
            let byte = *rom.get(offset).ok_or_else(|| {
                format!("dialogue address ${address:06x} extends past the loaded ROM")
            })?;
            address = next_lorom_address(address);
            if byte == ROM_DIALOGUE_TERMINATOR {
                break;
            }
        }
    }
    Ok(pointers)
}

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
    pub(crate) waiting: bool,
    pub(crate) stack_address: u16,
    pub(crate) stack_bytes: &'static [u8],
}

/// Read/write-isolated execution of a translated routine's original ROM path.
///
/// The clone supplies instruction ordering and cycle timing only. Its WRAM,
/// SRAM, PPU, DMA, and APU-port mutations are discarded, leaving the translated
/// Rust implementation as the sole owner of game state.
#[derive(Clone)]
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
        apu_output_ports: [u8; 4],
        checkpoint: RomCpuCheckpoint,
    ) -> Result<Self, String> {
        let mut shadow = Snes::new();
        snes::load_rom(&mut shadow, rom).map_err(|error| error.to_string())?;
        shadow.ram.copy_from_slice(ram);
        shadow.cart.ram.copy_from_slice(sram);
        shadow.ppu = ppu.clone();
        shadow.dma = dma.clone();
        shadow.apu.out_ports = apu_output_ports;
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
        shadow.cpu.waiting = checkpoint.waiting;

        let stack_start = usize::from(checkpoint.stack_address);
        let stack_end = stack_start + checkpoint.stack_bytes.len();
        shadow.ram[stack_start..stack_end].copy_from_slice(checkpoint.stack_bytes);

        Ok(Self {
            shadow,
            stop_pc: checkpoint.stop_pc,
        })
    }

    /// Keep the shadow's `Interrupt_NMI` on the main thread while the ROM's
    /// poly thread is active. The handler's thread exit (`$00:82C7`: `REP #$30
    /// : TSC : TAX : LDA $1F0A : TCS : STX $1F0A : PLB ...`) swaps stacks with
    /// the poly thread, whose stack contents the shadow does not model; the
    /// caller's cycle budget hands the thread its IRQ-to-NMI slice instead.
    pub(crate) fn disable_nmi_thread_switch(&mut self) -> Result<(), String> {
        const SWAP_OFFSET: usize = 0x02ce; // $00:82CE in LoROM bank 0
        let rom = &mut self.shadow.cart.rom;
        let swap = rom
            .get(SWAP_OFFSET..SWAP_OFFSET + 4)
            .ok_or_else(|| "ROM too short for the NMI thread switch".to_string())?;
        if swap != [0x1b, 0x8e, 0x0a, 0x1f] {
            return Err(format!(
                "unexpected NMI thread-switch bytes at $00:82CE: {swap:02x?}"
            ));
        }
        // TCS → NOP, STX $1F0A → NOP NOP NOP.
        rom[SWAP_OFFSET..SWAP_OFFSET + 4].copy_from_slice(&[0xea, 0xea, 0xea, 0xea]);
        Ok(())
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.pc() == self.stop_pc
    }

    pub(crate) fn pc(&self) -> u32 {
        (u32::from(self.shadow.cpu.k) << 16) | u32::from(self.shadow.cpu.pc)
    }

    pub(crate) fn stack_pointer(&self) -> u16 {
        self.shadow.cpu.sp
    }

    /// Decode the innermost 24-bit return address exactly as the Snes9x trace
    /// oracle does. This identifies the translated semantic caller when NMI
    /// interrupts inside a shared long-running helper.
    pub(crate) fn stack_return_address(&self) -> u32 {
        let start = usize::from(self.shadow.cpu.sp.wrapping_add(1));
        u32::from(self.shadow.ram[start])
            | (u32::from(self.shadow.ram[(start + 1) & 0xffff]) << 8)
            | (u32::from(self.shadow.ram[(start + 2) & 0xffff]) << 16)
    }

    /// The shadow machine's whole WRAM image after the run so far.
    pub(crate) fn ram(&self) -> &[u8] {
        &self.shadow.ram
    }

    pub(crate) fn ram_byte(&self, address: usize) -> u8 {
        self.shadow.ram[address]
    }

    /// Replace the C port's compatibility-only dialogue pointers with the
    /// original compressed-ROM boundaries expected by `Text_LoadCharacterBuffer`.
    ///
    /// `Text_GenerateMessagePointers` correctly derives translated state from
    /// the C port's re-encoded semantic dialogue asset. A timing shadow instead
    /// executes the original ROM, whose dictionary-compressed strings have
    /// different byte lengths. Keeping this normalization inside the isolated
    /// shadow preserves native C behavior and gives ROM execution its own data.
    pub(crate) fn restore_original_dialogue_pointer_table(&mut self) -> Result<(), String> {
        let pointers = rom_dialogue_message_pointers(&self.shadow.cart.rom)?;
        for (index, pointer) in pointers.into_iter().enumerate() {
            let destination = TEXT_DIALOGUE_POINTERS + index * 3;
            self.shadow.ram[destination] = pointer as u8;
            self.shadow.ram[destination + 1] = (pointer >> 8) as u8;
            self.shadow.ram[destination + 2] = (pointer >> 16) as u8;
        }
        Ok(())
    }

    pub(crate) fn window1_bounds(&self) -> (u8, u8) {
        (self.shadow.ppu.window1_left, self.shadow.ppu.window1_right)
    }

    pub(crate) fn set_raster_position(&mut self, scanline: u16, master_cycle: u16) {
        self.shadow.v_pos = scanline;
        self.shadow.h_pos = master_cycle;
        self.shadow.in_vblank = scanline >= 225;
    }

    /// Request a hardware NMI at the current instruction boundary. The next
    /// `step` performs the 65816 interrupt entry through the ordinary timed CPU
    /// path, so vector reads, stack writes, and handler control flow all remain
    /// part of the measured shadow execution.
    pub(crate) fn request_nmi(&mut self) {
        self.shadow.in_nmi = true;
        self.shadow.in_vblank = true;
        self.shadow.cpu.nmi_wanted = true;
    }

    pub(crate) fn drain_started_dma_master_cycles(&mut self) -> u32 {
        self.shadow.dma_run_to_completion_master_cycles()
    }

    /// Run the cloned HDMA initialization event and report the pinned Snes9x
    /// 1.63 bus cost. The native DMA core uses the Zelda C port's 16-cycle sync;
    /// the oracle uses an 18-cycle CPU/DMA sync, hence the two-cycle adjustment.
    pub(crate) fn run_hdma_init_master_cycles(&mut self) -> u32 {
        self.shadow.dma_init_hdma();
        self.take_hdma_master_cycles()
    }

    /// Run one cloned HDMA scanline, including descriptor reloads and transfer
    /// widths from the live channel state, and report the pinned oracle cost.
    pub(crate) fn run_hdma_scanline_master_cycles(&mut self) -> u32 {
        self.shadow.dma_do_hdma();
        self.take_hdma_master_cycles()
    }

    fn take_hdma_master_cycles(&mut self) -> u32 {
        let master_cycles = u32::from(self.shadow.dma.hdma_timer);
        self.shadow.dma.hdma_timer = 0;
        master_cycles + u32::from(master_cycles != 0) * 2
    }

    pub(crate) fn step(&mut self) -> CpuInstructionTiming {
        if let Some(trace) = self.shadow.debug_cpu_write_trace.as_mut() {
            trace.clear();
        }
        snes::cpu_run_opcode_timed(&mut self.shadow)
    }

    pub(crate) fn enable_cpu_write_trace(&mut self) {
        self.shadow.debug_cpu_write_trace = Some(Vec::new());
    }

    /// Drain WRAM writes performed by the most recently stepped instruction.
    /// Both banks $7E/$7F and the low-bank $0000-$1FFF mirrors are normalized
    /// to offsets in the native 128 KiB WRAM image.
    pub(crate) fn take_cpu_wram_writes(&mut self) -> Vec<(usize, u8)> {
        self.shadow
            .debug_cpu_write_trace
            .as_mut()
            .expect("ROM timing CPU write trace is not enabled")
            .drain(..)
            .filter_map(|(address, value)| {
                let bank = (address >> 16) as u8;
                let offset = (address & 0xffff) as usize;
                match bank {
                    0x7e => Some((offset, value)),
                    0x7f => Some((0x1_0000 + offset, value)),
                    0x00..=0x3f | 0x80..=0xbf if offset < 0x2000 => Some((offset, value)),
                    _ => None,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconstructs_compressed_rom_message_boundaries_across_both_segments() {
        let mut rom = vec![0; lorom_offset(ROM_DIALOGUE_FIRST_SEGMENT).unwrap() + 0x4000];
        let mut expected = [0; ROM_DIALOGUE_MESSAGE_COUNT];
        let mut address = ROM_DIALOGUE_FIRST_SEGMENT;
        for (index, pointer) in expected.iter_mut().enumerate() {
            if index == ROM_DIALOGUE_SECOND_SEGMENT_INDEX {
                address = ROM_DIALOGUE_SECOND_SEGMENT;
            }
            *pointer = address;
            for _ in 0..=index % 4 {
                rom[lorom_offset(address).unwrap()] = 0x42;
                address = next_lorom_address(address);
            }
            rom[lorom_offset(address).unwrap()] = ROM_DIALOGUE_TERMINATOR;
            address = next_lorom_address(address);
        }

        let pointers = rom_dialogue_message_pointers(&rom).unwrap();
        assert_eq!(pointers, expected);
        assert_eq!(pointers[0], ROM_DIALOGUE_FIRST_SEGMENT);
        assert_eq!(
            pointers[ROM_DIALOGUE_SECOND_SEGMENT_INDEX],
            ROM_DIALOGUE_SECOND_SEGMENT
        );
    }
}
