//! Module09 composes supported callees without relinquishing its inherited clock.
use super::{overworld_caller::{OverworldCaller, OverworldChild}, overworld_exit_chain::ExitChain,
    overworld_exit_walk::ExitWalkDirection, dungeon_sprite_prefix::{DungeonSpritePrefix, SpritePrefixTail},
    link_oam::LinkOam, hud_update::HudUpdate, native_work::WorkBus, nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Child { Exit(ExitChain), Sprites(DungeonSpritePrefix), Link(LinkOam), Hud(HudUpdate) }
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct OverworldModule { caller: OverworldCaller, child: Option<Child> }
impl OverworldModule {
    pub(crate) fn complete(&self) -> bool { self.caller.complete() }
    pub(crate) fn routine(&self) -> u32 {
        match &self.child {
            Some(Child::Sprites(work)) => work.routine(), Some(Child::Link(_)) => 0x0da18e,
            Some(Child::Hud(_)) => 0x0ddb75, _ => 0x02a475,
        }
    }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline, bus: impl WorkBus) -> Option<u64> {
        if self.complete() { return None; }
        if self.child.is_none() {
            self.child = match self.caller.child() {
                Some(OverworldChild::Submodule(0xac8f)) => Some(Child::Exit(ExitChain::new(ExitWalkDirection::Down))),
                Some(OverworldChild::Submodule(0xacc2)) => Some(Child::Exit(ExitChain::new(ExitWalkDirection::Up))),
                Some(OverworldChild::Sprites) => Some(Child::Sprites(DungeonSpritePrefix::from_overworld_entry(cpu))),
                Some(OverworldChild::LinkOam) => Some(Child::Link(Default::default())),
                Some(OverworldChild::Hud) => Some(Child::Hud(Default::default())),
                Some(child) => panic!("unowned overworld child {child:?} at {cpu:?}"), None => None,
            };
        }
        let Some(child) = &mut self.child else { return self.caller.step(cpu, clock, bus); };
        let (cycles, complete) = match child {
            Child::Exit(work) => { let cycles = work.step(cpu, clock, bus).0; (Some(cycles), work.complete()) }
            Child::Sprites(work) => {
                let cycles = work.step(cpu, clock, bus);
                if let Some(tail) = work.tail() { assert_eq!(tail, SpritePrefixTail::Returned, "unowned overworld sprite tail at {cpu:?}"); }
                (cycles, work.tail() == Some(SpritePrefixTail::Returned))
            }
            Child::Link(work) => { let cycles = work.step(cpu, clock, bus); (cycles, work.complete()) }
            Child::Hud(work) => { let cycles = work.step(cpu, clock, bus); (cycles, work.complete()) }
        };
        if complete { self.caller.return_from_child(cpu); self.child = None; }
        cycles
    }
}
