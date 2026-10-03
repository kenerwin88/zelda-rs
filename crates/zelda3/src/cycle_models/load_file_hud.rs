//! Save-load HUD calls with live inventory, indirect tile pointers, decimal
//! scratch and the interruptible template MVN. Nested calls share the CPU.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoadFileHud {
    entry: u16,
    pc: u16,
    complete: bool,
    near_return_sp: Option<u16>,
}
impl LoadFileHud {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(entry, 0xe395 | 0xdd32 | 0xfa6c | 0xfa68));
        Self {
            entry,
            pc: entry,
            complete: false,
            near_return_sp: None,
        }
    }
    /// The caller already performed its real JSR. Complete only when that
    /// return has been popped; nested helper returns stay within this owner.
    pub(crate) fn new_near(entry: u16, entry_sp: u16) -> Self {
        assert!(matches!(entry, 0xe399 | 0xdfa9 | 0xfa70 | 0xfb94 | 0xfc09));
        Self {
            entry,
            pc: entry,
            complete: false,
            near_return_sp: Some(entry_sp.wrapping_add(2)),
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x0d0000 | u32::from(self.entry)
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (13, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let index_mask = if x == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match self.pc {
            0xe395 | 0xe3c0 | 0xdee2 | 0xdee5 | 0xdd32 | 0xfa6c | 0xfa8a | 0xfb91 | 0xfbbd
            | 0xfc06 | 0xfc5b | 0xfc8b | 0xfcaf | 0xfcdd | 0xfdbd | 0xdfac => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                match self.pc {
                    0xe395 => 0xe399,
                    0xe3c0 | 0xdee5 => 0xdeb0,
                    0xdee2 => 0xdecb,
                    0xdd32 => 0xdfaf,
                    0xfa6c => 0xfa70,
                    0xdfac => 0xfa85,
                    0xfa8a => 0xfb91,
                    0xfb91 => 0xfafd,
                    0xfbbd | 0xfc06 => 0xfdab,
                    0xfdbd => 0xfdd9,
                    _ => 0xf0f7,
                }
            }
            0xe398 | 0xdd35 | 0xfa6f => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            0xe3b5 | 0xdeba | 0xdebc | 0xded8 | 0xdeea | 0xdfb9 | 0xfa92 | 0xfb90 | 0xfcf9
            | 0xfdd8 | 0xfdee | 0xf127 => {
                w.rts();
                if self.near_return_sp == Some(w.cpu.sp) {
                    self.complete = true;
                }
                w.cpu.pc
            }
            0xe3c5 => {
                w.drain(16);
                0xdee2
            }
            0xe399 | 0xfa70 | 0xfa8e | 0xfafd | 0xfb3c | 0xfb94 | 0xfbae | 0xfbc0 | 0xfbf4
            | 0xfc09 | 0xfc5e | 0xfc8e | 0xfcb2 | 0xfce0 | 0xfcf7 | 0xf0f7 | 0xf116 | 0xfa85 => {
                w.width(
                    0x30,
                    matches!(
                        self.pc,
                        0xfa70
                            | 0xfa85
                            | 0xfb3c
                            | 0xfbae
                            | 0xfbf4
                            | 0xfc09
                            | 0xfc5e
                            | 0xfc8e
                            | 0xfcb2
                            | 0xfce0
                            | 0xf0f7
                    ),
                );
                self.pc + 2
            }
            0xfa72 | 0xfa80 | 0xfa81 | 0xfa87 | 0xfa88 => {
                w.drain(6);
                w.push(
                    u16::from(if matches!(self.pc, 0xfa81 | 0xfa88) {
                        w.cpu.bank
                    } else {
                        w.cpu.db
                    }),
                    1,
                );
                self.pc + 1
            }
            0xfa7f | 0xfa82 | 0xfa8d | 0xfa89 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                self.pc + 1
            }
            0xdfa9 => {
                w.drain(16);
                let address = w.absolute(0x200);
                let value = w.read(address, m).wrapping_add(1) & mask;
                w.drain(6);
                w.flags(value, m);
                w.write_high_low(address, value, m);
                self.pc + 3
            }
            0xfa7c => {
                w.drain(8);
                assert_eq!(w.read(0x0dfa7e, 1), 13);
                let v = w.read(0x0d0000 | u32::from(w.cpu.x), 1);
                w.write(0x7e0000 | u32::from(w.cpu.y), v, 1);
                w.drain(12);
                w.cpu.db = 0x7e;
                w.cpu.x = w.cpu.x.wrapping_add(1);
                w.cpu.y = w.cpu.y.wrapping_add(1);
                w.cpu.a = w.cpu.a.wrapping_sub(1);
                if w.cpu.a != 65535 {
                    self.pc
                } else {
                    self.pc + 3
                }
            }
            0xe39b | 0xfa76 | 0xfb21 | 0xfb29 | 0xfb2d | 0xfb35 | 0xfdab | 0xfdde | 0xf0fc
            | 0xf118 => {
                let v = match self.pc {
                    0xe39b => 18,
                    0xfa76 => 0xfe77,
                    0xfb21 => 4,
                    0xfb29 => 3,
                    0xfb2d | 0xf118 => 2,
                    0xfb35 => 1,
                    _ => 0,
                };
                w.drain(8 * u32::from(x));
                w.cpu.x = v;
                w.flags(v, x);
                self.pc + 1 + u16::from(x)
            }
            0xfa79 | 0xfdba | 0xfdc9 | 0xfdd3 | 0xf0ff => {
                w.drain(8 * u32::from(x));
                w.cpu.y = match self.pc {
                    0xfa79 => 0xc700,
                    0xfdba | 0xfdc9 => 4,
                    _ => 2,
                };
                w.flags(w.cpu.y, x);
                self.pc + 1 + u16::from(x)
            }
            0xe3bb | 0xded3 | 0xfa73 | 0xfb09 | 0xfb0f | 0xfb15 | 0xfb1b | 0xfb4f | 0xfb96
            | 0xfb9a | 0xfb9e | 0xfba2 | 0xfba6 | 0xfbaa | 0xfbc2 | 0xfbc6 | 0xfbca | 0xfbce
            | 0xfbd2 | 0xfbd6 | 0xfc17 | 0xfc1e | 0xfc25 | 0xfccc => {
                let v = match self.pc {
                    0xe3bb | 0xded3 | 0xfb4f => 1,
                    0xfa73 => 0x149,
                    0xfb09 => 0x86,
                    0xfb15 => 0x87,
                    0xfb0f | 0xfb1b => 0x24,
                    0xfb96 => 0xfd,
                    0xfb9a => 0xf9,
                    0xfb9e | 0xfbca => 13,
                    0xfba2 | 0xfbce => 0x68,
                    0xfba6 | 0xfbd2 => 0xc7,
                    0xfbaa | 0xfbd6 => 0x7e,
                    0xfbc2 => 3,
                    0xfbc6 => 0xfa,
                    0xfc17 => 0x28f7,
                    0xfc1e => 0x2851,
                    0xfc25 => 0x28fa,
                    _ => 0x7f,
                };
                w.immediate(v, m);
                self.pc + 1 + u16::from(m)
            }
            0xe3a8 | 0xdecf | 0xfb05 | 0xfc12 | 0xfcd8 | 0xfcee | 0xfdb0 | 0xfdc4 | 0xfdce
            | 0xf11c => {
                w.compare_immediate(
                    match self.pc {
                        0xe3a8 => 0,
                        0xdecf => 21,
                        0xfb05 => 3,
                        0xfc12 | 0xfdce => 1,
                        0xfcd8 => 255,
                        0xfcee => 0x247f,
                        0xfdb0 => 8,
                        0xfdc4 => 5,
                        _ => 0x7f,
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0xfb4a | 0xfb52 | 0xfdd9 => {
                w.compare_x_immediate(
                    match self.pc {
                        0xfb4a => 4,
                        0xfb52 => 16,
                        _ => 20,
                    },
                    x,
                );
                self.pc + 1 + u16::from(x)
            }
            0xfb47 | 0xfb5d | 0xfbb4 | 0xfbf6 | 0xfc01 | 0xfc0f | 0xfc30 | 0xfc37 | 0xfc62
            | 0xfc6e | 0xfc7a | 0xfc88 | 0xfc92 | 0xfc9e | 0xfcac | 0xfcb6 | 0xfcc2 | 0xfcd5
            | 0xfce4 => {
                w.mask(
                    match self.pc {
                        0xfbf6 => 0xfc,
                        0xfc37 => 0xfff8,
                        _ => 255,
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0xfc65 | 0xfc71 | 0xfc7d | 0xfc95 | 0xfca1 | 0xfcb9 | 0xfcc5 | 0xfce7 | 0xf120 => {
                w.drain(8 * u32::from(m));
                w.load(
                    (w.cpu.a | if self.pc == 0xf120 { 0x90 } else { 0x2400 }) & mask,
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0xfb3e | 0xdfaf | 0xdeb0 => {
                w.drain(16);
                let v = w.read(w.absolute(0x202), x);
                w.cpu.x = v;
                w.flags(v, x);
                self.pc + 3
            }
            0xe3b6 | 0xdecb => {
                w.drain(16);
                let v = w.read(w.absolute(0x202), m);
                w.load(v, m);
                self.pc + 3
            }
            0xe39d | 0xe3a1 | 0xdeb3 | 0xdfb2 | 0xfaff | 0xfb23 | 0xfb2f | 0xfb43 | 0xfb59
            | 0xfbb0 | 0xfbda | 0xfbde | 0xfbe7 | 0xfbed | 0xfbfd | 0xfc0b | 0xfc2c | 0xfc57
            | 0xfc84 | 0xfca8 | 0xfcd1 => {
                let a = match self.pc {
                    0xe39d | 0xfaff => 0x7ef340,
                    0xe3a1 => 0x7ef341,
                    0xdeb3 | 0xfb43 => 0x7ef33f,
                    0xdfb2 => 0x0dfa15,
                    0xfb23 | 0xfb2f | 0xfca8 => 0x7ef377,
                    0xfb59 => 0x7ef35b,
                    0xfbb0 | 0xfbda | 0xfbfd => 0x7ef36c,
                    0xfbde | 0xfbe7 | 0xfbed => 0x7ef36d,
                    0xfc0b => 0x7ef37b,
                    0xfc2c => 0x7ef36e,
                    0xfc57 => 0x7ef362,
                    0xfc84 => 0x7ef343,
                    _ => 0x7ef36f,
                };
                w.drain(24);
                let v = w.read(
                    a + if matches!(self.pc, 0xe3a1 | 0xdeb3 | 0xdfb2 | 0xfb43 | 0xfb59) {
                        u32::from(w.cpu.x)
                    } else {
                        0
                    },
                    m,
                );
                if self.pc == 0xe3a1 {
                    w.load((w.cpu.a | v) & mask, m);
                } else if matches!(self.pc, 0xfbde | 0xfbe7) {
                    let a = w.cpu.a & mask;
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                    w.flags(a.wrapping_sub(v) & mask, m);
                } else {
                    w.load(v, m);
                }
                self.pc + 4
            }
            0xfb67 | 0xfc3b | 0xfc42 | 0xfc49 | 0xfc50 | 0xf102 | 0xf108 => {
                let a = match self.pc {
                    0xfb67 => 0xfa93,
                    0xfc3b => 0xfdef,
                    0xfc42 => 0xfdf1,
                    0xfc49 => 0xfdf3,
                    0xfc50 => 0xfdf5,
                    _ => 0xf9f9,
                };
                let i = if matches!(self.pc, 0xf102 | 0xf108) {
                    w.cpu.y
                } else {
                    w.cpu.x
                };
                w.drain(16);
                if x == 2 || (a & 255) + i > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(i), m);
                if self.pc == 0xf102 {
                    let a = w.cpu.a & mask;
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                    w.flags(a.wrapping_sub(v) & mask, m);
                } else if self.pc == 0xf108 {
                    w.sbc(v, m);
                } else {
                    w.load(v, m);
                }
                self.pc + 3
            }
            0xfb72 | 0xfb7a | 0xfb82 | 0xfb8a => {
                let p = w.dp_read(4, 2);
                if x == 2 || (p & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(p) + u32::from(w.cpu.y), m);
                w.load(v, m);
                self.pc + 2
            }
            0xfde9 => {
                let a = w.indirect_y_address(10);
                let v = w.read(a, m);
                w.load(v, m);
                self.pc + 2
            }
            0xfdec => {
                let a = w.indirect_y_address(7);
                w.write(a, w.cpu.a & mask, m);
                self.pc + 2
            }
            0xfb6c | 0xfc60 | 0xfc6c | 0xfc78 | 0xfc90 | 0xfc9c | 0xfcb4 | 0xfcc0 | 0xfce2
            | 0xfdae | 0xfde1 => {
                let a = match self.pc {
                    0xfb6c => 2,
                    0xfc60 => 3,
                    0xfc6c | 0xfc90 | 0xfcb4 => 4,
                    0xfdae => 0,
                    0xfde1 => 7,
                    _ => 5,
                };
                let v = w.dp_read(a, m);
                w.load(v, m);
                self.pc + 2
            }
            0xe3bd | 0xded5 | 0xdfb6 => {
                w.drain(16);
                w.write(
                    w.absolute(if self.pc == 0xdfb6 { 0x303 } else { 0x202 }),
                    w.cpu.a & mask,
                    m,
                );
                self.pc + 3
            }
            0xe3ac | 0xe3af | 0xe3b2 | 0xf0f9 => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0xe3ac => 0x202,
                        0xe3af => 0x203,
                        0xe3b2 => 0x204,
                        _ => 3,
                    }),
                    0,
                    m,
                );
                self.pc + 3
            }
            0xfa68 | 0xfb0b | 0xfb11 | 0xfb17 | 0xfb1d | 0xfb38 | 0xfb74 | 0xfb7c | 0xfb84
            | 0xfb8c | 0xfc1a | 0xfc21 | 0xfc28 | 0xfc3e | 0xfc45 | 0xfc4c | 0xfc53 | 0xfc68
            | 0xfc74 | 0xfc80 | 0xfc98 | 0xfca4 | 0xfcbc | 0xfcc8 | 0xfcea | 0xfcf3 => {
                let a = match self.pc {
                    0xfa68 => 0x7ef36f,
                    0xfb0b => 0x7ec71e,
                    0xfb11 => 0x7ec71f,
                    0xfb17 => 0x7ec720,
                    0xfb1d => 0x7ec721,
                    0xfb38 => 0x7ef340,
                    0xfb74 => 0x7ec74a,
                    0xfb7c => 0x7ec74c,
                    0xfb84 => 0x7ec78a,
                    0xfb8c => 0x7ec78c,
                    0xfc1a => 0x7ec704,
                    0xfc21 => 0x7ec706,
                    0xfc28 => 0x7ec708,
                    0xfc3e => 0x7ec746,
                    0xfc45 => 0x7ec786,
                    0xfc4c => 0x7ec7c6,
                    0xfc53 => 0x7ec806,
                    0xfc68 => 0x7ec750,
                    0xfc74 => 0x7ec752,
                    0xfc80 => 0x7ec754,
                    0xfc98 => 0x7ec758,
                    0xfca4 => 0x7ec75a,
                    0xfcbc => 0x7ec75e,
                    0xfcc8 => 0x7ec760,
                    0xfcea => 0x7ec764,
                    _ => 0x7ec724,
                };
                w.drain(24);
                w.write(a, w.cpu.a & mask, m);
                self.pc + 4
            }
            0xfb61 | 0xfb6a | 0xfb98 | 0xfb9c | 0xfba0 | 0xfba4 | 0xfba8 | 0xfbac | 0xfbb7
            | 0xfbb9 | 0xfbbb | 0xfbc4 | 0xfbc8 | 0xfbcc | 0xfbd0 | 0xfbd4 | 0xfbd8 | 0xfbf9
            | 0xfbfb | 0xfc04 | 0xfccf | 0xfdb8 | 0xfde7 | 0xf114 => {
                let a = match self.pc {
                    0xfb61 | 0xfbb9 | 0xfc04 => 2,
                    0xfb6a | 0xfbbb | 0xfbfb => 4,
                    0xfb98 | 0xfbc4 => 10,
                    0xfb9c | 0xfbc8 => 11,
                    0xfba0 | 0xfbcc => 12,
                    0xfba4 | 0xfbd0 | 0xfde7 => 7,
                    0xfba8 | 0xfbd4 => 8,
                    0xfbac | 0xfbd8 => 9,
                    0xfccf | 0xf114 => 5,
                    _ => 0,
                };
                w.dp_store(a, w.cpu.a & mask, m);
                self.pc + 2
            }
            0xfa90 => {
                w.dp_increment(0x16, m, true);
                self.pc + 2
            }
            0xf10b => {
                w.operand_then_internal(1);
                let a = 3 + u32::from(w.cpu.x);
                let v = w.read(a, m).wrapping_add(1) & mask;
                w.drain(6);
                w.flags(v, m);
                w.write(a, v, m);
                self.pc + 2
            }
            0xf11a | 0xf122 => {
                w.operand_then_internal(1);
                let a = 3 + u32::from(w.cpu.x);
                if self.pc == 0xf11a {
                    let v = w.read(a, m);
                    w.load(v, m);
                } else {
                    w.write(a, w.cpu.a & mask, m);
                }
                self.pc + 2
            }
            0xe3a5 | 0xf124 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & index_mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xf110 | 0xf111 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & index_mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xfdc0 | 0xfdc1 | 0xf10f => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & index_mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xfb78 | 0xfb79 | 0xfb80 | 0xfb81 | 0xfb88 | 0xfb89 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & index_mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xdece => {
                w.increment_a(m);
                self.pc + 1
            }
            0xfb64 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1) & mask, m);
                self.pc + 1
            }
            0xfb65 | 0xfb6e | 0xfb6f | 0xfb70 => {
                w.shift_left(m);
                self.pc + 1
            }
            0xfb37 | 0xfb63 => {
                w.drain(6);
                w.load(w.cpu.x, m);
                self.pc + 1
            }
            0xfb57 | 0xfdeb => {
                w.drain(6);
                w.cpu.y = w.cpu.x;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xfb58 | 0xfb66 | 0xfc3a => {
                w.drain(6);
                w.cpu.x = w.cpu.a & index_mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xfb60 => {
                w.drain(6);
                w.cpu.x = w.cpu.y;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xfb71 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & index_mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xdeb9 | 0xfbf1 | 0xfc33 | 0xfde3 => {
                w.clear_carry();
                self.pc + 1
            }
            0xdebb | 0xfbe4 | 0xf107 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0xfbe5 | 0xfdb5 => {
                w.drain(8 * u32::from(m));
                w.sbc(if self.pc == 0xfbe5 { 4 } else { 8 }, m);
                self.pc + 1 + u16::from(m)
            }
            0xfbf2 | 0xfc34 | 0xfde4 => {
                w.drain(8 * u32::from(m));
                let v = match self.pc {
                    0xfbf2 => 3,
                    0xfc34 => 7,
                    _ => 0x40,
                };
                if m == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                self.pc + 1 + u16::from(m)
            }
            0xe3a6 | 0xe3aa | 0xe3b9 | 0xe3c3 | 0xdeb7 | 0xded1 | 0xdee8 | 0xfa83 | 0xfb03
            | 0xfb07 | 0xfb27 | 0xfb2b | 0xfb33 | 0xfb41 | 0xfb4d | 0xfb55 | 0xfbe2 | 0xfbeb
            | 0xfc15 | 0xfcdb | 0xfcf1 | 0xfdb3 | 0xfdc2 | 0xfdc7 | 0xfdcc | 0xfdd1 | 0xfdd6
            | 0xfddc | 0xf105 | 0xf10d | 0xf112 | 0xf11e | 0xf125 => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let n = w.cpu.p & NEGATIVE != 0;
                let (t, a) = match self.pc {
                    0xe3a6 => (!n, 0xe3a1),
                    0xe3aa => (!z, 0xe3b6),
                    0xe3b9 => (!z, 0xe3c0),
                    0xe3c3 => (c, 0xe3b5),
                    0xdeb7 => (!z, 0xdebb),
                    0xded1 => (!c, 0xded5),
                    0xdee8 => (!c, 0xdee2),
                    0xfa83 => (true, 0xfa8a),
                    0xfb03 => (z, 0xfb3c),
                    0xfb07 => (!c, 0xfb2d),
                    0xfb27 | 0xfb33 => (!z, 0xfb37),
                    0xfb2b => (true, 0xfb37),
                    0xfb41 => (z, 0xfb90),
                    0xfb4d => (!z, 0xfb52),
                    0xfb55 => (!z, 0xfb61),
                    0xfbe2 => (z, 0xfbed),
                    0xfbeb => (c, 0xfbed),
                    0xfc15 => (!c, 0xfc2c),
                    0xfcdb => (z, 0xfce0),
                    0xfcf1 => (!z, 0xfcf7),
                    0xfdb3 => (!c, 0xfdc4),
                    0xfdc2 => (true, 0xfdae),
                    0xfdc7 => (!c, 0xfdce),
                    0xfdcc | 0xfdd6 => (true, 0xfdd9),
                    0xfdd1 => (!c, 0xfdd8),
                    0xfddc => (!c, 0xfde9),
                    0xf105 => (!c, 0xf10f),
                    0xf10d => (true, 0xf102),
                    0xf112 => (!n, 0xf102),
                    0xf11e => (z, 0xf122),
                    _ => (!n, 0xf11a),
                };
                if w.branch(t) {
                    a
                } else {
                    self.pc + 2
                }
            }
            _ => unreachable!("load HUD continuation {:04x}", self.pc),
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

    #[test]
    fn load_hud_native_tables_match_immutable_source_data() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut bytes = 0;
        for address in 0x0d8000..=0x0dffff {
            if let Some(value) = crate::ZeldaState::native_load_hud_table_byte(address) {
                assert_eq!(
                    value,
                    rom[13 * 0x8000 + (address as usize & 0x7fff)],
                    "HUD table byte {address:06x}"
                );
                bytes += 1;
            }
        }
        assert_eq!(bytes, 992);
    }

    #[test]
    fn load_hud_matches_source_inventory_hearts_magic_and_counters() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        // Each fixture starts both machines independently at the public HUD
        // entry. This proves the component, not its production caller or NMI.
        let mut fixtures = Vec::new();
        let mut base = vec![0u8; 0x20000];
        for (i, b) in base.iter_mut().enumerate() {
            *b = (i ^ (i >> 8)) as u8;
        }
        base[0xf340..0xf37c].fill(0);
        base[0x202..0x205].fill(0);
        base[0x204] = 0x80;
        base[0xf36c] = 160;
        base[0xf36d] = 160;
        base[0xf36f] = 255;
        base[0x1fb..0x1fe].copy_from_slice(&[0x59, 0x80, 0]);
        // The empty-inventory search repairs a nonzero high selection byte
        // before the item-box routine uses its 16-bit index. Other fixtures
        // enter that box directly and require an initially valid selection.
        for high in [1,0x55,0xaa,0xff] {
            let mut ram=base.clone();
            ram[0x203]=high;
            fixtures.push((0xe395,ram));
        }
        for equipped in 0..=20u8 {
            let mut ram = base.clone();
            ram[0x202] = equipped;
            fixtures.push((0xdd32, ram.clone()));
            fixtures.push((0xe395, ram.clone()));
            // Single available item forces selection to retain its wrap and
            // carry result through nested calls, including an empty first slot.
            for available in 0..20 {
                let mut sparse = ram.clone();
                sparse[0xf340 + available] = 1;
                fixtures.push((0xe395, sparse));
            }
            ram[0xf340..0xf354].fill(1);
            fixtures.push((0xe395, ram.clone()));
            fixtures.push((0xfa6c, ram));
        }
        for health in 0..=160 {
            let mut ram = base.clone();
            ram[0xf36d] = health;
            fixtures.push((0xfa6c, ram));
        }
        for maximum in [0, 8, 24, 72, 80, 88, 152, 160] {
            let mut ram = base.clone();
            ram[0xf36c] = maximum;
            ram[0xf36d] = maximum;
            fixtures.push((0xfa6c, ram));
        }
        for magic in 0..=128 {
            let mut ram = base.clone();
            ram[0xf36e] = magic;
            ram[0xf37b] = magic & 1;
            fixtures.push((0xfa6c, ram));
        }
        for bow in 0..=4 {
            for arrows in [0, 1, 9, 10, 30, 70, 99] {
                let mut ram = base.clone();
                ram[0x202] = 1;
                ram[0xf340] = bow;
                ram[0xf377] = arrows;
                fixtures.push((0xfa6c, ram));
            }
        }
        for bottle in 1..=4 {
            for contents in 0..=8 {
                let mut ram = base.clone();
                ram[0x202] = 16;
                ram[0xf34f] = bottle;
                ram[0xf35b + bottle as usize] = contents;
                fixtures.push((0xfa6c, ram));
            }
        }
        for rupees in [0u16, 1, 9, 10, 99, 100, 101, 127, 999] {
            for keys in [0, 1, 9, 10, 99, 255] {
                let mut ram = base.clone();
                ram[0xf362..0xf364].copy_from_slice(&rupees.to_le_bytes());
                ram[0xf343] = (rupees % 100) as u8;
                ram[0xf377] = (rupees % 71) as u8;
                ram[0xf36f] = keys;
                fixtures.push((0xfa6c, ram.clone()));
                fixtures.push((0xfa68, ram));
            }
        }
        let mut instructions = 0;
        for (case, (entry, mut ram)) in fixtures.iter().cloned().enumerate() {
            let mut state = crate::ZeldaState::new();
            state.ram.copy_from_slice(&ram);
            state.sync_native_game_state_from_ram();
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&ram);
            let mut native = NativeNmiContext {
                a: 0xab00 | u16::from(ram[0xf36f]),
                x: 0x71,
                y: 0x83,
                d: 0,
                db: 0,
                bank: 13,
                pc: entry,
                sp: 0x1fa,
                p: 0x31,
            };
            machine.cpu.a = native.a;
            machine.cpu.x = native.x;
            machine.cpu.y = native.y;
            machine.cpu.k = native.bank;
            machine.cpu.pc = native.pc;
            machine.cpu.sp = native.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(native.p);
            let (v, h) = if case & 1 == 0 {
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
            let mut work = LoadFileHud::new(entry);
            assert_eq!(work.routine(), 0x0d0000 | u32::from(entry));
            let mut count = 0;
            while !work.complete() {
                let label = format!("load HUD case{case} entry{entry:04x} pc{:04x}", native.pc);
                super::super::native_work::compare_instruction_with_devices(
                    &label,
                    &mut native,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    None,
                    |_, access| match access {
                        Access::Read(address, width)
                            if address & 0xff0000 == 0x0d0000 && address & 0x8000 != 0 =>
                        {
                            let mut value = 0;
                            for byte in 0..width {
                                let address = address + u32::from(byte);
                                value |= u16::from(
                                    crate::ZeldaState::native_load_hud_table_byte(address)
                                        .unwrap_or_else(|| {
                                            panic!("unbound HUD table byte {address:06x}")
                                        }),
                                ) << (byte * 8);
                            }
                            Some(value)
                        }
                        Access::Write(address, value, width) => {
                            let address = if address & 0x400000 == 0 && address & 65535 < 0x2000 {
                                address & 65535
                            } else { address };
                            if !state.native_load_hud_store(address, value, width) {
                                assert!(address < 16 || (0x100..0x200).contains(&address),
                                    "unowned HUD publication {address:06x}");
                                let offset=address as usize;
                                state.ram[offset..offset+usize::from(width)]
                                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                            }
                            // The independent fixture mirror still records the
                            // same native bus store. Compare both after retirement.
                            None
                        }
                        _ => None,
                    },
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                assert_eq!(ram, source.snes().ram, "{label}");
                assert_eq!(state.ram, ram, "native HUD publication {label}");
                assert_eq!(state.game_state.display.hud_tilemap,crate::game_state::HudTilemapState::load_from_ram(&ram),"HUD owner {label}");
                assert_eq!(state.game_state.inventory,crate::game_state::InventoryState::load_from_ram(&ram),"inventory owners {label}");
                assert_eq!(state.game_state.player.follower_link,crate::game_state::FollowerLinkState::load_from_ram(&ram),"player owner {label}");
                work = work.clone();
                count += 1;
                assert!(count < 20000, "{label}");
            }
            assert_eq!((native.bank, native.pc, native.sp), (0, 0x805a, 0x1fd));
            instructions += count;
        }
        eprintln!(
            "{} load HUD component fixtures matched {instructions} instructions",
            fixtures.len()
        );
    }
}
