use super::{
    DisplaySnapshotPublication, GameWorkContinuation, ItemReceiptGraphicsContinuation,
    PreMainCallerContinuation, PreMainNmiResume, SpotlightIteration,
    FILE_SELECT_GRAPHICS_NMI_SLICES, SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES,
    SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES,
};

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
                | GameWorkContinuation::FinishGameOverSpotlightBuild { .. }
                | GameWorkContinuation::FinishDungeonSupertileFilteringReturn
                | GameWorkContinuation::FinishDungeonSubtilePaletteFilter
                | GameWorkContinuation::FinishStraightInterroomFadeoutSuffix
                | GameWorkContinuation::FinishStraightInterroomSpriteReset
                | GameWorkContinuation::FinishDungeonRoomLoadSpriteMain { .. }
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
}

impl GameExecutionScheduler {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn is_idle(self) -> bool {
        self.continuation.is_none()
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
                        PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass
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
        let step = work.advance_one_nmi_slice();
        if matches!(step, GameWorkStep::Complete(_)) {
            self.continuation = None;
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
    }
}
