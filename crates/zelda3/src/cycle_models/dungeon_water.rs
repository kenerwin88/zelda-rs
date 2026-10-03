//! Water-overlay geometry, saved-room state and tile drawing continuations.
use super::{
    dungeon_floors::DungeonFloors,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonWater {
    pc: u16,
    blocks: Option<DungeonFloors>,
    complete: bool,
}
impl DungeonWater {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(entry, 0x9501 | 0x95ef));
        Self {
            pc: entry,
            blocks: None,
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
        if let Some(blocks) = self.blocks.as_mut() {
            let cycles = blocks.step(cpu, clock, bus);
            self.pc = cpu.pc;
            if blocks.complete() {
                self.blocks = None;
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
            0x9501 | 0x9510 | 0x952f | 0x954b | 0x9588 | 0x958f | 0x9595 | 0x959b | 0x95dc
            | 0x95e1 | 0x95ef | 0x9602 | 0x9628 | 0x9644 | 0x9689 | 0x9697 | 0x96cf => {
                let a = match self.pc {
                    0x9510 | 0x954b | 0x9595 | 0x9602 | 0x9644 | 0x9689 => 0xb4,
                    0x958f | 0x959b | 0x95e1 | 0x96cf => 8,
                    _ => 0xb2,
                };
                let v = w.dp_read(a, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0x9503
            | 0x9512
            | 0x952a
            | 0x952b
            | 0x9531..=0x9534
            | 0x954d..=0x9550
            | 0x958b
            | 0x958c
            | 0x95f1
            | 0x9604
            | 0x9623
            | 0x9624
            | 0x962a..=0x962d
            | 0x9646..=0x9649
            | 0x968b => {
                w.shift_left(2);
                self.pc + 1
            }
            0x9504 | 0x9513 | 0x959f | 0x95cf | 0x95f2 | 0x9605 | 0x968c => {
                w.tax_word();
                self.pc + 1
            }
            0x9505 | 0x950a | 0x9514 | 0x9519 | 0x95f3 | 0x95f8 | 0x9606 | 0x960b | 0x968d => {
                w.operand_then_internal(2);
                let base = match self.pc {
                    0x9505 | 0x9514 | 0x95f3 | 0x9606 => 0x009b3a,
                    0x968d => 0x009b46,
                    _ => 0x009b42,
                };
                let v = w.read(base + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x9508 | 0x9517 | 0x958d | 0x9593 | 0x9599 | 0x95a6 | 0x95e7 | 0x95f6 | 0x9609
            | 0x9690 | 0x9699 | 0x96d5 => {
                w.dp_store(
                    match self.pc {
                        0x9508 | 0x95f6 => 0xb2,
                        0x9517 | 0x9609 => 0xb4,
                        0x9593 | 0x95e7 | 0x96d5 => 8,
                        0x9690 => 4,
                        0x9699 => 0x0a,
                        _ => 0x0e,
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 2
            }
            0x950d | 0x951c | 0x9523 | 0x952c | 0x953d | 0x9548 | 0x9559 | 0x9570 | 0x9576
            | 0x9582 | 0x95ff | 0x9612 | 0x9619 | 0x9625 | 0x9636 | 0x9641 | 0x9656 | 0x966c
            | 0x9672 | 0x967e => {
                w.drain(16);
                let a = match self.pc {
                    0x950d | 0x95ff => 0x0686,
                    0x951c | 0x9619 => 0x0684,
                    0x9523 | 0x9612 => 0x0688,
                    0x952c | 0x953d | 0x9625 | 0x9636 => 0x0680,
                    0x9548 | 0x9559 | 0x9641 | 0x9656 => 0x0682,
                    0x9570 | 0x966c => 0x0440,
                    0x9576 | 0x9672 => 0x0448,
                    _ => 0x049e,
                };
                w.write(a, w.cpu.a, 2);
                self.pc + 3
            }
            0x951f | 0x95fb | 0x960e | 0x9615 | 0x9652 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0x9520 | 0x95fc | 0x960f | 0x9616 | 0x9653 => {
                w.drain(16);
                w.sbc(
                    if matches!(self.pc, 0x960f | 0x9653) {
                        8
                    } else {
                        24
                    },
                    2,
                );
                self.pc + 3
            }
            0x9526 | 0x9540 | 0x95c4 | 0x961f | 0x9639 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x9527 | 0x9541 | 0x9620 | 0x963a => {
                w.mask(
                    if matches!(self.pc, 0x9527 | 0x9620) {
                        0x007e
                    } else {
                        0x1f80
                    },
                    2,
                );
                self.pc + 3
            }
            0x9535 | 0x9539 | 0x9551 | 0x9555 | 0x95c5 | 0x95cb | 0x95e3 | 0x962e | 0x9632
            | 0x964a | 0x964e | 0x96d1 => {
                w.clear_carry();
                self.pc + 1
            }
            0x9536 | 0x953a | 0x9552 | 0x9556 | 0x962f | 0x9633 | 0x964b | 0x964f => {
                w.drain(16);
                let a = match self.pc {
                    0x9536 | 0x962f => 0x062c,
                    0x953a | 0x9633 => 0x0680,
                    0x9552 | 0x964b => 0x062e,
                    _ => 0x0682,
                };
                let v = w.read(a, 2);
                w.adc(v);
                self.pc + 3
            }
            0x9544..=0x9547 | 0x963d..=0x9640 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a & 1) as u8;
                w.load(w.cpu.a >> 1, 2);
                self.pc + 1
            }
            0x955c | 0x9659 | 0x956b | 0x95d5 | 0x9667 | 0x9687 => {
                w.width(0x30, !matches!(self.pc, 0x955c | 0x9659));
                self.pc + 2
            }
            0x955e | 0x965b => {
                w.drain(16);
                let v = w.read(0x0403, 1);
                w.load(v, 1);
                self.pc + 3
            }
            0x9561 | 0x965e => {
                w.drain(16);
                let v = w.read(0x0098c9, 1);
                w.load(w.cpu.a & v, 1);
                self.pc + 3
            }
            0x9564 | 0x9661 | 0x95d2 | 0x95ec | 0x96cd | 0x96d9 => {
                let equal = matches!(self.pc, 0x9564 | 0x9661);
                if w.branch((w.cpu.p & ZERO != 0) == equal) {
                    match self.pc {
                        0x9564 => 0x95d5,
                        0x9661 => 0x9667,
                        0x95d2 => 0x95a8,
                        0x95ec => 0x95dc,
                        0x96cd => 0x969b,
                        _ => 0x9695,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x9566 | 0x9663 => {
                w.dp_store(0xaf, 0, 1);
                self.pc + 2
            }
            0x9568 | 0x9579 | 0x957c | 0x9585 | 0x961c | 0x9675 | 0x9678 | 0x9681 | 0x9684 => {
                w.drain(16);
                let a = match self.pc {
                    0x9568 | 0x9684 => 0x0414,
                    0x9579 | 0x9675 => 0x0444,
                    0x957c | 0x9678 => 0x0442,
                    0x961c => 0x068a,
                    _ => 0x04ae,
                };
                w.write(a, 0, if self.pc == 0x9568 { 1 } else { 2 });
                self.pc + 3
            }
            0x956d | 0x9573 | 0x957f | 0x9669 | 0x966f | 0x967b => {
                w.drain(16);
                let v = w.read(
                    match self.pc {
                        0x956d | 0x9669 => 0x0442,
                        0x9573 | 0x966f => 0x0444,
                        _ => 0x04ae,
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0x958a | 0x9597 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1), 2);
                self.pc + 1
            }
            0x9591 | 0x959d => {
                let v = w.dp_read(0x0e, 2);
                w.adc(v);
                self.pc + 2
            }
            0x9598 => {
                w.swap_bytes();
                0x9599
            }
            0x95a0 => {
                w.drain(16);
                w.cpu.y = 0x1438;
                w.flags(w.cpu.y, 2);
                0x95a3
            }
            0x95a3 => {
                w.immediate(4, 2);
                0x95a6
            }
            0x95a8 | 0x95af | 0x95b6 | 0x95bd => {
                w.operand_then_internal(2);
                let v = w.read(
                    0x009b52 + u32::from(w.cpu.y) + u32::from((self.pc - 0x95a8) / 7 * 2),
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0x95ab | 0x95b2 | 0x95b9 | 0x95c0 => {
                w.drain(24);
                w.write(
                    0x7e2000 + u32::from(w.cpu.x) + u32::from((self.pc - 0x95ab) / 7 * 2),
                    w.cpu.a,
                    2,
                );
                self.pc + 4
            }
            0x95c6 | 0x95cc | 0x95e4 | 0x96d2 => {
                w.drain(16);
                w.adc(match self.pc {
                    0x95c6 => 8,
                    0x95cc => 0x80,
                    0x95e4 => 0x200,
                    _ => 0x100,
                });
                self.pc + 3
            }
            0x95c9 | 0x95e9 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x95ca => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                0x95cb
            }
            0x95d0 | 0x95ea | 0x96cb | 0x96d7 => {
                w.dp_increment(
                    match self.pc {
                        0x95d0 => 0x0e,
                        0x95ea => 0xb4,
                        0x96cb => 0x0a,
                        _ => 4,
                    },
                    2,
                    false,
                );
                self.pc + 2
            }
            0x95d7 | 0x9692 => {
                w.drain(16);
                w.cpu.x = 0x0110;
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x95da | 0x9695 => {
                w.cpu.y = w.dp_read(8, 2);
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0x95de => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.blocks = Some(DungeonFloors::blocks());
                0x8a44
            }
            0x9665 => {
                w.branch(true);
                0x9687
            }
            0x969b | 0x96a0 | 0x96a5 | 0x96aa | 0x96af | 0x96b4 | 0x96b9 | 0x96be => {
                w.operand_then_internal(2);
                let v = w.read(
                    0x009b52 + u32::from(w.cpu.x) + u32::from((self.pc - 0x969b) / 5 * 2),
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0x969e | 0x96a3 | 0x96a8 | 0x96ad | 0x96b2 | 0x96b7 | 0x96bc | 0x96c1 => {
                let a = w.indirect_y_address(0xbf + (self.pc - 0x969e) / 5 * 3);
                w.write(a, w.cpu.a, 2);
                self.pc + 2
            }
            0x96c3..=0x96ca => {
                w.increment_y();
                self.pc + 1
            }
            0x95d4 | 0x95ee | 0x96db => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => panic!("unowned water-overlay continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
