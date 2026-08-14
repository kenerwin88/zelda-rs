//! 65816 opcode dispatch + addressing modes. Port of the body of
//! `zelda3/snes/cpu.c`. State lives in [`crate::cpu::CpuState`]; this
//! module is the behavior.
//!
//! Helpers are methods on [`Snes`] because almost every opcode needs
//! the memory bus, which has access to PPU/DMA/Cart/RAM. The big
//! `match` in [`cpu_run_opcode`] mirrors the C `cpu_doOpcode` switch
//! 1:1 to keep the verification oracle happy.

use crate::snes::Snes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuInstructionTiming {
    pub cpu_cycles: u8,
    pub bus_accesses: u8,
    pub master_cycles: u32,
}

const CYCLES_PER_OPCODE: [u8; 256] = [
    7, 6, 7, 4, 5, 3, 5, 6, 3, 2, 2, 4, 6, 4, 6, 5, //
    2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 2, 2, 6, 4, 7, 5, //
    6, 6, 8, 4, 3, 3, 5, 6, 4, 2, 2, 5, 4, 4, 6, 5, //
    2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 2, 2, 4, 4, 7, 5, //
    6, 6, 2, 4, 7, 3, 5, 6, 3, 2, 2, 3, 3, 4, 6, 5, //
    2, 5, 5, 7, 7, 4, 6, 6, 2, 4, 3, 2, 4, 4, 7, 5, //
    6, 6, 6, 4, 3, 3, 5, 6, 4, 2, 2, 6, 5, 4, 6, 5, //
    2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 4, 2, 6, 4, 7, 5, //
    3, 6, 4, 4, 3, 3, 3, 6, 2, 2, 2, 3, 4, 4, 4, 5, //
    2, 6, 5, 7, 4, 4, 4, 6, 2, 5, 2, 2, 4, 5, 5, 5, //
    2, 6, 2, 4, 3, 3, 3, 6, 2, 2, 2, 4, 4, 4, 4, 5, //
    2, 5, 5, 7, 4, 4, 4, 6, 2, 4, 2, 2, 4, 4, 4, 5, //
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5, //
    2, 5, 5, 7, 6, 4, 6, 6, 2, 4, 3, 3, 6, 4, 7, 5, //
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5, //
    2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 4, 2, 8, 4, 7, 5, //
];

impl Snes {
    // ── primitive helpers ────────────────────────────────────────────

    fn cpu_set_flags(&mut self, val: u8) {
        self.cpu.unpack_flags(val);
        if self.cpu.e {
            self.cpu.mf = true;
            self.cpu.xf = true;
            self.cpu.sp = (self.cpu.sp & 0xff) | 0x100;
        }
        if self.cpu.xf {
            self.cpu.x &= 0xff;
            self.cpu.y &= 0xff;
        }
    }

    fn cpu_set_zn(&mut self, value: u32, byte: bool) {
        if byte {
            self.cpu.z = (value & 0xff) == 0;
            self.cpu.n = value & 0x80 != 0;
        } else {
            self.cpu.z = (value & 0xffff) == 0;
            self.cpu.n = value & 0x8000 != 0;
        }
    }

    fn cpu_do_branch(&mut self, offset: u8, check: bool) {
        if check {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            self.cpu.pc = self.cpu.pc.wrapping_add(offset as i8 as i16 as u16);
        }
    }

    fn cpu_read_opcode_byte(&mut self) -> u8 {
        let addr = ((self.cpu.k as u32) << 16) | self.cpu.pc as u32;
        self.cpu.pc = self.cpu.pc.wrapping_add(1);
        self.cpu_read(addr)
    }

    fn cpu_read_opcode_word(&mut self) -> u16 {
        let lo = self.cpu_read_opcode_byte() as u16;
        let hi = self.cpu_read_opcode_byte() as u16;
        lo | (hi << 8)
    }

    fn cpu_read_word(&mut self, adrl: u32, adrh: u32) -> u16 {
        let lo = self.cpu_read(adrl) as u16;
        let hi = self.cpu_read(adrh) as u16;
        lo | (hi << 8)
    }

    fn cpu_write_word(&mut self, adrl: u32, adrh: u32, value: u16, reversed: bool) {
        if reversed {
            self.cpu_write(adrh, (value >> 8) as u8);
            self.cpu_write(adrl, value as u8);
        } else {
            self.cpu_write(adrl, value as u8);
            self.cpu_write(adrh, (value >> 8) as u8);
        }
    }

    fn cpu_pull_byte(&mut self) -> u8 {
        self.cpu.sp = self.cpu.sp.wrapping_add(1);
        if self.cpu.e {
            self.cpu.sp = (self.cpu.sp & 0xff) | 0x100;
        }
        self.cpu_read(self.cpu.sp as u32)
    }

    fn cpu_push_byte(&mut self, value: u8) {
        self.cpu_write(self.cpu.sp as u32, value);
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
        if self.cpu.e {
            self.cpu.sp = (self.cpu.sp & 0xff) | 0x100;
        }
    }

    fn cpu_pull_word(&mut self) -> u16 {
        let lo = self.cpu_pull_byte() as u16;
        let hi = self.cpu_pull_byte() as u16;
        lo | (hi << 8)
    }

    fn cpu_push_word(&mut self, value: u16) {
        self.cpu_push_byte((value >> 8) as u8);
        self.cpu_push_byte(value as u8);
    }

    fn cpu_do_interrupt(&mut self, irq: bool) {
        self.cpu_push_byte(self.cpu.k);
        let pc = self.cpu.pc;
        self.cpu_push_word(pc);
        let flags = self.cpu.pack_flags();
        self.cpu_push_byte(flags);
        self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        self.cpu.i = true;
        self.cpu.d = false;
        self.cpu.k = 0;
        self.cpu.pc = if irq {
            self.cpu_read_word(0xffee, 0xffef)
        } else {
            self.cpu_read_word(0xffea, 0xffeb)
        };
    }

    // ── addressing modes (return (low, high)) ────────────────────────

    fn adr_imm(&mut self, x_flag: bool) -> (u32, u32) {
        // Returns (addr_of_low_byte, addr_of_high_byte). If the operand
        // is 8-bit, high is unused; we still produce a real address so
        // the API stays uniform.
        let m_is_byte = if x_flag { self.cpu.xf } else { self.cpu.mf };
        let low = ((self.cpu.k as u32) << 16) | self.cpu.pc as u32;
        self.cpu.pc = self.cpu.pc.wrapping_add(1);
        if m_is_byte {
            (low, 0)
        } else {
            let high = ((self.cpu.k as u32) << 16) | self.cpu.pc as u32;
            self.cpu.pc = self.cpu.pc.wrapping_add(1);
            (low, high)
        }
    }

    fn adr_dp(&mut self) -> (u32, u32) {
        let adr = self.cpu_read_opcode_byte();
        if self.cpu.dp & 0xff != 0 {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        }
        let low = (self.cpu.dp.wrapping_add(adr as u16)) as u32;
        let high = (self.cpu.dp.wrapping_add(adr as u16).wrapping_add(1)) as u32;
        (low, high)
    }

    fn adr_dpx(&mut self) -> (u32, u32) {
        let adr = self.cpu_read_opcode_byte();
        if self.cpu.dp & 0xff != 0 {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        }
        let base = self
            .cpu
            .dp
            .wrapping_add(adr as u16)
            .wrapping_add(self.cpu.x);
        (base as u32, base.wrapping_add(1) as u32)
    }

    fn adr_dpy(&mut self) -> (u32, u32) {
        let adr = self.cpu_read_opcode_byte();
        if self.cpu.dp & 0xff != 0 {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        }
        let base = self
            .cpu
            .dp
            .wrapping_add(adr as u16)
            .wrapping_add(self.cpu.y);
        (base as u32, base.wrapping_add(1) as u32)
    }

    fn adr_idp(&mut self) -> (u32, u32) {
        let adr = self.cpu_read_opcode_byte();
        if self.cpu.dp & 0xff != 0 {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        }
        let ptr_lo = self.cpu.dp.wrapping_add(adr as u16) as u32;
        let ptr_hi = self.cpu.dp.wrapping_add(adr as u16).wrapping_add(1) as u32;
        let pointer = self.cpu_read_word(ptr_lo, ptr_hi) as u32;
        let base = ((self.cpu.db as u32) << 16) + pointer;
        (base, (base + 1) & 0xffffff)
    }

    fn adr_idx(&mut self) -> (u32, u32) {
        let adr = self.cpu_read_opcode_byte();
        if self.cpu.dp & 0xff != 0 {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        }
        let ptr_lo = self
            .cpu
            .dp
            .wrapping_add(adr as u16)
            .wrapping_add(self.cpu.x) as u32;
        let ptr_hi = self
            .cpu
            .dp
            .wrapping_add(adr as u16)
            .wrapping_add(self.cpu.x)
            .wrapping_add(1) as u32;
        let pointer = self.cpu_read_word(ptr_lo, ptr_hi) as u32;
        let base = ((self.cpu.db as u32) << 16) + pointer;
        (base, (base + 1) & 0xffffff)
    }

    fn adr_idy(&mut self, write: bool) -> (u32, u32) {
        let adr = self.cpu_read_opcode_byte();
        if self.cpu.dp & 0xff != 0 {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        }
        let ptr_lo = self.cpu.dp.wrapping_add(adr as u16) as u32;
        let ptr_hi = self.cpu.dp.wrapping_add(adr as u16).wrapping_add(1) as u32;
        let pointer = self.cpu_read_word(ptr_lo, ptr_hi) as u32;
        let with_y = pointer.wrapping_add(self.cpu.y as u32);
        if !write && (!self.cpu.xf || ((pointer >> 8) != (with_y >> 8))) {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        }
        let base = (((self.cpu.db as u32) << 16) + with_y) & 0xffffff;
        (base, (base + 1) & 0xffffff)
    }

    fn adr_idl(&mut self) -> (u32, u32) {
        let adr = self.cpu_read_opcode_byte();
        if self.cpu.dp & 0xff != 0 {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        }
        let ptr_lo = self.cpu.dp.wrapping_add(adr as u16) as u32;
        let ptr_hi = self.cpu.dp.wrapping_add(adr as u16).wrapping_add(1) as u32;
        let ptr_bank = self.cpu.dp.wrapping_add(adr as u16).wrapping_add(2) as u32;
        let mut pointer = self.cpu_read_word(ptr_lo, ptr_hi) as u32;
        pointer |= (self.cpu_read(ptr_bank) as u32) << 16;
        (pointer, (pointer + 1) & 0xffffff)
    }

