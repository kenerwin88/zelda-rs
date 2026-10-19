//! Complete upper garnish caller, flash gates and empty-slot scan.
//! Selected palette/slot bodies retain their actual far/near stack.
use super::{
    native_work::WorkBus,
    nmi_stack::NativeNmiContext,
    sprite_main::{SpriteChild, SpriteMain},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GarnishTail {
    Returned,
    PaletteRestore,
    PaletteWhiten,
    PaletteFromFlash,
    Slot { slot: u8, kind: u8 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonGarnish {
    entry_sp: Option<u16>,
    work: SpriteMain,
    tail: Option<GarnishTail>,
}
impl Default for DungeonGarnish {
    fn default() -> Self {
        Self {
            entry_sp: None,
            work: SpriteMain::garnish_upper(),
            tail: None,
        }
    }
}
impl DungeonGarnish {
    pub(crate) fn tail(&self) -> Option<GarnishTail> {
        self.tail
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.tail.is_some() {
            return None;
        }
        let entry_sp = *self.entry_sp.get_or_insert(cpu.sp);
        let cycles = self.work.step(cpu, clock, bus);
        self.tail = match self.work.child() {
            Some(SpriteChild::PaletteRestore) => Some(GarnishTail::PaletteRestore),
            Some(SpriteChild::PaletteAdd) => Some(GarnishTail::PaletteWhiten),
            Some(SpriteChild::PaletteSubtract) => Some(GarnishTail::PaletteFromFlash),
            Some(SpriteChild::Garnish) => {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (9, 0xb0bf, entry_sp.wrapping_sub(5), 9));
                assert!((15..30).contains(&cpu.x));
                Some(GarnishTail::Slot {
                    slot: cpu.x as u8,
                    kind: cpu.a as u8,
                })
            }
            Some(child) => unreachable!("upper garnish child {child:?}"),
            None if self.work.complete() => {
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.p & 0x30),
                    (6, 0x8365, entry_sp.wrapping_add(3), 6, 0x30)
                );
                Some(GarnishTail::Returned)
            }
            None => None,
        };
        if matches!(
            self.tail,
            Some(
                GarnishTail::PaletteRestore
                    | GarnishTail::PaletteWhiten
                    | GarnishTail::PaletteFromFlash
            )
        ) {
            assert_eq!((cpu.bank, cpu.sp, cpu.db), (14, entry_sp.wrapping_sub(6), 6));
        }
        cycles
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    r[0xff9] = (case / 16 % 256) as u8;
    r[0x11] = if case / 8 % 2 == 0 { 0 } else { 15 };
    r[0xfb4] = u8::from(case % 8 != 0);
    r[0x1f800..0x1f81e].fill(0);
    if case % 8 >= 2 {
        let slot = 15 + case * 7 % 15;
        r[0x1f800 + slot] = [1, 5, 10, 22][case / 4 % 4];
    }
    r[0x1ee..0x1f1].copy_from_slice(&[0x64, 0x83, 6]);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_garnish_matches_source_flash_gates_upper_slots_and_child_stacks() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 5];
        let mut slots = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..4096usize {
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
                db: 6,
                bank: 9,
                pc: 0xb06e,
                sp: 0x1ed,
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
                snes::CpuRasterPosition::new(52, 1166),
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
            let mut work = DungeonGarnish::default();
            while work.tail().is_none() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("garnish caller case={case} pc={pc:04x}"),
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
            match work.tail().unwrap() {
                GarnishTail::Returned => tails[0] += 1,
                GarnishTail::PaletteRestore => tails[1] += 1,
                GarnishTail::PaletteWhiten => tails[2] += 1,
                GarnishTail::PaletteFromFlash => tails[3] += 1,
                GarnishTail::Slot { slot, .. } => {
                    tails[4] += 1;
                    slots.insert(slot);
                }
            }
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [
            0x09b06e, 0x1de9b6, 0x1de9bf, 0x1de9c4, 0x1de9cd, 0x1de9d3, 0x09b077, 0x09b08c,
            0x09b0b9, 0x09b123, 0x09b07e,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        assert!(tails.iter().all(|n| *n > 0));
        assert_eq!(slots.len(), 15);
        println!("native dungeon garnish: 4096 cases, {count} source instructions, {} PCs, tails={tails:?}, all 15 upper slots",visited.len());
    }
}
