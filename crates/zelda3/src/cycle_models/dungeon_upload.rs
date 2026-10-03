//! Room-buffer preparation and its shared tilemap DMA retain the source's live
//! pointers, call frames, transfer publication and channel-end clock.
#[cfg(test)]
#[path = "dungeon_upload_tests.rs"]
mod tests;
use super::{
    dungeon_room::DungeonRoom,
    intro_clear::Access,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::{CpuMasterTimeline, DmaState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TilemapPreparation {
    pc: u16,
    bg1: bool,
    complete: bool,
}
impl TilemapPreparation {
    pub(crate) fn new(bg1: bool) -> Self {
        Self {
            pc: if bg1 { 0x91d3 } else { 0x913f },
            bg1,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.bg1 {
            0x0091d3
        } else {
            0x00913f
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
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 8),
            (0, self.pc, 0, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x913f | 0x91d3 => {
                w.width(0x31, true);
                self.pc + 2
            }
            0x91ab | 0x923f => {
                w.width(0x30, false);
                self.pc + 2
            }
            0x9141 | 0x91d5 => {
                w.drain(16);
                let v = w.read(0x0418, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x9144 | 0x91d8 | 0x9194 | 0x9228 => {
                w.mask(
                    if matches!(self.pc, 0x9144 | 0x91d8) {
                        15
                    } else {
                        63
                    },
                    2,
                );
                self.pc + 3
            }
            0x9147 | 0x91db => {
                w.drain(16);
                let v = w.read(0x045c, 2);
                w.adc(v);
                self.pc + 3
            }
            0x914a | 0x91de => {
                w.drain(6);
                w.push(w.cpu.a, 2);
                self.pc + 1
            }
            0x914b | 0x91df => {
                w.shift_left(2);
                self.pc + 1
            }
            0x914c | 0x919e | 0x91e0 | 0x9232 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x914d | 0x91e1 => {
                w.operand_then_internal(2);
                w.cpu.x = w.read(u32::from(0x910fu16.wrapping_add(w.cpu.y)), 2);
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x9150 | 0x91e4 => {
                w.drain(16);
                w.cpu.y = 0;
                w.flags(0, 2);
                self.pc + 3
            }
            0x9153 | 0x915a | 0x9161 | 0x9168 | 0x916f | 0x9176 | 0x917d | 0x9184 | 0x91e7
            | 0x91ee | 0x91f5 | 0x91fc | 0x9203 | 0x920a | 0x9211 | 0x9218 => {
                let offset = match self.pc {
                    0x9153 | 0x91e7 => 0,
                    0x915a | 0x91ee => 2,
                    0x9161 | 0x91f5 => 0x80,
                    0x9168 | 0x91fc => 0x82,
                    0x916f | 0x9203 => 0x100,
                    0x9176 | 0x920a => 0x102,
                    0x917d | 0x9211 => 0x180,
                    _ => 0x182,
                };
                w.drain(24);
                let v = w.read(
                    (if self.bg1 { 0x7e4000 } else { 0x7e2000 }) + offset + u32::from(w.cpu.x),
                    2,
                );
                w.load(v, 2);
                self.pc + 4
            }
            0x9157 | 0x915e | 0x9165 | 0x916c | 0x9173 | 0x917a | 0x9181 | 0x9188 | 0x91eb
            | 0x91f2 | 0x91f9 | 0x9200 | 0x9207 | 0x920e | 0x9215 | 0x921c => {
                let base: u16 = match self.pc {
                    0x9157 | 0x91eb => 0x1000,
                    0x915e | 0x91f2 => 0x1002,
                    0x9165 | 0x91f9 => 0x1040,
                    0x916c | 0x9200 => 0x1042,
                    0x9173 | 0x9207 => 0x1080,
                    0x917a | 0x920e => 0x1082,
                    0x9181 | 0x9215 => 0x10c0,
                    _ => 0x10c2,
                };
                w.operand_then_internal(2);
                w.write(u32::from(base.wrapping_add(w.cpu.y)), w.cpu.a, 2);
                self.pc + 3
            }
            0x918b..=0x918e | 0x921f..=0x9222 => {
                w.increment_x();
                self.pc + 1
            }
            0x918f..=0x9192 | 0x9223..=0x9226 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x9193 | 0x9199 | 0x9227 | 0x922d => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x9197 | 0x91a8 | 0x922b | 0x923c => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    if self.bg1 {
                        0x91e7
                    } else {
                        0x9153
                    }
                } else {
                    self.pc + 2
                }
            }
            0x919a | 0x91a0 | 0x91b0 | 0x922e | 0x9234 | 0x9244 => {
                w.clear_carry();
                self.pc + 1
            }
            0x919b | 0x91a1 | 0x922f | 0x9235 => {
                w.drain(16);
                w.adc(if matches!(self.pc, 0x919b | 0x922f) {
                    0xc0
                } else {
                    0x1c0
                });
                self.pc + 3
            }
            0x919f | 0x9233 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x91a4 | 0x9238 => {
                w.drain(6);
                w.cpu.x = w.cpu.a;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x91a5 | 0x9239 => {
                w.drain(16);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= 0x0800);
                w.flags(w.cpu.y.wrapping_sub(0x0800), 2);
                self.pc + 3
            }
            0x91aa | 0x923e => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x91ad => {
                w.drain(16);
                let v = w.read(0x045c, 1);
                w.load(v, 1);
                0x91b0
            }
            0x91b1 | 0x9245 => {
                w.drain(8);
                w.adc_byte(if self.pc == 0x91b1 { 4 } else { 0x10 });
                self.pc + 2
            }
            0x91b3 | 0x91b9 | 0x91c0 | 0x9247 | 0x924e => {
                let a = match self.pc {
                    0x91b3 => 0x045c,
                    0x91b9 | 0x9247 => 0x0116,
                    _ => 0x0710,
                };
                w.drain(16);
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 3
            }
            0x91b6 | 0x9241 => {
                w.drain(16);
                if 0x2f + w.cpu.x > 255 { w.drain(6); }
                let v = w.read(u32::from(0x912fu16.wrapping_add(w.cpu.x)), 1);
                w.load(v, 1);
                self.pc + 3
            }
            0x91bc | 0x924a => {
                w.immediate(1, 1);
                self.pc + 2
            }
            0x91be | 0x924c => {
                w.dp_store(0x17, w.cpu.a & 255, 1);
                self.pc + 2
            }
            0x91c3 | 0x9251 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("tilemap preparation continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TilemapUpload {
    pc: u16,
    far: bool,
    complete: bool,
    dma_end: Option<u64>,
}
impl TilemapUpload {
    pub(crate) fn new(far: bool) -> Self {
        Self {
            pc: if far { 0x90e3 } else { 0x8cb0 },
            far,
            complete: false,
            dma_end: None,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.far {
            0x0090e3
        } else {
            0x008cb0
        }
    }
    pub(crate) fn take_dma_end(&mut self) -> Option<u64> {
        self.dma_end.take()
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x18),
            (0, self.pc, 0, 0, 0x10)
        );
        self.dma_end = None;
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x90e3 => {
                w.operand_then_internal(2);
                w.push(0x90e5, 2);
                0x8cb0
            }
            0x8cb0 => {
                w.drain(16);
                w.cpu.x = w.read(0x0116, 1);
                w.flags(w.cpu.x, 1);
                0x8cb3
            }
            0x8cb3 => {
                w.drain(16);
                if 0x88 + w.cpu.x > 255 { w.drain(6); }
                let v = w.read(u32::from(0x9888u16.wrapping_add(w.cpu.x)), 1);
                w.load(v, 1);
                0x8cb6
            }
            0x8cb6 | 0x8cb9 | 0x8cc1 | 0x8cc7 | 0x8ccd | 0x8cd3 => {
                let (a, value, width): (u16, u16, u8) = match self.pc {
                    0x8cb6 => (0x2117, w.cpu.a & 255, 1),
                    0x8cb9 => (0x4304, 0, 1),
                    0x8cc1 => (0x2115, w.cpu.a, 2),
                    0x8cc7 => (0x4300, w.cpu.a, 2),
                    0x8ccd => (0x4302, w.cpu.a, 2),
                    _ => (0x4305, w.cpu.a, 2),
                };
                w.drain(16);
                for byte in 0..width {
                    let value = (value >> (byte * 8)) & 255;
                    w.bus.access(
                        w.clock.clock_master_cycles(),
                        Access::Write(u32::from(a + u16::from(byte)), value, 1),
                    );
                    if a >= 0x4300 {
                        dma.write_register(a + u16::from(byte), value as u8);
                    }
                    w.drain(6);
                }
                self.pc + 3
            }
            0x8cbc | 0x8cde => {
                w.width(0x20, self.pc == 0x8cbc);
                self.pc + 2
            }
            0x8cbe | 0x8cc4 | 0x8cca | 0x8cd0 => {
                w.immediate(
                    match self.pc {
                        0x8cbe => 0x0080,
                        0x8cc4 => 0x1801,
                        0x8cca => 0x1000,
                        _ => 0x0800,
                    },
                    2,
                );
                self.pc + 3
            }
            0x8cd6 => {
                w.drain(8);
                w.cpu.y = 1;
                w.flags(1, 1);
                0x8cd8
            }
            0x8cd8 => {
                w.drain(16);
                w.bus.access(
                    w.clock.clock_master_cycles(),
                    Access::Write(0x420b, w.cpu.y & 255, 1),
                );
                let start = w.clock.clock_master_cycles();
                // Hardware events must publish before the next DMA byte, while
                // the bus still exposes the partially uploaded tilemap.
                let bus = std::cell::RefCell::new(&mut w.bus);
                self.dma_end = dma.run_synchronous_forward_ppu_with_timed_events(
                    w.clock,
                    w.cpu.y as u8,
                    |at, _, source, register| {
                        let mut bus = bus.borrow_mut();
                        let value = bus.access(at, Access::Read(source, 1));
                        bus.access(at, Access::Write(0x2100 + u32::from(register), value, 1));
                    },
                    |event, at| bus.borrow_mut().timeline_event(event, at),
                );
                w.cycles += w.clock.clock_master_cycles() - start;
                0x8cdb
            }
            0x8cdb | 0x8ce0 => {
                w.drain(16);
                w.write(
                    if self.pc == 0x8cdb { 0x1000 } else { 0x0710 },
                    0,
                    if self.pc == 0x8cdb { 2 } else { 1 },
                );
                self.pc + 3
            }
            0x8ce3 => {
                w.rts();
                self.complete = !self.far;
                w.cpu.pc
            }
            0x90e6 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("tilemap upload continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Room(DungeonRoom),
    Preparation(TilemapPreparation),
    Upload(TilemapUpload),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonLoadAndDraw {
    pc: u16,
    child: Option<Child>,
    complete: bool,
}
impl Default for DungeonLoadAndDraw {
    fn default() -> Self {
        Self {
            pc: 0xc57b,
            child: None,
            complete: false,
        }
    }
}
impl DungeonLoadAndDraw {
    /// The caller and its preparation/DMA children share the upload buffer.
    /// Room construction retains its separate object and header owners.
    pub(crate) fn upload_active(&self) -> bool {
        !matches!(self.child, Some(Child::Room(_)))
    }

    pub(crate) fn object_tiles_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.object_tiles_active())
    }

    pub(crate) fn saved_objects_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.saved_objects_active())
    }

    pub(crate) fn door_state_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.door_state_active())
    }

    pub(crate) fn object_parser_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.object_parser_active())
    }

    pub(crate) fn floors_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.floors_active())
    }

    pub(crate) fn room_parent_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.parent_active())
    }

    pub(crate) fn header_active(&self) -> bool {
        matches!(&self.child, Some(Child::Room(work)) if work.header_active())
    }

    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match &self.child {
            Some(Child::Room(w)) => w.routine(),
            Some(Child::Preparation(w)) => w.routine(),
            Some(Child::Upload(w)) => w.routine(),
            _ => 0x02c57b,
        }
    }
    pub(crate) fn take_dma_end(&mut self) -> Option<u64> {
        match &mut self.child {
            Some(Child::Upload(w)) => w.take_dma_end(),
            _ => None,
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        if let Some(child) = self.child.as_mut() {
            let (cycles, done) = match child {
                Child::Room(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Preparation(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Upload(w) => {
                    let c = w.step(cpu, clock, dma, bus);
                    (c, w.complete())
                }
            };
            if done {
                self.child = None;
                assert_eq!((cpu.bank, cpu.pc), (2, self.pc));
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
            (2, self.pc, 0, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xc57b => {
                let v = w.dp_read(0x9b, 1);
                w.load(v, 1);
                0xc57d
            }
            0xc57d => {
                w.drain(6);
                w.push(w.cpu.a & 255, 1);
                0xc57e
            }
            0xc57e => {
                w.drain(16);
                w.bus.access(w.clock.clock_master_cycles(), Access::Write(0x420c, 0, 1));
                for channel in &mut dma.channel {
                    channel.hdma_active = false;
                }
                w.drain(6);
                0xc581
            }
            0xc581 | 0xc5aa | 0xc5af => {
                w.dp_store(
                    match self.pc {
                        0xc581 => 0x9b,
                        0xc5aa => 0x17,
                        _ => 0xb0,
                    },
                    0,
                    1,
                );
                self.pc + 2
            }
            0xc583 | 0xc590 | 0xc594 | 0xc598 | 0xc59c => {
                let target = match self.pc {
                    0xc583 => 0x01873a,
                    0xc590 => 0x0091d3,
                    0xc598 => 0x00913f,
                    _ => 0x0090e3,
                };
                w.operand_then_internal(3);
                w.push(2, 1);
                w.push(self.pc + 3, 2);
                w.cpu.bank = (target >> 16) as u8;
                w.cpu.pc = target as u16;
                self.child = Some(match target {
                    0x01873a => Child::Room(DungeonRoom::default()),
                    0x0091d3 | 0x00913f => {
                        Child::Preparation(TilemapPreparation::new(target == 0x0091d3))
                    }
                    _ => Child::Upload(TilemapUpload::new(true)),
                });
                self.pc + 4
            }
            0xc587 | 0xc58a | 0xc58d | 0xc5ac => {
                w.drain(16);
                w.write(
                    match self.pc {
                        0xc587 => 0x0418,
                        0xc58a => 0x045c,
                        _ => 0x0200,
                    },
                    0,
                    1,
                );
                self.pc + 3
            }
            0xc5a0 => {
                w.drain(16);
                let v = w.read(0x045c, 1);
                w.load(v, 1);
                0xc5a3
            }
            0xc5a3 => {
                w.compare_immediate(0x10, 1);
                0xc5a5
            }
            0xc5a5 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0xc590
                } else {
                    0xc5a7
                }
            }
            0xc5a7 => {
                w.drain(12);
                let v = w.pull(1);
                w.load(v, 1);
                0xc5a8
            }
            0xc5a8 => {
                w.dp_store(0x9b, w.cpu.a & 255, 1);
                0xc5aa
            }
            0xc5b1 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("load and draw continuation {:04x}", self.pc),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}
