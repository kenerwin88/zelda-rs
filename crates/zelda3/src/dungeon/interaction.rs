//! Tile interactions, liftable objects, chests, and pushed blocks.

use super::*;
use crate::game_state::ObjectRecord;
use crate::tile_definition::{DungeonRole, NativeTile, TilePair};
use crate::types::Point16U;

impl ZeldaState {

    /// ROM: `Dungeon_Store2x2`
    pub(super) fn dungeon_store2x2(
        &mut self,
        pos: u16,
        t0: u16,
        t1: u16,
        t2: u16,
        t3: u16,
        attr: NativeTile,
    ) {
        let tiles = [t0, t1, t2, t3];
        let positions = [pos, pos + 64, pos + 1, pos + 65];
        for (&tile_pos, &tile) in positions.iter().zip(tiles.iter()) {
            // C writes `dung_bg2[tile_pos]` flat; dung_bg2 (0x2000) and dung_bg1
            // (0x4000) are contiguous in WRAM, so a `tile_pos >= 0x1000` (the
            // lower-level 0x1000 bit baked into object_tilemap_pos) spills into
            // the BG1 span. `set_room_tilemap_word` routes through the
            // spill-aware `set_tile_at_abs`; `set_bg2_tile` would silently drop
            // the OOB index and leave the BG1 tile unwritten.
            self.dungeon_room_tilemaps_mut().set_room_tilemap_word(
                crate::game_state::constants::DUNG_BG2,
                tile_pos,
                tile,
            );
            self.dungeon_bg2_attributes_mut()
                .set_bg2_tile(tile_pos as usize, attr);
        }

        let dst = self.game_state.display.current_vram_upload_data_address();
        for (i, (&tile_pos, &tile)) in positions.iter().zip(tiles.iter()).enumerate() {
            let base = dst + i * 6;
            let addr = self.dungeon_map_vram_addr(tile_pos);
            self.write_vram_upload_single_tile_stripe_packet(base, addr, tile);
        }
        self.write_vram_upload_tile_stripe_sentinel(dst + 24);
        self.advance_vram_upload_cursor_by(24);
        self.set_bg_vram_load_mode(1);
    }

    /// ROM: `Dungeon_UpdateTileMapWithCommonTile`
    pub(in crate::zelda_rtl) fn dungeon_update_tile_map_with_common_tile(&mut self, x: u16, y: u16, v: u8) {
        if v == 8 {
            self.dungeon_prep_sprite_induced_dma(x.wrapping_add(16), y, v + 2);
        }
        self.dungeon_prep_sprite_induced_dma(x, y, v);
        self.set_bg_vram_load_mode(1);
    }

    /// ROM: `Dungeon_PrepSpriteInducedDma`
    pub(super) fn dungeon_prep_sprite_induced_dma(&mut self, x: u16, y: u16, v: u8) {
        let pos = ((y.wrapping_add(1) & 0x01f8) << 3) | ((x & 0x01f8) >> 3);
        let src = DUNGEON_PREP_SPRITE_INDUCED_DMA_PREP_SPRITE_INDUCED_DMA_SRCS[(v >> 1) as usize];
        let tiles = [
            self.tile_word(src, 0),
            self.tile_word(src, 1),
            self.tile_word(src, 2),
            self.tile_word(src, 3),
        ];
        let attr = self.dungeon_tile_definition(tiles[3] as usize);
        let tile_positions = [pos, pos + 64, pos + 1, pos + 65];

        for &tile_pos in &tile_positions {
            self.dungeon_bg2_attributes_mut()
                .set_bg2_tile(tile_pos as usize, attr);
        }

        for (&tile_pos, &tile) in tile_positions.iter().zip(tiles.iter()) {
            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile(tile_pos as usize, tile);
        }

        let dst = self.game_state.display.current_vram_upload_data_address();
        for (i, (&tile_pos, &tile)) in tile_positions.iter().zip(tiles.iter()).enumerate() {
            let base = dst + i * 6;
            let vram_addr = self.dungeon_map_vram_addr(tile_pos);
            self.write_vram_upload_single_tile_stripe_packet(base, vram_addr, tile);
        }
        self.write_vram_upload_tile_stripe_sentinel(dst + 24);
        self.advance_vram_upload_cursor_by(24);
    }

    /// ROM: `RoomDraw_16x16Single`
    pub(super) fn room_draw_16x16_single(&mut self, index: u8) {
        let index = (index >> 1) as usize;
        let pos = (self
            .game_state
            .dungeon
            .object_tracking
            .object_tilemap_pos(index)
            & 0x3fff)
            >> 1;
        let [ul, ll, ur, lr] = self
            .game_state
            .dungeon
            .room_items
            .replacement_tilemap_quad(index);
        let attr = self.dungeon_tile_definition(lr as usize);
        self.dungeon_store2x2(pos, ul, ll, ur, lr, attr);
    }