    fn adr_ily(&mut self) -> (u32, u32) {
        let (mut low, _) = self.adr_idl();
        // adr_idl already added pointer to (low, high); we need pointer+Y.
        // Re-derive: extract pointer, add Y.
        // Simpler to inline:
        // — undo the +0 add we did and add Y. We can just shift by Y.
        low = (low.wrapping_add(self.cpu.y as u32)) & 0xffffff;
        (low, (low + 1) & 0xffffff)
    }

    fn adr_sr(&mut self) -> (u32, u32) {
        let adr = self.cpu_read_opcode_byte();
        let base = self.cpu.sp.wrapping_add(adr as u16);
        (base as u32, base.wrapping_add(1) as u32)
    }

    fn adr_isy(&mut self) -> (u32, u32) {
        let adr = self.cpu_read_opcode_byte();
        let ptr_lo = self.cpu.sp.wrapping_add(adr as u16) as u32;
        let ptr_hi = self.cpu.sp.wrapping_add(adr as u16).wrapping_add(1) as u32;
        let pointer = self.cpu_read_word(ptr_lo, ptr_hi) as u32;
        let base = (((self.cpu.db as u32) << 16) + pointer + self.cpu.y as u32) & 0xffffff;
        (base, (base + 1) & 0xffffff)
    }

    fn adr_abs(&mut self) -> (u32, u32) {
        let adr = self.cpu_read_opcode_word();
        let base = ((self.cpu.db as u32) << 16) + adr as u32;
        (base, (base + 1) & 0xffffff)
    }

    fn adr_abx(&mut self, write: bool) -> (u32, u32) {
        let adr = self.cpu_read_opcode_word();
        let with_x = (adr as u32).wrapping_add(self.cpu.x as u32);
        if !write && (!self.cpu.xf || ((adr >> 8) as u32 != (with_x >> 8))) {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        }
        let base = (((self.cpu.db as u32) << 16) + with_x) & 0xffffff;
        (base, (base + 1) & 0xffffff)
    }

    fn adr_aby(&mut self, write: bool) -> (u32, u32) {
        let adr = self.cpu_read_opcode_word();
        let with_y = (adr as u32).wrapping_add(self.cpu.y as u32);
        if !write && (!self.cpu.xf || ((adr >> 8) as u32 != (with_y >> 8))) {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
        }
        let base = (((self.cpu.db as u32) << 16) + with_y) & 0xffffff;
        (base, (base + 1) & 0xffffff)
    }

    fn adr_abl(&mut self) -> (u32, u32) {
        let mut adr = self.cpu_read_opcode_word() as u32;
        adr |= (self.cpu_read_opcode_byte() as u32) << 16;
        (adr, (adr + 1) & 0xffffff)
    }

    fn adr_alx(&mut self) -> (u32, u32) {
        let mut adr = self.cpu_read_opcode_word() as u32;
        adr |= (self.cpu_read_opcode_byte() as u32) << 16;
        let base = (adr.wrapping_add(self.cpu.x as u32)) & 0xffffff;
        (base, (base + 1) & 0xffffff)
    }

    fn adr_iax(&mut self) -> u16 {
        let adr = self.cpu_read_opcode_word();
        let lo = ((self.cpu.k as u32) << 16) | adr.wrapping_add(self.cpu.x) as u32;
        let hi = ((self.cpu.k as u32) << 16) | adr.wrapping_add(self.cpu.x).wrapping_add(1) as u32;
        self.cpu_read_word(lo, hi)
    }

    // ── opcode bodies ────────────────────────────────────────────────

    fn op_and(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let v = self.cpu_read(low);
            self.cpu.a = (self.cpu.a & 0xff00) | (((self.cpu.a as u8) & v) as u16);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            let v = self.cpu_read_word(low, high);
            self.cpu.a &= v;
        }
        self.cpu_set_zn(self.cpu.a as u32, self.cpu.mf);
    }

    fn op_ora(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let v = self.cpu_read(low);
            self.cpu.a = (self.cpu.a & 0xff00) | (((self.cpu.a as u8) | v) as u16);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            let v = self.cpu_read_word(low, high);
            self.cpu.a |= v;
        }
        self.cpu_set_zn(self.cpu.a as u32, self.cpu.mf);
    }

    fn op_eor(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let v = self.cpu_read(low);
            self.cpu.a = (self.cpu.a & 0xff00) | (((self.cpu.a as u8) ^ v) as u16);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            let v = self.cpu_read_word(low, high);
            self.cpu.a ^= v;
        }
        self.cpu_set_zn(self.cpu.a as u32, self.cpu.mf);
    }

    fn op_adc(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let value = self.cpu_read(low) as i32;
            let a = self.cpu.a as i32;
            let mut result;
            if self.cpu.d {
                result = (a & 0xf) + (value & 0xf) + self.cpu.c as i32;
                if result > 0x9 {
                    result = ((result + 0x6) & 0xf) + 0x10;
                }
                result = (a & 0xf0) + (value & 0xf0) + result;
            } else {
                result = (a & 0xff) + value + self.cpu.c as i32;
            }
            self.cpu.v = (a & 0x80) == (value & 0x80) && (value & 0x80) != (result & 0x80);
            if self.cpu.d && result > 0x9f {
                result += 0x60;
            }
            self.cpu.c = result > 0xff;
            self.cpu.a = (self.cpu.a & 0xff00) | (result as u16 & 0xff);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            let value = self.cpu_read_word(low, high) as i32;
            let a = self.cpu.a as i32;
            let mut result;
            if self.cpu.d {
                result = (a & 0xf) + (value & 0xf) + self.cpu.c as i32;
                if result > 0x9 {
                    result = ((result + 0x6) & 0xf) + 0x10;
                }
                result = (a & 0xf0) + (value & 0xf0) + result;
                if result > 0x9f {
                    result = ((result + 0x60) & 0xff) + 0x100;
                }
                result = (a & 0xf00) + (value & 0xf00) + result;
                if result > 0x9ff {
                    result = ((result + 0x600) & 0xfff) + 0x1000;
                }
                result = (a & 0xf000) + (value & 0xf000) + result;
            } else {
                result = a + value + self.cpu.c as i32;
            }
            self.cpu.v = (a & 0x8000) == (value & 0x8000) && (value & 0x8000) != (result & 0x8000);
            if self.cpu.d && result > 0x9fff {
                result += 0x6000;
            }
            self.cpu.c = result > 0xffff;
            self.cpu.a = result as u16;
        }
        self.cpu_set_zn(self.cpu.a as u32, self.cpu.mf);
    }

    fn op_sbc(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let value = (self.cpu_read(low) ^ 0xff) as i32;
            let a = self.cpu.a as i32;
            let mut result;
            if self.cpu.d {
                result = (a & 0xf) + (value & 0xf) + self.cpu.c as i32;
                if result < 0x10 {
                    let pre = result - 0x6;
                    let mask = if pre < 0 { 0xf } else { 0x1f };
                    result = pre & mask;
                }
                result = (a & 0xf0) + (value & 0xf0) + result;
            } else {
                result = (a & 0xff) + value + self.cpu.c as i32;
            }
            self.cpu.v = (a & 0x80) == (value & 0x80) && (value & 0x80) != (result & 0x80);
            if self.cpu.d && result < 0x100 {
                result -= 0x60;
            }
            self.cpu.c = result > 0xff;
            self.cpu.a = (self.cpu.a & 0xff00) | (result as u16 & 0xff);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            let value = (self.cpu_read_word(low, high) ^ 0xffff) as i32;
            let a = self.cpu.a as i32;
            let mut result;
            if self.cpu.d {
                result = (a & 0xf) + (value & 0xf) + self.cpu.c as i32;
                if result < 0x10 {
                    let pre = result - 0x6;
                    let mask = if pre < 0 { 0xf } else { 0x1f };
                    result = pre & mask;
                }
                result = (a & 0xf0) + (value & 0xf0) + result;
                if result < 0x100 {
                    let pre = result - 0x60;
                    let mask = if pre < 0 { 0xff } else { 0x1ff };
                    result = pre & mask;
                }
                result = (a & 0xf00) + (value & 0xf00) + result;
                if result < 0x1000 {
                    let pre = result - 0x600;
                    let mask = if pre < 0 { 0xfff } else { 0x1fff };
                    result = pre & mask;
                }
                result = (a & 0xf000) + (value & 0xf000) + result;
            } else {
                result = a + value + self.cpu.c as i32;
            }
            self.cpu.v = (a & 0x8000) == (value & 0x8000) && (value & 0x8000) != (result & 0x8000);
            if self.cpu.d && result < 0x10000 {
                result -= 0x6000;
            }
            self.cpu.c = result > 0xffff;
            self.cpu.a = result as u16;
        }
        self.cpu_set_zn(self.cpu.a as u32, self.cpu.mf);
    }

    fn op_cmp(&mut self, low: u32, high: u32) {
        let result;
        if self.cpu.mf {
            let v = (self.cpu_read(low) ^ 0xff) as i32;
            result = (self.cpu.a as i32 & 0xff) + v + 1;
            self.cpu.c = result > 0xff;
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            let v = (self.cpu_read_word(low, high) ^ 0xffff) as i32;
            result = self.cpu.a as i32 + v + 1;
            self.cpu.c = result > 0xffff;
        }
        self.cpu_set_zn(result as u32, self.cpu.mf);
    }

    fn op_cpx(&mut self, low: u32, high: u32) {
        let result;
        if self.cpu.xf {
            let v = (self.cpu_read(low) ^ 0xff) as i32;
            result = (self.cpu.x as i32 & 0xff) + v + 1;
            self.cpu.c = result > 0xff;
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            let v = (self.cpu_read_word(low, high) ^ 0xffff) as i32;
            result = self.cpu.x as i32 + v + 1;
            self.cpu.c = result > 0xffff;
        }
        self.cpu_set_zn(result as u32, self.cpu.xf);
    }

    fn op_cpy(&mut self, low: u32, high: u32) {
        let result;
        if self.cpu.xf {
            let v = (self.cpu_read(low) ^ 0xff) as i32;
            result = (self.cpu.y as i32 & 0xff) + v + 1;
            self.cpu.c = result > 0xff;
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            let v = (self.cpu_read_word(low, high) ^ 0xffff) as i32;
            result = self.cpu.y as i32 + v + 1;
            self.cpu.c = result > 0xffff;
        }
        self.cpu_set_zn(result as u32, self.cpu.xf);
    }

    fn op_bit(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let v = self.cpu_read(low);
            let result = (self.cpu.a as u8) & v;
            self.cpu.z = result == 0;
            self.cpu.n = v & 0x80 != 0;
            self.cpu.v = v & 0x40 != 0;
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            let v = self.cpu_read_word(low, high);
            let result = self.cpu.a & v;
            self.cpu.z = result == 0;
            self.cpu.n = v & 0x8000 != 0;
            self.cpu.v = v & 0x4000 != 0;
        }
    }

    fn op_lda(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            self.cpu.a = (self.cpu.a & 0xff00) | self.cpu_read(low) as u16;
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            self.cpu.a = self.cpu_read_word(low, high);
        }
        self.cpu_set_zn(self.cpu.a as u32, self.cpu.mf);
    }

    fn op_ldx(&mut self, low: u32, high: u32) {
        if self.cpu.xf {
            self.cpu.x = self.cpu_read(low) as u16;
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            self.cpu.x = self.cpu_read_word(low, high);
        }
        self.cpu_set_zn(self.cpu.x as u32, self.cpu.xf);
    }

    fn op_ldy(&mut self, low: u32, high: u32) {
        if self.cpu.xf {
            self.cpu.y = self.cpu_read(low) as u16;
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            self.cpu.y = self.cpu_read_word(low, high);
        }
        self.cpu_set_zn(self.cpu.y as u32, self.cpu.xf);
    }

    fn op_sta(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            self.cpu_write(low, self.cpu.a as u8);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            self.cpu_write_word(low, high, self.cpu.a, false);
        }
    }

    fn op_stx(&mut self, low: u32, high: u32) {
        if self.cpu.xf {
            self.cpu_write(low, self.cpu.x as u8);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            self.cpu_write_word(low, high, self.cpu.x, false);
        }
    }

    fn op_sty(&mut self, low: u32, high: u32) {
        if self.cpu.xf {
            self.cpu_write(low, self.cpu.y as u8);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            self.cpu_write_word(low, high, self.cpu.y, false);
        }
    }

    fn op_stz(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            self.cpu_write(low, 0);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(1);
            self.cpu_write_word(low, high, 0, false);
        }
    }

    fn op_ror(&mut self, low: u32, high: u32) {
        let result;
        let carry;
        if self.cpu.mf {
            let v = self.cpu_read(low);
            carry = v & 1 != 0;
            result = (v >> 1) | ((self.cpu.c as u8) << 7);
            self.cpu_write(low, result);
            self.cpu_set_zn(result as u32, true);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(2);
            let v = self.cpu_read_word(low, high);
            carry = v & 1 != 0;
            let r = (v >> 1) | ((self.cpu.c as u16) << 15);
            self.cpu_write_word(low, high, r, true);
            self.cpu_set_zn(r as u32, false);
        }
        self.cpu.c = carry;
    }

    fn op_rol(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let v = self.cpu_read(low) as u32;
            let r = (v << 1) | (self.cpu.c as u32);
            self.cpu.c = r & 0x100 != 0;
            self.cpu_write(low, r as u8);
            self.cpu_set_zn(r, true);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(2);
            let v = self.cpu_read_word(low, high) as u32;
            let r = (v << 1) | (self.cpu.c as u32);
            self.cpu.c = r & 0x10000 != 0;
            self.cpu_write_word(low, high, r as u16, true);
            self.cpu_set_zn(r, false);
        }
    }

    fn op_lsr(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let v = self.cpu_read(low);
            self.cpu.c = v & 1 != 0;
            let r = v >> 1;
            self.cpu_write(low, r);
            self.cpu_set_zn(r as u32, true);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(2);
            let v = self.cpu_read_word(low, high);
            self.cpu.c = v & 1 != 0;
            let r = v >> 1;
            self.cpu_write_word(low, high, r, true);
            self.cpu_set_zn(r as u32, false);
        }
    }

    fn op_asl(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let v = self.cpu_read(low) as u32;
            let r = v << 1;
            self.cpu.c = r & 0x100 != 0;
            self.cpu_write(low, r as u8);
            self.cpu_set_zn(r, true);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(2);
            let v = self.cpu_read_word(low, high) as u32;
            let r = v << 1;
            self.cpu.c = r & 0x10000 != 0;
            self.cpu_write_word(low, high, r as u16, true);
            self.cpu_set_zn(r, false);
        }
    }

    fn op_inc(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let r = self.cpu_read(low).wrapping_add(1);
            self.cpu_write(low, r);
            self.cpu_set_zn(r as u32, true);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(2);
            let r = self.cpu_read_word(low, high).wrapping_add(1);
            self.cpu_write_word(low, high, r, true);
            self.cpu_set_zn(r as u32, false);
        }
    }

    fn op_dec(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let r = self.cpu_read(low).wrapping_sub(1);
            self.cpu_write(low, r);
            self.cpu_set_zn(r as u32, true);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(2);
            let r = self.cpu_read_word(low, high).wrapping_sub(1);
            self.cpu_write_word(low, high, r, true);
            self.cpu_set_zn(r as u32, false);
        }
    }

    fn op_tsb(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let v = self.cpu_read(low);
            self.cpu.z = ((self.cpu.a as u8) & v) == 0;
            self.cpu_write(low, v | self.cpu.a as u8);
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(2);
            let v = self.cpu_read_word(low, high);
            self.cpu.z = (self.cpu.a & v) == 0;
            self.cpu_write_word(low, high, v | self.cpu.a, true);
        }
    }

    fn op_trb(&mut self, low: u32, high: u32) {
        if self.cpu.mf {
            let v = self.cpu_read(low);
            self.cpu.z = ((self.cpu.a as u8) & v) == 0;
            self.cpu_write(low, v & !(self.cpu.a as u8));
        } else {
            self.cpu.cycles_used = self.cpu.cycles_used.wrapping_add(2);
            let v = self.cpu_read_word(low, high);
            self.cpu.z = (self.cpu.a & v) == 0;
            self.cpu_write_word(low, high, v & !self.cpu.a, true);
        }
    }

    // ── CPU reset ────────────────────────────────────────────────────

    /// Re-implements the C `cpu_reset` which reads $FFFC..$FFFD through
    /// the cart to seed PC. Snes::reset already invoked `cpu.reset()`
    /// before any cart was loaded; once a ROM is in place call this to
    /// fetch the real reset vector.
    pub fn cpu_seed_reset_vector(&mut self) {
        let lo = self.read(0xfffc) as u16;
        let hi = self.read(0xfffd) as u16;
        self.cpu.pc = lo | (hi << 8);
        self.cpu.sp = 0x100;
        self.cpu.e = true;
        self.cpu.mf = true;
        self.cpu.xf = true;
        self.cpu.i = true;
    }
}

