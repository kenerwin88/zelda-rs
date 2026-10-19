//! Overworld_SetSongList on the properties caller's real far frame.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldSongList {
    pc: u16,
    complete: bool,
}
impl Default for OverworldSongList {
    fn default() -> Self {
        Self {
            pc: 0xc463,
            complete: false,
        }
    }
}
impl OverworldSongList {
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
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x28),
            (2, self.pc, 0, 0x20)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let xw = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if xw == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match pc {
            0xc463 | 0xc464 => {
                w.drain(6);
                w.push(
                    u16::from(if pc == 0xc463 { w.cpu.db } else { w.cpu.bank }),
                    1,
                );
                pc + 1
            }
            0xc465 | 0xc4be => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xc466 | 0xc4bc => {
                w.width(0x10, pc == 0xc466);
                pc + 2
            }
            0xc468 | 0xc485 => {
                w.immediate(if pc == 0xc468 { 2 } else { 5 }, 1);
                pc + 2
            }
            0xc46a | 0xc487 => {
                w.dp_store(0, w.cpu.a & 255, 1);
                pc + 2
            }
            0xc46c => {
                w.drain(8 * u32::from(xw));
                w.cpu.x = 0;
                w.flags(0, xw);
                pc + 1 + u16::from(xw)
            }
            0xc46f | 0xc47a | 0xc489 | 0xc494 | 0xc4a5 => {
                w.drain(8 * u32::from(xw));
                w.cpu.y = match pc {
                    0xc46f => 0xc0,
                    0xc47a => 0x80,
                    0xc489 => 0x40,
                    _ => 0,
                };
                w.flags(w.cpu.y, xw);
                pc + 1 + u16::from(xw)
            }
            0xc472 | 0xc47d | 0xc48c => {
                w.drain(24);
                let value = w.read(if pc == 0xc47d { 0x7ef359 } else { 0x7ef3c5 }, 1);
                w.load(value, 1);
                pc + 4
            }
            0xc476 | 0xc481 | 0xc490 => {
                w.compare_immediate(if pc == 0xc476 { 3 } else { 2 }, 1);
                pc + 2
            }
            0xc478 | 0xc483 | 0xc492 => {
                if w.branch(w.cpu.p & CARRY != 0) {
                    0xc497
                } else {
                    pc + 2
                }
            }
            0xc497 | 0xc4a8 => {
                w.drain(16);
                let base: u16 = if pc == 0xc497 { 0xc303 } else { 0xc403 };
                if xw == 2 || (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.y)), 1);
                w.load(value, 1);
                pc + 3
            }
            0xc49a | 0xc4ab => {
                w.drain(24);
                w.write(0x7f5b00 + u32::from(w.cpu.x), w.cpu.a & 255, 1);
                pc + 4
            }
            0xc49e | 0xc4b0 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & mask;
                w.flags(w.cpu.y, xw);
                pc + 1
            }
            0xc49f | 0xc4af => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & mask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xc4a0 => {
                w.compare_x_immediate(0x40, xw);
                pc + 1 + u16::from(xw)
            }
            0xc4b1 => {
                w.drain(8 * u32::from(xw));
                let old = w.cpu.y;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old >= 0x60);
                w.flags(old.wrapping_sub(0x60) & mask, xw);
                pc + 1 + u16::from(xw)
            }
            0xc4a3 | 0xc4b4 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    if pc == 0xc4a3 {
                        0xc497
                    } else {
                        0xc4a8
                    }
                } else {
                    pc + 2
                }
            }
            0xc4b6 => {
                let value = w.dp_read(0, 1);
                w.load(value, 1);
                pc + 2
            }
            0xc4b8 => {
                w.drain(24);
                w.write(0x7f5b80, w.cpu.a & 255, 1);
                pc + 4
            }
            0xc4bf => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("song list PC {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
