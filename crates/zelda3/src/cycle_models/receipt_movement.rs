//! Receipt movement gate, clamped upward speed and real Ancilla_MoveY call.
use super::{
    ancilla_axis::AncillaAxis,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReceiptMovement {
    axis: Option<AncillaAxis>,
    complete: bool,
}
impl ReceiptMovement {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        if let Some(axis) = &mut self.axis {
            let cycles = axis.step(cpu, clock, bus);
            self.complete = axis.complete();
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.db, cpu.d, cpu.sp, cpu.p & 0x38),
            (8, 8, 0, 0x1e8, 0x30)
        );
        assert!(cpu.x < 10);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = w.cpu.pc;
        w.fetch();
        let next = match pc {
            0xc5e0 | 0xc5e7 => {
                w.drain(16);
                let v = w.read(
                    w.absolute(if pc == 0xc5e0 { 0x3b1 } else { 0xc22 } + w.cpu.x),
                    1,
                );
                w.load(v, 1);
                pc + 3
            }
            0xc5e3 | 0xc5ed => {
                w.compare_immediate(if pc == 0xc5e3 { 24 } else { 248 }, 1);
                pc + 2
            }
            0xc5e5 | 0xc5ef => {
                let take = w.cpu.p & CARRY == 0;
                w.branch(take);
                if take {
                    if pc == 0xc5e5 {
                        0xc5f7
                    } else {
                        0xc5f4
                    }
                } else {
                    pc + 2
                }
            }
            0xc5ea => {
                w.clear_carry();
                pc + 1
            }
            0xc5eb => {
                w.drain(8);
                w.adc_byte(255);
                pc + 2
            }
            0xc5f1 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xc22 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xc5f4 => {
                w.operand_then_internal(2);
                w.push(0xc5f6, 2);
                self.axis = Some(AncillaAxis::new(0xc5f7));
                0x908b
            }
            _ => unreachable!("receipt movement pc={pc:04x}"),
        };
        w.cpu.pc = next;
        self.complete = next == 0xc5f7;
        Some(w.cycles)
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    super::ancilla_axis::seed_case(r, case);
    r[0x3b1 + case % 10] = [0, 1, 23, 24, 25, 128, 255][case / 40960 % 7];
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn receipt_movement_matches_source_all_velocities_subpixels_and_real_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;

        let mut visited = std::collections::BTreeSet::new();
        for case in 0..286720usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case);
            machine.ram[0x1e9..0x1eb].copy_from_slice(&0x8035u16.to_le_bytes());
            let mut cpu = NativeNmiContext {
                a: 0xab00,
                x: (case % 10) as u16,
                y: 7,
                d: 0,
                db: 8,
                bank: 8,
                pc: 0xc5e0,
                sp: 0x1e8,
                p: 0x30 | (case as u8 & 0xc3),
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
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(58, 1334),
                snes::CpuBusWorkload::default(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source = snes::RomCpuTimingProbe::new(
                machine,
                clock,
                snes::SourcePpuReadState::snes9x_reset(),
            )
            .unwrap();
            let mut clock = source.timeline().clone();
            let mut work = ReceiptMovement::default();
            while !work.complete() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("receipt_movement case={case} pc={pc:04x}"),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                work = work.clone();
                count += 1;
            }

            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [0x08c5e0, 0x08c5e5, 0x08c5f1, 0x08c5f4, 0x08908b, 0x0890b6] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        println!(
            "native receipt_movement: 286720 cases, {count} source instructions, {} PCs",
            visited.len()
        );
    }
}
