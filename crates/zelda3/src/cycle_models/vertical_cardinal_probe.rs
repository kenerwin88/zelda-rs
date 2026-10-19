//! Vertical cardinal direction selection, reset and three-sample geometry.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
    tile_classifier::TileClassifierTail,
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    [
        8, 0, 24, 0, 0, 0, 15, 0, 0, 0, 0, 0, 8, 0, 8, 0, 8, 0, 8, 0, 16, 0, 16, 0, 15, 0, 15, 0,
        23, 0, 23, 0,
    ]
    .get(address.checked_sub(0x07cd7b)? as usize)
    .copied()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VerticalCardinalTail {
    Returned,
    Response,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VerticalCardinalProbe {
    pc: Option<u16>,
    tail: Option<VerticalCardinalTail>,
    child_ready: bool,
    sample: u8,
    direction: u8,
    x: [u16; 3],
    y: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VerticalCardinalHandoff {
    pub direction: u8,
    pub x: [u16; 3],
    pub y: u16,
    pub samples: [Option<TileClassifierTail>; 3],
}
impl VerticalCardinalProbe {
    pub(crate) fn tail(&self) -> Option<VerticalCardinalTail> {
        self.tail
    }
    pub(crate) fn child_ready(&self) -> bool {
        self.child_ready
    }
    pub(crate) fn sample(&self) -> usize {
        usize::from(self.sample)
    }
    pub(crate) fn handoff(
        &self,
        samples: [Option<TileClassifierTail>; 3],
    ) -> VerticalCardinalHandoff {
        assert!(self.child_ready || self.tail == Some(VerticalCardinalTail::Response));
        VerticalCardinalHandoff {
            direction: self.direction,
            x: self.x,
            y: self.y,
            samples,
        }
    }
    pub(crate) fn begin_next(&mut self) {
        assert!(self.child_ready && self.sample < 3);
        self.child_ready = false;
        self.pc = Some([0xce0f, 0xce1b, 0xce27][self.sample()]);
        self.sample += 1;
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.tail.is_some() || self.child_ready {
            return None;
        }
        let pc = self.pc.unwrap_or(0xba0a);
        let wide_index = (0xce07..=0xce27).contains(&pc);
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x18),
            (7, pc, 0, 7, if wide_index { 0 } else { 0x10 })
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
            0xba0a => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                w.load(v, width);
                0xba0c
            }
            0xba0c => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xba0f
                } else {
                    0xba0e
                }
            }
            0xba0e => {
                w.rts();
                self.tail = Some(VerticalCardinalTail::Returned);
                assert!(matches!(w.cpu.pc, 0xb95f | 0xb97b));
                w.cpu.pc
            }
            0xba0f => {
                w.drain(8);
                let v = w.read(u32::from(0x6cu16), width);
                w.load(v, width);
                0xba11
            }
            0xba11 => {
                w.compare_immediate(1, width);
                0xba13
            }
            0xba13 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xba1f
                } else {
                    0xba15
                }
            }
            0xba15 => {
                w.item_y(0, 1);
                0xba17
            }
            0xba17 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xba19
            }
            0xba19 => {
                w.compare_immediate(128, width);
                0xba1b
            }
            0xba1b => {
                let taken = w.branch(w.cpu.p & CARRY == 0);
                if taken {
                    0xba27
                } else {
                    0xba1d
                }
            }
            0xba1d => {
                w.branch(true);
                0xba25
            }
            0xba1f => {
                w.item_y(0, 1);
                0xba21
            }
            0xba21 => {
                w.drain(8);
                let v = w.read(u32::from(0x30u16), width);
                w.load(v, width);
                0xba23
            }
            0xba23 => {
                let taken = w.branch(w.cpu.p & NEGATIVE != 0);
                if taken {
                    0xba27
                } else {
                    0xba25
                }
            }
            0xba25 => {
                w.item_y(2, 1);
                0xba27
            }
            0xba27 => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xba28
            }
            0xba28 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xba29
            }
            0xba29 => {
                w.drain(8);
                let address = u32::from(0x66u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                self.direction = w.cpu.a as u8;
                0xba2b
            }
            0xba2b => {
                w.item_call();
                0xcdcb
            }
            0xcdcb => {
                w.width(32, true);
                0xcdcd
            }
            0xcdcd => {
                w.item_call();
                0xd798
            }
            0xcdd0 => {
                w.drain(8);
                let address = u32::from(0x59u16);
                w.write(address, 0, width);
                0xcdd2
            }
            0xcdd2 => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xcdd4
            }
            0xcdd4 => {
                w.clear_carry();
                0xcdd5
            }
            0xcdd5 => {
                w.drain(16);
                if (0xcd7bu16 + w.cpu.y) & 0xff00 != 0xcd00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xcd7bu16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xcdd8
            }
            0xcdd8 => {
                w.drain(8);
                let address = u32::from(0x51u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xcdda
            }
            0xcdda => {
                w.drain(8);
                let v = w.read(u32::from(0xecu16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xcddc
            }
            0xcddc => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                self.y = w.cpu.a;
                0xcdde
            }
            0xcdde => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xcde0
            }
            0xcde0 => {
                w.clear_carry();
                0xcde1
            }
            0xcde1 => {
                w.drain(16);
                if (0xcd83u16 + w.cpu.y) & 0xff00 != 0xcd00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xcd83u16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xcde4
            }
            0xcde4 => {
                w.drain(8);
                let v = w.read(u32::from(0xecu16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xcde6
            }
            0xcde6 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xcde7
            }
            0xcde7 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xcde8
            }
            0xcde8 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xcde9
            }
            0xcde9 => {
                w.drain(8);
                let address = u32::from(0x2u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                self.x[0] = w.cpu.a;
                0xcdeb
            }
            0xcdeb => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xcded
            }
            0xcded => {
                w.clear_carry();
                0xcdee
            }
            0xcdee => {
                w.drain(16);
                if (0xcd8bu16 + w.cpu.y) & 0xff00 != 0xcd00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xcd8bu16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xcdf1
            }
            0xcdf1 => {
                w.drain(8);
                let v = w.read(u32::from(0xecu16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xcdf3
            }
            0xcdf3 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xcdf4
            }
            0xcdf4 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xcdf5
            }
            0xcdf5 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xcdf6
            }
            0xcdf6 => {
                w.drain(8);
                let address = u32::from(0x4u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                self.x[1] = w.cpu.a;
                0xcdf8
            }
            0xcdf8 => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xcdfa
            }
            0xcdfa => {
                w.clear_carry();
                0xcdfb
            }
            0xcdfb => {
                w.drain(16);
                if (0xcd93u16 + w.cpu.y) & 0xff00 != 0xcd00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xcd93u16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xcdfe
            }
            0xcdfe => {
                w.drain(8);
                let v = w.read(u32::from(0xecu16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xce00
            }
            0xce00 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xce01
            }
            0xce01 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xce02
            }
            0xce02 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xce03
            }
            0xce03 => {
                w.drain(8);
                let address = u32::from(0x74u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                self.x[2] = w.cpu.a;
                0xce05
            }
            0xce05 => {
                w.width(16, true);
                0xce07
            }
            0xce07 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xce0a
            }
            0xce0a => {
                w.drain(8);
                let address = u32::from(0xau16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xce0c
            }
            0xce0c => {
                w.item_call();
                self.child_ready = true;
                0xd9d8
            }
            0xce0f => {
                w.drain(8);
                let v = w.read(u32::from(0x4u16), width);
                w.load(v, width);
                0xce11
            }
            0xce11 => {
                w.drain(8);
                let address = u32::from(0x2u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xce13
            }
            0xce13 => {
                w.drain(8 * u32::from(width));
                let v = 2;
                w.load(v, width);
                0xce16
            }
            0xce16 => {
                w.drain(8);
                let address = u32::from(0xau16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xce18
            }
            0xce18 => {
                w.item_call();
                self.child_ready = true;
                0xd9d8
            }
            0xce1b => {
                w.drain(8);
                let v = w.read(u32::from(0x74u16), width);
                w.load(v, width);
                0xce1d
            }
            0xce1d => {
                w.drain(8);
                let address = u32::from(0x2u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xce1f
            }
            0xce1f => {
                w.drain(8 * u32::from(width));
                let v = 4;
                w.load(v, width);
                0xce22
            }
            0xce22 => {
                w.drain(8);
                let address = u32::from(0xau16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xce24
            }
            0xce24 => {
                w.item_call();
                self.child_ready = true;
                0xd9d8
            }
            0xce27 => {
                w.width(48, false);
                0xce29
            }
            0xce29 => {
                w.rts();
                self.tail = Some(VerticalCardinalTail::Response);
                assert_eq!(w.cpu.pc, 0xba2e);
                w.cpu.pc
            }
            0xd798 => {
                w.drain(8);
                let address = u32::from(0xcu16);
                w.write(address, 0, width);
                0xd79a
            }
            0xd79a => {
                w.drain(8);
                let address = u32::from(0xeu16);
                w.write(address, 0, width);
                0xd79c
            }
            0xd79c => {
                w.drain(8);
                let address = u32::from(0x38u16);
                w.write(address, 0, width);
                0xd79e
            }
            0xd79e => {
                w.drain(8);
                let address = u32::from(0x58u16);
                w.write(address, 0, width);
                0xd7a0
            }
            0xd7a0 => {
                w.drain(16);
                let address = w.absolute(0x2c0u16);
                w.write(address, 0, width);
                0xd7a3
            }
            0xd7a3 => {
                w.drain(8);
                let address = u32::from(0x5fu16);
                w.write(address, 0, width);
                0xd7a5
            }
            0xd7a5 => {
                w.drain(8);
                let address = u32::from(0x62u16);
                w.write(address, 0, width);
                0xd7a7
            }
            0xd7a7 => {
                w.drain(16);
                let address = w.absolute(0x320u16);
                w.write(address, 0, width);
                0xd7aa
            }
            0xd7aa => {
                w.drain(16);
                let address = w.absolute(0x341u16);
                w.write(address, 0, width);
                0xd7ad
            }
            0xd7ad => {
                w.drain(16);
                let address = w.absolute(0x343u16);
                w.write(address, 0, width);
                0xd7b0
            }
            0xd7b0 => {
                w.drain(16);
                let address = w.absolute(0x348u16);
                w.write(address, 0, width);
                0xd7b3
            }
            0xd7b3 => {
                w.drain(16);
                let address = w.absolute(0x34cu16);
                w.write(address, 0, width);
                0xd7b6
            }
            0xd7b6 => {
                w.drain(16);
                let address = w.absolute(0x357u16);
                w.write(address, 0, width);
                0xd7b9
            }
            0xd7b9 => {
                w.drain(16);
                let address = w.absolute(0x359u16);
                w.write(address, 0, width);
                0xd7bc
            }
            0xd7bc => {
                w.drain(16);
                let address = w.absolute(0x35bu16);
                w.write(address, 0, width);
                0xd7bf
            }
            0xd7bf => {
                w.drain(16);
                let address = w.absolute(0x366u16);
                w.write(address, 0, width);
                0xd7c2
            }
            0xd7c2 => {
                w.drain(16);
                let address = w.absolute(0x36du16);
                w.write(address, 0, width);
                0xd7c5
            }
            0xd7c5 => {
                w.drain(16);
                let address = w.absolute(0x36fu16);
                w.write(address, 0, width);
                0xd7c8
            }
            0xd7c8 => {
                w.drain(16);
                let address = w.absolute(0x2e5u16);
                w.write(address, 0, width);
                0xd7cb
            }
            0xd7cb => {
                w.drain(16);
                let address = w.absolute(0x2e7u16);
                w.write(address, 0, width);
                0xd7ce
            }
            0xd7ce => {
                w.drain(16);
                let address = w.absolute(0x2eeu16);
                w.write(address, 0, width);
                0xd7d1
            }
            0xd7d1 => {
                w.drain(16);
                let address = w.absolute(0x2f6u16);
                w.write(address, 0, width);
                0xd7d4
            }
            0xd7d4 => {
                w.drain(16);
                let address = w.absolute(0x3f1u16);
                w.write(address, 0, width);
                0xd7d7
            }
            0xd7d7 => {
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
    use crate::cycle_models::tile_classifier::{TileClassifier, TileClassifierTail};
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn vertical_cardinal_matches_source_geometry_and_three_calls() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x07cd7b..0x07cd9b {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
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
                r[0x1ef..0x1f1].copy_from_slice(&[0x5e, 0xb9]);
                r[0x30] = if variant & 3 == 0 {
                    0
                } else {
                    [1, 127, 128, 255][(timer >> 6) as usize]
                };
                r[0x6c] = timer & 3;
                r[0x1b] = u8::from(variant & 8 == 0);
                r[0xee] = variant & 1;
                r[0x37f] = u8::from(variant & 16 != 0);
                r[0x12000..0x15000].fill(timer);
                let values = [0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa];
                r[0x20..0x22].copy_from_slice(&values[(timer & 7) as usize].to_le_bytes());
                r[0x22..0x24].copy_from_slice(&values[((timer >> 3) & 7) as usize].to_le_bytes());
                r[0xec..0xee].copy_from_slice(
                    &[0x1ffu16, 0xff, 0x3ff, 0x7f][(variant & 3) as usize].to_le_bytes(),
                );
                let pc = 0xba0a;
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
                let mut work = VerticalCardinalProbe::default();
                let mut classifiers: [Option<TileClassifier>; 3] = Default::default();
                while work.tail().is_none() {
                    let sample = work.sample();
                    if work.child_ready() && classifiers[sample].is_none() {
                        classifiers[sample] = Some(TileClassifier::default());
                    }
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
                                let byte = |address| {
                                    table_byte(address).or_else(|| {
                                        crate::cycle_models::tile_classifier::table_byte(address)
                                    })
                                };
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
                            if work.child_ready() {
                                classifiers[sample]
                                    .as_mut()
                                    .unwrap()
                                    .step(cpu, clock, bus)
                                    .unwrap();
                            } else {
                                work.step(cpu, clock, bus).unwrap();
                            }
                        },
                    );
                    work = work.clone();
                    classifiers = classifiers.clone();
                    if work.child_ready() && classifiers[sample].is_some() {
                        match classifiers[sample].as_ref().unwrap().tail() {
                            Some(TileClassifierTail::Complete) => work.begin_next(),
                            Some(_) => {
                                instructions += 1;
                                break;
                            }
                            None => {}
                        }
                    }
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                match work.tail() {
                    Some(VerticalCardinalTail::Returned) => {
                        assert_eq!((cpu.pc, cpu.sp), (0xb95f, 0x1f0))
                    }
                    Some(VerticalCardinalTail::Response) => {
                        assert_eq!((cpu.pc, cpu.sp, cpu.p & 0x30), (0xba2e, 0x1ee, 0x30));
                        assert!(classifiers
                            .iter()
                            .all(|c| c.as_ref().unwrap().tail()
                                == Some(TileClassifierTail::Complete)));
                    }
                    None => assert!(work.child_ready()),
                }
                if work.tail() != Some(VerticalCardinalTail::Returned) {
                    let tails = std::array::from_fn(|i| {
                        classifiers[i].as_ref().and_then(TileClassifier::tail)
                    });
                    let handoff = work.handoff(tails);
                    assert!(handoff.direction <= 1);
                    assert_eq!(handoff.y, u16::from_le_bytes([ram[0], ram[1]]));
                }
            }
        }
        println!("vertical cardinal probe: 8192 cases, {instructions} source instructions, three samples, byte neighbors, zero velocity and classifier handoffs");
    }
}
