//! One retained poly thread. IRQ exchanges the live CPU stack without replacing
//! this progress; the next NMI resumes the same partially drawn polygon.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
    poly_buffer_clear::PolyBufferClear,
    poly_faces::PolyFaces,
    poly_interrupt::PolyWait,
    poly_raster::PolyRaster,
    poly_rotation::PolyRotation,
    poly_shape::PolyShape,
    poly_vertices::PolyVertices,
};
use snes::CpuMasterTimeline;

pub(crate) struct WorkerInstruction {
    pub(crate) master_cycles: u64,
    pub(crate) irq_masked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Phase {
    Wait(PolyWait),
    Clear(PolyBufferClear),
    Shape(PolyShape),
    Rotation(PolyRotation),
    Vertices(PolyVertices),
    Faces(PolyFaces),
    Raster {
        faces: PolyFaces,
        raster: PolyRaster,
    },
    Finish(u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PolyWorker {
    phase: Phase,
}
impl Default for PolyWorker {
    fn default() -> Self {
        Self {
            phase: Phase::Wait(PolyWait::default()),
        }
    }
}
impl PolyWorker {
    #[cfg(test)]
    #[cfg(feature = "rom-timing")]
    pub(crate) fn from_pending_faces(faces: PolyFaces) -> Self {
        assert!(faces.raster_pending());
        Self {
            phase: Phase::Faces(faces),
        }
    }

    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> WorkerInstruction {
        // Crossing a callee boundary costs no extra instruction or time. Only
        // the instruction implemented by the selected owner advances the CPU.
        loop {
            let transition = match &self.phase {
                Phase::Wait(work) if work.ready() => Some(Phase::Clear(PolyBufferClear::default())),
                Phase::Clear(work) if work.complete() => Some(Phase::Shape(PolyShape::default())),
                Phase::Shape(work) if work.complete() => {
                    Some(Phase::Rotation(PolyRotation::default()))
                }
                Phase::Rotation(work) if work.complete() => {
                    Some(Phase::Vertices(PolyVertices::default()))
                }
                Phase::Vertices(work) if work.complete() => {
                    Some(Phase::Faces(PolyFaces::default()))
                }
                Phase::Faces(work) if work.complete() => Some(Phase::Finish(0)),
                Phase::Faces(work) if work.raster_pending() => Some(Phase::Raster {
                    faces: work.clone(),
                    raster: PolyRaster::default(),
                }),
                Phase::Raster { faces, raster } if raster.complete() => {
                    let mut faces = faces.clone();
                    faces.resume_after_raster(cpu);
                    Some(Phase::Faces(faces))
                }
                _ => None,
            };
            if let Some(next) = transition {
                self.phase = next;
            } else {
                break;
            }
        }
        // S9xMainLoop selects an IRQ before publishing a CLI/SEI mask change.
        // The live P register already has the new bit (and is what IRQ pushes),
        // but this boundary must still use the instruction's previous I bit.
        let deferred_mask = match &self.phase {
            Phase::Rotation(work) => work.defers_irq_mask_change(),
            Phase::Vertices(work) => work.defers_irq_mask_change(),
            Phase::Faces(work) => work.defers_irq_mask_change(),
            Phase::Raster { raster, .. } => raster.defers_irq_mask_change(),
            _ => false,
        }.then_some(cpu.p & 4 != 0);
        let access = bus;
        let master_cycles = match &mut self.phase {
            Phase::Wait(work) => work.step(cpu, clock, access),
            Phase::Clear(work) => work.step(cpu, clock, access).unwrap(),
            Phase::Shape(work) => work.step(cpu, clock, access).unwrap(),
            Phase::Rotation(work) => work.step(cpu, clock, access).unwrap(),
            Phase::Vertices(work) => work.step(cpu, clock, access).unwrap(),
            Phase::Faces(work) => work.step(cpu, clock, access).unwrap(),
            Phase::Raster { raster, .. } => raster.step(cpu, clock, access).unwrap(),
            Phase::Finish(step) => {
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x30),
                    (
                        9,
                        [0xf835, 0xf837, 0xf839, 0xf83b][usize::from(*step)],
                        0x1f00,
                        9,
                        0x30
                    )
                );
                let mut w = Work {
                    cpu,
                    clock,
                    bus: access,
                    cycles: 0,
                };
                w.fetch();
                match *step {
                    0 => w.dp_store(0x1f00, 0, 1),
                    1 => w.immediate(255, 1),
                    2 => w.dp_store(0x1f0c, w.cpu.a & 255, 1),
                    3 => {
                        w.branch(true);
                    }
                    _ => unreachable!(),
                }
                if *step == 3 {
                    self.phase = Phase::Wait(PolyWait::default());
                    w.cpu.pc = 0xf81d;
                } else {
                    *step += 1;
                    w.cpu.pc += 2;
                }
                w.cycles
            }
        };
        WorkerInstruction { master_cycles, irq_masked: deferred_mask.unwrap_or(cpu.p & 4 != 0) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_worker_preserves_rom_instruction_costs_and_upload_publication() {
        use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition,
            RomCpuTimingProbe, Snes, SourcePpuReadState};
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        for angle in 0..=255u8 {
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            // $09:F81D worker entry, with its own bootstrap stack and DP.
            machine.ram[0x1f00..0x1f0a].copy_from_slice(&[
                1, 1, 0, 1, angle, angle.wrapping_mul(3), 32, 32, 128, 0]);
            let mut cpu = NativeNmiContext {
                a: 0, x: 0, y: 0, d: 0x1f00, db: 9, bank: 9,
                pc: 0xf81d, sp: 0x1f3e, p: 0x30,
            };
            machine.cpu.dp = cpu.d;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut ram = machine.ram.clone();
            let mut ppu = machine.ppu.clone();
            let mut clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(225, 86),
                CpuBusWorkload::default(), CpuFieldTiming::NON_INTERLACE_EVEN);
            let mut source = RomCpuTimingProbe::new(machine, clock.clone(),
                SourcePpuReadState::snes9x_reset()).unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut worker = PolyWorker::default();
            let mut divider = crate::cycle_models::poly_vertices::PolyDivider::default();
            for _ in 0..100_000 {
                let label = format!("angle={angle} pc={:04x}", cpu.pc);
                crate::cycle_models::native_work::compare_instruction_with_devices(
                    &label, &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                    Some(&mut ppu), |_, access| divider.access(access),
                    |cpu, clock, bus| { worker.step(cpu, clock, bus); });
                if cpu.pc == 0xf81d && ram[0x1f00] == 0 { break; }
            }
            assert_eq!((cpu.pc, ram[0x1f00], ram[0x1f0c]), (0xf81d, 0, 255));
            assert_eq!(ram, source.snes().ram, "angle={angle}");
        }
    }
}
