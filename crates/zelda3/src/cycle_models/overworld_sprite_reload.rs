//! Retained overworld sprite reload: disable/reset, presence records and
//! the complete horizontal proximity scan, including normal and overlord spawns.
//! Fixed instruction operands execute without decoding cartridge instructions.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
    sprite_reset::SpriteReset,
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldSpriteReload {
    pc: u16,
    calls: Vec<u16>,
    reset: Option<SpriteReset>,
    complete: bool,
}
impl Default for OverworldSpriteReload {
    fn default() -> Self {
        Self {
            pc: 0xc499,
            calls: Vec::new(),
            reset: None,
            complete: false,
        }
    }
}
impl OverworldSpriteReload {
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
        if let Some(reset) = self.reset.as_mut() {
            let cycles = reset.step(cpu, clock, bus);
            if reset.complete() {
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp),
                    (
                        9,
                        self.pc,
                        self.calls.last().copied().expect("reload reset frame")
                    )
                );
                self.calls.pop();
                self.reset = None;
            }
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (9, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let xw = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let xmask = if xw == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match pc {
            0xc499 | 0xc49d => {
                let sp = w.cpu.sp;
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(pc + 3, 2);
                w.cpu.bank = (match pc {
                    0xc499 => 0x9c22f,
                    0xc49d => 0x9c452,
                    _ => unreachable!(),
                } >> 16) as u8;
                w.cpu.pc = match pc {
                    0xc499 => 0x9c22f,
                    0xc49d => 0x9c452,
                    _ => unreachable!(),
                } as u16;
                self.calls.push(sp);
                self.reset = Some(SpriteReset::from_entry(match pc {
                    0xc499 => 0x9c22f,
                    0xc49d => 0x9c452,
                    _ => unreachable!(),
                }));
                pc + 4
            }
            0xc4a1 => {
                w.drain(6);
                w.push(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xc4a2 => {
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                pc + 1
            }
            0xc4a3 | 0xc4aa => {
                w.drain(12);
                let value = w.pull(1);
                w.cpu.db = value as u8;
                w.flags(value, 1);
                pc + 1
            }
            0xc4a4 | 0xc4a7 | 0xc570 | 0xc5e3 | 0xc72a | 0xc76c | 0xc776 => {
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                self.calls.push(pc + 3);
                match pc {
                    0xc4a4 => 0xc4ac,
                    0xc4a7 => 0xc55e,
                    0xc570 => 0xc5bb,
                    0xc5e3 => 0xc6f5,
                    0xc72a => 0xc739,
                    0xc76c => 0xc770,
                    0xc776 => 0xc80b,
                    _ => unreachable!(),
                }
            }
            0xc4ab => {
                w.rtl();
                assert!(self.calls.is_empty());
                self.complete = true;
                w.cpu.pc
            }
            0xc4ac | 0xc4b8 | 0xc4c7 | 0xc4dc | 0xc564 | 0xc5bd | 0xc871 => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match pc {
                        0xc4ac => 0x40a,
                        0xc4b8 => 0x40a,
                        0xc4c7 => 0x40a,
                        0xc4dc => 0x40a,
                        0xc564 => 0x69f,
                        0xc5bd => 0x69f,
                        0xc871 => 0x40a,
                        _ => unreachable!(),
                    }),
                    m,
                );
                w.load(value, m);
                pc + 3
            }
            0xc4af | 0xc4bb | 0xc4bf | 0xc548 | 0xc71b | 0xc762 | 0xc7da | 0xc7e1 | 0xc847
            | 0xc84e => {
                w.drain(8 * u32::from(m));
                let value = match pc {
                    0xc4af => 0x7,
                    0xc4bb => 0x3f,
                    0xc4bf => 0xe,
                    0xc548 => 0xf,
                    0xc71b => 0xf0,
                    0xc762 => 0x7,
                    0xc7da => 0xf0,
                    0xc7e1 => 0x3,
                    0xc847 => 0xf0,
                    0xc84e => 0x3,
                    _ => unreachable!(),
                };
                w.load(w.cpu.a & value & mask, m);
                pc + 1 + u16::from(m)
            }
            0xc4b1 | 0xc4df | 0xc52f | 0xc530 | 0xc540 | 0xc541 | 0xc542 | 0xc543 | 0xc713
            | 0xc714 | 0xc7b8 | 0xc7d1 | 0xc7d2 | 0xc7d3 | 0xc7d4 | 0xc81f | 0xc836 | 0xc837
            | 0xc838 | 0xc839 => {
                w.shift_left(m);
                pc + 1
            }
            0xc4b2 | 0xc4c4 | 0xc4d1 | 0xc4d7 => {
                w.drain(16);
                w.write(
                    w.absolute(match pc {
                        0xc4b2 => 0xfbc,
                        0xc4c4 => 0xfbe,
                        0xc4d1 => 0xfb8,
                        0xc4d7 => 0xfba,
                        _ => unreachable!(),
                    }),
                    0,
                    m,
                );
                pc + 3
            }
            0xc4b5 | 0xc4c1 | 0xc56a | 0xc585 => {
                w.drain(16);
                w.write(
                    w.absolute(match pc {
                        0xc4b5 => 0xfbd,
                        0xc4c1 => 0xfbf,
                        0xc56a => 0x69f,
                        0xc585 => 0x69f,
                        _ => unreachable!(),
                    }),
                    w.cpu.a & mask,
                    m,
                );
                pc + 3
            }
            0xc4bd | 0xc4be | 0xc52b | 0xc52c | 0xc52d | 0xc52e | 0xc536 | 0xc537 | 0xc538
            | 0xc539 | 0xc721 | 0xc722 | 0xc723 | 0xc724 | 0xc751 | 0xc752 | 0xc753 | 0xc7e8
            | 0xc7e9 | 0xc855 | 0xc856 => {
                w.drain(6);
                let value = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(value & 1 != 0);
                w.load(value >> 1, m);
                pc + 1
            }
            0xc4ca | 0xc4e0 | 0xc764 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & xmask;
                w.flags(w.cpu.y, xw);
                pc + 1
            }
            0xc4cb => {
                w.drain(16);
                let base: u16 = match pc {
                    0xc4cb => 0xc635,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.y)), xw);
                w.cpu.x = value;
                w.flags(value, xw);
                pc + 3
            }
            0xc4ce | 0xc4d4 => {
                w.drain(16);
                w.write(
                    w.absolute(match pc {
                        0xc4ce => 0xfb9,
                        0xc4d4 => 0xfbb,
                        _ => unreachable!(),
                    }),
                    w.cpu.x,
                    xw,
                );
                pc + 3
            }
            0xc4da | 0xc5e6 | 0xc6f5 | 0xc739 | 0xc74d | 0xc7ba | 0xc821 => {
                w.width(
                    match pc {
                        0xc4da => 0x30,
                        0xc5e6 => 0x20,
                        0xc6f5 => 0x20,
                        0xc739 => 0x20,
                        0xc74d => 0x20,
                        0xc7ba => 0x20,
                        0xc821 => 0x20,
                        _ => unreachable!(),
                    },
                    true,
                );
                pc + 2
            }
            0xc4e1 | 0xc55b | 0xc5f0 | 0xc710 | 0xc72e | 0xc743 | 0xc75a | 0xc7c1 | 0xc828 => {
                w.width(
                    match pc {
                        0xc4e1 => 0x20,
                        0xc55b => 0x10,
                        0xc5f0 => 0x20,
                        0xc710 => 0x20,
                        0xc72e => 0x20,
                        0xc743 => 0x20,
                        0xc75a => 0x20,
                        0xc7c1 => 0x20,
                        0xc828 => 0x20,
                        _ => unreachable!(),
                    },
                    false,
                );
                pc + 2
            }
            0xc4e3 => {
                w.drain(24);
                let value = w.read(
                    match pc {
                        0xc4e3 => 0x7ef3c5,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load(value, m);
                pc + 4
            }
            0xc4e7 | 0xc4eb | 0xc512 | 0xc51c | 0xc772 | 0xc77c | 0xc782 | 0xc788 | 0xc78c
            | 0xc790 | 0xc794 | 0xc7a2 => {
                w.drain(8 * u32::from(m));
                let value = match pc {
                    0xc4e7 => 0x3,
                    0xc4eb => 0x2,
                    0xc512 => 0xff,
                    0xc51c => 0xf4,
                    0xc772 => 0xf4,
                    0xc77c => 0x58,
                    0xc782 => 0xd0,
                    0xc788 => 0x58,
                    0xc78c => 0xeb,
                    0xc790 => 0x53,
                    0xc794 => 0xf3,
                    0xc7a2 => 0x41,
                    _ => unreachable!(),
                };
                let old = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old >= value);
                w.flags(old.wrapping_sub(value) & mask, m);
                pc + 1 + u16::from(m)
            }
            0xc4e9 | 0xc4ed | 0xc514 | 0xc5c0 | 0xc74b | 0xc77e | 0xc784 | 0xc78a | 0xc78e
            | 0xc792 | 0xc79d | 0xc810 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    match pc {
                        0xc4e9 => 0xc4f9,
                        0xc4ed => 0xc503,
                        0xc514 => 0xc55b,
                        0xc5c0 => 0xc5f5,
                        0xc74b => 0xc76f,
                        0xc77e => 0xc79a,
                        0xc784 => 0xc79a,
                        0xc78a => 0xc798,
                        0xc78e => 0xc798,
                        0xc792 => 0xc798,
                        0xc79d => 0xc7af,
                        0xc810 => 0xc816,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xc4ef | 0xc4f4 | 0xc4f9 | 0xc4fe | 0xc503 | 0xc508 => {
                w.drain(16);
                let base: u16 = match pc {
                    0xc4ef => 0xc881,
                    0xc4f4 => 0xc882,
                    0xc4f9 => 0xca21,
                    0xc4fe => 0xca22,
                    0xc503 => 0xc901,
                    0xc508 => 0xc902,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.y)), m);
                w.load(value, m);
                pc + 3
            }
            0xc4f2 | 0xc4fc | 0xc506 | 0xc50b | 0xc531 | 0xc53d | 0xc544 | 0xc54c | 0xc579
            | 0xc57f | 0xc589 | 0xc58c | 0xc5cb | 0xc5d2 | 0xc5d9 | 0xc5df | 0xc5ee | 0xc703
            | 0xc717 | 0xc71d | 0xc727 | 0xc741 | 0xc747 | 0xc758 | 0xc75e => {
                w.dp_store(
                    match pc {
                        0xc4f2 => 0x0,
                        0xc4fc => 0x0,
                        0xc506 => 0x0,
                        0xc50b => 0x1,
                        0xc531 => 0x2,
                        0xc53d => 0x6,
                        0xc544 => 0x7,
                        0xc54c => 0x5,
                        0xc579 => 0xe2,
                        0xc57f => 0xe3,
                        0xc589 => 0xe3,
                        0xc58c => 0xe2,
                        0xc5cb => 0xe,
                        0xc5d2 => 0xf,
                        0xc5d9 => 0xc,
                        0xc5df => 0xd,
                        0xc5ee => 0xc,
                        0xc703 => 0x0,
                        0xc717 => 0x1,
                        0xc71d => 0x0,
                        0xc727 => 0x0,
                        0xc741 => 0x5,
                        0xc747 => 0x7,
                        0xc758 => 0x2,
                        0xc75e => 0x4,
                        _ => unreachable!(),
                    },
                    w.cpu.a & mask,
                    m,
                );
                pc + 2
            }
            0xc4f7 | 0xc501 | 0xc526 | 0xc559 => {
                if w.branch(true) {
                    match pc {
                        0xc4f7 => 0xc50b,
                        0xc501 => 0xc50b,
                        0xc526 => 0xc510,
                        0xc559 => 0xc510,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xc50d | 0xc56d | 0xc5bb => {
                w.drain(8 * u32::from(xw));
                let value = match pc {
                    0xc50d => 0x0,
                    0xc56d => 0x15,
                    0xc5bb => 0x0,
                    _ => unreachable!(),
                };
                w.cpu.y = value;
                w.flags(value, xw);
                pc + 1 + u16::from(xw)
            }
            0xc510 | 0xc518 | 0xc528 | 0xc534 | 0xc546 | 0xc54f => {
                w.drain(8);
                let base = w.read(
                    match pc {
                        0xc510 => 0x0,
                        0xc518 => 0x0,
                        0xc528 => 0x0,
                        0xc534 => 0x0,
                        0xc546 => 0x0,
                        0xc54f => 0x0,
                        _ => unreachable!(),
                    },
                    2,
                );
                if xw == 2 || (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.y)), m);
                w.load(value, m);
                pc + 2
            }
            0xc516 | 0xc517 | 0xc523 | 0xc524 | 0xc525 | 0xc533 | 0xc54e | 0xc558 | 0xc5c4 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & xmask;
                w.flags(w.cpu.y, xw);
                pc + 1
            }
            0xc51a | 0xc51b | 0xc581 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & xmask;
                w.flags(w.cpu.y, xw);
                pc + 1
            }
            0xc51e | 0xc76a | 0xc796 | 0xc7a4 | 0xc7a9 | 0xc83d => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match pc {
                        0xc51e => 0xc528,
                        0xc76a => 0xc76f,
                        0xc796 => 0xc79a,
                        0xc7a4 => 0xc7ab,
                        0xc7a9 => 0xc7af,
                        0xc83d => 0xc842,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xc520 => {
                w.drain(16);
                let address = w.absolute(match pc {
                    0xc520 => 0xffd,
                    _ => unreachable!(),
                });
                let value = w.read(address, m).wrapping_add(1) & mask;
                w.drain(6);
                w.flags(value, m);
                w.write_high_low(address, value, m);
                pc + 3
            }
            0xc52a | 0xc560 | 0xc563 | 0xc567 | 0xc833 => {
                w.drain(6);
                w.push(w.cpu.a & mask, m);
                pc + 1
            }
            0xc53a | 0xc576 | 0xc5c7 | 0xc5ea | 0xc73d | 0xc754 | 0xc7f0 | 0xc7fa | 0xc83f
            | 0xc85d | 0xc867 => {
                w.drain(6);
                w.cpu.p &= !CARRY;
                pc + 1
            }
            0xc53b => {
                let value = w.dp_read(
                    match pc {
                        0xc53b => 0x2,
                        _ => unreachable!(),
                    },
                    m,
                );
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                pc + 2
            }
            0xc53f | 0xc584 | 0xc588 | 0xc58b => {
                w.drain(12);
                let value = w.pull(m);
                w.load(value, m);
                pc + 1
            }
            0xc54a | 0xc715 | 0xc725 => {
                let value = w.dp_read(
                    match pc {
                        0xc54a => 0x7,
                        0xc715 => 0x0,
                        0xc725 => 0x0,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load((w.cpu.a | value) & mask, m);
                pc + 2
            }
            0xc551 => {
                let value = w.dp_read(
                    match pc {
                        0xc551 => 0x5,
                        _ => unreachable!(),
                    },
                    xw,
                );
                w.cpu.x = value;
                w.flags(value, xw);
                pc + 2
            }
            0xc553 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_add(1) & mask, m);
                pc + 1
            }
            0xc554 => {
                w.drain(24);
                w.write(
                    (match pc {
                        0xc554 => 0x7fdf80,
                        _ => unreachable!(),
                    } as u32)
                        + u32::from(w.cpu.x),
                    w.cpu.a & mask,
                    m,
                );
                pc + 4
            }
            0xc55d | 0xc58e | 0xc5f5 | 0xc730 | 0xc76f | 0xc779 | 0xc7ae | 0xc80a | 0xc815
            | 0xc880 => {
                w.rts();
                assert_eq!(w.cpu.pc, self.calls.pop().expect("reload near frame"));
                w.cpu.pc
            }
            0xc55e | 0xc561 | 0xc574 | 0xc57b | 0xc5c5 | 0xc5cd | 0xc5d4 | 0xc5db | 0xc5e8
            | 0xc6f7 | 0xc705 | 0xc719 | 0xc71f | 0xc73b | 0xc74f | 0xc760 | 0xc7bc | 0xc7cf
            | 0xc7d8 | 0xc7df | 0xc7e6 | 0xc823 | 0xc834 | 0xc845 | 0xc84c | 0xc853 => {
                let value = w.dp_read(
                    match pc {
                        0xc55e => 0xe2,
                        0xc561 => 0xe3,
                        0xc574 => 0xe2,
                        0xc57b => 0xe3,
                        0xc5c5 => 0xe2,
                        0xc5cd => 0xe3,
                        0xc5d4 => 0xe8,
                        0xc5db => 0xe9,
                        0xc5e8 => 0xc,
                        0xc6f7 => 0xe,
                        0xc705 => 0xc,
                        0xc719 => 0xc,
                        0xc71f => 0xe,
                        0xc73b => 0x0,
                        0xc74f => 0x0,
                        0xc760 => 0x0,
                        0xc7bc => 0x0,
                        0xc7cf => 0x0,
                        0xc7d8 => 0x0,
                        0xc7df => 0x1,
                        0xc7e6 => 0x1,
                        0xc823 => 0x0,
                        0xc834 => 0x0,
                        0xc845 => 0x0,
                        0xc84c => 0x1,
                        0xc853 => 0x1,
                        _ => unreachable!(),
                    },
                    m,
                );
                w.load(value, m);
                pc + 2
            }
            0xc568 | 0xc745 | 0xc75c | 0xc7ca => {
                w.drain(8 * u32::from(m));
                let value = match pc {
                    0xc568 => 0xff,
                    0xc745 => 0x7f,
                    0xc75c => 0x7f,
                    0xc7ca => 0x8,
                    _ => unreachable!(),
                };
                w.load(value, m);
                pc + 1 + u16::from(m)
            }
            0xc56f => {
                w.drain(6);
                w.push(w.cpu.y, xw);
                pc + 1
            }
            0xc573 | 0xc83a => {
                w.drain(12);
                let value = w.pull(xw);
                w.cpu.y = value;
                w.flags(value, xw);
                pc + 1
            }
            0xc577 | 0xc57d | 0xc5eb | 0xc73e | 0xc755 | 0xc840 => {
                w.drain(8 * u32::from(m));
                let value = match pc {
                    0xc577 => 0x10,
                    0xc57d => 0x0,
                    0xc5eb => 0x10,
                    0xc73e => 0xdf80,
                    0xc755 => 0xef80,
                    0xc840 => 0x8,
                    _ => unreachable!(),
                };
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                pc + 1 + u16::from(m)
            }
            0xc582 | 0xc5c2 | 0xc5f3 | 0xc7ac | 0xc813 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    match pc {
                        0xc582 => 0xc56f,
                        0xc5c2 => 0xc5c5,
                        0xc5f3 => 0xc5e3,
                        0xc7ac => 0xc79a,
                        0xc813 => 0xc80d,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xc5c8 | 0xc5cf => {
                w.drain(16);
                let base: u16 = match pc {
                    0xc5c8 => 0xc5b7,
                    0xc5cf => 0xc5b9,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.y)), m);
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                pc + 3
            }
            0xc5d6 | 0xc6f9 | 0xc707 | 0xc82d => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xc5d7 | 0xc5dd | 0xc82e => {
                w.drain(8 * u32::from(m));
                let value = match pc {
                    0xc5d7 => 0x30,
                    0xc5dd => 0x0,
                    0xc82e => 0xf3,
                    _ => unreachable!(),
                };
                w.sbc(value, m);
                pc + 1 + u16::from(m)
            }
            0xc5e1 | 0xc77a | 0xc780 | 0xc786 | 0xc798 | 0xc80b => {
                w.drain(8 * u32::from(xw));
                let value = match pc {
                    0xc5e1 => 0x15,
                    0xc77a => 0x4,
                    0xc780 => 0x5,
                    0xc786 => 0xd,
                    0xc798 => 0xe,
                    0xc80b => 0x7,
                    _ => unreachable!(),
                };
                w.cpu.x = value;
                w.flags(value, xw);
                pc + 1 + u16::from(xw)
            }
            0xc5f2 | 0xc7ab | 0xc812 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xc6fa | 0xc708 => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match pc {
                        0xc6fa => 0xfbc,
                        0xc708 => 0xfbe,
                        _ => unreachable!(),
                    }),
                    m,
                );
                w.sbc(value, m);
                pc + 3
            }
            0xc6fd | 0xc70b => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match pc {
                        0xc6fd => 0xfb8,
                        0xc70b => 0xfba,
                        _ => unreachable!(),
                    }),
                    m,
                );
                let old = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old >= value);
                w.flags(old.wrapping_sub(value) & mask, m);
                pc + 3
            }
            0xc700 | 0xc70e => {
                if w.branch(w.cpu.p & CARRY != 0) {
                    match pc {
                        0xc700 => 0xc72e,
                        0xc70e => 0xc72e,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xc702 | 0xc712 => {
                w.swap_bytes();
                pc + 1
            }
            0xc729 | 0xc7b6 | 0xc81d => {
                w.drain(6);
                w.push(w.cpu.x, xw);
                pc + 1
            }
            0xc72d | 0xc7c3 | 0xc82a => {
                w.drain(12);
                let value = w.pull(xw);
                w.cpu.x = value;
                w.flags(value, xw);
                pc + 1
            }
            0xc749 | 0xc765 | 0xc770 | 0xc7af | 0xc7c4 | 0xc816 | 0xc82b => {
                w.drain(8);
                let low = w.read(
                    match pc {
                        0xc749 => 0x5,
                        0xc765 => 0x2,
                        0xc770 => 0x5,
                        0xc7af => 0x2,
                        0xc7c4 => 0x5,
                        0xc816 => 0x2,
                        0xc82b => 0x5,
                        _ => unreachable!(),
                    },
                    2,
                );
                let bank = w.read(
                    match pc {
                        0xc749 => 0x5,
                        0xc765 => 0x2,
                        0xc770 => 0x5,
                        0xc7af => 0x2,
                        0xc7c4 => 0x5,
                        0xc816 => 0x2,
                        0xc82b => 0x5,
                        _ => unreachable!(),
                    } + 2,
                    1,
                );
                let value = w.read((u32::from(bank) << 16) | u32::from(low), m);
                w.load(value, m);
                pc + 2
            }
            0xc767 => {
                w.drain(16);
                let base: u16 = match pc {
                    0xc767 => 0xc731,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.y)), m);
                w.load(w.cpu.a & value & mask, m);
                pc + 3
            }
            0xc774 => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    match pc {
                        0xc774 => 0xc77a,
                        _ => unreachable!(),
                    }
                } else {
                    pc + 2
                }
            }
            0xc79a | 0xc79f | 0xc7a6 | 0xc7ed | 0xc7f7 | 0xc80d | 0xc85a | 0xc864 => {
                w.drain(16);
                let base: u16 = match pc {
                    0xc79a => 0xdd0,
                    0xc79f => 0xe20,
                    0xc7a6 => 0xdb0,
                    0xc7ed => 0xd30,
                    0xc7f7 => 0xd20,
                    0xc80d => 0xb00,
                    0xc85a => 0xb10,
                    0xc864 => 0xb20,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.x)), m);
                w.load(value, m);
                pc + 3
            }
            0xc7b1 | 0xc818 => {
                w.drain(16);
                let base: u16 = match pc {
                    0xc7b1 => 0xc731,
                    0xc818 => 0xc731,
                    _ => unreachable!(),
                };
                if xw == 2 || (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base.wrapping_add(w.cpu.y)), m);
                w.load((w.cpu.a | value) & mask, m);
                pc + 3
            }
            0xc7b4 | 0xc81b => {
                w.drain(8);
                let low = w.read(
                    match pc {
                        0xc7b4 => 0x2,
                        0xc81b => 0x2,
                        _ => unreachable!(),
                    },
                    2,
                );
                let bank = w.read(
                    match pc {
                        0xc7b4 => 0x2,
                        0xc81b => 0x2,
                        _ => unreachable!(),
                    } + 2,
                    1,
                );
                w.write((u32::from(bank) << 16) | u32::from(low), w.cpu.a & mask, m);
                pc + 2
            }
            0xc7b7 | 0xc81e => {
                w.drain(6);
                w.load(w.cpu.x, m);
                pc + 1
            }
            0xc7b9 | 0xc820 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & xmask;
                w.flags(w.cpu.x, xw);
                pc + 1
            }
            0xc7be | 0xc7c7 | 0xc7cc | 0xc7d5 | 0xc7dc | 0xc7e3 | 0xc7ea | 0xc7f4 | 0xc7fe
            | 0xc825 | 0xc830 | 0xc842 | 0xc849 | 0xc850 | 0xc857 | 0xc861 | 0xc86b | 0xc874 => {
                w.drain(16);
                w.drain(6);
                let base: u16 = match pc {
                    0xc7be => 0xbc0,
                    0xc7c7 => 0xe20,
                    0xc7cc => 0xdd0,
                    0xc7d5 => 0xd10,
                    0xc7dc => 0xd00,
                    0xc7e3 => 0xd30,
                    0xc7ea => 0xd20,
                    0xc7f4 => 0xd30,
                    0xc7fe => 0xd20,
                    0xc825 => 0xb48,
                    0xc830 => 0xb00,
                    0xc842 => 0xb08,
                    0xc849 => 0xb18,
                    0xc850 => 0xb10,
                    0xc857 => 0xb20,
                    0xc861 => 0xb10,
                    0xc86b => 0xb20,
                    0xc874 => 0xcca,
                    _ => unreachable!(),
                };
                w.write(w.absolute(base.wrapping_add(w.cpu.x)), w.cpu.a & mask, m);
                pc + 3
            }
            0xc7c6 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1) & mask, m);
                pc + 1
            }
            0xc7f1 | 0xc7fb | 0xc85e | 0xc868 => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match pc {
                        0xc7f1 => 0xfbd,
                        0xc7fb => 0xfbf,
                        0xc85e => 0xfbd,
                        0xc868 => 0xfbf,
                        _ => unreachable!(),
                    }),
                    m,
                );
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                pc + 3
            }
            0xc801 | 0xc804 | 0xc807 | 0xc86e | 0xc877 | 0xc87a | 0xc87d => {
                w.drain(16);
                w.drain(6);
                let base: u16 = match pc {
                    0xc801 => 0xf20,
                    0xc804 => 0xe30,
                    0xc807 => 0xcba,
                    0xc86e => 0xb40,
                    0xc877 => 0xb30,
                    0xc87a => 0xb28,
                    0xc87d => 0xb38,
                    _ => unreachable!(),
                };
                w.write(w.absolute(base.wrapping_add(w.cpu.x)), 0, m);
                pc + 3
            }
            0xc83b => {
                w.drain(8 * u32::from(xw));
                let value = match pc {
                    0xc83b => 0x1,
                    _ => unreachable!(),
                };
                let old = w.cpu.y & xmask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old >= value);
                w.flags(old.wrapping_sub(value) & xmask, xw);
                pc + 1 + u16::from(xw)
            }
            _ => unreachable!("overworld sprite reload PC {pc:04x}"),
        };
        w.cpu.pc = if self.reset.is_some() {
            w.cpu.pc
        } else {
            self.pc
        };
        Some(w.cycles)
    }
}
