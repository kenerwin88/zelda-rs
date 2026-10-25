//! Sprite main dispatch, slot execution, timers, and OAM allocation.

use super::*;

impl ZeldaState {

    /// `Sprite_Main`'s work before its descending slot loop: the timing
    /// workload census, proximity activation, region resets, garnish,
    /// follower, ancilla, and overlord passes.
    fn sprite_main_prefix(&mut self) {
        let mut timing_workload = SpriteMainTimingWorkload::default();
        for slot in 0..16 {
            let sprite = self.sprite_slot_view(slot);
            if sprite.state() != 0 {
                timing_workload.record_active_sprite(sprite.sprite_type(), sprite.c());
            }
        }
        let active_garnish_count = (0..30)
            .filter(|&slot| !self.garnish_slot_view(slot).is_empty())
            .count() as u8;
        timing_workload.record_garnish_table(
            self.game_state.sprites.garnish_runtime.active_type() != 0,
            active_garnish_count,
        );
        self.last_sprite_main_timing_workload = Some(timing_workload);

        // Cycle ledger: Sprite_Main $06:8328 prefix (m8 x8 throughout). The
        // routine scope is opened by the entry points (`sprite_main`,
        // `resume_sprite_main_before_first_slot_prefix`).
        if let Some(tail) = self.early_dungeon_sprite_prefix_tail.take() {
            use crate::cycle_models::dungeon_sprite_prefix::SpritePrefixTail;
            match tail {
                SpritePrefixTail::Proximity => {
                    self.sprite_proximity_activation();
                    self.sprite_main_prefix_after_proximity();
                }
                SpritePrefixTail::Garnish => {
                    self.garnish_execute_upper_slots();
                    self.replay_trace_ram_watch("sprite-after-garnish-upper");
                    self.sprite_main_prefix_after_garnish();
                }
                SpritePrefixTail::AfterAncilla(tail)=>{
                    self.finish_native_ancilla_caller(tail);
                    if !self.game_execution_scheduler.work_suspends_translated_call_stack() {self.complete_sprite_main_prefix_after_ancilla();}
                }
                SpritePrefixTail::AfterOverlord(tail)=>self.finish_native_overlord_caller(tail),
                SpritePrefixTail::AfterSlots(_)|SpritePrefixTail::Returned=>{},
                SpritePrefixTail::AfterLowerGarnish(_)|SpritePrefixTail::AfterCachedSprites(_)=>{},
                SpritePrefixTail::AfterFollower(tail)=>{
                    self.sprite_main_prefix_with_follower(Some(tail));
                }
                SpritePrefixTail::AfterGarnish(tail)=>{
                    self.finish_native_garnish_caller(tail);
                    self.replay_trace_ram_watch("sprite-after-garnish-upper");
                    self.sprite_main_prefix_after_garnish();
                }

            }
            return;
        }
        if self.game_state.world.location.is_outdoors() {
            // $06:8328-832A LDA $1b : BNE not taken (40) + $06:832C-833E five
            // STZ + JSL Sprite_ProximityActivation (222; the callee charges
            // its own body).
            crate::cycle_ledger::charge(40 + 222);
            for j in 0..5 {
                self.ancilla_slot_view_mut(j).set_floor(0);
            }
            self.sprite_proximity_activation();
        } else {
            // $06:8328-832A with BNE taken.
            crate::cycle_ledger::charge(40 + 6);
        }
        self.sprite_main_prefix_after_proximity();
    }

    fn sprite_main_prefix_after_proximity(&mut self) {
        let dark_world = u8::from(self.game_state.inventory.save_progress.dark_world_state() != 0);
        // $06:833F-8348 PHB PHK PLB LDY LDA long BEQ (144): BEQ taken when the
        // save is light world (+6), else $06:834A INY (14).
        crate::cycle_ledger::charge(if dark_world != 0 { 144 + 14 } else { 144 + 6 });
        self.set_dark_world_region_index(dark_world);
        if self.game_state.frame.submodule == 0 {
            // $06:834B-8350 STY LDA $11 BNE not taken (72) + $06:8352-835D
            // four STZ (128).
            crate::cycle_ledger::charge(72 + 128);
            self.follower_link_state_mut().set_drag_player_x(0);
            self.follower_link_state_mut().set_drag_player_y(0);
        } else {
            // $06:834B-8350 with BNE taken.
            crate::cycle_ledger::charge(72 + 6);
        }
        // $06:835E-8364 JSR Oam_ResetRegionBases and the JSL to
        // garnish (46 + 62). The follower call follows garnish return.
        crate::cycle_ledger::charge(108);
        self.oam_reset_region_bases();
        self.replay_trace_ram_watch("sprite-after-oam-reset");
        self.garnish_execute_upper_slots();
        self.replay_trace_ram_watch("sprite-after-garnish-upper");
        self.sprite_main_prefix_after_garnish();
    }

    fn sprite_main_prefix_after_garnish(&mut self) {
        crate::cycle_ledger::charge(62); // JSL Tagalong_Main_ after garnish returns.
        self.sprite_main_prefix_with_follower(None);
    }

