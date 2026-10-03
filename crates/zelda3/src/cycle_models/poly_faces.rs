//! Retained face extraction, winding and color selection. Raster work is a
//! separate owner; reaching it leaves the live JSL frame and face cursor intact.
use super::{native_work::{Work, WorkBus}, nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Call,
    Setup(u8),
    Face(u8),
    Coordinate(u8),
    Visibility(u8),
    Orientation(u8),
    Winding(u8),
    Reverse(u8),
    Color {
        back: bool,
        step: u8,
    },
    Shade {
        back: bool,
        step: u8,
    },
    Palette {
        back: bool,
        step: u8,
    },
    ColorReturn {
        back: bool,
    },
    RasterCall {
        back: bool,
    },
    RasterPending {
        back: bool,
    },
    ReversedRasterReturn,
    Finish(u8),
    Complete,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PolyFaces {
    phase: Phase,
}
impl PolyFaces {
    pub(super) fn defers_irq_mask_change(&self) -> bool {
        matches!(self.phase, Phase::Orientation(14 | 33)) // CLI
    }
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn raster_pending(&self) -> bool {
        matches!(self.phase, Phase::RasterPending { .. })
    }
    /// Called only after the retained raster owner has executed its real RTL.
    pub(crate) fn resume_after_raster(&mut self, cpu: &NativeNmiContext) {
        let Phase::RasterPending { back } = self.phase else {
            panic!("no pending raster call")
        };
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d),
            (9, if back { 0xfac7 } else { 0xfa91 }, 9, 0x1f00)
        );
        self.phase = if back {
            Phase::ReversedRasterReturn
        } else {
            Phase::Finish(0)
        };
    }
    fn pc(&self) -> u16 {
        match self.phase {
            Phase::Call => 0xf832,
            Phase::Setup(i) => [0xfa4f, 0xfa51][usize::from(i)],
            Phase::Face(i) => [
                0xfa53, 0xfa55, 0xfa57, 0xfa59, 0xfa5b, 0xfa5c, 0xfa5e, 0xfa5f,
            ][usize::from(i)],
            Phase::Coordinate(i) => [
                0xfa61, 0xfa62, 0xfa64, 0xfa65, 0xfa68, 0xfa6a, 0xfa6b, 0xfa6e, 0xfa70, 0xfa71,
                0xfa72, 0xfa73, 0xfa75,
            ][usize::from(i)],
            Phase::Visibility(i) => [
                0xfa77, 0xfa79, 0xfa7a, 0xfa7c, 0xfa7d, 0xfa7f, 0xfa81, 0xfa83, 0xfa86, 0xfa88,
                0xfa8a,
            ][usize::from(i)],
            Phase::Orientation(i) => [
                0xfb24, 0xfb26, 0xfb28, 0xfb2a, 0xfb2d, 0xfb2f, 0xfb31, 0xfb34, 0xfb35, 0xfb37,
                0xfb39, 0xfb3c, 0xfb3f, 0xfb41, 0xfb44, 0xfb45, 0xfb47, 0xfb49, 0xfb4b, 0xfb4d,
                0xfb50, 0xfb52, 0xfb54, 0xfb57, 0xfb58, 0xfb5a, 0xfb5c, 0xfb5f, 0xfb61, 0xfb62,
                0xfb64, 0xfb67, 0xfb69, 0xfb6b, 0xfb6c,
            ][usize::from(i)],
            Phase::Winding(i) => [0xfa97, 0xfa99][usize::from(i)],
            Phase::Reverse(i) => [
                0xfa9b, 0xfa9d, 0xfa9f, 0xfaa2, 0xfaa3, 0xfaa4, 0xfaa6, 0xfaa7, 0xfaa8, 0xfaaa,
                0xfaac, 0xfaad, 0xfab0, 0xfab2, 0xfab3, 0xfab6, 0xfab7, 0xfab8, 0xfab9, 0xfaba,
                0xfabc, 0xfabe, 0xfac0,
            ][usize::from(i)],
            Phase::Color { back, step } => {
                if back {
                    [
                        0xfad7, 0xfad9, 0xfadb, 0xfadd, 0xfade, 0xfadf, 0xfae0, 0xfae1, 0xfae3,
                    ][usize::from(step)]
                } else {
                    [0xfaca, 0xfacc, 0xface, 0xfad0, 0xfad2][usize::from(step)]
                }
            }
            Phase::Shade { back, step } => {
                if step < 5 {
                    if back {
                        [0xfae8, 0xfaea, 0xfaec, 0xfaef, 0xfaf0][usize::from(step)]
                    } else {
                        [0xfaf2, 0xfaf4][usize::from(step)]
                    }
                } else {
                    [
                        0xfaf6, 0xfaf7, 0xfafa, 0xfafd, 0xfaff, 0xfb02, 0xfb05, 0xfb06, 0xfb07,
                        0xfb08, 0xfb09, 0xfb0a, 0xfb0b, 0xfb0c, 0xfb0d, 0xfb0e, 0xfb10, 0xfb12,
                        0xfb13, 0xfb15, 0xfb17, 0xfb19, 0xfb1b, 0xfb1d, 0xfb1f,
                    ][usize::from(step - 5)]
                }
            }
            Phase::Palette { step, .. } => [
                0xfcae, 0xfcaf, 0xfcb1, 0xfcb2, 0xfcb3, 0xfcb4, 0xfcb6, 0xfcba, 0xfcbc, 0xfcc0,
                0xfcc2, 0xfcc3,
            ][usize::from(step)],
            // The actual return PC is pulled from native stack; direct and
            // shaded paths each have their own RTS address.
            Phase::ColorReturn { .. } => unreachable!("return address belongs to live stack"),
            Phase::RasterCall { back } => {
                if back {
                    0xfac3
                } else {
                    0xfa8d
                }
            }
            Phase::RasterPending { .. } => 0xfd1e,
            Phase::ReversedRasterReturn => 0xfac7,
            Phase::Finish(i) => [0xfa91, 0xfa92, 0xfa94, 0xfa96][usize::from(i)],
            Phase::Complete => 0xf835,
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete() || self.raster_pending() {
            return None;
        }
        if matches!(self.phase, Phase::ColorReturn { .. }) {
            assert!(matches!(cpu.pc, 0xfad6 | 0xfae7 | 0xfb23));
        } else {
            assert_eq!(cpu.pc, self.pc());
        }
        assert_eq!((cpu.bank, cpu.d, cpu.db), (9, 0x1f00, 9));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.phase = match self.phase {
            Phase::Call => {
                jsr(&mut w);
                Phase::Setup(0)
            }
            Phase::Setup(i) => {
                if i == 0 {
                    w.width(0x30, false);
                } else {
                    w.drain(8);
                    w.cpu.y = 0;
                    w.flags(0, 1);
                }
                if i == 0 {
                    Phase::Setup(1)
                } else {
                    Phase::Face(0)
                }
            }
            Phase::Face(i) => {
                match i {
                    0 => face_read(&mut w),
                    1 | 3 | 5 => w.dp_store(
                        match i {
                            1 => 0x1f4e,
                            3 => 0x1fb0,
                            _ => 0x1fc0,
                        },
                        w.cpu.a & 255,
                        1,
                    ),
                    2 => w.mask(127, 1),
                    4 => w.shift_left(1),
                    6 => index_step(&mut w, false, true),
                    7 => {
                        w.drain(8);
                        w.cpu.x = 1;
                        w.flags(1, 1);
                    }
                    _ => unreachable!(),
                }
                if i == 7 {
                    Phase::Coordinate(0)
                } else {
                    Phase::Face(i + 1)
                }
            }
            Phase::Coordinate(i) => {
                match i {
                    0 => {
                        w.drain(6);
                        w.push(w.cpu.y, 1);
                    }
                    1 => face_read(&mut w),
                    2 => {
                        w.drain(6);
                        w.cpu.y = w.cpu.a & 255;
                        w.flags(w.cpu.y, 1);
                    }
                    3 | 6 => {
                        let v = indexed_read(&mut w, if i == 3 { 0x1f60 } else { 0x1f88 }, 1);
                        w.load(v, 1);
                    }
                    4 | 7 => indexed_store(&mut w, 1),
                    5 | 8 => index_step(&mut w, true, true),
                    9 => {
                        w.drain(12);
                        w.cpu.y = w.pull(1);
                        w.flags(w.cpu.y, 1);
                    }
                    10 => index_step(&mut w, false, true),
                    11 => w.dp_increment(0x1fb0, 1, false),
                    12 => {
                        let again = w.branch(w.cpu.p & 2 == 0);
                        self.phase = if again {
                            Phase::Coordinate(0)
                        } else {
                            Phase::Visibility(0)
                        };
                        w.cpu.pc = self.pc();
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Phase::Coordinate(i + 1)
            }
            Phase::Visibility(i) => {
                let mut next = Phase::Visibility(i + 1);
                match i {
                    0 => face_read(&mut w),
                    1 => index_step(&mut w, false, true),
                    2 => w.dp_store(0x1f4f, w.cpu.a & 255, 1),
                    3 => {
                        w.drain(6);
                        w.push(w.cpu.y, 1);
                    }
                    4 => {
                        let v = w.dp_read(0x1fc0, 1);
                        w.load(v, 1);
                    }
                    5 => w.compare_immediate(6, 1),
                    6 => {
                        if w.branch(w.cpu.p & 1 == 0) {
                            next = Phase::Finish(0);
                        }
                    }
                    7 => {
                        jsr(&mut w);
                        next = Phase::Orientation(0);
                    }
                    8 => {
                        if w.branch(w.cpu.p & 0x80 != 0) {
                            next = Phase::Winding(0);
                        }
                    }
                    9 => {
                        if w.branch(w.cpu.p & 2 != 0) {
                            next = Phase::Finish(0);
                        }
                    }
                    10 => {
                        jsr(&mut w);
                        next = Phase::Color {
                            back: false,
                            step: 0,
                        };
                    }
                    _ => unreachable!(),
                }
                next
            }
            Phase::Orientation(i) => {
                match i {
                    0 | 16 => w.width(5, false),
                    1 | 8 | 17 | 24 | 29 => {
                        let v = w.dp_read(
                            match i {
                                1 | 24 => 0x1fc3 + u16::from(i == 24),
                                8 => 0x1fc6,
                                17 => 0x1fc5,
                                _ => 0x1fb0,
                            },
                            if i == 29 { 2 } else { 1 },
                        );
                        w.load(v, if i == 29 { 2 } else { 1 });
                    }
                    2 | 9 | 18 | 25 => {
                        let v = w.dp_read(
                            match i {
                                2 => 0x1fc1,
                                9 => 0x1fc4,
                                18 => 0x1fc3,
                                _ => 0x1fc2,
                            },
                            1,
                        );
                        w.sbc(v, 1);
                    }
                    3 | 6 | 10 | 19 | 22 | 26 => {
                        w.drain(16);
                        w.write_io(
                            if matches!(i, 10 | 26) { 0x211c } else { 0x211b },
                            w.cpu.a & 255,
                            1,
                        );
                    }
                    4 | 20 => w.immediate(0, 1),
                    5 | 21 => {
                        w.drain(8);
                        w.sbc(0, 1);
                    }
                    7 | 23 | 28 => {
                        w.drain(6);
                        w.cpu.p |= 1;
                    }
                    11 | 13 => {
                        w.drain(16);
                        let v = w.read_io(if i == 11 { 0x2134 } else { 0x2135 }, 1);
                        w.load(v, 1);
                    }
                    12 | 15 => w.dp_store(if i == 12 { 0x1fb0 } else { 0x1fb1 }, w.cpu.a & 255, 1),
                    14 | 33 => {
                        w.drain(6);
                        w.cpu.p &= !4;
                    }
                    27 => w.width(0x20, true),
                    30 => {
                        w.drain(16);
                        let v = w.read_io(0x2134, 2);
                        w.sbc(v, 2);
                    }
                    31 => w.dp_store(0x1fb0, w.cpu.a, 2),
                    32 => w.width(0x20, false),
                    34 => {
                        w.rts();
                        assert_eq!(w.cpu.pc, 0xfa86);
                    }
                    _ => unreachable!(),
                }
                if i == 34 {
                    Phase::Visibility(8)
                } else {
                    Phase::Orientation(i + 1)
                }
            }
            Phase::Winding(i) => {
                if i == 0 {
                    let v = w.dp_read(0x1f4e, 1);
                    w.load(v, 1);
                    Phase::Winding(1)
                } else if w.branch(w.cpu.p & 0x80 == 0) {
                    Phase::Finish(0)
                } else {
                    Phase::Reverse(0)
                }
            }
            Phase::Reverse(i) => {
                let mut next = Phase::Reverse(i + 1);
                match i {
                    0 => w.width(0x20, true),
                    1 => {
                        let v = w.dp_read(0x1fc0, 2);
                        w.load(v, 2);
                    }
                    2 => w.mask(255, 2),
                    3 => {
                        w.drain(6);
                        w.cpu.y = w.cpu.a & 255;
                        w.flags(w.cpu.y, 1);
                    }
                    4 | 17 | 18 => index_step(&mut w, false, false),
                    5 => {
                        w.drain(8);
                        w.cpu.x = 1;
                        w.flags(1, 1);
                    }
                    6 | 7 => lsr(&mut w, 2),
                    8 => w.dp_store(0x1fb0, w.cpu.a, 2),
                    9 => {
                        w.drain(8);
                        w.drain(6);
                        let v = w.read(u32::from(0x1fc0 + w.cpu.x), 2);
                        w.load(v, 2);
                    }
                    10 => {
                        w.drain(6);
                        w.push(w.cpu.a, 2);
                    }
                    11 => {
                        let v = indexed_read(&mut w, 0x1fc0, 2);
                        w.load(v, 2);
                    }
                    12 => indexed_store(&mut w, 2),
                    13 => {
                        w.drain(12);
                        let v = w.pull(2);
                        w.load(v, 2);
                    }
                    14 => {
                        w.drain(16);
                        w.drain(6);
                        w.write(w.absolute(0x1fc0 + w.cpu.y), w.cpu.a, 2);
                    }
                    15 | 16 => index_step(&mut w, true, true),
                    19 => w.dp_increment(0x1fb0, 2, false),
                    20 => {
                        if w.branch(w.cpu.p & 2 == 0) {
                            next = Phase::Reverse(9);
                        }
                    }
                    21 => w.width(0x20, false),
                    22 => {
                        jsr(&mut w);
                        next = Phase::Color {
                            back: true,
                            step: 0,
                        };
                    }
                    _ => unreachable!(),
                }
                next
            }
            Phase::Color { back, step } => {
                let mut next = Phase::Color {
                    back,
                    step: step + 1,
                };
                match step {
                    0 | 2 => {
                        let v = w.dp_read(if step == 0 { 0x1f01 } else { 0x1f4f }, 1);
                        w.load(v, 1);
                    }
                    1 => {
                        if w.branch(w.cpu.p & 2 == 0) {
                            next = Phase::Shade { back, step: 0 };
                        }
                    }
                    3..=6 if back => lsr(&mut w, 1),
                    3 | 7 => w.mask(7, 1),
                    4 | 8 => {
                        jsl(&mut w);
                        next = Phase::Palette { back, step: 0 };
                    }
                    _ => unreachable!(),
                }
                next
            }
            Phase::Shade { back, step } => {
                let mut next = step + 1;
                match step {
                    0 => w.width(0x20, true),
                    1 => {
                        let v = w.dp_read(0x1fb0, 2);
                        w.load(v, 2);
                        if !back {
                            next = 5;
                        }
                    }
                    2 => {
                        w.drain(16);
                        w.load(!w.cpu.a, 2);
                    }
                    3 => w.increment_a(2),
                    4 => {
                        w.branch(true);
                    }
                    5 => {
                        w.drain(6);
                        w.push(w.cpu.a, 2);
                    }
                    6 | 9 => {
                        w.drain(16);
                        let v = w.read(w.absolute(if step == 6 { 0x1f03 } else { 0x1f02 }), 2);
                        w.load(v, 2);
                    }
                    7 | 10 => w.mask(255, 2),
                    8 => {
                        if w.branch(w.cpu.p & 2 != 0) {
                            next = 16;
                        }
                    }
                    11..=15 => lsr(&mut w, 2),
                    16 => {
                        w.drain(6);
                        w.cpu.x = w.cpu.a & 255;
                        w.flags(w.cpu.x, 1);
                    }
                    17 => {
                        w.drain(12);
                        let v = w.pull(2);
                        w.load(v, 2);
                    }
                    18 => w.shift_left(2),
                    19 => index_step(&mut w, true, false),
                    20 => {
                        if w.branch(w.cpu.p & 0x80 == 0) {
                            next = 18;
                        }
                    }
                    21 => w.width(0x20, false),
                    22 => w.swap_bytes(),
                    23 => {
                        if w.branch(w.cpu.p & 2 == 0) {
                            next = 26;
                        }
                    }
                    24 => w.immediate(1, 1),
                    25 => {
                        w.branch(true);
                        next = 29;
                    }
                    26 => w.compare_immediate(8, 1),
                    27 => {
                        if w.branch(w.cpu.p & 1 == 0) {
                            next = 29;
                        }
                    }
                    28 => w.immediate(7, 1),
                    29 => {
                        jsl(&mut w);
                        self.phase = Phase::Palette { back, step: 0 };
                        w.cpu.pc = self.pc();
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Phase::Shade { back, step: next }
            }
            Phase::Palette { back, step } => {
                match step {
                    0 => {
                        w.drain(6);
                        w.push(u16::from(w.cpu.p), 1);
                    }
                    1 => w.width(0x30, false),
                    2 | 3 => w.shift_left(1),
                    4 => {
                        w.drain(6);
                        w.cpu.x = w.cpu.a & 255;
                        w.flags(w.cpu.x, 1);
                    }
                    5 => w.width(0x20, true),
                    6 | 8 => {
                        w.drain(24);
                        let v = w.read(
                            (if step == 6 { 0x09fcc4 } else { 0x09fcc6 }) + u32::from(w.cpu.x),
                            2,
                        );
                        w.load(v, 2);
                    }
                    7 | 9 => w.dp_store(if step == 7 { 0x1fb5 } else { 0x1fb7 }, w.cpu.a, 2),
                    10 => {
                        w.drain(12);
                        w.cpu.p = w.pull(1) as u8;
                        if w.cpu.p & 0x10 != 0 {
                            w.cpu.x &= 255;
                            w.cpu.y &= 255;
                        }
                    }
                    11 => {
                        w.rtl();
                        self.phase = Phase::ColorReturn { back };
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Phase::Palette {
                    back,
                    step: step + 1,
                }
            }
            Phase::ColorReturn { back } => {
                w.rts();
                assert_eq!(w.cpu.pc, if back { 0xfac3 } else { 0xfa8d });
                Phase::RasterCall { back }
            }
            Phase::RasterCall { back } => {
                jsl(&mut w);
                Phase::RasterPending { back }
            }
            Phase::ReversedRasterReturn => {
                w.drain(16);
                Phase::Finish(0)
            }
            Phase::Finish(i) => {
                match i {
                    0 => {
                        w.drain(12);
                        w.cpu.y = w.pull(1);
                        w.flags(w.cpu.y, 1);
                    }
                    1 => w.dp_increment(0x1f40, 1, false),
                    2 => {
                        let again = w.branch(w.cpu.p & 2 == 0);
                        self.phase = if again {
                            Phase::Face(0)
                        } else {
                            Phase::Finish(3)
                        };
                        w.cpu.pc = self.pc();
                        return Some(w.cycles);
                    }
                    3 => {
                        w.rts();
                        assert_eq!(w.cpu.pc, 0xf835);
                        self.phase = Phase::Complete;
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Phase::Finish(i + 1)
            }
            Phase::Complete | Phase::RasterPending { .. } => unreachable!(),
        };
        w.cpu.pc = self.pc();
        Some(w.cycles)
    }
}
fn jsr<B: WorkBus>(w: &mut Work<'_, B>) {
    w.drain(16);
    w.drain(6);
    w.push(w.cpu.pc + 2, 2);
}
fn jsl<B: WorkBus>(w: &mut Work<'_, B>) {
    w.drain(24);
    w.drain(6);
    w.push(u16::from(w.cpu.bank), 1);
    w.push(w.cpu.pc + 3, 2);
}
fn lsr<B: WorkBus>(w: &mut Work<'_, B>, width: u8) {
    w.drain(6);
    let v = if width == 1 { w.cpu.a & 255 } else { w.cpu.a };
    w.cpu.p = (w.cpu.p & !1) | (v as u8 & 1);
    w.load(v >> 1, width);
}
fn index_step<B: WorkBus>(w: &mut Work<'_, B>, x: bool, increment: bool) {
    w.drain(6);
    let old = if x { w.cpu.x } else { w.cpu.y };
    let v = (if increment {
        old.wrapping_add(1)
    } else {
        old.wrapping_sub(1)
    }) & 255;
    if x {
        w.cpu.x = v
    } else {
        w.cpu.y = v
    }
    w.flags(v, 1);
}
fn face_read<B: WorkBus>(w: &mut Work<'_, B>) {
    let ptr = w.dp_read(0x1f43, 2);
    let at = ptr.wrapping_add(w.cpu.y);
    if ptr & 0xff00 != at & 0xff00 {
        w.drain(6);
    }
    let v = w.read(w.absolute(at), 1);
    w.load(v, 1);
}
fn indexed_read<B: WorkBus>(w: &mut Work<'_, B>, base: u16, width: u8) -> u16 {
    let at = base.wrapping_add(w.cpu.y);
    w.drain(16);
    if base & 0xff00 != at & 0xff00 {
        w.drain(6);
    }
    w.read(w.absolute(at), width)
}
fn indexed_store<B: WorkBus>(w: &mut Work<'_, B>, width: u8) {
    w.drain(8);
    w.drain(6);
    w.write(
        u32::from(0x1fc0 + w.cpu.x),
        if width == 1 { w.cpu.a & 255 } else { w.cpu.a },
        width,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn faces_retain_culling_reverse_color_raster_and_next_face_state() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for (points, count) in [
            ([(0, 0), (10, 0), (10, 10), (0, 10)], 3),
            ([(0, 0), (0, 10), (10, 10), (10, 0)], 4),
            ([(0, 0), (10, 0), (20, 0), (30, 0)], 3),
            ([(0, 0), (100, 0), (100, 100), (0, 100)], 3),
            ([(0, 0), (16, 0), (16, 16), (0, 16)], 3),
            ([(0, 0), (10, 0), (10, 10), (0, 10)], 2),
        ] {
            for back in [false, true] {
                for mode in [0, 1] {
                    for model in [0, 1] {
                        let mut cpu = NativeNmiContext {
                            a: 0xab34,
                            x: 0,
                            y: 0,
                            d: 0x1f00,
                            db: 9,
                            bank: 9,
                            pc: 0xf832,
                            sp: 0x1f3e,
                            p: 0x71,
                        };
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        for (i, b) in machine.ram.iter_mut().enumerate() {
                            *b = (i ^ (i >> 8) ^ 0x5a) as u8;
                        }
                        machine.ram[0x1f01] = mode;
                        machine.ram[0x1f02] = 255;
                        machine.ram[0x1f03] = model;
                        machine.ram[0x1f40] = 2;
                        machine.ram[0x1f43..0x1f45].copy_from_slice(&0x1800u16.to_le_bytes());
                        let mut descriptor = vec![count | if back { 128 } else { 0 }];
                        descriptor.extend(0..count);
                        descriptor.push(0x65);
                        for face in 0..2 {
                            let at = 0x1800 + face * descriptor.len();
                            machine.ram[at..at + descriptor.len()].copy_from_slice(&descriptor);
                        }
                        for (i, (x, y)) in points.into_iter().enumerate() {
                            machine.ram[0x1f60 + i] = x;
                            machine.ram[0x1f88 + i] = y;
                        }
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
                        let mut ppu = machine.ppu.clone();
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
                        let mut faces = PolyFaces::default();
                        let mut raster = None;
                        let mut divider = super::super::poly_vertices::PolyDivider::default();
                        let mut instructions = 0;
                        while !faces.complete() {
                            if faces.raster_pending() && raster.is_none() {
                                raster = Some(super::super::poly_raster::PolyRaster::default());
                            }
                            let deferred = raster.as_ref().map_or_else(
                                || faces.defers_irq_mask_change(), |work| work.defers_irq_mask_change());
                            assert_eq!(deferred,
                                matches!(rom[9 * 0x8000 + (cpu.pc as usize & 0x7fff)], 0x58 | 0x78));
                            let label=format!("points={points:?} count={count} back={back} mode={mode} model={model} {:?}",faces.phase);
                            crate::cycle_models::native_work::compare_instruction_with_devices(
                                &label,
                                &mut cpu,
                                &mut clock,
                                &mut source,
                                &mut ram,
                                &rom,
                                Some(&mut ppu),
                                |_, access| divider.access(access),
                                |cpu, clock, bus| {
                                    if let Some(work) = raster.as_mut() {
                                        work.step(cpu, clock, bus).unwrap();
                                    } else {
                                        faces.step(cpu, clock, bus).unwrap();
                                    }
                                },
                            );
                            if raster.as_ref().is_some_and(|work| work.complete()) {
                                faces.resume_after_raster(&cpu);
                                raster = None;
                            }
                            assert_eq!(ram, source.snes().ram, "{label}");
                            assert_eq!(ppu.m7_matrix, source.snes().ppu.m7_matrix, "{label}");
                            assert_eq!(ppu.m7_prev, source.snes().ppu.m7_prev, "{label}");
                            faces = faces.clone();
                            raster = raster.clone();
                            divider = divider.clone();
                            instructions += 1;
                            assert!(instructions < 50000, "{label}");
                        }
                        assert!(faces.complete() || cpu.pc == 0xfd1e);
                        if faces.complete() {
                            assert_eq!(cpu.sp, 0x1f3e);
                            assert_eq!(ram[0x1f40], 0);
                        }
                    }
                }
            }
        }
    }
}
