//! Room headers, resumable floor/object loading, and tilemap uploads.

use super::*;
use super::objects::{object_subtype1_param, object_subtype2_param, object_subtype3_param};
use crate::game_state::constants::{ALT_SPRITE_STATE, ALT_SPRITE_TYPE, ANCILLA_TYPE, DIALOGUE_MESSAGE_INDEX, GARNISH_TYPE, MAIN_MODULE, MESSAGING_MODULE, OVERLORD_TYPE, SUBMODULE, SUBSUBMODULE, TEXT_INCREMENTAL_STATE, TEXT_RENDER_STATE};

impl ZeldaState {
    pub(in crate::zelda_rtl) fn begin_dungeon_object_draw_current_room(&mut self) -> Option<DungeonObjectDrawWork> {
        let room = self.game_state.world.location.dungeon_room();
        let room_layout = self.dungeon_room_layout(usize::from(room))?;
        let first_room_stream_offset = self.game_state.dungeon.room_load.load_ptr_offset();
        let layout = usize::from(*room_layout.get(usize::from(first_room_stream_offset))?);
        self.dungeon_room_tilemaps_mut()
            .copy_line_pointer_bytes(&DUNGEON_DRAW_OBJECT_OFFSETS_BG1);
        self.dungeon_room_parser_mut()
            .set_room_layout_and_starting_quadrant(layout as u16);
        self.dungeon_room_load_mut().set_load_ptr_offset(0);
        Some(DungeonObjectDrawWork {
            room,
            first_room_stream_offset,
            phase: 0,
            stream: DungeonObjectStreamProgress::default(),
            completed_objects: 0,
        })
    }

    /// Advance to the next complete native object, crossing empty streams and
    /// layer transitions without creating artificial object completions.
    pub(in crate::zelda_rtl) fn advance_dungeon_object_draw_one(&mut self, work: &mut DungeonObjectDrawWork) -> bool {
        assert_eq!(self.game_state.world.location.dungeon_room(), work.room);
        if let Some(mut draw) = self.active_dungeon_object_4x2.take() {
            let total = draw.count * 8;
            self.advance_dungeon_4x2_draw_words(&mut draw, total);
            self.dungeon_room_load_mut().set_draw_width_indicator_word(0);
            work.completed_objects += 1;
            return true;
        }
        while !work.finished() {
            let room_layout = self.dungeon_room_layout(usize::from(work.room)).map(Vec::from)
                .expect("room layout disappeared during object draw");
            let layout_byte = room_layout[usize::from(work.first_room_stream_offset)];
            let stream = if work.phase == 0 {
                self.default_room_layout(usize::from(layout_byte >> 2)).map(Vec::from)
            } else {
                Some(room_layout)
            };
            if let Some(stream) = stream {
                if self.room_data_draw_next_object_from(&stream, &mut work.stream) {
                    // A tail-call boundary may return control to the scheduler
                    // with this object's body still suspended.
                    if self.active_dungeon_object_4x2.is_none() {
                        work.completed_objects += 1;
                    }
                    return true;
                }
            }
            work.phase += 1;
            work.stream = DungeonObjectStreamProgress::default();
            match work.phase {
                1 => self.dungeon_room_load_mut().set_load_ptr_offset(
                    work.first_room_stream_offset.saturating_add(1)),
                2 | 3 => {
                    let next = self.game_state.dungeon.room_load.load_ptr_offset().wrapping_add(2);
                    self.dungeon_room_load_mut().set_load_ptr_offset(next);
                    if work.phase == 2 {
                        self.dungeon_room_tilemaps_mut().copy_bg2_draw_line_offsets();
                    } else {
                        self.dungeon_room_tilemaps_mut().copy_bg1_draw_line_offsets();
                    }
                }
                4 => self.dungeon_room_load_mut().set_load_ptr_offset(0x0120),
                _ => unreachable!(),
            }
        }
        false
    }

    pub(in crate::zelda_rtl) fn advance_dungeon_object_4x2_prefix(
        &mut self,
        work: &mut DungeonObjectDrawWork,
        ordinal: u16,
        boundary: Dungeon4x2DrawBoundary,
    ) {
        assert_eq!(ordinal, work.completed_objects + 1);
        if let Some(mut draw) = self.active_dungeon_object_4x2.take() {
            self.advance_dungeon_4x2_draw_boundary(&mut draw, boundary);
            self.active_dungeon_object_4x2 = Some(draw);
        } else {
            assert!(self.pending_dungeon_object_4x2_boundary.is_none());
            self.pending_dungeon_object_4x2_boundary = Some(boundary);
            assert!(self.advance_dungeon_object_draw_one(work));
            assert!(self.pending_dungeon_object_4x2_boundary.is_none(),
                "native next object did not enter the source 4x2 tail call");
            assert!(self.active_dungeon_object_4x2.is_some());
            assert_eq!(work.completed_objects + 1, ordinal,
                "partial object was incorrectly marked complete");
        }
    }

    pub fn parity_probe_direct_entrance(&mut self, entrance_index: u16) -> u16 {
        self.set_which_entrance_word(entrance_index);
        self.dungeon_load_entrance();
        self.game_state.dungeon.room_tracking.room_index2_word()
    }

    pub fn parity_probe_dungeon_room(&mut self, room: u16) -> u16 {
        self.set_indoor_flag(1);
        self.set_dungeon_room(room);
        self.dungeon_room_tracking_mut().set_room_index2_word(room);
        self.game_state.dungeon.room_tracking.room_index2_word()
    }

    pub fn developer_prepare_synthetic_room(&mut self, room: u16) {
        self.set_indoor_flag(1);
        self.set_dungeon_room(room);
        self.dungeon_room_tracking_mut().set_room_index2_word(room);
        self.ram[MAIN_MODULE] = 0x07;
        self.ram[SUBMODULE] = 0x00;
        self.ram[SUBSUBMODULE] = 0x00;

        self.ram[SPRITE_STATE..SPRITE_STATE + 16].fill(0);
        self.ram[SPRITE_TYPE..SPRITE_TYPE + 16].fill(0);
        self.ram[ANCILLA_TYPE..ANCILLA_TYPE + 10].fill(0);
        self.ram[OVERLORD_TYPE..OVERLORD_TYPE + 8].fill(0);
        self.ram[GARNISH_TYPE..GARNISH_TYPE + 0x1e].fill(0);
        self.ram[ALT_SPRITE_STATE..ALT_SPRITE_STATE + 16].fill(0);
        self.ram[ALT_SPRITE_TYPE..ALT_SPRITE_TYPE + 16].fill(0);
        self.ram[ALT_SPRITES_FLAG] = 0;

        self.ram[TEXT_RENDER_STATE] = 0;
        self.ram[TEXT_INCREMENTAL_STATE] = 0;
        self.ram[MESSAGING_MODULE] = 0;
        self.ram[DIALOGUE_MESSAGE_INDEX] = 0;
        self.ram[DIALOGUE_MESSAGE_INDEX + 1] = 0;
        self.ram[SHARED_MESSAGE_TIMER] = 0;
        self.ram[SHARED_MESSAGE_TIMER + 1] = 0;
        self.ram[MESSAGE_OR_SPRITE_STATE_CACHE] = 0;

        self.sync_native_game_state_from_ram();
    }

    /// Spike probe (dungeon-tileset plan, Task 1): load + draw a real dungeon
    /// entrance room from a fresh state AND load its blockset CHR into VRAM,
    /// returning the blockset theme (`main_tile_theme_index`, the BG-CHR key).
    ///
    /// Sequence (mirrors the attract-mode dungeon scene + the overworld
    /// `parity_probe_overworld_screen_and_build_map` probe):
    ///   1. `Dungeon_LoadAndDrawEntranceRoom(entrance)` — resolves the entrance to
    ///      a room, runs `Dungeon_LoadHeader` (sets `palette_theme`), and draws the
    ///      64x64 BG1/BG2 tilemap into `game_state.dungeon.room_tilemaps`.
    ///   2. `InitializeTilesets()` — decompresses the main blockset CHR (keyed off
    ///      `palette_theme.main_tile_theme_index()`) into `ppu.vram` at word base
    ///      0x2000 (the first `load_background_graphics(0x2000, main_tile_set[0], ..)`).
    ///
    /// After this call, read BG1 tilemap words via
    /// `parity_probe_dungeon_bg1_map8_entry`, and decode CHR from `ppu.vram` at
    /// word base 0x2000.
    ///
    /// NOTE: the argument is an ENTRANCE index (consumed by
    /// `Dungeon_LoadAndDrawEntranceRoom`), not a raw room-header index.
    pub fn parity_probe_dungeon_load_and_draw(&mut self, entrance: u16) -> u16 {
        self.dungeon_load_and_draw_entrance_room(entrance as u8);
        self.InitializeTilesets();
        u16::from(self.game_state.world.palette_theme.main_tile_theme_index())
    }

    /// Spike probe: read a BG1 tilemap word (tile8 entry) from the loaded room.
    /// `tile_index` is a word index into the 64x64 (0x1000-word) BG1 tilemap;
    /// mirrors `parity_probe_overworld_bg2_map8_entry` for the dungeon BG1 layer.
    pub fn parity_probe_dungeon_bg1_map8_entry(&self, tile_index: usize) -> u16 {
        self.game_state.dungeon.room_tilemaps.bg1_tile(tile_index)
    }

