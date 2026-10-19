//! Retained area palette selection, backdrop and both palette-cache paths.
use super::{
    intro_palettes::IntroPalettes,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
    overworld_screen_palettes::LoaderOwner,
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldAreaPalettes {
    pc: u32,
    child: Option<LoaderOwner>,
    backdrop: Option<IntroPalettes>,
    returned: u32,
    sp: u16,
    complete: bool,
}
impl OverworldAreaPalettes {
    pub(crate) fn from_entry(pc: u32) -> Self {
        assert!(matches!(pc, 0x0ed5a8 | 0x0ed618 | 0x02c65f | 0x02c6eb));
        Self {
            pc,
            child: None,
            backdrop: None,
            returned: 0,
            sp: 0,
            complete: false,
        }
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
        if let Some(child) = self.child.as_mut() {
            let cycles = child.step(cpu, clock, bus);
            if child.complete() {
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp),
                    ((self.returned >> 16) as u8, self.returned as u16, self.sp)
                );
                self.pc = self.returned;
                self.child = None;
            }
            return cycles;
        }
        if let Some(child) = self.backdrop.as_mut() {
            let cycles = child.step(cpu, clock, bus);
            if child.complete() {
                self.complete = true;
                self.backdrop = None;
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 8),
            ((self.pc >> 16) as u8, self.pc as u16, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let xw = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let xmask = if xw == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match pc {
            0xed618 => {
                w.operand_then_internal(2);
                w.push(0xd61a, 2);
                0x0ed622
            }
            0xed61b => {
                w.branch(true);
                w.cpu.pc = 0xd5f9;
                self.backdrop = Some(IntroPalettes::from_backdrop_store());
                0x0ed5f9
            }
            0x2c65f | 0x2c666 | 0x2c66d => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match pc {
                        0x2c65f => 0xab6,
                        0x2c666 => 0xab8,
                        0x2c66d => 0xab7,
                        _ => unreachable!(),
                    }),
                    m,
                );
                w.load(value, m);
                pc + 3
            }
            0x2c662 | 0x2c669 | 0x2c670 | 0x2c679 | 0x2c680 | 0x2c687 | 0x2c762 => {
                w.drain(24);
                w.write(
                    match pc {
                        0x2c662 => 0x7ec20a,
                        0x2c669 => 0x7ec20b,
                        0x2c670 => 0x7ec20c,
                        0x2c679 => 0x7ec009,
                        0x2c680 => 0x7ec007,
                        0x2c687 => 0x7ec00b,
                        0x2c762 => 0x7ec011,
                        _ => unreachable!(),
                    },
                    w.cpu.a & if m == 1 { 255 } else { 65535 },
                    m,
                );
                pc + 4
            }
            0x2c674 | 0x2c6eb | 0x2c769 | 0xed622 => {
                w.width(
                    match pc {
                        0x2c674 => 0x20,
                        0x2c6eb => 0x20,
                        0x2c769 => 0x20,
                        0xed622 => 0x30,
                        _ => unreachable!(),
                    },
                    true,
                );
                pc + 2
            }
            0x2c676 | 0x2c67d | 0x2c684 | 0x2c6ef | 0x2c75e => {
                w.immediate(
                    match pc {
                        0x2c676 => 0x2,
                        0x2c67d => 0x0,
                        0x2c684 => 0x0,
                        0x2c6ef => 0x0,
                        0x2c75e => 0xf7,
                        _ => unreachable!(),
                    },
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0x2c68b => {
                w.drain(16);
                (pc & 0xff0000)
                    | u32::from(match pc {
                        0x2c68b => 0xc769,
                        _ => unreachable!(),
                    } as u16)
            }
            0x2c6ed | 0x2c714 | 0x2c76b | 0xed624 | 0xed63f | 0xed644 | 0xed64e => {
                w.drain(8 * u32::from(xw));
                w.cpu.x = match pc {
                    0x2c6ed => 0x0,
                    0x2c714 => 0x0,
                    0x2c76b => 0x0,
                    0xed624 => 0x2669,
                    0xed63f => 0x19c6,
                    0xed644 => 0x2669,
                    0xed64e => 0x2a32,
                    _ => unreachable!(),
                };
                w.flags(w.cpu.x, xw);
                pc + 1 + u32::from(xw)
            }
            0x2c6f2 | 0x2c6f6 | 0x2c6fa | 0x2c6fe | 0x2c702 | 0x2c706 | 0x2c70a | 0x2c71a
            | 0x2c722 | 0x2c72a | 0x2c732 | 0x2c73a | 0x2c742 | 0x2c74a | 0x2c752 | 0x2c771
            | 0x2c779 | 0x2c781 | 0x2c789 | 0x2c791 | 0x2c799 | 0x2c7a1 | 0x2c7a9 => {
                w.drain(24);
                w.write(
                    (match pc {
                        0x2c6f2 => 0x7ec540,
                        0x2c6f6 => 0x7ec580,
                        0x2c6fa => 0x7ec5c0,
                        0x2c6fe => 0x7ec600,
                        0x2c702 => 0x7ec640,
                        0x2c706 => 0x7ec680,
                        0x2c70a => 0x7ec6c0,
                        0x2c71a => 0x7ec500,
                        0x2c722 => 0x7ec510,
                        0x2c72a => 0x7ec520,
                        0x2c732 => 0x7ec530,
                        0x2c73a => 0x7ec6b0,
                        0x2c742 => 0x7ec6d0,
                        0x2c74a => 0x7ec6e0,
                        0x2c752 => 0x7ec6f0,
                        0x2c771 => 0x7ec500,
                        0x2c779 => 0x7ec540,
                        0x2c781 => 0x7ec580,
                        0x2c789 => 0x7ec5c0,
                        0x2c791 => 0x7ec600,
                        0x2c799 => 0x7ec640,
                        0x2c7a1 => 0x7ec680,
                        0x2c7a9 => 0x7ec6c0,
                        _ => unreachable!(),
                    } as u32)
                        + u32::from(w.cpu.x),
                    w.cpu.a & if m == 1 { 255 } else { 65535 },
                    m,
                );
                pc + 4
            }
            0x2c70e | 0x2c70f | 0x2c756 | 0x2c757 | 0x2c7ad | 0x2c7ae => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0x2c710 | 0x2c758 | 0x2c7af => {
                w.compare_x_immediate(
                    match pc {
                        0x2c710 => 0x40,
                        0x2c758 => 0x10,
                        0x2c7af => 0x40,
                        _ => unreachable!(),
                    },
                    xw,
                );
                pc + 1 + u32::from(xw)
            }
            0x2c712 | 0x2c75a | 0x2c7b1 | 0xed63d => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match pc {
                        0x2c712 => 0x2c6f2,
                        0x2c75a => 0x2c716,
                        0x2c7b1 => 0x2c76d,
                        0xed63d => 0xed651,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0x2c716 | 0x2c71e | 0x2c726 | 0x2c72e | 0x2c736 | 0x2c73e | 0x2c746 | 0x2c74e
            | 0x2c76d | 0x2c775 | 0x2c77d | 0x2c785 | 0x2c78d | 0x2c795 | 0x2c79d | 0x2c7a5
            | 0xed5ae | 0xed5b7 | 0xed5c0 | 0xed5cd | 0xed5d6 => {
                w.drain(24);
                let value = w.read(
                    (match pc {
                        0x2c716 => 0x7ec300,
                        0x2c71e => 0x7ec310,
                        0x2c726 => 0x7ec320,
                        0x2c72e => 0x7ec330,
                        0x2c736 => 0x7ec4b0,
                        0x2c73e => 0x7ec4d0,
                        0x2c746 => 0x7ec4e0,
                        0x2c74e => 0x7ec4f0,
                        0x2c76d => 0x7ec300,
                        0x2c775 => 0x7ec340,
                        0x2c77d => 0x7ec380,
                        0x2c785 => 0x7ec3c0,
                        0x2c78d => 0x7ec400,
                        0x2c795 => 0x7ec440,
                        0x2c79d => 0x7ec480,
                        0x2c7a5 => 0x7ec4c0,
                        0xed5ae => 0xed504,
                        0xed5b7 => 0xed505,
                        0xed5c0 => 0xed506,
                        0xed5cd => 0xed580,
                        0xed5d6 => 0xed581,
                        _ => unreachable!(),
                    } as u32)
                        + u32::from(w.cpu.x),
                    m,
                );
                w.load(value, m);
                pc + 4
            }
            0x2c75c | 0x2c7b3 => {
                w.width(
                    match pc {
                        0x2c75c => 0x20,
                        0x2c7b3 => 0x20,
                        _ => unreachable!(),
                    },
                    false,
                );
                pc + 2
            }
            0x2c760 => {
                w.dp_store(
                    match pc {
                        0x2c760 => 0x95,
                        _ => unreachable!(),
                    },
                    w.cpu.a & if m == 1 { 255 } else { 65535 },
                    m,
                );
                pc + 2
            }
            0x2c766 | 0x2c7b5 => {
                w.dp_increment(
                    match pc {
                        0x2c766 => 0x15,
                        0x2c7b5 => 0x15,
                        _ => unreachable!(),
                    },
                    m,
                    true,
                );
                pc + 2
            }
            0x2c768 | 0x2c7b7 | 0xed652 => {
                w.rts();
                if pc != 0xed652 {
                    self.complete = true;
                }
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0xed5a8 | 0xed5a9 | 0xed5cb => {
                w.shift_left(m);
                pc + 1
            }
            0xed5aa | 0xed5cc => {
                w.drain(6);
                w.cpu.x = w.cpu.a & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xed5ab => {
                w.drain(16);
                w.write(
                    w.absolute(match pc {
                        0xed5ab => 0xaa9,
                        _ => unreachable!(),
                    }),
                    0,
                    m,
                );
                pc + 3
            }
            0xed5b2 | 0xed5bb | 0xed5c4 | 0xed5d1 | 0xed5da => {
                if w.branch(w.cpu.p & NEGATIVE != 0) {
                    match pc {
                        0xed5b2 => 0xed5b7,
                        0xed5bb => 0xed5c0,
                        0xed5c4 => 0xed5c9,
                        0xed5d1 => 0xed5d6,
                        0xed5da => 0xed5df,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xed5b4 | 0xed5bd | 0xed5c6 | 0xed5d3 | 0xed5dc => {
                w.drain(16);
                w.write(
                    w.absolute(match pc {
                        0xed5b4 => 0xab4,
                        0xed5bd => 0xab5,
                        0xed5c6 => 0xab8,
                        0xed5d3 => 0xaad,
                        0xed5dc => 0xaae,
                        _ => unreachable!(),
                    }),
                    w.cpu.a & if m == 1 { 255 } else { 65535 },
                    m,
                );
                pc + 3
            }
            0xed5c9 | 0xed627 | 0xed62e | 0xed647 => {
                let value = w.dp_read(
                    match pc {
                        0xed5c9 => 0x0,
                        0xed627 => 0x8a,
                        0xed62e => 0xa0,
                        0xed647 => 0x8a,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load(value, m);
                pc + 2
            }
            0xed5df | 0xed5e3 | 0xed5e7 | 0xed5eb | 0xed5ef => {
                let target: u32 = match pc {
                    0xed5df => 0x1beee8,
                    0xed5e3 => 0x1bef0c,
                    0xed5e7 => 0x1beea8,
                    0xed5eb => 0x1becc5,
                    0xed5ef => 0x1bece4,
                    _ => unreachable!(),
                };
                self.returned = pc + 4;
                self.sp = w.cpu.sp;
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push((pc + 3) as u16, 2);
                self.child = Some(LoaderOwner::new(target as u16));
                target
            }
            0xed5f3 => {
                w.rtl();
                self.complete = true;
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0xed629 | 0xed630 | 0xed635 | 0xed63a => {
                w.compare_immediate(
                    match pc {
                        0xed629 => 0x80,
                        0xed630 => 0x183,
                        0xed635 => 0x182,
                        0xed63a => 0x180,
                        _ => unreachable!(),
                    },
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0xed62c => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    match pc {
                        0xed62c => 0xed644,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xed633 | 0xed638 | 0xed64c => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    match pc {
                        0xed633 => 0xed63f,
                        0xed638 => 0xed63f,
                        0xed64c => 0xed651,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xed642 => {
                if w.branch(true) {
                    match pc {
                        0xed642 => 0xed651,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xed649 => {
                w.mask(
                    match pc {
                        0xed649 => 0x40,
                        _ => unreachable!(),
                    },
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0xed651 => {
                w.drain(6);
                w.load(w.cpu.x, m);
                pc + 1
            }
            _ => unreachable!("area palettes PC {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}
