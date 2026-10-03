//! Door drawing retains state-child returns and each layer's live tile updates.
use super::{
    dungeon_door_state::DungeonDoorState,
    dungeon_objects::DungeonObjectTiles,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonDoors {
    pc: u16,
    state: Option<DungeonDoorState>,
    tiles: Option<Box<DungeonObjectTiles>>,
    tiles_tail: bool,
    depth: u8,
    complete: bool,
}
impl DungeonDoors {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(entry, 0xa81c | 0xa984 | 0xaad7 | 0xab99));
        Self {
            pc: entry,
            state: None,
            tiles: None,
            tiles_tail: false,
            depth: 0,
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
        if let Some(state) = self.state.as_mut() {
            let cycles = state.step(cpu, clock, bus);
            self.pc = cpu.pc;
            if state.complete() {
                self.state = None;
            }
            return cycles;
        }
        if let Some(tiles) = self.tiles.as_mut() {
            let cycles = tiles.step(cpu, clock, bus);
            self.pc = cpu.pc;
            if tiles.complete() {
                self.tiles = None;
                if self.tiles_tail {
                    if self.depth == 0 {
                        self.complete = true;
                    } else {
                        self.depth -= 1;
                    }
                }
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
            (
                1,
                self.pc,
                0,
                0,
                if (0xac99..=0xacac).contains(&self.pc) {
                    0x30
                } else {
                    0
                }
            )
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xaad7 | 0xab31 | 0xae52 | 0xab99 => {
                w.operand_then_internal(2);
                let base: u16 = match self.pc {
                    0xaad7 => 0x99ae,
                    0xab99 => 0x99c6,
                    _ => 0x99ba,
                };
                w.cpu.y = w.read(u32::from(base.wrapping_add(w.cpu.x)), 2);
                w.flags(w.cpu.y, 2);
                self.pc + 3
            }
            0xaada | 0xab42 | 0xae5d | 0xab9c | 0xabe2 | 0xaef0 => {
                w.dp_store(8, w.cpu.y, 2);
                self.pc + 2
            }
            0xaadc | 0xaae9 | 0xaaf1 | 0xaafe | 0xab0c | 0xab17 | 0xab9e | 0xabab | 0xabb3
            | 0xabc0 | 0xabc8 | 0xabd7 => {
                w.compare_immediate(
                    match self.pc {
                        0xaadc | 0xab9e => 0x16,
                        0xaae9 | 0xabab => 6,
                        0xaaf1 | 0xabb3 => 0x14,
                        0xab17 | 0xabc0 => 0x40,
                        0xaafe | 0xabc8 => 2,
                        _ => 8,
                    },
                    2,
                );
                self.pc + 3
            }
            0xaba1 | 0xabae | 0xabb6 | 0xabc3 | 0xabcb | 0xabda | 0xabed | 0xabf5 | 0xabfd
            | 0xac38 | 0xaf01 | 0xaf09 | 0xaf4b | 0xb08f | 0xb21d | 0xaadf | 0xaaec | 0xaaf4
            | 0xab01 | 0xab0f | 0xab1a | 0xab24 | 0xab4d | 0xab55 | 0xab5d | 0xab96 | 0xae45
            | 0xae6e | 0xae76 | 0xaedc | 0xb04d => {
                let taken = match self.pc {
                    0xab1a | 0xab24 | 0xab4d | 0xae45 | 0xabc3 | 0xabed => w.cpu.p & CARRY == 0,
                    0xab55 | 0xae6e | 0xabf5 | 0xaf01 => w.cpu.p & ZERO != 0,
                    _ => w.cpu.p & ZERO == 0,
                };
                let target = match self.pc {
                    0xaadf => 0xaae9,
                    0xaaec => 0xaaf1,
                    0xaaf4 => 0xaafe,
                    0xab01 => 0xab0c,
                    0xab0f => 0xab17,
                    0xab1a => 0xab1f,
                    0xab24 => 0xab42,
                    0xab4d => 0xab98,
                    0xab55 => 0xab5f,
                    0xab5d => 0xab6e,
                    0xab96 => 0xab78,
                    0xae45 => 0xae5d,
                    0xae6e => 0xae78,
                    0xae76 => 0xae87,
                    0xaedc => 0xaeb6,
                    0xb04d => 0xb01d,
                    0xaba1 => 0xabab,
                    0xabae => 0xabb3,
                    0xabb6 => 0xabc0,
                    0xabc3 => 0xabc8,
                    0xabcb => 0xabd7,
                    0xabda => 0xabe2,
                    0xabed => 0xac3a,
                    0xabf5 => 0xabff,
                    0xabfd => 0xac0e,
                    0xac38 => 0xac1a,
                    0xaf01 => 0xaf0b,
                    0xaf09 => 0xaf1a,
                    0xaf4b => 0xaf25,
                    0xb08f => 0xb05d,
                    _ => 0xb1ed,
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0xaae1 | 0xaaf6 | 0xab03 | 0xaea9 | 0xaed2 | 0xaba3 | 0xabb8 | 0xabcd | 0xaf41 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0xaae2 | 0xaaf7 | 0xab8d | 0xaeaa | 0xaed3 | 0xaba4 | 0xabb9 | 0xabce | 0xac2f
            | 0xaf42 | 0xaf6b | 0xb058 => {
                w.clear_carry();
                self.pc + 1
            }
            0xaae3 | 0xaaf8 | 0xab8e | 0xaeab | 0xaed4 | 0xaba5 | 0xabba | 0xabcf | 0xac30
            | 0xaf43 | 0xaf6c | 0xb059 => {
                w.drain(16);
                w.adc(match self.pc {
                    0xaae3 | 0xaaf8 => 0x7c,
                    0xaba5 | 0xabba => 0x88,
                    0xb059 => 4,
                    _ => 8,
                });
                self.pc + 3
            }
            0xaae6 | 0xaaee | 0xaafb | 0xab14 | 0xab1c | 0xaba8 | 0xabb0 | 0xabbd | 0xabc5
            | 0xabdf => {
                w.drain(16);
                match self.pc {
                    0xaae6 | 0xaba8 => 0xb09f,
                    0xaaee => 0xb00d,
                    0xabb0 => 0xb050,
                    0xaafb | 0xabbd => 0xb092,
                    0xab1c => 0xae40,
                    0xabc5 => 0xaef0,
                    _ => 0xa8fa,
                }
            }
            0xab0a | 0xabd5 => {
                w.branch(true);
                if self.pc == 0xab0a {
                    0xab1f
                } else {
                    0xabd7
                }
            }
            0xab07 | 0xab11 | 0xab36 | 0xab4a | 0xae55 | 0xae65 | 0xaee0 | 0xb010 | 0xabd2
            | 0xabdc | 0xabea | 0xaef8 | 0xaf6f | 0xb053 | 0xa9d0 | 0xa9f6 | 0xaa3d | 0xaa88
            | 0xaddc | 0xafcb | 0xaa34 | 0xaa58 | 0xaa70 | 0xaa7a | 0xae30 | 0xa85c | 0xa879
            | 0xa8cb | 0xa926 | 0xa93a | 0xac43 | 0xacc7 | 0xaceb | 0xad5d | 0xad6d | 0xadc4
            | 0xaf82 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                if matches!(
                    self.pc,
                    0xab4a
                        | 0xae65
                        | 0xb010
                        | 0xabea
                        | 0xaef8
                        | 0xb053
                        | 0xa9d0
                        | 0xa9f6
                        | 0xaa3d
                        | 0xaa88
                        | 0xaddc
                        | 0xafcb
                        | 0xa8cb
                        | 0xa93a
                        | 0xac43
                        | 0xad6d
                        | 0xaf82
                ) {
                    self.state = Some(DungeonDoorState::default());
                    0xb0da
                } else {
                    self.depth += 1;
                    match self.pc {
                        0xa85c => 0xb1a4,
                        0xa879 => 0xa90f,
                        0xa926 => 0xaa66,
                        0xacc7 => 0xace4,
                        0xaceb => 0xad25,
                        0xad5d => 0xadd4,
                        0xadc4 => 0xaf8b,
                        0xaa34 | 0xaa70 => 0xb1a4,
                        0xaa58 => 0xaa2f,
                        0xaa7a => 0xaa80,
                        0xae30 => 0xafd4,
                        0xab07 | 0xabd2 => 0xb1e7,
                        0xab11 => 0xab1f,
                        0xab36 => 0xabc8,
                        0xae55 => 0xaef0,
                        0xaee0 => 0xb017,
                        0xabdc => 0xabe2,
                        _ => 0xb05c,
                    }
                }
            }
            0xab44 | 0xae5f | 0xaee3 | 0xa8fa | 0xabe4 | 0xaef2 | 0xaf72 | 0xb092 | 0xb09f => {
                w.drain(16);
                w.cpu.x = w.read(
                    match self.pc {
                        0xb092 => 0x0450,
                        0xb09f => 0x044e,
                        _ => 0x0460,
                    },
                    2,
                );
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0xabe7 | 0xabef | 0xabf7 | 0xac15 | 0xaef5 | 0xaefb | 0xaf03 | 0xaf20 | 0xb050
            | 0xb1e8 | 0xab47 | 0xab4f | 0xab57 | 0xab73 | 0xae62 | 0xae68 | 0xae70 | 0xaeb1
            | 0xb00d => {
                w.immediate(
                    match self.pc {
                        0xab57 | 0xabef => 0,
                        0xab4f | 0xabf7 => 0x18,
                        0xae70 | 0xaefb => 0x40,
                        0xae68 | 0xaf03 => 0x44,
                        0xab47 | 0xae62 | 0xaeb1 | 0xb00d | 0xaf20 => 2,
                        0xb1e8 => 5,
                        _ => 3,
                    },
                    2,
                );
                self.pc + 3
            }
            0xab52 | 0xab5a | 0xae6b | 0xae73 | 0xabf2 | 0xabfa | 0xaefe | 0xaf06 => {
                let v = match self.pc {
                    0xab52 | 0xabf2 => 0x36,
                    0xab5a | 0xabfa => 0x38,
                    0xae6b | 0xaefe => 0x48,
                    _ => 0x4a,
                };
                w.drain(16);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= v);
                w.flags(w.cpu.y.wrapping_sub(v), 2);
                self.pc + 3
            }
            0xab5f | 0xab76 | 0xae78 | 0xaeb4 | 0xabff | 0xac18 | 0xaf0b | 0xaf23 | 0xb1eb => {
                w.dp_store(0x0e, w.cpu.a, 2);
                self.pc + 2
            }
            0xab61 | 0xae7a | 0xaee6 | 0xa8fd | 0xac01 | 0xaf0d | 0xaf75 => {
                w.operand_then_internal(2);
                let base: u16 = if matches!(self.pc, 0xab61 | 0xae7a | 0xac01 | 0xaf0d) {
                    0x197e
                } else {
                    0x199e
                };
                let v = w.read(u32::from(base.wrapping_add(w.cpu.x)), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xab04 | 0xb019 | 0xab64 | 0xae7d | 0xac04 | 0xaf10 => {
                w.mask(
                    if matches!(self.pc, 0xab04 | 0xb019) {
                        0xffc0
                    } else {
                        0xff00
                    },
                    2,
                );
                self.pc + 3
            }
            0xab67 | 0xae80 | 0xac07 | 0xaf13 => {
                let v = w.dp_read(0x0e, 2);
                w.load(w.cpu.a | v, 2);
                self.pc + 2
            }
            0xab69 | 0xae82 | 0xaeec | 0xa903 | 0xac09 | 0xaf15 | 0xaf7b | 0xb096 | 0xb0a3 => {
                w.operand_then_internal(2);
                let base: u16 = match self.pc {
                    0xab69 | 0xae82 | 0xac09 | 0xaf15 => 0x197e,
                    0xb096 => 0x06d0,
                    0xb0a3 => 0x06c0,
                    _ => 0x199e,
                };
                w.write(u32::from(base.wrapping_add(w.cpu.x)), w.cpu.a, 2);
                self.pc + 3
            }
            0xab6c | 0xab71 | 0xae85 | 0xac0c | 0xac11 | 0xaf18 => {
                w.cpu.y = w.dp_read(
                    if matches!(self.pc, 0xab71 | 0xac11) {
                        8
                    } else {
                        0x0e
                    },
                    2,
                );
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0xab6e | 0xae87 | 0xac0e | 0xaf1a => {
                w.operand_then_internal(2);
                let base: u16 = if matches!(self.pc, 0xab6e | 0xae87) {
                    0xce66
                } else {
                    0xcec6
                };
                let v = w.read(u32::from(base.wrapping_add(w.cpu.y)), 2);
                if matches!(self.pc, 0xab6e | 0xac0e) {
                    w.cpu.x = v;
                    w.flags(v, 2);
                } else {
                    w.load(v, 2);
                }
                self.pc + 3
            }
            0xab92 | 0xab93 | 0xac13 | 0xac14 | 0xac34 | 0xac35 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xac1a | 0xac1f | 0xac24 | 0xac29 | 0xaf25 | 0xaf2c | 0xaf33 | 0xaf3a | 0xaf4d
            | 0xaf54 | 0xaf5b | 0xaf62 | 0xab78 | 0xab7d | 0xab82 | 0xab87 | 0xae8d | 0xae94
            | 0xae9b | 0xaea2 | 0xaeb6 | 0xaebd | 0xaec4 | 0xaecb => {
                w.operand_then_internal(2);
                let base: u16 = match self.pc {
                    0xab78 | 0xae8d | 0xaeb6 | 0xac1a | 0xaf25 | 0xaf4d => 0x9b52,
                    0xab7d | 0xae94 | 0xaebd | 0xac1f | 0xaf2c | 0xaf54 => 0x9b54,
                    0xab82 | 0xae9b | 0xaec4 | 0xac24 | 0xaf33 | 0xaf5b => 0x9b56,
                    _ => 0x9b58,
                };
                let v = w.read(
                    u32::from(base.wrapping_add(
                        if matches!(
                            self.pc,
                            0xab78 | 0xab7d | 0xab82 | 0xab87 | 0xac1a | 0xac1f | 0xac24 | 0xac29
                        ) {
                            w.cpu.x
                        } else {
                            w.cpu.y
                        },
                    )),
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0xab7b | 0xab80 | 0xab85 | 0xab8a | 0xac1d | 0xac22 | 0xac27 | 0xac2c => {
                let pointer = match self.pc {
                    0xab7b | 0xac1d => 0xbf,
                    0xab80 | 0xac22 => 0xcb,
                    0xab85 | 0xac27 => 0xd7,
                    _ => 0xda,
                };
                let a = w.indirect_y_address(pointer);
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0xab8c | 0xac2e | 0xb08b => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0xab91 | 0xb01c | 0xac33 | 0xb05c | 0xb1e7 => {
                w.drain(6);
                w.cpu.x = w.cpu.a;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xab94 | 0xaeda | 0xac36 | 0xaf49 | 0xb21b => {
                w.dp_increment(0x0e, 2, false);
                self.pc + 2
            }
            0xae8a | 0xaeae | 0xaed7 | 0xaf1d | 0xaf46 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xae8b | 0xaf1e => {
                w.cpu.x = w.dp_read(8, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0xaf28 | 0xaf2f | 0xaf36 | 0xaf3d | 0xaf50 | 0xaf57 | 0xaf5e | 0xaf65 | 0xb064
            | 0xb06f | 0xb07a | 0xb085 | 0xb1f4 | 0xb1ff | 0xb20a | 0xb215 | 0xae90 | 0xae97
            | 0xae9e | 0xaea5 | 0xaeb9 | 0xaec0 | 0xaec7 | 0xaece | 0xb024 | 0xb02f | 0xb03a
            | 0xb045 => {
                let base = match self.pc {
                    0xaf28 => 0x7e4002,
                    0xaf2f => 0x7e4082,
                    0xaf36 => 0x7e4102,
                    0xaf3d => 0x7e4182,
                    0xaf50 => 0x7e2002,
                    0xaf57 => 0x7e2082,
                    0xaf5e => 0x7e2102,
                    0xaf65 => 0x7e2182,
                    0xaeb9 => 0x7e4000,
                    0xaec0 => 0x7e4080,
                    0xaec7 => 0x7e4100,
                    0xaece => 0x7e4180,
                    0xae90 | 0xb024 | 0xb064 | 0xb1f4 => 0x7e2000,
                    0xae97 | 0xb02f | 0xb06f | 0xb1ff => 0x7e2080,
                    0xae9e | 0xb03a | 0xb07a | 0xb20a => 0x7e2100,
                    _ => 0x7e2180,
                };
                w.drain(24);
                w.write(base + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0xaf47 | 0xaf48 | 0xb089 | 0xb08a | 0xb099 | 0xb09a | 0xb0a6 | 0xb0a7 | 0xb219
            | 0xb21a | 0xaeaf | 0xaeb0 | 0xaed8 | 0xaed9 | 0xb049 | 0xb04a => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xaede | 0xb013 | 0xaf69 | 0xb056 => {
                let v = w.dp_read(8, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xb01d | 0xb028 | 0xb033 | 0xb03e | 0xb05d | 0xb068 | 0xb073 | 0xb07e | 0xb1ed
            | 0xb1f8 | 0xb203 | 0xb20e => {
                let base = match self.pc {
                    0xb01d | 0xb05d | 0xb1ed => 0x7e2000,
                    0xb028 | 0xb068 | 0xb1f8 => 0x7e2080,
                    0xb033 | 0xb073 | 0xb203 => 0x7e2100,
                    _ => 0x7e2180,
                };
                w.drain(24);
                let v = w.read(base + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                self.pc + 4
            }
            0xa900 | 0xaf78 | 0xb061 | 0xb06c | 0xb077 | 0xb082 | 0xb1f1 | 0xb1fc | 0xb207
            | 0xb212 | 0xaee9 | 0xb021 | 0xb02c | 0xb037 | 0xb042 => {
                w.drain(16);
                w.load(w.cpu.a | 0x2000, 2);
                self.pc + 3
            }
            0xb08c => {
                w.mask(0x3f, 2);
                0xb08f
            }
            0xb095 | 0xb0a2 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a as u8 & CARRY);
                w.load(w.cpu.a >> 1, 2);
                self.pc + 1
            }
            0xb09b | 0xb0a8 => {
                w.drain(16);
                w.write(if self.pc == 0xb09b { 0x0450 } else { 0x044e }, w.cpu.x, 2);
                self.pc + 3
            }
            0xab98 | 0xaeef | 0xb04f | 0xa906 | 0xac3a | 0xaf7e | 0xb091 | 0xb09e | 0xb0ab
            | 0xb21f | 0xa9bc | 0xaa24 | 0xaad6 | 0xae3f | 0xb00c | 0xb1e0 | 0xa873 | 0xa8bb
            | 0xa983 | 0xac5a | 0xac6f | 0xad40 | 0xadd3 | 0xafc7 => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0xab1f | 0xae40 => {
                w.cpu.x = w.dp_read(2, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0xab21 | 0xae42 => {
                w.compare_x_immediate(12, 2);
                self.pc + 3
            }
            0xab26 | 0xae47 | 0xab2a | 0xae4b => {
                w.drain(6);
                w.push(
                    if matches!(self.pc, 0xab26 | 0xae47) {
                        w.cpu.y
                    } else {
                        w.cpu.a
                    },
                    2,
                );
                self.pc + 1
            }
            0xab27 | 0xae48 => {
                w.drain(16);
                let v = w.read(0x0460, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xab2b | 0xae4c => {
                w.drain(16);
                w.load(w.cpu.a | 0x10, 2);
                self.pc + 3
            }
            0xab2e | 0xab3a | 0xae4f | 0xae59 => {
                w.drain(16);
                w.write(0x0460, w.cpu.a, 2);
                self.pc + 3
            }
            0xab34 | 0xab3e => {
                let v = w.dp_read(4, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xab39 | 0xae58 => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                self.pc + 1
            }
            0xab3d | 0xae5c => {
                w.drain(12);
                w.cpu.y = w.pull(2);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xab40 => {
                w.dp_store(0x0a, w.cpu.a, 2);
                0xab42
            }
            0xb015 | 0xb016 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_add(1), 2);
                self.pc + 1
            }
            0xb017 => {
                w.dp_store(2, w.cpu.a, 2);
                0xb019
            }
            0xb04b => {
                let v = w.dp_read(2, 2);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.x >= v);
                w.flags(w.cpu.x.wrapping_sub(v), 2);
                0xb04d
            }
            0xa9e1 | 0xaa07 | 0xaa48 | 0xac52 | 0xac57 => {
                self.tiles_tail = !matches!(self.pc, 0xaa07 | 0xac57);
                if self.tiles_tail {
                    w.drain(16);
                } else {
                    w.operand_then_internal(2);
                    w.push(self.pc + 2, 2);
                }
                let target = if matches!(self.pc, 0xaa48 | 0xac52 | 0xac57) {
                    0x0197ed
                } else {
                    0x019da2
                };
                self.tiles = Some(Box::new(
                    DungeonObjectTiles::new(target).expect("door tile child"),
                ));
                target as u16
            }
            0xa984 => {
                w.operand_then_internal(2);
                w.cpu.y = w.read(u32::from(0x9996u16.wrapping_add(w.cpu.x)), 2);
                w.flags(w.cpu.y, 2);
                0xa987
            }
            0xa987 | 0xaa80 | 0xadd4 => {
                w.dp_store(8, w.cpu.y, 2);
                self.pc + 2
            }
            0xa989 | 0xa996 | 0xa99e | 0xa9ab | 0xa9bd | 0xa9c5 | 0xa9e4 | 0xaa25 | 0xaa2a
            | 0xaa4b | 0xaa66 | 0xaa75 => {
                w.compare_immediate(
                    match self.pc {
                        0xa989 => 0x16,
                        0xa996 => 6,
                        0xa99e => 0x14,
                        0xa9ab => 0x12,
                        0xa9bd => 0x40,
                        0xa9c5 => 10,
                        0xa9e4 => 12,
                        0xaa25 => 14,
                        0xaa2a => 16,
                        0xaa4b => 4,
                        0xaa66 => 2,
                        _ => 8,
                    },
                    2,
                );
                self.pc + 3
            }
            0xa98c | 0xa999 | 0xa9a1 | 0xa9ae | 0xa9c0 | 0xa9c8 | 0xa9e7 | 0xaa22 | 0xaa28
            | 0xaa2d | 0xaa4e | 0xaa69 | 0xaa78 | 0xaa8b | 0xaa93 | 0xaa98 | 0xaaa0 | 0xaad4
            | 0xade5 | 0xaded | 0xae28 | 0xb00a | 0xb1de => {
                let taken = match self.pc {
                    0xa9c0 | 0xaa8b => w.cpu.p & CARRY == 0,
                    0xaa28 | 0xaa93 | 0xaa98 | 0xade5 => w.cpu.p & ZERO != 0,
                    _ => w.cpu.p & ZERO == 0,
                };
                let target = match self.pc {
                    0xa98c => 0xa996,
                    0xa999 => 0xa99e,
                    0xa9a1 => 0xa9ab,
                    0xa9ae => 0xa9bd,
                    0xa9c0 => 0xa9c5,
                    0xa9c8 => 0xa9e4,
                    0xa9e7 => 0xaa25,
                    0xaa22 => 0xaa14,
                    0xaa28 => 0xaa37,
                    0xaa2d => 0xaa4b,
                    0xaa4e => 0xaa66,
                    0xaa69 => 0xaa75,
                    0xaa78 => 0xaa80,
                    0xaa8b => 0xaad6,
                    0xaa93 | 0xaa98 => 0xaaa2,
                    0xaaa0 => 0xaab1,
                    0xaad4 => 0xaabb,
                    0xade5 => 0xadef,
                    0xaded => 0xadfe,
                    0xae28 => 0xae09,
                    0xb00a => 0xafd5,
                    _ => 0xb1aa,
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0xa98e | 0xa9a3 | 0xa9b3 | 0xa9e9 | 0xaa2f | 0xaa50 | 0xaa6b | 0xae1e => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0xa98f | 0xa9a4 | 0xaa30 | 0xaa5c | 0xaa6c | 0xaacb | 0xae1f | 0xae2c | 0xafd0
            | 0xb002 | 0xb1d7 => {
                w.clear_carry();
                self.pc + 1
            }
            0xa990 | 0xa9a5 | 0xaa31 | 0xaa5d | 0xaa6d | 0xaacc | 0xae20 | 0xae2d | 0xafd1
            | 0xb003 | 0xb1d8 => {
                w.drain(16);
                w.adc(match self.pc {
                    0xa990 | 0xa9a5 => 0x0202,
                    0xaa31 | 0xaa6d | 0xae2d => 0x0200,
                    0xaa5d => 0x0180,
                    0xaacc | 0xae20 => 6,
                    0xafd1 => 0x0100,
                    _ => 0x0080,
                });
                self.pc + 3
            }
            0xa993 | 0xa99b | 0xa9a8 | 0xa9c2 | 0xaa7d => {
                w.drain(16);
                match self.pc {
                    0xa993 => 0xb09f,
                    0xa99b => 0xafc8,
                    0xa9a8 => 0xb092,
                    0xa9c2 => 0xadd4,
                    _ => 0xa8fa,
                }
            }
            0xa9b0 | 0xa9ca | 0xa9f0 | 0xaa37 | 0xaa82 | 0xadd6 | 0xae33 => {
                w.drain(16);
                w.cpu.x = w.read(if self.pc == 0xa9b0 { 0x19e0 } else { 0x0460 }, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0xa9b4 | 0xaaac | 0xadf9 | 0xae3c => {
                w.operand_then_internal(2);
                let base: u16 = match self.pc {
                    0xa9b4 => 0x19e2,
                    0xae3c => 0x199e,
                    _ => 0x197e,
                };
                w.write(u32::from(base.wrapping_add(w.cpu.x)), w.cpu.a, 2);
                self.pc + 3
            }
            0xa9b7 | 0xa9b8 | 0xaa1f | 0xaa20 | 0xae24 | 0xae25 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xa9b9 => {
                w.drain(16);
                w.write(0x19e0, w.cpu.x, 2);
                0xa9bc
            }
            0xa9cd | 0xa9de | 0xa9f3 | 0xaa04 | 0xaa3a | 0xaa45 | 0xaa85 | 0xaa8d | 0xaa9a
            | 0xaab6 | 0xadd9 | 0xaddf | 0xade7 | 0xae04 | 0xafc8 | 0xb1a5 => {
                w.immediate(
                    match self.pc {
                        0xa9de | 0xaa04 | 0xaa45 => 10,
                        0xaa8d => 0,
                        0xaa9a => 0x18,
                        0xaab6 | 0xae04 => 4,
                        0xaddf => 0x40,
                        0xade7 => 0x44,
                        0xb1a5 => 7,
                        _ => 1,
                    },
                    2,
                );
                self.pc + 3
            }
            0xa9d3 | 0xa9f9 | 0xaa0a | 0xae2a | 0xafce => {
                let v = w.dp_read(8, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xa9d5 | 0xa9fb | 0xaa0c => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0xa9d6 | 0xa9fc | 0xaa0d => {
                w.drain(16);
                w.sbc(if self.pc == 0xaa0d { 0x2080 } else { 0x0206 }, 2);
                self.pc + 3
            }
            0xa9d9 | 0xa9ed | 0xa9ff | 0xaa55 => {
                w.dp_store(8, w.cpu.a, 2);
                self.pc + 2
            }
            0xa9db | 0xaa01 | 0xaa11 | 0xaa61 => {
                w.drain(16);
                w.cpu.y = match self.pc {
                    0xaa11 => 10,
                    0xaa61 => 4,
                    _ => 0x2656,
                };
                w.flags(w.cpu.y, 2);
                self.pc + 3
            }
            0xa9ea | 0xaa18 | 0xaa52 | 0xae39 | 0xafd9 | 0xafe4 | 0xafef | 0xaffa | 0xb1ae
            | 0xb1b9 | 0xb1c4 | 0xb1cf => {
                w.drain(16);
                w.load(w.cpu.a | 0x2000, 2);
                self.pc + 3
            }
            0xa9ef | 0xaa57 | 0xae01 | 0xae23 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xaa10 | 0xaa60 | 0xaacf | 0xafd4 | 0xb006 | 0xb1a4 | 0xb1db => {
                w.drain(6);
                w.cpu.x = w.cpu.a;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xaa14 | 0xafd5 | 0xafe0 | 0xafeb | 0xaff6 | 0xb1aa | 0xb1b5 | 0xb1c0 | 0xb1cb => {
                let base = match self.pc {
                    0xaa14 => 0x7e4000,
                    0xafd5 | 0xb1aa => 0x7e2000,
                    0xafe0 | 0xb1b5 => 0x7e2002,
                    0xafeb | 0xb1c0 => 0x7e2004,
                    _ => 0x7e2006,
                };
                w.drain(24);
                let v = w.read(base + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                self.pc + 4
            }
            0xaa1b | 0xae0c | 0xae13 | 0xae1a | 0xafdc | 0xafe7 | 0xaff2 | 0xaffd | 0xb1b1
            | 0xb1bc | 0xb1c7 | 0xb1d2 => {
                let base = match self.pc {
                    0xaa1b | 0xafdc | 0xb1b1 => 0x7e2000,
                    0xae0c => 0x7e4080,
                    0xae13 => 0x7e4100,
                    0xae1a => 0x7e2180,
                    0xafe7 | 0xb1bc => 0x7e2002,
                    0xaff2 | 0xb1c7 => 0x7e2004,
                    _ => 0x7e2006,
                };
                w.drain(24);
                w.write(base + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0xaa21 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1);
                w.flags(w.cpu.y, 2);
                0xaa22
            }
            0xaa40 | 0xaaaf | 0xaab4 | 0xadfc => {
                w.cpu.y = w.dp_read(
                    if matches!(self.pc, 0xaa40 | 0xaab4) {
                        8
                    } else {
                        0x0e
                    },
                    2,
                );
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0xaa42 => {
                w.drain(16);
                w.cpu.x = 0x26f6;
                w.flags(w.cpu.x, 2);
                0xaa45
            }
            0xaa51 => {
                w.drain(6);
                w.push(w.cpu.y, 2);
                0xaa52
            }
            0xaa5b => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                0xaa5c
            }
            0xaa64 | 0xaa73 => {
                w.branch(true);
                if self.pc == 0xaa64 {
                    0xaa14
                } else {
                    0xaa80
                }
            }
            0xaa90 | 0xaa95 | 0xaa9d | 0xade2 | 0xadea => {
                let v = match self.pc {
                    0xaa90 => 0x1e,
                    0xaa95 => 0x36,
                    0xaa9d => 0x38,
                    0xade2 => 0x48,
                    _ => 0x4a,
                };
                w.drain(16);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= v);
                w.flags(w.cpu.y.wrapping_sub(v), 2);
                self.pc + 3
            }
            0xaaa2 | 0xaab9 | 0xadef | 0xae07 | 0xb1a8 => {
                w.dp_store(0x0e, w.cpu.a, 2);
                self.pc + 2
            }
            0xaaa4 | 0xadf1 | 0xae36 => {
                let base: u16 = if self.pc == 0xae36 { 0x199e } else { 0x197e };
                w.operand_then_internal(2);
                let v = w.read(u32::from(base.wrapping_add(w.cpu.x)), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xaaa7 | 0xadf4 | 0xb007 => {
                w.mask(if self.pc == 0xb007 { 0x0f80 } else { 0xff00 }, 2);
                self.pc + 3
            }
            0xaaaa | 0xadf7 => {
                let v = w.dp_read(0x0e, 2);
                w.load(w.cpu.a | v, 2);
                self.pc + 2
            }
            0xaab1 | 0xadfe => {
                w.operand_then_internal(2);
                let v = w.read(u32::from(0xce06u16.wrapping_add(w.cpu.y)), 2);
                if self.pc == 0xaab1 {
                    w.cpu.x = v;
                    w.flags(v, 2);
                } else {
                    w.load(v, 2);
                }
                self.pc + 3
            }
            0xaabb | 0xaac0 | 0xaac5 | 0xae09 | 0xae10 | 0xae17 => {
                let base: u16 = match self.pc {
                    0xaabb | 0xae09 => 0x9b52,
                    0xaac0 | 0xae10 => 0x9b54,
                    _ => 0x9b56,
                };
                let index = if matches!(self.pc, 0xaabb | 0xaac0 | 0xaac5) {
                    w.cpu.x
                } else {
                    w.cpu.y
                };
                w.operand_then_internal(2);
                let v = w.read(u32::from(base.wrapping_add(index)), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xaabe | 0xaac3 | 0xaac8 => {
                let a = w.indirect_y_address(match self.pc {
                    0xaabe => 0xcb,
                    0xaac3 => 0xd7,
                    _ => 0xda,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0xaaca | 0xb001 | 0xb1d6 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0xaad0 | 0xaad1 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xaad2 | 0xae26 | 0xb1dc => {
                w.dp_increment(0x0e, 2, false);
                self.pc + 2
            }
            0xae02 => {
                w.cpu.x = w.dp_read(8, 2);
                w.flags(w.cpu.x, 2);
                0xae04
            }
            0xa81c | 0xa921 | 0xac70 | 0xad5a => {
                let base: u16 = match self.pc {
                    0xa81c => 0x997e,
                    0xac70 => 0x99de,
                    _ => 0x998a,
                };
                w.operand_then_internal(2);
                w.cpu.y = w.read(u32::from(base.wrapping_add(w.cpu.x)), 2);
                w.flags(w.cpu.y, 2);
                self.pc + 3
            }
            0xa81f | 0xa8c3 | 0xa932 | 0xac3b | 0xac73 | 0xad65 => {
                w.dp_store(8, w.cpu.y, 2);
                self.pc + 2
            }
            0xa821 | 0xa829 | 0xa836 | 0xa83e | 0xa846 | 0xa853 | 0xa862 | 0xa874 | 0xa87e
            | 0xa883 | 0xa888 | 0xa88d | 0xa8be | 0xa907 | 0xad4a | 0xadbd => {
                w.compare_immediate(
                    match self.pc {
                        0xa821 => 0x30,
                        0xa829 => 0x16,
                        0xa836 => 0x32,
                        0xa83e => 6,
                        0xa846 => 0x14,
                        0xa853 => 2,
                        0xa862 => 0x12,
                        0xa874 => 8,
                        0xa87e => 0x20,
                        0xa883 => 0x22,
                        0xa888 | 0xa8be => 0x24,
                        0xa88d => 0x26,
                        0xa907 => 0x40,
                        _ => 0x46,
                    },
                    2,
                );
                self.pc + 3
            }
            0xa824 | 0xa82c | 0xa839 | 0xa841 | 0xa849 | 0xa856 | 0xa865 | 0xa877 | 0xa881
            | 0xa886 | 0xa88b | 0xa890 | 0xa8b4 | 0xa8c1 | 0xa8f8 | 0xa90a | 0xa914 | 0xa93d
            | 0xa945 | 0xa94d | 0xa981 | 0xac46 | 0xac95 | 0xac9f | 0xaca3 | 0xaca7 | 0xad1f
            | 0xad3e | 0xad46 | 0xad4d | 0xad76 | 0xad7e | 0xadb9 | 0xadc0 | 0xafc5 => {
                let taken = match self.pc {
                    0xa8c1 | 0xa90a | 0xa914 | 0xa93d | 0xac46 | 0xad46 => w.cpu.p & CARRY == 0,
                    0xa881 | 0xa886 | 0xa88b | 0xa8b4 | 0xa945 | 0xac95 | 0xac9f | 0xaca3
                    | 0xaca7 | 0xad4d | 0xad76 | 0xadc0 => w.cpu.p & ZERO != 0,
                    _ => w.cpu.p & ZERO == 0,
                };
                let target = match self.pc {
                    0xa824 => 0xa829,
                    0xa82c => 0xa836,
                    0xa839 => 0xa83e,
                    0xa841 => 0xa846,
                    0xa849 => 0xa853,
                    0xa856 => 0xa862,
                    0xa865 => 0xa874,
                    0xa877 => 0xa87e,
                    0xa881 | 0xa886 | 0xa88b => 0xa892,
                    0xa890 => 0xa907,
                    0xa8b4 => 0xa8bc,
                    0xa8c1 | 0xa914 => 0xa932,
                    0xa8f8 => 0xa8d9,
                    0xa90a => 0xa90f,
                    0xa93d => 0xa983,
                    0xa945 => 0xa94f,
                    0xa94d => 0xa95e,
                    0xa981 => 0xa968,
                    0xac46 => 0xac4d,
                    0xac95 => 0xac5b,
                    0xac9f | 0xaca3 | 0xaca7 => 0xacaa,
                    0xad1f => 0xad03,
                    0xad3e => 0xad2a,
                    0xad46 | 0xad4d => 0xad65,
                    0xad76 => 0xad80,
                    0xad7e => 0xad8f,
                    0xadb9 => 0xad9a,
                    0xadc0 => 0xadc7,
                    _ => 0xaf91,
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0xa826 | 0xa833 | 0xa83b | 0xa843 | 0xa850 | 0xa85f | 0xa90c => {
                w.drain(16);
                match self.pc {
                    0xa826 => 0xac70,
                    0xa833 => 0xb09f,
                    0xa83b => 0xac3b,
                    0xa843 => 0xaf7f,
                    0xa850 => 0xb092,
                    0xa85f => 0xa90f,
                    _ => 0xad41,
                }
            }
            0xa82e | 0xa84b | 0xa858 | 0xa86a | 0xa89b | 0xa8ee | 0xad36 | 0xadaf => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0xa82f | 0xa84c => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0xa830 | 0xa84d => {
                w.drain(16);
                w.sbc(0x00fe, 2);
                self.pc + 3
            }
            0xa859 | 0xa8a8 | 0xa954 | 0xac8b | 0xad85 | 0xaf8d => {
                w.mask(
                    match self.pc {
                        0xa859 | 0xaf8d => 0xf07f,
                        0xa8a8 | 0xac8b => 0xf,
                        _ => 0xff00,
                    },
                    2,
                );
                self.pc + 3
            }
            0xa867 | 0xa892 | 0xa8c5 | 0xa934 | 0xac3d | 0xac5b | 0xac75 | 0xad67 | 0xadc7 => {
                w.drain(16);
                w.cpu.x = w.read(if self.pc == 0xa867 { 0x19e0 } else { 0x0460 }, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0xa86b | 0xa898 | 0xa89c | 0xa8a4 | 0xa959 | 0xac67 | 0xac7e | 0xac87 | 0xad8a
            | 0xadd0 => {
                let base: u16 = match self.pc {
                    0xa86b => 0x19e2,
                    0xa898 => 0x19c0,
                    0xa89c | 0xac7e => 0x19a0,
                    0xa8a4 | 0xac67 | 0xac87 => 0x1980,
                    0xadd0 => 0x199e,
                    _ => 0x197e,
                };
                w.operand_then_internal(2);
                w.write(u32::from(base.wrapping_add(w.cpu.x)), w.cpu.a, 2);
                self.pc + 3
            }
            0xa86e | 0xa86f | 0xa8b6 | 0xa8b7 | 0xa8f4 | 0xa8f5 | 0xac6a | 0xac6b | 0xad1b
            | 0xad1c | 0xad23 | 0xad24 | 0xad34 | 0xad35 | 0xadb5 | 0xadb6 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xa870 | 0xa8b8 | 0xac6c => {
                w.drain(16);
                w.write(if self.pc == 0xa870 { 0x19e0 } else { 0x0460 }, w.cpu.x, 2);
                self.pc + 3
            }
            0xa87c | 0xac4b => {
                w.branch(true);
                if self.pc == 0xa87c {
                    0xa8fa
                } else {
                    0xac55
                }
            }
            0xa895 | 0xa8c8 | 0xa8d4 | 0xa937 | 0xa93f | 0xa947 | 0xa963 | 0xac40 | 0xace4
            | 0xad25 | 0xad6a | 0xad70 | 0xad78 | 0xad95 | 0xaf7f => {
                w.immediate(
                    match self.pc {
                        0xa8d4 | 0xa963 | 0xad95 => 4,
                        0xa93f => 0x18,
                        0xace4 => 0x12,
                        0xad25 => 6,
                        0xad70 => 0x44,
                        0xad78 => 0x40,
                        _ => 0,
                    },
                    2,
                );
                self.pc + 3
            }
            0xa89f | 0xa8a7 | 0xa977 | 0xac61 | 0xac81 | 0xac8a | 0xacf6 | 0xafbd => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0xa8a0 | 0xac62 | 0xac82 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a as u8 & CARRY);
                w.load(w.cpu.a >> 1, 2);
                self.pc + 1
            }
            0xa8a1 | 0xac63 | 0xac83 => {
                w.swap_bytes();
                self.pc + 1
            }
            0xa8a2 | 0xa957 | 0xad88 => {
                let v = w.dp_read(if self.pc == 0xa8a2 { 4 } else { 0x0e }, 2);
                w.load(w.cpu.a | v, 2);
                self.pc + 2
            }
            0xa8ab | 0xa8d1 | 0xa8f3 | 0xac8e | 0xad3b | 0xad92 | 0xadb4 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xa8ac | 0xa8ce | 0xad8f => {
                w.operand_then_internal(2);
                let base: u16 = if self.pc == 0xa8ac { 0x98c0 } else { 0xcd9e };
                let v = w.read(u32::from(base.wrapping_add(w.cpu.y)), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xa8af | 0xa95c | 0xa961 | 0xac4d | 0xac55 | 0xace9 | 0xad8d => {
                w.cpu.y = w.dp_read(
                    if matches!(self.pc, 0xa95c | 0xad8d) {
                        0x0e
                    } else {
                        8
                    },
                    2,
                );
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0xa8b1 => {
                w.drain(16);
                let v = w.read(0x068c, 2);
                w.load(w.cpu.a & v, 2);
                0xa8b4
            }
            0xa8bc | 0xa924 | 0xa92e | 0xac78 | 0xacae | 0xacb1 | 0xacd7 | 0xacee | 0xad48
            | 0xadbb | 0xadc2 | 0xaf85 => {
                let a = match self.pc {
                    0xa8bc | 0xa924 | 0xa92e | 0xad48 | 0xadbb => 4,
                    0xacb1 => 0xa7,
                    0xacd7 => 0xfc,
                    _ => 8,
                };
                let v = w.dp_read(a, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xa8d2 | 0xacfe | 0xad93 | 0xa90f | 0xad41 => {
                w.cpu.x = w.dp_read(
                    if matches!(self.pc, 0xa90f | 0xad41) {
                        2
                    } else {
                        8
                    },
                    2,
                );
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0xa8d7 | 0xa930 | 0xa94f | 0xa966 | 0xacb6 | 0xaccf | 0xacdc | 0xace7 | 0xacf4
            | 0xad28 | 0xad80 | 0xad98 | 0xaf8b => {
                let a = match self.pc {
                    0xa930 | 0xad28 => 0x0a,
                    0xacb6 => 0xa7,
                    0xaccf | 0xacf4 => 8,
                    0xacdc => 0xfc,
                    0xace7 => 0xb2,
                    0xaf8b => 2,
                    _ => 0x0e,
                };
                w.dp_store(a, w.cpu.a, 2);
                self.pc + 2
            }
            0xa8d9 | 0xa8e0 | 0xa8e7 | 0xa968 | 0xa96d | 0xa972 | 0xad00 | 0xad2a | 0xad2f
            | 0xad9a | 0xada1 | 0xada8 => {
                let base: u16 = match self.pc {
                    0xa8e0 | 0xa96d | 0xada1 => 0x9b54,
                    0xa8e7 | 0xa972 | 0xada8 => 0x9b56,
                    0xad2f => 0x9b5e,
                    _ => 0x9b52,
                };
                let index = if matches!(self.pc, 0xa968 | 0xa96d | 0xa972 | 0xad2a | 0xad2f) {
                    w.cpu.x
                } else {
                    w.cpu.y
                };
                w.operand_then_internal(2);
                let v = w.read(u32::from(base.wrapping_add(index)), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xa8dc | 0xa8e3 | 0xa8ea | 0xad03 | 0xad07 | 0xad0b | 0xad0f | 0xad13 | 0xad17
            | 0xad9d | 0xada4 | 0xadab | 0xaf98 | 0xafa3 | 0xafae | 0xafb9 => {
                let base = match self.pc {
                    0xa8dc => 0x7e4000,
                    0xa8e3 | 0xada4 => 0x7e4080,
                    0xa8ea | 0xadab => 0x7e4100,
                    0xad03 | 0xad9d | 0xaf98 => 0x7e2000,
                    0xad07 => 0x7e2080,
                    0xad0b => 0x7e2100,
                    0xad0f => 0x7e2180,
                    0xad13 => 0x7e2200,
                    0xad17 => 0x7e2280,
                    0xafa3 => 0x7e2002,
                    0xafae => 0x7e2004,
                    _ => 0x7e2006,
                };
                w.drain(24);
                w.write(base + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0xa8ef | 0xa978 | 0xac7a | 0xaccb | 0xacf0 | 0xacf7 | 0xad37 | 0xadb0 | 0xaf87
            | 0xafbe => {
                w.clear_carry();
                self.pc + 1
            }
            0xa8f0 | 0xa979 | 0xac7b | 0xaccc | 0xacf1 | 0xacf8 | 0xad38 | 0xadb1 | 0xaf88
            | 0xafbf => {
                w.drain(16);
                w.adc(match self.pc {
                    0xa8f0 | 0xa979 | 0xadb1 => 6,
                    0xac7b => 0x14,
                    0xaccc => 0x0300,
                    0xacf1 => 4,
                    0xacf8 => 12,
                    _ => 0x0080,
                });
                self.pc + 3
            }
            0xa8f6 | 0xa97f | 0xad1d | 0xad3c | 0xadb7 => {
                w.dp_increment(
                    match self.pc {
                        0xad1d => 0xb2,
                        0xad3c => 0x0a,
                        _ => 0x0e,
                    },
                    2,
                    false,
                );
                self.pc + 2
            }
            0xa911 | 0xad43 => {
                w.compare_x_immediate(12, 2);
                self.pc + 3
            }
            0xa916 | 0xa91a | 0xacb0 | 0xacfc | 0xad4f | 0xad53 => {
                w.drain(6);
                w.push(
                    match self.pc {
                        0xa916 | 0xad4f => w.cpu.y,
                        0xacfc => w.cpu.x,
                        _ => w.cpu.a,
                    },
                    2,
                );
                self.pc + 1
            }
            0xa917 | 0xac8f | 0xacb8 | 0xad50 => {
                w.drain(16);
                let v = w.read(
                    match self.pc {
                        0xac8f => 0x068c,
                        0xacb8 => 0x0452,
                        _ => 0x0460,
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0xa91b | 0xac64 | 0xac84 | 0xacb3 | 0xacbb | 0xacd9 | 0xad54 | 0xadcd | 0xaf95
            | 0xafa0 | 0xafab | 0xafb6 => {
                w.drain(16);
                w.load(
                    w.cpu.a
                        | match self.pc {
                            0xa91b | 0xad54 => 0x10,
                            0xac64 | 0xac84 => 0x30,
                            0xacb3 => 2,
                            0xacbb => 0x0100,
                            0xacd9 => 0x0200,
                            _ => 0x2000,
                        },
                    2,
                );
                self.pc + 3
            }
            0xa91e | 0xa92a | 0xacbe | 0xad57 | 0xad61 => {
                w.drain(16);
                w.write(if self.pc == 0xacbe { 0x0452 } else { 0x0460 }, w.cpu.a, 2);
                self.pc + 3
            }
            0xa929 | 0xacca | 0xad60 => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                self.pc + 1
            }
            0xa92d | 0xad64 => {
                w.drain(12);
                w.cpu.y = w.pull(2);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xa942 | 0xa94a | 0xad73 | 0xad7b => {
                let v = match self.pc {
                    0xa942 => 0x36,
                    0xa94a => 0x38,
                    0xad73 => 0x48,
                    _ => 0x4a,
                };
                w.drain(16);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= v);
                w.flags(w.cpu.y.wrapping_sub(v), 2);
                self.pc + 3
            }
            0xa951 | 0xad82 | 0xadca => {
                w.operand_then_internal(2);
                let base: u16 = if self.pc == 0xadca { 0x199e } else { 0x197e };
                let v = w.read(u32::from(base.wrapping_add(w.cpu.x)), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xa95e | 0xac48 | 0xacc4 | 0xace1 => {
                w.operand_then_internal(2);
                let base: u16 = if self.pc == 0xacc4 { 0xce06 } else { 0xcd9e };
                w.cpu.x = w.read(u32::from(base.wrapping_add(w.cpu.y)), 2);
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0xa96b | 0xa970 | 0xa975 | 0xad2d | 0xad32 => {
                let a = w.indirect_y_address(match self.pc {
                    0xa96b | 0xad2d => 0xbf,
                    0xa970 => 0xcb,
                    0xa975 => 0xd7,
                    _ => 0xc2,
                });
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0xa97c | 0xacfb | 0xaf90 | 0xafc2 => {
                w.drain(6);
                w.cpu.x = w.cpu.a;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xa97d | 0xa97e => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xac4f => {
                w.drain(16);
                w.cpu.x = 0x078a;
                w.flags(w.cpu.x, 2);
                0xac52
            }
            0xac5e => {
                w.operand_then_internal(2);
                w.write(u32::from(0x19c0u16.wrapping_add(w.cpu.x)), 0, 2);
                0xac61
            }
            0xac92 => {
                w.operand_then_internal(2);
                let v = w.read(u32::from(0x98c0u16.wrapping_add(w.cpu.y)), 2);
                w.load(w.cpu.a & v, 2);
                0xac95
            }
            // Only the tag selection uses byte registers. REP restores both widths
            // before any saved tile address or nested drawing frame is consumed.
            0xac97 | 0xacac => {
                w.width(0x30, self.pc == 0xacac);
                self.pc + 2
            }
            0xac99 => {
                w.drain(8);
                w.cpu.x = 0;
                w.flags(0, 1);
                0xac9b
            }
            0xac9b => {
                let v = w.dp_read(0xae, 1);
                w.load(v, 1);
                0xac9d
            }
            0xac9d | 0xaca1 | 0xaca5 => {
                w.compare_immediate(
                    match self.pc {
                        0xac9d => 0x20,
                        0xaca1 => 0x25,
                        _ => 0x28,
                    },
                    1,
                );
                self.pc + 2
            }
            0xaca9 => {
                w.drain(6);
                w.cpu.x = (w.cpu.x + 1) & 255;
                w.flags(w.cpu.x, 1);
                0xacaa
            }
            0xacaa => {
                w.drain(6);
                w.dp_store(0xae + w.cpu.x, 0, 1);
                0xacac
            }
            0xacc1 | 0xacde => {
                w.drain(16);
                w.cpu.y = 0x54;
                w.flags(w.cpu.y, 2);
                self.pc + 3
            }
            0xacd1 | 0xacd4 => {
                w.drain(16);
                let v = w.read(0x0460, 2).wrapping_add(1);
                w.drain(6);
                w.write(0x0460, v, 2);
                w.flags(v, 2);
                self.pc + 3
            }
            0xacfd | 0xad21 => {
                w.drain(6);
                w.cpu.y = w.cpu.x;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xad22 => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                0xad23
            }
            0xaf91 | 0xaf9c | 0xafa7 | 0xafb2 => {
                let base = match self.pc {
                    0xaf91 => 0x7e2000,
                    0xaf9c => 0x7e2002,
                    0xafa7 => 0x7e2004,
                    _ => 0x7e2006,
                };
                w.drain(24);
                let v = w.read(base + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                self.pc + 4
            }
            0xafc3 => {
                let v = w.dp_read(2, 2);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.x >= v);
                w.flags(w.cpu.x.wrapping_sub(v), 2);
                0xafc5
            }
            _ => unreachable!("door drawing continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
