//! Door drawing, opening, key interactions, and exploding walls.

use super::*;
use crate::tile_definition::{DungeonRole, NativeTile, TilePair};

impl ZeldaState {

    /// ROM: `RoomData_DrawObject_Door`
    pub(super) fn room_data_draw_object_door(&mut self, raw: u16) {
        let door_type = (raw >> 8) as u8;
        let position = ((raw >> 4) & 0x0f) as usize;
        match raw & 3 {
            0 => self.room_draw_door_north(door_type, position),
            1 => self.room_draw_door_south(door_type, position),
            2 => self.room_draw_door_west(door_type, position),
            3 => self.room_draw_door_east(door_type, position),
            _ => unreachable!(),
        }
    }

    /// ROM: `RoomDraw_Door_North`
    pub(super) fn room_draw_door_north(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_UP[position] / 2;
        match door_type {
            DOOR_TYPE_LG_EXPLOSION => self.room_draw_door_exploding_wall(position),
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.room_draw_mark_layer_toggle_door(dsto.wrapping_sub(0xfe / 2));
            }
            DOOR_TYPE_SLASHABLE => self.room_draw_north_curtain_door(dsto),
            DOOR_TYPE_ENTRANCE_DOOR => self.door_up_entrance_door(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.room_draw_mark_dungeon_toggle_door(dsto.wrapping_sub(0xfe / 2))
            }
            DOOR_TYPE_REGULAR2 => {
                self.room_draw_make_door_parts_high_priority_y(dsto & (0xf07f / 2));
                self.room_draw_normal_ranged_doors_north(door_type, dsto, position);
            }
            DOOR_TYPE_EXIT_TO_OW => self.room_draw_register_exit_door(dsto),
            DOOR_TYPE_WATERFALL_TUNNEL => {
                self.room_draw_normal_ranged_doors_north(door_type, dsto, position);
                self.door_prioritize_cur_door();
            }
            t if (DOOR_TYPE_STAIR_MASK_LOCKED0..=DOOR_TYPE_STAIR_MASK_LOCKED3).contains(&t) => {
                self.door_up_stair_mask_locked(door_type, dsto);
            }
            t if (DOOR_TYPE_REGULAR_DOOR33..).contains(&t) => {
                self.room_draw_high_range_door_north(door_type, dsto, position);
            }
            _ => self.room_draw_normal_ranged_doors_north(door_type, dsto, position),
        }
    }

    /// ROM: `RoomDraw_Door_South`
    pub(super) fn room_draw_door_south(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_DOWN[position] / 2;
        match door_type {
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.room_draw_mark_layer_toggle_door(dsto + xy(1, 4) as u16);
            }
            DOOR_TYPE_ENTRANCE_DOOR => self.door_down_entrance_door(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.room_draw_mark_dungeon_toggle_door(dsto + xy(1, 4) as u16);
            }
            DOOR_TYPE_EXIT_TO_OW => self.room_draw_register_exit_door(dsto),
            t if t >= DOOR_TYPE_REGULAR_DOOR33 => {
                self.room_draw_one_sided_lower_shutters_south(door_type, dsto);
            }
            DOOR_TYPE_ENTRANCE_LARGE => {
                self.room_draw_flag_doors_and_get_final_type(1, door_type, dsto);
                self.room_draw_some_big_decors(10, 0x2656, dsto.wrapping_sub(3 + 4 * 64));
            }
            DOOR_TYPE_ENTRANCE_LARGE2 => {
                let mut dsto = dsto | 0x1000;
                self.room_draw_flag_doors_and_get_final_type(1, door_type, dsto);
                dsto = dsto.wrapping_sub(3 + 4 * 64);
                self.room_draw_some_big_decors(10, 0x2656, dsto);
                dsto = dsto.wrapping_sub(0x1000).wrapping_add(7 * 64);
                for i in 0..10 {
                    let tile = self.room_read_bg1(dsto + i) | 0x2000;
                    self.room_write_bg2(dsto + i, tile);
                }
            }
            DOOR_TYPE_ENTRANCE_CAVE | DOOR_TYPE_ENTRANCE_CAVE2 => {
                if door_type == DOOR_TYPE_ENTRANCE_CAVE2 {
                    self.room_draw_make_door_parts_high_priority_y(dsto + xy(0, 4) as u16);
                }
                self.room_draw_flag_doors_and_get_final_type(1, door_type, dsto);
                // RoomDraw_Door_South $01AA48: JMP RoomDraw_4x4 (shared tail).
                self.room_draw_4x4_by_jump(0x26f6, dsto);
            }
            DOOR_TYPE_4 => {
                let high_dsto = dsto | 0x1000;
                self.room_draw_make_door_parts_high_priority_y(high_dsto + xy(0, 4) as u16);
                self.room_draw_flag_doors_and_get_final_type(1, door_type, high_dsto);
                // RoomDraw_Door_South $01AA48: JMP RoomDraw_4x4 (shared tail).
                self.room_draw_4x4_by_jump(0x26f6, high_dsto);
                for i in 0..4 {
                    let pos = dsto + i + xy(0, 3) as u16;
                    let tile = self.room_read_bg1(pos) | 0x2000;
                    self.room_write_bg2(pos, tile);
                }
            }
            _ => self.room_draw_check_if_lower_layer_doors_y(door_type, dsto),
        }
    }

    /// ROM: `RoomDraw_Door_West`
    pub(super) fn room_draw_door_west(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_LEFT[position] / 2;
        match door_type {
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.room_draw_mark_layer_toggle_door(dsto.wrapping_add(62));
            }
            DOOR_TYPE_ENTRANCE_DOOR => self.door_left_entrance_door(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.room_draw_mark_dungeon_toggle_door(dsto.wrapping_add(62));
            }
            DOOR_TYPE_REGULAR2 => {
                self.room_draw_make_door_parts_high_priority_x(dsto & !0x1f);
                self.room_draw_normal_ranged_doors_west(door_type, dsto, position);
            }
            DOOR_TYPE_WATERFALL_TUNNEL => {
                self.room_draw_normal_ranged_doors_west(door_type, dsto, position);
                self.door_prioritize_cur_door();
            }
            t if t < DOOR_TYPE_REGULAR_DOOR33 => {
                self.room_draw_normal_ranged_doors_west(door_type, dsto, position);
            }
            _ => self.room_draw_high_range_door_west(door_type, dsto, position),
        }
    }

    /// ROM: `RoomDraw_Door_East`
    pub(super) fn room_draw_door_east(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_RIGHT[position] / 2;
        match door_type {
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.room_draw_mark_layer_toggle_door(dsto + xy(4, 1) as u16);
            }
            DOOR_TYPE_ENTRANCE_DOOR => self.door_right_entrance_door(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.room_draw_mark_dungeon_toggle_door(dsto + xy(4, 1) as u16);
            }
            t if t < DOOR_TYPE_REGULAR_DOOR33 => {
                self.room_draw_normal_ranged_doors_east(door_type, dsto)
            }
            _ => self.room_draw_one_sided_lower_shutters_east(door_type, dsto),
        }
    }

    pub(super) fn room_draw_register_exit_door(&mut self, dsto: u16) {
        self.dungeon_room_doors_mut()
            .append_exit_door_address(dsto * 2);
    }

    /// ROM: `RoomDraw_NormalRangedDoors_North`
    pub(super) fn room_draw_normal_ranged_doors_north(
        &mut self,
        door_type: u8,
        dsto: u16,
        position: usize,
    ) {
        if position >= 6 {
            if let Some(&down_dsto_bytes) = DOOR_POSITION_DOWN.get(position - 6) {
                let saved = self.game_state.dungeon.doors.current_door_index();
                self.dungeon_doors_mut()
                    .set_current_door_index(saved | 0x10);
                self.room_draw_check_if_lower_layer_doors_y(door_type, down_dsto_bytes / 2);
                self.dungeon_doors_mut().set_current_door_index(saved);
            }
        }
        self.room_draw_one_sided_shutters_north(door_type, dsto);
    }

    /// ROM: `RoomDraw_OneSidedShutters_North`
    pub(super) fn room_draw_one_sided_shutters_north(&mut self, door_type: u8, dsto: u16) {
        let mut final_type = self.room_draw_flag_doors_and_get_final_type(0, door_type, dsto);
        if final_type & 0x100 != 0 {
            return;
        }
        if final_type as u8 == DOOR_TYPE_36 || final_type as u8 == DOOR_TYPE_38 {
            final_type = if final_type as u8 == DOOR_TYPE_36 {
                DOOR_TYPE_SHUTTERS_TWO_WAY
            } else {
                DOOR_TYPE_REGULAR
            } as u16;
            self.room_rewrite_last_door_type(final_type as u8);
        }
        if let Some(&src) = DOOR_TYPE_SRC_UP.get(final_type as usize >> 1) {
            self.room_data_draw_object_door_up_4x3(src as usize, dsto);
        }
    }

    /// ROM: `Door_Up_StairMaskLocked`
    pub(super) fn door_up_stair_mask_locked(&mut self, door_type: u8, mut dsto: u16) {
        let door = self.game_state.dungeon.doors.current_door_index() as usize >> 1;
        self.dungeon_doors_mut().set_door_direction_word(door, 0);
        self.dungeon_doors_mut()
            .set_door_tilemap_address(door, dsto * 2);
        self.dungeon_doors_mut()
            .set_door_type_word(door, ((door as u16) << 8) | door_type as u16);
        if self
            .game_state
            .dungeon
            .doors
            .opened_doors_including_adjacent()
            & upper_bitmask(door & 7)
            != 0
        {
            let next = self
                .game_state
                .dungeon
                .doors
                .current_door_index()
                .wrapping_add(2);
            self.dungeon_doors_mut().set_current_door_index(next);
            return;
        }
        if door_type < DOOR_TYPE_STAIR_MASK_LOCKED2 {
            self.room_draw_one_sided_shutters_north(door_type, dsto);
            return;
        }

        let t = self.room_draw_flag_doors_and_get_final_type(0, door_type, dsto) as usize;
        let src = DOOR_TYPE_SRC_UP.get(t >> 1).copied().unwrap_or(0) as usize;
        for i in 0..4u16 {
            self.room_write_bg(
                0x4000,
                dsto + xy(0, 0) as u16,
                self.tile_word(src, (i * 3) as usize),
            );
            self.room_write_bg(
                0x4000,
                dsto + xy(0, 1) as u16,
                self.tile_word(src, (i * 3 + 1) as usize),
            );
            self.room_write_bg(
                0x4000,
                dsto + xy(0, 2) as u16,
                self.tile_word(src, (i * 3 + 2) as usize),
            );
            dsto = dsto.wrapping_add(1);
        }
        self.door_prioritize_cur_door();
    }

    /// ROM: `RoomDraw_Door_ExplodingWall`
    pub(super) fn room_draw_door_exploding_wall(&mut self, pos_enum: usize) {
        let dsto = DOOR_BLAST_WALL_UP_DESTINATIONS
            .get(pos_enum)
            .copied()
            .unwrap_or(0)
            / 2;
        let door = self.game_state.dungeon.doors.current_door_index() as usize >> 1;
        self.dungeon_doors_mut()
            .set_door_tilemap_address(door, 2 * (dsto + 10));
        self.dungeon_doors_mut()
            .set_door_type_word(door, ((door as u16) << 8) | DOOR_TYPE_LG_EXPLOSION as u16);
        if self
            .game_state
            .dungeon
            .doors
            .opened_doors_including_adjacent()
            & upper_bitmask(door & 7)
            == 0
        {
            self.dungeon_doors_mut().set_door_direction_word(door, 0);
            let next = self
                .game_state
                .dungeon
                .doors
                .current_door_index()
                .wrapping_add(2);
            self.dungeon_doors_mut().set_current_door_index(next);
            return;
        }

        let slot = usize::from(
            self.game_state.dungeon.header.primary_header_tag() != 0x20
                && self.game_state.dungeon.header.primary_header_tag() != 0x25
                && self.game_state.dungeon.header.primary_header_tag() != 0x28,
        );
        self.dungeon_header_mut().clear_header_tag(slot);
        self.force_vertical_fullsize_for_blast_wall();
        self.dungeon_room_effects_mut().mark_blast_wall_y_open();
        self.room_draw_exploding_wall_segment(DOOR_TYPE_SRC_DOWN[42] as usize, dsto);
        let next = self
            .game_state
            .dungeon
            .doors
            .current_door_index()
            .wrapping_add(2);
        self.dungeon_doors_mut().set_current_door_index(next);
        self.dungeon_room_doors_mut()
            .add_reset_xy_check_flags(0x0200);
        self.room_draw_exploding_wall_segment(DOOR_TYPE_SRC_UP[42] as usize, dsto + xy(0, 6) as u16);
    }

    /// ROM: `RoomDraw_ExplodingWallSegment`
    pub(super) fn room_draw_exploding_wall_segment(&mut self, mut src: usize, mut dsto: u16) {
        self.room_draw_exploding_wall_column(src, dsto);
        src += 24;
        dsto = dsto.wrapping_add(2);
        let fill = self.tile_word(src, 0);
        self.dungeon_room_load_mut().set_draw_width_indicator(18);
        if self.lag_free {
            self.dungeon_room_load_mut().set_draw_width_indicator_word(18);
        }
        for x in 0..18u16 {
            for y in 0..6u16 {
                self.room_write_bg2(dsto + x + y * 64, fill);
            }
        }
        if self.lag_free {
            // $09:ACE7 stores the word loop count; $09:AD1D decrements
            // it to zero. The native for-loop must publish its final count.
            self.dungeon_room_load_mut().set_draw_width_indicator_word(0);
        }
        self.room_draw_exploding_wall_column(src + 2, dsto + 18);
    }

    /// ROM: `RoomDraw_ExplodingWallColumn`
    pub(super) fn room_draw_exploding_wall_column(&mut self, src: usize, dsto: u16) {
        for i in 0..6u16 {
            self.room_write_current(dsto + i * 64, self.tile_word(src, i as usize));
            self.room_write_current(dsto + 1 + i * 64, self.tile_word(src, (i + 6) as usize));
        }
    }

    /// ROM: `RoomDraw_CheckIfLowerLayerDoors_Y`
    pub(super) fn room_draw_check_if_lower_layer_doors_y(&mut self, door_type: u8, dsto: u16) {
        if door_type == DOOR_TYPE_REGULAR2 {
            self.room_draw_make_door_parts_high_priority_y(dsto + xy(0, 4) as u16);
            self.door_draw_helper4(door_type, dsto);
        } else if door_type == DOOR_TYPE_WATERFALL_TUNNEL {
            self.door_draw_helper4(door_type, dsto);
            self.door_prioritize_cur_door();
        } else {
            self.door_draw_helper4(door_type, dsto);
        }
    }

    /// ROM: `RoomDraw_NormalRangedDoors_West`
    pub(super) fn room_draw_normal_ranged_doors_west(
        &mut self,
        door_type: u8,
        dsto: u16,
        position: usize,
    ) {
        if position >= 6 {
            if let Some(&right_dsto_bytes) = DOOR_POSITION_RIGHT.get(position - 6) {
                let saved = self.game_state.dungeon.doors.current_door_index();
                self.dungeon_doors_mut()
                    .set_current_door_index(saved | 0x10);
                self.room_draw_normal_ranged_doors_east(door_type, right_dsto_bytes / 2);
                self.dungeon_doors_mut().set_current_door_index(saved);
            }
        }

        let mut t = self.room_draw_flag_doors_and_get_final_type(2, door_type, dsto);
        if t & 0x100 != 0 {
            return;
        }
        if t as u8 == DOOR_TYPE_36 || t as u8 == DOOR_TYPE_38 {
            let new_type = if t as u8 == DOOR_TYPE_36 {
                DOOR_TYPE_SHUTTERS_TWO_WAY
            } else {
                DOOR_TYPE_REGULAR
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        if let Some(&src) = DOOR_TYPE_SRC_LEFT.get(t as usize >> 1) {
            self.room_data_draw_object_door_left_3x4(src as usize, dsto);
        }
    }

    /// ROM: `RoomDraw_NormalRangedDoors_East`
    pub(super) fn room_draw_normal_ranged_doors_east(&mut self, door_type: u8, dsto: u16) {
        if door_type == DOOR_TYPE_REGULAR2 {
            self.room_draw_make_door_parts_high_priority_x(dsto + xy(4, 0) as u16);
        }
        if door_type == DOOR_TYPE_WATERFALL_TUNNEL {
            self.room_draw_one_sided_shutters_east(door_type, dsto);
            self.door_prioritize_cur_door();
        } else {
            self.room_draw_one_sided_shutters_east(door_type, dsto);
        }
    }

    /// ROM: `RoomDraw_OneSidedShutters_East`
    pub(super) fn room_draw_one_sided_shutters_east(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.room_draw_flag_doors_and_get_final_type(3, door_type, dsto);
        if t & 0x100 != 0 {
            return;
        }
        if t as u8 == DOOR_TYPE_36 || t as u8 == DOOR_TYPE_38 {
            let new_type = if t as u8 == DOOR_TYPE_36 {
                DOOR_TYPE_REGULAR
            } else {
                DOOR_TYPE_SHUTTERS_TWO_WAY
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        if let Some(&src) = DOOR_TYPE_SRC_RIGHT.get(t as usize >> 1) {
            self.room_data_draw_object_door_right_3x4(src as usize, dsto);
        }
    }

    pub(super) fn room_rewrite_last_door_type(&mut self, door_type: u8) {
        let index = (self
            .game_state
            .dungeon
            .doors
            .current_door_index()
            .wrapping_sub(2)
            >> 1) as usize;
        if index < 16 {
            self.dungeon_doors_mut()
                .set_door_type_word(index, ((index as u16) << 8) | door_type as u16);
        }
    }

    /// ROM: `RoomDraw_FlagDoorsAndGetFinalType`
    pub(super) fn room_draw_flag_doors_and_get_final_type(
        &mut self,
        direction: u16,
        door_type: u8,
        dsto: u16,
    ) -> u16 {
        let slot = self.game_state.dungeon.doors.current_door_index() as usize >> 1;
        if slot < 16 {
            self.dungeon_doors_mut()
                .set_door_direction_word(slot, direction);
            self.dungeon_doors_mut()
                .set_door_tilemap_address(slot, dsto * 2);
            self.dungeon_doors_mut()
                .set_door_type_word(slot, ((slot as u16) << 8) | door_type as u16);
        }
        let mut remapped = door_type;
        if (slot & 7) < 4
            && self
                .game_state
                .dungeon
                .doors
                .opened_doors_including_adjacent()
                & upper_bitmask(slot & 7)
                != 0
        {
            let is_shutter =
                door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER;
            if !(is_shutter && self.game_state.dungeon.environment.trapdoors_down() != 0) {
                remapped = DOOR_TYPE_REMAP
                    .get(door_type as usize >> 1)
                    .copied()
                    .unwrap_or(door_type);
                if !is_shutter
                    && door_type >= DOOR_TYPE_INVISIBLE_DOOR
                    && door_type != DOOR_TYPE_REGULAR_DOOR33
                    && door_type != DOOR_TYPE_WARP_ROOM_DOOR
                {
                    self.dungeon_doors_mut().mark_door_opened(slot);
                }
            }
        }
        self.dungeon_doors_mut()
            .set_current_door_index((slot as u16).wrapping_mul(2).wrapping_add(2));

        if remapped == DOOR_TYPE_SLASHABLE || remapped == DOOR_TYPE_WATERFALL_TUNNEL {
            return 0x100 | remapped as u16;
        }
        if door_type != DOOR_TYPE_INVISIBLE_DOOR {
            return remapped as u16;
        }

        self.dungeon_room_doors_mut()
            .set_invisible_door_marker(slot, direction);
        let opened = self
            .game_state
            .dungeon
            .doors
            .opened_doors_including_adjacent()
            | upper_bitmask(slot);
        self.dungeon_doors_mut()
            .set_opened_doors_including_adjacent(opened);
        DOOR_TYPE_REGULAR as u16
    }

    /// ROM: `RoomDraw_MarkDungeonToggleDoor`
    pub(super) fn room_draw_mark_dungeon_toggle_door(&mut self, dsto: u16) {
        self.dungeon_room_parser_mut()
            .append_toggle_palace_pos(dsto);
    }

    /// ROM: `RoomDraw_MarkLayerToggleDoor`
    pub(super) fn room_draw_mark_layer_toggle_door(&mut self, dsto: u16) {
        self.dungeon_room_parser_mut().append_toggle_floor_pos(dsto);
    }

    /// ROM: `RoomData_DrawObject_Door_up_4x3`
    pub(super) fn room_data_draw_object_door_up_4x3(&mut self, src: usize, dsto: u16) {
        for x in 0..4 {
            for y in 0..3 {
                let tile = self.tile_word(src, (x * 3 + y) as usize);
                self.room_write_current(dsto + x + y * 64, tile);
            }
        }
    }

    /// ROM: `RoomData_DrawObject_Door_left_3x4`
    pub(super) fn room_data_draw_object_door_left_3x4(&mut self, src: usize, dsto: u16) {
        for x in 0..3 {
            for y in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
    }

    /// ROM: `RoomData_DrawObject_Door_right_3x4`
    pub(super) fn room_data_draw_object_door_right_3x4(&mut self, src: usize, dsto: u16) {
        for x in 0..3 {
            for y in 0..4 {
                self.room_write_current(
                    dsto + 1 + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
    }

    /// ROM: `Door_PrioritizeCurDoor`
    pub(super) fn door_prioritize_cur_door(&mut self) {
        let index = (self
            .game_state
            .dungeon
            .doors
            .current_door_index()
            .wrapping_sub(2)
            >> 1) as usize;
        if index < 16 {
            let addr = self.game_state.dungeon.doors.door_tilemap_address(index) | 0x2000;
            self.dungeon_doors_mut()
                .set_door_tilemap_address(index, addr);
        }
    }

    /// ROM: `RoomDraw_NorthCurtainDoor`
    pub(super) fn room_draw_north_curtain_door(&mut self, dsto: u16) {
        let rv = self.room_draw_flag_doors_and_get_final_type(0, DOOR_TYPE_SLASHABLE, dsto);
        if rv & 0x100 != 0 {
            // $01AC4D-$01AC52: LDX #$078A, JMP RoomDraw_4x4 (no scope).
            self.room_draw_4x4_by_jump(0x078a, dsto);
        } else {
            let src = DOOR_TYPE_SRC_UP
                .get(rv as usize >> 1)
                .copied()
                .unwrap_or(0x078a) as usize;
            // $01AC57: JSR RoomDraw_4x4.
            self.room_draw_4x4(src, dsto);
        }
    }

    /// ROM: `RoomDraw_HighRangeDoor_North`
    pub(super) fn room_draw_high_range_door_north(
        &mut self,
        door_type: u8,
        dsto: u16,
        position: usize,
    ) {
        if position >= 6 && door_type != DOOR_TYPE_WARP_ROOM_DOOR {
            if let Some(&down_dsto_bytes) = DOOR_POSITION_DOWN.get(position - 6) {
                let saved = self.game_state.dungeon.doors.current_door_index();
                self.dungeon_doors_mut()
                    .set_current_door_index(saved | 0x10);
                self.room_draw_one_sided_lower_shutters_south(door_type, down_dsto_bytes / 2);
                self.dungeon_doors_mut().set_current_door_index(saved);
            }
        }
        let mut t = self.room_draw_flag_doors_and_get_final_type(0, door_type, dsto);
        if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR || t as u8 == DOOR_TYPE_SHUTTER_TRAP_DL {
            let new_type = if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR {
                DOOR_TYPE_REGULAR_DOOR33
            } else {
                DOOR_TYPE_SHUTTER
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        let src = DOOR_TYPE_SRC_UP.get(t as usize >> 1).copied().unwrap_or(0) as usize;
        for x in 0..4 {
            let d = dsto + x;
            self.room_write_bg2(d, self.tile_word(src, x as usize * 3));
            self.room_write_bg1(d + 64, self.tile_word(src, x as usize * 3 + 1));
            self.room_write_bg1(d + 128, self.tile_word(src, x as usize * 3 + 2));
        }
        if door_type != DOOR_TYPE_WARP_ROOM_DOOR {
            self.room_draw_make_door_high_priority_north(dsto);
        }
        self.door_prioritize_cur_door();
    }

    /// ROM: `RoomDraw_OneSidedLowerShutters_South`
    pub(super) fn room_draw_one_sided_lower_shutters_south(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.room_draw_flag_doors_and_get_final_type(1, door_type, dsto);
        if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR || t as u8 == DOOR_TYPE_SHUTTER_TRAP_DL {
            let new_type = if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR {
                DOOR_TYPE_SHUTTER
            } else {
                DOOR_TYPE_REGULAR_DOOR33
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        let src = DOOR_TYPE_SRC_DOWN
            .get(t as usize >> 1)
            .copied()
            .unwrap_or(0) as usize;
        for x in 0..4 {
            let d = dsto + x;
            self.room_write_bg1(d + 64, self.tile_word(src, x as usize * 3));
            self.room_write_bg1(d + 128, self.tile_word(src, x as usize * 3 + 1));
            self.room_write_bg2(d + 192, self.tile_word(src, x as usize * 3 + 2));
        }
        self.room_draw_make_door_high_priority_south(dsto + xy(0, 4) as u16);
        self.door_prioritize_cur_door();
    }

    /// ROM: `RoomDraw_HighRangeDoor_West`
    pub(super) fn room_draw_high_range_door_west(
        &mut self,
        door_type: u8,
        dsto: u16,
        position: usize,
    ) {
        if position >= 6 {
            if let Some(&right_dsto_bytes) = DOOR_POSITION_RIGHT.get(position - 6) {
                let saved = self.game_state.dungeon.doors.current_door_index();
                self.dungeon_doors_mut()
                    .set_current_door_index(saved | 0x10);
                self.room_draw_one_sided_lower_shutters_east(door_type, right_dsto_bytes / 2);
                self.dungeon_doors_mut().set_current_door_index(saved);
            }
        }
        let mut t = self.room_draw_flag_doors_and_get_final_type(2, door_type, dsto);
        if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR || t as u8 == DOOR_TYPE_SHUTTER_TRAP_DL {
            let new_type = if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR {
                DOOR_TYPE_SHUTTER
            } else {
                DOOR_TYPE_REGULAR_DOOR33
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        let src = DOOR_TYPE_SRC_LEFT
            .get(t as usize >> 1)
            .copied()
            .unwrap_or(0) as usize;
        for y in 0..4 {
            self.room_write_bg2(dsto + y * 64, self.tile_word(src, y as usize));
        }
        for x in 1..3 {
            for y in 0..4 {
                self.room_write_bg(
                    0x4000,
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
        self.room_draw_make_door_high_priority_west(dsto);
        self.door_prioritize_cur_door();
    }

    /// ROM: `RoomDraw_OneSidedLowerShutters_East`
    pub(super) fn room_draw_one_sided_lower_shutters_east(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.room_draw_flag_doors_and_get_final_type(3, door_type, dsto);
        if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR || t as u8 == DOOR_TYPE_SHUTTER_TRAP_DL {
            let new_type = if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR {
                DOOR_TYPE_REGULAR_DOOR33
            } else {
                DOOR_TYPE_SHUTTER
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        let src = DOOR_TYPE_SRC_RIGHT
            .get(t as usize >> 1)
            .copied()
            .unwrap_or(0) as usize;
        for x in 0..2 {
            for y in 0..4 {
                self.room_write_bg(
                    0x4000,
                    dsto + 1 + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
        for y in 0..4 {
            self.room_write_bg(
                0x2000,
                dsto + 3 + y * 64,
                self.tile_word(src, 8 + y as usize),
            );
        }
        self.room_draw_make_door_high_priority_east(dsto + xy(4, 0) as u16);
        self.door_prioritize_cur_door();
    }

    /// ROM: `RoomDraw_MakeDoorHighPriority_North`
    pub(super) fn room_draw_make_door_high_priority_north(&mut self, dsto_org: u16) {
        let mut dsto = dsto_org & (0xf07f >> 1);
        loop {
            for x in 0..4 {
                let tile = self.room_read_bg2(dsto + x) | 0x2000;
                self.room_write_bg2(dsto + x, tile);
            }
            dsto += 64;
            if dsto == dsto_org {
                break;
            }
        }
    }

    /// ROM: `RoomDraw_MakeDoorHighPriority_South`
    pub(super) fn room_draw_make_door_high_priority_south(&mut self, mut dsto: u16) {
        loop {
            for x in 0..4 {
                let tile = self.room_read_bg2(dsto + x) | 0x2000;
                self.room_write_bg2(dsto + x, tile);
            }
            dsto += 64;
            if dsto & 0x07c0 == 0 {
                break;
            }
        }
    }

    /// ROM: `RoomDraw_MakeDoorHighPriority_West`
    pub(super) fn room_draw_make_door_high_priority_west(&mut self, dsto_org: u16) {
        let mut dsto = dsto_org & 0xffe0;
        loop {
            for y in 0..4 {
                let pos = dsto + y * 64;
                let tile = self.room_read_bg2(pos) | 0x2000;
                self.room_write_bg2(pos, tile);
            }
            dsto += 1;
            if dsto == dsto_org {
                break;
            }
        }
    }

    /// ROM: `RoomDraw_MakeDoorHighPriority_East`
    pub(super) fn room_draw_make_door_high_priority_east(&mut self, mut dsto: u16) {
        loop {
            for y in 0..4 {
                let pos = dsto + y * 64;
                let tile = self.room_read_bg2(pos) | 0x2000;
                self.room_write_bg2(pos, tile);
            }
            dsto += 1;
            if dsto & 0x1f == 0 {
                break;
            }
        }
    }

    /// ROM: `RoomDraw_MakeDoorPartsHighPriority_Y`
    pub(super) fn room_draw_make_door_parts_high_priority_y(&mut self, dsto: u16) {
        for y in 0..7 {
            for x in 0..4 {
                let pos = dsto + x + y * 64;
                let tile = self.room_read_bg2(pos) | 0x2000;
                self.room_write_bg2(pos, tile);
            }
        }
    }

    /// ROM: `RoomDraw_MakeDoorPartsHighPriority_X`
    pub(super) fn room_draw_make_door_parts_high_priority_x(&mut self, dsto: u16) {
        for x in 0..5 {
            for y in 0..4 {
                let pos = dsto + x + y * 64;
                let tile = self.room_read_bg2(pos) | 0x2000;
                self.room_write_bg2(pos, tile);
            }
        }
    }

    pub(super) fn room_prioritize_throne_room_door_edge(&mut self) {
        for dsto in [0x0ede, 0x0f1e, 0x0f5e] {
            let tile = self.room_read_bg2(dsto);
            self.room_write_bg2(dsto, tile | 0x2000);
        }
    }

    pub(super) fn room_plane_offset(&self) -> u16 {
        if self
            .game_state
            .dungeon
            .room_tilemaps
            .first_line_pointer_row0()
            == 0x4000
        {
            0x1000
        } else {
            0
        }
    }

    pub(super) fn room_plane_tilemap_bit(&self) -> u16 {
        if self
            .game_state
            .dungeon
            .room_tilemaps
            .first_line_pointer_row0()
            == 0x4000
        {
            0x2000
        } else {
            0
        }
    }

    /// ROM: `Door_Up_EntranceDoor`
    pub(super) fn door_up_entrance_door(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Up_EntranceDoor assert");
    }

    /// ROM: `Door_Down_EntranceDoor`
    pub(super) fn door_down_entrance_door(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Down_EntranceDoor assert");
    }

    /// ROM: `Door_Left_EntranceDoor`
    pub(super) fn door_left_entrance_door(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Left_EntranceDoor assert");
    }

    /// ROM: `Door_Right_EntranceDoor`
    pub(super) fn door_right_entrance_door(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Right_EntranceDoor assert");
    }

    /// ROM: `Door_Draw_Helper4`
    pub(super) fn door_draw_helper4(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.room_draw_flag_doors_and_get_final_type(1, door_type, dsto);
        if t & 0x100 != 0 {
            return;
        }

        if t == DOOR_TYPE_1E as u16 || t == DOOR_TYPE_36 as u16 || t == DOOR_TYPE_38 as u16 {
            let new_type = if t == DOOR_TYPE_38 as u16 {
                DOOR_TYPE_SHUTTERS_TWO_WAY
            } else {
                DOOR_TYPE_REGULAR
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }

        if let Some(&src) = DOOR_TYPE_SRC_DOWN.get(t as usize >> 1) {
            for i in 0..4 {
                let d = dsto + i as u16;
                self.room_write_current(d + 64, self.tile_word(src as usize, i * 3));
                self.room_write_current(d + 128, self.tile_word(src as usize, i * 3 + 1));
                self.room_write_current(d + 192, self.tile_word(src as usize, i * 3 + 2));
            }
        }
    }

    pub(super) fn GetRoomDoorInfo(&self, room: usize) -> Option<&[u8]> {
        let offset = self.asset_u16(5, room) as usize;
        self.asset_raw(3)?.get(offset..)
    }

    pub(super) fn GetRoomHeaderPtr(&self, room: usize) -> Option<&[u8]> {
        self.dungeon_room_header(room)
    }

    /// ROM: `Object_Draw_DoorLeft_3x4`
    pub(super) fn object_draw_door_left_3x4(&mut self, src: u16, door: usize) {
        let dsto = self.game_state.dungeon.doors.door_tilemap_address(door) >> 1;
        for i in 0..3 {
            for y in 0..4 {
                self.room_write_bg(
                    0x2000,
                    dsto + i as u16 + y * 64,
                    self.tile_word(src as usize, i * 4 + y as usize),
                );
            }
        }
    }

    /// ROM: `Object_Draw_DoorRight_3x4`
    pub(super) fn object_draw_door_right_3x4(&mut self, src: u16, door: usize) {
        let dsto = self.game_state.dungeon.doors.door_tilemap_address(door) >> 1;
        for i in 0..3 {
            for y in 0..4 {
                self.room_write_bg(
                    0x2000,
                    dsto + 1 + i as u16 + y * 64,
                    self.tile_word(src as usize, i * 4 + y as usize),
                );
            }
        }
    }

    /// ROM: `GetDoorDrawDataIndex_North_clean_door_index`
    pub(super) fn get_door_draw_data_index_north_clean_door_index(&mut self, door: usize) {
        self.get_door_draw_data_index_north(door, door);
    }

    /// ROM: `DoorDoorStep1_North`
    pub(super) fn door_door_step1_north(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = self.game_state.dungeon.doors.door_tilemap_address(door) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x1fff) >= DOOR_POSITION_UP[6] as i32 {
            pos -= 0x500;
            if (self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe) >= 0x42 {
                pos -= 0x300;
            }
            self.get_door_draw_data_index_south(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.dungeon_load_single_door_attribute(door ^ 8);
        }
        self.get_door_draw_data_index_north(door, door & 7);
        dma_ptr
    }

    /// ROM: `GetDoorDrawDataIndex_North`
    pub(super) fn get_door_draw_data_index_north(&mut self, door: usize, r4_door: usize) {
        let door_type = self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe;
        let mut x = self.game_state.dungeon.doors.door_open_counter_low() as usize;
        if x == 0 || x == 4 {
            self.draw_door_to_tile_map_north(door, r4_door);
            return;
        }
        if door_type == DOOR_TYPE_STAIR_MASK_LOCKED2
            || door_type == DOOR_TYPE_STAIR_MASK_LOCKED3
            || door_type >= 0x42
        {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = DOOR_ANIMATION_UP_SOURCES[x >> 1];
        self.object_draw_door_up_4x3(src, door);
    }

    /// ROM: `DrawDoorToTileMap_North`
    pub(super) fn draw_door_to_tile_map_north(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_UP[index];
        self.object_draw_door_up_4x3(src, door);
    }

    /// ROM: `Object_Draw_DoorUp_4x3`
    pub(super) fn object_draw_door_up_4x3(&mut self, src: u16, door: usize) {
        let dsto = self.game_state.dungeon.doors.door_tilemap_address(door) >> 1;
        for i in 0..4 {
            for y in 0..3 {
                self.room_write_bg(
                    0x2000,
                    dsto + i as u16 + y * 64,
                    self.tile_word(src as usize, i * 3 + y as usize),
                );
            }
        }
    }

    /// ROM: `GetDoorDrawDataIndex_South_clean_door_index`
    pub(super) fn get_door_draw_data_index_south_clean_door_index(&mut self, door: usize) {
        self.get_door_draw_data_index_south(door, door);
    }

    /// ROM: `DoorDoorStep1_South`
    pub(super) fn door_door_step1_south(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = self.game_state.dungeon.doors.door_tilemap_address(door) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x1fff) < DOOR_POSITION_DOWN[9] as i32 {
            pos += 0x500;
            if (self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe) >= 0x42 {
                pos += 0x300;
            }
            self.get_door_draw_data_index_north(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.dungeon_load_single_door_attribute(door ^ 8);
        }
        self.get_door_draw_data_index_south(door, door & 7);
        dma_ptr
    }

    /// ROM: `GetDoorDrawDataIndex_South`
    pub(super) fn get_door_draw_data_index_south(&mut self, door: usize, r4_door: usize) {
        let door_type = self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe;
        let mut x = self.game_state.dungeon.doors.door_open_counter_low() as usize;
        if x == 0 || x == 4 {
            self.draw_door_to_tile_map_south(door, r4_door);
            return;
        }
        if door_type >= 0x42 {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = DOOR_ANIMATION_DOWN_SOURCES[x >> 1];
        self.object_draw_door_down_4x3(src, door);
    }

    /// ROM: `DrawDoorToTileMap_South`
    pub(super) fn draw_door_to_tile_map_south(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_DOWN[index];
        self.object_draw_door_down_4x3(src, door);
    }

    /// ROM: `Object_Draw_DoorDown_4x3`
    pub(super) fn object_draw_door_down_4x3(&mut self, src: u16, door: usize) {
        let dsto = self.game_state.dungeon.doors.door_tilemap_address(door) >> 1;
        for i in 0..4 {
            for y in 0..3 {
                self.room_write_bg(
                    0x2000,
                    dsto + i as u16 + (y + 1) * 64,
                    self.tile_word(src as usize, i * 3 + y as usize),
                );
            }
        }
    }

    /// ROM: `GetDoorDrawDataIndex_West_clean_door_index`
    pub(super) fn get_door_draw_data_index_west_clean_door_index(&mut self, door: usize) {
        self.get_door_draw_data_index_west(door, door);
    }

    /// ROM: `DoorDoorStep1_West`
    pub(super) fn door_door_step1_west(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = self.game_state.dungeon.doors.door_tilemap_address(door) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x7ff) >= DOOR_POSITION_LEFT[6] as i32 {
            pos -= 16;
            if (self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe) >= 0x42 {
                pos -= 12;
            }
            self.get_door_draw_data_index_east(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.dungeon_load_single_door_attribute(door ^ 8);
        }
        self.get_door_draw_data_index_west(door, door & 7);
        dma_ptr
    }

    /// ROM: `GetDoorDrawDataIndex_West`
    pub(super) fn get_door_draw_data_index_west(&mut self, door: usize, r4_door: usize) {
        let door_type = self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe;
        let mut x = self.game_state.dungeon.doors.door_open_counter_low() as usize;
        if x == 0 || x == 4 {
            self.draw_door_to_tile_map_west(door, r4_door);
            return;
        }
        if door_type >= 0x42 {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = DOOR_ANIMATION_LEFT_SOURCES[x >> 1];
        self.object_draw_door_left_3x4(src, door);
    }

    /// ROM: `DrawDoorToTileMap_West`
    pub(super) fn draw_door_to_tile_map_west(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_LEFT[index];
        self.object_draw_door_left_3x4(src, door);
    }

    /// ROM: `GetDoorDrawDataIndex_East_clean_door_index`
    pub(super) fn get_door_draw_data_index_east_clean_door_index(&mut self, door: usize) {
        self.get_door_draw_data_index_east(door, door);
    }

    /// ROM: `DoorDoorStep1_East`
    pub(super) fn door_door_step1_east(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = self.game_state.dungeon.doors.door_tilemap_address(door) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x7ff) < DOOR_POSITION_RIGHT[6] as i32 {
            pos += 16;
            if (self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe) >= 0x42 {
                pos += 12;
            }
            self.get_door_draw_data_index_west(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.dungeon_load_single_door_attribute(door ^ 8);
        }
        self.get_door_draw_data_index_east(door, door & 7);
        dma_ptr
    }

    /// ROM: `GetDoorDrawDataIndex_East`
    pub(super) fn get_door_draw_data_index_east(&mut self, door: usize, r4_door: usize) {
        let door_type = self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe;
        let mut x = self.game_state.dungeon.doors.door_open_counter_low() as usize;
        if x == 0 || x == 4 {
            self.draw_door_to_tile_map_east(door, r4_door);
            return;
        }
        if door_type >= 0x42 {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = DOOR_ANIMATION_RIGHT_SOURCES[x >> 1];
        self.object_draw_door_right_3x4(src, door);
    }

    /// ROM: `DrawDoorToTileMap_East`
    pub(super) fn draw_door_to_tile_map_east(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_RIGHT[index];
        self.object_draw_door_right_3x4(src, door);
    }

    pub(super) fn GetDoorGraphicsIndex(&self, door: usize, r4_door: usize) -> u8 {
        let mut door_type = self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe;
        if self
            .game_state
            .dungeon
            .doors
            .opened_doors_including_adjacent()
            & upper_bitmask(r4_door)
            != 0
        {
            door_type = DOOR_TYPE_REMAP[door_type as usize >> 1];
        }
        door_type
    }

    /// ROM: `ClearExplodingWallFromTileMap_ClearOnePair`
    pub(super) fn clear_exploding_wall_from_tile_map_clear_one_pair(
        &mut self,
        mut dsto: u16,
        mut src: usize,
    ) {
        for _ in 0..2 {
            for j in 0..12u16 {
                self.room_write_bg2(dsto + j * 64, self.tile_word(src, j as usize));
            }
            dsto = dsto.wrapping_add(1);
            src += 24;
        }
    }

    /// ROM: `Door_BlastWallExploding_Draw`
    pub(super) fn door_blast_wall_exploding_draw(&mut self, dsto: usize) {
        let src = 0x31eausize;
        self.clear_exploding_wall_from_tile_map_clear_one_pair(dsto as u16, src);
        let mut dst = dsto as u16 + 2;
        let fill = self.tile_word(src, 24);
        let count = self
            .game_state
            .dungeon
            .room_effects
            .crush_wall_progress()
            .wrapping_sub(1);
        for _ in 0..count {
            for j in 0..12u16 {
                self.room_write_bg2(dst + j * 64, fill);
            }
            dst = dst.wrapping_add(1);
        }
        self.clear_exploding_wall_from_tile_map_clear_one_pair(dst, src + 50);
    }

    pub(super) fn ClearAndStripeExplodingWall(&mut self, mut dsto: u16) {
        let mut r6 = 0x80u16;
        let mut r14 = 0u16;
        let mut r10 = self
            .game_state
            .dungeon
            .room_effects
            .crush_wall_progress()
            .wrapping_add(3);
        let mut r2 = 0u16;
        if r10 >= 8 {
            r2 = r10.wrapping_sub(6);
            r14 = 1;
            r10 = 3;
        }
        let door = self.game_state.dungeon.doors.current_door_index() as usize >> 1;
        if self.game_state.dungeon.doors.door_direction(door) & 2 == 0 {
            r6 = r6.wrapping_add(1);
        }

        let mut upload = UVRAM_DATA_DUNGEON;
        loop {
            let mut cols = r10;
            loop {
                let vram_addr = self.dungeon_map_vram_addr_no_swap(dsto);
                self.write_vram_upload_absolute_word(upload, vram_addr);
                self.write_vram_upload_absolute_word(upload + 2, r6 | 0x0a00);
                for y in 0..5u16 {
                    let tile = self.room_read_bg2(dsto + y * 64);
                    self.write_vram_upload_absolute_word(upload + 4 + y as usize * 2, tile);
                }
                self.write_vram_upload_absolute_word(upload + 14, vram_addr.wrapping_add(0x04a0));
                self.write_vram_upload_absolute_word(upload + 16, r6 | 0x0e00);
                for y in 0..7u16 {
                    let tile = self.room_read_bg2(dsto + (y + 5) * 64);
                    self.write_vram_upload_absolute_word(upload + 18 + y as usize * 2, tile);
                }
                dsto = dsto.wrapping_add(1);
                upload += 32;
                cols = cols.wrapping_sub(1);
                if cols == 0 {
                    break;
                }
            }
            if r14 == 0 {
                break;
            }
            r14 = r14.wrapping_sub(1);
            let tab_index = ((r2 >> 1) + if r6 & 1 != 0 { 0 } else { 8 } - 1) as usize;
            dsto = dsto.wrapping_add(
                CLEAR_AND_STRIPE_EXPLODING_WALL_BLAST_WALL_STRIPE_ROW_ADVANCES[tab_index] >> 1,
            );
            r10 = 3;
        }
        self.write_vram_upload_absolute_word(upload, 0xffff);
    }

    /// ROM: `DrawDoorOpening_Step1`
    pub(super) fn draw_door_opening_step1(&mut self, door: usize, dma_ptr: usize) -> usize {
        self.dungeon_doors_mut()
            .set_current_door_index_for_slot(door);
        self.dungeon_room_load_mut().set_selected_key_door(door);
        match self.game_state.dungeon.doors.door_direction(door) & 3 {
            0 => self.door_door_step1_north(door, dma_ptr),
            1 => self.door_door_step1_south(door, dma_ptr),
            2 => self.door_door_step1_west(door, dma_ptr),
            3 => self.door_door_step1_east(door, dma_ptr),
            _ => 0,
        }
    }

    pub(super) fn DrawShutterDoorSteps(&mut self, door: usize) {
        self.dungeon_doors_mut()
            .set_current_door_index_for_slot(door);
        self.dungeon_room_load_mut().set_selected_key_door(door);
        match self.game_state.dungeon.doors.door_direction(door) & 3 {
            0 => self.get_door_draw_data_index_north_clean_door_index(door),
            1 => self.get_door_draw_data_index_south_clean_door_index(door),
            2 => self.get_door_draw_data_index_west_clean_door_index(door),
            3 => self.get_door_draw_data_index_east_clean_door_index(door),
            _ => {}
        }
    }

    pub(super) fn DrawEyeWatchDoor(&mut self, door: usize) {
        self.dungeon_doors_mut()
            .set_current_door_index_for_slot(door);
        self.dungeon_room_load_mut().set_selected_key_door(door);
        match self.game_state.dungeon.doors.door_direction(door) & 3 {
            0 => self.draw_door_to_tile_map_north(door, door),
            1 => self.draw_door_to_tile_map_south(door, door),
            2 => self.draw_door_to_tile_map_west(door, door),
            3 => self.draw_door_to_tile_map_east(door, door),
            _ => {}
        }
    }

    pub(super) fn OperateShutterDoors(&mut self) {
        let mut anim_dst = 0usize;
        let mut y = 2u8;

        let step = self
            .game_state
            .dungeon
            .doors
            .door_animation_step()
            .wrapping_add(1);
        self.dungeon_doors_mut().set_door_animation_step(step);
        if step != 4 {
            y = if self.game_state.dungeon.environment.trapdoors_down_low() != 0 {
                0
            } else {
                4
            };
            if step != 8 {
                if self.game_state.dungeon.doors.door_animation_step_low() != 0x10 {
                    return;
                }
                self.set_submodule(0);
                self.clear_nmi_copy_packets_request();
                return;
            }
        }
        self.dungeon_doors_mut().set_door_open_counter(y as u16);

        let mut cur = 0usize;
        while cur != 0x18 {
            self.dungeon_doors_mut().set_current_door_pos(cur as u16);
            let j = cur >> 1;
            let door_type = self.game_state.dungeon.doors.door_type_and_slot(j) & 0xfe;
            if door_type == DOOR_TYPE_SHUTTER || door_type == DOOR_TYPE_SHUTTERS_TWO_WAY {
                let mask = upper_bitmask(j);
                let mut should_draw = true;
                let mut opened = self
                    .game_state
                    .dungeon
                    .doors
                    .opened_doors_including_adjacent();
                if self.game_state.dungeon.environment.trapdoors_down_low() == 0 {
                    if opened & mask != 0 {
                        should_draw = false;
                    } else if step == 8 {
                        self.set_sound_effect_2(21);
                        opened ^= mask;
                        self.dungeon_doors_mut()
                            .set_opened_doors_including_adjacent(opened);
                    }
                } else if opened & mask == 0 {
                    should_draw = false;
                } else if step == 8 {
                    self.set_sound_effect_2(22);
                    opened ^= mask;
                    self.dungeon_doors_mut()
                        .set_opened_doors_including_adjacent(opened);
                }

                if should_draw {
                    self.DrawShutterDoorSteps(j);
                    let addr = self.game_state.dungeon.doors.door_tilemap_address(j);
                    anim_dst = self.dungeon_prep_overlay_dma_next_prep(anim_dst, addr);
                    if step == 8 {
                        self.dungeon_load_toggle_door_attr_other_entry(j as i32);
                    }
                }
            }
            cur += 2;
        }
        self.dungeon_doors_mut().set_current_door_pos(0x16);

        if anim_dst != 0 {
            self.set_core_update_disable_flag(1);
            self.request_nmi_copy_packets();
            if self.game_state.dungeon.doors.door_animation_step_low() != 0x10 {
                return;
            }
        }
        self.set_submodule(0);
        self.clear_nmi_copy_packets_request();
    }

    pub(super) fn OpenCrackedDoor(&mut self) {
        self.dungeon_opening_locked_door_combined(true);
    }

    /// ROM: `Dungeon_OpeningLockedDoor_Combined`
    pub(super) fn dungeon_opening_locked_door_combined(&mut self, skip_anim: bool) {
        let mut ctr = 2u8;
        let step;
        if skip_anim {
            self.dungeon_doors_mut().set_door_animation_step(16);
            step = 16;
        } else {
            step = self
                .game_state
                .dungeon
                .doors
                .door_animation_step()
                .wrapping_add(1);
            self.dungeon_doors_mut().set_door_animation_step(step);
            if step != 4 && step != 12 {
                if step == 16 {
                    self.finish_locked_door_opening();
                }
                return;
            }
        }

        if step == 12 || skip_anim {
            let cur = self.game_state.dungeon.doors.current_door_pos() as usize;
            let mask = upper_bitmask(
                self.game_state
                    .dungeon
                    .bg2_attributes
                    .bg2_tile(cur)
                    .object_slot()
                    & 7,
            );
            let opened_adj = self
                .game_state
                .dungeon
                .doors
                .opened_doors_including_adjacent()
                | mask;
            self.dungeon_doors_mut()
                .set_opened_doors_including_adjacent(opened_adj);
            self.dungeon_doors_mut().or_opened_doors(mask);
            ctr = 4;
        }

        self.dungeon_doors_mut().set_door_open_counter_low(ctr);
        let cur = self.game_state.dungeon.doors.current_door_pos() as usize;
        let k = self
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_tile(cur)
            .object_slot();
        let dma_ptr = self.draw_door_opening_step1(k, 0);
        let addr = self.game_state.dungeon.doors.door_tilemap_address(k);
        self.dungeon_prep_overlay_dma_next_prep(dma_ptr, addr);
        self.set_sound_effect_2(21);
        self.request_nmi_copy_packets();

        if step == 16 {
            self.finish_locked_door_opening();
        }
    }

    fn finish_locked_door_opening(&mut self) {
        let cur = self.game_state.dungeon.doors.current_door_pos() as usize;
        let door = self.game_state.dungeon.bg2_attributes.bg2_tile(cur);
        let k = door.object_slot();
        self.dungeon_load_toggle_door_attr_other_entry(k as i32);
        if matches!(door.dungeon_role(), DungeonRole::ClosedDoor { .. }) {
            let door_type = self.game_state.dungeon.doors.door_type_and_slot(k);
            if (DOOR_TYPE_STAIR_MASK_LOCKED0..=DOOR_TYPE_STAIR_MASK_LOCKED3).contains(&door_type) {
                self.DrawCompletelyOpenDoor();
            }
        }
        self.set_submodule(0);
    }

    pub(super) fn DrawCompletelyOpenDoor(&mut self) {
        let mut i = 0usize;
        let mut attr = TilePair::stair_landing(0);
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomUpNorth) as usize
        {
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallUpNorthSpiral) as usize
        {
            let pos = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(
                pos + xy(1, 0),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD),
            );
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(pos + xy(1, 3), TilePair::repeated(NativeTile::GROUND));
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallUpNorthSpiralBg1) as usize
        {
            let pos = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(
                pos + xy(1, 0),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD_2),
            );
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(pos + xy(1, 3), TilePair::repeated(NativeTile::GROUND));
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomUpNorthStraight)
                as usize
        {
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomUpSouthStraight)
                as usize
        {
            i += 2;
            attr = attr.next_identity();
        }

        attr = attr.descending_stair_sequence();

        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomSouthDown) as usize
        {
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallDownNorthSpiral) as usize
        {
            let pos = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(
                pos + xy(1, 0),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD),
            );
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(pos + xy(1, 3), TilePair::repeated(NativeTile::GROUND));
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallDownNorthSpiralBg1) as usize
        {
            let pos = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(
                pos + xy(1, 0),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD_2),
            );
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(pos + xy(1, 3), TilePair::repeated(NativeTile::GROUND));
            i += 2;
            attr = attr.next_identity();
        }
    }

    /// `$01:CE70..CE98`: the timer scan is conditional CPU work, before the
    /// door checks. Its frame gate, each timer branch, and the expiry call
    /// belong to the native statements that execute them.
    pub(super) fn dungeon_process_torch_timers(&mut self) {
        // LDA $1A : AND #3 : BNE.
        crate::cycle_ledger::charge(56);
        if self.game_state.frame.frame_counter & 3 != 0 {
            crate::cycle_ledger::charge(6);
            return;
        }
        // LDA $0112 : BNE.
        crate::cycle_ledger::charge(48);
        if self
            .game_state
            .world
            .transient
            .flag_custom_spell_anim_active()
            != 0
        {
            crate::cycle_ledger::charge(6);
            return;
        }
        crate::cycle_ledger::charge(16); // LDX #0.
        self.dungeon_process_torch_timers_from(0);
    }

    fn dungeon_process_torch_timers_from(&mut self, start: usize) {
        for i in start..16 {
            let timer = self.game_state.dungeon.torch.timer(i);
            crate::cycle_ledger::charge(48); // LDA $04F0,X : BEQ.
            if timer == 0 {
                crate::cycle_ledger::charge(6);
            } else {
                let next = timer.wrapping_sub(1);
                crate::cycle_ledger::charge(68); // DEC $04F0,X : BNE.
                self.dungeon_torch_mut().set_timer(i, next);
                if next != 0 {
                    crate::cycle_ledger::charge(6);
                } else {
                    // PHX : TXA : ORA #$C0 : STA $0333 : JSL.
                    crate::cycle_ledger::charge(146);
                    self.dungeon_torch_mut().set_target(NativeTile::torch(i));
                    self.dungeon_extinguish_torch();
                    crate::cycle_ledger::charge(28); // PLX after the callee returns.
                }
            }
            // INX : CPX #$10 : BNE (the table never crosses a page).
            crate::cycle_ledger::charge(46 + if i != 15 { 6 } else { 0 });
        }
    }

    pub(super) fn finish_native_dungeon_interactions(
        &mut self,
        tail: crate::cycle_models::dungeon_interaction_caller::InteractionTail,
    ) {
        use crate::cycle_models::dungeon_interaction_caller::InteractionTail;
        let _scope = crate::cycle_ledger::routine(0x01_ce70);
        match tail {
            InteractionTail::Lamp => return,
            InteractionTail::Returned => {}
            InteractionTail::TorchExpired { index } => {
                self.dungeon_extinguish_torch();
                // PLX and loop advance follow the already retired real JSL.
                crate::cycle_ledger::charge(28 + 46 + if index != 15 { 6 } else { 0 });
                self.dungeon_process_torch_timers_from(usize::from(index) + 1);
                self.dungeon_process_doors_after_timers();
            }
            InteractionTail::KeyDoor { pos } => {
                let DungeonRole::ClosedDoor { slot } = self
                    .game_state
                    .dungeon
                    .bg2_attributes
                    .bg2_tile(usize::from(pos))
                    .dungeon_role()
                else {
                    unreachable!("selected closed door")
                };
                if !self.dungeon_process_key_door_at_position(slot, usize::from(pos)) {
                    self.dungeon_process_eye_and_slash_doors();
                }
            }
            InteractionTail::EyeDoor => {
                let invisible = self.game_state.dungeon.door_setup.invisible_door_marker();
                self.dungeon_process_eye_door_after_marker(invisible);
            }
            InteractionTail::Slash => self.dungeon_process_slash_doors_after_button_mask(),
        }
        self.replay_trace_ram_watch("module07-after-torches-doors");
        self.complete_module07_room_maintenance_after_interactions();
    }

    pub(super) fn dungeon_process_torches_and_doors(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x01_ce70);
        self.dungeon_process_torch_timers();
        self.dungeon_process_doors_after_timers();
    }

    fn dungeon_process_doors_after_timers(&mut self) {
        crate::cycle_ledger::charge(48);
        if !self.game_state.player.follower_link.is_immobilized() {
            crate::cycle_ledger::charge(6 + 518);
            let dir = self.game_state.player.follower_link.facing_index();
            let link_y = self.game_state.player.follower_link.y() as i32;
            let link_x = self.game_state.player.follower_link.x() as i32;
            let mut pos = (((link_y + DUNGEON_PROCESS_TORCHES_AND_DOORS_LINK_Y_OFFSETS[dir])
                & 0x01f8)
                << 3) as usize
                | (((link_x + DUNGEON_PROCESS_TORCHES_AND_DOORS_LINK_X_OFFSETS[dir]) & 0x01f8) >> 3)
                    as usize
                | usize::from(
                    self.game_state
                        .player
                        .follower_link
                        .lower_level_tilemap_offset(),
                );

            crate::cycle_ledger::charge(
                if self.game_state.player.follower_link.lower_level_state() == 0 {
                    6
                } else {
                    24
                },
            );
            crate::cycle_ledger::charge(148);
            let closed_door = |state: &Self, pos: usize| match state
                .game_state
                .dungeon
                .bg2_attributes
                .bg2_tile(pos)
                .dungeon_role()
            {
                DungeonRole::ClosedDoor { slot } => Some(slot),
                _ => None,
            };
            let mut openable = closed_door(self, pos);
            if openable.is_none() {
                crate::cycle_ledger::charge(200);
                pos += DUNGEON_PROCESS_TORCHES_AND_DOORS_LINK_POSITION_OFFSETS[dir];
                openable = closed_door(self, pos);
                if openable.is_none() {
                    crate::cycle_ledger::charge(6);
                }
            } else {
                crate::cycle_ledger::charge(6);
            }

            if let Some(k) = openable {
                if self.dungeon_process_key_door_at_position(k, pos) {
                    return;
                }
            } else {
                crate::cycle_ledger::charge(64);
                self.set_big_key_door_message_triggered(0);
            }
        } else {
            crate::cycle_ledger::charge(24);
        }

        self.dungeon_process_eye_and_slash_doors();
    }

    fn dungeon_process_key_door_at_position(&mut self, k: usize, pos: usize) -> bool {
        let dir = self.game_state.player.follower_link.facing_index();
        crate::cycle_ledger::charge(272);
        self.dungeon_room_load_mut().set_selected_key_door(k);

        if (self.game_state.dungeon.doors.door_direction(k) & 3) == dir as u8 {
            crate::cycle_ledger::charge(110);
            let door_type = self.game_state.dungeon.doors.door_type_and_slot(k) & 0xfe;
            if door_type == DOOR_TYPE_BREAKABLE_WALL {
                crate::cycle_ledger::charge(6 + 80);
                let running = self.game_state.player.follower_link.is_running();
                if running {
                    crate::cycle_ledger::charge(80);
                }
                // The source compares the full word at $02F1: its high
                // byte is owned by the follower event flags at $02F2.
                let dash_word = u16::from(self.game_state.player.follower_link.dash_counter())
                    | (u16::from(self.game_state.sprites.follower_runtime.event_flags()) << 8);
                if running && dash_word < 63 {
                    crate::cycle_ledger::charge(148);
                    self.dungeon_doors_mut().set_current_door_pos(pos as u16);

                    let db = {
                        let _scope = crate::cycle_ledger::routine(0x09_9c38);
                        self.ancilla_add_door_debris()
                    };
                    crate::cycle_ledger::charge(16);
                    if db >= 0 {
                        crate::cycle_ledger::charge(586);
                        let db = db as usize;
                        let direction = self.game_state.dungeon.doors.door_direction(k) & 3;
                        self.door_debris_mut().set_direction(db, direction);
                        let addr = self.game_state.dungeon.doors.door_tilemap_address(k);
                        let door_x = self
                            .game_state
                            .dungeon
                            .room_load
                            .loading_bg_offset_h()
                            .wrapping_add((addr & 0x007e) << 2);
                        let door_y = self
                            .game_state
                            .dungeon
                            .room_load
                            .loading_bg_offset_v()
                            .wrapping_add((addr & 0x1f80) >> 4);
                        self.door_debris_mut().set_x_word(db, door_x);
                        self.door_debris_mut().set_y_word(db, door_y);
                    } else {
                        crate::cycle_ledger::charge(6);
                    }
                    crate::cycle_ledger::charge(172);
                    self.set_sound_effect_2(27);
                    self.set_submodule(9);
                    {
                        let _scope = crate::cycle_ledger::routine(0x07_9291);
                        self.sprite_repel_dash();
                    }
                    crate::cycle_ledger::charge(44);
                    return true;
                }
                crate::cycle_ledger::charge(6);
            } else if door_type == DOOR_TYPE_1E {
                crate::cycle_ledger::charge(40 + 40 + 230);
                self.dungeon_doors_mut().clear_door_animation_step();
                self.dungeon_doors_mut().set_current_door_pos(pos as u16);
                let palace = (self
                    .game_state
                    .inventory
                    .save_progress
                    .palace_index_x2_word()
                    >> 1) as usize;
                if self
                    .game_state
                    .inventory
                    .player_resources
                    .has_big_key_mask(upper_bitmask(palace))
                {
                    crate::cycle_ledger::charge(6);
                    self.dungeon_publish_key_door_open(k, pos);
                    return true;
                }
                crate::cycle_ledger::charge(56);
                if self
                    .game_state
                    .world
                    .transient
                    .big_key_door_message_triggered()
                    == 0
                {
                    crate::cycle_ledger::charge(210);
                    self.set_big_key_door_message_triggered(1);
                    self.dialogue_message_index_mut().set_value(0x007a);
                    self.main_show_text_message();
                    crate::cycle_ledger::charge(22);
                } else {
                    crate::cycle_ledger::charge(6);
                }
                crate::cycle_ledger::charge(24);
            } else if self.dungeon_try_small_key_door(k, pos, door_type) {
                return true;
            }
        } else {
            crate::cycle_ledger::charge(6 + 64);
            self.set_big_key_door_message_triggered(0);
        }
        false
    }

    fn dungeon_process_eye_and_slash_doors(&mut self) {
        crate::cycle_ledger::charge(70);
        let invisible = self.game_state.dungeon.door_setup.invisible_door_marker();
        if invisible & 0x80 != 0 {
            crate::cycle_ledger::charge(6);
            self.dungeon_process_slash_doors();
        } else {
            self.dungeon_process_eye_door_after_marker(invisible);
        }
    }

    fn dungeon_process_eye_door_after_marker(&mut self, invisible: u16) {
        let eye_door = {
            crate::cycle_ledger::charge(40);
            if self.game_state.player.follower_link.doorway_state() != 0 {
                crate::cycle_ledger::charge(6);
                false
            } else {
                crate::cycle_ledger::charge(56);
                if self.game_state.player.follower_link.x() >> 8 != 0x000c {
                    crate::cycle_ledger::charge(6);
                    false
                } else {
                    true
                }
            }
        };
        if eye_door {
            crate::cycle_ledger::charge(104);
            let dir = invisible as u8;
            let j = ((invisible >> 8) >> 1) as usize;
            let mut opened = self
                .game_state
                .dungeon
                .doors
                .opened_doors_including_adjacent();
            if dir == self.game_state.player.follower_link.facing() {
                crate::cycle_ledger::charge(6 + 102);
                opened &= !upper_bitmask(j);
            } else {
                crate::cycle_ledger::charge(80);
                if (dir ^ 2) == self.game_state.player.follower_link.facing() {
                    crate::cycle_ledger::charge(124);
                    opened |= upper_bitmask(j);
                } else {
                    crate::cycle_ledger::charge(6 + 102);
                    opened &= !upper_bitmask(j);
                }
            }
            crate::cycle_ledger::charge(56);
            if opened
                != self
                    .game_state
                    .dungeon
                    .doors
                    .opened_doors_including_adjacent()
            {
                crate::cycle_ledger::charge(218);
                self.dungeon_doors_mut()
                    .set_opened_doors_including_adjacent(opened);
                {
                    let _scope = crate::cycle_ledger::routine(0x01_d33a);
                    self.DrawEyeWatchDoor(j);
                }
                let addr = self.game_state.dungeon.doors.door_tilemap_address(j);
                crate::cycle_ledger::charge(46);
                {
                    let _scope = crate::cycle_ledger::routine(0x01_f764);
                    self.dungeon_prep_overlay_dma_next_prep(0, addr);
                }
                crate::cycle_ledger::charge(86);
                {
                    let _scope = crate::cycle_ledger::routine(0x01_d51c);
                    self.dungeon_load_toggle_door_attr_other_entry(j as i32);
                }
                crate::cycle_ledger::charge(256);
                self.request_nmi_copy_packets();
                self.set_sound_effect_2(21);
                return;
            } else {
                crate::cycle_ledger::charge(6);
            }
        }

        self.dungeon_process_slash_doors();
    }

    fn dungeon_process_slash_doors(&mut self) {
        crate::cycle_ledger::charge(76);
        if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
            crate::cycle_ledger::charge(6 + 44);
            return;
        }
        self.dungeon_process_slash_doors_after_button_mask();
    }

    fn dungeon_process_slash_doors_after_button_mask(&mut self) {
        crate::cycle_ledger::charge(56);
        if self.game_state.player.follower_link.button_b_frames() != 4 {
            crate::cycle_ledger::charge(6 + 44);
            return;
        }

        crate::cycle_ledger::charge(
            118 + if self.game_state.player.follower_link.oam_y_offset_signed() < 0 {
                24
            } else {
                6
            },
        );
        crate::cycle_ledger::charge(
            240 + if self.game_state.player.follower_link.oam_x_offset_signed() < 0 {
                24
            } else {
                6
            },
        );
        crate::cycle_ledger::charge(294);
        let link_y =
            self.game_state.player.follower_link.y().wrapping_add(
                self.game_state.player.follower_link.oam_y_offset_signed() as i16 as u16,
            );
        let link_x =
            self.game_state.player.follower_link.x().wrapping_add(
                self.game_state.player.follower_link.oam_x_offset_signed() as i16 as u16,
            );
        let mut pos = (((link_y & 0x01f8) << 3) | ((link_x & 0x01f8) >> 3)) as usize;
        // A sword slash reaches a curtain or a slashable closed door in the
        // 2x2 cell around the player; the curtain's origin is its upper-left.
        let slashable = |state: &Self, pos: usize| match state
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_tile(pos)
            .dungeon_role()
        {
            DungeonRole::Curtain { .. } => {
                crate::cycle_ledger::charge(6);
                Some(None)
            }
            DungeonRole::ClosedDoor { slot } => {
                crate::cycle_ledger::charge(64 + 6);
                Some(Some(slot))
            }
            _ => {
                crate::cycle_ledger::charge(64);
                None
            }
        };
        let is_curtain = |state: &Self, pos: usize| {
            matches!(
                state
                    .game_state
                    .dungeon
                    .bg2_attributes
                    .bg2_tile(pos)
                    .dungeon_role(),
                DungeonRole::Curtain { .. }
            )
        };
        let mut y = 0x41u8;
        let mut hit = slashable(self, pos);
        if hit.is_none() {
            crate::cycle_ledger::charge(140);
            pos += 1;
            y = 0x40;
            hit = slashable(self, pos);
            if hit.is_none() {
                crate::cycle_ledger::charge(202);
                pos += 63;
                y = 1;
                hit = slashable(self, pos);
                if hit.is_none() {
                    crate::cycle_ledger::charge(140);
                    pos += 1;
                    y = 0;
                    hit = slashable(self, pos);
                }
            }
        }
        let Some(hit) = hit else {
            crate::cycle_ledger::charge(66);
            return;
        };

        crate::cycle_ledger::charge(72);
        let addr;
        if hit.is_none() {
            crate::cycle_ledger::charge(6 + 72);
            if y & 0x40 != 0 {
                crate::cycle_ledger::charge(248);
                pos -= 64;
                if !is_curtain(self, pos) {
                    crate::cycle_ledger::charge(66);
                    pos += 64;
                } else {
                    crate::cycle_ledger::charge(6);
                }
            } else {
                crate::cycle_ledger::charge(6);
            }
            crate::cycle_ledger::charge(48);
            if y & 1 != 0 {
                crate::cycle_ledger::charge(126);
                pos -= 1;
                if !is_curtain(self, pos) {
                    crate::cycle_ledger::charge(14);
                    pos += 1;
                } else {
                    crate::cycle_ledger::charge(6);
                }
            } else {
                crate::cycle_ledger::charge(6);
            }
            crate::cycle_ledger::charge(534);
            let DungeonRole::Curtain { panel } = self
                .game_state
                .dungeon
                .bg2_attributes
                .bg2_tile(pos)
                .dungeon_role()
            else {
                unreachable!("the curtain origin search only moves within the curtain");
            };
            self.write_attr2(pos + xy(0, 0), TilePair::repeated(NativeTile::SOLID_WALL));
            self.write_attr2(pos + xy(0, 1), TilePair::repeated(NativeTile::SOLID_WALL));
            addr = ((pos - xy(1, 1)) * 2) as u16;
            self.room_draw_object_nx4_bg2(
                4,
                DUNGEON_PROCESS_TORCHES_AND_DOORS_SOURCE_TILES1[panel] as usize,
                addr >> 1,
            );
            crate::cycle_ledger::charge(4 * 540 + 3 * 6 + 22); // Inline four-column draw and BRA.
        } else {
            crate::cycle_ledger::charge(24 + 250);
            self.dungeon_doors_mut().set_current_door_pos(pos as u16);
            let k = hit.unwrap_or_default();
            if self.game_state.dungeon.doors.door_type_and_slot(k) & 0xfe != DOOR_TYPE_SLASHABLE {
                crate::cycle_ledger::charge(6 + 66);
                return;
            }
            crate::cycle_ledger::charge(522);
            self.set_sound_effect_2(27);
            addr = self.game_state.dungeon.doors.door_tilemap_address(k);
            let opened_adj = self
                .game_state
                .dungeon
                .doors
                .opened_doors_including_adjacent()
                | upper_bitmask(k);
            self.dungeon_doors_mut()
                .set_opened_doors_including_adjacent(opened_adj);
            self.dungeon_doors_mut().mark_door_opened(k);
            self.dungeon_doors_mut().set_door_open_counter(0);
            self.dungeon_doors_mut().set_current_door_index_for_slot(k);
            self.dungeon_room_load_mut().set_selected_key_door(k);
            self.room_draw_object_nx4_bg2(4, DOOR_TYPE_SRC_UP[0x56 / 2] as usize, addr >> 1);
            crate::cycle_ledger::charge(86);
            {
                let _scope = crate::cycle_ledger::routine(0x01_d51c);
                self.dungeon_load_toggle_door_attr_other_entry(k as i32);
            }
        }

        crate::cycle_ledger::charge(46);
        {
            let _scope = crate::cycle_ledger::routine(0x01_f764);
            self.dungeon_prep_overlay_dma_next_prep(0, addr);
        }
        crate::cycle_ledger::charge(138);
        let sound_effect_1 = 30 | {
            let _scope = crate::cycle_ledger::routine(0x0d_bbd0);
            self.calculate_sfx_pan_arbitrary(((addr & 0x007f) * 2) as u8)
        };
        crate::cycle_ledger::charge(234 + 66);
        self.set_sound_effect_1(sound_effect_1);
        self.request_nmi_copy_packets();
    }
    fn dungeon_try_small_key_door(&mut self, door: usize, pos: usize, door_type: u8) -> bool {
        crate::cycle_ledger::charge(40); // The direct $1C branch precedes the $1E big-key gate.
        if door_type == DOOR_TYPE_SMALL_KEY_DOOR {
            crate::cycle_ledger::charge(6);
        } else {
            crate::cycle_ledger::charge(40 + 6 + 40);
            if door_type < DOOR_TYPE_SMALL_KEY_DOOR {
                crate::cycle_ledger::charge(6 + 24);
                return false;
            }
            crate::cycle_ledger::charge(40);
            if door_type >= 0x2c {
                crate::cycle_ledger::charge(6 + 24);
                return false;
            }
            crate::cycle_ledger::charge(40);
            if door_type == 0x2a {
                crate::cycle_ledger::charge(6 + 24);
                return false;
            }
        }
        crate::cycle_ledger::charge(88);
        if self.game_state.inventory.player_resources.keys() == 0 {
            crate::cycle_ledger::charge(6 + 24);
            return false;
        }
        crate::cycle_ledger::charge(110);
        self.player_resources_mut().decrement_keys();
        self.dungeon_publish_key_door_open(door, pos);
        true
    }

    fn dungeon_publish_key_door_open(&mut self, door: usize, pos: usize) {
        crate::cycle_ledger::charge(416); // Shared $01:CF6B..CF8D publication and RTL.
        self.dungeon_doors_mut().clear_door_animation_step();
        self.dungeon_doors_mut().set_current_door_pos(pos as u16);
        self.set_submodule(4);
        let pan = DUNGEON_PROCESS_TORCHES_AND_DOORS_OPEN_DOOR_PANNING
            [(self.game_state.dungeon.doors.door_direction(door) & 3) as usize];
        self.set_sound_effect_2(20 | pan);
    }

    pub(super) fn dungeon_clear_away_exploding_wall(&mut self) {
        self.follower_link_state_mut().set_immobilized_flag(6);
        self.set_modal_pause_flag(6);
        if self
            .game_state
            .effects
            .entrance_effects
            .blast_wall_explosion_slot(0)
            .phase()
            != 6
        {
            return;
        }

        self.dungeon_doors_mut().clear_door_barrier_or_switch_flag();
        self.tile_detect_position_mut().clear_slope_collision_bits();
        self.dungeon_doors_mut().clear_door_animation_step();
        let blast_door_x2 = self
            .game_state
            .dungeon
            .room_effects
            .blast_wall_door_index_x2();
        self.dungeon_doors_mut()
            .set_current_door_index(blast_door_x2);

        let door = usize::from(blast_door_x2 >> 1);
        let addr = self
            .game_state
            .dungeon
            .doors
            .door_tilemap_address(door)
            .wrapping_sub(2);
        self.dungeon_doors_mut()
            .set_door_tilemap_address(door, addr);
        let dsto = usize::from(addr >> 1);

        self.door_blast_wall_exploding_draw(dsto);
        self.ClearAndStripeExplodingWall(dsto as u16);

        self.set_core_update_disable_flag_word(0xffff);
        let walls2 = self
            .game_state
            .dungeon
            .room_effects
            .crush_wall_progress()
            .wrapping_add(2);
        self.dungeon_room_effects_mut()
            .set_crush_wall_progress(walls2);

        if walls2 == 21 {
            let mask = upper_bitmask(door);
            let opened_adj = self
                .game_state
                .dungeon
                .doors
                .opened_doors_including_adjacent()
                | mask;
            self.dungeon_doors_mut()
                .set_opened_doors_including_adjacent(opened_adj);
            self.dungeon_doors_mut().or_opened_doors(mask);

            if self.game_state.dungeon.doors.door_direction(door) & 2 != 0 {
                self.dungeon_room_effects_mut().mark_blast_wall_x_open();
                self.force_horizontal_fullsize_for_blast_wall();
            } else {
                self.dungeon_room_effects_mut().mark_blast_wall_y_open();
                self.force_vertical_fullsize_for_blast_wall();
            }
            self.cache_quadrant_fullsize_state();
            self.door_load_blast_wall_attr(door);
            self.dungeon_room_effects_mut().set_crush_wall_progress(0);
            self.dungeon_room_effects_mut()
                .clear_blast_wall_door_index();
            self.dungeon_flag_room_data_quadrants();
            self.follower_link_state_mut().clear_immobilized();
            self.clear_modal_pause_flag();
        }
        self.set_nmi_copy_packets_request(3);
    }
}