    /// ROM: `Dungeon_LiftAndReplaceLiftable`
    pub(in crate::zelda_rtl) fn dungeon_lift_and_replace_liftable(&mut self, pt: &mut Point16U) -> u8 {
        let direction = self.game_state.player.follower_link.facing_index();
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(LIFTABLE_TILE_PROBE_X_OFFSETS[direction] as u16);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(LIFTABLE_TILE_PROBE_Y_OFFSETS[direction] as u16);
        pt.x = x;
        pt.y = y;
        self.scratch_word_mut()
            .set_liftable_tile_probe_position(y, x);

        let x = x & 0x01f8;
        let y = y & 0x01f8;
        let xy = (y << 3)
            | (x >> 3)
            | self
                .game_state
                .player
                .follower_link
                .lower_level_tilemap_offset();
        let tile = self.game_state.dungeon.bg2_attributes.bg2_tile(xy as usize);
        assert!(matches!(
            tile.dungeon_role(),
            DungeonRole::TrackedObject { .. } | DungeonRole::ClosedDoor { .. }
        ));
        let attr = tile.object_slot() as u8;
        let record = self
            .game_state
            .dungeon
            .object_tracking
            .object_record(attr as usize);

        if let Some(kind) = record.liftable_kind() {
            let misc = u16::from(attr) * 2;
            self.dungeon_object_tracking_mut()
                .set_misc_object_index(misc);
            let tilemap = self
                .game_state
                .dungeon
                .object_tracking
                .object_tilemap_pos(attr as usize);
            self.RevealPotItem(xy, tilemap);
            self.room_draw_16x16_single(misc as u8);
            self.manip_block_something(pt);
            LIFTABLE_TILE_REPLACEMENT_ITEM_CODES[kind] as u8
        } else if let Some(segment) = record.big_rock_segment_index() {
            self.thieves_attic_draw_lightened_hole(
                xy,
                (u16::from(attr).wrapping_sub(segment as u16)).wrapping_mul(2),
                pt,
            )
        } else {
            0
        }
    }

    /// ROM: `ThievesAttic_DrawLightenedHole`
    pub(super) fn thieves_attic_draw_lightened_hole(
        &mut self,
        pos6: u16,
        a: u16,
        pt: &mut Point16U,
    ) -> u8 {
        self.dungeon_object_tracking_mut().set_misc_object_index(a);
        let tilemap = self
            .game_state
            .dungeon
            .object_tracking
            .object_tilemap_pos((a >> 1) as usize);
        self.RevealPotItem(pos6, tilemap);
        self.room_draw_16x16_single(a as u8);
        self.room_draw_16x16_single(a.wrapping_add(2) as u8);
        self.room_draw_16x16_single(a.wrapping_add(4) as u8);
        self.room_draw_16x16_single(a.wrapping_add(6) as u8);
        self.manip_block_something(pt);
        0x55
    }

    /// ROM: `HandleItemTileAction_Dungeon`
    pub(in crate::zelda_rtl) fn handle_item_tile_action_dungeon(&mut self, x: u16, y: u16) -> u8 {
        if !self.game_state.player.follower_link.item_in_hand_has(2)
            && (!self
                .game_state
                .enhanced_features
                .has(FEATURE_BREAK_POTS_WITH_SWORD_DUNGEON)
                || self.game_state.player.follower_link.button_b_frames() == 0
                || self.game_state.inventory.items.sword_type() == 1)
        {
            return 0;
        }

        let pos = (y & 0x01f8).wrapping_mul(8).wrapping_add(x)
            + self
                .game_state
                .player
                .follower_link
                .lower_level_tilemap_offset();
        let tile = self
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_tile(pos as usize);
        if let DungeonRole::TrackedObject { slot } = tile.dungeon_role() {
            let record = self.game_state.dungeon.object_tracking.object_record(slot);
            if record.is_hammer_peg() {
                if !self.game_state.player.follower_link.item_in_hand_has(2) {
                    return 0;
                }
                self.dungeon_object_tracking_mut()
                    .set_misc_object_index(slot as u16 * 2);
                self.room_draw_16x16_single(
                    self.game_state.dungeon.object_tracking.misc_object_index() as u8,
                );
                self.set_sound_effect_1(0x11);
            } else if record.liftable_kind().is_some() {
                self.dungeon_object_tracking_mut()
                    .set_misc_object_index(slot as u16 * 2);
                let tilemap = self
                    .game_state
                    .dungeon
                    .object_tracking
                    .object_tilemap_pos(slot);
                self.RevealPotItem(pos, tilemap);
                self.room_draw_16x16_single(
                    self.game_state.dungeon.object_tracking.misc_object_index() as u8,
                );
                let mut pt = Point16U { x: 0, y: 0 };
                self.manip_block_something(&mut pt);
                self.dungeon_secret_scratch_mut().mark_graphics_kind();
                self.sprite_spawn_immediately_smashed_terrain(1, pt.x, pt.y);
                self.ancilla_add_bush_poof(pt.x, pt.y);
            }
        }
        0
    }

