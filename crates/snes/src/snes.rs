//! Top-level SNES system: aggregates CPU/PPU/DMA/APU/Cart/Input,
//! holds WRAM and the bus-state registers, dispatches reads/writes.
//!
//! Port of `zelda3/snes/snes.c`. The C layout has every sub-component
//! holding a back-pointer to `Snes` for bus access; we collapse that by
//! making bus reads/writes methods on `Snes` itself and letting the
//! sub-state structs be pure data.

use crate::apu::{ApuState, APU_SAVELOAD_PREFIX_SIZE, DSP_SAVELOAD_SIZE, SPC_SAVELOAD_SIZE};
use crate::cart::Cart;
use crate::cpu::CpuState;
use crate::dma::DmaState;
use crate::input::InputState;
use crate::ppu::PpuState;

pub const WRAM_SIZE: usize = 0x20000;
const SNES_CORE_SAVELOAD_SIZE: usize = 58;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Snes {
    pub cpu: CpuState,
    pub apu: ApuState,
    pub ppu: PpuState,
    pub dma: DmaState,
    pub cart: Cart,
    pub input1: InputState,
    pub input2: InputState,

    pub debug_cycles: bool,
    pub disable_hpos: bool,

    // frame timing
    pub h_pos: u16,
    pub v_pos: u16,
    pub frames: u32,

    // cpu handling
    pub cpu_cycles_left: u8,
    pub cpu_mem_ops: u8,
    pub apu_catchup_cycles: f64,

    // nmi / irq
    pub h_irq_enabled: bool,
    pub v_irq_enabled: bool,
    pub nmi_enabled: bool,
    pub h_timer: u16,
    pub v_timer: u16,
    pub in_nmi: bool,
    pub in_irq: bool,
    pub in_vblank: bool,

    // joypad
    pub port_auto_read: [u16; 4],
    pub auto_joy_read: bool,
    pub auto_joy_timer: u16,
    pub ppu_latch: bool,

    // multiplication / division
    pub multiply_a: u8,
    pub multiply_result: u16,
    pub divide_a: u16,
    pub divide_result: u16,

    // misc
    pub fast_mem: bool,
    pub open_bus: u8,

    // wram
    pub ram: Vec<u8>,
    pub ram_adr: u32,
}

impl Snes {
    pub fn new() -> Self {
        Self {
            cpu: CpuState::new(),
            apu: ApuState::new(),
            ppu: PpuState::new(),
            dma: DmaState::new(),
            cart: Cart::new(),
            input1: InputState::new(),
            input2: InputState::new(),
            debug_cycles: false,
            disable_hpos: false,
            h_pos: 0,
            v_pos: 0,
            frames: 0,
            cpu_cycles_left: 0,
            cpu_mem_ops: 0,
            apu_catchup_cycles: 0.0,
            h_irq_enabled: false,
            v_irq_enabled: false,
            nmi_enabled: false,
            h_timer: 0,
            v_timer: 0,
            in_nmi: false,
            in_irq: false,
            in_vblank: false,
            port_auto_read: [0; 4],
            auto_joy_read: false,
            auto_joy_timer: 0,
            ppu_latch: false,
            multiply_a: 0,
            multiply_result: 0,
            divide_a: 0,
            divide_result: 0,
            fast_mem: false,
            open_bus: 0,
            ram: vec![0; WRAM_SIZE],
            ram_adr: 0,
        }
    }

