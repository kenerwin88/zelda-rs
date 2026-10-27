//! Room object drawing primitives, tilemap access, and lookup bytes.

use super::*;
use crate::game_state::ObjectRecord;
use crate::tile_definition::{DungeonRole, NativeTile};
use crate::types::Point16U;

impl ZeldaState {
    pub(crate) fn native_lamp_cone_table_byte(address: u32) -> Option<u8> {
        let (word, byte) = match address {
            0x00f53e..=0x00f545 => {
                let i = (address - 0x00f53e) as usize;
                (
                    ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_X_BASE_OFFSETS[i / 2],
                    i & 1,
                )
            }
            0x00f546..=0x00f54d => {
                let i = (address - 0x00f546) as usize;
                (
                    ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_Y_BASE_OFFSETS[i / 2],
                    i & 1,
                )
            }
            0x00f54e..=0x00f555 => {
                let i = (address - 0x00f54e) as usize;
                (
                    ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_ADJUSTMENTS[i / 2] as u16,
                    i & 1,
                )
            }
            0x00f556..=0x00f55d => {
                let i = (address - 0x00f556) as usize;
                (
                    ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_BASELINES[i / 2] as u16,
                    i & 1,
                )
            }
            0x00f55e..=0x00f565 => {
                let i = (address - 0x00f55e) as usize;
                (ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_CLAMPS[i / 2], i & 1)
            }
            _ => return None,
        };
        Some(word.to_le_bytes()[byte])
    }

    /// Immutable lookup bytes used by the retained push-block caller and children.
    pub(crate) fn native_push_block_table_byte(address: u32) -> Option<u8> {
        match address {
            0x01d7c0..=0x01d7c7 => {
                let i = (address - 0x01d7c0) as usize;
                Some(
                    DUNGEON_PUSH_BLOCK_HANDLER_PUSH_BLOCK_MOVE_DISTANCES[i / 2].to_le_bytes()
                        [i & 1],
                )
            }
            0x07edf4..=0x07edf8 => Some(9),
            0x07ee30 => Some(12),
            0x07ee31..=0x07ee34 => {
                Some(PUSH_BLOCK_APPLY_VELOCITY_PUSHED_BLOCK_DIR_MASK[(address - 0x07ee31) as usize])
            }
            0x07ef61..=0x07ef64 => Some(
                PUSH_BLOCK_APPLY_VELOCITY_PUSH_BLOCK_X_RECOIL_BY_DIRECTION
                    [(address - 0x07ef61) as usize],
            ),
            0x07ef65..=0x07ef68 => Some(
                PUSH_BLOCK_APPLY_VELOCITY_PUSH_BLOCK_Y_RECOIL_BY_DIRECTION
                    [(address - 0x07ef65) as usize],
            ),
            _ => {
                let (words, base): (&[u16], u32) = match address {
                    0x07ef69..=0x07ef70 => (&PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_E, 0x07ef69),
                    0x07ef71..=0x07ef78 => (&PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_A, 0x07ef71),
                    0x07ef79..=0x07ef80 => (&PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_B, 0x07ef79),
                    0x07ef81..=0x07ef88 => (&PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_F, 0x07ef81),
                    0x07ef89..=0x07ef90 => (&PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_C, 0x07ef89),
                    0x07ef91..=0x07ef98 => (&PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_D, 0x07ef91),
                    0x07ef99..=0x07efa0 => (&[0x20, 0x20, 0x22, 0x22], 0x07ef99),
                    0x07efa1..=0x07efa8 => (&[0x22, 0x22, 0x20, 0x20], 0x07efa1),
                    0x07efa9..=0x07efb0 => (&[0x72, 0x72, 0x74, 0x74], 0x07efa9),
                    0x07efb1..=0x07efb8 => (&[0x74, 0x74, 0x72, 0x72], 0x07efb1),
                    _ => return None,
                };
                let i = (address - base) as usize;
                Some(words[i / 2].to_le_bytes()[i & 1])
            }
        }
    }

