//! Native LoadOverworldFromDungeon and its cached-entrance, graphics and
//! camera callees. Fixed routine operands retire on the caller's live clock;
//! instruction bytes are never read from a cartridge.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldDungeonExit {
    pc: u16,
    calls: Vec<u16>,
    complete: bool,
}
impl Default for OverworldDungeonExit {
    fn default() -> Self {
        Self {
            pc: 0xe4a3,
            calls: Vec::new(),
            complete: false,
        }
    }
}
impl OverworldDungeonExit {
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (2, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let xw = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let xmask = if xw == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match pc {
            0xe4a3 | 0xe592 => {
                w.drain(6);
                w.push(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xe4a4 | 0xe593 => {
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                pc + 1
            }
            0xe4a5 | 0xe58a | 0xe594 | 0xe5b6 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xe4a6 | 0xe551 | 0xe5b7 | 0xe62f => {
                w.dp_store(
                    match pc {
                        0xe4a6 => 0x1b,
                        0xe551 => 0x8b,
                        0xe5b7 => 0xa9,
                        0xe62f => 0x2f,
                        _ => unreachable!(),
                    },
                    0,
                    m,
                );
                pc + 2
            }
            0xe4a8 | 0xe4ba | 0xe553 | 0xe55c | 0xe56b | 0xe59d | 0xab0a => {
                w.drain(16);
                w.write(
                    w.absolute(match pc {
                        0xe4a8 => 0x458,
                        0xe4ba => 0x4ac,
                        0xe553 => 0x40b,
                        0xe55c => 0x625,
                        0xe56b => 0x629,
                        0xe59d => 0x713,
                        0xab0a => 0x412,
                        _ => unreachable!(),
                    }),
                    0,
                    m,
                );
                pc + 3
            }
            0xe4ab | 0xe574 | 0xe5a6 | 0xe5d4 | 0xe6de | 0xab4a => {
                w.width(
                    match pc {
                        0xe4ab => 0x20,
                        0xe574 => 0x20,
                        0xe5a6 => 0x20,
                        0xe5d4 => 0x20,
                        0xe6de => 0x20,
                        0xab4a => 0x30,
                        _ => unreachable!(),
                    },
                    true,
                );
                pc + 2
            }
            0xe4ad | 0xe4b4 | 0xe533 | 0xe576 | 0xe580 | 0xe58b | 0xe598 | 0xe5b9 | 0xe5c1
            | 0xe5c7 | 0xe5cd | 0xe61a | 0xab64 | 0xab6c => {
                w.immediate(
                    match pc {
                        0xe4ad => 0x0,
                        0xe4b4 => 0xff,
                        0xe533 => 0x2,
                        0xe576 => 0x0,
                        0xe580 => 0x0,
                        0xe58b => 0xfff8,
                        0xe598 => 0xe4,
                        0xe5b9 => 0x2,
                        0xe5c1 => 0x80,
                        0xe5c7 => 0xf,
                        0xe5cd => 0xff,
                        0xe61a => 0x2,
                        0xab64 => 0x3f0,
                        0xab6c => 0x1f0,
                        _ => unreachable!(),
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xe4b0 => {
                w.drain(24);
                w.write(
                    match pc {
                        0xe4b0 => 0x7ec017,
                        _ => unreachable!(),
                    },
                    w.cpu.a & mask,
                    m,
                );
                pc + 4
            }
            0xe4b7 | 0xe4e4 | 0xe4e7 | 0xe4f1 | 0xe4f4 | 0xe520 | 0xe525 | 0xe52b | 0xe530
            | 0xe53b | 0xe541 | 0xe54e | 0xe559 | 0xe568 | 0xe57d | 0xe587 | 0xe59a | 0xe5da
            | 0xe5e9 | 0xe5ee | 0xe5f7 | 0xe5fc | 0xe658 | 0xe65d | 0xe664 | 0xe669 | 0xe670
            | 0xe677 | 0xe67e | 0xe685 | 0xe68c | 0xe693 | 0xe69a | 0xe6a1 | 0xe6a8 | 0xe6af
            | 0xe6b6 | 0xe6bd | 0xe6c6 | 0xe6cd | 0xe6d4 | 0xe6db | 0xab11 | 0xab18 | 0xab22
            | 0xab29 | 0xab30 | 0xab47 | 0xab57 | 0xab61 | 0xab6f | 0xab75 | 0xc0c6 | 0xc0cd
            | 0xc0d3 | 0xc0da | 0xc0e0 | 0xc0e7 | 0xc0ed | 0xc0f4 => {
                w.drain(16);
                w.write(
                    w.absolute(match pc {
                        0xe4b7 => 0x40c,
                        0xe4e4 => 0x122,
                        0xe4e7 => 0x124,
                        0xe4f1 => 0x11e,
                        0xe4f4 => 0x120,
                        0xe520 => 0x618,
                        0xe525 => 0x61a,
                        0xe52b => 0x61c,
                        0xe530 => 0x61e,
                        0xe53b => 0x696,
                        0xe541 => 0x698,
                        0xe54e => 0x40a,
                        0xe559 => 0x624,
                        0xe568 => 0x628,
                        0xe57d => 0x626,
                        0xe587 => 0x62a,
                        0xe59a => 0x716,
                        0xe5da => 0x40a,
                        0xe5e9 => 0x122,
                        0xe5ee => 0x124,
                        0xe5f7 => 0x11e,
                        0xe5fc => 0x120,
                        0xe658 => 0x618,
                        0xe65d => 0x61a,
                        0xe664 => 0x61c,
                        0xe669 => 0x61e,
                        0xe670 => 0x600,
                        0xe677 => 0x602,
                        0xe67e => 0x604,
                        0xe685 => 0x606,
                        0xe68c => 0x610,
                        0xe693 => 0x612,
                        0xe69a => 0x614,
                        0xe6a1 => 0x616,
                        0xe6a8 => 0x624,
                        0xe6af => 0x626,
                        0xe6b6 => 0x628,
                        0xe6bd => 0x62a,
                        0xe6c6 => 0xaa0,
                        0xe6cd => 0xaa1,
                        0xe6d4 => 0xaa2,
                        0xe6db => 0xaa3,
                        0xab11 => 0xaa3,
                        0xab18 => 0xaa2,
                        0xab22 => 0x714,
                        0xab29 => 0x712,
                        0xab30 => 0x717,
                        0xab47 => 0xaa4,
                        0xab57 => 0x708,
                        0xab61 => 0x70c,
                        0xab6f => 0x70a,
                        0xab75 => 0x70e,
                        0xc0c6 => 0x600,
                        0xc0cd => 0x602,
                        0xc0d3 => 0x604,
                        0xc0da => 0x606,
                        0xc0e0 => 0x610,
                        0xc0e7 => 0x612,
                        0xc0ed => 0x614,
                        0xc0f4 => 0x616,
                        _ => unreachable!(),
                    }),
                    w.cpu.a & mask,
                    m,
                );
                pc + 3
            }
            0xe4bd | 0xe511 | 0xe5a0 | 0xe60b | 0xe612 | 0xe627 | 0xe648 | 0xab37 | 0xab4c => {
                let value = w.dp_read(
                    match pc {
                        0xe4bd => 0xa0,
                        0xe511 => 0x84,
                        0xe5a0 => 0x8a,
                        0xe60b => 0xa0,
                        0xe612 => 0x20,
                        0xe627 => 0x20,
                        0xe648 => 0x84,
                        0xab37 => 0x8a,
                        0xab4c => 0x8a,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load(value, m);
                pc + 2
            }
            0xe4bf | 0xe4c4 | 0xe4c9 | 0xe60d | 0xe622 => {
                w.compare_immediate(
                    match pc {
                        0xe4bf => 0x104,
                        0xe4c4 => 0x180,
                        0xe4c9 => 0x100,
                        0xe60d => 0x124,
                        0xe622 => 0xffff,
                        _ => unreachable!(),
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xe4c2 | 0xe5ad | 0xab3b => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    match pc {
                        0xe4c2 => 0xe4d4,
                        0xe5ad => 0xe5b1,
                        0xab3b => 0xab40,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xe4c7 | 0xe610 => {
                if w.branch(w.cpu.p & CARRY != 0) {
                    match pc {
                        0xe4c7 => 0xe4d4,
                        0xe610 => 0xe61a,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xe4cc | 0xe560 | 0xe56f => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    match pc {
                        0xe4cc => 0xe4d4,
                        0xe560 => 0xe565,
                        0xe56f => 0xe574,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xe4ce | 0xe595 | 0xe5b1 => {
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                self.calls.push(pc + 3);
                match pc {
                    0xe4ce => 0xe5d4,
                    0xe595 => 0xab08,
                    0xe5b1 => 0xc0c3,
                    _ => unreachable!(),
                }
            }
            0xe4d1 => {
                w.drain(16);
                match pc {
                    0xe4d1 => 0xe58a,
                    _ => unreachable!(),
                }
            }
            0xe4d4 | 0xe5a8 | 0xab35 | 0xab3e => {
                w.drain(8 * u32::from(xw));
                w.cpu.x = match pc {
                    0xe4d4 => 0x9e,
                    0xe5a8 => 0x0,
                    0xab35 => 0x0,
                    0xab3e => 0x8,
                    _ => unreachable!(),
                };
                w.flags(w.cpu.x, xw);
                pc + 1 + u16::from(xw)
            }
            0xe4d6 | 0xe4d7 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xe4d8 => {
                w.drain(16);
                let base: u16 = match pc {
                    0xe4d8 => 0xdd8a,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.x)), m);
                let old = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old >= value);
                w.flags(old.wrapping_sub(value) & mask, m);
                pc + 3
            }
            0xe4db | 0xe625 | 0xab6a => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match pc {
                        0xe4db => 0xe4d6,
                        0xe625 => 0xe631,
                        0xab6a => 0xab6f,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xe4dd | 0xe4ea | 0xe4f7 | 0xe4fc | 0xe501 | 0xe51d | 0xe528 | 0xe538 | 0xe53e
            | 0xe549 | 0xe556 | 0xe565 => {
                w.drain(16);
                let base: u16 = match pc {
                    0xe4dd => 0xdf15,
                    0xe4ea => 0xdfb3,
                    0xe4f7 => 0xe051,
                    0xe4fc => 0xe0ef,
                    0xe501 => 0xde77,
                    0xe51d => 0xe18d,
                    0xe528 => 0xe22b,
                    0xe538 => 0xe367,
                    0xe53e => 0xe405,
                    0xe549 => 0xde28,
                    0xe556 => 0xe2c9,
                    0xe565 => 0xe318,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.x)), m);
                w.load(value, m);
                pc + 3
            }
            0xe4e0 | 0xe4e2 | 0xe4ed | 0xe4ef | 0xe4fa | 0xe4ff | 0xe504 | 0xe50f | 0xe51b
            | 0xe536 | 0xe54c | 0xe58e | 0xe5bb | 0xe5bd | 0xe5bf | 0xe5c3 | 0xe5c5 | 0xe5c9
            | 0xe5cb | 0xe5cf | 0xe5d1 | 0xe5e1 | 0xe5e7 | 0xe5ec | 0xe5f5 | 0xe5fa | 0xe603
            | 0xe609 | 0xe618 | 0xe61d | 0xe62d | 0xe635 | 0xe63b | 0xe646 | 0xe652 => {
                w.dp_store(
                    match pc {
                        0xe4e0 => 0xe6,
                        0xe4e2 => 0xe8,
                        0xe4ed => 0xe0,
                        0xe4ef => 0xe2,
                        0xe4fa => 0x20,
                        0xe4ff => 0x22,
                        0xe504 => 0x84,
                        0xe50f => 0x88,
                        0xe51b => 0x86,
                        0xe536 => 0x2f,
                        0xe54c => 0x8a,
                        0xe58e => 0xec,
                        0xe5bb => 0xaa,
                        0xe5bd => 0xa6,
                        0xe5bf => 0xa7,
                        0xe5c3 => 0x45,
                        0xe5c5 => 0x44,
                        0xe5c9 => 0x42,
                        0xe5cb => 0x43,
                        0xe5cf => 0x24,
                        0xe5d1 => 0x29,
                        0xe5e1 => 0x1c,
                        0xe5e7 => 0xe8,
                        0xe5ec => 0xe6,
                        0xe5f5 => 0xe2,
                        0xe5fa => 0xe0,
                        0xe603 => 0x22,
                        0xe609 => 0x20,
                        0xe618 => 0x20,
                        0xe61d => 0x2f,
                        0xe62d => 0x20,
                        0xe635 => 0x8a,
                        0xe63b => 0x84,
                        0xe646 => 0x88,
                        0xe652 => 0x86,
                        _ => unreachable!(),
                    },
                    w.cpu.a & mask,
                    m,
                );
                pc + 2
            }
            0xe506 | 0xe513 | 0xe579 | 0xe583 | 0xe614 | 0xe63d | 0xe64a => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xe507 | 0xe514 | 0xe615 | 0xe63e | 0xe64b => {
                w.drain(8 * u32::from(m));
                w.sbc(
                    match pc {
                        0xe507 => 0x400,
                        0xe514 => 0x10,
                        0xe615 => 0x10,
                        0xe63e => 0x400,
                        0xe64b => 0x10,
                        _ => unreachable!(),
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xe50a | 0xe517 | 0xe5a2 | 0xe641 | 0xe64e | 0xab1c | 0xab39 | 0xab4e => {
                w.mask(
                    match pc {
                        0xe50a => 0xf80,
                        0xe517 => 0x3e,
                        0xe5a2 => 0x3f,
                        0xe641 => 0xf80,
                        0xe64e => 0x3e,
                        0xab1c => 0x3f,
                        0xab39 => 0x40,
                        0xab4e => 0xbf,
                        _ => unreachable!(),
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xe50d | 0xe55f | 0xe56e | 0xe5a4 | 0xe644 | 0xab51 => {
                w.shift_left(m);
                pc + 1
            }
            0xe50e | 0xe645 => {
                w.swap_bytes();
                pc + 1
            }
            0xe51a | 0xe545 | 0xe651 | 0xab5e | 0xab5f | 0xab60 | 0xab72 | 0xab73 | 0xab74 => {
                w.drain(6);
                let value = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(value & 1 != 0);
                w.load(value >> 1, m);
                pc + 1
            }
            0xe523 | 0xe524 | 0xe52e | 0xe52f | 0xe65b | 0xe65c | 0xe667 | 0xe668 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1) & mask, m);
                pc + 1
            }
            0xe544 | 0xab1b => {
                w.drain(6);
                w.load(w.cpu.x, m);
                pc + 1
            }
            0xe546 | 0xab1e | 0xab52 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xe547 | 0xe590 | 0xe5b4 | 0xe6c0 | 0xab78 => {
                w.width(
                    match pc {
                        0xe547 => 0x20,
                        0xe590 => 0x30,
                        0xe5b4 => 0x20,
                        0xe6c0 => 0x20,
                        0xab78 => 0x30,
                        _ => unreachable!(),
                    },
                    false,
                );
                pc + 2
            }
            0xe562 | 0xe571 => {
                w.drain(16);
                let address = w.absolute(match pc {
                    0xe562 => 0x625,
                    0xe571 => 0x629,
                    _ => unreachable!(),
                });
                let value = w.read(address, m).wrapping_sub(1) & mask;
                w.drain(6);
                w.flags(value, m);
                w.write_high_low(address, value, m);
                pc + 3
            }
            0xe57a | 0xe584 => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match pc {
                        0xe57a => 0x624,
                        0xe584 => 0x628,
                        _ => unreachable!(),
                    }),
                    m,
                );
                w.sbc(value, m);
                pc + 3
            }
            0xe5a5 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & xmask;
                w.flags(w.cpu.y, xw);
                pc + 1
            }
            0xe5aa | 0xe61f | 0xab1f => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match pc {
                        0xe5aa => 0x712,
                        0xe61f => 0x696,
                        0xab1f => 0x712,
                        _ => unreachable!(),
                    }),
                    m,
                );
                w.load(value, m);
                pc + 3
            }
            0xe5af | 0xe5b0 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xe5d3 | 0xe6e0 | 0xab7a | 0xc0f7 => {
                w.rts();
                if let Some(returned) = self.calls.pop() {
                    assert_eq!(w.cpu.pc, returned);
                } else {
                    self.complete = true;
                }
                w.cpu.pc
            }
            0xe5d6 | 0xe5dd | 0xe5e3 | 0xe5f1 | 0xe5ff | 0xe605 | 0xe631 | 0xe637 | 0xe654
            | 0xe660 | 0xe66c | 0xe673 | 0xe67a | 0xe681 | 0xe688 | 0xe68f | 0xe696 | 0xe69d
            | 0xe6a4 | 0xe6ab | 0xe6b2 | 0xe6b9 | 0xe6c2 | 0xe6c9 | 0xe6d0 | 0xe6d7 => {
                w.drain(24);
                let value = w.read(
                    match pc {
                        0xe5d6 => 0x7ec140,
                        0xe5dd => 0x7ec142,
                        0xe5e3 => 0x7ec144,
                        0xe5f1 => 0x7ec146,
                        0xe5ff => 0x7ec14a,
                        0xe605 => 0x7ec148,
                        0xe631 => 0x7ec14c,
                        0xe637 => 0x7ec14e,
                        0xe654 => 0x7ec150,
                        0xe660 => 0x7ec152,
                        0xe66c => 0x7ec154,
                        0xe673 => 0x7ec156,
                        0xe67a => 0x7ec158,
                        0xe681 => 0x7ec15a,
                        0xe688 => 0x7ec15c,
                        0xe68f => 0x7ec15e,
                        0xe696 => 0x7ec160,
                        0xe69d => 0x7ec162,
                        0xe6a4 => 0x7ec16a,
                        0xe6ab => 0x7ec16c,
                        0xe6b2 => 0x7ec16e,
                        0xe6b9 => 0x7ec170,
                        0xe6c2 => 0x7ec164,
                        0xe6c9 => 0x7ec165,
                        0xe6d0 => 0x7ec166,
                        0xe6d7 => 0x7ec167,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load(value, m);
                pc + 4
            }
            0xe629 | 0xc0c9 | 0xc0d6 | 0xc0e3 | 0xc0f0 => {
                w.drain(6);
                w.cpu.p &= !CARRY;
                pc + 1
            }
            0xe62a => {
                w.drain(8 * u32::from(m));
                w.adc(match pc {
                    0xe62a => 0x20,
                    _ => unreachable!(),
                });
                pc + 1 + u16::from(m)
            }
            0xab08 => {
                let value = w.dp_read(
                    match pc {
                        0xab08 => 0x8a,
                        _ => unreachable!(),
                    },
                    xw,
                );
                w.cpu.x = value;
                w.flags(value, xw);
                pc + 2
            }
            0xab0d | 0xab14 | 0xab25 | 0xab2c | 0xab43 | 0xab53 | 0xab5a => {
                w.drain(24);
                let value = w.read(
                    (match pc {
                        0xab0d => 0x7efcc0,
                        0xab14 => 0xfc9c,
                        0xab25 => 0x2a844,
                        0xab2c => 0x2a884,
                        0xab43 => 0xd8f4,
                        0xab53 => 0x2a8c4,
                        0xab5a => 0x2a944,
                        _ => unreachable!(),
                    } as u32)
                        .wrapping_add(u32::from(w.cpu.x)),
                    m,
                );
                w.load(value, m);
                pc + 4
            }
            0xab33 => {
                w.drain(8 * u32::from(xw));
                w.cpu.y = match pc {
                    0xab33 => 0x20,
                    _ => unreachable!(),
                };
                w.flags(w.cpu.y, xw);
                pc + 1 + u16::from(xw)
            }
            0xab3d => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & xmask;
                w.flags(w.cpu.y, xw);
                pc + 1
            }
            0xab40 => {
                w.drain(16);
                w.write(
                    w.absolute(match pc {
                        0xab40 => 0xaa1,
                        _ => unreachable!(),
                    }),
                    w.cpu.y,
                    xw,
                );
                pc + 3
            }
            0xab67 => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match pc {
                        0xab67 => 0x712,
                        _ => unreachable!(),
                    }),
                    xw,
                );
                w.cpu.x = value;
                w.flags(value, xw);
                pc + 3
            }
            0xc0c3 | 0xc0d0 | 0xc0dd | 0xc0ea => {
                w.drain(16);
                let base: u16 = match pc {
                    0xc0c3 => 0xa8c4,
                    0xc0d0 => 0xa944,
                    0xc0dd => 0xbee2,
                    0xc0ea => 0xbf62,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.y)), m);
                w.load(value, m);
                pc + 3
            }
            0xc0ca | 0xc0d7 | 0xc0e4 | 0xc0f1 => {
                w.drain(16);
                let base: u16 = match pc {
                    0xc0ca => 0xbfe2,
                    0xc0d7 => 0xbfe6,
                    0xc0e4 => 0xbfea,
                    0xc0f1 => 0xbfee,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.x)), m);
                w.adc(value);
                pc + 3
            }
            _ => unreachable!("dungeon exit PC {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