    /// ROM: `ManipBlock_Something`
    pub(super) fn manip_block_something(&mut self, pt: &mut Point16U) {
        let index = self.game_state.dungeon.object_tracking.misc_object_index() as usize >> 1;
        let pos = self
            .game_state
            .dungeon
            .object_tracking
            .object_tilemap_pos(index);
        pt.x = (self.game_state.player.follower_link.x() & 0xfe00) | ((pos & 0x007e) << 2);
        pt.y = (self.game_state.player.follower_link.y() & 0xfe00) | ((pos & 0x1f80) >> 4);
    }

    pub(super) fn RevealPotItem(&mut self, pos6: u16, pos4: u16) {
        self.dungeon_secret_scratch_mut().clear_pending_kind();
        let room = self.game_state.world.location.dungeon_room() as usize;
        let secrets = self
            .asset_raw(50)
            .expect("missing dungeon secrets asset")
            .to_vec();
        let mut src = read_le_u16(&secrets, room * 2) as usize;
        let mut index = 0usize;
        loop {
            let test_pos = read_le_u16(&secrets, src);
            if test_pos == 0xffff {
                return;
            }
            assert_eq!(test_pos & 0x8000, 0);
            if test_pos == pos4 {
                break;
            }
            src += 3;
            index += 1;
        }

        let data = secrets[src + 2];
        if data == 0 {
            return;
        }
        if data < 0x80 {
            if data != 8 {
                let mask = 1u16 << index;
                let revealed = self
                    .game_state
                    .dungeon
                    .room_parser
                    .pots_revealed_in_room(room);
                if revealed & mask != 0 {
                    return;
                }
                self.dungeon_room_parser_mut()
                    .mark_pot_revealed_in_room(room, mask);
            }
            self.dungeon_secret_scratch_mut().or_pending_kind(data);
        } else if data != 0x88 {
            let j = self
                .game_state
                .dungeon
                .bg2_attributes
                .bg2_tile(pos6 as usize)
                .object_slot() as u8;
            let mut k = (u16::from(j).wrapping_sub(
                self.game_state
                    .dungeon
                    .object_tracking
                    .object_record(j as usize)
                    .kind_index() as u16,
            )) as usize;
            self.dungeon_object_tracking_mut()
                .set_misc_object_index((2 * k) as u16);
            self.set_sound_effect_2(0x1b);
            let src_words = self.read_predefined_tile_words(0x05ba, 16);
            for chunk in src_words.chunks_exact(4).take(4) {
                self.dungeon_room_items_mut()
                    .set_replacement_tilemap_quad(k, [chunk[0], chunk[1], chunk[2], chunk[3]]);
                k += 1;
            }
        } else {
            let k = self.game_state.dungeon.object_tracking.misc_object_index() as usize >> 1;
            self.dungeon_room_items_mut()
                .set_replacement_tilemap_quad(k, [0x0d0b, 0x0d1b, 0x4d0b, 0x4d1b]);
        }
    }

    /// ROM: `PushBlock_CheckForPit`
    pub(in crate::zelda_rtl) fn push_block_check_for_pit(&mut self, y: u8) {
        let y = (y >> 1) as usize;
        let tilemap = self
            .game_state
            .dungeon
            .object_tracking
            .object_tilemap_pos(y);
        if tilemap & 0x4000 == 0 {
            self.dungeon_environment_mut()
                .toggle_movable_block_was_pushed();
        }

        let p = (tilemap & 0x3fff) >> 1;
        let target = self.game_state.dungeon.bg2_attributes.bg2_tile(p as usize);
        if target == NativeTile::PIT {
            self.set_sound_effect_1(0x20);
            let k = (self
                .game_state
                .dungeon
                .object_tracking
                .object_pos_in_objdata(y)
                >> 2) as usize;
            let room = u16::from(self.game_state.dungeon.header.travel_destination(0));
            self.dungeon_movable_blocks_mut()
                .set_movable_block_record(k, room, tilemap);
            return;
        }

        let i = usize::from(
            self.game_state
                .dungeon
                .object_tracking
                .changeable_object_index(1)
                == y as u8 + 1,
        );
        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(i);

        if target == NativeTile::PRESSURE_PLATE {
            let related = self.game_state.dungeon.environment.trapdoors_down() ^ 1;
            self.dungeon_environment_mut()
                .set_block_trap_related_tile(related);
            self.dungeon_object_tracking_mut()
                .set_object_record(y, ObjectRecord::FALLING);
        } else {
            self.dungeon_object_tracking_mut()
                .set_object_record(y, ObjectRecord::VANISHED);
        }
        self.dungeon_store2x2(p, 0x0922, 0x0932, 0x0923, 0x0933, NativeTile::OPEN_CHEST);
    }

