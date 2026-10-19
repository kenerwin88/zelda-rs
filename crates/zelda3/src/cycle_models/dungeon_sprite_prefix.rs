//! Real Sprite_Main call, world guards, region reset and garnish call.
//! Outdoor proximity remains an explicit child on its real far stack.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
    sprite_main::{SpriteChild, SpriteMain},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpritePrefixTail {
    Returned,
    Proximity,
    Garnish,
    AfterGarnish(super::dungeon_garnish::GarnishTail),
    AfterFollower(super::dungeon_follower::FollowerTail),
    AfterAncilla(super::dungeon_ancilla::AncillaTail),
    AfterOverlord(super::dungeon_overlord::OverlordTail),
    AfterSlots(super::dungeon_sprite_slots::SpriteSlotsTail),
    AfterCachedSprites(super::dungeon_cached_sprites::CachedSpritesTail),
    AfterLowerGarnish(super::dungeon_lower_garnish::LowerGarnishTail),
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DungeonSpritePrefix {
    entry_sp: Option<u16>,
    closing_call: bool,
    opening_call: bool,
    proximity: Option<super::overworld_sprite_proximity::OverworldSpriteProximity>,
    work: Option<SpriteMain>,
    native_garnish: bool,
    native_follower: bool,
    native_ancilla: bool,
    inactive_slot_bodies: u16,
    live_inactive_slot_bodies: bool,
    ancilla: Option<super::dungeon_ancilla::DungeonAncilla>,
    overlord: Option<super::dungeon_overlord::DungeonOverlord>,
    slots: Option<super::dungeon_sprite_slots::DungeonSpriteSlots>,
    cached: Option<super::dungeon_cached_sprites::DungeonCachedSprites>,
    return_work: Option<SpriteMain>,
    lower_garnish: Option<super::dungeon_lower_garnish::DungeonLowerGarnish>,
    follower: Option<super::dungeon_follower::DungeonFollower>,
    garnish: Option<super::dungeon_garnish::DungeonGarnish>,
    tail: Option<SpritePrefixTail>,
}
impl DungeonSpritePrefix {
    /// Module09 has already retired JSL Sprite_Main and its eight scroll-stack bytes.
    pub(crate) fn from_overworld_entry(cpu: &NativeNmiContext) -> Self {
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (6, 0x8328, 0x1f1));
        Self { entry_sp: Some(cpu.sp.wrapping_add(3)), work: Some(Default::default()),
            ..Self::opening() }
    }
    pub(crate) fn opening()->Self {
        Self {opening_call:true,..Self::closing()}
    }
    /// Module0F owns a shallower far frame than the dungeon suffix. Execute
    /// its actual JSL and derive every child stack check from that frame.
    pub(crate) fn closing() -> Self {
        Self { closing_call: true, ..Default::default() }
            .with_native_garnish(true).with_native_follower(true)
            .with_native_ancilla(true).with_live_inactive_slot_bodies(true)
    }
    pub(crate) fn with_live_inactive_slot_bodies(mut self, enabled:bool)->Self {self.live_inactive_slot_bodies=enabled;self}
    pub(crate) fn with_inactive_slot_bodies(mut self, mask:u16)->Self {self.inactive_slot_bodies=mask;self}
    pub(crate) fn with_native_ancilla(mut self, enabled: bool) -> Self {
        self.native_ancilla = enabled;
        self
    }
    pub(crate) fn with_native_follower(mut self, enabled: bool) -> Self {
        self.native_follower = enabled;
        self
    }
    pub(crate) fn with_native_garnish(mut self, enabled: bool) -> Self {
        self.native_garnish = enabled;
        self
    }
    pub(crate) fn tail(&self) -> Option<SpritePrefixTail> {
        self.tail
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.proximity.is_some() {0x09c58f}
        else if self.return_work.is_some() {
            0x068328
        } else if self.cached.is_some() {
            0x1de9da
        } else if self.lower_garnish.is_some() {
            0x09b07f
        } else if self.slots.is_some() {
            0x068328
        } else if self.overlord.is_some() {
            0x09b773
        } else if self.ancilla.is_some() {
            0x088242
        } else if self.follower.is_some() {
            0x099f91
        } else if self.garnish.is_some() {
            0x09b06e
        } else if self.work.is_some() {
            0x068328
        } else {
            if self.opening_call {0x029ad7} else if self.closing_call { 0x029982 } else { 0x02883e }
        }
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
        if let Some(work)=&mut self.proximity {
            let cycles=work.step(cpu,clock,bus);
            if work.complete() {
                self.work.as_mut().unwrap().return_from_child(cpu);
                self.proximity=None;
            }
            return cycles;
        }
        if let Some(work) = &mut self.return_work {
            let cycles = work.step(cpu, clock, bus);
            if work.complete() { self.tail = Some(SpritePrefixTail::Returned); }
            return cycles;
        }
        if let Some(work)=&mut self.cached {
            let cycles=work.step(cpu,clock,bus);
            match work.tail() {
                Some(super::dungeon_cached_sprites::CachedSpritesTail::Returned) => {
                    self.return_work = Some(SpriteMain::after_cached());
                }
                tail => self.tail=tail.map(SpritePrefixTail::AfterCachedSprites),
            }
            return cycles;
        }
        if let Some(work)=&mut self.lower_garnish {
            let cycles=work.step(cpu,clock,bus);
            if work.tail()==Some(super::dungeon_lower_garnish::LowerGarnishTail::Returned) {
                self.cached=Some(Default::default());
            } else {
                self.tail=work.tail().map(SpritePrefixTail::AfterLowerGarnish);
            }
            return cycles;
        }
        if let Some(work)=&mut self.slots {
            let cycles=work.step(cpu,clock,bus);
            if work.tail()==Some(super::dungeon_sprite_slots::SpriteSlotsTail::LowerGarnish) {
                self.lower_garnish=Some(Default::default());
            } else {
                self.tail=work.tail().map(SpritePrefixTail::AfterSlots);
            }
            return cycles;
        }
        if let Some(work) = &mut self.overlord {
            let cycles = work.step(cpu,clock,bus);
            if work.tail()==Some(super::dungeon_overlord::OverlordTail::Returned) {
                self.slots=Some(super::dungeon_sprite_slots::DungeonSpriteSlots::default().with_inactive_slot_bodies(self.inactive_slot_bodies).with_live_inactive_slot_bodies(self.live_inactive_slot_bodies).with_native_slot_bodies(self.opening_call));
            } else {
                self.tail = work.tail().map(SpritePrefixTail::AfterOverlord);
            }
            return cycles;
        }
        if let Some(work) = &mut self.ancilla {
            let cycles = work.step(cpu, clock, bus);
            if work.tail()==Some(super::dungeon_ancilla::AncillaTail::Returned) {
                self.overlord=Some(Default::default());
            } else {
                self.tail = work.tail().map(SpritePrefixTail::AfterAncilla);
            }
            return cycles;
        }
        if let Some(work) = &mut self.follower {
            let cycles = work.step(cpu, clock, bus);
            if work.tail() == Some(super::dungeon_follower::FollowerTail::Returned)
                && self.native_ancilla
            {
                self.ancilla = Some(
                    super::dungeon_ancilla::DungeonAncilla::default()
                        .with_native_receipt(true)
                        .with_native_oam(true)
                        .with_native_slot_return(true),
                );
            } else {
                self.tail = work.tail().map(SpritePrefixTail::AfterFollower);
            }
            return cycles;
        }
        if let Some(work) = &mut self.garnish {
            let cycles = work.step(cpu, clock, bus);
            if work.tail() == Some(super::dungeon_garnish::GarnishTail::Returned)
                && self.native_follower
            {
                self.follower = Some(Default::default());
            } else {
                self.tail = work.tail().map(SpritePrefixTail::AfterGarnish);
            }
            return cycles;
        }
        if let Some(work) = &mut self.work {
            let cycles = work.step(cpu, clock, bus);
            if work.child() == Some(SpriteChild::Proximity) {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (9, 0xc58f, entry_sp.wrapping_sub(6), 0));
                if self.opening_call {self.proximity=Some(Default::default());}
                else {self.tail = Some(SpritePrefixTail::Proximity);}
            } else if (cpu.bank, cpu.pc) == (9, 0xb06e) {
                assert_eq!((cpu.sp, cpu.db, cpu.p & 0x30), (entry_sp.wrapping_sub(7), 6, 0x30));
                if self.native_garnish {
                    self.garnish = Some(Default::default());
                } else {
                    self.tail = Some(SpritePrefixTail::Garnish);
                }
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.d, cpu.p & 0x38),
            (2, if self.opening_call {0x9ad7} else if self.closing_call { 0x9982 } else { 0x883e },
                if self.closing_call { 0x1fc } else { 0x1f4 }, 0, 0, 0x30)
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
        w.push(if self.opening_call {0x9ada} else if self.closing_call { 0x9985 } else { 0x8841 }, 2);
        w.cpu.bank = 6;
        w.cpu.pc = 0x8328;
        self.work = Some(Default::default());
        Some(w.cycles)
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    r[0x1b] = (case % 2) as u8;
    r[0xf3ca] = (case / 2 % 256) as u8;
    r[0x11] = [0, 1, 14, 15, 17, 20, 128, 255][case / 512 % 8];
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_sprite_prefix_matches_source_world_submodule_region_reset_and_child_stacks() {
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
                pc: 0x883e,
                sp: 0x1f4,
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
                snes::CpuRasterPosition::new(51, 832),
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
            let mut work = DungeonSpritePrefix::default();
            while work.tail().is_none() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("sprite prefix case={case} pc={pc:04x}"),
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
            assert_eq!(
                work.tail(),
                Some(if case % 2 == 0 {
                    SpritePrefixTail::Proximity
                } else {
                    SpritePrefixTail::Garnish
                })
            );
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [
            0x02883e, 0x068328, 0x06832c, 0x06833b, 0x06833f, 0x06834a, 0x068352, 0x06835e,
            0x0683d7, 0x0683e5, 0x068361,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        println!("native dungeon sprite prefix: 8192 cases, {count} source instructions, {} PCs, both child stacks",visited.len());
    }
}