/// Free function so callers don't need to import the trait-via-impl
/// dance — mirrors the public C entry point `cpu_runOpcode`.
pub fn cpu_run_opcode(snes: &mut Snes) -> u8 {
    snes.cpu.cycles_used = 0;
    if snes.cpu.stopped {
        return 1;
    }
    if snes.cpu.waiting {
        if snes.cpu.irq_wanted || snes.cpu.nmi_wanted {
            snes.cpu.waiting = false;
        }
        return 1;
    }

    if (!snes.cpu.i && snes.cpu.irq_wanted) || snes.cpu.nmi_wanted {
        snes.cpu.cycles_used = 7;
        if snes.cpu.nmi_wanted {
            snes.cpu.nmi_wanted = false;
            snes.cpu_do_interrupt(false);
        } else {
            snes.cpu_do_interrupt(true);
        }
    } else {
        let opcode = snes.cpu_read_opcode_byte();
        snes.cpu.cycles_used = CYCLES_PER_OPCODE[opcode as usize];
        dispatch(snes, opcode);
    }
    snes.cpu.cycles_used
}

/// Execute one instruction and report its hardware master-cycle cost.
///
/// `cpu_run_opcode` retains the C port's coarse CPU-cycle return value. This
/// variant separates internal six-master-cycle work from address-dependent
/// bus accesses, matching the 65816 memory-speed map used by Snes9x.
pub fn cpu_run_opcode_timed(snes: &mut Snes) -> CpuInstructionTiming {
    snes.cpu_mem_ops = 0;
    snes.cpu_bus_master_cycles = 0;
    let cpu_cycles = cpu_run_opcode(snes);
    let bus_accesses = snes.cpu_mem_ops;
    let internal_cycles = cpu_cycles
        .checked_sub(bus_accesses)
        .expect("65816 instruction used more bus accesses than CPU cycles");
    CpuInstructionTiming {
        cpu_cycles,
        bus_accesses,
        master_cycles: u32::from(internal_cycles) * 6 + snes.cpu_bus_master_cycles,
    }
}