    /// ROM: `PushBlock_Slide`
    pub(super) fn push_block_slide(&mut self, j: u8) {
        if self.game_state.frame.submodule != 0 {
            return;
        }
        let i = usize::from(
            (i32::from(
                self.game_state
                    .dungeon
                    .object_tracking
                    .changeable_object_index(1),
            ) - 1)
                * 2
                == i32::from(j),
        );
        self.pushed_block_mut().reset_animation_timer();
        self.pushed_block_mut().set_animation_mode(0);
        self.push_block_apply_velocity(i as u8);
        let y = self.game_state.player.pushed_block.y(i);
        let x = self.game_state.player.pushed_block.x(i);
        self.push_block_handle_collision(i as u8, x, y);
    }

    /// ROM: `PushBlock_HandleFalling`
    pub(super) fn push_block_handle_falling(&mut self, y: u8) {
        let y = (y >> 1) as usize;
        if !(self.pushed_block_mut().decrement_animation_timer() as i8).is_negative() {
            return;
        }
        self.pushed_block_mut().reset_animation_timer();
        if self.pushed_block_mut().advance_animation_mode() == 4 {
            self.dungeon_object_tracking_mut().settle_object_record(y);
            self.pushed_block_mut().set_animation_mode(0);
            let i = usize::from(
                i32::from(
                    self.game_state
                        .dungeon
                        .object_tracking
                        .changeable_object_index(1),
                ) - 1
                    == y as i32,
            );
            self.dungeon_object_tracking_mut()
                .clear_changeable_object_index(i);
        }
    }

    /// ROM: `PushBlock_ApplyVelocity`
    pub(super) fn push_block_apply_velocity(&mut self, i: u8) {
        let i = i as usize;
        let facing = self.game_state.player.pushed_block.facing_player(i) >> 1;
        let m = PUSH_BLOCK_APPLY_VELOCITY_PUSHED_BLOCK_DIR_MASK[facing as usize];
        self.follower_link_state_mut().set_actual_velocity_xy(0, 0);

        let o;
        if m & 3 != 0 {
            let vel = if m & 2 != 0 { -12i32 } else { 12i32 };
            self.follower_link_state_mut()
                .set_actual_x_velocity(vel as i8 as u8);
            o = self
                .game_state
                .player
                .pushed_block
                .x_fixed24(i)
                .wrapping_add((vel * 16) as u32);
            self.pushed_block_mut().set_x_fixed24(i, o);
        } else {
            let vel = if m & 8 != 0 { -12i32 } else { 12i32 };
            self.follower_link_state_mut()
                .set_actual_y_velocity(vel as i8 as u8);
            o = self
                .game_state
                .player
                .pushed_block
                .y_fixed24(i)
                .wrapping_add((vel * 16) as u32);
            self.pushed_block_mut().set_y_fixed24(i, o);
        }

        if ((o >> 8) as u8 & 0x0f) == self.game_state.player.pushed_block.target_low(i) {
            let j = self
                .game_state
                .dungeon
                .object_tracking
                .changeable_object_index(i)
                .wrapping_sub(1) as usize;
            self.dungeon_object_tracking_mut().advance_object_record(j);
            self.follower_link_state_mut()
                .clear_direction_lock_bits(0x04);
            self.follower_link_state_mut().and_defense_flags(!0x04);
        }

        let x = self.game_state.player.pushed_block.x(i);
        let y = self.game_state.player.pushed_block.y(i);
        for j in (0..16usize).rev() {
            let sprite = self.sprite_slot_view(j);
            if sprite.state() >= 9 {
                let sx = sprite.x();
                let sy = sprite.y();
                if x.wrapping_sub(sx).wrapping_add(0x10) < 0x20
                    && y.wrapping_sub(sy).wrapping_add(0x10) < 0x20
                {
                    let k = facing as usize;
                    let mut sprite = self.sprite_slot_view_mut(j);
                    sprite.set_f(8);
                    sprite.set_x_recoil(
                        PUSH_BLOCK_APPLY_VELOCITY_PUSH_BLOCK_X_RECOIL_BY_DIRECTION[k],
                    );
                    sprite.set_y_recoil(
                        PUSH_BLOCK_APPLY_VELOCITY_PUSH_BLOCK_Y_RECOIL_BY_DIRECTION[k],
                    );
                }
            }
        }
    }

