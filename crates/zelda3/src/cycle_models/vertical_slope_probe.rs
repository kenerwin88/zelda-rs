//! Vertical slope reset, coordinate work, both tile calls and the probe return.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY},
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    [
        7, 0, 24, 0, 255, 255, 16, 0, 0, 0, 0, 0, 8, 0, 8, 0, 15, 0, 15, 0, 23, 0, 23, 0,
    ]
    .get(address.checked_sub(0x07cda3)? as usize)
    .copied()
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VerticalSlopeProbe {
    pc: Option<u16>,
    child_ready: bool,
    corner: u8,
    complete: bool,
}
impl VerticalSlopeProbe {
    pub(crate) fn child_ready(&self) -> bool {
        self.child_ready
    }
    pub(crate) fn corner(&self) -> u8 {
        self.corner
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn begin_high_corner(&mut self) {
        assert!(self.child_ready && self.corner == 0);
        self.pc = Some(0xceba);
        self.child_ready = false;
        self.corner = 1;
    }
    pub(crate) fn begin_return(&mut self) {
        assert!(self.child_ready && self.corner == 1);
        self.pc = Some(0xcec6);
        self.child_ready = false;
        self.corner = 2;
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.child_ready || self.complete {
            return None;
        }
        let pc = self.pc.unwrap_or(0xce85);
        let mode = if matches!(pc, 0xce85 | 0xcec8) {
            0x30
        } else if (0xceb2..=0xcec6).contains(&pc) {
            0
        } else {
            0x10
        };
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38), (7, pc, 0, mode));
        if self.corner == 0 {
            assert!(matches!(cpu.y, 0 | 2));
        }
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match pc {
            0xce85 | 0xceb0 => {
                w.width(if pc == 0xce85 { 0x20 } else { 0x10 }, true);
                pc + 2
            }
            0xcec6 => {
                w.width(0x30, false);
                0xcec8
            }
            0xcec8 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            0xceba => {
                let value = w.dp_read(4, 2);
                w.load(value, 2);
                0xcebc
            }
            0xcebc | 0xcec1 => {
                w.dp_store(if pc == 0xcebc { 2 } else { 0xa }, w.cpu.a, 2);
                pc + 2
            }
            0xcebe => {
                w.drain(16);
                w.load(2, 2);
                0xcec1
            }
            0xce87 | 0xceb7 | 0xcec3 => {
                w.item_call();
                if pc == 0xce87 {
                    0xd798
                } else {
                    self.child_ready = true;
                    0xd9d8
                }
            }
            0xce8a => {
                w.dp_store(0x59, 0, 2);
                0xce8c
            }
            0xce8c | 0xce96 | 0xcea3 => {
                let value = w.dp_read(if pc == 0xce8c { 0x20 } else { 0x22 }, 2);
                w.load(value, 2);
                pc + 2
            }
            0xce8e | 0xce98 | 0xcea5 => {
                w.clear_carry();
                pc + 1
            }
            0xce8f | 0xce99 | 0xcea6 => {
                w.drain(16);
                let value = w.read(
                    w.absolute(
                        (if pc == 0xce8f {
                            0xcda3
                        } else if pc == 0xce99 {
                            0xcdab
                        } else {
                            0xcdb3
                        }) + w.cpu.y,
                    ),
                    2,
                );
                w.adc(value);
                pc + 3
            }
            0xce92 | 0xce9c | 0xcea9 => {
                let value = w.dp_read(0xec, 2);
                w.load(w.cpu.a & value, 2);
                pc + 2
            }
            0xce94 | 0xcea1 | 0xceae | 0xceb5 => {
                w.dp_store(
                    match pc {
                        0xce94 => 0,
                        0xcea1 => 2,
                        0xceae => 4,
                        _ => 0xa,
                    },
                    w.cpu.a,
                    2,
                );
                pc + 2
            }
            0xce9e..=0xcea0 | 0xceab..=0xcead => {
                w.drain(6);
                let value = w.cpu.a;
                w.cpu.p = (w.cpu.p & !CARRY) | (value & 1) as u8;
                w.load(value >> 1, 2);
                pc + 1
            }
            0xceb2 => {
                w.drain(16);
                w.load(1, 2);
                0xceb5
            }
            0xd798 | 0xd79a | 0xd79c | 0xd79e | 0xd7a3 | 0xd7a5 => {
                let address = match pc {
                    0xd798 => 0xc,
                    0xd79a => 0xe,
                    0xd79c => 0x38,
                    0xd79e => 0x58,
                    0xd7a3 => 0x5f,
                    0xd7a5 => 0x62,
                    _ => unreachable!(),
                };
                w.dp_store(address, 0, 2);
                pc + 2
            }
            0xd7a0 | 0xd7a7 | 0xd7aa | 0xd7ad | 0xd7b0 | 0xd7b3 | 0xd7b6 | 0xd7b9 | 0xd7bc
            | 0xd7bf | 0xd7c2 | 0xd7c5 | 0xd7c8 | 0xd7cb | 0xd7ce | 0xd7d1 | 0xd7d4 => {
                let address = match pc {
                    0xd7a0 => 0x2c0,
                    0xd7a7 => 0x320,
                    0xd7aa => 0x341,
                    0xd7ad => 0x343,
                    0xd7b0 => 0x348,
                    0xd7b3 => 0x34c,
                    0xd7b6 => 0x357,
                    0xd7b9 => 0x359,
                    0xd7bc => 0x35b,
                    0xd7bf => 0x366,
                    0xd7c2 => 0x36d,
                    0xd7c5 => 0x36f,
                    0xd7c8 => 0x2e5,
                    0xd7cb => 0x2e7,
                    0xd7ce => 0x2ee,
                    0xd7d1 => 0x2f6,
                    0xd7d4 => 0x3f1,
                    _ => unreachable!(),
                };
                w.drain(16);
                w.write(w.absolute(address), 0, 2);
                pc + 3
            }
            0xd7d7 => {
                w.rts();
                assert_eq!(w.cpu.pc, 0xce8a);
                w.cpu.pc
            }
            _ => panic!("vertical slope probe PC {pc:04x}"),
        };
        self.pc = Some(next);
        w.cpu.pc = next;
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
    fn vertical_slope_probe_matches_source_masks_coordinates_and_reset() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x07cda3..0x07cdbb {
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
                let x = u16::from(timer) | ((variant as u16 & 7) << 13);
                let y = x.rotate_left(variant as u32);
                r[0x20..0x22].copy_from_slice(&y.to_le_bytes());
                r[0x22..0x24].copy_from_slice(&x.to_le_bytes());
                let mask = [0xffffu16, 0x1ff, 0x1f8, 0][(variant as usize >> 1) & 3];
                r[0xec..0xee].copy_from_slice(&mask.to_le_bytes());
                let pc = 0xce85;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: u16::from(variant & 1) * 2,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: if variant & 2 == 0 { 0x1f0 } else { 0x1ee },
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
                let mut work = VerticalSlopeProbe::default();
                while !work.child_ready() {
                    let label = format!("slope timer={timer} variant={variant} pc={:04x}", cpu.pc);
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
                                let low = table_byte(a)?;
                                Some(if width == 1 {
                                    u16::from(low)
                                } else {
                                    u16::from_le_bytes([low, table_byte(a + 1)?])
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
                assert_eq!(
                    (cpu.pc, cpu.sp),
                    (0xd9d8, if variant & 2 == 0 { 0x1ee } else { 0x1ec })
                );
            }
        }

        println!("vertical slope probe: 8192 cases, {instructions} source instructions, both directions and caller stack depths");
    }
    #[test]
    fn vertical_slope_probe_return_matches_source_high_corner_and_widths() {
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
                let high_x = u16::from(variant) * 5;
                let y = u16::from(variant) * 16;
                r[0..2].copy_from_slice(&y.to_le_bytes());
                r[2..4].copy_from_slice(&0xabcdu16.to_le_bytes());
                r[4..6].copy_from_slice(&high_x.to_le_bytes());
                r[0x1b] = u8::from(variant != 31);
                r[0xee] = variant & 1;
                r[0x37f] = u8::from(variant == 30);
                let offset =
                    ((y & !7) << 3) + (high_x & 63) + if variant & 1 != 0 { 0x1000 } else { 0 };
                r[0x12000 + usize::from(offset)] = timer;
                r[0x12000 + usize::from(offset) + 64] = if variant & 2 == 0 { timer } else { 0 };
                r[0x2f5] = u8::from(variant & 4 != 0);
                r[0xffc..0xffe].copy_from_slice(&u16::from(variant & 8 != 0).to_le_bytes());
                r[0x403..0x405]
                    .copy_from_slice(&if variant & 16 != 0 { 0x80u16 } else { 0 }.to_le_bytes());
                let sp = if variant & 2 == 0 { 0x1f0u16 } else { 0x1ee };
                r[usize::from(sp) + 1..usize::from(sp) + 3].copy_from_slice(&[0x81, 0xb6]);
                let mut cpu = NativeNmiContext {
                    a: 0xabcd,
                    x: 0x7654,
                    y: u16::from(variant) * 257,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc: 0xceba,
                    sp,
                    p: timer & 0xc3,
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
                let mut work = VerticalSlopeProbe {
                    pc: Some(0xceba),
                    corner: 1,
                    ..Default::default()
                };
                let mut classifier: Option<super::super::tile_classifier::TileClassifier> = None;
                while !work.complete() {
                    use super::super::tile_classifier::{TileClassifier, TileClassifierTail};
                    if work.child_ready() && classifier.is_none() {
                        classifier = Some(TileClassifier::default());
                    }
                    if classifier
                        .as_ref()
                        .is_some_and(|w| w.tail() == Some(TileClassifierTail::Complete))
                        && work.corner() == 1
                    {
                        work.begin_return();
                    }
                    if classifier.as_ref().is_some_and(|w| {
                        matches!(
                            w.tail(),
                            Some(
                                TileClassifierTail::Effect { .. }
                                    | TileClassifierTail::OutdoorAttribute
                            )
                        )
                    }) {
                        break;
                    }
                    let label = format!(
                        "high-corner tile={timer} variant={variant} pc={:04x}",
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
                                let byte = |a| {
                                    table_byte(a)
                                        .or_else(|| super::super::tile_classifier::table_byte(a))
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
                            if let Some(classifier) =
                                classifier.as_mut().filter(|w| w.tail().is_none())
                            {
                                classifier.step(cpu, clock, bus).unwrap();
                            } else {
                                work.step(cpu, clock, bus).unwrap();
                            }
                        },
                    );
                    work = work.clone();
                    classifier = classifier.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                if work.complete() {
                    assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp, cpu.p & 0x30),
                        (7, 0xb682, sp + 2, 0x30)
                    );
                } else {
                    assert!(classifier.as_ref().unwrap().tail().is_some());
                }
            }
        }

        println!("vertical slope return: 8192 cases, {instructions} source instructions, all byte attributes, high-corner setup, both stack depths and byte-width return");
    }
}