fn dispatch(snes: &mut Snes, mut opcode: u8) {
    loop {
        match opcode {
            // 0x00 brk — the C version asserts on unhandled BRKs and has
            // a wall of game-specific patches before its final assert(0).
            // Faithful port: apply known patches, then stop on unknown BRKs.
            0x00 => {
                let addr = ((snes.cpu.k as u32) << 16) | snes.cpu.pc as u32;
                if let Some(patched) = handle_brk_patch(snes, addr - 1) {
                    match patched {
                        BrkPatch::Restart(new_op) => {
                            opcode = new_op;
                            continue;
                        }
                        BrkPatch::Done => return,
                    }
                }
                panic!(
                    "unpatched BRK at {:06x} — see cpu.c case 0x00 for the patch table",
                    addr - 1
                );
            }
            0x01 => {
                let (l, h) = snes.adr_idx();
                snes.op_ora(l, h);
            }
            0x02 => {
                // cop imm(s)
                let _ = snes.cpu_read_opcode_byte();
                let k = snes.cpu.k;
                snes.cpu_push_byte(k);
                let pc = snes.cpu.pc;
                snes.cpu_push_word(pc);
                let flags = snes.cpu.pack_flags();
                snes.cpu_push_byte(flags);
                snes.cpu.cycles_used = snes.cpu.cycles_used.wrapping_add(1);
                snes.cpu.i = true;
                snes.cpu.d = false;
                snes.cpu.k = 0;
                snes.cpu.pc = snes.cpu_read_word(0xffe4, 0xffe5);
            }
            0x03 => {
                let (l, h) = snes.adr_sr();
                snes.op_ora(l, h);
            }
            0x04 => {
                let (l, h) = snes.adr_dp();
                snes.op_tsb(l, h);
            }
            0x05 => {
                let (l, h) = snes.adr_dp();
                snes.op_ora(l, h);
            }
            0x06 => {
                let (l, h) = snes.adr_dp();
                snes.op_asl(l, h);
            }
            0x07 => {
                let (l, h) = snes.adr_idl();
                snes.op_ora(l, h);
            }
            0x08 => {
                let flags = snes.cpu.pack_flags();
                snes.cpu_push_byte(flags);
            }
            0x09 => {
                let (l, h) = snes.adr_imm(false);
                snes.op_ora(l, h);
            }
            0x0a => {
                // asla
                if snes.cpu.mf {
                    snes.cpu.c = snes.cpu.a & 0x80 != 0;
                    snes.cpu.a = (snes.cpu.a & 0xff00) | ((snes.cpu.a << 1) & 0xff);
                } else {
                    snes.cpu.c = snes.cpu.a & 0x8000 != 0;
                    snes.cpu.a <<= 1;
                }
                snes.cpu_set_zn(snes.cpu.a as u32, snes.cpu.mf);
            }
            0x0b => {
                let dp = snes.cpu.dp;
                snes.cpu_push_word(dp);
            }
            0x0c => {
                let (l, h) = snes.adr_abs();
                snes.op_tsb(l, h);
            }
            0x0d => {
                let (l, h) = snes.adr_abs();
                snes.op_ora(l, h);
            }
            0x0e => {
                let (l, h) = snes.adr_abs();
                snes.op_asl(l, h);
            }
            0x0f => {
                let (l, h) = snes.adr_abl();
                snes.op_ora(l, h);
            }
            0x10 => {
                let v = snes.cpu_read_opcode_byte();
                let check = !snes.cpu.n;
                snes.cpu_do_branch(v, check);
            }
            0x11 => {
                let (l, h) = snes.adr_idy(false);
                snes.op_ora(l, h);
            }
            0x12 => {
                let (l, h) = snes.adr_idp();
                snes.op_ora(l, h);
            }
            0x13 => {
                let (l, h) = snes.adr_isy();
                snes.op_ora(l, h);
            }
            0x14 => {
                let (l, h) = snes.adr_dp();
                snes.op_trb(l, h);
            }
            0x15 => {
                let (l, h) = snes.adr_dpx();
                snes.op_ora(l, h);
            }
            0x16 => {
                let (l, h) = snes.adr_dpx();
                snes.op_asl(l, h);
            }
            0x17 => {
                let (l, h) = snes.adr_ily();
                snes.op_ora(l, h);
            }
            0x18 => snes.cpu.c = false,
            0x19 => {
                let (l, h) = snes.adr_aby(false);
                snes.op_ora(l, h);
            }
            0x1a => {
                if snes.cpu.mf {
                    snes.cpu.a = (snes.cpu.a & 0xff00) | (snes.cpu.a.wrapping_add(1) & 0xff);
                } else {
                    snes.cpu.a = snes.cpu.a.wrapping_add(1);
                }
                snes.cpu_set_zn(snes.cpu.a as u32, snes.cpu.mf);
            }
            0x1b => snes.cpu.sp = snes.cpu.a,
            0x1c => {
                let (l, h) = snes.adr_abs();
                snes.op_trb(l, h);
            }
            0x1d => {
                let (l, h) = snes.adr_abx(false);
                snes.op_ora(l, h);
            }
            0x1e => {
                let (l, h) = snes.adr_abx(true);
                snes.op_asl(l, h);
            }
            0x1f => {
                let (l, h) = snes.adr_alx();
                snes.op_ora(l, h);
            }
            0x20 => {
                let value = snes.cpu_read_opcode_word();
                let pc_minus = snes.cpu.pc.wrapping_sub(1);
                snes.cpu_push_word(pc_minus);
                snes.cpu.pc = value;
            }
            0x21 => {
                let (l, h) = snes.adr_idx();
                snes.op_and(l, h);
            }
            0x22 => {
                let value = snes.cpu_read_opcode_word();
                let new_k = snes.cpu_read_opcode_byte();
                let k = snes.cpu.k;
                snes.cpu_push_byte(k);
                let pc_minus = snes.cpu.pc.wrapping_sub(1);
                snes.cpu_push_word(pc_minus);
                snes.cpu.pc = value;
                snes.cpu.k = new_k;
            }
            0x23 => {
                let (l, h) = snes.adr_sr();
                snes.op_and(l, h);
            }
            0x24 => {
                let (l, h) = snes.adr_dp();
                snes.op_bit(l, h);
            }
            0x25 => {
                let (l, h) = snes.adr_dp();
                snes.op_and(l, h);
            }
            0x26 => {
                let (l, h) = snes.adr_dp();
                snes.op_rol(l, h);
            }
            0x27 => {
                let (l, h) = snes.adr_idl();
                snes.op_and(l, h);
            }
            0x28 => {
                let v = snes.cpu_pull_byte();
                snes.cpu_set_flags(v);
            }
            0x29 => {
                let (l, h) = snes.adr_imm(false);
                snes.op_and(l, h);
            }
            0x2a => {
                let result = ((snes.cpu.a as u32) << 1) | (snes.cpu.c as u32);
                if snes.cpu.mf {
                    snes.cpu.c = result & 0x100 != 0;
                    snes.cpu.a = (snes.cpu.a & 0xff00) | (result as u16 & 0xff);
                } else {
                    snes.cpu.c = result & 0x10000 != 0;
                    snes.cpu.a = result as u16;
                }
                snes.cpu_set_zn(snes.cpu.a as u32, snes.cpu.mf);
            }
            0x2b => {
                snes.cpu.dp = snes.cpu_pull_word();
                snes.cpu_set_zn(snes.cpu.dp as u32, false);
            }
            0x2c => {
                let (l, h) = snes.adr_abs();
                snes.op_bit(l, h);
            }
            0x2d => {
                let (l, h) = snes.adr_abs();
                snes.op_and(l, h);
            }
            0x2e => {
                let (l, h) = snes.adr_abs();
                snes.op_rol(l, h);
            }
            0x2f => {
                let (l, h) = snes.adr_abl();
                snes.op_and(l, h);
            }
            0x30 => {
                let v = snes.cpu_read_opcode_byte();
                let check = snes.cpu.n;
                snes.cpu_do_branch(v, check);
            }
            0x31 => {
                let (l, h) = snes.adr_idy(false);
                snes.op_and(l, h);
            }
            0x32 => {
                let (l, h) = snes.adr_idp();
                snes.op_and(l, h);
            }
            0x33 => {
                let (l, h) = snes.adr_isy();
                snes.op_and(l, h);
            }
            0x34 => {
                let (l, h) = snes.adr_dpx();
                snes.op_bit(l, h);
            }
            0x35 => {
                let (l, h) = snes.adr_dpx();
                snes.op_and(l, h);
            }
            0x36 => {
                let (l, h) = snes.adr_dpx();
                snes.op_rol(l, h);
            }
            0x37 => {
                let (l, h) = snes.adr_ily();
                snes.op_and(l, h);
            }
            0x38 => snes.cpu.c = true,
            0x39 => {
                let (l, h) = snes.adr_aby(false);
                snes.op_and(l, h);
            }
            0x3a => {
                if snes.cpu.mf {
                    snes.cpu.a = (snes.cpu.a & 0xff00) | (snes.cpu.a.wrapping_sub(1) & 0xff);
                } else {
                    snes.cpu.a = snes.cpu.a.wrapping_sub(1);
                }
                snes.cpu_set_zn(snes.cpu.a as u32, snes.cpu.mf);
            }
            0x3b => {
                snes.cpu.a = snes.cpu.sp;
                snes.cpu_set_zn(snes.cpu.a as u32, false);
            }
            0x3c => {
                let (l, h) = snes.adr_abx(false);
                snes.op_bit(l, h);
            }
            0x3d => {
                let (l, h) = snes.adr_abx(false);
                snes.op_and(l, h);
            }
            0x3e => {
                let (l, h) = snes.adr_abx(true);
                snes.op_rol(l, h);
            }
            0x3f => {
                let (l, h) = snes.adr_alx();
                snes.op_and(l, h);
            }
            0x40 => {
                let v = snes.cpu_pull_byte();
                snes.cpu_set_flags(v);
                snes.cpu.cycles_used = snes.cpu.cycles_used.wrapping_add(1);
                snes.cpu.pc = snes.cpu_pull_word();
                snes.cpu.k = snes.cpu_pull_byte();
            }
            0x41 => {
                let (l, h) = snes.adr_idx();
                snes.op_eor(l, h);
            }
            0x42 => {
                let _ = snes.cpu_read_opcode_byte();
            }
            0x43 => {
                let (l, h) = snes.adr_sr();
                snes.op_eor(l, h);
            }
            0x44 => {
                // mvp
                let dest = snes.cpu_read_opcode_byte();
                let src = snes.cpu_read_opcode_byte();
                snes.cpu.db = dest;
                let v = snes.cpu_read(((src as u32) << 16) | snes.cpu.x as u32);
                snes.cpu_write(((dest as u32) << 16) | snes.cpu.y as u32, v);
                snes.cpu.a = snes.cpu.a.wrapping_sub(1);
                snes.cpu.x = snes.cpu.x.wrapping_sub(1);
                snes.cpu.y = snes.cpu.y.wrapping_sub(1);
                if snes.cpu.a != 0xffff {
                    snes.cpu.pc = snes.cpu.pc.wrapping_sub(3);
                }
                if snes.cpu.xf {
                    snes.cpu.x &= 0xff;
                    snes.cpu.y &= 0xff;
                }
            }
            0x45 => {
                let (l, h) = snes.adr_dp();
                snes.op_eor(l, h);
            }
            0x46 => {
                let (l, h) = snes.adr_dp();
                snes.op_lsr(l, h);
            }
            0x47 => {
                let (l, h) = snes.adr_idl();
                snes.op_eor(l, h);
            }
            0x48 => {
                if snes.cpu.mf {
                    snes.cpu_push_byte(snes.cpu.a as u8);
                } else {
                    snes.cpu.cycles_used = snes.cpu.cycles_used.wrapping_add(1);
                    let a = snes.cpu.a;
                    snes.cpu_push_word(a);
                }
            }
            0x49 => {
                let (l, h) = snes.adr_imm(false);
                snes.op_eor(l, h);
            }
            0x4a => {
                snes.cpu.c = snes.cpu.a & 1 != 0;
                if snes.cpu.mf {
                    snes.cpu.a = (snes.cpu.a & 0xff00) | ((snes.cpu.a >> 1) & 0x7f);
                } else {
                    snes.cpu.a >>= 1;
                }
                snes.cpu_set_zn(snes.cpu.a as u32, snes.cpu.mf);
            }
            0x4b => {
                let k = snes.cpu.k;
                snes.cpu_push_byte(k);
            }
            0x4c => snes.cpu.pc = snes.cpu_read_opcode_word(),
            0x4d => {
                let (l, h) = snes.adr_abs();
                snes.op_eor(l, h);
            }
            0x4e => {
                let (l, h) = snes.adr_abs();
                snes.op_lsr(l, h);
            }
            0x4f => {
                let (l, h) = snes.adr_abl();
                snes.op_eor(l, h);
            }
            0x50 => {
                let v = snes.cpu_read_opcode_byte();
                let check = !snes.cpu.v;
                snes.cpu_do_branch(v, check);
            }
            0x51 => {
                let (l, h) = snes.adr_idy(false);
                snes.op_eor(l, h);
            }
            0x52 => {
                let (l, h) = snes.adr_idp();
                snes.op_eor(l, h);
            }
            0x53 => {
                let (l, h) = snes.adr_isy();
                snes.op_eor(l, h);
            }
            0x54 => {
                // mvn
                let dest = snes.cpu_read_opcode_byte();
                let src = snes.cpu_read_opcode_byte();
                snes.cpu.db = dest;
                let v = snes.cpu_read(((src as u32) << 16) | snes.cpu.x as u32);
                snes.cpu_write(((dest as u32) << 16) | snes.cpu.y as u32, v);
                snes.cpu.a = snes.cpu.a.wrapping_sub(1);
                snes.cpu.x = snes.cpu.x.wrapping_add(1);
                snes.cpu.y = snes.cpu.y.wrapping_add(1);
                if snes.cpu.a != 0xffff {
                    snes.cpu.pc = snes.cpu.pc.wrapping_sub(3);
                }
                if snes.cpu.xf {
                    snes.cpu.x &= 0xff;
                    snes.cpu.y &= 0xff;
                }
            }
            0x55 => {
                let (l, h) = snes.adr_dpx();
                snes.op_eor(l, h);
            }
            0x56 => {
                let (l, h) = snes.adr_dpx();
                snes.op_lsr(l, h);
            }
            0x57 => {
                let (l, h) = snes.adr_ily();
                snes.op_eor(l, h);
            }
            0x58 => snes.cpu.i = false,
            0x59 => {
                let (l, h) = snes.adr_aby(false);
                snes.op_eor(l, h);
            }
            0x5a => {
                if snes.cpu.xf {
                    snes.cpu_push_byte(snes.cpu.y as u8);
                } else {
                    snes.cpu.cycles_used = snes.cpu.cycles_used.wrapping_add(1);
                    let y = snes.cpu.y;
                    snes.cpu_push_word(y);
                }
            }
            0x5b => {
                snes.cpu.dp = snes.cpu.a;
                snes.cpu_set_zn(snes.cpu.dp as u32, false);
            }
            0x5c => {
                let value = snes.cpu_read_opcode_word();
                snes.cpu.k = snes.cpu_read_opcode_byte();
                snes.cpu.pc = value;
            }
            0x5d => {
                let (l, h) = snes.adr_abx(false);
                snes.op_eor(l, h);
            }
            0x5e => {
                let (l, h) = snes.adr_abx(true);
                snes.op_lsr(l, h);
            }
            0x5f => {
                let (l, h) = snes.adr_alx();
                snes.op_eor(l, h);
            }
            0x60 => {
                handle_rts_hook(snes, false);
                snes.cpu.pc = snes.cpu_pull_word().wrapping_add(1);
            }
            0x61 => {
                let (l, h) = snes.adr_idx();
                snes.op_adc(l, h);
            }
            0x62 => {
                let value = snes.cpu_read_opcode_word();
                let pc = snes.cpu.pc;
                snes.cpu_push_word(pc.wrapping_add(value as i16 as u16));
            }
            0x63 => {
                let (l, h) = snes.adr_sr();
                snes.op_adc(l, h);
            }
            0x64 => {
                let (l, h) = snes.adr_dp();
                snes.op_stz(l, h);
            }
            0x65 => {
                let (l, h) = snes.adr_dp();
                snes.op_adc(l, h);
            }
            0x66 => {
                let (l, h) = snes.adr_dp();
                snes.op_ror(l, h);
            }
            0x67 => {
                let (l, h) = snes.adr_idl();
                snes.op_adc(l, h);
            }
            0x68 => {
                if snes.cpu.mf {
                    let v = snes.cpu_pull_byte();
                    snes.cpu.a = (snes.cpu.a & 0xff00) | v as u16;
                } else {
                    snes.cpu.cycles_used = snes.cpu.cycles_used.wrapping_add(1);
                    snes.cpu.a = snes.cpu_pull_word();
                }
                snes.cpu_set_zn(snes.cpu.a as u32, snes.cpu.mf);
            }
            0x69 => {
                let (l, h) = snes.adr_imm(false);
                snes.op_adc(l, h);
            }
            0x6a => {
                let carry = snes.cpu.a & 1 != 0;
                if snes.cpu.mf {
                    snes.cpu.a = (snes.cpu.a & 0xff00)
                        | ((snes.cpu.a >> 1) & 0x7f)
                        | ((snes.cpu.c as u16) << 7);
                } else {
                    snes.cpu.a = (snes.cpu.a >> 1) | ((snes.cpu.c as u16) << 15);
                }
                snes.cpu.c = carry;
                snes.cpu_set_zn(snes.cpu.a as u32, snes.cpu.mf);
            }
            0x6b => {
                handle_rts_hook(snes, true);
                snes.cpu.pc = snes.cpu_pull_word().wrapping_add(1);
                snes.cpu.k = snes.cpu_pull_byte();
            }
            0x6c => {
                let adr = snes.cpu_read_opcode_word();
                snes.cpu.pc = snes.cpu_read_word(adr as u32, adr.wrapping_add(1) as u32);
            }
            0x6d => {
                let (l, h) = snes.adr_abs();
                snes.op_adc(l, h);
            }
            0x6e => {
                let (l, h) = snes.adr_abs();
                snes.op_ror(l, h);
            }
            0x6f => {
                let (l, h) = snes.adr_abl();
                snes.op_adc(l, h);
            }
            0x70 => {
                let v = snes.cpu_read_opcode_byte();
                let check = snes.cpu.v;
                snes.cpu_do_branch(v, check);
            }
            0x71 => {
                let (l, h) = snes.adr_idy(false);
                snes.op_adc(l, h);
            }
            0x72 => {
                let (l, h) = snes.adr_idp();
                snes.op_adc(l, h);
            }
            0x73 => {
                let (l, h) = snes.adr_isy();
                snes.op_adc(l, h);
            }
            0x74 => {
                let (l, h) = snes.adr_dpx();
                snes.op_stz(l, h);
            }
            0x75 => {
                let (l, h) = snes.adr_dpx();
                snes.op_adc(l, h);
            }
            0x76 => {
                let (l, h) = snes.adr_dpx();
                snes.op_ror(l, h);
            }
            0x77 => {
                let (l, h) = snes.adr_ily();
                snes.op_adc(l, h);
            }
            0x78 => snes.cpu.i = true,
            0x79 => {
                let (l, h) = snes.adr_aby(false);
                snes.op_adc(l, h);
            }
            0x7a => {
                if snes.cpu.xf {
                    snes.cpu.y = snes.cpu_pull_byte() as u16;
                } else {
                    snes.cpu.cycles_used = snes.cpu.cycles_used.wrapping_add(1);
                    snes.cpu.y = snes.cpu_pull_word();
                }
                snes.cpu_set_zn(snes.cpu.y as u32, snes.cpu.xf);
            }
            0x7b => {
                snes.cpu.a = snes.cpu.dp;
                snes.cpu_set_zn(snes.cpu.a as u32, false);
            }
            0x7c => snes.cpu.pc = snes.adr_iax(),
            0x7d => {
                let (l, h) = snes.adr_abx(false);
                snes.op_adc(l, h);
            }
            0x7e => {
                let (l, h) = snes.adr_abx(true);
                snes.op_ror(l, h);
            }
            0x7f => {
                let (l, h) = snes.adr_alx();
                snes.op_adc(l, h);
            }
            0x80 => {
                let v = snes.cpu_read_opcode_byte();
                snes.cpu.pc = snes.cpu.pc.wrapping_add(v as i8 as i16 as u16);
            }
            0x81 => {
                let (l, h) = snes.adr_idx();
                snes.op_sta(l, h);
            }
            0x82 => {
                let v = snes.cpu_read_opcode_word();
                snes.cpu.pc = snes.cpu.pc.wrapping_add(v as i16 as u16);
            }
            0x83 => {
                let (l, h) = snes.adr_sr();
                snes.op_sta(l, h);
            }
            0x84 => {
                let (l, h) = snes.adr_dp();
                snes.op_sty(l, h);
            }
            0x85 => {
                let (l, h) = snes.adr_dp();
                snes.op_sta(l, h);
            }
            0x86 => {
                let (l, h) = snes.adr_dp();
                snes.op_stx(l, h);
            }
            0x87 => {
                let (l, h) = snes.adr_idl();
                snes.op_sta(l, h);
            }
            0x88 => {
                if snes.cpu.xf {
                    snes.cpu.y = snes.cpu.y.wrapping_sub(1) & 0xff;
                } else {
                    snes.cpu.y = snes.cpu.y.wrapping_sub(1);
                }
                snes.cpu_set_zn(snes.cpu.y as u32, snes.cpu.xf);
            }
            0x89 => {
                if snes.cpu.mf {
                    let result = (snes.cpu.a as u8) & snes.cpu_read_opcode_byte();
                    snes.cpu.z = result == 0;
                } else {
                    snes.cpu.cycles_used = snes.cpu.cycles_used.wrapping_add(1);
                    let result = snes.cpu.a & snes.cpu_read_opcode_word();
                    snes.cpu.z = result == 0;
                }
            }
            0x8a => {
                if snes.cpu.mf {
                    snes.cpu.a = (snes.cpu.a & 0xff00) | (snes.cpu.x & 0xff);
                } else {
                    snes.cpu.a = snes.cpu.x;
                }
                snes.cpu_set_zn(snes.cpu.a as u32, snes.cpu.mf);
            }
            0x8b => {
                let db = snes.cpu.db;
                snes.cpu_push_byte(db);
            }
            0x8c => {
                let (l, h) = snes.adr_abs();
                snes.op_sty(l, h);
            }
            0x8d => {
                let (l, h) = snes.adr_abs();
                snes.op_sta(l, h);
            }
            0x8e => {
                let (l, h) = snes.adr_abs();
                snes.op_stx(l, h);
            }
            0x8f => {
                let (l, h) = snes.adr_abl();
                snes.op_sta(l, h);
            }
            0x90 => {
                let v = snes.cpu_read_opcode_byte();
                let check = !snes.cpu.c;
                snes.cpu_do_branch(v, check);
            }
            0x91 => {
                let (l, h) = snes.adr_idy(true);
                snes.op_sta(l, h);
            }
            0x92 => {
                let (l, h) = snes.adr_idp();
                snes.op_sta(l, h);
            }
            0x93 => {
                let (l, h) = snes.adr_isy();
                snes.op_sta(l, h);
            }
            0x94 => {
                let (l, h) = snes.adr_dpx();
                snes.op_sty(l, h);
            }
            0x95 => {
                let (l, h) = snes.adr_dpx();
                snes.op_sta(l, h);
            }
            0x96 => {
                let (l, h) = snes.adr_dpy();
                snes.op_stx(l, h);
            }
            0x97 => {
                let (l, h) = snes.adr_ily();
                snes.op_sta(l, h);
            }
            0x98 => {
                if snes.cpu.mf {
                    snes.cpu.a = (snes.cpu.a & 0xff00) | (snes.cpu.y & 0xff);
                } else {
                    snes.cpu.a = snes.cpu.y;
                }
                snes.cpu_set_zn(snes.cpu.a as u32, snes.cpu.mf);
            }
            0x99 => {
                let (l, h) = snes.adr_aby(true);
                snes.op_sta(l, h);
            }
            0x9a => snes.cpu.sp = snes.cpu.x,
            0x9b => {
                if snes.cpu.xf {
                    snes.cpu.y = snes.cpu.x & 0xff;
                } else {
                    snes.cpu.y = snes.cpu.x;
                }
                snes.cpu_set_zn(snes.cpu.y as u32, snes.cpu.xf);
            }
            0x9c => {
                let (l, h) = snes.adr_abs();
                snes.op_stz(l, h);
            }
            0x9d => {
                let (l, h) = snes.adr_abx(true);
                snes.op_sta(l, h);
            }
            0x9e => {
                let (l, h) = snes.adr_abx(true);
                snes.op_stz(l, h);
            }
            0x9f => {
                let (l, h) = snes.adr_alx();
                snes.op_sta(l, h);
            }
            0xa0 => {
                let (l, h) = snes.adr_imm(true);
                snes.op_ldy(l, h);
            }
            0xa1 => {
                let (l, h) = snes.adr_idx();
                snes.op_lda(l, h);
            }
            0xa2 => {
                let (l, h) = snes.adr_imm(true);
                snes.op_ldx(l, h);
            }
            0xa3 => {
                let (l, h) = snes.adr_sr();
                snes.op_lda(l, h);
            }
            0xa4 => {
                let (l, h) = snes.adr_dp();
                snes.op_ldy(l, h);
            }
            0xa5 => {
                let (l, h) = snes.adr_dp();
                snes.op_lda(l, h);
            }
            0xa6 => {
                let (l, h) = snes.adr_dp();
                snes.op_ldx(l, h);
            }
            0xa7 => {
                let (l, h) = snes.adr_idl();
                snes.op_lda(l, h);
            }
            0xa8 => {
                if snes.cpu.xf {
                    snes.cpu.y = snes.cpu.a & 0xff;
                } else {
                    snes.cpu.y = snes.cpu.a;
                }
                snes.cpu_set_zn(snes.cpu.y as u32, snes.cpu.xf);
            }
            0xa9 => {
                let (l, h) = snes.adr_imm(false);
                snes.op_lda(l, h);
            }
            0xaa => {
                if snes.cpu.xf {
                    snes.cpu.x = snes.cpu.a & 0xff;
                } else {
                    snes.cpu.x = snes.cpu.a;
                }
                snes.cpu_set_zn(snes.cpu.x as u32, snes.cpu.xf);
            }
            0xab => {
                snes.cpu.db = snes.cpu_pull_byte();
                snes.cpu_set_zn(snes.cpu.db as u32, true);
            }
            0xac => {
                let (l, h) = snes.adr_abs();
                snes.op_ldy(l, h);
            }
            0xad => {
                let (l, h) = snes.adr_abs();
                snes.op_lda(l, h);
            }
            0xae => {
                let (l, h) = snes.adr_abs();
                snes.op_ldx(l, h);
            }
            0xaf => {
                let (l, h) = snes.adr_abl();
                snes.op_lda(l, h);
            }
            0xb0 => {
                let v = snes.cpu_read_opcode_byte();
                let check = snes.cpu.c;
                snes.cpu_do_branch(v, check);
            }
            0xb1 => {
                let (l, h) = snes.adr_idy(false);
                snes.op_lda(l, h);
            }
            0xb2 => {
                let (l, h) = snes.adr_idp();
                snes.op_lda(l, h);
            }
            0xb3 => {
                let (l, h) = snes.adr_isy();
                snes.op_lda(l, h);
            }
            0xb4 => {
                let (l, h) = snes.adr_dpx();
                snes.op_ldy(l, h);
            }
            0xb5 => {
                let (l, h) = snes.adr_dpx();
                snes.op_lda(l, h);
            }
            0xb6 => {
                let (l, h) = snes.adr_dpy();
                snes.op_ldx(l, h);
            }
            0xb7 => {
                let (l, h) = snes.adr_ily();
                snes.op_lda(l, h);
            }
            0xb8 => snes.cpu.v = false,
            0xb9 => {
                let (l, h) = snes.adr_aby(false);
                snes.op_lda(l, h);
            }
            0xba => {
                if snes.cpu.xf {
                    snes.cpu.x = snes.cpu.sp & 0xff;
                } else {
                    snes.cpu.x = snes.cpu.sp;
                }
                snes.cpu_set_zn(snes.cpu.x as u32, snes.cpu.xf);
            }
            0xbb => {
                if snes.cpu.xf {
                    snes.cpu.x = snes.cpu.y & 0xff;
                } else {
                    snes.cpu.x = snes.cpu.y;
                }
                snes.cpu_set_zn(snes.cpu.x as u32, snes.cpu.xf);
            }
            0xbc => {
                let (l, h) = snes.adr_abx(false);
                snes.op_ldy(l, h);
            }
            0xbd => {
                let (l, h) = snes.adr_abx(false);
                snes.op_lda(l, h);
            }
            0xbe => {
                let (l, h) = snes.adr_aby(false);
                snes.op_ldx(l, h);
            }
            0xbf => {
                let (l, h) = snes.adr_alx();
                snes.op_lda(l, h);
            }
            0xc0 => {
                let (l, h) = snes.adr_imm(true);
                snes.op_cpy(l, h);
            }
            0xc1 => {
                let (l, h) = snes.adr_idx();
                snes.op_cmp(l, h);
            }
            0xc2 => {
                let v = snes.cpu_read_opcode_byte();
                let f = snes.cpu.pack_flags() & !v;
                snes.cpu_set_flags(f);
            }
            0xc3 => {
                let (l, h) = snes.adr_sr();
                snes.op_cmp(l, h);
            }
            0xc4 => {
                let (l, h) = snes.adr_dp();
                snes.op_cpy(l, h);
            }
            0xc5 => {
                let (l, h) = snes.adr_dp();
                snes.op_cmp(l, h);
            }
            0xc6 => {
                let (l, h) = snes.adr_dp();
                snes.op_dec(l, h);
            }
            0xc7 => {
                let (l, h) = snes.adr_idl();
                snes.op_cmp(l, h);
            }
            0xc8 => {
                if snes.cpu.xf {
                    snes.cpu.y = snes.cpu.y.wrapping_add(1) & 0xff;
                } else {
                    snes.cpu.y = snes.cpu.y.wrapping_add(1);
                }
                snes.cpu_set_zn(snes.cpu.y as u32, snes.cpu.xf);
            }
            0xc9 => {
                let (l, h) = snes.adr_imm(false);
                snes.op_cmp(l, h);
            }
            0xca => {
                if snes.cpu.xf {
                    snes.cpu.x = snes.cpu.x.wrapping_sub(1) & 0xff;
                } else {
                    snes.cpu.x = snes.cpu.x.wrapping_sub(1);
                }
                snes.cpu_set_zn(snes.cpu.x as u32, snes.cpu.xf);
            }
            0xcb => snes.cpu.waiting = true,
            0xcc => {
                let (l, h) = snes.adr_abs();
                snes.op_cpy(l, h);
            }
            0xcd => {
                let (l, h) = snes.adr_abs();
                snes.op_cmp(l, h);
            }
            0xce => {
                let (l, h) = snes.adr_abs();
                snes.op_dec(l, h);
            }
            0xcf => {
                let (l, h) = snes.adr_abl();
                snes.op_cmp(l, h);
            }
            0xd0 => {
                let v = snes.cpu_read_opcode_byte();
                let check = !snes.cpu.z;
                snes.cpu_do_branch(v, check);
            }
            0xd1 => {
                let (l, h) = snes.adr_idy(false);
                snes.op_cmp(l, h);
            }
            0xd2 => {
                let (l, h) = snes.adr_idp();
                snes.op_cmp(l, h);
            }
            0xd3 => {
                let (l, h) = snes.adr_isy();
                snes.op_cmp(l, h);
            }
            0xd4 => {
                let (l, h) = snes.adr_dp();
                let v = snes.cpu_read_word(l, h);
                snes.cpu_push_word(v);
            }
            0xd5 => {
                let (l, h) = snes.adr_dpx();
                snes.op_cmp(l, h);
            }
            0xd6 => {
                let (l, h) = snes.adr_dpx();
                snes.op_dec(l, h);
            }
            0xd7 => {
                let (l, h) = snes.adr_ily();
                snes.op_cmp(l, h);
            }
            0xd8 => snes.cpu.d = false,
            0xd9 => {
                let (l, h) = snes.adr_aby(false);
                snes.op_cmp(l, h);
            }
            0xda => {
                if snes.cpu.xf {
                    snes.cpu_push_byte(snes.cpu.x as u8);
                } else {
                    snes.cpu.cycles_used = snes.cpu.cycles_used.wrapping_add(1);
                    let x = snes.cpu.x;
                    snes.cpu_push_word(x);
                }
            }
            0xdb => snes.cpu.stopped = true,
            0xdc => {
                let adr = snes.cpu_read_opcode_word();
                snes.cpu.pc = snes.cpu_read_word(adr as u32, adr.wrapping_add(1) as u32);
                snes.cpu.k = snes.cpu_read(adr.wrapping_add(2) as u32);
            }
            0xdd => {
                let (l, h) = snes.adr_abx(false);
                snes.op_cmp(l, h);
            }
            0xde => {
                let (l, h) = snes.adr_abx(true);
                snes.op_dec(l, h);
            }
            0xdf => {
                let (l, h) = snes.adr_alx();
                snes.op_cmp(l, h);
            }
            0xe0 => {
                let (l, h) = snes.adr_imm(true);
                snes.op_cpx(l, h);
            }
            0xe1 => {
                let (l, h) = snes.adr_idx();
                snes.op_sbc(l, h);
            }
            0xe2 => {
                let v = snes.cpu_read_opcode_byte();
                let f = snes.cpu.pack_flags() | v;
                snes.cpu_set_flags(f);
            }
            0xe3 => {
                let (l, h) = snes.adr_sr();
                snes.op_sbc(l, h);
            }
            0xe4 => {
                let (l, h) = snes.adr_dp();
                snes.op_cpx(l, h);
            }
            0xe5 => {
                let (l, h) = snes.adr_dp();
                snes.op_sbc(l, h);
            }
            0xe6 => {
                let (l, h) = snes.adr_dp();
                snes.op_inc(l, h);
            }
            0xe7 => {
                let (l, h) = snes.adr_idl();
                snes.op_sbc(l, h);
            }
            0xe8 => {
                if snes.cpu.xf {
                    snes.cpu.x = snes.cpu.x.wrapping_add(1) & 0xff;
                } else {
                    snes.cpu.x = snes.cpu.x.wrapping_add(1);
                }
                snes.cpu_set_zn(snes.cpu.x as u32, snes.cpu.xf);
            }
            0xe9 => {
                let (l, h) = snes.adr_imm(false);
                snes.op_sbc(l, h);
            }
            0xea => {}
            0xeb => {
                let low = (snes.cpu.a & 0xff) as u8;
                let high = (snes.cpu.a >> 8) as u8;
                snes.cpu.a = ((low as u16) << 8) | high as u16;
                snes.cpu_set_zn(high as u32, true);
            }
            0xec => {
                let (l, h) = snes.adr_abs();
                snes.op_cpx(l, h);
            }
            0xed => {
                let (l, h) = snes.adr_abs();
                snes.op_sbc(l, h);
            }
            0xee => {
                let (l, h) = snes.adr_abs();
                snes.op_inc(l, h);
            }
            0xef => {
                let (l, h) = snes.adr_abl();
                snes.op_sbc(l, h);
            }
            0xf0 => {
                let v = snes.cpu_read_opcode_byte();
                let check = snes.cpu.z;
                snes.cpu_do_branch(v, check);
            }
            0xf1 => {
                let (l, h) = snes.adr_idy(false);
                snes.op_sbc(l, h);
            }
            0xf2 => {
                let (l, h) = snes.adr_idp();
                snes.op_sbc(l, h);
            }
            0xf3 => {
                let (l, h) = snes.adr_isy();
                snes.op_sbc(l, h);
            }
            0xf4 => {
                let v = snes.cpu_read_opcode_word();
                snes.cpu_push_word(v);
            }
            0xf5 => {
                let (l, h) = snes.adr_dpx();
                snes.op_sbc(l, h);
            }
            0xf6 => {
                let (l, h) = snes.adr_dpx();
                snes.op_inc(l, h);
            }
            0xf7 => {
                let (l, h) = snes.adr_ily();
                snes.op_sbc(l, h);
            }
            0xf8 => snes.cpu.d = true,
            0xf9 => {
                let (l, h) = snes.adr_aby(false);
                snes.op_sbc(l, h);
            }
            0xfa => {
                if snes.cpu.xf {
                    snes.cpu.x = snes.cpu_pull_byte() as u16;
                } else {
                    snes.cpu.cycles_used = snes.cpu.cycles_used.wrapping_add(1);
                    snes.cpu.x = snes.cpu_pull_word();
                }
                snes.cpu_set_zn(snes.cpu.x as u32, snes.cpu.xf);
            }
            0xfb => {
                let temp = snes.cpu.c;
                snes.cpu.c = snes.cpu.e;
                snes.cpu.e = temp;
                let f = snes.cpu.pack_flags();
                snes.cpu_set_flags(f);
            }
            0xfc => {
                let value = snes.adr_iax();
                let pc_minus = snes.cpu.pc.wrapping_sub(1);
                snes.cpu_push_word(pc_minus);
                snes.cpu.pc = value;
            }
            0xfd => {
                let (l, h) = snes.adr_abx(false);
                snes.op_sbc(l, h);
            }
            0xfe => {
                let (l, h) = snes.adr_abx(true);
                snes.op_inc(l, h);
            }
            0xff => {
                let (l, h) = snes.adr_alx();
                snes.op_sbc(l, h);
            }
        }
        return;
    }
}

