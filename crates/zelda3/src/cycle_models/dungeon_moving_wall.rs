//! Moving-wall room state and drawing continuations, including saved-tag gates.
use super::{
    dungeon_objects::DungeonObjectTiles, native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonMovingWall {
    pc: u16,
    depth: u8,
    tiles: Option<DungeonObjectTiles>,
    tail: bool,
    complete: bool,
}
impl DungeonMovingWall {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(entry, 0x9190 | 0x921c | 0x9298));
        Self {
            pc: entry,
            depth: 0,
            tiles: None,
            tail: false,
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
        if let Some(tiles) = self.tiles.as_mut() {
            let cycles = tiles.step(cpu, clock, bus);
            self.pc = cpu.pc;
            if tiles.complete() {
                self.tiles = None;
                if self.tail {
                    self.complete = true;
                }
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
            0x9190 | 0x921c | 0x91ee => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                if self.pc == 0x91ee {
                    0x92d1
                } else {
                    0x9298
                }
            }
            0x91f6 | 0x9203 | 0x922a | 0x923e | 0x924b => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                let target = match self.pc {
                    0x9203 | 0x923e => 0x9bd9,
                    0x924b => 0x9210,
                    _ => 0x9216,
                };
                self.tiles = Some(DungeonObjectTiles::new(0x010000 | u32::from(target)).unwrap());
                target
            }
            0x9193 | 0x91b9 | 0x91d9 | 0x91e6 | 0x920e | 0x921f | 0x9249 | 0x9284 | 0x9291
            | 0x92a7 | 0x92ab | 0x92b4 | 0x92b8 | 0x92c0 | 0x92dd | 0x92eb => {
                let (take, target) = match self.pc {
                    0x9193 => (w.cpu.p & 1 != 0, 0x9196),
                    0x921f => (w.cpu.p & 1 != 0, 0x9222),
                    0x91b9 => (w.cpu.p & 2 == 0, 0x91b6),
                    0x91d9 => (w.cpu.p & 2 == 0, 0x91cc),
                    0x91e6 => (w.cpu.p & 2 == 0, 0x91c1),
                    0x920e => (w.cpu.p & 2 == 0, 0x9203),
                    0x9249 => (w.cpu.p & 2 == 0, 0x923e),
                    0x9284 => (w.cpu.p & 2 == 0, 0x9277),
                    0x9291 => (w.cpu.p & 2 == 0, 0x926c),
                    0x92a7 => (w.cpu.p & 1 == 0, 0x92ad),
                    0x92ab => (w.cpu.p & 1 == 0, 0x92ba),
                    0x92b4 => (w.cpu.p & 1 == 0, 0x92cd),
                    0x92b8 => (w.cpu.p & 1 != 0, 0x92cd),
                    0x92c0 => (w.cpu.p & 2 != 0, 0x92cd),
                    0x92dd => (w.cpu.p & 0x80 == 0, 0x92d7),
                    _ => (w.cpu.p & 2 != 0, 0x92f2),
                };
                if w.branch(take) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x9195 | 0x9221 | 0x92cc | 0x92d0 | 0x92fa => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0x9196 | 0x9222 => {
                w.drain(16);
                let v = w.read(0x0428, 2).wrapping_add(1);
                w.drain(6);
                w.flags(v, 2);
                w.write(0x0428, v, 2);
                self.pc + 3
            }
            0x9199 | 0x91a7 | 0x91b4 | 0x91c1 | 0x91fc | 0x922d | 0x9237 | 0x9255 | 0x9261
            | 0x926c | 0x92df | 0x92f2 => {
                let v = w.dp_read(
                    match self.pc {
                        0x9199 | 0x922d => 0xb2,
                        0x91a7 | 0x9255 => 0xb4,
                        0x91c1 | 0x926c => 0x0e,
                        0x92df => 6,
                        0x92f2 => 0x0a,
                        _ => 8,
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0x919b | 0x91a1 | 0x91a9 | 0x922f | 0x924f | 0x9257 => {
                w.shift_left(2);
                self.pc + 1
            }
            0x919c | 0x91ad | 0x91b3 | 0x91d6 | 0x91ed | 0x9202 | 0x920b | 0x9230 | 0x923d
            | 0x9246 | 0x925b | 0x9281 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x919d | 0x91ae | 0x9231 | 0x925c => {
                w.operand_then_internal(2);
                let base = if matches!(self.pc, 0x919d | 0x9231) {
                    0x009b0a
                } else {
                    0x009b12
                };
                let v = w.read(base + u32::from(w.cpu.y), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x91a0 | 0x91bb | 0x9236 => {
                w.drain(6);
                w.push(w.cpu.a, 2);
                self.pc + 1
            }
            0x91e8 | 0x91f9 | 0x924e => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                self.pc + 1
            }
            0x91a2 | 0x91d3 | 0x91ff | 0x9208 | 0x923a | 0x9243 | 0x9250 | 0x9264 | 0x927e => {
                w.drain(16);
                w.adc(match self.pc {
                    0x91a2 | 0x9250 => 4,
                    0x91d3 | 0x927e => 0x0080,
                    0x91ff | 0x923a => 0x0180,
                    0x9264 => 6,
                    _ => 0x0100,
                });
                self.pc + 3
            }
            0x91a5 | 0x91b1 | 0x91bc | 0x91c3 | 0x91eb | 0x91fa | 0x9234 | 0x9253 | 0x925f
            | 0x9267 | 0x926e | 0x92e5 => {
                w.dp_store(
                    match self.pc {
                        0x91b1 | 0x925f => 0x0c,
                        0x91bc | 0x91eb | 0x9267 => 6,
                        0x91c3 | 0x926e | 0x92e5 => 0x0a,
                        _ => 0x0e,
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 2
            }
            0x91aa | 0x9258 | 0x92f7 => {
                w.drain(16);
                w.write(if self.pc == 0x92f7 { 0x042a } else { 0x041e }, w.cpu.a, 2);
                self.pc + 3
            }
            0x91b6 | 0x91b7 | 0x91e9 | 0x91ea => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1), 2);
                self.pc + 1
            }
            0x91b8 => {
                w.decrement_y();
                0x91b9
            }
            0x91be | 0x91f3 | 0x9227 | 0x9269 | 0x92d1 => {
                w.drain(16);
                w.cpu.x = match self.pc {
                    0x91f3 => 0x072a,
                    0x9227 => 0x075a,
                    0x92d1 => 0x007e,
                    _ => 0x03d8,
                };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x91c5 | 0x91f1 | 0x9225 | 0x9270 | 0x9293 => {
                w.cpu.y = w.dp_read(
                    if matches!(self.pc, 0x91f1 | 0x9225) {
                        8
                    } else {
                        6
                    },
                    2,
                );
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0x91c7 | 0x91cc | 0x91db | 0x9272 | 0x9277 | 0x9286 => {
                w.operand_then_internal(2);
                let offset = match self.pc {
                    0x91cc | 0x9277 => 2,
                    0x91db | 0x9286 => 4,
                    _ => 0,
                };
                let v = w.read(0x009b52 + u32::from(w.cpu.x) + offset, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x91ca | 0x91cf | 0x91de | 0x9275 | 0x927a | 0x9289 => {
                let a = w.indirect_y_address(if matches!(self.pc, 0x91ca | 0x9275) {
                    0xbf
                } else {
                    0xcb
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x91d1 | 0x9206 | 0x9241 | 0x927c | 0x92e7 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x91d2 | 0x91fe | 0x9207 | 0x9239 | 0x9242 | 0x9263 | 0x927d => {
                w.clear_carry();
                self.pc + 1
            }
            0x91d7 | 0x91e0 | 0x91e2 | 0x91e4 | 0x920c | 0x9247 | 0x9282 | 0x928b | 0x928d
            | 0x928f => {
                let (address, inc) = match self.pc {
                    0x91e0 | 0x91e2 | 0x928b | 0x928d => (6, true),
                    0x91e4 | 0x928f => (0x0c, false),
                    0x920c | 0x9247 => (0x0e, false),
                    _ => (0x0a, false),
                };
                w.dp_increment(address, 2, inc);
                self.pc + 2
            }
            0x9210 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                0x9211
            }
            0x9295 => {
                w.drain(16);
                0x92d1
            }
            0x9298 | 0x929b => {
                w.drain(16);
                w.write(if self.pc == 0x9298 { 0x041c } else { 0x041a }, 0, 2);
                self.pc + 3
            }
            0x929e | 0x92ca | 0x92cd => {
                w.width(
                    if self.pc == 0x92ca { 0x31 } else { 0x30 },
                    self.pc != 0x929e,
                );
                self.pc + 2
            }
            0x92a0 => {
                w.drain(8);
                w.cpu.x = 0;
                w.flags(0, 1);
                0x92a2
            }
            0x92a2 => {
                w.drain(6);
                w.cpu.y = w.cpu.x;
                w.flags(w.cpu.y, 1);
                0x92a3
            }
            0x92a3 | 0x92b0 => {
                let v = w.dp_read(if self.pc == 0x92a3 { 0xae } else { 0xaf }, 1);
                w.load(v, 1);
                self.pc + 2
            }
            0x92a5 | 0x92a9 | 0x92b2 | 0x92b6 => {
                w.compare_immediate(
                    if matches!(self.pc, 0x92a5 | 0x92b2) {
                        0x1c
                    } else {
                        0x20
                    },
                    1,
                );
                self.pc + 2
            }
            0x92ad => {
                w.drain(8);
                w.cpu.y = 2;
                w.flags(2, 1);
                0x92af
            }
            0x92af => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & 255;
                w.flags(w.cpu.x, 1);
                0x92b0
            }
            0x92ba => {
                w.drain(16);
                let v = w.read(0x0403, 1);
                w.load(v, 1);
                0x92bd
            }
            0x92bd => {
                w.drain(16);
                let v = w.read(0x0098c7 + u32::from(w.cpu.y), 1);
                w.load(w.cpu.a & v, 1);
                0x92c0
            }
            0x92c2 | 0x92c7 => {
                w.drain(16);
                w.write(if self.pc == 0x92c2 { 0x046c } else { 0x0414 }, 0, 1);
                self.pc + 3
            }
            0x92c5 => {
                w.operand_then_internal(1);
                w.write(0xae + u32::from(w.cpu.x), 0, 1);
                0x92c7
            }
            0x92cf => {
                w.drain(6);
                w.cpu.p |= 1;
                0x92d0
            }
            0x92d4 | 0x92ed => {
                w.immediate(if self.pc == 0x92d4 { 0x01ec } else { 0x0400 }, 2);
                self.pc + 3
            }
            0x92d7 => {
                w.drain(24);
                w.write(0x7ec880 + u32::from(w.cpu.x), w.cpu.a, 2);
                0x92db
            }
            0x92db | 0x92dc => {
                w.decrement_x();
                self.pc + 1
            }
            0x92e1 | 0x92e8 => {
                w.mask(if self.pc == 0x92e1 { 0x003f } else { 0x0040 }, 2);
                self.pc + 3
            }
            0x92e4 => {
                w.drain(6);
                let v = w.cpu.a;
                w.cpu.p = (w.cpu.p & !1) | (v as u8 & 1);
                w.load(v >> 1, 2);
                0x92e5
            }
            0x92f0 => {
                let old = w.dp_read(0x0a, 2);
                w.cpu.p = (w.cpu.p & !2) | if old & w.cpu.a == 0 { 2 } else { 0 };
                w.drain(6);
                w.write(0x0a, old | w.cpu.a, 2);
                0x92f2
            }
            0x92f4 => {
                w.drain(16);
                w.load(w.cpu.a | 0x1000, 2);
                0x92f7
            }
            _ => unreachable!("moving wall {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        // West wall's final corner shares the original return frame.
        if self.pc == 0x9211 {
            assert_eq!(self.depth, 0);
            self.tail = true;
            self.tiles = Some(DungeonObjectTiles::new(0x010000 | u32::from(self.pc)).unwrap());
        }
        Some(w.cycles)
    }
}
