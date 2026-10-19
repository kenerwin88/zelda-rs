//! Shared exit walk, camera and map children on the physical caller stack.
use super::{native_work::WorkBus, nmi_stack::NativeNmiContext,
    overworld_camera::OverworldCamera, overworld_exit_walk::{ExitWalkChild, ExitWalkDirection, OverworldExitWalk},
    overworld_map_scroll::OverworldMapScroll, overworld_stripes::OverworldStripes};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExitChain {
    walk: OverworldExitWalk,
    camera: Option<OverworldCamera>,
    map: Option<OverworldMapScroll>,
    stripes: Option<OverworldStripes>,
    routine: u32,
}
impl ExitChain {
    pub(crate) fn complete(&self) -> bool { self.walk.complete() }
    pub(crate) fn new(direction: ExitWalkDirection) -> Self {
        Self {
            walk: OverworldExitWalk::new(direction),
            camera: None,
            map: None,
            stripes: None,
            routine: if direction == ExitWalkDirection::Down {
                0x02ac8f
            } else {
                0x02acc2
            },
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> (u64, u32) {
        if self.camera.is_none() && self.map.is_none() {
            match self.walk.child() {
                Some(ExitWalkChild::Camera) => self.camera = Some(Default::default()),
                Some(ExitWalkChild::MapScroll) => self.map = Some(Default::default()),
                None => {}
            }
        }
        if let Some(map) = &self.map {
            if self.stripes.is_none() {
                if let Some(child) = map.child() {
                    self.stripes = Some(OverworldStripes::new(child, cpu));
                }
            }
        }
        let routine = if self.stripes.is_some() {
            if cpu.pc < 0xf482 {
                0x02f3b9
            } else {
                0x02f482
            }
        } else if self.map.is_some() {
            0x02f273
        } else if self.camera.is_some() {
            0x02bb90
        } else if cpu.bank == 7 {
            if self.routine == 0x02ac8f {
                0x07e69d
            } else {
                0x07e6a6
            }
        } else {
            self.routine
        };
        let cycles = if let Some(work) = &mut self.stripes {
            work.step(cpu, clock, bus)
        } else if let Some(work) = &mut self.map {
            work.step(cpu, clock, bus)
        } else if let Some(work) = &mut self.camera {
            work.step(cpu, clock, bus)
        } else {
            self.walk.step(cpu, clock, bus)
        }
        .unwrap();
        if self
            .stripes
            .as_ref()
            .is_some_and(OverworldStripes::complete)
        {
            self.map.as_mut().unwrap().return_from_child(cpu);
            self.stripes = None;
        }
        if self.map.as_ref().is_some_and(OverworldMapScroll::complete) {
            self.walk.return_from_child(cpu);
            self.map = None;
        }
        if self.camera.as_ref().is_some_and(OverworldCamera::complete) {
            self.walk.return_from_child(cpu);
            self.camera = None;
        }
        (cycles, routine)
    }
}