    /// ROM: `PushBlock_HandleCollision`
    pub(super) fn push_block_handle_collision(&mut self, i: u8, x: u16, y: u16) {
        let i = i as usize;
        // C writes ONLY the safe-return HIGH bytes here (LINK_*_COORD_SAFE_RETURN_HI = coord >> 8);
        // it leaves the LOW bytes (0x3e/0x3c) untouched. Using store_safe_return_position would
        // also stamp the low bytes, diverging RAW scratch (0x3e off by Link's sub-tile Y).
        self.follower_link_state_mut()
            .cache_safe_return_high_from_current();

        let mut dir = 3i32;
        let mut m = self.game_state.player.follower_link.direction() & 0x0f;
        while m & 1 == 0 {
            m >>= 1;
            dir -= 1;
            if dir < 0 {
                return;
            }
        }
        let dir = dir as usize;
        let l = if dir < 2 {
            self.game_state.player.follower_link.x()
        } else {
            self.game_state.player.follower_link.y()
        };
        let o = if dir < 2 { x } else { y };
        let r0 = l.wrapping_add(PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_A[dir]);
        let r2 = l.wrapping_add(PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_B[dir]);
        let r4 = o.wrapping_add(PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_C[dir]);
        let r6 = o.wrapping_add(PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_D[dir]);
        let horizontal = dir >= 2;
        let current_coord = if horizontal {
            self.game_state.player.follower_link.x()
        } else {
            self.game_state.player.follower_link.y()
        };
        let r8 = current_coord.wrapping_add(PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_E[dir]);
        let r10 = (if dir < 2 { y } else { x })
            .wrapping_add(PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_F[dir]);

        self.follower_link_state_mut().and_defense_flags(!4);
        if (r0 >= r4 && r0 < r6) || (r2 >= r4 && r2 < r6) {
            if self.game_state.player.follower_link.facing()
                == self.game_state.player.pushed_block.facing_player(i)
            {
                let defense_flags = if self
                    .game_state
                    .dungeon
                    .object_tracking
                    .changeable_object_index(i)
                    != 0
                {
                    4
                } else {
                    1
                };
                self.follower_link_state_mut()
                    .or_defense_flags(defense_flags);
            }
            let diff = r8.wrapping_sub(r10);
            if (dir & 1 != 0 && r8 >= r10 && diff < 8) || (dir & 1 == 0 && diff >= 0xfff8) {
                let coord = current_coord.wrapping_sub(diff);
                if horizontal {
                    self.follower_link_state_mut().set_x(coord);
                } else {
                    self.follower_link_state_mut().set_y(coord);
                }
                self.follower_link_state_mut()
                    .subtract_axis_velocity_delta(horizontal, diff as u8);
            }
        }
        self.handle_indoor_camera_and_doors();
    }

    /// ROM: `Dungeon_MapVramAddr`
    pub(in crate::zelda_rtl) fn dungeon_map_vram_addr(&self, pos: u16) -> u16 {
        self.dungeon_map_vram_addr_no_swap(pos).swap_bytes()
    }

    /// ROM: `Dungeon_MapVramAddrNoSwap`
    pub(super) fn dungeon_map_vram_addr_no_swap(&self, pos: u16) -> u16 {
        let pos = pos.wrapping_mul(2);
        ((pos & 0x40) << 4) | ((pos & 0x303f) >> 1) | ((pos & 0x0f80) >> 2)
    }

    /// ROM: `Dungeon_GetTeleMsg`
    pub(in crate::zelda_rtl) fn dungeon_get_tele_msg(&self, room: usize) -> u16 {
        self.asset_u16(9, room)
    }

    /// ROM: `Dungeon_IsPitThatHurtsPlayer`
    pub(in crate::zelda_rtl) fn dungeon_is_pit_that_hurts_player(&self) -> bool {
        let room = self.game_state.world.location.dungeon_room();
        let Some(data) = self.asset_raw(10) else {
            return false;
        };
        data.chunks_exact(2)
            .any(|entry| read_word_from_slice(entry, 0) == room)
    }

    /// ROM: `Dungeon_SaveAndLoadAllPalettes`
    pub(in crate::zelda_rtl) fn dungeon_save_and_load_all_palettes(&mut self, main_tile_theme: u8, sprite_gfx: u8) {
        self.sprite_system_mut().set_graphics_index(sprite_gfx);
        self.world_palette_theme_mut()
            .set_main_tile_theme_index(main_tile_theme);
        self.world_palette_theme_mut()
            .set_aux_tile_theme_index(main_tile_theme);
        self.initialize_tilesets();
        self.select_overworld_aux_palette_offset();
        self.increment_cgram_update_flag();
        self.palette_bg_and_fixed_color_black();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sprite_environment_dungeon();
        self.palette_load_hud();
        self.palette_load_dungeon_set();
    }
    /// ROM: `Dungeon_CheckForAndIDLiftableTile`
    pub(in crate::zelda_rtl) fn dungeon_check_for_and_id_liftable_tile(&self) -> u16 {
        let facing = self.game_state.player.follower_link.facing_index();
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(LIFTABLE_TILE_PROBE_X_OFFSETS[facing] as u16)
            & 0x01f8;
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(LIFTABLE_TILE_PROBE_Y_OFFSETS[facing] as u16)
            & 0x01f8;
        let offset = ((y << 3) | (x >> 3)) as usize
            + usize::from(
                self.game_state
                    .player
                    .follower_link
                    .lower_level_tilemap_offset(),
            );

        let DungeonRole::TrackedObject { slot } = self
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_tile(offset)
            .dungeon_role()
        else {
            return 0xffff;
        };

        let record = self.game_state.dungeon.object_tracking.object_record(slot);
        if record.is_idle() {
            return 0xffff;
        }
        if record.big_rock_segment_index().is_some() {
            return 0x55;
        }
        LIFTABLE_TILE_REPLACEMENT_ITEM_CODES[record.kind_index()]
    }

