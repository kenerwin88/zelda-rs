//! Room stream and object/door decode through the real tail dispatch.
//! A dispatched drawer must retire its own RTS before the parser can resume.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DrawReturn {
    pc: u16,
    sp: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonObjects {
    pc: u16,
    draw_return: Option<DrawReturn>,
    waiting: bool,
    complete: bool,
}
impl Default for DungeonObjects {
    fn default() -> Self {
        Self {
            pc: 0x88e4,
            draw_return: None,
            waiting: false,
            complete: false,
        }
    }
}
impl DungeonObjects {
    pub(crate) fn routine(&self) -> u32 {
        0x0188e4
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn pending_draw(&self) -> Option<u32> {
        self.waiting.then_some(0x010000 | u32::from(self.pc))
    }
    pub(crate) fn resume_after_draw(&mut self, cpu: &NativeNmiContext) {
        assert!(self.waiting, "object parser is not waiting for a drawer");
        let ret = self.draw_return.take().expect("drawer return frame");
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.sp),
            (1, ret.pc, ret.sp),
            "drawer must retire its actual RTS"
        );
        self.pc = ret.pc;
        self.waiting = false;
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete || self.waiting {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x18),
            (1, self.pc, 0, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 == 0 { 2 } else { 1 };
        w.fetch();
        self.pc = match self.pc {
            0x88e4 | 0x88e6 | 0x8964 | 0x8966 => {
                w.dp_store(
                    match self.pc {
                        0x88e4 => 0xb2,
                        0x88e6 => 0xb4,
                        0x8964 => 3,
                        _ => 5,
                    },
                    0,
                    m,
                );
                self.pc + 2
            }
            0x88e8 | 0x8902 | 0x897e | 0x89b3 | 0x89d7 => {
                let v = w.dp_read(
                    if matches!(self.pc, 0x88e8 | 0x8902) {
                        0xba
                    } else {
                        8
                    },
                    2,
                );
                w.cpu.y = v;
                w.flags(v, 2);
                self.pc + 2
            }
            0x88ea | 0x8904 | 0x8954 | 0x8991 | 0x899c => {
                let a = w.indirect_y_address(0xb7);
                let v = w.read(a, m);
                w.load(v, m);
                self.pc + 2
            }
            0x88ec | 0x8906 | 0x88f3 | 0x8940 | 0x896d => {
                w.compare_immediate(
                    match self.pc {
                        0x88ec | 0x8906 => 0xffff,
                        0x88f3 => 0xfff0,
                        0x8940 => 0xfc,
                        _ => 0x01f0,
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0x88ef | 0x88f6 | 0x8909 | 0x8942 | 0x8970 | 0x88fb | 0x8914 => {
                let (taken, target) = match self.pc {
                    0x88ef | 0x8909 => (w.cpu.p & ZERO != 0, 0x88fd),
                    0x88f6 => (w.cpu.p & ZERO != 0, 0x88fe),
                    0x8942 => (w.cpu.p & ZERO != 0, 0x8983),
                    0x8970 => (w.cpu.p & CARRY != 0, 0x89b8),
                    0x88fb => (true, 0x88e4),
                    _ => (true, 0x8902),
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x88f1 | 0x890b | 0x891c | 0x8924 | 0x8926 | 0x8933 | 0x8944 | 0x894a | 0x8950
            | 0x8956 | 0x8962 | 0x8977 | 0x898e | 0x899a | 0x89ac | 0x89be | 0x89d0 => {
                let a = match self.pc {
                    0x88f1 | 0x890b => 0,
                    0x891c => 2,
                    0x8924 => 0x0a,
                    0x8926 | 0x8956 | 0x89be => 4,
                    0x8933 | 0x8977 | 0x89ac | 0x89d0 => 0x0e,
                    0x8944 | 0x898e | 0x899a => 8,
                    0x894a => 0xb2,
                    0x8950 => 0xb4,
                    _ => 9,
                };
                w.dp_store(a, w.cpu.a & if m == 1 { 255 } else { 65535 }, m);
                self.pc + 2
            }
            0x88f8 | 0x890d => {
                assert!(self.draw_return.is_none());
                self.draw_return = Some(DrawReturn {
                    pc: self.pc + 3,
                    sp: w.cpu.sp,
                });
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                if self.pc == 0x88f8 {
                    0x893c
                } else {
                    0x8916
                }
            }
            0x88fd => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            0x88fe | 0x8900 | 0x8910 | 0x8912 => {
                w.dp_increment(0xba, 2, true);
                self.pc + 2
            }
            0x8916 | 0x8921 | 0x892a | 0x893e | 0x8948 | 0x894e | 0x8988 | 0x8994 | 0x899f
            | 0x89b8 => {
                w.mask(
                    match self.pc {
                        0x8916 => 0xf0,
                        0x8921 => 255,
                        0x892a | 0x8948 | 0x894e => 3,
                        0x893e => 0xfc,
                        0x8988 => 0x03f0,
                        0x8994 => 0x0fc0,
                        0x899f => 0x3f,
                        _ => 14,
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0x8919..=0x891b | 0x895d..=0x895f | 0x898b..=0x898d => {
                w.drain(6);
                let v = w.cpu.a & if m == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (v as u8 & 1);
                w.load(v >> 1, m);
                self.pc + 1
            }
            0x891e | 0x8928 | 0x8937 | 0x8946 | 0x894c | 0x895b | 0x896a | 0x8985 | 0x89c0
            | 0x89c8 => {
                let a = match self.pc {
                    0x891e | 0x8928 | 0x8946 | 0x8985 => 0,
                    0x894c | 0x895b => 1,
                    0x89c0 => 0xb4,
                    _ => 4,
                };
                let v = w.dp_read(a, m);
                w.load(v, m);
                self.pc + 2
            }
            0x8920 | 0x8987 | 0x8993 | 0x899e => {
                w.swap_bytes();
                self.pc + 1
            }
            0x892d | 0x896c | 0x8997 | 0x89a6 | 0x89bb..=0x89bd | 0x89c2 | 0x89c3 | 0x89ca => {
                w.shift_left(m);
                self.pc + 1
            }
            0x892e | 0x8972 | 0x897d | 0x89a7 | 0x89b2 | 0x89cb | 0x89d6 => {
                w.tax_word();
                self.pc + 1
            }
            0x892f | 0x8973 | 0x8979 | 0x89a8 | 0x89ae | 0x89cc | 0x89d2 => {
                let base = match self.pc {
                    0x892f => 0x0186f0,
                    0x8973 => 0x018200,
                    0x8979 => 0x018000,
                    0x89a8 => 0x018470,
                    0x89ae => 0x0183f0,
                    0x89cc => 0x0185f0,
                    _ => 0x0184f0,
                };
                w.drain(24);
                let v = w.read(base + u32::from(w.cpu.x), m);
                w.load(v, m);
                self.pc + 4
            }
            0x8935 => {
                let v = w.dp_read(2, 2);
                w.cpu.x = v;
                w.flags(v, 2);
                0x8937
            }
            0x8939 | 0x8980 | 0x89b5 | 0x89d9 => {
                w.drain(16);
                let target = w.read(0x0e, 2);
                assert!(self.draw_return.is_some());
                self.waiting = true;
                target
            }
            0x893c | 0x8968 | 0x8983 => {
                w.width(0x20, self.pc != 0x893c);
                self.pc + 2
            }
            0x8952 | 0x8953 | 0x8958 | 0x8990 | 0x89a2 | 0x89a3 => {
                w.increment_y();
                self.pc + 1
            }
            0x8959 | 0x89a4 => {
                w.dp_store(0xba, w.cpu.y, 2);
                self.pc + 2
            }
            0x8960 => {
                let old = w.dp_read(8, 1);
                let v = (old >> 1) | if w.cpu.p & CARRY != 0 { 0x80 } else { 0 };
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (old as u8 & 1);
                w.flags(v, 1);
                w.write(8, v, 1);
                0x8962
            }
            0x8998 | 0x89c4 => {
                let v = w.dp_read(if self.pc == 0x8998 { 8 } else { 0xb2 }, m);
                w.load(w.cpu.a | v, m);
                self.pc + 2
            }
            0x89c6 => {
                let old = w.dp_read(4, 2);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write(4, old | w.cpu.a, 2);
                0x89c8
            }
            _ => unreachable!("room object parser {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

/// Shared tile drawers, conditional edges and size loops with live returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonObjectTiles {
    pc: u16,
    blocks: Option<super::dungeon_floors::DungeonFloors>,
    blocks_tail: bool,
    wall: Option<Box<super::dungeon_moving_wall::DungeonMovingWall>>,
    water: Option<super::dungeon_water::DungeonWater>,
    saved: Option<Box<super::dungeon_saved_objects::DungeonSavedObjects>>,
    decor: Option<Box<super::dungeon_room_decor::DungeonRoomDecor>>,
    stairs: Option<Box<super::dungeon_stairs::DungeonStairs>>,
    doors: Option<super::dungeon_doors::DungeonDoors>,
    depth: u8,
    complete: bool,
}
impl DungeonObjectTiles {
    pub(crate) fn door_state_active(&self) -> bool {
        self.doors.as_ref().is_some_and(|work| work.state_active())
    }

    pub(crate) fn saved_objects_active(&self) -> bool {
        self.saved.is_some()
    }

    /// Shared tile/dimension work, including its retained floor-block child.
    /// Specialized object families keep their own state publication boundary.
    pub(crate) fn shared_tiles_active(&self) -> bool {
        self.wall.is_none() && self.water.is_none() && self.saved.is_none()
            && self.decor.is_none() && self.stairs.is_none() && self.doors.is_none()
    }

    pub(crate) fn new(entry: u32) -> Option<Self> {
        matches!(
            entry,
            0x019895
                | 0x01b2af
                | 0x018b74
                | 0x018b79
                | 0x018b7e
                | 0x018b89
                | 0x0197ed
                | 0x018fa5
                | 0x019c3e
                | 0x018aa3
                | 0x018f62
                | 0x0199e6
                | 0x019813
                | 0x018c58
                | 0x018c61
                | 0x018c6a
                | 0x018cb9
                | 0x018f8a
                | 0x018a89
                | 0x018c4f
                | 0x0196f9
                | 0x0194b4
                | 0x018a92
                | 0x018b0d
                | 0x018aa4
                | 0x018c37
                | 0x019111
                | 0x0190f9
                | 0x0192fb
                | 0x01930e
                | 0x0194df
                | 0x019120
                | 0x019136
                | 0x0196dc
                | 0x0196e4
                | 0x019702
                | 0x01971b
                | 0x0190d9
                | 0x0190e2
                | 0x0199ec
                | 0x019a06
                | 0x019a0c
                | 0x019b50
                | 0x018f30
                | 0x018f36
                | 0x019cc6
                | 0x019ceb
                | 0x018b94
                | 0x018cc7
                | 0x019bd9
                | 0x018d9e
                | 0x018be0
                | 0x018bf4
                | 0x018c0e
                | 0x018c22
                | 0x018e67
                | 0x018e7b
                | 0x018e95
                | 0x018ea9
                | 0x019190
                | 0x01921c
                | 0x019298
                | 0x019210
                | 0x019211
                | 0x019216
                | 0x018eeb
                | 0x018ebe
                | 0x018ec3
                | 0x018ef0
                | 0x018f0c
                | 0x01913f
                | 0x019338
                | 0x019347
                | 0x019357
                | 0x01936f
                | 0x019387
                | 0x01939f
                | 0x019446
                | 0x019456
                | 0x019323
                | 0x018f9d
                | 0x018fa2
                | 0x0190f8
                | 0x01971a
                | 0x018fbc
                | 0x01918f
                | 0x019854
                | 0x01985c
                | 0x019aa3
                | 0x019a90
                | 0x019a8d
                | 0x019aee
                | 0x019c3b
                | 0x019c44
                | 0x01a194
                | 0x01a7a3
                | 0x018fbd
                | 0x019001
                | 0x019045
                | 0x01908f
                | 0x0193b7
                | 0x0193dc
                | 0x019429
                | 0x018d5d
                | 0x019466
                | 0x019488
                | 0x0194bd
                | 0x0197b5
                | 0x0197dc
                | 0x019501
                | 0x0195ef
                | 0x018ddc
                | 0x019733
                | 0x018d80
                | 0x01b0ac
                | 0x019892
                | 0x0198ae
                | 0x0198d0
                | 0x0199b8
                | 0x0199bb
                | 0x019a00
                | 0x019a6f
                | 0x01b395
                | 0x01b493
                | 0x01b376
                | 0x01b381
                | 0x01b474
                | 0x01b47f
                | 0x01b306
                | 0x01b30b
                | 0x01b310
                | 0x01b3e1
                | 0x019a66
                | 0x019b48
                | 0x019d5d
                | 0x019d67
                | 0x019d6c
                | 0x019d96
                | 0x019da2
                | 0x019dd9
                | 0x01a7b6
                | 0x01a7d3
                | 0x01a7dc
                | 0x01a7f0
                | 0x01a809
                | 0x019d29
                | 0x01a255
                | 0x0197f0
                | 0x019a12
                | 0x019aa9
                | 0x019b56
                | 0x019bf8
                | 0x019de5
                | 0x019e30
                | 0x019ea3
                | 0x01a095
                | 0x01a1d1
                | 0x019b1e
                | 0x01a25d
                | 0x01a26d
                | 0x01a2c7
                | 0x01a2df
                | 0x01a30c
                | 0x01a31c
                | 0x01a36e
                | 0x01a380
                | 0x01a3ae
                | 0x01a41b
                | 0x01a458
                | 0x01a486
                | 0x01a4b4
                | 0x01a4f5
                | 0x01a533
                | 0x01a584
                | 0x01a5d2
                | 0x01a5f4
                | 0x01a607
                | 0x01a626
                | 0x01a664
                | 0x01a695
                | 0x01a71c
                | 0x01a74a
                | 0x01ab99
                | 0x01aad7
                | 0x01a81c
                | 0x01a984
        )
        .then_some(Self {
            pc: entry as u16,
            blocks: None,
            blocks_tail: false,
            doors: matches!(entry, 0x01a81c | 0x01a984 | 0x01aad7 | 0x01ab99)
                .then(|| super::dungeon_doors::DungeonDoors::new(entry as u16)),
            stairs: matches!(
                entry,
                0x019b1e
                    | 0x01a25d
                    | 0x01a26d
                    | 0x01a2c7
                    | 0x01a2df
                    | 0x01a30c
                    | 0x01a31c
                    | 0x01a36e
                    | 0x01a380
                    | 0x01a3ae
                    | 0x01a41b
                    | 0x01a458
                    | 0x01a486
                    | 0x01a4b4
                    | 0x01a4f5
                    | 0x01a533
                    | 0x01a584
                    | 0x01a5d2
                    | 0x01a5f4
                    | 0x01a607
                    | 0x01a626
                    | 0x01a664
                    | 0x01a695
                    | 0x01a71c
                    | 0x01a74a
            )
            .then(|| Box::new(super::dungeon_stairs::DungeonStairs::new(entry as u16))),
            decor: matches!(
                entry,
                0x019a12
                    | 0x019aa9
                    | 0x019b56
                    | 0x019bf8
                    | 0x019de5
                    | 0x019e30
                    | 0x019ea3
                    | 0x01a095
                    | 0x01a1d1
            )
            .then(|| {
                Box::new(super::dungeon_room_decor::DungeonRoomDecor::new(
                    entry as u16,
                ))
            }),
            saved: matches!(
                entry,
                0x019892
                    | 0x0198ae
                    | 0x0198d0
                    | 0x0199b8
                    | 0x0199bb
                    | 0x019a00
                    | 0x019a6f
                    | 0x01b395
                    | 0x01b493
                    | 0x01b376
                    | 0x01b381
                    | 0x01b474
                    | 0x01b47f
                    | 0x01b306
                    | 0x01b30b
                    | 0x01b310
                    | 0x01b3e1
            )
            .then(|| {
                Box::new(super::dungeon_saved_objects::DungeonSavedObjects::new(
                    entry as u16,
                ))
            }),
            water: matches!(entry, 0x019501 | 0x0195ef)
                .then(|| super::dungeon_water::DungeonWater::new(entry as u16)),
            wall: matches!(entry, 0x019190 | 0x01921c | 0x019298).then(|| {
                Box::new(super::dungeon_moving_wall::DungeonMovingWall::new(
                    entry as u16,
                ))
            }),
            depth: 0,
            complete: false,
        })
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
        if let Some(doors) = self.doors.as_mut() {
            let cycles = doors.step(cpu, clock, bus);
            self.pc = cpu.pc;
            self.complete = doors.complete();
            return cycles;
        }
        if let Some(saved) = self.saved.as_mut() {
            let cycles = saved.step(cpu, clock, bus);
            self.pc = cpu.pc;
            self.complete = saved.complete();
            return cycles;
        }
        if let Some(decor) = self.decor.as_mut() {
            let cycles = decor.step(cpu, clock, bus);
            self.pc = cpu.pc;
            self.complete = decor.complete();
            return cycles;
        }
        if let Some(stairs) = self.stairs.as_mut() {
            let cycles = stairs.step(cpu, clock, bus);
            self.pc = cpu.pc;
            self.complete = stairs.complete();
            return cycles;
        }
        if let Some(water) = self.water.as_mut() {
            let cycles = water.step(cpu, clock, bus);
            self.pc = cpu.pc;
            self.complete = water.complete();
            return cycles;
        }
        if let Some(wall) = self.wall.as_mut() {
            let cycles = wall.step(cpu, clock, bus);
            self.pc = cpu.pc;
            self.complete = wall.complete();
            return cycles;
        }
        if let Some(blocks) = self.blocks.as_mut() {
            let cycles = blocks.step(cpu, clock, bus);
            self.pc = cpu.pc;
            if blocks.complete() {
                self.blocks = None;
                if self.blocks_tail {
                    if self.depth == 0 {
                        self.complete = true;
                    } else {
                        self.depth -= 1;
                    }
                }
            }
            return cycles;
        }
        let accumulator_mode = match self.pc {
            0x9d45 | 0x9d47 | 0x9d49 | 0x9d4b | 0x9d4e | 0x9d51 | 0x9d53 => 0x20,
            0x9d5d => cpu.p & 0x20,
            _ => 0,
        };
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
            (1, self.pc, 0, 0, accumulator_mode)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x9d29 | 0x9d33 => {
                let value = w.dp_read(if self.pc == 0x9d29 { 0xaf } else { 0xa0 }, 2);
                w.load(value, 2);
                self.pc + 2
            }
            0x9d2b | 0x9d3b => {
                w.mask(if self.pc == 0x9d2b { 0xff } else { 0x100 }, 2);
                self.pc + 3
            }
            0x9d2e => {
                w.compare_immediate(0x1b, 2);
                self.pc + 3
            }
            0x9d31 | 0x9d3e | 0x9d49 | 0x9d51 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x9d31 => 0x9d43,
                        0x9d49 => 0x9d53,
                        _ => 0x9d5d,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x9d35 => {
                w.shift_left(2);
                self.pc + 1
            }
            0x9d36 => {
                w.tax_word();
                self.pc + 1
            }
            0x9d37 => {
                w.drain(24);
                let value = w.read(0x7ef000 + u32::from(w.cpu.x), 2);
                w.load(value, 2);
                self.pc + 4
            }
            0x9d40 | 0xa257 => {
                w.drain(16);
                w.cpu.x = if self.pc == 0x9d40 { 0x1614 } else { 0x00e0 };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x9d43 | 0x9d53 => {
                w.width(0x20, self.pc == 0x9d53);
                self.pc + 2
            }
            0x9d45 => {
                w.immediate(0x19, 1);
                self.pc + 2
            }
            0x9d47 => {
                let value = w.dp_read(0xaf, 1);
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= value);
                w.flags(a.wrapping_sub(value), 1);
                self.pc + 2
            }
            0x9d4b | 0x9d4e => {
                w.drain(16);
                let value = w.read(if self.pc == 0x9d4b { 0x0403 } else { 0x0098c9 }, 1);
                w.load(
                    if self.pc == 0x9d4b {
                        value
                    } else {
                        w.cpu.a & value
                    },
                    1,
                );
                self.pc + 3
            }
            0x9d55 => {
                w.drain(16);
                w.write(0x047c, w.cpu.y, 2);
                self.pc + 3
            }
            0x9d58 => {
                w.immediate(3, 2);
                self.pc + 3
            }
            0x9d5b => {
                w.branch(true);
                0x9d6f
            }
            0xa255 => {
                w.dp_store(0x0c, 0, 2);
                self.pc + 2
            }
            0xa25a => {
                w.drain(16);
                self.blocks = Some(super::dungeon_floors::DungeonFloors::chunks());
                self.blocks_tail = true;
                0x8a1f
            }
            0x9a66 | 0x9d9c => {
                w.drain(16);
                w.cpu.y = if self.pc == 0x9a66 { 0x1f92 } else { 0x1dfa };
                w.flags(w.cpu.y, 2);
                self.pc + 3
            }
            0x9a69 | 0x9b48 | 0x9d62 | 0x9d67 | 0x9d6c | 0x9d9f | 0x9db2 | 0x9de0 | 0xa809
            | 0xa816 => {
                w.immediate(
                    match self.pc {
                        0x9a69 => 6,
                        0x9b48 => 1,
                        0x9d62 | 0xa809 | 0xa816 => 5,
                        0x9d67 => 7,
                        0x9d6c => 2,
                        0x9db2 => 8,
                        _ => 10,
                    },
                    2,
                );
                self.pc + 3
            }
            0x9a6c | 0x9b4d | 0xa7d0 | 0xa806 => {
                w.drain(16);
                match self.pc {
                    0x9a6c => 0x9da2,
                    0x9b4d => 0xb220,
                    _ => 0x97ed,
                }
            }
            0x9b4b | 0x9d6f | 0x9da2 | 0x9db0 | 0x9db5 | 0x9db9 | 0x9dd2 => {
                w.dp_store(
                    match self.pc {
                        0x9b4b => 0xb2,
                        0x9da2 => 0x0a,
                        0x9db0 | 0x9dd2 => 8,
                        0x9db5 => 0x0c,
                        _ => 0x0e,
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 2
            }
            0x9d5d => {
                w.width(0x20, true);
                self.pc + 2
            }
            0x9d5f | 0xa7c0 | 0xa7cd => {
                w.drain(16);
                w.cpu.x = match self.pc {
                    0x9d5f => 0x162c,
                    0xa7c0 => 0x2376,
                    _ => 0x2396,
                };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x9d65 | 0x9d6a | 0x9de3 => {
                w.branch(true);
                if self.pc == 0x9de3 {
                    0x9da2
                } else {
                    0x9d6f
                }
            }
            0x9d71 | 0x9d76 | 0x9d7b | 0x9d80 | 0x9dbd => {
                w.operand_then_internal(2);
                let (index, offset) = if self.pc == 0x9dbd {
                    (w.cpu.y, 0)
                } else {
                    (w.cpu.x, (self.pc - 0x9d71) / 5 * 2)
                };
                let value = w.read(0x009b52 + u32::from(index) + u32::from(offset), 2);
                w.load(value, 2);
                self.pc + 3
            }
            0x9d74 | 0x9d79 | 0x9d7e | 0x9d83 => {
                let address = w.indirect_y_address(match self.pc {
                    0x9d74 => 0xbf,
                    0x9d79 => 0xc2,
                    0x9d7e => 0xc5,
                    _ => 0xc8,
                });
                w.write(address, w.cpu.a, 2);
                self.pc + 2
            }
            0x9d85 | 0x9d8b => {
                w.drain(6);
                w.load(if self.pc == 0x9d85 { w.cpu.x } else { w.cpu.y }, 2);
                self.pc + 1
            }
            0x9d86 | 0x9d8c | 0x9dce | 0xa7bb | 0xa7c8 | 0xa7e4 | 0xa7f5 | 0xa801 | 0xa811 => {
                w.clear_carry();
                self.pc + 1
            }
            0x9d87 | 0x9d8d | 0x9dcf | 0xa7bc | 0xa7c9 | 0xa7e5 | 0xa7f6 | 0xa802 | 0xa812 => {
                w.drain(16);
                w.adc(match self.pc {
                    0x9d87 => 8,
                    0x9d8d | 0x9dcf => 0x80,
                    0xa7bc => 0x100,
                    0xa7c9 => 0x300,
                    0xa7f6 => 0x1fc,
                    0xa802 => 0x204,
                    _ => 0x200,
                });
                self.pc + 3
            }
            0x9d8a => {
                w.tax_word();
                self.pc + 1
            }
            0x9d90 | 0xa7bf | 0xa7cc | 0xa7e8 | 0xa7f9 | 0xa805 | 0xa815 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x9d91 | 0x9dc8 | 0x9dd4 => {
                w.dp_increment(if self.pc == 0x9dd4 { 0x0c } else { 0x0e }, 2, false);
                self.pc + 2
            }
            0x9d93 | 0x9d9a | 0x9da9 | 0x9dca | 0x9dd6 | 0x9ddd | 0xa7da => {
                let (take, target) = match self.pc {
                    0x9d93 => (w.cpu.p & ZERO == 0, 0x9d71),
                    0x9d9a | 0x9ddd => (w.cpu.p & CARRY != 0, 0x9dd8),
                    0x9da9 => (w.cpu.p & ZERO == 0, 0x9db2),
                    0x9dca => (w.cpu.p & ZERO == 0, 0x9dbd),
                    0x9dd6 => (w.cpu.p & ZERO == 0, 0x9db7),
                    _ => (w.cpu.p & ZERO != 0, 0xa7ef),
                };
                if w.branch(take) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x9d96 | 0x9dd9 => {
                w.drain(16);
                let value = w.read(0x0402, 2);
                w.load(value, 2);
                self.pc + 3
            }
            0x9d99 | 0x9ddc => {
                w.shift_left(2);
                self.pc + 1
            }
            0x9da4 | 0x9dab | 0x9db7 | 0x9dcc | 0xa7b9 | 0xa7c6 | 0xa7e2 | 0xa7f3 | 0xa7ff
            | 0xa80f => {
                let value = w.dp_read(
                    match self.pc {
                        0x9da4 => 0xbf,
                        0x9db7 => 0x0a,
                        _ => 8,
                    },
                    2,
                );
                w.load(value, 2);
                self.pc + 2
            }
            0x9da6 => {
                w.compare_immediate(0x4000, 2);
                self.pc + 3
            }
            0x9dad => {
                w.drain(16);
                w.load(w.cpu.a | 0x2000, 2);
                self.pc + 3
            }
            0x9dbb => {
                w.cpu.x = w.dp_read(8, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0x9dc0 => {
                w.drain(24);
                w.write(0x7e2000 + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0x9dc4 | 0x9dc5 => {
                w.increment_y();
                self.pc + 1
            }
            0x9dc6 | 0x9dc7 => {
                w.increment_x();
                self.pc + 1
            }
            0x9ddf => {
                w.drain(6);
                w.cpu.y = w.cpu.x;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xa7b6 | 0xa7c3 | 0xa7dc | 0xa7df | 0xa7e9 | 0xa7ec | 0xa7f0 | 0xa7fb => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                0x97ed
            }
            0xa7d3 => {
                w.drain(24);
                let value = w.read(0x7ef0ca, 2);
                w.load(value, 2);
                self.pc + 4
            }
            0xa7d7 => {
                w.mask(0x100, 2);
                self.pc + 3
            }
            0xa7fa => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                self.pc + 1
            }
            0xa7fe => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xa819 => {
                w.drain(16);
                self.blocks = Some(super::dungeon_floors::DungeonFloors::blocks());
                self.blocks_tail = true;
                0x8a44
            }
            0x8ddc | 0x8df9 | 0x8e3c => {
                w.immediate(if self.pc == 0x8ddc { 4 } else { 2 }, 2);
                self.pc + 3
            }
            0x8ddf | 0x8de7 | 0x974e | 0x9757 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x8ddf => 0xb0af,
                    0x8de7 => 0xb2ce,
                    _ => 0x975c,
                }
            }
            0x8de2 | 0x8de4 | 0x8dea | 0x8def | 0x8dfc | 0x8e05 | 0x8e3a | 0x8e3f | 0x8e4d
            | 0x974c | 0x975f | 0x9791 => {
                w.dp_store(
                    match self.pc {
                        0x8de2 | 0x974c => 0xb4,
                        0x8dea | 0x8e3a => 0x0c,
                        0x8def => 8,
                        0x8e05 | 0x8e4d | 0x975f | 0x9791 => 0x0a,
                        _ => 0x0e,
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 2
            }
            0x8de6 | 0x975c => {
                w.drain(6);
                w.push(if self.pc == 0x8de6 { w.cpu.y } else { w.cpu.x }, 2);
                self.pc + 1
            }
            0x8dec | 0x8e1e | 0x8e2b | 0x8e44 | 0x8e56 | 0x9749 | 0x97b0 => {
                w.drain(16);
                w.adc(match self.pc {
                    0x8e1e => 6,
                    0x9749 => 5,
                    _ => 0x80,
                });
                self.pc + 3
            }
            0x8df1 | 0x8e47 | 0x8e59 | 0x973e => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x8df2 | 0x8e13 | 0x8e24 | 0x8e33 | 0x8e5a | 0x8e62 | 0x9751 | 0x9771 | 0x97a3 => {
                w.dp_increment(
                    match self.pc {
                        0x8e13 | 0x8e5a | 0x9771 | 0x97a3 => 0x0a,
                        0x8e33 | 0x9751 => 0xb4,
                        _ => 0x0e,
                    },
                    2,
                    false,
                );
                self.pc + 2
            }
            0x8df4 | 0x8e15 | 0x8e26 | 0x8e38 | 0x8e5c | 0x8e64 | 0x9738 | 0x9753 | 0x9773
            | 0x97a5 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x8df4 => 0x8de7,
                        0x8e15 => 0x8e0f,
                        0x8e26 => 0x8e01,
                        0x8e38 => 0x8e35,
                        0x8e5c => 0x8e4f,
                        0x8e64 => 0x8e4b,
                        0x9738 => 0x973f,
                        0x9753 => 0x974e,
                        0x9773 => 0x976b,
                        _ => 0x979d,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x8df6 => {
                w.drain(12);
                w.cpu.y = w.pull(2);
                w.flags(w.cpu.y, 2);
                0x8df7
            }
            0x8df7 | 0x8e31 => {
                w.dp_store(if self.pc == 0x8df7 { 8 } else { 0xb4 }, w.cpu.y, 2);
                self.pc + 2
            }
            0x8dfe | 0x8e48 => {
                w.drain(16);
                w.cpu.x = if self.pc == 0x8dfe { 0x063c } else { 0x0648 };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x8e01 | 0x8e28 | 0x8e41 | 0x8e4b | 0x9733 | 0x9745 | 0x975d | 0x978f => {
                let v = w.dp_read(
                    match self.pc {
                        0x8e28 | 0x8e41 => 8,
                        0x8e4b | 0x9745 => 0xb4,
                        0x9733 => 0xbf,
                        _ => 0xb2,
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0x8e03 | 0x8e04 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1), 2);
                self.pc + 1
            }
            0x8e07 | 0x8e0c | 0x8e17 | 0x8e4f => {
                w.operand_then_internal(2);
                let offset = match self.pc {
                    0x8e0c => 2,
                    0x8e17 => 4,
                    _ => 0,
                };
                let v = w.read(0x009b52 + u32::from(w.cpu.x) + offset, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x8e0a | 0x8e0f | 0x8e1a | 0x8e52 => {
                let a = w.indirect_y_address(if matches!(self.pc, 0x8e0f | 0x8e1a) {
                    0xc2
                } else {
                    0xbf
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8e11 | 0x8e12 | 0x9755 | 0x9756 | 0x975a | 0x975b => {
                w.increment_y();
                self.pc + 1
            }
            0x8e1c => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                0x8e1d
            }
            0x8e1d | 0x8e2a | 0x8e43 | 0x8e55 | 0x9748 | 0x97af => {
                w.clear_carry();
                self.pc + 1
            }
            0x8e21 | 0x97b3 => {
                w.tax_word();
                self.pc + 1
            }
            0x8e22 | 0x8e2e | 0x8e60 => {
                w.cpu.y = w.dp_read(if self.pc == 0x8e2e { 0xb2 } else { 0x0c }, 2);
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0x8e30 | 0x8e37 => {
                w.decrement_y();
                self.pc + 1
            }
            0x8e35 | 0x8e36 => {
                w.increment_a(2);
                self.pc + 1
            }
            0x8e54 | 0x973a => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x8e5e | 0x8e5f | 0x976f | 0x9770 | 0x97a1 | 0x97a2 => {
                w.increment_x();
                self.pc + 1
            }
            0x9735 => {
                w.compare_immediate(0x4000, 2);
                0x9738
            }
            0x973b => {
                w.drain(16);
                w.load(w.cpu.a | 0x2000, 2);
                0x973e
            }
            0x973f => {
                w.drain(6);
                w.cpu.x = w.cpu.y;
                w.flags(w.cpu.x, 2);
                0x9740
            }
            0x9740 => {
                w.drain(16);
                w.cpu.y = 0x0ab4;
                w.flags(w.cpu.y, 2);
                0x9743
            }
            0x9743 => {
                w.dp_increment(0xb2, 2, true);
                0x9745
            }
            0x9747 => {
                w.shift_left(2);
                0x9748
            }
            0x9761 | 0x9768 | 0x9775 | 0x977c | 0x9793 | 0x979a | 0x97a7 => {
                w.operand_then_internal(2);
                let offset = match self.pc {
                    0x9761 => 0,
                    0x9768 => 6,
                    0x9775 => 12,
                    0x977c => 18,
                    0x9793 => 24,
                    0x979a => 30,
                    _ => 36,
                };
                let v = w.read(0x009b52 + u32::from(w.cpu.y) + offset, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x9764 | 0x976b | 0x9778 | 0x977f | 0x9783 | 0x9787 | 0x978b | 0x9796 | 0x979d
            | 0x97aa => {
                w.drain(24);
                let offset = match self.pc {
                    0x9764 => 0,
                    0x976b | 0x9778 => 2,
                    0x977f => 4,
                    0x9783 => 6,
                    0x9787 => 8,
                    0x978b => 10,
                    0x9796 => 12,
                    _ => 14,
                };
                w.write(0x7e2000 + u32::from(w.cpu.x) + offset, w.cpu.a, 2);
                self.pc + 4
            }
            0x97ae => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                0x97af
            }
            0x8e66 | 0x97b4 => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0x9466 | 0x946b | 0x9476 | 0x9488 | 0x9490 | 0x94bd | 0x94c2 | 0x94cd | 0x97b8
            | 0x97dc | 0x97e5 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x946b | 0x9476 => 0xb279,
                    0x9490 => 0x8d80,
                    0x94c2 | 0x94cd => 0xb2f6,
                    0x97b8 => 0xb0af,
                    0x97e5 => 0x97f0,
                    _ => 0xb0ac,
                }
            }
            0x9469 | 0x948b | 0x94c0 | 0x97bb => {
                let v = w.dp_read(0xb2, 2);
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(v & 0x8000 != 0);
                let value = v.wrapping_shl(1);
                w.flags(value, 2);
                w.write(0xb2, value, 2);
                self.pc + 2
            }
            0x946e | 0x947f | 0x94c7 | 0x94d6 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x946f | 0x9480 | 0x94c8 | 0x94d7 | 0x97d2 => {
                w.clear_carry();
                self.pc + 1
            }
            0x9470 | 0x9481 | 0x94c9 | 0x94d8 | 0x97d3 => {
                w.drain(16);
                w.adc(match self.pc {
                    0x9470 | 0x9481 => 10,
                    0x97d3 => 0x80,
                    _ => 6,
                });
                self.pc + 3
            }
            0x9473 | 0x9484 | 0x94cc | 0x94db => {
                w.tax_word();
                self.pc + 1
            }
            0x9474 | 0x9475 | 0x9479 | 0x947a | 0x94a2 | 0x94a3 | 0x94c5 | 0x94c6 | 0x94d0
            | 0x94d1 => {
                w.increment_y();
                self.pc + 1
            }
            0x947b | 0x94a4 | 0x94d2 | 0x97d7 | 0x97e8 => {
                w.dp_increment(0xb2, 2, false);
                self.pc + 2
            }
            0x947d | 0x94a6 | 0x94d4 | 0x97d9 | 0x97ea => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x947d => 0x9476,
                        0x94a6 => 0x9493,
                        0x94d4 => 0x94cd,
                        0x97d9 => 0x97c7,
                        _ => 0x97df,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x9485 | 0x94b1 | 0x94dc => {
                w.drain(16);
                match self.pc {
                    0x9485 => 0xb279,
                    0x94b1 => 0x8d80,
                    _ => 0xb2f6,
                }
            }
            0x948d | 0x94ae | 0x97b5 | 0x97e2 => {
                w.immediate(if self.pc < 0x9700 { 1 } else { 2 }, 2);
                self.pc + 3
            }
            0x9493 | 0x9498 | 0x949d | 0x97bd | 0x97c2 | 0x97c7 | 0x97cc | 0xb2f6 | 0xb2fb
            | 0xb300 => {
                w.operand_then_internal(2);
                let start = match self.pc {
                    0x9493 | 0x9498 | 0x949d => 0x9493,
                    0x97bd | 0x97c2 | 0x97c7 | 0x97cc => 0x97bd,
                    _ => 0xb2f6,
                };
                let v = w.read(
                    0x009b52 + u32::from(w.cpu.x) + u32::from((self.pc - start) / 5 * 2),
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0x9496 | 0x949b | 0x94a0 | 0x97c0 | 0x97c5 | 0x97ca | 0x97cf | 0xb2f9 | 0xb2fe
            | 0xb303 => {
                let a = w.indirect_y_address(match self.pc {
                    0x949b | 0x97ca | 0xb2fe => 0xcb,
                    0x94a0 | 0xb303 => 0xd7,
                    0x97c5 => 0xc2,
                    0x97cf => 0xce,
                    _ => 0xbf,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x94a8..=0x94ad => {
                w.increment_x();
                self.pc + 1
            }
            0x97d1 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                0x97d2
            }
            0x97d6 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                0x97d7
            }
            0x97df => {
                w.drain(16);
                w.cpu.x = 0x0b16;
                w.flags(w.cpu.x, 2);
                0x97e2
            }
            0x97db | 0x97ec | 0xb305 => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0x8fbd | 0x9001 | 0x9045 | 0x908f | 0x8d67 | 0x8d73 | 0x8d7d => {
                w.immediate(
                    match self.pc {
                        0x8d67 => 2,
                        0x8d73 | 0x8d7d => 1,
                        _ => 10,
                    },
                    2,
                );
                self.pc + 3
            }
            0x8fc0 | 0x9004 | 0x9048 | 0x9092 | 0x8fd4 | 0x9018 | 0x905c | 0x90a6 | 0x93b7
            | 0x93bd | 0x93cb | 0x93e2 | 0x93ed | 0x93f4 | 0x9431 | 0x8d6a | 0x8d76 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x8fd4 => 0x8fec,
                    0x9018 => 0x9030,
                    0x905c => 0x9078,
                    0x90a6 => 0x90c2,
                    0x93b7 => 0xb0ac,
                    0x93bd | 0x93cb => 0xb2af,
                    0x93e2 | 0x93ed => 0x93ff,
                    0x93f4 => 0x93f7,
                    0x9431 => 0x9895,
                    0x8d6a | 0x8d76 => 0x8d80,
                    _ => 0xb0af,
                }
            }
            0x8fc3 | 0x8fdb | 0x8fec | 0x8ff1 | 0x9007 | 0x9023 | 0x9036 | 0x903b | 0x904b
            | 0x9063 | 0x9078 | 0x907d | 0x9095 | 0x90b1 | 0x90c8 | 0x90cd | 0x9403 | 0x9408
            | 0x940d | 0x941a => {
                w.operand_then_internal(2);
                let offset = match self.pc {
                    0x8ff1 | 0x903b | 0x907d | 0x90cd | 0x9408 => 2,
                    0x940d => 4,
                    0x941a => 6,
                    _ => 0,
                };
                let v = w.read(0x009b52 + u32::from(w.cpu.x) + offset, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x8fc6 | 0x900a | 0x904e | 0x9098 | 0x9401 | 0x942f | 0x8d65 | 0x93d4 | 0x9425
            | 0x943e => {
                w.dp_store(
                    match self.pc {
                        0x8d65 => 0xb2,
                        0x93d4 | 0x9425 | 0x943e => 8,
                        _ => 0x0e,
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 2
            }
            0x8fc8
            | 0x8fc9
            | 0x8fd7..=0x8fda
            | 0x8fea
            | 0x8feb
            | 0x900c
            | 0x900d
            | 0x901b..=0x901e
            | 0x902e
            | 0x902f
            | 0x9050
            | 0x9051
            | 0x905f..=0x9062
            | 0x9076
            | 0x9077
            | 0x909a
            | 0x909b
            | 0x90a9..=0x90ac
            | 0x90c0
            | 0x90c1
            | 0x93e5..=0x93ec
            | 0x93f7..=0x93fe => {
                w.increment_x();
                self.pc + 1
            }
            0x8fca | 0x900e | 0x9052 | 0x909c => {
                let a = w.indirect_y_address(match self.pc {
                    0x900e => 0xcb,
                    0x909c => 0xc2,
                    _ => 0xbf,
                });
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0x8fcc | 0x9010 | 0x9054 | 0x909e => {
                w.mask(0x03ff, 2);
                self.pc + 3
            }
            0x8fcf | 0x9013 | 0x9057 | 0x90a1 => {
                w.compare_immediate(if self.pc < 0x9045 { 0x00e2 } else { 0x00e3 }, 2);
                self.pc + 3
            }
            0x8fd2 | 0x9016 | 0x905a | 0x90a4 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    self.pc + 5
                } else {
                    self.pc + 2
                }
            }
            0x8fde | 0x8fe2 | 0x8fef | 0x8ff4 | 0x8ff8 | 0x8ffa | 0x9021 | 0x9026 | 0x9032
            | 0x9034 | 0x9039 | 0x903e | 0x9066 | 0x906a | 0x907b | 0x9080 | 0x9084 | 0x9086
            | 0x90af | 0x90b4 | 0x90c4 | 0x90c6 | 0x90cb | 0x90d0 | 0x9406 | 0x940b | 0x9410
            | 0x941d => {
                let a = w.indirect_y_address(match self.pc {
                    0x8fe2 | 0x8ff8 | 0x9026 | 0x9039 | 0x9080 | 0x90c6 => 0xcb,
                    0x8ff4 | 0x9034 | 0x906a | 0x9084 | 0x90b4 | 0x90cb | 0x940b | 0x941d => 0xc2,
                    0x8ffa | 0x903e | 0x9086 | 0x90d0 => 0xce,
                    0x9410 => 0xc5,
                    _ => 0xbf,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8fe0 | 0x8ff6 | 0x901f | 0x9030 | 0x9068 | 0x9082 | 0x90ad | 0x90c2 | 0x93ce
            | 0x941f | 0x9438 | 0x93ff | 0x942d | 0x8d5d => {
                let v = w.dp_read(
                    match self.pc {
                        0x93ce | 0x941f | 0x9438 => 8,
                        0x93ff | 0x942d | 0x8d5d => 0xb2,
                        _ => 0x0e,
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0x8fe4
            | 0x8fe5
            | 0x8ffc..=0x8fff
            | 0x9028
            | 0x9029
            | 0x9040..=0x9043
            | 0x9412..=0x9415 => {
                w.increment_y();
                self.pc + 1
            }
            0x8fe6 | 0x902a | 0x9072 | 0x90bc | 0x93d7 | 0x93f0 | 0x9416 | 0x9434 | 0x9441
            | 0x8d79 => {
                w.dp_increment(
                    match self.pc {
                        0x93f0 | 0x9441 => 0xb4,
                        0x9416 | 0x9434 => 0x0e,
                        _ => 0xb2,
                    },
                    2,
                    false,
                );
                self.pc + 2
            }
            0x8fe8 | 0x902c | 0x9074 | 0x90be | 0x93d9 | 0x93f2 | 0x9418 | 0x9436 | 0x9443
            | 0x8d7b => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x8fe8 => 0x8fdb,
                        0x902c => 0x901f,
                        0x9074 => 0x9063,
                        0x90be => 0x90ad,
                        0x93d9 => 0x93ba,
                        0x93f2 => 0x93ed,
                        0x9418 => 0x9408,
                        0x9436 => 0x9431,
                        0x9443 => 0x942d,
                        _ => 0x8d6d,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x906c | 0x9088 | 0x90b6 | 0x90d2 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x906d | 0x9089 | 0x90b7 | 0x90d3 | 0x93c0 | 0x93c6 | 0x93d0 | 0x9421 | 0x943a => {
                w.clear_carry();
                self.pc + 1
            }
            0x906e | 0x908a | 0x90b8 | 0x90d4 | 0x93c1 | 0x93c7 | 0x93d1 | 0x9422 | 0x943b => {
                w.drain(16);
                w.adc(match self.pc {
                    0x908a | 0x90d4 | 0x943b => 0x0100,
                    0x93c1 => 0x0200,
                    0x93c7 | 0x93d1 => 8,
                    _ => 0x0080,
                });
                self.pc + 3
            }
            0x9071 | 0x908d | 0x90bb | 0x90d7 | 0x93c4 | 0x93d6 | 0x9427 | 0x9440 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x93ba => {
                w.drain(16);
                w.cpu.x = 0x08ca;
                w.flags(w.cpu.x, 2);
                0x93bd
            }
            0x93c5 | 0x8d6d => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x93ca | 0x8d72 => {
                w.tax_word();
                self.pc + 1
            }
            0x93dc | 0x93e0 | 0x9429 | 0x942b => {
                w.dp_increment(
                    if matches!(self.pc, 0x93dc | 0x9429) {
                        0xb2
                    } else {
                        0xb4
                    },
                    2,
                    true,
                );
                self.pc + 2
            }
            0x93de => {
                let v = w.dp_read(0xb4, 2);
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(v & 0x8000 != 0);
                let result = v.wrapping_shl(1);
                w.flags(result, 2);
                w.write(0xb4, result, 2);
                0x93e0
            }
            0x8d5f | 0x8d60 | 0x8d63 => {
                w.shift_left(2);
                self.pc + 1
            }
            0x8d61 => {
                let v = w.dp_read(0xb4, 2);
                w.load(w.cpu.a | v, 2);
                0x8d63
            }
            0x8d64 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_add(1), 2);
                0x8d65
            }
            0x8d6e => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0x8d6f
            }
            0x8d6f => {
                w.drain(16);
                w.sbc(6, 2);
                0x8d72
            }
            0x9000 | 0x9044 | 0x908e | 0x90d8 | 0x93db | 0x9428 | 0x9445 => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0x9aa3 | 0x9a90 | 0x9a9d | 0xa7a3 | 0xa7b0 | 0x9aee | 0xa194 | 0x9c55 => {
                w.immediate(
                    match self.pc {
                        0x9aa3 => 6,
                        0x9a90 | 0x9a9d => 4,
                        0x9aee | 0x9c55 => 5,
                        _ => 3,
                    },
                    2,
                );
                self.pc + 3
            }
            0x9aa6 | 0x9aa0 | 0xa7b3 | 0x9a8d => {
                w.drain(16);
                match self.pc {
                    0x9aa6 => 0x97f0,
                    0x9a8d => 0xb2af,
                    _ => 0x8d80,
                }
            }
            0x9a93 | 0xa7a6 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                0x8d80
            }
            0x9a96 | 0xa7a9 | 0x9c45 => {
                let v = w.dp_read(if self.pc == 0x9c45 { 0xbf } else { 8 }, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0x9a98 | 0xa7ab | 0x9b08 | 0x9b0e | 0xa1b8 => {
                w.clear_carry();
                self.pc + 1
            }
            0x9a99 | 0xa7ac | 0x9b09 | 0x9b0f | 0xa1b9 => {
                w.drain(16);
                w.adc(match self.pc {
                    0x9a99 | 0xa7ac => 0x0180,
                    0x9b09 => 8,
                    _ => 0x0080,
                });
                self.pc + 3
            }
            0x9a9c | 0xa7af | 0x9b12 | 0xa1bc => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x9af1 | 0xa197 | 0x9c58 => {
                w.dp_store(if self.pc == 0x9c58 { 0x0c } else { 0x0e }, w.cpu.a, 2);
                self.pc + 2
            }
            0x9af3 | 0x9af8 | 0x9afd | 0x9b02 | 0xa199 | 0xa19e | 0xa1a3 | 0xa1a8 | 0xa1ad
            | 0xa1b2 | 0xa1c1 | 0xa1c6 | 0xa1cb => {
                w.operand_then_internal(2);
                let offset = match self.pc {
                    0x9af3 | 0x9af8 | 0x9afd | 0x9b02 => (self.pc - 0x9af3) / 5 * 2,
                    0xa1c1 | 0xa1c6 | 0xa1cb => 12 + (self.pc - 0xa1c1) / 5 * 2,
                    _ => (self.pc - 0xa199) / 5 * 2,
                };
                let v = w.read(0x009b52 + u32::from(w.cpu.x) + u32::from(offset), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x9af6 | 0x9afb | 0x9b00 | 0x9b05 | 0xa19c | 0xa1a1 | 0xa1a6 | 0xa1ab | 0xa1b0
            | 0xa1b5 | 0xa1c4 | 0xa1c9 | 0xa1ce => {
                let a = w.indirect_y_address(match self.pc {
                    0x9af6 | 0xa19c => 0xbf,
                    0x9afb | 0xa1a1 => 0xc2,
                    0x9b00 | 0xa1a6 => 0xc5,
                    0x9b05 => 0xc8,
                    0xa1ab | 0xa1c4 => 0xcb,
                    0xa1b0 | 0xa1c9 => 0xce,
                    _ => 0xd1,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x9b07 | 0x9c4c => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x9b0c | 0x9c50 => {
                w.tax_word();
                self.pc + 1
            }
            0x9b0d | 0xa1b7 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x9b13 | 0xa1bd | 0x9c91 => {
                w.dp_increment(if self.pc == 0x9c91 { 0x0c } else { 0x0e }, 2, false);
                self.pc + 2
            }
            0x9b15 | 0xa1bf | 0x9c93 | 0x9c4a => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x9b15 => 0x9af3,
                        0xa1bf => 0xa1a8,
                        0x9c93 => 0x9c5a,
                        _ => 0x9c51,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x9c3b => {
                w.drain(16);
                let v = w.read(0x03f4, 2).wrapping_add(1);
                w.drain(6);
                w.write(0x03f4, v, 2);
                w.flags(v, 2);
                0x9c3e
            }
            0x9c44 => {
                w.drain(6);
                w.cpu.x = w.cpu.y;
                w.flags(w.cpu.x, 2);
                0x9c45
            }
            0x9c47 => {
                w.compare_immediate(0x4000, 2);
                0x9c4a
            }
            0x9c4d | 0x9c6c | 0x9c7a | 0x9c88 | 0x9c9d | 0x9cbe => {
                w.drain(16);
                w.load(w.cpu.a | if self.pc == 0x9c4d { 0x2000 } else { 0x4000 }, 2);
                self.pc + 3
            }
            0x9c51 => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                0x9c52
            }
            0x9c52 => {
                w.drain(16);
                w.cpu.y = 0x1488;
                w.flags(w.cpu.y, 2);
                0x9c55
            }
            0x9c5a | 0x9c65 | 0x9c73 | 0x9c81 | 0x9c96 | 0x9ca4 | 0x9cb7 => {
                w.operand_then_internal(2);
                let offset = match self.pc {
                    0x9c96 => 0,
                    0x9c5a | 0x9ca4 => 2,
                    0x9c65 => 4,
                    0x9cb7 => 6,
                    0x9c73 => 8,
                    _ => 10,
                };
                let v = w.read(0x009b52 + u32::from(w.cpu.y) + offset, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x9c5d | 0x9c61 | 0x9c68 | 0x9c6f | 0x9c76 | 0x9c7d | 0x9c84 | 0x9c8b | 0x9c99
            | 0x9ca0 | 0x9ca7 | 0x9cab | 0x9caf | 0x9cb3 | 0x9cba | 0x9cc1 => {
                w.drain(24);
                let offset = match self.pc {
                    0x9c5d => 0x0004,
                    0x9c61 => 0x0012,
                    0x9c68 => 0x0084,
                    0x9c6f => 0x0092,
                    0x9c76 => 0x0104,
                    0x9c7d => 0x0112,
                    0x9c84 => 0x0184,
                    0x9c8b => 0x0192,
                    0x9c99 => 0,
                    0x9ca0 => 0x001e,
                    0x9ca7 => 2,
                    0x9cab => 0x000e,
                    0x9caf => 0x0010,
                    0x9cb3 => 0x001c,
                    0x9cba => 0x0102,
                    _ => 0x011c,
                };
                w.write(0x7e2000 + offset + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0x9c8f | 0x9c90 => {
                w.increment_x();
                self.pc + 1
            }
            0x9c95 => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                0x9c96
            }
            0x9b17 | 0xa1d0 | 0x9cc5 | 0x9891 => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0x9338 | 0x933b | 0x9347 | 0x934a | 0x9357 | 0x9361 | 0x936f | 0x9379 | 0x9389
            | 0x9391 | 0x93a1 | 0x93a9 | 0x9446 | 0x9449 | 0x9456 | 0x9459 | 0x9323 | 0x932c => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x933b => 0x9895,
                    0x934a | 0x9449 | 0x9459 => 0xb2af,
                    0x9361 | 0x9379 | 0x93a9 => 0x97f0,
                    0x9391 | 0x932c => 0x8d80,
                    _ => 0xb0ac,
                }
            }
            0x933e..=0x9341 | 0x932f..=0x9332 => {
                w.increment_y();
                self.pc + 1
            }
            0x9342 | 0x9352 | 0x936a | 0x9382 | 0x939a | 0x93b2 | 0x9451 | 0x9461 | 0x9333 => {
                w.dp_increment(0xb2, 2, false);
                self.pc + 2
            }
            0x9344 | 0x9354 | 0x936c | 0x9384 | 0x939c | 0x93b4 | 0x9453 | 0x9463 | 0x9335 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x9344 => 0x933b,
                        0x9354 => 0x934a,
                        0x936c => 0x935c,
                        0x9384 => 0x9374,
                        0x939c => 0x938c,
                        0x93b4 => 0x93a4,
                        0x9453 => 0x9449,
                        0x9463 => 0x9459,
                        _ => 0x9326,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x934d | 0x9365 | 0x937d | 0x9395 | 0x93ad | 0x944c | 0x945c => {
                w.clear_carry();
                self.pc + 1
            }
            0x934e | 0x9366 | 0x937e | 0x9396 | 0x93ae | 0x944d | 0x945d => {
                w.drain(16);
                w.adc(match self.pc {
                    0x934e => 0x0100,
                    0x9366 => 0x02fc,
                    0x93ae => 0x03fa,
                    0x944d => 0x0600,
                    0x945d => 0xff1c,
                    _ => 8,
                });
                self.pc + 3
            }
            0x9351 | 0x9369 | 0x9381 | 0x9399 | 0x93b1 | 0x9450 | 0x9460 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x935a | 0x9372 | 0x9387 | 0x939f => {
                w.dp_store(
                    if matches!(self.pc, 0x935a | 0x9372) {
                        0x0c
                    } else {
                        0x0a
                    },
                    w.cpu.x,
                    2,
                );
                self.pc + 2
            }
            0x935c | 0x9374 | 0x938c | 0x93a4 => {
                w.cpu.x = w.dp_read(
                    if matches!(self.pc, 0x935c | 0x9374) {
                        0x0c
                    } else {
                        0x0a
                    },
                    2,
                );
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0x935e | 0x9376 | 0x938e | 0x93a6 | 0x9329 => {
                w.immediate(
                    match self.pc {
                        0x938e => 4,
                        0x93a6 => 3,
                        _ => 2,
                    },
                    2,
                );
                self.pc + 3
            }
            0x9364 | 0x937c | 0x9394 | 0x93ac => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x9326 => {
                w.drain(16);
                w.cpu.x = 0x0e26;
                w.flags(w.cpu.x, 2);
                0x9329
            }
            0x8f9d | 0x8fa2 => {
                w.drain(16);
                w.cpu.x = w.read(if self.pc == 0x8f9d { 0x0490 } else { 0x046a }, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x8fa0 => {
                w.branch(true);
                0x8fa5
            }
            0x9346 | 0x9356 | 0x936e | 0x9386 | 0x939e | 0x93b6 | 0x9455 | 0x9465 | 0x9337
            | 0x90f8 | 0x971a | 0x8fbc | 0x918f => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0x8eeb | 0x8ebe | 0x8ec3 | 0x8ef0 | 0x8ec9 | 0x8ef6 | 0x9160 | 0x917c => {
                w.immediate(
                    match self.pc {
                        0x8eeb | 0x8ebe => 0x0015,
                        0x8ec9 => 0x00e3,
                        0x8ef6 => 0x00e2,
                        0x9160 | 0x917c => 3,
                        _ => 2,
                    },
                    2,
                );
                self.pc + 3
            }
            0x8eee | 0x8ec1 => {
                w.branch(true);
                if self.pc == 0x8eee {
                    0x8ef3
                } else {
                    0x8ec6
                }
            }
            0x8ef3 | 0x8ec6 | 0x8ecc | 0x8ef9 | 0x8f03 | 0x8f0c | 0x8f0f | 0x9165 | 0x9181 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x8ef3 | 0x8ec6 => 0xb0af,
                    0x8ecc | 0x8ef9 => 0xb191,
                    0x8f03 => 0xb2ca,
                    0x8f0c => 0xb0ac,
                    0x8f0f => 0xb2af,
                    0x9165 => 0xac1a,
                    _ => 0xab78,
                }
            }
            0x8ecf | 0x8efc | 0x8ee3 | 0x8f2a | 0x9147 | 0x9157 | 0x9176 | 0xac38 | 0xab96 => {
                let (take, target) = match self.pc {
                    0x8ecf => (w.cpu.p & CARRY == 0, 0x8ed6),
                    0x8efc => (w.cpu.p & CARRY == 0, 0x8f03),
                    0x8ee3 => (w.cpu.p & ZERO == 0, 0x8ed6),
                    0x8f2a => (w.cpu.p & ZERO == 0, 0x8f18),
                    0x9147 => (w.cpu.p & ZERO == 0, 0x9151),
                    0x9157 => (w.cpu.p & ZERO != 0, 0x9185),
                    0x9176 => (w.cpu.p & ZERO == 0, 0x917c),
                    0xac38 => (w.cpu.p & ZERO == 0, 0xac1a),
                    _ => (w.cpu.p & ZERO == 0, 0xab78),
                };
                if w.branch(take) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x8ed1 | 0x8edc | 0x8ee5 | 0x8efe | 0x8f06 | 0x8f18 | 0x8f1d | 0xac1a | 0xac1f
            | 0xac24 | 0xac29 | 0xab78 | 0xab7d | 0xab82 | 0xab87 => {
                w.operand_then_internal(2);
                let offset = match self.pc {
                    0x8edc | 0x8f06 | 0x8f1d => 2,
                    0x8ee5 => 4,
                    0xac1a | 0xac1f | 0xac24 | 0xac29 => (self.pc - 0xac1a) / 5 * 2,
                    0xab78 | 0xab7d | 0xab82 | 0xab87 => (self.pc - 0xab78) / 5 * 2,
                    _ => 0,
                };
                let v = w.read(0x009b52 + u32::from(w.cpu.x) + u32::from(offset), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x8ed4 | 0x8edf | 0x8ee8 | 0x8f01 | 0x8f09 | 0x8f1b | 0x8f20 | 0xac1d | 0xac22
            | 0xac27 | 0xac2c | 0xab7b | 0xab80 | 0xab85 | 0xab8a => {
                let a = w.indirect_y_address(match self.pc {
                    0x8ee8 | 0xac22 | 0xab80 => 0xcb,
                    0x8f20 => 0xc2,
                    0xac27 | 0xab85 => 0xd7,
                    0xac2c | 0xab8a => 0xda,
                    _ => 0xbf,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8ed6 | 0x8f22 | 0x9149 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x8ed7 | 0x8f13 | 0x8f23 | 0x916d | 0x9178 | 0xac2f | 0xab8d => {
                w.clear_carry();
                self.pc + 1
            }
            0x8ed8 | 0x8f14 | 0x8f24 | 0x916e | 0x9179 | 0xac30 | 0xab8e => {
                w.drain(16);
                w.adc(match self.pc {
                    0x8ed8 | 0x8f24 => 0x0080,
                    0x916e => 10,
                    0x9179 => 4,
                    _ => 8,
                });
                self.pc + 3
            }
            0x8edb | 0x8f27 | 0x9150 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x8ee1 | 0x8f28 | 0xac36 | 0xab94 => {
                w.dp_increment(
                    if matches!(self.pc, 0xac36 | 0xab94) {
                        0x0e
                    } else {
                        0xb2
                    },
                    2,
                    false,
                );
                self.pc + 2
            }
            0x8f12 | 0xac2e | 0xab8c => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x8f17 | 0xac33 | 0xab91 => {
                w.tax_word();
                self.pc + 1
            }
            0x8f2c..=0x8f2f => {
                w.increment_x();
                self.pc + 1
            }
            0xac34 | 0xac35 | 0xab92 | 0xab93 => {
                w.increment_y();
                self.pc + 1
            }
            0x913f | 0x914d | 0x918b => {
                w.drain(16);
                w.write(0x04b0, if self.pc == 0x913f { w.cpu.y } else { w.cpu.a }, 2);
                self.pc + 3
            }
            0x9142 | 0x916b => {
                let v = w.dp_read(if self.pc == 0x9142 { 0xbf } else { 8 }, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0x9144 => {
                w.compare_immediate(0x4000, 2);
                0x9147
            }
            0x914a | 0x9188 => {
                w.drain(16);
                w.load(w.cpu.a | if self.pc == 0x914a { 0x2000 } else { 0x8000 }, 2);
                self.pc + 3
            }
            0x9151 | 0x9185 => {
                w.drain(16);
                let v = w.read(if self.pc == 0x9151 { 0x0402 } else { 0x04b0 }, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x9154 => {
                w.mask(0x1000, 2);
                0x9157
            }
            0x9159 => {
                w.dp_store(8, w.cpu.y, 2);
                0x915b
            }
            0x915b | 0x9168 => {
                w.drain(16);
                w.cpu.y = 0x0052;
                w.flags(w.cpu.y, 2);
                self.pc + 3
            }
            0x915e | 0x9171 => {
                w.cpu.y = w.dp_read(if self.pc == 0x915e { 8 } else { 0xbf }, 2);
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0x9163 | 0x917f => {
                w.dp_store(0x0e, w.cpu.a, 2);
                self.pc + 2
            }
            0x9173 => {
                w.drain(16);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= 0x4000);
                w.flags(w.cpu.y.wrapping_sub(0x4000), 2);
                0x9176
            }
            0x8eea | 0x8f0b | 0x9184 | 0x918e | 0xac3a | 0xab98 => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0x9210 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                0x9211
            }
            0x8be0 | 0x8bf4 | 0x8c0e | 0x8c22 | 0x8e67 | 0x8e7b | 0x8e95 | 0x8ea9 => {
                w.immediate(4, 2);
                self.pc + 3
            }
            0x8be3 | 0x8bf7 | 0x8c11 | 0x8c25 | 0x8e6a | 0x8e7e | 0x8e98 | 0x8eac | 0x8be6
            | 0x8bfe | 0x8c14 | 0x8c28 | 0x8e6d | 0x8e85 | 0x8e9b | 0x8eaf => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x8be3 | 0x8bf7 | 0x8c11 | 0x8c25 | 0x8e6a | 0x8e7e | 0x8e98 | 0x8eac => 0xb0af,
                    0x8bfe | 0x8e85 => 0xb2d0,
                    _ => 0xb2ce,
                }
            }
            0x8be9 | 0x8c01 | 0x8c17 | 0x8e70 | 0x8e88 | 0x8e9e | 0x8db7 | 0x8dd1 => {
                w.drain(16);
                w.adc(match self.pc {
                    0x8c17 | 0x8e9e => 0x0082,
                    0x8db7 => 0x0100,
                    0x8dd1 => 0x0180,
                    _ => 0x0080,
                });
                self.pc + 3
            }
            0x8bec | 0x8c04 | 0x8c1a | 0x8c2f | 0x8e73 | 0x8e8b | 0x8ea1 | 0x8eb6 | 0x8dd4
            | 0x8da4 => {
                w.dp_store(if self.pc == 0x8da4 { 0x0a } else { 8 }, w.cpu.a, 2);
                self.pc + 2
            }
            0x8bee | 0x8c06 | 0x8c1c | 0x8c31 | 0x8e75 | 0x8e8d | 0x8ea3 | 0x8eb8 | 0x8dba
            | 0x8dc9 | 0x8dd6 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x8bef | 0x8c09 | 0x8c1d | 0x8c32 | 0x8e76 | 0x8e90 | 0x8ea4 | 0x8eb9 | 0x8dca
            | 0x8dd7 => {
                w.dp_increment(
                    match self.pc {
                        0x8dca => 0x0a,
                        0x8dd7 => 0xb4,
                        _ => 0xb2,
                    },
                    2,
                    false,
                );
                self.pc + 2
            }
            0x8bf1 | 0x8c0b | 0x8c1f | 0x8c34 | 0x8e78 | 0x8e92 | 0x8ea6 | 0x8ebb | 0x8dcc
            | 0x8dd9 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x8bf1 => 0x8be6,
                        0x8c0b => 0x8bfc,
                        0x8c1f => 0x8c14,
                        0x8c34 => 0x8c28,
                        0x8e78 => 0x8e6d,
                        0x8e92 => 0x8e83,
                        0x8ea6 => 0x8e9b,
                        0x8ebb => 0x8eaf,
                        0x8dcc => 0x8da6,
                        _ => 0x8da2,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x8bfa | 0x8c07 | 0x8e81 | 0x8e8e | 0x8d9e | 0x8da0 => {
                w.dp_increment(if self.pc == 0x8d9e { 0xb2 } else { 0xb4 }, 2, true);
                self.pc + 2
            }
            0x8bfc | 0x8e83 | 0x8da2 | 0x8dce => {
                let v = w.dp_read(
                    match self.pc {
                        0x8da2 => 0xb2,
                        0x8dce => 8,
                        _ => 0xb4,
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0x8c2b | 0x8eb2 | 0x8dc5 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0x8c2c | 0x8eb3 | 0x8dc6 => {
                w.drain(16);
                w.sbc(if self.pc == 0x8dc6 { 0x00fa } else { 0x007e }, 2);
                self.pc + 3
            }
            0x8da6 | 0x8dbb => {
                w.operand_then_internal(2);
                let v = w.read(0x009b52 + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x8da9 | 0x8dab | 0x8dad | 0x8daf | 0x8db1 | 0x8db3 | 0x8dbe | 0x8dc0 | 0x8dc2 => {
                let a = w.indirect_y_address(match self.pc {
                    0x8da9 | 0x8dbe => 0xbf,
                    0x8dab | 0x8dc0 => 0xc2,
                    0x8dad | 0x8dc2 => 0xc5,
                    0x8daf => 0xcb,
                    0x8db1 => 0xce,
                    _ => 0xd1,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8db5 | 0x8dc4 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x8db6 | 0x8dd0 => {
                w.clear_carry();
                self.pc + 1
            }
            0x8bf3 | 0x8c0d | 0x8c21 | 0x8c36 | 0x8e7a | 0x8e94 | 0x8ea8 | 0x8ebd | 0x8ddb => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0x8cc7 | 0x8cd9 | 0x8ce3 | 0x8ce7 | 0x8d0e | 0x8d22 => {
                let v = w.dp_read(
                    match self.pc {
                        0x8cd9 | 0x8d0e => 8,
                        0x8ce3 => 0xb4,
                        _ => 0xb2,
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0x8cc9 | 0x8cdd | 0x8cf3 | 0x8cf9 | 0x8d06 | 0x8d10 | 0x8d1b | 0x8d34 | 0x8d3d
            | 0x9211 => {
                w.clear_carry();
                self.pc + 1
            }
            0x8cca | 0x8cde | 0x8cf4 | 0x8cfa | 0x8d07 | 0x8d11 | 0x8d1c | 0x9212 => {
                w.drain(16);
                w.adc(match self.pc {
                    0x8cca => 4,
                    0x8cde => 0x0180,
                    0x8cfa => 6,
                    0x8d07 => 8,
                    0x8d11 => 0x0100,
                    _ => 12,
                });
                self.pc + 3
            }
            0x8ccd | 0x8ccf | 0x8cdb | 0x8ce1 | 0x8ce5 | 0x8ce9 | 0x8d14 | 0x8d24 => {
                w.dp_store(
                    match self.pc {
                        0x8ccd => 0xb2,
                        0x8cdb => 4,
                        0x8ce1 | 0x8d14 => 8,
                        0x8ce5 => 0x0e,
                        _ => 0x0a,
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 2
            }
            0x8cd1 | 0x8d37 | 0x8d39 => {
                w.dp_increment(if self.pc == 0x8cd1 { 0xb4 } else { 0xb2 }, 2, true);
                self.pc + 2
            }
            0x8cd3 | 0x8cef | 0x8cfe | 0x8d0b | 0x8d26 | 0x8d47 | 0x8d4d => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x8cd3 | 0x8d26 => 0x8d47,
                    0x8cef | 0x8d0b => 0x9bd9,
                    0x8cfe => 0x9895,
                    0x8d47 => 0x9216,
                    _ => 0x8d80,
                }
            }
            0x8cd6 => {
                w.drain(16);
                w.write(6, w.cpu.x, 2);
                0x8cd9
            }
            0x8ceb | 0x8d20 => {
                w.cpu.y = w.dp_read(8, 2);
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0x8ced => {
                w.cpu.x = w.dp_read(6, 2);
                w.flags(w.cpu.x, 2);
                0x8cef
            }
            0x8cf2 | 0x8d05 | 0x8d1a | 0x8d50 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x8cf7 | 0x8d0a | 0x8d1f | 0x8d55 | 0x9215 => {
                w.tax_word();
                self.pc + 1
            }
            0x8cf8 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                0x8cf9
            }
            0x8cfd | 0x8d40 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x8d01 | 0x8d16 | 0x8d30 | 0x8d56 => {
                w.dp_increment(
                    match self.pc {
                        0x8d16 => 0x0e,
                        0x8d30 => 0xb4,
                        _ => 0x0a,
                    },
                    2,
                    false,
                );
                self.pc + 2
            }
            0x8d03 | 0x8d18 | 0x8d32 | 0x8d58 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x8d03 => 0x8cfe,
                        0x8d18 => 0x8ce7,
                        0x8d32 => 0x8d2c,
                        _ => 0x8d4a,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x8d29 | 0x8d4a | 0x9216 => {
                w.immediate(
                    match self.pc {
                        0x8d29 => 0xff80,
                        0x9216 => 3,
                        _ => 2,
                    },
                    2,
                );
                self.pc + 3
            }
            0x8d2c | 0x8d51 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0x8d2d | 0x8d52 => {
                w.drain(16);
                w.sbc(if self.pc == 0x8d2d { 0x0080 } else { 12 }, 2);
                self.pc + 3
            }
            0x8d35 | 0x8d3e => {
                let v = w.dp_read(if self.pc == 0x8d35 { 8 } else { 0xb2 }, 2);
                w.adc(v);
                self.pc + 2
            }
            0x8d3b => {
                let v = w.dp_read(0xb2, 2);
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(v & 0x8000 != 0);
                let result = v.wrapping_shl(1);
                w.flags(result, 2);
                w.write(0xb2, result, 2);
                0x8d3d
            }
            0x8d41 => {
                w.drain(16);
                w.cpu.x = 0x0590;
                w.flags(w.cpu.x, 2);
                0x8d44
            }
            0x8d44 | 0x8d5a | 0x9219 => {
                w.drain(16);
                match self.pc {
                    0x8d44 => 0x9895,
                    0x8d5a => 0x9211,
                    _ => 0x8d80,
                }
            }
            0x9bd9 | 0x9bde | 0x9be3 | 0x9be8 | 0x9bed | 0x9bf2 => {
                w.operand_then_internal(2);
                let v = w.read(
                    0x009b52 + u32::from(w.cpu.x) + u32::from((self.pc - 0x9bd9) / 5 * 2),
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0x9bdc | 0x9be1 | 0x9be6 | 0x9beb | 0x9bf0 | 0x9bf5 => {
                let a = w.indirect_y_address(match self.pc {
                    0x9bdc => 0xbf,
                    0x9be1 => 0xc2,
                    0x9be6 => 0xc5,
                    0x9beb => 0xcb,
                    0x9bf0 => 0xce,
                    _ => 0xd1,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x99ec | 0x9a06 | 0x9a0c | 0x9b50 | 0x8f30 | 0x8f3b | 0x8f5c | 0x9cc9 | 0x9cd4
            | 0x9ce5 | 0x9d04 => {
                w.immediate(
                    match self.pc {
                        0x99ec => 3,
                        0x9a06 => 8,
                        0x9a0c | 0x9b50 => 6,
                        _ => 2,
                    },
                    2,
                );
                self.pc + 3
            }
            0x99ef | 0x9a09 | 0x9a0f | 0x9b53 | 0x8f33 | 0x8f5f | 0x9ce8 => {
                w.drain(16);
                if self.pc == 0x99ef {
                    0x97f0
                } else {
                    0x8d80
                }
            }
            0x8f36 | 0x8f3e | 0x9cc6 | 0x9ccc | 0x9cd7 | 0x9ceb | 0x9cee | 0x9cf6 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x8f36 | 0x9cc6 | 0x9ceb => 0xb0ac,
                    0x9cee | 0x9cf6 => 0x9d04,
                    _ => 0x8d80,
                }
            }
            0x8f39 | 0x8b94 | 0x8b96 => {
                w.dp_increment(if self.pc == 0x8b96 { 0xb4 } else { 0xb2 }, 2, true);
                self.pc + 2
            }
            0x8f41 | 0x8f46 | 0x8f4b | 0x9d09 | 0x9d0e | 0x9d13 | 0x8b9c | 0x8bb5 => {
                w.operand_then_internal(2);
                let offset = match self.pc {
                    0x8f46 | 0x9d0e => 2,
                    0x8f4b | 0x9d13 => 4,
                    _ => 0,
                };
                let v = w.read(0x009b52 + u32::from(w.cpu.x) + offset, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x8f44 | 0x8f49 | 0x8f4e | 0x9d0c | 0x9d11 | 0x9d16 => {
                let a = w.indirect_y_address(match self.pc {
                    0x8f49 => 0xcb,
                    0x8f4e => 0xd7,
                    0x9d11 => 0xc2,
                    0x9d16 => 0xc5,
                    _ => 0xbf,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8b9f | 0x8ba1 | 0x8ba3 | 0x8ba5 | 0x8ba7 | 0x8ba9 | 0x8bab | 0x8bad | 0x8bb8
            | 0x8bba | 0x8bbc | 0x8bbe | 0x8bc0 | 0x8bc2 | 0x8bc4 | 0x8bc6 => {
                let start = if self.pc < 0x8bb0 { 0x8b9f } else { 0x8bb8 };
                let a = w.indirect_y_address(0xbf + (self.pc - start) / 2 * 3);
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8f50 | 0x8f51 => {
                w.increment_y();
                self.pc + 1
            }
            0x8f56..=0x8f5b | 0x9d18..=0x9d1d => {
                w.increment_x();
                self.pc + 1
            }
            0x8f52 | 0x9ccf | 0x9cdb | 0x9cf1 | 0x9cfa | 0x9d24 | 0x8bce | 0x8bdb => {
                w.dp_increment(
                    match self.pc {
                        0x9d24 | 0x8bce => 0x0a,
                        0x8bdb => 0xb4,
                        _ => 0xb2,
                    },
                    2,
                    false,
                );
                self.pc + 2
            }
            0x8f54 | 0x9cd1 | 0x9cdd | 0x9cf3 | 0x9cfc | 0x9d26 | 0x8bd0 | 0x8bdd => {
                let zero = matches!(self.pc, 0x9cd1 | 0x9cf3);
                if w.branch((w.cpu.p & ZERO != 0) == zero) {
                    match self.pc {
                        0x8f54 => 0x8f41,
                        0x9cd1 => 0x9cdf,
                        0x9cdd => 0x9cd3,
                        0x9cf3 => 0x9cfe,
                        0x9cfc => 0x9cf5,
                        0x9d26 => 0x9d09,
                        0x8bd0 => 0x8b9c,
                        _ => 0x8b98,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x9cd3 | 0x9cf5 => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                self.pc + 1
            }
            0x9cda | 0x9cf9 => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x9cdf | 0x9cfe => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x9ce0 | 0x9cff | 0x9d1f | 0x8bb0 | 0x8bd4 => {
                w.clear_carry();
                self.pc + 1
            }
            0x9ce1 | 0x9d00 | 0x9d20 | 0x8bb1 | 0x8bd5 => {
                w.drain(16);
                w.adc(match self.pc {
                    0x9d20 => 0x0080,
                    0x8bb1 => 0x0100,
                    0x8bd5 => 0x0200,
                    _ => 12,
                });
                self.pc + 3
            }
            0x9ce4 | 0x9d03 => {
                w.tax_word();
                self.pc + 1
            }
            0x9d07 | 0x8b9a | 0x8bd8 => {
                w.dp_store(if self.pc == 0x8bd8 { 8 } else { 0x0a }, w.cpu.a, 2);
                self.pc + 2
            }
            0x9d1e | 0x8baf | 0x8bc8 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x9d23 | 0x8bb4 | 0x8bcd | 0x8bda => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x8b98 | 0x8bd2 => {
                let v = w.dp_read(if self.pc == 0x8b98 { 0xb2 } else { 8 }, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0x8bc9 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0x8bca
            }
            0x8bca => {
                w.drain(16);
                w.sbc(0x00f8, 2);
                0x8bcd
            }
            0x9d28 | 0x8bdf | 0x9bf7 => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0x8a92 | 0x8c37 | 0x8a9a | 0x8c3f | 0x8aa7 | 0x8b10 | 0x9113 | 0x9118 | 0x90fb
            | 0x9100 | 0x92fb | 0x9302 | 0x930e | 0x9315 | 0x94df | 0x94e5 | 0x94eb | 0x9123
            | 0x9139 | 0x96dc | 0x96e4 | 0x9704 | 0x971d | 0x970c | 0x9725 | 0x90dc | 0x90e5 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x8a92 => 0xb0be,
                    0x8a9a | 0x8c3f | 0x94e5 | 0x94eb | 0x970c | 0x9725 => 0x97f0,
                    0x9118 | 0x9100 | 0x9302 | 0x9315 => 0x97ed,
                    0x9123 | 0x9139 | 0x90dc | 0x90e5 => 0xb0af,
                    _ => 0xb0ac,
                }
            }
            0x8a95 | 0x8c3a | 0x9111 | 0x90f9 | 0x92fe | 0x9311 | 0x9702 | 0x971b => {
                w.dp_store(0x0a, w.cpu.x, 2);
                self.pc + 2
            }
            0x8a97 | 0x8c3c | 0x94e2 | 0x94e8 | 0x9120 | 0x9136 | 0x9709 | 0x9722 | 0x90d9
            | 0x90e2 => {
                w.immediate(
                    match self.pc {
                        0x94e2 => 1,
                        0x9120 | 0x9136 => 4,
                        0x9722 => 3,
                        0x90d9 | 0x90e2 => 8,
                        _ => 2,
                    },
                    2,
                );
                self.pc + 3
            }
            0x8a9d | 0x8c48 | 0x9116 | 0x90fe | 0x9300 | 0x9313 | 0x9707 | 0x9720 => {
                w.cpu.x = w.dp_read(0x0a, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0x8c42 | 0x9318 | 0x970f | 0x9728 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x8c43 | 0x8b03 | 0x9105 | 0x9319 | 0x94f9 | 0x9710 | 0x9729 => {
                w.clear_carry();
                self.pc + 1
            }
            0x8c44 | 0x8b04 | 0x9106 | 0x931a | 0x94fa | 0x9711 | 0x972a => {
                w.drain(16);
                w.adc(match self.pc {
                    0x8c44 => 8,
                    0x9106 => 0x0200,
                    0x931a => 0x02f8,
                    0x94fa => 0x0010,
                    0x9711 => 0x05fc,
                    0x972a => 0x02fa,
                    _ => 0x0100,
                });
                self.pc + 3
            }
            0x8c47 | 0x8aa6 | 0x8b0f | 0x910b | 0x931d | 0x9714 | 0x972d => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x8a9f | 0x8c4a | 0x8b08 | 0x8b6f | 0x911b | 0x910c | 0x9309 | 0x931e | 0x94f4
            | 0x9715 | 0x972e => {
                w.dp_increment(0xb2, 2, false);
                self.pc + 2
            }
            0x8aa1 | 0x8c4c | 0x8b0a | 0x8b71 | 0x911d | 0x910e | 0x930b | 0x9320 | 0x94f6
            | 0x9717 | 0x9730 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x8aa1 => 0x8a97,
                        0x8c4c => 0x8c3c,
                        0x8b0a => 0x8aaa,
                        0x911d => 0x9116,
                        0x910e => 0x90fe,
                        0x930b => 0x9300,
                        0x9320 => 0x9313,
                        0x94f6 => 0x94e8,
                        0x9717 => 0x9707,
                        0x9730 => 0x9720,
                        _ => 0x8b13,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x8aa4 | 0x8b0d | 0x8b02 | 0x94ee | 0x94f8 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x8aa5 | 0x8b0e => {
                w.drain(6);
                w.cpu.x = w.cpu.y;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x8aaa | 0x8ab5 | 0x8ac0 | 0x8acb | 0x8ad6 | 0x8ae1 | 0x8aec | 0x8af7 | 0x8b13
            | 0x8b1e | 0x8b29 | 0x8b34 | 0x8b3f | 0x8b4a | 0x8b55 | 0x8b60 => {
                w.operand_then_internal(2);
                let start = if self.pc < 0x8b00 { 0x8aaa } else { 0x8b13 };
                let offset = (self.pc - start) / 11 * 2;
                let v = w.read(0x009b52 + u32::from(w.cpu.y) + u32::from(offset), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x8aad | 0x8ab1 | 0x8ab8 | 0x8abc | 0x8ac3 | 0x8ac7 | 0x8ace | 0x8ad2 | 0x8ad9
            | 0x8add | 0x8ae4 | 0x8ae8 | 0x8aef | 0x8af3 | 0x8afa | 0x8afe | 0x8b16 | 0x8b1a
            | 0x8b21 | 0x8b25 | 0x8b2c | 0x8b30 | 0x8b37 | 0x8b3b | 0x8b42 | 0x8b46 | 0x8b4d
            | 0x8b51 | 0x8b58 | 0x8b5c | 0x8b63 | 0x8b67 => {
                w.drain(24);
                let vertical = self.pc < 0x8b00;
                let offset = self.pc - if vertical { 0x8aad } else { 0x8b16 };
                let tile = offset / 11;
                let position = if vertical {
                    tile / 4 * 0x80 + tile % 4 * 2
                } else {
                    tile % 4 * 0x80 + tile / 4 * 2
                };
                let base = if offset % 11 == 0 { 0x7e4000 } else { 0x7e2000 };
                w.write(base + u32::from(position) + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0x8b07 | 0x94f3 | 0x94fd => {
                w.tax_word();
                self.pc + 1
            }
            0x9305..=0x9308 => {
                w.increment_y();
                self.pc + 1
            }
            0x9103 => {
                let v = w.dp_read(8, 2);
                w.load(v, 2);
                0x9105
            }
            0x9109 => {
                w.dp_store(8, w.cpu.a, 2);
                0x910b
            }
            0x94ef => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0x94f0
            }
            0x94f0 => {
                w.drain(16);
                w.sbc(0x0010, 2);
                0x94f3
            }
            0x94fe | 0x913c | 0x96e1 | 0x90df => {
                w.drain(16);
                if self.pc == 0x94fe {
                    0xb2e1
                } else {
                    0xb2ce
                }
            }
            0xb2e1 | 0xb2e6 | 0xb2eb | 0xb2f0 => {
                w.operand_then_internal(2);
                let v = w.read(
                    0x009b52 + u32::from(w.cpu.x) + u32::from((self.pc - 0xb2e1) / 5 * 2),
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0xb2e4 | 0xb2e9 | 0xb2ee | 0xb2f3 => {
                let a = w.indirect_y_address(match self.pc {
                    0xb2e4 => 0xbf,
                    0xb2e9 => 0xcb,
                    0xb2ee => 0xd7,
                    _ => 0xda,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8b6b..=0x8b6e => {
                w.increment_x();
                self.pc + 1
            }
            0x96df | 0x96e7 => {
                w.dp_increment(0xb2, 2, true);
                self.pc + 2
            }
            0x8f8a | 0x8a89 | 0x8c4f | 0x96f9 | 0x94b4 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                if self.pc == 0x8a89 {
                    0xb0be
                } else {
                    0xb0ac
                }
            }
            0x8a8c | 0x8c52 | 0x96fc | 0x94b7 => {
                w.immediate(
                    match self.pc {
                        0x8a8c => 0x0100,
                        0x8c52 => 0x0300,
                        0x96fc => 0x0018,
                        _ => 8,
                    },
                    2,
                );
                self.pc + 3
            }
            0x8a8f | 0x8c55 | 0x96ff | 0x94ba => {
                w.drain(16);
                0xb220
            }
            0xb220 => {
                w.dp_store(0x0e, w.cpu.a, 2);
                0xb222
            }
            0x8f8d | 0x9126 | 0x96e9 | 0x90e8 | 0xb222 | 0xb227 | 0xb22c | 0xb231 | 0xb236
            | 0xb23b | 0xb240 | 0xb245 => {
                w.operand_then_internal(2);
                let offset = if matches!(self.pc, 0x8f8d | 0x9126 | 0x96e9 | 0x90e8) {
                    0
                } else {
                    (self.pc - 0xb222) / 5 * 2
                };
                let v = w.read(0x009b52 + u32::from(w.cpu.x) + u32::from(offset), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x8f90 | 0x9129 | 0x96ec | 0x90eb | 0xb225 | 0xb22a | 0xb22f | 0xb234 | 0xb239
            | 0xb23e | 0xb243 | 0xb248 => {
                let pointer = if matches!(self.pc, 0x8f90 | 0x9129 | 0x96ec | 0x90eb) {
                    0xbf
                } else {
                    0xbf + (self.pc - 0xb225) / 5 * 3
                };
                let a = w.indirect_y_address(pointer);
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8f92 | 0xb24a | 0x912b | 0x96ee | 0x90ed => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x8f93 | 0xb24b | 0x912c | 0x96ef | 0x90ee => {
                w.clear_carry();
                self.pc + 1
            }
            0x8f94 | 0x912d | 0x96f0 | 0x90ef => {
                w.drain(16);
                w.adc(0x0080);
                self.pc + 3
            }
            0xb24c => {
                let v = w.dp_read(0x0e, 2);
                w.adc(v);
                0xb24e
            }
            0x8f97 | 0xb24e | 0x9130 | 0x96f3 | 0x90f2 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x8f98 | 0xb24f | 0x9131 | 0x96f4 | 0x90f3 => {
                w.dp_increment(0xb2, 2, false);
                self.pc + 2
            }
            0x8f9a | 0xb251 | 0x9133 | 0x96f6 | 0x90f5 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x8f9a => 0x8f8d,
                        0x9133 => 0x9126,
                        0x96f6 => 0x96e9,
                        0x90f5 => 0x90e8,
                        _ => 0xb222,
                    }
                } else {
                    self.pc + 2
                }
            }
            0xb0c4 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0xb0c9
                } else {
                    0xb0c6
                }
            }
            0xb0c6 => {
                w.immediate(26, 2);
                0xb0c9
            }
            0x8c58 | 0x8c61 | 0x8c6d | 0x8cbc | 0x8c73 | 0x8cc2 => {
                w.immediate(
                    match self.pc {
                        0x8c58 | 0x8c61 => 7,
                        0x8c6d | 0x8cbc => 6,
                        0x8c73 => 0xff82,
                        _ => 0x0082,
                    },
                    2,
                );
                self.pc + 3
            }
            0x8c5b | 0x8c64 | 0x8c70 | 0x8cbf | 0xb293 | 0xb2a1 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                if matches!(self.pc, 0xb293 | 0xb2a1) {
                    0xb279
                } else {
                    0xb0af
                }
            }
            0x8c5e | 0x8c67 => {
                w.drain(16);
                if self.pc == 0x8c5e {
                    0xb2aa
                } else {
                    0xb29c
                }
            }
            0x8cc5 => {
                w.branch(true);
                0x8c76
            }
            0x8c6a | 0x8cb9 | 0x8caf => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x8c6b | 0x8cba => {
                w.drain(6);
                w.cpu.x = w.cpu.y;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x8c6c | 0x8cbb | 0xb29b | 0xb2a9 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x8c76 => {
                w.dp_store(0x0e, w.cpu.a, 2);
                0x8c78
            }
            0x8c78 | 0x8c83 | 0x8c8e | 0x8c99 | 0x8ca4 | 0xb279 | 0xb27e | 0xb283 | 0xb288
            | 0xb28d => {
                w.operand_then_internal(2);
                let (index, offset) = if self.pc < 0x9000 {
                    (w.cpu.y, (self.pc - 0x8c78) / 11 * 2)
                } else {
                    (w.cpu.x, (self.pc - 0xb279) / 5 * 2)
                };
                let v = w.read(0x009b52 + u32::from(index) + u32::from(offset), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x8c7b | 0x8c7f | 0x8c86 | 0x8c8a | 0x8c91 | 0x8c95 | 0x8c9c | 0x8ca0 | 0x8ca7
            | 0x8cab => {
                w.drain(24);
                let offset = self.pc - 0x8c7b;
                let base = if offset % 11 == 0 { 0x7e4000 } else { 0x7e2000 };
                w.write(
                    base + u32::from(offset / 11) * 0x80 + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                self.pc + 4
            }
            0xb27c | 0xb281 | 0xb286 | 0xb28b | 0xb290 => {
                let a = w.indirect_y_address(match self.pc {
                    0xb27c => 0xbf,
                    0xb281 => 0xcb,
                    0xb286 => 0xd7,
                    0xb28b => 0xda,
                    _ => 0xdd,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x8cb0 | 0xb297 => {
                w.clear_carry();
                self.pc + 1
            }
            0x8cb1 => {
                let v = w.dp_read(0x0e, 2);
                w.adc(v);
                0x8cb3
            }
            0x8cb3 => {
                w.tax_word();
                0x8cb4
            }
            0x8cb4 | 0xb29c | 0xb2aa => {
                w.dp_increment(0xb2, 2, false);
                self.pc + 2
            }
            0x8cb6 | 0xb29e | 0xb2ac => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x8cb6 => 0x8c78,
                        0xb29e => 0xb293,
                        _ => 0xb2a1,
                    }
                } else {
                    self.pc + 2
                }
            }
            0xb296 | 0xb2a4 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0xb298 => {
                w.drain(16);
                w.adc(0x0082);
                0xb29b
            }
            0xb2a5 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb2a6
            }
            0xb2a6 => {
                w.drain(16);
                w.sbc(0x007e, 2);
                0xb2a9
            }

            0x99e6 | 0x9816 | 0x9857 | 0x985f => {
                w.immediate(if self.pc == 0x9857 { 3 } else { 4 }, 2);
                self.pc + 3
            }
            0x99e9 => {
                w.drain(16);
                0x8d80
            }
            0x985a => {
                w.branch(true);
                0x9819
            }
            0x8d80 | 0x9819 | 0x9862 => {
                w.dp_store(0x0e, w.cpu.a, 2);
                self.pc + 2
            }
            0x8d82 | 0x8d87 | 0x8d8c | 0x981b | 0x9826 | 0x9831 | 0x983c | 0x9864 | 0x986f
            | 0x987a => {
                w.operand_then_internal(2);
                let (index, offset) = if self.pc < 0x9000 {
                    (w.cpu.x, (self.pc - 0x8d82) / 5 * 2)
                } else if self.pc >= 0x9864 {
                    (w.cpu.y, (self.pc - 0x9864) / 11 * 2)
                } else {
                    (w.cpu.y, (self.pc - 0x981b) / 11 * 2)
                };
                let v = w.read(0x009b52 + u32::from(index) + u32::from(offset), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x8d85 | 0x8d8a | 0x8d8f => {
                let a = w.indirect_y_address(match self.pc {
                    0x8d85 => 0xbf,
                    0x8d8a => 0xcb,
                    _ => 0xd7,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x981e | 0x9822 | 0x9829 | 0x982d | 0x9834 | 0x9838 | 0x983f | 0x9843 | 0x9867
            | 0x986b | 0x9872 | 0x9876 | 0x987d | 0x9881 => {
                w.drain(24);
                let offset = self.pc - if self.pc >= 0x9867 { 0x9867 } else { 0x981e };
                let base = if offset % 11 == 0 { 0x7e4000 } else { 0x7e2000 };
                w.write(
                    base + u32::from(offset / 11) * 0x80 + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                self.pc + 4
            }
            0x8d91 | 0x9813 | 0x9854 | 0x985c => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x9814 | 0x9855 | 0x985d => {
                w.drain(6);
                w.cpu.x = w.cpu.y;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x9815 | 0x984c | 0x9856 | 0x985e | 0x988a => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x9847 | 0x9885 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x8d92 | 0x9848 | 0x9886 => {
                w.clear_carry();
                self.pc + 1
            }
            0x8d93 | 0x9849 | 0x9887 => {
                w.drain(16);
                w.adc(if self.pc == 0x9849 { 8 } else { 6 });
                self.pc + 3
            }
            0x8d96 => {
                w.tax_word();
                0x8d97
            }
            0x8d97 | 0x8d98 => {
                w.increment_y();
                self.pc + 1
            }
            0x984d | 0x984e | 0x988b | 0x988c => {
                w.increment_x();
                self.pc + 1
            }
            0x8d99 | 0x984f | 0x988d => {
                w.dp_increment(0x0e, 2, false);
                self.pc + 2
            }
            0x8d9b | 0x9851 | 0x988f => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    if self.pc == 0x8d9b {
                        0x8d82
                    } else if self.pc == 0x988f {
                        0x9864
                    } else {
                        0x981b
                    }
                } else {
                    self.pc + 2
                }
            }

            0x97ed | 0x8f65 => {
                w.immediate(if self.pc == 0x97ed { 4 } else { 0x01db }, 2);
                self.pc + 3
            }
            0x97f0 | 0xb191 | 0x8fb4 | 0xb2d0 => {
                w.dp_store(
                    match self.pc {
                        0x8fb4 => 8,
                        0xb2d0 => 0x0a,
                        _ => 0x0e,
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 2
            }
            0x97f2 | 0x97f7 | 0x97fc | 0x9801 | 0x9c3e | 0x8f7c | 0x8f84 | 0xb2d2 => {
                w.operand_then_internal(2);
                let offset = match self.pc {
                    0x97f7 | 0x8f84 => 2,
                    0x97fc => 4,
                    0x9801 => 6,
                    _ => 0,
                };
                let v = w.read(0x009b52 + u32::from(w.cpu.x) + offset, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x97f5 | 0x97fa | 0x97ff | 0x9804 | 0x9c41 | 0x8f7f | 0x8f87 | 0xb2d5 => {
                let a = w.indirect_y_address(match self.pc {
                    0x97fa => 0xcb,
                    0x97ff => 0xd7,
                    0x9804 => 0xda,
                    _ => 0xbf,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x9806 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                0x9807
            }
            0x9807 | 0x8fb0 | 0xb18f | 0xb2df => {
                w.clear_carry();
                self.pc + 1
            }
            0x9808 | 0x8fb1 => {
                w.drain(16);
                w.adc(if self.pc == 0x9808 { 8 } else { 0x0200 });
                self.pc + 3
            }
            0x980b => {
                w.tax_word();
                0x980c
            }
            0x980c | 0x980d | 0xb2cc | 0xb2cd | 0xb2d7 | 0xb2d8 => {
                w.increment_y();
                self.pc + 1
            }
            0x980e | 0x8fa5 | 0x8fa7 | 0x8fb7 | 0xb2d9 => {
                let (a, inc) = match self.pc {
                    0x980e => (0x0e, false),
                    0x8fa5 => (0xb2, true),
                    0x8fa7 => (0xb4, true),
                    0x8fb7 => (0xb4, false),
                    _ => (0x0a, false),
                };
                w.dp_increment(a, 2, inc);
                self.pc + 2
            }
            0x9810 | 0x8fb9 | 0xb2db | 0xb19a | 0x8f6b | 0x8f70 | 0x8f75 | 0x8f7a => {
                let (take, target) = match self.pc {
                    0x9810 => (w.cpu.p & ZERO == 0, 0x97f2),
                    0x8fb9 => (w.cpu.p & ZERO == 0, 0x8fa9),
                    0xb2db => (w.cpu.p & ZERO == 0, 0xb2d5),
                    0xb19a => (w.cpu.p & ZERO != 0, 0xb18f),
                    0x8f6b => (w.cpu.p & CARRY == 0, 0x8f81),
                    _ => (w.cpu.p & ZERO != 0, 0x8f81),
                };
                if w.branch(take) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x8fa9 | 0x8fae | 0xb2ce | 0xb2dd => {
                let v = w.dp_read(
                    if matches!(self.pc, 0x8fa9 | 0xb2ce) {
                        0xb2
                    } else {
                        8
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0x8fab | 0xa80c => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.blocks = Some(super::dungeon_floors::DungeonFloors::blocks());
                self.blocks_tail = false;
                0x8a44
            }
            0x8fb6 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                0x8fb7
            }
            0x8f62 | 0x8f68 | 0x8f81 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x8f62 => 0xb0ac,
                    0x8f68 => 0xb191,
                    _ => 0xb2ca,
                }
            }
            0xb193 => {
                let a = w.indirect_y_address(0xbf);
                let v = w.read(a, 2);
                w.load(v, 2);
                0xb195
            }
            0xb195 => {
                w.mask(0x03ff, 2);
                0xb198
            }
            0xb198 => {
                let v = w.dp_read(0x0e, 2);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.a >= v);
                w.flags(w.cpu.a.wrapping_sub(v), 2);
                0xb19a
            }
            0xb19c => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb19d
            }
            0x8f6d | 0x8f72 | 0x8f77 => {
                w.compare_immediate(
                    match self.pc {
                        0x8f6d => 0x01a6,
                        0x8f72 => 0x01dd,
                        _ => 0x01fc,
                    },
                    2,
                );
                self.pc + 3
            }
            0xb2ca | 0xb2cb => {
                w.increment_x();
                self.pc + 1
            }

            0x9895 | 0x989a | 0x989f | 0x98a4 | 0xb2af | 0xb2b4 | 0xb2b9 | 0xb2be => {
                w.operand_then_internal(2);
                let base = if self.pc < 0xb000 { 0x9895 } else { 0xb2af };
                let v = w.read(
                    0x009b52 + u32::from(w.cpu.x) + u32::from((self.pc - base) / 5 * 2),
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0x9898 | 0x989d | 0x98a2 | 0x98a7 | 0xb2b2 | 0xb2b7 | 0xb2bc | 0xb2c1 => {
                let a = w.indirect_y_address(match self.pc {
                    0x9898 | 0xb2b2 => 0xbf,
                    0x989d | 0xb2b7 => 0xcb,
                    0x98a2 | 0xb2bc => 0xc2,
                    _ => 0xce,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x98a9..=0x98ac => {
                w.increment_y();
                self.pc + 1
            }
            0x8b74 | 0x8b79 | 0x8b7e | 0x8b89 | 0x8b81 | 0x8b8c => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                match self.pc {
                    0x8b74 | 0x8b79 => 0xb0ac,
                    0x8b7e | 0x8b89 => 0xb0cc,
                    0x8b81 => 0xb2af,
                    _ => 0x9895,
                }
            }
            0x8b77 | 0x8b7c | 0x8b86 | 0x8b91 | 0xb0d2 => {
                if w.branch(matches!(self.pc, 0x8b77 | 0x8b7c) || w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x8b77 | 0x8b86 => 0x8b81,
                        0xb0d2 => 0xb0d7,
                        _ => 0x8b8c,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x8b84 | 0x8b8f => {
                w.dp_increment(0xb2, 2, false);
                self.pc + 2
            }
            0xb0ac | 0xb0d4 => {
                w.immediate(if self.pc == 0xb0ac { 1 } else { 32 }, 2);
                self.pc + 3
            }
            0xb0af | 0xb0b9 | 0xb0d7 | 0xb0c9 => {
                w.dp_store(if self.pc == 0xb0af { 0x0e } else { 0xb2 }, w.cpu.a, 2);
                self.pc + 2
            }
            0xb0b1 | 0xb0cc | 0xb0be => {
                let v = w.dp_read(0xb2, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xb0b3 | 0xb0b4 | 0xb0ce | 0xb0cf | 0xb0c0 | 0xb0c1 => {
                w.shift_left(2);
                self.pc + 1
            }
            0xb0b5 | 0xb0d0 | 0xb0c2 => {
                let v = w.dp_read(0xb4, 2);
                w.load(w.cpu.a | v, 2);
                self.pc + 2
            }
            0xb0b7 => {
                let v = w.dp_read(0x0e, 2);
                w.adc(v);
                0xb0b9
            }
            0xb0bb => {
                w.dp_store(0xb4, 0, 2);
                0xb0bd
            }
            0xb2c3 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                0xb2c4
            }
            0xb2c4 => {
                w.clear_carry();
                0xb2c5
            }
            0xb2c5 => {
                w.drain(16);
                w.adc(0x0100);
                0xb2c8
            }
            0xb2c8 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                0xb2c9
            }
            0x98ad | 0x8b88 | 0x8b93 | 0xb0bd | 0xb0d9 | 0xb2c9 | 0x9812 | 0x8fbb | 0x9c43
            | 0x8aa3 | 0x8f89 | 0xb190 | 0xb19d | 0xb2e0 | 0x8d9d | 0x9853 | 0x8cb8 | 0xb292
            | 0xb2a0 | 0xb2ae | 0x8f9c | 0xb253 | 0xb0cb | 0x8b0c | 0x8b73 | 0x8c4e | 0x911f
            | 0x9110 | 0x930d | 0x9322 | 0xb2f5 | 0x9135 | 0x96f8 | 0x90f7 | 0x9719 | 0x9732
            | 0x9d95 | 0x9dd8 | 0xa7ef => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            _ => unreachable!("tile object drawer {:04x}", self.pc),
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
    use std::collections::{BTreeMap, BTreeSet};
    fn data() -> BTreeMap<u32, u8> {
        let mut s = crate::ZeldaState::new();
        s.set_assets(
            &std::fs::read(
                std::env::var_os("ZELDA3_ASSETS").expect("object tests require native assets"),
            )
            .unwrap(),
        )
        .unwrap();
        s.native_dungeon_object_data().unwrap()
    }
    fn rom() -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap()
    }
    #[test]
    fn dungeon_object_native_parameters_targets_and_tiles_match_source() {
        let rom = rom();
        let data = data();
        for (&a, &v) in &data {
            assert_eq!(
                v,
                rom[(a as usize >> 16) * 0x8000 + (a as usize & 0x7fff)],
                "object data {a:06x}"
            );
        }
        eprintln!(
            "{} native object parameter/dispatch/tile bytes match source",
            data.len()
        );
    }
    struct Outcome {
        count: usize,
        visited: BTreeSet<u16>,
        target: Option<u32>,
        draws: usize,
        counter_publications: usize,
        saved_publications: usize,
    }
    fn run(
        stream: &[u8],
        variant: usize,
        draw: bool,
        data: &BTreeMap<u32, u8>,
        rom: &[u8],
    ) -> Outcome {
        run_from(stream, variant, draw, data, rom, 0x88e4)
    }
    fn run_from(
        stream: &[u8],
        variant: usize,
        draw: bool,
        data: &BTreeMap<u32, u8>,
        rom: &[u8],
        entry: u16,
    ) -> Outcome {
        run_prepared(stream, variant, draw, data, rom, entry, None)
    }
    fn run_prepared(
        stream: &[u8],
        variant: usize,
        draw: bool,
        data: &BTreeMap<u32, u8>,
        rom: &[u8],
        entry: u16,
        initial_tile: Option<u16>,
    ) -> Outcome {
        run_configured(stream, variant, draw, data, rom, entry, initial_tile, &[])
    }
    fn run_configured(
        stream: &[u8],
        variant: usize,
        draw: bool,
        data: &BTreeMap<u32, u8>,
        rom: &[u8],
        entry: u16,
        initial_tile: Option<u16>,
        initial_bytes: &[(usize, u8)],
    ) -> Outcome {
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i.wrapping_mul(7) ^ (i >> 8)) as u8;
        }
        if let Some(tile) = initial_tile {
            for bytes in machine.ram[0x2000..0x6000].chunks_exact_mut(2) {
                bytes.copy_from_slice(&tile.to_le_bytes());
            }
        }
        for &(a, value) in initial_bytes {
            machine.ram[a] = value;
        }
        let offset = if variant & 1 == 0 { 0u16 } else { 0x0fff };
        machine.ram[0xb7..0xba].copy_from_slice(&[0, 0x80, 0x7f]);
        machine.ram[0xba..0xbc].copy_from_slice(&offset.to_le_bytes());
        let a = 0x18000 + usize::from(offset);
        machine.ram[a..a + stream.len()].copy_from_slice(stream);
        machine.ram[0x1fc..0x1fe].copy_from_slice(&[0x61, 0x88]);
        let pointers = if variant & 2 == 0 { 0x018719 } else { 0x0186f8 };
        for i in 0..33 {
            machine.ram[0xbf + i] = data[&(pointers + i as u32)];
        }
        let mut cpu = NativeNmiContext {
            a: 0x2468,
            x: 0xabcd,
            y: 0x9876,
            d: 0,
            db: 0,
            bank: 1,
            pc: entry,
            sp: 0x1fb,
            p: if variant & 1 == 0 { 0x41 } else { 0x84 },
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
        let mut source =
            RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                .unwrap();
        clock.begin_synchronous_timeline().unwrap();
        let mut work = DungeonObjects::default();
        work.pc = entry;
        let mut child: Option<DungeonObjectTiles> = None;
        assert_eq!(work.routine(), 0x0188e4);
        let mut out = Outcome {
            count: 0,
            visited: BTreeSet::new(),
            target: None,
            draws: 0,
            counter_publications: 0,
            saved_publications: 0,
        };
        let mut saved_owner: Option<crate::ZeldaState> = None;
        loop {
            if let Some(target) = work.pending_draw() {
                assert!(!work.complete());
                if !draw {
                    out.target = Some(target);
                    assert_eq!(cpu.sp, 0x1f9);
                    break;
                }
                if child.is_none() {
                    child =
                        Some(DungeonObjectTiles::new(target).expect("native tile drawing family"));
                }
            }
            if work.complete() {
                assert_eq!((cpu.pc, cpu.sp), (0x8862, 0x1fd));
                break;
            }
            out.visited.insert(cpu.pc);
            if child.as_ref().is_some_and(|child| child.saved_objects_active()) {
                if saved_owner.is_none() {
                    let mut state = crate::ZeldaState::new();
                    state.ram.copy_from_slice(&ram);
                    state.sync_native_game_state_from_ram();
                    saved_owner = Some(state);
                }
            } else {
                saved_owner = None;
            }
            let mut saved_published = false;
            let label = format!(
                "object stream {stream:02x?} variant{variant} pc{:04x}",
                cpu.pc
            );
            // This source word increment straddles the Somaria byte and the
            // player's timer. Preserve both native owners through overflow;
            // initialize this single-store fixture from native input only.
            let mut counter_owner = if cpu.pc == 0x9c3b {
                let mut state = crate::ZeldaState::new();
                state.ram.copy_from_slice(&ram);
                state.sync_native_game_state_from_ram();
                Some(state)
            } else { None };
            super::super::native_work::compare_instruction_with_devices(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                rom,
                None,
                |_, access| match access {
                    Access::Write(address, value, width) if saved_owner.is_some() => {
                        let state = saved_owner.as_mut().unwrap();
                        if state.native_dungeon_saved_object_store(address, value, width) {
                            saved_published = true;
                            out.saved_publications += 1;
                        } else {
                            assert!(matches!(address, 0..=0xf | 0x100..=0x1ff),
                                "{label}: unowned saved-object store {address:06x}");
                            state.ram[address as usize..address as usize + usize::from(width)]
                                .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                        }
                        None
                    }
                    Access::Write(0x3f4, value, width) if counter_owner.is_some() => {
                        assert!(counter_owner.as_mut().unwrap().native_dungeon_object_tile_store(0x3f4, value, width));
                        None
                    }
                    Access::Read(a, width) if a & 0x8000 != 0 && a < 0x7e0000 => {
                        let mut v = 0;
                        for b in 0..width {
                            let a = a + u32::from(b);
                            v |= u16::from(
                                *data
                                    .get(&a)
                                    .unwrap_or_else(|| panic!("unbound object data {a:06x}")),
                            ) << (8 * b);
                        }
                        Some(v)
                    }
                    _ => None,
                },
                |cpu, clock, bus| {
                    if let Some(child) = child.as_mut() {
                        child.step(cpu, clock, bus).unwrap();
                    } else {
                        work.step(cpu, clock, bus).unwrap();
                    }
                },
            );
            assert_eq!(ram, source.snes().ram, "{label}");
            if let Some(state) = saved_owner.as_ref() {
                assert_eq!(state.ram, ram, "{label}: retained saved-object publications");
                if saved_published {
                    let mut expected = crate::game_state::GameState::load_from_ram(&ram);
                    expected.dungeon.scratch_word = state.game_state.dungeon.scratch_word;
                    assert_eq!(state.game_state.dungeon, expected.dungeon, "{label}: saved-object owners");
                    assert_eq!(state.game_state.player.pushed_block, expected.player.pushed_block,
                        "{label}: adjacent pushed-block banks");
                    assert_eq!(state.game_state.player.follower_link.temp_bunny_timer(),
                        expected.player.follower_link.temp_bunny_timer(), "{label}: tile counter companion");
                }
            }
            if let Some(state) = counter_owner {
                assert_eq!(state.ram, ram, "{label}: exact counter store");
                assert_eq!(state.game_state.dungeon.environment,
                    crate::game_state::GameState::load_from_ram(&ram).dungeon.environment,
                    "{label}: Somaria owner");
                assert_eq!(state.game_state.player.follower_link.temp_bunny_timer(),
                    crate::types::read_le_u16(&ram, 0x3f5), "{label}: timer low byte and preserved high byte");
                out.counter_publications += 1;
            }
            out.count += 1;
            assert!(out.count < 30000, "{label}");
            if child.as_ref().is_some_and(|c| c.complete()) {
                child = None;
                work.resume_after_draw(&cpu);
                out.draws += 1;
            }
            work = work.clone();
            child = child.clone();
        }
        out
    }
    #[test]
    fn saved_object_publications_follow_native_records_and_nested_tile_returns() {
        use super::super::dungeon_data::{OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED};
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut stores = 0;
        let mut instructions = 0;
        for (family, table) in [OBJECT_TARGETS_ORDINARY.as_slice(), OBJECT_TARGETS_EXTENDED.as_slice(), OBJECT_TARGETS_SIZED.as_slice()].into_iter().enumerate() {
            for (kind, &target) in table.iter().enumerate() {
                if !DungeonObjectTiles::new(0x010000 | u32::from(target)).is_some_and(|draw| draw.saved_objects_active()) { continue; }
                for variant in 0..4 {
                    for saved in [0u16, 0xffff] {
                      for misc_index in [0u16, 30, 32, 34] {
                        let mut initial = vec![(0x402, saved as u8), (0x403, (saved >> 8) as u8)];
                        for address in [0x42c, 0x42e, 0x432, 0x45a, 0x496, 0x498] {
                            initial.extend([(address, 0), (address + 1, 0)]);
                        }
                        initial.extend([(0x42c, misc_index as u8), (0x42d, (misc_index >> 8) as u8)]);
                        let stream = saved_object_stream(family, kind as u8, 0, 16, 32);
                        let result = run_configured(&stream, variant, true, &data, &rom, 0x88e4, None, &initial);
                        assert_eq!(result.draws, 2);
                        cases += 1;
                        stores += result.saved_publications;
                        instructions += result.count;
                      }
                    }
                }
            }
        }
        assert!(cases > 0 && stores > 0);
        eprintln!("saved native publication fixtures: {cases} cases, {instructions} instructions, {stores} stores");
    }

    fn door_streams(direction: u8) -> (usize, usize, usize, BTreeSet<u16>) {
        let data = data();
        let rom = rom();
        let mut visited = BTreeSet::new();
        let (mut cases, mut instructions, mut returns) = (0, 0, 0);
        for kind in (0..80u8).step_by(2) {
            // Exploding walls have six destinations; ordinary doors have twelve.
            let positions = if direction == 0 && kind == 0x30 {
                6
            } else {
                12
            };
            for position in 0..positions {
                for slot in [0u16, 3, 7, 8, 11, 15] {
                    for variant in 0..4 {
                        for open in [false, true] {
                            let mut initial = Vec::new();
                            for (a, v) in [
                                (0x0460, slot * 2),
                                (0x068c, if open { 0xffff } else { 0 }),
                                (0x0400, 0x5555),
                                (0x0468, if variant & 1 == 0 { 0 } else { 0x0100 }),
                                (0x044e, (slot & 3) * 2),
                                (0x0450, (slot & 3) * 2),
                                (0x19e0, (slot & 3) * 2),
                                (0x2f, ((variant & 3) * 2) as u16),
                            ] {
                                for (i, byte) in v.to_le_bytes().into_iter().enumerate() {
                                    initial.push((a + i, byte));
                                }
                            }
                            let mut stream = vec![0xf0, 0xff, (position << 4) | direction, kind];
                            // A subsequent real east door consumes the restored parser and
                            // updated primary slot after the preceding door's child returns.
                            let following = direction != 3 && slot < 8;
                            if following {
                                stream.extend([(position << 4) | 3, [0, 8, 0x48, 0x1a][variant]]);
                            }
                            stream.extend([0xff, 0xff]);
                            let out = run_configured(
                                &stream,
                                variant,
                                true,
                                &data,
                                &rom,
                                0x88e4,
                                Some([0, 0x1fff, 0x2000, 0xffff][variant]),
                                &initial,
                            );
                            assert_eq!((out.draws, out.target), (1 + usize::from(following), None));
                            cases += 1;
                            instructions += out.count;
                            returns += out.draws;
                            visited.extend(out.visited);
                        }
                    }
                }
            }
        }
        (cases, instructions, returns, visited)
    }
    #[test]
    fn dungeon_east_doors_match_source_layers_shutters_lists_and_state_returns() {
        let (cases, instructions, returns, visited) = door_streams(3);
        for pc in [
            0xaba3, 0xabb0, 0xabb8, 0xabc5, 0xabcd, 0xabdc, 0xabff, 0xac1a, 0xac3a, 0xaf0b, 0xaf25,
            0xaf4d, 0xaf6f, 0xb050, 0xb05d, 0xb092, 0xb09f, 0xb0da, 0xb159, 0xb1ed, 0xb21f,
        ] {
            assert!(visited.contains(&pc), "missing east door branch {pc:04x}");
        }
        eprintln!("{cases} east door streams matched {instructions} instructions and {returns} native returns across 1 target");
    }
    #[test]
    fn dungeon_west_doors_match_source_adjacent_calls_slot_restore_and_following_doors() {
        let (cases, instructions, returns, visited) = door_streams(2);
        for pc in [
            0xaae1, 0xaaee, 0xaaf6, 0xab03, 0xab11, 0xab26, 0xab36, 0xab39, 0xab3d, 0xab42, 0xab5f,
            0xab78, 0xab98, 0xae47, 0xae55, 0xae58, 0xae5c, 0xae78, 0xae8d, 0xaeb6, 0xaee0, 0xaeef,
            0xb00d, 0xb017, 0xb01d, 0xb04f, 0xabc8, 0xabdc, 0xaef0, 0xb0da, 0xb159, 0xb1ed,
        ] {
            assert!(visited.contains(&pc), "missing west door branch {pc:04x}");
        }
        eprintln!("{cases} west door streams matched {instructions} instructions and {returns} native returns across 1 new target and its east child");
    }
    #[test]
    fn dungeon_south_doors_match_source_entrances_tile_children_layers_and_following_doors() {
        let (cases, instructions, returns, visited) = door_streams(1);
        for pc in [
            0xa98e, 0xa99b, 0xa9a3, 0xa9b0, 0xa9c2, 0xa9ca, 0xa9e1, 0xa9e9, 0xaa07, 0xaa0a, 0xaa14,
            0xaa24, 0xaa2f, 0xaa37, 0xaa48, 0xaa51, 0xaa58, 0xaa5b, 0xaa64, 0xaa6b, 0xaa7a, 0xaaa2,
            0xaabb, 0xaad6, 0xadef, 0xae09, 0xae30, 0xafc8, 0xafd5, 0xb00c, 0xb1aa, 0xb1e0, 0x97ed,
            0x9812, 0x9da2, 0x9dab, 0x9dd8,
        ] {
            assert!(visited.contains(&pc), "missing south door branch {pc:04x}");
        }
        eprintln!("{cases} south door streams matched {instructions} instructions and {returns} native returns across 1 new target and its tile children");
    }
    #[test]
    fn dungeon_north_doors_match_source_masks_curtains_adjacent_calls_and_following_doors() {
        let (cases, instructions, returns, visited) = door_streams(0);
        for pc in [
            0xa82e, 0xa83b, 0xa843, 0xa84b, 0xa85c, 0xa867, 0xa879, 0xa892, 0xa8b6, 0xa8c3, 0xa8d9,
            0xa8fa, 0xa916, 0xa926, 0xa929, 0xa92d, 0xa94f, 0xa968, 0xa983, 0xac4d, 0xac52, 0xac57,
            0xac5a, 0xac5b, 0xac70, 0xac97, 0xacac, 0xacc7, 0xacd1, 0xacd4, 0xace1, 0xaceb, 0xad03,
            0xad22, 0xad25, 0xad40, 0xad4f, 0xad5d, 0xad60, 0xad64, 0xad80, 0xad9a, 0xadc4, 0xadc7,
            0xadd3, 0xaf7f, 0xaf91, 0xafc7, 0xaa66, 0xadd4, 0x97ed, 0x9812, 0xb1a4, 0xb1e0,
        ] {
            assert!(visited.contains(&pc), "missing north door branch {pc:04x}");
        }
        eprintln!("{cases} north door streams matched {instructions} instructions and {returns} native returns across 1 new target and its south and tile children");
    }
    #[test]
    fn dungeon_exploding_wall_matches_source_byte_tags_flags_and_nested_segment_returns() {
        let data = data();
        let rom = rom();
        let mut visited = BTreeSet::new();
        let (mut cases, mut instructions, mut returns) = (0, 0, 0);
        for position in 0..6u8 {
            for slot in [0u16, 3, 8, 15] {
                for variant in 0..4 {
                    for tag in [0u8, 0x20, 0x25, 0x28, 0x21, 0xff] {
                        for open in [false, true] {
                            let mut initial =
                                vec![(0xae, tag), (0xaf, [0xff, 0x20, 0x25, 0x28][variant])];
                            for (a, v) in [
                                (0x0460, slot * 2),
                                (0x068c, if open { 0xffff } else { 0 }),
                                (0xa7, [0, 0x5afd, 0xa502, 0xffff][variant]),
                                (0x0452, [0, 0xfeff, 0x0100, 0xffff][variant]),
                                (0xfc, [0, 0xfdff, 0x0200, 0xffff][variant]),
                            ] {
                                for (i, byte) in v.to_le_bytes().into_iter().enumerate() {
                                    initial.push((a + i, byte));
                                }
                            }
                            let out = run_configured(
                                &[0xf0, 0xff, position << 4, 0x30, 0xff, 0xff],
                                variant,
                                true,
                                &data,
                                &rom,
                                0x88e4,
                                Some([0, 0x1fff, 0x2000, 0xffff][variant]),
                                &initial,
                            );
                            assert_eq!((out.draws, out.target), (1, None));
                            cases += 1;
                            instructions += out.count;
                            returns += out.draws;
                            visited.extend(out.visited);
                        }
                    }
                }
            }
        }
        for pc in [
            0xac5b, 0xac97, 0xac9d, 0xaca1, 0xaca5, 0xaca9, 0xacaa, 0xacac, 0xacb0, 0xacc7, 0xacca,
            0xacd1, 0xacd4, 0xace1, 0xaceb, 0xacfc, 0xad22, 0xad40,
        ] {
            assert!(
                visited.contains(&pc),
                "missing exploding-wall branch {pc:04x}"
            );
        }
        eprintln!("{cases} exploding wall streams matched {instructions} instructions and {returns} native returns across byte tag and live flag variants");
    }
    #[test]
    fn dungeon_object_parser_matches_all_dispatch_families() {
        let data = data();
        let rom = rom();
        let mut count = 0;
        let mut cases = 0;
        let mut visited = BTreeSet::new();
        let mut targets = BTreeSet::new();
        let mut check = |stream: Vec<u8>, variant| {
            let o = run(&stream, variant, false, &data, &rom);
            count += o.count;
            cases += 1;
            visited.extend(o.visited);
            if let Some(t) = o.target {
                targets.insert(t);
            }
        };
        for kind in 0..248u8 {
            for size in 0..16u8 {
                for v in 0..2 {
                    let lo = if v == 0 { 0 } else { 0xf8 };
                    let hi = if v == 0 { 0 } else { 0xfc };
                    check(vec![lo | (size & 3), hi | (size >> 2), kind], v);
                }
            }
        }
        for kind in 0..64u8 {
            for (v, (x, y)) in [(0u8, 0u8), (63, 63), (16, 32), (31, 17)]
                .into_iter()
                .enumerate()
            {
                check(
                    vec![0xfc | (x >> 4), (x << 4) | (y >> 2), kind | (y << 6)],
                    v,
                );
            }
        }
        for kind in 0..128u8 {
            for v in 0..2 {
                check(
                    vec![
                        (if v == 0 { 0 } else { 0xf8 }) | (kind & 3),
                        (if v == 0 { 0 } else { 0xfc }) | ((kind >> 2) & 3),
                        0xf8 | (kind >> 4),
                    ],
                    v,
                );
            }
        }
        for kind in 0..256u16 {
            for pos in 0..16u8 {
                for dir in 0..4u8 {
                    check(
                        vec![
                            0xf0,
                            0xff,
                            (pos << 4) | dir | ((kind as u8 & 3) << 2),
                            kind as u8,
                        ],
                        usize::from(pos & 1),
                    );
                }
            }
        }
        check(vec![0xff, 0xff], 0);
        check(vec![0xf0, 0xff, 0xff, 0xff], 1);
        // Isolated continuation after a door's actual return site. Door
        // drawing itself is not yet owned by this component.
        for variant in 0..2 {
            let o = run_from(&[0, 0, 0xff, 0xff], variant, false, &data, &rom, 0x8910);
            assert_eq!(o.target, None);
            count += o.count;
            cases += 1;
            visited.extend(o.visited);
        }
        assert_eq!(visited.len(), 126); // 88FB is covered by real 2x2 returns below.
        eprintln!("{cases} object dispatch fixtures matched {count} instructions; {} parser positions; {} distinct draw targets",visited.len(),targets.len());
    }
    #[test]
    fn dungeon_object_parser_resumes_after_real_shared_drawer_returns() {
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut visited = BTreeSet::new();
        // Extended and sized records which share the same four-tile child.
        // Each stream then crosses into an empty door section and returns.
        for v in 0..4 {
            for y in [0u8, 17, 62] {
                for x in [0u8, 16, 62] {
                    let stream = [
                        0xfc | (x >> 4),
                        (x << 4) | (y >> 2),
                        24 | (y << 6),
                        (x << 2),
                        (y << 2),
                        0xf9,
                        0xf0,
                        0xff,
                        0xff,
                        0xff,
                    ];
                    let o = run(&stream, v, true, &data, &rom);
                    assert_eq!(o.draws, 2);
                    assert_eq!(o.target, None);
                    cases += 1;
                    count += o.count;
                    draws += o.draws;
                    visited.extend(o.visited);
                }
            }
        }
        for kind in [
            0x00u8, 0xb8, 0xb9, 0x07, 0x08, 0x53, 0x60, 0x92, 0x93, 0x67, 0x68, 0x7d,
        ] {
            for size in 0..16u8 {
                for v in 0..4 {
                    let stream = [0x40 | (size & 3), 0x44 | (size >> 2), kind, 0xff, 0xff];
                    let o = run(&stream, v, true, &data, &rom);
                    assert_eq!(o.draws, 1);
                    assert_eq!(o.target, None);
                    cases += 1;
                    count += o.count;
                    draws += o.draws;
                    visited.extend(o.visited);
                }
            }
        }
        assert!(visited.contains(&0x88fb) && visited.contains(&0x98ad));
        eprintln!(
            "{cases} object streams matched {count} instructions and {draws} native 2x2 returns"
        );
    }
    #[test]
    fn dungeon_object_tile_primitives_preserve_layers_edges_sizes_and_returns() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if !matches!(
                    target,
                    0x8aa3 | 0x97ed | 0x8fa5 | 0x9c3e | 0x8f62 | 0x99e6 | 0x9813
                ) {
                    continue;
                }
                targets.insert(target);
                for size in 0..if family == 0 { 16 } else { 1 } {
                    for variant in 0..4 {
                        let tiles: &[Option<u16>] = if target == 0x8f62 {
                            &[
                                Some(0x01db),
                                Some(0x41db),
                                Some(0x01a6),
                                Some(0x81dd),
                                Some(0x21fc),
                                Some(0),
                                Some(0xffff),
                            ]
                        } else {
                            &[None]
                        };
                        for &tile in tiles {
                            let mut stream = match family {
                                0 => vec![0x40 | (size & 3), 0x44 | (size >> 2), kind as u8],
                                1 => vec![0xfd, 0x04, kind as u8 | 0x40],
                                _ => vec![
                                    0x40 | (kind as u8 & 3),
                                    0x44 | ((kind as u8 >> 2) & 3),
                                    0xf8 | (kind as u8 >> 4),
                                ],
                            };
                            // The next record proves that the drawer returns to
                            // the live stream and its own scratch is not replayed.
                            stream.extend_from_slice(&[0xfc, 0, 24, 0xff, 0xff]);
                            let o = run_prepared(&stream, variant, true, &data, &rom, 0x88e4, tile);
                            assert_eq!((o.draws, o.target), (2, None));
                            cases += 1;
                            count += o.count;
                            draws += o.draws;
                            visited.extend(o.visited);
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 7);
        for pc in [
            0x8d9d, 0x9853, 0x8aa3, 0x9812, 0x9c43, 0x8fbb, 0x8a88, 0x8f7c, 0x8f81, 0xb18f, 0xb19c,
        ] {
            assert!(visited.contains(&pc), "missing primitive path {pc:04x}");
        }
        eprintln!("{cases} primitive object streams matched {count} instructions and {draws} native returns across {} new targets", targets.len());
    }
    #[test]
    fn dungeon_object_diagonals_match_source_signed_steps_sizes_and_layers() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(target, 0x8c58 | 0x8c61 | 0x8c6a | 0x8cb9) {
                continue;
            }
            targets.insert(target);
            for size in 0..16u8 {
                for variant in 0..4 {
                    for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                        let stream = [
                            (x << 2) | (size & 3),
                            (y << 2) | (size >> 2),
                            kind as u8,
                            0xfc,
                            0,
                            24,
                            0xff,
                            0xff,
                        ];
                        let o = run(&stream, variant, true, &data, &rom);
                        assert_eq!((o.draws, o.target), (2, None));
                        cases += 1;
                        count += o.count;
                        draws += o.draws;
                        visited.extend(o.visited);
                    }
                }
            }
        }
        assert_eq!(targets.len(), 4);
        for pc in [0x8c5e, 0x8c67, 0x8cc5, 0x8cb8, 0xb292, 0xb2a0, 0xb2ae] {
            assert!(visited.contains(&pc), "missing diagonal path {pc:04x}");
        }
        eprintln!("{cases} diagonal object streams matched {count} instructions and {draws} native returns across {} targets",targets.len());
    }
    #[test]
    fn dungeon_object_spaced_rows_match_source_sizes_steps_and_returns() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(target, 0x8f8a | 0x8a89 | 0x8c4f | 0x96f9 | 0x94b4) {
                continue;
            }
            targets.insert(target);
            for size in 0..16u8 {
                for variant in 0..4 {
                    for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                        let stream = [
                            (x << 2) | (size & 3),
                            (y << 2) | (size >> 2),
                            kind as u8,
                            0xfc,
                            0,
                            24,
                            0xff,
                            0xff,
                        ];
                        let o = run(&stream, variant, true, &data, &rom);
                        assert_eq!((o.draws, o.target), (2, None));
                        cases += 1;
                        count += o.count;
                        draws += o.draws;
                        visited.extend(o.visited);
                    }
                }
            }
        }
        assert_eq!(targets.len(), 5);
        for pc in [
            0x8a8f, 0x8c55, 0x96ff, 0x94ba, 0x8f9c, 0xb253, 0xb0c6, 0xb0cb,
        ] {
            assert!(visited.contains(&pc), "missing spaced row path {pc:04x}");
        }
        eprintln!("{cases} spaced row object streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_object_repeated_blocks_match_source_children_layers_and_shelf_edges() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(
                target,
                0x8a92
                    | 0x8b0d
                    | 0x8aa4
                    | 0x8c37
                    | 0x9111
                    | 0x90f9
                    | 0x92fb
                    | 0x930e
                    | 0x94df
                    | 0x9120
                    | 0x9136
                    | 0x96dc
                    | 0x96e4
                    | 0x9702
                    | 0x971b
                    | 0x90d9
                    | 0x90e2
            ) {
                continue;
            }
            targets.insert(target);
            for size in 0..16u8 {
                for variant in 0..4 {
                    for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                        let stream = [
                            (x << 2) | (size & 3),
                            (y << 2) | (size >> 2),
                            kind as u8,
                            0xfc,
                            0,
                            24,
                            0xff,
                            0xff,
                        ];
                        let o = run(&stream, variant, true, &data, &rom);
                        assert_eq!((o.draws, o.target), (2, None));
                        cases += 1;
                        count += o.count;
                        draws += o.draws;
                        visited.extend(o.visited);
                    }
                }
            }
        }
        assert_eq!(targets.len(), 17);
        for pc in [
            0x8aa3, 0x8b0c, 0x8b73, 0x8c4e, 0x911f, 0x9110, 0x930d, 0x9322, 0x94e5, 0x94eb, 0x94f0,
            0x94fe, 0xb2f5, 0x9135, 0x913c, 0xb2e0, 0x96df, 0x96e7, 0x9719, 0x9732, 0x90df, 0x90f7,
        ] {
            assert!(
                visited.contains(&pc),
                "missing repeated block path {pc:04x}"
            );
        }
        eprintln!("{cases} repeated block object streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_object_rectangles_and_platforms_match_source_stack_and_end_caps() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if !matches!(
                    target,
                    0x99ec
                        | 0x9a06
                        | 0x9a0c
                        | 0x9b50
                        | 0x8f30
                        | 0x8f36
                        | 0x9cc6
                        | 0x9ceb
                        | 0x8b94
                        | 0x8cc7
                        | 0x9bd9
                ) {
                    continue;
                }
                targets.insert(target);
                for size in 0..if family == 0 { 16u8 } else { 1 } {
                    for variant in 0..4 {
                        for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                            let mut stream = match family {
                                0 => {
                                    vec![(x << 2) | (size & 3), (y << 2) | (size >> 2), kind as u8]
                                }
                                1 => vec![
                                    0xfc | (x >> 4),
                                    (x << 4) | (y >> 2),
                                    (y << 6) | kind as u8,
                                ],
                                _ => vec![
                                    (x << 2) | (kind as u8 & 3),
                                    (y << 2) | ((kind as u8 >> 2) & 3),
                                    0xf8 | (kind as u8 >> 4),
                                ],
                            };
                            stream.extend_from_slice(&[0xfc, 0, 24, 0xff, 0xff]);
                            let o = run(&stream, variant, true, &data, &rom);
                            assert_eq!((o.draws, o.target), (2, None));
                            cases += 1;
                            count += o.count;
                            draws += o.draws;
                            visited.extend(o.visited);
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 11);
        for pc in [
            0x99ef, 0x9a09, 0x9a0f, 0x9b53, 0x8f33, 0x8f5f, 0x9cd3, 0x9cda, 0x9cf5, 0x9cf9, 0x9d28,
            0x8bdf,
        ] {
            assert!(
                visited.contains(&pc),
                "missing rectangle or cannon path {pc:04x}"
            );
        }
        for pc in [0x8d26, 0x8d3b, 0x8d44, 0x8d5a, 0x9219, 0x9bf7] {
            assert!(
                visited.contains(&pc),
                "missing chest platform path {pc:04x}"
            );
        }
        eprintln!("{cases} rectangle and platform object streams matched {count} instructions and {draws} native returns across {} targets",targets.len());
    }
    #[test]
    fn dungeon_object_ceilings_and_square_match_source_live_row_returns() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(
                target,
                0x8d9e | 0x8be0 | 0x8bf4 | 0x8c0e | 0x8c22 | 0x8e67 | 0x8e7b | 0x8e95 | 0x8ea9
            ) {
                continue;
            }
            targets.insert(target);
            for size in 0..16u8 {
                for variant in 0..4 {
                    for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                        let stream = [
                            (x << 2) | (size & 3),
                            (y << 2) | (size >> 2),
                            kind as u8,
                            0xfc,
                            0,
                            24,
                            0xff,
                            0xff,
                        ];
                        let o = run(&stream, variant, true, &data, &rom);
                        assert_eq!((o.draws, o.target), (2, None));
                        cases += 1;
                        count += o.count;
                        draws += o.draws;
                        visited.extend(o.visited);
                    }
                }
            }
        }
        assert_eq!(targets.len(), 9);
        for pc in [
            0x8bf3, 0x8c0d, 0x8c21, 0x8c36, 0x8e7a, 0x8e94, 0x8ea8, 0x8ebd, 0x8ddb,
        ] {
            assert!(visited.contains(&pc), "missing ceiling path {pc:04x}");
        }
        eprintln!("{cases} ceiling object streams matched {count} instructions and {draws} native returns across {} targets",targets.len());
    }
    #[test]
    fn dungeon_moving_walls_match_source_saved_tags_buffers_and_children() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        let mut check = |kind: u8, size: u8, variant, x: u8, y: u8, tags: (u8, u8), saved| {
            let stream = [
                (x << 2) | (size & 3),
                (y << 2) | (size >> 2),
                kind,
                0xfc,
                0,
                24,
                0xff,
                0xff,
            ];
            let o = run_configured(
                &stream,
                variant,
                true,
                &data,
                &rom,
                0x88e4,
                None,
                &[(0xae, tags.0), (0xaf, tags.1), (0x403, saved)],
            );
            assert_eq!((o.draws, o.target), (2, None));
            cases += 1;
            count += o.count;
            draws += o.draws;
            visited.extend(o.visited);
        };
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(target, 0x9190 | 0x921c | 0x9298) {
                continue;
            }
            targets.insert(target);
            // Full geometry independent of the saved-tag gate.
            for size in 0..16 {
                for variant in 0..4 {
                    for (x, y) in [(0, 0), (16, 32), (62, 63)] {
                        check(kind as u8, size, variant, x, y, (0, 0), 0);
                    }
                }
            }
            // Both tag slots, range boundaries, priority and each saved bit.
            for tags in [
                (0, 0),
                (0x1b, 0),
                (0x1c, 0),
                (0x1f, 0),
                (0x20, 0),
                (0, 0x1b),
                (0, 0x1c),
                (0, 0x1f),
                (0, 0x20),
                (0x1c, 0x1c),
                (0x20, 0x1f),
            ] {
                for saved in [0, 8, 16, 24, 255] {
                    for variant in 0..4 {
                        check(kind as u8, 0, variant, 16, 32, tags, saved);
                    }
                }
            }
        }
        assert_eq!(targets.len(), 3);
        for pc in [
            0x9195, 0x9221, 0x92cc, 0x92d0, 0x92ed, 0x92fa, 0x9210, 0x924b,
        ] {
            assert!(visited.contains(&pc), "missing moving wall path {pc:04x}");
        }
        eprintln!("{cases} moving wall streams matched {count} instructions and {draws} native returns across {} targets",targets.len());
    }
    #[test]
    fn dungeon_edges_and_door_switch_match_source_conditions_layers_and_returns() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(target, 0x8eeb | 0x8ebe | 0x8ec3 | 0x8ef0 | 0x8f0c | 0x913f) {
                continue;
            }
            targets.insert(target);
            let tiles: &[Option<u16>] = if matches!(target, 0x8eeb | 0x8ebe | 0x8ec3 | 0x8ef0) {
                &[
                    Some(0),
                    Some(0x00e1),
                    Some(0x00e2),
                    Some(0x00e3),
                    Some(0x40e2),
                    Some(0x80e3),
                    Some(0x03ff),
                    Some(0xffff),
                ]
            } else {
                &[None]
            };
            let saved: &[u8] = if target == 0x913f {
                &[0, 8, 16, 255]
            } else {
                &[0]
            };
            for size in 0..16 {
                for variant in 0..4 {
                    for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                        for &tile in tiles {
                            for &bits in saved {
                                let stream = [
                                    (x << 2) | (size & 3),
                                    (y << 2) | (size >> 2),
                                    kind as u8,
                                    0xfc,
                                    0,
                                    24,
                                    0xff,
                                    0xff,
                                ];
                                let o = run_configured(
                                    &stream,
                                    variant,
                                    true,
                                    &data,
                                    &rom,
                                    0x88e4,
                                    tile,
                                    &[(0x402, 0xa5), (0x403, bits)],
                                );
                                assert_eq!((o.draws, o.target), (2, None));
                                cases += 1;
                                count += o.count;
                                draws += o.draws;
                                visited.extend(o.visited);
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 6);
        for pc in [
            0x8eee, 0x8ec1, 0x8ed1, 0x8efe, 0x8eea, 0x8f0b, 0x8f33, 0xb18f, 0x9184, 0x918e, 0x9178,
            0xac3a, 0xab98,
        ] {
            assert!(
                visited.contains(&pc),
                "missing edge or door-switch path {pc:04x}"
            );
        }
        eprintln!("{cases} conditional edge and door-switch streams matched {count} instructions and {draws} native returns across {} targets",targets.len());
    }
    #[test]
    fn dungeon_object_spacing_floor_selections_and_empty_returns_match_source() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(
                target,
                0x9338
                    | 0x9347
                    | 0x9357
                    | 0x936f
                    | 0x9387
                    | 0x939f
                    | 0x9446
                    | 0x9456
                    | 0x9323
                    | 0x8f9d
                    | 0x8fa2
                    | 0x90f8
                    | 0x971a
                    | 0x8fbc
                    | 0x918f
            ) {
                continue;
            }
            targets.insert(target);
            for pattern in 0..if matches!(target, 0x8f9d | 0x8fa2) {
                16
            } else {
                1
            } {
                for size in 0..16 {
                    for variant in 0..4 {
                        for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                            let stream = [
                                (x << 2) | (size & 3),
                                (y << 2) | (size >> 2),
                                kind as u8,
                                0xfc,
                                0,
                                24,
                                0xff,
                                0xff,
                            ];
                            let o = run_configured(
                                &stream,
                                variant,
                                true,
                                &data,
                                &rom,
                                0x88e4,
                                None,
                                &[
                                    (0x46a, pattern << 4),
                                    (0x46b, 0),
                                    (0x490, (15 - pattern) << 4),
                                    (0x491, 0),
                                ],
                            );
                            assert_eq!((o.draws, o.target), (2, None));
                            cases += 1;
                            count += o.count;
                            draws += o.draws;
                            visited.extend(o.visited);
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 15);
        for pc in [
            0x9346, 0x9356, 0x936e, 0x9386, 0x939e, 0x93b6, 0x9455, 0x9465, 0x9337, 0x8fa0, 0x8fa2,
            0x90f8, 0x971a, 0x8fbc, 0x918f,
        ] {
            assert!(
                visited.contains(&pc),
                "missing spacing/floor/return path {pc:04x}"
            );
        }
        eprintln!("{cases} spacing and floor object streams matched {count} instructions and {draws} native returns across {} targets",targets.len());
    }
    #[test]
    fn dungeon_object_corners_pipes_and_furniture_match_source_stack_and_layers() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if !matches!(
                    target,
                    0x9854
                        | 0x985c
                        | 0x9aa3
                        | 0x9a90
                        | 0x9a8d
                        | 0x9aee
                        | 0x9c3b
                        | 0x9c44
                        | 0xa194
                        | 0xa7a3
                ) {
                    continue;
                }
                targets.insert(target);
                let counters: &[u16] = if target == 0x9c3b {
                    &[0, 0x00ff, 0x7fff, 0xffff]
                } else {
                    &[0]
                };
                for &counter in counters {
                    for size in 0..if family == 0 { 16 } else { 1 } {
                        for variant in 0..4 {
                            for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                                let mut stream = match family {
                                    0 => vec![
                                        (x << 2) | (size & 3),
                                        (y << 2) | (size >> 2),
                                        kind as u8,
                                    ],
                                    1 => vec![
                                        0xfc | (x >> 4),
                                        (x << 4) | (y >> 2),
                                        kind as u8 | (y << 6),
                                    ],
                                    _ => vec![
                                        (x << 2) | (kind as u8 & 3),
                                        (y << 2) | ((kind as u8 >> 2) & 3),
                                        0xf8 | (kind as u8 >> 4),
                                    ],
                                };
                                stream.extend_from_slice(&[0xfc, 0, 24, 0xff, 0xff]);
                                let o = run_configured(
                                    &stream,
                                    variant,
                                    true,
                                    &data,
                                    &rom,
                                    0x88e4,
                                    None,
                                    &[(0x3f4, counter as u8), (0x3f5, (counter >> 8) as u8)],
                                );
                                assert_eq!(o.counter_publications, usize::from(target == 0x9c3b));
                                assert_eq!((o.draws, o.target), (2, None));
                                cases += 1;
                                count += o.count;
                                draws += o.draws;
                                visited.extend(o.visited);
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 10);
        for pc in [
            0x985a, 0x9891, 0x9aa6, 0x9aa0, 0x9a8d, 0x9b17, 0x9c3b, 0x9c4c, 0x9c51, 0x9c95, 0x9cc5,
            0xa1d0, 0xa7b3,
        ] {
            assert!(
                visited.contains(&pc),
                "missing corner/pipe/furniture path {pc:04x}"
            );
        }
        eprintln!("{cases} corner, pipe and furniture streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_object_long_edges_and_tables_match_source_live_tiles_and_nested_returns() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(
                target,
                0x8fbd | 0x9001 | 0x9045 | 0x908f | 0x93b7 | 0x93dc | 0x9429 | 0x8d5d
            ) {
                continue;
            }
            targets.insert(target);
            let tiles: &[Option<u16>] = if matches!(target, 0x8fbd | 0x9001 | 0x9045 | 0x908f) {
                &[
                    Some(0),
                    Some(0xe2),
                    Some(0xe3),
                    Some(0x40e2),
                    Some(0x80e3),
                    Some(0xffff),
                ]
            } else {
                &[None]
            };
            for &tile in tiles {
                for size in 0..16 {
                    for variant in 0..4 {
                        for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                            let stream = [
                                (x << 2) | (size & 3),
                                (y << 2) | (size >> 2),
                                kind as u8,
                                0xfc,
                                0,
                                24,
                                0xff,
                                0xff,
                            ];
                            let o = run_prepared(&stream, variant, true, &data, &rom, 0x88e4, tile);
                            assert_eq!((o.draws, o.target), (2, None));
                            cases += 1;
                            count += o.count;
                            draws += o.draws;
                            visited.extend(o.visited);
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 8);
        for pc in [
            0x8fd4, 0x9000, 0x9018, 0x9044, 0x905c, 0x908e, 0x90a6, 0x90d8, 0x93db, 0x93f4, 0x93f7,
            0x9428, 0x9445, 0x8d6d, 0x8d7d,
        ] {
            assert!(
                visited.contains(&pc),
                "missing long-edge/table path {pc:04x}"
            );
        }
        eprintln!("{cases} long edge and table streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_object_waterfalls_and_bars_match_source_children_and_returns() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(target, 0x9466 | 0x9488 | 0x94bd | 0x97b5 | 0x97dc) {
                continue;
            }
            targets.insert(target);
            for size in 0..16 {
                for variant in 0..4 {
                    for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                        let stream = [
                            (x << 2) | (size & 3),
                            (y << 2) | (size >> 2),
                            kind as u8,
                            0xfc,
                            0,
                            24,
                            0xff,
                            0xff,
                        ];
                        let o = run(&stream, variant, true, &data, &rom);
                        assert_eq!((o.draws, o.target), (2, None));
                        cases += 1;
                        count += o.count;
                        draws += o.draws;
                        visited.extend(o.visited);
                    }
                }
            }
        }
        assert_eq!(targets.len(), 5);
        for pc in [0x9485, 0x94b1, 0x94dc, 0xb305, 0x97db, 0x97ec] {
            assert!(visited.contains(&pc), "missing waterfall/bar path {pc:04x}");
        }
        eprintln!("{cases} waterfall and bar streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_water_overlays_match_source_geometry_saved_flags_and_stair_publication() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(target, 0x9501 | 0x95ef) {
                continue;
            }
            targets.insert(target);
            for size in 0..16 {
                for variant in 0..4 {
                    for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                        for saved in [0u8, 4, 8, 16, 255] {
                            for (horizontal, vertical) in
                                [(0u16, 0u16), (0xff80, 0xfffe), (0x123, 0x456)]
                            {
                                let mut initial = vec![(0x403, saved), (0xaf, 0x1b)];
                                for (address, value) in [
                                    (0x62c, horizontal),
                                    (0x62e, vertical),
                                    (0x414, 0xb6a5),
                                    (0x442, 0x1234),
                                    (0x444, 0x5678),
                                    (0x4ae, 0x9abc),
                                ] {
                                    initial.extend([
                                        (address, value as u8),
                                        (address + 1, (value >> 8) as u8),
                                    ]);
                                }
                                let stream = [
                                    (x << 2) | (size & 3),
                                    (y << 2) | (size >> 2),
                                    kind as u8,
                                    0xfc,
                                    0,
                                    24,
                                    0xff,
                                    0xff,
                                ];
                                let o = run_configured(
                                    &stream, variant, true, &data, &rom, 0x88e4, None, &initial,
                                );
                                assert_eq!((o.draws, o.target), (2, None));
                                cases += 1;
                                count += o.count;
                                draws += o.draws;
                                visited.extend(o.visited);
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 2);
        for pc in [
            0x9566, 0x9568, 0x9579, 0x95d4, 0x95d5, 0x95de, 0x95ee, 0x9663, 0x9667, 0x9684, 0x9687,
            0x96db, 0x8a88,
        ] {
            assert!(visited.contains(&pc), "missing water-overlay path {pc:04x}");
        }
        eprintln!("{cases} water overlay streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_object_holes_and_open_platform_match_source_saved_coordinates_and_borders() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(target, 0x8ddc | 0x9733) {
                continue;
            }
            targets.insert(target);
            for size in 0..16 {
                for variant in 0..4 {
                    for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                        let stream = [
                            (x << 2) | (size & 3),
                            (y << 2) | (size >> 2),
                            kind as u8,
                            0xfc,
                            0,
                            24,
                            0xff,
                            0xff,
                        ];
                        let o = run(&stream, variant, true, &data, &rom);
                        assert_eq!((o.draws, o.target), (2, None));
                        cases += 1;
                        count += o.count;
                        draws += o.draws;
                        visited.extend(o.visited);
                    }
                }
            }
        }
        assert_eq!(targets.len(), 2);
        for pc in [
            0x8de6, 0x8df6, 0x8e66, 0x973a, 0x9757, 0x975a, 0x975c, 0x97ae, 0x97b4,
        ] {
            assert!(visited.contains(&pc), "missing hole/platform path {pc:04x}");
        }
        eprintln!("{cases} hole and open platform streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    fn saved_object_stream(family: usize, kind: u8, size: u8, x: u8, y: u8) -> Vec<u8> {
        let mut stream = match family {
            0 => vec![(x << 2) | (size & 3), (y << 2) | (size >> 2), kind],
            1 => vec![0xfc | (x >> 4), (x << 4) | (y >> 2), kind | (y << 6)],
            _ => vec![
                (x << 2) | (kind & 3),
                (y << 2) | ((kind >> 2) & 3),
                0xf8 | (kind >> 4),
            ],
        };
        stream.extend_from_slice(&[0xfc, 0, 24, 0xff, 0xff]);
        stream
    }
    fn stair_initial_counters(seed: u16) -> Vec<(usize, u8)> {
        let mut bytes = Vec::new();
        for (i, address) in [
            0x438, 0x43a, 0x43c, 0x43e, 0x440, 0x442, 0x444, 0x446, 0x448, 0x47e, 0x480, 0x482,
            0x484, 0x49a, 0x49c, 0x49e, 0x4a2, 0x4a4, 0x4a6, 0x4a8, 0x4ae,
        ]
        .into_iter()
        .enumerate()
        {
            let value = seed + (i as u16 % 4) * 2;
            bytes.extend([(address, value as u8), (address + 1, (value >> 8) as u8)]);
        }
        bytes
    }
    #[test]
    fn dungeon_stairs_match_source_lists_counter_growth_layers_edges_and_returns() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if !matches!(
                    target,
                    0x9b1e
                        | 0xa25d
                        | 0xa26d
                        | 0xa2c7
                        | 0xa2df
                        | 0xa30c
                        | 0xa31c
                        | 0xa36e
                        | 0xa380
                        | 0xa3ae
                        | 0xa41b
                        | 0xa458
                        | 0xa486
                        | 0xa4b4
                        | 0xa4f5
                        | 0xa533
                        | 0xa584
                        | 0xa5d2
                        | 0xa5f4
                        | 0xa607
                        | 0xa626
                        | 0xa664
                        | 0xa695
                        | 0xa71c
                        | 0xa74a
                ) {
                    continue;
                }
                targets.insert(target);
                let tiles: &[Option<u16>] = if matches!(
                    target,
                    0xa4b4 | 0xa4f5 | 0xa533 | 0xa584 | 0xa664 | 0xa695 | 0xa71c | 0xa74a
                ) {
                    &[None, Some(0), Some(0x1fff), Some(0xffff)]
                } else {
                    &[None]
                };
                for seed in [0, 2, 6] {
                    let mut initial = stair_initial_counters(seed);
                    initial.push((0xaf, 0));
                    for &tile in tiles {
                        for variant in 0..4 {
                            for (x, y) in [(0u8, 0u8), (1, 1), (16, 32), (32, 32), (62, 63)] {
                                for size in 0..if family == 0 { 16 } else { 1 } {
                                    for repeat in [1, 3] {
                                        let first =
                                            saved_object_stream(family, kind as u8, size, x, y);
                                        let mut stream = first[..3].repeat(repeat);
                                        stream.extend_from_slice(&first[3..]);
                                        let o = run_configured(
                                            &stream, variant, true, &data, &rom, 0x88e4, tile,
                                            &initial,
                                        );
                                        assert_eq!((o.draws, o.target), (repeat + 1, None));
                                        cases += 1;
                                        count += o.count;
                                        draws += o.draws;
                                        visited.extend(o.visited);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 25);
        for pc in [
            0x9b48, 0xa282, 0xa2c0, 0xa2db, 0xa32b, 0xa369, 0xa37c, 0xa3aa, 0xa41a, 0xa42a, 0xa467,
            0xa495, 0xa4c7, 0xa508, 0xa546, 0xa597, 0xa569, 0xa5b7, 0xa583, 0xa5d1, 0xa663, 0xa674,
            0xa6a5, 0xa6ee, 0xa71b, 0xa72c, 0xa75a, 0xa7a0,
        ] {
            assert!(visited.contains(&pc), "missing stair path {pc:04x}");
        }
        eprintln!("{cases} stair list and drawing streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_conditional_stairs_match_source_tag_room_flags_and_color_publication() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if !matches!(target, 0x9b1e | 0xa2df | 0xa380) {
                    continue;
                }
                targets.insert(target);
                for tag in [0, 0x1a, 0x1b, 0x1c, 0xff] {
                    let rooms: &[u16] = if tag == 0x1b {
                        &[0, 0x65, 0x100, 0x127]
                    } else {
                        &[0]
                    };
                    let flags: &[u16] = if tag == 0x1b {
                        &[0, 0xff, 0x100, 0xffff]
                    } else {
                        &[0]
                    };
                    for &room in rooms {
                        for &saved in flags {
                            for high_tag in [0, 0xff] {
                                for seed in [0, 2, 6] {
                                    let mut initial = stair_initial_counters(seed);
                                    let address = 0xf000 + usize::from(room) * 2;
                                    initial.extend([
                                        (0xaf, tag),
                                        (0xb0, high_tag),
                                        (0xa0, room as u8),
                                        (0xa1, (room >> 8) as u8),
                                        (address, saved as u8),
                                        (address + 1, (saved >> 8) as u8),
                                        (0x414, 0xff),
                                        (0x415, 0xff),
                                        (0x99, 0x55),
                                        (0x9a, 0xaa),
                                    ]);
                                    for variant in 0..4 {
                                        for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                                            for size in 0..if family == 0 { 16 } else { 1 } {
                                                let stream = saved_object_stream(
                                                    family, kind as u8, size, x, y,
                                                );
                                                let o = run_configured(
                                                    &stream, variant, true, &data, &rom, 0x88e4,
                                                    None, &initial,
                                                );
                                                assert_eq!((o.draws, o.target), (2, None));
                                                cases += 1;
                                                count += o.count;
                                                draws += o.draws;
                                                visited.extend(o.visited);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 3);
        for pc in [
            0x9b2c, 0x9b35, 0x9b38, 0xa2c1, 0xa2ed, 0xa2f6, 0xa36a, 0xa36d, 0xa38f, 0xa398, 0xa39b,
            0xa39d,
        ] {
            assert!(
                visited.contains(&pc),
                "missing conditional stair path {pc:04x}"
            );
        }
        eprintln!("{cases} conditional stair streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_large_room_structures_match_source_mirrors_live_tiles_and_returns() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if !matches!(
                    target,
                    0x9a12 | 0x9aa9 | 0x9b56 | 0x9de5 | 0x9e30 | 0x9ea3 | 0xa095 | 0xa1d1
                ) {
                    continue;
                }
                targets.insert(target);
                let flags: &[u16] = if target == 0x9aa9 {
                    &[0, 0x0fff, 0x1000, 0x8000, 0xffff]
                } else {
                    &[0]
                };
                let tiles: &[Option<u16>] = if target == 0x9ea3 {
                    &[None, Some(0), Some(0xffff), Some(0x4000)]
                } else {
                    &[None]
                };
                for &saved in flags {
                    for &tile in tiles {
                        for variant in 0..4 {
                            for (x, y) in [(0u8, 0u8), (16, 32), (31, 31), (32, 32), (62, 63)] {
                                for size in 0..if family == 0 { 16 } else { 1 } {
                                    let stream =
                                        saved_object_stream(family, kind as u8, size, x, y);
                                    let initial =
                                        [(0x402, saved as u8), (0x403, (saved >> 8) as u8)];
                                    let o = run_configured(
                                        &stream, variant, true, &data, &rom, 0x88e4, tile, &initial,
                                    );
                                    assert_eq!((o.draws, o.target), (2, None));
                                    cases += 1;
                                    count += o.count;
                                    draws += o.draws;
                                    visited.extend(o.visited);
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 8);
        for pc in [
            0x9a1a, 0x9a65, 0x9ab1, 0x9ac2, 0x9aed, 0x9bd6, 0x8d9d, 0x9dee, 0x9df7, 0x9e00, 0x9e2f,
            0x9ea2, 0x9fdf, 0x9fe2, 0xa078, 0xa083, 0xa094, 0xa193, 0xa254,
        ] {
            assert!(visited.contains(&pc), "missing large-room path {pc:04x}");
        }
        eprintln!("{cases} large room structure streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_floodgate_matches_source_open_closed_water_layout_and_pointer_restore() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if target != 0x9bf8 {
                    continue;
                }
                for saved in [0u16, 0x07ff, 0x0800, 0x1000, 0xffff] {
                    for old_position in [0u16, 0xffff] {
                        let initial = [
                            (0x402, saved as u8),
                            (0x403, (saved >> 8) as u8),
                            (0x470, old_position as u8),
                            (0x471, (old_position >> 8) as u8),
                            (0x472, old_position as u8),
                            (0x473, (old_position >> 8) as u8),
                        ];
                        for variant in 0..4 {
                            for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                                for size in 0..if family == 0 { 16 } else { 1 } {
                                    let stream =
                                        saved_object_stream(family, kind as u8, size, x, y);
                                    let o = run_configured(
                                        &stream, variant, true, &data, &rom, 0x88e4, None, &initial,
                                    );
                                    assert_eq!((o.draws, o.target), (2, None));
                                    cases += 1;
                                    count += o.count;
                                    draws += o.draws;
                                    visited.extend(o.visited);
                                }
                            }
                        }
                    }
                }
            }
        }
        for pc in [
            0x9c09, 0x9c0e, 0x9c11, 0x9c1d, 0x9c20, 0x9c23, 0x9c2c, 0x9c2f, 0x9c31, 0x9c34, 0x9c37,
            0x9c38, 0x9c3a, 0xcc95, 0xcca6, 0xcca8, 0xccab, 0xccc9, 0xcccd, 0xcd1d, 0xcd27, 0xcd34,
            0xcd36,
        ] {
            assert!(visited.contains(&pc), "missing floodgate path {pc:04x}");
        }
        eprintln!("{cases} floodgate streams matched {count} instructions and {draws} native returns across 1 target");
    }
    #[test]
    fn dungeon_room_decorations_match_source_saved_flags_layers_and_nested_returns() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if !matches!(
                    target,
                    0x9a66
                        | 0x9b48
                        | 0x9d5d
                        | 0x9d67
                        | 0x9d6c
                        | 0x9d96
                        | 0x9dd9
                        | 0xa7b6
                        | 0xa7d3
                        | 0xa7dc
                        | 0xa7f0
                        | 0xa809
                        | 0xa255
                ) {
                    continue;
                }
                targets.insert(target);
                let flags: &[u16] = match target {
                    0x9d96 | 0x9dd9 => &[0, 1, 0x7fff, 0x8000, 0xffff],
                    0xa7d3 => &[0, 0xff, 0x100, 0x8000, 0xffff],
                    _ => &[0],
                };
                for &saved in flags {
                    for variant in 0..4 {
                        for (x, y) in [(0u8, 0u8), (16, 32), (31, 31), (32, 32), (62, 63)] {
                            for size in 0..if family == 0 { 16 } else { 1 } {
                                let initial = [
                                    (0x402, saved as u8),
                                    (0x403, (saved >> 8) as u8),
                                    (0xf0ca, saved as u8),
                                    (0xf0cb, (saved >> 8) as u8),
                                ];
                                let stream = saved_object_stream(family, kind as u8, size, x, y);
                                let o = run_configured(
                                    &stream, variant, true, &data, &rom, 0x88e4, None, &initial,
                                );
                                assert_eq!((o.draws, o.target), (2, None));
                                cases += 1;
                                count += o.count;
                                draws += o.draws;
                                visited.extend(o.visited);
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 13);
        for pc in [
            0x9a66, 0x9b4d, 0xb253, 0x9d5d, 0x9d67, 0x9d6c, 0x9d95, 0x9d9a, 0x9d9c, 0x9dab, 0x9dbb,
            0x9dd8, 0x9ddd, 0x9ddf, 0xa7b9, 0xa7c6, 0xa7d0, 0xa7da, 0xa7e2, 0xa7ef, 0xa7fa, 0xa7fe,
            0xa806, 0xa80c, 0xa80f, 0xa819, 0x8a88, 0xa25a, 0x8a43,
        ] {
            assert!(
                visited.contains(&pc),
                "missing room-decoration path {pc:04x}"
            );
        }
        eprintln!("{cases} room decoration streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_empty_water_face_matches_source_tag_flags_accumulator_width_and_publication() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if target != 0x9d29 {
                    continue;
                }
                for tag in [0, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0xff] {
                    let rooms: &[u16] = if tag == 0x1b {
                        &[0, 0x65, 0xff, 0x100, 0x127]
                    } else {
                        &[0]
                    };
                    let room_flags: &[u16] = if tag == 0x1b {
                        &[0, 0xff, 0x100, 0xffff]
                    } else {
                        &[0]
                    };
                    let live_flags: &[u8] = if tag == 0x19 {
                        &[0, 7, 8, 0x80, 0xff]
                    } else {
                        &[0]
                    };
                    for &room in rooms {
                        for &saved in room_flags {
                            for &live in live_flags {
                                for high_tag in [0, 0xff] {
                                    let address = 0xf000 + usize::from(room) * 2;
                                    let initial = [
                                        (0xaf, tag),
                                        (0xb0, high_tag),
                                        (0xa0, room as u8),
                                        (0xa1, (room >> 8) as u8),
                                        (0x403, live),
                                        (address, saved as u8),
                                        (address + 1, (saved >> 8) as u8),
                                    ];
                                    for variant in 0..4 {
                                        for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                                            for size in 0..if family == 0 { 16 } else { 1 } {
                                                let stream = saved_object_stream(
                                                    family, kind as u8, size, x, y,
                                                );
                                                let o = run_configured(
                                                    &stream, variant, true, &data, &rom, 0x88e4,
                                                    None, &initial,
                                                );
                                                assert_eq!((o.draws, o.target), (2, None));
                                                cases += 1;
                                                count += o.count;
                                                draws += o.draws;
                                                visited.extend(o.visited);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        for pc in [
            0x9d33, 0x9d37, 0x9d40, 0x9d43, 0x9d45, 0x9d4b, 0x9d4e, 0x9d53, 0x9d55, 0x9d5b, 0x9d5d,
            0x9d95,
        ] {
            assert!(visited.contains(&pc), "missing water-face path {pc:04x}");
        }
        eprintln!("{cases} empty water face streams matched {count} instructions and {draws} native returns across 1 target");
    }
    #[test]
    fn dungeon_special_pots_rocks_and_bomb_floors_match_source_records_and_saved_state() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if !matches!(target, 0xb306 | 0xb30b | 0xb310 | 0xb3e1) {
                    continue;
                }
                targets.insert(target);
                let rooms: &[u16] = if target == 0xb3e1 {
                    &[0x64, 0x65, 0x66, 0x165]
                } else {
                    &[0]
                };
                let flags: &[u16] = if target == 0xb3e1 {
                    &[0, 0x0800, 0x1000, 0xffff]
                } else {
                    &[0]
                };
                for &room in rooms {
                    for &saved in flags {
                        for counter in [0u16, 2, 24] {
                            for variant in 0..4 {
                                for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                                    for tile in [None, Some(0), Some(0xffff), Some(0x5a5a)] {
                                        let initial = [
                                            (0xa0, room as u8),
                                            (0xa1, (room >> 8) as u8),
                                            (0x402, saved as u8),
                                            (0x403, (saved >> 8) as u8),
                                            (0x42c, counter as u8),
                                            (0x42d, (counter >> 8) as u8),
                                        ];
                                        for size in 0..if family == 0 { 16 } else { 1 } {
                                            let stream =
                                                saved_object_stream(family, kind as u8, size, x, y);
                                            let o = run_configured(
                                                &stream, variant, true, &data, &rom, 0x88e4, tile,
                                                &initial,
                                            );
                                            assert_eq!((o.draws, o.target), (2, None));
                                            cases += 1;
                                            count += o.count;
                                            draws += o.draws;
                                            visited.extend(o.visited);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 4);
        for pc in [
            0xb309, 0xb30e, 0xb318, 0xb321, 0xb331, 0xb337, 0xb357, 0xb35e, 0xb363, 0xb368, 0xb36d,
            0xb372, 0xb373, 0xb3e8, 0xb3f0, 0xb3f7, 0xb3fa, 0xb407, 0xb410, 0xb420, 0xb426, 0xb446,
            0xb44d, 0xb450, 0xb456, 0xb45c, 0xb462, 0xb46f, 0xb470, 0xb471, 0x8e66,
        ] {
            assert!(
                visited.contains(&pc),
                "missing persistent object path {pc:04x}"
            );
        }
        eprintln!("{cases} special pot, rock and bomb floor streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_saved_objects_match_source_lists_torches_world_and_real_returns() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if !matches!(target, 0x9892 | 0x99b8 | 0x9a00 | 0x9a6f | 0xb395 | 0xb493) {
                    continue;
                }
                targets.insert(target);
                let counters: &[u16] = if target == 0x9892 {
                    &[0, 0xff, 0x7fff, 0xffff]
                } else {
                    &[0, 2, 30]
                };
                let worlds: &[u16] = if target == 0xb395 {
                    &[0, 1, 0x100, 0xffff]
                } else {
                    &[0]
                };
                for &counter in counters {
                    for &world in worlds {
                        for size in 0..if family == 0 { 16 } else { 1 } {
                            for variant in 0..4 {
                                for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                                    let stream =
                                        saved_object_stream(family, kind as u8, size, x, y);
                                    let initial = [
                                        (0x45a, counter as u8),
                                        (0x45b, (counter >> 8) as u8),
                                        (0x432, counter as u8),
                                        (0x433, (counter >> 8) as u8),
                                        (0x42c, counter as u8),
                                        (0x42d, (counter >> 8) as u8),
                                        (0xf3ca, world as u8),
                                        (0xf3cb, (world >> 8) as u8),
                                    ];
                                    let o = run_configured(
                                        &stream, variant, true, &data, &rom, 0x88e4, None, &initial,
                                    );
                                    assert_eq!((o.draws, o.target), (2, None));
                                    cases += 1;
                                    count += o.count;
                                    draws += o.draws;
                                    visited.extend(o.visited);
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 6);
        for pc in [
            0x9892, 0x99b8, 0x9a03, 0x9a7f, 0x9a8c, 0xb3b5, 0xb3d4, 0xb3db, 0xb4b3, 0xb4d2, 0xb4d3,
        ] {
            assert!(visited.contains(&pc), "missing saved-object path {pc:04x}");
        }
        eprintln!("{cases} saved object list streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_chests_and_key_locks_match_source_saved_slots_tags_and_early_returns() {
        use super::super::dungeon_data::{
            OBJECT_TARGETS_EXTENDED, OBJECT_TARGETS_ORDINARY, OBJECT_TARGETS_SIZED,
        };
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        let mut tag_pairs = BTreeSet::new();
        for tag in [
            0, 0x26, 0x27, 0x28, 0x29, 0x32, 0x33, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0xff,
        ] {
            tag_pairs.extend([(tag, 0), (0, tag), (tag, 0x27), (0x27, tag)]);
        }
        for (family, table) in [
            OBJECT_TARGETS_ORDINARY.as_slice(),
            OBJECT_TARGETS_EXTENDED.as_slice(),
            OBJECT_TARGETS_SIZED.as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            for (kind, &target) in table.iter().enumerate() {
                if !matches!(target, 0x98ae | 0x98d0 | 0x99bb) {
                    continue;
                }
                targets.insert(target);
                for slot in 0..6u16 {
                    let mask = 0x100u16 << slot;
                    let flags: BTreeSet<u16> = [
                        0,
                        mask,
                        0x100,
                        0x200,
                        0x300,
                        mask | 0x100,
                        mask | 0x200,
                        0xffff,
                    ]
                    .into_iter()
                    .collect();
                    let mut configurations = vec![];
                    if target == 0x98d0 {
                        for &(first, second) in &tag_pairs {
                            configurations.push((first, second, 0u8));
                        }
                        configurations.extend([
                            (0, 0, 0x1a),
                            (0x27, 0x3e, 0x1a),
                            (0x33, 0x29, 0x1a),
                        ]);
                    } else {
                        configurations.push((0, 0, 0));
                    }
                    for &saved in &flags {
                        for &(first, second, module) in &configurations {
                            for size in 0..if family == 0 { 16 } else { 1 } {
                                for variant in 0..4 {
                                    for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                                        let initial = [
                                            (0x496, (slot * 2) as u8),
                                            (0x497, 0),
                                            (0x498, (slot * 2) as u8),
                                            (0x499, 0),
                                            (0x402, saved as u8),
                                            (0x403, (saved >> 8) as u8),
                                            (0xae, first),
                                            (0xaf, second),
                                            (0x10, module),
                                            (0x11, 0x7b),
                                        ];
                                        let stream =
                                            saved_object_stream(family, kind as u8, size, x, y);
                                        let o = run_configured(
                                            &stream, variant, true, &data, &rom, 0x88e4, None,
                                            &initial,
                                        );
                                        assert_eq!((o.draws, o.target), (2, None));
                                        cases += 1;
                                        count += o.count;
                                        draws += o.draws;
                                        visited.extend(o.visited);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 3);
        for pc in [
            0x98c5, 0x98c7, 0x98cf, 0x98da, 0x98e8, 0x9941, 0x9946, 0x994f, 0x9959, 0x995c, 0x99a9,
            0x99ae, 0x99b8, 0x99cc, 0x99e3, 0x99f2, 0x99fd,
        ] {
            assert!(visited.contains(&pc), "missing chest path {pc:04x}");
        }
        eprintln!("{cases} chest and key-lock streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
    #[test]
    fn dungeon_repeated_pots_and_pegs_match_source_record_growth_and_child_returns() {
        use super::super::dungeon_data::OBJECT_TARGETS_ORDINARY;
        let data = data();
        let rom = rom();
        let mut cases = 0;
        let mut count = 0;
        let mut draws = 0;
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (kind, &target) in OBJECT_TARGETS_ORDINARY.iter().enumerate() {
            if !matches!(target, 0xb376 | 0xb381 | 0xb474 | 0xb47f) {
                continue;
            }
            targets.insert(target);
            for size in 0..16 {
                for variant in 0..4 {
                    for (x, y) in [(0u8, 0u8), (16, 32), (62, 63)] {
                        for world in [0u16, 1, 0x100] {
                            let initial = [
                                (0x42c, 0),
                                (0x42d, 0),
                                (0xf3ca, world as u8),
                                (0xf3cb, (world >> 8) as u8),
                            ];
                            let stream = saved_object_stream(0, kind as u8, size, x, y);
                            let o = run_configured(
                                &stream, variant, true, &data, &rom, 0x88e4, None, &initial,
                            );
                            assert_eq!((o.draws, o.target), (2, None));
                            cases += 1;
                            count += o.count;
                            draws += o.draws;
                            visited.extend(o.visited);
                        }
                    }
                }
            }
        }
        assert_eq!(targets.len(), 4);
        for pc in [0xb380, 0xb394, 0xb47e, 0xb492, 0xb3de, 0xb4d3] {
            assert!(
                visited.contains(&pc),
                "missing repeated pot/peg path {pc:04x}"
            );
        }
        eprintln!("{cases} repeated pot and peg streams matched {count} instructions and {draws} native returns across {} targets", targets.len());
    }
}
