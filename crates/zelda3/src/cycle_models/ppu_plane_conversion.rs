//! Live planar conversion into PPU ports. Scratch, loop counters and the
//! accumulator remain owned by the interrupted instruction sequence.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaneLoop {
    HighSheet,
    HighCommon,
    Low,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PpuPlaneConversion {
    kind: PlaneLoop,
    instruction: u8,
    complete: bool,
}
impl PpuPlaneConversion {
    pub(crate) fn new(kind: PlaneLoop) -> Self {
        Self {
            kind,
            instruction: 0,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.kind {
            PlaneLoop::HighSheet => 0xe5af,
            PlaneLoop::HighCommon => 0xe6cd,
            PlaneLoop::Low => 0xe63c,
        }
    }
    fn pc(&self) -> u16 {
        let i = self.instruction;
        if self.kind == PlaneLoop::Low {
            match i {
                0..=31 => 0xe63c + u16::from(i / 4) * 9 + [0, 2, 5, 7][usize::from(i % 4)],
                32 => 0xe684,
                33..=48 => {
                    0xe686 + u16::from((i - 33) / 4) * 10 + [0, 2, 5, 8][usize::from((i - 33) % 4)]
                }
                49..=53 => [0xe6ae, 0xe6af, 0xe6b1, 0xe6b2, 0xe6b4][usize::from(i - 49)],
                _ => unreachable!(),
            }
        } else {
            [
                0xe5af, 0xe5b1, 0xe5b3, 0xe5b6, 0xe5b7, 0xe5b9, 0xe5bc, 0xe5be, 0xe5c0, 0xe5c2,
                0xe5c3, 0xe5c4, 0xe5c6, 0xe5c9, 0xe5ca, 0xe5cc, 0xe5cf, 0xe5d1, 0xe5d3, 0xe5d5,
                0xe5d6, 0xe5d7, 0xe5d9, 0xe5db, 0xe5dd, 0xe5e0, 0xe5e2, 0xe5e4, 0xe5e5, 0xe5e7,
                0xe5ea, 0xe5ec, 0xe5ed, 0xe5ee, 0xe5f0, 0xe5f3, 0xe5f5, 0xe5f7, 0xe5f8, 0xe5fa,
                0xe5fd, 0xe5ff, 0xe600, 0xe601, 0xe603, 0xe604, 0xe606,
            ][usize::from(i)]
                + if self.kind == PlaneLoop::HighCommon {
                    0x11e
                } else {
                    0
                }
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
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x30),
            (0, self.pc(), 0, 0x10)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let i = self.instruction;
        let mut next = i + 1;
        if self.kind == PlaneLoop::Low {
            match i {
                0..=31 => match i % 4 {
                    0 => {
                        let value = w.indirect_read(0, 2);
                        w.load(value, 2);
                    }
                    1 => {
                        w.drain(16);
                        w.write_io(0x2118, w.cpu.a, 2);
                    }
                    _ => w.dp_increment(0, 2, true),
                },
                32 => w.item_x(1, 1),
                33..=48 => match (i - 33) % 4 {
                    0 => {
                        let value = w.indirect_read(0, 2);
                        w.load(value, 2);
                    }
                    1 => w.mask(255, 2),
                    2 => {
                        w.drain(16);
                        w.write_io(0x2118, w.cpu.a, 2);
                    }
                    _ => w.dp_increment(0, 2, true),
                },
                49 => decrement_index(&mut w, false),
                50 => {
                    if w.branch(w.cpu.p & NEGATIVE == 0) {
                        next = 33;
                    }
                }
                51 => decrement_index(&mut w, true),
                52 => {
                    if w.branch(w.cpu.p & NEGATIVE == 0) {
                        next = 0;
                    }
                }
                53 => {
                    w.width(0x20, false);
                    self.complete = true;
                    w.cpu.pc = 0xe6b6;
                }
                _ => unreachable!(),
            }
        } else {
            match i {
                0 | 22 => w.item_x(14, 1),
                1 | 11 | 23 | 33 => {
                    let value = w.indirect_read(0, 2);
                    w.load(value, 2);
                }
                2 | 12 | 29 | 39 => {
                    w.drain(16);
                    w.write_io(0x2118, w.cpu.a, 2);
                }
                3 | 13 | 27 | 37 => w.swap_bytes(),
                4 | 14 => {
                    let value = w.indirect_read(0, 2);
                    w.load(w.cpu.a | value, 2);
                }
                5 | 15 | 24 | 34 => w.mask(255, 2),
                6 | 16 => {
                    w.operand_then_internal(1);
                    w.write(0xbf + u32::from(w.cpu.x), w.cpu.a, 2);
                }
                7 | 8 | 17 | 18 | 30 | 40 => w.dp_increment(0, 2, true),
                9 | 10 | 19 | 20 | 31 | 32 | 41 | 42 => decrement_index(&mut w, false),
                21 => {
                    if w.branch(w.cpu.p & NEGATIVE == 0) {
                        next = 1;
                    }
                }
                25 | 35 => w.dp_store(0xbd, w.cpu.a, 2),
                26 | 36 => {
                    w.operand_then_internal(1);
                    let value = w.read(0xbf + u32::from(w.cpu.x), 2);
                    w.load(w.cpu.a | value, 2);
                }
                28 | 38 => {
                    let value = w.dp_read(0xbd, 2);
                    w.load(w.cpu.a | value, 2);
                }
                43 => {
                    if w.branch(w.cpu.p & NEGATIVE == 0) {
                        next = 23;
                    }
                }
                44 => decrement_index(&mut w, true),
                45 => {
                    let taken = if self.kind == PlaneLoop::HighCommon {
                        w.cpu.p & ZERO == 0
                    } else {
                        w.cpu.p & NEGATIVE == 0
                    };
                    if w.branch(taken) {
                        next = 0;
                    }
                }
                46 => {
                    w.width(0x20, false);
                    self.complete = true;
                    w.cpu.pc = if self.kind == PlaneLoop::HighCommon {
                        0xe726
                    } else {
                        0xe608
                    };
                }
                _ => unreachable!(),
            }
        }
        if !self.complete {
            self.instruction = next;
            w.cpu.pc = self.pc();
        }
        Some(w.cycles)
    }
}
pub(super) fn decrement_index<B: WorkBus>(w: &mut Work<'_, B>, y: bool) {
    w.drain(6);
    let value = (if y { w.cpu.y } else { w.cpu.x }).wrapping_sub(1) & 255;
    if y {
        w.cpu.y = value;
    } else {
        w.cpu.x = value;
    }
    w.flags(value, 1);
}
