//! Retained HUD refill/update caller. Shared inventory and tile routines keep
//! their own near returns, while refresh and heart animation retain nested calls.
use super::{
    load_file_hud::LoadFileHud,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HudUpdate {
    pc: u32,
    complete: bool,
    near_returns: Vec<u32>,
    far_returns: Vec<u32>,
    shared: Option<LoadFileHud>,
    shared_return: Option<(u32, u16)>,
}
impl Default for HudUpdate {
    fn default() -> Self {
        Self {
            pc: 0x0ddb75,
            complete: false,
            near_returns: Vec::new(),
            far_returns: Vec::new(),
            shared: None,
            shared_return: None,
        }
    }
}
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    crate::ZeldaState::native_hud_update_table_byte(address)
}
impl HudUpdate {
    pub(crate) fn refresh_icon() -> Self {
        Self { pc: 0x0ddb7f, ..Self::default() }
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
        if let Some(shared) = self.shared.as_mut() {
            let cycles = shared.step(cpu, clock, bus);
            if shared.complete() {
                let (target, sp) = self
                    .shared_return
                    .take()
                    .expect("shared HUD call has a return");
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp),
                    ((target >> 16) as u8, target as u16, sp)
                );
                self.pc = target;
                self.shared = None;
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 8),
            ((self.pc >> 16) as u8, self.pc as u16, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let xmask = if x == 1 { 255 } else { 65535 };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xddb75 | 0xddb7f => {
                let next = match pc {
                    0xddb75 => 0xddb76,
                    0xddb7f => 0xddb80,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.push(u16::from(w.cpu.db), 1);
                next
            }
            0xddb76 | 0xddb80 => {
                let next = match pc {
                    0xddb76 => 0xddb77,
                    0xddb80 => 0xddb81,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                next
            }
            0xddb77 | 0xddb7d | 0xddb81 | 0xddb90 | 0xddd18 | 0xddd28 => {
                let next = match pc {
                    0xddb77 => 0xddb78,
                    0xddb7d => 0xddb7e,
                    0xddb81 => 0xddb82,
                    0xddb90 => 0xddb91,
                    0xddd18 => 0xddd19,
                    0xddd28 => 0xddd29,
                    _ => unreachable!(),
                };
                w.drain(12);
                let value = w.pull(1);
                w.cpu.db = value as u8;
                w.flags(value, 1);
                next
            }
            0xddb78 | 0xddbc6 | 0xddbfd | 0xddc02 | 0xddc75 | 0xddc91 | 0xddc96 | 0xddca8
            | 0xddcd8 | 0xdf19e => {
                let (operand, next) = match pc {
                    0xddb78 => (0x0200, 0xddb7b),
                    0xddbc6 => (0x012e, 0xddbc9),
                    0xddbfd => (0x012e, 0xddc00),
                    0xddc02 => (0x0cfd, 0xddc05),
                    0xddc75 => (0x02e4, 0xddc78),
                    0xddc91 => (0x04ca, 0xddc94),
                    0xddc96 => (0x012e, 0xddc99),
                    0xddca8 => (0x020a, 0xddcab),
                    0xddcd8 => (0x012f, 0xddcdb),
                    0xdf19e => (0x0209, 0xdf1a1),
                    _ => unreachable!(),
                };
                w.drain(16);
                let value = w.read(w.absolute(operand), m);
                w.load(value, m);
                next
            }
            0xddb7b | 0xddb96 | 0xddbda | 0xddc1c | 0xddc33 | 0xddc3e | 0xddc55 | 0xddc60
            | 0xddcb1 => {
                let (operand, next) = match pc {
                    0xddb7b => (0xddb92, 0xddb7d),
                    0xddb96 => (0xddbd0, 0xddb98),
                    0xddbda => (0xddc13, 0xddbdc),
                    0xddc1c => (0xddc3a, 0xddc1e),
                    0xddc33 => (0xddc3a, 0xddc35),
                    0xddc3e => (0xddc75, 0xddc40),
                    0xddc55 => (0xddc5c, 0xddc57),
                    0xddc60 => (0xddc75, 0xddc62),
                    0xddcb1 => (0xddd1a, 0xddcb3),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & ZERO != 0) {
                    operand
                } else {
                    next
                }
            }
            0xddb7e | 0xddb91 | 0xddd19 | 0xddd29 => {
                w.rtl();
                let target = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                if let Some(expected) = self.far_returns.pop() {
                    assert_eq!(target, expected);
                } else {
                    self.complete = true;
                }
                target
            }
            0xddb82 | 0xddb85 | 0xddb88 | 0xddd0e | 0xddd21 | 0xdf1ab => {
                let (operand, next) = match pc {
                    0xddb82 => (0xde399, 0xddb85),
                    0xddb85 => (0xddfa9, 0xddb88),
                    0xddb88 => (0xdfa70, 0xddb8b),
                    0xddd0e => (0xdfc09, 0xddd11),
                    0xddd21 => (0xdfb94, 0xddd24),
                    0xdf1ab => (0xdfa70, 0xdf1ae),
                    _ => unreachable!(),
                };
                let return_sp = w.cpu.sp;
                w.operand_then_internal(2);
                w.push((next - 1) as u16, 2);
                self.shared = Some(LoadFileHud::new_near(operand as u16, w.cpu.sp));
                self.shared_return = Some((next, return_sp));
                operand
            }
            0xddb8b | 0xddbfb | 0xddc13 | 0xddd14 | 0xddd24 | 0xdf14f | 0xdf182 | 0xdf1a9 => {
                let (operand, next) = match pc {
                    0xddb8b => (0x0030, 0xddb8d),
                    0xddbfb => (0x0030, 0xddbfd),
                    0xddc13 => (0x0030, 0xddc15),
                    0xddd14 => (0x0030, 0xddd16),
                    0xddd24 => (0x0030, 0xddd26),
                    0xdf14f => (0x0030, 0xdf151),
                    0xdf182 => (0x0030, 0xdf184),
                    0xdf1a9 => (0x0030, 0xdf1ab),
                    _ => unreachable!(),
                };
                w.width(operand as u8, false);
                next
            }
            0xddb8d | 0xddc15 | 0xdf1ae => {
                let (operand, next) = match pc {
                    0xddb8d => (0x0200, 0xddb90),
                    0xddc15 => (0x0cfd, 0xddc18),
                    0xdf1ae => (0x020a, 0xdf1b1),
                    _ => unreachable!(),
                };
                w.drain(16);
                w.write(w.absolute(operand), 0, m);
                next
            }
            0xddb92 | 0xddb98 | 0xddbae | 0xddbb7 | 0xddbd2 | 0xddc18 | 0xddc1e | 0xddc27
            | 0xddc2c | 0xddc3a | 0xddc40 | 0xddc49 | 0xddc4e | 0xddc5c | 0xddc68 | 0xddc7a
            | 0xddc80 | 0xddc88 | 0xddcad | 0xddcb3 | 0xddcbd | 0xddccd | 0xddce2 | 0xddcec
            | 0xddcf4 | 0xdf164 => {
                let (operand, next) = match pc {
                    0xddb92 => (0x7ef373, 0xddb96),
                    0xddb98 => (0x7ef36e, 0xddb9c),
                    0xddbae => (0x7ef373, 0xddbb2),
                    0xddbb7 => (0x7ef36e, 0xddbbb),
                    0xddbd2 => (0x7ef362, 0xddbd6),
                    0xddc18 => (0x7ef375, 0xddc1c),
                    0xddc1e => (0x7ef375, 0xddc22),
                    0xddc27 => (0x7ef370, 0xddc2b),
                    0xddc2c => (0x7ef343, 0xddc30),
                    0xddc3a => (0x7ef376, 0xddc3e),
                    0xddc40 => (0x7ef376, 0xddc44),
                    0xddc49 => (0x7ef371, 0xddc4d),
                    0xddc4e => (0x7ef377, 0xddc52),
                    0xddc5c => (0x7ef340, 0xddc60),
                    0xddc68 => (0x7ef340, 0xddc6c),
                    0xddc7a => (0x7ef372, 0xddc7e),
                    0xddc80 => (0x7ef36c, 0xddc84),
                    0xddc88 => (0x7ef36d, 0xddc8c),
                    0xddcad => (0x7ef372, 0xddcb1),
                    0xddcb3 => (0x7ef36d, 0xddcb7),
                    0xddcbd => (0x7ef36c, 0xddcc1),
                    0xddccd => (0x7ef36d, 0xddcd1),
                    0xddce2 => (0x7ef36d, 0xddce6),
                    0xddcec => (0x7ef36c, 0xddcf0),
                    0xddcf4 => (0x7ef372, 0xddcf8),
                    0xdf164 => (0x7ef36d, 0xdf168),
                    _ => unreachable!(),
                };
                w.drain(24);
                let value = w.read(operand, m);
                w.load(value, m);
                next
            }
            0xddb9c | 0xddbeb | 0xddc64 | 0xdf171 => {
                let (operand, next) = match pc {
                    0xddb9c => (0x0080, 0xddb9e),
                    0xddbeb => (0x03e8, 0xddbee),
                    0xddc64 => (0x0001, 0xddc66),
                    0xdf171 => (0x0014, 0xdf174),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & mask, m);
                next
            }
            0xddb9e | 0xddbee | 0xddcbb | 0xddcea | 0xdf174 => {
                let (operand, next) = match pc {
                    0xddb9e => (0xddbae, 0xddba0),
                    0xddbee => (0xddbf7, 0xddbf0),
                    0xddcbb => (0xddccd, 0xddcbd),
                    0xddcea => (0xddcf4, 0xddcec),
                    0xdf174 => (0xdf182, 0xdf176),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & CARRY == 0) {
                    operand
                } else {
                    next
                }
            }
            0xddba0 | 0xddba6 | 0xddbcb | 0xddbe1 | 0xddbf0 | 0xddc0c | 0xddc9b | 0xddca0
            | 0xddcc5 | 0xddcdd | 0xddd02 | 0xddd09 | 0xddd1c | 0xdf151 | 0xdf155 | 0xdf159 => {
                let (operand, next) = match pc {
                    0xddba0 => (0x0080, 0xddba2),
                    0xddba6 => (0x0000, 0xddba8),
                    0xddbcb => (0x002d, 0xddbcd),
                    0xddbe1 => (0x0000, 0xddbe4),
                    0xddbf0 => (0x03e7, 0xddbf3),
                    0xddc0c => (0x0029, 0xddc0e),
                    0xddc9b => (0x0020, 0xddc9d),
                    0xddca0 => (0x002b, 0xddca2),
                    0xddcc5 => (0x0000, 0xddcc7),
                    0xddcdd => (0x000d, 0xddcdf),
                    0xddd02 => (0x0007, 0xddd04),
                    0xddd09 => (0xffff, 0xddd0c),
                    0xddd1c => (0xffff, 0xddd1f),
                    0xdf151 => (0x0068, 0xdf153),
                    0xdf155 => (0x00c7, 0xdf157),
                    0xdf159 => (0x007e, 0xdf15b),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load(value, m);
                next
            }
            0xddba2 | 0xddba8 | 0xddbb3 | 0xddbbc | 0xddbe4 | 0xddbf3 | 0xddbf7 | 0xddc23
            | 0xddc36 | 0xddc45 | 0xddc58 | 0xddc6d | 0xddcc1 | 0xddcc7 | 0xddcd4 | 0xddcf0
            | 0xddcfb => {
                let (operand, next) = match pc {
                    0xddba2 => (0x7ef36e, 0xddba6),
                    0xddba8 => (0x7ef373, 0xddbac),
                    0xddbb3 => (0x7ef373, 0xddbb7),
                    0xddbbc => (0x7ef36e, 0xddbc0),
                    0xddbe4 => (0x7ef360, 0xddbe8),
                    0xddbf3 => (0x7ef360, 0xddbf7),
                    0xddbf7 => (0x7ef362, 0xddbfb),
                    0xddc23 => (0x7ef375, 0xddc27),
                    0xddc36 => (0x7ef343, 0xddc3a),
                    0xddc45 => (0x7ef376, 0xddc49),
                    0xddc58 => (0x7ef377, 0xddc5c),
                    0xddc6d => (0x7ef340, 0xddc71),
                    0xddcc1 => (0x7ef36d, 0xddcc5),
                    0xddcc7 => (0x7ef372, 0xddccb),
                    0xddcd4 => (0x7ef36d, 0xddcd8),
                    0xddcf0 => (0x7ef36d, 0xddcf4),
                    0xddcfb => (0x7ef372, 0xddcff),
                    _ => unreachable!(),
                };
                w.drain(24);
                w.write(operand, w.cpu.a & mask, m);
                next
            }
            0xddbac | 0xddbe8 | 0xddc11 | 0xddccb => {
                let (operand, next) = match pc {
                    0xddbac => (0xddbd0, 0xddbae),
                    0xddbe8 => (0xddbf7, 0xddbea),
                    0xddc11 => (0xddc18, 0xddc13),
                    0xddccb => (0xddd1a, 0xddccd),
                    _ => unreachable!(),
                };
                if w.branch(true) {
                    operand
                } else {
                    next
                }
            }
            0xddbb2 | 0xddbde | 0xddc22 | 0xddc44 | 0xdf16b => {
                let next = match pc {
                    0xddbb2 => 0xddbb3,
                    0xddbde => 0xddbdf,
                    0xddc22 => 0xddc23,
                    0xddc44 => 0xddc45,
                    0xdf16b => 0xdf16c,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1) & mask, m);
                next
            }
            0xddbbb | 0xddbea | 0xddc35 | 0xddc57 | 0xddc6c | 0xdf1a1 => {
                let next = match pc {
                    0xddbbb => 0xddbbc,
                    0xddbea => 0xddbeb,
                    0xddc35 => 0xddc36,
                    0xddc57 => 0xddc58,
                    0xddc6c => 0xddc6d,
                    0xdf1a1 => 0xdf1a2,
                    _ => unreachable!(),
                };
                w.increment_a(m);
                next
            }
            0xddbc0 | 0xdf17a => {
                let (operand, next) = match pc {
                    0xddbc0 => (0x001a, 0xddbc2),
                    0xdf17a => (0x0000, 0xdf17c),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.load(value, m);
                next
            }
            0xddbc2 | 0xddc08 | 0xddc62 | 0xdf168 | 0xdf1a2 => {
                let (operand, next) = match pc {
                    0xddbc2 => (0x0003, 0xddbc4),
                    0xddc08 => (0x0007, 0xddc0a),
                    0xddc62 => (0x0001, 0xddc64),
                    0xdf168 => (0x00f8, 0xdf16b),
                    0xdf1a2 => (0x0003, 0xdf1a4),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load(w.cpu.a & value & mask, m);
                next
            }
            0xddbc4 | 0xddbc9 | 0xddc00 | 0xddc0a | 0xddc66 | 0xddc78 | 0xddc7e | 0xddc94
            | 0xddc99 | 0xddcab | 0xddcdb | 0xdf160 | 0xdf1a7 => {
                let (operand, next) = match pc {
                    0xddbc4 => (0xddbd0, 0xddbc6),
                    0xddbc9 => (0xddbd0, 0xddbcb),
                    0xddc00 => (0xddc13, 0xddc02),
                    0xddc0a => (0xddc18, 0xddc0c),
                    0xddc66 => (0xddc75, 0xddc68),
                    0xddc78 => (0xddca8, 0xddc7a),
                    0xddc7e => (0xddca8, 0xddc80),
                    0xddc94 => (0xddca5, 0xddc96),
                    0xddc99 => (0xddca8, 0xddc9b),
                    0xddcab => (0xddd07, 0xddcad),
                    0xddcdb => (0xddce2, 0xddcdd),
                    0xdf160 => (0xdf1b1, 0xdf162),
                    0xdf1a7 => (0xdf1b1, 0xdf1a9),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & ZERO == 0) {
                    operand
                } else {
                    next
                }
            }
            0xddbcd | 0xddc0e | 0xddc9d | 0xddca2 | 0xddcdf | 0xddd04 | 0xdf18b | 0xdf1a4 => {
                let (operand, next) = match pc {
                    0xddbcd => (0x012e, 0xddbd0),
                    0xddc0e => (0x012e, 0xddc11),
                    0xddc9d => (0x04ca, 0xddca0),
                    0xddca2 => (0x012e, 0xddca5),
                    0xddcdf => (0x012f, 0xddce2),
                    0xddd04 => (0x0208, 0xddd07),
                    0xdf18b => (0x0208, 0xdf18e),
                    0xdf1a4 => (0x0209, 0xdf1a7),
                    _ => unreachable!(),
                };
                w.drain(16);
                w.write(w.absolute(operand), w.cpu.a & mask, m);
                next
            }
            0xddbd0 | 0xddd07 | 0xddd1a | 0xdf162 => {
                let (operand, next) = match pc {
                    0xddbd0 => (0x0030, 0xddbd2),
                    0xddd07 => (0x0030, 0xddd09),
                    0xddd1a => (0x0030, 0xddd1c),
                    0xdf162 => (0x0030, 0xdf164),
                    _ => unreachable!(),
                };
                w.width(operand as u8, true);
                next
            }
            0xddbd6 | 0xddcb7 | 0xddce6 => {
                let (operand, next) = match pc {
                    0xddbd6 => (0x7ef360, 0xddbda),
                    0xddcb7 => (0x7ef36c, 0xddcbb),
                    0xddce6 => (0x7ef36c, 0xddcea),
                    _ => unreachable!(),
                };
                w.drain(24);
                let value = w.read(operand, m);
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & mask, m);
                next
            }
            0xddbdc => {
                let (operand, next) = (0xddbea, 0xddbde);
                if w.branch(w.cpu.p & NEGATIVE != 0) {
                    operand
                } else {
                    next
                }
            }
            0xddbdf => {
                let (operand, next) = (0xddbf7, 0xddbe1);
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    operand
                } else {
                    next
                }
            }
            0xddc05 | 0xddcff => {
                let (operand, next) = match pc {
                    0xddc05 => (0x0cfd, 0xddc08),
                    0xddcff => (0x020a, 0xddd02),
                    _ => unreachable!(),
                };
                w.drain(16);
                let address = w.absolute(operand);
                let old = w.read(address, m);
                w.drain(6);
                let value = old.wrapping_add(1) & mask;
                w.flags(value, m);
                w.write_high_low(address, value, m);
                next
            }
            0xddc2b | 0xddc4d | 0xdf170 | 0xdf179 => {
                let next = match pc {
                    0xddc2b => 0xddc2c,
                    0xddc4d => 0xddc4e,
                    0xdf170 => 0xdf171,
                    0xdf179 => 0xdf17a,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.y = w.cpu.a & xmask;
                w.flags(w.cpu.y, x);
                next
            }
            0xddc30 | 0xddc52 => {
                let (operand, next) = match pc {
                    0xddc30 => (0xdb48, 0xddc33),
                    0xddc52 => (0xdb58, 0xddc55),
                    _ => unreachable!(),
                };
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.y), m);
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & mask, m);
                next
            }
            0xddc71 => {
                let (operand, next) = (0xddb7f, 0xddc75);
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push((next - 1) as u16, 2);
                self.far_returns.push(next);
                operand
            }
            0xddc84 | 0xddc85 | 0xddc86 | 0xdf16c | 0xdf16d | 0xdf16e => {
                let next = match pc {
                    0xddc84 => 0xddc85,
                    0xddc85 => 0xddc86,
                    0xddc86 => 0xddc87,
                    0xdf16c => 0xdf16d,
                    0xdf16d => 0xdf16e,
                    0xdf16e => 0xdf16f,
                    _ => unreachable!(),
                };
                w.drain(6);
                let value = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | (value & 1) as u8;
                w.load(value >> 1, m);
                next
            }
            0xddc87 | 0xdf190 => {
                let next = match pc {
                    0xddc87 => 0xddc88,
                    0xdf190 => 0xdf191,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.x = w.cpu.a & xmask;
                w.flags(w.cpu.x, x);
                next
            }
            0xddc8c => {
                let (operand, next) = (0xdb60, 0xddc8f);
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.x), m);
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & mask, m);
                next
            }
            0xddc8f => {
                let (operand, next) = (0xddca8, 0xddc91);
                if w.branch(w.cpu.p & CARRY != 0) {
                    operand
                } else {
                    next
                }
            }
            0xddca5 | 0xdf15d => {
                let (operand, next) = match pc {
                    0xddca5 => (0x04ca, 0xddca8),
                    0xdf15d => (0x0208, 0xdf160),
                    _ => unreachable!(),
                };
                w.drain(16);
                let address = w.absolute(operand);
                let old = w.read(address, m);
                w.drain(6);
                let value = old.wrapping_sub(1) & mask;
                w.flags(value, m);
                w.write_high_low(address, value, m);
                next
            }
            0xddcd1 | 0xdf17c | 0xdf1b1 => {
                let next = match pc {
                    0xddcd1 => 0xddcd2,
                    0xdf17c => 0xdf17d,
                    0xdf1b1 => 0xdf1b2,
                    _ => unreachable!(),
                };
                w.clear_carry();
                next
            }
            0xddcd2 | 0xdf17d => {
                let (operand, next) = match pc {
                    0xddcd2 => (0x0008, 0xddcd4),
                    0xdf17d => (0x0040, 0xdf180),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                next
            }
            0xddcf8 => {
                let next = 0xddcf9;
                w.drain(6);
                w.cpu.p |= CARRY;
                next
            }
            0xddcf9 | 0xdf176 => {
                let (operand, next) = match pc {
                    0xddcf9 => (0x0008, 0xddcfb),
                    0xdf176 => (0x0014, 0xdf179),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.sbc(value, m);
                next
            }
            0xddd0c | 0xddd1f | 0xdf153 | 0xdf157 | 0xdf15b | 0xdf180 => {
                let (operand, next) = match pc {
                    0xddd0c => (0x000e, 0xddd0e),
                    0xddd1f => (0x000e, 0xddd21),
                    0xdf153 => (0x0000, 0xdf155),
                    0xdf157 => (0x0001, 0xdf159),
                    0xdf15b => (0x0002, 0xdf15d),
                    0xdf180 => (0x0000, 0xdf182),
                    _ => unreachable!(),
                };
                w.dp_store(operand, w.cpu.a & mask, m);
                next
            }
            0xddd11 => {
                let (operand, next) = (0xdf14f, 0xddd14);
                w.operand_then_internal(2);
                w.push((next - 1) as u16, 2);
                self.near_returns.push(next);
                operand
            }
            0xddd16 | 0xddd26 => {
                let (operand, next) = match pc {
                    0xddd16 => (0x0016, 0xddd18),
                    0xddd26 => (0x0016, 0xddd28),
                    _ => unreachable!(),
                };
                w.dp_increment(operand, m, true);
                next
            }
            0xdf16f | 0xdf18f => {
                let next = match pc {
                    0xdf16f => 0xdf170,
                    0xdf18f => 0xdf190,
                    _ => unreachable!(),
                };
                w.shift_left(m);
                next
            }
            0xdf184 => {
                let (operand, next) = (0x0209, 0xdf187);
                w.drain(16);
                let value = w.read(w.absolute(operand), x);
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0xdf187 | 0xdf191 | 0xdf198 => {
                let (operand, next) = match pc {
                    0xdf187 => (0xdfa11, 0xdf18b),
                    0xdf191 => (0xdfa09, 0xdf195),
                    0xdf198 => (0xdfa0a, 0xdf19c),
                    _ => unreachable!(),
                };
                w.drain(24);
                let value = w.read(operand + u32::from(w.cpu.x), m);
                w.load(value, m);
                next
            }
            0xdf18e => {
                let next = 0xdf18f;
                w.drain(6);
                w.load(w.cpu.x, m);
                next
            }
            0xdf195 | 0xdf19c => {
                let (operand, next) = match pc {
                    0xdf195 => (0x0000, 0xdf197),
                    0xdf19c => (0x0000, 0xdf19e),
                    _ => unreachable!(),
                };
                let address = w.indirect_y_address(operand);
                w.write(address, w.cpu.a & mask, m);
                next
            }
            0xdf197 => {
                let next = 0xdf198;
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & xmask;
                w.flags(w.cpu.y, x);
                next
            }
            0xdf1b2 => {
                w.rts();
                let target = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                assert_eq!(self.near_returns.pop(), Some(target));
                target
            }
            _ => unreachable!("HUD update continuation {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{intro_clear::Access, native_work::compare_instruction};
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn hud_update_matches_source_refills_refresh_animation_and_nested_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut visited = BTreeSet::new();
        let mut instructions = 0;
        for case in 0..4096usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            let r = &mut machine.ram;
            r[0x200] = u8::from(case % 32 == 0);
            r[0x202] = (case % 21) as u8;
            r[0xf340..0xf354].fill(1);
            r[0xf34f] = 1;
            r[0xf35c] = 1;
            r[0xf340] = (case % 5) as u8;
            r[0xf36f] = (case % 10) as u8;
            r[0xf373] = [0, 1, 127, 128][case % 4];
            r[0xf36e] = [0, 1, 126, 127, 128, 129, 254, 255][case / 4 % 8];
            let (current, target): (u16, u16) = [
                (0, 0),
                (0, 1),
                (1, 0),
                (10, 9),
                (998, 999),
                (999, 1000),
                (1000, 999),
                (0, 65535),
            ][case / 32 % 8];
            r[0xf362..0xf364].copy_from_slice(&current.to_le_bytes());
            r[0xf360..0xf362].copy_from_slice(&target.to_le_bytes());
            r[0x1a] = (case / 8) as u8;
            r[0x12e] = if case & 8 != 0 { 5 } else { 0 };
            r[0x12f] = if case & 16 != 0 { 6 } else { 0 };
            r[0xcfd] = (case / 16 % 16) as u8;
            r[0xf375] = (case % 3) as u8;
            r[0xf370] = (case / 8 % 8) as u8;
            let bomb_max = [10, 15, 20, 25, 30, 35, 40, 50][case / 8 % 8];
            r[0xf343] = if case & 32 != 0 {
                bomb_max
            } else {
                bomb_max - 1
            };
            r[0xf376] = (case / 3 % 3) as u8;
            r[0xf371] = (case / 16 % 8) as u8;
            let arrow_max = [30, 35, 40, 45, 50, 55, 60, 70][case / 16 % 8];
            r[0xf377] = if case & 64 != 0 {
                arrow_max
            } else {
                arrow_max - 1
            };
            let capacity = [8, 24, 72, 80, 88, 152, 160, 160][case / 128 % 8];
            r[0xf36c] = capacity;
            r[0xf36d] = [0, 1, 8, 9, 16, capacity - 1, capacity, capacity + 1][case / 256 % 8];
            r[0xf372] = [0, 8, 16, 160][case / 512 % 4];
            r[0x4ca] = [0, 1, 31, 255][case / 64 % 4];
            r[0x2e4] = u8::from(case % 7 == 0);
            r[0x20a] = u8::from(case & 512 != 0);
            r[0x208] = [0, 1, 2, 7][case / 1024 % 4];
            r[0x209] = (case / 256 % 4) as u8;
            r[0x1ed..0x1f0].copy_from_slice(&[0x59, 0x88, 2]);
            let mut cpu = NativeNmiContext {
                a: 0xabcd,
                x: 13,
                y: 7,
                d: 0,
                db: 0,
                bank: 13,
                pc: 0xdb75,
                sp: 0x1ec,
                p: 0x30 | (case as u8 & 0xc3),
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
            let timeline = CpuMasterTimeline::at_raster(
                2000 + case as u64,
                snes::CpuRasterPosition::new(if case & 1 == 0 { 224 } else { 261 }, 1300),
                snes::CpuBusWorkload::default(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source = snes::RomCpuTimingProbe::new(
                machine,
                timeline,
                snes::SourcePpuReadState::snes9x_reset(),
            )
            .unwrap();
            let mut clock = source.timeline().clone();
            let mut work = HudUpdate::default();
            for _ in 0..20000 {
                if work.complete() {
                    break;
                }
                visited.insert(cpu.pc);
                compare_instruction(
                    &format!("HUD update case={case} pc={:04x}", cpu.pc),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, |at, access| {
                            if let Access::Read(a, width) = access {
                                if a & 0x408000 == 0x8000 {
                                    for b in 0..width {
                                        let a = a + u32::from(b);
                                        assert_eq!(
                                            table_byte(a),
                                            Some(
                                                rom[(a >> 16) as usize * 0x8000
                                                    + (a as usize & 0x7fff)]
                                            ),
                                            "HUD table {a:06x}"
                                        );
                                    }
                                }
                            }
                            bus(at, access)
                        })
                        .unwrap();
                    },
                );
                work = work.clone();
                instructions += 1;
            }
            assert!(
                work.complete(),
                "HUD case {case} unfinished at {:04x}",
                cpu.pc
            );
            assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (2, 0x885a, 0x1ef, 0));
            assert!(
                work.near_returns.is_empty()
                    && work.far_returns.is_empty()
                    && work.shared.is_none()
            );
            assert_eq!(ram, source.snes().ram);
        }
        for pc in [
            0xdb7e, 0xdb91, 0xdbe1, 0xdbf0, 0xdc71, 0xdd19, 0xdd29, 0xf195, 0xf19c, 0xf1ab, 0xdfa9,
            0xfa85, 0xfb94, 0xfc09,
        ] {
            assert!(visited.contains(&pc), "uncovered HUD path {pc:04x}");
        }
        eprintln!(
            "4096 HUD-update fixtures, {instructions} instruction boundaries, {} distinct PCs",
            visited.len()
        );
    }
}
