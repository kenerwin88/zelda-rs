//! Vertical collision doorway handling, collision layers and conveyor selection.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    if let Some(i) = address.checked_sub(0x07b7c3) {
        if let Some(v) = [8, 4].get(i as usize) {
            return Some(*v);
        }
    }
    [8, 0, 24, 0, 16, 0, 16, 0, 8, 0, 8, 0, 0, 0, 15, 0]
        .get(address.checked_sub(0x07b9f7)? as usize)
        .copied()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VerticalResponseTail {
    Returned,
    Outdoor,
    Doorway { perpendicular: bool },
    DoorSnap { velocity_done: bool },
    Slope,
    Chest { collision: u16 },
    Rupee { tile_deleted: bool },
    SurfaceTail,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VerticalCollisionResponse {
    pc: Option<u16>,
    tail: Option<VerticalResponseTail>,
    collision: u16,
}
impl VerticalCollisionResponse {
    pub(crate) fn tail(&self) -> Option<VerticalResponseTail> {
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
        let pc = self.pc.unwrap_or(0xba2e);
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
            0xba2e => {
                w.drain(8);
                let v = w.read(u32::from(0x1bu16), width);
                w.load(v, width);
                0xba30
            }
            0xba30 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xba35
                } else {
                    0xba32
                }
            }
            0xba32 => {
                w.operand_then_internal(2);
                self.tail = Some(VerticalResponseTail::Outdoor);
                0xbeaf
            }
            0xba35 => {
                w.drain(16);
                let v = w.read(w.absolute(0x308u16), width);
                w.load(v, width);
                0xba38
            }
            0xba38 => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xba3e
                } else {
                    0xba3a
                }
            }
            0xba3a => {
                w.drain(8);
                let v = w.read(u32::from(0x46u16), width);
                w.load(v, width);
                0xba3c
            }
            0xba3c => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xba49
                } else {
                    0xba3e
                }
            }
            0xba3e => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xba40
            }
            0xba40 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xba41
            }
            0xba41 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xba42
            }
            0xba42 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xba43
            }
            0xba43 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xba44
            }
            0xba44 => {
                w.drain(8);
                let address = u32::from(0xeu16);
                let old = w.read(address, width);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(
                    address,
                    old | (w.cpu.a & if width == 1 { 255 } else { 65535 }),
                    width,
                );
                0xba46
            }
            0xba46 => {
                w.operand_then_internal(2);
                0xbada
            }
            0xba49 => {
                w.drain(8);
                let v = w.read(u32::from(0x6cu16), width);
                w.load(v, width);
                0xba4b
            }
            0xba4b => {
                w.compare_immediate(2, width);
                0xba4d
            }
            0xba4d => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xba70
                } else {
                    0xba4f
                }
            }
            0xba4f => {
                w.drain(8);
                let v = w.read(u32::from(0x6au16), width);
                w.load(v, width);
                0xba51
            }
            0xba51 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xba67
                } else {
                    0xba53
                }
            }
            0xba53 => {
                w.drain(16);
                let v = w.read(w.absolute(0x46cu16), width);
                w.load(v, width);
                0xba56
            }
            0xba56 => {
                w.compare_immediate(3, width);
                0xba58
            }
            0xba58 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xba61
                } else {
                    0xba5a
                }
            }
            0xba5a => {
                w.drain(8);
                let v = w.read(u32::from(0xeeu16), width);
                w.load(v, width);
                0xba5c
            }
            0xba5c => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xba61
                } else {
                    0xba5e
                }
            }
            0xba5e => {
                w.operand_then_internal(2);
                0xbaf4
            }
            0xba61 => {
                w.item_call();
                self.tail = Some(VerticalResponseTail::Doorway {
                    perpendicular: true,
                });
                0xc29f
            }
            0xba64 => {
                w.operand_then_internal(2);
                0xc23d
            }
            0xba67 => {
                w.drain(8);
                let v = w.read(u32::from(0x62u16), width);
                w.load(v, width);
                0xba69
            }
            0xba69 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xba70
                } else {
                    0xba6b
                }
            }
            0xba6b => {
                w.item_call();
                self.tail = Some(VerticalResponseTail::Doorway {
                    perpendicular: false,
                });
                0xc29f
            }
            0xba6e => {
                w.branch(true);
                0xbae3
            }
            0xba70 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xba72
            }
            0xba72 => {
                w.mask(112, width);
                0xba74
            }
            0xba74 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbada
                } else {
                    0xba76
                }
            }
            0xba76 => {
                w.drain(8);
                let address = u32::from(0x5u16);
                w.write(address, 0, width);
                0xba78
            }
            0xba78 => {
                w.drain(8);
                let v = w.read(u32::from(0xfu16), width);
                w.load(v, width);
                0xba7a
            }
            0xba7a => {
                w.mask(7, width);
                0xba7c
            }
            0xba7c => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xba8b
                } else {
                    0xba7e
                }
            }
            0xba7e => {
                w.item_y(0, 1);
                0xba80
            }
            0xba80 => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                w.load(v, width);
                0xba82
            }
            0xba82 => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xba86
                } else {
                    0xba84
                }
            }
            0xba84 => {
                w.item_y(1, 1);
                0xba86
            }
            0xba86 => {
                w.drain(16);
                if (0xb7c3u16 + w.cpu.y) & 0xff00 != 0xb700 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xb7c3u16 + w.cpu.y), width);
                w.load(v, width);
                0xba89
            }
            0xba89 => {
                w.drain(8);
                let address = u32::from(0x49u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xba8b
            }
            0xba8b => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xba8d
            }
            0xba8d => {
                w.drain(8);
                let address = u32::from(0x6cu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xba8f
            }
            0xba8f => {
                w.drain(16);
                let address = w.absolute(0x3f3u16);
                w.write(address, 0, width);
                0xba92
            }
            0xba92 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xba94
            }
            0xba94 => {
                w.mask(112, width);
                0xba96
            }
            0xba96 => {
                w.compare_immediate(112, width);
                0xba98
            }
            0xba98 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbacc
                } else {
                    0xba9a
                }
            }
            0xba9a => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xba9c
            }
            0xba9c => {
                w.mask(5, width);
                0xba9e
            }
            0xba9e => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbaa8
                } else {
                    0xbaa0
                }
            }
            0xbaa0 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbaa2
            }
            0xbaa2 => {
                w.mask(32, width);
                0xbaa4
            }
            0xbaa4 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbacc
                } else {
                    0xbaa6
                }
            }
            0xbaa6 => {
                w.branch(true);
                0xbada
            }
            0xbaa8 => {
                w.drain(8);
                let address = u32::from(0x6bu16);
                w.write(address, 0, width);
                0xbaaa
            }
            0xbaaa => {
                w.item_call();
                self.tail = Some(VerticalResponseTail::DoorSnap {
                    velocity_done: false,
                });
                0xc1e4
            }
            0xbaad => {
                w.item_call();
                self.tail = Some(VerticalResponseTail::DoorSnap {
                    velocity_done: true,
                });
                0xc1ff
            }
            0xbab0 => {
                w.drain(8);
                let address = u32::from(0x6cu16);
                w.write(address, 0, width);
                0xbab2
            }
            0xbab2 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbab4
            }
            0xbab4 => {
                w.mask(32, width);
                0xbab6
            }
            0xbab6 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbacc
                } else {
                    0xbab8
                }
            }
            0xbab8 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbaba
            }
            0xbaba => {
                w.mask(1, width);
                0xbabc
            }
            0xbabc => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbacc
                } else {
                    0xbabe
                }
            }
            0xbabe => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xbac0
            }
            0xbac0 => {
                w.mask(7, width);
                0xbac2
            }
            0xbac2 => {
                w.compare_immediate(1, width);
                0xbac4
            }
            0xbac4 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbacc
                } else {
                    0xbac6
                }
            }
            0xbac6 => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xbac8
            }
            0xbac8 => {
                w.mask(248, width);
                0xbaca
            }
            0xbaca => {
                w.drain(8);
                let address = u32::from(0x22u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbacc
            }
            0xbacc => {
                w.drain(16);
                let v = w.read(w.absolute(0x315u16), width);
                w.load(v, width);
                0xbacf
            }
            0xbacf => {
                w.mask(2, width);
                0xbad1
            }
            0xbad1 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbad9
                } else {
                    0xbad3
                }
            }
            0xbad3 => {
                w.drain(8);
                let v = w.read(u32::from(0x50u16), width);
                w.load(v, width);
                0xbad5
            }
            0xbad5 => {
                w.mask(253, width);
                0xbad7
            }
            0xbad7 => {
                w.drain(8);
                let address = u32::from(0x50u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbad9
            }
            0xbad9 => {
                w.rts();
                self.tail = Some(VerticalResponseTail::Returned);
                assert!(matches!(w.cpu.pc, 0xb95f | 0xb97b));
                w.cpu.pc
            }
            0xbada => {
                w.drain(16);
                let v = w.read(w.absolute(0x315u16), width);
                w.load(v, width);
                0xbadd
            }
            0xbadd => {
                w.mask(2, width);
                0xbadf
            }
            0xbadf => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbae3
                } else {
                    0xbae1
                }
            }
            0xbae1 => {
                w.drain(8);
                let address = u32::from(0x6cu16);
                w.write(address, 0, width);
                0xbae3
            }
            0xbae3 => {
                w.drain(16);
                let v = w.read(w.absolute(0x315u16), width);
                w.load(v, width);
                0xbae6
            }
            0xbae6 => {
                w.mask(2, width);
                0xbae8
            }
            0xbae8 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbaf4
                } else {
                    0xbaea
                }
            }
            0xbaea => {
                w.drain(8);
                let v = w.read(u32::from(0x50u16), width);
                w.load(v, width);
                0xbaec
            }
            0xbaec => {
                w.mask(253, width);
                0xbaee
            }
            0xbaee => {
                w.drain(8);
                let address = u32::from(0x50u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbaf0
            }
            0xbaf0 => {
                w.drain(8);
                let address = u32::from(0x49u16);
                w.write(address, 0, width);
                0xbaf2
            }
            0xbaf2 => {
                w.drain(8);
                let address = u32::from(0xefu16);
                w.write(address, 0, width);
                0xbaf4
            }
            0xbaf4 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbaf6
            }
            0xbaf6 => {
                w.mask(7, width);
                0xbaf8
            }
            0xbaf8 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbb0d
                } else {
                    0xbafa
                }
            }
            0xbafa => {
                w.drain(8);
                let v = w.read(u32::from(0xcu16), width);
                w.load(v, width);
                0xbafc
            }
            0xbafc => {
                w.mask(5, width);
                0xbafe
            }
            0xbafe => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbb0d
                } else {
                    0xbb00
                }
            }
            0xbb00 => {
                w.drain(16);
                let address = w.absolute(0x3f3u16);
                w.write(address, 0, width);
                0xbb03
            }
            0xbb03 => {
                w.item_call();
                self.tail = Some(VerticalResponseTail::Slope);
                0xe076
            }
            0xbb06 => {
                w.drain(8);
                let v = w.read(u32::from(0x6bu16), width);
                w.load(v, width);
                0xbb08
            }
            0xbb08 => {
                w.mask(15, width);
                0xbb0a
            }
            0xbb0a => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbb0d
                } else {
                    0xbb0c
                }
            }
            0xbb0c => {
                w.rts();
                self.tail = Some(VerticalResponseTail::Returned);
                assert!(matches!(w.cpu.pc, 0xb95f | 0xb97b));
                w.cpu.pc
            }
            0xbb0d => {
                w.drain(8);
                let address = u32::from(0x6bu16);
                w.write(address, 0, width);
                0xbb0f
            }
            0xbb0f => {
                w.drain(16);
                let v = w.read(w.absolute(0x2e7u16), width);
                w.load(v, width);
                0xbb12
            }
            0xbb12 => {
                w.mask(32, width);
                0xbb14
            }
            0xbb14 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbb2c
                } else {
                    0xbb16
                }
            }
            0xbb16 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbb18
            }
            0xbb18 => {
                w.drain(6);
                w.push(w.cpu.a & if width == 1 { 255 } else { 65535 }, width);
                self.collision = w.cpu.a & 255;
                0xbb19
            }
            0xbb19 => {
                w.drain(8);
                let v = w.read(u32::from(0xfu16), width);
                w.load(v, width);
                0xbb1b
            }
            0xbb1b => {
                w.drain(6);
                w.push(w.cpu.a & if width == 1 { 255 } else { 65535 }, width);
                self.collision |= (w.cpu.a & 255) << 8;
                0xbb1c
            }
            0xbb1c => {
                w.drain(16);
                let v = w.read(w.absolute(0x2eau16), width);
                w.load(v, width);
                0xbb1f
            }
            0xbb1f => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 1;
                self.tail = Some(VerticalResponseTail::Chest {
                    collision: self.collision,
                });
                0xeb66
            }
            0xbb23 => {
                w.drain(16);
                let address = w.absolute(0x2eau16);
                w.write(address, 0, width);
                0xbb26
            }
            0xbb26 => {
                w.drain(12);
                let v = w.pull(width);
                w.load(v, width);
                0xbb27
            }
            0xbb27 => {
                w.drain(8);
                let address = u32::from(0xfu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbb29
            }
            0xbb29 => {
                w.drain(12);
                let v = w.pull(width);
                w.load(v, width);
                0xbb2a
            }
            0xbb2a => {
                w.drain(8);
                let address = u32::from(0xeu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbb2c
            }
            0xbb2c => {
                w.drain(8);
                let v = w.read(u32::from(0xeeu16), width);
                w.load(v, width);
                0xbb2e
            }
            0xbb2e => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbb55
                } else {
                    0xbb30
                }
            }
            0xbb30 => {
                w.drain(16);
                let v = w.read(w.absolute(0x34cu16), width);
                w.load(v, width);
                0xbb33
            }
            0xbb33 => {
                w.mask(7, width);
                0xbb35
            }
            0xbb35 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbb3e
                } else {
                    0xbb37
                }
            }
            0xbb37 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xbb39
            }
            0xbb39 => {
                w.drain(16);
                let address = w.absolute(0x322u16);
                let old = w.read(address, width);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(
                    address,
                    old | (w.cpu.a & if width == 1 { 255 } else { 65535 }),
                    width,
                );
                0xbb3c
            }
            0xbb3c => {
                w.branch(true);
                0xbb6b
            }
            0xbb3e => {
                w.drain(16);
                let v = w.read(w.absolute(0x2e8u16), width);
                w.load(v, width);
                0xbb41
            }
            0xbb41 => {
                w.mask(7, width);
                0xbb43
            }
            0xbb43 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbb6b
                } else {
                    0xbb45
                }
            }
            0xbb45 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbb47
            }
            0xbb47 => {
                w.mask(2, width);
                0xbb49
            }
            0xbb49 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbb6b
                } else {
                    0xbb4b
                }
            }
            0xbb4b => {
                w.drain(16);
                let v = w.read(w.absolute(0x322u16), width);
                w.load(v, width);
                0xbb4e
            }
            0xbb4e => {
                w.mask(254, width);
                0xbb50
            }
            0xbb50 => {
                w.drain(16);
                let address = w.absolute(0x322u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbb53
            }
            0xbb53 => {
                w.branch(true);
                0xbb6b
            }
            0xbb55 => {
                w.drain(16);
                let v = w.read(w.absolute(0x320u16), width);
                w.load(v, width);
                0xbb58
            }
            0xbb58 => {
                w.mask(7, width);
                0xbb5a
            }
            0xbb5a => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbb63
                } else {
                    0xbb5c
                }
            }
            0xbb5c => {
                w.drain(8 * u32::from(width));
                let v = 2;
                w.load(v, width);
                0xbb5e
            }
            0xbb5e => {
                w.drain(16);
                let address = w.absolute(0x322u16);
                let old = w.read(address, width);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(
                    address,
                    old | (w.cpu.a & if width == 1 { 255 } else { 65535 }),
                    width,
                );
                0xbb61
            }
            0xbb61 => {
                w.branch(true);
                0xbb6b
            }
            0xbb63 => {
                w.drain(16);
                let v = w.read(w.absolute(0x322u16), width);
                w.load(v, width);
                0xbb66
            }
            0xbb66 => {
                w.mask(253, width);
                0xbb68
            }
            0xbb68 => {
                w.drain(16);
                let address = w.absolute(0x322u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbb6b
            }
            0xbb6b => {
                w.drain(16);
                let v = w.read(w.absolute(0x2f7u16), width);
                w.load(v, width);
                0xbb6e
            }
            0xbb6e => {
                w.mask(34, width);
                0xbb70
            }
            0xbb70 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbbae
                } else {
                    0xbb72
                }
            }
            0xbb72 => {
                w.item_x(0, 1);
                0xbb74
            }
            0xbb74 => {
                w.mask(32, width);
                0xbb76
            }
            0xbb76 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbb7a
                } else {
                    0xbb78
                }
            }
            0xbb78 => {
                w.item_x(8, 1);
                0xbb7a
            }
            0xbb7a => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(address, w.cpu.x, 1);
                0xbb7c
            }
            0xbb7c => {
                w.drain(8);
                let address = u32::from(0x1u16);
                w.write(address, 0, width);
                0xbb7e
            }
            0xbb7e => {
                w.drain(8);
                let v = w.read(u32::from(0x66u16), width);
                w.load(v, width);
                0xbb80
            }
            0xbb80 => {
                w.shift_left(width);
                0xbb81
            }
            0xbb81 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                0xbb82
            }
            0xbb82 => {
                w.width(32, true);
                0xbb84
            }
            0xbb84 => {
                w.drain(24);
                let v = w.read(0x7ef360u32, width);
                w.load(v, width);
                0xbb88
            }
            0xbb88 => {
                w.clear_carry();
                0xbb89
            }
            0xbb89 => {
                w.drain(8 * u32::from(width));
                let v = 5;
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xbb8c
            }
            0xbb8c => {
                w.drain(24);
                let address = 0x7ef360u32;
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbb90
            }
            0xbb90 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xbb92
            }
            0xbb92 => {
                w.clear_carry();
                0xbb93
            }
            0xbb93 => {
                w.drain(16);
                if (0xb9f7u16 + w.cpu.y) & 0xff00 != 0xb900 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xb9f7u16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xbb96
            }
            0xbb96 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xbb97
            }
            0xbb97 => {
                w.drain(8);
                let v = w.read(u32::from(0x0u16), width);
                w.sbc(v, width);
                0xbb99
            }
            0xbb99 => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbb9b
            }
            0xbb9b => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xbb9d
            }
            0xbb9d => {
                w.clear_carry();
                0xbb9e
            }
            0xbb9e => {
                w.drain(16);
                if (0xb9ffu16 + w.cpu.y) & 0xff00 != 0xb900 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xb9ffu16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xbba1
            }
            0xbba1 => {
                w.drain(8);
                let address = u32::from(0x2u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xbba3
            }
            0xbba3 => {
                w.width(32, false);
                0xbba5
            }
            0xbba5 => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 1;
                self.tail = Some(VerticalResponseTail::Rupee {
                    tile_deleted: false,
                });
                0xe8bd
            }
            0xbba9 => {
                w.drain(8 * u32::from(width));
                let v = 10;
                w.load(v, width);
                0xbbab
            }
            0xbbab => {
                w.item_call();
                self.tail = Some(VerticalResponseTail::Rupee { tile_deleted: true });
                0x802f
            }
            0xbbae => {
                w.item_y(1, 1);
                0xbbb0
            }
            0xbbb0 => {
                w.drain(16);
                let v = w.read(w.absolute(0x3f1u16), width);
                w.load(v, width);
                0xbbb3
            }
            0xbbb3 => {
                w.mask(34, width);
                0xbbb5
            }
            0xbbb5 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbbc2
                } else {
                    0xbbb7
                }
            }
            0xbbb7 => {
                w.mask(32, width);
                0xbbb9
            }
            0xbbb9 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbbbd
                } else {
                    0xbbbb
                }
            }
            0xbbbb => {
                w.item_y(2, 1);
                0xbbbd
            }
            0xbbbd => {
                w.drain(16);
                let address = w.absolute(0x3f3u16);
                w.write(address, w.cpu.y, 1);
                0xbbc0
            }
            0xbbc0 => {
                w.branch(true);
                0xbbe6
            }
            0xbbc2 => {
                w.item_y(3, 1);
                0xbbc4
            }
            0xbbc4 => {
                w.drain(16);
                let v = w.read(w.absolute(0x3f2u16), width);
                w.load(v, width);
                0xbbc7
            }
            0xbbc7 => {
                w.mask(34, width);
                0xbbc9
            }
            0xbbc9 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbbd6
                } else {
                    0xbbcb
                }
            }
            0xbbcb => {
                w.mask(32, width);
                0xbbcd
            }
            0xbbcd => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xbbd1
                } else {
                    0xbbcf
                }
            }
            0xbbcf => {
                w.item_y(4, 1);
                0xbbd1
            }
            0xbbd1 => {
                w.drain(16);
                let address = w.absolute(0x3f3u16);
                w.write(address, w.cpu.y, 1);
                0xbbd4
            }
            0xbbd4 => {
                w.branch(true);
                0xbbe6
            }
            0xbbd6 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2e8u16), width);
                w.load(v, width);
                0xbbd9
            }
            0xbbd9 => {
                w.mask(7, width);
                0xbbdb
            }
            0xbbdb => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbbe6
                } else {
                    0xbbdd
                }
            }
            0xbbdd => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xbbdf
            }
            0xbbdf => {
                w.mask(2, width);
                0xbbe1
            }
            0xbbe1 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xbbe6
                } else {
                    0xbbe3
                }
            }
            0xbbe3 => {
                w.drain(16);
                let address = w.absolute(0x3f3u16);
                w.write(address, 0, width);
                0xbbe6
            }
            _ => panic!("ground velocity PC {pc:04x}"),
        };
        if next == 0xbbe6 {
            self.tail = Some(VerticalResponseTail::SurfaceTail);
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
    fn vertical_response_matches_source_control_flow_and_bus() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in (0x07b7c3..0x07b7c5).chain(0x07b9f7..0x07ba07) {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
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
                r[0x1b] = u8::from(variant & 8 == 0);
                r[0x308] = if variant & 4 == 0 { 0 } else { 128 };
                r[0x46] = variant & 2;
                r[0x6c] = timer & 3;
                r[0x6a] = (timer >> 2) & 3;
                r[0x46c] = (timer >> 4) % 5;
                r[0xee] = variant & 1;
                r[0x315] = (variant >> 1) & 3;
                r[0x0e] = timer;
                r[0x0f] = timer.rotate_left(3);
                r[0x0c] = if variant & 16 == 0 { 0 } else { 5 };
                r[0x62] = variant & 1;
                r[0x2e7] = if timer & 128 == 0 { 0 } else { 32 };
                r[0x2f7] = if variant & 2 == 0 { 0 } else { 0x22 };
                r[0x34c] = variant;
                r[0x2e8] = variant & 7;
                r[0x320] = variant;
                r[0x3f1] = timer & 0x22;
                r[0x3f2] = timer.rotate_left(2) & 0x22;
                r[0x66] = (timer >> 6) & 1;
                let values = [0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa];
                for (n, address) in [0x20, 0x22, 0xf360].into_iter().enumerate() {
                    r[address..address + 2]
                        .copy_from_slice(&values[((timer as usize) + n) % 8].to_le_bytes());
                }
                let pc = 0xba2e;
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
                let mut work = VerticalCollisionResponse::default();
                while work.tail().is_none() {
                    let label = format!(
                        "vertical response controls={timer} variant={variant} pc={:04x}",
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
                tails.insert(
                    format!("{:?}", work.tail().unwrap())
                        .split("collision:")
                        .next()
                        .unwrap()
                        .to_owned(),
                );
                assert!(work.tail().is_some());
                assert_eq!(cpu.pc, source.snes().cpu.pc);
            }
        }
        assert_eq!(tails.len(), 9, "reachable tail coverage: {tails:?}");
        println!("vertical response: 8192 cases, {instructions} source instructions, doorway, collision layers, rupees, conveyor and child handoffs");
    }
}
