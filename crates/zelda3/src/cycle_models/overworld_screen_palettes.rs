//! Screen palette selection and its retained native loader/copy frames.
use super::{
    intro_palettes::IntroPalettes,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EquipmentLoader {
    pc: u16,
    complete: bool,
    copy: Option<IntroPalettes>,
    pair: Option<PalettePairCopy>,
    returned: u16,
}
impl EquipmentLoader {
    fn new(pc: u16) -> Self {
        Self {
            pc,
            complete: false,
            copy: None,
            pair: None,
            returned: 0,
        }
    }
    fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        if let Some(child) = self.copy.as_mut() {
            let cost = child.step(cpu, clock, bus);
            if child.complete() {
                assert_eq!((cpu.bank, cpu.pc), (27, self.returned));
                self.pc = self.returned;
                self.copy = None;
            }
            return cost;
        }
        if let Some(child) = self.pair.as_mut() {
            let cost = child.step(cpu, clock, bus);
            if child.complete {
                assert_eq!((cpu.bank, cpu.pc), (27, self.returned));
                self.pc = self.returned;
                self.pair = None;
            }
            return cost;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (27, self.pc, 0, 0));
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
            0xecc5 | 0xece4 | 0xed03 | 0xed19 | 0xed29 | 0xed3f | 0xed72 | 0xed83 | 0xed91
            | 0xedab | 0xedcf | 0xedf9 | 0xee10 | 0xee1b => {
                w.width(
                    match pc {
                        0xecc5 => 0x31,
                        0xece4 => 0x31,
                        0xed03 => 0x21,
                        0xed19 => 0x10,
                        0xed29 => 0x21,
                        0xed3f => 0x10,
                        0xed72 => 0x21,
                        0xed83 => 0x10,
                        0xed91 => 0x21,
                        0xedab => 0x10,
                        0xedcf => 0x10,
                        0xedf9 => 0x21,
                        0xee10 => 0x10,
                        0xee1b => 0x30,
                        _ => unreachable!(),
                    },
                    true,
                );
                pc + 2
            }
            0xecc7 | 0xece6 => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match pc {
                        0xecc7 => 0xaad,
                        0xece6 => 0xaae,
                        _ => unreachable!(),
                    }),
                    m,
                );
                w.load(value, m);
                pc + 3
            }
            0xecca | 0xece9 | 0xed09 | 0xed11 | 0xed2f | 0xed37 | 0xed7b | 0xed97 | 0xeda3
            | 0xedc6 | 0xedff | 0xee07 | 0xee21 => {
                w.mask(
                    match pc {
                        0xecca => 0xff,
                        0xece9 => 0xff,
                        0xed09 => 0xff,
                        0xed11 => 0xff,
                        0xed2f => 0xff,
                        0xed37 => 0xff,
                        0xed7b => 0xff,
                        0xed97 => 0x40,
                        0xeda3 => 0xff,
                        0xedc6 => 0xff,
                        0xedff => 0xff,
                        0xee07 => 0xff,
                        0xee21 => 0xff,
                        _ => unreachable!(),
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xeccd | 0xecec | 0xee0a | 0xee27 => {
                w.shift_left(m);
                pc + 1
            }
            0xecce | 0xeced | 0xed0c | 0xed32 | 0xee02 | 0xee28 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xeccf | 0xecee | 0xed0d | 0xed33 | 0xed77 | 0xed9f | 0xedc2 | 0xee03 | 0xee29 => {
                w.drain(24);
                let value = w.read(
                    (match pc {
                        0xeccf => 0x1bebd6,
                        0xecee => 0x1bebd6,
                        0xed0d => 0x1bebb4,
                        0xed33 => 0x1bebc1,
                        0xed77 => 0x1bebc6,
                        0xed9f => 0x1bebc6,
                        0xedc2 => 0x1bebc6,
                        0xee03 => 0x1bec06,
                        0xee29 => 0x1bedf5,
                        _ => unreachable!(),
                    } as u32)
                        + u32::from(w.cpu.x),
                    m,
                );
                w.load(value, m);
                pc + 4
            }
            0xecd3 | 0xecf2 | 0xed14 | 0xed3a | 0xed7e | 0xeda6 | 0xedca | 0xee0b => {
                w.drain(8 * u32::from(m));
                w.adc(match pc {
                    0xecd3 => 0xd4e0,
                    0xecf2 => 0xd4e0,
                    0xed14 => 0xd630,
                    0xed3a => 0xd648,
                    0xed7e => 0xd446,
                    0xeda6 => 0xd446,
                    0xedca => 0xd446,
                    0xee0b => 0xd308,
                    _ => unreachable!(),
                });
                pc + 1 + u16::from(m)
            }
            0xecd6 | 0xecf5 | 0xed17 | 0xed3d | 0xed81 | 0xeda9 | 0xedcd | 0xee0e => {
                w.dp_store(
                    match pc {
                        0xecd6 => 0x0,
                        0xecf5 => 0x0,
                        0xed17 => 0x0,
                        0xed3d => 0x0,
                        0xed81 => 0x0,
                        0xeda9 => 0x0,
                        0xedcd => 0x0,
                        0xee0e => 0x0,
                        _ => unreachable!(),
                    },
                    w.cpu.a,
                    m,
                );
                pc + 2
            }
            0xecd8 | 0xecf7 | 0xed1b | 0xed41 | 0xed85 | 0xedad | 0xedb5 | 0xedd1 | 0xee12 => {
                w.immediate(
                    match pc {
                        0xecd8 => 0x1a2,
                        0xecf7 => 0x1c2,
                        0xed1b => 0x1b2,
                        0xed41 => 0x1b8,
                        0xed85 => 0x1d2,
                        0xedad => 0x112,
                        0xedb5 => 0x1f2,
                        0xedd1 => 0x1d2,
                        0xee12 => 0x1e2,
                        _ => unreachable!(),
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xecdb | 0xecfa | 0xed1e | 0xed44 | 0xed88 | 0xed93 | 0xedb8 | 0xedd4 | 0xee15 => {
                w.drain(8 * u32::from(xw));
                w.cpu.x = match pc {
                    0xecdb => 0x6,
                    0xecfa => 0x6,
                    0xed1e => 0x2,
                    0xed44 => 0x3,
                    0xed88 => 0x6,
                    0xed93 => 0x7,
                    0xedb8 => 0x6,
                    0xedd4 => 0x6,
                    0xee15 => 0xe,
                    _ => unreachable!(),
                };
                w.flags(w.cpu.x, xw);
                pc + 1 + u16::from(xw)
            }
            0xecde | 0xecfd | 0xed21 | 0xed47 | 0xed8b | 0xedbb | 0xedd7 | 0xee18 => {
                let target: u16 = match pc {
                    0xecde => 0xef30,
                    0xecfd => 0xef30,
                    0xed21 => 0xef7b,
                    0xed47 => 0xef7b,
                    0xed8b => 0xef30,
                    0xedbb => 0xef30,
                    0xedd7 => 0xef30,
                    0xee18 => 0xef7b,
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                self.returned = pc + 3;
                if target == 0xef7b {
                    self.pair = Some(PalettePairCopy::default());
                } else {
                    self.copy = Some(IntroPalettes::from_copy(0x1b0000 | u32::from(target)));
                }
                target
            }
            0xece1 | 0xed00 | 0xed24 | 0xed4a | 0xed8e | 0xedbe | 0xedda | 0xee35 => {
                w.width(
                    match pc {
                        0xece1 => 0x30,
                        0xed00 => 0x30,
                        0xed24 => 0x30,
                        0xed4a => 0x30,
                        0xed8e => 0x30,
                        0xedbe => 0x10,
                        0xedda => 0x30,
                        0xee35 => 0x30,
                        _ => unreachable!(),
                    },
                    false,
                );
                pc + 2
            }
            0xece3 | 0xed02 | 0xed28 | 0xed4e | 0xed90 | 0xeddc | 0xee39 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            0xed05 | 0xed2b | 0xedfb | 0xee1d => {
                w.drain(24);
                let value = w.read(
                    match pc {
                        0xed05 => 0x7ef359,
                        0xed2b => 0x7ef35a,
                        0xedfb => 0x7ef35b,
                        0xee1d => 0x7ef354,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load(value, m);
                pc + 4
            }
            0xed26 | 0xed4c | 0xee37 => {
                w.dp_increment(
                    match pc {
                        0xed26 => 0x15,
                        0xed4c => 0x15,
                        0xee37 => 0x15,
                        _ => unreachable!(),
                    },
                    m,
                    true,
                );
                pc + 2
            }
            0xed6e | 0xed95 => {
                let value = w.dp_read(
                    match pc {
                        0xed6e => 0x1b,
                        0xed95 => 0x8a,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load(value, m);
                pc + 2
            }
            0xed70 | 0xed9a | 0xedb3 | 0xee24 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    match pc {
                        0xed70 => 0xed91,
                        0xed9a => 0xed9e,
                        0xedb3 => 0xedb8,
                        0xee24 => 0xee35,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xed74 | 0xedb0 => {
                w.drain(16);
                w.cpu.x = w.read(
                    w.absolute(match pc {
                        0xed74 => 0xab1,
                        0xedb0 => 0xabd,
                        _ => unreachable!(),
                    }),
                    xw,
                );
                w.flags(w.cpu.x, xw);
                pc + 3
            }
            0xed9c | 0xed9d => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xed9e => {
                w.drain(6);
                w.push(w.cpu.x, xw);
                pc + 1
            }
            0xedc0 => {
                w.drain(12);
                w.cpu.x = w.pull(xw);
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xedc1 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xedc9 => {
                w.drain(6);
                w.cpu.p &= !CARRY;
                pc + 1
            }
            0xee26 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1), m);
                pc + 1
            }
            0xee2d | 0xee31 => {
                w.drain(24);
                w.write(
                    match pc {
                        0xee2d => 0x7ec4fa,
                        0xee31 => 0x7ec6fa,
                        _ => unreachable!(),
                    },
                    w.cpu.a,
                    m,
                );
                pc + 4
            }
            _ => unreachable!("equipment palette PC {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct PalettePairCopy {
    pc: u16,
    complete: bool,
}
impl Default for PalettePairCopy {
    fn default() -> Self {
        Self {
            pc: 0xef7b,
            complete: false,
        }
    }
}
impl PalettePairCopy {
    fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.p & 0x38), (27, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xef7b => {
                w.drain(6);
                w.cpu.y = w.cpu.x;
                w.flags(w.cpu.y, 2);
                pc + 1
            }
            0xef7c => {
                w.tax_word();
                pc + 1
            }
            0xef7d => {
                w.immediate(27, 2);
                pc + 3
            }
            0xef80 => {
                w.dp_store(2, w.cpu.a, 2);
                pc + 2
            }
            0xef82 => {
                let value = w.indirect_read(0, 2);
                w.load(value, 2);
                pc + 2
            }
            0xef84 | 0xef88 => {
                w.drain(24);
                w.write(
                    (if pc == 0xef84 { 0x7ec300 } else { 0x7ec500 }) + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                pc + 4
            }
            0xef8c | 0xef8e => {
                w.dp_increment(0, 2, true);
                pc + 2
            }
            0xef90 | 0xef91 => {
                w.increment_x();
                pc + 1
            }
            0xef92 => {
                w.decrement_y();
                pc + 1
            }
            0xef93 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    0xef82
                } else {
                    pc + 2
                }
            }
            0xef95 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!(),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum LoaderOwner {
    Shared(IntroPalettes),
    Equipment(EquipmentLoader),
}
impl LoaderOwner {
    pub(super) fn new(entry: u16) -> Self {
        if matches!(entry, 0xec9e | 0xec77 | 0xee52 | 0xeec7 | 0xeee8 | 0xef0c | 0xeea8) {
            Self::Shared(IntroPalettes::from_entry(0x1b0000 | u32::from(entry)))
        } else {
            Self::Equipment(EquipmentLoader::new(entry))
        }
    }
    pub(super) fn complete(&self) -> bool {
        match self {
            Self::Shared(w) => w.complete(),
            Self::Equipment(w) => w.complete,
        }
    }
    pub(super) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        match self {
            Self::Shared(w) => w.step(cpu, clock, bus),
            Self::Equipment(w) => w.step(cpu, clock, bus),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldScreenPalettes {
    pc: u16,
    child: Option<LoaderOwner>,
    returned: u16,
    sp: u16,
    complete: bool,
}
impl Default for OverworldScreenPalettes {
    fn default() -> Self {
        Self {
            pc: 0xc692,
            child: None,
            returned: 0,
            sp: 0,
            complete: false,
        }
    }
}
impl OverworldScreenPalettes {
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
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, self.returned, self.sp));
                self.pc = self.returned;
                self.child = None;
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (2, self.pc, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xc692 | 0xc6a4 | 0xc6cf | 0xc6d9 => {
                w.drain(8);
                w.cpu.x = match pc {
                    0xc692 => 2,
                    0xc6a4 => 0,
                    0xc6cf => 1,
                    _ => 3,
                };
                w.flags(w.cpu.x, 1);
                pc + 2
            }
            0xc694 | 0xc6a6 => {
                let value = w.dp_read(0x8a, 1);
                w.load(value, 1);
                pc + 2
            }
            0xc696 | 0xc6a8 | 0xc6d5 => {
                w.mask(if pc == 0xc696 { 0x3f } else { 0x40 }, 1);
                pc + 2
            }
            0xc698 | 0xc69c | 0xc6a0 => {
                w.compare_immediate(
                    match pc {
                        0xc698 => 3,
                        0xc69c => 5,
                        _ => 7,
                    },
                    1,
                );
                pc + 2
            }
            0xc69a | 0xc69e | 0xc6a2 | 0xc6aa | 0xc6d7 => {
                let target = match pc {
                    0xc6aa => 0xc6ad,
                    0xc6d7 => 0xc6db,
                    _ => 0xc6a6,
                };
                if w.branch(w.cpu.p & ZERO != 0) {
                    target
                } else {
                    pc + 2
                }
            }
            0xc6ac => {
                w.drain(6);
                w.cpu.x = (w.cpu.x + 1) & 255;
                w.flags(w.cpu.x, 1);
                pc + 1
            }
            0xc6ad | 0xc6b0 | 0xc6db => {
                w.drain(16);
                w.write(
                    w.absolute(match pc {
                        0xc6ad => 0xab3,
                        0xc6b0 => 0xaa9,
                        _ => 0xaac,
                    }),
                    if pc == 0xc6b0 { 0 } else { w.cpu.x },
                    1,
                );
                pc + 3
            }
            0xc6d1 => {
                w.drain(24);
                let value = w.read(0x7ef3ca, 1);
                w.load(value, 1);
                pc + 4
            }
            0xc6b3 | 0xc6b7 | 0xc6bb | 0xc6bf | 0xc6c3 | 0xc6c7 | 0xc6cb | 0xc6de | 0xc6e2
            | 0xc6e6 => {
                let entry = match pc {
                    0xc6b3 => 0xec9e,
                    0xc6b7 => 0xed6e,
                    0xc6bb => 0xecc5,
                    0xc6bf => 0xece4,
                    0xc6c3 => 0xed03,
                    0xc6c7 => 0xed29,
                    0xc6cb => 0xedf9,
                    0xc6de => 0xec77,
                    0xc6e2 => 0xee52,
                    _ => 0xeec7,
                };
                self.returned = pc + 4;
                self.sp = w.cpu.sp;
                w.operand_then_internal(3);
                w.push(2, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 27;
                w.cpu.pc = entry;
                self.child = Some(LoaderOwner::new(entry));
                pc + 4
            }
            0xc6ea => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("screen palettes PC {pc:04x}"),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}
