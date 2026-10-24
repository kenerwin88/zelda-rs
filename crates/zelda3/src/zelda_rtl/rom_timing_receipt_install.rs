//! Exact cartridge timing entry points; excluded from the lagless build.

use super::*;

impl ZeldaState {
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
        if let Ok(range) = crate::debug_env::var!("ZELDA3_DEBUG_INSTALL_RECEIPTS") {
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
                    DungeonResetSpritesCpuProgress::GarnishTypesThrough { slot } => slot >= 30,
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
                crate::CachedSpriteExecutionProgress::AfterSlot(slot) => slot >= 16,
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
}
