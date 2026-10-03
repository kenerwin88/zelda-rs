//! Live tile-attribute preparation. Each owner retains the bank changes, loop
//! cursors and source return while publishing individual memory operations.
#[cfg(test)]
use super::intro_clear::Access;
use super::{dungeon_door_attributes::DoorAttributes, dungeon_object_attributes::ObjectAttributes};
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[cfg(test)]
#[path = "dungeon_attributes_tests.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
enum AttributeChild {
    Basic(BasicAttributes),
    Object(ObjectAttributes),
    Door(DoorAttributes),
    Peg(CrystalPegAttributes),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonAttributes {
    pc: u16,
    child: Option<AttributeChild>,
    complete: bool,
}
impl Default for DungeonAttributes {
    fn default() -> Self {
        Self {
            pc: 0xb8bf,
            child: None,
            complete: false,
        }
    }
}
impl DungeonAttributes {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match &self.child {
            Some(AttributeChild::Basic(w)) => w.routine(),
            Some(AttributeChild::Object(w)) => w.routine(),
            Some(AttributeChild::Door(w)) => w.routine(),
            Some(AttributeChild::Peg(w)) => w.routine(),
            None => 0x01b8bf,
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
        if let Some(child) = &mut self.child {
            let (cycles, done) = match child {
                AttributeChild::Basic(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                AttributeChild::Object(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                AttributeChild::Door(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                AttributeChild::Peg(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
            };
            if done {
                self.child = None;
                assert_eq!((cpu.bank, cpu.pc), (1, self.pc));
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 8),
            (1, self.pc, 0, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xb8bf => {
                w.width(0x20, true);
                0xb8c1
            }
            0xb8c1 | 0xb8c3 => {
                w.dp_store(if self.pc == 0xb8c1 { 0xb2 } else { 0xb4 }, 0, 2);
                self.pc + 2
            }
            0xb8c5 => {
                w.immediate(0x1000, 2);
                0xb8c8
            }
            0xb8c8 => {
                w.dp_store(0, w.cpu.a, 2);
                0xb8ca
            }
            0xb8ca | 0xb8cf | 0xb8d2 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                let (target, child) = match self.pc {
                    0xb8ca => (
                        0xb8f3,
                        AttributeChild::Basic(BasicAttributes::new(BasicAttributeEntry::Full)),
                    ),
                    0xb8cf => (0xb967, AttributeChild::Object(ObjectAttributes::default())),
                    _ => (0xbe17, AttributeChild::Door(DoorAttributes::default())),
                };
                w.cpu.pc = target;
                self.child = Some(child);
                self.pc + 3
            }
            0xb8cd => {
                w.width(0x30, false);
                0xb8cf
            }
            0xb8d5 => {
                w.drain(24);
                let v = w.read(0x7ec172, 1);
                w.load(v, 1);
                0xb8d9
            }
            0xb8d9 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0xb8df
                } else {
                    0xb8db
                }
            }
            0xb8db => {
                w.operand_then_internal(3);
                w.push(1, 1);
                w.push(0xb8de, 2);
                w.cpu.pc = 0xc22a;
                self.child = Some(AttributeChild::Peg(CrystalPegAttributes::default()));
                0xb8df
            }
            0xb8df => {
                w.drain(16);
                w.write(0x0200, 0, 1);
                0xb8e2
            }
            0xb8e2 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("attribute parent continuation {:04x}", self.pc),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CustomAttributes {
    pc: u16,
    complete: bool,
}
impl Default for CustomAttributes {
    fn default() -> Self {
        Self {
            pc: 0x942a,
            complete: false,
        }
    }
}
impl CustomAttributes {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x0e942a
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (0x0e, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x942a | 0x942b => {
                w.drain(6);
                w.push(
                    u16::from(if self.pc == 0x942a {
                        w.cpu.db
                    } else {
                        w.cpu.bank
                    }),
                    1,
                );
                self.pc + 1
            }
            0x942c | 0x9457 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                self.pc + 1
            }
            0x942d | 0x9455 => {
                w.width(0x30, self.pc == 0x942d);
                self.pc + 2
            }
            0x942f => {
                w.drain(16);
                let v = w.read((u32::from(w.cpu.db) << 16) | 0x0aa2, 2);
                w.load(v, 2);
                0x9432
            }
            0x9432 => {
                w.mask(255, 2);
                0x9435
            }
            0x9435 => {
                w.shift_left(2);
                0x9436
            }
            0x9436 => {
                w.drain(6);
                w.cpu.x = w.cpu.a;
                w.flags(w.cpu.x, 2);
                0x9437
            }
            0x9437 | 0x943e | 0x9445 => {
                w.operand_then_internal(2);
                let address = match self.pc {
                    0x9437 => 0x9000 + u32::from(w.cpu.x),
                    0x943e => 0x902a + u32::from(w.cpu.y),
                    _ => 0x906a + u32::from(w.cpu.y),
                } + (u32::from(w.cpu.db) << 16);
                let v = w.read(address, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x943a => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                0x943b
            }
            0x943b => {
                w.drain(16);
                w.cpu.x = 0;
                w.flags(0, 2);
                0x943e
            }
            0x9441 | 0x9448 => {
                w.drain(24);
                w.write(
                    (if self.pc == 0x9441 {
                        0x7eff40
                    } else {
                        0x7eff80
                    }) + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                self.pc + 4
            }
            0x944c | 0x944d => {
                w.increment_y();
                self.pc + 1
            }
            0x944e | 0x944f => {
                w.increment_x();
                self.pc + 1
            }
            0x9450 => {
                w.compare_x_immediate(0x40, 2);
                0x9453
            }
            0x9453 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0x943e
                } else {
                    0x9455
                }
            }
            0x9458 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("custom attributes continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BasicAttributeEntry {
    Initialize,
    Partial,
    Full,
}
impl BasicAttributeEntry {
    fn pc(self) -> u16 {
        match self {
            Self::Initialize => 0xb8e3,
            Self::Partial => 0xb8ec,
            Self::Full => 0xb8f3,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BasicAttributes {
    pc: u16,
    entry: BasicAttributeEntry,
    complete: bool,
}
impl BasicAttributes {
    pub(crate) fn new(entry: BasicAttributeEntry) -> Self {
        Self {
            pc: entry.pc(),
            entry,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x010000 | u32::from(self.entry.pc())
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (1, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        w.fetch();
        self.pc = match self.pc {
            0xb8e3 | 0xb8ec | 0xb8f8 | 0xb911 | 0xb944 => {
                let mask = match self.pc {
                    0xb8f8 => 0x10,
                    0xb944 => 0x21,
                    _ => 0x20,
                };
                w.width(mask, self.pc != 0xb911);
                self.pc + 2
            }
            0xb8e5 | 0xb962 => {
                w.drain(16);
                let a = (u32::from(w.cpu.db) << 16) | 0x0200;
                let v = w.read(a, 2).wrapping_add(1);
                w.drain(6);
                w.flags(v, 2);
                w.write(a, v, 2);
                self.pc + 3
            }
            0xb8e8 | 0xb8ea => {
                w.dp_store(if self.pc == 0xb8e8 { 0xb2 } else { 0xb4 }, 0, 2);
                self.pc + 2
            }
            0xb8ee => {
                w.immediate(0x40, 2);
                0xb8f1
            }
            0xb8f1 => {
                w.dp_store(0, w.cpu.a, 2);
                0xb8f3
            }
            0xb8f3 | 0xb8f6 => {
                w.drain(6);
                w.push(
                    if self.pc == 0xb8f3 {
                        u16::from(w.cpu.db)
                    } else {
                        w.cpu.x
                    },
                    1,
                );
                self.pc + 1
            }
            0xb8f4 => {
                w.drain(8);
                w.cpu.x = 0x7e;
                w.flags(0x7e, 1);
                0xb8f6
            }
            0xb8f7 | 0xb965 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                self.pc + 1
            }
            0xb8fa | 0xb946 => {
                w.cpu.x = w.dp_read(if self.pc == 0xb8fa { 0xb2 } else { 0xb4 }, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0xb8fc | 0xb909 | 0xb91e | 0xb936 => {
                w.drain(24);
                let a = match self.pc {
                    0xb8fc => 0x7e2002,
                    0xb909 => 0x7e2000,
                    0xb91e => 0x7e2001,
                    _ => 0x7e2003,
                };
                let v = w.read(a + u32::from(w.cpu.x), m);
                w.load(v, m);
                self.pc + 4
            }
            0xb900 | 0xb90d | 0xb925 | 0xb93d => {
                w.mask(if m == 2 { 0x3ff } else { 3 }, m);
                self.pc + 1 + u16::from(m)
            }
            0xb903 | 0xb910 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xb904 | 0xb913 | 0xb927 => {
                w.operand_then_internal(2);
                let v = w.read((u32::from(w.cpu.db) << 16) + 0xfe00 + u32::from(w.cpu.y), m);
                w.load(
                    if self.pc == 0xb927 {
                        (w.cpu.a | v) & 255
                    } else {
                        v
                    },
                    m,
                );
                self.pc + 3
            }
            0xb907 => {
                w.dp_store(4, w.cpu.a, 2);
                0xb909
            }
            0xb916 | 0xb91a | 0xb92e | 0xb932 | 0xb95d => {
                w.compare_immediate(
                    match self.pc {
                        0xb916 | 0xb92e => 0x10,
                        0xb95d => 0x2000,
                        _ => 0x1c,
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0xb918 | 0xb91c | 0xb930 | 0xb934 => {
                let take = if matches!(self.pc, 0xb918 | 0xb930) {
                    w.cpu.p & CARRY == 0
                } else {
                    w.cpu.p & CARRY != 0
                };
                if w.branch(take) {
                    if self.pc < 0xb930 {
                        0xb92a
                    } else {
                        0xb941
                    }
                } else {
                    self.pc + 2
                }
            }
            0xb922 | 0xb93a => {
                w.shift_left(1);
                self.pc + 1
            }
            0xb923 | 0xb924 | 0xb93b | 0xb93c => {
                w.drain(6);
                let old = w.cpu.a & 255;
                let v = ((old << 1) | u16::from(w.cpu.p & CARRY != 0)) & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old & 0x80 != 0);
                w.load(v, 1);
                self.pc + 1
            }
            0xb92a => {
                w.dp_store(2, w.cpu.a & 255, 1);
                0xb92c
            }
            0xb92c | 0xb93f | 0xb942 | 0xb950 | 0xb95b => {
                let a = match self.pc {
                    0xb92c | 0xb93f => 4,
                    0xb942 => 2,
                    0xb950 => 0xb2,
                    _ => 0xb4,
                };
                let v = w.dp_read(a, m);
                w.load(
                    if self.pc == 0xb93f {
                        (w.cpu.a | v) & 255
                    } else {
                        v
                    },
                    m,
                );
                self.pc + 2
            }
            0xb941 => {
                w.swap_bytes();
                0xb942
            }
            0xb948 => {
                w.drain(24);
                w.write(0x7f2000 + u32::from(w.cpu.x), w.cpu.a, 2);
                0xb94c
            }
            0xb94c | 0xb94d => {
                w.increment_x();
                self.pc + 1
            }
            0xb94e => {
                w.dp_store(0xb4, w.cpu.x, 2);
                0xb950
            }
            0xb952 => {
                w.drain(16);
                w.adc(4);
                0xb955
            }
            0xb955 => {
                w.dp_store(0xb2, w.cpu.a, 2);
                0xb957
            }
            0xb957 => {
                w.dp_increment(0, 2, false);
                0xb959
            }
            0xb959 | 0xb960 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    if self.pc == 0xb959 {
                        0xb8fa
                    } else {
                        0xb965
                    }
                } else {
                    self.pc + 2
                }
            }
            0xb966 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("basic attributes continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CrystalPegAttributes {
    pc: u16,
    complete: bool,
}
impl Default for CrystalPegAttributes {
    fn default() -> Self {
        Self {
            pc: 0xc22a,
            complete: false,
        }
    }
}
impl CrystalPegAttributes {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x01c22a
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
            (1, self.pc, 0, 0x20)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xc22a | 0xc27a => {
                w.width(0x10, self.pc == 0xc22a);
                self.pc + 2
            }
            0xc22c => {
                w.drain(16);
                w.cpu.x = 0x07ff;
                w.flags(w.cpu.x, 2);
                0xc22f
            }
            0xc22f | 0xc241 | 0xc253 | 0xc265 => {
                w.drain(24);
                let base = match self.pc {
                    0xc22f => 0x7f2000,
                    0xc241 => 0x7f2800,
                    0xc253 => 0x7f3000,
                    _ => 0x7f3800,
                };
                let v = w.read(base + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                self.pc + 4
            }
            0xc233 | 0xc237 | 0xc245 | 0xc249 | 0xc257 | 0xc25b | 0xc269 | 0xc26d => {
                w.compare_immediate(
                    if matches!(self.pc, 0xc233 | 0xc245 | 0xc257 | 0xc269) {
                        0x66
                    } else {
                        0x67
                    },
                    1,
                );
                self.pc + 2
            }
            0xc235 | 0xc247 | 0xc259 | 0xc26b => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    self.pc + 6
                } else {
                    self.pc + 2
                }
            }
            0xc239 | 0xc24b | 0xc25d | 0xc26f => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    self.pc + 8
                } else {
                    self.pc + 2
                }
            }
            0xc23b | 0xc24d | 0xc25f | 0xc271 => {
                w.drain(8);
                w.load((w.cpu.a ^ 1) & 255, 1);
                self.pc + 2
            }
            0xc23d | 0xc24f | 0xc261 | 0xc273 => {
                w.drain(24);
                let base = match self.pc {
                    0xc23d => 0x7f2000,
                    0xc24f => 0x7f2800,
                    0xc261 => 0x7f3000,
                    _ => 0x7f3800,
                };
                w.write(base + u32::from(w.cpu.x), w.cpu.a & 255, 1);
                self.pc + 4
            }
            0xc277 => {
                w.decrement_x();
                0xc278
            }
            0xc278 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    0xc22f
                } else {
                    0xc27a
                }
            }
            0xc27c => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("crystal peg attributes continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
