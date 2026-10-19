//! Module 7's caller-owned prefix through its indirect submodule call.
//! Layer effects run on the inherited clock and stack; the selected submodule
//! must claim the resulting context before any common caller suffix can run.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

const LAYERS: [u16; 8] = [
    0xfe87, 0xfe87, 0xfe88, 0xffde, 0xfeee, 0xff0d, 0xff5d, 0xffa4,
];
const SUBMODULES: [u16; 27] = [
    0x88de, 0x897c, 0x8a26, 0x8c05, 0x8c0a, 0x8c0f, 0x8c14, 0x8e27, 0x8f0c, 0x900f, 0x9014, 0x902d,
    0x904a, 0x904f, 0x9054, 0x931d, 0x8f88, 0x9357, 0x9357, 0x9357, 0x9520, 0x967a, 0x972a, 0x97c8,
    0x980a, 0x98f7, 0x9916,
];

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let (words, offset): (&[u16], usize) = match address {
        0x0afe70..=0x0afe7f => (&LAYERS, (address - 0x0afe70) as usize),
        0x02876c..=0x0287a1 => (&SUBMODULES, (address - 0x02876c) as usize),
        _ => return None,
    };
    Some(words[offset / 2].to_le_bytes()[offset & 1])
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonDispatch {
    pc: u32,
    entered_submodule: bool,
}
impl Default for DungeonDispatch {
    fn default() -> Self {
        Self {
            pc: 0x0287a2,
            entered_submodule: false,
        }
    }
}
impl DungeonDispatch {
    pub(crate) fn entered_submodule(&self) -> bool {
        self.entered_submodule
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.entered_submodule {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db),
            ((self.pc >> 16) as u8, self.pc as u16, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0x0287a2 | 0x0afed6 | 0x0afeeb | 0x0aff9d | 0x0affbc => {
                w.width(0x30, false);
                pc + 2
            }
            0x0afe92 | 0x0aff5d | 0x0affa7 => {
                w.width(0x30, true);
                pc + 2
            }
            0x0afeee | 0x0aff21 | 0x0aff48 => {
                w.width(0x20, true);
                pc + 2
            }
            0x0aff0a | 0x0aff3f | 0x0afff8 => {
                w.width(0x20, false);
                pc + 2
            }
            0x0affde => {
                w.width(0x21, true);
                pc + 2
            }
            0x0287a4 => {
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(0x87a7, 2);
                0x0afe80
            }
            0x0287a8 | 0x0afe80 | 0x0aff0d | 0x0aff79 => {
                let a = match pc {
                    0x0287a8 => 0x11,
                    0x0afe80 => 0xad,
                    0x0aff0d => 0x1a,
                    _ => 0,
                };
                let v = w.dp_read(a, m);
                w.load(v, m);
                pc + 2
            }
            0x0287aa | 0x0afe82 | 0x0afea0 | 0x0aff67 | 0x0affaf => {
                w.shift_left(m);
                pc + 1
            }
            0x0287ab | 0x0afe83 | 0x0afea4 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & if x == 1 { 255 } else { 65535 };
                w.flags(w.cpu.x, x);
                pc + 1
            }
            0x0287ac | 0x0afe84 => {
                if pc == 0x0287ac {
                    w.drain(8);
                    w.drain(6);
                    w.drain(8);
                } else {
                    w.operand_then_internal(2);
                }
                let a = if pc == 0x0287ac { 0x02876c } else { 0x0afe70 };
                let target = w.read(a + u32::from(w.cpu.x), 2);
                if pc == 0x0287ac {
                    w.push(0x87ae, 2);
                }
                self.entered_submodule = pc == 0x0287ac;
                (pc & 0xff0000) | u32::from(target)
            }
            0x0afe87 | 0x0afe91 | 0x0afed8 | 0x0afeed | 0x0aff0c | 0x0aff47 | 0x0affa3
            | 0x0affc9 | 0x0affd6 | 0x0affdd | 0x0afffa => {
                w.rtl();
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x0afe88 | 0x0afe9a | 0x0afea5 | 0x0afec7 | 0x0afedc | 0x0afef0 | 0x0afefa
            | 0x0affbe => {
                let a = match pc {
                    0x0afe88 => 0x403,
                    0x0afe9a => 0x41a,
                    0x0afea5 => 0x41c,
                    0x0afec7 | 0x0afef0 => 0x422,
                    0x0afedc | 0x0afefa => 0x424,
                    _ => 0x4c5,
                };
                w.drain(16);
                let v = w.read(a, m);
                w.load(v, m);
                pc + 3
            }
            0x0afe8b | 0x0afea1 | 0x0afeb0 | 0x0aff0f | 0x0affea => {
                w.mask(
                    match pc {
                        0x0afe8b => 0x80,
                        0x0afea1 => 2,
                        0x0aff0f => 0x7f,
                        _ => 1,
                    },
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0x0afe8d | 0x0afe9e | 0x0afeb6 | 0x0afec2 | 0x0aff13 | 0x0aff17 | 0x0aff1b
            | 0x0aff1f | 0x0aff5b | 0x0aff68 | 0x0aff71 | 0x0aff7b | 0x0aff88 | 0x0affb0
            | 0x0affba | 0x0affc1 | 0x0affcc => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let (take, to) = match pc {
                    0x0afe8d => (z, 0xfe92),
                    0x0afe9e => (c, 0xfeeb),
                    0x0afeb6 => (!z, 0xfebc),
                    0x0afec2 => (c, 0xfed9),
                    0x0aff13 | 0x0aff1b => (z, 0xff48),
                    0x0aff17 => (z, 0xff21),
                    0x0aff1f => (!z, 0xff43),
                    0x0aff5b => (true, 0xff37),
                    0x0aff68 => (!c, 0xff6c),
                    0x0aff71 => (!z, 0xff64),
                    0x0aff7b => (!z, 0xff83),
                    0x0aff88 => (z, 0xff9d),
                    0x0affb0 => (!c, 0xffb5),
                    0x0affba => (!z, 0xffac),
                    0x0affc1 => (!z, 0xffca),
                    _ => (!z, 0xffd7),
                };
                if w.branch(take) {
                    0x0a0000 | to
                } else {
                    pc + 2
                }
            }
            0x0afe8f | 0x0affc3 | 0x0affd7 => {
                w.dp_store(if pc == 0x0afe8f { 0xad } else { 0x1d }, 0, m);
                pc + 2
            }
            0x0afe94 | 0x0afe97 | 0x0aff04 | 0x0aff07 | 0x0affa4 => {
                let a = match pc {
                    0x0afe94 | 0x0aff04 => 0x312,
                    0x0afe97 | 0x0aff07 => 0x310,
                    _ => 0x4c5,
                };
                w.drain(16);
                w.write(a, 0, m);
                pc + 3
            }
            0x0afe9d => {
                w.drain(6);
                let v = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, m);
                pc + 1
            }
            0x0afea8 | 0x0afed1 | 0x0afee6 | 0x0afef3 | 0x0afefd => {
                w.clear_carry();
                pc + 1
            }
            0x0afeca | 0x0afedf | 0x0afff2 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0x0afea9 => {
                w.drain(16);
                w.adc(0x8000);
                pc + 3
            }
            0x0afeac | 0x0afec4 | 0x0afece | 0x0afed9 | 0x0afee3 | 0x0afef7 | 0x0aff01
            | 0x0affe6 | 0x0afff5 => {
                let a = match pc {
                    0x0afeac | 0x0affe6 => 0x41c,
                    0x0afec4 | 0x0afff5 => 0x312,
                    0x0afece | 0x0afef7 => 0x422,
                    0x0afed9 => 0x310,
                    _ => 0x424,
                };
                w.drain(16);
                w.write(a, w.cpu.a & mask, m);
                pc + 3
            }
            0x0afeaf | 0x0affe9 => {
                w.drain(6);
                let v = w.cpu.a;
                let c = u16::from(w.cpu.p & CARRY != 0);
                w.cpu.p = (w.cpu.p & !CARRY) | ((v >> 15) as u8);
                w.load((v << 1) | c, 2);
                pc + 1
            }
            0x0afeb3 | 0x0afebf | 0x0aff6e | 0x0affb7 => {
                w.compare_x_immediate(
                    match pc {
                        0x0afeb3 => 2,
                        0x0afebf => 4,
                        0x0aff6e => 32,
                        _ => 6,
                    },
                    x,
                );
                pc + 1 + u32::from(x)
            }
            0x0afeb8 => {
                w.drain(16);
                w.load(w.cpu.a ^ 0xffff, 2);
                pc + 3
            }
            0x0afebb => {
                w.increment_a(m);
                pc + 1
            }
            0x0afebc => {
                w.drain(16);
                let v = w.read(0x41a, x);
                w.cpu.x = v;
                w.flags(v, x);
                pc + 3
            }
            0x0afecb | 0x0afee0 => {
                w.drain(16);
                let v = w.read(if pc == 0x0afecb { 0x312 } else { 0x310 }, m);
                w.sbc(v, m);
                pc + 3
            }
            0x0afed2 | 0x0afee7 => {
                let v = w.dp_read(if pc == 0x0afed2 { 0xe2 } else { 0xe8 }, m);
                w.adc(v);
                pc + 2
            }
            0x0afed4 | 0x0afee9 | 0x0aff45 | 0x0affa1 | 0x0affc7 | 0x0affd0 | 0x0affd4
            | 0x0affdb | 0x0affed => {
                let a = match pc {
                    0x0afed4 => 0xe0,
                    0x0afee9 => 0xe6,
                    0x0affc7 | 0x0affd4 | 0x0affdb => 0x9a,
                    0x0affed => 0,
                    _ => 0x1d,
                };
                w.dp_store(a, w.cpu.a & mask, m);
                pc + 2
            }
            0x0afef4 | 0x0afefe | 0x0affe3 => {
                w.drain(16);
                let v = w.read(
                    match pc {
                        0x0afef4 => 0x312,
                        0x0afefe => 0x310,
                        _ => 0x41c,
                    },
                    m,
                );
                w.adc(v);
                pc + 3
            }
            0x0aff11 | 0x0aff15 | 0x0aff19 | 0x0aff1d | 0x0affca => {
                w.compare_immediate(
                    match pc {
                        0x0aff11 => 3,
                        0x0aff15 => 5,
                        0x0aff19 => 36,
                        0x0aff1d => 38,
                        _ => 1,
                    },
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0x0aff23 | 0x0aff2b | 0x0aff33 => {
                w.drain(24);
                let v = w.read(
                    match pc {
                        0x0aff23 => 0x7ec3da,
                        0x0aff2b => 0x7ec3dc,
                        _ => 0x7ec3de,
                    },
                    m,
                );
                w.load(v, m);
                pc + 4
            }
            0x0aff27 | 0x0aff2f | 0x0aff37 | 0x0aff3b | 0x0aff4d | 0x0aff54 | 0x0aff8a
            | 0x0aff8e | 0x0aff93 | 0x0aff97 => {
                let a = match pc {
                    0x0aff27 | 0x0aff4d => 0x7ec5da,
                    0x0aff2f | 0x0aff54 => 0x7ec5dc,
                    0x0aff37 => 0x7ec5de,
                    0x0aff3b => 0x7ec5ee,
                    0x0aff8a => 0x7ec3f6,
                    0x0aff8e => 0x7ec5f6,
                    0x0aff93 => 0x7ec3f8,
                    _ => 0x7ec5f8,
                };
                w.drain(24);
                w.write(a, w.cpu.a & mask, m);
                pc + 4
            }
            0x0aff41 | 0x0aff6a | 0x0aff9b => {
                w.dp_increment(if pc == 0x0aff6a { 0 } else { 0x15 }, m, true);
                pc + 2
            }
            0x0aff43 | 0x0aff4a | 0x0aff51 | 0x0aff58 | 0x0aff9f | 0x0affc5 | 0x0affce
            | 0x0affd2 | 0x0affd9 | 0x0affe0 | 0x0affef => {
                let v = match pc {
                    0x0aff4a => 0x1d59,
                    0x0aff51 => 0x25ff,
                    0x0aff58 => 26,
                    0x0affc5 => 0xb3,
                    0x0affd2 | 0x0affd9 => 0x70,
                    0x0affe0 => 0x8000,
                    0x0affef => 0,
                    _ => 2,
                };
                w.immediate(v, m);
                pc + 1 + u32::from(m)
            }
            0x0aff5f | 0x0aff73 | 0x0aff76 | 0x0aff7d | 0x0aff80 | 0x0affa9 => {
                let v = match pc {
                    0x0aff73 => 0x2940,
                    0x0aff76 => 0x4e60,
                    _ => 0,
                };
                w.drain(8 * u32::from(x));
                if matches!(pc, 0x0aff76 | 0x0aff80) {
                    w.cpu.y = v
                } else {
                    w.cpu.x = v
                };
                w.flags(v, x);
                pc + 1 + u32::from(x)
            }
            0x0aff62 => {
                w.dp_store(0, w.cpu.x, x);
                pc + 2
            }
            0x0aff64 | 0x0affac => {
                w.operand_then_internal(2);
                let v = w.read(0x540 + u32::from(w.cpu.x), m);
                w.load(v, m);
                pc + 3
            }
            0x0aff6c | 0x0aff6d | 0x0affb5 | 0x0affb6 => {
                w.increment_x();
                pc + 1
            }
            0x0aff83 | 0x0aff92 => {
                w.drain(6);
                w.load(if pc == 0x0aff83 { w.cpu.x } else { w.cpu.y }, m);
                pc + 1
            }
            0x0aff84 => {
                w.drain(24);
                let v = w.read(0x7ec3f6, m);
                let a = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & mask, m);
                pc + 4
            }
            0x0affb2 => {
                w.drain(16);
                let v = w.read(0x4c5, m).wrapping_add(1) & mask;
                w.drain(6);
                w.flags(v, m);
                w.write(0x4c5, v, m);
                pc + 3
            }
            0x0afff3 => {
                let v = w.dp_read(0, m);
                w.sbc(v, m);
                pc + 2
            }
            _ => unreachable!("dungeon dispatcher continuation {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_dispatch_all_layer_effects_match_source_instructions() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for range in [0x0afe70..0x0afe80, 0x02876c..0x0287a2] {
            for a in range {
                assert_eq!(
                    table_byte(a),
                    Some(rom[(a as usize >> 16) * 0x8000 + (a as usize & 0x7fff)])
                );
            }
        }
        assert_eq!(table_byte(0x0287ae), None);
        let mut instructions = 0;
        for field in [0, 1, 30_002, 30_003] {
            for layer in 0..8 {
                for variant in 0..16 {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    for (i, v) in machine.ram.iter_mut().enumerate() {
                        *v = (i ^ (i >> 8) ^ variant) as u8;
                    }
                    let submodule = (layer * 16 + variant) % SUBMODULES.len();
                    machine.ram[0xad] = layer as u8;
                    machine.ram[0x11] = submodule as u8;
                    machine.ram[0x1a] = [0, 3, 5, 36, 38, 127, 131, 133][variant % 8];
                    machine.ram[0x403] = if variant & 8 != 0 { 0x80 } else { 0 };
                    machine.ram[0x41a..0x41c]
                        .copy_from_slice(&((variant % 8) as u16).to_le_bytes());
                    machine.ram[0x41c..0x41e].copy_from_slice(
                        &(if variant & 1 == 0 { 0x7fffu16 } else { 0xffff }).to_le_bytes(),
                    );
                    machine.ram[0x15..0x17].copy_from_slice(&0xffffu16.to_le_bytes());
                    for slot in 0..16 {
                        let v = if slot < variant { 0x8000u16 } else { 0x7fff };
                        machine.ram[0x540 + slot * 2..0x542 + slot * 2]
                            .copy_from_slice(&v.to_le_bytes());
                    }
                    // Exercise both matching and changed invisible-floor palettes.
                    let color = if variant % 4 == 0 {
                        if variant == 0 {
                            0u16
                        } else {
                            0x2940
                        }
                    } else {
                        0xffff
                    };
                    machine.ram[0xc3f6..0xc3f8].copy_from_slice(&color.to_le_bytes());
                    let mut cpu = NativeNmiContext {
                        a: 0xb702,
                        x: 224,
                        y: 7,
                        d: 0,
                        db: 0,
                        bank: 2,
                        pc: 0x87a2,
                        sp: 0x1fc,
                        p: 0x30 | ((variant as u8) & 0xc3),
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let timeline = snes::CpuMasterTimeline::at_raster(
                        field,
                        snes::CpuRasterPosition::new(
                            224,
                            if variant & 1 == 0 { 1280 } else { 500 },
                        ),
                        snes::CpuBusWorkload::default(),
                        snes::CpuFieldTiming::NON_INTERLACE_EVEN,
                    );
                    let mut source = snes::RomCpuTimingProbe::new(
                        machine,
                        timeline,
                        snes::SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    let mut clock = source.timeline().clone();
                    let mut work = DungeonDispatch::default();
                    for _ in 0..1000 {
                        if work.entered_submodule() {
                            break;
                        }
                        let label = format!(
                            "field={field} layer={layer} variant={variant} pc={:06x}",
                            work.pc
                        );
                        super::super::native_work::compare_instruction(
                            &label,
                            &mut cpu,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            |cpu, clock, bus| {
                                work.step(cpu, clock, bus).unwrap();
                            },
                        );
                        instructions += 1;
                    }
                    assert!(work.entered_submodule());
                    assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp),
                        (2, SUBMODULES[submodule], 0x1fa)
                    );
                    assert_eq!(&ram[0x1fb..0x1fd], &0x87aeu16.to_le_bytes());
                    assert_eq!(ram, source.snes().ram, "layer={layer} variant={variant}");
                }
            }
        }
        eprintln!("512 dungeon entry fixtures, {instructions} instruction boundaries");
    }
}
