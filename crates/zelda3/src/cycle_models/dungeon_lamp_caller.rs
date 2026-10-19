//! The dungeon lamp call and its complete native cone/scroll body.
use super::{
    lamp_cone::LampCone,
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DungeonLampCaller {
    lamp: Option<LampCone>,
    complete: bool,
}
impl DungeonLampCaller {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.lamp.is_some() {
            0x00f567
        } else {
            0x0287e1
        }
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
        if let Some(lamp) = &mut self.lamp {
            let cycles = lamp.step(cpu, clock, bus);
            self.complete = lamp.complete();
            if self.complete {
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.p & 0x30),
                    (2, 0x87e5, 0x1fc, 0, 0x30)
                );
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.d, cpu.p & 0x38),
            (2, 0x87e1, 0x1fc, 0, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        w.operand_then_internal(3);
        w.push(2, 1);
        w.push(0x87e4, 2);
        w.cpu.bank = 0;
        w.cpu.pc = 0xf567;
        self.lamp = Some(Default::default());
        Some(w.cycles)
    }
}
#[cfg(test)]
pub(crate) fn seed_case(ram: &mut [u8], case: usize) {
    let case = case % 6144;
    ram[0x458] = (case / 3072) as u8;
    ram[0x11] = [0, 15, 20][case / 1024 % 3];
    ram[0x6c] = (case / 256 % 4) as u8;
    ram[0x2f] = (case / 64 % 4 * 2) as u8;
    for (i, at) in [0x20, 0x22, 0xe0, 0xe2, 0xe6, 0xe8].into_iter().enumerate() {
        let word = (case as u16 % 64)
            .wrapping_mul([1237, 997, 113, 2267, 229, 3617][i])
            .wrapping_add([0, 8, 0, 127, 255, 128][i]);
        ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_lamp_caller_matches_source_dark_guards_doorways_clamps_and_far_return() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..6144usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case);
            let mut cpu = NativeNmiContext {
                a: 0xab00,
                x: 3,
                y: 7,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x87e1,
                sp: 0x1fc,
                p: 0x30 | (case as u8 & 0xc3),
            };
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut ram = machine.ram.clone();
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(50, 794),
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
            let mut work = DungeonLampCaller::default();
            while !work.complete() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("lamp caller case={case} pc={pc:04x}"),
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
            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87e5, 0x1fc));
        }
        for pc in [
            0x0287e1, 0xf566, 0xf59f, 0xf5b0, 0xf5e9, 0xf5f2, 0xf5ff, 0xf600, 0xf632, 0xf63b,
            0xf648,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        println!("native dungeon lamp caller: 6144 cases, {count} source instructions, {} PCs, all far returns",visited.len());
    }
}