    fn sprite_main_prefix_with_follower(
        &mut self,
        native_tail: Option<crate::cycle_models::dungeon_follower::FollowerTail>,
    ) {
        if native_tail == Some(crate::cycle_models::dungeon_follower::FollowerTail::Returned) {
            self.complete_sprite_main_prefix_after_follower();
            return;
        }
        {
            // Tagalong_Main_ $09:9F91, the JSL target wrapping Follower_Main:
            // PHB PHK PLB JSR $09:9FC4 (118); PLB RTL remain below the call.
            let _tagalong = crate::cycle_ledger::routine(0x09_9f91);
            if native_tail.is_none() {
                crate::cycle_ledger::charge(118);
            }
            let parked_work = self.game_execution_scheduler.current_work();
            if let Some(tail) = native_tail {
                self.finish_native_follower(tail);
            } else {
                self.follower_main();
            }
            if self.game_execution_scheduler.current_work() != parked_work
                && self
                    .game_execution_scheduler
                    .work_suspends_translated_call_stack()
            {
                return;
            }
            if let Some(boundary @ SpriteMainCpuBoundary::FollowerWrapperReturn { plb_completed }) =
                self.sprite_main_cpu_boundary
            {
                if plb_completed {
                    crate::cycle_ledger::charge(28);
                }
                self.sprite_main_cpu_boundary = None;
                let slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, slices, caller);
                return;
            }
            crate::cycle_ledger::charge(72);
        }
        self.complete_sprite_main_prefix_after_follower();
    }
    pub(in crate::zelda_rtl) fn complete_sprite_main_prefix_after_follower(&mut self) {
        // $06:8369-837B pickup-flag moves, hitbox high byte and timer test.
        crate::cycle_ledger::charge(200);
        self.replay_trace_ram_watch("sprite-after-follower");
        let pickup_slot_cache = self.game_state.player.follower_link.sprite_pickup_flag();
        self.sprite_workspace_mut()
            .set_pickup_slot_cache(pickup_slot_cache);
        self.follower_link_state_mut().clear_sprite_pickup_flag();
        self.hitbox_scratch_offset_mut().set_x_high_offset(0x80);
        // $06:837D-8380 DEC $47 : BRA (60; BRA's 22 already includes the
        // taken cycles) when the masked timer is nonzero, else BEQ taken (+6)
        // into $06:8381 STZ $47 (24).
        let damaging_enemies_timer = self.game_state.sprite_battle.damaging_enemies_timer();
        crate::cycle_ledger::charge(if damaging_enemies_timer & 0x7f != 0 {
            60
        } else {
            6 + 24
        });
        self.sprite_battle_mut().tick_damaging_enemies_timer();
        self.follower_link_state_mut()
            .clear_player_pose_draw_counter();
        {
            self.follower_link_state_mut().set_pull_action_state(0);
            self.follower_link_state_mut().clear_prevent_movement();
        }
        // $06:8383-838F three STZ, LDA $0fdc, BEQ not taken (144).
        crate::cycle_ledger::charge(144);
        if self.game_state.sprites.system.alert_flag() != 0 {
            // $06:8391 DEC abs (46).
            crate::cycle_ledger::charge(46);
            self.sprite_system_mut().decrement_alert_flag();
        } else {
            // BEQ taken.
            crate::cycle_ledger::charge(6);
        }
        // $06:8394 JSL Ancilla_Main (62); the rest of the $06:8394-83A0 block
        // is charged when the prefix completes after the ancilla call.
        crate::cycle_ledger::charge(62);
        self.ancilla_main();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            // An ancilla's synchronous item receipt suspended the stack; the
            // receipt's completion resumes the rest of this prefix.
            return;
        }
        self.complete_sprite_main_prefix_after_ancilla();
    }

    pub(in crate::zelda_rtl) fn complete_sprite_main_prefix_after_ancilla(&mut self) {
        self.replay_trace_ram_watch("sprite-after-ancilla");
        // $06:8398-83A0 JSL Overlord_Main (62), STZ $0b9a (32), LDX #$0f (16).
        crate::cycle_ledger::charge(62 + 32 + 16);
        self.overlord_main();
        self.replay_trace_ram_watch("sprite-after-overlord");
        self.archery_game_mut().clear_out_of_arrows();
    }

    /// A saved `Sprite_Main` caller whose loop had not started (the
    /// `BeforeFirstSlot` boundary) resumed in this host and ran its prefix
    /// plus slots 15 down to `newly_completed_slot` before the wire suspended
    /// it again (route host 187518). Execute exactly that semantic delta.
    pub(in crate::zelda_rtl) fn advance_sprite_main_before_first_slot_to_after_slot(
        &mut self,
        newly_completed_slot: u8,
    ) {
        self.resume_sprite_main_before_first_slot_prefix();
        self.advance_sprite_main_after_slot_boundary(16, newly_completed_slot);
    }

    /// Resume a parked `Sprite_Main` entry through the timer/OAM return of
    /// `slot`. Higher slots have returned in full; the dispatch state is read
    /// before the timer helper exactly as `Sprite_ExecuteSingle` does, then
    /// retained in the bound continuation for the next source resume.
    pub(in crate::zelda_rtl) fn advance_sprite_main_before_first_slot_to_after_timers_and_oam(
        &mut self,
        slot: u8,
    ) -> SpriteMainCpuBoundary {
        self.resume_sprite_main_before_first_slot_prefix();
        self.advance_sprite_main_after_slot_to_after_timers(16, slot)
    }

    /// Publish a completed timer/OAM prefix while retaining the same suspended
    /// sprite caller and NMI ownership. The slot's dispatch runs on resume.
    pub(in crate::zelda_rtl) fn refine_sprite_main_to_timer_return(&mut self, slot: u8) {
        let Some(expected @ GameWorkContinuation::FinishSpriteMain { boundary, caller }) =
            self.game_execution_scheduler.current_work() else { return };
        let boundary = match boundary {
            SpriteMainCpuBoundary::BeforeFirstSlot =>
                self.advance_sprite_main_before_first_slot_to_after_timers_and_oam(slot),
            SpriteMainCpuBoundary::AfterSlot(completed) if slot < completed =>
                self.advance_sprite_main_after_slot_to_after_timers(completed, slot),
            _ => return,
        };
        self.game_execution_scheduler.refine_scheduled_work(expected,
            GameWorkContinuation::FinishSpriteMain { boundary, caller });
    }

    pub(in crate::zelda_rtl) fn refine_sprite_main_to_coordinate_moves(&mut self, slot: u8, next_instruction: u8) {
        assert!(next_instruction <= 8);
        let Some(expected @ GameWorkContinuation::FinishSpriteMain { boundary, caller }) =
            self.game_execution_scheduler.current_work() else { return };
        let completed_slot = match boundary {
            SpriteMainCpuBoundary::BeforeFirstSlot => {
                self.resume_sprite_main_before_first_slot_prefix();
                16
            }
            SpriteMainCpuBoundary::AfterSlot(completed) if slot < completed => completed,
            _ => return,
        };
        let state = self.begin_sprite_main_slot_after_boundary(completed_slot, slot);
        // ExecuteSingle's LDA/BEQ, PHA/JSR; TimersAndOam's coordinate JSR.
        crate::cycle_ledger::charge_routine(0x06_84e2, 48 + 22 + 46);
        crate::cycle_ledger::charge_routine(0x06_83f2, 46);
        let x = self.sprite_get_x(usize::from(slot)).to_le_bytes();
        let y = self.sprite_get_y(usize::from(slot)).to_le_bytes();
        let coordinates = [x[0], x[1], y[0], y[1]];
        self.advance_sprite_coordinate_moves(coordinates, 0, next_instruction);
        self.game_execution_scheduler.refine_scheduled_work(expected,
            GameWorkContinuation::FinishSpriteMain {
                boundary: SpriteMainCpuBoundary::TimerCoordinateMoves {
                    slot, state, next_instruction, coordinates,
                }, caller,
            });
    }

    pub(in crate::zelda_rtl) fn refine_sprite_main_to_guard_offscreen_kill(&mut self, slot: u8) {
        self.refine_sprite_main_to_guard_checkpoint(slot,
            crate::GuardAnimationCheckpoint::OffscreenKillSelfPending);
    }

    pub(in crate::zelda_rtl) fn refine_sprite_main_to_rat_oam_flags_store(&mut self, slot: u8) {
        self.refine_sprite_main_to_timer_return(slot);
        let Some(expected @ GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterTimersAndOam { slot: pending, state: Some(9) },
            caller,
        }) = self.game_execution_scheduler.current_work() else { return };
        if pending != slot { return; }
        let k = usize::from(slot);
        assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x6d);
        crate::cycle_ledger::charge(6 + 24);
        self.charge_sprite_active_main_dispatch(k);
        let flags = self.rat_animation_until_oam_flags_store(k);
        self.game_execution_scheduler.refine_scheduled_work(expected,
            GameWorkContinuation::FinishSpriteMain {
                boundary: SpriteMainCpuBoundary::RatOamFlagsPending { slot, flags }, caller,
            });
    }

    pub(in crate::zelda_rtl) fn refine_sprite_main_to_keese_inactive_check(&mut self, slot: u8) {
        self.refine_sprite_main_to_timer_return(slot);
        let Some(expected @ GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterTimersAndOam { slot: pending, state: Some(9) },
            caller,
        }) = self.game_execution_scheduler.current_work() else { return };
        if pending != slot { return; }
        let k = usize::from(slot);
        assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x6f);
        crate::cycle_ledger::charge(6 + 24);
        self.charge_sprite_active_main_dispatch(k);
        self.keese_draw_before_inactive_check(k);
        self.game_execution_scheduler.refine_scheduled_work(expected,
            GameWorkContinuation::FinishSpriteMain {
                boundary: SpriteMainCpuBoundary::KeeseInactiveCheckPending { slot }, caller,
            });
    }

    pub(in crate::zelda_rtl) fn refine_sprite_main_to_guard_checkpoint(
        &mut self,
        slot: u8,
        checkpoint: crate::GuardAnimationCheckpoint,
    ) {
        self.refine_sprite_main_to_timer_return(slot);
        let Some(expected @ GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterTimersAndOam { slot: pending, state: Some(9) },
            caller,
        }) = self.game_execution_scheduler.current_work() else { return };
        if pending != slot { return; }
        let k = usize::from(slot);
        assert!((0x41..=0x44).contains(&self.sprite_slot_view(k).sprite_type()));
        crate::cycle_ledger::charge(6 + 24);
        self.charge_sprite_active_main_dispatch(k);
        if self.sprite_slot_view(k).sprite_type() == 0x41 {
            assert_eq!(self.sprite_slot_view(k).c(), 0);
            crate::cycle_ledger::charge(48 + 24); // BlueGuard's Guard_Main dispatch.
        }
        let continuation = self.guard_animation_until_checkpoint(k, checkpoint);
        self.game_execution_scheduler.refine_scheduled_work(expected,
            GameWorkContinuation::FinishSpriteMain {
                boundary: SpriteMainCpuBoundary::GuardAnimation {
                    slot, checkpoint, continuation: Some(continuation),
                }, caller,
            });
    }

    /// Advance only the descending slot delta after an already returned call.
    /// The destination dispatch state precedes its timer/OAM helper.
    pub(in crate::zelda_rtl) fn advance_sprite_main_after_slot_to_after_timers(
        &mut self,
        completed_slot: u8,
        slot: u8,
    ) -> SpriteMainCpuBoundary {
        let state = self.begin_sprite_main_slot_after_boundary(completed_slot, slot);
        {
            let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
            crate::cycle_ledger::charge(48 + 128);
            self.sprite_timers_and_oam(usize::from(slot));
        }
        SpriteMainCpuBoundary::AfterTimersAndOam { slot, state: Some(state) }
    }

    fn begin_sprite_main_slot_after_boundary(&mut self, completed_slot: u8, slot: u8) -> u8 {
        assert!(slot < completed_slot && completed_slot <= 16);
        // Cycle ledger: a partial Sprite_Main $06:8328 body (its walk only).
        let _scope = crate::cycle_ledger::routine(0x06_8328);
        for completed_slot in ((slot + 1)..completed_slot).rev() {
            self.sprite_system_mut()
                .set_cur_object_index(completed_slot);
            // $06:83A1-83A4 STX $0fa0 : JSR Sprite_ExecuteSingle (78).
            crate::cycle_ledger::charge(78);
            self.sprite_execute_single(usize::from(completed_slot));
            // $06:83A7-83A8 DEX : BPL (30), taken while X stays non-negative.
            crate::cycle_ledger::charge(if completed_slot > 0 { 30 + 6 } else { 30 });
        }
        self.sprite_system_mut().set_cur_object_index(slot);
        let state = self.sprite_slot_view(usize::from(slot)).state();
        assert_ne!(
            state, 0,
            "source timer/OAM return requires an active sprite slot",
        );
        // $06:83A1-83A4 for the target slot. Its callee remains pending.
        crate::cycle_ledger::charge(78);
        state
    }

    fn resume_sprite_main_before_first_slot_prefix(&mut self) {
        let parked_work = self.game_execution_scheduler.current_work();
        if matches!(
            parked_work,
            Some(GameWorkContinuation::FinishSpriteMain {
                boundary: SpriteMainCpuBoundary::Module09FinalScrollPairPending,
                ..
            })
        ) {
            self.complete_module09_final_scroll_pair();
        }
        if let Some((main_module, submodule)) = self.pending_module09_frame_advance.take() {
            // The suspended Module09 handler's deferred advance lands right
            // before Sprite_Main, as the ROM writes it.
            if self.pending_overworld_sprite_reload_slots.is_some() {
                self.publish_deferred_module09_sprite_slots_at_reload_return();
            }
            self.set_submodule(submodule);
            self.set_main_module(main_module);
        }
        // Cycle ledger: a partial Sprite_Main $06:8328 body (its prefix only).
        let _scope = crate::cycle_ledger::routine(0x06_8328);
        self.sprite_main_prefix();
        assert_eq!(
            self.game_execution_scheduler.current_work(),
            parked_work,
            "source Sprite_Main slot progress crossed a newly suspended native prefix",
        );
    }

    pub(in crate::zelda_rtl) fn sprite_main(&mut self) {
        if self.sprite_main_cpu_boundary == Some(SpriteMainCpuBoundary::BeforeFirstSlot) {
            let boundary = self
                .sprite_main_cpu_boundary
                .take()
                .expect("Sprite_Main boundary was checked above");
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Sprite_Main continuation requires a measured NMI phase",
            );
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
            return;
        }

        // Cycle ledger: Sprite_Main $06:8328 (m8 x8). A body that a CPU
        // boundary suspends records its charge up to the suspension; the
        // continuation entries open their own scope for the remainder.
        let _scope = crate::cycle_ledger::routine(0x06_8328);
        let native_returned = self.early_dungeon_sprite_prefix_tail
            == Some(crate::cycle_models::dungeon_sprite_prefix::SpritePrefixTail::Returned);
        let native_slots = match self.early_dungeon_sprite_prefix_tail {
            Some(crate::cycle_models::dungeon_sprite_prefix::SpritePrefixTail::AfterSlots(tail))=>Some(tail),
            Some(crate::cycle_models::dungeon_sprite_prefix::SpritePrefixTail::AfterLowerGarnish(_)|crate::cycle_models::dungeon_sprite_prefix::SpritePrefixTail::AfterCachedSprites(_))=>Some(crate::cycle_models::dungeon_sprite_slots::SpriteSlotsTail::LowerGarnish),
            _=>None,
        };
        let native_lower_garnish=match self.early_dungeon_sprite_prefix_tail {
            Some(crate::cycle_models::dungeon_sprite_prefix::SpritePrefixTail::AfterLowerGarnish(tail))=>Some(tail),
            _=>None,
        };
        let native_cached=match self.early_dungeon_sprite_prefix_tail {
            Some(crate::cycle_models::dungeon_sprite_prefix::SpritePrefixTail::AfterCachedSprites(tail))=>Some(tail),
            _=>None,
        };
        self.sprite_main_prefix();
        if native_returned { return; }
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            // The prefix's ancilla item receipt suspended the stack; its
            // completion runs the slot loop and suffix.
            return;
        }
        let trace_sprite_slots = ZeldaState::parse_trace_env_u32("ZELDA3_REPLAY_RAM_WATCH_FRAME")
            .is_some_and(|frame| self.trace_frame_matches(frame));

        let slot_count=match native_slots {
            Some(crate::cycle_models::dungeon_sprite_slots::SpriteSlotsTail::AfterTimers {slot,..})=>usize::from(slot)+1,
            Some(crate::cycle_models::dungeon_sprite_slots::SpriteSlotsTail::LowerGarnish)=>0,
            None=>16,
        };
        for k in (0..slot_count).rev() {
            self.sprite_system_mut().set_cur_object_index(k as u8);
            // $06:83A1-83A4 STX $0fa0 : JSR Sprite_ExecuteSingle (78). The
            // DEX : BPL tail is charged once the slot completes, either below
            // or by `complete_sprite_main_after_interrupted_slot` when a CPU
            // boundary lane suspends this slot. The lanes that enter the slot
            // through `sprite_timers_and_oam` directly do not charge the
            // Sprite_ExecuteSingle entry and dispatch blocks.
            let captured_state=match native_slots {
                Some(crate::cycle_models::dungeon_sprite_slots::SpriteSlotsTail::AfterTimers {slot,state,tail}) if usize::from(slot)==k=>Some((state,tail)),
                _=>None,
            };
            if let Some((state,tail))=captured_state {
                assert!(self.native_sprite_timer_handoff.is_none());
                self.native_sprite_timer_handoff=Some((k as u8,state,tail));
            } else {crate::cycle_ledger::charge(78);}
            if trace_sprite_slots {
                self.replay_trace_ram_watch(&format!("sprite-before-execute-single slot={k}"));
            }
            if self.sprite_main_resume_initialize_reset_properties(k) {
                return;
            }
            if self.sprite_main_resume_initialize_load_properties(k) {
                return;
            }
            if self.sprite_main_resume_mini_moldorm_history(k) {
                return;
            }
            if self.sprite_main_resume_fire_debirando_before_spawn(k) {
                return;
            }
            if self.sprite_main_resume_trinexx_final_phase_draw(k) {
                return;
            }
            if let Some(SpriteMainCpuBoundary::LanmolaDrawPrefix {
                slot,
                completed_stores,
            }) = self.sprite_main_cpu_boundary
            {
                if slot == k as u8 {
                    let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                    assert_ne!(nmi_slices, 0);
                    self.sprite_main_cpu_boundary = None;
                    assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x54);
                    // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                    // directly; open its scope for the timers and the handler part.
                    let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                    self.sprite_timers_and_oam(k);
                    self.begin_lanmola_draw_prefix_checkpoint(k, completed_stores);
                    let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                    self.schedule_sprite_main_cpu_continuation(
                        SpriteMainCpuBoundary::LanmolaDrawPrefix {
                            slot,
                            completed_stores,
                        },
                        nmi_slices,
                        caller,
                    );
                    return;
                }
            }
            if let Some(SpriteMainCpuBoundary::HelmasaurHardHatTileCollision { slot, stage }) =
                self.sprite_main_cpu_boundary
            {
                if slot == k as u8 {
                    let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                    assert_ne!(nmi_slices, 0);
                    self.sprite_main_cpu_boundary = None;
                    assert_eq!(self.sprite_slot_view(k).state(), 9);
                    assert!(matches!(
                        self.sprite_slot_view(k).sprite_type(),
                        0x13 | 0x26
                    ));
                    // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                    // directly; open its scope for the timers and the handler part.
                    let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                    self.sprite_timers_and_oam(k);
                    self.begin_helmasaur_hard_hat_tile_collision_checkpoint(k, stage);
                    let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                    self.schedule_sprite_main_cpu_continuation(
                        SpriteMainCpuBoundary::HelmasaurHardHatTileCollision { slot, stage },
                        nmi_slices,
                        caller,
                    );
                    return;
                }
            }
            if self.sprite_main_resume_trinexx_final_phase_tile_collision(k) {
                return;
            }
            if self.sprite_main_resume_trinexx_death_explosion_spawn(k) {
                return;
            }
            if self.sprite_main_resume_fire_debirando_spawn(k) {
                return;
            }
            if self.sprite_main_resume_guard_prep_weapon_flags_pending(k) {
                return;
            }
            if self.sprite_main_resume_guard_prep_parry_hitbox(k) {
                return;
            }
            if self.sprite_main_resume_guard_prep_patrol_delay(k) {
                return;
            }
            if self.sprite_main_resume_guard_prep_tile_collision_returned(k) {
                return;
            }
            if self.sprite_main_resume_guard_animation(k) {
                return;
            }
            if self.sprite_main_resume_after_timers_and_oam(k) {
                return;
            }
            if self.sprite_main_resume_happiness_pond_rupee_graphics_started(k) {
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(
                    k as u8,
                ))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.catfish_before_medallion_graphics(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(k as u8),
                    nmi_slices,
                    caller,
                );
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::TrinexxHeadDrawSetup(k as u8))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sidenexx_head_draw_setup(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::TrinexxHeadDrawSetup(k as u8),
                    nmi_slices,
                    caller,
                );
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(
                    k as u8,
                ))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.waterfall_before_gt_cutscene_graphics(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(k as u8),
                    nmi_slices,
                    caller,
                );
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::TrinexxBreathTileCollisionReturned(
                    k as u8,
                ))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                assert!(
                    self.trinexx_breath_until_tile_collision(k),
                    "Trinexx breath tile-collision checkpoint requires an active breath"
                );
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::TrinexxBreathTileCollisionReturned(k as u8),
                    nmi_slices,
                    caller,
                );
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::LaserEyeDrawPrologue(k as u8))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                assert!(
                    self.sprite_95_laser_eye_until_draw_prologue(k),
                    "LaserEye draw prologue checkpoint requires the eye variant"
                );
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::LaserEyeDrawPrologue(k as u8),
                    nmi_slices,
                    caller,
                );
                return;
            }
            if self.sprite_main_resume_after_antfairy_subtype2_increment(k) {
                return;
            }
            if self.sprite_main_resume_after_lanmola_subtype2_increment(k) {
                return;
            }
            if self.sprite_main_resume_after_helmasaur_hard_hat_beetle_subtype2_increment(k) {
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::InitializePrepPending { slot: k as u8 })
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                let boundary = self.sprite_main_cpu_boundary.take().unwrap();
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if let Some(SpriteMainCpuBoundary::InitializePrepMoveY {
                slot,
                checkpoint,
                continuation: None,
            }) = self.sprite_main_cpu_boundary
            {
                if usize::from(slot) == k {
                    let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                    assert_ne!(nmi_slices, 0);
                    assert_eq!(self.sprite_slot_view(k).state(), 8);
                    // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                    // directly; open its scope for the timers and the handler part.
                    let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                    self.sprite_timers_and_oam(k);
                    self.sprite_module_initialize_properties(k);
                    let continuation = self.sprite_prep_move_y_through_checkpoint(k, checkpoint);
                    let boundary = SpriteMainCpuBoundary::InitializePrepMoveY {
                        slot,
                        checkpoint,
                        continuation: Some(continuation),
                    };
                    self.sprite_main_cpu_boundary = None;
                    let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                    self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                    return;
                }
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::HogSpearBodyGraphicsPending { slot: k as u8 })
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                let boundary = self.sprite_main_cpu_boundary.take().unwrap();
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x45);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                assert!(
                    self.hog_spear_man_through_body_increments(k),
                    "source Hog Spear body checkpoint did not reach its animation helper"
                );
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if self.sprite_main_resume_mini_moldorm_ai_pending(k) {
                return;
            }
            if let Some(
                boundary @ (SpriteMainCpuBoundary::SwamolaSegmentDraw { slot, .. }
                | SpriteMainCpuBoundary::SwamolaHeadDraw { slot }
                | SpriteMainCpuBoundary::SwamolaHeadDrawCompleted { slot }),
            ) = self.sprite_main_cpu_boundary
            {
                if slot == k as u8 {
                    assert_eq!(self.sprite_slot_view(k).state(), 9);
                    assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xcf);
                    let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                    assert_ne!(nmi_slices, 0);
                    // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                    // directly; open its scope for the timers and the handler part.
                    let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                    self.sprite_timers_and_oam(k);
                    self.sprite_active_main(k);
                    self.sprite_main_cpu_boundary = None;
                    let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                    self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                    return;
                }
            }
            if self.sprite_main_resume_trinexx_head_draw(k) {
                return;
            }
            if self.sprite_main_resume_sidenexx_neck_target_loop(k) {
                return;
            }
            if self.sprite_main_resume_trinexx_head_front_part(k) {
                return;
            }
            if matches!(self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AbsorbableHorizontalTileLookup { slot } | SpriteMainCpuBoundary::AbsorbableVerticalTileLookup { slot } | SpriteMainCpuBoundary::AbsorbableVerticalTileAttributeLoaded { slot }) if slot == k as u8)
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                let boundary = self.sprite_main_cpu_boundary.unwrap();
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert!((0xd8..=0xe6).contains(&self.sprite_slot_view(k).sprite_type()));
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_active_main(k);
                self.sprite_main_cpu_boundary = None;
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::PengatorSlidePending { slot: k as u8 })
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                let boundary = self.sprite_main_cpu_boundary.take().unwrap();
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x99);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                assert!(
                    self.pengator_before_ai(k),
                    "source Pengator slide requires its active movement prefix"
                );
                assert_eq!(self.sprite_slot_view(k).ai_state(), 3);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::AntifairyBouncePending { slot: k as u8 })
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                let boundary = self.sprite_main_cpu_boundary.take().unwrap();
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x15);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                assert!(
                    self.antifairy_before_bounce(k),
                    "source Antifairy bounce requires its active movement prefix"
                );
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::KholdstareDamagePending { slot: k as u8 })
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                let boundary = self.sprite_main_cpu_boundary.take().unwrap();
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xa2);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                assert!(
                    self.kholdstare_before_ai(k),
                    "source Kholdstare damage requires its active movement prefix"
                );
                assert!(self.sprite_slot_view(k).ai_state() < 2);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::BonkItemGraphicsEntered(k as u8))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "bonk-item graphics continuation requires a measured NMI phase",
                );
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("bonk-item graphics boundary was checked above");
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x3b);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                self.sprite_slot_view_mut(k).set_floor(2);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if self.sprite_main_resume_bari_before_random(k) {
                return;
            }
            if self.sprite_main_resume_after_main_and_aux1_timer_decrements(k) {
                return;
            }
            if self.sprite_main_resume_after_main_timer_decrement(k) {
                return;
            }
            if self.sprite_main_resume_after_zero_hit_timer_clear(k) {
                return;
            }
            if self.sprite_main_resume_after_primary_timer_decrements(k) {
                return;
            }
            if self.sprite_main_resume_after_hit_timer(k) {
                return;
            }
            if self.sprite_main_resume_after_timer_decrements(k) {
                return;
            }
            if self.sprite_main_resume_after_wallmaster_reset_prefix(k) {
                return;
            }
            if self.sprite_main_resume_after_throwable_scenery_state_clear(k) {
                return;
            }
            if self.sprite_main_resume_before_zelda_follower_graphics(k) {
                return;
            }
            if self.sprite_main_resume_zazak_after_graphics(k) {
                return;
            }
            if self.sprite_main_resume_follower_graphics(k) {
                return;
            }
            if self.sprite_main_resume_king_zora_flippers_graphics_started(k) {
                return;
            }
            if self.sprite_main_resume_after_single_small_draw_position(k) {
                return;
            }
            if self.sprite_main_resume_probe_after_oam_coordinates(k) {
                return;
            }
            let enters_big_key_graphics = self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::BigKeyDropGraphicsStarted(k as u8));
            if enters_big_key_graphics {
                self.sprite_main_cpu_boundary = None;
                assert_ne!(
                    std::mem::take(&mut self.sprite_main_cpu_nmi_slices),
                    0,
                    "big-key graphics continuation requires a source timing boundary",
                );
                assert!(matches!(
                    std::mem::take(&mut self.sprite_main_cpu_caller),
                    SpriteMainCpuCaller::DungeonModule07Live { .. }
                ));
            }
            let enters_item_receipt_graphics = self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::ItemReceiptGraphicsStarted(k as u8));
            if enters_item_receipt_graphics {
                self.sprite_main_cpu_boundary = None;
                assert_ne!(
                    std::mem::take(&mut self.sprite_main_cpu_nmi_slices),
                    0,
                    "item-receipt graphics continuation requires a source timing boundary",
                );
                assert!(matches!(
                    std::mem::take(&mut self.sprite_main_cpu_caller),
                    SpriteMainCpuCaller::DungeonModule07Live { .. }
                        | SpriteMainCpuCaller::Module09 { .. }
                ));
            }
            if let Some((state,tail))=captured_state {
                self.sprite_execute_single_after_native_timers(k,state,tail);
            } else {
                self.sprite_execute_single(k);
            }
            if enters_big_key_graphics {
                assert!(
                    matches!(
                        self.game_execution_scheduler.current_work(),
                        Some(GameWorkContinuation::FinishBigKeyDropGraphics {
                            sprite_slot,
                            ..
                        }) if sprite_slot == k as u8
                    ),
                    "source big-key boundary did not enter the native graphics continuation"
                );
                return;
            }
            if self.sprite_main_lane_at_4374(enters_item_receipt_graphics, k) {
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(
                    SpriteMainCpuBoundary::AfterSlot(slot)
                        | SpriteMainCpuBoundary::AfterActiveCuccoX { slot, .. }
                        | SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel { slot, .. }
                        | SpriteMainCpuBoundary::MasterSwordLightBeamMovement { slot, .. }
                        | SpriteMainCpuBoundary::BoulderMovement { slot, .. }
                        | SpriteMainCpuBoundary::ZoraFireballMovement { slot, .. }
                        | SpriteMainCpuBoundary::AgahnimMotionBlurSpawn { slot, .. }
                        | SpriteMainCpuBoundary::BuzzblobAfterXSubpixel { slot, .. }
                        | SpriteMainCpuBoundary::MasterSwordLightBeamSpawn { slot, .. }
                        | SpriteMainCpuBoundary::AfterCuccoFleeMovement { slot, .. }
                        | SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements { slot, .. }
                        | SpriteMainCpuBoundary::AfterCuccoGraphicsPublication { slot, .. }
                ) if slot == k as u8
            ) {
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("Sprite_Main boundary was checked above");
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Sprite_Main continuation requires a measured NMI phase",
                );
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
            }
            if self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
            {
                return;
            }
            // $06:83A7-83A8 DEX : BPL (30), taken (+6) for every slot but 0.
            crate::cycle_ledger::charge(if k > 0 { 30 + 6 } else { 30 });
            if trace_sprite_slots {
                self.replay_trace_ram_watch(&format!("sprite-after-execute-single slot={k}"));
            }
        }
        self.complete_sprite_main_after_all_slots_with_native_suffix(native_lower_garnish,native_cached);
        if self.game_execution_scheduler.work_suspends_translated_call_stack() {
            return;
        }
        let suffix_nmi_slices =
            std::mem::take(&mut self.dungeon_room_load_module_suffix_nmi_slices);
        if suffix_nmi_slices != 0 {
            self.game_execution_scheduler
                .schedule_cpu_timed_work_before_trailing_nmi(
                    GameWorkContinuation::FinishDungeonSupertileTransition {
                        work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
                    },
                    suffix_nmi_slices,
                );
        }
    }

    pub(in crate::zelda_rtl) fn arm_sprite_main_cpu_continuation(
        &mut self,
        boundary: SpriteMainCpuBoundary,
        nmi_slices: u8,
        caller: SpriteMainCpuCaller,
    ) {
        assert_ne!(nmi_slices, 0, "Sprite_Main continuation requires an NMI");
        assert!(
            self.sprite_main_cpu_boundary.is_none(),
            "Sprite_Main CPU continuation was already armed",
        );
        assert_eq!(self.sprite_main_cpu_nmi_slices, 0);
        assert_eq!(self.sprite_main_cpu_caller, SpriteMainCpuCaller::default());
        self.sprite_main_cpu_boundary = Some(boundary);
        self.sprite_main_cpu_nmi_slices = nmi_slices;
        self.sprite_main_cpu_caller = caller;
    }

    /// Replace an unconsumed reconstructed Sprite_Main checkpoint with the
    /// boundary published by the continuous semantic authority.
    ///
    /// Legacy CPU plans populate these three fields before the translated
    /// caller reaches Sprite_Main. No C work has executed at that checkpoint,
    /// so replacing it is an ownership transfer, not a replay or rollback.
    /// Once scheduled work exists the call is already in flight and must be
    /// refined through the scheduler instead.
    pub(in crate::zelda_rtl) fn arm_authoritative_sprite_main_cpu_continuation(
        &mut self,
        boundary: SpriteMainCpuBoundary,
        caller: SpriteMainCpuCaller,
    ) {
        assert!(
            !self
                .game_execution_scheduler
                .work_suspends_translated_call_stack(),
            "source Sprite_Main entry receipt cannot replace in-flight work",
        );
        self.sprite_main_cpu_boundary = None;
        self.sprite_main_cpu_nmi_slices = 0;
        self.sprite_main_cpu_caller = SpriteMainCpuCaller::default();
        self.arm_sprite_main_cpu_continuation(boundary, 1, caller);
    }

    pub(in crate::zelda_rtl) fn schedule_sprite_main_cpu_continuation(
        &mut self,
        boundary: SpriteMainCpuBoundary,
        nmi_slices: u8,
        caller: SpriteMainCpuCaller,
    ) {
        self.stage_interrupted_sprite_main_oam_scanout();
        // A retained CPU trial can select a precise suspension boundary
        // without installing the live semantic-receipt owner. Its resumed
        // caller must use native completion; otherwise the scheduler waits
        // forever for a Sprite_Main-return receipt that cannot arrive.
        let resume_caller = match caller {
            SpriteMainCpuCaller::DungeonModule07Live { .. }
                if self.native_exact_cpu_host_trace.is_some()
                    && !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) =>
                    SpriteMainCpuCaller::DungeonModule07,
            caller => caller,
        };
        let continuation = GameWorkContinuation::FinishSpriteMain { boundary, caller: resume_caller };
        let resumes_after_current_nmi = match caller {
            SpriteMainCpuCaller::DungeonModule07 => {
                self.dungeon_quadrant_cpu_continuation_active && nmi_slices == 1
            }
            SpriteMainCpuCaller::DungeonModule07Live { boundary } => {
                match boundary {
                    OriginalTimingBoundary::HostReturn => self
                        .game_execution_scheduler
                        .schedule_work(continuation, nmi_slices),
                    OriginalTimingBoundary::NmiAccepted => self
                        .game_execution_scheduler
                        .schedule_after_current_trailing_nmi(continuation),
                }
                return;
            }
            SpriteMainCpuCaller::WorldMapOverlayReload { .. } => {
                self.game_execution_scheduler
                    .schedule_cpu_timed_work_resuming_after_current_trailing_nmi(
                        continuation,
                        nmi_slices,
                    );
                return;
            }
            SpriteMainCpuCaller::Module09 { boundary }
            | SpriteMainCpuCaller::BossVictory { boundary }
            | SpriteMainCpuCaller::SaveAndQuit { boundary } => {
                match boundary {
                    OriginalTimingBoundary::HostReturn => self
                        .game_execution_scheduler
                        .schedule_work(continuation, nmi_slices),
                    OriginalTimingBoundary::NmiAccepted => self
                        .game_execution_scheduler
                        .schedule_after_current_trailing_nmi(continuation),
                }
                return;
            }
        };
        if resumes_after_current_nmi {
            self.game_execution_scheduler
                .schedule_after_current_trailing_nmi(continuation);
        } else {
            self.game_execution_scheduler
                .schedule_work(continuation, nmi_slices);
        }
    }

    pub(in crate::zelda_rtl) fn complete_sprite_main_after_interrupted_slot(&mut self, interrupted_slot: usize) {
        // Cycle ledger: the remainder of a suspended Sprite_Main $06:8328
        // body. The interrupted slot's STX : JSR was charged when it was
        // entered; its $06:83A7-83A8 DEX : BPL (30, +6 taken) lands here.
        // `interrupted_slot == 16` means no slot had started.
        let _scope = crate::cycle_ledger::routine(0x06_8328);
        if interrupted_slot < 16 {
            crate::cycle_ledger::charge(if interrupted_slot > 0 { 30 + 6 } else { 30 });
        }
        for k in (0..interrupted_slot).rev() {
            self.sprite_system_mut().set_cur_object_index(k as u8);
            // $06:83A1-83A4 STX $0fa0 : JSR Sprite_ExecuteSingle (78).
            crate::cycle_ledger::charge(78);
            self.sprite_execute_single(k);
            debug_assert!(!self.game_execution_scheduler.work_is_pending());
            // $06:83A7-83A8 DEX : BPL (30), taken (+6) for every slot but 0.
            crate::cycle_ledger::charge(if k > 0 { 30 + 6 } else { 30 });
        }
        self.complete_sprite_main_after_all_slots();
        if self.game_execution_scheduler.work_suspends_translated_call_stack() {
            return;
        }
        let suffix_nmi_slices =
            std::mem::take(&mut self.dungeon_room_load_module_suffix_nmi_slices);
        if suffix_nmi_slices != 0 {
            self.game_execution_scheduler
                .schedule_cpu_timed_work_before_trailing_nmi(
                    GameWorkContinuation::FinishDungeonSupertileTransition {
                        work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
                    },
                    suffix_nmi_slices,
                );
        }
    }

    /// Advance an already-suspended descending `Sprite_Main` slot loop to a
    /// later source statement without replaying its prefix or completing its
    /// caller.  Both boundaries name calls which have returned, so the exact
    /// semantic delta is the intervening descending slot range.
    pub(in crate::zelda_rtl) fn advance_sprite_main_after_slot_boundary(
        &mut self,
        completed_slot: u8,
        newly_completed_slot: u8,
    ) {
        assert!(
            newly_completed_slot < completed_slot,
            "Sprite_Main source boundary did not advance: {completed_slot} -> {newly_completed_slot}",
        );
        // Cycle ledger: a partial Sprite_Main $06:8328 body (its walk only).
        let _scope = crate::cycle_ledger::routine(0x06_8328);
        for slot in (newly_completed_slot..completed_slot).rev() {
            self.sprite_system_mut().set_cur_object_index(slot);
            // $06:83A1-83A4 STX $0fa0 : JSR Sprite_ExecuteSingle (78).
            crate::cycle_ledger::charge(78);
            self.sprite_execute_single(usize::from(slot));
            // $06:83A7-83A8 DEX : BPL (30), taken (+6) for every slot but 0.
            crate::cycle_ledger::charge(if slot > 0 { 30 + 6 } else { 30 });
        }
    }

    // void Sprite_ExecuteSingle(int k) {  // 8684e2
    //   uint8 st = sprite_state[k];
    //   if (st != 0)
    //     Sprite_TimersAndOam(k);
    //   kSprite_ExecuteSingle[st](k);
    // }
    pub(in crate::zelda_rtl) fn sprite_execute_single(&mut self, k: usize) {
        // Cycle ledger: Sprite_ExecuteSingle $06:84E2 (m8 x8).
        let _scope = crate::cycle_ledger::routine(0x06_84e2);
        let st = self.sprite_slot_view(k).state();
        // $06:84E2-84E5 LDA $dd0,x : BEQ Sprite_inactiveSprite (48).
        crate::cycle_ledger::charge(48);
        self.sprite_execute_single_after_entry(k,st);
    }

    fn sprite_execute_single_after_entry(&mut self,k:usize,st:u8) {
        let _scope = crate::cycle_ledger::routine_continuation(0x06_84e2);
        if st != 0 {
            // $06:84E7-84EE PHA, JSR Sprite_TimersAndOam, PLA, CMP #$09, BEQ
            // not taken (128; the callee charges its own body).
            crate::cycle_ledger::charge(128);
            self.sprite_timers_and_oam(k);
        } else {
            // BEQ taken into Sprite_inactiveSprite, which charges itself.
            crate::cycle_ledger::charge(6);
        }
        self.sprite_execute_single_after_timers(k, st);
    }

    pub(super) fn sprite_execute_single_after_timers(&mut self, k: usize, st: u8) {
        // The dispatch after the timers: state 9 takes the $06:84EE BEQ (+6)
        // to $06:850C JMP SpriteActive_Main (24); every other nonzero state
        // goes through $06:84F0 JSL JumpTableLocal (62) into the
        // twelve-entry table at $06:84F4: JumpTableLocal $00:8781 is a JSL
        // target whose own frame closes when it pulls the return address
        // (52), and the rest of its body (414 - 52) runs in this scope.
        // State 0 was dispatched by the BEQ charged at the entry.
        match st {
            0 => {}
            9 => crate::cycle_ledger::charge(6 + 24),
            _ => {
                crate::cycle_ledger::charge(62);
                crate::cycle_ledger::charge_routine(0x00_8781, 52);
                crate::cycle_ledger::charge(414 - 52);
            }
        }
        // $06:84F4's state body (or $850C's active tail), after timers and
        // dispatch. A body may change its slot's type/state; attribute it to
        // the source selection that entered it, without adding any charge.
        let _body = crate::cycle_ledger::sprite_body_profile(
            self.sprite_slot_view(k).sprite_type(), st, k as u8);
        match st {
            0 => self.sprite_inactive_sprite(k),
            1 => self.sprite_module_fall1(k),
            2 => self.sprite_module_poof(k),
            3 => self.sprite_module_drown(k),
            4 => self.sprite_module_explode(k),
            5 => self.sprite_module_fall2(k),
            6 => self.sprite_module_die(k),
            7 => self.sprite_module_burn(k),
            8 => self.sprite_module_initialize(k),
            9 => self.sprite_active_main(k),
            10 => self.sprite_module_carried(k),
            11 => self.sprite_module_stunned(k),
            _ => self.sprite_active_main(k),
        }
    }

    /// Cycle ledger for a CPU-boundary lane that enters a slot's
    /// Sprite_ExecuteSingle $06:84E2 body outside `sprite_execute_single`:
    /// opens the routine scope (bind it to a local that lives across the
    /// lane's timer and handler calls) and charges the entry blocks the ROM
    /// ran before the timers ($06:84E2-84EE, 48 + 128; the slot is active).
    /// With `dispatched`, the dispatch after the timers is charged too
    /// (state 9: BEQ taken + JMP SpriteActive_Main, 6 + 24; any other state:
    /// JSL JumpTableLocal 62, its own frame 52, the rest of its body 362 in
    /// this scope). The handler dispatch blocks that a lane bypasses
    /// (SpriteActive_Main's 260 and the bank-5 bounce) are charged only by
    /// `sprite_active_main` itself.
    #[must_use = "bind the scope to a local so it lives until the lane returns"]
    pub(in crate::zelda_rtl) fn sprite_execute_single_lane_scope(
        &self,
        k: usize,
        dispatched: bool,
    ) -> crate::cycle_ledger::RoutineScope {
        let scope = crate::cycle_ledger::routine(0x06_84e2);
        let native = self
            .native_sprite_timer_handoff
            .filter(|(slot, ..)| usize::from(*slot) == k);
        crate::cycle_ledger::charge(if native.is_some() { 60 } else { 48 + 128 });
        if dispatched {
            if native.map_or_else(|| self.sprite_slot_view(k).state(), |(_, state, _)| state) == 9 {
                crate::cycle_ledger::charge(6 + 24);
            } else {
                crate::cycle_ledger::charge(62);
                crate::cycle_ledger::charge_routine(0x00_8781, 52);
                crate::cycle_ledger::charge(414 - 52);
            }
        }
        scope
    }

    /// Cycle ledger for a lane resuming inside Sprite_TimersAndOam $06:83F2:
    /// the routine scope only; the pieces charge their own blocks.
    #[must_use = "bind the scope to a local so it lives across the timer pieces"]
    pub(in crate::zelda_rtl) fn sprite_timers_and_oam_lane_scope(&self) -> crate::cycle_ledger::RoutineScope {
        crate::cycle_ledger::routine(0x06_83f2)
    }

    // Cycle ledger: Oam_AllocateFromRegionA..F ($0D:BA80/84/88/8C/90/94, m8
    // x8) are six two-instruction entries (LDY #region : BRA, 38; BRA's 22
    // already includes the taken cycles) into the shared body at $0D:BA96
    // (PHB PHK PLB JSR Oam_GetBufferPosition PLB RTL, 190); region F sits
    // directly above the body, so its entry is LDY alone (16).
    // Oam_GetBufferPosition charges its own body.
    const OAM_ALLOCATE_FROM_REGION_ENTRY_MASTER_CYCLES: u64 = 38 + 190;
    const OAM_ALLOCATE_FROM_REGION_F_MASTER_CYCLES: u64 = 16 + 190;

    pub(in crate::zelda_rtl) fn oam_allocate_from_region_a(&mut self, num: u8) -> u16 {
        let _scope = crate::cycle_ledger::routine(0x0d_ba80);
        crate::cycle_ledger::charge(Self::OAM_ALLOCATE_FROM_REGION_ENTRY_MASTER_CYCLES);
        self.oam_get_buffer_position(num, 0)
    }

    pub(in crate::zelda_rtl) fn oam_allocate_from_region_b(&mut self, num: u8) -> u16 {
        let _scope = crate::cycle_ledger::routine(0x0d_ba84);
        crate::cycle_ledger::charge(Self::OAM_ALLOCATE_FROM_REGION_ENTRY_MASTER_CYCLES);
        self.oam_get_buffer_position(num, 2)
    }

    pub(in crate::zelda_rtl) fn oam_allocate_from_region_c(&mut self, num: u8) -> u16 {
        let _scope = crate::cycle_ledger::routine(0x0d_ba88);
        crate::cycle_ledger::charge(Self::OAM_ALLOCATE_FROM_REGION_ENTRY_MASTER_CYCLES);
        self.oam_get_buffer_position(num, 4)
    }

    pub(in crate::zelda_rtl) fn oam_allocate_from_region_d(&mut self, num: u8) -> u16 {
        let _scope = crate::cycle_ledger::routine(0x0d_ba8c);
        crate::cycle_ledger::charge(Self::OAM_ALLOCATE_FROM_REGION_ENTRY_MASTER_CYCLES);
        self.oam_get_buffer_position(num, 6)
    }

    pub(in crate::zelda_rtl) fn oam_allocate_from_region_e(&mut self, num: u8) -> u16 {
        let _scope = crate::cycle_ledger::routine(0x0d_ba90);
        crate::cycle_ledger::charge(Self::OAM_ALLOCATE_FROM_REGION_ENTRY_MASTER_CYCLES);
        self.oam_get_buffer_position(num, 8)
    }

    pub(in crate::zelda_rtl) fn oam_allocate_from_region_f(&mut self, num: u8) -> u16 {
        let _scope = crate::cycle_ledger::routine(0x0d_ba94);
        crate::cycle_ledger::charge(Self::OAM_ALLOCATE_FROM_REGION_F_MASTER_CYCLES);
        self.oam_get_buffer_position(num, 10)
    }

    // void Sprite_TimersAndOam(int k) {  // 8683f2
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_timers_and_oam(&mut self, k: usize) {
        if let Some((slot, _, tail)) = self.native_sprite_timer_handoff {
            if usize::from(slot) == k {
                self.native_sprite_timer_handoff = None;
                self.finish_native_sprite_timers(k, tail);
                return;
            }
        }
        // Cycle ledger: Sprite_TimersAndOam $06:83F2 (m8 x8, X <= 15 so no
        // abs,x page crossing). The blocks are charged inside the pieces
        // below so the CPU-boundary lanes that run a piece charge that piece;
        // a piece run outside this call lands in its caller's scope.
        let _scope = crate::cycle_ledger::routine(0x06_83f2);
        self.sprite_timers_and_oam_through_timer_decrements(k);
        self.sprite_timers_and_oam_after_timer_decrements(k);
    }

    pub(in crate::zelda_rtl) fn sprite_timers_and_oam_through_timer_decrements(&mut self, k: usize) {
        self.sprite_timers_and_oam_through_primary_timer_decrements(k);
        self.sprite_timers_and_oam_after_primary_through_timer_decrements(k);
    }

    pub(in crate::zelda_rtl) fn sprite_timers_and_oam_through_primary_timer_decrements(&mut self, k: usize) {
        self.sprite_timers_and_oam_through_main_and_aux1_timer_decrements(k);
        self.sprite_timers_and_oam_after_main_and_aux1_through_primary_timer_decrements(k);
    }

    pub(in crate::zelda_rtl) fn sprite_timers_and_oam_through_main_and_aux1_timer_decrements(
        &mut self,
        k: usize,
    ) {
        self.sprite_timers_and_oam_through_main_timer_decrement(k);
        self.sprite_timers_and_oam_aux1_timer_decrement(k);
    }

    pub(in crate::zelda_rtl) fn sprite_timers_and_oam_through_zero_hit_timer_clear(&mut self, k: usize) {
        self.sprite_timers_and_oam_through_primary_timer_decrements(k);
        assert_eq!(self.sprite_slot_view(k).hit_timer() & 0x7f, 0);
        // $06:8441-8446 LDA $ef0,x AND #$7f BEQ taken (64 + 6) and the first
        // store of $06:8496 STZ $ef0,x (38); the remainder of that block runs
        // in the continuation.
        crate::cycle_ledger::charge(64 + 6 + 38);
        self.sprite_slot_view_mut(k).set_hit_timer(0);
    }

    pub(in crate::zelda_rtl) fn sprite_timers_and_oam_through_main_timer_decrement(&mut self, k: usize) {
        self.sprite_timers_and_oam_through_oam_allocation(k);
        self.sprite_timers_and_oam_after_oam_allocation_through_main_timer_decrement(k);
    }

    pub(in crate::zelda_rtl) fn sprite_timers_and_oam_through_oam_allocation(&mut self, k: usize) {
        self.sprite_timers_and_oam_through_coordinates(k);
        self.sprite_timers_and_oam_after_coordinates_through_oam_allocation(k);
    }

    pub(in crate::zelda_rtl) fn sprite_timers_and_oam_through_coordinates(&mut self, k: usize) {
        // $06:83F2 JSR Sprite_Get16BitCoords (46); the shared callee is four
        // abs,x/abs coordinate moves and RTS (298). OAM allocation is later.
        crate::cycle_ledger::charge(46);
        let x = self.sprite_get_x(k).to_le_bytes();
        let y = self.sprite_get_y(k).to_le_bytes();
        self.advance_sprite_coordinate_moves([x[0], x[1], y[0], y[1]], 0, 8);
        crate::cycle_ledger::charge_routine(0x06_84c1, 42);
    }

    pub(super) fn advance_sprite_coordinate_moves(&mut self, coordinates: [u8; 4], from: u8, to: u8) {
        assert!(from <= to && to <= 8);
        let _scope = crate::cycle_ledger::routine(0x06_84c1);
        // Four LDA abs,x / STA abs pairs, all 32 master cycles. Slots are
        // bounded below 16, so none of the indexed reads crosses a page.
        for instruction in from..to {
            crate::cycle_ledger::charge(32);
            if instruction & 1 == 0 { continue; }
            let index = usize::from(instruction / 2);
            let value = coordinates[index];
            match index {
                0 => self.sprite_workspace_mut().set_current_sprite_x_low(value),
                1 => {
                    let low = self.game_state.sprites.workspace.current_sprite_x_low();
                    self.sprite_workspace_mut().set_current_sprite_x(u16::from_le_bytes([low, value]));
                }
                2 => self.sprite_workspace_mut().set_current_sprite_y_low(value),
                3 => {
                    let low = self.game_state.sprites.workspace.current_sprite_y_low();
                    self.sprite_workspace_mut().set_current_sprite_y(u16::from_le_bytes([low, value]));
                }
                _ => unreachable!(),
            }
        }
    }

    pub(super) fn sprite_timers_and_oam_after_coordinates_through_oam_allocation(&mut self, k: usize) {
        // $06:83F5-8400 LDA $e40,x AND #$1f INC ASL ASL LDY $0fb3 BEQ (138).
        crate::cycle_ledger::charge(138);
        let num = ((self.sprite_slot_view(k).flags2() & 0x1f).wrapping_add(1)).wrapping_mul(4);
        if self.game_state.oam.has_sprite_sorting() {
            // $06:8402-8405 LDY $f20,x : BEQ (48), then either $06:8407 JSL
            // RegionF : BRA (84) or BEQ taken (+6) into $06:840D JSL RegionD
            // : BRA (84); BRA's 22 already includes the taken cycles. The
            // allocators charge their own bodies.
            if self.sprite_slot_view(k).floor() != 0 {
                crate::cycle_ledger::charge(48 + 84);
                self.oam_allocate_from_region_f(num);
            } else {
                crate::cycle_ledger::charge(48 + 6 + 84);
                self.oam_allocate_from_region_d(num);
            }
        } else {
            // BEQ taken (+6) into $06:8413 JSL Oam_AllocateFromRegionA (62).
            crate::cycle_ledger::charge(6 + 62);
            self.oam_allocate_from_region_a(num);
        }

    }

    fn sprite_timers_and_oam_after_oam_allocation_through_main_timer_decrement(&mut self, k: usize) {

        // $06:8417-841C LDA $11 ORA $0fc1 BEQ (72): taken (+6) into the timer
        // blocks when both are zero, else $06:841E JMP $84A4 (24). The ROM
        // tests this once; the later pieces re-test it in Rust for free.
        let timers_run =
            (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0;
        crate::cycle_ledger::charge(if timers_run { 72 + 6 } else { 72 + 24 });
        if timers_run {
            // $06:8421-8424 LDA $df0,x : BEQ (48) + $06:8426 DEC abs,x (52)
            // or BEQ taken (+6).
            crate::cycle_ledger::charge(if self.sprite_slot_view(k).delay_main() != 0 {
                48 + 52
            } else {
                48 + 6
            });
        }
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0
            && self.sprite_slot_view(k).delay_main() != 0
        {
            let value = self.sprite_slot_view(k).delay_main().wrapping_sub(1);
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
    }

    pub(in crate::zelda_rtl) fn sprite_timers_and_oam_after_oam_allocation(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x06_83f2);
        self.sprite_timers_and_oam_after_oam_allocation_through_main_timer_decrement(k);
        self.sprite_timers_and_oam_aux1_timer_decrement(k);
        self.sprite_timers_and_oam_after_main_and_aux1_through_primary_timer_decrements(k);
        self.sprite_timers_and_oam_after_primary_through_timer_decrements(k);
        self.sprite_timers_and_oam_after_timer_decrements(k);
    }

    pub(super) fn sprite_timers_and_oam_aux1_timer_decrement(&mut self, k: usize) {
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0 {
            // $06:8429-842C LDA $e00,x : BEQ (48) + $06:842E DEC abs,x (52)
            // or BEQ taken (+6).
            crate::cycle_ledger::charge(if self.sprite_slot_view(k).delay_aux1() != 0 {
                48 + 52
            } else {
                48 + 6
            });
        }
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0
            && self.sprite_slot_view(k).delay_aux1() != 0
        {
            let value = self.sprite_slot_view(k).delay_aux1().wrapping_sub(1);
            self.sprite_slot_view_mut(k).set_delay_aux1(value);
        }
    }

    pub(super) fn sprite_timers_and_oam_after_main_and_aux1_through_primary_timer_decrements(
        &mut self,
        k: usize,
    ) {
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0 {
            // $06:8431-8434 LDA $e10,x : BEQ (48) + $06:8436 DEC (52) or +6.
            if self.sprite_slot_view(k).delay_aux2() != 0 {
                crate::cycle_ledger::charge(48 + 52);
                let value = self.sprite_slot_view(k).delay_aux2().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_aux2(value);
            } else {
                crate::cycle_ledger::charge(48 + 6);
            }
            // $06:8439-843C LDA $ee0,x : BEQ (48) + $06:843E DEC (52) or +6.
            if self.sprite_slot_view(k).delay_aux3() != 0 {
                crate::cycle_ledger::charge(48 + 52);
                let value = self.sprite_slot_view(k).delay_aux3().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_aux3(value);
            } else {
                crate::cycle_ledger::charge(48 + 6);
            }
        }
    }

    pub(super) fn sprite_timers_and_oam_after_primary_through_timer_decrements(&mut self, k: usize) {
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0 {
            self.sprite_timers_and_oam_after_primary_through_hit_timer(k);
            self.sprite_timers_and_oam_after_hit_through_timer_decrements(k);
        }
    }

    pub(in crate::zelda_rtl) fn sprite_timers_and_oam_after_primary_through_hit_timer(&mut self, k: usize) {
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0 {
            let timer = self.sprite_slot_view(k).hit_timer() & 0x7f;
            // $06:8441-8446 LDA $ef0,x AND #$7f BEQ (64).
            crate::cycle_ledger::charge(64);
            if timer != 0 {
                // $06:8448-844D LDY $dd0,x CPY #$09 BCC (64): taken (+6) for
                // states below 9.
                crate::cycle_ledger::charge(64);
                if self.sprite_slot_view(k).state() >= 9 {
                    // $06:844F-8451 CMP #$1f : BNE (32).
                    crate::cycle_ledger::charge(32);
                    if timer == 31 {
                        // $06:8453-8479 inline; charged in sprite_hit_timer31.
                        self.sprite_hit_timer31(k);
                        // $06:847A-847C CMP #$18 : BNE taken (32 + 6).
                        crate::cycle_ledger::charge(32 + 6);
                    } else if timer == 24 {
                        // BNE taken (+6), $06:847A-847C not taken (32),
                        // $06:847E JSR Sprite_MiniMoldorm_Recoil (46).
                        crate::cycle_ledger::charge(6 + 32 + 46);
                        self.sprite_mini_moldorm_recoil(k);
                    } else {
                        // BNE taken (+6), $06:847A-847C BNE taken (32 + 6).
                        crate::cycle_ledger::charge(6 + 32 + 6);
                    }
                } else {
                    crate::cycle_ledger::charge(6);
                }
                self.sprite_timers_and_oam_finish_hit_timer(k);
            } else {
                // BEQ taken (+6) into $06:8496-8499 STZ $ef0,x STZ $b89,x (76).
                crate::cycle_ledger::charge(6 + 76);
                let value = 0;
                self.sprite_slot_view_mut(k).set_hit_timer(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_object_priority(value);
            }
        }
    }

    fn sprite_timers_and_oam_finish_hit_timer(&mut self, k: usize) {
        // $06:8481-8486 LDA $ce2,x CMP #$fb BCS (64): not taken into
        // $06:8488-848E LDA ASL AND STA (100), else taken (+6).
        if self.sprite_slot_view(k).incoming_damage() < 251 {
            crate::cycle_ledger::charge(64 + 100);
            let value = ((u16::from(self.sprite_slot_view(k).hit_timer()) * 2) & 0x0e) as u8;
            self.sprite_slot_view_mut(k).set_object_priority(value);
        } else {
            crate::cycle_ledger::charge(64 + 6);
        }
        // $06:8491-8494 DEC $ef0,x : BRA (74; BRA's 22 already
        // includes the taken cycles).
        crate::cycle_ledger::charge(74);
        let value = self.sprite_slot_view(k).hit_timer().wrapping_sub(1);
        self.sprite_slot_view_mut(k).set_hit_timer(value);
    }

    pub(in crate::zelda_rtl) fn finish_native_sprite_timers(
        &mut self,
        k: usize,
        tail: crate::cycle_models::sprite_timers::SpriteTimersTail,
    ) {
        use crate::cycle_models::sprite_timers::SpriteTimersTail;
        if let SpriteTimersTail::Recoil(tail) = tail {
            let _timers = crate::cycle_ledger::routine(0x06_83f2);
            self.finish_native_sprite_recoil(k,tail);
            self.sprite_timers_and_oam_finish_hit_timer(k);
            self.sprite_timers_and_oam_after_hit_through_timer_decrements(k);
            self.sprite_timers_and_oam_after_timer_decrements(k);
        }
    }

    pub(super) fn sprite_execute_single_after_native_timers(
        &mut self,
        k: usize,
        state: u8,
        tail: crate::cycle_models::sprite_timers::SpriteTimersTail,
    ) {
        let _single = crate::cycle_ledger::routine(0x06_84e2);
        if let Some((slot, saved, saved_tail)) = self.native_sprite_timer_handoff {
            assert_eq!((usize::from(slot), saved, saved_tail), (k, state, tail));
            self.native_sprite_timer_handoff = None;
        }
        self.finish_native_sprite_timers(k, tail);
        crate::cycle_ledger::charge(60); // Saved state PLA, CMP #9, untaken BEQ.
        self.sprite_execute_single_after_timers(k, state);
    }

    pub(super) fn sprite_timers_and_oam_after_hit_through_timer_decrements(&mut self, k: usize) {
        // Entry proves the timer branch was taken before hit processing;
        // that processing may itself change the module's state.
        // $06:849C-849F LDA $f10,x : BEQ (48) + $06:84A1 DEC (52) or +6.
        if self.sprite_slot_view(k).delay_aux4() != 0 {
            crate::cycle_ledger::charge(48 + 52);
            let value = self.sprite_slot_view(k).delay_aux4().wrapping_sub(1);
            self.sprite_slot_view_mut(k).set_delay_aux4(value);
        } else {
            crate::cycle_ledger::charge(48 + 6);
        }
    }

    pub(super) fn sprite_timers_and_oam_after_timer_decrements(&mut self, k: usize) {
        let mut floor = self.game_state.player.follower_link.lower_level_state() as usize;
        // $06:84A4-84A8 LDY $ee CPY #$03 BEQ (56): taken (+6) on floor 3, else
        // $06:84AA LDY $f20,x (32). Then $06:84AD-84B8 LDA AND ORA abs,y STA
        // RTS (160).
        if floor != 3 {
            crate::cycle_ledger::charge(56 + 32 + 160);
            floor = self.sprite_slot_view(k).floor() as usize;
        } else {
            crate::cycle_ledger::charge(56 + 6 + 160);
        }
        let value = (self.sprite_slot_view(k).object_priority() & 0xcf)
            | SPRITE_TIMERS_AND_OAM_SPRITE_PRIOS[floor];
        self.sprite_slot_view_mut(k).set_object_priority(value);
    }

    pub(crate) fn native_sprite_property_table_byte(address: u32) -> Option<u8> {
        if (0x0db080..0x0db818).contains(&address) {
            let offset = (address - 0x0db080) as usize;
            return Some(sprite_init_value(
                offset / SPRITE_INIT_TABLE_LEN,
                (offset % SPRITE_INIT_TABLE_LEN) as u8,
            ));
        }
        None
    }

    pub(crate) fn native_oam_table_byte(address: u32) -> Option<u8> {
        if (0x0684b9..=0x0684bc).contains(&address) {
            return Some(SPRITE_TIMERS_AND_OAM_SPRITE_PRIOS[(address - 0x0684b9) as usize]);
        }
        let (words, offset): (&[u16], usize) = match address {
            0x0683c7..=0x0683d2 => (&OAM_RESET_REGION_BASES_OAM_RESET_REGION_BASES, (address - 0x0683c7) as usize),
            0x0dba9e..=0x0dbaa9 => (&OAM_GET_BUFFER_POSITION_LIMITS, (address - 0x0dba9e) as usize),
            0x0dbaaa..=0x0dbb09 => (&OAM_GET_BUFFER_POSITION_FALLBACKS, (address - 0x0dbaaa) as usize),
            _ => return None,
        };
        Some(words[offset / 2].to_le_bytes()[offset & 1])
    }

    pub(in crate::zelda_rtl) fn oam_get_buffer_position(&mut self, num: u8, y: u8) -> u16 {
        // Cycle ledger: Oam_GetBufferPosition $0D:BB0A (entry m8 x8, JSR
        // target; Y <= 10 and the fallback index stays below 96, so no abs,y
        // page crossing). $0D:BB0A-BB1C STA STZ REP LDA $fe0,y STA CLC ADC
        // CMP $ba9e,y BCC (252): a fitting request takes the BCC (+6) into
        // $0D:BB41 STA $fe0,y (46); an overflow runs $0D:BB1E-BB3F (520, BRA
        // included). Then $0D:BB44-BB5A (354, RTS included).
        let _scope = crate::cycle_ledger::routine(0x0d_bb0a);
        let region = (y >> 1) as usize;
        let mut pstart = self.game_state.oam.region_base_word(region);
        let p = pstart.wrapping_add(num as u16);
        crate::cycle_ledger::charge(if p >= OAM_GET_BUFFER_POSITION_LIMITS[region] {
            252 + 520 + 354
        } else {
            252 + 6 + 46 + 354
        });
        if p >= OAM_GET_BUFFER_POSITION_LIMITS[region] {
            let alloc = self.game_state.oam.region_alloc_counter(region);
            let j = alloc & 7;
            self.oam_state_mut()
                .set_region_alloc_counter(region, alloc.wrapping_add(1));
            pstart = OAM_GET_BUFFER_POSITION_FALLBACKS[region * 8 + j as usize];
        } else {
            self.oam_state_mut().set_region_base_word(region, p);
        }
        let oam = 0x0800 + pstart;
        self.oam_state_mut()
            .set_current_extended_pointer(0x0a20 + (pstart >> 2));
        self.oam_state_mut().set_current_pointer(oam);
        oam
    }
}