    /// `snes_reset` — order matters because `cpu_reset` reads the reset
    /// vector from $FFFC, which routes through the cart.
    pub fn reset(&mut self, hard: bool) {
        self.cart.reset();
        self.cpu.reset();
        self.apu.reset();
        self.dma.reset();
        self.ppu.reset();
        self.input1.reset();
        self.input2.reset();

        if hard {
            for b in &mut self.ram {
                *b = 0;
            }
        }

        self.ram_adr = 0;
        self.h_pos = 0;
        self.v_pos = 0;
        self.frames = 0;
        self.cpu_cycles_left = 52; // 5 reads (8) + 2 IntOp (6), per C
        self.cpu_mem_ops = 0;
        self.apu_catchup_cycles = 0.0;
        self.h_irq_enabled = false;
        self.v_irq_enabled = false;
        self.nmi_enabled = false;
        self.h_timer = 0x1ff;
        self.v_timer = 0x1ff;
        self.in_nmi = false;
        self.in_irq = false;
        self.in_vblank = false;
        self.port_auto_read = [0; 4];
        self.auto_joy_read = false;
        self.auto_joy_timer = 0;
        self.ppu_latch = false;
        self.multiply_a = 0xff;
        self.multiply_result = 0xfe01;
        self.divide_a = 0xffff;
        self.divide_result = 0x101;
        self.fast_mem = false;
        self.open_bus = 0;
    }

