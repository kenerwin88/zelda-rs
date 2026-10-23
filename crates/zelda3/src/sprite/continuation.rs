//! Sprite main CPU-boundary continuations and caller completion.

use super::*;

impl ZeldaState {

    pub(in crate::zelda_rtl) fn complete_sprite_main_after_cpu_boundary(
        &mut self,
        boundary: SpriteMainCpuBoundary,
    ) {
        match boundary {
            SpriteMainCpuBoundary::CuccoDraw { mut work } => {
                let slot = work.slot();
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.advance_cucco_draw(&mut work, None), CpuWorkAdvance::Complete);
                self.complete_cucco_draw(work);
                if !self.game_execution_scheduler.work_suspends_translated_call_stack() {
                    self.complete_sprite_main_after_interrupted_slot(usize::from(slot));
                }
            }
            SpriteMainCpuBoundary::FollowerWrapperReturn { plb_completed } => {
                crate::cycle_ledger::charge_routine(0x09_9f91, if plb_completed { 44 } else { 72 });
                self.complete_sprite_main_prefix_after_follower();
                if !self.game_execution_scheduler.work_suspends_translated_call_stack() {
                    self.complete_sprite_main_after_interrupted_slot(16);
                }
            }
            SpriteMainCpuBoundary::FollowerAnimationSelection { inputs: Some((ain, xin, yin)) } => {
                {
                    let _scope = crate::cycle_ledger::routine(0x09_9fc4);
                    self.follower_animate_movement_preserved(ain, xin, yin);
                }
                crate::cycle_ledger::charge_routine(0x09_9f91, 72); // PLB RTL.
                self.complete_sprite_main_prefix_after_follower();
                if !self.game_execution_scheduler.work_suspends_translated_call_stack() {
                    self.complete_sprite_main_after_interrupted_slot(16);
                }
            }
            SpriteMainCpuBoundary::FollowerAnimationSelection { inputs: None } => {
                unreachable!("follower selector must bind its native arguments before suspension")
            }
            follower @ (SpriteMainCpuBoundary::FollowerBodyDraw { continuation: Some(work), .. }
            | SpriteMainCpuBoundary::FollowerHeadCoordinates { continuation: Some(work), .. }) => {
                {
                    let _scope = crate::cycle_ledger::routine(0x09_9fc4);
                    if let SpriteMainCpuBoundary::FollowerHeadCoordinates { prefix, .. } = follower {
                        self.complete_follower_body_draw_after_coordinate_prefix(work, prefix);
                    } else {
                        self.complete_follower_body_draw(work);
                    }
                }
                crate::cycle_ledger::charge_routine(0x09_9f91, 72); // PLB RTL.
                self.complete_sprite_main_prefix_after_follower();
                if !self.game_execution_scheduler.work_suspends_translated_call_stack() {
                    self.complete_sprite_main_after_interrupted_slot(16);
                }
            }
            SpriteMainCpuBoundary::FollowerBodyDraw { continuation: None, .. }
            | SpriteMainCpuBoundary::FollowerHeadCoordinates { continuation: None, .. } => {
                unreachable!("follower draw receipt must bind native locals before suspension")
            }
            SpriteMainCpuBoundary::Module09FinalScrollPairPending => {
                self.complete_module09_final_scroll_pair();
                self.sprite_main();
            }
            SpriteMainCpuBoundary::BeforeFirstSlot => {
                if let Some((main_module, submodule)) = self.pending_module09_frame_advance.take() {
                    // The suspended Module09 handler's deferred advance lands
                    // right before Sprite_Main, as the ROM writes it.
                    if self.pending_overworld_sprite_reload_slots.is_some() {
                        self.publish_deferred_module09_sprite_slots_at_reload_return();
                    }
                    self.set_submodule(submodule);
                    self.set_main_module(main_module);
                }
                self.sprite_main()
            }
            SpriteMainCpuBoundary::AfterSlot(interrupted_slot) => {
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot as usize)
            }
            SpriteMainCpuBoundary::TimerCoordinateMoves { slot, state, next_instruction, coordinates } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.advance_sprite_coordinate_moves(coordinates, next_instruction, 8);
                crate::cycle_ledger::charge_routine(0x06_84c1, 42); // RTS.
                {
                    let _timers = crate::cycle_ledger::routine(0x06_83f2);
                    self.sprite_timers_and_oam_after_coordinates_through_oam_allocation(k);
                    self.sprite_timers_and_oam_after_oam_allocation(k);
                }
                {
                    let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                    crate::cycle_ledger::charge(28 + 16 + 16); // PLA, CMP, BEQ.
                    self.sprite_execute_single_after_timers(k, state);
                }
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::AfterTimersAndOam {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                {
                    // Cycle ledger: the ROM resumes inside Sprite_ExecuteSingle
                    // $06:84E2 after its timer call returned.
                    let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                    self.sprite_execute_single_after_timers(interrupted_slot, state);
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterTimersAndOam { state: None, .. } => unreachable!(
                "source timer/OAM boundary did not bind to the saved native dispatch state"
            ),
            SpriteMainCpuBoundary::InitializeResetProperties {
                slot,
                phase,
                completed_stores,
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                let expected_state = match phase {
                    crate::SpriteInitializeResetPropertiesPhase::InitialPropertyLoad => 8,
                    crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion => 9,
                };
                assert_eq!(
                    self.sprite_slot_view(interrupted_slot).state(),
                    expected_state,
                    "resumed sprite-init reset boundary lost its source state",
                );
                self.sprite_prep_reset_properties_from(interrupted_slot, completed_stores);
                match phase {
                    crate::SpriteInitializeResetPropertiesPhase::InitialPropertyLoad => {
                        self.sprite_module_initialize_properties_after_reset(interrupted_slot);
                        self.sprite_module_initialize_after_properties(interrupted_slot);
                    }
                    crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion => {
                        self.sprite_prep_load_properties_after_reset(interrupted_slot);
                        self.sprite_prep_fire_debirando_after_property_reload(interrupted_slot);
                    }
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::InitializeLoadProperties {
                slot,
                phase,
                completed_stores,
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                let expected_state = match phase {
                    crate::SpriteInitializeResetPropertiesPhase::InitialPropertyLoad => 8,
                    crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion => 9,
                };
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), expected_state);
                self.sprite_prep_load_properties_after_reset_from(
                    interrupted_slot,
                    completed_stores,
                );
                match phase {
                    crate::SpriteInitializeResetPropertiesPhase::InitialPropertyLoad => {
                        self.sprite_slot_view_mut(interrupted_slot).increment_state();
                        self.sprite_module_initialize_after_properties(interrupted_slot);
                    }
                    crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion => {
                        self.sprite_prep_fire_debirando_after_property_reload(interrupted_slot);
                    }
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::MiniMoldormHistory {
                slot,
                completed_stores,
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert_eq!(self.sprite_slot_view(interrupted_slot).sprite_type(), 0x18);
                self.sprite_prep_mini_moldorm_bounce_from(
                    interrupted_slot,
                    completed_stores,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::FireDebirandoBeforeSpawn(slot) => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert_eq!(self.sprite_slot_view(interrupted_slot).sprite_type(), 0x63);
                self.sprite_prep_debirando_pit_after_before_spawn(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
                                                SpriteMainCpuBoundary::TrinexxFinalPhaseDraw {
                slot,
                segment,
                stage,
                continuation,
            } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xcb);
                let continuation = continuation
                    .expect("Trinexx final-phase draw continuation lost its source locals");
                self.resume_trinexx_final_phase_draw(k, segment, stage, continuation);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
                        SpriteMainCpuBoundary::LanmolaDrawPrefix {
                slot,
                completed_stores,
            } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x54);
                self.resume_lanmola_draw_prefix(k, completed_stores);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
SpriteMainCpuBoundary::HelmasaurHardHatTileCollision { slot, stage } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert!(matches!(self.sprite_slot_view(k).sprite_type(), 0x13 | 0x26));
                self.resume_helmasaur_hard_hat_tile_collision(k, stage);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
SpriteMainCpuBoundary::TrinexxFinalPhaseTileCollision {
                slot,
                probes_completed,
            } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xcb);
                self.resume_trinexx_final_phase_tile_collision(k, probes_completed);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
SpriteMainCpuBoundary::TrinexxDeathExplosionSpawn {
                slot,
                spawned_slot,
                progress,
            } => {
                let interrupted_slot = usize::from(slot);
                let spawned = usize::from(spawned_slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert_eq!(self.sprite_slot_view(interrupted_slot).sprite_type(), 0xcb);
                assert_eq!(self.sprite_slot_view(spawned).sprite_type(), 0);
                self.resume_trinexx_death_explosion_spawn(interrupted_slot, spawned, progress);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AgahnimMotionBlurSpawn {
                slot,
                spawned_slot,
                progress,
                bound,
            } => {
                let interrupted_slot = usize::from(slot);
                let spawned = usize::from(spawned_slot);
                assert!(
                    bound,
                    "source Agahnim motion-blur spawn boundary did not bind on the interrupted host"
                );
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert_eq!(self.sprite_slot_view(spawned).sprite_type(), 0xc1);
                self.resume_agahnim_motion_blur_spawn(interrupted_slot, spawned, progress);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::FireDebirandoSpawn {
                slot,
                spawned_slot,
                progress,
            } => {
                let interrupted_slot = usize::from(slot);
                let spawned = usize::from(spawned_slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert_eq!(self.sprite_slot_view(interrupted_slot).sprite_type(), 0x63);
                assert_eq!(self.sprite_slot_view(spawned).sprite_type(), 0x64);
                let mut info = SpriteSpawnInfo::default();
                self.sprite_spawn_dynamically_selected_from(
                    interrupted_slot,
                    &mut info,
                    spawned,
                    progress,
                );
                self.sprite_prep_debirando_pit_after_spawn(interrupted_slot, spawned, &info);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::MasterSwordLightBeamSpawn {
                slot,
                spawned_slot,
                progress,
            } => {
                let interrupted_slot = usize::from(slot);
                let spawned = usize::from(spawned_slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert_eq!(self.sprite_slot_view(interrupted_slot).sprite_type(), 0x62);
                assert_eq!(self.sprite_slot_view(interrupted_slot).subtype2(), 2);
                assert_ne!(self.sprite_slot_view(interrupted_slot).a(), 0);
                assert_eq!(self.sprite_slot_view(spawned).sprite_type(), 0x62);
                let mut info = SpriteSpawnInfo::default();
                self.sprite_spawn_dynamically_selected_from(
                    interrupted_slot,
                    &mut info,
                    spawned,
                    progress,
                );
                self.master_sword_finish_replacement_light_beam_spawn(
                    interrupted_slot,
                    spawned,
                    info.r0_x,
                    info.r2_y,
                );
                let new_b = self.sprite_slot_view(interrupted_slot).b().wrapping_sub(1);
                self.sprite_slot_view_mut(interrupted_slot).set_b(new_b);
                if new_b == 0 {
                    self.sprite_slot_view_mut(interrupted_slot).set_state(0);
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                slot,
                continuation: Some(continuation),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_antfairy_after_subtype2_increment(
                    interrupted_slot,
                    continuation,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                continuation: None,
                ..
            } => unreachable!(
                "source Antfairy subtype boundary did not bind to a native draw caller"
            ),
            SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                slot,
                continuation: Some(continuation),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_lanmola_after_subtype2_increment(interrupted_slot, continuation);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                continuation: None,
                ..
            } => unreachable!(
                "source Lanmola subtype boundary did not bind to a native draw caller"
            ),
            SpriteMainCpuBoundary::AfterHelmasaurHardHatBeetleSubtype2Increment { slot } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert!(matches!(
                    self.sprite_slot_view(interrupted_slot).sprite_type(),
                    0x13 | 0x26
                ));
                self.helmasaur_hard_hat_beetle_common_after_subtype2_increment(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::InitializePrepPending { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                self.sprite_module_initialize_after_properties(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::InitializePrepMoveY {
                slot,
                checkpoint,
                continuation: Some(continuation),
            } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                self.sprite_prep_move_y_from_checkpoint(k, checkpoint, continuation);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::InitializePrepMoveY {
                continuation: None,
                ..
            } => unreachable!(
                "source state-8 prep Sprite_MoveY boundary did not bind to native coordinate results"
            ),
            SpriteMainCpuBoundary::HogSpearBodyGraphicsPending { slot } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert_eq!(self.sprite_slot_view(interrupted_slot).sprite_type(), 0x45);
                self.guard_update_body_graphics(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AbsorbableHorizontalTileLookup { slot } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                self.sprite_absorbable_after_horizontal_lookup(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AbsorbableVerticalTileLookup { slot } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                self.sprite_absorbable_after_vertical_lookup(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::VitreousPlayerDamagePending { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.vitreous_after_player_damage_checkpoint(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::VitreousAiPending { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.vitreous_ai_after_damage(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }            SpriteMainCpuBoundary::MiniMoldormAiPending { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.mini_moldorm_ai_after_collision(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::VitreousDamagePending { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xbd);
                self.vitreous_after_damage_checkpoint(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::MoblinCollisionGeometry { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.sprite_check_tile_collision_single_layer(k);
                self.moblin_after_tile_collision(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::MoblinAttributeLoaded { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.moblin_after_loaded_vertical_attribute(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::SwamolaHeadDrawCompleted { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.swamola_draw_after_completed_head(k);
                self.sprite_cf_swamola_after_draw(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::SwamolaHeadDraw { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.swamola_draw_after_head_checkpoint(k);
                self.sprite_cf_swamola_after_draw(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::SwamolaSegmentDraw { slot, segment } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.swamola_draw_after_segment_checkpoint(k, segment);
                self.sprite_cf_swamola_after_draw(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::TrinexxHeadDraw { slot, continuation: Some(draw), .. }
            | SpriteMainCpuBoundary::TrinexxHeadFrontPart { slot, continuation: Some(draw), .. } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.resume_sidenexx_head_draw(k, draw);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::TrinexxHeadDraw { continuation: None, .. }
            | SpriteMainCpuBoundary::TrinexxHeadFrontPart { continuation: None, .. } => unreachable!("Trinexx draw continuation lost its source locals"),
            SpriteMainCpuBoundary::SidenexxNeckTargetLoop { slot, step, continuation: Some(continuation) } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.resume_sidenexx_neck_target(k, step, continuation);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::SidenexxNeckTargetLoop { continuation: None, .. } => unreachable!("Sidenexx neck-target continuation lost its source locals"),
            SpriteMainCpuBoundary::AbsorbableVerticalTileAttributeLoaded { slot } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                self.sprite_absorbable_after_vertical_attribute_loaded(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::PengatorSlidePending { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x99);
                assert_eq!(self.sprite_slot_view(k).ai_state(), 3);
                self.pengator_after_movement(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::AntifairyBouncePending { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x15);
                self.sprite_bounce_from_tile_collision(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::KholdstareDamagePending { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xa2);
                self.kholdstare_after_movement(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::AfterTimerDecrements {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                {
                    // Cycle ledger: the ROM resumes inside Sprite_TimersAndOam
                    // under Sprite_ExecuteSingle; both scopes reopen here.
                    let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                    {
                        let _timers = self.sprite_timers_and_oam_lane_scope();
                        self.sprite_timers_and_oam_after_timer_decrements(interrupted_slot);
                    }
                    self.sprite_execute_single_after_timers(interrupted_slot, state);
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterTimerDecrements { state: None, .. } => unreachable!(
                "source timer decrement boundary did not bind to the saved native dispatch state"
            ),
            SpriteMainCpuBoundary::AfterPrimaryTimerDecrements {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                {
                    // Cycle ledger: resumed inside Sprite_TimersAndOam under
                    // Sprite_ExecuteSingle; both scopes reopen here.
                    let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                    {
                        let _timers = self.sprite_timers_and_oam_lane_scope();
                        self.sprite_timers_and_oam_after_primary_through_timer_decrements(
                            interrupted_slot,
                        );
                        self.sprite_timers_and_oam_after_timer_decrements(interrupted_slot);
                    }
                    self.sprite_execute_single_after_timers(interrupted_slot, state);
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterPrimaryTimerDecrements { state: None, .. } => unreachable!(
                "source primary timer decrement boundary did not bind to the saved native dispatch state"
            ),
            SpriteMainCpuBoundary::AfterHitTimer {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                {
                    // Cycle ledger: resumed inside Sprite_TimersAndOam under
                    // Sprite_ExecuteSingle; both scopes reopen here.
                    let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                    {
                        let _timers = self.sprite_timers_and_oam_lane_scope();
                        self.sprite_timers_and_oam_after_hit_through_timer_decrements(
                            interrupted_slot,
                        );
                        self.sprite_timers_and_oam_after_timer_decrements(interrupted_slot);
                    }
                    self.sprite_execute_single_after_timers(interrupted_slot, state);
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterHitTimer { state: None, .. } => unreachable!(
                "source hit timer decrement boundary did not bind to the saved native dispatch state"
            ),
            SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                {
                    // Cycle ledger: resumed inside Sprite_TimersAndOam under
                    // Sprite_ExecuteSingle; both scopes reopen here.
                    let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                    {
                        let _timers = self.sprite_timers_and_oam_lane_scope();
                        self.sprite_timers_and_oam_after_main_and_aux1_through_primary_timer_decrements(
                            interrupted_slot,
                        );
                        self.sprite_timers_and_oam_after_primary_through_timer_decrements(
                            interrupted_slot,
                        );
                        self.sprite_timers_and_oam_after_timer_decrements(interrupted_slot);
                    }
                    self.sprite_execute_single_after_timers(interrupted_slot, state);
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements { state: None, .. } => {
                unreachable!(
                    "source main/aux1 timer decrement boundary did not bind to the saved native dispatch state"
                )
            }
            SpriteMainCpuBoundary::AfterMainTimerDecrement {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                {
                    // Cycle ledger: resumed inside Sprite_TimersAndOam under
                    // Sprite_ExecuteSingle; both scopes reopen here.
                    let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                    {
                        let _timers = self.sprite_timers_and_oam_lane_scope();
                        self.sprite_timers_and_oam_aux1_timer_decrement(interrupted_slot);
                        self.sprite_timers_and_oam_after_main_and_aux1_through_primary_timer_decrements(
                            interrupted_slot,
                        );
                        self.sprite_timers_and_oam_after_primary_through_timer_decrements(
                            interrupted_slot,
                        );
                        self.sprite_timers_and_oam_after_timer_decrements(interrupted_slot);
                    }
                    self.sprite_execute_single_after_timers(interrupted_slot, state);
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterMainTimerDecrement { state: None, .. } => {
                unreachable!(
                    "source main timer decrement boundary did not bind to the saved native dispatch state"
                )
            }
            SpriteMainCpuBoundary::AfterZeroHitTimerClear {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                {
                    // Cycle ledger: resumed inside Sprite_TimersAndOam under
                    // Sprite_ExecuteSingle; both scopes reopen here. The
                    // second store of $06:8496 (STZ $b89,x, 38) completes the
                    // zero-hit-timer block begun before the boundary.
                    let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                    {
                        let _timers = self.sprite_timers_and_oam_lane_scope();
                        crate::cycle_ledger::charge(38);
                        self.sprite_slot_view_mut(interrupted_slot).set_object_priority(0);
                        self.sprite_timers_and_oam_after_hit_through_timer_decrements(
                            interrupted_slot,
                        );
                        self.sprite_timers_and_oam_after_timer_decrements(interrupted_slot);
                    }
                    self.sprite_execute_single_after_timers(interrupted_slot, state);
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterZeroHitTimerClear { state: None, .. } => {
                unreachable!(
                    "source main timer decrement boundary did not bind to the saved native dispatch state"
                )
            }
            SpriteMainCpuBoundary::BariBeforeRandom(slot) => {
                let interrupted_slot = usize::from(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert!(matches!(
                    self.sprite_slot_view(interrupted_slot).sprite_type(),
                    0x23 | 0x24
                ));
                self.sprite_system_mut().set_cur_object_index(slot);
                self.sprite_prep_bari_after_random_boundary(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterThrowableSceneryStateClear(interrupted_slot) => {
                let interrupted_slot = usize::from(interrupted_slot);
                self.throwable_scenery_scatter_after_state_clear(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                slot,
                continuation: Some(continuation),
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_sprite_slot_after_single_small_draw_position(
                    interrupted_slot,
                    continuation,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
                slot,
                oam_position: Some(oam_position),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_probe_after_oam_coordinates(interrupted_slot, oam_position);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterWallmasterResetPrefix(interrupted_slot) => {
                let interrupted_slot = usize::from(interrupted_slot);
                self.sprite_system_mut()
                    .set_cur_object_index(interrupted_slot as u8);
                self.wall_master_send_player_after_reset_fixed_prefix();
                self.link_initialize();
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::WallmasterResetClear { slot, cleared_bytes } => {
                assert!(cleared_bytes <= 0x1000);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.sprite_workspace_mut().clear_where_in_room_range(
                    0..0x1000 - usize::from(cleared_bytes),
                );
                self.sprite_reset_all_after_room_mask_clear();
                self.wall_master_send_player_after_sprite_reset();
                self.link_initialize();
                self.complete_sprite_main_after_interrupted_slot(usize::from(slot));
            }
            SpriteMainCpuBoundary::ZazakAfterGraphics(interrupted_slot) => {
                let interrupted_slot = usize::from(interrupted_slot);
                self.sprite_zazak_after_graphics_boundary(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterActiveCuccoX {
                slot,
                helper_ordinal,
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_active_cucco_after_x(interrupted_slot, helper_ordinal);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                slot,
                helper_ordinal,
                y_low: Some(y_low),
                y_high: Some(y_high),
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_active_cucco_after_y_subpixel(
                    interrupted_slot,
                    helper_ordinal,
                    y_low,
                    y_high,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::MasterSwordLightBeamMovement {
                slot,
                checkpoint,
                continuation: Some(continuation),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_master_sword_light_beam_movement(
                    interrupted_slot,
                    checkpoint,
                    continuation,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::MasterSwordLightBeamMovement {
                continuation: None,
                ..
            } => unreachable!(
                "source master-sword movement boundary did not bind to native coordinate results"
            ),
            SpriteMainCpuBoundary::BoulderMovement {
                slot,
                checkpoint,
                continuation: Some(continuation),
            } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_boulder_movement(k, checkpoint, continuation);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::BoulderMovement {
                continuation: None,
                ..
            } => unreachable!(
                "source Boulder movement boundary did not bind to native coordinate results"
            ),
            SpriteMainCpuBoundary::BuzzblobAfterXSubpixel { slot, pending: Some((x_low, x_high)) } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_sprite_move_x_after_subpixel(k, x_low, x_high);
                self.sprite_move_y(k);
                self.buzzblob_after_movement(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::BuzzblobAfterXSubpixel { pending: None, .. } => {
                unreachable!("Buzzblob movement did not bind its pending coordinate result")
            }
            SpriteMainCpuBoundary::AfterCuccoFleeMovement {
                slot,
                helper_ordinal,
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_cucco_flee_after_movement(interrupted_slot, helper_ordinal);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                slot,
                completed,
                total,
                continuation: Some(continuation),
                ..
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_cucco_after_subtype_increments(
                    interrupted_slot,
                    completed,
                    total,
                    continuation,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                slot,
                helper_ordinal: _,
                continuation: Some(continuation),
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_cucco_after_graphics_publication(interrupted_slot, continuation);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                continuation: None, ..
            }
            | SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel { .. }
            | SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                continuation: None, ..
            } => unreachable!("source Cucco boundary did not bind to a native C call site"),
            SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                continuation: None, ..
            } => unreachable!(
                "source single-small draw boundary did not bind to a native C call site"
            ),
            SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
                oam_position: None,
                ..
            } => unreachable!(
                "source guard-probe boundary did not bind to its native OAM coordinates"
            ),
            SpriteMainCpuBoundary::BigKeyDropGraphicsStarted(_) => {
                unreachable!(
                    "big-key graphics boundary transfers directly to its typed continuation"
                )
            }
            SpriteMainCpuBoundary::ItemReceiptGraphicsStarted(_) => {
                unreachable!(
                    "item-receipt graphics boundary transfers directly to its typed continuation"
                )
            }
            SpriteMainCpuBoundary::WishPondTossedItemGraphics {
                slot,
                continuation: Some(continuation),
            } => {
                let slot = usize::from(slot);
                self.complete_wish_pond_tossed_item_graphics(slot, continuation);
                self.complete_sprite_main_after_interrupted_slot(slot);
            }
            SpriteMainCpuBoundary::WishPondTossedItemGraphics {
                continuation: None,
                ..
            } => unreachable!(
                "source Wish Pond graphics boundary did not bind its native caller suffix"
            ),
            SpriteMainCpuBoundary::KingZoraFlippersGraphicsStarted(slot) => {
                let slot = usize::from(slot);
                self.decode_animated_sprite_tile_variable(0x11);
                self.complete_sprite_main_after_interrupted_slot(slot);
            }            SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(slot) => {
                let slot = usize::from(slot);
                self.complete_happiness_pond_rupee_graphics(slot);
                self.complete_sprite_main_after_interrupted_slot(slot);
            }
            SpriteMainCpuBoundary::AfterZeldaFollowerGraphics {
                slot,
                saved_follower_indicator,
            } => {
                let slot = usize::from(slot);
                self.load_follower_graphics();
                self.sprite_prep_zelda_after_follower_graphics(slot, saved_follower_indicator);
                self.complete_sprite_main_after_interrupted_slot(slot);
            }
            SpriteMainCpuBoundary::FollowerGraphics {
                slot,
                caller,
                prefix_completed: true,
                saved_follower_indicator,
                stage,
            } => {
                let slot = usize::from(slot);
                self.complete_follower_graphics_decompression(stage);
                match caller {
                    crate::SpriteFollowerGraphicsCaller::BlindMaiden => {
                        assert_eq!(saved_follower_indicator, None);
                        self.sprite_prep_blind_maiden_after_follower_graphics();
                    }
                    crate::SpriteFollowerGraphicsCaller::Zelda => {
                        self.sprite_prep_zelda_after_follower_graphics(
                            slot,
                            saved_follower_indicator.expect(
                                "Zelda follower-graphics continuation lost its saved follower",
                            ),
                        );
                    }
                    crate::SpriteFollowerGraphicsCaller::BlindMaidenBody => {
                        assert_eq!(saved_follower_indicator, None);
                        self.sprite_b7_blind_maiden_after_follower_graphics(slot);
                    }
                    crate::SpriteFollowerGraphicsCaller::PurpleChest => {
                        assert_eq!(saved_follower_indicator, None);
                        self.sprite_become_follower(slot);
                    }
                    crate::SpriteFollowerGraphicsCaller::SuperBomb => {
                        assert_eq!(saved_follower_indicator, None);
                        self.sprite_bomb_shop_super_bomb_after_follower_graphics(slot);
                    }
                    crate::SpriteFollowerGraphicsCaller::OldMan => {
                        let reset_follower_after_graphics = saved_follower_indicator
                            .expect("Old Man follower-graphics continuation lost its suffix flag");
                        assert!(reset_follower_after_graphics <= 1);
                        self.sprite_prep_old_man_after_follower_graphics(
                            reset_follower_after_graphics != 0,
                        );
                    }
                }
                self.complete_sprite_main_after_interrupted_slot(slot);
            }
            SpriteMainCpuBoundary::FollowerGraphics {
                prefix_completed: false,
                ..
            } => unreachable!("source follower-graphics boundary did not bind its caller prefix"),
            SpriteMainCpuBoundary::BeforeZeldaFollowerGraphics(_) => {
                unreachable!("unstarted Zelda prep boundary cannot complete scheduled work")
            }
            SpriteMainCpuBoundary::BonkItemGraphicsEntered(slot) => {
                // The initialize prefix already ran at suspension time; only
                // the interrupted sheet decode and the loop below this slot
                // remain (SpritePrep_BonkItem's room-$107 branch has no
                // statements after the decode).
                let slot = usize::from(slot);
                self.decode_animated_sprite_tile_variable(0x0e);
                self.complete_sprite_main_after_interrupted_slot(slot);
            }
            SpriteMainCpuBoundary::CatfishMedallionGraphicsStarted(slot) => {
                self.complete_catfish_medallion_graphics(usize::from(slot));
                self.complete_sprite_main_after_interrupted_slot(usize::from(slot));
            }
            SpriteMainCpuBoundary::TrinexxHeadDrawSetup(slot) => {
                self.sidenexx_after_head_draw_setup(usize::from(slot));
                self.complete_sprite_main_after_interrupted_slot(usize::from(slot));
            }
            SpriteMainCpuBoundary::WaterfallGtCutsceneGraphicsStarted(slot) => {
                self.complete_waterfall_gt_cutscene_graphics(usize::from(slot));
                self.complete_sprite_main_after_interrupted_slot(usize::from(slot));
            }
            SpriteMainCpuBoundary::TrinexxBreathTileCollisionReturned(slot) => {
                self.trinexx_breath_after_tile_collision(usize::from(slot));
                self.complete_sprite_main_after_interrupted_slot(usize::from(slot));
            }
            SpriteMainCpuBoundary::LaserEyeDrawPrologue(slot) => {
                self.sprite_95_laser_eye_after_draw_prologue(usize::from(slot));
                self.complete_sprite_main_after_interrupted_slot(usize::from(slot));
            }
                        SpriteMainCpuBoundary::ZoraFireballMovement {
                slot,
                checkpoint,
                continuation: Some(continuation),
            } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_zora_fireball_movement(k, checkpoint, continuation);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::ZoraFireballMovement {
                continuation: None,
                ..
            } => unreachable!(
                "source Zora fireball movement boundary did not bind to native coordinate results"
            ),
            SpriteMainCpuBoundary::GuardPrepWeaponFlagsPending {
                slot,
                continuation: Some(continuation),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert_eq!(self.sprite_slot_view(interrupted_slot).sprite_type(), 0x41);
                self.complete_sprite_prep_standard_guard_after_weapon_flags(
                    interrupted_slot,
                    continuation,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::GuardPrepWeaponFlagsPending {
                continuation: None,
                ..
            } => unreachable!(
                "source guard-prep weapon boundary did not bind its native caller suffix"
            ),
            SpriteMainCpuBoundary::GuardPrepParryHitbox { slot, active_call, continuation: Some(continuation) } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                self.complete_sprite_prep_standard_guard_after_parry_hitbox(k, active_call, continuation);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::GuardPrepParryHitbox { continuation: None, .. } => unreachable!("guard parry checkpoint did not bind its source locals"),
            SpriteMainCpuBoundary::GuardPrepPatrolDelay { slot, active_call, saved_submodule: Some(saved_submodule) } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                self.complete_sprite_prep_standard_guard_after_patrol_delay(k, active_call, saved_submodule);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::GuardPrepPatrolDelay { saved_submodule: None, .. } => unreachable!("guard patrol checkpoint lost its saved initializer submodule"),
            SpriteMainCpuBoundary::GuardPrepTileCollisionReturned { slot, active_call, saved_submodule: Some(saved_submodule) } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                self.complete_sprite_prep_standard_guard_after_tile_collision_return(k, active_call, saved_submodule);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::GuardPrepTileCollisionReturned { saved_submodule: None, .. } => unreachable!("guard patrol checkpoint lost its saved initializer submodule"),
            SpriteMainCpuBoundary::GuardAnimation { slot, continuation: Some(continuation), .. } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                self.complete_guard_animation_at_checkpoint(k, continuation);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::GuardAnimation { continuation: None, .. } => unreachable!("guard weapon checkpoint did not bind its source locals"),
            SpriteMainCpuBoundary::RatOamFlagsPending { slot, flags } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                self.complete_rat_animation_after_graphics(k, flags);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
            SpriteMainCpuBoundary::KeeseInactiveCheckPending { slot } => {
                let k = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_keese_after_draw(k);
                self.complete_sprite_main_after_interrupted_slot(k);
            }
        }
    }

    pub(in crate::zelda_rtl) fn complete_sprite_slot_after_single_small_draw_position(
        &mut self,
        slot: usize,
        continuation: SingleSmallDrawContinuation,
    ) {
        assert_eq!(self.sprite_slot_view(slot).state(), 9);
        assert!(matches!(
            self.sprite_slot_view(slot).sprite_type(),
            0x23 | 0x24
        ));
        self.sprite_system_mut().set_cur_object_index(slot as u8);
        self.sprite_draw_single_small_after_position(slot, continuation);
        self.sprite_23_red_bari_after_draw(slot);
    }

    pub(super) fn complete_sprite_main_after_all_slots(&mut self) {
        self.complete_sprite_main_after_all_slots_with_lower_garnish(None);
    }

    fn complete_sprite_main_after_all_slots_with_lower_garnish(
        &mut self,
        native: Option<crate::cycle_models::dungeon_lower_garnish::LowerGarnishTail>,
    ) {
        self.complete_sprite_main_after_all_slots_with_native_suffix(native, None);
    }

    pub(super) fn complete_sprite_main_after_all_slots_with_native_suffix(
        &mut self,
        native: Option<crate::cycle_models::dungeon_lower_garnish::LowerGarnishTail>,
        cached: Option<crate::cycle_models::dungeon_cached_sprites::CachedSpritesTail>,
    ) {
        // Every Sprite_Main body reaching its slot-zero return crosses this
        // boundary, whether it ran whole or resumed from a saved CPU slot, so
        // the one claimed source return is consumed here exactly once.
        if let Some(claims_remaining) = self.original_timing_sprite_main_return_claims_remaining {
            assert_ne!(
                claims_remaining, 0,
                "the native body returned from more Sprite_Main loops than its immutable host plan claimed: host={} frame={:?} scheduler={:?}",
                self.frame_ctr_dbg, self.game_state.frame, self.game_execution_scheduler,
            );
            assert!(
                self.take_original_timing_sprite_main_returned(),
                "the native Sprite_Main slot-zero boundary lost its claimed source return receipt",
            );
            self.original_timing_sprite_main_return_claims_remaining = Some(claims_remaining - 1);
        }
        // Sprite_Main suffix $06:83AA-83BC: JSL Garnish lower slots, STZ
        // $069f, STZ $069e, PLB, JSL ExecuteCachedSprites, LDA $0aaa, BEQ
        // not taken (264; the callees charge their own bodies).
        if let Some(tail) = cached {
            // The native pass retired the scroll clears, PLB, JSL and cache guards/scan.
            crate::cycle_ledger::charge(48); // LDA $0aaa : BEQ not taken.
            if let crate::cycle_models::dungeon_cached_sprites::CachedSpritesTail::Loaded { slot, backup,state,timer } =
                tail
            {
                self.execute_cached_sprites_after_native_entry(slot,&backup,state,timer);
            }
        } else {
            crate::cycle_ledger::charge(if native.is_some() { 202 } else { 264 });
            if let Some(tail) = native {
                self.finish_native_lower_garnish(tail);
            } else {
                self.garnish_execute_lower_slots();
            }
            self.clear_overworld_vertical_scroll_delta_low();
            self.set_overworld_horizontal_scroll_delta_low(0);
            self.execute_cached_sprites();
        }
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            return;
        }
        if self.game_state.display.has_chr_halfslot_request() {
            // $06:83BE STA $0fc6 (32) + $06:83C1 RTL (44).
            crate::cycle_ledger::charge(32 + 44);
            let chr_halfslot_request = self.game_state.display.chr_halfslot_request;
            self.sprite_system_mut()
                .set_chr_halfslot_state(chr_halfslot_request);
        } else {
            // BEQ taken (+6) + $06:83C1 RTL (44).
            crate::cycle_ledger::charge(6 + 44);
        }
    }
}
