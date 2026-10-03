//! Shared CPU song upload and conditional load-game bank selection.
//! The CPU retains headers, counters, pointers, flags and real call frames;
//! acknowledgements come only from the live APU at each timed port access.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Call {
    entry: u32,
    returned: u32,
    far: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SongBank {
    pc: u32,
    entry: u32,
    calls: Vec<Call>,
    complete: bool,
}
impl SongBank {
    pub(crate) fn from_entry(entry: u32) -> Self {
        assert!(matches!(
            entry,
            0x029bd7 | 0x02854c | 0x008913 | 0x008925 | 0x008888
        ));
        Self {
            pc: entry,
            entry,
            calls: Vec::new(),
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        self.calls.last().map_or(self.entry, |c| c.entry)
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
        assert_eq!(cpu.db & 0x40, 0, "song upload requires system-bank I/O");
        assert_eq!(
            (
                (u32::from(cpu.bank) << 16) | u32::from(cpu.pc),
                cpu.d,
                cpu.p & 8
            ),
            (self.pc, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x2854c | 0x29bd7 | 0x29bee => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x2854c | 0x29bee => 0x136,
                    0x29bd7 => 0x132,
                    _ => unreachable!(),
                });
                let v = w.read(a, 1);
                w.load(v, 1);
                self.pc + 3
            }
            0x2854f | 0x29bdc | 0x29be0 | 0x29be4 | 0x29be8 | 0x29bec | 0x88ca => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    (match self.pc {
                        0x2854f => 0x28569,
                        0x29bdc | 0x29be0 => 0x29c0b,
                        0x29be4 | 0x29be8 | 0x29bec => 0x29c0c,
                        0x88ca => 0x88c8,
                        _ => unreachable!(),
                    }) as u32
                } else {
                    self.pc + 2
                }
            }
            0x28551 | 0x29bf3 | 0x891f => {
                w.drain(6);
                w.cpu.p |= 4;
                self.pc + 1
            }
            0x28552 | 0x28555 | 0x29bf4 | 0x29bf7 | 0x88f3 | 0x88f6 | 0x88f9 | 0x88fc => {
                w.drain(16);
                w.write_io(
                    match self.pc {
                        0x28552 | 0x29bf4 => 0x4200,
                        0x28555 | 0x29bf7 => 0x420c,
                        0x88f3 => 0x2140,
                        0x88f6 => 0x2141,
                        0x88f9 => 0x2142,
                        0x88fc => 0x2143,
                        _ => unreachable!(),
                    },
                    0,
                    1,
                );
                self.pc + 3
            }
            0x28558 => {
                w.drain(16);
                let a = w.absolute(0x136);
                w.write(a, 0, 1);
                self.pc + 3
            }
            0x2855b | 0x28564 | 0x29bfd | 0x29c06 | 0x8925 | 0x8929 | 0x892d | 0x8913 | 0x8917
            | 0x891b | 0x8898 | 0x88a0 | 0x88e0 => {
                w.drain(8);
                let v = match self.pc {
                    0x2855b | 0x29bfd => 0xff,
                    0x28564 | 0x29c06 => 0x81,
                    0x8925 | 0x88a0 | 0x88e0 => 0x0,
                    0x8929 => 0x80,
                    0x892d => 0x1b,
                    0x8913 => 0xf5,
                    0x8917 => 0x9e,
                    0x891b => 0x1a,
                    0x8898 => 0xcc,
                    _ => unreachable!(),
                };
                w.load(v, 1);
                self.pc + 2
            }
            0x2855d | 0x28566 | 0x29bff | 0x29c08 | 0x88e3 | 0x88e9 => {
                w.drain(16);
                w.write_io(
                    match self.pc {
                        0x2855d | 0x29bff | 0x88e9 => 0x2140,
                        0x28566 | 0x29c08 => 0x4200,
                        0x88e3 => 0x2141,
                        _ => unreachable!(),
                    },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 3
            }
            0x28560 | 0x29c02 => {
                let target = (match self.pc {
                    0x28560 => 0x8913,
                    0x29c02 => 0x8925,
                    _ => unreachable!(),
                }) as u32;
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(w.cpu.pc + 3, 2);
                self.calls.push(Call {
                    entry: target,
                    returned: self.pc + 4,
                    far: true,
                });
                target
            }
            0x28569 | 0x29c0b | 0x8900 => {
                w.rts();
                if let Some(call) = self.calls.pop() {
                    assert_eq!(call.far, false);
                    assert_eq!(
                        (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc),
                        call.returned
                    );
                } else {
                    self.complete = true;
                }
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x29bda | 0x29bde | 0x29be2 | 0x29be6 | 0x29bea => {
                w.drain(8);
                let v = match self.pc {
                    0x29bda => 0xff,
                    0x29bde => 0xf2,
                    0x29be2 => 0x3,
                    0x29be6 => 0x7,
                    0x29bea => 0xe,
                    _ => unreachable!(),
                };
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & 255, 1);
                self.pc + 2
            }
            0x29bf1 | 0x88ab | 0x88b6 | 0x88c1 | 0x88c6 | 0x88ef => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    (match self.pc {
                        0x29bf1 => 0x29c0b,
                        0x88ab => 0x88b2,
                        0x88b6 => 0x88b3,
                        0x88c1 => 0x88a4,
                        0x88c6 => 0x88c3,
                        0x88ef => 0x88ec,
                        _ => unreachable!(),
                    }) as u32
                } else {
                    self.pc + 2
                }
            }
            0x29bfa => {
                w.drain(16);
                let a = w.absolute(0x136);
                let v = w.read(a, 1).wrapping_add(1) & 255;
                w.drain(6);
                w.flags(v, 1);
                w.write(a, v, 1);
                self.pc + 3
            }
            0x29c0c => {
                w.drain(16);
                (0x2854c) as u32
            }
            0x891d | 0x8927 | 0x892b | 0x8915 | 0x8919 => {
                w.dp_store(
                    match self.pc {
                        0x891d => 0x2,
                        0x8927 | 0x8915 => 0x0,
                        0x892b | 0x8919 => 0x1,
                        _ => unreachable!(),
                    },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 2
            }
            0x8920 => {
                let target = (0x8888) as u32;
                w.operand_then_internal(2);
                w.push(w.cpu.pc + 2, 2);
                self.calls.push(Call {
                    entry: target,
                    returned: self.pc + 3,
                    far: false,
                });
                target
            }
            0x8923 => {
                w.drain(6);
                w.cpu.p &= !4;
                self.pc + 1
            }
            0x8924 => {
                w.rtl();
                if let Some(call) = self.calls.pop() {
                    assert_eq!(call.far, true);
                    assert_eq!(
                        (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc),
                        call.returned
                    );
                } else {
                    self.complete = true;
                }
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x892f | 0x889a | 0x88a2 => {
                if w.branch(true) {
                    (match self.pc {
                        0x892f => 0x891d,
                        0x889a => 0x88cc,
                        0x88a2 => 0x88b9,
                        _ => unreachable!(),
                    }) as u32
                } else {
                    self.pc + 2
                }
            }
            0x8888 => {
                w.drain(6);
                w.push(u16::from(w.cpu.p), 1);
                self.pc + 1
            }
            0x8889 | 0x88b9 | 0x88cd => {
                w.width(
                    match self.pc {
                        0x8889 => 0x30,
                        0x88b9 | 0x88cd => 0x20,
                        _ => unreachable!(),
                    },
                    true,
                );
                self.pc + 2
            }
            0x888b | 0x88ad => {
                w.drain(16);
                let v = 0x0;
                w.cpu.y = v;
                w.flags(v, 2);
                self.pc + 3
            }
            0x888e => {
                w.drain(16);
                let v = 0xbbaa;
                w.load(v, 2);
                self.pc + 3
            }
            0x8891 => {
                w.drain(16);
                let v = w.read_io(0x2140, 2);
                let a = w.cpu.a & 65535;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & 65535, 2);
                self.pc + 3
            }
            0x8894 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    (0x8891) as u32
                } else {
                    self.pc + 2
                }
            }
            0x8896 | 0x88be | 0x88db => {
                w.width(0x20, false);
                self.pc + 2
            }
            0x889c | 0x88a5 => {
                let a = w.indirect_y_address(0x0);
                let v = w.read(a, 1);
                w.load(v, 1);
                self.pc + 2
            }
            0x889e | 0x88a7 | 0x88d1 | 0x88d2 | 0x88d6 | 0x88d7 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & 65535;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x889f | 0x88a4 | 0x88b2 => {
                w.swap_bytes();
                self.pc + 1
            }
            0x88a8 => {
                w.drain(16);
                let v = 0x8000;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= v);
                w.flags(w.cpu.y.wrapping_sub(v) & 65535, 2);
                self.pc + 3
            }
            0x88b0 => {
                w.dp_increment(0x2, 1, true);
                self.pc + 2
            }
            0x88b3 | 0x88c3 | 0x88ec => {
                w.drain(16);
                let v = w.read_io(0x2140, 1);
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & 255, 1);
                self.pc + 3
            }
            0x88b8 => {
                w.increment_a(1);
                self.pc + 1
            }
            0x88bb | 0x88d8 => {
                w.drain(16);
                w.write_io(
                    match self.pc {
                        0x88bb => 0x2140,
                        0x88d8 => 0x2142,
                        _ => unreachable!(),
                    },
                    w.cpu.a & 65535,
                    2,
                );
                self.pc + 3
            }
            0x88c0 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 65535;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x88c8 | 0x88e6 => {
                w.drain(8);
                w.adc_byte(match self.pc {
                    0x88c8 => 0x3,
                    0x88e6 => 0x7f,
                    _ => unreachable!(),
                });
                self.pc + 2
            }
            0x88cc => {
                w.drain(6);
                w.push(w.cpu.a & 255, 1);
                self.pc + 1
            }
            0x88cf | 0x88d4 => {
                let a = w.indirect_y_address(0x0);
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0x88d3 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 65535;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x88dd => {
                w.compare_x_immediate(0x1, 2);
                self.pc + 3
            }
            0x88e2 => {
                w.drain(6);
                let a = w.cpu.a & 255;
                let carry = u16::from(w.cpu.p & CARRY != 0);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a & 128 != 0);
                w.load(((a << 1) | carry) & 255, 1);
                self.pc + 1
            }
            0x88e8 => {
                w.drain(12);
                let v = w.pull(1);
                w.load(v, 1);
                self.pc + 1
            }
            0x88f1 => {
                if w.branch(w.cpu.p & 0x40 != 0) {
                    (0x889c) as u32
                } else {
                    self.pc + 2
                }
            }
            0x88ff => {
                w.drain(12);
                w.cpu.p = w.pull(1) as u8;
                if w.cpu.p & 0x10 != 0 {
                    w.cpu.x &= 255;
                    w.cpu.y &= 255;
                }
                self.pc + 1
            }
            _ => unreachable!("song-bank continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc as u16;
        w.cpu.bank = (self.pc >> 16) as u8;
        Some(w.cycles)
    }
}
#[cfg(test)]
#[path = "song_bank_tests.rs"]
mod tests;
#[cfg(test)]
pub(super) const OWNED_POSITIONS: &[u32] = &[
    0x8888, 0x8889, 0x888b, 0x888e, 0x8891, 0x8894, 0x8896, 0x8898, 0x889a, 0x889c, 0x889e, 0x889f,
    0x88a0, 0x88a2, 0x88a4, 0x88a5, 0x88a7, 0x88a8, 0x88ab, 0x88ad, 0x88b0, 0x88b2, 0x88b3, 0x88b6,
    0x88b8, 0x88b9, 0x88bb, 0x88be, 0x88c0, 0x88c1, 0x88c3, 0x88c6, 0x88c8, 0x88ca, 0x88cc, 0x88cd,
    0x88cf, 0x88d1, 0x88d2, 0x88d3, 0x88d4, 0x88d6, 0x88d7, 0x88d8, 0x88db, 0x88dd, 0x88e0, 0x88e2,
    0x88e3, 0x88e6, 0x88e8, 0x88e9, 0x88ec, 0x88ef, 0x88f1, 0x88f3, 0x88f6, 0x88f9, 0x88fc, 0x88ff,
    0x8900, 0x8913, 0x8915, 0x8917, 0x8919, 0x891b, 0x891d, 0x891f, 0x8920, 0x8923, 0x8924, 0x8925,
    0x8927, 0x8929, 0x892b, 0x892d, 0x892f, 0x2854c, 0x2854f, 0x28551, 0x28552, 0x28555, 0x28558,
    0x2855b, 0x2855d, 0x28560, 0x28564, 0x28566, 0x28569, 0x29bd7, 0x29bda, 0x29bdc, 0x29bde,
    0x29be0, 0x29be2, 0x29be4, 0x29be6, 0x29be8, 0x29bea, 0x29bec, 0x29bee, 0x29bf1, 0x29bf3,
    0x29bf4, 0x29bf7, 0x29bfa, 0x29bfd, 0x29bff, 0x29c02, 0x29c06, 0x29c08, 0x29c0b, 0x29c0c,
];