    /// ROM: `RoomDraw_1x3_rightwards`
    pub(super) fn room_draw_1x3_rightwards(&mut self, src: usize, dsto: u16, columns: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01_8d80));
        if self.lag_free {
            // $01:8D80..8D9D: setup/RTS, three stores per column, taken BNEs.
            crate::cycle_ledger::charge(74 + 470 * u64::from(columns)
                + 6 * u64::from(columns.saturating_sub(1)));
        }
        for x in 0..columns {
            for y in 0..3 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 3 + y) as usize),
                );
            }
        }
    }

    /// ROM: `Object_Draw_5x4`
    pub(super) fn object_draw_5x4(&mut self, src: usize, dsto: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01_9aee));
        if self.lag_free {
            // $01:9AEE..9B17: five four-store rows, four taken loop branches.
            crate::cycle_ledger::charge(56 + 5 * 610 + 4 * 6 + 42);
        }
        for y in 0..5 {
            for x in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    /// ROM: `RoomDraw_RightwardShelfEnd`
    pub(super) fn room_draw_rightward_shelf_end(&mut self, src: usize, dsto: u16) {
        if self.lag_free {
            // $01:B2E1..B2F5 four tile pairs and RTS (M16/X16).
            crate::cycle_ledger::charge(450);
        }
        for y in 0..4 {
            self.room_write_current(dsto + y * 64, self.tile_word(src, y as usize));
        }
    }

    /// ROM: `RoomDraw_RightwardBarSegment`
    pub(super) fn room_draw_rightward_bar_segment(&mut self, src: usize, dsto: u16) -> u16 {
        if self.lag_free {
            // $01:B2F6..B305 three tile pairs and RTS (M16/X16).
            crate::cycle_ledger::charge(348);
        }
        for y in 0..3 {
            self.room_write_current(dsto + y * 64, self.tile_word(src, y as usize));
        }
        dsto
    }

    /// ROM: `RoomDraw_DrawObject2x2and1`
    pub(super) fn room_draw_draw_object2x2and1(&mut self, src: usize, dsto: u16) -> u16 {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01_b279));
        if self.lag_free {
            crate::cycle_ledger::charge(552); // $01:B279..B292: five pairs and RTS.
        }
        for y in 0..5 {
            self.room_write_current(dsto + y * 64, self.tile_word(src, y as usize));
        }
        dsto
    }

    #[track_caller]
    /// ROM: `RoomDraw_Downwards4x2VariableSpacing`
    pub(super) fn room_draw_downwards4x2_variable_spacing(
        &mut self,
        increment: u16,
        src: usize,
        dsto: u16,
        count: u16,
    ) {
        let mut work = Dungeon4x2DrawWork {
            base: self.game_state.dungeon.room_tilemaps.first_line_pointer_row0() as usize,
            increment, src, dsto, count, completed_words: 0,
        };
        if let Some(boundary) = self.pending_dungeon_object_4x2_boundary.take() {
            assert!(self.active_dungeon_object_4x2.is_none());
            self.advance_dungeon_4x2_draw_boundary(&mut work, boundary);
            self.active_dungeon_object_4x2 = Some(work);
            return;
        }
        if self.lag_free {
            // $01:B220..B253: spacing store, eight load/store pairs per
            // actual block, pointer/counter control, taken BNEs and RTS.
            // A retained source boundary returned above; its staged owner
            // does not enter this completed eager-body charge.
            crate::cycle_ledger::charge(32 + 960 * u64::from(count)
                + 6 * u64::from(count.saturating_sub(1)) + 42);
        }
        self.advance_dungeon_4x2_draw_words(&mut work, count * 8);
        self.dungeon_room_load_mut().set_draw_width_indicator(0);
    }

    pub(super) fn advance_dungeon_4x2_draw_boundary(
        &mut self,
        work: &mut Dungeon4x2DrawWork,
        boundary: Dungeon4x2DrawBoundary,
    ) {
        let blocks = work.count.checked_sub(boundary.remaining_blocks)
            .expect("4x2 source boundary exceeds the native object's block count");
        assert!(boundary.words_in_block <= 8);
        let target = blocks * 8 + u16::from(boundary.words_in_block);
        self.advance_dungeon_4x2_draw_words(work, target);
        self.dungeon_room_load_mut().set_draw_width_indicator_word(work.count - blocks);
    }

    pub(super) fn advance_dungeon_4x2_draw_words(&mut self, work: &mut Dungeon4x2DrawWork, target: u16) {
        assert!(work.completed_words <= target && target <= work.count * 8);
        while work.completed_words < target {
            let block = work.completed_words / 8;
            let field = work.completed_words % 8;
            // The source stores src[0..4] on the first row, then src[4..8]
            // on the second row. Overlapping blocks retain that order too.
            let dst = work.dsto.wrapping_add(block.wrapping_mul(work.increment))
                .wrapping_add(field % 4 + (field / 4) * 64);
            self.room_write_bg(work.base, dst, self.tile_word(work.src, usize::from(field)));
            work.completed_words += 1;
        }
    }

    /// ROM: `Object_Table_Helper`
    pub(super) fn object_table_helper(&mut self, src: usize, dsto: u16, width: u16) {
        self.room_write_current(dsto, self.tile_word(src, 0));
        for segment in 0..width {
            let dst = dsto + 1 + segment * 2;
            self.room_write_current(dst, self.tile_word(src, 1));
            self.room_write_current(dst + 1, self.tile_word(src, 2));
        }
        self.room_write_current(dsto + 1 + width * 2, self.tile_word(src, 3));
    }

    /// ROM: `RoomDraw_CheckIfWallIsMoved`
    pub(super) fn room_draw_check_if_wall_is_moved(&mut self) -> bool {
        self.bg1_move_calc_mut().set_buffer(0);
        self.dungeon_moving_floor_mut().clear_floor_move_flags();

        let tag0 = self.game_state.dungeon.header.primary_header_tag();
        let tag1 = self.game_state.dungeon.header.header_tag(1);
        let i = if (0x1c..0x20).contains(&tag0) {
            Some(0usize)
        } else if (0x1c..0x20).contains(&tag1) {
            Some(1usize)
        } else {
            None
        };

        if let Some(i) = i {
            if self.game_state.dungeon.savegame_state.savegame_state_bits() & (0x1000 >> i) != 0 {
                self.dungeon_room_load_mut().set_header_collision(0);
                self.dungeon_header_mut().clear_header_tag(i);
                self.dungeon_room_load_mut().clear_bg2_properties();
                return false;
            }
        }
        true
    }

    /// ROM: `MovingWall_FillReplacementBuffer`
    pub(super) fn moving_wall_fill_replacement_buffer(&mut self, dsto: u16) {
        self.dungeon_room_effects_mut()
            .fill_moving_wall_replacement_buffer(0x01ec);
        let value = (dsto & 0x001f) | if dsto & 0x0020 != 0 { 0x0400 } else { 0 } | 0x1000;
        self.dungeon_room_effects_mut()
            .set_moving_wall_write_point(value);
    }

    /// ROM: `RoomDraw_MovingWallRight`
    pub(super) fn room_draw_moving_wall_right(&mut self, width: u8, height: u8, dsto: u16) {
        if !self.room_draw_check_if_wall_is_moved() {
            return;
        }
        self.dungeon_room_load_mut()
            .increment_header_collision_2_mirror();
        let size0 = MOVING_WALL_SIZE_TABLE0[width as usize];
        let size1 = MOVING_WALL_SIZE_TABLE1[height as usize];
        self.moving_wall_fill_replacement_buffer(dsto.wrapping_sub(size1).wrapping_sub(1));
        self.dungeon_room_effects_mut()
            .set_moving_wall_dot_pointer(height.wrapping_mul(2));

        let fill_src = 0x03d8;
        let mut dst1 = dsto.wrapping_sub(size1);
        for _ in 0..size1 {
            let mut dst2 = dst1;
            self.room_write_current(dst2, self.tile_word(fill_src, 0));
            for _ in 0..(size0 * 2 + 4) {
                self.room_write_current(dst2 + 64, self.tile_word(fill_src, 1));
                dst2 += 64;
            }
            self.room_write_current(dst2 + 64, self.tile_word(fill_src, 2));
            dst1 += 1;
        }

        let src = 0x072a;
        let mut dst = dsto;
        self.room_draw_1x3_rightwards(src, dst, 3);
        dst += 3 * 64;
        for _ in 0..size0 {
            self.object_draw_3x2(src + 18, dst);
            dst += 2 * 64;
        }
        self.room_draw_1x3_rightwards(src + 30, dst, 3);
    }

    /// ROM: `RoomDraw_MovingWallLeft`
    pub(super) fn room_draw_moving_wall_left(&mut self, width: u8, height: u8, dsto: u16) {
        if !self.room_draw_check_if_wall_is_moved() {
            return;
        }
        self.dungeon_room_load_mut()
            .increment_header_collision_2_mirror();
        let size1 = MOVING_WALL_SIZE_TABLE1[height as usize];
        let size0 = MOVING_WALL_SIZE_TABLE0[width as usize];
        self.dungeon_room_effects_mut()
            .set_moving_wall_dot_pointer(height.wrapping_mul(2));
        self.moving_wall_fill_replacement_buffer(dsto.wrapping_add(3).wrapping_add(size1));

        let src = 0x075a;
        let mut dst = dsto;
        self.room_draw_1x3_rightwards(src, dst, 3);
        dst += 3 * 64;
        for _ in 0..size0 {
            self.object_draw_3x2(src + 18, dst);
            dst += 2 * 64;
        }
        self.room_draw_1x3_rightwards(src + 30, dst, 3);

        let fill_src = 0x03d8;
        let mut dst1 = dsto + 3;
        for _ in 0..size1 {
            let mut dst2 = dst1;
            self.room_write_current(dst2, self.tile_word(fill_src, 0));
            for _ in 0..(size0 * 2 + 4) {
                self.room_write_current(dst2 + 64, self.tile_word(fill_src, 1));
                dst2 += 64;
            }
            self.room_write_current(dst2 + 64, self.tile_word(fill_src, 2));
            dst1 += 1;
        }
    }

    /// ROM: `Object_DrawNx3_BothBgs`
    pub(super) fn object_draw_nx3_both_bgs(&mut self, n: u16, src: usize, dsto: u16) {
        for x in 0..n {
            for y in 0..3 {
                let tile = self.tile_word(src, (x * 3 + y) as usize);
                self.room_write_bg2(dsto + x + y * 64, tile);
                self.room_write_bg1(dsto + x + y * 64, tile);
            }
        }
    }

    pub(in crate::zelda_rtl) fn bomb_check_for_destructibles(&mut self, x: u16, y: u16, r14: u8) {
        if self.game_state.frame.main_module != 7 {
            self.overworld_bomb_tiles32x32(x, y);
            return;
        }

        let mut k = (((y & 0x01f8) << 3) | ((x & 0x01f8) >> 3)).wrapping_sub(0x0082) as usize;
        for _ in (0..=2).rev() {
            for step in 0..3 {
                let role = self
                    .game_state
                    .dungeon
                    .bg2_attributes
                    .bg2_tile(k)
                    .dungeon_role();
                if role == DungeonRole::BombableFloor {
                    if self.game_state.world.location.dungeon_room() == 0x65 {
                        let bits =
                            self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x1000;
                        self.dungeon_savegame_state_mut()
                            .set_savegame_state_bits(bits);
                    }
                    let mut pt = Point16U { x: 0, y: 0 };
                    self.thieves_attic_draw_lightened_hole(0, 0, &mut pt);
                    self.set_sound_effect_2(0x1b);
                    return;
                }
                if let DungeonRole::ClosedDoor { slot: j } = role {
                    let ty = self.game_state.dungeon.doors.door_type_and_slot(j) & 0xfe;
                    if ty != DOOR_TYPE_BREAKABLE_WALL && ty != 0x2a && ty != 0x2e {
                        return;
                    }
                    self.dungeon_doors_mut().set_current_door_pos(k as u16);
                    let addr = self.game_state.dungeon.doors.door_tilemap_address(j);
                    let door_x = ((addr & 0x007e) << 2)
                        .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_h());
                    let door_y = ((addr & 0x1f80) >> 4)
                        .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_v());
                    let r14 = r14 as usize;
                    self.door_debris_mut().set_x_word(r14, door_x);
                    self.door_debris_mut().set_y_word(r14, door_y);
                    let direction = self.game_state.dungeon.doors.door_direction(j) & 3;
                    self.door_debris_mut().set_direction(r14, direction);
                    self.set_sound_effect_2(0x1b);
                    self.set_submodule(9);
                    return;
                }
                if step != 2 {
                    k = k.wrapping_add(2);
                }
            }
            k = k.wrapping_add(0x7c);
        }
    }

    pub(in crate::zelda_rtl) fn prepare_dungeon_exit_from_boss_fight(&mut self) {
        self.SavePalaceDeaths();
        self.save_dungeon_keys_misc();
        let bits = self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x8000;
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits(bits);
        self.dungeon_flag_room_data_quadrants();

        let room = self.game_state.world.location.dungeon_room_index();
        let j = DUNGEON_EXIT_SOURCE_ROOMS
            .iter()
            .position(|&from| from == room)
            .expect("dungeon room must have a boss-exit mapping");
        self.set_dungeon_room_index(DUNGEON_EXIT_TARGET_ROOMS[j]);
        if self.game_state.world.location.dungeon_room_index() == 0x20 {
            self.save_progress_mut().set_progress_indicator(3);
            self.set_overworld_event_bits(2, 0x20);
            self.save_progress_mut().xor_dark_world_state(0x40);
            self.sprite_load_graphics_properties_light_world_only();
            self.ancilla_terminate_select_interactives(0);
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
            self.follower_link_state_mut().clear_button_b_frames();
            self.follower_link_state_mut().set_button_mask_b_y(0);
            self.follower_link_state_mut().clear_force_hold_sword_up();
            self.follower_link_state_mut().immobilize();
            self.set_saved_module_for_menu(8);
            self.set_main_module(21);
            self.set_submodule(0);
            self.set_subsubmodule(0);
        } else if self.game_state.world.location.dungeon_room_index() == 0x0d {
            self.set_main_module(24);
            self.set_submodule(0);
            self.set_overworld_map_state(0);
            self.set_color_math_control(0x20);
        } else {
            if j >= 3 {
                self.set_music_control(0xf1);
                self.set_current_music_control(0xf1);
                self.set_main_module(22);
            } else {
                self.set_main_module(19);
            }
            self.set_saved_module_for_menu(8);
            self.set_submodule(0);
            self.set_subsubmodule(0);
        }
    }

    /// ROM: `Object_BombableFloorHelper`
    pub(super) fn object_bombable_floor_helper(
        &mut self,
        record: ObjectRecord,
        src: usize,
        src_below: usize,
        dsto: u16,
    ) {
        let index = self.game_state.dungeon.object_tracking.misc_object_index() as usize >> 1;
        self.dungeon_object_tracking_mut()
            .set_object_record(index, record);
        let next = self
            .game_state
            .dungeon
            .object_tracking
            .misc_object_index()
            .wrapping_add(2);
        self.dungeon_object_tracking_mut()
            .set_misc_object_index(next);
        let load_ptr = self.game_state.dungeon.room_load.load_ptr_offset();
        self.dungeon_object_tracking_mut()
            .set_object_data_pos(index, load_ptr);
        let tilemap_pos = (dsto * 2) | self.room_plane_tilemap_bit();
        let below = [
            self.tile_word(src_below, 0),
            self.tile_word(src_below, 1),
            self.tile_word(src_below, 2),
            self.tile_word(src_below, 3),
        ];
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(index, tilemap_pos);
        self.dungeon_room_items_mut()
            .set_replacement_tilemap_quad(index, below);
        self.room_draw_rightwards2x2(src, dsto);
    }

    /// ROM: `RoomDraw_BombableFloor`
    pub(super) fn room_draw_bombable_floor(&mut self, dsto: u16) {
        if self.game_state.world.location.dungeon_room() == 101
            && self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x1000 != 0
        {
            self.dungeon_room_load_mut().set_draw_width_indicator(0);
            self.dungeon_room_load_mut().set_draw_height_indicator(0);
            self.object_hole(0x05aa, dsto, 0, 0);
            return;
        }

        let src = 0x0220;
        let src_below = 0x05ba;
        self.object_bombable_floor_helper(
            ObjectRecord::bombable_floor_segment(0),
            src,
            src_below,
            dsto,
        );
        self.object_bombable_floor_helper(
            ObjectRecord::bombable_floor_segment(1),
            src + 8,
            src_below + 8,
            dsto + xy(2, 0) as u16,
        );
        self.object_bombable_floor_helper(
            ObjectRecord::bombable_floor_segment(2),
            src + 16,
            src_below + 16,
            dsto + xy(0, 2) as u16,
        );
        self.object_bombable_floor_helper(
            ObjectRecord::bombable_floor_segment(3),
            src + 24,
            src_below + 24,
            dsto + xy(2, 2) as u16,
        );
    }

    /// ROM: `RoomDraw_HammerPegSingle`
    pub(super) fn room_draw_hammer_peg_single(&mut self, src: usize, dsto: u16) {
        let index = self.game_state.dungeon.object_tracking.misc_object_index() as usize >> 1;
        let next = self
            .game_state
            .dungeon
            .object_tracking
            .misc_object_index()
            .wrapping_add(2);
        self.dungeon_object_tracking_mut()
            .set_misc_object_index(next);
        self.dungeon_object_tracking_mut()
            .set_object_record(index, ObjectRecord::HAMMER_PEG);
        let load_ptr = self.game_state.dungeon.room_load.load_ptr_offset();
        self.dungeon_object_tracking_mut()
            .set_object_data_pos(index, load_ptr);
        let plane = if self
            .game_state
            .dungeon
            .room_tilemaps
            .first_line_pointer_row0()
            != 0x4000
        {
            0
        } else {
            0x2000
        };
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(index, (dsto * 2) | plane);
        self.dungeon_room_items_mut()
            .set_replacement_tilemap_quad(index, [0x19d8, 0x19d9, 0x59d8, 0x59d9]);
        self.room_draw_rightwards2x2(src, dsto);
    }

    /// ROM: `Object_SizeAtoAplus15`
    pub(super) fn object_size_ato_aplus15(&mut self, a: u8) {
        let draw_width = (self.game_state.dungeon.room_load.draw_width_indicator() << 2
            | self.game_state.dungeon.room_load.draw_height_indicator())
        .wrapping_add(a);
        self.dungeon_room_load_mut()
            .set_draw_width_indicator(draw_width);
        self.dungeon_room_load_mut().set_draw_height_indicator(0);
    }

    pub(super) fn DrawWaterThing(&mut self, dsto: u16, src: usize) {
        for y in 0..4 {
            for x in 0..4 {
                self.room_write_current(
                    dsto + y * 64 + x,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    pub(super) fn DrawWaterThingBg(&mut self, base: usize, dsto: u16, src: usize) {
        for y in 0..4 {
            for x in 0..4 {
                self.room_write_bg(
                    base,
                    dsto + y * 64 + x,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    /// ROM: `RoomDraw_FortuneTellerRoom`
    pub(super) fn room_draw_fortune_teller_room(&mut self, dsto: u16) {
        let src_org = 0x202eusize;
        let mut src = src_org;
        let mut d = dsto;

        for _ in 0..6 {
            let tile0 = self.tile_word(src, 0);
            self.room_write_bg2(d + xy(1, 0) as u16, tile0);
            self.room_write_bg2(d + xy(2, 0) as u16, tile0);
            self.room_write_bg2(d + xy(1, 1) as u16, tile0);
            self.room_write_bg2(d + xy(2, 1) as u16, tile0);
            let tile1 = self.tile_word(src, 1);
            self.room_write_bg2(d + xy(1, 2) as u16, tile1);
            self.room_write_bg2(d + xy(2, 2) as u16, tile1 | 0x4000);
            d = d.wrapping_add(xy(2, 0) as u16);
        }
        d = d.wrapping_sub((xy(2, 0) * 6) as u16);

        for _ in 0..3 {
            let tile2 = self.tile_word(src, 2);
            for &off in &[xy(0, 3), xy(2, 3), xy(10, 3), xy(12, 3)] {
                self.room_write_bg2(d + off as u16, tile2);
            }
            for &off in &[xy(1, 3), xy(3, 3), xy(11, 3), xy(13, 3)] {
                self.room_write_bg2(d + off as u16, tile2 | 0x4000);
            }
            let tile5 = self.tile_word(src, 5);
            for &off in &[xy(4, 3), xy(6, 3), xy(8, 3)] {
                self.room_write_bg2(d + off as u16, tile5);
            }
            for &off in &[xy(5, 3), xy(7, 3), xy(9, 3)] {
                self.room_write_bg2(d + off as u16, tile5 | 0x4000);
            }
            src += 2;
            d = d.wrapping_add(xy(0, 1) as u16);
        }
        d = d.wrapping_sub((xy(0, 1) * 3) as u16);

        let tile5 = self.tile_word(src, 5);
        self.room_write_bg2(d + xy(0, 0) as u16, tile5);
        self.room_write_bg2(d + xy(0, 1) as u16, tile5);
        self.room_write_bg2(d + xy(13, 0) as u16, tile5 | 0x4000);
        self.room_write_bg2(d + xy(13, 1) as u16, tile5 | 0x4000);
        let tile6 = self.tile_word(src, 6);
        self.room_write_bg2(d + xy(0, 2) as u16, tile6);
        self.room_write_bg2(d + xy(13, 2) as u16, tile6 | 0x4000);

        src = src_org;
        for _ in 0..4 {
            let tile10 = self.tile_word(src, 10);
            self.room_write_bg2(d + xy(3, 10) as u16, tile10);
            self.room_write_bg2(d + xy(10, 10) as u16, tile10 ^ 0x4000);
            let tile14 = self.tile_word(src, 14);
            self.room_write_bg2(d + xy(4, 10) as u16, tile14);
            self.room_write_bg2(d + xy(9, 10) as u16, tile14 ^ 0x4000);
            let tile18 = self.tile_word(src, 18);
            self.room_write_bg2(d + xy(5, 10) as u16, tile18);
            self.room_write_bg2(d + xy(8, 10) as u16, tile18 ^ 0x4000);
            let tile22 = self.tile_word(src, 22);
            self.room_write_bg2(d + xy(6, 10) as u16, tile22);
            self.room_write_bg2(d + xy(7, 10) as u16, tile22 ^ 0x4000);
            src += 2;
            d = d.wrapping_add(xy(0, 1) as u16);
        }
    }

    /// ROM: `RoomDraw_PrisonCell`
    pub(super) fn room_draw_prison_cell(&mut self, dsto: u16) {
        if self.lag_free {
            // $01:9C44..9CC5 M16/X16: plane branch, five eight-store
            // columns, four taken loop branches, fixed stores and RTS.
            crate::cycle_ledger::charge(86 + if self.room_plane_offset() != 0 { 52 } else { 6 }
                + 110 + 5 * 738 + 4 * 6 + 648);
        }
        let src = 0x1488;
        let dsto = dsto | self.room_plane_offset();
        for i in 0..5 {
            let d = dsto + i;
            self.room_write_bg2(d + xy(2, 0) as u16, self.tile_word(src, 1));
            self.room_write_bg2(d + xy(9, 0) as u16, self.tile_word(src, 1));
            self.room_write_bg2(d + xy(2, 1) as u16, self.tile_word(src, 2));
            self.room_write_bg2(d + xy(9, 1) as u16, self.tile_word(src, 2) | 0x4000);
            self.room_write_bg2(d + xy(2, 2) as u16, self.tile_word(src, 4));
            self.room_write_bg2(d + xy(9, 2) as u16, self.tile_word(src, 4) | 0x4000);
            self.room_write_bg2(d + xy(2, 3) as u16, self.tile_word(src, 5));
            self.room_write_bg2(d + xy(9, 3) as u16, self.tile_word(src, 5) | 0x4000);
        }
        self.room_write_bg2(dsto, self.tile_word(src, 0));
        self.room_write_bg(
            0x2000,
            dsto + xy(15, 0) as u16,
            self.tile_word(src, 0) | 0x4000,
        );
        for offset in [xy(1, 0), xy(7, 0), xy(8, 0), xy(14, 0)] {
            self.room_write_bg2(dsto + offset as u16, self.tile_word(src, 1));
        }
        self.room_write_bg2(dsto + xy(1, 2) as u16, self.tile_word(src, 3));
        self.room_write_bg(
            0x2000,
            dsto + xy(14, 2) as u16,
            self.tile_word(src, 3) | 0x4000,
        );
    }

    /// ROM: `RoomDraw_CellLock`
    pub(super) fn room_draw_cell_lock(&mut self, dsto: u16) {
        let index = self.dungeon_room_items_mut().advance_big_key_lock_count();
        if index < DUNGEON_CHEST_OPEN_MASKS.len()
            && self.game_state.dungeon.savegame_state.savegame_state_bits()
                & DUNGEON_CHEST_OPEN_MASKS[index]
                == 0
        {
            self.dungeon_room_items_mut()
                .set_chest_location(index, dsto * 2);
            self.room_draw_rightwards2x2(0x1494, dsto);
        } else if index < 6 {
            self.dungeon_room_items_mut().clear_chest_location(index);
        }
    }

    /// ROM: `RoomDraw_LowerDoorStairsUp`
    pub(super) fn room_draw_lower_door_stairs_up(
        &mut self,
        mut src: usize,
        mut dsto: u16,
        from_upnorth: bool,
    ) {
        let stair_list = if from_upnorth {
            DungeonStairList::InterRoomUpNorthStraight
        } else {
            DungeonStairList::InterRoomDownNorthStraight
        };
        let plane = self.room_plane_offset();
        let next = self
            .dungeon_stair_lists_mut()
            .append_interroom_staircase(stair_list, dsto | plane);
        if from_upnorth {
            self.dungeon_stair_lists_mut().sync_stair_list_counts(
                &[
                    DungeonStairList::InterRoomUpNorthStraight,
                    DungeonStairList::InterRoomUpSouthStraight,
                    DungeonStairList::InterRoomSouthDown,
                    DungeonStairList::WallDownNorthSpiral,
                    DungeonStairList::WallDownNorthSpiralBg1,
                    DungeonStairList::InterRoomDownNorthStraight,
                    DungeonStairList::InterRoomDownSouthStraight,
                ],
                next,
            );
        } else {
            self.dungeon_stair_lists_mut().sync_stair_list_counts(
                &[
                    DungeonStairList::InterRoomDownNorthStraight,
                    DungeonStairList::InterRoomDownSouthStraight,
                ],
                next,
            );
        }

        for _ in 0..4 {
            self.room_write_bg1(dsto, self.tile_word(src, 0));
            self.room_write_bg2(dsto, self.tile_word(src, 0));
            self.room_write_bg1(dsto + 64, self.tile_word(src, 1));
            self.room_write_bg1(dsto + 2 * 64, self.tile_word(src, 2));
            self.room_write_bg1(dsto + 3 * 64, self.tile_word(src, 3));
            src += 8;
            dsto = dsto.wrapping_add(1);
        }
        let priority = dsto.wrapping_sub(4).wrapping_sub(4 * 64);
        self.room_draw_lower_door_bg2_priority(priority);
    }

    /// ROM: `RoomDraw_LowerDoorStairsDown`
    pub(super) fn room_draw_lower_door_stairs_down(
        &mut self,
        mut src: usize,
        mut dsto: u16,
        from_upsouth: bool,
    ) {
        let stair_list = if from_upsouth {
            DungeonStairList::InterRoomUpSouthStraight
        } else {
            DungeonStairList::InterRoomDownSouthStraight
        };
        let plane = self.room_plane_offset();
        let next = self
            .dungeon_stair_lists_mut()
            .append_interroom_staircase(stair_list, dsto | plane);
        if from_upsouth {
            self.dungeon_stair_lists_mut().sync_stair_list_counts(
                &[
                    DungeonStairList::InterRoomUpSouthStraight,
                    DungeonStairList::InterRoomSouthDown,
                    DungeonStairList::WallDownNorthSpiral,
                    DungeonStairList::WallDownNorthSpiralBg1,
                    DungeonStairList::InterRoomDownNorthStraight,
                    DungeonStairList::InterRoomDownSouthStraight,
                ],
                next,
            );
        } else {
            self.dungeon_stair_lists_mut()
                .set_stair_list_count(DungeonStairList::InterRoomDownSouthStraight, next);
        }

        for _ in 0..4 {
            self.room_write_bg1(dsto, self.tile_word(src, 0));
            self.room_write_bg1(dsto + 64, self.tile_word(src, 1));
            self.room_write_bg1(dsto + 2 * 64, self.tile_word(src, 2));
            self.room_write_bg1(dsto + 3 * 64, self.tile_word(src, 3));
            self.room_write_bg2(dsto + 3 * 64, self.tile_word(src, 3));
            src += 8;
            dsto = dsto.wrapping_add(1);
        }
        let priority = dsto.wrapping_sub(4).wrapping_add(4 * 64);
        self.room_draw_lower_door_bg2_priority(priority);
    }

    /// ROM: `RoomDraw_LowerDoorBg2Priority`
    pub(super) fn room_draw_lower_door_bg2_priority(&mut self, dsto: u16) {
        for y in 0..4 {
            let pos = dsto + y * 64;
            let tile = self.room_read_bg2(pos) | 0x2000;
            self.room_write_bg2(pos, tile);
        }
    }

    /// ROM: `RoomDraw_Chest`
    pub(super) fn room_draw_chest(&mut self, dsto: u16) {
        if self.game_state.frame.main_module == 26 {
            return;
        }
        let index = self
            .dungeon_room_items_mut()
            .advance_chest_and_big_key_counts();
        if index >= DUNGEON_CHEST_OPEN_MASKS.len() {
            return;
        }
        let location = 2 * (dsto | self.room_plane_offset());
        self.dungeon_room_items_mut()
            .set_chest_location(index, location);
        let tag_slot = self.chest_tag_gate_slot();
        if self.game_state.dungeon.savegame_state.savegame_state_bits()
            & DUNGEON_CHEST_OPEN_MASKS[index]
            == 0
        {
            if let Some(slot) = tag_slot {
                if self.game_state.dungeon.savegame_state.savegame_state_bits()
                    & DUNGEON_CHEST_OPEN_MASKS[slot]
                    == 0
                {
                    return;
                }
                self.dungeon_header_mut().clear_header_tag(slot);
            }
            self.room_draw_rightwards2x2(0x149c, dsto);
        } else {
            self.dungeon_room_items_mut().clear_chest_location(index);
            if let Some(slot) = tag_slot {
                self.dungeon_header_mut().clear_header_tag(slot);
            }
            self.room_draw_rightwards2x2(0x14a4, dsto);
        }
    }

    pub(super) fn chest_tag_gate_slot(&self) -> Option<usize> {
        for slot in 0..2 {
            let tag = self.game_state.dungeon.header.header_tag(slot);
            if tag == 0x27 || tag == 0x3c || tag == 0x3e || (0x29..0x33).contains(&tag) {
                return Some(slot);
            }
        }
        None
    }

    /// ROM: `RoomDraw_SinglePot`
    pub(super) fn room_draw_single_pot(&mut self, src: usize, dsto: u16) {
        if self.lag_free {
            // $01:B395..B3DE, M16/X16; JMP enters the already priced 2x2 body.
            let plane = self.game_state.dungeon.room_tilemaps.first_line_pointer_row0();
            let world = self.game_state.inventory.save_progress.dark_world_state();
            crate::cycle_ledger::charge(474 + if plane == 0x4000 { 84 } else { 6 }
                + 380 + if world != 0 { 24 } else { 6 } + 24);
        }
        let index = self.game_state.dungeon.object_tracking.misc_object_index() as usize >> 1;
        self.dungeon_object_tracking_mut()
            .set_misc_object_index(((index + 1) * 2) as u16);
        self.dungeon_object_tracking_mut()
            .set_object_record(index, ObjectRecord::POT);
        let load_ptr = self.game_state.dungeon.room_load.load_ptr_offset();
        self.dungeon_object_tracking_mut()
            .set_object_data_pos(index, load_ptr);
        let plane_bit = self.room_plane_tilemap_bit();
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(index, (dsto * 2) | plane_bit);
        self.dungeon_room_items_mut()
            .set_replacement_tilemap_quad(index, [0x0d0e, 0x0d1e, 0x4d0e, 0x4d1e]);
        let src = if self.game_state.inventory.save_progress.dark_world_state() != 0 {
            0x0e92
        } else {
            src
        };
        self.room_draw_rightwards2x2(src, dsto);
    }

    /// ROM: `Object_Draw_4x2_BothBgs`
    pub(super) fn object_draw_4x2_both_bgs(&mut self, src: usize, dsto: u16) {
        for y in 0..2 {
            for x in 0..4 {
                let tile = self.tile_word(src, (y * 4 + x) as usize);
                self.room_write_bg2(dsto + x + y * 64, tile);
                self.room_write_bg1(dsto + x + y * 64, tile);
            }
        }
    }

    /// ROM: `RoomDraw_Chest_platform_row`
    pub(super) fn room_draw_chest_platform_row(&mut self, src: usize, dsto: u16, width: u16) {
        self.room_write_bg2(dsto, self.tile_word(src, 0));
        let left_fill = self.tile_word(src, 3);
        for x in 0..width {
            self.room_write_bg2(dsto + 1 + x, left_fill);
        }
        self.room_write_bg2(dsto + 1 + width, self.tile_word(src, 6));

        let middle_fill = self.tile_word(src, 9);
        for x in 0..4 {
            self.room_write_bg2(dsto + 2 + width + x, middle_fill);
        }
        self.room_write_bg2(dsto + 6 + width, self.tile_word(src, 12));

        let right_fill = self.tile_word(src, 15);
        for x in 0..width {
            self.room_write_bg2(dsto + 7 + width + x, right_fill);
        }
        self.room_write_bg2(dsto + 7 + width * 2, self.tile_word(src, 18));
    }

    /// ROM: `RoomDraw_4x4`
    pub(super) fn room_draw_4x4(&mut self, src: usize, dsto: u16) {
        // Cycle ledger: RoomDraw_4x4 $01:97ED is `LDA #$0004` (24) falling
        // into RoomDraw_Object_Nx4 at $0197F0, so the body runs under this scope.
        let _scope = crate::cycle_ledger::routine(0x01_97ed);
        crate::cycle_ledger::charge(24);
        self.room_data_draw_object_nx4(src, dsto, 4);
    }

    /// ROM: `Object_Draw8x8`
    pub(super) fn object_draw8x8(&mut self, src: usize, dsto: u16) {
        if self.lag_free {
            // $01:A7DC..A7EF four JSRs, row reposition, RTS; children charge separately (M16/X16).
            crate::cycle_ledger::charge(310);
        }
        self.room_draw_4x4(src, dsto);
        self.room_draw_4x4(src + 32, dsto + 4);
        self.room_draw_4x4(src + 64, dsto + 4 * 64);
        self.room_draw_4x4(src + 96, dsto + 4 + 4 * 64);
    }

    /// ROM: `Object_Draw_3x2`
    pub(super) fn object_draw_3x2(&mut self, src: usize, dsto: u16) {
        if self.lag_free {
            // $01:9D04..9D28 two three-store rows and actual loop branch (M16/X16).
            crate::cycle_ledger::charge(56 + 2 * 526 + 6 + 42);
        }
        for y in 0..2 {
            for x in 0..3 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (y * 3 + x) as usize),
                );
            }
        }
    }

    /// ROM: `RoomDraw_WaterHoldingObject`
    pub(super) fn room_draw_water_holding_object(&mut self, n: u16, src: usize, dsto: u16) {
        if self.lag_free {
            // $01:9D6F..9D95 four-store rows, counted by the executed height (M16/X16).
            crate::cycle_ledger::charge(32 + 610 * u64::from(n) + 6 * u64::from(n.saturating_sub(1)) + 42);
        }
        for y in 0..n {
            for x in 0..4 {
                self.room_write_current(
                    dsto + y * 64 + x,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    /// ROM: `RoomDraw_SomeBigDecors`
    pub(super) fn room_draw_some_big_decors(&mut self, n: u16, src: usize, dsto: u16) {
        if self.lag_free {
            // $01:9DA2..9DD8 M16/X16: eight rows, n actual tile pairs
            // per row; the plane branch changes only the destination.
            crate::cycle_ledger::charge(104 + if self.room_plane_offset() != 0 { 88 } else { 6 }
                + 56 + 8 * (96 + 220 * u64::from(n)
                    + 6 * u64::from(n.saturating_sub(1)) + 172) + 7 * 6 + 42);
        }
        let mut dst = dsto | self.room_plane_offset();
        for y in 0..8 {
            for x in 0..n {
                self.room_write_bg2(dst + x, self.tile_word(src, (y * n + x) as usize));
            }
            dst += 64;
        }
    }

    /// ROM: `RoomDraw_SingleLampCone`
    pub(super) fn room_draw_single_lamp_cone(&mut self, a: u16, y: usize) {
        if self.lag_free {
            // $01:9E06..9E2F M16/X16: twelve rows of twelve tile pairs,
            // eleven taken inner branches per row and eleven outer ones.
            crate::cycle_ledger::charge(88 + 12 * (88 + 12 * 220 + 11 * 6 + 172) + 11 * 6 + 42);
        }
        for row in 0..12 {
            for col in 0..12 {
                self.room_write_bg(
                    0x4000,
                    a / 2 + row * 64 + col,
                    self.tile_word(y, (row * 12 + col) as usize),
                );
            }
        }
    }

    /// ROM: `RoomDraw_AgahnimsWindows`
    pub(super) fn room_draw_agahnims_windows(&mut self, dsto: u16) {
        let mut d = dsto;
        let mut src = 0x1bf2;
        for _ in 0..6 {
            for x in [7, 13, 19] {
                for y in 0..4 {
                    self.room_write_bg2(d + xy(x, 4 + y) as u16, self.tile_word(src, y));
                }
            }
            src += 8;
            d += 1;
        }
        d -= 6;

        src = 0x1c22;
        for _ in 0..5 {
            let tile = self.tile_word(src, 0);
            for (x, y) in [(2, 10), (3, 9), (4, 8), (5, 7), (6, 6), (7, 5), (8, 4)] {
                self.room_write_bg2(d + xy(x, y) as u16, tile);
            }
            for (x, y) in [
                (23, 4),
                (24, 5),
                (25, 6),
                (26, 7),
                (27, 8),
                (28, 9),
                (29, 10),
            ] {
                self.room_write_bg2(d + xy(x, y) as u16, tile | 0x4000);
            }
            src += 2;
            d += 64;
        }
        d -= 64 * 5;

        src = 0x1c2c;
        for _ in 0..6 {
            for k in 0..4 {
                let tile = self.tile_word(src, k);
                for y in [11, 17, 23] {
                    self.room_write_bg2(d + xy(2 + k, y) as u16, tile);
                    self.room_write_bg2(d + xy(29 - k, y) as u16, tile | 0x4000);
                }
            }
            src += 8;
            d += 64;
        }
        d -= 64 * 6;

        src = 0x1c5c;
        for _ in 0..6 {
            let top = self.tile_word(src, 0);
            let bottom = self.tile_word(src, 6);
            self.room_write_bg2(d + xy(12, 9) as u16, top);
            self.room_write_bg2(d + xy(18, 9) as u16, top);
            self.room_write_bg2(d + xy(12, 10) as u16, bottom);
            self.room_write_bg2(d + xy(18, 10) as u16, bottom);
            src += 2;
            d += 1;
        }
        d -= 6;

        src = 0x1c74;
        for _ in 0..6 {
            self.room_write_bg2(d + xy(7, 14) as u16, self.tile_word(src, 0));
            self.room_write_bg2(d + xy(7, 20) as u16, self.tile_word(src, 0));
            self.room_write_bg2(d + xy(8, 14) as u16, self.tile_word(src, 1));
            self.room_write_bg2(d + xy(8, 20) as u16, self.tile_word(src, 1));
            src += 4;
            d += 64;
        }
        d -= 64 * 6;

        src = 0x1c8c;
        for _ in 0..5 {
            for y in 0..5 {
                self.room_write_bg2(d + xy(7, 9 + y) as u16, self.tile_word(src, y));
            }
            src += 10;
            d += 1;
        }
        d -= 5;

        for _ in 0..4 {
            let pos0 = d + xy(14, 28) as u16;
            let pos1 = d + xy(14, 29) as u16;
            let tile0 = self.room_read_bg2(pos0) | 0x2000;
            let tile1 = self.room_read_bg2(pos1) | 0x2000;
            self.room_write_bg2(pos0, tile0);
            self.room_write_bg2(pos1, tile1);
            d += 1;
        }
    }

    /// ROM: `RoomDraw_AgahnimAltar`
    pub(super) fn room_draw_agahnim_altar(&mut self, dsto: u16) {
        let base = 0x2000;
        for y in 0..14 {
            let row = dsto + y * 64;
            let src = 0x1b4a + y as usize * 2;
            let tile0 = self.tile_word(src, 0);
            self.room_write_bg(base, row, tile0);
            self.room_write_bg(base, row + 13, tile0 | 0x4000);

            let tile1 = self.tile_word(src, 14);
            self.room_write_bg(base, row + 1, tile1);
            self.room_write_bg(base, row + 2, tile1);
            self.room_write_bg(base, row + 11, tile1 ^ 0x4000);
            self.room_write_bg(base, row + 12, tile1 ^ 0x4000);

            for x in 3..=6 {
                let tile = self.tile_word(src, (x - 1) * 14);
                self.room_write_bg(base, row + x as u16, tile);
                self.room_write_bg(base, row + (13 - x) as u16, tile ^ 0x4000);
            }
        }
    }

    /// ROM: `RoomDraw_A_Many32x32Blocks`
    pub(super) fn room_draw_a_many32x32_blocks(&mut self, n: i32, src: usize, dst: u16) -> u16 {
        // Cycle ledger: RoomDraw_A_Many32x32Blocks $01:8A44, entered with M=16 X=16
        // by JSR. RoomDraw_VitreousGooDamage also reaches it by JMP, which uses
        // the body below so the cost lands in the jumping frame.
        let _scope = crate::cycle_ledger::routine(0x01_8a44);
        self.room_draw_a_many_32x32_blocks_body(n, src, dst)
    }

    /// The body of RoomDraw_A_Many32x32Blocks from $018A44, unscoped.
    pub(super) fn room_draw_a_many_32x32_blocks_body(
        &mut self,
        mut n: i32,
        src: usize,
        mut dst: u16,
    ) -> u16 {
        // $018A44: STA $0A (the block count).
        crate::cycle_ledger::charge(32);
        loop {
            // $018A46-$018A49: LDA #$0002 / STA $04 (the two 4x2 passes).
            crate::cycle_ledger::charge(56);
            for pass in 0..2 {
                // $018A4B-$018A7B: eight tile stores through the line pointers,
                // Y += $100, DEC $04, BNE (priced not taken).
                crate::cycle_ledger::charge(952);
                if pass == 0 {
                    // BNE $018A4B taken after the first pass.
                    crate::cycle_ledger::charge(6);
                }
                for y in 0..2 {
                    for x in 0..4 {
                        let tile = self.tile_word(src, (y * 4 + x) as usize);
                        self.room_write_current(dst + xy(x as usize, y as usize) as u16, tile);
                    }
                }
                dst += xy(0, 2) as u16;
            }
            // $018A7D-$018A85: Y -= $1F8, DEC $0A, BNE (priced not taken).
            crate::cycle_ledger::charge(136);
            dst = dst
                .wrapping_add(xy(4, 0) as u16)
                .wrapping_sub(xy(0, 4) as u16);
            n -= 1;
            if n == 0 {
                break;
            }
            // BNE $018A46 taken for every block but the last.
            crate::cycle_ledger::charge(6);
        }
        // $018A87-$018A88: CLC / RTS.
        crate::cycle_ledger::charge(56);
        dst
    }

    /// The shared body of RoomDraw_Object_Nx4 ($01:97F0), unscoped. JSR
    /// callers enter through `RoomDraw_Object_Nx4` (scope $0197F0) or
    /// `RoomDraw_4x4` (scope $0197ED); the ROM's `JMP` entries
    /// (RoomDraw_SolidWallDecor3x4, RoomDraw_HorizontalTurtleRockPipe) and
    /// `room_draw_4x4_by_jump` call this directly so the cost lands in the
    /// frame that jumped.
    /// ROM: `RoomData_DrawObject_nx4`
    pub(super) fn room_data_draw_object_nx4(&mut self, src: usize, dsto: u16, columns: u16) {
        // $0197F0: STA $0E (the column count).
        crate::cycle_ledger::charge(32);
        for x in 0..columns {
            // $0197F2-$019810: four tile stores through the line pointers,
            // X += 8, INY twice, DEC $0E, BNE (priced not taken).
            crate::cycle_ledger::charge(572);
            if x + 1 < columns {
                // BNE $0197F2 taken for every column but the last.
                crate::cycle_ledger::charge(6);
            }
        }
        // $019812: RTS.
        crate::cycle_ledger::charge(42);
        self.room_draw_object_nx4_tiles(src, dsto, columns);
    }

    /// The Nx4 tile writes with no cycle charge: the straight inter-room
    /// staircase drawers ($01A634-$01A661) inline their own 4-column loop and
    /// never enter RoomDraw_Object_Nx4, so their translation writes the tiles
    /// here and stays uncharged until those drawers are annotated.
    pub(super) fn room_draw_object_nx4_tiles(&mut self, src: usize, dsto: u16, columns: u16) {
        for x in 0..columns {
            for y in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
    }

    /// RoomDraw_4x4 $01:97ED reached by `JMP` (the stairs drawers, Door_South,
    /// NorthCurtainDoor, the third 4x4 of LightBeamOnFloor and
    /// GanonTriforceFloorDecor) or straight from the `JMP ($0E)` dispatch
    /// (the subtype-2/3 table entries that are RoomDraw_4x4 itself): the
    /// `LDA #$0004` (24) and the Nx4 body charge into the jumping frame, no
    /// scope of their own.
    pub(super) fn room_draw_4x4_by_jump(&mut self, src: usize, dsto: u16) {
        crate::cycle_ledger::charge(24);
        self.room_data_draw_object_nx4(src, dsto, 4);
    }

    /// ROM: `RoomDraw_Object_Nx4`
    pub(super) fn room_draw_object_nx4(&mut self, n: u16, src: usize, dsto: u16) {
        // Cycle ledger: RoomDraw_Object_Nx4 $01:97F0, entered with M=16 X=16.
        let _scope = crate::cycle_ledger::routine(0x01_97f0);
        self.room_data_draw_object_nx4(src, dsto, n);
    }

    /// ROM: `RoomDraw_Object_Nx4_Bg2`
    pub(super) fn room_draw_object_nx4_bg2(&mut self, n: u16, src: usize, dsto: u16) {
        for x in 0..n {
            for y in 0..4 {
                self.room_write_bg2(dsto + x + y * 64, self.tile_word(src, (x * 4 + y) as usize));
            }
        }
    }

    /// ROM: `Object_DrawNx4_BothBgs`
    pub(super) fn object_draw_nx4_both_bgs(&mut self, n: u16, src: usize, dsto: u16) {
        self.room_data_draw_object_nx4_both_bgs(src, dsto, n);
    }

    /// ROM: `RoomData_DrawObject_nx4_both_bgs`
    pub(super) fn room_data_draw_object_nx4_both_bgs(&mut self, src: usize, dsto: u16, columns: u16) {
        for x in 0..columns {
            for y in 0..4 {
                let tile = self.tile_word(src, (x * 4 + y) as usize);
                self.room_write_bg2(dsto + x + y * 64, tile);
                self.room_write_bg1(dsto + x + y * 64, tile);
            }
        }
    }

    /// ROM: `RoomDraw_Rightwards2x2`
    pub(super) fn room_draw_rightwards2x2(&mut self, src: usize, dsto: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01_9895));
        if self.lag_free {
            // $01:9895..98AD: four load/store pairs, four INYs, RTS.
            crate::cycle_ledger::charge(506);
        }
        self.room_draw_2x2_tiles(src, dsto);
    }

    /// ROM: `RoomDraw_Object_2x2_downwards`
    pub(super) fn room_draw_downwards2x2(&mut self, src: usize, dsto: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01_b2af));
        if self.lag_free {
            // $01:B2AF..B2C9: four load/store pairs, TYA/CLC/ADC/TAY,
            // RTS. Its pointer return costs 10 more than $9895's INYs.
            crate::cycle_ledger::charge(516);
        }
        self.room_draw_2x2_tiles(src, dsto);
    }

    fn room_draw_2x2_tiles(&mut self, src: usize, dsto: u16) {
        self.room_write_current(dsto, self.tile_word(src, 0));
        self.room_write_current(dsto + 64, self.tile_word(src, 1));
        self.room_write_current(dsto + 1, self.tile_word(src, 2));
        self.room_write_current(dsto + 65, self.tile_word(src, 3));
    }

    pub(super) fn DrawBigGraySegment(
        &mut self,
        record: ObjectRecord,
        src: usize,
        dsto: u16,
    ) {
        let index = self.game_state.dungeon.object_tracking.misc_object_index() as usize >> 1;
        self.dungeon_object_tracking_mut()
            .set_object_record(index, record);
        let next = self
            .game_state
            .dungeon
            .object_tracking
            .misc_object_index()
            .wrapping_add(2);
        self.dungeon_object_tracking_mut()
            .set_misc_object_index(next);
        let load_ptr = self.game_state.dungeon.room_load.load_ptr_offset();
        self.dungeon_object_tracking_mut()
            .set_object_data_pos(index, load_ptr);
        let plane = if self
            .game_state
            .dungeon
            .room_tilemaps
            .first_line_pointer_row0()
            != 0x4000
        {
            0
        } else {
            0x2000
        };
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(index, dsto.wrapping_mul(2) | plane);
        let ul = self.room_read_current(dsto);
        let ll = self.room_read_current(dsto + 64);
        let ur = self.room_read_current(dsto + 1);
        let lr = self.room_read_current(dsto + 65);
        self.dungeon_room_items_mut()
            .set_replacement_tilemap_quad(index, [ul, ll, ur, lr]);
        self.room_draw_rightwards2x2(src, dsto);
    }

    /// ROM: `DrawObjects_PushableBlock`
    pub(super) fn draw_objects_pushable_block(&mut self, dsto_x2: u16, slot: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01_b4d6));
        if self.lag_free {
            // $01:B4D6..B506: save block/replacement records, tail JMP only.
            // RoomDraw_Rightwards2x2 prices its body and RTS separately.
            crate::cycle_ledger::charge(842);
        }
        let x = self.game_state.dungeon.object_tracking.misc_object_index() as usize >> 1;
        let next = self
            .game_state
            .dungeon
            .object_tracking
            .misc_object_index()
            .wrapping_add(2);
        self.dungeon_object_tracking_mut()
            .set_misc_object_index(next);
        self.dungeon_object_tracking_mut()
            .set_object_record(x, ObjectRecord::IDLE_PUSH_BLOCK);
        self.dungeon_object_tracking_mut()
            .set_object_data_pos(x, slot);
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(x, dsto_x2);
        let dsto = (dsto_x2 >> 1) & 0x1fff;
        let ul = self.room_read_current(dsto);
        let ll = self.room_read_current(dsto + 64);
        let ur = self.room_read_current(dsto + 1);
        let lr = self.room_read_current(dsto + 65);
        self.dungeon_room_items_mut()
            .set_replacement_tilemap_quad(x, [ul, ll, ur, lr]);
        self.room_draw_rightwards2x2(0x0e52, dsto);
    }

    /// ROM: `DrawObjects_LightableTorch`
    pub(super) fn draw_objects_lightable_torch(&mut self, dsto_x2: u16, slot: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01_b509));
        if self.lag_free {
            // $01:B509..B539: record setup; lit/unlit branch; tail JMP 2x2.
            let lit = if dsto_x2 & 0x8000 == 0 { 6 } else {
                104 + if self.game_state.dungeon.torch.lit_torches() < 3 { 62 } else { 6 }
            };
            crate::cycle_ledger::charge(384 + lit + 126);
        }
        let x = self.game_state.dungeon.torch.torch_index() as usize >> 1;
        let next = self.game_state.dungeon.torch.torch_index().wrapping_add(2);
        self.dungeon_torch_mut().set_torch_index(next);
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(x, dsto_x2);
        self.dungeon_object_tracking_mut()
            .set_object_data_pos(x, slot);
        let mut src_img = 0x0ec2;
        let dsto = (dsto_x2 >> 1) & 0x1fff;
        if dsto_x2 & 0x8000 != 0 {
            src_img = 0x0eca;
            if self.game_state.dungeon.torch.lit_torches() < 3 {
                self.dungeon_torch_mut().increment_lit_torches();
            }
        }
        self.room_draw_rightwards2x2(src_img, dsto);
    }

    #[track_caller]
    /// ROM: `Object_Fill_Nx1`
    pub(super) fn object_fill_nx1(&mut self, count: u16, src: usize, dsto: u16) {
        let tile = self.tile_word(src, 0);
        for i in 0..count {
            self.room_write_current(dsto + i, tile);
        }
    }

    /// $01:8E6D/$8E9B/$8EAF JSR $B2CE, M16/X16.
    pub(super) fn room_object_fill_row_width(&mut self, count: u16, src: usize, dsto: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01b2ce));
        crate::cycle_ledger::blocks(self.lag_free).block(0x01b2ce);
        self.room_object_fill_row_body(count, src, dsto);
    }

    /// $01:8E85 JSR $B2D0, M16/X16: count arrives in A, not $B2.
    pub(super) fn room_object_fill_row_a(&mut self, count: u16, src: usize, dsto: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01b2d0));
        self.room_object_fill_row_body(count, src, dsto);
    }

    /// $01:8F03/$8F81 JSR $B2CA, M16/X16: advance X/Y before the shared fill.
    /// Native callers supply the already advanced source/destination.
    pub(super) fn room_object_fill_row_advance(&mut self, count: u16, src: usize, dsto: u16) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01b2ca));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x01b2ca);
        b.block(0x01b2ce);
        self.room_object_fill_row_body(count, src, dsto);
    }

    fn room_object_fill_row_body(&mut self, count: u16, src: usize, dsto: u16) {
        // $01:B2D0..B2E0, including each executed store and the shared RTS.
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x01b2d0);
        let tile = self.tile_word(src, 0);
        for i in 0..count {
            b.block(0x01b2d5);
            self.room_write_current(dsto + i, tile);
            if i + 1 != count { b.taken(0x01b2db); }
        }
        b.block(0x01b2dd);
    }

    /// ROM: `Object_Hole`
    pub(super) fn object_hole(&mut self, src: usize, dsto: u16, width: u8, height: u8) {
        self.dungeon_room_load_mut().set_draw_width_indicator(width);
        self.dungeon_room_load_mut()
            .set_draw_height_indicator(height);
        self.object_size_ato_aplus15(4);
        let width = self.game_state.dungeon.room_load.draw_width_indicator() as u16;
        for y in 0..width {
            self.object_fill_nx1(width, src, dsto + y * 64);
        }

        let edge_src = 0x063c;
        self.room_write_current(dsto, self.tile_word(edge_src, 0));
        self.object_fill_nx1(width - 2, edge_src + 2, dsto + 1);
        self.room_write_current(dsto + width - 1, self.tile_word(edge_src, 2));

        let bottom = dsto + (width - 1) * 64;
        self.room_write_current(bottom, self.tile_word(edge_src, 3));
        self.object_fill_nx1(width - 2, edge_src + 8, bottom + 1);
        self.room_write_current(bottom + width - 1, self.tile_word(edge_src, 5));

        let side_src = 0x0648;
        for y in 1..width - 1 {
            self.room_write_current(dsto + y * 64, self.tile_word(side_src, 0));
            self.room_write_current(dsto + width - 1 + y * 64, self.tile_word(side_src, 1));
        }
    }

    #[track_caller]
    pub(super) fn room_fill_rect(&mut self, dsto: u16, width: u16, height: u16, tile: u16) {
        for y in 0..height {
            for x in 0..width {
                self.room_write_current(dsto + x + y * 64, tile);
            }
        }
    }

    #[track_caller]
    pub(super) fn room_write_current(&mut self, dsto: u16, tile: u16) {
        self.room_write_bg(
            self.game_state
                .dungeon
                .room_tilemaps
                .first_line_pointer_row0() as usize,
            dsto,
            tile,
        );
    }

    pub(super) fn room_read_current(&self, dsto: u16) -> u16 {
        let base = self
            .game_state
            .dungeon
            .room_tilemaps
            .first_line_pointer_row0() as usize;
        self.game_state
            .dungeon
            .room_tilemaps
            .room_tilemap_word(base, dsto)
    }

    /// A room drawer's `$7E2000,X` access past the end of BG1 lands in the
    /// RAM-backed graphics scratch at `$6000` (e.g. `$01:B1A4` raising door
    /// priority near the bottom of the lower layer); the ROM reads and writes
    /// those bytes like any other.
    fn room_tilemap_spill(base: usize, dsto: u16) -> Option<usize> {
        let offset = base + dsto as usize * 2;
        (matches!(base, ROOM_BG1_TILEMAP_BASE | ROOM_BG2_TILEMAP_BASE)
            && offset >= crate::game_state::constants::BG_DECOMP_BUFFER_LOAD_GFX)
            .then_some(offset)
    }

    pub(super) fn room_read_bg1(&self, dsto: u16) -> u16 {
        if let Some(offset) = Self::room_tilemap_spill(ROOM_BG1_TILEMAP_BASE, dsto) {
            return read_word_from_slice(&self.ram, offset);
        }
        self.game_state
            .dungeon
            .room_tilemaps
            .room_tilemap_word(ROOM_BG1_TILEMAP_BASE, dsto)
    }

    pub(super) fn room_read_bg2(&self, dsto: u16) -> u16 {
        if let Some(offset) = Self::room_tilemap_spill(ROOM_BG2_TILEMAP_BASE, dsto) {
            return read_word_from_slice(&self.ram, offset);
        }
        self.game_state
            .dungeon
            .room_tilemaps
            .room_tilemap_word(ROOM_BG2_TILEMAP_BASE, dsto)
    }

    #[track_caller]
    pub(super) fn room_write_bg(&mut self, base: usize, dsto: u16, tile: u16) {
        if let Some(offset) = Self::room_tilemap_spill(base, dsto) {
            GraphicsDecompressionScratch::copy_to_buffer(&mut self.ram, offset, &tile.to_le_bytes());
            return;
        }
        self.dungeon_room_tilemaps_mut()
            .set_room_tilemap_word(base, dsto, tile);
    }

    #[track_caller]
    pub(super) fn room_write_bg1(&mut self, dsto: u16, tile: u16) {
        self.room_write_bg(ROOM_BG1_TILEMAP_BASE, dsto, tile);
    }

    #[track_caller]
    pub(super) fn room_write_bg2(&mut self, dsto: u16, tile: u16) {
        self.room_write_bg(ROOM_BG2_TILEMAP_BASE, dsto, tile);
    }

    pub(in crate::zelda_rtl) fn tile_word(&self, src: usize, index: usize) -> u16 {
        read_word_from_slice(
            self.asset_raw(69)
                .expect("missing predefined dungeon tile asset"),
            src + index * 2,
        )
    }

    /// ROM: `RoomDraw_FloorChunks`
    pub(super) fn room_draw_floor_chunks(&mut self, base: usize, src_offset: usize) {
        // Cycle ledger: RoomDraw_FloorChunks $01:8A1F, entered with M=16 X=16
        // (the JSR from RoomDraw_DrawFloors; its second pass falls through
        // into the body without a JSR and is scoped by the caller).
        let _scope = crate::cycle_ledger::routine(0x01_8a1f);
        self.room_draw_floor_chunks_body(base, src_offset);
    }

    pub(super) fn room_draw_floor_chunks_body(&mut self, base: usize, src_offset: usize) {
        let Some(tile_data) = self.asset_raw(69).map(Vec::from) else {
            return;
        };
        for (quadrant_index, &quadrant) in DUNGEON_QUADRANT_OFFSETS.iter().enumerate() {
            // $018A1F-$018A28: quadrant offset from the table at $9B02, the
            // eight-row count into $0E.
            crate::cycle_ledger::charge(148);
            let mut dst = quadrant;
            for row in 0..8 {
                // $018A2A-$018A36: LDA #$0008, JSR RoomDraw_A_Many32x32Blocks
                // (46, the callee charges itself), Y += $1C0, DEC $0E, BNE
                // (priced not taken).
                crate::cycle_ledger::charge(178);
                if row < 7 {
                    // BNE $018A2A taken for every row but the last.
                    crate::cycle_ledger::charge(6);
                }
                self.room_draw_many_32x32_blocks(base, src_offset, &tile_data, dst);
                dst += xy(0, 4) * 2;
            }
            // $018A38-$018A41: $0C += 2, CMP #$0008, BNE (priced not taken).
            crate::cycle_ledger::charge(180);
            if quadrant_index < 3 {
                // BNE $018A1F taken for every quadrant but the last.
                crate::cycle_ledger::charge(6);
            }
        }
        // $018A43: RTS.
        crate::cycle_ledger::charge(42);
    }

    pub(super) fn room_draw_many_32x32_blocks(
        &mut self,
        base: usize,
        src_offset: usize,
        tile_data: &[u8],
        dst: usize,
    ) {
        // Cycle ledger: this is RoomDraw_A_Many32x32Blocks $01:8A44 with a
        // fixed n = 8 (the floor-chunk row), so its cost is a constant:
        // STA $0A (32) + 8 blocks of [$018A46 (56) + two 4x2 passes ($018A4B,
        // 952 each, BNE taken once: +6) + $018A7D (136)] = 8 * 2,102, + 7
        // taken BNE $018A46 (42) + CLC / RTS (56) = 16,946. Same blocks as the
        // general translation above.
        crate::cycle_ledger::charge_routine(0x01_8a44, 16_946);
        let mut cursor = dst;
        for _ in 0..8 {
            for _ in 0..2 {
                for y in 0..2 {
                    for x in 0..4 {
                        let src = read_word_from_slice(tile_data, src_offset + (y * 4 + x) * 2);
                        self.dungeon_room_tilemaps_mut()
                            .set_room_tilemap_word_by_byte_offset(base, cursor + xy(x, y) * 2, src);
                    }
                }
                cursor += xy(0, 2) * 2;
            }
            cursor = cursor.wrapping_add(xy(4, 0) * 2).wrapping_sub(xy(0, 4) * 2);
        }
    }

    /// ROM: `Dungeon_DrawRoomOverlay`
    pub(super) fn dungeon_draw_room_overlay(&mut self, src: &[u8]) {
        let mut offset = 0usize;
        loop {
            self.dungeon_room_load_mut()
                .set_draw_width_indicator_word(0);
            self.dungeon_room_load_mut()
                .set_draw_height_indicator_word(0);
            let marker = src[offset] as u16 | ((src[offset + 1] as u16) << 8);
            if marker == 0xffff {
                break;
            }
            self.dungeon_draw_overlay_one(&src[offset..]);
            offset += 3;
        }
    }

    fn dungeon_draw_overlay_one(&mut self, src: &[u8]) {
        // $01:F978 JSR $F980, M16/X16: decode one three-byte object,
        // then write its complete 4x4 tile body ($01:F980..FA49).
        // $F9A7 and $F9FA each include their own RTS; no priced child.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01f980));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x01f980);
        let p = (src[0] as u16 >> 2) | ((src[1] as u16 >> 2) << 6);
        let kind = src[2];
        if kind == 0xa4 {
            b.block(0x01f9a7);
            let mid = self.tile_word(0x05aa, 0);
            let top = self.tile_word(0x063c, 1);
            let bottom = self.tile_word(0x0642, 1);
            for x in 0..4u16 {
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile((p + x) as usize, top);
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile((p + x + 64) as usize, mid);
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile((p + x + 128) as usize, mid);
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile((p + x + 192) as usize, bottom);
            }
        } else {
            b.taken(0x01f9a5);
            b.block(0x01f9fa);
            let floor = self
                .game_state
                .dungeon
                .room_parser
                .floor_2_filler_tile_source() as u16 as usize;
            for y in 0..4u16 {
                for x in 0..4u16 {
                    let idx = match (x & 1, y & 1) {
                        (0, 0) => 0,
                        (1, 0) => 1,
                        (0, 1) => 4,
                        _ => 5,
                    };
                    let tile = self.tile_word(floor, idx);
                    self.room_write_bg2(p + x + y * 64, tile);
                }
            }
        }
    }

    /// ROM: `Dungeon_DrawRoomOverlay_Apply`
    pub(super) fn dungeon_draw_room_overlay_apply(&mut self, mut p: usize) {
        // $01:B892 JSR $FE41, M16/X16: four rows of four attribute
        // classifications and stores. Overlay drawing is a separate owner.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x01fe41));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x01fe41);
        for row in 0..4 {
            b.block(0x01fe48);
            for i in 0..4 {
                b.block(0x01fe65);
                let t = self.game_state.dungeon.room_tilemaps.bg2_tile(p + i) & 0x03fe;
                if t == 0x00ee { b.taken(0x01fe6f); }
                else {
                    b.block(0x01fe71);
                    if t == 0x00fe { b.taken(0x01fe74); }
                    else { b.block(0x01fe76); }
                }
                let attr = if t == 0x00ee || t == 0x00fe {
                    NativeTile::GROUND
                } else {
                    NativeTile::PIT
                };
                self.dungeon_bg2_attributes_mut().set_bg2_tile(p + i, attr);
                b.block(0x01fe7b);
                if i != 3 { b.taken(0x01fe7d); }
            }
            b.block(0x01fe7f);
            if row != 3 { b.taken(0x01fea9); }
            p += 64;
        }
        b.block(0x01feab);
    }
}

