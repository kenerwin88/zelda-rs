//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (spotlight).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub(super) fn begin_dungeon_landing_spotlight_publication(
        &mut self,
        entry_started_after_leading_nmi: bool,
    ) {
        self.interrupted_dungeon_spotlight_build_in_flight = None;
        self.last_completed_interrupted_dungeon_spotlight_scanout = None;
        self.dungeon_landing_entry_started_after_leading_nmi = entry_started_after_leading_nmi;
        self.dungeon_landing_spotlight_reset_prefix_scanlines = None;
        self.active_dungeon_landing_spotlight_reset_prefix_scanlines = None;
    }

    pub(crate) fn set_spotlight_y_lower(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_y_lower(value);
    }

    pub(crate) fn set_spotlight_y_upper(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_y_upper(value);
    }

    pub(crate) fn set_spotlight_window_x_center(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_window_x_center(value);
    }

    pub(crate) fn set_spotlight_window_state(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_window_state(value);
    }

    pub(crate) fn set_spotlight_window_radius(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_window_radius(value);
    }

    pub(crate) fn set_spotlight_window_y_buffer(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_window_y_buffer(value);
    }

    pub(crate) fn decrement_spotlight_window_y_buffer(&mut self) -> u16 {
        self.spotlight_hdma_mut().decrement_window_y_buffer()
    }

    pub(crate) fn set_spotlight_window_radius_byte(&mut self, value: u8) {
        self.spotlight_hdma_mut().set_window_radius_byte(value);
    }

    pub(crate) fn set_spotlight_window_state_byte(&mut self, value: u8) {
        self.spotlight_hdma_mut().set_window_state_byte(value);
    }

    pub(crate) fn set_spotlight_window_y_buffer_byte(&mut self, value: u8) {
        self.spotlight_hdma_mut().set_window_y_buffer_byte(value);
    }

    pub(crate) fn increment_spotlight_window_y_buffer_byte(&mut self) {
        self.spotlight_hdma_mut().increment_window_y_buffer_byte();
    }

    pub(crate) fn shr_spotlight_window_radius_byte(&mut self, shift: u8) {
        self.spotlight_hdma_mut().shr_window_radius_byte(shift);
    }

    pub(crate) fn add_spotlight_window_radius_byte(&mut self, value: u8) {
        self.spotlight_hdma_mut().add_window_radius_byte(value);
    }

    pub(crate) fn spotlight_hdma_table_dynamic_entry(&self, index: usize) -> u16 {
        self.game_state
            .display
            .spotlight_hdma
            .hdma_table_dynamic_entry(index)
    }

    pub(crate) fn set_spotlight_hdma_table_dynamic_entry(&mut self, index: usize, value: u16) {
        self.spotlight_hdma_mut()
            .set_hdma_table_dynamic_entry(index, value);
    }

    pub(crate) fn clear_spotlight_hdma_table_dynamic(&mut self, count: usize) {
        self.spotlight_hdma_mut().clear_hdma_table_dynamic(count);
    }

    pub(crate) fn clear_spotlight_hdma_table_dynamic_range(&mut self, start: usize, count: usize) {
        self.spotlight_hdma_mut()
            .clear_hdma_table_dynamic_range(start, count);
    }

    pub(super) fn restore_spotlight_hdma_from_saveload_buffer(&mut self) {
        // C copies SAVELOAD_HDMA_TABLE[0..224] -> HDMA_TABLE_DYNAMIC[0..224] as a raw ram copy,
        // leaving the off-screen entries 224-239 at their loaded-snapshot value. Routing through
        // the native spotlight table instead re-projects ITS stale 224-239 (0xff00) over the
        // snapshot's zeros -> a 1390-frame parity divergence in the off-screen HDMA scanlines
        // (page 0x1dc00). Do the raw ram copy; load_snes_state's following
        // sync_native_game_state_from_ram reloads the native table from this ram.
        self.spotlight_hdma_mut()
            .copy_saveload_buffer_to_dynamic_table_ram(224);
    }

    pub(super) fn backup_spotlight_hdma_to_saveload_buffer(&mut self) {
        self.spotlight_hdma_mut()
            .backup_dynamic_table_to_saveload_buffer(224);
    }

    pub(crate) fn project_spotlight_dynamic_hdma_table_range_to_reserved(
        &mut self,
        start: usize,
        count: usize,
    ) {
        self.spotlight_hdma_mut()
            .project_dynamic_table_range_to_reserved_hdma_table(start, count);
    }

    /// Prove the exact recurring-spotlight host whose saved Build reaches
    /// Module0F's LinkOam boundary but not ZeldaRunGameLoop's common suffix.
    pub(super) fn original_timing_spotlight_build_link_oam_plan(
        &self,
    ) -> Option<OriginalTimingSpotlightBuildLinkOamPlan> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || self.original_timing_main_loop_progress()
                != Some(crate::MainLoopProgress::CallStackContinued)
        {
            return None;
        }
        let work = self.game_execution_scheduler.current_work()?;
        let GameWorkContinuation::FinishDungeonExitSpotlightBuild {
            mut table_build,
            projection_completed,
            iteration,
        } = work
        else {
            return None;
        };
        let interruption = self.original_timing_main_loop_interruption()?;
        match interruption {
            crate::MainLoopInterruption::LinkOam
            | crate::MainLoopInterruption::LinkActualVelocity { .. }
            | crate::MainLoopInterruption::LinkActualVelocityCompleted
            | crate::MainLoopInterruption::LinkVelocityClearProgress { .. }
            | crate::MainLoopInterruption::DungeonExitSpotlightTableCompleted
            | crate::MainLoopInterruption::LinkPositionAfterSubpixel { .. }
            | crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { .. }
            | crate::MainLoopInterruption::LinkPositionAfterCoordinates { .. } => {}
            other => panic!(
                "a continued recurring spotlight Build requires its LinkOam or mid-loop Link position boundary, not {other:?}"
            ),
        }
        assert!(
            iteration.is_closing(),
            "a recurring spotlight Build-LinkOam boundary requires a closing iteration",
        );
        assert!(
            iteration.rom_following_field_receipt().is_none(),
            "a recurring spotlight Build-LinkOam boundary cannot bypass a following-field receipt",
        );
        assert!(
            !iteration.prepares_main_loop_sprites_before_second_nmi(),
            "a recurring spotlight Build-LinkOam boundary cannot bypass an earlier sprite-preparation boundary",
        );
        assert_eq!(
            self.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "a recurring spotlight Build-LinkOam boundary lost its ordinary main-loop suffix",
        );
        assert!(
            !self.main_loop_sprite_preparation_completed,
            "a recurring spotlight Build-LinkOam boundary cannot follow a completed common suffix",
        );
        let publication_pending_at_entry = self.original_timing_nmi_publication_pending;
        if publication_pending_at_entry {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate,
                Some(NmiUpdateGate::LatchHeld),
                "a carried recurring spotlight Build-LinkOam handler lost its Held disposition",
            );
            // The LinkOam interruption either rides a second in-host Held
            // acceptance or lands at the host return with only the carried
            // gate installed (route host 54149).
            assert!(
                matches!(
                    self.original_timing_expected_nmi_update_gates.as_slice(),
                    [NmiUpdateGate::LatchHeld]
                        | [NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld]
                ),
                "a carried recurring spotlight Build-LinkOam host disagrees with its installed NMI gate authority: {:?}",
                self.original_timing_expected_nmi_update_gates,
            );
        } else {
            assert_eq!(
                self.original_timing_pending_nmi_update_gate, None,
                "the recurring spotlight Build-LinkOam host retained a gate without a carried handler",
            );
            // The LinkOam interruption either lands at the host return (one
            // Held acceptance) or rides a second in-host Held acceptance
            // (route host 37587).
            assert!(
                matches!(
                    self.original_timing_expected_nmi_update_gates.as_slice(),
                    [NmiUpdateGate::LatchHeld]
                        | [NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld]
                ),
                "the recurring spotlight Build-LinkOam host disagrees with its installed NMI gate authority: {:?}",
                self.original_timing_expected_nmi_update_gates,
            );
        }
        assert!(
            self.game_state.display.nmi_update_is_latched(),
            "the recurring spotlight Build-LinkOam host's Held acceptance disagrees with the native latch",
        );
        assert_eq!(
            self.original_timing_sprite_main_return_claims_remaining, None,
            "a recurring spotlight Build-LinkOam host cannot overlap a Sprite_Main claim scope",
        );
        assert!(
            !self.original_timing_dungeon_exit_spotlight_entry_return_pending,
            "a recurring spotlight Build-LinkOam host cannot retain an earlier entry return",
        );
        assert!(
            !self.original_timing_scheduled_nmi_accepted_at_host_return,
            "a recurring spotlight Build-LinkOam host cannot overlap an older staged acceptance",
        );

        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .expect("a recurring spotlight Build-LinkOam host lost its semantic authority")
            .semantic()
            .to_vec();
        let checkpoint_claims = semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(receipt) => {
                    Some((index, *receipt))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            checkpoint_claims.len() <= 1,
            "one recurring spotlight Build-LinkOam host cannot re-checkpoint its table twice",
        );
        let mut cpu_probe = self.clone();
        let mut rebuild_progress = None;
        let mut semantic_without_checkpoint = semantic.clone();
        if let Some(&(checkpoint_index, claim)) = checkpoint_claims.first() {
            assert_eq!(
                claim.boundary,
                crate::OriginalTimingBoundary::NmiAccepted,
                "a recurring spotlight Build-LinkOam re-checkpoint must ride an accepting NMI",
            );
            table_build.assert_recheckpoint_not_behind(claim.progress);
            if table_build.source_progress != Some(claim.progress) {
                rebuild_progress = Some(claim.progress);
                table_build =
                    crate::cycle_ledger::muted(|| cpu_probe.begin_iris_spotlight_configure_table_at_progress(claim.progress));
            }
            let last_acceptance = semantic[..checkpoint_index]
                .iter()
                .rposition(|receipt| {
                    matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_))
                })
                .expect("a spotlight NmiAccepted checkpoint preceded every accepting NMI");
            let last_completion = semantic[..checkpoint_index].iter().rposition(|receipt| {
                matches!(receipt, OriginalTimingSemanticReceipt::NmiHandlerCompleted)
            });
            assert!(
                last_completion.is_none_or(|completion| last_acceptance > completion),
                "a spotlight table checkpoint was not published by the currently accepted NMI",
            );
            semantic_without_checkpoint.remove(checkpoint_index);
        }
        let expected_semantic = if publication_pending_at_entry {
            if semantic_without_checkpoint
                .iter()
                .any(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_)))
            {
                // The carried handler completes first; the resumed Build then
                // runs until the following Held acceptance interrupts it
                // inside LinkOam.
                vec![
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption),
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::CallStackContinued,
                    ),
                ]
            } else {
                // The carried handler completes and the host returns while
                // still inside LinkOam, with no further acceptance (route
                // host 54149).
                vec![
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::CallStackContinued,
                    ),
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption),
                ]
            }
        } else if semantic_without_checkpoint
            .iter()
            .filter(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_)))
            .count()
            == 2
        {
            // The first Held pair completes in-host; the Build then runs
            // until a SECOND Held acceptance interrupts it inside LinkOam,
            // restating the build checkpoint at that boundary (route host
            // 37587). The second handler belongs to the next host.
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ]
        } else {
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption),
            ]
        };
        assert_eq!(
            semantic_without_checkpoint, expected_semantic,
            "a recurring spotlight Build-LinkOam host published an unsupported or reordered semantic vector",
        );
        let second_acceptance_rides_interruption = !publication_pending_at_entry
            && semantic
                .iter()
                .filter(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_)))
                .count()
                == 2;
        let timeline = OriginalTimingMainLoopInterruptionTimeline {
            progress: crate::MainLoopProgress::CallStackContinued,
            interruption,
            nmi_phases_before_interruption: if publication_pending_at_entry {
                if semantic
                    .iter()
                    .any(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_)))
                {
                    vec![
                        OriginalTimingNmiPhase::HandlerCompleted,
                        OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                    ]
                } else {
                    vec![OriginalTimingNmiPhase::HandlerCompleted]
                }
            } else if second_acceptance_rides_interruption {
                vec![
                    OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingNmiPhase::HandlerCompleted,
                    OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                ]
            } else {
                vec![
                    OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingNmiPhase::HandlerCompleted,
                ]
            },
            nmi_phases_after_interruption: Vec::new(),
        };

        let mut scheduler_before_completion = self.game_execution_scheduler;
        scheduler_before_completion.begin_host_frame();
        assert_eq!(
            scheduler_before_completion.current_work(),
            Some(work),
            "host normalization changed the recurring spotlight Build owner",
        );
        let mut scheduler_after_completion = scheduler_before_completion;
        assert_eq!(
            scheduler_after_completion
                .advance_work_one_nmi_slice_with_authoritative_completion(true),
            Some(GameWorkStep::Complete(work)),
            "the source-proven LinkOam boundary did not complete its saved Build",
        );
        assert!(
            scheduler_after_completion.is_idle(),
            "the recurring spotlight Build probe retained work before its CPU closure",
        );

        // The semantic LinkOam boundary supersedes only the replaceable
        // scheduler estimate.  Prove the exact stored C continuation can run
        // to that boundary before the live Held handler publishes anything.
        let work = GameWorkContinuation::FinishDungeonExitSpotlightBuild {
            table_build,
            projection_completed,
            iteration,
        };
        cpu_probe.game_execution_scheduler = scheduler_after_completion;
        let suffix_before = cpu_probe.pending_main_loop_common_suffix;
        let latch_before = cpu_probe.game_state.display.nmi_update_is_latched();
        let epoch_before = cpu_probe.display_snapshot_epoch;
        let gates_before = cpu_probe.original_timing_expected_nmi_update_gates.clone();
        let semantic_before = cpu_probe
            .original_timing_semantic_receipts
            .as_ref()
            .map(|receipts| receipts.semantic().to_vec());
        match interruption {
            crate::MainLoopInterruption::LinkActualVelocity {
                horizontal_resolved,
            } => {
                crate::cycle_ledger::muted(|| cpu_probe.complete_dungeon_exit_spotlight_build_until_link_actual_velocity(
                    table_build,
                    projection_completed,
                    iteration,
                    horizontal_resolved,
                ));
                assert!(matches!(
                    cpu_probe.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity { .. })
                ));
            }
            crate::MainLoopInterruption::DungeonExitSpotlightTableCompleted => {
                crate::cycle_ledger::muted(|| cpu_probe.complete_dungeon_exit_spotlight_build_until_control(
                    table_build,
                    projection_completed,
                    iteration,
                ));
                assert!(matches!(
                    cpu_probe.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonExitSpotlightControl { .. })
                ));
            }
            crate::MainLoopInterruption::LinkActualVelocityCompleted => {
                crate::cycle_ledger::muted(|| cpu_probe.complete_dungeon_exit_spotlight_build_until_link_actual_velocity(
                    table_build,
                    projection_completed,
                    iteration,
                    LinkActualVelocityCheckpoint::AfterBoth,
                ));
                assert!(matches!(
                    cpu_probe.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity { .. })
                ));
            }
            crate::MainLoopInterruption::LinkVelocityClearProgress { completed } => {
                crate::cycle_ledger::muted(|| cpu_probe.complete_dungeon_exit_spotlight_build_until_link_actual_velocity(
                    table_build,
                    projection_completed,
                    iteration,
                    LinkActualVelocityCheckpoint::Clearing { completed },
                ));
                assert!(matches!(
                    cpu_probe.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity { .. })
                ));
            }
            crate::MainLoopInterruption::LinkPositionAfterSubpixel { pass } => {
                crate::cycle_ledger::muted(|| cpu_probe.complete_dungeon_exit_spotlight_build_until_link_position_partial(
                    table_build,
                    projection_completed,
                    iteration,
                    pass,
                ));
                assert!(matches!(
                    cpu_probe.game_execution_scheduler.current_work(),
                    Some(
                        GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel { .. }
                    )
                ));
            }
            crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { pass } => {
                cpu_probe
                    .complete_dungeon_exit_spotlight_build_until_link_position_after_coordinate_low(
                        table_build,
                        projection_completed,
                        iteration,
                        pass,
                    );
                assert!(matches!(
                    cpu_probe.game_execution_scheduler.current_work(),
                    Some(
                        GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow { .. }
                    )
                ));
            }
            crate::MainLoopInterruption::LinkPositionAfterCoordinates { pass } => {
                cpu_probe
                    .complete_dungeon_exit_spotlight_build_until_link_position_after_coordinates(
                        table_build,
                        projection_completed,
                        iteration,
                        pass,
                    );
                assert!(matches!(
                    cpu_probe.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates { .. })
                ));
            }
            crate::MainLoopInterruption::LinkOam => {
                crate::cycle_ledger::muted(|| cpu_probe.complete_dungeon_exit_spotlight_build(
                    table_build,
                    projection_completed,
                    iteration,
                    false,
                    true,
                ));
                assert_eq!(
                    cpu_probe.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { iteration }),
                    "the recurring spotlight Build probe did not retain exactly its LinkOam caller suffix",
                );
            }
            _ => unreachable!("validated recurring spotlight boundary changed"),
        }
        assert_eq!(
            cpu_probe.pending_main_loop_common_suffix, suffix_before,
            "the recurring spotlight Build probe consumed the following host's common suffix",
        );
        assert_eq!(
            cpu_probe.game_state.display.nmi_update_is_latched(),
            latch_before,
            "the recurring spotlight Build probe changed the source-held NMI latch",
        );
        assert_eq!(
            cpu_probe.display_snapshot_epoch, epoch_before,
            "the recurring spotlight Build CPU callback cannot publish a host boundary itself",
        );
        assert_eq!(
            cpu_probe.original_timing_expected_nmi_update_gates, gates_before,
            "the recurring spotlight Build CPU callback consumed hardware gate authority",
        );
        assert_eq!(
            cpu_probe
                .original_timing_semantic_receipts
                .as_ref()
                .map(|receipts| receipts.semantic().to_vec()),
            semantic_before,
            "the recurring spotlight Build CPU callback consumed source semantic authority",
        );
        let scheduler_after_cpu = cpu_probe.game_execution_scheduler;

        Some(OriginalTimingSpotlightBuildLinkOamPlan {
            semantic,
            rebuild_progress,
            timeline,
            interruption,
            work,
            scheduler_before_completion,
            scheduler_after_completion,
            scheduler_after_cpu,
        })
    }

    /// Prove a source host which returns a closing spotlight caller to
    /// ZeldaRunGameLoop's main wait, without consuming its NMI, scheduler, or
    /// wire-last caller-return authority.
    pub(super) fn original_timing_terminal_spotlight_iteration_plan(
        &self,
    ) -> Option<OriginalTimingTerminalSpotlightIterationPlan> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let work = self.game_execution_scheduler.current_work()?;
        let (iteration, mut cpu_action) =
            if let Some(iteration) = work.terminal_spotlight_suffix_only_iteration() {
                (
                    iteration,
                    OriginalTimingTerminalSpotlightCpuAction::SuffixOnly,
                )
            } else if let GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                iteration,
            } = work
            {
                (
                    iteration,
                    OriginalTimingTerminalSpotlightCpuAction::CompleteBuild {
                        table_build,
                        projection_completed,
                    },
                )
            } else {
                return None;
            };
        if !iteration.is_closing()
            && !matches!(
                cpu_action,
                OriginalTimingTerminalSpotlightCpuAction::SuffixOnly
            )
        {
            // An opening table Build belongs to the overworld spotlight
            // lanes; only its saved suffix-and-projection continuation shares
            // this terminal caller-return grammar.
            return None;
        }
        assert!(
            iteration.rom_following_field_receipt().is_none(),
            "a terminal closing spotlight caller cannot bypass an authoritative following-field receipt",
        );
        if iteration.prepares_main_loop_sprites_before_second_nmi() {
            // Only begin_dungeon_exit_spotlight_build produces this estimate,
            // and only when the live receipt interrupted the C call inside its
            // ProjectionCopy memcpy — a continuation whose row-pair loop is
            // complete and whose projection tail is already cleared. The
            // wire-last caller return below proves the same single same-host
            // suffix the estimate predicted, so it supersedes the replaceable
            // annotation instead of competing with it. Every other carrier of
            // the estimate remains fail-closed.
            let OriginalTimingTerminalSpotlightCpuAction::CompleteBuild { table_build, .. } =
                cpu_action
            else {
                panic!(
                    "a suffix-only spotlight terminal cannot carry a same-host sprite-preparation estimate",
                );
            };
            assert!(
                table_build.completed && table_build.projection_tail_cleared,
                "a same-host sprite-preparation estimate requires its interrupted ProjectionCopy continuation",
            );
        }
        let timeline = self.original_timing_main_loop_return_timeline()?;

        assert_eq!(
            timeline.progress,
            crate::MainLoopProgress::CallStackContinued,
            "a terminal closing spotlight caller cannot begin a fresh main-loop iteration",
        );
        let complete_build = matches!(
            cpu_action,
            OriginalTimingTerminalSpotlightCpuAction::CompleteBuild { .. }
        );
        if complete_build {
            let canonical = if iteration.prepares_main_loop_sprites_before_second_nmi() {
                SpotlightIteration::closing(iteration.phase)
                    .with_main_loop_sprite_preparation_before_second_nmi()
            } else {
                SpotlightIteration::closing(iteration.phase)
            };
            assert_eq!(
                iteration, canonical,
                "a terminal spotlight Build requires its canonical plain closing display policy",
            );
        }
        let publication_pending_at_entry = self.original_timing_nmi_publication_pending;
        // The adapter's caller-return receipt is module-qualified: it exists
        // only when the resumed source call both entered and returned inside
        // Module0F ($0F/sub 1). A suffix-only continuation whose module body
        // already advanced the frame out of Module0F in its scheduling host
        // therefore returns without the token; its terminal authority is the
        // continued-call return timeline plus the completed common suffix. A
        // Build terminal resumes the module body inside this host, so its
        // native frame still names Module0F and the token stays required.
        let caller_return_token =
            self.original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic().iter().any(|receipt| {
                        matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait
                    )
                    })
                });
        let entry_return_token =
            self.original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic().iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned
                        )
                    })
                });
        let overworld_goal_return_token = self
            .original_timing_semantic_receipts
            .as_ref()
            .is_some_and(|receipts| {
                receipts.semantic().iter().any(|receipt| {
                    matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned
                    )
                })
            });
        let module_frame_owns_spotlight_close =
            self.game_state.frame.main_module == 0x0f && self.game_state.frame.submodule == 1;
        if complete_build {
            // The final closing iteration's module body leaves Module0F for
            // Module08_PreOverworld inside this same host, so the adapter's
            // module-qualified token is absent there; the continued-call
            // return timeline (the completed common suffix) is the return
            // proof in that case (route host 597513: returned main=8).
            let module_body_exits_module0f_in_host =
                self.original_timing_main_loop_return_timeline().is_some();
            assert!(
                caller_return_token || module_body_exits_module0f_in_host,
                "a terminal spotlight Build requires its module-qualified caller-return receipt: host={} frame={:?} receipts={:?} gates={:?}",
                self.frame_ctr_dbg,
                self.game_state.frame,
                self.original_timing_semantic_receipts
                    .as_ref()
                    .map(|receipts| receipts.semantic().to_vec()),
                self.original_timing_expected_nmi_update_gates,
            );
            assert!(
                !entry_return_token,
                "a terminal spotlight Build cannot also publish an entry-return token",
            );
        } else {
            // Only a closing continuation can be the Module0F close caller
            // the adapter's token names; an opening caller returns through
            // its own module and never carries it. A close-entry iteration
            // whose host itself crossed the sub-0 entry transition publishes
            // the entry-return token instead, which its pre-entry owner has
            // already consumed by this point.
            let token_expected = iteration.is_closing() && module_frame_owns_spotlight_close;
            if token_expected && !caller_return_token {
                assert!(
                    iteration.phase.is_close_entry() && entry_return_token,
                    "a suffix-only spotlight terminal's caller-return receipt disagrees with its native Module0F frame",
                );
            } else {
                assert!(
                    !entry_return_token,
                    "an entry-return token cannot accompany a recurring caller-return terminal",
                );
                assert_eq!(
                    caller_return_token, token_expected,
                    "a suffix-only spotlight terminal's caller-return receipt disagrees with its native Module0F frame",
                );
            }
        }
        let spotlight_claim =
            self.original_timing_semantic_receipts
                .as_ref()
                .and_then(|receipts| {
                    receipts
                        .semantic()
                        .iter()
                        .find_map(|receipt| match receipt {
                            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(claim) => {
                                Some(*claim)
                            }
                            _ => None,
                        })
                });
        if let Some(claim) = spotlight_claim {
            // The leading vblank can interrupt any still-running statement
            // after the prior host-return checkpoint. Prove that the new
            // statement advances the saved continuation; CPU execution below
            // rebuilds from that exact checkpoint before completing it.
            assert_eq!(
                claim.boundary,
                crate::OriginalTimingBoundary::NmiAccepted,
                "a terminal spotlight re-checkpoint must be published at its interrupting acceptance",
            );
            assert!(
                !publication_pending_at_entry,
                "a terminal spotlight re-checkpoint requires its interrupting acceptance inside this host",
            );
            if let OriginalTimingTerminalSpotlightCpuAction::CompleteBuild { table_build, .. } =
                &mut cpu_action
            {
                table_build.assert_recheckpoint_not_behind(claim.progress);
            } else {
                // A suffix-only terminal's interrupting acceptance can
                // re-checkpoint the separately-suspended recurring build;
                // the claim corroborates the saved in-flight continuation
                // and is consumed with this plan (route host 40982).
            }
        }
        let trailing_open = matches!(
            timeline.nmi_phases_after_return.as_slice(),
            [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)]
        );
        if complete_build {
            let expected_before = if publication_pending_at_entry {
                vec![OriginalTimingNmiPhase::HandlerCompleted]
            } else {
                vec![
                    OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingNmiPhase::HandlerCompleted,
                ]
            };
            assert_eq!(
                timeline.nmi_phases_before_return, expected_before,
                "a terminal spotlight Build must complete exactly one carried or same-host Held handler: {timeline:?}",
            );
            assert!(
                timeline.nmi_phases_after_return.is_empty() || trailing_open,
                "a terminal spotlight Build may only carry one trailing Open acceptance: {timeline:?}",
            );
        } else {
            let expected_before = if publication_pending_at_entry {
                vec![OriginalTimingNmiPhase::HandlerCompleted]
            } else {
                vec![
                    OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingNmiPhase::HandlerCompleted,
                ]
            };
            assert_eq!(
                timeline.nmi_phases_before_return, expected_before,
                "a terminal closing spotlight caller must complete exactly one carried or same-host Held handler: {timeline:?}",
            );
            assert!(
                timeline.nmi_phases_after_return.is_empty() || trailing_open,
                "a suffix-only spotlight terminal may only carry one trailing Open acceptance: {timeline:?}",
            );
        }
        assert_eq!(
            self.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "a terminal closing spotlight caller lost its ordinary main-loop suffix",
        );
        assert!(
            !self.main_loop_sprite_preparation_completed,
            "a terminal closing spotlight caller cannot replay a completed common suffix",
        );
        assert!(
            self.original_timing_main_loop_interruption().is_none(),
            "a terminal closing spotlight caller cannot also interrupt the main loop",
        );
        assert_eq!(
            self.original_timing_nmi_publication_pending, publication_pending_at_entry,
            "a terminal spotlight caller disagrees about same-host versus carried Held ownership",
        );
        assert_eq!(
            self.original_timing_pending_nmi_update_gate,
            publication_pending_at_entry.then_some(NmiUpdateGate::LatchHeld),
            "a terminal spotlight caller retained the wrong Held gate owner",
        );
        if publication_pending_at_entry {
            assert!(
                self.display_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
                "a terminal spotlight caller lost its receptive carried-Held snapshot",
            );
        }
        // A Build terminal may follow one host after a wire-held close entry
        // whose completion published `AdvanceStaged` (route host 76634, the
        // cave-entry close). That staged generation is the entry's own table
        // scanout still in the ordinary one-generation-deferred pipeline, not
        // a stale leak; the Build's typed capture below promotes it exactly
        // as hardware scans it out. The cold A/V gate arbitrates the pixels.
        let mut expected_gates = vec![NmiUpdateGate::LatchHeld];
        if trailing_open {
            expected_gates.push(NmiUpdateGate::Open);
        }
        assert_eq!(
            self.original_timing_expected_nmi_update_gates.as_slice(),
            expected_gates,
            "a terminal closing spotlight caller disagrees with its installed NMI gate authority",
        );
        assert!(
            self.game_state.display.nmi_update_is_latched(),
            "a terminal closing spotlight caller's held handler disagrees with the native latch",
        );
        assert!(
            !self.original_timing_scheduled_nmi_accepted_at_host_return,
            "a terminal closing spotlight caller cannot overlap an older staged acceptance",
        );
        assert_eq!(
            self.original_timing_sprite_main_return_claims_remaining, None,
            "a terminal closing spotlight caller cannot overlap an older Sprite_Main claim scope",
        );
        assert!(
            !self.original_timing_dungeon_exit_spotlight_entry_return_pending,
            "a terminal closing spotlight caller cannot retain an already-consumed entry return",
        );

        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .expect("a terminal closing spotlight caller lost its semantic authority")
            .semantic()
            .to_vec();
        let mut expected_semantic = timeline
            .nmi_phases_before_return
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
        if let Some(claim) = spotlight_claim {
            assert_eq!(
                timeline.nmi_phases_before_return.first(),
                Some(&OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld)),
                "a terminal spotlight re-checkpoint requires its interrupting Held acceptance first",
            );
            expected_semantic.insert(
                1,
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(claim),
            );
        }
        expected_semantic.extend([
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ]);
        expected_semantic.extend(timeline.nmi_phases_after_return.iter().map(
            |phase| match phase {
                OriginalTimingNmiPhase::Accepted(gate) => {
                    OriginalTimingSemanticReceipt::NmiAccepted(*gate)
                }
                OriginalTimingNmiPhase::HandlerCompleted => {
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted
                }
            },
        ));
        if caller_return_token {
            expected_semantic
                .push(OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait);
        }
        if entry_return_token {
            expected_semantic
                .push(OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned);
        }
        if overworld_goal_return_token {
            expected_semantic
                .push(OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned);
        }
        assert_eq!(
            semantic, expected_semantic,
            "a terminal closing spotlight caller published an unsupported or reordered semantic vector",
        );
        // The entry-return token's consumption belongs to the pre-entry
        // owner between this preflight and terminal execution; the stored
        // authority snapshot therefore excludes it.
        let semantic = semantic
            .into_iter()
            .filter(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned
                )
            })
            .collect::<Vec<_>>();

        let mut scheduler_before_completion = self.game_execution_scheduler;
        scheduler_before_completion.begin_host_frame();
        assert_eq!(
            scheduler_before_completion.current_work(),
            Some(work),
            "host normalization changed the terminal closing spotlight caller",
        );
        let mut scheduler_after_completion = scheduler_before_completion;
        assert_eq!(
            scheduler_after_completion
                .advance_work_one_nmi_slice_with_authoritative_completion(true),
            Some(GameWorkStep::Complete(work)),
            "the source-proven spotlight caller return did not complete its scheduled owner",
        );
        assert!(
            scheduler_after_completion.is_idle(),
            "the terminal closing spotlight caller left successor scheduled work",
        );

        if let OriginalTimingTerminalSpotlightCpuAction::CompleteBuild {
            mut table_build,
            projection_completed,
        } = cpu_action
        {
            // The wire-last return supersedes the scheduler estimate, but it
            // cannot make a malformed saved C continuation safe. Run the
            // exact CPU-only callback on a clone before the carried handler
            // consumes any receipt or mutates its receptive display.
            let mut cpu_probe = self.clone();
            cpu_probe.game_execution_scheduler = scheduler_after_completion;
            let suffix_before = cpu_probe.pending_main_loop_common_suffix;
            let latch_before = cpu_probe.game_state.display.nmi_update_is_latched();
            let epoch_before = cpu_probe.display_snapshot_epoch;
            let gates_before = cpu_probe.original_timing_expected_nmi_update_gates.clone();
            let semantic_before = cpu_probe
                .original_timing_semantic_receipts
                .as_ref()
                .map(|receipts| receipts.semantic().to_vec());
            let publication_before = (
                cpu_probe.original_timing_nmi_publication_pending,
                cpu_probe.original_timing_pending_nmi_update_gate,
            );
            if let Some(claim) = spotlight_claim {
                if table_build.source_progress != Some(claim.progress) {
                    table_build =
                        crate::cycle_ledger::muted(|| cpu_probe.begin_iris_spotlight_configure_table_at_progress(claim.progress));
                }
            }
            crate::cycle_ledger::muted(|| cpu_probe.complete_dungeon_exit_spotlight_build_cpu(table_build, projection_completed));
            assert!(
                cpu_probe.game_execution_scheduler.is_idle(),
                "the terminal spotlight Build CPU callback scheduled successor work",
            );
            assert_eq!(
                cpu_probe.pending_main_loop_common_suffix, suffix_before,
                "the terminal spotlight Build CPU callback consumed its central suffix",
            );
            assert_eq!(
                cpu_probe.game_state.display.nmi_update_is_latched(),
                latch_before,
                "the terminal spotlight Build CPU callback changed its carried Held latch",
            );
            assert_eq!(
                cpu_probe.display_snapshot_epoch, epoch_before,
                "the terminal spotlight Build CPU callback published a display boundary",
            );
            assert_eq!(
                cpu_probe.original_timing_expected_nmi_update_gates, gates_before,
                "the terminal spotlight Build CPU callback consumed gate authority",
            );
            assert_eq!(
                cpu_probe
                    .original_timing_semantic_receipts
                    .as_ref()
                    .map(|receipts| receipts.semantic().to_vec()),
                semantic_before,
                "the terminal spotlight Build CPU callback consumed semantic authority",
            );
            assert_eq!(
                (
                    cpu_probe.original_timing_nmi_publication_pending,
                    cpu_probe.original_timing_pending_nmi_update_gate,
                ),
                publication_before,
                "the terminal spotlight Build CPU callback changed carried-NMI ownership",
            );
        }

        Some(OriginalTimingTerminalSpotlightIterationPlan {
            semantic,
            timeline,
            work,
            iteration,
            cpu_action,
            spotlight_claim,
            caller_return_token,
            entry_return_token,
            overworld_goal_return_token,
            scheduler_before_completion,
            scheduler_after_completion,
        })
    }

    pub(super) fn take_original_timing_spotlight_table_build_progress(
        &mut self,
    ) -> Option<SpotlightTableBuildProgressReceipt> {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            return None;
        }
        let receipts = self.original_timing_semantic_receipts.as_mut()?;
        let matches = receipts
            .semantic
            .iter()
            .enumerate()
            .filter_map(|(index, receipt)| match receipt {
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(receipt) => {
                    Some((index, *receipt))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot publish multiple spotlight table checkpoints",
        );
        matches.first().map(|&(index, receipt)| {
            receipts.semantic.remove(index);
            receipt
        })
    }

    pub(super) fn take_original_timing_dungeon_exit_spotlight_caller_returned(&mut self) -> bool {
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
                    OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot return the same spotlight caller twice",
        );
        matches.first().is_some_and(|&index| {
            receipts.semantic.remove(index);
            true
        })
    }

    pub(super) fn take_original_timing_overworld_spotlight_goal_caller_returned(&mut self) -> bool {
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
                    OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert!(
            matches.len() <= 1,
            "one host call cannot return the same overworld spotlight goal caller twice",
        );
        matches.first().is_some_and(|&index| {
            receipts.semantic.remove(index);
            true
        })
    }

    pub(super) fn take_original_timing_dungeon_exit_spotlight_entry_returned(&mut self) -> bool {
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
                OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned
            ) {
                assert!(!returned, "dungeon-exit spotlight entry return replayed");
                returned = true;
                false
            } else {
                true
            }
        });
        returned
    }

    /// Bind an early source return to the matching native semantic owner.
    ///
    /// The pinned source can return from the entry call before the translated
    /// scheduler has constructed its continuation. The per-host receipt must
    /// still expire, but its completed-call fact cannot be discarded or
    /// replayed into a later Module15 invocation.
    pub(super) fn take_or_defer_original_timing_dungeon_exit_spotlight_entry_returned(
        &mut self,
        owner_active: bool,
    ) -> bool {
        let returned = self.take_original_timing_dungeon_exit_spotlight_entry_returned();
        if owner_active {
            assert!(
                !(returned && self.original_timing_dungeon_exit_spotlight_entry_return_pending),
                "dungeon-exit spotlight entry return was published twice",
            );
            return returned
                || std::mem::take(
                    &mut self.original_timing_dungeon_exit_spotlight_entry_return_pending,
                );
        }
        if returned {
            assert!(
                matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                    && self.game_state.frame.main_module == 15,
                "dungeon-exit spotlight entry returned outside native Module15 entry transition: main={:02x} sub={:02x}",
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            );
            let native_caller_already_owns_entry_return = self.game_state.frame.submodule == 1
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishSpotlightIteration {
                        iteration: SpotlightIteration {
                            direction: SpotlightDirection::Closing,
                            phase,
                            ..
                        },
                    }) if phase.is_close_entry()
                );
            if native_caller_already_owns_entry_return {
                return false;
            }
            assert!(
                self.game_state.frame.submodule == 0
                    && self.game_execution_scheduler.current_work().is_none(),
                "dungeon-exit spotlight entry returned without its pre-entry native owner: sub={:02x} work={:?}",
                self.game_state.frame.submodule,
                self.game_execution_scheduler.current_work(),
            );
            assert!(
                !self.original_timing_dungeon_exit_spotlight_entry_return_pending,
                "dungeon-exit spotlight entry return replayed before native ownership",
            );
            self.original_timing_dungeon_exit_spotlight_entry_return_pending = true;
        }
        false
    }

    pub(super) fn schedule_spotlight_iteration_return(&mut self, iteration: SpotlightIteration) {
        if !self.rom_startup_timing() {
            return;
        }
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishSpotlightIteration { iteration },
            SPOTLIGHT_ITERATION_SUFFIX_NMI_SLICES,
        );
    }

    pub(super) fn schedule_overworld_spotlight_build(
        &mut self,
        table_build: SpotlightTableBuildContinuation,
        phase: OverworldSpotlightBuildPhase,
        projection_completed: bool,
        iteration: SpotlightIteration,
    ) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishOverworldSpotlightBuild {
                table_build,
                phase,
                projection_completed,
                iteration,
            },
            1,
        );
    }

    pub(super) fn schedule_overworld_spotlight_link_oam(&mut self, iteration: SpotlightIteration) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishOverworldSpotlightLinkOam { iteration },
            1,
        );
    }

    pub(super) fn schedule_game_over_spotlight_build(
        &mut self,
        table_build: SpotlightTableBuildContinuation,
        entry: bool,
        iteration: SpotlightIteration,
    ) {
        if !self.rom_startup_timing() {
            return;
        }
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishGameOverSpotlightBuild {
                table_build,
                entry,
                iteration,
            },
            1,
        );
    }

    pub(super) fn stage_spotlight_scanout_for_next_display(&mut self) {
        self.next_display_spotlight_scanout = Some(LiveSpotlightScanout::capture(self));
    }

    pub(super) fn stage_spotlight_scanout_after_active_field(&mut self) {
        self.spotlight_scanout_after_active_field = Some(LiveSpotlightScanout::capture(self));
    }

    pub(super) fn stage_rom_spotlight_scanout_after_active_field(
        &mut self,
        words: &[u16; SPOTLIGHT_VISIBLE_SCANLINES],
    ) {
        self.spotlight_scanout_after_active_field =
            Some(LiveSpotlightScanout::capture(self).with_authoritative_rom_hdma_words(words));
    }

    /// Publish the exact channel-7 rows consumed after the first NMI in a
    /// recurring dungeon-exit spotlight call. When the C caller has already
    /// crossed a leading NMI, that field is the scanout captured earlier in
    /// this host callback; otherwise it is the next staged scanout.
    pub(super) fn publish_or_stage_spotlight_active_field(
        &mut self,
        words: &[u16; SPOTLIGHT_VISIBLE_SCANLINES],
    ) {
        let scanout = LiveSpotlightScanout::capture(self).with_authoritative_rom_hdma_words(words);
        if self
            .game_execution_scheduler
            .current_main_iteration_follows_leading_nmi()
        {
            let display = self
                .display_snapshot
                .as_mut()
                .expect("leading-NMI spotlight main requires an active display snapshot");
            display.spotlight_scanout_generation =
                SpotlightScanoutGeneration::ComposeLiveAfterNmi(scanout);
        } else {
            debug_assert!(self.spotlight_scanout_after_active_field.is_none());
            self.spotlight_scanout_after_active_field = Some(scanout);
        }
    }

    /// With the maximal spotlight table, the module-15 sub-0 call's first
    /// IrisSpotlight_ConfigureTable build is interrupted by vblank. The entry
    /// frame runs the PrepExit and SpotlightInternal prefixes; the table copy,
    /// first radius write, SpotlightInternal suffix, submodule advance, and
    /// Link/OAM suffix complete on the next host frame through the scheduled
    /// continuation.
    pub(super) fn begin_dungeon_exit_spotlight_entry(
        &mut self,
        cpu_plan: Option<DungeonExitSpotlightCpuPlan>,
        live_progress: Option<SpotlightTableBuildProgress>,
        iteration: SpotlightIteration,
    ) -> bool {
        let legacy_plan = cpu_plan.filter(|plan| {
            self.rom_startup_timing() && plan.interrupted_during_table_build_or_copy()
        });
        // A live host whose wire still holds a trailing mid-module LatchHeld
        // acceptance with neither a main-loop suffix completion nor this
        // host's own entry return proves the close entry is interrupted
        // inside its first IrisSpotlight_ConfigureTable build: the table
        // copy, radius write, submodule advance, and Link/OAM suffix belong
        // to the next host's DungeonExitSpotlightEntryReturned (route host
        // 64253, oracle SUBMODULE write at frame 64253 V=221).
        let live_interrupted_mid_entry = legacy_plan.is_none()
            && live_progress.is_none()
            && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            // The pre-entry owner may already have deferred this host's entry
            // return before the module body runs (route host 252089): the
            // entry completed before the trailing Held acceptance.
            && !self.original_timing_dungeon_exit_spotlight_entry_return_pending
            && (self
                .original_timing_expected_nmi_update_gates
                .contains(&NmiUpdateGate::LatchHeld)
                || self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .is_some_and(|receipts| {
                        receipts.semantic().iter().any(|receipt| {
                            matches!(
                                receipt,
                                OriginalTimingSemanticReceipt::NmiAccepted(
                                    NmiUpdateGate::LatchHeld
                                )
                            )
                        })
                    }))
            && !self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    receipts.semantic().iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                                | OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned
                        )
                    })
                });
        if legacy_plan.is_none() && live_progress.is_none() && !live_interrupted_mid_entry {
            return false;
        }
        self.spotlight_internal_before_table(0x7e, 0);
        let table_build = match live_progress {
            Some(progress) => self.begin_iris_spotlight_configure_table_at_progress(progress),
            None => self.begin_iris_spotlight_configure_table(
                legacy_plan.map_or(0, |plan| plan.iterations_before_nmi),
            ),
        };
        // The isolated ROM run records what channel 7 actually consumed after
        // the interrupt while $00:f3b7 copied $7f:7000 to $7e:1b00. Compose
        // those row receipts with the translated controls instead of choosing
        // either whole CPU buffer as a display-generation approximation.
        self.next_display_spotlight_scanout = Some(match legacy_plan {
            Some(plan) => LiveSpotlightScanout::capture(self)
                .with_authoritative_rom_hdma_words(&plan.active_window_words),
            None => LiveSpotlightScanout::capture(self),
        });
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonExitSpotlightEntry {
                table_build,
                iteration,
            },
            1,
        );
        true
    }

    pub(super) fn begin_dungeon_exit_spotlight_build(
        &mut self,
        cpu_plan: Option<DungeonExitSpotlightCpuPlan>,
        live_progress: Option<SpotlightTableBuildProgress>,
        iteration: SpotlightIteration,
    ) -> bool {
        let legacy_plan = cpu_plan.filter(|plan| {
            self.rom_startup_timing() && plan.interrupted_during_table_build_or_copy()
        });
        if legacy_plan.is_none() && live_progress.is_none() {
            return false;
        }
        let live_projection_will_return_before_next_nmi = live_progress.is_some_and(|progress| {
            matches!(
                progress.checkpoint,
                crate::SpotlightTableBuildCheckpoint::ProjectionCopy { .. }
            )
        });
        let table_build = match live_progress {
            Some(progress) => self.begin_iris_spotlight_configure_table_at_progress(progress),
            None => self.begin_iris_spotlight_configure_table(
                legacy_plan
                    .expect("spotlight build requires one timing authority")
                    .iterations_before_nmi,
            ),
        };
        let projection_completed =
            legacy_plan.is_some_and(DungeonExitSpotlightCpuPlan::interrupted_during_table_copy);
        if projection_completed {
            self.complete_iris_spotlight_table_projection(table_build);
        }
        // Once the row-pair loop has finished, even the full 224-word memcpy
        // plus the Module0F suffix returns before the following vblank. The
        // interrupting projection receipt therefore owns the only suspended
        // boundary; completing it must not manufacture a second host hold.
        let iteration = if live_projection_will_return_before_next_nmi {
            iteration.with_main_loop_sprite_preparation_before_second_nmi()
        } else {
            iteration
        };
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                iteration,
            },
            1,
        );
        true
    }

    pub(super) fn schedule_dungeon_exit_spotlight_link_oam(
        &mut self,
        iteration: SpotlightIteration,
    ) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { iteration },
            1,
        );
    }

    pub(super) fn schedule_dungeon_exit_spotlight_goal_caller(
        &mut self,
        iteration: SpotlightIteration,
    ) {
        // Snes9x enters the C IrisSpotlight_ConfigureTable goal call at V=21.
        // The active field is already scanning, so the call's first NMI
        // crossing publishes only a future field even though it suspends the
        // translated caller stack.
        self.game_execution_scheduler
            .schedule_cpu_timed_work_after_active_field_started(
                GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller { iteration },
                DUNGEON_EXIT_SPOTLIGHT_GOAL_CALLER_NMI_SLICES,
            );
    }

    pub(super) fn publish_completed_spotlight_hdma_table_to_active_scanout(
        &mut self,
        active_table: Vec<u8>,
    ) {
        let Some(display) = self.display_snapshot.as_mut() else {
            return;
        };
        // An explicit ROM CPU/HDMA receipt already records the table word
        // consumed on every scanline. A whole-table publication is only used
        // when the ROM run proves that the completed copy/reset preceded the
        // active field.
        let has_authoritative_rom_receipt = matches!(
            &display.spotlight_scanout_generation,
            SpotlightScanoutGeneration::ComposeLiveAfterNmi(scanout)
                if scanout.authoritative_rom_hdma_receipt
        );
        if !has_authoritative_rom_receipt {
            display.hdma_table_generation =
                DisplayHdmaTableGeneration::SpotlightPublishedAheadOfSnapshot { active_table };
        }
    }

    pub(super) fn project_following_spotlight_tail_to_active_scanout(
        &mut self,
        phase: SpotlightIterationPhase,
        use_published_prefix: bool,
    ) {
        let live_tables = spotlight_hdma_tables_from_ram(&self.ram);
        let before_projection = if use_published_prefix {
            self.display_snapshot
                .as_ref()
                .map(|display| display.effective_spotlight_hdma_tables())
                .unwrap_or_else(|| live_tables.clone())
        } else {
            live_tables.clone()
        };
        let mut after_projection = live_tables;
        let vertical_center = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        let radius = self.game_state.display.spotlight_hdma.window_radius();
        let live_tail_start = spotlight_mixed_scanout_live_tail_start(vertical_center, radius);
        if matches!(
            phase,
            SpotlightIterationPhase::MixedTailAfterReturn
                | SpotlightIterationPhase::WholeTableAfterTablePublication
        ) {
            // The fixed 448-byte copy crosses HDMA at scanline 221. HDMA has
            // already consumed the published table above that line; from the
            // crossing onward it reads the table which just completed in WRAM.
            // `WholeTableAfterTablePublication` reaches this point after the
            // radius has advanced, so rebuilding its tail would incorrectly
            // use the following iteration's radius instead of the authored
            // table still present in WRAM.
        } else {
            // Follow the ROM builder's paired lower/upper cursors exactly. The
            // lower cursor starts at max(2*center, 224), so its radial operand
            // is not equivalent to abs(scanline-center) at the bottom edge.
            let mut lower_cursor = vertical_center.wrapping_mul(2).max(224);
            let mut upper_cursor = vertical_center.wrapping_mul(2).wrapping_sub(lower_cursor);
            let y_upper = vertical_center.wrapping_add(radius);
            let mut radial_operand = radius;
            loop {
                let value = if lower_cursor < y_upper {
                    let operand = radial_operand as u8;
                    radial_operand = radial_operand.saturating_sub(1);
                    // A display-side derivation of the tail: the ROM builder
                    // already charged these calculations.
                    self.iris_spotlight_calculate_circle_value_priced(operand, false)
                } else {
                    0x00ff
                };
                for scanline in [upper_cursor, lower_cursor] {
                    let scanline = scanline as usize;
                    if (live_tail_start..224).contains(&scanline) {
                        for table in &mut after_projection {
                            let offset = scanline * 2;
                            table[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
                        }
                    }
                }
                if upper_cursor == vertical_center {
                    break;
                }
                upper_cursor = upper_cursor.wrapping_add(1);
                lower_cursor = lower_cursor.wrapping_sub(1);
            }
        }
        if let Some(display) = self.display_snapshot.as_mut() {
            display.hdma_table_generation =
                DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
                    before_projection,
                    after_projection,
                    live_tail_start,
                };
            if phase == SpotlightIterationPhase::WholeTableAfterTablePublication
                && self.game_state.frame.main_module == 15
                && !spotlight_table_has_long_nmi_workload(vertical_center)
            {
                // The C iris table copy completed during this active field,
                // but its LinkOam_Main/Main_PrepSpritesForNmi caller suffix
                // only prepares the following one. Project OBJ provenance at
                // the same hardware boundary as the HDMA tail: sprite
                // evaluation keeps the previously presented OAM, while the
                // decoded Link batch uses the captured host operands.
                display.oam_scanout_source = OamScanoutSource::RetainPreviousPresented;
                display.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
                display.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
            }
        }
    }
}
