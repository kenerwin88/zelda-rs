//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (original_timing).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;
use crate::game_state::constants::VIRQ_TRIGGER;

impl ZeldaState {
    /// Capture the Zelda-level semantic continuation owned at one exact
    /// pre-frame boundary. This sidecar intentionally excludes the temporary
    /// authority backend and preserves positional `ZeldaState` bytes.
    pub fn capture_original_timing_resume_checkpoint(
        &self,
    ) -> Result<crate::OriginalTimingResumeCheckpoint, OriginalTimingResumeCheckpointError> {
        if matches!(
            self.original_timing_owner,
            OriginalTimingOwnerState::Unavailable(_)
        ) {
            return Err(OriginalTimingResumeCheckpointError::UnavailableOwner);
        }
        if self.original_timing_host_dispatch_active {
            return Err(OriginalTimingResumeCheckpointError::ActiveHostDispatch);
        }
        if self.original_timing_semantic_receipts.is_some() {
            return Err(OriginalTimingResumeCheckpointError::UnconsumedHostReceipt);
        }
        if self.original_timing_presented_audio.is_some()
            || self.active_presented_bg_scroll.is_some()
            || self.active_presented_mode7_transform.is_some()
            || self.active_presented_window_mask.is_some()
        {
            return Err(OriginalTimingResumeCheckpointError::UnconsumedPresentation);
        }
        if self.original_timing_nmi_publication_pending {
            // Display snapshots are runtime presentation owners and are not
            // serialized by the semantic sidecar.  A checkpoint between NMI
            // acceptance and handler completion would therefore restore a
            // gate without the immutable field that handler must refine.
            return Err(OriginalTimingResumeCheckpointError::ActiveTranslatedContinuation);
        }
        if !self.paired_resume_cpu_boundary_is_quiescent() {
            return Err(OriginalTimingResumeCheckpointError::ActiveTranslatedContinuation);
        }
        let has_continuation = self.original_timing_nmi_publication_pending
            || self.original_timing_dungeon_exit_spotlight_entry_return_pending
            || self.original_timing_pre_dungeon_return_pending.is_some()
            || self.item_receipt_completion_live_link_dma_host.is_some();
        if has_continuation && self.original_timing_last_oracle_host_call.is_none() {
            return Err(OriginalTimingResumeCheckpointError::MissingHostCallProvenance);
        }
        Ok(crate::OriginalTimingResumeCheckpoint {
            schema: crate::OriginalTimingResumeCheckpoint::SCHEMA,
            last_consumed_host_call: self.original_timing_last_oracle_host_call,
            nmi_publication_pending: self.original_timing_nmi_publication_pending,
            pending_nmi_update_gate: self.original_timing_pending_nmi_update_gate,
            dungeon_exit_spotlight_entry_return_pending: self
                .original_timing_dungeon_exit_spotlight_entry_return_pending,
            pre_dungeon_return_pending: self.original_timing_pre_dungeon_return_pending,
            item_receipt_live_link_dma_host: self.item_receipt_completion_live_link_dma_host,
        })
    }

    /// Restore a separately persisted semantic continuation after ordinary
    /// Zelda state restoration has invalidated runtime-only timing ownership.
    /// The next typed authority receipt re-enters `Live`; this method never
    /// reconstructs or exposes emulator-private state.
    pub fn restore_original_timing_resume_checkpoint(
        &mut self,
        checkpoint: crate::OriginalTimingResumeCheckpoint,
    ) -> Result<(), OriginalTimingResumeCheckpointError> {
        if !matches!(
            checkpoint.schema,
            1 | crate::OriginalTimingResumeCheckpoint::SCHEMA
        ) {
            return Err(OriginalTimingResumeCheckpointError::Version(
                checkpoint.schema,
            ));
        }
        if checkpoint.schema == 1 && checkpoint.nmi_publication_pending {
            return Err(OriginalTimingResumeCheckpointError::Version(
                checkpoint.schema,
            ));
        }
        if checkpoint.schema == 1 && checkpoint.pending_nmi_update_gate.is_some() {
            return Err(OriginalTimingResumeCheckpointError::Version(
                checkpoint.schema,
            ));
        }
        if checkpoint.schema == crate::OriginalTimingResumeCheckpoint::SCHEMA
            && checkpoint.nmi_publication_pending != checkpoint.pending_nmi_update_gate.is_some()
        {
            return Err(OriginalTimingResumeCheckpointError::Version(
                checkpoint.schema,
            ));
        }
        if !self.rom_startup_timing {
            return Err(OriginalTimingResumeCheckpointError::TimingDisabled);
        }
        if self.original_timing_host_dispatch_active {
            return Err(OriginalTimingResumeCheckpointError::ActiveHostDispatch);
        }
        if self.original_timing_semantic_receipts.is_some() {
            return Err(OriginalTimingResumeCheckpointError::UnconsumedHostReceipt);
        }
        if self.original_timing_presented_audio.is_some()
            || self.active_presented_bg_scroll.is_some()
            || self.active_presented_mode7_transform.is_some()
            || self.active_presented_window_mask.is_some()
        {
            return Err(OriginalTimingResumeCheckpointError::UnconsumedPresentation);
        }
        if !self.paired_resume_cpu_boundary_is_quiescent() {
            return Err(OriginalTimingResumeCheckpointError::ActiveTranslatedContinuation);
        }
        let has_continuation = checkpoint.nmi_publication_pending
            || checkpoint.dungeon_exit_spotlight_entry_return_pending
            || checkpoint.pre_dungeon_return_pending.is_some()
            || checkpoint.item_receipt_live_link_dma_host.is_some();
        if has_continuation && checkpoint.last_consumed_host_call.is_none() {
            return Err(OriginalTimingResumeCheckpointError::MissingHostCallProvenance);
        }
        if checkpoint.nmi_publication_pending {
            return Err(OriginalTimingResumeCheckpointError::ActiveTranslatedContinuation);
        }

        self.original_timing_last_oracle_host_call = checkpoint.last_consumed_host_call;
        self.original_timing_nmi_publication_pending = checkpoint.nmi_publication_pending;
        self.original_timing_pending_nmi_update_gate = checkpoint.pending_nmi_update_gate;
        self.original_timing_pending_nmi_ppu_register_operands = None;
        self.original_timing_dungeon_exit_spotlight_entry_return_pending =
            checkpoint.dungeon_exit_spotlight_entry_return_pending;
        self.original_timing_pre_dungeon_return_pending = checkpoint.pre_dungeon_return_pending;
        self.item_receipt_completion_live_link_dma_host =
            checkpoint.item_receipt_live_link_dma_host;
        Ok(())
    }

    pub(crate) const fn original_timing_owner(&self) -> OriginalTimingOwner {
        self.original_timing_owner.status()
    }

    /// Host-only acknowledgment that the timing-authority receipt for this
    /// call was accepted and all mandatory semantic facts were consumed. No
    /// CPU, raster, emulator, or backend identity crosses this boundary.
    pub fn last_consumed_original_timing_host_call(&self) -> Option<u64> {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            .then_some(self.original_timing_last_oracle_host_call)
            .flatten()
    }

    pub(super) fn invalidate_original_timing_after_checkpoint(&mut self) {
        self.original_timing_owner = OriginalTimingOwnerState::Unavailable(
            OriginalTimingUnavailableReason::CheckpointRestore,
        );
        self.original_timing_cold_start_eligible = false;
        self.original_timing_host_dispatch_active = false;
        self.original_timing_semantic_receipts = None;
        self.original_timing_sprite_main_return_claims_remaining = None;
        self.original_timing_nmi_publication_pending = false;
        self.original_timing_pending_nmi_update_gate = None;
        self.original_timing_pending_nmi_ppu_register_operands = None;
        self.original_timing_expected_nmi_update_gates.clear();
        self.original_timing_expected_nmi_ppu_register_operands
            .clear();
        self.original_timing_dungeon_exit_spotlight_entry_return_pending = false;
        self.original_timing_pre_dungeon_return_pending = None;
        self.original_timing_presented_audio = None;
        self.original_timing_audio_shadow_result = None;
        self.original_timing_bg_tilemap_shadow_result = None;
        self.original_timing_dialogue_text_shadow_result = None;
        self.original_timing_bg_scroll_shadow_result = None;
        self.original_timing_mode7_transform_shadow_result = None;
        self.original_timing_window_mask_shadow_result = None;
        self.original_timing_last_oracle_host_call = None;
        self.pending_main_loop_common_suffix = None;
    }

    /// Install one timing authority's semantic result for the next host call.
    /// The backend remains externally owned; ZeldaState receives only the
    /// replaceable, Zelda-level receipt vocabulary.
    pub fn install_original_timing_host_receipts(
        &mut self,
        receipts: OriginalTimingHostReceipts,
    ) -> Result<(), OriginalTimingReceiptInstallError> {
        if !self.rom_startup_timing {
            return Err(OriginalTimingReceiptInstallError::TimingDisabled);
        }
        record_recent_host_receipt_vector(format!(
            "host_call={} gates={:?} pub_pending={} fc={:02x} frame={:02x}/{:02x}/{:02x} work={:?} semantic={:?}",
            receipts.host_call,
            self.original_timing_expected_nmi_update_gates,
            self.original_timing_nmi_publication_pending,
            self.game_state.frame.frame_counter,
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
            self.game_state.frame.subsubmodule,
            self.game_execution_scheduler.current_work(),
            receipts.semantic,
        ));
        if let Ok(range) = crate::debug_env::var("ZELDA3_DEBUG_INSTALL_RECEIPTS") {
            let mut parts = range.split('-');
            let lo: u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            let hi: u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(lo);
            if receipts.host_call >= lo && receipts.host_call <= hi {
                eprintln!(
                    "[RCPT] host_call={} gates={:?} pub_pending={} radius={} y={} bg2v={} fc={} work={:?} semantic={:?}",
                    receipts.host_call,
                    self.original_timing_expected_nmi_update_gates,
                    self.original_timing_nmi_publication_pending,
                    self.game_state.display.spotlight_hdma.window_radius(),
                    self.game_state.player.follower_link.y(),
                    self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
                    self.game_state.frame.frame_counter,
                    self.game_execution_scheduler.current_work(),
                    receipts.semantic,
                );
            }
        }
        if self.original_timing_host_dispatch_active {
            return Err(OriginalTimingReceiptInstallError::ActiveHostDispatch);
        }
        if self
            .original_timing_sprite_main_return_claims_remaining
            .is_some()
        {
            return Err(OriginalTimingReceiptInstallError::ActiveSpriteMainReturnClaim);
        }
        if self.original_timing_semantic_receipts.is_some() {
            return Err(OriginalTimingReceiptInstallError::ReceiptAlreadyInstalled);
        }
        if self.original_timing_presented_audio.is_some() {
            return Err(OriginalTimingReceiptInstallError::UnconsumedPresentedAudio);
        }
        let nmi_acceptance_count = receipts
            .semantic
            .iter()
            .filter(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_)))
            .count();
        if !receipts.nmi_acceptance_ppu_register_operands.is_empty()
            && receipts.nmi_acceptance_ppu_register_operands.len() != nmi_acceptance_count
        {
            return Err(OriginalTimingReceiptInstallError::InvalidNmiPpuRegisterOperands);
        }
        if let Some(expected) = self
            .original_timing_last_oracle_host_call
            .map(|host_call| host_call.wrapping_add(1))
        {
            if receipts.host_call != expected {
                return Err(OriginalTimingReceiptInstallError::OutOfSequence {
                    expected,
                    actual: receipts.host_call,
                });
            }
        }
        // A host that begins inside the previous iteration's common suffix
        // (before its latch-clear store) publishes that carried completion
        // as a leading [CallStackContinued, MainLoopCommonSuffixCompleted]
        // pair before its own fresh iteration (route host 511525). Strip the
        // pair into a retained fact; the host body retires the pending suffix
        // before anything else runs.
        let mut receipts = receipts;
        if receipts.semantic.len() > 2
            && receipts.semantic[0]
                == OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                )
            && receipts.semantic[1] == OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
            && receipts.semantic[2..].iter().any(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::IterationStarted
                    )
                )
            })
        {
            // The atomic runtime may already have run that suffix at the end
            // of the previous host (the latch cleared there, so the leading
            // NMI is Open on both sides); only a still-pending suffix needs
            // retiring first.
            self.original_timing_carried_suffix_completion_pending =
                self.pending_main_loop_common_suffix.is_some();
            receipts.semantic.drain(0..2);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::MainLoopProgress(_)))
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateMainLoopProgress);
        }
        self.original_timing_host_iteration_uninterrupted = {
            let progress_index = receipts.semantic.iter().position(|receipt| {
                matches!(receipt, OriginalTimingSemanticReceipt::MainLoopProgress(_))
            });
            let suffix_index = receipts.semantic.iter().position(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                )
            });
            match (progress_index, suffix_index) {
                (Some(progress), Some(suffix)) if progress < suffix => {
                    !receipts.semantic[progress..suffix].iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::NmiAccepted(_)
                                | OriginalTimingSemanticReceipt::MainLoopInterrupted(_)
                        )
                    })
                }
                _ => false,
            }
        };
        let main_loop_common_suffix_completions = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| {
                original_timing_receipt_completes_main_loop_common_suffix(receipt).then_some(index)
            })
            .collect::<Vec<_>>();
        if main_loop_common_suffix_completions.len() > 1 {
            return Err(OriginalTimingReceiptInstallError::DuplicateMainLoopIterationReturn);
        }
        if let Some(completion_index) = main_loop_common_suffix_completions.first().copied() {
            let progress = receipts
                .semantic
                .iter()
                .enumerate()
                .filter_map(|(index, receipt)| match receipt {
                    OriginalTimingSemanticReceipt::MainLoopProgress(progress) => {
                        Some((index, *progress))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let continued_suffix_owner_is_valid = match self
                .game_execution_scheduler
                .pre_main_caller_continuation()
            {
                Some(
                    PreMainCallerContinuation::FileSelectCheckerboardUpload
                    | PreMainCallerContinuation::NamePlayerTilemapUpload,
                ) => {
                    self.pending_main_loop_common_suffix
                        == Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
                }
                _ => {
                    self.pending_main_loop_common_suffix.is_some()
                        // A suspended dungeon quadrant-upload caller resumed
                        // by the leading NMI owns its own suffix completion
                        // (route host 925559: [Held, Handler, Continued,
                        // SuffixCompleted, Open]).
                        || matches!(
                            self.game_execution_scheduler.pre_main_nmi_resume(),
                            Some(
                                PreMainNmiResume::DungeonSupertileQuadrantUploads
                                    | PreMainNmiResume::DungeonSupertileQuadrantUploadsAfterHeldNmi
                            )
                        )
                }
            };
            let valid_owner = match progress.as_slice() {
                [(progress_index, crate::MainLoopProgress::CallStackContinued)] => {
                    *progress_index < completion_index && continued_suffix_owner_is_valid
                }
                [(progress_index, crate::MainLoopProgress::IterationStarted)] => {
                    *progress_index < completion_index
                        && (self.pending_main_loop_common_suffix.is_none()
                            // The carried completion stripped above retires
                            // that pending suffix before this iteration runs.
                            || self.original_timing_carried_suffix_completion_pending)
                }
                _ => false,
            };
            let incompatible_interruption = receipts.semantic.iter().any(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(_)
                )
            });
            let mut nmi_phases_before_completion = Vec::new();
            let mut nmi_phases_after_completion = Vec::new();
            for (index, receipt) in receipts.semantic.iter().enumerate() {
                let phase = match receipt {
                    OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                        OriginalTimingNmiPhase::Accepted(*gate)
                    }
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted => {
                        OriginalTimingNmiPhase::HandlerCompleted
                    }
                    _ => continue,
                };
                if index < completion_index {
                    nmi_phases_before_completion.push(phase);
                } else {
                    nmi_phases_after_completion.push(phase);
                }
            }
            let before_completion = try_classify_original_timing_nmi_phases(
                self.original_timing_nmi_publication_pending,
                &nmi_phases_before_completion,
            );
            let after_completion =
                try_classify_original_timing_nmi_phases(false, &nmi_phases_after_completion);
            let pending_at_completion = before_completion
                .map(|classification| classification.publication_pending_at_exit)
                .unwrap_or(true);
            if !valid_owner
                || incompatible_interruption
                || pending_at_completion
                || after_completion.is_none()
            {
                eprintln!(
                    "[RCPT-REJECT] host={} valid_owner={valid_owner} incompatible_interruption={incompatible_interruption} pending_at_completion={pending_at_completion} after_completion_none={} pending_suffix={:?} publication_pending={} scheduler={:?}",
                    self.frame_ctr_dbg,
                    after_completion.is_none(),
                    self.pending_main_loop_common_suffix,
                    self.original_timing_nmi_publication_pending,
                    self.game_execution_scheduler,
                );
                return Err(OriginalTimingReceiptInstallError::InvalidMainLoopIterationReturn);
            }
        }
        let sprite_main_progress = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::SpriteMainProgressed(progress) => Some(*progress),
                _ => None,
            })
            .collect::<Vec<_>>();
        if sprite_main_progress.len() > 1 {
            return Err(OriginalTimingReceiptInstallError::DuplicateSpriteMainProgress);
        }
        if sprite_main_progress
            .iter()
            .any(|progress| !valid_sprite_main_progress(*progress))
        {
            return Err(OriginalTimingReceiptInstallError::InvalidSpriteMainProgress);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::SpriteMainReturned))
            .count()
            > 2
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateSpriteMainReturn);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::PreDungeonModuleReturned
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicatePreDungeonModuleReturn);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(_)
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateDungeonResetProgress);
        }
        if receipts.semantic.iter().any(|receipt| match receipt {
            OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(receipt) => {
                match receipt.progress {
                    DungeonResetSpritesCpuProgress::Cache { slot, .. } => slot >= 16,
                    DungeonResetSpritesCpuProgress::Disable(
                        DungeonSpriteDisableCpuProgress::SpriteStatesThrough { slot },
                    ) => slot >= 16,
                    DungeonResetSpritesCpuProgress::Disable(
                        DungeonSpriteDisableCpuProgress::AncillasThrough { slot },
                    ) => slot >= 10,
                    DungeonResetSpritesCpuProgress::Disable(
                        DungeonSpriteDisableCpuProgress::AncillaPickupFlagCleared
                        | DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
                    )
                    | DungeonResetSpritesCpuProgress::SpritesDisabled
                    | DungeonResetSpritesCpuProgress::CollisionXSizeSet
                    | DungeonResetSpritesCpuProgress::LoadStarted
                    | DungeonResetSpritesCpuProgress::LoadBeforeOrigin
                    | DungeonResetSpritesCpuProgress::RoomHistorySearchStarted => false,
                    DungeonResetSpritesCpuProgress::Load(progress) => progress.slot >= 16,
                }
            }
            _ => false,
        }) {
            return Err(OriginalTimingReceiptInstallError::InvalidDungeonResetProgress);
        }
        let sprite_reset_all_progress = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::SpriteResetAllProgress(receipt) => Some(*receipt),
                _ => None,
            })
            .collect::<Vec<_>>();
        if sprite_reset_all_progress.len() > 1 {
            return Err(OriginalTimingReceiptInstallError::DuplicateSpriteResetAllProgress);
        }
        let cached_sprite_progress = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(receipt) => {
                    Some(*receipt)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if cached_sprite_progress.len() > 1 {
            return Err(OriginalTimingReceiptInstallError::DuplicateCachedSpriteExecutionProgress);
        }
        if cached_sprite_progress
            .iter()
            .any(|receipt| match receipt.progress {
                crate::CachedSpriteExecutionProgress::Loading {
                    slot,
                    copied_fields,
                } => slot >= 16 || copied_fields > 24,
                crate::CachedSpriteExecutionProgress::Executing { slot, .. } => slot >= 16,
                crate::CachedSpriteExecutionProgress::Restoring { slot, live_fields } => {
                    slot >= 16 || live_fields > 24
                }
            })
        {
            return Err(OriginalTimingReceiptInstallError::InvalidCachedSpriteExecutionProgress);
        }
        let peg_attribute_flip_progress = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(receipt) => {
                    Some(*receipt)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if peg_attribute_flip_progress.len() > 1 {
            return Err(
                OriginalTimingReceiptInstallError::DuplicateDungeonPegAttributeFlipProgress,
            );
        }
        if peg_attribute_flip_progress.iter().any(|receipt| {
            (receipt.index > 0x07ff && receipt.index != 0xffff)
                || receipt.completed_banks > 4
                || (receipt.index == 0xffff && receipt.completed_banks != 0)
        }) {
            return Err(OriginalTimingReceiptInstallError::InvalidDungeonPegAttributeFlipProgress);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::DialogueExecutionProgress(_)
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateDialogueExecutionProgress);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::SaveMenuInitializationProgress(_)
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateSaveMenuInitializationProgress);
        }
        let item_receipt_progress = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(progress) => {
                    Some(*progress)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if item_receipt_progress.len() > 1 {
            return Err(OriginalTimingReceiptInstallError::DuplicateItemReceiptGraphicsProgress);
        }
        if item_receipt_progress
            .iter()
            .any(|receipt| match receipt.caller {
                ItemReceiptGraphicsCaller::SpriteMain { slot }
                | ItemReceiptGraphicsCaller::SpriteMainDirect { slot }
                | ItemReceiptGraphicsCaller::UnclePassage { slot } => slot >= 16,
                ItemReceiptGraphicsCaller::SpriteMainAncilla { slot } => slot >= 10,
            })
        {
            return Err(OriginalTimingReceiptInstallError::InvalidItemReceiptGraphicsProgress);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::DialogueClosed))
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateDialogueClosed);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(_)
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateMainLoopInterruption);
        }
        if receipts.semantic.iter().any(|receipt| match receipt {
            OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption) => {
                !valid_sprite_main_interruption(*interruption)
            }
            _ => false,
        }) {
            return Err(OriginalTimingReceiptInstallError::InvalidMainLoopInterruption);
        }
        if let Some(interruption_index) = receipts.semantic.iter().position(|receipt| {
            matches!(
                receipt,
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }
                )
            )
        }) {
            let accepted_nmi_owns_interruption = interruption_index
                .checked_sub(1)
                .and_then(|index| receipts.semantic.get(index))
                .is_some_and(|receipt| {
                    matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld)
                    )
                });
            let progress = receipts.semantic.iter().find_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::MainLoopProgress(progress) => Some(*progress),
                _ => None,
            });
            let progress_owns_source_caller = match progress {
                Some(crate::MainLoopProgress::IterationStarted) => true,
                Some(crate::MainLoopProgress::CallStackContinued) => matches!(
                    self.pending_main_loop_common_suffix,
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
                ),
                None => false,
            };
            if !accepted_nmi_owns_interruption || !progress_owns_source_caller {
                return Err(OriginalTimingReceiptInstallError::InvalidMainLoopInterruption);
            }
        }
        let spotlight_progress = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(receipt) => {
                    Some(*receipt)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if spotlight_progress.len() > 1 {
            return Err(OriginalTimingReceiptInstallError::DuplicateSpotlightTableBuildProgress);
        }
        if spotlight_progress.iter().any(|receipt| {
            !matches!(
                receipt.boundary,
                OriginalTimingBoundary::NmiAccepted | OriginalTimingBoundary::HostReturn
            ) || match receipt.progress.checkpoint {
                crate::SpotlightTableBuildCheckpoint::ProjectionCopy { .. } => {
                    receipt.progress.completed_iterations > 240
                }
                _ => receipt.progress.completed_iterations >= 240,
            } || matches!(
                receipt.progress.checkpoint,
                crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                    pending_circle_input: 0,
                }
            ) || matches!(
                receipt.progress.checkpoint,
                crate::SpotlightTableBuildCheckpoint::BeforeLowerTableWrite {
                    lower_cursor,
                    ..
                } if lower_cursor >= 224
            ) || matches!(
                receipt.progress.checkpoint,
                crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words }
                    if copied_words > 224
            )
        }) {
            return Err(OriginalTimingReceiptInstallError::InvalidSpotlightTableBuildProgress);
        }
        let overworld_sprite_progress = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(progress) => {
                    Some(*progress)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if overworld_sprite_progress
            .iter()
            .filter(|progress| {
                matches!(
                    progress,
                    crate::OverworldSpriteReloadProgress::PresencePublished
                )
            })
            .count()
            > 1
        {
            return Err(
                OriginalTimingReceiptInstallError::DuplicateOverworldSpritePresencePublished,
            );
        }
        if overworld_sprite_progress
            .iter()
            .filter(|progress| {
                matches!(
                    progress,
                    crate::OverworldSpriteReloadProgress::ReloadReturned
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::InvalidOverworldSpriteReloadProgress);
        }
        if overworld_sprite_progress
            .iter()
            .filter(|progress| {
                matches!(
                    progress,
                    crate::OverworldSpriteReloadProgress::GenerationReturned
                        | crate::OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveCleanup { .. }
                        | crate::OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveTypeClear { .. }
                        | crate::OverworldSpriteReloadProgress::GenerationReturnedAtPortalReset { .. }
                        | crate::OverworldSpriteReloadProgress::GenerationReturnedAtPortalLoadProperties { .. }
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::InvalidOverworldSpriteReloadProgress);
        }
        // One host interval may accept more than one NMI while the source
        // proximity scan remains suspended. Preserve every ordered scratch
        // cursor publication: the first may restate the entry cursor before
        // the resumed loop advances to the second.
        if overworld_sprite_progress
            .iter()
            .any(|progress| {
                match progress {
                crate::OverworldSpriteReloadProgress::GenerationReturnedAtPortalReset { slot, completed_stores } => *slot >= 16 || *completed_stores > 40,
                crate::OverworldSpriteReloadProgress::GenerationReturnedAtPortalLoadProperties { slot, completed_stores } => *slot >= 16 || *completed_stores > 10,
                crate::OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveCleanup {
                    slot,
                } | crate::OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveTypeClear {
                    slot,
                } => *slot >= 6,
                crate::OverworldSpriteReloadProgress::PresencePublished
                | crate::OverworldSpriteReloadProgress::ReloadReturned
                | crate::OverworldSpriteReloadProgress::GenerationReturned
                | crate::OverworldSpriteReloadProgress::ProximityScanSuspended { .. } => false,
                crate::OverworldSpriteReloadProgress::SpriteActivated {
                    block,
                    slot,
                    sprite_type,
                } => *block >= 0x1000 || *slot >= 16 || *sprite_type >= 0xf3,
            }
            })
        {
            return Err(OriginalTimingReceiptInstallError::InvalidOverworldSpriteReloadProgress);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                **receipt == OriginalTimingSemanticReceipt::SaveQuitResetStatePublished
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateSaveQuitResetStatePublished);
        }
        let file_select_low_wram_publications =
            receipts
                .semantic
                .iter()
                .filter_map(|receipt| match receipt {
                    OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramClearProgress(
                        progress,
                    ) => Some(OriginalTimingFileSelectLowWramPublication::Progress(
                        *progress,
                    )),
                    OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramCleared => {
                        Some(OriginalTimingFileSelectLowWramPublication::Complete)
                    }
                    _ => None,
                });
        if file_select_low_wram_publications.clone().count() > 1 {
            return Err(
                OriginalTimingReceiptInstallError::DuplicateFileSelectGraphicsLowWramCleared,
            );
        }
        if file_select_low_wram_publications
            .into_iter()
            .any(|publication| {
                matches!(
                    publication,
                    OriginalTimingFileSelectLowWramPublication::Progress(progress)
                        if progress.word_offset & 1 != 0
                            || progress.completed_page_stores == 0
                            || progress.completed_page_stores > 3
                )
            })
        {
            return Err(
                OriginalTimingReceiptInstallError::InvalidFileSelectGraphicsLowWramClearProgress,
            );
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                **receipt
                    == OriginalTimingSemanticReceipt::SelectedGameLoadMessageInterfacePublished
            })
            .count()
            > 1
        {
            return Err(
                OriginalTimingReceiptInstallError::DuplicateSelectedGameLoadMessageInterfacePublished,
            );
        }
        let triforce_case2_palette_progress =
            receipts
                .semantic
                .iter()
                .filter_map(|receipt| match receipt {
                    OriginalTimingSemanticReceipt::TriforceRoomCase2PaletteProgress(progress) => {
                        Some(*progress)
                    }
                    _ => None,
                });
        if triforce_case2_palette_progress.clone().count() > 1 {
            return Err(
                OriginalTimingReceiptInstallError::DuplicateTriforceRoomCase2PaletteProgress,
            );
        }
        if triforce_case2_palette_progress
            .into_iter()
            .any(|progress| progress.completed_ow_bg2_words > 21)
        {
            return Err(OriginalTimingReceiptInstallError::InvalidTriforceRoomCase2PaletteProgress);
        }
        let credits_scene_load_progress =
            receipts
                .semantic
                .iter()
                .filter_map(|receipt| match receipt {
                    OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(progress) => {
                        Some(*progress)
                    }
                    _ => None,
                });
        if credits_scene_load_progress.clone().count() > 1 {
            return Err(OriginalTimingReceiptInstallError::DuplicateCreditsSceneLoadProgress);
        }
        if credits_scene_load_progress.into_iter().any(|progress| {
            matches!(
                progress.progress,
                crate::CreditsSceneLoadProgress::EndingTextPayloadBytes(bytes) if bytes & 1 != 0
            )
        }) {
            return Err(OriginalTimingReceiptInstallError::InvalidCreditsSceneLoadProgress);
        }
        let credits_end_sequence_32_progress =
            receipts
                .semantic
                .iter()
                .filter_map(|receipt| match receipt {
                    OriginalTimingSemanticReceipt::CreditsEndSequence32Progress(progress) => {
                        Some(*progress)
                    }
                    _ => None,
                });
        if credits_end_sequence_32_progress.clone().count() > 1 {
            return Err(OriginalTimingReceiptInstallError::DuplicateCreditsEndSequence32Progress);
        }
        if credits_end_sequence_32_progress
            .into_iter()
            .any(|progress| progress.completed_checksum_words > 0x4fe / 2)
        {
            return Err(OriginalTimingReceiptInstallError::InvalidCreditsEndSequence32Progress);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::PreOverworldStageCompleted(_)
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicatePreOverworldStageCompletion);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::DungeonFallingEntranceProgress(_)
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateDungeonFallingEntranceProgress);
        }
        let rescued_maiden_clear_progress = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::RescuedMaidenTilemapClearProgress(progress) => {
                    Some(*progress)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if rescued_maiden_clear_progress.len() > 1 {
            return Err(
                OriginalTimingReceiptInstallError::DuplicateRescuedMaidenTilemapClearProgress,
            );
        }
        if rescued_maiden_clear_progress
            .iter()
            .any(|progress| progress.completed_stores > 8192)
        {
            return Err(
                OriginalTimingReceiptInstallError::InvalidRescuedMaidenTilemapClearProgress,
            );
        }
        let rescued_maiden_initialization_progress = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::RescuedMaidenInitializationProgress(progress) => {
                    Some(*progress)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if rescued_maiden_initialization_progress.len() > 1 {
            return Err(
                OriginalTimingReceiptInstallError::DuplicateRescuedMaidenInitializationProgress,
            );
        }
        if rescued_maiden_initialization_progress
            .iter()
            .any(|progress| {
                let within_source_range = match progress.stage {
                    crate::RescuedMaidenInitializationStage::FirstFollowerSheet {
                        completed_bytes,
                    }
                    | crate::RescuedMaidenInitializationStage::SecondFollowerSheet {
                        completed_bytes,
                    } => completed_bytes <= 0x0600,
                    crate::RescuedMaidenInitializationStage::Conversion { completed_stores } => {
                        completed_stores <= 512
                    }
                };
                !within_source_range || progress.boundary != OriginalTimingBoundary::HostReturn
            })
        {
            return Err(
                OriginalTimingReceiptInstallError::InvalidRescuedMaidenInitializationProgress,
            );
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::OverworldMapQuadrantsPublished
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateOverworldMapQuadrantsPublished);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::WorldMapOverlayReloadReturned
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateWorldMapOverlayReloadReturn);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::WorldMapAmbientMap8Returned
                )
            })
            .count()
            > 1
        {
            return Err(OriginalTimingReceiptInstallError::DuplicateWorldMapAmbientMap8Return);
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned
                )
            })
            .count()
            > 1
        {
            return Err(
                OriginalTimingReceiptInstallError::DuplicateDungeonExitSpotlightEntryReturn,
            );
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait
                )
            })
            .count()
            > 1
        {
            return Err(
                OriginalTimingReceiptInstallError::DuplicateDungeonExitSpotlightCallerReturn,
            );
        }
        if receipts
            .semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned
                )
            })
            .count()
            > 1
        {
            return Err(
                OriginalTimingReceiptInstallError::DuplicateOverworldSpotlightGoalCallerReturn,
            );
        }
        let mut pending_nmi_gate = self.original_timing_pending_nmi_update_gate;
        let mut completed_nmi_gate = None;
        for receipt in &receipts.semantic {
            if completed_nmi_gate == Some(NmiUpdateGate::Open)
                && !matches!(receipt, OriginalTimingSemanticReceipt::JoypadPublication(_))
            {
                return Err(OriginalTimingReceiptInstallError::InvalidNmiLifecycle);
            }
            match receipt {
                OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                    if pending_nmi_gate.replace(*gate).is_some() {
                        return Err(OriginalTimingReceiptInstallError::InvalidNmiLifecycle);
                    }
                    completed_nmi_gate = None;
                }
                OriginalTimingSemanticReceipt::NmiHandlerCompleted => {
                    let Some(gate) = pending_nmi_gate.take() else {
                        return Err(OriginalTimingReceiptInstallError::InvalidNmiLifecycle);
                    };
                    completed_nmi_gate = (gate == NmiUpdateGate::Open).then_some(gate);
                }
                OriginalTimingSemanticReceipt::JoypadPublication(_) => {
                    if completed_nmi_gate.take() != Some(NmiUpdateGate::Open) {
                        return Err(OriginalTimingReceiptInstallError::InvalidNmiLifecycle);
                    }
                }
                _ => {}
            }
        }
        if completed_nmi_gate == Some(NmiUpdateGate::Open) {
            return Err(OriginalTimingReceiptInstallError::InvalidNmiLifecycle);
        }
        let mut expected_nmi_update_gates = Vec::new();
        let mut expected_nmi_ppu_register_operands = Vec::new();
        if self.original_timing_nmi_publication_pending {
            expected_nmi_update_gates.push(
                self.original_timing_pending_nmi_update_gate
                    .expect("pending source NMI lost its update-gate disposition"),
            );
            expected_nmi_ppu_register_operands
                .push(self.original_timing_pending_nmi_ppu_register_operands);
        }
        expected_nmi_update_gates.extend(receipts.semantic.iter().filter_map(
            |receipt| match receipt {
                OriginalTimingSemanticReceipt::NmiAccepted(gate) => Some(*gate),
                _ => None,
            },
        ));
        if receipts.nmi_acceptance_ppu_register_operands.is_empty() {
            expected_nmi_ppu_register_operands
                .extend(std::iter::repeat_n(None, nmi_acceptance_count));
        } else {
            expected_nmi_ppu_register_operands.extend(
                receipts
                    .nmi_acceptance_ppu_register_operands
                    .iter()
                    .copied()
                    .map(Some),
            );
        }
        self.original_timing_expected_nmi_update_gates = expected_nmi_update_gates;
        self.original_timing_expected_nmi_ppu_register_operands =
            expected_nmi_ppu_register_operands;
        self.original_timing_semantic_receipts = Some(receipts);
        Ok(())
    }

    pub(super) fn validate_next_original_timing_nmi_update_gate(&mut self) {
        if !self.original_timing_host_dispatch_active
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            return;
        }
        let expected = self
            .original_timing_expected_nmi_update_gates
            .first()
            .copied()
            .expect("live timing host executed an NMI absent from its typed authority receipt");
        let actual = if self.game_state.display.nmi_update_is_latched() {
            NmiUpdateGate::LatchHeld
        } else {
            NmiUpdateGate::Open
        };
        assert_eq!(
            actual,
            expected,
            "native Zelda NMI latch disagreed with the source acceptance disposition: host={:?} frame={:?} pending_suffix={:?} scheduler={:?}",
            self.original_timing_last_oracle_host_call,
            self.game_state.frame,
            self.pending_main_loop_common_suffix,
            self.game_execution_scheduler,
        );
        self.original_timing_expected_nmi_update_gates.remove(0);
        self.original_timing_expected_nmi_ppu_register_operands
            .remove(0);
    }

    /// The slot of a `Sprite_Main`-direct item receipt the live wire reports
    /// suspended inside this host, without consuming the receipt.
    pub(super) fn original_timing_direct_item_receipt_suspended_slot(&self) -> Option<u8> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        self.original_timing_semantic_receipts
            .as_ref()?
            .semantic()
            .iter()
            .find_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(progress) => {
                    match progress.caller {
                        ItemReceiptGraphicsCaller::SpriteMainDirect { slot }
                            if progress.progress == SourceCallProgress::Suspended =>
                        {
                            Some(slot)
                        }
                        _ => None,
                    }
                }
                _ => None,
            })
    }

    /// The wire suspended the falling milestone item's receipt inside
    /// Sprite_Main's prefix (route host 1142850): the loop must run natively
    /// so the ancilla's receipt call suspends at the exact statement.
    pub(super) fn original_timing_ancilla_item_receipt_suspended(&self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        self.original_timing_semantic_receipts
            .as_ref()
            .is_some_and(|receipts| {
                receipts.semantic().iter().any(|receipt| {
                    matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                            ItemReceiptGraphicsProgressReceipt {
                                caller: ItemReceiptGraphicsCaller::SpriteMainAncilla { .. },
                                progress: SourceCallProgress::Suspended,
                            }
                        )
                    )
                })
            })
    }

    pub(super) fn take_original_timing_item_receipt_graphics_progress(
        &mut self,
    ) -> Option<ItemReceiptGraphicsProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple item-receipt call outcomes",
        );
        matches.first().map(|&(index, receipt)| {
            receipts.semantic.remove(index);
            receipt
        })
    }

    pub(super) fn take_original_timing_preemptive_polyhedral_render_started(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::PreemptivePolyhedralRenderStarted
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one source host cannot start multiple preemptive poly renders",
        );
        matches.first().is_some_and(|&index| {
            receipts.semantic.remove(index);
            true
        })
    }

    pub(super) fn original_timing_dungeon_reset_sprites_progress_pending(&self) -> bool {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic().iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(_)
                        )
                    })
                })
    }

    pub(super) fn take_original_timing_dungeon_reset_sprites_progress(
        &mut self,
    ) -> Option<DungeonResetSpritesProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(receipt) => {
                    Some((index, *receipt))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple Dungeon_ResetSprites interruption points",
        );
        matches.first().map(|&(index, receipt)| {
            receipts.semantic.remove(index);
            receipt
        })
    }

    pub(super) fn take_original_timing_sprite_reset_all_progress(
        &mut self,
    ) -> Option<SpriteResetAllProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::SpriteResetAllProgress(receipt) => {
                    Some((index, *receipt))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple Sprite_ResetAll interruption points",
        );
        matches.first().map(|&(index, receipt)| {
            receipts.semantic.remove(index);
            receipt
        })
    }

    pub(super) fn take_original_timing_cached_sprite_execution_progress(
        &mut self,
    ) -> Option<CachedSpriteExecutionProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(receipt) => {
                    Some((index, *receipt))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple cached-sprite interruption points",
        );
        matches.first().map(|&(index, receipt)| {
            receipts.semantic.remove(index);
            receipt
        })
    }

    pub(super) fn take_original_timing_dungeon_peg_attribute_flip_progress(
        &mut self,
    ) -> Option<crate::DungeonPegAttributeFlipProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(receipt) => {
                    Some((index, *receipt))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple peg-attribute flip checkpoints",
        );
        matches.first().map(|&(index, receipt)| {
            receipts.semantic.remove(index);
            receipt
        })
    }

    pub(super) fn original_timing_dungeon_peg_attribute_flip_progress(
        &self,
    ) -> Option<crate::DungeonPegAttributeFlipProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        self.original_timing_semantic_receipts
            .as_ref()?
            .semantic
            .iter()
            .find_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(receipt) => {
                    Some(*receipt)
                }
                _ => None,
            })
    }

    pub(super) fn original_timing_cached_sprite_execution_progress(
        &self,
    ) -> Option<CachedSpriteExecutionProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        self.original_timing_semantic_receipts
            .as_ref()?
            .semantic
            .iter()
            .find_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(receipt) => {
                    Some(*receipt)
                }
                _ => None,
            })
    }

    pub(super) fn take_original_timing_main_loop_interruption_any(
        &mut self,
    ) -> Option<crate::MainLoopInterruption> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::MainLoopInterrupted(phase) => Some((index, *phase)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple main-loop interruption phases",
        );
        matches.first().map(|&(index, phase)| {
            receipts.semantic.remove(index);
            phase
        })
    }

    pub(super) fn original_timing_main_loop_interruption(
        &self,
    ) -> Option<crate::MainLoopInterruption> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_ref()?;
        if let Some(forwarded) = receipts.forwarded_main_loop_interruption() {
            return Some(forwarded.interruption());
        }
        let matches = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption) => {
                    Some(*interruption)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple main-loop interruption phases",
        );
        matches.first().copied()
    }

    pub(super) fn take_original_timing_main_loop_interruption_timeline(
        &mut self,
        expected_interruption: Option<crate::MainLoopInterruption>,
    ) -> Option<OriginalTimingMainLoopInterruptionTimeline> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let publication_pending_at_entry = self.original_timing_nmi_publication_pending;
        let receptive_display_snapshot = self
            .display_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts);
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let interruptions = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| {
                let OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption) = receipt
                else {
                    return None;
                };
                expected_interruption
                    .is_none_or(|expected| expected == *interruption)
                    .then_some((index, *interruption))
            })
            .collect::<Vec<_>>();
        assert!(
            interruptions.len() <= 1,
            "one host call cannot publish multiple main-loop interruption phases",
        );
        let &(interruption_index, interruption) = interruptions.first()?;
        let progress = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::MainLoopProgress(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            progress.len(),
            1,
            "an interrupted main-loop host must publish exactly one call-progress outcome",
        );
        let (progress_index, progress) = progress[0];
        let mut nmi_phases_before_interruption = Vec::new();
        let mut nmi_phases_after_interruption = Vec::new();
        let mut consumed = vec![progress_index, interruption_index];
        for (index, receipt) in receipts.semantic.iter().enumerate() {
            let phase = match receipt {
                OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                    OriginalTimingNmiPhase::Accepted(*gate)
                }
                OriginalTimingSemanticReceipt::NmiHandlerCompleted => {
                    OriginalTimingNmiPhase::HandlerCompleted
                }
                _ => continue,
            };
            if index < interruption_index {
                nmi_phases_before_interruption.push(phase);
            } else {
                nmi_phases_after_interruption.push(phase);
            }
            consumed.push(index);
        }
        let phases = nmi_phases_before_interruption
            .iter()
            .chain(nmi_phases_after_interruption.iter())
            .copied()
            .collect::<Vec<_>>();
        if let Some(classification) =
            try_classify_original_timing_nmi_phases(publication_pending_at_entry, &phases)
        {
            assert_original_timing_carry_in_handler_has_receptive_display(
                classification,
                receptive_display_snapshot,
            );
        }
        consumed.sort_unstable();
        consumed.dedup();
        for index in consumed.into_iter().rev() {
            receipts.semantic.remove(index);
        }
        Some(OriginalTimingMainLoopInterruptionTimeline {
            progress,
            interruption,
            nmi_phases_before_interruption,
            nmi_phases_after_interruption,
        })
    }

    /// Forward a timeline-owned interruption to the translated source caller.
    ///
    /// Building [`OriginalTimingMainLoopInterruptionTimeline`] consumes the
    /// ordered host receipts so the outer loop can classify the NMI lifecycle.
    /// When that same host also begins a fresh `ZeldaRunGameLoop` iteration,
    /// the interruption belongs to the module call in that new iteration. Put
    /// only the backend-neutral phase back on the one-host receipt bus so the
    /// module's semantic continuation owner can stop at the C call boundary.
    /// CPU addresses and emulator state remain private to the authority.
    pub(super) fn forward_original_timing_main_loop_interruption_to_native_owner(
        &mut self,
        interruption: crate::MainLoopInterruption,
        boundary: OriginalTimingBoundary,
    ) {
        assert!(matches!(
            self.original_timing_owner,
            OriginalTimingOwnerState::Live
        ));
        let receipts = self
            .original_timing_semantic_receipts
            .as_mut()
            .expect("live timing authority omitted its current host receipt");
        receipts
            .forward_main_loop_interruption(interruption, boundary)
            .unwrap_or_else(|error| {
                panic!(
                    "one source host cannot forward two native interruption boundaries: existing={:?} replacement={interruption:?}/{boundary:?}",
                    error.existing(),
                )
            });
    }

    /// Consume a host-timeline interruption together with the hardware
    /// boundary which exposed it. `HostReturn` leaves the C stack suspended
    /// before interrupt acceptance; `NmiAccepted` resumes it next host.
    pub(super) fn take_forwarded_original_timing_main_loop_interruption(
        &mut self,
        phase: crate::MainLoopInterruption,
    ) -> Option<OriginalTimingBoundary> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        receipts.take_forwarded_main_loop_interruption(phase)
    }

    /// A dungeon fresh iteration interrupted inside NMI_PrepareSprites'
    /// extended OAM packing (route host 767104, group 28) resumes through
    /// the same caller-return continuation as a whole-prep interruption:
    /// the suspended stack's sprite state is unchanged when the packing
    /// completes at the return, so the prep is re-run whole there.
    pub(super) fn take_original_timing_extended_oam_packing_interruption_for_caller_return(
        &mut self,
    ) -> bool {
        let forwarded = self
            .original_timing_semantic_receipts
            .as_ref()
            .and_then(OriginalTimingHostReceipts::forwarded_main_loop_interruption)
            .map(|forwarded| forwarded.interruption());
        if let Some(
            interruption @ crate::MainLoopInterruption::SpritePreparationExtendedOamPacking {
                ..
            },
        ) = forwarded
        {
            return self
                .take_forwarded_original_timing_main_loop_interruption(interruption)
                .is_some();
        }
        match self.original_timing_main_loop_interruption() {
            Some(crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }) => self
                .take_original_timing_main_loop_interruption_any()
                .is_some(),
            _ => false,
        }
    }

    /// Consume a forwarded source-level `Sprite_Main` loop boundary without
    /// exposing the timing backend's PC, registers, or raster to gameplay.
    pub(super) fn take_original_timing_sprite_main_boundary_for_fresh_caller(
        &mut self,
    ) -> Option<(SpriteMainCpuBoundary, OriginalTimingBoundary)> {
        let forwarded = self
            .original_timing_semantic_receipts
            .as_ref()
            .and_then(OriginalTimingHostReceipts::forwarded_main_loop_interruption);
        if let Some(forwarded) = forwarded {
            let interruption = forwarded.interruption();
            let sprite_boundary = sprite_main_cpu_boundary_from_interruption(interruption)?;
            let boundary =
                self.take_forwarded_original_timing_main_loop_interruption(interruption)?;
            // The adapter may restate the interrupted statement as a paired
            // SpriteMainProgressed checkpoint in the same host vector. When it
            // names the identical C boundary, this fresh caller is its one
            // owner; a checkpoint naming a different statement (the
            // item-receipt AfterSlot restatement) belongs to the resumed
            // graphics caller instead.
            let paired_same_boundary =
                self.original_timing_semantic_receipts
                    .as_ref()
                    .is_some_and(|receipts| {
                        receipts.semantic.iter().any(|receipt| {
                            matches!(
                                receipt,
                                OriginalTimingSemanticReceipt::SpriteMainProgressed(progress)
                                    if sprite_main_cpu_boundary_from_progress(*progress)
                                        == sprite_boundary
                            )
                        })
                    });
            if paired_same_boundary {
                let paired = self
                    .take_original_timing_sprite_main_progress()
                    .expect("paired Sprite_Main checkpoint disappeared before consumption");
                assert_eq!(
                    paired, sprite_boundary,
                    "paired Sprite_Main checkpoint changed boundary before consumption",
                );
            }
            // When the host also restates a suspended item-receipt graphics
            // call started directly by the interrupted slot, the pair names
            // one C statement inside that call: the native body must enter
            // it and suspend there; the begin site consumes the claim.
            let suspended_direct_slot =
                self.original_timing_semantic_receipts
                    .as_ref()
                    .and_then(|receipts| {
                        receipts.semantic.iter().find_map(|receipt| match receipt {
                            OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                                progress,
                            ) if progress.progress == crate::SourceCallProgress::Suspended => {
                                match progress.caller {
                                    ItemReceiptGraphicsCaller::SpriteMainDirect { slot } => {
                                        Some(slot)
                                    }
                                    _ => None,
                                }
                            }
                            _ => None,
                        })
                    });
            if let Some(slot) = suspended_direct_slot {
                assert!(
                    direct_item_receipt_slot_pairs_with_boundary(slot, sprite_boundary),
                    "a suspended direct item-receipt claim disagrees with its interrupted Sprite_Main statement: slot={slot} boundary={sprite_boundary:?}",
                );
                return Some((
                    SpriteMainCpuBoundary::ItemReceiptGraphicsStarted(slot),
                    boundary,
                ));
            }
            return Some((sprite_boundary, boundary));
        }

        // SCAN_KEYS can return a libretro host while the current C call is
        // still inside Sprite_Main without accepting an NMI. The source
        // adapter publishes the furthest returned slot as SpriteMainProgressed
        // in that case. With no continuation active yet, this fresh translated
        // module caller is its sole semantic owner: stop at the same slot and
        // carry the remaining C suffix across the host return.
        let boundary = self.take_original_timing_sprite_main_progress()?;
        // When the same host also restates a suspended item-receipt graphics
        // call started by the source slot, the pair names one C statement:
        // the source is inside that slot's item-receipt call. Depending on
        // where the slot body entered the graphics helper, the outer tracker
        // can restate either the preceding completed slot or a prefix of the
        // same active slot. The shared pairing predicate owns both forms.
        let suspended_direct_slot =
            self.original_timing_semantic_receipts
                .as_ref()
                .and_then(|receipts| {
                    receipts.semantic.iter().find_map(|receipt| match receipt {
                        OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(progress)
                            if progress.progress == crate::SourceCallProgress::Suspended =>
                        {
                            match progress.caller {
                                ItemReceiptGraphicsCaller::SpriteMainDirect { slot } => Some(slot),
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                });
        if let Some(slot) = suspended_direct_slot {
            assert!(
                direct_item_receipt_slot_pairs_with_boundary(slot, boundary),
                "a suspended direct item-receipt claim disagrees with its Sprite_Main slot checkpoint: slot={slot} boundary={boundary:?}",
            );
            return Some((
                SpriteMainCpuBoundary::ItemReceiptGraphicsStarted(slot),
                OriginalTimingBoundary::HostReturn,
            ));
        }
        Some((boundary, OriginalTimingBoundary::HostReturn))
    }

    pub(super) fn original_timing_main_loop_interruption_is_pending(&self) -> bool {
        self.original_timing_semantic_receipts
            .as_ref()
            .is_some_and(|receipts| {
                receipts.forwarded_main_loop_interruption().is_some()
                    || receipts.semantic.iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::MainLoopInterrupted(_)
                        )
                    })
            })
    }

    /// The wire's unconsumed resumed-Sprite_Main checkpoint for this host, if
    /// any, without taking it.
    pub(super) fn original_timing_sprite_main_progress_boundary(
        &self,
    ) -> Option<SpriteMainCpuBoundary> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        self.original_timing_semantic_receipts
            .as_ref()?
            .semantic()
            .iter()
            .find_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::SpriteMainProgressed(progress) => {
                    Some(sprite_main_cpu_boundary_from_progress(*progress))
                }
                _ => None,
            })
    }

    pub(super) fn take_original_timing_sprite_main_progress(
        &mut self,
    ) -> Option<SpriteMainCpuBoundary> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::SpriteMainProgressed(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple resumed Sprite_Main progress facts",
        );
        let (index, progress) = matches.first().copied()?;
        receipts.semantic.remove(index);
        Some(sprite_main_cpu_boundary_from_progress(progress))
    }

    /// Whether the live wire still owes this host an unconsumed
    /// `SpriteMainReturned` claim — proof the shared `Sprite_Main` body
    /// The live wire began a fresh main-loop iteration in this host but owes
    /// no Sprite_Main return, no Sprite_Main checkpoint and no main-loop
    /// return: the ROM's iteration was interrupted inside Sprite_Main's shared
    /// prefix before any slot returned (route host 91639 with a trailing Held
    /// acceptance; route hosts 165775/186360 in Module0B and 508544/811514 in
    /// Module09 with the host simply ending mid-iteration). Returns the
    /// boundary the caller must suspend at: `NmiAccepted` when the host carries
    /// a trailing Held acceptance, `HostReturn` otherwise.
    pub(super) fn original_timing_fresh_iteration_interrupted_before_sprite_main(
        &self,
    ) -> Option<OriginalTimingBoundary> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || self.original_timing_semantic_receipts.is_none()
            || self.sprite_main_cpu_boundary.is_some()
            || self.original_timing_sprite_main_return_claims_remaining != Some(0)
            || self.original_timing_owes_sprite_main_return()
            || self.original_timing_owes_sprite_main_progress()
            || self.original_timing_main_loop_return_timeline().is_some()
            || self.original_timing_main_loop_iteration_returned_to_wait()
        {
            return None;
        }
        Some(
            if self
                .original_timing_expected_nmi_update_gates
                .contains(&NmiUpdateGate::LatchHeld)
            {
                OriginalTimingBoundary::NmiAccepted
            } else {
                OriginalTimingBoundary::HostReturn
            },
        )
    }

    /// returns later in this same host, so no native estimate may suspend
    /// the caller before that body runs.
    pub(super) fn original_timing_owes_sprite_main_return(&self) -> bool {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic().iter().any(|receipt| {
                        *receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                    })
                })
    }

    /// Whether the live wire still holds an unconsumed fresh-iteration
    /// progress receipt for this host. Sprite receipts in such a host belong
    /// to that iteration's own `Sprite_Main`, not to a resumed caller.
    pub(super) fn original_timing_hosts_fresh_iteration(&self) -> bool {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic().iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::MainLoopProgress(
                                crate::MainLoopProgress::IterationStarted
                            )
                        )
                    })
                })
    }

    /// Whether the live wire holds an unconsumed `SpriteMainProgressed`
    /// checkpoint for this host — proof the caller runs into (or through)
    /// the shared `Sprite_Main` body before the host ends, so no native
    /// estimate may suspend the caller ahead of that body.
    pub(super) fn original_timing_owes_sprite_main_progress(&self) -> bool {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic().iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::SpriteMainProgressed(_)
                        )
                    })
                })
    }

    /// Whether the live wire's unconsumed `SpriteMainProgressed` checkpoint
    /// (if any) names exactly the given suspended boundary — a re-statement
    /// of a held suspension rather than a new refinement.
    pub(super) fn original_timing_restates_sprite_main_checkpoint(
        &self,
        boundary: SpriteMainCpuBoundary,
    ) -> bool {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic().iter().any(|receipt| match receipt {
                        OriginalTimingSemanticReceipt::SpriteMainProgressed(progress) => {
                            same_sprite_main_source_checkpoint(
                                sprite_main_cpu_boundary_from_progress(*progress),
                                boundary,
                            )
                        }
                        _ => false,
                    })
                })
    }

    pub(super) fn take_original_timing_sprite_main_returned(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| {
                (*receipt == OriginalTimingSemanticReceipt::SpriteMainReturned).then_some(index)
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 2,
            "one host call cannot return from Sprite_Main more than twice",
        );
        let Some(index) = matches.first().copied() else {
            return false;
        };
        receipts.semantic.remove(index);
        true
    }

    pub(super) fn original_timing_uninterrupted_main_loop_timeline(
        &self,
        expected_progress: crate::MainLoopProgress,
    ) -> Option<OriginalTimingMainLoopTimeline> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_ref()?;
        let progress = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::MainLoopProgress(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            progress.len(),
            1,
            "one live host must publish exactly one main-loop progress receipt",
        );
        let (progress_index, progress) = progress[0];
        if progress != expected_progress {
            return None;
        }
        let module_body_owns_sprite_preparation_interruption =
            self.game_execution_scheduler.current_work().is_none();
        if receipts.semantic.iter().any(|receipt| {
            matches!(
                receipt,
                OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption)
                    // On an idle host the plain sprite-preparation
                    // interruption is consumed by the dungeon module body as
                    // its own suffix continuation; it does not name an outer
                    // host timeline. A scheduled caller's host keeps the
                    // interruption on its own timeline instead.
                    if *interruption != crate::MainLoopInterruption::SpritePreparation
                        || !module_body_owns_sprite_preparation_interruption
            )
        }) {
            return None;
        }
        let mut nmi_phases_before_progress = Vec::new();
        let mut nmi_phases_after_progress = Vec::new();
        for (index, receipt) in receipts.semantic.iter().enumerate() {
            let phase = match receipt {
                OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                    OriginalTimingNmiPhase::Accepted(*gate)
                }
                OriginalTimingSemanticReceipt::NmiHandlerCompleted => {
                    OriginalTimingNmiPhase::HandlerCompleted
                }
                _ => continue,
            };
            if index < progress_index {
                nmi_phases_before_progress.push(phase);
            } else {
                nmi_phases_after_progress.push(phase);
            }
        }
        Some(OriginalTimingMainLoopTimeline {
            progress,
            nmi_phases_before_progress,
            nmi_phases_after_progress,
        })
    }

    pub(super) fn take_original_timing_uninterrupted_main_loop_timeline(
        &mut self,
        expected_progress: crate::MainLoopProgress,
    ) -> Option<OriginalTimingMainLoopTimeline> {
        let timeline = self.original_timing_uninterrupted_main_loop_timeline(expected_progress)?;
        if let Some(classification) = try_classify_original_timing_nmi_phases(
            self.original_timing_nmi_publication_pending,
            &timeline.nmi_phases_before_progress,
        ) {
            assert_original_timing_carry_in_handler_has_receptive_display(
                classification,
                self.display_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
            );
        }
        let receipts = self
            .original_timing_semantic_receipts
            .as_mut()
            .expect("the inspected live main-loop timeline disappeared before consumption");
        receipts.semantic.retain(|receipt| {
            !matches!(
                receipt,
                OriginalTimingSemanticReceipt::MainLoopProgress(_)
                    | OriginalTimingSemanticReceipt::NmiAccepted(_)
                    | OriginalTimingSemanticReceipt::NmiHandlerCompleted
            )
        });
        Some(timeline)
    }

    pub(super) fn original_timing_interrupted_idle_main_loop_plan(
        &self,
    ) -> Option<OriginalTimingInterruptedIdleMainLoopPlan> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let interruption = self.original_timing_main_loop_interruption()?;
        let scheduled_predecessor = self.game_execution_scheduler.current_work().map(|work| {
            assert!(
                work.can_return_directly_to_same_host_fresh_iteration(),
                "scheduled work cannot return directly into a source-proven same-host fresh iteration: {work:?}",
            );
            assert!(
                interruption == crate::MainLoopInterruption::LinkOam
                    || interruption == crate::MainLoopInterruption::SpritePreparation
                    || matches!(
                        interruption,
                        crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }
                    )
                    || interruption.is_sprite_main(),
                "a scheduled caller has no native same-host fresh-iteration owner for {interruption:?}: {work:?}",
            );
            assert!(
                self.game_execution_scheduler
                    .work_suspends_translated_call_stack(),
                "a same-host fresh iteration cannot follow non-suspended scheduled work",
            );
            let mut scheduler_before_step = self.game_execution_scheduler;
            scheduler_before_step.begin_host_frame();
            assert_eq!(scheduler_before_step.current_work(), Some(work));
            let mut scheduler_after_step = scheduler_before_step;
            let probed_step = if work.same_host_fresh_iteration_completion_is_authoritative() {
                scheduler_after_step
                    .advance_work_one_nmi_slice_with_authoritative_completion(true)
            } else {
                scheduler_after_step.advance_work_one_nmi_slice()
            };
            assert_eq!(
                probed_step,
                Some(GameWorkStep::Complete(work)),
                "same-host fresh-iteration authority did not complete its scheduled predecessor",
            );
            assert!(
                scheduler_after_step.is_idle(),
                "same-host fresh-iteration predecessor retained scheduled work after completion",
            );
            OriginalTimingScheduledFreshIterationPredecessor {
                work,
                scheduler_before_step,
                scheduler_after_step,
            }
        });
        if scheduled_predecessor.is_none() {
            assert!(
                interruption == crate::MainLoopInterruption::LinkOam
                    || matches!(
                        interruption,
                        crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }
                            // Module0F's spotlight-close Link movement
                            // consumes this boundary inside the module body
                            // (route hosts 50635, 179586).
                            | crate::MainLoopInterruption::DungeonExitSpotlightAfterSubmodule
                            | crate::MainLoopInterruption::LinkActualVelocity { .. }
                            | crate::MainLoopInterruption::LinkActualVelocityCompleted
                            | crate::MainLoopInterruption::LinkVelocityClearProgress { .. }
                            | crate::MainLoopInterruption::DungeonExitSpotlightTableCompleted
                            | crate::MainLoopInterruption::LinkPositionBeforeCoordinates
                            | crate::MainLoopInterruption::LinkPositionAfterSubpixel { .. }
                            | crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { .. }
                            | crate::MainLoopInterruption::LinkPositionAfterCoordinates { .. }
                            | crate::MainLoopInterruption::GameOverIrisGoalPaletteFill { .. }
                            | crate::MainLoopInterruption::SpotlightGoalResetTable { .. }
                            | crate::MainLoopInterruption::DesertPrayerIris { .. }
                            | crate::MainLoopInterruption::DesertPrayerPaletteFilterBeforeColor { .. }
                    )
                    || interruption.is_sprite_main(),
                "an interrupted idle main-loop plan has no native outer owner for {interruption:?}",
            );
        }
        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .expect("an interrupted idle main-loop host lost its semantic authority")
            .semantic
            .clone();
        let progress = semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::MainLoopProgress(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            progress.len(),
            1,
            "an interrupted idle main-loop host must publish exactly one progress fact",
        );
        assert_eq!(
            progress[0].1,
            crate::MainLoopProgress::IterationStarted,
            "an interrupted idle main-loop host must begin one fresh source iteration",
        );
        let progress_index = progress[0].0;
        let interruption_index = semantic
            .iter()
            .position(|receipt| {
                *receipt == OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption)
            })
            .expect("an interrupted idle main-loop host lost its interruption fact");
        assert!(
            progress_index < interruption_index,
            "an interrupted idle main-loop host published its interruption before beginning the source iteration",
        );
        let mut nmi_phases_before_progress = Vec::new();
        let mut nmi_phases_after_progress = Vec::new();
        for (index, receipt) in semantic.iter().enumerate() {
            let phase = match receipt {
                OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                    OriginalTimingNmiPhase::Accepted(*gate)
                }
                OriginalTimingSemanticReceipt::NmiHandlerCompleted => {
                    OriginalTimingNmiPhase::HandlerCompleted
                }
                _ => continue,
            };
            if index < progress_index {
                nmi_phases_before_progress.push(phase);
            } else {
                assert!(
                    index < interruption_index,
                    "an interrupted idle main-loop host published hardware lifecycle after its terminal interruption",
                );
                nmi_phases_after_progress.push(phase);
            }
        }
        let timeline = OriginalTimingMainLoopTimeline {
            progress: crate::MainLoopProgress::IterationStarted,
            nmi_phases_before_progress,
            nmi_phases_after_progress,
        };
        assert_eq!(
            self.original_timing_sprite_main_return_claims_remaining, None,
            "an interrupted idle main-loop plan cannot overlap an older Sprite_Main return-claim scope",
        );
        assert_eq!(
            self.pending_main_loop_common_suffix, None,
            "a fresh interrupted main-loop iteration cannot overlap a pending common suffix",
        );
        assert!(
            self.original_timing_main_loop_return_timeline().is_none(),
            "an interrupted main-loop host cannot also complete the common suffix",
        );
        assert!(
            matches!(
                timeline.nmi_phases_after_progress.as_slice(),
                [] | [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld)]
            ),
            "a fresh interrupted main-loop body may only stop at its host-return boundary or carry the Held NMI which exposed it: {timeline:?}",
        );

        let before_main = classify_original_timing_nmi_phases_with_ownership(
            self.original_timing_nmi_publication_pending,
            &timeline.nmi_phases_before_progress,
        );
        assert!(
            !before_main.publication_pending_at_exit,
            "the source cannot begin ZeldaRunGameLoop while an accepted NMI publication remains unfinished",
        );
        assert_original_timing_carry_in_handler_has_receptive_display(
            before_main,
            self.display_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
        );
        self.preflight_dialogue_text_dma_before_early_stage(before_main.handler_completion, true);
        let pre_main_timing_shadow = if scheduled_predecessor.is_some() {
            assert_eq!(
                self.game_execution_scheduler.pre_main_nmi_resume(),
                None,
                "a scheduled caller-to-fresh-iteration plan cannot overlap pre-main work",
            );
            assert!(
                before_main.handler_completion.completed(),
                "a scheduled caller cannot return to a fresh iteration before its leading NMI handler completes",
            );
            None
        } else {
            self.original_timing_fresh_iteration_pre_main_shadow(before_main)
        };

        let sprite_main_returned_claims = semantic
            .iter()
            .filter(|receipt| **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned)
            .count();
        assert!(
            sprite_main_returned_claims <= 2,
            "one interrupted native module body cannot return from more than two Sprite_Main loops",
        );
        if let Some(predecessor) = scheduled_predecessor {
            let expected_sprite_main_returns = usize::from(!interruption.is_sprite_main());
            assert_eq!(
                sprite_main_returned_claims, expected_sprite_main_returns,
                "a scheduled caller-to-fresh-iteration host disagrees with the exact Sprite_Main boundary preceding {interruption:?}",
            );
            if matches!(
                predecessor.work,
                GameWorkContinuation::FinishSpotlightIteration { .. }
            ) {
                assert_ne!(
                    interruption,
                    crate::MainLoopInterruption::SpritePreparation,
                    "Module0F spotlight cannot own Module07's plain sprite-preparation interruption",
                );
            }
        }
        let joypads = semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::JoypadPublication(joypad) => Some(*joypad),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut joypad_index = 0usize;
        let mut expected_semantic = Vec::new();
        let mut expected_gates = Vec::new();
        let mut semantic_gate = if self.original_timing_nmi_publication_pending {
            let gate = self.original_timing_pending_nmi_update_gate.expect(
                "an interrupted main-loop host lost its carried NMI acceptance disposition",
            );
            expected_gates.push(gate);
            Some(gate)
        } else {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate, None,
                "an interrupted main-loop host retained an NMI gate without a carried handler",
            );
            None
        };
        let mut append_phases =
            |phases: &[OriginalTimingNmiPhase],
             expected: &mut Vec<OriginalTimingSemanticReceipt>| {
                for phase in phases {
                    match phase {
                        OriginalTimingNmiPhase::Accepted(gate) => {
                            expected_gates.push(*gate);
                            semantic_gate = Some(*gate);
                            expected.push(OriginalTimingSemanticReceipt::NmiAccepted(*gate));
                        }
                        OriginalTimingNmiPhase::HandlerCompleted => {
                            let gate = semantic_gate.take().expect(
                                "an interrupted main-loop handler completed without an accepted NMI",
                            );
                            expected.push(OriginalTimingSemanticReceipt::NmiHandlerCompleted);
                            if gate == NmiUpdateGate::Open {
                                let joypad = joypads.get(joypad_index).copied().expect(
                                    "an Open interrupted main-loop handler omitted its Joypad publication",
                                );
                                joypad_index += 1;
                                expected
                                    .push(OriginalTimingSemanticReceipt::JoypadPublication(joypad));
                            }
                        }
                    }
                }
            };
        append_phases(&timeline.nmi_phases_before_progress, &mut expected_semantic);
        expected_semantic.push(OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::IterationStarted,
        ));
        expected_semantic.extend(std::iter::repeat_n(
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            sprite_main_returned_claims,
        ));
        append_phases(&timeline.nmi_phases_after_progress, &mut expected_semantic);
        expected_semantic.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(
            interruption,
        ));
        let stair_progress = semantic
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::LinkOamStairProgress(_)
                )
            })
            .collect::<Vec<_>>();
        assert!(stair_progress.len() <= 1, "LinkOam stair progress replayed");
        if let Some(receipt) = stair_progress.first() {
            assert_eq!(interruption, crate::MainLoopInterruption::LinkOam);
            assert!(matches!(self.game_state.frame.main_module, 7 | 9));
            assert!(
                matches!(self.game_state.frame.submodule, 18 | 19),
                "stair drawing receipt reached an unrelated fresh caller"
            );
            expected_semantic.push(**receipt);
        }
        if matches!(
            interruption,
            crate::MainLoopInterruption::DungeonExitSpotlightAfterSubmodule
                | crate::MainLoopInterruption::LinkActualVelocity { .. }
                | crate::MainLoopInterruption::LinkActualVelocityCompleted
                | crate::MainLoopInterruption::LinkVelocityClearProgress { .. }
                | crate::MainLoopInterruption::DungeonExitSpotlightTableCompleted
                | crate::MainLoopInterruption::LinkPositionBeforeCoordinates
                | crate::MainLoopInterruption::LinkPositionAfterSubpixel { .. }
                | crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { .. }
                | crate::MainLoopInterruption::LinkPositionAfterCoordinates { .. }
                | crate::MainLoopInterruption::LinkOam
        ) && semantic.iter().any(|receipt| {
            matches!(
                receipt,
                OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned
            )
        }) {
            assert_eq!(
                (
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule
                ),
                (0x0f, 0),
                "spotlight entry return reached an unrelated fresh caller"
            );
            // The submodule-return fact and the following interruption are
            // two views of the same exact source ordering: the entry call
            // returned, then the host stopped its Module0F caller before or
            // inside the Link suffix. The pre-entry owner consumes the return token
            // after this immutable preflight.
            expected_semantic
                .push(OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned);
        }
        if let crate::MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(started_slot) =
            interruption
        {
            // The typed interruption owns the surrounding Sprite_Main
            // remainder.  A generic SpriteMainProgressed claim would name an
            // earlier statement in the same slot and has no independent
            // consumer, so the adapter must not publish one.
            for receipt in &semantic {
                match receipt {
                    OriginalTimingSemanticReceipt::SpriteMainProgressed(_) => panic!(
                        "a caller-specific item-receipt interruption must not duplicate an earlier Sprite_Main checkpoint"
                    ),
                    OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(progress) => {
                        assert!(matches!(
                            progress.caller,
                            ItemReceiptGraphicsCaller::SpriteMain { slot }
                                | ItemReceiptGraphicsCaller::UnclePassage { slot }
                                if slot == started_slot
                        ));
                        assert_eq!(
                            progress.progress,
                            crate::SourceCallProgress::Suspended,
                            "an interrupted item-receipt claim must report its suspended source call",
                        );
                        expected_semantic.push(*receipt);
                    }
                    _ => {}
                }
            }
        }
        if !matches!(
            interruption,
            crate::MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(_)
        ) {
            if let Some(expected_boundary) =
                sprite_main_cpu_boundary_from_interruption(interruption)
            {
                // The adapter appends the suspended Sprite_Main's checkpoint
                // at host finish; it restates the same C statement the
                // interruption names. Validate that coherence and let the
                // resumed caller consume it. (The item-receipt interruption's
                // AfterSlot restatement is validated by its own block above.)
                for receipt in &semantic {
                    match receipt {
                        OriginalTimingSemanticReceipt::SpriteMainProgressed(boundary) => {
                            assert_eq!(
                                sprite_main_cpu_boundary_from_progress(*boundary),
                                expected_boundary,
                                "a paired Sprite_Main checkpoint disagrees with its interruption statement",
                            );
                            expected_semantic.push(*receipt);
                        }
                        OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(progress) => {
                            // The interrupted statement can sit inside the
                            // slot's direct item-receipt graphics call; the
                            // adapter then restates that suspended source
                            // call beside the slot checkpoint (route: slot
                            // 15 suspends before any slot returns).
                            assert_eq!(
                                progress.progress,
                                crate::SourceCallProgress::Suspended,
                                "an interrupted item-receipt claim must report its suspended source call",
                            );
                            match progress.caller {
                                ItemReceiptGraphicsCaller::SpriteMainDirect { slot } => {
                                    assert!(
                                        direct_item_receipt_slot_pairs_with_boundary(
                                            slot,
                                            expected_boundary,
                                        ),
                                        "a suspended direct item-receipt claim disagrees with its interruption statement: slot={slot} boundary={expected_boundary:?}",
                                    );
                                }
                                ItemReceiptGraphicsCaller::SpriteMainAncilla { .. } => {
                                    // Ancilla_Main runs in Sprite_Main's prefix,
                                    // so its suspended receipt always pairs with
                                    // the loop's entry statement (route hosts
                                    // 1142850, 514800).
                                    assert_eq!(
                                        expected_boundary,
                                        SpriteMainCpuBoundary::BeforeFirstSlot,
                                        "a suspended ancilla item-receipt claim disagrees with its interruption statement: {expected_boundary:?}",
                                    );
                                }
                                other => panic!(
                                    "an interrupted main-loop host restated an unsupported item-receipt caller: {other:?}"
                                ),
                            }
                            expected_semantic.push(*receipt);
                        }
                        _ => {}
                    }
                }
            }
        }
        assert_eq!(
            joypad_index,
            joypads.len(),
            "an interrupted main-loop host published an unowned Joypad receipt",
        );
        assert_eq!(
            self.original_timing_expected_nmi_update_gates, expected_gates,
            "an interrupted main-loop host disagrees with its exact NMI gate authority",
        );
        if before_main.handler_completion.completed() {
            let native_gate = if self.game_state.display.nmi_update_is_latched() {
                NmiUpdateGate::LatchHeld
            } else {
                NmiUpdateGate::Open
            };
            assert_eq!(
                expected_gates.first().copied(),
                Some(native_gate),
                "an interrupted main-loop host's leading handler disagrees with the native NMI latch",
            );
        }
        assert_eq!(
            semantic, expected_semantic,
            "an interrupted main-loop host published an unsupported or reordered semantic vector",
        );

        let semantic = semantic
            .into_iter()
            .filter(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned
                )
            })
            .collect();

        Some(OriginalTimingInterruptedIdleMainLoopPlan {
            semantic,
            timeline: OriginalTimingMainLoopInterruptionTimeline {
                progress: crate::MainLoopProgress::IterationStarted,
                interruption,
                nmi_phases_before_interruption: timeline
                    .nmi_phases_before_progress
                    .iter()
                    .chain(timeline.nmi_phases_after_progress.iter())
                    .copied()
                    .collect(),
                nmi_phases_after_interruption: Vec::new(),
            },
            sprite_main_returned_claims,
            pre_main_timing_shadow,
            scheduled_predecessor,
        })
    }

    pub(super) fn original_timing_fresh_iteration_pre_main_shadow(
        &self,
        before_main: OriginalTimingNmiPhaseClassification,
    ) -> Option<PreMainNmiResume> {
        if self.game_execution_scheduler.is_idle() {
            return None;
        }
        if matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishItemReceiptGraphics {
                continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { .. },
            })
        ) && self
            .game_execution_scheduler
            .pre_main_nmi_resume()
            .is_none()
        {
            // Atomic item-receipt decompression slices do not suspend the
            // translated call stack; a fresh iteration runs alongside them
            // with no pre-main timing shadow to retire (route host 158014).
            return None;
        }
        let mut scheduler_probe = self.game_execution_scheduler;
        let resume = scheduler_probe
            .take_pre_main_nmi_resume()
            .expect("a fresh idle-main iteration overlaps scheduled native work");
        assert!(
            resume.is_timing_shadow_completed_by_fresh_iteration(),
            "a fresh iteration cannot retire semantic pre-main work: {resume:?}",
        );
        assert!(
            scheduler_probe.is_idle(),
            "a fresh iteration cannot overlap work attached to its pre-main timing shadow",
        );
        assert!(
            before_main.handler_completion.completed(),
            "a fresh iteration cannot retire a pre-main timing shadow before completing its carried NMI handler",
        );
        Some(resume)
    }

    pub(super) fn retire_original_timing_fresh_iteration_pre_main_shadow(
        &mut self,
        expected: Option<PreMainNmiResume>,
    ) {
        assert_eq!(
            self.game_execution_scheduler.pre_main_nmi_resume(),
            expected,
            "fresh-iteration pre-main timing-shadow authority changed after immutable preflight",
        );
        if let Some(resume) = expected {
            assert_eq!(
                self.game_execution_scheduler.take_pre_main_nmi_resume(),
                Some(resume),
            );
        }
    }

    pub(super) fn original_timing_uninterrupted_idle_main_loop_plan(
        &self,
        expected_progress: crate::MainLoopProgress,
    ) -> Option<OriginalTimingUninterruptedIdleMainLoopPlan> {
        let timeline = self.original_timing_uninterrupted_main_loop_timeline(expected_progress)?;
        assert_eq!(
            self.original_timing_sprite_main_return_claims_remaining, None,
            "an uninterrupted idle main-loop plan cannot overlap an older Sprite_Main return-claim scope",
        );
        let return_timeline = self.original_timing_main_loop_return_timeline();
        let completes_main_loop = return_timeline.is_some();
        let suffix_action = match (
            timeline.progress,
            self.pending_main_loop_common_suffix,
            completes_main_loop,
        ) {
            (crate::MainLoopProgress::IterationStarted, None, false) => {
                OriginalTimingIdleMainLoopSuffixAction::ArmOrdinary
            }
            (crate::MainLoopProgress::IterationStarted, None, true) => {
                OriginalTimingIdleMainLoopSuffixAction::CompleteInSequence
            }
            (
                crate::MainLoopProgress::CallStackContinued,
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                false,
            ) => OriginalTimingIdleMainLoopSuffixAction::RetainOrdinary,
            (crate::MainLoopProgress::CallStackContinued, None, false)
                if timeline.nmi_phases_before_progress.is_empty()
                    && timeline.nmi_phases_after_progress.is_empty()
                    && !self.original_timing_nmi_publication_pending
                    && self.game_execution_scheduler.is_idle() =>
            {
                // A bare Continued host with no NMI lifecycle and an idle
                // native call stack: the ROM waited out the interval with the
                // NMI disabled (route host 37663).
                OriginalTimingIdleMainLoopSuffixAction::HoldAtMainWait
            }
            (crate::MainLoopProgress::IterationStarted, suffix, _) => panic!(
                "a fresh uninterrupted main-loop iteration overlapped a pending common suffix: {suffix:?}",
            ),
            (crate::MainLoopProgress::CallStackContinued, suffix, _) => panic!(
                "a continued uninterrupted main-loop call lost its exact ordinary suffix: {suffix:?} host={:?} frame={:?} scheduler={:?} timeline={timeline:?}",
                self.original_timing_last_oracle_host_call,
                self.game_state.frame,
                self.game_execution_scheduler,
            ),
        };
        if suffix_action == OriginalTimingIdleMainLoopSuffixAction::RetainOrdinary {
            assert!(
                !self.main_loop_sprite_preparation_completed,
                "a continued uninterrupted main-loop call cannot retain an already-completed sprite-preparation suffix",
            );
            assert!(
                self.game_state.display.nmi_update_is_latched(),
                "a continued uninterrupted main-loop call cannot retain its pre-clear suffix with the native NMI latch open",
            );
            assert!(
                timeline
                    .nmi_phases_before_progress
                    .iter()
                    .all(|phase| !matches!(
                        phase,
                        OriginalTimingNmiPhase::Accepted(gate)
                            if *gate != NmiUpdateGate::LatchHeld
                    )),
                "a continued uninterrupted main-loop call may only accept Held NMIs before its pending latch clear: {timeline:?}",
            );
        }

        let before_main = classify_original_timing_nmi_phases_with_ownership(
            self.original_timing_nmi_publication_pending,
            &timeline.nmi_phases_before_progress,
        );
        assert_original_timing_carry_in_handler_has_receptive_display(
            before_main,
            self.display_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
        );
        match timeline.progress {
            crate::MainLoopProgress::IterationStarted => assert!(
                !before_main.publication_pending_at_exit,
                "the source cannot begin ZeldaRunGameLoop while an accepted NMI publication remains unfinished",
            ),
            crate::MainLoopProgress::CallStackContinued => assert!(
                timeline.nmi_phases_after_progress.is_empty(),
                "CallStackContinued is the terminal source-host progress fact: {timeline:?}",
            ),
        }
        self.preflight_dialogue_text_dma_before_early_stage(
            before_main.handler_completion,
            timeline.progress == crate::MainLoopProgress::IterationStarted,
        );
        let pre_main_timing_shadow =
            if timeline.progress == crate::MainLoopProgress::IterationStarted {
                self.original_timing_fresh_iteration_pre_main_shadow(before_main)
            } else {
                assert!(
                    self.game_execution_scheduler.is_idle(),
                    "a continued idle-main call cannot retire pre-main scheduled work",
                );
                None
            };

        let mut expected_gates = Vec::new();
        let mut active_gate = if self.original_timing_nmi_publication_pending {
            let gate = self.original_timing_pending_nmi_update_gate.expect(
                "an uninterrupted main-loop host lost its carried NMI acceptance disposition",
            );
            expected_gates.push(gate);
            Some(gate)
        } else {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate, None,
                "an uninterrupted main-loop host retained an NMI gate without a carried handler",
            );
            None
        };
        for phase in timeline
            .nmi_phases_before_progress
            .iter()
            .chain(timeline.nmi_phases_after_progress.iter())
        {
            match phase {
                OriginalTimingNmiPhase::Accepted(gate) => {
                    expected_gates.push(*gate);
                    active_gate = Some(*gate);
                }
                OriginalTimingNmiPhase::HandlerCompleted => {
                    active_gate.take().expect(
                        "an uninterrupted main-loop handler completed without an accepted NMI",
                    );
                }
            }
        }
        assert_eq!(
            self.original_timing_expected_nmi_update_gates, expected_gates,
            "an uninterrupted main-loop host disagrees with its exact NMI gate authority",
        );
        if before_main.handler_completion.completed() {
            let native_gate = if self.game_state.display.nmi_update_is_latched() {
                NmiUpdateGate::LatchHeld
            } else {
                NmiUpdateGate::Open
            };
            assert_eq!(
                expected_gates.first().copied(),
                Some(native_gate),
                "an uninterrupted main-loop host's leading handler disagrees with the native NMI latch",
            );
        }

        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .expect("an uninterrupted main-loop host lost its semantic authority")
            .semantic
            .clone();
        let completed_suffix_receipts = semantic
            .iter()
            .filter(|receipt| original_timing_receipt_completes_main_loop_common_suffix(receipt))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(
            completed_suffix_receipts.len(),
            usize::from(completes_main_loop),
            "an uninterrupted main-loop plan disagrees with its exact source suffix receipt",
        );
        let (in_module_phases_after_progress, post_suffix_phases) =
            if let Some(return_timeline) = return_timeline.as_ref() {
                assert_eq!(
                    return_timeline.progress, timeline.progress,
                    "uninterrupted and return timelines disagreed about main-loop progress",
                );
                let in_module_phases_after_progress = return_timeline
                    .nmi_phases_before_return
                    .strip_prefix(timeline.nmi_phases_before_progress.as_slice())
                    .expect("main-loop return timeline lost the NMI phases which preceded progress")
                    .to_vec();
                let mut combined_after_progress = in_module_phases_after_progress.clone();
                combined_after_progress.extend_from_slice(&return_timeline.nmi_phases_after_return);
                assert_eq!(
                    combined_after_progress, timeline.nmi_phases_after_progress,
                    "main-loop return timeline changed the ordered after-progress NMI lifecycle",
                );
                (
                    in_module_phases_after_progress,
                    return_timeline.nmi_phases_after_return.clone(),
                )
            } else {
                (timeline.nmi_phases_after_progress.clone(), Vec::new())
            };
        match suffix_action {
            OriginalTimingIdleMainLoopSuffixAction::ArmOrdinary => assert!(
                matches!(
                    in_module_phases_after_progress.as_slice(),
                    [] | [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld)]
                ) && post_suffix_phases.is_empty(),
                "a suspended fresh iteration may only carry one terminal Held acceptance after its CPU slice",
            ),
            OriginalTimingIdleMainLoopSuffixAction::RetainOrdinary
            | OriginalTimingIdleMainLoopSuffixAction::HoldAtMainWait => assert!(
                in_module_phases_after_progress.is_empty() && post_suffix_phases.is_empty(),
                "CallStackContinued is the terminal source-host progress fact",
            ),
            OriginalTimingIdleMainLoopSuffixAction::CompleteInSequence => {
                assert!(
                    in_module_phases_after_progress.is_empty(),
                    "an IterationStarted host cannot complete an in-module NMI without a typed CPU continuation split",
                );
                assert!(
                    matches!(
                        post_suffix_phases.as_slice(),
                        [] | [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)]
                            | [
                                OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open),
                                OriginalTimingNmiPhase::HandlerCompleted,
                            ]
                            | [
                                OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open),
                                OriginalTimingNmiPhase::HandlerCompleted,
                                OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                            ]
                    ),
                    "a completed fresh iteration published an unsupported post-suffix NMI lifecycle",
                );
            }
        }
        let dialogue_progress = self.original_timing_dialogue_execution_progress();
        let dialogue_closed_count = semantic
            .iter()
            .filter(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::DialogueClosed))
            .count();
        assert!(
            dialogue_closed_count <= 1,
            "one host call cannot close dialogue twice",
        );
        let dialogue_claim = match (dialogue_progress, dialogue_closed_count) {
            (None, 0) => OriginalTimingIdleMainLoopDialogueClaim::None,
            (Some(progress), 0) => {
                let message_read_position = progress.message_read_position();
                let current_glyph_started = progress.current_glyph_started();
                assert_eq!(
                    timeline.progress,
                    crate::MainLoopProgress::CallStackContinued,
                    "a resumed dialogue endpoint cannot begin a fresh main-loop iteration",
                );
                assert!(
                    self.frame_module_hosts_dialogue(),
                    "a resumed dialogue endpoint escaped Module0E/Module1B",
                );
                let transition = self.original_timing_suspended_vwf_endpoint_transition_plan(
                    message_read_position,
                    current_glyph_started,
                );
                OriginalTimingIdleMainLoopDialogueClaim::ResumedRenderingWithoutMainIteration {
                    message_read_position,
                    current_glyph_started,
                    transition,
                }
            }
            (None, 1) => {
                assert_eq!(
                    timeline.progress,
                    crate::MainLoopProgress::IterationStarted,
                    "dialogue close must be owned by the fresh main iteration which observes it",
                );
                assert!(
                    self.frame_module_hosts_dialogue(),
                    "dialogue-close authority escaped Module0E/Module1B",
                );
                OriginalTimingIdleMainLoopDialogueClaim::DialogueClosed
            }
            _ => panic!("one uninterrupted main-loop host published overlapping dialogue claims"),
        };
        let spotlight_claims = semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(receipt) => {
                    Some(*receipt)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            spotlight_claims.len() <= 1,
            "one uninterrupted main-loop host cannot publish multiple spotlight checkpoints",
        );
        let spotlight_claim = spotlight_claims.first().copied();
        if let Some(claim) = spotlight_claim {
            assert_eq!(
                suffix_action,
                OriginalTimingIdleMainLoopSuffixAction::ArmOrdinary,
                "spotlight table progress must suspend one fresh main-loop iteration before its common suffix",
            );
            assert!(
                matches!(
                    (
                        self.game_state.frame.main_module,
                        self.game_state.frame.submodule,
                    ),
                    (0x0f | 0x10, 0 | 1) | (0x12, 2 | 3)
                ),
                "spotlight table progress escaped its exact Module0F/Module10/Module12 owner",
            );
            match claim.boundary {
                crate::OriginalTimingBoundary::NmiAccepted => assert_eq!(
                    in_module_phases_after_progress,
                    [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld)],
                    "an NMI-bound spotlight checkpoint must bind exactly to its terminal Held acceptance",
                ),
                crate::OriginalTimingBoundary::HostReturn => assert!(
                    in_module_phases_after_progress.is_empty(),
                    "a host-return spotlight checkpoint cannot retain a later hardware lifecycle",
                ),
            }
        }
        let sprite_main_returned_claims = semantic
            .iter()
            .filter(|receipt| **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned)
            .count();
        if timeline.progress == crate::MainLoopProgress::CallStackContinued {
            assert_eq!(
                sprite_main_returned_claims, 0,
                "an idle continued-call host cannot own a fresh Sprite_Main return; resumed scheduled callers must claim it explicitly",
            );
        }
        if sprite_main_returned_claims != 0 {
            assert_eq!(
                self.sprite_main_cpu_boundary, None,
                "an idle-body Sprite_Main return cannot overlap a translated early-return boundary",
            );
            assert_eq!(
                self.sprite_main_cpu_nmi_slices, 0,
                "an idle-body Sprite_Main return cannot overlap pending translated NMI slices",
            );
        }

        let joypads = semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::JoypadPublication(joypad) => Some(*joypad),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut joypad_index = 0usize;
        let mut expected_semantic = Vec::new();
        let mut semantic_gate = if self.original_timing_nmi_publication_pending {
            self.original_timing_pending_nmi_update_gate
        } else {
            None
        };
        let mut append_phases =
            |phases: &[OriginalTimingNmiPhase],
             expected: &mut Vec<OriginalTimingSemanticReceipt>| {
                for phase in phases {
                    match phase {
                        OriginalTimingNmiPhase::Accepted(gate) => {
                            expected.push(OriginalTimingSemanticReceipt::NmiAccepted(*gate));
                            semantic_gate = Some(*gate);
                        }
                        OriginalTimingNmiPhase::HandlerCompleted => {
                            let gate = semantic_gate.take().expect(
                            "an uninterrupted main-loop semantic handler completed without an accepted NMI",
                        );
                            expected.push(OriginalTimingSemanticReceipt::NmiHandlerCompleted);
                            if gate == NmiUpdateGate::Open {
                                let joypad = joypads.get(joypad_index).copied().expect(
                                "an Open uninterrupted main-loop handler omitted its Joypad publication",
                            );
                                joypad_index += 1;
                                expected
                                    .push(OriginalTimingSemanticReceipt::JoypadPublication(joypad));
                            }
                        }
                    }
                }
            };
        append_phases(&timeline.nmi_phases_before_progress, &mut expected_semantic);
        if timeline.progress == crate::MainLoopProgress::CallStackContinued {
            for _ in 0..sprite_main_returned_claims {
                expected_semantic.push(OriginalTimingSemanticReceipt::SpriteMainReturned);
            }
        }
        expected_semantic.push(OriginalTimingSemanticReceipt::MainLoopProgress(
            timeline.progress,
        ));
        if timeline.progress == crate::MainLoopProgress::IterationStarted {
            for _ in 0..sprite_main_returned_claims {
                expected_semantic.push(OriginalTimingSemanticReceipt::SpriteMainReturned);
            }
            // SCAN_KEYS may return the host while the fresh iteration is
            // still inside Sprite_Main's item-receipt graphics call without
            // accepting an NMI. The adapter restates that suspension as
            // paired host-return checkpoints: the furthest returned slot and
            // the suspended item-receipt source call. The translated caller
            // entering Sprite_Main and the item-receipt machinery are their
            // owners; this plan only pins the wire positions.
            for receipt in &semantic {
                match receipt {
                    OriginalTimingSemanticReceipt::SpriteMainProgressed(_) => {
                        expected_semantic.push(*receipt);
                    }
                    OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(progress) => {
                        assert_eq!(
                            progress.progress,
                            crate::SourceCallProgress::Suspended,
                            "an uninterrupted host's item-receipt claim must report its suspended source call",
                        );
                        expected_semantic.push(*receipt);
                    }
                    _ => {}
                }
            }
        }
        // The plain sprite-preparation interruption is owned by the dungeon
        // module body, which consumes it mid-iteration as its own suffix
        // continuation; this outer plan only pins its wire position. The
        // receipt precedes the trailing acceptance when the host returned
        // before that NMI, and follows it when the preparation was
        // interrupted at the accepted boundary itself (route host 23202).
        let sprite_preparation_pin = (self.original_timing_main_loop_interruption()
            == Some(crate::MainLoopInterruption::SpritePreparation))
        .then_some(OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::SpritePreparation,
        ));
        let sprite_preparation_rides_trailing_acceptance = sprite_preparation_pin.is_some()
            && semantic
                .iter()
                .position(|receipt| {
                    matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::MainLoopInterrupted(
                            crate::MainLoopInterruption::SpritePreparation,
                        )
                    )
                })
                .zip(semantic.iter().rposition(|receipt| {
                    matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_))
                }))
                .is_some_and(|(interruption_index, acceptance_index)| {
                    interruption_index > acceptance_index
                });
        if let Some(pin) =
            sprite_preparation_pin.filter(|_| !sprite_preparation_rides_trailing_acceptance)
        {
            expected_semantic.push(pin);
        }
        if let Some(
            receipt @ SpotlightTableBuildProgressReceipt {
                boundary: crate::OriginalTimingBoundary::HostReturn,
                ..
            },
        ) = spotlight_claim
        {
            expected_semantic.push(OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                receipt,
            ));
        }
        append_phases(&in_module_phases_after_progress, &mut expected_semantic);
        if let Some(pin) =
            sprite_preparation_pin.filter(|_| sprite_preparation_rides_trailing_acceptance)
        {
            expected_semantic.push(pin);
        }
        if let Some(
            receipt @ SpotlightTableBuildProgressReceipt {
                boundary: crate::OriginalTimingBoundary::NmiAccepted,
                ..
            },
        ) = spotlight_claim
        {
            expected_semantic.push(OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                receipt,
            ));
        }
        expected_semantic.extend(completed_suffix_receipts);
        append_phases(&post_suffix_phases, &mut expected_semantic);
        match dialogue_claim {
            OriginalTimingIdleMainLoopDialogueClaim::None => {}
            OriginalTimingIdleMainLoopDialogueClaim::ResumedRenderingWithoutMainIteration {
                message_read_position,
                current_glyph_started,
                ..
            } => {
                let progress = if current_glyph_started {
                    crate::DialogueExecutionProgress::ResumedRenderingWithCurrentGlyphStarted {
                        message_read_position,
                    }
                } else {
                    crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                        message_read_position,
                    }
                };
                expected_semantic.push(OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                    progress,
                ));
            }
            OriginalTimingIdleMainLoopDialogueClaim::DialogueClosed => {
                expected_semantic.push(OriginalTimingSemanticReceipt::DialogueClosed);
            }
        }
        assert_eq!(
            joypad_index,
            joypads.len(),
            "an uninterrupted main-loop host published an unowned Joypad receipt",
        );
        // A cached-sprite execution, Sprite_ResetAll, Dungeon_ResetSprites,
        // or save-menu initialization claim is consumed by the iteration's own executor
        // for that domain; this plan only pins it at its exact wire position
        // (route hosts 25348, 25921, 47619).
        for (index, receipt) in semantic.iter().enumerate() {
            if matches!(
                receipt,
                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(_)
                    | OriginalTimingSemanticReceipt::PreDungeonSpriteDisableThrough { .. }
                    | OriginalTimingSemanticReceipt::PreDungeonGarnishDisableThrough { .. }
                    | OriginalTimingSemanticReceipt::SpriteResetAllProgress(_)
                    | OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(_)
                    // Module07/$16 consumes this source X/bank cursor when it
                    // enters the peg-attribute loop in this same iteration.
                    | OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(_)
                    | OriginalTimingSemanticReceipt::SaveMenuInitializationProgress(_)
                    // Module07_18 consumes this checkpoint when its source
                    // store loop is reached later in the same translated
                    // iteration; the outer timeline only preserves its exact
                    // position after the accepting NMI.
                    | OriginalTimingSemanticReceipt::RescuedMaidenTilemapClearProgress(_)
                    // The state-10 body consumes this host-return cursor when
                    // it reaches the synchronous follower-graphics call.
                    | OriginalTimingSemanticReceipt::RescuedMaidenInitializationProgress(_)
                    // Module0B/$24 consumes these staged source checkpoints
                    // from its suspended animated-sprite decode continuation.
                    | OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicRestored
                    | OriginalTimingSemanticReceipt::DungeonPushBlocksPending
                    | OriginalTimingSemanticReceipt::DungeonPushBlocksInProgress { .. }
                    | OriginalTimingSemanticReceipt::DungeonPushBlocksHandled
                    | OriginalTimingSemanticReceipt::Module09FinalScrollPairPending
                    | OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicReturned
                    | OriginalTimingSemanticReceipt::DungeonFallingFadeInPaletteDirectionToggled
                    // Module0F's entry call can return inside a fresh
                    // iteration whose trailing acceptance stays held; the
                    // pre-entry owner defers the token to the native entry
                    // (route host 252089).
                    | OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned
            ) {
                expected_semantic.insert(index.min(expected_semantic.len()), *receipt);
            }
        }
        assert_eq!(
            semantic, expected_semantic,
            "an uninterrupted main-loop host published an unsupported or reordered semantic vector",
        );

        // The pre-entry Module0F owner defers the spotlight entry-return
        // token before this plan is consumed; compare against the vector it
        // leaves behind (route host 252089).
        let semantic = semantic
            .iter()
            .copied()
            .filter(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned
                )
            })
            .collect::<Vec<_>>();
        Some(OriginalTimingUninterruptedIdleMainLoopPlan {
            semantic,
            timeline,
            in_module_phases_after_progress,
            post_suffix_phases,
            before_main,
            suffix_action,
            sprite_main_returned_claims,
            spotlight_claim,
            dialogue_claim,
            pre_main_timing_shadow,
        })
    }

    /// Build the source-owned lifecycle for one suspended caller slice which
    /// resumes through exactly one Held NMI handler but does not yet reach the
    /// common main-loop suffix.
    pub(super) fn original_timing_nonterminal_continuation_plan(
        &self,
    ) -> Option<OriginalTimingNonterminalContinuationPlan> {
        self.original_timing_nonterminal_continuation_plan_with_receipt_before_progress(None)
    }

    pub(super) fn original_timing_nonterminal_continuation_plan_with_receipt_before_progress(
        &self,
        receipt_before_progress: Option<OriginalTimingSemanticReceipt>,
    ) -> Option<OriginalTimingNonterminalContinuationPlan> {
        self.original_timing_nonterminal_continuation_plan_with_receipt(
            receipt_before_progress.map(|receipt| {
                (
                    receipt,
                    OriginalTimingNonterminalReceiptPlacement::BeforeProgress,
                )
            }),
        )
    }

    pub(super) fn original_timing_nonterminal_continuation_plan_with_receipt_before_trailing_acceptance(
        &self,
        receipt: OriginalTimingSemanticReceipt,
    ) -> Option<OriginalTimingNonterminalContinuationPlan> {
        self.original_timing_nonterminal_continuation_plan_with_receipt(Some((
            receipt,
            OriginalTimingNonterminalReceiptPlacement::BeforeTrailingAcceptance,
        )))
    }

    pub(super) fn original_timing_nonterminal_continuation_plan_with_receipt(
        &self,
        receipt: Option<(
            OriginalTimingSemanticReceipt,
            OriginalTimingNonterminalReceiptPlacement,
        )>,
    ) -> Option<OriginalTimingNonterminalContinuationPlan> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let timeline = self
            .original_timing_uninterrupted_main_loop_timeline(
                crate::MainLoopProgress::CallStackContinued,
            )
            .expect("live suspended caller omitted its continued-call timeline");
        assert!(
            timeline.nmi_phases_after_progress.is_empty(),
            "a nonterminal suspended caller cannot publish hardware phases after its terminal progress fact: {timeline:?}",
        );
        assert_eq!(
            self.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "a nonterminal suspended caller lost its ordinary main-loop suffix",
        );
        assert!(
            !self.main_loop_sprite_preparation_completed,
            "a nonterminal suspended caller cannot own an already-completed sprite-preparation suffix",
        );
        assert!(
            self.original_timing_main_loop_interruption().is_none(),
            "an uninterrupted suspended caller cannot also publish a main-loop interruption",
        );

        let nmi = classify_original_timing_nmi_phases_with_ownership(
            self.original_timing_nmi_publication_pending,
            &timeline.nmi_phases_before_progress,
        );
        assert!(
            nmi.handler_completion.completed(),
            "a nonterminal suspended caller must resume through exactly one completed NMI handler: {timeline:?}",
        );
        assert!(
            timeline
                .nmi_phases_before_progress
                .iter()
                .all(|phase| !matches!(
                    phase,
                    OriginalTimingNmiPhase::Accepted(gate) if *gate != NmiUpdateGate::LatchHeld
                )),
            "a nonterminal suspended caller may only accept Held NMIs before its source return: {timeline:?}",
        );
        assert!(
            self.game_state.display.nmi_update_is_latched(),
            "a nonterminal suspended caller's Held handler requires the native NMI latch",
        );
        if self.original_timing_nmi_publication_pending {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate,
                Some(NmiUpdateGate::LatchHeld),
                "a carried nonterminal handler lost its Held acceptance disposition",
            );
        } else {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate, None,
                "a nonterminal caller retained an NMI disposition without a carried handler",
            );
        }
        assert_original_timing_carry_in_handler_has_receptive_display(
            nmi,
            self.display_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
        );

        let acceptance_count = timeline
            .nmi_phases_before_progress
            .iter()
            .filter(|phase| matches!(phase, OriginalTimingNmiPhase::Accepted(_)))
            .count();
        let expected_gate_count =
            acceptance_count + usize::from(self.original_timing_nmi_publication_pending);
        assert_eq!(
            self.original_timing_expected_nmi_update_gates,
            vec![NmiUpdateGate::LatchHeld; expected_gate_count],
            "a nonterminal suspended caller lost its exact Held acceptance dispositions",
        );

        let mut semantic = timeline
            .nmi_phases_before_progress
            .iter()
            .map(|phase| match phase {
                OriginalTimingNmiPhase::Accepted(gate) => {
                    OriginalTimingSemanticReceipt::NmiAccepted(*gate)
                }
                OriginalTimingNmiPhase::HandlerCompleted => {
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted
                }
            })
            .collect::<Vec<_>>();
        if let Some((receipt, placement)) = receipt {
            match placement {
                OriginalTimingNonterminalReceiptPlacement::BeforeProgress => {
                    semantic.push(receipt);
                }
                OriginalTimingNonterminalReceiptPlacement::BeforeTrailingAcceptance => {
                    assert!(
                        matches!(
                            semantic.last(),
                            Some(OriginalTimingSemanticReceipt::NmiAccepted(_))
                        ),
                        "a receipt placed before the trailing acceptance requires that exact source phase",
                    );
                    let insertion = semantic.len() - 1;
                    semantic.insert(insertion, receipt);
                }
            }
        }
        semantic.push(OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ));
        assert_eq!(
            self.original_timing_semantic_receipts
                .as_ref()
                .expect("live suspended caller lost its semantic authority")
                .semantic,
            semantic,
            "a nonterminal suspended caller published an unsupported or reordered semantic vector",
        );

        Some(OriginalTimingNonterminalContinuationPlan {
            semantic,
            timeline,
            nmi,
        })
    }

    /// Selected-game decompression can complete either a carried Open handler
    /// (with its Joypad publication) or a Held handler while the shared caller
    /// remains suspended. The ordinary Held-only continuation plan is too
    /// narrow for that source family, but execution ownership is otherwise the
    /// same: one handler, no common suffix, and at most one trailing Held
    /// acceptance.
    pub(super) fn original_timing_selected_game_nonterminal_plan(
        &self,
    ) -> Option<OriginalTimingNonterminalContinuationPlan> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let timeline = self
            .original_timing_uninterrupted_main_loop_timeline(
                crate::MainLoopProgress::CallStackContinued,
            )
            .expect("live selected-game caller omitted its continued-call timeline");
        assert!(timeline.nmi_phases_after_progress.is_empty());
        assert_eq!(
            self.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
        );
        assert!(!self.main_loop_sprite_preparation_completed);
        assert!(self.original_timing_main_loop_interruption().is_none());
        assert!(timeline.nmi_phases_before_progress.iter().all(|phase| {
            !matches!(phase, OriginalTimingNmiPhase::Accepted(gate) if *gate != NmiUpdateGate::LatchHeld)
        }));

        let nmi = classify_original_timing_nmi_phases_with_ownership(
            self.original_timing_nmi_publication_pending,
            &timeline.nmi_phases_before_progress,
        );
        assert!(nmi.handler_completion.completed());
        let native_entry_gate = if self.game_state.display.nmi_update_is_latched() {
            NmiUpdateGate::LatchHeld
        } else {
            NmiUpdateGate::Open
        };
        if self.original_timing_nmi_publication_pending {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate,
                Some(native_entry_gate),
            );
        } else {
            assert_eq!(self.original_timing_pending_nmi_update_gate, None);
            assert_eq!(
                timeline.nmi_phases_before_progress.first(),
                Some(&OriginalTimingNmiPhase::Accepted(native_entry_gate)),
            );
        }
        assert_original_timing_carry_in_handler_has_receptive_display(
            nmi,
            self.display_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
        );
        let mut expected_gates = Vec::new();
        if self.original_timing_nmi_publication_pending {
            expected_gates.push(native_entry_gate);
        }
        expected_gates.extend(
            timeline
                .nmi_phases_before_progress
                .iter()
                .filter_map(|phase| match phase {
                    OriginalTimingNmiPhase::Accepted(gate) => Some(*gate),
                    OriginalTimingNmiPhase::HandlerCompleted => None,
                }),
        );
        assert_eq!(
            self.original_timing_expected_nmi_update_gates,
            expected_gates
        );

        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .expect("live selected-game caller lost its semantic authority")
            .semantic
            .clone();
        let joypad_publications = semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::JoypadPublication(publication) => Some(*publication),
                _ => None,
            })
            .collect::<Vec<_>>();
        let expected_joypad_count = usize::from(native_entry_gate == NmiUpdateGate::Open);
        assert_eq!(
            joypad_publications.len(),
            expected_joypad_count,
            "selected-game waiting slice must publish Joypad exactly once for an Open handler and never for a Held handler: {semantic:?}",
        );
        let mut expected_semantic = Vec::new();
        for phase in &timeline.nmi_phases_before_progress {
            expected_semantic.push(match phase {
                OriginalTimingNmiPhase::Accepted(gate) => {
                    OriginalTimingSemanticReceipt::NmiAccepted(*gate)
                }
                OriginalTimingNmiPhase::HandlerCompleted => {
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted
                }
            });
            if matches!(phase, OriginalTimingNmiPhase::HandlerCompleted)
                && native_entry_gate == NmiUpdateGate::Open
            {
                expected_semantic.push(OriginalTimingSemanticReceipt::JoypadPublication(
                    joypad_publications[0],
                ));
            }
        }
        if semantic.contains(&OriginalTimingSemanticReceipt::SelectedGameEntranceReturned) {
            assert!(self
                .game_execution_scheduler
                .selected_game_load_pending_entry_room_load()
                .is_some());
            let insertion = expected_semantic.len()
                - usize::from(matches!(
                    expected_semantic.last(),
                    Some(OriginalTimingSemanticReceipt::NmiAccepted(_))
                ));
            expected_semantic.insert(
                insertion,
                OriginalTimingSemanticReceipt::SelectedGameEntranceReturned,
            );
        }
        expected_semantic.push(OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ));
        assert_eq!(
            semantic, expected_semantic,
            "selected-game waiting slice published an unsupported or reordered semantic vector",
        );
        Some(OriginalTimingNonterminalContinuationPlan {
            semantic,
            timeline,
            nmi,
        })
    }

    pub(super) fn execute_original_timing_nonterminal_continuation<F>(
        &mut self,
        plan: OriginalTimingNonterminalContinuationPlan,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        complete_cpu_slice: F,
    ) where
        F: FnOnce(&mut Self),
    {
        assert_eq!(
            self.original_timing_semantic_receipts
                .as_ref()
                .expect("validated suspended caller lost its semantic authority")
                .semantic,
            plan.semantic,
            "validated suspended-caller authority changed before consumption",
        );
        let consumed = self
            .take_original_timing_uninterrupted_main_loop_timeline(
                crate::MainLoopProgress::CallStackContinued,
            )
            .expect("validated suspended-caller timeline disappeared");
        assert_eq!(consumed, plan.timeline);
        self.complete_original_timing_nmi_handler_for_active_scanout(
            plan.nmi.handler_completion,
            input,
            oam_dma_source,
        )
        .assert_no_unclaimed_dialogue_text_dma();
        complete_cpu_slice(self);
        if plan.nmi.publication_pending_at_exit {
            self.capture_and_carry_original_timing_nmi_publication_at_host_return();
        }
        assert!(
            self.original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| receipts.semantic.is_empty()),
            "nonterminal suspended-caller execution left unowned semantic receipts",
        );
        assert_eq!(
            self.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "nonterminal suspended-caller execution retired the common suffix too early",
        );
    }

    /// Build the source-owned post-audio selected-load slice before ordinary
    /// host setup can sample audio or normalize the real scheduler. The
    /// destination and nested Sprite_ResetAll phase come only from the frozen
    /// scheduler continuation, never from mutable gameplay selectors.
    pub(super) fn original_timing_selected_game_load_plan(
        &self,
    ) -> Option<OriginalTimingSelectedGameLoadPlan> {
        if !self.rom_startup_timing()
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            return None;
        }
        let destination = self
            .game_execution_scheduler
            .selected_game_load_destination()?;
        let sprite_reset = self
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset()
            .or_else(|| {
                (destination == SelectedGameLoadDestination::DarkWorldOverworld)
                    .then_some(PreDungeonSpriteResetContinuation::NotApplicable)
            })?;
        assert!(
            matches!(
                (destination, sprite_reset),
                (SelectedGameLoadDestination::Dungeon, _)
                    | (
                        SelectedGameLoadDestination::Message
                            | SelectedGameLoadDestination::DarkWorldOverworld,
                        PreDungeonSpriteResetContinuation::NotApplicable
                    )
            ),
            "live post-audio selected-game timing is only proven for the dungeon, Message, and dark-world overworld source routes",
        );
        assert!(
            matches!(
                (destination, sprite_reset),
                (
                    SelectedGameLoadDestination::Dungeon,
                    PreDungeonSpriteResetContinuation::Pending
                        | PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted
                ) | (
                    SelectedGameLoadDestination::DarkWorldOverworld
                        | SelectedGameLoadDestination::Message,
                    PreDungeonSpriteResetContinuation::NotApplicable
                )
            ),
            "selected-game destination and nested Sprite_ResetAll owner diverged",
        );

        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .expect("live selected-game caller lost its semantic authority")
            .semantic
            .clone();
        let sprite_reset_receipts = semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::SpriteResetAllProgress(receipt) => Some(*receipt),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            sprite_reset_receipts.len() <= 1,
            "one selected-game source host cannot publish multiple Sprite_ResetAll checkpoints",
        );
        let progress_receipt = sprite_reset_receipts.first().copied();
        let terminal_timeline = self.original_timing_main_loop_return_timeline();
        // A host that begins with the held acceptance which interrupted the
        // reset re-publishes the already-consumed SpriteDisableAllCompleted
        // checkpoint at that acceptance (route host 618346, after the host-
        // return checkpoint at 618345). It is a restatement, not progress.
        let restated_checkpoint = progress_receipt.is_some_and(|receipt| {
            receipt.progress == SpriteResetAllProgress::SpriteDisableAllCompleted
                && receipt.boundary == OriginalTimingBoundary::NmiAccepted
        }) && self
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset()
            == Some(PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted);
        let progress_for_transition = if restated_checkpoint {
            None
        } else {
            progress_receipt
        };

        let mut scheduler_before_transition = self.game_execution_scheduler;
        scheduler_before_transition.begin_host_frame();
        let mut scheduler_after_transition = scheduler_before_transition;
        // The entrance return or a later reset/caller-return checkpoint is
        // ordered after Dungeon_LoadEntrance. Retire that nested owner on the
        // scheduler probe before applying Sprite_ResetAll or caller-return
        // progress; execution performs the corresponding native room load
        // before installing this final scheduler state.
        let completes_entry_room_load = scheduler_after_transition
            .selected_game_load_pending_entry_room_load()
            .is_some()
            && (progress_receipt.is_some()
                || terminal_timeline.is_some()
                || semantic.contains(&OriginalTimingSemanticReceipt::SelectedGameEntranceReturned));
        if completes_entry_room_load {
            scheduler_after_transition.mark_selected_game_load_entry_room_load_completed();
        }
        let step = scheduler_after_transition
            .advance_selected_game_load_from_source(
                progress_for_transition.map(|receipt| receipt.progress),
                terminal_timeline.is_some(),
            )
            .expect("selected-game source authority reached a non-selected scheduler owner");

        let action = if let Some(timeline) = terminal_timeline {
            assert_eq!(step, StartupSequenceStep::CompleteSelectedGameLoad);
            assert!(
                progress_receipt.is_none() || restated_checkpoint,
                "selected-game terminal return cannot also publish a nested Sprite_ResetAll checkpoint",
            );
            assert_eq!(
                timeline.progress,
                crate::MainLoopProgress::CallStackContinued,
                "terminal selected-game load cannot begin a fresh main-loop iteration",
            );
            // The previous host may have ended mid-call with no NMI; this one
            // then begins with the held acceptance instead of a carried
            // handler (route host 618346).
            let leading_held_acceptance = timeline.nmi_phases_before_return
                == [
                    OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingNmiPhase::HandlerCompleted,
                ];
            assert!(
                leading_held_acceptance
                    || timeline.nmi_phases_before_return
                        == [OriginalTimingNmiPhase::HandlerCompleted],
                "terminal selected-game load must first complete its carried source NMI: {timeline:?}",
            );
            if destination == SelectedGameLoadDestination::Message {
                // Module05's Message tail returns to the main wait; whether
                // the following Open NMI is accepted before the host ends is
                // a wire fact (route host 160303 carries it, 750107 does not).
                assert!(
                    timeline.nmi_phases_after_return.is_empty()
                        || timeline.nmi_phases_after_return
                            == [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)],
                    "terminal Message selected-game load carries at most the following Open acceptance: {timeline:?}",
                );
            } else if destination == SelectedGameLoadDestination::DarkWorldOverworld {
                // The dark-world overworld reload's Module05 tail returns to
                // the main wait as well; whether the following Open NMI lands
                // inside the host is a wire fact (route host 422632).
                assert!(
                    timeline.nmi_phases_after_return.is_empty()
                        || timeline.nmi_phases_after_return
                            == [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)],
                    "terminal dark-world selected-game load carries at most the following Open acceptance: {timeline:?}",
                );
            } else {
                // The dungeon reload's Module05 tail may likewise accept the
                // following Open NMI before the host ends (route hosts 618346
                // and 746289).
                let trailing_open = timeline.nmi_phases_after_return
                    == [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)];
                assert!(
                    timeline.nmi_phases_after_return.is_empty() || trailing_open,
                    "terminal selected-game load cannot execute a post-return NMI: {timeline:?}",
                );
            }
            let trailing_open_acceptance = timeline.nmi_phases_after_return
                == [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)];
            assert_eq!(
                self.pending_main_loop_common_suffix,
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                "terminal selected-game load lost its suspended ZeldaRunGameLoop suffix",
            );
            assert!(!self.main_loop_sprite_preparation_completed);
            assert!(self.original_timing_main_loop_interruption().is_none());
            if leading_held_acceptance {
                assert!(!self.original_timing_nmi_publication_pending);
                assert_eq!(self.original_timing_pending_nmi_update_gate, None);
            } else {
                assert!(self.original_timing_nmi_publication_pending);
                assert_eq!(
                    self.original_timing_pending_nmi_update_gate,
                    Some(NmiUpdateGate::LatchHeld),
                );
            }
            assert!(self.game_state.display.nmi_update_is_latched());
            if !leading_held_acceptance {
                assert!(
                    self.display_snapshot
                        .as_ref()
                        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
                    "terminal selected-game load lost its carried acceptance snapshot",
                );
            }
            let mut expected = Vec::new();
            if restated_checkpoint {
                expected.push(OriginalTimingSemanticReceipt::SpriteResetAllProgress(
                    progress_receipt.expect("restated checkpoint disappeared"),
                ));
            }
            if leading_held_acceptance {
                expected.push(OriginalTimingSemanticReceipt::NmiAccepted(
                    NmiUpdateGate::LatchHeld,
                ));
            }
            expected.extend([
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ]);
            if destination == SelectedGameLoadDestination::Message {
                // Main_ShowTextMessage clears the messaging module inside the
                // Module05 tail; the decoder publishes that as DialogueClosed.
                if trailing_open_acceptance {
                    assert_eq!(
                        self.original_timing_expected_nmi_update_gates.as_slice(),
                        [NmiUpdateGate::LatchHeld, NmiUpdateGate::Open],
                    );
                    expected.push(OriginalTimingSemanticReceipt::NmiAccepted(
                        NmiUpdateGate::Open,
                    ));
                } else {
                    assert_eq!(
                        self.original_timing_expected_nmi_update_gates.as_slice(),
                        [NmiUpdateGate::LatchHeld],
                    );
                }
                expected.push(OriginalTimingSemanticReceipt::DialogueClosed);
            } else if destination == SelectedGameLoadDestination::DarkWorldOverworld
                && timeline.nmi_phases_after_return
                    == [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)]
            {
                assert_eq!(
                    self.original_timing_expected_nmi_update_gates.as_slice(),
                    [NmiUpdateGate::LatchHeld, NmiUpdateGate::Open],
                );
                expected.push(OriginalTimingSemanticReceipt::NmiAccepted(
                    NmiUpdateGate::Open,
                ));
            } else if trailing_open_acceptance {
                assert_eq!(
                    self.original_timing_expected_nmi_update_gates.as_slice(),
                    [NmiUpdateGate::LatchHeld, NmiUpdateGate::Open],
                );
                expected.push(OriginalTimingSemanticReceipt::NmiAccepted(
                    NmiUpdateGate::Open,
                ));
                if destination == SelectedGameLoadDestination::Dungeon
                    && semantic.last()
                        == Some(&OriginalTimingSemanticReceipt::PreDungeonModuleReturned)
                {
                    expected.push(OriginalTimingSemanticReceipt::PreDungeonModuleReturned);
                }
            } else {
                assert_eq!(
                    self.original_timing_expected_nmi_update_gates.as_slice(),
                    [NmiUpdateGate::LatchHeld],
                );
                if destination == SelectedGameLoadDestination::Dungeon
                    && semantic.last()
                        == Some(&OriginalTimingSemanticReceipt::PreDungeonModuleReturned)
                {
                    // Module_PreDungeon's 07/0f publication is reported for the
                    // Module05 loader too (route host 2292); the terminal
                    // completion below performs exactly that publication.
                    expected.push(OriginalTimingSemanticReceipt::PreDungeonModuleReturned);
                }
            }
            assert_eq!(
                semantic, expected,
                "terminal selected-game load published an unsupported or reordered semantic vector",
            );
            OriginalTimingSelectedGameLoadAction::Terminal {
                semantic,
                timeline,
                sprite_reset,
            }
        } else if let Some(receipt) = progress_receipt {
            assert_eq!(step, StartupSequenceStep::SelectedGameLoadWaiting);
            assert_eq!(destination, SelectedGameLoadDestination::Dungeon);
            assert_eq!(sprite_reset, PreDungeonSpriteResetContinuation::Pending);
            assert_eq!(
                receipt.progress,
                SpriteResetAllProgress::SpriteDisableAllCompleted,
            );
            let timeline = self
                .original_timing_uninterrupted_main_loop_timeline(
                    crate::MainLoopProgress::CallStackContinued,
                )
                .expect(
                    "selected-game Sprite_ResetAll checkpoint lost its continued-call timeline",
                );
            assert!(timeline.nmi_phases_after_progress.is_empty());
            assert_eq!(
                self.pending_main_loop_common_suffix,
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            );
            assert!(!self.main_loop_sprite_preparation_completed);
            assert!(self.original_timing_main_loop_interruption().is_none());
            assert!(self.game_state.display.nmi_update_is_latched());
            let nmi = classify_original_timing_nmi_phases_with_ownership(
                self.original_timing_nmi_publication_pending,
                &timeline.nmi_phases_before_progress,
            );
            let expected = match receipt.boundary {
                OriginalTimingBoundary::NmiAccepted
                    if timeline.nmi_phases_before_progress.first()
                        == Some(&OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld)) =>
                {
                    // The previous host ended mid-call without an NMI: this
                    // host begins with the held acceptance, completes its
                    // handler, and the reset reaches its checkpoint at the
                    // trailing held acceptance (route host 746288).
                    assert_eq!(
                        timeline.nmi_phases_before_progress,
                        [
                            OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                            OriginalTimingNmiPhase::HandlerCompleted,
                            OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                        ],
                    );
                    assert_eq!(
                        nmi.handler_completion,
                        OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence,
                    );
                    assert!(nmi.publication_pending_at_exit);
                    assert!(!self.original_timing_nmi_publication_pending);
                    assert_eq!(self.original_timing_pending_nmi_update_gate, None);
                    assert_eq!(
                        self.original_timing_expected_nmi_update_gates.as_slice(),
                        [NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld],
                    );
                    vec![
                        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                        OriginalTimingSemanticReceipt::SpriteResetAllProgress(receipt),
                        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                        OriginalTimingSemanticReceipt::MainLoopProgress(
                            crate::MainLoopProgress::CallStackContinued,
                        ),
                    ]
                }
                OriginalTimingBoundary::NmiAccepted => {
                    // The carried handler completes, the reset runs to its
                    // checkpoint, and the trailing held acceptance interrupts
                    // it there (route host 2235).
                    assert_eq!(
                        timeline.nmi_phases_before_progress,
                        [
                            OriginalTimingNmiPhase::HandlerCompleted,
                            OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                        ],
                    );
                    assert_eq!(
                        nmi.handler_completion,
                        OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry,
                    );
                    assert!(nmi.publication_pending_at_exit);
                    assert!(self.original_timing_nmi_publication_pending);
                    assert_eq!(
                        self.original_timing_pending_nmi_update_gate,
                        Some(NmiUpdateGate::LatchHeld),
                    );
                    assert_eq!(
                        self.original_timing_expected_nmi_update_gates.as_slice(),
                        [NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld],
                    );
                    vec![
                        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                        OriginalTimingSemanticReceipt::SpriteResetAllProgress(receipt),
                        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                        OriginalTimingSemanticReceipt::MainLoopProgress(
                            crate::MainLoopProgress::CallStackContinued,
                        ),
                    ]
                }
                OriginalTimingBoundary::HostReturn
                    if self.original_timing_nmi_publication_pending =>
                {
                    // The previous host's trailing held acceptance carried
                    // its handler into this host, which completes it and ends
                    // at the checkpoint without accepting another NMI (route
                    // host 638770, a dark-world death's dungeon reload).
                    assert_eq!(
                        timeline.nmi_phases_before_progress,
                        [OriginalTimingNmiPhase::HandlerCompleted],
                    );
                    assert_eq!(
                        nmi.handler_completion,
                        OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry,
                    );
                    assert!(!nmi.publication_pending_at_exit);
                    assert_eq!(
                        self.original_timing_pending_nmi_update_gate,
                        Some(NmiUpdateGate::LatchHeld),
                    );
                    assert_eq!(
                        self.original_timing_expected_nmi_update_gates.as_slice(),
                        [NmiUpdateGate::LatchHeld],
                    );
                    vec![
                        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                        OriginalTimingSemanticReceipt::SpriteResetAllProgress(receipt),
                        OriginalTimingSemanticReceipt::MainLoopProgress(
                            crate::MainLoopProgress::CallStackContinued,
                        ),
                    ]
                }
                OriginalTimingBoundary::HostReturn => {
                    // The previous host ended mid-call without an NMI; this
                    // host begins with the held acceptance, completes its
                    // handler, and ends at the checkpoint with no trailing
                    // acceptance (route host 618345, a dark-world death's
                    // dungeon reload).
                    assert_eq!(
                        timeline.nmi_phases_before_progress,
                        [
                            OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                            OriginalTimingNmiPhase::HandlerCompleted,
                        ],
                    );
                    assert_eq!(
                        nmi.handler_completion,
                        OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence,
                    );
                    assert!(!nmi.publication_pending_at_exit);
                    assert_eq!(
                        self.original_timing_expected_nmi_update_gates.as_slice(),
                        [NmiUpdateGate::LatchHeld],
                    );
                    vec![
                        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                        OriginalTimingSemanticReceipt::SpriteResetAllProgress(receipt),
                        OriginalTimingSemanticReceipt::MainLoopProgress(
                            crate::MainLoopProgress::CallStackContinued,
                        ),
                    ]
                }
            };
            assert_original_timing_carry_in_handler_has_receptive_display(
                nmi,
                self.display_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
            );
            assert_eq!(
                semantic, expected,
                "selected-game Sprite_ResetAll checkpoint published an unsupported or reordered semantic vector",
            );
            OriginalTimingSelectedGameLoadAction::SpriteResetCheckpoint {
                semantic,
                timeline,
                nmi,
                receipt,
            }
        } else {
            assert_eq!(step, StartupSequenceStep::SelectedGameLoadWaiting);
            OriginalTimingSelectedGameLoadAction::Nonterminal(
                self.original_timing_selected_game_nonterminal_plan()
                    .expect("live selected-game waiting slice lost its continued-call owner"),
            )
        };

        Some(OriginalTimingSelectedGameLoadPlan {
            action,
            destination,
            message_interface_published: self
                .game_execution_scheduler
                .selected_game_load_message_interface_published(),
            completes_entry_room_load,
            scheduler_before_transition,
            scheduler_after_transition,
        })
    }

    /// Build the exact source boundary which leaves the selected-load
    /// decompressor and enters Module_PreDungeon. This probe includes host
    /// normalization and the numeric pre-audio scheduler transition, but it
    /// mutates only a copy until the handler and CPU boundary have completed.
    pub(super) fn original_timing_begin_selected_game_load_plan(
        &self,
    ) -> Option<OriginalTimingBeginSelectedGameLoadPlan> {
        if !self.rom_startup_timing()
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || self
                .game_execution_scheduler
                .selected_game_load_destination()
                .is_none()
            || self
                .game_execution_scheduler
                .selected_game_load_after_pre_dungeon_audio_sprite_reset()
                .is_some()
        {
            return None;
        }

        let mut scheduler_before_transition = self.game_execution_scheduler;
        scheduler_before_transition.begin_host_frame();
        let mut scheduler_after_transition = scheduler_before_transition;
        if scheduler_after_transition.advance_startup_sequence()
            != Some(StartupSequenceStep::BeginPreDungeonAudio)
        {
            return None;
        }

        let destination = self
            .game_execution_scheduler
            .selected_game_load_destination()
            .expect("pre-dungeon-audio boundary lost its frozen destination");
        if destination != SelectedGameLoadDestination::Dungeon {
            // These destinations have no pre-dungeon audio boundary. Their
            // source publication/return plans own the caller independently
            // of the compatibility decompression count.
            return None;
        }
        let timeline = self
            .original_timing_uninterrupted_main_loop_timeline(
                crate::MainLoopProgress::CallStackContinued,
            )
            .expect("live pre-dungeon-audio boundary omitted its continued-call timeline");
        assert!(
            timeline.nmi_phases_after_progress.is_empty(),
            "pre-dungeon-audio boundary cannot publish hardware phases after its terminal progress fact: {timeline:?}",
        );
        assert_eq!(
            self.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "pre-dungeon-audio boundary lost its suspended ZeldaRunGameLoop suffix",
        );
        // The previous decompressor host may end with a trailing Held
        // acceptance whose handler completes at the top of this boundary host
        // (route host 1344254: [NmiHandlerCompleted, NmiAccepted(LatchHeld),
        // CallStackContinued] after a double-acceptance host).
        let carried_handler = self.original_timing_nmi_publication_pending;
        if carried_handler {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate,
                Some(NmiUpdateGate::LatchHeld),
                "pre-dungeon-audio boundary's carried handler must belong to a Held acceptance",
            );
        } else {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate, None,
                "pre-dungeon-audio boundary cannot retain an NMI disposition without a carried handler",
            );
        }
        assert!(
            self.game_state.display.nmi_update_is_latched(),
            "pre-dungeon-audio boundary source Held acceptances require the native NMI latch to be held",
        );
        // The Module05 decompressor's boundary host holds one NMI whose
        // handler completes in-host; whether a second held acceptance also
        // lands before the host returns depends only on where the ~60.1 Hz
        // hardware cadence falls (two at route host 160282, one at 638711).
        // With a carried handler the one acceptance of this host is the
        // trailing one.
        // The carried acceptance keeps its own installed disposition, so
        // with a carried handler the gate list holds it plus this host's
        // trailing acceptance when one lands (route host 1344254:
        // [LatchHeld, LatchHeld]).
        let trailing_held = if carried_handler {
            timeline.nmi_phases_before_progress.last()
                == Some(&OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld))
        } else {
            self.original_timing_expected_nmi_update_gates.len() == 2
        };
        assert!(
            if carried_handler {
                self.original_timing_expected_nmi_update_gates.len()
                    == 1 + usize::from(trailing_held)
                    && self
                        .original_timing_expected_nmi_update_gates
                        .iter()
                        .all(|gate| *gate == NmiUpdateGate::LatchHeld)
            } else {
                matches!(
                    self.original_timing_expected_nmi_update_gates.as_slice(),
                    [NmiUpdateGate::LatchHeld]
                        | [NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld]
                )
            },
            "pre-dungeon-audio boundary lost its installed source acceptance dispositions: {:?}",
            self.original_timing_expected_nmi_update_gates,
        );
        let mut expected_phases = if carried_handler {
            vec![OriginalTimingNmiPhase::HandlerCompleted]
        } else {
            vec![
                OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                OriginalTimingNmiPhase::HandlerCompleted,
            ]
        };
        if trailing_held {
            expected_phases.push(OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld));
        }
        assert_eq!(
            timeline.nmi_phases_before_progress, expected_phases,
            "pre-dungeon-audio boundary must complete its same-host held NMI before carrying the trailing held acceptance: {timeline:?}",
        );
        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .expect("live pre-dungeon-audio boundary lost its semantic authority")
            .semantic
            .clone();
        let mut expected_semantic = if carried_handler {
            vec![OriginalTimingSemanticReceipt::NmiHandlerCompleted]
        } else {
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            ]
        };
        let entrance_scroll_published =
            semantic.contains(&OriginalTimingSemanticReceipt::SelectedGameEntranceScrollPublished);
        if entrance_scroll_published {
            assert!(
                trailing_held,
                "entrance scroll checkpoint requires its trailing NMI"
            );
            expected_semantic
                .push(OriginalTimingSemanticReceipt::SelectedGameEntranceScrollPublished);
        }
        let entrance_before_selection =
            semantic.contains(&OriginalTimingSemanticReceipt::SelectedGameEntranceBeforeSelection);
        let entrance_returned =
            semantic.contains(&OriginalTimingSemanticReceipt::SelectedGameEntranceReturned);
        assert!(!(entrance_before_selection && (entrance_scroll_published || entrance_returned)));
        if entrance_returned {
            expected_semantic.push(OriginalTimingSemanticReceipt::SelectedGameEntranceReturned);
        }
        if trailing_held {
            expected_semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::LatchHeld,
            ));
        }
        if entrance_before_selection {
            expected_semantic
                .push(OriginalTimingSemanticReceipt::SelectedGameEntranceBeforeSelection);
        }
        expected_semantic.push(OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ));
        assert_eq!(
            semantic, expected_semantic,
            "pre-dungeon-audio boundary published an unsupported or reordered semantic vector",
        );
        let nmi = classify_original_timing_nmi_phases_with_ownership(
            carried_handler,
            &timeline.nmi_phases_before_progress,
        );
        if carried_handler {
            assert!(
                nmi.handler_completion.completed()
                    && nmi.handler_completion
                        != OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence,
                "pre-dungeon-audio boundary lost its carried handler ownership: {timeline:?}",
            );
        } else {
            assert_eq!(
                nmi.handler_completion,
                OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence,
                "pre-dungeon-audio boundary lost same-host handler ownership: {timeline:?}",
            );
        }
        assert_eq!(
            nmi.publication_pending_at_exit, trailing_held,
            "pre-dungeon-audio boundary must carry exactly its trailing source NMI: {timeline:?}",
        );

        Some(OriginalTimingBeginSelectedGameLoadPlan {
            entrance_scroll_published,
            entrance_before_selection,
            entrance_returned,
            semantic,
            timeline,
            nmi,
            destination,
            scheduler_before_transition,
            scheduler_after_transition,
        })
    }

    /// Build the exact source host which completes a carried Open NMI, starts
    /// a fresh ZeldaRunGameLoop iteration, and then returns from a suspended
    /// caller with one Held NMI accepted for the following host.  This is the
    /// IterationStarted counterpart to `OriginalTimingNonterminalContinuationPlan`:
    /// the whole semantic vector is validated before the caller prefix, NMI,
    /// display, or suffix ownership can move.
    pub(super) fn original_timing_iteration_started_continuation_plan(
        &self,
    ) -> Option<OriginalTimingIterationStartedContinuationPlan> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let timeline = self
            .original_timing_uninterrupted_main_loop_timeline(
                crate::MainLoopProgress::IterationStarted,
            )
            .expect("live suspended caller omitted its iteration-start timeline");
        // The preceding main wait either carried its Open acceptance into
        // this host (boot: [HandlerCompleted]) or cleared the latch in its
        // own host so the Open NMI is accepted here (save-quit reset re-entry,
        // route host 159405: [Accepted(Open), HandlerCompleted]).
        let carried_open = self.original_timing_nmi_publication_pending;
        if carried_open {
            assert_eq!(
                timeline.nmi_phases_before_progress,
                [OriginalTimingNmiPhase::HandlerCompleted],
                "a suspended fresh iteration requires exactly its carried Open handler before the main-loop prefix",
            );
        } else {
            assert_eq!(
                timeline.nmi_phases_before_progress,
                [
                    OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open),
                    OriginalTimingNmiPhase::HandlerCompleted,
                ],
                "a suspended fresh iteration without a carried publication must accept and complete its Open NMI in-host before the main-loop prefix",
            );
        }
        // The suspended iteration usually ends at its trailing held
        // acceptance; the intro module's long triforce-thread initializer can
        // also run the host out with no NMI at all (route host 965252).
        let trailing_held = timeline.nmi_phases_after_progress
            == [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld)];
        assert!(
            trailing_held || timeline.nmi_phases_after_progress.is_empty(),
            "a suspended fresh iteration may only carry its trailing Held acceptance",
        );
        assert!(
            self.pending_main_loop_common_suffix.is_none(),
            "a fresh suspended iteration cannot begin over another main-loop suffix",
        );
        // This flag may still describe the preceding iteration's completed
        // suffix (host 221 in the intro path).  The CPU callback below resets
        // it at the exact fresh ZeldaRunGameLoop prefix before arming the new
        // suffix owner.
        assert!(
            self.game_execution_scheduler.is_idle(),
            "a fresh suspended iteration overlaps translated scheduled work",
        );
        assert!(
            self.original_timing_main_loop_interruption().is_none(),
            "an uninterrupted fresh suspended iteration cannot also publish a main-loop interruption",
        );
        if carried_open {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate,
                Some(NmiUpdateGate::Open),
                "a fresh suspended iteration requires the carried Open acceptance from the preceding main wait",
            );
            assert!(
                self.display_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
                "a fresh suspended iteration lost its carried Open acceptance snapshot",
            );
        } else {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate, None,
                "a fresh suspended iteration accepting its Open NMI in-host cannot also carry a pending gate",
            );
        }
        assert!(
            !self.game_state.display.nmi_update_is_latched(),
            "a fresh suspended iteration's Open handler disagrees with the native NMI latch",
        );
        if trailing_held {
            assert_eq!(
                self.original_timing_expected_nmi_update_gates.as_slice(),
                [NmiUpdateGate::Open, NmiUpdateGate::LatchHeld],
                "a fresh suspended iteration lost its exact Open-then-Held gate authority",
            );
        } else {
            assert_eq!(
                self.original_timing_expected_nmi_update_gates.as_slice(),
                [NmiUpdateGate::Open],
                "an open-ended fresh suspended iteration lost its exact Open gate authority",
            );
        }

        let before_progress = classify_original_timing_nmi_phases_with_ownership(
            carried_open,
            &timeline.nmi_phases_before_progress,
        );
        assert_eq!(
            before_progress,
            OriginalTimingNmiPhaseClassification {
                handler_completion: if carried_open {
                    OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry
                } else {
                    OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence
                },
                publication_pending_at_exit: false,
            },
        );
        let after_progress = classify_original_timing_nmi_phases_with_ownership(
            false,
            &timeline.nmi_phases_after_progress,
        );
        assert_eq!(
            after_progress,
            OriginalTimingNmiPhaseClassification {
                handler_completion: OriginalTimingNmiHandlerCompletionOwner::None,
                publication_pending_at_exit: trailing_held,
            },
        );

        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .expect("live fresh suspended iteration lost its semantic authority")
            .semantic
            .clone();
        let body = if carried_open {
            semantic.as_slice()
        } else {
            let [OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open), rest @ ..] =
                semantic.as_slice()
            else {
                panic!(
                    "a fresh suspended iteration accepting its Open NMI in-host must publish that acceptance first: {semantic:?}"
                );
            };
            rest
        };
        assert!(
            matches!(
                body,
                [
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                    OriginalTimingSemanticReceipt::JoypadPublication(_),
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::IterationStarted
                    ),
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                ]
            ) || (!trailing_held
                && matches!(
                    body,
                    [
                        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                        OriginalTimingSemanticReceipt::JoypadPublication(_),
                        OriginalTimingSemanticReceipt::MainLoopProgress(
                            crate::MainLoopProgress::IterationStarted
                        ),
                    ]
                )),
            "a fresh suspended iteration published an unsupported or reordered semantic vector: {semantic:?}",
        );

        Some(OriginalTimingIterationStartedContinuationPlan {
            semantic,
            timeline,
            before_progress,
            after_progress,
        })
    }

    pub(super) fn execute_original_timing_iteration_started_continuation<F>(
        &mut self,
        plan: OriginalTimingIterationStartedContinuationPlan,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        complete_cpu_slice: F,
    ) where
        F: FnOnce(&mut Self),
    {
        assert_eq!(
            self.original_timing_semantic_receipts
                .as_ref()
                .expect("validated fresh suspended iteration lost its semantic authority")
                .semantic,
            plan.semantic,
            "validated fresh suspended-iteration authority changed before consumption",
        );
        let consumed = self
            .take_original_timing_uninterrupted_main_loop_timeline(
                crate::MainLoopProgress::IterationStarted,
            )
            .expect("validated fresh suspended-iteration timeline disappeared");
        assert_eq!(consumed, plan.timeline);
        self.complete_original_timing_nmi_handler_for_active_scanout(
            plan.before_progress.handler_completion,
            input,
            oam_dma_source,
        )
        .assert_no_unclaimed_dialogue_text_dma();
        complete_cpu_slice(self);
        if plan.after_progress.publication_pending_at_exit {
            self.capture_and_carry_original_timing_nmi_publication_at_host_return();
        }
        assert!(
            self.original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| receipts.semantic.is_empty()),
            "fresh suspended-iteration execution left unowned semantic receipts",
        );
        assert_eq!(
            self.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "fresh suspended-iteration execution lost its common suffix",
        );
    }

    pub(super) fn original_timing_intro_poly_plan(
        &self,
        run_what: u8,
    ) -> Option<OriginalTimingIntroPolyPlan> {
        if !self.rom_startup_timing()
            || self.intro_poly_thread_initialization_phase == 0
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            // The save-quit reset hold owns the CPU until its terminal
            // return; the poly phase armed by its reset writes begins only
            // afterward (route hosts 159380-159401).
            || self.save_quit_reset_hold
        {
            return None;
        }
        assert!(
            run_what & crate::RUN_MAIN != 0,
            "live intro-poly authority cannot be consumed by a host policy which omits main execution",
        );
        assert!(
            rom_intro_poly_initialization_is_active(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            ),
            "live intro-poly continuation escaped its source module owner",
        );
        if matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishSpriteMain {
                caller: SpriteMainCpuCaller::SaveAndQuit { .. },
                ..
            })
        ) {
            // The save-quit reset's suspended Module17 caller still owns the
            // continued stack: its Sprite_Main/LinkOam suffix returns before
            // the ROM enters the intro-poly initialization (route host
            // 159334).
            return None;
        }
        assert!(
            self.game_execution_scheduler.is_idle(),
            "the suspended intro-poly caller overlaps translated scheduled work",
        );
        assert_eq!(
            self.intro_initialization_work_frames_pending, 0,
            "the intro-poly caller overlaps another suspended intro initialization",
        );
        assert_eq!(
            self.intro_memory_darken_frame_delay, 0,
            "the intro-poly caller overlaps the suspended intro memory-darken caller",
        );

        match self.intro_poly_thread_initialization_phase {
            3 if self.original_timing_main_loop_progress()
                == Some(crate::MainLoopProgress::CallStackContinued) =>
            {
                // The iteration which armed the poly thread is still
                // finishing its interrupted common suffix (the save-quit
                // reset's terminal host left NMI_PrepareSprites pending);
                // the ordinary continued-return owner completes it and the
                // poly thread waits for the next fresh iteration (route host
                // 159404).
                assert_eq!(
                    self.pending_main_loop_common_suffix,
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                    "an intro-poly arming iteration continued without its pending common suffix",
                );
                None
            }
            3 => self
                .original_timing_iteration_started_continuation_plan()
                .map(OriginalTimingIntroPolyPlan::AwaitingIterationStart),
            1 => {
                if let Some(timeline) = self.original_timing_main_loop_return_timeline() {
                    assert_eq!(
                        timeline.progress,
                        crate::MainLoopProgress::CallStackContinued,
                        "terminal intro-poly caller cannot begin another main-loop iteration",
                    );
                    if self.original_timing_nmi_publication_pending {
                        // The Held acceptance was carried in from the
                        // preceding suspended host (save-quit reset
                        // re-entry, route host 159407).
                        assert_eq!(
                            timeline.nmi_phases_before_return,
                            [OriginalTimingNmiPhase::HandlerCompleted],
                            "terminal intro-poly caller with a carried Held acceptance completes exactly that handler before returning",
                        );
                    } else {
                        assert_eq!(
                            timeline.nmi_phases_before_return,
                            [
                                OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                                OriginalTimingNmiPhase::HandlerCompleted,
                            ],
                            "terminal intro-poly caller requires its same-host Held handler before returning",
                        );
                    }
                    assert_eq!(
                        timeline.nmi_phases_after_return,
                        [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)],
                        "terminal intro-poly caller may only carry the following Open acceptance",
                    );
                    assert_eq!(
                        self.pending_main_loop_common_suffix,
                        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                        "terminal intro-poly caller lost its ordinary main-loop suffix",
                    );
                    assert!(!self.main_loop_sprite_preparation_completed);
                    assert!(self.game_state.display.nmi_update_is_latched());
                    assert!(self.original_timing_main_loop_interruption().is_none());
                    let semantic = &self
                        .original_timing_semantic_receipts
                        .as_ref()
                        .expect("terminal intro-poly caller lost its semantic authority")
                        .semantic;
                    if self.original_timing_nmi_publication_pending {
                        assert_eq!(
                            self.original_timing_pending_nmi_update_gate,
                            Some(NmiUpdateGate::LatchHeld),
                            "terminal intro-poly caller with a carried publication must carry the Held acceptance",
                        );
                        assert_eq!(
                            self.original_timing_expected_nmi_update_gates.as_slice(),
                            [NmiUpdateGate::LatchHeld, NmiUpdateGate::Open],
                        );
                        assert_eq!(
                            *semantic,
                            [
                                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                                OriginalTimingSemanticReceipt::MainLoopProgress(
                                    crate::MainLoopProgress::CallStackContinued,
                                ),
                                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                            ],
                            "terminal intro-poly caller (carried Held) published an unsupported or reordered semantic vector",
                        );
                    } else {
                        assert_eq!(self.original_timing_pending_nmi_update_gate, None);
                        assert_eq!(
                            self.original_timing_expected_nmi_update_gates.as_slice(),
                            [NmiUpdateGate::LatchHeld, NmiUpdateGate::Open],
                        );
                        assert_eq!(
                            *semantic,
                            [
                                OriginalTimingSemanticReceipt::NmiAccepted(
                                    NmiUpdateGate::LatchHeld
                                ),
                                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                                OriginalTimingSemanticReceipt::MainLoopProgress(
                                    crate::MainLoopProgress::CallStackContinued,
                                ),
                                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                            ],
                            "terminal intro-poly caller published an unsupported or reordered semantic vector",
                        );
                    }
                    Some(OriginalTimingIntroPolyPlan::Terminal(timeline))
                } else {
                    self.original_timing_nonterminal_continuation_plan()
                        .map(OriginalTimingIntroPolyPlan::Suspended)
                }
            }
            2 => panic!("live intro-poly timing cannot use the legacy numeric intermediate phase"),
            phase => panic!("invalid live intro-poly continuation phase {phase}"),
        }
    }

    pub(super) fn original_timing_intro_memory_darken_plan(
        &self,
    ) -> Option<OriginalTimingIntroMemoryDarkenPlan> {
        if !self.rom_startup_timing()
            || self.intro_memory_darken_frame_delay == 0
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            return None;
        }
        assert!(
            self.game_execution_scheduler.is_idle(),
            "the suspended intro memory-darken caller overlaps translated scheduled work",
        );
        assert_eq!(
            self.intro_initialization_work_frames_pending, 0,
            "the memory-darken caller overlaps another suspended intro initialization",
        );
        assert_eq!(
            self.intro_poly_thread_initialization_phase, 0,
            "the memory-darken caller overlaps the suspended intro poly thread",
        );
        if let Some(timeline) = self.original_timing_main_loop_return_timeline() {
            assert_eq!(
                timeline.progress,
                crate::MainLoopProgress::CallStackContinued,
                "terminal intro memory initialization cannot begin another main-loop iteration",
            );
            assert_eq!(
                timeline.nmi_phases_before_return,
                [OriginalTimingNmiPhase::HandlerCompleted],
                "terminal intro memory initialization requires its carried Held handler before the caller return",
            );
            assert_eq!(
                timeline.nmi_phases_after_return,
                [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)],
                "terminal intro memory initialization requires exactly one following Open acceptance",
            );
            assert_eq!(
                self.pending_main_loop_common_suffix,
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                "terminal intro memory initialization lost its ordinary main-loop suffix",
            );
            assert!(
                !self.main_loop_sprite_preparation_completed,
                "terminal intro memory initialization would repeat sprite preparation",
            );
            assert!(self.original_timing_nmi_publication_pending);
            assert_eq!(
                self.original_timing_pending_nmi_update_gate,
                Some(NmiUpdateGate::LatchHeld),
            );
            assert!(self.game_state.display.nmi_update_is_latched());
            assert!(
                self.display_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
                "terminal intro memory initialization lost its carried acceptance snapshot",
            );
            assert_eq!(
                self.original_timing_expected_nmi_update_gates.as_slice(),
                [NmiUpdateGate::LatchHeld, NmiUpdateGate::Open],
            );
            assert_eq!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .expect("terminal intro memory initialization lost its semantic authority")
                    .semantic,
                [
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::CallStackContinued,
                    ),
                    OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                ],
                "terminal intro memory initialization published an unsupported or reordered semantic vector",
            );
            return Some(OriginalTimingIntroMemoryDarkenPlan::Terminal(timeline));
        }
        self.original_timing_nonterminal_continuation_plan()
            .map(OriginalTimingIntroMemoryDarkenPlan::Nonterminal)
    }

    pub(super) fn take_original_timing_save_quit_reset_state_published(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let mut published = false;
        receipts.semantic.retain(|receipt| {
            if *receipt == OriginalTimingSemanticReceipt::SaveQuitResetStatePublished {
                assert!(!published, "save-quit reset state publication replayed");
                published = true;
                false
            } else {
                true
            }
        });
        published
    }

    pub(super) fn take_original_timing_file_select_low_wram_publication(
        &mut self,
    ) -> Option<OriginalTimingFileSelectLowWramPublication> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let mut publication = None;
        receipts.semantic.retain(|receipt| {
            let current = match receipt {
                OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramClearProgress(progress) => {
                    Some(OriginalTimingFileSelectLowWramPublication::Progress(
                        *progress,
                    ))
                }
                OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramCleared => {
                    Some(OriginalTimingFileSelectLowWramPublication::Complete)
                }
                _ => None,
            };
            if let Some(current) = current {
                assert!(
                    publication.replace(current).is_none(),
                    "file-select low-WRAM publication replayed in one host"
                );
                false
            } else {
                true
            }
        });
        publication
    }

    pub(super) fn take_original_timing_triforce_room_case2_palette_progress(
        &mut self,
    ) -> Option<TriforceRoomCase2PaletteProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let mut progress = None;
        receipts.semantic.retain(|receipt| {
            if let OriginalTimingSemanticReceipt::TriforceRoomCase2PaletteProgress(value) = receipt
            {
                assert!(
                    progress.replace(*value).is_none(),
                    "Triforce case-2 palette progress replayed in one host",
                );
                false
            } else {
                true
            }
        });
        progress
    }

    pub(super) fn original_timing_triforce_room_case2_palette_progress(
        &self,
    ) -> Option<TriforceRoomCase2PaletteProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        self.original_timing_semantic_receipts
            .as_ref()?
            .semantic
            .iter()
            .find_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::TriforceRoomCase2PaletteProgress(progress) => {
                    Some(*progress)
                }
                _ => None,
            })
    }

    pub(super) fn take_original_timing_credits_scene_load_progress(
        &mut self,
    ) -> Option<CreditsSceneLoadProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let mut progress = None;
        receipts.semantic.retain(|receipt| {
            if let OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(value) = receipt {
                assert!(
                    progress.replace(*value).is_none(),
                    "credits scene-load progress replayed in one host",
                );
                false
            } else {
                true
            }
        });
        progress
    }

    pub(super) fn original_timing_credits_scene_load_progress(
        &self,
    ) -> Option<CreditsSceneLoadProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        self.original_timing_semantic_receipts
            .as_ref()?
            .semantic
            .iter()
            .find_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(progress) => {
                    Some(*progress)
                }
                _ => None,
            })
    }

    pub(super) fn take_original_timing_credits_end_sequence_32_progress(
        &mut self,
    ) -> Option<CreditsEndSequence32ProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let mut progress = None;
        receipts.semantic.retain(|receipt| {
            if let OriginalTimingSemanticReceipt::CreditsEndSequence32Progress(value) = receipt {
                assert!(
                    progress.replace(*value).is_none(),
                    "credits finale save progress replayed in one host",
                );
                false
            } else {
                true
            }
        });
        progress
    }

    pub(super) fn original_timing_credits_end_sequence_32_progress(
        &self,
    ) -> Option<CreditsEndSequence32ProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        self.original_timing_semantic_receipts
            .as_ref()?
            .semantic
            .iter()
            .find_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::CreditsEndSequence32Progress(progress) => {
                    Some(*progress)
                }
                _ => None,
            })
    }

    pub(super) fn original_timing_save_quit_reset_plan(
        &self,
    ) -> Option<OriginalTimingSaveQuitResetPlan> {
        if !self.rom_startup_timing()
            || !self.save_quit_reset_hold
            || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            return None;
        }
        assert!(
            self.game_execution_scheduler.is_idle(),
            "the suspended save-quit reset caller overlaps translated scheduled work",
        );
        assert_eq!(
            self.intro_memory_darken_frame_delay, 0,
            "the save-quit reset caller overlaps the suspended intro memory-darken caller",
        );
        assert!(
            self.intro_poly_thread_initialization_phase == 0
                || self.save_quit_reset_state_published
                || (self.intro_poly_thread_initialization_phase == 3
                    && (
                        self.game_state.frame.main_module,
                        self.game_state.frame.submodule
                    ) == (0x17, 2)),
            "the save-quit reset caller overlaps the suspended intro poly thread",
        );
        if self
            .original_timing_semantic_receipts
            .as_ref()
            .is_some_and(|receipts| {
                receipts
                    .semantic
                    .contains(&OriginalTimingSemanticReceipt::SaveQuitIntroMemoryReturned)
            })
        {
            assert_eq!(
                (
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule
                ),
                (0x17, 1)
            );
            return Some(OriginalTimingSaveQuitResetPlan::IntroMemoryReturned(
                self.original_timing_nonterminal_continuation_plan_with_receipt_before_progress(
                    Some(OriginalTimingSemanticReceipt::SaveQuitIntroMemoryReturned),
                )
                .expect("save-quit intro-memory return lost its continued-call owner"),
            ));
        }
        let reset_state_published =
            self.original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts
                        .semantic
                        .contains(&OriginalTimingSemanticReceipt::SaveQuitResetStatePublished)
                });
        if reset_state_published {
            assert!(
                !self.save_quit_reset_state_published,
                "save-quit reset state publication repeated after native application",
            );
            assert!(
                self.original_timing_main_loop_return_timeline().is_none(),
                "save-quit reset state publication cannot also return the song-upload caller",
            );
            return Some(OriginalTimingSaveQuitResetPlan::ResetStatePublished(
                self.original_timing_nonterminal_continuation_plan_with_receipt_before_progress(
                    Some(OriginalTimingSemanticReceipt::SaveQuitResetStatePublished),
                )
                .expect("save-quit reset state publication lost its continued-call owner"),
            ));
        }
        if let Some(timeline) = self.original_timing_main_loop_return_timeline() {
            assert_eq!(
                timeline.progress,
                crate::MainLoopProgress::CallStackContinued,
                "the terminal save-quit reset cannot begin another main-loop iteration",
            );
            return Some(OriginalTimingSaveQuitResetPlan::Terminal(timeline));
        }
        if self.original_timing_owes_sprite_main_return()
            && self.original_timing_main_loop_interruption()
                == Some(crate::MainLoopInterruption::SpritePreparation)
        {
            // The upload finished mid-host: Module17's Sprite_Main returned
            // and the shared suffix was interrupted by a Held acceptance
            // (route host 159403: [SpriteMainReturned, NmiAccepted(LatchHeld),
            // MainLoopInterrupted(SpritePreparation), CallStackContinued]).
            return Some(OriginalTimingSaveQuitResetPlan::TerminalInterruptedSuffix);
        }
        if !self.original_timing_nmi_publication_pending
            && self
                .original_timing_uninterrupted_main_loop_timeline(
                    crate::MainLoopProgress::CallStackContinued,
                )
                .is_some_and(|timeline| {
                    timeline.nmi_phases_before_progress.is_empty()
                        && timeline.nmi_phases_after_progress.is_empty()
                })
        {
            assert!(
                !self.original_timing_owes_sprite_main_return(),
                "an NMI-masked save-quit hold cannot own a Sprite_Main return",
            );
            return Some(OriginalTimingSaveQuitResetPlan::NmiMaskedHold);
        }
        self.original_timing_nonterminal_continuation_plan()
            .map(OriginalTimingSaveQuitResetPlan::Nonterminal)
    }

    /// Take the ordered hardware phases around the source proof that an
    /// already-running ZeldaRunGameLoop returned to its NMI wait.
    ///
    /// This boundary can occur between two accepted NMIs in one libretro host
    /// call. Keeping the split here prevents a translated caller from moving
    /// `NMI_PrepareSprites`/the `$12` clear before the NMI which interrupted
    /// it, while still carrying a later accepted handler into the next host.
    pub(super) fn original_timing_main_loop_return_timeline(
        &self,
    ) -> Option<OriginalTimingMainLoopReturnTimeline> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_ref()?;
        let return_indices = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| {
                original_timing_receipt_completes_main_loop_common_suffix(receipt).then_some(index)
            })
            .collect::<Vec<_>>();
        if return_indices.is_empty() {
            return None;
        }
        assert_eq!(
            return_indices.len(),
            1,
            "one live host cannot return the same main-loop iteration more than once",
        );
        let return_index = return_indices[0];
        let progress = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::MainLoopProgress(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            progress.len(),
            1,
            "one live host must publish exactly one main-loop progress receipt",
        );
        let (_, progress) = progress[0];
        let mut nmi_phases_before_return = Vec::new();
        let mut nmi_phases_after_return = Vec::new();
        let mut first_nmi_phase_index = None;
        for (index, receipt) in receipts.semantic.iter().enumerate() {
            let phase = match receipt {
                OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                    OriginalTimingNmiPhase::Accepted(*gate)
                }
                OriginalTimingSemanticReceipt::NmiHandlerCompleted => {
                    OriginalTimingNmiPhase::HandlerCompleted
                }
                _ => continue,
            };
            if index < return_index {
                first_nmi_phase_index.get_or_insert(index);
                nmi_phases_before_return.push(phase);
            } else {
                nmi_phases_after_return.push(phase);
            }
        }
        let sprite_main_returned_before_nmi = receipts
            .semantic
            .iter()
            .position(|receipt| *receipt == OriginalTimingSemanticReceipt::SpriteMainReturned)
            .is_some_and(|sprite_main_index| {
                sprite_main_index < return_index
                    && first_nmi_phase_index.is_some_and(|nmi_index| sprite_main_index < nmi_index)
            });
        Some(OriginalTimingMainLoopReturnTimeline {
            progress,
            nmi_phases_before_return,
            nmi_phases_after_return,
            sprite_main_returned_before_nmi,
        })
    }

    /// Prove the terminal host of a synchronous chest item-graphics call
    /// without consuming its handler, scheduler, or Sprite_Main authority.
    pub(super) fn original_timing_terminal_ground_item_receipt_plan(
        &self,
    ) -> Option<OriginalTimingTerminalGroundItemReceiptPlan> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let work = self.game_execution_scheduler.current_work()?;
        let continuation = match work {
            GameWorkContinuation::FinishItemReceiptGraphics {
                continuation:
                    continuation @ ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                        ground_apress_tail: Some(_),
                        ..
                    },
            } => continuation,
            _ => return None,
        };
        let timeline = self.original_timing_main_loop_return_timeline()?;

        assert_eq!(
            timeline.progress,
            crate::MainLoopProgress::CallStackContinued,
            "a terminal ground-item graphics caller cannot begin a fresh main-loop iteration",
        );
        // The decompressor's held NMI is either accepted and completed inside
        // this same host, or was accepted at the preceding host's return and
        // carries only its handler completion into this one. Both are exact
        // wire shapes of the same C boundary.
        let carried_handler = self.original_timing_nmi_publication_pending;
        if carried_handler {
            assert_eq!(
                timeline.nmi_phases_before_return,
                [OriginalTimingNmiPhase::HandlerCompleted],
                "a carried ground-item handler must complete without a second acceptance: {timeline:?}",
            );
        } else {
            assert_eq!(
                timeline.nmi_phases_before_return,
                [
                    OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingNmiPhase::HandlerCompleted,
                ],
                "a terminal ground-item graphics caller must complete exactly its same-host held handler: {timeline:?}",
            );
        }
        let trailing_open = matches!(
            timeline.nmi_phases_after_return.as_slice(),
            [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)]
        );
        assert!(
            timeline.nmi_phases_after_return.is_empty() || trailing_open,
            "a ground-item terminal may only carry one trailing Open acceptance: {timeline:?}",
        );
        assert_eq!(
            self.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "a terminal ground-item graphics caller lost its ordinary main-loop suffix",
        );
        assert!(
            !self.main_loop_sprite_preparation_completed,
            "a terminal ground-item graphics caller cannot replay a completed common suffix",
        );
        assert!(
            self.original_timing_main_loop_interruption().is_none(),
            "a terminal ground-item graphics caller cannot also publish a main-loop interruption",
        );
        if carried_handler {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate,
                Some(NmiUpdateGate::LatchHeld),
                "a carried ground-item handler lost its Held acceptance disposition",
            );
        } else {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate, None,
                "a terminal ground-item graphics caller retained a gate without a carried handler",
            );
        }
        let mut expected_gates = vec![NmiUpdateGate::LatchHeld];
        if trailing_open {
            expected_gates.push(NmiUpdateGate::Open);
        }
        assert_eq!(
            self.original_timing_expected_nmi_update_gates, expected_gates,
            "a terminal ground-item graphics caller disagrees with its installed NMI gate authority",
        );
        assert!(
            self.game_state.display.nmi_update_is_latched(),
            "a terminal ground-item graphics caller's held handler disagrees with the native latch",
        );
        assert!(
            !self.original_timing_scheduled_nmi_accepted_at_host_return,
            "a terminal ground-item graphics caller cannot overlap an older staged acceptance",
        );
        assert_eq!(
            self.original_timing_sprite_main_return_claims_remaining, None,
            "a terminal ground-item graphics caller cannot overlap an older Sprite_Main claim scope",
        );
        assert_eq!(
            self.enemy_drop_item_graphics_deferred_sound_effect_2, None,
            "a ground-item caller cannot inherit the atomic enemy-drop sound owner",
        );

        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .expect("a terminal ground-item graphics caller lost its semantic authority")
            .semantic()
            .to_vec();
        let mut expected_semantic = Vec::new();
        if !carried_handler {
            expected_semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::LatchHeld,
            ));
        }
        expected_semantic.extend([
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ]);
        if trailing_open {
            expected_semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open,
            ));
        }
        assert_eq!(
            semantic, expected_semantic,
            "a terminal ground-item graphics caller published an unsupported or reordered semantic vector",
        );

        let mut scheduler_before_completion = self.game_execution_scheduler;
        scheduler_before_completion.begin_host_frame();
        assert_eq!(
            scheduler_before_completion.current_work(),
            Some(work),
            "host normalization changed the terminal ground-item caller",
        );
        let mut scheduler_after_completion = scheduler_before_completion;
        assert_eq!(
            scheduler_after_completion
                .advance_work_one_nmi_slice_with_authoritative_completion(true),
            Some(GameWorkStep::Complete(work)),
            "the source-proven ground-item return did not complete its scheduled caller",
        );
        assert_eq!(
            scheduler_after_completion.current_work(),
            None,
            "the terminal ground-item caller left successor scheduled work before its CPU closure",
        );

        Some(OriginalTimingTerminalGroundItemReceiptPlan {
            semantic,
            timeline,
            work,
            continuation,
            scheduler_before_completion,
            scheduler_after_completion,
        })
    }

    pub(super) fn take_original_timing_main_loop_return_timeline(
        &mut self,
    ) -> Option<OriginalTimingMainLoopReturnTimeline> {
        let timeline = self.original_timing_main_loop_return_timeline()?;
        let before_return = classify_original_timing_nmi_phases_with_ownership(
            self.original_timing_nmi_publication_pending,
            &timeline.nmi_phases_before_return,
        );
        assert_original_timing_carry_in_handler_has_receptive_display(
            before_return,
            self.display_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
        );
        let receipts = self
            .original_timing_semantic_receipts
            .as_mut()
            .expect("the inspected live host receipt disappeared before consumption");
        receipts.semantic.retain(|receipt| {
            !matches!(
                receipt,
                OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait
                    | OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                    | OriginalTimingSemanticReceipt::MainLoopProgress(_)
                    | OriginalTimingSemanticReceipt::NmiAccepted(_)
                    | OriginalTimingSemanticReceipt::NmiHandlerCompleted
            )
        });
        Some(timeline)
    }

    /// Prove one idle translated caller's terminal continued-call lifecycle
    /// without consuming any of its hardware, control, or dialogue receipts.
    pub(super) fn original_timing_uninterrupted_idle_continued_return_plan(
        &self,
    ) -> Option<OriginalTimingUninterruptedIdleContinuedReturnPlan> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || !self.game_execution_scheduler.is_idle()
            || self.original_timing_main_loop_interruption().is_some()
            || self.original_timing_main_loop_progress()
                != Some(crate::MainLoopProgress::CallStackContinued)
        {
            if crate::debug_env::var_os("ZELDA3_DEBUG_HOST_PATH").is_some() {
                eprintln!(
                    "[HOSTPATH] host={} idle-continued plan: preconditions failed idle={} interruption={:?} progress={:?} scheduler={:?}",
                    self.frame_ctr_dbg,
                    self.game_execution_scheduler.is_idle(),
                    self.original_timing_main_loop_interruption(),
                    self.original_timing_main_loop_progress(),
                    self.game_execution_scheduler,
                );
            }
            return None;
        }
        let Some(timeline) = self.original_timing_main_loop_return_timeline() else {
            if crate::debug_env::var_os("ZELDA3_DEBUG_HOST_PATH").is_some() {
                eprintln!(
                    "[HOSTPATH] host={} idle-continued plan: no return timeline",
                    self.frame_ctr_dbg,
                );
            }
            return None;
        };
        let has_dialogue_endpoint = self.original_timing_dialogue_execution_progress().is_some();
        if has_dialogue_endpoint {
            assert_eq!(
                self.pending_main_loop_common_suffix,
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                "a terminal dialogue caller lost its ordinary main-loop suffix",
            );
        } else if self.dialogue_fast_forward_hold_active {
            // A caller whose suffix was interrupted mid extended OAM packing
            // legitimately returns through the resumed-packing continuation
            // instead of the whole ordinary suffix.
            assert!(
                matches!(
                    self.pending_main_loop_common_suffix,
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
                        | Some(
                            MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch { .. }
                        )
                ),
                "a terminal dialogue caller lost its ordinary main-loop suffix: {:?}",
                self.pending_main_loop_common_suffix,
            );
        } else if self.pending_main_loop_common_suffix.is_none() {
            if crate::debug_env::var_os("ZELDA3_DEBUG_HOST_PATH").is_some() {
                eprintln!(
                    "[HOSTPATH] host={} idle-continued plan: no pending suffix",
                    self.frame_ctr_dbg,
                );
            }
            return None;
        }
        assert!(
            !self.main_loop_sprite_preparation_completed,
            "an idle terminal continued caller cannot replay a completed main-loop suffix",
        );

        let before_return = try_classify_original_timing_nmi_phases(
            self.original_timing_nmi_publication_pending,
            &timeline.nmi_phases_before_return,
        )
        .expect("an idle terminal continued caller published an invalid leading NMI lifecycle");
        assert!(
            !before_return.publication_pending_at_exit,
            "an idle terminal continued caller cannot reach its common suffix with an unfinished NMI handler",
        );
        assert_original_timing_carry_in_handler_has_receptive_display(
            before_return,
            self.display_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
        );
        let after_return =
            try_classify_original_timing_nmi_phases(false, &timeline.nmi_phases_after_return)
                .expect(
                    "an idle terminal continued caller published an invalid trailing NMI lifecycle",
                );

        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .expect("an idle terminal continued caller lost its semantic authority")
            .semantic()
            .to_vec();
        let completed_suffix_receipts = semantic
            .iter()
            .filter(|receipt| original_timing_receipt_completes_main_loop_common_suffix(receipt))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(
            completed_suffix_receipts.len(),
            1,
            "one idle terminal continued caller must publish exactly one common-suffix receipt",
        );
        let dialogue_endpoints = semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::DialogueExecutionProgress(progress) => {
                    Some(*progress)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            dialogue_endpoints.len() <= 1,
            "one idle terminal continued caller cannot publish multiple dialogue endpoints",
        );
        let dialogue_progress = dialogue_endpoints.first().copied();
        let dialogue_message_endpoint =
            dialogue_progress.map(crate::DialogueExecutionProgress::message_read_position);
        let save_menu_initialization = semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::SaveMenuInitializationProgress(progress) => {
                    Some(*progress)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            save_menu_initialization.len() <= 1,
            "one idle terminal continued caller cannot publish multiple save-menu outcomes",
        );
        let cpu_action = if matches!(
            self.dialogue_scroll_phase(),
            DialogueScrollPhase::CopyingRemainingPixels { .. }
        ) {
            assert_eq!(
                dialogue_message_endpoint, None,
                "a scroll-copy terminal return cannot also claim a VWF decoder endpoint",
            );
            OriginalTimingIdleContinuedReturnCpuAction::DialogueScrollCompletion
        } else if let Some(progress) = save_menu_initialization.first().copied() {
            assert_eq!(
                progress,
                SaveMenuInitializationProgress::Completed,
                "a terminal continued save-menu caller cannot return with initialization still in progress",
            );
            assert_eq!(
                (
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule,
                    self.game_state.frame.subsubmodule,
                    self.game_state.messaging.runtime.module(),
                ),
                (14, 11, 0, 0),
                "a save-menu initialization terminal lost its exact Module0E_0B owner",
            );
            assert_eq!(
                dialogue_message_endpoint, None,
                "a save-menu initialization terminal cannot also claim a VWF endpoint",
            );
            OriginalTimingIdleContinuedReturnCpuAction::SaveMenuInitializationCompletion
        } else if let Some(message_read_position) = dialogue_message_endpoint {
            let current_glyph_started = dialogue_progress
                .expect("a dialogue endpoint lost its execution progress")
                .current_glyph_started();
            assert_eq!(
                completed_suffix_receipts,
                [OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted],
                "a terminal dialogue endpoint requires its typed common-suffix completion",
            );
            assert!(
                self.frame_hosts_resident_render_text(),
                "a resumed terminal dialogue endpoint escaped Module0E/Module1B RenderText: {:?}",
                self.game_state.frame,
            );
            assert!(
                !self.dialogue_fast_forward_hold_pending,
                "a terminal dialogue endpoint cannot retain another suspended VWF hold",
            );
            match (
                self.game_state.display.pending_nmi_subroutine,
                self.game_state.display.core_update_disable_flag,
            ) {
                (0, 0) => {}
                (2, 2) => {
                    // A preceding in-frame character/scroll completion queued
                    // its BG3 text upload; this host's same-host Held handler
                    // consumes it before the endpoint work runs (route host
                    // 20773).
                }
                (2, 0) => {
                    // The save-menu RenderText queues the same BG3 upload
                    // without the core-disable hold (route host 47630).
                }
                other => panic!(
                    "a terminal dialogue endpoint found an unsupported NMI subroutine owner: {other:?}"
                ),
            }
            assert!(
                !self.original_timing_nmi_publication_pending,
                "the observed terminal dialogue endpoint does not complete a carried handler",
            );
            assert_eq!(
                timeline.nmi_phases_before_return,
                [
                    OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingNmiPhase::HandlerCompleted,
                ],
                "a terminal dialogue endpoint requires its exact same-host Held handler",
            );
            let carries_open_after_suffix = match timeline.nmi_phases_after_return.as_slice() {
                [] => false,
                [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)] => true,
                phases => panic!(
                    "a terminal dialogue endpoint published an unsupported post-suffix NMI lifecycle: {phases:?}"
                ),
            };
            assert!(
                !after_return.handler_completion.completed(),
                "a terminal dialogue endpoint cannot complete another post-suffix NMI handler",
            );
            assert_eq!(
                after_return.publication_pending_at_exit, carries_open_after_suffix,
                "a terminal dialogue endpoint disagreed with its trailing Open-NMI authority",
            );
            OriginalTimingIdleContinuedReturnCpuAction::DialogueEndpoint {
                message_read_position,
                current_glyph_started,
                transition: self.original_timing_suspended_vwf_endpoint_transition_plan(
                    message_read_position,
                    current_glyph_started,
                ),
            }
        } else if self.dialogue_fast_forward_hold_active
            && matches!(
                self.pending_main_loop_common_suffix,
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
            )
        {
            // A caller whose pending suffix is the resumed extended-OAM
            // packing continuation is an interrupted-suffix return, not a
            // VWF line-completion terminal: the source is still rendering at
            // its own endpoint pace, so the ordinary continued-return action
            // below owns it and the fast-forward hold stays armed.
            assert_eq!(
                completed_suffix_receipts,
                [OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted],
                "a suspended VWF terminal return requires its typed common-suffix completion",
            );
            assert!(
                self.frame_hosts_resident_render_text()
                    && self.game_state.messaging.runtime.module() == 1
                    && self.game_state.messaging.runtime.text_render_state() == 3,
                "a suspended VWF terminal return lost its resident Module0E caller: {:?}",
                self.game_state.frame,
            );
            assert!(
                !self.dialogue_fast_forward_hold_pending,
                "a suspended VWF terminal return overlapped another pending hold",
            );
            assert_eq!(
                self.dialogue_live_message_read_position_target, None,
                "a suspended VWF terminal return retained an unconsumed endpoint target",
            );
            assert!(
                matches!(
                    (
                        self.game_state.display.pending_nmi_subroutine,
                        self.game_state.display.core_update_disable_flag,
                    ),
                    // A character DMA queued by the final rendered glyph may
                    // still be pending when the caller's terminal return
                    // crosses its NMI; that handler consumes the subroutine
                    // (route host 92200), and when every recent NMI was held
                    // its paired disable flag survives too (route host
                    // 105948). A (2,2) epilogue with the decoder still at
                    // position zero is NOT this shape: it is the message
                    // initialization's own epilogue, whose caller-return host
                    // a different owner completes (route host 103733).
                    (0, 0) | (2, 0)
                ) || (self.game_state.display.pending_nmi_subroutine == 2
                    && self.game_state.display.core_update_disable_flag == 2
                    && self.game_state.messaging.runtime.dialogue_msg_read_pos() != 0),
                "a suspended VWF terminal return cannot overwrite an existing character epilogue: host={} frame={:?} subroutine={} disable={}",
                self.frame_ctr_dbg,
                self.game_state.frame,
                self.game_state.display.pending_nmi_subroutine,
                self.game_state.display.core_update_disable_flag,
            );
            // The suffix-completing Held NMI is either carried in from the
            // preceding host's return boundary or accepted and completed
            // inside this same host. Both are exact wire shapes of the same
            // C boundary.
            if self.original_timing_nmi_publication_pending {
                assert_eq!(
                    self.original_timing_pending_nmi_update_gate,
                    Some(NmiUpdateGate::LatchHeld),
                    "a D-less VWF terminal return lost its carried Held gate",
                );
                assert_eq!(
                    timeline.nmi_phases_before_return,
                    [OriginalTimingNmiPhase::HandlerCompleted],
                    "a D-less VWF terminal return requires its exact carried handler completion",
                );
            } else {
                assert_eq!(
                    timeline.nmi_phases_before_return,
                    [
                        OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                        OriginalTimingNmiPhase::HandlerCompleted,
                    ],
                    "a D-less VWF terminal return requires its exact same-host Held handler",
                );
            }
            let carries_open_after_suffix = match timeline.nmi_phases_after_return.as_slice() {
                [] => false,
                [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)] => true,
                phases => panic!(
                    "a D-less VWF terminal return published an unsupported post-suffix NMI lifecycle: {phases:?}"
                ),
            };
            assert!(
                !after_return.handler_completion.completed(),
                "a D-less VWF terminal return cannot complete another post-suffix NMI handler",
            );
            assert_eq!(
                after_return.publication_pending_at_exit, carries_open_after_suffix,
                "a D-less VWF terminal return disagreed with its trailing Open-NMI authority",
            );
            OriginalTimingIdleContinuedReturnCpuAction::SuspendedVwfCompletion {
                transition: self.original_timing_suspended_vwf_completion_transition_plan(),
            }
        } else {
            OriginalTimingIdleContinuedReturnCpuAction::None
        };

        let joypads = semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::JoypadPublication(joypad) => Some(*joypad),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut joypad_index = 0usize;
        let mut expected_gates = Vec::new();
        let mut expected_semantic = Vec::new();
        let mut semantic_gate = if self.original_timing_nmi_publication_pending {
            let gate = self
                .original_timing_pending_nmi_update_gate
                .expect("an idle terminal continued caller lost its carried NMI disposition");
            expected_gates.push(gate);
            Some(gate)
        } else {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate, None,
                "an idle terminal continued caller retained an NMI gate without a carried handler",
            );
            None
        };
        let mut append_phases =
            |phases: &[OriginalTimingNmiPhase],
             expected: &mut Vec<OriginalTimingSemanticReceipt>| {
                for phase in phases {
                    match phase {
                        OriginalTimingNmiPhase::Accepted(gate) => {
                            expected_gates.push(*gate);
                            semantic_gate = Some(*gate);
                            expected.push(OriginalTimingSemanticReceipt::NmiAccepted(*gate));
                        }
                        OriginalTimingNmiPhase::HandlerCompleted => {
                            let gate = semantic_gate.take().expect(
                                "an idle terminal continued caller completed an unaccepted NMI",
                            );
                            expected.push(OriginalTimingSemanticReceipt::NmiHandlerCompleted);
                            if gate == NmiUpdateGate::Open {
                                let joypad = joypads.get(joypad_index).copied().expect(
                                    "an Open idle terminal handler omitted its Joypad publication",
                                );
                                joypad_index += 1;
                                expected
                                    .push(OriginalTimingSemanticReceipt::JoypadPublication(joypad));
                            }
                        }
                    }
                }
            };
        append_phases(&timeline.nmi_phases_before_return, &mut expected_semantic);
        expected_semantic.extend([OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        )]);
        expected_semantic.extend(completed_suffix_receipts);
        append_phases(&timeline.nmi_phases_after_return, &mut expected_semantic);
        if let Some(progress) = dialogue_progress {
            expected_semantic.push(OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                progress,
            ));
        }
        assert_eq!(
            joypad_index,
            joypads.len(),
            "an idle terminal continued caller published an unowned Joypad receipt",
        );
        assert_eq!(
            self.original_timing_expected_nmi_update_gates, expected_gates,
            "an idle terminal continued caller disagrees with its exact NMI gate authority",
        );
        if before_return.handler_completion.completed() {
            let native_gate = if self.game_state.display.nmi_update_is_latched() {
                NmiUpdateGate::LatchHeld
            } else {
                NmiUpdateGate::Open
            };
            assert_eq!(
                expected_gates.first().copied(),
                Some(native_gate),
                "an idle terminal continued caller's leading handler disagrees with the native NMI latch",
            );
        }
        // A save-menu initialization claim, deferred spotlight caller-return
        // token, or dialogue-close fact is consumed by its own machinery;
        // this plan only pins it at its exact wire position (route hosts
        // 47624, 53926, and the caller-return dialogue close at 123210).
        for (index, receipt) in semantic.iter().enumerate() {
            if matches!(
                receipt,
                OriginalTimingSemanticReceipt::SaveMenuInitializationProgress(_)
                    | OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait
                    | OriginalTimingSemanticReceipt::DialogueClosed
            ) {
                expected_semantic.insert(index.min(expected_semantic.len()), *receipt);
            }
            // The intro module's long triforce-thread initializer returns
            // through a Sprite_Main-shaped receipt without a native Sprite_Main
            // body (route hosts 749955, 965252); the return executor drains it.
            if *receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                && self.game_state.frame.main_module == 0
            {
                expected_semantic.insert(index.min(expected_semantic.len()), *receipt);
            }
        }
        assert_eq!(
            semantic, expected_semantic,
            "an idle terminal continued caller published an unsupported or reordered semantic vector: host={} frame={:?} scheduler={:?}",
            self.frame_ctr_dbg, self.game_state.frame, self.game_execution_scheduler,
        );

        Some(OriginalTimingUninterruptedIdleContinuedReturnPlan {
            semantic,
            timeline,
            cpu_action,
        })
    }

    /// A live pre-main caller is already executing on the source CPU stack;
    /// its terminal ordering must therefore come from the source receipt, not
    /// from the translated scheduler's legacy one-NMI approximation.
    pub(super) fn take_original_timing_pre_main_caller_return_timeline(
        &mut self,
        caller: PreMainCallerContinuation,
    ) -> Option<OriginalTimingMainLoopReturnTimeline> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        assert_eq!(
            self.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "live {caller:?} caller requires its exact NMI_PrepareSprites/$12-clear suffix",
        );
        let expected = self
            .original_timing_main_loop_return_timeline()
            .unwrap_or_else(|| panic!("live {caller:?} caller omitted its typed return timeline"));
        let consumed = self
            .take_original_timing_main_loop_return_timeline()
            .expect("the inspected pre-main caller return timeline disappeared");
        assert_eq!(
            consumed, expected,
            "the {caller:?} return timeline changed before consumption",
        );
        Some(consumed)
    }

    /// Return the one source-call progress fact owned by an active scheduled
    /// caller.
    ///
    /// A host which returns inside Link/OAM or sprite preparation has already
    /// moved `MainLoopProgress` into its ordered interruption timeline. An
    /// uninterrupted host leaves the same fact either in the corresponding
    /// timeline or on the receipt bus. Route all three representations through
    /// one owner so a semantic caller never attempts to consume the raw receipt
    /// after the timeline parser has taken it.
    pub(super) fn take_original_timing_scheduled_caller_progress(
        &mut self,
        interrupted: Option<&OriginalTimingMainLoopInterruptionTimeline>,
        uninterrupted: Option<&OriginalTimingMainLoopTimeline>,
    ) -> Option<crate::MainLoopProgress> {
        interrupted
            .map(|timeline| timeline.progress)
            .or_else(|| uninterrupted.map(|timeline| timeline.progress))
            .or_else(|| self.take_original_timing_main_loop_progress())
    }

    pub(super) fn take_original_timing_nmi_phases(&mut self) -> Vec<OriginalTimingNmiPhase> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return Vec::new();
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return Vec::new();
        };
        let mut phases = Vec::new();
        let mut consumed = Vec::new();
        for (index, receipt) in receipts.semantic.iter().enumerate() {
            let phase = match receipt {
                OriginalTimingSemanticReceipt::NmiAccepted(gate) => {
                    OriginalTimingNmiPhase::Accepted(*gate)
                }
                OriginalTimingSemanticReceipt::NmiHandlerCompleted => {
                    OriginalTimingNmiPhase::HandlerCompleted
                }
                _ => continue,
            };
            phases.push(phase);
            consumed.push(index);
        }
        for index in consumed.into_iter().rev() {
            receipts.semantic.remove(index);
        }
        phases
    }

    /// Apply the next Zelda-visible controller publication owned by a
    /// completed authoritative NMI.
    ///
    /// This is intentionally separate from the host call's raw input. The
    /// original auto-joy registers update later in VBlank, so a handler may
    /// publish the preceding sample even after libretro has supplied new
    /// buttons for the current call.
    pub(super) fn apply_original_timing_joypad_publication(&mut self) {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return;
        };
        let Some(index) = receipts.semantic.iter().position(|receipt| {
            matches!(receipt, OriginalTimingSemanticReceipt::JoypadPublication(_))
        }) else {
            // Older immutable receipt caches predate this semantic domain.
            return;
        };
        let OriginalTimingSemanticReceipt::JoypadPublication(publication) =
            receipts.semantic.remove(index)
        else {
            unreachable!("joypad publication index changed receipt kind")
        };

        let mut link = self.follower_link_state_mut();
        link.set_joypad1h_last(publication.high);
        link.set_joypad1l_last(publication.low);
        link.set_filtered_joypad_h(publication.high_filtered);
        link.set_filtered_joypad_l(publication.low_filtered);
        // NMI_ReadJoypads publishes each current sample into both the Zelda
        // input byte and its following-edge baseline before returning.
        link.set_joypad1h_last2(publication.high);
        link.set_joypad1l_last2(publication.low);
    }

    pub(super) fn carry_original_timing_nmi_publication_at_host_return(&mut self) {
        assert!(
            !self.original_timing_nmi_publication_pending,
            "one source host cannot queue a second unfinished NMI publication",
        );
        self.original_timing_nmi_publication_pending = true;
    }

    pub(super) fn capture_and_carry_original_timing_nmi_publication_at_host_return(&mut self) {
        self.capture_display_snapshot();
        self.carry_original_timing_nmi_publication_at_host_return();
    }

    pub(super) fn stage_original_timing_scheduled_caller_acceptance_at_host_return(
        &mut self,
        accepts_nmi_at_return: Option<bool>,
    ) {
        if accepts_nmi_at_return == Some(true) {
            assert!(
                !self.original_timing_scheduled_nmi_accepted_at_host_return,
                "one source host cannot stage two following NMI acceptances",
            );
            self.original_timing_scheduled_nmi_accepted_at_host_return = true;
        }
    }

    pub(super) fn finish_original_timing_scheduled_caller_host_return(&mut self) {
        if std::mem::take(&mut self.original_timing_scheduled_nmi_accepted_at_host_return) {
            // The scheduled caller has now reached the source host return
            // which accepted this NMI. Close that host's live generation
            // before carrying the handler; a preceding carry-in or same-host
            // completion may have left an older receptive snapshot behind.
            self.capture_and_carry_original_timing_nmi_publication_at_host_return();
        }
    }

    pub(super) fn carry_original_timing_scheduled_caller_host_return_from_active_capture(
        &mut self,
    ) {
        if std::mem::take(&mut self.original_timing_scheduled_nmi_accepted_at_host_return) {
            // Some source-owned scheduled completions must compose their
            // caller-return scanout before leaving the completion arm. That
            // capture is also the trailing NMI's acceptance generation; carry
            // it directly instead of replacing it with a second capture at
            // the outer host return. A deliberately retained (non-receptive)
            // caller-return image has already been consumed for this host's
            // presentation, so the carried acceptance takes a fresh receptive
            // capture instead (route host 39759).
            if self
                .display_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts)
            {
                self.carry_original_timing_nmi_publication_at_host_return();
            } else {
                self.capture_and_carry_original_timing_nmi_publication_at_host_return();
            }
        }
    }

    pub(super) fn take_original_timing_main_loop_interruption(
        &mut self,
        phase: crate::MainLoopInterruption,
    ) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        if receipts.discard_forwarded_main_loop_interruption(phase) {
            return true;
        }
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(observed)
                        if *observed == phase
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot interrupt the same main-loop phase twice",
        );
        matches.first().is_some_and(|&index| {
            receipts.semantic.remove(index);
            true
        })
    }

    pub(super) fn take_original_timing_save_menu_initialization_progress(
        &mut self,
    ) -> Option<SaveMenuInitializationProgress> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::SaveMenuInitializationProgress(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple save-menu initialization outcomes",
        );
        matches.first().map(|&(index, progress)| {
            receipts.semantic.remove(index);
            progress
        })
    }

    pub(super) fn take_original_timing_pre_overworld_stage_completion(
        &mut self,
        stage: crate::PreOverworldStageCompletion,
    ) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::PreOverworldStageCompleted(observed)
                        if *observed == stage
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot complete the same pre-overworld stage twice",
        );
        matches.first().is_some_and(|&index| {
            receipts.semantic.remove(index);
            true
        })
    }

    pub(super) fn take_original_timing_overworld_sprite_reload_progress(
        &mut self,
    ) -> Vec<crate::OverworldSpriteReloadProgress> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return Vec::new();
        }
        let current_work = self.game_execution_scheduler.current_work();
        let deferred_overworld_load_overlays_sprite_reload_active =
            self.pending_overworld_sprite_reload_slots.is_some()
                && matches!(
                    current_work,
                    Some(
                        GameWorkContinuation::FinishOverworldLoadOverlaysSpriteReload
                            | GameWorkContinuation::FinishOverworldSpriteReloadTail { .. }
                            | GameWorkContinuation::FinishModule09LongLoad {
                                step: Module09LongLoadStep::MirrorWarpSpriteLoadReload
                                    | Module09LongLoadStep::Module15MirrorWarpSpriteLoadReload
                                    | Module09LongLoadStep::LoadOverlays2,
                            }
                    )
                );
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return Vec::new();
        };
        let mut progress = Vec::new();
        // One host can publish the presence map and activate a sprite while
        // the native shadow is still in its sprite-reset estimate; applying
        // the publication first advances the shadow into the properties
        // stage that owns the activation (route host 208356).
        let presence_published_in_host = receipts.semantic.iter().any(|receipt| {
            matches!(
                receipt,
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    crate::OverworldSpriteReloadProgress::PresencePublished
                )
            )
        });
        receipts.semantic.retain(|receipt| {
            let OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(observed) = receipt
            else {
                return true;
            };
            let native_caller_is_suspended = match observed {
                crate::OverworldSpriteReloadProgress::PresencePublished => {
                    matches!(
                        current_work,
                        Some(
                            GameWorkContinuation::PreOverworldPropertiesSpriteReset { .. }
                                | GameWorkContinuation::FinishPreOverworldProperties { .. }
                                | GameWorkContinuation::FinishFluteMenuSelectedScreen {
                                    step: FluteMenuSelectedScreenStep::OverworldReloadReset,
                                }
                        )
                    ) || deferred_overworld_load_overlays_sprite_reload_active
                }
                crate::OverworldSpriteReloadProgress::SpriteActivated { .. } => {
                    matches!(
                        current_work,
                        Some(
                            GameWorkContinuation::FinishPreOverworldProperties { .. }
                                | GameWorkContinuation::FinishFluteMenuSelectedScreen {
                                    step: FluteMenuSelectedScreenStep::OverworldReloadReset
                                        | FluteMenuSelectedScreenStep::OverworldReloadScan,
                                }
                        )
                    ) || (presence_published_in_host
                        && matches!(
                            current_work,
                            Some(GameWorkContinuation::PreOverworldPropertiesSpriteReset { .. })
                        ))
                        || deferred_overworld_load_overlays_sprite_reload_active
                }
                crate::OverworldSpriteReloadProgress::ProximityScanSuspended { .. } => {
                    deferred_overworld_load_overlays_sprite_reload_active
                        || matches!(
                            current_work,
                            Some(
                                GameWorkContinuation::FinishPreOverworldProperties { .. }
                                    | GameWorkContinuation::FinishFluteMenuSelectedScreen {
                                        step: FluteMenuSelectedScreenStep::OverworldReloadScan,
                                    }
                            )
                        )
                        || (presence_published_in_host
                            && matches!(
                                current_work,
                                Some(GameWorkContinuation::FinishFluteMenuSelectedScreen {
                                    step: FluteMenuSelectedScreenStep::OverworldReloadReset,
                                })
                            ))
                }
                crate::OverworldSpriteReloadProgress::GenerationReturned
                | crate::OverworldSpriteReloadProgress::GenerationReturnedAtPortalReset { .. }
                | crate::OverworldSpriteReloadProgress::GenerationReturnedAtPortalLoadProperties { .. }
                | crate::OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveCleanup {
                    ..
                } | crate::OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveTypeClear {
                    ..
                } => {
                    deferred_overworld_load_overlays_sprite_reload_active
                        || matches!(
                            current_work,
                            Some(GameWorkContinuation::FinishFluteMenuSelectedScreen {
                                step: FluteMenuSelectedScreenStep::OverworldReloadScan,
                            })
                        )
                }
                crate::OverworldSpriteReloadProgress::ReloadReturned => matches!(
                    current_work,
                    Some(GameWorkContinuation::FinishOverworldSpriteReloadTail { .. })
                ),
            };
            if native_caller_is_suspended {
                progress.push(*observed);
                false
            } else {
                true
            }
        });
        progress
    }

    pub(super) fn take_original_timing_overworld_map_quadrants_published(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let mut published = false;
        receipts.semantic.retain(|receipt| {
            if matches!(
                receipt,
                OriginalTimingSemanticReceipt::OverworldMapQuadrantsPublished
            ) {
                assert!(!published, "overworld map-quadrant publication replayed");
                published = true;
                false
            } else {
                true
            }
        });
        published
    }

    pub(super) fn original_timing_dungeon_push_blocks_pending(&self) -> bool {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts
                        .semantic
                        .contains(&OriginalTimingSemanticReceipt::DungeonPushBlocksPending)
                })
    }

    /// The wire interrupted Dungeon_PushBlock_Handler's loop at this host's
    /// boundary; the misc objects before the returned word offset have run.
    pub(super) fn original_timing_dungeon_push_blocks_in_progress(&self) -> Option<u16> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        self.original_timing_semantic_receipts
            .as_ref()?
            .semantic
            .iter()
            .find_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::DungeonPushBlocksInProgress { next_index } => {
                    Some(*next_index)
                }
                _ => None,
            })
    }

    pub(super) fn take_original_timing_dungeon_push_blocks_in_progress(&mut self) -> Option<u16> {
        let next_index = self.original_timing_dungeon_push_blocks_in_progress()?;
        let receipts = self.original_timing_semantic_receipts.as_mut().unwrap();
        let before = receipts.semantic.len();
        receipts.semantic.retain(|receipt| {
            !matches!(
                receipt,
                OriginalTimingSemanticReceipt::DungeonPushBlocksInProgress { .. }
            )
        });
        assert_eq!(
            before - receipts.semantic.len(),
            1,
            "one host may interrupt Dungeon_PushBlock_Handler once"
        );
        Some(next_index)
    }

    /// Consume a `DungeonPushBlocksInProgress` receipt that merely restates
    /// the resumed loop's own cursor. A later, genuine re-interruption of the
    /// same host names a greater index and stays in the vector.
    pub(super) fn take_original_timing_dungeon_push_blocks_restatement(&mut self, cursor: u16) {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return;
        };
        if let Some(index) = receipts.semantic.iter().position(|receipt| {
            *receipt
                == OriginalTimingSemanticReceipt::DungeonPushBlocksInProgress { next_index: cursor }
        }) {
            receipts.semantic.remove(index);
        }
    }

    /// Module 7's push-block handler returned before this host's boundary
    /// and OrientLampLightCone stored nothing yet.
    pub(super) fn original_timing_dungeon_push_blocks_handled(&self) -> bool {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts
                        .semantic
                        .contains(&OriginalTimingSemanticReceipt::DungeonPushBlocksHandled)
                })
    }

    pub(super) fn take_original_timing_dungeon_push_blocks_handled(&mut self) -> bool {
        if !self.original_timing_dungeon_push_blocks_handled() {
            return false;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut().unwrap();
        let before = receipts.semantic.len();
        receipts
            .semantic
            .retain(|receipt| *receipt != OriginalTimingSemanticReceipt::DungeonPushBlocksHandled);
        assert_eq!(
            before - receipts.semantic.len(),
            1,
            "one host may return from Dungeon_PushBlock_Handler once"
        );
        true
    }

    pub(super) fn take_original_timing_dungeon_push_blocks_pending(&mut self) -> bool {
        if !self.original_timing_dungeon_push_blocks_pending() {
            return false;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut().unwrap();
        let before = receipts.semantic.len();
        receipts
            .semantic
            .retain(|receipt| *receipt != OriginalTimingSemanticReceipt::DungeonPushBlocksPending);
        assert_eq!(
            before - receipts.semantic.len(),
            1,
            "push-block checkpoint replayed"
        );
        true
    }

    pub(super) fn take_original_timing_overworld_special_exit_mosaic_restored(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let mut restored = false;
        receipts.semantic.retain(|receipt| {
            if matches!(
                receipt,
                OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicRestored
            ) {
                assert!(!restored, "special-exit mosaic restore receipt replayed");
                restored = true;
                false
            } else {
                true
            }
        });
        restored
    }

    pub(super) fn take_original_timing_overworld_special_exit_mosaic_returned(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let mut returned = false;
        receipts.semantic.retain(|receipt| {
            if matches!(
                receipt,
                OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicReturned
            ) {
                assert!(!returned, "special-exit mosaic return receipt replayed");
                returned = true;
                false
            } else {
                true
            }
        });
        returned
    }

    pub(super) fn take_original_timing_dungeon_falling_entrance_progress(
        &mut self,
    ) -> Option<crate::DungeonFallingEntranceProgress> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::DungeonFallingEntranceProgress(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one source host cannot publish two falling-entrance control stages",
        );
        matches.first().map(|&(index, progress)| {
            receipts.semantic.remove(index);
            progress
        })
    }

    pub(super) fn take_original_timing_rescued_maiden_tilemap_clear_progress(
        &mut self,
    ) -> Option<crate::RescuedMaidenTilemapClearProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::RescuedMaidenTilemapClearProgress(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one source host cannot publish two rescued-maiden clear checkpoints",
        );
        matches.first().map(|&(index, progress)| {
            receipts.semantic.remove(index);
            progress
        })
    }

    pub(super) fn take_original_timing_rescued_maiden_initialization_progress(
        &mut self,
    ) -> Option<crate::RescuedMaidenInitializationProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::RescuedMaidenInitializationProgress(progress) => {
                    Some((index, *progress))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one source host cannot publish two rescued-maiden initialization checkpoints",
        );
        matches.first().map(|&(index, progress)| {
            receipts.semantic.remove(index);
            progress
        })
    }

    pub(super) fn apply_original_timing_dungeon_falling_entrance_progress(
        &mut self,
        progress: crate::DungeonFallingEntranceProgress,
    ) {
        assert_eq!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonFallingEntrance {
                work: DungeonFallingEntranceWork::RoomAndTilesets,
            }),
            "falling-entrance source progress reached the wrong suspended caller",
        );
        assert_eq!(
            self.game_state.frame.main_module, 0x11,
            "falling-entrance source progress reached the wrong main module",
        );
        match progress {
            crate::DungeonFallingEntranceProgress::RoomParserClearedSubsubmodule => {
                assert_eq!(
                    self.game_state.frame.subsubmodule, 2,
                    "falling-entrance room-parser clear replayed or skipped its entry phase",
                );
                self.set_subsubmodule(0);
            }
            crate::DungeonFallingEntranceProgress::RoomLoadAdvancedSubsubmodule => {
                assert_eq!(
                    self.game_state.frame.subsubmodule, 0,
                    "falling-entrance phase restore replayed or preceded the room-parser clear",
                );
                self.set_subsubmodule(3);
            }
            crate::DungeonFallingEntranceProgress::SongBankTailEntered => {
                assert_eq!(
                    (
                        self.game_state.frame.submodule,
                        self.game_state.frame.subsubmodule,
                    ),
                    (0, 3),
                    "falling-entrance song-bank tail replayed or preceded the phase restore",
                );
                self.set_submodule(7);
            }
        }
    }

    pub(super) fn take_original_timing_world_map_ambient_map8_returned(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let mut returned = false;
        receipts.semantic.retain(|receipt| {
            if matches!(
                receipt,
                OriginalTimingSemanticReceipt::WorldMapAmbientMap8Returned
            ) {
                assert!(!returned, "world-map ambient Map8 return receipt replayed");
                returned = true;
                false
            } else {
                true
            }
        });
        returned
    }

    pub(super) fn take_original_timing_world_map_overlay_reload_returned(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let mut returned = false;
        receipts.semantic.retain(|receipt| {
            if matches!(
                receipt,
                OriginalTimingSemanticReceipt::WorldMapOverlayReloadReturned
            ) {
                assert!(
                    !returned,
                    "world-map overlay reload return receipt replayed"
                );
                returned = true;
                false
            } else {
                true
            }
        });
        returned
    }

    pub(super) fn apply_original_timing_overworld_sprite_reload_progress(
        &mut self,
        progress: Vec<crate::OverworldSpriteReloadProgress>,
    ) -> bool {
        let mut reload_returned = false;
        for progress in progress {
            match progress {
                crate::OverworldSpriteReloadProgress::PresencePublished => {
                    if self.advance_flute_menu_reload_to_scan_if_needed() {
                        continue;
                    }
                    if self.pending_overworld_sprite_reload_slots.is_some() {
                        assert!(
                            matches!(
                                self.game_execution_scheduler.current_work(),
                                Some(
                                    GameWorkContinuation::FinishOverworldLoadOverlaysSpriteReload
                                        | GameWorkContinuation::FinishOverworldSpriteReloadTail { .. }
                                        | GameWorkContinuation::FinishModule09LongLoad {
                                            step: Module09LongLoadStep::MirrorWarpSpriteLoadReload
                                                | Module09LongLoadStep::Module15MirrorWarpSpriteLoadReload
                                                | Module09LongLoadStep::LoadOverlays2,
                                        }
                                )
                            ),
                            "deferred overworld sprite presence arrived outside its source reload",
                        );
                        continue;
                    }
                    if let Some(GameWorkContinuation::PreOverworldPropertiesSpriteReset {
                        overworld_screen,
                        animated_tiles,
                    }) = self.game_execution_scheduler.current_work()
                    {
                        // The source receipt is emitted from Overworld_LoadSprites,
                        // after Sprite_DisableAll and Sprite_ResetAll_noDisable.
                        // If the older coarse timing shadow has not reached that
                        // statement boundary yet, project the still-pending C prefix
                        // exactly once before applying the observed publication.
                        self.complete_pre_overworld_load_properties_through_sprite_reset(
                            overworld_screen,
                            animated_tiles,
                        );
                        // The wire's publication supersedes whatever slice
                        // budget the sprite-reset estimate still held (route
                        // host 208356 published one host early).
                        self.game_execution_scheduler.finish_work();
                        self.game_execution_scheduler.schedule_work(
                            GameWorkContinuation::FinishPreOverworldProperties {
                                overworld_screen,
                                sprite_presence_published: false,
                            },
                            PRE_OVERWORLD_PROPERTIES_AFTER_SPRITE_RESET_NMI_SLICES,
                        );
                    }
                    assert!(
                        matches!(
                            self.game_execution_scheduler.current_work(),
                            Some(GameWorkContinuation::FinishPreOverworldProperties {
                                sprite_presence_published: false,
                                ..
                            })
                        ),
                        "overworld sprite-presence publication arrived outside its suspended C caller: {:?}",
                        self.game_execution_scheduler.current_work(),
                    );
                    self.overworld_load_sprites();
                    self.begin_overworld_proximity_scan_continuation();
                    assert!(
                        self.game_execution_scheduler
                            .mark_pre_overworld_sprite_presence_published(),
                        "overworld sprite-presence publication replayed",
                    );
                }
                crate::OverworldSpriteReloadProgress::SpriteActivated {
                    block,
                    slot,
                    sprite_type,
                } => {
                    self.advance_flute_menu_reload_to_scan_if_needed();
                    if self.pending_overworld_sprite_reload_slots.is_some() {
                        self.begin_overworld_proximity_scan_continuation();
                        self.set_overworld_horizontal_scroll_delta_low(0xff);
                        self.publish_deferred_module09_sprite_slot(slot);
                    } else {
                        assert!(
                            matches!(
                                self.game_execution_scheduler.current_work(),
                                Some(
                                    GameWorkContinuation::FinishPreOverworldProperties {
                                        sprite_presence_published: true,
                                        ..
                                    } | GameWorkContinuation::FinishFluteMenuSelectedScreen {
                                        step: FluteMenuSelectedScreenStep::OverworldReloadScan,
                                    }
                                )
                            ),
                            "overworld sprite activation arrived before the presence map was published",
                        );
                        self.overworld_load_proxima_sprite_if_alive(block);
                    }
                    let sprite = self.sprite_slot_view(usize::from(slot));
                    assert_eq!(
                        sprite.n_word(),
                        block,
                        "native overworld activation chose a different sprite slot",
                    );
                    assert_eq!(
                        sprite.sprite_type(),
                        sprite_type,
                        "native overworld activation disagreed on sprite type",
                    );
                    assert_eq!(
                        sprite.state(),
                        8,
                        "native overworld activation did not publish active state",
                    );
                }
                crate::OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h } => {
                    assert!(
                        self.pending_overworld_sprite_reload_slots.is_some()
                            || matches!(
                                self.game_execution_scheduler.current_work(),
                                Some(
                                    GameWorkContinuation::FinishPreOverworldProperties { .. }
                                        | GameWorkContinuation::FinishFluteMenuSelectedScreen {
                                            step: FluteMenuSelectedScreenStep::OverworldReloadScan,
                                        }
                                )
                            ),
                        "overworld proximity scan progress arrived without its suspended reload generation",
                    );
                    if self.pending_overworld_sprite_reload_slots.is_some()
                        || matches!(
                            self.game_execution_scheduler.current_work(),
                            Some(GameWorkContinuation::FinishPreOverworldProperties { .. })
                        )
                    {
                        self.begin_overworld_proximity_scan_continuation();
                        self.set_overworld_horizontal_scroll_delta_low(0xff);
                    }
                    self.set_bg2_x(bg2_h);
                }
                progress @ (crate::OverworldSpriteReloadProgress::GenerationReturned
                | crate::OverworldSpriteReloadProgress::GenerationReturnedAtPortalReset { .. }
                | crate::OverworldSpriteReloadProgress::GenerationReturnedAtPortalLoadProperties { .. }
                | crate::OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveCleanup { .. }
                | crate::OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveTypeClear { .. }) => {
                    if let Some(GameWorkContinuation::FinishFluteMenuSelectedScreen {
                        step: FluteMenuSelectedScreenStep::OverworldReloadScan,
                    }) = self.game_execution_scheduler.current_work()
                    {
                        self.complete_overworld_proximity_scan_continuation();
                        self.complete_bird_travel_position_after_sprite_reload();
                        self.game_execution_scheduler.refine_scheduled_work(
                            GameWorkContinuation::FinishFluteMenuSelectedScreen {
                                step: FluteMenuSelectedScreenStep::OverworldReloadScan,
                            },
                            GameWorkContinuation::FinishFluteMenuSelectedScreen {
                                step: FluteMenuSelectedScreenStep::SelectedScreenSuffix,
                            },
                        );
                        continue;
                    }
                    if let Some(GameWorkContinuation::FinishModule09LongLoad { step }) =
                        self.game_execution_scheduler.current_work()
                    {
                        if step == Module09LongLoadStep::LoadOverlays2 {
                            if self.overworld_proximity_scan_saved_scroll.is_some() {
                                self.complete_overworld_proximity_scan_continuation();
                            }
                            assert!(
                                !reload_returned,
                                "whirlpool bird-travel sprite generation return replayed"
                            );
                            reload_returned = true;
                            continue;
                        }
                        let tail = match step {
                            Module09LongLoadStep::MirrorWarpSpriteLoadReload => {
                                Module09LongLoadStep::MirrorWarpSpriteLoadTail
                            }
                            Module09LongLoadStep::Module15MirrorWarpSpriteLoadReload => {
                                Module09LongLoadStep::Module15MirrorWarpSpriteLoadTail
                            }
                            _ => panic!(
                                "overworld sprite generation returned to an unsupported Module09 long-load phase: {step:?}",
                            ),
                        };
                        if self.overworld_proximity_scan_saved_scroll.is_some() {
                            self.complete_overworld_proximity_scan_continuation();
                        }
                        // This receipt names the inner JSL return. Publish the
                        // completed generation now, but leave every mutation
                        // after the enclosing mirror-warp caller returns under
                        // its independently observed Sprite_Main/terminal
                        // boundary. If the source stops inside interactive
                        // cleanup, preserve that inner continuation before
                        // publishing the portal or animation tail.
                        self.publish_deferred_module09_sprite_slots_at_reload_return();
                        let tail = if let crate::OverworldSpriteReloadProgress::GenerationReturnedAtPortalReset { slot, completed_stores } = progress {
                            self.begin_mirror_warp_portal_reset(slot, completed_stores);
                            Module09LongLoadStep::MirrorWarpPortalReset { slot, completed_stores, module15: step.caller_is_module15() }
                        } else if let crate::OverworldSpriteReloadProgress::GenerationReturnedAtPortalLoadProperties { slot, completed_stores } = progress {
                            self.begin_mirror_warp_portal_load_properties(slot, completed_stores);
                            Module09LongLoadStep::MirrorWarpPortalLoadProperties { slot, completed_stores, module15: step.caller_is_module15() }
                        } else if let crate::OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveCleanup { slot } = progress {
                            self.begin_mirror_warp_interactive_cleanup(slot);
                            Module09LongLoadStep::MirrorWarpInteractiveCleanup { slot, module15: step.caller_is_module15() }
                        } else if let crate::OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveTypeClear { slot } = progress {
                            self.begin_mirror_warp_interactive_type_clear(slot);
                            Module09LongLoadStep::MirrorWarpInteractiveTypeClear { slot, module15: step.caller_is_module15() }
                        } else {
                            self.complete_mirror_warp_after_sprite_generation_return();
                            tail
                        };
                        self.game_execution_scheduler.refine_scheduled_work(
                            GameWorkContinuation::FinishModule09LongLoad { step },
                            GameWorkContinuation::FinishModule09LongLoad { step: tail },
                        );
                        continue;
                    }
                    assert!(
                        matches!(
                            self.game_execution_scheduler.current_work(),
                            Some(GameWorkContinuation::FinishOverworldLoadOverlaysSpriteReload)
                        ),
                        "overworld sprite generation returned outside its suspended source caller",
                    );
                    if self.overworld_proximity_scan_saved_scroll.is_some() {
                        self.complete_overworld_proximity_scan_continuation();
                    }
                    assert!(
                        !reload_returned,
                        "overworld sprite generation return replayed"
                    );
                    reload_returned = true;
                }
                crate::OverworldSpriteReloadProgress::ReloadReturned => {
                    assert!(
                        matches!(
                            self.game_execution_scheduler.current_work(),
                            Some(GameWorkContinuation::FinishOverworldSpriteReloadTail { .. })
                        ),
                        "overworld sprite reload returned outside its suspended C caller",
                    );
                    if self.overworld_proximity_scan_saved_scroll.is_some() {
                        self.complete_overworld_proximity_scan_continuation();
                    }
                    assert!(!reload_returned, "overworld sprite reload return replayed");
                    reload_returned = true;
                }
            }
        }
        reload_returned
    }

    pub(super) fn original_timing_main_loop_progress(&self) -> Option<crate::MainLoopProgress> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_ref()?;
        let matches = receipts
            .semantic
            .iter()
            .filter_map(|receipt| match receipt {
                OriginalTimingSemanticReceipt::MainLoopProgress(progress) => Some(*progress),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple main-loop progress outcomes",
        );
        matches.first().copied()
    }

    pub(super) fn take_original_timing_main_loop_progress(
        &mut self,
    ) -> Option<crate::MainLoopProgress> {
        let progress = self.original_timing_main_loop_progress()?;
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let index = receipts
            .semantic
            .iter()
            .position(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::MainLoopProgress(candidate)
                        if *candidate == progress
                )
            })
            .expect("the observed live main-loop progress receipt disappeared before consumption");
        receipts.semantic.remove(index);
        Some(progress)
    }

    pub(super) fn take_original_timing_main_loop_iteration_returned_to_wait(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| {
                original_timing_receipt_completes_main_loop_common_suffix(receipt).then_some(index)
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot return one main iteration twice",
        );
        matches.first().is_some_and(|&index| {
            receipts.semantic.remove(index);
            true
        })
    }

    pub(super) fn original_timing_main_loop_iteration_returned_to_wait(&self) -> bool {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic.iter().any(|receipt| {
                        original_timing_receipt_completes_main_loop_common_suffix(receipt)
                    })
                })
    }

    pub(super) fn take_original_timing_pre_dungeon_module_returned(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_mut() else {
            return false;
        };
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::PreDungeonModuleReturned
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot return Module_PreDungeon twice",
        );
        matches.first().is_some_and(|&index| {
            receipts.semantic.remove(index);
            true
        })
    }

    pub(super) fn defer_original_timing_pre_dungeon_return_before_native_owner(&mut self) {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || !self.take_original_timing_pre_dungeon_module_returned()
        {
            return;
        }
        let progress = self
            .take_original_timing_main_loop_progress()
            .expect("Module_PreDungeon return omitted its main-loop progress receipt");
        assert_eq!(
            progress,
            crate::MainLoopProgress::CallStackContinued,
            "an early Module_PreDungeon return must still be on its source caller stack",
        );
        assert!(
            self.game_state.frame.main_module == 6
                && self.game_execution_scheduler.current_work().is_none(),
            "Module_PreDungeon returned without its delayed native Module 6 owner: main={:02x} work={:?}",
            self.game_state.frame.main_module,
            self.game_execution_scheduler.current_work(),
        );
        assert!(
            self.original_timing_pre_dungeon_return_pending.is_none(),
            "Module_PreDungeon return replayed before native ownership",
        );
        self.original_timing_pre_dungeon_return_pending = Some(progress);
    }

    pub(super) fn take_deferred_original_timing_pre_dungeon_return(
        &mut self,
    ) -> Option<crate::MainLoopProgress> {
        let progress = self.original_timing_pre_dungeon_return_pending.take()?;
        assert!(
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live),
            "a deferred Module_PreDungeon return outlived its timing authority",
        );
        assert!(
            self.game_state.frame.main_module == 6
                && self.game_execution_scheduler.current_work().is_none(),
            "a deferred Module_PreDungeon return reached the wrong native owner: main={:02x} work={:?}",
            self.game_state.frame.main_module,
            self.game_execution_scheduler.current_work(),
        );
        Some(progress)
    }

    pub(super) fn discard_original_timing_transient_nmi_authority(&mut self) {
        self.original_timing_semantic_receipts = None;
        self.original_timing_nmi_publication_pending = false;
        self.original_timing_pending_nmi_update_gate = None;
        self.original_timing_pending_nmi_ppu_register_operands = None;
        self.original_timing_expected_nmi_update_gates.clear();
        self.original_timing_expected_nmi_ppu_register_operands
            .clear();
    }

    pub(super) fn begin_original_timing_host_dispatch(&mut self, input_state: u16) -> bool {
        if self.original_timing_host_dispatch_active {
            return false;
        }
        assert!(
            !self.original_timing_scheduled_nmi_accepted_at_host_return,
            "the preceding source host left a staged NMI acceptance unclosed",
        );
        self.original_timing_host_dispatch_active = true;
        if let Some(receipts) = self.original_timing_semantic_receipts.as_ref() {
            if receipts.input_state == input_state {
                self.original_timing_cold_start_eligible = false;
                self.original_timing_last_oracle_host_call = Some(receipts.host_call);
                self.original_timing_owner = OriginalTimingOwnerState::Live;
            } else {
                self.original_timing_owner = OriginalTimingOwnerState::Unavailable(
                    OriginalTimingUnavailableReason::AuthorityReceiptInputMismatch,
                );
                // The receipt belongs to a different host input and therefore
                // cannot authorize any native NMI or carry a handler across the
                // boundary. Drop the complete transient authority atomically so
                // host-finalization cannot mistake its gates for executed work.
                self.discard_original_timing_transient_nmi_authority();
            }
        } else {
            self.original_timing_owner = match self.original_timing_owner {
                OriginalTimingOwnerState::PendingColdStart if self.rom.is_empty() => {
                    self.original_timing_cold_start_eligible = false;
                    OriginalTimingOwnerState::Unavailable(
                        OriginalTimingUnavailableReason::MissingRom,
                    )
                }
                OriginalTimingOwnerState::PendingColdStart | OriginalTimingOwnerState::Live => {
                    self.original_timing_cold_start_eligible = false;
                    OriginalTimingOwnerState::Unavailable(
                        OriginalTimingUnavailableReason::MissingAuthorityReceipt,
                    )
                }
                ref owner => owner.clone(),
            };
            if matches!(
                self.original_timing_owner,
                OriginalTimingOwnerState::Unavailable(
                    OriginalTimingUnavailableReason::MissingAuthorityReceipt
                )
            ) {
                // Losing the replaceable timing authority invalidates any NMI
                // it had carried across the preceding host boundary. Keeping
                // that gate after the receipt stream disappears would let a
                // later host execute source work without authority.
                self.discard_original_timing_transient_nmi_authority();
            }
        }
        true
    }

    pub(super) fn publish_original_timing_presented_obj_tiles(
        &mut self,
        receipt: crate::PresentedObjTiles,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.presented_obj_tiles_override = Some(receipt);
    }

    pub(super) fn apply_original_timing_presented_obj_tiles(
        &mut self,
        receipt: &crate::PresentedObjTiles,
    ) {
        let captured_obj_vram_latch = self.ppu.obj_vram_latch.clone();
        self.apply_original_timing_presented_obj_tiles_from_captured_scanout(
            receipt,
            captured_obj_vram_latch.as_deref(),
        );
    }

    pub(super) fn apply_original_timing_presented_obj_tiles_from_captured_scanout(
        &mut self,
        receipt: &crate::PresentedObjTiles,
        captured_obj_vram_latch: Option<&[u16]>,
    ) {
        if self.debug_obj_pipe_enabled() {
            eprintln!(
                "objtiles host={} base={} tiles={:04x?}",
                self.frame_ctr_dbg,
                if self.ppu.obj_vram_latch.is_some() {
                    "latch"
                } else if captured_obj_vram_latch.is_some() {
                    "captured_latch"
                } else if self.last_presented_obj_vram.is_some() {
                    "last_presented"
                } else {
                    "live"
                },
                receipt.tile_word_addresses,
            );
        }
        let mut obj_vram = self
            .ppu
            .obj_vram_latch
            .take()
            .or_else(|| captured_obj_vram_latch.map(<[u16]>::to_vec))
            .or_else(|| {
                self.last_presented_obj_vram
                    .as_ref()
                    .filter(|vram| vram.len() == self.ppu.vram.len())
                    .cloned()
            })
            .unwrap_or_else(|| self.ppu.vram.clone());
        let mut exact_content_sources = Vec::with_capacity(receipt.tile_word_addresses.len());
        for (tile, &tile_base) in receipt.tile_word_addresses.iter().enumerate() {
            let pixels = &receipt.tile_pixels[tile * crate::PresentedObjTiles::PIXELS_PER_TILE
                ..(tile + 1) * crate::PresentedObjTiles::PIXELS_PER_TILE];
            let tile_base = usize::from(tile_base);
            for y in 0..8 {
                let mut planes = [0_u8; 4];
                for x in 0..8 {
                    let pixel = pixels[y * 8 + x];
                    let bit = 1_u8 << (7 - x);
                    for (plane, value) in planes.iter_mut().enumerate() {
                        if pixel & (1 << plane) != 0 {
                            *value |= bit;
                        }
                    }
                }
                obj_vram[tile_base + y] = u16::from_le_bytes([planes[0], planes[1]]);
                obj_vram[tile_base + 8 + y] = u16::from_le_bytes([planes[2], planes[3]]);
            }
            let slot = tile_base / 16;
            let hash = crate::chr_source::chr_content_hash32(&obj_vram[tile_base..tile_base + 16]);
            exact_content_sources.push((slot, hash));
        }
        // The sparse receipt is pixel authority for exactly these tile
        // addresses. Re-key their logical identities by the resulting exact
        // content so the modern asset resolver cannot select an older upload
        // identity while the raw renderer sees the receipt pixels. Unmentioned
        // slots keep their existing source ownership.
        for (slot, hash) in exact_content_sources {
            self.vram_chr_source.record_tile_content_hash(
                slot,
                crate::chr_source::CHR_KIND_BG_STREAM,
                hash,
            );
            self.vram_chr_preview_source.record_tile_content_hash(
                slot,
                crate::chr_source::CHR_KIND_BG_STREAM,
                hash,
            );
        }
        self.set_obj_vram_latch_traced(Some(obj_vram));
    }

    pub(super) fn publish_original_timing_presented_animated_bg_tiles(
        &mut self,
        receipt: crate::PresentedAnimatedBgTiles,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.presented_animated_bg_tiles_override = Some(receipt);
    }

    pub(super) fn apply_original_timing_presented_animated_bg_tiles(
        &mut self,
        receipt: &crate::PresentedAnimatedBgTiles,
    ) {
        let destination = receipt.destination.word_address();
        let word_count = crate::PresentedAnimatedBgTiles::TILE_COUNT * 16;
        let Some(end) = destination.checked_add(word_count) else {
            return;
        };
        if end > self.ppu.vram.len() {
            debug_assert!(
                false,
                "animated BG presentation destination is outside VRAM"
            );
            return;
        }
        let mut bg_vram = self
            .ppu
            .bg_vram_latch
            .take()
            .unwrap_or_else(|| self.ppu.vram.clone());
        for tile in 0..crate::PresentedAnimatedBgTiles::TILE_COUNT {
            let tile_base = destination + tile * 16;
            let pixels = &receipt.tile_pixels[tile * 64..tile * 64 + 64];
            for y in 0..8 {
                let mut planes = [0_u8; 4];
                for x in 0..8 {
                    let pixel = pixels[y * 8 + x];
                    let bit = 1_u8 << (7 - x);
                    for (plane, value) in planes.iter_mut().enumerate() {
                        if pixel & (1 << plane) != 0 {
                            *value |= bit;
                        }
                    }
                }
                bg_vram[tile_base + y] = u16::from_le_bytes([planes[0], planes[1]]);
                bg_vram[tile_base + 8 + y] = u16::from_le_bytes([planes[2], planes[3]]);
            }
        }
        self.ppu.bg_vram_latch = (bg_vram != self.ppu.vram).then_some(bg_vram);
    }

    pub(super) fn publish_original_timing_presented_cgram(
        &mut self,
        receipt: crate::PresentedCgram,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.ppu.cgram.clone_from_slice(&receipt.colors);
        display.cgram_scanout_override = Some(receipt.colors);
    }

    pub(super) fn publish_original_timing_presented_oam(&mut self, receipt: crate::PresentedOam) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.presented_oam_override = Some(
            receipt
                .bytes
                .chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect(),
        );
    }

    pub(super) fn publish_original_timing_presented_hud_tilemap(
        &mut self,
        receipt: crate::PresentedHudTilemap,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.presented_hud_tilemap_override = Some(receipt.words);
    }

    pub(super) fn shadow_and_publish_original_timing_presented_bg_tilemaps(
        &mut self,
        receipt: crate::PresentedBgTilemaps,
    ) {
        let mut mismatched_geometry_layers = 0;
        let mut compared_words = 0;
        let mut mismatched_words = 0;
        let mut first_mismatch = None;
        for layer in &receipt.layers {
            let layer_index = usize::from(layer.layer);
            let native = &self.ppu.bg_layer[layer_index];
            if native.tilemap_adr != layer.word_address
                || native.tilemap_wider != layer.wider
                || native.tilemap_higher != layer.higher
            {
                mismatched_geometry_layers += 1;
            }
            for (offset, &authority) in layer.words.iter().enumerate() {
                let word = (usize::from(layer.word_address) + offset) & 0x7fff;
                compared_words += 1;
                if self.ppu.vram[word] != authority {
                    mismatched_words += 1;
                    first_mismatch.get_or_insert((layer.layer, offset));
                }
            }
        }
        self.original_timing_bg_tilemap_shadow_result =
            Some(crate::OriginalTimingBgTilemapShadowResult {
                mismatched_geometry_layers,
                compared_words,
                mismatched_words,
                first_mismatch,
            });

        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        for layer in receipt.layers {
            let layer_index = usize::from(layer.layer);
            display.ppu.bg_layer[layer_index].tilemap_adr = layer.word_address;
            display.ppu.bg_layer[layer_index].tilemap_wider = layer.wider;
            display.ppu.bg_layer[layer_index].tilemap_higher = layer.higher;
            for (offset, value) in layer.words.into_iter().enumerate() {
                let word = (usize::from(layer.word_address) + offset) & 0x7fff;
                display.ppu.vram[word] = value;
            }
        }
    }

    pub(super) fn shadow_and_publish_original_timing_presented_bg_scroll(
        &mut self,
        receipt: crate::PresentedBgScroll,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.presented_bg_scroll_override = Some(receipt);
    }

    pub(super) fn shadow_and_publish_original_timing_presented_mode7_transform(
        &mut self,
        receipt: crate::PresentedMode7Transform,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.presented_mode7_transform_override = Some(receipt);
    }

    pub(super) fn shadow_and_publish_original_timing_presented_window_mask(
        &mut self,
        receipt: crate::PresentedWindowMask,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.presented_window_mask_override = Some(receipt);
    }

    pub(super) fn publish_original_timing_presented_inidisp(
        &mut self,
        receipt: crate::PresentedInidisp,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.presented_inidisp_override = Some(receipt);
    }

    pub(super) fn publish_original_timing_presented_scanout_geometry(
        &mut self,
        receipt: crate::PresentedScanoutGeometry,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        display.presented_scanout_geometry_override = Some(receipt);
    }

    /// Classify the only source-control states which may remain at a live host
    /// return. Presentation authority is stored in independent receipt
    /// sidecars, so every semantic fact must have a native owner before close
    /// except one terminal NMI acceptance whose handler belongs to the next
    /// host. This is deliberately read-only: malformed authority fails before
    /// joypad, suffix, display, gate, or host-dispatch state can mutate.
    pub(super) fn original_timing_host_close_control(&self) -> OriginalTimingHostCloseControl {
        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .map(OriginalTimingHostReceipts::semantic)
            .unwrap_or_default();
        let control = match semantic {
            [] => OriginalTimingHostCloseControl::Exhausted,
            [OriginalTimingSemanticReceipt::NmiAccepted(gate)] => {
                OriginalTimingHostCloseControl::CarryTerminalAcceptance(*gate)
            }
            _ => {
                panic!(
                    "source host {} returned with unowned semantic control receipts: {semantic:?} frame={:?} pending_suffix={:?} scheduler={:?}",
                    self.frame_ctr_dbg,
                    self.game_state.frame,
                    self.pending_main_loop_common_suffix,
                    self.game_execution_scheduler,
                )
            }
        };
        let native_gate = if self.game_state.display.nmi_update_is_latched() {
            NmiUpdateGate::LatchHeld
        } else {
            NmiUpdateGate::Open
        };
        match control {
            OriginalTimingHostCloseControl::Exhausted => {
                if self.original_timing_nmi_publication_pending {
                    let [gate] = self.original_timing_expected_nmi_update_gates.as_slice() else {
                        panic!(
                            "a pending source NMI requires exactly one installed disposition: {:?}",
                            self.original_timing_expected_nmi_update_gates,
                        );
                    };
                    assert_eq!(
                        native_gate,
                        *gate,
                        "an explicitly carried source NMI disagrees with the native update latch at host close: host={} frame={:?} pending_suffix={:?} scheduler={:?}",
                        self.frame_ctr_dbg,
                        self.game_state.frame,
                        self.pending_main_loop_common_suffix,
                        self.game_execution_scheduler,
                    );
                    assert!(
                        self.display_snapshot
                            .as_ref()
                            .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
                        "an explicitly carried source NMI lost its receptive acceptance snapshot",
                    );
                } else {
                    // `original_timing_pending_nmi_update_gate` is the
                    // cross-host disposition from entry. A completed carry-in
                    // handler deliberately leaves it intact until this close;
                    // the drained installed-gate queue proves it was consumed.
                    assert!(
                        self.original_timing_expected_nmi_update_gates.is_empty(),
                        "host close retained installed NMI dispositions after all handlers completed",
                    );
                }
            }
            OriginalTimingHostCloseControl::CarryTerminalAcceptance(gate) => {
                assert!(
                    !self.original_timing_nmi_publication_pending,
                    "one source host cannot queue a terminal acceptance behind an unfinished handler",
                );
                assert_eq!(
                    self.original_timing_pending_nmi_update_gate, None,
                    "a terminal source acceptance cannot replace an existing pending disposition",
                );
                assert_eq!(
                    self.original_timing_expected_nmi_update_gates.as_slice(),
                    [gate],
                    "a terminal source acceptance must own exactly one installed disposition",
                );
                assert_eq!(
                    native_gate, gate,
                    "a terminal source acceptance disagrees with the native update latch",
                );
            }
        }
        control
    }

    pub(super) fn finish_original_timing_host_dispatch(&mut self, owns_dispatch: bool) {
        if !owns_dispatch {
            return;
        }
        assert_eq!(
            self.original_timing_sprite_main_return_claims_remaining, None,
            "original-timing host close reached an active Sprite_Main return claim scope (opened at {:?})",
            self.original_timing_sprite_main_return_claim_scope_site,
        );
        let host_close_control = self.original_timing_host_close_control();
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            let receipts = self
                .original_timing_semantic_receipts
                .as_mut()
                .expect("live host close lost its timing receipts");
            if !receipts.song_end_poll_native_sample_offsets.is_empty()
                && tolerate_unconsumed_song_end_poll_for_diagnostics()
            {
                // Oracle-seeded / resumed diagnostic probes do not carry the
                // oracle's audio driver state, so the native song-end poll
                // consumer can legitimately disagree with the wire. Video
                // evidence stays valid; audio is never authoritative here.
                eprintln!(
                    "diagnostic: dropping {} unconsumed APUI00 song-end poll offset(s) at host {}",
                    receipts.song_end_poll_native_sample_offsets.len(),
                    self.frame_ctr_dbg
                );
                receipts.song_end_poll_native_sample_offsets.clear();
            }
            assert!(
                receipts.song_end_poll_native_sample_offsets.is_empty(),
                "source APUI00 song-end poll timing was not consumed by its native caller",
            );
        }
        if let OriginalTimingHostCloseControl::CarryTerminalAcceptance(gate) = host_close_control {
            let receipts = self
                .original_timing_semantic_receipts
                .as_mut()
                .expect("validated terminal acceptance lost its host receipts");
            assert_eq!(
                receipts.semantic.as_slice(),
                [OriginalTimingSemanticReceipt::NmiAccepted(gate)],
                "validated terminal acceptance changed before consumption",
            );
            receipts.semantic.clear();
            self.capture_and_carry_original_timing_nmi_publication_at_host_return();
        }
        let presented_mode7_transform =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                .then(|| {
                    self.original_timing_semantic_receipts
                        .as_ref()
                        .and_then(|receipts| receipts.presented_mode7_transform.clone())
                })
                .flatten();
        let (
            presented_animated_bg_tiles,
            presented_cgram,
            presented_inidisp,
            presented_scanout_geometry,
            presented_hud_tilemap,
            presented_dialogue_text,
            presented_bg_tilemaps,
            presented_bg_scroll,
            presented_window_mask,
            presented_oam,
            presented_obj_tiles,
            presented_audio,
        ) = if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            self.original_timing_semantic_receipts
                .as_ref()
                .map(|receipts| {
                    (
                        receipts.presented_animated_bg_tiles.clone(),
                        receipts.presented_cgram.clone(),
                        receipts.presented_inidisp,
                        receipts.presented_scanout_geometry,
                        receipts.presented_hud_tilemap.clone(),
                        receipts.presented_dialogue_text.clone(),
                        receipts.presented_bg_tilemaps.clone(),
                        receipts.presented_bg_scroll.clone(),
                        receipts.presented_window_mask.clone(),
                        receipts.presented_oam.clone(),
                        receipts.presented_obj_tiles.clone(),
                        receipts.presented_audio.clone(),
                    )
                })
                .unwrap_or_default()
        } else {
            (
                None, None, None, None, None, None, None, None, None, None, None, None,
            )
        };
        if let Some(presented_animated_bg_tiles) = presented_animated_bg_tiles {
            self.publish_original_timing_presented_animated_bg_tiles(presented_animated_bg_tiles);
        }
        if let Some(presented_cgram) = presented_cgram {
            // The source main thread may already have authored a later palette
            // buffer. This receipt is the palette used by the surface returned
            // from the current host call, so publish it only to that immutable
            // display snapshot.
            self.publish_original_timing_presented_cgram(presented_cgram);
        }
        if let Some(presented_inidisp) = presented_inidisp {
            self.publish_original_timing_presented_inidisp(presented_inidisp);
        }
        if let Some(presented_scanout_geometry) = presented_scanout_geometry {
            self.publish_original_timing_presented_scanout_geometry(presented_scanout_geometry);
        }
        if let Some(presented_oam) = presented_oam {
            // The completed scanout may precede a VBlank OAM DMA that already
            // changed both Zelda's live shadow and the PPU's current bytes.
            // Publish the rendered generation only into the outgoing surface.
            self.publish_original_timing_presented_oam(presented_oam);
        }
        if let Some(presented_hud_tilemap) = presented_hud_tilemap {
            self.publish_original_timing_presented_hud_tilemap(presented_hud_tilemap);
        }
        self.original_timing_dialogue_text_shadow_result = None;
        if let Some(presented_dialogue_text) = presented_dialogue_text {
            self.publish_original_timing_presented_dialogue_text(presented_dialogue_text);
        }
        self.original_timing_bg_tilemap_shadow_result = None;
        if let Some(presented_bg_tilemaps) = presented_bg_tilemaps {
            self.shadow_and_publish_original_timing_presented_bg_tilemaps(presented_bg_tilemaps);
        }
        self.original_timing_bg_scroll_shadow_result = None;
        if let Some(presented_bg_scroll) = presented_bg_scroll {
            self.shadow_and_publish_original_timing_presented_bg_scroll(presented_bg_scroll);
        }
        self.original_timing_mode7_transform_shadow_result = None;
        if let Some(presented_mode7_transform) = presented_mode7_transform {
            self.shadow_and_publish_original_timing_presented_mode7_transform(
                presented_mode7_transform,
            );
        }
        self.original_timing_window_mask_shadow_result = None;
        if let Some(presented_window_mask) = presented_window_mask {
            self.shadow_and_publish_original_timing_presented_window_mask(presented_window_mask);
        }
        if let Some(presented_obj_tiles) = presented_obj_tiles {
            // The authority receipt describes the surface returned by this
            // host call. Apply it after translated execution has selected the
            // outgoing display snapshot, so later native main/NMI work cannot
            // overwrite the already-published tile pixels.
            self.publish_original_timing_presented_obj_tiles(presented_obj_tiles);
        }
        if let Some(presented_audio) = presented_audio {
            assert!(
                self.original_timing_presented_audio.is_none(),
                "an authoritative audio presentation must be consumed before the next host call",
            );
            self.original_timing_presented_audio = Some(presented_audio);
        }
        self.original_timing_pending_nmi_update_gate = match (
            self.original_timing_nmi_publication_pending,
            self.original_timing_expected_nmi_update_gates.as_slice(),
        ) {
            (false, []) => None,
            (true, [gate]) => Some(*gate),
            (pending, gates) => panic!(
                "source NMI update-gate ownership disagreed at host close: pending={pending} gates={gates:?}",
            ),
        };
        self.original_timing_pending_nmi_ppu_register_operands = match (
            self.original_timing_nmi_publication_pending,
            self.original_timing_expected_nmi_ppu_register_operands
                .as_slice(),
        ) {
            (false, []) => None,
            (true, [operands]) => *operands,
            (pending, operands) => panic!(
                "source NMI PPU-register ownership disagreed at host close: pending={pending} operands={operands:?}",
            ),
        };
        self.original_timing_expected_nmi_update_gates.clear();
        self.original_timing_expected_nmi_ppu_register_operands
            .clear();
        // Presentation receipt domains migrate independently after every
        // semantic control fact has been consumed by its native owner. Input,
        // host-call ordering, and duplicate facts remain fail-closed at
        // installation.
        assert!(
            self.original_timing_semantic_receipts
                .as_ref()
                .is_none_or(|receipts| receipts.dialogue_scroll_progress.is_empty()),
            "source dialogue scroll operations were not consumed by their native caller"
        );
        self.original_timing_semantic_receipts = None;
        self.original_timing_host_dispatch_active = false;
        self.original_timing_host_iteration_uninterrupted = false;
    }

    #[track_caller]
    pub(super) fn begin_original_timing_sprite_main_return_claim_scope(&mut self, claims: usize) {
        assert_eq!(
            self.original_timing_sprite_main_return_claims_remaining, None,
            "a Sprite_Main return claim scope overlapped an older native caller",
        );
        self.original_timing_sprite_main_return_claim_scope_site =
            Some(std::panic::Location::caller());
        self.original_timing_sprite_main_return_claims_remaining = Some(
            u8::try_from(claims).expect("one host published too many Sprite_Main return claims"),
        );
    }

    pub(super) fn finish_original_timing_sprite_main_return_claim_scope(&mut self) {
        assert_eq!(
            self.original_timing_sprite_main_return_claims_remaining,
            Some(0),
            "the native caller did not consume every claimed Sprite_Main return: scope opened at {:?} host={} frame={:?} scheduler={:?}",
            self.original_timing_sprite_main_return_claim_scope_site,
            self.frame_ctr_dbg,
            self.game_state.frame,
            self.game_execution_scheduler,
        );
        self.original_timing_sprite_main_return_claims_remaining = None;
    }

    pub fn last_original_timing_audio_shadow_result(
        &self,
    ) -> Option<crate::OriginalTimingAudioShadowResult> {
        self.original_timing_audio_shadow_result
    }

    pub fn last_original_timing_bg_tilemap_shadow_result(
        &self,
    ) -> Option<crate::OriginalTimingBgTilemapShadowResult> {
        self.original_timing_bg_tilemap_shadow_result
    }

    pub fn last_original_timing_bg_scroll_shadow_result(
        &self,
    ) -> Option<crate::OriginalTimingBgScrollShadowResult> {
        self.original_timing_bg_scroll_shadow_result
    }

    pub fn last_original_timing_mode7_transform_shadow_result(
        &self,
    ) -> Option<crate::OriginalTimingMode7TransformShadowResult> {
        self.original_timing_mode7_transform_shadow_result
    }

    pub fn last_original_timing_window_mask_shadow_result(
        &self,
    ) -> Option<crate::OriginalTimingWindowMaskShadowResult> {
        self.original_timing_window_mask_shadow_result
    }

    pub(crate) fn apply_original_timing_presented_audio(
        &mut self,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
    ) {
        self.original_timing_audio_shadow_result = None;
        let Some(receipt) = self.original_timing_presented_audio.take() else {
            return;
        };
        let Some(output_len) = usize::try_from(samples)
            .ok()
            .zip(usize::try_from(channels).ok())
            .and_then(|(samples, channels)| samples.checked_mul(channels))
        else {
            self.original_timing_owner = OriginalTimingOwnerState::Unavailable(
                OriginalTimingUnavailableReason::AuthorityAudioShapeMismatch,
            );
            return;
        };
        if channels as usize != crate::PresentedAudio::CHANNELS
            || output_len != receipt.interleaved_stereo.len()
            || audio_buffer.len() < output_len
        {
            self.original_timing_owner = OriginalTimingOwnerState::Unavailable(
                OriginalTimingUnavailableReason::AuthorityAudioShapeMismatch,
            );
            return;
        }
        let output = &mut audio_buffer[..output_len];
        let first_mismatch_interleaved = output
            .iter()
            .zip(&receipt.interleaved_stereo)
            .position(|(native, authority)| native != authority);
        let mismatched_interleaved_samples = output
            .iter()
            .zip(&receipt.interleaved_stereo)
            .filter(|(native, authority)| native != authority)
            .count();
        self.original_timing_audio_shadow_result = Some(crate::OriginalTimingAudioShadowResult {
            sample_frames: receipt.sample_frames(),
            mismatched_interleaved_samples,
            first_mismatch_interleaved,
        });
        output.copy_from_slice(&receipt.interleaved_stereo);
    }

    pub(super) fn begin_original_timing_flute_menu_selected_screen(&mut self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || !self.flute_menu_selected_screen_caller_is_active()
            || self.game_execution_scheduler.current_work().is_some()
        {
            return false;
        }
        let Some(receipt) = self.take_original_timing_sprite_reset_all_progress() else {
            return false;
        };
        assert_eq!(
            receipt.progress,
            SpriteResetAllProgress::SpriteDisableAllCompleted,
        );
        assert!(
            matches!(
                receipt.boundary,
                crate::OriginalTimingBoundary::HostReturn
                    | crate::OriginalTimingBoundary::NmiAccepted
            ),
            "the initial bird-travel Sprite_ResetAll checkpoint requires a host or NMI boundary",
        );
        self.FluteMenu_LoadTransportThroughInitialSpriteDisable();
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishFluteMenuSelectedScreen {
                step: FluteMenuSelectedScreenStep::InitialSpriteReset,
            },
            1,
        );
        true
    }

    pub(super) fn begin_item_receipt_graphics_work(
        &mut self,
        gfx: u8,
        receipt: ItemReceiptReturn,
        caller: ItemReceiptCaller,
    ) -> GameCallStatus {
        if !self.rom_startup_timing() {
            return GameCallStatus::Returned;
        }
        let nmi_slices = rom_item_receipt_graphics_nmi_slices(gfx);
        if nmi_slices == 0 {
            return GameCallStatus::Returned;
        }
        match self.game_state.player.follower_link.item_receipt_method() {
            // The two ROM entry paths whose decompression timing has been
            // measured. Their different entry boundaries are scheduled by
            // their semantic callers, not stored on the continuation.
            0 | 1 => {}
            // A sprite's own state machine can hand out an item with method
            // 2 (the heart-container pickup); its entry boundary is the
            // direct slot caller's Sprite_Main statement and the wire's
            // suspended claim proves the same measured decompression crosses
            // NMIs (route host 102905).
            2 if matches!(caller, ItemReceiptCaller::SpriteMainDirect { .. }) => {}
            // The falling milestone item hands out crystals/pendants with
            // method 3; the wire's suspended claim proves the decompression
            // crosses NMIs from Sprite_Main's prefix (route host 1142850).
            3 if matches!(caller, ItemReceiptCaller::AncillaMilestone { .. }) => {}
            // Other entry paths remain atomic until their ROM boundary is
            // measured.
            _ => return GameCallStatus::Returned,
        }
        let authoritative_caller = match caller {
            ItemReceiptCaller::SpriteMain { sprite_slot, .. }
                if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) =>
            {
                Some(ItemReceiptGraphicsCaller::SpriteMain { slot: sprite_slot })
            }
            ItemReceiptCaller::SpriteMainDirect { sprite_slot, .. }
                if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) =>
            {
                Some(ItemReceiptGraphicsCaller::SpriteMainDirect { slot: sprite_slot })
            }
            ItemReceiptCaller::UnclePassage { sprite_slot }
                if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) =>
            {
                Some(ItemReceiptGraphicsCaller::UnclePassage { slot: sprite_slot })
            }
            ItemReceiptCaller::AncillaMilestone { ancilla_slot, .. }
                if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) =>
            {
                Some(ItemReceiptGraphicsCaller::SpriteMainAncilla { slot: ancilla_slot })
            }
            _ => None,
        };
        let authoritative_progress = if let Some(expected_caller) = authoritative_caller {
            let progress = self
                .take_original_timing_item_receipt_graphics_progress()
                .unwrap_or_else(|| {
                    panic!(
                        "live timing authority omitted item-receipt progress for {expected_caller:?}"
                    )
                });
            assert_eq!(progress.caller, expected_caller);
            Some(progress)
        } else if caller == ItemReceiptCaller::AtomicCaller
            && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            self.take_original_timing_item_receipt_graphics_progress()
                .inspect(|progress| {
                    let slot = self.game_state.sprites.system.cur_object_index();
                    assert_eq!(
                        progress.caller,
                        ItemReceiptGraphicsCaller::SpriteMainDirect { slot },
                        "live direct item-receipt progress named a different source caller",
                    );
                })
        } else {
            None
        };
        if authoritative_progress.is_none()
            && caller == ItemReceiptCaller::AtomicCaller
            && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self.original_timing_host_iteration_uninterrupted
        {
            // The wire returns this whole iteration inside the host with no
            // NMI accepted between its progress and its common suffix, so the
            // decompression did not cross an NMI here (route host 514803,
            // the dug-up flute): no graphics work remains to schedule.
            return GameCallStatus::Returned;
        }
        if let Some(progress) = authoritative_progress {
            if progress.progress == SourceCallProgress::Returned {
                return GameCallStatus::Returned;
            }
            assert_ne!(
                caller,
                ItemReceiptCaller::AtomicCaller,
                "native Sprite_Main reached a direct item call before the source call returned: host={} slot={} sprite_type={:#04x} gfx={:#04x} method={} frame={:?}",
                self.frame_ctr_dbg,
                self.game_state.sprites.system.cur_object_index(),
                self.sprite_slot_view(usize::from(
                    self.game_state.sprites.system.cur_object_index()
                ))
                .sprite_type(),
                gfx,
                self.game_state.player.follower_link.item_receipt_method(),
                self.game_state.frame,
            );
        }
        let continuation = match caller {
            ItemReceiptCaller::AtomicCaller => {
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                    gfx,
                    ground_apress_tail: None,
                }
            }
            ItemReceiptCaller::GroundApress => {
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                    gfx,
                    ground_apress_tail: Some(receipt),
                }
            }
            ItemReceiptCaller::UnclePassage { sprite_slot } => {
                let dungeon = self
                    .active_dungeon_sprite_main_return
                    .take()
                    .expect("uncle passage item receipt must suspend a Module 7 sprite loop");
                ItemReceiptGraphicsContinuation::ResumeUnclePassage {
                    receipt,
                    sprite_slot,
                    dungeon,
                }
            }
            ItemReceiptCaller::AncillaMilestone {
                ancilla_slot,
                suffix,
            } => {
                let caller = if let Some(dungeon) = self.active_dungeon_sprite_main_return.take() {
                    SpriteMainItemReceiptCallerReturn::Module07(dungeon)
                } else if let Some(module09) = self.active_module09_sprite_main_return.take() {
                    SpriteMainItemReceiptCallerReturn::Module09(module09)
                } else {
                    panic!("milestone ancilla item receipt lost its native module caller frame");
                };
                ItemReceiptGraphicsContinuation::ResumeAncillaItemReceipt {
                    receipt,
                    ancilla_slot,
                    suffix,
                    caller,
                }
            }
            ItemReceiptCaller::SpriteMain {
                sprite_slot,
                suffix,
            }
            | ItemReceiptCaller::SpriteMainDirect {
                sprite_slot,
                suffix,
            } => {
                let caller = if let Some(dungeon) = self.active_dungeon_sprite_main_return.take() {
                    SpriteMainItemReceiptCallerReturn::Module07(dungeon)
                } else if let Some(module09) = self.active_module09_sprite_main_return.take() {
                    SpriteMainItemReceiptCallerReturn::Module09(module09)
                } else {
                    panic!("Sprite_Main item receipt lost its native module caller frame");
                };
                ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt {
                    receipt,
                    sprite_slot,
                    suffix,
                    caller,
                }
            }
        };
        let call_status = match continuation {
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                ground_apress_tail: None,
                ..
            } => GameCallStatus::Returned,
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                ground_apress_tail: Some(_),
                ..
            }
            | ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. }
            | ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt { .. }
            | ItemReceiptGraphicsContinuation::ResumeAncillaItemReceipt { .. } => {
                GameCallStatus::Suspended
            }
        };
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishItemReceiptGraphics { continuation },
            nmi_slices,
        );
        call_status
    }

    /// The live wire's own Sprite_Main activity in a continuation host (no
    /// fresh iteration owns those receipts) proves the resumed Module 7
    /// caller already reached Sprite_Main within this host; the completion
    /// arm must run that caller now instead of deferring it a slice (route
    /// host 95851, the spiral-stairs caller interrupted at slot 1).
    pub(super) fn wire_spiral_caller_resumed_this_host(&self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || self.original_timing_hosts_fresh_iteration()
        {
            return false;
        }
        let interrupted_in_sprite_main = self
            .original_timing_semantic_receipts
            .as_ref()
            .and_then(OriginalTimingHostReceipts::forwarded_main_loop_interruption)
            .is_some_and(|forwarded| forwarded.interruption().is_sprite_main())
            || self
                .original_timing_main_loop_interruption()
                .is_some_and(crate::MainLoopInterruption::is_sprite_main);
        // The wire can also carry the caller past Sprite_Main into a module
        // CPU phase (LinkOam) within this host; the Sprite_Main return is then
        // owned by that continuation timeline rather than owed separately
        // (route host 1509192, spiral room $5d initialization).
        let continued_into_module_tail =
            self.original_timing_main_loop_interruption()
                .is_some_and(|interruption| {
                    module_cpu_phase_from_main_loop_interruption(interruption).is_some()
                });
        self.original_timing_owes_sprite_main_return()
            || self.original_timing_owes_sprite_main_progress()
            || interrupted_in_sprite_main
            || continued_into_module_tail
    }

    pub(super) fn complete_dungeon_supertile_caller_return_host(
        &mut self,
        work: DungeonSupertileTransitionWork,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        wire_owns_return: bool,
    ) {
        debug_assert!(matches!(
            work,
            DungeonSupertileTransitionWork::RoomLoadCallerResume
                | DungeonSupertileTransitionWork::SpriteConversionCallerResume
        ));
        if work == DungeonSupertileTransitionWork::RoomLoadCallerResume
            && self.active_dungeon_sprite_main_return.is_some()
        {
            let sprite_return = self
                .active_dungeon_sprite_main_return
                .take()
                .expect("post-Sprite_Main caller suffix must retain Module 7 return state");
            self.complete_module07_after_sprite_main(sprite_return);
        } else {
            self.complete_module07_dungeon_after_submodule();
        }
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            // The live wire suspended the resumed caller inside its
            // post-Sprite_Main Link OAM (route host 89672); the suffix and
            // main-wait return belong to the scheduled continuation.
            return;
        }
        let mut suffix_retired_before_trailing_nmi = false;
        if !wire_owns_return {
            if self.pending_main_loop_common_suffix.is_some()
                && !self.original_timing_live_suffix_outstanding()
            {
                // The wire completes the shared suffix BEFORE its trailing
                // acceptance (the vector holds the suffix receipt and the
                // trailing Open): the latch must clear before that NMI runs
                // (route host 124700).
                self.complete_pending_main_loop_common_suffix_after_module_return();
                suffix_retired_before_trailing_nmi = true;
            }
            if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && (!self.original_timing_expected_nmi_update_gates.is_empty()
                    || self.original_timing_host_dispatch_active)
            {
                // The wire owns any trailing acceptance at this return
                // boundary. With a gate remaining, the trailing NMI was
                // accepted here but its handler belongs to the next host and
                // the generic scheduled-caller host-return staging carries it
                // (route host 124700). With no gate remaining under active
                // dispatch, the wire accepted no NMI at all this host — the
                // caller return ends at the main wait and the next host's
                // receipt owns the following acceptance (route host 132388).
                // Synthesizing the NMI here would run a handler the typed
                // authority never granted.
            } else {
                self.capture_display_snapshot();
                self.interrupt_nmi(input, oam_dma_source, false);
            }
        }
        self.restore_room_61_sprite_conversion_resident_oam();
        if work == DungeonSupertileTransitionWork::RoomLoadCallerResume {
            self.stage_live_animated_bg_scanout();
        }
        if suffix_retired_before_trailing_nmi {
            // Already retired above, in wire order.
        } else if self.pending_main_loop_common_suffix.is_some() {
            // The source-proven caller return already carries the shared
            // ZeldaRunGameLoop suffix; retire its one owner.
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
        }
        if work == DungeonSupertileTransitionWork::RoomLoadCallerResume {
            self.finish_dungeon_room_load_caller_at_main_wait();
        } else {
            self.clear_nmi_update_latch();
        }
        if work.next_module_resumes_after_pre_main_nmi() {
            self.game_execution_scheduler
                .schedule_pre_main_nmi_resume(PreMainNmiResume::DungeonSupertileQuadrantUploads);
        }
        if room_61_sprite_conversion_retains_resident_oam(
            work,
            self.game_state.world.location.dungeon_room_index(),
        ) {
            self.active_display_obj_generation = DisplayObjGeneration::FollowModuleCadence;
        }
    }

    pub(super) fn atomic_item_graphics_uses_partial_receipt_obj_cache(&self) -> bool {
        matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishItemReceiptGraphics {
                continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx, .. },
            }) if gfx != 0x22
        )
    }

    pub(super) fn publish_or_defer_item_receipt_sound_effect_2(&mut self, value: u8) {
        let enemy_drop_graphics_are_pending = matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishItemReceiptGraphics {
                continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                    gfx: 0x22,
                    ..
                },
            })
        );
        if enemy_drop_graphics_are_pending {
            debug_assert!(
                self.enemy_drop_item_graphics_deferred_sound_effect_2
                    .is_none(),
                "enemy-drop item graphics may defer only one receipt sound command"
            );
            self.enemy_drop_item_graphics_deferred_sound_effect_2 = Some(value);
            return;
        }
        self.set_sound_effect_2(value);
    }

    pub(super) fn item_receipt_graphics_return_uses_ordinary_module_epilogue(
        &self,
        continuation: ItemReceiptGraphicsContinuation,
    ) -> bool {
        if !matches!(
            continuation,
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x21, .. }
        ) {
            return false;
        }
        self.game_state.player.follower_link.handler_state() != 21
    }

    /// Open the write scope for an ordinary Rust trailing-NMI call.
    ///
    /// Most atomic main iterations have already completed; their appended NMI
    /// is the next hardware field and must not rewrite the immutable scanout.
    /// When the original 65816 call stack is suspended across NMI, however,
    /// that boundary occurred inside the current host slice and its completed
    /// `WritePpuRegisters` result belongs to the accepted active snapshot.
    pub(super) fn begin_trailing_nmi_receipts(&mut self) {
        self.begin_effective_presented_dma();
        let active_field_precedes_scheduled_work = self
            .game_execution_scheduler
            .active_field_precedes_current_scheduled_work();
        let owns_active_scanout = !active_field_precedes_scheduled_work
            && (self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
                || self
                    .game_execution_scheduler
                    .main_call_stack_is_suspended_before_nmi());
        if let Some(writes) = self.active_effective_dma_writes.as_mut() {
            writes.completed_ppu_registers_own_active_scanout = owns_active_scanout;
        }
    }

    pub(super) fn record_trailing_nmi_receipts(&mut self) {
        let Some(writes) = self.active_effective_dma_writes.take() else {
            return;
        };
        let dialogue_text_token = self.dialogue_text_dma_publication_token(&writes);
        let active_snapshot_accepts_receipt = writes.active_snapshot_accepts_receipt;
        let completed_ppu_registers_own_active_scanout =
            writes.completed_ppu_registers_own_active_scanout;
        let mut receipt = EffectivePresentedDma::from_write_set(writes, self);

        if dialogue_text_token.is_some() {
            let dialogue_receipt = EffectivePresentedDma {
                vram_writes: receipt.take_vram_range(0x7c00..0x7ff0),
                decoded_bg_vram_writes: Vec::new(),
                completed_oam: None,
                completed_link_obj_dma: None,
                completed_cgram: None,
                completed_ppu_registers: None,
                completed_dialogue_metadata: receipt.completed_dialogue_metadata.take(),
            };
            let active = self
                .display_snapshot
                .as_mut()
                .expect("dialogue text-DMA receipt requires its receptive acceptance snapshot");
            if let Some(existing) = active.effective_presented_dma.as_mut() {
                existing.merge_after(dialogue_receipt);
            } else {
                active.effective_presented_dma = Some(dialogue_receipt);
            }
        }
        let dialogue_text_token =
            self.consume_staged_dialogue_text_dma_publication(dialogue_text_token);
        assert!(
            dialogue_text_token.is_none(),
            "a trailing BG3 text-DMA publication completed before its immediate dialogue staging boundary",
        );

        // C `Interrupt_NMI` calls `WritePpuRegisters` after the optional DMA
        // work on every register-publishing vblank. A translated call may be
        // suspended across that boundary, so its immutable display snapshot
        // cannot rely on a later host capture to discover the completed PPU
        // generation. Attach the observed coupled register result to the
        // snapshot that explicitly accepted this NMI; no module, room, or
        // frame identity participates in the decision.
        if let Some(registers) = receipt.completed_ppu_registers.take() {
            if active_snapshot_accepts_receipt && completed_ppu_registers_own_active_scanout {
                let active = self
                    .display_snapshot
                    .as_mut()
                    .expect("trailing NMI register receipt requires an active display snapshot");
                let register_receipt = EffectivePresentedDma::ppu_registers_only(registers);
                if let Some(existing) = active.effective_presented_dma.as_mut() {
                    existing.merge_after(register_receipt);
                } else {
                    active.effective_presented_dma = Some(register_receipt);
                }
            }
        }

        let accepts_decoded_bg_receipt = |snapshot: &DisplaySnapshot| {
            snapshot.accepts_nmi_dma_receipts
                && snapshot.animated_bg_scanout_generation
                    == AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
        };
        if !self
            .deferred_display_snapshot
            .as_deref()
            .is_some_and(&accepts_decoded_bg_receipt)
        {
            // A normal trailing DMA becomes resident PPU state for a later
            // capture. It can refine only an already-materialized following
            // field, never the active display snapshot.
            return;
        }

        let destination = self
            .game_state
            .display
            .animated_tile_vram_destination_usize();
        receipt.completed_link_obj_dma = None;
        receipt.decoded_bg_vram_writes =
            receipt.take_vram_range(destination..destination.saturating_add(0x200));
        if receipt.decoded_bg_vram_writes.is_empty() {
            return;
        }
        receipt.vram_writes.clear();
        receipt.completed_oam = None;
        receipt.completed_cgram = None;
        receipt.completed_ppu_registers = None;
        receipt.completed_dialogue_metadata = None;
        let scanout = self
            .deferred_display_snapshot
            .as_mut()
            .expect("decoded BG receipt selected a missing deferred snapshot");
        if let Some(existing) = scanout.effective_presented_dma.as_mut() {
            existing.merge_after(receipt);
        } else {
            scanout.effective_presented_dma = Some(receipt);
        }
    }

    pub(super) fn close_display_boundary_dma_receipts(&mut self) {
        if let Some(display) = self.display_snapshot.as_mut() {
            display.accepts_nmi_dma_receipts = false;
        }
    }

    /// Whether the live wire keeps this host's shared `ZeldaRunGameLoop`
    /// suffix outstanding: neither a suffix-completion receipt nor an Open
    /// trailing acceptance (nor its consumed installed gate) exists, so the
    /// suspended caller cannot have reached the suffix within this host.
    /// A live host whose wire shows nothing but held NMI acceptances inside a
    /// continued call stack: no interruption checkpoint, no Sprite_Main
    /// activity, no terminal return, no fresh iteration. The acceptance and
    /// progress receipts are already folded into the retained host facts by
    /// the time scheduled work advances, so those facts are consulted here.
    pub(super) fn original_timing_host_is_bare_held_continuation(&self) -> bool {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self.game_execution_scheduler.call_stack_is_suspended()
            && !self.original_timing_hosts_fresh_iteration()
            && self.original_timing_main_loop_interruption().is_none()
            && self.original_timing_main_loop_return_timeline().is_none()
            && !self.original_timing_owes_sprite_main_return()
            && !self.original_timing_owes_sprite_main_progress()
    }

    pub(super) fn original_timing_live_suffix_outstanding(&self) -> bool {
        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    !receipts.semantic().iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                                // An Open trailing acceptance proves the
                                // suffix's latch clear already ran even
                                // when the suffix receipt is elided.
                                | OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open)
                        )
                    })
                })
            // The trailing Open acceptance may already have been consumed
            // into the staged host-return carry; its installed gate still
            // proves the suffix's latch clear ran on the wire.
            && !self
                .original_timing_expected_nmi_update_gates
                .contains(&NmiUpdateGate::Open)
    }

    pub(super) fn resumed_dungeon_caller_audio_follows_host_publication(&self) -> bool {
        self.dungeon_landing_entry_started_after_leading_nmi
            && self
                .game_execution_scheduler
                .scheduled_work_slices_remaining()
                == Some(1)
            && self.game_execution_scheduler.current_work()
                == Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn)
    }

    pub(super) fn run_frame_internal_after_original_timing(&mut self, input: u16, run_what: u8) {
        self.run_frame_internal_after_original_timing_body(input, run_what);
        self.finish_original_timing_scheduled_caller_host_return();
    }

    pub(super) fn run_frame_internal_after_original_timing_body(
        &mut self,
        input: u16,
        run_what: u8,
    ) {
        // The crystal maiden's poly thread owns every other host through the
        // NMI thread switch while the main thread's iteration stays suspended;
        // it renders regardless of which lane owns the main thread's host
        // (route hosts 413552/413554), so run it before any lane returns.
        // The ROM's crystal thread is IRQ-driven: every host gives it the
        // IRQ-to-NMI remainder after the main iteration, and its frame
        // completion is visible to the maiden's iteration of the host that
        // holds the completing slice (oracle `$1F0A` idles to `$1F31` on the
        // completing host, 413556/413560/…; the maiden steps the same host).
        // The atomic frontend's RUN_POLY alternation does not apply here.
        // The dungeon and Triforce-room V-IRQ threads render a slice every
        // host regardless of the frontend's poly/main alternation.
        let dungeon_poly_thread_host = self.rom_startup_timing()
            && (self.dungeon_poly_thread_is_active() || self.triforce_room_poly_thread_is_active());
        if dungeon_poly_thread_host {
            self.zelda_run_poly_loop();
        }
        if std::mem::take(&mut self.original_timing_carried_suffix_completion_pending) {
            // The previous host returned between NMI_PrepareSprites and the
            // `$12` clear; this host's first source event is that clear, then
            // the main wait accepts the leading Open NMI (route host 511525).
            assert!(
                self.pending_main_loop_common_suffix.is_some(),
                "a carried suffix completion requires the suspended ZeldaRunGameLoop suffix",
            );
            self.complete_pending_main_loop_common_suffix_after_module_return();
            if self
                .game_execution_scheduler
                .resumed_call_stack_is_before_nmi()
                || self.game_execution_scheduler.is_idle()
            {
                self.game_execution_scheduler
                    .finish_call_stack_at_main_wait_before_nmi();
            }
        }
        if crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some()
            && (self.dungeon_poly_thread_is_active()
                || (self.game_state.frame.main_module == 7
                    && self.sprite_slot_view(15).sprite_type() == 0xab))
        {
            eprintln!(
                "[POLY] host={} run_what={} thread_active={} stack={:#x} sub={:02x}/{:02x} maiden_ai={} e={} did_run_step={} pending={} sched={:?}",
                self.frame_ctr_dbg,
                run_what,
                self.game_state.display.nmi_thread_active,
                self.game_state.display.nmi_thread_stack_pointer,
                self.game_state.frame.submodule,
                self.game_state.frame.subsubmodule,
                self.sprite_slot_view(15).ai_state(),
                self.sprite_slot_view(15).e(),
                self.game_state.ending.attract_scene.intro_did_run_step(),
                self.game_state.display.has_pending_polyhedral_update(),
                self.game_execution_scheduler.current_work(),
            );
        }
        if matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::HoldOverworldSpriteReloadReturn)
        ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self.original_timing_hosts_fresh_iteration()
        {
            // The estimate held the reload's next main-loop iteration one
            // scanout, but the live wire begins that fresh iteration in this
            // very host: the ROM did not linger in submodule 5. Refute the
            // hold and let the iteration run after the ordinary pre-main
            // resume (route host 108685).
            let step = self
                .game_execution_scheduler
                .advance_work_one_nmi_slice_with_authoritative_completion(true);
            debug_assert_eq!(
                step,
                Some(GameWorkStep::Complete(
                    GameWorkContinuation::HoldOverworldSpriteReloadReturn
                )),
            );
            self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                PreMainNmiResume::OverworldSpriteReloadReturn {
                    scanout: OverworldSpriteReloadResumeScanout::ByReturnPhase(NmiPhase::BeforeNmi),
                },
            );
        }
        // This long C caller spans many host calls.  Select and fully validate
        // its source lifecycle before ordinary host setup, audio preparation,
        // or scheduler normalization can mutate state.  A sentinel first
        // armed during this host intentionally has no plan and remains owned
        // by the ordinary IterationStarted path.
        let intro_memory_darken_plan = self.original_timing_intro_memory_darken_plan();
        let save_quit_reset_plan = self.original_timing_save_quit_reset_plan();
        let intro_poly_plan = self.original_timing_intro_poly_plan(run_what);
        let mut selected_game_load_plan = self.original_timing_selected_game_load_plan();
        let mut begin_selected_game_load_plan =
            self.original_timing_begin_selected_game_load_plan();
        let mut selected_game_load_message_interface_plan =
            self.original_timing_selected_game_load_message_interface_plan();
        let pre_audio_selected_game_waiting_plan = if self.rom_startup_timing()
            && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && selected_game_load_plan.is_none()
            && begin_selected_game_load_plan.is_none()
            && selected_game_load_message_interface_plan.is_none()
            && self
                .game_execution_scheduler
                .selected_game_load_destination()
                .is_some()
            && self
                .game_execution_scheduler
                .selected_game_load_after_pre_dungeon_audio_sprite_reset()
                .is_none()
        {
            let mut scheduler_probe = self.game_execution_scheduler;
            (scheduler_probe.advance_startup_sequence()
                == Some(StartupSequenceStep::SelectedGameLoadWaiting))
            .then(|| {
                self.original_timing_selected_game_nonterminal_plan()
                    .expect("live pre-audio selected-game wait lost its continued-call owner")
            })
        } else {
            None
        };
        if self.rom_startup_timing()
            && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && selected_game_load_plan.is_none()
            && self
                .game_execution_scheduler
                .selected_game_load_destination()
                .is_some()
            && self
                .game_execution_scheduler
                .selected_game_load_after_pre_dungeon_audio_sprite_reset()
                .is_none()
        {
            assert!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .is_none_or(|receipts| !receipts.semantic.iter().any(|receipt| matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::SpriteResetAllProgress(_)
                    ))),
                "pre-audio selected-game caller cannot publish Sprite_ResetAll progress",
            );
            assert!(
                self.original_timing_main_loop_return_timeline().is_none(),
                "pre-audio selected-game caller cannot return before its source entry boundary",
            );
        }
        let file_select_low_wram_publication = self
            .original_timing_semantic_receipts
            .as_ref()
            .and_then(|receipts| {
                receipts.semantic.iter().find_map(|receipt| match receipt {
                    OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramClearProgress(
                        progress,
                    ) => Some(OriginalTimingFileSelectLowWramPublication::Progress(
                        *progress,
                    )),
                    OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramCleared => {
                        Some(OriginalTimingFileSelectLowWramPublication::Complete)
                    }
                    _ => None,
                })
            });
        let file_select_authoritative_probe = if self.rom_startup_timing()
            && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            if self
                .game_execution_scheduler
                .file_select_graphics_is_loading()
                == Some(false)
            {
                panic!(
                    "live file-select graphics retained ResumeModule past its source-owned terminal host",
                );
            }
            // Mirror the shared host-boundary normalization in the probe so
            // committing it later cannot roll `cpu_host_phase` backward.
            let mut scheduler_before_transition = self.game_execution_scheduler;
            scheduler_before_transition.begin_host_frame();
            let mut scheduler_after_transition = scheduler_before_transition;
            let completes_caller = self.original_timing_main_loop_return_timeline().is_some();
            scheduler_after_transition
                .advance_file_select_graphics_with_authoritative_completion(completes_caller)
                .map(|step| {
                    (
                        step,
                        scheduler_before_transition,
                        scheduler_after_transition,
                    )
                })
        } else {
            None
        };
        let nonterminal_file_select_plan = file_select_authoritative_probe
            .as_ref()
            .filter(|(step, _, _)| *step == StartupSequenceStep::FileSelectWaiting)
            .map(|_| {
                self.original_timing_nonterminal_continuation_plan_with_receipt(
                    file_select_low_wram_publication
                        .map(|publication| {
                            match publication {
                        OriginalTimingFileSelectLowWramPublication::Progress(progress) => {
                            OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramClearProgress(
                                progress,
                            )
                        }
                        OriginalTimingFileSelectLowWramPublication::Complete => {
                            OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramCleared
                        }
                    }
                        })
                        .map(|receipt| {
                            let semantic = &self
                                .original_timing_semantic_receipts
                                .as_ref()
                                .unwrap()
                                .semantic;
                            let index = semantic
                                .iter()
                                .position(|candidate| *candidate == receipt)
                                .unwrap();
                            let placement = if matches!(
                                semantic.get(index + 1),
                                Some(OriginalTimingSemanticReceipt::NmiAccepted(
                                    NmiUpdateGate::LatchHeld
                                ))
                            ) {
                                OriginalTimingNonterminalReceiptPlacement::BeforeTrailingAcceptance
                            } else {
                                OriginalTimingNonterminalReceiptPlacement::BeforeProgress
                            };
                            (receipt, placement)
                        }),
                )
                .expect("live nonterminal file-select graphics lost its continued-call owner")
            });
        let terminal_intro_initialization_return_timeline = (self.rom_startup_timing()
            && self.intro_initialization_work_frames_pending != 0
            && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live))
        .then(|| self.original_timing_main_loop_return_timeline())
        .flatten();
        if let Some(timeline) = terminal_intro_initialization_return_timeline.as_ref() {
            // A source terminal suffix cannot be downgraded into the coarse
            // nonterminal delay merely because another translated scheduler
            // owner is active. Reject the ownership conflict before either
            // parser consumes the shared progress/NMI receipts.
            assert!(
                self.game_execution_scheduler.is_idle(),
                "terminal intro initialization return overlaps scheduled translated work: {timeline:?}",
            );
            assert_eq!(
                timeline.progress,
                crate::MainLoopProgress::CallStackContinued,
                "the terminal intro caller cannot begin a second ZeldaRunGameLoop iteration",
            );
            assert_eq!(
                self.pending_main_loop_common_suffix,
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                "terminal intro initialization returned through an incompatible main-loop suffix",
            );
            assert!(
                self.original_timing_main_loop_interruption().is_none(),
                "terminal intro initialization return cannot also interrupt the main-loop suffix",
            );
        }
        let prospective_uninterrupted_idle_continued_return_plan =
            self.original_timing_uninterrupted_idle_continued_return_plan();
        let prospective_terminal_ground_item_receipt_plan =
            self.original_timing_terminal_ground_item_receipt_plan();
        let prospective_spotlight_build_link_oam_plan =
            self.original_timing_spotlight_build_link_oam_plan();
        let prospective_terminal_spotlight_iteration_plan =
            self.original_timing_terminal_spotlight_iteration_plan();
        self.lane_live_owner_main_loop_return_timeline(
            prospective_terminal_ground_item_receipt_plan.clone(),
            prospective_terminal_spotlight_iteration_plan.clone(),
        );
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self.game_execution_scheduler.is_idle()
            && matches!(
                self.pending_main_loop_common_suffix,
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
                    | Some(
                        MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch { .. }
                    )
            )
            && self.original_timing_main_loop_progress()
                == Some(crate::MainLoopProgress::CallStackContinued)
            && self.original_timing_main_loop_return_timeline().is_none()
        {
            // The resumed NMI_PrepareSprites suffix was interrupted again
            // before returning; the receipt re-states the suffix this state
            // already holds pending, so the pending owner stays armed.
            let _ = self.take_original_timing_main_loop_interruption(
                crate::MainLoopInterruption::SpritePreparation,
            );
        }
        let prospective_interrupted_idle_main_loop_plan =
            (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self
                    .original_timing_main_loop_interruption()
                    // On an idle host a plain sprite-preparation interruption
                    // stays on the receipt bus: the dungeon module body
                    // consumes it as its own suffix continuation, not this
                    // outer host timeline. A scheduled predecessor's same-host
                    // fresh iteration still owns it through this plan.
                    .is_some_and(|interruption| {
                        interruption != crate::MainLoopInterruption::SpritePreparation
                            || self.game_execution_scheduler.current_work().is_some()
                    })
                && self.original_timing_main_loop_progress()
                    == Some(crate::MainLoopProgress::IterationStarted))
            .then(|| self.original_timing_interrupted_idle_main_loop_plan())
            .flatten();
        let prospective_uninterrupted_idle_main_loop_plan =
            (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                // The suspended save-quit reset caller owns every host of its
                // hold, including the one carrying Module17's Sprite_Main
                // return (route host 159403).
                && !self.save_quit_reset_hold
                && (run_what & crate::RUN_MAIN != 0
                    || self.original_timing_main_loop_progress().is_some())
                && (self.game_execution_scheduler.is_idle()
                    || (self.original_timing_main_loop_progress()
                        == Some(crate::MainLoopProgress::IterationStarted)
                        && (self
                            .game_execution_scheduler
                            .pre_main_nmi_resume()
                            .is_some()
                            // Atomic item-receipt graphics slices do not
                            // suspend the translated call stack: the ROM's
                            // main loop keeps iterating (the pendant
                            // receipt's dialogue opens across them) while
                            // the decompression holds only the NMI latch
                            // (route host 158014).
                            || matches!(
                                self.game_execution_scheduler.current_work(),
                                Some(GameWorkContinuation::FinishItemReceiptGraphics {
                                    continuation:
                                        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                                            ..
                                        },
                                })
                            ))))
                && self
                    .original_timing_main_loop_interruption()
                    .is_none_or(|interruption| {
                        interruption == crate::MainLoopInterruption::SpritePreparation
                    })
                && !(self.original_timing_main_loop_progress()
                    == Some(crate::MainLoopProgress::CallStackContinued)
                    && self.original_timing_main_loop_return_timeline().is_some()))
            .then(|| {
                // A libretro host interval can finish an already-running NMI
                // or Zelda call stack without entering ZeldaRunGameLoop at
                // all. Validate the complete source lifecycle before host
                // setup, audio preparation, or scheduler normalization can
                // mutate state; consumption remains deferred until execution.
                let progress = self.original_timing_main_loop_progress()?;
                self.original_timing_uninterrupted_idle_main_loop_plan(progress)
            })
            .flatten();
        if let Ok(range) = crate::debug_env::var("ZELDA3_DEBUG_IDLE_PLAN") {
            let in_range = range
                .split_once('-')
                .and_then(|(lo, hi)| Some((lo.parse::<u32>().ok()?, hi.parse::<u32>().ok()?)))
                .is_some_and(|(lo, hi)| (lo..=hi).contains(&self.frame_ctr_dbg));
            if in_range {
                eprintln!(
                    "[PLAN] f={} progress={:?} interruption={:?} pending_suffix={:?} idle={} owner_live={} interrupted_plan={} uninterrupted_plan={} return_tl={} run_what={:x}",
                    self.frame_ctr_dbg,
                    self.original_timing_main_loop_progress(),
                    self.original_timing_main_loop_interruption(),
                    self.pending_main_loop_common_suffix,
                    self.game_execution_scheduler.is_idle(),
                    matches!(self.original_timing_owner, OriginalTimingOwnerState::Live),
                    prospective_interrupted_idle_main_loop_plan.is_some(),
                    prospective_uninterrupted_idle_main_loop_plan.is_some(),
                    self.original_timing_main_loop_return_timeline().is_some(),
                    run_what,
                );
            }
        }
        self.sync_native_game_state_from_ram();
        self.oam_law_entry_frame_counter = Some(self.game_state.frame.frame_counter);
        if nmi::debug_frame_selection_env_matches("ZELDA3_DEBUG_LINK_FALL", self.frame_ctr_dbg) {
            use crate::game_state::constants::{
                FRAME_COUNTER, LINK_ACTUAL_Y_VELOCITY, LINK_AUXILIARY_STATE, LINK_HANDLER_STATE,
                LINK_X_COORD, LINK_Y_COORD, LINK_Y_SUBPIXEL, LINK_Z_COORD, MAIN_MODULE, SUBMODULE,
                SUBSUBMODULE,
            };
            eprintln!(
                "link_fall host={} in={:04x} y={:04x} x={:04x} z={:04x} yvel={:02x} subpix={:02x} handler={:02x} state={:02x} fc={:02x} mod={:02x}/{:02x}/{:02x}",
                self.frame_ctr_dbg,
                input,
                read_le_u16(&self.ram, LINK_Y_COORD),
                read_le_u16(&self.ram, LINK_X_COORD),
                read_le_u16(&self.ram, LINK_Z_COORD),
                self.ram[LINK_ACTUAL_Y_VELOCITY],
                self.ram[LINK_Y_SUBPIXEL],
                self.ram[LINK_HANDLER_STATE],
                self.ram[LINK_AUXILIARY_STATE],
                self.ram[FRAME_COUNTER],
                self.ram[MAIN_MODULE],
                self.ram[SUBMODULE],
                self.ram[SUBSUBMODULE],
            );
        }
        self.game_execution_scheduler.begin_host_frame();
        if self
            .game_execution_scheduler
            .leading_nmi_upload_pipeline_is_active()
            && (self.game_state.frame.main_module != 7
                || self.game_state.frame.submodule != 0x12
                || self.game_state.frame.subsubmodule > 0x0f)
        {
            self.game_execution_scheduler
                .finish_leading_nmi_upload_pipeline();
        }
        if self
            .spiral_stair_return_oam_publication_host_frame
            .is_some_and(|start| self.frame_ctr_dbg.wrapping_sub(start) > 1)
        {
            self.spiral_stair_return_oam_publication_host_frame = None;
            self.spiral_stair_return_player_oam_scanout = None;
        }
        self.link_obj_dma_completed_this_frame = false;
        self.active_display_force_blank_event = None;
        self.last_sprite_main_timing_workload = None;
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        self.replay_trace_ram_watch("run-frame-entry");
        if CaptureDisplayDiagnostics::from_env().frame_boundary {
            eprintln!(
                "frame_boundary_entry host={} main={:02x} sub={:02x} frame_counter={:02x} work={:?} caller={:?} dialogue_init={} dialogue_scroll={:?} bg1=({:04x},{:04x}) scroll_copy=({:04x},{:04x}) animated_bg=(source={:04x},countdown={:04x})",
                self.frame_ctr_dbg,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                self.game_state.frame.frame_counter,
                self.game_execution_scheduler.current_work(),
                self.game_execution_scheduler.pre_main_caller_continuation(),
                matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(
                        GameWorkContinuation::FinishDialogueInitializationPrefix { .. }
                            | GameWorkContinuation::FinishDialogueInitializationCallerReturn
                    )
                ),
                self.dialogue_scroll_phase(),
                self.ppu.bg_layer[0].h_scroll,
                self.ppu.bg_layer[0].v_scroll,
                self.game_state.display.ppu_scroll_copy.bg1_h_copy(),
                self.game_state.display.ppu_scroll_copy.bg1_v_copy(),
                self.game_state.display.animated_tile_data_source_usize(),
                self.game_state.display.bg_tile_animation_countdown,
            );
        }
        if !self.initialized {
            self.zelda_initialize();
        }
        self.pre_nmi_animated_bg_scanout = self.capture_animated_bg_scanout();
        self.pre_main_graphics_dma = if self.rom_startup_timing() {
            // Capture both the DMA operands and their hardware phase at the
            // host boundary. A main-thread module switch cannot retroactively
            // move a leading NMI behind the CPU work it already preceded.
            let animated_tile = self
                .game_state
                .display
                .has_animated_tile_data_source()
                .then(|| {
                    let source_address = self.game_state.display.animated_tile_data_source_usize();
                    let destination_address = self
                        .game_state
                        .display
                        .animated_tile_vram_destination_usize();
                    (source_address + 0x400 <= self.ram.len()
                        && destination_address + 0x200 <= self.ppu.vram.len())
                    .then(|| PreMainAnimatedTileDma {
                        source_address,
                        destination_address,
                        data: self.ram[source_address..source_address + 0x400].to_vec(),
                    })
                })
                .flatten();
            Some(PreMainGraphicsDma {
                entry_frame: self.game_state.frame,
                entry_plan: rom_graphics_dma_plan_at_host_boundary(self.game_state.frame),
                entry_link_handler_state: self.game_state.player.follower_link.handler_state(),
                animated_tile,
                link_operands: PreMainLinkDmaOperands::capture(&self.ram),
                obj_vram: self.ppu.vram.clone(),
                oam_shadow: self.sprite_oam_shadow_buffer().to_vec(),
            })
        } else {
            None
        };
        // Reuse the OAM shadow captured with the other pre-main DMA operands so
        // NMI consumption and display publication cannot select adjacent copies.
        let oam_dma_source = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.oam_shadow.clone());
        let frame = self.game_state.frame;
        if self.rom_startup_timing
            && rom_intro_poly_thread_is_active(frame.main_module, frame.submodule)
            && self.game_state.display.has_pending_polyhedral_update()
        {
            // The NMI at the frame boundary consumes the buffer completed in a
            // prior CPU slice. A buffer completed below remains pending until
            // the next boundary instead of being uploaded in the same frame.
            self.nmi_poly_upload_from_deferred = true;
            self.nmi_update_irqgfx();
        }
        if self.rom_startup_timing() && self.rom_reset_frame_delay != 0 {
            // Isolate the reset-to-main handoff as a first-class timing
            // boundary. A parity policy can run the boot initialization
            // without executing the first game-loop slice, which is distinct
            // from both a reset-delay frame and a normal frame.
            if self.parity_runtime_nmi_rule_matches("boot_init_only") {
                self.rom_reset_frame_delay = 0;
                self.zelda_initialization_code();
                self.capture_display_snapshot();
                return;
            }
            self.rom_reset_frame_delay = self.rom_reset_frame_delay.saturating_sub(1);
            self.capture_display_snapshot();
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        if let Some(expected_timeline) = terminal_intro_initialization_return_timeline.as_ref() {
            let consumed_timeline = self.take_original_timing_main_loop_return_timeline();
            assert_eq!(
                consumed_timeline.as_ref(),
                Some(expected_timeline),
                "validated terminal intro timeline changed before consumption",
            );
        }
        if let Some(timeline) = terminal_intro_initialization_return_timeline {
            // The trace has reached the exact C caller return which the
            // coarse intro delay used to approximate as another whole host.
            // The CPU work is already represented; retire only that legacy
            // counter before running the source-ordered common suffix/NMI
            // lifecycle owned by the typed timeline.
            self.complete_original_timing_main_loop_return(
                timeline,
                input,
                oam_dma_source.as_deref(),
                |state| {
                    state.intro_initialization_work_frames_pending = 0;
                    state.intro_initialization_reset_obj_control_pending = false;
                },
            );
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        // ReturnOnly and the AfterCurrent dialogue initializer share the same
        // main-loop receipts. Validate the complete ReturnOnly authority first
        // so the scheduler parser below cannot consume a conflicting vector.
        let authoritative_dialogue_return_only =
            self.original_timing_dialogue_return_only_timeline();
        let (
            authoritative_dialogue_after_current_return,
            authoritative_dialogue_after_current_wait,
        ) = {
            let mut scheduler_probe = self.game_execution_scheduler;
            let after_current = scheduler_probe.take_after_current_trailing_nmi();
            let live_dialogue_terminal_caller_is_active =
                matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                    && self.game_execution_scheduler.current_work()
                        == Some(GameWorkContinuation::FinishDialogueInitializationCallerReturn);
            if live_dialogue_terminal_caller_is_active {
                if after_current.is_none() {
                    // A caller tail crossing two or more NMIs is scheduled
                    // work (route host 414033: the crystal poly thread's
                    // V-IRQ steals most of each field). It completes only on
                    // the host whose wire returns to the main wait; earlier
                    // hosts are waiting slices.
                    if self.original_timing_main_loop_return_timeline().is_none() {
                        (None, None)
                    } else {
                        let step = scheduler_probe
                            .advance_work_one_nmi_slice_with_authoritative_completion(true);
                        assert_eq!(
                            step,
                            Some(GameWorkStep::Complete(
                                GameWorkContinuation::FinishDialogueInitializationCallerReturn
                            )),
                            "a scheduled dialogue caller return did not complete on its terminal host",
                        );
                        let terminal = self
                            .take_original_timing_dialogue_terminal_return()
                            .expect(
                                "a live scheduled dialogue caller return requires exact terminal source authority",
                            );
                        (Some((terminal, scheduler_probe)), None)
                    }
                } else {
                    assert_eq!(
                        after_current,
                        Some(GameWorkContinuation::FinishDialogueInitializationCallerReturn),
                        "a live terminal dialogue caller requires the exact AfterCurrent owner",
                    );
                    if self.original_timing_main_loop_return_timeline().is_none() {
                        // The caller-crossing estimate scheduled this as an
                        // AfterCurrent return, but the poly thread can steal
                        // enough of the following field that the ROM is still
                        // inside Text_LoadCharacterBuffer when another Held
                        // NMI arrives (route host 414030). The exact wire is a
                        // nonterminal continued-call slice; retain the same
                        // AfterCurrent owner until a later host publishes the
                        // common-suffix return.
                        let waiting = self
                            .original_timing_nonterminal_continuation_plan()
                            .expect(
                                "a live dialogue AfterCurrent wait requires exact nonterminal source authority",
                            );
                        (None, Some((waiting, scheduler_probe)))
                    } else {
                        let terminal = self
                            .take_original_timing_dialogue_terminal_return()
                            .expect(
                                "a live dialogue AfterCurrent caller return requires exact terminal source authority",
                            );
                        (Some((terminal, scheduler_probe)), None)
                    }
                }
            } else {
                (None, None)
            }
        };
        let initialized_audio_bank_this_frame =
            !self.game_state.display.has_animated_tile_data_source();
        // Ordinary live NMI samples the preceding audio commands before main;
        // measured interrupted callers can instead assign that ownership to
        // the trailing boundary. The exact dialogue terminal lifecycle above
        // is validated first so malformed authority cannot mutate audio ports
        // or initialization state before failing closed.
        self.prepare_audio_nmi_for_main_boundary(frame);
        if initialized_audio_bank_this_frame {
            self.zelda_initialization_code();
        }
        if let Some((waiting, mut scheduler_probe)) = authoritative_dialogue_after_current_wait {
            scheduler_probe.schedule_after_current_trailing_nmi(
                GameWorkContinuation::FinishDialogueInitializationCallerReturn,
            );
            self.game_execution_scheduler = scheduler_probe;
            self.execute_original_timing_nonterminal_continuation(
                waiting,
                input,
                oam_dma_source.as_deref(),
                |_| {},
            );
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        if let Some((timeline, scheduler_probe)) = authoritative_dialogue_after_current_return {
            // Commit the exact AfterCurrent owner only after the complete
            // typed lifecycle has passed read-only validation. The preceding
            // host already accepted the NMI; this host completes its handler,
            // resumes Text_LoadCharacterBuffer's caller, and reaches the one
            // shared main-loop suffix.
            self.game_execution_scheduler = scheduler_probe;
            self.complete_original_timing_main_loop_return(
                timeline,
                input,
                oam_dma_source.as_deref(),
                |state| {
                    state.complete_text_initialization_carry_suffix();
                    state.complete_module0e_interface_after_run();
                    state.next_core_nmi_active_scanout_uses_host_animated_bg_operands =
                        std::mem::take(
                            &mut state
                                .normal_dialogue_following_main_nmi_uses_host_animated_bg_operands,
                        );
                    state.dialogue_live_message_read_position_target = None;
                },
            );
            self.game_execution_scheduler
                .finish_call_stack_at_main_wait_before_nmi();
            let returned_scroll = self.bg_scroll_scanout_from_nmi_register_mirrors();
            self.publish_bg_scroll_for_following_scanout(returned_scroll);
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        if let Some(expected_timeline) = authoritative_dialogue_return_only {
            let timeline = self
                .take_original_timing_main_loop_return_timeline()
                .expect("validated return-only dialogue timeline disappeared");
            assert_eq!(timeline, expected_timeline);
            self.complete_original_timing_main_loop_return(
                timeline,
                input,
                oam_dma_source.as_deref(),
                |state| {
                    state.finish_dialogue_scroll_return();
                    let completed_scanout = state.dialogue_text_scanout_from_render_buffer();
                    state.stage_dialogue_scroll_completion_after_return(completed_scanout);
                },
            );
            self.game_execution_scheduler
                .finish_call_stack_at_main_wait_before_nmi();
            assert!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .is_none_or(|receipts| receipts.semantic().is_empty()),
                "a return-only dialogue slice left typed lifecycle receipts unconsumed",
            );
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        if self.lane_scheduled_finish_sprite_main_boundary(input, oam_dma_source.clone()) {
            return;
        }
        if self.lane_after_current_trailing_nmi_continuation(input, oam_dma_source.clone()) {
            return;
        }
        if self.lane_post_trailing_nmi_continuation(input, oam_dma_source.clone()) {
            return;
        }
        if self.resume_pre_main_caller_continuation(input, oam_dma_source.as_deref()) {
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        if self.rom_startup_timing() && self.dialogue_scroll_is_return_only() {
            assert!(
                !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live),
                "a live return-only dialogue slice cannot fall back to coarse translated timing",
            );
            // The scroll copy and RenderText handler returned after the prior
            // frame's NMI. On this boundary the next NMI sees $12 still
            // latched, so it leaves $17/$0710 pending; only afterward does the
            // caller suffix reach Main_PrepSpritesForNmi and clear $12.
            // This measured return-only slice is distinct from both the 2/3
            // pixel copy slices and from a fresh module iteration.
            self.finish_dialogue_scroll_return();
            // The interrupted NMI cannot consume the pending dialogue upload,
            // but it still advances the ordinary vblank-owned presentation
            // state (animated BG tiles, Link DMA, and OAM). Capture after that
            // NMI so the scanout combines those updates with the still-pending
            // dialogue buffer, exactly as the hardware does.
            // BG scroll is a separate register generation: writes performed by
            // this NMI configure the active frame that follows vblank, even
            // though the pending dialogue-memory upload remains deferred.
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            self.capture_display_snapshot();
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
            // The completed text becomes visible only after the next Open NMI
            // publishes its BG3 DMA receipt. Keep the semantic glyph positions
            // staged with the exact buffer they describe.
            let completed_scanout = self.dialogue_text_scanout_from_render_buffer();
            self.stage_dialogue_scroll_completion_after_return(completed_scanout);
            // The proven source slice resumed RenderText's caller after its
            // carried Held handler and returned to main wait, accepting the
            // Open NMI whose handler begins the next host. Preserve that CPU
            // phase; only the receipt-owned Open handler may expose the staged
            // text and start an adjacent main iteration.
            self.game_execution_scheduler
                .finish_call_stack_at_main_wait_before_nmi();
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        if self.rom_startup_timing() {
            let fallback_selected_game_continuation = self
                .game_execution_scheduler
                .selected_game_load_destination()
                .zip(
                    self.game_execution_scheduler
                        .selected_game_load_after_pre_dungeon_audio_sprite_reset(),
                );
            let (probed_startup_step, startup_scheduler_probe, startup_uses_authority) =
                if let Some((step, scheduler_before_transition, scheduler_after_transition)) =
                    file_select_authoritative_probe
                {
                    assert_eq!(
                        self.game_execution_scheduler, scheduler_before_transition,
                        "validated file-select scheduler owner changed before its source transition",
                    );
                    (Some(step), scheduler_after_transition, true)
                } else if let Some(plan) = selected_game_load_plan.as_ref() {
                    assert_eq!(
                        self.game_execution_scheduler, plan.scheduler_before_transition,
                        "validated selected-game scheduler owner changed before its source transition",
                    );
                    let step = match &plan.action {
                        OriginalTimingSelectedGameLoadAction::Terminal { .. } => {
                            StartupSequenceStep::CompleteSelectedGameLoad
                        }
                        OriginalTimingSelectedGameLoadAction::Nonterminal(_)
                        | OriginalTimingSelectedGameLoadAction::SpriteResetCheckpoint { .. } => {
                            StartupSequenceStep::SelectedGameLoadWaiting
                        }
                    };
                    (Some(step), plan.scheduler_after_transition, true)
                } else if let Some(plan) = begin_selected_game_load_plan.as_ref() {
                    assert_eq!(
                        self.game_execution_scheduler, plan.scheduler_before_transition,
                        "validated pre-dungeon-audio scheduler owner changed before its source transition",
                    );
                    (
                        Some(StartupSequenceStep::BeginPreDungeonAudio),
                        plan.scheduler_after_transition,
                        true,
                    )
                } else if let Some(plan) = selected_game_load_message_interface_plan.as_ref() {
                    assert_eq!(
                        self.game_execution_scheduler, plan.scheduler_before_transition,
                        "validated Message-interface scheduler owner changed before its source transition",
                    );
                    (
                        Some(StartupSequenceStep::SelectedGameLoadWaiting),
                        plan.scheduler_after_transition,
                        true,
                    )
                } else {
                    let mut scheduler_probe = self.game_execution_scheduler;
                    let step = scheduler_probe.advance_startup_sequence();
                    (step, scheduler_probe, false)
                };
            if nonterminal_file_select_plan.is_some() {
                assert_eq!(
                    probed_startup_step,
                    Some(StartupSequenceStep::FileSelectWaiting),
                    "validated nonterminal file-select scheduler owner changed before execution",
                );
            }
            let live_main_loop_return_timeline = self.original_timing_main_loop_return_timeline();
            let terminal_file_select_return_timeline = if matches!(
                self.original_timing_owner,
                OriginalTimingOwnerState::Live
            ) && probed_startup_step
                == Some(StartupSequenceStep::CompleteFileSelectGraphics)
            {
                let Some(timeline) = live_main_loop_return_timeline.as_ref() else {
                    panic!(
                        "live terminal file-select graphics omitted its typed main-loop return timeline",
                    );
                };
                assert_eq!(
                    timeline.progress,
                    crate::MainLoopProgress::CallStackContinued,
                    "the terminal file-select caller cannot begin a fresh main-loop iteration",
                );
                assert_eq!(
                    self.pending_main_loop_common_suffix,
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                    "terminal file-select graphics lost its suspended ZeldaRunGameLoop suffix",
                );
                assert!(
                    self.original_timing_main_loop_interruption().is_none(),
                    "terminal file-select graphics cannot also interrupt the shared main-loop suffix",
                );
                let Some(before_return) = try_classify_original_timing_nmi_phases(
                    self.original_timing_nmi_publication_pending,
                    &timeline.nmi_phases_before_return,
                ) else {
                    panic!(
                        "terminal file-select graphics has an unsupported pre-return NMI sequence: {timeline:?}",
                    );
                };
                assert!(
                    !before_return.publication_pending_at_exit,
                    "terminal file-select graphics cannot return while its pre-return handler is unfinished: {timeline:?}",
                );
                assert_original_timing_carry_in_handler_has_receptive_display(
                    before_return,
                    self.display_snapshot
                        .as_ref()
                        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
                );
                let after_return = try_classify_original_timing_nmi_phases(
                    false,
                    &timeline.nmi_phases_after_return,
                );
                assert!(
                    after_return.is_some(),
                    "terminal file-select graphics has an unsupported post-return NMI sequence: {timeline:?}",
                );
                Some(timeline.clone())
            } else {
                if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                    && probed_startup_step == Some(StartupSequenceStep::FileSelectWaiting)
                {
                    assert!(
                        live_main_loop_return_timeline.is_none(),
                        "nonterminal file-select graphics cannot own a terminal main-loop return timeline",
                    );
                }
                None
            };
            // A typed nonterminal FileSelect slice commits the already-probed
            // scheduler only after its source NMI handler.  Advancing the real
            // scheduler here would reverse the `$e304 -> $8225 -> $e304`
            // handler/resume order and would make malformed authority mutate
            // the 56-slice continuation before validation failed.
            let startup_step = if startup_uses_authority {
                probed_startup_step
            } else {
                self.game_execution_scheduler.advance_startup_sequence()
            };
            let consumes_frame = match startup_step {
                Some(StartupSequenceStep::FileSelectWaiting) => {
                    if let Some(plan) = nonterminal_file_select_plan {
                        self.execute_original_timing_nonterminal_continuation(
                            plan,
                            input,
                            oam_dma_source.as_deref(),
                            |state| {
                                if let Some(expected) = file_select_low_wram_publication {
                                    let publication = state
                                        .take_original_timing_file_select_low_wram_publication()
                                        .expect(
                                            "validated file-select low-WRAM publication disappeared",
                                        );
                                    assert_eq!(publication, expected);
                                    match publication {
                                        OriginalTimingFileSelectLowWramPublication::Progress(
                                            progress,
                                        ) => state
                                            .publish_file_select_graphics_low_wram_clear_progress(
                                                progress,
                                            ),
                                        OriginalTimingFileSelectLowWramPublication::Complete => {
                                            state.publish_file_select_graphics_low_wram_clear()
                                        }
                                    }
                                }
                                state.game_execution_scheduler = startup_scheduler_probe;
                            },
                        );
                        return;
                    }
                    true
                }
                Some(StartupSequenceStep::CompleteFileSelectGraphics) => {
                    if let Some(expected_timeline) = terminal_file_select_return_timeline {
                        let timeline = self
                            .take_original_timing_main_loop_return_timeline()
                            .expect("validated file-select return timeline disappeared");
                        assert_eq!(
                            timeline, expected_timeline,
                            "file-select return timeline changed before consumption",
                        );
                        // The final decompression slice resumes only after the
                        // preceding held handler. It then finishes the module,
                        // returns through NMI_PrepareSprites/$12 clear, and may
                        // accept the next Open handler at main wait. Keep that
                        // source order so the acceptance host captures the
                        // receptive scanout consumed by the following host.
                        self.complete_original_timing_main_loop_return(
                            timeline,
                            input,
                            oam_dma_source.as_deref(),
                            |state| {
                                state.complete_module_select_file_0();
                                state.game_execution_scheduler = startup_scheduler_probe;
                            },
                        );
                        assert_eq!(
                            self.game_execution_scheduler.advance_startup_sequence(),
                            Some(StartupSequenceStep::ResumeFileSelectModule),
                            "terminal file-select graphics lost its caller-resume boundary",
                        );
                        assert!(
                            self.game_execution_scheduler.is_idle(),
                            "the source-proven file-select caller return must retire its scheduler continuation",
                        );
                        self.assert_native_frame_state_matches_ram();
                        self.assert_native_world_location_state_matches_ram();
                        self.assert_native_display_state_matches_ram();
                        return;
                    }
                    self.complete_module_select_file_0();
                    true
                }
                Some(StartupSequenceStep::ResumeFileSelectModule) | None => false,
                Some(StartupSequenceStep::SelectedGameLoadWaiting) => {
                    if let Some(plan) = selected_game_load_message_interface_plan.take() {
                        self.execute_original_timing_nonterminal_continuation(
                            plan.continuation,
                            input,
                            oam_dma_source.as_deref(),
                            |state| {
                                assert!(
                                    state.take_original_timing_selected_game_load_message_interface_published(),
                                    "validated selected-game Message-interface receipt disappeared",
                                );
                                state.publish_selected_game_load_message_interface();
                                state.game_execution_scheduler = plan.scheduler_after_transition;
                            },
                        );
                        return;
                    }
                    if let Some(plan) = selected_game_load_plan.take() {
                        match plan.action {
                            OriginalTimingSelectedGameLoadAction::Nonterminal(continuation) => {
                                self.execute_original_timing_nonterminal_continuation(
                                    continuation,
                                    input,
                                    oam_dma_source.as_deref(),
                                    |state| {
                                        state.original_timing_semantic_receipts.as_mut().unwrap().semantic.retain(|receipt| *receipt != OriginalTimingSemanticReceipt::SelectedGameEntranceReturned);
                                        if plan.completes_entry_room_load {
                                            state.complete_pending_selected_game_load_entry_room_load();
                                        }
                                        state.game_execution_scheduler =
                                            plan.scheduler_after_transition;
                                    },
                                );
                            }
                            OriginalTimingSelectedGameLoadAction::SpriteResetCheckpoint {
                                semantic,
                                timeline,
                                nmi,
                                receipt,
                            } => {
                                assert_eq!(
                                    self.original_timing_semantic_receipts
                                        .as_ref()
                                        .expect("validated selected-game checkpoint disappeared")
                                        .semantic,
                                    semantic,
                                );
                                let consumed = self
                                    .take_original_timing_uninterrupted_main_loop_timeline(
                                        crate::MainLoopProgress::CallStackContinued,
                                    )
                                    .expect(
                                        "validated selected-game checkpoint timeline disappeared",
                                    );
                                assert_eq!(consumed, timeline);
                                self.complete_original_timing_nmi_handler_for_active_scanout(
                                    nmi.handler_completion,
                                    input,
                                    oam_dma_source.as_deref(),
                                )
                                .assert_no_unclaimed_dialogue_text_dma();
                                assert_eq!(
                                    self.take_original_timing_sprite_reset_all_progress(),
                                    Some(receipt),
                                    "validated selected-game Sprite_ResetAll checkpoint disappeared",
                                );
                                if plan.completes_entry_room_load {
                                    self.complete_pending_selected_game_load_entry_room_load();
                                }
                                self.game_execution_scheduler = plan.scheduler_after_transition;
                                self.complete_selected_game_load_through_sprite_disable_all(
                                    plan.destination,
                                );
                                if nmi.publication_pending_at_exit {
                                    self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                                }
                                assert!(
                                    self.original_timing_semantic_receipts
                                        .as_ref()
                                        .is_some_and(|receipts| receipts.semantic.is_empty()),
                                    "selected-game Sprite_ResetAll checkpoint left lifecycle receipts unconsumed",
                                );
                            }
                            OriginalTimingSelectedGameLoadAction::Terminal { .. } => {
                                unreachable!("terminal selected-game plan selected a waiting step")
                            }
                        }
                        return;
                    }
                    if let Some(plan) = pre_audio_selected_game_waiting_plan {
                        self.execute_original_timing_nonterminal_continuation(
                            plan,
                            input,
                            oam_dma_source.as_deref(),
                            |_| {},
                        );
                        return;
                    }
                    true
                }
                Some(StartupSequenceStep::BeginPreDungeonAudio) => {
                    if let Some(plan) = begin_selected_game_load_plan.take() {
                        assert_eq!(
                            self.original_timing_semantic_receipts
                                .as_ref()
                                .expect("validated pre-dungeon-audio authority disappeared")
                                .semantic,
                            plan.semantic,
                            "pre-dungeon-audio authority changed before consumption",
                        );
                        self.original_timing_semantic_receipts.as_mut().unwrap().semantic.retain(|receipt| {
                            !matches!(receipt, OriginalTimingSemanticReceipt::SelectedGameEntranceScrollPublished | OriginalTimingSemanticReceipt::SelectedGameEntranceBeforeSelection | OriginalTimingSemanticReceipt::SelectedGameEntranceReturned)
                        });
                        let timeline = self
                            .take_original_timing_uninterrupted_main_loop_timeline(
                                crate::MainLoopProgress::CallStackContinued,
                            )
                            .expect("validated pre-dungeon-audio timeline disappeared");
                        assert_eq!(
                            timeline, plan.timeline,
                            "pre-dungeon-audio timeline changed before consumption",
                        );
                        // The source accepts and completes the first held NMI
                        // inside Decompression_GetNextByte. Only after its
                        // handler returns does Module_PreDungeon publish the
                        // new audio command and enter Dungeon_LoadEntrance.
                        self.complete_original_timing_nmi_handler_for_active_scanout(
                            plan.nmi.handler_completion,
                            input,
                            oam_dma_source.as_deref(),
                        )
                        .assert_no_unclaimed_dialogue_text_dma();
                        self.begin_selected_game_load_pre_dungeon_audio(plan.destination);
                        self.game_execution_scheduler = startup_scheduler_probe;
                        if plan.entrance_scroll_published {
                            self.begin_selected_game_entrance_scroll_prefix();
                        }
                        if plan.entrance_before_selection {
                            self.begin_selected_game_entrance_before_selection();
                        }
                        if plan.entrance_returned {
                            self.complete_pending_selected_game_load_entry_room_load();
                        }
                        // The room-load then accepts another held NMI at the
                        // source host return. Preserve this post-CPU scanout
                        // as the receptive owner for the next host's handler.
                        // A boundary host whose one NMI completed in-host
                        // (route host 638711) carries nothing.
                        if plan.nmi.publication_pending_at_exit {
                            self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                        }
                        self.assert_native_frame_state_matches_ram();
                        self.assert_native_world_location_state_matches_ram();
                        self.assert_native_display_state_matches_ram();
                        return;
                    }
                    let destination = self
                        .game_execution_scheduler
                        .selected_game_load_destination()
                        .expect(
                            "pre-dungeon-audio boundary lost its frozen selected-game destination",
                        );
                    self.begin_selected_game_load_pre_dungeon_audio(destination);
                    // Run the ROM's entry room-load (Dungeon_LoadEntrance, incl.
                    // DUNGEON_ROOM) on this same frame instead of collapsing it onto
                    // the completion boundary ~57 slices later (bug class 5). The room
                    // DRAW stays at CompleteSelectedGameLoad.
                    self.complete_pending_selected_game_load_entry_room_load();
                    true
                }
                Some(StartupSequenceStep::CompleteSelectedGameLoad) => {
                    if let Some(plan) = selected_game_load_plan.take() {
                        let OriginalTimingSelectedGameLoadAction::Terminal {
                            semantic,
                            timeline: expected_timeline,
                            sprite_reset,
                        } = plan.action
                        else {
                            unreachable!("nonterminal selected-game plan selected a terminal step")
                        };
                        assert_eq!(
                            self.original_timing_semantic_receipts
                                .as_ref()
                                .expect("validated selected-game terminal authority disappeared")
                                .semantic,
                            semantic,
                        );
                        let timeline = self
                            .take_original_timing_main_loop_return_timeline()
                            .expect("validated selected-game return timeline disappeared");
                        assert_eq!(
                            timeline, expected_timeline,
                            "selected-game return timeline changed before consumption",
                        );
                        // Source host 2292 first completes the held NMI accepted
                        // by host 2291, then resumes the selected-game loader,
                        // and only after that caller returns does the shared
                        // ZeldaRunGameLoop suffix run NMI_PrepareSprites and
                        // clear $12. Keep that ordering instead of letting the
                        // coarse terminal slice clear the latch before the
                        // carried handler validates its disposition.
                        self.complete_original_timing_main_loop_return(
                            timeline,
                            input,
                            oam_dma_source.as_deref(),
                            |state| {
                                if plan.completes_entry_room_load {
                                    state.complete_pending_selected_game_load_entry_room_load();
                                }
                                if plan.message_interface_published {
                                    assert_eq!(
                                        plan.destination,
                                        SelectedGameLoadDestination::Message,
                                    );
                                    state.complete_selected_game_load_message_after_interface_publication();
                                } else {
                                    state.complete_selected_game_load_from_frozen_continuation(
                                        plan.destination,
                                        sprite_reset,
                                    );
                                }
                                state.game_execution_scheduler = plan.scheduler_after_transition;
                            },
                        );
                        self.game_execution_scheduler
                            .finish_call_stack_at_main_wait_before_nmi();
                        assert!(
                            self.game_execution_scheduler.is_idle(),
                            "the source-proven selected-game return must retire its scheduler continuation",
                        );
                        if plan.destination == SelectedGameLoadDestination::Dungeon {
                            // The Module_PreDungeon publication ran inside the
                            // completion above.
                            let _ = self.take_original_timing_pre_dungeon_module_returned();
                        }
                        if plan.destination == SelectedGameLoadDestination::Message {
                            assert!(
                                self.take_original_timing_dialogue_closed(),
                                "the Message selected-game return lost its DialogueClosed fact",
                            );
                        }
                        // A leading held acceptance may restate the already
                        // consumed Sprite_ResetAll checkpoint (route host 618346).
                        let _ = self.take_original_timing_sprite_reset_all_progress();
                        assert!(
                            self.original_timing_semantic_receipts
                                .as_ref()
                                .is_some_and(|receipts| receipts.semantic.is_empty()),
                            "selected-game terminal return left unowned semantic receipts",
                        );
                        self.assert_native_frame_state_matches_ram();
                        self.assert_native_world_location_state_matches_ram();
                        self.assert_native_display_state_matches_ram();
                        return;
                    }
                    let (destination, sprite_reset) = fallback_selected_game_continuation.expect(
                        "selected-game compatibility completion lost its frozen continuation",
                    );
                    self.complete_selected_game_load_from_frozen_continuation(
                        destination,
                        sprite_reset,
                    );
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    // The selected-game loader has returned to the main wait,
                    // and the NMI below completes before the freshly loaded
                    // module begins on the next host callback. Preserve that
                    // CPU/NMI ordering so synchronous work started by the
                    // first module iteration does not claim this already-
                    // consumed boundary as its trailing NMI.
                    self.game_execution_scheduler
                        .mark_main_iteration_after_leading_nmi();
                    if self.game_state.frame.main_module == 7 {
                        let advance = dungeon_module_7_cpu_advance_after_leading_nmi(
                            self,
                            DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT,
                        );
                        if crate::debug_env::var_os("ZELDA3_DEBUG_DUNGEON_CPU_SCHEDULE").is_some() {
                            eprintln!(
                                "selected_load_return_cpu host={} state={:02x}/{:02x}/{:02x} phase={:?} resumed={:?} subsubmodule={:02x} countdown={}",
                                self.frame_ctr_dbg,
                                self.game_state.frame.main_module,
                                self.game_state.frame.submodule,
                                self.game_state.frame.subsubmodule,
                                advance.phase,
                                advance.resumed_phase,
                                advance.subsubmodule,
                                advance.palette_countdown,
                            );
                        }
                        self.dungeon_landing_spotlight_reset_prefix_scanlines = None;
                        self.dungeon_landing_cpu_advance_pending = Some(advance);
                    }
                    true
                }
            };
            if consumes_frame {
                // The crystal maiden's poly thread owns this host's CPU while
                // the main thread's Sprite_Main stays suspended (route hosts
                // 413552/413554): the thread still renders during it.
                if run_what & crate::RUN_POLY != 0 && !dungeon_poly_thread_host {
                    self.zelda_run_poly_loop();
                }
                self.capture_display_snapshot();
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                return;
            }
        }
        if run_what & crate::RUN_POLY != 0 && !dungeon_poly_thread_host {
            self.zelda_run_poly_loop();
        }
        if self.intro_zelda_fade_transition_pending {
            self.complete_intro_zelda_fade_transition();
        }
        if self.intro_title_fade_suffix_pending {
            self.complete_intro_zelda_fade_suffix(true);
        }
        if self.intro_bg_fade_suffix_pending {
            self.complete_intro_bg_fade_suffix();
        }
        if self.lane_intro_initialization_work_frames(input, oam_dma_source.clone()) {
            return;
        }
        if let Some(plan) = intro_memory_darken_plan {
            match plan {
                OriginalTimingIntroMemoryDarkenPlan::Nonterminal(plan) => {
                    // The counter is a translated sentinel, not an estimate of
                    // how many source host calls remain.  The exact terminal
                    // suffix below is the only authority which retires it.
                    self.execute_original_timing_nonterminal_continuation(
                        plan,
                        input,
                        oam_dma_source.as_deref(),
                        |_| {},
                    );
                }
                OriginalTimingIntroMemoryDarkenPlan::Terminal(expected) => {
                    let timeline = self
                        .take_original_timing_main_loop_return_timeline()
                        .expect("validated terminal intro memory timeline disappeared");
                    assert_eq!(timeline, expected);
                    self.complete_original_timing_main_loop_return(
                        timeline,
                        input,
                        oam_dma_source.as_deref(),
                        |state| {
                            state.intro_initialize_memory_darken_finish();
                            state.intro_memory_darken_frame_delay = 0;
                        },
                    );
                    assert_eq!(
                        self.game_state.display.bg_tile_animation_countdown, 1,
                        "the source seed 2 must be decremented exactly once by the terminal common suffix",
                    );
                }
            }
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        if self.lane_save_quit_reset_plan(input, oam_dma_source.clone(), save_quit_reset_plan) {
            return;
        }
        if self.rom_startup_timing()
            && self.intro_memory_darken_frame_delay != 0
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            let frame = &self.game_state.frame;
            if frame.main_module == 0 && frame.submodule == 3 {
                self.intro_animate_triforce();
            }
            self.intro_memory_darken_frame_delay =
                self.intro_memory_darken_frame_delay.saturating_sub(1);
            if self.intro_memory_darken_frame_delay == 0 {
                self.intro_initialize_memory_darken_finish();
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        if self.lane_intro_poly_plan(input, intro_poly_plan, oam_dma_source.clone()) {
            return;
        }
        if self.rom_startup_timing()
            && rom_intro_poly_initialization_is_active(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            )
            && self.intro_poly_thread_initialization_phase != 0
            && run_what & crate::RUN_MAIN != 0
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            let (begin_main_loop, complete_module, next_phase) =
                rom_intro_poly_init_decision(self.intro_poly_thread_initialization_phase);
            self.intro_poly_thread_initialization_phase = next_phase;
            if begin_main_loop {
                // This is the same ZeldaRunGameLoop entry edge as the generic
                // path below. A prior iteration's completed sprite-prep pass
                // cannot satisfy the newly suspended caller's suffix.
                self.main_loop_sprite_preparation_completed = false;
                assert!(
                    self.pending_main_loop_common_suffix.is_none(),
                    "intro poly caller cannot begin over another suspended ZeldaRunGameLoop suffix",
                );
                self.pending_main_loop_common_suffix =
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
                self.increment_frame_counter();
                self.clear_oam_buffer();
            }
            if complete_module {
                self.intro_initialize_triforce_poly_thread();
                self.complete_pending_main_loop_common_suffix_after_module_return();
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        if self.rom_startup_timing()
            && self.game_state.frame.main_module == 20
            && self.attract_init_graphics_phase != 0
        {
            let (complete_graphics, next_phase) =
                rom_attract_init_graphics_decision(self.attract_init_graphics_phase);
            self.attract_init_graphics_phase = next_phase;
            if complete_graphics {
                self.complete_attract_init_graphics();
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        if self.rom_startup_timing()
            && self.game_state.frame.main_module == 20
            && self.attract_first_story_render_delay != 0
        {
            let opening_polka_dots = self.game_state.ending.attract_scene.sequence() == 0;
            if (opening_polka_dots && self.attract_first_story_render_delay == 7)
                || (!opening_polka_dots && self.attract_first_story_render_delay == 6)
            {
                self.increment_frame_counter();
                self.clear_oam_buffer();
                if opening_polka_dots {
                    self.ResetHUDPalettes4and5();
                }
            } else if opening_polka_dots && self.attract_first_story_render_delay == 3 {
                self.complete_text_initialization_state_prefix();
            } else if opening_polka_dots && self.attract_first_story_render_delay == 2 {
                self.Attract_DecompressStoryGFX();
                self.complete_text_initialization_suffix();
                self.attract_build_legend_image_tile_map(0);
                // The ROM resumes the first polka-dot story tick on this
                // GFX-completion boundary. Its visible text work is deferred
                // to the next slice, but the scene lifetime counter is not.
                self.attract_scene_mut().decrement_legend_ctr();
                self.clear_nmi_update_latch();
            } else if self.attract_first_story_render_delay == 1 {
                self.attract_enact_story();
                if !opening_polka_dots {
                    self.nmi_prepare_sprites();
                }
                self.clear_nmi_update_latch();
            }
            self.attract_first_story_render_delay =
                self.attract_first_story_render_delay.saturating_sub(1);
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return;
        }
        let scheduled_work_entry_scroll = BgScrollRegisterScanout::capture(&self.ppu);
        let scheduled_work_started_after_leading_nmi = self
            .game_execution_scheduler
            .current_scheduled_work_started_after_leading_nmi();
        let scheduled_work_audio_follows_host_publication =
            self.resumed_dungeon_caller_audio_follows_host_publication();
        let dialogue_initialization_prefix_pending = matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDialogueInitializationPrefix { .. })
        );
        if dialogue_initialization_prefix_pending {
            self.stage_dialogue_initialization_obj_scanout();
        }
        let authoritative_item_receipt_is_active = matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishItemReceiptGraphics {
                continuation: ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt { .. }
                    | ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. }
                    | ItemReceiptGraphicsContinuation::ResumeAncillaItemReceipt { .. },
            })
        ) && matches!(
            self.original_timing_owner,
            OriginalTimingOwnerState::Live
        );
        let authoritative_item_receipt_returned =
            self.compute_item_receipt_returned(authoritative_item_receipt_is_active);
        let sprite_reset_all_work = self.game_execution_scheduler.current_work();
        if matches!(
            sprite_reset_all_work,
            Some(GameWorkContinuation::FinishGameOverDeathAfterSpriteReset { .. })
        ) {
            assert!(matches!(
                self.original_timing_owner,
                OriginalTimingOwnerState::Live
            ));
            // The accepting NMI may restate the completed disable in the same
            // host that returns from ResetAll and Death_Func15. Consume it
            // while its caller is still scheduled, before return retirement.
            if let Some(receipt) = self.take_original_timing_sprite_reset_all_progress() {
                assert_eq!(
                    receipt.progress,
                    crate::SpriteResetAllProgress::SpriteDisableAllCompleted
                );
            }
        }
        let flute_menu_sprite_reset_is_active = matches!(
            sprite_reset_all_work,
            Some(GameWorkContinuation::FinishFluteMenuSelectedScreen {
                step: FluteMenuSelectedScreenStep::InitialSpriteReset
                    | FluteMenuSelectedScreenStep::OverworldReloadReset,
            })
        );
        let pre_overworld_sprite_reset_is_active = matches!(
            sprite_reset_all_work,
            Some(GameWorkContinuation::PreOverworldPropertiesSpriteReset { .. })
        );
        let authoritative_sprite_reset_all_progress =
            (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && (flute_menu_sprite_reset_is_active || pre_overworld_sprite_reset_is_active))
                .then(|| self.take_original_timing_sprite_reset_all_progress())
                .flatten();
        if flute_menu_sprite_reset_is_active {
            if let Some(receipt) = authoritative_sprite_reset_all_progress {
                self.apply_flute_menu_sprite_reset_progress(receipt);
            }
        }
        if let Some(receipt) =
            authoritative_sprite_reset_all_progress.filter(|_| pre_overworld_sprite_reset_is_active)
        {
            assert_eq!(
                receipt.progress,
                SpriteResetAllProgress::SpriteDisableAllCompleted,
                "pre-overworld reset authority named an unsupported source checkpoint",
            );
            assert!(
                matches!(
                    receipt.boundary,
                    crate::OriginalTimingBoundary::HostReturn
                        | crate::OriginalTimingBoundary::NmiAccepted
                ),
                "pre-overworld reset authority requires an observable host boundary",
            );
        }
        let authoritative_overworld_sprite_progress =
            self.take_original_timing_overworld_sprite_reload_progress();
        let authoritative_overworld_sprite_reload_returned = self
            .apply_original_timing_overworld_sprite_reload_progress(
                authoritative_overworld_sprite_progress,
            );
        let authoritative_overworld_sprite_reload_is_active =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(
                        GameWorkContinuation::FinishOverworldSpriteReloadTail { .. }
                            | GameWorkContinuation::FinishOverworldLoadOverlaysSpriteReload
                            | GameWorkContinuation::FinishModule09LongLoad {
                                step: Module09LongLoadStep::MirrorWarpSpriteLoadReload
                                    | Module09LongLoadStep::Module15MirrorWarpSpriteLoadReload
                                    | Module09LongLoadStep::LoadOverlays2,
                            }
                    )
                );
        let authoritative_overworld_load_overlays_overlay_is_active =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishOverworldLoadOverlaysOverlay)
                        | Some(GameWorkContinuation::FinishModule09LongLoad {
                            step: Module09LongLoadStep::LoadOverlays2Overlay,
                        })
                );
        let authoritative_overworld_map_quadrants_published =
            self.take_original_timing_overworld_map_quadrants_published();
        let mut authoritative_link_oam_equipment_prefix = None;
        // A fresh dungeon iteration consumes its drawing prefix in the
        // concrete Sprite_Main caller. Only a scheduled predecessor owns the
        // early extraction used by the long room-loading continuation below.
        if matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonSupertileTransition { .. })
        ) {
            if let Some(receipts) = self.original_timing_semantic_receipts.as_mut() {
                receipts.semantic.retain(|receipt| {
                    if let OriginalTimingSemanticReceipt::LinkOamStairProgress(progress) = *receipt
                    {
                        assert!(
                            authoritative_link_oam_equipment_prefix.is_none(),
                            "LinkOam prefix replayed"
                        );
                        authoritative_link_oam_equipment_prefix = Some(progress);
                        false
                    } else {
                        true
                    }
                });
            }
        }
        if authoritative_link_oam_equipment_prefix.is_some() {
            assert!(
                matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                        work: DungeonSupertileTransitionWork::StraightInterroomRoomInitialization
                            | DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics
                            | DungeonSupertileTransitionWork::StraightInterroomBgCharacters34
                            | DungeonSupertileTransitionWork::SpiralRoomInitialization
                            | DungeonSupertileTransitionWork::SpiralBackgroundSync
                    })
                ),
                "LinkOam equipment prefix reached an unsupported caller"
            );
        }
        // A fresh Module0B/$24 caller consumes its own source checkpoints.
        // Only an already suspended caller transfers them to this scheduler.
        let special_exit_is_scheduled = self.game_execution_scheduler.current_work().is_some();
        let authoritative_overworld_special_exit_mosaic_restored = special_exit_is_scheduled
            && self.take_original_timing_overworld_special_exit_mosaic_restored();
        let authoritative_overworld_special_exit_mosaic_returned = special_exit_is_scheduled
            && self.take_original_timing_overworld_special_exit_mosaic_returned();
        if authoritative_overworld_special_exit_mosaic_restored {
            assert_eq!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishOverworldSpecialExitMosaic),
                "special-exit mosaic restore receipt reached the wrong native source stage",
            );
        }
        if authoritative_overworld_special_exit_mosaic_returned {
            assert!(
                matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(
                        GameWorkContinuation::FinishOverworldSpecialExitMosaic
                            | GameWorkContinuation::FinishOverworldSpecialExitMosaicSecondDecode
                    )
                ),
                "special-exit mosaic return receipt reached the wrong native source stage",
            );
        }
        if let Some(progress) = self.take_original_timing_dungeon_falling_entrance_progress() {
            self.apply_original_timing_dungeon_falling_entrance_progress(progress);
        }
        if let Some(GameWorkContinuation::FinishRescuedMaidenTilemapClear { completed_stores }) =
            self.game_execution_scheduler.current_work()
        {
            if let Some(progress) =
                self.take_original_timing_rescued_maiden_tilemap_clear_progress()
            {
                assert!(
                    progress.completed_stores >= completed_stores,
                    "rescued-maiden clear progress moved backward: {completed_stores} -> {}",
                    progress.completed_stores,
                );
                self.apply_rescued_maiden_tilemap_clear_stores(
                    completed_stores,
                    progress.completed_stores,
                );
                self.game_execution_scheduler.refine_scheduled_work(
                    GameWorkContinuation::FinishRescuedMaidenTilemapClear { completed_stores },
                    GameWorkContinuation::FinishRescuedMaidenTilemapClear {
                        completed_stores: progress.completed_stores,
                    },
                );
            }
        }
        if let Some(GameWorkContinuation::FinishRescuedMaidenInitialization { stage }) =
            self.game_execution_scheduler.current_work()
        {
            if let Some(progress) =
                self.take_original_timing_rescued_maiden_initialization_progress()
            {
                assert_eq!(
                    progress.boundary,
                    OriginalTimingBoundary::HostReturn,
                    "rescued-maiden initialization progress requires a source host return",
                );
                self.apply_follower_graphics_progress(Some(stage), progress.stage);
                self.game_execution_scheduler.refine_scheduled_work(
                    GameWorkContinuation::FinishRescuedMaidenInitialization { stage },
                    GameWorkContinuation::FinishRescuedMaidenInitialization {
                        stage: progress.stage,
                    },
                );
            }
        }
        let authoritative_overworld_map_quadrants_is_active =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishOverworldMapQuadrants { .. })
                );
        let authoritative_world_map_overlay_reload_is_active =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishWorldMapOverlayReload)
                );
        let authoritative_world_map_overlay_reload_returned =
            authoritative_world_map_overlay_reload_is_active
                // A terminal-shaped host is owned by the scheduled return
                // lane, which validates and consumes the module-return token
                // itself (route host 61935).
                && self.original_timing_main_loop_return_timeline().is_none()
                && self.take_original_timing_world_map_overlay_reload_returned();
        let authoritative_world_map_ambient_map8_is_active =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishWorldMapAmbientMap8)
                );
        let authoritative_world_map_ambient_map8_returned =
            authoritative_world_map_ambient_map8_is_active
                && self.take_original_timing_world_map_ambient_map8_returned();
        let authoritative_overworld_spotlight_goal_is_active =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(
                        GameWorkContinuation::FinishOverworldSpotlightBuild { .. }
                            | GameWorkContinuation::FinishOverworldSpotlightGoalResetTable { .. }
                    )
                );
        let authoritative_overworld_spotlight_goal_returned =
            authoritative_overworld_spotlight_goal_is_active
                && self.take_original_timing_overworld_spotlight_goal_caller_returned();
        let authoritative_dungeon_exit_spotlight_entry_is_active =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. })
                );
        let authoritative_dungeon_exit_spotlight_entry_returned = self
            .take_or_defer_original_timing_dungeon_exit_spotlight_entry_returned(
                authoritative_dungeon_exit_spotlight_entry_is_active,
            );
        let authoritative_dungeon_exit_spotlight_entry_iteration_returned =
            authoritative_dungeon_exit_spotlight_entry_is_active
                && self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .is_some_and(|receipts| {
                        receipts
                            .semantic()
                            .contains(&OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted)
                    });
        // A suspended scheduled caller owns whichever common suffix phase the
        // source reached before returning this host. Without such a caller,
        // SpritePreparation belongs to the translated module's own
        if let Some(plan) = prospective_spotlight_build_link_oam_plan.as_ref() {
            assert_eq!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .expect("spotlight Build-LinkOam preflight lost its semantic authority")
                    .semantic(),
                plan.semantic,
                "spotlight Build-LinkOam semantic authority changed after immutable preflight",
            );
            assert_eq!(
                self.game_execution_scheduler, plan.scheduler_before_completion,
                "spotlight Build-LinkOam scheduler changed after immutable preflight",
            );
        }
        // NMI_PrepareSprites continuation and must remain on the receipt bus;
        // only LinkOam is classified by this outer host timeline.
        let idle_outer_interruption = self
            .game_execution_scheduler
            .current_work()
            .is_none()
            .then(|| self.original_timing_main_loop_interruption())
            .flatten()
            .filter(|interruption| {
                *interruption == crate::MainLoopInterruption::LinkOam
                    || matches!(
                        interruption,
                        crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }
                            | crate::MainLoopInterruption::DungeonExitSpotlightAfterSubmodule
                            | crate::MainLoopInterruption::LinkActualVelocity { .. }
                            | crate::MainLoopInterruption::LinkActualVelocityCompleted
                            | crate::MainLoopInterruption::LinkVelocityClearProgress { .. }
                            | crate::MainLoopInterruption::DungeonExitSpotlightTableCompleted
                            | crate::MainLoopInterruption::LinkPositionBeforeCoordinates
                            | crate::MainLoopInterruption::LinkPositionAfterSubpixel { .. }
                            | crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { .. }
                            | crate::MainLoopInterruption::LinkPositionAfterCoordinates { .. }
                            | crate::MainLoopInterruption::GameOverIrisGoalPaletteFill { .. }
                            | crate::MainLoopInterruption::SpotlightGoalResetTable { .. }
                            | crate::MainLoopInterruption::DesertPrayerIris { .. }
                            | crate::MainLoopInterruption::DesertPrayerPaletteFilterBeforeColor { .. }
                    )
                    || interruption.is_sprite_main()
            })
            // Preserve the previous fail-closed behavior for interruption
            // kinds owned directly by the module body rather than this outer
            // host timeline.
            .or_else(|| {
                self.game_execution_scheduler
                    .current_work()
                    .is_none()
                    .then_some(crate::MainLoopInterruption::LinkOam)
            });
        if let Some(plan) = prospective_terminal_ground_item_receipt_plan.as_ref() {
            assert_eq!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .expect("terminal ground-item preflight lost its semantic authority")
                    .semantic(),
                plan.semantic,
                "terminal ground-item semantic authority changed after immutable preflight",
            );
            assert_eq!(
                self.game_execution_scheduler, plan.scheduler_before_completion,
                "terminal ground-item scheduler changed after immutable preflight",
            );
        }
        if self.lane_terminal_spotlight_iteration_plan(
            input,
            oam_dma_source.clone(),
            prospective_terminal_spotlight_iteration_plan,
        ) {
            return;
        }
        if let Some(plan) = prospective_interrupted_idle_main_loop_plan.as_ref() {
            assert_eq!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .expect("interrupted idle-main preflight lost its semantic authority")
                    .semantic(),
                plan.semantic,
                "interrupted idle-main semantic authority changed after immutable preflight",
            );
            assert_eq!(
                self.game_execution_scheduler.pre_main_nmi_resume(),
                plan.pre_main_timing_shadow,
                "interrupted idle-main pre-main timing-shadow ownership changed after immutable preflight",
            );
            if let Some(predecessor) = plan.scheduled_predecessor {
                assert_eq!(
                    self.game_execution_scheduler, predecessor.scheduler_before_step,
                    "scheduled caller-to-fresh-iteration ownership changed after immutable preflight",
                );
            } else {
                assert_eq!(
                    self.game_execution_scheduler.current_work(),
                    None,
                    "an idle interrupted iteration gained a scheduled predecessor after immutable preflight",
                );
            }
        }
        // Validate cursor-to-NMI adjacency before the timeline consumes those phases.
        let authoritative_dungeon_peg_flip_progress = self.preflight_continued_peg_attribute_flip();
        let authoritative_main_loop_interruption_timeline =
            self.take_original_timing_main_loop_interruption_timeline(idle_outer_interruption);
        if let Some(timeline) = authoritative_main_loop_interruption_timeline.as_ref() {
            // A scheduled tail load whose caller suffix was interrupted inside
            // Sprite_Main publishes the boundary twice: as the interruption
            // and as a Sprite_Main progress claim. The interruption drives
            // execution; the claim corroborates the same C statement.
            if timeline.interruption.is_sprite_main()
                && self
                    .game_execution_scheduler
                    .current_work()
                    .is_some_and(|work| {
                        work.consumes_uninterrupted_scheduled_caller_timeline()
                            // The spiral/sprite-conversion callers' own
                            // Module07 Sprite_Main can be re-interrupted
                            // (slot 1 at route host 95850).
                            || matches!(
                                work,
                                GameWorkContinuation::FinishDungeonSupertileTransition { .. }
                            )
                    })
            {
                if let Some(claimed) = self.take_original_timing_sprite_main_progress() {
                    assert_eq!(
                        Some(claimed),
                        sprite_main_cpu_boundary_from_interruption(timeline.interruption),
                        "a scheduled tail's Sprite_Main claim disagrees with its interruption boundary",
                    );
                }
            }
            if let crate::MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(_) =
                timeline.interruption
            {
                assert!(
                    self.original_timing_sprite_main_progress_boundary()
                        .is_none(),
                    "a caller-specific item-receipt interruption duplicated an earlier Sprite_Main checkpoint",
                );
            }
        }
        if let Some(plan) = prospective_interrupted_idle_main_loop_plan.as_ref() {
            assert_eq!(
                authoritative_main_loop_interruption_timeline.as_ref(),
                Some(&plan.timeline),
                "the interrupted idle-main timeline changed after immutable preflight",
            );
        }
        let authoritative_link_oam_timeline = authoritative_main_loop_interruption_timeline
            .as_ref()
            .filter(|timeline| {
                timeline.interruption == crate::MainLoopInterruption::LinkOam
                    // The recurring spotlight Build may instead end inside
                    // Link_MovePosition's axis loop (route host 179586); the
                    // Build plan owns that boundary through the same lane.
                    || prospective_spotlight_build_link_oam_plan
                        .as_ref()
                        .is_some_and(|plan| plan.interruption == timeline.interruption)
            });
        if let Some(plan) = prospective_spotlight_build_link_oam_plan.as_ref() {
            assert_eq!(
                authoritative_link_oam_timeline,
                Some(&plan.timeline),
                "spotlight Build-LinkOam timeline changed after immutable preflight",
            );
        }
        let authoritative_scheduled_caller_nmi_timeline =
            (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.game_execution_scheduler.current_work().is_some())
            .then_some(authoritative_main_loop_interruption_timeline.as_ref())
            .flatten();
        let authoritative_scheduled_caller_return_timeline = self
            .compute_scheduled_caller_return_timeline(
                &authoritative_main_loop_interruption_timeline,
            );
        let authoritative_uninterrupted_idle_continued_return_plan =
            prospective_uninterrupted_idle_continued_return_plan;
        let authoritative_uninterrupted_idle_main_loop_timeline =
            prospective_uninterrupted_idle_main_loop_plan;
        let authoritative_supertile_sprite_main_returned = if matches!(
            self.original_timing_owner,
            OriginalTimingOwnerState::Live
        )
            && matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                    work:
                        DungeonSupertileTransitionWork::AuxiliarySpriteGraphics
                            | DungeonSupertileTransitionWork::SpriteConversion
                            | DungeonSupertileTransitionWork::SpiralRoomInitialization
                            | DungeonSupertileTransitionWork::SpiralBackgroundSync
                            | DungeonSupertileTransitionWork::SpiralSpriteGraphics,
                })
            )
            && authoritative_scheduled_caller_return_timeline.is_none()
            // A fresh iteration's own body consumes its Sprite_Main claim
            // through the iteration-plan claim scope; only a Continued host
            // (whose progress receipt an earlier consumer already retired)
            // leaves this corroborative claim to the schedule owner.
            && self.original_timing_main_loop_progress()
                != Some(crate::MainLoopProgress::IterationStarted)
        {
            // The resumed room-load caller runs its one Sprite_Main between
            // NMI slices while a supertile-transition stage is still
            // scheduled; the native model executes that same body inside the
            // schedule-owned completion. The claim corroborates the source
            // statement this Continued host reached.
            // Entry and mid-slot checkpoints belong to the native caller
            // which refines its schedule below. Even BeforeFirstSlot proves
            // its preceding graphics/reset calls have returned; consuming it
            // here would strand that caller inside the preceding operation.
            self.take_original_timing_sprite_main_returned()
        } else {
            false
        };
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && matches!(
                self.game_execution_scheduler.current_work(),
                Some(
                    GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { .. }
                        | GameWorkContinuation::FinishPreDungeonEntranceLoad {
                            sprite_reset:
                                PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted
                                    | PreDungeonSpriteResetContinuation::DungeonResetSprites(_)
                                    | PreDungeonSpriteResetContinuation::SpriteResetAllCompleted
                                    | PreDungeonSpriteResetContinuation::DungeonResetSpritesCompleted,
                        }
                )
            )
        {
            // Sprite_ResetAll can be restated at the interrupting acceptance
            // after its host-return checkpoint. Dungeon_ResetSprites is not
            // a restatement here: the pre-dungeon continuation consumes its
            // exact room-load cursor below.
            if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { .. })
            ) {
                let _ = self.take_original_timing_dungeon_reset_sprites_progress();
            }
            if let Some(receipt) = self.take_original_timing_sprite_reset_all_progress() {
                assert_eq!(
                    receipt.progress,
                    crate::SpriteResetAllProgress::SpriteDisableAllCompleted,
                    "a resumed caller's reset-all restatement changed its completed statement",
                );
            }
        }
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishSpotlightIteration { .. })
            )
        {
            // The recurring build's in-flight state can be re-checkpointed at
            // an interrupting acceptance while its deferred iteration return
            // is still scheduled; the saved continuation carries the same
            // statement (route host 56934).
            if let Some(claim) = self.take_original_timing_spotlight_table_build_progress() {
                assert_eq!(
                    claim.boundary,
                    OriginalTimingBoundary::NmiAccepted,
                    "a deferred spotlight iteration's build re-checkpoint must ride an accepting NMI",
                );
            }
        }
        let authoritative_uninterrupted_scheduled_caller_nmi_timeline =
            (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && prospective_terminal_ground_item_receipt_plan.is_none()
                && self.game_execution_scheduler.current_work().is_some()
                && authoritative_scheduled_caller_nmi_timeline.is_none()
                && authoritative_scheduled_caller_return_timeline.is_none()
                // A host that proves the suspended caller returned to main
                // wait is a terminal boundary; only its return-timeline owner
                // may consume it.
                && self.original_timing_main_loop_return_timeline().is_none()
                && (self
                    .game_execution_scheduler
                    .work_suspends_translated_call_stack()
                    || self
                        .game_execution_scheduler
                        .current_work()
                        .is_some_and(|work| {
                            work.consumes_uninterrupted_scheduled_caller_timeline()
                        })))
            .then(|| {
                let progress = self
                    .original_timing_semantic_receipts
                    .as_ref()?
                    .semantic()
                    .iter()
                    .find_map(|receipt| match receipt {
                        OriginalTimingSemanticReceipt::MainLoopProgress(progress) => {
                            Some(*progress)
                        }
                        _ => None,
                    })?;
                self.take_original_timing_uninterrupted_main_loop_timeline(progress)
            })
            .flatten();
        if authoritative_uninterrupted_scheduled_caller_nmi_timeline.is_some()
            && matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonFallingEntrance {
                    work: DungeonFallingEntranceWork::SpriteGraphics,
                })
            )
        {
            // The falling-entrance sprite-graphics completion runs the whole
            // Dungeon_ResetSprites body when the saved caller returns; a
            // mid-window checkpoint only names the reset statement its
            // interrupting acceptance landed on.
            if let Some(receipt) = self.take_original_timing_dungeon_reset_sprites_progress() {
                assert_eq!(
                    receipt.boundary,
                    OriginalTimingBoundary::NmiAccepted,
                    "a falling-entrance Dungeon_ResetSprites checkpoint must ride its interrupting acceptance",
                );
            }
        }
        // An existing native continuation consumes source progress here to
        // refine its exact C checkpoint. With no continuation yet, leave the
        // host-return receipt on the semantic bus for the translated caller
        // which is about to enter Sprite_Main and establish that owner.
        let authoritative_big_key_drop_slot = match self.game_execution_scheduler.current_work() {
            Some(GameWorkContinuation::FinishBigKeyDropGraphics { sprite_slot, .. })
                if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                    // A terminal caller return is owned by the scheduled
                    // return lane, whose claim scope consumes the wire's
                    // Sprite_Main return at the native crossing; taking it
                    // here as well would double-consume the one receipt.
                    && authoritative_scheduled_caller_return_timeline.is_none() =>
            {
                Some(sprite_slot)
            }
            _ => None,
        };
        let authoritative_big_key_drop_progress = authoritative_big_key_drop_slot
            .and_then(|_| self.take_original_timing_sprite_main_progress());
        let authoritative_sprite_main_progress = matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishSpriteMain { .. })
        )
        .then(|| self.take_original_timing_sprite_main_progress())
        .flatten();
        let authoritative_sprite_main_returned = if authoritative_big_key_drop_slot.is_some()
            || matches!(
                self.game_execution_scheduler.current_work(),
                Some(
                    GameWorkContinuation::FinishSpriteMain { .. }
                        | GameWorkContinuation::FinishWorldMapOverlayReload
                        | GameWorkContinuation::FinishItemReceiptGraphics {
                            continuation:
                                ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt { .. }
                                    | ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. }
                                    | ItemReceiptGraphicsContinuation::ResumeAncillaItemReceipt { .. },
                        }
                )
            ) {
            self.take_original_timing_sprite_main_returned()
        } else {
            false
        };
        let authoritative_big_key_drop_completed = authoritative_big_key_drop_slot
            .is_some_and(|sprite_slot| {
                authoritative_sprite_main_returned
                    || match authoritative_big_key_drop_progress {
                        Some(SpriteMainCpuBoundary::BigKeyDropGraphicsStarted(progress_slot)) => {
                            assert_eq!(
                                progress_slot, sprite_slot,
                                "source big-key progress changed sprite slot",
                            );
                            false
                        }
                        Some(SpriteMainCpuBoundary::AfterSlot(completed_slot)) => {
                            assert!(
                                completed_slot <= sprite_slot,
                                "source big-key progress rewound before the active slot",
                            );
                            true
                        }
                        Some(other) => panic!(
                            "source big-key graphics returned through incompatible Sprite_Main progress {other:?}",
                        ),
                        None => false,
                    }
            });
        let authoritative_live_sprite_main_work = match self.game_execution_scheduler.current_work()
        {
            Some(GameWorkContinuation::FinishSpriteMain { boundary, caller })
                if matches!(caller, SpriteMainCpuCaller::DungeonModule07Live { .. })
                    || (matches!(
                        caller,
                        SpriteMainCpuCaller::DungeonModule07
                            | SpriteMainCpuCaller::Module09 { .. }
                            | SpriteMainCpuCaller::BossVictory { .. }
                            | SpriteMainCpuCaller::SaveAndQuit { .. }
                    ) && (authoritative_scheduled_caller_nmi_timeline.is_some()
                        || authoritative_sprite_main_progress.is_some()
                        || authoritative_sprite_main_returned)) =>
            {
                Some((boundary, caller))
            }
            _ => None,
        };
        let authoritative_live_sprite_main_refined_boundary = authoritative_live_sprite_main_work
            .and_then(|_| {
                authoritative_scheduled_caller_nmi_timeline
                    .filter(|timeline| {
                        timeline.progress == crate::MainLoopProgress::CallStackContinued
                    })
                    .and_then(|timeline| {
                        sprite_main_cpu_boundary_from_interruption(timeline.interruption)
                    })
                    .or(authoritative_sprite_main_progress)
            });
        let authoritative_live_sprite_main_completed = authoritative_live_sprite_main_work
            .is_some()
            && (authoritative_sprite_main_returned
                || authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                    timeline.progress == crate::MainLoopProgress::IterationStarted
                        || sprite_main_cpu_boundary_from_interruption(timeline.interruption)
                            .is_none()
                })
                || authoritative_uninterrupted_scheduled_caller_nmi_timeline
                    .as_ref()
                    .is_some_and(|timeline| {
                        timeline.progress == crate::MainLoopProgress::IterationStarted
                    }));
        let authoritative_scheduled_caller_interrupted_fresh_iteration =
            authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                timeline.progress == crate::MainLoopProgress::IterationStarted
            });
        let authoritative_overworld_spotlight_same_host_iteration =
            authoritative_overworld_spotlight_goal_is_active
                && (authoritative_scheduled_caller_interrupted_fresh_iteration
                    || matches!(
                        authoritative_uninterrupted_scheduled_caller_nmi_timeline
                            .as_ref()
                            .map(|timeline| timeline.progress),
                        Some(crate::MainLoopProgress::IterationStarted)
                    ));
        let authoritative_overworld_spotlight_build_returned =
            authoritative_overworld_spotlight_goal_returned
                || (authoritative_overworld_spotlight_goal_is_active
                    && authoritative_uninterrupted_scheduled_caller_nmi_timeline.is_some())
                // The resumed build ran to its goal transition and was
                // interrupted inside IrisSpotlight_ResetTable: the saved
                // Build completes here and its partial reset schedules the
                // remaining transition (route host 182709).
                || (matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishOverworldSpotlightBuild { .. })
                ) && authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                    matches!(
                        timeline.interruption,
                        crate::MainLoopInterruption::SpotlightGoalResetTable { .. }
                    )
                }));
        let mut authoritative_scheduled_caller_leading_nmi_completion =
            OriginalTimingNmiHandlerCompletionOwner::None;
        let mut authoritative_scheduled_caller_completes_nmi_after_same_host_iteration = false;
        let authoritative_scheduled_caller_accepts_nmi_at_return = if let Some((timeline, _, _)) =
            authoritative_scheduled_caller_return_timeline.as_ref()
        {
            let before_return = classify_original_timing_nmi_phases_with_ownership(
                self.original_timing_nmi_publication_pending,
                &timeline.nmi_phases_before_return,
            );
            authoritative_scheduled_caller_leading_nmi_completion =
                before_return.handler_completion;
            let after_return = classify_original_timing_nmi_phases_with_ownership(
                false,
                &timeline.nmi_phases_after_return,
            );
            authoritative_scheduled_caller_completes_nmi_after_same_host_iteration =
                after_return.handler_completion.completed();
            Some(after_return.publication_pending_at_exit)
        } else if let Some(timeline) = authoritative_scheduled_caller_nmi_timeline {
            let phases = timeline
                .nmi_phases_before_interruption
                .iter()
                .chain(timeline.nmi_phases_after_interruption.iter())
                .copied()
                .collect::<Vec<_>>();
            let classification = classify_original_timing_nmi_phases_with_ownership(
                self.original_timing_nmi_publication_pending,
                &phases,
            );
            // The interrupting held acceptance can land at the very end of
            // the host, its handler still pending when the host returns
            // (route hosts 641001 and 802105: [NmiAccepted(LatchHeld),
            // MainLoopInterrupted(SpritePreparation), CallStackContinued]);
            // the next host's leading completion finishes it.
            assert!(
                classification.handler_completion.completed()
                    || classification.publication_pending_at_exit,
                "an interrupted scheduled caller did not publish its accepted NMI: {timeline:?}",
            );
            authoritative_scheduled_caller_leading_nmi_completion =
                classification.handler_completion;
            Some(classification.publication_pending_at_exit)
        } else if let Some(timeline) =
            authoritative_uninterrupted_scheduled_caller_nmi_timeline.as_ref()
        {
            let phases_before_progress = timeline.nmi_phases_before_progress.as_slice();
            if !phases_before_progress.is_empty() {
                assert!(
                    self.original_timing_nmi_publication_pending
                        || phases_before_progress.first()
                            != Some(&OriginalTimingNmiPhase::HandlerCompleted),
                    "a scheduled caller completed an NMI whose acceptance marker was lost: host={:?} timeline={timeline:?} work={:?}",
                    self.original_timing_last_oracle_host_call,
                    self.game_execution_scheduler.current_work(),
                );
                let classification = classify_original_timing_nmi_phases_with_ownership(
                    self.original_timing_nmi_publication_pending,
                    phases_before_progress,
                );
                assert!(classification.handler_completion.completed(),
                    "a scheduled caller reached its semantic progress before the leading NMI lifecycle was closed: {timeline:?}",
                );
                authoritative_scheduled_caller_leading_nmi_completion =
                    classification.handler_completion;
                let accepts_before_progress = classification.publication_pending_at_exit;
                match timeline.progress {
                    crate::MainLoopProgress::CallStackContinued => {
                        // The resumed caller may run far enough to accept its
                        // following NMI before returning to the host. That
                        // acceptance belongs to the next host's publication.
                        authoritative_scheduled_caller_completes_nmi_after_same_host_iteration =
                            false;
                        Some(accepts_before_progress)
                    }
                    crate::MainLoopProgress::IterationStarted => {
                        assert!(
                            !accepts_before_progress,
                            "the source cannot begin ZeldaRunGameLoop while an accepted NMI publication remains unfinished: {timeline:?}",
                        );
                        let (completes_after_progress, accepts_after_progress) =
                            classify_original_timing_nmi_phases(
                                false,
                                &timeline.nmi_phases_after_progress,
                            );
                        authoritative_scheduled_caller_completes_nmi_after_same_host_iteration =
                            completes_after_progress;
                        Some(accepts_after_progress)
                    }
                }
            } else {
                match timeline.progress {
                    crate::MainLoopProgress::CallStackContinued => None,
                    crate::MainLoopProgress::IterationStarted => {
                        let (completes_after_progress, accepts_after_progress) =
                            classify_original_timing_nmi_phases(
                                false,
                                &timeline.nmi_phases_after_progress,
                            );
                        authoritative_scheduled_caller_completes_nmi_after_same_host_iteration =
                            completes_after_progress;
                        Some(accepts_after_progress)
                    }
                }
            }
            .inspect(|_accepts_nmi_at_return| {
                if timeline.progress == crate::MainLoopProgress::CallStackContinued {
                    assert!(
                        timeline.nmi_phases_after_progress.is_empty(),
                        "an uninterrupted source caller published an NMI lifecycle after its continuation receipt: {timeline:?}",
                    );
                }
            })
        } else {
            None
        };
        if self.lane_uninterrupted_idle_continued_return_plan(
            input,
            authoritative_uninterrupted_idle_continued_return_plan,
            oam_dma_source.clone(),
        ) {
            return;
        }
        self.stage_original_timing_scheduled_caller_acceptance_at_host_return(
            authoritative_scheduled_caller_accepts_nmi_at_return,
        );
        if self.lane_uninterrupted_idle_main_loop_timeline(
            input,
            authoritative_uninterrupted_idle_main_loop_timeline,
            oam_dma_source.clone(),
        ) {
            return;
        }
        let authoritative_dungeon_exit_spotlight_caller_is_active =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(
                        GameWorkContinuation::FinishDungeonExitSpotlightBuild { .. }
                            | GameWorkContinuation::FinishSpotlightIteration { .. }
                    )
                );
        let authoritative_dungeon_exit_spotlight_main_loop_progress =
            authoritative_dungeon_exit_spotlight_caller_is_active.then(|| {
                self.take_original_timing_scheduled_caller_progress(
                    authoritative_link_oam_timeline,
                    authoritative_uninterrupted_scheduled_caller_nmi_timeline.as_ref(),
                )
                .unwrap_or_else(|| {
                    panic!("live timing authority omitted Module0F spotlight call progress")
                })
            });
        let authoritative_dungeon_exit_spotlight_same_host_iteration = matches!(
            authoritative_dungeon_exit_spotlight_main_loop_progress,
            Some(crate::MainLoopProgress::IterationStarted)
        );
        let authoritative_dungeon_exit_spotlight_caller_returned =
            authoritative_dungeon_exit_spotlight_caller_is_active
                && (self.take_original_timing_dungeon_exit_spotlight_caller_returned()
                    || authoritative_dungeon_exit_spotlight_same_host_iteration);
        let authoritative_dungeon_exit_spotlight_link_oam_interruption_observed =
            authoritative_dungeon_exit_spotlight_caller_is_active
                && match authoritative_link_oam_timeline {
                    Some(timeline) => {
                        // A fresh ZeldaRunGameLoop iteration belongs to the new
                        // call, not the suspended spotlight caller which
                        // preceded it. CallStackContinued leaves the old caller
                        // stopped inside LinkOam_Main only when no later receipt
                        // proves that the same caller resumed through
                        // NMI_PrepareSprites and returned to main wait. The
                        // ordered source stream can contain both facts in one
                        // host: LinkOam interruption, NMI publication, then
                        // caller return. In that case the later completion owns
                        // the final source boundary. A mid-loop Link position
                        // boundary is owned by the Build plan instead.
                        timeline.interruption == crate::MainLoopInterruption::LinkOam
                            && timeline.progress == crate::MainLoopProgress::CallStackContinued
                    }
                    None => self.take_original_timing_main_loop_interruption(
                        crate::MainLoopInterruption::LinkOam,
                    ),
                };
        let authoritative_dungeon_exit_spotlight_link_oam_interruption =
            spotlight_caller_remains_interrupted_in_link_oam(
                authoritative_dungeon_exit_spotlight_caller_returned,
                authoritative_dungeon_exit_spotlight_link_oam_interruption_observed,
            );
        // A helper NMI accepted at this host's entry re-checkpoints the saved
        // build and its handler completes in-host: the resumed table ran to
        // completion and the caller continued past every tracked boundary
        // without reaching main wait (route host 207630). The Build completes
        // here; its deferred iteration return owns the next host's suffix.
        let authoritative_dungeon_exit_spotlight_build_resumed_in_host =
            matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild { .. })
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && authoritative_uninterrupted_scheduled_caller_nmi_timeline
                    .as_ref()
                    .is_some_and(|timeline| {
                        timeline.progress == crate::MainLoopProgress::CallStackContinued
                            && timeline.nmi_phases_before_progress.iter().any(|phase| {
                                matches!(phase, OriginalTimingNmiPhase::HandlerCompleted)
                            })
                    })
                && self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .is_some_and(|receipts| {
                        receipts.semantic().iter().any(|receipt| {
                            matches!(
                                receipt,
                                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                                    SpotlightTableBuildProgressReceipt {
                                        boundary: crate::OriginalTimingBoundary::NmiAccepted,
                                        ..
                                    }
                                )
                            )
                        })
                    });
        let authoritative_dungeon_exit_spotlight_work_completed =
            authoritative_dungeon_exit_spotlight_caller_returned
                || (matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild { .. })
                ) && authoritative_dungeon_exit_spotlight_link_oam_interruption)
                || authoritative_dungeon_exit_spotlight_build_resumed_in_host;
        let authoritative_pre_overworld_stage =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                .then(|| match self.game_execution_scheduler.current_work() {
                    Some(GameWorkContinuation::FinishPreOverworldProperties { .. }) => {
                        Some(crate::PreOverworldStageCompletion::PropertiesReturned)
                    }
                    Some(GameWorkContinuation::FinishPreOverworldOverlays) => {
                        Some(crate::PreOverworldStageCompletion::OverlaysReturned)
                    }
                    Some(GameWorkContinuation::FinishPreOverworldScreenBuild) => {
                        Some(crate::PreOverworldStageCompletion::ScreenBuildReturned)
                    }
                    _ => None,
                })
                .flatten();
        let authoritative_pre_overworld_completion = authoritative_pre_overworld_stage
            .map(|stage| self.take_original_timing_pre_overworld_stage_completion(stage));
        let pre_dungeon_work_is_active = matches!(
            self.game_execution_scheduler.current_work(),
            Some(
                GameWorkContinuation::FinishPreDungeonEntranceLoad { .. }
                    | GameWorkContinuation::FinishPreDungeonSongBankTransfer
            )
        );
        let mut pre_dungeon_disable_prefix = None;
        let mut pre_dungeon_garnish_prefix = None;
        if let Some(receipts) = self.original_timing_semantic_receipts.as_mut() {
            receipts.semantic.retain(|receipt| {
                if let OriginalTimingSemanticReceipt::PreDungeonSpriteDisableThrough {
                    slot, ..
                } = receipt
                {
                    assert!(
                        pre_dungeon_disable_prefix.replace(*slot).is_none(),
                        "duplicate pre-dungeon disable prefix"
                    );
                    false
                } else if let OriginalTimingSemanticReceipt::PreDungeonGarnishDisableThrough {
                    slot,
                    ..
                } = receipt
                {
                    assert!(
                        pre_dungeon_garnish_prefix.replace(*slot).is_none(),
                        "duplicate garnish disable prefix"
                    );
                    false
                } else {
                    true
                }
            });
        }
        if let Some(slot) = pre_dungeon_disable_prefix {
            self.apply_pre_dungeon_sprite_disable_prefix(slot);
        }
        if let Some(slot) = pre_dungeon_garnish_prefix {
            self.apply_pre_dungeon_garnish_disable_prefix(slot);
        }
        let pre_dungeon_sprite_reset_is_pending = matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishPreDungeonEntranceLoad {
                sprite_reset: PreDungeonSpriteResetContinuation::Pending
                    | PreDungeonSpriteResetContinuation::SpriteDisableAllThrough(_)
                    | PreDungeonSpriteResetContinuation::GarnishDisableThrough(_),
            })
        );
        if !pre_dungeon_work_is_active {
            self.defer_original_timing_pre_dungeon_return_before_native_owner();
        }
        let authoritative_pre_dungeon_sprite_reset_progress =
            (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && pre_dungeon_sprite_reset_is_pending)
                .then(|| self.take_original_timing_sprite_reset_all_progress())
                .flatten();
        if let Some(receipt) = authoritative_pre_dungeon_sprite_reset_progress {
            self.apply_pre_dungeon_sprite_reset_progress(receipt);
        }
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && pre_dungeon_work_is_active
        {
            if let Some(receipt) = self.take_original_timing_dungeon_reset_sprites_progress() {
                self.apply_pre_dungeon_dungeon_reset_progress(receipt);
            }
        }
        let authoritative_pre_dungeon_progress =
            (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && pre_dungeon_work_is_active
                && authoritative_scheduled_caller_return_timeline.is_none())
            .then(|| {
                self.take_original_timing_scheduled_caller_progress(
                    authoritative_scheduled_caller_nmi_timeline,
                    authoritative_uninterrupted_scheduled_caller_nmi_timeline.as_ref(),
                )
                .unwrap_or_else(|| {
                    panic!("live timing authority omitted Module_PreDungeon call progress")
                })
            });
        let authoritative_pre_dungeon_returned =
            matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && pre_dungeon_work_is_active
                && self.take_original_timing_pre_dungeon_module_returned();
        if authoritative_pre_dungeon_returned
            && self
                .game_execution_scheduler
                .current_work()
                .is_some_and(|work| {
                    matches!(
                        work,
                        GameWorkContinuation::FinishPreDungeonEntranceLoad { .. }
                    )
                })
            && self
                .game_execution_scheduler
                .scheduled_work_slices_remaining()
                .is_some_and(|remaining| remaining > PRE_DUNGEON_RETURN_SUFFIX_NMI_SLICES)
        {
            // The replaceable authority observed the original C call return
            // before the native timing shadow reached its final checkpoint.
            // Everything before that return is therefore proven complete,
            // including the Module_PreDungeon state publication which precedes
            // Sprite_ResetAll/Dungeon_ResetSprites. Apply that still-pending C
            // prefix before clearing the scheduled continuation; otherwise the
            // early semantic completion would skip CGWSEL_copy and the other
            // source-ordered mutations entirely.
            let GameWorkContinuation::FinishPreDungeonEntranceLoad { sprite_reset } = self
                .game_execution_scheduler
                .current_work()
                .expect("pre-dungeon work was checked above")
            else {
                unreachable!("pre-dungeon return completion changed work kind")
            };
            self.complete_pre_dungeon_sprite_reset_continuation(sprite_reset);
            assert!(self
                .game_execution_scheduler
                .mark_pre_dungeon_sprite_reset_completed());
        }
        if let Some((boundary, caller)) = authoritative_live_sprite_main_work.filter(|_| {
            authoritative_scheduled_caller_leading_nmi_completion.completed()
                && authoritative_live_sprite_main_completed
                && authoritative_scheduled_caller_return_timeline
                    .as_ref()
                    .is_some_and(|(timeline, _, _)| timeline.sprite_main_returned_before_nmi)
        }) {
            let work = GameWorkContinuation::FinishSpriteMain { boundary, caller };
            assert!(
                scheduled_caller_return_runs_dungeon_sprite_main_before_leading_nmi(work),
                "only a parked dungeon Sprite_Main can return before this host's leading NMI: {work:?}",
            );
            assert!(
                authoritative_live_sprite_main_refined_boundary
                    .is_none_or(|refined| refined == boundary),
                "a Sprite_Main return before the leading NMI cannot also refine its slot boundary",
            );
            // The wire returned from the parked slot loop before this
            // host's held vblank (route host 1415870): the remaining slots
            // run against the pre-NMI frame counter, then the handler
            // completes, then the Module 7 tail and the shared suffix
            // resume through the post-Sprite_Main continuation.
            self.complete_sprite_main_after_cpu_boundary(boundary);
            self.game_execution_scheduler.refine_scheduled_work(
                work,
                GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn,
            );
        }
        self.lane_retire_completed_leading_nmi(
            input,
            authoritative_scheduled_caller_leading_nmi_completion,
            authoritative_scheduled_caller_nmi_timeline,
            oam_dma_source.clone(),
        );
        if let Some(next) = authoritative_dungeon_peg_flip_progress {
            if next.boundary == OriginalTimingBoundary::NmiAccepted {
                assert!(
                    authoritative_scheduled_caller_leading_nmi_completion.completed(),
                    "an NMI-exposed peg-attribute slice must resume after its leading held handler",
                );
            }
            assert_eq!(
                self.take_original_timing_dungeon_peg_attribute_flip_progress(),
                Some(next),
                "the validated peg-attribute cursor changed before CPU continuation",
            );
            let mut continuation = self
                .dungeon_peg_attribute_flip_pending
                .take()
                .expect("continued peg-attribute slice lost its preceding source cursor");
            self.advance_dungeon_peg_attribute_flip_between_progress(continuation.progress, next);
            continuation.progress = next;
            self.dungeon_peg_attribute_flip_pending = Some(continuation);
        }
        self.lane_scheduled_caller_call_stack_continued_nmi(
            authoritative_scheduled_caller_accepts_nmi_at_return,
            authoritative_scheduled_caller_nmi_timeline,
        );
        // A Sprite_Main continuation parked behind the current trailing NMI
        // (an after-trailing lane, not scheduled work) can also be refined by
        // the wire's slot progress while the ROM stays inside Sprite_Main
        // across several held hosts (route host 402689: BeforeFirstSlot with
        // SpriteMainProgressed(AfterSlot(4)) three hosts later).
        if self.game_execution_scheduler.current_work().is_none()
            && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            if let Some(GameWorkContinuation::FinishSpriteMain {
                boundary: SpriteMainCpuBoundary::BeforeFirstSlot,
                caller,
            }) = self
                .game_execution_scheduler
                .peek_after_current_trailing_nmi()
            {
                if let Some(SpriteMainCpuBoundary::AfterSlot(newly_completed_slot)) =
                    self.take_original_timing_sprite_main_progress()
                {
                    self.advance_sprite_main_before_first_slot_to_after_slot(newly_completed_slot);
                    let taken = self
                        .game_execution_scheduler
                        .take_after_current_trailing_nmi();
                    debug_assert!(taken.is_some());
                    self.game_execution_scheduler
                        .schedule_after_current_trailing_nmi(
                            GameWorkContinuation::FinishSpriteMain {
                                boundary: SpriteMainCpuBoundary::AfterSlot(newly_completed_slot),
                                caller,
                            },
                        );
                }
            }
        }
        self.lane_live_sprite_main_refined_boundary(
            authoritative_live_sprite_main_refined_boundary,
            authoritative_live_sprite_main_work,
        );
        if let Some(predecessor) = prospective_interrupted_idle_main_loop_plan
            .as_ref()
            .and_then(|plan| plan.scheduled_predecessor)
        {
            assert_eq!(
                self.game_execution_scheduler, predecessor.scheduler_before_step,
                "scheduled caller-to-fresh-iteration state changed before its source-authoritative completion",
            );
        }
        if crate::debug_env::var_os("ZELDA3_DEBUG_HOST_PATH").is_some()
            && matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                    work: DungeonSupertileTransitionWork::AuxiliarySpriteGraphics,
                })
            )
        {
            eprintln!(
                "[HOSTPATH] host={} aux-sprite-graphics hold check: live={} slices={:?} bare={} rng_sample={:?} progress={:?} fresh={} interruption={:?} return_timeline={} owes_return={} owes_progress={}",
                self.frame_ctr_dbg,
                matches!(self.original_timing_owner, OriginalTimingOwnerState::Live),
                self.game_execution_scheduler
                    .scheduled_work_slices_remaining(),
                self.original_timing_host_is_bare_held_continuation(),
                self.rom_random_replay.has_sample_for_current_frame(),
                self.original_timing_main_loop_progress(),
                self.original_timing_hosts_fresh_iteration(),
                self.original_timing_main_loop_interruption(),
                self.original_timing_main_loop_return_timeline().is_some(),
                self.original_timing_owes_sprite_main_return(),
                self.original_timing_owes_sprite_main_progress(),
            );
        }
        let scheduled_work_step = if self.rom_startup_timing() {
            if let Some(plan) = prospective_terminal_ground_item_receipt_plan.as_ref() {
                assert_eq!(
                    self.game_execution_scheduler, plan.scheduler_before_completion,
                    "terminal ground-item scheduler changed before its source-authoritative completion",
                );
                self.game_execution_scheduler = plan.scheduler_after_completion;
                Some(GameWorkStep::Complete(plan.work))
            } else if let Some(plan) = prospective_spotlight_build_link_oam_plan.as_ref() {
                assert_eq!(
                    self.game_execution_scheduler, plan.scheduler_before_completion,
                    "spotlight Build-LinkOam scheduler changed before its source-authoritative completion",
                );
                self.game_execution_scheduler = plan.scheduler_after_completion;
                // The interrupting acceptance can restate the build's
                // checkpoint alongside the interruption; the plan already
                // validated it against the exact vector, and the saved
                // continuation carries the same statement (route host 37587).
                if let Some(claim) = self.take_original_timing_spotlight_table_build_progress() {
                    assert_eq!(
                        claim.boundary,
                        crate::OriginalTimingBoundary::NmiAccepted,
                        "a Build-LinkOam checkpoint restatement must ride the interrupting acceptance",
                    );
                    let GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                        table_build,
                        ..
                    } = plan.work
                    else {
                        unreachable!("Build-LinkOam plan lost its table continuation")
                    };
                    table_build.assert_recheckpointed_at(claim.progress);
                    if let Some(progress) = plan.rebuild_progress {
                        assert_eq!(progress, claim.progress);
                        assert_eq!(
                            self.begin_iris_spotlight_configure_table_at_progress(progress),
                            table_build,
                            "spotlight source prefix changed after immutable preflight",
                        );
                    }
                }
                Some(GameWorkStep::Complete(plan.work))
            } else if let Some((_, expected_work, _)) =
                authoritative_scheduled_caller_return_timeline.as_ref()
            {
                Some(GameWorkStep::Complete(*expected_work))
            } else if matches!(self.game_execution_scheduler.current_work(), Some(GameWorkContinuation::FinishGameOverDeathAfterSpriteReset { .. })) {
                assert!(matches!(self.original_timing_owner, OriginalTimingOwnerState::Live));
                self.game_execution_scheduler.advance_work_one_nmi_slice_with_authoritative_completion(false)
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                    work: DungeonSupertileTransitionWork::SpiralRoomInitialization,
                })
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
            {
                // This ROM room initializer has an exact live caller-return
                // wire. Its estimated slice count is only a fallback for
                // non-live execution: a bare CallStackContinued host keeps the
                // source call in flight (route host 716884), while a Sprite_Main
                // checkpoint or caller return completes it even if an estimate
                // remains (route host 907403).
                let returned = authoritative_supertile_sprite_main_returned
                    || self.original_timing_owes_sprite_main_progress()
                    || self.original_timing_owes_sprite_main_return()
                    || self.original_timing_main_loop_interruption().is_some()
                    || self.original_timing_main_loop_return_timeline().is_some();
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(returned)
            } else if self.game_execution_scheduler.current_work()
                == Some(GameWorkContinuation::FinishWorldMapExitTilesets)
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
            {
                // The world-map exit's tileset reload returns through the
                // shared suffix (or is interrupted inside it): a live host
                // whose wire still holds the latch with no interruption or
                // return proves the reload is still running (route host
                // 529536).
                let returned = !self.original_timing_live_suffix_outstanding()
                    || self.original_timing_main_loop_interruption().is_some()
                    || authoritative_scheduled_caller_nmi_timeline.is_some()
                    || self.original_timing_main_loop_return_timeline().is_some();
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(returned)
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishTriforceRoomLoad {
                    step: TriforceRoomLoadStep::Case2Upload
                        | TriforceRoomLoadStep::Case2SpecialAreaPalettes { .. },
                })
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
            {
                // The source checkpoint is both the upload-return proof and
                // the exact palette prefix committed in this host. No native
                // slice estimate may complete either live continuation.
                let returned = self
                    .original_timing_triforce_room_case2_palette_progress()
                    .is_some();
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(returned)
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishTriforceRoomLoad {
                    step: TriforceRoomLoadStep::CreditsIteration { .. },
                })
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
            {
                // A credits scene may publish its submodule/subsubmodule
                // advance and a text-buffer prefix one host before the outer
                // ZeldaRunGameLoop call returns.
                let returned = self.original_timing_credits_scene_load_progress().is_some()
                    || self
                        .original_timing_credits_end_sequence_32_progress()
                        .is_some();
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(returned)
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishTriforceRoomLoad {
                    step: TriforceRoomLoadStep::Case9Scroll,
                })
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
                && !self.dialogue_scroll_cpu_is_idle()
            {
                // Lag host of the Triforce room's message-line scroll (route
                // hosts 1558262-1558267): the ROM's main thread is still
                // inside RenderText_Draw_Scroll, so this host copies the
                // receipt's pixel passes and nothing else; the source RTS
                // returns through RenderText into Module19's tail.
                // The scroll's RTS stayed inside this host whenever the wire shows
                // the caller running on afterwards: a completed shared suffix,
                // or an interruption inside that suffix (SpritePreparation at
                // route host 1558283). Only a bare continued return leaves the
                // RTS itself to cross the boundary.
                let completion_timing = if self.original_timing_main_loop_iteration_returned_to_wait()
                    || self.original_timing_main_loop_interruption().is_some()
                    || authoritative_scheduled_caller_nmi_timeline.is_some()
                {
                    DialogueScrollCompletionTiming::BeforeNextVblank
                } else {
                    DialogueScrollCompletionTiming::AfterReturnBoundary
                };
                let returned =
                    self.advance_triforce_room_dialogue_scroll_lag_host(completion_timing);
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(returned)
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(
                    GameWorkContinuation::FinishDungeonMapRecovery
                        | GameWorkContinuation::FinishDungeonMapGraphicsPreparation
                        | GameWorkContinuation::FinishTriforceRoomLoad { .. }
                )
            ) && !matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishTriforceRoomLoad {
                    step: TriforceRoomLoadStep::Case7PolyGraphics,
                })
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
            {
                // Module0E's dungeon-map recovery returns through the shared
                // ZeldaRunGameLoop suffix; a live host whose wire still holds
                // the update latch (Held acceptance, no suffix completion)
                // proves the recovery is still running (route host 897354).
                // The map graphics preparation (InitializeTilesets) returns
                // the same way; its slice estimate ended one host before the
                // wire's return at route host 1280082.
                // The Triforce room's case-2 tail may also end its host inside
                // LinkOam_Main (route host 1557676): the load itself returned.
                let returned = !self.original_timing_live_suffix_outstanding()
                    || (matches!(
                        self.game_execution_scheduler.current_work(),
                        Some(GameWorkContinuation::FinishTriforceRoomLoad { .. })
                    ) && (self.original_timing_main_loop_interruption()
                        == Some(crate::MainLoopInterruption::LinkOam)
                        || authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                            timeline.interruption == crate::MainLoopInterruption::LinkOam
                        })));
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(returned)
            } else if authoritative_big_key_drop_slot.is_some() {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_big_key_drop_completed,
                    )
            } else if authoritative_live_sprite_main_work.is_some() {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_live_sprite_main_completed,
                    )
            } else if authoritative_item_receipt_is_active {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_item_receipt_returned,
                    )
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishItemReceiptGraphics {
                    continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { .. },
                })
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
            {
                // The atomic item-graphics decompression blocks the ROM
                // iteration before Sprite_Main. A live host that only holds
                // the update latch (Held acceptances, no main-loop progress
                // past the receipt) proves the decompression is still
                // running; the first host that reaches Sprite_Main, returns
                // to the shared suffix, or interrupts later proves it
                // returned (route host 1339570: the mirror-shield chest's
                // sheet $5c ran two hosts past the standard estimate).
                let returned = self.original_timing_owes_sprite_main_progress()
                    || self.original_timing_owes_sprite_main_return()
                    || !self.original_timing_live_suffix_outstanding()
                    || self.original_timing_main_loop_interruption().is_some()
                    || authoritative_scheduled_caller_nmi_timeline.is_some()
                    || self.original_timing_main_loop_return_timeline().is_some();
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(returned)
            } else if pre_overworld_sprite_reset_is_active
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            {
                // The decompression/palette prefix has variable CPU cost. The
                // live source boundary inside Sprite_ResetAll is the first
                // point where the old sprite generation is actually gone;
                // the fixed 37-crossing count is only the no-wire fallback.
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_sprite_reset_all_progress.is_some(),
                    )
            } else if authoritative_overworld_sprite_reload_is_active {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_overworld_sprite_reload_returned,
                    )
            } else if authoritative_overworld_load_overlays_overlay_is_active {
                // The fixed four-crossing count is only a native fallback.
                // With a live wire, `LoadOverworldOverlay` has returned only
                // once its enclosing Module09 caller reaches Sprite_Main (or
                // the terminal-return lane above proves Sprite_Main and the
                // shared suffix both returned). A bare Continued host keeps
                // the overlay call suspended even after the estimate reaches
                // zero (route host 186367).
                let caller_reached_sprite_main = !self.original_timing_hosts_fresh_iteration()
                    && (self.original_timing_owes_sprite_main_return()
                        || self.original_timing_owes_sprite_main_progress()
                        || authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                            timeline.progress == crate::MainLoopProgress::CallStackContinued
                                && timeline.interruption.is_sprite_main()
                        }));
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        caller_reached_sprite_main,
                    )
            } else if authoritative_overworld_map_quadrants_is_active {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_overworld_map_quadrants_published,
                    )
            } else if authoritative_world_map_overlay_reload_is_active {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_world_map_overlay_reload_returned,
                    )
            } else if authoritative_world_map_ambient_map8_is_active {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_world_map_ambient_map8_returned,
                    )
            } else if authoritative_overworld_spotlight_goal_is_active {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_overworld_spotlight_build_returned,
                    )
            } else if authoritative_dungeon_exit_spotlight_entry_is_active {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_dungeon_exit_spotlight_entry_returned,
                    )
            } else if authoritative_dungeon_exit_spotlight_caller_is_active {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_dungeon_exit_spotlight_work_completed,
                    )
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishFluteMenuSelectedScreen { .. })
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            {
                // Every phase transition is supplied by a typed source
                // receipt above, and the complete selected-screen caller is
                // retired only by its terminal main-loop return timeline.
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(false)
            } else if let Some(completed) = authoritative_pre_overworld_completion {
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(completed)
            } else if self.game_execution_scheduler.current_work()
                == Some(GameWorkContinuation::FinishWorldMapExitTilesets)
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                    timeline.progress == crate::MainLoopProgress::CallStackContinued
                        && timeline.interruption == crate::MainLoopInterruption::SpritePreparation
                })
            {
                // The wire interrupted NMI_PrepareSprites after WorldMap_ExitMap
                // returned: the tileset load is complete regardless of the
                // slice estimate (route host 1230513).
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(true)
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                    work: DungeonSupertileTransitionWork::AuxiliarySpriteGraphics,
                })
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            {
                // The ROM-CPU shadow supplies the native fallback budget, but
                // only a source statement after LoadTransAuxGFX may publish
                // its mutations in live mode.  A bare Continued host is still
                // executing the decompressor even when that estimate reaches
                // zero (room $65, source run 711825).  Conversely, an exact
                // Dungeon_ResetSprites/Sprite_Main checkpoint proves the call
                // returned even when the estimate has slices left.  Keep both
                // directions under the same source-authoritative predicate;
                // do not infer completion from RNG activity or add a room-
                // specific slice adjustment.
                let returned = !self.original_timing_hosts_fresh_iteration()
                    && (authoritative_supertile_sprite_main_returned
                    || self.original_timing_owes_sprite_main_return()
                    || self.original_timing_owes_sprite_main_progress()
                    || self.original_timing_cached_sprite_execution_progress().is_some()
                    || self.original_timing_dungeon_reset_sprites_progress_pending()
                    || self.original_timing_dungeon_push_blocks_pending()
                    || self.original_timing_dungeon_push_blocks_in_progress().is_some()
                    || self.original_timing_dungeon_push_blocks_handled()
                    || authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                        timeline.progress == crate::MainLoopProgress::CallStackContinued
                            && module_cpu_phase_from_main_loop_interruption(
                                timeline.interruption,
                            )
                            .is_some()
                    }));
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(returned)
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                    work: DungeonSupertileTransitionWork::RoomLoadCallerResume
                        | DungeonSupertileTransitionWork::SpiralRoomCallerResume
                        | DungeonSupertileTransitionWork::SpriteConversionCallerResume,
                })
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            {
                // A caller-resume stage returns through the module body's
                // Sprite_Main; the wire's own Sprite_Main activity in a
                // continuation host (no fresh iteration owns those receipts)
                // proves the resume happened here, superseding the slice
                // estimate in both directions (route host 89672; a host that
                // also starts a fresh iteration keeps the estimate, route
                // host 12138).
                let caller_reached_sprite_main = !self
                    .original_timing_hosts_fresh_iteration()
                    && (self.original_timing_owes_sprite_main_return()
                        || self.original_timing_owes_sprite_main_progress()
                        || authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                            timeline.progress == crate::MainLoopProgress::CallStackContinued
                                && timeline.interruption.is_sprite_main()
                        }));
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        caller_reached_sprite_main,
                    )
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(
                    GameWorkContinuation::FinishOverworldSpecialExitMosaic
                        | GameWorkContinuation::FinishOverworldSpecialExitMosaicSecondDecode
                )
            )
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            {
                let source_stage_completed = match self.game_execution_scheduler.current_work() {
                    Some(GameWorkContinuation::FinishOverworldSpecialExitMosaic) => {
                        authoritative_overworld_special_exit_mosaic_restored
                            || authoritative_overworld_special_exit_mosaic_returned
                    }
                    Some(GameWorkContinuation::FinishOverworldSpecialExitMosaicSecondDecode) => {
                        authoritative_overworld_special_exit_mosaic_returned
                    }
                    _ => unreachable!("special-exit mosaic work changed before advancement"),
                };
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        source_stage_completed,
                    )
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(
                    GameWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail
                        | GameWorkContinuation::FinishSpiralStaircasePaletteFilter {
                            tail: SpiralStaircasePaletteTail::FallingFadeInAfterDirectionToggle,
                            ..
                        }
                        // The whirlpool's long loads return into the same
                        // Module09 Sprite_Main suffix (route host 183230).
                        | GameWorkContinuation::FinishModule09LongLoad { .. }
                        // The straight-stair room initializer returns into
                        // Module07's Sprite_Main the same way (route host
                        // 307614).
                        | GameWorkContinuation::FinishDungeonSupertileTransition {
                            work: DungeonSupertileTransitionWork::StraightInterroomRoomInitialization
                                | DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics
                                | DungeonSupertileTransitionWork::StraightInterroomBgCharacters34
                                | DungeonSupertileTransitionWork::FallingBgCharacters34
                                | DungeonSupertileTransitionWork::FallingSpriteGraphics
                        }
                        // The special-overworld aux-graphics caller (route
                        // host 263657).
                        | GameWorkContinuation::FinishOverworldAuxGraphics
                        | GameWorkContinuation::FinishOverworldMosaicSpriteGraphics
                )
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && !matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishModule09LongLoad { step })
                        if !step.completes_by_wire()
                )
            {
                // The wire's own Sprite_Main activity proves the screen-map
                // and sprite-conversion tail already returned in this host;
                // the slice estimate cannot complete the caller before that
                // receipt nor hold it past it (return at route host 70608,
                // mid-loop suspension at route host 70838).
                let caller_reached_sprite_main = !self
                    .original_timing_hosts_fresh_iteration()
                    && (self.original_timing_owes_sprite_main_return()
                        || authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                            timeline.progress == crate::MainLoopProgress::CallStackContinued
                                && timeline.interruption.is_sprite_main()
                        })
                        || self.original_timing_owes_sprite_main_progress());
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        caller_reached_sprite_main,
                    )
            } else if matches!(
                self.game_execution_scheduler.current_work(),
                Some(
                    GameWorkContinuation::FinishDungeonFallingEntrance { .. }
                        | GameWorkContinuation::FinishDungeonFallingRoomInitialization
                        | GameWorkContinuation::FinishRescuedMaidenTilemapClear { .. }
                        | GameWorkContinuation::FinishRescuedMaidenInitialization { .. }
                )
            ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            {
                // The wire's terminal return, not the slice estimate, decides
                // when the long Module 11 load completes (route host 58563).
                // The falling room initializer's caller can also be proven
                // complete by the wire's in-host Sprite_Main return when the
                // module tail is then interrupted before the shared suffix
                // (route host 378771: [NmiHandlerCompleted, SpriteMainReturned,
                // NmiAccepted(LatchHeld), CallStackContinued]).
                let falling_room_init_reached_sprite_main = self
                    .game_execution_scheduler
                    .current_work()
                    == Some(GameWorkContinuation::FinishDungeonFallingRoomInitialization)
                    && !self.original_timing_hosts_fresh_iteration()
                    && (self.original_timing_owes_sprite_main_return()
                        || self.original_timing_owes_sprite_main_progress()
                        || authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                            timeline.progress == crate::MainLoopProgress::CallStackContinued
                                && timeline.interruption.is_sprite_main()
                        }));
                let rescued_maiden_initialization_reached_sprite_main = matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishRescuedMaidenInitialization { .. })
                ) && !self.original_timing_hosts_fresh_iteration()
                    && (self.original_timing_owes_sprite_main_return()
                        || self.original_timing_owes_sprite_main_progress()
                        || authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                            timeline.progress == crate::MainLoopProgress::CallStackContinued
                                && timeline.interruption.is_sprite_main()
                        }));
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        self.original_timing_main_loop_return_timeline().is_some()
                            || falling_room_init_reached_sprite_main
                            || rescued_maiden_initialization_reached_sprite_main,
                    )
            } else if pre_dungeon_work_is_active
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
            {
                // Live Module_PreDungeon work is owned by its source return,
                // including hosts with no intermediate semantic checkpoint.
                // The native slice count is a fallback estimate only and may
                // reach zero before the ROM enters Sprite_ResetAll.
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(
                        authoritative_pre_dungeon_returned,
                    )
            } else if self.game_execution_scheduler.current_work()
                == Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn)
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
            {
                // The source caller has returned as soon as its enclosing
                // Module 7 tail reaches Sprite_Main (or a later shared-loop
                // boundary). A bare held host proves the opposite. The ROM
                // instruction stream, not the spotlight slice estimate, owns
                // this transition: in room $107 it resumes after the held NMI
                // and enters the bonk item's sheet decode in the same host.
                let caller_reached_sprite_main = !self.original_timing_hosts_fresh_iteration()
                    && (self.original_timing_owes_sprite_main_return()
                        || self.original_timing_owes_sprite_main_progress()
                        || authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                            timeline.progress == crate::MainLoopProgress::CallStackContinued
                                && timeline.interruption.is_sprite_main()
                        }));
                let caller_returned = caller_reached_sprite_main
                    || self.original_timing_main_loop_return_timeline().is_some();
                self.game_execution_scheduler
                    .advance_work_one_nmi_slice_with_authoritative_completion(caller_returned)
            } else {
                self.game_execution_scheduler.advance_work_one_nmi_slice()
            }
        } else {
            None
        };
        if crate::debug_env::var_os("ZELDA3_DEBUG_HOST_PATH").is_some() {
            eprintln!(
                "[HOSTPATH] host={} step={:?} nmi_timeline={:?} return_timeline={} remaining={:?}",
                self.frame_ctr_dbg,
                scheduled_work_step,
                authoritative_scheduled_caller_nmi_timeline.map(|t| (t.progress, t.interruption)),
                authoritative_scheduled_caller_return_timeline.is_some(),
                self.original_timing_semantic_receipts
                    .as_ref()
                    .map(|r| r.semantic().to_vec()),
            );
        }
        if let Some(predecessor) = prospective_interrupted_idle_main_loop_plan
            .as_ref()
            .and_then(|plan| plan.scheduled_predecessor)
        {
            assert_eq!(
                scheduled_work_step,
                Some(GameWorkStep::Complete(predecessor.work)),
                "scheduled caller-to-fresh-iteration did not complete its immutable predecessor",
            );
            assert_eq!(
                self.game_execution_scheduler, predecessor.scheduler_after_step,
                "scheduled caller-to-fresh-iteration completion changed its probed scheduler transition",
            );
        }
        if let Some(plan) = prospective_spotlight_build_link_oam_plan.as_ref() {
            assert_eq!(
                scheduled_work_step,
                Some(GameWorkStep::Complete(plan.work)),
                "spotlight Build-LinkOam scheduler did not commit its probed completion",
            );
            assert_eq!(
                self.game_execution_scheduler, plan.scheduler_after_completion,
                "spotlight Build-LinkOam completion changed its probed scheduler transition",
            );
        }
        if self.lane_terminal_ground_item_receipt_plan(
            input,
            oam_dma_source.clone(),
            prospective_terminal_ground_item_receipt_plan,
            scheduled_work_step,
        ) {
            return;
        }
        if matches!(scheduled_work_step, Some(GameWorkStep::Waiting))
            && self
                .game_execution_scheduler
                .current_work()
                .is_some_and(|work| {
                    matches!(
                        work,
                        GameWorkContinuation::FinishPreDungeonEntranceLoad { .. }
                    )
                })
            && self
                .game_execution_scheduler
                .scheduled_work_slices_remaining()
                == Some(PRE_DUNGEON_RETURN_SUFFIX_NMI_SLICES)
            && !(matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some())
        {
            // On the C stack, Sprite_ResetAll finishes before the final NMI
            // that separates it from Dungeon_ResetSprites and the module-7
            // publication. Preserve that observable call boundary instead of
            // collapsing both sprite generations into the return slice.
            let GameWorkContinuation::FinishPreDungeonEntranceLoad { sprite_reset } = self
                .game_execution_scheduler
                .current_work()
                .expect("pre-dungeon work was checked above")
            else {
                unreachable!("pre-dungeon threshold changed work kind")
            };
            self.complete_pre_dungeon_sprite_reset_continuation(sprite_reset);
            assert!(self
                .game_execution_scheduler
                .mark_pre_dungeon_sprite_reset_completed());
        }
        if let Some(GameWorkStep::Complete(
            continuation @ (GameWorkContinuation::PreOverworldPropertiesSpriteReset { .. }
            | GameWorkContinuation::FinishPreOverworldProperties { .. }
            | GameWorkContinuation::FinishPreOverworldOverlays
            | GameWorkContinuation::FinishPreOverworldScreenBuild),
        )) = scheduled_work_step
        {
            // Each pre-overworld load is one synchronous C call. Its final
            // measured slice is the NMI which interrupts that call; only
            // afterward does the saved CPU stack mutate the semantic stage
            // and return through ZeldaRunGameLoop. Publishing the completion
            // before this NMI shifts every following module iteration one
            // hardware boundary early.
            if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
                self.capture_display_snapshot();
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            }
            match continuation {
                GameWorkContinuation::PreOverworldPropertiesSpriteReset {
                    overworld_screen,
                    animated_tiles,
                } => {
                    self.complete_pre_overworld_load_properties_through_sprite_reset(
                        overworld_screen,
                        animated_tiles,
                    );
                    self.game_execution_scheduler.schedule_work(
                        GameWorkContinuation::FinishPreOverworldProperties {
                            overworld_screen,
                            sprite_presence_published: false,
                        },
                        PRE_OVERWORLD_PROPERTIES_AFTER_SPRITE_RESET_NMI_SLICES,
                    );
                }
                GameWorkContinuation::FinishPreOverworldProperties {
                    overworld_screen,
                    sprite_presence_published,
                } => {
                    self.complete_pre_overworld_load_properties_after_sprite_reset_with_presence(
                        overworld_screen,
                        sprite_presence_published,
                    );
                    if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && authoritative_scheduled_caller_return_timeline.is_none()
                    {
                        // A nonterminal wire stage return keeps the shared
                        // ZeldaRunGameLoop suffix outstanding across the
                        // forced-blank wait hosts; arm its one owner for the
                        // later suffix-completing host (route host 37662).
                        if self.pending_main_loop_common_suffix.is_none() {
                            self.pending_main_loop_common_suffix = Some(
                                MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch,
                            );
                        }
                    } else if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven stage return already carries the
                        // shared ZeldaRunGameLoop suffix; retire it through
                        // the one typed owner instead of repeating it later.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    self.game_execution_scheduler
                        .finish_call_stack_at_main_wait_before_nmi();
                }
                GameWorkContinuation::FinishPreOverworldOverlays => {
                    self.complete_pre_overworld_load_overlays();
                    if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && authoritative_scheduled_caller_return_timeline.is_none()
                    {
                        // A nonterminal wire stage return keeps the shared
                        // ZeldaRunGameLoop suffix outstanding across the
                        // forced-blank wait hosts; arm its one owner for the
                        // later suffix-completing host (route host 37662).
                        if self.pending_main_loop_common_suffix.is_none() {
                            self.pending_main_loop_common_suffix = Some(
                                MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch,
                            );
                        }
                    } else if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven stage return already carries the
                        // shared ZeldaRunGameLoop suffix; retire it through
                        // the one typed owner instead of repeating it later.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    self.game_execution_scheduler
                        .finish_call_stack_at_main_wait_before_nmi();
                }
                GameWorkContinuation::FinishPreOverworldScreenBuild => {
                    self.complete_pre_overworld_screen_build();
                    if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && authoritative_scheduled_caller_return_timeline.is_none()
                    {
                        // A nonterminal wire stage return keeps the shared
                        // ZeldaRunGameLoop suffix outstanding across the
                        // forced-blank wait hosts; arm its one owner for the
                        // later suffix-completing host (route host 37662).
                        if self.pending_main_loop_common_suffix.is_none() {
                            self.pending_main_loop_common_suffix = Some(
                                MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch,
                            );
                        }
                    } else if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven stage return already carries the
                        // shared ZeldaRunGameLoop suffix; retire it through
                        // the one typed owner instead of repeating it later.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    self.game_execution_scheduler
                        .finish_call_stack_at_main_wait_before_nmi();
                }
                _ => unreachable!(),
            }
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return;
        }
        if let Some(GameWorkStep::Complete(
            continuation @ (GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn
            | GameWorkContinuation::FinishModule09LinkOamCallerReturn { .. }
            | GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn { .. }),
        )) = scheduled_work_step
        {
            // The prior host returned when vblank interrupted the translated
            // Module 7 caller. Publish that interrupting NMI first, resume only
            // the saved suffix, then cross the following field boundary before
            // returning this host interval.
            if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
                self.capture_display_snapshot_with_publication(
                    DisplaySnapshotPublication::RetainPublished,
                );
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            }
            self.complete_post_trailing_nmi_continuation(
                continuation,
                input,
                false,
                authoritative_scheduled_caller_return_timeline.is_some(),
            );
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return;
        }
        if self.lane_completed_dungeon_after_submodule_caller_return(
            input,
            authoritative_scheduled_caller_return_timeline.clone(),
            scheduled_work_audio_follows_host_publication,
            scheduled_work_started_after_leading_nmi,
            scheduled_work_step,
        ) {
            return;
        }
        if self.lane_completed_spiral_staircase_palette_filter(
            input,
            authoritative_scheduled_caller_accepts_nmi_at_return,
            authoritative_scheduled_caller_nmi_timeline,
            authoritative_scheduled_caller_return_timeline.clone(),
            oam_dma_source.clone(),
            scheduled_work_step,
        ) {
            return;
        }
        if let Some(work_slice) = scheduled_work_step {
            if let GameWorkStep::Complete(continuation) = work_slice {
                let publication = continuation.completion_publication(scheduled_work_entry_scroll);
                if let Some(generation) = publication.bg_scroll {
                    self.next_display_bg_scroll_generation = generation;
                }
                if let Some(generation) = publication.obj {
                    self.set_next_display_obj_scanout(Some(generation));
                }
            }
            let publication_override = match work_slice {
                GameWorkStep::Complete(GameWorkContinuation::FinishGameOverSpotlightBuild {
                    iteration,
                    ..
                }) => Some(iteration.game_over_build_completion_publication()),
                GameWorkStep::Complete(GameWorkContinuation::FinishSpotlightIteration {
                    iteration,
                }) => Some(iteration.completion_publication()),
                GameWorkStep::Complete(GameWorkContinuation::FinishOverworldSpotlightBuild {
                    iteration,
                    ..
                }) => Some(iteration.completion_publication()),
                GameWorkStep::Complete(GameWorkContinuation::FinishOverworldSpotlightLinkOam {
                    iteration,
                    ..
                }) => Some(iteration.completion_publication()),
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishOverworldSpotlightGoalResetTable {
                        iteration, ..
                    },
                ) => Some(iteration.completion_publication()),
                // The interrupted entry build finishes mid-frame; its first
                // table generation belongs to the scanout already staged.
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonExitSpotlightEntry {
                    ..
                }) => Some(DisplaySnapshotPublication::AdvanceStaged),
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                    iteration,
                    ..
                }) => Some(iteration.completion_publication()),
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { iteration },
                ) => Some(iteration.completion_publication()),
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity {
                        iteration, ..
                    },
                ) => Some(iteration.completion_publication()),
                GameWorkStep::Complete(
                GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity {
                    iteration,
                    ..
                },
            ) => Some(iteration.completion_publication()),
            GameWorkStep::Complete(
                GameWorkContinuation::FinishDungeonExitSpotlightLinkAndOam { iteration }
                | GameWorkContinuation::FinishDungeonExitSpotlightControl { iteration },
            ) => Some(iteration.completion_publication()),
            GameWorkStep::Complete(
                GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement {
                        iteration, ..
                    }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel {
                        iteration,
                        ..
                    }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow {
                        iteration,
                        ..
                    }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates {
                        iteration,
                        ..
                    },
                ) => Some(iteration.completion_publication()),
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { .. },
                ) => {
                    // The retained goal field and the following all-black hold
                    // have both elapsed. The resumed C caller runs in vblank,
                    // so its HDMA disable/window terminal state owns the next
                    // active field directly.
                    Some(DisplaySnapshotPublication::PublishCaptured)
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishItemReceiptGraphics {
                    ..
                }) => Some(DisplaySnapshotPublication::RetainPublished),
                _ => self
                    .game_execution_scheduler
                    .in_flight_display_publication(),
            };
            let big_key_drop_graphics_slice = matches!(
                work_slice,
                GameWorkStep::Complete(GameWorkContinuation::FinishBigKeyDropGraphics { .. })
            ) || matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishBigKeyDropGraphics { .. })
            );
            if big_key_drop_graphics_slice {
                // Core DMA remains gated while the decompressor owns the main
                // stack, so OAM keeps the shadow published at entry.
                self.stage_big_key_drop_waiting_obj_scanout();
            }
            match work_slice {
                GameWorkStep::Waiting => {}
                GameWorkStep::Complete(GameWorkContinuation::FinishCpuInstructionNmi {
                    resume,
                }) => {
                    self.game_execution_scheduler
                        .schedule_pre_main_nmi_resume(resume);
                    let resumed = self.resume_after_pre_main_nmi(input, oam_dma_source.as_deref());
                    debug_assert!(resumed);
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractWorldMap) => {
                    self.complete_attract_scene_world_map();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractWorldMapExit) => {
                    self.complete_attract_world_map_exit();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapLightLoad) => {
                    self.world_map_load_light_world_map();
                    // TransferMode7Characters returns to WorldMap_LoadLightWorldMap,
                    // then through Module0E_Interface and ZeldaRunGameLoop. The
                    // measured return frame therefore publishes sprite DMA state
                    // and releases the software NMI latch just like an ordinary
                    // completed module iteration.
                    self.complete_module0e_interface_after_run();
                    if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven caller return already carries the
                        // shared ZeldaRunGameLoop suffix; retire its one owner.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishFluteMenuSelectedScreen {
                    step,
                }) => {
                    assert_eq!(
                        step,
                        FluteMenuSelectedScreenStep::SelectedScreenSuffix,
                        "the flute selected-screen caller returned before its sprite reload",
                    );
                    self.FluteMenu_LoadSelectedScreenAfterTransport();
                    if self.game_state.frame.main_module == 0x18 {
                        // Module18_GanonEmerges state 3 finishes its caller
                        // suffix and the module's LinkOam_Main on the
                        // returning host.
                        self.LoadOWMusicIfNeeded();
                        self.set_music_control(9);
                        self.link_oam_main();
                    }
                    if self.pending_main_loop_common_suffix.is_some() {
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDialogueInitializationPrefix {
                        caller_nmi_crossings,
                    },
                ) => {
                    self.complete_text_initialization_prefix();
                    self.prepare_text_character_buffer_for_carry();
                    if caller_nmi_crossings == 0 {
                        // The final interrupt inside Text_Initialize can land
                        // before Text_LoadCharacterBuffer, while the rest of
                        // that function and Module0E's caller still return to
                        // ZeldaRunGameLoop's main wait before the next NMI.
                        // Complete that synchronous C suffix in this resumed
                        // host slice; scheduling a zero-length continuation
                        // would invent an extra interrupt and host frame.
                        self.complete_post_trailing_nmi_continuation(
                            GameWorkContinuation::FinishDialogueInitializationCallerReturn,
                            input,
                            false,
                            authoritative_scheduled_caller_return_timeline.is_some(),
                        );
                        self.game_execution_scheduler
                            .finish_call_stack_at_main_wait_before_nmi();
                        let returned_scroll = self.bg_scroll_scanout_from_nmi_register_mirrors();
                        self.publish_bg_scroll_for_following_scanout(returned_scroll);
                        self.assert_native_frame_state_matches_ram();
                        self.assert_native_world_location_state_matches_ram();
                        self.assert_native_display_state_matches_ram();
                        return;
                    } else if caller_nmi_crossings == 1 {
                        self.game_execution_scheduler
                            .schedule_after_current_trailing_nmi(
                                GameWorkContinuation::FinishDialogueInitializationCallerReturn,
                            );
                    } else {
                        self.game_execution_scheduler
                            .schedule_work_before_trailing_nmi(
                                GameWorkContinuation::FinishDialogueInitializationCallerReturn,
                                caller_nmi_crossings,
                            );
                    }
                }
                GameWorkStep::Complete(
                    continuation @ GameWorkContinuation::FinishDialogueInitializationCallerReturn,
                ) => {
                    self.complete_post_trailing_nmi_continuation(continuation, input, false, false);
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDesertPrayerIris {
                    progress: _,
                    caller,
                }) => {
                if self.lane_finish_desert_prayer_iris(input, caller, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_nmi_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDesertPrayerPaletteFilter {
                    countdown,
                    next_color,
                }) => {
                if self.lane_finish_desert_prayer_palette_filter(input, countdown, next_color, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_nmi_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractThroneRoom) => {
                    self.complete_attract_scene_throne_room();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractZeldaPrison) => {
                    self.complete_attract_prep_zelda_prison();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractMaidenWarp) => {
                    self.complete_attract_prep_maiden_warp();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractEndOfStory) => {
                    self.complete_attract_scene_end_of_story();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishRescuedMaidenTilemapClear {
                    completed_stores,
                }) => {
                    // The preceding host's Held NMI interrupted the source
                    // store loop. Its handler publishes first; only then does
                    // the saved Module07_18 stack finish the remaining stores,
                    // clear the transition scratch, run Sprite_Main, and
                    // return through ZeldaRunGameLoop's shared suffix.
                    if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
                        self.capture_display_snapshot();
                        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    }
                    if let Some((_, _, sprite_main_return_claims)) =
                        authoritative_scheduled_caller_return_timeline.as_ref()
                    {
                        self.begin_original_timing_sprite_main_return_claim_scope(
                            *sprite_main_return_claims,
                        );
                    }
                    self.complete_rescued_maiden_tilemap_clear(completed_stores);
                    self.complete_module07_dungeon_after_submodule();
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                    if self.pending_main_loop_common_suffix.is_some() {
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    return;
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishRescuedMaidenInitialization { stage },
                ) => {
                if self.lane_finish_rescued_maiden_initialization(input, stage, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_nmi_timeline, authoritative_scheduled_caller_return_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonFallingEntrance {
                    work,
                }) => {
                    // Both long Module 11 calls return just after the final
                    // interrupted NMI. Preserve that ordering: the scanout and
                    // NMI consume the in-flight generation first; only then
                    // does the caller suffix publish work for the next
                    // boundary. In particular, the room loader's HUD/CGRAM
                    // requests must remain pending for the following frame.
                    if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
                        self.capture_display_snapshot();
                        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    }
                    match work {
                        DungeonFallingEntranceWork::RoomAndTilesets => {
                            self.complete_module11_02_load_entrance();
                        }
                        DungeonFallingEntranceWork::SpriteGraphics => {
                            self.DungeonTransition_LoadSpriteGFX();
                        }
                    }
                    // Both calls return through Module11 and the ordinary game
                    // loop suffix after their final interrupted NMI slice.
                    if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven caller return already carries the
                        // shared ZeldaRunGameLoop suffix; retire its one owner.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    return;
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonFallingRoomInitialization,
                ) => {
                if self.lane_finish_dungeon_falling_room_initialization(input, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_nmi_timeline, authoritative_scheduled_caller_return_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonSupertileTransition { work },
                ) => {
                if self.lane_finish_dungeon_supertile_transition(input, work, authoritative_link_oam_equipment_prefix, authoritative_scheduled_caller_nmi_timeline, authoritative_scheduled_caller_return_timeline, authoritative_supertile_sprite_main_returned, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishPreDungeonEntranceLoad {
                    sprite_reset,
                }) => {
                if self.lane_finish_pre_dungeon_entrance_load(input, sprite_reset, authoritative_pre_dungeon_progress, authoritative_pre_dungeon_returned, authoritative_scheduled_caller_return_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishPreDungeonSongBankTransfer) => {
                if self.lane_finish_pre_dungeon_song_bank_transfer(input, authoritative_pre_dungeon_progress, authoritative_pre_dungeon_returned, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishItemReceiptGraphics {
                    continuation,
                }) => {
                if self.lane_finish_item_receipt_graphics(input, continuation, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_return_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishBigKeyDropGraphics {
                    sprite_slot,
                    dungeon,
                    ..
                }) => {
                    let sprite_slot = usize::from(sprite_slot);
                    // Resume exactly where PrepareEnemyDrop was interrupted:
                    // finish the fixed big-key graphics call, its drop setup,
                    // the remaining lower sprite slots, and the Module 7
                    // sprite/camera suffix. The ordinary game-loop epilogue
                    // below then publishes the new OAM/CHR generation.
                    if let Some((_, _, sprite_main_return_claims)) =
                        authoritative_scheduled_caller_return_timeline.as_ref()
                    {
                        self.begin_original_timing_sprite_main_return_claim_scope(
                            *sprite_main_return_claims,
                        );
                    }
                    self.sprite_prep_big_key_load_graphics(sprite_slot);
                    self.complete_prepare_enemy_drop(sprite_slot);
                    self.complete_sprite_main_after_interrupted_slot(sprite_slot);
                    self.complete_module07_after_sprite_main(dungeon);
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                    self.retire_or_run_main_loop_common_suffix_after_module_return();
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonMapGraphicsPreparation,
                ) => {
                    // InitializeTilesets returns just after this interrupt. The
                    // active scanout is still the fully blank in-flight frame;
                    // only afterward does the dungeon-map caller publish its
                    // palettes/tilemaps and release the main-loop NMI latch.
                    if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
                        self.capture_display_snapshot();
                        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    }
                    self.complete_dungeon_map_graphics_preparation();
                    self.complete_module0e_interface_after_run();
                    if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven caller return already carries the
                        // shared ZeldaRunGameLoop suffix; retire its one owner.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonMapRoomDrawing) => {
                    // Most dungeon-map room planes remain in flight through
                    // this host entry. Preserve the interrupted scanout, then
                    // resume the routine and its ordinary Module0E suffix.
                    if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
                        self.capture_display_snapshot();
                        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    }
                    self.complete_dungeon_map_room_drawing();
                    self.complete_module0e_interface_after_run();
                    if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven caller return already carries the
                        // shared ZeldaRunGameLoop suffix; retire its one owner.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonMapRecovery) => {
                    // Tileset conversion and the room-quadrant rebuild return
                    // after this interrupt. The active scanout remains forced
                    // blank; publish the restored dungeon and audio state only
                    // through the resumed Module0E/game-loop suffix.
                    if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
                        self.capture_display_snapshot();
                        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    }
                    self.complete_dungeon_map_recovery();
                    self.complete_module0e_interface_after_run();
                    if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven caller return already carries the
                        // shared ZeldaRunGameLoop suffix; retire its one owner.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishTriforceRoomLoad { step }) => {
                if self.lane_finish_triforce_room_load(input, step, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_nmi_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonSubtilePaletteFilter) => {
                self.lane_finish_dungeon_subtile_palette_filter(authoritative_scheduled_caller_return_timeline);
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishStraightInterroomFadeoutSuffix,
                ) => {
                if self.lane_finish_straight_interroom_fadeout_suffix(input, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishStraightInterroomSpriteReset { progress },
                ) => {
                    if let Some((_, _, sprite_main_return_claims)) =
                        authoritative_scheduled_caller_return_timeline.as_ref()
                    {
                        self.begin_original_timing_sprite_main_return_claim_scope(
                            *sprite_main_return_claims,
                        );
                    }
                    self.complete_straight_interroom_sprite_reset_after_timing_boundary(progress);
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishSpriteMain {
                    boundary,
                    caller,
                }) => {
                if self.lane_finish_sprite_main_work(input, boundary, caller, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_nmi_timeline, authoritative_scheduled_caller_return_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity {
                        position_return,
                        iteration,
                    },
                ) => {
                    // This vblank began while Module0F was still inside
                    // Link_HandleVelocity. Its pre-existing OAM and Link CHR
                    // remain resident for the active scanout; the resumed
                    // LinkOam_Main/Main_PrepSpritesForNmi suffix feeds the
                    // following DMA boundary.
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                    self.complete_dungeon_exit_spotlight_link_velocity(position_return, iteration);
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity {
                        velocity_return,
                        iteration,
                    },
                ) => {
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                    self.complete_dungeon_exit_spotlight_link_actual_velocity(
                        velocity_return,
                        iteration,
                        authoritative_scheduled_caller_return_timeline.is_some(),
                    );
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        self.retire_or_run_main_loop_common_suffix_after_module_return();
                    }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement { iteration },
                ) => {
                    // The accepted NMI preceded Link_MovePosition's first
                    // coordinate publication. Resume the complete source leaf
                    // once, then let its LinkOam/Main_PrepSpritesForNmi suffix
                    // author the following DMA generation.
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                    self.complete_dungeon_exit_spotlight_link_movement(
                        iteration,
                        authoritative_scheduled_caller_return_timeline.is_some(),
                    );
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        // The wire's terminal return proves the shared
                        // ZeldaRunGameLoop suffix completes inside this host
                        // (route host 50636).
                        self.retire_or_run_main_loop_common_suffix_after_module_return();
                    }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel {
                        iteration,
                        pass,
                        pending_pixel_delta,
                        old_x,
                        old_y,
                    },
                ) => {
                    // The host ended inside Link_MovePosition's axis loop after
                    // one subpixel store (route host 179586). Finish that
                    // coordinate, the remaining axes and the caller suffix.
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                    self.complete_dungeon_exit_spotlight_link_movement_after_subpixel(
                        iteration,
                        LinkMovePositionPartialReturn {
                            old_x,
                            old_y,
                            partial: LinkMovePositionPartial {
                                axis: crate::game_state::PlayerAxis::from_rom_pass(pass),
                                pending_pixel_delta,
                            },
                        },
                        authoritative_scheduled_caller_return_timeline.is_some(),
                    );
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        self.retire_or_run_main_loop_common_suffix_after_module_return();
                    }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow {
                        iteration,
                        pass,
                        pending_coordinate_high,
                        old_x,
                        old_y,
                    },
                ) => {
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                    self.complete_dungeon_exit_spotlight_link_movement_after_coordinate_low(
                        iteration,
                        LinkMovePositionAfterCoordinateLowReturn {
                            old_x,
                            old_y,
                            axis: crate::game_state::PlayerAxis::from_rom_pass(pass),
                            pending_coordinate_high,
                        },
                        authoritative_scheduled_caller_return_timeline.is_some(),
                    );
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        self.retire_or_run_main_loop_common_suffix_after_module_return();
                    }
                }
                GameWorkStep::Complete(
                    work @ (GameWorkContinuation::FinishDungeonExitSpotlightLinkAndOam { iteration }
                    | GameWorkContinuation::FinishDungeonExitSpotlightControl { iteration }),
                ) => {
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                    if matches!(work, GameWorkContinuation::FinishDungeonExitSpotlightControl { .. }) {
                        self.complete_dungeon_exit_spotlight_control();
                    }
                    self.complete_dungeon_exit_spotlight_link_and_oam(
                        iteration,
                        authoritative_scheduled_caller_return_timeline.is_some(),
                    );
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        self.retire_or_run_main_loop_common_suffix_after_module_return();
                    }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates {
                        iteration,
                        pass,
                        old_x,
                        old_y,
                    },
                ) => {
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                    self.complete_dungeon_exit_spotlight_link_movement_after_coordinates(
                        iteration,
                        LinkMovePositionAfterCoordinatesReturn { old_x, old_y, axis: crate::game_state::PlayerAxis::from_rom_pass(pass) },
                        authoritative_scheduled_caller_return_timeline.is_some(),
                    );
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        self.retire_or_run_main_loop_common_suffix_after_module_return();
                    }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonPushBlocks { dungeon }) => {
                    self.resume_dungeon_push_blocks_caller(dungeon);
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonPushBlockHandler {
                    handler_pending,
                }) => {
                    self.resume_dungeon_push_block_handler(handler_pending);
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonCachedSpriteMain {
                    boundary,
                    live_slot_backup,
                    dungeon,
                }) => {
                    // The interrupting acceptance can refine which exact
                    // UncacheAndExecuteSprite statement the source suspended
                    // on; the typed receipt supersedes the estimate boundary
                    // (route host 31288).
                    let boundary =
                        match self.take_original_timing_cached_sprite_execution_progress() {
                            Some(receipt) => {
                                assert_eq!(
                                receipt.boundary,
                                OriginalTimingBoundary::NmiAccepted,
                                "a cached-sprite completion may refine only at its accepting NMI",
                            );
                                receipt.progress.into()
                            }
                            None => boundary,
                        };
                    self.complete_cached_sprite_main_after_interrupted_slot(
                        boundary,
                        &live_slot_backup,
                    );
                    self.complete_module07_after_sprite_main(dungeon);
                    if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven caller return already carries the
                        // shared ZeldaRunGameLoop suffix; retire its one owner.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    self.stage_resumed_sprite_main_return_obj_scanout();
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishSpiralStaircasePaletteFilter { .. },
                ) => {
                    unreachable!("spiral palette completion handled before generic publication")
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonExitSpotlightEntry {
                    table_build,
                    iteration,
                }) => {
                self.lane_finish_dungeon_exit_spotlight_entry(table_build, iteration, authoritative_dungeon_exit_spotlight_entry_iteration_returned, authoritative_scheduled_caller_nmi_timeline, authoritative_scheduled_caller_return_timeline);
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                    table_build,
                    projection_completed,
                    iteration,
                }) => {
                self.lane_finish_dungeon_exit_spotlight_build(table_build, projection_completed, iteration, authoritative_dungeon_exit_spotlight_caller_returned, authoritative_dungeon_exit_spotlight_link_oam_interruption, authoritative_dungeon_exit_spotlight_same_host_iteration, authoritative_scheduled_caller_interrupted_fresh_iteration, prospective_spotlight_build_link_oam_plan);
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishOverworldSpotlightBuild {
                    table_build,
                    phase,
                    projection_completed,
                    iteration,
                }) => {
                self.lane_finish_overworld_spotlight_build(table_build, phase, projection_completed, iteration, authoritative_overworld_spotlight_same_host_iteration, authoritative_scheduled_caller_interrupted_fresh_iteration, authoritative_scheduled_caller_nmi_timeline);
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { .. },
                ) => {
                    self.complete_dungeon_exit_spotlight_goal_caller();
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                        // The resumed Module0F caller has now reached the same
                        // NMI_PrepareSprites suffix as ZeldaRunGameLoop in C.
                        // Publish that complete sorted shadow as one DMA
                        // generation; a pre-main image was only a substitute
                        // for the omitted caller suffix.
                        oam: OamScanoutSource::ComposeLiveShadowAfterMain,
                        link_obj: GraphicsDmaGeneration::LiveAfterMain,
                        link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
                    }));
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishGameOverSpotlightBuild {
                    table_build,
                    entry,
                    iteration,
                }) => {
                self.lane_finish_game_over_spotlight_build(table_build, entry, iteration, authoritative_scheduled_caller_nmi_timeline, authoritative_scheduled_caller_return_timeline);
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishGameOverIrisGoalPaletteFill { completed_stores },
                ) => {
                    assert!(
                        authoritative_scheduled_caller_return_timeline.is_some(),
                        "the game-over goal palette continuation completed without its source caller return",
                    );
                    self.complete_game_over_iris_goal_palette_fill(completed_stores);
                    self.complete_pending_main_loop_common_suffix_after_module_return();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishGameOverDeathAfterSpriteReset { count_as_death }) => {
                    assert!(authoritative_scheduled_caller_return_timeline.is_some(),
                        "Game Over's reset continuation requires its source caller return");
                    self.complete_game_over_death_after_sprite_reset(count_as_death);
                    self.complete_pending_main_loop_common_suffix_after_module_return();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishSpotlightIteration {
                    ..
                }) => {
                self.lane_finish_spotlight_iteration(authoritative_dungeon_exit_spotlight_same_host_iteration, authoritative_scheduled_caller_interrupted_fresh_iteration);
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishOverworldSpotlightLinkOam {
                    ..
                }) => {
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                    self.nmi_prepare_sprites_for_main_loop_once();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishOverworldSpotlightGoalResetTable {
                        completed_stores,
                        ..
                    },
                ) => {
                    self.complete_overworld_spotlight_goal_reset_table(completed_stores);
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { .. },
                ) => {
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                    self.nmi_prepare_sprites_for_main_loop_once();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapExitTilesets) => {
                    // InitializeTilesets has returned through WorldMap_ExitMap
                    // and Module0E_Interface. Publish the same caller suffix
                    // that an uninterrupted game-loop iteration would reach.
                    self.complete_world_map_exit_after_tileset_load();
                    self.complete_module0e_interface_after_run();
                    let wire_interrupts_sprite_preparation =
                        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                            && authoritative_scheduled_caller_return_timeline.is_none()
                            && authoritative_scheduled_caller_nmi_timeline.is_some_and(
                                |timeline| {
                                    timeline.progress == crate::MainLoopProgress::CallStackContinued
                                        && timeline.interruption
                                            == crate::MainLoopInterruption::SpritePreparation
                                },
                            );
                    if wire_interrupts_sprite_preparation {
                        // The C translation above performed the dialogue close
                        // the wire corroborates; the shared suffix (whose
                        // NMI_PrepareSprites the wire interrupted) completes on
                        // the next host (route host 1230513).
                        let _ = self.take_original_timing_dialogue_closed();
                        let _ = self.take_original_timing_main_loop_interruption_any();
                        self.retire_or_defer_main_loop_common_suffix_by_wire();
                    } else if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven caller return already carries the
                        // shared ZeldaRunGameLoop suffix; retire its one owner.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishOverworldLoadOverlaysSpriteReload,
                ) => {
                if self.lane_finish_overworld_load_overlays_sprite_reload(authoritative_scheduled_caller_return_timeline) {
                    return;
                }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishOverworldLoadOverlaysOverlay,
                ) => {
                if self.lane_finish_overworld_load_overlays_overlay(input, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_return_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapOverlayReload) => {
                if self.lane_finish_world_map_overlay_reload(authoritative_scheduled_caller_return_timeline) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapAmbientMap8) => {
                    let schedule = self
                        .module09_cpu_schedule
                        .take()
                        .expect("Module09/$21 completion lost its ROM CPU schedule");
                    assert_eq!(schedule.caller_nmis, 0);
                    assert_eq!(schedule.caller_sprite_main_nmis, 0);
                    assert_eq!(schedule.caller_suffix_nmis, 0);
                    assert_eq!(schedule.sprite_main_boundary, None);
                    if let Some((_, _, sprite_main_return_claims)) =
                        authoritative_scheduled_caller_return_timeline.as_ref()
                    {
                        self.begin_original_timing_sprite_main_return_claim_scope(
                            *sprite_main_return_claims,
                        );
                    }
                    self.Overworld_LoadAmbientOverlay(false);
                    self.complete_module09_overworld_after_submodule();
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                    if self.pending_main_loop_common_suffix.is_some() {
                        // The source-proven caller return already carries the
                        // shared ZeldaRunGameLoop suffix; retire its one owner.
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                }
                GameWorkStep::Complete(
                    work @ (GameWorkContinuation::FinishOverworldAuxGraphics
                    | GameWorkContinuation::FinishOverworldMosaicSpriteGraphics
                    | GameWorkContinuation::FinishOverworldSpecialExitMosaic
                    | GameWorkContinuation::FinishOverworldSpecialExitMosaicSecondDecode),
                ) => {
                if self.lane_finish_overworld_aux_graphics(input, work, authoritative_overworld_special_exit_mosaic_restored, authoritative_overworld_special_exit_mosaic_returned, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_nmi_timeline, authoritative_scheduled_caller_return_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishModule09LongLoad { step }) => {
                if self.lane_finish_module09_long_load(input, step, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_nmi_timeline, authoritative_scheduled_caller_return_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishOverworldMapQuadrants {
                    scroll_map_and_sprite_gfx_tail_nmi_slices,
                }) => {
                    // SomeTileMapChange increments the submodule before the
                    // remaining screen-map build and sprite conversion return.
                    // Publish that CPU-visible generation after this vblank,
                    // then keep the caller stack suspended for its measured
                    // four-boundary tail.
                    if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
                        self.capture_display_snapshot();
                        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    }
                    self.complete_module09_load_new_map_quadrants();
                    self.game_execution_scheduler.schedule_work(
                        GameWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail,
                        scroll_map_and_sprite_gfx_tail_nmi_slices,
                    );
                    return;
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail,
                ) => {
                if self.lane_finish_overworld_screen_map_and_sprite_graphics_tail(input, authoritative_scheduled_caller_accepts_nmi_at_return, authoritative_scheduled_caller_return_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishOverworldSpriteReloadTail {
                    post_return_hold_nmi_slices,
                    return_phase: _,
                    epilogue_phase,
                    resume_scanout,
                }) => {
                if self.lane_finish_overworld_sprite_reload_tail(input, post_return_hold_nmi_slices, epilogue_phase, resume_scanout, authoritative_scheduled_caller_return_timeline, oam_dma_source.clone()) {
                    return;
                }
                }
                GameWorkStep::Complete(GameWorkContinuation::HoldOverworldSpriteReloadReturn) => {
                    // The light sprite loader returns at V=213, so its camera
                    // and caller suffix are already visible. Snes9x remains in
                    // submodule 5 for the following scanout, however; the next
                    // Overworld_StartScrollTransition call lands at V=255,
                    // after that image has been emitted. Hold only that next
                    // main-loop iteration while still running the frame NMI.
                    self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                        PreMainNmiResume::OverworldSpriteReloadReturn {
                            scanout: OverworldSpriteReloadResumeScanout::ByReturnPhase(
                                NmiPhase::BeforeNmi,
                            ),
                        },
                    );
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::PreOverworldPropertiesSpriteReset { .. }
                    | GameWorkContinuation::FinishPreOverworldProperties { .. }
                    | GameWorkContinuation::FinishPreOverworldOverlays
                    | GameWorkContinuation::FinishPreOverworldScreenBuild
                    | GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn
                    | GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn
                    | GameWorkContinuation::FinishModule09LinkOamCallerReturn { .. }
                    | GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn { .. },
                ) => {
                    unreachable!(
                        "post-NMI caller return is handled before generic work publication"
                    )
                }
            }
            if let Some(timeline) = authoritative_scheduled_caller_nmi_timeline
                .filter(|timeline| timeline.progress == crate::MainLoopProgress::IterationStarted)
            {
                let fresh_iteration_plan = prospective_interrupted_idle_main_loop_plan
                    .as_ref()
                    .filter(|plan| plan.scheduled_predecessor.is_some())
                    .expect("a scheduled caller-to-fresh-iteration transition lost its immutable host plan");
                assert_eq!(fresh_iteration_plan.timeline, *timeline);
                assert!(
                    matches!(work_slice, GameWorkStep::Complete(_)),
                    "a fresh main iteration cannot begin before its suspended caller completes: {timeline:?}",
                );
                assert!(
                    self.game_execution_scheduler.is_idle(),
                    "a fresh main iteration cannot overlap a replacement native continuation: work={:?} timeline={timeline:?}",
                    self.game_execution_scheduler.current_work(),
                );

                // The source caller returned, then ZeldaRunGameLoop began again
                // and reached the reported phase in this same host interval.
                // Re-enter the translated loop from that semantic receipt; the
                // backend's PC and raster remain private to the timing owner.
                self.game_execution_scheduler
                    .prepare_same_host_main_iteration_after_authoritative_return();
                self.forward_original_timing_main_loop_interruption_to_native_owner(
                    timeline.interruption,
                    if authoritative_scheduled_caller_accepts_nmi_at_return
                        .expect("fresh scheduled-caller timeline lost its return boundary")
                    {
                        OriginalTimingBoundary::NmiAccepted
                    } else {
                        OriginalTimingBoundary::HostReturn
                    },
                );
                self.begin_original_timing_sprite_main_return_claim_scope(
                    fresh_iteration_plan.sprite_main_returned_claims,
                );
                self.zelda_run_game_loop_with_progress(Some(timeline.progress));
                self.finish_original_timing_sprite_main_return_claim_scope();
                assert!(
                    !self.original_timing_main_loop_interruption_is_pending(),
                    "fresh main iteration has no semantic owner for {timeline:?}: frame={:?} work={:?} dungeon_advance={:?}",
                    self.game_state.frame,
                    self.game_execution_scheduler.current_work(),
                    self.dungeon_landing_cpu_advance_pending,
                );
                self.stage_pending_dialogue_scroll_completion_after_captured_boundary();

                if authoritative_scheduled_caller_leading_nmi_completion.completed() {
                    // The completed leading handler DMAed the host-boundary OAM
                    // shadow before this fresh main slice authored its successor.
                    // Link's CHR upload uses the same pre-main operands. Preserve
                    // those independent hardware generations while the source is
                    // suspended in LinkOam or NMI_PrepareSprites.
                    self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                        oam: OamScanoutSource::ComposeLiveAfterNmi,
                        link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                        link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    }));
                }
            }
            // The original ROM returns to the NMI boundary after the final
            // main-thread work slice. Attract loaders and item graphics both
            // publish only after their measured continuation completes.
            let item_receipt_graphics_slice = matches!(
                work_slice,
                GameWorkStep::Complete(GameWorkContinuation::FinishItemReceiptGraphics { .. })
            ) || matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishItemReceiptGraphics { .. })
            );
            if item_receipt_graphics_slice {
                // The main-loop latch holds OAM, Link OBJ, and ordinary tilemap
                // publication while the decompressor is interrupted. Dungeon
                // animated tiles are an independent NMI DMA domain and still
                // upload from the just-authored buffer before this scanout is
                // captured.
                let graphics_dma_plan = rom_graphics_dma_plan(
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule,
                );
                self.nmi_core_animated_bg_update(graphics_dma_plan);
            }
            if self.dungeon_quadrant_cpu_continuation_active
                && self.game_execution_scheduler.is_idle()
                && self
                    .game_execution_scheduler
                    .resumed_call_stack_is_before_nmi()
                && self.active_dungeon_sprite_main_return.is_none()
                && !self.game_state.display.nmi_update_is_latched()
            {
                // The interrupting NMI was consumed by a preceding host. This
                // resumed caller has now cleared the software latch and
                // reached the main wait, so appending the generic trailing NMI
                // would manufacture a second boundary. Preserve the active
                // scanout and carry an exact shadow of the next leading-NMI
                // iteration instead.
                self.dungeon_quadrant_cpu_continuation_active = false;
                self.prepare_dungeon_cpu_advance_after_returned_main_wait();
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return;
            }
            if let GameWorkStep::Complete(
                GameWorkContinuation::FinishSpotlightIteration { iteration }
                | GameWorkContinuation::FinishOverworldSpotlightBuild { iteration, .. }
                | GameWorkContinuation::FinishOverworldSpotlightLinkOam { iteration, .. }
                | GameWorkContinuation::FinishOverworldSpotlightGoalResetTable { iteration, .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightBuild { iteration, .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { iteration }
                | GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity {
                    iteration, ..
                }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement { iteration, .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel {
                    iteration,
                    ..
                }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow {
                    iteration,
                    ..
                }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates {
                    iteration,
                    ..
                }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity { iteration, .. },
            ) = work_slice
            {
                if let Some(following) = iteration.rom_following_field_receipt() {
                    let receipt = LiveSpotlightScanout::capture(self)
                        .with_authoritative_rom_hdma_words(&following.words);
                    match following.publication {
                        SpotlightFollowingFieldPublication::WithCompletionCapture => {
                            debug_assert!(self.next_display_spotlight_scanout.is_none());
                            self.next_display_spotlight_scanout = Some(receipt);
                        }
                        SpotlightFollowingFieldPublication::AfterCompletionCapture => {
                            self.spotlight_scanout_after_active_field = Some(receipt);
                        }
                    }
                }
            }
            self.capture_display_snapshot_with_override(publication_override);
            if let GameWorkStep::Complete(
                GameWorkContinuation::FinishOverworldSpotlightBuild { iteration, .. }
                | GameWorkContinuation::FinishOverworldSpotlightLinkOam { iteration }
                | GameWorkContinuation::FinishOverworldSpotlightGoalResetTable { iteration, .. },
            ) = work_slice
            {
                if iteration.completed_hdma_table_owns_active_scanout() {
                    let active_table = self.hdma_dynamic_table_bytes();
                    self.publish_completed_spotlight_hdma_table_to_active_scanout(active_table);
                }
            }
            if let GameWorkStep::Complete(GameWorkContinuation::FinishGameOverSpotlightBuild {
                iteration,
                ..
            }) = work_slice
            {
                if iteration.projects_following_table_tail_on_completion() {
                    self.project_following_spotlight_tail_to_active_scanout(
                        iteration.phase,
                        iteration.projection_uses_published_prefix(),
                    );
                }
            }
            if let GameWorkStep::Complete(
                GameWorkContinuation::FinishSpotlightIteration { iteration }
                | GameWorkContinuation::FinishOverworldSpotlightBuild { iteration, .. }
                | GameWorkContinuation::FinishOverworldSpotlightLinkOam { iteration, .. }
                | GameWorkContinuation::FinishOverworldSpotlightGoalResetTable { iteration, .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightBuild { iteration, .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { iteration }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement { iteration, .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel {
                    iteration,
                    ..
                }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow {
                    iteration,
                    ..
                }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates {
                    iteration,
                    ..
                }
                | GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity { iteration, .. },
            ) = work_slice
            {
                if iteration.projects_following_table_tail_on_completion() {
                    self.project_following_spotlight_tail_to_active_scanout(
                        iteration.phase,
                        iteration.projection_uses_published_prefix(),
                    );
                }
            }
            // The scheduled caller return reaches the ordinary trailing NMI.
            // C's ZeldaRunGameLoop calls NMI_PrepareSprites after the module
            // returns and before clearing $12. If that suffix completed above,
            // the hardware OAM DMA consumes the shadow it just packed; the
            // host-boundary copy predates ClearOamBuffer and belongs only to an
            // interrupt which occurred before the saved caller returned.
            let trailing_oam_dma_source = if self.main_loop_sprite_preparation_completed {
                None
            } else {
                oam_dma_source.as_deref()
            };
            if authoritative_scheduled_caller_completes_nmi_after_same_host_iteration {
                self.interrupt_nmi(input, trailing_oam_dma_source, false);
                self.apply_original_timing_joypad_publication();
            }
            if let Some(accepts_nmi_at_return) =
                authoritative_scheduled_caller_accepts_nmi_at_return
            {
                // The saved C caller has advanced and cleared the software
                // latch. Its ordered source receipt is authoritative about
                // whether this host also reaches the next NMI acceptance. If
                // it does, the source returns at that handler entry and none
                // of the NMI publication belongs to this host. If it does not,
                // return after the completed NMI publication without fabricating the
                // ordinary translated trailing NMI.
                debug_assert_eq!(
                    self.original_timing_scheduled_nmi_accepted_at_host_return,
                    accepts_nmi_at_return,
                );
                return;
            }
            if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && !self.original_timing_nmi_publication_pending
                && self.original_timing_expected_nmi_update_gates.is_empty()
            {
                // The wire crossed this host interval without any NMI (a
                // forced-blank load lag, route host 58565); fabricating the
                // ordinary trailing NMI would execute an acceptance the
                // source never published.
                return;
            }
            self.interrupt_nmi(input, trailing_oam_dma_source, false);
            self.stage_suspended_dungeon_submodule_after_nmi();
            if scheduled_work_completion_clears_nmi_latch_after_interrupt(work_slice) {
                self.clear_nmi_update_latch();
            }
            self.restore_room_61_sprite_conversion_resident_oam();
            return;
        }
        if self.lane_main_loop_interruption_timeline(
            input,
            authoritative_dungeon_exit_spotlight_caller_is_active,
            authoritative_main_loop_interruption_timeline.clone(),
            oam_dma_source.clone(),
            prospective_interrupted_idle_main_loop_plan,
        ) {
            return;
        }
        if self.lane_rom_startup_run_main(input, run_what, frame, oam_dma_source.clone()) {
            return;
        }
        if run_what & crate::RUN_MAIN != 0 {
            self.replay_trace_ram_watch("before-game-loop");
            self.zelda_run_game_loop();
            self.replay_trace_ram_watch("after-game-loop");
        }
        if matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishItemReceiptGraphics { .. })
        ) {
            // The decompressor's entry slice has already set the software NMI
            // latch, but the vblank that interrupts it still completes the
            // independently wired graphics DMAs. Link consumes the operands
            // captured at the host boundary before the atomic caller entered
            // the decompressor, while dungeon animated BG follows the main
            // slice that advanced its source address and consumes live operands.
            let captured_link_operands = self
                .pre_main_graphics_dma
                .as_ref()
                .map(|graphics| graphics.link_operands);
            self.nmi_core_link_graphics_update(captured_link_operands);
            self.link_obj_dma_completed_this_frame = true;
            let mut graphics_dma_plan = rom_graphics_dma_plan(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            );
            graphics_dma_plan.animated_bg_operands = GraphicsDmaGeneration::LiveAfterMain;
            self.nmi_core_animated_bg_update(graphics_dma_plan);
        }
        let publication_override = self
            .game_execution_scheduler
            .in_flight_display_publication();
        // The circle builder is suspended across this vblank. Its WRAM table
        // is CPU-visible, but HDMA has already consumed the preceding staged
        // generation for the scanout that ends here.
        self.capture_display_snapshot_with_override(publication_override);
        let effective_leading_graphics_dma = self.rom_startup_timing()
            && (self
                .game_execution_scheduler
                .eligible_leading_nmi_preceded_suspended_work()
                || self
                    .game_execution_scheduler
                    .main_call_stack_is_suspended_before_nmi())
            && self.game_state.display.nmi_update_is_latched();
        if effective_leading_graphics_dma {
            // The ROM interval begins with NMI, then runs main until this caller
            // is interrupted. The atomic port reaches the same boundary in the
            // opposite order and still carries the preceding NMI's software
            // latch, so the ordinary handler below cannot expose the hardware
            // graphics transfers. Record the real leading OAM and Link DMAs
            // explicitly from their shared pre-main operand capture. Their raw
            // writes remain after the immutable scanout capture, while the
            // effective receipt attaches them to the scanout which consumed
            // those transfers.
            let oam_shadow = self
                .pre_main_graphics_dma
                .as_ref()
                .map(|graphics| graphics.oam_shadow.clone());
            let operands = self
                .pre_main_graphics_dma
                .as_ref()
                .map(|graphics| graphics.link_operands);
            self.begin_effective_presented_dma();
            if let Some(oam_shadow) = oam_shadow.as_deref() {
                self.complete_oam_dma_from_source(oam_shadow);
            }
            self.nmi_core_link_graphics_update(operands);
            self.link_obj_dma_completed_this_frame = true;
            assert!(
                self.record_effective_presented_dma_for_active_scanout()
                    .is_none(),
                "a Link/OAM-only receipt cannot publish dialogue text DMA",
            );
        }
        self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
        self.replay_trace_ram_watch("before-nmi");
        let defer_interface_exit_bg_upload = interface_exit_bg_upload_misses_current_scanout(
            frame.main_module,
            self.game_state.frame.main_module,
            self.game_state.display.has_bg_vram_load(),
        );
        let defer_room_82_sprite_conversion_nmi = self.rom_startup_timing()
            && run_what & crate::RUN_MAIN != 0
            && rom_room_82_sprite_conversion_defers_trailing_nmi(
                frame,
                self.game_state.frame,
                self.game_state.world.location.dungeon_room_index(),
            );
        if defer_room_82_sprite_conversion_nmi {
            if let Some(snapshot) = self.display_snapshot.as_mut() {
                snapshot.room_82_sprite_conversion_deferred_nmi = true;
            }
        }
        let resumed_main_is_waiting_for_nmi = self
            .game_execution_scheduler
            .returned_main_is_waiting_for_nmi();
        self.lane_trailing_nmi_without_deferral(
            input,
            defer_interface_exit_bg_upload,
            defer_room_82_sprite_conversion_nmi,
            frame,
            oam_dma_source.clone(),
            resumed_main_is_waiting_for_nmi,
        );
        if faded_filter_palette_completion_clears_nmi_latch_after_interrupt(
            self.dungeon_faded_filter_palette_completion_host_frame,
            self.frame_ctr_dbg,
            self.game_execution_scheduler.is_idle(),
        ) {
            // The penultimate landing palette pass returns after this trailing
            // NMI. Its ordinary caller epilogue clears $12 after hardware has
            // consumed the pending update. The final pass remains scheduled
            // and releases the same latch in its dedicated continuation.
            self.clear_nmi_update_latch();
        }
        self.replay_trace_ram_watch("after-nmi");
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        self.sync_overworld_map16_state_from_ram();
    }

    pub(super) fn rom_triforce_poly_render_hosts(&mut self) -> Option<(u8, u64)> {
        // Both polyhedral threads (Triforce room $FF=$90, dungeon crystal
        // maiden $FF=$30) run the same $09:F81D loop; the V-IRQ line the ROM
        // programmed bounds the main thread's slot [virq, 225).
        let virq_scanline = match self.ram[VIRQ_TRIGGER] {
            line @ 1..=224 => u16::from(line),
            _ => 144,
        };
        // The thread resumes when the NMI handler swaps stacks; slot 1's
        // handler also uploads the frame the thread just finished.
        let swap_with_upload = self.rom_triforce_nmi_swap_master_cycles(true)?;
        let swap_without_upload = self.rom_triforce_nmi_swap_master_cycles(false)?;
        let swap_entry = |master: u64| -> (u16, u16, bool) {
            // Master cycles after the NMI acceptance at line 225 (plus refresh).
            let master = master * 1364 / 1324;
            let lines = 225 + (master / 1364) as u16;
            let cycle = (master % 1364) as u16;
            if lines >= 262 {
                (lines - 262, cycle, true)
            } else {
                (lines, cycle, false)
            }
        };
        // Thread-owned RAM: the poly work area past the main-written
        // parameters ($1F00-$1F0F) and the bitmap buffer at $7E:E800.
        const THREAD_WORK_RANGE: std::ops::Range<usize> = 0x1f10..0x2000;
        const THREAD_BITMAP_RANGE: std::ops::Range<usize> = 0xe800..0xf000;
        let timing_dma = self.dma_with_native_hdma_enable();
        let mut ram_image = self.ram.to_vec();
        if let Some(saved) = self.triforce_poly_shadow_ram.as_ref() {
            ram_image[THREAD_WORK_RANGE].copy_from_slice(&saved[THREAD_WORK_RANGE]);
            ram_image[THREAD_BITMAP_RANGE].copy_from_slice(&saved[THREAD_BITMAP_RANGE]);
        }
        let mut run = RomCpuTimingRun::new(
            &self.rom,
            &ram_image,
            &self.sram,
            &self.ppu,
            &timing_dma,
            self.zelda_audio_apu_output_ports(),
            TRIFORCE_ROOM_POLY_RENDER_CPU_CHECKPOINT,
        )
        .ok()?;
        let mut master = 0u64;
        let mut hosts = 0u8;
        let mut even_field = self.frame_ctr_dbg & 1 == 0;
        let mut steps = 0u32;
        while hosts < 16 {
            hosts += 1;
            // Slot 1 follows the upload NMI; later slots follow an NMI
            // without the poly upload. Both are measured on the ROM's own
            // handler so the swap line (239.9 bare, up to line 0 of the next
            // field with the text and poly DMAs) is exact.
            let (scanline, cycle, wrapped) = if hosts == 1 {
                swap_entry(swap_with_upload)
            } else {
                swap_entry(swap_without_upload)
            };
            if wrapped {
                even_field = !even_field;
            }
            let entry = CpuRasterPosition::new(scanline, cycle);
            let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
                entry,
                CpuBusWorkload::with_dynamic_hdma(),
                CpuFieldTiming::non_interlace(even_field),
            );
            if crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some() && hosts == 1 {
                eprintln!(
                    "[POLY-NMI] host={} swap_upload_master={} swap_bare_master={} slot1_entry={}:{}",
                    self.frame_ctr_dbg, swap_with_upload, swap_without_upload, scanline, cycle
                );
            }
            loop {
                if run.is_complete() {
                    self.triforce_poly_shadow_ram = Some(run.ram().to_vec());
                    return Some((hosts, master));
                }
                steps += 1;
                if steps > 4_000_000 {
                    return None;
                }
                let before = budget.raster_position();
                let advance = advance_rom_cpu_step(&mut run, &mut budget);
                let after = budget.raster_position();
                let (s0, c0) = before.coordinates();
                let (s1, c1) = after.coordinates();
                let delta =
                    (i64::from(s1) - i64::from(s0)) * 1364 + (i64::from(c1) - i64::from(c0));
                if delta > 0 {
                    master += delta as u64;
                }
                let (scanline, _) = after.coordinates();
                if advance.reached_boundary().is_some()
                    || (scanline >= virq_scanline && scanline < 225)
                {
                    break;
                }
            }
            even_field = !even_field;
        }
        None
    }

    /// Field offsets (from the current host) of the hosts whose V-IRQ slice
    /// the crystal poly thread does NOT take: the thread disables its IRQ when
    /// a frame completes and the NMI re-enables it only after the maiden's
    /// next iteration requests a frame, so the completion host's successor
    /// runs the main thread whole (route host 415939: the second dialogue's
    /// caller returned inside such a host).
    pub(super) fn dungeon_poly_thread_free_host_offsets(&self) -> u64 {
        let period = u32::from(self.poly_dungeon_current_frame_slices.max(1));
        let first = if self.poly_job_in_flight {
            u32::from(self.poly_job_hold_frames) + 1
        } else {
            period
        };
        let mut mask = 0u64;
        let mut offset = first;
        while offset < 64 {
            mask |= 1 << offset;
            offset += period;
        }
        mask
    }

    /// Advance the in-flight shadow render through this host's thread slot:
    /// from the NMI swap measured at this host's NMI to the V-IRQ line
    /// (`$FF`). Returns `true` when the render completed within the slot.
    pub(super) fn advance_poly_shadow_host(&mut self) -> bool {
        const THREAD_WORK_RANGE: std::ops::Range<usize> = 0x1f10..0x2000;
        const THREAD_BITMAP_RANGE: std::ops::Range<usize> = 0xe800..0xf000;
        let virq_scanline = match self.ram[VIRQ_TRIGGER] {
            line @ 1..=224 => u16::from(line),
            _ => 144,
        };
        // Measure this host's NMI swap on the top-of-host RAM (the DMA
        // queues the handler drained are what the ROM's handler saw), with
        // the latch and upload bytes as they were at that NMI.
        let (_, upload) = self
            .poly_next_host_nmi_state
            .take()
            .unwrap_or((None, false));
        let latch = self
            .poly_receipt_gates_prev
            .and_then(|(_, trailing)| trailing)
            .or_else(|| self.poly_receipt_gates_cur.and_then(|(leading, _)| leading))
            .unwrap_or_else(|| self.game_state.display.nmi_update_is_latched());
        let Some(swap) = self.rom_nmi_swap_master_cycles(upload, Some(latch)) else {
            return true;
        };
        self.poly_next_host_swap_master = Some(swap);
        let Some(mut run) = self.poly_shadow_run.take() else {
            return true;
        };
        self.poly_shadow_hosts = self.poly_shadow_hosts.saturating_add(1);
        // On the frame's first slot the resumed thread still runs one idle
        // loop check ($09:F81D LDA/BEQ/LDA/BNE, 10 cycles) before the render
        // entry at $09:F825 where the checkpoint begins.
        const IDLE_LOOP_CHECK_MASTER_CYCLES: u64 = 64;
        let swap = if self.poly_shadow_hosts == 1 {
            swap + IDLE_LOOP_CHECK_MASTER_CYCLES
        } else {
            swap
        };
        let master = swap * 1364 / 1324;
        let mut scanline = 225 + (master / 1364) as u16;
        let cycle = (master % 1364) as u16;
        let mut even_field = self.frame_ctr_dbg & 1 == 0;
        if scanline >= 262 {
            scanline -= 262;
            even_field = !even_field;
        }
        // A poly upload can keep the NMI handler in flight across line zero
        // and past HDMA start. The pinned source then resumes the thread with
        // channel 7 at the tail left after the prior field's 225 visible
        // lines plus the current line-zero transfer. A newly-created isolated
        // CPU run otherwise starts from the static channel descriptor and
        // wrongly charges a fresh 240-line table (crystal maiden host 677596).
        if self.poly_shadow_hosts == 1
            && upload
            && scanline == 0
            && u32::from(cycle) >= HDMA_START_CYCLE
        {
            run.seed_hdma_after_upload_crossed_line_zero();
        }
        let entry = CpuRasterPosition::new(scanline, cycle);
        let mut budget = CpuCycleBudget::until_next_nmi_acceptance(
            entry,
            CpuBusWorkload::with_dynamic_hdma(),
            CpuFieldTiming::non_interlace(even_field),
        );
        let debug = crate::debug_env::var_os("ZELDA3_DEBUG_POLY").is_some();
        for counter in &ROM_CPU_SHADOW_HDMA_DEBUG {
            counter.store(0, std::sync::atomic::Ordering::Relaxed);
        }
        let mut steps = 0u32;
        let mut mvn_steps = 0u32;
        let mut mvn_master = 0u64;
        let mut mvn_first = None;
        let mut mvn_last = None;
        let mut mvn_deltas = std::collections::BTreeMap::<i64, u32>::new();
        let mut pc_counts = [0u32; 5];
        let completed = loop {
            if run.is_complete() {
                let mut saved = self
                    .triforce_poly_shadow_ram
                    .take()
                    .unwrap_or_else(|| self.ram.to_vec());
                saved[THREAD_WORK_RANGE].copy_from_slice(&run.ram()[THREAD_WORK_RANGE]);
                saved[THREAD_BITMAP_RANGE].copy_from_slice(&run.ram()[THREAD_BITMAP_RANGE]);
                self.triforce_poly_shadow_ram = Some(saved);
                break true;
            }
            steps += 1;
            if steps > 4_000_000 {
                break true;
            }
            let before = budget.raster_position();
            let step_pc = run.pc();
            let advance = advance_rom_cpu_step(&mut run, &mut budget);
            let after = budget.raster_position();
            let (s0, c0) = before.coordinates();
            let (s1, c1) = after.coordinates();
            let delta = (i64::from(s1) - i64::from(s0)) * 1364 + (i64::from(c1) - i64::from(c0));
            if delta > 0 {
                self.poly_shadow_master += delta as u64;
            }
            if debug {
                match step_pc {
                    0x09_fd18 => {
                        mvn_first.get_or_insert((s0, c0));
                        mvn_last = Some((s0, c0));
                        mvn_steps += 1;
                        mvn_master += delta.max(0) as u64;
                        *mvn_deltas.entry(delta).or_default() += 1;
                    }
                    0x09_fd74 => pc_counts[0] += 1,
                    0x09_fdcf => pc_counts[1] += 1,
                    0x09_feb4 => pc_counts[2] += 1,
                    0x09_ff1e => pc_counts[3] += 1,
                    0x09_fe27 => pc_counts[4] += 1,
                    _ => {}
                }
            }
            let (now, _) = after.coordinates();
            if advance.reached_boundary().is_some() || (now >= virq_scanline && now < 225) {
                break false;
            }
        };
        if debug {
            let (now, cyc) = budget.raster_position().coordinates();
            eprintln!(
                "[POLY-PCS] host={} mvn_steps={} mvn_master={} mvn_first={:?} mvn_last={:?} mvn_deltas={:?} fd74={} fdcf={} feb4={} ff1e={} fe27={}",
                self.frame_ctr_dbg,
                mvn_steps,
                mvn_master,
                mvn_first,
                mvn_last,
                mvn_deltas,
                pc_counts[0],
                pc_counts[1],
                pc_counts[2],
                pc_counts[3],
                pc_counts[4]
            );
            eprintln!(
                "[POLY-SLOT] host={} slot_entry={}:{} latch={} upload={} gates_prev={:?} gates_cur={:?} virq={} hosts={} master={} completed={} end={}:{} pc={:06x} hdma_init={} hdma_lines={} hdma_master={} hdma_mask={:#04x} ch7={{indirect={} mode={} a={:02x}:{:04x}}}",
                self.frame_ctr_dbg,
                scanline,
                cycle,
                latch,
                upload,
                self.poly_receipt_gates_prev,
                self.poly_receipt_gates_cur,
                virq_scanline,
                self.poly_shadow_hosts,
                self.poly_shadow_master,
                completed,
                now,
                cyc,
                run.pc(),
                ROM_CPU_SHADOW_HDMA_DEBUG[0].load(std::sync::atomic::Ordering::Relaxed),
                ROM_CPU_SHADOW_HDMA_DEBUG[2].load(std::sync::atomic::Ordering::Relaxed),
                ROM_CPU_SHADOW_HDMA_DEBUG[1].load(std::sync::atomic::Ordering::Relaxed),
                self.game_state.display.hdma_enable_mask,
                self.dma.channel[7].indirect,
                self.dma.channel[7].mode,
                self.dma.channel[7].a_bank,
                self.dma.channel[7].a_adr,
            );
        }
        if !completed {
            self.poly_shadow_run = Some(run);
        }
        completed
    }

    pub(super) fn apply_original_timing_idle_main_loop_suffix_action(
        &mut self,
        action: OriginalTimingIdleMainLoopSuffixAction,
    ) {
        match action {
            OriginalTimingIdleMainLoopSuffixAction::ArmOrdinary => {
                assert_eq!(
                    self.pending_main_loop_common_suffix, None,
                    "a fresh uninterrupted main-loop iteration produced an overlapping common suffix",
                );
                self.pending_main_loop_common_suffix =
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
            }
            OriginalTimingIdleMainLoopSuffixAction::RetainOrdinary => {
                assert_eq!(
                    self.pending_main_loop_common_suffix,
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                    "a continued uninterrupted main-loop call changed its ordinary common suffix",
                );
            }
            OriginalTimingIdleMainLoopSuffixAction::CompleteInSequence => assert_eq!(
                self.pending_main_loop_common_suffix, None,
                "a source-completed main-loop host retained a pending common suffix",
            ),
            OriginalTimingIdleMainLoopSuffixAction::HoldAtMainWait => assert_eq!(
                self.pending_main_loop_common_suffix, None,
                "a held main-wait host cannot own a pending common suffix",
            ),
        }
    }

    pub(super) fn assert_original_timing_idle_main_loop_suffix_postcondition(
        &self,
        action: OriginalTimingIdleMainLoopSuffixAction,
    ) {
        let expected = match action {
            OriginalTimingIdleMainLoopSuffixAction::ArmOrdinary
            | OriginalTimingIdleMainLoopSuffixAction::RetainOrdinary => {
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
            }
            OriginalTimingIdleMainLoopSuffixAction::CompleteInSequence
            | OriginalTimingIdleMainLoopSuffixAction::HoldAtMainWait => None,
        };
        assert_eq!(
            self.pending_main_loop_common_suffix, expected,
            "uninterrupted main-loop execution violated its validated suffix action",
        );
    }

    /// Whether the live wire leaves a fresh iteration's shared suffix
    /// outstanding past this host: no suffix completion and no Open acceptance
    /// after the iteration's own progress receipt (a leading Open acceptance
    /// belongs to the previous iteration's suffix).
    pub(super) fn original_timing_live_fresh_iteration_suffix_outstanding(&self) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return false;
        }
        let Some(receipts) = self.original_timing_semantic_receipts.as_ref() else {
            return false;
        };
        let semantic = receipts.semantic();
        // The iteration's own progress receipt is consumed when it starts, so
        // locate the trailing region after the last handler completion: a
        // leading Open acceptance precedes its handler, a trailing one follows.
        let trailing_start = semantic
            .iter()
            .rposition(|receipt| {
                matches!(receipt, OriginalTimingSemanticReceipt::NmiHandlerCompleted)
            })
            .map_or(0, |index| index + 1);
        !semantic.iter().any(|receipt| {
            matches!(
                receipt,
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
            )
        }) && !semantic[trailing_start..].iter().any(|receipt| {
            matches!(
                receipt,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open)
            )
        })
    }

    /// Execute the source-owned terminal ordering for a suspended
    /// `ZeldaRunGameLoop` caller.
    ///
    /// The CPU-only caller completion is deliberately placed between the two
    /// hardware phase groups. In the C call stack, an NMI may interrupt the
    /// caller while `$12` is still held; only after that handler returns may
    /// the caller finish and reach `NMI_PrepareSprites` plus the `$12` clear.
    /// A later accepted NMI belongs to the following host when its handler has
    /// not completed yet.
    pub(super) fn complete_original_timing_nmi_handler_for_active_scanout(
        &mut self,
        completion_owner: OriginalTimingNmiHandlerCompletionOwner,
        input: u16,
        oam_dma_source: Option<&[u8]>,
    ) -> OriginalTimingNmiHandlerCompletion {
        if !completion_owner.completed() {
            return OriginalTimingNmiHandlerCompletion::none();
        }
        if completion_owner == OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence {
            // A host can finish just before a cached-slot restore store and
            // accept its next NMI just after it. Advance the native store
            // cursor before this acceptance captures its scanout.
            self.publish_cached_sprite_restore_before_acceptance();
        }
        assert_original_timing_carry_in_handler_has_receptive_display(
            OriginalTimingNmiPhaseClassification {
                handler_completion: completion_owner,
                publication_pending_at_exit: false,
            },
            self.display_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
        );
        // A return-only dialogue scroll captures its outgoing field before it
        // stages the completed text generation. The following Open NMI is the
        // first hardware boundary which can publish that text. Capture alone
        // is not proof: both a carried handler and an acceptance completed in
        // this sequence must publish the exact BG3 DMA receipt before the
        // dialogue state machine advances.
        let has_staged_dialogue_completion = matches!(
            self.dialogue_scroll_phase(),
            DialogueScrollPhase::CompletionStagedAfterSnapshot
                | DialogueScrollPhase::CompletionStagedAfterFrozenScanout
        );
        let staged_dialogue_waits_for_caller = has_staged_dialogue_completion
            && self.original_timing_expected_nmi_update_gates.first()
                == Some(&NmiUpdateGate::LatchHeld);
        if staged_dialogue_waits_for_caller {
            // RenderText can return before NMI_PrepareSprites finishes its
            // extended-OAM packing. The completed CPU text stays staged
            // through that Held handler; only the caller's later latch clear
            // permits an Open NMI to publish the BG3 DMA.
            assert!(
                self.pending_main_loop_common_suffix.is_some(),
                "staged dialogue cannot wait on a Held NMI without its suspended caller"
            );
            assert!(
                self.game_state.display.nmi_update_is_latched(),
                "a suspended dialogue caller must retain its native NMI latch"
            );
        }
        let publishes_staged_dialogue_completion =
            (has_staged_dialogue_completion && !staged_dialogue_waits_for_caller).then(|| {
                if completion_owner
                    == OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry
                {
                    assert_eq!(
                        self.original_timing_pending_nmi_update_gate,
                        Some(NmiUpdateGate::Open),
                        "a staged dialogue completion requires its carried Open NMI",
                    );
                }
                assert_eq!(
                    self.original_timing_expected_nmi_update_gates.first(),
                    Some(&NmiUpdateGate::Open),
                    "a staged dialogue completion disagrees with its installed Open-NMI authority",
                );
                assert!(
                !self.game_state.display.nmi_update_is_latched(),
                "a staged dialogue completion cannot publish while the native NMI latch is held",
            );
                assert_eq!(
                    self.game_state.display.pending_nmi_subroutine, 2,
                    "a staged dialogue completion requires the source BG3 text-DMA subroutine",
                );
                assert_eq!(
                    self.game_state.display.core_update_disable_flag, 2,
                    "a staged dialogue completion requires the source BG3 text-DMA disable state",
                );
                true
            });
        let acceptance_ppu_registers = (completion_owner
            == OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry)
            .then(|| {
                nmi_ppu_register_scanout_from_acceptance_snapshot(
                    self.display_snapshot
                        .as_ref()
                        .expect("carry-in handler lost its receptive acceptance snapshot"),
                    self.original_timing_expected_nmi_ppu_register_operands
                        .first()
                        .copied()
                        .flatten(),
                )
            });
        self.original_timing_nmi_publication_pending = false;
        if completion_owner == OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence {
            self.capture_display_snapshot();
        }
        let dialogue_text_dma = self.interrupt_nmi_for_active_scanout(input, oam_dma_source, false);
        if let Some(registers) = acceptance_ppu_registers {
            // Correct both owners: the already-receptive scanout receipt and
            // resident hardware inherited by a trailing acceptance capture.
            // Updating only the former lets the later capture reintroduce the
            // post-interruption native mirrors into the next visible field.
            registers.publish_to(&mut self.ppu);
            let effective = self
                .display_snapshot
                .as_mut()
                .and_then(|snapshot| snapshot.effective_presented_dma.as_mut())
                .expect("carry-in handler lost its active register receipt");
            effective.completed_ppu_registers = Some(registers);
        }
        self.apply_original_timing_joypad_publication();
        if publishes_staged_dialogue_completion == Some(true) {
            assert!(
                dialogue_text_dma.is_none(),
                "a completed staged dialogue handler left its exact BG3 text-DMA evidence unconsumed",
            );
            assert_eq!(
                self.dialogue_scroll_phase(),
                DialogueScrollPhase::CompletedScroll,
                "a completed Open dialogue handler did not publish its staged BG3 generation",
            );
        }
        OriginalTimingNmiHandlerCompletion {
            completed: true,
            dialogue_text_dma,
        }
    }

    pub(super) fn complete_original_timing_main_loop_return<F>(
        &mut self,
        timeline: OriginalTimingMainLoopReturnTimeline,
        input: u16,
        oam_dma_source: Option<&[u8]>,
        complete_cpu_caller: F,
    ) where
        F: FnOnce(&mut Self),
    {
        assert_eq!(
            timeline.progress,
            crate::MainLoopProgress::CallStackContinued,
            "a terminal caller return cannot begin a fresh ZeldaRunGameLoop iteration: host={} frame={:?} scheduler={:?}",
            self.frame_ctr_dbg,
            self.game_state.frame,
            self.game_execution_scheduler,
        );
        assert!(
            self.pending_main_loop_common_suffix.is_some(),
            "a terminal caller return lost its suspended ZeldaRunGameLoop suffix",
        );

        // Classify the complete receipt before applying any semantic writes,
        // so a malformed source lifecycle fails without partially returning
        // the translated caller.
        let before_return = classify_original_timing_nmi_phases_with_ownership(
            self.original_timing_nmi_publication_pending,
            &timeline.nmi_phases_before_return,
        );
        assert!(
            !before_return.publication_pending_at_exit,
            "ZeldaRunGameLoop cannot complete its common suffix while an accepted NMI handler remains unfinished: {timeline:?}",
        );
        let (completes_after_return, pending_after_return) =
            classify_original_timing_nmi_phases(false, &timeline.nmi_phases_after_return);

        self.complete_original_timing_nmi_handler_for_active_scanout(
            before_return.handler_completion,
            input,
            oam_dma_source,
        )
        .assert_no_unclaimed_dialogue_text_dma();

        complete_cpu_caller(self);
        self.complete_pending_main_loop_common_suffix_after_module_return();

        if completes_after_return {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, None, false);
            self.apply_original_timing_joypad_publication();
        }
        if pending_after_return {
            // An accepted-at-return NMI belongs to the next host, but its
            // acceptance host owns the scanout which that handler will refine.
            // Capture after any earlier same-host handler so the carried
            // completion cannot reuse an older receptive snapshot.
            self.capture_and_carry_original_timing_nmi_publication_at_host_return();
        }
    }
}