#[cfg(test)]
mod load_pacing3_tests {
    use super::*;

    fn native() -> ZeldaState {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut n = ZeldaState::new();
        n.set_assets(&std::fs::read(root.join("zelda3_assets.dat")).unwrap()).unwrap();
        n.set_lag_free(true);
        n
    }

    fn source(n: &ZeldaState, pc: u16) -> snes::Snes {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        let mut s = snes::Snes::new();
        s.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        s.ram.copy_from_slice(&n.wram_image());
        s.cpu.e = false; s.cpu.unpack_flags(0);
        s.cpu.k = 1; s.cpu.db = 1; s.cpu.pc = pc; s.cpu.sp = 0x1fd;
        s.ram[0x1fe..0x200].copy_from_slice(&[0xff, 0x7f]);
        s
    }

    #[test]
    fn overlay_dma_matches_rom_markers_orientation_and_address_edges() {
        for address in [0x0000u16, 0x0038, 0x003a, 0x003e, 0x0040, 0x01d8, 0x1fba, 0x303e] {
            let mut n = native();
            let mut s = source(&n, 0xf764);
            s.ram[8..10].copy_from_slice(&address.to_le_bytes());
            s.ram[12..14].copy_from_slice(&12u16.to_le_bytes());
            crate::cycle_ledger::assert_rom_marker_work(0x01f764, s,
                |s| s.cpu.k == 1 && (0xf764..0xf7f1).contains(&s.cpu.pc),
                || { assert_eq!(n.dungeon_prep_overlay_dma_next_prep(6, address), 30); });
        }
    }