    pub(in crate::zelda_rtl) fn OpenChestForItem(&mut self, tile: u8, chest_position: &mut u16) -> u8 {
        if let Some((item, position)) = self.OpenChestForItemResult(tile) {
            *chest_position = position;
            item
        } else {
            0xff
        }
    }

    pub(in crate::zelda_rtl) fn OpenChestForItemResult(&mut self, tile: u8) -> Option<(u8, u16)> {
        if tile == 0x63 {
            return self.OpenMiniGameChestResult();
        }
        let chest_idx_org = tile.wrapping_sub(0x58) as usize;
        let loc = self
            .game_state
            .dungeon
            .room_items
            .chest_location(chest_idx_org);
        let palace_mask = upper_bitmask(
            (self
                .game_state
                .inventory
                .save_progress
                .palace_index_x2_word()
                >> 1) as usize,
        );
        if loc >= 0x8000 {
            if self
                .game_state
                .inventory
                .player_resources
                .lacks_big_key_mask(palace_mask)
            {
                self.dialogue_message_index_mut().set_value(0x007a);
                self.main_show_text_message();
                return None;
            }
            self.dungeon_savegame_state_mut()
                .or_savegame_state_bits(DUNGEON_CHEST_OPEN_MASKS[chest_idx_org]);
            self.set_sound_effect_1(0x29);
            self.set_sound_effect_2(0x15);
            let pos = (loc & 0x7fff) >> 1;
            let src = self.read_predefined_tile_words(
                self.game_state
                    .dungeon
                    .room_parser
                    .floor_2_filler_tile_source() as u16,
                4,
            );
            let chest_position = self.apply_opened_chest_tiles(pos, loc, &src);
            return Some((0xff, chest_position));
        }

        let chest_data = self
            .asset_raw(8)
            .expect("missing dungeon room chests asset")
            .to_vec();
        let mut chest_idx = chest_idx_org as isize;
        let room = self.game_state.world.location.dungeon_room();
        for entry in chest_data.chunks_exact(3) {
            let chest_room = read_word_from_slice(entry, 0);
            if (chest_room & 0x7fff) == room {
                chest_idx -= 1;
                if chest_idx < 0 {
                    let item = entry[2];
                    if chest_room & 0x8000 != 0 {
                        if self
                            .game_state
                            .inventory
                            .player_resources
                            .lacks_big_key_mask(palace_mask)
                        {
                            self.dialogue_message_index_mut().set_value(0x007a);
                            self.main_show_text_message();
                            return None;
                        }
                        self.dungeon_savegame_state_mut()
                            .or_savegame_state_bits(DUNGEON_CHEST_OPEN_MASKS[chest_idx_org]);
                        let chest_position = self.OpenBigChestResult(loc);
                        return Some((item, chest_position));
                    }
                    self.dungeon_savegame_state_mut()
                        .or_savegame_state_bits(DUNGEON_CHEST_OPEN_MASKS[chest_idx_org]);
                    let src = self.read_predefined_tile_words(0x14a4, 4);
                    let chest_position = self.apply_opened_chest_tiles(loc >> 1, loc, &src);
                    return Some((item, chest_position));
                }
            }
        }
        None
    }