    /// Spike probe: read a BG2 (dungeon floor) tilemap word from the loaded room.
    /// `tile_index` is a word index into the 64x64 (0x1000-word) BG2 tilemap;
    /// mirrors `parity_probe_dungeon_bg1_map8_entry` for the dungeon BG2 layer.
    pub fn parity_probe_dungeon_bg2_map8_entry(&self, tile_index: usize) -> u16 {
        self.game_state.dungeon.room_tilemaps.bg2_tile(tile_index)
    }

    pub fn developer_queue_music_track(&mut self, track: u8) {
        self.set_music_control(track);
        self.set_last_music_control(0);
    }

    /// ROM: `Dungeon_LoadAndDrawEntranceRoom`
    pub(in crate::zelda_rtl) fn dungeon_load_and_draw_entrance_room(&mut self, room: u8) {
        self.set_which_entrance(room);
        self.dungeon_load_entrance();
        self.dungeon_torch_mut().clear_lit_torches();
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
        self.dungeon_load_and_draw_room();
        self.dungeon_reset_torch_background_and_player();
    }

    /// ROM: `Dungeon_LoadAndDrawRoom`
    pub(in crate::zelda_rtl) fn dungeon_load_and_draw_room(&mut self) {
        let hdma = self.game_state.display.hdma_enable_mask;
        self.clear_hdma_enable_mask();
        self.dungeon_load_room();
        self.dungeon_load_and_draw_room_after_room_load(hdma);
    }

    fn dungeon_load_and_draw_room_after_early_header(&mut self, hdma: u8) {
        self.dungeon_load_room_after_header();
        self.dungeon_load_and_draw_room_after_room_load(hdma);
    }

    fn dungeon_load_and_draw_room_after_early_floor(&mut self, hdma: u8) {
        self.dungeon_load_room_after_floors();
        self.dungeon_load_and_draw_room_after_room_load(hdma);
    }

    fn dungeon_load_and_draw_room_after_early_objects(&mut self, hdma: u8) {
        self.dungeon_load_room_after_objects();
        self.dungeon_load_and_draw_room_after_room_load(hdma);
    }

    pub(in crate::zelda_rtl) fn begin_dungeon_room_upload_after_early_objects(&mut self) -> DungeonRoomUploadWork {
        self.dungeon_load_room_after_objects();
        self.clear_screen_transition();
        self.set_overworld_map_state(0);
        self.begin_dungeon_room_upload()
    }

    pub(in crate::zelda_rtl) fn begin_dungeon_room_upload_after_early_object_prefix(&mut self) -> DungeonRoomUploadWork {
        self.dungeon_load_room_after_object_prefix();
        self.clear_screen_transition();
        self.set_overworld_map_state(0);
        self.begin_dungeon_room_upload()
    }

    fn dungeon_load_and_draw_room_after_early_upload(
        &mut self,
        work: DungeonRoomUploadWork,
        hdma: u8,
    ) {
        self.finish_dungeon_room_upload(work);
        self.dungeon_load_and_draw_room_after_upload(hdma);
    }

    fn dungeon_load_and_draw_room_after_room_load(&mut self, hdma: u8) {
        self.clear_screen_transition();
        self.set_overworld_map_state(0);
        self.dungeon_upload_room_quadrants();
        self.dungeon_load_and_draw_room_after_upload(hdma);
    }

    fn dungeon_load_and_draw_room_after_upload(&mut self, hdma: u8) {
        self.set_hdma_enable_mask(hdma);
        self.clear_pending_nmi_subroutine();
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
    }

    /// ROM: `Dungeon_LoadCustomTileAttr`
    pub(in crate::zelda_rtl) fn dungeon_load_custom_tile_attr(&mut self) {
        let offset = self.asset_u16(
            51,
            self.game_state.world.palette_theme.aux_tile_theme_index() as usize,
        ) as usize;
        let attrs = self.asset_raw(52).expect("missing dungeon tile attr asset");
        let custom_attrs = attrs[offset..offset + 0x80].to_vec();
        self.dungeon_room_parser_mut()
            .copy_custom_tile_attrs(&custom_attrs);
    }

    pub(crate) fn native_dungeon_visited_flag_byte(address: u32) -> Option<u8> {
        address.checked_sub(0x02b5cc)
            .and_then(|i| DUNGEON_QUADRANT_VISITING_FLAGS.get(i as usize))
            .map(|&value| value as u8)
    }

    /// Physical immutable selectors and bytes for the custom attribute owner.
    pub(crate) fn native_dungeon_custom_attribute_data(&self) -> Option<std::collections::BTreeMap<u32, u8>> {
        let mut bytes = std::collections::BTreeMap::new();
        for (base, asset) in [(0x0e9000, 51), (0x0e902a, 52)] {
            for (i, &value) in self.asset_raw(asset)?.iter().enumerate() {
                if let Some(previous) = bytes.insert(base + i as u32, value) {
                    assert_eq!(previous, value, "overlapping custom attribute data");
                }
            }
        }
        Some(bytes)
    }

    pub(crate) fn native_dungeon_attribute_data(&self) -> Option<std::collections::BTreeMap<u32, u8>> {
        let mut bytes = self.native_dungeon_custom_attribute_data()?;
        for i in 0..16 {
            for (b, value) in upper_bitmask(i).to_le_bytes().into_iter().enumerate() {
                bytes.insert(0x0098c0 + (i * 2 + b) as u32, value);
            }
        }
        for (i, pair) in DUNGEON_LOAD_SINGLE_DOOR_ATTRIBUTE_TILE_ATTRS_BY_DOOR.iter().enumerate() {
            for b in 0..2 {
                bytes.insert(0x009a52 + (i * 2 + b) as u32, pair.0[b].cartridge_attribute());
            }
        }
        Some(bytes)
    }

    pub(in crate::zelda_rtl) fn SaveDungeonKeys(&mut self) {
        let mut idx = self.game_state.inventory.save_progress.palace_index_x2();
        if idx == 0xff {
            return;
        }
        if idx == 2 {
            idx = 0;
        }
        let keys = self.game_state.inventory.player_resources.keys();
        self.dungeon_key_slots_mut().set_keys_earned(idx, keys);
    }

    /// ROM: `Dungeon_LoadRoom`
    pub(super) fn dungeon_load_room(&mut self) {
        self.dungeon_load_header();
        self.dungeon_load_room_after_header();
    }

    fn dungeon_load_room_after_header(&mut self) {
        self.dungeon_load_room_before_floors();
        self.room_draw_draw_floors_current_room();
        self.dungeon_load_room_after_floors();
    }

