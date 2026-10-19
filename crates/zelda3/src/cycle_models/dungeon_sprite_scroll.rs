//! The real saved-scroll stack and sprite drawing view, before SEP/drawing.
use super::{dungeon_common::DungeonCommon, native_work::WorkBus, nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SavedSpriteScroll {
    pub(crate) bg2_x: u16,
    pub(crate) bg2_y: u16,
    pub(crate) bg1_x: u16,
    pub(crate) bg1_y: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonSpriteScroll {
    caller: DungeonCommon,
    saved: SavedSpriteScroll,
    complete: bool,
}
impl Default for DungeonSpriteScroll {
    fn default() -> Self {
        Self {
            caller: DungeonCommon::sprite_scroll(),
            saved: Default::default(),
            complete: false,
        }
    }
}
impl DungeonSpriteScroll {
    pub(crate) fn saved(&self) -> Option<SavedSpriteScroll> {
        self.complete.then_some(self.saved)
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
        let pc = cpu.pc;
        let cycles = self.caller.step(cpu, clock, bus);
        // Capture only words whose PHA just retired on the original stack.
        match pc {
            0x87e9 => self.saved.bg2_x = cpu.a,
            0x87f4 => self.saved.bg2_y = cpu.a,
            0x8800 | 0x882b => self.saved.bg1_x = cpu.a,
            0x880c | 0x8837 => self.saved.bg1_y = cpu.a,
            _ => {}
        }
        self.complete = cpu.pc == 0x8838;
        if self.complete {
            assert_eq!(
                (cpu.bank, cpu.sp, cpu.db, cpu.p & 0x30),
                (2, 0x1f4, 0, 0x10)
            );
        }
        cycles
    }
}

#[cfg(test)]
pub(crate) fn seed_case(ram: &mut [u8], case: usize) {
    for (i, at) in [0xe0, 0xe2, 0xe6, 0xe8, 0x11a, 0x11c, 0x422, 0x424]
        .into_iter()
        .enumerate()
    {
        let word = (case as u16)
            .wrapping_mul([997, 1237, 2267, 3617, 113, 229, 4093, 65521][i])
            .wrapping_add([0, 1, 127, 128, 255, 32767, 32768, 65535][i]);
        ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
    }
    // Only the collision header's low byte controls the moving-floor branch.
    ram[0x428] = [0, 1, 128, 255][case % 4];
    ram[0x429] = (case / 4) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_sprite_scroll_matches_source_stack_words_offsets_and_both_floor_branches() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..8192usize {
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
                pc: 0x87e5,
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
                snes::CpuRasterPosition::new(50, 954),
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
            let mut work = DungeonSpriteScroll::default();
            while work.saved().is_none() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("sprite scroll case={case} pc={pc:04x}"),
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
            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x8838, 0x1f4));
            let saved = work.saved().unwrap();
            let word = |at| u16::from_le_bytes([ram[at], ram[at + 1]]);
            assert_eq!(
                [saved.bg2_x, saved.bg2_y, saved.bg1_x, saved.bg1_y],
                [word(0x1fb), word(0x1f9), word(0x1f7), word(0x1f5)]
            );
            assert_eq!(
                work.step(&mut cpu, &mut clock, |_, _| panic!(
                    "completed scroll has no bus work"
                )),
                None
            );
        }
        for pc in [0x0287e5, 0x0287e9, 0x02881c, 0x02881e, 0x02882b, 0x028837] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        println!("native dungeon sprite scroll: 8192 cases, {count} source instructions, {} PCs, all saved stack words",visited.len());
    }
}
