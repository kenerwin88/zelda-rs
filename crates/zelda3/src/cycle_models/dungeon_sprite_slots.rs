//! Descending sprite slot loop, inactive markers and actual active entry guard.
use super::{native_work::WorkBus, nmi_stack::NativeNmiContext, sprite_main::SpriteMain};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpriteSlotsTail {
    AfterTimers {
        slot: u8,
        state: u8,
        tail: super::sprite_timers::SpriteTimersTail,
    },
    LowerGarnish,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonSpriteSlots {
    entry_sp: Option<u16>,
    work: SpriteMain,
    timers: Option<super::sprite_timers::SpriteTimers>,
    tail: Option<SpriteSlotsTail>,
    inactive_bodies: u16,
    live_inactive_slot_bodies: bool,
    native_slot_bodies: bool,
    body: Option<super::sprite_slot::SpriteSlot>,
}
impl Default for DungeonSpriteSlots {
    fn default() -> Self {
        Self {
            entry_sp: None,
            work: SpriteMain::first_slot(),
            timers: None,
            tail: None,
            inactive_bodies: 0,
            live_inactive_slot_bodies: false,
            native_slot_bodies: false,
            body: None,
        }
    }
}
impl DungeonSpriteSlots {
    pub(crate) fn with_native_slot_bodies(mut self,enabled:bool)->Self {self.native_slot_bodies=enabled;self}
    pub(crate) fn with_live_inactive_slot_bodies(mut self, enabled:bool)->Self {self.live_inactive_slot_bodies=enabled;self}
    pub(crate) fn with_inactive_slot_bodies(mut self, mask: u16) -> Self { self.inactive_bodies=mask; self }
    pub(crate) fn tail(&self) -> Option<SpriteSlotsTail> {
        self.tail
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        mut bus: impl WorkBus,
    ) -> Option<u64> {
        let entry_sp = *self.entry_sp.get_or_insert(cpu.sp);
        if self.live_inactive_slot_bodies && self.timers.is_some()
            && (cpu.bank, cpu.pc) == (6, 0x84d9) {
            // Timer instructions can clear an actor's pause before its body.
            // Sample at the real timer RTS; an entry-only mask can
            // incorrectly admit active AI. Drawing still retires its own
            // pause stores and double returns on the physical stack.
            self.inactive_bodies = bus.inactive_sprite_body_mask();
        }
        if self.tail.is_some() {
            return None;
        }
        if let Some(work) = &mut self.body {
            let cycles = work.step(cpu, clock, bus);
            assert!(work.child().is_none(), "sprite slot body child {:?} at {cpu:?}", work.child());
            if work.complete() {
                self.work.return_from_slot_body(cpu);
                self.body = None;
            }
            return cycles;
        }
        if let Some(work) = &mut self.timers {
            let cycles = work.step(cpu, clock, bus);
            if work.tail() == Some(super::sprite_timers::SpriteTimersTail::Returned)
                && work.state() == 9 && (self.native_slot_bodies || self.inactive_bodies & (1 << cpu.x) != 0) {
                self.body = Some(work.take_slot_body());
                self.timers = None;
                return cycles;
            }
            self.tail = work.tail().map(|tail| SpriteSlotsTail::AfterTimers {
                slot: cpu.x as u8,
                state: work.state(),
                tail,
            });
            return cycles;
        }
        let cycles = self.work.step(cpu, clock, bus);
        self.tail = match (cpu.bank, cpu.pc) {
            (6, 0x84e7) => {
                assert_eq!((cpu.sp, cpu.db, cpu.p & 0x30), (entry_sp.wrapping_sub(2), 6, 0x30));
                assert!(cpu.x < 16 && cpu.a & 255 != 0);
                self.timers = Some(super::sprite_timers::SpriteTimers::after_entry_with_slot(cpu.a as u8, self.work.take_slot_body()));
                None
            }
            (6, 0x83aa) => {
                assert_eq!((cpu.sp, cpu.db, cpu.x, cpu.p & 0x30), (entry_sp, 6, 255, 0x30));
                Some(SpriteSlotsTail::LowerGarnish)
            }
            _ => None,
        };
        cycles
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    r[0x1b] = (case % 2) as u8;
    r[0xdd0..0xde0].fill(0);
    if case / 2 % 12 != 0 {
        r[0xdd0 + case / 24 % 16] = (case / 2 % 12) as u8;
    }
    super::sprite_timers::seed_case(r, case / 24 % 16, case);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_sprite_slots_matches_source_all_slots_states_worlds_and_inactive_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 2];
        let mut slots = std::collections::BTreeSet::new();
        let mut kinds = std::collections::BTreeSet::new();
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
                x: 15,
                y: 7,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x83a1,
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
            let mut ram = machine.ram.clone();
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(69, 452),
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
            let mut work = DungeonSpriteSlots::default();
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
                SpriteSlotsTail::LowerGarnish => tails[0] += 1,
                SpriteSlotsTail::AfterTimers { slot, state, .. } => {
                    tails[1] += 1;
                    slots.insert(slot);
                    kinds.insert(state);
                }
            }
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [
            0x0683a1, 0x0683a4, 0x0684e2, 0x0684e5, 0x068510, 0x068514, 0x068519, 0x06851c,
            0x06851f, 0x068520, 0x068522, 0x068525, 0x0683a7, 0x0683a8,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        assert!(tails.iter().all(|n| *n > 0));
        assert_eq!(slots.len(), 16);
        assert_eq!(kinds.len(), 11);
        println!("native dungeon sprite slots: 8192 cases, {count} source instructions, {} PCs, tails={tails:?}, all16 slots and11 active states in both worlds",visited.len());
    }
    #[test]
    fn inactive_zelda_and_priest_bodies_resume_the_actual_descending_slot_loop() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 2];
        let mut slots = std::collections::BTreeSet::new();
        let mut kinds = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..128usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            machine.ram.fill(0);
            let r = &mut machine.ram;
            r[0x1b] = 1;
            r[0xfc1] = 1;
            r[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
            r[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
            for slot in [0,14,15] {
                r[0xdd0+slot] = 9;
                r[0xe20+slot] = if slot==15 {0x76} else {0x73};
                r[0xe90+slot] = if slot==14 {2} else if slot==0 {1} else {0};
                r[0xde0+slot] = (case % 2) as u8;
                r[0xdc0+slot] = (case / 4 % 2) as u8;
                r[0xd10+slot] = (case as u8).wrapping_mul(7);
                r[0xd00+slot] = (case as u8).wrapping_mul(11);
            }
            let mut cpu = NativeNmiContext {
                a: 0xab00,
                x: 15,
                y: 7,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x83a1,
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
            let mut ram = machine.ram.clone();
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(69, 452),
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
            let mut work = DungeonSpriteSlots::default().with_inactive_slot_bodies((1<<15)|(1<<14)|1);
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
                SpriteSlotsTail::LowerGarnish => tails[0] += 1,
                SpriteSlotsTail::AfterTimers { slot, state, .. } => {
                    tails[1] += 1;
                    slots.insert(slot);
                    kinds.insert(state);
                }
            }
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        assert_eq!(tails, [128,0]);
        assert!(visited.contains(&0x05ec96));
        assert!(visited.contains(&0x05ec9c));
        assert!(visited.contains(&0x05f969));
        assert!(visited.contains(&0x0683a7));
        println!("128 native inactive actor slot-loop fixtures, {count} reference instruction boundaries");
    }

}
