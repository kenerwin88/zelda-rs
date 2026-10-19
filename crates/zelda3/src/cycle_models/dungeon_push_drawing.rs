//! Dungeon SEP/JSL, complete block drawing and allocator, and real far return.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
    push_block_drawing::PushBlockDrawing,
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DungeonPushDrawing {
    call_pending: bool,
    drawing: Option<PushBlockDrawing>,
    complete: bool,
}
impl DungeonPushDrawing {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.drawing.is_some() {
            0x07f0ac
        } else {
            0x028838
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
        if let Some(drawing) = &mut self.drawing {
            let cycles = drawing.step(cpu, clock, bus);
            self.complete = drawing.complete();
            if self.complete {
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.p & 0x30),
                    (2, 0x883e, 0x1f4, 0, 0x30)
                );
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.d, cpu.p & 0x38),
            (
                2,
                if self.call_pending { 0x883a } else { 0x8838 },
                0x1f4,
                0,
                0,
                if self.call_pending { 0x30 } else { 0x10 }
            )
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        if self.call_pending {
            w.operand_then_internal(3);
            w.push(2, 1);
            w.push(0x883d, 2);
            w.cpu.bank = 7;
            w.cpu.pc = 0xf0ac;
            self.drawing = Some(Default::default());
        } else {
            w.width(0x20, false);
            w.cpu.pc = 0x883a;
            self.call_pending = true;
        }
        Some(w.cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    r[0x5fc] = (case % 4 & 1) as u8;
    r[0x5fd] = (case % 4 & 2) as u8;
    r[0x2c3] = (case / 4 % 9) as u8;
    let base: u16 = [0, 0x1f9, 0x1fc, 0x1fd, 0x1fe, 0x200, 0x201, 0x202][case / 36 % 8];
    r[0xfe2..0xfe4].copy_from_slice(&base.to_le_bytes());
    r[0xfee..0xff0].copy_from_slice(&(case as u16).to_le_bytes());
    for (i, at) in [
        0x5e0, 0x5e2, 0x5e4, 0x5e6, 0x5ec, 0x5ee, 0x5f0, 0x5f2, 0xe2, 0xe8,
    ]
    .into_iter()
    .enumerate()
    {
        let word = (case as u16)
            .wrapping_mul(997)
            .wrapping_add((i * 257) as u16);
        r[at..at + 2].copy_from_slice(&word.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_push_drawing_matches_source_slots_modes_allocator_and_real_caller_return() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..2304usize {
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
                pc: 0x8838,
                sp: 0x1f4,
                p: 0x10 | (case as u8 & 0xc3),
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
                snes::CpuRasterPosition::new(
                    [51, 224, 261, 261][case / 576],
                    [478, 1300, 1200, 1350][case / 576],
                ),
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
            let mut work = DungeonPushDrawing::default();
            while !work.complete() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("drawing caller case={case} pc={pc:04x}"),
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
            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x883e, 0x1f4));
        }
        for pc in [
            0x028838, 0x02883a, 0x07f0ac, 0x07f0c2, 0x07f0ca, 0x07f119, 0x07f11b, 0x07f138,
            0x07f13b, 0x0dbb1e, 0x0dbb41,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        println!("native dungeon push drawing: 2304 cases, {count} source instructions, {} PCs, all modes and far returns",visited.len());
    }
}
