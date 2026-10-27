//! Item receipts, A-button actions, lifting and throwing, gravestones, and pushed blocks.

use super::*;
use super::super::misc::chest_item_alternate;
use super::super::sprite::SpriteSpawnInfo;
use crate::tile_definition::NativeTile;
use crate::types::Point16U;

impl ZeldaState {
    pub(in crate::zelda_rtl) fn link_receive_item(&mut self, item: u8, chest_position: u16) {
        let _ = self.link_receive_item_from(item, chest_position, ItemReceiptCaller::AtomicCaller);
    }

    pub(in crate::zelda_rtl) fn link_receive_item_from(
        &mut self,
        item: u8,
        chest_position: u16,
        caller: ItemReceiptCaller,
    ) -> GameCallStatus {
        if self.game_state.player.follower_link.has_auxiliary_state() {
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut().clear_blink_countdown();
            self.follower_link_state_mut().clear_state_bits();
        }
        self.follower_link_state_mut().set_receive_item_index(item);
        if item == 0x3e {
            self.ancilla_sfx3_near(0x2e);
        }
        self.follower_link_state_mut().set_item_holding_timer(0x60);
        if self.game_state.player.follower_link.item_receipt_method() == 0
            || self.game_state.player.follower_link.item_receipt_method() == 3
        {
            self.follower_link_state_mut().clear_state_bits();
            self.follower_link_state_mut().set_button_mask_b_y(0);
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut().set_button_b_frames(0);
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_direction_lock();
            self.follower_link_state_mut().clear_item_in_hand();
            self.follower_link_state_mut().clear_position_mode();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_handler_state(21);
            self.follower_link_state_mut().set_item_hold_pose(1);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            if item == 0x20 {
                self.follower_link_state_mut().set_item_hold_pose(2);
            }
        }
        let call_status = self.ancilla_add_item_receipt_from(0x22, 4, chest_position, caller);
        if call_status.is_suspended() {
            return call_status;
        }
        self.complete_link_receive_item(item);
        GameCallStatus::Returned
    }

    pub(in crate::zelda_rtl) fn complete_link_receive_item(&mut self, item: u8) {
        if let Some(native_item) = self.early_ground_link_receive_item_tail.take() {
            assert_eq!(native_item, item, "native Link receipt returned to another caller");
            return;
        }
        if item != 0x20 && item != 0x37 && item != 0x38 && item != 0x39 {
            self.hud_refresh_icon();
        }
        self.link_cancel_dash();
    }

    pub(super) fn link_handle_liftables(&mut self) -> u8 {
        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();

        let facing = self.game_state.player.follower_link.facing_index();
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let y0 = link_y.wrapping_add(LINK_HANDLE_LIFTABLES_ACTION_Y[facing] as i16 as u16) & mask;
        let y1 = link_y.wrapping_add(20) & mask;
        let x0 =
            (link_x.wrapping_add(LINK_HANDLE_LIFTABLES_ACTION_X[facing] as i16 as u16) & mask) >> 3;
        let x1 = (link_x.wrapping_add(8) & mask) >> 3;

        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);

        let mut action = if (self.game_state.player.tile_detection.collision_bits()
            | self.game_state.player.tile_detection.vertical_ledge() as u16)
            & 1
            != 0
        {
            3
        } else {
            2
        };

        if self.game_state.world.location.is_indoors() {
            let liftable = self.dungeon_check_for_and_id_liftable_tile();
            if liftable != 0xffff {
                self.tile_detect_position_mut()
                    .set_liftable_action_index_primary(
                        LINK_HANDLE_LIFTABLES_ACTION_FOR_TILE[(liftable & 0x0f) as usize],
                    );
            } else {
                if self.game_state.player.tile_detection.read_something() & 1 != 0
                    && self.game_state.player.follower_link.facing() == 0
                    && self.game_state.player.tile_detection.liftable_tile_index() == 0
                {
                    action = 4;
                }
                if self.game_state.player.tile_detection.chest() & 1 != 0 {
                    action = 5;
                }
                return action;
            }
        } else {
            if self.game_state.player.tile_detection.read_something() & 1 == 0 {
                if self.game_state.player.tile_detection.chest() & 1 != 0 {
                    action = 5;
                }
                return action;
            }
            if self.game_state.player.follower_link.facing() == 0
                && self.game_state.player.tile_detection.liftable_tile_index() == 0
            {
                action = 4;
                if self.game_state.player.tile_detection.chest() & 1 != 0 {
                    action = 5;
                }
                return action;
            }
            let liftable_primary = self.game_state.player.tile_detection.liftable_tile_index() >> 1;
            self.tile_detect_position_mut()
                .set_liftable_action_index_primary(liftable_primary);
        }

        let liftable_index = self
            .game_state
            .player
            .tile_detection
            .liftable_action_index_primary() as usize;
        if self.game_state.inventory.items.gloves()
            >= LINK_HANDLE_LIFTABLES_ACTION_FOR_GLOVES[liftable_index]
        {
            action = 1;
        }

