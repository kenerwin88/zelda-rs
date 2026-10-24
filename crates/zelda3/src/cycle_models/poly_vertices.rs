//! Retained vertex transform/projection worker. CPU registers, partial WRAM
//! stores and external math registers survive between individual instructions.
use super::{native_work::{Work, WorkBus}, nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;

#[cfg(any(test, feature = "rom-timing"))]
pub(crate) use super::native_cpu_math::NativeCpuMath as PolyDivider;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Call,
    Setup(u8),
    Vertex(u8),
    Transform(u8),
    ProjectCall,
    Project {
        axis: u8,
        step: u8,
    },
    ProjectReturn,
    Publish(u8),
    Return,
    Complete,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PolyVertices {
    phase: Phase,
}
impl PolyVertices {
    pub(super) fn defers_irq_mask_change(&self) -> bool {
        matches!(self.phase, Phase::Transform(1 | 16 | 19 | 42 | 45)
            | Phase::Project { step: 14 | 29, .. }) // SEI / CLI; REP applies immediately
    }
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    fn pc(&self) -> u16 {
        match self.phase {
            Phase::Call => 0xf82f,
            Phase::Setup(i) => [0xf8fb, 0xf8fd, 0xf8ff, 0xf900, 0xf901, 0xf903][usize::from(i)],
            Phase::Vertex(i) => [
                0xf904, 0xf905, 0xf906, 0xf908, 0xf90a, 0xf90b, 0xf90d, 0xf90f, 0xf910, 0xf912,
                0xf914, 0xf915, 0xf917,
            ][usize::from(i)],
            Phase::Transform(i) => [
                0xf931, 0xf933, 0xf934, 0xf937, 0xf939, 0xf93c, 0xf93e, 0xf941, 0xf944, 0xf946,
                0xf949, 0xf94b, 0xf94e, 0xf950, 0xf953, 0xf954, 0xf957, 0xf958, 0xf95a, 0xf95c,
                0xf95d, 0xf960, 0xf962, 0xf965, 0xf967, 0xf96a, 0xf96d, 0xf96f, 0xf972, 0xf974,
                0xf977, 0xf979, 0xf97c, 0xf97d, 0xf980, 0xf982, 0xf985, 0xf987, 0xf98a, 0xf98c,
                0xf98f, 0xf990, 0xf993, 0xf994, 0xf996, 0xf998, 0xf999, 0xf99c, 0xf99e, 0xf9a1,
                0xf9a3, 0xf9a6, 0xf9a9, 0xf9ab, 0xf9ae, 0xf9b0, 0xf9b3, 0xf9b5, 0xf9b8, 0xf9b9,
                0xf9bc, 0xf9be, 0xf9c1, 0xf9c3, 0xf9c6, 0xf9c8, 0xf9cb, 0xf9cc, 0xf9cf, 0xf9d1,
                0xf9d3, 0xf9d5,
            ][usize::from(i)],
            Phase::ProjectCall => 0xf91a,
            Phase::Project { axis, step } => {
                0xf9d6
                    + u16::from(axis) * 0x3c
                    + [
                        0, 2, 4, 7, 8, 10, 12, 14, 15, 18, 20, 22, 24, 25, 27, 28, 30, 33, 35, 38,
                        39, 40, 43, 45, 46, 48, 49, 52, 54, 57, 58,
                    ][usize::from(step)]
            }
            Phase::ProjectReturn => 0xfa4e,
            Phase::Publish(i) => [
                0xf91d, 0xf91f, 0xf920, 0xf922, 0xf924, 0xf926, 0xf927, 0xf929, 0xf92b, 0xf92d,
                0xf92e,
            ][usize::from(i)],
            Phase::Return => 0xf930,
            Phase::Complete => 0xf832,
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.db), (9, self.pc(), 0x1f00, 9));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.phase = match self.phase {
            Phase::Call => {
                w.drain(16);
                w.drain(6);
                w.push(w.cpu.pc + 2, 2);
                Phase::Setup(0)
            }
            Phase::Setup(i) => {
                match i {
                    0 => w.width(0x30, false),
                    1 => {
                        let v = w.dp_read(0x1f3f, 1);
                        w.load(v, 1);
                    }
                    2 => {
                        w.drain(6);
                        w.cpu.x = w.cpu.a & 255;
                        w.flags(w.cpu.x, 1);
                    }
                    3 => w.shift_left(1),
                    4 => {
                        let v = w.dp_read(0x1f3f, 1);
                        w.adc_byte(v);
                    }
                    5 => {
                        w.drain(6);
                        w.cpu.y = w.cpu.a & 255;
                        w.flags(w.cpu.y, 1);
                    }
                    _ => unreachable!(),
                }
                if i == 5 {
                    Phase::Vertex(0)
                } else {
                    Phase::Setup(i + 1)
                }
            }
            Phase::Vertex(i) => {
                match i {
                    0 => {
                        w.drain(6);
                        w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                        w.flags(w.cpu.x, 1);
                    }
                    1 | 4 | 7 => {
                        w.drain(6);
                        w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                        w.flags(w.cpu.y, 1);
                    }
                    2 | 5 | 8 => {
                        let ptr = w.dp_read(0x1f41, 2);
                        let addr = ptr.wrapping_add(w.cpu.y);
                        if ptr & 0xff00 != addr & 0xff00 {
                            w.drain(6);
                        }
                        let value = w.read(w.absolute(addr), 1);
                        w.load(value, 1);
                    }
                    3 | 6 | 9 => w.dp_store(0x1f48 - u16::from(i / 3), w.cpu.a & 255, 1),
                    10 => {
                        w.drain(6);
                        w.push(w.cpu.y, 1);
                    }
                    11 => w.width(0x20, true),
                    12 => {
                        w.drain(16);
                        w.drain(6);
                        w.push(w.cpu.pc + 2, 2);
                    }
                    _ => unreachable!(),
                }
                if i == 12 {
                    Phase::Transform(0)
                } else {
                    Phase::Vertex(i + 1)
                }
            }
            Phase::Transform(i) => {
                match i {
                    0 | 3 | 5 | 8 | 10 | 12 | 18 | 21 | 23 | 26 | 28 | 30 | 34 | 36 | 38 | 44
                    | 47 | 49 | 52 | 54 | 56 | 60 | 62 | 64 => {
                        let offset = match i {
                            0 => 0x56,
                            3 => 0x57,
                            5 | 23 | 49 => 0x45,
                            8 => 0x54,
                            10 => 0x55,
                            12 | 38 | 64 => 0x47,
                            18 => 0x58,
                            21 => 0x59,
                            26 => 0x52,
                            28 => 0x53,
                            30 | 56 => 0x46,
                            34 => 0x5a,
                            36 => 0x5b,
                            44 => 0x5c,
                            47 => 0x5d,
                            52 => 0x50,
                            54 => 0x51,
                            60 => 0x5e,
                            62 => 0x5f,
                            _ => unreachable!(),
                        };
                        let v = w.dp_read(0x1f00 + offset, 1);
                        w.cpu.y = v;
                        w.flags(v, 1);
                    }
                    1 | 19 | 45 => {
                        w.drain(6);
                        w.cpu.p |= 4;
                    }
                    2 | 4 | 6 | 9 | 11 | 13 | 20 | 22 | 24 | 27 | 29 | 31 | 35 | 37 | 39 | 46
                    | 48 | 50 | 53 | 55 | 57 | 61 | 63 | 65 => {
                        w.drain(16);
                        w.write_io(
                            if matches!(i, 6 | 13 | 24 | 31 | 39 | 50 | 57 | 65) {
                                0x211c
                            } else {
                                0x211b
                            },
                            w.cpu.y,
                            1,
                        );
                    }
                    7 | 25 | 51 => {
                        w.drain(16);
                        let v = w.read_io(if i == 51 { 0x2135 } else { 0x2134 }, 2);
                        w.load(v, 2);
                    }
                    14 | 58 => {
                        w.drain(6);
                        w.cpu.p |= 1;
                    }
                    15 | 59 => {
                        w.drain(16);
                        let v = w.read_io(if i == 59 { 0x2135 } else { 0x2134 }, 2);
                        w.sbc(v, 2);
                    }
                    16 | 42 => {
                        w.drain(6);
                        w.cpu.p &= !4;
                    }
                    17 | 43 | 70 => w.dp_store(
                        match i {
                            17 => 0x1f48,
                            43 => 0x1f4a,
                            _ => 0x1f4c,
                        },
                        w.cpu.a,
                        2,
                    ),
                    32 | 40 | 66 => w.clear_carry(),
                    33 | 41 | 67 => {
                        w.drain(16);
                        let v = w.read_io(if i == 67 { 0x2135 } else { 0x2134 }, 2);
                        w.adc(v);
                    }
                    68 => w.width(5, true),
                    69 => {
                        let v = w.dp_read(0x1f08, 2);
                        w.adc(v);
                    }
                    71 => {
                        w.rts();
                        assert_eq!(w.cpu.pc, 0xf91a);
                    }
                    _ => unreachable!(),
                }
                if i == 71 {
                    Phase::ProjectCall
                } else {
                    Phase::Transform(i + 1)
                }
            }
            Phase::ProjectCall => {
                w.drain(16);
                w.drain(6);
                w.push(w.cpu.pc + 2, 2);
                Phase::Project { axis: 0, step: 0 }
            }
            Phase::Project { axis, step } => {
                let at = 0x1f48 + u16::from(axis) * 2;
                let mut next = step + 1;
                match step {
                    0 | 5 | 15 => {
                        let v = w.dp_read(
                            match step {
                                0 => at,
                                5 => 0x1f4c,
                                _ => 0x1fb2,
                            },
                            2,
                        );
                        w.load(v, 2);
                    }
                    1 => {
                        if w.branch(w.cpu.p & 0x80 == 0) {
                            next = 4;
                        }
                    }
                    2 => {
                        w.drain(16);
                        w.load(!w.cpu.a, 2);
                    }
                    3 => w.increment_a(2),
                    4 | 6 => w.dp_store(if step == 4 { 0x1fb2 } else { 0x1fb0 }, w.cpu.a, 2),
                    7 => w.swap_bytes(),
                    8 => w.mask(255, 2),
                    9 => {
                        if w.branch(w.cpu.p & 2 != 0) {
                            next = 14;
                        }
                    }
                    10 | 11 => {
                        let address = if step == 10 { 0x1fb2 } else { 0x1fb0 };
                        let v = w.dp_read(address, 2);
                        w.drain(6);
                        w.cpu.p = (w.cpu.p & !1) | (v as u8 & 1);
                        w.flags(v >> 1, 2);
                        w.write(u32::from(address), v >> 1, 2);
                    }
                    12 => {
                        w.drain(6);
                        let v = w.cpu.a;
                        w.cpu.p = (w.cpu.p & !1) | (v as u8 & 1);
                        w.load(v >> 1, 2);
                    }
                    13 => {
                        if w.branch(w.cpu.p & 2 == 0) {
                            next = 10;
                        }
                    }
                    14 => {
                        w.drain(6);
                        w.cpu.p |= 4;
                    }
                    16 => {
                        w.drain(16);
                        w.write_io(0x4204, w.cpu.a, 2);
                    }
                    17 | 22 => {
                        let v = w.dp_read(if step == 17 { 0x1fb0 } else { at + 1 }, 1);
                        w.cpu.y = v;
                        w.flags(v, 1);
                    }
                    18 => {
                        w.drain(16);
                        w.write_io(0x4206, w.cpu.y, 1);
                    }
                    19 | 20 | 25 => w.drain(6),
                    21 => w.immediate(0, 2),
                    23 => {
                        w.drain(6);
                        w.cpu.p |= 1;
                    }
                    24 => {
                        if w.branch(w.cpu.p & 0x80 != 0) {
                            next = 28;
                        }
                    }
                    26 | 28 => {
                        w.drain(16);
                        let v = w.read_io(0x4214, 2);
                        if step == 26 {
                            w.load(v, 2)
                        } else {
                            w.sbc(v, 2)
                        }
                    }
                    27 => {
                        w.branch(true);
                        next = 29;
                    }
                    29 => {
                        w.drain(6);
                        w.cpu.p &= !4;
                    }
                    30 => w.dp_store(at, w.cpu.a, 2),
                    _ => unreachable!(),
                }
                if step == 30 {
                    if axis == 0 {
                        Phase::Project { axis: 1, step: 0 }
                    } else {
                        Phase::ProjectReturn
                    }
                } else {
                    Phase::Project { axis, step: next }
                }
            }
            Phase::ProjectReturn => {
                w.rts();
                assert_eq!(w.cpu.pc, 0xf91d);
                Phase::Publish(0)
            }
            Phase::Publish(i) => {
                match i {
                    0 => w.width(0x20, false),
                    1 => w.clear_carry(),
                    2 | 6 => {
                        let v = w.dp_read(if i == 2 { 0x1f06 } else { 0x1f07 }, 1);
                        w.load(v, 1);
                    }
                    3 | 7 => {
                        let v = w.dp_read(if i == 3 { 0x1f48 } else { 0x1f4a }, 1);
                        if i == 3 {
                            w.adc_byte(v)
                        } else {
                            w.sbc(v, 1)
                        }
                    }
                    4 | 8 => {
                        w.drain(8);
                        w.drain(6);
                        w.write(
                            u32::from(w.cpu.d + (if i == 4 { 0x60 } else { 0x88 }) + w.cpu.x),
                            w.cpu.a & 255,
                            1,
                        );
                    }
                    5 => {
                        w.drain(6);
                        w.cpu.p |= 1;
                    }
                    9 => {
                        w.drain(12);
                        w.cpu.y = w.pull(1);
                        w.flags(w.cpu.y, 1);
                    }
                    10 => {
                        let again = w.branch(w.cpu.p & 2 == 0);
                        self.phase = if again {
                            Phase::Vertex(0)
                        } else {
                            Phase::Return
                        };
                        w.cpu.pc = self.pc();
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Phase::Publish(i + 1)
            }
            Phase::Return => {
                w.rts();
                assert_eq!(w.cpu.pc, 0xf832);
                self.phase = Phase::Complete;
                return Some(w.cycles);
            }
            Phase::Complete => unreachable!(),
        };
        w.cpu.pc = self.pc();
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cycle_models::{poly_rotation::PolyRotation, poly_shape::PolyShape};
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };

    fn compare_fixture(
        model: u8,
        config: u8,
        angles: (u8, u8),
        projection: Option<(u16, u16, u16)>,
    ) {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut cpu = NativeNmiContext {
            a: 0xab00,
            x: 0x45,
            y: 0x67,
            d: 0x1f00,
            db: 9,
            bank: 9,
            pc: if projection.is_some() { 0xf91a } else { 0xf829 },
            sp: 0x1f3e,
            p: if projection.is_some() { 0x10 } else { 0x30 },
        };
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, &rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 8) ^ 0xa7) as u8;
        }
        machine.ram[0x1f02] = config;
        machine.ram[0x1f03] = model;
        machine.ram[0x1f04] = angles.0;
        machine.ram[0x1f05] = angles.1;
        machine.ram[0x1f06] = 96;
        machine.ram[0x1f07] = 64;
        if let Some((x, y, z)) = projection {
            for (at, value) in [(0x1f48, x), (0x1f4a, y), (0x1f4c, z)] {
                machine.ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
        }
        machine.cpu.a = cpu.a;
        machine.cpu.x = cpu.x;
        machine.cpu.y = cpu.y;
        machine.cpu.dp = cpu.d;
        machine.cpu.db = cpu.db;
        machine.cpu.k = cpu.bank;
        machine.cpu.pc = cpu.pc;
        machine.cpu.sp = cpu.sp;
        machine.cpu.e = false;
        machine.cpu.unpack_flags(cpu.p);
        let mut ram = machine.ram.clone();
        let mut ppu = machine.ppu.clone();
        let mut clock = CpuMasterTimeline::at_raster(
            0,
            CpuRasterPosition::new(224, 1300),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut source =
            RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                .unwrap();
        clock.begin_synchronous_timeline().unwrap();
        if projection.is_none() {
            let mut shape = PolyShape::default();
            while !shape.complete() {
                crate::cycle_models::native_work::compare_instruction(
                    "shape prefix",
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        shape.step(cpu, clock, bus).unwrap();
                    },
                );
            }
            let mut rotation = PolyRotation::default();
            while !rotation.complete() {
                crate::cycle_models::native_work::compare_instruction_with_ppu(
                    "rotation prefix",
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    Some(&mut ppu),
                    |cpu, clock, bus| {
                        rotation.step(cpu, clock, bus).unwrap();
                    },
                );
            }
        }
        let mut vertices = PolyVertices {
            phase: if projection.is_some() {
                Phase::ProjectCall
            } else {
                Phase::Call
            },
        };
        let mut divider = PolyDivider::default();
        let mut instructions = 0;
        while !vertices.complete() && !(projection.is_some() && vertices.phase == Phase::Publish(0))
        {
            assert_eq!(vertices.defers_irq_mask_change(),
                matches!(rom[9 * 0x8000 + (cpu.pc as usize & 0x7fff)], 0x58 | 0x78));
            let label = format!(
                "model={model} config={config} angles={angles:?} projection={projection:?} {:?}",
                vertices.phase
            );
            crate::cycle_models::native_work::compare_instruction_with_devices(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                &rom,
                Some(&mut ppu),
                |_, access| divider.access(access),
                |cpu, clock, bus| {
                    vertices.step(cpu, clock, bus).unwrap();
                },
            );
            assert_eq!(ram, source.snes().ram, "{label}");
            assert_eq!(ppu.m7_matrix, source.snes().ppu.m7_matrix, "{label}");
            assert_eq!(ppu.m7_prev, source.snes().ppu.m7_prev, "{label}");
            assert_eq!(
                (divider.dividend, divider.quotient, divider.remainder),
                (
                    source.snes().divide_a,
                    source.snes().divide_result,
                    source.snes().multiply_result
                ),
                "{label}"
            );
            vertices = vertices.clone();
            divider = divider.clone();
            instructions += 1;
            assert!(instructions < 5000, "{label}");
        }
        assert_eq!(cpu.sp, 0x1f3e);
    }

    #[test]
    fn vertices_match_each_bus_transaction_register_and_partial_publication() {
        for model in 0..2 {
            for (config, a, b) in [(0, 0, 0), (128, 32, 200), (255, 192, 64)] {
                compare_fixture(model, config, (a, b), None);
            }
        }
    }

    #[test]
    fn projection_retains_sign_scaling_and_zero_divisor_paths() {
        for values in [
            (0, 0, 0),
            (1, 65535, 1),
            (0x8000, 0x7fff, 0x100),
            (0x1234, 0xedcc, 0xffff),
        ] {
            compare_fixture(0, 0, (0, 0), Some(values));
        }
    }
}
