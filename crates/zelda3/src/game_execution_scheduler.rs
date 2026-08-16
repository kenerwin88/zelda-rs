use super::{
    DisplaySnapshotPublication, GameWorkContinuation, ItemReceiptGraphicsContinuation,
    PreMainCallerContinuation, PreMainNmiResume, SpotlightIteration,
    FILE_SELECT_GRAPHICS_NMI_SLICES, SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES,
    SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES,
};

const MASTER_CYCLES_PER_SCANLINE: u32 = 1_364;
const NTSC_SCANLINES_PER_FIELD: u32 = 262;
const NMI_SCANLINE: u32 = 225;
const HDMA_INIT_CYCLE: u32 = 20;
const WRAM_REFRESH_CYCLE: u32 = 538;
const WRAM_REFRESH_STALL_MASTER_CYCLES: u32 = 40;
const HDMA_START_CYCLE: u32 = 1_106;
const SHORT_SCANLINE_END_CYCLE: u32 = 1_360;
const SHORT_SCANLINE_MISSING_MASTER_CYCLES: u32 = 4;

/// A 65816 position within an NTSC field, expressed in S-CPU master cycles.
///
/// Translated routines normally execute atomically. Work that reaches NMI is
/// instead advanced through this clock so the continuation is selected by the
/// remaining cycle budget, not by a room or route identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CpuRasterPosition {
    scanline: u16,
    master_cycle: u16,
}

impl CpuRasterPosition {
    pub(super) const fn new(scanline: u16, master_cycle: u16) -> Self {
        Self {
            scanline,
            master_cycle,
        }
    }

    const fn unwrapped_master_cycles(self) -> u32 {
        self.scanline as u32 * MASTER_CYCLES_PER_SCANLINE + self.master_cycle as u32
    }

    pub(super) const fn coordinates(self) -> (u16, u16) {
        (self.scanline, self.master_cycle)
    }
}

/// Bus work which can preempt the CPU before NMI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CpuBusWorkload {
    hdma_stall_master_cycles: u16,
    dynamic_hdma: bool,
}

impl CpuBusWorkload {
    pub(super) const fn with_hdma_stall(hdma_stall_master_cycles: u16) -> Self {
        Self {
            hdma_stall_master_cycles,
            dynamic_hdma: false,
        }
    }

