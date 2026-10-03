//! Active name entry retains cursor motion, character stripes and save checksum
//! work on the interrupted CPU, including the real nested return stack.
#[cfg(test)]
use super::intro_clear::Access;
use super::native_work::WorkBus;
use super::{
    native_work::Work,
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NameFileActive {
    pc: u16,
    complete: bool,
}
impl Default for NameFileActive {
    fn default() -> Self {
        Self {
            pc: 0xda4d,
            complete: false,
        }
    }
}
impl NameFileActive {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x0c0000
            | u32::from(match self.pc {
                0xdc8c..=0xdcbe => 0xdc8cu16,
                0xdcbf..=0xdd23 => 0xdcbf,
                0xdd30..=0xdd6c => 0xdd30,
                0xd22d..=0xd239 => 0xd22d,
                _ => 0xda4d,
            })
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (12, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let index_mask = if x == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match self.pc {
            // Accumulator/index width changes at the source call boundaries.
            0xda62 | 0xda6e | 0xda83 | 0xda9b | 0xdb3e | 0xdb54 | 0xdb8c | 0xdbb1 | 0xdbd2
            | 0xdbd9 | 0xdbdc | 0xdbe3 | 0xdc79 | 0xdc89 | 0xdc8c | 0xdc9b | 0xdcb7 | 0xdcbc
            | 0xdd60 => {
                let mask = if matches!(self.pc, 0xda62 | 0xda6e | 0xda83 | 0xda9b | 0xdb54 | 0xdbd2)
                {
                    0x20
                } else {
                    0x30
                };
                w.width(
                    mask,
                    matches!(
                        self.pc,
                        0xda62 | 0xda83 | 0xdb3e | 0xdb8c | 0xdbb1 | 0xdbe3 | 0xdc9b
                    ),
                );
                self.pc + 2
            }
            // Live absolute accumulator reads (DB follows the real PHB/PLB).
            0xda5d | 0xda65 | 0xda8c | 0xdaa2 | 0xdaac | 0xdadf | 0xdafc | 0xdb19 | 0xdb40
            | 0xdb6c | 0xdb80 | 0xdb93 | 0xdbb5 | 0xdca2 | 0xdce2 | 0xdd1d => {
                let a = match self.pc {
                    0xda5d | 0xdca2 => 0xb10,
                    0xda65 | 0xda8c => 0x630,
                    0xdaa2 => 0xb14,
                    0xdaac | 0xdadf => 0xb11,
                    0xdafc | 0xdb6c | 0xdb80 | 0xdb93 => 0xb12,
                    0xdb19 => 0xb13,
                    0xdb40 | 0xdce2 => 0xb15,
                    0xdbb5 => 0x200,
                    _ => 2,
                };
                w.drain(16);
                let v = w.read(w.absolute(a), if self.pc == 0xdafc { x } else { m });
                if self.pc == 0xdafc {
                    w.cpu.x = v;
                    w.flags(v, x);
                } else {
                    w.load(v, m);
                }
                self.pc + 3
            }
            0xda4d | 0xda85 | 0xdaa7 | 0xdcd6 => {
                let a = match self.pc {
                    0xda4d => 0xb13,
                    0xda85 => 0xb16,
                    0xdaa7 => 0xb15,
                    _ => 0xb10,
                };
                w.drain(16);
                let v = w.read(w.absolute(a), x);
                if self.pc == 0xda4d {
                    w.cpu.y = v;
                } else {
                    w.cpu.x = v;
                }
                w.flags(v, x);
                self.pc + 3
            }
            0xda75 | 0xdb21 | 0xdb2a | 0xdb30 | 0xdba0 | 0xdbc5 | 0xdbe5 | 0xdc8e | 0xdcbf
            | 0xdccf | 0xdcdd => {
                let a = match self.pc {
                    0xda75 | 0xdc8e | 0xdcbf => 0xf0,
                    0xdb21 | 0xdb2a => 0xf4,
                    0xdb30 => 0xf6,
                    0xdba0 => 0,
                    0xdbe5 => 0xc8,
                    _ => 2,
                };
                let v = w.dp_read(a, m);
                w.load(v, m);
                self.pc + 2
            }
            0xdc58 | 0xdc6d => {
                let v = w.dp_read(0, x);
                w.cpu.x = v;
                w.flags(v, x);
                self.pc + 2
            }
            // Immediate loads, logical operations and comparisons.
            0xda70 | 0xdad1 | 0xdae5 | 0xdaea | 0xdb06 | 0xdb0c | 0xdb11 | 0xdb39 | 0xdb71
            | 0xdbd4 | 0xdbdf | 0xdbf6 | 0xdbfd | 0xdc08 | 0xdc0f | 0xdc31 | 0xdc38 | 0xdc3f
            | 0xdc6f | 0xdc7f | 0xdc84 | 0xdd33 | 0xdd46 | 0xdd62 | 0xdd67 | 0xd22d | 0xd231 => {
                let v = match self.pc {
                    0xda70 => 0x30,
                    0xdad1 => 0x18,
                    0xdae5 => 0x2e,
                    0xdaea | 0xdbd4 => 0x3c,
                    0xdb06 => 0x58,
                    0xdb0c => 0x29,
                    0xdb11 => 0x0c,
                    0xdb39 => 0x2b,
                    0xdb71 => 5,
                    0xdbdf => 4,
                    0xdbf6 => 0x55aa,
                    0xdbfd => 0xf000,
                    0xdc08 => 0xffff,
                    0xdc0f => 0x1d,
                    0xdc31 => 0xf0,
                    0xdc38 => 0x1502,
                    0xdc3f | 0xdd46 => 0x100,
                    0xdc6f => 0x5a5a,
                    0xdc7f | 0xdd62 => 255,
                    0xdc84 => 0x2c,
                    0xdd33 => 0x6100,
                    _ => 1,
                };
                w.immediate(v, m);
                self.pc + 1 + u16::from(m)
            }
            0xda77 | 0xda95 | 0xdb23 | 0xdb2c | 0xdb32 | 0xdb43 | 0xdb50 | 0xdb8e | 0xdb96
            | 0xdba2 | 0xdc20 | 0xdc90 | 0xdc9d | 0xdca5 | 0xdcc1 => {
                let v = match self.pc {
                    0xda77 | 0xdc90 => 3,
                    0xda95 => 0x1ff,
                    0xdb23 => 0x10,
                    0xdb2c | 0xdb32 => 0xc0,
                    0xdb8e => 15,
                    0xdba2 => 0xfff0,
                    0xdcc1 => 12,
                    _ => 255,
                };
                w.mask(v, m);
                self.pc + 1 + u16::from(m)
            }
            0xda53 | 0xdb5a | 0xdb5e | 0xdb62 | 0xdb83 | 0xdbc0 | 0xdbc7 | 0xdc23 | 0xdc2c
            | 0xdccb | 0xdcd9 => {
                let v = match self.pc {
                    0xda53 => 0x31,
                    0xdb5a => 0x5a,
                    0xdb5e => 0x44,
                    0xdb62 => 0x6f,
                    0xdb83 => 6,
                    0xdbc0 => 0xa9,
                    0xdbc7 => 10,
                    0xdc23 => 0x60,
                    0xdc2c => 1,
                    0xdccb => 0x10,
                    _ => 0x13,
                };
                w.compare_immediate(v, m);
                self.pc + 1 + u16::from(m)
            }
            0xdaf7 | 0xdc17 => {
                w.compare_x_immediate(if self.pc == 0xdaf7 { 26 } else { 0 }, x);
                self.pc + 1 + u16::from(x)
            }
            0xdc66 => {
                w.drain(16);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= 0x27f);
                w.flags(w.cpu.y.wrapping_sub(0x27f), 2);
                self.pc + 3
            }
            0xdaaa | 0xdabf | 0xdc14 | 0xdc46 | 0xdc5a => {
                let v = match self.pc {
                    0xdaaa => 2,
                    0xdabf => 254,
                    0xdc14 => 60,
                    _ => 0,
                };
                w.drain(8 * u32::from(x));
                w.cpu.y = v;
                w.flags(v, x);
                self.pc + 1 + u16::from(x)
            }
            0xdace => {
                w.drain(8);
                w.cpu.x = 0;
                w.flags(0, 1);
                self.pc + 2
            }
            // Source ALU work, preserving the high accumulator byte in M8.
            0xda52 | 0xdac1 | 0xdc5d => {
                w.drain(6);
                w.load(w.cpu.y, m);
                self.pc + 1
            }
            0xda60 | 0xdb46 | 0xdb99 | 0xdba5 | 0xdbe7 | 0xdca0 | 0xdcc7 | 0xdcd1 | 0xdcd2 => {
                w.shift_left(m);
                self.pc + 1
            }
            0xda61 | 0xdb47 | 0xdb53 | 0xdb9f | 0xdbbb | 0xdbee | 0xdbf5 | 0xdca1 | 0xdce1 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & index_mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xda8f => {
                w.drain(6);
                w.cpu.x = w.cpu.y;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xdad0 => {
                w.drain(6);
                w.cpu.y = w.cpu.x;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xdb9a => {
                w.drain(6);
                w.cpu.y = w.cpu.a & index_mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xda57 | 0xda90 | 0xdac2 | 0xdada | 0xdb4c | 0xdb9b | 0xdbb8 | 0xdc5e | 0xdca8
            | 0xdce5 | 0xdd3e | 0xdd59 => {
                w.clear_carry();
                self.pc + 1
            }
            0xdc72 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0xda58 | 0xdadb | 0xdd3f | 0xdd5a => {
                w.drain(8 * u32::from(m));
                let v = match self.pc {
                    0xda58 => 4,
                    0xdadb => 8,
                    0xdd3f => 32,
                    _ => 16,
                };
                if m == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                self.pc + 1 + u16::from(m)
            }
            0xdac3 | 0xdb4d | 0xdb9c => {
                w.drain(16);
                let v = w.read(
                    w.absolute(match self.pc {
                        0xdac3 => 0xb11,
                        0xdb4d => 0xb10,
                        _ => 0x200,
                    }),
                    m,
                );
                if m == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                self.pc + 3
            }
            0xdbb9 | 0xdc73 => {
                let v = w.dp_read(2, m);
                if self.pc == 0xdc73 {
                    w.sbc(v, m);
                } else {
                    w.adc(v);
                }
                self.pc + 2
            }
            0xdb1c | 0xdcc8 | 0xdcd3 => {
                w.drain(16);
                let v = w.read(w.absolute(if self.pc == 0xdb1c { 0xb14 } else { 0xb15 }), m);
                w.load((w.cpu.a | v) & mask, m);
                self.pc + 3
            }
            0xdba6 => {
                let v = w.dp_read(2, m);
                w.load(w.cpu.a | v, m);
                self.pc + 2
            }
            0xdd53 => {
                w.drain(16);
                w.load(w.cpu.a | 0x1800, 2);
                self.pc + 3
            }
            0xdbcc | 0xdbcd | 0xdbe8 | 0xdbe9 => {
                w.increment_a(m);
                self.pc + 1
            }
            0xdc97 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1) & mask, m);
                self.pc + 1
            }
            0xdcdf | 0xdce0 => {
                w.drain(6);
                let v = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | (v as u8 & 1);
                w.load(v >> 1, m);
                self.pc + 1
            }
            0xda64 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & index_mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xda8a | 0xda8b | 0xdaf2 | 0xdaf3 | 0xdaf4 | 0xdaf5 | 0xdc52 | 0xdc53 | 0xdc65 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & index_mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xdaf6 | 0xdc50 | 0xdc51 | 0xdc63 | 0xdc64 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & index_mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xdd39 | 0xdd3d | 0xdd42 => {
                w.swap_bytes();
                self.pc + 1
            }
            // Long table/SRAM loads, including comparisons at the same bus time.
            0xda68 | 0xda91 | 0xdaaf | 0xdaff | 0xdb48 | 0xdb56 | 0xdbbc | 0xdbef | 0xdc1c
            | 0xdc28 | 0xdc5f | 0xdca9 | 0xdcad | 0xdcb3 | 0xdce6 | 0xdcea | 0xdcf0 | 0xdd4f => {
                let a = match self.pc {
                    0xda68 => 0x0cd9b5,
                    0xda91 => 0x0cda19,
                    0xdaaf => 0x0cda01,
                    0xdaff => 0x0cda13,
                    0xdb48 => 0x0cda0b,
                    0xdb56 => 0x0cd935,
                    0xdbbc | 0xdc28 | 0xdd4f => 0x7003d9,
                    0xdbef => 0x00848a,
                    0xdc1c => 0x0083f8,
                    0xdc5f => 0x700000,
                    0xdca9 => 0x0cd9f5,
                    0xdcad => 0x0cd9f9,
                    0xdcb3 => 0x0cd9fd,
                    0xdce6 => 0x0cda04,
                    0xdcea => 0x0cda06,
                    _ => 0x0cda08,
                };
                w.drain(24);
                let v = w.read(
                    a + if matches!(self.pc, 0xdc1c | 0xdc28) {
                        0
                    } else {
                        u32::from(w.cpu.x)
                    },
                    m,
                );
                if matches!(self.pc, 0xda68 | 0xdaaf | 0xdcad | 0xdcea) {
                    let a = w.cpu.a & mask;
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                    w.flags(a.wrapping_sub(v) & mask, m);
                } else if matches!(self.pc, 0xda91 | 0xdc5f | 0xdca9 | 0xdce6) {
                    if m == 1 {
                        w.adc_byte(v);
                    } else {
                        w.adc(v);
                    }
                } else {
                    w.load(v, m);
                }
                self.pc + 4
            }
            0xdc49 | 0xdd36 => {
                w.drain(16);
                if x == 2
                    || ((if self.pc == 0xdc49 { 0xf48a } else { 0xdd24 }) & 255) + w.cpu.y > 255
                {
                    w.drain(6);
                }
                let v = w.read(
                    w.absolute(if self.pc == 0xdc49 { 0xf48a } else { 0xdd24 })
                        + u32::from(w.cpu.y),
                    m,
                );
                w.load(if self.pc == 0xdd36 { w.cpu.a | v } else { v }, m);
                self.pc + 3
            }
            // Every store is retired individually by the shared native bus.
            0xda5a | 0xda72 | 0xda98 | 0xdac6 | 0xdb3b | 0xdb73 | 0xdbd6 | 0xdc81 | 0xdc86
            | 0xdc98 | 0xdcb9 | 0xdcf4 | 0xdd20 | 0xdd3a | 0xdd43 | 0xdd49 | 0xdd4c | 0xdd56
            | 0xdd5d | 0xdd64 => {
                let a = match self.pc {
                    0xda5a | 0xda72 => 0xb13,
                    0xda98 => 0x630,
                    0xdac6 => 0xb11,
                    0xdb3b | 0xdbd6 | 0xdc86 => 0x12e,
                    0xdb73 => 0xb12,
                    0xdc81 => 0x128,
                    0xdc98 => 0xb16,
                    0xdcb9 => 0xb10,
                    0xdcf4 => 0xb15,
                    0xdd20 => 0xcb,
                    0xdd3a => 0x1002,
                    0xdd43 => 0x1008,
                    0xdd49 => 0x1004,
                    0xdd4c => 0x100a,
                    0xdd56 => 0x1006,
                    0xdd5d => 0x100c,
                    _ => 0x100e,
                };
                w.drain(16);
                w.write(w.absolute(a), w.cpu.a & mask, m);
                self.pc + 3
            }
            0xda7b | 0xdab5 | 0xdb87 | 0xdd1a => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0xda7b => 0xb13,
                        0xdab5 => 0xb14,
                        0xdb87 => 0xb12,
                        _ => 0xca,
                    }),
                    0,
                    m,
                );
                self.pc + 3
            }
            0xdb78 | 0xdb7d | 0xdc94 | 0xdd15 => {
                let a = w.absolute(match self.pc {
                    0xdc94 => 0xb13,
                    0xdd15 => 0xb14,
                    _ => 0xb12,
                });
                w.drain(16);
                let v = w.read(a, m);
                let v = if self.pc == 0xdb78 {
                    v.wrapping_sub(1)
                } else {
                    v.wrapping_add(1)
                } & mask;
                w.drain(6);
                w.flags(v, m);
                w.write(a, v, m);
                self.pc + 3
            }
            0xdad3 | 0xdadd | 0xdb66 | 0xdb68 | 0xdb91 | 0xdbb3 | 0xdbce | 0xdbf3 | 0xdc12
            | 0xdc6b | 0xdcc5 | 0xdd69 | 0xd22f | 0xd233 | 0xd235 | 0xd237 => {
                let a = match self.pc {
                    0xdad3 | 0xdadd | 0xdb66 | 0xdbf3 => 0,
                    0xdb68 => 1,
                    0xdd69 => 0x14,
                    0xd22f => 0x10,
                    0xd233 => 0x11,
                    0xd235 => 0xb0,
                    0xd237 => 0xc8,
                    _ => 2,
                };
                w.dp_store(
                    a,
                    if matches!(self.pc, 0xdb68 | 0xdbb3 | 0xd235 | 0xd237) {
                        0
                    } else {
                        w.cpu.a & mask
                    },
                    m,
                );
                self.pc + 2
            }
            0xdad5 => {
                let v = w.dp_read(0, m);
                w.load(v, m);
                self.pc + 2
            }
            0xdc54 => {
                w.dp_increment(2, m, false);
                self.pc + 2
            }
            0xdad7 | 0xdae2 | 0xdae7 | 0xdaec | 0xdb03 | 0xdb08 | 0xdb0e | 0xdb13 => {
                let a = match self.pc {
                    0xdad7 | 0xdb03 => 0x800,
                    0xdae2 | 0xdb08 => 0x801,
                    0xdae7 | 0xdb0e => 0x802,
                    _ => 0x803,
                };
                w.drain(22);
                w.write(w.absolute(a) + u32::from(w.cpu.y), w.cpu.a & mask, m);
                self.pc + 3
            }
            0xdaef | 0xdb16 => {
                w.drain(22);
                w.write(w.absolute(0xa20) + u32::from(w.cpu.x), 0, m);
                self.pc + 3
            }
            0xdba8 | 0xdbea | 0xdbf9 | 0xdc00 | 0xdc04 | 0xdc0b | 0xdc34 | 0xdc3b | 0xdc42
            | 0xdc4c | 0xdc75 => {
                let a = match self.pc {
                    0xdba8 => 0x7003d9,
                    0xdbea => 0x701ffe,
                    0xdbf9 => 0x7003e5,
                    0xdc00 => 0x70020c,
                    0xdc04 => 0x70020e,
                    0xdc0b => 0x700405,
                    0xdc34 => 0x700212,
                    0xdc3b => 0x7003c5,
                    0xdc42 => 0x7003c7,
                    0xdc4c => 0x700340,
                    _ => 0x7004fe,
                };
                w.drain(24);
                w.write(
                    a + if self.pc == 0xdbea {
                        0
                    } else {
                        u32::from(w.cpu.x)
                    },
                    w.cpu.a & mask,
                    m,
                );
                self.pc + 4
            }
            // Nested calls preserve the actual source stack and width changes.
            0xda7e | 0xda9f | 0xdab8 | 0xdacb | 0xdbac | 0xdc7c => {
                w.drain(22);
                w.push(self.pc + 2, 2);
                match self.pc {
                    0xda7e | 0xda9f => 0xdc8c,
                    0xdab8 | 0xdacb => 0xdcbf,
                    0xdbac => 0xdd30,
                    _ => 0xd22d,
                }
            }
            0xdcbe | 0xdd23 | 0xdd6c | 0xd239 => {
                w.rts();
                w.cpu.pc
            }
            0xdbdb | 0xdc8b => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            0xdafb | 0xdbde | 0xdbe1 | 0xdd30 | 0xdd31 => {
                w.drain(6);
                let v = match self.pc {
                    0xdafb => w.cpu.x,
                    0xdbe1 => w.cpu.a & mask,
                    0xdd31 => u16::from(w.cpu.bank),
                    _ => u16::from(w.cpu.db),
                };
                w.push(
                    v,
                    if self.pc == 0xdafb {
                        x
                    } else if self.pc == 0xdbe1 {
                        m
                    } else {
                        1
                    },
                );
                self.pc + 1
            }
            0xdb0b | 0xdbe2 | 0xdc7b | 0xdd32 | 0xdd6b => {
                w.drain(12);
                let v = w.pull(if self.pc == 0xdb0b { x } else { 1 });
                if self.pc == 0xdb0b {
                    w.cpu.x = v;
                    w.flags(v, x);
                } else {
                    w.cpu.db = v as u8;
                    w.flags(v, 1);
                }
                self.pc + 1
            }
            0xdb27 | 0xdb36 => {
                w.drain(16);
                if self.pc == 0xdb27 {
                    0xdbb1
                } else {
                    0xdbd9
                }
            }
            0xda50 | 0xda55 | 0xda6c | 0xda79 | 0xda81 | 0xda88 | 0xda9d | 0xdaa5 | 0xdab3
            | 0xdabb | 0xdabd | 0xdac9 | 0xdaf9 | 0xdb1f | 0xdb25 | 0xdb2e | 0xdb34 | 0xdb5c
            | 0xdb60 | 0xdb64 | 0xdb6a | 0xdb6f | 0xdb76 | 0xdb7b | 0xdb85 | 0xdb8a | 0xdbaf
            | 0xdbc3 | 0xdbca | 0xdbd0 | 0xdc1a | 0xdc26 | 0xdc2f | 0xdc56 | 0xdc69 | 0xdc92
            | 0xdcb1 | 0xdcc3 | 0xdccd | 0xdcdb | 0xdcee | 0xdcf7 | 0xdd18 => {
                let z = w.cpu.p & ZERO != 0;
                let n = w.cpu.p & NEGATIVE != 0;
                let (t, a) = match self.pc {
                    0xda50 => (z, 0xda9f),
                    0xda55 => (z, 0xda5d),
                    0xda6c => (!z, 0xda83),
                    0xda79 => (!z, 0xda7e),
                    0xda81 => (true, 0xda4d),
                    0xda88 => (!z, 0xda8c),
                    0xda9d => (true, 0xdaa2),
                    0xdaa5 => (z, 0xdacb),
                    0xdab3 => (!z, 0xdabd),
                    0xdabb => (true, 0xdaa2),
                    0xdabd => (n, 0xdac1),
                    0xdac9 => (true, 0xdace),
                    0xdaf9 => (!z, 0xdad5),
                    0xdb1f => (!z, 0xdb36),
                    0xdb25 => (z, 0xdb2a),
                    0xdb2e | 0xdb34 => (!z, 0xdb39),
                    0xdb5c => (z, 0xdb6c),
                    0xdb60 => (z, 0xdb7d),
                    0xdb64 => (z, 0xdbb1),
                    0xdb6a => (true, 0xdb8c),
                    0xdb6f => (!z, 0xdb78),
                    0xdb76 | 0xdb7b | 0xdb8a => (true, 0xdbd9),
                    0xdb85 => (!z, 0xdb8a),
                    0xdbaf => (true, 0xdb7d),
                    0xdbc3 => (!z, 0xdbdc),
                    0xdbca => (z, 0xdbd2),
                    0xdbd0 => (true, 0xdbb5),
                    0xdc1a | 0xdc2f => (!z, 0xdc49),
                    0xdc26 => (z, 0xdc49),
                    0xdc56 => (!n, 0xdc49),
                    0xdc69 => (!z, 0xdc5e),
                    0xdc92 => (z, 0xdcbc),
                    0xdcb1 => (!z, 0xdcb7),
                    0xdcc3 => (z, 0xdd1a),
                    0xdccd | 0xdcdb => (z, 0xdd1d),
                    0xdcee => (!z, 0xdcf4),
                    0xdcf7 => (true, 0xdd15),
                    _ => (true, 0xdd1d),
                };
                if w.branch(t) {
                    a
                } else {
                    self.pc + 2
                }
            }
            _ => unreachable!("active naming continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn active_naming_matches_source_input_character_and_checksum_paths() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        // All keyboard cells (including arrows and END), slot wraps, input
        // directions, scroll acceleration/arrival and empty/nonempty saves.
        let mut cases = Vec::new();
        for row in 0..4 {
            for col in 0..32 {
                for name_slot in [0, 5] {
                    cases.push((row, col, name_slot, 0, 0, 0, 0x80, false));
                }
            }
        }
        for input in 0..16 {
            cases.push((0, 0, 0, 0, 0, input, 0, false));
            cases.push((3, 31, 5, 0, 0, input, 0, false));
        }
        for step in [1, 5, 29, 33, 45, 49] {
            for direction in [0, 1] {
                for at_target in [false, true] {
                    cases.push((
                        1,
                        2,
                        3,
                        step,
                        if at_target { 1 } else { 0 },
                        direction + 1,
                        0,
                        false,
                    ));
                }
            }
        }
        for row in 0..4 {
            for input in [0, 4, 8] {
                cases.push((row, 0, 0, 0, 2, input, 0, false));
            }
        }
        for nonempty in [false, true] {
            cases.push((0, 0, 0, 0, 0, 0, 0x10, nonempty));
            cases.push((3, 2, 0, 0, 0, 0, 0x80, nonempty));
        }
        for filtered in [0x40, 0x4000, 0x8000] {
            cases.push((0, 0, 0, 0, 0, 0, filtered, false));
        }
        for (case, &(row, col, name_slot, step, vertical, input, filtered, nonempty)) in
            cases.iter().enumerate()
        {
            for save_slot in 0..3 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 8)) as u8;
                }
                for (i, b) in machine.cart.ram.iter_mut().enumerate() {
                    *b = (i.wrapping_mul(7) ^ (i >> 8)) as u8;
                }
                let base = save_slot * 0x500;
                machine.cart.ram[base..base + 0x500].fill(0);
                for i in 0..6 {
                    machine.cart.ram[base + 0x3d9 + i * 2..base + 0x3db + i * 2].copy_from_slice(
                        &(if nonempty && i == 5 { 6u16 } else { 0xa9 }).to_le_bytes(),
                    );
                }
                machine.ram[0xc8..0xca].copy_from_slice(&(save_slot as u16).to_le_bytes());
                machine.ram[0x200..0x202].copy_from_slice(&(base as u16).to_le_bytes());
                machine.ram[0xb10..0xb17].copy_from_slice(&[
                    col,
                    if vertical == 2 { 0x85 } else { 0x83 + row * 16 },
                    name_slot,
                    step,
                    u8::from(vertical == 2),
                    row,
                    u8::from(input == 2),
                ]);
                let scroll = if vertical == 1 {
                    if col == 0 {
                        0x1f0u16
                    } else {
                        (col as u16 - 1) * 16
                    }
                } else {
                    0x1f0
                };
                machine.ram[0x630..0x632].copy_from_slice(&scroll.to_le_bytes());
                machine.ram[0xf0] = input;
                machine.ram[0xf4] = filtered as u8;
                machine.ram[0xf6] = (filtered >> 8) as u8;
                machine.ram[0x1fb..0x1fe].copy_from_slice(&[0x59, 0x80, 0]);
                let mut native = NativeNmiContext {
                    a: 0xab03,
                    x: 0x71,
                    y: 0x83,
                    d: 0,
                    db: 0,
                    bank: 12,
                    pc: 0xda4d,
                    sp: 0x1fa,
                    p: 0x31,
                };
                machine.cpu.a = native.a;
                machine.cpu.x = native.x;
                machine.cpu.y = native.y;
                machine.cpu.k = 12;
                machine.cpu.pc = native.pc;
                machine.cpu.sp = native.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(native.p);
                let mut ram = machine.ram.clone();
                let mut sram = machine.cart.ram.clone();
                let before = sram.clone();
                let (v, h) = if case & 1 == 0 {
                    (224, 1338)
                } else {
                    (261, 1350)
                };
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(v == 261),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = NameFileActive::default();
                let mut count = 0;
                while !work.complete() {
                    let label = format!("naming case{case} save{save_slot} pc{:04x}", native.pc);
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut native,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, a| match a {
                            Access::Read(a, width) if (0x700000..0x702000).contains(&a) => {
                                let i = (a - 0x700000) as usize;
                                Some(if width == 1 {
                                    u16::from(sram[i])
                                } else {
                                    u16::from_le_bytes([sram[i], sram[i + 1]])
                                })
                            }
                            Access::Write(a, value, width) if (0x700000..0x702000).contains(&a) => {
                                let i = (a - 0x700000) as usize;
                                sram[i..i + usize::from(width)]
                                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                                Some(0)
                            }
                            _ => None,
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(ram, source.snes().ram, "{label}");
                    assert_eq!(sram, source.snes().cart.ram, "{label}");
                    work = work.clone();
                    count += 1;
                    assert!(count < 20000, "{label}");
                }
                assert_eq!((native.bank, native.pc, native.sp), (0, 0x805a, 0x1fd));
                assert_eq!(&sram[..base], &before[..base]);
                assert_eq!(&sram[base + 0x500..0x1ffe], &before[base + 0x500..0x1ffe]);
                if nonempty {
                    let sum = sram[base..base + 0x500].chunks_exact(2).fold(0u16, |s, b| {
                        s.wrapping_add(u16::from_le_bytes([b[0], b[1]]))
                    });
                    assert_eq!(sum, 0x5a5a);
                    assert_eq!(ram[0x10], 1);
                    assert_eq!(ram[0x128], 255);
                }
            }
        }
        eprintln!(
            "{} active naming component fixtures matched every instruction",
            cases.len() * 3
        );
    }
}
