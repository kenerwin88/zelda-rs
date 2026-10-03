//! The room's visited flags are published first to live room state, then SRAM
//! shadow state. The intervening width change and return remain resumable.
use super::{native_work::{Work, WorkBus}, nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VisitedFlags {
    pc: u16,
    complete: bool,
}
impl Default for VisitedFlags {
    fn default() -> Self {
        Self {
            pc: 0xb8cb,
            complete: false,
        }
    }
}
impl VisitedFlags {
    pub(crate) fn routine(&self) -> u32 {
        0x02b8cb
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (2, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xb8cb | 0xb8d1 => {
                let v = w.dp_read(if self.pc == 0xb8cb { 0xa7 } else { 0xa6 }, 1);
                w.load(v, 1);
                self.pc + 2
            }
            0xb8cd | 0xb8ce | 0xb8d3 => {
                w.shift_left(1);
                self.pc + 1
            }
            0xb8cf => {
                w.dp_store(0, w.cpu.a & 255, 1);
                self.pc + 2
            }
            0xb8d4 | 0xb8d6 | 0xb8d8 => {
                let v = w.dp_read(
                    match self.pc {
                        0xb8d4 => 0,
                        0xb8d6 => 0xaa,
                        _ => 0xa9,
                    },
                    1,
                );
                w.load((w.cpu.a | v) & 255, 1);
                self.pc + 2
            }
            0xb8da => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0xb8db => {
                w.drain(24);
                let v = w.read(0x02b5cc + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                self.pc + 4
            }
            0xb8df | 0xb8ef => {
                let width = if self.pc == 0xb8df { 1 } else { 2 };
                w.drain(16);
                let v = w.read(w.absolute(0x408), width);
                w.load((w.cpu.a | v) & if width == 1 { 255 } else { 65535 }, width);
                self.pc + 3
            }
            0xb8e2 => {
                w.drain(16);
                w.write(w.absolute(0x408), w.cpu.a & 255, 1);
                self.pc + 3
            }
            0xb8e5 => {
                w.width(0x30, true);
                self.pc + 2
            }
            0xb8e7 => {
                let v = w.dp_read(0xa0, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xb8e9 => {
                w.shift_left(2);
                self.pc + 1
            }
            0xb8ea => {
                w.tax_word();
                self.pc + 1
            }
            0xb8eb => {
                w.drain(24);
                let v = w.read(0x7ef000 + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                self.pc + 4
            }
            0xb8f2 => {
                w.drain(24);
                w.write(0x7ef000 + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0xb8f6 => {
                w.width(0x30, false);
                self.pc + 2
            }
            0xb8f8 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("visited-room continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
