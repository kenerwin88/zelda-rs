//! Module 7 retains its dispatch, selected submodule and common suffix on one
//! live stack. Unowned calls remain explicit boundaries until their native
//! owner retires the real return; this owner never fabricates a return.
use super::{
    dungeon_common::{CommonChild, DungeonCommon},
    dungeon_dispatch::DungeonDispatch,
    dungeon_landing::{DungeonLanding, LandingChild},
    native_work::WorkBus,
    nmi_stack::NativeNmiContext,
};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    super::dungeon_dispatch::table_byte(address)
        .or_else(|| super::dungeon_landing::table_byte(address))
        .or_else(|| super::dungeon_common::table_byte(address))
        .or_else(|| crate::ZeldaState::native_spotlight_table_byte(address))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DungeonChild {
    Submodule(u32),
    Landing(LandingChild),
    Common(CommonChild),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Phase {
    Dispatch(DungeonDispatch),
    Landing(DungeonLanding),
    Submodule(u32),
    Common(DungeonCommon),
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonModule {
    phase: Phase,
    entry_sp: Option<u16>,
}

impl Default for DungeonModule {
    fn default() -> Self {
        Self {
            phase: Phase::Dispatch(Default::default()),
            entry_sp: None,
        }
    }
}

impl DungeonModule {
    pub(crate) fn complete(&self) -> bool {
        matches!(self.phase, Phase::Complete)
    }

    pub(crate) fn child(&self) -> Option<DungeonChild> {
        match &self.phase {
            Phase::Submodule(target) => Some(DungeonChild::Submodule(*target)),
            Phase::Landing(work) => work.child().map(DungeonChild::Landing),
            Phase::Common(work) => work.child().map(DungeonChild::Common),
            _ => None,
        }
    }

    fn enter_common(&mut self, cpu: &NativeNmiContext) {
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.sp),
            (2, 0x87af, self.entry_sp.unwrap()),
            "the submodule must retire its actual near return"
        );
        self.phase = Phase::Common(Default::default());
    }

    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        assert!(self.child().is_some(), "no dungeon child owns the CPU");
        match &mut self.phase {
            Phase::Submodule(_) => self.enter_common(cpu),
            Phase::Landing(work) => work.return_from_child(cpu),
            Phase::Common(work) => work.return_from_child(cpu),
            _ => unreachable!(),
        }
    }

    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete() {
            return None;
        }
        assert!(self.child().is_none(), "dungeon child {:?} still owns the CPU at {cpu:?}, field={} raster={:?}",
            self.child(), clock.field_index(), clock.raster_position());
        self.entry_sp.get_or_insert(cpu.sp);
        match &mut self.phase {
            Phase::Dispatch(work) => {
                let cycles = work.step(cpu, clock, bus);
                if work.entered_submodule() {
                    assert_eq!(cpu.sp, self.entry_sp.unwrap().wrapping_sub(2));
                    let target = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
                    self.phase = if target == 0x02931d {
                        Phase::Landing(Default::default())
                    } else {
                        Phase::Submodule(target)
                    };
                }
                cycles
            }
            Phase::Landing(work) => {
                let cycles = work.step(cpu, clock, bus);
                if work.complete() {
                    self.enter_common(cpu);
                }
                cycles
            }
            Phase::Common(work) => {
                let cycles = work.step(cpu, clock, bus);
                if work.complete() {
                    assert_eq!(cpu.sp, self.entry_sp.unwrap().wrapping_add(3));
                    self.phase = Phase::Complete;
                }
                cycles
            }
            Phase::Submodule(_) | Phase::Complete => unreachable!(),
        }
    }
}