    /// Byte layout used by C `snes_saveload`.
    ///
    /// This aggregates the component C saveload blocks in the same order as
    /// `snes/snes.c`: CPU, APU prefix, DSP, SPC, DMA, PPU, cart SRAM, the
    /// contiguous `hPos..openBus` native block, WRAM, and `ramAdr`.
    pub fn save_c_saveload(&mut self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::C_SAVELOAD_SIZE);
        out.extend_from_slice(&self.cpu.save_c_saveload());
        out.extend_from_slice(&self.apu.save_c_saveload_prefix());
        out.extend_from_slice(&self.apu.dsp.save_c_saveload());
        out.extend_from_slice(&self.apu.spc.save_c_saveload());
        out.extend_from_slice(&self.dma.save_c_saveload());
        out.extend_from_slice(&self.ppu.save_c_saveload());
        out.extend_from_slice(&self.cart.save_c_saveload());
        out.extend_from_slice(&self.save_c_snes_core_block());
        out.extend_from_slice(&self.ram[..WRAM_SIZE]);
        out.extend_from_slice(&self.ram_adr.to_le_bytes());
        self.disable_hpos = false;
        debug_assert_eq!(out.len(), Self::C_SAVELOAD_SIZE);
        out
    }

    pub fn load_c_saveload(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != Self::C_SAVELOAD_SIZE {
            return Err(format!(
                "invalid SNES saveload size {}, expected {}",
                data.len(),
                Self::C_SAVELOAD_SIZE
            ));
        }

        let mut pos = 0usize;
        self.cpu
            .load_c_saveload(&data[pos..pos + CpuState::C_SAVELOAD_SIZE])?;
        pos += CpuState::C_SAVELOAD_SIZE;
        self.apu
            .load_c_saveload_prefix(&data[pos..pos + APU_SAVELOAD_PREFIX_SIZE])?;
        pos += APU_SAVELOAD_PREFIX_SIZE;
        self.apu
            .dsp
            .load_c_saveload(&data[pos..pos + DSP_SAVELOAD_SIZE])?;
        self.apu.dsp_regs.clone_from(&self.apu.dsp.ram);
        pos += DSP_SAVELOAD_SIZE;
        self.apu
            .spc
            .load_c_saveload(&data[pos..pos + SPC_SAVELOAD_SIZE])?;
        pos += SPC_SAVELOAD_SIZE;
        self.dma
            .load_c_saveload(&data[pos..pos + DmaState::C_SAVELOAD_SIZE])?;
        pos += DmaState::C_SAVELOAD_SIZE;
        self.ppu
            .load_c_saveload(&data[pos..pos + PpuState::C_SAVELOAD_SIZE])?;
        pos += PpuState::C_SAVELOAD_SIZE;
        self.cart
            .load_c_saveload(&data[pos..pos + Cart::C_SAVELOAD_SIZE])?;
        pos += Cart::C_SAVELOAD_SIZE;
        self.load_c_snes_core_block(&data[pos..pos + SNES_CORE_SAVELOAD_SIZE]);
        pos += SNES_CORE_SAVELOAD_SIZE;
        if self.ram.len() != WRAM_SIZE {
            self.ram.resize(WRAM_SIZE, 0);
        }
        self.ram.copy_from_slice(&data[pos..pos + WRAM_SIZE]);
        pos += WRAM_SIZE;
        self.ram_adr = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        self.disable_hpos = false;
        Ok(())
    }

    pub const C_SAVELOAD_SIZE: usize = CpuState::C_SAVELOAD_SIZE
        + APU_SAVELOAD_PREFIX_SIZE
        + DSP_SAVELOAD_SIZE
        + SPC_SAVELOAD_SIZE
        + DmaState::C_SAVELOAD_SIZE
        + PpuState::C_SAVELOAD_SIZE
        + Cart::C_SAVELOAD_SIZE
        + SNES_CORE_SAVELOAD_SIZE
        + WRAM_SIZE
        + 4;

    fn save_c_snes_core_block(&self) -> [u8; SNES_CORE_SAVELOAD_SIZE] {
        let mut out = [0; SNES_CORE_SAVELOAD_SIZE];
        put_u16(&mut out, 0, self.h_pos);
        put_u16(&mut out, 2, self.v_pos);
        put_u32(&mut out, 4, self.frames);
        out[8] = self.cpu_cycles_left;
        out[9] = self.cpu_mem_ops;
        out[16..24].copy_from_slice(&self.apu_catchup_cycles.to_le_bytes());
        out[24] = self.h_irq_enabled as u8;
        out[25] = self.v_irq_enabled as u8;
        out[26] = self.nmi_enabled as u8;
        put_u16(&mut out, 28, self.h_timer);
        put_u16(&mut out, 30, self.v_timer);
        out[32] = self.in_nmi as u8;
        out[33] = self.in_irq as u8;
        out[34] = self.in_vblank as u8;
        for (i, value) in self.port_auto_read.iter().enumerate() {
            put_u16(&mut out, 36 + i * 2, *value);
        }
        out[44] = self.auto_joy_read as u8;
        put_u16(&mut out, 46, self.auto_joy_timer);
        out[48] = self.ppu_latch as u8;
        out[49] = self.multiply_a;
        put_u16(&mut out, 50, self.multiply_result);
        put_u16(&mut out, 52, self.divide_a);
        put_u16(&mut out, 54, self.divide_result);
        out[56] = self.fast_mem as u8;
        out[57] = self.open_bus;
        out
    }

    fn load_c_snes_core_block(&mut self, data: &[u8]) {
        self.h_pos = get_u16(data, 0);
        self.v_pos = get_u16(data, 2);
        self.frames = get_u32(data, 4);
        self.cpu_cycles_left = data[8];
        self.cpu_mem_ops = data[9];
        self.apu_catchup_cycles = f64::from_le_bytes([
            data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23],
        ]);
        self.h_irq_enabled = data[24] != 0;
        self.v_irq_enabled = data[25] != 0;
        self.nmi_enabled = data[26] != 0;
        self.h_timer = get_u16(data, 28);
        self.v_timer = get_u16(data, 30);
        self.in_nmi = data[32] != 0;
        self.in_irq = data[33] != 0;
        self.in_vblank = data[34] != 0;
        for (i, value) in self.port_auto_read.iter_mut().enumerate() {
            *value = get_u16(data, 36 + i * 2);
        }
        self.auto_joy_read = data[44] != 0;
        self.auto_joy_timer = get_u16(data, 46);
        self.ppu_latch = data[48] != 0;
        self.multiply_a = data[49];
        self.multiply_result = get_u16(data, 50);
        self.divide_a = get_u16(data, 52);
        self.divide_result = get_u16(data, 54);
        self.fast_mem = data[56] != 0;
        self.open_bus = data[57];
    }

    /// `snes_doAutoJoypad` — shifts each input's serial line 16 times
    /// to assemble the auto-joypad port words.
    pub fn do_auto_joypad(&mut self) {
        self.port_auto_read = [0; 4];
        self.input1.latch_line = true;
        self.input2.latch_line = true;
        self.input1.cycle();
        self.input2.cycle();
        self.input1.latch_line = false;
        self.input2.latch_line = false;
        for i in 0..16 {
            let v1 = self.input1.read();
            self.port_auto_read[0] |= ((v1 & 1) as u16) << (15 - i);
            self.port_auto_read[2] |= (((v1 >> 1) & 1) as u16) << (15 - i);
            let v2 = self.input2.read();
            self.port_auto_read[1] |= ((v2 & 1) as u16) << (15 - i);
            self.port_auto_read[3] |= (((v2 >> 1) & 1) as u16) << (15 - i);
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // B-Bus ($21xx, mapped from $2100..$21ff)
    // ──────────────────────────────────────────────────────────────────

    pub fn read_b_bus(&mut self, adr: u8) -> u8 {
        if adr < 0x40 {
            return self.ppu.read(adr);
        }
        if adr < 0x80 {
            return self.apu.read_snes_port(adr);
        }
        if adr == 0x80 {
            let v = self.ram[self.ram_adr as usize];
            self.ram_adr = (self.ram_adr + 1) & 0x1ffff;
            return v;
        }
        self.open_bus
    }

    pub fn write_b_bus(&mut self, adr: u8, val: u8) {
        if adr < 0x40 {
            self.ppu.write(adr, val);
            return;
        }
        if adr < 0x80 {
            self.catchup_apu();
            self.apu.write_snes_port(adr, val);
            return;
        }
        match adr {
            0x80 => {
                self.ram[self.ram_adr as usize] = val;
                self.ram_adr = (self.ram_adr + 1) & 0x1ffff;
            }
            0x81 => self.ram_adr = (self.ram_adr & 0x1ff00) | val as u32,
            0x82 => self.ram_adr = (self.ram_adr & 0x100ff) | ((val as u32) << 8),
            0x83 => self.ram_adr = (self.ram_adr & 0x0ffff) | (((val & 1) as u32) << 16),
            _ => {}
        }
    }

    fn catchup_apu(&mut self) {
        let catchup = self.apu_catchup_cycles as i32;
        for _ in 0..catchup {
            self.apu.cycle();
        }
        self.apu_catchup_cycles -= catchup as f64;
    }

    // ──────────────────────────────────────────────────────────────────
    // Internal registers ($4200..$421f)
    // ──────────────────────────────────────────────────────────────────

    fn read_reg(&mut self, adr: u16) -> u8 {
        match adr {
            0x4210 => {
                let mut v = 0x2u8; // CPU version
                v |= (self.in_nmi as u8) << 7;
                self.in_nmi = false;
                v | (self.open_bus & 0x70)
            }
            0x4211 => {
                let v = (self.in_irq as u8) << 7;
                self.in_irq = false;
                self.cpu.irq_wanted = false;
                v | (self.open_bus & 0x7f)
            }
            0x4212 => {
                let mut v = (self.auto_joy_timer > 0) as u8;
                v |= ((self.h_pos >= 1024) as u8) << 6;
                v |= (self.in_vblank as u8) << 7;
                v | (self.open_bus & 0x3e)
            }
            0x4213 => (self.ppu_latch as u8) << 7,
            0x4214 => (self.divide_result & 0xff) as u8,
            0x4215 => (self.divide_result >> 8) as u8,
            0x4216 => (self.multiply_result & 0xff) as u8,
            0x4217 => (self.multiply_result >> 8) as u8,
            0x4218 | 0x421a | 0x421c | 0x421e => {
                (self.port_auto_read[((adr - 0x4218) / 2) as usize] & 0xff) as u8
            }
            0x4219 | 0x421b | 0x421d | 0x421f => {
                (self.port_auto_read[((adr - 0x4219) / 2) as usize] >> 8) as u8
            }
            _ => self.open_bus,
        }
    }

    fn write_reg(&mut self, adr: u16, val: u8) {
        match adr {
            0x4200 => {
                self.auto_joy_read = val & 0x01 != 0;
                if !self.auto_joy_read {
                    self.auto_joy_timer = 0;
                }
                self.h_irq_enabled = val & 0x10 != 0;
                self.v_irq_enabled = val & 0x20 != 0;
                self.nmi_enabled = val & 0x80 != 0;
                if !self.h_irq_enabled && !self.v_irq_enabled {
                    self.in_irq = false;
                    self.cpu.irq_wanted = false;
                }
            }
            0x4201 => {
                if val & 0x80 == 0 && self.ppu_latch {
                    let _ = self.ppu.read(0x37); // latches PPU
                }
                self.ppu_latch = val & 0x80 != 0;
            }
            0x4202 => self.multiply_a = val,
            0x4203 => {
                self.multiply_result = (self.multiply_a as u16).wrapping_mul(val as u16);
            }
            0x4204 => self.divide_a = (self.divide_a & 0xff00) | val as u16,
            0x4205 => self.divide_a = (self.divide_a & 0x00ff) | ((val as u16) << 8),
            0x4206 => {
                if val == 0 {
                    self.divide_result = 0xffff;
                    self.multiply_result = self.divide_a;
                } else {
                    self.divide_result = self.divide_a / val as u16;
                    self.multiply_result = self.divide_a % val as u16;
                }
            }
            0x4207 => self.h_timer = (self.h_timer & 0x100) | val as u16,
            0x4208 => self.h_timer = (self.h_timer & 0x0ff) | (((val & 1) as u16) << 8),
            0x4209 => self.v_timer = (self.v_timer & 0x100) | val as u16,
            0x420a => self.v_timer = (self.v_timer & 0x0ff) | (((val & 1) as u16) << 8),
            0x420b => self.dma_start(val, false),
            0x420c => self.dma_start(val, true),
            0x420d => self.fast_mem = val & 0x1 != 0,
            _ => {}
        }
    }

    fn dma_start(&mut self, val: u8, hdma: bool) {
        self.dma_start_real(val, hdma);
    }

    fn dma_read(&self, adr: u16) -> u8 {
        self.dma_read_reg(adr)
    }

    fn dma_write(&mut self, adr: u16, val: u8) {
        self.dma_write_reg(adr, val);
    }

    // ──────────────────────────────────────────────────────────────────
    // Full 24-bit address bus
    // ──────────────────────────────────────────────────────────────────

    fn raw_read(&mut self, full_adr: u32) -> u8 {
        let bank = (full_adr >> 16) as u8;
        let adr = (full_adr & 0xffff) as u16;
        if (bank & 0x7f) < 0x40 && adr < 0x4380 {
            if adr < 0x2000 {
                return self.ram[adr as usize]; // ram mirror
            }
            if (0x2100..0x2200).contains(&adr) {
                return self.read_b_bus((adr & 0xff) as u8);
            }
            if adr == 0x4016 {
                return self.input1.read() | (self.open_bus & 0xfc);
            }
            if adr == 0x4017 {
                return self.input2.read() | (self.open_bus & 0xe0) | 0x1c;
            }
            if (0x4200..0x4220).contains(&adr) {
                return self.read_reg(adr);
            }
            if (0x4300..0x4380).contains(&adr) {
                return self.dma_read(adr);
            }
        } else if (bank & !1) == 0x7e {
            // bank 7e/7f — direct WRAM
            let idx = (((bank as usize) & 1) << 16) | adr as usize;
            return self.ram[idx];
        }
        let open_bus = self.open_bus;
        self.cart.read(bank, adr, open_bus)
    }

    pub fn read(&mut self, full_adr: u32) -> u8 {
        let v = self.raw_read(full_adr);
        self.open_bus = v;
        v
    }

    pub fn write(&mut self, full_adr: u32, val: u8) {
        self.open_bus = val;
        let bank = (full_adr >> 16) as u8;
        let adr = (full_adr & 0xffff) as u16;
        if bank == 0x7e || bank == 0x7f {
            let idx = (((bank as usize) & 1) << 16) | adr as usize;
            self.ram[idx] = val;
        } else if bank < 0x40 || (0x80..0xc0).contains(&bank) {
            if adr < 0x2000 {
                self.ram[adr as usize] = val;
            } else if (0x2100..0x2200).contains(&adr) {
                self.write_b_bus((adr & 0xff) as u8, val);
            } else if adr == 0x4016 {
                let lo = val & 1 != 0;
                self.input1.latch_line = lo;
                self.input2.latch_line = lo;
            } else if (0x4200..0x4220).contains(&adr) {
                self.write_reg(adr, val);
            } else if (0x4300..0x4380).contains(&adr) {
                self.dma_write(adr, val);
            }
        }
        self.cart.write(bank, adr, val);
    }

    fn access_time(&self, _full_adr: u32) -> u8 {
        // optimization matching `snes_getAccessTime` in C — flat 6.
        6
    }

    pub fn cpu_read(&mut self, full_adr: u32) -> u8 {
        self.cpu_mem_ops = self.cpu_mem_ops.wrapping_add(1);
        self.cpu_cycles_left = self
            .cpu_cycles_left
            .wrapping_add(self.access_time(full_adr));
        self.read(full_adr)
    }

    pub fn cpu_write(&mut self, full_adr: u32, val: u8) {
        self.cpu_mem_ops = self.cpu_mem_ops.wrapping_add(1);
        self.cpu_cycles_left = self
            .cpu_cycles_left
            .wrapping_add(self.access_time(full_adr));
        self.write(full_adr, val);
    }
}

fn put_u16(out: &mut [u8], offset: usize, value: u16) {
    out[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut [u8], offset: usize, value: u32) {
    out[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn get_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn get_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

impl Default for Snes {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wram_mirror_at_low_addresses() {
        let mut snes = Snes::new();
        snes.write(0x0000_1234, 0xab);
        assert_eq!(snes.read(0x0000_1234), 0xab);
        // banks 7e/7f mirror the same low WRAM at offset 0x1234.
        assert_eq!(snes.read(0x007e_1234), 0xab);
    }

    #[test]
    fn wram_data_port_walks_address() {
        let mut snes = Snes::new();
        snes.write(0x0000_2181, 0x00);
        snes.write(0x0000_2182, 0x00);
        snes.write(0x0000_2183, 0x00);
        // write three bytes via $2180, then read them back.
        snes.write(0x0000_2180, 0x11);
        snes.write(0x0000_2180, 0x22);
        snes.write(0x0000_2180, 0x33);
        // rewind address pointer
        snes.write(0x0000_2181, 0x00);
        snes.write(0x0000_2182, 0x00);
        snes.write(0x0000_2183, 0x00);
        assert_eq!(snes.read(0x0000_2180), 0x11);
        assert_eq!(snes.read(0x0000_2180), 0x22);
        assert_eq!(snes.read(0x0000_2180), 0x33);
    }

    #[test]
    fn multiply_register_works() {
        let mut snes = Snes::new();
        snes.write(0x0000_4202, 0x10);
        snes.write(0x0000_4203, 0x20);
        assert_eq!(snes.read(0x0000_4216), 0x00);
        assert_eq!(snes.read(0x0000_4217), 0x02); // 0x10 * 0x20 = 0x0200
    }

    #[test]
    fn divide_register_works() {
        let mut snes = Snes::new();
        snes.write(0x0000_4204, 100); // lo
        snes.write(0x0000_4205, 0); // hi
        snes.write(0x0000_4206, 7);
        // quotient = 14, remainder = 2
        assert_eq!(snes.read(0x0000_4214), 14);
        assert_eq!(snes.read(0x0000_4215), 0);
        assert_eq!(snes.read(0x0000_4216), 2);
    }

    #[test]
    fn divide_by_zero_matches_c() {
        let mut snes = Snes::new();
        snes.write(0x0000_4204, 0x34);
        snes.write(0x0000_4205, 0x12);
        snes.write(0x0000_4206, 0); // div by zero
        assert_eq!(snes.read(0x0000_4214), 0xff);
        assert_eq!(snes.read(0x0000_4215), 0xff);
        assert_eq!(snes.read(0x0000_4216), 0x34);
        assert_eq!(snes.read(0x0000_4217), 0x12);
    }
}
