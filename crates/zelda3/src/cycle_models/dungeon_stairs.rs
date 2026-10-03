//! Stair lists, layer decisions and live edge tiles with retained drawing returns.
use super::{
    dungeon_objects::DungeonObjectTiles,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonStairs {
    pc: u16,
    tiles: Option<DungeonObjectTiles>,
    tail: bool,
    complete: bool,
}
impl DungeonStairs {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(
            entry,
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
        ));
        Self {
            pc: entry,
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
                self.complete = self.tail;
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
            0xa25d | 0xa26d | 0xa2c7 | 0xa30c | 0xa31c | 0xa36e | 0xa380 | 0xa5d2 | 0xa5f4
            | 0xa607 | 0xa626 | 0xa664 | 0xa695 | 0xa71c | 0xa74a => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                self.pc + 1
            }
            0xa2db | 0xa36d | 0xa37c | 0xa3aa => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xa282 | 0xa32b | 0xa635 | 0xa6b6 | 0xa768 => {
                w.drain(12);
                w.cpu.y = w.pull(2);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x9b38 | 0xa25e | 0xa26e | 0xa2c8 | 0xa2f6 | 0xa30d | 0xa31d | 0xa36f | 0xa39d
            | 0xa3ae | 0xa41b | 0xa458 | 0xa486 | 0xa4b4 | 0xa4f5 | 0xa533 | 0xa584 | 0xa5d3
            | 0xa5f5 | 0xa608 | 0xa627 | 0xa665 | 0xa696 | 0xa71d | 0xa74b => {
                let address = match self.pc {
                    0x9b38 => 0x0444,
                    0xa25e => 0x043c,
                    0xa26e => 0x043e,
                    0xa2c8 => 0x0440,
                    0xa2f6 => 0x0442,
                    0xa30d => 0x049a,
                    0xa31d => 0x049c,
                    0xa36f => 0x049e,
                    0xa39d => 0x04ae,
                    0xa3ae => 0x0446,
                    0xa41b => 0x0438,
                    0xa458 | 0xa486 => 0x043a,
                    0xa4b4 => 0x047e,
                    0xa4f5 => 0x0482,
                    0xa533 => 0x0480,
                    0xa584 => 0x0484,
                    0xa5d3 | 0xa665 => 0x04a2,
                    0xa5f5 | 0xa696 => 0x04a6,
                    0xa608 | 0xa71d => 0x04a4,
                    _ => 0x04a8,
                };
                w.drain(16);
                w.cpu.x = w.read(address, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x9b3b | 0xa261 | 0xa271 | 0xa2b4 | 0xa2cb | 0xa2f9 | 0xa310 | 0xa320 | 0xa35d
            | 0xa372 | 0xa3a0 | 0xa3b1 | 0xa41e | 0xa42a | 0xa45b | 0xa467 | 0xa489 | 0xa495
            | 0xa4b7 | 0xa4c7 | 0xa4f8 | 0xa508 | 0xa536 | 0xa546 | 0xa587 | 0xa597 | 0xa5d6
            | 0xa5f8 | 0xa60b | 0xa62a | 0xa657 | 0xa668 | 0xa674 | 0xa699 | 0xa6a5 | 0xa6dc
            | 0xa720 | 0xa72c | 0xa74e | 0xa75a | 0xa78e => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x9b3c | 0xa262 | 0xa272 | 0xa2cc | 0xa2fa | 0xa311 | 0xa321 | 0xa373 | 0xa3a1
            | 0xa3b2 | 0xa41f | 0xa42e | 0xa45c | 0xa46b | 0xa48a | 0xa499 | 0xa4bc | 0xa4cf
            | 0xa4fd | 0xa510 | 0xa53b | 0xa54e | 0xa58c | 0xa59f | 0xa5d7 | 0xa5f9 | 0xa60c
            | 0xa62b | 0xa669 | 0xa678 | 0xa69a | 0xa6a9 | 0xa721 | 0xa730 | 0xa74f | 0xa75e => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a & 1) as u8;
                w.load(w.cpu.a >> 1, 2);
                self.pc + 1
            }
            0x9b3d | 0xa263 | 0xa273 | 0xa2cd | 0xa2fb | 0xa312 | 0xa322 | 0xa374 | 0xa3a2
            | 0xa3b3 | 0xa420 | 0xa42f | 0xa45d | 0xa46c | 0xa48b | 0xa49a | 0xa4bd | 0xa4d0
            | 0xa4fe | 0xa511 | 0xa53c | 0xa54f | 0xa58d | 0xa5a0 | 0xa5d8 | 0xa5fa | 0xa60d
            | 0xa62c | 0xa66a | 0xa679 | 0xa69b | 0xa6aa | 0xa722 | 0xa731 | 0xa750 | 0xa75f => {
                let base: u16 = match self.pc {
                    0xa322 | 0xa374 | 0xa3a2 => 0x06ec,
                    0x9b3d | 0xa263 | 0xa273 | 0xa2cd | 0xa2fb | 0xa312 | 0xa3b3 => 0x06b8,
                    _ => 0x06b0,
                };
                w.operand_then_internal(2);
                w.write(u32::from(base.wrapping_add(w.cpu.x)), w.cpu.a, 2);
                self.pc + 3
            }
            0x9b40 | 0x9b41 | 0xa266 | 0xa267 | 0xa276 | 0xa277 | 0xa2ba | 0xa2bb | 0xa2d0
            | 0xa2d1 | 0xa2fe | 0xa2ff | 0xa315 | 0xa316 | 0xa325 | 0xa326 | 0xa363 | 0xa364
            | 0xa377 | 0xa378 | 0xa3a5 | 0xa3a6 | 0xa3b6 | 0xa3b7 | 0xa432 | 0xa433 | 0xa46f
            | 0xa470 | 0xa49d | 0xa49e | 0xa4d3 | 0xa4d4 | 0xa514 | 0xa515 | 0xa552 | 0xa553
            | 0xa5a3 | 0xa5a4 | 0xa5db | 0xa5dc | 0xa5fd | 0xa5fe | 0xa610 | 0xa611 | 0xa62f
            | 0xa630 | 0xa65d | 0xa65e | 0xa67c | 0xa67d | 0xa6ad | 0xa6ae | 0xa6e2 | 0xa6e3
            | 0xa734 | 0xa735 | 0xa762 | 0xa763 | 0xa794 | 0xa795 => {
                w.increment_x();
                self.pc + 1
            }
            0x9b42 | 0xa268 | 0xa278 | 0xa27b | 0xa27e | 0xa2d2 | 0xa2d5 | 0xa2d8 | 0xa300
            | 0xa303 | 0xa317 | 0xa327 | 0xa379 | 0xa3a7 | 0xa3b8 | 0xa3bb | 0xa434 | 0xa437
            | 0xa43a | 0xa43d | 0xa440 | 0xa443 | 0xa446 | 0xa449 | 0xa44c | 0xa44f | 0xa471
            | 0xa474 | 0xa477 | 0xa47a | 0xa47d | 0xa49f | 0xa4a2 | 0xa4a5 | 0xa4a8 | 0xa4ab
            | 0xa4d5 | 0xa4d8 | 0xa4db | 0xa4de | 0xa4e1 | 0xa4e4 | 0xa4e7 | 0xa4ea | 0xa4ed
            | 0xa516 | 0xa519 | 0xa51c | 0xa51f | 0xa522 | 0xa525 | 0xa528 | 0xa52b | 0xa554
            | 0xa557 | 0xa55a | 0xa55d | 0xa5a5 | 0xa5a8 | 0xa5ab | 0xa5dd | 0xa5e0 | 0xa5e3
            | 0xa5e6 | 0xa5e9 | 0xa5ec | 0xa5ef | 0xa5ff | 0xa602 | 0xa612 | 0xa615 | 0xa618
            | 0xa61b | 0xa61e | 0xa621 | 0xa631 | 0xa67e | 0xa681 | 0xa684 | 0xa687 | 0xa68a
            | 0xa68d | 0xa690 | 0xa6af | 0xa6b2 | 0xa736 | 0xa739 | 0xa73c | 0xa73f | 0xa742
            | 0xa745 | 0xa764 => {
                let address = match self.pc {
                    0x9b42 | 0xa303 => 0x0444,
                    0xa268 => 0x043c,
                    0xa278 => 0x043e,
                    0xa27b | 0xa2d5 | 0xa3b8 => 0x0446,
                    0xa27e | 0xa2d8 | 0xa3bb => 0x0448,
                    0xa2d2 => 0x0440,
                    0xa300 => 0x0442,
                    0xa317 => 0x049a,
                    0xa327 => 0x049c,
                    0xa379 => 0x049e,
                    0xa3a7 => 0x04ae,
                    0xa434 => 0x0438,
                    0xa437 | 0xa4d5 => 0x047e,
                    0xa43a | 0xa4d8 | 0xa516 => 0x0482,
                    0xa43d | 0xa4db | 0xa519 | 0xa5dd | 0xa67e => 0x04a2,
                    0xa440 | 0xa4de | 0xa51c | 0xa5e0 | 0xa612 | 0xa681 | 0xa736 => 0x04a4,
                    0xa443 | 0xa471 | 0xa49f | 0xa4e1 | 0xa51f | 0xa5e3 | 0xa615 | 0xa684
                    | 0xa739 => 0x043a,
                    0xa446 | 0xa474 | 0xa4a2 | 0xa4e4 | 0xa522 | 0xa554 | 0xa5e6 | 0xa618
                    | 0xa687 | 0xa73c => 0x0480,
                    0xa449 | 0xa477 | 0xa4a5 | 0xa4e7 | 0xa525 | 0xa557 | 0xa5a5 | 0xa5e9
                    | 0xa61b | 0xa68a | 0xa73f => 0x0484,
                    0xa44c | 0xa47a | 0xa4a8 | 0xa4ea | 0xa528 | 0xa55a | 0xa5a8 | 0xa5ec
                    | 0xa5ff | 0xa61e | 0xa68d | 0xa6af | 0xa742 => 0x04a6,
                    _ => 0x04a8,
                };
                w.drain(16);
                w.write(address, w.cpu.x, 2);
                self.pc + 3
            }
            0xa281 | 0xa32a | 0xa3be | 0xa634 | 0xa6b5 | 0xa767 => {
                w.drain(6);
                w.cpu.x = w.cpu.y;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xa283 | 0xa32c | 0xa563 | 0xa5b1 | 0xa636 | 0xa6b7 | 0xa769 => {
                w.immediate(4, 2);
                self.pc + 3
            }
            0xa286 | 0xa32f | 0xa639 | 0xa6ba | 0xa76c | 0xa39b => {
                w.dp_store(if self.pc == 0xa39b { 0x99 } else { 0x0e }, w.cpu.a, 2);
                self.pc + 2
            }
            0xa288 | 0xa293 | 0xa29e | 0xa2a9 | 0xa331 | 0xa33c | 0xa347 | 0xa352 | 0xa3c2
            | 0xa3cd | 0xa3d8 | 0xa3e3 | 0xa3ee | 0xa3f9 | 0xa404 | 0xa40f | 0xa63b | 0xa642
            | 0xa649 | 0xa650 | 0xa6bc | 0xa6c7 | 0xa6ce | 0xa6d5 | 0xa76e | 0xa775 | 0xa77c
            | 0xa783 => {
                let offset = match self.pc {
                    0xa293 | 0xa33c | 0xa3cd | 0xa642 | 0xa6c7 | 0xa775 => 2,
                    0xa29e | 0xa347 | 0xa3d8 | 0xa649 | 0xa6ce | 0xa77c => 4,
                    0xa2a9 | 0xa352 | 0xa3e3 | 0xa650 | 0xa6d5 | 0xa783 => 6,
                    0xa3ee => 8,
                    0xa3f9 => 10,
                    0xa404 => 12,
                    0xa40f => 14,
                    _ => 0,
                };
                w.operand_then_internal(2);
                let value = w.read(0x009b52 + u32::from(w.cpu.y) + offset, 2);
                w.load(value, 2);
                self.pc + 3
            }
            0xa28b | 0xa28f | 0xa296 | 0xa29a | 0xa2a1 | 0xa2a5 | 0xa2ac | 0xa2b0 | 0xa334
            | 0xa338 | 0xa33f | 0xa343 | 0xa34a | 0xa34e | 0xa355 | 0xa359 | 0xa3c5 | 0xa3c9
            | 0xa3d0 | 0xa3d4 | 0xa3db | 0xa3df | 0xa3e6 | 0xa3ea | 0xa3f1 | 0xa3f5 | 0xa3fc
            | 0xa400 | 0xa407 | 0xa40b | 0xa412 | 0xa416 | 0xa574 | 0xa57f | 0xa5c2 | 0xa5cd
            | 0xa63e | 0xa645 | 0xa64c | 0xa653 | 0xa6bf | 0xa6c3 | 0xa6ca | 0xa6d1 | 0xa6d8
            | 0xa6f6 | 0xa701 | 0xa70c | 0xa717 | 0xa771 | 0xa778 | 0xa77f | 0xa786 | 0xa78a => {
                let address = match self.pc {
                    0xa28b | 0xa334 | 0xa3c5 | 0xa574 | 0xa63e | 0xa6bf | 0xa6f6 => 0x7e2000,
                    0xa28f | 0xa338 | 0xa3c9 | 0xa5c2 | 0xa6c3 | 0xa771 => 0x7e4000,
                    0xa296 | 0xa33f | 0xa3f1 | 0xa645 | 0xa701 => 0x7e2080,
                    0xa29a | 0xa343 | 0xa3f5 | 0xa6ca | 0xa778 => 0x7e4080,
                    0xa2a1 | 0xa34a | 0xa64c | 0xa70c => 0x7e2100,
                    0xa2a5 | 0xa34e | 0xa6d1 | 0xa77f => 0x7e4100,
                    0xa2ac | 0xa355 | 0xa653 | 0xa717 | 0xa786 => 0x7e2180,
                    0xa2b0 | 0xa359 | 0xa6d8 | 0xa78a => 0x7e4180,
                    0xa3d0 => 0x7e2002,
                    0xa3d4 => 0x7e4002,
                    0xa3db => 0x7e2004,
                    0xa3df => 0x7e4004,
                    0xa3e6 => 0x7e2006,
                    0xa3ea => 0x7e4006,
                    0xa3fc => 0x7e2082,
                    0xa400 => 0x7e4082,
                    0xa407 => 0x7e2084,
                    0xa40b => 0x7e4084,
                    0xa412 => 0x7e2086,
                    0xa416 => 0x7e4086,
                    0xa57f => 0x7e200a,
                    _ => 0x7e400a,
                };
                w.drain(24);
                w.write(address + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0xa2b5 | 0xa35e | 0xa658 | 0xa6dd | 0xa78f | 0xa79c => {
                w.clear_carry();
                self.pc + 1
            }
            0xa2b6 | 0xa35f | 0xa659 | 0xa6de | 0xa790 | 0xa79d => {
                w.drain(16);
                w.adc(if self.pc == 0xa79d { 0x200 } else { 8 });
                self.pc + 3
            }
            0xa2b9 | 0xa362 | 0xa65c | 0xa6e1 | 0xa793 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xa2bc | 0xa365 | 0xa65f | 0xa6e4 | 0xa796 => {
                w.dp_increment(0x0e, 2, false);
                self.pc + 2
            }
            0xa26b | 0xa31a | 0xa4f3 | 0xa531 | 0xa5f2 | 0xa605 | 0xa624 | 0xa693 | 0xa748 => {
                w.branch(true);
                match self.pc {
                    0xa26b => 0xa27b,
                    0xa31a => 0xa32a,
                    0xa4f3 => 0xa563,
                    0xa531 => 0xa5b1,
                    0xa693 => 0xa6b5,
                    0xa748 => 0xa767,
                    _ => 0xa634,
                }
            }
            0x9b26 | 0x9b33 | 0xa2be | 0xa2e7 | 0xa2f4 | 0xa367 | 0xa389 | 0xa396 | 0xa428
            | 0xa465 | 0xa493 | 0xa4c5 | 0xa506 | 0xa544 | 0xa595 | 0xa661 | 0xa672 | 0xa6a3
            | 0xa6e6 | 0xa72a | 0xa758 | 0xa798 => {
                let target = match self.pc {
                    0x9b26 | 0x9b33 => 0x9b38,
                    0xa2be => 0xa288,
                    0xa2e7 => 0xa2f6,
                    0xa2f4 => 0xa2c1,
                    0xa367 => 0xa331,
                    0xa389 => 0xa39d,
                    0xa396 => 0xa36a,
                    0xa428 => 0xa432,
                    0xa465 => 0xa46f,
                    0xa493 => 0xa49d,
                    0xa4c5 => 0xa4d3,
                    0xa506 => 0xa514,
                    0xa544 => 0xa552,
                    0xa595 => 0xa5a3,
                    0xa661 => 0xa63b,
                    0xa672 => 0xa67c,
                    0xa6a3 => 0xa6ad,
                    0xa6e6 => 0xa6bc,
                    0xa72a => 0xa734,
                    0xa758 => 0xa762,
                    _ => 0xa76e,
                };
                let take = if matches!(self.pc, 0xa2f4 | 0xa396) {
                    w.cpu.p & ZERO != 0
                } else {
                    w.cpu.p & ZERO == 0
                };
                if w.branch(take) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0xa2c1 | 0xa36a => {
                w.drain(16);
                w.write(0x0414, 0, 2);
                self.pc + 3
            }
            0x9b45 | 0xa2c4 | 0xa306 | 0xa452 | 0xa480 | 0xa4ae | 0xa4f0 | 0xa52e | 0xa560
            | 0xa5ae => {
                w.drain(16);
                w.cpu.x = match self.pc {
                    0x9b45 => 0x1108,
                    0xa2c4 | 0xa306 => 0x10c8,
                    0xa452 => 0x1088,
                    0xa480 | 0xa4ae => 0x10a8,
                    0xa4f0 => 0x1148,
                    0xa52e => 0x1178,
                    0xa560 => 0x1160,
                    _ => 0x1190,
                };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x9b35 | 0xa7a0 => {
                w.drain(16);
                if self.pc == 0x9b35 {
                    0xa3ae
                } else {
                    0xa6ee
                }
            }
            0xa2dc | 0xa309 | 0xa37d | 0xa3ab | 0xa455 | 0xa483 | 0xa4b1 => {
                w.drain(16);
                self.tiles = Some(DungeonObjectTiles::new(0x0197ed).unwrap());
                self.tail = true;
                0x97ed
            }
            0x9b1e | 0x9b28 | 0xa2df | 0xa2e9 | 0xa381 | 0xa38b | 0xa423 | 0xa460 | 0xa48e
            | 0xa4c0 | 0xa501 | 0xa53f | 0xa590 | 0xa66d | 0xa69e | 0xa6e8 | 0xa725 | 0xa753
            | 0xa79a => {
                let address = match self.pc {
                    0x9b1e | 0xa2df | 0xa381 => 0xaf,
                    0x9b28 | 0xa2e9 | 0xa38b => 0xa0,
                    0xa6e8 | 0xa79a => 8,
                    _ => 0xbf,
                };
                let value = w.dp_read(address, 2);
                w.load(value, 2);
                self.pc + 2
            }
            0x9b20 | 0x9b30 | 0xa2e1 | 0xa2f1 | 0xa383 | 0xa393 => {
                w.mask(
                    if matches!(self.pc, 0x9b20 | 0xa2e1 | 0xa383) {
                        0xff
                    } else {
                        0x100
                    },
                    2,
                );
                self.pc + 3
            }
            0x9b23 | 0xa2e4 | 0xa386 | 0xa425 | 0xa462 | 0xa490 | 0xa4c2 | 0xa503 | 0xa541
            | 0xa592 | 0xa66f | 0xa6a0 | 0xa727 | 0xa755 => {
                w.compare_immediate(
                    if matches!(self.pc, 0x9b23 | 0xa2e4 | 0xa386) {
                        0x1b
                    } else {
                        0x4000
                    },
                    2,
                );
                self.pc + 3
            }
            0x9b2a | 0xa2eb | 0xa38d => {
                w.shift_left(2);
                self.pc + 1
            }
            0x9b2b | 0xa2ec | 0xa38e | 0xa6ee => {
                w.tax_word();
                self.pc + 1
            }
            0x9b2c | 0xa2ed | 0xa38f => {
                w.drain(24);
                let value = w.read(0x7ef000 + u32::from(w.cpu.x), 2);
                w.load(value, 2);
                self.pc + 4
            }
            0xa398 => {
                w.immediate(0x6202, 2);
                self.pc + 3
            }
            0xa3bf => {
                w.drain(16);
                w.cpu.y = 0x1108;
                w.flags(w.cpu.y, 2);
                self.pc + 3
            }
            0xa42b | 0xa468 | 0xa496 | 0xa4cc | 0xa50d | 0xa54b | 0xa571 | 0xa57c | 0xa59c
            | 0xa5bf | 0xa5ca | 0xa675 | 0xa6a6 | 0xa6f3 | 0xa6fe | 0xa709 | 0xa714 | 0xa72d
            | 0xa75b => {
                w.drain(16);
                w.load(w.cpu.a | 0x2000, 2);
                self.pc + 3
            }
            0xa4b8 | 0xa4c8 | 0xa4f9 | 0xa509 | 0xa537 | 0xa547 | 0xa588 | 0xa598 | 0xa6ea => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0xa4b9 | 0xa4c9 | 0xa4fa | 0xa50a | 0xa538 | 0xa548 | 0xa589 | 0xa599 | 0xa6eb => {
                w.drain(16);
                w.sbc(if self.pc == 0xa6eb { 0x200 } else { 0x80 }, 2);
                self.pc + 3
            }
            0xa566 | 0xa5b4 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.tiles = Some(DungeonObjectTiles::new(0x018d80).unwrap());
                self.tail = false;
                0x8d80
            }
            0xa569 | 0xa5b7 => {
                w.cpu.x = w.dp_read(8, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0xa56b | 0xa56c | 0xa5b9 | 0xa5ba => {
                w.decrement_x();
                self.pc + 1
            }
            0xa56d | 0xa578 | 0xa5bb | 0xa5c6 | 0xa6ef | 0xa6fa | 0xa705 | 0xa710 => {
                let address = match self.pc {
                    0xa56d | 0xa6ef => 0x7e2000,
                    0xa578 => 0x7e200a,
                    0xa5bb => 0x7e4000,
                    0xa5c6 => 0x7e400a,
                    0xa6fa => 0x7e2080,
                    0xa705 => 0x7e2100,
                    _ => 0x7e2180,
                };
                w.drain(24);
                let value = w.read(address + u32::from(w.cpu.x), 2);
                w.load(value, 2);
                self.pc + 4
            }
            0xa2c0 | 0xa369 | 0xa41a | 0xa583 | 0xa5d1 | 0xa663 | 0xa71b => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => panic!("unowned stair continuation {:04x}", self.pc),
        };
        // Water-hop A falls through the already-owned portrait drawer on its outer frame.
        if self.pc == 0x9b48 {
            self.tiles = Some(DungeonObjectTiles::new(0x019b48).unwrap());
            self.tail = true;
        }
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