    pub(super) fn OpenMiniGameChestResult(&mut self) -> Option<(u8, u16)> {
        if self.game_state.minigame.credits() == 0 {
            self.dialogue_message_index_mut().set_value(0x0163);
            self.main_show_text_message();
            return None;
        }
        if self.game_state.minigame.credits() == 0xff {
            self.dialogue_message_index_mut().set_value(0x0162);
            self.main_show_text_message();
            return None;
        }
        self.minigame_state_mut().decrement_credits();

        let mut pos = (self.game_state.player.follower_link.y().wrapping_sub(4) & 0x01f8) * 8;
        pos |= (self.game_state.player.follower_link.x().wrapping_add(7) & 0x01f8) >> 3;
        let chest = TilePair::repeated(NativeTile::MINIGAME_CHEST);
        if self
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_tiles(pos as usize)
            != chest
        {
            pos = pos.wrapping_sub(1);
            if self
                .game_state
                .dungeon
                .bg2_attributes
                .bg2_tiles(pos as usize)
                != chest
            {
                pos = pos.wrapping_add(2);
            }
        }

        self.dungeon_bg2_attributes_mut()
            .set_bg2_tiles(pos as usize, [NativeTile::SOLID_WALL; 2]);
        self.dungeon_bg2_attributes_mut()
            .set_bg2_tiles(pos as usize + 64, [NativeTile::SOLID_WALL; 2]);

        let src = self.read_predefined_tile_words(0x14a4, 4);
        let pos_wrong = pos as usize + 128;
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile(pos_wrong, src[0]);
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile(pos_wrong + 64, src[1]);
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile(pos_wrong + 1, src[2]);
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile(pos_wrong + 65, src[3]);

        let dst = self.game_state.display.current_vram_upload_data_address();
        let positions = [pos, pos + 64, pos + 1, pos + 65];
        for (i, &tile_pos) in positions.iter().enumerate() {
            let base = dst + i * 6;
            let vram_addr = self.dungeon_map_vram_addr(tile_pos);
            self.write_vram_upload_absolute_word(base, vram_addr);
            self.write_vram_upload_absolute_word(base + 2, 0x0100);
            self.write_vram_upload_absolute_word(base + 4, src[i]);
        }
        self.write_vram_upload_absolute_word(dst + 24, 0xffff);
        let next_upload = self.game_state.display.vram_upload_cursor.wrapping_add(24);
        self.set_vram_upload_cursor(next_upload);

        let old_choice = self
            .game_state
            .dungeon
            .scratch_word
            .minigame_previous_chest_choice();
        let mut choice = self.get_random_number();
        let room = self.game_state.world.location.dungeon_room_index();
        let item = if room == 0 {
            choice &= 0x0f;
            RUPEE_CHEST_MINIGAME_PRIZES[choice as usize]
        } else if room == 0x18 {
            choice = 0x10 + (choice & 0x0f);
            RUPEE_CHEST_MINIGAME_PRIZES[(0x10 + (choice & 0x0f)) as usize]
        } else {
            choice &= 7;
            if choice >= 2 && choice == old_choice {
                choice = choice.wrapping_add(1) & 7;
            }
            if choice == 7 {
                let save_bits = self.game_state.dungeon.savegame_state.savegame_state_bits();
                if save_bits & 0x4000 != 0 {
                    choice = 0;
                } else {
                    self.dungeon_savegame_state_mut()
                        .set_savegame_state_bits(save_bits | 0x4000);
                }
            }
            DUNGEON_MINIGAME_CHEST_PRIZES[choice as usize]
        };
        self.scratch_word_mut()
            .set_minigame_previous_chest_choice(choice);
        self.set_bg_vram_load_mode(1);
        self.set_sound_effect_2(14);
        Some((item, pos * 2))
    }

    pub(super) fn OpenBigChestResult(&mut self, loc: u16) -> u16 {
        let pos = loc >> 1;
        let src = self.read_predefined_tile_words(0x14c4, 12);
        for i in 0..4 {
            let dst = pos as usize + i;
            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile(dst, src[i * 3]);
            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile(dst + 64, src[i * 3 + 1]);
            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile(dst + 128, src[i * 3 + 2]);
        }
        self.dungeon_prep_overlay_dma_next_prep(0, loc);
        for &tile_pos in &[pos, pos + 2, pos + 64, pos + 66, pos + 128, pos + 130] {
            self.dungeon_bg2_attributes_mut()
                .set_bg2_tiles(tile_pos as usize, [NativeTile::OPEN_CHEST; 2]);
        }
        self.dungeon_flag_room_data_quadrants();
        self.set_sound_effect_2(14);
        self.request_nmi_copy_packets();
        self.dungeon_room_effects_mut().mark_trap_trigger_latched();
        loc + 2
    }

    pub(in crate::zelda_rtl) fn dungeon_push_block_handler(&mut self) {
        // Cycle ledger: Dungeon_PushBlock_Handler $01:D81B (JSL target, m8
        // x8 entry). The loop and its exit are charged by the body below; a
        // resumed lane (`dungeon_push_block_handler_until`) charges into
        // its own open scope.
        let _scope = crate::cycle_ledger::routine(0x01_d81b);
        self.dungeon_push_block_handler_until(u16::MAX);
    }