// ── BRK patch table (cpu.c case 0x00) ────────────────────────────────

/// Outcome of a BRK patch lookup. The C version uses goto labels which
/// don't translate cleanly, so we model the two real outcomes:
/// - `Restart(opcode)`: re-dispatch a different opcode
/// - `Done`: the patch handled everything, return
enum BrkPatch {
    Restart(u8),
    Done,
}

fn handle_brk_patch(snes: &mut Snes, addr: u32) -> Option<BrkPatch> {
    // Mirrors the C switch in cpu.c case 0x00. Same comments retained
    // because they explain why each patch exists.
    match addr {
        0x7B269 => {
            // Link_APress_LiftCarryThrow reads OOB
            if (snes.cpu.x & 0xff) >= 28 {
                snes.cpu.pc = 0xB280; // RTS
            }
            Some(BrkPatch::Restart(0xE8))
        }
        0x5DEC7 => {
            // Uncle_AtHome case 3 reads random memory
            let v = snes.cpu_read(0x5DEB0 + (snes.cpu.y as u32 & 0xff));
            snes.cpu.a = (snes.cpu.a & 0xff00) | v as u16;
            if snes.cpu_read(0xD90 + (snes.cpu.x as u32 & 0xff)) == 2 {
                snes.cpu.pc = 0xdeea;
            } else {
                snes.cpu.pc = snes.cpu.pc.wrapping_add(2);
            }
            Some(BrkPatch::Done)
        }
        0x9be5e => {
            // Overlord_StalfosTrap doesn't init sprite_D
            snes.cpu.a = (snes.cpu.a & 0xff00) | 224;
            snes.cpu_write(0xDE0 + (snes.cpu.y as u32 & 0xff), 0);
            snes.cpu.pc = snes.cpu.pc.wrapping_add(1);
            Some(BrkPatch::Done)
        }
        0x1AF9A4 => {
            // Lanmola_SpawnShrapnel uses undefined carry value
            snes.cpu.a = (snes.cpu.a & 0xff00) | ((snes.cpu.a as u8).wrapping_add(4) as u16);
            snes.cpu.c = false;
            snes.cpu.pc = snes.cpu.pc.wrapping_add(1);
            Some(BrkPatch::Done)
        }
        0x1E8A46 => {
            // carry junk: A = A - $E2 - $08 + 12
            let m1 = snes.cpu_read(0xe2);
            let m2 = snes.cpu_read(0x08);
            snes.cpu.a = snes
                .cpu
                .a
                .wrapping_sub(m1 as u16)
                .wrapping_sub(m2 as u16)
                .wrapping_add(12);
            snes.cpu.pc = snes.cpu.pc.wrapping_add(5);
            Some(BrkPatch::Done)
        }
        0x1E8A52 => {
            // carry junk: A = A - $E8 + 8 - $09 + 8
            let m1 = snes.cpu_read(0xe8);
            let m2 = snes.cpu_read(0x09);
            snes.cpu.a = snes
                .cpu
                .a
                .wrapping_sub(m1 as u16)
                .wrapping_add(8)
                .wrapping_sub(m2 as u16)
                .wrapping_add(8);
            snes.cpu.pc = snes.cpu.pc.wrapping_add(7);
            Some(BrkPatch::Done)
        }
        0x9a966 => {
            // TAgalong_DrawInner doesn't init scratch_0 / scratch_1
            for i in 0..4u32 {
                snes.cpu_write(0x72 + i, 0);
            }
            snes.cpu.pc = snes.cpu.pc.wrapping_add(1);
            Some(BrkPatch::Done)
        }
        0x8f708 => {
            // Falls through to iny in C; we emulate by writing then INY.
            snes.cpu_write(0x75, 0);
            // INY (case_iny_c8 in C)
            if snes.cpu.xf {
                snes.cpu.y = snes.cpu.y.wrapping_add(1) & 0xff;
            } else {
                snes.cpu.y = snes.cpu.y.wrapping_add(1);
            }
            let val = snes.cpu.y as u32;
            let xf = snes.cpu.xf;
            snes.cpu_set_zn(val, xf);
            Some(BrkPatch::Done)
        }
        0x1de0e5 => {
            // GreatCatfish_ConversateThenSubmerge - not carry preserving
            if (snes.cpu.a as u8) >= 160 {
                snes.cpu.pc = 0xe164;
            } else {
                snes.cpu.pc = snes.cpu.pc.wrapping_add(1);
            }
            Some(BrkPatch::Done)
        }
        0x6d0b6 | 0x6d0c6 => {
            // Sprite_CommonItemPickup - wrong carry chain
            snes.cpu.c = (snes.cpu.a as u8) >= 4;
            snes.cpu.a = snes.cpu.a.wrapping_sub(4);
            snes.cpu.pc = snes.cpu.pc.wrapping_add(1);
            Some(BrkPatch::Done)
        }
        0x1d8f29 | 0x1dc812 | 0x1DDBD3 | 0x1DF856 | 0x6ED0B | 0x9b478 | 0x9b46c => {
            snes.cpu.c = false;
            Some(BrkPatch::Restart(0x69)) // adc imm
        }
        0x1E88DA => {
            snes.cpu.c = false;
            Some(BrkPatch::Restart(0x65)) // adc dp
        }
        0x9B468 | 0x9B46A | 0x9B474 | 0x9B476 => {
            snes.cpu.c = true;
            Some(BrkPatch::Restart(0xe5)) // sbc dp
        }
        0x9B60C => {
            snes.cpu.c = true;
            Some(BrkPatch::Restart(0xe9)) // sbc imm
        }
        0x1DCDEB => {
            // Y = sprite_head_dir[X]; A = X
            snes.cpu.y = snes.cpu_read(0x0eb0 + (snes.cpu.x as u32 & 0xff)) as u16;
            snes.cpu.a = snes.cpu.x;
            Some(BrkPatch::Done)
        }
        _ => None,
    }
}