        if self.game_state.player.tile_detection.chest() & 1 != 0 {
            action = 5;
        }
        action
    }

    pub(super) fn link_bonk_and_smash(&mut self) {
        if !self.game_state.player.follower_link.is_running()
            || self.game_state.player.follower_link.dash_counter() == 64
            || self.game_state.player.tile_detection.dashable_tiles() & 0x70 == 0
        {
            return;
        }

        for i in 0..2 {
            if let Some((j, x, y)) = self.overworld_smash_rock_pile_result(i != 0) {
                if let Some(k) = LINK_BONK_AND_SMASH_LIFTABLE_TILE_ATTR_TO_TERRAIN_TYPE
                    .iter()
                    .position(|&v| v == j)
                {
                    if k == 2 || k == 4 {
                        self.ancilla_sfx3_near(0x32);
                    }
                    self.sprite_spawn_immediately_smashed_terrain(k as u8, x, y);
                }
            }
        }
    }

    pub(super) fn overworld_smash_rock_pile_result(
        &mut self,
        down_one_tile: bool,
    ) -> Option<(u8, u16, u16)> {
        let bak = self.game_state.player.follower_link.y();
        if down_one_tile {
            self.follower_link_state_mut().set_y(bak.wrapping_add(8));
        }
        let (pos, x, y) = self.overworld_get_link_map16_coords_result();
        self.follower_link_state_mut().set_y(bak);
        let a = self
            .game_state
            .dungeon
            .room_tilemaps
            .bg2_tile_by_byte_pos(pos);
        match a {
            0x226 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 0, x, y), x, y)),
            0x227 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 1, x, y), x, y)),
            0x228 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 2, x, y), x, y)),
            0x229 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 3, x, y), x, y)),
            0x36 => Some((
                self.overworld_lifting_small_obj_impl(a, pos, 0x0dc7, x, y),
                x,
                y,
            )),
            _ => None,
        }
    }

    pub(in crate::zelda_rtl) fn overworld_get_link_map16_coords_result(&self) -> (u16, u16, u16) {
        let dir = self.game_state.player.follower_link.facing_index();
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(OVERWORLD_GET_LINK_MAP16_COORDS_RESULT_X_OFFSETS[dir] as u16)
            & !0x0f;
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(OVERWORLD_GET_LINK_MAP16_COORDS_RESULT_Y_OFFSETS[dir] as u16)
            & !0x0f;
        let pos = ((y.wrapping_sub(self.game_state.world.scroll.overworld_offset_base_y())
            & self.game_state.world.scroll.overworld_offset_mask_y())
            << 3)
            .wrapping_add(
                ((x >> 3).wrapping_sub(self.game_state.world.scroll.overworld_offset_base_x()))
                    & self.game_state.world.scroll.overworld_offset_mask_x(),
            );
        (pos, x, y)
    }

    pub(in crate::zelda_rtl) fn overworld_lifting_small_obj_impl(
        &mut self,
        a: u16,
        pos: u16,
        mut tile: u16,
        x: u16,
        y: u16,
    ) -> u8 {
        let secret = self.overworld_reveal_secret(pos);
        if secret != 0 {
            tile = secret;
        }
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile_by_byte_pos(pos, tile);
        self.overworld_memorize_map16_change_for_smash(pos, tile);
        self.overworld_draw_map16(pos, tile);
        self.set_bg_vram_load_mode(1);
        self.map16_quadrant_attr(a, x, y)
    }

    pub(in crate::zelda_rtl) fn smash_rock_pile_from_lift_impl(
        &mut self,
        a: u16,
        pos: u16,
        quadrant: usize,
        x: u16,
        y: u16,
    ) -> u8 {
        let pos = 2
            * ((pos >> 1).wrapping_add(
                SMASH_ROCK_PILE_FROM_LIFT_IMPL_BIG_ROCK_MAP16_QUADRANT_OFFSETS[quadrant] as u16,
            ));
        self.dungeon_object_tracking_mut()
            .set_big_rock_starting_address(pos);
        self.dungeon_doors_mut().set_door_open_counter(40);
        let secret = self.overworld_reveal_secret(pos);
        if secret == 0xffff {
            let screen =
                u16::from(self.game_state.world.location.overworld_screen_index()) as usize;
            self.set_overworld_event_bits(screen, 0x20);
            self.set_sound_effect_2(27);
            self.dungeon_doors_mut().set_door_open_counter(80);
        }
        let x = x.wrapping_add(
            (SMASH_ROCK_PILE_FROM_LIFT_IMPL_BIG_ROCK_QUADRANT_X_OFFSETS[quadrant] * 2) as u16,
        );
        let y = y.wrapping_add(
            (SMASH_ROCK_PILE_FROM_LIFT_IMPL_BIG_ROCK_QUADRANT_Y_OFFSETS[quadrant] * 2) as u16,
        );
        self.overworld_do_map_update32x32_b();
        self.map16_quadrant_attr(a, x, y)
    }

    pub(in crate::zelda_rtl) fn overworld_memorize_map16_change_for_smash(&mut self, pos: u16, value: u16) {
        if value == 0x0dc5 || value == 0x0dc9 {
            return;
        }
        let x = self.game_state.memorized_tiles.count() as usize;
        self.memorized_tile_mut().set_entry_value(x, value);
        self.memorized_tile_mut().set_entry_addr(x, pos);
        self.memorized_tile_mut().set_count((x + 2) as u16);
    }

    fn map16_quadrant_attr(&self, map16: u16, x: u16, y: u16) -> u8 {
        let index = map16 as usize * 4 + usize::from(x & 8 != 0) * 2 + usize::from(y & 8 != 0);
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("map16_quadrant_attr missing kMap16ToMap8 asset");
        let tile_attrs = self
            .asset_raw(163)
            .expect("map16_quadrant_attr missing kMap8DataToTileAttr asset");
        let map8 = read_word_from_slice(map16_to_map8, index * 2);
        tile_attrs[(map8 & 0x01ff) as usize]
    }

    pub(super) fn link_handle_a_press_marked(&mut self) {
        // $07:81B0 and $07:8510 JSR $9BAA; resumed tails retain their entry.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x079baa));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x079baa);
        let p = &self.game_state.player.follower_link;
        if p.has_item_in_hand() { b.taken(0x079bb0); }
        else {
            b.block(0x079bb2);
            if p.position_mode_has(0x1f) { b.taken(0x079bb7); }
            else {
                b.block(0x079bb9);
                if p.player_pose_draw_counter() != 0 { b.taken(0x079bbc); }
                else {
                    b.block(0x079bbe);
                    if p.button_b_frames() >= 9 { b.taken(0x079bc2); }
                    else { b.block(0x079bc4); if p.button_mask_b_y() & 0x80 == 0 { b.taken(0x079bc8); } }
                }
            }
        }
        self.follower_link_state_mut()
            .set_sprite_pickup_flag_cached(0);
        if self.game_state.player.follower_link.has_item_in_hand()
            || self.game_state.player.follower_link.position_mode_has(0x1f)
            || self
                .game_state
                .player
                .follower_link
                .player_pose_draw_counter()
                != 0
        {
            b.block(0x079bca); return;
        }
        if self.game_state.player.follower_link.button_b_frames() < 9
            && (self.game_state.player.follower_link.button_mask_b_y() & 0x80) != 0
        {
            b.block(0x079bca); return;
        }

        b.block(0x079bcb);
        let mut action = self.game_state.player.follower_link.tile_action_index();
        if !self.game_state.player.follower_link.has_action_state()
            && !self
                .game_state
                .player
                .follower_link
                .has_grabbing_wall_state()
        {
            b.block(0x079bd6);
            let new_press = self.link_check_new_a_press();
            b.block(0x079bd9);
            if !new_press {
                b.taken(0x079bd9); b.block(0x079c4c);
                self.follower_link_state_mut().set_y_button_action_flags(0);
                return;
            }
            b.block(0x079bdb);
            if !self.game_state.player.follower_link.needs_pull_for_rupees_sprite() { b.taken(0x079bde); }
            else {
                b.block(0x079be0);
                if self.game_state.player.follower_link.facing() != 0 { b.taken(0x079be2); }
            }
            if self
                .game_state
                .player
                .follower_link
                .needs_pull_for_rupees_sprite()
                && self.game_state.player.follower_link.facing() == 0
            {
                b.block(0x079be4);
                action = 7;
            } else {
                b.block(0x079be9);
                if self.game_state.player.follower_link.is_near_moveable_statue() {
                    b.block(0x079bee); action = 6;
                } else {
                    b.taken(0x079bec); b.block(0x079bf3);
                    let mut attempt_action = false;
                    if self.game_state.player.follower_link.ancilla_pickup_flag() == 0 {
                        b.block(0x079bf8);
                        if self.game_state.player.follower_link.sprite_pickup_flag() == 0 {
                            b.taken(0x079bfb); b.block(0x079c2c);
                            action = self.link_handle_liftables();
                            attempt_action = true;
                        } else {
                            b.block(0x079bfd);
                            let pickup_flag = self.game_state.player.follower_link.sprite_pickup_flag();
                            self.follower_link_state_mut()
                                .set_sprite_pickup_flag_cached(pickup_flag);
                        }
                    }
                    if !attempt_action {
                        if self.game_state.player.follower_link.ancilla_pickup_flag() != 0 { b.taken(0x079bf6); }
                        b.block(0x079c03);
                        if self.game_state.player.follower_link.button_b_frames() == 0 { b.taken(0x079c05); }
                        if self.game_state.player.follower_link.button_b_frames() != 0 {
                            b.block(0x079c07);
                            self.link_reset_sword_and_item_usage();
                        }
                        b.block(0x079c0a);
                        if !self.game_state.player.follower_link.has_item_or_position_mode() { b.taken(0x079c10); }
                        if self
                            .game_state
                            .player
                            .follower_link
                            .has_item_or_position_mode()
                        {
                            b.block(0x079c12);
                            self.follower_link_state_mut().clear_item_in_hand();
                            self.follower_link_state_mut().clear_position_mode();
                            self.link_reset_boomerang_y_stuff();
                            b.block(0x079c1b);
                            self.minigame_state_mut().clear_flag_boomerang_in_place();
                            if self.ancilla_slot_view(0).ancilla_type() == 5 {
                                b.block(0x079c25);
                                self.ancilla_slot_view_mut(0).set_ancilla_type(0);
                            } else { b.taken(0x079c23); }
                        }
                        b.block(0x079c28);
                        action = 1;
                    }
                }
            }

            b.block(0x079c2f);
            if action as usize >= LINK_APRESS_BASIC_ABILITY_BITMASKS.len()
                || (LINK_APRESS_BASIC_ABILITY_BITMASKS[action as usize]
                    & self.game_state.inventory.player_resources.ability_flags())
                    == 0
            {
                b.taken(0x079c36); b.block(0x079c4c);
                self.follower_link_state_mut().set_y_button_action_flags(0);
                return;
            }
            b.block(0x079c38);
            self.follower_link_state_mut().set_tile_action_index(action);
            self.link_a_press_perform_basic(action.wrapping_mul(2));
        } else { b.taken(0x079bd4); }

        b.block(0x079c41);
        let action_index = self.game_state.player.follower_link.tile_action_index();
        self.follower_link_state_mut()
            .set_cached_tile_action_index(action_index);
        match self.game_state.player.follower_link.tile_action_index() {
            1 => self.link_a_press_lift_carry_throw(),
            3 => self.link_a_press_pull_object(),
            6 => self.link_a_press_statue_drag(),
            _ => {}
        }
    }

    pub(super) fn link_a_press_perform_basic(&mut self, action_x2: u8) {
        match action_x2 >> 1 {
            0 => self.link_perform_desert_prayer(),
            1 => self.link_perform_throw(),
            2 => self.link_perform_dash(),
            3 => self.link_perform_grab(),
            4 => self.link_perform_read(),
            5 => self.link_perform_open_chest(),
            6 => self.link_perform_statue_drag(),
            7 => self.link_perform_rupee_pull(),
            // C Link_APress_PerformBasic asserts for action slots outside 0..=7.
            _ => panic!("Link_APress_PerformBasic action {}", action_x2 >> 1),
        }
    }

    pub(super) fn link_check_new_a_press(&mut self) -> bool {
        if self.game_state.player.follower_link.y_button_action_flags() & 0x80 != 0
            || self.game_state.player.follower_link.incapacitated_timer() != 0
            || self.game_state.player.follower_link.filtered_joypad_l() & 0x80 == 0
        {
            return false;
        }
        self.follower_link_state_mut()
            .add_y_button_action_flag_bits(0x80);
        true
    }

    pub(in crate::zelda_rtl) fn link_perform_dash(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .has_somaria_platform_state()
            || (self.game_state.player.follower_link.sprite_pickup_flag()
                | self.game_state.player.follower_link.ancilla_pickup_flag())
                != 0
            || self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
        {
            return;
        }
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut().set_dash_countdown(29);
        self.follower_link_state_mut().set_dash_counter(64);
        self.follower_link_state_mut().set_handler_state(17);
        self.follower_link_state_mut().start_running();
        let button_mask_b_y = self.game_state.player.follower_link.button_mask_b_y() & 0x80;
        self.follower_link_state_mut()
            .set_button_mask_b_y(button_mask_b_y);
        self.follower_link_state_mut().clear_state_bits();
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().clear_defense_flags();
        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();

        let follower = self.game_state.sprites.follower_runtime.indicator() as usize;
        if self.game_state.sprites.follower_runtime.indicator()
            == DASH_FOLLOWER_SLOWDOWN_INDICATORS[follower]
        {
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_state_mut().set_reacquire_timer(64);
        }
    }

    pub(super) fn link_perform_grab(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x80 != 0
            && self.game_state.player.follower_link.button_b_frames() >= 9
        {
            return;
        }
        self.follower_link_state_mut().set_grabbing_wall(1);
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_timer(0);
        self.follower_link_state_mut().clear_item_action_step_var();
    }

    pub(super) fn link_perform_read(&mut self) {
        let message = if self.game_state.world.location.is_indoors() {
            self.dungeon_get_tele_msg(self.game_state.world.location.dungeon_room() as usize)
        } else if self.game_state.inventory.save_progress.progress_indicator() < 2 {
            0x003a
        } else {
            self.asset_u16(
                110,
                u16::from(self.game_state.world.location.overworld_screen_index()) as usize,
            )
        };
        self.dialogue_message_index_mut().set_value(message);
        self.main_show_text_message();
        self.follower_link_state_mut().set_y_button_action_flags(0);
    }

    pub(super) fn link_perform_open_chest(&mut self) {
        if self.game_state.player.follower_link.facing() != 0
            || self.game_state.player.follower_link.item_receipt_method() != 0
            || self.game_state.player.follower_link.has_auxiliary_state()
        {
            return;
        }

        self.follower_link_state_mut().set_y_button_action_flags(0);
        let Some((mut item, chest_position)) = self
            .OpenChestForItemResult(self.game_state.player.tile_detection.interacting_tile() as u8)
        else {
            self.follower_link_state_mut().set_item_receipt_method(0);
            return;
        };

        self.follower_link_state_mut().set_item_receipt_method(1);

        // HandleAPress saves twice the original chest item-table index,
        // including when an owned item is replaced with its alternate.
        if let Some(entry) = &mut self.native_item_graphics_entry {
            entry.saved_link_x = Some(item.wrapping_mul(2));
        }

        if let Some((equipment, alternate)) = chest_item_alternate(item) {
            if self.game_state.inventory.items.equipment(equipment) != 0 {
                item = alternate;
            }
        }

        let caller = if self.ground_apress_defers_atomic_item_receipt {
            ItemReceiptCaller::GroundApress
        } else {
            ItemReceiptCaller::AtomicCaller
        };
        let _ = self.link_receive_item_from(item, chest_position, caller);
    }

    pub(super) fn link_perform_statue_drag(&mut self) {
        self.follower_link_state_mut().set_grabbing_wall(2);
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_timer(0);
        self.follower_link_state_mut().clear_item_action_step_var();
    }

    pub(super) fn link_perform_rupee_pull(&mut self) {
        if self.game_state.player.follower_link.facing() != 0 {
            return;
        }
        self.link_reset_properties_a();
        self.follower_link_state_mut().set_grabbing_wall(2);
        self.follower_link_state_mut().set_direction_lock_bits(2);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_timer(0);
        self.follower_link_state_mut().clear_item_action_step_var();
        self.follower_link_state_mut().set_handler_state(29);
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_button_mask_b_y(0);
    }

    pub(super) fn search_for_byrna_spark(&self) -> bool {
        if self.game_state.player.follower_link.position_mode_has(8) {
            return false;
        }
        (0..=4)
            .rev()
            .any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x31)
    }

    pub(super) fn link_permission_for_slosh_sounds(&self) -> bool {
        if self.game_state.player.follower_link.direction() & 0x0f == 0 {
            return true;
        }
        if self.game_state.player.follower_link.handler_state() != 17 {
            self.game_state.frame.frame_counter & 0x0f != 0
        } else {
            self.game_state.frame.frame_counter & 0x07 != 0
        }
    }

    pub(super) fn link_a_press_lift_carry_throw(&mut self) {
        if !self.game_state.player.follower_link.has_action_state() {
            return;
        }
        if self
            .game_state
            .player
            .follower_link
            .picking_throw_state_has(2)
            && self.game_state.player.follower_link.y_button_action_timer() >= 5
        {
            self.follower_link_state_mut().set_y_button_action_timer(5);
        }
        if self
            .game_state
            .player
            .follower_link
            .has_picking_throw_state()
        {
            self.halt_link_when_using_items();
        }
        if self.game_state.player.follower_link.is_lift_throw_primed() {
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_frame_change_counter();
            self.follower_link_state_mut().clear_direction_flags(0x0f);
        }
        self.follower_link_state_mut()
            .decrement_y_button_action_timer();
        if self.game_state.player.follower_link.y_button_action_timer() != 0 {
            return;
        }
        if self
            .game_state
            .player
            .follower_link
            .picking_throw_state_has(2)
        {
            self.follower_link_state_mut().clear_state_bits();
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            if self.game_state.player.follower_link.handler_state() == 24 {
                self.follower_link_state_mut().clear_handler_state();
            }
        } else if self.game_state.player.follower_link.action_handler_timer() != 0 {
            if self
                .game_state
                .player
                .follower_link
                .action_handler_timer()
                .wrapping_add(1)
                != 9
            {
                self.follower_link_state_mut()
                    .increment_action_handler_timer();
                let timer = self.game_state.player.follower_link.action_handler_timer() as usize;
                self.follower_link_state_mut().set_y_button_action_timer(
                    LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_ACTION_TIMERS[timer],
                );
                self.follower_link_state_mut().set_y_button_action_step(
                    LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_ACTION_STEPS[timer],
                );
                if self.game_state.player.follower_link.action_handler_timer() == 6 {
                    self.dungeon_secret_scratch_mut().clear_pending_kind();
                    let (what, x, y) = if self.game_state.world.location.is_indoors() {
                        let mut pt = Point16U { x: 0, y: 0 };
                        let what = self.dungeon_lift_and_replace_liftable(&mut pt);
                        (what, pt.x, pt.y)
                    } else {
                        let mut pt = Point16U { x: 0, y: 0 };
                        let what = self.overworld_handle_liftable_tiles(&mut pt);
                        (what, pt.x, pt.y)
                    };
                    self.follower_link_state_mut().set_handler_state(24);
                    self.follower_link_state_mut().set_sprite_pickup_flag(1);
                    self.sprite_spawn_throwable_terrain((what & 0x0f).wrapping_add(1), x, y);
                    self.follower_link_state_mut()
                        .clear_filtered_joypad_l_bits(0x80);
                }
                return;
            }
        } else {
            if self.game_state.player.follower_link.y_button_action_step() as usize
                >= LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_SEQUENCE_TIMERS.len() - 1
            {
                return;
            }
            let y_button_action_step = self
                .game_state
                .player
                .follower_link
                .y_button_action_step()
                .wrapping_add(1);
            self.follower_link_state_mut()
                .set_y_button_action_step(y_button_action_step);
            self.follower_link_state_mut().set_y_button_action_timer(
                LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_SEQUENCE_TIMERS
                    [y_button_action_step as usize],
            );
            if self.game_state.player.follower_link.y_button_action_step() != 3 {
                return;
            }
        }
        self.follower_link_state_mut().clear_picking_throw_state();
        self.follower_link_state_mut().clear_direction_lock_bits(1);
    }

    pub(super) fn link_a_press_pull_object(&mut self) {
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        let facing = self.game_state.player.follower_link.facing_index();
        if LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_DIRS[facing]
            & self.game_state.player.follower_link.joypad1h_last()
            == 0
        {
            self.follower_link_state_mut().clear_item_action_step_var();
            let step = self.game_state.player.follower_link.item_action_step_var() as usize;
            self.follower_link_state_mut()
                .set_y_button_action_step(LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_ANIM_STEPS[step]);
            self.follower_link_state_mut()
                .set_y_button_action_timer(LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_ANIM_TIMER[step]);
        } else {
            self.follower_link_state_mut()
                .decrement_y_button_action_timer();
            if (self.game_state.player.follower_link.y_button_action_timer() as i8) < 0 {
                let step = self
                    .follower_link_state_mut()
                    .advance_item_action_step_var_wrapping_7_to_1()
                    as usize;
                self.follower_link_state_mut()
                    .set_y_button_action_step(LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_ANIM_STEPS[step]);
                self.follower_link_state_mut()
                    .set_y_button_action_timer(LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_ANIM_TIMER[step]);
            }
        }
        if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut().clear_direction_lock_bits(1);
        }
    }

    pub(super) fn link_a_press_statue_drag(&mut self) {
        self.follower_link_state_mut().set_speed_setting(20);
        let j = self.game_state.player.follower_link.joypad1h_last()
            & LINK_A_PRESS_STATUE_DRAG_GRAB_WALL_DIRS
                [self.game_state.player.follower_link.facing_index()];
        if j == 0 {
            self.follower_link_state_mut()
                .clear_movement_velocity_and_direction();
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_item_action_step_var();
        } else {
            self.follower_link_state_mut().set_direction(j);
            self.follower_link_state_mut()
                .decrement_y_button_action_timer();
            if (self.game_state.player.follower_link.y_button_action_timer() as i8) >= 0 {
                if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
                    self.link_a_press_statue_drag_release();
                }
                return;
            }
            self.follower_link_state_mut()
                .advance_item_action_step_var_wrapping_7_to_1();
        }
        let step = self.game_state.player.follower_link.item_action_step_var() as usize;
        self.follower_link_state_mut()
            .set_y_button_action_step(LINK_A_PRESS_STATUE_DRAG_GRAB_WALL_ANIM_STEPS[step]);
        self.follower_link_state_mut()
            .set_y_button_action_timer(LINK_A_PRESS_STATUE_DRAG_GRAB_WALL_ANIM_TIMER[step]);
        if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
            self.link_a_press_statue_drag_release();
        }
    }

    fn link_a_press_statue_drag_release(&mut self) {
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_near_moveable_statue();
        self.follower_link_state_mut().clear_item_action_step_var();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().clear_grabbing_wall();
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
    }

    pub(super) fn gravestone_move(&mut self, k: usize) {
        if self.game_state.frame.submodule != 0 {
            return;
        }
        self.ancilla_slot_view_mut(k).set_y_velocity((-8i8) as u8);
        self.ancilla_move_y(k);

        self.gravestone_act_as_barrier(k);
        let y_target = self.ancilla_slot_view(k).ab_word();
        let y_cur = self.ancilla_get_y(k);
        if y_cur >= y_target {
            return;
        }

        self.ancilla_slot_view_mut(k).set_ancilla_type(0);
        self.follower_link_state_mut().clear_hookshot_grave_latch();
        self.follower_link_state_mut().and_defense_flags(!4);
        // `LDA $03BA,X` / `LDA $03B6,X`: byte-indexed reads of the RAM-resident
        // door-debris arrays.
        let debris_y = self.ram[crate::game_state::constants::DOOR_DEBRIS_Y + k];
        let debris_x = self.ram[crate::game_state::constants::DOOR_DEBRIS_X + k];
        self.tile_detect_position_mut()
            .set_interaction_scratch_y_bytes(debris_y, debris_x);
        let big_rock = self
            .game_state
            .player
            .tile_detection
            .interaction_scratch_y();
        self.dungeon_object_tracking_mut()
            .set_big_rock_starting_address(big_rock);
        let counter = match big_rock {
            0x0532 => 0x48,
            0x0488 => 0x60,
            _ => 0x40,
        };
        self.dungeon_doors_mut().set_door_open_counter(counter);
        self.overworld_do_map_update32x32_b();
    }

    pub(super) fn somaria_block_handle_player_interaction(&mut self, k: usize) {
        self.sprite_system_mut().set_cur_object_index(k as u8);
        if self.ancilla_slot_view(k).g() != 0 {
            return;
        }

        if self.ancilla_slot_view(k).h() == 0 {
            if self.game_state.player.follower_link.has_auxiliary_state()
                || self.game_state.player.follower_link.state_bits_has(1)
                || {
                    let z = self.ancilla_slot_view(k).z();
                    z != 0 && z != 0xff
                }
                || self.ancilla_slot_view(k).k() != 0
                || self.ancilla_slot_view(k).l() != 0
            {
                return;
            }
            if self.game_state.player.follower_link.joypad1h_last() & 0x0f == 0 {
                self.ancilla_slot_view_mut(k).set_work_byte_3(0);
                self.follower_link_state_mut().clear_defense_flags();
                self.ancilla_slot_view_mut(k).set_a(255);
                if !self.game_state.player.follower_link.is_running() {
                    self.follower_link_state_mut().set_speed_setting(0);
                    return;
                }
            } else if self.game_state.player.follower_link.joypad1h_last() & 0x0f
                == self.ancilla_slot_view(k).work_byte_3()
            {
                if self.game_state.player.follower_link.speed_setting() == 18 {
                    self.follower_link_state_mut().or_defense_flags(0x81);
                }
            } else {
                let last_direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
                self.ancilla_slot_view_mut(k)
                    .set_work_byte_3(last_direction);
                self.follower_link_state_mut().set_speed_setting(0);
            }

            if !self.ancilla_check_link_collision(k, 4)
                || self.ancilla_slot_view(k).floor()
                    != self.game_state.player.follower_link.lower_level_state()
            {
                return;
            }

            if !self.game_state.player.follower_link.is_running()
                || self.game_state.player.follower_link.dash_counter() == 64
            {
                let t = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
                self.ancilla_slot_view_mut(k).set_work_byte_3(t);
                if t & 3 != 0 {
                    let mut ancilla = self.ancilla_slot_view_mut(k);
                    ancilla.set_y_velocity(0);
                    ancilla.set_x_velocity(if t & 1 != 0 { 16 } else { (-16i8) as u8 });
                    ancilla.set_direction(if t & 1 != 0 { 3 } else { 2 });
                } else {
                    let mut ancilla = self.ancilla_slot_view_mut(k);
                    ancilla.set_x_velocity(0);
                    ancilla.set_y_velocity(if t & 8 != 0 { (-16i8) as u8 } else { 16 });
                    ancilla.set_direction(if t & 8 != 0 { 0 } else { 1 });
                }
                if self.game_state.player.follower_link.actual_y_velocity() == 0
                    || self.game_state.player.follower_link.actual_x_velocity() == 0
                {
                    if !self.ancilla_check_tile_collision_class2(k, AncillaClass2Entry::Unpriced) {
                        self.ancilla_move_y(k);
                        self.ancilla_move_x(k);
                        let movement_ticks = self.ancilla_slot_view_mut(k).advance_a();
                        if !self
                            .game_state
                            .player
                            .follower_link
                            .is_lifting_or_carrying()
                            && movement_ticks & 7 == 0
                        {
                            self.ancilla_sfx2_pan(k, 0x22);
                        }
                    }
                    self.follower_link_state_mut().set_defense_flags(0x81);
                    self.follower_link_state_mut().set_speed_setting(0x12);
                }
                self.sprite_nullify_hookshot_drag();
                return;
            }

            if self.game_state.player.follower_link.ancilla_pickup_flag() == k as u8 + 1 {
                self.follower_link_state_mut().clear_ancilla_pickup_flag();
            }
            self.link_cancel_dash();
            self.ancilla_sfx3_pan(k, 0x32);
            let j = self.game_state.player.follower_link.facing_index();
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_direction(j as u8);
                ancilla.set_y_velocity(LINK_ITEM_CANE_OF_SOMARIA_BLOCK_Y_VELOCITIES[j]);
                ancilla.set_x_velocity(LINK_ITEM_CANE_OF_SOMARIA_BLOCK_X_VELOCITIES[j]);
                ancilla.set_z_velocity(48);
                ancilla.set_z(0);
            }
            self.ancilla_slot_view_mut(k).set_h(1);
        }

        self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
        self.ancilla_move_z(k);
        let z = self.ancilla_slot_view(k).z();
        if z != 0 && z < 252 {
            return;
        }

        self.ancilla_sfx2_pan(k, 0x21);
        self.ancilla_slot_view_mut(k).set_z(0);
        let j = self.ancilla_slot_view(k).h();
        self.ancilla_slot_view_mut(k).set_h(j.wrapping_add(1));
        if j == 3 {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_work_byte_4(0);
            ancilla.set_h(0);
        } else {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            let y_velocity = ((ancilla.y_velocity() as i8) / 2) as u8;
            let x_velocity = ((ancilla.x_velocity() as i8) / 2) as u8;
            ancilla.set_z_velocity(
                LINK_ITEM_CANE_OF_SOMARIA_BLOCK_Z_VELOCITIES[j.wrapping_sub(1) as usize],
            );
            ancilla.set_y_velocity(y_velocity);
            ancilla.set_x_velocity(x_velocity);
        }
    }

    pub(super) fn gravestone_act_as_barrier(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x08_ee57);
        crate::cycle_ledger::charge(600); // Coordinates, bounds and first Y comparison.
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        let r4 = y.wrapping_add(0x18);
        let r6 = x.wrapping_add(0x20);
        let lx = self.game_state.player.follower_link.x().wrapping_add(8);
        let ly = self.game_state.player.follower_link.y().wrapping_add(8);
        let intersects = if ly < y {
            crate::cycle_ledger::charge(6);
            false
        } else {
            crate::cycle_ledger::charge(48);
            if ly >= r4 {
                crate::cycle_ledger::charge(6);
                false
            } else {
                crate::cycle_ledger::charge(118);
                if lx < x {
                    crate::cycle_ledger::charge(6);
                    false
                } else {
                    crate::cycle_ledger::charge(48);
                    if lx >= r6 {
                        crate::cycle_ledger::charge(6);
                        false
                    } else {
                        true
                    }
                }
            }
        };
        if intersects {
            crate::cycle_ledger::charge(94 + if ly.wrapping_sub(r4) & 0x8000 != 0 { 38 } else { 6 });
            let r10 = ly.abs_diff(r4);
            let new_y = self.game_state.player.follower_link.y().wrapping_add(r10);
            self.follower_link_state_mut().set_y(new_y);
            // The source compares the full word at $30 before isolating the
            // resulting low byte. X velocity supplies that word's high byte.
            let velocity_word = u16::from(self.game_state.player.follower_link.y_velocity())
                | (u16::from(self.game_state.player.follower_link.x_velocity()) << 8);
            crate::cycle_ledger::charge(182 + if velocity_word < 0x80 { 6 } else { 24 } + 364);
            self.follower_link_state_mut()
                .add_y_velocity_delta(r10 as u8);
            self.follower_link_state_mut().or_defense_flags(4);
        }
        crate::cycle_ledger::charge(62); // SEP, facing load and BEQ.
        if self.game_state.player.follower_link.has_facing() {
            crate::cycle_ledger::charge(64);
            // EED7..EEDB reads and writes $48, not the facing byte at $2F.
            self.follower_link_state_mut().and_defense_flags(!4);
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(42);
    }

    pub(super) fn find_free_moving_block_slot(&mut self, x: u8) -> u8 {
        if self
            .game_state
            .dungeon
            .object_tracking
            .changeable_object_index(1)
            == 0
        {
            self.dungeon_object_tracking_mut()
                .set_changeable_object_index(1, x.wrapping_add(1));
            return 1;
        }
        if self
            .game_state
            .dungeon
            .object_tracking
            .changeable_object_index(0)
            == 0
        {
            self.dungeon_object_tracking_mut()
                .set_changeable_object_index(0, x.wrapping_add(1));
            return 0;
        }
        0xff
    }

    pub(super) fn initialize_push_block(&mut self, r14: u8, idx: u8) -> bool {
        let slot = r14 as usize;
        let idx_word = (idx >> 1) as usize;
        let pos = self
            .game_state
            .dungeon
            .object_tracking
            .object_tilemap_pos(idx_word);
        let mut x = (pos & 0x007e) << 2;
        let mut y = (pos & 0x1f80) >> 4;
        x = x.wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_h() & 0xff00);
        y = y.wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_v() & 0xff00);

        self.pushed_block_mut().init_slot(slot, x, y);

        if self.game_state.dungeon.header.primary_header_tag() != 38
            && self
                .game_state
                .dungeon
                .object_tracking
                .object_record(idx_word)
                .is_idle()
            && !self.push_block_attempt_to_push_the_block(0, x, y)
        {
            self.ancilla_sfx2_near(0x22);
            self.dungeon_object_tracking_mut()
                .set_object_record(idx_word, crate::game_state::ObjectRecord::PUSHED);
            return false;
        }

        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(slot);
        true
    }

    pub(crate) fn native_push_block_drawing_table_byte(address: u32) -> Option<u8> {
        match address {
            0x07f0cc..=0x07f0cf => Some(
                SPRITE_DUNGEON_DRAW_SINGLE_PUSH_BLOCK_CHARS[(address - 0x07f0cc) as usize],
            ),
            0x07f0d0..=0x07f0d8 => Some(
                SPRITE_DUNGEON_DRAW_SINGLE_PUSH_BLOCK_PUSH_BLOCK_CHAR_INDEX_BY_MODE
                    [(address - 0x07f0d0) as usize] as u8,
            ),
            _ => None,
        }
    }

    pub(in crate::zelda_rtl) fn sprite_dungeon_draw_single_push_block(&mut self, mut j: usize) {
        let _scope = crate::cycle_ledger::routine(0x07_f0d9);
        crate::cycle_ledger::charge(100); // PHY, allocation size, JSL allocator.
        j >>= 1;
        self.oam_allocate_from_region_b(4);
        crate::cycle_ledger::charge(710); // PLY, relative coordinates, animation table and BNE.
        let y = self
            .game_state
            .player
            .pushed_block
            .y(j)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
            .wrapping_sub(1);
        let x = self
            .game_state
            .player
            .pushed_block
            .x(j)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let ch = SPRITE_DUNGEON_DRAW_SINGLE_PUSH_BLOCK_CHARS
            [SPRITE_DUNGEON_DRAW_SINGLE_PUSH_BLOCK_PUSH_BLOCK_CHAR_INDEX_BY_MODE
                [self.game_state.player.pushed_block.animation_mode() as usize]];
        if ch != 0xff {
            crate::cycle_ledger::charge(6 + 492); // Taken BNE and OAM/extension stores.
            self.oam_state_mut()
                .write_current_entry_with_extended(x as u8, y as u8, ch, 0x20, 2);
        } else {
            crate::cycle_ledger::charge(22); // BRA over the stores.
        }
        crate::cycle_ledger::charge(70); // PLY : RTS.
    }

    pub(super) fn link_perform_throw(&mut self) {
        if (self.game_state.player.follower_link.sprite_pickup_flag()
            | self.game_state.player.follower_link.ancilla_pickup_flag())
            == 0
        {
            self.link_reset_sword_and_item_usage();
            self.follower_link_state_mut().set_y_button_action_flags(0);
            let mut i = 15i8;
            while self.sprite_slot_view(i as usize).state() != 0 {
                i -= 1;
                if i < 0 {
                    return;
                }
            }

            if matches!(
                self.game_state
                    .player
                    .tile_detection
                    .liftable_action_index_primary(),
                5 | 6
            ) {
                self.follower_link_state_mut().set_action_handler_timer(1);
            } else {
                let (attr, x, y) = if self.game_state.world.location.is_indoors() {
                    let mut pt = Point16U { x: 0, y: 0 };
                    let attr = self.dungeon_lift_and_replace_liftable(&mut pt);
                    (attr, pt.x, pt.y)
                } else {
                    let mut pt = Point16U { x: 0, y: 0 };
                    let attr = self.overworld_handle_liftable_tiles(&mut pt);
                    (attr, pt.x, pt.y)
                };
                let Some(idx) = LINK_PERFORM_THROW_LIFTABLE_TILE_ATTR_TO_TERRAIN_TYPE
                    .iter()
                    .rposition(|&value| value == attr)
                else {
                    return;
                };
                self.follower_link_state_mut().set_sprite_pickup_flag(1);
                self.sprite_spawn_throwable_terrain(idx as u8, x, y);
                self.follower_link_state_mut()
                    .clear_filtered_joypad_l_bits(0x80);
                self.follower_link_state_mut().clear_action_handler_timer();
            }
        } else {
            self.follower_link_state_mut().clear_action_handler_timer();
        }

        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().set_y_button_action_timer(6);
        self.follower_link_state_mut().start_lift_throw_state();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_speed_setting(12);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().mask_direction(0xf0);
        self.follower_link_state_mut().set_direction_lock_bits(1);
    }

    pub(super) fn spawn_hammer_water_splash(&mut self) {
        if (self.game_state.frame.submodule
            | self.game_state.player.follower_link.immobilized_flag()
            | self.game_state.frame.modal_pause_flag)
            != 0
        {
            return;
        }
        let i = self.game_state.player.follower_link.facing_index();
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(SPAWN_HAMMER_WATER_SPLASH_HAMMER_WATER_X[i] as i16 as u16);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(SPAWN_HAMMER_WATER_SPLASH_HAMMER_WATER_Y[i] as i16 as u16);
        // The splash probe does not publish the sprite scratch tile; the
        // splash spawns at the pixel.
        let floor = self.game_state.player.follower_link.lower_level_state();
        let (tile, _) = self.entity_tile_at(floor, x, y);

        if tile == NativeTile::DEEP_WATER || tile == NativeTile::SHALLOW_WATER {
            let j = self.sprite_spawn_small_splash(0);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_x(j, x.wrapping_sub(8));
                self.sprite_set_y(j, y.wrapping_sub(16));
                let floor = self.game_state.player.follower_link.lower_level_state();
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_floor(floor);
                sprite.set_z(0);
            }
        }
    }

    pub(super) fn digging_game_guy_attempt_prize_spawn(&mut self) {
        self.digging_game_prize_mut().increment_attempts();
        if self.game_state.player.follower_link.y() >= 0x0b18 {
            return;
        }
        let j = self.get_random_number() & 7;
        let item_to_spawn = match j {
            0..=3 => DIGGING_GAME_GUY_ATTEMPT_PRIZE_SPAWN_DIGGING_GAME_ITEMS[j as usize],
            4 => {
                if self.game_state.effects.digging_game_prize.attempts() < 25
                    || self.game_state.effects.digging_game_prize.spawned_marker() != 0
                    || self.get_random_number() & 3 != 0
                {
                    return;
                }
                self.digging_game_prize_mut().mark_spawned();
                0xeb
            }
            _ => return,
        };

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(4, item_to_spawn, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = usize::from(self.game_state.player.follower_link.facing() != 4);
            {
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_x_velocity(DIGGING_GAME_GUY_ATTEMPT_PRIZE_SPAWN_DIGGING_GAME_XVEL[i]);
                sprite.set_y_velocity(0);
                sprite.set_z_velocity(24);
                sprite.set_stunned(255);
                sprite.set_delay_aux4(48);
            }
            let x =
                self.game_state.player.follower_link.x().wrapping_add(
                    DIGGING_GAME_GUY_ATTEMPT_PRIZE_SPAWN_DIGGING_GAME_X[i] as i16 as u16,
                ) & !0x0f;
            let y = self.game_state.player.follower_link.y().wrapping_add(22) & !0x0f;
            self.sprite_set_x(j, x);
            self.sprite_set_y(j, y);
            self.sprite_slot_view_mut(j).set_floor(0);
            self.sprite_sfx_queue_sfx3_with_pan(j, 0x30);
        }
    }
}