    /// Dungeon_PushBlock_Handler's loop while the misc object word offset is
    /// below `limit`; the source loop is `limit == u16::MAX`.
    pub(in crate::zelda_rtl) fn dungeon_push_block_handler_until(&mut self, limit: u16) {
        while self.game_state.dungeon.object_tracking.misc_object_index()
            != self.game_state.dungeon.torch.torches_start_index()
            && self.game_state.dungeon.object_tracking.misc_object_index() < limit
        {
            // $01:D81B REP #$30, LDY $042C, CPY $0478, BNE (118; taken +6
            // into PushBlock_Main $01:D7C8 LDA $0500,y : BEQ, 62).
            crate::cycle_ledger::charge(118 + 6 + 62);
            let obj = self.game_state.dungeon.object_tracking.misc_object_index();
            let k = usize::from(obj >> 1);
            let record = self.game_state.dungeon.object_tracking.object_record(k);
            if record == ObjectRecord::PUSHED {
                // $01:D7CD CMP #$0001 : BNE (40), $01:D7D2-D7E3 JSR
                // RoomDraw_16x16Single, the tilemap-position advance, BRA
                // (262), $01:D80F LDX $042C : INC $0500,x (108).
                crate::cycle_ledger::charge(40 + 262 + 108);
                self.room_draw_16x16_single(obj as u8);
                let dir = self.game_state.player.pushed_block.push_direction_index();
                let pos = self
                    .game_state
                    .dungeon
                    .object_tracking
                    .object_tilemap_pos(k)
                    .wrapping_add_signed(DUNGEON_PUSH_BLOCK_HANDLER_PUSH_BLOCK_MOVE_DISTANCES[dir]);
                self.dungeon_object_tracking_mut()
                    .set_object_tilemap_pos(k, pos);
                self.dungeon_object_tracking_mut()
                    .set_object_record(k, ObjectRecord::SLIDING);
            } else if record == ObjectRecord::SLIDING {
                // $01:D7CD BNE taken (+6), $01:D7E5 CMP #$0002 : BNE (40),
                // $01:D7EA-D7FB SEP, JSL PushBlock_Slide, REP, the record
                // reload, CMP #$0003, BNE (232); an arrival runs $01:D7FD JSR
                // PushBlock_CheckForPit : BRA (68) and $01:D80F (108), else
                // the BNE is taken (+6).
                crate::cycle_ledger::charge(40 + 6 + 40 + 232);
                self.push_block_slide(obj as u8);
                self.finish_dungeon_push_slide_after_motion();
            } else if record == ObjectRecord::FALLING {
                // Two taken BNEs (+6 each), $01:D802 CMP #$0004 : BNE (40),
                // $01:D807 SEP, JSL PushBlock_HandleFalling, BRA (106).
                crate::cycle_ledger::charge(40 + 6 + 40 + 6 + 40 + 106);
                self.push_block_handle_falling(obj as u8);
            } else if record == ObjectRecord::IDLE_PUSH_BLOCK {
                // The $01:D7CB BEQ taken (+6) straight to $01:D815.
                crate::cycle_ledger::charge(6);
            } else {
                // Every other record falls through the three compares with
                // their BNEs taken.
                crate::cycle_ledger::charge(40 + 6 + 40 + 6 + 40 + 6);
            }
            self.advance_dungeon_push_block_index();
        }
        if self.game_state.dungeon.object_tracking.misc_object_index()
            == self.game_state.dungeon.torch.torches_start_index()
        {
            // The loop head with the BNE not taken (118) and $01:D825 SEP
            // #$30 : RTL (66).
            crate::cycle_ledger::charge(118 + 66);
        }
    }

    /// Resume after the block's movement/collision has already retired.
    /// The room child and remaining loop are still owned by the semantic lane.
    pub(in crate::zelda_rtl) fn finish_native_dungeon_push_handler(
        &mut self,
        tail: crate::cycle_models::dungeon_push_caller_return::PushCallerTail,
    ) {
        use crate::cycle_models::dungeon_push_caller_return::PushCallerTail;
        match tail {
            PushCallerTail::Returned => {}
            PushCallerTail::Handler => self.dungeon_push_block_handler(),
            PushCallerTail::RoomTransition(target) => {
                match target {
                    0x02b62e => self.handle_edge_transition_movement_east_right_by8(),
                    0x02b6cd => self.dung_start_inter_room_trans_left_plus(),
                    0x02b76e => self.handle_edge_transition_movement_south_down_by16(),
                    0x02b81c => self.dungeon_start_inter_room_trans_up(),
                    _ => panic!("unknown push-block room transition {target:06x}"),
                }
                // Keep the existing post-call guard price: REP/LDY/LDA/CMP/BNE.
                // SEP and JSL (84 of the old 232) have already retired natively.
                crate::cycle_ledger::charge(232 - 84);
                self.finish_dungeon_push_slide_after_motion();
                self.advance_dungeon_push_block_index();
                self.dungeon_push_block_handler_until(u16::MAX);
            }
        }
    }

    fn finish_dungeon_push_slide_after_motion(&mut self) {
        let obj = self.game_state.dungeon.object_tracking.misc_object_index();
        let k = usize::from(obj >> 1);
        if self.game_state.dungeon.object_tracking.object_record(k) == ObjectRecord::ARRIVED {
            crate::cycle_ledger::charge(68 + 108);
            // A landing on a plate or a hole rewrites the record
            // before this advance, so a vanished block wraps to idle.
            self.push_block_check_for_pit(obj as u8);
            self.dungeon_object_tracking_mut().advance_object_record(k);
        } else {
            crate::cycle_ledger::charge(6);
        }
    }

    fn advance_dungeon_push_block_index(&mut self) {
        // $01:D815 INC $042C : INC $042C (124), back to the loop head.
        crate::cycle_ledger::charge(124);
        let next = self
            .game_state
            .dungeon
            .object_tracking
            .misc_object_index()
            .wrapping_add(2);
        self.dungeon_object_tracking_mut()
            .set_misc_object_index(next);
    }
}
