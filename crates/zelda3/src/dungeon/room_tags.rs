//! Room tag dispatch, puzzles, switches, and moving walls.

use super::*;
use crate::game_state::ObjectRecord;
use crate::tile_definition::{DungeonRole, NativeTile, TilePair};

impl ZeldaState {

    /// ROM: `Dung_TagRoutine_0x22_0x3B`
    pub(super) fn dung_tag_routine_0x22_0x3b(&mut self, k: usize, j: u8) {
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0100 != 0 {
            self.dungeon_header_mut().clear_header_tag(k);
            self.dungeon_room_load_mut().set_overlay_to_load(j);
            self.dungeon_room_load_mut().set_load_ptr_offset(0);
            self.set_subsubmodule(0);
            self.set_sound_effect_2(0x1b);
            self.set_submodule(3);
        }
    }

    /// ROM: `Dung_TagRoutine_0x1B`
    pub(super) fn dung_tag_routine_0x1b(&mut self, _k: usize) {}

    /// ROM: `RoomTag_NorthWestTrigger`
    pub(super) fn room_tag_north_west_trigger(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 == 0
            && self.game_state.player.follower_link.y() & 0x0100 == 0
        {
            self.room_tag_quadrant_trigger(k);
        }
    }

    /// ROM: `RoomTag_Holes0`
    pub(super) fn room_tag_holes0(&mut self, _k: usize) {
        self.dung_tag_routine_func2(1);
    }

    /// ROM: `Dung_TagRoutine_0x23`
    pub(super) fn dung_tag_routine_0x23(&mut self, _k: usize) {
        self.dung_tag_routine_func2(3);
    }

    /// ROM: `Dung_TagRoutine_0x2A`
    pub(super) fn dung_tag_routine_0x2a(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 != 0
            && self.game_state.player.follower_link.y() & 0x0100 == 0
        {
            self.room_tag_quadrant_trigger(k);
        }
    }

    /// ROM: `Dung_TagRoutine_0x2B`
    pub(super) fn dung_tag_routine_0x2b(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 == 0
            && self.game_state.player.follower_link.y() & 0x0100 != 0
        {
            self.room_tag_quadrant_trigger(k);
        }
    }

    /// ROM: `Dung_TagRoutine_0x2C`
    pub(super) fn dung_tag_routine_0x2c(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 != 0
            && self.game_state.player.follower_link.y() & 0x0100 != 0
        {
            self.room_tag_quadrant_trigger(k);
        }
    }

    /// ROM: `Dung_TagRoutine_0x2D`
    pub(super) fn dung_tag_routine_0x2d(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 == 0 {
            self.room_tag_quadrant_trigger(k);
        }
    }

    /// ROM: `Dung_TagRoutine_0x2E`
    pub(super) fn dung_tag_routine_0x2e(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 != 0 {
            self.room_tag_quadrant_trigger(k);
        }
    }

    /// ROM: `Dung_TagRoutine_0x2F`
    pub(super) fn dung_tag_routine_0x2f(&mut self, k: usize) {
        if self.game_state.player.follower_link.y() & 0x0100 == 0 {
            self.room_tag_quadrant_trigger(k);
        }
    }

    /// ROM: `Dung_TagRoutine_0x30`
    pub(super) fn dung_tag_routine_0x30(&mut self, k: usize) {
        if self.game_state.player.follower_link.y() & 0x0100 != 0 {
            self.room_tag_quadrant_trigger(k);
        }
    }

    /// ROM: `Dung_TagRoutine_0x34`
    pub(super) fn dung_tag_routine_0x34(&mut self, _k: usize) {
        self.dung_tag_routine_func2(6);
    }

    /// ROM: `Dung_TagRoutine_0x35`
    pub(super) fn dung_tag_routine_0x35(&mut self, _k: usize) {
        self.dung_tag_routine_func2(8);
    }

    /// ROM: `Dung_TagRoutine_0x36`
    pub(super) fn dung_tag_routine_0x36(&mut self, _k: usize) {
        self.dung_tag_routine_func2(10);
    }

    /// ROM: `Dung_TagRoutine_0x37`
    pub(super) fn dung_tag_routine_0x37(&mut self, _k: usize) {
        self.dung_tag_routine_func2(12);
    }

    /// ROM: `Dung_TagRoutine_0x39`
    pub(super) fn dung_tag_routine_0x39(&mut self, _k: usize) {
        self.dung_tag_routine_func2(14);
    }

    /// ROM: `Dung_TagRoutine_0x3A`
    pub(super) fn dung_tag_routine_0x3a(&mut self, _k: usize) {
        self.dung_tag_routine_func2(16);
    }

    /// ROM: `Dung_TagRoutine_Func2`
    pub(super) fn dung_tag_routine_func2(&mut self, mut av: u8) {
        self.dungeon_room_load_mut()
            .set_overlay_to_load_if_empty(av);

        if let Some(star_untoggled) = self.room_tag_check_for_pressed_switch() {
            av = av.wrapping_add(u8::from(star_untoggled));
            if av != self.game_state.dungeon.room_load.overlay_to_load() {
                self.dungeon_room_load_mut().set_overlay_to_load(av);
                self.dungeon_room_load_mut().set_load_ptr_offset(0);
                self.set_subsubmodule(0);
                self.set_sound_effect_2(27);
                self.set_submodule(3);
                self.dungeon_room_effects_mut().toggle_star_tile_phase();
                self.dungeon_restore_star_tile_chr();
            }
        }
    }

    /// ROM: `RoomTag_ChestHoles0`
    pub(super) fn room_tag_chest_holes0(&mut self, k: usize) {
        self.dung_tag_routine_0x22_0x3b(k, 0);
    }

    /// ROM: `Dung_TagRoutine_0x3B`
    pub(super) fn dung_tag_routine_0x3b(&mut self, k: usize) {
        self.dung_tag_routine_0x22_0x3b(k, 0x12);
    }

    /// ROM: `RoomTag_Holes2`
    pub(super) fn room_tag_holes2(&mut self, k: usize) {
        if self.room_tag_check_for_pressed_switch().is_none() {
            return;
        }

        self.dungeon_header_mut().clear_header_tag(k);
        self.dungeon_room_load_mut().set_overlay_to_load(5);
        self.dungeon_room_load_mut().set_load_ptr_offset(0);
        self.set_subsubmodule(0);
        self.set_sound_effect_2(0x1b);
        self.set_submodule(3);
    }

