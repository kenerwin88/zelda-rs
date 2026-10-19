//! Retained overworld animated-sheet caller. Decompression and plane
//! conversion own their nested near frames on the same hardware clock.
use super::{
    item_graphics::ItemConversion,
    native_decompress::NativeDecompression,
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
enum Phase {
    Caller(u8),
    Decompressing(u8),
    Converting(u8),
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnimatedOverworldGraphics {
    phase: Phase,
    decompression: Option<NativeDecompression>,
    conversion: Option<ItemConversion>,
}
impl Default for AnimatedOverworldGraphics {
    fn default() -> Self {
        Self {
            phase: Phase::Caller(0),
            decompression: None,
            conversion: None,
        }
    }
}
impl AnimatedOverworldGraphics {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    fn pc(i: u8) -> u16 {
        [
            0xd394, 0xd395, 0xd396, 0xd397, 0xd398, 0xd39b, 0xd39d, 0xd39f, 0xd3a2, 0xd3a5, 0xd3a8,
            0xd3aa, 0xd3ab, 0xd3ac, 0xd3af, 0xd3b1, 0xd3b3, 0xd3b6, 0xd3b9, 0xd3bc, 0xd3bf, 0xd3c2,
            0xd3c4, 0xd3c5,
        ][usize::from(i)]
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        match self.phase {
            Phase::Complete => return None,
            Phase::Decompressing(next) => {
                let child = self.decompression.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.decompression = None;
                    self.phase = Phase::Caller(next);
                    assert_eq!(cpu.pc, Self::pc(next));
                }
                return cycles;
            }
            Phase::Converting(next) => {
                let child = self.conversion.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.conversion = None;
                    self.phase = Phase::Caller(next);
                    assert_eq!(cpu.pc, Self::pc(next));
                }
                return cycles;
            }
            _ => {}
        }
        let Phase::Caller(i) = self.phase else {
            unreachable!()
        };
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (0, Self::pc(i), 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = i + 1;
        match i {
            0 | 1 => {
                w.drain(6);
                w.push(u16::from(if i == 0 { w.cpu.db } else { w.cpu.bank }), 1);
            }
            2 | 22 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
            }
            3 => {
                w.drain(6);
                w.push(w.cpu.y, 1);
            }
            4 | 13 => {
                let child = NativeDecompression::new(false);
                w.item_call();
                w.cpu.pc = child.routine() as u16;
                self.decompression = Some(child);
                self.phase = Phase::Decompressing(next);
                return Some(w.cycles);
            }
            5 | 14 => w.width(0x30, true),
            6 | 15 => {
                let value = w.dp_read(0, 2);
                w.load(value, 2);
            }
            7 | 16 => w.item_y(if i == 7 { 64 } else { 32 }, 2),
            8 | 17 => w.item_x(if i == 8 { 0x1680 } else { 0x1e80 }, 2),
            9 | 18 => {
                let child = ItemConversion::new(false);
                w.item_call();
                w.cpu.pc = child.routine() as u16;
                self.conversion = Some(child);
                self.phase = Phase::Converting(next);
                return Some(w.cycles);
            }
            10 | 21 => w.width(0x30, false),
            11 => {
                w.drain(12);
                w.cpu.y = w.pull(1);
                w.flags(w.cpu.y, 1);
            }
            12 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & 255;
                w.flags(w.cpu.y, 1);
            }
            19 => w.immediate(0x3c00, 2),
            20 => {
                w.drain(16);
                w.write(0x134, w.cpu.a, 2);
            }
            23 => {
                w.rtl();
                self.phase = Phase::Complete;
                return Some(w.cycles);
            }
            _ => unreachable!(),
        }
        self.phase = Phase::Caller(next);
        w.cpu.pc = Self::pc(next);
        Some(w.cycles)
    }
}
