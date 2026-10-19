//! Vertical surfaces, pits, stairs and pushing/snapping on the live CPU clock.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    [8, 8, 8]
        .get(address.checked_sub(0x07ba07)? as usize)
        .copied()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VerticalSurfaceTail {
    Returned,
    SurfaceEffect,
    StairDash,
    Damage,
    PushBlock,
    ResetSwim,
    Bonk {
        return_pc: u16,
        saved_collision_low: Option<u8>,
        smash_done: bool,
    },
    Nudge,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VerticalSurfaceResponse {
    pc: Option<u16>,
    tail: Option<VerticalSurfaceTail>,
    bonk_return: u16,
    saved_collision_low: Option<u8>,
}
impl VerticalSurfaceResponse {
    pub(crate) fn tail(&self) -> Option<VerticalSurfaceTail> {
        self.tail
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.tail.is_some() {
            return None;
        }
        let pc = self.pc.unwrap_or(0xbbe6);
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x18),
            (7, pc, 0, 7, 0x10)
        );
        let width = if cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match pc {
            0x91f1 => {
                w.drain(16);
                let v = w.read(w.absolute(0x372u16), width);
                w.load(v, width);
                0x91f4
            }
            0x91f4 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0x91fd
                } else {
                    0x91f6
                }
            }
            0x91f6 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2f1u16), width);
                w.load(v, width);
                0x91f9
            }
            0x91f9 => {
                w.compare_immediate(64, width);
                0x91fb
            }
            0x91fb => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0x9200
                } else {
                    0x91fd
                }
            }
            0x91fd => {
                w.operand_then_internal(2);
                0x9290
            }
            0x9200 => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 7;
                self.tail = Some(VerticalSurfaceTail::Bonk {
                    return_pc: self.bonk_return,
                    saved_collision_low: self.saved_collision_low,
                    smash_done: true,
                });
                0x983a
            }
            0x9290 => {
                w.rts();
                assert!(matches!(
                    w.cpu.pc,
                    0xbde5 | 0xbe3b | 0xbe60 | 0xbde8 | 0xbe3e | 0xbe63 | 0xbdcf | 0xbdfb | 0xbe09
                ));
                w.cpu.pc
            }
            0xbbe6 => {
                w.drain(16);
                let v = w.read(w.absolute(0x36du16), width);
                w.load(v, width);
                0xbbe9
            }
            0xbbe9 => {
                w.mask(7, width);
                0xbbeb
            }
            0xbbeb => {
                w.compare_immediate(7, width);
                0xbbed
            }
            0xbbed => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbc0a
                } else {
                    0xbbef
                }
            }
            0xbbef => {
                w.item_call();
                self.tail = Some(VerticalSurfaceTail::SurfaceEffect);
                0xc16d
            }
            0xbc0a => {
                w.drain(16);
                let v = w.read(w.absolute(0x341u16), width);
                w.load(v, width);
                0xbc0d
            }
            0xbc0d => {
                w.mask(7, width);
                0xbc0f
            }
            0xbc0f => {
                w.compare_immediate(7, width);
                0xbc11
            }
            0xbc11 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbc4d
                } else {
                    0xbc13
                }
            }
            0xbc13 => {
                w.drain(16);
                let v = w.read(w.absolute(0x345u16), width);
                w.load(v, width);
                0xbc16
            }
            0xbc16 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbc4d
                } else {
                    0xbc18
                }
            }
            0xbc18 => {
                w.item_call();
                self.tail = Some(VerticalSurfaceTail::SurfaceEffect);
                0x9195
            }
            0xbc4d => {
                w.drain(16);
                let v = w.read(w.absolute(0x343u16), width);
                w.load(v, width);
                0xbc50
            }
            0xbc50 => {
                w.mask(2, width);
                0xbc52
            }
            0xbc52 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbc8b
                } else {
                    0xbc54
                }
            }
            0xbc54 => {
                w.drain(16);
                let v = w.read(w.absolute(0x345u16), width);
                w.load(v, width);
                0xbc57
            }
            0xbc57 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbc8b
                } else {
                    0xbc59
                }
            }
            0xbc59 => {
                w.drain(8);
                let v = w.read(u32::from(0x4du16), width);
                w.load(v, width);
                0xbc5b
            }
            0xbc5b => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbc63
                } else {
                    0xbc5d
                }
            }
            0xbc5d => {
                w.drain(8 * u32::from(width));
                let v = 7;
                w.load(v, width);
                0xbc5f
            }
            0xbc5f => {
                w.drain(8);
                let address = u32::from(0xeu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbc61
            }
            0xbc61 => {
                w.branch(true);
                0xbc8b
            }
            0xbc63 => {
                w.item_call();
                self.tail = Some(VerticalSurfaceTail::SurfaceEffect);
                0x9195
            }
            0xbc8b => {
                w.drain(8);
                let v = w.read(u32::from(0x58u16), width);
                w.load(v, width);
                0xbc8d
            }
            0xbc8d => {
                w.mask(7, width);
                0xbc8f
            }
            0xbc8f => {
                w.compare_immediate(7, width);
                0xbc91
            }
            0xbc91 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbcc7
                } else {
                    0xbc93
                }
            }
            0xbc93 => {
                w.drain(8);
                let v = w.read(u32::from(0x46u16), width);
                w.load(v, width);
                0xbc95
            }
            0xbc95 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbca0
                } else {
                    0xbc97
                }
            }
            0xbc97 => {
                w.drain(8);
                let v = w.read(u32::from(0x58u16), width);
                w.load(v, width);
                0xbc99
            }
            0xbc99 => {
                w.mask(7, width);
                0xbc9b
            }
            0xbc9b => {
                w.drain(8);
                let address = u32::from(0xeu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbc9d
            }
            0xbc9d => {
                w.operand_then_internal(2);
                0xbdb1
            }
            0xbca0 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2c0u16), width);
                w.load(v, width);
                0xbca3
            }
            0xbca3 => {
                w.mask(119, width);
                0xbca5
            }
            0xbca5 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbcb8
                } else {
                    0xbca7
                }
            }
            0xbca7 => {
                w.item_y(8, 1);
                0xbca9
            }
            0xbca9 => {
                w.mask(112, width);
                0xbcab
            }
            0xbcab => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbcaf
                } else {
                    0xbcad
                }
            }
            0xbcad => {
                w.item_y(16, 1);
                0xbcaf
            }
            0xbcaf => {
                w.drain(8);
                let address = u32::from(0x11u16);
                w.write(address, w.cpu.y, 1);
                0xbcb1
            }
            0xbcb1 => {
                w.drain(8 * u32::from(width));
                let v = 7;
                w.load(v, width);
                0xbcb3
            }
            0xbcb3 => {
                w.drain(8);
                let address = u32::from(0x10u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbcb5
            }
            0xbcb5 => {
                w.item_call();
                self.tail = Some(VerticalSurfaceTail::StairDash);
                0x9195
            }
            0xbcb8 => {
                w.drain(8);
                let v = w.read(u32::from(0x66u16), width);
                w.load(v, width);
                0xbcba
            }
            0xbcba => {
                w.mask(2, width);
                0xbcbc
            }
            0xbcbc => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbcc7
                } else {
                    0xbcbe
                }
            }
            0xbcbe => {
                w.drain(8 * u32::from(width));
                let v = 2;
                w.load(v, width);
                0xbcc0
            }
            0xbcc0 => {
                w.drain(8);
                let address = u32::from(0x5eu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbcc2
            }
            0xbcc2 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xbcc4
            }
            0xbcc4 => {
                w.drain(8);
                let address = u32::from(0x57u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbcc6
            }
            0xbcc6 => {
                w.rts();
                self.tail = Some(VerticalSurfaceTail::Returned);
                assert!(matches!(w.cpu.pc, 0xb95f | 0xb97b));
                w.cpu.pc
            }
            0xbcc7 => {
                w.drain(8);
                let v = w.read(u32::from(0x5eu16), width);
                w.load(v, width);
                0xbcc9
            }
            0xbcc9 => {
                w.compare_immediate(2, width);
                0xbccb
            }
            0xbccb => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbcd8
                } else {
                    0xbccd
                }
            }
            0xbccd => {
                w.item_x(16, 1);
                0xbccf
            }
            0xbccf => {
                w.drain(16);
                let v = w.read(w.absolute(0x372u16), width);
                w.load(v, width);
                0xbcd2
            }
            0xbcd2 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbcd6
                } else {
                    0xbcd4
                }
            }
            0xbcd4 => {
                w.item_x(0, 1);
                0xbcd6
            }
            0xbcd6 => {
                w.drain(8);
                let address = u32::from(0x5eu16);
                w.write(address, w.cpu.x, 1);
                0xbcd8
            }
            0xbcd8 => {
                w.drain(8);
                let v = w.read(u32::from(0x57u16), width);
                w.load(v, width);
                0xbcda
            }
            0xbcda => {
                w.compare_immediate(1, width);
                0xbcdc
            }
            0xbcdc => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbce2
                } else {
                    0xbcde
                }
            }
            0xbcde => {
                w.item_x(2, 1);
                0xbce0
            }
            0xbce0 => {
                w.drain(8);
                let address = u32::from(0x57u16);
                w.write(address, w.cpu.x, 1);
                0xbce2
            }
            0xbce2 => {
                w.drain(8);
                let v = w.read(u32::from(0x59u16), width);
                w.load(v, width);
                0xbce4
            }
            0xbce4 => {
                w.mask(5, width);
                0xbce6
            }
            0xbce6 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbd07
                } else {
                    0xbce8
                }
            }
            0xbce8 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbcea
            }
            0xbcea => {
                w.mask(2, width);
                0xbcec
            }
            0xbcec => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbd07
                } else {
                    0xbcee
                }
            }
            0xbcee => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xbcf0
            }
            0xbcf0 => {
                w.compare_immediate(5, width);
                0xbcf2
            }
            0xbcf2 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbd06
                } else {
                    0xbcf4
                }
            }
            0xbcf4 => {
                w.compare_immediate(2, width);
                0xbcf6
            }
            0xbcf6 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbd06
                } else {
                    0xbcf8
                }
            }
            0xbcf8 => {
                w.drain(8 * u32::from(width));
                let v = 9;
                w.load(v, width);
                0xbcfa
            }
            0xbcfa => {
                w.drain(8);
                let address = u32::from(0x5cu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbcfc
            }
            0xbcfc => {
                w.drain(8);
                let address = u32::from(0x5au16);
                w.write(address, 0, width);
                0xbcfe
            }
            0xbcfe => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xbd00
            }
            0xbd00 => {
                w.drain(8);
                let address = u32::from(0x5bu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbd02
            }
            0xbd02 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xbd04
            }
            0xbd04 => {
                w.drain(8);
                let address = u32::from(0x5du16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbd06
            }
            0xbd06 => {
                w.rts();
                self.tail = Some(VerticalSurfaceTail::Returned);
                assert!(matches!(w.cpu.pc, 0xb95f | 0xb97b));
                w.cpu.pc
            }
            0xbd07 => {
                w.drain(8);
                let address = u32::from(0x5au16);
                w.write(address, 0, width);
                0xbd09
            }
            0xbd09 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2e8u16), width);
                w.load(v, width);
                0xbd0c
            }
            0xbd0c => {
                w.mask(7, width);
                0xbd0e
            }
            0xbd0e => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbd49
                } else {
                    0xbd10
                }
            }
            0xbd10 => {
                w.drain(8);
                let v = w.read(u32::from(0x46u16), width);
                w.load(v, width);
                0xbd12
            }
            0xbd12 => {
                w.drain(16);
                let v = w.read(w.absolute(0x31fu16), width);
                w.load((w.cpu.a | v) & if width == 1 { 255 } else { 65535 }, width);
                0xbd15
            }
            0xbd15 => {
                w.drain(8);
                let v = w.read(u32::from(0x55u16), width);
                w.load((w.cpu.a | v) & if width == 1 { 255 } else { 65535 }, width);
                0xbd17
            }
            0xbd17 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbd42
                } else {
                    0xbd19
                }
            }
            0xbd19 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xbd1b
            }
            0xbd1b => {
                w.drain(8);
                let v = w.read(u32::from(0x66u16), 1);
                w.cpu.y = v;
                w.flags(v, 1);
                0xbd1d
            }
            0xbd1d => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbd25
                } else {
                    0xbd1f
                }
            }
            0xbd1f => {
                w.mask(4, width);
                0xbd21
            }
            0xbd21 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbd29
                } else {
                    0xbd23
                }
            }
            0xbd23 => {
                w.branch(true);
                0xbd49
            }
            0xbd25 => {
                w.mask(4, width);
                0xbd27
            }
            0xbd27 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbd49
                } else {
                    0xbd29
                }
            }
            0xbd29 => {
                w.drain(16);
                let v = w.read(w.absolute(0x31fu16), width);
                w.load(v, width);
                0xbd2c
            }
            0xbd2c => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbd49
                } else {
                    0xbd2e
                }
            }
            0xbd2e => {
                w.drain(24);
                let v = w.read(0x7ef35bu32, width);
                w.load(v, width);
                0xbd32
            }
            0xbd32 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                0xbd33
            }
            0xbd33 => {
                w.drain(16);
                if (0xba07u16 + w.cpu.y) & 0xff00 != 0xba00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xba07u16 + w.cpu.y), width);
                w.load(v, width);
                0xbd36
            }
            0xbd36 => {
                w.drain(16);
                let address = w.absolute(0x373u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbd39
            }
            0xbd39 => {
                w.item_call();
                self.tail = Some(VerticalSurfaceTail::Damage);
                0x9195
            }
            0xbd42 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2e8u16), width);
                w.load(v, width);
                0xbd45
            }
            0xbd45 => {
                w.mask(7, width);
                0xbd47
            }
            0xbd47 => {
                w.drain(8);
                let address = u32::from(0xeu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbd49
            }
            0xbd49 => {
                w.drain(16);
                let v = w.read(w.absolute(0x46cu16), width);
                w.load(v, width);
                0xbd4c
            }
            0xbd4c => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbd56
                } else {
                    0xbd4e
                }
            }
            0xbd4e => {
                w.compare_immediate(4, width);
                0xbd50
            }
            0xbd50 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbd56
                } else {
                    0xbd52
                }
            }
            0xbd52 => {
                w.drain(8);
                let v = w.read(u32::from(0xeeu16), width);
                w.load(v, width);
                0xbd54
            }
            0xbd54 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbdb1
                } else {
                    0xbd56
                }
            }
            0xbd56 => {
                w.drain(8);
                let v = w.read(u32::from(0x5fu16), width);
                w.load(v, width);
                0xbd58
            }
            0xbd58 => {
                w.drain(8);
                let v = w.read(u32::from(0x60u16), width);
                w.load((w.cpu.a | v) & if width == 1 { 255 } else { 65535 }, width);
                0xbd5a
            }
            0xbd5a => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbdad
                } else {
                    0xbd5c
                }
            }
            0xbd5c => {
                w.drain(8);
                let v = w.read(u32::from(0x6au16), width);
                w.load(v, width);
                0xbd5e
            }
            0xbd5e => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbdad
                } else {
                    0xbd60
                }
            }
            0xbd60 => {
                w.drain(8);
                let v = w.read(u32::from(0x5fu16), width);
                w.load(v, width);
                0xbd62
            }
            0xbd62 => {
                w.drain(16);
                let address = w.absolute(0x2c2u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbd65
            }
            0xbdad => {
                w.drain(8 * u32::from(width));
                let v = 21;
                w.load(v, width);
                0xbdaf
            }
            0xbdaf => {
                w.drain(8);
                let address = u32::from(0x61u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbdb1
            }
            0xbdb1 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbdb3
            }
            0xbdb3 => {
                w.mask(7, width);
                0xbdb5
            }
            0xbdb5 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbdba
                } else {
                    0xbdb7
                }
            }
            0xbdb7 => {
                w.operand_then_internal(2);
                0xbe99
            }
            0xbdba => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xbdbc
            }
            0xbdbc => {
                w.compare_immediate(4, width);
                0xbdbe
            }
            0xbdbe => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbdd1
                } else {
                    0xbdc0
                }
            }
            0xbdc0 => {
                w.drain(16);
                let v = w.read(w.absolute(0x310u16), width);
                w.load(v, width);
                0xbdc3
            }
            0xbdc3 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbdc8
                } else {
                    0xbdc5
                }
            }
            0xbdc5 => {
                w.item_call();
                self.tail = Some(VerticalSurfaceTail::ResetSwim);
                0x9873
            }
            0xbdc8 => {
                w.drain(8);
                let v = w.read(u32::from(0x6au16), width);
                w.load(v, width);
                0xbdca
            }
            0xbdca => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbdd1
                } else {
                    0xbdcc
                }
            }
            0xbdcc => {
                w.item_call();
                0xc1e4
            }
            0xbdcf => {
                w.branch(true);
                0xbe0f
            }
            0xbdd1 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbdd3
            }
            0xbdd3 => {
                w.mask(2, width);
                0xbdd5
            }
            0xbdd5 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbddf
                } else {
                    0xbdd7
                }
            }
            0xbdd7 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbdd9
            }
            0xbdd9 => {
                w.mask(5, width);
                0xbddb
            }
            0xbddb => {
                w.compare_immediate(5, width);
                0xbddd
            }
            0xbddd => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbdeb
                } else {
                    0xbddf
                }
            }
            0xbddf => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbde1
            }
            0xbde1 => {
                w.drain(6);
                w.push(w.cpu.a & if width == 1 { 255 } else { 65535 }, width);
                self.saved_collision_low = Some(w.cpu.a as u8);
                0xbde2
            }
            0xbde2 => {
                w.item_call();
                self.bonk_return = 0xbde5;
                0xc1a1
            }
            0xbde5 => {
                w.item_call();
                0x91f1
            }
            0xbde8 => {
                w.drain(12);
                let v = w.pull(width);
                w.load(v, width);
                0xbde9
            }
            0xbde9 => {
                w.drain(8);
                let address = u32::from(0xeu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbdeb
            }
            0xbdeb => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xbded
            }
            0xbded => {
                w.drain(16);
                let address = w.absolute(0x302u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbdf0
            }
            0xbdf0 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbdf2
            }
            0xbdf2 => {
                w.mask(2, width);
                0xbdf4
            }
            0xbdf4 => {
                w.compare_immediate(2, width);
                0xbdf6
            }
            0xbdf6 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbdfd
                } else {
                    0xbdf8
                }
            }
            0xbdf8 => {
                w.item_call();
                0xc1e4
            }
            0xbdfb => {
                w.branch(true);
                0xbe0f
            }
            0xbdfd => {
                w.drain(8);
                let v = w.read(u32::from(0x6au16), width);
                w.load(v, width);
                0xbdff
            }
            0xbdff => {
                w.compare_immediate(1, width);
                0xbe01
            }
            0xbe01 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbe06
                } else {
                    0xbe03
                }
            }
            0xbe03 => {
                w.operand_then_internal(2);
                0xbea3
            }
            0xbe06 => {
                w.item_call();
                0xc1e4
            }
            0xbe09 => {
                w.drain(8);
                let v = w.read(u32::from(0x6au16), width);
                w.load(v, width);
                0xbe0b
            }
            0xbe0b => {
                w.compare_immediate(2, width);
                0xbe0d
            }
            0xbe0d => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbe03
                } else {
                    0xbe0f
                }
            }
            0xbe0f => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbe11
            }
            0xbe11 => {
                w.mask(5, width);
                0xbe13
            }
            0xbe13 => {
                w.compare_immediate(5, width);
                0xbe15
            }
            0xbe15 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbe5d
                } else {
                    0xbe17
                }
            }
            0xbe17 => {
                w.mask(4, width);
                0xbe19
            }
            0xbe19 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbe40
                } else {
                    0xbe1b
                }
            }
            0xbe1b => {
                w.item_y(1, 1);
                0xbe1d
            }
            0xbe1d => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                w.load(v, width);
                0xbe1f
            }
            0xbe1f => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xbe24
                } else {
                    0xbe21
                }
            }
            0xbe21 => {
                w.drain(8 * u32::from(width));
                let v = 255;
                w.load((w.cpu.a ^ v) & if width == 1 { 255 } else { 65535 }, width);
                0xbe23
            }
            0xbe23 => {
                w.increment_a(width);
                0xbe24
            }
            0xbe24 => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xbe28
                } else {
                    0xbe26
                }
            }
            0xbe26 => {
                w.item_y(255, 1);
                0xbe28
            }
            0xbe28 => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(address, w.cpu.y, 1);
                0xbe2a
            }
            0xbe2a => {
                w.drain(8);
                let address = u32::from(0x1u16);
                w.write(address, 0, width);
                0xbe2c
            }
            0xbe2c => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbe2e
            }
            0xbe2e => {
                w.mask(2, width);
                0xbe30
            }
            0xbe30 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbe6b
                } else {
                    0xbe32
                }
            }
            0xbe32 => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xbe34
            }
            0xbe34 => {
                w.mask(7, width);
                0xbe36
            }
            0xbe36 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbe65
                } else {
                    0xbe38
                }
            }
            0xbe38 => {
                w.item_call();
                self.bonk_return = 0xbe3b;
                self.saved_collision_low = None;
                0xc1a1
            }
            0xbe3b => {
                w.item_call();
                0x91f1
            }
            0xbe3e => {
                w.branch(true);
                0xbe6b
            }
            0xbe40 => {
                w.item_y(1, 1);
                0xbe42
            }
            0xbe42 => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                w.load(v, width);
                0xbe44
            }
            0xbe44 => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xbe49
                } else {
                    0xbe46
                }
            }
            0xbe46 => {
                w.drain(8 * u32::from(width));
                let v = 255;
                w.load((w.cpu.a ^ v) & if width == 1 { 255 } else { 65535 }, width);
                0xbe48
            }
            0xbe48 => {
                w.increment_a(width);
                0xbe49
            }
            0xbe49 => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xbe4d
                } else {
                    0xbe4b
                }
            }
            0xbe4b => {
                w.item_y(255, 1);
                0xbe4d
            }
            0xbe4d => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(address, w.cpu.y, 1);
                0xbe4f
            }
            0xbe4f => {
                w.drain(8);
                let address = u32::from(0x1u16);
                w.write(address, 0, width);
                0xbe51
            }
            0xbe51 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbe53
            }
            0xbe53 => {
                w.mask(2, width);
                0xbe55
            }
            0xbe55 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbe6b
                } else {
                    0xbe57
                }
            }
            0xbe57 => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xbe59
            }
            0xbe59 => {
                w.mask(7, width);
                0xbe5b
            }
            0xbe5b => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbe65
                } else {
                    0xbe5d
                }
            }
            0xbe5d => {
                w.item_call();
                self.bonk_return = 0xbe60;
                self.saved_collision_low = None;
                0xc1a1
            }
            0xbe60 => {
                w.item_call();
                0x91f1
            }
            0xbe63 => {
                w.branch(true);
                0xbe6b
            }
            0xbe65 => {
                w.item_call();
                self.tail = Some(VerticalSurfaceTail::Nudge);
                0xc229
            }
            0xbe6b => {
                w.drain(8);
                let v = w.read(u32::from(0x66u16), width);
                w.load(v, width);
                0xbe6d
            }
            0xbe6d => {
                w.shift_left(width);
                0xbe6e
            }
            0xbe6e => {
                w.drain(8);
                let v = w.read(u32::from(0x2fu16), width);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old >= v);
                w.flags(
                    old.wrapping_sub(v) & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbe70
            }
            0xbe70 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbea3
                } else {
                    0xbe72
                }
            }
            0xbe72 => {
                w.drain(16);
                let v = w.read(w.absolute(0x315u16), width);
                w.load(v, width);
                0xbe75
            }
            0xbe75 => {
                w.mask(1, width);
                0xbe77
            }
            0xbe77 => {
                w.shift_left(width);
                0xbe78
            }
            0xbe78 => {
                w.drain(8);
                let address = u32::from(0x48u16);
                let old = w.read(address, width);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(
                    address,
                    old | (w.cpu.a & if width == 1 { 255 } else { 65535 }),
                    width,
                );
                0xbe7a
            }
            0xbe7a => {
                w.drain(8);
                let v = w.read(u32::from(0x3cu16), width);
                w.load(v, width);
                0xbe7c
            }
            0xbe7c => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbe83
                } else {
                    0xbe7e
                }
            }
            0xbe7e => {
                w.drain(16);
                let address = w.absolute(0x371u16);
                let old = w.read(address, width);
                let v = old.wrapping_sub(1) & if width == 1 { 255 } else { 65535 };
                w.flags(v, width);
                w.drain(6);
                w.write_high_low(address, v, width);
                0xbe81
            }
            0xbe81 => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xbeae
                } else {
                    0xbe83
                }
            }
            0xbe83 => {
                w.drain(16);
                let v = w.read(w.absolute(0x315u16), 1);
                w.cpu.y = v;
                w.flags(v, 1);
                0xbe86
            }
            0xbe86 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2f6u16), width);
                w.load(v, width);
                0xbe89
            }
            0xbe89 => {
                w.mask(32, width);
                0xbe8b
            }
            0xbe8b => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbe94
                } else {
                    0xbe8d
                }
            }
            0xbe8d => {
                w.drain(16);
                let v = w.read(w.absolute(0x315u16), width);
                w.load(v, width);
                0xbe90
            }
            0xbe90 => {
                w.shift_left(width);
                0xbe91
            }
            0xbe91 => {
                w.shift_left(width);
                0xbe92
            }
            0xbe92 => {
                w.shift_left(width);
                0xbe93
            }
            0xbe93 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                0xbe94
            }
            0xbe94 => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xbe95
            }
            0xbe95 => {
                w.drain(8);
                let address = u32::from(0x48u16);
                let old = w.read(address, width);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(
                    address,
                    old | (w.cpu.a & if width == 1 { 255 } else { 65535 }),
                    width,
                );
                0xbe97
            }
            0xbe97 => {
                w.branch(true);
                0xbea3
            }
            0xbe99 => {
                w.drain(8);
                let v = w.read(u32::from(0xeeu16), width);
                w.load(v, width);
                0xbe9b
            }
            0xbe9b => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbeae
                } else {
                    0xbe9d
                }
            }
            0xbe9d => {
                w.drain(8);
                let v = w.read(u32::from(0x48u16), width);
                w.load(v, width);
                0xbe9f
            }
            0xbe9f => {
                w.mask(246, width);
                0xbea1
            }
            0xbea1 => {
                w.drain(8);
                let address = u32::from(0x48u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbea3
            }
            0xbea3 => {
                w.drain(8 * u32::from(width));
                let v = 32;
                w.load(v, width);
                0xbea5
            }
            0xbea5 => {
                w.drain(16);
                let address = w.absolute(0x371u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbea8
            }
            0xbea8 => {
                w.drain(8);
                let v = w.read(u32::from(0x48u16), width);
                w.load(v, width);
                0xbeaa
            }
            0xbeaa => {
                w.mask(253, width);
                0xbeac
            }
            0xbeac => {
                w.drain(8);
                let address = u32::from(0x48u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbeae
            }
            0xbeae => {
                w.rts();
                self.tail = Some(VerticalSurfaceTail::Returned);
                assert!(matches!(w.cpu.pc, 0xb95f | 0xb97b));
                w.cpu.pc
            }
            0xc19f => {
                w.clear_carry();
                0xc1a0
            }
            0xc1a0 => {
                w.rts();
                assert!(matches!(
                    w.cpu.pc,
                    0xbde5 | 0xbe3b | 0xbe60 | 0xbde8 | 0xbe3e | 0xbe63 | 0xbdcf | 0xbdfb | 0xbe09
                ));
                w.cpu.pc
            }
            0xc1a1 => {
                w.drain(16);
                let v = w.read(w.absolute(0x372u16), width);
                w.load(v, width);
                0xc1a4
            }
            0xc1a4 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xc19f
                } else {
                    0xc1a6
                }
            }
            0xc1a6 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2f1u16), width);
                w.load(v, width);
                0xc1a9
            }
            0xc1a9 => {
                w.compare_immediate(64, width);
                0xc1ab
            }
            0xc1ab => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xc19f
                } else {
                    0xc1ad
                }
            }
            0xc1ad => {
                w.drain(16);
                let v = w.read(w.absolute(0x2efu16), width);
                w.load(v, width);
                0xc1b0
            }
            0xc1b0 => {
                w.mask(112, width);
                0xc1b2
            }
            0xc1b2 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xc19f
                } else {
                    0xc1b4
                }
            }
            0xc1b4 => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 27;
                self.tail = Some(VerticalSurfaceTail::Bonk {
                    return_pc: self.bonk_return,
                    saved_collision_low: self.saved_collision_low,
                    smash_done: false,
                });
                0xc076
            }
            0xc1e4 => {
                w.width(32, true);
                0xc1e6
            }
            0xc1e6 => {
                w.drain(8);
                let v = w.read(u32::from(0x51u16), width);
                w.load(v, width);
                0xc1e8
            }
            0xc1e8 => {
                w.mask(7, width);
                0xc1eb
            }
            0xc1eb => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), 1);
                w.cpu.y = v;
                w.flags(v, 1);
                0xc1ed
            }
            0xc1ed => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xc1f3
                } else {
                    0xc1ef
                }
            }
            0xc1ef => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xc1f0
            }
            0xc1f0 => {
                w.drain(8 * u32::from(width));
                let v = 8;
                w.sbc(v, width);
                0xc1f3
            }
            0xc1f3 => {
                w.drain(8 * u32::from(width));
                let v = 65535;
                w.load((w.cpu.a ^ v) & if width == 1 { 255 } else { 65535 }, width);
                0xc1f6
            }
            0xc1f6 => {
                w.increment_a(width);
                0xc1f7
            }
            0xc1f7 => {
                w.clear_carry();
                0xc1f8
            }
            0xc1f8 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xc1fa
            }
            0xc1fa => {
                w.drain(8);
                let address = u32::from(0x20u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xc1fc
            }
            0xc1fc => {
                w.width(32, false);
                0xc1fe
            }
            0xc1fe => {
                w.rts();
                assert!(matches!(
                    w.cpu.pc,
                    0xbde5 | 0xbe3b | 0xbe60 | 0xbde8 | 0xbe3e | 0xbe63 | 0xbdcf | 0xbdfb | 0xbe09
                ));
                w.cpu.pc
            }
            _ => panic!("vertical surface response PC {pc:04x}"),
        };
        if next == 0xbd65 {
            self.tail = Some(VerticalSurfaceTail::PushBlock);
        }
        w.cpu.pc = next;
        self.pc = Some(next);
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
    fn vertical_surface_matches_source_pits_stairs_snap_and_return() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x07ba07..0x07ba0a {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        assert_eq!(
            table_byte(0x07ba0a),
            None,
            "the following instruction is outside the armor data table"
        );
        let mut instructions = 0;
        let mut tails = std::collections::HashSet::new();

        for timer in 0..=255u8 {
            for variant in 0..32u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7) ^ timer as usize) as u8;
                }
                r[0x1f5..0x1f7].copy_from_slice(&[0x1b, 0x80]);
                r[0x1ef..0x1f1].copy_from_slice(&[0x5e, 0xb9]);
                r[0x36d] = if variant & 3 == 1 { 7 } else { timer & 6 };
                r[0x341] = if variant & 3 == 2 { 7 } else { 0 };
                r[0x343] = if variant & 3 == 3 { 2 } else { 0 };
                r[0x345] = timer & 1;
                r[0x4d] = (timer >> 1) & 1;
                r[0x58] = if variant & 4 != 0 && timer & 7 == 0 {
                    7
                } else {
                    0
                };
                r[0x2c0] = timer & 0x77;
                r[0x46] = (timer >> 3) & 1;
                r[0x5e] = timer & 3;
                r[0x57] = (timer >> 2) & 3;
                r[0x372] = u8::from(variant & 16 != 0);
                r[0x59] = (timer >> 4) & 5;
                r[0x5d] = timer & 7;
                r[0x2e8] = (timer >> 5) & 7;
                r[0x31f] = timer & 1;
                r[0x55] = variant & 2;
                r[0xf35b] = (timer >> 6) % 3;
                r[0x46c] = (variant >> 1) % 5;
                r[0xee] = variant & 1;
                r[0x5f] = if variant & 8 != 0 { timer | 1 } else { 0 };
                r[0x60] = if variant & 8 != 0 { 128 } else { 0 };
                r[0x6a] = variant & 3;
                r[0x310] = timer & 1;
                r[0x0e] = timer;
                r[0x30] = [0, 1, 127, 128, 255][(timer as usize) % 5];
                r[0x66] = variant & 1;
                r[0x2f] = if timer & 2 == 0 { 8 } else { (variant & 1) * 2 };
                r[0x315] = (timer >> 6) & 3;
                r[0x3c] = timer & 7;
                r[0x371] = timer;
                r[0x2f6] = timer & 0x20;
                r[0x2f1] = timer & 0x40;
                r[0x2ef] = timer & 0x70;
                let values = [0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa];
                for (n, address) in [0x20, 0x22, 0x51].into_iter().enumerate() {
                    r[address..address + 2]
                        .copy_from_slice(&values[((timer as usize) + n) % 8].to_le_bytes());
                }
                let pc = 0xbbe6;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: variant as u16,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: 0x1ee,
                    p: 0x30 | (timer & 0xc3),
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.db = cpu.db;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let mut clock = CpuMasterTimeline::at_raster(
                    4586,
                    CpuRasterPosition::new(
                        if variant & 1 == 0 { 22 } else { 261 },
                        if variant & 1 == 0 { 26 } else { 1350 },
                    ),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(variant & 1 != 0),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = VerticalSurfaceResponse::default();
                while work.tail().is_none() {
                    let label = format!(
                        "vertical surface controls={timer} variant={variant} pc={:04x}",
                        cpu.pc
                    );
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| {
                            if let super::super::intro_clear::Access::Read(a, width) = access {
                                let byte = table_byte;
                                let low = byte(a)?;
                                Some(if width == 1 {
                                    u16::from(low)
                                } else {
                                    u16::from_le_bytes([low, byte(a + 1)?])
                                })
                            } else {
                                None
                            }
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                if let Some(VerticalSurfaceTail::Bonk {
                    return_pc,
                    saved_collision_low,
                    smash_done,
                }) = work.tail()
                {
                    let stack_return_at = if saved_collision_low.is_some() {
                        0x1ec
                    } else {
                        0x1ed
                    };
                    let stack_return =
                        u16::from_le_bytes([ram[stack_return_at], ram[stack_return_at + 1]])
                            .wrapping_add(1);
                    assert_eq!(return_pc + if smash_done { 3 } else { 0 }, stack_return);
                    if let Some(value) = saved_collision_low {
                        assert_eq!(value, ram[0x1ee]);
                    }
                    assert_eq!(
                        (cpu.bank, cpu.pc),
                        if smash_done {
                            (7, 0x983a)
                        } else {
                            (0x1b, 0xc076)
                        }
                    );
                }
                let category = match work.tail().unwrap() {
                    VerticalSurfaceTail::Returned => 0,
                    VerticalSurfaceTail::SurfaceEffect => 1,
                    VerticalSurfaceTail::StairDash => 2,
                    VerticalSurfaceTail::Damage => 3,
                    VerticalSurfaceTail::PushBlock => 4,
                    VerticalSurfaceTail::ResetSwim => 5,
                    VerticalSurfaceTail::Bonk {
                        smash_done: false, ..
                    } => 6,
                    VerticalSurfaceTail::Bonk {
                        smash_done: true, ..
                    } => 7,
                    VerticalSurfaceTail::Nudge => 8,
                };
                tails.insert(category);
                assert!(work.tail().is_some());
                assert_eq!(cpu.pc, source.snes().cpu.pc);
            }
        }
        assert_eq!(tails.len(), 9, "reachable tail coverage: {tails:?}");
        println!("vertical surface: 8192 cases, {instructions} source instructions, pits, stairs, byte timers, snap, bonk and return handoffs");
    }
}
