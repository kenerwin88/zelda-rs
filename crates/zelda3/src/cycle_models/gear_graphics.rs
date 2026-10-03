//! Sword/shield sheet callers retain both decompression children and both
//! conversion rows. Equipment and pointers are read from the live native bus.
use super::{
    item_graphics::ItemConversion, native_decompress::NativeDecompression,
    native_work::{Work, WorkBus}, nmi_stack::NativeNmiContext,
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Decompress(NativeDecompression),
    Convert(ItemConversion),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GearGraphics {
    entry: u16,
    pc: u16,
    child: Option<Child>,
    count_prefix: bool,
    complete: bool,
}
impl GearGraphics {
    pub(crate) fn new(sword: bool) -> Self {
        let pc = if sword { 0xd2c8 } else { 0xd308 };
        Self {
            entry: pc,
            pc,
            child: None,
            count_prefix: false,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match &self.child {
            Some(Child::Decompress(w)) => w.routine(),
            Some(Child::Convert(w)) => w.routine(),
            _ => u32::from(self.entry),
        }
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
        if self.count_prefix {
            assert_eq!((cpu.bank, cpu.pc, cpu.p & 0x30), (0, 0xd619, 0));
            let mut w = Work {
                cpu,
                clock,
                bus,
                cycles: 0,
            };
            w.fetch();
            w.drain(16);
            w.cpu.y = 6;
            w.flags(6, 2);
            w.cpu.pc = 0xd61c;
            self.count_prefix = false;
            self.child = Some(Child::Convert(ItemConversion::new(true)));
            return Some(w.cycles);
        }
        if let Some(child) = &mut self.child {
            let (c, done) = match child {
                Child::Decompress(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Convert(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
            };
            if done {
                self.child = None;
                assert_eq!((cpu.bank, cpu.pc), (0, self.pc));
            }
            return c;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (0, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xd2c8 | 0xd2c9 | 0xd308 | 0xd309 => {
                w.drain(6);
                w.push(
                    u16::from(if matches!(self.pc, 0xd2c9 | 0xd309) {
                        w.cpu.bank
                    } else {
                        w.cpu.db
                    }),
                    1,
                );
                self.pc + 1
            }
            0xd2ca | 0xd2fe | 0xd30a | 0xd335 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                self.pc + 1
            }
            0xd2cb | 0xd2d0 | 0xd30b | 0xd310 => {
                w.drain(8);
                w.cpu.y = if matches!(self.pc, 0xd2cb | 0xd30b) {
                    95
                } else {
                    94
                };
                w.flags(w.cpu.y, 1);
                self.pc + 2
            }
            0xd2cd | 0xd2d2 | 0xd30d | 0xd312 => {
                let child = if matches!(self.pc, 0xd2cd | 0xd30d) {
                    NativeDecompression::sprite_secondary()
                } else {
                    NativeDecompression::new(true)
                };
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                w.cpu.pc = child.routine() as u16;
                self.child = Some(Child::Decompress(child));
                self.pc + 3
            }
            0xd2d5 | 0xd315 | 0xd2e5 | 0xd322 | 0xd2fc | 0xd333 => {
                w.width(
                    if matches!(self.pc, 0xd2d5 | 0xd315) {
                        0x21
                    } else if matches!(self.pc, 0xd2e5 | 0xd322) {
                        0x10
                    } else {
                        0x30
                    },
                    !matches!(self.pc, 0xd2fc | 0xd333),
                );
                self.pc + 2
            }
            0xd2d7 | 0xd317 => {
                w.drain(24);
                let v = w.read(
                    if self.pc == 0xd2d7 {
                        0x7ef359
                    } else {
                        0x7ef35a
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 4
            }
            0xd2db => {
                w.mask(255, 2);
                self.pc + 3
            }
            0xd2de | 0xd31b => {
                w.shift_left(2);
                self.pc + 1
            }
            0xd2df | 0xd31c => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                self.pc + 1
            }
            0xd2e0 | 0xd31d => {
                let v = w.dp_read(0, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xd2e2 | 0xd31f => {
                let a = if self.pc == 0xd2e2 { 0xd2be } else { 0xd300 };
                w.drain(16);
                if (a & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(w.cpu.y), 2);
                w.adc(v);
                self.pc + 3
            }
            0xd2e7 | 0xd324 => {
                w.drain(16);
                w.cpu.x = if self.pc == 0xd2e7 { 0 } else { 0x300 };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0xd2ea | 0xd2f6 => {
                w.drain(16);
                w.cpu.y = 12;
                w.flags(12, 2);
                self.pc + 3
            }
            0xd2ed | 0xd327 => {
                w.drain(6);
                w.push(w.cpu.a, 2);
                self.pc + 1
            }
            0xd2f1 | 0xd32b => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                self.pc + 1
            }
            0xd2f2 | 0xd32c => {
                w.clear_carry();
                self.pc + 1
            }
            0xd2f3 | 0xd32d => {
                w.drain(16);
                w.adc(0x180);
                self.pc + 3
            }
            0xd2ee | 0xd2f9 | 0xd328 | 0xd330 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                w.cpu.pc = if matches!(self.pc, 0xd328 | 0xd330) {
                    0xd619
                } else {
                    0xd61c
                };
                self.count_prefix = matches!(self.pc, 0xd328 | 0xd330);
                if !self.count_prefix {
                    self.child = Some(Child::Convert(ItemConversion::new(true)));
                }
                self.pc + 3
            }
            0xd2ff | 0xd336 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("gear caller continuation {:04x}", self.pc),
        };
        // D619 is the real LDY #6 prefix of the shared high-palette converter.
        // Retain it as a separate instruction before entering that child.
        if self.child.is_none() && !self.count_prefix {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}