    pub(super) const fn with_dynamic_hdma() -> Self {
        Self {
            hdma_stall_master_cycles: 0,
            dynamic_hdma: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CpuBusEvent {
    WramRefresh,
    HdmaInit,
    HdmaStart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CpuTimelineEvent {
    Bus(CpuBusEvent),
    ShortScanline,
}

/// Video-field state which changes the CPU's available master-cycle budget.
/// In non-interlace mode, scanline 240 of each odd field is one dot shorter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CpuFieldTiming {
    odd_field: bool,
    interlace: bool,
}

impl CpuFieldTiming {
    pub(super) const NON_INTERLACE_EVEN: Self = Self {
        odd_field: false,
        interlace: false,
    };

    pub(super) const fn non_interlace(odd_field: bool) -> Self {
        Self {
            odd_field,
            interlace: false,
        }
    }

    const fn field_is_odd(self, field_index: u32) -> bool {
        self.odd_field ^ (field_index & 1 != 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CpuWorkAdvance {
    Complete,
    InterruptedAtNmi { remaining_work_master_cycles: u32 },
}

/// Result of advancing a sequence of semantic CPU phases toward NMI.
///
/// The phase index is stable across translated implementations: callers can
/// distinguish an interrupted routine body from an interrupted caller suffix
/// without recovering that distinction from a room or module-state value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CpuPhaseSequenceAdvance {
    Complete,
    InterruptedAtNmi {
        phase_index: usize,
        remaining_work_master_cycles: u32,
    },
}

impl CpuWorkAdvance {
    pub(super) const fn was_interrupted(self) -> bool {
        matches!(self, Self::InterruptedAtNmi { .. })
    }
}

/// Remaining CPU time before the next NMI, including refresh and HDMA bus
/// steals. Instructions are advanced atomically: when NMI becomes pending in
/// the middle of an instruction, that instruction completes before the
/// continuation is suspended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CpuCycleBudget {
    clock_master_cycles: u32,
    nmi_master_cycles: u32,
    bus: CpuBusWorkload,
    field_timing: CpuFieldTiming,
    processed_timeline_event: Option<(u32, CpuTimelineEvent)>,
}

impl CpuCycleBudget {
    pub(super) fn until_next_nmi(
        entry: CpuRasterPosition,
        bus: CpuBusWorkload,
        field_timing: CpuFieldTiming,
    ) -> Self {
        let clock_master_cycles = entry.unwrapped_master_cycles();
        let nmi_field = u32::from(entry.scanline >= NMI_SCANLINE as u16);
        let nmi_master_cycles =
            (nmi_field * NTSC_SCANLINES_PER_FIELD + NMI_SCANLINE) * MASTER_CYCLES_PER_SCANLINE;
        debug_assert!(clock_master_cycles < nmi_master_cycles);
        Self {
            clock_master_cycles,
            nmi_master_cycles,
            bus,
            field_timing,
            processed_timeline_event: None,
        }
    }

    /// Start at the pinned Snes9x core's vblank NMI trigger so the caller can
    /// execute the WAI wake and handler before the main-thread entry position.
    pub(super) fn at_nmi_trigger(bus: CpuBusWorkload, field_timing: CpuFieldTiming) -> Self {
        let nmi_master_cycles = NMI_SCANLINE * MASTER_CYCLES_PER_SCANLINE;
        Self {
            clock_master_cycles: nmi_master_cycles + 12,
            nmi_master_cycles,
            bus,
            field_timing,
            processed_timeline_event: None,
        }
    }

    /// Advance an interruptible span, stopping exactly when NMI becomes
    /// pending and retaining the unexecuted work for a continuation.
    pub(super) fn advance_interruptible(&mut self, mut work_master_cycles: u32) -> CpuWorkAdvance {
        debug_assert!(self.clock_master_cycles < self.nmi_master_cycles);
        while work_master_cycles != 0 {
            let (work_until_event, event) = self.next_timeline_event();
            let event_stall = event.map_or(0, |event| self.fixed_event_advance(event));
            let master_cycles_until_nmi = self.nmi_master_cycles - self.clock_master_cycles;
            if master_cycles_until_nmi <= work_until_event {
                if work_master_cycles <= master_cycles_until_nmi {
                    self.clock_master_cycles += work_master_cycles;
                    return CpuWorkAdvance::Complete;
                }
                self.clock_master_cycles = self.nmi_master_cycles;
                return CpuWorkAdvance::InterruptedAtNmi {
                    remaining_work_master_cycles: work_master_cycles - master_cycles_until_nmi,
                };
            }
            if work_master_cycles <= work_until_event {
                self.clock_master_cycles += work_master_cycles;
                return CpuWorkAdvance::Complete;
            }

            self.clock_master_cycles += work_until_event;
            work_master_cycles -= work_until_event;
            if let Some(event) = event {
                self.processed_timeline_event = Some((self.clock_master_cycles, event));
            }
            if self.clock_master_cycles + event_stall >= self.nmi_master_cycles {
                self.clock_master_cycles = self.nmi_master_cycles;
                return CpuWorkAdvance::InterruptedAtNmi {
                    remaining_work_master_cycles: work_master_cycles,
                };
            }
            self.clock_master_cycles += event_stall;
        }
        CpuWorkAdvance::Complete
    }

    /// Advance one indivisible 65816 instruction. An NMI which becomes pending
    /// during the instruction is observed immediately after it completes.
    pub(super) fn advance_instruction(&mut self, instruction_master_cycles: u32) -> CpuWorkAdvance {
        debug_assert!(self.clock_master_cycles < self.nmi_master_cycles);
        self.advance_uninterruptible(instruction_master_cycles)
    }

    /// Advance one instruction while deriving each HDMA steal from the cloned
    /// machine's live channel/table state. This is used by ROM timing shadows;
    /// translated fixed-cost phases retain `advance_instruction`.
    pub(super) fn advance_instruction_with_hdma(
        &mut self,
        instruction_master_cycles: u32,
        mut hdma_stall: impl FnMut(CpuBusEvent, u16) -> u32,
    ) -> CpuWorkAdvance {
        debug_assert!(self.clock_master_cycles < self.nmi_master_cycles);
        let dynamic_hdma = self.bus.dynamic_hdma;
        let fixed_hdma_stall = u32::from(self.bus.hdma_stall_master_cycles);
        self.advance_work_unbounded_with(
            instruction_master_cycles,
            |event, scanline| match event {
                CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh) => WRAM_REFRESH_STALL_MASTER_CYCLES,
                CpuTimelineEvent::Bus(event @ (CpuBusEvent::HdmaInit | CpuBusEvent::HdmaStart)) => {
                    if dynamic_hdma {
                        hdma_stall(event, scanline)
                    } else {
                        fixed_hdma_stall
                    }
                }
                CpuTimelineEvent::ShortScanline => SHORT_SCANLINE_MISSING_MASTER_CYCLES,
            },
        );
        if self.clock_master_cycles >= self.nmi_master_cycles {
            CpuWorkAdvance::InterruptedAtNmi {
                remaining_work_master_cycles: 0,
            }
        } else {
            CpuWorkAdvance::Complete
        }
    }

    /// Advance CPU-blocking work that must finish even when NMI is already
    /// pending, such as a general DMA started by the just-completed
    /// instruction.
    pub(super) fn advance_uninterruptible(&mut self, work_master_cycles: u32) -> CpuWorkAdvance {
        self.advance_work_unbounded(work_master_cycles);
        if self.clock_master_cycles >= self.nmi_master_cycles {
            CpuWorkAdvance::InterruptedAtNmi {
                remaining_work_master_cycles: 0,
            }
        } else {
            CpuWorkAdvance::Complete
        }
    }

    /// Open the next field's CPU budget after an NMI becomes pending.
    ///
    /// The caller must execute the real interrupt entry and handler through
    /// this same budget before resuming the interrupted instruction stream.
    /// Handler duration is deliberately not accepted here: it depends on the
    /// live NMI/DMA workload and therefore is not a constant.
    pub(super) fn begin_nmi_handler(&mut self) {
        debug_assert!(self.clock_master_cycles >= self.nmi_master_cycles);
        self.nmi_master_cycles = self
            .nmi_master_cycles
            .checked_add(NTSC_SCANLINES_PER_FIELD * MASTER_CYCLES_PER_SCANLINE)
            .expect("CPU continuation NMI deadline overflowed");
        debug_assert!(self.clock_master_cycles < self.nmi_master_cycles);
    }

    /// Advance ordered semantic phases until they complete or NMI preempts
    /// the current phase. The returned phase index identifies the continuation
    /// point without coupling the scheduler to a particular game subsystem.
    pub(super) fn advance_phases(
        &mut self,
        phase_work_master_cycles: &[u32],
    ) -> CpuPhaseSequenceAdvance {
        for (phase_index, &work_master_cycles) in phase_work_master_cycles.iter().enumerate() {
            if let CpuWorkAdvance::InterruptedAtNmi {
                remaining_work_master_cycles,
            } = self.advance_interruptible(work_master_cycles)
            {
                return CpuPhaseSequenceAdvance::InterruptedAtNmi {
                    phase_index,
                    remaining_work_master_cycles,
                };
            }
        }
        CpuPhaseSequenceAdvance::Complete
    }

    fn next_timeline_event(self) -> (u32, Option<CpuTimelineEvent>) {
        let nominal_field_master_cycles = NTSC_SCANLINES_PER_FIELD * MASTER_CYCLES_PER_SCANLINE;
        let field_index = self.clock_master_cycles / nominal_field_master_cycles;
        let field_cycle = self.clock_master_cycles % nominal_field_master_cycles;
        let scanline = field_cycle / MASTER_CYCLES_PER_SCANLINE;
        let cycle = field_cycle % MASTER_CYCLES_PER_SCANLINE;
        let mut next_event_cycle = MASTER_CYCLES_PER_SCANLINE;
        let mut next_event = None;

        for (event_cycle, event, enabled) in [
            (
                HDMA_INIT_CYCLE,
                CpuTimelineEvent::Bus(CpuBusEvent::HdmaInit),
                scanline == 0 && self.bus.dynamic_hdma,
            ),
            (
                WRAM_REFRESH_CYCLE,
                CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh),
                true,
            ),
            (
                HDMA_START_CYCLE,
                CpuTimelineEvent::Bus(CpuBusEvent::HdmaStart),
                scanline < NMI_SCANLINE
                    && (self.bus.dynamic_hdma || self.bus.hdma_stall_master_cycles != 0),
            ),
            (
                SHORT_SCANLINE_END_CYCLE,
                CpuTimelineEvent::ShortScanline,
                scanline == 240
                    && !self.field_timing.interlace
                    && self.field_timing.field_is_odd(field_index),
            ),
        ] {
            let already_processed =
                self.processed_timeline_event == Some((self.clock_master_cycles, event));
            if enabled
                && !already_processed
                && cycle <= event_cycle
                && event_cycle < next_event_cycle
            {
                next_event_cycle = event_cycle;
                next_event = Some(event);
            }
        }
        (next_event_cycle - cycle, next_event)
    }

    fn fixed_event_advance(&self, event: CpuTimelineEvent) -> u32 {
        match event {
            CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh) => WRAM_REFRESH_STALL_MASTER_CYCLES,
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaStart) => {
                u32::from(self.bus.hdma_stall_master_cycles)
            }
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaInit) => 0,
            CpuTimelineEvent::ShortScanline => SHORT_SCANLINE_MISSING_MASTER_CYCLES,
        }
    }

    fn advance_work_unbounded(&mut self, work_master_cycles: u32) {
        let hdma_stall_master_cycles = self.bus.hdma_stall_master_cycles;
        self.advance_work_unbounded_with(work_master_cycles, |event, _| match event {
            CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh) => WRAM_REFRESH_STALL_MASTER_CYCLES,
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaStart) => u32::from(hdma_stall_master_cycles),
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaInit) => 0,
            CpuTimelineEvent::ShortScanline => SHORT_SCANLINE_MISSING_MASTER_CYCLES,
        });
    }

    fn advance_work_unbounded_with(
        &mut self,
        mut work_master_cycles: u32,
        mut event_advance: impl FnMut(CpuTimelineEvent, u16) -> u32,
    ) {
        while work_master_cycles != 0 {
            let (work_until_event, event) = self.next_timeline_event();
            if work_until_event < work_master_cycles {
                self.clock_master_cycles += work_until_event;
                work_master_cycles -= work_until_event;
                if let Some(event) = event {
                    let field_cycle = self.clock_master_cycles
                        % (NTSC_SCANLINES_PER_FIELD * MASTER_CYCLES_PER_SCANLINE);
                    let scanline = (field_cycle / MASTER_CYCLES_PER_SCANLINE) as u16;
                    self.processed_timeline_event = Some((self.clock_master_cycles, event));
                    self.clock_master_cycles += event_advance(event, scanline);
                }
            } else {
                self.clock_master_cycles += work_master_cycles;
                work_master_cycles = 0;
            }
        }
    }

    pub(super) fn raster_position(self) -> CpuRasterPosition {
        let field_cycle =
            self.clock_master_cycles % (NTSC_SCANLINES_PER_FIELD * MASTER_CYCLES_PER_SCANLINE);
        CpuRasterPosition::new(
            (field_cycle / MASTER_CYCLES_PER_SCANLINE) as u16,
            (field_cycle % MASTER_CYCLES_PER_SCANLINE) as u16,
        )
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
}

