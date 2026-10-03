//! Retained polygon edge walking and bitmap writes. The live CPU stack owns
//! calls/returns; the phase owns progress while IRQ/NMI handlers run elsewhere.
use super::{native_work::{Work, WorkBus}, nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Body(u8),
    Edge { right: bool, step: u8 },
    Slope { right: bool, step: u8 },
    Span(u8),
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PolyRaster {
    phase: Phase,
    edge_return: u8,
}
impl Default for PolyRaster {
    fn default() -> Self {
        Self {
            phase: Phase::Body(0),
            edge_return: 0,
        }
    }
}
impl PolyRaster {
    pub(super) fn defers_irq_mask_change(&self) -> bool {
        matches!(self.phase, Phase::Slope { step: 11 | 31, .. }) // SEI / CLI
    }
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    fn pc(&self) -> u16 {
        match self.phase {
            Phase::Body(i) => BODY_PC[usize::from(i)],
            Phase::Span(i) => SPAN_PC[usize::from(i)],
            Phase::Edge { right, step } => {
                if right {
                    RIGHT_PC[usize::from(step)]
                } else {
                    LEFT_PC[usize::from(step)]
                }
            }
            Phase::Slope { right, step } => {
                SLOPE_PC[usize::from(step)] + if right { 0x6e } else { 0 }
            }
            Phase::Complete => unreachable!("return address belongs to the live stack"),
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (9, self.pc(), 0x1f00));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let width = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mut next = match self.phase {
            Phase::Body(i) => Phase::Body(i + 1),
            Phase::Span(i) => Phase::Span(i + 1),
            Phase::Edge { right, step } => Phase::Edge {
                right,
                step: step + 1,
            },
            Phase::Slope { right, step } => Phase::Slope {
                right,
                step: step + 1,
            },
            Phase::Complete => unreachable!(),
        };
        match self.phase {
            Phase::Body(i) => match i {
                0 => {
                    w.drain(6);
                    w.push(u16::from(w.cpu.p), 1);
                }
                1 => {
                    w.drain(6);
                    w.push(u16::from(w.cpu.db), 1);
                }
                2 => w.width(0x30, false),
                3 => w.immediate(0x7e, 1),
                4 => {
                    w.drain(6);
                    w.push(w.cpu.a & 255, 1);
                }
                5 | 45 => {
                    w.drain(12);
                    w.cpu.db = w.pull(1) as u8;
                    w.flags(u16::from(w.cpu.db), 1);
                }
                6 => {
                    let v = w.dp_read(0x1fc0, 1);
                    w.cpu.y = v;
                    w.flags(v, 1);
                }
                7 => {
                    let v = w.cpu.y;
                    transfer_index(&mut w, true, v);
                }
                8 | 16 => {
                    let v = dp_x(&mut w, 0xc0);
                    w.load(v, 1);
                }
                9 | 10 => index_step(&mut w, false, 1),
                11 => {
                    if w.branch(w.cpu.p & 2 != 0) {
                        next = Phase::Body(18);
                    }
                }
                12 => {
                    let v = dp_x(&mut w, 0xc0);
                    compare(&mut w, v, false, 1);
                }
                13 | 14 => {
                    if w.branch(if i == 13 {
                        w.cpu.p & 1 == 0
                    } else {
                        w.cpu.p & 2 != 0
                    }) {
                        next = Phase::Body(9);
                    }
                }
                15 => {
                    let v = w.cpu.x;
                    transfer_index(&mut w, false, v);
                }
                17 => {
                    w.branch(true);
                    next = Phase::Body(9);
                }
                18 => w.mask(7, 1),
                19 => w.shift_left(1),
                20 | 29 | 34 | 36 | 37 | 39 | 40 | 54 | 62 => {
                    let at = match i {
                        20 | 54 => 0xb9,
                        29 | 62 => 0xba,
                        34 => 0xe0,
                        36 => 0xe2,
                        37 => 0xeb,
                        39 => 0xe1,
                        40 => 0xea,
                        _ => unreachable!(),
                    };
                    w.dp_store(0x1f00 + at, w.cpu.a & 255, 1);
                }
                21 | 35 | 38 => {
                    let v = indexed_read(&mut w, if i == 38 { 0x1fbf } else { 0x1fc0 }, 1);
                    w.load(v, 1);
                }
                22 => w.mask(0x38, 1),
                23 | 59 => bit_immediate(&mut w, if i == 23 { 0x20 } else { 8 }),
                24 => {
                    if w.branch(w.cpu.p & 2 != 0) {
                        next = Phase::Body(26);
                    }
                }
                25 | 61 => {
                    w.drain(8);
                    let v = (w.cpu.a ^ if i == 25 { 0x24 } else { 0x19 }) & 255;
                    w.load(v, 1);
                }
                26 | 27 | 33 => lsr(&mut w, 1),
                28 => {
                    w.drain(8);
                    w.adc_byte(0xe8);
                }
                30 | 31 => w.dp_store(if i == 30 { 0x1fe9 } else { 0x1ff2 }, w.cpu.y, 1),
                32 | 49 | 56 => {
                    let v = w.dp_read(
                        match i {
                            32 => 0x1fc0,
                            49 => 0x1fb9,
                            _ => 0x1fba,
                        },
                        1,
                    );
                    w.load(v, 1);
                }
                41 | 43 | 69 | 79 => {
                    jsr(&mut w);
                    self.edge_return = i + 1;
                    next = Phase::Edge {
                        right: i == 43 || i == 79,
                        step: 0,
                    };
                }
                42 | 70 | 80 => {
                    if w.branch(w.cpu.p & 1 != 0) {
                        next = Phase::Body(45);
                    }
                }
                44 => {
                    if w.branch(w.cpu.p & 1 == 0) {
                        next = Phase::Body(48);
                    }
                }
                46 => {
                    w.drain(12);
                    w.cpu.p = w.pull(1) as u8;
                    if w.cpu.p & 0x10 != 0 {
                        w.cpu.x &= 255;
                        w.cpu.y &= 255;
                    }
                }
                47 => {
                    w.rtl();
                    self.phase = Phase::Complete;
                    return Some(w.cycles);
                }
                48 => {
                    jsr(&mut w);
                    next = Phase::Span(0);
                }
                50 | 51 | 57 | 58 => w.increment_a(1),
                52 => w.compare_immediate(0x10, 1),
                53 => {
                    if w.branch(w.cpu.p & 2 != 0) {
                        next = Phase::Body(56);
                    }
                }
                55 => {
                    w.branch(true);
                    next = Phase::Body(64);
                }
                60 => {
                    if w.branch(w.cpu.p & 2 == 0) {
                        next = Phase::Body(62);
                    }
                }
                63 => w.dp_store(0x1fb9, 0, 1),
                64 | 67 | 71 | 74 | 77 | 81 => {
                    let at = match i {
                        64 | 71 => 0x1fe2,
                        67 => 0x1fe3,
                        74 | 81 => 0x1feb,
                        _ => 0x1fec,
                    };
                    let v = w.dp_read(at, 1);
                    w.cpu.x = v;
                    w.flags(v, 1);
                }
                65 | 75 => {
                    let v = w.dp_read(if i == 65 { 0x1fe4 } else { 0x1fed }, 1);
                    compare(&mut w, v, true, 1);
                }
                66 | 76 => {
                    if w.branch(w.cpu.p & 2 == 0) {
                        next = Phase::Body(if i == 66 { 72 } else { 82 });
                    }
                }
                68 | 73 | 78 | 83 => w.dp_store(
                    match i {
                        68 => 0x1fe1,
                        73 => 0x1fe2,
                        78 => 0x1fea,
                        _ => 0x1feb,
                    },
                    w.cpu.x,
                    1,
                ),
                72 | 82 => index_step(&mut w, true, 1),
                84 => w.width(0x21, true),
                85 | 89 => {
                    let v = w.dp_read(if i == 85 { 0x1fe5 } else { 0x1fee }, 2);
                    w.load(v, 2);
                }
                86 | 90 => {
                    let v = w.dp_read(if i == 86 { 0x1fe7 } else { 0x1ff0 }, 2);
                    w.adc(v);
                }
                87 | 91 => w.dp_store(if i == 87 { 0x1fe5 } else { 0x1fee }, w.cpu.a, 2),
                88 => w.clear_carry(),
                92 => w.width(0x20, false),
                93 => {
                    w.branch(true);
                    next = Phase::Body(48);
                }
                _ => unreachable!(),
            },
            Phase::Edge { right, step } => {
                let edge = |step| Phase::Edge { right, step };
                if step < 4 {
                    match step {
                        0 => w.dp_increment(0x1fe0, 1, false),
                        1 => {
                            if w.branch(w.cpu.p & 0x80 == 0) {
                                next = edge(4);
                            }
                        }
                        2 => {
                            w.drain(6);
                            w.cpu.p |= 1;
                        }
                        3 => {
                            w.rts();
                            next = Phase::Body(self.edge_return);
                        }
                        _ => unreachable!(),
                    }
                } else if right {
                    match step {
                        4 => {
                            let v = w.dp_read(0x1ff2, 1);
                            w.cpu.x = v;
                            w.flags(v, 1);
                        }
                        5 => {
                            let v = w.dp_read(0x1fc0, 1);
                            compare(&mut w, v, true, 1);
                        }
                        6 => {
                            if w.branch(w.cpu.p & 2 == 0) {
                                next = edge(9);
                            }
                        }
                        7 => {
                            w.drain(8);
                            w.cpu.x = 2;
                            w.flags(2, 1);
                        }
                        8 => {
                            w.branch(true);
                            next = edge(11);
                        }
                        9 | 10 => index_step(&mut w, true, 1),
                        11 | 15 => {
                            let v = dp_x(&mut w, if step == 11 { 0xc0 } else { 0xbf });
                            w.load(v, 1);
                        }
                        12 => {
                            let v = w.dp_read(0x1feb, 1);
                            compare(&mut w, v, false, 1);
                        }
                        13 => {
                            if w.branch(w.cpu.p & 1 == 0) {
                                next = edge(2);
                            }
                        }
                        14 => {
                            if w.branch(w.cpu.p & 2 == 0) {
                                next = Phase::Slope { right, step: 0 };
                            }
                        }
                        16 => w.dp_store(0x1fea, w.cpu.a & 255, 1),
                        17 => w.dp_store(0x1ff2, w.cpu.x, 1),
                        18 => {
                            w.branch(true);
                            next = edge(0);
                        }
                        _ => unreachable!(),
                    }
                } else {
                    match step {
                        4 | 8 => {
                            let v = w.dp_read(if step == 4 { 0x1fe9 } else { 0x1fc0 }, 1);
                            w.cpu.x = v;
                            w.flags(v, 1);
                        }
                        5 | 6 => index_step(&mut w, false, 1),
                        7 => {
                            if w.branch(w.cpu.p & 2 == 0) {
                                next = edge(9);
                            }
                        }
                        9 | 13 => {
                            let v = dp_x(&mut w, if step == 9 { 0xc0 } else { 0xbf });
                            w.load(v, 1);
                        }
                        10 => {
                            let v = w.dp_read(0x1fe2, 1);
                            compare(&mut w, v, false, 1);
                        }
                        11 => {
                            if w.branch(w.cpu.p & 1 == 0) {
                                next = edge(2);
                            }
                        }
                        12 => {
                            if w.branch(w.cpu.p & 2 == 0) {
                                next = Phase::Slope { right, step: 0 };
                            }
                        }
                        14 => w.dp_store(0x1fe1, w.cpu.a & 255, 1),
                        15 => w.dp_store(0x1fe9, w.cpu.x, 1),
                        16 => {
                            w.branch(true);
                            next = edge(0);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Phase::Slope { right, step } => {
                let delta = if right { 9 } else { 0 };
                let slope = |step| Phase::Slope { right, step };
                match step {
                    0 | 2 => w.dp_store(
                        if step == 0 { 0x1fe4 } else { 0x1fe3 } + delta,
                        w.cpu.a & 255,
                        1,
                    ),
                    1 => {
                        let v = dp_x(&mut w, 0xbf);
                        w.load(v, 1);
                    }
                    3 => w.dp_store(0x1fe9 + delta, w.cpu.x, 1),
                    4 => {
                        w.drain(8);
                        w.cpu.x = 0;
                        w.flags(0, 1);
                    }
                    5 | 15 | 29 => {
                        w.drain(6);
                        w.cpu.p |= 1;
                    }
                    6 | 17 => {
                        let v = w.dp_read(if step == 6 { 0x1fe1 } else { 0x1fe2 } + delta, 1);
                        w.sbc(v, 1);
                    }
                    7 => {
                        if w.branch(w.cpu.p & 1 != 0) {
                            next = slope(11);
                        }
                    }
                    8 => index_step(&mut w, false, 1),
                    9 => {
                        w.drain(8);
                        let v = (w.cpu.a ^ 255) & 255;
                        w.load(v, 1);
                    }
                    10 => w.increment_a(1),
                    11 => {
                        w.drain(6);
                        w.cpu.p |= 4;
                    }
                    12 | 14 | 18 => {
                        w.drain(24);
                        w.write_io(
                            match step {
                                12 => 0x4205,
                                14 => 0x4204,
                                _ => 0x4206,
                            },
                            w.cpu.a & 255,
                            1,
                        );
                    }
                    13 => w.immediate(0, 1),
                    16 => {
                        let v = w.dp_read(0x1fe4 + delta, 1);
                        w.load(v, 1);
                    }
                    19 => w.width(0x20, true),
                    20 => {
                        let v = w.dp_read(0x1fe0 + delta, 2);
                        w.load(v, 2);
                    }
                    21 => w.mask(0xff00, 2),
                    22 => {
                        w.drain(16);
                        let v = w.cpu.a | 0x80;
                        w.load(v, 2);
                    }
                    23 | 32 => {
                        w.dp_store(if step == 23 { 0x1fe5 } else { 0x1fe7 } + delta, w.cpu.a, 2)
                    }
                    24 => w.immediate(0, 2),
                    25 => w.compare_x_immediate(0, 1),
                    26 => {
                        if w.branch(w.cpu.p & 2 == 0) {
                            next = slope(29);
                        }
                    }
                    27 | 30 => {
                        w.drain(24);
                        let v = w.read_io(0x4214, 2);
                        if step == 27 {
                            w.load(v, 2);
                        } else {
                            w.sbc(v, 2);
                        }
                    }
                    28 => {
                        w.branch(true);
                        next = slope(31);
                    }
                    31 => {
                        w.drain(6);
                        w.cpu.p &= !4;
                    }
                    33 => w.width(0x20, false),
                    34 => w.clear_carry(),
                    35 => {
                        w.rts();
                        next = Phase::Body(self.edge_return);
                    }
                    _ => unreachable!(),
                }
            }
            Phase::Span(i) => match i {
                0 | 4 | 8 | 11 | 21 | 49 => {
                    let v = w.dp_read(if i == 0 || i == 8 { 0x1fe6 } else { 0x1fef }, width);
                    w.load(v, width);
                }
                1 | 5 | 9 | 12 | 22 | 50 => w.mask(if i == 1 || i == 5 { 7 } else { 0x38 }, width),
                2 | 6 | 23 | 24 | 51 | 52 => w.shift_left(width),
                3 | 26 | 54 | 68 | 77 => {
                    let v = w.cpu.a;
                    transfer_index(&mut w, false, v);
                }
                7 => {
                    let v = w.cpu.a;
                    transfer_index(&mut w, true, v);
                }
                10 => w.dp_store(0x1fbc, w.cpu.a & 255, 1),
                13 | 65 => {
                    w.drain(6);
                    w.cpu.p |= 1;
                }
                14 => {
                    let v = w.dp_read(0x1fbc, 1);
                    w.sbc(v, 1);
                }
                15 => {
                    if w.branch(w.cpu.p & 2 == 0) {
                        next = Phase::Span(39);
                    }
                }
                16 | 45 => w.width(0x30, true),
                17 | 46 | 80 => {
                    let v = mask_read(&mut w, if i == 80 { 0x09fe94 } else { 0x09fea4 });
                    w.load(v, 2);
                }
                18 | 48 => {
                    let v = w.cpu.y;
                    transfer_index(&mut w, true, v);
                }
                19 => {
                    let v = mask_read(&mut w, 0x09fe94);
                    let v = w.cpu.a & v;
                    w.load(v, 2);
                }
                20 | 47 | 81 => w.dp_store(0x1fb2, w.cpu.a, 2),
                25 | 53 => {
                    let v = w.dp_read(0x1fb9, 2);
                    let v = w.cpu.a | v;
                    w.load(v, 2);
                }
                27 | 32 | 55 | 60 | 71 | 73 | 82 | 87 => {
                    let v = w.dp_read(
                        if matches!(i, 32 | 60 | 73 | 87) {
                            0x1fb7
                        } else {
                            0x1fb5
                        },
                        2,
                    );
                    w.load(v, 2);
                }
                28 | 30 | 33 | 35 | 56 | 58 | 61 | 63 | 83 | 85 | 88 | 90 => {
                    let v = indexed_read(
                        &mut w,
                        if matches!(i, 33 | 35 | 61 | 63 | 88 | 90) {
                            16
                        } else {
                            0
                        },
                        2,
                    );
                    let v = w.cpu.a ^ v;
                    w.load(v, 2);
                }
                29 | 34 | 57 | 62 | 84 | 89 => {
                    let v = w.dp_read(0x1fb2, 2);
                    let v = w.cpu.a & v;
                    w.load(v, 2);
                }
                31 | 36 | 59 | 64 | 72 | 74 | 86 | 91 => indexed_store(
                    &mut w,
                    if matches!(i, 36 | 64 | 74 | 91) {
                        16
                    } else {
                        0
                    },
                ),
                37 | 92 => w.width(0x30, false),
                38 | 93 => {
                    w.rts();
                    next = Phase::Body(49);
                }
                39 => {
                    if w.branch(w.cpu.p & 1 == 0) {
                        next = Phase::Span(37);
                    }
                }
                40..=42 => lsr(&mut w, 1),
                43 => w.dp_store(0x1ffa, w.cpu.a & 255, 1),
                44 => w.dp_store(0x1ffb, 0, 1),
                66 | 75 => {
                    w.drain(6);
                    let v = w.cpu.y;
                    w.load(v, 2);
                }
                67 | 76 => {
                    w.drain(16);
                    w.sbc(0x20, 2);
                }
                69 | 78 => w.dp_increment(0x1ffa, 2, false),
                70 => {
                    if w.branch(w.cpu.p & 2 != 0) {
                        next = Phase::Span(80);
                    }
                }
                79 => {
                    if w.branch(w.cpu.p & 2 == 0) {
                        next = Phase::Span(71);
                    }
                }
                _ => unreachable!("span step {i}"),
            },
            Phase::Complete => unreachable!(),
        }
        // Return instructions have already pulled a real address. Verify it
        // rather than synthesizing a return from the continuation phase.
        if matches!(
            self.phase,
            Phase::Edge { step: 3, .. } | Phase::Slope { step: 35, .. } | Phase::Span(38 | 93)
        ) {
            self.phase = next;
            assert_eq!(w.cpu.pc, self.pc());
        } else {
            self.phase = next;
            w.cpu.pc = self.pc();
        }
        Some(w.cycles)
    }
}
fn jsr<B: WorkBus>(w: &mut Work<'_, B>) {
    w.drain(16);
    w.drain(6);
    w.push(w.cpu.pc + 2, 2);
}
fn dp_x<B: WorkBus>(w: &mut Work<'_, B>, offset: u16) -> u16 {
    w.drain(8);
    w.drain(6);
    w.read(u32::from(0x1f00 + offset + w.cpu.x), 1)
}
fn compare<B: WorkBus>(w: &mut Work<'_, B>, rhs: u16, x: bool, width: u8) {
    let mask = if width == 1 { 255 } else { 65535 };
    let lhs = (if x { w.cpu.x } else { w.cpu.a }) & mask;
    w.cpu.p = (w.cpu.p & !1) | u8::from(lhs >= rhs);
    w.flags(lhs.wrapping_sub(rhs) & mask, width);
}
fn transfer_index<B: WorkBus>(w: &mut Work<'_, B>, x: bool, value: u16) {
    w.drain(6);
    let width = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
    let value = value & if width == 1 { 255 } else { 65535 };
    if x {
        w.cpu.x = value;
    } else {
        w.cpu.y = value;
    }
    w.flags(value, width);
}
fn index_step<B: WorkBus>(w: &mut Work<'_, B>, increment: bool, width: u8) {
    w.drain(6);
    w.cpu.x = (if increment {
        w.cpu.x.wrapping_add(1)
    } else {
        w.cpu.x.wrapping_sub(1)
    }) & if width == 1 { 255 } else { 65535 };
    w.flags(w.cpu.x, width);
}
fn lsr<B: WorkBus>(w: &mut Work<'_, B>, width: u8) {
    w.drain(6);
    let v = w.cpu.a & if width == 1 { 255 } else { 65535 };
    w.cpu.p = (w.cpu.p & !1) | (v as u8 & 1);
    w.load(v >> 1, width);
}
fn bit_immediate<B: WorkBus>(w: &mut Work<'_, B>, mask: u16) {
    w.drain(8);
    w.cpu.p = (w.cpu.p & !2) | if w.cpu.a & mask == 0 { 2 } else { 0 };
}
fn indexed_read<B: WorkBus>(w: &mut Work<'_, B>, base: u16, width: u8) -> u16 {
    let at = base.wrapping_add(w.cpu.y);
    w.drain(16);
    if w.cpu.p & 0x10 == 0 || base & 0xff00 != at & 0xff00 {
        w.drain(6);
    }
    w.read(w.absolute(at), width)
}
fn indexed_store<B: WorkBus>(w: &mut Work<'_, B>, base: u16) {
    w.drain(16);
    w.drain(6);
    w.write(w.absolute(base.wrapping_add(w.cpu.y)), w.cpu.a, 2);
}
fn mask_read<B: WorkBus>(w: &mut Work<'_, B>, base: u32) -> u16 {
    w.drain(24);
    w.read(base + u32::from(w.cpu.x), 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cycle_models::poly_vertices::PolyDivider;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };

    #[test]
    fn raster_preserves_every_bus_access_partial_bitmap_and_math_register() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        // Narrow/wide spans, both slope signs, flat edges, point/line degeneracy,
        // row remapping, and opposite winding all start with the same live call.
        for points in [
            vec![(2, 2), (5, 2), (5, 6), (2, 6)],
            vec![(1, 1), (62, 1), (62, 62), (1, 62)],
            vec![(32, 0), (63, 32), (32, 63), (0, 32)],
            vec![(2, 4), (3, 4), (4, 4)],
            vec![(20, 20), (20, 20), (20, 20)],
            vec![(1, 2), (61, 31), (3, 62)],
        ] {
            for reverse in [false, true] {
                for status in [0x30, 0x01] {
                    let mut cpu = NativeNmiContext {
                        a: 0xab34,
                        x: 0x13,
                        y: 0x15,
                        d: 0x1f00,
                        db: 9,
                        bank: 9,
                        pc: 0xfd1e,
                        sp: 0x1f30,
                        p: status,
                    };
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, b) in machine.ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 8) ^ 0x5a) as u8;
                    }
                    machine.ram[0x1f31..0x1f34].copy_from_slice(&[0x90, 0xfa, 9]);
                    machine.ram[0x1fc0] = (points.len() * 2) as u8;
                    let mut polygon = points.clone();
                    if reverse {
                        polygon.reverse();
                    }
                    for (i, (x, y)) in polygon.iter().enumerate() {
                        machine.ram[0x1fc1 + i * 2] = *x;
                        machine.ram[0x1fc2 + i * 2] = *y;
                    }
                    machine.ram[0x1fb5..0x1fb9].copy_from_slice(&[0xff, 0, 0, 0xff]);
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.dp = cpu.d;
                    machine.cpu.db = cpu.db;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let mut clock = CpuMasterTimeline::at_raster(
                        0,
                        CpuRasterPosition::new(224, 1300),
                        CpuBusWorkload::default(),
                        CpuFieldTiming::NON_INTERLACE_EVEN,
                    );
                    let mut source = RomCpuTimingProbe::new(
                        machine,
                        clock.clone(),
                        SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    clock.begin_synchronous_timeline().unwrap();
                    let mut raster = PolyRaster::default();
                    let mut divider = PolyDivider::default();
                    let mut instructions = 0;
                    while !raster.complete() {
                        assert_eq!(raster.defers_irq_mask_change(),
                            matches!(rom[9 * 0x8000 + (cpu.pc as usize & 0x7fff)], 0x58 | 0x78));
                        let label =
                            format!("points={polygon:?} status={status:02x} {:?}", raster.phase);
                        crate::cycle_models::native_work::compare_instruction_with_devices(
                            &label,
                            &mut cpu,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            None,
                            |_, access| divider.access(access),
                            |cpu, clock, bus| {
                                raster.step(cpu, clock, bus).unwrap();
                            },
                        );
                        assert_eq!(ram, source.snes().ram, "{label}");
                        assert_eq!(
                            (divider.dividend, divider.quotient, divider.remainder),
                            (
                                source.snes().divide_a,
                                source.snes().divide_result,
                                source.snes().multiply_result
                            ),
                            "{label}"
                        );
                        raster = raster.clone();
                        divider = divider.clone();
                        instructions += 1;
                        assert!(instructions < 30000, "{label}");
                    }
                    assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.p),
                        (9, 0xfa91, 0x1f33, 9, status)
                    );
                }
            }
        }
    }
}

