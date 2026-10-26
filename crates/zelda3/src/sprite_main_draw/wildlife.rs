//! Cuccos and their continuations, birds, crabs, and small roaming creatures.

use super::*;
use crate::zelda_rtl::sprite::PrepOamCoordsRet;
use crate::types::sign8;

impl ZeldaState {

    // -----------------------------------------------------------------------
    // void Crab_Draw(int k) {  // 859510
    //   Two mirrored 16x16 body tiles plus the common shadow.
    // }
    pub(super) fn crab_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let d = (self.sprite_slot_view(k).graphics() as usize) * 2;
        for i in (0..2).rev() {
            let j = d + i;
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(CRAB_DRAW_X_OFFSETS[j] as u16),
                y,
                CRAB_DRAW_CHARS[j],
                CRAB_DRAW_FLAGS[j] | flags,
                2,
            );
            oam += 4;
        }
        let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Fish_Draw(int k) {  // 9d8483
    //   Two fish body tiles plus three splash tiles at z-adjusted y.
    // }
    pub(in crate::zelda_rtl) fn fish_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).graphics() == 0 {
            self.sprite_prep_oam_coord(k);
            return;
        }
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        self.sprite_workspace_mut()
            .set_current_sprite_x(cur_x.wrapping_add(4));
        let base = usize::from(self.sprite_slot_view(k).graphics().wrapping_sub(1)) * 2;
        self.sprite_draw_multiple(k, &FISH_DRAW_FRAMES[base..base + 2]);
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let z = self.sprite_slot_view(k).z();
        self.sprite_workspace_mut()
            .set_current_sprite_y(cur_y.wrapping_add(z as u16));
        let j = (z >> 2).min(2) as usize;
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(8));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(2));
        self.sprite_draw_multiple(k, &FISH_DRAW_FRAMES2[j * 3..j * 3 + 3]);
        self.sprite_get16_bit_coords(k);
    }

    // -----------------------------------------------------------------------
    // void ChimneySmoke_Draw(int k) {  // 9d8531
    //   Four smoke tiles selected by low sprite_graphics bit.
    // }
    pub(super) fn chimney_smoke_draw(&mut self, k: usize) {
        let base = ((self.sprite_slot_view(k).graphics() & 1) as usize) * 4;
        self.sprite_draw_multiple(k, &CHIMNEY_SMOKE_DRAW_FRAMES[base..base + 4]);
    }

    // -----------------------------------------------------------------------
    // void Vulture_Draw(int k) {  // 9ddd5e
    //   Two large wing/body tiles plus common shadow.
    // }
    pub(super) fn vulture_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let mut info = self.sprite_draw_multiple(k, &VULTURE_DRAW_FRAMES[base..base + 2]);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Raven(int k) {  // 9ddd85
    //   Raven wait, ascend, attack, and flee state machine.
    // }
    pub(super) fn sprite_raven(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).or_object_priority(0x30);
        // ROM $1D:DD8D JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let r = self.sprite_is_right_of_link(k);
                let value = (self.sprite_slot_view(k).oam_flags() & !0x40) | r.a.wrapping_mul(0x40);
                self.sprite_slot_view_mut(k).set_oam_flags(value);
                let x = i32::from(self.game_state.player.follower_link.x())
                    - i32::from(self.game_state.sprites.workspace.current_sprite_x());
                let y = i32::from(self.game_state.player.follower_link.y())
                    - i32::from(self.game_state.sprites.workspace.current_sprite_y());
                if ((x + 0x50 + i32::from(x >= 0)) as u16) < 0x00a0
                    && ((y + 0x58 + i32::from(y >= 0)) as u16) < 0x00a0
                {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 24;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x1e);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let j = usize::from(self.sprite_slot_view(k).a());
                    let value = RAVEN_ASCEND_TIMERS[j];
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_apply_speed_towards_link(k, 32);
                }
                self.sprite_slot_view_mut(k).add_z(1);
                let value = ((self.game_state.frame.frame_counter >> 1) & 1).wrapping_add(1);
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0
                    && !(self.game_state.world.region.is_in_dark_world()
                        && self.sprite_slot_view(k).a() != 0)
                {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
                self.raven_fly(k, false);
            }
            3 => {
                self.raven_fly(k, true);
            }
            _ => {}
        }
    }

    fn raven_fly(&mut self, k: usize, fleeing: bool) {
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 1) == 0 {
            let mut pt = self.sprite_project_speed_towards_link(k, if fleeing { 48 } else { 32 });
            if fleeing {
                pt.x = pt.x.wrapping_neg();
                pt.y = pt.y.wrapping_neg();
            }
            let dx = self.sprite_slot_view(k).x_velocity().wrapping_sub(pt.x);
            if dx != 0 {
                let value = self
                    .sprite_slot_view(k)
                    .x_velocity()
                    .wrapping_add(if sign8(dx) { 1 } else { 0xff });
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            let dy = self.sprite_slot_view(k).y_velocity().wrapping_sub(pt.y);
            if dy != 0 {
                let value = self
                    .sprite_slot_view(k)
                    .y_velocity()
                    .wrapping_add(if sign8(dy) { 1 } else { 0xff });
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
        }
        let value = ((self.game_state.frame.frame_counter >> 1) & 1).wrapping_add(1);
        self.sprite_slot_view_mut(k).set_graphics(value);
        let j = (self.sprite_slot_view(k).x_velocity() >> 7) & 1;
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40) | j.wrapping_mul(0x40);
        self.sprite_slot_view_mut(k).set_oam_flags(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_0B_Cucco(int k) {  // 86a5c2
    pub(in crate::zelda_rtl) fn sprite_0_b_cucco(&mut self, k: usize) {
        let work = self.begin_cucco_draw(k);
        assert!(self.advance_or_suspend_cucco_draw(work, None, self.sprite_main_cpu_caller));
    }

    pub(super) fn begin_cucco_draw(&self, k: usize) -> CuccoDrawContinuation {
        CuccoDrawContinuation {
            slot: u8::try_from(k).expect("sprite slot fits a byte"),
            prefix: crate::cycle_models::cucco::DrawPrefixWork::new(),
            draw: None,
        }
    }

    /// Advance from Cucco entry without executing the post-draw suffix. The caller must
    /// supply its own running clock; this method never estimates a raster or
    /// reconstructs progress from a source PC. Coordinate preparation, its
    /// offscreen KillSelf call, and the double return share this retained owner.
    pub(in crate::zelda_rtl) fn advance_cucco_draw(
        &mut self,
        work: &mut CuccoDrawContinuation,
        mut budget: Option<&mut CpuCycleBudget>,
    ) -> CpuWorkAdvance {
        while let Some(cycles) = self.retire_cucco_draw_instruction(work) {
            if let Some(budget) = budget.as_deref_mut() {
                let advance = budget.advance_instruction(cycles);
                if advance.reached_boundary().is_some() {
                    return advance;
                }
            }
        }
        CpuWorkAdvance::Complete
    }

    pub(super) fn retire_cucco_draw_instruction(&mut self, work: &mut CuccoDrawContinuation) -> Option<u32> {
        use crate::cycle_models::cucco::Publication;
        let k = usize::from(work.slot);
        let slot = self.sprite_slot_view(k);
        if let Some(instruction) = work.prefix.step(slot.x_velocity(), slot.oam_flags()) {
            crate::cycle_ledger::charge_routine(0x06_a5c2, u64::from(instruction.master_cycles));
            match instruction.publication {
                Some(Publication::Flags(value)) => self.sprite_slot_view_mut(k).set_oam_flags(value),
                Some(Publication::EnterDraw) => work.draw = Some(self.begin_single_large_draw(k)),
                None => {}
            }
            return Some(instruction.master_cycles);
        }
        if let Some(draw) = work.draw.as_mut() {
            if let Some(cycles) = self.retire_single_large_draw_instruction(draw) {
                return Some(cycles);
            }
            work.draw = None;
        }
        None
    }

    /// Consume the returned call before executing the first post-draw load.
    /// A held draw cannot publish the suffix, and resume cannot redo facing,
    /// coordinate preparation, or any retired draw instruction.
    pub(in crate::zelda_rtl) fn complete_cucco_draw(&mut self, work: CuccoDrawContinuation) {
        assert!(work.prefix.is_complete() && work.draw.is_none(),
            "Cucco caller resumed before its draw returned");
        self.complete_cucco_after_large_draw(usize::from(work.slot));
    }

    /// Transfer the interrupted native call to Sprite_Main's existing stack
    /// owner. Exactly one acceptance boundary was reached by this budget;
    /// the typed caller decides which NMI-resume lane owns its return.
    pub(super) fn advance_or_suspend_cucco_draw(
        &mut self,
        mut work: CuccoDrawContinuation,
        budget: Option<&mut CpuCycleBudget>,
        caller: SpriteMainCpuCaller,
    ) -> bool {
        match self.advance_cucco_draw(&mut work, budget) {
            CpuWorkAdvance::Complete => {
                self.complete_cucco_draw(work);
                true
            }
            CpuWorkAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::CpuNmiAcceptance, ..
            } => {
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::CuccoDraw { work }, 1, caller);
                false
            }
            CpuWorkAdvance::ReachedBoundary { .. } => {
                panic!("Cucco suspension requires a CPU NMI acceptance budget")
            }
        }
    }

    fn complete_cucco_after_large_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).head_direction() != 0 {
            let value = 0x3d;
            self.sprite_slot_view_mut(k).set_sprite_type(value);
            self.sprite_prep_load_properties(k);
            self.sprite_slot_view_mut(k).add_subtype(1);
            let value = 48;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.set_sound_effect_1(21);
            let value = 21;
            self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            return;
        }
        let mut helper_ordinal = 0;
        if self.sprite_slot_view(k).state() == 10 {
            let value = 3;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            if self.game_state.frame.submodule == 0 {
                if self.chicken_incr_subtype2_for_draw(k, 3, 0, CuccoSubtypeContinuation::State10) {
                    return;
                }
                self.cucco_draw_panic(k);
                if (self.game_state.frame.frame_counter & 0x0f) == 0 {
                    self.bawk_bawk(k);
                }
                helper_ordinal = 1;
            }
        }
        self.complete_cucco_after_state10_branch(k, helper_ordinal);
    }

    /// Resume the state-10 Cucco immediately after its initial `subtype2 += 3`
    /// publication. This is the exact remaining C suffix; neither the slot
    /// timers/OAM prefix nor the three subtype stores are replayed.
    pub(in crate::zelda_rtl) fn complete_cucco_after_subtype_increments(
        &mut self,
        k: usize,
        completed: u8,
        total: u8,
        continuation: CuccoSubtypeContinuation,
    ) {
        assert!(completed <= total);
        for _ in completed..total {
            self.chicken_add_subtype2_for_draw(k, 1);
        }
        self.chicken_publish_graphics_for_draw(k);
        self.sprite_return_if_lifted(k);
        self.complete_cucco_helper_caller(k, continuation);
    }

    /// Advance an already-suspended `Chicken_IncrSubtype2` call to a later
    /// source-observed increment without publishing graphics or running its
    /// caller suffix. This is used when consecutive live-owner host receipts
    /// refine the same semantic helper invocation.
    pub(in crate::zelda_rtl) fn advance_cucco_subtype_increment_checkpoint(
        &mut self,
        k: usize,
        completed: u8,
        refined_completed: u8,
        total: u8,
    ) {
        assert!(completed <= refined_completed);
        assert!(refined_completed <= total);
        for _ in completed..refined_completed {
            self.chicken_add_subtype2_for_draw(k, 1);
        }
    }

    /// Advance the same suspended helper through its remaining increments and
    /// graphics publication, stopping before `Sprite_ReturnIfLifted` and the
    /// caller-specific suffix.
    pub(in crate::zelda_rtl) fn advance_cucco_subtype_checkpoint_to_graphics(
        &mut self,
        k: usize,
        completed: u8,
        total: u8,
    ) {
        self.advance_cucco_subtype_increment_checkpoint(k, completed, total, total);
        self.chicken_publish_graphics_for_draw(k);
    }

    pub(in crate::zelda_rtl) fn complete_cucco_after_graphics_publication(
        &mut self,
        k: usize,
        continuation: CuccoSubtypeContinuation,
    ) {
        self.sprite_return_if_lifted(k);
        self.complete_cucco_helper_caller(k, continuation);
    }

    fn complete_cucco_helper_caller(&mut self, k: usize, continuation: CuccoSubtypeContinuation) {
        match continuation {
            CuccoSubtypeContinuation::State10 => {
                self.cucco_draw_panic(k);
                if (self.game_state.frame.frame_counter & 0x0f) == 0 {
                    self.bawk_bawk(k);
                }
                self.complete_cucco_after_state10_branch(k, 1);
            }
            CuccoSubtypeContinuation::Flee | CuccoSubtypeContinuation::CarriedLanding => {
                self.cucco_draw_panic(k);
            }
            CuccoSubtypeContinuation::ActiveC
            | CuccoSubtypeContinuation::Hopping
            | CuccoSubtypeContinuation::CarriedAirborne => {}
        }
    }

    fn complete_cucco_after_state10_branch(&mut self, k: usize, helper_ordinal: u8) {
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).c() != 0 {
            self.sprite_slot_view_mut(k).or_oam_flags(0x10);
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterActiveCuccoX {
                    slot,
                    helper_ordinal: expected_ordinal,
                }) if slot == k as u8 && expected_ordinal == helper_ordinal
            ) {
                assert_ne!(
                    self.sprite_slot_view(k).x_velocity(),
                    0,
                    "an active-Cucco X publication requires Sprite_MoveX to enter its source body",
                );
                self.sprite_move_x(k);
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                    slot,
                    helper_ordinal: expected_ordinal,
                    y_low: None,
                    y_high: None,
                }) if slot == k as u8 && expected_ordinal == helper_ordinal
            ) {
                let (y_low, y_high) = self.sprite_move_xy_through_y_subpixel(k);
                self.sprite_main_cpu_boundary =
                    Some(SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                        slot: k as u8,
                        helper_ordinal,
                        y_low: Some(y_low),
                        y_high: Some(y_high),
                    });
                return;
            }
            self.sprite_move_xy(k);
            self.complete_active_cucco_after_movement(k, helper_ordinal);
        } else {
            let value = 255;
            self.sprite_slot_view_mut(k).set_health(value);
            if self.sprite_slot_view(k).b() >= 35 {
                self.cucco_summon_avenger(k);
            }
            if self.sprite_slot_view(k).f() != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_f(value);
                if self.sprite_slot_view(k).b() < 35 {
                    self.sprite_slot_view_mut(k).add_b(1);
                    self.bawk_bawk(k);
                }
                let value = 2;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            self.sprite_check_damage_from_link(k);
            match self.sprite_slot_view(k).ai_state() {
                0 => self.cucco_calm(k),
                1 => {
                    self.chicken_hopping(k, helper_ordinal);
                }
                2 => {
                    self.cucco_flee(k, helper_ordinal);
                }
                3 => {
                    self.cucco_carried(k, helper_ordinal);
                }
                _ => {}
            }
        }
    }

    fn complete_active_cucco_after_movement(&mut self, k: usize, helper_ordinal: u8) {
        self.advance_active_cucco_after_movement_to_helper(k);
        self.chicken_incr_subtype2_for_draw(
            k,
            4,
            helper_ordinal,
            CuccoSubtypeContinuation::ActiveC,
        );
    }

    pub(in crate::zelda_rtl) fn complete_active_cucco_after_x(&mut self, k: usize, helper_ordinal: u8) {
        self.sprite_move_y(k);
        self.complete_active_cucco_after_movement(k, helper_ordinal);
    }

    pub(in crate::zelda_rtl) fn advance_active_cucco_x_to_y_subpixel(&mut self, k: usize) -> (u8, u8) {
        self.sprite_move_y_through_subpixel(k)
    }

    pub(in crate::zelda_rtl) fn advance_active_cucco_x_to_helper(&mut self, k: usize) {
        self.sprite_move_y(k);
        self.advance_active_cucco_after_movement_to_helper(k);
    }

    fn advance_active_cucco_after_movement_to_helper(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_z(12);
        self.sprite_slot_view_mut(k).set_ignore_projectile(12);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 7) == 0 {
            self.sprite_check_damage_to_link(k);
        }
    }

    pub(in crate::zelda_rtl) fn complete_active_cucco_after_y_subpixel(
        &mut self,
        k: usize,
        helper_ordinal: u8,
        y_low: u8,
        y_high: u8,
    ) {
        self.complete_sprite_move_y_after_subpixel(k, y_low, y_high);
        self.complete_active_cucco_after_movement(k, helper_ordinal);
    }

    pub(in crate::zelda_rtl) fn advance_active_cucco_y_subpixel_to_helper(
        &mut self,
        k: usize,
        y_low: u8,
        y_high: u8,
    ) {
        self.complete_sprite_move_y_after_subpixel(k, y_low, y_high);
        self.advance_active_cucco_after_movement_to_helper(k);
    }

    // Local duplicate of private dungeon NPC helper used by the original
    // top-level Cucco actor.
    pub(in crate::zelda_rtl) fn chicken_incr_subtype2_for_draw(
        &mut self,
        k: usize,
        increments: u8,
        helper_ordinal: u8,
        continuation: CuccoSubtypeContinuation,
    ) -> bool {
        if let Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
            slot,
            helper_ordinal: expected_ordinal,
            completed,
            total: 0,
            continuation: None,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 && expected_ordinal == helper_ordinal {
                assert!(
                    completed <= increments,
                    "source Cucco helper published increment {completed}, but native call has only {increments}",
                );
            }
        }
        for completed in 1..=increments {
            self.chicken_add_subtype2_for_draw(k, 1);
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                    slot,
                    helper_ordinal: expected_ordinal,
                    completed: expected_completed,
                    total: 0,
                    continuation: None,
                }) if slot == k as u8
                    && expected_ordinal == helper_ordinal
                    && expected_completed == completed
            ) {
                self.sprite_main_cpu_boundary =
                    Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                        slot: k as u8,
                        helper_ordinal,
                        completed,
                        total: increments,
                        continuation: Some(continuation),
                    });
                return true;
            }
        }
        self.chicken_publish_graphics_for_draw(k);
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                slot,
                helper_ordinal: expected_ordinal,
                continuation: None,
            }) if slot == k as u8 && expected_ordinal == helper_ordinal
        ) {
            self.sprite_main_cpu_boundary =
                Some(SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                    slot: k as u8,
                    helper_ordinal,
                    continuation: Some(continuation),
                });
            return true;
        }
        self.sprite_return_if_lifted(k);
        false
    }

    pub(in crate::zelda_rtl) fn chicken_add_subtype2_for_draw(&mut self, k: usize, j: u8) {
        self.sprite_slot_view_mut(k).add_subtype2(j);
    }

    pub(in crate::zelda_rtl) fn chicken_publish_graphics_for_draw(&mut self, k: usize) {
        let value = (self.sprite_slot_view(k).subtype2() >> 4) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_01_Vulture_bounce(int k) {  // 869473
    //   Dormant vulture that begins circling and tracking Link.
    // }
    pub(super) fn sprite_01_vulture_bounce(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).or_object_priority(0x30);
        self.vulture_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_slot_view_mut(k).add_subtype2(1);
                if self.sprite_slot_view(k).subtype2() == 160 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x1e);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                let value =
                    VULTURE_GRAPHICS[usize::from((self.game_state.frame.frame_counter >> 1) & 3)];
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_slot_view(k).delay_main() != 0 {
                    self.sprite_slot_view_mut(k).add_z(1);
                    return;
                }
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 1) != 0 {
                    return;
                }
                let pt =
                    self.sprite_project_speed_towards_link(k, ((k & 0x0f) as u8).wrapping_add(24));
                let value = pt.y.wrapping_neg();
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = pt.x;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                if pt.xdiff.wrapping_add(0x28) < 0x50 && pt.ydiff.wrapping_add(0x28) < 0x50 {
                    return;
                }
                let value = self
                    .sprite_slot_view(k)
                    .y_velocity()
                    .wrapping_add(((pt.y as i8) >> 2) as u8);
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = self
                    .sprite_slot_view(k)
                    .x_velocity()
                    .wrapping_add(((pt.x as i8) >> 2) as u8);
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_27_Deadrock(int k) {  // 86948a
    //   Deadrock walking/petrified state machine.
    // }
    pub(super) fn sprite_27_deadrock(&mut self, k: usize) {
        let j = if if self.sprite_slot_view(k).delay_aux2() != 0 {
            (self.sprite_slot_view(k).delay_aux2() & 4) != 0
        } else {
            self.sprite_slot_view(k).ai_state() != 2
        } {
            self.sprite_slot_view(k).a()
        } else {
            8
        };
        let j = usize::from(j);
        let value = DEAD_ROCK_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40) | DEAD_ROCK_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).f() == 0
            && (self.sprite_check_damage_from_link(k) & PLAYER_DAMAGE_CARRY_MASK_DRAW) != 0
            && self.game_state.system_signals.sound_effect_1() == 0
        {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x0b);
        }
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_nullify_hookshot_drag();
            self.sprite_repel_dash();
        }
        if self.sprite_slot_view(k).f() == 14 {
            let value = 2;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 255;
            self.sprite_slot_view_mut(k).set_delay_aux1(value);
            let value = 64;
            self.sprite_slot_view_mut(k).set_delay_aux2(value);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).and_flags2(!0x80);
                    self.sprite_slot_view_mut(k).and_deflection_bits(!4);
                    self.sprite_slot_view_mut(k).and_flags3(!0x40);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    // ROM Sprite_27_Deadrock $06:9524: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(31, 32);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let dir = if self.sprite_slot_view(k).b().wrapping_add(1) == 4 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_b(value);
                        self.sprite_direction_to_face_link(k)
                    } else {
                        self.sprite_slot_view_mut(k).add_b(1);
                        self.get_random_number() & 3
                    };
                    self.deadrock_set_dir(k, dir, &DEAD_ROCK_X_VELOCITIES, &DEAD_ROCK_Y_VELOCITIES);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    self.sprite_move_xy(k);
                    if self.sprite_check_tile_collision(k) != 0 {
                        let dir = self.sprite_slot_view(k).direction() ^ 1;
                        self.deadrock_set_dir(
                            k,
                            dir,
                            &DEAD_ROCK_X_VELOCITIES,
                            &DEAD_ROCK_Y_VELOCITIES,
                        );
                    } else {
                        let value = self.sprite_slot_view(k).subtype2().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                        let value = (self.sprite_slot_view(k).direction() << 1)
                            | ((self.sprite_slot_view(k).subtype2() >> 2) & 1);
                        self.sprite_slot_view_mut(k).set_a(value);
                    }
                }
            }
            2 => {
                self.sprite_slot_view_mut(k).or_flags2(0x80);
                self.sprite_slot_view_mut(k).or_deflection_bits(4);
                self.sprite_slot_view_mut(k).or_flags3(0x40);
                if (self.game_state.frame.frame_counter & 1) == 0 {
                    if self.sprite_slot_view(k).delay_aux1() == 0 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 16;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else if self.sprite_slot_view(k).delay_aux1() == 0x20 {
                        let value = 0x40;
                        self.sprite_slot_view_mut(k).set_delay_aux2(value);
                    }
                } else {
                    let value = self.sprite_slot_view(k).delay_aux1().wrapping_add(1);
                    self.sprite_slot_view_mut(k).set_delay_aux1(value);
                }
            }
            _ => {}
        }
    }

    fn deadrock_set_dir(&mut self, k: usize, dir: u8, xvel: &[u8; 4], yvel: &[u8; 4]) {
        let value = dir;
        self.sprite_slot_view_mut(k).set_direction(value);
        let idx = usize::from(dir);
        let value = xvel[idx];
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = yvel[idx];
        self.sprite_slot_view_mut(k).set_y_velocity(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_20_Sluggula(int k) {  // 8695d9
    pub(super) fn sprite_20_sluggula(&mut self, k: usize) {
        let mut j = usize::from(
            (self.sprite_slot_view(k).direction() << 1)
                | ((self.sprite_slot_view(k).subtype2() & 8) >> 3),
        );
        let value = SPRITE_20_SLUGGULA_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & 191) | SPRITE_20_SLUGGULA_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    // ROM Sprite_20_Sluggula $06:9623: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(31, 32);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    j = usize::from(self.sprite_slot_view(k).delay_main() & 3);
                    let value = j as u8;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = SPRITE_20_SLUGGULA_XYVEL[j] as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = SPRITE_20_SLUGGULA_XYVEL[j + 2] as u8;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                } else if self.sprite_slot_view(k).delay_main() == 16
                    && (self.get_random_number() & 1) == 0
                {
                    self.sluggula_drop_bomb(k);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                self.sprite_move_xy(k);
                if self.sprite_check_tile_collision(k) == 0 {
                    return;
                }
                self.sprite_slot_view_mut(k).xor_direction(1);
                j = usize::from(self.sprite_slot_view(k).direction());
                let value = SPRITE_20_SLUGGULA_XYVEL[j] as u8;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = SPRITE_20_SLUGGULA_XYVEL[j + 2] as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_22_Ropa(int k) {  // 869e1f
    pub(super) fn sprite_22_ropa(&mut self, k: usize) {
        self.ropa_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = (self.sprite_slot_view(k).subtype2() >> 3) & 3;
        self.sprite_slot_view_mut(k).set_graphics(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_apply_speed_towards_link(k, 16);
                    // ROM $86:9E6C: `JSL GetRandomNumber : AND #$0F : ADC #$14`
                    // — the ADC consumes GetRandomNumber's carry-out (the C
                    // port drops it). Route boundary 21 (frame 567894): the
                    // hop left at $1C, the carry-less port at $1B.
                    let value = self.get_random_number_with_carry().masked_adc(15, 20);
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
            }
            1 => {
                self.sprite_move_xy(k);
                if self.sprite_check_tile_collision(k) != 0 {
                    self.sprite_zero_velocity_xy(k);
                }
                self.sprite_move_z(k);
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_59_LostWoodsBird(int k) {  // 85940e
    pub(super) fn sprite_59_lost_woods_bird(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            return;
        }
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40)
            | if sign8(self.sprite_slot_view(k).x_velocity()) {
                0
            } else {
                0x40
            };
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        // ROM $05:9422 JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // ROM $05:9429
        self.sprite_move_xy_marked(k, 0x05fa00);
        self.sprite_move_z(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.sprite_slot_view_mut(k).subtract_z_velocity(1);
                if sign8(self.sprite_slot_view(k).z_velocity().wrapping_sub(0xf1)) {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                self.sprite_slot_view_mut(k).add_z_velocity(2);
                if !sign8(self.sprite_slot_view(k).z_velocity().wrapping_sub(0x10)) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
                self.sprite_slot_view_mut(k).add_subtype2(1);
                let value = (self.sprite_slot_view(k).subtype2() >> 1) & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_5A_LostWoodsSquirrel(int k) {  // 859468
    pub(super) fn sprite_5_a_lost_woods_squirrel(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            return;
        }
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40)
            | if sign8(self.sprite_slot_view(k).x_velocity()) {
                0
            } else {
                0x40
            };
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        // ROM $05:947C JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // ROM $05:9483
        self.sprite_move_xy_marked(k, 0x05fa00);
        self.sprite_move_z(k);
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 16;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            let value = 12;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        let value = u8::from(self.sprite_slot_view(k).delay_main() != 0);
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_58_Crab(int k) {  // 8594b5
    pub(super) fn sprite_58_crab(&mut self, k: usize) {
        self.crab_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        // ROM $05:94C1
        self.sprite_move_xy_marked(k, 0x05fa00);
        // ROM $05:94C4 -> $05:B88D -> $06:E496
        if self.sprite_check_tile_collision_long(k) != 0 || self.sprite_slot_view(k).delay_main() == 0 {
            // ROM Sprite_58_Crab $05:94CE: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
            let value = self.get_random_number_with_carry().masked_adc(63, 32);
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = self.sprite_slot_view(k).delay_main() & 3;
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        let j = self.sprite_slot_view(k).direction();
        let value = SPRITE_58_CRAB_LOCAL_X_VELOCITIES[usize::from(j & 3)] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = SPRITE_58_CRAB_LOCAL_Y_VELOCITIES[usize::from(j & 3)] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = (self.sprite_slot_view(k).subtype2() >> if j < 2 { 1 } else { 3 }) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }
}
