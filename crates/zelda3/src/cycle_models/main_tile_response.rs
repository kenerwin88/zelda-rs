//! Main tile response, swim reset and cardinal return on the live CPU clock.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    [8, 8, 8]
        .get(address.checked_sub(0x07d06c)? as usize)
        .copied()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MainTileResponseTail {
    Returned,
    WhirlpoolIndoor,
    WhirlpoolOutdoor,
    Sound(u8),
    Damage,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MainTileResponse {
    pc: Option<u16>,
    tail: Option<MainTileResponseTail>,
}
impl MainTileResponse {
    pub(crate) fn tail(&self) -> Option<MainTileResponseTail> {
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
        let pc = self.pc.unwrap_or(0xd104);
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
            0x983a => {
                w.drain(6);
                w.push(w.cpu.db as u16, 1);
                0x983b
            }
            0x983b => {
                w.drain(6);
                w.push(w.cpu.bank as u16, 1);
                0x983c
            }
            0x983c => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                0x983d
            }
            0x983d => {
                w.drain(16);
                let address = w.absolute(0x2cbu16);
                w.write(address, 0, width);
                0x9840
            }
            0x9840 => {
                w.drain(16);
                let address = w.absolute(0x34fu16);
                w.write(address, 0, width);
                0x9843
            }
            0x9843 => {
                w.drain(16);
                let address = w.absolute(0x32au16);
                w.write(address, 0, width);
                0x9846
            }
            0x9846 => {
                w.item_call();
                0x9873
            }
            0x9849 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                0x984a
            }
            0x984a => {
                w.rtl();
                assert!(matches!(w.cpu.pc, 0xd29a | 0xd2b1));
                w.cpu.pc
            }
            0x9873 => {
                w.width(32, true);
                0x9875
            }
            0x9875 => {
                w.drain(16);
                let address = w.absolute(0x32fu16);
                w.write(address, 0, width);
                0x9878
            }
            0x9878 => {
                w.drain(16);
                let address = w.absolute(0x331u16);
                w.write(address, 0, width);
                0x987b
            }
            0x987b => {
                w.drain(16);
                let address = w.absolute(0x326u16);
                w.write(address, 0, width);
                0x987e
            }
            0x987e => {
                w.drain(16);
                let address = w.absolute(0x328u16);
                w.write(address, 0, width);
                0x9881
            }
            0x9881 => {
                w.drain(16);
                let address = w.absolute(0x32bu16);
                w.write(address, 0, width);
                0x9884
            }
            0x9884 => {
                w.drain(16);
                let address = w.absolute(0x32du16);
                w.write(address, 0, width);
                0x9887
            }
            0x9887 => {
                w.drain(16);
                let address = w.absolute(0x33cu16);
                w.write(address, 0, width);
                0x988a
            }
            0x988a => {
                w.drain(16);
                let address = w.absolute(0x33eu16);
                w.write(address, 0, width);
                0x988d
            }
            0x988d => {
                w.drain(16);
                let address = w.absolute(0x334u16);
                w.write(address, 0, width);
                0x9890
            }
            0x9890 => {
                w.drain(16);
                let address = w.absolute(0x336u16);
                w.write(address, 0, width);
                0x9893
            }
            0x9893 => {
                w.width(32, false);
                0x9895
            }
            0x9895 => {
                w.rts();
                assert!(matches!(
                    w.cpu.pc,
                    0xd16e | 0xd1ae | 0xd1e1 | 0x9849 | 0xd222
                ));
                w.cpu.pc
            }
            0xafb5 => {
                w.drain(16);
                let v = w.read(w.absolute(0x37au16), width);
                w.load(v, width);
                0xafb8
            }
            0xafb8 => {
                w.mask(8, width);
                0xafba
            }
            0xafba => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xafc8
                } else {
                    0xafbc
                }
            }
            0xafbc => {
                w.item_x(4, 1);
                0xafbe
            }
            0xafbe => {
                w.drain(16);
                if (0xc4au16 + w.cpu.x) & 0xff00 != 0xc00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xc4au16 + w.cpu.x), width);
                w.load(v, width);
                0xafc1
            }
            0xafc1 => {
                w.compare_immediate(49, width);
                0xafc3
            }
            0xafc3 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xafca
                } else {
                    0xafc5
                }
            }
            0xafc5 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                0xafc6
            }
            0xafc6 => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xafbe
                } else {
                    0xafc8
                }
            }
            0xafc8 => {
                w.clear_carry();
                0xafc9
            }
            0xafc9 => {
                w.rts();
                assert!(matches!(
                    w.cpu.pc,
                    0xd16e | 0xd1ae | 0xd1e1 | 0x9849 | 0xd222
                ));
                w.cpu.pc
            }
            0xafca => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xafcb
            }
            0xafcb => {
                w.rts();
                assert!(matches!(
                    w.cpu.pc,
                    0xd16e | 0xd1ae | 0xd1e1 | 0x9849 | 0xd222
                ));
                w.cpu.pc
            }
            0xb8d8 => {
                w.drain(8);
                let v = w.read(u32::from(0x6au16), width);
                w.load(v, width);
                0xb8da
            }
            0xb8da => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb8de
                } else {
                    0xb8dc
                }
            }
            0xb8dc => {
                w.drain(8);
                let address = u32::from(0x6bu16);
                w.write(address, 0, width);
                0xb8de
            }
            0xb8de => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xb8e0
            }
            0xb8e0 => {
                w.compare_immediate(11, width);
                0xb8e2
            }
            0xb8e2 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb8fc
                } else {
                    0xb8e4
                }
            }
            0xb8e4 => {
                w.item_y(8, 1);
                0xb8e6
            }
            0xb8e6 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xb8e8
            }
            0xb8e8 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb8e9
            }
            0xb8e9 => {
                w.drain(8);
                let v = w.read(u32::from(0x3eu16), width);
                w.sbc(v, width);
                0xb8eb
            }
            0xb8eb => {
                w.drain(8);
                let address = u32::from(0x30u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb8ed
            }
            0xb8ed => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb8fc
                } else {
                    0xb8ef
                }
            }
            0xb8ef => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xb8f3
                } else {
                    0xb8f1
                }
            }
            0xb8f1 => {
                w.item_y(4, 1);
                0xb8f3
            }
            0xb8f3 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xb8f5
            }
            0xb8f5 => {
                w.mask(3, width);
                0xb8f7
            }
            0xb8f7 => {
                w.drain(8);
                let address = u32::from(0x67u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb8f9
            }
            0xb8f9 => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xb8fa
            }
            0xb8fa => {
                w.drain(8);
                let address = u32::from(0x67u16);
                let old = w.read(address, width);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(
                    address,
                    old | (w.cpu.a & if width == 1 { 255 } else { 65535 }),
                    width,
                );
                0xb8fc
            }
            0xb8fc => {
                w.item_y(2, 1);
                0xb8fe
            }
            0xb8fe => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xb900
            }
            0xb900 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb901
            }
            0xb901 => {
                w.drain(8);
                let v = w.read(u32::from(0x3fu16), width);
                w.sbc(v, width);
                0xb903
            }
            0xb903 => {
                w.drain(8);
                let address = u32::from(0x31u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb905
            }
            0xb905 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb914
                } else {
                    0xb907
                }
            }
            0xb907 => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xb90b
                } else {
                    0xb909
                }
            }
            0xb909 => {
                w.item_y(1, 1);
                0xb90b
            }
            0xb90b => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xb90d
            }
            0xb90d => {
                w.mask(12, width);
                0xb90f
            }
            0xb90f => {
                w.drain(8);
                let address = u32::from(0x67u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb911
            }
            0xb911 => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xb912
            }
            0xb912 => {
                w.drain(8);
                let address = u32::from(0x67u16);
                let old = w.read(address, width);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(
                    address,
                    old | (w.cpu.a & if width == 1 { 255 } else { 65535 }),
                    width,
                );
                0xb914
            }
            0xb914 => {
                w.drain(8);
                let v = w.read(u32::from(0x1bu16), width);
                w.load(v, width);
                0xb916
            }
            0xb916 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb955
                } else {
                    0xb918
                }
            }
            0xb918 => {
                w.drain(16);
                let v = w.read(w.absolute(0x46cu16), width);
                w.load(v, width);
                0xb91b
            }
            0xb91b => {
                w.compare_immediate(4, width);
                0xb91d
            }
            0xb91d => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb955
                } else {
                    0xb91f
                }
            }
            0xb91f => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xb921
            }
            0xb921 => {
                w.compare_immediate(4, width);
                0xb923
            }
            0xb923 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb955
                } else {
                    0xb925
                }
            }
            0xb925 => {
                w.item_y(247, 1);
                0xb927
            }
            0xb927 => {
                w.drain(16);
                let v = w.read(w.absolute(0x310u16), width);
                w.load(v, width);
                0xb92a
            }
            0xb92a => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb93d
                } else {
                    0xb92c
                }
            }
            0xb92c => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xb930
                } else {
                    0xb92e
                }
            }
            0xb92e => {
                w.item_y(251, 1);
                0xb930
            }
            0xb930 => {
                w.drain(8 * u32::from(width));
                let v = 255;
                w.load((w.cpu.a ^ v) & if width == 1 { 255 } else { 65535 }, width);
                0xb932
            }
            0xb932 => {
                w.increment_a(width);
                0xb933
            }
            0xb933 => {
                w.clear_carry();
                0xb934
            }
            0xb934 => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xb936
            }
            0xb936 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb93d
                } else {
                    0xb938
                }
            }
            0xb938 => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xb939
            }
            0xb939 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xb93b
            }
            0xb93b => {
                w.drain(8);
                let address = u32::from(0x67u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb93d
            }
            0xb93d => {
                w.item_y(253, 1);
                0xb93f
            }
            0xb93f => {
                w.drain(16);
                let v = w.read(w.absolute(0x312u16), width);
                w.load(v, width);
                0xb942
            }
            0xb942 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb955
                } else {
                    0xb944
                }
            }
            0xb944 => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xb948
                } else {
                    0xb946
                }
            }
            0xb946 => {
                w.item_y(254, 1);
                0xb948
            }
            0xb948 => {
                w.drain(8 * u32::from(width));
                let v = 255;
                w.load((w.cpu.a ^ v) & if width == 1 { 255 } else { 65535 }, width);
                0xb94a
            }
            0xb94a => {
                w.increment_a(width);
                0xb94b
            }
            0xb94b => {
                w.clear_carry();
                0xb94c
            }
            0xb94c => {
                w.drain(8);
                let v = w.read(u32::from(0x31u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xb94e
            }
            0xb94e => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb955
                } else {
                    0xb950
                }
            }
            0xb950 => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xb951
            }
            0xb951 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xb953
            }
            0xb953 => {
                w.drain(8);
                let address = u32::from(0x67u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb955
            }
            0xb955 => {
                w.rts();
                self.tail = Some(MainTileResponseTail::Returned);
                assert_eq!((w.cpu.pc, w.cpu.sp), (0x82c5, 0x1f4));
                w.cpu.pc
            }
            0xd104 => {
                w.drain(16);
                let v = w.read(w.absolute(0x357u16), width);
                w.load(v, width);
                0xd107
            }
            0xd107 => {
                w.mask(16, width);
                0xd109
            }
            0xd109 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd15c
                } else {
                    0xd10b
                }
            }
            0xd10b => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xd10d
            }
            0xd10d => {
                w.clear_carry();
                0xd10e
            }
            0xd10e => {
                w.drain(8 * u32::from(width));
                let v = 8;
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xd110
            }
            0xd110 => {
                w.mask(15, width);
                0xd112
            }
            0xd112 => {
                w.compare_immediate(4, width);
                0xd114
            }
            0xd114 => {
                let taken = w.branch(w.cpu.p & CARRY == 0);
                if taken {
                    0xd11a
                } else {
                    0xd116
                }
            }
            0xd116 => {
                w.compare_immediate(11, width);
                0xd118
            }
            0xd118 => {
                let taken = w.branch(w.cpu.p & CARRY == 0);
                if taken {
                    0xd159
                } else {
                    0xd11a
                }
            }
            0xd11a => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xd11c
            }
            0xd11c => {
                w.mask(15, width);
                0xd11e
            }
            0xd11e => {
                w.compare_immediate(4, width);
                0xd120
            }
            0xd120 => {
                let taken = w.branch(w.cpu.p & CARRY == 0);
                if taken {
                    0xd126
                } else {
                    0xd122
                }
            }
            0xd122 => {
                w.compare_immediate(12, width);
                0xd124
            }
            0xd124 => {
                let taken = w.branch(w.cpu.p & CARRY == 0);
                if taken {
                    0xd159
                } else {
                    0xd126
                }
            }
            0xd126 => {
                w.drain(16);
                let v = w.read(w.absolute(0x31fu16), width);
                w.load(v, width);
                0xd129
            }
            0xd129 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd159
                } else {
                    0xd12b
                }
            }
            0xd12b => {
                w.drain(8);
                let v = w.read(u32::from(0x4du16), width);
                w.load(v, width);
                0xd12d
            }
            0xd12d => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd159
                } else {
                    0xd12f
                }
            }
            0xd12f => {
                w.drain(8);
                let v = w.read(u32::from(0x1bu16), width);
                w.load(v, width);
                0xd131
            }
            0xd131 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd151
                } else {
                    0xd133
                }
            }
            0xd133 => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 2;
                self.tail = Some(MainTileResponseTail::WhirlpoolIndoor);
                0xb929
            }
            0xd151 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2dbu16), width);
                w.load(v, width);
                0xd154
            }
            0xd154 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd159
                } else {
                    0xd156
                }
            }
            0xd156 => {
                w.item_call();
                self.tail = Some(MainTileResponseTail::WhirlpoolOutdoor);
                0xa95c
            }
            0xd159 => {
                w.operand_then_internal(2);
                0xd1fc
            }
            0xd15c => {
                w.drain(16);
                let address = w.absolute(0x2dbu16);
                w.write(address, 0, width);
                0xd15f
            }
            0xd15f => {
                w.drain(16);
                let v = w.read(w.absolute(0x357u16), width);
                w.load(v, width);
                0xd162
            }
            0xd162 => {
                w.mask(1, width);
                0xd164
            }
            0xd164 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd17c
                } else {
                    0xd166
                }
            }
            0xd166 => {
                w.drain(8 * u32::from(width));
                let v = 2;
                w.load(v, width);
                0xd168
            }
            0xd168 => {
                w.drain(16);
                let address = w.absolute(0x351u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd16b
            }
            0xd16b => {
                w.item_call();
                0xd2c6
            }
            0xd16e => {
                let taken = w.branch(w.cpu.p & CARRY != 0);
                if taken {
                    0xd179
                } else {
                    0xd170
                }
            }
            0xd170 => {
                w.drain(8);
                let v = w.read(u32::from(0x4du16), width);
                w.load(v, width);
                0xd172
            }
            0xd172 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd179
                } else {
                    0xd174
                }
            }
            0xd174 => {
                w.drain(8 * u32::from(width));
                let v = 26;
                w.load(v, width);
                0xd176
            }
            0xd176 => {
                w.item_call();
                self.tail = Some(MainTileResponseTail::Sound(w.cpu.a as u8));
                0x8028
            }
            0xd179 => {
                w.operand_then_internal(2);
                0xd2c5
            }
            0xd17c => {
                w.drain(16);
                let v = w.read(w.absolute(0x359u16), width);
                w.load(v, width);
                0xd17f
            }
            0xd17f => {
                w.mask(1, width);
                0xd181
            }
            0xd181 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd1c9
                } else {
                    0xd183
                }
            }
            0xd183 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xd185
            }
            0xd185 => {
                w.drain(16);
                let address = w.absolute(0x351u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd188
            }
            0xd188 => {
                w.drain(8);
                let v = w.read(u32::from(0x1bu16), width);
                w.load(v, width);
                0xd18a
            }
            0xd18a => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd1ab
                } else {
                    0xd18c
                }
            }
            0xd18c => {
                w.drain(16);
                let v = w.read(w.absolute(0x345u16), width);
                w.load(v, width);
                0xd18f
            }
            0xd18f => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd1ab
                } else {
                    0xd191
                }
            }
            0xd191 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2e0u16), width);
                w.load(v, width);
                0xd194
            }
            0xd194 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd1ab
                } else {
                    0xd196
                }
            }
            0xd196 => {
                w.drain(24);
                let v = w.read(0x7ef356u32, width);
                w.load(v, width);
                0xd19a
            }
            0xd19a => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd179
                } else {
                    0xd19c
                }
            }
            0xd19c => {
                w.drain(16);
                let address = w.absolute(0x345u16);
                w.write(address, 0, width);
                0xd19f
            }
            0xd19f => {
                w.drain(16);
                let v = w.read(w.absolute(0x340u16), width);
                w.load(v, width);
                0xd1a2
            }
            0xd1a2 => {
                w.drain(8);
                let address = u32::from(0x26u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd1a4
            }
            0xd1a4 => {
                w.drain(8 * u32::from(width));
                let v = 0;
                w.load(v, width);
                0xd1a6
            }
            0xd1a6 => {
                w.drain(8);
                let address = u32::from(0x5du16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd1a8
            }
            0xd1a8 => {
                w.operand_then_internal(2);
                0xd2c5
            }
            0xd1ab => {
                w.item_call();
                0xd2c6
            }
            0xd1ae => {
                let taken = w.branch(w.cpu.p & CARRY != 0);
                if taken {
                    0xd1c6
                } else {
                    0xd1b0
                }
            }
            0xd1b0 => {
                w.drain(8);
                let v = w.read(u32::from(0x8au16), width);
                w.load(v, width);
                0xd1b2
            }
            0xd1b2 => {
                w.compare_immediate(112, width);
                0xd1b4
            }
            0xd1b4 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd1bd
                } else {
                    0xd1b6
                }
            }
            0xd1b6 => {
                w.drain(8 * u32::from(width));
                let v = 27;
                w.load(v, width);
                0xd1b8
            }
            0xd1b8 => {
                w.item_call();
                self.tail = Some(MainTileResponseTail::Sound(w.cpu.a as u8));
                0x8028
            }
            0xd1bd => {
                w.drain(8);
                let v = w.read(u32::from(0x4du16), width);
                w.load(v, width);
                0xd1bf
            }
            0xd1bf => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd1c6
                } else {
                    0xd1c1
                }
            }
            0xd1c1 => {
                w.drain(8 * u32::from(width));
                let v = 28;
                w.load(v, width);
                0xd1c3
            }
            0xd1c3 => {
                w.item_call();
                self.tail = Some(MainTileResponseTail::Sound(w.cpu.a as u8));
                0x8028
            }
            0xd1c6 => {
                w.operand_then_internal(2);
                0xd2c5
            }
            0xd1c9 => {
                w.drain(8);
                let v = w.read(u32::from(0x1bu16), width);
                w.load(v, width);
                0xd1cb
            }
            0xd1cb => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd1fc
                } else {
                    0xd1cd
                }
            }
            0xd1cd => {
                w.drain(16);
                let v = w.read(w.absolute(0x345u16), width);
                w.load(v, width);
                0xd1d0
            }
            0xd1d0 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd1fc
                } else {
                    0xd1d2
                }
            }
            0xd1d2 => {
                w.drain(16);
                let v = w.read(w.absolute(0x341u16), width);
                w.load(v, width);
                0xd1d5
            }
            0xd1d5 => {
                w.mask(1, width);
                0xd1d7
            }
            0xd1d7 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd1fc
                } else {
                    0xd1d9
                }
            }
            0xd1d9 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xd1db
            }
            0xd1db => {
                w.drain(16);
                let address = w.absolute(0x351u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd1de
            }
            0xd1de => {
                w.item_call();
                0xd2c6
            }
            0xd1e1 => {
                let taken = w.branch(w.cpu.p & CARRY != 0);
                if taken {
                    0xd1f9
                } else {
                    0xd1e3
                }
            }
            0xd1e3 => {
                w.drain(8);
                let v = w.read(u32::from(0x8au16), width);
                w.load(v, width);
                0xd1e5
            }
            0xd1e5 => {
                w.compare_immediate(112, width);
                0xd1e7
            }
            0xd1e7 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd1f0
                } else {
                    0xd1e9
                }
            }
            0xd1e9 => {
                w.drain(8 * u32::from(width));
                let v = 27;
                w.load(v, width);
                0xd1eb
            }
            0xd1eb => {
                w.item_call();
                self.tail = Some(MainTileResponseTail::Sound(w.cpu.a as u8));
                0x8028
            }
            0xd1f0 => {
                w.drain(8);
                let v = w.read(u32::from(0x4du16), width);
                w.load(v, width);
                0xd1f2
            }
            0xd1f2 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd1f9
                } else {
                    0xd1f4
                }
            }
            0xd1f4 => {
                w.drain(8 * u32::from(width));
                let v = 28;
                w.load(v, width);
                0xd1f6
            }
            0xd1f6 => {
                w.item_call();
                self.tail = Some(MainTileResponseTail::Sound(w.cpu.a as u8));
                0x8028
            }
            0xd1f9 => {
                w.operand_then_internal(2);
                0xd2c5
            }
            0xd1fc => {
                w.drain(16);
                let address = w.absolute(0x351u16);
                w.write(address, 0, width);
                0xd1ff
            }
            0xd1ff => {
                w.drain(16);
                let v = w.read(w.absolute(0x2eeu16), width);
                w.load(v, width);
                0xd202
            }
            0xd202 => {
                w.mask(1, width);
                0xd204
            }
            0xd204 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd20e
                } else {
                    0xd206
                }
            }
            0xd206 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xd208
            }
            0xd208 => {
                w.drain(16);
                let address = w.absolute(0x2edu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd20b
            }
            0xd20b => {
                w.operand_then_internal(2);
                0xd2c5
            }
            0xd20e => {
                w.drain(16);
                let address = w.absolute(0x2edu16);
                w.write(address, 0, width);
                0xd211
            }
            0xd211 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2eeu16), width);
                w.load(v, width);
                0xd214
            }
            0xd214 => {
                w.mask(16, width);
                0xd216
            }
            0xd216 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd24b
                } else {
                    0xd218
                }
            }
            0xd218 => {
                w.drain(16);
                let address = w.absolute(0x373u16);
                w.write(address, 0, width);
                0xd21b
            }
            0xd21b => {
                w.drain(8);
                let v = w.read(u32::from(0x55u16), width);
                w.load(v, width);
                0xd21d
            }
            0xd21d => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd24b
                } else {
                    0xd21f
                }
            }
            0xd21f => {
                w.item_call();
                0xafb5
            }
            0xd222 => {
                let taken = w.branch(w.cpu.p & CARRY != 0);
                if taken {
                    0xd24b
                } else {
                    0xd224
                }
            }
            0xd224 => {
                w.drain(16);
                let v = w.read(w.absolute(0x31fu16), width);
                w.load(v, width);
                0xd227
            }
            0xd227 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd24b
                } else {
                    0xd229
                }
            }
            0xd229 => {
                w.drain(16);
                let address = w.absolute(0x3f7u16);
                w.write(address, 0, width);
                0xd22c
            }
            0xd22c => {
                w.drain(16);
                let address = w.absolute(0x3f5u16);
                w.write(address, 0, width);
                0xd22f
            }
            0xd22f => {
                w.drain(16);
                let address = w.absolute(0x3f6u16);
                w.write(address, 0, width);
                0xd232
            }
            0xd232 => {
                w.drain(24);
                let v = w.read(0x7ef357u32, width);
                w.load(v, width);
                0xd236
            }
            0xd236 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd23d
                } else {
                    0xd238
                }
            }
            0xd238 => {
                w.drain(8);
                let address = u32::from(0x56u16);
                w.write(address, 0, width);
                0xd23a
            }
            0xd23a => {
                w.drain(16);
                let address = w.absolute(0x2e0u16);
                w.write(address, 0, width);
                0xd23d
            }
            0xd23d => {
                w.drain(24);
                let v = w.read(0x7ef35bu32, width);
                w.load(v, width);
                0xd241
            }
            0xd241 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                0xd242
            }
            0xd242 => {
                w.drain(16);
                if (0xd06cu16 + w.cpu.y) & 0xff00 != 0xd000 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xd06cu16 + w.cpu.y), width);
                w.load(v, width);
                0xd245
            }
            0xd245 => {
                w.drain(16);
                let address = w.absolute(0x373u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd248
            }
            0xd248 => {
                w.operand_then_internal(2);
                0x9195
            }
            0xd24b => {
                w.drain(16);
                let v = w.read(w.absolute(0x348u16), width);
                w.load(v, width);
                0xd24e
            }
            0xd24e => {
                w.mask(17, width);
                0xd250
            }
            0xd250 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd29d
                } else {
                    0xd252
                }
            }
            0xd252 => {
                w.drain(16);
                let v = w.read(w.absolute(0x34au16), width);
                w.load(v, width);
                0xd255
            }
            0xd255 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd263
                } else {
                    0xd257
                }
            }
            0xd257 => {
                w.drain(8);
                let v = w.read(u32::from(0x6au16), width);
                w.load(v, width);
                0xd259
            }
            0xd259 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd2b4
                } else {
                    0xd25b
                }
            }
            0xd25b => {
                w.drain(16);
                let v = w.read(w.absolute(0x340u16), width);
                w.load(v, width);
                0xd25e
            }
            0xd25e => {
                w.drain(8);
                let address = u32::from(0x26u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd260
            }
            0xd260 => {
                w.operand_then_internal(2);
                0xd2b4
            }
            0xd263 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xd265
            }
            0xd265 => {
                w.mask(12, width);
                0xd267
            }
            0xd267 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd273
                } else {
                    0xd269
                }
            }
            0xd269 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xd26b
            }
            0xd26b => {
                w.drain(16);
                let address = w.absolute(0x33du16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd26e
            }
            0xd26e => {
                w.drain(8 * u32::from(width));
                let v = 128;
                w.load(v, width);
                0xd270
            }
            0xd270 => {
                w.drain(16);
                let address = w.absolute(0x33cu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd273
            }
            0xd273 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xd275
            }
            0xd275 => {
                w.mask(3, width);
                0xd277
            }
            0xd277 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd283
                } else {
                    0xd279
                }
            }
            0xd279 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xd27b
            }
            0xd27b => {
                w.drain(16);
                let address = w.absolute(0x33du16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd27e
            }
            0xd27e => {
                w.drain(8 * u32::from(width));
                let v = 128;
                w.load(v, width);
                0xd280
            }
            0xd280 => {
                w.drain(16);
                let address = w.absolute(0x33cu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd283
            }
            0xd283 => {
                w.item_y(1, 1);
                0xd285
            }
            0xd285 => {
                w.drain(16);
                let v = w.read(w.absolute(0x348u16), width);
                w.load(v, width);
                0xd288
            }
            0xd288 => {
                w.mask(1, width);
                0xd28a
            }
            0xd28a => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd28e
                } else {
                    0xd28c
                }
            }
            0xd28c => {
                w.item_y(2, 1);
                0xd28e
            }
            0xd28e => {
                w.drain(16);
                let address = w.absolute(0x34au16);
                w.write(address, w.cpu.y, 1);
                0xd291
            }
            0xd291 => {
                w.drain(8);
                let v = w.read(u32::from(0x26u16), width);
                w.load(v, width);
                0xd293
            }
            0xd293 => {
                w.drain(16);
                let address = w.absolute(0x340u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd296
            }
            0xd296 => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 7;
                0x983a
            }
            0xd29a => {
                w.operand_then_internal(2);
                0xd2b4
            }
            0xd29d => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xd29f
            }
            0xd29f => {
                w.compare_immediate(4, width);
                0xd2a1
            }
            0xd2a1 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd2b1
                } else {
                    0xd2a3
                }
            }
            0xd2a3 => {
                w.drain(16);
                let v = w.read(w.absolute(0x34au16), width);
                w.load(v, width);
                0xd2a6
            }
            0xd2a6 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd2ad
                } else {
                    0xd2a8
                }
            }
            0xd2a8 => {
                w.drain(16);
                let v = w.read(w.absolute(0x340u16), width);
                w.load(v, width);
                0xd2ab
            }
            0xd2ab => {
                w.drain(8);
                let address = u32::from(0x26u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd2ad
            }
            0xd2ad => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 7;
                0x983a
            }
            0xd2b1 => {
                w.drain(16);
                let address = w.absolute(0x34au16);
                w.write(address, 0, width);
                0xd2b4
            }
            0xd2b4 => {
                w.drain(16);
                let v = w.read(w.absolute(0x2e8u16), width);
                w.load(v, width);
                0xd2b7
            }
            0xd2b7 => {
                w.mask(16, width);
                0xd2b9
            }
            0xd2b9 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd2c5
                } else {
                    0xd2bb
                }
            }
            0xd2bb => {
                w.drain(16);
                let v = w.read(w.absolute(0x31fu16), width);
                w.load(v, width);
                0xd2be
            }
            0xd2be => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd2c5
                } else {
                    0xd2c0
                }
            }
            0xd2c0 => {
                w.drain(8 * u32::from(width));
                let v = 58;
                w.load(v, width);
                0xd2c2
            }
            0xd2c2 => {
                w.drain(16);
                let address = w.absolute(0x31fu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd2c5
            }
            0xd2c5 => {
                w.rts();
                assert_eq!(w.cpu.pc, 0xb8d8);
                w.cpu.pc
            }
            0xd2c6 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xd2c8
            }
            0xd2c8 => {
                w.mask(15, width);
                0xd2ca
            }
            0xd2ca => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd2e2
                } else {
                    0xd2cc
                }
            }
            0xd2cc => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xd2ce
            }
            0xd2ce => {
                w.compare_immediate(17, width);
                0xd2d0
            }
            0xd2d0 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd2da
                } else {
                    0xd2d2
                }
            }
            0xd2d2 => {
                w.drain(8);
                let v = w.read(u32::from(0x1au16), width);
                w.load(v, width);
                0xd2d4
            }
            0xd2d4 => {
                w.mask(15, width);
                0xd2d6
            }
            0xd2d6 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd2e0
                } else {
                    0xd2d8
                }
            }
            0xd2d8 => {
                w.branch(true);
                0xd2e2
            }
            0xd2da => {
                w.drain(8);
                let v = w.read(u32::from(0x1au16), width);
                w.load(v, width);
                0xd2dc
            }
            0xd2dc => {
                w.mask(7, width);
                0xd2de
            }
            0xd2de => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd2e2
                } else {
                    0xd2e0
                }
            }
            0xd2e0 => {
                w.clear_carry();
                0xd2e1
            }
            0xd2e1 => {
                w.rts();
                assert!(matches!(
                    w.cpu.pc,
                    0xd16e | 0xd1ae | 0xd1e1 | 0x9849 | 0xd222
                ));
                w.cpu.pc
            }
            0xd2e2 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xd2e3
            }
            0xd2e3 => {
                w.rts();
                assert!(matches!(
                    w.cpu.pc,
                    0xd16e | 0xd1ae | 0xd1e1 | 0x9849 | 0xd222
                ));
                w.cpu.pc
            }
            _ => panic!("main tile response PC {pc:04x}"),
        };
        if next == 0x9195 {
            self.tail = Some(MainTileResponseTail::Damage);
        }
        w.cpu.pc = next;
        self.pc = Some(next);
        Some(w.cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], timer: u8, variant: u8) {
    r[0x1f1..0x1f5].copy_from_slice(&[0xd7, 0xb8, 0xc4, 0x82]);
    for a in [0x357, 0x359, 0x341, 0x2ee, 0x348, 0x2e8] {
        r[a] = 0;
        r[a + 1] = 0xa5;
    }
    r[0x1b] = u8::from(timer & 1 == 0);
    r[0x67] = (timer >> 4) & 15;
    r[0x1a] = timer;
    r[0x5d] = [0, 4, 11, 17][(timer as usize >> 4) & 3];
    r[0x4d] = u8::from(variant & 16 != 0);
    r[0x31f] = u8::from(variant & 8 != 0);
    r[0x55] = u8::from(timer & 16 != 0);
    r[0x37a] = if timer & 32 != 0 { 8 } else { 0 };
    r[0xc4a..0xc4f].fill(0);
    if timer % 6 < 5 {
        r[0xc4a + usize::from(timer % 6)] = 0x31;
    }
    r[0xf35b] = (timer >> 6) % 3;
    r[0xf356] = timer & 1;
    r[0xf357] = (timer >> 1) & 1;
    r[0x345] = u8::from(timer & 2 != 0);
    r[0x2e0] = u8::from(timer & 32 != 0);
    r[0x2db] = u8::from(timer & 16 != 0);
    r[0x34a] = (timer >> 5) % 3;
    r[0x6a] = timer & 3;
    r[0x8a] = if timer & 64 == 0 { 0x70 } else { 0x20 };
    r[0x46c] = if variant & 8 == 0 { 4 } else { 0 };
    let values = [0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa];
    r[0x20..0x22].copy_from_slice(&values[(timer & 7) as usize].to_le_bytes());
    r[0x22..0x24].copy_from_slice(&values[((timer >> 3) & 7) as usize].to_le_bytes());
    r[0x3e] = (timer & 7).wrapping_sub(1);
    r[0x3f] = (timer >> 3).wrapping_add(1);
    r[0x310] = [0, 1, 127, 128, 255][timer as usize % 5];
    r[0x312] = [0, 1, 127, 128, 255][(timer as usize / 5) % 5];
    match variant & 7 {
        0 => {}
        1 => r[0x357] = 1,
        2 => r[0x359] = 1,
        3 => {
            r[0x357] = 0x10;
            r[0x20] = timer;
            r[0x22] = timer.wrapping_mul(3);
        }
        4 => r[0x2ee] = 1,
        5 => r[0x2ee] = 0x10,
        6 => r[0x348] = if timer & 128 == 0 { 1 } else { 0x10 },
        7 => {
            r[0x2e8] = 0x10;
            r[0x341] = timer & 1;
        }
        _ => unreachable!(),
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
    fn main_tile_response_matches_source_effects_reset_and_cardinal_return() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x07d06c..0x07d06f {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        assert_eq!(table_byte(0x07d06f), None);
        let mut categories = std::collections::BTreeSet::new();
        let mut instructions = 0;

        for timer in 0..=255u8 {
            for variant in 0..32u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7) ^ timer as usize) as u8;
                }
                seed_case(r, timer, variant);
                let pc = 0xd104;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: variant as u16,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: 0x1f0,
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
                let mut work = MainTileResponse::default();
                while work.tail().is_none() {
                    let label = format!(
                        "velocity controls={timer} variant={variant} pc={:04x}",
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
                                let byte = |address| table_byte(address);
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
                match work.tail().unwrap() {
                    MainTileResponseTail::Returned => {
                        assert_eq!((cpu.pc, cpu.sp, cpu.p & 0x30), (0x82c5, 0x1f4, 0x30))
                    }
                    MainTileResponseTail::WhirlpoolIndoor => {
                        assert_eq!((cpu.bank, cpu.pc), (2, 0xb929))
                    }
                    MainTileResponseTail::WhirlpoolOutdoor => assert_eq!(cpu.pc, 0xa95c),
                    MainTileResponseTail::Sound(sound) => {
                        assert_eq!(cpu.pc, 0x8028);
                        assert_eq!(sound, cpu.a as u8);
                    }
                    MainTileResponseTail::Damage => {
                        assert_eq!(cpu.pc, 0x9195);
                        assert_eq!(ram[0x373], 8);
                    }
                }
                categories.insert(format!("{:?}", work.tail().unwrap()));
            }
        }
        assert_eq!(categories.len(), 7);
        println!("categories: {categories:?}");
        println!("main tile response: 8192 cases, {instructions} source instructions, effects, byte neighbors, swim reset and cardinal return");
    }
}
