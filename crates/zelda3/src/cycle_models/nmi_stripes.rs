//! Shared NMI stripe caller and copy/fill DMA routine. Live registers, scratch
//! RAM, pointer reads and channel state own progress; no opcode decoder is used.
use super::nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Access {
    Read(u32, u8),
    Write(u32, u16, u8),
    /// Includes the nested DMA and enclosing six-clock CPU write.
    Dma(u8),
}

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    const POINTERS: [u8; 28] = [
        0x60, 2, 0, 0x6d, 0x1b, 0xbf, 0xa8, 0x3c, 0x56, 0x9c, 0x10, 0x10, 0xdd, 2, 0xe7, 0xe2,
        0xe6, 0xe4, 0xda, 0, 0, 0x0c, 0, 0x0c, 0x0c, 0x0c, 0x0c, 0x0e,
    ];
    address
        .checked_sub(0x937a)
        .and_then(|i| POINTERS.get(i as usize))
        .copied()
}

pub(crate) const ASSET_SOURCES: [(usize, u32); 6] = [
    (99, 0x0cdd6d),
    (100, 0x0ce7bf),
    (101, 0x0ce2a8),
    (102, 0x0ce63c),
    (103, 0x0ce456),
    (104, 0x0eda9c),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StripeUpload {
    pc: u16,
}
impl Default for StripeUpload {
    fn default() -> Self {
        Self { pc: 0x8bd3 }
    }
}
impl StripeUpload {
    pub(crate) fn complete(&self) -> bool {
        self.pc == 0x8bf3
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(&mut CpuMasterTimeline, Access) -> u16,
    ) -> Option<u64> {
        if self.complete() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.db, cpu.d), (0, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let width = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        w.clock.advance_synchronous_pcbase_opcode_fetch(8);
        w.cycles += 8;
        let next = match self.pc {
            0x8bd3 | 0x8bd8 | 0x8bdd => {
                let base = match self.pc {
                    0x8bd3 => 0x937au16,
                    0x8bd8 => 0x9383,
                    _ => 0x938c,
                };
                w.drain(16);
                if (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let v = w.read(u32::from(base.wrapping_add(w.cpu.y)), 1);
                w.load(v, 1);
                self.pc + 3
            }
            0x8bd6 | 0x8bdb | 0x8be0 => {
                w.drain(8);
                let a = match self.pc {
                    0x8bd6 => 0,
                    0x8bdb => 1,
                    _ => 2,
                };
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 2
            }
            0x8be2 => {
                w.drain(22);
                w.write(u32::from(w.cpu.sp - 1), 0x8be4, 2);
                w.cpu.sp -= 2;
                0x92a1
            }
            0x8be5 => {
                w.dp_load(0x14, 1);
                0x8be7
            }
            0x8be7 => {
                w.drain(8);
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= 1);
                w.flags(a.wrapping_sub(1) & 255, 1);
                0x8be9
            }
            0x8be9 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0x8bf1
                } else {
                    0x8beb
                }
            }
            0x8beb | 0x8bee => {
                w.drain(16);
                w.write(if self.pc == 0x8beb { 0x1000 } else { 0x1001 }, 0, 1);
                self.pc + 3
            }
            0x8bf1 => {
                w.drain(8);
                w.write(0x14, 0, 1);
                0x8bf3
            }
            0x92a1 | 0x92d5 | 0x9314 => {
                w.drain(14);
                w.cpu.p &= !match self.pc {
                    0x92a1 => 0x10,
                    0x92d5 => 0x20,
                    _ => 0x21,
                };
                self.pc + 2
            }
            0x92af | 0x9344 | 0x92fb | 0x932f => {
                w.drain(14);
                w.cpu.p |= if matches!(self.pc, 0x92af | 0x9344) {
                    0x30
                } else {
                    0x20
                };
                if w.cpu.p & 0x10 != 0 {
                    w.cpu.x &= 255;
                    w.cpu.y &= 255;
                }
                self.pc + 2
            }
            0x92b1 | 0x9346 => {
                w.drain(12);
                let pc = w.read(u32::from(w.cpu.sp + 1), 2);
                w.cpu.sp += 2;
                w.drain(6);
                pc.wrapping_add(1)
            }
            0x92a3 | 0x92cd | 0x92d2 | 0x92d9 | 0x92ed | 0x9302 | 0x9307 | 0x9311 | 0x931a
            | 0x931f | 0x9335 => {
                let a = match self.pc {
                    0x92a3 => 0x4314,
                    0x92cd | 0x9302 => 0x4310,
                    0x92d2 | 0x9311 => 0x4311,
                    0x92d9 | 0x931f => 0x2116,
                    0x92ed | 0x931a => 0x4312,
                    _ => 0x2115,
                };
                w.drain(16);
                w.write(a, w.cpu.a, width);
                self.pc + 3
            }
            0x92a6 => {
                w.drain(8);
                w.write(6, 0, 1);
                0x92a8
            }
            0x92a8 => {
                w.drain(16);
                w.cpu.y = 0;
                w.flags(0, 2);
                0x92ab
            }
            0x92ab | 0x92b5 | 0x92ba | 0x92c2 | 0x92dc | 0x933d => {
                w.drain(8);
                let low = w.read(0, 2);
                let bank = w.read(2, 1);
                let a = ((u32::from(bank) << 16) | u32::from(low)).wrapping_add(u32::from(w.cpu.y))
                    & 0xffffff;
                let v = w.read(a, width);
                w.load(v, width);
                self.pc + 2
            }
            0x92ad => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    0x92b2
                } else {
                    0x92af
                }
            }
            0x933f => {
                if w.branch(w.cpu.p & NEGATIVE != 0) {
                    0x9344
                } else {
                    0x9341
                }
            }
            0x92b2 | 0x92b7 | 0x92c0 | 0x92c6 => {
                w.drain(8);
                let a = match self.pc {
                    0x92b2 => 4,
                    0x92b7 => 3,
                    0x92c0 => 7,
                    _ => 5,
                };
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 2
            }
            0x92b4 | 0x92b9 | 0x92e7 | 0x92e8 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x92bc | 0x92c4 | 0x92df => {
                w.drain(8 * u32::from(width));
                w.load(
                    w.cpu.a
                        & match self.pc {
                            0x92bc => 0x80,
                            0x92c4 => 0x40,
                            _ => 0x3fff,
                        },
                    width,
                );
                self.pc + 1 + u16::from(width)
            }
            0x92be => {
                w.drain(6);
                let v = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(v & 128 != 0);
                w.load((v << 1) & 255, 1);
                0x92bf
            }
            0x92bf => {
                w.drain(6);
                let v = w.cpu.a & 255;
                let c = w.cpu.p & CARRY;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(v & 128 != 0);
                w.load(((v << 1) | u16::from(c)) & 255, 1);
                0x92c0
            }
            0x92c8 | 0x92c9 | 0x92ca | 0x92f6 | 0x92ff | 0x9300 | 0x9301 => {
                w.drain(6);
                let v = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (v as u8 & 1);
                w.load(v >> 1, width);
                self.pc + 1
            }
            0x92cb | 0x9333 => {
                w.drain(8);
                w.load(
                    (w.cpu.a | if self.pc == 0x92cb { 1 } else { 0x80 }) & 255,
                    1,
                );
                self.pc + 2
            }
            0x92d0 | 0x930a | 0x930f | 0x9338 => {
                w.drain(8);
                w.load(
                    match self.pc {
                        0x92d0 => 0x18,
                        0x930f => 0x19,
                        _ => 2,
                    },
                    1,
                );
                self.pc + 2
            }
            0x92d7 | 0x92f0 | 0x92fd | 0x9305 | 0x931d | 0x9331 => {
                w.dp_load(
                    match self.pc {
                        0x92d7 | 0x931d => 3,
                        0x92f0 | 0x92fd => 5,
                        _ => 7,
                    },
                    width,
                );
                self.pc + 2
            }
            0x92de => {
                w.drain(12);
                w.cpu.a = w.cpu.a.swap_bytes();
                w.flags(w.cpu.a & 255, 1);
                0x92df
            }
            0x92e2 | 0x92f7 => {
                w.drain(6);
                w.cpu.x = w.cpu.a;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x92e3 | 0x92f4 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x92e4 | 0x92f8 | 0x9322 => {
                w.drain(16);
                w.write(0x4315, w.cpu.x, 2);
                self.pc + 3
            }
            0x92e9 | 0x9316 | 0x932a => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x92ea | 0x932b => {
                w.drain(6);
                w.cpu.p &= !CARRY;
                self.pc + 1
            }
            0x92eb | 0x9317 | 0x932c => {
                w.drain(8);
                let rhs = w.read(if self.pc == 0x932c { 3 } else { 0 }, 2);
                w.adc(rhs);
                self.pc + 2
            }
            0x92f2 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0x9328
                } else {
                    0x92f4
                }
            }
            0x92f5 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                0x92f6
            }
            0x930c | 0x933a => {
                w.drain(16);
                (w.bus)(w.clock, Access::Dma(w.cpu.a as u8));
                w.cycles += 6;
                self.pc + 3
            }
            0x9319 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_add(1), 2);
                0x931a
            }
            0x9325 => {
                w.drain(16);
                w.cpu.x = 2;
                w.flags(2, 2);
                0x9328
            }
            0x9328 => {
                w.drain(8);
                w.write(3, w.cpu.x, 2);
                0x932a
            }
            0x932e => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                0x932f
            }
            0x9341 => {
                w.drain(16);
                0x92b2
            }
            pc => panic!("unowned stripe continuation {pc:04x}"),
        };
        self.pc = next;
        w.cpu.pc = next;
        Some(w.cycles)
    }
}

