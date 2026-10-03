//! Load-game player state, follower history, camera cache and bed setup.
//! Each publication and nested return retains its live CPU/WRAM operands.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

/// Address bindings for existing native semantic tables. Source ROM is never
/// part of this continuation's bus.
pub(crate) fn data_byte(address: u32) -> Option<u8> {
    match address {
        0x02894c..=0x028953 => {
            crate::ZeldaState::native_dungeon_spiral_background((address - 0x02894c) as usize)
        }
        0x099ef8..=0x099efb => Some(
            crate::game_state::FollowerLinkState::oam_priority_for_floor_value(
                (address - 0x099ef8) as u8,
            ),
        ),
        0x08806f..=0x0880b2 => {
            crate::ZeldaState::native_ancilla_sprite_count((address - 0x08806f) as usize)
        }
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Call {
    entry: u32,
    returned: u32,
    far: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoadFilePlayer {
    pc: u32,
    entry: u32,
    calls: Vec<Call>,
    complete: bool,
}
impl LoadFilePlayer {
    pub(crate) fn from_entry(entry: u32) -> Self {
        assert!(matches!(
            entry,
            0x01fec1
                | 0x028aef
                | 0x0ffd22
                | 0x028d71
                | 0x0afd90
                | 0x099efc
                | 0x079a2c
                | 0x0dfa33
                | 0x09ac6b
                | 0x098091
                | 0x09ad06
        ));
        Self {
            pc: entry,
            entry,
            calls: Vec::new(),
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        self.calls.last().map_or(self.entry, |c| c.entry)
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
            (
                (u32::from(cpu.bank) << 16) | u32::from(cpu.pc),
                cpu.d,
                cpu.p & 8
            ),
            (self.pc, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x1fec1 | 0x1fec7 | 0x1fecd => {
                w.drain(8);
                let v = match self.pc {
                    0x1fec1 => 0x20,
                    0x1fec7 => 0x40,
                    0x1fecd => 0x80,
                    _ => unreachable!(),
                };
                w.load((w.cpu.a | v) & 255, 1);
                self.pc + 2
            }
            0x1fec3 | 0x1fec9 | 0x1fecf | 0x28b06 | 0x28b1b | 0x28b1d | 0x28b2b | 0x99f16
            | 0x79a3f | 0xdfa55 | 0x9acd8 | 0x9acde | 0x9ace4 | 0x9809f => {
                w.dp_store(
                    match self.pc {
                        0x1fec3 => 0x9c,
                        0x1fec9 => 0x9d,
                        0x1fecf => 0x9e,
                        0x28b06 | 0xdfa55 => 0x1d,
                        0x28b1b => 0x29,
                        0x28b1d => 0xc7,
                        0x28b2b | 0x79a3f | 0x9acd8 => 0x5d,
                        0x99f16 => 0x0,
                        0x9acde => 0x3a,
                        0x9ace4 => 0x50,
                        0x9809f => 0xe,
                        _ => unreachable!(),
                    },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 2
            }
            0x1fec5 | 0x1fecb | 0x9ac86 | 0x9acaf | 0x9acdc | 0x9ace2 | 0x9ace9 => {
                w.drain(8);
                let v = match self.pc {
                    0x1fec5 | 0x1fecb => 0x1f,
                    0x9ac86 => 0x80,
                    0x9acaf => 0x10,
                    0x9acdc => 0xbf,
                    0x9ace2 => 0xfe,
                    0x9ace9 => 0xfb,
                    _ => unreachable!(),
                };
                w.load((w.cpu.a & v) & 255, 1);
                self.pc + 2
            }
            0x1fed1 | 0xffd3b | 0xafda7 | 0x99f38 | 0x79a53 | 0xdfa57 | 0x9acf2 | 0x980c7 => {
                w.rtl();
                if let Some(call) = self.calls.pop() {
                    assert_eq!(call.far, true);
                    assert_eq!(
                        (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc),
                        call.returned
                    );
                } else {
                    self.complete = true;
                }
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x28aef | 0x28afa | 0xffd28 | 0xffd2f | 0xffd37 => {
                w.drain(8);
                let v = match self.pc {
                    0x28aef => 0x16,
                    0x28afa | 0xffd2f => 0x17,
                    0xffd28 => 0x0,
                    0xffd37 => 0x1c,
                    _ => unreachable!(),
                };
                w.cpu.y = v;
                w.flags(v, 1);
                self.pc + 2
            }
            0x28af1 => {
                w.drain(16);
                let a = w.absolute(0x414);
                let v = w.read(a, 1);
                w.cpu.x = v;
                w.flags(v, 1);
                self.pc + 3
            }
            0x28af4 | 0x9809b => {
                w.drain(24);
                let a = match self.pc {
                    0x28af4 => 0x2894c,
                    0x9809b => 0x8806f,
                    _ => unreachable!(),
                } + u32::from(w.cpu.x);
                let v = w.read(a, 1);
                w.load(v, 1);
                self.pc + 4
            }
            0x28af8 | 0x9ac91 | 0x9acaa => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    (match self.pc {
                        0x28af8 => 0x28afe,
                        0x9ac91 => 0x9ac9c,
                        0x9acaa => 0x9ac70,
                        _ => unreachable!(),
                    }) as u32
                } else {
                    self.pc + 2
                }
            }
            0x28afc | 0x28b02 | 0x28b19 | 0x28b29 | 0x99f24 | 0x79a3d | 0x79a47 | 0x79a4c
            | 0xdfa53 | 0x9acd6 => {
                w.drain(8);
                let v = match self.pc {
                    0x28afc | 0x28b29 | 0x9acd6 => 0x0,
                    0x28b02 | 0x79a47 => 0x3,
                    0x28b19 => 0xff,
                    0x99f24 => 0x40,
                    0x79a3d => 0x16,
                    0x79a4c => 0x20,
                    0xdfa53 => 0x1,
                    _ => unreachable!(),
                };
                w.load(v, 1);
                self.pc + 2
            }
            0x28afe => {
                w.compare_x_immediate(0x2, 1);
                self.pc + 2
            }
            0x28b00 | 0xffd26 | 0xdfa42 | 0xdfa47 | 0x9ac75 | 0x9ac7f | 0x9aca1 | 0x9acd4 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    (match self.pc {
                        0x28b00 => 0x28b04,
                        0xffd26 => 0xffd3b,
                        0xdfa42 | 0xdfa47 => 0xdfa57,
                        0x9ac75 => 0x9ac7a,
                        0x9ac7f => 0x9ac8e,
                        0x9aca1 => 0x9aca6,
                        0x9acd4 => 0x9acf1,
                        _ => unreachable!(),
                    }) as u32
                } else {
                    self.pc + 2
                }
            }
            0x28b04 | 0xffd39 => {
                w.dp_store(
                    match self.pc {
                        0x28b04 => 0x1c,
                        0xffd39 => 0x5d,
                        _ => unreachable!(),
                    },
                    w.cpu.y & 255,
                    1,
                );
                self.pc + 2
            }
            0x28b08 | 0x28b0c | 0x79a4e => {
                let target = (match self.pc {
                    0x28b08 => 0xdfa33,
                    0x28b0c => 0x9ac6b,
                    0x79a4e => 0x98091,
                    _ => unreachable!(),
                }) as u32;
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(w.cpu.pc + 3, 2);
                self.calls.push(Call {
                    entry: target,
                    returned: self.pc + 4,
                    far: true,
                });
                target
            }
            0x28b10 | 0xffd2a | 0x28dfb | 0xdfa3f | 0xdfa44 | 0xdfa4c | 0x9ac8e | 0x9acac
            | 0x9ace6 | 0x980ac => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x28b10 => 0x372,
                    0xffd2a => 0x2e0,
                    0x28dfb | 0x980ac => 0x476,
                    0xdfa3f => 0x458,
                    0xdfa44 => 0x45a,
                    0xdfa4c => 0x414,
                    0x9ac8e => 0x308,
                    0x9acac | 0x9ace6 => 0x37a,
                    _ => unreachable!(),
                });
                let v = w.read(a, 1);
                w.load(v, 1);
                self.pc + 3
            }
            0x28b13 | 0xffd2d | 0xffd35 | 0xdfa37 | 0xdfa3d | 0xdfa51 | 0x9ac88 | 0x9ac98
            | 0x9acb1 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    (match self.pc {
                        0x28b13 => 0x28b2d,
                        0xffd2d | 0xffd35 => 0xffd39,
                        0xdfa37 | 0xdfa3d | 0xdfa51 => 0xdfa57,
                        0x9ac88 => 0x9ac8e,
                        0x9ac98 => 0x9aca9,
                        0x9acb1 => 0x9acb8,
                        _ => unreachable!(),
                    }) as u32
                } else {
                    self.pc + 2
                }
            }
            0x28b15 | 0x28b17 | 0x28b1f | 0x28b21 | 0x28d74 | 0x28d79 | 0x99f35 | 0x9ac8a
            | 0x9ac8c | 0x9acb3 => {
                w.dp_store(
                    match self.pc {
                        0x28b15 => 0x4d,
                        0x28b17 | 0x9acb3 => 0x46,
                        0x28b1f => 0x3d,
                        0x28b21 | 0x99f35 | 0x9ac8c => 0x5e,
                        0x28d74 => 0xb0,
                        0x28d79 => 0x11,
                        0x9ac8a => 0x48,
                        _ => unreachable!(),
                    },
                    0,
                    1,
                );
                self.pc + 2
            }
            0x28b23 | 0x28b26 | 0x28d71 | 0x28d76 | 0x28d7b | 0x28d7e | 0x99f29 | 0x99f2c
            | 0x99f2f | 0x99f32 | 0x79a41 | 0x79a44 | 0x9ac81 | 0x9aca3 | 0x9acb5 | 0x9acb8
            | 0x9acbb | 0x9acbe | 0x9acc1 | 0x9acc4 | 0x9acc7 | 0x9acca | 0x9accd | 0x9acee => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x28b23 => 0x32b,
                    0x28b26 => 0x372,
                    0x28d71 => 0x200,
                    0x28d76 => 0x418,
                    0x28d7b => 0x642,
                    0x28d7e => 0x641,
                    0x99f29 => 0x2cf,
                    0x99f2c => 0x2d3,
                    0x99f2f => 0x2d0,
                    0x99f32 => 0x2d6,
                    0x79a41 => 0x37c,
                    0x79a44 => 0x37d,
                    0x9ac81 => 0x646,
                    0x9aca3 => 0x2ec,
                    0x9acb5 => 0x37a,
                    0x9acb8 => 0x3f0,
                    0x9acbb => 0x2f2,
                    0x9acbe => 0x2f3,
                    0x9acc1 => 0x35f,
                    0x9acc4 => 0x3fc,
                    0x9acc7 => 0x37b,
                    0x9acca => 0x3fd,
                    0x9accd => 0x360,
                    0x9acee => 0x37e,
                    _ => unreachable!(),
                });
                w.write(a, 0, 1);
                self.pc + 3
            }
            0x28b2d | 0x28e0e | 0x9ad1a => {
                w.rts();
                if let Some(call) = self.calls.pop() {
                    assert_eq!(call.far, false);
                    assert_eq!(
                        (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc),
                        call.returned
                    );
                } else {
                    self.complete = true;
                }
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0xffd22 | 0x28def | 0x28df5 | 0x28e02 | 0x28e08 | 0x99eff | 0x99f04 | 0x99f09
            | 0x99f0e | 0x99f13 | 0x9ac84 | 0x9acd0 | 0x9acda | 0x9ace0 | 0x980a2 | 0x980a7
            | 0x9ad06 | 0x9ad0b | 0x9ad10 | 0x9ad15 => {
                let v = w.dp_read(
                    match self.pc {
                        0xffd22 | 0x9acd0 => 0x5d,
                        0x28def | 0x99f13 => 0x2f,
                        0x28df5 | 0x980a7 => 0xee,
                        0x28e02 => 0x6c,
                        0x28e08 => 0xa4,
                        0x99eff => 0x20,
                        0x99f04 => 0x21,
                        0x99f09 => 0x22,
                        0x99f0e => 0x23,
                        0x9ac84 => 0x48,
                        0x9acda => 0x3a,
                        0x9ace0 => 0x50,
                        0x980a2 => 0xe,
                        0x9ad06 => 0x0,
                        0x9ad0b => 0x1,
                        0x9ad10 => 0x2,
                        0x9ad15 => 0x3,
                        _ => unreachable!(),
                    },
                    1,
                );
                w.load(v, 1);
                self.pc + 2
            }
            0xffd24 | 0xdfa4f | 0x9ac73 | 0x9ac7d | 0x9acd2 => {
                w.drain(8);
                let v = match self.pc {
                    0xffd24 | 0xdfa4f => 0x2,
                    0x9ac73 => 0x3e,
                    0x9ac7d => 0x2c,
                    0x9acd2 => 0x13,
                    _ => unreachable!(),
                };
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & 255, 1);
                self.pc + 2
            }
            0xffd31 | 0xdfa33 | 0xdfa39 => {
                w.drain(24);
                let a = match self.pc {
                    0xffd31 => 0x7ef357,
                    0xdfa33 => 0x7ef34a,
                    0xdfa39 => 0x7ec005,
                    _ => unreachable!(),
                };
                let v = w.read(a, 1);
                w.load(v, 1);
                self.pc + 4
            }
            0x28d81 | 0xafd90 | 0x79a2f | 0x980b5 => {
                w.width(0x20, true);
                self.pc + 2
            }
            0x28d83 | 0x28d89 | 0x28d8f | 0x28d95 | 0x28de1 | 0x28de7 => {
                let v = w.dp_read(
                    match self.pc {
                        0x28d83 => 0xe2,
                        0x28d89 => 0xe8,
                        0x28d8f => 0x20,
                        0x28d95 => 0x22,
                        0x28de1 => 0xa6,
                        0x28de7 => 0xa9,
                        _ => unreachable!(),
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0x28d85 | 0x28d8b | 0x28d91 | 0x28d97 | 0x28d9e | 0x28da5 | 0x28dac | 0x28db3
            | 0x28dba | 0x28dc1 | 0x28dc8 | 0x28dcf | 0x28dd6 | 0x28ddd | 0x28de3 | 0x28de9
            | 0xafd95 | 0xafd99 | 0xafd9d | 0xafda1 => {
                w.drain(24);
                let a = match self.pc {
                    0x28d85 => 0x7ec180,
                    0x28d8b => 0x7ec182,
                    0x28d91 => 0x7ec184,
                    0x28d97 => 0x7ec186,
                    0x28d9e => 0x7ec188,
                    0x28da5 => 0x7ec18a,
                    0x28dac => 0x7ec18c,
                    0x28db3 => 0x7ec18e,
                    0x28dba => 0x7ec190,
                    0x28dc1 => 0x7ec192,
                    0x28dc8 => 0x7ec194,
                    0x28dcf => 0x7ec196,
                    0x28dd6 => 0x7ec198,
                    0x28ddd => 0x7ec19a,
                    0x28de3 => 0x7ec19c,
                    0x28de9 => 0x7ec19e,
                    0xafd95 => 0x7ec7f2,
                    0xafd99 => 0x7ec832,
                    0xafd9d => 0x7ec7f4,
                    0xafda1 => 0x7ec834,
                    _ => unreachable!(),
                };
                w.write(a, w.cpu.a & 65535, 2);
                self.pc + 4
            }
            0x28d9b | 0x28da2 | 0x28da9 | 0x28db0 | 0x28db7 | 0x28dbe | 0x28dc5 | 0x28dcc
            | 0x28dd3 | 0x28dda => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x28d9b => 0x600,
                    0x28da2 => 0x604,
                    0x28da9 => 0x608,
                    0x28db0 => 0x60c,
                    0x28db7 => 0x610,
                    0x28dbe => 0x612,
                    0x28dc5 => 0x614,
                    0x28dcc => 0x616,
                    0x28dd3 => 0x618,
                    0x28dda => 0x61c,
                    _ => unreachable!(),
                });
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0x28ded | 0xafda5 | 0x79a3b | 0x980c1 => {
                w.width(
                    match self.pc {
                        0x28ded | 0x79a3b | 0x980c1 => 0x20,
                        0xafda5 => 0x30,
                        _ => unreachable!(),
                    },
                    false,
                );
                self.pc + 2
            }
            0x28df1 | 0x28df7 | 0x28dfe | 0x28e04 | 0x28e0a => {
                w.drain(24);
                let a = match self.pc {
                    0x28df1 => 0x7ec1a6,
                    0x28df7 => 0x7ec1a7,
                    0x28dfe => 0x7ec1a8,
                    0x28e04 => 0x7ec1a9,
                    0x28e0a => 0x7ec1aa,
                    _ => unreachable!(),
                };
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 4
            }
            0xafd92 | 0x79a31 | 0x79a36 | 0x980b7 | 0x980bc => {
                w.drain(16);
                let v = match self.pc {
                    0xafd92 => 0x7f,
                    0x79a31 => 0x215a,
                    0x79a36 => 0x940,
                    0x980b7 => 0x2162,
                    0x980bc => 0x938,
                    _ => unreachable!(),
                };
                w.load(v, 2);
                self.pc + 3
            }
            0x99efc | 0x79a2c | 0x9ac6b | 0x98091 => {
                w.drain(6);
                w.push(u16::from(w.cpu.db), 1);
                self.pc + 1
            }
            0x99efd | 0x79a2d | 0x9ac6c | 0x98092 => {
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                self.pc + 1
            }
            0x99efe | 0x99f37 | 0x79a2e | 0x79a52 | 0x9ac6d | 0x9acf1 | 0x98093 | 0x980c6 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                self.pc + 1
            }
            0x99f01 | 0x99f06 | 0x99f0b | 0x99f10 | 0x99f21 | 0x99f26 | 0x79a49 | 0x9aceb => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0x99f01 => 0x1a00,
                    0x99f06 => 0x1a14,
                    0x99f0b => 0x1a28,
                    0x99f10 => 0x1a3c,
                    0x99f21 => 0x1a64,
                    0x99f26 => 0x2d2,
                    0x79a49 => 0x374,
                    0x9aceb => 0x37a,
                    _ => unreachable!(),
                });
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 3
            }
            0x99f15 | 0x99f1d | 0x99f1e => {
                w.drain(6);
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (a & 1) as u8;
                w.load(a >> 1, 1);
                self.pc + 1
            }
            0x99f18 => {
                let v = w.dp_read(0xee, 1);
                w.cpu.y = v;
                w.flags(v, 1);
                self.pc + 2
            }
            0x99f1a => {
                w.drain(16);
                let a: u16 = 0x9ef8;
                if false || (a & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(w.cpu.y), 1);
                w.load(v, 1);
                self.pc + 3
            }
            0x99f1f => {
                let v = w.dp_read(0x0, 1);
                w.load((w.cpu.a | v) & 255, 1);
                self.pc + 2
            }
            0x79a34 | 0x79a39 | 0x980ba | 0x980bf => {
                w.dp_store(
                    match self.pc {
                        0x79a34 => 0x20,
                        0x79a39 => 0x22,
                        0x980ba => 0x0,
                        0x980bf => 0x2,
                        _ => unreachable!(),
                    },
                    w.cpu.a & 65535,
                    2,
                );
                self.pc + 2
            }
            0xdfa49 => {
                w.drain(16);
                let a = w.absolute(0x458);
                let v = w.read(a, 1).wrapping_add(1) & 255;
                w.drain(6);
                w.flags(v, 1);
                w.write(a, v, 1);
                self.pc + 3
            }
            0x9ac6e | 0x98094 => {
                w.drain(8);
                let v = match self.pc {
                    0x9ac6e => 0x5,
                    0x98094 => 0x0,
                    _ => unreachable!(),
                };
                w.cpu.x = v;
                w.flags(v, 1);
                self.pc + 2
            }
            0x9ac70 | 0x9ac7a => {
                w.drain(16);
                let a: u16 = 0xc4a;
                if false || (a & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                self.pc + 3
            }
            0x9ac77 => {
                w.drain(6);
                w.cpu.y = w.cpu.x & 255;
                w.flags(w.cpu.y, 1);
                self.pc + 1
            }
            0x9ac78 | 0x9ac9a => {
                if w.branch(true) {
                    (match self.pc {
                        0x9ac78 => 0x9ac8e,
                        0x9ac9a => 0x9aca6,
                        _ => unreachable!(),
                    }) as u32
                } else {
                    self.pc + 2
                }
            }
            0x9ac93 | 0x9ac9c => {
                w.drain(6);
                w.load(w.cpu.x & 255, 1);
                self.pc + 1
            }
            0x9ac94 | 0x9ac9d => {
                w.increment_a(1);
                self.pc + 1
            }
            0x9ac95 | 0x9ac9e => {
                w.drain(16);
                let a = w.absolute(0x2ec);
                let v = w.read(a, 1);
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & 255, 1);
                self.pc + 3
            }
            0x9aca6 | 0x980b2 => {
                w.operand_then_internal(2);
                let a = w.absolute(match self.pc {
                    0x9aca6 => 0xc4a,
                    0x980b2 => 0x280,
                    _ => unreachable!(),
                }) + u32::from(w.cpu.x);
                w.write(a, 0, 1);
                self.pc + 3
            }
            0x9aca9 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0x98096 | 0x980a4 | 0x980a9 | 0x980af | 0x9ad08 | 0x9ad0d | 0x9ad12 | 0x9ad17 => {
                w.operand_then_internal(2);
                let a = w.absolute(match self.pc {
                    0x98096 => 0xc4a,
                    0x980a4 => 0xc90,
                    0x980a9 => 0xc7c,
                    0x980af => 0x3ca,
                    0x9ad08 => 0xbfa,
                    0x9ad0d => 0xc0e,
                    0x9ad12 => 0xc04,
                    0x9ad17 => 0xc18,
                    _ => unreachable!(),
                }) + u32::from(w.cpu.x);
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 3
            }
            0x98099 => {
                w.drain(6);
                w.push(w.cpu.x, 1);
                self.pc + 1
            }
            0x9809a => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0x980a1 => {
                w.drain(12);
                w.cpu.x = w.pull(1);
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0x980c3 => {
                let target = (0x9ad06) as u32;
                w.operand_then_internal(2);
                w.push(w.cpu.pc + 2, 2);
                self.calls.push(Call {
                    entry: target,
                    returned: self.pc + 3,
                    far: false,
                });
                target
            }
            _ => unreachable!("load-game player continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc as u16;
        w.cpu.bank = (self.pc >> 16) as u8;
        Some(w.cycles)
    }
}
#[cfg(test)]
#[path = "load_file_player_tests.rs"]
mod tests;
#[cfg(test)]
pub(super) const OWNED_POSITIONS: &[u32] = &[
    0x1fec1, 0x1fec3, 0x1fec5, 0x1fec7, 0x1fec9, 0x1fecb, 0x1fecd, 0x1fecf, 0x1fed1, 0x28aef,
    0x28af1, 0x28af4, 0x28af8, 0x28afa, 0x28afc, 0x28afe, 0x28b00, 0x28b02, 0x28b04, 0x28b06,
    0x28b08, 0x28b0c, 0x28b10, 0x28b13, 0x28b15, 0x28b17, 0x28b19, 0x28b1b, 0x28b1d, 0x28b1f,
    0x28b21, 0x28b23, 0x28b26, 0x28b29, 0x28b2b, 0x28b2d, 0x28d71, 0x28d74, 0x28d76, 0x28d79,
    0x28d7b, 0x28d7e, 0x28d81, 0x28d83, 0x28d85, 0x28d89, 0x28d8b, 0x28d8f, 0x28d91, 0x28d95,
    0x28d97, 0x28d9b, 0x28d9e, 0x28da2, 0x28da5, 0x28da9, 0x28dac, 0x28db0, 0x28db3, 0x28db7,
    0x28dba, 0x28dbe, 0x28dc1, 0x28dc5, 0x28dc8, 0x28dcc, 0x28dcf, 0x28dd3, 0x28dd6, 0x28dda,
    0x28ddd, 0x28de1, 0x28de3, 0x28de7, 0x28de9, 0x28ded, 0x28def, 0x28df1, 0x28df5, 0x28df7,
    0x28dfb, 0x28dfe, 0x28e02, 0x28e04, 0x28e08, 0x28e0a, 0x28e0e, 0x79a2c, 0x79a2d, 0x79a2e,
    0x79a2f, 0x79a31, 0x79a34, 0x79a36, 0x79a39, 0x79a3b, 0x79a3d, 0x79a3f, 0x79a41, 0x79a44,
    0x79a47, 0x79a49, 0x79a4c, 0x79a4e, 0x79a52, 0x79a53, 0x98091, 0x98092, 0x98093, 0x98094,
    0x98096, 0x98099, 0x9809a, 0x9809b, 0x9809f, 0x980a1, 0x980a2, 0x980a4, 0x980a7, 0x980a9,
    0x980ac, 0x980af, 0x980b2, 0x980b5, 0x980b7, 0x980ba, 0x980bc, 0x980bf, 0x980c1, 0x980c3,
    0x980c6, 0x980c7, 0x99efc, 0x99efd, 0x99efe, 0x99eff, 0x99f01, 0x99f04, 0x99f06, 0x99f09,
    0x99f0b, 0x99f0e, 0x99f10, 0x99f13, 0x99f15, 0x99f16, 0x99f18, 0x99f1a, 0x99f1d, 0x99f1e,
    0x99f1f, 0x99f21, 0x99f24, 0x99f26, 0x99f29, 0x99f2c, 0x99f2f, 0x99f32, 0x99f35, 0x99f37,
    0x99f38, 0x9ac6b, 0x9ac6c, 0x9ac6d, 0x9ac6e, 0x9ac70, 0x9ac73, 0x9ac75, 0x9ac77, 0x9ac78,
    0x9ac7a, 0x9ac7d, 0x9ac7f, 0x9ac81, 0x9ac84, 0x9ac86, 0x9ac88, 0x9ac8a, 0x9ac8c, 0x9ac8e,
    0x9ac91, 0x9ac93, 0x9ac94, 0x9ac95, 0x9ac98, 0x9ac9a, 0x9ac9c, 0x9ac9d, 0x9ac9e, 0x9aca1,
    0x9aca3, 0x9aca6, 0x9aca9, 0x9acaa, 0x9acac, 0x9acaf, 0x9acb1, 0x9acb3, 0x9acb5, 0x9acb8,
    0x9acbb, 0x9acbe, 0x9acc1, 0x9acc4, 0x9acc7, 0x9acca, 0x9accd, 0x9acd0, 0x9acd2, 0x9acd4,
    0x9acd6, 0x9acd8, 0x9acda, 0x9acdc, 0x9acde, 0x9ace0, 0x9ace2, 0x9ace4, 0x9ace6, 0x9ace9,
    0x9aceb, 0x9acee, 0x9acf1, 0x9acf2, 0x9ad06, 0x9ad08, 0x9ad0b, 0x9ad0d, 0x9ad10, 0x9ad12,
    0x9ad15, 0x9ad17, 0x9ad1a, 0xafd90, 0xafd92, 0xafd95, 0xafd99, 0xafd9d, 0xafda1, 0xafda5,
    0xafda7, 0xdfa33, 0xdfa37, 0xdfa39, 0xdfa3d, 0xdfa3f, 0xdfa42, 0xdfa44, 0xdfa47, 0xdfa49,
    0xdfa4c, 0xdfa4f, 0xdfa51, 0xdfa53, 0xdfa55, 0xdfa57, 0xffd22, 0xffd24, 0xffd26, 0xffd28,
    0xffd2a, 0xffd2d, 0xffd2f, 0xffd31, 0xffd35, 0xffd37, 0xffd39, 0xffd3b,
];
