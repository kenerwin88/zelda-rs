use super::{
    DisplaySnapshotPublication, GameWorkContinuation, ItemReceiptGraphicsContinuation,
    PreMainCallerContinuation, PreMainNmiResume, SpotlightIteration,
    FILE_SELECT_GRAPHICS_NMI_SLICES, SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES,
    SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES,
};
pub(super) use snes::{CpuBusEvent, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition};
use snes::{
    CpuMasterTimeline, CpuTimelineDeadlineAdvance, CpuTimelineEvent, NMI_SCANLINE,
    SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES, SNES9X_NMI_GENERAL_DMA_DELAY_MASTER_CYCLES,
    WRAM_REFRESH_STALL_MASTER_CYCLES,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CpuWorkAdvance {
    Complete,
    ReachedBoundary {
        boundary: CpuRasterBoundary,
        remaining_work_master_cycles: u32,
    },
}

/// Result of advancing a sequence of semantic CPU phases toward a raster
/// boundary.
///
/// The phase index is stable across translated implementations: callers can
/// distinguish an interrupted routine body from an interrupted caller suffix
/// without recovering that distinction from a room or module-state value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CpuPhaseSequenceAdvance {
    Complete,
    ReachedBoundary {
        boundary: CpuRasterBoundary,
        phase_index: usize,
        remaining_work_master_cycles: u32,
    },
}

impl CpuWorkAdvance {
    pub(super) const fn reached_boundary(self) -> Option<CpuRasterBoundary> {
        match self {
            Self::Complete => None,
            Self::ReachedBoundary { boundary, .. } => Some(boundary),
        }
    }
}

/// The two hardware-visible boundaries which translated work and original-ROM
/// execution can target. VBlank publication is a display ownership boundary at
/// H=0; an enabled NMI becomes eligible for CPU acceptance at H=12.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CpuRasterBoundary {
    VblankPublication,
    CpuNmiAcceptance,
}

