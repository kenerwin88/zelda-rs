//! Interruption-resume lanes of `sprite_main`'s per-slot loop, one method per
//! early-returning block (mechanically extracted; bodies unchanged).

use super::sprite::SpriteSpawnInfo;
use super::sprite_main_mothula::WallmasterMainPrefixOutcome;
use super::*;
use crate::types::sign8;

impl ZeldaState {
    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_initialize_reset_properties(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::InitializeResetProperties {
            slot,
            phase,
            completed_stores,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "sprite-init reset continuation requires a measured NMI phase",
                );
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("sprite-init reset boundary was checked above");
                assert_eq!(
                    self.sprite_slot_view(k).state(),
                    8,
                    "source sprite-init reset boundary requires state 8",
                );
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                if phase == crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion
                {
                    // The source is inside Fire Debirando's second
                    // property load: its first load, state promotion, and
                    // type conversion all precede this reset call.
                    self.sprite_module_initialize_properties(k);
                    self.sprite_slot_view_mut(k).set_sprite_type(0x63);
                }
                self.sprite_prep_reset_properties_prefix(k, completed_stores);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_initialize_load_properties(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::InitializeLoadProperties {
            slot,
            phase,
            completed_stores,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "sprite property-load continuation requires a measured NMI phase",
                );
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("sprite property-load boundary was checked above");
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                match phase {
                    crate::SpriteInitializeResetPropertiesPhase::InitialPropertyLoad => {
                        self.sprite_prep_reset_properties(k);
                    }
                    crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion => {
                        self.sprite_module_initialize_properties(k);
                        self.sprite_slot_view_mut(k).set_sprite_type(0x63);
                        self.sprite_prep_reset_properties(k);
                    }
                }
                self.sprite_prep_load_properties_after_reset_prefix(k, completed_stores);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_mini_moldorm_history(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::MiniMoldormHistory {
            slot,
            completed_stores,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Mini Moldorm history continuation requires a measured NMI phase",
                );
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("Mini Moldorm history boundary was checked above");
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x18);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                self.sprite_prep_mini_moldorm_bounce_prefix(k, completed_stores);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_fire_debirando_before_spawn(&mut self, k: usize) -> bool {
        if self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::FireDebirandoBeforeSpawn(k as u8))
        {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Fire Debirando spawn continuation requires a measured NMI phase",
            );
            let boundary = self
                .sprite_main_cpu_boundary
                .take()
                .expect("Fire Debirando boundary was checked above");
            assert_eq!(
                self.sprite_slot_view(k).state(),
                8,
                "Fire Debirando source boundary requires state 8 at slot entry",
            );
            assert_eq!(
                self.sprite_slot_view(k).sprite_type(),
                0x64,
                "Fire Debirando source boundary requires type $64 at slot entry",
            );
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            self.sprite_module_initialize_properties(k);
            self.sprite_slot_view_mut(k).set_sprite_type(0x63);
            self.sprite_prep_load_properties(k);
            self.sprite_prep_fire_debirando_after_property_reload_before_spawn(k);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_trinexx_final_phase_draw(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::TrinexxFinalPhaseDraw {
            slot,
            segment,
            stage,
            continuation: None,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xcb);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                let continuation =
                    self.begin_trinexx_final_phase_draw_checkpoint(k, segment, stage);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::TrinexxFinalPhaseDraw {
                        slot,
                        segment,
                        stage,
                        continuation: Some(continuation),
                    },
                    nmi_slices,
                    caller,
                );
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_trinexx_final_phase_tile_collision(
        &mut self,
        k: usize,
    ) -> bool {
        if let Some(SpriteMainCpuBoundary::TrinexxFinalPhaseTileCollision {
            slot,
            probes_completed,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xcb);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.begin_trinexx_final_phase_tile_collision_checkpoint(k, probes_completed);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::TrinexxFinalPhaseTileCollision {
                        slot,
                        probes_completed,
                    },
                    nmi_slices,
                    caller,
                );
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_trinexx_death_explosion_spawn(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::TrinexxDeathExplosionSpawn {
            slot,
            spawned_slot,
            progress,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Trinexx death explosion spawn continuation requires a measured NMI phase",
                );
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("Trinexx death explosion spawn boundary was checked above");
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xcb);
                let spawned = usize::from(spawned_slot);
                assert_ne!(spawned, k);
                for candidate in (spawned + 1)..16 {
                    assert_ne!(
                        self.sprite_slot_view(candidate).state(),
                        0,
                        "source dynamic-spawn receipt skipped a higher free slot {candidate}",
                    );
                }
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.begin_trinexx_death_explosion_spawn_checkpoint(k, spawned, progress);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_fire_debirando_spawn(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::FireDebirandoSpawn {
            slot,
            spawned_slot,
            progress,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Fire Debirando dynamic-spawn continuation requires a measured NMI phase",
                );
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("Fire Debirando dynamic-spawn boundary was checked above");
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x64);
                let spawned = usize::from(spawned_slot);
                assert_ne!(spawned, k);
                for candidate in (spawned + 1)..16 {
                    assert_ne!(
                        self.sprite_slot_view(candidate).state(),
                        0,
                        "source dynamic-spawn receipt skipped a higher free slot {candidate}",
                    );
                }
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                self.sprite_slot_view_mut(k).set_sprite_type(0x63);
                self.sprite_prep_load_properties(k);
                self.sprite_prep_fire_debirando_after_property_reload_before_spawn(k);
                let mut info = SpriteSpawnInfo::default();
                self.sprite_spawn_dynamically_selected_prefix(
                    k, 0x64, &mut info, spawned, progress,
                );
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_guard_prep_weapon_flags_pending(&mut self, k: usize) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::GuardPrepWeaponFlagsPending {
                slot,
                continuation: None,
            }) if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "guard-prep weapon continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            assert_eq!(
                self.sprite_slot_view(k).state(),
                8,
                "source guard-prep weapon boundary requires state 8 at slot entry",
            );
            assert_eq!(
                self.sprite_slot_view(k).sprite_type(),
                0x41,
                "source standard-guard checkpoint requires a blue guard",
            );
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            self.sprite_module_initialize_properties(k);
            let continuation = self
                .sprite_prep_standard_guard_until_weapon_flags(k)
                .expect("source guard-prep checkpoint did not reach its nested weapon draw");
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::GuardPrepWeaponFlagsPending {
                    slot: k as u8,
                    continuation: Some(continuation),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_guard_prep_parry_hitbox(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::GuardPrepParryHitbox {
            slot,
            active_call,
            continuation: None,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x41);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                let continuation =
                    self.sprite_prep_standard_guard_until_parry_hitbox(k, active_call);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::GuardPrepParryHitbox {
                        slot,
                        active_call,
                        continuation: Some(continuation),
                    },
                    nmi_slices,
                    caller,
                );
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_guard_prep_patrol_delay(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::GuardPrepPatrolDelay {
            slot,
            active_call,
            saved_submodule: None,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x41);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                let saved_submodule =
                    self.sprite_prep_standard_guard_until_patrol_delay(k, active_call);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::GuardPrepPatrolDelay {
                        slot,
                        active_call,
                        saved_submodule: Some(saved_submodule),
                    },
                    nmi_slices,
                    caller,
                );
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_guard_prep_tile_collision_returned(
        &mut self,
        k: usize,
    ) -> bool {
        if let Some(SpriteMainCpuBoundary::GuardPrepTileCollisionReturned {
            slot,
            active_call,
            saved_submodule: None,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x41);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                let saved_submodule =
                    self.sprite_prep_standard_guard_until_tile_collision_return(k, active_call);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::GuardPrepTileCollisionReturned {
                        slot,
                        active_call,
                        saved_submodule: Some(saved_submodule),
                    },
                    nmi_slices,
                    caller,
                );
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_guard_animation(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::GuardAnimation {
            slot,
            checkpoint,
            continuation: None,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                self.sprite_main_cpu_boundary = None;
                let initializer = matches!(
                    checkpoint,
                    crate::GuardAnimationCheckpoint::HogSpearInitializerBodyReturned { .. }
                );
                assert_eq!(
                    self.sprite_slot_view(k).state(),
                    if initializer { 8 } else { 9 }
                );
                if initializer {
                    assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x45);
                } else {
                    assert!((0x41..=0x44).contains(&self.sprite_slot_view(k).sprite_type()));
                }
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                if initializer {
                    self.sprite_module_initialize_properties(k);
                }
                let continuation = self.guard_animation_until_checkpoint(k, checkpoint);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::GuardAnimation {
                        slot,
                        checkpoint,
                        continuation: Some(continuation),
                    },
                    nmi_slices,
                    caller,
                );
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_timers_and_oam(&mut self, k: usize) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterTimersAndOam { slot, state: None })
                if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "timer/OAM continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            let state = self.sprite_slot_view(k).state();
            assert_ne!(
                state, 0,
                "source timer/OAM return requires an active sprite slot",
            );
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, false);
            self.sprite_timers_and_oam(k);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterTimersAndOam {
                    slot: k as u8,
                    state: Some(state),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_happiness_pond_rupee_graphics_started(
        &mut self,
        k: usize,
    ) -> bool {
        if self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(
                k as u8,
            ))
        {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Happiness Pond rupee graphics continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            assert_eq!(self.sprite_slot_view(k).state(), 9);
            assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x72);
            assert_eq!(self.sprite_slot_view(k).ai_state(), 3);
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            assert!(
                self.sprite_happiness_pond_before_rupee_graphics(k),
                "source Happiness Pond rupee boundary requires the live purchase-completion path",
            );
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(k as u8),
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_antfairy_subtype2_increment(
        &mut self,
        k: usize,
    ) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                slot,
                continuation: None,
            }) if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Antfairy subtype continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            assert_eq!(
                self.sprite_slot_view(k).state(),
                9,
                "source Antfairy subtype boundary requires an active sprite",
            );
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            let continuation = self.antfairy_draw_continuation(k);
            self.sprite_slot_view_mut(k).add_subtype2(1);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                    slot: k as u8,
                    continuation: Some(continuation),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_lanmola_subtype2_increment(&mut self, k: usize) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                slot,
                continuation: None,
            }) if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Lanmola subtype continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            assert_eq!(
                self.sprite_slot_view(k).state(),
                9,
                "source Lanmola subtype boundary requires an active sprite",
            );
            assert_eq!(
                self.sprite_slot_view(k).sprite_type(),
                0x54,
                "source Lanmola subtype boundary requires a Lanmola",
            );
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            let continuation = self.lanmola_prep_and_draw_through_subtype2_increment(k);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                    slot: k as u8,
                    continuation: Some(continuation),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_helmasaur_hard_hat_beetle_subtype2_increment(
        &mut self,
        k: usize,
    ) -> bool {
        if self.sprite_main_cpu_boundary
            == Some(
                SpriteMainCpuBoundary::AfterHelmasaurHardHatBeetleSubtype2Increment {
                    slot: k as u8,
                },
            )
        {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Helmasaur/Hardhat subtype continuation requires a measured NMI phase",
            );
            let boundary = self
                .sprite_main_cpu_boundary
                .take()
                .expect("Helmasaur/Hardhat subtype boundary was checked above");
            assert_eq!(
                self.sprite_slot_view(k).state(),
                9,
                "source Helmasaur/Hardhat subtype boundary requires an active sprite",
            );
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            let reached_increment = match self.sprite_slot_view(k).sprite_type() {
                0x13 => self.sprite_13_mini_helmasaur_through_subtype2_increment(k),
                0x26 => self.sprite_26_hardhat_beetle_through_subtype2_increment(k),
                sprite_type => panic!(
                    "source Helmasaur/Hardhat subtype boundary used sprite type ${sprite_type:02x}"
                ),
            };
            assert!(
                reached_increment,
                "source Helmasaur/Hardhat subtype receipt did not reach its increment",
            );
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_mini_moldorm_ai_pending(&mut self, k: usize) -> bool {
        if matches!(self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::MiniMoldormAiPending { slot } | SpriteMainCpuBoundary::MoblinAttributeLoaded { slot } | SpriteMainCpuBoundary::MoblinCollisionGeometry { slot } | SpriteMainCpuBoundary::VitreousDamagePending { slot } | SpriteMainCpuBoundary::VitreousAiPending { slot } | SpriteMainCpuBoundary::VitreousPlayerDamagePending { slot }) if slot == k as u8)
        {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(nmi_slices, 0);
            let boundary = self.sprite_main_cpu_boundary.unwrap();
            assert_eq!(self.sprite_slot_view(k).state(), 9);
            let expected_type =
                if matches!(boundary, SpriteMainCpuBoundary::MiniMoldormAiPending { .. }) {
                    0x18
                } else if matches!(
                    boundary,
                    SpriteMainCpuBoundary::MoblinCollisionGeometry { .. }
                        | SpriteMainCpuBoundary::MoblinAttributeLoaded { .. }
                ) {
                    0x12
                } else {
                    0xbd
                };
            assert_eq!(self.sprite_slot_view(k).sprite_type(), expected_type);
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            self.sprite_active_main(k);
            self.sprite_main_cpu_boundary = None;
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_trinexx_head_draw(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::TrinexxHeadDraw {
            slot,
            segment,
            continuation: None,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_main_cpu_boundary = None;
                let draw = self.begin_sidenexx_head_draw_checkpoint(k, segment);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::TrinexxHeadDraw {
                        slot,
                        segment,
                        continuation: Some(draw),
                    },
                    nmi_slices,
                    caller,
                );
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_sidenexx_neck_target_loop(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::SidenexxNeckTargetLoop {
            slot,
            step,
            continuation: None,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_main_cpu_boundary = None;
                let continuation = self.begin_sidenexx_neck_target_checkpoint(k, step);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::SidenexxNeckTargetLoop {
                        slot,
                        step,
                        continuation: Some(continuation),
                    },
                    nmi_slices,
                    caller,
                );
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_trinexx_head_front_part(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::TrinexxHeadFrontPart {
            slot,
            completed_stores,
            continuation: None,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(nmi_slices, 0);
                // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                // directly; open its scope for the timers and the handler part.
                let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                self.sprite_timers_and_oam(k);
                self.sprite_main_cpu_boundary = None;
                let draw = self.begin_sidenexx_front_part_checkpoint(k, completed_stores);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::TrinexxHeadFrontPart {
                        slot,
                        completed_stores,
                        continuation: Some(draw),
                    },
                    nmi_slices,
                    caller,
                );
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_bari_before_random(&mut self, k: usize) -> bool {
        if self.sprite_main_cpu_boundary == Some(SpriteMainCpuBoundary::BariBeforeRandom(k as u8)) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Bari pre-RNG continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            assert_eq!(self.sprite_slot_view(k).state(), 8);
            assert!(matches!(
                self.sprite_slot_view(k).sprite_type(),
                0x23 | 0x24
            ));
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            self.sprite_module_initialize_properties(k);
            self.sprite_prep_bari_before_random(k);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::BariBeforeRandom(k as u8),
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_main_and_aux1_timer_decrements(
        &mut self,
        k: usize,
    ) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements {
                slot,
                state: None,
            }) if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "main/aux1 timer decrement continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            let state = self.sprite_slot_view(k).state();
            assert_ne!(
                state, 0,
                "source main/aux1 timer decrement boundary requires an active sprite slot",
            );
            // Cycle ledger: this lane suspends inside Sprite_TimersAndOam under
            // Sprite_ExecuteSingle (its dispatch has not run yet); open both scopes.
            let _execute_single = self.sprite_execute_single_lane_scope(k, false);
            let _timers = self.sprite_timers_and_oam_lane_scope();
            self.sprite_timers_and_oam_through_main_and_aux1_timer_decrements(k);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements {
                    slot: k as u8,
                    state: Some(state),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_main_timer_decrement(&mut self, k: usize) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterMainTimerDecrement {
                slot,
                state: None,
            }) if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "main timer decrement continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            let state = self.sprite_slot_view(k).state();
            assert_ne!(
                state, 0,
                "source main timer decrement boundary requires an active sprite slot",
            );
            // Cycle ledger: this lane suspends inside Sprite_TimersAndOam under
            // Sprite_ExecuteSingle (its dispatch has not run yet); open both scopes.
            let _execute_single = self.sprite_execute_single_lane_scope(k, false);
            let _timers = self.sprite_timers_and_oam_lane_scope();
            self.sprite_timers_and_oam_through_main_timer_decrement(k);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterMainTimerDecrement {
                    slot: k as u8,
                    state: Some(state),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_zero_hit_timer_clear(&mut self, k: usize) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterZeroHitTimerClear {
                slot,
                state: None,
            }) if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "main timer decrement continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            let state = self.sprite_slot_view(k).state();
            assert_ne!(
                state, 0,
                "source main timer decrement boundary requires an active sprite slot",
            );
            // Cycle ledger: this lane suspends inside Sprite_TimersAndOam under
            // Sprite_ExecuteSingle (its dispatch has not run yet); open both scopes.
            let _execute_single = self.sprite_execute_single_lane_scope(k, false);
            let _timers = self.sprite_timers_and_oam_lane_scope();
            self.sprite_timers_and_oam_through_zero_hit_timer_clear(k);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterZeroHitTimerClear {
                    slot: k as u8,
                    state: Some(state),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_primary_timer_decrements(&mut self, k: usize) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterPrimaryTimerDecrements { slot, state: None })
                if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "primary timer decrement continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            let state = self.sprite_slot_view(k).state();
            assert_ne!(
                state, 0,
                "source primary timer decrement boundary requires an active sprite slot",
            );
            // Cycle ledger: this lane suspends inside Sprite_TimersAndOam under
            // Sprite_ExecuteSingle (its dispatch has not run yet); open both scopes.
            let _execute_single = self.sprite_execute_single_lane_scope(k, false);
            let _timers = self.sprite_timers_and_oam_lane_scope();
            self.sprite_timers_and_oam_through_primary_timer_decrements(k);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterPrimaryTimerDecrements {
                    slot: k as u8,
                    state: Some(state),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_hit_timer(&mut self, k: usize) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterHitTimer { slot, state: None })
                if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "hit timer decrement continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            let state = self.sprite_slot_view(k).state();
            assert_ne!(
                state, 0,
                "source hit timer decrement boundary requires an active sprite slot",
            );
            // Cycle ledger: this lane suspends inside Sprite_TimersAndOam under
            // Sprite_ExecuteSingle (its dispatch has not run yet); open both scopes.
            let _execute_single = self.sprite_execute_single_lane_scope(k, false);
            let _timers = self.sprite_timers_and_oam_lane_scope();
            self.sprite_timers_and_oam_through_primary_timer_decrements(k);
            self.sprite_timers_and_oam_after_primary_through_hit_timer(k);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterHitTimer {
                    slot: k as u8,
                    state: Some(state),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_timer_decrements(&mut self, k: usize) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterTimerDecrements { slot, state: None })
                if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "timer decrement continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            let state = self.sprite_slot_view(k).state();
            assert_ne!(
                state, 0,
                "source timer decrement boundary requires an active sprite slot",
            );
            // Cycle ledger: this lane suspends inside Sprite_TimersAndOam under
            // Sprite_ExecuteSingle (its dispatch has not run yet); open both scopes.
            let _execute_single = self.sprite_execute_single_lane_scope(k, false);
            let _timers = self.sprite_timers_and_oam_lane_scope();
            self.sprite_timers_and_oam_through_timer_decrements(k);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterTimerDecrements {
                    slot: k as u8,
                    state: Some(state),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_wallmaster_reset_prefix(&mut self, k: usize) -> bool {
        if matches!(self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterWallmasterResetPrefix(slot)
                | SpriteMainCpuBoundary::WallmasterResetClear { slot, .. }) if usize::from(slot) == k)
        {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Wallmaster reset continuation requires a measured NMI phase",
            );
            let boundary = self
                .sprite_main_cpu_boundary
                .take()
                .expect("Wallmaster reset boundary was checked above");
            assert_eq!(self.sprite_slot_view(k).state(), 9);
            assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x90);
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            assert_eq!(
                self.sprite_90_wallmaster_through_send_decision(k),
                WallmasterMainPrefixOutcome::SendPlayer,
                "source Wallmaster reset boundary requires the send-player branch: state={:#04x} sub={:#04x} modal={:#04x} pause={:#04x} deflection={:#04x}",
                self.sprite_slot_view(k).state(),
                self.game_state.frame.submodule,
                self.game_state.frame.modal_pause_flag,
                self.sprite_slot_view(k).pause(),
                self.sprite_slot_view(k).deflection_bits(),
            );
            self.wall_master_send_player_through_reset_fixed_prefix();
            if let SpriteMainCpuBoundary::WallmasterResetClear { cleared_bytes, .. } = boundary {
                assert!(cleared_bytes <= 0x1000);
                self.sprite_workspace_mut()
                    .clear_where_in_room_range(0x1000 - usize::from(cleared_bytes)..0x1000);
            }
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_throwable_scenery_state_clear(
        &mut self,
        k: usize,
    ) -> bool {
        if self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::AfterThrowableSceneryStateClear(
                k as u8,
            ))
        {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "throwable-scenery continuation requires a measured NMI phase",
            );
            let boundary = self
                .sprite_main_cpu_boundary
                .take()
                .expect("throwable-scenery boundary was checked above");
            assert_eq!(self.sprite_slot_view(k).state(), 6);
            assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xec);
            assert!(
                sign8(self.sprite_slot_view(k).c()) || self.sprite_slot_view(k).c() < 6,
                "throwable-scenery state-clear boundary requires the small-debris branch",
            );
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            self.sprite_slot_view_mut(k).set_state(0);
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_before_zelda_follower_graphics(&mut self, k: usize) -> bool {
        if self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::BeforeZeldaFollowerGraphics(k as u8))
        {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Zelda follower-graphics continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            assert_eq!(self.sprite_slot_view(k).state(), 8);
            assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x76);
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            self.sprite_module_initialize_properties(k);
            let saved_follower_indicator = self
                .sprite_prep_zelda_before_follower_graphics(k)
                .expect("ROM follower-graphics boundary requires Zelda's live prep path");
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterZeldaFollowerGraphics {
                    slot: k as u8,
                    saved_follower_indicator,
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_zazak_after_graphics(&mut self, k: usize) -> bool {
        if self.sprite_main_cpu_boundary == Some(SpriteMainCpuBoundary::ZazakAfterGraphics(k as u8))
        {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Zazak graphics continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            assert_eq!(self.sprite_slot_view(k).state(), 9);
            assert!(matches!(
                self.sprite_slot_view(k).sprite_type(),
                0xa5..=0xa7
            ));
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            assert!(
                self.sprite_zazak_before_graphics_boundary(k),
                "source Zazak graphics boundary requires the ordinary live body",
            );
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::ZazakAfterGraphics(k as u8),
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_follower_graphics(&mut self, k: usize) -> bool {
        if let Some(SpriteMainCpuBoundary::FollowerGraphics {
            slot,
            caller: follower_graphics_caller,
            prefix_completed: false,
            saved_follower_indicator: None,
            stage,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "partial sprite follower-graphics continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                let saved_follower_indicator = match follower_graphics_caller {
                    crate::SpriteFollowerGraphicsCaller::BlindMaiden => {
                        assert_eq!(self.sprite_slot_view(k).state(), 8);
                        assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xb7);
                        // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                        // directly; open its scope for the timers and the handler part.
                        let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                        self.sprite_timers_and_oam(k);
                        self.sprite_module_initialize_properties(k);
                        assert!(
                            self.sprite_prep_blind_maiden_before_follower_graphics(k),
                            "source follower-graphics progress requires Blind Maiden's live prep path",
                        );
                        None
                    }
                    crate::SpriteFollowerGraphicsCaller::Zelda => {
                        assert_eq!(self.sprite_slot_view(k).state(), 8);
                        assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x76);
                        // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                        // directly; open its scope for the timers and the handler part.
                        let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                        self.sprite_timers_and_oam(k);
                        self.sprite_module_initialize_properties(k);
                        Some(self.sprite_prep_zelda_before_follower_graphics(k).expect(
                            "source follower-graphics progress requires Zelda's live prep path",
                        ))
                    }
                    crate::SpriteFollowerGraphicsCaller::BlindMaidenBody => {
                        assert_eq!(self.sprite_slot_view(k).state(), 9);
                        assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xb7);
                        // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                        // directly; open its scope for the timers and the handler part.
                        let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                        self.sprite_timers_and_oam(k);
                        assert!(
                            self.sprite_b7_blind_maiden_before_follower_graphics(k),
                            "source follower-graphics progress requires Blind Maiden's live become-follower path",
                        );
                        None
                    }
                    crate::SpriteFollowerGraphicsCaller::OldMan => {
                        assert_eq!(self.sprite_slot_view(k).state(), 8);
                        assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xad);
                        // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                        // directly; open its scope for the timers and the handler part.
                        let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                        self.sprite_timers_and_oam(k);
                        self.sprite_module_initialize_properties(k);
                        let reset_follower_after_graphics = self
                            .sprite_prep_old_man_before_follower_graphics(k)
                            .expect(
                                "source follower-graphics progress requires Old Man's live prep path",
                            );
                        Some(u8::from(reset_follower_after_graphics))
                    }
                    crate::SpriteFollowerGraphicsCaller::PurpleChest => {
                        assert_eq!(self.sprite_slot_view(k).state(), 9);
                        assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xb4);
                        // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                        // directly; open its scope for the timers and the handler part.
                        let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                        self.sprite_timers_and_oam(k);
                        assert!(self.sprite_b4_purple_chest_before_follower_graphics(k),
                            "source follower-graphics progress requires the chest's follower transition");
                        None
                    }
                    crate::SpriteFollowerGraphicsCaller::SuperBomb => {
                        assert_eq!(self.sprite_slot_view(k).state(), 9);
                        assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xb5);
                        assert_eq!(self.sprite_slot_view(k).subtype2(), 2);
                        // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
                        // directly; open its scope for the timers and the handler part.
                        let _execute_single = self.sprite_execute_single_lane_scope(k, true);
                        self.sprite_timers_and_oam(k);
                        assert!(
                            self.sprite_bomb_shop_super_bomb_before_follower_graphics(k),
                            "source follower graphics requires a successful super-bomb purchase"
                        );
                        None
                    }
                };
                self.apply_follower_graphics_progress(None, stage);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::FollowerGraphics {
                        slot,
                        caller: follower_graphics_caller,
                        prefix_completed: true,
                        saved_follower_indicator,
                        stage,
                    },
                    nmi_slices,
                    caller,
                );
                return true;
            }
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_king_zora_flippers_graphics_started(
        &mut self,
        k: usize,
    ) -> bool {
        if self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::KingZoraFlippersGraphicsStarted(
                k as u8,
            ))
        {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "King Zora flippers graphics continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            assert_eq!(self.sprite_slot_view(k).state(), 9);
            assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x52);
            assert_eq!(self.sprite_slot_view(k).ai_state(), 3);
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            assert!(
                self.sprite_52_king_zora_before_flippers_graphics(k),
                "source King Zora flippers boundary requires the live purchase-completion path",
            );
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            assert!(matches!(caller, SpriteMainCpuCaller::Module09 { .. }));
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::KingZoraFlippersGraphicsStarted(k as u8),
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_after_single_small_draw_position(&mut self, k: usize) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                slot,
                continuation: None,
            }) if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "single-small draw continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            assert_eq!(self.sprite_slot_view(k).state(), 9);
            assert!(matches!(
                self.sprite_slot_view(k).sprite_type(),
                0x23 | 0x24
            ));
            assert!(
                self.sprite_slot_view(k).c() != 0 && !sign8(self.sprite_slot_view(k).c()),
                "source single-small draw boundary requires Red Bari's positive-C draw path",
            );
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            let continuation = self
                .sprite_draw_single_small_position_prefix(k)
                .expect("source single-small draw boundary requires visible OAM preparation");
            assert!(
                continuation.visible,
                "source single-small draw position boundary requires the visible Y store",
            );
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                    slot: k as u8,
                    continuation: Some(continuation),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_resume_probe_after_oam_coordinates(&mut self, k: usize) -> bool {
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
                slot,
                oam_position: None,
            }) if slot == k as u8
        ) {
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "guard-probe continuation requires a measured NMI phase",
            );
            self.sprite_main_cpu_boundary = None;
            assert_eq!(self.sprite_slot_view(k).state(), 9);
            assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x41);
            assert_ne!(
                self.sprite_slot_view(k).c(),
                0,
                "source guard-probe boundary requires Probe rather than Guard_Main",
            );
            // Cycle ledger: this lane enters the slot's Sprite_ExecuteSingle body
            // directly; open its scope for the timers and the handler part.
            let _execute_single = self.sprite_execute_single_lane_scope(k, true);
            self.sprite_timers_and_oam(k);
            let oam_position = self
                .probe_until_after_oam_coordinates(k)
                .expect("source guard-probe boundary did not reach its OAM-coordinate return");
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(
                SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
                    slot: k as u8,
                    oam_position: Some(oam_position),
                },
                nmi_slices,
                caller,
            );
            return true;
        }
        false
    }

    /// Extracted from `sprite_main` (mechanical move; body unchanged).
    /// Returns `true` when the caller must return.
    pub(super) fn sprite_main_lane_at_4374(
        &mut self,
        enters_item_receipt_graphics: bool,
        k: usize,
    ) -> bool {
        if enters_item_receipt_graphics {
            assert!(
                matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishItemReceiptGraphics {
                        continuation:
                            ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt {
                                sprite_slot,
                                ..
                            }
                            | ItemReceiptGraphicsContinuation::ResumeUnclePassage {
                                sprite_slot,
                                ..
                            },
                    }) if sprite_slot == k as u8
                ),
                "source item-receipt boundary did not enter the native graphics continuation: host={} slot={} sprite_type={:#04x} sprite_state={:#04x} item_receipt_method={} work={:?}",
                self.frame_ctr_dbg,
                k,
                self.sprite_slot_view(k).sprite_type(),
                self.sprite_slot_view(k).state(),
                self.game_state.player.follower_link.item_receipt_method(),
                self.game_execution_scheduler.current_work(),
            );
            return true;
        }
        false
    }
}
