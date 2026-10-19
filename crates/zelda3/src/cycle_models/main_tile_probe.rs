//! Main walking tile probe: shared reset, coordinate geometry and classifier call.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
    tile_classifier::TileClassifierTail,
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    [
        20, 0, 20, 0, 20, 0, 20, 0, 4, 0, 28, 0, 16, 0, 16, 0, 22, 0, 22, 0, 22, 0, 22, 0, 4, 0,
        24, 0, 16, 0, 16, 0, 16, 0, 16, 0, 16, 0, 16, 0, 20, 0, 20, 0, 23, 0, 23, 0, 252, 255, 36,
        0, 16, 0, 16, 0, 4, 0, 28, 0, 16, 0, 16, 0, 4, 0, 28, 0, 16, 0, 16, 0, 4, 0, 4, 0, 28, 0,
        28, 0, 8, 0, 8, 0, 8, 0, 8, 0, 6, 0, 8, 0, 255, 255, 22, 0, 19, 0, 19, 0, 0, 0, 19, 0, 6,
        0, 8, 0, 255, 255, 22, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 0, 0, 15, 0, 6, 0, 8, 0, 246,
        255, 29, 0, 6, 0, 8, 0, 250, 255, 22, 0, 6, 0, 8, 0, 252, 255, 22, 0, 252, 255, 22, 0, 252,
        255, 22, 0,
    ]
    .get(address.checked_sub(0x07cfcc)? as usize)
    .copied()
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MainTileProbe {
    pc: Option<u16>,
    complete: bool,
    child_ready: bool,
    item: u8,
    x: u16,
    y: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MainTileHandoff {
    pub item: u8,
    pub x: u16,
    pub y: u16,
    pub classifier: TileClassifierTail,
}
impl MainTileProbe {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn child_ready(&self) -> bool {
        self.child_ready
    }
    pub(crate) fn handoff(&self, classifier: TileClassifierTail) -> MainTileHandoff {
        assert!(self.complete || self.child_ready);
        MainTileHandoff {
            item: self.item,
            x: self.x,
            y: self.y,
            classifier,
        }
    }
    pub(crate) fn begin_response(&mut self) {
        assert!(self.child_ready);
        self.child_ready = false;
        self.pc = Some(0xd0f1);
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete || self.child_ready {
            return None;
        }
        let pc = self.pc.unwrap_or(0xd077);
        if self.pc.is_none() {
            assert!(matches!(cpu.y, 0 | 4 | 5));
            self.item = cpu.y as u8;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 8),
            (7, pc, 0, 7, 0)
        );
        let width = if cpu.p & 0x20 != 0 { 1 } else { 2 };
        let index_width = if cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match pc {
            0xd077 => {
                w.drain(8);
                let address = u32::from(0x59u16);
                w.write(address, 0, width);
                0xd079
            }
            0xd079 => {
                w.width(32, true);
                0xd07b
            }
            0xd07b => {
                w.item_call();
                0xd798
            }
            0xd07e => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(address, w.cpu.y, index_width);
                0xd080
            }
            0xd080 => {
                w.drain(8 * u32::from(index_width));
                let old = w.cpu.y;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old >= 8);
                w.flags(
                    old.wrapping_sub(8) & if index_width == 1 { 255 } else { 65535 },
                    index_width,
                );
                0xd082
            }
            0xd082 => {
                let taken = w.branch(w.cpu.p & ZERO == 0);
                if taken {
                    0xd0a2
                } else {
                    0xd084
                }
            }
            0xd0a2 => {
                w.drain(6);
                w.push(w.cpu.y, index_width);
                0xd0a3
            }
            0xd0a3 => {
                w.drain(8);
                let v = w.read(u32::from(0x0u16), width);
                w.load(v, width);
                0xd0a5
            }
            0xd0a5 => {
                w.mask(255, width);
                0xd0a8
            }
            0xd0a8 => {
                w.shift_left(width);
                0xd0a9
            }
            0xd0a9 => {
                w.shift_left(width);
                0xd0aa
            }
            0xd0aa => {
                w.shift_left(width);
                0xd0ab
            }
            0xd0ab => {
                w.clear_carry();
                0xd0ac
            }
            0xd0ac => {
                w.drain(8);
                let v = w.read(u32::from(0x2fu16), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xd0ae
            }
            0xd0ae => {
                w.drain(6);
                w.cpu.y = w.cpu.a & if index_width == 1 { 255 } else { 65535 };
                w.flags(w.cpu.y, index_width);
                0xd0af
            }
            0xd0af => {
                w.drain(8);
                let v = w.read(u32::from(0x22u16), width);
                w.load(v, width);
                0xd0b1
            }
            0xd0b1 => {
                w.clear_carry();
                0xd0b2
            }
            0xd0b2 => {
                w.drain(16);
                if (0xd01cu16 + w.cpu.y) & 0xff00 != 0xd000 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xd01cu16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xd0b5
            }
            0xd0b5 => {
                w.drain(8);
                let v = w.read(u32::from(0xecu16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xd0b7
            }
            0xd0b7 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xd0b8
            }
            0xd0b8 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xd0b9
            }
            0xd0b9 => {
                w.drain(6);
                let old = w.cpu.a & if width == 1 { 255 } else { 65535 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, width);
                0xd0ba
            }
            0xd0ba => {
                w.drain(8);
                let address = u32::from(0x2u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                self.x = w.cpu.a;
                0xd0bc
            }
            0xd0bc => {
                w.drain(8);
                let v = w.read(u32::from(0x20u16), width);
                w.load(v, width);
                0xd0be
            }
            0xd0be => {
                w.clear_carry();
                0xd0bf
            }
            0xd0bf => {
                w.drain(16);
                if (0xcfccu16 + w.cpu.y) & 0xff00 != 0xcf00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xcfccu16 + w.cpu.y), width);
                if width == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                0xd0c2
            }
            0xd0c2 => {
                w.drain(8);
                let v = w.read(u32::from(0xecu16), width);
                w.load((w.cpu.a & v) & if width == 1 { 255 } else { 65535 }, width);
                0xd0c4
            }
            0xd0c4 => {
                w.drain(8);
                let address = u32::from(0x0u16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                self.y = w.cpu.a;
                0xd0c6
            }
            0xd0c6 => {
                w.drain(8 * u32::from(width));
                let v = 1;
                w.load(v, width);
                0xd0c9
            }
            0xd0c9 => {
                w.drain(8);
                let address = u32::from(0xau16);
                w.write(
                    address,
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                0xd0cb
            }
            0xd0cb => {
                w.drain(12);
                let v = w.pull(index_width);
                w.cpu.y = v;
                w.flags(v, index_width);
                0xd0cc
            }
            0xd0cc => {
                w.width(16, true);
                0xd0ce
            }
            0xd0ce => {
                w.drain(6);
                w.load(w.cpu.y, width);
                0xd0cf
            }
            0xd0cf => {
                w.compare_immediate(1, width);
                0xd0d2
            }
            0xd0d2 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd0f4
                } else {
                    0xd0d4
                }
            }
            0xd0d4 => {
                w.compare_immediate(2, width);
                0xd0d7
            }
            0xd0d7 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd0f4
                } else {
                    0xd0d9
                }
            }
            0xd0d9 => {
                w.compare_immediate(3, width);
                0xd0dc
            }
            0xd0dc => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd0f4
                } else {
                    0xd0de
                }
            }
            0xd0de => {
                w.compare_immediate(6, width);
                0xd0e1
            }
            0xd0e1 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd0f4
                } else {
                    0xd0e3
                }
            }
            0xd0e3 => {
                w.compare_immediate(7, width);
                0xd0e6
            }
            0xd0e6 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd0f4
                } else {
                    0xd0e8
                }
            }
            0xd0e8 => {
                w.compare_immediate(8, width);
                0xd0eb
            }
            0xd0eb => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd0f4
                } else {
                    0xd0ed
                }
            }
            0xd0ed => {
                w.drain(6);
                w.push(w.cpu.y, index_width);
                0xd0ee
            }
            0xd0ee => {
                w.item_call();
                self.child_ready = true;
                0xd9d8
            }
            0xd0f1 => {
                w.drain(12);
                let v = w.pull(index_width);
                w.cpu.y = v;
                w.flags(v, index_width);
                0xd0f2
            }
            0xd0f2 => {
                w.branch(true);
                0xd0fe
            }
            0xd0fe => {
                w.width(48, false);
                0xd100
            }
            0xd100 => {
                w.drain(8 * u32::from(index_width));
                let old = w.cpu.y;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old >= 5);
                w.flags(
                    old.wrapping_sub(5) & if index_width == 1 { 255 } else { 65535 },
                    index_width,
                );
                0xd102
            }
            0xd102 => {
                let taken = w.branch(w.cpu.p & ZERO != 0);
                if taken {
                    0xd0fb
                } else {
                    0xd104
                }
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
            _ => panic!("main tile probe PC {pc:04x}"),
        };
        w.cpu.pc = next;
        self.pc = Some(next);
        if matches!(next, 0xd104 | 0xd0fb) {
            self.complete = true;
        }
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
    fn main_tile_probe_matches_source_geometry_and_classifier() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x07cfcc..0x07d06c {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        assert_eq!(table_byte(0x07cfcb), None);
        assert_eq!(table_byte(0x07d06c), None);
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
                r[0x1f1..0x1f3].copy_from_slice(&[0xd7, 0xb8]);
                r[0x2f] = (variant & 3) * 2;
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
                let pc = 0xd077;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: [0, 4, 5][(variant as usize) % 3],
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
                let mut work = MainTileProbe::default();
                let mut classifier: Option<TileClassifier> = None;
                while !work.complete() {
                    if work.child_ready() && classifier.is_none() {
                        classifier = Some(TileClassifier::default());
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
                                classifier.as_mut().unwrap().step(cpu, clock, bus).unwrap();
                            } else {
                                work.step(cpu, clock, bus).unwrap();
                            }
                        },
                    );
                    work = work.clone();
                    classifier = classifier.clone();
                    if work.child_ready() && classifier.is_some() {
                        match classifier.as_ref().unwrap().tail() {
                            Some(TileClassifierTail::Complete) => work.begin_response(),
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
                let tail = classifier.as_ref().unwrap().tail().unwrap();
                let handoff = work.handoff(tail);
                assert!(matches!(handoff.item, 0 | 4 | 5));
                if work.complete() {
                    assert_eq!(
                        (cpu.pc, cpu.sp, cpu.p & 0x30),
                        (if handoff.item == 5 { 0xd0fb } else { 0xd104 }, 0x1f0, 0x30)
                    );
                    assert_eq!(tail, TileClassifierTail::Complete);
                } else {
                    assert!(work.child_ready());
                }
            }
        }
        println!("main tile probe: 8192 cases, {instructions} source instructions, index widths, byte neighbors and classifier handoffs");
    }
}
