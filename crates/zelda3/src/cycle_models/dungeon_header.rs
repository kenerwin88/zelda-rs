//! Room-header parsing and adjacent-door merging retain live source-data
//! pointers, flags, loop indices and nested returns at instruction boundaries.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonHeader {
    pc: u16,
    depth: u8,
    complete: bool,
}
impl Default for DungeonHeader {
    fn default() -> Self {
        Self {
            pc: 0xb564,
            depth: 0,
            complete: false,
        }
    }
}
impl DungeonHeader {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x01b564
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
        assert_eq!((cpu.bank, cpu.pc, cpu.db, cpu.d), (1, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match self.pc {
            0xb56d | 0xb5e2 | 0xb5e4 | 0xb618 | 0xb623 | 0xb6b0 | 0xb756 => {
                w.width(
                    if matches!(self.pc, 0xb56d | 0xb6b0 | 0xb756) {
                        0x30
                    } else if self.pc == 0xb5e4 {
                        0x10
                    } else {
                        0x20
                    },
                    matches!(self.pc, 0xb56d | 0xb5e4 | 0xb618 | 0xb6b0),
                );
                self.pc + 2
            }
            0xb723 | 0xb732 | 0xb741 | 0xb753 | 0xb75b => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                if self.pc == 0xb75b {
                    0xb7ef
                } else {
                    0xb759
                }
            }
            0xb758 | 0xb7ee => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0xb7eb => {
                w.drain(16);
                0xb761
            }
            0xb5e6 | 0xb6b2 | 0xb720 | 0xb72f | 0xb73e | 0xb750 => {
                w.immediate(
                    match self.pc {
                        0xb5e6 => 4,
                        0xb6b2 => 1,
                        0xb720 => 0x24,
                        0xb72f => 0x18,
                        0xb73e => 12,
                        _ => 0,
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0xb5f1 | 0xb75e => {
                w.drain(16);
                w.cpu.y = 0;
                w.flags(0, 2);
                self.pc + 3
            }
            0xb6fd | 0xb79d | 0xb814 => {
                w.drain(16);
                w.cpu.x = 0;
                w.flags(0, 2);
                self.pc + 3
            }
            0xb571 | 0xb578 | 0xb59b | 0xb5ad | 0xb5c5 | 0xb5d2 | 0xb5f6 | 0xb601 | 0xb612
            | 0xb61d | 0xb662 | 0xb669 | 0xb672 | 0xb67d | 0xb688 | 0xb6d9 | 0xb6e9 | 0xb6f7
            | 0xb718 | 0xb72a | 0xb76d | 0xb7a3 | 0xb7b6 | 0xb80b | 0xb822 => {
                let v = match self.pc {
                    0xb571 | 0xb61d | 0xb76d | 0xb7a3 => 255,
                    0xb578 | 0xb5ad | 0xb5c5 | 0xb5d2 => 0xfe00,
                    0xb59b | 0xb6f7 | 0xb718 | 0xb72a => 15,
                    0xb5f6 => 0xe0,
                    0xb601 => 0x1c,
                    0xb612 => 1,
                    0xb662 | 0xb688 => 3,
                    0xb669 => 12,
                    0xb672 => 0x30,
                    0xb67d => 0xc0,
                    0xb6d9 | 0xb80b => 0xf000,
                    0xb6e9 => 0xff0,
                    _ => 0xff00,
                };
                w.mask(v, m);
                self.pc + 1 + u16::from(m)
            }
            0xb582 | 0xb587 | 0xb58c | 0xb59f | 0xb706 | 0xb71b | 0xb74b | 0xb764 | 0xb7b9
            | 0xb7be | 0xb7c3 | 0xb81d | 0xb825 | 0xb82a => {
                w.compare_immediate(
                    match self.pc {
                        0xb582 => 21,
                        0xb587 => 18,
                        0xb58c => 6,
                        0xb59f => 2,
                        0xb71b => 15,
                        0xb74b => 0x140,
                        0xb7b9 => 0x3000,
                        0xb7be => 0x4400,
                        0xb7c3 => 0x1800,
                        0xb825 => 0x4000,
                        0xb82a => 0x200,
                        _ => 65535,
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0xb7ac => {
                w.compare_x_immediate(16, 2);
                self.pc + 3
            }
            0xb7e6 => {
                w.drain(16);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= 16);
                w.flags(w.cpu.y.wrapping_sub(16), 2);
                self.pc + 3
            }
            0xb593 | 0xb5a8 | 0xb5b5 | 0xb5c1 | 0xb5cd | 0xb64a | 0xb6be | 0xb746 | 0xb7f3 => {
                w.clear_carry();
                self.pc + 1
            }
            0xb737 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0xb594 | 0xb5b6 | 0xb5c2 | 0xb64b | 0xb747 => {
                w.drain(8 * u32::from(m));
                let v = match self.pc {
                    0xb64b => 0x40,
                    0xb747 => 16,
                    _ => 0x20,
                };
                if m == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                self.pc + 1 + u16::from(m)
            }
            0xb738 => {
                w.drain(16);
                w.sbc(16, 2);
                self.pc + 3
            }
            0xb59e | 0xb5bb | 0xb5bc | 0xb603 | 0xb604 | 0xb66b | 0xb66c | 0xb674..=0xb677 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | ((w.cpu.a & 1) as u8);
                w.load((w.cpu.a & mask) >> 1, m);
                self.pc + 1
            }
            0xb5a4
            | 0xb5bd
            | 0xb5da
            | 0xb5f8
            | 0xb620
            | 0xb621
            | 0xb67f
            | 0xb6bd
            | 0xb6d3
            | 0xb6ec..=0xb6ef
            | 0xb7f2
            | 0xb805 => {
                w.shift_left(m);
                self.pc + 1
            }
            0xb5f9..=0xb5fb | 0xb680 | 0xb681 => {
                w.drain(6);
                let a = w.cpu.a & mask;
                let carry = u16::from(w.cpu.p & CARRY);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a & if m == 1 { 128 } else { 32768 } != 0);
                w.load(((a << 1) | carry) & mask, m);
                self.pc + 1
            }
            0xb5a5 | 0xb5be | 0xb5db | 0xb622 | 0xb6c4 | 0xb6d4 | 0xb717 | 0xb729 | 0xb73b
            | 0xb74a | 0xb7f6 | 0xb806 => {
                w.drain(6);
                w.cpu.x = w.cpu.a;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xb700 | 0xb817 => {
                w.drain(6);
                w.cpu.y = w.cpu.x;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xb7f1 => {
                w.drain(6);
                w.load(w.cpu.x, m);
                self.pc + 1
            }
            0xb61a | 0xb641 | 0xb647 | 0xb650 | 0xb655 | 0xb65a | 0xb65f | 0xb685 | 0xb68d
            | 0xb694 | 0xb69b | 0xb6a2 | 0xb6a9 | 0xb70e | 0xb70f | 0xb7e4 | 0xb7e5 | 0xb838
            | 0xb839 => {
                w.increment_y();
                self.pc + 1
            }
            0xb710 | 0xb711 | 0xb775 | 0xb776 | 0xb77c | 0xb77d | 0xb783 | 0xb784 | 0xb78a
            | 0xb78b | 0xb791 | 0xb792 | 0xb7aa | 0xb7ab | 0xb83a | 0xb83b => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xb716 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1), 2);
                self.pc + 1
            }
            0xb728 => {
                w.increment_a(2);
                self.pc + 1
            }
            0xb6df | 0xb80e => {
                w.drain(16);
                w.load(w.cpu.a | 0x0f00, 2);
                self.pc + 3
            }
            0xb564 | 0xb567 | 0xb56a | 0xb6b8 | 0xb7ce => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0xb564 => 0x642,
                        0xb567 => 0x646,
                        0xb56a => 0x641,
                        0xb6b8 => 0x4ba,
                        _ => 0x468,
                    }),
                    0,
                    m,
                );
                self.pc + 3
            }
            0xb57b | 0xb5b0 | 0xb5c8 | 0xb5d5 | 0xb5fc | 0xb605 | 0xb629 | 0xb630 | 0xb637
            | 0xb63e | 0xb644 | 0xb64d | 0xb664 | 0xb66d | 0xb678 | 0xb682 | 0xb68a | 0xb6b5
            | 0xb6c1 | 0xb6dc | 0xb6e2 | 0xb6f0 | 0xb6fa | 0xb7e1 | 0xb811 | 0xb835 => {
                let a = match self.pc {
                    0xb57b | 0xb5b0 | 0xb5c8 => 0x62c,
                    0xb5d5 => 0x62e,
                    0xb5fc => 0x414,
                    0xb605 => 0x46c,
                    0xb629 => 0xab6,
                    0xb630 => 0xaac,
                    0xb637 => 0xaad,
                    0xb63e => 0xaae,
                    0xb644 => 0xaa2,
                    0xb64d => 0xaa3,
                    0xb664 => 0x63c,
                    0xb66d => 0x63d,
                    0xb678 => 0x63e,
                    0xb682 => 0x63f,
                    0xb68a => 0x640,
                    0xb6b5 => 0x468,
                    0xb6c1 => 0x110,
                    0xb6dc => 0x400,
                    0xb6e2 | 0xb7e1 => 0x68c,
                    0xb6f0 => 0x402,
                    0xb6fa => 0x408,
                    _ => 0x1100,
                };
                w.drain(16);
                w.write(w.absolute(a), w.cpu.a & mask, m);
                self.pc + 3
            }
            0xb5ea | 0xb7d3 | 0xb7db | 0xb82f => {
                w.drain(16);
                let v = w.read(
                    w.absolute(match self.pc {
                        0xb5ea => 0x414,
                        0xb7db => 0x68c,
                        _ => 0x1100,
                    }),
                    m,
                );
                w.load(v, m);
                self.pc + 3
            }
            0xb5ed | 0xb60c | 0xb614 | 0xb690 | 0xb697 | 0xb69e | 0xb6a5 | 0xb6ac => {
                w.drain(24);
                w.write(
                    match self.pc {
                        0xb5ed => 0x7ec208,
                        0xb60c => 0x7ec006,
                        0xb614 => 0x7ec005,
                        0xb690 => 0x7ec000,
                        0xb697 => 0x7ec001,
                        0xb69e => 0x7ec002,
                        0xb6a5 => 0x7ec003,
                        _ => 0x7ec004,
                    },
                    w.cpu.a & mask,
                    m,
                );
                self.pc + 4
            }
            0xb608 => {
                w.drain(24);
                let v = w.read(0x7ec005, m);
                w.load(v, m);
                self.pc + 4
            }
            0xb56f | 0xb576 | 0xb57e | 0xb591 | 0xb599 | 0xb5a6 | 0xb5b3 | 0xb5bf | 0xb5cb
            | 0xb5d8 | 0xb6bb | 0xb6bf | 0xb6d1 | 0xb714 | 0xb726 | 0xb735 | 0xb744 | 0xb76b
            | 0xb7a6 | 0xb7c8 | 0xb7ca | 0xb7f4 | 0xb803 => {
                let a = match self.pc {
                    0xb56f => 0x11,
                    0xb576 | 0xb591 | 0xb5a6 | 0xb5bf => 0xe2,
                    0xb57e | 0xb5b3 | 0xb5cb => 0xe8,
                    0xb599 => 0x67,
                    0xb76b => 4,
                    0xb7a6 => 0,
                    0xb7c8 | 0xb7f4 | 0xb803 => 14,
                    0xb7ca => 0xa2,
                    _ => 0xa0,
                };
                let v = w.dp_read(a, m);
                if self.pc == 0xb76b {
                    w.cpu.x = v;
                    w.flags(v, 2);
                } else if matches!(self.pc, 0xb6bf | 0xb7f4) {
                    w.adc(v);
                } else if matches!(self.pc, 0xb7a6 | 0xb7ca) {
                    let a = w.cpu.a;
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                    w.flags(a.wrapping_sub(v), 2);
                } else {
                    w.load(v, m);
                }
                self.pc + 2
            }
            0xb5e0 | 0xb5e8 | 0xb653 | 0xb658 | 0xb65d | 0xb6c9 | 0xb6cf | 0xb759 | 0xb769
            | 0xb79b | 0xb7ef | 0xb7fb | 0xb801 => {
                let a = match self.pc {
                    0xb5e0 => 13,
                    0xb5e8 => 15,
                    0xb653 => 0xad,
                    0xb658 => 0xae,
                    0xb65d => 0xaf,
                    0xb6c9 | 0xb7fb => 0xb8,
                    0xb6cf | 0xb801 => 0xb7,
                    0xb759 => 4,
                    0xb769 => 2,
                    0xb79b => 0,
                    _ => 14,
                };
                w.dp_store(
                    a,
                    if self.pc == 0xb7ef {
                        w.cpu.x
                    } else {
                        w.cpu.a & mask
                    },
                    m,
                );
                self.pc + 2
            }
            0xb5a9 | 0xb5ce | 0xb5dc | 0xb625 | 0xb62c | 0xb633 | 0xb63a | 0xb6c5 | 0xb6cb
            | 0xb6d5 | 0xb6e5 | 0xb6f3 | 0xb7f7 | 0xb7fd | 0xb807 => {
                let a = match self.pc {
                    0xb5a9 | 0xb5ce => 0x01b560,
                    0xb5dc => 0x04f502,
                    0xb625 => 0x0ed460,
                    0xb62c => 0x0ed461,
                    0xb633 => 0x0ed462,
                    0xb63a => 0x0ed463,
                    0xb6c5 | 0xb7f7 => 0x1f83c1,
                    0xb6cb | 0xb7fd => 0x1f83c0,
                    _ => 0x7ef000,
                };
                w.drain(24);
                let v = w.read(a + u32::from(w.cpu.x), m);
                if matches!(self.pc, 0xb5a9 | 0xb5ce) {
                    w.adc(v);
                } else {
                    w.load(v, m);
                }
                self.pc + 4
            }
            0xb5f4 | 0xb5ff | 0xb610 | 0xb61b | 0xb642 | 0xb648 | 0xb651 | 0xb656 | 0xb65b
            | 0xb660 | 0xb667 | 0xb670 | 0xb67b | 0xb686 | 0xb68e | 0xb695 | 0xb69c | 0xb6a3
            | 0xb6aa | 0xb704 | 0xb818 => {
                let a = w.indirect_y_address(if matches!(self.pc, 0xb704 | 0xb818) {
                    0xb7
                } else {
                    13
                });
                let v = w.read(a, m);
                w.load(v, m);
                self.pc + 2
            }
            0xb701 | 0xb70b | 0xb81a => {
                w.operand_then_internal(2);
                w.write(
                    w.absolute(if self.pc == 0xb81a { 0x1110 } else { 0x19a0 })
                        + u32::from(w.cpu.x),
                    if self.pc == 0xb701 { 0 } else { w.cpu.a },
                    2,
                );
                self.pc + 3
            }
            0xb761 | 0xb770 | 0xb777 | 0xb77e | 0xb785 | 0xb78c | 0xb793 | 0xb798 | 0xb7a0
            | 0xb7b3 | 0xb7d6 | 0xb7de | 0xb832 => {
                let a = match self.pc {
                    0xb761 => 0x1110,
                    0xb798 => 0x9ad2,
                    0xb7a0 | 0xb7b3 => 0x19a0,
                    0xb7d6 | 0xb7de | 0xb832 => 0x98c0,
                    _ => 0x9aa2,
                };
                let i = if matches!(self.pc, 0xb761 | 0xb7d6) {
                    w.cpu.y
                } else {
                    w.cpu.x
                };
                w.operand_then_internal(2);
                let v = w.read(w.absolute(a) + u32::from(i), 2);
                if matches!(self.pc, 0xb770 | 0xb777 | 0xb77e | 0xb785 | 0xb78c | 0xb793) {
                    let a = w.cpu.a;
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                    w.flags(a.wrapping_sub(v), 2);
                } else if self.pc == 0xb7d6 {
                    w.load(w.cpu.a & v, 2);
                } else if matches!(self.pc, 0xb7de | 0xb832) {
                    w.load(w.cpu.a | v, 2);
                } else {
                    w.load(v, 2);
                }
                self.pc + 3
            }
            0xb574 | 0xb580 | 0xb585 | 0xb58a | 0xb58f | 0xb597 | 0xb5a2 | 0xb5b9 | 0xb709
            | 0xb712 | 0xb71e | 0xb72d | 0xb73c | 0xb74e | 0xb767 | 0xb773 | 0xb77a | 0xb781
            | 0xb788 | 0xb78f | 0xb796 | 0xb7a8 | 0xb7af | 0xb7b1 | 0xb7bc | 0xb7c1 | 0xb7c6
            | 0xb7cc | 0xb7d1 | 0xb7d9 | 0xb7e9 | 0xb820 | 0xb828 | 0xb82d | 0xb83c => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let (t, a) = match self.pc {
                    0xb574 => (!z, 0xb582),
                    0xb580 | 0xb5b9 => (true, 0xb5d2),
                    0xb585 => (z, 0xb591),
                    0xb58a => (c, 0xb599),
                    0xb58f => (!c, 0xb599),
                    0xb597 => (true, 0xb5ad),
                    0xb5a2 => (c, 0xb5bb),
                    0xb709 => (z, 0xb714),
                    0xb712 => (true, 0xb701),
                    0xb71e => (z, 0xb726),
                    0xb72d => (z, 0xb735),
                    0xb73c => (w.cpu.p & NEGATIVE != 0, 0xb744),
                    0xb74e => (c, 0xb756),
                    0xb767 => (z, 0xb758),
                    0xb773 | 0xb77a | 0xb781 | 0xb788 | 0xb78f => (z, 0xb798),
                    0xb796 | 0xb7c6 | 0xb7cc => {
                        (!z, if self.pc == 0xb7c6 { 0xb7d3 } else { 0xb7e4 })
                    }
                    0xb7a8 => (z, 0xb7b3),
                    0xb7af => (!z, 0xb7a0),
                    0xb7b1 => (true, 0xb7e4),
                    0xb7bc | 0xb7d9 => (z, 0xb7e4),
                    0xb7c1 => (z, 0xb7c8),
                    0xb7d1 => (true, 0xb7db),
                    0xb7e9 | 0xb820 => (z, 0xb7ee),
                    0xb828 => (z, 0xb82f),
                    0xb82d => (c, 0xb838),
                    _ => (true, 0xb818),
                };
                if w.branch(t) {
                    a
                } else {
                    self.pc + 2
                }
            }
            _ => unreachable!("room header continuation {:04x}", self.pc),
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
    fn native_data() -> std::collections::BTreeMap<u32, u8> {
        let mut state = crate::ZeldaState::new();
        let path =
            std::env::var_os("ZELDA3_ASSETS").expect("room header test requires native assets");
        state.set_assets(&std::fs::read(path).unwrap()).unwrap();
        state
            .native_dungeon_header_data()
            .expect("consistent native room header and door records")
    }
    #[test]
    fn dungeon_header_native_data_matches_source_records_and_pointers() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let data = native_data();
        for (&address, &value) in &data {
            assert_eq!(
                value,
                rom[(address as usize >> 16) * 0x8000 + (address as usize & 0x7fff)],
                "room header data {address:06x}"
            );
        }
        eprintln!(
            "{} immutable room header/door bytes match native assets and pointer bindings",
            data.len()
        );
    }
    #[test]
    fn dungeon_header_matches_source_rooms_scroll_and_adjacent_doors() {
        let data = native_data();
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut visited = std::collections::BTreeSet::new();
        let mut total = 0;
        let mut cases = 0;
        let mut stores = 0;
        let mut domains = std::collections::BTreeSet::new();
        for room in 0..320u16 {
            for variant in 0..8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i.wrapping_mul(7) ^ (i >> 8)) as u8;
                }
                machine.ram[0x11] = [0, 5, 6, 17, 18, 21, 22, 0xff][variant];
                machine.ram[0x67] = [0, 0, 2, 4, 8, 15, 4, 2][variant];
                machine.ram[0xa0..0xa2].copy_from_slice(&room.to_le_bytes());
                let previous = match variant & 3 {
                    0 => room.wrapping_sub(1),
                    1 => room + 1,
                    2 => room.wrapping_sub(16),
                    _ => room + 16,
                };
                machine.ram[0xa2..0xa4].copy_from_slice(&previous.to_le_bytes());
                machine.ram[0xe2..0xe4].copy_from_slice(
                    &[
                        0x0000u16, 0x01ff, 0xfff0, 0x0200, 0x0100, 0xfeff, 0x001f, 0x8000,
                    ][variant]
                        .to_le_bytes(),
                );
                machine.ram[0xe8..0xea].copy_from_slice(
                    &[
                        0xffffu16, 0x0200, 0x0000, 0x0100, 0xfeff, 0x01ff, 0x8000, 0xffe0,
                    ][variant]
                        .to_le_bytes(),
                );
                for i in 0..320usize {
                    let saved = match variant & 3 {
                        0 => 0,
                        1 => 65535,
                        2 => 0x8000u16 >> (i % 16),
                        _ => ((i as u16).wrapping_mul(251)) ^ 0xa59a,
                    };
                    machine.ram[0xf000 + i * 2..0xf002 + i * 2]
                        .copy_from_slice(&saved.to_le_bytes());
                }
                machine.ram[0x1fc..0x1fe].copy_from_slice(&[0x3c, 0x87]);
                let mut cpu = NativeNmiContext {
                    a: 0xab19,
                    x: 0x71,
                    y: 0x83,
                    d: 0,
                    db: 0,
                    bank: 1,
                    pc: 0xb564,
                    sp: 0x1fb,
                    p: 0x31,
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let mut state = crate::ZeldaState::new();
                state.ram.copy_from_slice(&ram);
                state.sync_native_game_state_from_ram();
                let (v, h) = if variant & 1 == 0 {
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
                let mut work = DungeonHeader::default();
                assert_eq!(work.routine(), 0x01b564);
                let mut count = 0;
                while !work.complete() {
                    visited.insert(cpu.pc);
                    let label = format!("room header {room:03x} variant{variant} pc{:04x}", cpu.pc);
                    let mut published = false;
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| match access {
                            Access::Write(address, value, width) => {
                                if state.native_dungeon_header_store(address, value, width) {
                                    published = true;
                                    stores += 1;
                                    domains.insert((address, width));
                                } else {
                                    assert!(matches!(address, 0..=0xf | 0x100..=0x1ff),
                                        "{label}: unowned header store {address:06x}");
                                    state.ram[address as usize..address as usize + usize::from(width)]
                                        .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                                }
                                None
                            }
                            Access::Read(address, width)
                                if address & 0x8000 != 0 && address < 0x7e0000 =>
                            {
                                let mut value = 0;
                                for byte in 0..width {
                                    let address = address + u32::from(byte);
                                    value |= u16::from(*data.get(&address).unwrap_or_else(|| {
                                        panic!("unbound room header data {address:06x}")
                                    })) << (byte * 8);
                                }
                                Some(value)
                            }
                            _ => None,
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(ram, source.snes().ram, "{label}");
                    assert_eq!(state.ram, ram, "{label}: published native RAM");
                    // Reads and CPU-only instructions cannot change these
                    // owners; check them after every actual publication.
                    if published {
                        let expected = crate::game_state::GameState::load_from_ram(&ram);
                        assert_eq!(state.game_state.dungeon, expected.dungeon, "{label}: dungeon owner");
                        assert_eq!(state.game_state.world, expected.world, "{label}: world owner");
                        assert_eq!(state.game_state.display, expected.display, "{label}: display owner");
                        assert_eq!(state.game_state.sprites.system, expected.sprites.system, "{label}: graphics owner");
                    }
                    work = work.clone();
                    count += 1;
                    assert!(count < 20000, "{label}");
                }
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, work.depth),
                    (1, 0x873d, 0x1fd, 0)
                );
                total += count;
                cases += 1;
            }
        }
        assert_eq!(
            visited.len(),
            335,
            "all header and adjacent-door continuation positions exercised"
        );
        eprintln!(
            "{cases} room header fixtures matched {total} instructions; {} continuation positions",
            visited.len()
        );
        eprintln!("dungeon header native publications: stores={stores} domains={}", domains.len());
    }
}
