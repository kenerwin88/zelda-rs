//! Module 07 dispatch, player control, camera, and room maintenance.

use super::*;

impl ZeldaState {

    /// ROM: `Mirror_SaveRoomData`
    pub(in crate::zelda_rtl) fn mirror_save_room_data(&mut self) {
        if self.game_state.inventory.save_progress.palace_index_x2() == 0xff {
            self.set_sound_effect_1(60);
            return;
        }
        self.set_submodule(25);
        self.set_subsubmodule(0);
        self.set_sound_effect_1(51);
        self.dungeon_flag_room_data_quadrants();
        self.SaveDungeonKeys();
    }

    /// ROM: `Dung_TagRoutine_0x00`
    pub(super) fn dung_tag_routine_0x00(&mut self, _k: usize) {
        let _scope = crate::cycle_ledger::routine(0x01_c328);
        crate::cycle_ledger::charge(42); // RTS, including the empty tag's return.
    }

    pub(in crate::zelda_rtl) fn module07_dungeon(&mut self) {
        if self.try_native_dungeon_entry() {
            self.run_native_intro_clear_until_boundary();
            return;
        }
        crate::cycle_ledger::charge_routine(0x02_87a2, 84); // SEP #$30 : JSL layer effect.
        self.dungeon_handle_layer_effect();
        self.replay_trace_ram_watch("module07-after-layer-effect");
        crate::cycle_ledger::charge_routine(0x02_87a2, 114); // LDA $11 : ASL : TAX : JSR (table,X).
        self.run_dungeon_submodule();
        self.replay_trace_ram_watch("module07-after-submodule");
        if !self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            self.arm_live_cached_sprite_main_cpu_continuation();
        }
        if !self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
            && self
                .take_original_timing_main_loop_interruption(crate::MainLoopInterruption::LinkOam)
        {
            // The authority observed this completed submodule's shared Link
            // OAM caller phase being interrupted. Reuse the existing semantic
            // continuation so the following host resumes that phase instead
            // of beginning the next dungeon state.
            self.dungeon_post_sprite_main_return_pending = true;
        }
        if !self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
            && (self.take_original_timing_main_loop_interruption(
                crate::MainLoopInterruption::SpritePreparation,
            ) || self
                .take_original_timing_extended_oam_packing_interruption_for_caller_return())
        {
            // The authority observed this completed submodule's shared
            // NMI_PrepareSprites caller suffix being interrupted. Reuse the
            // existing semantic continuation so the following host resumes
            // that suffix instead of beginning the next dungeon state.
            self.dungeon_nmi_prepare_sprites_return_pending = true;
        }
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            return;
        }
        self.complete_module07_dungeon_after_submodule();
        self.complete_module07_dungeon_after_submodule_caller();
    }

    pub(in crate::zelda_rtl) fn complete_module07_dungeon_after_submodule_caller(&mut self) {
        if std::mem::take(&mut self.dungeon_nmi_prepare_sprites_return_pending) {
            debug_assert!(self.game_execution_scheduler.is_idle());
            // A fresh iteration's forwarded extended-OAM-packing interruption
            // (route host 767104) is owned by this same caller return.
            let _ = self.take_original_timing_extended_oam_packing_interruption_for_caller_return();
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn {
                    caller: NmiPrepareSpritesCpuCaller::DungeonModule07,
                },
                1,
            );
            return;
        }
        if std::mem::take(&mut self.dungeon_state_12_caller_suffix_nmi_pending) {
            if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
                // The measured CPU stopped before preparation, so retain the
                // whole common suffix for its return after the held handler.
                self.pending_main_loop_common_suffix = Some(
                    MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch,
                );
            }
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishCpuInstructionNmi {
                    // The module body returned immediately before
                    // Main_PrepSpritesForNmi. After the interrupt, finish
                    // that real caller suffix and wait for the following
                    // leading NMI before starting a fresh dungeon iteration.
                    resume: PreMainNmiResume::DungeonSupertileCallerReturnNmi,
                },
                1,
            );
            return;
        }
        if self.dungeon_state_13_atomic_caller_return_publication_host_frame
            == Some(self.frame_ctr_dbg)
        {
            debug_assert!(self.game_execution_scheduler.is_idle());
            if self.game_execution_scheduler.is_idle() {
                // This measured body and its common suffix retire within one
                // CPU slice, after the NMI request they consumed.
                // This is the atomic publication's gate update; the enclosing
                // main caller still executes sprite preparation after this
                // module returns, and owns its own return work there.
                self.clear_nmi_update_latch();
            }
        }
    }

    pub(in crate::zelda_rtl) fn complete_module07_dungeon_after_submodule(&mut self) {
        if let Some(tail)=self.early_dungeon_push_caller_tail.take() {
            self.finish_native_dungeon_push_handler(tail);
            self.complete_module07_dungeon_after_push_block_handler(true);
            return;
        }

        let runs_push_block_handler = !self
            .game_state
            .enhanced_features
            .has(FEATURE_MISC_BUG_FIXES_DUNGEON)
            || self.game_state.frame.main_module == 7;
        if runs_push_block_handler {
            crate::cycle_ledger::charge_routine(0x02_87a2, 94); // STZ $042C : JSL push-block handler.
            self.dungeon_object_tracking_mut().clear_misc_object_index();
            if let Some(next_index) = self.take_original_timing_dungeon_push_blocks_in_progress() {
                // The wire's host boundary fell inside Dungeon_PushBlock_Handler:
                // run the misc objects it completed and park the remainder of
                // the loop with the Module 7 tail behind it.
                self.dungeon_push_block_handler_until(next_index);
                self.game_execution_scheduler.schedule_work(
                    GameWorkContinuation::FinishDungeonPushBlockHandler {
                        handler_pending: true,
                    },
                    1,
                );
                return;
            }
            self.dungeon_push_block_handler();
            self.replay_trace_ram_watch("module07-after-push-blocks");
            if self.take_original_timing_dungeon_push_blocks_handled() {
                // The wire's boundary fell right after the handler returned
                // (inside OrientLampLightCone's guard): park the tail.
                self.game_execution_scheduler.schedule_work(
                    GameWorkContinuation::FinishDungeonPushBlockHandler {
                        handler_pending: false,
                    },
                    1,
                );
                return;
            }
        }
        self.complete_module07_dungeon_after_push_block_handler(runs_push_block_handler);
    }

    /// Module07_Dungeon after Dungeon_PushBlock_Handler: the submodule-0
    /// room maintenance, the lamp cone and scroll copies, then push-block
    /// drawing and the Sprite_Main caller (or the parked draw work).
    pub(in crate::zelda_rtl) fn complete_module07_dungeon_after_push_block_handler(
        &mut self,
        runs_push_block_handler: bool,
    ) {
        use crate::cycle_models::dungeon_after_push::DungeonAfterPushTail;
        let mut lamp_completed = false;
        let mut saved_sprite_scroll = None;
        let mut drawing_completed = false;
        if let Some(tail) = self.early_dungeon_after_push_tail.take() {
            match tail {
                DungeonAfterPushTail::MainWait => return,
                DungeonAfterPushTail::Lamp => {}
                DungeonAfterPushTail::AfterLamp => {
                    lamp_completed = true;
                }
                DungeonAfterPushTail::BeforeDrawing(saved)
                | DungeonAfterPushTail::AfterDrawing(saved)
                | DungeonAfterPushTail::SpritePrefix {saved,..} => {
                    drawing_completed = matches!(tail, DungeonAfterPushTail::AfterDrawing(_)|DungeonAfterPushTail::SpritePrefix {..});
                    if let DungeonAfterPushTail::SpritePrefix {tail,..}=tail {self.early_dungeon_sprite_prefix_tail=Some(tail);}
                    lamp_completed = true;
                    saved_sprite_scroll = Some(DungeonSpriteMainReturn {
                        bg2_x: saved.bg2_x,
                        bg2_y: saved.bg2_y,
                        bg1_x: saved.bg1_x,
                        bg1_y: saved.bg1_y,
                        link_oam: None,
                    });
                }
                DungeonAfterPushTail::RoomTags(tail) => self.finish_native_room_tags(tail),
                DungeonAfterPushTail::Interactions(tail) => {
                    self.finish_native_dungeon_interactions(tail)
                }
                DungeonAfterPushTail::Edge(tail) => self.finish_native_dungeon_edge(tail),
                DungeonAfterPushTail::AfterCamera => {
                    self.complete_module07_room_maintenance_after_camera()
                }
                DungeonAfterPushTail::HalfSlotBody => {
                    let request = self.game_state.display.chr_halfslot_request;
                    assert_ne!(request, 0);
                    self.graphics_load_chr_half_slot_after_request(request);
                    crate::cycle_ledger::charge_routine(0x02_87a2, 46);
                    self.dungeon_handle_camera();
                    self.replay_trace_ram_watch("module07-after-camera");
                    self.complete_module07_room_maintenance_after_camera();
                }
            }
        } else if runs_push_block_handler {
            crate::cycle_ledger::charge_routine(0x02_87a2, 40); // 87B6: LDA $11 : BNE.
            if self.game_state.frame.submodule == 0 {
                crate::cycle_ledger::charge_routine(0x02_87a2, 62);
                self.graphics_load_chr_half_slot();
                crate::cycle_ledger::charge_routine(0x02_87a2, 46);
                self.dungeon_handle_camera();
                self.replay_trace_ram_watch("module07-after-camera");
                self.complete_module07_room_maintenance_after_camera();
            } else {
                crate::cycle_ledger::charge_routine(0x02_87a2, 6);
            }
        }

        if !lamp_completed {
            crate::cycle_ledger::charge_routine(0x02_87a2, 62); // JSL OrientLampLightCone.
            self.orient_lamp_light_cone();
        }
        self.replay_trace_ram_watch("module07-after-lamp");

        let sprite_return =
            saved_sprite_scroll.unwrap_or_else(|| self.begin_module07_sprite_scroll());
        // SEP #$20 : JSL push-block drawing. A parked draw has entered the
        // callee; its resumed caller must not charge this call again.
        if !drawing_completed {
            crate::cycle_ledger::charge_routine(0x02_87a2, 84);
            if self.take_original_timing_dungeon_push_blocks_pending() {
                self.game_execution_scheduler.schedule_work(
                    GameWorkContinuation::FinishDungeonPushBlocks {
                        dungeon: sprite_return,
                    },
                    1,
                );
                return;
            }
            self.sprite_dungeon_draw_all_push_blocks();
        }
        self.replay_trace_ram_watch("module07-after-draw-push-blocks");
        self.run_module07_sprite_main_caller(sprite_return);
    }

    /// Resume room tags, doors and screen-edge checks after the camera return.
    fn complete_module07_room_maintenance_after_camera(&mut self) {
        crate::cycle_ledger::charge_routine(0x02_87a2, 40); // 87C1: LDA $11 : BNE.
        if self.game_state.frame.submodule == 0 {
            crate::cycle_ledger::charge_routine(0x02_87a2, 62);
            self.dungeon_handle_room_tags();
            self.replay_trace_ram_watch("module07-after-room-tags");
            self.complete_module07_room_maintenance_after_room_tags();
        } else {
            crate::cycle_ledger::charge_routine(0x02_87a2, 6);
        }
    }

    pub(super) fn complete_module07_room_maintenance_after_room_tags(&mut self) {
        crate::cycle_ledger::charge_routine(0x02_87a2, 40); // 87C9: LDA $11 : BNE.
        if self.game_state.frame.submodule == 0 {
            crate::cycle_ledger::charge_routine(0x02_87a2, 62);
            self.dungeon_process_torches_and_doors();
            self.replay_trace_ram_watch("module07-after-torches-doors");
            self.complete_module07_room_maintenance_after_interactions();
        } else {
            crate::cycle_ledger::charge_routine(0x02_87a2, 6);
        }
    }

    pub(super) fn finish_native_dungeon_edge(
        &mut self,
        tail: crate::cycle_models::dungeon_edge_caller::EdgeTail,
    ) {
        use crate::cycle_models::dungeon_edge_caller::EdgeTail;
        match tail {
            EdgeTail::Lamp => {}
            EdgeTail::BlastWall => {
                self.dungeon_clear_away_exploding_wall();
                self.replay_trace_ram_watch("module07-after-blast-wall");
                self.complete_module07_room_maintenance_after_blast_wall();
            }
            EdgeTail::Movement { dir } => {
                let _scope = crate::cycle_ledger::routine(0x02_885e);
                self.dungeon_handle_edge_transition_movement(dir);
                self.dungeon_finish_screen_edge_after_movement();
                self.replay_trace_ram_watch("module07-after-screen-edge");
            }
        }
    }

    pub(super) fn complete_module07_room_maintenance_after_interactions(&mut self) {
        crate::cycle_ledger::charge_routine(0x02_87a2, 48); // LDA $0454 : BEQ.
        if self.game_state.dungeon.room_effects.crush_wall_progress() & 255 != 0 {
            crate::cycle_ledger::charge_routine(0x02_87a2, 62);
            self.dungeon_clear_away_exploding_wall();
            self.replay_trace_ram_watch("module07-after-blast-wall");
        } else {
            crate::cycle_ledger::charge_routine(0x02_87a2, 6);
        }
        self.complete_module07_room_maintenance_after_blast_wall();
    }

    fn complete_module07_room_maintenance_after_blast_wall(&mut self) {
        crate::cycle_ledger::charge_routine(0x02_87a2, 40); // LDA $6C : BNE.
        if self.game_state.player.follower_link.doorway_state() == 0 {
            crate::cycle_ledger::charge_routine(0x02_87a2, 46);
            self.dungeon_try_screen_edge_transition();
            self.replay_trace_ram_watch("module07-after-screen-edge");
        } else {
            crate::cycle_ledger::charge_routine(0x02_87a2, 6);
        }
    }
    /// Save the caller's scroll words and publish the sprite drawing view.
    /// The returned words own restoration after Sprite_Main actually returns.
    pub(super) fn begin_module07_sprite_scroll(&mut self) -> DungeonSpriteMainReturn {
        crate::cycle_ledger::charge_routine(0x02_87a2, 840); // 87E5..881C, through the collision test.
        let bg2x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg2y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        let bg1x = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
        let bg1y = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();

        let bg1_x_offset = self.game_state.world.scroll.bg1_x_offset();
        let bg1_y_offset = self.game_state.world.scroll.bg1_y_offset();
        self.set_bg1_bg2_live_and_copy(
            bg2x.wrapping_add(bg1_x_offset),
            bg2y.wrapping_add(bg1_y_offset),
            bg1x.wrapping_add(bg1_x_offset),
            bg1y.wrapping_add(bg1_y_offset),
        );

        let mut bg1x_restore = bg1x;
        let mut bg1y_restore = bg1y;
        if self
            .game_state
            .dungeon
            .room_load
            .header_collision_2_mirror()
            != 0
        {
            crate::cycle_ledger::charge_routine(0x02_87a2, 448); // Replace the saved BG1 words with moving-floor coordinates.
            bg1x_restore = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add(self.game_state.dungeon.moving_floor.floor_x_offset());
            bg1y_restore = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_add(self.game_state.dungeon.moving_floor.floor_y_offset());
            self.set_bg1_x(bg1x_restore);
            self.copy_bg1_live_to_ppu_copy();
            self.set_bg1_y(bg1y_restore);
            self.copy_bg1_live_to_ppu_copy();
        } else {
            crate::cycle_ledger::charge_routine(0x02_87a2, 6); // Taken BEQ skips moving-floor work.
        }

        DungeonSpriteMainReturn {
            bg2_x: bg2x,
            bg2_y: bg2y,
            bg1_x: bg1x_restore,
            bg1_y: bg1y_restore,
            link_oam: None,
        }
    }

    /// Resume the existing Module 7 caller at its `Sprite_Main` statement.
    ///
    /// A continuous source host can remain inside the same C call stack for
    /// more than one vblank. In that case the translated submodule and the
    /// common dungeon prefix have already run; the source interruption receipt
    pub(in crate::zelda_rtl) fn run_module07_sprite_main_caller(
        &mut self,
        sprite_return: DungeonSpriteMainReturn,
    ) {
        if self.early_dungeon_sprite_prefix_tail.is_none() {
            crate::cycle_ledger::charge_routine(0x02_87a2, 62); // 883E: JSL Sprite_Main, once on entry.
        }
        self.active_dungeon_sprite_main_return = Some(sprite_return);
        if let Some((progress, boundary)) =
            self.take_native_exact_cpu_follower_body_draw_boundary()
        {
            self.arm_authoritative_sprite_main_cpu_continuation(
                progress,
                SpriteMainCpuCaller::DungeonModule07Live { boundary },
            );
        } else if self.original_timing_ancilla_item_receipt_suspended() {
            // The falling milestone item's receipt suspended inside
            // Sprite_Main's prefix before any slot ran (route host 1142850:
            // SpriteMainProgressed(BeforeFirstSlot) with the ancilla receipt
            // suspended). Run natively; the receipt call suspends the stack,
            // and the BeforeFirstSlot restatements belong to that suspension.
            let _ = self.take_original_timing_sprite_main_progress();
            let _ = self.take_forwarded_original_timing_main_loop_interruption(
                crate::MainLoopInterruption::SpriteMainBeforeFirstSlot,
            );
            let _ = self.take_original_timing_main_loop_interruption(
                crate::MainLoopInterruption::SpriteMainBeforeFirstSlot,
            );
        } else if let Some(slot) = self.original_timing_direct_item_receipt_suspended_slot() {
            // The wire's Sprite_Main checkpoint names the last completed slot,
            // but the host ended inside a lower slot's synchronous item
            // receipt (route host 968295: SpriteMainProgressed(BeforeFirstSlot)
            // with slot 15's receipt suspended). Run the loop natively; the
            // receipt call suspends it at the exact statement.
            let checkpoint = self.take_original_timing_sprite_main_progress();
            debug_assert!(
                checkpoint.is_none_or(|boundary| match boundary {
                    crate::zelda_rtl::SpriteMainCpuBoundary::BeforeFirstSlot => slot == 15,
                    crate::zelda_rtl::SpriteMainCpuBoundary::AfterSlot(completed) => {
                        slot + 1 == completed
                    }
                    _ => true,
                }),
                "a direct item receipt suspended in a slot the Sprite_Main checkpoint had not reached",
            );
        } else if let Some((boundary, resume_boundary)) =
            self.take_original_timing_sprite_main_boundary_for_fresh_caller()
        {
            self.arm_authoritative_sprite_main_cpu_continuation(
                boundary,
                SpriteMainCpuCaller::DungeonModule07Live {
                    boundary: resume_boundary,
                },
            );
        } else if let Some(boundary) = self.take_original_timing_sprite_main_progress() {
            // SCAN_KEYS may return the host while the source remains inside
            // Sprite_Main without accepting an NMI. The typed host-boundary
            // receipt owns the same C continuation as an NMI interruption;
            // no CPU address, frame, or room identity enters gameplay.
            self.arm_authoritative_sprite_main_cpu_continuation(
                boundary,
                SpriteMainCpuCaller::DungeonModule07Live {
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            );
        } else if let Some(boundary) =
            self.original_timing_fresh_iteration_interrupted_before_sprite_main()
        {
            // The wire holds no Sprite_Main return or checkpoint for this
            // fresh iteration: the ROM's iteration was interrupted inside
            // Sprite_Main's shared prefix before any slot returned (route host
            // 91639 with a trailing Held acceptance; route host 1409333's
            // Module07/$16 host simply ends mid-iteration). Suspend at the loop
            // entry and resume at that boundary.
            self.arm_authoritative_sprite_main_cpu_continuation(
                crate::zelda_rtl::SpriteMainCpuBoundary::BeforeFirstSlot,
                SpriteMainCpuCaller::DungeonModule07Live { boundary },
            );
        }
        self.sprite_main();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            return;
        }
        let sprite_return = self
            .active_dungeon_sprite_main_return
            .take()
            .expect("Module 7 sprite return frame must remain active");
        self.replay_trace_ram_watch("module07-after-sprite-main");
        if std::mem::take(&mut self.dungeon_post_sprite_main_return_pending) {
            let mut sprite_return = sprite_return;
            let mut stair_progress = None;
            if let Some(receipts) = self.original_timing_semantic_receipts.as_mut() {
                receipts.semantic.retain(|receipt| {
                    if let OriginalTimingSemanticReceipt::LinkOamStairProgress(progress) = *receipt
                    {
                        assert!(stair_progress.is_none(), "stair drawing prefix replayed");
                        stair_progress = Some(progress);
                        false
                    } else {
                        true
                    }
                });
            }
            if let Some(progress) = stair_progress {
                self.restore_module07_scroll_before_link_oam(&sprite_return);
                sprite_return.link_oam = Some(player_oam::LinkOamContinuation::Equipment(match progress {
                    crate::LinkOamStairProgress::PoseSelected => {
                        self.link_oam_after_pose_selection()
                    }
                    crate::LinkOamStairProgress::EquipmentSelection => {
                        self.link_oam_before_equipment()
                    }
                    crate::LinkOamStairProgress::BodySelection => {
                        let continuation = self.link_oam_before_equipment();
                        self.link_oam_before_body(continuation)
                    }
                    crate::LinkOamStairProgress::ShadowSelection => {
                        let continuation = self.link_oam_before_equipment();
                        self.link_oam_before_shadow(continuation)
                    }
                }));
            }
            self.active_dungeon_sprite_main_return = Some(sprite_return);
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn,
                1,
            );
            return;
        }
        self.complete_module07_after_sprite_main(sprite_return);
    }

    /// Refine a parked LinkOam caller through its graphics-index store, keeping
    /// the native drawing locals and the existing NMI/caller ownership.
    pub(in crate::zelda_rtl) fn refine_dungeon_link_oam_to_body_selection(&mut self) {
        self.refine_dungeon_link_oam_body_progress(None);
    }

    pub(in crate::zelda_rtl) fn refine_dungeon_link_oam_body_progress(
        &mut self, progress: Option<player_oam::LinkOamLowerBodyProgress>,
    ) {
        if self.game_execution_scheduler.current_work()
            != Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn) {
            return;
        }
        let mut sprite_return = self.active_dungeon_sprite_main_return
            .expect("parked dungeon LinkOam lost its caller");
        use player_oam::LinkOamContinuation::{Body, Equipment};
        let continuation = match sprite_return.link_oam {
            Some(continuation) => continuation,
            None => {
                self.restore_module07_scroll_before_link_oam(&sprite_return);
                let continuation = self.link_oam_before_equipment();
                Equipment(self.link_oam_before_body(continuation))
            }
        };
        sprite_return.link_oam = Some(match (continuation, progress) {
            (Equipment(equipment), Some(progress)) =>
                Body(self.link_oam_before_lower_body_progress(equipment, progress)),
            (Body(body), Some(progress)) =>
                Body(self.advance_link_oam_lower_body_progress(body, progress)),
            (continuation, None) => continuation,
        });
        self.active_dungeon_sprite_main_return = Some(sprite_return);
    }

    pub(in crate::zelda_rtl) fn complete_module07_after_sprite_main(
        &mut self,
        sprite_return: DungeonSpriteMainReturn,
    ) {
        self.complete_module07_link_oam_after_sprite_main(sprite_return);
        self.complete_module07_hud_after_link_oam();
    }

    pub(super) fn restore_module07_scroll_before_link_oam(&mut self, sprite_return: &DungeonSpriteMainReturn) {
        // 8842..8852: REP, four PLA/STA pairs, SEP and JSL LinkOam_Main.
        // A retained LinkOam continuation already owns this completed prefix.
        crate::cycle_ledger::charge_routine(0x02_87a2, 378);
        self.set_bg2_x(sprite_return.bg2_x);
        self.set_bg2_y(sprite_return.bg2_y);
        self.set_bg1_x(sprite_return.bg1_x);
        self.set_bg1_y(sprite_return.bg1_y);
    }

    /// LinkOam_Main returns before Module 7 enters Hud_RefillLogic.
    pub(in crate::zelda_rtl) fn complete_module07_link_oam_after_sprite_main(
        &mut self,
        sprite_return: DungeonSpriteMainReturn,
    ) {
        if let Some(continuation) = sprite_return.link_oam {
            match continuation {
                player_oam::LinkOamContinuation::Equipment(equipment) =>
                    self.link_oam_after_equipment(equipment),
                player_oam::LinkOamContinuation::Body(body) =>
                    self.link_oam_after_lower_body(body),
            }
        } else {
            self.restore_module07_scroll_before_link_oam(&sprite_return);
            self.link_oam_main();
        }
        self.replay_trace_ram_watch("module07-after-link-oam");
    }

    pub(in crate::zelda_rtl) fn complete_module07_hud_after_link_oam(&mut self) {
        crate::cycle_ledger::charge_routine(0x02_87a2, 62); // 8856: JSL Hud_RefillLogic_Far.
        self.hud_refill_logic();
        self.complete_module07_after_hud_refill();
    }

    pub(in crate::zelda_rtl) fn complete_module07_after_hud_refill(&mut self) {
        self.replay_trace_ram_watch("module07-after-refill");
        crate::cycle_ledger::charge_routine(0x02_87a2, 32); // 885A: JML Hud_FloorIndicator.
        self.hud_floor_indicator();
        self.replay_trace_ram_watch("module07-after-floor-indicator");
        if std::mem::take(&mut self.spiral_stair_return_player_control_pending) {
            // The original main loop reaches its epilogue, then re-enters the
            // next game-loop OAM generation and player-control path before the
            // next NMI. Clear the shadow buffer exactly where that re-entry
            // would: otherwise equipment slots not authored by the second
            // Link draw leak the preceding generation into scanout.
            self.spiral_stair_return_oam_publication_host_frame = Some(self.frame_ctr_dbg);
            self.spiral_stair_return_player_oam_scanout = Some(std::array::from_fn(|word| {
                read_le_u16(&self.ram, OAM_BUF + 102 * 4 + word * 2)
            }));
            self.nmi_prepare_sprites();
            self.finish_main_loop_caller_return();
            self.begin_spiral_stair_return_main_loop_reentry();
            self.module07_00_player_control();
            self.link_oam_main();
            if crate::debug_env::var_os("ZELDA3_DEBUG_SPIRAL_RETURN").is_some() {
                let reentered_equipment: [u16; 4] = std::array::from_fn(|word| {
                    read_le_u16(&self.ram, OAM_BUF + 112 * 4 + word * 2)
                });
                eprintln!(
                    "spiral_return_equipment host={} returning={:04x?} reentered={:04x?}",
                    self.frame_ctr_dbg,
                    &self
                        .spiral_stair_return_player_oam_scanout
                        .expect("spiral-return debug requires returning player OAM")
                        [20..24],
                    reentered_equipment,
                );
            }
            self.replay_trace_ram_watch("module07-after-spiral-return-player-control");
        }
    }

    pub(in crate::zelda_rtl) fn begin_spiral_stair_return_main_loop_reentry(&mut self) {
        self.increment_frame_counter();
        self.clear_oam_buffer();
    }

    pub(in crate::zelda_rtl) fn refine_dungeon_subtile_caller_to_hud_arrow_digits(&mut self, master_cycles: u16) {
        let expected = GameWorkContinuation::FinishDungeonSubtilePaletteFilter;
        if self.game_execution_scheduler.current_work() != Some(expected) {
            return;
        }
        // Run the completed native prefix without retiring or re-scheduling
        // its existing interrupt. The temporary caller stops at LinkOam so
        // HUD can return its own saved locals instead of an overworld owner.
        let scheduler = std::mem::take(&mut self.game_execution_scheduler);
        assert!(self.active_dungeon_sprite_main_return.is_none());
        assert!(!self.dungeon_post_sprite_main_return_pending);
        if self.game_state.display.palette_filter.countdown() != 0 {
            self.apply_palette_filter_bounce();
        }
        self.dungeon_post_sprite_main_return_pending = true;
        self.complete_module07_dungeon_after_submodule();
        assert_eq!(self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn));
        let caller = self.active_dungeon_sprite_main_return.take()
            .expect("subtile prefix must retain its LinkOam caller");
        self.game_execution_scheduler.finish_work();
        self.complete_module07_link_oam_after_sprite_main(caller);
        assert!(self.native_overworld_hud_interruption.is_none());
        self.native_overworld_hud_interruption = Some(HudUpdateInterruption::ArrowDigits { master_cycles });
        crate::cycle_ledger::charge_routine(0x02_87a2, 62); // Enter the same HUD callee, stopping inside it.
        let hud = self.hud_refill_logic_until_interruption()
            .expect("source arrow-store boundary requires a native HUD update");
        assert!(self.native_overworld_hud_interruption.is_none());
        self.game_execution_scheduler = scheduler;
        self.game_execution_scheduler.refine_scheduled_work(expected,
            GameWorkContinuation::FinishDungeonSubtileHud { hud });
    }

    pub(in crate::zelda_rtl) fn refine_dungeon_spiral_caller_to_sprite_prefix(&mut self, boundary: SpriteMainCpuBoundary) {
        let expected = GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::SpiralRoomCallerResume,
        };
        if self.game_execution_scheduler.current_work() != Some(expected) {
            return;
        }
        let scheduler = std::mem::take(&mut self.game_execution_scheduler);
        assert!(self.active_dungeon_sprite_main_return.is_none());
        // The ordinary Module07 caller binds its native prefix. Keep the spiral return's pending NMI
        // and its caller/display policy, rather than scheduling another call.
        self.arm_authoritative_sprite_main_cpu_continuation(boundary, SpriteMainCpuCaller::DungeonModule07);
        self.complete_module07_dungeon_after_submodule();
        let Some(GameWorkContinuation::FinishSpriteMain { boundary, .. }) =
            self.game_execution_scheduler.current_work() else {
                panic!("spiral source prefix must suspend the native Sprite_Main caller");
            };
        self.game_execution_scheduler = scheduler;
        self.game_execution_scheduler.refine_scheduled_work(expected,
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::SpiralRoomSpriteMain { boundary },
            });
    }

    pub(in crate::zelda_rtl) fn module07_00_player_control(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x02_88de);
        crate::cycle_ledger::charge(112); // Spell, immobilization and menu-block gate.
        if (self
            .game_state
            .world
            .transient
            .flag_custom_spell_anim_active()
            | self.game_state.player.follower_link.immobilized_flag()
            | self.game_state.player.follower_link.menu_block_flag())
            == 0
        {
            crate::cycle_ledger::charge(6 + 56); // Taken BEQ, then pause-button test.
            if self.game_state.player.follower_link.filtered_joypad_h() & 0x10 != 0 {
                crate::cycle_ledger::charge(70 + 162); // Clear map state, select pause, publish module, RTS.
                self.set_overworld_map_state(0);
                self.set_submodule(1);
                self.save_main_module_for_menu();
                self.set_main_module(14);
                return;
            }
            crate::cycle_ledger::charge(6 + 56); // Skip pause setup, then map-button test.
            if self.did_press_button_for_map() {
                crate::cycle_ledger::charge(64); // Palace gate.
                if self.game_state.inventory.save_progress.palace_index_x2() != 0xff {
                    crate::cycle_ledger::charge(40); // Low room-byte gate.
                    if self.game_state.world.location.dungeon_room_index() != 0 {
                        crate::cycle_ledger::charge(48 + 162);
                        self.set_overworld_map_state(0);
                        self.set_submodule(3);
                        self.save_main_module_for_menu();
                        self.set_main_module(14);
                        return;
                    }
                }
            }
            // Every rejected map gate lands at the same Select check. The ROM
            // leaves $0200 alone on this path; only pause/map entry clears it.
            crate::cycle_ledger::charge(6 + 56);
            if self.game_state.player.follower_link.joypad1h_last() & 0x20 != 0 {
                crate::cycle_ledger::charge(56); // Progress gate.
                if self.game_state.inventory.save_progress.progress_indicator() != 0 {
                    crate::cycle_ledger::charge(280); // Choice backup, message index, saved module, JSL.
                    self.DisplaySelectMenu();
                    crate::cycle_ledger::charge(114 + 162); // Restore module, select menu, common RTS.
                    return;
                }
            }
            crate::cycle_ledger::charge(6); // Taken Select/progress rejection.
            self.replay_trace_ram_watch("module07-before-hud-switch");
            self.hud_handle_item_switch_inputs();
            self.replay_trace_ram_watch("module07-after-hud-switch");
        } else {
            crate::cycle_ledger::charge(24); // JMP Link_Main call.
        }
        crate::cycle_ledger::charge(62);
        self.replay_trace_ram_watch("module07-before-link-main");
        self.link_main();
        crate::cycle_ledger::charge(42); // RTS after Link_Main returns.
        self.replay_trace_ram_watch("module07-after-link-main");
    }

    pub(in crate::zelda_rtl) fn dungeon_handle_camera(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x02_ba31);
        // BA31..BA5E executes even when vertical velocity is zero.
        crate::cycle_ledger::charge(150);
        let z = if self.game_state.world.transient.allow_scroll_z() != 0 {
            crate::cycle_ledger::charge(72);
            if self.game_state.player.follower_link.z() != 0xffff {
                crate::cycle_ledger::charge(6);
                self.game_state.player.follower_link.z()
            } else {
                crate::cycle_ledger::charge(24);
                0
            }
        } else {
            crate::cycle_ledger::charge(6);
            0
        };
        crate::cycle_ledger::charge(276);
        let link_y_vel = self.game_state.player.follower_link.y_velocity();
        if link_y_vel != 0 {
            let y = self.game_state.player.follower_link.y().wrapping_sub(z) & 0x01ff;
            let y = y.wrapping_add(12);
            let moving_up = (link_y_vel as i8).is_negative();
            crate::cycle_ledger::charge(64 + if moving_up { 146 } else { 6 } + 14);
            let scrollamt: i16 = if moving_up { -1 } else { 1 };
            let steps = if moving_up {
                (link_y_vel as i8).wrapping_neg() as u8
            } else {
                link_y_vel
            };

            for step in 0..steps {
                crate::cycle_ledger::charge(120); // Loop velocity/direction test.
                let mut qm = self
                    .game_state
                    .world
                    .transient
                    .vertical_room_bounds_base_index();
                if moving_up {
                    crate::cycle_ledger::charge(88); // Compare the low camera limit.
                    if y > self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_y_coord_scroll_low()
                    {
                        crate::cycle_ledger::charge(22); // BCS not taken, BCC taken.
                        crate::cycle_ledger::charge(30 + if step + 1 < steps { 6 } else { 0 });
                        continue;
                    }
                    crate::cycle_ledger::charge(6); // Taken BCS to room-bound test.
                } else {
                    crate::cycle_ledger::charge(6 + 88); // Taken direction branch, high limit.
                    if y < self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_y_coord_scroll_hi()
                    {
                        crate::cycle_ledger::charge(6);
                        crate::cycle_ledger::charge(30 + if step + 1 < steps { 6 } else { 0 });
                        continue;
                    }
                    crate::cycle_ledger::charge(56); // Four INX instructions.
                    qm += 2;
                }

                crate::cycle_ledger::charge(88); // Live BG2 versus selected room bound.
                if self.game_state.display.ppu_scroll_copy.bg2_v_copy2()
                    == self.game_state.world.room_bounds.y_bound(qm)
                {
                    crate::cycle_ledger::charge(6);
                    crate::cycle_ledger::charge(30 + if step + 1 < steps { 6 } else { 0 });
                    continue;
                }

                crate::cycle_ledger::charge(150); // Update BG2, then test room sentinel.
                let bg2 = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg2_v_copy2()
                    .wrapping_add(scrollamt as u16);
                self.set_bg2_y(bg2);
                if self.game_state.world.location.dungeon_room() == 0xffff {
                    crate::cycle_ledger::charge(6);
                    crate::cycle_ledger::charge(30 + if step + 1 < steps { 6 } else { 0 });
                    continue;
                }

                crate::cycle_ledger::charge(172 + if moving_up { 24 } else { 6 } + 448);
                let subpixel = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg1_v_subpixel()
                    .wrapping_add(0x8000);
                self.set_bg1_v_subpixel(subpixel);
                let bg1_delta = (scrollamt >> 1) + i16::from(subpixel & 0x8000 == 0);
                let bg1 = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg1_v_copy2()
                    .wrapping_add(bg1_delta as u16);
                self.set_bg1_y(bg1);
                let camera_low = self
                    .game_state
                    .world
                    .camera_boundaries
                    .camera_y_coord_scroll_low()
                    .wrapping_add(scrollamt as u16);
                self.set_camera_y_coord_scroll_low(camera_low);
                self.set_camera_y_coord_scroll_hi(camera_low.wrapping_add(2));
                crate::cycle_ledger::charge(30 + if step + 1 < steps { 6 } else { 0 }); // DEY : BNE.
            }
        } else {
            crate::cycle_ledger::charge(6);
        }

        crate::cycle_ledger::charge(254); // BAD4..BAE9 horizontal setup and velocity test.
        let link_x_vel = self.game_state.player.follower_link.x_velocity();
        if link_x_vel != 0 {
            let x = (self.game_state.player.follower_link.x() & 0x01ff).wrapping_add(8);
            let moving_left = (link_x_vel as i8).is_negative();
            crate::cycle_ledger::charge(64 + if moving_left { 146 } else { 6 } + 14);
            let scrollamt: i16 = if moving_left { -1 } else { 1 };
            let steps = if moving_left {
                (link_x_vel as i8).wrapping_neg() as u8
            } else {
                link_x_vel
            };

            for step in 0..steps {
                crate::cycle_ledger::charge(120);
                let mut qm = self
                    .game_state
                    .world
                    .transient
                    .horizontal_room_bounds_base_index();
                if moving_left {
                    crate::cycle_ledger::charge(88);
                    if x > self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_x_coord_scroll_low()
                    {
                        crate::cycle_ledger::charge(22);
                        crate::cycle_ledger::charge(30 + if step + 1 < steps { 6 } else { 0 });
                        continue;
                    }
                    crate::cycle_ledger::charge(6);
                } else {
                    crate::cycle_ledger::charge(6 + 88);
                    if x < self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_x_coord_scroll_hi()
                    {
                        crate::cycle_ledger::charge(6);
                        crate::cycle_ledger::charge(30 + if step + 1 < steps { 6 } else { 0 });
                        continue;
                    }
                    crate::cycle_ledger::charge(56);
                    qm += 2;
                }

                crate::cycle_ledger::charge(88);
                if self.game_state.display.ppu_scroll_copy.bg2_h_copy2()
                    == self.game_state.world.room_bounds.x_bound(qm)
                {
                    crate::cycle_ledger::charge(6);
                    crate::cycle_ledger::charge(30 + if step + 1 < steps { 6 } else { 0 });
                    continue;
                }

                crate::cycle_ledger::charge(150);
                let bg2 = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg2_h_copy2()
                    .wrapping_add(scrollamt as u16);
                self.set_bg2_x(bg2);
                if self.game_state.world.location.dungeon_room() == 0xffff {
                    crate::cycle_ledger::charge(6);
                    crate::cycle_ledger::charge(30 + if step + 1 < steps { 6 } else { 0 });
                    continue;
                }

                crate::cycle_ledger::charge(172 + if moving_left { 24 } else { 6 } + 448);
                let subpixel = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg1_h_subpixel()
                    .wrapping_add(0x8000);
                self.set_bg1_h_subpixel(subpixel);
                let bg1_delta = (scrollamt >> 1) + i16::from(subpixel & 0x8000 == 0);
                let bg1 = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg1_h_copy2()
                    .wrapping_add(bg1_delta as u16);
                self.set_bg1_x(bg1);
                let camera_low = self
                    .game_state
                    .world
                    .camera_boundaries
                    .camera_x_coord_scroll_low()
                    .wrapping_add(scrollamt as u16);
                self.set_camera_x_coord_scroll_low(camera_low);
                self.set_camera_x_coord_scroll_hi(camera_low.wrapping_add(2));
                crate::cycle_ledger::charge(30 + if step + 1 < steps { 6 } else { 0 });
            }
        } else {
            crate::cycle_ledger::charge(6);
        }

        crate::cycle_ledger::charge(72); // BB5F room sentinel test.
        if self.game_state.world.location.dungeon_room() != 0xffff {
            let bg2_properties = self.game_state.dungeon.room_load.bg2_properties();
            crate::cycle_ledger::charge(48 + match bg2_properties {
                0 => 6,
                6..=255 => 32 + 6,
                4 => 64 + 6,
                3 => 96 + 6,
                2 => 128,
                _ => 128 + 6,
            });
            if bg2_properties == 0
                || bg2_properties == 2
                || bg2_properties == 3
                || bg2_properties == 4
                || bg2_properties >= 6
            {
                crate::cycle_ledger::charge(150); // Fall through MirrorBg1Bg2Offs.
                self.copy_bg2_live_to_bg1_live();
            }
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(64); // SEP : RTS, shared by both tails.
    }

    pub(in crate::zelda_rtl) fn sprite_dungeon_draw_all_push_blocks(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x07_f0ac);
        crate::cycle_ledger::charge(152); // Bank setup and the two-slot activity gate.
        if self.game_state.dungeon.object_tracking.changeable_object_index(0)
            | self.game_state.dungeon.object_tracking.changeable_object_index(1) == 0
        {
            crate::cycle_ledger::charge(6 + 72); // Taken BEQ, PLB, RTL.
            return;
        }
        crate::cycle_ledger::charge(16); // LDX #1.
        for i in (0..=1).rev() {
            crate::cycle_ledger::charge(48);
            if self
                .game_state
                .dungeon
                .object_tracking
                .changeable_object_index(i)
                != 0
            {
                crate::cycle_ledger::charge(110); // Select slot, save X, JSR.
                self.sprite_handle_pushed_blocks_one(i);
                crate::cycle_ledger::charge(28); // Restore X after the callee returns.
            } else {
                crate::cycle_ledger::charge(6);
            }
            crate::cycle_ledger::charge(30 + if i != 0 { 6 } else { 0 });
        }
        crate::cycle_ledger::charge(72);
    }
}
