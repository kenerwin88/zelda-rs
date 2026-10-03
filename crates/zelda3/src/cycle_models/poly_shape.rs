//! Retained model selection and descriptor copy for the poly worker.
//! Each step owns one source instruction, including one MVN byte, so an
//! interrupt can retain the live registers and partially published descriptor.
use super::{intro_clear::Access, native_work::Work, nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PolyShape {
    phase: u8,
}

impl PolyShape {
    pub(crate) fn complete(&self) -> bool {
        self.phase == 19
    }

    fn pc(&self) -> u16 {
        [
            0xf829, 0xf83d, 0xf83f, 0xf841, 0xf844, 0xf845, 0xf848, 0xf84a, 0xf84d, 0xf850, 0xf851,
            0xf853, 0xf854, 0xf856, 0xf859, 0xf85a, 0xf85d, 0xf860, 0xf863, 0xf82c,
        ][usize::from(self.phase)]
    }

    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        if self.complete() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc), (9, self.pc()));
        assert_eq!(cpu.d, 0x1f00);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        match self.phase {
            0 => {
                w.drain(16);
                w.drain(6);
                w.push(w.cpu.pc + 2, 2);
            }
            1 => w.width(0x30, true),
            2 => {
                let value = w.dp_read(w.cpu.d + 2, 2);
                w.load(value, 2);
            }
            3 | 8 => w.mask(255, 2),
            4 | 9 | 11 => w.shift_left(2),
            5 | 13 => {
                w.drain(16);
                w.adc(if self.phase == 5 { 0x80 } else { 0xff8c });
            }
            6 | 10 => w.dp_store(w.cpu.d + if self.phase == 6 { 8 } else { 0xb0 }, w.cpu.a, 2),
            7 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1f03), 2);
                w.load(value, 2);
            }
            12 => {
                let value = w.dp_read(w.cpu.d + 0xb0, 2);
                w.adc(value);
            }
            14 => w.tax_word(),
            15 => {
                w.drain(16);
                w.cpu.y = 0x1f3f;
                w.flags(w.cpu.y, 2);
            }
            16 => w.immediate(5, 2),
            17 => {
                w.drain(8);
                let bank = w.read(0x09f862, 1);
                assert_eq!(bank, 9);
                let value = w.read(0x090000 | u32::from(w.cpu.x), 1);
                w.write(0x090000 | u32::from(w.cpu.y), value, 1);
                w.drain(12);
                w.cpu.db = 9;
                w.cpu.x = w.cpu.x.wrapping_add(1);
                w.cpu.y = w.cpu.y.wrapping_add(1);
                w.cpu.a = w.cpu.a.wrapping_sub(1);
                if w.cpu.a != 0xffff {
                    return Some(w.cycles);
                }
            }
            18 => {
                w.rts();
                assert_eq!(w.cpu.pc, 0xf82c);
                self.phase = 19;
                return Some(w.cycles);
            }
            _ => unreachable!(),
        }
        self.phase += 1;
        w.cpu.pc = self.pc();
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
    fn shape_descriptor_matches_each_instruction_and_partial_copy() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for model in 0..2 {
            for (config, status, v, h) in [
                (0, 0x30, 235, 358),
                (255, 0x31, 224, 1300),
                (128, 0x74, 100, 510),
            ] {
                let mut cpu = NativeNmiContext {
                    a: 0xab00,
                    x: 0x45,
                    y: 0x67,
                    d: 0x1f00,
                    db: 9,
                    bank: 9,
                    pc: 0xf829,
                    sp: 0x1f3e,
                    p: status,
                };
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 8) ^ 0xa5) as u8;
                }
                machine.ram[0x1f02] = config;
                machine.ram[0x1f03] = model;
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
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::NON_INTERLACE_EVEN,
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut shape = PolyShape::default();
                while !shape.complete() {
                    let label = format!(
                        "model={model} config={config} phase={} x={:04x}",
                        shape.phase, cpu.x
                    );
                    crate::cycle_models::native_work::compare_instruction(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        |cpu, clock, bus| {
                            shape.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(ram, source.snes().ram, "{label}");
                    shape = shape.clone();
                }
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.d),
                    (9, 0xf82c, 0x1f3e, 9, 0x1f00)
                );
                let at = 9 * 0x8000 + 0x7f8c + usize::from(model) * 6;
                assert_eq!(&ram[0x1f3f..0x1f45], &rom[at..at + 6]);
                assert_eq!(
                    u16::from_le_bytes([ram[0x1f08], ram[0x1f09]]),
                    u16::from(config) * 2 + 0x80
                );
            }
        }
    }
}