    #[test]
    fn watergate_dma_matches_rom_markers_supplied_rows_and_header() {
        for (address, header, rows) in [(0x01d8u16, 0x0881u16, 8usize),
            (0x0040, 0x0881, 4), (0x0038, 0x0880, 4), (0x1040, 0x0880, 1)] {
            let mut n = native();
            let mut s = source(&n, 0xf77c);
            s.cpu.y = 12;
            s.ram[6..8].copy_from_slice(&header.to_le_bytes());
            s.ram[8..10].copy_from_slice(&address.to_le_bytes());
            s.ram[10..12].copy_from_slice(&(rows as u16).to_le_bytes());
            crate::cycle_ledger::assert_rom_marker_work(0x01f77c, s,
                |s| s.cpu.k == 1 && (0xf77c..0xf7f1).contains(&s.cpu.pc),
                || { assert_eq!(n.dungeon_prep_overlay_dma_watergate(6, address, header, rows), 6 + rows * 6); });
        }
    }

    #[test]
    fn overlay_drawing_matches_rom_markers_both_tile_bodies() {
        for encoded in [[0, 0, 0xa4], [0x7c, 0x7c, 0xa4],
            [0x40, 0x20, 0], [0x7c, 0x7c, 0x23]] {
            let mut n = native();
            let mut s = source(&n, 0xf980);
            s.ram[0..2].copy_from_slice(&encoded[..2]);
            // [$B7],Y names the three-byte object; the parent retained its
            // second byte in $01 before JSR $F980.
            s.ram[0xb7..0xba].copy_from_slice(&[0, 0x18, 0x7e]);
            s.ram[0x1800..0x1803].copy_from_slice(&encoded);
            crate::cycle_ledger::assert_rom_marker_work(0x01f980, s,
                |s| s.cpu.k == 1 && (0xf980..0xfa4a).contains(&s.cpu.pc),
                || n.dungeon_draw_overlay_one(&encoded));
        }
    }
}
