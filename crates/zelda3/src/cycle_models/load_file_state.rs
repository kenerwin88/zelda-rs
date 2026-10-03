//! Shared save-load state initializers. Individual table copies and Link
//! reset stores retain their instruction boundary and the live return stack.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoadFileState {
    entry: u32,
    pc: u16,
    complete: bool,
}
impl LoadFileState {
    pub(crate) fn new(entry: u32) -> Self {
        assert!(matches!(entry, 0x00fc41 | 0x0e97d9 | 0x07f13c));
        Self {
            entry,
            pc: entry as u16,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        self.entry
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
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d),
            ((self.entry >> 16) as u8, self.pc, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        w.fetch();
        self.pc = match self.pc {
            0xfc41 | 0xfc42 | 0xf13c | 0xf13d | 0x983a | 0x983b => {
                w.drain(6);
                w.push(
                    u16::from(if matches!(self.pc, 0xfc42 | 0xf13d | 0x983b) {
                        w.cpu.bank
                    } else {
                        w.cpu.db
                    }),
                    1,
                );
                self.pc + 1
            }
            0xfc43 | 0xfc9a | 0xf13e | 0xf1a1 | 0x983c | 0x9849 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                self.pc + 1
            }
            0xfc44 | 0xfc98 | 0x97d9 | 0x9811 | 0x9873 | 0x9893 => {
                w.width(
                    if matches!(self.pc, 0xfc44 | 0xfc98) {
                        0x30
                    } else {
                        0x20
                    },
                    matches!(self.pc, 0xfc44 | 0x97d9 | 0x9873),
                );
                self.pc + 2
            }
            0xfc46 | 0xfc67 | 0xfc76 | 0xfc7e => {
                w.drain(16);
                w.cpu.y = match self.pc {
                    0xfc46 => 254,
                    0xfc76 => 126,
                    0xfc7e => 190,
                    _ => 62,
                };
                w.flags(w.cpu.y, 2);
                self.pc + 3
            }
            0xfc49 | 0xfc81 | 0x97db => {
                w.drain(8 * u32::from(x));
                w.cpu.x = 62;
                w.flags(62, x);
                self.pc + 1 + u16::from(x)
            }
            0xfc4c | 0xfc53 | 0xfc84 | 0xfc8b => {
                w.operand_then_internal(2);
                let v = w.read(
                    w.absolute(if matches!(self.pc, 0xfc4c | 0xfc84) {
                        0xfa41
                    } else {
                        0xfb41
                    }) + u32::from(w.cpu.y),
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0xfc4f | 0xfc56 | 0xfc87 | 0xfc8e => {
                w.drain(24);
                w.write(
                    match self.pc {
                        0xfc4f => 0x7efd00,
                        0xfc56 => 0x7efd80,
                        0xfc87 => 0x7efcc0,
                        _ => 0x7efd40,
                    } + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                self.pc + 4
            }
            0xfc5a | 0xfc5b | 0xfc92 | 0xfc93 => {
                w.decrement_y();
                self.pc + 1
            }
            0xfc5c | 0xfc5d | 0xfc94 | 0xfc95 | 0x980d | 0x980e => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & if x == 1 { 255 } else { 65535 };
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xfc5e | 0xfc60 | 0xfc74 | 0xfc7c | 0xfc96 | 0x980f => {
                let (t, a) = match self.pc {
                    0xfc5e => (w.cpu.p & NEGATIVE == 0, 0xfc4c),
                    0xfc60 => (true, 0xfc67),
                    0xfc74 => (w.cpu.p & CARRY == 0, 0xfc81),
                    0xfc7c => (w.cpu.p & ZERO == 0, 0xfc81),
                    0xfc96 => (w.cpu.p & NEGATIVE == 0, 0xfc84),
                    _ => (w.cpu.p & NEGATIVE == 0, 0x97dd),
                };
                if w.branch(t) {
                    a
                } else {
                    self.pc + 2
                }
            }
            0xfc6a => {
                w.drain(24);
                let v = w.read(0x7ef3c5, 2);
                w.load(v, 2);
                self.pc + 4
            }
            0xfc6e => {
                w.mask(255, 2);
                self.pc + 3
            }
            0xfc71 | 0xfc79 => {
                w.compare_immediate(if self.pc == 0xfc71 { 2 } else { 3 }, 2);
                self.pc + 3
            }
            0x97dd | 0x97e5 | 0x97ed | 0x97f5 | 0x97fd | 0x9805 => {
                w.drain(24);
                let a = 0x0e9659 + u32::from((self.pc - 0x97dd) / 8) * 64 + u32::from(w.cpu.x);
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 4
            }
            0x97e1 | 0x97e9 | 0x97f1 | 0x97f9 | 0x9801 | 0x9809 => {
                let i = (self.pc - 0x97e1) / 8;
                w.drain(24);
                w.write(
                    0x7efe00 + u32::from(if i == 5 { 7 } else { i }) * 64 + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                self.pc + 4
            }
            0xf13f | 0xae54 => {
                w.immediate(if self.pc == 0xf13f { 2 } else { 0x20 }, 1);
                self.pc + 2
            }
            0xf141 | 0xf163 | 0xf176 | 0xf199 | 0x9d8a | 0x9d94 | 0x9d9a => {
                w.dp_store(
                    match self.pc {
                        0xf141 => 0x2f,
                        0xf163 | 0x9d94 => 0x3a,
                        0xf199 => 0x67,
                        0x9d8a => 0x48,
                        _ => 0x50,
                    },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 2
            }
            0xf143 | 0xf15d | 0xf178 | 0xf17a | 0xf17c | 0xf187 | 0xae5c | 0x9d84 | 0x9d8c
            | 0x9d8e => {
                w.dp_store(
                    match self.pc {
                        0xf143 => 0x26,
                        0xf15d => 0x3b,
                        0xf178 => 0x25,
                        0xf17a => 0x4d,
                        0xf17c => 0x46,
                        0x9d84 => 0x5e,
                        0x9d8c => 0x3d,
                        0x9d8e => 0x3c,
                        _ => 0x55,
                    },
                    0,
                    1,
                );
                self.pc + 2
            }
            0xf15f | 0xf172 | 0xf195 | 0x9d86 | 0x9d90 | 0x9d96 => {
                let v = w.dp_read(
                    match self.pc {
                        0xf15f | 0x9d90 => 0x3a,
                        0xf195 => 0x67,
                        0x9d86 => 0x48,
                        _ => 0x50,
                    },
                    1,
                );
                w.load(v, 1);
                self.pc + 2
            }
            0xf161 | 0xf174 | 0xf197 | 0x9d88 | 0x9d92 | 0x9d98 => {
                w.mask(
                    match self.pc {
                        0xf161 => 0xbf,
                        0xf197 => 0xf0,
                        0x9d88 => 0xf6,
                        0x9d92 => 0x7e,
                        _ => 0xfe,
                    },
                    1,
                );
                self.pc + 2
            }
            0xf145 | 0xf148 | 0xf14b | 0xf14e | 0xf151 | 0xf154 | 0xf157 | 0xf15a | 0xf165
            | 0xf168 | 0xf16b | 0xf17e | 0xf181 | 0xf184 | 0xf18f | 0xf192 | 0xf19b | 0xf19e
            | 0x983d | 0x9840 | 0x9843 | 0xae56 | 0xae59 | 0xae5e | 0x9875 | 0x9878 | 0x987b
            | 0x987e | 0x9881 | 0x9884 | 0x9887 | 0x988a | 0x988d | 0x9890 => {
                let a = match self.pc {
                    0xf145 => 0x301,
                    0xf148 => 0x37a,
                    0xf14b => 0x20b,
                    0xf14e => 0x350,
                    0xf151 => 0x30d,
                    0xf154 => 0x30e,
                    0xf157 => 0x30a,
                    0xf15a => 0x2e1,
                    0xf165 => 0x308,
                    0xf168 => 0x309,
                    0xf16b => 0x376,
                    0xf17e => 0x31f,
                    0xf181 | 0xae5e => 0x360,
                    0xf184 => 0x2da,
                    0xf18f | 0xae59 => 0x37b,
                    0xf192 => 0x300,
                    0xf19b => 0x2f5,
                    0xf19e => 0x79,
                    0x983d => 0x2cb,
                    0x9840 => 0x34f,
                    0x9843 => 0x32a,
                    0xae56 => 0x2e2,
                    0x9875 => 0x32f,
                    0x9878 => 0x331,
                    0x987b => 0x326,
                    0x987e => 0x328,
                    0x9881 => 0x32b,
                    0x9884 => 0x32d,
                    0x9887 => 0x33c,
                    0x988a => 0x33e,
                    0x988d => 0x334,
                    _ => 0x336,
                };
                w.drain(16);
                w.write(
                    w.absolute(a),
                    if self.pc == 0xae56 { w.cpu.a & 255 } else { 0 },
                    m,
                );
                self.pc + 3
            }
            0xf16e => {
                w.operand_then_internal(3);
                w.push(7, 1);
                w.push(0xf171, 2);
                0x983a
            }
            0xf189 | 0xf18c | 0x9846 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                match self.pc {
                    0xf189 => 0xae54,
                    0xf18c => 0x9d84,
                    _ => 0x9873,
                }
            }
            0xae61 | 0x9e62 | 0x9895 => {
                w.rts();
                w.cpu.pc
            }
            0x9d9c => {
                w.operand_then_internal(2);
                0x9e62
            }
            0xfc9b | 0x9813 | 0xf1a2 | 0x984a => {
                w.rtl();
                if self.pc != 0x984a {
                    self.complete = true;
                }
                w.cpu.pc
            }
            _ => unreachable!("save-load state continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
