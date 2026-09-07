//! The large per-event arms of `Snes9xOracleSemanticTrace::consume_event`
//! (mechanically extracted; bodies unchanged).

use super::*;

impl Snes9xOracleSemanticTrace {
    /// Extracted from `consume_event` (mechanical move; body unchanged).
    pub(super) fn consume_pc_event(
        &mut self,
        event: &RawTraceEvent,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    ) -> Result<(), String> {
        let pc = event.pc.ok_or("Snes9x PC receipt omitted PC")? & 0x00ff_ffff;
        if pc == 0x02_824d
            && (event.main, event.sub) == (Some(5), Some(0))
            && event.return_address.map(|pc| pc & 0xffffff) == Some(0x00_8059)
        {
            // Module05 tail-enters PreDungeon under the game-loop
            // dispatcher. Starting-point selection also sets main=5,
            // but calls this body from its own $02:85AD return.
            receipts.push(OriginalTimingSemanticReceipt::SelectedGameEntranceReturned);
        }
        if pc == DUNGEON_RESET_SPRITES_RETURN_PC {
            self.pending_reset_progress = None;
            self.last_host_return_reset_progress = None;
            self.cache_write_progress = None;
            self.normal_load_ordinal = None;
        }
        if pc == SAVE_QUIT_RESET_DUNGEON_INFO_CLEAR_ENTRY_PC {
            if (event.main, event.sub, event.subsub) != (Some(0), Some(10), Some(10)) {
                return Err(format!(
                    "Snes9x save-quit reset prefix returned with unexpected module state {:?}/{:?}/{:?}",
                    event.main, event.sub, event.subsub,
                ));
            }
            receipts.retain(|receipt| {
                *receipt != OriginalTimingSemanticReceipt::SaveQuitIntroMemoryReturned
            });
            receipts.push(OriginalTimingSemanticReceipt::SaveQuitResetStatePublished);
        }
        if pc == FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_RETURN_PC
            && (event.main, event.sub) == (Some(1), Some(1))
        {
            receipts.retain(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramClearProgress(_)
                )
            });
            receipts.push(OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramCleared);
        }
        if pc == SELECTED_GAME_LOAD_MESSAGE_INTERFACE_RETURN_PC
            && event.return_address.map(|pc| pc & 0x00ff_ffff)
                == Some(MODULE05_AFTER_SHOW_TEXT_MESSAGE_PC)
            && (event.main, event.sub) == (Some(14), Some(2))
        {
            receipts.push(OriginalTimingSemanticReceipt::SelectedGameLoadMessageInterfacePublished);
        }
        if pc == RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC
            && (event.main, event.sub, event.subsub) == (Some(7), Some(0x18), Some(10))
        {
            if self.rescued_maiden_initialization.is_some() {
                return Err(
                    "Snes9x re-entered rescued-maiden follower graphics before its prior call returned"
                        .to_string(),
                );
            }
            self.rescued_maiden_initialization =
                Some(RescuedMaidenInitializationTracker::first_sheet());
        }
        let sprite_follower_graphics_caller = match event.return_address.map(|pc| pc & 0x00ff_ffff)
        {
            Some(SPRITE_PREP_BLIND_MAIDEN_FOLLOWER_GRAPHICS_RETURN_PC) => {
                Some(SpriteFollowerGraphicsCaller::BlindMaiden)
            }
            Some(SPRITE_PREP_ZELDA_FOLLOWER_GRAPHICS_RETURN_PC) => {
                Some(SpriteFollowerGraphicsCaller::Zelda)
            }
            Some(SPRITE_BLIND_MAIDEN_BODY_FOLLOWER_GRAPHICS_RETURN_PC) => {
                Some(SpriteFollowerGraphicsCaller::BlindMaidenBody)
            }
            Some(SPRITE_PREP_OLD_MAN_FOLLOWER_GRAPHICS_RETURN_PC) => {
                Some(SpriteFollowerGraphicsCaller::OldMan)
            }
            Some(SPRITE_PURPLE_CHEST_FOLLOWER_GRAPHICS_RETURN_PC) => {
                Some(SpriteFollowerGraphicsCaller::PurpleChest)
            }
            Some(0x1e_e201) => Some(SpriteFollowerGraphicsCaller::SuperBomb),
            _ => None,
        };
        if pc == RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC
            && sprite_follower_graphics_caller.is_some()
        {
            let caller = sprite_follower_graphics_caller
                .expect("checked sprite follower-graphics caller disappeared");
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Snes9x entered sprite follower graphics outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Snes9x entered follower graphics before a Sprite_Main slot")?;
            if execution.follower_graphics.is_some() {
                return Err(format!(
                    "Snes9x re-entered {caller:?} follower graphics in slot {slot} before its prior call returned",
                ));
            }
            execution.follower_graphics =
                Some((caller, RescuedMaidenInitializationTracker::first_sheet()));
        }
        if pc == RESCUED_MAIDEN_FIRST_FOLLOWER_SHEET_ENTRY_PC
            && (self.rescued_maiden_initialization.is_some()
                || self
                    .sprite_main_execution
                    .as_ref()
                    .is_some_and(|execution| execution.follower_graphics.is_some()))
        {
            let purple_chest = self
                .sprite_main_execution
                .as_ref()
                .is_some_and(|execution| {
                    execution
                        .follower_graphics
                        .as_ref()
                        .is_some_and(|(caller, _)| {
                            matches!(
                                *caller,
                                SpriteFollowerGraphicsCaller::PurpleChest
                                    | SpriteFollowerGraphicsCaller::SuperBomb
                            )
                        })
                });
            let valid_sheet = if purple_chest {
                event.y == Some(0x58)
            } else {
                matches!(event.y, Some(0x64 | 0x66))
            };
            if !valid_sheet {
                return Err(format!(
                    "Snes9x rescued-maiden first follower sheet used unexpected asset {:?}",
                    event.y,
                ));
            }
        }
        if pc == RESCUED_MAIDEN_SECOND_FOLLOWER_SHEET_ENTRY_PC {
            if let Some(tracker) = self.rescued_maiden_initialization.as_mut() {
                if event.y != Some(0x65) {
                    return Err(format!(
                        "Snes9x rescued-maiden second follower sheet used unexpected asset {:?}",
                        event.y,
                    ));
                }
                tracker.begin_second_sheet()?;
            }
            if let Some(tracker) = self
                .sprite_main_execution
                .as_mut()
                .and_then(|execution| execution.follower_graphics.as_mut())
            {
                if event.y != Some(0x65) {
                    return Err(format!(
                        "Snes9x Zelda's second follower sheet used unexpected asset {:?}",
                        event.y,
                    ));
                }
                tracker.1.begin_second_sheet()?;
            }
        }
        if pc == RESCUED_MAIDEN_FOLLOWER_SHEETS_RETURN_PC {
            if let Some(tracker) = self.rescued_maiden_initialization.as_mut() {
                tracker.begin_conversion()?;
            }
            if let Some(tracker) = self
                .sprite_main_execution
                .as_mut()
                .and_then(|execution| execution.follower_graphics.as_mut())
            {
                tracker.1.begin_conversion()?;
            }
        }
        if pc == POLYHEDRAL_RENDER_START_PC && matches!(event.main, Some(0x07 | 0x0e | 0x19)) {
            receipts.push(OriginalTimingSemanticReceipt::PreemptivePolyhedralRenderStarted);
        }
        if pc == OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC
            && matches!(
                event.return_address.map(|pc| pc & 0x00ff_ffff),
                Some(
                    OVERWORLD_LOAD_OVERLAYS_AFTER_SPRITE_RELOAD_PC
                        | BIRD_TRAVEL_AFTER_SPRITE_RELOAD_PC
                        | MIRROR_WARP_AFTER_SPRITE_RELOAD_PC
                        | PRE_OVERWORLD_AFTER_SPRITE_RELOAD_PC
                )
            )
        {
            if self.overworld_load_overlays_sprite_reload_active {
                return Err(
                    "Snes9x re-entered Overworld_LoadOverlays sprite reload before its prior call returned"
                        .to_string(),
                );
            }
            self.overworld_load_overlays_sprite_reload_active = true;
            self.overworld_sprite_reload_reset_published = false;
            self.overworld_presence_published = false;
            self.overworld_sprite_activation = None;
        }
        if pc == OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_RETURN_PC
            && self.overworld_load_overlays_sprite_reload_active
        {
            self.overworld_load_overlays_sprite_reload_active = false;
            self.overworld_sprite_reload_reset_published = false;
            if !matches!((event.main, event.sub), (Some(8), Some(0))) {
                receipts.push(
                    OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                        OverworldSpriteReloadProgress::GenerationReturned,
                    ),
                );
            }
        }
        // The body has completed movement and collision. The shared
        // jump-table helper has no gameplay effects before the AI target.
        if pc == 0x06_9853 {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Mini Moldorm AI dispatch has no Sprite_Main owner")?;
            let slot = execution
                .current_slot
                .ok_or("Mini Moldorm dispatch has no slot")?;
            if event.x != Some(u16::from(slot)) {
                return Err("Mini Moldorm dispatch disagrees with its active slot".into());
            }
            execution.mini_moldorm_ai_pending = Some(slot);
        }
        if matches!(pc, 0x06_9860 | 0x06_988d | 0x06_98b2) {
            if let Some(execution) = self.sprite_main_execution.as_mut() {
                execution.mini_moldorm_ai_pending = None;
            }
        }
        if pc == 0x06_e4ab {
            if let Some(execution) = self.sprite_main_execution.as_mut() {
                if execution
                    .boulder_movement
                    .is_some_and(|(slot, ..)| event.x == Some(u16::from(slot)))
                {
                    execution.boulder_movement = None;
                    execution.zora_fireball = None;
                    execution.laser_eye_draw_prologue = None;
                }
                if let Some((slot, _, moved)) = execution.zora_fireball.as_mut() {
                    if event.x == Some(u16::from(*slot)) {
                        // The fireball's frame-gated tile collision
                        // follows its movement.
                        *moved = true;
                    }
                }
            }
        }
        if pc == 0x06_e4ab
            && event
                .return_address
                .is_some_and(|stack| stack & 0xffff == 0x98f5)
        {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Moblin collision has no Sprite_Main owner")?;
            if execution.current_slot.map(u16::from) != event.x {
                return Err("Moblin collision disagrees with its active slot".into());
            }
            execution.moblin_collision_started = true;
        }
        if pc == DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC
            && event.return_address.map(|pc| pc & 0x00ff_ffff)
                == Some(ZORA_FLIPPERS_GRAPHICS_RETURN_ADDRESS)
        {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Snes9x entered King Zora flippers graphics outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Snes9x entered King Zora flippers graphics before a sprite slot")?;
            execution.king_zora_flippers_graphics_slot = Some(slot);
        }
        if pc == DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC
            && event.return_address.map(|pc| pc & 0x00ff_ffff) == Some(0x09_8b05)
        {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Snes9x entered Happiness Pond rupee graphics outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Snes9x entered Happiness Pond rupee graphics before a sprite slot")?;
            execution.happiness_pond_rupee_graphics_slot = Some(slot);
        }
        if pc == DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC
            && event.return_address.map(|pc| pc & 0x00ff_ffff) == Some(0x1d_e1a6)
        {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Snes9x entered Catfish medallion graphics outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Snes9x entered Catfish medallion graphics before a sprite slot")?;
            execution.catfish_medallion_graphics_slot = Some(slot);
        }
        if pc == DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC
            && event.return_address.map(|pc| pc & 0x00ff_ffff) == Some(0x09_9bd5)
        {
            // AncillaAdd_GTCutscene ($09:9B83) decodes the cutscene
            // graphics after killing the waterfall trigger sprites.
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Snes9x entered GT cutscene graphics outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Snes9x entered GT cutscene graphics before a sprite slot")?;
            execution.waterfall_gt_cutscene_graphics_slot = Some(slot);
        }
        if pc == DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC
            && event.return_address.map(|pc| pc & 0x00ff_ffff)
                == Some(BONK_ITEM_GRAPHICS_RETURN_ADDRESS)
        {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Snes9x entered bonk-item graphics outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Snes9x entered bonk-item graphics before a sprite slot")?;
            execution.bonk_item_graphics_slot = Some(slot);
        }
        if pc == DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC
            && event.return_address.map(|pc| pc & 0x00ff_ffff)
                == Some(WISH_POND_TOSSED_ITEM_GRAPHICS_RETURN_ADDRESS)
        {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Snes9x entered Wish Pond tossed-item graphics outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Snes9x entered Wish Pond tossed-item graphics before a sprite slot")?;
            execution.wish_pond_tossed_item_graphics_slot = Some(slot);
        }
        match pc {
            0x06_d051 => {
                if let Some(execution) = self.sprite_main_execution.as_mut() {
                    if execution.current_slot.map(u16::from) != event.x {
                        return Err("absorbable body entry disagrees with its current slot".into());
                    }
                    execution.absorbable_body_active = true;
                }
            }
            SPRITE_MAIN_ENTRY_PC => {
                if let Some(tracker) = self.rescued_maiden_initialization.take() {
                    if tracker.phase != RescuedMaidenInitializationTrackerPhase::Converting {
                        return Err(format!(
                            "Snes9x rescued-maiden caller entered Sprite_Main from {:?}",
                            tracker.phase,
                        ));
                    }
                }
                // A fresh entry is also a source-level proof that any
                // prior call returned. Different module callers have
                // different private return PCs, so the generic adapter
                // closes the old tracker here instead of enumerating
                // every call site.
                if let Some(caller) = self.item_receipt_caller.take() {
                    receipts.push(OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                        ItemReceiptGraphicsProgressReceipt {
                            caller,
                            progress: SourceCallProgress::Returned,
                        },
                    ));
                }
                if self.sprite_main_execution.take().is_some() {
                    receipts.push(OriginalTimingSemanticReceipt::SpriteMainReturned);
                }
                self.sprite_main_execution = Some(SpriteMainExecutionTracker::default());
            }
            SPRITE_EXECUTE_SINGLE_ENTRY_PC => {
                // ExecuteCachedSprites calls this leaf directly after
                // Sprite_Main's descending loop has returned. Those
                // calls belong to the separate cached-sprite tracker
                // and must not reopen or corrupt the Sprite_Main
                // cursor. Only an active Sprite_Main entry owns this
                // receipt domain.
                if let Some(execution) = self.sprite_main_execution.as_mut() {
                    let slot = u8::try_from(
                        event
                            .x
                            .ok_or("Snes9x Sprite_ExecuteSingle receipt omitted slot X")?,
                    )
                    .map_err(|_| "Snes9x Sprite_ExecuteSingle slot exceeded one byte")?;
                    if slot >= 16 {
                        return Err(format!(
                            "Snes9x Sprite_ExecuteSingle used invalid slot {slot}"
                        ));
                    }
                    execution.current_slot = Some(slot);
                    execution.timers_and_oam_slot = None;
                    execution.timers_and_oam_dispatch_state = None;
                    execution.initialize_active_main_calls = 0;
                    execution.guard_prep_parry_hitbox = None;
                    execution.guard_prep_patrol_delay = None;
                    execution.guard_prep_tile_collision_return = None;
                    execution.guard_animation_checkpoint = None;
                    execution.hog_spear_body_graphics_pending = None;
                    execution.absorbable_body_active = false;
                    execution.absorbable_horizontal_lookup = None;
                    execution.absorbable_vertical_lookup = None;
                    execution.absorbable_vertical_attribute_loaded = None;
                    execution.dispatch_trampoline_return = None;
                    execution.moblin_collision_started = false;
                    execution.moblin_attribute_loaded = None;
                    execution.moblin_collision_geometry = None;
                    execution.vitreous_minions_seen = false;
                    execution.vitreous_player_damage_pending = None;
                    execution.vitreous_ai_pending = None;
                    execution.mini_moldorm_ai_pending = None;
                    execution.vitreous_damage_pending = None;
                    execution.swamola_segment = None;
                    execution.swamola_head_prepared = false;
                    execution.swamola_head_draw_completed = None;
                    execution.swamola_head_draw = None;
                    execution.swamola_segment_draw = None;
                    execution.pengator_slide_pending = None;
                    execution.antifairy_bounce_pending = None;
                    execution.kholdstare_subtype_decremented = false;
                    execution.kholdstare_damage_pending = None;
                    execution.initialize_prep_pending = None;
                    execution.initialize_prep_move_y = None;
                    execution.guard_animation_pose_slot = None;
                    execution.guard_prep_weapon_flags_pending_slot = None;
                    execution.mini_moldorm_history = None;
                    execution.initialize_reset_properties = None;
                    execution.initialize_load_properties = None;
                    execution.fire_debirando_property_reload = false;
                    execution.fire_debirando_before_spawn_slot = None;
                    execution.fire_debirando_spawn = None;
                    execution.trinexx_death_spawn = None;
                    execution.agahnim_motion_blur_spawn = None;
                    execution.antfairy_subtype2_increment_slot = None;
                    execution.lanmola_subtype2_increment_slot = None;
                    execution.lanmola_draw_prefix = None;
                    execution.helmasaur_hard_hat_beetle_subtype2_increment_slot = None;
                    execution.timer_decrements_slot = None;
                    execution.primary_timer_decrements_slot = None;
                    execution.main_timer_decrement_slot = None;
                    execution.zero_hit_timer_clear_slot = None;
                    execution.main_and_aux1_timer_decrements_slot = None;
                    execution.hit_timer_slot = None;
                    execution.bari_before_random_slot = None;
                    execution.throwable_scenery_state_clear_slot = None;
                    execution.cucco_subtype_increments = None;
                    execution.cucco_animation_slot = None;
                    execution.cucco_flee_movement = None;
                    execution.active_cucco_movement = None;
                    execution.active_cucco_x_publications = 0;
                    execution.active_cucco_y_subpixel = None;
                    execution.master_sword_light_beam_movement = None;
                    execution.boulder_movement = None;
                    execution.zora_fireball = None;
                    execution.laser_eye_draw_prologue = None;
                    execution.master_sword_light_beam_spawn = None;
                    execution.cucco_helper_ordinal = 0;
                    execution.big_key_drop_graphics_slot = None;
                    execution.king_zora_flippers_graphics_slot = None;
                    execution.happiness_pond_rupee_graphics_slot = None;
                    execution.catfish_medallion_graphics_slot = None;
                    execution.waterfall_gt_cutscene_graphics_slot = None;
                    execution.bonk_item_graphics_slot = None;
                    execution.single_small_draw_position_slot = None;
                    execution.probe_after_oam_coordinates_slot = None;
                    execution.wallmaster_reset_prefix_slot = None;
                    execution.wallmaster_reset_cleared_bytes = None;
                    execution.zazak_graphics_slot = None;
                    execution.follower_graphics = None;
                }
            }
            SPRITE_ACTIVE_MAIN_ENTRY_PC => {
                if let Some(execution) = self.sprite_main_execution.as_mut() {
                    execution.hog_spear_active_body = false;
                    execution.buzzblob_movement = None;
                    execution.trinexx_head_draw = None;
                    execution.trinexx_segment_counter = None;
                    execution.trinexx_head_draw_setup = None;
                    execution.trinexx_breath_tile_collision = None;
                    execution.handler_returned_slot = None;
                    execution.trinexx_final_phase_case0 = None;
                    execution.trinexx_final_phase_tile_collision = None;
                    execution.helmasaur_hard_hat_tile_collision = None;
                    execution.trinexx_d_draw_counter = None;
                    execution.trinexx_d_draw_active = None;
                    execution.trinexx_final_phase_draw = None;
                    execution.sidenexx_neck_target = None;
                    execution.trinexx_front_part = None;
                    execution.guard_prep_parry_hitbox = None;
                    execution.guard_prep_patrol_delay = None;
                    execution.guard_prep_tile_collision_return = None;
                    execution.guard_animation_checkpoint = None;
                    execution.hog_spear_body_graphics_pending = None;
                    execution.absorbable_body_active = false;
                    execution.absorbable_horizontal_lookup = None;
                    execution.absorbable_vertical_lookup = None;
                    execution.absorbable_vertical_attribute_loaded = None;
                    execution.dispatch_trampoline_return = None;
                    execution.moblin_collision_started = false;
                    execution.moblin_attribute_loaded = None;
                    execution.moblin_collision_geometry = None;
                    execution.vitreous_minions_seen = false;
                    execution.vitreous_player_damage_pending = None;
                    execution.vitreous_ai_pending = None;
                    execution.mini_moldorm_ai_pending = None;
                    execution.vitreous_damage_pending = None;
                    execution.swamola_segment = None;
                    execution.swamola_head_prepared = false;
                    execution.swamola_head_draw_completed = None;
                    execution.swamola_head_draw = None;
                    execution.swamola_segment_draw = None;
                    execution.pengator_slide_pending = None;
                    execution.antifairy_bounce_pending = None;
                    execution.kholdstare_subtype_decremented = false;
                    execution.kholdstare_damage_pending = None;
                    execution.initialize_prep_pending = None;
                    execution.initialize_prep_move_y = None;
                    execution.guard_animation_pose_slot = None;
                    if execution.timers_and_oam_dispatch_state == Some(8) {
                        execution.initialize_active_main_calls = execution
                            .initialize_active_main_calls
                            .checked_add(1)
                            .ok_or("Snes9x state-8 initializer active-call count overflowed")?;
                    }
                }
            }
            0x05_cbe0 | 0x06_d8e2 | 0x06_d8e5 => {
                if let Some(execution) = self.sprite_main_execution.as_mut() {
                    execution.observe_buzzblob_movement(event)?;
                    execution.observe_trinexx_head_draw(event)?;
                    execution.observe_trinexx_breath_tile_collision(event)?;
                    execution.observe_lanmola_draw_prefix(event)?;
                    execution.observe_laser_eye_draw_prologue(event);
                    execution.observe_sprite_handler_returned(event)?;
                    execution.observe_trinexx_final_phase_tile_collision(event)?;
                    execution.observe_helmasaur_hard_hat_tile_collision(event)?;
                    execution.observe_trinexx_final_phase_draw(event)?;
                    execution.observe_sidenexx_neck_target(event)?;
                    execution.observe_guard_animation_checkpoint(event)?;
                }
            }
            SPRITE_TIMERS_AND_OAM_RETURN_PC => {
                if let Some(execution) = self.sprite_main_execution.as_mut() {
                    execution.observe_bari_before_random(event)?;
                    execution.observe_timers_and_oam_return(event)?;
                }
            }
            SPRITE_TIMER_DECREMENTS_TRACE_PC => {
                if let Some(execution) = self.sprite_main_execution.as_mut() {
                    execution.observe_timer_decrements(event)?;
                }
            }
            SPRITE_SLOT_RETURN_PC => {
                let execution = self
                    .sprite_main_execution
                    .as_mut()
                    .ok_or("Snes9x returned one sprite slot outside Sprite_Main")?;
                let slot = execution
                    .current_slot
                    .ok_or("Snes9x returned one Sprite_Main slot before entering it")?;
                execution.last_completed_slot = Some(slot);
                execution.timers_and_oam_slot = None;
                execution.timers_and_oam_dispatch_state = None;
                execution.initialize_active_main_calls = 0;
                execution.guard_prep_parry_hitbox = None;
                execution.guard_prep_patrol_delay = None;
                execution.guard_prep_tile_collision_return = None;
                execution.guard_animation_checkpoint = None;
                execution.hog_spear_body_graphics_pending = None;
                execution.absorbable_body_active = false;
                execution.absorbable_horizontal_lookup = None;
                execution.absorbable_vertical_lookup = None;
                execution.absorbable_vertical_attribute_loaded = None;
                execution.dispatch_trampoline_return = None;
                execution.moblin_collision_started = false;
                execution.moblin_attribute_loaded = None;
                execution.moblin_collision_geometry = None;
                execution.vitreous_minions_seen = false;
                execution.vitreous_player_damage_pending = None;
                execution.vitreous_ai_pending = None;
                execution.mini_moldorm_ai_pending = None;
                execution.vitreous_damage_pending = None;
                execution.swamola_segment = None;
                execution.swamola_head_prepared = false;
                execution.swamola_head_draw_completed = None;
                execution.swamola_head_draw = None;
                execution.swamola_segment_draw = None;
                execution.pengator_slide_pending = None;
                execution.antifairy_bounce_pending = None;
                execution.kholdstare_subtype_decremented = false;
                execution.kholdstare_damage_pending = None;
                execution.initialize_prep_pending = None;
                execution.initialize_prep_move_y = None;
                execution.guard_animation_pose_slot = None;
                execution.guard_prep_weapon_flags_pending_slot = None;
                execution.mini_moldorm_history = None;
                execution.initialize_reset_properties = None;
                execution.initialize_load_properties = None;
                execution.fire_debirando_property_reload = false;
                execution.fire_debirando_before_spawn_slot = None;
                execution.fire_debirando_spawn = None;
                execution.trinexx_death_spawn = None;
                execution.agahnim_motion_blur_spawn = None;
                execution.antfairy_subtype2_increment_slot = None;
                execution.lanmola_subtype2_increment_slot = None;
                execution.lanmola_draw_prefix = None;
                execution.helmasaur_hard_hat_beetle_subtype2_increment_slot = None;
                execution.timer_decrements_slot = None;
                execution.primary_timer_decrements_slot = None;
                execution.main_timer_decrement_slot = None;
                execution.zero_hit_timer_clear_slot = None;
                execution.main_and_aux1_timer_decrements_slot = None;
                execution.hit_timer_slot = None;
                execution.bari_before_random_slot = None;
                execution.throwable_scenery_state_clear_slot = None;
                execution.cucco_subtype_increments = None;
                execution.cucco_animation_slot = None;
                execution.cucco_flee_movement = None;
                execution.active_cucco_movement = None;
                execution.active_cucco_x_publications = 0;
                execution.active_cucco_y_subpixel = None;
                execution.master_sword_light_beam_movement = None;
                execution.boulder_movement = None;
                execution.zora_fireball = None;
                execution.laser_eye_draw_prologue = None;
                execution.master_sword_light_beam_spawn = None;
                execution.cucco_helper_ordinal = 0;
                execution.big_key_drop_graphics_slot = None;
                execution.king_zora_flippers_graphics_slot = None;
                execution.happiness_pond_rupee_graphics_slot = None;
                execution.catfish_medallion_graphics_slot = None;
                execution.waterfall_gt_cutscene_graphics_slot = None;
                execution.bonk_item_graphics_slot = None;
                execution.single_small_draw_position_slot = None;
                execution.probe_after_oam_coordinates_slot = None;
                execution.wallmaster_reset_prefix_slot = None;
                execution.wallmaster_reset_cleared_bytes = None;
                execution.zazak_graphics_slot = None;
                if let Some((caller, tracker)) = execution.follower_graphics.take() {
                    if tracker.phase != RescuedMaidenInitializationTrackerPhase::Converting {
                        return Err(format!(
                            "Snes9x {caller:?} sprite slot {slot} returned from follower graphics in {:?}",
                            tracker.phase,
                        ));
                    }
                }
                // Slot zero is the final iteration of the descending C
                // loop. No caller-specific return address is needed to
                // prove that later NMIs are outside Sprite_Main.
                if slot == 0 {
                    self.sprite_main_execution = None;
                    receipts.push(OriginalTimingSemanticReceipt::SpriteMainReturned);
                }
            }
            SPRITE_MAIN_RETURN_PC => {
                // Slot zero closes the descending loop as soon as its
                // source call returns. The later common caller-return
                // marker is therefore idempotent for a complete loop,
                // while still closing early-return paths which never
                // entered all sixteen slots.
                if self.sprite_main_execution.take().is_some() {
                    receipts.push(OriginalTimingSemanticReceipt::SpriteMainReturned);
                }
            }
            MASTER_SWORD_LIGHT_BEAM_MOVEMENT_CALL_PC => {
                let execution = self
                    .sprite_main_execution
                    .as_mut()
                    .ok_or("Snes9x entered master-sword light-beam movement outside Sprite_Main")?;
                let slot = execution.current_slot.ok_or(
                    "Snes9x entered master-sword light-beam movement before a sprite slot",
                )?;
                if event.x != Some(u16::from(slot)) {
                    return Err(format!(
                        "Snes9x master-sword light-beam movement disagreed on slot {slot}: x={:?}",
                        event.x,
                    ));
                }
                if execution.master_sword_light_beam_movement.is_some() {
                    return Err(
                        "Snes9x restarted master-sword light-beam movement before its slot returned"
                            .to_string(),
                    );
                }
                execution.master_sword_light_beam_movement = Some((slot, 0));
            }
            ACTIVE_CUCCO_MOVEMENT_CALL_PC | CUCCO_FLEE_SUBTYPE_HELPER_CALL_PC
                if self.sprite_main_execution.is_none() && event.main == Some(0x1a) =>
            {
                // Module1A credits scenes call `SpriteActive_Main`
                // directly (route host 1573154); those Cucco helpers
                // publish no Sprite_Main receipts.
            }
            ACTIVE_CUCCO_MOVEMENT_CALL_PC => {
                let execution = self
                    .sprite_main_execution
                    .as_mut()
                    .ok_or("Snes9x entered active Cucco movement outside Sprite_Main")?;
                let slot = execution
                    .current_slot
                    .ok_or("Snes9x entered active Cucco movement before entering a slot")?;
                if event.x != Some(u16::from(slot)) {
                    return Err(format!(
                        "Snes9x active Cucco movement disagreed on slot {slot}: x={:?}",
                        event.x,
                    ));
                }
                if execution.cucco_subtype_increments.is_some()
                    || execution.cucco_flee_movement.is_some()
                    || execution.active_cucco_movement.is_some()
                    || execution.active_cucco_x_publications != 0
                    || execution.active_cucco_y_subpixel.is_some()
                {
                    return Err(
                        "Snes9x restarted active Cucco movement with unfinished work".to_string(),
                    );
                }
                if execution.cucco_animation_slot.take().is_some() {
                    execution.cucco_helper_ordinal = execution
                        .cucco_helper_ordinal
                        .checked_add(1)
                        .ok_or("Snes9x Cucco helper ordinal overflowed")?;
                }
                execution.active_cucco_movement = Some((slot, execution.cucco_helper_ordinal));
                execution.active_cucco_x_publications = 0;
            }
            CUCCO_FLEE_SUBTYPE_HELPER_CALL_PC => {
                let execution = self
                    .sprite_main_execution
                    .as_mut()
                    .ok_or("Snes9x completed Cucco flee movement outside Sprite_Main")?;
                let slot = execution
                    .current_slot
                    .ok_or("Snes9x completed Cucco flee movement before entering a slot")?;
                if event.x != Some(u16::from(slot)) {
                    return Err(format!(
                        "Snes9x Cucco flee movement disagreed on slot {slot}: x={:?}",
                        event.x,
                    ));
                }
                if execution.cucco_subtype_increments.is_some()
                    || execution.cucco_flee_movement.is_some()
                {
                    return Err(
                        "Snes9x entered Cucco flee helper with an unfinished helper".to_string()
                    );
                }
                if execution.cucco_animation_slot.take().is_some() {
                    execution.cucco_helper_ordinal = execution
                        .cucco_helper_ordinal
                        .checked_add(1)
                        .ok_or("Snes9x Cucco helper ordinal overflowed")?;
                }
                execution.active_cucco_y_subpixel = None;
                execution.active_cucco_x_publications = 0;
                execution.cucco_flee_movement = Some((slot, execution.cucco_helper_ordinal));
            }
            _ => {}
        }
        match pc {
            pc if NMI_HANDLER_COMPLETE_PCS.contains(&pc) => {
                if !self.nmi_publication_pending {
                    if !self.seed_warmup_active {
                        return Err(
                            "Snes9x reached NMI publication completion without an accepted NMI"
                                .to_string(),
                        );
                    }
                    // Seed warm-up: the NMI was accepted before the
                    // seeded oracle state was captured; its completion
                    // belongs to no host this decoder saw. Drop it.
                } else {
                    let update_gate = self
                        .pending_nmi_update_gate
                        .ok_or("Snes9x NMI completion lost its accepted update-gate disposition")?;
                    let joypad_publication =
                        if update_gate == NmiUpdateGate::Open {
                            Some(event.joypad_publication()?.ok_or(
                                "open Snes9x NMI completion omitted Zelda joypad publication",
                            )?)
                        } else {
                            None
                        };
                    self.pending_nmi_update_gate = None;
                    self.nmi_publication_pending = false;
                    receipts.push(OriginalTimingSemanticReceipt::NmiHandlerCompleted);
                    if let Some(publication) = joypad_publication {
                        receipts.push(OriginalTimingSemanticReceipt::JoypadPublication(
                            publication,
                        ));
                    }
                }
            }
            BOTTLE_VENDOR_ITEM_RECEIPT_CALL_PC | SICK_KID_ITEM_RECEIPT_CALL_PC => {
                let slot = u8::try_from(
                    event
                        .x
                        .ok_or("Snes9x Sprite_Main item call omitted sprite slot")?,
                )
                .map_err(|_| "Snes9x Sprite_Main sprite slot exceeded one byte")?;
                if slot >= 16 {
                    return Err(format!(
                        "Snes9x Sprite_Main item call used invalid sprite slot {slot}"
                    ));
                }
                let caller = ItemReceiptGraphicsCaller::SpriteMain { slot };
                if let Some(active) = self.item_receipt_caller.replace(caller) {
                    return Err(format!(
                        "Snes9x entered a second Sprite_Main item receipt in slot {slot} while {active:?} remained suspended"
                    ));
                }
            }
            BOTTLE_VENDOR_ITEM_RECEIPT_RETURN_PC | SICK_KID_ITEM_RECEIPT_RETURN_PC => {
                let caller = self.item_receipt_caller.take().ok_or(
                    "Snes9x returned from a Sprite_Main item receipt without an active caller",
                )?;
                if !matches!(caller, ItemReceiptGraphicsCaller::SpriteMain { .. }) {
                    return Err(format!(
                        "Snes9x returned from a Sprite_Main item receipt while {caller:?} remained active"
                    ));
                }
                receipts.push(OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                    ItemReceiptGraphicsProgressReceipt {
                        caller,
                        progress: SourceCallProgress::Returned,
                    },
                ));
            }
            LINK_RECEIVE_ITEM_ENTRY_PC => {
                // Caller-specific item paths install their tracker at
                // the outer C call site. Otherwise a direct item
                // pickup inside Sprite_Main owns the same synchronous
                // graphics boundary and the active source slot is its
                // semantic caller identity.
                if self.item_receipt_caller.is_none() {
                    if let Some(execution) = self.sprite_main_execution {
                        if let Some(slot) = execution.current_slot {
                            self.item_receipt_caller =
                                Some(ItemReceiptGraphicsCaller::SpriteMainDirect { slot });
                        } else if execution.last_completed_slot.is_none() {
                            // Sprite_Main's prefix (Ancilla_Main) ran
                            // the falling milestone item's receipt
                            // before the slot loop began (route host
                            // 1142850); X is the ancilla slot.
                            let slot = u8::try_from(
                                event
                                    .x
                                    .ok_or("Snes9x ancilla item receipt omitted its slot X")?,
                            )
                            .map_err(|_| "Snes9x ancilla item receipt slot exceeded one byte")?;
                            self.item_receipt_caller =
                                Some(ItemReceiptGraphicsCaller::SpriteMainAncilla { slot });
                        }
                    }
                }
            }
            LINK_RECEIVE_ITEM_GRAPHICS_RETURN_PC => {
                if matches!(
                    self.item_receipt_caller,
                    Some(
                        ItemReceiptGraphicsCaller::SpriteMainDirect { .. }
                            | ItemReceiptGraphicsCaller::SpriteMainAncilla { .. }
                    )
                ) {
                    let caller = self
                        .item_receipt_caller
                        .take()
                        .expect("direct Sprite_Main item receipt was matched above");
                    receipts.push(OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                        ItemReceiptGraphicsProgressReceipt {
                            caller,
                            progress: SourceCallProgress::Returned,
                        },
                    ));
                }
            }
            UNCLE_PASSAGE_ITEM_RECEIPT_CALL_PC => {
                let slot = u8::try_from(
                    event
                        .x
                        .ok_or("Snes9x Uncle item call omitted sprite slot X")?,
                )
                .map_err(|_| "Snes9x Uncle item-call slot exceeded one byte")?;
                if slot >= 16 {
                    return Err(format!(
                        "Snes9x Uncle item call used invalid sprite slot {slot}"
                    ));
                }
                let caller = ItemReceiptGraphicsCaller::UnclePassage { slot };
                if let Some(active) = self.item_receipt_caller.replace(caller) {
                    return Err(format!(
                        "Snes9x entered Uncle item receipt in slot {slot} while {active:?} remained suspended"
                    ));
                }
            }
            UNCLE_PASSAGE_ITEM_RECEIPT_RETURN_PC => {
                let caller = self
                    .item_receipt_caller
                    .take()
                    .ok_or("Snes9x returned from Uncle item receipt without an active caller")?;
                if !matches!(caller, ItemReceiptGraphicsCaller::UnclePassage { .. }) {
                    return Err(format!(
                        "Snes9x returned from Uncle item receipt while {caller:?} remained active"
                    ));
                }
                receipts.push(OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                    ItemReceiptGraphicsProgressReceipt {
                        caller,
                        progress: SourceCallProgress::Returned,
                    },
                ));
            }
            _ => {}
        }
        Ok(())
    }

    /// Extracted from `consume_event` (mechanical move; body unchanged).
    pub(super) fn consume_wram_write_event(
        &mut self,
        event: &RawTraceEvent,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    ) -> Result<(), String> {
        let pc = event.pc.ok_or("Snes9x WRAM write omitted PC")? & 0x00ff_ffff;
        let address = event.address.ok_or("Snes9x WRAM write omitted address")?;
        if pc == 0x1d_e5dd {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Vitreous minion cadence outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Vitreous minion cadence lost slot")?;
            if event.x != Some(u16::from(slot)) || address != 0x0e80 + u16::from(slot) {
                return Err("Vitreous cadence disagrees with source slot".into());
            }
            execution.vitreous_minions_seen = true;
        }
        if pc == 0x1d_9f88 {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Swamola head flags outside Sprite_Main")?;
            let slot = execution.current_slot.ok_or("Swamola head lost slot")?;
            if event.x != Some(u16::from(slot)) || address != 0x0f50 + u16::from(slot) {
                return Err("Swamola head flags disagree with source slot".into());
            }
            execution.swamola_head_prepared = true;
        }
        if address == 0x0fb6 && matches!(pc, 0x1d_9fd7 | 0x1d_a034) {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Swamola segment publication outside Sprite_Main")?;
            if execution.current_slot.map(u16::from) != event.x {
                return Err("Swamola segment slot disagrees with the active caller".into());
            }
            let segment = event
                .value
                .ok_or("Swamola segment publication omitted its value")?;
            if segment > 4 {
                return Err("Swamola segment exceeds its four-part body".into());
            }
            execution.swamola_segment = Some(segment);
            execution.swamola_head_prepared = false;
            execution.swamola_head_draw_completed = None;
            execution.swamola_head_draw = None;
            execution.swamola_segment_draw = None;
        }
        if pc == CREDITS_SCENE_OVERWORLD_SUBSUBMODULE_INCREMENT_PC
            && address == SUBSUBMODULE_INDEX
            && event.main == Some(0x1a)
        {
            receipts.retain(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(_)
                )
            });
            receipts.push(OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(
                CreditsSceneLoadProgressReceipt {
                    progress: CreditsSceneLoadProgress::SceneLoadCompleted,
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            ));
        }
        if let Some(progress) = dungeon_falling_entrance_progress(event, pc, address)? {
            receipts.push(OriginalTimingSemanticReceipt::DungeonFallingEntranceProgress(progress));
        }
        if pc == 0x0c_c25b
            && address == 0x11
            && event.return_address == Some(0x0c_f0e8)
            && event.main == Some(0x17)
        {
            if event.sub != Some(1) || event.value != Some(2) {
                return Err("save-quit intro-memory return has the wrong submodule advance".into());
            }
            receipts.push(OriginalTimingSemanticReceipt::SaveQuitIntroMemoryReturned);
        }
        if let Some(execution) = self.sprite_main_execution.as_mut() {
            execution.observe_guard_prep_weapon_flags_pending(event)?;
            execution.observe_buzzblob_movement(event)?;
            execution.observe_trinexx_head_draw(event)?;
            execution.observe_trinexx_breath_tile_collision(event)?;
            execution.observe_lanmola_draw_prefix(event)?;
            execution.observe_laser_eye_draw_prologue(event);
            execution.observe_sprite_handler_returned(event)?;
            execution.observe_trinexx_final_phase_tile_collision(event)?;
            execution.observe_helmasaur_hard_hat_tile_collision(event)?;
            execution.observe_trinexx_final_phase_draw(event)?;
            execution.observe_sidenexx_neck_target(event)?;
            execution.observe_guard_animation_checkpoint(event)?;
            execution.observe_hog_spear_body_graphics_pending(event)?;
            execution.observe_absorbable_tile_lookup(event)?;
            execution.observe_swamola_segment_draw(event)?;
            execution.observe_vitreous_damage_pending(event);
            execution.observe_moblin_collision_geometry(event);
            execution.observe_dispatch_trampoline_return(event)?;
            execution.observe_pengator_slide_pending(event)?;
            execution.observe_antifairy_bounce_pending(event)?;
            execution.observe_kholdstare_damage_pending(event)?;
            execution.observe_fire_debirando_spawn_write(event)?;
            execution.observe_trinexx_death_spawn_write(event)?;
            execution.observe_agahnim_motion_blur_spawn_write(event)?;
            execution.observe_master_sword_light_beam_spawn_write(event)?;
            execution.observe_antfairy_subtype2_increment(event)?;
            execution.observe_lanmola_subtype2_increment(event)?;
            execution.observe_helmasaur_hard_hat_beetle_subtype2_increment(event)?;
            execution.observe_zazak_graphics(event)?;
            if pc == THROWABLE_SCENERY_STATE_CLEAR_PC {
                let slot = execution
                    .current_slot
                    .ok_or("Snes9x cleared throwable scenery before entering a sprite slot")?;
                let value = event
                    .value
                    .ok_or("Snes9x throwable-scenery state clear omitted value")?;
                if event.x != Some(u16::from(slot))
                    || address != SPRITE_STATE_BASE + u16::from(slot)
                    || value != 0
                {
                    return Err(format!(
                        "Snes9x throwable-scenery state clear disagreed on slot {slot}: x={:?}, address=${address:04x}, value=${value:02x}",
                        event.x,
                    ));
                }
                execution.throwable_scenery_state_clear_slot = Some(slot);
            }
            if let Some((slot, helper_ordinal)) = execution.active_cucco_movement {
                let expected_x_address = match execution.active_cucco_x_publications {
                    0 => Some(SPRITE_X_SUBPIXEL_BASE + u16::from(slot)),
                    1 => Some(SPRITE_X_LOW_BASE + u16::from(slot)),
                    2 => Some(SPRITE_X_HIGH_BASE + u16::from(slot)),
                    3 => None,
                    count => {
                        return Err(format!(
                            "Snes9x active Cucco published invalid X field count {count}",
                        ));
                    }
                };
                if expected_x_address == Some(address) {
                    execution.active_cucco_x_publications += 1;
                } else if [
                    SPRITE_X_SUBPIXEL_BASE + u16::from(slot),
                    SPRITE_X_LOW_BASE + u16::from(slot),
                    SPRITE_X_HIGH_BASE + u16::from(slot),
                ]
                .contains(&address)
                {
                    return Err(format!(
                        "Snes9x active Cucco X publications were out of source order at ${address:04x}",
                    ));
                }
                if address == SPRITE_Y_SUBPIXEL_BASE + u16::from(slot) {
                    if execution.active_cucco_x_publications != 3 {
                        return Err(
                            "Snes9x published active Cucco Y before Sprite_MoveX returned"
                                .to_string(),
                        );
                    }
                    if execution.active_cucco_y_subpixel.is_some() {
                        return Err("Snes9x published active Cucco Y subpixel twice".to_string());
                    }
                    execution.active_cucco_y_subpixel = Some((slot, helper_ordinal));
                }
            }
            // LaserEye_Draw ($1E:A708) stores the object priority
            // right before its Sprite_DrawMultiple; any store from
            // outside the draw helpers means the handler moved on.
            if execution.laser_eye_draw_prologue.is_some() && !laser_eye_draw_in_flight_pc(pc) {
                execution.laser_eye_draw_prologue = None;
            }
            if pc == 0x1e_a71f {
                let slot = execution
                    .current_slot
                    .ok_or("Snes9x published a laser eye's object priority before a sprite slot")?;
                if event.x != Some(u16::from(slot))
                    || address != SPRITE_OBJECT_PRIORITY_BASE + u16::from(slot)
                {
                    return Err(format!(
                        "Snes9x laser eye object-priority publication disagreed on slot {slot}: x={:?}, address=${address:04x}",
                        event.x,
                    ));
                }
                execution.laser_eye_draw_prologue = Some(slot);
            }
            // Sprite_Fireball ($05:9683) stores sprite_ignore_projectile
            // first; its bank-$05 Sprite_MoveXY copy stores y_high at
            // $05:FA2D last.
            if pc == 0x05_9686 {
                let slot = execution
                    .current_slot
                    .ok_or("Snes9x published a fireball's projectile flag before a sprite slot")?;
                if event.x != Some(u16::from(slot)) || address != 0x0ba0 + u16::from(slot) {
                    return Err(format!(
                        "Snes9x fireball projectile-flag publication disagreed on slot {slot}: x={:?}, address=${address:04x}",
                        event.x,
                    ));
                }
                execution.zora_fireball = Some((slot, 0, false));
            }
            if let Some((slot, checkpoint_ordinal, _)) = execution.zora_fireball.as_mut() {
                let slot = *slot;
                let expected = [
                    SPRITE_X_SUBPIXEL_BASE,
                    SPRITE_X_LOW_BASE,
                    SPRITE_X_HIGH_BASE,
                    SPRITE_Y_SUBPIXEL_BASE,
                    SPRITE_Y_LOW_BASE,
                    SPRITE_Y_HIGH_BASE,
                ];
                let movement_addresses = expected.map(|base| base + u16::from(slot));
                // The bank-$05 Sprite_MoveX copy indexes the low byte
                // with X = slot + 16, so match on address alone.
                if let Some(index) = movement_addresses
                    .iter()
                    .position(|&candidate| candidate == address)
                {
                    let next_ordinal = match (*checkpoint_ordinal, index) {
                        (0, 0) => 1,
                        (1, 1) => 2,
                        (2, 2) => 3,
                        (0 | 3, 3) => 4,
                        (4, 4) => 5,
                        (5, 5) => 6,
                        _ => {
                            return Err(format!(
                                "Snes9x Zora fireball movement stores were out of source order: checkpoint={} address=${address:04x}",
                                *checkpoint_ordinal,
                            ));
                        }
                    };
                    *checkpoint_ordinal = next_ordinal;
                }
                if address == SPRITE_STATE_BASE + u16::from(slot) && event.value == Some(0) {
                    // The fireball killed itself; nothing of its
                    // body remains to checkpoint.
                    execution.zora_fireball = None;
                }
            }
            // Sprite_C2_Boulder (indoors) stores its OAM flags right
            // before Sprite_MoveXYZ; the Z result and the six XY
            // coordinate stores then follow in source order.
            if pc == 0x1d_cfe8 {
                let slot = execution
                    .current_slot
                    .ok_or("Snes9x published Boulder OAM flags before a sprite slot")?;
                if event.x != Some(u16::from(slot))
                    || address != SPRITE_OAM_FLAGS_BASE + u16::from(slot)
                {
                    return Err(format!(
                        "Snes9x Boulder OAM flags publication disagreed on slot {slot}: x={:?}, address=${address:04x}",
                        event.x,
                    ));
                }
                execution.boulder_movement = Some((slot, 0, false));
            }
            if let Some((slot, checkpoint_ordinal, z_done)) = execution.boulder_movement.as_mut() {
                let slot = *slot;
                if address == SPRITE_Z_BASE + u16::from(slot) && event.x == Some(u16::from(slot)) {
                    *z_done = true;
                }
                let expected = [
                    SPRITE_X_SUBPIXEL_BASE,
                    SPRITE_X_LOW_BASE,
                    SPRITE_X_HIGH_BASE,
                    SPRITE_Y_SUBPIXEL_BASE,
                    SPRITE_Y_LOW_BASE,
                    SPRITE_Y_HIGH_BASE,
                ];
                let movement_addresses = expected.map(|base| base + u16::from(slot));
                if let Some(index) = movement_addresses
                    .iter()
                    .position(|&candidate| candidate == address)
                {
                    let next_ordinal = match (*checkpoint_ordinal, index) {
                        (0, 0) => 1,
                        (1, 1) => 2,
                        (2, 2) => 3,
                        (0 | 3, 3) => 4,
                        (4, 4) => 5,
                        (5, 5) => 6,
                        _ => {
                            return Err(format!(
                                "Snes9x Boulder movement stores were out of source order: checkpoint={} address=${address:04x}",
                                *checkpoint_ordinal,
                            ));
                        }
                    };
                    *checkpoint_ordinal = next_ordinal;
                }
            }
            if let Some((slot, checkpoint_ordinal)) =
                execution.master_sword_light_beam_movement.as_mut()
            {
                let slot = *slot;
                let expected = [
                    SPRITE_X_SUBPIXEL_BASE,
                    SPRITE_X_LOW_BASE,
                    SPRITE_X_HIGH_BASE,
                    SPRITE_Y_SUBPIXEL_BASE,
                    SPRITE_Y_LOW_BASE,
                    SPRITE_Y_HIGH_BASE,
                ];
                let movement_addresses = expected.map(|base| base + u16::from(slot));
                if let Some(index) = movement_addresses
                    .iter()
                    .position(|&candidate| candidate == address)
                {
                    // `Sprite_MoveX` and `Sprite_MoveY` each return
                    // without publishing any coordinate assignment
                    // when that axis' velocity is zero. The first
                    // observed Y store can therefore legitimately
                    // follow the call site with no X stores. Preserve
                    // the source checkpoint reached, rather than
                    // counting only writes that happened to execute.
                    let next_ordinal = match (*checkpoint_ordinal, index) {
                        (0, 0) => 1,
                        (1, 1) => 2,
                        (2, 2) => 3,
                        (0 | 3, 3) => 4,
                        (4, 4) => 5,
                        (5, 5) => 6,
                        _ => {
                            return Err(format!(
                                "Snes9x master-sword light-beam movement stores were out of source order: checkpoint={} address=${address:04x}",
                                *checkpoint_ordinal,
                            ));
                        }
                    };
                    *checkpoint_ordinal = next_ordinal;
                }
            }
        }
        let credits_direct_sprite_call =
            self.sprite_main_execution.is_none() && event.main == Some(0x1a);
        if let Some(increment_index) = CUCCO_SUBTYPE_INCREMENT_PUBLICATION_PCS
            .iter()
            .position(|&increment_pc| pc == increment_pc)
            .filter(|_| !credits_direct_sprite_call)
        {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Snes9x published a Cucco subtype increment outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Snes9x published a Cucco subtype increment before entering a slot")?;
            if event.x != Some(u16::from(slot)) || address != SPRITE_SUBTYPE2_BASE + u16::from(slot)
            {
                return Err(format!(
                    "Snes9x Cucco subtype publication disagreed on slot {slot}: x={:?}, address=${address:04x}",
                    event.x,
                ));
            }
            if increment_index == 0 {
                if execution.cucco_subtype_increments.is_some() {
                    return Err(
                        "Snes9x restarted a Cucco helper before publishing graphics".to_string()
                    );
                }
                if execution.cucco_animation_slot.take().is_some() {
                    execution.cucco_helper_ordinal = execution
                        .cucco_helper_ordinal
                        .checked_add(1)
                        .ok_or("Snes9x Cucco helper ordinal overflowed")?;
                }
                execution.cucco_flee_movement = None;
                execution.active_cucco_movement = None;
                execution.active_cucco_x_publications = 0;
                execution.active_cucco_y_subpixel = None;
            }
            let completed = u8::try_from(increment_index + 1)
                .expect("the Cucco increment table has five entries");
            execution.cucco_subtype_increments =
                Some((slot, execution.cucco_helper_ordinal, completed));
        }
        if pc == CUCCO_ANIMATION_PUBLICATION_PC && !credits_direct_sprite_call {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Snes9x published Cucco animation outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Snes9x published Cucco animation before entering a slot")?;
            if event.x != Some(u16::from(slot)) || address != SPRITE_GRAPHICS_BASE + u16::from(slot)
            {
                return Err(format!(
                    "Snes9x Cucco animation publication disagreed on slot {slot}: x={:?}, address=${address:04x}",
                    event.x,
                ));
            }
            execution.cucco_subtype_increments = None;
            execution.cucco_flee_movement = None;
            execution.active_cucco_movement = None;
            execution.active_cucco_x_publications = 0;
            execution.active_cucco_y_subpixel = None;
            execution.cucco_animation_slot = Some((slot, execution.cucco_helper_ordinal));
        }
        if pc == BIG_KEY_DROP_TYPE_PUBLICATION_PC {
            let value = event
                .value
                .ok_or("Snes9x enemy-drop type publication omitted value")?;
            // This is the shared `sprite_type[k] = item` statement.
            // Only `$e5` takes the following big-key graphics branch;
            // ordinary prize/key drops remain outside this domain.
            if value == BIG_KEY_DROP_SPRITE_TYPE {
                let execution = self
                    .sprite_main_execution
                    .as_mut()
                    .ok_or("Snes9x entered big-key graphics outside Sprite_Main")?;
                let slot = execution
                    .current_slot
                    .ok_or("Snes9x entered big-key graphics before entering a slot")?;
                if event.x != Some(u16::from(slot)) || address != SPRITE_TYPE_BASE + u16::from(slot)
                {
                    return Err(format!(
                        "Snes9x big-key type publication disagreed on slot {slot}: x={:?}, address=${address:04x}, value=${value:02x}",
                        event.x,
                    ));
                }
                execution.big_key_drop_graphics_slot = Some(slot);
            }
        }
        if pc == SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC {
            let execution = self
                .sprite_main_execution
                .as_mut()
                .ok_or("Snes9x converted Fire Debirando outside Sprite_Main")?;
            let slot = execution
                .current_slot
                .ok_or("Snes9x converted Fire Debirando before entering a slot")?;
            if execution.timers_and_oam_dispatch_state != Some(8)
                || event.x != Some(u16::from(slot))
                || address != SPRITE_TYPE_BASE + u16::from(slot)
                || event.value != Some(0x63)
            {
                return Err(format!(
                    "Snes9x Fire Debirando conversion disagreed on slot {slot}: dispatch={:?}, x={:?}, address=${address:04x}, value={:?}",
                    execution.timers_and_oam_dispatch_state,
                    event.x,
                    event.value,
                ));
            }
            execution.fire_debirando_property_reload = true;
            // Any checkpoint from the first property reset is now
            // superseded by the later source call.
            execution.initialize_reset_properties = None;
            execution.initialize_load_properties = None;
        }
        self.observe_overworld_sprite_publication(event, pc, address, receipts)?;
        let disable_progress = sprite_disable_progress(pc, address, event.value)?;
        // `SpritesDisabled` is a candidate for a host boundary at the
        // final Sprite_DisableAll write, not a durable description of
        // the rest of Dungeon_ResetSprites.  Once any later source
        // write is observed, execution has advanced beyond that exact
        // statement.  Drop the candidate fail-closed; a more precise
        // cache/load receipt below may replace it.
        if matches!(
            self.pending_reset_progress,
            Some(DungeonResetSpritesCpuProgress::SpritesDisabled)
        ) && !(pc == SPRITE_DISABLE_ALL_FINAL_GARNISH_PC
            && event.x == Some(0)
            && address == GARNISH_TYPE_SLOT_ZERO)
        {
            self.pending_reset_progress = None;
        }
        if matches!(
            self.pending_reset_progress,
            Some(DungeonResetSpritesCpuProgress::Disable(_))
        ) && disable_progress.is_none()
        {
            self.pending_reset_progress = None;
        }
        let cached_sprite_write = (UNCACHE_SPRITE_START_PC..UNCACHE_SPRITE_END_PC)
            .contains(&pc)
            .then(|| cached_sprite_live_field(address))
            .flatten();
        if pc == ANTFAIRY_SUBTYPE2_INCREMENT_PC {
            if let Some(progress) = self.cached_sprite_execution.as_mut() {
                if progress.restore_started
                    || usize::from(progress.copied_fields) != CACHED_SPRITE_LIVE_FIELDS.len()
                    || event.x != Some(u16::from(progress.slot))
                    || address != SPRITE_SUBTYPE2_BASE + u16::from(progress.slot)
                {
                    return Err(format!(
                        "Snes9x cached Antfairy subtype publication disagreed with the live-slot swap: tracker={progress:?}, x={:?}, address=${address:04x}",
                        event.x,
                    ));
                }
                if progress
                    .body_progress
                    .replace(CachedSpriteExecutionBodyProgress::AfterAntfairySubtype2Increment)
                    .is_some()
                {
                    return Err(
                        "Snes9x cached Antfairy published its subtype increment twice".to_string(),
                    );
                }
            }
        }
        if let Some((field_index, slot)) = cached_sprite_write {
            if let Some(progress) = self.cached_sprite_execution.as_mut() {
                if progress.observe_write(pc, slot, field_index)? {
                    self.cached_sprite_execution = None;
                }
            } else {
                self.cached_sprite_execution = Some(
                    CachedSpriteExecutionTracker::from_observed_write(pc, slot, field_index),
                );
            }
        } else if let Some(progress) = disable_progress {
            self.cache_write_progress = None;
            self.normal_load_ordinal = None;
            self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::Disable(progress));
        } else if pc == SPRITE_DISABLE_ALL_FINAL_GARNISH_PC
            && event.x == Some(0)
            && address == GARNISH_TYPE_SLOT_ZERO
        {
            self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::SpritesDisabled);
        } else if (DUNGEON_CACHE_TRANS_SPRITES_START_PC..DUNGEON_CACHE_TRANS_SPRITES_END_PC)
            .contains(&pc)
            && CACHE_FIELD_WRITES
                .iter()
                .any(|&(_, base)| (base..base + 16).contains(&address))
        {
            let slot = u8::try_from(
                event
                    .x
                    .ok_or("Snes9x Dungeon_CacheTransSprites write omitted X")?,
            )
            .map_err(|_| "Snes9x Dungeon_CacheTransSprites X exceeded one byte")?;
            if slot >= 16 {
                return Err(format!(
                    "Snes9x Dungeon_CacheTransSprites slot {slot} is outside 0..16"
                ));
            }
            let progress = match self.cache_write_progress {
                Some(progress) if progress.slot == slot => progress,
                Some(progress) if slot < progress.slot => CacheWriteProgress {
                    slot,
                    next_field_index: 0,
                },
                // A completed call may have no later traced reset
                // write (Sprite_DisableAll stores only active slots).
                // The next source call is nevertheless unambiguous:
                // its descending C loop begins with slot 15's
                // StateClear publication.
                Some(_) if slot == 15 && address == 0x1d0f => CacheWriteProgress {
                    slot,
                    next_field_index: 0,
                },
                Some(progress) => {
                    return Err(format!(
                        "Snes9x Dungeon_CacheTransSprites slot order advanced from {} to {slot}",
                        progress.slot
                    ));
                }
                None => CacheWriteProgress {
                    slot,
                    next_field_index: 0,
                },
            };
            let &(field, base) = CACHE_FIELD_WRITES
                .get(progress.next_field_index)
                .ok_or("Snes9x Dungeon_CacheTransSprites wrote past the final field")?;
            let expected_address = base + u16::from(slot);
            if address != expected_address {
                return Err(format!(
                    "Snes9x Dungeon_CacheTransSprites field {field:?} for slot {slot} expected ${expected_address:04x}, observed ${address:04x}"
                ));
            }
            self.cache_write_progress = Some(CacheWriteProgress {
                slot,
                next_field_index: progress.next_field_index + 1,
            });
            self.pending_reset_progress =
                Some(DungeonResetSpritesCpuProgress::Cache { slot, field });
        } else if pc == DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC
            && (SPRITE_STATE_BASE..SPRITE_STATE_BASE + 16).contains(&address)
        {
            let slot = (address - SPRITE_STATE_BASE) as u8;
            if event.x != Some(u16::from(slot)) {
                return Err(format!(
                    "Snes9x Dungeon_LoadSingleSprite state write disagrees on slot: x={:?}, address=${address:04x}",
                    event.x
                ));
            }
            self.normal_load_ordinal = Some(
                self.normal_load_ordinal
                    .map(|ordinal| ordinal.saturating_add(1))
                    .unwrap_or(0),
            );
            self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::Load(
                DungeonLoadSpritesCpuProgress {
                    normal_load_ordinal: self.normal_load_ordinal.unwrap(),
                    slot,
                    checkpoint: DungeonSpriteLoadCheckpoint::State,
                },
            ));
        } else if (DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC..DUNGEON_LOAD_SINGLE_SPRITE_END_PC)
            .contains(&pc)
        {
            let Some((slot, checkpoint)) =
                dungeon_load_single_sprite_write_progress(pc, address, event.x)?
            else {
                return Err(format!(
                    "Snes9x Dungeon_LoadSingleSprite wrote unsupported source field ${address:04x} at ${pc:06x}",
                ));
            };
            let normal_load_ordinal = self
                .normal_load_ordinal
                .ok_or("Snes9x observed Dungeon_LoadSingleSprite field before record state")?;
            self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::Load(
                DungeonLoadSpritesCpuProgress {
                    normal_load_ordinal,
                    slot,
                    checkpoint,
                },
            ));
        }
        Ok(())
    }

    /// Extracted from `consume_event` (mechanical move; body unchanged).
    pub(super) fn consume_nmi_event(
        &mut self,
        event: &RawTraceEvent,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    ) -> Result<(), String> {
        if event.pc == Some(0x02_dc76) && matches!((event.main, event.sub), (Some(5), Some(0))) {
            receipts.push(OriginalTimingSemanticReceipt::SelectedGameEntranceScrollPublished);
        }
        // An NMI accepted inside Dungeon_PushBlock_Handler's loop names
        // the misc object the resumed handler continues from.
        if let Some(next_index) = dungeon_push_blocks_in_progress(event) {
            receipts
                .push(OriginalTimingSemanticReceipt::DungeonPushBlocksInProgress { next_index });
        }
        if dungeon_push_blocks_handled(event) {
            receipts.push(OriginalTimingSemanticReceipt::DungeonPushBlocksHandled);
        }
        if let Some(progress) =
            credits_scene_load_boundary_progress(event, OriginalTimingBoundary::NmiAccepted)?
        {
            receipts.retain(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(_)
                )
            });
            receipts.push(OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(
                progress,
            ));
        }
        if let Some(progress) =
            credits_end_sequence_32_boundary_progress(event, OriginalTimingBoundary::NmiAccepted)?
        {
            receipts.retain(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::CreditsEndSequence32Progress(_)
                )
            });
            receipts.push(OriginalTimingSemanticReceipt::CreditsEndSequence32Progress(
                progress,
            ));
        }
        if let Some(progress) =
            triforce_room_case2_palette_progress(event, OriginalTimingBoundary::NmiAccepted)?
        {
            receipts.retain(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::TriforceRoomCase2PaletteProgress(_)
                )
            });
            receipts
                .push(OriginalTimingSemanticReceipt::TriforceRoomCase2PaletteProgress(progress));
        }
        if let Some(progress) =
            dungeon_peg_attribute_flip_progress(event, OriginalTimingBoundary::NmiAccepted)?
        {
            receipts.retain(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(_)
                )
            });
            receipts.push(OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(progress));
        }
        if let Some(tracker) = self.rescued_maiden_initialization.as_mut() {
            if (event.main, event.sub, event.subsub) != (Some(7), Some(0x18), Some(10)) {
                return Err(format!(
                    "Snes9x rescued-maiden decompressor escaped its source domain: main={:?}, sub={:?}, subsub={:?}",
                    event.main, event.sub, event.subsub,
                ));
            }
            tracker.observe_boundary(event)?;
        }
        if let Some(execution) = self.sprite_main_execution.as_mut() {
            if let Some((_, tracker)) = execution.follower_graphics.as_mut() {
                tracker.observe_boundary(event)?;
            }
        }
        if let Some(execution) = self.sprite_main_execution.as_mut() {
            execution.observe_guard_prep_weapon_flags_pending(event)?;
            execution.observe_buzzblob_movement(event)?;
            execution.observe_trinexx_head_draw(event)?;
            execution.observe_trinexx_breath_tile_collision(event)?;
            execution.observe_lanmola_draw_prefix(event)?;
            execution.observe_laser_eye_draw_prologue(event);
            execution.observe_sprite_handler_returned(event)?;
            execution.observe_trinexx_final_phase_tile_collision(event)?;
            execution.observe_helmasaur_hard_hat_tile_collision(event)?;
            execution.observe_trinexx_final_phase_draw(event)?;
            execution.observe_sidenexx_neck_target(event)?;
            execution.observe_guard_animation_checkpoint(event)?;
            execution.observe_hog_spear_body_graphics_pending(event)?;
            execution.observe_absorbable_tile_lookup(event)?;
            execution.observe_swamola_segment_draw(event)?;
            execution.observe_vitreous_damage_pending(event);
            execution.observe_moblin_collision_geometry(event);
            execution.observe_dispatch_trampoline_return(event)?;
            execution.observe_pengator_slide_pending(event)?;
            execution.observe_antifairy_bounce_pending(event)?;
            execution.observe_kholdstare_damage_pending(event)?;
            execution.observe_fire_debirando_spawn_boundary(event)?;
            execution.observe_trinexx_death_spawn_boundary(event)?;
            execution.observe_agahnim_motion_blur_spawn_boundary(event)?;
            execution.observe_guard_prep_parry_hitbox(event)?;
            execution.observe_guard_prep_patrol_delay(event)?;
            execution.observe_guard_prep_tile_collision_return(event)?;
            execution.observe_bari_before_random(event)?;
            execution.observe_main_and_aux1_timer_decrements(event)?;
            execution.observe_main_timer_decrement(event)?;
            execution.observe_zero_hit_timer_clear(event)?;
            execution.observe_primary_timer_decrements(event)?;
            execution.observe_hit_timer(event)?;
            execution.observe_timer_decrements(event)?;
            execution.observe_single_small_draw_position(event)?;
            execution.observe_probe_after_oam_coordinates(event)?;
            execution.observe_initialize_reset_properties(event)?;
            execution.observe_initialize_load_properties(event)?;
            execution.observe_initialize_prep_pending(event)?;
            execution.observe_initialize_prep_move_y(event)?;
            execution.observe_fire_debirando_before_spawn(event)?;
            execution.observe_zazak_graphics(event)?;
            execution.observe_wallmaster_reset_prefix(event)?;
        }
        let ppu_register_operands = event.nmi_ppu_register_operands()?;
        let target = nmi_resume_target(event)?;
        let update_gate = match event
            .nmi_latch
            .ok_or("Snes9x NMI receipt omitted Zelda's software update latch")?
        {
            0 => NmiUpdateGate::Open,
            _ => NmiUpdateGate::LatchHeld,
        };
        // Validate the source-stage cursor before changing any
        // cross-host NMI ownership or publishing partial receipts.
        let main_loop_interruption = main_loop_interruption_for_event(event)?;
        if matches!(
            main_loop_interruption,
            Some(MainLoopInterruption::SpritePreparationExtendedOamPacking { .. })
        ) && update_gate != NmiUpdateGate::LatchHeld
        {
            return Err(
                "Snes9x extended-OAM packing interruption observed an open Zelda NMI latch"
                    .to_string(),
            );
        }
        self.nmi_publication_pending = true;
        self.pending_nmi_update_gate = Some(update_gate);
        // An NMI that interrupts Zelda's poly thread (stack page $1F)
        // resumes through the IRQ-driven thread switch before the next
        // NMI while that thread lives. When the main thread retires
        // the thread instead (the intro hands off to module 1 while
        // the poly context is parked; route runs 965..967 accept
        // `$09:FE65/S=$1F34` and then `$00:E304/S=$01F8` with no
        // resume between), the parked context is never resumed. The
        // next acceptance on the main stack proves the abandonment;
        // retire the dead target instead of carrying it for the rest
        // of the route, where its presence disabled every fine
        // Sprite_Main observer on the host-return event (f1371214).
        if !is_poly_thread_stack(target.1) {
            while self
                .nmi_resume_targets
                .last()
                .is_some_and(|&(_, stack)| is_poly_thread_stack(stack))
            {
                self.nmi_resume_targets.pop();
            }
        }
        self.nmi_resume_targets.push(target);
        self.host_nmi_ppu_register_operands
            .push(ppu_register_operands);
        if publish_pre_dungeon_sprite_reset_progress(
            event,
            OriginalTimingBoundary::NmiAccepted,
            self.overworld_load_overlays_sprite_reload_active
                && !self.overworld_sprite_reload_reset_published,
            receipts,
        )? {
            // `Sprite_DisableAll` is shared by `Sprite_ResetAll` and
            // `Dungeon_ResetSprites`. The interrupted PC and the
            // innermost source return address prove this execution is
            // the former, so the generic reset candidate must not
            // escape into the wrong semantic domain below.
            self.pending_reset_progress = None;
            if self.overworld_load_overlays_sprite_reload_active {
                self.overworld_sprite_reload_reset_published = true;
            }
        } else if let Some(progress) = dungeon_reset_sprites_caller_progress(event) {
            self.pending_reset_progress = Some(progress);
        }
        let overworld_sprite_scan_suspended =
            self.publish_overworld_presence_at_scan_boundary(event, receipts);
        if overworld_sprite_scan_suspended {
            receipts.push(
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::ProximityScanSuspended {
                        bg2_h: event.bg2_h.ok_or(
                            "Snes9x overworld scan NMI omitted the BG2 scratch coordinate",
                        )?,
                    },
                ),
            );
        }
        self.flush_host_boundary_progress(receipts, OriginalTimingBoundary::NmiAccepted);
        receipts.push(OriginalTimingSemanticReceipt::NmiAccepted(update_gate));
        if event.pc == Some(0x00_e9bc) && event.return_address == Some(0x02_8ea4) {
            if (event.main, event.sub, event.subsub) != (Some(7), Some(7), Some(15)) {
                return Err(
                    "falling fade-in palette checkpoint has the wrong source caller".into(),
                );
            }
            receipts
                .push(OriginalTimingSemanticReceipt::DungeonFallingFadeInPaletteDirectionToggled);
        }
        if let Some(progress) =
            rescued_maiden_tilemap_clear_progress(event, OriginalTimingBoundary::NmiAccepted)?
        {
            receipts
                .push(OriginalTimingSemanticReceipt::RescuedMaidenTilemapClearProgress(progress));
        }
        if let Some(execution) = self.sprite_main_execution {
            let interruption = match self.item_receipt_caller {
                Some(
                    ItemReceiptGraphicsCaller::SpriteMain { slot }
                    | ItemReceiptGraphicsCaller::UnclePassage { slot },
                ) => {
                    if execution.current_slot != Some(slot) {
                        return Err(format!(
                            "Snes9x item-receipt caller slot {slot} disagreed with active Sprite_Main slot {:?}",
                            execution.current_slot,
                        ));
                    }
                    MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(slot)
                }
                _ => execution.interruption(),
            };
            receipts.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(
                interruption,
            ));
        }
        let progress_requires_return_scratch = spotlight_receipt_domain(event)
            && event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
                pc == IRIS_SPOTLIGHT_ITERATION_VALUE_STORE_PC
                    || pc == IRIS_SPOTLIGHT_NEXT_ITERATION_PC
                    || (IRIS_SPOTLIGHT_CIRCLE_VALUE_START_PC..IRIS_SPOTLIGHT_CIRCLE_VALUE_END_PC)
                        .contains(&pc)
            });
        if progress_requires_return_scratch {
            if self
                .pending_spotlight_helper_nmi
                .replace(event.clone())
                .is_some()
            {
                return Err(
                    "Snes9x host call accepted multiple NMIs inside the spotlight helper"
                        .to_string(),
                );
            }
            self.pending_spotlight_helper_nmi_acceptance_index =
                receipts.iter().rposition(|receipt| {
                    matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_))
                });
        } else if let Some(progress) = spotlight_table_build_progress(event, None, None)? {
            receipts.push(OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress,
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ));
        }
        if let Some(phase) = main_loop_interruption {
            receipts.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(phase));
        }
        if let Some(progress) = event
            .pc
            .and_then(|pc| link_oam_stair_progress(pc, event.sub))
        {
            receipts.push(OriginalTimingSemanticReceipt::LinkOamStairProgress(
                progress,
            ));
        }
        Ok(())
    }
}
