//! Dungeon palettes keep their live selectors, copy cursors and real call returns.
//! Shared palette copies use the same owner as the intro. Cache publication is
//! ordered word by word, so suspension never exposes an eager final palette.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    intro_palettes::{IntroPalettes, PaletteSource},
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonPalettes {
    pc: u32,
    entry: u32,
    loader: Option<(u32, u32)>,
    copy_return: Option<(u32, u32)>,
    shared: Option<(IntroPalettes, Option<u32>)>,
    complete: bool,
}
impl DungeonPalettes {
    pub(crate) fn from_entry(entry: u32) -> Self {
        assert!(matches!(
            entry,
            0x02c630 | 0x1becc5 | 0x1bece4 | 0x1bed03 | 0x1bed29 | 0x1bed6e | 0x1bedf9 | 0x0ed6dd
        ));
        Self {
            pc: entry,
            entry,
            loader: None,
            copy_return: None,
            shared: None,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        if let Some((owner, _)) = &self.shared {
            owner.routine()
        } else if let Some((entry, _)) = self.copy_return {
            entry
        } else if let Some((entry, _)) = self.loader {
            entry
        } else {
            self.entry
        }
    }
    pub(crate) fn palette_source(&self, cpu: &NativeNmiContext) -> PaletteSource {
        if let Some((owner, _)) = &self.shared {
            return owner.palette_source(cpu);
        }
        match self.pc {
            0x1bef84 | 0x1bef88 | 0x1bee2d | 0x1bee31 | 0x0ed745 | 0x0ed749 => PaletteSource::Asset,
            0x02c771 | 0x02c779 | 0x02c781 | 0x02c789 | 0x02c791 | 0x02c799 | 0x02c7a1
            | 0x02c7a9 => PaletteSource::AuxCopy(cpu.x + ((self.pc - 0x02c771) / 8 * 0x40) as u16),
            _ => PaletteSource::Constant,
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
        if let Some((owner, expected)) = &mut self.shared {
            let cycles = owner.step(cpu, clock, bus);
            if owner.complete() {
                let returned = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
                if let Some(expected) = *expected {
                    assert_eq!(returned, expected);
                    self.pc = returned;
                } else if let Some((_, expected)) = self.loader.take() {
                    assert_eq!(returned, expected);
                    self.pc = returned;
                } else {
                    self.complete = true;
                }
                self.shared = None;
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
        w.fetch();
        self.pc = match self.pc {
            0x1becc5 | 0x1bece4 | 0x1bed03 | 0x1bed29 | 0x1bed91 | 0x1bedf9 | 0x0ed6dd
            | 0x02c674 => {
                w.width(
                    match self.pc {
                        0x1becc5 | 0x1bece4 => 0x31,
                        0x1bed03 | 0x1bed29 | 0x1bed91 | 0x1bedf9 => 0x21,
                        0x0ed6dd | 0x02c674 => 0x20,
                        _ => unreachable!(),
                    },
                    true,
                );
                self.pc + 2
            }
            0x1becc7 | 0x1bece6 => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x1becc7 => 0xaad,
                    0x1bece6 => 0xaae,
                    _ => unreachable!(),
                });
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x1becca | 0x1bece9 | 0x1bed09 | 0x1bed11 | 0x1bed2f | 0x1bed37 | 0x1bed97
            | 0x1beda3 | 0x1bedc6 | 0x1bedff | 0x1bee07 | 0x1bee21 | 0x0ed6f5 | 0x0ed70f
            | 0x0ed729 => {
                w.mask(
                    match self.pc {
                        0x1becca | 0x1bece9 | 0x1bed09 | 0x1bed11 | 0x1bed2f | 0x1bed37
                        | 0x1beda3 | 0x1bedc6 | 0x1bedff | 0x1bee07 | 0x1bee21 | 0x0ed6f5
                        | 0x0ed70f | 0x0ed729 => 0xff,
                        0x1bed97 => 0x40,
                        _ => unreachable!(),
                    },
                    2,
                );
                self.pc + 3
            }
            0x1beccd | 0x1becec | 0x1bee0a | 0x1bee27 | 0x0ed72c => {
                w.shift_left(2);
                self.pc + 1
            }
            0x1becce | 0x1beced | 0x1bee28 | 0x1bef7c => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 65535;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x1beccf | 0x1becee | 0x1bed0d | 0x1bed33 | 0x1bed9f | 0x1bedc2 | 0x1bee03
            | 0x1bee29 | 0x0ed6f1 | 0x0ed70b | 0x0ed725 | 0x02c76d | 0x02c775 | 0x02c77d
            | 0x02c785 | 0x02c78d | 0x02c795 | 0x02c79d | 0x02c7a5 => {
                w.drain(24);
                let a = match self.pc {
                    0x1beccf | 0x1becee => 0x1bebd6,
                    0x1bed0d | 0x0ed6f1 => 0x1bebb4,
                    0x1bed33 | 0x0ed70b => 0x1bebc1,
                    0x1bed9f | 0x1bedc2 => 0x1bebc6,
                    0x1bee03 | 0x0ed725 => 0x1bec06,
                    0x1bee29 => 0x1bedf5,
                    0x02c76d => 0x7ec300,
                    0x02c775 => 0x7ec340,
                    0x02c77d => 0x7ec380,
                    0x02c785 => 0x7ec3c0,
                    0x02c78d => 0x7ec400,
                    0x02c795 => 0x7ec440,
                    0x02c79d => 0x7ec480,
                    0x02c7a5 => 0x7ec4c0,
                    _ => unreachable!(),
                } + u32::from(w.cpu.x);
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 4
            }
            0x1becd3 | 0x1becf2 | 0x1bed14 | 0x1bed3a | 0x1beda6 | 0x1bedca | 0x1bee0b
            | 0x0ed6f9 | 0x0ed713 | 0x0ed72e => {
                w.drain(16);
                w.adc(match self.pc {
                    0x1becd3 | 0x1becf2 => 0xd4e0,
                    0x1bed14 | 0x0ed6f9 => 0xd630,
                    0x1bed3a | 0x0ed713 => 0xd648,
                    0x1beda6 | 0x1bedca => 0xd446,
                    0x1bee0b | 0x0ed72e => 0xd308,
                    _ => unreachable!(),
                });
                self.pc + 3
            }
            0x1becd6 | 0x1becf5 | 0x1bed17 | 0x1bed3d | 0x1beda9 | 0x1bedcd | 0x1bee0e
            | 0x1bef80 | 0x0ed6e3 | 0x0ed6e8 | 0x0ed6ed | 0x0ed741 => {
                w.dp_store(
                    match self.pc {
                        0x1becd6 | 0x1becf5 | 0x1bed17 | 0x1bed3d | 0x1beda9 | 0x1bedcd
                        | 0x1bee0e | 0x0ed741 => 0x0,
                        0x1bef80 | 0x0ed6ed => 0x2,
                        0x0ed6e3 => 0xc,
                        0x0ed6e8 => 0xe,
                        _ => unreachable!(),
                    },
                    w.cpu.a & 65535,
                    2,
                );
                self.pc + 2
            }
            0x1becd8 | 0x1becf7 | 0x1bed1b | 0x1bed41 | 0x1bedad | 0x1bedb5 | 0x1bedd1
            | 0x1bee12 | 0x1bef7d | 0x0ed6e5 | 0x0ed6ea | 0x02c676 | 0x02c67d | 0x02c684 => {
                w.drain(16);
                let v = match self.pc {
                    0x1becd8 => 0x1a2,
                    0x1becf7 => 0x1c2,
                    0x1bed1b => 0x1b2,
                    0x1bed41 => 0x1b8,
                    0x1bedad => 0x112,
                    0x1bedb5 => 0x1f2,
                    0x1bedd1 => 0x1d2,
                    0x1bee12 => 0x1e2,
                    0x1bef7d | 0x0ed6ea => 0x1b,
                    0x0ed6e5 => 0x303,
                    0x02c676 => 0x2,
                    0x02c67d | 0x02c684 => 0x0,
                    _ => unreachable!(),
                };
                w.load(v, 2);
                self.pc + 3
            }
            0x1becdb | 0x1becfa | 0x1bed1e | 0x1bed44 | 0x1bedb8 | 0x1bedd4 | 0x1bee15
            | 0x0ed6fe | 0x0ed718 | 0x0ed733 => {
                w.drain(16);
                let v = match self.pc {
                    0x1becdb | 0x1becfa | 0x1bedb8 | 0x1bedd4 => 0x6,
                    0x1bed1e => 0x2,
                    0x1bed44 => 0x3,
                    0x1bee15 => 0xe,
                    0x0ed6fe => 0x1b2,
                    0x0ed718 => 0x1b8,
                    0x0ed733 => 0x1e2,
                    _ => unreachable!(),
                };
                w.cpu.x = v;
                w.flags(v, 2);
                self.pc + 3
            }
            0x1becde | 0x1becfd | 0x1bed21 | 0x1bed47 | 0x1bedbb | 0x1bedd7 | 0x1bee18
            | 0x0ed704 | 0x0ed71e | 0x0ed739 => {
                let target = match self.pc {
                    0x1becde | 0x1becfd | 0x1bedbb | 0x1bedd7 => 0x1bef30,
                    0x1bed21 | 0x1bed47 | 0x1bee18 => 0x1bef7b,
                    0x0ed704 | 0x0ed71e | 0x0ed739 => 0xed741,
                    _ => unreachable!(),
                };
                let next = self.pc + 3;
                w.operand_then_internal(2);
                w.push((self.pc + 2) as u16, 2);
                if target == 0x1bef30 {
                    self.shared = Some((IntroPalettes::from_copy(target), Some(next)));
                } else {
                    assert!(self.copy_return.is_none());
                    self.copy_return = Some((target, next));
                }
                target
            }
            0x1bece1 | 0x1bed00 | 0x1bed24 | 0x1bed4a | 0x1bedbe | 0x1bedda | 0x1bee35
            | 0x0ed707 | 0x0ed721 | 0x0ed73c | 0x02c7b3 => {
                w.width(
                    match self.pc {
                        0x1bece1 | 0x1bed00 | 0x1bed24 | 0x1bed4a | 0x1bedda | 0x1bee35
                        | 0x0ed73c => 0x30,
                        0x1bedbe | 0x0ed707 | 0x0ed721 => 0x10,
                        0x02c7b3 => 0x20,
                        _ => unreachable!(),
                    },
                    false,
                );
                self.pc + 2
            }
            0x1bece3 | 0x1bed02 | 0x1bed28 | 0x1bed4e | 0x1beddc | 0x1bee39 | 0x0ed740 => {
                w.rtl();
                let returned = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                assert!(self.copy_return.is_none());
                if let Some((_, expected)) = self.loader.take() {
                    assert_eq!(returned, expected);
                } else {
                    self.complete = true;
                }
                returned
            }
            0x1bed05 | 0x1bed2b | 0x1bedfb | 0x1bee1d | 0x0ed6df => {
                w.drain(24);
                let a = match self.pc {
                    0x1bed05 | 0x0ed6df => 0x7ef359,
                    0x1bed2b => 0x7ef35a,
                    0x1bedfb => 0x7ef35b,
                    0x1bee1d => 0x7ef354,
                    _ => unreachable!(),
                };
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 4
            }
            0x1bed0c | 0x1bed32 | 0x1bee02 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0x1bed19 | 0x1bed3f | 0x1bedab | 0x1bedcf | 0x1bee10 | 0x1bee1b | 0x0ed6fc
            | 0x0ed716 | 0x0ed731 | 0x02c769 => {
                w.width(
                    match self.pc {
                        0x1bed19 | 0x1bed3f | 0x1bedab | 0x1bedcf | 0x1bee10 | 0x0ed6fc
                        | 0x0ed716 | 0x0ed731 => 0x10,
                        0x1bee1b => 0x30,
                        0x02c769 => 0x20,
                        _ => unreachable!(),
                    },
                    true,
                );
                self.pc + 2
            }
            0x1bed26 | 0x1bed4c | 0x1bee37 | 0x0ed73e | 0x02c7b5 => {
                w.dp_increment(0x15, 1, true);
                self.pc + 2
            }
            0x1bed6e => {
                let v = w.dp_read(0x1b, 1);
                w.load(v, 1);
                self.pc + 2
            }
            0x1bed70 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0x1bed91
                } else {
                    self.pc + 2
                }
            }
            0x1bed93 | 0x02c76b => {
                w.drain(8);
                let v = match self.pc {
                    0x1bed93 => 0x7,
                    0x02c76b => 0x0,
                    _ => unreachable!(),
                };
                w.cpu.x = v;
                w.flags(v, 1);
                self.pc + 2
            }
            0x1bed95 => {
                let v = w.dp_read(0x8a, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0x1bed9a | 0x1bedb3 | 0x1bee24 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    match self.pc {
                        0x1bed9a => 0x1bed9e,
                        0x1bedb3 => 0x1bedb8,
                        0x1bee24 => 0x1bee35,
                        _ => unreachable!(),
                    }
                } else {
                    self.pc + 2
                }
            }
            0x1bed9c | 0x1bed9d | 0x02c7ad | 0x02c7ae => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0x1bed9e => {
                w.drain(6);
                w.push(w.cpu.x, 1);
                self.pc + 1
            }
            0x1bedb0 => {
                w.drain(16);
                let a = w.absolute(0xabd);
                let v = w.read(a, 2);
                w.cpu.x = v;
                w.flags(v, 2);
                self.pc + 3
            }
            0x1bedc0 => {
                w.drain(12);
                w.cpu.x = w.pull(1);
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0x1bedc1 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0x1bedc9 | 0x0ed6f8 | 0x0ed712 | 0x0ed72d => {
                w.clear_carry();
                self.pc + 1
            }
            0x1bee26 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1) & 65535, 2);
                self.pc + 1
            }
            0x1bee2d | 0x1bee31 | 0x02c679 | 0x02c680 | 0x02c687 => {
                w.drain(24);
                let a = match self.pc {
                    0x1bee2d => 0x7ec4fa,
                    0x1bee31 => 0x7ec6fa,
                    0x02c679 => 0x7ec009,
                    0x02c680 => 0x7ec007,
                    0x02c687 => 0x7ec00b,
                    _ => unreachable!(),
                };
                w.write(a, w.cpu.a & 65535, 2);
                self.pc + 4
            }
            0x1bef7b => {
                w.drain(6);
                w.cpu.y = w.cpu.x & 65535;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x1bef82 | 0x0ed743 => {
                let v = w.indirect_read(0x0, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0x1bef84 | 0x1bef88 | 0x0ed745 | 0x0ed749 | 0x02c771 | 0x02c779 | 0x02c781
            | 0x02c789 | 0x02c791 | 0x02c799 | 0x02c7a1 | 0x02c7a9 => {
                w.drain(24);
                let a = match self.pc {
                    0x1bef84 | 0x0ed745 => 0x7ec300,
                    0x1bef88 | 0x0ed749 | 0x02c771 => 0x7ec500,
                    0x02c779 => 0x7ec540,
                    0x02c781 => 0x7ec580,
                    0x02c789 => 0x7ec5c0,
                    0x02c791 => 0x7ec600,
                    0x02c799 => 0x7ec640,
                    0x02c7a1 => 0x7ec680,
                    0x02c7a9 => 0x7ec6c0,
                    _ => unreachable!(),
                } + u32::from(w.cpu.x);
                w.write(a, w.cpu.a & 65535, 2);
                self.pc + 4
            }
            0x1bef8c | 0x1bef8e | 0x0ed74d | 0x0ed74f => {
                w.dp_increment(0x0, 2, true);
                self.pc + 2
            }
            0x1bef90 | 0x1bef91 | 0x0ed751 | 0x0ed752 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & 65535;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x1bef92 | 0x0ed753 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 65535;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x1bef93 | 0x0ed754 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    match self.pc {
                        0x1bef93 => 0x1bef82,
                        0x0ed754 => 0xed743,
                        _ => unreachable!(),
                    }
                } else {
                    self.pc + 2
                }
            }
            0x1bef95 | 0x0ed756 => {
                w.rts();
                let returned = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                if let Some((_, expected)) = self.copy_return.take() {
                    assert_eq!(returned, expected);
                } else {
                    assert_eq!(self.entry, 0x02c630);
                    assert!(self.loader.is_none());
                    self.complete = true;
                }
                returned
            }
            0x0ed6ef | 0x0ed709 | 0x0ed723 => {
                let v = w.dp_read(
                    match self.pc {
                        0x0ed6ef => 0xc,
                        0x0ed709 => 0xd,
                        0x0ed723 => 0xe,
                        _ => unreachable!(),
                    },
                    1,
                );
                w.cpu.x = v;
                w.flags(v, 1);
                self.pc + 2
            }
            0x0ed701 | 0x0ed71b | 0x0ed736 => {
                w.drain(16);
                let v = match self.pc {
                    0x0ed701 => 0x2,
                    0x0ed71b => 0x3,
                    0x0ed736 => 0xe,
                    _ => unreachable!(),
                };
                w.cpu.y = v;
                w.flags(v, 2);
                self.pc + 3
            }
            0x02c630 => {
                w.drain(16);
                let a = w.absolute(0xaa9);
                w.write(a, 0, 1);
                self.pc + 3
            }
            0x02c633 | 0x02c637 | 0x02c63b | 0x02c63f | 0x02c643 | 0x02c647 | 0x02c64b
            | 0x02c64f | 0x02c653 | 0x02c657 | 0x02c65b => {
                let target = match self.pc {
                    0x02c633 => 0xed5f4,
                    0x02c637 => 0x1bec77,
                    0x02c63b => 0x1bec9e,
                    0x02c63f => 0x1becc5,
                    0x02c643 => 0x1bece4,
                    0x02c647 => 0x1bed03,
                    0x02c64b => 0x1bed29,
                    0x02c64f => 0x1bed6e,
                    0x02c653 => 0x1bedf9,
                    0x02c657 => 0x1bee52,
                    0x02c65b => 0x1bee74,
                    _ => unreachable!(),
                };
                let next = self.pc + 4;
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push((self.pc + 3) as u16, 2);
                if matches!(target, 0x0ed5f4 | 0x1bec77 | 0x1bec9e | 0x1bee52 | 0x1bee74) {
                    self.shared = Some((IntroPalettes::from_entry(target), Some(next)));
                } else {
                    assert!(self.loader.is_none());
                    self.loader = Some((target, next));
                }
                target
            }
            0x02c65f | 0x02c666 | 0x02c66d => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x02c65f => 0xab6,
                    0x02c666 => 0xab8,
                    0x02c66d => 0xab7,
                    _ => unreachable!(),
                });
                let v = w.read(a, 1);
                w.load(v, 1);
                self.pc + 3
            }
            0x02c662 | 0x02c669 | 0x02c670 => {
                w.drain(24);
                let a = match self.pc {
                    0x02c662 => 0x7ec20a,
                    0x02c669 => 0x7ec20b,
                    0x02c670 => 0x7ec20c,
                    _ => unreachable!(),
                };
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 4
            }
            0x02c68b => {
                w.drain(16);
                0x2c769
            }
            0x02c7af => {
                w.compare_x_immediate(0x40, 1);
                self.pc + 2
            }
            0x02c7b1 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0x2c76d
                } else {
                    self.pc + 2
                }
            }
            0x02c7b7 => {
                w.rts();
                let returned = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                if let Some((_, expected)) = self.copy_return.take() {
                    assert_eq!(returned, expected);
                } else {
                    assert_eq!(self.entry, 0x02c630);
                    assert!(self.loader.is_none());
                    self.complete = true;
                }
                returned
            }
            _ => unreachable!("dungeon palette continuation {:06x}", self.pc),
        };
        if self.pc == 0x1bed72 {
            self.shared = Some((IntroPalettes::from_entry(self.pc), None));
        }
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}
#[cfg(test)]
#[path = "dungeon_palettes_tests.rs"]
mod tests;
#[cfg(test)]
pub(super) const OWNED_POSITIONS: &[u32] = &[
    0x2c630, 0x2c633, 0x2c637, 0x2c63b, 0x2c63f, 0x2c643, 0x2c647, 0x2c64b, 0x2c64f, 0x2c653,
    0x2c657, 0x2c65b, 0x2c65f, 0x2c662, 0x2c666, 0x2c669, 0x2c66d, 0x2c670, 0x2c674, 0x2c676,
    0x2c679, 0x2c67d, 0x2c680, 0x2c684, 0x2c687, 0x2c68b, 0x2c769, 0x2c76b, 0x2c76d, 0x2c771,
    0x2c775, 0x2c779, 0x2c77d, 0x2c781, 0x2c785, 0x2c789, 0x2c78d, 0x2c791, 0x2c795, 0x2c799,
    0x2c79d, 0x2c7a1, 0x2c7a5, 0x2c7a9, 0x2c7ad, 0x2c7ae, 0x2c7af, 0x2c7b1, 0x2c7b3, 0x2c7b5,
    0x2c7b7, 0xed6dd, 0xed6df, 0xed6e3, 0xed6e5, 0xed6e8, 0xed6ea, 0xed6ed, 0xed6ef, 0xed6f1,
    0xed6f5, 0xed6f8, 0xed6f9, 0xed6fc, 0xed6fe, 0xed701, 0xed704, 0xed707, 0xed709, 0xed70b,
    0xed70f, 0xed712, 0xed713, 0xed716, 0xed718, 0xed71b, 0xed71e, 0xed721, 0xed723, 0xed725,
    0xed729, 0xed72c, 0xed72d, 0xed72e, 0xed731, 0xed733, 0xed736, 0xed739, 0xed73c, 0xed73e,
    0xed740, 0xed741, 0xed743, 0xed745, 0xed749, 0xed74d, 0xed74f, 0xed751, 0xed752, 0xed753,
    0xed754, 0xed756, 0x1becc5, 0x1becc7, 0x1becca, 0x1beccd, 0x1becce, 0x1beccf, 0x1becd3,
    0x1becd6, 0x1becd8, 0x1becdb, 0x1becde, 0x1bece1, 0x1bece3, 0x1bece4, 0x1bece6, 0x1bece9,
    0x1becec, 0x1beced, 0x1becee, 0x1becf2, 0x1becf5, 0x1becf7, 0x1becfa, 0x1becfd, 0x1bed00,
    0x1bed02, 0x1bed03, 0x1bed05, 0x1bed09, 0x1bed0c, 0x1bed0d, 0x1bed11, 0x1bed14, 0x1bed17,
    0x1bed19, 0x1bed1b, 0x1bed1e, 0x1bed21, 0x1bed24, 0x1bed26, 0x1bed28, 0x1bed29, 0x1bed2b,
    0x1bed2f, 0x1bed32, 0x1bed33, 0x1bed37, 0x1bed3a, 0x1bed3d, 0x1bed3f, 0x1bed41, 0x1bed44,
    0x1bed47, 0x1bed4a, 0x1bed4c, 0x1bed4e, 0x1bed6e, 0x1bed70, 0x1bed91, 0x1bed93, 0x1bed95,
    0x1bed97, 0x1bed9a, 0x1bed9c, 0x1bed9d, 0x1bed9e, 0x1bed9f, 0x1beda3, 0x1beda6, 0x1beda9,
    0x1bedab, 0x1bedad, 0x1bedb0, 0x1bedb3, 0x1bedb5, 0x1bedb8, 0x1bedbb, 0x1bedbe, 0x1bedc0,
    0x1bedc1, 0x1bedc2, 0x1bedc6, 0x1bedc9, 0x1bedca, 0x1bedcd, 0x1bedcf, 0x1bedd1, 0x1bedd4,
    0x1bedd7, 0x1bedda, 0x1beddc, 0x1bedf9, 0x1bedfb, 0x1bedff, 0x1bee02, 0x1bee03, 0x1bee07,
    0x1bee0a, 0x1bee0b, 0x1bee0e, 0x1bee10, 0x1bee12, 0x1bee15, 0x1bee18, 0x1bee1b, 0x1bee1d,
    0x1bee21, 0x1bee24, 0x1bee26, 0x1bee27, 0x1bee28, 0x1bee29, 0x1bee2d, 0x1bee31, 0x1bee35,
    0x1bee37, 0x1bee39, 0x1bef7b, 0x1bef7c, 0x1bef7d, 0x1bef80, 0x1bef82, 0x1bef84, 0x1bef88,
    0x1bef8c, 0x1bef8e, 0x1bef90, 0x1bef91, 0x1bef92, 0x1bef93, 0x1bef95,
];
