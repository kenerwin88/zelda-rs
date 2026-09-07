//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (actors).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub(crate) fn player_state_mut(&mut self) -> RamPlayerStateViewMut<'_> {
        RamPlayerStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn set_sound_effect_1_with_link_pan(&mut self, effect: u8) {
        let sound_effect = self.link_calculate_sfx_pan() | effect;
        self.set_sound_effect_1(sound_effect);
    }

    pub(crate) fn set_sound_effect_2_with_link_pan(&mut self, effect: u8) {
        let sound_effect = self.link_calculate_sfx_pan() | effect;
        self.set_sound_effect_2(sound_effect);
    }

    pub(crate) fn set_sound_effect_1_with_ancilla_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.ancilla_calculate_sfx_pan(slot) | effect;
        self.set_sound_effect_1(sound_effect);
    }

    pub(crate) fn set_sound_effect_2_with_ancilla_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.ancilla_calculate_sfx_pan(slot) | effect;
        self.set_sound_effect_2(sound_effect);
    }

    pub(crate) fn set_sound_effect_1_with_sprite_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.sprite_calculate_sfx_pan(slot) | effect;
        self.set_sound_effect_1(sound_effect);
    }

    pub(crate) fn set_sound_effect_2_with_sprite_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.sprite_calculate_sfx_pan(slot) | effect;
        self.set_sound_effect_2(sound_effect);
    }

    pub(crate) fn set_current_area_of_player_word(&mut self, value: u16) {
        self.world_region_mut()
            .set_current_area_of_player_word(value);
    }

    pub(crate) fn set_sprite_dma_head_pointer(&mut self, value: u8) {
        self.display_core_mut().set_sprite_dma_head_pointer(value);
    }

    pub(crate) fn set_sprite_dma_body_pointer(&mut self, value: u8) {
        self.display_core_mut().set_sprite_dma_body_pointer(value);
    }

    pub(crate) fn set_link_body_dma_sources(&mut self, top: u16, bottom: u16) {
        self.display_core_mut()
            .set_link_body_dma_sources(top, bottom);
    }

    pub(crate) fn set_link_head_dma_sources(&mut self, top: u16, bottom: u16) {
        self.display_core_mut()
            .set_link_head_dma_sources(top, bottom);
    }

    pub(crate) fn set_link_hand_dma_sources(&mut self, left: u16, right: u16) {
        self.display_core_mut()
            .set_link_hand_dma_sources(left, right);
    }

    pub(crate) fn set_link_sword_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_sword_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_shield_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_shield_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_aux_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_aux_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_push_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_push_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_animated_tile_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_animated_tile_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_head_pointer_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_head_pointer_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_body_pointer_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_body_pointer_dma_sources(upper, lower);
    }

    pub(crate) fn alt_sprite_slot_mut(&mut self, slot: usize) -> NativeCachedSpriteBridgeMut<'_> {
        NativeCachedSpriteBridgeMut::new(
            &mut self.game_state.sprites.cached_sprites,
            &mut self.game_state.sprites.sprite_slots,
            &mut self.game_state.sprites.system,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn tagalong_slot(&self, slot: usize) -> TagalongSlotRead<'_> {
        TagalongSlotRead::new(&self.game_state.sprites.tagalong_trail, slot)
    }

    pub(crate) fn tagalong_slot_mut(&mut self, slot: usize) -> NativeTagalongSlotBridgeMut<'_> {
        NativeTagalongSlotBridgeMut::new(
            &mut self.game_state.sprites.tagalong_trail,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn ancilla_spawn_scratch_mut(
        &mut self,
    ) -> NativeFailedSpinSparkleSpawnBridgeMut<'_> {
        NativeFailedSpinSparkleSpawnBridgeMut::new(
            &mut self.game_state.sprites.failed_spin_sparkle_spawn,
            &mut self.ram,
        )
    }

    pub(crate) fn sprite_slot_view(&self, slot: usize) -> NativeSpriteSlotView<'_> {
        self.game_state.sprites.sprite_slots.slot(slot)
    }

    pub(crate) fn sprite_slot_view_mut(&mut self, slot: usize) -> NativeSpriteSlotBridgeMut<'_> {
        self.game_state
            .sprites
            .sprite_slots
            .slot_mut(&mut self.ram, slot)
    }

    pub(crate) fn overlord_slot_view(&self, slot: usize) -> NativeOverlordSlotView<'_> {
        self.game_state.sprites.overlord_slots.slot(slot)
    }

    pub(crate) fn overlord_slot_view_mut(
        &mut self,
        slot: usize,
    ) -> NativeOverlordSlotBridgeMut<'_> {
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, slot)
    }

    pub(crate) fn ancilla_slot_view(&self, slot: usize) -> NativeAncillaSlotView<'_> {
        self.game_state.sprites.ancilla_slots.slot(slot)
    }

    pub(crate) fn ancilla_slot_view_mut(&mut self, slot: usize) -> NativeAncillaSlotBridgeMut<'_> {
        self.game_state
            .sprites
            .ancilla_slots
            .slot_mut(&mut self.ram, slot)
    }

    pub(crate) fn garnish_slot_view(&self, slot: usize) -> NativeGarnishSlotView<'_> {
        self.game_state.sprites.garnish_slots.slot(slot)
    }

    pub(crate) fn garnish_slot_view_mut(&mut self, slot: usize) -> NativeGarnishSlotBridgeMut<'_> {
        self.game_state
            .sprites
            .garnish_slots
            .slot_mut(&mut self.ram, slot)
    }

    pub(crate) fn sprite_system_mut(&mut self) -> NativeSpriteSystemBridgeMut<'_> {
        NativeSpriteSystemBridgeMut::new(
            &mut self.game_state.sprites.system,
            &mut self.game_state.sprites.sprite_slots,
            &mut self.ram,
        )
    }

    pub(super) fn link_state_pits_after_aux_state(&mut self) {
        self.tile_detect_main_handler(4);
        if self.game_state.player.tile_detection.pit_tile() & 1 == 0 {
            if self
                .game_state
                .enhanced_features
                .has(FEATURES0_MISC_BUG_FIXES)
            {
                self.follower_link_state_mut().clear_near_pit_state();
            }
            if self.game_state.player.follower_link.is_running() {
                self.link_state_dashing();
                return;
            }
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_cancel_dash();
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                self.follower_link_state_mut().clear_direction_lock_bits(1);
            }
            self.follower_link_state_mut().clear_near_pit_state();
            let handler_state = if !self.game_state.player.follower_link.is_bunny_mirror() {
                0
            } else if self.game_state.player.follower_link.has_moon_pearl() {
                3
            } else {
                23
            };
            self.follower_link_state_mut()
                .set_handler_state(handler_state);
            match self.game_state.player.follower_link.handler_state() {
                23 => self.player_handler_17_bunny(),
                3 => self.link_state_temporary_bunny(),
                _ => self.link_state_default(),
            }
            return;
        }

        self.player_tile_detect_nearby();
        self.follower_link_state_mut().set_speed_setting(4);
        if self.game_state.player.tile_detection.pit_tile() & 0x0f == 0 {
            self.follower_link_state_mut().clear_near_pit_state();
            self.follower_link_state_mut().set_speed_setting(0);
            let handler_state = if !self.game_state.player.follower_link.is_bunny_mirror() {
                0
            } else if self.game_state.player.follower_link.has_moon_pearl() {
                3
            } else {
                23
            };
            self.follower_link_state_mut()
                .set_handler_state(handler_state);
            self.link_cancel_dash();
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                self.follower_link_state_mut().clear_direction_lock_bits(1);
            }
            return;
        }

        if self.game_state.player.tile_detection.pit_tile() & 0x0f != 0x0f {
            let mut i = 3i8;
            loop {
                if self.game_state.player.tile_detection.pit_tile() & 0x0f
                    == FALL_HOLE_PIT_DIRS[i as usize]
                {
                    i += 4;
                    break;
                }
                i -= 1;
                if i < 0 {
                    i = 3;
                    let mut pit_tile = self.game_state.player.tile_detection.pit_tile();
                    while pit_tile & 1 == 0 {
                        i -= 1;
                        pit_tile >>= 1;
                    }
                    break;
                }
            }
            self.tile_detect_position_mut()
                .set_fall_hole_scan_index(i as u8);
            let idx = i as usize;
            if self.game_state.player.follower_link.direction() & FALL_HOLE_DIRS[idx] != 0 {
                self.follower_link_state_mut()
                    .set_last_direction_from_current_direction();
                self.follower_link_state_mut().set_speed_setting(6);
                self.link_handle_moving_animation_full_long_entry();
            } else {
                let old_dir = self.game_state.player.follower_link.direction();
                self.follower_link_state_mut()
                    .add_direction_flags(FALL_HOLE_DIRS2[idx]);
                if old_dir != 0 {
                    self.link_handle_moving_animation_full_long_entry();
                }
            }
            self.link_handle_diagonal_collision();
            self.link_handle_velocity();
            self.link_handle_cardinal_collision();
            self.apply_links_movement_to_camera();
            return;
        }

        if !self.game_state.player.follower_link.near_pit_state_is(2) {
            if self.game_state.player.follower_link.has_moon_pearl() {
                self.follower_link_state_mut()
                    .clear_bunny_transform_after_moon_pearl();
            }
            self.follower_link_state_mut().set_direction(0);
            self.follower_link_state_mut().set_near_pit_state(2);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.follower_link_state_mut().set_button_mask_b_y(0);
            self.follower_link_state_mut().set_button_b_frames(0);
            self.follower_link_state_mut().clear_item_in_hand();
            self.follower_link_state_mut().clear_position_mode();
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut().clear_auxiliary_state();
            self.ancilla_sfx3_near(31);
        }

        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().set_z(0);
        self.follower_link_state_mut().set_actual_z_velocity(0);
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().clear_transforming();
        self.link_force_unequip_cape_quietly();
        self.follower_link_state_mut()
            .increment_sprite_damage_disable_timer();
        if (self
            .follower_link_state_mut()
            .decrement_sprite_oam_state_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut().advance_pit_data_index();
        let x = self.game_state.player.follower_link.pit_data_index();
        self.follower_link_state_mut().set_sprite_oam_state_timer(9);
        if self.game_state.sprites.follower_runtime.indicator() != 13 && x == 1 {
            self.follower_state_mut().set_appearance_none_flag(x);
        }

        if x == 6 {
            self.link_cancel_dash();
            self.set_submodule(7);
            self.follower_link_state_mut().set_pit_data_index(6);
            self.follower_link_state_mut().set_near_pit_state(3);
            self.follower_link_state_mut().set_visibility_status(12);
            self.follower_link_state_mut().set_speed_modifier(16);
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                as u8;
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            self.follower_link_state_mut().set_y_button_action_timer(0);
            if self.game_state.world.location.is_indoors() {
                let room = self.game_state.world.location.dungeon_room_index();
                self.dungeon_room_tracking_mut().set_room_index_prev(room);
                self.Dungeon_FlagRoomData_Quadrants();
                if self.Dungeon_IsPitThatHurtsPlayer() {
                    self.dungeon_pit_do_damage();
                    return;
                }
            }
            let previous_room = self.game_state.world.location.dungeon_room_index();
            self.dungeon_room_tracking_mut()
                .set_room_index_prev(previous_room);
            let room = self.game_state.dungeon.header.travel_destination(0);
            self.set_dungeon_room_index(room);
            let player_y = self.game_state.player.follower_link.y();
            self.tile_detect_position_mut().set_y(player_y);
            let new_y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(y as u16)
                .wrapping_sub(0x10);
            self.follower_link_state_mut().set_y(new_y);
            if self.game_state.world.location.is_indoors() {
                self.handle_layer_of_destination();
            } else if self.game_state.world.location.overworld_screen_index() != 5 {
                self.Overworld_GetPitDestination();
                self.set_main_module(17);
                self.set_submodule(0);
                self.set_subsubmodule(0);
            } else {
                self.TakeDamageFromPit();
            }
        }
    }

    pub(super) fn link_state_tree_pull_reset_to_normal(&mut self) {
        self.follower_link_state_mut().set_facing(0);
        self.follower_link_state_mut().clear_state_bits();
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().clear_handler_state();
    }

    pub(super) fn link_state_tree_pull_tail(&mut self) {
        self.link_move_position();
        self.link_handle_cardinal_collision();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn store_link_safe_return_position(&mut self, x: u16, y: u16) {
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
    }

    pub(super) fn ancilla_x(&self, k: usize) -> u16 {
        self.ancilla_slot_view(k).x()
    }

    pub(super) fn ancilla_y(&self, k: usize) -> u16 {
        self.ancilla_slot_view(k).y()
    }

    pub(super) fn has_player_layer_collision(&self, mask: u8) -> bool {
        self.game_state
            .player
            .tile_detection
            .has_layer_collision(mask)
    }

    pub(super) fn set_player_layer_collision(&mut self, mask: u8, enabled: bool) {
        self.tile_detect_position_mut()
            .set_layer_collision(mask, enabled);
    }

    pub(super) fn set_player_layer_collision_flags(&mut self, value: u8) {
        self.tile_detect_position_mut()
            .set_layer_collision_flags(value);
    }
}