    /// ROM: `RoomTag_QuadrantTrigger`
    pub(super) fn room_tag_quadrant_trigger(&mut self, k: usize) {
        let tag = self.game_state.dungeon.header.header_tag(k);
        if tag >= 0x0b {
            if tag >= 0x29 {
                if self.sprite_check_if_screen_is_clear() {
                    self.room_tag_operate_chest_reveal(k);
                }
            } else {
                let down = self
                    .game_state
                    .dungeon
                    .environment
                    .movable_block_was_not_pushed_flag();
                if down != self.game_state.dungeon.environment.trapdoors_down_low() {
                    self.dungeon_environment_mut().set_trapdoors_down_low(down);
                    self.set_sound_effect_2(37);
                    self.set_submodule(5);
                    self.dungeon_doors_mut().clear_current_door_pos();
                    self.dungeon_doors_mut().clear_door_animation_step();
                }
            }
        } else if self.sprite_check_if_screen_is_clear() {
            self.dung_tag_routine_trapdoors_up();
        }
    }

    /// ROM: `RoomTag_RoomTrigger`
    pub(super) fn room_tag_room_trigger(&mut self, k: usize) {
        if self.game_state.dungeon.header.header_tag(k) == 10 {
            if self.sprite_check_if_room_is_clear() {
                self.dung_tag_routine_trapdoors_up();
            }
        } else if self.sprite_check_if_room_is_clear() {
            self.room_tag_operate_chest_reveal(k);
        }
    }

    /// ROM: `RoomTag_RekillableBoss`
    pub(super) fn room_tag_rekillable_boss(&mut self, k: usize) {
        if self.sprite_check_if_room_is_clear() {
            self.follower_link_state_mut().clear_menu_block();
            self.dungeon_header_mut().clear_header_tag(k);
        }
    }

    /// ROM: `RoomTag_RoomTrigger_BlockDoor`
    pub(super) fn room_tag_room_trigger_block_door(&mut self, _k: usize) {
        if self
            .game_state
            .dungeon
            .environment
            .water_puzzle_state_changed()
            != 0
            && self.game_state.dungeon.environment.trapdoors_down() != 0
        {
            self.dungeon_environment_mut().clear_trapdoors_down();
            self.dungeon_doors_mut().clear_current_door_pos();
            self.dungeon_doors_mut().clear_door_animation_step();
            self.set_submodule(5);
        }
    }

    /// ROM: `RoomTag_PrizeTriggerDoorDoor`
    pub(super) fn room_tag_prize_trigger_door_door(&mut self, k: usize) {
        let prizes = if self.game_state.inventory.save_progress.dark_world_state() != 0 {
            self.game_state.inventory.player_resources.crystal_flags()
        } else {
            self.game_state.inventory.player_resources.pendant_flags()
        };
        let palace = (self.game_state.inventory.save_progress.palace_index_x2() >> 1) as usize;
        if prizes & DUNGEON_CRYSTAL_PENDANT_BITS[palace] != 0 {
            self.dungeon_environment_mut().clear_trapdoors_down();
            self.dungeon_doors_mut().clear_current_door_pos();
            self.dungeon_doors_mut().clear_door_animation_step();
            self.set_submodule(5);
            self.dungeon_header_mut().clear_header_tag(k);
        }
    }

    /// ROM: `RoomTag_TorchPuzzleDoor`
    pub(super) fn room_tag_torch_puzzle_door(&mut self, _k: usize) {
        let mut lit = 0;
        for i in 0..16 {
            if self
                .game_state
                .dungeon
                .object_tracking
                .object_tilemap_pos(i)
                & 0x8000
                != 0
            {
                lit += 1;
            }
        }
        let down = u16::from(lit < 4);
        if down != self.game_state.dungeon.environment.trapdoors_down() {
            self.dungeon_environment_mut().set_trapdoors_down(down);
            self.dungeon_doors_mut().clear_current_door_pos();
            self.dungeon_doors_mut().clear_door_animation_step();
            self.set_sound_effect_2(0x1b);
            self.set_submodule(5);
        }
    }

    /// ROM: `RoomTag_Switch_ExplodingWall`
    pub(super) fn room_tag_switch_exploding_wall(&mut self, k: usize) {
        if self.room_tag_maybe_check_shutters().is_some() {
            self.dung_tag_routine_blast_wall_stuff(k);
        }
    }

    /// ROM: `RoomTag_PullSwitchExplodingWall`
    pub(super) fn room_tag_pull_switch_exploding_wall(&mut self, k: usize) {
        if self
            .game_state
            .dungeon
            .environment
            .water_puzzle_state_changed()
            != 0
        {
            self.dung_tag_routine_blast_wall_stuff(k);
        }
    }