struct Work<'a, F> {
    cpu: &'a mut NativeNmiContext,
    clock: &'a mut CpuMasterTimeline,
    bus: F,
    cycles: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, Snes9xColdCpuExecutor,
        SourceCpuBusAccessKind as Kind,
    };

    fn byte(ram: &[u8], rom: &[u8], a: u32) -> u8 {
        if (0x7e0000..0x800000).contains(&a) {
            ram[(a - 0x7e0000) as usize]
        } else if a & 0x400000 == 0 && a & 65535 < 0x2000 {
            ram[(a & 65535) as usize]
        } else {
            rom[(((a >> 16) & 127) * 0x8000 + (a & 0x7fff)) as usize]
        }
    }

    #[test]
    fn stripe_copy_fill_and_caller_match_source_instructions_and_dma() {
        let mut rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in 0x937a..=0x9395 {
            assert_eq!(table_byte(a), Some(rom[a as usize - 0x8000]));
        }
        // Test setup only: enter native mode, force blank and park. All source
        // stripe instructions, pointer tables and asset data stay unchanged.
        rom[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0x80, 0xfe]);
        rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
        let mut total = 0;
        let mut cases = 0;
        for mode in 1..=9u8 {
            for &length in if matches!(mode, 1 | 2 | 4) {
                &[0usize, 1, 2, 3, 32, 257][..]
            } else {
                &[0usize][..]
            } {
                for (v, h) in [(100, 510), (225, 1350), (261, 1300)] {
                    let mut source = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
                    let target = CpuMasterTimeline::at_raster(
                        1,
                        CpuRasterPosition::new(v, h),
                        CpuBusWorkload::default(),
                        CpuFieldTiming::NON_INTERLACE_EVEN,
                    )
                    .clock_master_cycles();
                    while source.machine().timestamp().master_cycles() < target {
                        source.step().unwrap();
                    }
                    let mut ram = source.machine().snes().ram.clone();
                    if matches!(mode, 1 | 2 | 4) {
                        let start = match mode {
                            1 => 0x1002,
                            2 => 0x1000,
                            _ => 0x21b,
                        };
                        let mut stream = Vec::new();
                        for flags in [0u8, 0x40, 0x80, 0xc0].into_iter().filter(|_| length != 0) {
                            stream.extend([
                                0x7f,
                                0xfe,
                                flags | ((length - 1) >> 8) as u8,
                                (length - 1) as u8,
                            ]);
                            stream.extend(
                                (0..if flags & 0x40 != 0 { 2 } else { length })
                                    .map(|i| (i * 47 + 0x5a) as u8),
                            );
                        }
                        stream.push(0xff);
                        ram[start..start + stream.len()].copy_from_slice(&stream);
                    }
                    ram[0x14] = mode;
                    let mut cpu = NativeNmiContext {
                        a: 0xab35,
                        x: 0x37,
                        y: u16::from(mode),
                        d: 0,
                        db: 0,
                        bank: 0,
                        pc: 0x8bd3,
                        sp: 0x1f0,
                        p: 0x75,
                    };
                    let mut registers = source.machine().snes().cpu.clone();
                    registers.a = cpu.a;
                    registers.x = cpu.x;
                    registers.y = cpu.y;
                    registers.dp = 0;
                    registers.db = 0;
                    registers.k = 0;
                    registers.pc = cpu.pc;
                    registers.sp = cpu.sp;
                    registers.e = false;
                    registers.unpack_flags(cpu.p);
                    let at = source.machine().timestamp();
                    let sram = source.machine().snes().cart.ram.clone();
                    source
                        .rebase_private_timing_state_at(at, &registers, &ram, &sram)
                        .unwrap();
                    let mut clock = source.machine().timeline().clone();
                    let mut ppu = source.machine().snes().ppu.clone();
                    let mut dma = source.machine().snes().dma.clone();
                    let mut work = StripeUpload::default();
                    while !work.complete() {
                        let pc = cpu.pc;
                        let receipt = source.step().unwrap();
                        let mut first_read = true;
                        let expected: Vec<_> = receipt
                            .accesses
                            .into_iter()
                            .filter_map(|a| {
                                let (access, value) = match a.kind {
                                    Kind::Read { value, width } => {
                                        let operand = first_read && a.address == u32::from(pc + 1);
                                        first_read = false;
                                        if operand {
                                            return None;
                                        }
                                        (Access::Read(a.address, width), value)
                                    }
                                    Kind::Write { value, width: 1 } if a.address == 0x420b => {
                                        (Access::Dma(value as u8), 0)
                                    }
                                    Kind::Write { value, width } => {
                                        (Access::Write(a.address, value, width), 0)
                                    }
                                    _ => return None,
                                };
                                Some((a.timestamp.master_cycles(), access, value))
                            })
                            .collect();
                        let mut actual = Vec::new();
                        work.step(&mut cpu, &mut clock, |clock, access| {
                            let at = clock.clock_master_cycles();
                            let value = match access {
                                Access::Read(a, width) => {
                                    let low = byte(&ram, &rom, a);
                                    if width == 1 {
                                        u16::from(low)
                                    } else {
                                        u16::from_le_bytes([low, byte(&ram, &rom, a + 1)])
                                    }
                                }
                                Access::Write(a, v, width) => {
                                    if a < 0x2000 {
                                        ram[a as usize..a as usize + usize::from(width)]
                                            .copy_from_slice(
                                                &v.to_le_bytes()[..usize::from(width)],
                                            );
                                    } else if a >= 0x4300 {
                                        assert_eq!(width, 1);
                                        dma.write_register(a as u16, v as u8);
                                    } else {
                                        assert_eq!(width, 1);
                                        ppu.write(a as u8, v as u8);
                                    }
                                    0
                                }
                                Access::Dma(mask) => {
                                    dma.run_synchronous_forward_ppu(clock, mask, |_, _, a, b| {
                                        ppu.write(b, byte(&ram, &rom, a))
                                    });
                                    0
                                }
                            };
                            actual.push((at, access, value));
                            value
                        })
                        .unwrap();
                        let label =
                            format!("mode={mode} length={length} raster={v},{h} pc={pc:04x}");
                        assert_eq!(actual, expected, "bus {label}");
                        assert_eq!(clock, *source.machine().timeline(), "clock {label}");
                        let c = &source.machine().snes().cpu;
                        assert_eq!(
                            cpu,
                            NativeNmiContext {
                                a: c.a,
                                x: c.x,
                                y: c.y,
                                d: c.dp,
                                db: c.db,
                                bank: c.k,
                                pc: c.pc,
                                sp: c.sp,
                                p: c.pack_flags()
                            },
                            "CPU {label}"
                        );
                        assert_eq!(ram, source.machine().snes().ram, "RAM {label}");
                        assert_eq!(ppu.vram, source.machine().snes().ppu.vram, "VRAM {label}");
                        assert_eq!(
                            ppu.vram_pointer,
                            source.machine().snes().ppu.vram_pointer,
                            "VRAM pointer {label}"
                        );
                        assert_eq!(
                            format!("{dma:?}"),
                            format!("{:?}", source.machine().snes().dma),
                            "DMA {label}"
                        );
                        work = work.clone();
                        total += 1;
                        assert!(total < 1_000_000);
                    }
                    cases += 1;
                }
            }
        }
        eprintln!("{cases} stripe cases, {total} instruction/DMA boundaries checked");
    }
}
impl<F: FnMut(&mut CpuMasterTimeline, Access) -> u16> Work<'_, F> {
    fn drain(&mut self, n: u32) {
        self.cycles += u64::from(n);
        self.clock
            .advance_synchronous_after_semantics_with(n, |_, _| Ok::<_, String>(0))
            .unwrap();
    }
    fn read(&mut self, a: u32, width: u8) -> u16 {
        let v = (self.bus)(self.clock, Access::Read(a, width));
        self.drain(8 * u32::from(width));
        v
    }
    fn write(&mut self, a: u32, v: u16, width: u8) {
        if (0x2000..0x6000).contains(&a) {
            for i in 0..width {
                (self.bus)(
                    self.clock,
                    Access::Write(a + u32::from(i), (v >> (i * 8)) & 255, 1),
                );
                self.drain(6);
            }
        } else {
            (self.bus)(self.clock, Access::Write(a, v, width));
            self.drain(8 * u32::from(width));
        }
    }
    fn flags(&mut self, v: u16, width: u8) {
        self.cpu.p = (self.cpu.p & !(ZERO | NEGATIVE))
            | if v == 0 { ZERO } else { 0 }
            | if width == 1 {
                v as u8 & NEGATIVE
            } else {
                (v >> 8) as u8 & NEGATIVE
            };
    }
    fn load(&mut self, v: u16, width: u8) {
        self.cpu.a = if width == 1 {
            (self.cpu.a & 0xff00) | (v & 255)
        } else {
            v
        };
        self.flags(v, width);
    }
    fn dp_load(&mut self, a: u32, width: u8) {
        self.drain(8);
        let v = self.read(a, width);
        self.load(v, width);
    }
    fn branch(&mut self, taken: bool) -> bool {
        self.drain(8);
        if taken {
            self.drain(6);
        }
        taken
    }
    fn adc(&mut self, rhs: u16) {
        assert_eq!(self.cpu.p & 8, 0);
        let old = self.cpu.a;
        let sum = u32::from(old) + u32::from(rhs) + u32::from(self.cpu.p & CARRY);
        let v = sum as u16;
        self.cpu.p = (self.cpu.p & !0x41)
            | u8::from(sum > 65535)
            | if (!(old ^ rhs) & (old ^ v) & 0x8000) != 0 {
                0x40
            } else {
                0
            };
        self.load(v, 2);
    }
}
