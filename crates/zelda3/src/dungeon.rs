// Methods ported from zelda3/src/dungeon.c and included inside ZeldaState.

use super::*;
use crate::game_state::constants::{
    ALT_SPRITE_STATE, ALT_SPRITE_TYPE, ANCILLA_TYPE, DIALOGUE_MESSAGE_INDEX, GARNISH_TYPE,
    MAIN_MODULE, MESSAGING_MODULE, OVERLORD_TYPE, SUBMODULE, SUBSUBMODULE, TEXT_INCREMENTAL_STATE,
    TEXT_RENDER_STATE,
};
use crate::types::Point16U;
use crate::zelda_rtl::misc::DUNG_ANIMATED_TILES;
use crate::zelda_rtl::sprite::SpriteSpawnInfo;

mod dungeon_shared;
use dungeon_shared::*;

impl ZeldaState {
    pub fn parity_probe_direct_entrance(&mut self, entrance_index: u16) -> u16 {
        self.set_which_entrance(entrance_index);
        self.Dungeon_LoadEntrance();
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
        self.Dungeon_LoadAndDrawEntranceRoom(entrance as u8);
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

    pub(super) fn Dungeon_LoadAndDrawEntranceRoom(&mut self, room: u8) {
        self.set_which_entrance_byte(room);
        self.Dungeon_LoadEntrance();
        self.dungeon_torch_mut().clear_lit_torches();
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
        self.Dungeon_LoadAndDrawRoom();
        self.Dungeon_ResetTorchBackgroundAndPlayer();
    }

    pub(super) fn Dungeon_LoadAndDrawRoom(&mut self) {
        let hdma = self.game_state.display.hdma_enable_mask;
        self.clear_hdma_enable_mask();
        self.Dungeon_LoadRoom();
        self.clear_screen_transition();
        self.set_overworld_map_state(0);
        self.Dungeon_UploadRoomQuadrants();
        self.set_hdma_enable_mask(hdma);
        self.clear_pending_nmi_subroutine();
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
    }

    pub(super) fn Dungeon_LoadEntrance(&mut self) {
        self.set_indoor_flag(1);
        if self.game_state.system_signals.game_over_check_flag() != 0 {
            self.clear_game_over_check_flag();
        } else {
            self.save_exit_area_index();
            self.save_exit_tm_copy();
            self.save_exit_bg2_live_scroll();
            self.follower_link_state_mut()
                .store_overworld_exit_position_from_current();
            let overworld_screen = self.game_state.world.location.overworld_screen();
            self.set_exit_screen_index(overworld_screen);
            self.set_overworld_exit_map16_src_off(
                self.game_state.world.overworld.map16.active_load.src_off,
            );
            self.save_exit_camera_coords();
            let room_bounds_y_start = self.game_state.world.room_bounds.packed_bound(0);
            let room_bounds_y_end = self.game_state.world.room_bounds.packed_bound(1);
            let room_bounds_x_start = self.game_state.world.room_bounds.packed_bound(2);
            let room_bounds_x_end = self.game_state.world.room_bounds.packed_bound(3);
            self.save_exit_room_bounds(
                room_bounds_y_start,
                room_bounds_y_end,
                room_bounds_x_start,
                room_bounds_x_end,
            );
            self.copy_exit_scroll_targets();
            self.copy_exit_scroll_counters();
            self.dungeon_entrance_backup_mut().cache_exit_tile_themes();
            self.set_overworld_screen(0);
            self.dungeon_entrance_backup_mut()
                .clear_overworld_screen_high();
            self.clear_overlay_index_word();
            self.dungeon_entrance_backup_mut().clear_overlay_high();
        }
        self.set_bg1_y_offset(0);
        self.set_bg1_x_offset(0);
        self.clear_game_over_check_flag();

        if self.game_state.sprites.follower_runtime.indicator_word() == 4
            || self.game_state.system_signals.restart_check_flag() != 0
        {
            let i = self
                .game_state
                .inventory
                .save_progress
                .which_starting_point() as usize;
            let entrance = self.asset_u8(44, i);
            self.set_which_entrance(entrance as u16);
            self.dungeon_load_entrance_fields(i, &STARTING_POINT_ASSETS);
            self.follower_link_state_mut().set_facing(2);
            self.follower_link_state_mut().clear_doorway_state();
            let queued_music_control = self.asset_u8(45, i);
            self.set_queued_music_control(queued_music_control);
            if i == 0 && self.game_state.inventory.save_progress.progress_indicator() == 0 {
                self.set_queued_music_control(0xff);
            }
            self.clear_restart_check_flag();
        } else {
            let i = self.game_state.world.region.which_entrance() as usize;
            let room = self.dungeon_load_entrance_fields(i, &ENTRANCE_DATA_ASSETS);
            self.dungeon_object_tracking_mut()
                .set_big_rock_starting_address(0);
            self.follower_link_state_mut()
                .set_facing(if i == 0 || i == 0x43 { 2 } else { 0 });
            let doorway_state = self.asset_u8(ENTRANCE_DATA_ASSETS.doorway_orientation, i);
            self.follower_link_state_mut()
                .set_doorway_state(doorway_state);
            let queued_music_control = self.zelda_get_entrance_music_track(i as i32);
            self.set_queued_music_control(queued_music_control);
            if self.game_state.system_signals.queued_music_control() == 3
                && self.game_state.inventory.save_progress.progress_indicator() >= 2
            {
                self.set_queued_music_control(18);
            }
            if room >= 0x100 {
                self.dungeon_stair_movement_mut().set_current_floor(0);
            }
        }

        self.follower_link_state_mut().disable_oam_offsets();
        {
            self.follower_link_state_mut().reset_direction_masks();
            self.follower_link_state_mut().set_z_low(0xff);
            self.follower_link_state_mut().set_actual_z_velocity(0xff);
        }

        self.dungeon_room_effects_mut()
            .clear_moving_wall_torch_blink_phase();
        self.dungeon_environment_mut()
            .clear_orange_blue_barrier_state();
        let movable_init = self
            .asset_raw(53)
            .expect("missing movable block init asset")
            .to_vec();
        self.dungeon_movable_blocks_mut()
            .copy_records_from_bytes(&movable_init);
        let torch_init = self
            .asset_raw(54)
            .expect("missing torch init asset")
            .to_vec();
        {
            let mut torch = self.dungeon_torch_mut();
            torch.copy_torch_data_table(&torch_init);
            torch.copy_torch_init_to_movable_blocks(&torch_init);
        }
        let torch_junk = self
            .asset_raw(55)
            .expect("missing torch junk asset")
            .to_vec();
        self.dungeon_torch_mut().copy_torch_junk(&torch_junk);
        self.dungeon_room_parser_mut().clear_pot_reveal_masks();
        self.memorized_tile_mut().clear_entry_addresses();
    }

    fn dungeon_load_entrance_fields(&mut self, i: usize, assets: &EntranceAssetSet) -> u16 {
        let room = self.asset_u16(assets.rooms, i);
        self.set_dungeon_room(room);
        self.dungeon_room_tracking_mut().set_room_index2_word(room);

        let scroll_y = self.asset_u16(assets.scroll_y, i);
        self.set_bg1_v_copy(scroll_y);
        self.set_bg2_v_copy(scroll_y);
        self.set_bg1_y(scroll_y);
        self.set_bg2_y(scroll_y);

        let scroll_x = self.asset_u16(assets.scroll_x, i);
        self.set_bg1_h_copy(scroll_x);
        self.set_bg2_h_copy(scroll_x);
        self.set_bg1_x(scroll_x);
        self.set_bg2_x(scroll_x);

        if self
            .game_state
            .inventory
            .save_progress
            .progress_indicator_word()
            != 0
        {
            let player_y = self.asset_u16(assets.player_y, i);
            let player_x = self.asset_u16(assets.player_x, i);
            self.follower_link_state_mut().set_y(player_y);
            self.follower_link_state_mut().set_x(player_x);
        }

        let camera_y = self.asset_u16(assets.camera_y, i);
        self.set_camera_y_coord_scroll_low(camera_y);
        self.set_camera_y_coord_scroll_hi(camera_y.wrapping_add(2));
        let camera_x = self.asset_u16(assets.camera_x, i);
        self.set_camera_x_coord_scroll_low(camera_x);
        self.set_camera_x_coord_scroll_hi(camera_x.wrapping_add(2));

        self.tile_detect_position_mut()
            .set_location_calc_mask(0x01f8);
        let door_settings = self.asset_u16(assets.door_settings, i);
        self.set_ow_entrance_value(door_settings);
        self.set_up_down_scroll_target(0);
        self.set_up_down_scroll_target_end(0x0110);
        self.set_left_right_scroll_target(0);
        self.set_left_right_scroll_target_end(0x0100);

        for j in 0..4 {
            let value = (self.asset_u8(assets.relative_coords, i * 8 + j) as u16) << 8;
            let value = if j >= 2 { value | 0x10 } else { value };
            self.room_bounds_mut().set_y_bound(j, value);
        }
        for j in 0..4 {
            let value = (self.asset_u8(assets.relative_coords, i * 8 + 4 + j) as u16) << 8;
            self.room_bounds_mut().set_x_bound(j, value);
        }

        let main_tile_theme = self.asset_u8(assets.blockset, i);
        self.world_palette_theme_mut()
            .set_main_tile_theme_index(main_tile_theme);
        let current_floor = self.asset_u8(assets.floor, i);
        self.dungeon_stair_movement_mut()
            .set_current_floor(current_floor);
        let palace_index_x2 = self.asset_u8(assets.palace, i);
        self.save_progress_mut()
            .set_palace_index_x2(palace_index_x2);

        let starting_bg = self.asset_u8(assets.starting_bg, i);
        self.follower_link_state_mut()
            .set_lower_level_states(starting_bg >> 4, starting_bg & 0x0f);

        let quadrant1 = self.asset_u8(assets.quadrant1, i);
        self.set_quadrant_fullsize_x(quadrant1 >> 4);
        self.set_quadrant_fullsize_y(quadrant1 & 0x0f);
        let quadrant2 = self.asset_u8(assets.quadrant2, i);
        self.follower_link_state_mut()
            .set_quadrants_from_packed_nibbles(quadrant2);

        room
    }

    pub(super) fn Dungeon_ResetTorchBackgroundAndPlayer(&mut self) {
        let bg_properties = self.game_state.dungeon.room_load.bg2_properties() as usize;
        let mut tm = 0x16;
        let mut ts = DUNGEON_RESET_TORCH_BACKGROUND_AND_PLAYER_SPIRAL_BG_PROPERTIES[bg_properties];
        if ts < 0 {
            tm = 0x17;
            ts = 0;
        }
        if bg_properties == 2 {
            ts = 3;
        }
        self.set_main_screen_layers(tm);
        self.set_sub_screen_layers(ts as u8);
        self.hud_restore_torch_background();
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
    }

    pub(super) fn Dungeon_ResetTorchBackgroundAndPlayerInner(&mut self) {
        self.ancilla_terminate_select_interactives(0);

        if self.game_state.player.follower_link.is_running()
            && !self
                .game_state
                .enhanced_features
                .has(DUNGEON_RESET_TORCH_BACKGROUND_AND_PLAYER_INNER_FEATURES0_TURN_WHILE_DASHING)
        {
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_incapacitated_timer(0);
            {
                self.follower_link_state_mut().set_actual_z_velocity(0xff);
                self.follower_link_state_mut()
                    .set_recoil_z_velocity_for_dungeon_reset(0xff);
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(0);
            }
            self.follower_link_state_mut().set_speed_setting(0);
            self.swim_acceleration_mut().set_mode(0, 0);
            self.follower_link_state_mut().clear_running();
            self.follower_link_state_mut().set_handler_state(0);
        }
    }

    pub(super) fn LoadOWMusicIfNeeded(&mut self) {
        if self
            .game_state
            .dungeon
            .room_runtime
            .dungeon_music_type_flag()
            == 0
        {
            return;
        }
        self.dungeon_room_runtime_mut()
            .clear_dungeon_music_type_flag();
        self.load_overworld_songs();
    }

    pub(super) fn Dungeon_LoadSongBankIfNeeded(&mut self) {
        let queued = self.game_state.system_signals.queued_music_control();
        if queued == 0xff || queued == 0xf2 {
            return;
        }
        if queued == 3 || queued == 7 || queued == 14 {
            self.LoadOWMusicIfNeeded();
        } else {
            if self
                .game_state
                .dungeon
                .room_runtime
                .dungeon_music_type_flag()
                != 0
            {
                return;
            }
            self.dungeon_room_runtime_mut()
                .set_dungeon_music_type_flag(1);
            self.load_dungeon_songs();
        }
    }

    pub(super) fn ApplyGrayscaleFixed_Incremental(&mut self) {
        let mut a = self.game_state.display.palette_filter.fixed_color_red() & 0x1f;
        let target = self.game_state.dungeon.room_effects.fixed_color_plusminus();
        if a == target {
            return;
        }
        if a < target {
            a = a.wrapping_add(1);
        } else {
            a = a.wrapping_sub(1);
        }
        self.Dungeon_ApproachFixedColor_variable(a);
    }

    pub(super) fn Dungeon_ApproachFixedColor_variable(&mut self, a: u8) {
        self.set_fixed_color_red(a | 0x20);
        self.set_fixed_color_green(a | 0x40);
        self.set_fixed_color_blue(a | 0x80);
    }

    pub(super) fn Dungeon_DoubleApplyAndIncrementGrayscale(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.ApplyPaletteFilter_bounce();
        self.ApplyGrayscaleFixed_Incremental();
    }

    pub(super) fn Module07_0A_ChangeBrightness(&mut self) {
        self.OrientLampLightCone();
        self.ApplyGrayscaleFixed_Incremental();
        if self.game_state.display.palette_filter.fixed_color_red() & 0x1f
            != self.game_state.dungeon.room_effects.fixed_color_plusminus()
        {
            return;
        }
        self.set_submodule(0);
        self.set_subsubmodule(0);
    }

    pub(super) fn Module07_0B_DrainSwampPool(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => {
                if self
                    .game_state
                    .dungeon
                    .environment
                    .water_transition_counter()
                    & 7
                    == 0
                {
                    let k = ((self
                        .game_state
                        .dungeon
                        .environment
                        .water_transition_counter()
                        >> 2)
                        & 3) as usize;
                    if self.game_state.dungeon.environment.water_hdma_y_radius()
                        == self.game_state.dungeon.environment.water_hdma_y_target()
                    {
                        self.Dungeon_SetAttrForActivatedWaterOff();
                        return;
                    }
                    let delta = MODULE07_0_B_DRAIN_SWAMP_POOL_SWAMP_DRAIN_WINDOW_RADIUS_DELTAS[k]
                        as i16 as u16;
                    let y_radius = self
                        .game_state
                        .dungeon
                        .environment
                        .water_hdma_y_radius()
                        .wrapping_add(delta);
                    let x_radius = self
                        .game_state
                        .dungeon
                        .environment
                        .water_hdma_x_radius()
                        .wrapping_add(delta);
                    self.dungeon_environment_mut()
                        .set_water_hdma_y_radius(y_radius);
                    self.dungeon_environment_mut()
                        .set_water_hdma_x_radius(x_radius);
                }
                self.dungeon_environment_mut()
                    .increment_water_transition_counter();
                self.AdjustWaterHDMAWindow();
            }
            1 => {
                let tile = self.tile_word(0x01e0, 0);
                for i in 0..0x1000usize {
                    self.dungeon_room_tilemaps_mut().set_bg1_tile(i, tile);
                }
                self.dungeon_room_load_mut().clear_quadrant_upload_index();
                self.increment_subsubmodule();
            }
            2..=5 => self.Dungeon_FloodSwampWater_PrepTileMap(),
            _ => {}
        }
    }

    pub(super) fn Module07_0C_FloodSwampWater(&mut self) {
        match self.game_state.frame.subsubmodule {
            0..=3 => self.Dungeon_FloodSwampWater_PrepTileMap(),
            4..=8 => {
                if self
                    .dungeon_environment_mut()
                    .decrement_water_transition_counter()
                    == 0
                {
                    self.dungeon_environment_mut()
                        .set_water_transition_counter(4);
                    self.increment_subsubmodule();
                    let depth = i32::from(self.game_state.frame.subsubmodule) - 4;
                    self.dungeon_environment_mut().set_water_hdma_x_radius(8);
                    self.dungeon_environment_mut()
                        .set_water_hdma_y_radius_alt(0);
                    self.dungeon_environment_mut().set_water_hdma_y_radius(0x30);
                    self.Dungeon_AdjustWaterVomit(0x1654 + 0x10, depth);
                }
            }
            9 => {
                self.set_bg12_window_selection(3);
                self.set_bg34_window_selection(0);
                self.set_object_color_window_selection(0);
                self.set_main_screen_window_layers(22);
                self.set_sub_screen_window_layers(1);
                self.set_sub_screen_layers(1);
                self.set_color_window_selection(2);
                self.set_color_math_control(98);
                self.dungeon_environment_mut()
                    .set_water_transition_counter(0);
                self.increment_subsubmodule();
                self.Module07_0C_FloodSwampWater_raise_window(
                    MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_WINDOW_LEFT_DELTAS,
                    MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_WINDOW_RIGHT_DELTAS,
                );
            }
            10 => self.Module07_0C_FloodSwampWater_raise_window(
                MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_WINDOW_LEFT_DELTAS,
                MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_WINDOW_RIGHT_DELTAS,
            ),
            11 => {
                if self
                    .game_state
                    .dungeon
                    .environment
                    .water_transition_counter()
                    & 7
                    == 0
                {
                    let k = ((self
                        .game_state
                        .dungeon
                        .environment
                        .water_transition_counter()
                        >> 2)
                        & 3) as usize;
                    if self.game_state.dungeon.environment.water_hdma_y_radius()
                        == self.game_state.dungeon.environment.water_hdma_y_target()
                    {
                        self.Dungeon_SetAttrForActivatedWater();
                        return;
                    }
                    let delta = MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_FINAL_RADIUS_DELTAS[k]
                        as i16 as u16;
                    let y_radius = self
                        .game_state
                        .dungeon
                        .environment
                        .water_hdma_y_radius()
                        .wrapping_add(delta);
                    let x_radius = self
                        .game_state
                        .dungeon
                        .environment
                        .water_hdma_x_radius()
                        .wrapping_add(delta);
                    self.dungeon_environment_mut()
                        .set_water_hdma_y_radius(y_radius);
                    self.dungeon_environment_mut()
                        .set_water_hdma_x_radius(x_radius);

                    let a = self
                        .game_state
                        .dungeon
                        .environment
                        .water_hdma_y_target()
                        .wrapping_sub(y_radius);
                    if a == 0 || a == 8 {
                        self.Dungeon_AdjustWaterVomit(if a == 0 { 0x16b4 } else { 0x168c }, 5);
                    }
                }
                self.dungeon_environment_mut()
                    .increment_water_transition_counter();
                self.AdjustWaterHDMAWindow();
            }
            _ => {}
        }
    }

    fn Module07_0C_FloodSwampWater_raise_window(&mut self, tab0: [i8; 4], tab1: [i8; 4]) {
        let k = (self
            .game_state
            .dungeon
            .environment
            .water_transition_counter()
            & 3) as usize;
        let r0 = 0x0688u16
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
            .wrapping_sub(0x24);
        let x_radius = self
            .game_state
            .dungeon
            .environment
            .water_hdma_x_radius()
            .wrapping_add(tab0[k] as i16 as u16);
        let y_span = self
            .game_state
            .dungeon
            .environment
            .water_hdma_y_radius_alt()
            .wrapping_add(tab1[k] as i16 as u16);
        self.dungeon_environment_mut()
            .set_water_hdma_x_radius(x_radius);
        self.dungeon_environment_mut()
            .set_water_hdma_y_radius_alt(y_span);
        if y_span >= r0 {
            self.dungeon_room_load_mut().set_bg2_properties(7);
            self.increment_subsubmodule();
        }
        self.dungeon_environment_mut()
            .increment_water_transition_counter();
        let lower = 0x0688u16
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
            .wrapping_sub(self.game_state.dungeon.environment.water_hdma_y_radius());
        self.set_spotlight_y_lower(lower);
        let upper = lower.wrapping_add(y_span);
        self.set_spotlight_y_upper(upper);
        self.AdjustWaterHDMAWindow_X(upper);
    }

    pub(super) fn Module07_0D_FloodDam(&mut self) {
        self.FloodDam_PrepFloodHDMA();
        match self.game_state.frame.subsubmodule {
            0 => self.FloodDam_PrepTiles_init(),
            1..=3 => self.Watergate_Main_State1(),
            4 => self.FloodDam_Expand(),
            5 => self.FloodDam_Fill(),
            other => panic!("invalid Module07_0D_FloodDam subsubmodule_index {other}"),
        }
    }

    pub(super) fn Module07_0E_01_HandleMusicAndResetProps(&mut self) {
        let room = self.game_state.world.location.dungeon_room();
        if (room == 7 || (room == 23 && !self.zelda_is_playing_music_track(17)))
            && self.game_state.inventory.player_resources.pendant_flags() & 1 == 0
        {
            self.set_music_control(0xf1);
        }
        let staircase_move_counter =
            if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                106
            } else {
                88
            };
        self.dungeon_stair_movement_mut()
            .set_staircase_move_counter(staircase_move_counter);
        self.set_overworld_map_state(0);
        self.ResetTransitionPropsAndAdvanceSubmodule();
    }

    pub(super) fn Module07_0E_02_ApplyFilterIf(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            < 9
        {
            self.ApplyPaletteFilter_bounce();
            if self.game_state.display.palette_filter.countdown() != 0 {
                self.ApplyPaletteFilter_bounce();
            }
        }
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            != 0
        {
            self.dungeon_stair_movement_mut()
                .decrement_staircase_move_counter();
            return;
        }
        self.follower_state_mut().set_appearance_none_flag(12);
        self.follower_link_state_mut().set_visibility_status(12);
    }

    pub(super) fn Dungeon_AdvanceThenSetBossMusicUnorthodox(&mut self) {
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        self.dungeon_stair_movement_mut()
            .set_staircase_move_counter(0x38);
        self.increment_subsubmodule();
        self.Dungeon_SetBossMusicUnorthodox();
    }

    pub(super) fn Dungeon_SetBossMusicUnorthodox(&mut self) {
        let room = self.game_state.world.location.dungeon_room();
        let mut x = 0x1c;
        if room != 16 {
            x = 0x15;
            if room != 7 {
                x = 0x11;
                if room != 23 || self.zelda_is_playing_music_track(17) {
                    return;
                }
            }
            if self.game_state.system_signals.current_music_control() != 0xf1
                && self.game_state.inventory.player_resources.pendant_flags() & 1 != 0
            {
                return;
            }
        }
        self.set_music_control(x);
    }

    pub(super) fn Module07_0E_SpiralStairs(&mut self) {
        if self.game_state.frame.subsubmodule >= 7 {
            self.Graphics_IncrementalVRAMUpload();
            self.Dungeon_LoadAttribute_Selectable();
        }
        self.HandleLinkOnSpiralStairs();
        match self.game_state.frame.subsubmodule {
            0 => self.Module07_0E_00_InitPriorityAndScreens(),
            1 => self.Module07_0E_01_HandleMusicAndResetProps(),
            2 => self.Module07_0E_02_ApplyFilterIf(),
            3 => self.Dungeon_InitializeRoomFromSpecial(),
            4 => self.DungeonTransition_TriggerBGC34UpdateAndAdvance(),
            5 => self.DungeonTransition_TriggerBGC56UpdateAndAdvance(),
            6 => self.DungeonTransition_LoadSpriteGFX(),
            7 => self.Dungeon_SyncBackgroundsFromSpiralStairs(),
            8 => self.Dungeon_InterRoomTrans_State4(),
            9 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            10 => self.Dungeon_InterRoomTrans_State4(),
            11 => self.Dungeon_SpiralStaircase11(),
            12 => self.Dungeon_SpiralStaircase12(),
            13 => self.Dungeon_SpiralStaircase11(),
            14 => self.Dungeon_SpiralStaircase12(),
            15 => self.Dungeon_DoubleApplyAndIncrementGrayscale(),
            16 => self.Dungeon_AdvanceThenSetBossMusicUnorthodox(),
            17 => self.Dungeon_SpiralStaircase17(),
            18 => self.Dungeon_SpiralStaircase18(),
            19 => self.Module07_0E_13_SetRoomAndLayerAndCache(),
            other => panic!("invalid Module07_0E_SpiralStairs subsubmodule_index {other}"),
        }
    }

    pub(super) fn Dungeon_SyncBackgroundsFromSpiralStairs(&mut self) {
        if self.game_state.sprites.follower_runtime.indicator() == 6
            && self.game_state.world.location.dungeon_room_index() == 100
        {
            self.follower_state_mut().set_indicator(0);
        }
        let bak = self.game_state.player.follower_link.lower_level_state();
        let y_delta = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            48
        } else {
            (-48i16) as u16
        };
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(y_delta);
        self.follower_link_state_mut().set_y(y);
        let lower_level_state = TELEPORT_PIT_SECONDARY_LEVELS[self
            .game_state
            .dungeon
            .stair_movement
            .current_staircase_plane()
            as usize];
        self.follower_link_state_mut()
            .set_lower_level_state(lower_level_state);
        self.SpiralStairs_MakeNearbyWallsHighPriority_Exiting();
        self.follower_link_state_mut().set_lower_level_state(bak);
        let y_delta = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            (-48i16) as u16
        } else {
            48
        };
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(y_delta);
        self.follower_link_state_mut().set_y(y);
        self.copy_bg2_live_to_bg1_live();
        self.Dungeon_AdjustForRoomLayout();
        let mut ts = SPIRAL_SUBSCREEN_LAYER_BY_BG2
            [self.game_state.dungeon.room_load.bg2_properties() as usize];
        let mut tm = 0x16;
        if ts < 0 {
            tm = 0x17;
            ts = 0;
        }
        if self.game_state.dungeon.room_load.bg2_properties() == 2 {
            ts = 3;
        }
        self.set_main_screen_layers(tm);
        self.set_sub_screen_layers(ts as u8);
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            self.dungeon_stair_movement_mut().decrement_current_floor();
        } else {
            self.dungeon_stair_movement_mut().increment_current_floor();
        }
        self.dungeon_stair_movement_mut()
            .set_staircase_move_counter(24);
        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
        self.hud_restore_torch_background();
        self.Dungeon_InterRoomTrans_notDarkRoom();
    }

    pub(super) fn Dungeon_SpiralStaircase17(&mut self) {
        self.SpiralStairs_FindLandingSpot();
        self.dungeon_stair_movement_mut()
            .decrement_staircase_move_counter();
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            == 0
        {
            let staircase_move_counter =
                if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                    10
                } else {
                    24
                };
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(staircase_move_counter);
            self.increment_subsubmodule();
        }
    }

    pub(super) fn Dungeon_SpiralStaircase18(&mut self) {
        self.SpiralStairs_FindLandingSpot();
        self.dungeon_stair_movement_mut()
            .decrement_staircase_move_counter();
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            == 0
        {
            self.increment_subsubmodule();
            self.set_overworld_map_state(0);
        }
    }

    pub(super) fn Module07_0E_00_InitPriorityAndScreens(&mut self) {
        self.SpiralStairs_MakeNearbyWallsHighPriority_Entering();
        if self.game_state.player.follower_link.is_on_lower_level() {
            self.and_main_screen_layers(0x0f);
            self.or_sub_screen_layers(0x10);
            self.follower_link_state_mut().set_lower_level_state(3);
        }
        self.increment_subsubmodule();
    }

    pub(super) fn Module07_0E_13_SetRoomAndLayerAndCache(&mut self) {
        let plane = self
            .game_state
            .dungeon
            .stair_movement
            .current_staircase_plane() as usize;
        self.follower_link_state_mut().set_lower_level_states(
            TELEPORT_PIT_SECONDARY_LEVELS[plane],
            TELEPORT_PIT_PRIMARY_LEVELS[plane],
        );
        self.or_main_screen_layers(0x10);
        self.and_sub_screen_layers(0x0f);
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 == 0 {
            self.SpiralStairs_MakeNearbyWallsLowPriority();
        }
        let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
        self.dungeon_room_tracking_mut()
            .set_room_index2(dungeon_room_index);
        self.ResetThenCacheRoomEntryProperties();
    }

    pub(super) fn RepositionLinkAfterSpiralStairs(&mut self) {
        self.follower_link_state_mut().set_visibility_status(0);
        self.follower_state_mut().set_appearance_none_flag(0);
        let mut i = if self
            .game_state
            .dungeon
            .stair_movement
            .current_staircase_plane()
            == 0
            && self
                .game_state
                .dungeon
                .stair_movement
                .staircase_lower_level_status()
                != 0
        {
            1usize
        } else {
            0usize
        };
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            i += 2;
        }
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(SPIRAL_STAIRCASE_X_OFFSETS[i] as i16 as u16);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(SPIRAL_STAIRCASE_Y_OFFSETS[i] as i16 as u16);
        self.follower_link_state_mut().set_x(x);
        self.follower_link_state_mut().set_y(y);

        if self.game_state.display.main_screen_layers & 0x10 != 0 {
            if self
                .game_state
                .dungeon
                .stair_movement
                .current_staircase_plane()
                == 2
            {
                self.follower_link_state_mut().set_lower_level_state(3);
                self.and_main_screen_layers(0x0f);
                self.or_sub_screen_layers(0x10);
                if self
                    .game_state
                    .dungeon
                    .stair_movement
                    .staircase_lower_level_status()
                    != 2
                {
                    let y = self.game_state.player.follower_link.y().wrapping_add(24);
                    self.follower_link_state_mut().set_y(y);
                }
            }
            self.follower_initialize();
        } else {
            if self
                .game_state
                .dungeon
                .stair_movement
                .current_staircase_plane()
                != 2
            {
                self.or_main_screen_layers(0x10);
                self.and_sub_screen_layers(0x0f);
                if self
                    .game_state
                    .dungeon
                    .stair_movement
                    .staircase_lower_level_status()
                    != 2
                {
                    let y = self.game_state.player.follower_link.y().wrapping_sub(24);
                    self.follower_link_state_mut().set_y(y);
                }
            }
            self.follower_initialize();
        }
    }

    pub(super) fn Dungeon_PlayMusicIfDefeated(&mut self) {
        let room = self.game_state.world.location.dungeon_room();
        let mut x = 0x14;
        if room != 18 {
            x = 0x10;
            if room != 2 {
                if !DUNGEON_BOSS_ROOMS.contains(&room) {
                    return;
                }
                if self.sprite_check_if_screen_is_clear() {
                    return;
                }
                x = 0x15;
            }
        }
        self.set_music_control(x);
    }

    pub(super) fn Dungeon_LoadCustomTileAttr(&mut self) {
        let offset = self.asset_u16(
            51,
            self.game_state.world.palette_theme.aux_tile_theme_index() as usize,
        ) as usize;
        let attrs = self.asset_raw(52).expect("missing dungeon tile attr asset");
        let custom_attrs = attrs[offset..offset + 0x80].to_vec();
        self.dungeon_room_parser_mut()
            .copy_custom_tile_attrs(&custom_attrs);
    }

    pub(super) fn SaveDungeonKeys(&mut self) {
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

    pub(super) fn Dungeon_LoadRoom(&mut self) {
        self.Dungeon_LoadHeader();
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
        // The clear loop zeroes the blast-wall flag/progress RAM bytes (0x452 FLAG_X,
        // 0x453 FLAG_Y, 0x454 CRUSH_WALL_PROGRESS, 0x456 CRUSH_WALL_DOOR_INDEX_X2) but
        // clear_room_parser_words only touches RAM + the room parser, leaving the
        // DungeonRoomEffectsState native (blast_wall_x/y_open, crush_wall_progress,
        // door_index) stale from a just-opened blast wall — it would re-stamp FLAG_Y=1
        // back over the cleared RAM. Reset the native too (same stale-native-field hazard
        // as misc_object_index / cur_door_idx below).
        self.dungeon_room_effects_mut().clear_blast_wall_state();
        self.dungeon_room_doors_mut().clear_invisible_door_marker();
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
        // 0x41a/0x422/0x424 (DungeonMovingFloorState move_flags/x_offset/y_offset) and 0x424
        // (DungeonEnvironmentState water_transition_counter) are zeroed by the
        // clear_room_parser_words loop above, but that only touches RAM + the room parser —
        // these native fields would keep last room's value and re-project it (a stale moving-
        // floor offset / water-animation counter re-stamping the cleared byte; f473650, a
        // dungeon water room kept TURN_ON_OFF_WATER_CTR=0x20). Reset them to match the clear.
        self.dungeon_moving_floor_mut().clear_floor_move_flags();
        self.dungeon_moving_floor_mut().clear_floor_offsets();
        self.dungeon_environment_mut()
            .set_water_transition_counter(0);
        for i in 0..16 {
            self.dungeon_object_tracking_mut()
                .set_object_tilemap_pos(i, 0);
        }
        self.dungeon_doors_mut().clear_door_tilemap_addresses();
        self.dungeon_doors_mut().clear_door_tables();
        self.dungeon_room_doors_mut()
            .clear_exit_door_count_and_flags();
        self.dungeon_room_load_mut().set_load_ptr_offset(0);
        self.RoomDraw_DrawFloorsCurrentRoom();
        self.dungeon_room_tilemaps_mut()
            .copy_line_pointer_bytes(&DUNGEON_DRAW_OBJECT_OFFSETS_BG1);
        self.RoomDraw_DrawAllObjectsCurrentRoom();
        self.dungeon_torch_mut().refresh_object_data_positions();
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
                self.DrawObjects_PushableBlock(tilemap, offset as u16);
            }
        }

        let misc_objs = self.game_state.dungeon.object_tracking.misc_object_index();
        self.dungeon_torch_mut()
            .set_torch_index_range_start(misc_objs);
        self.dungeon_torch_mut().set_torch_index(misc_objs);
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
                    self.DrawObjects_LightableTorch(t, (i - 2) as u16);
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

    fn dungeon_load_room_reset_floor_velocity(&mut self) {
        self.dungeon_moving_floor_mut().set_floor_y_velocity(0);
        self.dungeon_moving_floor_mut().set_floor_x_velocity(0);
    }

    pub(super) fn Dungeon_LoadHeader(&mut self) {
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
        } else if submodule == 21 || (submodule < 18 && submodule >= 6) {
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

        let saved = self.loaded_room_data_word(0xf000, room);
        self.dungeon_doors_mut().set_opened_doors(saved & 0xf000);
        self.dungeon_doors_mut()
            .set_opened_doors_including_adjacent((saved & 0xf000) | 0x0f00);
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits((saved & 0x0ff0) << 4);
        self.dungeon_room_load_mut()
            .set_quadrants_visited(saved & 0x000f);

        self.load_current_room_door_tilemap_addresses(room);
        if (room.wrapping_sub(1) & 0x0f) != 0x0f {
            self.Dungeon_CheckAdjacentRoomsForOpenDoors(18, room.wrapping_sub(1));
        }
        if (room.wrapping_add(1) & 0x0f) != 0 {
            self.Dungeon_CheckAdjacentRoomsForOpenDoors(12, room.wrapping_add(1));
        }
        if room >= 16 {
            self.Dungeon_CheckAdjacentRoomsForOpenDoors(6, room - 16);
        }
        if room + 16 < 0x140 {
            self.Dungeon_CheckAdjacentRoomsForOpenDoors(0, room + 16);
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

    fn Dungeon_CheckAdjacentRoomsForOpenDoors(&mut self, idx: usize, room: usize) {
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
        let flags = (self.loaded_room_data_word(0xf000, room) & 0xf000) | 0x0f00;
        self.dungeon_room_doors_mut().set_adjacent_door_flags(flags);
        let Some(doors) = self.GetRoomDoorInfo(room).map(Vec::from) else {
            self.dungeon_room_doors_mut().mark_no_adjacent_doors();
            return;
        };
        self.dungeon_room_doors_mut()
            .load_adjacent_doors_from_room_info(&doors);
    }

    pub(super) fn RoomDraw_DrawFloorsCurrentRoom(&mut self) {
        let room = self.game_state.world.location.dungeon_room() as usize;
        let Some(room_layout) = self.dungeon_room_layout(room).map(Vec::from) else {
            return;
        };
        if room_layout.is_empty() {
            return;
        }
        self.RoomDraw_DrawFloors(&room_layout);
    }

    pub(super) fn RoomDraw_DrawFloors(&mut self, level_data: &[u8]) {
        let offs = self.game_state.dungeon.room_load.load_ptr_offset() as usize;
        let floor_types = level_data.get(offs).copied().unwrap_or(0);
        self.dungeon_room_tilemaps_mut()
            .copy_bg2_draw_line_offsets();
        self.dungeon_room_parser_mut()
            .set_floor_1_filler_low(floor_types & 0xf0);
        self.dungeon_room_parser_mut().set_floor_1_filler_high(0);
        self.RoomDraw_FloorChunks(0x4000, (floor_types & 0xf0) as usize);

        self.dungeon_room_tilemaps_mut()
            .copy_bg1_draw_line_offsets();
        self.dungeon_room_parser_mut()
            .set_floor_2_filler_low((floor_types & 0x0f) << 4);
        self.dungeon_room_parser_mut().set_floor_2_filler_high(0);
        self.RoomDraw_FloorChunks(0x2000, ((floor_types & 0x0f) << 4) as usize);
        self.dungeon_room_load_mut().set_load_ptr_offset(1);
    }

    pub(super) fn RoomDraw_DrawAllObjectsCurrentRoom(&mut self) {
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
            self.RoomData_DrawObjects_from(&default_layout);
        }

        self.dungeon_room_load_mut()
            .set_load_ptr_offset(old_offs.saturating_add(1) as u16);
        self.RoomData_DrawObjects_from(&room_layout);
        let pos = self
            .game_state
            .dungeon
            .room_load
            .load_ptr_offset()
            .wrapping_add(2);
        self.dungeon_room_load_mut().set_load_ptr_offset(pos);
        self.dungeon_room_tilemaps_mut()
            .copy_bg2_draw_line_offsets();
        self.RoomData_DrawObjects_from(&room_layout);
        let pos = self
            .game_state
            .dungeon
            .room_load
            .load_ptr_offset()
            .wrapping_add(2);
        self.dungeon_room_load_mut().set_load_ptr_offset(pos);
        self.dungeon_room_tilemaps_mut()
            .copy_bg1_draw_line_offsets();
        self.RoomData_DrawObjects_from(&room_layout);
        self.dungeon_room_load_mut().set_load_ptr_offset(0x0120);
    }

    pub(super) fn RoomDraw_DrawAllObjects(&mut self, level_data: &[u8]) {
        self.RoomData_DrawObjects_from(level_data);
    }

    pub(super) fn RoomData_DrawObjects_from(&mut self, layout: &[u8]) {
        loop {
            self.dungeon_room_load_mut()
                .set_draw_width_indicator_word(0);
            self.dungeon_room_load_mut()
                .set_draw_height_indicator_word(0);
            let pos = self.game_state.dungeon.room_load.load_ptr_offset() as usize;
            let raw = read_word_from_slice(layout, pos);
            if raw == 0xffff {
                return;
            }
            if raw == 0xfff0 {
                break;
            }
            let idx = layout[pos + 2];
            self.dungeon_room_load_mut()
                .set_load_ptr_offset(pos.wrapping_add(3) as u16);
            self.RoomData_DrawObject(raw, idx);
        }
        loop {
            let pos = self
                .game_state
                .dungeon
                .room_load
                .load_ptr_offset()
                .wrapping_add(2) as usize;
            self.dungeon_room_load_mut().set_load_ptr_offset(pos as u16);
            let raw = read_word_from_slice(layout, pos);
            if raw == 0xffff {
                return;
            }
            self.RoomData_DrawObject_Door(raw);
        }
    }

    pub(super) fn RoomData_DrawObject(&mut self, raw: u16, idx: u8) {
        if raw & 0xfc != 0xfc {
            let width = (raw & 3) as u8;
            let height = ((raw >> 8) & 3) as u8;
            self.dungeon_room_load_mut()
                .set_draw_dimensions_words(u16::from(width), u16::from(height));
            let x = (raw as u8 >> 2) as u16;
            let y = raw >> 10;
            let dsto = y * 64 + x;
            if idx < 0xf8 {
                self.LoadType1ObjectSubtype1(idx, width, height, dsto);
            } else {
                let object = ((idx & 7) << 4) | (((raw >> 8) as u8 & 3) << 2) | (raw as u8 & 3);
                let mut dst = 0;
                self.LoadType1ObjectSubtype3(object, &mut dst, dsto);
            }
        } else {
            let x = ((raw & 3) << 4) | ((raw >> 12) & 0x0f);
            let y = (((raw >> 8) & 0x0f) << 2) | ((idx as u16) >> 6);
            let dsto = y * 64 + x;
            let mut dst = 0;
            self.LoadType1ObjectSubtype2(idx & 0x3f, &mut dst, dsto);
        }
    }

    pub(super) fn LoadType1ObjectSubtype1(&mut self, idx: u8, width: u8, height: u8, dsto: u16) {
        let Some(src) = object_subtype1_param(idx) else {
            panic!("LoadType1ObjectSubtype1 invalid object id {idx:#04x}");
        };
        match idx {
            0x00 | 0xb8 | 0xb9 => {
                let count = size_1to15_or(width, height, 32);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 2);
                }
            }
            0x01 | 0x02 | 0xb6 | 0xb7 => {
                let count = size_1to15_or(width, height, 26);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 2, 2);
                }
            }
            0x03 | 0x04 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4_both_bgs(src, dsto + i * 2, 2);
                }
            }
            0x05 | 0x06 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 6, 2);
                }
            }
            0x07 | 0x08 | 0x53 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 2);
                }
            }
            0x09 | 0x0c | 0x0d | 0x10 | 0x11 | 0x14 => {
                let count = size_a_to_a_plus_15(width, height, 6);
                let mut dst = dsto;
                for _ in 0..count {
                    self.RoomDraw_DrawObject2x2and1(src, dst);
                    dst = dst.wrapping_sub(63);
                }
            }
            0x0a | 0x0b | 0x0e | 0x0f | 0x12 | 0x13 => {
                let count = size_a_to_a_plus_15(width, height, 6);
                let mut dst = dsto;
                for _ in 0..count {
                    self.RoomDraw_DrawObject2x2and1(src, dst);
                    dst = dst.wrapping_add(65);
                }
            }
            0x15 | 0x18 | 0x19 | 0x1c | 0x1d | 0x20 => {
                let count = size_a_to_a_plus_15(width, height, 6);
                let mut dst = dsto;
                for _ in 0..count {
                    for y in 0..5 {
                        self.room_write_bg2(dst + y * 64, self.tile_word(src, y as usize));
                        self.room_write_bg1(dst + y * 64, self.tile_word(src, y as usize));
                    }
                    dst = dst.wrapping_sub(63);
                }
            }
            0x16 | 0x17 | 0x1a | 0x1b | 0x1e | 0x1f => {
                let count = size_a_to_a_plus_15(width, height, 6);
                let mut dst = dsto;
                for _ in 0..count {
                    for y in 0..5 {
                        self.room_write_bg2(dst + y * 64, self.tile_word(src, y as usize));
                        self.room_write_bg1(dst + y * 64, self.tile_word(src, y as usize));
                    }
                    dst = dst.wrapping_add(65);
                }
            }
            0x21 => {
                let mut count = (((width as u16) << 2) | height as u16) * 2 + 1;
                let mut dst = dsto;
                self.RoomDraw_1x3_rightwards(src, dst, 2);
                dst += 2;
                while count != 0 {
                    self.RoomDraw_1x3_rightwards(src + 6, dst, 1);
                    dst += 1;
                    count -= 1;
                }
                self.RoomDraw_1x3_rightwards(src + 12, dst, 1);
            }
            0x22 => {
                let count = size_a_to_a_plus_15(width, height, 2);
                if self.room_read_current(dsto) & 0x03ff != 0x00e2 {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                let tile = self.tile_word(src, 1);
                for i in 1..=count {
                    self.room_write_current(dsto + i, tile);
                }
                self.room_write_current(dsto + count + 1, self.tile_word(src, 2));
            }
            0x23..=0x2e | 0x3f..=0x46 | 0xb3 | 0xb4 => {
                let count = size_1to16(width, height);
                let tile = self.room_read_current(dsto) & 0x03ff;
                if tile != 0x01db && tile != 0x01a6 && tile != 0x01dd && tile != 0x01fc {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                let fill = self.tile_word(src, 1);
                for i in 1..=count {
                    self.room_write_current(dsto + i, fill);
                }
                self.room_write_current(dsto + count + 1, self.tile_word(src, 2));
            }
            0x2f => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst) & 0x03ff != 0x00e2 {
                    self.room_write_current(dst, self.tile_word(src, 1));
                    self.room_write_current(dst + 1, self.tile_word(src, 2));
                    self.room_write_current(dst + 64, fill);
                    self.room_write_current(dst + 65, fill);
                    dst += 2;
                }
                for _ in 0..count {
                    self.room_write_current(dst, self.tile_word(src, 3));
                    self.room_write_current(dst + 64, fill);
                    dst += 1;
                }
                self.room_write_current(dst, self.tile_word(src, 4));
                self.room_write_current(dst + 1, self.tile_word(src, 5));
                self.room_write_current(dst + 64, fill);
                self.room_write_current(dst + 65, fill);
            }
            0x30 => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst + 64) & 0x03ff != 0x00e2 {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 1, fill);
                    self.room_write_current(dst + 64, self.tile_word(src, 1));
                    self.room_write_current(dst + 65, self.tile_word(src, 2));
                    dst += 2;
                }
                for _ in 0..count {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 64, self.tile_word(src, 3));
                    dst += 1;
                }
                self.room_write_current(dst, fill);
                self.room_write_current(dst + 1, fill);
                self.room_write_current(dst + 64, self.tile_word(src, 4));
                self.room_write_current(dst + 65, self.tile_word(src, 5));
            }
            0x33 | 0xb2 | 0xba => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_4x4(src, dsto + i * 4);
                }
            }
            0x34 => {
                let count = size_a_to_a_plus_15(width, height, 4);
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i, tile);
                }
            }
            0x36 | 0x37 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_4x4(src, dsto + i * 6);
                }
            }
            0x38 => {
                let statue_src = 0x0e26;
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_1x3_rightwards(statue_src, dsto + i * 4, 2);
                }
            }
            0x39 | 0x3d => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 6, 2);
                }
            }
            0x3a | 0x3b => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_1x3_rightwards(src, dsto + i * 8, 4);
                }
            }
            0x3c => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    let dst = dsto + i * 4;
                    self.RoomDraw_Rightwards2x2(src, dst);
                    self.RoomDraw_Rightwards2x2(src + 8, dst + 6 * 64);
                }
            }
            0x3e | 0x4b => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 14);
                }
            }
            0x47 => {
                let count = size_1to16(width, height) * 2;
                let mut dst = self.RoomDraw_DrawObject2x2and1(src, dsto) + 1;
                for _ in 0..count {
                    self.RoomDraw_DrawObject2x2and1(src + 10, dst);
                    dst += 1;
                }
                self.RoomDraw_DrawObject2x2and1(src + 20, dst);
            }
            0x48 => {
                let count = size_1to16(width, height) * 2;
                let mut dst = dsto;
                self.RoomDraw_1x3_rightwards(src, dst, 1);
                dst += 1;
                for _ in 0..count {
                    self.room_write_current(dst, self.tile_word(src, 3));
                    self.room_write_current(dst + 64, self.tile_word(src, 4));
                    self.room_write_current(dst + 128, self.tile_word(src, 5));
                    dst += 1;
                }
                self.RoomDraw_1x3_rightwards(src + 12, dst, 1);
            }
            0x49 | 0x4a => {
                let count = size_1to16(width, height);
                self.RoomDraw_Downwards4x2VariableSpacing(4, src, dsto, count);
            }
            0x4c => {
                let count = size_1to16(width, height) * 2;
                let mut dst = self.RoomDraw_RightwardBarSegment(src, dsto) + 1;
                for _ in 0..count {
                    dst = self.RoomDraw_RightwardBarSegment(src + 6, dst) + 1;
                }
                self.RoomDraw_RightwardBarSegment(src + 12, dst);
            }
            0x4d..=0x4f => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                self.RoomData_DrawObject_nx4(src, dst, 1);
                dst += 1;
                for _ in 0..count {
                    self.RoomData_DrawObject_nx4(src + 8, dst, 2);
                    dst += 2;
                }
                self.RoomDraw_RightwardShelfEnd(src + 24, &mut dst);
            }
            0x50 => {
                let count = size_a_to_a_plus_15(width, height, 2);
                self.Object_Fill_Nx1(count, src, dsto);
            }
            0x51 | 0x52 | 0x5b | 0x5c => {
                let mut count = size_1to16(width, height);
                let mut dst = dsto;
                self.RoomDraw_1x3_rightwards(src, dst, 2);
                dst += 2;
                while count > 1 {
                    self.RoomDraw_1x3_rightwards(src + 12, dst, 2);
                    dst += 2;
                    count -= 1;
                }
                self.RoomDraw_1x3_rightwards(src + 24, dst, 2);
            }
            0x55 | 0x56 => {
                let count = size_1to16(width, height);
                self.RoomDraw_Downwards4x2VariableSpacing(12, src, dsto, count);
            }
            0x5d => {
                let count = size_1to16(width, height) + 1;
                let mut dst = dsto;
                self.RoomDraw_1x3_rightwards(src, dst, 2);
                dst += 2;
                for _ in 0..count {
                    self.RoomDraw_RightwardBarSegment(src + 12, dst);
                    dst += 1;
                }
                self.RoomDraw_1x3_rightwards(src + 18, dst, 2);
            }
            0x5e | 0xbb => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 4);
                }
            }
            0x5f => {
                let count = size_a_to_a_plus_15(width, height, 21);
                if self.room_read_current(dsto) & 0x03ff != 0x00e2 {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                let fill = self.tile_word(src, 1);
                for i in 1..=count {
                    self.room_write_current(dsto + i, fill);
                }
                self.room_write_current(dsto + count + 1, self.tile_word(src, 2));
            }
            0x60 | 0x92 | 0x93 => {
                let count = size_1to15_or(width, height, 32);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 64 * 2);
                }
            }
            0x61 | 0x62 | 0x90 | 0x91 => {
                let count = size_1to15_or(width, height, 26);
                if replay_room_write_trace_enabled() {
                    eprintln!(
                        "room-object idx=0x{idx:02x} src=0x{src:04x} dsto=0x{dsto:04x} count=0x{count:04x} branch=wall-vert-ud"
                    );
                }
                self.RoomDraw_Downwards4x2VariableSpacing(2 * 64, src, dsto, count);
            }
            0x63 | 0x64 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.Object_Draw_4x2_BothBgs(src, dsto + i * 2 * 64);
                }
            }
            0x65 | 0x66 => {
                let count = size_1to16(width, height);
                self.RoomDraw_Downwards4x2VariableSpacing(6 * 64, src, dsto, count);
            }
            0x67 | 0x68 | 0x7d => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 2 * 64);
                }
            }
            0x7c => {
                let count = size_1to16(width, height) + 1;
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i * 64, tile);
                }
            }
            0x7f | 0x80 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 12 * 64, 2);
                }
            }
            0x81..=0x84 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 6 * 64, 3);
                }
            }
            0x85 | 0x86 => {
                let mut count = size_1to16(width, height);
                let mut dst = dsto;
                self.Object_Draw_3x2(src, dst);
                dst = dst.wrapping_add(2 * 64);
                while {
                    count -= 1;
                    count != 0
                } {
                    self.Object_Draw_3x2(src + 12, dst);
                    dst = dst.wrapping_add(2 * 64);
                }
                self.Object_Draw_3x2(src + 24, dst);
            }
            0x69 | 0x71 => {
                let count = size_a_to_a_plus_15(width, height, if idx == 0x69 { 2 } else { 4 });
                if idx == 0x69 {
                    if self.room_read_current(dsto) & 0x03ff != 0x00e3 {
                        self.room_write_current(dsto, self.tile_word(src, 0));
                    }
                    let tile = self.tile_word(src, 1);
                    for i in 1..=count {
                        self.room_write_current(dsto + i * 64, tile);
                    }
                    self.room_write_current(dsto + (count + 1) * 64, self.tile_word(src, 2));
                } else {
                    let tile = self.tile_word(src, 0);
                    for i in 0..count {
                        self.room_write_current(dsto + i * 64, tile);
                    }
                }
            }
            0x6a | 0x6b | 0x79 | 0x7a | 0x8d | 0x8e => {
                let count = size_1to16(width, height);
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i * 64, tile);
                }
            }
            0x6c => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst) & 0x03ff != 0x00e3 {
                    self.room_write_current(dst, self.tile_word(src, 1));
                    self.room_write_current(dst + 64, self.tile_word(src, 2));
                    self.room_write_current(dst + 1, fill);
                    self.room_write_current(dst + 65, fill);
                    dst += 128;
                }
                for _ in 0..count {
                    self.room_write_current(dst, self.tile_word(src, 3));
                    self.room_write_current(dst + 1, fill);
                    dst += 64;
                }
                self.room_write_current(dst, self.tile_word(src, 4));
                self.room_write_current(dst + 64, self.tile_word(src, 5));
                self.room_write_current(dst + 1, fill);
                self.room_write_current(dst + 65, fill);
            }
            0x6d => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst + 1) & 0x03ff != 0x00e3 {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 64, fill);
                    self.room_write_current(dst + 1, self.tile_word(src, 1));
                    self.room_write_current(dst + 65, self.tile_word(src, 2));
                    dst += 128;
                }
                for _ in 0..count {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 1, self.tile_word(src, 3));
                    dst += 64;
                }
                self.room_write_current(dst, fill);
                self.room_write_current(dst + 64, fill);
                self.room_write_current(dst + 1, self.tile_word(src, 4));
                self.room_write_current(dst + 65, self.tile_word(src, 5));
            }
            0x70 | 0x94 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_4x4(src, dsto + i * 4 * 64);
                }
            }
            0x73 | 0x74 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_4x4(src, dsto + i * 6 * 64);
                }
            }
            0x75 | 0x87 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 6 * 64, 2);
                }
            }
            0x76 | 0x77 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 8 * 64, 3);
                }
            }
            0x78 | 0x7b => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 14 * 64);
                }
            }
            0x89 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 4 * 64);
                }
            }
            0x8a => {
                let count = size_a_to_a_plus_15(width, height, 21);
                if self.room_read_current(dsto) & 0x03ff != 0x00e3 {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                let tile = self.tile_word(src, 1);
                for i in 1..=count {
                    self.room_write_current(dsto + i * 64, tile);
                }
                self.room_write_current(dsto + (count + 1) * 64, self.tile_word(src, 2));
            }
            0x88 => {
                let mut count = size_1to16(width, height);
                let mut dst = dsto;
                self.RoomDraw_Rightwards2x2(src, dst);
                dst = dst.wrapping_add(2 * 64);
                let src = src + 8;
                while count != 0 {
                    self.room_write_current(dst, self.tile_word(src, 0));
                    self.room_write_current(dst + 1, self.tile_word(src, 1));
                    dst = dst.wrapping_add(64);
                    count -= 1;
                }
                self.RoomDraw_1x3_rightwards(src + 4, dst, 2);
            }
            0x8b | 0x8c => {
                let count = size_a_to_a_plus_15(width, height, 8);
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i * 64, tile);
                }
            }
            0x8f => {
                let count = size_a_to_a_plus_15(width, height, 2) * 2;
                let mut dst = dsto;
                self.room_write_current(dst, self.tile_word(src, 0));
                self.room_write_current(dst + 1, self.tile_word(src, 1));
                for _ in 0..count {
                    self.room_write_current(dst + 64, self.tile_word(src, 2));
                    self.room_write_current(dst + 65, self.tile_word(src, 3));
                    dst = dst.wrapping_add(64);
                }
            }
            0x95 => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                let mut pos = dsto;
                for _ in 0..count {
                    self.RoomDraw_SinglePot(src, &mut dst, pos);
                    dst = dst.wrapping_add(2 * 64);
                    pos = pos.wrapping_add(2 * 64);
                }
            }
            0x96 => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                let mut pos = dsto;
                for _ in 0..count {
                    self.RoomDraw_HammerPegSingle(src, &mut dst, pos);
                    dst = dst.wrapping_add(2 * 64);
                    pos = pos.wrapping_add(2 * 64);
                }
            }
            0xa0 | 0xa5 | 0xa9 => {
                let mut count = size_a_to_a_plus_15(width, height, 4);
                let mut dst = dsto;
                while count != 0 {
                    self.room_fill_horizontal(dst, count, self.tile_word(src, 0));
                    dst = dst.wrapping_add(64);
                    count -= 1;
                }
            }
            0xa1 | 0xa6 | 0xaa => {
                let count = size_a_to_a_plus_15(width, height, 4);
                for y in 0..count {
                    self.room_fill_horizontal(dsto + y * 64, y + 1, self.tile_word(src, 0));
                }
            }
            0xa2 | 0xa7 | 0xab => {
                let mut count = size_a_to_a_plus_15(width, height, 4);
                let mut dst = dsto;
                while count != 0 {
                    self.room_fill_horizontal(dst, count, self.tile_word(src, 0));
                    dst = dst.wrapping_add(65);
                    count -= 1;
                }
            }
            0xa3 | 0xa8 | 0xac => {
                let mut count = size_a_to_a_plus_15(width, height, 4);
                let mut dst = dsto;
                while count != 0 {
                    self.room_fill_horizontal(dst, count, self.tile_word(src, 0));
                    dst = dst.wrapping_sub(63);
                    count -= 1;
                }
            }
            0xa4 => self.Object_Hole(src, dsto, width, height),
            0xb0 | 0xb1 => {
                let count = size_a_to_a_plus_15(width, height, 8);
                self.Object_Fill_Nx1(count, src, dsto);
            }
            0xb5 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(0x0b16, dsto + i * 2, 2);
                }
            }
            0xbc => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                let mut pos = dsto;
                for _ in 0..count {
                    self.RoomDraw_SinglePot(src, &mut dst, pos);
                    dst = dst.wrapping_add(2);
                    pos = pos.wrapping_add(2);
                }
            }
            0xbd => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                let mut pos = dsto;
                for _ in 0..count {
                    self.RoomDraw_HammerPegSingle(src, &mut dst, pos);
                    dst = dst.wrapping_add(2);
                    pos = pos.wrapping_add(2);
                }
            }
            0xc5..=0xca | 0xd1 | 0xd2 | 0xd9 | 0xdf..=0xe8 => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                for y in 0..count_y {
                    let mut dst = dsto + y * 4 * 64;
                    for _ in 0..count_x {
                        self.RoomDraw_A_Many32x32Blocks(1, src, &mut dst);
                    }
                }
            }
            0xc3 | 0xd7 => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                let tile = self.tile_word(src, 0);
                for y in 0..count_y {
                    for x in 0..count_x {
                        self.room_fill_rect(dsto + y * 3 * 64 + x * 3, 3, 3, tile);
                    }
                }
            }
            0xd8 => {
                let count_x = width as u16 + 2;
                let count_y = height as u16 + 2;
                self.dungeon_environment_mut()
                    .set_water_hdma_x_radius(count_x << 4);
                self.dungeon_environment_mut()
                    .set_water_hdma_y_radius(count_y << 4);
                self.dungeon_environment_mut()
                    .set_water_hdma_y_target((count_y << 4).wrapping_sub(24));
                let hdma0 = ((dsto & 0x003f) << 3)
                    .wrapping_add(count_x << 4)
                    .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_h());
                let hdma1 = ((dsto & 0x0fc0) >> 3)
                    .wrapping_add(count_y << 4)
                    .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_v());
                self.dungeon_environment_mut()
                    .set_water_window_position(hdma0, hdma1);
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0800 != 0 {
                    self.dungeon_header_mut().clear_header_tag(1);
                    self.dungeon_room_load_mut().clear_bg2_properties();
                    self.dungeon_stair_lists_mut()
                        .promote_water_stairs_to_active();
                    let water_dsto = dsto
                        .wrapping_add((count_x - 1) << 1)
                        .wrapping_add((count_y - 1) << 7);
                    self.DrawWaterThingBg(0x4000, water_dsto, 0x1438);
                } else {
                    let mut dst = dsto;
                    for _ in 0..count_y {
                        let mut row = dst;
                        self.RoomDraw_A_Many32x32Blocks(count_x as i32, 0x0110, &mut row);
                        dst = dst.wrapping_add(4 * 64);
                    }
                }
            }
            0xda => {
                let count_x = width as u16 + 2;
                let count_y = height as u16 + 2;
                self.dungeon_environment_mut()
                    .set_water_hdma_x_radius((count_x << 4).wrapping_sub(24));
                self.dungeon_environment_mut()
                    .set_water_hdma_y_target((count_y << 4).wrapping_sub(8));
                self.dungeon_environment_mut()
                    .set_water_hdma_y_radius((count_y << 4).wrapping_sub(32));
                self.dungeon_environment_mut()
                    .set_water_hdma_y_radius_alt(0);
                let hdma0 = ((dsto & 0x003f) << 3)
                    .wrapping_add(count_x << 4)
                    .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_h());
                let hdma1 = ((dsto & 0x0fc0) >> 3)
                    .wrapping_add(count_y << 4)
                    .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_v())
                    .wrapping_sub(8);
                self.dungeon_environment_mut()
                    .set_water_window_position(hdma0, hdma1);
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0800 != 0 {
                    self.dungeon_header_mut().clear_header_tag(1);
                } else {
                    self.dungeon_room_load_mut().clear_bg2_properties();
                    self.dungeon_stair_lists_mut()
                        .promote_water_stairs_to_active();
                }
                let mut dst = dsto;
                for _ in 0..(count_y * 2 - 1) {
                    let row = dst;
                    for _ in 0..count_x {
                        for y in 0..2 {
                            for x in 0..4 {
                                self.room_write_current(
                                    dst + y * 64 + x,
                                    self.tile_word(0x0110, (y * 4 + x) as usize),
                                );
                            }
                        }
                        dst = dst.wrapping_add(4);
                    }
                    dst = row.wrapping_add(2 * 64);
                }
            }
            0xc4 => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                let src = self
                    .game_state
                    .dungeon
                    .room_parser
                    .floor_2_filler_tile_source();
                for y in 0..count_y {
                    let mut dst = dsto + y * 4 * 64;
                    self.RoomDraw_A_Many32x32Blocks(count_x as i32, src, &mut dst);
                }
            }
            0xdb => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                let src = self
                    .game_state
                    .dungeon
                    .room_parser
                    .floor_1_filler_tile_source();
                for y in 0..count_y {
                    let mut dst = dsto + y * 4 * 64;
                    self.RoomDraw_A_Many32x32Blocks(count_x as i32, src, &mut dst);
                }
            }
            0xc0 | 0xc2 => {
                let tile = self.tile_word(src, 0);
                for y in 0..=height as u16 {
                    for x in 0..=width as u16 {
                        self.room_fill_rect(dsto + y * 4 * 64 + x * 4, 4, 4, tile);
                    }
                }
            }
            0xc1 => {
                let width = width as u16 + 4;
                let height = height as u16 + 1;
                let mut src = src;
                let mut dst = dsto;
                self.RoomDraw_1x3_rightwards(src, dst, 3);
                src += 18;
                dst += 3;
                for _ in 0..width {
                    self.RoomDraw_1x3_rightwards(src, dst, 2);
                    dst += 2;
                }
                self.RoomDraw_1x3_rightwards(src + 12, dst, 3);
                src += 30;

                dst = dsto + 3 * 64;
                for _ in 0..height {
                    let mut row = dst;
                    self.Object_Draw_3x2(src, row);
                    row += 3;
                    for _ in 0..width {
                        self.RoomDraw_Rightwards2x2(src + 12, row);
                        row += 2;
                    }
                    self.Object_Draw_3x2(src + 20, row);
                    dst += 2 * 64;
                }

                let bottom_start = dst;
                src += 32;
                self.RoomDraw_1x3_rightwards(src, dst, 3);
                src += 18;
                dst += 3;
                for _ in 0..width {
                    self.RoomDraw_1x3_rightwards(src, dst, 2);
                    dst += 2;
                }
                self.RoomDraw_1x3_rightwards(src + 12, dst, 3);

                self.RoomDraw_Rightwards2x2(
                    0x0590,
                    bottom_start
                        .wrapping_add(width + 2)
                        .wrapping_sub((height + 1) * 64),
                );
            }
            0xcd => self.RoomDraw_MovingWallRight(width, height, dsto),
            0xce => self.RoomDraw_MovingWallLeft(width, height, dsto),
            0x31
            | 0x32
            | 0x35
            | 0x54
            | 0x57..=0x5a
            | 0x6e
            | 0x6f
            | 0x72
            | 0x7e
            | 0x97..=0x9f
            | 0xad..=0xaf
            | 0xbe
            | 0xbf => {}
            0xdd => {
                let width = width as u16 + 1;
                let height = height as u16 * 2 + 2;
                self.Object_Table_Helper(src, dsto, width);
                for y in 1..height {
                    self.Object_Table_Helper(src + 8, dsto + y * 64, width);
                }
                self.Object_Table_Helper(src + 16, dsto + height * 64, width);
                self.Object_Table_Helper(src + 24, dsto + (height + 1) * 64, width);
            }
            0xdc => {
                let mut dst = dsto
                    | if self
                        .game_state
                        .dungeon
                        .room_tilemaps
                        .first_line_pointer_row0()
                        == 0x4000
                    {
                        0x1000
                    } else {
                        0
                    };
                let width = width as u16 + 1;
                let height = height as u16 * 2 + 5;
                for _ in 0..height {
                    self.RoomDraw_Chest_platform_row(0x0ab4, dst, width);
                    dst += 64;
                }
                self.RoomDraw_Chest_platform_row(0x0ab4 + 2, dst, width);
                dst += 64;
                self.RoomDraw_Chest_platform_row(0x0ab4 + 4, dst, width);
            }
            0xde => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                for y in 0..count_y {
                    for x in 0..count_x {
                        self.RoomDraw_Rightwards2x2(src, dsto + y * 2 * 64 + x * 2);
                    }
                }
            }
            _ => panic!("LoadType1ObjectSubtype1 unhandled object id {idx:#04x}"),
        }
    }

    pub(super) fn LoadType1ObjectSubtype3(&mut self, idx: u8, _dst: &mut u16, dsto: u16) {
        let Some(src) = object_subtype3_param(idx) else {
            panic!("LoadType1ObjectSubtype3 invalid object id {idx:#04x}");
        };
        match idx {
            0x00 => {
                if self.game_state.dungeon.header.header_tag(1) == 27 {
                    let room = self.game_state.world.location.dungeon_room() as usize;
                    if self
                        .game_state
                        .inventory
                        .save_progress
                        .dungeon_info_word(room)
                        & 0x0100
                        != 0
                    {
                        self.RoomDraw_WaterHoldingObject(5, 0x162c, dsto);
                        return;
                    }
                } else if self.game_state.dungeon.header.header_tag(1) == 25
                    && self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0800 != 0
                {
                    self.RoomDraw_WaterHoldingObject(5, 0x162c, dsto);
                    return;
                }
                self.dungeon_room_items_mut()
                    .set_replacement_tile_source_x2(dsto * 2);
                self.RoomDraw_WaterHoldingObject(3, src, dsto);
            }
            0x01 => self.RoomDraw_WaterHoldingObject(5, 0x162c, dsto),
            0x02 => self.RoomDraw_WaterHoldingObject(7, src, dsto),
            0x03 | 0x0e => {
                self.dungeon_environment_mut()
                    .increment_somaria_block_bg_check_flag();
                self.room_write_current(dsto, self.tile_word(src, 0));
            }
            0x04..=0x0c | 0x0f => {
                self.room_write_current(dsto, self.tile_word(src, 0));
            }
            0x0d | 0x17 => self.RoomDraw_PrisonCell(dsto),
            0x10
            | 0x11
            | 0x13
            | 0x1a
            | 0x22..=0x25
            | 0x3e..=0x46
            | 0x49
            | 0x4a
            | 0x4f..=0x53
            | 0x56..=0x59
            | 0x5e
            | 0x5f
            | 0x63..=0x65
            | 0x75
            | 0x7c..=0x7e => {
                self.RoomDraw_Rightwards2x2(src, dsto);
            }
            0x12 => {
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x1000 == 0 {
                    let dst = dsto | self.room_plane_offset();
                    let src = 0x1dd6;
                    for i in 0..3 {
                        let col = i * 2;
                        self.room_write_bg2(dst + col, self.tile_word(src, 0));
                        self.room_write_bg2(dst + col + 3 * 64, self.tile_word(src, 0));
                        self.room_write_bg2(dst + col + 6 * 64, self.tile_word(src, 0));
                        self.room_write_bg2(dst + col + 64, self.tile_word(src, 1));
                        self.room_write_bg2(dst + col + 4 * 64, self.tile_word(src, 1));
                        self.room_write_bg2(dst + col + 7 * 64, self.tile_word(src, 1));
                    }
                }
            }
            0x14 | 0x4e | 0x67 | 0x68 | 0x6c | 0x6d | 0x79 => {
                self.RoomDraw_1x3_rightwards(src, dsto, 4);
            }
            0x15 => {
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 == 0 {
                    self.RoomDraw_SomeBigDecors(10, 0x1dfa, dsto);
                }
            }
            0x16 => {
                let mut dst = dsto;
                self.RoomDraw_HammerPegSingle(src, &mut dst, dsto);
            }
            0x18 => self.RoomDraw_CellLock(dsto),
            0x19 => self.RoomDraw_Chest(dsto),
            0x1b => {
                self.dungeon_stair_lists_mut()
                    .append_stair_table_position(DungeonStairList::Stairs1, dsto);
                self.Object_DrawNx4_BothBgs(4, src, dsto);
            }
            0x1c => {
                self.dungeon_stair_lists_mut()
                    .append_stair_table_position(DungeonStairList::Stairs2, dsto);
                self.Object_DrawNx4_BothBgs(4, src, dsto);
            }
            0x1d => {
                self.dungeon_stair_lists_mut()
                    .append_stair_table_position(DungeonStairList::WetStairs, dsto);
                self.RoomDraw_4x4(src, dsto);
            }
            0x1e => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomUpNorthStraight, dsto);
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
                self.RoomDraw_Object_Nx4(4, src, dsto);
            }
            0x1f => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomDownNorthStraight, dsto);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.RoomDraw_Object_Nx4(4, src, dsto);
            }
            0x20 => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomUpSouthStraight, dsto);
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
                self.RoomDraw_Object_Nx4(4, src, dsto);
            }
            0x21 => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomDownSouthStraight, dsto);
                self.dungeon_stair_lists_mut()
                    .set_stair_list_count(DungeonStairList::InterRoomDownSouthStraight, next);
                self.RoomDraw_Object_Nx4(4, src, dsto);
            }
            0x26 => self.RoomDraw_LowerDoorStairsUp(src, dsto, true),
            0x27 => self.RoomDraw_LowerDoorStairsUp(src, dsto, false),
            0x28 => self.RoomDraw_LowerDoorStairsDown(src, dsto, true),
            0x29 => self.RoomDraw_LowerDoorStairsDown(src, dsto, false),
            0x2a => {
                self.RoomDraw_SingleLampCone(0x0514, 0x16dc);
                self.RoomDraw_SingleLampCone(0x0554, 0x17f6);
                self.RoomDraw_SingleLampCone(0x1514, 0x1914);
                self.RoomDraw_SingleLampCone(0x1554, 0x1a2a);
            }
            0x2b => {
                let mut dst = dsto;
                self.DrawBigGraySegment(0x1010, src, &mut dst, dsto);
            }
            0x2c => {
                let mut dst = dsto;
                self.DrawBigGraySegment(0x2020, 0x0e62, &mut dst, dsto);
                self.DrawBigGraySegment(0x2121, 0x0e6a, &mut dst, dsto + xy(2, 0) as u16);
                self.DrawBigGraySegment(0x2222, 0x0e72, &mut dst, dsto + xy(0, 2) as u16);
                self.DrawBigGraySegment(0x2323, 0x0e7a, &mut dst, dsto + xy(2, 2) as u16);
            }
            0x2d => self.RoomDraw_AgahnimAltar(dsto),
            0x2e => self.RoomDraw_AgahnimsWindows(dsto),
            0x2f => {
                let mut dst = 0;
                self.RoomDraw_SinglePot(0x0e82, &mut dst, dsto);
            }
            0x30 => {
                let mut dst = dsto;
                self.DrawBigGraySegment(0x1212, src, &mut dst, dsto);
            }
            0x31 => {
                let loc = dsto * 2 | 0x8000 | self.room_plane_tilemap_bit();
                let chest = self
                    .dungeon_room_items_mut()
                    .append_chest_location_and_sync_big_key_count(loc);
                if chest < DUNGEON_CHEST_OPEN_MASKS.len()
                    && self.game_state.dungeon.savegame_state.savegame_state_bits()
                        & DUNGEON_CHEST_OPEN_MASKS[chest]
                        != 0
                {
                    self.dungeon_room_items_mut().clear_chest_location(chest);
                    self.RoomDraw_1x3_rightwards(0x14c4, dsto, 4);
                } else {
                    self.RoomDraw_1x3_rightwards(0x14ac, dsto, 4);
                }
            }
            0x32 => self.RoomDraw_1x3_rightwards(src, dsto, 4),
            0x33 => {
                if self.game_state.dungeon.header.header_tag(1) == 27 {
                    let room = self.game_state.world.location.dungeon_room() as usize;
                    if self
                        .game_state
                        .inventory
                        .save_progress
                        .dungeon_info_word(room)
                        & 0x0100
                        == 0
                    {
                        self.dungeon_room_load_mut().clear_bg2_properties();
                        self.dungeon_stair_lists_mut()
                            .append_stair_table_position(DungeonStairList::WetStairs, dsto);
                    } else {
                        self.set_color_window_selection(2);
                        self.set_color_math_control(0x62);
                        self.dungeon_stair_lists_mut().append_stair_table_position(
                            DungeonStairList::InRoomUpSouthWater,
                            dsto,
                        );
                    }
                } else {
                    self.dungeon_stair_lists_mut()
                        .append_stair_table_position(DungeonStairList::InRoomUpSouthWater, dsto);
                }
                self.RoomDraw_4x4(src, dsto);
            }
            0x3a | 0x3b => {
                self.RoomDraw_1x3_rightwards(src, dsto, 4);
                self.RoomDraw_1x3_rightwards(src + 24, dsto + 3 * 64, 4);
            }
            0x3c | 0x3d | 0x5c => self.RoomDraw_Object_Nx4(6, src, dsto),
            0x47 => {
                let mut dst = dsto;
                self.RoomDraw_BombableFloor(src, &mut dst, dsto);
            }
            0x48 | 0x66 | 0x6b | 0x7a => self.RoomDraw_4x4(src, dsto),
            0x4b | 0x76 | 0x77 => self.RoomDraw_1x3_rightwards(src, dsto, 8),
            0x4c => self.RoomDraw_SomeBigDecors(6, 0x1f92, dsto),
            0x4d | 0x5d => self.RoomDraw_1x3_rightwards(src, dsto, 6),
            0x54 => self.RoomDraw_FortuneTellerRoom(dsto),
            0x55 | 0x5b => {
                for x in 0..3 {
                    self.room_write_current(dsto + x, self.tile_word(src, x as usize));
                }
                for y in 1..=3 {
                    for x in 0..3 {
                        self.room_write_current(
                            dsto + y * 64 + x,
                            self.tile_word(src, (3 + x) as usize),
                        );
                    }
                }
                for x in 0..3 {
                    self.room_write_current(
                        dsto + x + 4 * 64,
                        self.tile_word(src, (6 + x) as usize),
                    );
                }
            }
            0x5a => self.RoomDraw_WaterHoldingObject(2, src, dsto),
            0x60 | 0x61 => {
                self.RoomDraw_1x3_rightwards(src, dsto, 3);
                self.RoomDraw_1x3_rightwards(src + 18, dsto + 3 * 64, 3);
            }
            0x62 => {
                let mut dst = dsto;
                let mut s = 0x20f6;
                for _ in 0..22 {
                    for y in 0..11 {
                        self.room_write_bg1(dst + y * 64, self.tile_word(s, y as usize));
                    }
                    dst += 1;
                    s += 22;
                }
                dst -= 22;
                s = 0x22da;
                for i in 0..3 {
                    self.room_write_bg1(dst + 9 + 11 * 64, self.tile_word(s, i));
                    self.room_write_bg1(dst + 9 + 12 * 64, self.tile_word(s, i + 3));
                    dst += 1;
                }
            }
            0x69 | 0x6a | 0x6e | 0x6f => self.RoomDraw_Object_Nx4(3, src, dsto),
            0x70 => {
                self.RoomDraw_4x4(src, dsto);
                self.RoomDraw_4x4(0x2376, dsto + 2 * 64);
                self.RoomDraw_4x4(0x2396, dsto + 6 * 64);
            }
            0x71 => {
                if self
                    .game_state
                    .inventory
                    .save_progress
                    .dungeon_info_word(101)
                    & 0x0100
                    != 0
                {
                    self.Object_Draw8x8(src, dsto);
                }
            }
            0x72 => {
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 == 0 {
                    self.RoomDraw_SomeBigDecors(10, src, dsto);
                }
            }
            0x73 => self.RoomDraw_FloorChunks(
                self.game_state
                    .dungeon
                    .room_tilemaps
                    .first_line_pointer_row0() as usize,
                0x00e0,
            ),
            0x74 => {
                self.Object_Draw8x8(src, dsto);
            }
            0x78 => {
                self.RoomDraw_4x4(src, dsto);
                self.RoomDraw_4x4(src + 32, dsto.wrapping_sub(2).wrapping_add(4 * 64));
                self.RoomDraw_4x4(src + 32, dsto + 2 + 4 * 64);
            }
            0x7b => {
                let mut dst = dsto;
                for _ in 0..5 {
                    self.RoomDraw_A_Many32x32Blocks(1, src, &mut dst);
                }
                let mut dst = dsto + 4 * 64;
                for _ in 0..5 {
                    self.RoomDraw_A_Many32x32Blocks(1, src, &mut dst);
                }
            }
            _ => panic!("LoadType1ObjectSubtype3 unhandled object id {idx:#04x}"),
        }
    }

    pub(super) fn LoadType1ObjectSubtype2(&mut self, idx: u8, _dst: &mut u16, dsto: u16) {
        let Some(src) = object_subtype2_param(idx) else {
            panic!("LoadType1ObjectSubtype2 invalid object id {idx:#04x}");
        };
        match idx {
            0x00..=0x07 | 0x1c | 0x24 | 0x25 | 0x29 => {
                self.RoomData_DrawObject_nx4(src, dsto, 4);
            }
            0x10..=0x13 => self.RoomData_DrawObject_nx4_both_bgs(src, dsto, 3),
            0x14..=0x17 => self.Object_DrawNx3_BothBgs(4, src, dsto),
            0x18..=0x1b | 0x27 | 0x2b | 0x34 => self.RoomDraw_Rightwards2x2(src, dsto),
            0x1d | 0x21 | 0x26 => self.RoomDraw_1x3_rightwards(src, dsto, 2),
            0x1e => self.RoomDraw_Rightwards2x2(src, dsto),
            0x1f => {
                let plane = self.room_plane_offset();
                self.dungeon_room_parser_mut()
                    .append_star_switch_tile(dsto | plane);
                self.RoomDraw_Rightwards2x2(src, dsto);
            }
            0x20 => {
                self.dungeon_torch_mut().increment_lit_torches();
                self.RoomDraw_Rightwards2x2(src, dsto);
            }
            0x22 | 0x28 => self.Object_Draw_5x4(src, dsto),
            0x23 => self.RoomDraw_1x3_rightwards(src, dsto, 4),
            0x2a => {
                self.dungeon_room_load_mut().set_draw_width_indicator(1);
                self.RoomDraw_Downwards4x2VariableSpacing(1, src, dsto, 1);
            }
            0x2c => self.RoomDraw_1x3_rightwards(src, dsto, 6),
            0x2d => {
                let plane = self.room_plane_offset();
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomUpNorth, dsto | plane);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::InterRoomUpNorth,
                        DungeonStairList::WallUpNorthSpiral,
                        DungeonStairList::WallUpNorthSpiralBg1,
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
                self.RoomDraw_4x4(0x1088, dsto);
            }
            0x2e | 0x2f => {
                let plane = self.room_plane_offset();
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomSouthDown, dsto | plane);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::InterRoomSouthDown,
                        DungeonStairList::WallDownNorthSpiral,
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.RoomDraw_4x4(0x10a8, dsto);
            }
            0x08..=0x0f => self.RoomData_DrawObject_nx4_both_bgs(src, dsto, 4),
            0x31 => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_bg1_stair_table_position(DungeonStairList::InRoomSouthDown, dsto);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WaterLadders,
                        DungeonStairList::WaterSideStepSwitch,
                    ],
                    next,
                );
                self.RoomData_DrawObject_nx4_both_bgs(src, dsto, 4);
            }
            0x32 => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_bg1_stair_table_position(DungeonStairList::InterPseudoUpNorth, dsto);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WaterLadders,
                        DungeonStairList::WaterSideStepSwitch,
                    ],
                    next,
                );
                self.RoomDraw_4x4(src, dsto);
            }
            0x33 => {
                let room = self.game_state.world.location.dungeon_room() as usize;
                if self.game_state.dungeon.header.header_tag(1) == 27
                    && self
                        .game_state
                        .inventory
                        .save_progress
                        .dungeon_info_word(room)
                        & 0x0100
                        == 0
                {
                    self.dungeon_room_load_mut().clear_bg2_properties();
                    let next = self
                        .dungeon_stair_lists_mut()
                        .append_bg1_stair_table_position(
                            DungeonStairList::InterPseudoUpNorth,
                            dsto,
                        );
                    self.dungeon_stair_lists_mut().sync_stair_list_counts(
                        &[
                            DungeonStairList::WaterLadders,
                            DungeonStairList::WaterSideStepSwitch,
                        ],
                        next,
                    );
                    self.RoomDraw_4x4(0x10c8, dsto);
                } else {
                    let next = self
                        .dungeon_stair_lists_mut()
                        .append_bg1_stair_table_position(
                            DungeonStairList::InRoomUpNorthWater,
                            dsto,
                        );
                    self.dungeon_stair_lists_mut()
                        .set_stair_list_count(DungeonStairList::ActivatedWaterLadders, next);
                    self.RoomDraw_4x4(0x10c8, dsto);
                }
            }
            0x35 => {
                let room = self.game_state.world.location.dungeon_room() as usize;
                if self.game_state.dungeon.header.header_tag(1) == 27
                    && self
                        .game_state
                        .inventory
                        .save_progress
                        .dungeon_info_word(room)
                        & 0x0100
                        == 0
                {
                    let next = self
                        .dungeon_stair_lists_mut()
                        .append_bg1_stair_table_position(DungeonStairList::WaterLadders, dsto);
                    self.dungeon_stair_lists_mut()
                        .set_stair_list_count(DungeonStairList::WaterSideStepSwitch, next);
                    self.Object_Draw_4x2_BothBgs(0x1108, dsto);
                } else {
                    self.dungeon_stair_lists_mut()
                        .append_bg1_stair_table_position(
                            DungeonStairList::ActivatedWaterLadders,
                            dsto,
                        );
                    self.dungeon_room_load_mut().set_draw_width_indicator(1);
                    self.RoomDraw_Downwards4x2VariableSpacing(1, 0x1108, dsto, 1);
                }
            }
            0x36 => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_bg1_stair_table_position(DungeonStairList::WaterLadders, dsto);
                self.dungeon_stair_lists_mut()
                    .set_stair_list_count(DungeonStairList::WaterSideStepSwitch, next);
                self.Object_Draw_4x2_BothBgs(0x1108, dsto);
            }
            0x37 => {
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0800 == 0 {
                    self.RoomDraw_Object_Nx4(10, src, dsto);
                    self.water_hdma_window_mut().set_watergate_pointer(0x0f);
                    self.water_hdma_window_mut()
                        .set_watergate_tilemap_pos_x2(dsto * 2);
                } else {
                    self.RoomDraw_Object_Nx4(10, 0x13e8, dsto);
                    let load_ptr = self.game_state.dungeon.door_setup.active_room_load_ptr();
                    let load_ptr_offs = self.game_state.dungeon.room_load.load_ptr_offset();
                    let load_ptr_bank = self
                        .game_state
                        .dungeon
                        .door_setup
                        .active_room_load_ptr_bank();
                    self.RoomTag_OperateWaterFlooring();
                    self.dungeon_room_doors_mut()
                        .set_active_room_load_ptr_bank(load_ptr_bank);
                    self.dungeon_room_load_mut()
                        .set_load_ptr_offset(load_ptr_offs);
                    self.dungeon_room_doors_mut()
                        .set_active_room_load_ptr(load_ptr);
                }
            }
            0x38 => {
                let plane = self.room_plane_offset();
                let next = self.dungeon_stair_lists_mut().append_interroom_staircase(
                    DungeonStairList::WallUpNorthSpiral,
                    dsto.wrapping_sub(0x40) | plane,
                );
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WallUpNorthSpiral,
                        DungeonStairList::WallUpNorthSpiralBg1,
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
                self.RoomDraw_1x3_rightwards(0x1148, dsto, 4);
                let left = self.room_read_bg2(dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg2(dsto.wrapping_sub(1), left);
                let right = self.room_read_bg2(dsto + 4) | 0x2000;
                self.room_write_bg2(dsto + 4, right);
            }
            0x39 => {
                let plane = self.room_plane_offset();
                let next = self.dungeon_stair_lists_mut().append_interroom_staircase(
                    DungeonStairList::WallDownNorthSpiral,
                    dsto.wrapping_sub(0x40) | plane,
                );
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WallDownNorthSpiral,
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.RoomDraw_1x3_rightwards(0x1160, dsto, 4);
                let left = self.room_read_bg2(dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg2(dsto.wrapping_sub(1), left);
                let right = self.room_read_bg2(dsto + 4) | 0x2000;
                self.room_write_bg2(dsto + 4, right);
            }
            0x3a => {
                let plane = self.room_plane_offset();
                let next = self.dungeon_stair_lists_mut().append_interroom_staircase(
                    DungeonStairList::WallUpNorthSpiralBg1,
                    dsto.wrapping_sub(0x40) | plane,
                );
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WallUpNorthSpiralBg1,
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
                self.RoomDraw_1x3_rightwards(0x1178, dsto, 4);
                let left = self.room_read_bg1(dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg1(dsto.wrapping_sub(1), left);
                let right = self.room_read_bg1(dsto + 4) | 0x2000;
                self.room_write_bg1(dsto + 4, right);
            }
            0x3b => {
                let plane = self.room_plane_offset();
                let next = self.dungeon_stair_lists_mut().append_interroom_staircase(
                    DungeonStairList::WallDownNorthSpiralBg1,
                    dsto.wrapping_sub(0x40) | plane,
                );
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.RoomDraw_1x3_rightwards(0x1190, dsto, 4);
                let left = self.room_read_bg1(dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg1(dsto.wrapping_sub(1), left);
                let right = self.room_read_bg1(dsto + 4) | 0x2000;
                self.room_write_bg1(dsto + 4, right);
            }
            0x3c => {
                let mut dst = dsto;
                let mut s = src;
                for _ in 0..6 {
                    let tile0 = self.tile_word(s, 0);
                    let tile6 = self.tile_word(s, 6);
                    for x in [0, 4, 8, 14, 18, 22] {
                        self.room_write_bg2(dst + x, tile0);
                    }
                    for x in [1, 5, 9, 15, 19, 23] {
                        self.room_write_bg2(dst + x, tile0 | 0x4000);
                    }
                    for x in [2, 6, 16, 20] {
                        self.room_write_bg2(dst + x, tile6);
                    }
                    for x in [3, 7, 17, 21] {
                        self.room_write_bg2(dst + x, tile6 | 0x4000);
                    }
                    dst += 64;
                    s += 2;
                }
                self.RoomDraw_1x3_rightwards(src + 24, dsto + 10, 4);
            }
            0x3e => self.RoomDraw_1x3_rightwards(src, dsto, 6),
            0x3f => {
                let mut dst = dsto | self.room_plane_offset();
                let mut s = src;
                for _ in 0..8 {
                    for y in 0..7 {
                        self.room_write_bg2(dst + y * 64, self.tile_word(s, y as usize));
                    }
                    dst += 1;
                    s += 14;
                }
            }
            _ => panic!("LoadType1ObjectSubtype2 unhandled object id {idx:#04x}"),
        }
    }

    pub(super) fn RoomData_DrawObject_Door(&mut self, raw: u16) {
        let door_type = (raw >> 8) as u8;
        let position = ((raw >> 4) & 0x0f) as usize;
        match raw & 3 {
            0 => self.RoomDraw_Door_North(door_type, position),
            1 => self.RoomDraw_Door_South(door_type, position),
            2 => self.RoomDraw_Door_West(door_type, position),
            3 => self.RoomDraw_Door_East(door_type, position),
            _ => unreachable!(),
        }
    }

    pub(super) fn RoomDraw_Door_North(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_UP[position] / 2;
        match door_type {
            DOOR_TYPE_LG_EXPLOSION => self.RoomDraw_Door_ExplodingWall(position),
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.RoomDraw_MarkLayerToggleDoor(dsto.wrapping_sub(0xfe / 2));
            }
            DOOR_TYPE_SLASHABLE => self.RoomDraw_NorthCurtainDoor(dsto),
            DOOR_TYPE_ENTRANCE_DOOR => self.Door_Up_EntranceDoor(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.RoomDraw_MarkDungeonToggleDoor(dsto.wrapping_sub(0xfe / 2))
            }
            DOOR_TYPE_REGULAR2 => {
                self.RoomDraw_MakeDoorPartsHighPriority_Y(dsto & (0xf07f / 2));
                self.RoomDraw_NormalRangedDoors_North(door_type, dsto, position);
            }
            DOOR_TYPE_EXIT_TO_OW => self.room_draw_register_exit_door(dsto),
            DOOR_TYPE_WATERFALL_TUNNEL => {
                self.RoomDraw_NormalRangedDoors_North(door_type, dsto, position);
                self.Door_PrioritizeCurDoor();
            }
            t if (DOOR_TYPE_STAIR_MASK_LOCKED0..=DOOR_TYPE_STAIR_MASK_LOCKED3).contains(&t) => {
                self.Door_Up_StairMaskLocked(door_type, dsto);
            }
            t if (DOOR_TYPE_REGULAR_DOOR33..).contains(&t) => {
                self.RoomDraw_HighRangeDoor_North(door_type, dsto, position);
            }
            _ => self.RoomDraw_NormalRangedDoors_North(door_type, dsto, position),
        }
    }

    pub(super) fn RoomDraw_Door_South(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_DOWN[position] / 2;
        match door_type {
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.RoomDraw_MarkLayerToggleDoor(dsto + xy(1, 4) as u16);
            }
            DOOR_TYPE_ENTRANCE_DOOR => self.Door_Down_EntranceDoor(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.RoomDraw_MarkDungeonToggleDoor(dsto + xy(1, 4) as u16);
            }
            DOOR_TYPE_EXIT_TO_OW => self.room_draw_register_exit_door(dsto),
            t if t >= DOOR_TYPE_REGULAR_DOOR33 => {
                self.RoomDraw_OneSidedLowerShutters_South(door_type, dsto);
            }
            DOOR_TYPE_ENTRANCE_LARGE => {
                self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, dsto);
                self.RoomDraw_SomeBigDecors(10, 0x2656, dsto.wrapping_sub(3 + 4 * 64));
            }
            DOOR_TYPE_ENTRANCE_LARGE2 => {
                let mut dsto = dsto | 0x1000;
                self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, dsto);
                dsto = dsto.wrapping_sub(3 + 4 * 64);
                self.RoomDraw_SomeBigDecors(10, 0x2656, dsto);
                dsto = dsto.wrapping_sub(0x1000).wrapping_add(7 * 64);
                for i in 0..10 {
                    let tile = self.room_read_bg1(dsto + i) | 0x2000;
                    self.room_write_bg2(dsto + i, tile);
                }
            }
            DOOR_TYPE_ENTRANCE_CAVE | DOOR_TYPE_ENTRANCE_CAVE2 => {
                if door_type == DOOR_TYPE_ENTRANCE_CAVE2 {
                    self.RoomDraw_MakeDoorPartsHighPriority_Y(dsto + xy(0, 4) as u16);
                }
                self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, dsto);
                self.RoomDraw_4x4(0x26f6, dsto);
            }
            DOOR_TYPE_4 => {
                let high_dsto = dsto | 0x1000;
                self.RoomDraw_MakeDoorPartsHighPriority_Y(high_dsto + xy(0, 4) as u16);
                self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, high_dsto);
                self.RoomDraw_4x4(0x26f6, high_dsto);
                for i in 0..4 {
                    let pos = dsto + i + xy(0, 3) as u16;
                    let tile = self.room_read_bg1(pos) | 0x2000;
                    self.room_write_bg2(pos, tile);
                }
            }
            _ => self.RoomDraw_CheckIfLowerLayerDoors_Y(door_type, dsto),
        }
    }

    pub(super) fn RoomDraw_Door_West(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_LEFT[position] / 2;
        match door_type {
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.RoomDraw_MarkLayerToggleDoor(dsto.wrapping_add(62));
            }
            DOOR_TYPE_ENTRANCE_DOOR => self.Door_Left_EntranceDoor(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.RoomDraw_MarkDungeonToggleDoor(dsto.wrapping_add(62));
            }
            DOOR_TYPE_REGULAR2 => {
                self.RoomDraw_MakeDoorPartsHighPriority_X(dsto & !0x1f);
                self.RoomDraw_NormalRangedDoors_West(door_type, dsto, position);
            }
            DOOR_TYPE_WATERFALL_TUNNEL => {
                self.RoomDraw_NormalRangedDoors_West(door_type, dsto, position);
                self.Door_PrioritizeCurDoor();
            }
            t if t < DOOR_TYPE_REGULAR_DOOR33 => {
                self.RoomDraw_NormalRangedDoors_West(door_type, dsto, position);
            }
            _ => self.RoomDraw_HighRangeDoor_West(door_type, dsto, position),
        }
    }

    pub(super) fn RoomDraw_Door_East(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_RIGHT[position] / 2;
        match door_type {
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.RoomDraw_MarkLayerToggleDoor(dsto + xy(4, 1) as u16);
            }
            DOOR_TYPE_ENTRANCE_DOOR => self.Door_Right_EntranceDoor(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.RoomDraw_MarkDungeonToggleDoor(dsto + xy(4, 1) as u16);
            }
            t if t < DOOR_TYPE_REGULAR_DOOR33 => {
                self.RoomDraw_NormalRangedDoors_East(door_type, dsto)
            }
            _ => self.RoomDraw_OneSidedLowerShutters_East(door_type, dsto),
        }
    }

    pub(super) fn room_draw_register_exit_door(&mut self, dsto: u16) {
        self.dungeon_room_doors_mut()
            .append_exit_door_address(dsto * 2);
    }

    pub(super) fn RoomDraw_NormalRangedDoors_North(
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
                self.RoomDraw_CheckIfLowerLayerDoors_Y(door_type, down_dsto_bytes / 2);
                self.dungeon_doors_mut().set_current_door_index(saved);
            }
        }
        self.RoomDraw_OneSidedShutters_North(door_type, dsto);
    }

    pub(super) fn RoomDraw_OneSidedShutters_North(&mut self, door_type: u8, dsto: u16) {
        let mut final_type = self.RoomDraw_FlagDoorsAndGetFinalType(0, door_type, dsto);
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
            self.RoomData_DrawObject_Door_up_4x3(src as usize, dsto);
        }
    }

    pub(super) fn Door_Up_StairMaskLocked(&mut self, door_type: u8, mut dsto: u16) {
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
            self.RoomDraw_OneSidedShutters_North(door_type, dsto);
            return;
        }

        let t = self.RoomDraw_FlagDoorsAndGetFinalType(0, door_type, dsto) as usize;
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
        self.Door_PrioritizeCurDoor();
    }

    pub(super) fn RoomDraw_Door_ExplodingWall(&mut self, pos_enum: usize) {
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
        self.RoomDraw_ExplodingWallSegment(DOOR_TYPE_SRC_DOWN[42] as usize, dsto);
        let next = self
            .game_state
            .dungeon
            .doors
            .current_door_index()
            .wrapping_add(2);
        self.dungeon_doors_mut().set_current_door_index(next);
        self.dungeon_room_doors_mut()
            .add_reset_xy_check_flags(0x0200);
        self.RoomDraw_ExplodingWallSegment(DOOR_TYPE_SRC_UP[42] as usize, dsto + xy(0, 6) as u16);
    }

    pub(super) fn RoomDraw_ExplodingWallSegment(&mut self, mut src: usize, mut dsto: u16) {
        self.RoomDraw_ExplodingWallColumn(src, dsto);
        src += 24;
        dsto = dsto.wrapping_add(2);
        let fill = self.tile_word(src, 0);
        self.dungeon_room_load_mut().set_draw_width_indicator(18);
        for x in 0..18u16 {
            for y in 0..6u16 {
                self.room_write_bg2(dsto + x + y * 64, fill);
            }
        }
        self.RoomDraw_ExplodingWallColumn(src + 2, dsto + 18);
    }

    pub(super) fn RoomDraw_ExplodingWallColumn(&mut self, src: usize, dsto: u16) {
        for i in 0..6u16 {
            self.room_write_current(dsto + i * 64, self.tile_word(src, i as usize));
            self.room_write_current(dsto + 1 + i * 64, self.tile_word(src, (i + 6) as usize));
        }
    }

    pub(super) fn RoomDraw_CheckIfLowerLayerDoors_Y(&mut self, door_type: u8, dsto: u16) {
        if door_type == DOOR_TYPE_REGULAR2 {
            self.RoomDraw_MakeDoorPartsHighPriority_Y(dsto + xy(0, 4) as u16);
            self.Door_Draw_Helper4(door_type, dsto);
        } else if door_type == DOOR_TYPE_WATERFALL_TUNNEL {
            self.Door_Draw_Helper4(door_type, dsto);
            self.Door_PrioritizeCurDoor();
        } else {
            self.Door_Draw_Helper4(door_type, dsto);
        }
    }

    pub(super) fn RoomDraw_NormalRangedDoors_West(
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
                self.RoomDraw_NormalRangedDoors_East(door_type, right_dsto_bytes / 2);
                self.dungeon_doors_mut().set_current_door_index(saved);
            }
        }

        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(2, door_type, dsto);
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
            self.RoomData_DrawObject_Door_left_3x4(src as usize, dsto);
        }
    }

    pub(super) fn RoomDraw_NormalRangedDoors_East(&mut self, door_type: u8, dsto: u16) {
        if door_type == DOOR_TYPE_REGULAR2 {
            self.RoomDraw_MakeDoorPartsHighPriority_X(dsto + xy(4, 0) as u16);
        }
        if door_type == DOOR_TYPE_WATERFALL_TUNNEL {
            self.RoomDraw_OneSidedShutters_East(door_type, dsto);
            self.Door_PrioritizeCurDoor();
        } else {
            self.RoomDraw_OneSidedShutters_East(door_type, dsto);
        }
    }

    pub(super) fn RoomDraw_OneSidedShutters_East(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(3, door_type, dsto);
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
            self.RoomData_DrawObject_Door_right_3x4(src as usize, dsto);
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

    pub(super) fn RoomDraw_FlagDoorsAndGetFinalType(
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

    pub(super) fn RoomDraw_MarkDungeonToggleDoor(&mut self, dsto: u16) {
        self.dungeon_room_parser_mut()
            .append_toggle_palace_pos(dsto);
    }

    pub(super) fn RoomDraw_MarkLayerToggleDoor(&mut self, dsto: u16) {
        self.dungeon_room_parser_mut().append_toggle_floor_pos(dsto);
    }

    pub(super) fn RoomData_DrawObject_Door_up_4x3(&mut self, src: usize, dsto: u16) {
        for x in 0..4 {
            for y in 0..3 {
                let tile = self.tile_word(src, (x * 3 + y) as usize);
                self.room_write_current(dsto + x + y * 64, tile);
            }
        }
    }

    pub(super) fn RoomData_DrawObject_Door_down_4x3(&mut self, src: usize, dsto: u16) {
        for x in 0..4 {
            for y in 0..3 {
                let tile = self.tile_word(src, (x * 3 + y) as usize);
                self.room_write_current(dsto + x + (y + 1) * 64, tile);
            }
        }
    }

    pub(super) fn RoomData_DrawObject_Door_left_3x4(&mut self, src: usize, dsto: u16) {
        for x in 0..3 {
            for y in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
    }

    pub(super) fn RoomData_DrawObject_Door_right_3x4(&mut self, src: usize, dsto: u16) {
        for x in 0..3 {
            for y in 0..4 {
                self.room_write_current(
                    dsto + 1 + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
    }

    pub(super) fn Door_PrioritizeCurDoor(&mut self) {
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

    pub(super) fn RoomDraw_NorthCurtainDoor(&mut self, dsto: u16) {
        let rv = self.RoomDraw_FlagDoorsAndGetFinalType(0, DOOR_TYPE_SLASHABLE, dsto);
        let src = if rv & 0x100 != 0 {
            0x078a
        } else {
            DOOR_TYPE_SRC_UP
                .get(rv as usize >> 1)
                .copied()
                .unwrap_or(0x078a) as usize
        };
        self.RoomDraw_4x4(src, dsto);
    }

    pub(super) fn RoomDraw_HighRangeDoor_North(
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
                self.RoomDraw_OneSidedLowerShutters_South(door_type, down_dsto_bytes / 2);
                self.dungeon_doors_mut().set_current_door_index(saved);
            }
        }
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(0, door_type, dsto);
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
            self.RoomDraw_MakeDoorHighPriority_North(dsto);
        }
        self.Door_PrioritizeCurDoor();
    }

    pub(super) fn RoomDraw_OneSidedLowerShutters_South(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, dsto);
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
        self.RoomDraw_MakeDoorHighPriority_South(dsto + xy(0, 4) as u16);
        self.Door_PrioritizeCurDoor();
    }

    pub(super) fn RoomDraw_HighRangeDoor_West(
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
                self.RoomDraw_OneSidedLowerShutters_East(door_type, right_dsto_bytes / 2);
                self.dungeon_doors_mut().set_current_door_index(saved);
            }
        }
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(2, door_type, dsto);
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
        self.RoomDraw_MakeDoorHighPriority_West(dsto);
        self.Door_PrioritizeCurDoor();
    }

    pub(super) fn RoomDraw_OneSidedLowerShutters_East(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(3, door_type, dsto);
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
        self.RoomDraw_MakeDoorHighPriority_East(dsto + xy(4, 0) as u16);
        self.Door_PrioritizeCurDoor();
    }

    pub(super) fn RoomDraw_MakeDoorHighPriority_North(&mut self, dsto_org: u16) {
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

    pub(super) fn RoomDraw_MakeDoorHighPriority_South(&mut self, mut dsto: u16) {
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

    pub(super) fn RoomDraw_MakeDoorHighPriority_West(&mut self, dsto_org: u16) {
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

    pub(super) fn RoomDraw_MakeDoorHighPriority_East(&mut self, mut dsto: u16) {
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

    pub(super) fn RoomDraw_MakeDoorPartsHighPriority_Y(&mut self, dsto: u16) {
        for y in 0..7 {
            for x in 0..4 {
                let pos = dsto + x + y * 64;
                let tile = self.room_read_bg2(pos) | 0x2000;
                self.room_write_bg2(pos, tile);
            }
        }
    }

    pub(super) fn RoomDraw_MakeDoorPartsHighPriority_X(&mut self, dsto: u16) {
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

    pub(super) fn RoomDraw_1x3_rightwards(&mut self, src: usize, dsto: u16, columns: u16) {
        for x in 0..columns {
            for y in 0..3 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 3 + y) as usize),
                );
            }
        }
    }

    pub(super) fn Object_Draw_5x4(&mut self, src: usize, dsto: u16) {
        for y in 0..5 {
            for x in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    pub(super) fn RoomDraw_RightwardShelfEnd<'a>(
        &mut self,
        src: usize,
        dst: &'a mut u16,
    ) -> &'a mut u16 {
        let dsto = *dst;
        for y in 0..4 {
            self.room_write_current(dsto + y * 64, self.tile_word(src, y as usize));
        }
        dst
    }

    pub(super) fn RoomDraw_RightwardBarSegment(&mut self, src: usize, dsto: u16) -> u16 {
        for y in 0..3 {
            self.room_write_current(dsto + y * 64, self.tile_word(src, y as usize));
        }
        dsto
    }

    pub(super) fn RoomDraw_DrawObject2x2and1(&mut self, src: usize, dsto: u16) -> u16 {
        for y in 0..5 {
            self.room_write_current(dsto + y * 64, self.tile_word(src, y as usize));
        }
        dsto
    }

    #[track_caller]
    pub(super) fn RoomDraw_Downwards4x2VariableSpacing(
        &mut self,
        increment: u16,
        src: usize,
        dsto: u16,
        count: u16,
    ) {
        let mut dst = dsto;
        for _ in 0..count {
            for x in 0..4 {
                for y in 0..2 {
                    self.room_write_current(
                        dst + x + y * 64,
                        self.tile_word(src, (y * 4 + x) as usize),
                    );
                }
            }
            dst = dst.wrapping_add(increment);
        }
        self.dungeon_room_load_mut().set_draw_width_indicator(0);
    }

    pub(super) fn Object_Table_Helper(&mut self, src: usize, dsto: u16, width: u16) {
        self.room_write_current(dsto, self.tile_word(src, 0));
        for segment in 0..width {
            let dst = dsto + 1 + segment * 2;
            self.room_write_current(dst, self.tile_word(src, 1));
            self.room_write_current(dst + 1, self.tile_word(src, 2));
        }
        self.room_write_current(dsto + 1 + width * 2, self.tile_word(src, 3));
    }

    pub(super) fn RoomDraw_CheckIfWallIsMoved(&mut self) -> bool {
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

    pub(super) fn MovingWall_FillReplacementBuffer(&mut self, dsto: u16) {
        self.dungeon_room_effects_mut()
            .fill_moving_wall_replacement_buffer(0x01ec);
        let value = (dsto & 0x001f) | if dsto & 0x0020 != 0 { 0x0400 } else { 0 } | 0x1000;
        self.dungeon_room_effects_mut()
            .set_moving_wall_write_point(value);
    }

    pub(super) fn RoomDraw_MovingWallRight(&mut self, width: u8, height: u8, dsto: u16) {
        if !self.RoomDraw_CheckIfWallIsMoved() {
            return;
        }
        self.dungeon_room_load_mut()
            .increment_header_collision_2_mirror();
        let size0 = MOVING_WALL_SIZE_TABLE0[width as usize];
        let size1 = MOVING_WALL_SIZE_TABLE1[height as usize];
        self.MovingWall_FillReplacementBuffer(dsto.wrapping_sub(size1).wrapping_sub(1));
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
        self.RoomDraw_1x3_rightwards(src, dst, 3);
        dst += 3 * 64;
        for _ in 0..size0 {
            self.Object_Draw_3x2(src + 18, dst);
            dst += 2 * 64;
        }
        self.RoomDraw_1x3_rightwards(src + 30, dst, 3);
    }

    pub(super) fn RoomDraw_MovingWallLeft(&mut self, width: u8, height: u8, dsto: u16) {
        if !self.RoomDraw_CheckIfWallIsMoved() {
            return;
        }
        self.dungeon_room_load_mut()
            .increment_header_collision_2_mirror();
        let size1 = MOVING_WALL_SIZE_TABLE1[height as usize];
        let size0 = MOVING_WALL_SIZE_TABLE0[width as usize];
        self.dungeon_room_effects_mut()
            .set_moving_wall_dot_pointer(height.wrapping_mul(2));
        self.MovingWall_FillReplacementBuffer(dsto.wrapping_add(3).wrapping_add(size1));

        let src = 0x075a;
        let mut dst = dsto;
        self.RoomDraw_1x3_rightwards(src, dst, 3);
        dst += 3 * 64;
        for _ in 0..size0 {
            self.Object_Draw_3x2(src + 18, dst);
            dst += 2 * 64;
        }
        self.RoomDraw_1x3_rightwards(src + 30, dst, 3);

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

    pub(super) fn Object_DrawNx3_BothBgs(&mut self, n: u16, src: usize, dsto: u16) {
        for x in 0..n {
            for y in 0..3 {
                let tile = self.tile_word(src, (x * 3 + y) as usize);
                self.room_write_bg2(dsto + x + y * 64, tile);
                self.room_write_bg1(dsto + x + y * 64, tile);
            }
        }
    }

    pub(super) fn bomb_check_for_destructibles(&mut self, x: u16, y: u16, r14: u8) {
        if self.game_state.frame.main_module != 7 {
            self.overworld_bomb_tiles32x32(x, y);
            return;
        }

        let mut k = (((y & 0x01f8) << 3) | ((x & 0x01f8) >> 3)).wrapping_sub(0x0082) as usize;
        for _ in (0..=2).rev() {
            for step in 0..3 {
                let a = self.game_state.dungeon.bg2_attributes.bg2_attr(k);
                if a == 0x62 {
                    if self.game_state.world.location.dungeon_room() == 0x65 {
                        let bits =
                            self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x1000;
                        self.dungeon_savegame_state_mut()
                            .set_savegame_state_bits(bits);
                    }
                    let mut pt = Point16U { x: 0, y: 0 };
                    self.ThievesAttic_DrawLightenedHole(0, 0, &mut pt);
                    self.set_sound_effect_2(0x1b);
                    return;
                }
                if (a & 0xf0) == 0xf0 {
                    let j = (a & 0x0f) as usize;
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

    pub(super) fn prepare_dungeon_exit_from_boss_fight(&mut self) {
        self.SavePalaceDeaths();
        self.SaveDungeonKeys_misc();
        let bits = self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x8000;
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits(bits);
        self.Dungeon_FlagRoomData_Quadrants();

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

    pub(super) fn Object_BombableFloorHelper(
        &mut self,
        state: u16,
        src: usize,
        src_below: usize,
        _dst: &mut u16,
        dsto: u16,
    ) {
        let index = self.game_state.dungeon.object_tracking.misc_object_index() as usize >> 1;
        self.dungeon_object_tracking_mut()
            .set_replacement_tile_state(index, state);
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
        let tilemap_pos = dsto * 2 | self.room_plane_tilemap_bit();
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
        self.RoomDraw_Rightwards2x2(src, dsto);
    }

    pub(super) fn RoomDraw_BombableFloor(&mut self, _src: usize, dst: &mut u16, dsto: u16) {
        if self.game_state.world.location.dungeon_room() == 101
            && self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x1000 != 0
        {
            self.dungeon_room_load_mut().set_draw_width_indicator(0);
            self.dungeon_room_load_mut().set_draw_height_indicator(0);
            self.Object_Hole(0x05aa, *dst, 0, 0);
            return;
        }

        let src = 0x0220;
        let src_below = 0x05ba;
        self.Object_BombableFloorHelper(0x3030, src, src_below, dst, dsto);
        self.Object_BombableFloorHelper(
            0x3131,
            src + 8,
            src_below + 8,
            dst,
            dsto + xy(2, 0) as u16,
        );
        self.Object_BombableFloorHelper(
            0x3232,
            src + 16,
            src_below + 16,
            dst,
            dsto + xy(0, 2) as u16,
        );
        self.Object_BombableFloorHelper(
            0x3333,
            src + 24,
            src_below + 24,
            dst,
            dsto + xy(2, 2) as u16,
        );
    }

    pub(super) fn RoomDraw_HammerPegSingle(&mut self, src: usize, _dst: &mut u16, dsto: u16) {
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
            .set_replacement_tile_state(index, 0x4040);
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
            .set_object_tilemap_pos(index, dsto * 2 | plane);
        self.dungeon_room_items_mut()
            .set_replacement_tilemap_quad(index, [0x19d8, 0x19d9, 0x59d8, 0x59d9]);
        self.RoomDraw_Rightwards2x2(src, dsto);
    }

    pub(super) fn Object_ChestPlatform_Helper(&mut self, src: usize, dsto: i32) {
        let mut dsto = dsto as usize;
        let t0 = self.tile_word(src, 0);
        let t3 = self.tile_word(src, 3);
        let t6 = self.tile_word(src, 6);
        let t9 = self.tile_word(src, 9);
        let t12 = self.tile_word(src, 12);
        let t15 = self.tile_word(src, 15);
        let t18 = self.tile_word(src, 18);

        self.dungeon_room_tilemaps_mut().set_bg2_tile(dsto, t0);
        for _ in 0..self.game_state.dungeon.room_load.draw_width_indicator() {
            self.dungeon_room_tilemaps_mut().set_bg2_tile(dsto + 1, t3);
            dsto += 1;
        }

        self.dungeon_room_tilemaps_mut().set_bg2_tile(dsto + 1, t6);
        self.dungeon_room_tilemaps_mut().set_bg2_tile(dsto + 2, t9);
        self.dungeon_room_tilemaps_mut().set_bg2_tile(dsto + 3, t9);
        self.dungeon_room_tilemaps_mut().set_bg2_tile(dsto + 4, t9);
        self.dungeon_room_tilemaps_mut().set_bg2_tile(dsto + 5, t9);

        self.dungeon_room_tilemaps_mut().set_bg2_tile(dsto + 6, t12);
        for _ in 0..self.game_state.dungeon.room_load.draw_width_indicator() {
            self.dungeon_room_tilemaps_mut().set_bg2_tile(dsto + 7, t15);
            dsto += 1;
        }

        self.dungeon_room_tilemaps_mut().set_bg2_tile(dsto + 7, t18);
    }

    pub(super) fn RoomDraw_GetObjectSize_1to16(&mut self) {
        self.Object_SizeAtoAplus15(1);
    }

    pub(super) fn Object_SizeAtoAplus15(&mut self, a: u8) {
        let draw_width = (self.game_state.dungeon.room_load.draw_width_indicator() << 2
            | self.game_state.dungeon.room_load.draw_height_indicator())
        .wrapping_add(a);
        self.dungeon_room_load_mut()
            .set_draw_width_indicator(draw_width);
        self.dungeon_room_load_mut().set_draw_height_indicator(0);
    }

    pub(super) fn RoomDraw_GetObjectSize_1to15or26(&mut self) {
        let x = (self.game_state.dungeon.room_load.draw_width_indicator() << 2)
            | self.game_state.dungeon.room_load.draw_height_indicator();
        self.dungeon_room_load_mut()
            .set_draw_width_indicator(if x != 0 { x } else { 26 });
    }

    pub(super) fn RoomDraw_GetObjectSize_1to15or32(&mut self) {
        let x = (self.game_state.dungeon.room_load.draw_width_indicator() << 2)
            | self.game_state.dungeon.room_load.draw_height_indicator();
        self.dungeon_room_load_mut()
            .set_draw_width_indicator(if x != 0 { x } else { 32 });
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

    pub(super) fn RoomDraw_FortuneTellerRoom(&mut self, dsto: u16) {
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

    pub(super) fn RoomDraw_PrisonCell(&mut self, dsto: u16) {
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

    pub(super) fn RoomDraw_CellLock(&mut self, dsto: u16) {
        let index = self.dungeon_room_items_mut().advance_big_key_lock_count();
        if index < DUNGEON_CHEST_OPEN_MASKS.len()
            && self.game_state.dungeon.savegame_state.savegame_state_bits()
                & DUNGEON_CHEST_OPEN_MASKS[index]
                == 0
        {
            self.dungeon_room_items_mut()
                .set_chest_location(index, dsto * 2);
            self.RoomDraw_Rightwards2x2(0x1494, dsto);
        } else if index < 6 {
            self.dungeon_room_items_mut().clear_chest_location(index);
        }
    }

    pub(super) fn RoomDraw_LowerDoorStairsUp(
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
        self.RoomDraw_LowerDoorBg2Priority(priority);
    }

    pub(super) fn RoomDraw_LowerDoorStairsDown(
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
        self.RoomDraw_LowerDoorBg2Priority(priority);
    }

    pub(super) fn RoomDraw_LowerDoorBg2Priority(&mut self, dsto: u16) {
        for y in 0..4 {
            let pos = dsto + y * 64;
            let tile = self.room_read_bg2(pos) | 0x2000;
            self.room_write_bg2(pos, tile);
        }
    }

    pub(super) fn RoomDraw_Chest(&mut self, dsto: u16) {
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
            self.RoomDraw_Rightwards2x2(0x149c, dsto);
        } else {
            self.dungeon_room_items_mut().clear_chest_location(index);
            if let Some(slot) = tag_slot {
                self.dungeon_header_mut().clear_header_tag(slot);
            }
            self.RoomDraw_Rightwards2x2(0x14a4, dsto);
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

    pub(super) fn RoomDraw_SinglePot(&mut self, src: usize, _dst: &mut u16, dsto: u16) {
        let index = self.game_state.dungeon.object_tracking.misc_object_index() as usize >> 1;
        self.dungeon_object_tracking_mut()
            .set_misc_object_index(((index + 1) * 2) as u16);
        self.dungeon_object_tracking_mut()
            .set_replacement_tile_state(index, 0x1111);
        let load_ptr = self.game_state.dungeon.room_load.load_ptr_offset();
        self.dungeon_object_tracking_mut()
            .set_object_data_pos(index, load_ptr);
        let plane_bit = self.room_plane_tilemap_bit();
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(index, dsto * 2 | plane_bit);
        self.dungeon_room_items_mut()
            .set_replacement_tilemap_quad(index, [0x0d0e, 0x0d1e, 0x4d0e, 0x4d1e]);
        let src = if self.game_state.inventory.save_progress.dark_world_state() != 0 {
            0x0e92
        } else {
            src
        };
        self.RoomDraw_Rightwards2x2(src, dsto);
    }

    pub(super) fn Object_Draw_4x2(&mut self, src: usize, dsto: u16) {
        for y in 0..2 {
            for x in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    pub(super) fn Object_Draw_4x2_BothBgs(&mut self, src: usize, dsto: u16) {
        for y in 0..2 {
            for x in 0..4 {
                let tile = self.tile_word(src, (y * 4 + x) as usize);
                self.room_write_bg2(dsto + x + y * 64, tile);
                self.room_write_bg1(dsto + x + y * 64, tile);
            }
        }
    }

    pub(super) fn RoomDraw_Chest_platform_row(&mut self, src: usize, dsto: u16, width: u16) {
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

    pub(super) fn RoomDraw_4x4(&mut self, src: usize, dsto: u16) {
        self.RoomData_DrawObject_nx4(src, dsto, 4);
    }

    pub(super) fn Object_Draw8x8(&mut self, src: usize, dsto: u16) {
        self.RoomDraw_4x4(src, dsto);
        self.RoomDraw_4x4(src + 32, dsto + 4);
        self.RoomDraw_4x4(src + 64, dsto + 4 * 64);
        self.RoomDraw_4x4(src + 96, dsto + 4 + 4 * 64);
    }

    pub(super) fn Object_Draw_3x2(&mut self, src: usize, dsto: u16) {
        for y in 0..2 {
            for x in 0..3 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (y * 3 + x) as usize),
                );
            }
        }
    }

    pub(super) fn RoomDraw_WaterHoldingObject(&mut self, n: u16, src: usize, dsto: u16) {
        for y in 0..n {
            for x in 0..4 {
                self.room_write_current(
                    dsto + y * 64 + x,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    pub(super) fn RoomDraw_SomeBigDecors(&mut self, n: u16, src: usize, dsto: u16) {
        let mut dst = dsto | self.room_plane_offset();
        for y in 0..8 {
            for x in 0..n {
                self.room_write_bg2(dst + x, self.tile_word(src, (y * n + x) as usize));
            }
            dst += 64;
        }
    }

    pub(super) fn RoomDraw_SingleLampCone(&mut self, a: u16, y: usize) {
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

    pub(super) fn RoomDraw_AgahnimsWindows(&mut self, dsto: u16) {
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

    pub(super) fn RoomDraw_AgahnimAltar(&mut self, dsto: u16) {
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

    pub(super) fn RoomDraw_A_Many32x32Blocks(&mut self, mut n: i32, src: usize, dst: &mut u16) {
        loop {
            for _ in 0..2 {
                for y in 0..2 {
                    for x in 0..4 {
                        let tile = self.tile_word(src, (y * 4 + x) as usize);
                        self.room_write_current(*dst + xy(x as usize, y as usize) as u16, tile);
                    }
                }
                *dst += xy(0, 2) as u16;
            }
            *dst = dst
                .wrapping_add(xy(4, 0) as u16)
                .wrapping_sub(xy(0, 4) as u16);
            n -= 1;
            if n == 0 {
                break;
            }
        }
    }

    pub(super) fn RoomData_DrawObject_nx4(&mut self, src: usize, dsto: u16, columns: u16) {
        for x in 0..columns {
            for y in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
    }

    pub(super) fn RoomDraw_Object_Nx4(&mut self, n: u16, src: usize, dsto: u16) {
        self.RoomData_DrawObject_nx4(src, dsto, n);
    }

    pub(super) fn RoomDraw_Object_Nx4_Bg2(&mut self, n: u16, src: usize, dsto: u16) {
        for x in 0..n {
            for y in 0..4 {
                self.room_write_bg2(dsto + x + y * 64, self.tile_word(src, (x * 4 + y) as usize));
            }
        }
    }

    pub(super) fn Object_DrawNx4_BothBgs(&mut self, n: u16, src: usize, dsto: u16) {
        self.RoomData_DrawObject_nx4_both_bgs(src, dsto, n);
    }

    pub(super) fn RoomData_DrawObject_nx4_both_bgs(&mut self, src: usize, dsto: u16, columns: u16) {
        for x in 0..columns {
            for y in 0..4 {
                let tile = self.tile_word(src, (x * 4 + y) as usize);
                self.room_write_bg2(dsto + x + y * 64, tile);
                self.room_write_bg1(dsto + x + y * 64, tile);
            }
        }
    }

    pub(super) fn RoomDraw_Rightwards2x2(&mut self, src: usize, dsto: u16) {
        self.room_write_current(dsto, self.tile_word(src, 0));
        self.room_write_current(dsto + 64, self.tile_word(src, 1));
        self.room_write_current(dsto + 1, self.tile_word(src, 2));
        self.room_write_current(dsto + 65, self.tile_word(src, 3));
    }

    pub(super) fn DrawBigGraySegment(&mut self, a: u16, src: usize, _dst: &mut u16, dsto: u16) {
        let index = self.game_state.dungeon.object_tracking.misc_object_index() as usize >> 1;
        self.dungeon_object_tracking_mut()
            .set_replacement_tile_state(index, a);
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
        self.RoomDraw_Rightwards2x2(src, dsto);
    }

    pub(super) fn DrawObjects_PushableBlock(&mut self, dsto_x2: u16, slot: u16) {
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
            .set_replacement_tile_state(x, 0);
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
        self.RoomDraw_Rightwards2x2(0x0e52, dsto);
    }

    pub(super) fn DrawObjects_LightableTorch(&mut self, dsto_x2: u16, slot: u16) {
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
        self.RoomDraw_Rightwards2x2(src_img, dsto);
    }

    #[track_caller]
    pub(super) fn Object_Fill_Nx1(&mut self, count: u16, src: usize, dsto: u16) {
        let tile = self.tile_word(src, 0);
        for i in 0..count {
            self.room_write_current(dsto + i, tile);
        }
    }

    pub(super) fn Object_Hole(&mut self, src: usize, dsto: u16, width: u8, height: u8) {
        self.dungeon_room_load_mut().set_draw_width_indicator(width);
        self.dungeon_room_load_mut()
            .set_draw_height_indicator(height);
        self.Object_SizeAtoAplus15(4);
        let width = self.game_state.dungeon.room_load.draw_width_indicator() as u16;
        for y in 0..width {
            self.Object_Fill_Nx1(width, src, dsto + y * 64);
        }

        let edge_src = 0x063c;
        self.room_write_current(dsto, self.tile_word(edge_src, 0));
        self.Object_Fill_Nx1(width - 2, edge_src + 2, dsto + 1);
        self.room_write_current(dsto + width - 1, self.tile_word(edge_src, 2));

        let bottom = dsto + (width - 1) * 64;
        self.room_write_current(bottom, self.tile_word(edge_src, 3));
        self.Object_Fill_Nx1(width - 2, edge_src + 8, bottom + 1);
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
    pub(super) fn room_fill_horizontal(&mut self, dsto: u16, count: u16, tile: u16) {
        for x in 0..count {
            self.room_write_current(dsto + x, tile);
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

    pub(super) fn DstoPtr(&self, d: u16) -> usize {
        self.game_state
            .dungeon
            .room_tilemaps
            .first_line_pointer_row0() as usize
            + d as usize * 2
    }

    pub(super) fn room_read_bg(&self, base: usize, dsto: u16) -> u16 {
        self.game_state
            .dungeon
            .room_tilemaps
            .room_tilemap_word(base, dsto)
    }

    pub(super) fn room_read_bg1(&self, dsto: u16) -> u16 {
        self.game_state
            .dungeon
            .room_tilemaps
            .room_tilemap_word(ROOM_BG1_TILEMAP_BASE, dsto)
    }

    pub(super) fn room_read_bg2(&self, dsto: u16) -> u16 {
        self.game_state
            .dungeon
            .room_tilemaps
            .room_tilemap_word(ROOM_BG2_TILEMAP_BASE, dsto)
    }

    #[track_caller]
    pub(super) fn room_write_bg(&mut self, base: usize, dsto: u16, tile: u16) {
        let offset = base + dsto as usize * 2;
        if replay_room_write_trace_addr(offset) {
            let caller = std::panic::Location::caller();
            eprintln!(
                "room-write addr=0x{offset:05x} base=0x{base:05x} dsto=0x{dsto:04x} tile=0x{tile:04x} caller={}:{}",
                caller.file(),
                caller.line()
            );
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

    pub(super) fn tile_word(&self, src: usize, index: usize) -> u16 {
        read_word_from_slice(
            self.asset_raw(69)
                .expect("missing predefined dungeon tile asset"),
            src + index * 2,
        )
    }

    pub(super) fn RoomDraw_FloorChunks(&mut self, base: usize, src_offset: usize) {
        let Some(tile_data) = self.asset_raw(69).map(Vec::from) else {
            return;
        };
        for &quadrant in &DUNGEON_QUADRANT_OFFSETS {
            let mut dst = quadrant;
            for _ in 0..8 {
                self.room_draw_many_32x32_blocks(base, src_offset, &tile_data, dst);
                dst += xy(0, 4) * 2;
            }
        }
    }

    pub(super) fn room_draw_many_32x32_blocks(
        &mut self,
        base: usize,
        src_offset: usize,
        tile_data: &[u8],
        dst: usize,
    ) {
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

    pub(super) fn Dungeon_UploadRoomQuadrants(&mut self) {
        self.dungeon_room_load_mut().clear_quadrant_upload_index();
        self.set_overworld_map_state(0);
        while self.game_state.dungeon.room_load.quadrant_upload_index() != 16 {
            self.TileMapPrep_NotWaterOnTag();
            self.upload_tilemap_now();
            self.Dungeon_PrepareNextRoomQuadrantUpload();
            self.upload_tilemap_now();
        }
        self.clear_pending_nmi_subroutine();
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
    }

    pub(super) fn Dungeon_PrepareNextRoomQuadrantUpload(&mut self) {
        let quadrant = self.game_state.dungeon.room_load.quadrant_upload_index() as usize;
        self.Dungeon_PrepareNextRoomQuadrantUploadFromBg2(quadrant, 0);
        self.dungeon_room_load_mut()
            .advance_quadrant_upload_index_by(4);
    }

    pub(super) fn Dungeon_PrepareNextRoomQuadrantUploadFromBg2(
        &mut self,
        quadrant_upload: usize,
        dst_bias: u8,
    ) {
        self.Dungeon_PrepareNextRoomQuadrantUploadFrom(
            ROOM_BG2_TILEMAP_BASE,
            quadrant_upload,
            dst_bias,
        );
    }

    pub(super) fn Dungeon_PrepareNextRoomQuadrantUploadFrom(
        &mut self,
        source_base: usize,
        quadrant_upload: usize,
        dst_bias: u8,
    ) {
        let table_index = ((self.screen_transition() as usize & 0x0f) + quadrant_upload)
            & ROOM_QUADRANT_UPLOAD_TABLE_MASK;
        let mut src = UPLOAD_BG_SRCS[table_index];
        let mut p = 0usize;
        loop {
            loop {
                for y in 0..4 {
                    for x in 0..2 {
                        let value = self
                            .game_state
                            .dungeon
                            .room_tilemaps
                            .room_tilemap_word_by_byte_offset(source_base, src + xy(x, y) * 2);
                        self.write_vram_upload_tilemap_word((p + y * 32 + x) * 2, value);
                    }
                }
                src += 2 * 2;
                p += 2;
                if p & 0x1f == 0 {
                    break;
                }
            }
            src += 224 * 2;
            p += 128 - 32;
            if p == 0x400 {
                break;
            }
        }
        self.set_nmi_load_target_page(UPLOAD_BG_DSTS[table_index] + dst_bias);
        self.set_pending_nmi_subroutine(1);
        self.set_core_update_disable_flag(1);
    }

    pub(super) fn WaterFlood_BuildOneQuadrantForVRAM(&mut self) {
        assert_ne!(self.game_state.dungeon.header.primary_header_tag(), 25);
        self.TileMapPrep_NotWaterOnTag();
    }

    pub(super) fn FloodDam_PrepTiles_init(&mut self) {
        self.dungeon_room_load_mut().clear_quadrant_upload_index();
        self.clear_screen_transition();
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.dungeon_room_load_mut()
            .advance_quadrant_upload_index_by(4);
        self.increment_subsubmodule();
    }

    pub(super) fn Watergate_Main_State1(&mut self) {
        self.clear_screen_transition();
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.dungeon_room_load_mut()
            .advance_quadrant_upload_index_by(4);
        self.increment_subsubmodule();
    }

    pub(super) fn Dungeon_FloodSwampWater_PrepTileMap(&mut self) {
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.dungeon_room_load_mut()
            .advance_quadrant_upload_index_by(4);
        self.increment_subsubmodule();
        if self.game_state.frame.subsubmodule == 6 {
            self.dungeon_room_load_mut().clear_quadrant_upload_index();
            self.set_subsubmodule(0);
            self.set_submodule(0);
        }
    }

    pub(super) fn Dungeon_AdjustWaterVomit(&mut self, src: usize, depth: i32) {
        let mut dsto = (self
            .game_state
            .dungeon
            .room_items
            .replacement_tile_source_pos())
        .wrapping_add(xy(0, 2) as u16);
        let mut row = 0usize;
        let mut remaining = depth;
        loop {
            for x in 0..4 {
                let tile = self.tile_word(src, row * 4 + x);
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile(dsto as usize + x, tile);
            }
            dsto = dsto.wrapping_add(xy(0, 1) as u16);
            row += 1;
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }

        let base_dsto = (self
            .game_state
            .dungeon
            .room_items
            .replacement_tile_source_pos())
        .wrapping_add(xy(0, 2) as u16);
        let mut upload = self.game_state.display.vram_upload_buffer_base();
        for i in 0..4u16 {
            let col = base_dsto.wrapping_add(i);
            let vram_addr = self.Dungeon_MapVramAddr(col);
            self.write_vram_upload_absolute_word(upload, vram_addr);
            self.write_vram_upload_absolute_word(upload + 2, 0x0980);
            for y in 0..5usize {
                let tile = self
                    .game_state
                    .dungeon
                    .room_tilemaps
                    .bg2_tile(col as usize + y * 64);
                self.write_vram_upload_absolute_word(upload + 4 + y * 2, tile);
            }
            upload += 14;
        }
        self.write_vram_upload_absolute_word(upload, 0xffff);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn FloodDam_Expand(&mut self) {
        let watergate_var1 = self.water_hdma_window_mut().increment_watergate_pointer();
        self.water_hdma_window_mut()
            .set_window_x_radius(u16::from(watergate_var1 >> 1));
        self.water_hdma_window_mut()
            .advance_watergate_window_y_radius();

        if watergate_var1 & 0x0f != 0 {
            return;
        }
        if watergate_var1 == 64 {
            self.increment_subsubmodule();
        }

        let src =
            FLOOD_DAM_EXPAND_WATERGATE_SRCS1[((watergate_var1 >> 4).wrapping_sub(1)) as usize];
        let dsto = self
            .game_state
            .display
            .water_hdma_window
            .watergate_tilemap_pos_x2()
            >> 1;
        for x in 0..10u16 {
            for y in 0..4u16 {
                let tile = self.tile_word(src, (x * 4 + y) as usize);
                self.room_write_bg2(dsto + x + y * 64, tile);
            }
        }

        let mut pos = self
            .game_state
            .display
            .water_hdma_window
            .watergate_tilemap_pos_x2();
        let mut dma_ptr = 0usize;
        for _ in 0..3 {
            dma_ptr = self.dungeon_prep_overlay_dma_watergate(dma_ptr, pos, 0x0881, 4);
            pos = pos.wrapping_add(6);
        }
        self.request_nmi_copy_packets();
    }

    pub(super) fn FloodDam_Fill(&mut self) {
        let t = self
            .water_hdma_window_mut()
            .increment_window_y_radius_byte()
            .wrapping_add(self.game_state.display.spotlight_hdma.y_upper() as u8);
        if t >= 225 {
            self.dungeon_room_load_mut().clear_quadrant_upload_index();
            self.set_submodule(0);
            self.set_subsubmodule(0);
            self.clear_window_main_sub_masks();
            self.IrisSpotlight_ResetTable();
        }
    }

    pub(super) fn TileMapPrep_NotWaterOnTag(&mut self) {
        self.Dungeon_PrepareNextRoomQuadrantUploadFrom(
            ROOM_BG1_TILEMAP_BASE,
            self.game_state.dungeon.room_load.quadrant_upload_index() as usize,
            0x10,
        );
    }

    pub(super) fn OrientLampLightCone(&mut self) {
        if !self.game_state.dungeon.torch.dungeon_dark_with_lantern()
            || self.game_state.frame.submodule == 20
        {
            return;
        }

        let a = self.game_state.player.follower_link.facing() >> 1;
        let mut idx = a;
        if self.game_state.player.follower_link.doorway_state() != 0 {
            idx = self.game_state.player.follower_link.doorway_state() & 0xfe;
            if idx != 0 {
                if a < 2 {
                    idx = idx.wrapping_add(u8::from(
                        self.game_state.player.follower_link.x().wrapping_add(8) as u8 >= 0x80,
                    ));
                } else {
                    idx = a;
                }
            } else if a >= 2 {
                idx = idx.wrapping_add(u8::from(
                    self.game_state.player.follower_link.y() as u8 >= 0x80,
                ));
            } else {
                idx = a;
            }
        }

        let idx = idx as usize;
        if idx >= 4 {
            return;
        }

        if idx < 2 {
            let h = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_sub(self.game_state.player.follower_link.x().wrapping_sub(0x77))
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_X_BASE_OFFSETS[idx]);
            self.set_bg1_x(h);

            let t = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_sub(self.game_state.player.follower_link.y().wrapping_sub(0x58))
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_Y_BASE_OFFSETS[idx])
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_ADJUSTMENTS[idx] as u16)
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_BASELINES[idx] as u16);
            let t = clamp_c_int16_to_u16(t, ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_CLAMPS[idx]);
            self.set_bg1_v_copy2(
                t.wrapping_sub(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_BASELINES[idx] as u16),
            );
        } else {
            let v = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_sub(self.game_state.player.follower_link.y().wrapping_sub(0x72))
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_Y_BASE_OFFSETS[idx]);
            self.set_bg1_y(v);

            let t = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_sub(self.game_state.player.follower_link.x().wrapping_sub(0x58))
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_X_BASE_OFFSETS[idx])
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_ADJUSTMENTS[idx] as u16)
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_BASELINES[idx] as u16);
            let t = clamp_c_int16_to_u16(t, ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_CLAMPS[idx]);
            self.set_bg1_h_copy2(
                t.wrapping_sub(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_BASELINES[idx] as u16),
            );
        }
    }

    pub(super) fn SavePalaceDeaths(&mut self) {
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

    pub(super) fn upload_tilemap_now(&mut self) {
        let target = self.game_state.display.nmi_load_target_page() as usize;
        let vram_page = NMI_VRAM_ADDRS[target];
        let dst = vram_page << 8;
        for i in 0..0x400 {
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

    pub(super) fn SetAndSaveVisitedQuadrantFlags(&mut self) {
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
        let saved = self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(room)
            | visited;
        self.save_progress_mut().set_dungeon_info_word(room, saved);
    }

    pub(super) fn Dungeon_PlayBlipAndCacheQuadrantVisits(&mut self) {
        self.set_hud_floor_changed_timer(1);
        self.set_sound_effect_2(36);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn ResetTransitionPropsAndAdvance_ResetInterface(&mut self) {
        self.set_overworld_map_state(0);
        self.ResetTransitionPropsAndAdvanceSubmodule();
    }

    pub(super) fn ResetTransitionPropsAndAdvanceSubmodule(&mut self) {
        self.clear_mosaic_level_word();
        self.set_darkening_or_lightening_screen(0);
        self.set_countdown(0);
        self.set_mosaic_target_level(31);
        self.dungeon_room_runtime_mut().clear_reserved_gfx_config();
        self.dungeon_torch_mut().clear_lit_torches();
        if self.game_state.dungeon.torch.dungeon_dark_with_lantern() {
            self.set_color_window_selection(0x02);
            self.set_color_math_control(0xb3);
        }
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        self.Overworld_CopyPalettesToCache();
        self.increment_subsubmodule();
    }

    pub(super) fn RoomBounds_AddX_A(&mut self) {
        self.room_bounds_mut().add_x_bounds_a(0x0100);
    }

    pub(super) fn RoomBounds_AddX_B(&mut self) {
        self.room_bounds_mut().add_x_bounds_b(0x0200);
    }

    pub(super) fn RoomBounds_SubX_B(&mut self) {
        self.room_bounds_mut().add_x_bounds_b(0xfe00);
    }

    pub(super) fn RoomBounds_SubX_A(&mut self) {
        self.room_bounds_mut().add_x_bounds_a(0xff00);
    }

    pub(super) fn RoomBounds_AddY_A(&mut self) {
        self.room_bounds_mut().add_y_bounds_a(0x0100);
    }

    pub(super) fn RoomBounds_AddY_B(&mut self) {
        self.room_bounds_mut().add_y_bounds_b(0x0200);
    }

    pub(super) fn RoomBounds_SubY_B(&mut self) {
        self.room_bounds_mut().add_y_bounds_b(0xfe00);
    }

    pub(super) fn RoomBounds_SubY_A(&mut self) {
        self.room_bounds_mut().add_y_bounds_a(0xff00);
    }

    pub(super) fn AdjustQuadrantAndCamera_right(&mut self) {
        self.follower_link_state_mut().toggle_quadrant_x();
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_AddX_A();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn AdjustQuadrantAndCamera_left(&mut self) {
        self.follower_link_state_mut().toggle_quadrant_x();
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_SubX_A();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn AdjustQuadrantAndCamera_down(&mut self) {
        self.follower_link_state_mut().toggle_quadrant_y();
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_AddY_A();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn AdjustQuadrantAndCamera_up(&mut self) {
        self.follower_link_state_mut().toggle_quadrant_y();
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_SubY_A();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn Dungeon_AdjustQuadrant(&mut self) {
        let quadrant_y = self.game_state.player.follower_link.quadrant_y();
        let quadrant_x = self.game_state.player.follower_link.quadrant_x();
        // dung_layout_and_starting_quadrant is owned solely by DungeonRoomParserState
        // (it is a uint16 set by the room-object parser, C dungeon.c:2661); the low
        // byte feeds the composite key (C dungeon.c:8239).
        let layout = self
            .game_state
            .dungeon
            .room_parser
            .room_layout_and_starting_quadrant() as u8;
        self.dungeon_room_load_mut()
            .update_layout_quadrant_key(layout, quadrant_y, quadrant_x);
    }

    pub(super) fn Dungeon_AdjustForRoomLayout(&mut self) {
        self.Dungeon_AdjustQuadrant();
        let flags =
            LAYOUT_QUADRANT_FLAGS[self.game_state.dungeon.room_load.layout_quadrant_key() as usize];
        let horizontal_mask = self.game_state.player.follower_link.quadrant_x_mask();
        let vertical_mask = self.game_state.player.follower_link.quadrant_y_mask();
        let blast_wall_x_open = self.game_state.dungeon.room_effects.blast_wall_x_open();
        let blast_wall_y_open = self.game_state.dungeon.room_effects.blast_wall_y_open();
        self.apply_dungeon_layout_quadrant_fullsize(
            flags,
            horizontal_mask,
            vertical_mask,
            blast_wall_x_open,
            blast_wall_y_open,
        );
        let reset_xy_flags = self.game_state.dungeon.door_setup.reset_xy_check_flags();
        self.apply_reset_xy_quadrant_overrides(reset_xy_flags);
    }

    pub(super) fn Dung_SaveDataForCurrentRoom(&mut self) {
        let saved = (self.game_state.dungeon.savegame_state.savegame_state_bits() >> 4)
            | (self.game_state.dungeon.doors.opened_doors() & 0xf000)
            | self.game_state.dungeon.room_load.quadrants_visited();
        let room = self.game_state.world.location.dungeon_room() as usize;
        self.save_progress_mut().set_dungeon_info_word(room, saved);
    }

    pub(super) fn SaveQuadrantsToSram(&mut self) {
        let room = self.game_state.world.location.dungeon_room() as usize;
        let saved = self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(room)
            | self.game_state.dungeon.room_load.quadrants_visited();
        self.save_progress_mut().set_dungeon_info_word(room, saved);
    }

    pub(super) fn Dung_HandleExitToOverworld(&mut self) {
        self.SaveDungeonKeys();
        self.SaveQuadrantsToSram();
        self.set_saved_module_for_menu(8);
        self.set_main_module(15);
        self.set_submodule(0);
        self.set_subsubmodule(0);
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
    }

    pub(super) fn Dungeon_FlagRoomData_Quadrants(&mut self) {
        let player_quadrant_y = self.game_state.player.follower_link.quadrant_y();
        let player_quadrant_x = self.game_state.player.follower_link.quadrant_x();
        let index = self
            .game_state
            .world
            .transient
            .dungeon_quadrant_visit_index(player_quadrant_y, player_quadrant_x);
        self.dungeon_room_load_mut()
            .or_quadrants_visited(DUNGEON_QUADRANT_VISITING_FLAGS[index]);
        self.Dung_SaveDataForCurrentRoom();
    }

    pub(super) fn DungeonTransition_AdjustCamera_X(&mut self, arg: u8) {
        let index = arg as usize * 2;
        self.set_left_right_scroll_target(
            DUNGEON_TRANSITION_ADJUST_CAMERA_X_UP_DOWN_SCROLL_VALUES[index],
        );
        self.set_left_right_scroll_target_end(
            DUNGEON_TRANSITION_ADJUST_CAMERA_X_UP_DOWN_SCROLL_VALUES[index + 1],
        );
    }

    pub(super) fn DungeonTransition_AdjustCamera_Y(&mut self, arg: u8) {
        let index = arg as usize;
        self.set_up_down_scroll_target(
            DUNGEON_TRANSITION_ADJUST_CAMERA_Y_UP_DOWN_SCROLL_VALUES[index],
        );
        self.set_up_down_scroll_target_end(
            DUNGEON_TRANSITION_ADJUST_CAMERA_Y_UP_DOWN_SCROLL_VALUES[index + 1],
        );
    }

    pub(super) fn HandleEdgeTransition_AdjustCameraBoundaries(&mut self, arg: u8) {
        self.set_screen_transition(arg);
        if self.game_state.player.follower_link.direction() & 3 != 0 {
            let mut index = if self.game_state.player.follower_link.direction() & 1 != 0 {
                0
            } else {
                2
            };
            if self.game_state.player.follower_link.quadrant_x() != 0 {
                index += 1;
            }
            self.set_camera_x_coord_scroll_low(
                HANDLE_EDGE_TRANSITION_ADJUST_CAMERA_BOUNDARIES_CAMERA_X_BOUNDS[index],
            );
            self.set_camera_x_coord_scroll_hi(
                HANDLE_EDGE_TRANSITION_ADJUST_CAMERA_BOUNDARIES_CAMERA_X_BOUNDS[index]
                    .wrapping_add(2),
            );
        } else {
            let mut index = if self.game_state.player.follower_link.direction() & 4 != 0 {
                0
            } else {
                2
            };
            if self.game_state.player.follower_link.quadrant_y() != 0 {
                index += 1;
            }
            self.set_camera_y_coord_scroll_low(
                HANDLE_EDGE_TRANSITION_ADJUST_CAMERA_BOUNDARIES_CAMERA_Y_BOUNDS[index],
            );
            self.set_camera_y_coord_scroll_hi(
                HANDLE_EDGE_TRANSITION_ADJUST_CAMERA_BOUNDARIES_CAMERA_Y_BOUNDS[index]
                    .wrapping_add(2),
            );
        }
    }

    pub(super) fn Dungeon_StartInterRoomTrans_Left(&mut self) {
        assert_eq!(self.game_state.frame.submodule, 0);
        let quadrant_x = self.follower_link_state_mut().toggle_quadrant_x();
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_SubX_A();
        self.Dung_SaveDataForCurrentRoom();
        self.DungeonTransition_AdjustCamera_X(quadrant_x ^ 1);
        self.HandleEdgeTransition_AdjustCameraBoundaries(3);
        self.set_submodule(1);
        if self.game_state.player.follower_link.quadrant_x() != 0 {
            self.RoomBounds_SubX_B();
            let previous_room = self.game_state.world.location.dungeon_room_index();
            self.dungeon_room_tracking_mut()
                .set_room_index_prev(previous_room);
            if self.game_state.player.follower_link.tile_below() & 0xcf == 0x89 {
                let room = self.game_state.dungeon.header.travel_destination(3);
                self.set_dungeon_room_index(room);
                self.Dungeon_AdjustForTeleportDoors(
                    self.game_state
                        .world
                        .location
                        .dungeon_room_index()
                        .wrapping_add(1),
                    0xff,
                );
            } else {
                if self.game_state.world.location.dungeon_room_index()
                    != self.game_state.dungeon.room_tracking.room_index2()
                {
                    let previous_room = self.game_state.dungeon.room_tracking.room_index2();
                    self.dungeon_room_tracking_mut()
                        .set_room_index_prev(previous_room);
                    self.Dungeon_AdjustAfterSpiralStairs();
                }
                self.decrement_dungeon_room_index_by(1);
            }
            self.finish_super_tile_transition_room_side_effects();
        }
        self.set_room_transitioning_flags(0);
        self.update_quadrant_fullsize_y_after_transition();
    }

    pub(super) fn Dung_StartInterRoomTrans_Left_Plus(&mut self) {
        let x = self.game_state.player.follower_link.x().wrapping_sub(8);
        self.follower_link_state_mut().set_x(x);
        self.Dungeon_StartInterRoomTrans_Left();
    }

    pub(super) fn Dungeon_StartInterRoomTrans_Right(&mut self) {
        assert_eq!(self.game_state.frame.submodule, 0);
        let quadrant_x = self.follower_link_state_mut().toggle_quadrant_x();
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_AddX_A();
        self.Dung_SaveDataForCurrentRoom();
        self.DungeonTransition_AdjustCamera_X(quadrant_x);
        self.HandleEdgeTransition_AdjustCameraBoundaries(2);
        self.set_submodule(1);
        if self.game_state.player.follower_link.quadrant_x() == 0 {
            self.RoomBounds_AddX_B();
            let previous_room = self.game_state.world.location.dungeon_room_index();
            self.dungeon_room_tracking_mut()
                .set_room_index_prev(previous_room);
            if self.game_state.player.follower_link.tile_below() & 0xcf == 0x89 {
                let room = self.game_state.dungeon.header.travel_destination(4);
                self.set_dungeon_room_index(room);
                self.Dungeon_AdjustForTeleportDoors(
                    self.game_state
                        .world
                        .location
                        .dungeon_room_index()
                        .wrapping_sub(1),
                    1,
                );
            } else {
                if self.game_state.world.location.dungeon_room_index()
                    != self.game_state.dungeon.room_tracking.room_index2()
                {
                    let previous_room = self.game_state.dungeon.room_tracking.room_index2();
                    self.dungeon_room_tracking_mut()
                        .set_room_index_prev(previous_room);
                    self.Dungeon_AdjustAfterSpiralStairs();
                }
                self.increment_dungeon_room_index_by(1);
            }
            self.finish_super_tile_transition_room_side_effects();
        }
        self.set_room_transitioning_flags(0);
        self.update_quadrant_fullsize_y_after_transition();
    }

    pub(super) fn Dungeon_StartInterRoomTrans_Up(&mut self) {
        assert_eq!(self.game_state.frame.submodule, 0);
        let quadrant_y = self.follower_link_state_mut().toggle_quadrant_y();
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_SubY_A();
        self.Dung_SaveDataForCurrentRoom();
        self.DungeonTransition_AdjustCamera_Y(quadrant_y ^ 2);
        self.HandleEdgeTransition_AdjustCameraBoundaries(1);
        self.set_submodule(1);
        if self.game_state.player.follower_link.quadrant_y() != 0 {
            self.RoomBounds_SubY_B();
            let previous_room = self.game_state.world.location.dungeon_room_index();
            self.dungeon_room_tracking_mut()
                .set_room_index_prev(previous_room);
            if self.game_state.player.follower_link.tile_below() == 0x8e {
                self.Dung_HandleExitToOverworld();
                return;
            }
            if self.game_state.world.location.dungeon_room_index() == 0 {
                self.SaveDungeonKeys();
                self.set_main_module(25);
                self.set_submodule(0);
                self.set_subsubmodule(0);
                return;
            }
            if self.game_state.dungeon.room_tracking.room_index2()
                == self.game_state.world.location.dungeon_room_index()
            {
                let previous_room = self.game_state.dungeon.room_tracking.room_index2();
                self.dungeon_room_tracking_mut()
                    .set_room_index_prev(previous_room);
                self.Dungeon_AdjustAfterSpiralStairs();
            }
            self.decrement_dungeon_room_index_by(0x10);
            self.finish_super_tile_transition_room_side_effects();
        }
        self.set_room_transitioning_flags(0);
        self.update_quadrant_fullsize_x_after_transition();
    }

    pub(super) fn Dungeon_StartInterRoomTrans_Down(&mut self) {
        assert_eq!(self.game_state.frame.submodule, 0);
        let quadrant_y = self.follower_link_state_mut().toggle_quadrant_y();
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_AddY_A();
        self.Dung_SaveDataForCurrentRoom();
        self.DungeonTransition_AdjustCamera_Y(quadrant_y);
        self.HandleEdgeTransition_AdjustCameraBoundaries(0);
        self.set_submodule(1);
        if self.game_state.player.follower_link.quadrant_y() == 0 {
            self.RoomBounds_AddY_B();
            let previous_room = self.game_state.world.location.dungeon_room_index();
            self.dungeon_room_tracking_mut()
                .set_room_index_prev(previous_room);
            if self.game_state.player.follower_link.tile_below() == 0x8e {
                self.Dung_HandleExitToOverworld();
                return;
            }
            if self.game_state.world.location.dungeon_room_index()
                != self.game_state.dungeon.room_tracking.room_index2()
            {
                let previous_room = self.game_state.dungeon.room_tracking.room_index2();
                self.dungeon_room_tracking_mut()
                    .set_room_index_prev(previous_room);
                self.Dungeon_AdjustAfterSpiralStairs();
            }
            self.increment_dungeon_room_index_by(0x10);
            self.finish_super_tile_transition_room_side_effects();
        }
        self.set_room_transitioning_flags(0);
        self.update_quadrant_fullsize_x_after_transition();
    }

    fn finish_super_tile_transition_room_side_effects(&mut self) {
        self.set_submodule(2);
        if self.game_state.world.transient.room_transitioning_flags() & 1 != 0 {
            self.follower_link_state_mut().toggle_lower_level_state();
            self.follower_link_state_mut().mirror_lower_level_state();
        }
        if self.game_state.world.transient.room_transitioning_flags() & 2 != 0 {
            self.save_progress_mut().xor_palace_index_x2(2);
        }
    }

    fn update_quadrant_fullsize_x_after_transition(&mut self) {
        let flags =
            LAYOUT_QUADRANT_FLAGS[self.game_state.dungeon.room_load.layout_quadrant_key() as usize];
        let mask = self.game_state.player.follower_link.quadrant_x_mask();
        let blast_wall_x_open = self.game_state.dungeon.room_effects.blast_wall_x_open();
        self.apply_dungeon_layout_horizontal_fullsize(flags, mask, blast_wall_x_open);
    }

    fn update_quadrant_fullsize_y_after_transition(&mut self) {
        let flags =
            LAYOUT_QUADRANT_FLAGS[self.game_state.dungeon.room_load.layout_quadrant_key() as usize];
        let mask = self.game_state.player.follower_link.quadrant_y_mask();
        let blast_wall_y_open = self.game_state.dungeon.room_effects.blast_wall_y_open();
        self.apply_dungeon_layout_vertical_fullsize(flags, mask, blast_wall_y_open);
    }

    pub(super) fn HandleEdgeTransitionMovementEast_RightBy8(&mut self) {
        let x = self.game_state.player.follower_link.x().wrapping_add(8);
        self.follower_link_state_mut().set_x(x);
        self.Dungeon_StartInterRoomTrans_Right();
    }

    pub(super) fn HandleEdgeTransitionMovementSouth_DownBy16(&mut self) {
        let y = self.game_state.player.follower_link.y().wrapping_add(16);
        self.follower_link_state_mut().set_y(y);
        self.Dungeon_StartInterRoomTrans_Down();
    }

    pub(super) fn Dungeon_Store2x2(
        &mut self,
        pos: u16,
        t0: u16,
        t1: u16,
        t2: u16,
        t3: u16,
        attr: u8,
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
                .set_bg2_attr(tile_pos as usize, attr);
        }

        let dst = self.game_state.display.current_vram_upload_data_address();
        for (i, (&tile_pos, &tile)) in positions.iter().zip(tiles.iter()).enumerate() {
            let base = dst + i * 6;
            let addr = self.Dungeon_MapVramAddr(tile_pos);
            self.write_vram_upload_single_tile_stripe_packet(base, addr, tile);
        }
        self.write_vram_upload_tile_stripe_sentinel(dst + 24);
        self.advance_vram_upload_cursor_by(24);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn Dungeon_UpdateTileMapWithCommonTile(&mut self, x: i32, y: i32, v: u8) {
        if v == 8 {
            self.Dungeon_PrepSpriteInducedDma(x + 16, y, v + 2);
        }
        self.Dungeon_PrepSpriteInducedDma(x, y, v);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn Dungeon_PrepSpriteInducedDma(&mut self, x: i32, y: i32, v: u8) {
        let pos = ((((y + 1) as u16) & 0x01f8) << 3) | (((x as u16) & 0x01f8) >> 3);
        let src = DUNGEON_PREP_SPRITE_INDUCED_DMA_PREP_SPRITE_INDUCED_DMA_SRCS[(v >> 1) as usize];
        let tiles = [
            self.tile_word(src, 0),
            self.tile_word(src, 1),
            self.tile_word(src, 2),
            self.tile_word(src, 3),
        ];
        let attr = self.dungeon_tile_attribute(tiles[3] as usize);
        let tile_positions = [pos, pos + 64, pos + 1, pos + 65];
        if std::env::var_os("ZELDA3_TRACE_SPRITE_DMA").is_some() {
            let target = std::env::var("ZELDA3_TRACE_SPRITE_DMA_POS")
                .ok()
                .and_then(|value| {
                    value
                        .strip_prefix("0x")
                        .or_else(|| value.strip_prefix("0X"))
                        .and_then(|hex| u16::from_str_radix(hex, 16).ok())
                        .or_else(|| value.parse::<u16>().ok())
                });
            if target.map_or(true, |target| tile_positions.contains(&target)) {
                eprintln!(
                    "R sprite_dma fc=0x{:02x} x=0x{:04x} y=0x{:04x} v=0x{:02x} pos=0x{:04x} src=0x{:04x} attr=0x{:02x} tiles={:04x},{:04x},{:04x},{:04x}",
                    self.game_state.frame.frame_counter,
                    x as u16,
                    y as u16,
                    v,
                    pos,
                    src,
                    attr,
                    tiles[0],
                    tiles[1],
                    tiles[2],
                    tiles[3]
                );
            }
        }

        for &tile_pos in &tile_positions {
            self.dungeon_bg2_attributes_mut()
                .set_bg2_attr(tile_pos as usize, attr);
        }

        for (&tile_pos, &tile) in tile_positions.iter().zip(tiles.iter()) {
            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile(tile_pos as usize, tile);
        }

        let dst = self.game_state.display.current_vram_upload_data_address();
        for (i, (&tile_pos, &tile)) in tile_positions.iter().zip(tiles.iter()).enumerate() {
            let base = dst + i * 6;
            let vram_addr = self.Dungeon_MapVramAddr(tile_pos);
            self.write_vram_upload_single_tile_stripe_packet(base, vram_addr, tile);
        }
        self.write_vram_upload_tile_stripe_sentinel(dst + 24);
        self.advance_vram_upload_cursor_by(24);
    }

    pub(super) fn Dungeon_DeleteRupeeTile(&mut self, x: u16, y: u16) {
        let pos = ((y & 0x01f8) << 3) | ((x & 0x01f8) >> 3);
        let dst = self.game_state.display.current_vram_upload_data_address();
        let tile = 0x190f;

        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile(pos as usize, tile);
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile((pos + 64) as usize, tile);

        let attr = u16::from(self.dungeon_tile_attribute(tile as usize)) * 0x0101;
        self.dungeon_bg2_attributes_mut()
            .set_bg2_attr_word(pos as usize, attr);
        self.dungeon_bg2_attributes_mut()
            .set_bg2_attr_word((pos + 64) as usize, attr);

        let vram_addr_0 = self.Dungeon_MapVramAddr(pos);
        let vram_addr_1 = self.Dungeon_MapVramAddr(pos + 64);
        self.write_vram_upload_single_tile_stripe_packet(dst, vram_addr_0, tile);
        self.write_vram_upload_single_tile_stripe_packet(dst + 6, vram_addr_1, tile);
        self.write_vram_upload_tile_stripe_sentinel(dst + 12);
        self.advance_vram_upload_cursor_by(24);

        let state = self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x1000;
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits(state);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn RoomDraw_16x16Single(&mut self, index: u8) {
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
        let attr = self.dungeon_tile_attribute(lr as usize);
        self.Dungeon_Store2x2(pos, ul, ll, ur, lr, attr);
    }

    pub(super) fn Dungeon_LiftAndReplaceLiftable(&mut self, pt: &mut Point16U) -> u8 {
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
        let attr = self.game_state.dungeon.bg2_attributes.bg2_attr(xy as usize);
        assert_eq!(attr & 0x70, 0x70);
        let attr = attr & 0x0f;
        let rt = self
            .game_state
            .dungeon
            .object_tracking
            .replacement_tile_state(attr as usize);

        if rt & 0xf0f0 == 0x1010 {
            let misc = u16::from(attr) * 2;
            self.dungeon_object_tracking_mut()
                .set_misc_object_index(misc);
            let tilemap = self
                .game_state
                .dungeon
                .object_tracking
                .object_tilemap_pos(attr as usize);
            self.RevealPotItem(xy, tilemap);
            self.RoomDraw_16x16Single(misc as u8);
            self.ManipBlock_Something(pt);
            LIFTABLE_TILE_REPLACEMENT_ITEM_CODES[(rt & 0x0f) as usize] as u8
        } else if rt & 0xf0f0 == 0x2020 {
            self.ThievesAttic_DrawLightenedHole(
                xy,
                (u16::from(attr).wrapping_sub(rt & 0x0f)).wrapping_mul(2),
                pt,
            )
        } else {
            0
        }
    }

    pub(super) fn ThievesAttic_DrawLightenedHole(
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
        self.RoomDraw_16x16Single(a as u8);
        self.RoomDraw_16x16Single(a.wrapping_add(2) as u8);
        self.RoomDraw_16x16Single(a.wrapping_add(4) as u8);
        self.RoomDraw_16x16Single(a.wrapping_add(6) as u8);
        self.ManipBlock_Something(pt);
        0x55
    }

    pub(super) fn HandleItemTileAction_Dungeon(&mut self, x: u16, y: u16) -> u8 {
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
            .bg2_attr(pos as usize);
        if tile & 0xf0 == 0x70 {
            let tile2 = self
                .game_state
                .dungeon
                .object_tracking
                .replacement_tile_state((tile & 0x0f) as usize);
            if tile2 & 0xf0f0 == 0x4040 {
                if !self.game_state.player.follower_link.item_in_hand_has(2) {
                    return 0;
                }
                self.dungeon_object_tracking_mut()
                    .set_misc_object_index(u16::from(tile & 0x0f) * 2);
                self.RoomDraw_16x16Single(
                    self.game_state.dungeon.object_tracking.misc_object_index() as u8,
                );
                self.set_sound_effect_1(0x11);
            } else if tile2 & 0xf0f0 == 0x1010 {
                self.dungeon_object_tracking_mut()
                    .set_misc_object_index(u16::from(tile & 0x0f) * 2);
                let tilemap = self
                    .game_state
                    .dungeon
                    .object_tracking
                    .object_tilemap_pos((tile & 0x0f) as usize);
                self.RevealPotItem(pos, tilemap);
                self.RoomDraw_16x16Single(
                    self.game_state.dungeon.object_tracking.misc_object_index() as u8,
                );
                let mut pt = Point16U { x: 0, y: 0 };
                self.ManipBlock_Something(&mut pt);
                self.dungeon_secret_scratch_mut().mark_graphics_kind();
                self.sprite_spawn_immediately_smashed_terrain(1, pt.x, pt.y);
                self.ancilla_add_bush_poof(pt.x, pt.y);
            }
        }
        0
    }

    pub(super) fn ManipBlock_Something(&mut self, pt: &mut Point16U) {
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
                .bg2_attr(pos6 as usize)
                & 0x0f;
            let mut k = (u16::from(j).wrapping_sub(
                self.game_state
                    .dungeon
                    .object_tracking
                    .replacement_tile_state(j as usize)
                    & 0x0f,
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

    pub(super) fn PushBlock_CheckForPit(&mut self, y: u8) {
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
        let attr = self.game_state.dungeon.bg2_attributes.bg2_attr(p as usize);
        if attr == 0x20 {
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

        if attr == 0x23 {
            let related = self.game_state.dungeon.environment.trapdoors_down() ^ 1;
            self.dungeon_environment_mut()
                .set_block_trap_related_tile(related);
            self.dungeon_object_tracking_mut()
                .set_replacement_tile_state(y, 4);
        } else {
            self.dungeon_object_tracking_mut()
                .set_replacement_tile_state(y, 0xffff);
        }
        self.Dungeon_Store2x2(p, 0x0922, 0x0932, 0x0923, 0x0933, 0x27);
    }

    pub(super) fn PushBlock_Slide(&mut self, j: u8) {
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
        self.PushBlock_ApplyVelocity(i as u8);
        let y = self.game_state.player.pushed_block.y(i);
        let x = self.game_state.player.pushed_block.x(i);
        self.PushBlock_HandleCollision(i as u8, x, y);
    }

    pub(super) fn PushBlock_HandleFalling(&mut self, y: u8) {
        let y = (y >> 1) as usize;
        if !(self.pushed_block_mut().decrement_animation_timer() as i8).is_negative() {
            return;
        }
        self.pushed_block_mut().reset_animation_timer();
        if self.pushed_block_mut().advance_animation_mode() == 4 {
            self.dungeon_object_tracking_mut()
                .clear_replacement_tile_state_low(y);
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

    pub(super) fn PushBlock_ApplyVelocity(&mut self, i: u8) {
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
            let state = self
                .game_state
                .dungeon
                .object_tracking
                .replacement_tile_state(j)
                .wrapping_add(1);
            self.dungeon_object_tracking_mut()
                .set_replacement_tile_state(j, state);
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

    pub(super) fn PushBlock_HandleCollision(&mut self, i: u8, x: u16, y: u16) {
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

    pub(super) fn Dungeon_MapVramAddr(&self, pos: u16) -> u16 {
        self.Dungeon_MapVramAddrNoSwap(pos).swap_bytes()
    }

    pub(super) fn Dungeon_MapVramAddrNoSwap(&self, pos: u16) -> u16 {
        let pos = pos.wrapping_mul(2);
        ((pos & 0x40) << 4) | ((pos & 0x303f) >> 1) | ((pos & 0x0f80) >> 2)
    }

    pub(super) fn Dungeon_GetTeleMsg(&self, room: usize) -> u16 {
        self.asset_u16(9, room)
    }

    pub(super) fn GetDungPalInfo(&self, idx: usize) -> DungPalInfo {
        DUNG_PAL_INFOS.get(idx).copied().unwrap_or_default()
    }

    pub(super) fn Dungeon_IsPitThatHurtsPlayer(&self) -> bool {
        let room = self.game_state.world.location.dungeon_room();
        let Some(data) = self.asset_raw(10) else {
            return false;
        };
        data.chunks_exact(2)
            .any(|entry| read_word_from_slice(entry, 0) == room)
    }

    pub(super) fn Door_Up_EntranceDoor(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Up_EntranceDoor assert");
    }

    pub(super) fn Door_Down_EntranceDoor(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Down_EntranceDoor assert");
    }

    pub(super) fn Door_Left_EntranceDoor(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Left_EntranceDoor assert");
    }

    pub(super) fn Door_Right_EntranceDoor(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Right_EntranceDoor assert");
    }

    pub(super) fn Door_Draw_Helper4(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, dsto);
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

    pub(super) fn GetDefaultRoomLayout(&self, index: usize) -> Option<&[u8]> {
        self.default_room_layout(index)
    }

    pub(super) fn GetDungeonRoomLayout(&self, room: usize) -> Option<&[u8]> {
        self.dungeon_room_layout(room)
    }

    pub(super) fn Dung_TagRoutine_0x22_0x3B(&mut self, k: usize, j: u8) {
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0100 != 0 {
            self.dungeon_header_mut().clear_header_tag(k);
            self.dungeon_room_load_mut().set_overlay_to_load(j);
            self.dungeon_room_load_mut().set_load_ptr_offset(0);
            self.set_subsubmodule(0);
            self.set_sound_effect_2(0x1b);
            self.set_submodule(3);
        }
    }

    pub(super) fn Dung_TagRoutine_0x1B(&mut self, _k: usize) {}

    pub(super) fn RoomTag_NorthWestTrigger(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 == 0
            && self.game_state.player.follower_link.y() & 0x0100 == 0
        {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn RoomTag_Holes0(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(1);
    }

    pub(super) fn Dung_TagRoutine_0x23(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(3);
    }

    pub(super) fn Dung_TagRoutine_0x2A(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 != 0
            && self.game_state.player.follower_link.y() & 0x0100 == 0
        {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x2B(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 == 0
            && self.game_state.player.follower_link.y() & 0x0100 != 0
        {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x2C(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 != 0
            && self.game_state.player.follower_link.y() & 0x0100 != 0
        {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x2D(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 == 0 {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x2E(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 != 0 {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x2F(&mut self, k: usize) {
        if self.game_state.player.follower_link.y() & 0x0100 == 0 {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x30(&mut self, k: usize) {
        if self.game_state.player.follower_link.y() & 0x0100 != 0 {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x34(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(6);
    }

    pub(super) fn Dung_TagRoutine_0x35(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(8);
    }

    pub(super) fn Dung_TagRoutine_0x36(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(10);
    }

    pub(super) fn Dung_TagRoutine_0x37(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(12);
    }

    pub(super) fn Dung_TagRoutine_0x39(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(14);
    }

    pub(super) fn Dung_TagRoutine_0x3A(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(16);
    }

    pub(super) fn Dung_TagRoutine_Func2(&mut self, mut av: u8) {
        self.dungeon_room_load_mut()
            .set_overlay_to_load_if_empty(av);

        let mut yv = 0;
        if self.RoomTag_CheckForPressedSwitch(&mut yv) {
            av = av.wrapping_add(yv);
            if av != self.game_state.dungeon.room_load.overlay_to_load() {
                self.dungeon_room_load_mut().set_overlay_to_load(av);
                self.dungeon_room_load_mut().set_load_ptr_offset(0);
                self.set_subsubmodule(0);
                self.set_sound_effect_2(27);
                self.set_submodule(3);
                self.dungeon_room_effects_mut()
                    .toggle_moving_wall_torch_blink_phase();
                self.Dungeon_RestoreStarTileChr();
            }
        }
    }

    pub(super) fn RoomTag_ChestHoles0(&mut self, k: usize) {
        self.Dung_TagRoutine_0x22_0x3B(k, 0);
    }

    pub(super) fn Dung_TagRoutine_0x3B(&mut self, k: usize) {
        self.Dung_TagRoutine_0x22_0x3B(k, 0x12);
    }

    pub(super) fn RoomTag_Holes2(&mut self, k: usize) {
        let mut yv = 0;
        if !self.RoomTag_CheckForPressedSwitch(&mut yv) {
            return;
        }

        self.dungeon_header_mut().clear_header_tag(k);
        self.dungeon_room_load_mut().set_overlay_to_load(5);
        self.dungeon_room_load_mut().set_load_ptr_offset(0);
        self.set_subsubmodule(0);
        self.set_sound_effect_2(0x1b);
        self.set_submodule(3);
    }

    pub(super) fn RoomTag_QuadrantTrigger(&mut self, k: usize) {
        let tag = self.game_state.dungeon.header.header_tag(k);
        if tag >= 0x0b {
            if tag >= 0x29 {
                if self.sprite_check_if_screen_is_clear() {
                    self.RoomTag_OperateChestReveal(k);
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
            self.Dung_TagRoutine_TrapdoorsUp();
        }
    }

    pub(super) fn RoomTag_RoomTrigger(&mut self, k: usize) {
        if self.game_state.dungeon.header.header_tag(k) == 10 {
            if self.sprite_check_if_room_is_clear() {
                self.Dung_TagRoutine_TrapdoorsUp();
            }
        } else if self.sprite_check_if_room_is_clear() {
            self.RoomTag_OperateChestReveal(k);
        }
    }

    pub(super) fn RoomTag_RekillableBoss(&mut self, k: usize) {
        if self.sprite_check_if_room_is_clear() {
            self.follower_link_state_mut().clear_menu_block();
            self.dungeon_header_mut().clear_header_tag(k);
        }
    }

    pub(super) fn RoomTag_RoomTrigger_BlockDoor(&mut self, _k: usize) {
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

    pub(super) fn RoomTag_PrizeTriggerDoorDoor(&mut self, k: usize) {
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

    pub(super) fn RoomTag_TorchPuzzleDoor(&mut self, _k: usize) {
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

    pub(super) fn RoomTag_Switch_ExplodingWall(&mut self, k: usize) {
        let mut yv = 0;
        if self.RoomTag_MaybeCheckShutters(&mut yv) {
            self.Dung_TagRoutine_BlastWallStuff(k);
        }
    }

    pub(super) fn RoomTag_PullSwitchExplodingWall(&mut self, k: usize) {
        if self
            .game_state
            .dungeon
            .environment
            .water_puzzle_state_changed()
            != 0
        {
            self.Dung_TagRoutine_BlastWallStuff(k);
        }
    }

    pub(super) fn Dung_TagRoutine_BlastWallStuff(&mut self, k: usize) {
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

        self.dungeon_room_effects_mut()
            .set_blast_wall_message_direction(u16::from(
                DUNG_TAG_ROUTINE_BLAST_WALL_STUFF_BLAST_WALL_MESSAGE_DIRECTION_BY_QUADRANT
                    [i as usize],
            ));
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
        self.dungeon_room_effects_mut()
            .set_blast_wall_message_position(x, y);
        self.set_sound_effect_2(27);
        self.dungeon_room_effects_mut()
            .set_crush_wall_progress_low(1);
        // C writes the blast-wall direction/center into the MESSAGING_BUF_DUNGEON region
        // (0x1c/0x1a/0x18 = BLAST_WALL_DIRECTION/CENTER_X/CENTER_Y) raw at trigger time, and
        // ancilla_add_blast_wall reads them back via the entrance-effects native the same
        // frame. The dungeon room-effects native gates its projection of those bytes on the
        // wall being open (it is not yet), so the direction/center never reached RAM and the
        // read picked up the stale gfx-staging-buffer leftover. Write them through to RAM here
        // and resync the entrance-effects native that owns the read path.
        crate::types::write_le_u16(&mut self.ram, MESSAGING_BUF_DUNGEON + 0x18, y);
        crate::types::write_le_u16(&mut self.ram, MESSAGING_BUF_DUNGEON + 0x1a, x);
        crate::types::write_le_u16(
            &mut self.ram,
            MESSAGING_BUF_DUNGEON + 0x1c,
            u16::from(
                DUNG_TAG_ROUTINE_BLAST_WALL_STUFF_BLAST_WALL_MESSAGE_DIRECTION_BY_QUADRANT
                    [i as usize],
            ),
        );
        self.game_state
            .effects
            .reload_entrance_effects_from_ram(&self.ram);
        self.ancilla_add_blast_wall();
    }

    pub(super) fn RoomTag_WaterOn(&mut self, _k: usize) {
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
            self.dungeon_moving_floor_mut().set_floor_y_offset_low(1);
            // 0x424 is mode-reused: DUNG_FLOOR_Y_OFFS (written just above) AND TURN_ON_OFF_WATER_CTR.
            // In RoomTag_WaterOn the byte is the water transition counter (C writes the single byte
            // =1). Keep the water-counter native coherent too, or its DungeonEnvironmentState
            // projection re-stamps the stale frame-start 0 over the floor-offset write (f606590).
            self.dungeon_environment_mut()
                .set_water_transition_counter(1);
            self.dungeon_header_mut().clear_header_tag(1);
            let save_bits = self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x0800;
            self.dungeon_savegame_state_mut()
                .set_savegame_state_bits(save_bits);
            self.dungeon_environment_mut()
                .clear_water_puzzle_state_changed();
            self.dungeon_room_load_mut().clear_quadrant_upload_index();
        }
    }

    pub(super) fn RoomTag_WaterOff(&mut self, _k: usize) {
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
        self.dungeon_environment_mut()
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

    pub(super) fn RoomTag_WaterGate(&mut self, _k: usize) {
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
        let dung_info = self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(0x28)
            | 0x0100;
        self.save_progress_mut()
            .set_dungeon_info_word(0x28, dung_info);

        self.RoomTag_OperateWaterFlooring();
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

    pub(super) fn RoomTag_OperateWaterFlooring(&mut self) {
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

    pub(super) fn RoomTag_GetHeartForPrize(&mut self, k: usize) {
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
            self.dungeon_room_effects_mut()
                .request_moving_wall_torch_update();
            if self.ancilla_spawn_falling_prize(
                ROOM_TAG_GET_HEART_FOR_PRIZE_BOSS_FINISHED_FALLING_ITEM[palace],
            ) < 0
            {
                return;
            }
        }
        self.dungeon_header_mut().clear_header_tag(k);
    }

    pub(super) fn RoomTag_Agahnim(&mut self, _k: usize) {
        if self.game_state.world.overworld.event_info.event_info(0x5b) & 0x20 == 0
            && self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 != 0
        {
            self.Palette_RevertTranslucencySwap();
            self.dungeon_header_mut().clear_header_tag(0);
            self.prepare_dungeon_exit_from_boss_fight();
        }
    }

    pub(super) fn RoomTag_GanonDoor(&mut self, _tagidx: usize) {
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

    pub(super) fn RoomTag_SwitchTrigger_HoldDoor(&mut self, _k: usize) {
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
                        && {
                            let mut tmp = 0;
                            !self.RoomTag_CheckForPressedSwitch(&mut tmp)
                        },
                );
            }
            if self
                .game_state
                .dungeon
                .object_tracking
                .replacement_tile_state((i >> 1) as usize)
                == 5
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
                        && {
                            let mut tmp = 0;
                            !self.RoomTag_CheckForPressedSwitch(&mut tmp)
                        },
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

    pub(super) fn RoomTag_SwitchTrigger_ToggleDoor(&mut self, _k: usize) {
        let mut attr = 0;
        if !self.game_state.dungeon.doors.door_switch_triggered() {
            if self.RoomTag_MaybeCheckShutters(&mut attr) {
                self.dungeon_doors_mut().clear_current_door_pos();
                self.dungeon_doors_mut().clear_door_animation_step();
                self.set_sound_effect_2(0x25);
                self.PushPressurePlate(attr);
                let down = self.game_state.dungeon.environment.trapdoors_down() ^ 1;
                self.dungeon_environment_mut().set_trapdoors_down(down);
                self.dungeon_doors_mut().mark_door_switch_triggered();
            }
        } else if !self.RoomTag_MaybeCheckShutters(&mut attr) {
            self.dungeon_doors_mut().clear_door_switch_triggered();
        }
    }

    pub(super) fn PushPressurePlate(&mut self, attr: u8) {
        self.set_submodule(5);
        if attr == 0x23
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
        if self
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_word(pos as usize)
            & 0xfe00
            != 0x2400
        {
            pos = pos.wrapping_add(1);
            self.dungeon_room_items_mut()
                .set_replacement_tile_destination_x2(pos);
        }
        self.Dungeon_UpdateTileMapWithCommonTile(
            i32::from((pos & 0x003f) << 3),
            i32::from((pos >> 3) & 0x01f8),
            0x10,
        );
    }

    pub(super) fn RoomTag_KillRoomBlock(&mut self, k: usize) {
        if self.game_state.player.follower_link.x() & 0x0100 != 0
            && self.game_state.player.follower_link.y() & 0x0100 != 0
            && self.sprite_check_if_screen_is_clear()
        {
            self.set_sound_effect_2(0x1b);
            self.dungeon_header_mut().clear_header_tag(k);
        }
    }

    pub(super) fn RoomTag_PushBlockForChest(&mut self, k: usize) {
        if self.game_state.display.bg_vram_load_mode == 0
            && self
                .game_state
                .dungeon
                .environment
                .movable_block_was_pushed()
                != 0
        {
            self.RoomTag_OperateChestReveal(k);
        }
    }

    pub(super) fn RoomTag_TriggerChest(&mut self, k: usize) {
        let mut attr = 0;
        if self.game_state.player.follower_link.blink_countdown() == 0
            && self.RoomTag_MaybeCheckShutters(&mut attr)
        {
            self.RoomTag_OperateChestReveal(k);
        }
    }

    pub(super) fn RoomTag_TorchPuzzleChest(&mut self, k: usize) {
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
            self.RoomTag_OperateChestReveal(k);
        }
    }

    pub(super) fn RoomTag_OperateChestReveal(&mut self, k: usize) {
        self.dungeon_header_mut().clear_header_tag(k);
        self.clear_vram_upload_cursor();
        self.dungeon_room_items_mut().clear_chest_reveal_cursor();

        let mut attr = 0x5858;
        loop {
            let yy = self.game_state.dungeon.room_items.chest_reveal_cursor_x2();
            let pos = (self
                .game_state
                .dungeon
                .room_items
                .chest_location_for_cursor(yy)
                >> 1)
                & 0x1fff;

            self.dungeon_bg2_attributes_mut()
                .set_bg2_attr_word(pos as usize, attr);
            self.dungeon_bg2_attributes_mut()
                .set_bg2_attr_word((pos + 64) as usize, attr);
            attr = attr.wrapping_add(0x0101);

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
                let stripe = self.RoomTag_BuildChestStripes(offset, yy);
                let base = dst + i * 6;
                self.write_vram_upload_single_tile_stripe_packet(base, stripe, tile);
            }
            self.write_vram_upload_tile_stripe_sentinel(dst + 24);
            self.advance_vram_upload_cursor_by(24);

            let next = yy.wrapping_add(2);
            self.dungeon_room_items_mut()
                .set_chest_reveal_cursor_x2(next);
            if self
                .game_state
                .dungeon
                .room_items
                .chest_reveal_cursor_reached_end(next)
            {
                break;
            }
        }

        self.dungeon_room_items_mut().clear_chest_reveal_cursor();
        self.set_sound_effect_2(26);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn RoomTag_BuildChestStripes(&self, pos: u16, y: u16) -> u16 {
        let loc = self
            .game_state
            .dungeon
            .room_items
            .chest_location_for_cursor(y);
        let pos = pos.wrapping_add(loc);
        (((pos & 0x0040) << 4) | ((pos & 0x303f) >> 1) | ((pos & 0x0f80) >> 2)).swap_bytes()
    }

    pub(super) fn RoomTag_GetTilemapCoords(&self) -> i32 {
        let pos = ((self.game_state.player.follower_link.x().wrapping_sub(1) & 0x01f8) >> 3)
            | ((self.game_state.player.follower_link.y().wrapping_add(14) & 0x01f8) << 3)
            | self
                .game_state
                .player
                .follower_link
                .lower_level_tilemap_offset();
        i32::from(pos)
    }

    pub(super) fn RoomTag_MaybeCheckShutters(&mut self, attr_out: &mut u8) -> bool {
        self.dungeon_room_items_mut()
            .clear_replacement_tile_destination();
        if self.game_state.player.follower_link.is_immobilized()
            || self.game_state.player.follower_link.has_auxiliary_state()
        {
            return false;
        }

        let p = self.RoomTag_GetTilemapCoords() as u16;
        let checks = [p, p.wrapping_add(64), p.wrapping_add(1), p.wrapping_add(65)];
        for &q in &checks {
            let t = self
                .game_state
                .dungeon
                .bg2_attributes
                .bg2_attr_word(q as usize);
            if t == 0x2323 || t == 0x2424 {
                if t != self
                    .game_state
                    .dungeon
                    .bg2_attributes
                    .bg2_attr_word((q + 64) as usize)
                {
                    return false;
                }
                *attr_out = t as u8;
                self.dungeon_room_items_mut()
                    .set_replacement_tile_destination_x2(q);
                return true;
            }
        }
        false
    }

    pub(super) fn RoomTag_CheckForPressedSwitch(&mut self, y_out: &mut u8) -> bool {
        self.dungeon_room_items_mut()
            .clear_replacement_tile_destination();
        if self.game_state.player.follower_link.is_immobilized()
            || self.game_state.player.follower_link.has_auxiliary_state()
        {
            return false;
        }

        let p = self.RoomTag_GetTilemapCoords() as u16;
        let checks = [p, p.wrapping_add(64), p.wrapping_add(1), p.wrapping_add(65)];
        for &q in &checks {
            let t = self
                .game_state
                .dungeon
                .bg2_attributes
                .bg2_attr_word(q as usize);
            if t == 0x2323 || t == 0x3a3a || t == 0x3b3b {
                if t != self
                    .game_state
                    .dungeon
                    .bg2_attributes
                    .bg2_attr_word((q + 64) as usize)
                {
                    return false;
                }
                *y_out = u8::from(t == 0x3b3b);
                self.dungeon_room_items_mut()
                    .set_replacement_tile_destination_x2(q);
                return true;
            }
        }
        false
    }

    pub(super) fn Dungeon_SetAttrForActivatedWaterOff(&mut self) {
        self.set_color_window_selection(2);
        self.set_color_math_control(0x32);
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
            self.write_attr2(dsto as usize + xy(1, 1), 0x1d1d);
            self.write_attr2(dsto as usize + xy(1, 2), 0x1d1d);
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
            self.write_attr2(dsto as usize + xy(1, 1), 0x1d1d);
            self.write_attr2(dsto as usize + xy(1, 2), 0x1d1d);
            j += 2;
        }

        self.increment_cgram_update_flag();
        self.increment_subsubmodule();
    }

    pub(super) fn Dungeon_SetAttrForActivatedWater(&mut self) {
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
            self.write_attr2(dsto, 0x0003);
            self.write_attr2(dsto + 2, 0x0300);
            self.write_attr1(dsto, 0x0a03);
            self.write_attr1(dsto + 2, 0x030a);
            self.write_attr2(dsto + xy(0, 1), 0x0808);
            self.write_attr2(dsto + xy(2, 1), 0x0808);
            self.write_attr1(dsto + xy(0, 1), 0x0808);
            self.write_attr1(dsto + xy(2, 1), 0x0808);
            self.write_attr1(dsto + xy(0, 2), 0x0808);
            self.write_attr1(dsto + xy(2, 2), 0x0808);
            self.write_attr1(dsto + xy(0, 3), 0x0808);
            self.write_attr1(dsto + xy(2, 3), 0x0808);
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
            self.write_attr2(dsto + xy(0, 3), 0x0003);
            self.write_attr2(dsto + xy(2, 3), 0x0300);
            self.write_attr1(dsto + xy(0, 3), 0x0a03);
            self.write_attr1(dsto + xy(2, 3), 0x030a);
            self.write_attr2(dsto + xy(0, 2), 0x0808);
            self.write_attr2(dsto + xy(2, 2), 0x0808);
            self.write_attr1(dsto, 0x0808);
            self.write_attr1(dsto + 2, 0x0808);
            self.write_attr1(dsto + xy(0, 1), 0x0808);
            self.write_attr1(dsto + xy(2, 1), 0x0808);
            self.write_attr1(dsto + xy(0, 2), 0x0808);
            self.write_attr1(dsto + xy(2, 2), 0x0808);
            j += 2;
        }

        self.set_submodule(0);
        self.clear_nmi_update_latch();
        self.set_subsubmodule(0);
    }

    pub(super) fn Sprite_HandlePushedBlocks_One(&mut self, i: usize) {
        self.oam_allocate_from_region_b(4);

        let y = self
            .game_state
            .player
            .pushed_block
            .y(i)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
            .wrapping_sub(1);
        let x = self
            .game_state
            .player
            .pushed_block
            .x(i)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());

        if self.game_state.player.pushed_block.animation_mode() < 3 {
            self.oam_state_mut()
                .write_current_entry_with_extended(x as u8, y as u8, 12, 0x20, 2);
        }
    }

    pub(super) fn Object_Draw_DoorLeft_3x4(&mut self, src: u16, door: usize) {
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

    pub(super) fn Object_Draw_DoorRight_3x4(&mut self, src: u16, door: usize) {
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

    pub(super) fn GetDoorDrawDataIndex_North_clean_door_index(&mut self, door: usize) {
        self.GetDoorDrawDataIndex_North(door, door);
    }

    pub(super) fn DoorDoorStep1_North(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = self.game_state.dungeon.doors.door_tilemap_address(door) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x1fff) >= DOOR_POSITION_UP[6] as i32 {
            pos -= 0x500;
            if (self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe) >= 0x42 {
                pos -= 0x300;
            }
            self.GetDoorDrawDataIndex_South(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.Dungeon_LoadSingleDoorAttribute(door ^ 8);
        }
        self.GetDoorDrawDataIndex_North(door, door & 7);
        dma_ptr
    }

    pub(super) fn GetDoorDrawDataIndex_North(&mut self, door: usize, r4_door: usize) {
        let door_type = self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe;
        let mut x = self.game_state.dungeon.doors.door_open_counter_low() as usize;
        if x == 0 || x == 4 {
            self.DrawDoorToTileMap_North(door, r4_door);
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
        self.Object_Draw_DoorUp_4x3(src, door);
    }

    pub(super) fn DrawDoorToTileMap_North(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_UP[index];
        self.Object_Draw_DoorUp_4x3(src, door);
    }

    pub(super) fn Object_Draw_DoorUp_4x3(&mut self, src: u16, door: usize) {
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

    pub(super) fn GetDoorDrawDataIndex_South_clean_door_index(&mut self, door: usize) {
        self.GetDoorDrawDataIndex_South(door, door);
    }

    pub(super) fn DoorDoorStep1_South(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = self.game_state.dungeon.doors.door_tilemap_address(door) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x1fff) < DOOR_POSITION_DOWN[9] as i32 {
            pos += 0x500;
            if (self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe) >= 0x42 {
                pos += 0x300;
            }
            self.GetDoorDrawDataIndex_North(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.Dungeon_LoadSingleDoorAttribute(door ^ 8);
        }
        self.GetDoorDrawDataIndex_South(door, door & 7);
        dma_ptr
    }

    pub(super) fn GetDoorDrawDataIndex_South(&mut self, door: usize, r4_door: usize) {
        let door_type = self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe;
        let mut x = self.game_state.dungeon.doors.door_open_counter_low() as usize;
        if x == 0 || x == 4 {
            self.DrawDoorToTileMap_South(door, r4_door);
            return;
        }
        if door_type >= 0x42 {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = DOOR_ANIMATION_DOWN_SOURCES[x >> 1];
        self.Object_Draw_DoorDown_4x3(src, door);
    }

    pub(super) fn DrawDoorToTileMap_South(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_DOWN[index];
        self.Object_Draw_DoorDown_4x3(src, door);
    }

    pub(super) fn Object_Draw_DoorDown_4x3(&mut self, src: u16, door: usize) {
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

    pub(super) fn GetDoorDrawDataIndex_West_clean_door_index(&mut self, door: usize) {
        self.GetDoorDrawDataIndex_West(door, door);
    }

    pub(super) fn DoorDoorStep1_West(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = self.game_state.dungeon.doors.door_tilemap_address(door) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x7ff) >= DOOR_POSITION_LEFT[6] as i32 {
            pos -= 16;
            if (self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe) >= 0x42 {
                pos -= 12;
            }
            self.GetDoorDrawDataIndex_East(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.Dungeon_LoadSingleDoorAttribute(door ^ 8);
        }
        self.GetDoorDrawDataIndex_West(door, door & 7);
        dma_ptr
    }

    pub(super) fn GetDoorDrawDataIndex_West(&mut self, door: usize, r4_door: usize) {
        let door_type = self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe;
        let mut x = self.game_state.dungeon.doors.door_open_counter_low() as usize;
        if x == 0 || x == 4 {
            self.DrawDoorToTileMap_West(door, r4_door);
            return;
        }
        if door_type >= 0x42 {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = DOOR_ANIMATION_LEFT_SOURCES[x >> 1];
        self.Object_Draw_DoorLeft_3x4(src, door);
    }

    pub(super) fn DrawDoorToTileMap_West(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_LEFT[index];
        self.Object_Draw_DoorLeft_3x4(src, door);
    }

    pub(super) fn GetDoorDrawDataIndex_East_clean_door_index(&mut self, door: usize) {
        self.GetDoorDrawDataIndex_East(door, door);
    }

    pub(super) fn DoorDoorStep1_East(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = self.game_state.dungeon.doors.door_tilemap_address(door) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x7ff) < DOOR_POSITION_RIGHT[6] as i32 {
            pos += 16;
            if (self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe) >= 0x42 {
                pos += 12;
            }
            self.GetDoorDrawDataIndex_West(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.Dungeon_LoadSingleDoorAttribute(door ^ 8);
        }
        self.GetDoorDrawDataIndex_East(door, door & 7);
        dma_ptr
    }

    pub(super) fn GetDoorDrawDataIndex_East(&mut self, door: usize, r4_door: usize) {
        let door_type = self.game_state.dungeon.doors.door_type_and_slot(door) & 0xfe;
        let mut x = self.game_state.dungeon.doors.door_open_counter_low() as usize;
        if x == 0 || x == 4 {
            self.DrawDoorToTileMap_East(door, r4_door);
            return;
        }
        if door_type >= 0x42 {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = DOOR_ANIMATION_RIGHT_SOURCES[x >> 1];
        self.Object_Draw_DoorRight_3x4(src, door);
    }

    pub(super) fn DrawDoorToTileMap_East(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_RIGHT[index];
        self.Object_Draw_DoorRight_3x4(src, door);
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

    pub(super) fn ClearExplodingWallFromTileMap_ClearOnePair(
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

    pub(super) fn Door_BlastWallExploding_Draw(&mut self, dsto: usize) {
        let src = 0x31eausize;
        self.ClearExplodingWallFromTileMap_ClearOnePair(dsto as u16, src);
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
        self.ClearExplodingWallFromTileMap_ClearOnePair(dst, src + 50);
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
                let vram_addr = self.Dungeon_MapVramAddrNoSwap(dsto);
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

    pub(super) fn Dungeon_DrawRoomOverlay(&mut self, src: &[u8]) {
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
            let p = ((src[offset] as u16 >> 2) | ((src[offset + 1] as u16 >> 2) << 6)) as u16;
            let kind = src[offset + 2];
            if kind == 0xa4 {
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
            offset += 3;
        }
    }

    pub(super) fn Dungeon_DrawRoomOverlay_Apply(&mut self, mut p: usize) {
        for _ in 0..4 {
            for i in 0..4 {
                let t = self.game_state.dungeon.room_tilemaps.bg2_tile(p + i) & 0x03fe;
                let attr = if t == 0x00ee || t == 0x00fe { 0 } else { 0x20 };
                self.dungeon_bg2_attributes_mut().set_bg2_attr(p + i, attr);
                if std::env::var_os("ZELDA3_TRACE_OVERLAY_ATTR").is_some() {
                    let pos = p + i;
                    let trace_pos = std::env::var("ZELDA3_TRACE_OVERLAY_ATTR_POS")
                        .ok()
                        .and_then(|value| {
                            value
                                .strip_prefix("0x")
                                .or_else(|| value.strip_prefix("0X"))
                                .and_then(|hex| usize::from_str_radix(hex, 16).ok())
                                .or_else(|| value.parse::<usize>().ok())
                        });
                    if trace_pos.map_or(true, |target| target == pos) {
                        eprintln!(
                            "R overlay_attr fc={} room=0x{:04x} overlay=0x{:02x} p=0x{:04x} pos=0x{:04x} tile=0x{:04x} attr=0x{:02x}",
                            self.game_state.frame.frame_counter,
                            self.game_state.world.location.dungeon_room(),
                            self.game_state.dungeon.room_load.overlay_to_load(),
                            p,
                            pos,
                            t,
                            attr
                        );
                    }
                }
            }
            p += 64;
        }
    }

    pub(super) fn DrawDoorOpening_Step1(&mut self, door: usize, dma_ptr: usize) -> usize {
        self.dungeon_doors_mut()
            .set_current_door_index_for_slot(door);
        self.dungeon_room_load_mut().set_selected_key_door(door);
        match self.game_state.dungeon.doors.door_direction(door) & 3 {
            0 => self.DoorDoorStep1_North(door, dma_ptr),
            1 => self.DoorDoorStep1_South(door, dma_ptr),
            2 => self.DoorDoorStep1_West(door, dma_ptr),
            3 => self.DoorDoorStep1_East(door, dma_ptr),
            _ => 0,
        }
    }

    pub(super) fn DrawShutterDoorSteps(&mut self, door: usize) {
        self.dungeon_doors_mut()
            .set_current_door_index_for_slot(door);
        self.dungeon_room_load_mut().set_selected_key_door(door);
        match self.game_state.dungeon.doors.door_direction(door) & 3 {
            0 => self.GetDoorDrawDataIndex_North_clean_door_index(door),
            1 => self.GetDoorDrawDataIndex_South_clean_door_index(door),
            2 => self.GetDoorDrawDataIndex_West_clean_door_index(door),
            3 => self.GetDoorDrawDataIndex_East_clean_door_index(door),
            _ => {}
        }
    }

    pub(super) fn DrawEyeWatchDoor(&mut self, door: usize) {
        self.dungeon_doors_mut()
            .set_current_door_index_for_slot(door);
        self.dungeon_room_load_mut().set_selected_key_door(door);
        match self.game_state.dungeon.doors.door_direction(door) & 3 {
            0 => self.DrawDoorToTileMap_North(door, door),
            1 => self.DrawDoorToTileMap_South(door, door),
            2 => self.DrawDoorToTileMap_West(door, door),
            3 => self.DrawDoorToTileMap_East(door, door),
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
                        self.Dungeon_LoadToggleDoorAttr_OtherEntry(j as i32);
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
        self.Dungeon_OpeningLockedDoor_Combined(true);
    }

    pub(super) fn Dungeon_OpeningLockedDoor_Combined(&mut self, skip_anim: bool) {
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
            let mask =
                upper_bitmask((self.game_state.dungeon.bg2_attributes.bg2_attr(cur) & 7) as usize);
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
        let k = (self.game_state.dungeon.bg2_attributes.bg2_attr(cur) & 0x0f) as usize;
        let dma_ptr = self.DrawDoorOpening_Step1(k, 0);
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
        let k = (self.game_state.dungeon.bg2_attributes.bg2_attr(cur) & 0x0f) as usize;
        self.Dungeon_LoadToggleDoorAttr_OtherEntry(k as i32);
        if self.game_state.dungeon.bg2_attributes.bg2_attr(cur) >= 0xf0 {
            let door_type = self.game_state.dungeon.doors.door_type_and_slot(k);
            if (DOOR_TYPE_STAIR_MASK_LOCKED0..=DOOR_TYPE_STAIR_MASK_LOCKED3).contains(&door_type) {
                self.DrawCompletelyOpenDoor();
            }
        }
        self.set_submodule(0);
    }

    pub(super) fn DrawCompletelyOpenDoor(&mut self) {
        let mut i = 0usize;
        let mut attr = 0x3030u16;
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomUpNorth) as usize
        {
            i += 2;
            attr = attr.wrapping_add(0x0101);
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
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(pos + xy(1, 0), 0x5e5e);
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), 0);
            self.write_attr2(pos + xy(1, 3), 0);
            i += 2;
            attr = attr.wrapping_add(0x0101);
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
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(pos + xy(1, 0), 0x5f5f);
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), 0);
            self.write_attr2(pos + xy(1, 3), 0);
            i += 2;
            attr = attr.wrapping_add(0x0101);
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
            attr = attr.wrapping_add(0x0101);
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
            attr = attr.wrapping_add(0x0101);
        }

        attr = (attr & 0x0707) | 0x3434;

        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomSouthDown) as usize
        {
            i += 2;
            attr = attr.wrapping_add(0x0101);
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
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(pos + xy(1, 0), 0x5e5e);
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), 0);
            self.write_attr2(pos + xy(1, 3), 0);
            i += 2;
            attr = attr.wrapping_add(0x0101);
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
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(pos + xy(1, 0), 0x5f5f);
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), 0);
            self.write_attr2(pos + xy(1, 3), 0);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
    }

    pub(super) fn Dungeon_LoadAttributeTable(&mut self) {
        self.dungeon_room_load_mut()
            .set_draw_width_indicator_word(0);
        self.dungeon_room_load_mut()
            .set_draw_height_indicator_word(0);
        self.Dungeon_LoadBasicAttribute_full(0x1000);
        self.Dungeon_LoadObjectAttribute();
        self.Dungeon_LoadDoorAttribute();
        if self
            .game_state
            .dungeon
            .environment
            .orange_blue_barrier_state()
            != 0
        {
            self.Dungeon_FlipCrystalPegAttribute();
        }
        self.set_overworld_map_state(0);
    }

    pub(super) fn Dungeon_LoadAttribute_Selectable(&mut self) {
        match self.overworld_map_state() {
            0 => {
                self.set_overworld_map_state(1);
                self.dungeon_room_load_mut()
                    .set_draw_width_indicator_word(0);
                self.dungeon_room_load_mut()
                    .set_draw_height_indicator_word(0);
                self.Dungeon_LoadBasicAttribute_full(0x40);
            }
            1 => self.Dungeon_LoadBasicAttribute_full(0x40),
            2 => self.Dungeon_LoadObjectAttribute(),
            3 => self.Dungeon_LoadDoorAttribute(),
            4 => {
                self.set_overworld_map_state(5);
                if self
                    .game_state
                    .dungeon
                    .environment
                    .orange_blue_barrier_state()
                    != 0
                {
                    self.Dungeon_FlipCrystalPegAttribute();
                }
            }
            5 => {}
            // C Dungeon_LoadAttribute_Selectable asserts outside states 0..=5.
            _ => panic!(
                "Dungeon_LoadAttribute_Selectable overworld_map_state {}",
                self.overworld_map_state()
            ),
        }
    }

    fn Dungeon_LoadBasicAttribute_full(&mut self, loops: usize) {
        for _ in 0..loops {
            let i = self
                .game_state
                .dungeon
                .room_load
                .draw_width_indicator_word() as usize
                / 2;
            let tile0 = self.game_state.dungeon.room_tilemaps.attr_source_tile(i);
            let tile1 = self
                .game_state
                .dungeon
                .room_tilemaps
                .attr_source_tile(i + 1);
            let a0 = self.attribute_for_bg_tile(tile0);
            let a1 = self.attribute_for_bg_tile(tile1);
            let j = self
                .game_state
                .dungeon
                .room_load
                .draw_height_indicator_word() as usize;
            self.dungeon_bg2_attributes_mut().set_bg2_attr(j, a0);
            self.dungeon_bg2_attributes_mut().set_bg2_attr(j + 1, a1);
            self.dungeon_room_load_mut()
                .set_draw_height_indicator_word((j as u16).wrapping_add(2));
            let width = self
                .game_state
                .dungeon
                .room_load
                .draw_width_indicator_word()
                .wrapping_add(4);
            self.dungeon_room_load_mut()
                .set_draw_width_indicator_word(width);
        }
        if self
            .game_state
            .dungeon
            .room_load
            .draw_height_indicator_word()
            == 0x2000
        {
            self.increment_overworld_map_state();
        }
    }

    fn attribute_for_bg_tile(&self, tile: u16) -> u8 {
        let mut attr = self.dungeon_tile_attribute(tile as usize);
        if (0x10..0x1c).contains(&attr) {
            attr |= (tile >> 14) as u8;
        }
        attr
    }

    fn Dungeon_LoadObjectAttribute(&mut self) {
        if std::env::var_os("ZELDA3_REPLAY_DUNGEON_ATTR_STATE_DUMP").is_some() {
            eprintln!(
                "dungeon-attr-state room=0x{:04x} star=0x{:04x} inter={:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x} in1={:04x},{:04x},{:04x},{:04x},{:04x},{:04x} misc=0x{:04x} torch=0x{:04x} chest=0x{:04x} big=0x{:04x} in2={:04x},{:04x},{:04x},{:04x} table1={:04x},{:04x},{:04x},{:04x} table2={:04x},{:04x},{:04x},{:04x} obj={:04x},{:04x},{:04x},{:04x}",
                self.game_state.world.location.dungeon_room(),
                self.game_state.dungeon.room_parser.star_switch_count_x2(),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InterRoomUpNorth),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::WallUpNorthSpiral),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::WallUpNorthSpiralBg1),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InterRoomUpNorthStraight),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InterRoomUpSouthStraight),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InterRoomSouthDown),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::WallDownNorthSpiral),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::WallDownNorthSpiralBg1),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InterRoomDownNorthStraight),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InRoomUpNorth),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InRoomSouthDown),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InterPseudoUpNorth),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::WaterSideStepSwitch),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InRoomUpNorthWater),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::ActivatedWaterLadders),
                self.game_state.dungeon.object_tracking.misc_object_index(),
                self.game_state.dungeon.torch.torch_index(),
                self.game_state.dungeon.room_items.num_chests_x2(),
                self.game_state.dungeon.room_items.num_big_key_locks_x2(),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::Stairs1),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::Stairs2),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::WetStairs),
                self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InRoomUpSouthWater),
                self.game_state.dungeon.stair_lists.stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, 0),
                self.game_state.dungeon.stair_lists.stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, 2),
                self.game_state.dungeon.stair_lists.stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, 4),
                self.game_state.dungeon.stair_lists.stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, 6),
                self.game_state.dungeon.stair_lists.stair_list_tilemap_pos(DungeonStairList::Stairs1, 0),
                self.game_state.dungeon.stair_lists.stair_list_tilemap_pos(DungeonStairList::Stairs1, 2),
                self.game_state.dungeon.stair_lists.stair_list_tilemap_pos(DungeonStairList::Stairs1, 4),
                self.game_state.dungeon.stair_lists.stair_list_tilemap_pos(DungeonStairList::Stairs1, 6),
                self.game_state.dungeon.object_tracking.object_tilemap_pos(0),
                self.game_state.dungeon.object_tracking.object_tilemap_pos(1),
                self.game_state.dungeon.object_tracking.object_tilemap_pos(2),
                self.game_state.dungeon.object_tracking.object_tilemap_pos(3),
            );
        }
        let mut i = 0usize;
        while i != self.game_state.dungeon.room_parser.star_switch_count_x2() as usize {
            let j = self
                .game_state
                .dungeon
                .room_parser
                .star_switch_tilemap_pos(i) as usize;
            self.write_attr2(j + xy(0, 0), 0x3b3b);
            self.write_attr2(j + xy(0, 1), 0x3b3b);
            i += 2;
        }

        i = 0;
        let mut attr = 0x3030u16;
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomUpNorth) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(j + xy(1, 2), 0);
            self.write_attr2(j + xy(1, 0), 0x2626);
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallUpNorthSpiral) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(j + xy(1, 0), 0x5e5e);
            self.write_attr2(j + xy(1, 2), 0x5e5e);
            self.write_attr2(j + xy(1, 3), 0x5e5e);
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallUpNorthSpiralBg1) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(j + xy(1, 0), 0x5f5f);
            self.write_attr2(j + xy(1, 2), 0x5f5f);
            self.write_attr2(j + xy(1, 3), 0x5f5f);
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomUpNorthStraight)
                as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(j + xy(1, 0), 0x3838);
            self.write_attr2(j + xy(1, 2), 0);
            self.write_attr2(j + xy(1, 3), 0);
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomUpSouthStraight)
                as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(j + xy(1, 0), 0);
            self.write_attr2(j + xy(1, 1), 0);
            self.write_attr2(j + xy(1, 2), attr);
            self.write_attr2(j + xy(1, 3), 0x3939);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        attr = (attr & 0x0707) | 0x3434;
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomSouthDown) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(j + xy(1, 2), attr);
            self.write_attr2(j + xy(1, 3), 0x2626);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallDownNorthSpiral) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(j + xy(1, 0), 0x5e5e);
            self.write_attr2(j + xy(1, 1), attr);
            self.write_attr2(j + xy(1, 2), 0x5e5e);
            self.write_attr2(j + xy(1, 3), 0x5e5e);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallDownNorthSpiralBg1) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(j + xy(1, 0), 0x5f5f);
            self.write_attr2(j + xy(1, 1), attr);
            self.write_attr2(j + xy(1, 2), 0x5f5f);
            self.write_attr2(j + xy(1, 3), 0x5f5f);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomDownNorthStraight)
                as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(j + xy(1, 0), 0x3838);
            self.write_attr2(j + xy(1, 1), attr);
            self.write_attr2(j + xy(1, 2), 0);
            self.write_attr2(j + xy(1, 3), 0);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomDownSouthStraight)
                as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos((i >> 1) as usize) as usize;
            self.write_attr2(j + xy(1, 0), 0);
            self.write_attr2(j + xy(1, 1), 0);
            self.write_attr2(j + xy(1, 2), attr);
            self.write_attr2(j + xy(1, 3), 0x3939);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }

        i = 0;
        let mut stair_type = 0u16;
        let mut iend = self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::InRoomUpNorth) as usize;
        attr = 0x1f1f;
        if iend == 0 {
            stair_type = 1;
            attr = 0x1e1e;
            iend = self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InRoomSouthDown) as usize;
            if iend == 0 {
                stair_type = 2;
                attr = 0x1d1d;
                iend = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::InterPseudoUpNorth)
                    as usize;
            }
        }
        if iend != 0 {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(stair_type);
            while i != iend {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, i as u16)
                    as usize;
                self.write_attr2(j + xy(0, 0), 0x0002);
                self.write_attr1(j + xy(0, 3), 0x0002);
                self.write_attr2(j + xy(2, 0), 0x0200);
                self.write_attr1(j + xy(2, 3), 0x0200);
                self.write_attr2(j + xy(0, 1), 0x0001);
                self.write_attr1(j + xy(0, 2), 0x0001);
                self.write_attr2(j + xy(2, 1), 0x0100);
                self.write_attr1(j + xy(2, 2), 0x0100);
                self.write_attr2(j + xy(1, 1), attr);
                self.write_attr1(j + xy(1, 1), attr);
                self.write_attr2(j + xy(1, 2), attr);
                self.write_attr1(j + xy(1, 2), attr);
                i += 2;
            }
        }
        if i != self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::WaterSideStepSwitch) as usize
        {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(2);
            while i
                != self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::WaterSideStepSwitch)
                    as usize
            {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, i as u16)
                    as usize;
                self.write_attr2(j + xy(0, 0), 0x0a03);
                self.write_attr1(j + xy(0, 0), 0x0a03);
                self.write_attr2(j + xy(2, 0), 0x030a);
                self.write_attr1(j + xy(2, 0), 0x030a);
                self.write_attr2(j + xy(0, 1), 0x0803);
                self.write_attr2(j + xy(2, 1), 0x0308);
                i += 2;
            }
        }
        i = 0;
        if i != self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::InRoomUpNorthWater) as usize
        {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(2);
            while i
                != self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::InRoomUpNorthWater)
                    as usize
            {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, i as u16)
                    as usize;
                self.write_attr2(j + xy(0, 0), 0x0003);
                self.write_attr2(j + xy(2, 0), 0x0300);
                self.write_attr1(j + xy(0, 0), 0x0a03);
                self.write_attr1(j + xy(2, 0), 0x030a);
                self.write_attr2(j + xy(0, 1), 0x0808);
                self.write_attr2(j + xy(2, 1), 0x0808);
                i += 2;
            }
        }
        if i != self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::ActivatedWaterLadders) as usize
        {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(2);
            while i
                != self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::ActivatedWaterLadders)
                    as usize
            {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, i as u16)
                    as usize;
                self.write_attr2(j + xy(0, 0), 0x0003);
                self.write_attr2(j + xy(2, 0), 0x0300);
                self.write_attr1(j + xy(0, 0), 0x0a03);
                self.write_attr1(j + xy(2, 0), 0x030a);
                i += 2;
            }
        }

        let mut i = 0usize;
        let mut attr = 0x7070u16;
        let misc_end = self.game_state.dungeon.object_tracking.misc_object_index() as usize;
        while i != misc_end {
            let k = self
                .game_state
                .dungeon
                .object_tracking
                .replacement_tile_state((i >> 1) as usize);
            if (k & 0x00f0) != 0x0030 {
                let j = (self
                    .game_state
                    .dungeon
                    .object_tracking
                    .object_tilemap_pos(i >> 1)
                    & 0x3fff)
                    >> 1;
                self.write_attr2(j as usize + xy(0, 0), attr);
                self.write_attr2(j as usize + xy(0, 1), attr);
            }
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }

        if i != self.game_state.dungeon.torch.torch_index() as usize {
            attr = 0xc0c0;
            while i != self.game_state.dungeon.torch.torch_index() as usize {
                let j = (self
                    .game_state
                    .dungeon
                    .object_tracking
                    .object_tilemap_pos(i >> 1)
                    & 0x3fff)
                    >> 1;
                self.write_attr2(j as usize + xy(0, 0), attr);
                self.write_attr2(j as usize + xy(0, 1), attr);
                i += 2;
                attr = (attr & 0xefef).wrapping_add(0x0101);
            }
            self.dungeon_torch_mut().set_torch_index(0);
        }

        let mut attr = 0x5858u16;
        let mut i = 0usize;
        let skip_big_key_locks = self.game_state.dungeon.room_items.num_chests_x2() != 0
            && self.hud_tags_suppress_big_key_locks();
        if self.game_state.dungeon.room_items.num_chests_x2() != 0 && !skip_big_key_locks {
            while i != self.game_state.dungeon.room_items.num_chests_x2() as usize {
                let k = self
                    .game_state
                    .dungeon
                    .room_items
                    .chest_location_for_offset_x2(i);
                if k != 0 {
                    let j = (k & 0x7fff) >> 1;
                    self.write_attr2(j as usize + xy(0, 0), attr);
                    self.write_attr2(j as usize + xy(0, 1), attr);
                    if k & 0x8000 != 0 {
                        self.dungeon_room_items_mut()
                            .set_chest_location_for_offset_x2(i, k & 0x7fff);
                        self.write_attr2(j as usize + xy(2, 1), attr);
                        self.write_attr2(j as usize + xy(0, 2), attr);
                        self.write_attr2(j as usize + xy(2, 2), attr);
                    }
                }
                i += 2;
                attr = attr.wrapping_add(0x0101);
            }
        }

        if !skip_big_key_locks {
            while i != self.game_state.dungeon.room_items.num_big_key_locks_x2() as usize {
                let k = self
                    .game_state
                    .dungeon
                    .room_items
                    .chest_location_for_offset_x2(i);
                self.dungeon_room_items_mut()
                    .set_chest_location_for_offset_x2(i, k | 0x8000);
                let j = (k & 0x7fff) >> 1;
                self.write_attr2(j as usize + xy(0, 0), attr);
                self.write_attr2(j as usize + xy(0, 1), attr);
                i += 2;
                attr = attr.wrapping_add(0x0101);
            }
        }

        i = 0;
        let mut stair_type = 0u16;
        let mut iend = self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::Stairs1) as usize;
        attr = 0x3f3f;
        if iend == 0 {
            stair_type = 1;
            attr = 0x3e3e;
            iend = self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::Stairs2) as usize;
            if iend == 0 {
                stair_type = 2;
                attr = 0x3d3d;
                iend = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::WetStairs) as usize;
            }
        }
        if iend != 0 {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(stair_type);
            while i != iend {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::Stairs1, i as u16)
                    as usize;
                self.write_attr1(j + xy(0, 0), 0x0002);
                self.write_attr2(j + xy(0, 3), 0x0002);
                self.write_attr1(j + xy(0, 1), 0x0001);
                self.write_attr2(j + xy(0, 2), 0x0001);
                self.write_attr1(j + xy(2, 0), 0x0200);
                self.write_attr2(j + xy(2, 3), 0x0200);
                self.write_attr1(j + xy(2, 1), 0x0100);
                self.write_attr2(j + xy(2, 2), 0x0100);
                self.write_attr1(j + xy(1, 1), attr);
                self.write_attr2(j + xy(1, 1), attr);
                self.write_attr1(j + xy(1, 2), attr);
                self.write_attr2(j + xy(1, 2), attr);
                i += 2;
            }
        }

        if self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::InRoomUpSouthWater)
            != 0
        {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(2);
            i = 0;
            while i
                != self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::InRoomUpSouthWater)
                    as usize
            {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::Stairs1, i as u16)
                    as usize;
                self.write_attr1(j + xy(0, 3), 0x0a03);
                self.write_attr1(j + xy(2, 3), 0x030a);
                self.write_attr2(j + xy(0, 3), 0x0003);
                self.write_attr2(j + xy(2, 3), 0x0300);
                self.write_attr2(j + xy(0, 2), 0x0808);
                self.write_attr2(j + xy(2, 2), 0x0808);
                i += 2;
            }
        }
        self.increment_overworld_map_state();
    }

    fn hud_tags_suppress_big_key_locks(&self) -> bool {
        (0..2).any(|i| {
            let tag = self.game_state.dungeon.header.header_tag(i);
            tag == 0x27 || tag == 0x3c || tag == 0x3e || (0x29..0x33).contains(&tag)
        })
    }

    fn Dungeon_LoadDoorAttribute(&mut self) {
        for k in 0..16 {
            if self.game_state.dungeon.doors.door_tilemap_address(k) != 0 {
                self.Dungeon_LoadSingleDoorAttribute(k);
            }
        }
        self.Dungeon_LoadSingleDoorTileAttribute();
        self.ChangeDoorToSwitch();
        self.increment_overworld_map_state();
    }

    pub(super) fn Dungeon_LoadToggleDoorAttr_OtherEntry(&mut self, door: i32) {
        self.Dungeon_LoadSingleDoorAttribute(door as usize);
        self.Dungeon_LoadSingleDoorTileAttribute();
    }

    fn Dungeon_LoadSingleDoorAttribute(&mut self, k: usize) {
        let t = self.game_state.dungeon.doors.door_type_and_slot(k) & 0xfe;
        if std::env::var_os("ZELDA3_REPLAY_DOOR_ATTR_TRACE").is_some() {
            eprintln!(
                "door-attr frame={} entry k={} t=0x{:02x} raw=0x{:04x} opened=0x{:04x} opened_adj=0x{:04x} cur=0x{:04x} addr=0x{:04x} dir=0x{:04x} sub={} step=0x{:04x}",
                self.game_state.frame.frame_counter,
                k,
                t,
                self.game_state.dungeon.doors.door_type_word(k),
                self.game_state.dungeon.doors.opened_doors(),
                self.game_state.dungeon.doors.opened_doors_including_adjacent(),
                self.game_state.dungeon.doors.current_door_pos(),
                self.game_state.dungeon.doors.door_tilemap_address(k),
                self.game_state.dungeon.doors.door_direction_word(k),
                self.game_state.frame.submodule,
                self.game_state.dungeon.doors.door_animation_step(),
            );
        }
        if !matches!(
            t,
            DOOR_TYPE_REGULAR
                | DOOR_TYPE_ENTRANCE_DOOR
                | DOOR_TYPE_EXIT_TO_OW
                | DOOR_TYPE_ENTRANCE_LARGE
                | DOOR_TYPE_ENTRANCE_CAVE
                | DOOR_TYPE_ENTRANCE_LARGE2
                | DOOR_TYPE_ENTRANCE_CAVE2
                | DOOR_TYPE_4
                | DOOR_TYPE_REGULAR2
                | DOOR_TYPE_WATERFALL_TUNNEL
        ) {
            if t == DOOR_TYPE_LG_EXPLOSION {
                return;
            }
            if t >= DOOR_TYPE_REGULAR_DOOR33 {
                if t != DOOR_TYPE_REGULAR_DOOR33
                    && t != DOOR_TYPE_WARP_ROOM_DOOR
                    && self
                        .game_state
                        .dungeon
                        .doors
                        .opened_doors_including_adjacent()
                        & upper_bitmask(k)
                        == 0
                {
                    let j = self.game_state.dungeon.doors.door_tilemap_address(k) >> 1;
                    let attr = (0xf0u16.wrapping_add(k as u16)).wrapping_mul(0x0101);
                    self.write_attr2(j as usize + xy(1, 1), attr);
                    self.write_attr2(j as usize + xy(1, 2), attr);
                    return;
                }
            } else {
                let i = if t == DOOR_TYPE_SHUTTERS_TWO_WAY || t == DOOR_TYPE_SHUTTER {
                    k
                } else {
                    k & 7
                };
                if self
                    .game_state
                    .dungeon
                    .doors
                    .opened_doors_including_adjacent()
                    & upper_bitmask(i)
                    == 0
                {
                    let j = self.game_state.dungeon.doors.door_tilemap_address(k) >> 1;
                    let attr = (0xf0u16.wrapping_add(k as u16)).wrapping_mul(0x0101);
                    self.write_attr2(j as usize + xy(1, 1), attr);
                    self.write_attr2(j as usize + xy(1, 2), attr);
                    return;
                }
            }
        }

        if (DOOR_TYPE_STAIR_MASK_LOCKED0..=DOOR_TYPE_STAIR_MASK_LOCKED3).contains(&t) {
            if std::env::var_os("ZELDA3_REPLAY_DOOR_ATTR_TRACE").is_some() {
                eprintln!(
                    "door-attr frame={} stairmask-return k={} t=0x{:02x}",
                    self.game_state.frame.frame_counter, k, t,
                );
            }
            return;
        }
        let mut attr = DUNGEON_LOAD_SINGLE_DOOR_ATTRIBUTE_TILE_ATTRS_BY_DOOR
            .get(t as usize >> 1)
            .copied()
            .unwrap_or(0x8080);
        if std::env::var_os("ZELDA3_REPLAY_DOOR_ATTR_TRACE").is_some() {
            eprintln!(
                "door-attr frame={} alpha k={} t=0x{:02x} attr=0x{:04x}",
                self.game_state.frame.frame_counter, k, t, attr,
            );
        }
        let dir = self.game_state.dungeon.doors.door_direction(k) & 3;
        let address = self.game_state.dungeon.doors.door_tilemap_address(k);
        let beta = matches!(
            t,
            DOOR_TYPE_ENTRANCE_LARGE2
                | DOOR_TYPE_ENTRANCE_CAVE2
                | DOOR_TYPE_4
                | DOOR_TYPE_REGULAR2
                | DOOR_TYPE_WATERFALL_TUNNEL
                | DOOR_TYPE_REGULAR_DOOR33
                | DOOR_TYPE_WARP_ROOM_DOOR
        ) || (t >= DOOR_TYPE_REGULAR_DOOR33
            && self
                .game_state
                .dungeon
                .doors
                .opened_doors_including_adjacent()
                & upper_bitmask(k)
                != 0);

        if !beta {
            if dir == 0 {
                if self.door_address_is_exit(address) {
                    attr = 0x8e8e;
                }
                let j = ((address >> 1) & !0x07c0) as usize;
                for y in 0..=6 {
                    self.write_attr2(j + xy(1, y), attr);
                }
                self.write_attr2(j + xy(1, 7), 0);
            } else if dir == 1 {
                if t == DOOR_TYPE_ENTRANCE_LARGE
                    || t == DOOR_TYPE_ENTRANCE_CAVE
                    || self.door_address_is_exit(address)
                {
                    attr = 0x8e8e;
                }
                let j = (address >> 1) as usize;
                for y in 1..=5 {
                    self.write_attr2(j + xy(1, y), attr);
                }
            } else if dir == 2 {
                let j = ((address >> 1) & !0x001f) as usize;
                self.write_attr2(j + xy(0, 1), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(2, 1), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(0, 2), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(2, 2), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(4, 1), attr.wrapping_add(0x0101) & 0x00ff);
                self.write_attr2(j + xy(4, 2), attr.wrapping_add(0x0101) & 0x00ff);
            } else {
                let j = (address >> 1) as usize;
                self.write_attr2(j + xy(2, 1), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(4, 1), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(2, 2), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(4, 2), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(0, 1), attr.wrapping_add(0x0101) & 0xff00);
                self.write_attr2(j + xy(0, 2), attr.wrapping_add(0x0101) & 0xff00);
            }
            return;
        }

        if dir == 0 {
            let j = ((address >> 1) & !0x07c0) as usize;
            for y in 0..=9 {
                self.write_attr2(j + xy(1, y), attr);
            }
        } else if dir == 1 {
            if t == DOOR_TYPE_ENTRANCE_LARGE2
                || t == DOOR_TYPE_ENTRANCE_CAVE2
                || t == DOOR_TYPE_4
                || self.door_address_is_exit(address & 0x1fff)
            {
                attr = 0x8e8e;
            }
            let j = (address >> 1) as usize;
            for y in 1..=8 {
                self.write_attr2(j + xy(1, y), attr);
            }
        } else if dir == 2 {
            let j = ((address >> 1) & !0x001f) as usize;
            for y in 1..=2 {
                for x in [0, 2, 4, 6] {
                    self.write_attr2(j + xy(x, y), attr.wrapping_add(0x0101));
                }
            }
        } else {
            let j = (address >> 1).wrapping_add(1) as usize;
            for y in 1..=2 {
                for x in [0, 2, 4, 6] {
                    self.write_attr2(j + xy(x, y), attr.wrapping_add(0x0101));
                }
            }
        }
    }

    fn door_address_is_exit(&self, address: u16) -> bool {
        self.game_state
            .dungeon
            .door_setup
            .has_exit_door_address(address)
    }

    fn Door_LoadBlastWallAttr(&mut self, k: usize) {
        let mut j = (self.game_state.dungeon.doors.door_tilemap_address(k) >> 1) as usize;
        if self.game_state.dungeon.doors.door_direction(k) & 2 == 0 {
            for _ in 0..12 {
                self.write_attr2(j + xy(0, 0), 0x0102);
                for i in (2..20).step_by(2) {
                    self.write_attr2(j + xy(i, 0), 0);
                }
                self.write_attr2(j + xy(20, 0), 0x0201);
                j += xy(0, 1);
            }
        } else {
            for _ in 0..5 {
                self.write_attr2(j + xy(0, 0), 0x0101);
                self.write_attr2(j + xy(0, 21), 0x0101);
                self.write_attr2(j + xy(0, 1), 0x0202);
                self.write_attr2(j + xy(0, 20), 0x0202);
                for i in 2..20 {
                    self.write_attr2(j + xy(0, i), 0);
                }
                j += xy(2, 0);
            }
        }
    }

    fn ChangeDoorToSwitch(&self) {
        assert_eq!(self.game_state.dungeon.door_setup.width_road_address(), 0);
    }

    fn Dungeon_FlipCrystalPegAttribute(&mut self) {
        for i in (0..=0x0fff).rev() {
            if self.game_state.dungeon.bg2_attributes.bg2_attr(i) & !1 == 0x66 {
                self.dungeon_bg2_attributes_mut().xor_bg2_attr(i, 1);
            }
            if self.game_state.dungeon.bg2_attributes.bg1_attr(i) & !1 == 0x66 {
                self.dungeon_bg2_attributes_mut().xor_bg1_attr(i, 1);
            }
        }
    }

    fn write_attr2(&mut self, j: usize, attr: u16) {
        let attr_view = &self.game_state.dungeon.bg2_attributes;
        let base = attr_view.bg2_attr_address(j);
        if attr_view.bg2_attr_pair(j).is_none() {
            if std::env::var_os("ZELDA3_REPLAY_DUNGEON_ATTR_TRACE").is_some() {
                eprintln!(
                    "attr-write-oob frame={} fn=write_attr2 j=0x{:04x} attr=0x{:04x} base=0x{:05x} ram_len=0x{:05x} stairs1=0x{:04x} stairs2=0x{:04x} inter=0x{:04x} misc=0x{:04x} chest=0x{:04x} big=0x{:04x} counts={:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x}",
                    self.state_recorder.replay_frame_counter,
                    j,
                    attr,
                    base,
                    self.compatibility_state_len(),
                    self.game_state.dungeon.stair_lists.stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, 0),
                    self.game_state.dungeon.stair_lists.stair_list_tilemap_pos(DungeonStairList::Stairs1, 0),
                    self.game_state.dungeon.stair_lists.inter_staircase_pos(0),
                    self.game_state.dungeon.object_tracking.misc_object_index(),
                    self.game_state.dungeon.room_items.num_chests_x2(),
                    self.game_state.dungeon.room_items.num_big_key_locks_x2(),
                    self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InRoomUpNorth),
                    self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InRoomSouthDown),
                    self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InterPseudoUpNorth),
                    self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::WaterSideStepSwitch),
                    self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::InRoomUpNorthWater),
                    self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::ActivatedWaterLadders),
                    self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::Stairs1),
                    self.game_state.dungeon.stair_lists.stair_list_count(DungeonStairList::Stairs2),
                );
            }
            return;
        }
        if std::env::var_os("ZELDA3_REPLAY_DUNGEON_ATTR_TRACE").is_some() {
            let frame_target = std::env::var("ZELDA3_REPLAY_DUNGEON_ATTR_FRAME")
                .ok()
                .and_then(|value| parse_usize_env(&value));
            let target = std::env::var("ZELDA3_REPLAY_DUNGEON_ATTR_POS")
                .ok()
                .and_then(|value| parse_usize_env(&value));
            let frame_matches = frame_target
                .map(|target| self.state_recorder.replay_frame_counter as usize == target)
                .unwrap_or(true);
            if frame_matches
                && match target {
                    Some(target) => j == target || j + 1 == target,
                    None => true,
                }
            {
                let before = self.game_state.dungeon.bg2_attributes.bg2_attr_pair(j);
                eprintln!(
                    "attr-write frame={} fn=write_attr2 j=0x{:04x} attr=0x{:04x} addr=0x{:05x} before={}/{} door_open=0x{:04x} door_adj=0x{:04x} cur=0x{:04x} sub={} step=0x{:04x}",
                    self.state_recorder.replay_frame_counter,
                    j,
                    attr,
                    base,
                    format_optional_hex(before.map(|pair| pair.0)),
                    format_optional_hex(before.map(|pair| pair.1)),
                    self.game_state.dungeon.doors.opened_doors(),
                    self.game_state.dungeon.doors.opened_doors_including_adjacent(),
                    self.game_state.dungeon.doors.current_door_pos(),
                    self.game_state.frame.submodule,
                    self.game_state.dungeon.doors.door_animation_step(),
                );
            }
        }
        self.dungeon_bg2_attributes_mut().set_bg2_attr_word(j, attr);
    }

    fn write_attr1(&mut self, j: usize, attr: u16) {
        if j + 1 >= 0x1000 {
            return;
        }
        self.dungeon_bg2_attributes_mut().set_bg1_attr_word(j, attr);
    }

    fn Dungeon_LoadSingleDoorTileAttribute(&mut self) {
        let mut i = 0usize;
        while i != self.game_state.dungeon.room_parser.toggle_floor_count_x2() as usize {
            let j = self.game_state.dungeon.room_parser.toggle_floor_pos(i >> 1) as usize;
            if self.game_state.dungeon.bg2_attributes.bg2_attr(j) & 0xf0 == 0x80 {
                let attr = self.game_state.dungeon.bg2_attributes.bg2_attr_word(j);
                self.write_attr2(j + xy(0, 0), attr | 0x1010);
                self.write_attr2(j + xy(0, 1), attr | 0x1010);
            } else {
                let attr = self.game_state.dungeon.bg2_attributes.bg1_attr_word(j);
                self.write_attr1(j + xy(0, 0), attr | 0x1010);
                self.write_attr1(j + xy(0, 1), attr | 0x1010);
            }
            i += 2;
        }

        i = 0;
        while i != self.game_state.dungeon.room_parser.toggle_palace_count_x2() as usize {
            let j = self
                .game_state
                .dungeon
                .room_parser
                .toggle_palace_pos(i >> 1) as usize;
            if self.game_state.dungeon.bg2_attributes.bg2_attr(j) & 0xf0 == 0x80 {
                let attr = self.game_state.dungeon.bg2_attributes.bg2_attr_word(j);
                self.write_attr2(j + xy(0, 0), attr | 0x2020);
                self.write_attr2(j + xy(0, 1), attr | 0x2020);
            } else {
                let attr = self.game_state.dungeon.bg2_attributes.bg1_attr_word(j);
                self.write_attr1(j + xy(0, 0), attr | 0x2020);
                self.write_attr1(j + xy(0, 1), attr | 0x2020);
            }
            i += 2;
        }
    }

    pub(super) fn Mirror_SaveRoomData(&mut self) {
        if self.game_state.inventory.save_progress.palace_index_x2() == 0xff {
            self.set_sound_effect_1(60);
            return;
        }
        self.set_submodule(25);
        self.set_subsubmodule(0);
        self.set_sound_effect_1(51);
        self.Dungeon_FlagRoomData_Quadrants();
        self.SaveDungeonKeys();
    }

    pub(super) fn Dung_TagRoutine_0x00(&mut self, _k: usize) {}

    pub(super) fn Dungeon_DetectStaircase(&mut self) {
        let k = self.game_state.player.follower_link.direction() & 12;
        if k == 0 {
            return;
        }

        let lookup = DUNGEON_DETECT_STAIRCASE_BUGGY_LOOKUP[(k >> 1) as usize] as i16 as u16;
        let mut pos = (self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(lookup)
            & 0x01f8)
            << 3;
        pos |= (self.game_state.player.follower_link.x() & 0x01f8) >> 3;
        pos |= self
            .game_state
            .player
            .follower_link
            .lower_level_tilemap_offset();

        let at_pos = pos.wrapping_add(if k == 4 { 0x80 } else { 0 }) as usize;
        let at = self.game_state.dungeon.bg2_attributes.bg2_attr(at_pos);
        if !matches!(at, 0x26 | 0x38 | 0x39 | 0x5e | 0x5f) {
            return;
        }

        let attr2 = self
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr(pos as usize + xy(0, 1));
        if attr2 & 0xf8 != 0x30 {
            return;
        }

        if self
            .game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
        {
            self.follower_link_state_mut()
                .restore_y_from_previous_position();
            return;
        }

        self.dungeon_stair_movement_mut().set_staircase_index(attr2);
        self.dungeon_stair_movement_mut()
            .set_staircase_index_high((pos >> 8) as u8);
        let dungeon_room = self.game_state.world.location.dungeon_room();
        self.dungeon_room_tracking_mut()
            .set_previous_room_index_word(dungeon_room);
        self.Dungeon_FlagRoomData_Quadrants();

        if at == 0x38 || at == 0x39 {
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(0x20);
            if at == 0x38 {
                self.Dungeon_StartInterRoomTrans_Up();
            } else {
                self.Dungeon_StartInterRoomTrans_Down();
            }
        }

        let j = (self.game_state.dungeon.stair_movement.staircase_index() & 3) as usize;
        let room = self.game_state.dungeon.header.travel_destination(j + 1);
        let plane = self.game_state.dungeon.header.staircase_plane(j);
        let lower_level_status = if self
            .game_state
            .player
            .follower_link
            .has_lower_level_state_or_mirror()
        {
            2
        } else {
            0
        };
        self.set_dungeon_room_index(room);
        self.dungeon_stair_movement_mut()
            .set_current_staircase_plane(plane);
        self.dungeon_stair_movement_mut()
            .set_staircase_lower_level_status(lower_level_status);
        self.set_subsubmodule(0);
        self.follower_link_state_mut().clear_defense_flags();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().clear_button_b_frames();
        self.follower_link_state_mut().clear_direction_lock_bits(1);

        if at == 0x26 {
            self.set_submodule(6);
            let sound_effect_1 = if self
                .game_state
                .dungeon
                .stair_movement
                .current_staircase_plane()
                < 0x34
            {
                22
            } else {
                24
            };
            self.set_sound_effect_1(sound_effect_1);
        } else if at == 0x38 || at == 0x39 {
            let submodule = if at == 0x38 { 18 } else { 19 };
            self.set_submodule(submodule);
            self.follower_link_state_mut().set_push_fatigue_timer(7);
        } else {
            self.UsedForStraightInterRoomStaircase();
            self.set_submodule(14);
        }
    }

    pub(super) fn UsedForStraightInterRoomStaircase(&mut self) {
        for i in (0..=9).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 13 {
                self.ancilla_slot_view_mut(i).clear();
            }
        }
        self.follower_link_state_mut()
            .clear_animation_step_if_at_least(5);
        self.follower_link_state_mut().clear_movement_subpixels();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_push_fatigue_timer(28);
        self.dungeon_stair_movement_mut()
            .set_staircase_countdown(32);
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.ancilla_sfx2_near(
            if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                0x18
            } else {
                0x16
            },
        );

        let x = self.game_state.player.follower_link.x();
        let detect_x = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            x.wrapping_sub(15)
        } else {
            x.wrapping_add(16)
        };
        self.tile_detect_position_mut().set_x(detect_x);
        let link_y = self.game_state.player.follower_link.y();
        self.tile_detect_position_mut().set_y(link_y);
    }

    pub(super) fn RoomTag_MovingWall_East(&mut self, k: usize) {
        if self.game_state.dungeon.moving_floor.floor_move_flags() == 0 {
            self.RoomTag_MovingWallTorchesCheck(k);
            self.dungeon_moving_floor_mut().set_floor_x_velocity(0);
        } else {
            self.set_modal_pause_flag(1);
            self.RoomTag_MovingWallShakeItUp(k);
            let vel = self.MovingWall_MoveALittle();
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
                    [(self.RoomTag_AdvanceGiganticWall(k) >> 1) as usize & 7];
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

    pub(super) fn RoomTag_MovingWallShakeItUp(&mut self, k: usize) {
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

    pub(super) fn RoomTag_MovingWall_West(&mut self, k: usize) {
        if self.game_state.dungeon.moving_floor.floor_move_flags() == 0 {
            self.RoomTag_MovingWallTorchesCheck(k);
            self.dungeon_moving_floor_mut().set_floor_x_velocity(0);
        } else {
            self.set_modal_pause_flag(1);
            self.RoomTag_MovingWallShakeItUp(k);
            let vel = self.MovingWall_MoveALittle();
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
                    [(self.RoomTag_AdvanceGiganticWall(k) >> 1) as usize & 7];
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

    pub(super) fn RoomTag_MovingWallTorchesCheck(&mut self, k: usize) {
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

    pub(super) fn MovingWall_MoveALittle(&mut self) -> u16 {
        let t = self.bg1_move_calc_mut().advance_x_subpixel(0x22);
        t >> 8
    }

    pub(super) fn RoomTag_AdvanceGiganticWall(&mut self, k: usize) -> u8 {
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

    pub(super) fn Dungeon_SaveAndLoadAllPalettes(&mut self, main_tile_theme: u8, sprite_gfx: u8) {
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
    pub(super) fn Dungeon_CheckForAndIDLiftableTile(&self) -> u16 {
        let facing = self.game_state.player.follower_link.facing_index();
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(LIFTABLE_TILE_PROBE_X_OFFSETS[facing] as i16 as u16)
            & 0x01f8;
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(LIFTABLE_TILE_PROBE_Y_OFFSETS[facing] as i16 as u16)
            & 0x01f8;
        let offset = ((y << 3) | (x >> 3)) as usize
            + usize::from(
                self.game_state
                    .player
                    .follower_link
                    .lower_level_tilemap_offset(),
            );

        let attr = self.game_state.dungeon.bg2_attributes.bg2_attr(offset);
        if attr & 0xf0 != 0x70 {
            return 0xffff;
        }

        let replacement = self
            .game_state
            .dungeon
            .object_tracking
            .replacement_tile_state((attr & 0x0f) as usize);
        if replacement == 0 {
            return 0xffff;
        }
        if replacement & 0xf0f0 == 0x2020 {
            return 0x55;
        }
        LIFTABLE_TILE_REPLACEMENT_ITEM_CODES[(replacement & 0x0f) as usize]
    }

    pub(super) fn OpenChestForItem(&mut self, tile: u8, chest_position: &mut u16) -> u8 {
        if let Some((item, position)) = self.OpenChestForItemResult(tile) {
            *chest_position = position;
            item
        } else {
            0xff
        }
    }

    pub(super) fn OpenChestForItemResult(&mut self, tile: u8) -> Option<(u8, u16)> {
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

    pub(super) fn OpenMiniGameChest(&mut self, chest_position: &mut u16) -> u8 {
        if let Some((item, position)) = self.OpenMiniGameChestResult() {
            *chest_position = position;
            item
        } else {
            0xff
        }
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
        if self
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_word(pos as usize)
            != 0x6363
        {
            pos = pos.wrapping_sub(1);
            if self
                .game_state
                .dungeon
                .bg2_attributes
                .bg2_attr_word(pos as usize)
                != 0x6363
            {
                pos = pos.wrapping_add(2);
            }
        }

        self.dungeon_bg2_attributes_mut()
            .set_bg2_attr_word(pos as usize, 0x0202);
        self.dungeon_bg2_attributes_mut()
            .set_bg2_attr_word(pos as usize + 64, 0x0202);

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
            let vram_addr = self.Dungeon_MapVramAddr(tile_pos);
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

    pub(super) fn OpenBigChest(&mut self, loc: u16, chest_position: &mut u16) {
        *chest_position = self.OpenBigChestResult(loc);
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
                .set_bg2_attr_word(tile_pos as usize, 0x2727);
        }
        self.Dungeon_FlagRoomData_Quadrants();
        self.set_sound_effect_2(14);
        self.request_nmi_copy_packets();
        self.dungeon_room_effects_mut().mark_trap_trigger_latched();
        loc + 2
    }

    pub(super) fn Module07_15_WarpPad(&mut self) {
        if self.game_state.frame.subsubmodule >= 3 {
            self.Graphics_IncrementalVRAMUpload();
            self.Dungeon_LoadAttribute_Selectable();
        }
        match self.game_state.frame.subsubmodule {
            0 => self.reset_transition_props_and_advance_reset_interface(),
            1 => self.Module07_15_01_ApplyMosaicAndFilter(),
            2 => self.Dungeon_InitializeRoomFromSpecial(),
            3 => self.DungeonTransition_LoadSpriteGFX(),
            4 => self.Module07_15_04_SyncRoomPropsAndBuildOverlay(),
            5 => self.Dungeon_InterRoomTrans_State4(),
            6 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            7 => self.Dungeon_InterRoomTrans_State4(),
            8 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            9 => self.Dungeon_InterRoomTrans_State4(),
            10 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            11 => self.Dungeon_InterRoomTrans_State4(),
            12 => self.Dungeon_Staircase14(),
            13 => self.Module07_15_0E_FadeInFromWarp(),
            14 => self.Module07_15_0F_FinalizeAndCacheEntry(),
            other => panic!("invalid Module07_15_WarpPad subsubmodule_index {other}"),
        }
    }

    pub(super) fn Dung_TagRoutine_TrapdoorsUp(&mut self) {
        if self.game_state.dungeon.environment.trapdoors_down() != 0 {
            self.dungeon_environment_mut().clear_trapdoors_down();
            self.dungeon_doors_mut().clear_current_door_pos();
            self.dungeon_doors_mut().clear_door_animation_step();
            self.set_sound_effect_2(0x1b);
            self.set_submodule(5);
        }
    }

    pub(super) fn CalculateTransitionLanding(&mut self) -> u8 {
        let mut pos = ((self.game_state.player.follower_link.y().wrapping_add(12) & 0x01f8) << 3)
            | ((self.game_state.player.follower_link.x().wrapping_add(8) & 0x01f8) >> 3);
        pos |= self
            .game_state
            .player
            .follower_link
            .lower_level_tilemap_offset();

        let mut attr = self
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_attr(pos as usize);
        let result = if attr == 0 || attr == 9 {
            0
        } else {
            attr &= 0x8e;
            if attr == 0x80 {
                1
            } else if attr == 0x82 {
                2
            } else if attr == 0x84 || attr == 0x88 {
                3
            } else if attr == 0x86 {
                4
            } else {
                2
            }
        };

        self.dungeon_room_runtime_mut().set_landing_class(result);
        result
    }

    pub(super) fn MirrorBg1Bg2Offs(&mut self) {
        let h = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let v = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.set_bg1_x(h);
        self.set_bg1_y(v);
    }

    pub(super) fn Dungeon_InterRoomTrans_State13(&mut self) {
        if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
            self.ApplyPaletteFilter_bounce();
        }
        self.Dungeon_IntraRoomTrans_State5();
    }

    pub(super) fn Module07_01_SubtileTransition(&mut self) {
        self.follower_link_state_mut()
            .cache_previous_position_from_current();
        self.link_handle_moving_animation_full_long_entry();
        match self.game_state.frame.subsubmodule {
            0 => self.DungeonTransition_Subtile_PrepTransition(),
            1 => self.DungeonTransition_Subtile_ApplyFilter(),
            2 => self.DungeonTransition_Subtile_ResetShutters(),
            3 => self.DungeonTransition_ScrollRoom(),
            4 => self.DungeonTransition_FindSubtileLanding(),
            5 => self.Dungeon_IntraRoomTrans_State5(),
            6 => self.DungeonTransition_Subtile_ApplyFilter(),
            7 => self.DungeonTransition_Subtile_TriggerShutters(),
            _ => panic!("invalid dungeon subtile transition index"),
        }
    }

    pub(super) fn Module07_02_SupertileTransition(&mut self) {
        self.follower_link_state_mut()
            .cache_previous_position_from_current();
        if self.game_state.frame.subsubmodule != 0 {
            if self.game_state.frame.subsubmodule >= 7 {
                self.Graphics_IncrementalVRAMUpload();
            }
            self.Dungeon_LoadAttribute_Selectable();
        }
        self.link_handle_moving_animation_full_long_entry();
        match self.game_state.frame.subsubmodule {
            0 => self.Module07_02_00_InitializeTransition(),
            1 => self.Module07_02_01_LoadNextRoom(),
            2 => self.Module07_02_FadedFilter(),
            3 => self.Dungeon_InterRoomTrans_State3(),
            4 => self.Dungeon_InterRoomTrans_State4(),
            5 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            6 => self.Dungeon_InterRoomTrans_State4(),
            7 => self.Dungeon_InterRoomTrans_State7(),
            8 => self.DungeonTransition_ScrollRoom(),
            9 => self.Dungeon_InterRoomTrans_State9(),
            10 => self.Dungeon_InterRoomTrans_State10(),
            11 => self.Dungeon_InterRoomTrans_State9(),
            12 => self.Dungeon_InterRoomTrans_State12(),
            13 => self.Dungeon_InterRoomTrans_State13(),
            14 => self.Module07_02_FadedFilter(),
            15 => self.Dungeon_InterRoomTrans_State15(),
            _ => panic!("invalid dungeon supertile transition index"),
        }
    }

    pub(super) fn Module07_02_00_InitializeTransition(&mut self) {
        let bak = self
            .game_state
            .dungeon
            .torch
            .dungeon_dark_with_lantern_raw();
        self.ResetTransitionPropsAndAdvanceSubmodule();
        self.dungeon_torch_mut()
            .set_dungeon_dark_with_lantern_raw(bak);
    }

    pub(super) fn Module07_02_01_LoadNextRoom(&mut self) {
        self.Dungeon_LoadRoom();
        self.ResetStarTileGraphics();
        self.LoadTransAuxGFX_sprite();
        self.increment_subsubmodule();
        self.set_overworld_map_state(0);
        let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
        self.dungeon_room_tracking_mut()
            .set_room_index2(dungeon_room_index);
        self.dungeon_reset_sprites();
        if !self.game_state.dungeon.torch.dungeon_dark_with_lantern() {
            self.MirrorBg1Bg2Offs();
        }
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
    }

    pub(super) fn Dungeon_InterRoomTrans_State3(&mut self) {
        if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
            self.set_sub_screen_layers(0);
        }
        self.Dungeon_AdjustForRoomLayout();
        self.LoadNewSpriteGFXSet();
        self.MirrorBg1Bg2Offs();
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.increment_subsubmodule();
    }

    pub(super) fn Module07_07_FallingTransition(&mut self) {
        if self.game_state.frame.subsubmodule >= 6 {
            self.Graphics_IncrementalVRAMUpload();
            self.Dungeon_LoadAttribute_Selectable();
            self.ApplyGrayscaleFixed_Incremental();
        }
        match self.game_state.frame.subsubmodule {
            0 => self.Module07_07_00_HandleMusicAndResetRoom(),
            1 => self.ApplyPaletteFilter_bounce(),
            2 => self.Dungeon_InitializeRoomFromSpecial(),
            3 => self.DungeonTransition_TriggerBGC34UpdateAndAdvance(),
            4 => self.DungeonTransition_TriggerBGC56UpdateAndAdvance(),
            5 => self.DungeonTransition_LoadSpriteGFX(),
            6 => self.Module07_07_06_SyncBG1and2(),
            7 => self.Dungeon_InterRoomTrans_State4(),
            8 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            9 => self.Dungeon_InterRoomTrans_State4(),
            10 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            11 => self.Dungeon_InterRoomTrans_State4(),
            12 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            13 => self.Dungeon_InterRoomTrans_State4(),
            14 => self.Dungeon_Staircase14(),
            15 => self.Module07_07_0F_FallingFadeIn(),
            16 => self.Module07_07_10_LandLinkFromFalling(),
            17 => self.Module07_07_11_CacheRoomAndSetMusic(),
            other => panic!("invalid Module07_07_FallingTransition subsubmodule_index {other}"),
        }
    }

    pub(super) fn Module07_07_00_HandleMusicAndResetRoom(&mut self) {
        let room = self.game_state.world.location.dungeon_room();
        if room == 0x10 || room == 7 || room == 0x17 {
            self.set_music_control(0xf1);
        }
        self.ResetTransitionPropsAndAdvance_ResetInterface();
    }

    pub(super) fn Module07_07_06_SyncBG1and2(&mut self) {
        self.MirrorBg1Bg2Offs();
        self.Dungeon_AdjustForRoomLayout();
        let mut ts = SPIRAL_SUBSCREEN_LAYER_BY_BG2
            [self.game_state.dungeon.room_load.bg2_properties() as usize]
            as u8;
        let mut tm = 0x16;
        if ts & 0x80 != 0 {
            tm = 0x17;
            ts = 0;
        }
        self.set_main_screen_layers(tm);
        self.set_sub_screen_layers(ts);
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.increment_subsubmodule();
    }

    pub(super) fn Module07_07_0F_FallingFadeIn(&mut self) {
        self.ApplyPaletteFilter_bounce();
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            != 0
        {
            return;
        }

        let link_y = self.game_state.player.follower_link.y();
        let detect_y = self.game_state.player.tile_detection.y();
        let high = ((link_y >> 8) as u8).wrapping_add(u8::from((link_y as u8) >= detect_y as u8));
        self.tile_detect_position_mut().set_y_high(high);
        self.Dungeon_SetBossMusicUnorthodox();

        let room = self.game_state.world.location.dungeon_room_index();
        if room == 0x89 || room == 0x4f {
            return;
        }
        if room == 0xa7 {
            self.clear_hud_floor_changed_timer();
            self.dungeon_stair_movement_mut().set_current_floor(1);
            return;
        }
        self.dungeon_stair_movement_mut().decrement_current_floor();
        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
    }

    pub(super) fn Module07_07_10_LandLinkFromFalling(&mut self) {
        self.handle_dungeon_landing_from_pit();
        if self.game_state.frame.submodule != 0 {
            return;
        }
        self.set_submodule(7);
        self.set_subsubmodule(17);
        self.set_chr_halfslot_request(1);
        self.Graphics_LoadChrHalfSlot();
    }

    pub(super) fn Module07_07_11_CacheRoomAndSetMusic(&mut self) {
        if self.overworld_map_state() == 5 {
            self.ResetThenCacheRoomEntryProperties();
            self.Dungeon_PlayMusicIfDefeated();
            self.Graphics_LoadChrHalfSlot();
        }
    }

    pub(super) fn Module11_DungeonFallingEntrance(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => {
                let entrance_music = self.asset_raw(27).expect("missing entrance music asset")
                    [self.game_state.world.region.which_entrance() as usize];
                if entrance_music != 3
                    || self.game_state.inventory.save_progress.progress_indicator() >= 2
                {
                    self.set_music_control(0xf1);
                }
                self.ResetTransitionPropsAndAdvance_ResetInterface();
            }
            1 => {
                if self.game_state.frame.frame_counter & 1 == 0 {
                    self.ApplyPaletteFilter_bounce();
                }
            }
            2 => self.Module11_02_LoadEntrance(),
            3 => self.DungeonTransition_LoadSpriteGFX(),
            4 => {
                let screen_brightness =
                    self.game_state.display.screen_brightness.wrapping_add(1) & 0x0f;
                self.set_screen_brightness(screen_brightness);
                if self.game_state.display.screen_brightness == 15 {
                    self.increment_subsubmodule();
                }
                self.Module11_DungeonFallingEntrance_land();
            }
            5 => self.Module11_DungeonFallingEntrance_land(),
            _ => {}
        }
    }

    fn Module11_DungeonFallingEntrance_land(&mut self) {
        self.handle_dungeon_landing_from_pit();
        if self.game_state.frame.submodule != 0 {
            return;
        }
        self.set_main_module(7);
        self.dungeon_room_runtime_mut().skip_room_tags_once();
        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
        self.ResetThenCacheRoomEntryProperties();
        let queued_music_control = self.game_state.system_signals.queued_music_control();
        self.set_music_control(queued_music_control);
        self.save_current_music_as_last();
    }

    pub(super) fn Module11_02_LoadEntrance(&mut self) {
        self.EnableForceBlank();
        self.set_color_window_selection(2);
        self.Dungeon_LoadEntrance();

        let dung = self.game_state.inventory.save_progress.palace_index_x2();
        let keys = if dung != 0xff {
            let palace_index_x2 = if dung == 2 { 0 } else { dung };
            self.game_state
                .inventory
                .dungeon_key_slots
                .keys_earned(palace_index_x2)
        } else {
            0xff
        };
        self.player_resources_mut().set_keys(keys);
        self.hud_rebuild();
        self.follower_link_state_mut().set_pit_data_index(4);
        self.follower_link_state_mut().set_near_pit_state(3);
        {
            self.follower_link_state_mut().set_visibility_status(12);
        }
        self.follower_link_state_mut().set_speed_modifier(16);

        let y = (self.game_state.player.follower_link.y() as u8)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
        {
            self.follower_link_state_mut().clear_state_bits();
            self.follower_link_state_mut().clear_picking_throw_state();
        }
        self.follower_link_state_mut().set_y_button_action_timer(0);
        let dungeon_room = self.game_state.world.location.dungeon_room();
        self.dungeon_room_tracking_mut()
            .set_previous_room_index_word(dungeon_room);
        let link_y = self.game_state.player.follower_link.y();
        self.tile_detect_position_mut().set_y(link_y);
        let new_y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(u16::from(y).wrapping_add(16));
        self.follower_link_state_mut().set_y(new_y);

        let bak = self.game_state.frame.subsubmodule;
        self.dungeon_torch_mut().clear_lit_torches();
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
        self.Dungeon_LoadAndDrawRoom();
        self.Dungeon_LoadCustomTileAttr();
        let animated = DUNG_ANIMATED_TILES
            [self.game_state.world.palette_theme.main_tile_theme_index() as usize];
        self.decompress_animated_dungeon_tiles(animated as usize);
        self.Dungeon_LoadAttributeTable();
        self.set_subsubmodule(bak.wrapping_add(1));
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(10);
        self.initialize_tilesets();
        // PALETTE_SP6R_INDOORS (0xab1) is read by palette_load_sprite_environment_dungeon
        // off display.palette_buffer, not world.palette_theme. Set the display owner so
        // the immediately-following dungeon_load_palettes() loads the selector-10 sprite
        // environment palette instead of a stale selector (matches raw-RAM single-byte).
        self.set_sp6r_indoors(10);
        self.dungeon_load_palettes();
        self.hud_restore_torch_background();
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().clear_button_b_frames();
        self.Dungeon_ResetTorchBackgroundAndPlayer();
        if self.game_state.player.follower_link.is_bunny_mirror() {
            self.LoadGearPalettes_bunny();
        }
        self.set_hdma_enable_mask(0x80);
        self.hud_refill_logic();
        self.module_pre_dungeon_set_ambient_sfx();
        self.set_submodule(7);
        self.Dungeon_LoadSongBankIfNeeded();
    }

    pub(super) fn Dungeon_InterRoomTrans_State10(&mut self) {
        if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
            self.ApplyPaletteFilter_bounce();
        }
        self.Dungeon_InterRoomTrans_notDarkRoom();
    }

    pub(super) fn Dungeon_SpiralStaircase11(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.increment_subsubmodule();
    }

    pub(super) fn Dungeon_InterRoomTrans_notDarkRoom(&mut self) {
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.increment_subsubmodule();
    }

    pub(super) fn Dungeon_InterRoomTrans_State9(&mut self) {
        if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
            self.ApplyPaletteFilter_bounce();
        }
        self.Dungeon_InterRoomTrans_State4();
    }

    pub(super) fn Dungeon_SpiralStaircase12(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.Dungeon_PrepareNextRoomQuadrantUpload();
        self.increment_subsubmodule();
    }

    pub(super) fn Dungeon_InterRoomTrans_State4(&mut self) {
        self.Dungeon_PrepareNextRoomQuadrantUpload();
        self.increment_subsubmodule();
    }

    pub(super) fn Dungeon_InterRoomTrans_State12(&mut self) {
        if self.game_state.frame.submodule == 2 {
            if self.overworld_map_state() != 5 {
                return;
            }
            self.SubtileTransitionCalculateLanding();
            if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
                self.ApplyPaletteFilter_bounce();
            }
        }
        self.increment_subsubmodule();
        self.Dungeon_ResetTorchBackgroundAndPlayer();
    }

    pub(super) fn Dungeon_Staircase14(&mut self) {
        self.increment_subsubmodule();
        self.Dungeon_ResetTorchBackgroundAndPlayer();
    }

    pub(super) fn Dungeon_InterRoomTrans_State7(&mut self) {
        self.MirrorBg1Bg2Offs();
        if self.game_state.world.location.dungeon_room() != 54
            && self.game_state.world.location.dungeon_room() != 56
        {
            let y = if SPIRAL_SUBSCREEN_LAYER_BY_BG2
                [self.game_state.dungeon.room_load.bg2_properties() as usize]
                != 0
            {
                0x0116
            } else {
                0x0016
            };
            let tm_ts = self.game_state.display.main_screen_layers as u16
                | ((self.game_state.display.sub_screen_layers as u16) << 8);
            if y != tm_ts
                && (self.game_state.display.main_screen_layers == 0x17
                    || (self.game_state.display.main_screen_layers
                        | self.game_state.display.sub_screen_layers)
                        != 0x17)
            {
                self.set_main_screen_layers(y as u8);
                self.set_sub_screen_layers((y >> 8) as u8);
            }
        }
        self.DungeonTransition_RunFiltering();
    }

    pub(super) fn DungeonTransition_FindSubtileLanding(&mut self) {
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        self.SubtileTransitionCalculateLanding();
        self.increment_subsubmodule();
        let room = self.game_state.world.location.dungeon_room() as usize;
        let visited = self.game_state.dungeon.room_load.quadrants_visited();
        self.save_progress_mut().or_dungeon_info_word(room, visited);
    }

    pub(super) fn SubtileTransitionCalculateLanding(&mut self) {
        let st = self.screen_transition();
        let mut a = self.CalculateTransitionLanding();
        if a == 2 {
            a = 1;
        } else if a == 4 {
            a = 2;
        }
        let index = a as usize + self.screen_transition() as usize * 5;
        let mut v = STAIRCASE_LANDING_COORDINATES[index];
        if v < 0 {
            v = v.wrapping_add(8);
        } else {
            v = v.wrapping_sub(8);
        }
        if st & 2 != 0 {
            self.follower_link_state_mut().set_x_low(v as u8);
        } else {
            self.follower_link_state_mut().set_y_low(v as u8);
        }
        self.follower_link_state_mut().set_visibility_status(0);
    }

    pub(super) fn Dungeon_IntraRoomTrans_State5(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        if !self.DungeonTransition_MoveLinkOutDoor() {
            return;
        }
        if self.game_state.dungeon.room_runtime.landing_class_is_pit() {
            self.follower_link_state_mut().clear_doorway_state();
        }
        self.follower_link_state_mut()
            .set_force_move_any_direction(0);
        self.dungeon_room_runtime_mut().clear_landing_class();
        self.clear_screen_transition();
        self.increment_subsubmodule();
    }

    pub(super) fn DungeonTransition_MoveLinkOutDoor(&mut self) -> bool {
        let index = self.game_state.dungeon.room_runtime.landing_class() as usize
            + self.screen_transition() as usize * 5;
        let target = STAIRCASE_LANDING_COORDINATES[index] as u8;
        let step = if self.screen_transition() & 1 != 0 {
            (-2i16) as u16
        } else {
            2
        };
        if self.screen_transition() & 2 == 0 {
            let y = self.game_state.player.follower_link.y().wrapping_add(step);
            self.follower_link_state_mut().set_y(y);
            (y as u8 & 0xfe) == target
        } else {
            let x = self.game_state.player.follower_link.x().wrapping_add(step);
            self.follower_link_state_mut().set_x(x);
            (x as u8 & 0xfe) == target
        }
    }

    pub(super) fn DungeonTransition_Subtile_PrepTransition(&mut self) {
        self.set_darkening_or_lightening_screen_word(0);
        self.set_countdown_word(0);
        self.set_mosaic_target_level(31);
        self.dungeon_room_runtime_mut().clear_reserved_gfx_config();
        self.dungeon_environment_mut()
            .clear_somaria_block_switch_counter();
        self.dungeon_environment_mut()
            .clear_water_puzzle_state_changed();
        self.increment_subsubmodule();
    }

    pub(super) fn DungeonTransition_Subtile_ApplyFilter(&mut self) {
        if self.game_state.dungeon.torch.wants_lights_out() == 0 {
            self.increment_subsubmodule();
            return;
        }
        self.ApplyPaletteFilter_bounce();
        if self.game_state.display.palette_filter.countdown() != 0 {
            self.ApplyPaletteFilter_bounce();
        }
    }

    pub(super) fn DungeonTransition_Subtile_ResetShutters(&mut self) {
        self.dungeon_environment_mut().set_trapdoors_down_low(0);
        self.dungeon_doors_mut().set_door_animation_step_low(7);
        let bak = self.game_state.frame.submodule;
        self.OperateShutterDoors();
        self.set_submodule(bak);
        self.set_countdown(31);
        self.clear_mosaic_target_level();
        self.increment_subsubmodule();
    }

    pub(super) fn DungeonTransition_Subtile_TriggerShutters(&mut self) {
        self.ResetThenCacheRoomEntryProperties();
        if self.game_state.dungeon.environment.trapdoors_down_low() == 0 {
            self.dungeon_environment_mut().set_trapdoors_down_low(1);
            self.dungeon_doors_mut().clear_current_door_pos();
            self.dungeon_doors_mut().clear_door_animation_step();
            self.set_submodule(5);
        }
    }

    pub(super) fn DungeonTransition_RunFiltering(&mut self) {
        if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
            let torch = if self.game_state.dungeon.torch.wants_lights_out() != 0 {
                self.game_state.dungeon.torch.lit_torches() as usize
            } else {
                3
            };
            self.dungeon_room_effects_mut()
                .set_fixed_color_plusminus(DUNGEON_LIT_TORCH_COLOR_PLUS[torch]);
            self.Dungeon_ApproachFixedColor_variable(
                self.game_state.dungeon.room_effects.fixed_color_plusminus(),
            );
            self.clear_mosaic_target_level();
        }
        self.Dungeon_HandleTranslucencyAndPalette();
    }

    pub(super) fn Module07_02_FadedFilter(&mut self) {
        if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
            self.ApplyPaletteFilter_bounce();
            if self.game_state.display.palette_filter.countdown() != 0 {
                self.ApplyPaletteFilter_bounce();
            }
        } else {
            self.increment_subsubmodule();
        }
    }

    pub(super) fn Dungeon_InterRoomTrans_State15(&mut self) {
        self.ResetThenCacheRoomEntryProperties();
        if self.game_state.dungeon.environment.trapdoors_down_low() == 0
            && (self.game_state.world.location.dungeon_room_index() != 172
                || self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x3000 != 0)
        {
            self.dungeon_environment_mut().set_trapdoors_down_low(1);
            // C writes `ram[DUNG_CUR_DOOR_POS_DUNGEON] = 0` as a byte here (0x68e only),
            // leaving the high byte 0x68f intact (a stale leftover).
            self.dungeon_doors_mut().clear_current_door_pos_low_byte();
            self.dungeon_doors_mut().clear_door_animation_step();
            self.set_submodule(5);
        }
        self.Dungeon_PlayMusicIfDefeated();
    }

    pub(super) fn DungeonTransition_LoadSpriteGFX(&mut self) {
        self.LoadNewSpriteGFXSet();
        self.dungeon_reset_sprites();
        self.DungeonTransition_RunFiltering();
    }

    pub(super) fn Module07_04_UnlockDoor(&mut self) {
        self.Dungeon_OpeningLockedDoor_Combined(false);
    }

    pub(super) fn Module07_03_OverlayChange(&mut self) {
        let overlay_index = self.game_state.dungeon.room_load.overlay_to_load() as usize;
        let overlay_offs = self.asset_u16(49, overlay_index) as usize;
        let overlay_data = self
            .asset_raw(48)
            .expect("missing dungeon room overlay asset")
            .to_vec();
        let overlay = &overlay_data[overlay_offs..];
        self.Dungeon_DrawRoomOverlay(overlay);
        let mut dst_pos = 0usize;
        let mut offset = 0usize;
        loop {
            let marker = overlay[offset] as u16 | ((overlay[offset + 1] as u16) << 8);
            if marker == 0xffff {
                break;
            }
            let p =
                ((overlay[offset] as u16 >> 2) | ((overlay[offset + 1] as u16 >> 2) << 6)) as usize;
            dst_pos = self.dungeon_prep_overlay_dma_next_prep(dst_pos, (p * 2) as u16);
            self.Dungeon_DrawRoomOverlay_Apply(p);
            offset += 3;
        }
        self.request_nmi_copy_packets();
        self.set_submodule(0);
    }

    pub(super) fn Module07_05_ControlShutters(&mut self) {
        self.OperateShutterDoors();
    }

    pub(super) fn Module07_06_FatInterRoomStairs(&mut self) {
        if self.game_state.frame.subsubmodule >= 3 {
            self.Dungeon_LoadAttribute_Selectable();
        }

        if self.game_state.frame.subsubmodule >= 13 {
            self.Graphics_IncrementalVRAMUpload();
            if self
                .game_state
                .dungeon
                .stair_movement
                .staircase_move_counter()
                == 0
            {
                self.Module07_06_FatInterRoomStairs_dispatch();
                return;
            }
            if self
                .game_state
                .dungeon
                .stair_movement
                .staircase_move_counter()
                == 0x10
            {
                self.follower_link_state_mut().set_speed_modifier(2);
            }
            self.dungeon_stair_movement_mut()
                .decrement_staircase_move_counter();
            let direction = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                4
            } else {
                8
            };
            self.follower_link_state_mut().set_direction(direction);
            self.link_handle_velocity();
            self.dungeon_handle_camera();
        }

        self.link_handle_moving_animation_full_long_entry();
        self.Module07_06_FatInterRoomStairs_dispatch();
    }

    fn Module07_06_FatInterRoomStairs_dispatch(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => self.ResetTransitionPropsAndAdvance_ResetInterface(),
            1 => {
                self.ApplyPaletteFilter_bounce();
                if self.game_state.display.palette_filter.countdown() != 0 {
                    self.ApplyPaletteFilter_bounce();
                }
            }
            2 => self.Dungeon_InitializeRoomFromSpecial(),
            3 => self.DungeonTransition_TriggerBGC34UpdateAndAdvance(),
            4 => self.DungeonTransition_TriggerBGC56UpdateAndAdvance(),
            5 => self.DungeonTransition_LoadSpriteGFX(),
            6 => self.DungeonTransition_AdjustForFatStairScroll(),
            7 => self.Dungeon_InterRoomTrans_State4(),
            8 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            9 => self.Dungeon_InterRoomTrans_State4(),
            10 => self.Dungeon_SpiralStaircase11(),
            11 => self.Dungeon_SpiralStaircase12(),
            12 => self.Dungeon_SpiralStaircase11(),
            13 => self.Dungeon_SpiralStaircase12(),
            14 => self.Dungeon_DoubleApplyAndIncrementGrayscale(),
            15 => self.Dungeon_Staircase14(),
            16 => {
                if (self
                    .game_state
                    .display
                    .palette_filter
                    .darkening_or_lightening_screen()
                    | self.game_state.display.palette_filter.countdown())
                    == 0
                    && self.overworld_map_state() == 5
                {
                    self.ResetThenCacheRoomEntryProperties();
                }
            }
            _ => panic!("invalid fat inter-room stair index"),
        }
    }

    pub(super) fn Dungeon_InitializeRoomFromSpecial(&mut self) {
        self.Dungeon_AdjustAfterSpiralStairs();
        self.Dungeon_LoadRoom();
        self.ResetStarTileGraphics();
        self.LoadTransAuxGFX();
        self.Dungeon_LoadCustomTileAttr();
        let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
        self.dungeon_room_tracking_mut()
            .set_room_index2(dungeon_room_index);
        self.follower_initialize();
        self.increment_subsubmodule();
    }

    pub(super) fn DungeonTransition_AdjustForFatStairScroll(&mut self) {
        self.MirrorBg1Bg2Offs();
        self.Dungeon_AdjustForRoomLayout();
        let mut ts = SPIRAL_SUBSCREEN_LAYER_BY_BG2
            [self.game_state.dungeon.room_load.bg2_properties() as usize];
        let mut tm = 0x16;
        if ts < 0 {
            tm = 0x17;
            ts = 0;
        }
        self.set_main_screen_layers(tm);
        self.set_sub_screen_layers(ts as u8);

        self.follower_link_state_mut().set_speed_modifier(1);
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            self.dungeon_stair_movement_mut().decrement_current_floor();
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(32);
            self.set_sound_effect_1(0x19);
        } else {
            self.dungeon_stair_movement_mut().increment_current_floor();
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(48);
            self.set_sound_effect_1(0x17);
        }
        self.set_sound_effect_2(0x24);
        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
        self.Dungeon_InterRoomTrans_notDarkRoom();
    }

    pub(super) fn Module07_16_UpdatePegs(&mut self) {
        self.increment_subsubmodule();
        if self.game_state.frame.subsubmodule & 3 != 0 {
            return;
        }
        match self.game_state.frame.subsubmodule >> 2 {
            0 | 1 => self.Module07_16_UpdatePegs_Step1(),
            2 => self.Module07_16_UpdatePegs_Step2(),
            3 => self.RecoverPegGFXFromMapping(),
            4 => {
                self.Dungeon_FlipCrystalPegAttribute();
                self.set_subsubmodule(0);
                self.set_submodule(0);
            }
            _ => {}
        }
    }

    pub(super) fn Module07_17_PressurePlate(&mut self) {
        self.decrement_subsubmodule();
        if self.game_state.frame.subsubmodule != 0 {
            return;
        }
        let link_y = self.game_state.player.follower_link.y().wrapping_sub(2);
        self.follower_link_state_mut().set_y(link_y);
        let pos = self
            .game_state
            .dungeon
            .room_items
            .replacement_tile_destination_x2();
        self.Dungeon_UpdateTileMapWithCommonTile(
            i32::from((pos & 0x003f) << 3),
            i32::from((pos >> 3) & 0x01f8),
            0x0e,
        );
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_submodule(saved_module);
    }

    pub(super) fn Module07_18_RescuedMaiden(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => {
                self.PaletteFilter_RestoreBGSubstractiveStrict();
                self.copy_color(
                    (zelda3_palette::Bank::Main, 32),
                    (zelda3_palette::Bank::Main, 0),
                );
                if self
                    .game_state
                    .display
                    .palette_filter
                    .darkening_or_lightening_screen()
                    != 0xff
                {
                    return;
                }
                for i in 0..0x1000usize {
                    self.dungeon_room_tilemaps_mut().set_bg2_tile(i, 0x01ec);
                    self.dungeon_room_tilemaps_mut().set_bg1_tile(i, 0x01ec);
                }
                self.set_bg1_y_offset(0);
                self.set_bg1_x_offset(0);
                self.dungeon_moving_floor_mut().clear_floor_offsets();
                self.clear_screen_transition();
                self.dungeon_room_load_mut().clear_quadrant_upload_index();
                self.increment_subsubmodule();
            }
            1 => {
                self.PaletteFilter_Crystal();
                self.set_sub_screen_layers(1);
                self.follower_link_state_mut().set_immobilized_flag(2);
                let room = self.game_state.world.location.dungeon_room();
                let j = DUNGEON_BOSS_ROOMS
                    .iter()
                    .rposition(|&r| r == room)
                    .expect("rescued maiden room must be a boss room")
                    .checked_sub(4)
                    .expect("rescued maiden boss room index must select a crystal slot");
                let mut dsto = DUNGEON_CRYSTAL_CUTSCENE_TILE_BASES[j] >> 1;
                let mut tile = 0u16;
                for _ in 0..4 {
                    for x in 0..8u16 {
                        self.room_write_bg1(dsto + x, 0x1f80 | tile);
                        self.room_write_bg1(dsto + x + xy(0, 4) as u16, 0x1f88 | tile);
                        tile = tile.wrapping_add(1);
                    }
                    tile = tile.wrapping_add(8);
                    dsto = dsto.wrapping_add(xy(0, 1) as u16);
                }
                self.increment_subsubmodule();
            }
            2 | 4 | 6 | 8 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            3 | 5 | 7 | 9 => self.Dungeon_InterRoomTrans_State4(),
            10 => {
                self.activate_nmi_thread();
                self.Polyhedral_InitializeThread();
                self.CrystalCutscene_Initialize();
                self.set_submodule(0);
                self.set_subsubmodule(0);
            }
            _ => {}
        }
    }

    pub(super) fn Module07_19_MirrorFade(&mut self) {
        self.Overworld_ResetMosaic_alwaysIncrease();
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        self.set_main_module(5);
        self.set_submodule(0);
        self.clear_bg_vram_load_mode();
        self.save_current_music_as_last();
        if self.palette_swap_enabled() {
            self.Palette_RevertTranslucencySwap();
        }
    }

    pub(super) fn Module07_1A_RoomDraw_OpenTriforceDoor_bounce(&mut self) {
        self.follower_link_state_mut().immobilize();
        if self.game_state.dungeon.scratch_word.word() != 0 {
            if self.scratch_word_mut().decrement_ganon_door_bounce_low() != 0 {
                return;
            }
            if self.scratch_word_mut().decrement_high() != 0 {
                return;
            }
            self.set_ambient_sound_effect(21);
            self.follower_link_state_mut().clear_force_hold_sword_up();
            self.follower_link_state_mut().clear_direction_lock();
        }
        self.follower_link_state_mut().clear_immobilized();
        self.increment_subsubmodule();
        if self.game_state.frame.subsubmodule & 3 != 0 {
            return;
        }

        let index = self.game_state.frame.subsubmodule.wrapping_sub(4) as usize >> 2;
        let src = MODULE07_1_A_ROOM_DRAW_OPEN_TRIFORCE_DOOR_BOUNCE_OPEN_GANON_DOOR_TILE_SOURCES
            [index] as usize;
        for i in 0..8u16 {
            self.room_write_bg(
                0x2000,
                xy(44, 3) as u16 + i,
                self.tile_word(src, (i * 4) as usize),
            );
            self.room_write_bg(
                0x2000,
                xy(44, 4) as u16 + i,
                self.tile_word(src, (i * 4 + 1) as usize),
            );
            self.room_write_bg(
                0x2000,
                xy(44, 5) as u16 + i,
                self.tile_word(src, (i * 4 + 2) as usize),
            );
            self.room_write_bg(
                0x2000,
                xy(44, 6) as u16 + i,
                self.tile_word(src, (i * 4 + 3) as usize),
            );
        }

        self.dungeon_prep_overlay_dma_watergate(0, 0x01d8, 0x0881, 8);
        if self.game_state.frame.subsubmodule == 16 {
            self.write_attr2(xy(44, 5), 0x0202);
            self.write_attr2(xy(44, 6), 0x0202);
            self.write_attr2(xy(50, 5), 0x0200);
            self.write_attr2(xy(50, 6), 0x0200);
            for i in (0..6).step_by(2) {
                for y in 0..7 {
                    self.write_attr2(xy(45 + i, y), 0);
                }
            }
            self.room_bounds_mut().set_y_bound(0, (-64i16) as u16);
            self.set_submodule(0);
            self.set_subsubmodule(0);
        }
        self.request_nmi_copy_packets();
    }

    pub(super) fn DungeonTransition_ScrollRoom(&mut self) {
        self.increment_transition_counter();
        let i = self.screen_transition() as usize;
        self.set_bg1_y_offset(0);
        self.set_bg1_x_offset(0);
        let delta = DUNGEON_TRANSITION_SCROLL_DELTAS[i] as i16 as u16;

        let t = if i >= 2 {
            let t = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add(delta)
                & !1;
            self.set_bg2_x(t);
            self.set_bg1_x(t);
            if self.transition_counter() >= DUNGEON_TRANSITION_PLAYER_MOVE_FRAMES[i] {
                let x = self.game_state.player.follower_link.x().wrapping_add(delta);
                self.follower_link_state_mut().set_x(x);
            }
            t
        } else {
            let t = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_add(delta)
                & !1;
            self.set_bg2_y(t);
            self.set_bg1_y(t);
            if self.transition_counter() >= DUNGEON_TRANSITION_PLAYER_MOVE_FRAMES[i] {
                let y = self.game_state.player.follower_link.y().wrapping_add(delta);
                self.follower_link_state_mut().set_y(y);
            }
            t
        };

        if (t & 0x01fc)
            == self
                .game_state
                .world
                .camera_boundaries
                .up_down_scroll_target(i)
        {
            self.SetAndSaveVisitedQuadrantFlags();
            self.increment_subsubmodule();
            self.set_transition_counter(0);
            if self.game_state.frame.submodule == 2 {
                self.WaterFlood_BuildOneQuadrantForVRAM();
            }
        }
    }

    pub(super) fn DungeonTransition_TriggerBGC34UpdateAndAdvance(&mut self) {
        self.PrepTransAuxGfx();
        self.set_pending_nmi_subroutine(9);
        self.set_core_update_disable_flag(9);
        self.increment_subsubmodule();
    }

    pub(super) fn DungeonTransition_TriggerBGC56UpdateAndAdvance(&mut self) {
        self.set_pending_nmi_subroutine(10);
        self.set_core_update_disable_flag(10);
        self.increment_subsubmodule();
    }

    pub(super) fn Dungeon_TryScreenEdgeTransition(&mut self) {
        let mut dir = None;

        if self.game_state.player.follower_link.y_velocity() != 0 {
            let y = self.game_state.player.follower_link.y() & 0x01ff;
            if y < 4 {
                dir = Some(3);
            } else if y >= 476 {
                dir = Some(2);
            }
        }

        if dir.is_none() && self.game_state.player.follower_link.x_velocity() != 0 {
            let x = self.game_state.player.follower_link.x() & 0x01ff;
            if x < 8 {
                dir = Some(1);
            } else if x >= 489 {
                dir = Some(0);
            }
        }

        let Some(dir) = dir else {
            return;
        };

        if !self.link_check_for_edge_screen_transition() && self.game_state.frame.main_module == 7 {
            self.Dungeon_HandleEdgeTransitionMovement(dir);
            if self.game_state.frame.main_module == 7 {
                self.set_submodule(2);
            }
        }
    }

    pub(super) fn Dungeon_HandleEdgeTransitionMovement(&mut self, dir: u8) {
        self.follower_link_state_mut().mask_direction(
            DUNGEON_HANDLE_EDGE_TRANSITION_MOVEMENT_LIMIT_DIRECTION_ON_ONE_AXIS[dir as usize],
        );
        match dir {
            0 => self.Dungeon_StartInterRoomTrans_Right(),
            1 => self.Dungeon_StartInterRoomTrans_Left(),
            2 => self.Dungeon_StartInterRoomTrans_Down(),
            3 => self.Dungeon_StartInterRoomTrans_Up(),
            _ => unreachable!(),
        }
    }

    pub(super) fn Dungeon_AdjustAfterSpiralStairs(&mut self) {
        let room = self.game_state.world.location.dungeon_room();
        let prev = self
            .game_state
            .dungeon
            .room_tracking
            .previous_room_index_word();
        let xd = ((room & 0x000f) as i32 - (prev & 0x000f) as i32) * 0x200;
        self.add_dungeon_room_delta_x(xd as i16 as u16);

        let yd = (((room & 0x00f0) >> 4) as i32 - ((prev & 0x00f0) >> 4) as i32) * 0x200;
        self.add_dungeon_room_delta_y(yd as i16 as u16);
    }

    pub(super) fn Dungeon_AdjustForTeleportDoors(&mut self, room: u8, flag: u8) {
        self.dungeon_room_tracking_mut()
            .set_room_index2_word(room as u16);
        self.dungeon_room_tracking_mut()
            .set_previous_room_index_word(room as u16);

        let link_x_hi = self.game_state.player.follower_link.x() >> 8;
        let xx = ((room & 0x0f) as u16)
            .wrapping_mul(2)
            .wrapping_sub(link_x_hi)
            .wrapping_add(flag as u16);
        self.add_dungeon_room_delta_x(xx << 8);

        let link_y_hi = self.game_state.player.follower_link.y() >> 8;
        let yy = (((room & 0xf0) >> 3) as u16).wrapping_sub(link_y_hi);
        self.add_dungeon_room_delta_y(yy << 8);

        let y_hi = (self.game_state.player.follower_link.y() >> 8) as u8;
        for i in 0..20 {
            self.tagalong_slot_mut(i).set_y_high(y_hi);
        }
    }

    fn add_dungeon_room_delta_x(&mut self, delta: u16) {
        let link_x = self.game_state.player.follower_link.x().wrapping_add(delta);
        self.follower_link_state_mut().set_x(link_x);
        self.add_bg2_h_copy2(delta);
        for index in 0..4 {
            let value = self
                .game_state
                .world
                .room_bounds
                .x_bound(index)
                .wrapping_add(delta);
            self.room_bounds_mut().set_x_bound(index, value);
        }
    }

    fn add_dungeon_room_delta_y(&mut self, delta: u16) {
        let link_y = self.game_state.player.follower_link.y().wrapping_add(delta);
        self.follower_link_state_mut().set_y(link_y);
        self.add_bg2_v_copy2(delta);
        for index in 0..4 {
            let value = self
                .game_state
                .world
                .room_bounds
                .y_bound(index)
                .wrapping_add(delta);
            self.room_bounds_mut().set_y_bound(index, value);
        }
    }

    pub(super) fn Ganon_ExtinguishTorch_adjust_translucency(&mut self) {
        self.Palette_AssertTranslucencySwap();
        self.dungeon_torch_mut().set_attr(0xc0);
        self.Dungeon_ExtinguishTorch();
    }

    pub(super) fn Ganon_ExtinguishTorch(&mut self) {
        self.dungeon_torch_mut().set_attr(193);
        self.Dungeon_ExtinguishTorch();
    }

    pub(super) fn Dungeon_ExtinguishTorch(&mut self) {
        let y = ((self.game_state.dungeon.torch.torch_attr() & 0x0f) as usize) * 2
            + self.game_state.dungeon.torch.torches_start_index() as usize;
        let idx = y >> 1;
        let mut r8 = self
            .game_state
            .dungeon
            .object_tracking
            .object_tilemap_pos(idx)
            & 0x7fff;
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(idx, r8);

        let obj_pos = (self
            .game_state
            .dungeon
            .object_tracking
            .object_pos_in_objdata(idx)
            & 0x00ff)
            >> 1;
        self.dungeon_torch_mut()
            .set_torch_data_word_index(obj_pos as usize, r8);

        r8 &= 0x3fff;
        self.room_draw_adjust_torch_lighting_change(r8, 0x0ec2, r8);
        self.request_nmi_copy_packets();

        if self.game_state.dungeon.torch.wants_lights_out() != 0
            && self.game_state.dungeon.torch.lit_torches() != 0
        {
            self.dungeon_torch_mut().decrement_lit_torches();
            if self.game_state.dungeon.torch.lit_torches() < 3 {
                if self.game_state.dungeon.torch.lit_torches() == 0 {
                    self.set_sub_screen_layers(1);
                }
                let torch = self.game_state.dungeon.torch.lit_torches() as usize;
                self.dungeon_room_effects_mut()
                    .set_fixed_color_plusminus(DUNGEON_LIT_TORCH_COLOR_PLUS[torch]);
                self.set_submodule(10);
                self.set_subsubmodule(0);
            }
        }

        let torch_timer = (self.game_state.dungeon.torch.torch_attr() & 0x0f) as usize;
        self.dungeon_torch_mut().clear_timer(torch_timer);
        self.dungeon_torch_mut().clear_attr();
    }

    fn set_spiral_stair_wall_priority(&mut self, pos: u16, high: bool) {
        let mask = if high { 0x2000 } else { 0xdfff };
        for i in 0..5usize {
            for y in 0..4usize {
                let index = (pos as usize + i + y * 64) as u16;
                // C reads/writes `dung_bg2[index]` flat; for `index >= 0x1000`
                // (a lower-level staircase wall) that spills into the contiguous
                // BG1 span. `bg2_tile`/`set_bg2_tile` would read 0 and drop the
                // OOB write, so the priority clear became a no-op and the wall
                // stayed high-priority. Route through the spill-aware path.
                let value = self
                    .game_state
                    .dungeon
                    .room_tilemaps
                    .room_tilemap_word(crate::game_state::constants::DUNG_BG2, index);
                let value = if high { value | mask } else { value & mask };
                self.dungeon_room_tilemaps_mut().set_room_tilemap_word(
                    crate::game_state::constants::DUNG_BG2,
                    index,
                    value,
                );
            }
        }
    }

    pub(super) fn SpiralStairs_MakeNearbyWallsHighPriority_Entering(&mut self) {
        let index = (self.game_state.dungeon.stair_movement.staircase_index() & 3) as usize;
        let pos = self
            .game_state
            .dungeon
            .stair_lists
            .inter_staircase_pos(index)
            .wrapping_sub(4);
        self.dungeon_stair_movement_mut()
            .set_staircase_tilemap_pos_x2(pos.wrapping_mul(2));
        self.set_spiral_stair_wall_priority(pos, true);
        let dma_ptr = self.dungeon_prep_overlay_dma_next_prep(0, pos.wrapping_mul(2));
        self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos.wrapping_mul(2).wrapping_add(8));
        self.request_nmi_copy_packets();
    }

    pub(super) fn SpiralStairs_MakeNearbyWallsLowPriority(&mut self) {
        let pos = self
            .game_state
            .dungeon
            .stair_movement
            .staircase_tilemap_pos_x2()
            >> 1;
        self.set_spiral_stair_wall_priority(pos, false);
        let dma_ptr = self.dungeon_prep_overlay_dma_next_prep(0, pos.wrapping_mul(2));
        self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos.wrapping_mul(2).wrapping_add(8));
        self.request_nmi_copy_packets();
    }

    pub(super) fn SpiralStairs_MakeNearbyWallsHighPriority_Exiting(&mut self) {
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            return;
        }
        let lf = self
            .game_state
            .dungeon
            .stair_movement
            .staircase_tilemap_pos_x2()
            .wrapping_add(8)
            & 0x007f;
        let mut x = 0usize;
        let mut p;
        loop {
            p = self.game_state.dungeon.stair_lists.inter_staircase_pos(x);
            if (p.wrapping_mul(2) & 0x007f) == lf {
                break;
            }
            x += 1;
        }
        p = p.wrapping_sub(4);
        self.dungeon_stair_movement_mut()
            .set_staircase_tilemap_pos_x2(p.wrapping_mul(2));
        self.set_spiral_stair_wall_priority(p, true);
    }

    pub(super) fn Module07_0F_00_InitSpotlight(&mut self) {
        self.Spotlight_open();
        self.increment_subsubmodule();
    }

    pub(super) fn Module07_0F_01_OperateSpotlight(&mut self) {
        self.sprite_main();
        self.IrisSpotlight_ConfigureTable();
        self.complete_module07_0f_operate_spotlight_suffix();
    }

    pub(super) fn complete_module07_0f_operate_spotlight_suffix(&mut self) {
        if self.game_state.frame.submodule == 0 {
            self.clear_window_layer_masks();
            self.clear_window_main_sub_masks();
            self.set_subsubmodule(0);
            let queued_music_control = self.game_state.system_signals.queued_music_control();
            if queued_music_control != 0xff {
                self.set_music_control(queued_music_control);
            }
        }
    }

    pub(super) fn Module07_0F_LandingWipe(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => self.Module07_0F_00_InitSpotlight(),
            1 => self.Module07_0F_01_OperateSpotlight(),
            other => panic!("invalid Module07_0F_LandingWipe subsubmodule_index {other}"),
        }
        self.link_handle_moving_animation_full_long_entry();
        self.link_oam_main();
        if self.rom_startup_timing()
            && rom_dungeon_landing_wipe_is_active(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            )
        {
            self.dungeon_landing_wipe_carry_pending = true;
        }
    }

    pub(super) fn Module07_10_SouthIntraRoomStairs(&mut self) {
        let t = self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter();
        if t != 0 {
            self.dungeon_stair_movement_mut()
                .decrement_staircase_move_counter();
            if t == 20 {
                self.follower_link_state_mut().set_speed_modifier(2);
            }
            self.link_handle_velocity();
            self.apply_links_movement_to_camera();
            self.dungeon_handle_camera();
            self.link_handle_moving_animation_full_long_entry();
        }
        match self.game_state.frame.subsubmodule {
            0 => self.Module07_10_00_InitStairs(),
            1 => self.Module07_10_01_ClimbStairs(),
            other => panic!("invalid Module07_10_SouthIntraRoomStairs subsubmodule_index {other}"),
        }
    }

    pub(super) fn Module07_09_OpenCrackedDoor(&mut self) {
        self.OpenCrackedDoor();
    }

    pub(super) fn Module07_10_00_InitStairs(&mut self) {
        let mut v1 = 0x3c;
        let mut sfx = 25;
        if self.game_state.player.follower_link.direction() & 4 != 0 {
            v1 = 0x38;
            sfx = 23;
            self.follower_link_state_mut()
                .toggle_lower_level_mirror_state();
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                != 2
            {
                self.follower_link_state_mut().toggle_lower_level_state();
            }
        }
        self.dungeon_stair_movement_mut()
            .set_staircase_move_counter(v1);
        self.set_sound_effect_1(sfx);
        self.follower_link_state_mut().set_speed_modifier(1);
        self.increment_subsubmodule();
    }

    pub(super) fn Module07_10_01_ClimbStairs(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            != 0
        {
            return;
        }
        if self.game_state.player.follower_link.direction() & 8 != 0 {
            self.follower_link_state_mut()
                .toggle_lower_level_mirror_state();
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                != 2
            {
                self.follower_link_state_mut().toggle_lower_level_state();
            }
        }
        self.set_subsubmodule(0);
        self.clear_screen_transition();
        self.set_submodule(0);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn Module07_08_NorthIntraRoomStairs(&mut self) {
        let t = self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter();
        if t != 0 {
            self.dungeon_stair_movement_mut()
                .decrement_staircase_move_counter();
            if t == 20 {
                self.follower_link_state_mut().set_speed_modifier(2);
            }
            self.link_handle_velocity();
            self.apply_links_movement_to_camera();
            self.dungeon_handle_camera();
            self.link_handle_moving_animation_full_long_entry();
        }
        match self.game_state.frame.subsubmodule {
            0 => self.Module07_08_00_InitStairs(),
            1 => self.Module07_08_01_ClimbStairs(),
            other => panic!("invalid Module07_08_NorthIntraRoomStairs subsubmodule_index {other}"),
        }
    }

    pub(super) fn Module07_08_00_InitStairs(&mut self) {
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        let mut v1 = 0x3c;
        let mut sfx = 25;
        if self.game_state.player.follower_link.direction() & 8 != 0 {
            v1 = 0x38;
            sfx = 23;
            self.follower_link_state_mut()
                .set_lower_level_mirror_state(0);
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                != 2
            {
                self.follower_link_state_mut().clear_lower_level();
            }
        }
        self.dungeon_stair_movement_mut()
            .set_staircase_move_counter(v1);
        self.set_sound_effect_1(sfx);
        self.follower_link_state_mut().set_speed_modifier(1);
        self.increment_subsubmodule();
    }

    pub(super) fn Module07_08_01_ClimbStairs(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            != 0
        {
            return;
        }
        if self.game_state.player.follower_link.direction() & 4 != 0 {
            self.follower_link_state_mut()
                .set_lower_level_mirror_state(1);
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                != 2
            {
                self.follower_link_state_mut().mark_lower_level();
            }
        }
        self.set_subsubmodule(0);
        self.clear_screen_transition();
        self.set_submodule(0);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn Module07_11_StraightInterroomStairs(&mut self) {
        if self.game_state.frame.subsubmodule >= 3 {
            self.Dungeon_LoadAttribute_Selectable();
        }
        if self.game_state.frame.subsubmodule >= 13 {
            self.Graphics_IncrementalVRAMUpload();
        }
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            != 0
        {
            if self
                .game_state
                .dungeon
                .stair_movement
                .staircase_move_counter()
                == 16
            {
                self.follower_link_state_mut().set_speed_modifier(2);
            }
            self.dungeon_stair_movement_mut()
                .decrement_staircase_move_counter();
            let direction = if self.game_state.frame.submodule == 18 {
                8
            } else {
                4
            };
            self.follower_link_state_mut().set_direction(direction);
            self.link_handle_velocity();
        }
        self.link_handle_moving_animation_full_long_entry();
        match self.game_state.frame.subsubmodule {
            0 => self.Module07_11_00_PrepAndReset(),
            1 => self.Module07_11_01_FadeOut(),
            2 => self.Module07_11_02_LoadAndPrepRoom(),
            3 => self.Module07_11_03_FilterAndLoadBGChars(),
            4 => self.Module07_11_04_FilterDoBGAndResetSprites(),
            5 => self.Dungeon_SpiralStaircase11(),
            6 => self.Dungeon_SpiralStaircase12(),
            7 => self.Dungeon_SpiralStaircase11(),
            8 => self.Dungeon_SpiralStaircase12(),
            9 => self.Module07_11_09_LoadSpriteGraphics(),
            10 => self.Module07_11_0A_ScrollCamera(),
            11 => self.Module07_11_0B_PrepDestination(),
            12 => self.Dungeon_InterRoomTrans_State4(),
            13 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            14 => self.Dungeon_InterRoomTrans_State4(),
            15 => self.Dungeon_DoubleApplyAndIncrementGrayscale(),
            16 => self.Module07_11_19_SetSongAndFilter(),
            17 => self.Module07_11_11_KeepSliding(),
            18 => self.ResetThenCacheRoomEntryProperties(),
            other => {
                panic!("invalid Module07_11_StraightInterroomStairs subsubmodule_index {other}")
            }
        }
    }

    pub(super) fn Module07_11_00_PrepAndReset(&mut self) {
        if self.game_state.player.follower_link.is_running() {
            self.follower_link_state_mut().clear_running();
            self.follower_link_state_mut().set_speed_setting(2);
        }
        let sound_effect_1 = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            24
        } else {
            22
        };
        self.set_sound_effect_1(sound_effect_1);
        let room = self.game_state.world.location.dungeon_room();
        if room == 48 || room == 64 {
            self.set_music_control(0xf1);
        }
        self.ResetTransitionPropsAndAdvance_ResetInterface();
    }

    pub(super) fn Module07_11_01_FadeOut(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            < 9
        {
            self.ApplyPaletteFilter_bounce();
            if self.game_state.display.palette_filter.countdown() == 23 {
                self.increment_subsubmodule();
            }
        }
    }

    pub(super) fn Module07_11_02_LoadAndPrepRoom(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.Dungeon_LoadRoom();
        self.Dungeon_RestoreStarTileChr();
        self.LoadTransAuxGFX();
        self.Dungeon_LoadCustomTileAttr();
        self.Dungeon_AdjustForRoomLayout();
        self.follower_initialize();
        self.increment_subsubmodule();
    }

    pub(super) fn Module07_11_03_FilterAndLoadBGChars(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.DungeonTransition_TriggerBGC34UpdateAndAdvance();
    }

    pub(super) fn Module07_11_04_FilterDoBGAndResetSprites(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.DungeonTransition_TriggerBGC56UpdateAndAdvance();
        let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
        self.dungeon_room_tracking_mut()
            .set_room_index2(dungeon_room_index);
        self.dungeon_reset_sprites();
    }

    pub(super) fn Module07_11_09_LoadSpriteGraphics(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.decrement_subsubmodule();
        self.LoadNewSpriteGFXSet();
        self.Dungeon_HandleTranslucencyAndPalette();
    }

    pub(super) fn Module07_11_0A_ScrollCamera(&mut self) {
        self.follower_link_state_mut().set_visibility_status(12);
        self.follower_state_mut().set_appearance_none_flag(12);
        let mut i = self.screen_transition() as usize;
        let y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add(DUNGEON_TRANSITION_SCROLL_DELTAS[i] as i16 as u16)
            & !3;
        self.set_bg1_y(y);
        self.set_bg2_y(y);
        if (y & 0x01fc)
            == self
                .game_state
                .world
                .camera_boundaries
                .up_down_scroll_target(i)
        {
            if self.game_state.frame.submodule >= 18 {
                i += 2;
            }
            let link_y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(STAIRCASE_CAMERA_LINK_Y_ADJUSTMENTS[i] as i16 as u16);
            self.follower_link_state_mut().set_y(link_y);
            self.follower_link_state_mut().set_visibility_status(0);
            self.follower_state_mut().set_appearance_none_flag(0);
            self.increment_subsubmodule();
        }
    }

    pub(super) fn Module07_11_0B_PrepDestination(&mut self) {
        let mut ts = SPIRAL_SUBSCREEN_LAYER_BY_BG2
            [self.game_state.dungeon.room_load.bg2_properties() as usize];
        let mut tm = 0x16;
        if ts < 0 {
            tm = 0x17;
            ts = 0;
        }
        self.set_main_screen_layers(tm);
        self.set_sub_screen_layers(ts as u8);

        self.follower_link_state_mut().set_speed_modifier(1);
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            self.dungeon_stair_movement_mut().decrement_current_floor();
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(0x32);
            self.set_sound_effect_1(25);
        } else {
            self.dungeon_stair_movement_mut().increment_current_floor();
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(0x3c);
            self.set_sound_effect_1(23);
        }

        let mut r0 = 0u8;
        let y_delta = if self.game_state.frame.submodule == 18 {
            (-32i16) as u16
        } else {
            32
        };
        if self.game_state.player.follower_link.is_on_lower_level() {
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(y_delta);
            self.follower_link_state_mut().set_y(y);
            r0 = r0.wrapping_add(1);
        }
        let plane = self
            .game_state
            .dungeon
            .stair_movement
            .current_staircase_plane() as usize;
        self.follower_link_state_mut().set_lower_level_states(
            TELEPORT_PIT_SECONDARY_LEVELS[plane],
            TELEPORT_PIT_PRIMARY_LEVELS[plane],
        );
        if self.game_state.player.follower_link.is_on_lower_level() {
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(y_delta);
            self.follower_link_state_mut().set_y(y);
            r0 = r0.wrapping_add(1);
        }

        if r0 == 0 {
            let delta = if self.game_state.frame.submodule == 18 {
                if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                    (-24i16) as u16
                } else {
                    (-8i16) as u16
                }
            } else {
                12
            };
            let y = self.game_state.player.follower_link.y().wrapping_add(delta);
            self.follower_link_state_mut().set_y(y);
        }

        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
        self.hud_restore_torch_background();
        self.Dungeon_InterRoomTrans_notDarkRoom();
    }

    pub(super) fn Module07_11_19_SetSongAndFilter(&mut self) {
        if self.overworld_map_state() == 5
            && self
                .game_state
                .display
                .palette_filter
                .darkening_or_lightening_screen()
                == 0
        {
            self.increment_subsubmodule();
            let room = self.game_state.world.location.dungeon_room();
            if room == 48 {
                self.set_music_control(0x1c);
            } else if room == 64 {
                self.set_music_control(0x10);
            }
        }
        self.ApplyGrayscaleFixed_Incremental();
    }

    pub(super) fn Module07_11_11_KeepSliding(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            == 0
        {
            self.increment_subsubmodule();
        } else {
            self.ApplyGrayscaleFixed_Incremental();
        }
    }

    pub(super) fn Module07_14_RecoverFromFall(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => self.Module07_14_00_ScrollCamera(),
            1 => self.RecoverPositionAfterDrowning(),
            _ => {}
        }
    }

    pub(super) fn Module07_14_00_ScrollCamera(&mut self) {
        for _ in 0..2 {
            self.step_bg2_h_copy2_toward_cached();
            self.step_bg2_v_copy2_toward_cached();
        }
        if self.game_state.display.ppu_scroll_copy.bg2_h_copy2()
            == self.game_state.display.ppu_scroll_copy.bg2_h_copy2_cached()
            && self.game_state.display.ppu_scroll_copy.bg2_v_copy2()
                == self.game_state.display.ppu_scroll_copy.bg2_v_copy2_cached()
        {
            self.increment_subsubmodule();
        }
        if !self.game_state.dungeon.torch.dungeon_dark_with_lantern() {
            self.MirrorBg1Bg2Offs();
        }
    }

    pub(super) fn HandleLinkOnSpiralStairs(&mut self) {
        self.follower_link_state_mut()
            .cache_previous_position_from_current_xy_order();
        if self.game_state.player.follower_link.y_button_action_step() != 0 {
            return;
        }

        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().clear_auxiliary_state();

        self.follower_link_state_mut()
            .set_actual_y_velocity((-2i8) as u8);
        let push_fatigue_timer = self
            .follower_link_state_mut()
            .decrement_push_fatigue_timer();
        if (push_fatigue_timer as i8).is_negative() {
            self.follower_link_state_mut().set_push_fatigue_timer(0);
            if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                self.follower_link_state_mut()
                    .set_actual_velocity_xy((-2i8) as u8, 0);
            } else {
                self.follower_link_state_mut()
                    .set_actual_velocity_xy(2, (-2i8) as u8);
            }
        }
        self.link_move_position();
        self.link_handle_moving_animation_start_with_dash();
        if self.game_state.player.follower_link.push_fatigue_timer() == 0 {
            if self
                .dungeon_stair_movement_mut()
                .decrement_staircase_countdown_underflowed()
            {
                let facing = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                    4
                } else {
                    6
                };
                self.follower_link_state_mut().set_facing(facing);
            }
        }

        let mut xd = (self.game_state.player.follower_link.x() as u8)
            .wrapping_sub(self.game_state.player.tile_detection.x_low()) as i8;
        if xd < 0 {
            xd = xd.wrapping_neg();
        }
        if xd != 0 {
            return;
        }

        self.RepositionLinkAfterSpiralStairs();
        if self.game_state.sprites.follower_runtime.indicator() != 0 {
            self.follower_initialize();
        }

        let detect_x = self.game_state.player.follower_link.x().wrapping_add(
            if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                (-8i16) as u16
            } else {
                12
            },
        );
        self.tile_detect_position_mut().set_x(detect_x);
        self.follower_link_state_mut().set_y_button_action_step(1);
        self.dungeon_stair_movement_mut().set_staircase_countdown(6);
        self.ancilla_sfx2_near(
            if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                25
            } else {
                23
            },
        );
    }

    pub(super) fn SpiralStairs_FindLandingSpot(&mut self) {
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        self.follower_link_state_mut()
            .cache_previous_position_from_current_xy_order();
        if self
            .dungeon_stair_movement_mut()
            .decrement_staircase_countdown_underflowed()
        {
            self.follower_link_state_mut().set_facing(2);
        }

        self.follower_link_state_mut().set_actual_velocity_xy(4, 0);
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            self.follower_link_state_mut()
                .set_actual_velocity_xy((-4i8) as u8, 2);
        }
        if self.game_state.player.follower_link.y_button_action_step() == 2 {
            self.follower_link_state_mut().set_actual_velocity_xy(0, 16);
        }
        self.link_move_position();
        self.link_handle_moving_animation_start_with_dash();
        if self.game_state.player.follower_link.x() as u8
            == self.game_state.player.tile_detection.x_low()
        {
            self.follower_link_state_mut().set_y_button_action_step(2);
        }
    }
}

fn clamp_c_int16_to_u16(value: u16, max: u16) -> u16 {
    if (value as i16) < 0 {
        0
    } else if value > max {
        max
    } else {
        value
    }
}

fn object_subtype1_param(idx: u8) -> Option<usize> {
    const PARAMS: [usize; 256] = [
        0x03d8, 0x02e8, 0x02f8, 0x0328, 0x0338, 0x0400, 0x0410, 0x0388, 0x0390, 0x0420, 0x042a,
        0x0434, 0x043e, 0x0448, 0x0452, 0x045c, 0x0466, 0x0470, 0x047a, 0x0484, 0x048e, 0x0498,
        0x04a2, 0x04ac, 0x04b6, 0x04c0, 0x04ca, 0x04d4, 0x04de, 0x04e8, 0x04f2, 0x04fc, 0x0506,
        0x0598, 0x0600, 0x063c, 0x063c, 0x063c, 0x063c, 0x063c, 0x0642, 0x064c, 0x0652, 0x0658,
        0x065e, 0x0664, 0x066a, 0x0688, 0x0694, 0x06a8, 0x06a8, 0x06a8, 0x06c8, 0x0000, 0x078a,
        0x07aa, 0x0e26, 0x084a, 0x086a, 0x0882, 0x08ca, 0x085a, 0x08fa, 0x091a, 0x0920, 0x092a,
        0x0930, 0x0936, 0x093c, 0x0942, 0x0948, 0x094e, 0x096c, 0x097e, 0x098e, 0x0902, 0x099e,
        0x09d8, 0x09d8, 0x09d8, 0x09fa, 0x156c, 0x1590, 0x1d86, 0x0000, 0x0a14, 0x0a24, 0x0a54,
        0x0a54, 0x0a84, 0x0a84, 0x14dc, 0x1500, 0x061e, 0x0e52, 0x0600, 0x03d8, 0x02c8, 0x02d8,
        0x0308, 0x0318, 0x03e0, 0x03f0, 0x0378, 0x0380, 0x05fa, 0x0648, 0x064a, 0x0670, 0x067c,
        0x06a8, 0x06a8, 0x06a8, 0x06c8, 0x0000, 0x07aa, 0x07ca, 0x084a, 0x089a, 0x08b2, 0x090a,
        0x0926, 0x0928, 0x0912, 0x09f8, 0x1d7e, 0x0000, 0x0a34, 0x0a44, 0x0a54, 0x0a6c, 0x0a84,
        0x0a9c, 0x1524, 0x1548, 0x085a, 0x0606, 0x0e52, 0x05fa, 0x06a0, 0x06a2, 0x0b12, 0x0b14,
        0x09b0, 0x0b46, 0x0b56, 0x1f52, 0x1f5a, 0x0288, 0x0e82, 0x1df2, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x03d8, 0x03d8, 0x03d8, 0x03d8, 0x05aa,
        0x05b2, 0x05b2, 0x05b2, 0x05b2, 0x00e0, 0x00e0, 0x00e0, 0x00e0, 0x0110, 0x0000, 0x0000,
        0x06a4, 0x06a6, 0x0ae6, 0x0b06, 0x0b0c, 0x0b16, 0x0b26, 0x0b36, 0x1f52, 0x1f5a, 0x0288,
        0x0eba, 0x0e82, 0x1df2, 0x0000, 0x0000, 0x03d8, 0x0510, 0x05aa, 0x05aa, 0x0000, 0x0168,
        0x00e0, 0x0158, 0x0100, 0x0110, 0x0178, 0x072a, 0x072a, 0x072a, 0x075a, 0x0670, 0x0670,
        0x0130, 0x0148, 0x072a, 0x072a, 0x072a, 0x075a, 0x00e0, 0x0110, 0x00f0, 0x0110, 0x0000,
        0x0ab4, 0x08da, 0x0ade, 0x0188, 0x01a0, 0x01b0, 0x01c0, 0x01d0, 0x01e0, 0x01f0, 0x0200,
        0x0120, 0x02a8, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000,
    ];
    Some(PARAMS[idx as usize])
}

fn object_subtype2_param(idx: u8) -> Option<usize> {
    const PARAMS: [usize; 64] = [
        0x0b66, 0x0b86, 0x0ba6, 0x0bc6, 0x0c66, 0x0c86, 0x0ca6, 0x0cc6, 0x0be6, 0x0c06, 0x0c26,
        0x0c46, 0x0ce6, 0x0d06, 0x0d26, 0x0d46, 0x0d66, 0x0d7e, 0x0d96, 0x0dae, 0x0dc6, 0x0dde,
        0x0df6, 0x0e0e, 0x0398, 0x03a0, 0x03a8, 0x03b0, 0x0e32, 0x0e26, 0x0ea2, 0x0e9a, 0x0eca,
        0x0ed2, 0x0ede, 0x0ede, 0x0f1e, 0x0f3e, 0x0f5e, 0x0f6a, 0x0ef6, 0x0f72, 0x0f92, 0x0fa2,
        0x0fa2, 0x1088, 0x10a8, 0x10a8, 0x10c8, 0x10c8, 0x10c8, 0x10c8, 0x0e52, 0x1108, 0x1108,
        0x12a8, 0x1148, 0x1160, 0x1178, 0x1190, 0x1458, 0x1488, 0x2062, 0x2086,
    ];
    PARAMS.get(idx as usize).copied()
}

fn object_subtype3_param(idx: u8) -> Option<usize> {
    const PARAMS: [usize; 128] = [
        0x1614, 0x162c, 0x1654, 0x0a0e, 0x0a0c, 0x09fc, 0x09fe, 0x0a00, 0x0a02, 0x0a04, 0x0a06,
        0x0a08, 0x0a0a, 0x0000, 0x0a10, 0x0a12, 0x1dda, 0x1de2, 0x1dd6, 0x1dea, 0x15fc, 0x1dfa,
        0x1df2, 0x1488, 0x1494, 0x149c, 0x14a4, 0x10e8, 0x10e8, 0x10e8, 0x11a8, 0x11c8, 0x11e8,
        0x1208, 0x03b8, 0x03c0, 0x03c8, 0x03d0, 0x1228, 0x1248, 0x1268, 0x1288, 0x0000, 0x0e5a,
        0x0e62, 0x0000, 0x0000, 0x0e82, 0x0e8a, 0x14ac, 0x14c4, 0x10e8, 0x1614, 0x1614, 0x1614,
        0x1614, 0x1614, 0x1614, 0x1cbe, 0x1cee, 0x1d1e, 0x1d4e, 0x1d8e, 0x1d96, 0x1d9e, 0x1da6,
        0x1dae, 0x1db6, 0x1dbe, 0x1dc6, 0x1dce, 0x0220, 0x0260, 0x0280, 0x1f3a, 0x1f62, 0x1f92,
        0x1ff2, 0x2016, 0x1f42, 0x0eaa, 0x1f4a, 0x1f52, 0x1f5a, 0x202e, 0x2062, 0x09b8, 0x09c0,
        0x09c8, 0x09d0, 0x0fa2, 0x0fb2, 0x0fc4, 0x0ff4, 0x1018, 0x1020, 0x15b4, 0x15d8, 0x20f6,
        0x0eba, 0x22e6, 0x22ee, 0x05da, 0x281e, 0x2ae0, 0x2d2a, 0x2f2a, 0x22f6, 0x2316, 0x232e,
        0x2346, 0x235e, 0x2376, 0x23b6, 0x1e9a, 0x0000, 0x2436, 0x149c, 0x24b6, 0x24e6, 0x2516,
        0x1028, 0x1040, 0x1060, 0x1070, 0x1078, 0x1080, 0x0000,
    ];
    PARAMS.get(idx as usize).copied()
}

fn replay_room_write_trace_addr(offset: usize) -> bool {
    let Ok(raw) = std::env::var("ZELDA3_REPLAY_ROOM_WRITE_TRACE_ADDR") else {
        return false;
    };
    raw.split(',').any(|part| {
        let part = part.trim();
        if part.is_empty() {
            return false;
        }
        let parsed = part
            .strip_prefix("0x")
            .or_else(|| part.strip_prefix("0X"))
            .map_or_else(
                || part.parse::<usize>().ok(),
                |hex| usize::from_str_radix(hex, 16).ok(),
            );
        parsed == Some(offset)
    })
}

fn replay_room_write_trace_enabled() -> bool {
    std::env::var_os("ZELDA3_REPLAY_ROOM_WRITE_TRACE_ADDR").is_some()
}

impl ZeldaState {
    pub(super) fn module_pre_dungeon(&mut self) {
        self.module_pre_dungeon_audio_prefix();
        self.module_pre_dungeon_after_audio_prefix();
    }

    pub(super) fn module_pre_dungeon_audio_prefix(&mut self) {
        self.set_ambient_sound_effect(5);
        self.set_sound_effect_1(0);
    }

    pub(super) fn module_pre_dungeon_after_audio_prefix(&mut self) {
        self.set_dungeon_room(0);
        self.dungeon_room_tracking_mut()
            .set_previous_room_index_word(0);
        self.dungeon_savegame_state_mut().clear_savegame_state_low();
        self.dungeon_savegame_state_mut()
            .clear_savegame_state_high();
        self.clear_agahnim_palette_settings(12);

        self.Dungeon_LoadEntrance();
        self.load_pre_dungeon_keys();
        self.hud_rebuild();
        self.dungeon_torch_mut().clear_lit_torches();
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
        self.Dungeon_LoadAndDrawRoom();
        self.Dungeon_LoadCustomTileAttr();

        let animated = DUNG_ANIMATED_TILES
            [self.game_state.world.palette_theme.main_tile_theme_index() as usize];
        self.decompress_animated_dungeon_tiles(animated as usize);
        self.Dungeon_LoadAttributeTable();
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(10);
        self.initialize_tilesets();
        // PALETTE_SP6R_INDOORS (0xab1) is read by palette_load_sprite_environment_dungeon
        // off display.palette_buffer, not world.palette_theme. Set the display owner so
        // the immediately-following dungeon_load_palettes() loads the selector-10 sprite
        // environment palette instead of a stale selector (matches raw-RAM single-byte).
        self.set_sp6r_indoors(10);
        self.dungeon_load_palettes();

        let room = self.game_state.world.location.dungeon_room();
        self.dungeon_room_load_mut()
            .set_loading_bg_offsets((room & 0x000f) << 9, (room & 0x0ff0) << 5);
        if room == 0x0104 && self.game_state.inventory.save_progress.progress_flags() & 0x10 != 0 {
            self.dungeon_torch_mut().clear_lights_out_request();
        }
        self.SetAndSaveVisitedQuadrantFlags();

        self.set_color_window_selection(2);
        let mut torch = self.game_state.dungeon.torch.lit_torches() as usize;
        let color_math_control = if self.game_state.dungeon.torch.wants_lights_out() == 0 {
            torch = 3;
            if self.game_state.dungeon.room_load.bg2_properties() == 7 {
                0x32
            } else if self.game_state.dungeon.room_load.bg2_properties() == 4 {
                0x62
            } else {
                0x20
            }
        } else {
            0xb3
        };
        self.set_color_math_control(color_math_control);
        self.dungeon_room_effects_mut()
            .set_fixed_color_plusminus(MODULE_PRE_DUNGEON_LIT_TORCHES_COLOR_PLUS[torch]);
        self.Dungeon_ApproachFixedColor_variable(
            self.game_state.dungeon.room_effects.fixed_color_plusminus(),
        );
        self.set_countdown(0x1f);
        self.clear_mosaic_target_level();
        self.set_darkening_or_lightening_screen(2);
        self.clear_overworld_aux_or_main_offset();
        self.follower_link_state_mut().clear_speed_modifier();
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().clear_button_b_frames();
        self.Dungeon_ResetTorchBackgroundAndPlayer();
        self.link_check_bunny_status();
        self.reset_then_cache_room_entry_properties();
        if self.game_state.sprites.follower_runtime.indicator() == 13 {
            self.follower_state_mut().set_indicator(0);
            self.set_super_bomb_indicator_timer(0);
            self.hud_remove_super_bomb_indicator();
        }
        self.set_bg_mode(9);
        self.follower_initialize();
        self.sprite_reset_all();
        self.dungeon_reset_sprites();
        self.messaging_state_mut()
            .clear_message_or_sprite_state_cache();
        self.dungeon_room_runtime_mut().skip_room_tags_once();

        if self.game_state.inventory.save_progress.progress_indicator() == 0
            && self.game_state.inventory.save_progress.progress_flags() & 0x10 == 0
        {
            self.set_fixed_color_red(0x30);
            self.set_fixed_color_green(0x50);
            self.set_fixed_color_blue(0x80);
            self.dungeon_torch_mut().clear_lights_out_requests();
            self.link_tuck_into_bed();
        }

        self.set_saved_module_for_menu(7);
        self.set_main_module(7);
        self.set_submodule(15);
        self.Dungeon_LoadSongBankIfNeeded();
        self.module_pre_dungeon_set_ambient_sfx();
    }

    pub(super) fn link_check_bunny_status(&mut self) {
        if self.game_state.player.follower_link.handler_state() == 2 {
            let handler_state = if !self.game_state.player.follower_link.is_bunny_mirror() {
                0
            } else if self.game_state.player.follower_link.has_moon_pearl() {
                28
            } else {
                23
            };
            self.follower_link_state_mut()
                .set_handler_state(handler_state);
        }
    }

    pub(super) fn CrystalCutscene_Initialize(&mut self) {
        self.set_color_math_control(0x33);
        self.set_countdown(0);
        self.set_darkening_or_lightening_screen(0);
        self.Palette_AssertTranslucencySwap();
        self.PaletteFilter_Crystal();
        for (i, color) in CRYSTAL_CUTSCENE_INITIALIZE_CRYSTAL_MAIDEN_PAL
            .iter()
            .enumerate()
        {
            self.set_main_color_asset(112 + i, *color);
        }
        self.increment_cgram_update_flag();
        self.CrystalCutscene_SpawnMaiden();
        self.crystal_cutscene_initialize_polyhedral();
    }

    pub(super) fn CrystalCutscene_SpawnMaiden(&mut self) {
        self.sprite_system_mut().fill_live_states(0);
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(0, 0xab, &mut info);
        let j = j as usize;

        let player_x = self.game_state.player.follower_link.x();
        let player_y = self.game_state.player.follower_link.y();
        let maiden_a = self.ancilla_terminate_select_interactives(j as u8);
        let mut maiden = self.sprite_slot_view_mut(j);
        maiden.set_x((player_x & 0xff00) | 0x78);
        maiden.set_y((player_y & 0xff00) | 0x7c);
        maiden.set_direction(1);
        maiden.set_oam_flags(0x0b);
        maiden.set_subtype2(0);
        maiden.set_floor(0);
        maiden.set_a(maiden_a);
        self.follower_link_state_mut().set_item_receipt_method(0);

        if self.game_state.inventory.save_progress.palace_index_x2() == 24 {
            self.sprite_slot_view_mut(j).set_oam_flags(9);
            self.follower_state_mut().set_indicator(1);
        } else {
            self.follower_state_mut().set_indicator(6);
        }
        self.LoadFollowerGraphics();
        self.follower_state_mut().set_indicator(0);

        let floor_x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_sub(self.game_state.player.follower_link.x())
            .wrapping_add(0x79);
        let floor_y =
            0x30u16.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_v_copy2_low() as u16);
        self.dungeon_moving_floor_mut()
            .set_floor_offsets(floor_x, floor_y);
        self.dungeon_room_load_mut()
            .set_header_collision_2_mirror(1);
    }

    pub(super) fn reset_then_cache_room_entry_properties(&mut self) {
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
        self.clear_screen_transition();
        self.set_submodule(0);
        self.dungeon_environment_mut()
            .clear_water_puzzle_state_changed();
        self.dungeon_environment_mut()
            .clear_movable_block_was_pushed();
        self.cache_camera_properties();
    }

    pub(super) fn ResetThenCacheRoomEntryProperties(&mut self) {
        self.reset_then_cache_room_entry_properties();
    }

    pub(super) fn module_pre_dungeon_set_ambient_sfx(&mut self) {
        if self.game_state.inventory.save_progress.progress_indicator() < 2 {
            self.set_ambient_sound_effect(5);
            if (self.game_state.dungeon.stair_movement.current_floor() as i8) >= 0
                && self.game_state.world.location.dungeon_room() != 2
                && self.game_state.world.location.dungeon_room() != 18
            {
                self.set_ambient_sound_effect(3);
            }
        }
    }

    pub(super) fn module07_dungeon(&mut self) {
        self.dungeon_handle_layer_effect();
        self.replay_trace_ram_watch("module07-after-layer-effect");
        self.run_dungeon_submodule();
        self.replay_trace_ram_watch("module07-after-submodule");
        if self.rom_startup_timing() && self.dungeon_landing_wipe_carry_pending {
            return;
        }
        self.complete_module07_dungeon_after_submodule();
    }

    pub(super) fn complete_module07_dungeon_after_submodule(&mut self) {
        let shared_message_timer = self.game_state.messaging.shared_message_timer.value();
        if self.state_recorder.replay_mode
            && std::env::var_os("ZELDA3_SMV_DUNGEON_TIMING_HACKS").is_some()
            && self.game_state.world.location.is_indoors()
            && self.game_state.world.location.dungeon_room() == 0x0104
            && self.game_state.messaging.dialogue_message_index.value() == 0x007b
            && self.game_state.messaging.runtime.text_render_state() == 4
            && self.game_state.messaging.runtime.dialogue_msg_read_pos() == 0x0052
            && self.game_state.player.follower_link.x() == 0x0937
            && self.game_state.player.follower_link.y() == 0x21c0
            && shared_message_timer == 0x014e
            && (self.game_state.player.follower_link.joypad1h_last()
                | self.game_state.player.follower_link.joypad1l_last())
                == 0
        {
            if !self.replay_reopened_lamp_prompt {
                self.main_show_text_message();
                self.replay_reopened_lamp_prompt = true;
            }
        } else if self.game_state.messaging.runtime.dialogue_msg_read_pos() != 0x0052
            || shared_message_timer != 0x014e
            || self.game_state.player.follower_link.x() != 0x0937
        {
            self.replay_reopened_lamp_prompt = false;
        }

        if !self
            .game_state
            .enhanced_features
            .has(FEATURE_MISC_BUG_FIXES_DUNGEON)
            || self.game_state.frame.main_module == 7
        {
            self.dungeon_object_tracking_mut().clear_misc_object_index();
            self.dungeon_push_block_handler();
            self.replay_trace_ram_watch("module07-after-push-blocks");
            if self.game_state.frame.submodule == 0 {
                self.graphics_load_chr_half_slot();
                self.dungeon_handle_camera();
                self.replay_trace_ram_watch("module07-after-camera");
            }
            if self.game_state.frame.submodule == 0 {
                self.dungeon_handle_room_tags();
                self.replay_trace_ram_watch("module07-after-room-tags");
            }
            if self.game_state.frame.submodule == 0 {
                self.dungeon_process_torches_and_doors();
                self.replay_trace_ram_watch("module07-after-torches-doors");
                if self.game_state.dungeon.room_effects.crush_wall_progress() != 0 {
                    self.dungeon_clear_away_exploding_wall();
                    self.replay_trace_ram_watch("module07-after-blast-wall");
                }
                if self.game_state.player.follower_link.doorway_state() == 0 {
                    self.Dungeon_TryScreenEdgeTransition();
                    self.replay_trace_ram_watch("module07-after-screen-edge");
                }
            }
        }

        self.orient_lamp_light_cone();
        self.replay_trace_ram_watch("module07-after-lamp");

        let bg2x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg2y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        let bg1x = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
        let bg1y = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();

        let bg1_x_offset = self.game_state.world.scroll.bg1_x_offset();
        let bg1_y_offset = self.game_state.world.scroll.bg1_y_offset();
        self.set_bg1_bg2_live_and_copy(
            bg2x.wrapping_add(bg1_x_offset),
            bg2y.wrapping_add(bg1_y_offset),
            bg1x.wrapping_add(bg1_x_offset),
            bg1y.wrapping_add(bg1_y_offset),
        );

        let mut bg1x_restore = bg1x;
        let mut bg1y_restore = bg1y;
        if self
            .game_state
            .dungeon
            .room_load
            .header_collision_2_mirror()
            != 0
        {
            bg1x_restore = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add(self.game_state.dungeon.moving_floor.floor_x_offset());
            bg1y_restore = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_add(self.game_state.dungeon.moving_floor.floor_y_offset());
            self.set_bg1_x(bg1x_restore);
            self.copy_bg1_live_to_ppu_copy();
            self.set_bg1_y(bg1y_restore);
            self.copy_bg1_live_to_ppu_copy();
        }

        self.sprite_dungeon_draw_all_push_blocks();
        self.replay_trace_ram_watch("module07-after-draw-push-blocks");
        self.sprite_main();
        self.replay_trace_ram_watch("module07-after-sprite-main");

        self.set_bg2_x(bg2x);
        self.set_bg2_y(bg2y);
        self.set_bg1_x(bg1x_restore);
        self.set_bg1_y(bg1y_restore);

        self.link_oam_main();
        self.replay_trace_ram_watch("module07-after-link-oam");
        self.hud_refill_logic();
        self.replay_trace_ram_watch("module07-after-refill");
        self.hud_floor_indicator();
        self.replay_trace_ram_watch("module07-after-floor-indicator");
    }

    pub(super) fn module07_00_player_control(&mut self) {
        if (self
            .game_state
            .world
            .transient
            .flag_custom_spell_anim_active()
            | self.game_state.player.follower_link.immobilized_flag()
            | self.game_state.player.follower_link.menu_block_flag())
            == 0
        {
            if self.game_state.player.follower_link.filtered_joypad_h() & 0x10 != 0 {
                self.set_overworld_map_state(0);
                self.set_submodule(1);
                self.save_main_module_for_menu();
                self.set_main_module(14);
                return;
            } else if self.did_press_button_for_map() {
                if self.game_state.inventory.save_progress.palace_index_x2() != 0xff
                    && self.game_state.world.location.dungeon_room_index() != 0
                {
                    self.set_overworld_map_state(0);
                    self.set_submodule(3);
                    self.save_main_module_for_menu();
                    self.set_main_module(14);
                    return;
                }
            } else if self.game_state.player.follower_link.joypad1h_last() & 0x20 != 0
                && self.game_state.inventory.save_progress.progress_indicator() != 0
            {
                self.set_overworld_map_state(0);
                self.DisplaySelectMenu();
                return;
            }
            self.replay_trace_ram_watch("module07-before-hud-switch");
            self.hud_handle_item_switch_inputs();
            self.replay_trace_ram_watch("module07-after-hud-switch");
        }
        self.replay_trace_ram_watch("module07-before-link-main");
        self.link_main();
        self.replay_trace_ram_watch("module07-after-link-main");
    }

    pub(super) fn dungeon_handle_layer_effect(&mut self) {
        self.Dungeon_HandleLayerEffect();
    }

    pub(super) fn Dungeon_HandleLayerEffect(&mut self) {
        match self.game_state.dungeon.room_load.header_collision_2() {
            0 | 1 => self.LayerEffect_Nothing(),
            2 => self.LayerEffect_Scroll(),
            3 => self.LayerEffect_WaterRapids(),
            4 => self.LayerEffect_Trinexx(),
            5 => self.LayerEffect_Agahnim2(),
            6 => self.LayerEffect_InvisibleFloor(),
            7 => self.LayerEffect_Ganon(),
            _ => panic!("invalid dungeon layer effect index"),
        }
    }

    pub(super) fn LayerEffect_Nothing(&mut self) {}

    pub(super) fn LayerEffect_Scroll(&mut self) {
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 != 0 {
            self.dungeon_room_load_mut().clear_header_collision_2();
            return;
        }
        self.dungeon_moving_floor_mut().set_floor_x_velocity(0);
        self.dungeon_moving_floor_mut().set_floor_y_velocity(0);
        let flags = self.game_state.dungeon.moving_floor.floor_move_flags();
        if flags & 1 != 0 {
            return;
        }
        let subpixel = self.bg1_move_calc_mut().advance_x_subpixel(0x80);
        let mut t = (subpixel >> 8) as i16;
        if flags & 2 != 0 {
            t = -t;
        }
        if flags < 4 {
            self.dungeon_moving_floor_mut()
                .set_floor_x_velocity(t as u16);
            let x_offs = self
                .game_state
                .dungeon
                .moving_floor
                .floor_x_offset()
                .wrapping_sub(t as u16);
            self.dungeon_moving_floor_mut().set_floor_x_offset(x_offs);
            let bg1 = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add(x_offs);
            self.set_bg1_x(bg1);
        } else {
            self.dungeon_moving_floor_mut()
                .set_floor_y_velocity(t as u16);
            let y_offs = self
                .game_state
                .dungeon
                .moving_floor
                .floor_y_offset()
                .wrapping_sub(t as u16);
            self.dungeon_moving_floor_mut().set_floor_y_offset(y_offs);
            let bg1 = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_add(y_offs);
            self.set_bg1_y(bg1);
        }
    }

    pub(super) fn LayerEffect_Trinexx(&mut self) {
        let x = self
            .game_state
            .dungeon
            .moving_floor
            .floor_x_offset()
            .wrapping_add(self.game_state.dungeon.moving_floor.floor_x_velocity());
        let y = self
            .game_state
            .dungeon
            .moving_floor
            .floor_y_offset()
            .wrapping_add(self.game_state.dungeon.moving_floor.floor_y_velocity());
        self.dungeon_moving_floor_mut().set_floor_x_offset(x);
        self.dungeon_moving_floor_mut().set_floor_y_offset(y);
        self.dungeon_moving_floor_mut().set_floor_x_velocity(0);
        self.dungeon_moving_floor_mut().set_floor_y_velocity(0);
    }

    pub(super) fn LayerEffect_Agahnim2(&mut self) {
        let j = self.game_state.frame.frame_counter & 0x7f;
        if j == 3 || j == 36 {
            self.set_main_color_constant(0x6d, 0x1d59);
            self.set_main_color_constant(0x6e, 0x25ff);
            self.set_main_color_constant(0x6f, 0x001a);
            self.set_main_color_constant(0x77, 0x001a);
            self.increment_cgram_update_flag();
        } else if j == 5 || j == 38 {
            self.copy_color(
                (zelda3_palette::Bank::Aux, 0x6d),
                (zelda3_palette::Bank::Main, 0x6d),
            );
            self.copy_color(
                (zelda3_palette::Bank::Aux, 0x6e),
                (zelda3_palette::Bank::Main, 0x6e),
            );
            self.copy_color(
                (zelda3_palette::Bank::Aux, 0x6f),
                (zelda3_palette::Bank::Main, 0x6f),
            );
            self.copy_color(
                (zelda3_palette::Bank::Aux, 0x6f),
                (zelda3_palette::Bank::Main, 0x77),
            );
            self.increment_cgram_update_flag();
        }
        self.set_sub_screen_layers(2);
    }

    pub(super) fn LayerEffect_InvisibleFloor(&mut self) {
        let mut count = 0;
        for i in 0..16 {
            if self
                .game_state
                .dungeon
                .object_tracking
                .object_tilemap_pos(i)
                & 0x8000
                != 0
            {
                count += 1;
            }
        }
        let (x, y) = if count == 0 { (0, 0) } else { (0x2940, 0x4e60) };
        if self.game_state.display.palette_buffer.aux_color(0x7b) != x {
            self.set_main_color_constant(0x7b, x);
            self.set_aux_color_constant(0x7b, x);
            self.set_main_color_constant(0x7c, y);
            self.set_aux_color_constant(0x7c, y);
            self.increment_cgram_update_flag();
        }
        self.set_sub_screen_layers(2);
    }

    pub(super) fn LayerEffect_Ganon(&mut self) {
        let mut count = 0u8;
        for i in 0..16 {
            if self
                .game_state
                .dungeon
                .object_tracking
                .object_tilemap_pos(i)
                & 0x8000
                != 0
            {
                count = count.wrapping_add(1);
            }
        }
        self.dungeon_torch_mut().set_ganon_torch_count(count);
        if count == 0 {
            self.set_sub_screen_layers(0);
            self.set_color_math_control(0xb3);
        } else if count == 1 {
            self.set_sub_screen_layers(2);
            self.set_color_math_control(0x70);
        } else {
            self.set_sub_screen_layers(0);
            self.set_color_math_control(0x70);
        }
    }

    pub(super) fn LayerEffect_WaterRapids(&mut self) {
        let t = self.bg1_move_calc_mut().advance_x_subpixel(0x80);
        self.dungeon_moving_floor_mut()
            .set_floor_x_velocity((-(t as i16 >> 8)) as u16);
    }

    pub(super) fn Module07_15_01_ApplyMosaicAndFilter(&mut self) {
        self.conditional_mosaic_control();
        self.set_mosaic_copy_from_level_or(3);
        self.apply_palette_filter_bounce();
    }

    pub(super) fn Module07_15_04_SyncRoomPropsAndBuildOverlay(&mut self) {
        self.ApplyGrayscaleFixed_Incremental();
        if self.game_state.world.location.dungeon_room() == 0x17 {
            self.dungeon_stair_movement_mut().set_current_floor(4);
        }
        self.MirrorBg1Bg2Offs();
        self.Dungeon_AdjustForRoomLayout();
        let mut ts = SPIRAL_SUBSCREEN_LAYER_BY_BG2
            [self.game_state.dungeon.room_load.bg2_properties() as usize]
            as u8;
        let mut tm = 0x16;
        if ts & 0x80 != 0 {
            tm = 0x17;
            ts = 0;
        }
        self.set_main_screen_layers(tm);
        self.set_sub_screen_layers(ts);
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.increment_subsubmodule();
    }

    pub(super) fn Module07_15_0E_FadeInFromWarp(&mut self) {
        if self.game_state.display.palette_filter.countdown() & 1 != 0
            && self.game_state.display.mosaic_level != 0
        {
            self.decrement_mosaic_level_by(0x10);
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(3);
        self.ApplyPaletteFilter_bounce();
    }

    pub(super) fn Module07_15_0F_FinalizeAndCacheEntry(&mut self) {
        if self.overworld_map_state() == 5 {
            self.SetAndSaveVisitedQuadrantFlags();
            self.set_submodule(0);
            self.ResetThenCacheRoomEntryProperties();
        }
    }

    pub(super) fn dungeon_push_block_handler(&mut self) {
        while self.game_state.dungeon.object_tracking.misc_object_index()
            != self.game_state.dungeon.torch.torches_start_index()
        {
            let obj = self.game_state.dungeon.object_tracking.misc_object_index();
            let k = usize::from(obj >> 1);
            match self
                .game_state
                .dungeon
                .object_tracking
                .replacement_tile_state(k)
            {
                1 => {
                    self.RoomDraw_16x16Single(obj as u8);
                    let dir = self.game_state.player.pushed_block.push_direction_index();
                    let pos = self
                        .game_state
                        .dungeon
                        .object_tracking
                        .object_tilemap_pos(k)
                        .wrapping_add_signed(
                            DUNGEON_PUSH_BLOCK_HANDLER_PUSH_BLOCK_MOVE_DISTANCES[dir],
                        );
                    self.dungeon_object_tracking_mut()
                        .set_object_tilemap_pos(k, pos);
                    self.dungeon_object_tracking_mut()
                        .set_replacement_tile_state(k, 2);
                }
                2 => {
                    self.PushBlock_Slide(obj as u8);
                    let obj = self.game_state.dungeon.object_tracking.misc_object_index();
                    let k = usize::from(obj >> 1);
                    if self
                        .game_state
                        .dungeon
                        .object_tracking
                        .replacement_tile_state(k)
                        == 3
                    {
                        self.PushBlock_CheckForPit(obj as u8);
                        let state = self
                            .game_state
                            .dungeon
                            .object_tracking
                            .replacement_tile_state(k)
                            .wrapping_add(1);
                        self.dungeon_object_tracking_mut()
                            .set_replacement_tile_state(k, state);
                    }
                }
                4 => {
                    self.PushBlock_HandleFalling(obj as u8);
                }
                _ => {}
            }
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

    pub(super) fn dungeon_handle_camera(&mut self) {
        let link_y_vel = self.game_state.player.follower_link.y_velocity();
        if link_y_vel != 0 {
            let z = if self.game_state.world.transient.allow_scroll_z() != 0
                && self.game_state.player.follower_link.z() != 0xffff
            {
                self.game_state.player.follower_link.z()
            } else {
                0
            };
            let y = self.game_state.player.follower_link.y().wrapping_sub(z) & 0x01ff;
            let y = y.wrapping_add(12);
            let moving_up = (link_y_vel as i8).is_negative();
            let scrollamt: i16 = if moving_up { -1 } else { 1 };
            let steps = if moving_up {
                (link_y_vel as i8).wrapping_neg() as u8
            } else {
                link_y_vel
            };

            for _ in 0..steps {
                let mut qm = self
                    .game_state
                    .world
                    .transient
                    .vertical_room_bounds_base_index();
                if moving_up {
                    if y > self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_y_coord_scroll_low()
                    {
                        continue;
                    }
                } else {
                    if y < self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_y_coord_scroll_hi()
                    {
                        continue;
                    }
                    qm += 2;
                }

                if self.game_state.display.ppu_scroll_copy.bg2_v_copy2()
                    == self.game_state.world.room_bounds.y_bound(qm)
                {
                    continue;
                }

                let bg2 = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg2_v_copy2()
                    .wrapping_add(scrollamt as u16);
                self.set_bg2_y(bg2);
                if self.game_state.world.location.dungeon_room() == 0xffff {
                    continue;
                }

                let subpixel = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg1_v_subpixel()
                    .wrapping_add(0x8000);
                self.set_bg1_v_subpixel(subpixel);
                let bg1_delta = (scrollamt >> 1) + i16::from(subpixel & 0x8000 == 0);
                let bg1 = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg1_v_copy2()
                    .wrapping_add(bg1_delta as u16);
                self.set_bg1_y(bg1);
                let camera_low = self
                    .game_state
                    .world
                    .camera_boundaries
                    .camera_y_coord_scroll_low()
                    .wrapping_add(scrollamt as u16);
                self.set_camera_y_coord_scroll_low(camera_low);
                self.set_camera_y_coord_scroll_hi(camera_low.wrapping_add(2));
            }
        }

        let link_x_vel = self.game_state.player.follower_link.x_velocity();
        if link_x_vel != 0 {
            let x = (self.game_state.player.follower_link.x() & 0x01ff).wrapping_add(8);
            let moving_left = (link_x_vel as i8).is_negative();
            let scrollamt: i16 = if moving_left { -1 } else { 1 };
            let steps = if moving_left {
                (link_x_vel as i8).wrapping_neg() as u8
            } else {
                link_x_vel
            };

            for _ in 0..steps {
                let mut qm = self
                    .game_state
                    .world
                    .transient
                    .horizontal_room_bounds_base_index();
                if moving_left {
                    if x > self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_x_coord_scroll_low()
                    {
                        continue;
                    }
                } else {
                    if x < self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_x_coord_scroll_hi()
                    {
                        continue;
                    }
                    qm += 2;
                }

                if self.game_state.display.ppu_scroll_copy.bg2_h_copy2()
                    == self.game_state.world.room_bounds.x_bound(qm)
                {
                    continue;
                }

                let bg2 = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg2_h_copy2()
                    .wrapping_add(scrollamt as u16);
                self.set_bg2_x(bg2);
                if self.game_state.world.location.dungeon_room() == 0xffff {
                    continue;
                }

                let subpixel = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg1_h_subpixel()
                    .wrapping_add(0x8000);
                self.set_bg1_h_subpixel(subpixel);
                let bg1_delta = (scrollamt >> 1) + i16::from(subpixel & 0x8000 == 0);
                let bg1 = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg1_h_copy2()
                    .wrapping_add(bg1_delta as u16);
                self.set_bg1_x(bg1);
                let camera_low = self
                    .game_state
                    .world
                    .camera_boundaries
                    .camera_x_coord_scroll_low()
                    .wrapping_add(scrollamt as u16);
                self.set_camera_x_coord_scroll_low(camera_low);
                self.set_camera_x_coord_scroll_hi(camera_low.wrapping_add(2));
            }
        }

        if self.game_state.world.location.dungeon_room() != 0xffff {
            let bg2_properties = self.game_state.dungeon.room_load.bg2_properties();
            if bg2_properties == 0
                || bg2_properties == 2
                || bg2_properties == 3
                || bg2_properties == 4
                || bg2_properties >= 6
            {
                self.copy_bg2_live_to_bg1_live();
            }
        }
    }

    pub(super) fn dungeon_handle_room_tags(&mut self) {
        if self.game_state.dungeon.room_runtime.should_run_room_tags() {
            self.Dungeon_DetectStaircase();

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
            self.tile_detect_position_mut()
                .set_collision_bits_low_byte(0);
            self.dungeon_run_tag_routine(0);
            self.tile_detect_position_mut()
                .set_collision_bits_low_byte(1);
            self.dungeon_run_tag_routine(1);
        }
        self.dungeon_room_runtime_mut().clear_room_tag_skip();
    }

    fn dungeon_run_tag_routine(&mut self, k: usize) {
        match self.game_state.dungeon.header.header_tag(k) {
            0x00 => self.Dung_TagRoutine_0x00(k),
            0x01 | 0x0b | 0x29 => self.RoomTag_NorthWestTrigger(k),
            0x02 | 0x0c | 0x2a => self.Dung_TagRoutine_0x2A(k),
            0x03 | 0x0d | 0x2b => self.Dung_TagRoutine_0x2B(k),
            0x04 | 0x0e | 0x2c => self.Dung_TagRoutine_0x2C(k),
            0x05 | 0x0f | 0x2d => self.Dung_TagRoutine_0x2D(k),
            0x06 | 0x10 | 0x2e => self.Dung_TagRoutine_0x2E(k),
            0x07 | 0x11 | 0x2f => self.Dung_TagRoutine_0x2F(k),
            0x08 | 0x12 | 0x30 => self.Dung_TagRoutine_0x30(k),
            0x09 | 0x13 | 0x31 => self.RoomTag_QuadrantTrigger(k),
            0x0a | 0x32 => self.RoomTag_RoomTrigger(k),
            0x14 => self.RoomTag_RoomTrigger_BlockDoor(k),
            0x15 => self.RoomTag_PrizeTriggerDoorDoor(k),
            0x16 => self.RoomTag_SwitchTrigger_HoldDoor(k),
            0x17 => self.RoomTag_SwitchTrigger_ToggleDoor(k),
            0x18 => self.RoomTag_WaterOff(k),
            0x19 => self.RoomTag_WaterOn(k),
            0x1a => self.RoomTag_WaterGate(k),
            0x1b => self.Dung_TagRoutine_0x1B(k),
            0x1c => self.RoomTag_MovingWall_East(k),
            0x1d => self.RoomTag_MovingWall_West(k),
            0x1e | 0x1f => self.RoomTag_MovingWallTorchesCheck(k),
            0x20 => self.RoomTag_Switch_ExplodingWall(k),
            0x21 => self.RoomTag_Holes0(k),
            0x22 => self.RoomTag_ChestHoles0(k),
            0x23 => self.Dung_TagRoutine_0x23(k),
            0x24 => self.RoomTag_Holes2(k),
            0x25 => self.RoomTag_GetHeartForPrize(k),
            0x26 => self.RoomTag_KillRoomBlock(k),
            0x27 => self.RoomTag_TriggerChest(k),
            0x28 => self.RoomTag_PullSwitchExplodingWall(k),
            0x33 => self.RoomTag_TorchPuzzleDoor(k),
            0x34 => self.Dung_TagRoutine_0x34(k),
            0x35 => self.Dung_TagRoutine_0x35(k),
            0x36 => self.Dung_TagRoutine_0x36(k),
            0x37 => self.Dung_TagRoutine_0x37(k),
            0x38 => self.RoomTag_Agahnim(k),
            0x39 => self.Dung_TagRoutine_0x39(k),
            0x3a => self.Dung_TagRoutine_0x3A(k),
            0x3b => self.Dung_TagRoutine_0x3B(k),
            0x3c => self.RoomTag_PushBlockForChest(k),
            0x3d => self.RoomTag_GanonDoor(k),
            0x3e => self.RoomTag_TorchPuzzleChest(k),
            0x3f => self.RoomTag_RekillableBoss(k),
            _ => {}
        }
    }

    pub(super) fn dungeon_process_torches_and_doors(&mut self) {
        if self.game_state.frame.frame_counter & 3 == 0
            && self
                .game_state
                .world
                .transient
                .flag_custom_spell_anim_active()
                == 0
        {
            for i in 0..16 {
                let timer = self.game_state.dungeon.torch.timer(i);
                if timer != 0 {
                    let next = timer.wrapping_sub(1);
                    self.dungeon_torch_mut().set_timer(i, next);
                    if next == 0 {
                        self.dungeon_torch_mut().set_attr(0xc0 + i as u8);
                        self.Dungeon_ExtinguishTorch();
                    }
                }
            }
        }

        if !self.game_state.player.follower_link.is_immobilized() {
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

            let mut openable =
                (self.game_state.dungeon.bg2_attributes.bg2_attr(pos) & 0xf0) == 0xf0;
            if !openable {
                pos += DUNGEON_PROCESS_TORCHES_AND_DOORS_LINK_POSITION_OFFSETS[dir];
                openable = (self.game_state.dungeon.bg2_attributes.bg2_attr(pos) & 0xf0) == 0xf0;
            }

            if openable {
                let k = (self.game_state.dungeon.bg2_attributes.bg2_attr(pos) & 0x0f) as usize;
                self.dungeon_room_load_mut().set_selected_key_door(k);

                if (self.game_state.dungeon.doors.door_direction(k) & 3) == dir as u8 {
                    let door_type = self.game_state.dungeon.doors.door_type_and_slot(k) & 0xfe;
                    if door_type == DOOR_TYPE_BREAKABLE_WALL {
                        if self.game_state.player.follower_link.is_running()
                            && self.game_state.player.follower_link.dash_counter() < 63
                        {
                            self.dungeon_doors_mut().set_current_door_pos(pos as u16);

                            let db = self.ancilla_add_door_debris();
                            if db >= 0 {
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
                            }
                            self.set_sound_effect_2(27);
                            self.set_submodule(9);
                            self.sprite_repel_dash();
                            return;
                        }
                    } else if door_type == DOOR_TYPE_1E {
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
                            self.dungeon_doors_mut().clear_door_animation_step();
                            self.dungeon_doors_mut().set_current_door_pos(pos as u16);
                            self.set_submodule(4);
                            let sound_effect_2 = 20
                                | DUNGEON_PROCESS_TORCHES_AND_DOORS_OPEN_DOOR_PANNING[(self
                                    .game_state
                                    .dungeon
                                    .doors
                                    .door_direction(k)
                                    & 3)
                                    as usize];
                            self.set_sound_effect_2(sound_effect_2);
                            return;
                        }
                        if self
                            .game_state
                            .world
                            .transient
                            .big_key_door_message_triggered()
                            == 0
                        {
                            self.set_big_key_door_message_triggered(1);
                            self.dialogue_message_index_mut().set_value(0x007a);
                            self.main_show_text_message();
                        }
                    } else if door_type >= DOOR_TYPE_SMALL_KEY_DOOR
                        && door_type < 0x2c
                        && door_type != 0x2a
                        && self.game_state.inventory.player_resources.keys() != 0
                    {
                        self.player_resources_mut().decrement_keys();
                        self.dungeon_doors_mut().clear_door_animation_step();
                        self.dungeon_doors_mut().set_current_door_pos(pos as u16);
                        self.set_submodule(4);
                        let sound_effect_2 = 20
                            | DUNGEON_PROCESS_TORCHES_AND_DOORS_OPEN_DOOR_PANNING
                                [(self.game_state.dungeon.doors.door_direction(k) & 3) as usize];
                        self.set_sound_effect_2(sound_effect_2);
                        return;
                    }
                } else {
                    self.set_big_key_door_message_triggered(0);
                }
            } else {
                self.set_big_key_door_message_triggered(0);
            }
        }

        let invisible = self.game_state.dungeon.door_setup.invisible_door_marker();
        if invisible & 0x0080 == 0
            && self.game_state.player.follower_link.doorway_state() == 0
            && (self.game_state.player.follower_link.x() >> 8) == 0x000c
        {
            let dir = invisible as u8;
            let j = ((invisible >> 8) >> 1) as usize;
            let mut opened = self
                .game_state
                .dungeon
                .doors
                .opened_doors_including_adjacent();
            if dir != self.game_state.player.follower_link.facing()
                && (dir ^ 2) == self.game_state.player.follower_link.facing()
            {
                opened |= upper_bitmask(j);
            } else {
                opened &= !upper_bitmask(j);
            }
            if opened
                != self
                    .game_state
                    .dungeon
                    .doors
                    .opened_doors_including_adjacent()
            {
                self.dungeon_doors_mut()
                    .set_opened_doors_including_adjacent(opened);
                self.DrawEyeWatchDoor(j);
                let addr = self.game_state.dungeon.doors.door_tilemap_address(j);
                self.dungeon_prep_overlay_dma_next_prep(0, addr);
                self.Dungeon_LoadToggleDoorAttr_OtherEntry(j as i32);
                self.request_nmi_copy_packets();
                self.set_sound_effect_2(21);
                return;
            }
        }

        if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0
            || self.game_state.player.follower_link.button_b_frames() != 4
        {
            return;
        }

        let link_y =
            self.game_state.player.follower_link.y().wrapping_add(
                self.game_state.player.follower_link.oam_y_offset_signed() as i16 as u16,
            );
        let link_x =
            self.game_state.player.follower_link.x().wrapping_add(
                self.game_state.player.follower_link.oam_x_offset_signed() as i16 as u16,
            );
        let mut pos = (((link_y & 0x01f8) << 3) | ((link_x & 0x01f8) >> 3)) as usize;
        let mut attr = self.game_state.dungeon.bg2_attributes.bg2_attr(pos) & 0xfc;
        let mut y = 0x41u8;

        if attr != 0x6c && (attr & 0xf0) != 0xf0 {
            pos += 1;
            attr = self.game_state.dungeon.bg2_attributes.bg2_attr(pos) & 0xfc;
            y = 0x40;
            if attr != 0x6c && (attr & 0xf0) != 0xf0 {
                pos += 63;
                attr = self.game_state.dungeon.bg2_attributes.bg2_attr(pos) & 0xfc;
                y = 1;
                if attr != 0x6c && (attr & 0xf0) != 0xf0 {
                    pos += 1;
                    attr = self.game_state.dungeon.bg2_attributes.bg2_attr(pos) & 0xfc;
                    y = 0;
                    if attr != 0x6c && (attr & 0xf0) != 0xf0 {
                        return;
                    }
                }
            }
        }

        let addr;
        if attr == 0x6c {
            if y & 0x40 != 0 {
                pos -= 64;
                if self.game_state.dungeon.bg2_attributes.bg2_attr(pos) & 0xfc != 0x6c {
                    pos += 64;
                }
            }
            if y & 1 != 0 {
                pos -= 1;
                if self.game_state.dungeon.bg2_attributes.bg2_attr(pos) & 0xfc != 0x6c {
                    pos += 1;
                }
            }
            attr = self.game_state.dungeon.bg2_attributes.bg2_attr(pos);
            self.write_attr2(pos + xy(0, 0), 0x0202);
            self.write_attr2(pos + xy(0, 1), 0x0202);
            addr = ((pos - xy(1, 1)) * 2) as u16;
            self.RoomDraw_Object_Nx4_Bg2(
                4,
                DUNGEON_PROCESS_TORCHES_AND_DOORS_SOURCE_TILES1[(attr & 3) as usize] as usize,
                addr >> 1,
            );
        } else {
            self.dungeon_doors_mut().set_current_door_pos(pos as u16);
            let k = (attr & 0x0f) as usize;
            if self.game_state.dungeon.doors.door_type_and_slot(k) != DOOR_TYPE_SLASHABLE {
                return;
            }
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
            self.RoomDraw_Object_Nx4_Bg2(4, DOOR_TYPE_SRC_UP[0x56 / 2] as usize, addr >> 1);
            self.Dungeon_LoadToggleDoorAttr_OtherEntry(k as i32);
        }

        self.dungeon_prep_overlay_dma_next_prep(0, addr);
        let sound_effect_1 = 30 | self.calculate_sfx_pan_arbitrary(((addr & 0x007f) * 2) as u8);
        self.set_sound_effect_1(sound_effect_1);
        self.request_nmi_copy_packets();
    }

    pub(super) fn dungeon_clear_away_exploding_wall(&mut self) {
        self.follower_link_state_mut().set_immobilized_flag(6);
        self.set_modal_pause_flag(6);
        if self
            .game_state
            .dungeon
            .room_effects
            .blast_wall_message_state()
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

        self.Door_BlastWallExploding_Draw(dsto);
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
            self.Door_LoadBlastWallAttr(door);
            self.dungeon_room_effects_mut().set_crush_wall_progress(0);
            self.dungeon_room_effects_mut()
                .clear_blast_wall_door_index();
            self.Dungeon_FlagRoomData_Quadrants();
            self.follower_link_state_mut().clear_immobilized();
            self.clear_modal_pause_flag();
        }
        self.set_nmi_copy_packets_request(3);
    }

    pub(super) fn orient_lamp_light_cone(&mut self) {
        self.OrientLampLightCone();
    }

    pub(super) fn sprite_dungeon_draw_all_push_blocks(&mut self) {
        for i in (0..=1).rev() {
            if self
                .game_state
                .dungeon
                .object_tracking
                .changeable_object_index(i)
                != 0
            {
                self.Sprite_HandlePushedBlocks_One(i);
            }
        }
    }

    pub(super) fn reset_transition_props_and_advance_reset_interface(&mut self) {
        self.ResetTransitionPropsAndAdvance_ResetInterface();
    }

    pub(super) fn reset_transition_props_and_advance_submodule(&mut self) {
        self.ResetTransitionPropsAndAdvanceSubmodule();
    }
}

#[cfg(test)]
#[path = "dungeon_tests.rs"]
mod tests;
