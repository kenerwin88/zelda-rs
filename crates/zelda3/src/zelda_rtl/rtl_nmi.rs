//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (nmi).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub(super) fn debug_latch_frame_matches(&self) -> bool {
        static RANGE: std::sync::OnceLock<Option<(u32, u32)>> = std::sync::OnceLock::new();
        let Some((lo, hi)) = RANGE.get_or_init(|| {
            let value = crate::debug_env::var("ZELDA3_DEBUG_LATCH").ok()?;
            let (lo, hi) = value.split_once('-')?;
            Some((lo.parse().ok()?, hi.parse().ok()?))
        }) else {
            return false;
        };
        (*lo..=*hi).contains(&self.frame_ctr_dbg)
    }

    /// `ZELDA3_DEBUG_LATCH=<lo>-<hi>` traces every software NMI-latch
    /// set/clear with its `#[track_caller]` provenance for hosts in range —
    /// the direct probe for latch-disposition divergences.
    #[track_caller]
    pub(crate) fn latch_nmi_update(&mut self) {
        if self.debug_latch_frame_matches() {
            eprintln!(
                "[LATCH] set host={} caller={}",
                self.frame_ctr_dbg,
                core::panic::Location::caller()
            );
        }
        self.display_core_mut().latch_nmi_update();
    }

    #[track_caller]
    pub(crate) fn clear_nmi_update_latch(&mut self) {
        if self.debug_latch_frame_matches() {
            eprintln!(
                "[LATCH] clear host={} caller={}",
                self.frame_ctr_dbg,
                core::panic::Location::caller()
            );
        }
        self.display_core_mut().clear_nmi_update_latch();
    }

    pub(super) fn stage_spiral_stairs_second_grayscale_nmi(&mut self) -> GraphicsDmaGeneration {
        // The final quadrant-upload NMI has returned, so the atomic ROM path
        // exposes $0710 = 0 to this caller. Its core DMA is allowed to run.
        // The resident animated batch remains visible for this image. When
        // this suffix consumes countdown 1 it advances $0adc before NMI, so
        // the DMA must use the live operand even though its result belongs to
        // the following scanout.
        self.clear_core_update_disable_flag();
        self.next_display_animated_bg_scanout_generation =
            Some(AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi);
        if self.game_state.display.bg_tile_animation_countdown == 1 {
            GraphicsDmaGeneration::LiveAfterMain
        } else {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        }
    }

    /// Env-gated provenance for the presented Link OBJ CHR latch: every
    /// assignment logs its call site under `ZELDA3_DEBUG_OBJ_LATCH=<frames>`,
    /// naming the rule that authored a frame's presented OBJ memory in one
    /// probe (the CHR-domain counterpart of the OBJ-scanout staged_by tool).
    #[track_caller]
    pub(crate) fn set_obj_vram_latch_traced(&mut self, value: Option<Vec<u16>>) {
        if nmi::debug_frame_selection_env_matches("ZELDA3_DEBUG_OBJ_LATCH", self.frame_ctr_dbg) {
            let sample = value
                .as_deref()
                .map(|latch| latch.get(0x4020).copied().unwrap_or(0));
            eprintln!(
                "obj_latch host={} some={} w4020={:04x?} caller={}",
                self.frame_ctr_dbg,
                value.is_some(),
                sample,
                std::panic::Location::caller()
            );
        }
        self.ppu.obj_vram_latch = value;
    }

    /// Link OBJ operands consumed if the following NMI runs after this CPU
    /// slice. This is intentionally distinct from the presented generation.
    pub(super) fn following_nmi_link_obj_dma_generation(&self) -> GraphicsDmaGeneration {
        self.link_obj_dma_phase_generations().following_nmi
    }

    pub(crate) fn set_pending_nmi_subroutine(&mut self, value: u8) {
        self.display_core_mut().set_pending_nmi_subroutine(value);
    }

    pub(crate) fn clear_pending_nmi_subroutine(&mut self) {
        self.display_core_mut().clear_pending_nmi_subroutine();
    }

    pub(crate) fn take_pending_nmi_subroutine(&mut self) -> u8 {
        self.display_core_mut().take_pending_nmi_subroutine()
    }

    pub(crate) fn set_nmi_copy_packets_request(&mut self, value: u8) {
        self.display_core_mut().set_nmi_copy_packets_request(value);
    }

    pub(crate) fn request_nmi_copy_packets(&mut self) {
        self.display_core_mut().request_nmi_copy_packets();
    }

    pub(crate) fn clear_nmi_copy_packets_request(&mut self) {
        self.display_core_mut().clear_nmi_copy_packets_request();
    }

    pub(crate) fn activate_nmi_thread(&mut self) {
        if !self.game_state.display.nmi_thread_active {
            // This remains only as the no-wire fallback. Live parity consumes
            // the source-level render-start receipt because initial thread
            // scheduling varies with the activation host's CPU/NMI phase.
            self.poly_dungeon_thread_startup_hold = Some(6);
            self.poly_dungeon_activation_host = self.frame_ctr_dbg;
        }
        self.display_core_mut().activate_nmi_thread();
    }

    pub(crate) fn deactivate_nmi_thread(&mut self) {
        self.display_core_mut().deactivate_nmi_thread();
    }

    pub(crate) fn set_nmi_thread_stack_pointer(&mut self, value: u16) {
        self.display_core_mut().set_nmi_thread_stack_pointer(value);
    }

    pub(crate) fn set_nmi_load_target_page(&mut self, value: u8) {
        self.display_core_mut().set_nmi_load_target_page(value);
    }

    pub(crate) fn set_nmi_load_target_address(&mut self, value: u16) {
        self.display_core_mut().set_nmi_load_target_address(value);
    }

    pub(crate) fn nmi_vram_packet_buffer(&self) -> &[u8] {
        self.game_state.display.nmi_vram_packet_buffer(&self.ram)
    }

    pub(crate) fn weather_vane_music_latch(&self) -> u8 {
        self.game_state.world.overworld.weather_vane.music_latch
    }

    pub(crate) fn set_weather_vane_music_latch(&mut self, value: u8) {
        self.weather_vane_bridge_mut().set_music_latch(value);
    }

    pub(super) fn finish_dungeon_room_load_caller_after_trailing_nmi(&mut self) {
        if let Some(sprite_return) = self.active_dungeon_sprite_main_return.take() {
            self.complete_module07_after_sprite_main(sprite_return);
        } else {
            self.complete_module07_dungeon_after_submodule();
        }
        debug_assert!(!self
            .game_execution_scheduler
            .work_suspends_translated_call_stack());
        self.stage_live_animated_bg_scanout();
        if self.pending_main_loop_common_suffix.is_some() {
            // The source-proven caller return already carries the shared
            // ZeldaRunGameLoop suffix; retire its one owner.
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
        }
        self.finish_dungeon_room_load_caller_at_main_wait();
    }

    pub(super) fn retain_completed_nmi_scroll_for_current_scanout(&mut self) {
        // WritePpuRegisters runs even when the software latch holds the DMA
        // body. The resumed caller can author the next scroll mirrors, but
        // this field retains the registers that the completed handler wrote.
        let scroll = BgScrollRegisterScanout::capture(&self.ppu);
        self.display_snapshot.as_mut()
            .expect("completed held NMI requires its captured display")
            .bg_scroll_generation = DisplayBgScrollGeneration::RetainCpuSliceEntry(scroll);
    }

    pub(super) fn bg_scroll_scanout_from_nmi_register_mirrors(&self) -> BgScrollRegisterScanout {
        let scroll = &self.game_state.display.ppu_scroll_copy;
        BgScrollRegisterScanout::after_nmi_writes(
            &self.ppu,
            [
                [
                    scroll.bg1_h_copy_low(),
                    scroll.bg1_h_high(),
                    scroll.bg1_v_copy_low(),
                    scroll.bg1_v_high(),
                ],
                [
                    scroll.bg2_h_copy_low(),
                    scroll.bg2_h_high(),
                    scroll.bg2_v_copy_low(),
                    scroll.bg2_v_high(),
                ],
                [
                    scroll.bg3_h_copy2_low(),
                    scroll.bg3_h_high(),
                    scroll.bg3_v_copy2_low(),
                    scroll.bg3_v_high(),
                ],
            ],
        )
    }

    pub(super) fn atomic_item_graphics_holds_following_nmi(&self) -> bool {
        matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishItemReceiptGraphics {
                continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { .. },
            })
        )
    }

    /// Consume a wire-proven terminal caller return for a resumed post-NMI
    /// continuation: the carried handler completes first, the resumed body
    /// consumes exactly the Sprite_Main return claims the source published,
    /// and the caller runs through ZeldaRunGameLoop's shared suffix before an
    /// optional trailing Open acceptance.
    pub(super) fn complete_live_terminal_post_nmi_continuation(
        &mut self,
        continuation: GameWorkContinuation,
        input: u16,
        oam_dma_source: Option<&[u8]>,
    ) -> bool {
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || self.original_timing_semantic_receipts.is_none()
        {
            return false;
        }
        let Some(timeline) = self.take_original_timing_main_loop_return_timeline() else {
            return false;
        };
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
        if sprite_main_return_claims != 0 {
            self.begin_original_timing_sprite_main_return_claim_scope(sprite_main_return_claims);
        }
        self.complete_original_timing_main_loop_return(timeline, input, oam_dma_source, |state| {
            state.complete_post_trailing_nmi_continuation(continuation, input, false, true);
        });
        if sprite_main_return_claims != 0 {
            self.finish_original_timing_sprite_main_return_claim_scope();
        }
        if self
            .game_execution_scheduler
            .resumed_call_stack_is_before_nmi()
            && !self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
        {
            self.game_execution_scheduler
                .finish_call_stack_at_main_wait_before_nmi();
        }
        assert!(
            self.original_timing_semantic_receipts
                .as_ref()
                .is_none_or(|receipts| receipts.semantic().is_empty()),
            "a resumed post-NMI caller return left typed lifecycle receipts unconsumed",
        );
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        true
    }

    pub(super) fn complete_post_trailing_nmi_continuation(
        &mut self,
        continuation: GameWorkContinuation,
        input: u16,
        started_after_leading_nmi: bool,
        typed_main_loop_return: bool,
    ) {
        assert!(
            !typed_main_loop_return
                || continuation == GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn
                || matches!(
                    continuation,
                    GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn { .. }
                        | GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn
                        | GameWorkContinuation::FinishDialogueInitializationCallerReturn
                        | GameWorkContinuation::FinishModule09LinkOamCallerReturn { .. }
                        | GameWorkContinuation::FinishSpriteMain { .. }
                        | GameWorkContinuation::FinishDungeonCachedSpriteMain { .. }
                        | GameWorkContinuation::FinishDungeonSupertileTransition {
                            work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
                        }
                        | GameWorkContinuation::FinishStraightInterroomSpriteReset { .. }
                ),
            "only the source-owned dungeon caller return has a split CPU/common-suffix completion",
        );
        match continuation {
            GameWorkContinuation::FinishStraightInterroomSpriteReset { progress } => {
                // The wire's terminal return owns the shared suffix (route host
                // 307622); the estimate lane keeps the ordinary pair.
                self.complete_straight_interroom_sprite_reset_body(progress);
                if typed_main_loop_return {
                    self.complete_pending_main_loop_common_suffix_after_module_return();
                } else {
                    self.retire_or_run_main_loop_common_suffix_after_module_return();
                }
            }
            GameWorkContinuation::FinishDialogueInitializationCallerReturn => {
                self.complete_text_initialization_carry_suffix();
                if self.game_state.frame.main_module == 14
                    && self.game_state.frame.submodule == 11
                {
                    // Save-menu RenderText is the suspended callee. Its
                    // flags and selection-state suffix follow the completed
                    // initialization, before Module0E's scroll-register tail.
                    self.complete_save_menu_after_render_text();
                }
                self.complete_module0e_interface_after_run();
                if self.pending_main_loop_common_suffix.is_some() {
                    // The source-proven caller return already carries the
                    // shared ZeldaRunGameLoop suffix; retire its one owner.
                    self.complete_pending_main_loop_common_suffix_after_module_return();
                } else {
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                self.next_core_nmi_active_scanout_uses_host_animated_bg_operands = std::mem::take(
                    &mut self.normal_dialogue_following_main_nmi_uses_host_animated_bg_operands,
                );
            }
            GameWorkContinuation::FinishDungeonMapRoomDrawing => {
                // The room builder resumes after the NMI and returns through
                // Module0E without beginning another main-loop iteration.
                self.complete_dungeon_map_room_drawing();
                self.complete_module0e_interface_after_run();
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
            }
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
            } => {
                // Resume only the interrupted Module 7 suffix. The next fresh
                // state-2 iteration remains behind the following leading NMI.
                self.finish_dungeon_room_load_caller_after_trailing_nmi();
            }
            GameWorkContinuation::FinishSpriteMain { boundary, caller } => {
                if matches!(caller, SpriteMainCpuCaller::BossVictory { .. }) {
                    self.complete_victory_module_dialogue_scroll_before_sprite_main();
                }
                // A parked dungeon Sprite_Main whose host is neither a bare
                // hold nor a terminal return: the wire publishes the return
                // inside this host and then holds (route host 877298) or
                // interrupts NMI_PrepareSprites (route host 1185490). The
                // native body owns that return claim and the shared suffix
                // stays with the wire.
                let live_nonterminal_dungeon =
                    matches!(
                        caller,
                        SpriteMainCpuCaller::DungeonModule07
                            | SpriteMainCpuCaller::DungeonModule07Live { .. }
                    ) && matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
                        && self.original_timing_semantic_receipts.is_some()
                        && !typed_main_loop_return;
                let nonterminal_dungeon_sprite_main_claim =
                    live_nonterminal_dungeon && self.original_timing_owes_sprite_main_return();
                if nonterminal_dungeon_sprite_main_claim {
                    self.begin_original_timing_sprite_main_return_claim_scope(1);
                }
                self.complete_sprite_main_after_cpu_boundary(boundary);
                if nonterminal_dungeon_sprite_main_claim {
                    self.finish_original_timing_sprite_main_return_claim_scope();
                }
                if !self
                    .game_execution_scheduler
                    .work_suspends_translated_call_stack()
                {
                    match caller {
                        SpriteMainCpuCaller::DungeonModule07 => {
                            let sprite_return =
                                self.active_dungeon_sprite_main_return.take().expect(
                                    "resumed Sprite_Main must retain its Module 7 return state",
                                );
                            self.complete_module07_after_sprite_main(sprite_return);
                            self.complete_parked_dungeon_sprite_main_suffix_by_wire(
                                live_nonterminal_dungeon,
                            );
                            self.dungeon_quadrant_cpu_continuation_active = false;
                            self.prepare_dungeon_cpu_advance_after_returned_main_wait();
                        }
                        SpriteMainCpuCaller::WorldMapOverlayReload { module09 } => {
                            self.complete_module09_link_oam_caller_return(
                                Module09ItemReceiptCallerReturn {
                                    link_oam: None,
                                    scroll: module09,
                                    rain_already_published: false,
                                    after_sprite_main: Module09AfterSpriteMain::Ordinary,
                                },
                            );
                            if self.pending_main_loop_common_suffix.is_some() {
                                // A source-proven caller return carries the
                                // shared ZeldaRunGameLoop suffix; retire its
                                // one owner.
                                self.complete_pending_main_loop_common_suffix_after_module_return();
                            } else {
                                self.nmi_prepare_sprites();
                                self.clear_nmi_update_latch();
                            }
                        }
                        SpriteMainCpuCaller::Module09 { .. } => {
                            let caller = self
                                .active_module09_sprite_main_return
                                .take()
                                .expect("resumed Sprite_Main lost its ordinary Module09 caller");
                            let oam_dma_source = self.sprite_oam_shadow_buffer().to_vec();
                            if self.complete_resumed_module09_sprite_main_caller_with_wire(
                                caller,
                                input,
                                Some(&oam_dma_source),
                                typed_main_loop_return,
                            ) {
                                return;
                            }
                            if typed_main_loop_return {
                                self.complete_pending_main_loop_common_suffix_after_module_return();
                                return;
                            }
                            self.nmi_prepare_sprites();
                            self.clear_nmi_update_latch();
                        }
                        SpriteMainCpuCaller::DungeonModule07Live { .. } => {
                            let sprite_return = self
                                .active_dungeon_sprite_main_return
                                .take()
                                .expect("resumed live Sprite_Main lost its Module 7 return state");
                            self.complete_module07_after_sprite_main(sprite_return);
                            self.complete_parked_dungeon_sprite_main_suffix_by_wire(
                                live_nonterminal_dungeon,
                            );
                        }
                        SpriteMainCpuCaller::BossVictory { .. }
                        | SpriteMainCpuCaller::SaveAndQuit { .. } => {
                            // Module13's and Module17's callers run only
                            // LinkOam and the shared game-loop suffix after
                            // Sprite_Main (route hosts 103998, 159333).
                            self.link_oam_main();
                            if self.pending_main_loop_common_suffix.is_some() {
                                self.complete_pending_main_loop_common_suffix_after_module_return();
                            } else {
                                self.nmi_prepare_sprites();
                                self.clear_nmi_update_latch();
                            }
                        }
                    }
                }
            }
            GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity {
                position_return,
                iteration,
            } => {
                self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                    GraphicsDmaGeneration::HostBoundaryBeforeMain,
                )));
                self.complete_dungeon_exit_spotlight_link_velocity(position_return, iteration);
            }
            GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity {
                velocity_return,
                iteration,
            } => {
                self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                    GraphicsDmaGeneration::HostBoundaryBeforeMain,
                )));
                self.complete_dungeon_exit_spotlight_link_actual_velocity(
                    velocity_return,
                    iteration,
                    false,
                );
            }
            GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement { iteration } => {
                self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                    GraphicsDmaGeneration::HostBoundaryBeforeMain,
                )));
                self.complete_dungeon_exit_spotlight_link_movement(iteration, false);
            }
            GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel {
                iteration,
                pass,
                pending_pixel_delta,
                old_x,
                old_y,
            } => {
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
                    false,
                );
            }
            GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow {
                iteration,
                pass,
                pending_coordinate_high,
                old_x,
                old_y,
            } => {
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
                    false,
                );
            }
            GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates {
                iteration,
                pass,
                old_x,
                old_y,
            } => {
                self.set_next_display_obj_scanout(Some(ObjScanoutGenerations::coherent(
                    GraphicsDmaGeneration::HostBoundaryBeforeMain,
                )));
                self.complete_dungeon_exit_spotlight_link_movement_after_coordinates(
                    iteration,
                    LinkMovePositionAfterCoordinatesReturn {
                        old_x,
                        old_y,
                        axis: crate::game_state::PlayerAxis::from_rom_pass(pass),
                    },
                    false,
                );
            }
            GameWorkContinuation::FinishDungeonCachedSpriteMain {
                boundary,
                live_slot_backup,
                dungeon,
            } => {
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
                self.dungeon_quadrant_cpu_continuation_active = false;
                self.prepare_dungeon_cpu_advance_after_returned_main_wait();
            }
            GameWorkContinuation::FinishDungeonPushBlocks { dungeon } => {
                self.resume_dungeon_push_blocks_caller(dungeon);
            }
            GameWorkContinuation::FinishDungeonPushBlockHandler { handler_pending } => {
                self.resume_dungeon_push_block_handler(handler_pending);
            }
            GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn => {
                let retains_goal_caller_return = self.dungeon_landing_goal_display_handoff
                    == DungeonLandingGoalDisplayHandoff::RetainCallerReturn;
                if typed_main_loop_return {
                    // The typed return fact separates the saved Module 7 CPU
                    // caller from ZeldaRunGameLoop's unconditional suffix.
                    // Preserve that source order rather than letting the
                    // legacy atomic continuation own both at once.
                    self.complete_dungeon_after_submodule_cpu_caller_return();
                    self.finish_original_timing_sprite_main_return_claim_scope();
                    self.complete_pending_main_loop_common_suffix_after_module_return();
                    self.prepare_dungeon_cpu_advance_after_returned_main_wait();
                } else {
                    self.complete_dungeon_after_submodule_caller_return();
                }
                // The held NMI has resumed the C caller, but the following
                // full-DMA NMI has not run yet. Publish the immutable resident
                // OAM and the Link CHR generation sampled at this host
                // boundary; live post-return buffers belong to the next NMI.
                self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::RetainImmutableCapturedPpu,
                    link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                }));
                let host_boundary_obj_vram = self
                    .pre_main_graphics_dma
                    .as_ref()
                    .map(|graphics| graphics.obj_vram[0x4000..0x4400].to_vec())
                    .unwrap_or_else(|| self.ppu.vram[0x4000..0x4400].to_vec());
                self.next_display_obj_memory_generation =
                    Some(DisplayObjGeneration::RetainCapturedMemory {
                        oam: self.ppu.oam.clone(),
                        vram: host_boundary_obj_vram,
                    });
                // The interrupted C table builder has now returned. Whether
                // its completed capture is current or staged depends on the
                // hardware boundary from which this synchronous call began.
                let publication_override = if retains_goal_caller_return {
                    Some(DisplaySnapshotPublication::RetainPublished)
                } else if self.dungeon_landing_entry_started_after_leading_nmi
                    && started_after_leading_nmi
                {
                    // State 0 began after its field's leading NMI. For this
                    // wipe, caller-return captures are the next hardware
                    // generation; applying the module-wide staged delay again
                    // would make every radius arrive one field late.
                    Some(DisplaySnapshotPublication::PublishCaptured)
                } else {
                    None
                };
                self.capture_display_snapshot_with_override(publication_override);
                if typed_main_loop_return {
                    self.carry_original_timing_scheduled_caller_host_return_from_active_capture();
                }
                if retains_goal_caller_return {
                    self.dungeon_landing_goal_display_handoff =
                        DungeonLandingGoalDisplayHandoff::None;
                }
                if !self
                    .game_execution_scheduler
                    .work_suspends_translated_call_stack()
                {
                    // The resumed Module 7 suffix can itself reach vblank in
                    // the common Sprite_Main call. In that case the C stack is
                    // still suspended inside the nested call; only an idle
                    // scheduler means the dispatcher actually reached the
                    // main wait represented by this CPU phase.
                    self.game_execution_scheduler
                        .finish_call_stack_at_main_wait_before_nmi();
                }
            }
            GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn => {
                // The first NMI for this host landed after Sprite_Main and was
                // consumed immediately before this continuation. Resume the
                // remaining Module 7 suffix without replaying the submodule,
                // Sprite_Main, or frame prefix.
                let sprite_return = self
                    .active_dungeon_sprite_main_return
                    .take()
                    .expect("Link OAM continuation must retain Sprite_Main return state");
                self.complete_module07_after_sprite_main(sprite_return);
                if self.pending_main_loop_common_suffix.is_some() {
                    // The source-proven caller return already carries the
                    // shared ZeldaRunGameLoop suffix; retire its one owner.
                    self.complete_pending_main_loop_common_suffix_after_module_return();
                } else {
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                self.prepare_dungeon_cpu_advance_after_returned_main_wait();
                let native_quadrant_main_wait = self.game_state.frame.submodule == 2
                    && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live);
                if typed_main_loop_return || native_quadrant_main_wait {
                    // The wire proves this host ends at main wait; the next
                    // host's own leading NMI crosses the following boundary.
                    // Native quadrant callers follow the same order: their
                    // resumed LinkOam/HUD/preparation suffix must not consume
                    // its queued uploads with an extra trailing interrupt.
                    if native_quadrant_main_wait {
                        self.game_execution_scheduler
                            .finish_call_stack_at_main_wait_before_nmi();
                    }
                    return;
                }
                // The resumed caller reaches the main wait before the next
                // field boundary. The libretro interval then crosses that
                // following NMI before returning its next video surface.
                let next_oam_dma_source = self.sprite_oam_shadow_buffer().to_vec();
                self.interrupt_nmi(input, Some(&next_oam_dma_source), false);
                self.game_execution_scheduler
                    .mark_main_iteration_after_leading_nmi();
            }
            GameWorkContinuation::FinishModule09LinkOamCallerReturn { caller } => {
                // The prior host's trailing NMI interrupted Module09 inside
                // LinkOam_Main after Sprite_Main returned. Resume the exact C
                // caller suffix without replaying the overlay body or sprites.
                let oam_dma_source = self.sprite_oam_shadow_buffer().to_vec();
                if self.complete_resumed_module09_sprite_main_caller_with_wire(
                    caller,
                    input,
                    Some(&oam_dma_source),
                    typed_main_loop_return,
                ) {
                    return;
                }
                if typed_main_loop_return {
                    self.complete_pending_main_loop_common_suffix_after_module_return();
                    return;
                }
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
            }
            GameWorkContinuation::FinishOverworldHudCallerReturn { inventory, animate_hearts } => {
                self.resume_overworld_hud_inventory(inventory, animate_hearts);
                self.OverworldOverlay_HandleRain();
                self.complete_pending_main_loop_common_suffix_after_module_return();
                self.game_execution_scheduler.finish_call_stack_at_main_wait_before_nmi();
                return;
            }
            GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn { caller } => {
                // The interrupt landed in NMI_PrepareSprites' initial OAM
                // packing loop. The NMI update latch was still set, so that
                // handler skipped the consumer. Finish the idempotent
                // preparation after the handler, clear the latch, and let the
                // following NMI consume the completed operands.
                let interrupted_obj_cache_vram = self
                    .interrupted_nmi_prepare_obj_cache_vram
                    .take()
                    .unwrap_or_else(|| {
                        self.ppu
                            .obj_vram_latch
                            .clone()
                            .unwrap_or_else(|| self.ppu.vram.clone())
                    });
                if self.pending_main_loop_common_suffix.is_some() {
                    // The source-proven caller return already carries the
                    // shared ZeldaRunGameLoop suffix; retire its one owner.
                    self.complete_pending_main_loop_common_suffix_after_module_return();
                } else {
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                // The accepted NMI which suspended NMI_PrepareSprites has not
                // yet published the Link OBJ operands authored by this resumed
                // caller. Keep the resident host-boundary OBJ generation for
                // the scanout being retired; the synthetic trailing NMI below
                // owns the following field.
                let native_dungeon_main_wait = matches!(caller,
                    NmiPrepareSpritesCpuCaller::DungeonModule07 | NmiPrepareSpritesCpuCaller::OverworldModule09
                        | NmiPrepareSpritesCpuCaller::OverworldSongUpload)
                    && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live);
                if !native_dungeon_main_wait {
                    self.stage_resumed_sprite_main_return_obj_scanout();
                }
                if caller == NmiPrepareSpritesCpuCaller::DungeonModule07 {
                    // The dungeon caller has now reached the real main wait.
                    // Preserve a continuous ROM-timed advance from this exact
                    // pre-NMI state for the fresh Module 7 iteration which
                    // begins next host.
                    self.prepare_dungeon_cpu_advance_after_returned_main_wait();
                    if self.dungeon_landing_cpu_advance_pending.is_none() {
                        self.dungeon_landing_cpu_advance_pending =
                            begin_dungeon_cached_sprite_cpu_advance_after_leading_nmi(self);
                    }
                }
                if typed_main_loop_return || native_dungeon_main_wait {
                    // The interrupted dungeon caller retires after its held
                    // NMI. Original quadrant hosts 23203/23205 clear $12 and
                    // wait; their next hosts accept the publishing Open NMI.
                    // Preserve that boundary instead of consuming the upload
                    // with a synthetic trailing NMI in this return host.
                    if native_dungeon_main_wait {
                        // This retained cache belongs to the held return's
                        // already captured field. Carrying it into the next
                        // capture would undo that field's Open-NMI Link DMA.
                        if caller == NmiPrepareSpritesCpuCaller::DungeonModule07 {
                            self.display_snapshot.as_mut()
                                .expect("native preparation return requires its captured display")
                                .explicit_obj_cache_vram = Some(interrupted_obj_cache_vram);
                        }
                        self.game_execution_scheduler
                            .finish_call_stack_at_main_wait_before_nmi();
                    } else {
                        self.next_display_obj_cache_vram = Some(interrupted_obj_cache_vram);
                    }
                    return;
                }
                let next_oam_dma_source = self.sprite_oam_shadow_buffer().to_vec();
                self.interrupt_nmi(input, Some(&next_oam_dma_source), false);
                // The trailing NMI's OBJ DMA is now complete, but a later NMI
                // in the same coarse host interval may already begin authoring
                // another generation. Preserve this completed publication as
                // the immutable input to the following scanout.
                self.next_display_obj_memory_generation =
                    Some(DisplayObjGeneration::RetainCapturedMemory {
                        oam: self.ppu.oam.clone(),
                        vram: self.ppu.vram[0x4000..0x4400].to_vec(),
                    });
                self.next_display_obj_cache_vram = Some(interrupted_obj_cache_vram);
                if self.debug_obj_pipe_enabled() {
                    let cache = self.next_display_obj_cache_vram.as_deref().unwrap();
                    self.debug_obj_pipe("semantic_cache_queued", &cache[0x4000..0x4400]);
                }
                self.game_execution_scheduler
                    .mark_main_iteration_after_leading_nmi();
            }
            continuation => {
                panic!("unsupported post-trailing-NMI continuation {continuation:?}")
            }
        }
    }

    pub(super) fn prepare_audio_nmi_for_main_boundary(
        &mut self,
        frame: crate::game_state::FrameState,
    ) {
        if !self.rom_startup_timing() {
            return;
        }
        if self.resumed_dungeon_caller_audio_follows_host_publication() {
            return;
        }
        let debug_audio_nmi = crate::debug_env::var("ZELDA3_DEBUG_AUDIO_NMI_FRAME")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            == Some(self.frame_ctr_dbg);
        if debug_audio_nmi {
            eprintln!(
                "audio_nmi_prepare host={} phase={:02x}/{:02x}/{:02x} prior={} work={:?} ambient={:02x} effect1={:02x} effect2={:02x}",
                self.frame_ctr_dbg,
                frame.main_module,
                frame.submodule,
                frame.subsubmodule,
                self.audio_nmi_processed_before_main,
                self.game_execution_scheduler.current_work(),
                self.game_state.system_signals.ambient_sound_effect(),
                self.game_state.system_signals.sound_effect_1(),
                self.game_state.system_signals.sound_effect_2(),
            );
        }
        if !self.audio_nmi_processed_before_main {
            self.interrupt_nmi_audio_parts();
            self.audio_nmi_processed_before_main = true;
        }
        if debug_audio_nmi {
            eprintln!(
                "audio_nmi_prepared host={} processed={}",
                self.frame_ctr_dbg, self.audio_nmi_processed_before_main,
            );
        }
    }

    pub(super) fn hdma_channel_targets_window_latches(channel: &DmaChannel) -> bool {
        let mode = usize::from(channel.mode & 7);
        SIMPLE_HDMA_B_ADR_OFFSETS[mode][..SIMPLE_HDMA_TRANSFER_LENGTH[mode]]
            .iter()
            .any(|offset| matches!(channel.b_adr.wrapping_add(*offset), 0x26..=0x29))
    }

    pub(super) fn hdma_state_targets_window_latches(enable_mask: u8, dma: &DmaState) -> bool {
        dma.channel.iter().enumerate().any(|(index, channel)| {
            enable_mask & (1 << index) != 0 && Self::hdma_channel_targets_window_latches(channel)
        })
    }

    pub(super) fn final_window_latches_after_scanout(
        snapshot: &DisplaySnapshot,
    ) -> Option<[u8; 4]> {
        let captured_targets_window = Self::hdma_state_targets_window_latches(
            snapshot.ram[crate::game_state::constants::HDMAEN_COPY],
            &snapshot.dma,
        );
        let spotlight_targets_window = match &snapshot.spotlight_scanout_generation {
            SpotlightScanoutGeneration::ComposeLiveAfterNmi(live) => {
                let mut dma = snapshot.dma.clone();
                dma.channel[6..8].copy_from_slice(&live.dma_channels);
                Self::hdma_state_targets_window_latches(live.hdma_enable_mask, &dma)
            }
            SpotlightScanoutGeneration::CapturedBeforeNmi => false,
        };
        if !captured_targets_window && !spotlight_targets_window {
            return None;
        }

        let mut ram = snapshot.ram.clone();
        let mut dma = snapshot.dma.clone();
        snapshot
            .spotlight_scanout_generation
            .compose_hdma_into(&mut ram, &mut dma);
        snapshot.hdma_table_generation.compose_into(&mut ram);

        let enable_mask = ram[crate::game_state::constants::HDMAEN_COPY];
        let mut channels = dma.channel;
        let mut hdma: [SimpleHdma; 8] = std::array::from_fn(|_| SimpleHdma::default());
        let mut active = [false; 8];
        for index in 0..8 {
            channels[index].hdma_active = enable_mask & (1 << index) != 0;
            active[index] = channels[index].hdma_active
                && Self::hdma_channel_targets_window_latches(&channels[index]);
            if active[index] {
                Self::simple_hdma_init_from_ram(&ram, &mut hdma[index], &channels[index]);
            }
        }
        if !active.iter().any(|enabled| *enabled) {
            return None;
        }

        let mut latches = [
            snapshot.ppu.window1_left,
            snapshot.ppu.window1_right,
            snapshot.ppu.window2_left,
            snapshot.ppu.window2_right,
        ];
        // The scanline renderer performs the field's line-zero transfer before
        // its first visible row and the row-224 transfer after visible scanout.
        // Preserve the physical latch left by that complete field.
        for _ in 0..=224 {
            for index in 0..8 {
                if !active[index] {
                    continue;
                }
                let (writes, write_count) = Self::simple_hdma_line_writes(&ram, &mut hdma[index]);
                for (address, value) in writes.into_iter().take(write_count) {
                    if let 0x2126..=0x2129 = address {
                        latches[(address - 0x2126) as usize] = value;
                    }
                }
            }
        }
        Some(latches)
    }

    /// Retiring HDMA leaves its final register values in the physical PPU.
    ///
    /// The immutable display snapshot owns the table generation hardware just
    /// consumed. Replaying the live WRAM table here would be wrong because the
    /// CPU may already be authoring the following scanout.
    pub(super) fn commit_retiring_display_window_latches(&mut self) {
        let latches = self
            .display_snapshot
            .as_ref()
            .and_then(|snapshot| Self::final_window_latches_after_scanout(snapshot));
        let Some(latches) = latches else {
            return;
        };
        for (offset, value) in latches.into_iter().enumerate() {
            self.zelda_ppu_write(0x2126 + offset as u32, value);
        }
    }

    /// How many hosts the ROM's polyhedral thread needs to render the next
    /// frame from the current RAM, measured by executing the thread's render
    /// call in the ROM CPU shadow across the thread's real slots. The NMI
    /// and the V-IRQ each swap threads unconditionally, so the thread owns
    /// the lines from the NMI handler's return to the IRQ at `virq_trigger`
    /// (144 in the Triforce room): its first slot begins at line 0 after the
    /// handler uploaded the previous frame, later slots at line ~249.8 of the
    /// preceding field, and each ends at line 144. The budget charges the
    /// DRAM refresh and the HDMA stalls the thread suffers on the way (route
    /// hosts 1557812-1563400: shadow exact to 0.5% before HDMA, 5% short
    /// without the stalls after host ~1558400). Returns the shadow's raw
    /// master cycles too for diagnostics.
    /// Master cycles from the NMI vector to the handler's thread swap at
    /// `$00:82C7`, executing the ROM's NMI handler in the shadow over the
    /// current RAM with the poly upload flag forced (the handler's uploads
    /// decide when the poly thread resumes: line ~239.9 bare, later with the
    /// BG3 text and poly-buffer DMAs; route hosts 1557815-1558362).
    pub(super) fn rom_triforce_nmi_swap_master_cycles(
        &self,
        poly_upload_pending: bool,
    ) -> Option<u64> {
        self.rom_nmi_swap_master_cycles(poly_upload_pending, None)
    }

    /// Master cycles from NMI entry ($00:80C9) to the thread swap ($00:82C7)
    /// on the current RAM, with the poly-upload byte and optionally the
    /// main-loop latch byte ($12) overridden to what the NMI saw.
    pub(super) fn rom_nmi_swap_master_cycles(
        &self,
        poly_upload_pending: bool,
        latch_held: Option<bool>,
    ) -> Option<u64> {
        // Run through the thread-swap sequence ($82C7 REP/TSC/TAX/LDA/TCS/STX
        // and the PLB/PLD/PLY/PLX/PLA pops) up to the RTI at $82D7; the RTI
        // itself and the resumed thread's idle-loop check are added below.
        const NMI_HANDLER_CHECKPOINT: RomCpuCheckpoint = RomCpuCheckpoint {
            entry_pc: 0x00_80c9,
            stop_pc: 0x00_82d7,
            a: 0,
            x: 0,
            y: 0,
            sp: 0x01f8,
            dp: 0,
            db: 0,
            carry: false,
            zero: true,
            overflow: false,
            negative: false,
            interrupt_disable: false,
            decimal: false,
            accumulator_is_8_bit: true,
            index_is_8_bit: true,
            emulation: false,
            waiting: false,
            stack_address: 0x01f9,
            stack_bytes: &[],
        };
        let timing_dma = self.dma_with_native_hdma_enable();
        let mut ram_image = self.ram.to_vec();
        ram_image[0x1f0c] = if poly_upload_pending { 0xff } else { 0 };
        if let Some(latch_held) = latch_held {
            ram_image[0x12] = u8::from(latch_held);
        }
        let mut run = RomCpuTimingRun::new(
            &self.rom,
            &ram_image,
            &self.sram,
            &self.ppu,
            &timing_dma,
            self.zelda_audio_apu_output_ports(),
            NMI_HANDLER_CHECKPOINT,
        )
        .ok()?;
        // RTI: opcode fetch (slow ROM, 8) + one internal cycle (6) + five
        // stack pops from WRAM (8 each) = 54 master cycles.
        const RTI_MASTER_CYCLES: u64 = 54;
        // The run's V=225 origin is the NMI acceptance; the handler's first
        // instruction at $00:80C9 executes ~78 master cycles later (interrupt
        // sequence: internal cycles, four stack pushes, vector fetch —
        // oracle frame events show pc=80c9 at cycles 74-82).
        const NMI_ENTRY_MASTER_CYCLES: u64 = 78;
        let mut master = NMI_ENTRY_MASTER_CYCLES;
        for _ in 0..200_000 {
            if run.is_complete() {
                return Some(master + RTI_MASTER_CYCLES);
            }
            master += u64::from(run.step().master_cycles);
            master += u64::from(run.drain_started_dma_master_cycles());
        }
        None
    }

    pub(super) fn zelda_run_game_loop_after_leading_nmi(&mut self) {
        self.game_execution_scheduler
            .mark_main_iteration_after_leading_nmi();
        self.zelda_run_game_loop();
        self.stage_pending_dialogue_scroll_completion_after_captured_boundary();
    }

    pub(super) fn nmi_prepare_sprites_for_main_loop(&mut self) {
        self.nmi_prepare_sprites();
        self.main_loop_sprite_preparation_completed = true;
    }

    pub(super) fn nmi_prepare_sprites_for_main_loop_once(&mut self) {
        if !self.main_loop_sprite_preparation_completed {
            self.nmi_prepare_sprites_for_main_loop();
        }
    }
}
