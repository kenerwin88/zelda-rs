//! Cardinal collision parent, room relocation and child selection on the live clock.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CardinalCollisionTail {
    KickbackHorizontal {
        direction: u8,
    },
    PrimaryFootprint,
    FinalFootprint {
        collision_two: bool,
    },
    SlopeChecks {
        primary_layer: bool,
        horizontal_first: bool,
        vertical_completed: bool,
    },
    MainHandler,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CardinalCollisionPrefix {
    pc: Option<u16>,
    tail: Option<CardinalCollisionTail>,
    primary_layer: bool,
    horizontal_first: bool,
    vertical_completed: bool,
}
impl CardinalCollisionPrefix {
    pub(crate) fn resume_after_vertical_axis(&mut self, return_pc: u16) {
        assert!(matches!(
            self.tail,
            Some(CardinalCollisionTail::SlopeChecks { .. })
        ));
        assert!(matches!(return_pc, 0xb95f | 0xb97b));
        self.vertical_completed = true;
        self.pc = Some(return_pc);
        self.tail = None;
    }
    #[cfg(test)]
    pub(crate) fn after_footprint_for_test(pc: u16) -> Self {
        assert!(matches!(pc, 0xb7f8 | 0xb892 | 0xb879));
        Self {
            pc: Some(pc),
            ..Default::default()
        }
    }
    pub(crate) fn tail(&self) -> Option<CardinalCollisionTail> {
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
        let pc = self.pc.unwrap_or(0x82c2);
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 0x18), (7, pc, 0, 0x10));
        let width = if cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match pc {
            0x82c2 => {
                w.item_call();
                0xb7c7
            }
            0xb7c7 => {
                w.drain(8);
                let address = u32::from(0x6eu16);
                w.write(address, 0, width);
                0xb7c9
            }
            0xb7c9 => {
                w.drain(8);
                let address = u32::from(0x38u16);
                w.write(address, 0, width);
                0xb7cb
            }
            0xb7cb => {
                w.drain(8);
                let v = w.read(u32::from(0x6bu16), width);
                w.load(v, width);
                0xb7cd
            }
            0xb7cd => {
                w.mask(48, width);
                0xb7cf
            }
            0xb7cf => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb7db
                } else {
                    0xb7d1
                }
            }
            0xb7d1 => {
                w.item_call();
                0xccab
            }
            0xb7d4 => {
                w.drain(8);
                let v = w.read(u32::from(0x6du16), width);
                w.load(v, width);
                0xb7d6
            }
            0xb7d6 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb7db
                } else {
                    0xb7d8
                }
            }
            0xb7d8 => {
                w.operand_then_internal(2);
                0xb84b
            }
            0xb7db => {
                w.item_call();
                0xb97c
            }
            0xb7de => {
                let taken = w.branch(w.cpu.p & CARRY == 0);
                if taken {
                    0xb84b
                } else {
                    0xb7e0
                }
            }
            0xb7e0 => {
                w.drain(16);
                let v = w.read(w.absolute(0x46cu16), width);
                w.load(v, width);
                0xb7e3
            }
            0xb7e3 => {
                w.compare_immediate(2, width);
                0xb7e5
            }
            0xb7e5 => {
                let taken = w.branch(w.cpu.p & CARRY == 0);
                if taken {
                    0xb840
                } else {
                    0xb7e7
                }
            }
            0xb7e7 => {
                w.drain(16);
                let v = w.read(w.absolute(0x46cu16), width);
                w.load(v, width);
                0xb7ea
            }
            0xb7ea => {
                w.compare_immediate(3, width);
                0xb7ec
            }
            0xb7ec => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb840
                } else {
                    0xb7ee
                }
            }
            0xb7ee => {
                w.drain(8 * u32::from(width));
                let v = 2;
                w.load(v, width);
                0xb7f0
            }
            0xb7f0 => {
                w.drain(16);
                let address = w.absolute(0x315u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb7f3
            }
            0xb7f3 => {
                w.width(32, true);
                0xb7f5
            }
            0xb7f5 => {
                w.item_call();
                self.tail = Some(CardinalCollisionTail::PrimaryFootprint);
                0xcf12
            }
            0xb7f8 => {
                w.width(32, false);
                0xb7fa
            }
            0xb7fa => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xb7fc
            }
            0xb7fc => {
                w.drain(16);
                let address = w.absolute(0x316u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb7ff
            }
            0xb7ff => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb840
                } else {
                    0xb801
                }
            }
            0xb801 => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                w.load(v, width);
                0xb803
            }
            0xb803 => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb805
            }
            0xb805 => {
                w.clear_carry();
                0xb806
            }
            0xb806 => {
                w.drain(16);
                let v = w.read(w.absolute(0x310u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xb809
            }
            0xb809 => {
                w.drain(8);
                let address = u32::from(0x30u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb80b
            }
            0xb80b => {
                w.drain(8);
                let v = w.read(u32::from(0x31u16), width);
                w.load(v, width);
                0xb80d
            }
            0xb80d => {
                w.drain(8);
                let address = u32::from(0x1u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb80f
            }
            0xb80f => {
                w.clear_carry();
                0xb810
            }
            0xb810 => {
                w.drain(16);
                let v = w.read(w.absolute(0x312u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xb813
            }
            0xb813 => {
                w.drain(8);
                let address = u32::from(0x31u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb815
            }
            0xb815 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xb817
            }
            0xb817 => {
                w.compare_immediate(12, width);
                0xb819
            }
            0xb819 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb840
                } else {
                    0xb81b
                }
            }
            0xb81b => {
                w.compare_immediate(3, width);
                0xb81d
            }
            0xb81d => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb840
                } else {
                    0xb81f
                }
            }
            0xb81f => {
                w.compare_immediate(10, width);
                0xb821
            }
            0xb821 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb845
                } else {
                    0xb823
                }
            }
            0xb823 => {
                w.compare_immediate(5, width);
                0xb825
            }
            0xb825 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb845
                } else {
                    0xb827
                }
            }
            0xb827 => {
                w.mask(12, width);
                0xb829
            }
            0xb829 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb833
                } else {
                    0xb82b
                }
            }
            0xb82b => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xb82d
            }
            0xb82d => {
                w.mask(3, width);
                0xb82f
            }
            0xb82f => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb833
                } else {
                    0xb831
                }
            }
            0xb831 => {
                w.branch(true);
                0xb840
            }
            0xb833 => {
                w.drain(8);
                let v = w.read(u32::from(0x0u16), width);
                w.load(v, width);
                0xb835
            }
            0xb835 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb845
                } else {
                    0xb837
                }
            }
            0xb837 => {
                w.drain(8);
                let v = w.read(u32::from(0x1u16), width);
                w.load(v, width);
                0xb839
            }
            0xb839 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb840
                } else {
                    0xb83b
                }
            }
            0xb83b => {
                w.drain(16);
                let v = w.read(w.absolute(0x310u16), width);
                w.load(v, width);
                0xb83e
            }
            0xb83e => {
                let taken = w.branch(w.cpu.p & NEGATIVE == 0);
                if taken {
                    0xb845
                } else {
                    0xb840
                }
            }
            0xb840 => {
                w.item_call();
                self.primary_layer = true;
                self.horizontal_first = false;
                self.vertical_completed = false;
                0xb956
            }
            0xb843 => {
                w.branch(true);
                0xb848
            }
            0xb845 => {
                w.item_call();
                self.primary_layer = true;
                self.horizontal_first = true;
                self.vertical_completed = false;
                0xb969
            }
            0xb848 => {
                w.item_call();
                0xb9b3
            }
            0xb84b => {
                w.drain(16);
                let v = w.read(w.absolute(0x46cu16), width);
                w.load(v, width);
                0xb84e
            }
            0xb84e => {
                w.compare_immediate(2, width);
                0xb850
            }
            0xb850 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb88f
                } else {
                    0xb852
                }
            }
            0xb852 => {
                w.compare_immediate(3, width);
                0xb854
            }
            0xb854 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb8cb
                } else {
                    0xb856
                }
            }
            0xb856 => {
                w.compare_immediate(4, width);
                0xb858
            }
            0xb858 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb8c1
                } else {
                    0xb85a
                }
            }
            0xb85a => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                w.load(v, width);
                0xb85c
            }
            0xb85c => {
                w.drain(8);
                let v = w.read(u32::from(0x31u16), width);
                w.load((w.cpu.a | v) & if width == 1 { 255 } else { 65535 }, width);
                0xb85e
            }
            0xb85e => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb8c1
                } else {
                    0xb860
                }
            }
            0xb860 => {
                w.drain(8);
                let v = w.read(u32::from(0x5du16), width);
                w.load(v, width);
                0xb862
            }
            0xb862 => {
                w.compare_immediate(19, width);
                0xb864
            }
            0xb864 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb88c
                } else {
                    0xb866
                }
            }
            0xb866 => {
                w.compare_immediate(8, width);
                0xb868
            }
            0xb868 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb88c
                } else {
                    0xb86a
                }
            }
            0xb86a => {
                w.compare_immediate(9, width);
                0xb86c
            }
            0xb86c => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb88c
                } else {
                    0xb86e
                }
            }
            0xb86e => {
                w.compare_immediate(10, width);
                0xb870
            }
            0xb870 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb88c
                } else {
                    0xb872
                }
            }
            0xb872 => {
                w.compare_immediate(3, width);
                0xb874
            }
            0xb874 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb88c
                } else {
                    0xb876
                }
            }
            0xb876 => {
                w.item_call();
                self.tail = Some(CardinalCollisionTail::FinalFootprint {
                    collision_two: false,
                });
                0xcf12
            }
            0xb879 => {
                w.drain(8);
                let v = w.read(u32::from(0x59u16), width);
                w.load(v, width);
                0xb87b
            }
            0xb87b => {
                w.mask(15, width);
                0xb87d
            }
            0xb87d => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb88c
                } else {
                    0xb87f
                }
            }
            0xb87f => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xb881
            }
            0xb881 => {
                w.drain(8);
                let address = u32::from(0x5du16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb883
            }
            0xb883 => {
                w.drain(16);
                let v = w.read(w.absolute(0x372u16), width);
                w.load(v, width);
                0xb886
            }
            0xb886 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb88c
                } else {
                    0xb888
                }
            }
            0xb888 => {
                w.drain(8 * u32::from(width));
                let v = 4;
                w.load(v, width);
                0xb88a
            }
            0xb88a => {
                w.drain(8);
                let address = u32::from(0x5eu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb88c
            }
            0xb88c => {
                w.operand_then_internal(2);
                0xb8d3
            }
            0xb88f => {
                w.item_call();
                self.tail = Some(CardinalCollisionTail::FinalFootprint {
                    collision_two: true,
                });
                0xcf12
            }
            0xb892 => {
                w.drain(8);
                let v = w.read(u32::from(0xeu16), width);
                w.load(v, width);
                0xb894
            }
            0xb894 => {
                w.drain(16);
                let v = w.read(w.absolute(0x316u16), width);
                w.load((w.cpu.a | v) & if width == 1 { 255 } else { 65535 }, width);
                0xb897
            }
            0xb897 => {
                w.compare_immediate(15, width);
                0xb899
            }
            0xb899 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb8c1
                } else {
                    0xb89b
                }
            }
            0xb89b => {
                w.drain(16);
                let v = w.read(w.absolute(0x31fu16), width);
                w.load(v, width);
                0xb89e
            }
            0xb89e => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb8a5
                } else {
                    0xb8a0
                }
            }
            0xb8a0 => {
                w.drain(8 * u32::from(width));
                let v = 58;
                w.load(v, width);
                0xb8a2
            }
            0xb8a2 => {
                w.drain(16);
                let address = w.absolute(0x31fu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb8a5
            }
            0xb8a5 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xb8a7
            }
            0xb8a7 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb8c1
                } else {
                    0xb8a9
                }
            }
            0xb8a9 => {
                w.drain(16);
                let v = w.read(w.absolute(0x310u16), width);
                w.load(v, width);
                0xb8ac
            }
            0xb8ac => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb8b5
                } else {
                    0xb8ae
                }
            }
            0xb8ae => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                w.load(v, width);
                0xb8b0
            }
            0xb8b0 => {
                w.drain(8 * u32::from(width));
                let v = 255;
                w.load((w.cpu.a ^ v) & if width == 1 { 255 } else { 65535 }, width);
                0xb8b2
            }
            0xb8b2 => {
                w.increment_a(width);
                0xb8b3
            }
            0xb8b3 => {
                w.drain(8);
                let address = u32::from(0x30u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb8b5
            }
            0xb8b5 => {
                w.drain(16);
                let v = w.read(w.absolute(0x312u16), width);
                w.load(v, width);
                0xb8b8
            }
            0xb8b8 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb8c1
                } else {
                    0xb8ba
                }
            }
            0xb8ba => {
                w.drain(8);
                let v = w.read(u32::from(0x31u16), width);
                w.load(v, width);
                0xb8bc
            }
            0xb8bc => {
                w.drain(8 * u32::from(width));
                let v = 255;
                w.load((w.cpu.a ^ v) & if width == 1 { 255 } else { 65535 }, width);
                0xb8be
            }
            0xb8be => {
                w.increment_a(width);
                0xb8bf
            }
            0xb8bf => {
                w.drain(8);
                let address = u32::from(0x31u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb8c1
            }
            0xb8c1 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xb8c3
            }
            0xb8c3 => {
                w.drain(16);
                let address = w.absolute(0x315u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb8c6
            }
            0xb8c6 => {
                w.item_call();
                self.primary_layer = false;
                self.horizontal_first = false;
                self.vertical_completed = false;
                0xb956
            }
            0xb8c9 => {
                w.branch(true);
                0xb8d3
            }
            0xb8cb => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xb8cd
            }
            0xb8cd => {
                w.drain(16);
                let address = w.absolute(0x315u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb8d0
            }
            0xb8d0 => {
                w.item_call();
                self.primary_layer = false;
                self.horizontal_first = true;
                self.vertical_completed = false;
                0xb969
            }
            0xb8d3 => {
                w.item_y(0, 1);
                0xb8d5
            }
            0xb8d5 => {
                w.item_call();
                self.tail = Some(CardinalCollisionTail::MainHandler);
                0xd077
            }
            0xb956 => {
                w.drain(8);
                let v = w.read(u32::from(0x6bu16), width);
                w.load(v, width);
                0xb958
            }
            0xb958 => {
                w.mask(32, width);
                0xb95a
            }
            0xb95a => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb95f
                } else {
                    0xb95c
                }
            }
            0xb95c => {
                w.item_call();
                self.tail = Some(CardinalCollisionTail::SlopeChecks {
                    primary_layer: self.primary_layer,
                    horizontal_first: self.horizontal_first,
                    vertical_completed: self.vertical_completed,
                });
                0xba0a
            }
            0xb95f => {
                w.drain(8);
                let v = w.read(u32::from(0x6bu16), width);
                w.load(v, width);
                0xb961
            }
            0xb961 => {
                w.mask(16, width);
                0xb963
            }
            0xb963 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb968
                } else {
                    0xb965
                }
            }
            0xb965 => {
                w.item_call();
                0xc4d4
            }
            0xb968 => {
                w.rts();
                w.cpu.pc
            }
            0xb969 => {
                w.drain(8);
                let v = w.read(u32::from(0x6bu16), width);
                w.load(v, width);
                0xb96b
            }
            0xb96b => {
                w.mask(16, width);
                0xb96d
            }
            0xb96d => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb972
                } else {
                    0xb96f
                }
            }
            0xb96f => {
                w.item_call();
                0xc4d4
            }
            0xb972 => {
                w.drain(8);
                let v = w.read(u32::from(0x6bu16), width);
                w.load(v, width);
                0xb974
            }
            0xb974 => {
                w.mask(32, width);
                0xb976
            }
            0xb976 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb97b
                } else {
                    0xb978
                }
            }
            0xb978 => {
                w.item_call();
                self.tail = Some(CardinalCollisionTail::SlopeChecks {
                    primary_layer: self.primary_layer,
                    horizontal_first: self.horizontal_first,
                    vertical_completed: self.vertical_completed,
                });
                0xba0a
            }
            0xb97b => {
                w.rts();
                w.cpu.pc
            }
            0xc4d4 => {
                w.drain(8);
                let value = w.read(0x31, width);
                w.load(value, width);
                0xc4d6
            }
            0xc4d6 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    self.tail = Some(CardinalCollisionTail::SlopeChecks {
                        primary_layer: self.primary_layer,
                        horizontal_first: self.horizontal_first,
                        vertical_completed: self.vertical_completed,
                    });
                    0xc4d9
                } else {
                    0xc4d8
                }
            }
            0xc4d8 => {
                w.rts();
                assert!(matches!(w.cpu.pc, 0xb968 | 0xb972));
                w.cpu.pc
            }
            0xb97c => {
                w.drain(16);
                let v = w.read(w.absolute(0x46cu16), width);
                w.load(v, width);
                0xb97f
            }
            0xb97f => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb9b1
                } else {
                    0xb981
                }
            }
            0xb981 => {
                w.compare_immediate(4, width);
                0xb983
            }
            0xb983 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb9b1
                } else {
                    0xb985
                }
            }
            0xb985 => {
                w.compare_immediate(2, width);
                0xb987
            }
            0xb987 => {
                let taken = w.branch(w.cpu.p & CARRY == 0);
                if taken {
                    0xb9ab
                } else {
                    0xb989
                }
            }
            0xb989 => {
                w.compare_immediate(3, width);
                0xb98b
            }
            0xb98b => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xb98d
                } else {
                    0xb98d
                }
            }
            0xb98d => {
                w.width(32, true);
                0xb98f
            }
            0xb98f => {
                w.drain(8);
                let v = w.read(u32::from(0xe6u16), width);
                w.load(v, width);
                0xb991
            }
            0xb991 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb992
            }
            0xb992 => {
                w.drain(8);
                let v = w.read(u32::from(0xe8u16), width);
                w.sbc(v, width);
                0xb994
            }
            0xb994 => {
                w.clear_carry();
                0xb995
            }
            0xb995 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xb997
            }
            0xb997 => {
                w.drain(8);
                let address = u32::from(0x20u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb999
            }
            0xb999 => {
                w.drain(16);
                let address = w.absolute(0x318u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb99c
            }
            0xb99c => {
                w.drain(8);
                let v = w.read(u32::from(0xe0u16), width);
                w.load(v, width);
                0xb99e
            }
            0xb99e => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb99f
            }
            0xb99f => {
                w.drain(8);
                let v = w.read(u32::from(0xe2u16), width);
                w.sbc(v, width);
                0xb9a1
            }
            0xb9a1 => {
                w.clear_carry();
                0xb9a2
            }
            0xb9a2 => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xb9a4
            }
            0xb9a4 => {
                w.drain(8);
                let address = u32::from(0x22u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb9a6
            }
            0xb9a6 => {
                w.drain(16);
                let address = w.absolute(0x31au16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb9a9
            }
            0xb9a9 => {
                w.width(32, false);
                0xb9ab
            }
            0xb9ab => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xb9ad
            }
            0xb9ad => {
                w.drain(8);
                let address = u32::from(0xeeu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb9af
            }
            0xb9af => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb9b0
            }
            0xb9b0 => {
                w.rts();
                w.cpu.pc
            }
            0xb9b1 => {
                w.clear_carry();
                0xb9b2
            }
            0xb9b2 => {
                w.rts();
                w.cpu.pc
            }
            0xb9b3 => {
                w.drain(16);
                let v = w.read(w.absolute(0x46cu16), width);
                w.load(v, width);
                0xb9b6
            }
            0xb9b6 => {
                w.compare_immediate(1, width);
                0xb9b8
            }
            0xb9b8 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb9f4
                } else {
                    0xb9ba
                }
            }
            0xb9ba => {
                w.width(32, true);
                0xb9bc
            }
            0xb9bc => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xb9be
            }
            0xb9be => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb9bf
            }
            0xb9bf => {
                w.drain(16);
                let v = w.read(w.absolute(0x318u16), width);
                w.sbc(v, width);
                0xb9c2
            }
            0xb9c2 => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb9c4
            }
            0xb9c4 => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xb9c6
            }
            0xb9c6 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb9c7
            }
            0xb9c7 => {
                w.drain(16);
                let v = w.read(w.absolute(0x31au16), width);
                w.sbc(v, width);
                0xb9ca
            }
            0xb9ca => {
                w.drain(8);
                let address = u32::from(0x2u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb9cc
            }
            0xb9cc => {
                w.drain(8);
                let v = w.read(u32::from(0xe8u16), width);
                w.load(v, width);
                0xb9ce
            }
            0xb9ce => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb9cf
            }
            0xb9cf => {
                w.drain(8);
                let v = w.read(u32::from(0xe6u16), width);
                w.sbc(v, width);
                0xb9d1
            }
            0xb9d1 => {
                w.clear_carry();
                0xb9d2
            }
            0xb9d2 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xb9d4
            }
            0xb9d4 => {
                w.drain(8);
                let address = u32::from(0x20u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb9d6
            }
            0xb9d6 => {
                w.drain(8);
                let v = w.read(u32::from(0xe2u16), width);
                w.load(v, width);
                0xb9d8
            }
            0xb9d8 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb9d9
            }
            0xb9d9 => {
                w.drain(8);
                let v = w.read(u32::from(0xe0u16), width);
                w.sbc(v, width);
                0xb9db
            }
            0xb9db => {
                w.clear_carry();
                0xb9dc
            }
            0xb9dc => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xb9de
            }
            0xb9de => {
                w.drain(8);
                let address = u32::from(0x22u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb9e0
            }
            0xb9e0 => {
                w.width(32, false);
                0xb9e2
            }
            0xb9e2 => {
                w.drain(8);
                let v = w.read(u32::from(0x67u16), width);
                w.load(v, width);
                0xb9e4
            }
            0xb9e4 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xb9f4
                } else {
                    0xb9e6
                }
            }
            0xb9e6 => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                w.load(v, width);
                0xb9e8
            }
            0xb9e8 => {
                w.clear_carry();
                0xb9e9
            }
            0xb9e9 => {
                w.drain(8);
                let v = w.read(u32::from(0x0u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xb9eb
            }
            0xb9eb => {
                w.drain(8);
                let address = u32::from(0x30u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb9ed
            }
            0xb9ed => {
                w.drain(8);
                let v = w.read(u32::from(0x31u16), width);
                w.load(v, width);
                0xb9ef
            }
            0xb9ef => {
                w.clear_carry();
                0xb9f0
            }
            0xb9f0 => {
                w.drain(8);
                let v = w.read(u32::from(0x2u16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xb9f2
            }
            0xb9f2 => {
                w.drain(8);
                let address = u32::from(0x31u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xb9f4
            }
            0xb9f4 => {
                w.drain(8);
                let address = u32::from(0xeeu16);
                w.write(address, 0, width);
                0xb9f6
            }
            0xb9f6 => {
                w.rts();
                w.cpu.pc
            }
            0xccab => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                w.load(v, width);
                0xccad
            }
            0xccad => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xccb3
                } else {
                    0xccaf
                }
            }
            0xccaf => {
                w.drain(8);
                let v = w.read(u32::from(0x31u16), width);
                w.load(v, width);
                0xccb1
            }
            0xccb1 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xccb6
                } else {
                    0xccb3
                }
            }
            0xccb3 => {
                w.operand_then_internal(2);
                0xcd76
            }
            0xccb6 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xccb8
            }
            0xccb8 => {
                w.drain(16);
                let address = w.absolute(0x2deu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xccbb
            }
            0xccbb => {
                w.drain(8);
                let v = w.read(u32::from(0x21u16), width);
                w.load(v, width);
                0xccbd
            }
            0xccbd => {
                w.drain(16);
                let address = w.absolute(0x2dfu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xccc0
            }
            0xccc0 => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xccc2
            }
            0xccc2 => {
                w.drain(16);
                let address = w.absolute(0x2dcu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xccc5
            }
            0xccc5 => {
                w.drain(8);
                let v = w.read(u32::from(0x23u16), width);
                w.load(v, width);
                0xccc7
            }
            0xccc7 => {
                w.drain(16);
                let address = w.absolute(0x2ddu16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xccca
            }
            0xccca => {
                w.item_y(4, 1);
                0xcccc
            }
            0xcccc => {
                w.drain(8);
                let v = w.read(u32::from(0x31u16), width);
                w.load(v, width);
                0xccce
            }
            0xccce => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xccd2
                } else {
                    0xccd0
                }
            }
            0xccd0 => {
                w.item_y(6, 1);
                0xccd2
            }
            0xccd2 => {
                w.item_call();
                self.tail = Some(CardinalCollisionTail::KickbackHorizontal {
                    direction: (w.cpu.y / 2) as u8,
                });
                0xce2a
            }
            0xcd76 => {
                w.drain(8);
                let address = u32::from(0x6du16);
                w.write(address, 0, width);
                0xcd78
            }
            0xcd78 => {
                w.drain(8);
                let address = u32::from(0x6bu16);
                w.write(address, 0, width);
                0xcd7a
            }
            0xcd7a => {
                w.rts();
                w.cpu.pc
            }
            _ => panic!("ground velocity PC {pc:04x}"),
        };
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
    fn cardinal_collision_matches_source_kickback_layers_and_child_selection() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;

        for timer in 0..=255u8 {
            for variant in 0..32u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7) ^ timer as usize) as u8;
                }
                r[0x1f5..0x1f7].copy_from_slice(&[0x1b, 0x80]);
                r[0x6b] = [0, 0x10, 0x20, 0x30][(variant & 3) as usize];
                r[0x30] = [0, 1, 127, 128, 255][(timer as usize) % 5];
                r[0x31] = [0, 1, 127, 128, 255][((timer as usize) / 5) % 5];
                r[0x46c] = variant % 5;
                r[0x5d] = [0, 3, 4, 8, 9, 10, 19, 11][((timer >> 5) & 7) as usize];
                for (n, address) in [0x20, 0x22, 0xe0, 0xe2, 0xe6, 0xe8].into_iter().enumerate() {
                    r[address..address + 2].copy_from_slice(
                        &[0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa]
                            [((timer as usize) + n) % 8]
                            .to_le_bytes(),
                    );
                }
                let pc = 0x82c2;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: variant as u16,
                    d: 0,
                    db: variant,
                    bank: 7,
                    pc,
                    sp: 0x1f4,
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
                let mut work = CardinalCollisionPrefix::default();
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
                        |_, _| None,
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                match work.tail().unwrap() {
                    CardinalCollisionTail::KickbackHorizontal { direction } => {
                        assert_eq!((cpu.pc, cpu.sp, cpu.p & 0x30), (0xce2a, 0x1ee, 0x30));
                        assert!(direction == 2 || direction == 3);
                        assert_eq!(direction, u8::try_from(cpu.y / 2).unwrap());
                    }
                    CardinalCollisionTail::PrimaryFootprint => {
                        assert_eq!((cpu.pc, cpu.sp, cpu.p & 0x30), (0xcf12, 0x1f0, 0x10))
                    }
                    CardinalCollisionTail::FinalFootprint { .. } => {
                        assert_eq!((cpu.pc, cpu.sp, cpu.p & 0x30), (0xcf12, 0x1f0, 0x30))
                    }
                    CardinalCollisionTail::SlopeChecks { .. } => {
                        assert!(matches!(cpu.pc, 0xba0a | 0xc4d9))
                    }
                    CardinalCollisionTail::MainHandler => {
                        assert_eq!((cpu.pc, cpu.sp, cpu.y), (0xd077, 0x1f0, 0))
                    }
                }
            }
        }

        println!("cardinal collision prefix: 8192 cases, {instructions} source instructions, kickback, moving-floor layers and child selection");
    }
    #[test]
    fn cardinal_collision_after_footprint_matches_source_floor_axis_and_pit_response() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;

        for timer in 0..=255u8 {
            for variant in 0..16u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7) ^ timer as usize) as u8;
                }
                r[0x1f5..0x1f7].copy_from_slice(&[0x1b, 0x80]);
                r[0x6b] = [0, 0x10, 0x20, 0x30][(variant & 3) as usize];
                r[0x30] = [0, 1, 127, 128, 255][(timer as usize) % 5];
                r[0x31] = [0, 1, 127, 128, 255][((timer as usize) / 5) % 5];
                r[0x46c] = variant % 5;
                r[0x5d] = [0, 3, 4, 8, 9, 10, 19, 11][((timer >> 5) & 7) as usize];
                for (n, address) in [0x20, 0x22, 0xe0, 0xe2, 0xe6, 0xe8].into_iter().enumerate() {
                    r[address..address + 2].copy_from_slice(
                        &[0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa]
                            [((timer as usize) + n) % 8]
                            .to_le_bytes(),
                    );
                }
                r[0xe] = timer & 15;
                r[0x316] = (timer >> 4) & 15;
                r[0x59] = timer & 15;
                r[0x372] = timer & 1;
                r[0x31f] = timer & 2;
                r[0x67] = timer & 15;
                r[0x310] = [0, 1, 127, 128, 255][(timer as usize) % 5];
                r[0x312] = [0, 1, 127, 128, 255][((timer as usize) / 5) % 5];
                let pc = if variant < 8 {
                    0xb7f8
                } else if variant < 12 {
                    0xb892
                } else {
                    0xb879
                };
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: variant as u16,
                    d: 0,
                    db: variant,
                    bank: 7,
                    pc,
                    sp: 0x1f2,
                    p: (if pc == 0xb7f8 { 0x10 } else { 0x30 }) | (timer & 0xc3),
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
                let mut work = CardinalCollisionPrefix {
                    pc: Some(pc),
                    ..Default::default()
                };
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
                        |_, _| None,
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                match work.tail().unwrap() {
                    CardinalCollisionTail::KickbackHorizontal { direction } => {
                        assert_eq!((cpu.pc, cpu.sp, cpu.p & 0x30), (0xce2a, 0x1ee, 0x30));
                        assert!(direction == 2 || direction == 3);
                        assert_eq!(direction, u8::try_from(cpu.y / 2).unwrap());
                    }
                    CardinalCollisionTail::PrimaryFootprint => {
                        assert_eq!((cpu.pc, cpu.sp, cpu.p & 0x30), (0xcf12, 0x1f0, 0x10))
                    }
                    CardinalCollisionTail::FinalFootprint { .. } => {
                        assert_eq!((cpu.pc, cpu.sp, cpu.p & 0x30), (0xcf12, 0x1f0, 0x30))
                    }
                    CardinalCollisionTail::SlopeChecks { .. } => {
                        assert!(matches!(cpu.pc, 0xba0a | 0xc4d9))
                    }
                    CardinalCollisionTail::MainHandler => {
                        assert_eq!((cpu.pc, cpu.sp, cpu.y), (0xd077, 0x1f0, 0))
                    }
                }
            }
        }

        println!("cardinal after footprint: 4096 cases, {instructions} source instructions, kickback, moving-floor layers and child selection");
    }
}
