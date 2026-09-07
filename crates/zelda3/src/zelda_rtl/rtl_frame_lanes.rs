//! Lanes of the original-timing host-frame dispatcher
//! (`run_frame_internal_after_original_timing_body`), one method per
//! early-returning block. Each lane receives the dispatcher locals it reads
//! and reports whether it completed the host frame.

use super::*;
use crate::game_state::FrameState;

impl ZeldaState {
    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_live_owner_main_loop_return_timeline(
        &mut self,
        prospective_terminal_ground_item_receipt_plan: Option<
            OriginalTimingTerminalGroundItemReceiptPlan,
        >,
        prospective_terminal_spotlight_iteration_plan: Option<
            OriginalTimingTerminalSpotlightIterationPlan,
        >,
    ) {
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self.original_timing_main_loop_return_timeline().is_some()
        {
            if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishItemReceiptGraphics {
                    continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                        ground_apress_tail: Some(_),
                        ..
                    },
                })
            ) {
                // A receipt whose caller already completed with no ground
                // A-press tail (route host 514805) only loads graphics in
                // later NMIs; its hosts run ordinary fresh iterations.
                assert!(
                    prospective_terminal_ground_item_receipt_plan.is_some(),
                    "a terminal atomic item caller published unsupported ground-item ownership",
                );
            }
            if self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
            {
                assert!(
                    prospective_terminal_ground_item_receipt_plan.is_some()
                        || prospective_terminal_spotlight_iteration_plan.is_some()
                        || self
                            .game_execution_scheduler
                            .pre_main_caller_continuation()
                            .is_some()
                        || self.game_execution_scheduler.current_work().is_some_and(
                            |work| {
                                work.scheduled_caller_return_timeline_owns_terminal_return()
                                    || matches!(
                                        work,
                                        GameWorkContinuation::FinishDialogueInitializationCallerReturn
                                            | GameWorkContinuation::FinishDialogueInitializationPrefix { .. }
                                            | GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. }
                                            | GameWorkContinuation::FinishWorldMapOverlayReload
                                            | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement { .. }
                                            | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel { .. }
                                            | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow { .. }
                                            | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates { .. }
                                            | GameWorkContinuation::FinishSpriteMain { .. }
                                            | GameWorkContinuation::FinishDungeonSupertileTransition { .. }
                                    )
                            }
                        ),
                    "a terminal source return cannot be consumed by an unsupported scheduled caller",
                );
            }
        }
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_scheduled_finish_sprite_main_boundary(
        &mut self,
        input: u16,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if let Some(GameWorkContinuation::FinishSpriteMain {
            boundary: scheduled_sprite_main_boundary,
            ..
        }) = self.game_execution_scheduler.current_work()
        {
            // A host-return-scheduled Sprite_Main suspension (the caller was
            // interrupted before its first slot, or a typed host-boundary
            // checkpoint) whose next host carries no Sprite_Main return,
            // progress or return timeline: the ROM's caller is still inside
            // the long module work preceding Sprite_Main (Module0B/$18
            // Overworld_Func18 spans route hosts 165774-165776; Sprite_Main
            // returns at 165777). Complete the carried handler and stay held
            // without advancing the scheduled slice.
            if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
                && match self.original_timing_main_loop_interruption() {
                    None => true,
                    Some(interruption) => same_optional_sprite_main_source_checkpoint(
                        sprite_main_cpu_boundary_from_interruption(interruption),
                        Some(scheduled_sprite_main_boundary),
                    ),
                }
                && !self.original_timing_owes_sprite_main_return()
                && (!self.original_timing_owes_sprite_main_progress()
                    || self.original_timing_restates_sprite_main_checkpoint(
                        scheduled_sprite_main_boundary,
                    ))
                && self.original_timing_main_loop_return_timeline().is_none()
                && self.original_timing_main_loop_progress()
                    == Some(crate::MainLoopProgress::CallStackContinued)
                && self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .is_some_and(|receipts| {
                        !receipts.semantic().iter().any(|receipt| {
                            matches!(
                                receipt,
                                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                                    | OriginalTimingSemanticReceipt::NmiAccepted(
                                        NmiUpdateGate::Open
                                    )
                                    | OriginalTimingSemanticReceipt::JoypadPublication(_)
                                    | OriginalTimingSemanticReceipt::MainLoopProgress(
                                        crate::MainLoopProgress::IterationStarted
                                    )
                            )
                        })
                    })
            {
                let phases = self.take_original_timing_nmi_phases();
                let before = classify_original_timing_nmi_phases_with_ownership(
                    self.original_timing_nmi_publication_pending,
                    &phases,
                );
                self.complete_original_timing_nmi_handler_for_active_scanout(
                    before.handler_completion,
                    input,
                    oam_dma_source.as_deref(),
                )
                .assert_no_unclaimed_dialogue_text_dma();
                let reload_progress = self.take_original_timing_overworld_sprite_reload_progress();
                assert!(
                    !self.apply_original_timing_overworld_sprite_reload_progress(reload_progress),
                    "a held scheduled Sprite_Main host cannot publish its caller's reload return",
                );
                assert_eq!(
                    self.take_original_timing_main_loop_progress(),
                    Some(crate::MainLoopProgress::CallStackContinued),
                    "a held scheduled Sprite_Main host must continue its suspended stack",
                );
                if self.original_timing_owes_sprite_main_progress() {
                    let restated = self
                        .take_original_timing_sprite_main_progress()
                        .expect("restated Sprite_Main checkpoint disappeared");
                    assert!(
                        same_sprite_main_source_checkpoint(
                            restated,
                            scheduled_sprite_main_boundary,
                        ),
                        "a held scheduled Sprite_Main host restated a different checkpoint than its suspension",
                    );
                }
                if let Some(reinterrupted) = self.take_original_timing_main_loop_interruption_any()
                {
                    assert!(
                        same_optional_sprite_main_source_checkpoint(
                            sprite_main_cpu_boundary_from_interruption(reinterrupted),
                            Some(scheduled_sprite_main_boundary),
                        ),
                        "a held scheduled Sprite_Main host was re-interrupted at a different checkpoint",
                    );
                }
                assert!(
                    self.original_timing_semantic_receipts
                        .as_ref()
                        .is_none_or(|receipts| receipts.semantic().is_empty()),
                    "a held scheduled Sprite_Main host left unconsumed semantic authority",
                );
                if before.publication_pending_at_exit {
                    self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                }
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_after_current_trailing_nmi_continuation(
        &mut self,
        input: u16,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if let Some(continuation) = self
            .game_execution_scheduler
            .take_after_current_trailing_nmi()
        {
            let live_nonterminal_peg_flip = continuation
                == GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn
                && self.dungeon_peg_attribute_flip_pending.is_some()
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
                && self.original_timing_main_loop_return_timeline().is_none()
                && self.original_timing_main_loop_progress()
                    == Some(crate::MainLoopProgress::CallStackContinued);
            if live_nonterminal_peg_flip {
                // The saved stack resumed after the preceding trailing NMI,
                // but the 8,192-store attribute walk can consume another
                // complete frontend host without returning or accepting a
                // second NMI (route host 711898). Advance to the wire's exact
                // source cursor and move the still-live stack into the normal
                // scheduled lane; completing the continuation here would run
                // state 8's scroll several hosts before $01:c229 returns.
                let phases = self.take_original_timing_nmi_phases();
                let lifecycle = classify_original_timing_nmi_phases_with_ownership(
                    self.original_timing_nmi_publication_pending,
                    &phases,
                );
                self.complete_original_timing_nmi_handler_for_active_scanout(
                    lifecycle.handler_completion,
                    input,
                    oam_dma_source.as_deref(),
                )
                .assert_no_unclaimed_dialogue_text_dma();
                let next = self
                    .take_original_timing_dungeon_peg_attribute_flip_progress()
                    .expect("a continued selectable peg flip lost its source cursor");
                let mut pending = self
                    .dungeon_peg_attribute_flip_pending
                    .take()
                    .expect("a continued selectable peg flip lost its saved caller");
                self.advance_dungeon_peg_attribute_flip_between_progress(pending.progress, next);
                pending.progress = next;
                self.dungeon_peg_attribute_flip_pending = Some(pending);
                assert_eq!(
                    self.take_original_timing_main_loop_progress(),
                    Some(crate::MainLoopProgress::CallStackContinued),
                    "a nonterminal peg-flip host must retain the interrupted call stack",
                );
                assert!(
                    self.original_timing_semantic_receipts
                        .as_ref()
                        .is_none_or(|receipts| receipts.semantic().is_empty()),
                    "a nonterminal peg-flip host left unconsumed semantic authority",
                );
                self.game_execution_scheduler.schedule_work(continuation, 1);
                if lifecycle.publication_pending_at_exit {
                    self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                }
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                debug_host_path_early_return(self.frame_ctr_dbg, line!());
                return true;
            }
            let held_sprite_main_boundary = match continuation {
                GameWorkContinuation::FinishSpriteMain { boundary, .. } => Some(boundary),
                _ => None,
            };
            if held_sprite_main_boundary.is_some()
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
                && match self.original_timing_main_loop_interruption() {
                    None => true,
                    // A held host can be re-interrupted at the SAME suspended
                    // statement: another held vblank lands while the ROM has
                    // not moved past the checkpoint (route host 102850). Any
                    // other interruption keeps its refining owner — except a
                    // re-interruption at a LATER slot of the same descending
                    // loop, which the progress receipt restates and the body
                    // advances to below (route hosts 891726, 1172216).
                    Some(interruption) => {
                        let interrupted = sprite_main_cpu_boundary_from_interruption(interruption);
                        same_optional_sprite_main_source_checkpoint(
                            interrupted,
                            held_sprite_main_boundary,
                        ) || (interrupted.is_some()
                            && interrupted == self.original_timing_sprite_main_progress_boundary()
                            && sprite_main_in_flight_checkpoint_advances(
                                held_sprite_main_boundary
                                    .expect("held Sprite_Main continuation lost its boundary"),
                                interrupted.expect("interruption boundary checked above"),
                            ))
                    }
                }
                && !self.original_timing_owes_sprite_main_return()
                && (!self.original_timing_owes_sprite_main_progress()
                    // The wire may re-checkpoint the SAME suspended boundary
                    // at the host's end: the ROM stayed at that statement
                    // through this whole host (route host 102849). A NEW
                    // boundary stays on the bus for its refining owner —
                    // except the first-slot suspension, whose loop this host
                    // ran down to a later slot (route host 402689:
                    // BeforeFirstSlot -> AfterSlot(4)), refined below.
                    || self.original_timing_restates_sprite_main_checkpoint(
                        held_sprite_main_boundary
                            .expect("held Sprite_Main continuation lost its boundary"),
                    )
                    || self.original_timing_sprite_main_progress_boundary().is_some_and(
                        |progress| {
                            sprite_main_in_flight_checkpoint_advances(
                                held_sprite_main_boundary
                                    .expect("held Sprite_Main continuation lost its boundary"),
                                progress,
                            )
                        },
                    ))
                && self.original_timing_main_loop_return_timeline().is_none()
                && self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .is_some_and(|receipts| {
                        !receipts.semantic().iter().any(|receipt| {
                            matches!(
                                receipt,
                                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                                    | OriginalTimingSemanticReceipt::NmiAccepted(
                                        NmiUpdateGate::Open
                                    )
                                    | OriginalTimingSemanticReceipt::JoypadPublication(_)
                                    | OriginalTimingSemanticReceipt::MainLoopProgress(
                                        crate::MainLoopProgress::IterationStarted
                                    )
                            )
                        })
                    })
            {
                // A host whose vector carries only the carried handler and a
                // continued-stack fact keeps the suspended Sprite_Main in
                // flight with no new checkpoint: the resumed body ran out
                // the host budget without an NMI or a named boundary (route
                // host 91640). Complete the handler and stay held.
                let phases = self.take_original_timing_nmi_phases();
                let before = classify_original_timing_nmi_phases_with_ownership(
                    self.original_timing_nmi_publication_pending,
                    &phases,
                );
                self.complete_original_timing_nmi_handler_for_active_scanout(
                    before.handler_completion,
                    input,
                    oam_dma_source.as_deref(),
                )
                .assert_no_unclaimed_dialogue_text_dma();
                assert_eq!(
                    self.take_original_timing_main_loop_progress(),
                    Some(crate::MainLoopProgress::CallStackContinued),
                    "an un-named held Sprite_Main host must continue its suspended stack",
                );
                let mut continuation = continuation;
                if self.original_timing_owes_sprite_main_progress() {
                    // Retire the wire's same-boundary re-checkpoint; the
                    // suspension continues from the identical statement
                    // (route host 102849).
                    let restated = self
                        .take_original_timing_sprite_main_progress()
                        .expect("restated Sprite_Main checkpoint disappeared");
                    match (held_sprite_main_boundary, restated, continuation) {
                                                (
                            Some(SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(completed_slot)),
                            SpriteMainCpuBoundary::AfterTimersAndOam { slot, state: None },
                            GameWorkContinuation::FinishSpriteMain { caller, .. },
                        ) if slot < completed_slot => {
                            self.complete_waterfall_gt_cutscene_graphics(usize::from(completed_slot));
                            let boundary = self.advance_sprite_main_after_slot_to_after_timers(completed_slot, slot);
                            continuation = GameWorkContinuation::FinishSpriteMain { boundary, caller };
                        }
                        (
                            Some(SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(completed_slot)),
                            SpriteMainCpuBoundary::AfterSlot(slot),
                            GameWorkContinuation::FinishSpriteMain { caller, .. },
                        ) if slot <= completed_slot => {
                            self.complete_waterfall_gt_cutscene_graphics(usize::from(completed_slot));
                            if slot < completed_slot {
                                self.advance_sprite_main_after_slot_boundary(completed_slot, slot);
                            }
                            continuation = GameWorkContinuation::FinishSpriteMain { boundary: restated, caller };
                        }
(
                            Some(SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(completed_slot) | SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(completed_slot)),
                            SpriteMainCpuBoundary::AfterTimersAndOam { slot, state: None },
                            GameWorkContinuation::FinishSpriteMain { caller, .. },
                        ) if slot < completed_slot => {
                            self.complete_happiness_pond_rupee_graphics(usize::from(completed_slot));
                            let boundary = self.advance_sprite_main_after_slot_to_after_timers(completed_slot, slot);
                            continuation = GameWorkContinuation::FinishSpriteMain { boundary, caller };
                        }
                        (
                            Some(SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(completed_slot) | SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(completed_slot)),
                            SpriteMainCpuBoundary::AfterSlot(slot),
                            GameWorkContinuation::FinishSpriteMain { caller, .. },
                        ) if slot <= completed_slot => {
                            self.complete_happiness_pond_rupee_graphics(usize::from(completed_slot));
                            if slot < completed_slot {
                                self.advance_sprite_main_after_slot_boundary(completed_slot, slot);
                            }
                            continuation = GameWorkContinuation::FinishSpriteMain { boundary: restated, caller };
                        }
                        (
                            Some(SpriteMainCpuBoundary::BeforeFirstSlot),
                            SpriteMainCpuBoundary::AfterSlot(newly_completed_slot),
                            GameWorkContinuation::FinishSpriteMain { caller, .. },
                        ) => {
                            // The suspended Sprite_Main ran its descending loop
                            // down to the wire's newly returned slot before
                            // vblank held it again (route host 402689).
                            self.advance_sprite_main_before_first_slot_to_after_slot(
                                newly_completed_slot,
                            );
                            continuation = GameWorkContinuation::FinishSpriteMain {
                                boundary: restated,
                                caller,
                            };
                        }
                        (
                            Some(SpriteMainCpuBoundary::AfterSlot(completed_slot)),
                            SpriteMainCpuBoundary::AfterSlot(newly_completed_slot),
                            GameWorkContinuation::FinishSpriteMain { caller, .. },
                        ) if newly_completed_slot < completed_slot => {
                            // Likewise from a mid-loop suspension (route host
                            // 1172216: AfterSlot(6) -> AfterSlot(4)).
                            self.advance_sprite_main_after_slot_boundary(
                                completed_slot,
                                newly_completed_slot,
                            );
                            continuation = GameWorkContinuation::FinishSpriteMain {
                                boundary: restated,
                                caller,
                            };
                        }
                        (
                            Some(SpriteMainCpuBoundary::FollowerGraphics {
                                slot,
                                caller: graphics_caller,
                                prefix_completed: true,
                                saved_follower_indicator,
                                stage: prior_stage,
                            }),
                            SpriteMainCpuBoundary::FollowerGraphics {
                                slot: refined_slot,
                                caller: refined_caller,
                                prefix_completed: false,
                                saved_follower_indicator: None,
                                stage: next_stage,
                            },
                            GameWorkContinuation::FinishSpriteMain { caller, .. },
                        ) if slot == refined_slot && graphics_caller == refined_caller => {
                            self.apply_follower_graphics_progress(Some(prior_stage), next_stage);
                            continuation = GameWorkContinuation::FinishSpriteMain {
                                boundary: SpriteMainCpuBoundary::FollowerGraphics {
                                    slot,
                                    caller: graphics_caller,
                                    prefix_completed: true,
                                    saved_follower_indicator,
                                    stage: next_stage,
                                },
                                caller,
                            };
                        }
                        _ => assert!(
                            same_optional_sprite_main_source_checkpoint(
                                Some(restated),
                                held_sprite_main_boundary,
                            ),
                            "a held Sprite_Main host restated a different checkpoint than its suspension",
                        ),
                    }
                    if let Some(reinterrupted) = self.original_timing_main_loop_interruption() {
                        if same_optional_sprite_main_source_checkpoint(
                            sprite_main_cpu_boundary_from_interruption(reinterrupted),
                            Some(restated),
                        ) && !same_optional_sprite_main_source_checkpoint(
                            Some(restated),
                            held_sprite_main_boundary,
                        ) {
                            // The interruption receipt names the same newly
                            // reached slot; the refined continuation owns it.
                            let _ = self.take_original_timing_main_loop_interruption_any();
                        }
                    }
                }
                if let Some(reinterrupted) = self.take_original_timing_main_loop_interruption_any()
                {
                    // The re-interruption names the same suspended statement
                    // (validated above); retire it with the re-checkpoint
                    // (route host 102850).
                    assert!(
                        same_optional_sprite_main_source_checkpoint(
                            sprite_main_cpu_boundary_from_interruption(reinterrupted),
                            held_sprite_main_boundary,
                        ),
                        "a held Sprite_Main host was re-interrupted at a different checkpoint",
                    );
                }
                assert!(
                    self.original_timing_semantic_receipts
                        .as_ref()
                        .is_none_or(|receipts| receipts.semantic().is_empty()),
                    "an un-named held Sprite_Main host left unconsumed semantic authority",
                );
                self.game_execution_scheduler
                    .schedule_after_current_trailing_nmi(continuation);
                if before.publication_pending_at_exit {
                    // The mid-suspension Held acceptance's handler runs at
                    // the start of the following host (route host 91641).
                    self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                }
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }
            if matches!(
                continuation,
                GameWorkContinuation::FinishSpriteMain { .. }
                    | GameWorkContinuation::FinishDungeonCachedSpriteMain { .. }
                    | GameWorkContinuation::FinishDungeonSupertileTransition {
                        work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
                    }
                    | GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn { .. }
                    | GameWorkContinuation::FinishModule09LinkOamCallerReturn { .. }
                    | GameWorkContinuation::FinishStraightInterroomSpriteReset { .. }
            ) && self.complete_live_terminal_post_nmi_continuation(
                continuation,
                input,
                oam_dma_source.as_deref(),
            ) {
                return true;
            }
            let live_module09_sprite_main_owned_by_scheduled_lane =
                matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                    && self.original_timing_semantic_receipts.is_some()
                    && matches!(
                        continuation,
                        GameWorkContinuation::FinishSpriteMain {
                            caller: SpriteMainCpuCaller::Module09 { .. },
                            ..
                        }
                    );
            if live_module09_sprite_main_owned_by_scheduled_lane {
                // A resumed Module09 Sprite_Main whose host is neither a bare
                // hold nor a terminal return: the wire may return Sprite_Main
                // and then interrupt the caller suffix (NMI_PrepareSprites at
                // route host 202669, LinkOam at 321896) or re-interrupt the
                // loop mid-way (slot 11 at 981878). The scheduled-caller lane
                // below owns those shapes through its interruption timeline.
                self.game_execution_scheduler.schedule_work(continuation, 1);
            } else {
                // The preceding host already consumed the interrupt which
                // suspended this CPU stack. Resume it directly; inserting another
                // NMI here shifts every following dungeon state by one frame.
                match continuation {
                    GameWorkContinuation::FinishDungeonSupertileTransition {
                        work: DungeonSupertileTransitionWork::RoomLoadSpriteReset { progress },
                    } => {
                        let terminal_timeline =
                            (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                                && self.original_timing_semantic_receipts.is_some())
                            .then(|| self.take_original_timing_main_loop_return_timeline())
                            .flatten();
                        if let Some(timeline) = terminal_timeline {
                            // The wire proves the resumed sprite reset ran through
                            // the room-load caller and ZeldaRunGameLoop's suffix
                            // in this one host: the shared return executor owns
                            // its carried handler, suffix, and trailing lifecycle.
                            let sprite_main_return_claims = self
                                .original_timing_semantic_receipts
                                .as_ref()
                                .map(|receipts| {
                                    receipts
                                        .semantic()
                                        .iter()
                                        .filter(|receipt| {
                                            **receipt
                                                == OriginalTimingSemanticReceipt::SpriteMainReturned
                                        })
                                        .count()
                                })
                                .unwrap_or(0);
                            if sprite_main_return_claims != 0 {
                                self.begin_original_timing_sprite_main_return_claim_scope(
                                    sprite_main_return_claims,
                                );
                            }
                            self.complete_original_timing_main_loop_return(
                                timeline,
                                input,
                                oam_dma_source.as_deref(),
                                |state| {
                                    state.complete_room_load_sprite_reset_after_timing_boundary(
                                        progress, input, None,
                                    );
                                },
                            );
                            if sprite_main_return_claims != 0 {
                                self.finish_original_timing_sprite_main_return_claim_scope();
                            }
                        } else if matches!(
                            self.original_timing_owner,
                            OriginalTimingOwnerState::Live
                        ) && self.original_timing_semantic_receipts.is_some()
                            && self
                                .original_timing_main_loop_interruption()
                                .is_some_and(|interruption| interruption.is_sprite_main())
                        {
                            // The resumed sprite reset ran into the room-load
                            // caller's Sprite_Main, which the wire suspends at a
                            // slot boundary in this host (route host 729546,
                            // AfterSlot(1) for three hosts). The body parks that
                            // Sprite_Main; the host's lifecycle receipts — the
                            // carried handler, the continued progress, and the
                            // interrupting held acceptance — are owned here.
                            let phases = self.take_original_timing_nmi_phases();
                            let before = classify_original_timing_nmi_phases_with_ownership(
                                self.original_timing_nmi_publication_pending,
                                &phases,
                            );
                            self.complete_original_timing_nmi_handler_for_active_scanout(
                                before.handler_completion,
                                input,
                                oam_dma_source.as_deref(),
                            )
                            .assert_no_unclaimed_dialogue_text_dma();
                            assert_eq!(
                                self.take_original_timing_main_loop_progress(),
                                Some(crate::MainLoopProgress::CallStackContinued),
                                "a resumed room-load sprite reset host must continue its suspended stack",
                            );
                            self.complete_room_load_sprite_reset_after_timing_boundary(
                                progress, input, None,
                            );
                            if let Some(GameWorkContinuation::FinishSpriteMain {
                                boundary, ..
                            }) = self.game_execution_scheduler.current_work()
                            {
                                if self
                                    .original_timing_main_loop_interruption()
                                    .and_then(sprite_main_cpu_boundary_from_interruption)
                                    == Some(boundary)
                                {
                                    let _ = self.take_original_timing_main_loop_interruption_any();
                                }
                            }
                            if before.publication_pending_at_exit {
                                self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                            }
                        } else {
                            self.complete_room_load_sprite_reset_after_timing_boundary(
                                progress,
                                input,
                                oam_dma_source.as_deref(),
                            );
                        }
                    }
                    GameWorkContinuation::FinishStraightInterroomSpriteReset { progress } => {
                        self.complete_straight_interroom_sprite_reset_after_timing_boundary(
                            progress,
                        );
                    }
                    _ => {
                        // A parked dungeon Sprite_Main resumed on a live host that
                        // neither holds bare nor returns terminally (route hosts
                        // 877298, 1185490): the body owns the wire's Sprite_Main
                        // return and interruption, so the host's lifecycle
                        // receipts — the carried handler's completion, the
                        // continued progress, and a trailing held acceptance —
                        // are consumed here instead of by a same-host suffix.
                        let live_nonterminal_dungeon_sprite_main =
                            matches!(
                                continuation,
                                GameWorkContinuation::FinishSpriteMain {
                                    caller: SpriteMainCpuCaller::DungeonModule07
                                        | SpriteMainCpuCaller::DungeonModule07Live { .. },
                                    ..
                                }
                            ) && matches!(
                                self.original_timing_owner,
                                OriginalTimingOwnerState::Live
                            ) && self.original_timing_semantic_receipts.is_some()
                                && self.original_timing_main_loop_return_timeline().is_none();
                        let mut carry_publication = false;
                        if live_nonterminal_dungeon_sprite_main {
                            let phases = self.take_original_timing_nmi_phases();
                            let before = classify_original_timing_nmi_phases_with_ownership(
                                self.original_timing_nmi_publication_pending,
                                &phases,
                            );
                            self.complete_original_timing_nmi_handler_for_active_scanout(
                                before.handler_completion,
                                input,
                                oam_dma_source.as_deref(),
                            )
                            .assert_no_unclaimed_dialogue_text_dma();
                            assert_eq!(
                                self.take_original_timing_main_loop_progress(),
                                Some(crate::MainLoopProgress::CallStackContinued),
                                "a resumed parked Sprite_Main host must continue its suspended stack",
                            );
                            carry_publication = before.publication_pending_at_exit;
                        }
                        self.complete_post_trailing_nmi_continuation(
                            continuation,
                            input,
                            false,
                            false,
                        );
                        if carry_publication {
                            self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                        }
                    }
                }
                if self
                    .game_execution_scheduler
                    .resumed_call_stack_is_before_nmi()
                    && !self
                        .game_execution_scheduler
                        .work_suspends_translated_call_stack()
                {
                    // The saved C stack has reached ZeldaRunGameLoop's wait after
                    // the interrupt which suspended it. Its next hardware event
                    // is therefore a leading NMI, regardless of which synchronous
                    // caller happened to resume here.
                    self.game_execution_scheduler
                        .finish_call_stack_at_main_wait_before_nmi();
                }
                if continuation == GameWorkContinuation::FinishDialogueInitializationCallerReturn {
                    // Text_Initialize returned shortly after the preceding field
                    // began, then Module0E authored its scroll copies before the
                    // next hardware boundary returned by this frontend callback.
                    // Project that register-only boundary into the following
                    // scanout. The retained field is already complete; mutating
                    // its generation would rewrite the preceding image too.
                    // Running the whole NMI here would replay CPU-visible DMA/state
                    // which the scheduler already owns on the next host.
                    let returned_scroll = self.bg_scroll_scanout_from_nmi_register_mirrors();
                    self.publish_bg_scroll_for_following_scanout(returned_scroll);
                }
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                debug_host_path_early_return(self.frame_ctr_dbg, line!());
                return true;
            }
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_post_trailing_nmi_continuation(
        &mut self,
        input: u16,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if let Some(continuation) = self.game_execution_scheduler.take_post_trailing_nmi() {
            if continuation == GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && self.original_timing_semantic_receipts.is_some()
                && self.original_timing_main_loop_interruption().is_none()
                && !self.original_timing_owes_sprite_main_return()
                && !self.original_timing_owes_sprite_main_progress()
                && self.original_timing_main_loop_return_timeline().is_none()
                && self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .is_some_and(|receipts| {
                        !receipts.semantic().iter().any(|receipt| {
                            matches!(
                                receipt,
                                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                                    | OriginalTimingSemanticReceipt::NmiAccepted(
                                        NmiUpdateGate::Open
                                    )
                                    | OriginalTimingSemanticReceipt::JoypadPublication(_)
                                    | OriginalTimingSemanticReceipt::MainLoopProgress(
                                        crate::MainLoopProgress::IterationStarted
                                    )
                            )
                        })
                    })
            {
                // The suspended after-submodule caller stays held through this
                // whole host: another held vblank lands before its resumed
                // suffix reaches the shared suffix (route host 111181, the
                // landing-spotlight iteration). Complete the handler and stay
                // held.
                let phases = self.take_original_timing_nmi_phases();
                let before = classify_original_timing_nmi_phases_with_ownership(
                    self.original_timing_nmi_publication_pending,
                    &phases,
                );
                self.complete_original_timing_nmi_handler_for_active_scanout(
                    before.handler_completion,
                    input,
                    oam_dma_source.as_deref(),
                )
                .assert_no_unclaimed_dialogue_text_dma();
                assert_eq!(
                    self.take_original_timing_main_loop_progress(),
                    Some(crate::MainLoopProgress::CallStackContinued),
                    "a held after-submodule caller host must continue its suspended stack",
                );
                assert!(
                    self.original_timing_semantic_receipts
                        .as_ref()
                        .is_none_or(|receipts| receipts.semantic().is_empty()),
                    "a held after-submodule caller host left unconsumed semantic authority",
                );
                self.game_execution_scheduler
                    .schedule_post_trailing_nmi(continuation);
                if before.publication_pending_at_exit {
                    self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                }
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }
            if continuation == GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            {
                if let Some(timeline) = self
                    .take_original_timing_main_loop_interruption_timeline(Some(
                        crate::MainLoopInterruption::LinkOam,
                    ))
                    .filter(|timeline| {
                        timeline.progress == crate::MainLoopProgress::CallStackContinued
                    })
                {
                    // The wire keeps this post-Sprite_Main caller suspended
                    // inside LinkOam for another host: its carried handler
                    // completes, the restated Sprite_Main claim corroborates
                    // the crossing the native body already ran, and the
                    // continuation stays armed (route host 31246).
                    assert!(
                        timeline.nmi_phases_after_interruption.is_empty(),
                        "a held LinkOam caller cannot publish a post-interruption NMI lifecycle: {timeline:?}",
                    );
                    let before = classify_original_timing_nmi_phases_with_ownership(
                        self.original_timing_nmi_publication_pending,
                        &timeline.nmi_phases_before_interruption,
                    );
                    assert!(
                        !before.publication_pending_at_exit,
                        "a held LinkOam caller cannot end with an unfinished NMI handler: {timeline:?}",
                    );
                    self.complete_original_timing_nmi_handler_for_active_scanout(
                        before.handler_completion,
                        input,
                        oam_dma_source.as_deref(),
                    )
                    .assert_no_unclaimed_dialogue_text_dma();
                    let _ = self.take_original_timing_sprite_main_returned();
                    assert!(
                        self.original_timing_semantic_receipts
                            .as_ref()
                            .is_none_or(|receipts| receipts.semantic().is_empty()),
                        "a held LinkOam caller left unconsumed semantic authority",
                    );
                    self.game_execution_scheduler
                        .schedule_post_trailing_nmi(continuation);
                    self.assert_native_frame_state_matches_ram();
                    self.assert_native_world_location_state_matches_ram();
                    self.assert_native_display_state_matches_ram();
                    return true;
                }
            }
            if let GameWorkContinuation::FinishDungeonSupertileTransition {
                work: work @ DungeonSupertileTransitionWork::RoomLoadCallerResume,
            } = continuation
            {
                let live_continued_link_oam =
                    matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && self.original_timing_semantic_receipts.is_some()
                        && self.original_timing_main_loop_interruption()
                            == Some(crate::MainLoopInterruption::LinkOam)
                        && !self.original_timing_hosts_fresh_iteration();
                if let Some(timeline) = live_continued_link_oam
                    .then(|| {
                        self.take_original_timing_main_loop_interruption_timeline(Some(
                            crate::MainLoopInterruption::LinkOam,
                        ))
                    })
                    .flatten()
                    .filter(|timeline| {
                        timeline.progress == crate::MainLoopProgress::CallStackContinued
                    })
                {
                    // The resumed room-load caller runs its Sprite_Main in
                    // this host and stays suspended inside the following
                    // LinkOam_Main at the trailing Held acceptance (route
                    // host 89672). Complete the leading handler, scope the
                    // in-host Sprite_Main claim, and let the caller body
                    // suspend through its post-Sprite_Main continuation.
                    let before = classify_original_timing_nmi_phases_with_ownership(
                        self.original_timing_nmi_publication_pending,
                        &timeline.nmi_phases_before_interruption,
                    );
                    assert!(
                        timeline.nmi_phases_after_interruption.is_empty(),
                        "a suspended room-load LinkOam caller cannot publish a post-interruption NMI lifecycle: {timeline:?}",
                    );
                    self.complete_original_timing_nmi_handler_for_active_scanout(
                        before.handler_completion,
                        input,
                        oam_dma_source.as_deref(),
                    )
                    .assert_no_unclaimed_dialogue_text_dma();
                    if self.active_dungeon_sprite_main_return.is_some() {
                        // The native room-load flow executed the caller's
                        // Sprite_Main in an earlier slice; the wire's return
                        // claim restates that completed body, and the ROM
                        // stays inside the following LinkOam_Main. Keep the
                        // saved return frame and resume the whole post-
                        // Sprite_Main suffix through the shared continuation.
                        let _ = self.take_original_timing_sprite_main_returned();
                        self.game_execution_scheduler.schedule_work(
                            GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn,
                            1,
                        );
                    } else {
                        let claims = self
                            .original_timing_semantic_receipts
                            .as_ref()
                            .map(|receipts| {
                                receipts
                                    .semantic()
                                    .iter()
                                    .filter(|receipt| {
                                        **receipt
                                            == OriginalTimingSemanticReceipt::SpriteMainReturned
                                    })
                                    .count()
                            })
                            .unwrap_or(0);
                        if claims != 0 {
                            self.begin_original_timing_sprite_main_return_claim_scope(claims);
                        }
                        self.dungeon_post_sprite_main_return_pending = true;
                        self.complete_dungeon_supertile_caller_return_host(
                            work,
                            input,
                            oam_dma_source.as_deref(),
                            true,
                        );
                        if claims != 0
                            && self
                                .original_timing_sprite_main_return_claims_remaining
                                .is_some()
                        {
                            self.finish_original_timing_sprite_main_return_claim_scope();
                        }
                    }
                    assert!(
                        self.game_execution_scheduler
                            .work_suspends_translated_call_stack()
                            || self.game_execution_scheduler.current_work().is_some(),
                        "the suspended room-load LinkOam caller lost its scheduled continuation",
                    );
                    if before.publication_pending_at_exit {
                        self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                    }
                    self.assert_native_frame_state_matches_ram();
                    self.assert_native_world_location_state_matches_ram();
                    self.assert_native_display_state_matches_ram();
                    return true;
                }
            }
            if matches!(
                continuation,
                GameWorkContinuation::FinishSpriteMain { .. }
                    | GameWorkContinuation::FinishDungeonCachedSpriteMain { .. }
                    | GameWorkContinuation::FinishDungeonSupertileTransition {
                        work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
                    }
                    | GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn
                    | GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn { .. }
                    | GameWorkContinuation::FinishModule09LinkOamCallerReturn { .. }
            ) && self.complete_live_terminal_post_nmi_continuation(
                continuation,
                input,
                oam_dma_source.as_deref(),
            ) {
                return true;
            }
            // This continuation was armed after the preceding host had
            // already consumed its trailing boundary. The translated call
            // stack is still suspended, so the current host must take NMI
            // before any fresh main-loop iteration can run.
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            self.complete_post_trailing_nmi_continuation(continuation, input, false, false);
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_intro_initialization_work_frames(
        &mut self,
        input: u16,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if self.rom_startup_timing() && self.intro_initialization_work_frames_pending != 0 {
            if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
                assert_eq!(
                    self.pending_main_loop_common_suffix,
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                    "continued intro initialization lost its suspended ZeldaRunGameLoop suffix",
                );
                let expected_timeline = self
                    .original_timing_uninterrupted_main_loop_timeline(
                        crate::MainLoopProgress::CallStackContinued,
                    )
                    .unwrap_or_else(|| {
                        panic!(
                            "live nonterminal intro initialization omitted its uninterrupted continued-call timeline",
                        )
                    });
                assert!(
                    expected_timeline.nmi_phases_after_progress.is_empty(),
                    "CallStackContinued must be the terminal fact for a nonterminal intro initialization host: {expected_timeline:?}",
                );
                let nmi_phases = expected_timeline
                    .nmi_phases_before_progress
                    .iter()
                    .chain(expected_timeline.nmi_phases_after_progress.iter())
                    .copied()
                    .collect::<Vec<_>>();
                let classification = classify_original_timing_nmi_phases_with_ownership(
                    self.original_timing_nmi_publication_pending,
                    &nmi_phases,
                );
                assert_original_timing_carry_in_handler_has_receptive_display(
                    classification,
                    self.display_snapshot
                        .as_ref()
                        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
                );
                let consumed_timeline = self
                    .take_original_timing_uninterrupted_main_loop_timeline(
                        crate::MainLoopProgress::CallStackContinued,
                    )
                    .expect("the validated intro initialization timeline disappeared");
                assert_eq!(
                    consumed_timeline, expected_timeline,
                    "validated intro initialization timeline changed before consumption",
                );

                // Runs 159-160 in the pinned cold trace complete the handler
                // accepted by the preceding host, resume the long item-
                // graphics caller, return its video generation, and only then
                // accept the next held NMI. The coarse translated counter is
                // not source completion authority: keep it armed until the
                // exact common-suffix receipt arrives, but carry a receptive
                // snapshot for the following host's handler.
                let completed_handler = self
                    .complete_original_timing_nmi_handler_for_active_scanout(
                        classification.handler_completion,
                        input,
                        oam_dma_source.as_deref(),
                    );
                if completed_handler.completed() {
                    self.intro_initialization_reset_obj_control_pending = false;
                }
                if classification.publication_pending_at_exit {
                    // An acceptance-only host returns the reset-OBSEL scanout
                    // before retiring this CPU transient. Capture while the
                    // flag is still armed; the following handler may refine
                    // that immutable generation without moving OBSEL.
                    self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                    self.intro_initialization_reset_obj_control_pending = false;
                }
                return true;
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            self.intro_initialization_work_frames_pending -= 1;
            self.intro_initialization_reset_obj_control_pending = false;
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_save_quit_reset_plan(
        &mut self,
        input: u16,
        oam_dma_source: Option<Vec<u8>>,
        save_quit_reset_plan: Option<OriginalTimingSaveQuitResetPlan>,
    ) -> bool {
        if let Some(plan) = save_quit_reset_plan {
            match plan {
                OriginalTimingSaveQuitResetPlan::Nonterminal(plan) => {
                    // Each held host stays inside Death_Func15's save-quit
                    // tail; the exact terminal suffix below is the only
                    // authority which retires the hold.
                    self.execute_original_timing_nonterminal_continuation(
                        plan,
                        input,
                        oam_dma_source.as_deref(),
                        |_| {},
                    );
                }
                OriginalTimingSaveQuitResetPlan::ResetStatePublished(plan) => {
                    self.execute_original_timing_nonterminal_continuation(
                        plan,
                        input,
                        oam_dma_source.as_deref(),
                        |state| {
                            assert!(
                                state.take_original_timing_save_quit_reset_state_published(),
                                "validated save-quit reset state publication disappeared",
                            );
                            state.death_func15_save_quit_reset_state_before_dungeon_info_clear();
                            state.save_quit_reset_state_published = true;
                        },
                    );
                }
                OriginalTimingSaveQuitResetPlan::IntroMemoryReturned(plan) => {
                    self.execute_original_timing_nonterminal_continuation(
                        plan, input, oam_dma_source.as_deref(), |state| {
                            let receipts = state.original_timing_semantic_receipts.as_mut().unwrap();
                            let before = receipts.semantic.len();
                            receipts.semantic.retain(|receipt| *receipt != OriginalTimingSemanticReceipt::SaveQuitIntroMemoryReturned);
                            assert_eq!(before - receipts.semantic.len(), 1);
                            state.death_func31_through_intro_memory();
                        },
                    );
                }
                OriginalTimingSaveQuitResetPlan::NmiMaskedHold => {
                    let timeline = self
                        .take_original_timing_uninterrupted_main_loop_timeline(
                            crate::MainLoopProgress::CallStackContinued,
                        )
                        .expect("validated NMI-masked save-quit hold timeline disappeared");
                    assert!(
                        timeline.nmi_phases_before_progress.is_empty()
                            && timeline.nmi_phases_after_progress.is_empty(),
                        "an NMI-masked save-quit hold cannot carry NMI phases: {timeline:?}",
                    );
                    assert!(
                        self.original_timing_semantic_receipts
                            .as_ref()
                            .is_some_and(|receipts| receipts.semantic().is_empty()),
                        "an NMI-masked save-quit hold left unconsumed semantic authority",
                    );
                    if !self.save_quit_reset_writes_applied {
                        if !self.save_quit_reset_state_published {
                            self.death_func15_save_quit_reset_state_before_dungeon_info_clear();
                            self.save_quit_reset_state_published = true;
                        }
                        // The source-ordered dungeon-info clear completes in
                        // the first NMI-masked host; only the song upload then
                        // remains for the terminal return.
                        self.death_func15_save_quit_finish_dungeon_info_clear();
                        self.save_quit_reset_writes_applied = true;
                    }
                }
                OriginalTimingSaveQuitResetPlan::TerminalInterruptedSuffix => {
                    let claims = self
                        .original_timing_semantic_receipts
                        .as_ref()
                        .map(|receipts| {
                            receipts
                                .semantic
                                .iter()
                                .filter(|receipt| {
                                    **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                                })
                                .count()
                        })
                        .unwrap_or(0);
                    // The common suffix has been pending since the hold's
                    // first host (Death_Func15 was deferred inside its
                    // iteration), so the wire's SpritePreparation
                    // interruption restates that armed owner. The generic
                    // re-interruption path may already have retired it.
                    let _ = self.take_original_timing_main_loop_interruption(
                        crate::MainLoopInterruption::SpritePreparation,
                    );
                    assert_eq!(
                        self.pending_main_loop_common_suffix,
                        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                        "the save-quit terminal lost its pending common suffix",
                    );
                    self.begin_original_timing_sprite_main_return_claim_scope(claims);
                    if !self.save_quit_reset_writes_applied {
                        if !self.save_quit_reset_state_published {
                            self.death_func15_save_quit_reset_state_before_dungeon_info_clear();
                        }
                        self.death_func15_save_quit_finish_dungeon_info_clear();
                    }
                    self.save_quit_reset_state_published = false;
                    self.save_quit_reset_writes_applied = false;
                    self.death_func15_save_quit_song_upload();
                    self.sprite_main();
                    self.link_oam_main();
                    self.save_quit_reset_hold = false;
                    self.finish_original_timing_sprite_main_return_claim_scope();
                    assert_eq!(
                        self.pending_main_loop_common_suffix,
                        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                        "the save-quit terminal's caller body disturbed the pending common suffix",
                    );
                    let _ = self.take_original_timing_uninterrupted_main_loop_timeline(
                        crate::MainLoopProgress::CallStackContinued,
                    );
                    self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                }
                OriginalTimingSaveQuitResetPlan::Terminal(expected) => {
                    let timeline = self
                        .take_original_timing_main_loop_return_timeline()
                        .expect("validated terminal save-quit reset timeline disappeared");
                    assert_eq!(timeline, expected);
                    let sprite_main_claims = self
                        .original_timing_semantic_receipts
                        .as_ref()
                        .map(|receipts| {
                            receipts
                                .semantic
                                .iter()
                                .filter(|receipt| {
                                    **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                                })
                                .count()
                        })
                        .unwrap_or(0);
                    if sprite_main_claims != 0 {
                        self.begin_original_timing_sprite_main_return_claim_scope(
                            sprite_main_claims,
                        );
                    }
                    self.complete_original_timing_main_loop_return(
                        timeline,
                        input,
                        oam_dma_source.as_deref(),
                        |state| {
                            // Death_Func15's slow save-quit remainder
                            // returns to Module17 within this terminal host
                            // (the fast prefix already ran at the hold's
                            // entry); the caller then runs its
                            // Sprite_Main/LinkOam suffix before the shared
                            // game-loop suffix.
                            if !state.save_quit_reset_writes_applied {
                                if !state.save_quit_reset_state_published {
                                    state
                                        .death_func15_save_quit_reset_state_before_dungeon_info_clear();
                                }
                                state.death_func15_save_quit_finish_dungeon_info_clear();
                            }
                            state.save_quit_reset_state_published = false;
                            state.save_quit_reset_writes_applied = false;
                            state.death_func15_save_quit_song_upload();
                            state.sprite_main();
                            state.link_oam_main();
                            state.save_quit_reset_hold = false;
                        },
                    );
                    if sprite_main_claims != 0 {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                }
            }
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_intro_poly_plan(
        &mut self,
        input: u16,
        intro_poly_plan: Option<OriginalTimingIntroPolyPlan>,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if let Some(plan) = intro_poly_plan {
            match plan {
                OriginalTimingIntroPolyPlan::AwaitingIterationStart(plan) => {
                    self.execute_original_timing_iteration_started_continuation(
                        plan,
                        input,
                        oam_dma_source.as_deref(),
                        |state| {
                            // ZeldaRunGameLoop has begun, but the source is
                            // suspended inside Module00 before it can return.
                            // Phase 1 is a boolean call-stack sentinel; no
                            // fixed number of following Continued hosts is
                            // encoded in translated state.
                            state.intro_poly_thread_initialization_phase = 1;
                            state.main_loop_sprite_preparation_completed = false;
                            state.pending_main_loop_common_suffix = Some(
                                MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch,
                            );
                            state.increment_frame_counter();
                            state.clear_oam_buffer();
                        },
                    );
                }
                OriginalTimingIntroPolyPlan::Suspended(plan) => {
                    self.execute_original_timing_nonterminal_continuation(
                        plan,
                        input,
                        oam_dma_source.as_deref(),
                        |_| {},
                    );
                }
                OriginalTimingIntroPolyPlan::Terminal(expected) => {
                    let timeline = self
                        .take_original_timing_main_loop_return_timeline()
                        .expect("validated terminal intro-poly timeline disappeared");
                    assert_eq!(timeline, expected);
                    self.complete_original_timing_main_loop_return(
                        timeline,
                        input,
                        oam_dma_source.as_deref(),
                        |state| {
                            state.intro_poly_thread_initialization_phase = 0;
                            state.intro_initialize_triforce_poly_thread();
                        },
                    );
                    assert!(
                        self.original_timing_semantic_receipts
                            .as_ref()
                            .is_some_and(|receipts| receipts.semantic.is_empty()),
                        "terminal intro-poly execution left unowned semantic receipts",
                    );
                }
            }
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_terminal_spotlight_iteration_plan(
        &mut self,
        input: u16,
        oam_dma_source: Option<Vec<u8>>,
        prospective_terminal_spotlight_iteration_plan: Option<
            OriginalTimingTerminalSpotlightIterationPlan,
        >,
    ) -> bool {
        if let Some(plan) = prospective_terminal_spotlight_iteration_plan.as_ref() {
            assert_eq!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .expect("terminal spotlight preflight lost its semantic authority")
                    .semantic(),
                plan.semantic,
                "terminal spotlight semantic authority changed after immutable preflight",
            );
            assert_eq!(
                self.game_execution_scheduler, plan.scheduler_before_completion,
                "terminal spotlight scheduler changed after immutable preflight",
            );
            let consumed_timeline = self
                .take_original_timing_main_loop_return_timeline()
                .expect("terminal spotlight return timeline disappeared before execution");
            assert_eq!(
                consumed_timeline, plan.timeline,
                "terminal spotlight return timeline changed before execution",
            );
            assert_eq!(
                self.take_original_timing_dungeon_exit_spotlight_caller_returned(),
                plan.caller_return_token,
                "terminal spotlight execution disagrees with its module-qualified caller-return authority",
            );
            assert!(
                !self.take_original_timing_dungeon_exit_spotlight_entry_returned(),
                "the pre-entry owner must consume the entry-return token before terminal execution",
            );
            assert_eq!(
                self.take_original_timing_overworld_spotlight_goal_caller_returned(),
                plan.overworld_goal_return_token,
                "terminal spotlight execution disagrees with its Module10 goal-return authority",
            );
            assert_eq!(
                self.take_original_timing_spotlight_table_build_progress(),
                plan.spotlight_claim,
                "terminal spotlight execution consumed a different table re-checkpoint than its preflight",
            );

            self.complete_original_timing_main_loop_return(
                consumed_timeline,
                input,
                oam_dma_source.as_deref(),
                |state| {
                    assert_eq!(
                        state.game_execution_scheduler, plan.scheduler_before_completion,
                        "terminal spotlight scheduler changed before its source-authoritative commit",
                    );
                    state.game_execution_scheduler = plan.scheduler_after_completion;
                    assert!(
                        state.game_execution_scheduler.is_idle(),
                        "terminal spotlight completion retained scheduled work",
                    );
                    if let OriginalTimingTerminalSpotlightCpuAction::CompleteBuild {
                        mut table_build,
                        projection_completed,
                    } = plan.cpu_action
                    {
                        if let Some(claim) = plan.spotlight_claim {
                            if table_build.source_progress != Some(claim.progress) {
                                table_build = state
                                    .begin_iris_spotlight_configure_table_at_progress(
                                        claim.progress,
                                    );
                            }
                        }
                        state.complete_dungeon_exit_spotlight_build_cpu(
                            table_build,
                            projection_completed,
                        );
                        assert!(
                            state.game_execution_scheduler.is_idle(),
                            "terminal spotlight Build CPU completion scheduled successor work",
                        );
                    }
                    state.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )));
                    if matches!(
                        plan.cpu_action,
                        OriginalTimingTerminalSpotlightCpuAction::CompleteBuild { .. }
                    ) {
                        // The carried Held handler refines the already-active
                        // receptive generation without recapturing it.  The
                        // completed Build then owns its normal typed display
                        // publication before the retained generation receives
                        // the source-derived table-tail projection.
                        state.capture_display_snapshot_with_override(Some(
                            plan.iteration.phase.close_completion_publication(),
                        ));
                    }
                    if plan
                        .iteration
                        .projects_following_table_tail_on_completion()
                    {
                        state.project_following_spotlight_tail_to_active_scanout(
                            plan.iteration.phase,
                            plan.iteration.projection_uses_published_prefix(),
                        );
                    }
                },
            );
            assert_eq!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .expect("terminal spotlight execution lost its semantic ledger")
                    .semantic(),
                [],
                "terminal spotlight execution retained source authority",
            );
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            debug_host_path_early_return(self.frame_ctr_dbg, line!());
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_uninterrupted_idle_continued_return_plan(
        &mut self,
        input: u16,
        authoritative_uninterrupted_idle_continued_return_plan: Option<
            OriginalTimingUninterruptedIdleContinuedReturnPlan,
        >,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if let Some(plan) = authoritative_uninterrupted_idle_continued_return_plan {
            // `CallStackContinued` is emitted immediately before this exact C
            // boundary. The module and ZeldaRunGameLoop prefix have already
            // run in the source call stack; only the shared suffix remains.
            assert_eq!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .expect("validated idle terminal caller lost its semantic authority")
                    .semantic(),
                plan.semantic,
                "validated idle terminal authority changed before consumption",
            );
            let timeline = self
                .take_original_timing_main_loop_return_timeline()
                .expect("validated idle terminal return timeline disappeared");
            assert_eq!(timeline, plan.timeline);
            // A deferred spotlight caller can reach its main wait inside
            // this continued call (route host 53926); the token corroborates
            // the native return that already ran.
            let _ = self.take_original_timing_dungeon_exit_spotlight_caller_returned();
            // A Module0E interface sequence can close its dialogue inside
            // this continued call (route host 123210, the desert-prayer
            // sequence). Match the idle Module0E consumer: mark the semantic
            // branch and let the C translation own every mutation.
            if self.take_original_timing_dialogue_closed() && self.frame_module_hosts_dialogue() {
                self.messaging_state_mut().set_text_render_state(4);
            }
            // Outside Module0E the native continued caller already
            // executed the close (the saved module is restored); the
            // receipt corroborates that completed C branch.
            let cpu_action = plan.cpu_action;
            self.complete_original_timing_main_loop_return(
                timeline,
                input,
                oam_dma_source.as_deref(),
                |state| match cpu_action {
                    OriginalTimingIdleContinuedReturnCpuAction::None => {
                        if state.game_state.frame.main_module == 0 {
                            // Intro-module return-shaped Sprite_Main receipt
                            // (see the idle terminal plan).
                            let _ = state.take_original_timing_sprite_main_returned();
                        }
                    }
                    OriginalTimingIdleContinuedReturnCpuAction::SaveMenuInitializationCompletion => {
                        assert_eq!(
                            state.take_original_timing_save_menu_initialization_progress(),
                            Some(SaveMenuInitializationProgress::Completed),
                            "the source-proven save-menu terminal lost its completion receipt",
                        );
                        // The module's live guard normally consumes the same
                        // receipt. Mark that exact prerequisite as already
                        // consumed, then execute the C body and its suffix in
                        // this return host rather than a fabricated next
                        // iteration.
                        state.save_menu_initialization_completed_pending = true;
                        state.Module0E_0B_SaveMenu();
                    }
                    OriginalTimingIdleContinuedReturnCpuAction::DialogueEndpoint {
                        message_read_position,
                        current_glyph_started,
                        transition,
                    } => {
                        state.complete_original_timing_terminal_dialogue_endpoint(
                            message_read_position,
                            current_glyph_started,
                            transition,
                        );
                    }
                    OriginalTimingIdleContinuedReturnCpuAction::SuspendedVwfCompletion {
                        transition,
                    } => state.complete_original_timing_terminal_suspended_vwf(transition),
                    OriginalTimingIdleContinuedReturnCpuAction::DialogueScrollCompletion => {
                        // Mirror the lag-frame scroll body for the Continued
                        // receipt: the copy finishes, the RenderText and
                        // Module0E callers return, and the completion stages
                        // after the return boundary. The shared executor owns
                        // the suffix and any trailing acceptance.
                        state.complete_module0e_dialogue_scroll_before_common_suffix();
                    }
                },
            );
            assert!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .is_none_or(
                        |receipts| !receipts.semantic().iter().any(|receipt| matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::DialogueExecutionProgress(_)
                        ))
                    ),
                "an idle terminal caller retained its claimed dialogue endpoint",
            );
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_uninterrupted_idle_main_loop_timeline(
        &mut self,
        input: u16,
        authoritative_uninterrupted_idle_main_loop_timeline: Option<
            OriginalTimingUninterruptedIdleMainLoopPlan,
        >,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if let Some(plan) = authoritative_uninterrupted_idle_main_loop_timeline {
            assert_eq!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .expect("validated uninterrupted main-loop host lost its semantic authority")
                    .semantic,
                plan.semantic,
                "validated uninterrupted main-loop authority changed before consumption",
            );
            assert_eq!(
                self.game_execution_scheduler.pre_main_nmi_resume(),
                plan.pre_main_timing_shadow,
                "uninterrupted idle-main pre-main timing-shadow ownership changed after immutable preflight",
            );
            let timeline = self
                .take_original_timing_uninterrupted_main_loop_timeline(plan.timeline.progress)
                .expect("validated uninterrupted main-loop timeline disappeared");
            assert_eq!(timeline, plan.timeline);
            if plan.suffix_action == OriginalTimingIdleMainLoopSuffixAction::HoldAtMainWait {
                // The ROM waited out this interval at its main wait with the
                // NMI disabled; no native CPU, display, or suffix state moves.
                assert_eq!(
                    plan.sprite_main_returned_claims, 0,
                    "a held main-wait host cannot claim a Sprite_Main return",
                );
                assert!(
                    self.original_timing_semantic_receipts
                        .as_ref()
                        .is_none_or(|receipts| receipts.semantic().is_empty()),
                    "a held main-wait host left unconsumed semantic authority",
                );
                return true;
            }
            assert!(
                self.original_timing_nmi_publication_pending
                    || timeline.nmi_phases_before_progress.first()
                        != Some(&OriginalTimingNmiPhase::HandlerCompleted),
                "an uninterrupted source host completed an NMI whose acceptance marker was lost: host={:?} frame={:?} work={:?} pre_main={:?} timeline={timeline:?}",
                self.original_timing_last_oracle_host_call,
                self.game_state.frame,
                self.game_execution_scheduler.current_work(),
                self.game_execution_scheduler.pre_main_nmi_resume(),
            );
            let before_main = plan.before_main;
            let pending_publication_before_main = before_main.publication_pending_at_exit;
            let iteration_started = timeline.progress == crate::MainLoopProgress::IterationStarted;
            if iteration_started {
                assert!(
                    !pending_publication_before_main,
                    "the source cannot begin ZeldaRunGameLoop while an accepted NMI publication remains unfinished",
                );
            } else {
                assert!(
                    timeline.nmi_phases_after_progress.is_empty(),
                    "CallStackContinued is the terminal source-host progress fact: {timeline:?}",
                );
            }
            self.preflight_dialogue_text_dma_before_early_stage(
                before_main.handler_completion,
                iteration_started,
            );
            if let Some(resume @ PreMainNmiResume::OverworldSpriteReloadReturn { .. }) =
                plan.pre_main_timing_shadow
            {
                // The retired reload-return shadow still owns the display
                // generations its leading NMI consumes; stage them before the
                // handler publishes (route host 38519).
                let scanout = resume.scanout_generations();
                self.next_display_vram_generation = scanout.vram;
                self.next_display_animated_bg_scanout_generation = scanout.animated_bg;
                self.next_display_bg_scroll_generation = scanout.bg_scroll;
                self.set_next_display_obj_scanout(scanout.obj);
            }
            let mut handler_completion = self
                .complete_original_timing_nmi_handler_for_active_scanout(
                    before_main.handler_completion,
                    input,
                    oam_dma_source.as_deref(),
                );
            if iteration_started {
                self.retire_original_timing_fresh_iteration_pre_main_shadow(
                    plan.pre_main_timing_shadow,
                );
                if handler_completion.completed() {
                    self.game_execution_scheduler
                        .mark_main_iteration_after_leading_nmi();
                }
            }
            self.begin_original_timing_sprite_main_return_claim_scope(
                plan.sprite_main_returned_claims,
            );
            let dialogue_endpoint_transition = match plan.dialogue_claim {
                OriginalTimingIdleMainLoopDialogueClaim::ResumedRenderingWithoutMainIteration {
                    transition,
                    ..
                } => Some(transition),
                OriginalTimingIdleMainLoopDialogueClaim::None
                | OriginalTimingIdleMainLoopDialogueClaim::DialogueClosed => None,
            };
            self.zelda_run_game_loop_with_progress_and_dialogue_text_dma(
                Some(timeline.progress),
                &mut handler_completion.dialogue_text_dma,
                Some(plan.suffix_action),
                dialogue_endpoint_transition,
            );
            self.finish_original_timing_sprite_main_return_claim_scope();
            self.assert_original_timing_idle_main_loop_suffix_postcondition(plan.suffix_action);
            // A reset-sprites claim riding the trailing acceptance or the
            // host return restates a statement the native iteration already
            // executed inside this host; drain the corroboration (route
            // hosts 39663, 49355).
            let _ = self.take_original_timing_dungeon_reset_sprites_progress();
            let remaining_semantic = &self
                .original_timing_semantic_receipts
                .as_ref()
                .expect("uninterrupted main-loop execution lost its semantic authority")
                .semantic;
            match plan.dialogue_claim {
                OriginalTimingIdleMainLoopDialogueClaim::None => assert!(
                    !remaining_semantic.iter().any(|receipt| matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::DialogueExecutionProgress(_)
                            | OriginalTimingSemanticReceipt::DialogueClosed
                    )),
                    "uninterrupted main-loop execution retained an unclaimed dialogue receipt",
                ),
                OriginalTimingIdleMainLoopDialogueClaim::ResumedRenderingWithoutMainIteration {
                    ..
                } => assert!(
                    !remaining_semantic.iter().any(|receipt| matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::DialogueExecutionProgress(_)
                    )),
                    "uninterrupted main-loop execution did not consume its dialogue endpoint",
                ),
                OriginalTimingIdleMainLoopDialogueClaim::DialogueClosed => assert!(
                    !remaining_semantic.iter().any(|receipt| matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::DialogueClosed
                    )),
                    "uninterrupted main-loop execution did not consume its dialogue-close claim",
                ),
            }
            if plan.spotlight_claim.is_some() {
                assert!(
                    !remaining_semantic.iter().any(|receipt| matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(_)
                    )),
                    "uninterrupted main-loop execution did not consume its spotlight checkpoint claim",
                );
            }
            if plan.sprite_main_returned_claims != 0 {
                assert!(
                    !remaining_semantic.contains(&OriginalTimingSemanticReceipt::SpriteMainReturned),
                    "uninterrupted main-loop execution did not consume its idle-body Sprite_Main return claim",
                );
            }
            self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
            if let Some(token) = handler_completion.dialogue_text_dma.take() {
                assert!(
                    self.complete_dialogue_scroll_after_text_dma_publication(token),
                    "a leading BG3 text-DMA handler did not reach its immediate dialogue staging boundary",
                );
            }

            if !iteration_started {
                if pending_publication_before_main {
                    self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                }
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }

            let (completes_publication_after_main, accepts_nmi_at_return) =
                classify_original_timing_nmi_phases(false, &timeline.nmi_phases_after_progress);
            let captured_before_main = before_main.handler_completion
                == OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence;
            if (!captured_before_main && !accepts_nmi_at_return) || completes_publication_after_main
            {
                self.capture_display_snapshot();
            }
            if completes_publication_after_main {
                self.interrupt_nmi(input, None, false);
                self.apply_original_timing_joypad_publication();
            }
            if accepts_nmi_at_return {
                self.capture_and_carry_original_timing_nmi_publication_at_host_return();
            }
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_retire_completed_leading_nmi(
        &mut self,
        input: u16,
        authoritative_scheduled_caller_leading_nmi_completion: OriginalTimingNmiHandlerCompletionOwner,
        authoritative_scheduled_caller_nmi_timeline: Option<
            &OriginalTimingMainLoopInterruptionTimeline,
        >,
        oam_dma_source: Option<Vec<u8>>,
    ) {
        if authoritative_scheduled_caller_leading_nmi_completion.completed() {
            // Both scheduled timeline forms above proved that one accepted NMI
            // completed its Zelda-visible publication in this host. Retire
            // the carried marker before applying that publication; a later
            // acceptance, if any, is carried again at the source return.
            if let Some(timeline) = authoritative_scheduled_caller_nmi_timeline {
                assert!(
                    timeline.interruption == crate::MainLoopInterruption::LinkOam
                        || timeline.interruption == crate::MainLoopInterruption::SpritePreparation
                        || matches!(
                            timeline.interruption,
                            crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }
                                // The recurring spotlight Build's host may end
                                // inside Link_MovePosition (route host 179586).
                                | crate::MainLoopInterruption::LinkPositionAfterSubpixel { .. }
                                | crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { .. }
                                | crate::MainLoopInterruption::LinkPositionAfterCoordinates { .. }
                        )
                        || (matches!(
                            timeline.interruption,
                            crate::MainLoopInterruption::SpotlightGoalResetTable { .. }
                        ) && matches!(
                            self.game_execution_scheduler.current_work(),
                            Some(GameWorkContinuation::FinishOverworldSpotlightBuild { .. })
                        ))
                        || (matches!(
                            timeline.interruption,
                            crate::MainLoopInterruption::DesertPrayerIris { .. }
                        ) && matches!(
                            self.game_execution_scheduler.current_work(),
                            Some(
                                GameWorkContinuation::FinishDesertPrayerIris { .. }
                                    | GameWorkContinuation::FinishDesertPrayerPaletteFilter { .. }
                            )
                        ))
                        || timeline.interruption.is_sprite_main()
                );
            }
            // A same-host acceptance owns the ordinary capture boundary. A
            // carry-in handler instead refines the receptive scanout retained
            // from its acceptance host without recapturing later CPU state.
            self.complete_original_timing_nmi_handler_for_active_scanout(
                authoritative_scheduled_caller_leading_nmi_completion,
                input,
                oam_dma_source.as_deref(),
            )
            .assert_no_unclaimed_dialogue_text_dma();
        }
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_scheduled_caller_call_stack_continued_nmi(
        &mut self,
        authoritative_scheduled_caller_accepts_nmi_at_return: Option<bool>,
        authoritative_scheduled_caller_nmi_timeline: Option<
            &OriginalTimingMainLoopInterruptionTimeline,
        >,
    ) {
        if let Some(timeline) = authoritative_scheduled_caller_nmi_timeline.filter(|timeline| {
            timeline.progress == crate::MainLoopProgress::CallStackContinued
                && (timeline.interruption.is_sprite_main()
                    || (timeline.interruption == crate::MainLoopInterruption::LinkOam
                        && matches!(
                            self.game_execution_scheduler.current_work(),
                            Some(
                                GameWorkContinuation::FinishOverworldAuxGraphics
                                    | GameWorkContinuation::FinishOverworldMosaicSpriteGraphics
                                    // The straight-stair/spiral room callers'
                                    // Module07 LinkOam (route host 307614).
                                    | GameWorkContinuation::FinishDungeonSupertileTransition {
                                        work: DungeonSupertileTransitionWork::StraightInterroomRoomInitialization
                                            | DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics
                                            | DungeonSupertileTransitionWork::StraightInterroomBgCharacters34
                                            | DungeonSupertileTransitionWork::SpiralRoomInitialization
                                            | DungeonSupertileTransitionWork::SpiralBackgroundSync
                                    }
                            )
                        )))
                && matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(
                        GameWorkContinuation::FinishOverworldSpriteReloadTail { .. }
                            | GameWorkContinuation::FinishOverworldAuxGraphics
                            | GameWorkContinuation::FinishOverworldMosaicSpriteGraphics
                            | GameWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail
                            // The mirror-warp sprite reload's own Module09
                            // Sprite_Main is re-interrupted mid-loop (slot 9
                            // at route host 321925).
                            | GameWorkContinuation::FinishModule09LongLoad { .. }
                            // The spiral room/background callers' Module07
                            // Sprite_Main likewise (slot 1 at route host 95850).
                            | GameWorkContinuation::FinishDungeonSupertileTransition {
                                work: DungeonSupertileTransitionWork::SpiralRoomInitialization
                                    | DungeonSupertileTransitionWork::SpiralBackgroundSync
                                    | DungeonSupertileTransitionWork::StraightInterroomRoomInitialization
                                    | DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics
                                    | DungeonSupertileTransitionWork::StraightInterroomBgCharacters34
                                    | DungeonSupertileTransitionWork::SpriteConversion
                                    | DungeonSupertileTransitionWork::RoomLoadSpriteReset { .. }
                                    // Falling loads resume the same Module07
                                    // Sprite_Main caller and must transfer its
                                    // interrupted slot before running it.
                                    | DungeonSupertileTransitionWork::FallingSpriteGraphics
                                    | DungeonSupertileTransitionWork::FallingBgCharacters34
                            }
                    )
                )
        }) {
            // The resumed reload tail's own Sprite_Main is re-interrupted in
            // this host; forward the boundary so the native body suspends at
            // the same statement (route host 38934; the aux-graphics caller's
            // Module09 Sprite_Main at route host 67569).
            self.forward_original_timing_main_loop_interruption_to_native_owner(
                timeline.interruption,
                if authoritative_scheduled_caller_accepts_nmi_at_return.unwrap_or(false) {
                    OriginalTimingBoundary::NmiAccepted
                } else {
                    OriginalTimingBoundary::HostReturn
                },
            );
        }
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_live_sprite_main_refined_boundary(
        &mut self,
        authoritative_live_sprite_main_refined_boundary: Option<SpriteMainCpuBoundary>,
        authoritative_live_sprite_main_work: Option<(SpriteMainCpuBoundary, SpriteMainCpuCaller)>,
    ) {
        if let (Some((current_boundary, caller)), Some(refined_boundary)) = (
            authoritative_live_sprite_main_work,
            authoritative_live_sprite_main_refined_boundary,
        ) {
            if current_boundary != refined_boundary {
                let scheduled_refined_boundary = match (current_boundary, refined_boundary) {
                    (
                        SpriteMainCpuBoundary::Module09FinalScrollPairPending,
                        SpriteMainCpuBoundary::BeforeFirstSlot,
                    ) => {
                        self.complete_module09_final_scroll_pair();
                        refined_boundary
                    }
                    (
                        SpriteMainCpuBoundary::BeforeFirstSlot
                        | SpriteMainCpuBoundary::Module09FinalScrollPairPending,
                        SpriteMainCpuBoundary::AfterSlot(newly_completed_slot),
                    ) => {
                        // The saved caller's Sprite_Main started in this host
                        // and ran down to the wire's newly returned slot
                        // before suspending again (route host 187518).
                        self.advance_sprite_main_before_first_slot_to_after_slot(
                            newly_completed_slot,
                        );
                        refined_boundary
                    }
                    (
                        SpriteMainCpuBoundary::BeforeFirstSlot
                        | SpriteMainCpuBoundary::Module09FinalScrollPairPending,
                        SpriteMainCpuBoundary::AfterTimersAndOam { slot, state: None },
                    ) => self.advance_sprite_main_before_first_slot_to_after_timers_and_oam(slot),
                    (
                        SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(completed_slot),
                        SpriteMainCpuBoundary::AfterTimersAndOam { slot, state: None },
                    ) if slot < completed_slot => {
                        self.complete_waterfall_gt_cutscene_graphics(usize::from(completed_slot));
                        self.advance_sprite_main_after_slot_to_after_timers(completed_slot, slot)
                    }
                    (
                        SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(completed_slot),
                        SpriteMainCpuBoundary::AfterSlot(slot),
                    ) if slot <= completed_slot => {
                        self.complete_waterfall_gt_cutscene_graphics(usize::from(completed_slot));
                        if slot < completed_slot {
                            self.advance_sprite_main_after_slot_boundary(completed_slot, slot);
                        }
                        refined_boundary
                    }
                    (
                        SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(completed_slot)
                        | SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(completed_slot),
                        SpriteMainCpuBoundary::AfterTimersAndOam { slot, state: None },
                    ) if slot < completed_slot => {
                        self.complete_happiness_pond_rupee_graphics(usize::from(completed_slot));
                        self.advance_sprite_main_after_slot_to_after_timers(completed_slot, slot)
                    }
                    (
                        SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(completed_slot)
                        | SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(completed_slot),
                        SpriteMainCpuBoundary::AfterSlot(slot),
                    ) if slot <= completed_slot => {
                        self.complete_happiness_pond_rupee_graphics(usize::from(completed_slot));
                        if slot < completed_slot {
                            self.advance_sprite_main_after_slot_boundary(completed_slot, slot);
                        }
                        refined_boundary
                    }
                    (
                        SpriteMainCpuBoundary::FollowerGraphics {
                            slot,
                            caller,
                            prefix_completed: true,
                            saved_follower_indicator,
                            stage: prior_stage,
                        },
                        SpriteMainCpuBoundary::FollowerGraphics {
                            slot: refined_slot,
                            caller: refined_caller,
                            prefix_completed: false,
                            saved_follower_indicator: None,
                            stage: next_stage,
                        },
                    ) if slot == refined_slot && caller == refined_caller => {
                        self.apply_follower_graphics_progress(Some(prior_stage), next_stage);
                        SpriteMainCpuBoundary::FollowerGraphics {
                            slot,
                            caller,
                            prefix_completed: true,
                            saved_follower_indicator,
                            stage: next_stage,
                        }
                    }
                    (
                        SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                            slot,
                            continuation: Some(_),
                        },
                        SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                            slot: refined_slot,
                            continuation: None,
                        },
                    ) if slot == refined_slot => current_boundary,
                    (
                        SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                            slot,
                            continuation: Some(continuation),
                        },
                        SpriteMainCpuBoundary::AfterSlot(newly_completed_slot),
                    ) if newly_completed_slot <= slot => {
                        self.complete_sprite_slot_after_single_small_draw_position(
                            usize::from(slot),
                            continuation,
                        );
                        if newly_completed_slot < slot {
                            self.advance_sprite_main_after_slot_boundary(
                                slot,
                                newly_completed_slot,
                            );
                        }
                        refined_boundary
                    }
                    (
                        SpriteMainCpuBoundary::AfterActiveCuccoX {
                            slot,
                            helper_ordinal,
                        },
                        SpriteMainCpuBoundary::AfterActiveCuccoX {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        current_boundary
                    }
                    (
                        SpriteMainCpuBoundary::AfterActiveCuccoX {
                            slot,
                            helper_ordinal,
                        },
                        SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            y_low: None,
                            y_high: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        let (y_low, y_high) =
                            self.advance_active_cucco_x_to_y_subpixel(usize::from(slot));
                        SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                            slot,
                            helper_ordinal,
                            y_low: Some(y_low),
                            y_high: Some(y_high),
                        }
                    }
                    (
                        SpriteMainCpuBoundary::AfterActiveCuccoX {
                            slot,
                            helper_ordinal,
                        },
                        SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            completed,
                            total: 0,
                            continuation: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        self.advance_active_cucco_x_to_helper(usize::from(slot));
                        self.advance_cucco_helper_to_subtype_checkpoint(
                            usize::from(slot),
                            completed,
                        );
                        SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                            slot,
                            helper_ordinal,
                            completed,
                            total: 4,
                            continuation: Some(CuccoSubtypeContinuation::ActiveC),
                        }
                    }
                    (
                        SpriteMainCpuBoundary::AfterActiveCuccoX {
                            slot,
                            helper_ordinal,
                        },
                        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            continuation: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        self.advance_active_cucco_x_to_helper(usize::from(slot));
                        self.advance_cucco_helper_to_graphics(usize::from(slot), 4);
                        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                            slot,
                            helper_ordinal,
                            continuation: Some(CuccoSubtypeContinuation::ActiveC),
                        }
                    }
                    (
                        SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                            slot,
                            helper_ordinal,
                            y_low: Some(_),
                            y_high: Some(_),
                        },
                        SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            y_low: None,
                            y_high: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        current_boundary
                    }
                    (
                        SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                            slot,
                            helper_ordinal,
                            y_low: Some(y_low),
                            y_high: Some(y_high),
                        },
                        SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            completed,
                            total: 0,
                            continuation: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        self.advance_active_cucco_y_subpixel_to_helper(
                            usize::from(slot),
                            y_low,
                            y_high,
                        );
                        self.advance_cucco_helper_to_subtype_checkpoint(
                            usize::from(slot),
                            completed,
                        );
                        SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                            slot,
                            helper_ordinal,
                            completed,
                            total: 4,
                            continuation: Some(CuccoSubtypeContinuation::ActiveC),
                        }
                    }
                    (
                        SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                            slot,
                            helper_ordinal,
                            y_low: Some(y_low),
                            y_high: Some(y_high),
                        },
                        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            continuation: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        self.advance_active_cucco_y_subpixel_to_helper(
                            usize::from(slot),
                            y_low,
                            y_high,
                        );
                        self.advance_cucco_helper_to_graphics(usize::from(slot), 4);
                        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                            slot,
                            helper_ordinal,
                            continuation: Some(CuccoSubtypeContinuation::ActiveC),
                        }
                    }
                    (
                        SpriteMainCpuBoundary::AfterCuccoFleeMovement {
                            slot,
                            helper_ordinal,
                        },
                        SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            completed,
                            total: 0,
                            continuation: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        self.advance_cucco_helper_to_subtype_checkpoint(
                            usize::from(slot),
                            completed,
                        );
                        SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                            slot,
                            helper_ordinal,
                            completed,
                            total: 5,
                            continuation: Some(CuccoSubtypeContinuation::Flee),
                        }
                    }
                    (
                        SpriteMainCpuBoundary::AfterCuccoFleeMovement {
                            slot,
                            helper_ordinal,
                        },
                        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            continuation: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        self.advance_cucco_helper_to_graphics(usize::from(slot), 5);
                        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                            slot,
                            helper_ordinal,
                            continuation: Some(CuccoSubtypeContinuation::Flee),
                        }
                    }
                    (
                        SpriteMainCpuBoundary::AfterCuccoFleeMovement {
                            slot,
                            helper_ordinal,
                        },
                        SpriteMainCpuBoundary::AfterCuccoFleeMovement {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        current_boundary
                    }
                    (
                        SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                            slot,
                            helper_ordinal,
                            completed,
                            total,
                            continuation: Some(continuation),
                        },
                        SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            completed: refined_completed,
                            total: 0,
                            continuation: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        assert!(
                            refined_completed >= completed,
                            "source Cucco helper progress rewound from increment {completed} to {refined_completed}",
                        );
                        self.advance_cucco_subtype_increment_checkpoint(
                            usize::from(slot),
                            completed,
                            refined_completed,
                            total,
                        );
                        SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                            slot,
                            helper_ordinal,
                            completed: refined_completed,
                            total,
                            continuation: Some(continuation),
                        }
                    }
                    (
                        SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                            slot,
                            helper_ordinal,
                            completed,
                            total,
                            continuation: Some(continuation),
                        },
                        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            continuation: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        self.advance_cucco_subtype_checkpoint_to_graphics(
                            usize::from(slot),
                            completed,
                            total,
                        );
                        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                            slot,
                            helper_ordinal,
                            continuation: Some(continuation),
                        }
                    }
                    (
                        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                            slot,
                            helper_ordinal,
                            continuation: Some(_),
                        },
                        SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                            slot: refined_slot,
                            helper_ordinal: refined_ordinal,
                            continuation: None,
                        },
                    ) if slot == refined_slot && helper_ordinal == refined_ordinal => {
                        current_boundary
                    }
                    _ => {
                        let current_order = sprite_main_cpu_boundary_order(current_boundary);
                        let refined_order = sprite_main_cpu_boundary_order(refined_boundary);
                        if refined_order < current_order {
                            // A resumed entry-NMI can name the last complete
                            // slot even when native already owns a finer C
                            // checkpoint in the following slot. It confirms
                            // the prefix but cannot rewind the continuation.
                            debug_assert!(matches!(
                                refined_boundary,
                                SpriteMainCpuBoundary::AfterSlot(_)
                            ));
                            current_boundary
                        } else {
                            assert_ne!(
                                refined_order, current_order,
                                "conflicting Sprite_Main checkpoints at one source statement: {current_boundary:?} vs {refined_boundary:?}",
                            );
                            match (current_boundary, refined_boundary) {
                                (
                                    SpriteMainCpuBoundary::AfterSlot(completed_slot),
                                    SpriteMainCpuBoundary::AfterSlot(newly_completed_slot),
                                ) => self.advance_sprite_main_after_slot_boundary(
                                    completed_slot,
                                    newly_completed_slot,
                                ),
                                _ => panic!(
                                    "unsupported live Sprite_Main boundary refinement: {current_boundary:?} -> {refined_boundary:?}",
                                ),
                            }
                            refined_boundary
                        }
                    }
                };
                if scheduled_refined_boundary != current_boundary {
                    self.game_execution_scheduler.refine_scheduled_work(
                        GameWorkContinuation::FinishSpriteMain {
                            boundary: current_boundary,
                            caller,
                        },
                        GameWorkContinuation::FinishSpriteMain {
                            boundary: scheduled_refined_boundary,
                            caller,
                        },
                    );
                }
            }
        }
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_terminal_ground_item_receipt_plan(
        &mut self,
        input: u16,
        oam_dma_source: Option<Vec<u8>>,
        prospective_terminal_ground_item_receipt_plan: Option<
            OriginalTimingTerminalGroundItemReceiptPlan,
        >,
        scheduled_work_step: Option<GameWorkStep>,
    ) -> bool {
        if let Some(plan) = prospective_terminal_ground_item_receipt_plan.as_ref() {
            assert_eq!(
                scheduled_work_step,
                Some(GameWorkStep::Complete(plan.work)),
                "terminal ground-item scheduler did not commit its probed completion",
            );
            assert_eq!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .expect("terminal ground-item execution lost its semantic authority")
                    .semantic(),
                plan.semantic,
                "terminal ground-item semantic authority changed before execution",
            );
            let consumed_timeline = self
                .take_original_timing_main_loop_return_timeline()
                .expect("terminal ground-item return timeline disappeared before execution");
            assert_eq!(
                consumed_timeline, plan.timeline,
                "terminal ground-item return timeline changed before execution",
            );

            // The accepted Held NMI interrupts the decompressor before its C
            // caller returns. Preserve the item hold's measured retained-display
            // policy while moving this one handler ahead of the resumed CPU
            // callback; the generic item-completion arm below must not run it a
            // second time. A gfx-$21 return outside handler 21 instead uses
            // the ordinary module epilogue's plain capture.
            let ordinary_epilogue =
                self.item_receipt_graphics_return_uses_ordinary_module_epilogue(plan.continuation);
            let carried_handler = matches!(
                plan.timeline.nmi_phases_before_return.as_slice(),
                [OriginalTimingNmiPhase::HandlerCompleted]
            );
            if carried_handler {
                // The held NMI was accepted at the preceding host's return
                // boundary; this host only completes its handler against the
                // retained receptive acceptance snapshot.
                self.complete_original_timing_nmi_handler_for_active_scanout(
                    OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry,
                    input,
                    oam_dma_source.as_deref(),
                )
                .assert_no_unclaimed_dialogue_text_dma();
            } else if ordinary_epilogue {
                self.capture_display_snapshot();
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            } else {
                let graphics_dma_plan = rom_graphics_dma_plan(
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule,
                );
                self.nmi_core_animated_bg_update(graphics_dma_plan);
                self.capture_display_snapshot_with_publication(
                    DisplaySnapshotPublication::RetainPublished,
                );
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            }

            let ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                ground_apress_tail: Some(receipt),
                ..
            } = plan.continuation
            else {
                unreachable!("terminal ground-item plan changed continuation kind")
            };
            self.begin_original_timing_sprite_main_return_claim_scope(1);
            self.complete_ancilla_add_item_receipt(receipt);
            self.complete_link_receive_item(receipt.item);
            self.player_handler_00_ground_3_after_a_press(true);
            self.complete_module07_dungeon_after_submodule();
            assert!(
                !self
                    .game_execution_scheduler
                    .work_suspends_translated_call_stack(),
                "source-proven ground-item terminal callback created a second suspended caller",
            );
            self.complete_module07_dungeon_after_submodule_caller();
            self.finish_original_timing_sprite_main_return_claim_scope();
            assert!(
                self.game_execution_scheduler.is_idle(),
                "source-proven ground-item terminal callback scheduled successor work",
            );
            self.item_receipt_completion_live_link_dma_host = Some(self.frame_ctr_dbg);
            self.complete_pending_main_loop_common_suffix_after_module_return();
            if !ordinary_epilogue {
                self.complete_atomic_item_graphics_return_postlude(plan.continuation);
            }
            if matches!(
                plan.timeline.nmi_phases_after_return.as_slice(),
                [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)]
            ) {
                // The trailing acceptance belongs to the next host's handler;
                // its acceptance host owns the scanout that handler refines.
                self.capture_and_carry_original_timing_nmi_publication_at_host_return();
            }
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_completed_dungeon_after_submodule_caller_return(
        &mut self,
        input: u16,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
        scheduled_work_audio_follows_host_publication: bool,
        scheduled_work_started_after_leading_nmi: bool,
        scheduled_work_step: Option<GameWorkStep>,
    ) -> bool {
        if let Some(GameWorkStep::Complete(
            continuation @ GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
        )) = scheduled_work_step
        {
            // The interrupting NMI was the preceding host's trailing boundary.
            // Resume the saved dispatcher caller now. The C table memcpy
            // finishes during active display, so preserve the preceding table
            // above HDMA's crossing and publish the completed table below it.
            let spotlight_vertical_center = spotlight_vertical_center(
                self.game_state.player.follower_link.y(),
                self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
            );
            let projects_live_tail = spotlight_opening_projects_live_tail_before_hdma(
                self.game_state.display.spotlight_hdma.window_radius(),
                spotlight_vertical_center,
            );
            let before_projection = self
                .last_completed_interrupted_dungeon_spotlight_scanout
                .as_ref()
                .map(|spotlight| spotlight.hdma_tables.clone())
                .or_else(|| {
                    self.display_snapshot
                        .as_ref()
                        .map(|display| display.effective_spotlight_hdma_tables())
                });
            let completed_spotlight = self.interrupted_dungeon_spotlight_build_in_flight.take();
            let after_projection = completed_spotlight
                .as_ref()
                .map(|spotlight| spotlight.hdma_tables.clone());
            let completes_landing_wipe = self.dungeon_landing_goal_transition_pending;
            self.last_completed_interrupted_dungeon_spotlight_scanout = completed_spotlight.clone();
            self.next_display_spotlight_scanout = completed_spotlight;
            if completes_landing_wipe {
                // The C builder projects the final circle, then its reset loop
                // races HDMA across the same physical field. Publish the
                // pre-clear window controls now; the resumed caller replaces
                // this snapshot's table tail with the ROM-derived reset
                // generation before the frontend consumes it.
                self.capture_display_snapshot_with_publication(
                    DisplaySnapshotPublication::PublishCaptured,
                );
                self.dungeon_landing_goal_display_handoff =
                    DungeonLandingGoalDisplayHandoff::RetainCallerReturn;
            }
            if let Some((_, _, sprite_main_return_claims)) =
                authoritative_scheduled_caller_return_timeline.as_ref()
            {
                self.begin_original_timing_sprite_main_return_claim_scope(
                    *sprite_main_return_claims,
                );
            }
            self.complete_post_trailing_nmi_continuation(
                continuation,
                input,
                scheduled_work_started_after_leading_nmi,
                authoritative_scheduled_caller_return_timeline.is_some(),
            );
            assert_eq!(
                self.original_timing_sprite_main_return_claims_remaining, None,
                "scheduled caller return retained its Sprite_Main claim scope past the CPU callback",
            );
            if scheduled_work_audio_follows_host_publication {
                self.game_execution_scheduler
                    .mark_audio_nmi_after_host_publication();
            }
            if projects_live_tail && !scheduled_work_started_after_leading_nmi {
                let live_tail_start = spotlight_mixed_scanout_live_tail_start(
                    spotlight_vertical_center,
                    self.game_state.display.spotlight_hdma.window_radius(),
                );
                if let (Some(before_projection), Some(after_projection), Some(display)) = (
                    before_projection,
                    after_projection,
                    self.display_snapshot.as_mut(),
                ) {
                    display.hdma_table_generation =
                        DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
                            before_projection,
                            after_projection,
                            live_tail_start,
                        };
                }
            }
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_completed_spiral_staircase_palette_filter(
        &mut self,
        input: u16,
        authoritative_scheduled_caller_accepts_nmi_at_return: Option<bool>,
        authoritative_scheduled_caller_nmi_timeline: Option<
            &OriginalTimingMainLoopInterruptionTimeline,
        >,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
        oam_dma_source: Option<Vec<u8>>,
        scheduled_work_step: Option<GameWorkStep>,
    ) -> bool {
        if let Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishSpiralStaircasePaletteFilter { tail, caller },
        )) = scheduled_work_step
        {
            let palette_entry_frame = self.game_state.frame;
            let dungeon_room = self.game_state.world.location.dungeon_room_index();
            let staircase_index = self.game_state.dungeon.stair_movement.staircase_index();
            let retain_straight_interroom_oam =
                if straight_interroom_palette_filter_retains_presented_oam(
                    palette_entry_frame,
                    dungeon_room,
                    staircase_index,
                ) {
                    self.last_presented_oam.clone()
                } else if straight_interroom_palette_filter_retains_captured_oam(
                    palette_entry_frame,
                    dungeon_room,
                    staircase_index,
                ) {
                    Some(self.ppu.oam.clone())
                } else {
                    None
                };
            // The measured NMI crossing belongs inside the translated call:
            // Snes9x reaches vblank in the quadrant-build/prepare tail while
            // $0710 is still latched. Publish that interrupt first. The
            // handler therefore skips NMI_DoUpdates, and only then does the
            // saved CPU stack resume, return through Module 7, prepare the
            // next OAM shadow, and clear the software latch. Executing this in
            // the opposite order falsely admitted the future CGRAM/OAM DMA and
            // required a room/staircase publication exception.
            if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
                self.capture_display_snapshot();
                self.interrupt_nmi_for_active_scanout_without_dialogue_owner(
                    input,
                    oam_dma_source.as_deref(),
                    false,
                );
            }
            // Without a terminal return the wire can still publish the module
            // tail's Sprite_Main return inside this host and then interrupt
            // NMI_PrepareSprites (route host 402730, fat stairs: [Completed,
            // SpriteMainReturned, CallStackContinued,
            // MainLoopInterrupted(SpritePreparation)]).
            let nonterminal_sprite_main_returned = authoritative_scheduled_caller_return_timeline
                .is_none()
                && self.original_timing_owes_sprite_main_return();
            let wire_interrupts_sprite_preparation = authoritative_scheduled_caller_return_timeline
                .is_none()
                && authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                    timeline.progress == crate::MainLoopProgress::CallStackContinued
                        && timeline.interruption == crate::MainLoopInterruption::SpritePreparation
                });
            if let Some((_, _, sprite_main_return_claims)) =
                authoritative_scheduled_caller_return_timeline.as_ref()
            {
                self.begin_original_timing_sprite_main_return_claim_scope(
                    *sprite_main_return_claims,
                );
            } else if nonterminal_sprite_main_returned {
                self.begin_original_timing_sprite_main_return_claim_scope(1);
            }
            self.complete_dungeon_spiral_staircase_palette_filter(tail);
            self.complete_module07_dungeon_after_submodule();
            if authoritative_scheduled_caller_return_timeline.is_some()
                || nonterminal_sprite_main_returned
            {
                self.finish_original_timing_sprite_main_return_claim_scope();
            }
            if let Some(oam) = retain_straight_interroom_oam {
                self.next_display_obj_memory_generation =
                    Some(DisplayObjGeneration::RetainCapturedOam { oam });
            }
            if caller.requeues_core_dma_after_nmi() {
                self.set_core_update_disable_flag(1);
            }
            if wire_interrupts_sprite_preparation {
                // The suffix's sprite preparation resumes on the next host;
                // the software latch stays held across this boundary. The
                // quadrant uploads still ride the NMI that follows the return.
                self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                    NmiPrepareSpritesCpuCaller::DungeonModule07,
                );
                if matches!(self.game_state.frame.subsubmodule, 12..=15) {
                    self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                        PreMainNmiResume::DungeonSupertileQuadrantUploads,
                    );
                }
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }
            if authoritative_scheduled_caller_return_timeline.is_none()
                && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            {
                self.retire_or_defer_main_loop_common_suffix_by_wire();
            } else if self.pending_main_loop_common_suffix.is_some() {
                // The source-proven caller return already carries the shared
                // ZeldaRunGameLoop suffix; retire its one owner.
                self.complete_pending_main_loop_common_suffix_after_module_return();
            } else {
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
            }
            if matches!(self.game_state.frame.subsubmodule, 12..=15) {
                self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                    PreMainNmiResume::DungeonSupertileQuadrantUploads,
                );
            }
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_main_loop_interruption_timeline(
        &mut self,
        input: u16,
        authoritative_dungeon_exit_spotlight_caller_is_active: bool,
        authoritative_main_loop_interruption_timeline: Option<
            OriginalTimingMainLoopInterruptionTimeline,
        >,
        oam_dma_source: Option<Vec<u8>>,
        prospective_interrupted_idle_main_loop_plan: Option<
            OriginalTimingInterruptedIdleMainLoopPlan,
        >,
    ) -> bool {
        if let Some(timeline) = authoritative_main_loop_interruption_timeline {
            assert!(
                !authoritative_dungeon_exit_spotlight_caller_is_active,
                "an active spotlight caller timeline must be consumed by scheduled work",
            );
            let iteration_started = timeline.progress == crate::MainLoopProgress::IterationStarted;
            if !iteration_started {
                assert_eq!(
                    timeline.progress,
                    crate::MainLoopProgress::CallStackContinued,
                );
                assert!(
                    matches!(
                        self.pending_main_loop_common_suffix,
                        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
                    ),
                    "a continued source sprite-preparation boundary lost its existing common-suffix owner: host={:?} timeline={timeline:?}",
                    self.original_timing_last_oracle_host_call,
                );
            }
            let before_main = classify_original_timing_nmi_phases_with_ownership(
                self.original_timing_nmi_publication_pending,
                timeline.nmi_phases_before_interruption.as_slice(),
            );
            let accepts_nmi_at_return = before_main.publication_pending_at_exit;
            let pre_main_timing_shadow = iteration_started
                .then(|| {
                    prospective_interrupted_idle_main_loop_plan
                        .as_ref()
                        .expect("a fresh interrupted source call lost its immutable host plan")
                        .pre_main_timing_shadow
                })
                .flatten();
            assert!(timeline.nmi_phases_after_interruption.is_empty());
            self.preflight_dialogue_text_dma_before_early_stage(
                before_main.handler_completion,
                iteration_started,
            );
            let mut handler_completion = self
                .complete_original_timing_nmi_handler_for_active_scanout(
                    before_main.handler_completion,
                    input,
                    oam_dma_source.as_deref(),
                );
            if iteration_started {
                self.retire_original_timing_fresh_iteration_pre_main_shadow(pre_main_timing_shadow);
                assert!(
                    self.game_execution_scheduler.is_idle(),
                    "a fresh interrupted source call retained scheduled native work after its timing shadow",
                );
                if handler_completion.completed() {
                    self.game_execution_scheduler
                        .mark_main_iteration_after_leading_nmi();
                }
            }
            if !iteration_started {
                let crate::MainLoopInterruption::SpritePreparationExtendedOamPacking {
                    next_group_start,
                } = timeline.interruption
                else {
                    panic!(
                        "a continued idle source interruption requires its dedicated native owner: {timeline:?}"
                    );
                };
                assert!(
                    accepts_nmi_at_return,
                    "continued extended-OAM progress must end at its accepted NMI boundary",
                );
                if let Some(dialogue_progress) = self.original_timing_dialogue_execution_progress()
                {
                    let message_read_position = dialogue_progress.message_read_position();
                    let current_glyph_started = dialogue_progress.current_glyph_started();
                    // The source rendered VWF glyphs to this decoder endpoint
                    // before its common suffix was interrupted mid OAM
                    // packing. Apply the endpoint through the suspended-VWF
                    // transition prover; the caller epilogue stays owned by
                    // the interrupted suffix below.
                    let transition = self.original_timing_suspended_vwf_endpoint_transition_plan(
                        message_read_position,
                        current_glyph_started,
                    );
                    self.complete_original_timing_suspended_vwf_endpoint(
                        message_read_position,
                        current_glyph_started,
                        transition,
                    );
                    // Endpoint receipts exist only for hosts without a fresh
                    // main iteration; once the native decoder has caught up,
                    // following fresh iterations render at their own budget
                    // again, so the catch-up hold must not outlive this host.
                    self.dialogue_fast_forward_hold_active = false;
                }
                if self.dialogue_scroll_is_copying_remaining_pixels() {
                    // The source has returned from the slow copy before
                    // entering NMI_PrepareSprites. Finish that caller first;
                    // only the extended-OAM suffix remains suspended below.
                    self.complete_module0e_dialogue_scroll_before_common_suffix();
                }
                self.nmi_prepare_sprites_through_extended_oam_packing(next_group_start);
                assert_eq!(
                    self.pending_main_loop_common_suffix.replace(
                        MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
                            next_group_start,
                        },
                    ),
                    Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                    "continued extended-OAM progress lost its existing common suffix",
                );
                self.capture_and_carry_original_timing_nmi_publication_at_host_return();
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }
            // The timeline classifier above consumed the ordered phase while
            // closing the leading/trailing NMI lifecycle. Forward the exact
            // interruption either to the fresh module iteration or to the
            // already-owned common suffix of a continued iteration.
            let interrupted_plan = prospective_interrupted_idle_main_loop_plan
                .as_ref()
                .expect("a fresh interrupted source call lost its immutable host plan");
            assert_eq!(interrupted_plan.timeline, timeline);
            self.begin_original_timing_sprite_main_return_claim_scope(
                interrupted_plan.sprite_main_returned_claims,
            );
            self.forward_original_timing_main_loop_interruption_to_native_owner(
                timeline.interruption,
                if accepts_nmi_at_return {
                    OriginalTimingBoundary::NmiAccepted
                } else {
                    OriginalTimingBoundary::HostReturn
                },
            );
            self.zelda_run_game_loop_with_progress_and_dialogue_text_dma(
                Some(timeline.progress),
                &mut handler_completion.dialogue_text_dma,
                None,
                None,
            );
            self.finish_original_timing_sprite_main_return_claim_scope();
            assert!(
                !self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .expect("interrupted main-loop execution lost its semantic authority")
                    .semantic()
                    .contains(&OriginalTimingSemanticReceipt::SpriteMainReturned),
                "interrupted main-loop execution did not consume its Sprite_Main return claims",
            );
            // A synchronous Sprite_Main-direct item receipt suspends the slot
            // loop inside its slot; the wire's checkpoint then names the last
            // completed slot (BeforeFirstSlot for slot 15). The scheduled
            // receipt continuation owns that boundary (route host 753968).
            if let Some(GameWorkContinuation::FinishItemReceiptGraphics {
                continuation: ItemReceiptGraphicsContinuation::ResumeAncillaItemReceipt { .. },
            }) = self.game_execution_scheduler.current_work()
            {
                // The ancilla receipt suspended inside Sprite_Main's prefix:
                // the loop-entry restatements belong to that suspension
                // (route hosts 1142850, 514800).
                let owns = |interruption: crate::MainLoopInterruption| {
                    interruption == crate::MainLoopInterruption::SpriteMainBeforeFirstSlot
                };
                if self
                    .original_timing_main_loop_interruption()
                    .is_some_and(owns)
                {
                    let _ = self.take_original_timing_main_loop_interruption_any();
                }
                let forwarded = self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .and_then(OriginalTimingHostReceipts::forwarded_main_loop_interruption)
                    .map(|forwarded| forwarded.interruption());
                if let Some(interruption) = forwarded.filter(|&interruption| owns(interruption)) {
                    let _ =
                        self.take_forwarded_original_timing_main_loop_interruption(interruption);
                }
                let _ = self.take_original_timing_sprite_main_progress();
                if self.pending_main_loop_common_suffix.is_none() {
                    self.pending_main_loop_common_suffix =
                        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
                }
            }
            if let Some(GameWorkContinuation::FinishItemReceiptGraphics {
                continuation:
                    ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt { sprite_slot, .. },
            }) = self.game_execution_scheduler.current_work()
            {
                let owns = |interruption: crate::MainLoopInterruption| {
                    sprite_main_cpu_boundary_from_interruption(interruption).is_some_and(
                        |boundary| {
                            direct_item_receipt_slot_pairs_with_boundary(sprite_slot, boundary)
                        },
                    )
                };
                if self
                    .original_timing_main_loop_interruption()
                    .is_some_and(owns)
                {
                    let _ = self.take_original_timing_main_loop_interruption_any();
                }
                let forwarded = self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .and_then(OriginalTimingHostReceipts::forwarded_main_loop_interruption)
                    .map(|forwarded| forwarded.interruption());
                if let Some(interruption) = forwarded.filter(|&interruption| owns(interruption)) {
                    let _ =
                        self.take_forwarded_original_timing_main_loop_interruption(interruption);
                }
                // The fresh iteration is suspended inside the receipt: its
                // shared ZeldaRunGameLoop suffix is still owed and completes
                // with the continued return that follows (route host 753972).
                if self.pending_main_loop_common_suffix.is_none() {
                    self.pending_main_loop_common_suffix =
                        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
                }
            }
            if let Some(
                work @ GameWorkContinuation::FinishGameOverIrisGoalPaletteFill { completed_stores },
            ) = self.game_execution_scheduler.current_work()
            {
                assert_eq!(
                    timeline.interruption,
                    crate::MainLoopInterruption::GameOverIrisGoalPaletteFill { completed_stores },
                    "the scheduled game-over palette continuation changed its source cursor",
                );
                assert_eq!(
                    self.take_forwarded_original_timing_main_loop_interruption(
                        timeline.interruption,
                    ),
                    Some(if accepts_nmi_at_return {
                        OriginalTimingBoundary::NmiAccepted
                    } else {
                        OriginalTimingBoundary::HostReturn
                    }),
                    "the scheduled game-over palette continuation lost its forwarded source boundary",
                );
                assert_eq!(
                    self.game_execution_scheduler.current_work(),
                    Some(work),
                    "consuming the game-over palette boundary changed scheduled work",
                );
                assert!(
                    self.pending_main_loop_common_suffix
                        .replace(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
                        .is_none(),
                    "the game-over palette interruption overlapped an older main-loop suffix",
                );
            }
            assert!(
                !self.original_timing_main_loop_interruption_is_pending(),
                "source continuation has no semantic owner for {timeline:?}; scheduler={:?}",
                self.game_execution_scheduler,
            );
            self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
            if let Some(token) = handler_completion.dialogue_text_dma.take() {
                assert!(
                    self.complete_dialogue_scroll_after_text_dma_publication(token),
                    "an interrupted BG3 text-DMA handler did not reach its immediate dialogue staging boundary",
                );
            }
            if !accepts_nmi_at_return
                && before_main.handler_completion
                    != OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence
            {
                // No handler ran during this source interval. The SCAN_KEYS
                // return presents the generation authored by the main slice
                // itself. A carry-in handler also retained the already-closed
                // acceptance-host generation, so a fresh iteration must now
                // capture its own output.
                self.capture_display_snapshot();
            }
            if accepts_nmi_at_return {
                // The source host ends at the newly accepted handler entry.
                // Its body belongs to the following host interval.
                assert!(
                    self.game_state.display.nmi_update_is_latched(),
                    "LinkOam must still own the software latch at the deferred NMI entry",
                );
                self.capture_and_carry_original_timing_nmi_publication_at_host_return();
            }
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the lane completed the host frame.
    pub(super) fn lane_rom_startup_run_main(
        &mut self,
        input: u16,
        run_what: u8,
        frame: FrameState,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if self.rom_startup_timing() && run_what & crate::RUN_MAIN != 0 {
            if self.uncle_passage_item_receipt_starts_this_main_slice() {
                // ROM trace: Uncle_InPassage enters Link_ReceiveItem only
                // after the pending NMI has consumed the dialogue-clear
                // publication. The item decompressor then spans four further
                // vblanks with the main-loop latch set. Run this boundary
                // before the atomic port mutates sprite/OAM/palette state so
                // every display domain belongs to the same hardware
                // generation.
                self.clear_nmi_update_latch();
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                self.capture_display_snapshot();
                self.replay_trace_ram_watch("before-game-loop");
                self.zelda_run_game_loop_after_leading_nmi();
                self.replay_trace_ram_watch("after-game-loop");
                debug_assert!(matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishItemReceiptGraphics { .. })
                ));
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }
            if self.dialogue_long_scroll_starts_this_frame() {
                // Snes9x enters this host slice with NMI pending, consumes the
                // prior RenderText publication, then starts the slow scroll
                // copy. The copy crosses the following vblank before
                // Main_PrepSpritesForNmi can run. Preserve that real ordering
                // instead of coalescing both boundaries after Module0E.
                self.finish_dialogue_character_render_call();
                // Ordinary host frames consumed the previous handler's NMI
                // publication at their trailing boundary. Re-open the exact
                // pre-main boundary here before consuming that carry so this
                // scroll starts from the ROM's $12 == 0 state.
                self.clear_nmi_update_latch();
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                // This host scanout includes the upload consumed by the
                // boundary above, while the CPU state returned below includes
                // the next upload authored by submodule 2. Capture between
                // those two generations.
                self.capture_display_snapshot();
                self.replay_trace_ram_watch("before-game-loop");
                self.zelda_run_game_loop_after_leading_nmi();
                self.replay_trace_ram_watch("after-game-loop");
                debug_assert!(self.dialogue_scroll_is_copying_remaining_pixels());
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }
            if self.resume_after_pre_main_nmi(input, oam_dma_source.as_deref()) {
                return true;
            }
            if self.game_execution_scheduler.is_idle()
                && game_over_upload_pipeline_runs_after_leading_nmi(
                    frame,
                    self.game_state.messaging.runtime.menu_animation_timer(),
                    self.game_state.display.palette_filter.countdown(),
                    self.game_state.display.pending_nmi_subroutine,
                )
            {
                // The preceding ordinary frame left the software NMI latch set.
                // The ROM's main-loop return clears it before this boundary;
                // expose that boundary explicitly, publish its completed DMA,
                // then run one CPU state without synthesizing a trailing NMI.
                self.clear_nmi_update_latch();
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                self.capture_display_snapshot();
                self.zelda_run_game_loop_after_leading_nmi();
                if self.game_execution_scheduler.is_idle()
                    && self.game_state.frame.main_module == 7
                    && self.game_state.frame.submodule == 2
                    && matches!(self.game_state.frame.subsubmodule, 9..=15)
                {
                    // This main iteration returned to the ROM's NMI wait loop
                    // after consuming a leading vblank. Carry that CPU phase
                    // into the next host boundary instead of resetting to the
                    // ordinary main-then-trailing-NMI cadence.
                    self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                        PreMainNmiResume::DungeonSupertileQuadrantUploads,
                    );
                }
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }
            if self.game_execution_scheduler.is_idle()
                && straight_interroom_upload_pipeline_runs_after_leading_nmi(
                    frame,
                    self.game_execution_scheduler
                        .leading_nmi_upload_pipeline_is_active(),
                )
            {
                // The frame begins scanning the pre-NMI generation. Consume
                // the previous state's queued upload, then run exactly one CPU
                // state without synthesizing a second trailing NMI.
                self.capture_display_snapshot();
                self.interrupt_nmi_for_active_scanout_without_dialogue_owner(
                    input,
                    oam_dma_source.as_deref(),
                    false,
                );
                self.zelda_run_game_loop_after_leading_nmi();
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }
            // A live host whose fresh iteration ended with its shared suffix
            // still outstanding (route host 256363: Sprite_Main returned, no
            // suffix completion, no trailing acceptance) has not reached the
            // main wait; the following continued return owns that suffix, so
            // no leading-NMI iteration may be manufactured here.
            let live_suffix_pending =
                matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                    && self.pending_main_loop_common_suffix.is_some();
            if self.game_execution_scheduler.is_idle()
                && !live_suffix_pending
                && (self
                    .game_execution_scheduler
                    .main_return_requires_leading_nmi()
                    || rom_dungeon_module_iteration_runs_after_leading_nmi(
                        frame,
                        self.game_state.world.location.dungeon_room_index(),
                    )
                    || post_landing_player_control_runs_after_leading_nmi(
                        self.dungeon_post_landing_leading_nmi_room,
                        frame,
                        self.game_state.world.location.dungeon_room_index(),
                    ))
            {
                // The active image is emitted from the leading NMI. Capture
                // the immutable pre-handler state, then apply the handler's
                // effective DMA receipts to that scanout before the CPU
                // authors the following dungeon-module iteration. Most paths
                // return here; a long landing spotlight call can cross the
                // explicitly handled held-NMI boundary below.
                self.capture_display_snapshot();
                self.interrupt_nmi_for_active_scanout_without_dialogue_owner(
                    input,
                    oam_dma_source.as_deref(),
                    false,
                );
                self.zelda_run_game_loop_after_leading_nmi();
                if self.game_execution_scheduler.current_work()
                    == Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn)
                {
                    // Spotlight_open began after the leading NMI but its ROM
                    // instruction stream reaches the following vblank before
                    // returning. Consume that measured second boundary in this
                    // same host interval.
                    // The held latch suppresses OAM/VRAM
                    // DMA while WritePpuRegisters still publishes the new iris
                    // controls and INIDISP value.
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.stage_suspended_dungeon_submodule_after_nmi();
                    if let Some(continuation) =
                        self.game_execution_scheduler.take_post_trailing_nmi()
                    {
                        debug_assert_eq!(
                            continuation,
                            GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
                        );
                        if self.original_timing_live_suffix_outstanding() {
                            // The wire keeps the shared suffix outstanding
                            // past this host: the spotlight caller has not
                            // resumed yet, so the measured second-boundary
                            // completion stands down and the suspension
                            // continues (route host 111180; the resume holds
                            // through 111181 and returns at 111182).
                            self.game_execution_scheduler
                                .schedule_post_trailing_nmi(continuation);
                        } else {
                            self.complete_dungeon_after_submodule_caller_return();
                            if !self
                                .game_execution_scheduler
                                .work_suspends_translated_call_stack()
                            {
                                self.game_execution_scheduler
                                    .finish_call_stack_at_main_wait_before_nmi();
                            }
                        }
                    } else {
                        // A larger circle build can cross more than one NMI.
                        // Its first trailing boundary has now elapsed, while
                        // the scheduled call stack remains suspended for the
                        // remaining measured crossings.
                        debug_assert_eq!(
                            self.game_execution_scheduler.current_work(),
                            Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn),
                        );
                        debug_assert!(self
                            .game_execution_scheduler
                            .scheduled_work_slices_remaining()
                            .is_some_and(|remaining| remaining != 0));
                    }
                }
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return true;
            }
            self.nmi_read_joypads(input);
            self.joypad_sampled_before_main = true;
        }
        false
    }

    /// Lane of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_trailing_nmi_without_deferral(
        &mut self,
        input: u16,
        defer_interface_exit_bg_upload: bool,
        defer_room_82_sprite_conversion_nmi: bool,
        frame: FrameState,
        oam_dma_source: Option<Vec<u8>>,
        resumed_main_is_waiting_for_nmi: bool,
    ) {
        if !defer_room_82_sprite_conversion_nmi && !resumed_main_is_waiting_for_nmi {
            let trailing_oam_dma_source = self
                .game_execution_scheduler
                .current_work()
                .filter(|continuation| continuation.trailing_nmi_uses_live_oam_shadow())
                .map(|_| self.sprite_oam_shadow_buffer().to_vec());
            let trailing_oam_dma_source = trailing_oam_dma_source
                .as_deref()
                .or(oam_dma_source.as_deref());
            let room_82_deferred_nmi_retains_resident_oam = self.rom_startup_timing()
                && rom_room_82_deferred_nmi_retains_resident_oam(
                    frame,
                    self.game_state.world.location.dungeon_room_index(),
                    self.game_state.display.nmi_update_is_latched(),
                    self.screen_transition(),
                );
            let resident_oam = room_82_deferred_nmi_retains_resident_oam.then(|| {
                self.display_snapshot
                    .as_ref()
                    .and_then(|snapshot| snapshot.published_shadow_oam_dma.clone())
                    .or_else(|| self.last_presented_oam.clone())
                    .unwrap_or_else(|| self.ppu.oam.clone())
            });
            self.interrupt_nmi(
                input,
                trailing_oam_dma_source,
                defer_interface_exit_bg_upload,
            );
            self.stage_suspended_dungeon_submodule_after_nmi();
            self.game_execution_scheduler
                .finish_trailing_nmi_after_main_return();
            if let Some(continuation) = self.game_execution_scheduler.take_post_trailing_nmi() {
                if continuation == GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn
                    && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                    && self.pending_main_loop_common_suffix.is_some()
                {
                    // The wire kept this host's shared ZeldaRunGameLoop
                    // suffix outstanding: the interrupted caller does not
                    // resume within this host. Stay suspended across the
                    // trailing NMI instead (route host 111180; the resume
                    // holds through 111181 and returns at 111182).
                    self.game_execution_scheduler
                        .schedule_post_trailing_nmi(continuation);
                } else {
                    self.complete_post_trailing_nmi_continuation(continuation, input, false, false);
                }
            }
            if let Some(resident_oam) = resident_oam {
                // This deferred vblank still publishes Link/BG/CGRAM work, but
                // its completed OBJ list remains the resident state-$02 list.
                // Restore only that independently measured PPU domain after
                // the shared NMI path has completed its other transfers.
                self.ppu.oam = resident_oam;
            }
            self.restore_room_61_sprite_conversion_resident_oam();
        }
    }
}
