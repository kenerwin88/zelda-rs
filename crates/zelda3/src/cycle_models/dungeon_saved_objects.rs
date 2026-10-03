//! Saved chest/tag decisions and persistent room object records with live returns.
use super::{
    dungeon_objects::DungeonObjectTiles,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonSavedObjects {
    pc: u16,
    depth: u8,
    tiles: Option<DungeonObjectTiles>,
    tail: bool,
    complete: bool,
}
impl DungeonSavedObjects {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(
            entry,
            0x9892
                | 0x98ae
                | 0x98d0
                | 0x99b8
                | 0x99bb
                | 0x9a00
                | 0x9a6f
                | 0xb395
                | 0xb493
                | 0xb376
                | 0xb381
                | 0xb474
                | 0xb47f
                | 0xb306
                | 0xb30b
                | 0xb310
                | 0xb3e1
                | 0xb4d6
                | 0xb509
        ));
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
            0x9892 | 0xb399 | 0xb39c | 0xb497 | 0xb49a | 0xb341 | 0xb344 | 0xb430 | 0xb433 => {
                w.drain(16);
                let a = if self.pc == 0x9892 { 0x045a } else { 0x042c };
                let v = w.read(a, 2).wrapping_add(1);
                w.drain(6);
                w.write(a, v, 2);
                w.flags(v, 2);
                self.pc + 3
            }
            0x98ae | 0x98da | 0x99bb | 0x9a70 | 0xb396 | 0xb494 | 0xb33b | 0xb42a => {
                w.drain(16);
                w.cpu.x = w.read(
                    match self.pc {
                        0x98ae => 0x0498,
                        0x98da | 0x99bb => 0x0496,
                        0x9a70 => 0x0432,
                        _ => 0x042c,
                    },
                    2,
                );
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x98b1 | 0x98dd | 0x98e8 | 0x994f | 0x99ae | 0x99be | 0x99cc | 0x9a73 | 0x9a7f
            | 0xb3aa | 0xb3b5 | 0xb4a8 | 0xb4b3 | 0xb34c | 0xb357 | 0xb43b | 0xb446 | 0xb468 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x98b2 | 0x98de | 0x98ec | 0x99c2 | 0x99d0 | 0x9a75 | 0x9a84 | 0xb3a2 | 0xb3a7
            | 0xb3ab | 0xb3b9 | 0xb3bf | 0xb3c5 | 0xb3cb | 0xb3d1 | 0xb4a0 | 0xb4a5 | 0xb4a9
            | 0xb4b7 | 0xb4bd | 0xb4c3 | 0xb4c9 | 0xb4cf | 0xb33e | 0xb349 | 0xb34d | 0xb35b
            | 0xb360 | 0xb365 | 0xb36a | 0xb36f | 0xb42d | 0xb438 | 0xb43c | 0xb44a | 0xb453
            | 0xb459 | 0xb45f | 0xb465 => {
                w.operand_then_internal(2);
                let base: u16 = match self.pc {
                    0x9a75 | 0x9a84 => 0x06a0,
                    0xb3a2 | 0xb4a0 | 0xb33e | 0xb42d => 0x0500,
                    0xb3a7 | 0xb4a5 | 0xb349 | 0xb438 => 0x0520,
                    0xb3ab | 0xb3b9 | 0xb4a9 | 0xb4b7 | 0xb34d | 0xb35b | 0xb43c | 0xb44a => 0x0540,
                    0xb3bf | 0xb4bd | 0xb360 | 0xb453 => 0x0560,
                    0xb3c5 | 0xb4c3 | 0xb365 | 0xb459 => 0x0580,
                    0xb3cb | 0xb4c9 | 0xb36a | 0xb45f => 0x05a0,
                    0xb3d1 | 0xb4cf | 0xb36f | 0xb465 => 0x05c0,
                    _ => 0x06e0,
                };
                w.write(u32::from(base.wrapping_add(w.cpu.x)), w.cpu.a, 2);
                self.pc + 3
            }
            0x98b5 | 0x98ef | 0x9946 | 0x99d3 | 0xb3e8 => {
                w.drain(16);
                let v = w.read(0x0402, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x98b8 | 0x98f2 | 0x99d6 | 0x9949 => {
                if self.pc == 0x9949 { w.drain(24); } else { w.operand_then_internal(2); }
                let v = w.read(0x009900 + u32::from(w.cpu.x), 2);
                w.load(w.cpu.a & v, 2);
                self.pc + if self.pc == 0x9949 { 4 } else { 3 }
            }
            0x98bb | 0x98d8 | 0x98e6 | 0x98f5 | 0x990d | 0x9912 | 0x9917 | 0x991c | 0x9921
            | 0x992b | 0x9930 | 0x9935 | 0x993a | 0x993f | 0x994d | 0x9975 | 0x997a | 0x997f
            | 0x9984 | 0x9989 | 0x9993 | 0x9998 | 0x999d | 0x99a2 | 0x99a7 | 0x99ca | 0x99d9
            | 0x9a7d | 0xb3b3 | 0xb3d9 | 0xb4b1 | 0xb355 | 0xb444 | 0xb3e6 | 0xb3ee => {
                let (take, target) = match self.pc {
                    0x98bb => (w.cpu.p & ZERO == 0, 0x98c7),
                    0x98d8 | 0x994d => (w.cpu.p & ZERO != 0, 0x98cf),
                    0x98e6 => (w.cpu.p & ZERO == 0, 0x98ef),
                    0x98f5 => (w.cpu.p & ZERO == 0, 0x995c),
                    0x990d | 0x9912 | 0x9917 => (w.cpu.p & ZERO != 0, 0x9946),
                    0x991c => (w.cpu.p & CARRY == 0, 0x9923),
                    0x9921 => (w.cpu.p & CARRY == 0, 0x9946),
                    0x992b | 0x9930 | 0x9935 => (w.cpu.p & ZERO != 0, 0x9941),
                    0x993a => (w.cpu.p & CARRY == 0, 0x9954),
                    0x993f => (w.cpu.p & CARRY != 0, 0x9954),
                    0x9975 | 0x997a | 0x997f => (w.cpu.p & ZERO != 0, 0x99ae),
                    0x9984 => (w.cpu.p & CARRY == 0, 0x998b),
                    0x9989 => (w.cpu.p & CARRY == 0, 0x99ae),
                    0x9993 | 0x9998 | 0x999d => (w.cpu.p & ZERO != 0, 0x99a9),
                    0x99a2 => (w.cpu.p & CARRY == 0, 0x99b3),
                    0x99a7 => (w.cpu.p & CARRY != 0, 0x99b3),
                    0x99ca => (w.cpu.p & ZERO == 0, 0x99d3),
                    0x99d9 => (w.cpu.p & ZERO == 0, 0x99f2),
                    0x9a7d => (w.cpu.p & ZERO == 0, 0x9a87),
                    0xb3b3 => (w.cpu.p & ZERO == 0, 0xb3bc),
                    0xb3d9 => (w.cpu.p & ZERO != 0, 0xb3de),
                    0xb355 => (w.cpu.p & ZERO == 0, 0xb35e),
                    0xb444 => (w.cpu.p & ZERO == 0, 0xb44d),
                    0xb3e6 => (w.cpu.p & ZERO == 0, 0xb3fa),
                    0xb3ee => (w.cpu.p & ZERO != 0, 0xb3fa),
                    _ => (w.cpu.p & ZERO == 0, 0xb4ba),
                };
                if w.branch(take) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x98bd | 0x98be | 0x98ca | 0x98cb | 0x98f7 | 0x98f8 | 0x9944 | 0x9945 | 0x995f
            | 0x9960 | 0x99ac | 0x99ad | 0x99db | 0x99dc | 0x99f5 | 0x99f6 | 0x9a87 | 0x9a88 => {
                w.increment_x();
                self.pc + 1
            }
            0x98bf | 0x98cc | 0x98f9 | 0x98fc | 0x9961 | 0x9964 | 0x99dd | 0x99e0 | 0x99f7
            | 0x99fa | 0x9a89 => {
                w.drain(16);
                w.write(
                    match self.pc {
                        0x98f9 | 0x9961 | 0x99dd | 0x99f7 => 0x0496,
                        0x9a89 => 0x0432,
                        _ => 0x0498,
                    },
                    w.cpu.x,
                    2,
                );
                self.pc + 3
            }
            0x98c2 | 0x9902 | 0x9956 | 0x996a | 0x99b5 | 0x99e3 | 0x99fd | 0xb3db | 0xb312
            | 0xb31b | 0xb32b | 0xb334 | 0xb3f4 | 0xb401 | 0xb40a | 0xb41a | 0xb423 => {
                w.drain(16);
                w.cpu.x = match self.pc {
                    0x98c2 => 0x1494,
                    0x9902 | 0x996a => 0,
                    0x9956 => 0x149c,
                    0x99b5 => 0x14a4,
                    0x99e3 => 0x14ac,
                    0x99fd => 0x14c4,
                    0xb312 => 0x0e62,
                    0xb31b => 0x0e6a,
                    0xb32b => 0x0e72,
                    0xb334 => 0x0e7a,
                    0xb3f4 => 0x05aa,
                    0xb401 => 0x0220,
                    0xb40a => 0x0228,
                    0xb41a => 0x0230,
                    0xb423 => 0x0238,
                    _ => 0x0e92,
                };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x98c5 => {
                w.branch(true);
                0x9895
            }
            0x98c7 | 0x995c | 0x99f2 => {
                w.operand_then_internal(2);
                w.write(u32::from(0x06e0u16.wrapping_add(w.cpu.x)), 0, 2);
                self.pc + 3
            }
            0x98d0 | 0x98e1 | 0x9905 | 0x9923 | 0x996d | 0x998b | 0x99c5 | 0x9a78 | 0xb3a5
            | 0xb3ae | 0xb4a3 | 0xb4ac | 0xb347 | 0xb350 | 0xb436 | 0xb43f | 0xb3e1 => {
                let v = w.dp_read(
                    match self.pc {
                        0x98d0 => 0x10,
                        0x9905 | 0x996d => 0xae,
                        0x9923 | 0x998b => 0xaf,
                        0xb3a5 | 0xb4a3 | 0xb347 | 0xb436 => 0xba,
                        0xb3e1 => 0xa0,
                        _ => 0xbf,
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0x98d2 | 0x9907 | 0x9925 | 0x996f | 0x998d => {
                w.mask(0xff, 2);
                self.pc + 3
            }
            0x98d5 | 0x98e3 | 0x990a | 0x990f | 0x9914 | 0x9919 | 0x991e | 0x9928 | 0x992d
            | 0x9932 | 0x9937 | 0x993c | 0x9972 | 0x9977 | 0x997c | 0x9981 | 0x9986 | 0x9990
            | 0x9995 | 0x999a | 0x999f | 0x99a4 | 0x99c7 | 0x9a7a | 0xb3b0 | 0xb4ae | 0xb352
            | 0xb441 | 0xb3e3 => {
                w.compare_immediate(
                    match self.pc {
                        0x98d5 => 0x1a,
                        0xb3e3 => 0x65,
                        0x990a | 0x9928 | 0x9972 | 0x9990 => 0x27,
                        0x990f | 0x992d | 0x9977 | 0x9995 => 0x3c,
                        0x9914 | 0x9932 | 0x997c | 0x999a => 0x3e,
                        0x9919 | 0x9937 | 0x9981 | 0x999f => 0x29,
                        0x991e | 0x993c | 0x9986 | 0x99a4 => 0x33,
                        _ => 0x4000,
                    },
                    2,
                );
                self.pc + 3
            }
            0x98e9 | 0x99bf | 0x99cd | 0x9a80 | 0xb3b6 | 0xb4b4 | 0xb358 | 0xb447 => {
                w.drain(16);
                w.load(
                    w.cpu.a
                        | match self.pc {
                            0x99bf => 0x8000,
                            0x99cd => 0xa000,
                            _ => 0x2000,
                        },
                    2,
                );
                self.pc + 3
            }
            0x98ff | 0x9941 | 0x9967 | 0x99a9 => {
                w.drain(16);
                w.cpu.y = if matches!(self.pc, 0x98ff | 0x9967) {
                    0xff00
                } else {
                    0x00ff
                };
                w.flags(w.cpu.y, 2);
                self.pc + 3
            }
            0x9950 | 0x99af => {
                let v = w.dp_read(0xae, 2);
                w.load(w.cpu.a & v, 2);
                self.pc + 2
            }
            0x9952 | 0x99b1 => {
                w.dp_store(0xae, w.cpu.a, 2);
                self.pc + 2
            }
            0x9954 | 0x99b3 => {
                w.cpu.y = w.dp_read(8, 2);
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0x9959 | 0x99b8 | 0x99e9 | 0x9a03 | 0x9a8d | 0xb3de | 0xb4d3 | 0xb373 | 0xb471
            | 0xb3f7 => {
                w.drain(16);
                match self.pc {
                    0x99e9 | 0x9a03 => 0x8d80,
                    0x9a8d => 0xb2af,
                    0xb3f7 => 0x8ddc,
                    _ => 0x9895,
                }
            }
            0x99e6 | 0x9a00 | 0xb39f | 0xb3bc | 0xb3c2 | 0xb3c8 | 0xb3ce | 0xb49d | 0xb4ba
            | 0xb4c0 | 0xb4c6 | 0xb4cc | 0xb306 | 0xb30b | 0xb315 | 0xb31e | 0xb32e | 0xb337
            | 0xb3fc | 0xb404 | 0xb40d | 0xb41d | 0xb426 => {
                w.immediate(
                    match self.pc {
                        0xb39f => 0x1111,
                        0xb3bc => 0x0d0e,
                        0xb3c2 => 0x0d1e,
                        0xb3c8 => 0x4d0e,
                        0xb3ce => 0x4d1e,
                        0xb49d => 0x4040,
                        0xb4ba => 0x19d8,
                        0xb4c0 => 0x19d9,
                        0xb4c6 => 0x59d8,
                        0xb4cc => 0x59d9,
                        0xb306 => 0x1010,
                        0xb30b => 0x1212,
                        0xb315 => 0x2020,
                        0xb31e => 0x2121,
                        0xb32e => 0x2222,
                        0xb337 => 0x2323,
                        0xb3fc => 0x05ba,
                        0xb404 => 0x3030,
                        0xb40d => 0x3131,
                        0xb41d => 0x3232,
                        0xb426 => 0x3333,
                        _ => 4,
                    },
                    2,
                );
                self.pc + 3
            }
            0x9a6f | 0xb395 | 0xb493 | 0xb33a | 0xb429 => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                self.pc + 1
            }
            0x9a74 | 0x9a83 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a & 1) as u8;
                w.load(w.cpu.a >> 1, 2);
                self.pc + 1
            }
            0x9a8c | 0xb3d4 | 0xb4d2 | 0xb372 | 0xb470 => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xb3d5 => {
                w.drain(24);
                let v = w.read(0x7ef3ca, 2);
                w.load(v, 2);
                0xb3d9
            }
            0xb376 | 0xb381 | 0xb474 | 0xb47f => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.tiles = Some(DungeonObjectTiles::new(0x01b0ac).unwrap());
                self.tail = false;
                0xb0ac
            }
            0xb379 | 0xb384 | 0xb477 | 0xb482 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                if self.pc < 0xb400 {
                    0xb395
                } else {
                    0xb493
                }
            }
            0xb37c | 0xb390 | 0xb47a | 0xb48e => {
                w.dp_increment(0xb2, 2, false);
                self.pc + 2
            }
            0xb37e | 0xb392 | 0xb47c | 0xb490 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0xb37e => 0xb379,
                        0xb392 => 0xb384,
                        0xb47c => 0xb477,
                        _ => 0xb482,
                    }
                } else {
                    self.pc + 2
                }
            }
            0xb387 | 0xb485 | 0xb324 | 0xb413 => {
                let v = w.dp_read(8, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xb389 | 0xb487 | 0xb326 | 0xb415 | 0xb469 => {
                w.clear_carry();
                self.pc + 1
            }
            0xb38a | 0xb488 | 0xb327 | 0xb416 | 0xb46a => {
                w.drain(16);
                w.adc(if self.pc == 0xb46a { 8 } else { 0x100 });
                self.pc + 3
            }
            0xb38d | 0xb48b => {
                w.dp_store(8, w.cpu.a, 2);
                self.pc + 2
            }
            0xb38f | 0xb48d | 0xb32a | 0xb419 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x98cf | 0xb380 | 0xb394 | 0xb47e | 0xb492 => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            0xb309 | 0xb30e => {
                w.branch(true);
                0xb33a
            }
            0xb310 | 0xb3fa => {
                w.dp_store(8, w.cpu.y, 2);
                self.pc + 2
            }
            0xb318 | 0xb321 | 0xb331 | 0xb407 | 0xb410 | 0xb420 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                if self.pc < 0xb400 {
                    0xb33a
                } else {
                    0xb429
                }
            }
            // Pots and rock segments preserve the live tiles before drawing.
            0xb35e | 0xb363 | 0xb368 | 0xb36d => {
                let address = w.indirect_y_address(match self.pc {
                    0xb35e => 0xbf,
                    0xb363 => 0xcb,
                    0xb368 => 0xc2,
                    _ => 0xce,
                });
                let value = w.read(address, 2);
                w.load(value, 2);
                self.pc + 2
            }
            0xb3eb => {
                w.mask(0x1000, 2);
                self.pc + 3
            }
            0xb3f0 | 0xb3f2 => {
                w.dp_store(if self.pc == 0xb3f0 { 0xb2 } else { 0xb4 }, 0, 2);
                self.pc + 2
            }
            0xb3ff | 0xb46d => {
                w.dp_store(0x0e, w.cpu.a, 2);
                self.pc + 2
            }
            0xb44d => {
                w.drain(6);
                w.push(w.cpu.y, 2);
                self.pc + 1
            }
            0xb44e => {
                w.cpu.y = w.dp_read(0x0e, 2);
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0xb450 | 0xb456 | 0xb45c | 0xb462 => {
                w.operand_then_internal(2);
                let offset = u32::from((self.pc - 0xb450) / 6 * 2);
                let value = w.read(0x009b52 + u32::from(w.cpu.y) + offset, 2);
                w.load(value, 2);
                self.pc + 3
            }
            0xb46f => {
                w.drain(12);
                w.cpu.y = w.pull(2);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xb4d6 => {
                w.drain(16);
                w.cpu.x = w.read(0x042c, 2);
                w.flags(w.cpu.x, 2);
                0xb4d9
            }
            0xb4d9 | 0xb4dc | 0xb515 | 0xb518 | 0xb52e => {
                let a = match self.pc {
                    0xb4d9 | 0xb4dc => 0x042c,
                    0xb52e => 0x045a,
                    _ => 0x042e,
                };
                w.drain(16);
                let v = w.read(a, 2).wrapping_add(1);
                w.drain(6);
                w.write(a, v, 2);
                w.flags(v, 2);
                self.pc + 3
            }
            0xb4df => {
                w.operand_then_internal(2);
                w.write(u32::from(0x0500u16.wrapping_add(w.cpu.x)), 0, 2);
                0xb4e2
            }
            0xb4e2 | 0xb51e | 0xb533 => {
                let v = w.dp_read(if self.pc == 0xb4e2 { 0xba } else { 8 }, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xb4e4 | 0xb4e8 | 0xb4f1 | 0xb4f6 | 0xb4fb | 0xb500 => {
                let base: u16 = match self.pc {
                    0xb4e4 => 0x0520,
                    0xb4e8 => 0x0540,
                    0xb4f1 => 0x0560,
                    0xb4f6 => 0x0580,
                    0xb4fb => 0x05a0,
                    _ => 0x05c0,
                };
                w.operand_then_internal(2);
                w.write(u32::from(base.wrapping_add(w.cpu.x)), w.cpu.a, 2);
                self.pc + 3
            }
            0xb4e7 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                0xb4e8
            }
            0xb4eb | 0xb535 => {
                w.mask(0x3fff, 2);
                self.pc + 3
            }
            0xb4ee | 0xb538 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xb4ef | 0xb4f4 | 0xb4f9 | 0xb4fe => {
                let a = w.indirect_y_address(match self.pc {
                    0xb4ef => 0xbf,
                    0xb4f4 => 0xcb,
                    0xb4f9 => 0xc2,
                    _ => 0xce,
                });
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xb503 | 0xb51b | 0xb523 => {
                w.drain(16);
                w.cpu.x = match self.pc {
                    0xb503 => 0x0e52,
                    0xb51b => 0x0ec2,
                    _ => 0x0eca,
                };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0xb506 | 0xb539 => {
                w.drain(16);
                0x9895
            }
            0xb509 => {
                w.drain(16);
                w.cpu.y = w.read(0x042e, 2);
                w.flags(w.cpu.y, 2);
                0xb50c
            }
            0xb50c | 0xb512 => {
                let base: u16 = if self.pc == 0xb50c { 0x0540 } else { 0x0520 };
                w.operand_then_internal(2);
                w.write(u32::from(base.wrapping_add(w.cpu.y)), w.cpu.a, 2);
                self.pc + 3
            }
            0xb50f | 0xb510 => {
                w.decrement_x();
                self.pc + 1
            }
            0xb511 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                0xb512
            }
            0xb520 => {
                w.shift_left(2);
                0xb521
            }
            0xb521 | 0xb52c => {
                if w.branch((w.cpu.p & CARRY != 0) == (self.pc == 0xb52c)) {
                    0xb531
                } else {
                    self.pc + 2
                }
            }
            0xb526 => {
                w.drain(16);
                let v = w.read(0x045a, 2);
                w.load(v, 2);
                0xb529
            }
            0xb529 => {
                w.compare_immediate(3, 2);
                0xb52c
            }
            0xb531 => {
                w.dp_store(0x0c, w.cpu.x, 2);
                0xb533
            }
            _ => panic!("unowned saved-object continuation {:04x}", self.pc),
        };
        if matches!(self.pc, 0x9895 | 0x8d80 | 0xb2af | 0x8ddc) {
            self.tiles = Some(DungeonObjectTiles::new(0x010000 | u32::from(self.pc)).unwrap());
            self.tail = true;
        }
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
