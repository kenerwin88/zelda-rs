//! Cached-sprite caller, ordered exit guards and descending inactive slots.
//! Active slots back up and load all fields before their actual far call.
use super::{
    native_work::WorkBus,
    nmi_stack::NativeNmiContext,
    sprite_main::{SpriteChild, SpriteMain},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CachedSpritesTail {
    Returned,
    Loaded {
        slot: u8,
        backup: [u8; 24],
        state: u8,
        timer: super::sprite_timers::SpriteTimersTail,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonCachedSprites {
    entry_sp: Option<u16>,
    work: SpriteMain,
    timers: Option<super::sprite_timers::SpriteTimers>,
    load: Option<super::cached_sprite_load::CachedSpriteLoad>,
    tail: Option<CachedSpritesTail>,
}
impl Default for DungeonCachedSprites {
    fn default() -> Self {
        Self {
            entry_sp: None,
            work: SpriteMain::cached_pass(),
            load: None,
            timers: None,
            tail: None,
        }
    }
}
impl DungeonCachedSprites {
    pub(crate) fn tail(&self) -> Option<CachedSpritesTail> {
        self.tail
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        let entry_sp = *self.entry_sp.get_or_insert(cpu.sp);
        if self.tail.is_some() {
            return None;
        }
        if let Some(work) = &mut self.timers {
            let cycles = work.step(cpu, clock, bus);
            self.tail = work.tail().map(|timer| CachedSpritesTail::Loaded {
                slot: cpu.x as u8,
                backup: self.load.as_ref().unwrap().backup(),
                state: work.state(),
                timer,
            });
            return cycles;
        }
        if let Some(work) = &mut self.load {
            let cycles = work.step(cpu, clock, bus);
            if work.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (6, 0x84da, entry_sp.wrapping_sub(31), 0));
                self.timers = Some(super::sprite_timers::SpriteTimers::cached());
            }
            return cycles;
        }
        let cycles = self.work.step(cpu, clock, bus);
        match self.work.child() {
            Some(SpriteChild::CachedSprite) => {
                assert_eq!((cpu.bank, cpu.pc, cpu.db, cpu.sp), (0x1d, 0xea00, 0, entry_sp.wrapping_sub(4)));
                assert!(cpu.x < 16);
                self.load = Some(Default::default());
            }
            Some(child) => unreachable!("cached pass child {child:?}"),
            None if (cpu.bank, cpu.pc) == (6, 0x83b9) => {
                assert_eq!((cpu.sp, cpu.db, cpu.p & 0x30), (entry_sp.wrapping_add(1), 0, 0x30));
                self.tail = Some(CachedSpritesTail::Returned);
            }
            None => {}
        }
        cycles
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    r[0x1b] = [0, 1, 255][case % 3];
    r[0x11] = [0, 1, 14, 15, 17, 255][case / 3 % 6];
    r[0xffa] = [0, 1, 255][case / 18 % 3];
    r[0x1d00..0x1d10].fill(0);
    let active = case / 54 % 17;
    if active != 16 {
        r[0x1d00 + active] = [1, 9, 255][case / 918 % 3];
    }
    r[0x1f1] = 0;
    super::sprite_timers::seed_case(r, if active == 16 { 0 } else { active }, case);
    for slot in 0..16 {
        r[0x1dd0 + slot] = (case / 3 % 4) as u8;
        r[0x1d10 + slot] = [0x73, 0x7a][case / 256 % 2];
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_sprites_matches_source_guards_all_slots_and_real_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 2];
        let mut timer_tails = [0; 2];
        let mut slots = std::collections::BTreeSet::new();
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
                x: 255,
                y: 7,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x83ae,
                sp: 0x1f0,
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
            let initial_ram = machine.ram.clone();
            let mut ram = machine.ram.clone();
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(76, 54),
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
            let mut work = DungeonCachedSprites::default();
            while work.tail().is_none() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("ancilla caller case={case} pc={pc:04x}"),
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
                CachedSpritesTail::Returned => tails[0] += 1,
                CachedSpritesTail::Loaded {
                    slot,
                    backup,
                    timer,
                    ..
                } => {
                    for (i, a) in crate::game_state::constants::CACHED_SPRITE_LIVE_FIELDS
                        .iter()
                        .enumerate()
                    {
                        assert_eq!(backup[i], initial_ram[*a + usize::from(slot)]);
                    }
                    timer_tails[match timer {
                        super::super::sprite_timers::SpriteTimersTail::Returned => 0,
                        super::super::sprite_timers::SpriteTimersTail::Recoil(_) => 1,
                    }] += 1;
                    tails[1] += 1;
                    slots.insert(slot);
                }
            }
        }
        for pc in [
            0x0683ae, 0x0683b1, 0x0683b4, 0x0683b5, 0x1de9da, 0x1de9de, 0x1de9e2, 0x1de9e6,
            0x1de9eb, 0x1de9ed, 0x1de9f0, 0x1de9f3, 0x1de9f5, 0x1de9f8, 0x1de9f9, 0x1de9fb,
            0x1de9fc, 0x1de9ff,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        assert!(tails.iter().all(|n| *n > 0));
        assert!(timer_tails[0]>0);
        assert_eq!(slots.len(), 16);
        println!("native cached sprites: 8192 cases, {count} source instructions, {} PCs, tails={tails:?}, all16 slots timer_tails={timer_tails:?}",visited.len());
    }
}