impl ScheduledGameWork {
    pub(super) fn schedule(continuation: GameWorkContinuation, nmi_slices: u8) -> Self {
        debug_assert!(nmi_slices != 0);
        Self {
            continuation,
            nmi_slices_remaining: nmi_slices,
            entry_display_boundary_pending: true,
        }
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
                continuation: ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. },
            } | GameWorkContinuation::FinishDungeonSupertileTransition { .. }
                | GameWorkContinuation::FinishDungeonLinkOamCallerReturn
                | GameWorkContinuation::FinishDungeonNmiPrepareSpritesCallerReturn
                | GameWorkContinuation::FinishGameOverSpotlightBuild { .. }
                | GameWorkContinuation::FinishDungeonSubtilePaletteFilter
                | GameWorkContinuation::FinishStraightInterroomFadeoutSuffix
                | GameWorkContinuation::FinishStraightInterroomSpriteReset
                | GameWorkContinuation::FinishDungeonSpriteMain { .. }
                | GameWorkContinuation::FinishDungeonCachedSpriteMain { .. }
                | GameWorkContinuation::FinishSpiralStaircasePaletteFilter { .. }
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
            | GameWorkContinuation::FinishGameOverSpotlightBuild { iteration } => {
                Some(iteration.in_flight_publication())
            }
            GameWorkContinuation::FinishAttractWorldMapExit => {
                Some(DisplaySnapshotPublication::RetainPublished)
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
            | GameWorkContinuation::FinishGameOverSpotlightBuild { iteration } => Some(iteration),
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
    /// The translated caller has returned through the leading NMI which
    /// starts a multi-state upload pipeline. This survives the one-shot
    /// continuation so later states can preserve that CPU/NMI ordering
    /// without guessing from a room or from the module state alone.
    leading_nmi_upload_pipeline_active: bool,
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
        } else {
            CpuHostPhase::ReturnedToMainLoop
        };
    }

    pub(super) fn fresh_main_loop_iteration_is_ready(self) -> bool {
        self.cpu_host_phase == CpuHostPhase::MainLoopReady
    }

    pub(super) fn eligible_leading_nmi_preceded_suspended_work(self) -> bool {
        self.leading_nmi_follows_returned_main && self.work_suspends_translated_call_stack()
    }

    pub(super) fn resumed_call_stack_is_before_nmi(self) -> bool {
        self.cpu_host_phase == CpuHostPhase::ResumedCallStackBeforeNmi
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

    pub(super) fn schedule_post_trailing_nmi(&mut self, continuation: GameWorkContinuation) {
        self.schedule_continuation(GameExecutionContinuation::PostTrailingNmi(continuation));
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
                Some(GameExecutionContinuation::PostTrailingNmi(_))
            )
    }

    pub(super) fn work_suspends_translated_call_stack(self) -> bool {
        self.scheduled_work()
            .is_some_and(ScheduledGameWork::suspends_translated_call_stack)
            || matches!(
                self.continuation,
                Some(GameExecutionContinuation::PostTrailingNmi(_))
                    | Some(GameExecutionContinuation::PreMainCaller(
                        PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass { .. }
                            | PreMainCallerContinuation::SpiralStairsSecondPaletteFilter
                            | PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter
                    ))
            )
    }

    pub(super) fn current_work(self) -> Option<GameWorkContinuation> {
        self.scheduled_work()
            .map(|work| work.continuation)
            .or_else(|| match self.continuation {
                Some(GameExecutionContinuation::PostTrailingNmi(continuation)) => {
                    Some(continuation)
                }
                _ => None,
            })
    }

    pub(super) fn take_post_trailing_nmi(&mut self) -> Option<GameWorkContinuation> {
        match self.continuation {
            Some(GameExecutionContinuation::PostTrailingNmi(continuation)) => {
                self.continuation = None;
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

    const DUNGEON_HDMA_STALL: u16 = 42;
    const WORK_TO_CACHED_RESTORE: u32 = 1_400 + 4 * 10_674 + 8_884;

    fn restored_fields_before_nmi(entry: CpuRasterPosition) -> usize {
        let mut budget = CpuCycleBudget::until_next_nmi(
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
            if budget.advance_instruction(28).was_interrupted() {
                break;
            }
            let advance = budget.advance_instruction(if field == 1 { 40 } else { 38 });
            restored += 1;
            if advance.was_interrupted() {
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
            let mut budget = CpuCycleBudget::until_next_nmi(
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
            restored_fields_before_nmi(CpuRasterPosition::new(182, 1_266)),
            15
        );
        assert_eq!(
            restored_fields_before_nmi(CpuRasterPosition::new(183, 44)),
            12
        );
        assert_eq!(
            restored_fields_before_nmi(CpuRasterPosition::new(183, 124)),
            11
        );
    }

    #[test]
    fn interrupted_work_retains_its_remaining_cycle_budget() {
        let mut budget = CpuCycleBudget::until_next_nmi(
            CpuRasterPosition::new(224, 1_300),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            budget.advance_interruptible(100),
            CpuWorkAdvance::InterruptedAtNmi {
                remaining_work_master_cycles: 36,
            }
        );
    }

    #[test]
    fn interrupted_instruction_stream_resumes_after_nmi_in_the_next_field() {
        let mut budget = CpuCycleBudget::until_next_nmi(
            CpuRasterPosition::new(224, 1_350),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            budget.advance_instruction(28),
            CpuWorkAdvance::InterruptedAtNmi {
                remaining_work_master_cycles: 0,
            }
        );
        assert_eq!(budget.raster_position(), CpuRasterPosition::new(225, 14),);

        budget.begin_nmi_handler();
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
            let mut budget = CpuCycleBudget::until_next_nmi(
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
    fn fixed_hdma_budget_does_not_invoke_the_dynamic_dma_model() {
        let mut budget = CpuCycleBudget::until_next_nmi(
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
        let mut budget = CpuCycleBudget::until_next_nmi(
            CpuRasterPosition::new(224, 1_300),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            budget.advance_phases(&[32, 68]),
            CpuPhaseSequenceAdvance::InterruptedAtNmi {
                phase_index: 1,
                remaining_work_master_cycles: 36,
            }
        );
    }

    #[test]
    fn odd_noninterlace_field_skips_the_missing_dot_on_scanline_240() {
        let entry = CpuRasterPosition::new(240, 1_350);
        let mut even = CpuCycleBudget::until_next_nmi(
            entry,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut odd = CpuCycleBudget::until_next_nmi(
            entry,
            CpuBusWorkload::default(),
            CpuFieldTiming::non_interlace(true),
        );

        assert_eq!(even.advance_interruptible(20), CpuWorkAdvance::Complete);
        assert_eq!(odd.advance_interruptible(20), CpuWorkAdvance::Complete);
        assert_eq!(even.raster_position(), CpuRasterPosition::new(241, 6),);
        assert_eq!(odd.raster_position(), CpuRasterPosition::new(241, 10));
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
}