    /// ROM: `Dung_TagRoutine_BlastWallStuff`
    pub(super) fn dung_tag_routine_blast_wall_stuff(&mut self, k: usize) {
        self.dungeon_header_mut().clear_header_tag(k);

        let mut door = 0usize;
        while self.game_state.dungeon.doors.door_type_and_slot(door) & !1 != 0x30 {
            door += 1;
        }
        self.dungeon_room_effects_mut()
            .set_blast_wall_door_index(door);

        let mut i = (((self.game_state.player.follower_link.y() >> 8) & 1) + 1) * 2;
        if self.game_state.dungeon.doors.door_direction(door) & 2 != 0 {
            i = (self.game_state.player.follower_link.x() >> 8) & 1;
        }

        self.blast_wall_scratch_mut().set_direction(
            DUNG_TAG_ROUTINE_BLAST_WALL_STUFF_BLAST_WALL_MESSAGE_DIRECTION_BY_QUADRANT[i as usize],
        );
        let pos = self
            .game_state
            .dungeon
            .doors
            .door_tilemap_address(door)
            .wrapping_add(
                DUNG_TAG_ROUTINE_BLAST_WALL_STUFF_BLAST_WALL_DOOR_TILEMAP_OFFSETS[i as usize],
            );
        let x = ((pos & 0x007e) << 2)
            .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_h());
        let y = ((pos & 0x1f80) >> 4)
            .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_v());
        self.blast_wall_scratch_mut().set_center(x, y);
        self.set_sound_effect_2(27);
        self.dungeon_room_effects_mut()
            .set_crush_wall_progress_low(1);
        self.ancilla_add_blast_wall();
    }

    /// `Dungeon_DeleteRupeeTile` (dungeon.c): the rupee tile Link touched becomes floor.
    /// ROM: `Dungeon_DeleteRupeeTile`
    pub(in crate::zelda_rtl) fn dungeon_delete_rupee_tile(&mut self, x: u16, y: u16) {
        let pos = ((y & 0x01f8) * 8) | ((x & 0x01f8) >> 3);
        let dst = self.game_state.display.current_vram_upload_data_address();
        self.write_vram_upload_absolute_word(dst + 4, 0x190f);
        self.write_vram_upload_absolute_word(dst + 10, 0x190f);
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile(pos as usize, 0x190f);
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile((pos + 64) as usize, 0x190f);
        let attr = [self.dungeon_tile_definition(0x190f); 2];
        let vram0 = self.dungeon_map_vram_addr(pos);
        let vram1 = self.dungeon_map_vram_addr(pos + 64);
        self.dungeon_bg2_attributes_mut()
            .set_bg2_tiles(pos as usize, attr);
        self.dungeon_bg2_attributes_mut()
            .set_bg2_tiles((pos + 64) as usize, attr);
        self.write_vram_upload_absolute_word(dst, vram0);
        self.write_vram_upload_absolute_word(dst + 6, vram1);
        self.write_vram_upload_absolute_word(dst + 2, 0x0100);
        self.write_vram_upload_absolute_word(dst + 8, 0x0100);
        self.write_vram_upload_absolute_word(dst + 12, 0xffff);
        self.advance_vram_upload_cursor_by(24);
        self.dungeon_savegame_state_mut()
            .set_savegame_state_high_bits(0x10);
        self.set_bg_vram_load_mode(1);
    }

    /// ROM: `RoomTag_WaterOn`
    pub(super) fn room_tag_water_on(&mut self, _k: usize) {
        if self
            .game_state
            .dungeon
            .environment
            .water_puzzle_state_changed()
            != 0
        {
            self.set_sound_effect_2(0x1b);
            self.set_sound_effect_1(0x2f);
            self.set_submodule(12);
            self.set_subsubmodule(0);
            self.dungeon_moving_floor_mut().set_water_transition_counter(1);
            self.dungeon_header_mut().clear_header_tag(1);
            let save_bits = self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x0800;
            self.dungeon_savegame_state_mut()
                .set_savegame_state_bits(save_bits);
            self.dungeon_environment_mut()
                .clear_water_puzzle_state_changed();
            self.dungeon_room_load_mut().clear_quadrant_upload_index();
        }
    }

    /// ROM: `RoomTag_WaterOff`
    pub(super) fn room_tag_water_off(&mut self, _k: usize) {
        if self
            .game_state
            .dungeon
            .environment
            .water_puzzle_state_changed()
            == 0
        {
            return;
        }

        self.set_window_layer_masks(3, 0, 0, 22, 1);
        self.dungeon_moving_floor_mut()
            .set_water_transition_counter(1);
        self.AdjustWaterHDMAWindow();
        self.set_submodule(11);
        self.set_countdown(0);
        self.set_darkening_or_lightening_screen(0);
        self.set_mosaic_target_level(31);
        self.increment_cgram_update_flag();
        self.dungeon_header_mut().clear_header_tag(1);
        let save_bits = self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x0800;
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits(save_bits);
        self.dungeon_environment_mut()
            .clear_water_puzzle_state_changed();

        let dsto = ((self.game_state.display.water_hdma_window.window_y() & 0x01ff)
            .wrapping_sub(0x10)
            << 3)
            | ((self.game_state.display.water_hdma_window.window_x() & 0x01ff).wrapping_sub(0x10)
                >> 3);
        self.DrawWaterThing(dsto, 0x1438);
        self.dungeon_prep_overlay_dma_next_prep(0, dsto.wrapping_mul(2));
        self.set_sound_effect_2(0x1b);
        self.set_sound_effect_1(0x2e);
        self.request_nmi_copy_packets();
    }

    /// ROM: `RoomTag_WaterGate`
    pub(super) fn room_tag_water_gate(&mut self, _k: usize) {
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0800 != 0
            || self
                .game_state
                .dungeon
                .environment
                .water_puzzle_state_changed()
                == 0
        {
            return;
        }

        self.set_submodule(13);
        self.set_subsubmodule(0);
        self.dungeon_header_mut().clear_header_tag(1);
        let save_bits = self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x0800;
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits(save_bits);
        self.dungeon_environment_mut()
            .clear_water_puzzle_state_changed();
        self.water_hdma_window_mut().set_window_y_radius_byte(0);
        self.set_spotlight_window_y_buffer_byte(0);
        self.set_window_layer_masks(3, 0, 0, 0x16, 1);
        self.set_color_window_selection(2);
        self.set_color_math_control(0x62);
        self.set_overworld_event_bits(0x3b, 0x20);
        self.set_overworld_event_bits(0x7b, 0x20);
        let dung_info = self.saved_room_flags(0x28) | 0x0100;
        self.set_saved_room_flags(0x28, dung_info);

        self.room_tag_operate_water_flooring();
        let watergate_pos = self
            .game_state
            .display
            .water_hdma_window
            .watergate_tilemap_pos_x2();
        let hdma0 = ((watergate_pos & 0x007e) << 2)
            .wrapping_add(u16::from(self.game_state.dungeon.room_load.draw_width_indicator()) * 16)
            .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_h())
            .wrapping_add(40);
        self.water_hdma_window_mut().set_window_x(hdma0);
        let y = (watergate_pos & 0x1f80) >> 4;
        self.water_hdma_window_mut()
            .set_watergate_spotlight_y_upper(y);
        self.set_spotlight_y_upper(y);
        let hdma1 = y.wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_v());
        self.water_hdma_window_mut().set_window_y(hdma1);
        self.water_hdma_window_mut().set_window_x_radius(0);
        self.set_sound_effect_2(0x1b);
        self.set_sound_effect_1(0x2f);
    }

    /// ROM: `RoomTag_OperateWaterFlooring`
    pub(super) fn room_tag_operate_water_flooring(&mut self) {
        self.dungeon_room_load_mut().set_load_ptr_offset(0);
        let mut layout = 0usize;
        loop {
            self.dungeon_room_load_mut()
                .set_draw_width_indicator_word(0);
            self.dungeon_room_load_mut()
                .set_draw_height_indicator_word(0);
            let t = u16::from(WATERGATE_LAYOUT_BYTES[layout])
                | (u16::from(WATERGATE_LAYOUT_BYTES[layout + 1]) << 8);
            if t == 0xffff {
                break;
            }
            self.dungeon_room_load_mut()
                .set_draw_dimensions_words((t & 3) + 1, ((t >> 8) & 3) + 1);
            let load = self
                .game_state
                .dungeon
                .room_load
                .load_ptr_offset()
                .wrapping_add(3);
            self.dungeon_room_load_mut().set_load_ptr_offset(load);
            layout += 3;

            let mut dsto2 = ((t & 0x00fc) >> 2) | ((t >> 10) << 6);
            let mut height = self
                .game_state
                .dungeon
                .room_load
                .draw_height_indicator_word();
            while height != 0 {
                let mut dsto = dsto2;
                let mut width = self
                    .game_state
                    .dungeon
                    .room_load
                    .draw_width_indicator_word();
                while width != 0 {
                    for _ in 0..2 {
                        for y in 0..2u16 {
                            for x in 0..4u16 {
                                let tile = self.tile_word(0x0110, (y * 4 + x) as usize);
                                self.room_write_bg1(dsto + x + y * 64, tile);
                            }
                        }
                        dsto = dsto.wrapping_add(xy(0, 2) as u16);
                    }
                    dsto = dsto
                        .wrapping_add(xy(4, 0) as u16)
                        .wrapping_sub(xy(0, 4) as u16);
                    width -= 1;
                }
                dsto2 = dsto2.wrapping_add(xy(0, 4) as u16);
                height -= 1;
            }
        }
    }

    /// ROM: `RoomTag_GetHeartForPrize`
    pub(in crate::zelda_rtl) fn room_tag_get_heart_for_prize(&mut self, k: usize) {
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 == 0 {
            return;
        }
        let prizes = if self.game_state.inventory.save_progress.dark_world_state() != 0 {
            self.game_state.inventory.player_resources.crystal_flags()
        } else {
            self.game_state.inventory.player_resources.pendant_flags()
        };
        let palace = (self.game_state.inventory.save_progress.palace_index_x2() >> 1) as usize;
        if prizes & DUNGEON_CRYSTAL_PENDANT_BITS[palace] == 0 {
            self.world_transient_mut()
                .begin_boss_prize_graphics_countdown();
            if self.ancilla_spawn_falling_prize(
                ROOM_TAG_GET_HEART_FOR_PRIZE_BOSS_FINISHED_FALLING_ITEM[palace],
            ) < 0
            {
                return;
            }
        }
        self.dungeon_header_mut().clear_header_tag(k);
    }

    /// ROM: `RoomTag_Agahnim`
    pub(super) fn room_tag_agahnim(&mut self, _k: usize) {
        if self.game_state.world.overworld.event_info.event_info(0x5b) & 0x20 == 0
            && self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 != 0
        {
            self.palette_revert_translucency_swap();
            self.dungeon_header_mut().clear_header_tag(0);
            self.prepare_dungeon_exit_from_boss_fight();
        }
    }

    /// ROM: `RoomTag_GanonDoor`
    pub(super) fn room_tag_ganon_door(&mut self, _tagidx: usize) {
        for k in (0..16).rev() {
            let sprite = self.sprite_slot_view(k);
            if sprite.state() == 4 || (sprite.flags4() & 64 == 0 && sprite.state() != 0) {
                return;
            }
        }

        if self.game_state.player.follower_link.handler_state() != 1 {
            self.follower_link_state_mut().set_immobilized_flag(26);
            self.set_submodule(26);
            self.set_subsubmodule(0);
            self.dungeon_header_mut().clear_header_tag(0);
            self.follower_link_state_mut().force_hold_sword_up();
            self.follower_link_state_mut().set_button_mask_b_y(0);
            self.follower_link_state_mut().clear_button_b_frames();
            self.scratch_word_mut()
                .set_ganon_door_bounce_countdown(0x0364);
        }
    }

    /// ROM: `RoomTag_SwitchTrigger_HoldDoor`
    pub(super) fn room_tag_switch_trigger_hold_door(&mut self, _k: usize) {
        let mut i = 0usize;
        let end = self.game_state.dungeon.torch.torches_start_index() as usize;
        let down = loop {
            if i == end {
                break u16::from(
                    self.game_state
                        .dungeon
                        .environment
                        .somaria_block_switch_counter()
                        == 0
                        && self
                            .game_state
                            .dungeon
                            .environment
                            .water_puzzle_state_changed()
                            == 0
                        && self.room_tag_check_for_pressed_switch().is_none(),
                );
            }
            if self
                .game_state
                .dungeon
                .object_tracking
                .object_record(i >> 1)
                == ObjectRecord::RESTING_ON_PLATE
            {
                let value = self
                    .game_state
                    .dungeon
                    .environment
                    .block_trap_related_tile();
                if value != 0xffff {
                    break value;
                }
                break u16::from(
                    self.game_state
                        .dungeon
                        .environment
                        .somaria_block_switch_counter()
                        == 0
                        && self
                            .game_state
                            .dungeon
                            .environment
                            .water_puzzle_state_changed()
                            == 0
                        && self.room_tag_check_for_pressed_switch().is_none(),
                );
            }
            i += 2;
        };

        if down != self.game_state.dungeon.environment.trapdoors_down() {
            self.dungeon_environment_mut().set_trapdoors_down(down);
            self.dungeon_doors_mut().clear_current_door_pos();
            self.dungeon_doors_mut().clear_door_animation_step();
            if down == 0 {
                self.set_sound_effect_2(0x25);
            }
            self.set_submodule(5);
        }
    }

    /// ROM: `RoomTag_SwitchTrigger_ToggleDoor`
    pub(super) fn room_tag_switch_trigger_toggle_door(&mut self, _k: usize) {
        if !self.game_state.dungeon.doors.door_switch_triggered() {
            if let Some(switch) = self.room_tag_maybe_check_shutters() {
                self.dungeon_doors_mut().clear_current_door_pos();
                self.dungeon_doors_mut().clear_door_animation_step();
                self.set_sound_effect_2(0x25);
                self.PushPressurePlate(switch);
                let down = self.game_state.dungeon.environment.trapdoors_down() ^ 1;
                self.dungeon_environment_mut().set_trapdoors_down(down);
                self.dungeon_doors_mut().mark_door_switch_triggered();
            }
        } else if self.room_tag_maybe_check_shutters().is_none() {
            self.dungeon_doors_mut().clear_door_switch_triggered();
        }
    }

    pub(super) fn PushPressurePlate(&mut self, switch: NativeTile) {
        self.set_submodule(5);
        if switch == NativeTile::PRESSURE_PLATE
            || self
                .game_state
                .dungeon
                .room_items
                .replacement_tile_destination_x2()
                == 0
        {
            return;
        }
        self.save_submodule_for_menu();
        self.set_submodule(23);
        self.set_subsubmodule(32);
        let link_y = self.game_state.player.follower_link.y().wrapping_add(2);
        self.follower_link_state_mut().set_y(link_y);

        let mut pos = self
            .game_state
            .dungeon
            .room_items
            .replacement_tile_destination_x2();
        // The held switch's second cell must sit at the destination.
        if !matches!(
            self.game_state
                .dungeon
                .bg2_attributes
                .bg2_tiles(pos as usize)
                .0[1]
                .dungeon_role(),
            DungeonRole::FloorSwitch { variant: 1 | 2 }
        ) {
            pos = pos.wrapping_add(1);
            self.dungeon_room_items_mut()
                .set_replacement_tile_destination_x2(pos);
        }
        self.dungeon_update_tile_map_with_common_tile(
            (pos & 0x003f) << 3,
            (pos >> 3) & 0x01f8,
            0x10,
        );
    }

    /// ROM: `RoomTag_KillRoomBlock`
    pub(super) fn room_tag_kill_room_block(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 != 0
            && self.game_state.player.follower_link.y() & 0x0100 != 0
            && self.sprite_check_if_screen_is_clear()
        {
            self.set_sound_effect_2(0x1b);
            self.dungeon_header_mut().clear_header_tag(k);
        }
    }

    /// ROM: `RoomTag_PushBlockForChest`
    pub(super) fn room_tag_push_block_for_chest(&mut self, k: usize) {
        if self.game_state.display.bg_vram_load_mode == 0
            && self
                .game_state
                .dungeon
                .environment
                .movable_block_was_pushed()
                != 0
        {
            self.room_tag_operate_chest_reveal(k);
        }
    }

    /// ROM: `RoomTag_TriggerChest`
    pub(super) fn room_tag_trigger_chest(&mut self, k: usize) {
        if self.game_state.player.follower_link.blink_countdown() == 0
            && self.room_tag_maybe_check_shutters().is_some()
        {
            self.room_tag_operate_chest_reveal(k);
        }
    }

    /// ROM: `RoomTag_TorchPuzzleChest`
    pub(super) fn room_tag_torch_puzzle_chest(&mut self, k: usize) {
        let mut lit = 0;
        for i in 0..16 {
            if self
                .game_state
                .dungeon
                .object_tracking
                .object_tilemap_pos(i)
                & 0x8000
                != 0
            {
                lit += 1;
            }
        }
        if lit >= 4 {
            self.room_tag_operate_chest_reveal(k);
        }
    }

    /// ROM: `RoomTag_OperateChestReveal`
    pub(super) fn room_tag_operate_chest_reveal(&mut self, k: usize) {
        self.dungeon_header_mut().clear_header_tag(k);
        self.clear_vram_upload_cursor();
        self.set_chest_reveal_cursor_x2(0);

        let mut attr = NativeTile::import_pair(0x5858);
        loop {
            let yy = self.chest_reveal_cursor_x2();
            let pos = (self
                .game_state
                .dungeon
                .room_items
                .chest_location_for_cursor(yy)
                >> 1)
                & 0x1fff;

            self.dungeon_bg2_attributes_mut()
                .set_bg2_tiles(pos as usize, attr);
            self.dungeon_bg2_attributes_mut()
                .set_bg2_tiles((pos + 64) as usize, attr);
            attr = NativeTile::next_pair_identity(attr);

            let src = 0x149c;
            let tiles = [
                self.tile_word(src, 0),
                self.tile_word(src, 1),
                self.tile_word(src, 2),
                self.tile_word(src, 3),
            ];
            let positions = [pos, pos + 64, pos + 1, pos + 65];
            for (&tile_pos, &tile) in positions.iter().zip(tiles.iter()) {
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile(tile_pos as usize, tile);
            }

            let dst = self.game_state.display.current_vram_upload_data_address();
            for (i, (&offset, &tile)) in [0u16, 128, 2, 130].iter().zip(tiles.iter()).enumerate() {
                let stripe = self.room_tag_build_chest_stripes(offset, yy);
                let base = dst + i * 6;
                self.write_vram_upload_single_tile_stripe_packet(base, stripe, tile);
            }
            self.write_vram_upload_tile_stripe_sentinel(dst + 24);
            self.advance_vram_upload_cursor_by(24);

            let next = yy.wrapping_add(2);
            self.set_chest_reveal_cursor_x2(next);
            if self
                .game_state
                .dungeon
                .room_items
                .chest_reveal_cursor_reached_end(next)
            {
                break;
            }
        }

        self.set_chest_reveal_cursor_x2(0);
        self.set_sound_effect_2(26);
        self.set_bg_vram_load_mode(1);
    }

    /// ROM: `RoomTag_BuildChestStripes`
    pub(super) fn room_tag_build_chest_stripes(&self, pos: u16, y: u16) -> u16 {
        let loc = self
            .game_state
            .dungeon
            .room_items
            .chest_location_for_cursor(y);
        let pos = pos.wrapping_add(loc);
        (((pos & 0x0040) << 4) | ((pos & 0x303f) >> 1) | ((pos & 0x0f80) >> 2)).swap_bytes()
    }

    /// ROM: `RoomTag_GetTilemapCoords`
    pub(in crate::zelda_rtl) fn room_tag_get_tilemap_coords(&self) -> i32 {
        let pos = ((self.game_state.player.follower_link.x().wrapping_sub(1) & 0x01f8) >> 3)
            | ((self.game_state.player.follower_link.y().wrapping_add(14) & 0x01f8) << 3)
            | self
                .game_state
                .player
                .follower_link
                .lower_level_tilemap_offset();
        i32::from(pos)
    }

    /// The pressure plate or first held switch under the player, when its
    /// whole 2x2 cell agrees.
    /// ROM: `RoomTag_MaybeCheckShutters`
    pub(in crate::zelda_rtl) fn room_tag_maybe_check_shutters(&mut self) -> Option<NativeTile> {
        self.room_tag_find_switch(|role| {
            matches!(role, DungeonRole::FloorSwitch { variant: 0 | 1 })
        })
    }

    fn room_tag_find_switch(
        &mut self,
        accepts: impl Fn(DungeonRole) -> bool,
    ) -> Option<NativeTile> {
        self.dungeon_room_items_mut()
            .clear_replacement_tile_destination();
        if self.game_state.player.follower_link.is_immobilized()
            || self.game_state.player.follower_link.has_auxiliary_state()
        {
            return None;
        }

        let p = self.room_tag_get_tilemap_coords() as u16;
        let checks = [p, p.wrapping_add(64), p.wrapping_add(1), p.wrapping_add(65)];
        for &q in &checks {
            let pair = self.game_state.dungeon.bg2_attributes.bg2_tiles(q as usize);
            let Some(switch) = pair.uniform().filter(|tile| accepts(tile.dungeon_role())) else {
                continue;
            };
            if pair
                != self
                    .game_state
                    .dungeon
                    .bg2_attributes
                    .bg2_tiles((q + 64) as usize)
            {
                return None;
            }
            self.dungeon_room_items_mut()
                .set_replacement_tile_destination_x2(q);
            return Some(switch);
        }
        None
    }

    /// The pressure plate or star switch under the player; `Some(true)`
    /// names an untoggled star switch.
    /// ROM: `RoomTag_CheckForPressedSwitch`
    pub(in crate::zelda_rtl) fn room_tag_check_for_pressed_switch(&mut self) -> Option<bool> {
        self.room_tag_find_switch(|role| {
            matches!(
                role,
                DungeonRole::FloorSwitch { variant: 0 } | DungeonRole::StarSwitch { .. }
            )
        })
        .map(|switch| switch == NativeTile::STAR_SWITCH)
    }

    /// ROM: `Dungeon_SetAttrForActivatedWaterOff`
    pub(super) fn dungeon_set_attr_for_activated_water_off(&mut self) {
        self.set_color_window_selection(2);
        self.set_color_math_control(0x32);
        // ROM $01:EF9D `STZ $212D`: the routine disables the subscreen on the
        // live PPU immediately, before clearing the TS copy, so the pool's
        // drained floor is visible in the same frame (route f489892). The C
        // port only writes the copy, which reaches the register one NMI later.
        self.zelda_ppu_write(0x212d, 0);
        self.set_sub_screen_layers(0);
        self.set_bg12_window_selection(0);
        self.dungeon_room_load_mut().set_header_collision(0);
        self.clear_window_main_sub_masks();

        let mut j = 0;
        while j
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InRoomUpNorthWater)
        {
            let dsto = self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_tilemap_pos(DungeonStairList::InRoomUpNorthWater, j);
            self.write_attr2(
                dsto as usize + xy(1, 1),
                TilePair::repeated(NativeTile::IN_ROOM_STAIR_PSEUDO_UP_NORTH),
            );
            self.write_attr2(
                dsto as usize + xy(1, 2),
                TilePair::repeated(NativeTile::IN_ROOM_STAIR_PSEUDO_UP_NORTH),
            );
            j += 2;
        }

        let mut j = 0;
        while j
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InRoomUpSouthWater)
        {
            let dsto = self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_tilemap_pos(DungeonStairList::InRoomUpSouthWater, j);
            self.write_attr2(
                dsto as usize + xy(1, 1),
                TilePair::repeated(NativeTile::IN_ROOM_STAIR_PSEUDO_UP_NORTH),
            );
            self.write_attr2(
                dsto as usize + xy(1, 2),
                TilePair::repeated(NativeTile::IN_ROOM_STAIR_PSEUDO_UP_NORTH),
            );
            j += 2;
        }

        self.increment_cgram_update_flag();
        self.increment_subsubmodule();
    }

    /// ROM: `Dungeon_SetAttrForActivatedWater`
    pub(super) fn dungeon_set_attr_for_activated_water(&mut self) {
        self.clear_window_main_sub_masks();

        let mut j = 0;
        while j
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterPseudoUpNorth)
        {
            let dsto = self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_tilemap_pos(DungeonStairList::InterPseudoUpNorth, j)
                as usize;
            self.write_attr2(dsto, TilePair([NativeTile::LAYER_WALL, NativeTile::GROUND]));
            self.write_attr2(
                dsto + 2,
                TilePair([NativeTile::GROUND, NativeTile::LAYER_WALL]),
            );
            self.write_attr1(
                dsto,
                TilePair([NativeTile::LAYER_WALL, NativeTile::LAYER_LANDING]),
            );
            self.write_attr1(
                dsto + 2,
                TilePair([NativeTile::LAYER_LANDING, NativeTile::LAYER_WALL]),
            );
            self.write_attr2(dsto + xy(0, 1), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr2(dsto + xy(2, 1), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + xy(0, 1), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + xy(2, 1), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + xy(0, 2), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + xy(2, 2), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + xy(0, 3), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + xy(2, 3), TilePair::repeated(NativeTile::DEEP_WATER));
            j += 2;
        }

        let mut j = 0;
        while j
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WetStairs)
        {
            let dsto = self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_tilemap_pos(DungeonStairList::WetStairs, j)
                as usize;
            self.write_attr2(
                dsto + xy(0, 3),
                TilePair([NativeTile::LAYER_WALL, NativeTile::GROUND]),
            );
            self.write_attr2(
                dsto + xy(2, 3),
                TilePair([NativeTile::GROUND, NativeTile::LAYER_WALL]),
            );
            self.write_attr1(
                dsto + xy(0, 3),
                TilePair([NativeTile::LAYER_WALL, NativeTile::LAYER_LANDING]),
            );
            self.write_attr1(
                dsto + xy(2, 3),
                TilePair([NativeTile::LAYER_LANDING, NativeTile::LAYER_WALL]),
            );
            self.write_attr2(dsto + xy(0, 2), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr2(dsto + xy(2, 2), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto, TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + 2, TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + xy(0, 1), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + xy(2, 1), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + xy(0, 2), TilePair::repeated(NativeTile::DEEP_WATER));
            self.write_attr1(dsto + xy(2, 2), TilePair::repeated(NativeTile::DEEP_WATER));
            j += 2;
        }

        self.set_submodule(0);
        self.clear_nmi_update_latch();
        self.set_subsubmodule(0);
    }

    /// ROM: `Sprite_HandlePushedBlocks_One`
    pub(super) fn sprite_handle_pushed_blocks_one(&mut self, i: usize) {
        self.sprite_dungeon_draw_single_push_block(i * 2);
    }

    /// ROM: `RoomTag_MovingWall_East`
    pub(super) fn room_tag_moving_wall_east(&mut self, k: usize) {
        if self.game_state.dungeon.moving_floor.floor_move_flags() == 0 {
            self.room_tag_moving_wall_torches_check(k);
            self.dungeon_moving_floor_mut().set_floor_x_velocity(0);
        } else {
            self.set_modal_pause_flag(1);
            self.room_tag_moving_wall_shake_it_up(k);
            let vel = self.moving_wall_move_slightly();
            self.dungeon_moving_floor_mut().set_floor_x_velocity(vel);
        }

        let floor_x_velocity = self.game_state.dungeon.moving_floor.floor_x_velocity();
        let offs = self
            .dungeon_moving_floor_mut()
            .sub_floor_x_offset(floor_x_velocity);
        let bg1 = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_add(offs);
        self.set_bg1_x(bg1);

        if self.game_state.dungeon.moving_floor.floor_x_velocity() != 0 {
            let target0 = ROOM_TAG_MOVING_WALL_EAST_MOVING_WALL_EAST_TARGET_OFFSETS
                [self.game_state.dungeon.room_effects.moving_wall_dot_index()];
            if offs < target0 {
                let target1 = ROOM_TAG_MOVING_WALL_EAST_MOVING_WALL_EAST_TARGET_OFFSETS
                    [(self.room_tag_advance_gigantic_wall(k) >> 1) as usize & 7];
                if offs < target1 {
                    self.finish_moving_wall_tag(k);
                }
            }
            self.set_pending_nmi_subroutine(5);
            let neg = (0u16.wrapping_sub(offs) & 0x01f8) >> 3;
            let target = self
                .game_state
                .dungeon
                .room_effects
                .moving_wall_write_point()
                .wrapping_sub(neg)
                & 0x141f;
            self.set_nmi_load_target_address(target);
        }
    }

    /// ROM: `RoomTag_MovingWallShakeItUp`
    pub(super) fn room_tag_moving_wall_shake_it_up(&mut self, k: usize) {
        let x = if self.game_state.frame.frame_counter & 1 != 0 {
            -1i16
        } else {
            1
        };
        self.set_bg1_x_offset(x as u16);
        self.set_bg1_y_offset((-x) as u16);
        if self.game_state.dungeon.header.header_tag(k) == 0 {
            self.set_bg1_x_offset(0);
            self.set_bg1_y_offset(0);
        }
    }

    /// ROM: `RoomTag_MovingWall_West`
    pub(super) fn room_tag_moving_wall_west(&mut self, k: usize) {
        if self.game_state.dungeon.moving_floor.floor_move_flags() == 0 {
            self.room_tag_moving_wall_torches_check(k);
            self.dungeon_moving_floor_mut().set_floor_x_velocity(0);
        } else {
            self.set_modal_pause_flag(1);
            self.room_tag_moving_wall_shake_it_up(k);
            let vel = self.moving_wall_move_slightly();
            self.dungeon_moving_floor_mut().set_floor_x_velocity(vel);
        }

        let floor_x_velocity = self.game_state.dungeon.moving_floor.floor_x_velocity();
        let offs = self
            .dungeon_moving_floor_mut()
            .add_floor_x_offset(floor_x_velocity);
        let bg1 = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_add(offs);
        self.set_bg1_x(bg1);

        if self.game_state.dungeon.moving_floor.floor_x_velocity() != 0 {
            let target0 = ROOM_TAG_MOVING_WALL_WEST_MOVING_WALL_WEST_TARGET_OFFSETS
                [self.game_state.dungeon.room_effects.moving_wall_dot_index()];
            if offs >= target0 {
                let target1 = ROOM_TAG_MOVING_WALL_WEST_MOVING_WALL_WEST_TARGET_OFFSETS
                    [(self.room_tag_advance_gigantic_wall(k) >> 1) as usize & 7];
                if offs >= target1 {
                    self.finish_moving_wall_tag(k);
                }
            }
            self.set_pending_nmi_subroutine(5);
            let mut target = self
                .game_state
                .dungeon
                .room_effects
                .moving_wall_write_point()
                .wrapping_add((offs & 0x01f8) >> 3);
            if target & 0x1020 != 0 {
                target = (target & 0x1020) ^ 0x0420;
            }
            self.set_nmi_load_target_address(target);
        }
    }

    fn finish_moving_wall_tag(&mut self, k: usize) {
        self.set_sound_effect_2(0x1b);
        self.set_ambient_sound_effect(5);
        self.dungeon_header_mut().clear_header_tag(k);
        self.follower_link_state_mut().clear_immobilized();
        self.clear_modal_pause_flag();
        self.set_bg1_x_offset(0);
        self.set_bg1_y_offset(0);
    }

    /// ROM: `RoomTag_MovingWallTorchesCheck`
    pub(super) fn room_tag_moving_wall_torches_check(&mut self, k: usize) {
        if u16::from(
            self.game_state
                .dungeon
                .environment
                .water_puzzle_state_changed(),
        ) == 0
        {
            let mut count = 0;
            for i in 0..16 {
                count += u8::from(
                    self.game_state
                        .dungeon
                        .object_tracking
                        .object_tilemap_pos(i)
                        & 0x8000
                        != 0,
                );
            }
            if count < 4 {
                return;
            }
        }
        let flags = self
            .game_state
            .dungeon
            .moving_floor
            .floor_move_flags()
            .wrapping_add(1);
        self.dungeon_moving_floor_mut().set_floor_move_flags(flags);
        self.dungeon_environment_mut()
            .clear_water_puzzle_state_changed();
        let save_bits =
            self.game_state.dungeon.savegame_state.savegame_state_bits() | (0x1000 >> k);
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits(save_bits);
        self.set_ambient_sound_effect(7);
        self.follower_link_state_mut().immobilize();
        self.set_modal_pause_flag(1);
    }

    /// ROM: `MovingWall_MoveALittle`
    pub(super) fn moving_wall_move_slightly(&mut self) -> u16 {
        let t = self.bg1_move_calc_mut().advance_x_subpixel(0x22);
        t >> 8
    }

    /// ROM: `RoomTag_AdvanceGiganticWall`
    pub(super) fn room_tag_advance_gigantic_wall(&mut self, k: usize) -> u8 {
        let mut i = self
            .game_state
            .dungeon
            .room_effects
            .moving_wall_dot_pointer();
        if self.game_state.dungeon.header.header_tag(k) < 0x20 {
            self.dungeon_room_load_mut().set_header_collision(0);
            self.set_main_screen_layers(0x16);
            i = i.wrapping_add(8);
        }
        i
    }

    pub(in crate::zelda_rtl) fn dungeon_handle_room_tags(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x01_c2fd);
        crate::cycle_ledger::charge(48); // LDA $04C7 : BNE.
        if self.game_state.dungeon.room_runtime.should_run_room_tags() {
            crate::cycle_ledger::charge(68); // SEP #$30 : JSR DetectStaircase.
            self.dungeon_detect_staircase();

            self.dungeon_handle_room_tags_after_staircase();
            return;
        } else {
            crate::cycle_ledger::charge(6); // Taken BNE skips both tag calls.
        }
        crate::cycle_ledger::charge(76); // STZ $04C7 : RTL.
        self.dungeon_room_runtime_mut().clear_room_tag_skip();
    }

    fn dungeon_handle_room_tags_after_staircase(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x01_c2fd);
        if self
            .game_state
            .enhanced_features
            .has(FEATURE_MISC_BUG_FIXES_DUNGEON)
            && self.game_state.frame.submodule != 0
        {
            return;
        }

        // C writes the tag-routine index via `ram[R14] = k` as a BYTE (0x0e only); the
        // high byte 0x0f (R15) is left intact. Use the low-byte setter so we don't clobber
        // R15's stale leftover (collision_bits stays a u16 everywhere else).
        crate::cycle_ledger::charge(138); // STZ $0E : LDA $AE : ASL : TAX : JSR (table,X).
        self.tile_detect_position_mut()
            .set_collision_bits_low_byte(0);
        self.dungeon_run_tag_routine(0);
        crate::cycle_ledger::charge(154); // LDA #1 : STA $0E : LDA $AF : ASL : TAX : JSR.
        self.tile_detect_position_mut()
            .set_collision_bits_low_byte(1);
        self.dungeon_run_tag_routine(1);
        crate::cycle_ledger::charge(76);
        self.dungeon_room_runtime_mut().clear_room_tag_skip();
    }

    pub(super) fn finish_native_room_tags(
        &mut self,
        tail: crate::cycle_models::dungeon_room_tags::RoomTagTail,
    ) {
        use crate::cycle_models::dungeon_room_tags::RoomTagTail;
        match tail {
            RoomTagTail::Lamp => return,
            RoomTagTail::Returned => {}
            RoomTagTail::Staircase { head, landing, .. } => {
                self.dungeon_detect_staircase_after_landing(
                    NativeTile::from_cartridge(head),
                    landing,
                );
                self.dungeon_handle_room_tags_after_staircase();
            }
            RoomTagTail::Tag { index, .. } => {
                self.dungeon_run_tag_routine(usize::from(index));
                if index == 0 {
                    crate::cycle_ledger::charge(154);
                    self.tile_detect_position_mut()
                        .set_collision_bits_low_byte(1);
                    self.dungeon_run_tag_routine(1);
                }
                crate::cycle_ledger::charge(76);
                self.dungeon_room_runtime_mut().clear_room_tag_skip();
            }
        }
        self.replay_trace_ram_watch("module07-after-room-tags");
        self.complete_module07_room_maintenance_after_room_tags();
    }

    fn dungeon_run_tag_routine(&mut self, k: usize) {
        match self.game_state.dungeon.header.header_tag(k) {
            0x00 => self.dung_tag_routine_0x00(k),
            0x01 | 0x0b | 0x29 => self.room_tag_north_west_trigger(k),
            0x02 | 0x0c | 0x2a => self.dung_tag_routine_0x2a(k),
            0x03 | 0x0d | 0x2b => self.dung_tag_routine_0x2b(k),
            0x04 | 0x0e | 0x2c => self.dung_tag_routine_0x2c(k),
            0x05 | 0x0f | 0x2d => self.dung_tag_routine_0x2d(k),
            0x06 | 0x10 | 0x2e => self.dung_tag_routine_0x2e(k),
            0x07 | 0x11 | 0x2f => self.dung_tag_routine_0x2f(k),
            0x08 | 0x12 | 0x30 => self.dung_tag_routine_0x30(k),
            0x09 | 0x13 | 0x31 => self.room_tag_quadrant_trigger(k),
            0x0a | 0x32 => self.room_tag_room_trigger(k),
            0x14 => self.room_tag_room_trigger_block_door(k),
            0x15 => self.room_tag_prize_trigger_door_door(k),
            0x16 => self.room_tag_switch_trigger_hold_door(k),
            0x17 => self.room_tag_switch_trigger_toggle_door(k),
            0x18 => self.room_tag_water_off(k),
            0x19 => self.room_tag_water_on(k),
            0x1a => self.room_tag_water_gate(k),
            0x1b => self.dung_tag_routine_0x1b(k),
            0x1c => self.room_tag_moving_wall_east(k),
            0x1d => self.room_tag_moving_wall_west(k),
            0x1e | 0x1f => self.room_tag_moving_wall_torches_check(k),
            0x20 => self.room_tag_switch_exploding_wall(k),
            0x21 => self.room_tag_holes0(k),
            0x22 => self.room_tag_chest_holes0(k),
            0x23 => self.dung_tag_routine_0x23(k),
            0x24 => self.room_tag_holes2(k),
            0x25 => self.room_tag_get_heart_for_prize(k),
            0x26 => self.room_tag_kill_room_block(k),
            0x27 => self.room_tag_trigger_chest(k),
            0x28 => self.room_tag_pull_switch_exploding_wall(k),
            0x33 => self.room_tag_torch_puzzle_door(k),
            0x34 => self.dung_tag_routine_0x34(k),
            0x35 => self.dung_tag_routine_0x35(k),
            0x36 => self.dung_tag_routine_0x36(k),
            0x37 => self.dung_tag_routine_0x37(k),
            0x38 => self.room_tag_agahnim(k),
            0x39 => self.dung_tag_routine_0x39(k),
            0x3a => self.dung_tag_routine_0x3a(k),
            0x3b => self.dung_tag_routine_0x3b(k),
            0x3c => self.room_tag_push_block_for_chest(k),
            0x3d => self.room_tag_ganon_door(k),
            0x3e => self.room_tag_torch_puzzle_chest(k),
            0x3f => self.room_tag_rekillable_boss(k),
            _ => {}
        }
    }
}
