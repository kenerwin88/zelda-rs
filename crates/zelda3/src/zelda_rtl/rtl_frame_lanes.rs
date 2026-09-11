//! Lanes of the original-timing host-frame dispatcher
//! (`run_frame_internal_after_original_timing_body`), one method per
//! early-returning block. Each lane receives the dispatcher locals it reads
//! and reports whether it completed the host frame.

#![allow(clippy::too_many_arguments)]

use super::*;
use crate::game_state::FrameState;
use crate::LinkOamStairProgress;
use crate::MainLoopProgress;
use crate::RescuedMaidenInitializationStage;

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

    /// Resume a native scroll's copy/return work after its held NMI, preserving
    /// the subsequent main-wait boundary. Source receipts own their own lane.
    pub(super) fn lane_native_dialogue_scroll_continuation(
        &mut self,
        input: u16,
        oam_dma_source: Option<&[u8]>,
    ) -> bool {
        if !self.rom_startup_timing()
            || matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || self.game_state.frame.main_module != 0x0e
            || !self.dialogue_scroll_is_copying_remaining_pixels()
        {
            return false;
        }
        let Some(remaining) = self.dialogue_scroll_remaining_master_cycles.take() else {
            return false;
        };
        // Resume the held NMI before pricing the CPU interval it leaves.
        // Snes9x run 8889 returns from $0E:D0C2 at V=196, clears $12
        // at $00:805D at V=209, then waits for the Open NMI at V=225.
        // The outgoing scanout still owns the frozen text generation.
        self.interrupt_nmi(input, oam_dma_source, false);
        self.capture_display_snapshot();
        if self.native_dialogue_scroll_can_return_after_nmi(remaining) {
            self.complete_module0e_dialogue_scroll_before_common_suffix();
            self.nmi_prepare_sprites_for_main_loop();
            self.clear_nmi_update_latch();
            // The following host must publish the staged text at its leading
            // NMI before Module0E can enter another scroll call.
            self.game_execution_scheduler
                .finish_call_stack_at_main_wait_before_nmi();
        } else {
            // The later V=32 entry (run 2506) crosses a second held NMI;
            // keep the existing copy/return-only split for that call.
            self.zelda_run_game_loop_body_with_dialogue_text_dma(None, &mut None, None, None);
        }
        true
    }

    /// Run a fresh native iteration after any leading NMI it owns.
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
            // A message-line scroll copy still in flight keeps the ROM inside
            // the previous main iteration: without a timing authority the
            // lag frame runs through the ordinary game-loop path below, which
            // copies the remaining passes around this frame's NMI. Neither
            // leading-NMI lane below may claim an iteration return that has
            // not happened.
            let main_iteration_returned = self.dialogue_scroll_cpu_is_idle();
            if self.game_execution_scheduler.is_idle()
                && main_iteration_returned
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
                && main_iteration_returned
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

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_desert_prayer_iris(
        &mut self,
        input: u16,
        caller: DesertPrayerIrisCaller,
        authoritative_scheduled_caller_accepts_nmi_at_return: Option<bool>,
        authoritative_scheduled_caller_nmi_timeline: Option<
            &OriginalTimingMainLoopInterruptionTimeline,
        >,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        }
        let next_interruption = self.original_timing_main_loop_interruption().or_else(|| {
            authoritative_scheduled_caller_nmi_timeline.map(|timeline| timeline.interruption)
        });
        if let Some(
            interruption @ crate::MainLoopInterruption::DesertPrayerIris {
                source_subsubmodule,
                palette_countdown,
                radius,
                progress,
            },
        ) = next_interruption
        {
            if self.original_timing_main_loop_interruption() == Some(interruption) {
                let _ = self.take_original_timing_main_loop_interruption(interruption);
            }
            assert_eq!(self.game_state.frame.subsubmodule, source_subsubmodule);
            assert_eq!(
                self.game_state.display.palette_filter.countdown_word(),
                u16::from(palette_countdown),
            );
            assert_eq!(
                u16::from(self.game_state.display.spotlight_hdma.window_radius_byte(),),
                radius,
            );
            self.begin_desert_prayer_iris(progress, caller);
            return true;
        }
        if self.desert_prayer_iris_completion_closes_dialogue() {
            assert_eq!(caller, DesertPrayerIrisCaller::RecurringCase4);
            assert!(
                self.take_original_timing_dialogue_closed(),
                "the terminal Desert Prayer iris return lost its dialogue-close receipt",
            );
        }
        self.complete_desert_prayer_iris(caller);
        if matches!(
            next_interruption,
            Some(
                crate::MainLoopInterruption::SpritePreparation
                    | crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }
            )
        ) {
            if let Some(interruption) = self.original_timing_main_loop_interruption() {
                let _ = self.take_original_timing_main_loop_interruption(interruption);
            }
            self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                NmiPrepareSpritesCpuCaller::DesertPrayer,
            );
            return true;
        }
        if self.pending_main_loop_common_suffix.is_some() {
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_desert_prayer_palette_filter(
        &mut self,
        input: u16,
        countdown: u8,
        next_color: u8,
        authoritative_scheduled_caller_accepts_nmi_at_return: Option<bool>,
        authoritative_scheduled_caller_nmi_timeline: Option<
            &OriginalTimingMainLoopInterruptionTimeline,
        >,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        }
        let interruption = crate::MainLoopInterruption::DesertPrayerPaletteFilterBeforeColor {
            countdown,
            next_color,
        };
        if self.original_timing_main_loop_interruption() == Some(interruption) {
            let _ = self.take_original_timing_main_loop_interruption_any();
        }
        self.complete_desert_prayer_palette_filter_before_iris(countdown, next_color);
        let suffix_interruption = self.original_timing_main_loop_interruption().or_else(|| {
            authoritative_scheduled_caller_nmi_timeline.map(|timeline| timeline.interruption)
        });
        if let Some(
            interruption @ crate::MainLoopInterruption::DesertPrayerIris {
                source_subsubmodule,
                palette_countdown,
                radius,
                progress,
            },
        ) = suffix_interruption
        {
            if let Some(bus_interruption) = self.original_timing_main_loop_interruption() {
                assert_eq!(bus_interruption, interruption);
                let _ = self.take_original_timing_main_loop_interruption(interruption);
            }
            assert_eq!(self.game_state.frame.subsubmodule, source_subsubmodule);
            assert_eq!(
                self.game_state.display.palette_filter.countdown_word(),
                u16::from(palette_countdown),
            );
            assert_eq!(
                u16::from(self.game_state.display.spotlight_hdma.window_radius_byte(),),
                radius,
            );
            self.begin_desert_prayer_iris(progress, DesertPrayerIrisCaller::PaletteFilterCase3);
            return true;
        }
        self.complete_desert_prayer_palette_filter_after_iris();
        if matches!(
            suffix_interruption,
            Some(
                crate::MainLoopInterruption::SpritePreparation
                    | crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }
            )
        ) {
            if let Some(interruption) = self.original_timing_main_loop_interruption() {
                let _ = self.take_original_timing_main_loop_interruption(interruption);
            }
            self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                NmiPrepareSpritesCpuCaller::DesertPrayer,
            );
            return true;
        }
        if self.pending_main_loop_common_suffix.is_some() {
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_rescued_maiden_initialization(
        &mut self,
        input: u16,
        stage: RescuedMaidenInitializationStage,
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
    ) -> bool {
        // The source host-return cursor has already committed the
        // exact decompression prefix. Either the terminal return
        // timeline or this host's own Sprite_Main activity proves
        // the remaining sheet/conversion/cutscene body returned.
        if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        }
        let nonterminal_sprite_main_returned = authoritative_scheduled_caller_return_timeline
            .is_none()
            && self.original_timing_owes_sprite_main_return();
        if let Some((_, _, sprite_main_return_claims)) =
            authoritative_scheduled_caller_return_timeline.as_ref()
        {
            self.begin_original_timing_sprite_main_return_claim_scope(*sprite_main_return_claims);
        } else if nonterminal_sprite_main_returned {
            self.begin_original_timing_sprite_main_return_claim_scope(1);
        }
        if authoritative_scheduled_caller_return_timeline.is_none() {
            if let Some(interruption) = authoritative_scheduled_caller_nmi_timeline
                .map(|timeline| timeline.interruption)
                .filter(|interruption| interruption.is_sprite_main())
            {
                if self.original_timing_main_loop_interruption() == Some(interruption) {
                    let _ = self.take_original_timing_main_loop_interruption_any();
                }
                self.forward_original_timing_main_loop_interruption_to_native_owner(
                    interruption,
                    OriginalTimingBoundary::NmiAccepted,
                );
            }
        }
        self.complete_rescued_maiden_initialization(stage);
        self.complete_module07_dungeon_after_submodule();
        if authoritative_scheduled_caller_return_timeline.is_some()
            || nonterminal_sprite_main_returned
        {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if authoritative_scheduled_caller_return_timeline.is_none()
            && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            self.retire_or_defer_main_loop_common_suffix_by_wire();
        } else if self.pending_main_loop_common_suffix.is_some() {
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_dungeon_falling_room_initialization(
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
    ) -> bool {
        // Dungeon_InitializeRoomFromSpecial returns through
        // Module07_07 and the ordinary game-loop suffix at the
        // wire's terminal host; the subsubmodule advance is the
        // call's own final statement (route host 91656).
        if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        }
        // Without a terminal return the wire may still publish
        // the module tail's Sprite_Main return inside this host
        // (route host 378771); the native tail below owns it and
        // the shared suffix stays with the wire.
        let nonterminal_sprite_main_returned = authoritative_scheduled_caller_return_timeline
            .is_none()
            && self.original_timing_owes_sprite_main_return();
        if let Some((_, _, sprite_main_return_claims)) =
            authoritative_scheduled_caller_return_timeline.as_ref()
        {
            self.begin_original_timing_sprite_main_return_claim_scope(*sprite_main_return_claims);
        } else if nonterminal_sprite_main_returned {
            self.begin_original_timing_sprite_main_return_claim_scope(1);
        }
        // The wire may suspend the module tail's Sprite_Main at a
        // slot boundary inside this host (route host 1449581:
        // MainLoopInterrupted(SpriteMainAfterSlot(11))); hand that
        // interruption to the Sprite_Main caller so the loop parks
        // there and the next host's return owns the remainder.
        if authoritative_scheduled_caller_return_timeline.is_none() {
            if let Some(interruption) = authoritative_scheduled_caller_nmi_timeline
                .map(|timeline| timeline.interruption)
                .filter(|interruption| interruption.is_sprite_main())
            {
                if self.original_timing_main_loop_interruption() == Some(interruption) {
                    let _ = self.take_original_timing_main_loop_interruption_any();
                }
                self.forward_original_timing_main_loop_interruption_to_native_owner(
                    interruption,
                    OriginalTimingBoundary::NmiAccepted,
                );
            }
        }
        self.dungeon_initialize_room_from_special_after_adjust();
        // The resumed caller falls out of Module07_07 into the
        // module tail: Sprite_Main, LinkOam, and the HUD suffix
        // run before the main-wait return.
        self.complete_module07_dungeon_after_submodule();
        if authoritative_scheduled_caller_return_timeline.is_some()
            || nonterminal_sprite_main_returned
        {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if authoritative_scheduled_caller_return_timeline.is_none()
            && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            self.retire_or_defer_main_loop_common_suffix_by_wire();
        } else if self.pending_main_loop_common_suffix.is_some() {
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_dungeon_supertile_transition(
        &mut self,
        input: u16,
        work: DungeonSupertileTransitionWork,
        authoritative_link_oam_equipment_prefix: Option<LinkOamStairProgress>,
        authoritative_scheduled_caller_nmi_timeline: Option<
            &OriginalTimingMainLoopInterruptionTimeline,
        >,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
        authoritative_supertile_sprite_main_returned: bool,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        // A wire-proven caller return names how many shared
        // Sprite_Main loops the resumed transition body runs;
        // scope them for whichever sub-continuation completes.
        // A caller-resume work runs no native body here — its
        // Sprite_Main crossed on an earlier native slice, so its
        // restated return is drained in-arm instead of scoped.
        let supertile_wire_claims = if matches!(
            work,
            DungeonSupertileTransitionWork::RoomLoadCallerResume
                | DungeonSupertileTransitionWork::SpriteConversionCallerResume
        ) {
            0
        } else {
            authoritative_scheduled_caller_return_timeline
                .as_ref()
                .map(|(_, _, claims)| *claims)
                .unwrap_or_else(|| {
                    // Falling graphics can return through Sprite_Main
                    // and suspend later in the HUD. That loop owns
                    // its return even while the common suffix is
                    // still pending; a terminal caller timeline is
                    // not required to consume this source fact.
                    if work == DungeonSupertileTransitionWork::FallingSpriteGraphics {
                        self.original_timing_semantic_receipts
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
                            .unwrap_or(0)
                    } else {
                        0
                    }
                })
        };
        if supertile_wire_claims != 0 {
            self.begin_original_timing_sprite_main_return_claim_scope(supertile_wire_claims);
        }
        let finish_supertile_claims = |state: &mut Self| {
            if supertile_wire_claims != 0 {
                state.finish_original_timing_sprite_main_return_claim_scope();
            }
        };
        match work {
            DungeonSupertileTransitionWork::RoomLoad => {
                // Dungeon_LoadRoom has returned before this NMI.
                // Begin the next real call on the same suspended
                // stack so its NMI request and graphics generation
                // are visible at the correct boundary.
                self.continue_module07_02_01_after_room_load();
            }
            DungeonSupertileTransitionWork::AuxiliarySpriteGraphics => {
                let mut schedule = self
                    .dungeon_room_load_cpu_schedule
                    .take()
                    .expect("room-load CPU schedule must survive auxiliary graphics");
                if self.original_timing_dungeon_push_blocks_pending()
                    || self
                        .original_timing_dungeon_push_blocks_in_progress()
                        .is_some()
                    || self.original_timing_dungeon_push_blocks_handled()
                {
                    assert_eq!(
                        supertile_wire_claims, 0,
                        "push-block entry precedes Sprite_Main"
                    );
                    self.complete_module07_02_01_after_auxiliary_sprite_graphics();
                    self.dungeon_room_load_module_suffix_nmi_slices = 0;
                    self.complete_module07_dungeon_after_submodule();
                    assert!(matches!(
                        self.game_execution_scheduler.current_work(),
                        Some(
                            GameWorkContinuation::FinishDungeonPushBlocks { .. }
                                | GameWorkContinuation::FinishDungeonPushBlockHandler { .. }
                        )
                    ));
                    return true;
                }
                self.complete_module07_02_01_before_dungeon_reset_sprites();
                let progress = self.take_original_timing_dungeon_reset_sprites_progress();
                let progressed_sprite_main_boundary =
                    self.original_timing_sprite_main_progress_boundary();
                let interrupted_sprite_main_boundary = authoritative_scheduled_caller_nmi_timeline
                    .filter(|timeline| timeline.interruption.is_sprite_main())
                    .and_then(|timeline| {
                        sprite_main_cpu_boundary_from_interruption(timeline.interruption)
                    });
                if let (Some(progressed), Some(interrupted)) = (
                    progressed_sprite_main_boundary,
                    interrupted_sprite_main_boundary,
                ) {
                    assert!(
                            same_sprite_main_source_checkpoint(progressed, interrupted),
                            "one auxiliary-graphics host reported incompatible Sprite_Main progress and interruption checkpoints",
                        );
                }
                // The scheduled timeline owns and consumes an NMI
                // interruption before this completion arm runs.
                // Preserve that exact slot statement in the room-
                // load schedule instead of looking only for an
                // unconsumed progress restatement (route host
                // 80273, state-8 property-reset prefix).
                let wire_sprite_main_boundary =
                    interrupted_sprite_main_boundary.or(progressed_sprite_main_boundary);
                // A wire host that already shows Sprite_Main (or its
                // cached-sprite tail) proves the reset ran here too;
                // the estimated prefix split cannot defer it.
                let wire_sprite_main_in_host =
                    matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && (self.original_timing_owes_sprite_main_return()
                            || authoritative_supertile_sprite_main_returned
                            || wire_sprite_main_boundary.is_some()
                            || self
                                .original_timing_cached_sprite_execution_progress()
                                .is_some()
                            || authoritative_scheduled_caller_nmi_timeline.is_some_and(
                                |timeline| {
                                    matches!(
                                        module_cpu_phase_from_main_loop_interruption(
                                            timeline.interruption,
                                        ),
                                        Some(
                                            ModuleCpuPhase::InterruptedAfterSpriteMain
                                                | ModuleCpuPhase::InterruptedInLinkOam
                                                | ModuleCpuPhase::InterruptedBeforeNmiPrepareSprites
                                                | ModuleCpuPhase::InterruptedInNmiPrepareSprites
                                        )
                                    )
                                },
                            ));
                if let Some(receipt) = progress {
                    // This semantic domain transfers only when the
                    // authority reports an exact source statement.
                    // Remove the old predicted prefix in that case;
                    // the typed boundary owns its replacement.
                    schedule.caller_nmis = schedule
                        .caller_nmis
                        .saturating_sub(schedule.caller_prefix_nmis);
                    schedule.caller_prefix_nmis = 0;
                    let progress = receipt.progress;
                    self.dungeon_reset_sprites_through_cpu_progress(progress);
                    // In C this state publication precedes
                    // Dungeon_ResetSprites. The ROM can cross NMI
                    // inside that reset, leaving the old live slots
                    // observable with the advanced state.
                    self.dungeon_room_load_cpu_schedule = Some(schedule);
                    let continuation = GameWorkContinuation::FinishDungeonSupertileTransition {
                        work: DungeonSupertileTransitionWork::RoomLoadSpriteReset { progress },
                    };
                    match receipt.boundary {
                        OriginalTimingBoundary::HostReturn => {
                            let scheduled = self.begin_dungeon_supertile_transition_work(
                                DungeonSupertileTransitionWork::RoomLoadSpriteReset { progress },
                            );
                            debug_assert!(scheduled);
                        }
                        OriginalTimingBoundary::NmiAccepted => self
                            .game_execution_scheduler
                            .schedule_after_current_trailing_nmi(continuation),
                    }
                } else if schedule.caller_prefix_nmis != 0 && !wire_sprite_main_in_host {
                    // No typed receipt means this domain has not
                    // transferred. Preserve the existing scheduler
                    // owner until Snes9x (or a native shadow) emits
                    // the semantic boundary.
                    self.dungeon_room_load_cpu_schedule = Some(schedule);
                    let scheduled = self.begin_dungeon_supertile_transition_work(
                        DungeonSupertileTransitionWork::RoomLoadSpriteResetFallback,
                    );
                    debug_assert!(scheduled);
                } else {
                    self.complete_module07_02_01_from_dungeon_reset_sprites();
                    let mut schedule = schedule;
                    let wire_owns_return = authoritative_scheduled_caller_return_timeline.is_some();
                    if wire_owns_return {
                        // The wire-proven caller return supersedes
                        // the schedule's remaining NMI budget: the
                        // reset, room-load caller, Sprite_Main,
                        // and shared suffix all completed inside
                        // this one host.
                        schedule.caller_nmis = 0;
                        schedule.caller_prefix_nmis = 0;
                        schedule.caller_sprite_main_nmis = 0;
                        schedule.caller_suffix_nmis = 0;
                    } else if wire_sprite_main_in_host {
                        // Sprite_Main ran inside this host without
                        // a terminal return: its cached-sprite tail
                        // crosses the boundary when the wire says so,
                        // and the shared suffix stays with the wire
                        // (route host 688997).
                        let cached_tail_crosses = self
                            .original_timing_cached_sprite_execution_progress()
                            .is_some();
                        schedule.caller_prefix_nmis = 0;
                        schedule.caller_sprite_main_nmis =
                            u8::from(cached_tail_crosses || wire_sprite_main_boundary.is_some());
                        schedule.caller_suffix_nmis =
                            u8::from(!cached_tail_crosses && wire_sprite_main_boundary.is_none());
                        schedule.caller_nmis = 1;
                        schedule.sprite_main_boundary = wire_sprite_main_boundary;
                    }
                    let nonterminal_sprite_main_claim =
                        !wire_owns_return && self.original_timing_owes_sprite_main_return();
                    if nonterminal_sprite_main_claim {
                        self.begin_original_timing_sprite_main_return_claim_scope(1);
                    }
                    let completed = self.continue_dungeon_room_load_after_sprite_reset(
                        schedule,
                        input,
                        oam_dma_source.as_deref(),
                        wire_owns_return,
                    );
                    if nonterminal_sprite_main_claim {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                    if completed {
                        finish_supertile_claims(self);
                        return true;
                    }
                }
            }
            DungeonSupertileTransitionWork::RoomLoadSpriteReset { progress } => {
                if self.complete_room_load_sprite_reset_after_timing_boundary(
                    progress,
                    input,
                    oam_dma_source.as_deref(),
                ) {
                    finish_supertile_claims(self);
                    return true;
                }
            }
            DungeonSupertileTransitionWork::RoomLoadSpriteResetFallback => {
                self.complete_module07_02_01_from_dungeon_reset_sprites();
                let schedule = self
                    .dungeon_room_load_cpu_schedule
                    .take()
                    .expect("room-load CPU schedule must survive sprite reset");
                if self.continue_dungeon_room_load_after_sprite_reset(
                    schedule,
                    input,
                    oam_dma_source.as_deref(),
                    false,
                ) {
                    finish_supertile_claims(self);
                    return true;
                }
            }
            DungeonSupertileTransitionWork::SpriteConversion => {
                self.complete_dungeon_inter_room_transition_state3_after_sprite_conversion();
                let mut schedule = self
                    .dungeon_submodule_cpu_schedule
                    .take()
                    .expect("sprite conversion requires its ROM CPU schedule");
                // The estimated envelope may place the caller's one
                // NMI after Sprite_Main returned while the wire
                // names a slot boundary inside it (route host
                // 988054, AfterSlot(1)): the wire's checkpoint is
                // the caller's Sprite_Main suspension.
                if let Some(boundary) = self
                    .original_timing_main_loop_interruption()
                    .and_then(sprite_main_cpu_boundary_from_interruption)
                {
                    if schedule.caller_nmis == 1 && schedule.caller_sprite_main_nmis == 0 {
                        schedule.caller_sprite_main_nmis = 1;
                        schedule.caller_suffix_nmis = 0;
                        schedule.sprite_main_boundary = Some(boundary);
                        schedule.caller_first_nmi_phase =
                            Some(ModuleCpuPhase::InterruptedInSpriteMain);
                    }
                }
                // A terminal return in this host proves the whole
                // caller suffix ran here (route host 1136316:
                // [Handler, SpriteMainReturned, Continued,
                // SuffixCompleted] against an estimated
                // prep-interrupted split).
                if schedule.caller_sprite_main_nmis == 0
                    && authoritative_scheduled_caller_return_timeline.is_some()
                {
                    schedule.caller_suffix_nmis = 0;
                    schedule.caller_nmis = 0;
                }
                if schedule.caller_sprite_main_nmis != 0 {
                    assert_eq!(
                        schedule.caller_nmis, schedule.caller_sprite_main_nmis,
                        "sprite-conversion caller has work after Sprite_Main",
                    );
                    let cached_sprite_interruption = self
                        .take_authoritative_cached_sprite_interruption(
                            schedule.cached_sprite_interruption,
                        );
                    if let Some((boundary, authority_boundary)) = cached_sprite_interruption {
                        assert_eq!(
                            schedule.caller_sprite_main_nmis, 1,
                            "cached-sprite conversion continuation crossed more than one NMI",
                        );
                        assert!(
                            self.dungeon_cached_sprite_cpu_interruption_pending
                                .replace(boundary)
                                .is_none(),
                            "cached-sprite conversion continuation was already armed",
                        );
                        self.dungeon_cached_sprite_cpu_interruption_boundary = authority_boundary;
                        // Sprite conversion returns into the quadrant-upload
                        // chain. Its cached-sprite caller must retire at main
                        // wait and prepare the next leading-NMI CPU advance,
                        // just like a cached interruption inside that chain.
                        // Otherwise the generic return appends an extra NMI
                        // and the next quadrant runs without its CPU phase.
                        self.dungeon_quadrant_cpu_continuation_active = true;
                    } else {
                        let boundary = schedule
                            .sprite_main_boundary
                            .unwrap_or(SpriteMainCpuBoundary::BeforeFirstSlot);
                        self.sprite_main_cpu_boundary = Some(boundary);
                        self.sprite_main_cpu_nmi_slices = schedule.caller_sprite_main_nmis;
                    }
                    self.complete_module07_dungeon_after_submodule();
                    debug_assert!(self
                        .game_execution_scheduler
                        .work_suspends_translated_call_stack());
                } else if schedule.caller_suffix_nmis == 0 {
                    assert_eq!(schedule.caller_nmis, 0);
                    self.complete_module07_dungeon_after_submodule();
                    // The suspended iteration's shared suffix owner
                    // (armed when the conversion held the fresh
                    // iteration) retires here; running the prep
                    // directly would strand it and reject the next
                    // host's install (route host 101340).
                    self.retire_or_run_main_loop_common_suffix_after_module_return();
                } else {
                    assert_eq!(
                        schedule.caller_nmis, schedule.caller_suffix_nmis,
                        "sprite-conversion suffix phase must cover the remaining caller NMIs",
                    );
                    assert_eq!(
                        schedule.caller_suffix_nmis, 1,
                        "sprite-conversion suffix crossed more than one NMI",
                    );
                    match schedule.caller_first_nmi_phase {
                            Some(ModuleCpuPhase::InterruptedInLinkOam) => {
                                // Sprite_Main returned, then vblank
                                // interrupted LinkOam_Main. The existing
                                // semantic continuation resumes that C
                                // suffix without replaying Sprite_Main.
                                self.dungeon_post_sprite_main_return_pending = true;
                                self.complete_module07_dungeon_after_submodule();
                                debug_assert!(matches!(
                                    self.game_execution_scheduler.current_work(),
                                    Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn)
                                ));
                            }
                            Some(
                                ModuleCpuPhase::InterruptedInNmiPrepareSprites,
                            ) => {
                                // C has returned from Module 7 and
                                // entered the main-loop OAM packing
                                // suffix. Finish Module 7 now, then
                                // resume only NMI_PrepareSprites.
                                self.complete_module07_dungeon_after_submodule();
                                self.interrupted_nmi_prepare_obj_cache_vram = Some(
                                    self.last_presented_obj_vram
                                        .as_ref()
                                        .cloned()
                                        .or_else(|| self.ppu.obj_vram_latch.clone())
                                        .unwrap_or_else(|| self.ppu.vram.clone()),
                                );
                                self.game_execution_scheduler.schedule_work(
                                    GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn {
                                        caller: NmiPrepareSpritesCpuCaller::DungeonModule07,
                                    },
                                    schedule.caller_suffix_nmis,
                                );
                            }
                            phase => panic!(
                                "sprite-conversion caller crossed an unsupported suffix phase: {phase:?}"
                            ),
                        }
                }
            }
            DungeonSupertileTransitionWork::FilteredQuadrantTilemapBuild => {
                // The palette walk plus quadrant build crosses NMI,
                // then returns through the common Module 7 suffix
                // before the following fresh state-11 iteration.
                self.complete_dungeon_inter_room_transition_not_dark_room();
                self.complete_module07_dungeon_after_submodule();
                self.retire_or_run_main_loop_common_suffix_after_module_return();
                self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                    PreMainNmiResume::DungeonSupertileQuadrantUploads,
                );
            }
            DungeonSupertileTransitionWork::QuadrantUploadCallerReturn => {
                // State 11 prepared its quadrant upload before the
                // interrupt. Only the suspended caller return and
                // the common Module 7 suffix remain on this host;
                // state 12 must not begin until the next one.
                self.increment_subsubmodule();
                self.complete_module07_dungeon_after_submodule();
                self.retire_or_run_main_loop_common_suffix_after_module_return();
                self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                    PreMainNmiResume::DungeonSupertileQuadrantUploads,
                );
            }
            DungeonSupertileTransitionWork::State13CallerReturn => {
                // State 13's movement/palette body completed before
                // vblank. Resume only the common Module 7 suffix on
                // this CPU generation; the next movement iteration
                // begins on the following host frame.
                self.dungeon_state_13_caller_return_publication_host_frame =
                    Some(self.frame_ctr_dbg);
                self.complete_module07_dungeon_after_submodule();
                self.retire_or_run_main_loop_common_suffix_after_module_return();
            }
            work @ (DungeonSupertileTransitionWork::FadedFilterPreCompletionCallerReturn
            | DungeonSupertileTransitionWork::FadedFilterCallerReturn) => {
                // The landing fade advanced to state 15 before
                // vblank, or completed its penultimate countdown
                // write while state 14 was still current. Resume
                // only the ordinary Module 7 suffix on this CPU
                // generation; the next state iteration begins on
                // the following host.
                self.dungeon_faded_filter_caller_return_publication_host_frame =
                    Some(self.frame_ctr_dbg);
                if work == DungeonSupertileTransitionWork::FadedFilterCallerReturn {
                    self.dungeon_post_landing_leading_nmi_room =
                        Some(self.game_state.world.location.dungeon_room_index());
                }
                self.complete_module07_dungeon_after_submodule();
                self.retire_or_run_main_loop_common_suffix_after_module_return();
            }
            DungeonSupertileTransitionWork::SpiralRoomInitialization => {
                self.increment_subsubmodule();
                let schedule = self
                    .dungeon_submodule_cpu_schedule
                    .take()
                    .expect("spiral-room completion requires its CPU schedule");
                if crate::debug_env::var_os("ZELDA3_DEBUG_SPIRAL_ROOM_INIT").is_some() {
                    eprintln!(
                            "[SPIRAL-INIT] host={} caller_nmis={} sprite_main_nmis={} suffix_nmis={} resumed_this_host={} owes_return={} owes_progress={} fresh={} interruption={:?} boundary={:?} claims={:?}",
                            self.frame_ctr_dbg,
                            schedule.caller_nmis,
                            schedule.caller_sprite_main_nmis,
                            schedule.caller_suffix_nmis,
                            self.wire_spiral_caller_resumed_this_host(),
                            self.original_timing_owes_sprite_main_return(),
                            self.original_timing_owes_sprite_main_progress(),
                            self.original_timing_hosts_fresh_iteration(),
                            self.original_timing_main_loop_interruption(),
                            schedule.sprite_main_boundary,
                            self.original_timing_sprite_main_return_claims_remaining,
                        );
                }
                if schedule.caller_nmis == 0 || self.wire_spiral_caller_resumed_this_host() {
                    self.complete_module07_dungeon_after_submodule();
                    if !self
                        .game_execution_scheduler
                        .work_suspends_translated_call_stack()
                    {
                        // The wire can hold the shared suffix past
                        // this host behind a trailing Held
                        // acceptance (route host 237365); a
                        // terminal return already consumed its
                        // suffix receipt and retires it here.
                        if authoritative_scheduled_caller_return_timeline.is_some() {
                            self.retire_or_run_main_loop_common_suffix_after_module_return();
                        } else {
                            self.retire_or_defer_main_loop_common_suffix_by_wire();
                        }
                    }
                } else {
                    self.game_execution_scheduler.schedule_work(
                        GameWorkContinuation::FinishDungeonSupertileTransition {
                            work: DungeonSupertileTransitionWork::SpiralRoomCallerResume,
                        },
                        schedule.caller_nmis,
                    );
                }
            }
            DungeonSupertileTransitionWork::SpiralBackgroundSync => {
                self.complete_dungeon_inter_room_transition_not_dark_room();
                let schedule = self
                    .dungeon_submodule_cpu_schedule
                    .take()
                    .expect("spiral background completion requires its CPU schedule");
                if schedule.caller_nmis == 0 || self.wire_spiral_caller_resumed_this_host() {
                    self.complete_module07_dungeon_after_submodule();
                    if !self
                        .game_execution_scheduler
                        .work_suspends_translated_call_stack()
                    {
                        // The wire can hold the shared suffix past
                        // this host behind a trailing Held
                        // acceptance (route host 237365); a
                        // terminal return already consumed its
                        // suffix receipt and retires it here.
                        if authoritative_scheduled_caller_return_timeline.is_some() {
                            self.retire_or_run_main_loop_common_suffix_after_module_return();
                        } else {
                            self.retire_or_defer_main_loop_common_suffix_by_wire();
                        }
                    }
                } else {
                    self.game_execution_scheduler.schedule_work(
                        GameWorkContinuation::FinishDungeonSupertileTransition {
                            work: DungeonSupertileTransitionWork::SpiralRoomCallerResume,
                        },
                        schedule.caller_nmis,
                    );
                }
            }
            DungeonSupertileTransitionWork::StraightInterroomRoomInitialization => {
                // The straight-stair room initializer returns after
                // the final interrupted NMI, then completes its
                // ordinary Module 7 caller suffix in the same host
                // slice. The next state-3 iteration begins on the
                // following host frame.
                self.increment_subsubmodule();
                self.complete_resumed_module07_caller_suffix_by_wire(
                    authoritative_scheduled_caller_return_timeline.is_some(),
                    authoritative_link_oam_equipment_prefix,
                );
                if let Some(oam) = self
                    .active_display_obj_generation
                    .retained_oam()
                    .map(<[u16]>::to_vec)
                {
                    // The completion scanout still owns the held
                    // table; release normal cadence only after its
                    // one-shot queued generation is captured.
                    self.next_display_obj_memory_generation =
                        Some(DisplayObjGeneration::RetainCapturedOam { oam });
                }
                self.active_display_obj_generation = DisplayObjGeneration::FollowModuleCadence;
            }
            DungeonSupertileTransitionWork::StraightInterroomBgCharacters34 => {
                // The 3bpp-to-4bpp conversion returns after its
                // fourth interrupted NMI. Complete state 3 and the
                // Module 7 caller suffix without starting state 4
                // until the following host frame.
                self.increment_subsubmodule();
                self.complete_module07_dungeon_after_submodule();
                self.retire_or_run_main_loop_common_suffix_after_module_return();
                if let Some(oam) = self
                    .active_display_obj_generation
                    .retained_oam()
                    .map(<[u16]>::to_vec)
                {
                    self.next_display_obj_memory_generation =
                        Some(DisplayObjGeneration::RetainCapturedOam { oam });
                }
                self.active_display_obj_generation = DisplayObjGeneration::FollowModuleCadence;
            }
            DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics => {
                // The converted sprite sheet returns after four
                // interrupted NMIs. Only then may the palette tail
                // advance state 9 to state 10.
                self.Dungeon_HandleTranslucencyAndPalette();
                // The wire may suspend the module tail's cached-
                // sprite execution at the host return (route host
                // 1409941: CachedSpriteExecutionProgress Loading
                // slot 4, HostReturn); arm that boundary so
                // ExecuteCachedSprites parks there.
                if let Some((boundary, authority_boundary)) =
                    self.take_authoritative_cached_sprite_interruption(None)
                {
                    assert!(
                        self.dungeon_cached_sprite_cpu_interruption_pending
                            .replace(boundary)
                            .is_none(),
                        "cached-sprite interroom continuation was already armed",
                    );
                    self.dungeon_cached_sprite_cpu_interruption_boundary = authority_boundary;
                }
                self.complete_resumed_module07_caller_suffix_by_wire(
                    authoritative_scheduled_caller_return_timeline.is_some(),
                    authoritative_link_oam_equipment_prefix,
                );
                if let Some(oam) = self
                    .active_display_obj_generation
                    .retained_oam()
                    .map(<[u16]>::to_vec)
                {
                    self.next_display_obj_memory_generation =
                        Some(DisplayObjGeneration::RetainCapturedOam { oam });
                }
                self.active_display_obj_generation = DisplayObjGeneration::FollowModuleCadence;
            }
            DungeonSupertileTransitionWork::SpiralRoomCallerResume => {
                self.complete_module07_dungeon_after_submodule();
                if self.pending_main_loop_common_suffix.is_some() {
                    // The source-proven caller return already
                    // carries the shared suffix; retire its owner.
                    self.complete_pending_main_loop_common_suffix_after_module_return();
                } else {
                    if self.pending_main_loop_common_suffix.is_some() {
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                }
            }
            DungeonSupertileTransitionWork::SpiralBgCharacters34
            | DungeonSupertileTransitionWork::FallingBgCharacters34 => {
                self.increment_subsubmodule();
                self.complete_module07_dungeon_after_submodule();
                if authoritative_scheduled_caller_return_timeline.is_some() {
                    // The validated timeline already consumed the
                    // suffix receipt before this native caller ran.
                    self.retire_or_run_main_loop_common_suffix_after_module_return();
                } else {
                    self.retire_or_defer_main_loop_common_suffix_by_wire();
                }
            }
            DungeonSupertileTransitionWork::FallingSpriteGraphics => {
                self.complete_dungeon_transition_load_sprite_gfx();
                self.complete_module07_dungeon_after_submodule();
                // Sprite_Main can suspend again while initializing
                // a newly loaded slot. Its caller still owns the
                // common suffix until the source return arrives.
                if authoritative_scheduled_caller_return_timeline.is_some() {
                    self.retire_or_run_main_loop_common_suffix_after_module_return();
                } else {
                    self.retire_or_defer_main_loop_common_suffix_by_wire();
                }
            }
            DungeonSupertileTransitionWork::SpiralSpriteGraphics => {
                self.complete_dungeon_transition_load_sprite_gfx();
                let mut schedule = self
                    .dungeon_submodule_cpu_schedule
                    .take()
                    .expect("spiral sprite completion requires its CPU schedule");
                // The sprite-graphics body can return early enough
                // for its Module 7 caller to enter a fresh
                // Sprite_Main in this same host.  A standalone
                // SpriteMainProgressed receipt names the exact
                // source statement reached even when the
                // interrupting NMI acceptance heads the following
                // host vector (route host 722099: the source stops
                // after slot 0's timer/OAM prefix).  Refine the
                // saved caller schedule before executing any of
                // that Sprite_Main; otherwise the raster estimate
                // can run the slot's AI body one host too early.
                let authoritative_sprite_main_progress =
                    self.take_original_timing_sprite_main_progress();
                // The wire's in-host interruption timeline supersedes
                // the estimated caller-suffix split: Sprite_Main
                // returned and vblank interrupted LinkOam_Main /
                // NMI_PrepareSprites, so the Module 7 suffix and the
                // shared ZeldaRunGameLoop suffix cross to the next
                // host (route host 357284: [NmiHandlerCompleted,
                // SpriteMainReturned, NmiAccepted(LatchHeld),
                // MainLoopInterrupted(LinkOam), CallStackContinued]).
                let wire_interrupts_caller_suffix = schedule.caller_sprite_main_nmis == 0
                    && authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                        timeline.progress == crate::MainLoopProgress::CallStackContinued
                            && matches!(
                                timeline.interruption,
                                crate::MainLoopInterruption::LinkOam
                                    | crate::MainLoopInterruption::SpritePreparation
                            )
                    });
                // Conversely a terminal return in this host proves
                // the whole caller suffix ran here (route host
                // 470070: [NmiHandlerCompleted, SpriteMainReturned,
                // CallStackContinued, MainLoopCommonSuffixCompleted,
                // NmiAccepted(Open)] against an estimated split).
                let wire_completes_caller_suffix = schedule.caller_sprite_main_nmis == 0
                    && authoritative_scheduled_caller_return_timeline.is_some();
                let mut caller_suffix_nmis = if wire_interrupts_caller_suffix {
                    schedule.caller_suffix_nmis.max(1)
                } else if wire_completes_caller_suffix {
                    0
                } else {
                    schedule.caller_suffix_nmis
                };
                // The wire may suspend the module tail's Sprite_Main
                // at a slot boundary inside this host (route host
                // 1449586: SpriteMainAfterSlot(2)); hand that
                // interruption to the Sprite_Main caller so the loop
                // parks there and the next host's return owns the
                // remainder and the shared suffix.
                let authoritative_interruption_boundary =
                    authoritative_scheduled_caller_nmi_timeline
                        .map(|timeline| timeline.interruption)
                        .filter(|interruption| interruption.is_sprite_main())
                        .and_then(sprite_main_cpu_boundary_from_interruption);
                if let (Some(interrupted), Some(progress)) = (
                    authoritative_interruption_boundary,
                    authoritative_sprite_main_progress,
                ) {
                    assert_eq!(
                        interrupted, progress,
                        "the spiral sprite caller's interruption and progress checkpoints disagree",
                    );
                }
                if let Some(boundary) =
                    authoritative_interruption_boundary.or(authoritative_sprite_main_progress)
                {
                    if schedule.caller_sprite_main_nmis == 0 {
                        // The estimate saw no Sprite_Main crossing:
                        // the source checkpoint proves the fresh
                        // Sprite_Main itself crosses the host
                        // boundary.  A full interruption receipt
                        // additionally carries the main-loop
                        // ownership token; a standalone progress
                        // checkpoint does not need one.
                        if let Some(interruption) = authoritative_scheduled_caller_nmi_timeline
                            .map(|timeline| timeline.interruption)
                            .filter(|interruption| interruption.is_sprite_main())
                        {
                            if self.original_timing_main_loop_interruption() == Some(interruption) {
                                let _ = self.take_original_timing_main_loop_interruption_any();
                            }
                            self.forward_original_timing_main_loop_interruption_to_native_owner(
                                interruption,
                                OriginalTimingBoundary::NmiAccepted,
                            );
                        }
                        schedule.caller_nmis = schedule.caller_nmis.max(1);
                        schedule.caller_sprite_main_nmis = 1;
                        schedule.caller_suffix_nmis = 0;
                        schedule.caller_first_nmi_phase =
                            Some(ModuleCpuPhase::InterruptedInSpriteMain);
                        schedule.sprite_main_boundary = Some(boundary);
                    } else {
                        // The estimate predicted a crossing at a
                        // different slot; the wire's slot is the
                        // one the ROM suspended on (route host
                        // 1490403: AfterSlot(4) against an estimate
                        // that let a lower Bari slot prep its RNG).
                        schedule.sprite_main_boundary = Some(boundary);
                    }
                    caller_suffix_nmis = 0;
                }
                self.dungeon_room_load_module_suffix_nmi_slices = caller_suffix_nmis;
                if schedule.caller_sprite_main_nmis != 0 {
                    let boundary = schedule
                        .sprite_main_boundary
                        .unwrap_or(SpriteMainCpuBoundary::BeforeFirstSlot);
                    self.sprite_main_cpu_boundary = Some(boundary);
                    self.sprite_main_cpu_nmi_slices = schedule.caller_sprite_main_nmis;
                    self.complete_module07_dungeon_after_submodule();
                    debug_assert!(self
                        .game_execution_scheduler
                        .work_suspends_translated_call_stack());
                } else if caller_suffix_nmis == 0 {
                    self.complete_module07_dungeon_after_submodule();
                    if self
                        .game_execution_scheduler
                        .work_suspends_translated_call_stack()
                    {
                        // The wire parked the module tail's
                        // Sprite_Main at a slot boundary; its
                        // completion owns the suffix and the
                        // following resume (route host 1449586).
                        return true;
                    }
                    let mut suffix_deferred_by_wire = false;
                    if authoritative_scheduled_caller_return_timeline.is_some() {
                        // The terminal return already consumed its
                        // suffix receipt into the retained timeline
                        // (route host 898148); retire it here.
                        self.retire_or_run_main_loop_common_suffix_after_module_return();
                    } else if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && self.original_timing_semantic_receipts.is_some()
                    {
                        // The wire may hold the shared suffix past
                        // this host ([Handler, SpriteMainReturned,
                        // Continued] at route host 925558); the
                        // caller then returns and clears the latch
                        // behind the next held acceptance.
                        suffix_deferred_by_wire = self.original_timing_live_suffix_outstanding();
                        self.retire_or_defer_main_loop_common_suffix_by_wire();
                    } else if self.pending_main_loop_common_suffix.is_some() {
                        self.complete_pending_main_loop_common_suffix_after_module_return();
                    } else {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                        if suffix_deferred_by_wire {
                            PreMainNmiResume::DungeonSupertileCallerReturnNmi
                        } else {
                            PreMainNmiResume::DungeonSupertileQuadrantUploads
                        },
                    );
                } else {
                    self.complete_module07_dungeon_after_submodule();
                }
            }
            work @ (DungeonSupertileTransitionWork::RoomLoadCallerResume
            | DungeonSupertileTransitionWork::SpriteConversionCallerResume) => {
                // Snes9x returns the host call at the NMI that
                // interrupted the long Module 7 stack. Resume the
                // common sprite/HUD suffix on the following host
                // call, just as the ROM resumes its interrupted
                // caller. In particular, sprite initialization RNG
                // belongs to this post-NMI generation.
                if authoritative_scheduled_caller_return_timeline.is_some() {
                    // The wire's terminal return restates the
                    // Sprite_Main return that this resumed caller
                    // consumed on an earlier native slice; the
                    // scope above stays unopened for this work.
                    let _ = self.take_original_timing_sprite_main_returned();
                }
                self.complete_dungeon_supertile_caller_return_host(
                    work,
                    input,
                    oam_dma_source.as_deref(),
                    authoritative_scheduled_caller_return_timeline.is_some(),
                );
                return true;
            }
        }
        finish_supertile_claims(self);
        false
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_pre_dungeon_entrance_load(
        &mut self,
        input: u16,
        sprite_reset: PreDungeonSpriteResetContinuation,
        authoritative_pre_dungeon_progress: Option<MainLoopProgress>,
        authoritative_pre_dungeon_returned: bool,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        self.complete_pre_dungeon_sprite_reset_continuation(sprite_reset);
        if authoritative_scheduled_caller_return_timeline.is_some() {
            // The wire-proven terminal return owns its leading
            // handler and trailing publication through the shared
            // lane; only the module return and its ordinary
            // ZeldaRunGameLoop suffix run here (route host 39723).
            self.complete_pre_dungeon_authoritative_return_after(sprite_reset);
            self.retire_or_run_main_loop_common_suffix_after_module_return();
            return true;
        }
        if authoritative_pre_dungeon_returned
            && matches!(
                authoritative_pre_dungeon_progress,
                Some(crate::MainLoopProgress::IterationStarted)
            )
        {
            // The continuous authority observed Module_PreDungeon
            // return, the enclosing ZeldaRunGameLoop finish its
            // Main_PrepSpritesForNmi/$12 suffix, and the following
            // ZeldaRunGameLoop begin before this host interval's
            // trailing NMI. Preserve that C order. Capturing before
            // these writes retained the old CGWSEL generation even
            // though WritePpuRegisters installed the new one for the
            // active dungeon scanout.
            self.complete_pre_dungeon_authoritative_return_after(sprite_reset);
            self.nmi_prepare_sprites_for_main_loop_once();
            self.clear_nmi_update_latch();
            self.game_execution_scheduler
                .prepare_same_host_main_iteration_after_authoritative_return();
            self.zelda_run_game_loop_with_progress(Some(crate::MainLoopProgress::IterationStarted));
            self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return true;
        }
        if authoritative_pre_dungeon_returned {
            assert_eq!(
                authoritative_pre_dungeon_progress,
                Some(crate::MainLoopProgress::CallStackContinued),
                "Module_PreDungeon returned without a valid main-loop progress receipt",
            );
            // The C module return is complete, but this host call
            // ended before another ZeldaRunGameLoop iteration or
            // NMI began. Publish the source-ordered module state and
            // leave both later boundaries to their own receipts.
            self.complete_pre_dungeon_authoritative_return_after(sprite_reset);
            return true;
        }
        // The final sprite-reset slice is interrupted before
        // Module_PreDungeon publishes module 7 and releases the
        // main-loop NMI latch. Resume that caller suffix only
        // after this scanout boundary.
        self.capture_display_snapshot();
        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        self.complete_pre_dungeon_entrance_load_after(sprite_reset);
        if self.game_execution_scheduler.work_is_pending() {
            return true;
        }
        self.nmi_prepare_sprites();
        self.clear_nmi_update_latch();
        let progress = authoritative_pre_dungeon_progress
            .or_else(|| self.finish_pre_dungeon_caller_at_main_wait());
        if let Some(progress) = progress {
            debug_assert_eq!(progress, crate::MainLoopProgress::IterationStarted);
            self.game_execution_scheduler
                .mark_main_iteration_after_leading_nmi();
            self.zelda_run_game_loop_with_progress(Some(progress));
            self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_pre_dungeon_song_bank_transfer(
        &mut self,
        input: u16,
        authoritative_pre_dungeon_progress: Option<MainLoopProgress>,
        authoritative_pre_dungeon_returned: bool,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if authoritative_pre_dungeon_returned
            && matches!(
                authoritative_pre_dungeon_progress,
                Some(crate::MainLoopProgress::IterationStarted)
            )
        {
            self.complete_module_pre_dungeon_after_song_bank_transfer();
            self.nmi_prepare_sprites_for_main_loop_once();
            self.clear_nmi_update_latch();
            self.game_execution_scheduler
                .prepare_same_host_main_iteration_after_authoritative_return();
            self.zelda_run_game_loop_with_progress(Some(crate::MainLoopProgress::IterationStarted));
            self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return true;
        }
        if authoritative_pre_dungeon_returned {
            assert_eq!(
                authoritative_pre_dungeon_progress,
                Some(crate::MainLoopProgress::CallStackContinued),
                "Module_PreDungeon song-bank return omitted valid main-loop progress",
            );
            self.complete_module_pre_dungeon_after_song_bank_transfer();
            return true;
        }
        // The upload receiver was selected when the transfer
        // began. The original caller remains suspended through
        // this boundary, so finish its semantic suffix only after
        // the copy returns.
        self.capture_display_snapshot();
        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        self.complete_module_pre_dungeon_after_song_bank_transfer();
        self.nmi_prepare_sprites();
        self.clear_nmi_update_latch();
        let progress = authoritative_pre_dungeon_progress
            .or_else(|| self.finish_pre_dungeon_caller_at_main_wait());
        if let Some(progress) = progress {
            debug_assert_eq!(progress, crate::MainLoopProgress::IterationStarted);
            self.game_execution_scheduler
                .mark_main_iteration_after_leading_nmi();
            self.zelda_run_game_loop_with_progress(Some(progress));
            self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_item_receipt_graphics(
        &mut self,
        input: u16,
        continuation: ItemReceiptGraphicsContinuation,
        authoritative_scheduled_caller_accepts_nmi_at_return: Option<bool>,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if let ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            ground_apress_tail: Some(receipt),
            ..
        } = continuation
        {
            // AncillaAdd_ItemReceipt is a synchronous C call. Its
            // decompressor blocks the ancilla tail, Link_ReceiveItem,
            // the rest of HandleLink_From1D, and Module 7's entire
            // post-submodule suffix. Resume that call chain in order;
            // Sprite_Main must not run while the ROM CPU is still
            // inside the decompressor.
            self.complete_ancilla_add_item_receipt(receipt);
            self.complete_link_receive_item(receipt.item);
            self.player_handler_00_ground_3_after_a_press(true);
            self.complete_module07_dungeon_after_submodule();
            if self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
            {
                return true;
            }
            self.complete_module07_dungeon_after_submodule_caller();
            if self.game_execution_scheduler.work_is_pending() {
                return true;
            }
            self.item_receipt_completion_live_link_dma_host = Some(self.frame_ctr_dbg);
        }
        // The $21 item sheet returns through the ordinary module
        // epilogue in v1.0.0: sprite preparation releases the NMI
        // latch, then the common boundary below publishes the
        // complete OAM image. Keep the measured retained path while
        // Link is still in handler 21, but once that receipt has
        // ended it would retain an older packed OAM size bit after
        // the lower sprite entry has already advanced.
        let ordinary_gfx_21_return =
            self.item_receipt_graphics_return_uses_ordinary_module_epilogue(continuation);
        if !ordinary_gfx_21_return && authoritative_scheduled_caller_accepts_nmi_at_return.is_none()
        {
            // The final measured slice is the vblank that interrupts
            // the decompressor itself. The software NMI latch is still
            // set here, so hardware retains its resident OAM and Link
            // tiles. Dungeon animated tiles are a separate DMA domain
            // which still completes before this scanout is captured.
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
        if matches!(
            continuation,
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x22, .. }
        ) {
            self.retire_enemy_drop_item_graphics_sound_effect_2();
        }
        if let Some((_, _, sprite_main_return_claims)) =
            authoritative_scheduled_caller_return_timeline.as_ref()
        {
            self.begin_original_timing_sprite_main_return_claim_scope(*sprite_main_return_claims);
        }
        if let ItemReceiptGraphicsContinuation::ResumeUnclePassage {
            receipt,
            sprite_slot,
            dungeon,
        } = continuation
        {
            self.complete_ancilla_add_item_receipt(receipt);
            self.complete_link_receive_item(receipt.item);
            self.complete_uncle_passage_item_receipt(sprite_slot as usize);
            self.complete_sprite_main_after_interrupted_slot(sprite_slot as usize);
            self.complete_module07_after_sprite_main(dungeon);
        }
        if let ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt {
            receipt,
            sprite_slot,
            suffix,
            caller,
        } = continuation
        {
            let sprite_slot = usize::from(sprite_slot);
            self.complete_ancilla_add_item_receipt(receipt);
            self.complete_link_receive_item(receipt.item);
            match suffix {
                SpriteMainItemReceiptSuffix::BottleVendor => {
                    self.complete_bottle_vendor_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::SickKid => {
                    self.complete_sick_kid_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::BigKeyAbsorption => {
                    self.complete_big_key_absorption_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::HeartContainerFull => {
                    self.complete_heart_container_full_item_receipt()
                }
                SpriteMainItemReceiptSuffix::HeartContainerUpgrade => {
                    self.complete_heart_container_upgrade_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::SahasrahlaBoots => {
                    self.complete_sahasrahla_boots_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::BookOfMudora => {
                    self.complete_book_of_mudora_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::CatfishMedallion => {
                    self.complete_catfish_medallion_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::HappinessPondReward => {
                    self.complete_happiness_pond_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::HoboBottle => {
                    self.complete_hobo_bottle_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::Mushroom => {}
                SpriteMainItemReceiptSuffix::OldManMirror => {
                    self.complete_old_man_mirror_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::ShopItem { flags4 } => {
                    self.complete_shop_item_receipt(sprite_slot, flags4)
                }
                SpriteMainItemReceiptSuffix::MasterSword => {
                    self.complete_master_sword_item_receipt(sprite_slot)
                }
                SpriteMainItemReceiptSuffix::PotionShopPowder => {
                    self.sprite_slot_view_mut(sprite_slot).set_state(0);
                }
                SpriteMainItemReceiptSuffix::PotionCauldron => {}
                SpriteMainItemReceiptSuffix::SmithyTemperedSword => {
                    self.complete_smithy_tempered_sword_receipt();
                }
                SpriteMainItemReceiptSuffix::HeartPiece => {
                    self.sprite_slot_view_mut(sprite_slot).set_state(0);
                    self.heart_upgrade_set_obtained_flag(sprite_slot);
                }
                SpriteMainItemReceiptSuffix::FluteKidShovel => {
                    self.sprite_slot_view_mut(sprite_slot).set_ai_state(0);
                }
                SpriteMainItemReceiptSuffix::LocksmithChain => {
                    self.save_progress_mut().or_progress_indicator_3(0x10);
                    self.sprite_slot_view_mut(sprite_slot).set_ai_state(4);
                    self.follower_state_mut().set_indicator(0);
                }
            }
            // The receipt returned inside this host but the wire
            // shows neither the Sprite_Main return nor the shared
            // suffix: the remaining slots cross the host boundary
            // (route host 809107: [LatchHeld, Completed,
            // ItemReceipt Returned, SpriteMainProgressed(AfterSlot(15)),
            // CallStackContinued]). Park the loop after the
            // receipt's slot and let the next host finish it.
            let remainder_crosses_host =
                matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                    && self.original_timing_semantic_receipts.is_some()
                    && authoritative_scheduled_caller_return_timeline.is_none()
                    && !self.original_timing_owes_sprite_main_return()
                    && !self.original_timing_hosts_fresh_iteration()
                    && self.original_timing_main_loop_interruption().is_none();
            if let (true, SpriteMainItemReceiptCallerReturn::Module07(dungeon)) =
                (remainder_crosses_host, caller)
            {
                let _ = self.take_original_timing_sprite_main_progress();
                self.active_dungeon_sprite_main_return = Some(dungeon);
                self.game_execution_scheduler.schedule_work(
                    GameWorkContinuation::FinishSpriteMain {
                        boundary: SpriteMainCpuBoundary::AfterSlot(sprite_slot as u8),
                        caller: SpriteMainCpuCaller::DungeonModule07Live {
                            boundary: OriginalTimingBoundary::HostReturn,
                        },
                    },
                    1,
                );
                self.retire_or_defer_main_loop_common_suffix_by_wire();
                return true;
            }
            self.complete_sprite_main_after_interrupted_slot(sprite_slot);
            match caller {
                SpriteMainItemReceiptCallerReturn::Module07(dungeon) => {
                    self.complete_module07_after_sprite_main(dungeon)
                }
                SpriteMainItemReceiptCallerReturn::Module09(module09) => {
                    if self.complete_resumed_module09_sprite_main_caller(
                        module09,
                        input,
                        oam_dma_source.as_deref(),
                    ) {
                        return true;
                    }
                }
            }
            if self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
            {
                return true;
            }
        }
        if let ItemReceiptGraphicsContinuation::ResumeAncillaItemReceipt {
            receipt,
            ancilla_slot,
            suffix,
            caller,
        } = continuation
        {
            // The ancilla's receipt returned: its own tail, the
            // remaining ancilla slots, the prefix tail, the whole
            // descending slot loop, and the module caller follow
            // (route hosts 1142850, 514800).
            self.complete_ancilla_add_item_receipt(receipt);
            self.complete_link_receive_item(receipt.item);
            match suffix {
                AncillaItemReceiptSuffix::None => {}
                AncillaItemReceiptSuffix::ClearAncillaAndModalPause => {
                    self.ancilla_slot_view_mut(usize::from(ancilla_slot))
                        .set_ancilla_type(0);
                    self.clear_modal_pause_flag();
                }
            }
            self.ancilla_execute_slots_below(usize::from(ancilla_slot));
            self.complete_sprite_main_prefix_after_ancilla();
            self.complete_sprite_main_after_interrupted_slot(16);
            match caller {
                SpriteMainItemReceiptCallerReturn::Module07(dungeon) => {
                    self.complete_module07_after_sprite_main(dungeon)
                }
                SpriteMainItemReceiptCallerReturn::Module09(module09) => {
                    if self.complete_resumed_module09_sprite_main_caller(
                        module09,
                        input,
                        oam_dma_source.as_deref(),
                    ) {
                        return true;
                    }
                }
            }
            if self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
            {
                return true;
            }
        }
        // The decompressor has finally returned through
        // Module_MainRouting. Only now can the ROM publish sprite
        // DMA sources and release the software NMI latch.
        if authoritative_scheduled_caller_return_timeline.is_some() {
            self.finish_original_timing_sprite_main_return_claim_scope();
            // The source-proven caller return already carries the
            // shared ZeldaRunGameLoop suffix; retire its one owner.
            self.retire_or_run_main_loop_common_suffix_after_module_return();
        } else {
            // Without a terminal return the wire can hold the
            // caller's shared suffix past this host: the resumed
            // Module07 caller returns, the trailing acceptance
            // stays held, and the next host's continued return
            // completes the suffix (route host 182374). When the
            // resumed caller's Sprite_Main returned inside this
            // host the wire publishes that return here without a
            // terminal return timeline (route host 570353:
            // [NmiAccepted(LatchHeld), NmiHandlerCompleted,
            // SpriteMainReturned, CallStackContinued]); the native
            // caller suffix above owns it.
            let _ = self.take_original_timing_sprite_main_returned();
            self.retire_or_defer_main_loop_common_suffix_by_wire();
        }
        if !ordinary_gfx_21_return && authoritative_scheduled_caller_accepts_nmi_at_return.is_none()
        {
            // The pinned core resumes the $2104 OAM transfer at
            // this completion vblank for both measured receipts
            // (chest $14 at f4587, uncle passage at f9020): the
            // decompressor returns and the epilogue sets
            // nmi_boolean before the vblank, whose DoUpdates then
            // carries the fully prepared post-receipt shadow. The
            // in-arm NMI above ran before that epilogue, so record
            // the law transfer it performs here.
            self.complete_atomic_item_graphics_return_postlude(continuation);
            return true;
        }
        false
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_triforce_room_load(
        &mut self,
        input: u16,
        step: TriforceRoomLoadStep,
        authoritative_scheduled_caller_accepts_nmi_at_return: Option<bool>,
        authoritative_scheduled_caller_nmi_timeline: Option<
            &OriginalTimingMainLoopInterruptionTimeline,
        >,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if authoritative_scheduled_caller_accepts_nmi_at_return.is_none()
            && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
        {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        }
        self.complete_triforce_room_load_step(step);
        if step == TriforceRoomLoadStep::Case7TextInit {
            // The decoder reports the module $0E -> $19 restore
            // inside case 7 as a dialogue close (route host
            // 1557809).
            let _ = self.take_original_timing_dialogue_closed();
        }
        if self.game_execution_scheduler.work_is_pending() {
            // A phased step scheduled its successor (the upload
            // handed over to the overlay load); the iteration
            // stays suspended.
            return true;
        }
        if self.original_timing_main_loop_interruption()
            == Some(crate::MainLoopInterruption::LinkOam)
            || authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                timeline.interruption == crate::MainLoopInterruption::LinkOam
            })
        {
            // Vblank interrupted the module tail's LinkOam_Main
            // after the case-2 loads; the shared suffix crosses
            // to the next host (route host 1557676 -> 1557677).
            let _ = self.take_original_timing_main_loop_interruption_any();
            self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                NmiPrepareSpritesCpuCaller::TriforceRoom,
            );
            return true;
        }
        if matches!(
            self.original_timing_main_loop_interruption(),
            Some(
                crate::MainLoopInterruption::SpritePreparation
                    | crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }
            )
        ) {
            let _ = self.take_original_timing_main_loop_interruption_any();
            self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                NmiPrepareSpritesCpuCaller::TriforceRoom,
            );
            return true;
        }
        if authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
            matches!(
                timeline.interruption,
                crate::MainLoopInterruption::SpritePreparation
                    | crate::MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }
            )
        }) {
            self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                NmiPrepareSpritesCpuCaller::TriforceRoom,
            );
            return true;
        }
        if self.pending_main_loop_common_suffix.is_some() {
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_finish_dungeon_subtile_palette_filter(
        &mut self,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
    ) {
        // ApplyPaletteFilter_bounce returns after the NMI that
        // interrupted its color loop. Resume the caller's optional
        // second filter pass only after that boundary instead of
        // collapsing both passes into one host frame.
        if self.game_state.display.palette_filter.countdown() != 0 {
            self.ApplyPaletteFilter_bounce();
        }
        // The filter interrupted Module07's translated call stack.
        // Resume its ordinary sprite/Link/HUD suffix before
        // NMI_PrepareSprites so LinkOam_Main's new DMA pose is the
        // source consumed by the ensuing NMI. OAM and Link CHR keep
        // independent scanout generations across this boundary.
        if let Some((_, _, sprite_main_return_claims)) =
            authoritative_scheduled_caller_return_timeline.as_ref()
        {
            self.begin_original_timing_sprite_main_return_claim_scope(*sprite_main_return_claims);
        }
        self.complete_module07_dungeon_after_submodule();
        if authoritative_scheduled_caller_return_timeline.is_some() {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }));
        // This resumed caller suffix reaches the next NMI only
        // after the scanout selected here. Keep the animated BG
        // batch on the same completed host-boundary generation;
        // publishing its newly prepared $3b00/$3c00 words here
        // advances dungeon torches by one physical frame.
        self.next_display_animated_bg_scanout_generation =
            Some(AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi);
        if self.pending_main_loop_common_suffix.is_some() {
            // The source-proven caller return already carries the
            // shared ZeldaRunGameLoop suffix; retire its one owner.
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        }
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_straight_interroom_fadeout_suffix(
        &mut self,
        input: u16,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        // The palette walk completed in the preceding main slice,
        // but its Module 7 caller did not return before vblank.
        // Publish that held scanout first, then resume the ordinary
        // dungeon sprite/Link/HUD suffix without starting a second
        // module iteration or ticking the frame counter.
        // NMI skipped OAM DMA while the long caller kept $12 set.
        // Preserve the exact resident table and decoded Link page;
        // the cleared software shadow built by the suspended main
        // slice is not a hardware-published OAM generation.
        self.next_display_obj_memory_generation = Some(DisplayObjGeneration::RetainCapturedOam {
            oam: self.ppu.oam.clone(),
        });
        self.set_next_display_obj_scanout(Some(straight_interroom_fadeout_following_obj_scanout()));
        self.capture_display_snapshot();
        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        self.complete_module07_dungeon_after_submodule();
        self.nmi_prepare_sprites();
        self.clear_nmi_update_latch();
        // The resumed suffix authored this shadow after the held
        // frame's vblank. The display boundary now selects the
        // effective OAM DMA generation from scheduler events, so
        // both straight-interroom callers use the same cadence.
        self.next_display_obj_memory_generation = Some(DisplayObjGeneration::RetainCapturedOam {
            oam: self.ppu.oam.clone(),
        });
        self.set_next_display_obj_scanout(Some(straight_interroom_fadeout_return_obj_scanout()));
        // The saved C caller has now returned through LinkOam,
        // both HUD calls, NMI_PrepareSprites, and `$12 = 0`.
        // If the ordered source timeline accepted the following
        // NMI at this host return, its publication belongs to the
        // next host just like every other early-returning
        // scheduled caller.
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_sprite_main_work(
        &mut self,
        input: u16,
        boundary: SpriteMainCpuBoundary,
        caller: SpriteMainCpuCaller,
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
    ) -> bool {
        assert!(matches!(
            caller,
            SpriteMainCpuCaller::DungeonModule07
                | SpriteMainCpuCaller::DungeonModule07Live { .. }
                | SpriteMainCpuCaller::Module09 { .. }
                | SpriteMainCpuCaller::BossVictory { .. }
                | SpriteMainCpuCaller::SaveAndQuit { .. }
        ));
        if crate::debug_env::var_os("ZELDA3_DEBUG_DUNGEON_CPU_SCHEDULE").is_some() {
            eprintln!(
                    "dungeon_sprite_main_cpu_complete host={} boundary={boundary:?} active_return={} state={:02x}/{:02x}/{:02x}",
                    self.frame_ctr_dbg,
                    self.active_dungeon_sprite_main_return.is_some(),
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule,
                    self.game_state.frame.subsubmodule,
                );
        }
        // When the wire owns this terminal return, the count of
        // SpriteMainReturned facts it published this host bounds
        // the native slot-zero crossings below. A zero count means
        // the wire's return receipt landed on the acceptance host;
        // the native crossing then completes without a this-host
        // claim to consume.
        let scheduled_return_sprite_main_claims = authoritative_scheduled_caller_return_timeline
            .as_ref()
            .map(|(_, _, claims)| *claims)
            .filter(|claims| *claims > 0);
        // A scheduled dungeon Sprite_Main whose host is neither
        // terminal nor bare: the wire publishes the return inside
        // this host and then interrupts the module tail or holds
        // (route host 1206045: [LatchHeld, Completed,
        // SpriteMainReturned, LatchHeld,
        // MainLoopInterrupted(SpritePreparation), Continued]).
        let live_nonterminal_dungeon = matches!(
            caller,
            SpriteMainCpuCaller::DungeonModule07 | SpriteMainCpuCaller::DungeonModule07Live { .. }
        ) && matches!(
            self.original_timing_owner,
            OriginalTimingOwnerState::Live
        ) && self.original_timing_semantic_receipts.is_some()
            && authoritative_scheduled_caller_return_timeline.is_none();
        let nonterminal_dungeon_sprite_main_claim = live_nonterminal_dungeon
            && scheduled_return_sprite_main_claims.is_none()
            && self.original_timing_owes_sprite_main_return();
        if let Some(claims) = scheduled_return_sprite_main_claims {
            self.begin_original_timing_sprite_main_return_claim_scope(claims);
        } else if nonterminal_dungeon_sprite_main_claim {
            self.begin_original_timing_sprite_main_return_claim_scope(1);
        }
        if matches!(caller, SpriteMainCpuCaller::BossVictory { .. }) {
            self.complete_victory_module_dialogue_scroll_before_sprite_main();
        }
        if authoritative_scheduled_caller_return_timeline.is_some() {
            // The wire returns the parked loop, its module tail and
            // the shared suffix inside this host (route host
            // 1452888); an estimated room-load suffix slice would
            // suspend the tail again and strand the return claim.
            self.dungeon_room_load_module_suffix_nmi_slices = 0;
        }
        self.complete_sprite_main_after_cpu_boundary(boundary);
        if nonterminal_dungeon_sprite_main_claim {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if !self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            // A lower sprite can begin another measured operation
            // while this Sprite_Main suffix is resuming. That nested
            // continuation owns the saved Module 7 return state;
            // only an idle scheduler permits this outer caller to
            // consume and complete it here.
            match caller {
                SpriteMainCpuCaller::DungeonModule07 => {
                    let sprite_return = self
                        .active_dungeon_sprite_main_return
                        .take()
                        .expect("Sprite_Main continuation must retain Module 7 return state");
                    self.complete_module07_after_sprite_main(sprite_return);
                    if live_nonterminal_dungeon {
                        self.complete_parked_dungeon_sprite_main_suffix_by_wire(true);
                        self.game_execution_scheduler
                            .finish_call_stack_at_main_wait_before_nmi();
                        self.stage_resumed_sprite_main_return_obj_scanout();
                        return true;
                    }
                }
                SpriteMainCpuCaller::DungeonModule07Live { .. } => {
                    let sprite_return = self
                        .active_dungeon_sprite_main_return
                        .take()
                        .expect("live Sprite_Main continuation must retain Module 7 return state");
                    self.complete_module07_after_sprite_main(sprite_return);
                    if live_nonterminal_dungeon {
                        self.complete_parked_dungeon_sprite_main_suffix_by_wire(true);
                        self.game_execution_scheduler
                            .finish_call_stack_at_main_wait_before_nmi();
                        self.stage_resumed_sprite_main_return_obj_scanout();
                        return true;
                    }
                }
                SpriteMainCpuCaller::Module09 { .. } => {
                    let module09 = self
                        .active_module09_sprite_main_return
                        .take()
                        .expect("live Sprite_Main continuation must retain Module09 return state");
                    let live_nonterminal =
                        matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                            && authoritative_scheduled_caller_return_timeline.is_none();
                    let suffix_interruption = live_nonterminal
                        .then_some(authoritative_scheduled_caller_nmi_timeline)
                        .flatten()
                        .map(|timeline| timeline.interruption);
                    if suffix_interruption == Some(crate::MainLoopInterruption::LinkOam) {
                        // The resumed caller's Sprite_Main returned and
                        // the wire interrupted LinkOam_Main (route host
                        // 321896): resume the LinkOam/HUD/rain suffix on
                        // the next host, as the fresh caller does.
                        if scheduled_return_sprite_main_claims.is_some() {
                            self.finish_original_timing_sprite_main_return_claim_scope();
                        }
                        let continuation =
                            GameWorkContinuation::FinishModule09LinkOamCallerReturn {
                                caller: module09,
                            };
                        if authoritative_scheduled_caller_accepts_nmi_at_return.unwrap_or(false) {
                            self.game_execution_scheduler
                                .schedule_after_current_trailing_nmi(continuation);
                        } else {
                            self.game_execution_scheduler.schedule_work(continuation, 1);
                        }
                        return true;
                    }
                    if self.complete_resumed_module09_sprite_main_caller_with_wire(
                        module09,
                        input,
                        oam_dma_source.as_deref(),
                        authoritative_scheduled_caller_return_timeline.is_some(),
                    ) {
                        if scheduled_return_sprite_main_claims.is_some() {
                            self.finish_original_timing_sprite_main_return_claim_scope();
                        }
                        return true;
                    }
                    if suffix_interruption == Some(crate::MainLoopInterruption::SpritePreparation) {
                        // The caller suffix ran through LinkOam and the
                        // wire interrupted NMI_PrepareSprites (route
                        // host 202669).
                        if scheduled_return_sprite_main_claims.is_some() {
                            self.finish_original_timing_sprite_main_return_claim_scope();
                        }
                        self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                            NmiPrepareSpritesCpuCaller::Module09LongLoad,
                        );
                        return true;
                    }
                    if live_nonterminal {
                        // Without a terminal return the wire holds the
                        // shared suffix past this host; the next
                        // host's continued return completes it.
                        if scheduled_return_sprite_main_claims.is_some() {
                            self.finish_original_timing_sprite_main_return_claim_scope();
                        }
                        self.retire_or_defer_main_loop_common_suffix_by_wire();
                        return true;
                    }
                }
                SpriteMainCpuCaller::BossVictory { .. }
                | SpriteMainCpuCaller::SaveAndQuit { .. } => {
                    // The wire-proven terminal return owns the
                    // leading handler and trailing publication
                    // through the shared lane; only the
                    // Module13/Module17 Link/OAM suffix runs
                    // here (route hosts 104004, 159333).
                    self.link_oam_main();
                }
                _ => unreachable!(
                    "this completion arm accepts dungeon, Module09, and boss-victory callers"
                ),
            }
            if scheduled_return_sprite_main_claims.is_some() {
                self.finish_original_timing_sprite_main_return_claim_scope();
            }
            if self.pending_main_loop_common_suffix.is_some() {
                // The source-proven caller return already carries
                // the shared ZeldaRunGameLoop suffix; retire its
                // one owner instead of repeating the raw pair.
                self.complete_pending_main_loop_common_suffix_after_module_return();
            } else {
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
            }
            self.game_execution_scheduler
                .finish_call_stack_at_main_wait_before_nmi();
            self.stage_resumed_sprite_main_return_obj_scanout();
            // The generic scheduled-work tail below publishes the
            // following NMI after this resumed caller returns to the
            // wait loop. That is the same trailing boundary the ROM
            // reaches later in this retro_run, so the next host may
            // begin a fresh main-loop iteration without inserting a
            // second synthetic leading NMI.
        }
        false
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_finish_dungeon_exit_spotlight_entry(
        &mut self,
        mut table_build: SpotlightTableBuildContinuation,
        iteration: SpotlightIteration,
        authoritative_dungeon_exit_spotlight_entry_iteration_returned: bool,
        authoritative_scheduled_caller_nmi_timeline: Option<
            &OriginalTimingMainLoopInterruptionTimeline,
        >,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
    ) {
        // The vblank-interrupted first IrisSpotlight_ConfigureTable
        // build finishes here: table copy, first radius write,
        // submodule advance, and the Link/OAM suffix run, then the
        // scheduled iteration return owns the boundary that reaches
        // Main_PrepSpritesForNmi (oracle $0c00d trace: no prep on
        // the resumed-build frame, one at the following return).
        // The frame counter was ticked by the entry frame's main
        // prefix; this resumed slice must not tick it again.
        if let Some(claim) = self.take_original_timing_spotlight_table_build_progress() {
            assert_eq!(
                claim.boundary,
                crate::OriginalTimingBoundary::NmiAccepted,
                "a dungeon-exit entry re-checkpoint must ride its accepting NMI",
            );
            table_build.assert_recheckpoint_not_behind(claim.progress);
            if table_build.source_progress != Some(claim.progress) {
                table_build = self.begin_iris_spotlight_configure_table_at_progress(claim.progress);
            }
        }
        let link_position_interruption = authoritative_scheduled_caller_nmi_timeline
            .map(|timeline| timeline.interruption)
            .filter(|interruption| {
                matches!(
                    interruption,
                    crate::MainLoopInterruption::LinkActualVelocity { .. }
                        | crate::MainLoopInterruption::LinkActualVelocityCompleted
                        | crate::MainLoopInterruption::LinkVelocityClearProgress { .. }
                        | crate::MainLoopInterruption::DungeonExitSpotlightTableCompleted
                        | crate::MainLoopInterruption::LinkPositionAfterSubpixel { .. }
                        | crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { .. }
                        | crate::MainLoopInterruption::LinkPositionAfterCoordinates { .. }
                )
            });
        match link_position_interruption {
            Some(crate::MainLoopInterruption::LinkActualVelocity {
                horizontal_resolved,
            }) => {
                self.complete_dungeon_exit_spotlight_entry_until_link_actual_velocity(
                    table_build,
                    iteration,
                    horizontal_resolved,
                );
            }
            Some(crate::MainLoopInterruption::LinkActualVelocityCompleted) => {
                self.complete_dungeon_exit_spotlight_entry_until_link_actual_velocity(
                    table_build,
                    iteration,
                    LinkActualVelocityCheckpoint::AfterBoth,
                );
            }
            Some(crate::MainLoopInterruption::LinkVelocityClearProgress { completed }) => {
                self.complete_dungeon_exit_spotlight_entry_until_link_actual_velocity(
                    table_build,
                    iteration,
                    LinkActualVelocityCheckpoint::Clearing { completed },
                );
            }
            Some(crate::MainLoopInterruption::LinkPositionAfterSubpixel { pass }) => {
                self.complete_dungeon_exit_spotlight_entry_until_link_position_partial(
                    table_build,
                    iteration,
                    pass,
                );
            }
            Some(crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { pass }) => {
                self.complete_dungeon_exit_spotlight_entry_until_link_position_after_coordinate_low(
                        table_build,
                        iteration,
                        pass,
                    );
            }
            Some(crate::MainLoopInterruption::LinkPositionAfterCoordinates { pass }) => {
                self.complete_dungeon_exit_spotlight_entry_until_link_position_after_coordinates(
                    table_build,
                    iteration,
                    pass,
                );
            }
            None => {
                if authoritative_dungeon_exit_spotlight_entry_iteration_returned {
                    self.complete_dungeon_exit_spotlight_entry_returned(table_build, iteration);
                } else {
                    self.complete_dungeon_exit_spotlight_entry(table_build, iteration);
                }
            }
            _ => unreachable!("filtered entry Link interruption changed"),
        }
        if authoritative_scheduled_caller_return_timeline.is_some()
            || authoritative_dungeon_exit_spotlight_entry_iteration_returned
        {
            // The wire's terminal return proves the shared
            // ZeldaRunGameLoop suffix completes inside this host
            // (route host 39630).
            self.retire_or_run_main_loop_common_suffix_after_module_return();
        }
        self.set_next_display_obj_scanout(Some(dungeon_exit_spotlight_entry_return_obj_scanout()));
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_finish_dungeon_exit_spotlight_build(
        &mut self,
        table_build: SpotlightTableBuildContinuation,
        projection_completed: bool,
        iteration: SpotlightIteration,
        authoritative_dungeon_exit_spotlight_caller_returned: bool,
        authoritative_dungeon_exit_spotlight_link_oam_interruption: bool,
        authoritative_dungeon_exit_spotlight_same_host_iteration: bool,
        authoritative_scheduled_caller_interrupted_fresh_iteration: bool,
        prospective_spotlight_build_link_oam_plan: Option<OriginalTimingSpotlightBuildLinkOamPlan>,
    ) {
        let link_position_interruption = prospective_spotlight_build_link_oam_plan
            .as_ref()
            .map(|plan| plan.interruption)
            .filter(|interruption| {
                matches!(
                    interruption,
                    crate::MainLoopInterruption::LinkActualVelocity { .. }
                        | crate::MainLoopInterruption::LinkActualVelocityCompleted
                        | crate::MainLoopInterruption::LinkVelocityClearProgress { .. }
                        | crate::MainLoopInterruption::DungeonExitSpotlightTableCompleted
                        | crate::MainLoopInterruption::LinkPositionAfterSubpixel { .. }
                        | crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { .. }
                        | crate::MainLoopInterruption::LinkPositionAfterCoordinates { .. }
                )
            });
        if let Some(interruption) = link_position_interruption {
            // The host ended inside Link_MovePosition after the
            // completed build (route host 179586): consume that
            // boundary and suspend the movement mid-loop.
            // The interruption timeline taken above already
            // consumed the receipt; retire any restatement.
            let _ = self.take_original_timing_main_loop_interruption(interruption);
            assert!(
                    !authoritative_dungeon_exit_spotlight_caller_returned
                        && !authoritative_dungeon_exit_spotlight_link_oam_interruption,
                    "a mid-loop Link position boundary cannot share its host with a caller return or LinkOam interruption",
                );
            match interruption {
                crate::MainLoopInterruption::LinkActualVelocity {
                    horizontal_resolved,
                } => self.complete_dungeon_exit_spotlight_build_until_link_actual_velocity(
                    table_build,
                    projection_completed,
                    iteration,
                    horizontal_resolved,
                ),
                crate::MainLoopInterruption::DungeonExitSpotlightTableCompleted => self
                    .complete_dungeon_exit_spotlight_build_until_control(
                        table_build,
                        projection_completed,
                        iteration,
                    ),
                crate::MainLoopInterruption::LinkActualVelocityCompleted => self
                    .complete_dungeon_exit_spotlight_build_until_link_actual_velocity(
                        table_build,
                        projection_completed,
                        iteration,
                        LinkActualVelocityCheckpoint::AfterBoth,
                    ),
                crate::MainLoopInterruption::LinkVelocityClearProgress { completed } => self
                    .complete_dungeon_exit_spotlight_build_until_link_actual_velocity(
                        table_build,
                        projection_completed,
                        iteration,
                        LinkActualVelocityCheckpoint::Clearing { completed },
                    ),
                crate::MainLoopInterruption::LinkPositionAfterSubpixel { pass } => self
                    .complete_dungeon_exit_spotlight_build_until_link_position_partial(
                        table_build,
                        projection_completed,
                        iteration,
                        pass,
                    ),
                crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { pass } => self
                    .complete_dungeon_exit_spotlight_build_until_link_position_after_coordinate_low(
                        table_build,
                        projection_completed,
                        iteration,
                        pass,
                    ),
                crate::MainLoopInterruption::LinkPositionAfterCoordinates { pass } => self
                    .complete_dungeon_exit_spotlight_build_until_link_position_after_coordinates(
                        table_build,
                        projection_completed,
                        iteration,
                        pass,
                    ),
                _ => unreachable!("filtered Link position interruption changed"),
            }
        } else {
            if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                && !authoritative_dungeon_exit_spotlight_same_host_iteration
            {
                // The helper NMI accepted at this host's entry
                // re-checkpointed the saved build; the continuation
                // already carries that statement (route host
                // 207630). A same-host fresh iteration's own
                // host-return checkpoint belongs to Module0F.
                if let Some(claim) = self.take_original_timing_spotlight_table_build_progress() {
                    assert_eq!(
                        claim.boundary,
                        crate::OriginalTimingBoundary::NmiAccepted,
                        "a dungeon-exit Build re-checkpoint must ride an accepting NMI",
                    );
                }
            }
            self.complete_dungeon_exit_spotlight_build(
                table_build,
                projection_completed,
                iteration,
                authoritative_dungeon_exit_spotlight_caller_returned,
                authoritative_dungeon_exit_spotlight_link_oam_interruption,
            );
        }
        if let Some(plan) = prospective_spotlight_build_link_oam_plan.as_ref() {
            assert_eq!(
                self.game_execution_scheduler, plan.scheduler_after_cpu,
                "spotlight Build-LinkOam CPU callback changed its probed successor",
            );
        }
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        )));
        if authoritative_dungeon_exit_spotlight_same_host_iteration
            && !authoritative_scheduled_caller_interrupted_fresh_iteration
        {
            // A fresh ZeldaRunGameLoop entry is source-level proof
            // that the suspended Module0F caller and its shared
            // Main_PrepSpritesForNmi suffix returned first. Retire
            // that caller once, then execute the newly observed
            // iteration in this same host interval. Returning from
            // the scheduled-work path here would discard the
            // authority's iteration and leave native gameplay one
            // main-loop tick behind.
            self.game_execution_scheduler
                .prepare_same_host_main_iteration_after_authoritative_return();
            self.zelda_run_game_loop_with_progress(Some(crate::MainLoopProgress::IterationStarted));
            self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
        }
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_finish_overworld_spotlight_build(
        &mut self,
        table_build: SpotlightTableBuildContinuation,
        phase: OverworldSpotlightBuildPhase,
        projection_completed: bool,
        iteration: SpotlightIteration,
        authoritative_overworld_spotlight_same_host_iteration: bool,
        authoritative_scheduled_caller_interrupted_fresh_iteration: bool,
        authoritative_scheduled_caller_nmi_timeline: Option<
            &OriginalTimingMainLoopInterruptionTimeline,
        >,
    ) {
        // The wire may interrupt the resumed build's goal
        // transition inside IrisSpotlight_ResetTable (route host
        // 182709); the timeline above already consumed that
        // boundary, so hand its store cursor to the native body.
        let reset_table_interruption =
            authoritative_scheduled_caller_nmi_timeline.and_then(|timeline| {
                match timeline.interruption {
                    crate::MainLoopInterruption::SpotlightGoalResetTable { completed_stores } => {
                        Some(completed_stores)
                    }
                    _ => None,
                }
            });
        if reset_table_interruption.is_some() {
            // The interrupting acceptance at this host's entry
            // re-checkpointed the saved build; the continuation
            // already carries the same statement.
            if let Some(claim) = self.take_original_timing_spotlight_table_build_progress() {
                assert_eq!(
                    claim.boundary,
                    crate::OriginalTimingBoundary::NmiAccepted,
                    "a spotlight build re-checkpoint must ride an accepting NMI",
                );
            }
        }
        let suspended = self.complete_overworld_spotlight_build(
            table_build,
            phase,
            projection_completed,
            iteration,
            reset_table_interruption,
        );
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        )));
        if suspended {
            assert!(
                    !authoritative_overworld_spotlight_same_host_iteration,
                    "a goal transition suspended inside IrisSpotlight_ResetTable cannot share its host with a fresh iteration",
                );
        } else if authoritative_overworld_spotlight_same_host_iteration
            && !authoritative_scheduled_caller_interrupted_fresh_iteration
        {
            // A new ZeldaRunGameLoop entry proves that the saved
            // Module10 caller returned before this host's fresh
            // iteration. Complete the native shadow once, then run
            // the source-observed iteration without dropping its
            // table-progress receipt.
            self.game_execution_scheduler
                .prepare_same_host_main_iteration_after_authoritative_return();
            self.zelda_run_game_loop_with_progress(Some(crate::MainLoopProgress::IterationStarted));
            self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
        }
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_finish_game_over_spotlight_build(
        &mut self,
        mut table_build: SpotlightTableBuildContinuation,
        entry: bool,
        iteration: SpotlightIteration,
        authoritative_scheduled_caller_nmi_timeline: Option<
            &OriginalTimingMainLoopInterruptionTimeline,
        >,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
    ) {
        // Game Over's recurring circle build itself crosses this
        // vblank. Finish the table now, then keep its caller return
        // as a second resumable slice so Link/OAM publication lands
        // on the same third host frame as the 65816 routine.
        // The suffix receipt is already consumed by the time this
        // arm runs; the scheduled caller's timelines carry the
        // same fact: an in-host LinkOam/NMI_PrepareSprites
        // interruption without a terminal return means the caller
        // returns on the next host (route host 589119).
        let wire_defers_caller_return = authoritative_scheduled_caller_return_timeline.is_none()
            && authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                timeline.progress == crate::MainLoopProgress::CallStackContinued
                    && matches!(
                        timeline.interruption,
                        crate::MainLoopInterruption::LinkOam
                            | crate::MainLoopInterruption::SpritePreparation
                    )
            });
        if let Some(claim) = self.take_original_timing_spotlight_table_build_progress() {
            assert_eq!(
                claim.boundary,
                OriginalTimingBoundary::NmiAccepted,
                "a game-over Build re-checkpoint must ride an accepting NMI",
            );
            if table_build.source_progress.is_some() {
                table_build.assert_recheckpoint_not_behind(claim.progress);
            }
            if table_build.source_progress != Some(claim.progress) {
                // The source can execute more row-loop or
                // projection statements between its host-return
                // checkpoint and the accepting NMI. Rebuild at
                // that later statement so every intervening store
                // becomes live before the interrupted call resumes.
                table_build = self.begin_iris_spotlight_configure_table_at_progress(claim.progress);
            }
        }
        self.complete_game_over_spotlight_build(
            table_build,
            entry,
            iteration,
            wire_defers_caller_return,
            authoritative_scheduled_caller_return_timeline.is_some(),
        );
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    pub(super) fn lane_finish_spotlight_iteration(
        &mut self,
        authoritative_dungeon_exit_spotlight_same_host_iteration: bool,
        authoritative_scheduled_caller_interrupted_fresh_iteration: bool,
    ) {
        // The opening or closing iris has returned through
        // LinkOam_Main and the normal game-loop suffix only after
        // this scanout has started. Its HDMA table can already be
        // visible, but OAM and Link OBJ CHR still belong to the
        // host-boundary generation; the NMI below publishes their
        // newly prepared sources for the following scanout.
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        )));
        // A deferred return completes the same ZeldaRunGameLoop
        // caller. Its sprite-preparation suffix runs exactly once:
        // the isolated ROM CPU plan may have proved it returned in
        // the prior slice, otherwise it returns here. A live host
        // whose wire publishes no suffix completion proves the
        // suffix already ran in the prior slice; repeating the
        // latch clear here would drop the update latch the new
        // iteration holds across its suspended spotlight build
        // (route host 40979).
        let live_suffix_ran_in_prior_slice = matches!(
                self.original_timing_owner,
                OriginalTimingOwnerState::Live
            ) && self
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| {
                    !receipts.semantic().iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                                | OriginalTimingSemanticReceipt::NmiAccepted(
                                    NmiUpdateGate::Open,
                                )
                        )
                    })
                })
                // An installed Open gate likewise proves the suffix's
                // latch clear runs inside this host.
                && !self
                    .original_timing_expected_nmi_update_gates
                    .contains(&NmiUpdateGate::Open);
        if !live_suffix_ran_in_prior_slice {
            self.nmi_prepare_sprites_for_main_loop_once();
            self.clear_nmi_update_latch();
        }
        if authoritative_dungeon_exit_spotlight_same_host_iteration
            && !authoritative_scheduled_caller_interrupted_fresh_iteration
        {
            // Snes9x observed the prior suspended caller return and
            // a new ZeldaRunGameLoop entry before this host's
            // trailing boundary. This is a semantic call-order
            // receipt, not a route or program-counter condition.
            self.game_execution_scheduler
                .prepare_same_host_main_iteration_after_authoritative_return();
            let same_host_sprite_main_claims = self
                .original_timing_semantic_receipts
                .as_ref()
                .map(|receipts| {
                    receipts
                        .semantic()
                        .iter()
                        .filter(|receipt| {
                            **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                        })
                        .count()
                })
                .unwrap_or(0);
            if same_host_sprite_main_claims != 0 {
                self.begin_original_timing_sprite_main_return_claim_scope(
                    same_host_sprite_main_claims,
                );
            }
            self.zelda_run_game_loop_with_progress(Some(crate::MainLoopProgress::IterationStarted));
            if same_host_sprite_main_claims != 0
                && self
                    .original_timing_sprite_main_return_claims_remaining
                    .is_some()
            {
                self.finish_original_timing_sprite_main_return_claim_scope();
            }
            self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
        }
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_overworld_load_overlays_sprite_reload(
        &mut self,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
    ) -> bool {
        // `$09:c4aa` returned from the concrete sprite-generation
        // call. Publish the remaining staged slots, execute only
        // the source statements between that RTL and the overlay
        // decoder, then keep the enclosing call suspended.
        let special_area_returned = self.complete_overworld_load_overlays_after_sprite_reload();
        if !special_area_returned {
            assert!(
                authoritative_scheduled_caller_return_timeline.is_none(),
                "Overworld_LoadOverlays cannot return before its overlay decoder",
            );
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishOverworldLoadOverlaysOverlay,
                OVERWORLD_LOAD_OVERLAYS_OVERLAY_NMI_SLICES,
            );
            return true;
        }

        let sprite_main_return_claims = authoritative_scheduled_caller_return_timeline
            .as_ref()
            .map(|(_, _, claims)| *claims);
        if let Some(claims) = sprite_main_return_claims {
            self.begin_original_timing_sprite_main_return_claim_scope(claims);
        }
        self.complete_module09_overworld_after_submodule();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            assert!(
                authoritative_scheduled_caller_return_timeline.is_none(),
                "a suspended special-area caller cannot also own a terminal return",
            );
            return true;
        }
        if sprite_main_return_claims.is_some() {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && authoritative_scheduled_caller_return_timeline.is_none()
        {
            self.retire_or_defer_main_loop_common_suffix_by_wire();
        } else {
            self.retire_or_run_main_loop_common_suffix_after_module_return();
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_overworld_load_overlays_overlay(
        &mut self,
        input: u16,
        authoritative_scheduled_caller_accepts_nmi_at_return: Option<bool>,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        }
        let continued_sprite_main_claims =
            if authoritative_scheduled_caller_return_timeline.is_none() {
                self.original_timing_semantic_receipts
                    .as_ref()
                    .map(|receipts| {
                        receipts
                            .semantic()
                            .iter()
                            .filter(|receipt| {
                                **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                            })
                            .count()
                    })
                    .unwrap_or(0)
            } else {
                0
            };
        let sprite_main_return_claims = authoritative_scheduled_caller_return_timeline
            .as_ref()
            .map(|(_, _, claims)| *claims)
            .or((continued_sprite_main_claims != 0).then_some(continued_sprite_main_claims));
        if let Some(claims) = sprite_main_return_claims {
            self.begin_original_timing_sprite_main_return_claim_scope(claims);
        }
        self.finish_overworld_load_overlays();
        self.complete_module09_overworld_after_submodule();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            assert!(
                authoritative_scheduled_caller_return_timeline.is_none(),
                "a suspended Overworld_LoadOverlays caller cannot own a terminal return",
            );
            return true;
        }
        if sprite_main_return_claims.is_some() {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && authoritative_scheduled_caller_return_timeline.is_none()
        {
            self.retire_or_defer_main_loop_common_suffix_by_wire();
        } else {
            self.retire_or_run_main_loop_common_suffix_after_module_return();
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_world_map_overlay_reload(
        &mut self,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
    ) -> bool {
        let schedule = self
            .module09_cpu_schedule
            .take()
            .expect("Module09/$20 completion lost its ROM CPU schedule");
        // The live wire may re-state the resumed caller's exact
        // Sprite_Main boundary; it must agree with the ROM
        // schedule's saved plan before either is executed.
        let claimed_sprite_main_boundary = self.take_original_timing_sprite_main_progress();
        self.finish_overworld_load_overlays();
        assert_eq!(
            schedule.caller_nmis,
            schedule.caller_sprite_main_nmis + schedule.caller_suffix_nmis,
        );
        if let Some((_, _, sprite_main_return_claims)) =
            authoritative_scheduled_caller_return_timeline.as_ref()
        {
            // The wire-proven terminal return supersedes the ROM
            // schedule's remaining NMI budget: the resumed caller,
            // its one Sprite_Main, and the shared suffix all
            // complete inside this host (route host 61935).
            assert_eq!(
                    claimed_sprite_main_boundary, None,
                    "a terminal overlay-reload return cannot also claim a mid-body Sprite_Main boundary",
                );
            self.begin_original_timing_sprite_main_return_claim_scope(*sprite_main_return_claims);
            self.complete_module09_sprite_and_hud_suffix();
            self.finish_original_timing_sprite_main_return_claim_scope();
            self.retire_or_run_main_loop_common_suffix_after_module_return();
            return true;
        }
        if schedule.caller_sprite_main_nmis != 0 {
            assert_eq!(schedule.caller_suffix_nmis, 0);
            assert_eq!(
                schedule.caller_first_nmi_phase,
                Some(ModuleCpuPhase::InterruptedInSpriteMain),
            );
            let boundary = schedule
                .sprite_main_boundary
                .expect("Module09/$20 ROM plan must name its Sprite_Main boundary");
            // The wire interruption/progress pair is the source's
            // own Sprite_Main boundary. The ROM schedule's saved
            // slot is a replaceable estimate of the same fact and
            // is superseded when the live claim is present.
            let boundary = claimed_sprite_main_boundary.unwrap_or(boundary);
            self.begin_world_map_overlay_module09_sprite_return(
                boundary,
                schedule.caller_sprite_main_nmis,
            );
        } else if schedule.caller_suffix_nmis == 0 {
            assert_eq!(
                    claimed_sprite_main_boundary, None,
                    "a Module09/$20 caller without a Sprite_Main phase cannot claim Sprite_Main progress",
                );
            assert_eq!(schedule.caller_nmis, 0);
            self.complete_module09_sprite_and_hud_suffix();
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        } else {
            assert_eq!(
                claimed_sprite_main_boundary, None,
                "a Module09/$20 suffix-phase caller cannot claim Sprite_Main progress",
            );
            assert_eq!(schedule.caller_nmis, schedule.caller_suffix_nmis);
            assert_eq!(
                schedule.caller_suffix_nmis, 1,
                "Module09/$20 caller suffix crossed more than one NMI",
            );
            let authoritative_phase =
                self.take_authoritative_module09_caller_phase(schedule.caller_first_nmi_phase);
            match authoritative_phase {
                ModuleCpuPhase::InterruptedInLinkOam => {
                    // C returned from Overworld_LoadOverlays2 and
                    // completed Sprite_Main before vblank entered
                    // LinkOam_Main. Preserve Module09's four live
                    // scroll locals and resume only Link OAM, HUD,
                    // rain, and the shared main-loop suffix.
                    let module09 = self.begin_module09_sprite_main();
                    self.sprite_main();
                    assert!(
                        !self
                            .game_execution_scheduler
                            .work_suspends_translated_call_stack(),
                        "Module09 LinkOam receipt cannot also interrupt Sprite_Main",
                    );
                    self.game_execution_scheduler
                        .schedule_cpu_timed_work_resuming_after_current_trailing_nmi(
                            GameWorkContinuation::FinishModule09LinkOamCallerReturn {
                                caller: Module09ItemReceiptCallerReturn {
                                    link_oam: None,
                                    scroll: module09,
                                    rain_already_published: false,
                                    after_sprite_main: Module09AfterSpriteMain::Ordinary,
                                },
                            },
                            schedule.caller_suffix_nmis,
                        );
                }
                ModuleCpuPhase::InterruptedInNmiPrepareSprites => {
                    // The source completed Module09's
                    // Sprite_Main/LinkOam/HUD/rain suffix and was
                    // interrupted only after entering the common
                    // NMI_PrepareSprites loop. Execute that exact C
                    // prefix once, then resume only the interrupted
                    // common suffix on the following host boundary.
                    self.complete_module09_sprite_and_hud_suffix();
                    self.interrupted_nmi_prepare_obj_cache_vram = Some(
                        self.last_presented_obj_vram
                            .as_ref()
                            .cloned()
                            .or_else(|| self.ppu.obj_vram_latch.clone())
                            .unwrap_or_else(|| self.ppu.vram.clone()),
                    );
                    self.game_execution_scheduler
                        .schedule_cpu_timed_work_resuming_after_current_trailing_nmi(
                            GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn {
                                caller: NmiPrepareSpritesCpuCaller::WorldMapOverlayReload,
                            },
                            schedule.caller_suffix_nmis,
                        );
                }
                phase => {
                    panic!("Module09/$20 caller crossed an unsupported semantic phase: {phase:?}")
                }
            }
        }
        false
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_overworld_aux_graphics(
        &mut self,
        input: u16,
        work: GameWorkContinuation,
        authoritative_overworld_special_exit_mosaic_restored: bool,
        authoritative_overworld_special_exit_mosaic_returned: bool,
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
    ) -> bool {
        // PC/V-counter traces remain in the active Module09
        // graphics conversion through this frame's vblank,
        // returning to its module caller immediately afterward.
        // Preserve that ordering: this scanout uses the pre-load
        // display, while the completed graphics and caller suffix
        // become CPU-visible before the next frame.
        //
        // The interrupt's scroll-register writes occur at this
        // vblank and are visible in the scanout even though the
        // decompressed VRAM generation remains the captured one.
        if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        }
        let continued_sprite_main_claims =
            if authoritative_scheduled_caller_return_timeline.is_none() {
                self.original_timing_semantic_receipts
                    .as_ref()
                    .map(|receipts| {
                        receipts
                            .semantic()
                            .iter()
                            .filter(|receipt| {
                                **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                            })
                            .count()
                    })
                    .unwrap_or(0)
            } else {
                0
            };
        if let Some((_, _, sprite_main_return_claims)) =
            authoritative_scheduled_caller_return_timeline.as_ref()
        {
            self.begin_original_timing_sprite_main_return_claim_scope(*sprite_main_return_claims);
        } else if continued_sprite_main_claims != 0 {
            // The continued caller's own Sprite_Main returns
            // inside this host even though the iteration suspends
            // later (in LinkOam at route host 69140).
            self.begin_original_timing_sprite_main_return_claim_scope(continued_sprite_main_claims);
        }
        let caller_remains_suspended = match work {
            GameWorkContinuation::FinishOverworldAuxGraphics => {
                self.complete_module09_load_aux_gfx();
                false
            }
            GameWorkContinuation::FinishOverworldMosaicSpriteGraphics => {
                self.complete_overworld_mosaic_sprite_graphics();
                false
            }
            GameWorkContinuation::FinishOverworldSpecialExitMosaic => {
                let special_exit_caller = self.game_state.frame.main_module == 0x0b
                    && self.game_state.frame.submodule == 0x24;
                if !special_exit_caller || authoritative_overworld_special_exit_mosaic_returned {
                    self.complete_overworld_start_mosaic_after_animated_sprite_tile();
                    false
                } else {
                    assert!(
                            authoritative_overworld_special_exit_mosaic_restored,
                            "special-exit mosaic first decode completed without a source restore checkpoint",
                        );
                    assert!(
                            self.publish_overworld_special_exit_mosaic_restore_prefix(),
                            "special-exit mosaic restore checkpoint did not enter its source second decode",
                        );
                    self.game_execution_scheduler.schedule_work(
                        GameWorkContinuation::FinishOverworldSpecialExitMosaicSecondDecode,
                        4,
                    );
                    true
                }
            }
            GameWorkContinuation::FinishOverworldSpecialExitMosaicSecondDecode => {
                assert!(
                    authoritative_overworld_special_exit_mosaic_returned,
                    "special-exit mosaic second decode completed without its source caller return",
                );
                self.complete_overworld_special_exit_mosaic_after_second_decode();
                false
            }
            _ => unreachable!("combined Module09 graphics completion changed kind"),
        };
        if caller_remains_suspended {
            // The restore checkpoint is inside Module0B/$24's
            // second animated-tile decode. The ROM has not yet
            // reached Module09's Sprite_Main/HUD suffix, so do
            // not create or consume any of that caller's claims.
            assert!(
                authoritative_scheduled_caller_return_timeline.is_none(),
                "a special-exit restore checkpoint cannot own a terminal caller return",
            );
            assert_eq!(
                continued_sprite_main_claims, 0,
                "a special-exit restore checkpoint cannot own Sprite_Main returns",
            );
            return true;
        }
        self.complete_module09_overworld_after_submodule();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            // The forwarded wire boundary suspended the caller
            // mid-loop (Sprite_Main at route host 67569, LinkOam
            // at route host 69140); the shared suffix and the
            // following fresh iteration belong to the resumed
            // slices, not this host.
            assert!(
                authoritative_scheduled_caller_return_timeline.is_none(),
                "a suspended Module09 graphics caller cannot also own a terminal return",
            );
            if continued_sprite_main_claims != 0
                && self
                    .original_timing_sprite_main_return_claims_remaining
                    .is_some()
            {
                self.finish_original_timing_sprite_main_return_claim_scope();
            }
            return true;
        }
        if authoritative_scheduled_caller_return_timeline.is_some()
            || continued_sprite_main_claims != 0
        {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                timeline.interruption == crate::MainLoopInterruption::SpritePreparation
            })
        {
            // The host returned inside the resumed caller's
            // NMI_PrepareSprites (route host 187535).
            assert!(
                    authoritative_scheduled_caller_return_timeline.is_none(),
                    "a Module09 graphics caller interrupted in NMI_PrepareSprites cannot own a terminal return",
                );
            self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                NmiPrepareSpritesCpuCaller::Module09LongLoad,
            );
            return true;
        }
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && authoritative_scheduled_caller_return_timeline.is_none()
        {
            // Without a terminal return the wire can hold the
            // caller's shared suffix past this host behind a
            // trailing Held acceptance (route host 196969); the
            // next host's continued return completes it.
            self.retire_or_defer_main_loop_common_suffix_by_wire();
            return true;
        }
        if self.pending_main_loop_common_suffix.is_some() {
            // The source-proven caller return already carries the
            // shared ZeldaRunGameLoop suffix; retire its one owner.
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        }
        if work == GameWorkContinuation::FinishOverworldAuxGraphics
            && authoritative_scheduled_caller_return_timeline.is_none()
        {
            // The estimate lane holds the next iteration for its
            // measured pre-main resume; a wire-proven return has
            // already reached main wait, and the following fresh
            // iteration proceeds on its own receipts.
            self.game_execution_scheduler
                .schedule_pre_main_nmi_resume(PreMainNmiResume::OverworldAuxGraphicsReturn);
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_module09_long_load(
        &mut self,
        input: u16,
        step: Module09LongLoadStep,
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
    ) -> bool {
        // The whirlpool's long load returns to Module09_2E_Whirlpool
        // immediately after this vblank; the case tail and the
        // Module09 Sprite_Main/HUD suffix follow in the same host.
        if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        }
        let continued_sprite_main_claims =
            if authoritative_scheduled_caller_return_timeline.is_none() {
                self.original_timing_semantic_receipts
                    .as_ref()
                    .map(|receipts| {
                        receipts
                            .semantic()
                            .iter()
                            .filter(|receipt| {
                                **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                            })
                            .count()
                    })
                    .unwrap_or(0)
            } else {
                0
            };
        if let Some((_, _, sprite_main_return_claims)) =
            authoritative_scheduled_caller_return_timeline.as_ref()
        {
            self.begin_original_timing_sprite_main_return_claim_scope(*sprite_main_return_claims);
        } else if continued_sprite_main_claims != 0 {
            self.begin_original_timing_sprite_main_return_claim_scope(continued_sprite_main_claims);
        }
        self.complete_module09_long_load_step(step);
        if self.game_execution_scheduler.work_is_pending() {
            // The step scheduled its own successor phase (the
            // mirror-warp sprite load's reload after its Map16
            // conversion); the caller stays suspended.
            assert!(
                authoritative_scheduled_caller_return_timeline.is_none(),
                "a phased Module09 long load cannot own a terminal return mid-way",
            );
            return true;
        }
        if step.caller_is_module15() {
            if step.module15_runs_sprite_main_after() {
                // KillAghanim_Func5 returns into Module15/6, whose
                // dispatcher runs Sprite_Main and LinkOam.
                assert!(
                    self.take_original_timing_dialogue_closed(),
                    "Module15's reload return left the dialogue module without a close receipt",
                );
                self.arm_live_victory_module_sprite_main_boundary();
                self.sprite_main();
                if self
                    .game_execution_scheduler
                    .work_suspends_translated_call_stack()
                {
                    if continued_sprite_main_claims != 0
                        && self
                            .original_timing_sprite_main_return_claims_remaining
                            .is_some()
                    {
                        self.finish_original_timing_sprite_main_return_claim_scope();
                    }
                    return true;
                }
                self.link_oam_main();
            }
            // Module15's warp caller has no Sprite_Main/LinkOam
            // suffix; only the shared suffix remains.
            if authoritative_scheduled_caller_return_timeline.is_some()
                || continued_sprite_main_claims != 0
            {
                self.finish_original_timing_sprite_main_return_claim_scope();
            }
            if authoritative_scheduled_caller_return_timeline.is_some()
                || !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            {
                self.retire_or_run_main_loop_common_suffix_after_module_return();
            } else {
                self.retire_or_defer_main_loop_common_suffix_by_wire();
            }
            return true;
        }
        self.complete_module09_overworld_after_submodule();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            // The resumed caller's Sprite_Main suspended mid-loop
            // (route host 183230); the shared suffix belongs to
            // the resumed slice.
            assert!(
                authoritative_scheduled_caller_return_timeline.is_none(),
                "a suspended whirlpool caller cannot also own a terminal return",
            );
            if continued_sprite_main_claims != 0
                && self
                    .original_timing_sprite_main_return_claims_remaining
                    .is_some()
            {
                self.finish_original_timing_sprite_main_return_claim_scope();
            }
            return true;
        }
        if authoritative_scheduled_caller_return_timeline.is_some()
            || continued_sprite_main_claims != 0
        {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && authoritative_scheduled_caller_nmi_timeline.is_some_and(|timeline| {
                timeline.interruption == crate::MainLoopInterruption::SpritePreparation
            })
        {
            assert!(
                authoritative_scheduled_caller_return_timeline.is_none(),
                "a whirlpool caller interrupted in NMI_PrepareSprites cannot own a terminal return",
            );
            self.schedule_live_interrupted_nmi_prepare_sprites_caller_return(
                NmiPrepareSpritesCpuCaller::Module09LongLoad,
            );
            return true;
        }
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && authoritative_scheduled_caller_return_timeline.is_none()
        {
            self.retire_or_defer_main_loop_common_suffix_by_wire();
        } else {
            self.retire_or_run_main_loop_common_suffix_after_module_return();
        }
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_overworld_screen_map_and_sprite_graphics_tail(
        &mut self,
        input: u16,
        authoritative_scheduled_caller_accepts_nmi_at_return: Option<bool>,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        // The initial screen-map build and 3bpp-to-4bpp sprite
        // conversion return after this frame's NMI. The caller
        // suffix then publishes sprite DMA sources and releases
        // the software NMI latch for the following boundary.
        if authoritative_scheduled_caller_accepts_nmi_at_return.is_none() {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        }
        if let Some((_, _, sprite_main_return_claims)) =
            authoritative_scheduled_caller_return_timeline.as_ref()
        {
            self.begin_original_timing_sprite_main_return_claim_scope(*sprite_main_return_claims);
        }
        let continued_tail_sprite_main_claims =
            if authoritative_scheduled_caller_return_timeline.is_none() {
                self.original_timing_semantic_receipts
                    .as_ref()
                    .map(|receipts| {
                        receipts
                            .semantic()
                            .iter()
                            .filter(|receipt| {
                                **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                            })
                            .count()
                    })
                    .unwrap_or(0)
            } else {
                0
            };
        if continued_tail_sprite_main_claims != 0 {
            // The wire completed this caller's Sprite_Main inside
            // the host while the iteration stays suspended at the
            // trailing Held acceptance (route host 70608). The
            // decoder names no boundary for the few instructions
            // between the Sprite_Main return and LinkOam_Main, so
            // resume the whole LinkOam/HUD/rain suffix after the
            // interrupt through the shared Module09 continuation.
            self.begin_original_timing_sprite_main_return_claim_scope(
                continued_tail_sprite_main_claims,
            );
            self.forward_original_timing_main_loop_interruption_to_native_owner(
                crate::MainLoopInterruption::LinkOam,
                OriginalTimingBoundary::NmiAccepted,
            );
        }
        self.complete_module09_load_new_map_and_gfx_tail();
        self.complete_module09_overworld_after_submodule();
        if continued_tail_sprite_main_claims == 0
            && self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
        {
            // The forwarded wire boundary suspended the caller's
            // Sprite_Main mid-loop (route host 70838); the suffix
            // and scroll publication belong to the resumed slices.
            assert!(
                authoritative_scheduled_caller_return_timeline.is_none(),
                "a suspended screen-map tail Sprite_Main cannot also own a terminal return",
            );
            return true;
        }
        if continued_tail_sprite_main_claims != 0 {
            if self
                .original_timing_sprite_main_return_claims_remaining
                .is_some()
            {
                self.finish_original_timing_sprite_main_return_claim_scope();
            }
            assert!(
                !self.game_execution_scheduler.is_idle(),
                "the continued screen-map tail did not retain its suspended Module09 caller",
            );
            return true;
        }
        if authoritative_scheduled_caller_return_timeline.is_some() {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        // Although this caller suffix returns after NMI, Snes9x
        // exposes its direct BG register writes in the scanout
        // returned for this boundary. Publish only that register
        // domain into the captured image: VRAM, OAM, CGRAM, and
        // the remaining controls still belong to the pre-return
        // generation.
        let returned_scroll = self.bg_scroll_scanout_from_nmi_register_mirrors();
        // These direct PPU writes occur after this scanout's NMI.
        // They configure the following active frame; the immutable
        // snapshot captured above retains the registers that were
        // already being scanned out.
        self.publish_bg_scroll_for_following_scanout(returned_scroll);
        if self.pending_main_loop_common_suffix.is_some() {
            // The source-proven caller return already carries the
            // shared ZeldaRunGameLoop suffix; retire its one owner.
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        }
        // The map/sprite graphics tail has now returned and
        // rebuilt the same transition-entry sprite image. The
        // snapshot above still owns the held hardware OAM for
        // this scanout; subsequent frames resume normal cadence.
        self.active_display_obj_generation = DisplayObjGeneration::FollowModuleCadence;
        // The next ordinary Module09 iteration begins at the
        // vblank edge immediately following this returned
        // graphics tail. Carry that CPU phase explicitly into
        // the sprite-loader timing decision instead of encoding
        // the route or overworld screen number.
        self.next_overworld_sprite_reload_entry_phase =
            Some(OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail);
        true
    }

    /// Scheduled-work arm of `run_frame_internal_after_original_timing_body` (mechanically extracted; the body is unchanged).
    /// Returns `true` when the arm completed the host frame.
    pub(super) fn lane_finish_overworld_sprite_reload_tail(
        &mut self,
        input: u16,
        post_return_hold_nmi_slices: u8,
        epilogue_phase: NmiPhase,
        resume_scanout: OverworldSpriteReloadResumeScanout,
        authoritative_scheduled_caller_return_timeline: Option<(
            OriginalTimingMainLoopReturnTimeline,
            GameWorkContinuation,
            usize,
        )>,
        oam_dma_source: Option<Vec<u8>>,
    ) -> bool {
        // The long sprite reset/load loop is interrupted in the
        // ROM. Workload-derived return phase owns publication and
        // epilogue order independently from the host hold count.
        let untimed_sprite_main_return_claims =
            if authoritative_scheduled_caller_return_timeline.is_none() {
                // Without a wire-proven terminal return (the
                // suffix stays outstanding past this host), the
                // resumed caller's own Sprite_Main still crosses
                // its slot-zero return here; the host vector's
                // SpriteMainReturned facts bound those native
                // crossings (route host 117638).
                self.original_timing_semantic_receipts
                    .as_ref()
                    .map(|receipts| {
                        receipts
                            .semantic()
                            .iter()
                            .filter(|receipt| {
                                **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                            })
                            .count()
                    })
                    .filter(|count| *count != 0)
            } else {
                None
            };
        if let Some((_, _, sprite_main_return_claims)) =
            authoritative_scheduled_caller_return_timeline.as_ref()
        {
            self.begin_original_timing_sprite_main_return_claim_scope(*sprite_main_return_claims);
        } else if let Some(claims) = untimed_sprite_main_return_claims {
            self.begin_original_timing_sprite_main_return_claim_scope(claims);
        }
        self.publish_deferred_module09_sprite_slots_at_reload_return();
        self.complete_module09_load_new_sprites_after_reload();
        self.complete_module09_overworld_after_prepublished_rain();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            let caller = self
                .active_module09_sprite_main_return
                .as_mut()
                .expect("interrupted overworld reload must retain its Module09 Sprite_Main frame");
            assert_eq!(
                caller.after_sprite_main,
                Module09AfterSpriteMain::Ordinary,
                "overworld reload cannot replace an existing Module09 caller suffix",
            );
            caller.after_sprite_main = Module09AfterSpriteMain::FinishOverworldSpriteReload {
                post_return_hold_nmi_slices,
                epilogue_phase,
                resume_scanout,
            };
            return true;
        }
        if authoritative_scheduled_caller_return_timeline.is_some() {
            // The wire-proven return already ran its leading
            // acceptance and handler through the shared timeline
            // machinery, and the following hosts drive their own
            // NMIs and holds. Only the shared suffix remains.
            self.finish_original_timing_sprite_main_return_claim_scope();
            if self.pending_main_loop_common_suffix.is_some() {
                self.complete_pending_main_loop_common_suffix_after_module_return();
            } else {
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
            }
            return true;
        }
        if untimed_sprite_main_return_claims.is_some() {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        self.finish_overworld_sprite_reload_return(
            post_return_hold_nmi_slices,
            epilogue_phase,
            resume_scanout,
            input,
            oam_dma_source.as_deref(),
            false,
        );
        true
    }

    /// Dispatcher value `authoritative_item_receipt_returned` of `run_frame_internal_after_original_timing_body` (mechanically extracted; the initializer is unchanged).
    pub(super) fn compute_item_receipt_returned(
        &mut self,
        authoritative_item_receipt_is_active: bool,
    ) -> bool {
        if authoritative_item_receipt_is_active {
            let expected_caller = match self
                .game_execution_scheduler
                .current_work()
                .expect("item receipt work was checked above")
            {
                GameWorkContinuation::FinishItemReceiptGraphics {
                    continuation:
                        ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt {
                            sprite_slot,
                            suffix,
                            ..
                        },
                } => {
                    if suffix.caller_is_direct() {
                        ItemReceiptGraphicsCaller::SpriteMainDirect { slot: sprite_slot }
                    } else {
                        ItemReceiptGraphicsCaller::SpriteMain { slot: sprite_slot }
                    }
                }
                GameWorkContinuation::FinishItemReceiptGraphics {
                    continuation:
                        ItemReceiptGraphicsContinuation::ResumeUnclePassage { sprite_slot, .. },
                } => ItemReceiptGraphicsCaller::UnclePassage { slot: sprite_slot },
                GameWorkContinuation::FinishItemReceiptGraphics {
                    continuation:
                        ItemReceiptGraphicsContinuation::ResumeAncillaItemReceipt {
                            ancilla_slot, ..
                        },
                } => ItemReceiptGraphicsCaller::SpriteMainAncilla { slot: ancilla_slot },
                _ => unreachable!("item receipt work kind changed"),
            };
            let progress = self
                .take_original_timing_item_receipt_graphics_progress()
                .unwrap_or_else(|| {
                    panic!(
                        "live timing authority omitted progress for suspended {expected_caller:?}"
                    )
                });
            assert_eq!(
                progress.caller, expected_caller,
                "live item-receipt progress named a different suspended caller",
            );
            if let (
                ItemReceiptGraphicsCaller::SpriteMainDirect { slot }
                | ItemReceiptGraphicsCaller::SpriteMain { slot },
                GameWorkContinuation::FinishItemReceiptGraphics {
                    continuation:
                        ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt { .. },
                },
            ) = (
                expected_caller,
                self.game_execution_scheduler
                    .current_work()
                    .expect("item receipt work was checked above"),
            ) {
                // A sprite-slot call restates its surrounding
                // descending-loop checkpoint alongside the graphics claim on
                // every suspended host. Both name the same C statement; this
                // suspended continuation is their one owner.
                if let Some(restated_slot_boundary) =
                    self.take_original_timing_sprite_main_progress()
                {
                    // Slot 15 suspends before any slot boundary is recorded,
                    // so its restatement is BeforeFirstSlot (route host
                    // 102905); lower slots restate AfterSlot(slot + 1).
                    assert!(
                    direct_item_receipt_slot_pairs_with_boundary(slot, restated_slot_boundary,),
                    "an item-receipt claim disagrees with its restated Sprite_Main checkpoint: slot={slot} restated={restated_slot_boundary:?}",
                );
                }
            } else if let ItemReceiptGraphicsCaller::SpriteMainAncilla { .. } = expected_caller {
                // The prefix's ancilla receipt restates the loop-entry
                // checkpoint on every suspended host (route hosts 1142851,
                // 514801).
                if let Some(restated) = self.take_original_timing_sprite_main_progress() {
                    assert_eq!(
                    restated,
                    SpriteMainCpuBoundary::BeforeFirstSlot,
                    "an ancilla item-receipt claim disagrees with its restated Sprite_Main checkpoint: {restated:?}",
                );
                }
            }
            progress.progress == SourceCallProgress::Returned
        } else {
            false
        }
    }

    /// Dispatcher value `authoritative_scheduled_caller_return_timeline` of `run_frame_internal_after_original_timing_body` (mechanically extracted; the initializer is unchanged).
    pub(super) fn compute_scheduled_caller_return_timeline(
        &mut self,
        authoritative_main_loop_interruption_timeline: &Option<
            OriginalTimingMainLoopInterruptionTimeline,
        >,
    ) -> Option<(
        OriginalTimingMainLoopReturnTimeline,
        GameWorkContinuation,
        usize,
    )> {
        (matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            && self.game_execution_scheduler.current_work().is_some_and(|work| {
                work.scheduled_caller_return_timeline_owns_terminal_return()
            })
            && authoritative_main_loop_interruption_timeline.is_none())
        .then(|| self.original_timing_main_loop_return_timeline())
        .flatten()
        .map(|expected_timeline| {
            // This receipt is an exact terminal C boundary, not merely a
            // progress hint. Validate the complete lifecycle and the
            // scheduler commit on copies before consuming any authority or
            // advancing the real continuation.
            assert_eq!(
                expected_timeline.progress,
                crate::MainLoopProgress::CallStackContinued,
                "a scheduled caller's terminal return cannot begin a fresh main-loop iteration",
            );
            assert_eq!(
                self.pending_main_loop_common_suffix,
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
                "a scheduled caller's terminal return lost its ordinary ZeldaRunGameLoop suffix",
            );
            assert!(
                self.original_timing_main_loop_interruption().is_none(),
                "an uninterrupted scheduled caller return cannot also publish a main-loop interruption",
            );
            if self.original_timing_nmi_publication_pending {
                assert_eq!(
                    self.original_timing_pending_nmi_update_gate,
                    Some(NmiUpdateGate::LatchHeld),
                    "a scheduled terminal caller may only resume a carried held handler",
                );
                assert_eq!(
                    expected_timeline.nmi_phases_before_return,
                    [OriginalTimingNmiPhase::HandlerCompleted],
                    "a scheduled terminal caller with a carried handler must complete it without accepting another NMI: {expected_timeline:?}",
                );
            } else {
                assert_eq!(
                    self.original_timing_pending_nmi_update_gate, None,
                    "a scheduled terminal caller retained an NMI gate without a carried handler",
                );
                // A forced-blank load can return with no NMI in the whole
                // host interval (route host 58588).
                assert!(
                    expected_timeline.nmi_phases_before_return.is_empty()
                        || expected_timeline.nmi_phases_before_return
                            == [
                                OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                                OriginalTimingNmiPhase::HandlerCompleted,
                            ],
                    "a scheduled terminal caller must close its same-host held handler before returning: {expected_timeline:?}",
                );
            }
            assert!(
                expected_timeline.nmi_phases_after_return.is_empty()
                    || expected_timeline.nmi_phases_after_return
                        == [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open)],
                "a scheduled terminal caller may only stop at main wait or carry exactly its following open NMI: {expected_timeline:?}",
            );
            assert!(
                !self.original_timing_scheduled_nmi_accepted_at_host_return,
                "a scheduled terminal caller cannot overlap an older staged acceptance",
            );
            assert_eq!(
                self.original_timing_sprite_main_return_claims_remaining, None,
                "a scheduled terminal caller cannot overlap an older Sprite_Main return-claim scope",
            );
            let before_return = try_classify_original_timing_nmi_phases(
                self.original_timing_nmi_publication_pending,
                &expected_timeline.nmi_phases_before_return,
            )
            .unwrap_or_else(|| {
                panic!(
                    "scheduled caller return has an unsupported pre-return NMI lifecycle: {expected_timeline:?}",
                )
            });
            assert!(
                !before_return.publication_pending_at_exit,
                "a scheduled caller cannot enter its common suffix with an unfinished NMI handler: {expected_timeline:?}",
            );
            let after_return = try_classify_original_timing_nmi_phases(
                false,
                &expected_timeline.nmi_phases_after_return,
            )
            .unwrap_or_else(|| {
                panic!(
                    "scheduled caller return has an unsupported post-return NMI lifecycle: {expected_timeline:?}",
                )
            });
            assert_eq!(
                before_return.handler_completion,
                if self.original_timing_nmi_publication_pending {
                    OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry
                } else if expected_timeline.nmi_phases_before_return.is_empty() {
                    OriginalTimingNmiHandlerCompletionOwner::None
                } else {
                    OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence
                },
                "a scheduled terminal caller lost ownership of its leading handler",
            );
            assert_eq!(
                after_return.handler_completion,
                OriginalTimingNmiHandlerCompletionOwner::None,
                "a scheduled terminal caller cannot execute a post-return NMI handler",
            );
            assert_eq!(
                after_return.publication_pending_at_exit,
                !expected_timeline.nmi_phases_after_return.is_empty(),
                "a scheduled terminal caller's pending publication must exactly match its optional trailing open acceptance",
            );
            assert_original_timing_carry_in_handler_has_receptive_display(
                before_return,
                self.display_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
            );

            let mut expected_gates = Vec::new();
            if self.original_timing_nmi_publication_pending {
                expected_gates.push(
                    self.original_timing_pending_nmi_update_gate.expect(
                        "a scheduled caller's carried handler lost its accepted gate",
                    ),
                );
            } else {
                assert_eq!(
                    self.original_timing_pending_nmi_update_gate, None,
                    "a scheduled caller retained an NMI gate without a carried handler",
                );
            }
            expected_gates.extend(
                expected_timeline
                    .nmi_phases_before_return
                    .iter()
                    .chain(expected_timeline.nmi_phases_after_return.iter())
                    .filter_map(|phase| match phase {
                        OriginalTimingNmiPhase::Accepted(gate) => Some(*gate),
                        OriginalTimingNmiPhase::HandlerCompleted => None,
                    }),
            );
            assert_eq!(
                self.original_timing_expected_nmi_update_gates, expected_gates,
                "a scheduled caller return disagrees with its installed NMI gate authority",
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
                    "a scheduled caller return's leading handler disagrees with the native NMI latch",
                );
            }

            let mut expected_work = self
                .game_execution_scheduler
                .current_work()
                .expect("scheduled caller return lost its active continuation");
            let mut expected_semantic = expected_timeline
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
            // The saved caller resumes after its submodule and runs
            // however many shared Sprite_Main loops the wire itself
            // proves before returning to ZeldaRunGameLoop's
            // unconditional common suffix. A scheduled pre-overworld
            // stage returns through the same suffix without Sprite_Main;
            // its stage receipt is appended at host finish.
            let sprite_main_return_claims = self
                .original_timing_semantic_receipts
                .as_ref()
                .map(|receipts| {
                    receipts
                        .semantic()
                        .iter()
                        .filter(|receipt| {
                            **receipt == OriginalTimingSemanticReceipt::SpriteMainReturned
                        })
                        .count()
                })
                .unwrap_or(0);
            match expected_work.scheduled_caller_return_expected_sprite_main_claims() {
                Some(expected_claims) => assert_eq!(
                    sprite_main_return_claims, expected_claims,
                    "a scheduled caller return published the wrong number of Sprite_Main returns for {expected_work:?}",
                ),
                None => assert!(
                    sprite_main_return_claims <= 1,
                    "a scheduled caller return cannot run Sprite_Main more than once",
                ),
            }
            if expected_timeline.sprite_main_returned_before_nmi
                && scheduled_caller_return_runs_dungeon_sprite_main_before_leading_nmi(
                    expected_work,
                )
            {
                // The parked dungeon slot loop finished before this
                // host's held vblank; the Module 7 tail and the shared
                // suffix ran after the handler (route host 1415870:
                // [SpriteMainReturned, NmiAccepted(LatchHeld),
                // NmiHandlerCompleted, CallStackContinued,
                // MainLoopCommonSuffixCompleted, NmiAccepted(Open)]).
                expected_semantic.splice(
                    0..0,
                    std::iter::repeat_n(
                        OriginalTimingSemanticReceipt::SpriteMainReturned,
                        sprite_main_return_claims,
                    ),
                );
            } else {
                expected_semantic.extend(std::iter::repeat_n(
                    OriginalTimingSemanticReceipt::SpriteMainReturned,
                    sprite_main_return_claims,
                ));
            }
            expected_semantic.extend([
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ]);
            expected_semantic.extend(expected_timeline.nmi_phases_after_return.iter().map(
                |phase| match phase {
                    OriginalTimingNmiPhase::Accepted(gate) => {
                        OriginalTimingSemanticReceipt::NmiAccepted(*gate)
                    }
                    OriginalTimingNmiPhase::HandlerCompleted => {
                        OriginalTimingSemanticReceipt::NmiHandlerCompleted
                    }
                },
            ));
            if let Some(stage) = expected_work.pre_overworld_stage_completion() {
                expected_semantic
                    .push(OriginalTimingSemanticReceipt::PreOverworldStageCompleted(
                        stage,
                    ));
            }
            if matches!(
                expected_work,
                GameWorkContinuation::FinishTriforceRoomLoad {
                    step: TriforceRoomLoadStep::Case7TextInit,
                }
            ) {
                // Case 7 restores module $19 from the dialogue module as it
                // returns; the decoder reports that as a dialogue close in the
                // same host (route host 1557809).
                expected_semantic.push(OriginalTimingSemanticReceipt::DialogueClosed);
            }
            if expected_work == GameWorkContinuation::FinishWorldMapExitTilesets {
                // WorldMap_ExitMap closes the Module0E map overlay as part
                // of this same return, so the adapter appends its
                // dialogue-close fact at host finish.
                expected_semantic.push(OriginalTimingSemanticReceipt::DialogueClosed);
            }
            if expected_work == GameWorkContinuation::FinishWorldMapOverlayReload {
                // The Module09/$20 overlay reload publishes its module
                // return alongside this same host return (route host
                // 61935).
                expected_semantic
                    .push(OriginalTimingSemanticReceipt::WorldMapOverlayReloadReturned);
            }
            if matches!(
                expected_work,
                GameWorkContinuation::FinishModule09LongLoad {
                    step: Module09LongLoadStep::Module15ReloadSheetsAfterMessage,
                }
            ) {
                // KillAghanim_Func5's reload ran under the dialogue module;
                // its return to Module15 closes it at this same host
                // return (route host 315338).
                expected_semantic.push(OriginalTimingSemanticReceipt::DialogueClosed);
            }
            if matches!(
                expected_work,
                GameWorkContinuation::FinishDesertPrayerIris {
                    caller: DesertPrayerIrisCaller::RecurringCase4,
                    ..
                }
            ) && self.desert_prayer_iris_completion_closes_dialogue()
            {
                // The terminal opening-radius step restores the saved
                // gameplay module from Module0E in the same source host
                // that returns the suspended iris builder.
                expected_semantic.push(OriginalTimingSemanticReceipt::DialogueClosed);
            }
            if matches!(
                expected_work,
                GameWorkContinuation::FinishDungeonExitSpotlightLinkAndOam { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightControl { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates { .. }
            ) {
                // The resumed post-submodule suffix or movement leaf
                // returns through Module0F to the main wait alongside
                // this same host return (route hosts 252090 and 50636).
                expected_semantic.push(
                    OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
                );
            }
            if matches!(
                expected_work,
                GameWorkContinuation::FinishPreDungeonEntranceLoad { .. }
            ) {
                // Module_PreDungeon publishes its module return alongside
                // this same host return (route host 39723).
                expected_semantic.push(OriginalTimingSemanticReceipt::PreDungeonModuleReturned);
            }
            {
                // A reset, cached-sprite, spotlight, peg-loop, or
                // push-block refinement claim is consumed by the resumed
                // body itself; this lane only pins it at its exact wire
                // position (route hosts 29550, 31288, 712708, and
                // 1526914).
                let semantic_actual = self
                    .original_timing_semantic_receipts
                    .as_ref()
                    .expect("scheduled caller return lost its semantic authority")
                    .semantic()
                    .to_vec();
                for (index, receipt) in semantic_actual.iter().enumerate() {
                    if matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(_)
                            | OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(_)
                            | OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(_)
                            | OriginalTimingSemanticReceipt::SpriteResetAllProgress(_)
                            | OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(_)
                            | OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(_)
                            | OriginalTimingSemanticReceipt::CreditsEndSequence32Progress(_)
                            | OriginalTimingSemanticReceipt::DungeonPushBlocksInProgress { .. }
                            | OriginalTimingSemanticReceipt::DungeonPushBlocksHandled
                    ) {
                        expected_semantic.insert(index.min(expected_semantic.len()), *receipt);
                    }
                }
            }
            assert_eq!(
                self.original_timing_semantic_receipts
                    .as_ref()
                    .expect("scheduled caller return lost its semantic authority")
                    .semantic(),
                expected_semantic,
                "a scheduled caller return published an unsupported or reordered semantic vector",
            );
            if after_return.publication_pending_at_exit {
                // A retained landing-goal caller-return image composes its
                // own trailing-acceptance capture in the completion arm
                // (`carry_original_timing_scheduled_caller_host_return_
                // from_active_capture`), so both display policies are
                // valid here (route host 39759).
            }
            let mut scheduler_probe = self.game_execution_scheduler;
            assert_eq!(
                scheduler_probe.advance_work_one_nmi_slice_with_authoritative_completion(true),
                Some(GameWorkStep::Complete(expected_work)),
                "a terminal source return cannot complete only part of its scheduled caller",
            );

            let consumed_timeline = self
                .take_original_timing_main_loop_return_timeline()
                .expect("validated scheduled-caller return timeline disappeared");
            assert_eq!(
                consumed_timeline, expected_timeline,
                "scheduled-caller return timeline changed before consumption",
            );
            if let Some(stage) = expected_work.pre_overworld_stage_completion() {
                // The scheduler commit below retires the stage owner, so
                // its host-finish stage receipt must be claimed here.
                assert!(
                    self.take_original_timing_pre_overworld_stage_completion(stage),
                    "a pre-overworld stage return lost its boundary-qualified stage receipt",
                );
            }
            if matches!(
                expected_work,
                GameWorkContinuation::FinishDungeonExitSpotlightLinkAndOam { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightControl { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates { .. }
            ) {
                assert!(
                    self.take_original_timing_dungeon_exit_spotlight_caller_returned(),
                    "a spotlight movement terminal lost its caller-return token",
                );
            }
            if expected_work == GameWorkContinuation::FinishWorldMapOverlayReload {
                assert!(
                    self.take_original_timing_world_map_overlay_reload_returned(),
                    "an overlay-reload terminal lost its module-return token",
                );
            }
            if expected_work
                == (GameWorkContinuation::FinishTriforceRoomLoad {
                    step: TriforceRoomLoadStep::Case9Scroll,
                })
                && !self.dialogue_scroll_cpu_is_idle()
            {
                // The terminal host copies the scroll's last passes and
                // returns through RenderText into Module19's tail (route
                // host 1558267).
                assert!(
                    self.advance_triforce_room_dialogue_scroll_lag_host(
                        DialogueScrollCompletionTiming::BeforeNextVblank,
                    ),
                    "a Triforce-room scroll terminal lost its copy/return receipt",
                );
            }
            if matches!(
                expected_work,
                GameWorkContinuation::FinishPreDungeonEntranceLoad { .. }
            ) {
                assert!(
                    self.take_original_timing_pre_dungeon_module_returned(),
                    "a pre-dungeon terminal return lost its module-return receipt",
                );
                // The reset claim restates the statement the completed
                // reset continuation already carries.
                if let Some(receipt) = self.take_original_timing_sprite_reset_all_progress() {
                    assert_eq!(
                        receipt.boundary,
                        crate::OriginalTimingBoundary::NmiAccepted,
                        "a pre-dungeon reset claim must ride its accepting NMI",
                    );
                }
            }
            if matches!(
                expected_work,
                GameWorkContinuation::FinishTriforceRoomLoad {
                    step: TriforceRoomLoadStep::CreditsIteration { .. },
                }
            ) {
                // A terminal outer return dominates any earlier scene/text
                // checkpoint in the same host. Retire the corroborative
                // checkpoint so the completion body runs the entire
                // source call and does not invent another continuation.
                let _ = self.take_original_timing_credits_scene_load_progress();
                let _ = self.take_original_timing_credits_end_sequence_32_progress();
            }
            if matches!(
                expected_work,
                GameWorkContinuation::FinishOverworldSpotlightBuild { .. }
                    | GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. }
            ) {
                // An acceptance can expose later source stores than the
                // preceding host-return checkpoint. Validate monotonic C
                // progress and rebuild from that exact statement before
                // the accepting handler observes the table (route hosts
                // 37742, 48445, and 66297).
                if let Some(claim) = self.take_original_timing_spotlight_table_build_progress()
                {
                    assert_eq!(
                        claim.boundary,
                        crate::OriginalTimingBoundary::NmiAccepted,
                        "a spotlight build re-checkpoint must ride an accepting NMI",
                    );
                    expected_work = match expected_work {
                        GameWorkContinuation::FinishOverworldSpotlightBuild {
                            table_build,
                            phase,
                            projection_completed,
                            iteration,
                        } => {
                            table_build.assert_recheckpoint_not_behind(claim.progress);
                            let table_build = if table_build.source_progress
                                == Some(claim.progress)
                            {
                                table_build
                            } else {
                                self.begin_iris_spotlight_configure_table_at_progress(
                                    claim.progress,
                                )
                            };
                            GameWorkContinuation::FinishOverworldSpotlightBuild {
                                table_build,
                                phase,
                                projection_completed,
                                iteration,
                            }
                        }
                        GameWorkContinuation::FinishDungeonExitSpotlightEntry {
                            table_build,
                            iteration,
                        } => {
                            table_build.assert_recheckpoint_not_behind(claim.progress);
                            let table_build = if table_build.source_progress
                                == Some(claim.progress)
                            {
                                table_build
                            } else {
                                self.begin_iris_spotlight_configure_table_at_progress(
                                    claim.progress,
                                )
                            };
                            GameWorkContinuation::FinishDungeonExitSpotlightEntry {
                                table_build,
                                iteration,
                            }
                        }
                        _ => unreachable!("spotlight re-checkpoint owner changed"),
                    };
                }
            }
            if expected_work == GameWorkContinuation::FinishWorldMapExitTilesets {
                assert!(
                    self.take_original_timing_dialogue_closed(),
                    "a world-map exit return lost its dialogue-close receipt",
                );
                assert!(
                    self.frame_module_hosts_dialogue(),
                    "dialogue-close receipt reached native gameplay outside Module0E/Module1B",
                );
                // Match the idle Module0E consumer: mark the semantic
                // branch and let the existing C translation own every
                // border/messaging/submodule mutation.
                self.messaging_state_mut().set_text_render_state(4);
            }
            // Commit the exact scheduler state whose completion was
            // validated above. The handler and resumed CPU caller below
            // must never run against a second, independently advanced
            // scheduler generation.
            if matches!(expected_work, GameWorkContinuation::FinishDungeonCachedSpriteMain { .. }) {
                // Preserve the final restore stores while this caller
                // still owns its backup, before committing its retirement
                // and publishing the accepting handler below.
                self.publish_cached_sprite_restore_before_acceptance();
                expected_work = self.game_execution_scheduler.current_work().unwrap();
            }
            self.game_execution_scheduler = scheduler_probe;
            (consumed_timeline, expected_work, sprite_main_return_claims)
        })
    }
}