    /// Stores from DungeonRoom itself, excluding its retained child routines.
    pub(crate) fn native_dungeon_room_parent_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let offset = address as usize;
        match offset {
            0x41c => { assert_eq!(width, 2); self.bg1_move_calc_mut().set_buffer(value); }
            0x68a => { assert_eq!(width, 2); self.water_hdma_window_mut().set_window_y_radius_alt(value); }
            0xb7 | 0xb8 | 0xba | 0xc0..=0xde | 0xfc | 0x310 | 0x312 | 0x3f4 |
            0x40e | 0x41a | 0x420 | 0x422 | 0x424 | 0x428 | 0x42c | 0x42e |
            0x430 | 0x432 | 0x436 | 0x438 | 0x43a | 0x43c | 0x43e |
            0x440 | 0x442 | 0x444 | 0x446 | 0x448 | 0x44e | 0x450 | 0x452 |
            0x454 | 0x456 | 0x45c | 0x460 | 0x478 | 0x47e | 0x480 | 0x482 |
            0x484 | 0x496 | 0x498 | 0x49a | 0x49c | 0x49e | 0x4a2 | 0x4a4 |
            0x4a6 | 0x4a8 | 0x4ae | 0x4b0 | 0x4f0..=0x55e | 0x1980..=0x19e8 => {
                assert_eq!(width, if offset == 0x3f4 { 1 } else { 2 });
                for byte in 0..usize::from(width) {
                    let at = offset + byte;
                    assert!(self.game_state.dungeon.write_room_parent_byte(at, (value >> (byte * 8)) as u8),
                        "unowned dungeon room parent byte {at:04x}");
                }
            }
            _ => return false,
        }
        self.ram[offset..offset + usize::from(width)]
            .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        true
    }

    /// The retained object parser owns its dimensions and stream cursor.
    /// Byte-sized dimensions preserve the native high byte left by the clear.
    pub(crate) fn native_dungeon_object_parser_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        match address {
            0xb2 | 0xb4 => assert!(matches!(width, 1 | 2)),
            0xba => assert_eq!(width, 2),
            _ => return false,
        }
        for byte in 0..usize::from(width) {
            self.game_state.dungeon.write_object_parser_byte(address as usize + byte, (value >> (byte * 8)) as u8);
        }
        self.ram[address as usize..address as usize + usize::from(width)]
            .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        true
    }

    /// Shared object drawers publish their actual stores, including floor-block
    /// children and the word counter spanning a dungeon and player field.
    pub(crate) fn native_dungeon_object_tile_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        if matches!(address, 0xb2 | 0xb4) {
            assert_eq!(width, 2);
            return self.native_dungeon_object_parser_store(address, value, width);
        }
        if self.native_dungeon_floor_store(address, value, width) { return true; }
        let offset = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        // A drawer's indirect row pointer can cross the tilemap boundary.
        // The adjacent graphics scratch is RAM-backed; preserve those actual
        // stores without wrapping them into either native tilemap layer.
        if (crate::game_state::constants::BG_DECOMP_BUFFER_LOAD_GFX
            ..crate::game_state::constants::GRAPHICS_DECOMP_BUFFER_END).contains(&offset) {
            assert_eq!(width, 2);
            assert_eq!(offset & 1, 0);
            GraphicsDecompressionScratch::copy_to_buffer(
                &mut self.ram, offset, &value.to_le_bytes());
            return true;
        }
        if !matches!(address, 0x3f4 | 0x47c | 0x4b0) { return false; }
        assert_eq!(width, 2);
        self.game_state.dungeon.write_object_tile_word(address as usize, value);
        if address == 0x3f4 {
            self.game_state.player.follower_link.write_temp_bunny_timer_low((value >> 8) as u8);
        }
        self.ram[address as usize..address as usize + 2].copy_from_slice(&value.to_le_bytes());
        true
    }

    /// Visitation first changes the live quadrant byte, then the saved room
    /// word. Publish each store without refreshing the rest of the save state.
    pub(crate) fn native_dungeon_visited_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let offset = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        if offset == 0x408 {
            assert_eq!(width, 1);
            assert!(self.game_state.dungeon.write_header_byte(offset, value as u8));
        } else if (0xf000..0xf000 + crate::game_state::DUNGEON_ROOM_COUNT * 2).contains(&offset) {
            assert_eq!(width, 2);
            assert_eq!(offset & 1, 0);
            self.game_state.inventory.save_progress.set_dungeon_info_word((offset - 0xf000) / 2, value);
        } else {
            return false;
        }
        self.ram[offset..offset + usize::from(width)].copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        true
    }

    /// Attribute construction retains its cursors and native collision tiles
    /// through basic, object, door, and crystal-peg child returns.
    pub(crate) fn native_dungeon_attribute_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let offset = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        if !matches!(offset, 0xb2 | 0xb4 | 0x200 | 0x42e | 0x44a | 0x6e0..=0x6ea |
            0xfe00..=0xffff | 0x12000..=0x13fff) { return false; }
        assert!(matches!(width, 1 | 2));
        if offset == 0x200 {
            if width == 1 {
                self.game_state.world.overworld.map_ui.set_map_state(value as u8);
            } else {
                self.game_state.world.overworld.map_ui.set_map_state_word(value);
            }
        } else {
            for byte in 0..usize::from(width) {
                assert!(self.game_state.dungeon.write_attribute_byte(offset + byte, (value >> (byte * 8)) as u8),
                    "unowned attribute byte {:05x}", offset + byte);
            }
        }
        self.ram[offset..offset + usize::from(width)].copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        true
    }

    /// Publish the load-and-draw caller and its preparation/DMA children. The
    /// upload buffer is RAM-backed and owns its cursor/door-scratch aliases for
    /// this phase; do not project those inactive scalar views over its tiles.
    pub(crate) fn native_dungeon_room_upload_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        if (0x1000..=0x17fe).contains(&address) {
            assert_eq!(width, 2);
            assert_eq!(address & 1, 0);
            self.write_vram_upload_tilemap_word(address as usize - 0x1000, value);
            return true;
        }
        if !matches!(address, 0x17 | 0x9b | 0xb0 | 0x116 | 0x200 | 0x418 | 0x45c | 0x710) {
            return false;
        }
        assert_eq!(width, 1);
        let byte = value as u8;
        match address {
            0x17 => self.game_state.display.pending_nmi_subroutine = byte,
            0x9b => self.game_state.display.hdma_enable_mask = byte,
            0xb0 => self.game_state.frame.set_subsubmodule(byte),
            0x116 => self.game_state.display.set_nmi_load_target_page(byte),
            0x200 => self.game_state.world.overworld.map_ui.set_map_state(byte),
            0x418 => self.game_state.world.overworld.transition.set_screen_transition(byte),
            0x45c => { assert!(self.game_state.dungeon.write_room_parent_byte(address as usize, byte)); }
            0x710 => self.game_state.display.core_update_disable_flag = byte,
            _ => unreachable!(),
        }
        self.ram[address as usize] = byte;
        true
    }

    /// The shared door-state child owns all record and open-state stores until
    /// its carry result returns to the directional drawer.
    pub(crate) fn native_dungeon_door_state_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        if !matches!(address, 0x400 | 0x436 | 0x460 | 0x68c | 0x1980..=0x19de) {
            return false;
        }
        assert_eq!(width, 2);
        assert_eq!(address & 1, 0);
        self.game_state.dungeon.write_door_state_word(address as usize, value);
        self.ram[address as usize..address as usize + 2].copy_from_slice(&value.to_le_bytes());
        true
    }

    /// Saved objects retain their counters, records and background tiles across
    /// nested drawing calls. Publish each word through its native component.
    pub(crate) fn native_dungeon_saved_object_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        if self.native_dungeon_object_tile_store(address, value, width) { return true; }
        if self.game_state.player.pushed_block.write_bank_word(address as usize, value) {
            assert_eq!(width, 2);
            self.ram[address as usize..address as usize + 2].copy_from_slice(&value.to_le_bytes());
            return true;
        }
        if !matches!(address, 0xae | 0x42c | 0x42e | 0x432 | 0x45a | 0x496 | 0x498 |
            0x500..=0x5de | 0x6a0..=0x6ea) { return false; }
        assert_eq!(width, 2);
        assert_eq!(address & 1, 0);
        self.game_state.dungeon.write_saved_object_word(address as usize, value);
        self.ram[address as usize..address as usize + 2].copy_from_slice(&value.to_le_bytes());
        true
    }

    /// Floor drawing retires one pointer, selector or tile store at a time.
    /// Do not project the other layer or the live line-pointer table with it.
    pub(crate) fn native_dungeon_floor_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let offset = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        if !matches!(offset, 0xba | 0xbf..=0xde | 0x46a | 0x490 | 0x2000..=0x5ffe) {
            return false;
        }
        assert_eq!(width, 2);
        if offset >= 0x2000 { assert_eq!(offset & 1, 0); }
        for byte in 0..2 {
            assert!(self.game_state.dungeon.write_floor_byte(offset + byte, (value >> (byte * 8)) as u8));
        }
        self.ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        true
    }

    pub(super) fn dungeon_load_room_before_floors(&mut self) {
        self.dungeon_load_room_reset_floor_velocity();
        // C clears SOMARIA_BLOCK_BG_CHECK_FLAG (0x3f4) here at room load (dungeon.c). It is
        // incremented per somaria-block tile during room draw, so without this reset the count
        // accumulates across rooms (stuck high) instead of recounting. The increment is owned by
        // DungeonEnvironmentState, so reset it on the same native.
        self.dungeon_environment_mut()
            .clear_somaria_block_bg_check_flag();
        self.dungeon_environment_mut()
            .clear_somaria_block_switch_counter();
        self.dungeon_room_load_mut()
            .copy_header_collision_2_to_mirror();
        let primary_header_tag = self.game_state.dungeon.header.primary_header_tag();
        self.dungeon_room_load_mut()
            .set_header_collision_2_mirror_high(primary_header_tag);
        self.bg1_move_calc_mut().set_buffer(0x30);
        self.bg1_move_calc_mut().set_x_subpixel(0xff);
        for &offset in &[
            0x41a, 0x420, 0x422, 0x424, 0x436, 0x452, 0x453, 0x454, 0x456, 0x44e, 0x450, 0x0fc,
            0x45c, 0x438, 0x43a, 0x43c, 0x43e, 0x440, 0x442, 0x4ae, 0x444, 0x446, 0x448, 0x49a,
            0x49c, 0x49e, 0x47e, 0x480, 0x482, 0x484, 0x4a2, 0x4a4, 0x4a6, 0x4a8, 0x430, 0x432,
            0x42c, 0x42e, 0x478, 0x496, 0x498, 0x4b0, 0x460,
        ] {
            self.dungeon_room_parser_mut()
                .clear_room_parser_words(&[offset]);
        }
        // $01:877D `STZ $068A` sits in the same clear block; the water window's
        // alternate radius belongs to the display state, not the room parser.
        self.water_hdma_window_mut().set_window_y_radius_alt(0);
        // The clear loop zeroes the blast-wall flag/progress RAM bytes (0x452 FLAG_X,
        // 0x453 FLAG_Y, 0x454 CRUSH_WALL_PROGRESS, 0x456 CRUSH_WALL_DOOR_INDEX_X2) but
        // clear_room_parser_words only touches RAM + the room parser, leaving the
        // DungeonRoomEffectsState native (blast_wall_x/y_open, crush_wall_progress,
        // door_index) stale from a just-opened blast wall — it would re-stamp FLAG_Y=1
        // back over the cleared RAM. Reset the native too (same stale-native-field hazard
        // as misc_object_index / cur_door_idx below).
        self.dungeon_room_effects_mut().clear_blast_wall_state();
        self.dungeon_room_doors_mut().clear_invisible_door_marker();
        // dung_unk2 (0xfc, the exploding-wall quadrant override) is in the same
        // RAM clear loop; DungeonDoorSetupState owns it and would otherwise
        // re-project the previous room's 0x200 over the cleared bytes, locking
        // the camera's vertical quadrant to full size (route host 619103).
        self.dungeon_room_doors_mut().clear_reset_xy_check_flags();
        // dung_index_of_torches(_start) are zeroed by the room-load RAM loop. Keep
        // DungeonTorchState aligned until the current room's misc-object index is installed.
        self.dungeon_torch_mut().clear_torch_indices();
        self.dungeon_torch_mut().clear_timers();
        self.dungeon_object_tracking_mut()
            .clear_replacement_tile_states();
        self.dungeon_object_tracking_mut()
            .clear_object_data_positions();
        // dung_misc_objs_index (0x42c) is owned by DungeonObjectTrackingState, but
        // the clear_room_parser_words loop above only zeroes RAM + reloads the room
        // parser — it leaves this native field stale, so RoomDraw_SinglePot would
        // keep counting pots from the previous room. Clear it here (C: dungeon.c
        // dung_misc_objs_index = 0 at room load).
        self.dungeon_object_tracking_mut().clear_misc_object_index();
        // dung_num_chests_x2 (0x496) / dung_num_bigkey_locks_x2 (0x498) — same
        // stale-native-counter hazard; the room's chest/big-key attribute writes
        // would otherwise miscount and skip object-attribute slots.
        self.dungeon_room_items_mut().clear_item_counts();
        // dung_cur_door_idx (0x460) — same stale-native-field hazard as
        // misc_object_index: the clear_room_parser_words loop only zeroes RAM, so
        // DungeonDoorState.current_door_index would keep last room's value and the
        // door draw would write door tables to the wrong slots (C: dung_cur_door_idx
        // = 0 at room load).
        self.dungeon_doors_mut().set_current_door_index(0);
        // dung_num_stairs_* (stair-list counters, e.g. dung_num_stairs_wet 0x49e) are
        // cleared via the clear_room_parser_words RAM loop above, but DungeonStairLists-
        // State's native counters would otherwise stay stale and re-project a prior
        // room's stair count (mis-placing stair objects in the new room).
        self.dungeon_stair_lists_mut().clear_all_counts();
        // dung_cur_quadrant_upload (0x45c) is in the clear_room_parser_words list above,
        // but that only zeroes RAM; DungeonRoomLoadState.quadrant_upload_index would stay
        // at its prior value (0x10 = "upload complete") and re-project it, so the new
        // room's quadrant VRAM upload would never run (C: clears the byte at room load).
        self.dungeon_room_load_mut().clear_quadrant_upload_index();
        // 0x41a/0x422/0x424 (the moving floor's move flags and offsets, the low
        // byte of the latter doubling as the water transition counter) are zeroed
        // by the clear_room_parser_words loop above; clear the native fields too so
        // a native read before the next import sees the cleared room (f473650).
        self.dungeon_moving_floor_mut().clear_floor_move_flags();
        self.dungeon_moving_floor_mut().clear_floor_offsets();
        for i in 0..16 {
            self.dungeon_object_tracking_mut()
                .set_object_tilemap_pos(i, 0);
        }
        self.dungeon_doors_mut().clear_door_tilemap_addresses();
        self.dungeon_doors_mut().clear_door_tables();
        self.dungeon_room_doors_mut()
            .clear_exit_door_count_and_flags();
        self.dungeon_room_load_mut().set_load_ptr_offset(0);
    }

    fn dungeon_load_room_after_floors(&mut self) {
        self.dungeon_room_tilemaps_mut()
            .copy_line_pointer_bytes(&DUNGEON_DRAW_OBJECT_OFFSETS_BG1);
        self.room_draw_draw_all_objects_current_room();
        self.dungeon_load_room_after_objects();
    }

    pub(super) fn dungeon_load_room_after_objects(&mut self) {
        self.dungeon_load_room_post_object_prefix();
        self.dungeon_load_room_after_object_prefix();
    }

    /// The source has finished its push-block loop and published both torch
    /// range indices before entering the room's torch stream.
    pub(in crate::zelda_rtl) fn dungeon_load_room_post_object_prefix(&mut self) {
        self.dungeon_torch_mut().resync_from_ram();
        let room = self.game_state.world.location.dungeon_room();
        for offset in (0..0x018c).step_by(4) {
            if self
                .game_state
                .dungeon
                .movable_blocks
                .movable_block_room_for_offset(offset)
                == room
            {
                let tilemap = self
                    .game_state
                    .dungeon
                    .movable_blocks
                    .movable_block_tilemap_for_offset(offset);
                self.draw_objects_pushable_block(tilemap, offset as u16);
            }
        }

        let misc_objs = self.game_state.dungeon.object_tracking.misc_object_index();
        self.dungeon_torch_mut()
            .set_torch_index_range_start(misc_objs);
        self.dungeon_torch_mut().set_torch_index(misc_objs);
    }

    pub(super) fn dungeon_load_room_after_object_prefix(&mut self) {
        let room = self.game_state.world.location.dungeon_room();
        let mut i = 0usize;
        loop {
            if self
                .game_state
                .dungeon
                .torch
                .torch_data_word_at_byte_offset(i)
                == room
            {
                i += 2;
                loop {
                    let t = self
                        .game_state
                        .dungeon
                        .torch
                        .torch_data_word_at_byte_offset(i);
                    i += 2;
                    self.draw_objects_lightable_torch(t, (i - 2) as u16);
                    if self
                        .game_state
                        .dungeon
                        .torch
                        .torch_data_word_at_byte_offset(i)
                        == 0xffff
                    {
                        break;
                    }
                }
                break;
            }
            i += 2;
            loop {
                let t = self
                    .game_state
                    .dungeon
                    .torch
                    .torch_data_word_at_byte_offset(i);
                i += 2;
                if t == 0xffff {
                    break;
                }
            }
            if i == 0x0120 {
                break;
            }
        }
        self.dungeon_room_load_mut().set_load_ptr_offset(0x0120);
        if room == 0x51 {
            self.room_prioritize_throne_room_door_edge();
        }
    }

    pub(super) fn dungeon_load_room_reset_floor_velocity(&mut self) {
        self.dungeon_moving_floor_mut().set_floor_y_velocity(0);
        self.dungeon_moving_floor_mut().set_floor_x_velocity(0);
    }

    /// The retained header CPU owns these stores; its stack and direct-page
    /// work bytes continue through the shared native bus.
    pub(crate) fn native_dungeon_header_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let offset = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        match offset {
            0xaa2 => { assert_eq!(width, 1); self.world_palette_theme_mut().set_aux_tile_theme_index(value as u8); }
            0xaa3 => { assert_eq!(width, 1); self.sprite_system_mut().set_graphics_index(value as u8); }
            0xaac => { assert_eq!(width, 1); self.set_sp0l(value as u8); }
            0xaad => { assert_eq!(width, 1); self.set_sp5l(value as u8); }
            0xaae => { assert_eq!(width, 1); self.set_sp6l(value as u8); }
            0xab6 => { assert_eq!(width, 1); self.set_palette_main_indoors(value as u8); }
            0xad..=0xaf | 0xb7..=0xb8 | 0x110 | 0x400 | 0x402 | 0x408 |
            0x414 | 0x468 | 0x46c | 0x4ba | 0x62c | 0x62e | 0x63c..=0x642 |
            0x646 | 0x68c | 0x1100 | 0x1110..=0x1120 | 0x19a0..=0x19be |
            0xc000..=0xc006 | 0xc208 => {
                assert!(matches!(width, 1 | 2));
                for byte in 0..usize::from(width) {
                    let at = offset + byte;
                    assert!(self.game_state.dungeon.write_header_byte(at, (value >> (byte * 8)) as u8),
                        "unowned dungeon header byte {at:04x}");
                }
            }
            _ => return false,
        }
        self.ram[offset..offset + usize::from(width)]
            .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        true
    }

    /// Immutable packed headers and door records from native assets at their
    /// physical addresses. Shared records must agree byte for byte.
    /// Mutable door flags and scratch are never included in this data image.
    pub(crate) fn native_dungeon_header_data(&self) -> Option<std::collections::BTreeMap<u32, u8>> {
        use crate::cycle_models::dungeon_data::DOOR_POINTERS;
        let mut bytes = std::collections::BTreeMap::new();
        let mut insert = |address: u32, data: &[u8]| -> Option<()> {
            for (i, &value) in data.iter().enumerate() {
                if let Some(old) = bytes.insert(address + i as u32, value) {
                    if old != value { return None; }
                }
            }
            Some(())
        };
        let headers = self.assets.as_ref()?.asset_by_name("kDungeonHeaderEncoding")?;
        if headers.len() != 0xefe { return None; }
        insert(0x04f502, &headers[..0xafe])?;
        // Palette selectors are bytes, including FF in unused packed headers.
        // Bind their entire immutable read window, not only the named palettes.
        insert(0x0ed460, &headers[0xafe..])?;
        for room in 0..320 {
            let door = DOOR_POINTERS[room];
            insert(0x1f83c0 + room as u32 * 3, &door.to_le_bytes()[..3])?;
            let data = self.GetRoomDoorInfo(room)?;
            let end = data.chunks_exact(2).take(9).position(|v| v == [255, 255])? * 2 + 2;
            insert(door, &data[..end])?;
        }
        for (i, value) in DUNGEON_LOAD_HEADER_ADJUSTMENTS.iter().enumerate() {
            insert(0x01b560 + i as u32 * 2, &value.to_le_bytes())?;
        }
        for (i, value) in DUNG_PAL_INFOS.iter().enumerate() {
            insert(0x0ed460 + i as u32 * 4, value)?;
        }
        for (i, value) in DUNGEON_CHECK_ADJACENT_ROOMS_FOR_OPEN_DOORS_LOOKUP_TABLE.iter().enumerate() {
            insert(0x009aa2 + i as u32 * 2, &value.to_le_bytes())?;
        }
        for (i, value) in DUNGEON_CHECK_ADJACENT_ROOMS_FOR_OPEN_DOORS_LOOKUP_TABLE2.iter().enumerate() {
            insert(0x009ad2 + i as u32 * 2, &value.to_le_bytes())?;
        }
        for i in 0..16 {
            insert(0x0098c0 + i as u32 * 2, &upper_bitmask(i).to_le_bytes())?;
        }
        Some(bytes)
    }

    /// ROM: `Dungeon_LoadHeader`
    pub(super) fn dungeon_load_header(&mut self) {
        self.dungeon_environment_mut()
            .clear_water_puzzle_state_changed();
        self.dungeon_environment_mut()
            .clear_somaria_block_switch_counter();
        self.dungeon_environment_mut()
            .clear_movable_block_was_pushed();

        let submodule = self.game_state.frame.submodule;
        let bg_h = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg_v = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        let direction = self.game_state.player.follower_link.direction() & 0x0f;
        let (load_h, load_v) = if submodule == 0 {
            (bg_h & !0x01ff, bg_v & !0x01ff)
        } else if submodule == 21 || (6..18).contains(&submodule) {
            (
                bg_h.wrapping_add(0x20) & !0x01ff,
                bg_v.wrapping_add(0x20) & !0x01ff,
            )
        } else if (direction >> 1) < 2 {
            (
                bg_h.wrapping_add(
                    DUNGEON_LOAD_HEADER_ADJUSTMENTS[(direction >> 1) as usize] as u16,
                ) & !0x01ff,
                bg_v.wrapping_add(0x20) & !0x01ff,
            )
        } else {
            (
                bg_h.wrapping_add(0x20) & !0x01ff,
                bg_v.wrapping_add(
                    DUNGEON_LOAD_HEADER_ADJUSTMENTS[(direction >> 3) as usize] as u16,
                ) & !0x01ff,
            )
        };
        self.dungeon_room_load_mut()
            .set_loading_bg_offsets(load_h, load_v);

        let room = self.game_state.world.location.dungeon_room() as usize;
        let header = self
            .GetRoomHeaderPtr(room)
            .expect("dungeon room must have a header")
            .to_vec();

        let bg2_properties = self.game_state.dungeon.room_load.bg2_properties();
        self.dungeon_room_load_mut()
            .set_bg2_properties_backup(bg2_properties);
        self.dungeon_room_load_mut()
            .set_bg2_properties(header[0] >> 5);
        self.dungeon_room_load_mut()
            .set_header_collision((header[0] >> 2) & 7);
        self.dungeon_torch_mut().copy_lights_out_request();
        self.dungeon_torch_mut()
            .set_lights_out_request(header[0] & 1);
        let pal = DUNG_PAL_INFOS[header[1] as usize];
        self.set_palette_main_indoors(pal[0]);
        self.set_sp0l(pal[1]);
        self.set_sp5l(pal[2]);
        self.set_sp6l(pal[3]);
        self.world_palette_theme_mut()
            .set_aux_tile_theme_index(header[2]);
        self.sprite_system_mut()
            .set_graphics_index(header[3].wrapping_add(0x40));
        self.dungeon_room_load_mut()
            .set_header_collision_2(header[4]);
        self.dungeon_header_mut().set_header_tag(0, header[5]);
        self.dungeon_header_mut().set_header_tag(1, header[6]);
        self.dungeon_header_mut()
            .set_hole_teleporter_planes(header[7], header[8]);
        self.dungeon_header_mut()
            .copy_travel_destinations_from_header(&header);
        self.dungeon_environment_mut().set_trapdoors_down(1);
        self.dungeon_room_load_mut().clear_overlay_to_load();
        self.dungeon_room_runtime_mut()
            .set_room_index_x3((room as u16).wrapping_mul(3));

        let saved = self.saved_room_flags(room);
        self.dungeon_doors_mut().set_opened_doors(saved & 0xf000);
        self.dungeon_doors_mut()
            .set_opened_doors_including_adjacent((saved & 0xf000) | 0x0f00);
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits((saved & 0x0ff0) << 4);
        self.dungeon_room_load_mut()
            .set_quadrants_visited(saved & 0x000f);

        self.load_current_room_door_tilemap_addresses(room);
        if (room.wrapping_sub(1) & 0x0f) != 0x0f {
            self.dungeon_check_adjacent_rooms_for_open_doors(18, room.wrapping_sub(1));
        }
        if (room.wrapping_add(1) & 0x0f) != 0 {
            self.dungeon_check_adjacent_rooms_for_open_doors(12, room.wrapping_add(1));
        }
        if room >= 16 {
            self.dungeon_check_adjacent_rooms_for_open_doors(6, room - 16);
        }
        if room + 16 < 0x140 {
            self.dungeon_check_adjacent_rooms_for_open_doors(0, room + 16);
        }
    }

    fn load_current_room_door_tilemap_addresses(&mut self, room: usize) {
        let doors = self
            .GetRoomDoorInfo(room)
            .expect("dungeon room must have door info")
            .to_vec();
        self.dungeon_doors_mut()
            .load_room_door_tilemap_addresses_from_info(&doors);
    }

    /// ROM: `Dungeon_CheckAdjacentRoomsForOpenDoors`
    fn dungeon_check_adjacent_rooms_for_open_doors(&mut self, idx: usize, room: usize) {
        self.load_adjacent_room_doors(room);
        for i in 0..8 {
            let mut a = self.game_state.dungeon.door_setup.adjacent_door(i);
            if a == 0xffff {
                break;
            }
            a &= 0x00ff;
            let mut j = idx;
            while j < idx + 6 {
                if a == DUNGEON_CHECK_ADJACENT_ROOMS_FOR_OPEN_DOORS_LOOKUP_TABLE[j] {
                    let rev = DUNGEON_CHECK_ADJACENT_ROOMS_FOR_OPEN_DOORS_LOOKUP_TABLE2[j] as u8;
                    for door in 0..8 {
                        let cur = self.game_state.dungeon.doors.door_tilemap_address(door);
                        if cur as u8 == rev {
                            let kind = (cur >> 8) as u8;
                            if kind == 0x30 {
                                break;
                            }
                            if kind == 0x44 || kind == 0x18 {
                                if room
                                    != self.game_state.dungeon.room_tracking.previous_room_index()
                                {
                                    break;
                                }
                                self.dungeon_environment_mut().clear_trapdoors_down();
                            } else if self.game_state.dungeon.door_setup.adjacent_door_flags()
                                & upper_bitmask(i)
                                == 0
                            {
                                break;
                            }
                            let opened = self
                                .game_state
                                .dungeon
                                .doors
                                .opened_doors_including_adjacent()
                                | upper_bitmask(door);
                            self.dungeon_doors_mut()
                                .set_opened_doors_including_adjacent(opened);
                            break;
                        }
                    }
                    break;
                }
                j += 1;
            }
        }
    }

    fn load_adjacent_room_doors(&mut self, room: usize) {
        let flags = (self.saved_room_flags(room) & 0xf000) | 0x0f00;
        self.dungeon_room_doors_mut().set_adjacent_door_flags(flags);
        let Some(doors) = self.GetRoomDoorInfo(room).map(Vec::from) else {
            self.dungeon_room_doors_mut().mark_no_adjacent_doors();
            return;
        };
        self.dungeon_room_doors_mut()
            .load_adjacent_doors_from_room_info(&doors);
    }

    /// ROM: `RoomDraw_DrawFloorsCurrentRoom`
    pub(super) fn room_draw_draw_floors_current_room(&mut self) {
        let room = self.game_state.world.location.dungeon_room() as usize;
        let Some(room_layout) = self.dungeon_room_layout(room).map(Vec::from) else {
            return;
        };
        if room_layout.is_empty() {
            return;
        }
        self.room_draw_draw_floors(&room_layout);
    }

    pub(super) fn begin_dungeon_floor_draw_current_room(&mut self) -> Option<DungeonFloorDrawWork> {
        let room = self.game_state.world.location.dungeon_room();
        let floor_types = *self.dungeon_room_layout(usize::from(room))?.first()?;
        self.dungeon_room_tilemaps_mut().copy_bg2_draw_line_offsets();
        self.dungeon_room_parser_mut()
            .set_floor_1_filler_low(floor_types & 0xf0);
        self.dungeon_room_parser_mut().set_floor_1_filler_high(0);
        Some(DungeonFloorDrawWork {
            room,
            floor_types,
            completed_words: 0,
        })
    }

    pub(in crate::zelda_rtl) fn advance_dungeon_floor_draw_one(
        &mut self,
        work: &mut DungeonFloorDrawWork,
        ordinal: u16,
        source_address: u32,
        source_value: u16,
    ) {
        assert_eq!(self.game_state.world.location.dungeon_room(), work.room);
        assert_eq!(ordinal, work.completed_words + 1);
        let (base, byte_offset, source_offset, address) = work.next_tile();
        assert_eq!(source_address, address,
            "translated floor draw reached a different tilemap address");
        let tile = read_word_from_slice(
            self.asset_raw(69).expect("missing predefined dungeon tile asset"),
            source_offset,
        );
        assert_eq!(source_value, tile,
            "translated floor asset disagrees with source tile value");
        self.dungeon_room_tilemaps_mut()
            .set_room_tilemap_word_by_byte_offset(base, byte_offset, tile);
        work.completed_words = ordinal;
        if ordinal == 4096 {
            self.dungeon_room_tilemaps_mut().copy_bg1_draw_line_offsets();
            self.dungeon_room_parser_mut()
                .set_floor_2_filler_low((work.floor_types & 0x0f) << 4);
            self.dungeon_room_parser_mut().set_floor_2_filler_high(0);
        } else if ordinal == DungeonFloorDrawWork::TOTAL_WORDS {
            self.dungeon_room_load_mut().set_load_ptr_offset(1);
        }
    }

    /// Immutable floor tiles and pointer tables, shared with semantic drawing.
    pub(crate) fn native_dungeon_floor_data(&self) -> Option<std::collections::BTreeMap<u32, u8>> {
        use crate::game_state::{
            DUNGEON_DRAW_OBJECT_OFFSETS_BG1, DUNGEON_DRAW_OBJECT_OFFSETS_BG2,
        };
        let mut bytes = std::collections::BTreeMap::new();
        for (base, data) in [
            (0x0186f8, DUNGEON_DRAW_OBJECT_OFFSETS_BG1.as_slice()),
            (0x018719, DUNGEON_DRAW_OBJECT_OFFSETS_BG2.as_slice()),
            (0x009b52, self.asset_raw(69)?.get(..256)?),
        ] {
            for (i, &value) in data.iter().enumerate() {
                bytes.insert(base + i as u32, value);
            }
        }
        for (i, &offset) in DUNGEON_QUADRANT_OFFSETS.iter().enumerate() {
            for (b, value) in (offset as u16).to_le_bytes().into_iter().enumerate() {
                bytes.insert(0x009b02 + (i * 2 + b) as u32, value);
            }
        }
        Some(bytes)
    }

    /// ROM: `RoomDraw_DrawFloors`
    pub(super) fn room_draw_draw_floors(&mut self, level_data: &[u8]) {
        // Cycle ledger: RoomDraw_DrawFloors $01:89DC, entered with M=16 X=16.
        let _scope = crate::cycle_ledger::routine(0x01_89dc);
        let offs = self.game_state.dungeon.room_load.load_ptr_offset() as usize;
        let floor_types = level_data.get(offs).copied().unwrap_or(0);
        // $0189DC LDX #$1E (24), then $0189DF-$0189EE: the eleven-pass copy of
        // the BG2 line pointers (X = $1E, $1B, ..., 0; 230 per pass, BPL taken
        // ten times): 24 + 11 * 230 + 10 * 6.
        crate::cycle_ledger::charge(2_614);
        self.dungeon_room_tilemaps_mut()
            .copy_bg2_draw_line_offsets();
        self.dungeon_room_parser_mut()
            .set_floor_1_filler_low(floor_types & 0xf0);
        self.dungeon_room_parser_mut().set_floor_1_filler_high(0);
        // $0189F0-$018A03: read the floor byte, INC $BA, store filler 1, the
        // JSR RoomDraw_FloorChunks (46, the callee charges itself) and the
        // LDX #$1E that follows its return.
        crate::cycle_ledger::charge(352);
        self.room_draw_floor_chunks(0x4000, (floor_types & 0xf0) as usize);

        // $018A06-$018A0F: the eleven-pass copy of the BG1 line pointers
        // (144 per pass, BPL taken ten times): 11 * 144 + 10 * 6.
        crate::cycle_ledger::charge(1_644);
        self.dungeon_room_tilemaps_mut()
            .copy_bg1_draw_line_offsets();
        self.dungeon_room_parser_mut()
            .set_floor_2_filler_low((floor_types & 0x0f) << 4);
        self.dungeon_room_parser_mut().set_floor_2_filler_high(0);
        // $018A11-$018A1E: filler 2 = (byte & $0F) << 4, then the code falls
        // through into RoomDraw_FloorChunks at $018A1F (no JSR): that second
        // pass runs under this routine's frame, so it uses the unscoped body.
        crate::cycle_ledger::charge(202);
        self.room_draw_floor_chunks_body(0x2000, ((floor_types & 0x0f) << 4) as usize);
        self.dungeon_room_load_mut().set_load_ptr_offset(1);
    }

    /// ROM: `RoomDraw_DrawAllObjectsCurrentRoom`
    pub(super) fn room_draw_draw_all_objects_current_room(&mut self) {
        let room = self.game_state.world.location.dungeon_room() as usize;
        let Some(room_layout) = self.dungeon_room_layout(room).map(Vec::from) else {
            return;
        };
        let old_offs = self.game_state.dungeon.room_load.load_ptr_offset() as usize;
        let layout = room_layout.get(old_offs).copied().unwrap_or(0) as usize;
        self.dungeon_room_parser_mut()
            .set_room_layout_and_starting_quadrant(layout as u16);
        if let Some(default_layout) = self.default_room_layout(layout >> 2).map(Vec::from) {
            self.dungeon_room_load_mut().set_load_ptr_offset(0);
            self.room_data_draw_objects_from(&default_layout);
        }

        self.dungeon_room_load_mut()
            .set_load_ptr_offset(old_offs.saturating_add(1) as u16);
        self.room_data_draw_objects_from(&room_layout);
        let pos = self
            .game_state
            .dungeon
            .room_load
            .load_ptr_offset()
            .wrapping_add(2);
        self.dungeon_room_load_mut().set_load_ptr_offset(pos);
        self.dungeon_room_tilemaps_mut()
            .copy_bg2_draw_line_offsets();
        self.room_data_draw_objects_from(&room_layout);
        let pos = self
            .game_state
            .dungeon
            .room_load
            .load_ptr_offset()
            .wrapping_add(2);
        self.dungeon_room_load_mut().set_load_ptr_offset(pos);
        self.dungeon_room_tilemaps_mut()
            .copy_bg1_draw_line_offsets();
        self.room_data_draw_objects_from(&room_layout);
        self.dungeon_room_load_mut().set_load_ptr_offset(0x0120);
    }

    /// Immutable parameters, dispatch addresses and tile data for native object owners.
    pub(crate) fn native_dungeon_object_data(&self) -> Option<std::collections::BTreeMap<u32, u8>> {
        use crate::cycle_models::dungeon_data::*;
        let mut bytes = self.native_dungeon_floor_data()?;
        let mut insert_word = |address: u32, value: u16| {
            for (i, v) in value.to_le_bytes().into_iter().enumerate() {
                bytes.insert(address + i as u32, v);
            }
        };
        for (base, targets) in [
            (0x018200, OBJECT_TARGETS_ORDINARY.as_slice()),
            (0x018470, OBJECT_TARGETS_EXTENDED.as_slice()),
            (0x0185f0, OBJECT_TARGETS_SIZED.as_slice()),
            (0x0186f0, OBJECT_TARGETS_DOOR.as_slice()),
        ] {
            for (i, &target) in targets.iter().enumerate() { insert_word(base + i as u32 * 2, target); }
        }
        for (base, count, param) in [
            (0x018000, 248, object_subtype1_param as fn(u8) -> Option<usize>),
            (0x0183f0, 64, object_subtype2_param as fn(u8) -> Option<usize>),
            (0x0184f0, 128, object_subtype3_param as fn(u8) -> Option<usize>),
        ] {
            for i in 0..count { insert_word(base + i as u32 * 2, param(i as u8)? as u16); }
        }
        for (base, sizes) in [(0x009b0a, MOVING_WALL_SIZE_TABLE0), (0x009b12, MOVING_WALL_SIZE_TABLE1)] {
            for (i, size) in sizes.into_iter().enumerate() { insert_word(base + i as u32 * 2, size); }
        }
        // Water overlays use 2..5 blocks, their pixel radii, and 2*n-1 tile rows.
        for indicator in 0..4u16 {
            let blocks = indicator + 2;
            insert_word(0x009b3a + u32::from(indicator) * 2, blocks);
            insert_word(0x009b42 + u32::from(indicator) * 2, blocks << 4);
            insert_word(0x009b4a + u32::from(indicator) * 2, blocks * 2 - 1);
        }
        for (i, mask) in DUNGEON_CHEST_OPEN_MASKS.into_iter().enumerate() {
            insert_word(0x009900 + i as u32 * 2, mask);
        }
        for i in 0..16 {
            insert_word(0x0098c0 + i as u32 * 2, upper_bitmask(i));
        }
        for (i, value) in DOOR_TYPE_REMAP.into_iter().enumerate() {
            insert_word(0x009a02 + i as u32 * 2, u16::from(value));
        }
        // Opened stair-mask types can index past the right-door table into
        // the adjacent animation sources; retain their actual data addresses.
        for (base, words) in [(0x00997e, DOOR_POSITION_UP.as_slice()), (0x009996, DOOR_POSITION_DOWN.as_slice()), (0x0099ae, DOOR_POSITION_LEFT.as_slice()), (0x0099c6, DOOR_POSITION_RIGHT.as_slice()), (0x0099de, DOOR_BLAST_WALL_UP_DESTINATIONS.as_slice()), (0x00cd9e, DOOR_TYPE_SRC_UP.as_slice()), (0x00ce06, DOOR_TYPE_SRC_DOWN.as_slice()), (0x00ce66, DOOR_TYPE_SRC_LEFT.as_slice()), (0x00cec6, DOOR_TYPE_SRC_RIGHT.as_slice()), (0x00cf24, DOOR_ANIMATION_UP_SOURCES.as_slice())] {
            for (i, &value) in words.iter().enumerate() {
                insert_word(base + i as u32 * 2, value);
            }
        }
        // High bytes of the two saved-room tag masks used by the wall gate.
        for slot in 0..2 {
            bytes.insert(0x0098c7 + slot * 2, ((0x1000u16 >> slot) >> 8) as u8);
        }
        for (i, &value) in self.asset_raw(69)?.iter().enumerate() {
            bytes.insert(0x009b52 + i as u32, value);
        }
        for (i, &value) in WATERGATE_LAYOUT_BYTES.iter().enumerate() {
            bytes.insert(0x04f1cd + i as u32, value);
        }
        Some(bytes)
    }

    /// Complete immutable room streams at their physical addresses. This composes
    /// native assets only; mutable saved objects and parser scratch stay on the bus.
    pub(crate) fn native_dungeon_room_data(&self) -> Option<std::collections::BTreeMap<u32, u8>> {
        use crate::cycle_models::dungeon_data::{ROOM_POINTERS, DEFAULT_LAYOUT_POINTERS};
        let mut bytes = self.native_dungeon_object_data()?;
        let mut insert = |address: u32, data: &[u8]| -> Option<()> {
            for (i, &value) in data.iter().enumerate() {
                if let Some(old) = bytes.insert(address + i as u32, value) {
                    if old != value { return None; }
                }
            }
            Some(())
        };
        for (base, pointers, asset, offsets) in [
            (0x1f8000, ROOM_POINTERS.as_slice(), 3, 4),
            (0x04ef2f, DEFAULT_LAYOUT_POINTERS.as_slice(), 46, 47),
        ] {
            let data = self.asset_raw(asset)?;
            let offsets: Vec<_> = self.asset_raw(offsets)?.chunks_exact(2)
                .map(|v| usize::from(u16::from_le_bytes([v[0], v[1]]))).collect();
            if offsets.len() != pointers.len() { return None; }
            for (i, &address) in pointers.iter().enumerate() {
                insert(base + i as u32 * 3, &address.to_le_bytes()[..3])?;
                let start = offsets[i];
                let end = offsets.iter().copied().filter(|&v| v > start).min().unwrap_or(data.len());
                insert(address, data.get(start..end)?)?;
            }
        }
        for (address, value) in self.native_dungeon_header_data()? {
            insert(address, &[value])?;
        }
        Some(bytes)
    }

    /// Immutable selectors used by the room-buffer preparation and DMA caller.
    pub(crate) fn native_dungeon_upload_data(&self) -> Option<std::collections::BTreeMap<u32, u8>> {
        let mut bytes = self.native_dungeon_room_data()?;
        for (i, &value) in UPLOAD_BG_SRCS.iter().enumerate() {
            for (b, v) in (value as u16).to_le_bytes().into_iter().enumerate() {
                bytes.insert(0x00910f + (i * 2 + b) as u32, v);
            }
        }
        for (i, &v) in UPLOAD_BG_DSTS.iter().enumerate() { bytes.insert(0x00912f + i as u32, v); }
        for (i, &v) in NMI_VRAM_ADDRS.iter().enumerate() { bytes.insert(0x009888 + i as u32, v as u8); }
        Some(bytes)
    }

    /// ROM: `RoomData_DrawObjects_from`
    pub(super) fn room_data_draw_objects_from(&mut self, layout: &[u8]) {
        // Cycle ledger: RoomDraw_DrawAllObjects $01:88E4, entered with M=16 X=16.
        let _scope = crate::cycle_ledger::routine(0x01_88e4);
        let mut progress = DungeonObjectStreamProgress::default();
        while self.room_data_draw_next_object_from(layout, &mut progress) {}
    }

    /// Execute one complete object or door from the current stream. A caller
    /// can retain `progress` and the room-load pointer across CPU interrupts.
    fn room_data_draw_next_object_from(
        &mut self,
        layout: &[u8],
        progress: &mut DungeonObjectStreamProgress,
    ) -> bool {
        if !progress.in_doors {
        loop {
            // $0188E4-$0188EF: STZ $B2 / STZ $B4, read the object word, CMP #$FFFF.
            crate::cycle_ledger::charge(192);
            self.dungeon_room_load_mut()
                .set_draw_width_indicator_word(0);
            self.dungeon_room_load_mut()
                .set_draw_height_indicator_word(0);
            let pos = self.game_state.dungeon.room_load.load_ptr_offset() as usize;
            let raw = read_word_from_slice(layout, pos);
            if raw == 0xffff {
                // BEQ $0188FD taken (+6) and the RTS at $0188FD (42).
                crate::cycle_ledger::charge(48);
                return false;
            }
            // $0188F1-$0188F6: STA $00, CMP #$FFF0.
            crate::cycle_ledger::charge(72);
            if raw == 0xfff0 {
                // BEQ $0188FE taken (+6), then $0188FE-$018900 INC $BA twice (108).
                crate::cycle_ledger::charge(114);
                progress.in_doors = true;
                break;
            }
            // $0188F8-$0188FB: JSR RoomData_DrawObject (46, the callee charges
            // itself; the object drawer's RTS returns here) and the BRA back to
            // $0188E4 (22, already priced taken).
            crate::cycle_ledger::charge(68);
            let idx = layout[pos + 2];
            self.dungeon_room_load_mut()
                .set_load_ptr_offset(pos.wrapping_add(3) as u16);
            self.room_data_draw_object(raw, idx);
            return true;
        }
        }
        loop {
            // $018902-$018909: read the door word, CMP #$FFFF. The INC $BA pair
            // before each read is charged with the previous block ($0188FE for
            // the first door, $018910 for the rest).
            crate::cycle_ledger::charge(128);
            let pos = self
                .game_state
                .dungeon
                .room_load
                .load_ptr_offset()
                .wrapping_add(2) as usize;
            self.dungeon_room_load_mut().set_load_ptr_offset(pos as u16);
            let raw = read_word_from_slice(layout, pos);
            if raw == 0xffff {
                // BEQ $0188FD taken (+6) and the RTS at $0188FD (42).
                crate::cycle_ledger::charge(48);
                return false;
            }
            // $01890B-$018914: STA $00, JSR RoomData_DrawObject_Door (46, the
            // callee charges itself), INC $BA twice, BRA $018902 (priced taken).
            crate::cycle_ledger::charge(208);
            self.room_data_draw_object_door(raw);
            return true;
        }
    }

    /// ROM: `RoomData_DrawObject`
    pub(super) fn room_data_draw_object(&mut self, raw: u16, idx: u8) {
        // Cycle ledger: RoomData_DrawObject $01:893C, entered with M=16 X=16
        // (SEP #$20 first). Each arm ends in a `JMP ($0E)` (40, priced inside
        // the block) into the object drawer, whose own RTS returns to
        // RoomDraw_DrawAllObjects; the drawers charge themselves, so this
        // routine's scope covers exactly the decode plus the dispatch.
        let _scope = crate::cycle_ledger::routine(0x01_893c);
        if raw & 0xfc != 0xfc {
            // $01893C-$018942: SEP #$20, AND #$FC, CMP #$FC, BEQ not taken (70),
            // then $018944-$018970: width/height, the third byte, $BA += 3,
            // the position word, REP #$20, ASL, CMP #$01F0 with BCS priced not
            // taken (582).
            crate::cycle_ledger::charge(652);
            let width = (raw & 3) as u8;
            let height = ((raw >> 8) & 3) as u8;
            self.dungeon_room_load_mut()
                .set_draw_dimensions_words(u16::from(width), u16::from(height));
            let x = (raw as u8 >> 2) as u16;
            let y = raw >> 10;
            let dsto = y * 64 + x;
            if idx < 0xf8 {
                // $018972-$018980: subtype-1 table lookup and JMP ($0E).
                crate::cycle_ledger::charge(228);
                self.LoadType1ObjectSubtype1(idx, width, height, dsto);
            } else {
                // BCS $0189B8 taken (+6), then $0189B8-$0189D9: the subtype-3
                // index from bits 1-3, width and height, table lookup, JMP ($0E).
                crate::cycle_ledger::charge(524);
                let object = ((idx & 7) << 4) | (((raw >> 8) as u8 & 3) << 2) | (raw as u8 & 3);
                self.LoadType1ObjectSubtype3(object, dsto);
            }
        } else {
            // $01893C-$018942 with BEQ $018983 taken (70 + 6), then
            // $018983-$0189B5: the subtype-2 position from both words, $BA += 3,
            // table lookup and JMP ($0E) (766).
            crate::cycle_ledger::charge(842);
            let x = ((raw & 3) << 4) | ((raw >> 12) & 0x0f);
            let y = (((raw >> 8) & 0x0f) << 2) | ((idx as u16) >> 6);
            let dsto = y * 64 + x;
            self.LoadType1ObjectSubtype2(idx & 0x3f, dsto);
        }
    }

    /// ROM: `Dungeon_UploadRoomQuadrants`
    pub(super) fn dungeon_upload_room_quadrants(&mut self) {
        let mut work = self.begin_dungeon_room_upload();
        while work.completed_quadrants() != 4 {
            self.advance_dungeon_room_upload_one(&mut work);
        }
        self.finish_dungeon_room_upload(work);
    }

    pub(super) fn begin_dungeon_room_upload(&mut self) -> DungeonRoomUploadWork {
        self.dungeon_room_load_mut().clear_quadrant_upload_index();
        self.set_overworld_map_state(0);
        DungeonRoomUploadWork { completed_quadrants: 0 }
    }

    pub(in crate::zelda_rtl) fn advance_dungeon_room_upload_one(&mut self, work: &mut DungeonRoomUploadWork) {
        assert!(work.completed_quadrants < 4);
        assert_eq!(self.game_state.dungeon.room_load.quadrant_upload_index(),
            work.completed_quadrants * 4);
        self.tile_map_prep_not_water_on_tag();
        self.upload_tilemap_now();
        self.dungeon_prepare_next_room_quadrant_upload();
        self.upload_tilemap_now();
        work.completed_quadrants += 1;
    }

    pub(super) fn finish_dungeon_room_upload(&mut self, work: DungeonRoomUploadWork) {
        assert_eq!(work.completed_quadrants, 4);
        assert_eq!(self.game_state.dungeon.room_load.quadrant_upload_index(), 16);
        self.clear_pending_nmi_subroutine();
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
    }

    /// ROM: `Dungeon_PrepareNextRoomQuadrantUpload`
    pub(in crate::zelda_rtl) fn dungeon_prepare_next_room_quadrant_upload(&mut self) {
        // $00:913F..91C3: 254 setup, 128 passes of the eight-word copy
        // (918, BNE taken on 15 of every 16), eight row steps (172, BNE
        // taken on seven) and the 332-cycle tail with its RTL; the copy is
        // the same for every quadrant.
        crate::cycle_ledger::charge_routine(
            0x00_913f,
            254 + 128 * 918 + 120 * 6 + 8 * 172 + 7 * 6 + 332,
        );
        let quadrant = self.game_state.dungeon.room_load.quadrant_upload_index() as usize;
        let work = self.begin_dungeon_quadrant_upload(ROOM_BG2_TILEMAP_BASE, quadrant, 0);
        self.complete_dungeon_quadrant_upload(work);
    }

    pub(in crate::zelda_rtl) fn complete_dungeon_quadrant_upload(&mut self, mut work: DungeonQuadrantUploadWork) {
        self.advance_dungeon_quadrant_upload(&mut work, 1024);
        // $00:91B3 precedes the target-page and NMI-request stores.
        self.dungeon_room_load_mut()
            .advance_quadrant_upload_index_by(4);
        self.publish_dungeon_quadrant_upload(work);
    }

    /// ROM: `Dungeon_PrepareNextRoomQuadrantUploadFrom`
    pub(super) fn dungeon_prepare_next_room_quadrant_upload_from(
        &mut self,
        source_base: usize,
        quadrant_upload: usize,
        dst_bias: u8,
    ) {
        let mut work = self.begin_dungeon_quadrant_upload(source_base, quadrant_upload, dst_bias);
        self.advance_dungeon_quadrant_upload(&mut work, 1024);
        self.publish_dungeon_quadrant_upload(work);
    }

    pub(super) fn begin_dungeon_quadrant_upload(&self, source_base: usize, quadrant_upload: usize,
        dst_bias: u8) -> DungeonQuadrantUploadWork {
        let table_index = ((self.screen_transition() as usize & 0x0f) + quadrant_upload)
            & ROOM_QUADRANT_UPLOAD_TABLE_MASK;
        DungeonQuadrantUploadWork { source_base, table_index, dst_bias, completed_words: 0 }
    }

    pub(super) fn advance_dungeon_quadrant_upload(&mut self, work: &mut DungeonQuadrantUploadWork, words: u16) {
        assert!(words >= work.completed_words && words <= 1024);
        while work.completed_words < words {
            let ordinal = usize::from(work.completed_words);
            let row = ordinal / 128;
            let block = ordinal % 128 / 8;
            let y = ordinal % 8 / 2;
            let x = ordinal % 2;
            let src = UPLOAD_BG_SRCS[work.table_index] + row * 512 + block * 4 + xy(x, y) * 2;
            let dst = (row * 128 + block * 2 + y * 32 + x) * 2;
            let value = self.game_state.dungeon.room_tilemaps
                .room_tilemap_word_by_byte_offset(work.source_base, src);
            self.write_vram_upload_tilemap_word(dst, value);
            work.completed_words += 1;
        }
    }

    fn publish_dungeon_quadrant_upload(&mut self, work: DungeonQuadrantUploadWork) {
        assert_eq!(work.completed_words, 1024);
        self.set_nmi_load_target_page(UPLOAD_BG_DSTS[work.table_index] + work.dst_bias);
        self.set_pending_nmi_subroutine(1);
        self.set_core_update_disable_flag(1);
    }

    pub(in crate::zelda_rtl) fn SavePalaceDeaths(&mut self) {
        let j = self.game_state.inventory.save_progress.palace_index_x2() as usize;
        let deaths = self
            .game_state
            .inventory
            .save_progress
            .pending_death_save_counter();
        self.save_progress_mut()
            .set_death_count_for_palace(j >> 1, deaths);
        if j != 8 {
            self.save_progress_mut().clear_pending_death_save_counter();
        }
    }

    pub(in crate::zelda_rtl) fn upload_tilemap_now(&mut self) {
        let Some((dst, word_count)) =
            full_tilemap_nmi_vram_region(self.game_state.display.nmi_load_target_page())
        else {
            return;
        };
        for i in 0..word_count {
            self.ppu.vram[dst + i] = self.vram_upload_tilemap_word(i * 2);
        }
        self.clear_vram_upload_cursor();
        self.clear_core_update_disable_flag();
    }

    pub(super) fn dungeon_room_layout(&self, room: usize) -> Option<&[u8]> {
        let offset = self.asset_u16(4, room) as usize;
        let data = self.asset_raw(3)?;
        data.get(offset..)
    }

    pub(super) fn default_room_layout(&self, index: usize) -> Option<&[u8]> {
        let offset = self.asset_u16(47, index) as usize;
        let data = self.asset_raw(46)?;
        data.get(offset..)
    }

    pub(super) fn dungeon_room_header(&self, room: usize) -> Option<&[u8]> {
        let offset = self.asset_u16(7, room) as usize;
        let data = self.asset_raw(6)?;
        data.get(offset..)
    }

    pub(in crate::zelda_rtl) fn SetAndSaveVisitedQuadrantFlags(&mut self) {
        let player_quadrant_y = self.game_state.player.follower_link.quadrant_y();
        let player_quadrant_x = self.game_state.player.follower_link.quadrant_x();
        let index = self
            .game_state
            .world
            .transient
            .dungeon_quadrant_visit_index(player_quadrant_y, player_quadrant_x);
        let flag = DUNGEON_QUADRANT_VISITING_FLAGS[index];
        let visited = self.dungeon_room_load_mut().or_quadrants_visited(flag);

        let room = self.game_state.world.location.dungeon_room() as usize;
        let saved = self.saved_room_flags(room) | visited;
        self.set_saved_room_flags(room, saved);
    }

    /// ROM: `Dungeon_PlayBlipAndCacheQuadrantVisits`
    pub(in crate::zelda_rtl) fn dungeon_play_blip_and_cache_quadrant_visits(&mut self) {
        self.set_hud_floor_changed_timer(1);
        self.set_sound_effect_2(36);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn begin_module11_02_room_header_if_source_returned(&mut self) {
        self.begin_early_dungeon_room_header_if_source_returned(
            EarlyDungeonRoomOwner::FallingEntrance,
        );
    }

    pub(in crate::zelda_rtl) fn begin_early_dungeon_room_header_if_source_returned(
        &mut self,
        owner: EarlyDungeonRoomOwner,
    ) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_ROOM_HEADER_LIVE_TRIAL").is_none() {
            return;
        }
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        if trace.dungeon_header_index_writes.is_empty() {
            return;
        }
        assert_eq!(trace.dungeon_header_index_writes.len(), 1);
        let source_value = trace.dungeon_header_index_writes[0].value;
        let room = self.game_state.world.location.dungeon_room();
        assert_eq!(source_value, room.wrapping_mul(3),
            "source header index disagrees with translated room");
        assert!(self.pending_early_dungeon_room_hdma.is_none(),
            "source-ordered room header began twice");
        assert!(self.pending_early_dungeon_room_owner.is_none());
        let floor_draw_trial =
            crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_FLOOR_DRAW_LIVE_TRIAL").is_some();
        if floor_draw_trial {
            self.dungeon_torch_mut().clear_lit_torches();
            self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
        }
        let hdma = self.game_state.display.hdma_enable_mask;
        self.clear_hdma_enable_mask();
        self.dungeon_load_header();
        self.pending_early_dungeon_room_hdma = Some(hdma);
        self.pending_early_dungeon_room_owner = Some(owner);
        if floor_draw_trial {
            self.dungeon_load_room_before_floors();
            assert!(self.active_early_dungeon_floor_draw.is_none());
            self.active_early_dungeon_floor_draw =
                self.begin_dungeon_floor_draw_current_room();
            assert!(self.active_early_dungeon_floor_draw.is_some(),
                "source floor draw entered without translated room layout");
        }
    }

    pub(super) fn finish_early_dungeon_room_load(&mut self, owner: EarlyDungeonRoomOwner) -> bool {
        let Some(hdma) = self.pending_early_dungeon_room_hdma.take() else {
            assert!(self.pending_early_dungeon_room_owner.is_none());
            assert!(self.active_early_dungeon_floor_draw.is_none());
            return false;
        };
        assert_eq!(self.pending_early_dungeon_room_owner.take(), Some(owner));
        if let Some(work) = self.active_early_dungeon_floor_draw.take() {
            assert_eq!(work.completed_words(), DungeonFloorDrawWork::TOTAL_WORDS,
                "room draw reached caller return before floors completed");
            if let Some(upload) = self.active_early_dungeon_room_upload.take() {
                assert_eq!(upload.completed_quadrants(), 4,
                    "room upload reached caller return before four quadrants");
                self.dungeon_load_and_draw_room_after_early_upload(upload, hdma);
            } else if let Some(mut objects) = self.active_early_dungeon_object_draw.take() {
                assert!(!self.advance_dungeon_object_draw_one(&mut objects),
                    "room draw reached caller return with objects pending");
                assert!(objects.finished());
                self.dungeon_load_and_draw_room_after_early_objects(hdma);
            } else {
                self.dungeon_load_and_draw_room_after_early_floor(hdma);
            }
        } else {
            self.dungeon_load_and_draw_room_after_early_header(hdma);
        }
        true
    }
}