fn handle_rts_hook(snes: &mut Snes, _long: bool) {
    if snes.cpu.sp_breakpoint != 0 && snes.cpu.sp >= snes.cpu.sp_breakpoint {
        assert_eq!(snes.cpu.sp, snes.cpu.sp_breakpoint);
        snes.cpu.sp_breakpoint = 0;
        // The C hook clears g_calling_asm_from_c; Rust callers poll the
        // cleared breakpoint because the SNES crate cannot call into Zelda.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Load a tiny program into WRAM and set PC there. Bank 0 / addr
    /// $0000..$1fff is mirrored to WRAM, so the CPU fetches will hit
    /// it via the bus.
    fn load_code(snes: &mut Snes, addr: u16, bytes: &[u8]) {
        for (i, &b) in bytes.iter().enumerate() {
            snes.ram[addr as usize + i] = b;
        }
        snes.cpu.pc = addr;
        snes.cpu.k = 0;
        // Native mode, 16-bit A/X/Y so tests don't have to flip flags.
        snes.cpu.e = false;
        snes.cpu.mf = false;
        snes.cpu.xf = false;
    }

    #[test]
    fn lda_imm_loads_a() {
        let mut snes = Snes::new();
        load_code(&mut snes, 0x1000, &[0xa9, 0x34, 0x12]); // LDA #$1234
        cpu_run_opcode(&mut snes);
        assert_eq!(snes.cpu.a, 0x1234);
        assert!(!snes.cpu.z);
        assert!(!snes.cpu.n);
    }

    #[test]
    fn lda_imm_sets_zn() {
        let mut snes = Snes::new();
        load_code(&mut snes, 0x1000, &[0xa9, 0x00, 0x00]); // LDA #$0000
        cpu_run_opcode(&mut snes);
        assert!(snes.cpu.z);

        load_code(&mut snes, 0x1000, &[0xa9, 0x00, 0x80]); // LDA #$8000
        cpu_run_opcode(&mut snes);
        assert!(snes.cpu.n);
    }

    #[test]
    fn adc_imm_no_decimal() {
        let mut snes = Snes::new();
        snes.cpu.a = 0x0010;
        snes.cpu.c = false;
        load_code(&mut snes, 0x1000, &[0x69, 0x20, 0x00]); // ADC #$0020
        cpu_run_opcode(&mut snes);
        assert_eq!(snes.cpu.a, 0x0030);
        assert!(!snes.cpu.c);
    }

    #[test]
    fn adc_imm_carry_out() {
        let mut snes = Snes::new();
        snes.cpu.a = 0xffff;
        snes.cpu.c = false;
        load_code(&mut snes, 0x1000, &[0x69, 0x01, 0x00]);
        cpu_run_opcode(&mut snes);
        assert_eq!(snes.cpu.a, 0x0000);
        assert!(snes.cpu.c);
        assert!(snes.cpu.z);
    }

    #[test]
    fn sbc_imm_no_decimal() {
        let mut snes = Snes::new();
        snes.cpu.a = 0x0030;
        snes.cpu.c = true;
        load_code(&mut snes, 0x1000, &[0xe9, 0x10, 0x00]); // SBC #$0010
        cpu_run_opcode(&mut snes);
        assert_eq!(snes.cpu.a, 0x0020);
        assert!(snes.cpu.c);
    }

    #[test]
    fn inx_wraps_in_8bit_mode() {
        let mut snes = Snes::new();
        snes.cpu.xf = true;
        snes.cpu.x = 0x00ff;
        load_code(&mut snes, 0x1000, &[0xe8]); // INX
                                               // load_code sets xf=false; restore
        snes.cpu.xf = true;
        cpu_run_opcode(&mut snes);
        assert_eq!(snes.cpu.x, 0);
        assert!(snes.cpu.z);
    }

    #[test]
    fn jmp_absolute() {
        let mut snes = Snes::new();
        load_code(&mut snes, 0x1000, &[0x4c, 0x34, 0x12]); // JMP $1234
        cpu_run_opcode(&mut snes);
        assert_eq!(snes.cpu.pc, 0x1234);
    }

    #[test]
    fn jsr_pushes_return_address() {
        let mut snes = Snes::new();
        load_code(&mut snes, 0x1000, &[0x20, 0x00, 0x20]); // JSR $2000
        snes.cpu.sp = 0x01ff;
        cpu_run_opcode(&mut snes);
        assert_eq!(snes.cpu.pc, 0x2000);
        // pushed return = 0x1002 (pc-1 after fetching 3 bytes = 0x1003-1)
        // sp is now 0x01fd
        assert_eq!(snes.cpu.sp, 0x01fd);
        // word at 01fe (lo) / 01ff (hi) == 0x1002
        assert_eq!(snes.ram[0x1fe], 0x02);
        assert_eq!(snes.ram[0x1ff], 0x10);
    }

    #[test]
    fn rts_clears_sp_breakpoint_before_return() {
        let mut snes = Snes::new();
        load_code(&mut snes, 0x1000, &[0x60]); // RTS
        snes.cpu.sp = 0x01fd;
        snes.cpu.sp_breakpoint = 0x01fd;
        snes.ram[0x01fe] = 0x34;
        snes.ram[0x01ff] = 0x12;

        cpu_run_opcode(&mut snes);

        assert_eq!(snes.cpu.sp_breakpoint, 0);
        assert_eq!(snes.cpu.pc, 0x1235);
    }

    #[test]
    #[should_panic]
    fn rts_breakpoint_overshoot_panics_like_c_assert() {
        let mut snes = Snes::new();
        load_code(&mut snes, 0x1000, &[0x60]); // RTS
        snes.cpu.sp = 0x01fe;
        snes.cpu.sp_breakpoint = 0x01fd;

        cpu_run_opcode(&mut snes);
    }

    #[test]
    fn xba_swaps_a_halves() {
        let mut snes = Snes::new();
        snes.cpu.a = 0x1234;
        load_code(&mut snes, 0x1000, &[0xeb]); // XBA
        cpu_run_opcode(&mut snes);
        assert_eq!(snes.cpu.a, 0x3412);
    }

    #[test]
    fn nop_increments_pc() {
        let mut snes = Snes::new();
        load_code(&mut snes, 0x1000, &[0xea, 0xea]);
        cpu_run_opcode(&mut snes);
        assert_eq!(snes.cpu.pc, 0x1001);
        cpu_run_opcode(&mut snes);
        assert_eq!(snes.cpu.pc, 0x1002);
    }

    #[test]
    fn rep_sep_set_flags() {
        let mut snes = Snes::new();
        snes.cpu.mf = false;
        snes.cpu.xf = false;
        snes.cpu.e = false;
        // SEP #$30 sets m and x flags
        load_code(&mut snes, 0x1000, &[0xe2, 0x30]);
        snes.cpu.mf = false;
        snes.cpu.xf = false;
        snes.cpu.e = false;
        cpu_run_opcode(&mut snes);
        assert!(snes.cpu.mf);
        assert!(snes.cpu.xf);

        // REP #$30 clears them
        load_code(&mut snes, 0x1010, &[0xc2, 0x30]);
        snes.cpu.mf = true;
        snes.cpu.xf = true;
        snes.cpu.e = false;
        cpu_run_opcode(&mut snes);
        assert!(!snes.cpu.mf);
        assert!(!snes.cpu.xf);
    }

    #[test]
    fn indexed_read_page_crossing_costs_one_cycle_but_indexed_write_does_not() {
        let mut snes = Snes::new();

        load_code(&mut snes, 0x1000, &[0xbd, 0xff, 0x10]); // LDA $10ff,X
        snes.cpu.mf = true;
        snes.cpu.xf = true;
        snes.cpu.x = 1;
        assert_eq!(cpu_run_opcode(&mut snes), 5);

        load_code(&mut snes, 0x1010, &[0xb9, 0xff, 0x10]); // LDA $10ff,Y
        snes.cpu.mf = true;
        snes.cpu.xf = true;
        snes.cpu.y = 1;
        assert_eq!(cpu_run_opcode(&mut snes), 5);

        load_code(&mut snes, 0x1020, &[0x9d, 0xff, 0x10]); // STA $10ff,X
        snes.cpu.mf = true;
        snes.cpu.xf = true;
        snes.cpu.x = 1;
        assert_eq!(cpu_run_opcode(&mut snes), 5);

        load_code(&mut snes, 0x1030, &[0xb1, 0x20]); // LDA ($20),Y
        snes.cpu.mf = true;
        snes.cpu.xf = true;
        snes.cpu.y = 1;
        snes.ram[0x20] = 0xff;
        snes.ram[0x21] = 0x10;
        assert_eq!(cpu_run_opcode(&mut snes), 6);

        load_code(&mut snes, 0x1040, &[0x91, 0x20]); // STA ($20),Y
        snes.cpu.mf = true;
        snes.cpu.xf = true;
        snes.cpu.y = 1;
        assert_eq!(cpu_run_opcode(&mut snes), 6);
    }

    #[test]
    fn timed_opcode_separates_internal_cycles_from_slow_bus_accesses() {
        let mut snes = Snes::new();

        load_code(&mut snes, 0x1000, &[0xea]); // NOP
        let nop = cpu_run_opcode_timed(&mut snes);
        assert_eq!(nop.cpu_cycles, 2);
        assert_eq!(nop.bus_accesses, 1);
        assert_eq!(nop.master_cycles, 14);

        load_code(&mut snes, 0x1010, &[0xbd, 0xff, 0x10]); // LDA $10ff,X
        snes.cpu.mf = true;
        snes.cpu.xf = true;
        snes.cpu.x = 1;
        let page_crossing_load = cpu_run_opcode_timed(&mut snes);
        assert_eq!(page_crossing_load.cpu_cycles, 5);
        assert_eq!(page_crossing_load.bus_accesses, 4);
        assert_eq!(page_crossing_load.master_cycles, 38);
    }
}