impl CpuRasterBoundary {
    const fn raster_position(self) -> CpuRasterPosition {
        CpuRasterPosition::new(
            NMI_SCANLINE as u16,
            match self {
                Self::VblankPublication => 0,
                Self::CpuNmiAcceptance => SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES as u16,
            },
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CpuBoundaryDeadline {
    boundary: CpuRasterBoundary,
    master_cycles: u64,
}

impl CpuBoundaryDeadline {
    fn next_after(
        entry: CpuRasterPosition,
        boundary: CpuRasterBoundary,
        field_timing: CpuFieldTiming,
    ) -> Self {
        let entry_master_cycles = field_timing.master_cycles_at(0, entry);
        let boundary_master_cycles = field_timing.master_cycles_at(0, boundary.raster_position());
        let field = u64::from(entry_master_cycles >= boundary_master_cycles);
        Self {
            boundary,
            master_cycles: field_timing.master_cycles_at(field, boundary.raster_position()),
        }
    }
}

/// Remaining CPU time before a typed raster boundary, including refresh and
/// HDMA bus steals. Instructions are advanced atomically: when a boundary is
/// reached in the middle of an instruction, that instruction completes before
/// the continuation is suspended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CpuCycleBudget {
    timeline: CpuMasterTimeline,
    deadline: CpuBoundaryDeadline,
}

impl CpuCycleBudget {
    pub(super) fn until_next_vblank_publication(
        entry: CpuRasterPosition,
        bus: CpuBusWorkload,
        field_timing: CpuFieldTiming,
    ) -> Self {
        Self::until_next_boundary(
            entry,
            CpuRasterBoundary::VblankPublication,
            bus,
            field_timing,
        )
    }

    pub(super) fn until_next_nmi_acceptance(
        entry: CpuRasterPosition,
        bus: CpuBusWorkload,
        field_timing: CpuFieldTiming,
    ) -> Self {
        Self::until_next_boundary(
            entry,
            CpuRasterBoundary::CpuNmiAcceptance,
            bus,
            field_timing,
        )
    }

    fn until_next_boundary(
        entry: CpuRasterPosition,
        boundary: CpuRasterBoundary,
        bus: CpuBusWorkload,
        field_timing: CpuFieldTiming,
    ) -> Self {
        let timeline = CpuMasterTimeline::at_raster(0, entry, bus, field_timing);
        let deadline = CpuBoundaryDeadline::next_after(entry, boundary, field_timing);
        debug_assert!(timeline.clock_master_cycles() < deadline.master_cycles);
        Self { timeline, deadline }
    }

    /// Start at the pinned Snes9x core's earliest CPU NMI acceptance boundary
    /// so the caller can execute the interrupt entry and handler before the
    /// main-thread entry position.
    ///
    /// The existing synthetic dungeon main-wait seed still models its busy
    /// loop separately; this constructor does not claim that seed is a WAI.
    pub(super) fn at_nmi_acceptance(bus: CpuBusWorkload, field_timing: CpuFieldTiming) -> Self {
        let deadline = CpuBoundaryDeadline {
            boundary: CpuRasterBoundary::CpuNmiAcceptance,
            master_cycles: field_timing
                .master_cycles_at(0, CpuRasterBoundary::CpuNmiAcceptance.raster_position()),
        };
        Self {
            timeline: CpuMasterTimeline::at_raster(
                0,
                CpuRasterBoundary::CpuNmiAcceptance.raster_position(),
                bus,
                field_timing,
            ),
            deadline,
        }
    }

    /// Advance an interruptible span, stopping exactly at the selected boundary
    /// and retaining the unexecuted work for a continuation.
    pub(super) fn advance_interruptible(&mut self, work_master_cycles: u32) -> CpuWorkAdvance {
        debug_assert!(self.timeline.clock_master_cycles() < self.deadline.master_cycles);
        match self
            .timeline
            .advance_interruptible_until(self.deadline.master_cycles, work_master_cycles)
        {
            CpuTimelineDeadlineAdvance::Complete => CpuWorkAdvance::Complete,
            CpuTimelineDeadlineAdvance::ReachedDeadline {
                remaining_work_master_cycles,
            } => CpuWorkAdvance::ReachedBoundary {
                boundary: self.deadline.boundary,
                remaining_work_master_cycles,
            },
        }
    }

    /// Advance one indivisible 65816 instruction. A boundary reached during the
    /// instruction is observed immediately after it completes.
    pub(super) fn advance_instruction(&mut self, instruction_master_cycles: u32) -> CpuWorkAdvance {
        debug_assert!(self.timeline.clock_master_cycles() < self.deadline.master_cycles);
        self.advance_atomic_work(instruction_master_cycles)
    }

    /// Advance one instruction while deriving each HDMA steal from the cloned
    /// machine's live channel/table state. This is used by ROM timing shadows;
    /// translated fixed-cost phases retain `advance_instruction`.
    pub(super) fn advance_instruction_with_hdma(
        &mut self,
        instruction_master_cycles: u32,
        mut hdma_stall: impl FnMut(CpuBusEvent, u16) -> u32,
    ) -> CpuWorkAdvance {
        debug_assert!(self.timeline.clock_master_cycles() < self.deadline.master_cycles);
        let bus = self.timeline.bus_workload();
        let dynamic_hdma = bus.dynamic_hdma();
        let fixed_hdma_stall = u32::from(bus.hdma_stall_master_cycles());
        self.timeline
            .advance_work_unbounded_with(
                instruction_master_cycles,
                |event, scanline| match event {
                    CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh) => {
                        WRAM_REFRESH_STALL_MASTER_CYCLES
                    }
                    CpuTimelineEvent::Bus(
                        event @ (CpuBusEvent::HdmaInit | CpuBusEvent::HdmaStart),
                    ) => {
                        if dynamic_hdma {
                            hdma_stall(event, scanline)
                        } else {
                            fixed_hdma_stall
                        }
                    }
                    CpuTimelineEvent::ShortScanline => 0,
                },
            );
        if self.timeline.clock_master_cycles() >= self.deadline.master_cycles {
            CpuWorkAdvance::ReachedBoundary {
                boundary: self.deadline.boundary,
                remaining_work_master_cycles: 0,
            }
        } else {
            CpuWorkAdvance::Complete
        }
    }

    /// Complete any general DMA started by the just-executed instruction.
    ///
    /// Snes9x does not accept a pending NMI between that instruction and DMA.
    /// If either crossed the H=12 acceptance deadline, DMA completes first and
    /// the pending NMI is retimed to DMA-end plus the pinned 24-master-cycle
    /// delay. Consuming the provisional instruction result here prevents an
    /// H=12 crossing from escaping as a false accepted interrupt.
    pub(super) fn advance_started_general_dma(
        &mut self,
        instruction: CpuWorkAdvance,
        dma_master_cycles: u32,
    ) -> CpuWorkAdvance {
        if dma_master_cycles == 0 {
            return instruction;
        }
        assert_eq!(
            self.deadline.boundary,
            CpuRasterBoundary::CpuNmiAcceptance,
            "general-DMA NMI deferral requires a CPU acceptance deadline",
        );
        self.timeline.advance_work_unbounded(dma_master_cycles);
        if self.timeline.clock_master_cycles() >= self.deadline.master_cycles {
            self.deadline.master_cycles = self
                .timeline
                .clock_master_cycles()
                .checked_add(SNES9X_NMI_GENERAL_DMA_DELAY_MASTER_CYCLES)
                .expect("general-DMA NMI deadline overflowed");
            return CpuWorkAdvance::Complete;
        }
        debug_assert_eq!(instruction, CpuWorkAdvance::Complete);
        CpuWorkAdvance::Complete
    }

    fn advance_atomic_work(&mut self, work_master_cycles: u32) -> CpuWorkAdvance {
        self.timeline.advance_work_unbounded(work_master_cycles);
        if self.timeline.clock_master_cycles() >= self.deadline.master_cycles {
            CpuWorkAdvance::ReachedBoundary {
                boundary: self.deadline.boundary,
                remaining_work_master_cycles: 0,
            }
        } else {
            CpuWorkAdvance::Complete
        }
    }

    /// Open the next field's CPU budget after an NMI becomes eligible for CPU
    /// acceptance.
    ///
    /// The caller must execute the real interrupt entry and handler through
    /// this same budget before resuming the interrupted instruction stream.
    /// Handler duration is deliberately not accepted here: it depends on the
    /// live NMI/DMA workload and therefore is not a constant.
    pub(super) fn begin_nmi_handler(&mut self) {
        assert_eq!(
            self.deadline.boundary,
            CpuRasterBoundary::CpuNmiAcceptance,
            "only a CPU NMI acceptance boundary can begin the NMI handler",
        );
        debug_assert!(self.timeline.clock_master_cycles() >= self.deadline.master_cycles);
        let next_field = self
            .timeline
            .field_index()
            .checked_add(1)
            .expect("CPU continuation NMI field overflowed");
        self.deadline.master_cycles = self.timeline.master_cycles_at_raster(
            next_field,
            CpuRasterBoundary::CpuNmiAcceptance.raster_position(),
        );
        debug_assert!(self.timeline.clock_master_cycles() < self.deadline.master_cycles);
    }

    /// Advance ordered semantic phases until they complete or reach the selected
    /// raster boundary. The returned phase index identifies the continuation
    /// point without coupling the scheduler to a particular game subsystem.
    pub(super) fn advance_phases(
        &mut self,
        phase_work_master_cycles: &[u32],
    ) -> CpuPhaseSequenceAdvance {
        for (phase_index, &work_master_cycles) in phase_work_master_cycles.iter().enumerate() {
            if let CpuWorkAdvance::ReachedBoundary {
                boundary,
                remaining_work_master_cycles,
            } = self.advance_interruptible(work_master_cycles)
            {
                return CpuPhaseSequenceAdvance::ReachedBoundary {
                    boundary,
                    phase_index,
                    remaining_work_master_cycles,
                };
            }
        }
        CpuPhaseSequenceAdvance::Complete
    }

    pub(super) fn raster_position(self) -> CpuRasterPosition {
        self.timeline.raster_position()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum CpuHostPhase {
    #[default]
    AwaitingHostFrame,
    MainLoopReady,
    MainLoopRunning,
    SuspendedCallStack,
    /// A caller suspended in the prior field has resumed, but the following
    /// NMI has not completed yet. CPU-authored buffers are therefore newer
    /// than the effective hardware DMA generation at this publication.
    ResumedCallStackBeforeNmi,
    /// CPU work has reached the hardware wait loop, but the following NMI has
    /// not completed yet. This covers both a resumed caller suffix and a fresh
    /// main iteration which ran after the current interval's leading NMI.
    ReturnedToMainLoopBeforeNmi,
    ReturnedToMainLoop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GameWorkStep {
    Waiting,
    Complete(GameWorkContinuation),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ScheduledGameWork {
    pub(super) continuation: GameWorkContinuation,
    nmi_slices_remaining: u8,
    entry_display_boundary_pending: bool,
    scheduled_after_leading_nmi: bool,
}

impl ScheduledGameWork {
    pub(super) fn schedule(continuation: GameWorkContinuation, nmi_slices: u8) -> Self {
        debug_assert!(nmi_slices != 0);
        Self {
            continuation,
            nmi_slices_remaining: nmi_slices,
            entry_display_boundary_pending: true,
            scheduled_after_leading_nmi: false,
        }
    }

    fn schedule_after_leading_nmi(continuation: GameWorkContinuation, nmi_slices: u8) -> Self {
        let mut work = Self::schedule(continuation, nmi_slices);
        work.scheduled_after_leading_nmi = true;
        work
    }

    pub(super) fn schedule_before_trailing_nmi(
        continuation: GameWorkContinuation,
        total_nmi_slices: u8,
    ) -> Self {
        // The translated main call will still run its trailing NMI after this
        // schedule point. Count that boundary here because future host calls
        // only advance the remaining slices.
        debug_assert!(total_nmi_slices > 1);
        Self::schedule(continuation, total_nmi_slices.saturating_sub(1))
    }

    pub(super) fn suspends_translated_call_stack(self) -> bool {
        matches!(
            self.continuation,
            GameWorkContinuation::FinishItemReceiptGraphics {
                continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                    ground_apress_tail: Some(_),
                    ..
                } | ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. },
            } | GameWorkContinuation::FinishDungeonSupertileTransition { .. }
                | GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn
                | GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn
                | GameWorkContinuation::FinishDungeonNmiPrepareSpritesCallerReturn
                | GameWorkContinuation::FinishDialogueInitializationPrefix { .. }
                | GameWorkContinuation::FinishDialogueInitializationCallerReturn
                | GameWorkContinuation::FinishGameOverSpotlightBuild { .. }
                | GameWorkContinuation::FinishDungeonSubtilePaletteFilter
                | GameWorkContinuation::FinishStraightInterroomFadeoutSuffix
                | GameWorkContinuation::FinishStraightInterroomSpriteReset
                | GameWorkContinuation::FinishSpriteMain { .. }
                | GameWorkContinuation::FinishWorldMapOverlayReload
                | GameWorkContinuation::FinishWorldMapAmbientMap8
                | GameWorkContinuation::FinishDungeonCachedSpriteMain { .. }
                | GameWorkContinuation::FinishSpiralStaircasePaletteFilter { .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightBuild { .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity { .. }
                | GameWorkContinuation::FinishOverworldSpotlightBuild { .. }
                | GameWorkContinuation::FinishOverworldSpotlightLinkOam { .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { .. }
                | GameWorkContinuation::FinishBigKeyDropGraphics { .. }
        )
    }

    pub(super) fn is_complete(self) -> bool {
        self.nmi_slices_remaining == 0
    }

    pub(super) fn in_flight_display_snapshot_publication_override(
        self,
    ) -> Option<DisplaySnapshotPublication> {
        match self.continuation {
            GameWorkContinuation::FinishSpotlightIteration { iteration }
            | GameWorkContinuation::FinishGameOverSpotlightBuild { iteration }
            | GameWorkContinuation::FinishDungeonExitSpotlightEntry { iteration, .. }
            | GameWorkContinuation::FinishDungeonExitSpotlightBuild { iteration, .. }
            | GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity { iteration, .. }
            | GameWorkContinuation::FinishOverworldSpotlightBuild { iteration, .. }
            | GameWorkContinuation::FinishOverworldSpotlightLinkOam { iteration, .. }
            | GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { iteration } => {
                Some(iteration.in_flight_publication())
            }
            GameWorkContinuation::FinishAttractWorldMapExit => {
                // The final zoom projection completed before the ROM entered
                // EraseTileMaps. Publish that staged field while the force-
                // blank/tilemap-clear generation remains staged behind it.
                Some(DisplaySnapshotPublication::AdvanceStaged)
            }
            GameWorkContinuation::FinishItemReceiptGraphics { .. }
                if !self.entry_display_boundary_pending =>
            {
                Some(DisplaySnapshotPublication::RetainPublished)
            }
            _ => None,
        }
    }

    fn spotlight_iteration(self) -> Option<SpotlightIteration> {
        match self.continuation {
            GameWorkContinuation::FinishSpotlightIteration { iteration }
            | GameWorkContinuation::FinishGameOverSpotlightBuild { iteration }
            | GameWorkContinuation::FinishDungeonExitSpotlightEntry { iteration, .. }
            | GameWorkContinuation::FinishDungeonExitSpotlightBuild { iteration, .. }
            | GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity { iteration, .. }
            | GameWorkContinuation::FinishOverworldSpotlightBuild { iteration, .. }
            | GameWorkContinuation::FinishOverworldSpotlightLinkOam { iteration, .. }
            | GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { iteration } => {
                Some(iteration)
            }
            _ => None,
        }
    }

    pub(super) fn advance_one_nmi_slice(&mut self) -> GameWorkStep {
        self.entry_display_boundary_pending = false;
        self.nmi_slices_remaining = self.nmi_slices_remaining.saturating_sub(1);
        if self.nmi_slices_remaining == 0 {
            GameWorkStep::Complete(self.continuation)
        } else {
            GameWorkStep::Waiting
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileSelectGraphicsContinuation {
    Loading { nmi_slices_remaining: u8 },
    ResumeModule,
}

impl FileSelectGraphicsContinuation {
    fn begin() -> Self {
        Self::Loading {
            nmi_slices_remaining: FILE_SELECT_GRAPHICS_NMI_SLICES,
        }
    }

    fn advance_one_nmi_slice(&mut self) -> StartupSequenceStep {
        match self {
            Self::Loading {
                nmi_slices_remaining,
            } => {
                debug_assert_ne!(*nmi_slices_remaining, 0);
                *nmi_slices_remaining = nmi_slices_remaining.saturating_sub(1);
                if *nmi_slices_remaining == 0 {
                    *self = Self::ResumeModule;
                    StartupSequenceStep::CompleteFileSelectGraphics
                } else {
                    StartupSequenceStep::FileSelectWaiting
                }
            }
            Self::ResumeModule => StartupSequenceStep::ResumeFileSelectModule,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectedGameLoadContinuation {
    BeforePreDungeonAudio { nmi_slices_remaining: u8 },
    AfterPreDungeonAudio { nmi_slices_remaining: u8 },
}

impl SelectedGameLoadContinuation {
    fn begin() -> Self {
        Self::BeforePreDungeonAudio {
            nmi_slices_remaining: SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES,
        }
    }

    fn remaining_nmi_slices(self) -> u8 {
        match self {
            Self::BeforePreDungeonAudio {
                nmi_slices_remaining,
            } => nmi_slices_remaining + SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES,
            Self::AfterPreDungeonAudio {
                nmi_slices_remaining,
            } => nmi_slices_remaining,
        }
    }

    fn advance_one_nmi_slice(&mut self) -> StartupSequenceStep {
        match self {
            Self::BeforePreDungeonAudio {
                nmi_slices_remaining,
            } => {
                debug_assert_ne!(*nmi_slices_remaining, 0);
                *nmi_slices_remaining = nmi_slices_remaining.saturating_sub(1);
                if *nmi_slices_remaining == 0 {
                    *self = Self::AfterPreDungeonAudio {
                        nmi_slices_remaining: SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES,
                    };
                    StartupSequenceStep::BeginPreDungeonAudio
                } else {
                    StartupSequenceStep::SelectedGameLoadWaiting
                }
            }
            Self::AfterPreDungeonAudio {
                nmi_slices_remaining,
            } => {
                debug_assert_ne!(*nmi_slices_remaining, 0);
                *nmi_slices_remaining = nmi_slices_remaining.saturating_sub(1);
                if *nmi_slices_remaining == 0 {
                    StartupSequenceStep::CompleteSelectedGameLoad
                } else {
                    StartupSequenceStep::SelectedGameLoadWaiting
                }
            }
        }
    }
}

/// A semantic action reached while advancing the startup call stack by one NMI.
///
/// The scheduler owns which startup continuation is active, so callers should
/// not have to unwrap a second subsystem-specific step enum before dispatching
/// the action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StartupSequenceStep {
    FileSelectWaiting,
    CompleteFileSelectGraphics,
    ResumeFileSelectModule,
    SelectedGameLoadWaiting,
    BeginPreDungeonAudio,
    CompleteSelectedGameLoad,
}

/// Host representation of the translated game thread around vblank.
///
/// Only execution ownership belongs here: suspended game work, a continuation
/// that must resume on the pre-main side of NMI, and the caller suffix that
/// returns before the next fresh module iteration. Display state remains in
/// the publication layer; continuation values describe which generation that
/// layer owns at a boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameExecutionContinuation {
    ScheduledWork(ScheduledGameWork),
    /// The caller was interrupted by the preceding host's trailing NMI and
    /// resumes at the start of this host without crossing another vblank.
    AfterCurrentTrailingNmi(GameWorkContinuation),
    /// A translated call that crosses the current host frame's trailing NMI,
    /// then returns before `retro_run` reaches the following video boundary.
    /// Keeping this phase distinct prevents one-NMI calls from being resumed
    /// a whole host frame late by `advance_work_one_nmi_slice`.
    PostTrailingNmi(GameWorkContinuation),
    FileSelectGraphics(FileSelectGraphicsContinuation),
    SelectedGameLoad(SelectedGameLoadContinuation),
    PreMainNmiResume(PreMainNmiResume),
    PreMainCaller(PreMainCallerContinuation),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct GameExecutionScheduler {
    continuation: Option<GameExecutionContinuation>,
    cpu_host_phase: CpuHostPhase,
    /// The preceding host slice reached the ordinary main-loop return. The
    /// next hardware interval therefore begins with an eligible leading NMI,
    /// even when the atomic port has not yet normalized its software latch.
    leading_nmi_follows_returned_main: bool,
    /// The preceding host reached the main wait without consuming the next
    /// NMI. The following callback must run that NMI before fresh main work.
    leading_nmi_follows_unconsumed_main_return: bool,
    /// The next translated main iteration begins after this callback's NMI.
    /// If it reaches the wait loop without scheduling more work, the callback
    /// must stop before the following NMI instead of reverting to the atomic
    /// main-then-NMI ordering.
    main_iteration_follows_leading_nmi: bool,
    /// The translated caller has returned through the leading NMI which
    /// starts a multi-state upload pipeline. This survives the one-shot
    /// continuation so later states can preserve that CPU/NMI ordering
    /// without guessing from a room or from the module state alone.
    leading_nmi_upload_pipeline_active: bool,
    /// A scheduled translated caller returned before the following NMI, but
    /// that boundary lies after the current host audio publication. The game
    /// state samples NMI-owned commands after the publication and exposes them
    /// through the next audio batch.
    audio_nmi_after_host_publication: bool,
}

impl GameExecutionScheduler {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn is_idle(self) -> bool {
        self.continuation.is_none()
    }

    pub(super) fn begin_host_frame(&mut self) {
        self.leading_nmi_follows_returned_main =
            self.cpu_host_phase == CpuHostPhase::ReturnedToMainLoop;
        self.leading_nmi_follows_unconsumed_main_return =
            self.cpu_host_phase == CpuHostPhase::ReturnedToMainLoopBeforeNmi;
        self.cpu_host_phase = if self.work_suspends_translated_call_stack() {
            CpuHostPhase::SuspendedCallStack
        } else {
            CpuHostPhase::MainLoopReady
        };
    }

    pub(super) fn begin_main_loop_iteration(&mut self) {
        debug_assert!(matches!(
            self.cpu_host_phase,
            CpuHostPhase::MainLoopReady | CpuHostPhase::SuspendedCallStack
        ));
        self.cpu_host_phase = CpuHostPhase::MainLoopRunning;
    }

    pub(super) fn finish_main_loop_iteration(&mut self) {
        debug_assert_eq!(self.cpu_host_phase, CpuHostPhase::MainLoopRunning);
        self.cpu_host_phase = if self.work_suspends_translated_call_stack() {
            CpuHostPhase::SuspendedCallStack
        } else if self.main_iteration_follows_leading_nmi {
            CpuHostPhase::ReturnedToMainLoopBeforeNmi
        } else {
            CpuHostPhase::ReturnedToMainLoop
        };
        self.main_iteration_follows_leading_nmi = false;
    }

    pub(super) fn mark_main_iteration_after_leading_nmi(&mut self) {
        debug_assert!(matches!(
            self.cpu_host_phase,
            CpuHostPhase::MainLoopReady | CpuHostPhase::ResumedCallStackBeforeNmi
        ));
        self.main_iteration_follows_leading_nmi = true;
    }

    /// True while the current main iteration is running on the CPU side of an
    /// NMI already consumed by this host callback. An ordinary iteration has
    /// its trailing NMI still ahead of it and can spend the first slice of a
    /// newly suspended workload on that boundary.
    pub(super) fn current_main_iteration_follows_leading_nmi(self) -> bool {
        self.cpu_host_phase == CpuHostPhase::MainLoopRunning
            && self.main_iteration_follows_leading_nmi
    }

    #[cfg(test)]
    pub(super) fn fresh_main_loop_iteration_is_ready(self) -> bool {
        self.cpu_host_phase == CpuHostPhase::MainLoopReady
    }

    pub(super) fn eligible_leading_nmi_preceded_suspended_work(self) -> bool {
        self.leading_nmi_follows_returned_main && self.work_suspends_translated_call_stack()
    }

    pub(super) fn resumed_call_stack_is_before_nmi(self) -> bool {
        self.cpu_host_phase == CpuHostPhase::ResumedCallStackBeforeNmi
    }

    /// The current main-loop call stack reached an interruptible boundary and
    /// is waiting for the NMI that suspended it. DMA at that boundary consumes
    /// CPU-authored operands from the completed prefix, not the host-entry
    /// snapshot retained for display publication.
    pub(super) fn main_call_stack_is_suspended_before_nmi(self) -> bool {
        self.cpu_host_phase == CpuHostPhase::SuspendedCallStack
    }

    /// Record that a translated caller suffix reached the main wait after this
    /// host's NMI. Scheduled work enters through `ResumedCallStackBeforeNmi`;
    /// continuations represented by another native state machine enter while
    /// the scheduler is otherwise `MainLoopReady`.
    pub(super) fn finish_call_stack_at_main_wait_before_nmi(&mut self) {
        debug_assert!(matches!(
            self.cpu_host_phase,
            CpuHostPhase::MainLoopReady | CpuHostPhase::ResumedCallStackBeforeNmi
        ));
        debug_assert!(!self.work_suspends_translated_call_stack());
        self.cpu_host_phase = CpuHostPhase::ReturnedToMainLoopBeforeNmi;
    }

    pub(super) fn main_return_requires_leading_nmi(self) -> bool {
        self.leading_nmi_follows_unconsumed_main_return
            && self.cpu_host_phase == CpuHostPhase::MainLoopReady
            && !self.work_suspends_translated_call_stack()
    }

    pub(super) fn returned_main_is_waiting_for_nmi(self) -> bool {
        self.cpu_host_phase == CpuHostPhase::ReturnedToMainLoopBeforeNmi
    }

    /// Record that the NMI following a completed atomic main iteration ran in
    /// this host callback. A main iteration which returns after a leading NMI
    /// deliberately does not call this method, so the next callback preserves
    /// the same leading-NMI cadence without module-specific inference.
    pub(super) fn finish_trailing_nmi_after_main_return(&mut self) {
        if self.cpu_host_phase == CpuHostPhase::ReturnedToMainLoop {
            self.cpu_host_phase = CpuHostPhase::AwaitingHostFrame;
        }
    }

    fn schedule_continuation(&mut self, continuation: GameExecutionContinuation) {
        assert!(
            self.continuation.is_none(),
            "cannot schedule {continuation:?} while {:?} is pending",
            self.continuation
        );
        self.continuation = Some(continuation);
    }

    pub(super) fn schedule_work(&mut self, continuation: GameWorkContinuation, nmi_slices: u8) {
        self.schedule_continuation(GameExecutionContinuation::ScheduledWork(
            ScheduledGameWork::schedule(continuation, nmi_slices),
        ));
    }

    /// Schedule measured CPU work whose entry occurs after the active field
    /// has already begun scanning out. Every NMI crossing belongs to a future
    /// field, so the entry snapshot cannot accept the first crossing's
    /// register receipt even though the translated call stack is suspended.
    pub(super) fn schedule_cpu_timed_work_after_active_field_started(
        &mut self,
        continuation: GameWorkContinuation,
        total_nmi_crossings: u8,
    ) {
        assert_eq!(self.cpu_host_phase, CpuHostPhase::MainLoopRunning);
        assert_ne!(total_nmi_crossings, 0);
        self.schedule_continuation(GameExecutionContinuation::ScheduledWork(
            ScheduledGameWork::schedule_after_leading_nmi(continuation, total_nmi_crossings),
        ));
    }

    pub(super) fn schedule_work_before_trailing_nmi(
        &mut self,
        continuation: GameWorkContinuation,
        total_nmi_slices: u8,
    ) {
        self.schedule_continuation(GameExecutionContinuation::ScheduledWork(
            ScheduledGameWork::schedule_before_trailing_nmi(continuation, total_nmi_slices),
        ));
    }

    /// Schedule CPU work measured as crossings from a call stack that is
    /// currently executing before this host frame's trailing NMI.
    pub(super) fn schedule_cpu_timed_work_before_trailing_nmi(
        &mut self,
        continuation: GameWorkContinuation,
        total_nmi_crossings: u8,
    ) {
        assert!(
            matches!(
                self.cpu_host_phase,
                CpuHostPhase::MainLoopRunning | CpuHostPhase::ResumedCallStackBeforeNmi
            ),
            "CPU-timed pre-NMI work scheduled from {:?}",
            self.cpu_host_phase,
        );
        assert_ne!(total_nmi_crossings, 0);
        if total_nmi_crossings == 1 {
            self.schedule_post_trailing_nmi(continuation);
        } else {
            self.schedule_work_before_trailing_nmi(continuation, total_nmi_crossings);
        }
    }

    /// Schedule a synchronous C call from the current main-loop iteration.
    ///
    /// An ordinary atomic iteration still owns its trailing NMI, so that
    /// boundary consumes the first measured crossing. A main iteration which
    /// began after a leading NMI has no boundary left in the current host
    /// callback; all measured crossings remain future scheduled work.
    pub(super) fn schedule_cpu_timed_work_from_current_main_iteration(
        &mut self,
        continuation: GameWorkContinuation,
        total_nmi_crossings: u8,
    ) {
        assert_eq!(self.cpu_host_phase, CpuHostPhase::MainLoopRunning);
        assert_ne!(total_nmi_crossings, 0);
        if self.current_main_iteration_follows_leading_nmi() {
            self.schedule_continuation(GameExecutionContinuation::ScheduledWork(
                ScheduledGameWork::schedule_after_leading_nmi(continuation, total_nmi_crossings),
            ));
        } else {
            self.schedule_cpu_timed_work_before_trailing_nmi(continuation, total_nmi_crossings);
        }
    }

    /// Schedule a translated call whose interrupting trailing NMI ends the
    /// current frontend callback. The C stack resumes on a later host call,
    /// even when the measured work crosses only that one NMI.
    pub(super) fn schedule_cpu_timed_work_returning_on_later_host(
        &mut self,
        continuation: GameWorkContinuation,
        total_nmi_crossings: u8,
    ) {
        assert!(
            matches!(
                self.cpu_host_phase,
                CpuHostPhase::MainLoopRunning | CpuHostPhase::ResumedCallStackBeforeNmi
            ),
            "CPU-timed later-host work scheduled from {:?}",
            self.cpu_host_phase,
        );
        assert_ne!(total_nmi_crossings, 0);
        self.schedule_work(continuation, total_nmi_crossings);
    }

    pub(super) fn schedule_post_trailing_nmi(&mut self, continuation: GameWorkContinuation) {
        self.schedule_continuation(GameExecutionContinuation::PostTrailingNmi(continuation));
    }

    pub(super) fn schedule_after_current_trailing_nmi(
        &mut self,
        continuation: GameWorkContinuation,
    ) {
        self.schedule_continuation(GameExecutionContinuation::AfterCurrentTrailingNmi(
            continuation,
        ));
    }

    /// Schedule a saved CPU stack whose first measured crossing is this
    /// callback's trailing NMI and whose return must occur on a later host.
    /// A one-crossing caller resumes directly on the next callback; longer
    /// callers retain only the crossings which remain after this boundary.
    pub(super) fn schedule_cpu_timed_work_resuming_after_current_trailing_nmi(
        &mut self,
        continuation: GameWorkContinuation,
        total_nmi_crossings: u8,
    ) {
        assert_ne!(total_nmi_crossings, 0);
        if total_nmi_crossings == 1 {
            self.schedule_after_current_trailing_nmi(continuation);
        } else {
            self.schedule_work_before_trailing_nmi(continuation, total_nmi_crossings);
        }
    }

    pub(super) fn schedule_file_select_graphics(&mut self) {
        self.schedule_continuation(GameExecutionContinuation::FileSelectGraphics(
            FileSelectGraphicsContinuation::begin(),
        ));
    }

    pub(super) fn schedule_selected_game_load(&mut self) {
        self.schedule_continuation(GameExecutionContinuation::SelectedGameLoad(
            SelectedGameLoadContinuation::begin(),
        ));
    }

    pub(super) fn advance_startup_sequence(&mut self) -> Option<StartupSequenceStep> {
        let step = match self.continuation.as_mut()? {
            GameExecutionContinuation::FileSelectGraphics(continuation) => {
                continuation.advance_one_nmi_slice()
            }
            GameExecutionContinuation::SelectedGameLoad(continuation) => {
                continuation.advance_one_nmi_slice()
            }
            _ => return None,
        };
        if matches!(
            step,
            StartupSequenceStep::ResumeFileSelectModule
                | StartupSequenceStep::CompleteSelectedGameLoad
        ) {
            self.continuation = None;
        }
        Some(step)
    }

    pub(super) fn selected_game_load_remaining_nmi_slices(self) -> u8 {
        match self.continuation {
            Some(GameExecutionContinuation::SelectedGameLoad(continuation)) => {
                continuation.remaining_nmi_slices()
            }
            _ => 0,
        }
    }

    fn scheduled_work(self) -> Option<ScheduledGameWork> {
        match self.continuation {
            Some(GameExecutionContinuation::ScheduledWork(work)) => Some(work),
            _ => None,
        }
    }

    pub(super) fn work_is_pending(self) -> bool {
        self.scheduled_work().is_some()
            || matches!(
                self.continuation,
                Some(
                    GameExecutionContinuation::AfterCurrentTrailingNmi(_)
                        | GameExecutionContinuation::PostTrailingNmi(_)
                )
            )
    }

    pub(super) fn work_suspends_translated_call_stack(self) -> bool {
        self.scheduled_work()
            .is_some_and(ScheduledGameWork::suspends_translated_call_stack)
            || matches!(
                self.continuation,
                Some(
                    GameExecutionContinuation::AfterCurrentTrailingNmi(_)
                        | GameExecutionContinuation::PostTrailingNmi(_)
                ) | Some(GameExecutionContinuation::PreMainCaller(
                    PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass { .. }
                        | PreMainCallerContinuation::SpiralStairsSecondPaletteFilter
                        | PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter
                ))
            )
    }

    /// Remaining NMI slices of the currently scheduled work, if any. Lets
    /// measured mid-window side effects fire at their exact held boundary.
    pub(super) fn scheduled_work_slices_remaining(&self) -> Option<u8> {
        self.scheduled_work().map(|work| work.nmi_slices_remaining)
    }

    pub(super) fn mark_audio_nmi_after_host_publication(&mut self) {
        debug_assert!(!self.audio_nmi_after_host_publication);
        self.audio_nmi_after_host_publication = true;
    }

    pub(super) fn take_audio_nmi_after_host_publication(&mut self) -> bool {
        std::mem::take(&mut self.audio_nmi_after_host_publication)
    }

    /// True while a newly scheduled synchronous call is executing after the
    /// leading NMI which already published this host's active field. Advancing
    /// the first real NMI slice clears the entry boundary before publication
    /// code observes this work again.
    pub(super) fn active_field_precedes_current_scheduled_work(self) -> bool {
        self.scheduled_work().is_some_and(|work| {
            work.scheduled_after_leading_nmi && work.entry_display_boundary_pending
        })
    }

    pub(super) fn current_scheduled_work_started_after_leading_nmi(self) -> bool {
        self.scheduled_work()
            .is_some_and(|work| work.scheduled_after_leading_nmi)
    }

    pub(super) fn current_scheduled_work_is_at_entry_boundary(self) -> bool {
        self.scheduled_work()
            .is_some_and(|work| work.entry_display_boundary_pending)
    }

    pub(super) fn current_work(self) -> Option<GameWorkContinuation> {
        self.scheduled_work()
            .map(|work| work.continuation)
            .or_else(|| match self.continuation {
                Some(
                    GameExecutionContinuation::AfterCurrentTrailingNmi(continuation)
                    | GameExecutionContinuation::PostTrailingNmi(continuation),
                ) => Some(continuation),
                _ => None,
            })
    }

    pub(super) fn take_post_trailing_nmi(&mut self) -> Option<GameWorkContinuation> {
        match self.continuation {
            Some(GameExecutionContinuation::PostTrailingNmi(continuation)) => {
                self.continuation = None;
                if self.cpu_host_phase == CpuHostPhase::SuspendedCallStack {
                    self.cpu_host_phase = CpuHostPhase::ResumedCallStackBeforeNmi;
                }
                Some(continuation)
            }
            _ => None,
        }
    }

    pub(super) fn take_after_current_trailing_nmi(&mut self) -> Option<GameWorkContinuation> {
        match self.continuation {
            Some(GameExecutionContinuation::AfterCurrentTrailingNmi(continuation)) => {
                self.continuation = None;
                if self.cpu_host_phase == CpuHostPhase::SuspendedCallStack {
                    self.cpu_host_phase = CpuHostPhase::ResumedCallStackBeforeNmi;
                }
                Some(continuation)
            }
            _ => None,
        }
    }

    pub(super) fn advance_work_one_nmi_slice(&mut self) -> Option<GameWorkStep> {
        let Some(GameExecutionContinuation::ScheduledWork(work)) = self.continuation.as_mut()
        else {
            return None;
        };
        let suspends_translated_call_stack = work.suspends_translated_call_stack();
        let step = work.advance_one_nmi_slice();
        if matches!(step, GameWorkStep::Complete(_)) {
            self.continuation = None;
            if suspends_translated_call_stack
                && self.cpu_host_phase == CpuHostPhase::SuspendedCallStack
            {
                self.cpu_host_phase = CpuHostPhase::ResumedCallStackBeforeNmi;
            }
        }
        Some(step)
    }

    pub(super) fn finish_work(&mut self) {
        if matches!(
            self.continuation,
            Some(
                GameExecutionContinuation::ScheduledWork(_)
                    | GameExecutionContinuation::AfterCurrentTrailingNmi(_)
                    | GameExecutionContinuation::PostTrailingNmi(_)
            )
        ) {
            self.continuation = None;
        }
    }

    pub(super) fn in_flight_display_publication(self) -> Option<DisplaySnapshotPublication> {
        self.scheduled_work()
            .and_then(ScheduledGameWork::in_flight_display_snapshot_publication_override)
    }

    pub(super) fn spotlight_iteration(self) -> Option<SpotlightIteration> {
        self.scheduled_work()
            .and_then(ScheduledGameWork::spotlight_iteration)
    }

    pub(super) fn schedule_pre_main_nmi_resume(&mut self, continuation: PreMainNmiResume) {
        self.schedule_continuation(GameExecutionContinuation::PreMainNmiResume(continuation));
    }

    pub(super) fn take_pre_main_nmi_resume(&mut self) -> Option<PreMainNmiResume> {
        match self.continuation {
            Some(GameExecutionContinuation::PreMainNmiResume(continuation)) => {
                self.continuation = None;
                Some(continuation)
            }
            _ => None,
        }
    }

    pub(super) fn begin_leading_nmi_upload_pipeline(&mut self) {
        self.leading_nmi_upload_pipeline_active = true;
    }

    pub(super) fn leading_nmi_upload_pipeline_is_active(self) -> bool {
        self.leading_nmi_upload_pipeline_active
    }

    pub(super) fn finish_leading_nmi_upload_pipeline(&mut self) {
        self.leading_nmi_upload_pipeline_active = false;
    }

    pub(super) fn pre_main_nmi_resume(self) -> Option<PreMainNmiResume> {
        match self.continuation {
            Some(GameExecutionContinuation::PreMainNmiResume(continuation)) => Some(continuation),
            _ => None,
        }
    }

    pub(super) fn schedule_pre_main_caller_continuation(
        &mut self,
        continuation: PreMainCallerContinuation,
    ) {
        self.schedule_continuation(GameExecutionContinuation::PreMainCaller(continuation));
    }

    pub(super) fn pre_main_caller_continuation(self) -> Option<PreMainCallerContinuation> {
        match self.continuation {
            Some(GameExecutionContinuation::PreMainCaller(continuation)) => Some(continuation),
            _ => None,
        }
    }

    pub(super) fn pre_main_caller_continuation_is(
        self,
        continuation: PreMainCallerContinuation,
    ) -> bool {
        self.pre_main_caller_continuation() == Some(continuation)
    }

    pub(super) fn finish_pre_main_caller_continuation(
        &mut self,
        expected: PreMainCallerContinuation,
    ) {
        let Some(actual) = self.pre_main_caller_continuation() else {
            // Untimed execution completes these calls atomically and therefore
            // has no scheduled caller continuation to consume.
            return;
        };
        assert_eq!(
            actual, expected,
            "pre-main caller completed through the wrong game-thread return path"
        );
        self.continuation = None;
        if self.cpu_host_phase == CpuHostPhase::SuspendedCallStack {
            self.cpu_host_phase = CpuHostPhase::ResumedCallStackBeforeNmi;
        }
    }
}

#[cfg(test)]
mod cpu_timing_tests {
    use super::*;
    use crate::zelda_rtl::{SpriteMainCpuBoundary, SpriteMainCpuCaller};
    use snes::{
        HDMA_START_CYCLE, MASTER_CYCLES_PER_SCANLINE, NTSC_FIELD_MASTER_CYCLES, WRAM_REFRESH_CYCLE,
    };

    const DUNGEON_HDMA_STALL: u16 = 42;
    const LONG_TIMELINE_FIELD: u64 = 24_001;
    const WORK_TO_CACHED_RESTORE: u32 = 1_400 + 4 * 10_674 + 8_884;

    fn budget_at_field(
        field_index: u64,
        entry: CpuRasterPosition,
        bus: CpuBusWorkload,
        field_timing: CpuFieldTiming,
    ) -> CpuCycleBudget {
        let entry_master_cycles = field_timing.master_cycles_at(field_index, entry);
        let boundary_master_cycles = field_timing.master_cycles_at(
            field_index,
            CpuRasterBoundary::VblankPublication.raster_position(),
        );
        let boundary_field = field_index + u64::from(entry_master_cycles >= boundary_master_cycles);
        CpuCycleBudget {
            timeline: CpuMasterTimeline::at_raster(field_index, entry, bus, field_timing),
            deadline: CpuBoundaryDeadline {
                boundary: CpuRasterBoundary::VblankPublication,
                master_cycles: field_timing.master_cycles_at(
                    boundary_field,
                    CpuRasterBoundary::VblankPublication.raster_position(),
                ),
            },
        }
    }

    fn restored_fields_before_vblank(entry: CpuRasterPosition) -> usize {
        let mut budget = CpuCycleBudget::until_next_vblank_publication(
            entry,
            CpuBusWorkload::with_hdma_stall(DUNGEON_HDMA_STALL),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            budget.advance_interruptible(WORK_TO_CACHED_RESTORE),
            CpuWorkAdvance::Complete
        );
        let mut restored = 0;
        for field in 0..24 {
            if budget.advance_instruction(28).reached_boundary().is_some() {
                break;
            }
            let advance = budget.advance_instruction(if field == 1 { 40 } else { 38 });
            restored += 1;
            if advance.reached_boundary().is_some() {
                break;
            }
        }
        restored
    }

    #[test]
    fn cached_restore_position_follows_cpu_work_and_bus_stalls() {
        for (entry, restore) in [
            (
                CpuRasterPosition::new(182, 1_266),
                CpuRasterPosition::new(224, 320),
            ),
            (
                CpuRasterPosition::new(183, 44),
                CpuRasterPosition::new(224, 462),
            ),
            (
                CpuRasterPosition::new(183, 124),
                CpuRasterPosition::new(224, 582),
            ),
        ] {
            let mut budget = CpuCycleBudget::until_next_vblank_publication(
                entry,
                CpuBusWorkload::with_hdma_stall(DUNGEON_HDMA_STALL),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            assert_eq!(
                budget.advance_interruptible(WORK_TO_CACHED_RESTORE),
                CpuWorkAdvance::Complete
            );
            assert_eq!(budget.raster_position(), restore);
        }
    }

    #[test]
    fn cached_restore_cut_is_derived_from_remaining_cycles() {
        assert_eq!(
            restored_fields_before_vblank(CpuRasterPosition::new(182, 1_266)),
            15
        );
        assert_eq!(
            restored_fields_before_vblank(CpuRasterPosition::new(183, 44)),
            12
        );
        assert_eq!(
            restored_fields_before_vblank(CpuRasterPosition::new(183, 124)),
            11
        );
    }

    #[test]
    fn publication_boundary_retains_the_remaining_cycle_budget() {
        let mut budget = CpuCycleBudget::until_next_vblank_publication(
            CpuRasterPosition::new(224, 1_300),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            budget.advance_interruptible(100),
            CpuWorkAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::VblankPublication,
                remaining_work_master_cycles: 36,
            }
        );
    }

    #[test]
    fn nmi_acceptance_is_twelve_master_cycles_after_vblank_publication() {
        // Pinned Snes9x 1.63 publishes VBlank at V=225,H=0 and schedules an
        // enabled NMI for H=12. Work can cross publication without claiming a
        // CPU interrupt boundary.
        let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
            CpuRasterPosition::new(224, 1_358),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(budget.advance_interruptible(12), CpuWorkAdvance::Complete);
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 6));
        assert_eq!(
            budget.advance_interruptible(6),
            CpuWorkAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::CpuNmiAcceptance,
                remaining_work_master_cycles: 0,
            },
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 12));
    }

    #[test]
    fn instruction_spanning_h12_completes_before_reporting_nmi_acceptance() {
        let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
            CpuRasterPosition::new(224, 1_358),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            budget.advance_instruction(20),
            CpuWorkAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::CpuNmiAcceptance,
                remaining_work_master_cycles: 0,
            },
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 14));
    }

    #[test]
    fn general_dma_crossing_h12_defers_nmi_until_dma_end_plus_24() {
        // Snes9x dma.cpp retimes a pending NMI at DMA completion; memmap.cpp
        // fixes NMIDMADelay at 24 master cycles for this model.
        let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
            CpuRasterPosition::new(224, 1_358),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let instruction = budget.advance_instruction(12);
        assert_eq!(instruction, CpuWorkAdvance::Complete);
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 6));

        assert_eq!(
            budget.advance_started_general_dma(instruction, 20),
            CpuWorkAdvance::Complete,
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 26));
        assert_eq!(
            budget.deadline.master_cycles % NTSC_FIELD_MASTER_CYCLES,
            u64::from(NMI_SCANLINE * MASTER_CYCLES_PER_SCANLINE + 50),
        );

        assert_eq!(budget.advance_instruction(20), CpuWorkAdvance::Complete);
        assert_eq!(
            budget.advance_instruction(8),
            CpuWorkAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::CpuNmiAcceptance,
                remaining_work_master_cycles: 0,
            },
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 54));
    }

    #[test]
    fn instruction_triggered_dma_suppresses_its_provisional_h12_boundary() {
        let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
            CpuRasterPosition::new(224, 1_358),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let instruction = budget.advance_instruction(20);
        assert_eq!(
            instruction,
            CpuWorkAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::CpuNmiAcceptance,
                remaining_work_master_cycles: 0,
            },
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 14));

        assert_eq!(
            budget.advance_started_general_dma(instruction, 40),
            CpuWorkAdvance::Complete,
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 54));
        assert_eq!(
            budget.deadline.master_cycles % NTSC_FIELD_MASTER_CYCLES,
            u64::from(NMI_SCANLINE * MASTER_CYCLES_PER_SCANLINE + 78),
        );

        assert_eq!(budget.advance_instruction(20), CpuWorkAdvance::Complete);
        assert_eq!(
            budget.advance_instruction(8),
            CpuWorkAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::CpuNmiAcceptance,
                remaining_work_master_cycles: 0,
            },
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 82));
        budget.begin_nmi_handler();
        assert_eq!(
            budget.deadline.master_cycles,
            CpuFieldTiming::NON_INTERLACE_EVEN
                .master_cycles_at(1, CpuRasterBoundary::CpuNmiAcceptance.raster_position(),),
        );
    }

    #[test]
    fn vblank_publication_budget_still_stops_at_h0() {
        let mut budget = CpuCycleBudget::until_next_vblank_publication(
            CpuRasterPosition::new(224, 1_358),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            budget.advance_interruptible(6),
            CpuWorkAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::VblankPublication,
                remaining_work_master_cycles: 0,
            },
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 0));
    }

    #[test]
    fn instruction_stream_resumes_after_nmi_at_next_fields_h12() {
        let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
            CpuRasterPosition::new(224, 1_350),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            budget.advance_instruction(28),
            CpuWorkAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::CpuNmiAcceptance,
                remaining_work_master_cycles: 0,
            }
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 14),);

        budget.begin_nmi_handler();
        assert_eq!(
            budget.deadline.master_cycles,
            CpuFieldTiming::NON_INTERLACE_EVEN
                .master_cycles_at(1, CpuRasterBoundary::CpuNmiAcceptance.raster_position(),),
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 14),);
        assert_eq!(
            budget.advance_interruptible(2_984),
            CpuWorkAdvance::Complete,
        );

        assert_eq!(budget.raster_position(), CpuRasterPosition::new(227, 350),);
        assert_eq!(budget.advance_instruction(24), CpuWorkAdvance::Complete,);
    }

    #[test]
    fn bus_stall_at_an_instruction_boundary_precedes_the_next_instruction() {
        for (event_cycle, stall) in [
            (WRAM_REFRESH_CYCLE, WRAM_REFRESH_STALL_MASTER_CYCLES),
            (HDMA_START_CYCLE, u32::from(DUNGEON_HDMA_STALL)),
        ] {
            let mut budget = CpuCycleBudget::until_next_vblank_publication(
                CpuRasterPosition::new(100, (event_cycle - 6) as u16),
                CpuBusWorkload::with_hdma_stall(DUNGEON_HDMA_STALL),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            assert_eq!(budget.advance_instruction(6), CpuWorkAdvance::Complete);
            assert_eq!(
                budget.raster_position(),
                CpuRasterPosition::new(100, event_cycle as u16),
            );

            assert_eq!(budget.advance_instruction(6), CpuWorkAdvance::Complete);
            assert_eq!(
                budget.raster_position(),
                CpuRasterPosition::new(100, (event_cycle + stall + 6) as u16),
            );
        }
    }

    #[test]
    fn m1_wram_refresh_starts_at_530_on_short_and_long_timelines() {
        // Pinned Snes9x 1.63 source authority:
        //   globals.cpp: M1SNES = { 1, 3, 2 }, Model = &M1SNES
        //   cpu.cpp: _5A22 != 2 selects SNES_WRAM_REFRESH_HC_v1
        //   snes9x.h: v1 = 530, refresh duration = 40 master cycles
        let entry = CpuRasterPosition::new(100, 524);
        let short_timeline = CpuCycleBudget::until_next_vblank_publication(
            entry,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let long_timeline = budget_at_field(
            LONG_TIMELINE_FIELD,
            entry,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );

        for mut budget in [short_timeline, long_timeline] {
            assert_eq!(budget.advance_instruction(6), CpuWorkAdvance::Complete);
            assert_eq!(budget.raster_position(), CpuRasterPosition::new(100, 530),);

            assert_eq!(budget.advance_instruction(6), CpuWorkAdvance::Complete);
            assert_eq!(budget.raster_position(), CpuRasterPosition::new(100, 576),);
        }
    }

    #[test]
    fn fixed_hdma_budget_does_not_invoke_the_dynamic_dma_model() {
        let mut budget = CpuCycleBudget::until_next_vblank_publication(
            CpuRasterPosition::new(100, 1_100),
            CpuBusWorkload::with_hdma_stall(DUNGEON_HDMA_STALL),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );

        assert_eq!(
            budget.advance_instruction_with_hdma(12, |_, _| {
                panic!("fixed HDMA workload invoked the dynamic DMA model")
            }),
            CpuWorkAdvance::Complete,
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(100, 1_154),);
    }

    #[test]
    fn phase_sequence_reports_the_interrupted_continuation_phase() {
        let mut budget = CpuCycleBudget::until_next_vblank_publication(
            CpuRasterPosition::new(224, 1_300),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            budget.advance_phases(&[32, 68]),
            CpuPhaseSequenceAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::VblankPublication,
                phase_index: 1,
                remaining_work_master_cycles: 36,
            }
        );
    }

    #[test]
    fn odd_noninterlace_field_skips_the_missing_dot_on_scanline_240() {
        let entry = CpuRasterPosition::new(240, 1_350);
        let mut even = CpuCycleBudget::until_next_vblank_publication(
            entry,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut odd = CpuCycleBudget::until_next_vblank_publication(
            entry,
            CpuBusWorkload::default(),
            CpuFieldTiming::non_interlace(true),
        );

        let even_clock = even.timeline.clock_master_cycles();
        let odd_clock = odd.timeline.clock_master_cycles();
        assert_eq!(even.advance_interruptible(20), CpuWorkAdvance::Complete);
        assert_eq!(odd.advance_interruptible(20), CpuWorkAdvance::Complete);
        assert_eq!(even.timeline.clock_master_cycles() - even_clock, 20);
        assert_eq!(odd.timeline.clock_master_cycles() - odd_clock, 20);
        assert_eq!(even.raster_position(), CpuRasterPosition::new(241, 6),);
        assert_eq!(odd.raster_position(), CpuRasterPosition::new(241, 10));
    }

    #[test]
    fn nmi_acceptance_deadlines_remain_exact_beyond_twenty_four_thousand_fields() {
        let mut budget = CpuCycleBudget::at_nmi_acceptance(
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let initial_nmi_master_cycles = budget.deadline.master_cycles;

        for _ in 0..LONG_TIMELINE_FIELD {
            budget.timeline = CpuMasterTimeline::new(
                budget.deadline.master_cycles,
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            budget.begin_nmi_handler();
        }

        assert_eq!(
            budget.deadline.master_cycles,
            initial_nmi_master_cycles
                + CpuFieldTiming::NON_INTERLACE_EVEN.field_start_master_cycles(LONG_TIMELINE_FIELD),
        );
        assert!(budget.deadline.master_cycles > u64::from(u32::MAX));
        assert_eq!(
            budget.deadline.master_cycles - budget.timeline.clock_master_cycles(),
            CpuFieldTiming::NON_INTERLACE_EVEN.field_master_cycles(LONG_TIMELINE_FIELD - 1),
        );
        assert_eq!(
            budget.raster_position().coordinates(),
            (NMI_SCANLINE as u16, 12),
        );

        let late_raster = budget_at_field(
            LONG_TIMELINE_FIELD,
            CpuRasterPosition::new(259, 8),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(late_raster.raster_position().coordinates(), (259, 8));
    }

    #[test]
    fn odd_field_short_scanline_math_remains_exact_on_a_long_timeline() {
        let entry = CpuRasterPosition::new(240, 1_350);
        let mut even = budget_at_field(
            LONG_TIMELINE_FIELD - 1,
            entry,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut odd = budget_at_field(
            LONG_TIMELINE_FIELD,
            entry,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );

        let even_clock = even.timeline.clock_master_cycles();
        let odd_clock = odd.timeline.clock_master_cycles();
        assert_eq!(even.advance_interruptible(20), CpuWorkAdvance::Complete);
        assert_eq!(odd.advance_interruptible(20), CpuWorkAdvance::Complete);
        assert_eq!(even.timeline.clock_master_cycles() - even_clock, 20);
        assert_eq!(odd.timeline.clock_master_cycles() - odd_clock, 20);
        assert_eq!(even.raster_position(), CpuRasterPosition::new(241, 6));
        assert_eq!(odd.raster_position(), CpuRasterPosition::new(241, 10));
    }

    #[test]
    fn dynamic_bus_events_remain_exact_on_a_long_timeline() {
        let mut hdma_init = budget_at_field(
            LONG_TIMELINE_FIELD,
            CpuRasterPosition::new(0, 14),
            CpuBusWorkload::with_dynamic_hdma(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut init_events = Vec::new();
        assert_eq!(
            hdma_init.advance_instruction_with_hdma(12, |event, scanline| {
                init_events.push((event, scanline));
                18
            }),
            CpuWorkAdvance::Complete,
        );
        assert_eq!(init_events, [(CpuBusEvent::HdmaInit, 0)]);
        assert_eq!(hdma_init.raster_position(), CpuRasterPosition::new(0, 44));

        let mut refresh = budget_at_field(
            LONG_TIMELINE_FIELD,
            CpuRasterPosition::new(100, 524),
            CpuBusWorkload::with_dynamic_hdma(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            refresh.advance_instruction_with_hdma(12, |_, _| {
                panic!("WRAM refresh invoked the HDMA model")
            }),
            CpuWorkAdvance::Complete,
        );
        assert_eq!(refresh.raster_position(), CpuRasterPosition::new(100, 576));

        let mut hdma_start = budget_at_field(
            LONG_TIMELINE_FIELD,
            CpuRasterPosition::new(100, 1_100),
            CpuBusWorkload::with_dynamic_hdma(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut start_events = Vec::new();
        assert_eq!(
            hdma_start.advance_instruction_with_hdma(12, |event, scanline| {
                start_events.push((event, scanline));
                26
            }),
            CpuWorkAdvance::Complete,
        );
        assert_eq!(start_events, [(CpuBusEvent::HdmaStart, 100)]);
        assert_eq!(
            hdma_start.raster_position(),
            CpuRasterPosition::new(100, 1_138),
        );
    }

    #[test]
    fn returned_main_loop_waits_for_the_next_host_frame() {
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.begin_host_frame();
        assert!(scheduler.fresh_main_loop_iteration_is_ready());
        assert!(!scheduler.eligible_leading_nmi_preceded_suspended_work());
        scheduler.begin_main_loop_iteration();
        scheduler.finish_main_loop_iteration();
        assert!(!scheduler.fresh_main_loop_iteration_is_ready());

        scheduler.begin_host_frame();
        assert!(scheduler.fresh_main_loop_iteration_is_ready());
        assert!(!scheduler.eligible_leading_nmi_preceded_suspended_work());
    }

    #[test]
    fn returned_main_records_the_following_leading_nmi_before_suspended_work() {
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.begin_host_frame();
        scheduler.begin_main_loop_iteration();
        scheduler.finish_main_loop_iteration();

        scheduler.begin_host_frame();
        scheduler.begin_main_loop_iteration();
        scheduler.schedule_pre_main_caller_continuation(
            PreMainCallerContinuation::SpiralStairsSecondPaletteFilter,
        );
        scheduler.finish_main_loop_iteration();

        assert!(scheduler.eligible_leading_nmi_preceded_suspended_work());
    }

    #[test]
    fn dungeon_exit_spotlight_entry_suspends_the_translated_call_stack() {
        let work = ScheduledGameWork::schedule(
            GameWorkContinuation::FinishDungeonExitSpotlightEntry {
                table_build: crate::zelda_rtl::SpotlightTableBuildContinuation::default(),
                iteration: crate::zelda_rtl::SpotlightIteration::closing(
                    crate::zelda_rtl::SpotlightIterationPhase::CloseEntryBeforeTablePublication,
                ),
            },
            1,
        );
        assert!(work.suspends_translated_call_stack());
    }

    #[test]
    fn resumed_caller_suffix_remains_before_the_following_nmi() {
        let mut scheduler = GameExecutionScheduler::default();
        let continuation = PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter;
        scheduler.schedule_pre_main_caller_continuation(continuation);
        scheduler.begin_host_frame();
        assert!(!scheduler.resumed_call_stack_is_before_nmi());

        scheduler.finish_pre_main_caller_continuation(continuation);

        assert!(scheduler.resumed_call_stack_is_before_nmi());
    }

    #[test]
    fn completed_scheduled_stack_can_continue_through_the_current_trailing_nmi() {
        let continuation = GameWorkContinuation::FinishDungeonSubtilePaletteFilter;
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.schedule_work(continuation, 1);
        scheduler.begin_host_frame();

        assert_eq!(
            scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(continuation))
        );
        assert!(scheduler.resumed_call_stack_is_before_nmi());

        scheduler.schedule_cpu_timed_work_before_trailing_nmi(continuation, 1);
        assert_eq!(scheduler.take_post_trailing_nmi(), Some(continuation));
    }

    #[test]
    fn post_trailing_nmi_resumes_the_suspended_caller_before_the_next_nmi() {
        let continuation = GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn;
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.begin_host_frame();
        scheduler.begin_main_loop_iteration();
        scheduler.schedule_cpu_timed_work_before_trailing_nmi(continuation, 1);
        scheduler.finish_main_loop_iteration();

        assert!(scheduler.main_call_stack_is_suspended_before_nmi());
        assert_eq!(scheduler.take_post_trailing_nmi(), Some(continuation));
        assert!(scheduler.resumed_call_stack_is_before_nmi());
    }

    #[test]
    fn cpu_timed_main_work_counts_only_a_boundary_still_ahead_of_the_cpu() {
        let continuation = GameWorkContinuation::FinishPreOverworldScreenBuild;

        let mut ordinary = GameExecutionScheduler::default();
        ordinary.begin_host_frame();
        ordinary.begin_main_loop_iteration();
        ordinary.schedule_cpu_timed_work_from_current_main_iteration(continuation, 3);
        assert!(!ordinary.active_field_precedes_current_scheduled_work());
        assert_eq!(
            ordinary.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting)
        );
        assert_eq!(
            ordinary.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(continuation))
        );

        let mut after_leading_nmi = GameExecutionScheduler::default();
        after_leading_nmi.begin_host_frame();
        after_leading_nmi.mark_main_iteration_after_leading_nmi();
        after_leading_nmi.begin_main_loop_iteration();
        after_leading_nmi.schedule_cpu_timed_work_from_current_main_iteration(continuation, 3);
        assert!(after_leading_nmi.active_field_precedes_current_scheduled_work());
        assert_eq!(
            after_leading_nmi.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting)
        );
        assert!(!after_leading_nmi.active_field_precedes_current_scheduled_work());
        assert_eq!(
            after_leading_nmi.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting)
        );
        assert_eq!(
            after_leading_nmi.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(continuation))
        );
    }

    #[test]
    fn completed_loader_nmi_marks_the_next_host_main_as_after_leading_nmi() {
        let continuation = GameWorkContinuation::FinishDialogueInitializationPrefix {
            caller_nmi_crossings: 1,
        };
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.begin_host_frame();

        // The loader returns and consumes the field boundary before the fresh
        // module begins in the following frontend callback.
        scheduler.mark_main_iteration_after_leading_nmi();
        scheduler.begin_host_frame();
        scheduler.begin_main_loop_iteration();
        assert!(scheduler.current_main_iteration_follows_leading_nmi());

        scheduler.schedule_cpu_timed_work_from_current_main_iteration(continuation, 1);
        scheduler.finish_main_loop_iteration();

        // That loader-owned NMI cannot also be the new synchronous call's
        // trailing boundary; its sole crossing is still future work.
        assert_eq!(scheduler.take_post_trailing_nmi(), None);
        assert_eq!(scheduler.scheduled_work_slices_remaining(), Some(1));
        scheduler.begin_host_frame();
        assert_eq!(
            scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(continuation))
        );
    }

    #[test]
    fn caller_resumed_after_an_interrupt_returns_before_the_next_leading_nmi() {
        let continuation = GameWorkContinuation::FinishDialogueInitializationCallerReturn;
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.schedule_after_current_trailing_nmi(continuation);
        scheduler.begin_host_frame();

        assert_eq!(
            scheduler.take_after_current_trailing_nmi(),
            Some(continuation)
        );
        assert!(scheduler.resumed_call_stack_is_before_nmi());

        scheduler.finish_call_stack_at_main_wait_before_nmi();
        scheduler.begin_host_frame();

        assert!(scheduler.main_return_requires_leading_nmi());
    }

    #[test]
    fn one_crossing_later_host_work_does_not_resume_in_the_scheduling_callback() {
        let continuation = GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn;
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.begin_host_frame();
        scheduler.begin_main_loop_iteration();
        scheduler.schedule_cpu_timed_work_returning_on_later_host(continuation, 1);
        scheduler.finish_main_loop_iteration();

        assert!(scheduler.main_call_stack_is_suspended_before_nmi());
        assert_eq!(scheduler.take_post_trailing_nmi(), None);

        scheduler.begin_host_frame();
        assert_eq!(
            scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(continuation))
        );
        assert!(scheduler.resumed_call_stack_is_before_nmi());
    }

    #[test]
    fn completed_scheduled_stack_and_following_main_preserve_leading_nmi_cadence() {
        let continuation = GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(3),
            caller: SpriteMainCpuCaller::DungeonModule07,
        };
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.schedule_work(continuation, 1);
        scheduler.begin_host_frame();

        assert_eq!(
            scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(continuation))
        );
        assert!(scheduler.resumed_call_stack_is_before_nmi());
        assert!(scheduler.is_idle());

        scheduler.finish_call_stack_at_main_wait_before_nmi();

        // The resumed suffix returned after this callback's NMI. The next
        // callback must therefore consume another leading NMI before main.
        scheduler.begin_host_frame();
        assert!(scheduler.fresh_main_loop_iteration_is_ready());
        assert!(scheduler.main_return_requires_leading_nmi());

        scheduler.mark_main_iteration_after_leading_nmi();
        scheduler.begin_main_loop_iteration();
        scheduler.finish_main_loop_iteration();
        scheduler.begin_host_frame();
        assert!(scheduler.main_return_requires_leading_nmi());
    }

    #[test]
    fn nmi_after_resumed_stack_marks_the_next_main_as_post_leading_nmi() {
        let continuation = GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(3),
            caller: SpriteMainCpuCaller::DungeonModule07,
        };
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.schedule_work(continuation, 1);
        scheduler.begin_host_frame();
        assert_eq!(
            scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(continuation))
        );
        assert!(scheduler.resumed_call_stack_is_before_nmi());

        // The resumed suffix reached a second NMI before the next fresh main
        // iteration. Preserve that event across the host callback boundary.
        scheduler.mark_main_iteration_after_leading_nmi();
        scheduler.begin_host_frame();
        scheduler.begin_main_loop_iteration();
        scheduler.finish_main_loop_iteration();

        assert!(scheduler.returned_main_is_waiting_for_nmi());
    }

    #[test]
    fn completed_trailing_nmi_releases_atomic_main_for_the_next_host() {
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.begin_host_frame();
        scheduler.begin_main_loop_iteration();
        scheduler.finish_main_loop_iteration();
        scheduler.finish_trailing_nmi_after_main_return();
        scheduler.begin_host_frame();

        assert!(scheduler.fresh_main_loop_iteration_is_ready());
        assert!(!scheduler.main_return_requires_leading_nmi());
    }

    #[test]
    fn native_caller_state_machine_can_return_to_wait_after_leading_nmi() {
        let mut scheduler = GameExecutionScheduler::default();
        scheduler.begin_host_frame();
        scheduler.finish_call_stack_at_main_wait_before_nmi();
        scheduler.begin_host_frame();

        assert!(scheduler.fresh_main_loop_iteration_is_ready());
        assert!(scheduler.main_return_requires_leading_nmi());
    }
}
