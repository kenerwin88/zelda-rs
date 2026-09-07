//! Types split out of `zelda_rtl.rs` by family (types_original_timing).
//! Mechanical move: definitions are unchanged; items and struct fields
//! became `pub(crate)` so the parent module and its siblings see them.

use super::*;

/// Ordered, backend-neutral source events surrounding one interrupted main-loop
/// call. The temporary Snes9x authority derives these facts from its private
/// execution trace, but translated gameplay sees only accepted NMIs, the C
/// main-loop outcome, and the semantic interruption phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingNmiPhase {
    Accepted(NmiUpdateGate),
    HandlerCompleted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingMainLoopInterruptionTimeline {
    pub(crate) progress: crate::MainLoopProgress,
    pub(crate) interruption: crate::MainLoopInterruption,
    pub(crate) nmi_phases_before_interruption: Vec<OriginalTimingNmiPhase>,
    pub(crate) nmi_phases_after_interruption: Vec<OriginalTimingNmiPhase>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingMainLoopTimeline {
    pub(crate) progress: crate::MainLoopProgress,
    pub(crate) nmi_phases_before_progress: Vec<OriginalTimingNmiPhase>,
    pub(crate) nmi_phases_after_progress: Vec<OriginalTimingNmiPhase>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingIdleMainLoopSuffixAction {
    ArmOrdinary,
    RetainOrdinary,
    CompleteInSequence,
    /// The ROM idled at its main wait with the NMI disabled for a whole
    /// host interval (pre-overworld forced blank); nothing runs natively.
    HoldAtMainWait,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingIdleMainLoopDialogueClaim {
    None,
    ResumedRenderingWithoutMainIteration {
        message_read_position: u16,
        current_glyph_started: bool,
        transition: messaging::SuspendedVwfEndpointTransition,
    },
    DialogueClosed,
}

/// Pure authority for one idle-scheduler host which entered or resumed
/// `ZeldaRunGameLoop` but did not reach its unconditional common suffix.
///
/// The full semantic vector is retained so hardware/control receipts cannot
/// be removed before every body-owned domain fact and suffix transition has
/// been proven consumable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingUninterruptedIdleMainLoopPlan {
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    pub(crate) timeline: OriginalTimingMainLoopTimeline,
    pub(crate) in_module_phases_after_progress: Vec<OriginalTimingNmiPhase>,
    pub(crate) post_suffix_phases: Vec<OriginalTimingNmiPhase>,
    pub(crate) before_main: OriginalTimingNmiPhaseClassification,
    pub(crate) suffix_action: OriginalTimingIdleMainLoopSuffixAction,
    pub(crate) sprite_main_returned_claims: usize,
    pub(crate) spotlight_claim: Option<SpotlightTableBuildProgressReceipt>,
    pub(crate) dialogue_claim: OriginalTimingIdleMainLoopDialogueClaim,
    pub(crate) pre_main_timing_shadow: Option<PreMainNmiResume>,
}

/// Immutable authority for an idle translated caller which returns through
/// ZeldaRunGameLoop's already-owned common suffix in this host.
///
/// A dialogue endpoint is appended by the adapter at host finish even though
/// its C work precedes the common suffix. Keeping the claim in this plan lets
/// execution consume it in source order without weakening ordinary terminal
/// continued-call ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingUninterruptedIdleContinuedReturnPlan {
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    pub(crate) timeline: OriginalTimingMainLoopReturnTimeline,
    pub(crate) cpu_action: OriginalTimingIdleContinuedReturnCpuAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingIdleContinuedReturnCpuAction {
    None,
    /// The carried save-menu `RenderText_PostDeathSaveOptions` call finishes,
    /// then Module0E_0B executes its HUD/core-flag and subsubmodule suffix
    /// before ZeldaRunGameLoop reaches the common return path.
    SaveMenuInitializationCompletion,
    DialogueEndpoint {
        message_read_position: u16,
        current_glyph_started: bool,
        transition: messaging::SuspendedVwfEndpointTransition,
    },
    SuspendedVwfCompletion {
        transition: messaging::SuspendedVwfCompletionTransition,
    },
    /// The ROM's main loop is still inside the message-line scroll copy at
    /// entry; this terminal host copies the remaining pixels, returns
    /// through RenderText/Module0E, and then runs the shared suffix.
    DialogueScrollCompletion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingInterruptedIdleMainLoopPlan {
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    pub(crate) timeline: OriginalTimingMainLoopInterruptionTimeline,
    pub(crate) sprite_main_returned_claims: usize,
    pub(crate) pre_main_timing_shadow: Option<PreMainNmiResume>,
    pub(crate) scheduled_predecessor: Option<OriginalTimingScheduledFreshIterationPredecessor>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingScheduledFreshIterationPredecessor {
    pub(crate) work: GameWorkContinuation,
    pub(crate) scheduler_before_step: GameExecutionScheduler,
    pub(crate) scheduler_after_step: GameExecutionScheduler,
}

/// Immutable authority for the final graphics slice of a chest item receipt.
///
/// The source NMI completes while the decompressor still owns the C stack.  The
/// resumed ground-handler tail then returns through Module 7's one outer
/// `Sprite_Main` call and ZeldaRunGameLoop's ordinary common suffix.  Keeping
/// this distinct from generic item-progress ownership preserves that exact
/// ordering and the retained-display publication used by the graphics hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingTerminalGroundItemReceiptPlan {
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    pub(crate) timeline: OriginalTimingMainLoopReturnTimeline,
    pub(crate) work: GameWorkContinuation,
    pub(crate) continuation: ItemReceiptGraphicsContinuation,
    pub(crate) scheduler_before_completion: GameExecutionScheduler,
    pub(crate) scheduler_after_completion: GameExecutionScheduler,
}

/// Immutable authority for a source-proven terminal closing-spotlight caller.
///
/// A suffix-only action proves that table/Link-OAM work returned on the
/// preceding source host. A terminal Build action instead proves the exact
/// host which resumes its saved table/control/Link-OAM CPU body after exactly
/// one carried or same-host Held handler, before completing ZeldaRunGameLoop's
/// already-armed ordinary common suffix and optionally carrying one trailing
/// Open acceptance.
/// The wire-last caller-return fact distinguishes both from nonterminal
/// spotlight work and from a fresh main-loop iteration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingTerminalSpotlightIterationPlan {
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    pub(crate) timeline: OriginalTimingMainLoopReturnTimeline,
    pub(crate) work: GameWorkContinuation,
    pub(crate) iteration: SpotlightIteration,
    pub(crate) cpu_action: OriginalTimingTerminalSpotlightCpuAction,
    /// A re-checkpoint of the saved ProjectionCopy published at this host's
    /// leading Held acceptance, proving the vblank interrupted the resumed
    /// copy before the terminal caller return completed it.
    pub(crate) spotlight_claim: Option<SpotlightTableBuildProgressReceipt>,
    /// Whether the wire carries the module-qualified
    /// `DungeonExitSpotlightCallerReturnedToMainWait` receipt. The final
    /// suffix-only continuation returns after its module body already left
    /// Module0F, so the adapter proves that return without the token.
    pub(crate) caller_return_token: bool,
    /// Whether the wire instead carries the entry-return token: a close-entry
    /// iteration whose host itself crossed the sub-0 entry transition is
    /// classified as the entry call returning, not the recurring caller.
    pub(crate) entry_return_token: bool,
    /// Whether the wire carries the Module10 opening-goal caller-return
    /// token: an opening suffix-only terminal on the overworld publishes it
    /// instead of the Module0F token (route host 74314).
    pub(crate) overworld_goal_return_token: bool,
    pub(crate) scheduler_before_completion: GameExecutionScheduler,
    pub(crate) scheduler_after_completion: GameExecutionScheduler,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingTerminalSpotlightCpuAction {
    SuffixOnly,
    CompleteBuild {
        table_build: SpotlightTableBuildContinuation,
        projection_completed: bool,
    },
}

/// Immutable authority for the recurring spotlight host which finishes its
/// saved table build exactly at Module0F's LinkOam boundary.
///
/// The interruption is the source proof that the Build CPU prefix completed;
/// its ordinary ZeldaRunGameLoop suffix remains scheduled for the following
/// host.  Validate the entire same-host Held lifecycle before consuming any
/// receipt or advancing the real scheduler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingSpotlightBuildLinkOamPlan {
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    pub(crate) rebuild_progress: Option<SpotlightTableBuildProgress>,
    pub(crate) timeline: OriginalTimingMainLoopInterruptionTimeline,
    /// `LinkOam`, or the mid-loop Link position boundary when the host ended
    /// inside `Link_MovePosition` after the completed build (route host
    /// 179586).
    pub(crate) interruption: crate::MainLoopInterruption,
    pub(crate) work: GameWorkContinuation,
    pub(crate) scheduler_before_completion: GameExecutionScheduler,
    pub(crate) scheduler_after_completion: GameExecutionScheduler,
    pub(crate) scheduler_after_cpu: GameExecutionScheduler,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingMainLoopReturnTimeline {
    pub(crate) progress: crate::MainLoopProgress,
    pub(crate) nmi_phases_before_return: Vec<OriginalTimingNmiPhase>,
    pub(crate) nmi_phases_after_return: Vec<OriginalTimingNmiPhase>,
    /// The wire published its Sprite_Main return before the first NMI phase
    /// which precedes the caller return: the slot loop finished before
    /// vblank and the module tail ran after the held handler (route host
    /// 1415870).
    pub(crate) sprite_main_returned_before_nmi: bool,
}

/// One source host which resumes a suspended caller but does not yet reach the
/// shared `ZeldaRunGameLoop` suffix.  The complete semantic vector is retained
/// so validation and consumption cannot disagree about an otherwise-unowned
/// receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingNonterminalContinuationPlan {
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    pub(crate) timeline: OriginalTimingMainLoopTimeline,
    pub(crate) nmi: OriginalTimingNmiPhaseClassification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingFileSelectLowWramPublication {
    Progress(FileSelectGraphicsLowWramClearProgress),
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingIterationStartedContinuationPlan {
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    pub(crate) timeline: OriginalTimingMainLoopTimeline,
    pub(crate) before_progress: OriginalTimingNmiPhaseClassification,
    pub(crate) after_progress: OriginalTimingNmiPhaseClassification,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingSelectedGameLoadAction {
    Nonterminal(OriginalTimingNonterminalContinuationPlan),
    SpriteResetCheckpoint {
        semantic: Vec<OriginalTimingSemanticReceipt>,
        timeline: OriginalTimingMainLoopTimeline,
        nmi: OriginalTimingNmiPhaseClassification,
        receipt: SpriteResetAllProgressReceipt,
    },
    Terminal {
        semantic: Vec<OriginalTimingSemanticReceipt>,
        timeline: OriginalTimingMainLoopReturnTimeline,
        sprite_reset: PreDungeonSpriteResetContinuation,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingSelectedGameLoadPlan {
    pub(crate) action: OriginalTimingSelectedGameLoadAction,
    pub(crate) destination: SelectedGameLoadDestination,
    pub(crate) message_interface_published: bool,
    pub(crate) completes_entry_room_load: bool,
    pub(crate) scheduler_before_transition: GameExecutionScheduler,
    pub(crate) scheduler_after_transition: GameExecutionScheduler,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Exact mid-call publication of Module05's Message interface. The source has
/// returned from `Main_ShowTextMessage`, but its palette and Module1B tail are
/// still suspended.
pub(crate) struct OriginalTimingSelectedGameLoadMessageInterfacePlan {
    pub(crate) continuation: OriginalTimingNonterminalContinuationPlan,
    pub(crate) scheduler_before_transition: GameExecutionScheduler,
    pub(crate) scheduler_after_transition: GameExecutionScheduler,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingNonterminalReceiptPlacement {
    BeforeProgress,
    BeforeTrailingAcceptance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingBeginSelectedGameLoadPlan {
    pub(crate) entrance_scroll_published: bool,
    pub(crate) entrance_before_selection: bool,
    pub(crate) entrance_returned: bool,
    pub(crate) semantic: Vec<OriginalTimingSemanticReceipt>,
    pub(crate) timeline: OriginalTimingMainLoopTimeline,
    pub(crate) nmi: OriginalTimingNmiPhaseClassification,
    pub(crate) destination: SelectedGameLoadDestination,
    pub(crate) scheduler_before_transition: GameExecutionScheduler,
    pub(crate) scheduler_after_transition: GameExecutionScheduler,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingIntroMemoryDarkenPlan {
    Nonterminal(OriginalTimingNonterminalContinuationPlan),
    Terminal(OriginalTimingMainLoopReturnTimeline),
}

/// The Module17 save-quit reset (`Death_Func15`'s save-quit tail: the intro
/// WRAM clear plus the overworld song-bank upload) holds the wire for tens of
/// slices before its caller returns through Sprite_Main/LinkOam and the
/// shared suffix (route hosts 159333-159401).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingSaveQuitResetPlan {
    Nonterminal(OriginalTimingNonterminalContinuationPlan),
    IntroMemoryReturned(OriginalTimingNonterminalContinuationPlan),
    ResetStatePublished(OriginalTimingNonterminalContinuationPlan),
    /// The ROM masks NMI during the reset's song-bank upload: the host
    /// carries a bare `CallStackContinued` with no NMI lifecycle at all
    /// (route host 159380).
    NmiMaskedHold,
    /// The upload finishes mid-host: Module17's Sprite_Main returns, then
    /// the shared NMI_PrepareSprites suffix is interrupted by a Held
    /// acceptance and resumes on a later host (route host 159403).
    TerminalInterruptedSuffix,
    Terminal(OriginalTimingMainLoopReturnTimeline),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingIntroPolyPlan {
    AwaitingIterationStart(OriginalTimingIterationStartedContinuationPlan),
    Suspended(OriginalTimingNonterminalContinuationPlan),
    Terminal(OriginalTimingMainLoopReturnTimeline),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingNmiHandlerCompletionOwner {
    None,
    AcceptedInThisSequence,
    PendingAtSequenceEntry,
}

impl OriginalTimingNmiHandlerCompletionOwner {
    pub(crate) const fn completed(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OriginalTimingNmiPhaseClassification {
    pub(crate) handler_completion: OriginalTimingNmiHandlerCompletionOwner,
    pub(crate) publication_pending_at_exit: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingHostCloseControl {
    Exhausted,
    CarryTerminalAcceptance(NmiUpdateGate),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingOwner {
    /// Original timing is not requested by the current runtime policy.
    Disabled,
    /// A genuinely fresh reset is eligible to seed the exact owner at the
    /// next host execution boundary.
    PendingColdStart,
    /// An exact timing backend owns one receipt interval per host call.
    Live,
    /// Exact timing cannot safely resume or retry from the current state.
    Unavailable(OriginalTimingUnavailableReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginalTimingUnavailableReason {
    ProgressedState,
    CheckpointRestore,
    MissingRom,
    MissingAuthorityReceipt,
    AuthorityReceiptInputMismatch,
    AuthorityAudioShapeMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OriginalTimingResumeCheckpointError {
    Version(u32),
    UnavailableOwner,
    ActiveHostDispatch,
    UnconsumedHostReceipt,
    UnconsumedPresentation,
    ActiveTranslatedContinuation,
    MissingHostCallProvenance,
    TimingDisabled,
}

impl core::fmt::Display for OriginalTimingResumeCheckpointError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Version(schema) => write!(
                formatter,
                "unsupported original-timing resume checkpoint schema {schema}"
            ),
            Self::UnavailableOwner => formatter.write_str(
                "original-timing resume checkpoint was captured from an unavailable owner",
            ),
            Self::ActiveHostDispatch => formatter.write_str(
                "original-timing resume checkpoint was captured during an active host dispatch",
            ),
            Self::UnconsumedHostReceipt => formatter
                .write_str("original-timing resume checkpoint contains an unconsumed host receipt"),
            Self::UnconsumedPresentation => formatter.write_str(
                "original-timing resume checkpoint contains unconsumed presentation state",
            ),
            Self::ActiveTranslatedContinuation => formatter.write_str(
                "original-timing resume checkpoint is inside a translated call continuation",
            ),
            Self::MissingHostCallProvenance => formatter.write_str(
                "original-timing resume checkpoint has a continuation without host-call provenance",
            ),
            Self::TimingDisabled => formatter
                .write_str("original-timing resume checkpoint restore requires live timing policy"),
        }
    }
}

impl std::error::Error for OriginalTimingResumeCheckpointError {}

#[derive(Debug, Clone)]
pub(crate) enum OriginalTimingOwnerState {
    Disabled,
    PendingColdStart,
    /// A replaceable timing authority is publishing one typed receipt interval
    /// per host call. Backend identity remains outside ZeldaState: today the
    /// producer is pinned Snes9x, while a native timing owner can later publish
    /// the same Zelda-level receipt vocabulary without changing gameplay.
    Live,
    Unavailable(OriginalTimingUnavailableReason),
}

impl OriginalTimingOwnerState {
    pub(crate) const fn status(&self) -> OriginalTimingOwner {
        match self {
            Self::Disabled => OriginalTimingOwner::Disabled,
            Self::PendingColdStart => OriginalTimingOwner::PendingColdStart,
            Self::Live => OriginalTimingOwner::Live,
            Self::Unavailable(reason) => OriginalTimingOwner::Unavailable(*reason),
        }
    }
}

impl Default for OriginalTimingOwnerState {
    fn default() -> Self {
        // A skipped field is defaulted during deserialization. Defaulting to
        // unavailable makes every positional ZeldaState restore fail closed,
        // including frame-zero and direct PlayCrash checkpoints whose caller
        // has not yet invoked the live-ROM restoration hook.
        Self::Unavailable(OriginalTimingUnavailableReason::CheckpointRestore)
    }
}

#[must_use]
pub(crate) struct OriginalTimingNmiHandlerCompletion {
    pub(crate) completed: bool,
    pub(crate) dialogue_text_dma: Option<DialogueTextDmaPublicationToken>,
}

impl OriginalTimingNmiHandlerCompletion {
    pub(crate) const fn none() -> Self {
        Self {
            completed: false,
            dialogue_text_dma: None,
        }
    }

    pub(crate) const fn completed(&self) -> bool {
        self.completed
    }

    pub(crate) fn assert_no_unclaimed_dialogue_text_dma(self) -> bool {
        assert!(
            self.dialogue_text_dma.is_none(),
            "a completed BG3 text-DMA publication lost its immediate dialogue staging owner",
        );
        self.completed
    }
}