// Source addresses document and assert each retained phase; no PC dispatch.
const BODY_PC: [u16; 94] = [
    0xfd1e, 0xfd1f, 0xfd20, 0xfd22, 0xfd24, 0xfd25, 0xfd26, 0xfd28, 0xfd29, 0xfd2b, 0xfd2c, 0xfd2d,
    0xfd2f, 0xfd31, 0xfd33, 0xfd35, 0xfd36, 0xfd38, 0xfd3a, 0xfd3c, 0xfd3d, 0xfd3f, 0xfd42, 0xfd44,
    0xfd46, 0xfd48, 0xfd4a, 0xfd4b, 0xfd4c, 0xfd4e, 0xfd50, 0xfd52, 0xfd54, 0xfd56, 0xfd57, 0xfd59,
    0xfd5c, 0xfd5e, 0xfd60, 0xfd63, 0xfd65, 0xfd67, 0xfd6a, 0xfd6c, 0xfd6f, 0xfd71, 0xfd72, 0xfd73,
    0xfd74, 0xfd77, 0xfd79, 0xfd7a, 0xfd7b, 0xfd7d, 0xfd7f, 0xfd81, 0xfd83, 0xfd85, 0xfd86, 0xfd87,
    0xfd89, 0xfd8b, 0xfd8d, 0xfd8f, 0xfd91, 0xfd93, 0xfd95, 0xfd97, 0xfd99, 0xfd9b, 0xfd9e, 0xfda0,
    0xfda2, 0xfda3, 0xfda5, 0xfda7, 0xfda9, 0xfdab, 0xfdad, 0xfdaf, 0xfdb2, 0xfdb4, 0xfdb6, 0xfdb7,
    0xfdb9, 0xfdbb, 0xfdbd, 0xfdbf, 0xfdc1, 0xfdc2, 0xfdc4, 0xfdc6, 0xfdc8, 0xfdca,
];
const SPAN_PC: [u16; 94] = [
    0xfdcf, 0xfdd1, 0xfdd3, 0xfdd4, 0xfdd5, 0xfdd7, 0xfdd9, 0xfdda, 0xfddb, 0xfddd, 0xfddf, 0xfde1,
    0xfde3, 0xfde5, 0xfde6, 0xfde8, 0xfdea, 0xfdec, 0xfdf0, 0xfdf1, 0xfdf5, 0xfdf7, 0xfdf9, 0xfdfc,
    0xfdfd, 0xfdfe, 0xfe00, 0xfe01, 0xfe03, 0xfe06, 0xfe08, 0xfe0b, 0xfe0e, 0xfe10, 0xfe13, 0xfe15,
    0xfe18, 0xfe1b, 0xfe1d, 0xfe1e, 0xfe20, 0xfe21, 0xfe22, 0xfe23, 0xfe25, 0xfe27, 0xfe29, 0xfe2d,
    0xfe2f, 0xfe30, 0xfe32, 0xfe35, 0xfe36, 0xfe37, 0xfe39, 0xfe3a, 0xfe3c, 0xfe3f, 0xfe41, 0xfe44,
    0xfe47, 0xfe49, 0xfe4c, 0xfe4e, 0xfe51, 0xfe54, 0xfe55, 0xfe56, 0xfe59, 0xfe5a, 0xfe5c, 0xfe5e,
    0xfe60, 0xfe63, 0xfe65, 0xfe68, 0xfe69, 0xfe6c, 0xfe6d, 0xfe6f, 0xfe71, 0xfe75, 0xfe77, 0xfe79,
    0xfe7c, 0xfe7e, 0xfe81, 0xfe84, 0xfe86, 0xfe89, 0xfe8b, 0xfe8e, 0xfe91, 0xfe93,
];
const LEFT_PC: [u16; 17] = [
    0xfeb4, 0xfeb6, 0xfeb8, 0xfeb9, 0xfeba, 0xfebc, 0xfebd, 0xfebe, 0xfec0, 0xfec2, 0xfec4, 0xfec6,
    0xfec8, 0xfeca, 0xfecc, 0xfece, 0xfed0,
];
const RIGHT_PC: [u16; 19] = [
    0xff1e, 0xff20, 0xff22, 0xff23, 0xff24, 0xff26, 0xff28, 0xff2a, 0xff2c, 0xff2e, 0xff2f, 0xff30,
    0xff32, 0xff34, 0xff36, 0xff38, 0xff3a, 0xff3c, 0xff3e,
];
const SLOPE_PC: [u16; 36] = [
    0xfed2, 0xfed4, 0xfed6, 0xfed8, 0xfeda, 0xfedc, 0xfedd, 0xfedf, 0xfee1, 0xfee2, 0xfee4, 0xfee5,
    0xfee6, 0xfeea, 0xfeec, 0xfef0, 0xfef1, 0xfef3, 0xfef5, 0xfef9, 0xfefb, 0xfefd, 0xff00, 0xff03,
    0xff05, 0xff08, 0xff0a, 0xff0c, 0xff10, 0xff12, 0xff13, 0xff17, 0xff18, 0xff1a, 0xff1c, 0xff1d,
];
