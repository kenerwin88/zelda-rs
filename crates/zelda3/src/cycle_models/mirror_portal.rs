//! Mirror portal replacement, dynamic allocation and sprite initialization.
//! Nested far returns preserve the parent's live stack, registers and clock.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MirrorPortal {
    pc: u32,
    calls: Vec<u32>,
    complete: bool,
}
impl Default for MirrorPortal {
    fn default() -> Self {
        Self {
            pc: 0x09af89,
            calls: Vec::new(),
            complete: false,
        }
    }
}
impl MirrorPortal {
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
            (
                (u32::from(cpu.bank) << 16) | u32::from(cpu.pc),
                cpu.d,
                cpu.p & 0x38
            ),
            (self.pc, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let m = 1;
        let mask = 255;
        let xw = 1;
        let xmask = 255;
        w.fetch();
        self.pc = match pc {
            0x09af89 | 0x0db81d | 0x0db85d => {
                w.drain(6);
                w.push(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0x09af8a | 0x0db81e | 0x0db85e => {
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                pc + 1
            }
            0x09af8b | 0x09afd4 | 0x0db81f | 0x0db85a | 0x0db85f | 0x0db86e => {
                w.drain(12);
                let value = w.pull(1);
                w.cpu.db = value as u8;
                w.flags(value, 1);
                pc + 1
            }
            0x09af8c => {
                w.drain(8 * u32::from(xw));
                let value = match pc {
                    0x09af8c => 0xf,
                    _ => unreachable!(),
                };
                w.cpu.x = value;
                w.flags(value, xw);
                pc + 1 + u32::from(xw)
            }
            0x09af8e | 0x09af93 | 0x1df674 | 0x1df679 | 0x1df67e | 0x1df683 | 0x1df688
            | 0x1df68d | 0x1df692 | 0x1df697 | 0x1df69c | 0x1df6b9 | 0x1df6bf => {
                w.drain(16);
                let base: u16 = match pc {
                    0x09af8e => 0xdd0,
                    0x09af93 => 0xe20,
                    0x1df674 => 0xd10,
                    0x1df679 => 0xd30,
                    0x1df67e => 0xd00,
                    0x1df683 => 0xd20,
                    0x1df688 => 0xf70,
                    0x1df68d => 0xb08,
                    0x1df692 => 0xb10,
                    0x1df697 => 0xb18,
                    0x1df69c => 0xb20,
                    0x1df6b9 => 0xf20,
                    0x1df6bf => 0xde0,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.x)), m);
                w.load(value, m);
                pc + 3
            }
            0x09af91 | 0x1df663 | 0x0db852 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    match pc {
                        0x09af91 => 0x9af9d,
                        0x1df663 => 0x1df66b,
                        0x0db852 => 0xdb857,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0x09af96 => {
                w.drain(8 * u32::from(m));
                let value = match pc {
                    0x09af96 => 0x6c,
                    _ => unreachable!(),
                };
                let old = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old >= value);
                w.flags(old.wrapping_sub(value) & mask, m);
                pc + 1 + u32::from(m)
            }
            0x09af98 | 0x1df6a9 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match pc {
                        0x09af98 => 0x9af9d,
                        0x1df6a9 => 0x1df6b3,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0x09af9a | 0x0db871 | 0x0db874 | 0x0db877 | 0x0db87a | 0x0db87d | 0x0db880
            | 0x0db883 | 0x0db886 | 0x0db889 | 0x0db88c | 0x0db88f | 0x0db892 | 0x0db895
            | 0x0db898 | 0x0db89b | 0x0db89e | 0x0db8a1 | 0x0db8a4 | 0x0db8a7 | 0x0db8aa
            | 0x0db8ad | 0x0db8b0 | 0x0db8b3 | 0x0db8b6 | 0x0db8b9 | 0x0db8bc | 0x0db8bf
            | 0x0db8c2 | 0x0db8c5 | 0x0db8c8 | 0x0db8cb | 0x0db8ce | 0x0db8d1 | 0x0db8d4
            | 0x0db8d7 => {
                w.drain(16);
                w.drain(6);
                let base: u16 = match pc {
                    0x09af9a => 0xdd0,
                    0x0db871 => 0xf00,
                    0x0db874 => 0xe90,
                    0x0db877 => 0xd50,
                    0x0db87a => 0xd40,
                    0x0db87d => 0xf80,
                    0x0db880 => 0xd70,
                    0x0db883 => 0xd60,
                    0x0db886 => 0xf90,
                    0x0db889 => 0xd80,
                    0x0db88c => 0xdc0,
                    0x0db88f => 0xde0,
                    0x0db892 => 0xdf0,
                    0x0db895 => 0xe00,
                    0x0db898 => 0xe10,
                    0x0db89b => 0xf10,
                    0x0db89e => 0xeb0,
                    0x0db8a1 => 0xec0,
                    0x0db8a4 => 0xed0,
                    0x0db8a7 => 0xef0,
                    0x0db8aa => 0xe70,
                    0x0db8ad => 0xf70,
                    0x0db8b0 => 0xe50,
                    0x0db8b3 => 0xea0,
                    0x0db8b6 => 0xf40,
                    0x0db8b9 => 0xf30,
                    0x0db8bc => 0xd90,
                    0x0db8bf => 0xda0,
                    0x0db8c2 => 0xdb0,
                    0x0db8c5 => 0xbb0,
                    0x0db8c8 => 0xe80,
                    0x0db8cb => 0xba0,
                    0x0db8ce => 0xb89,
                    0x0db8d1 => 0xf50,
                    0x0db8d4 => 0xb58,
                    0x0db8d7 => 0xce2,
                    _ => unreachable!(),
                };
                w.write(w.absolute(base.wrapping_add(w.cpu.x)), 0, m);
                pc + 3
            }
            0x09af9d => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0x09af9e | 0x09afa6 | 0x1df666 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    match pc {
                        0x09af9e => 0x9af8e,
                        0x09afa6 => 0x9afaa,
                        0x1df666 => 0x1df660,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0x09afa0 | 0x09afcb | 0x1df66f | 0x1df6ae | 0x1df6b3 | 0x1df6c5 | 0x0db8da => {
                w.drain(8 * u32::from(m));
                let value = match pc {
                    0x09afa0 => 0x6c,
                    0x09afcb => 0x0,
                    0x1df66f => 0x9,
                    0x1df6ae => 0xff,
                    0x1df6b3 => 0xff,
                    0x1df6c5 => 0x0,
                    0x0db8da => 0x0,
                    _ => unreachable!(),
                };
                w.load(value, m);
                pc + 1 + u32::from(m)
            }
            0x09afa2 | 0x1df6a3 | 0x0db818 => {
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push((pc + 3) as u16, 2);
                self.calls.push(pc + 4);
                match pc {
                    0x09afa2 => 0x1df65d,
                    0x1df6a3 => 0xdb818,
                    0x0db818 => 0xdb871,
                    _ => unreachable!(),
                }
            }
            0x09afa8 | 0x1df65d => {
                w.drain(8 * u32::from(xw));
                let value = match pc {
                    0x09afa8 => 0x0,
                    0x1df65d => 0xf,
                    _ => unreachable!(),
                };
                w.cpu.y = value;
                w.flags(value, xw);
                pc + 1 + u32::from(xw)
            }
            0x09afaa | 0x09afb1 | 0x09afb8 | 0x09afc2 => {
                w.drain(24);
                let value = w.read(
                    match pc {
                        0x09afaa => 0x1abf,
                        0x09afb1 => 0x1acf,
                        0x09afb8 => 0x1adf,
                        0x09afc2 => 0x1aef,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load(value, m);
                pc + 4
            }
            0x09afae | 0x09afb5 | 0x09afbf | 0x09afc8 | 0x09afcd | 0x09afd1 | 0x1df66c
            | 0x1df671 | 0x1df6bc | 0x1df6c2 | 0x1df6c7 | 0x1df6ca => {
                w.drain(16);
                w.drain(6);
                let base: u16 = match pc {
                    0x09afae => 0xd10,
                    0x09afb5 => 0xd30,
                    0x09afbf => 0xd00,
                    0x09afc8 => 0xd20,
                    0x09afcd => 0xf20,
                    0x09afd1 => 0xba0,
                    0x1df66c => 0xe20,
                    0x1df671 => 0xdd0,
                    0x1df6bc => 0xf20,
                    0x1df6c2 => 0xde0,
                    0x1df6c7 => 0xcba,
                    0x1df6ca => 0xe30,
                    _ => unreachable!(),
                };
                w.write(w.absolute(base.wrapping_add(w.cpu.y)), w.cpu.a & mask, m);
                pc + 3
            }
            0x09afbc => {
                w.drain(6);
                w.cpu.p &= !CARRY;
                pc + 1
            }
            0x09afbd | 0x09afc6 => {
                w.drain(8 * u32::from(m));
                let value = match pc {
                    0x09afbd => 0x8,
                    0x09afc6 => 0x0,
                    _ => unreachable!(),
                };
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                pc + 1 + u32::from(m)
            }
            0x09afd0 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_add(1) & mask, m);
                pc + 1
            }
            0x09afd5 | 0x1df66a | 0x1df6ce | 0x0db870 | 0x0db8f0 => {
                w.rtl();
                let target = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                if let Some(returned) = self.calls.pop() {
                    assert_eq!(target, returned);
                } else {
                    self.complete = true;
                }
                target
            }
            0x1df65f => {
                w.drain(6);
                w.push(w.cpu.a & mask, m);
                pc + 1
            }
            0x1df660 | 0x0db823 | 0x0db829 | 0x0db82f | 0x0db835 | 0x0db83b | 0x0db841
            | 0x0db847 | 0x0db863 => {
                w.drain(16);
                let base: u16 = match pc {
                    0x1df660 => 0xdd0,
                    0x0db823 => 0xb080,
                    0x0db829 => 0xb173,
                    0x0db82f => 0xb44c,
                    0x0db835 => 0xb632,
                    0x0db83b => 0xb725,
                    0x0db841 => 0xb266,
                    0x0db847 => 0xb53f,
                    0x0db863 => 0xb359,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.y)), m);
                w.load(value, m);
                pc + 3
            }
            0x1df665 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & xmask;
                w.flags(w.cpu.y, xw);
                pc + 1
            }
            0x1df668 | 0x1df66b => {
                w.drain(12);
                let value = w.pull(m);
                w.load(value, m);
                pc + 1
            }
            0x1df669 | 0x1df6cd => {
                w.drain(6);
                w.load(w.cpu.y, m);
                pc + 1
            }
            0x1df677 | 0x1df67c | 0x1df681 | 0x1df686 | 0x1df68b | 0x1df690 | 0x1df695
            | 0x1df69a | 0x1df69f => {
                w.dp_store(
                    match pc {
                        0x1df677 => 0x0,
                        0x1df67c => 0x1,
                        0x1df681 => 0x2,
                        0x1df686 => 0x3,
                        0x1df68b => 0x4,
                        0x1df690 => 0x5,
                        0x1df695 => 0x6,
                        0x1df69a => 0x7,
                        0x1df69f => 0x8,
                        _ => unreachable!(),
                    },
                    w.cpu.a & mask,
                    m,
                );
                pc + 2
            }
            0x1df6a1 => {
                w.drain(6);
                w.push(w.cpu.x, xw);
                pc + 1
            }
            0x1df6a2 => {
                w.drain(6);
                w.cpu.x = w.cpu.y;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0x1df6a7 => {
                let value = w.dp_read(
                    match pc {
                        0x1df6a7 => 0x1b,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load(value, m);
                pc + 2
            }
            0x1df6ab => {
                w.drain(6);
                w.load(w.cpu.x, m);
                pc + 1
            }
            0x1df6ac => {
                w.shift_left(m);
                pc + 1
            }
            0x1df6ad => {
                w.drain(6);
                w.cpu.x = w.cpu.a & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0x1df6b0 | 0x1df6b5 | 0x0db826 | 0x0db82c | 0x0db832 | 0x0db838 | 0x0db83e
            | 0x0db844 | 0x0db84a | 0x0db857 | 0x0db866 | 0x0db86b => {
                w.drain(16);
                w.drain(6);
                let base: u16 = match pc {
                    0x1df6b0 => 0xbc1,
                    0x1df6b5 => 0xbc0,
                    0x0db826 => 0xe40,
                    0x0db82c => 0xe50,
                    0x0db832 => 0xf60,
                    0x0db838 => 0xbe0,
                    0x0db83e => 0xcaa,
                    0x0db844 => 0xcd2,
                    0x0db84a => 0xb6b,
                    0x0db857 => 0xc9a,
                    0x0db866 => 0xe60,
                    0x0db86b => 0xf50,
                    _ => unreachable!(),
                };
                w.write(w.absolute(base.wrapping_add(w.cpu.x)), w.cpu.a & mask, m);
                pc + 3
            }
            0x1df6b8 => {
                w.drain(12);
                let value = w.pull(xw);
                w.cpu.x = value;
                w.flags(value, xw);
                pc + 1
            }
            0x0db81c | 0x0db85c => {
                w.drain(6);
                w.push(w.cpu.y, xw);
                pc + 1
            }
            0x0db820 | 0x0db860 => {
                w.drain(16);
                let base: u16 = match pc {
                    0x0db820 => 0xe20,
                    0x0db860 => 0xe20,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.x)), xw);
                w.cpu.y = value;
                w.flags(value, xw);
                pc + 3
            }
            0x0db84d | 0x0db854 => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match pc {
                        0x0db84d => 0x40a,
                        0x0db854 => 0x48e,
                        _ => unreachable!(),
                    }),
                    m,
                );
                w.load(value, m);
                pc + 3
            }
            0x0db850 => {
                let value = w.dp_read(
                    match pc {
                        0x0db850 => 0x1b,
                        _ => unreachable!(),
                    },
                    xw,
                );
                w.cpu.y = value;
                w.flags(value, xw);
                pc + 2
            }
            0x0db85b | 0x0db86f => {
                w.drain(12);
                let value = w.pull(xw);
                w.cpu.y = value;
                w.flags(value, xw);
                pc + 1
            }
            0x0db869 => {
                w.drain(8 * u32::from(m));
                let value = match pc {
                    0x0db869 => 0xf,
                    _ => unreachable!(),
                };
                w.load(w.cpu.a & value & mask, m);
                pc + 1 + u32::from(m)
            }
            0x0db8dc | 0x0db8e0 | 0x0db8e4 | 0x0db8e8 | 0x0db8ec => {
                w.drain(24);
                w.write(
                    (match pc {
                        0x0db8dc => 0x7ffa1c,
                        0x0db8e0 => 0x7ffa2c,
                        0x0db8e4 => 0x7ffa3c,
                        0x0db8e8 => 0x7ffa4c,
                        0x0db8ec => 0x7ff9c2,
                        _ => unreachable!(),
                    } as u32)
                        + u32::from(w.cpu.x),
                    w.cpu.a & mask,
                    m,
                );
                pc + 4
            }
            _ => unreachable!("mirror portal PC {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}
