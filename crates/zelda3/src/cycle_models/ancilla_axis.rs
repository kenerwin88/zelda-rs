//! Native signed fixed-point ancilla XY movement and the real near returns.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AncillaAxis {
    return_pc: u16,
    complete: bool,
}
impl AncillaAxis {
    pub(crate) fn new(return_pc: u16) -> Self {
        Self {
            return_pc,
            complete: false,
        }
    }
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
        assert_eq!((cpu.bank, cpu.db, cpu.d, cpu.p & 0x38), (8, 8, 0, 0x30));
        assert!(cpu.x < 20);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = w.cpu.pc;
        w.fetch();
        let next = match pc {
            0x9080 => {
                w.drain(6);
                w.load(w.cpu.x, 1);
                pc + 1
            }
            0x9081 | 0x9092 => {
                w.clear_carry();
                pc + 1
            }
            0x9082 => {
                w.drain(8);
                w.adc_byte(10);
                pc + 2
            }
            0x9084 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                pc + 1
            }
            0x9085 => {
                w.operand_then_internal(2);
                w.push(0x9087, 2);
                0x908b
            }
            0x9088 => {
                w.operand_then_internal(2);
                0xa752
            }
            0x908b | 0x909b | 0x9093 | 0x90a9 | 0x90b0 => {
                let base = match pc {
                    0x908b | 0x909b => 0xc22,
                    0x9093 => 0xc36,
                    0x90a9 => 0xbfa,
                    _ => 0xc0e,
                };
                let a = base + w.cpu.x;
                w.drain(16);
                if base & 0xff00 != a & 0xff00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a), 1);
                if matches!(pc, 0x908b | 0x909b) {
                    w.load(v, 1)
                } else {
                    w.adc_byte(v)
                };
                pc + 3
            }
            0x908e..=0x9091 => {
                w.shift_left(1);
                pc + 1
            }
            0x9096 | 0x90ac | 0x90b3 => {
                let base = match pc {
                    0x9096 => 0xc36,
                    0x90ac => 0xbfa,
                    _ => 0xc0e,
                };
                w.operand_then_internal(2);
                w.write(w.absolute(base + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0x9099 => {
                w.drain(8);
                w.cpu.y = 0;
                w.flags(0, 1);
                pc + 2
            }
            0x909e => {
                w.drain(6);
                w.push(u16::from(w.cpu.p), 1);
                pc + 1
            }
            0x909f..=0x90a2 => {
                w.drain(6);
                let v = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | ((v & 1) as u8);
                w.load(v >> 1, 1);
                pc + 1
            }
            0x90a3 => {
                w.drain(12);
                w.cpu.p = w.pull(1) as u8;
                pc + 1
            }
            0x90a4 => {
                let take = w.cpu.p & NEGATIVE == 0;
                w.branch(take);
                if take {
                    0x90a9
                } else {
                    pc + 2
                }
            }
            0x90a6 => {
                w.drain(8);
                w.load((w.cpu.a & 255) | 0xf0, 1);
                pc + 2
            }
            0x90a8 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0x90af => {
                w.drain(6);
                w.load(w.cpu.y, 1);
                pc + 1
            }
            0xa752 => {
                w.drain(16);
                w.cpu.x = w.read(w.absolute(0xfa0), 1);
                w.flags(w.cpu.x, 1);
                pc + 3
            }
            0x90b6 | 0xa755 => {
                w.rts();
                w.cpu.pc
            }
            _ => unreachable!("ancilla axis pc={pc:04x}"),
        };
        w.cpu.pc = next;
        self.complete = next == self.return_pc;
        Some(w.cycles)
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    let k = case % 10;
    let v = case / 10;
    r[0xfa0] = k as u8;
    for index in [k, k + 10] {
        let pos = [0u16, 0x00ff, 0x0fff, 0xffff][v / 256 % 4];
        r[0xbfa + index] = pos as u8;
        r[0xc0e + index] = (pos >> 8) as u8;
        r[0xc22 + index] = (v % 256) as u8;
        r[0xc36 + index] = [0, 1, 240, 255][v / 1024 % 4];
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ancilla_axis_matches_source_all_velocities_subpixels_and_real_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;

        let mut visited = std::collections::BTreeSet::new();
        for case in 0..81920usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case / 2);
            machine.ram[0x1e9..0x1eb].copy_from_slice(&0x8035u16.to_le_bytes());
            let mut cpu = NativeNmiContext {
                a: 0xab00,
                x: (case / 2 % 10) as u16,
                y: 7,
                d: 0,
                db: 8,
                bank: 8,
                pc: if case % 2 == 0 { 0x9080 } else { 0x908b },
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
            let mut work = AncillaAxis::new(0x8036);
            while !work.complete() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("ancilla_axis case={case} pc={pc:04x}"),
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
        for pc in [
            0x089080, 0x089085, 0x089088, 0x08908b, 0x089093, 0x0890a3, 0x0890a6, 0x0890b6,
            0x08a755,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        println!(
            "native ancilla_axis: 81920 cases, {count} source instructions, {} PCs",
            visited.len()
        );
    }
}
