// Methods ported from zelda3/src/overworld.c and included inside ZeldaState.

use super::*;
use crate::game_state::constants::DUNG_BG2;
use crate::types::{sign16, Point16U};

mod overworld_shared;
use overworld_shared::*;

const TURTLE_ROCK_ENTRANCE_DRAW_POSITIONS: [u16; 16] = [
    0x099e, 0x09a0, 0x09a2, 0x09a4, 0x0a1e, 0x0a20, 0x0a22, 0x0a24, 0x0a9e, 0x0aa0, 0x0aa2, 0x0aa4,
    0x0b1e, 0x0b20, 0x0b22, 0x0b24,
];
const MIRE_ENTRANCE_BODY_POSITIONS: [u16; 12] = [
    0x0622, 0x0624, 0x0626, 0x0628, 0x06a2, 0x06a4, 0x06a6, 0x06a8, 0x0722, 0x0724, 0x0726, 0x0728,
];
const MIRE_ENTRANCE_MASK_BITS: [u8; 26] = [
    0xff, 0xf7, 0xf7, 0xfb, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
    0xaa, 0x88, 0x88, 0x88, 0x88, 0x80, 0x80, 0x80, 0x80, 0x80,
];
const OVERWORLD_RAIN_X_OFFSETS: [u8; 4] = [1, 0, 1, 0];
const OVERWORLD_RAIN_Y_OFFSETS: [u8; 4] = [0, 17, 0, 17];
const OVERWORLD_SCREEN_TRANSITION_DIRECTION_BITS: [u8; 4] = [8, 4, 2, 1];
const OVERWORLD_SECRET_TILE_BY_TYPE: [u16; 4] = [0x0dcc, 0x0212, 0xffff, 0x0db4];
const LW_TURTLE_ROCK_PEG_POSITIONS: [u16; 3] = [0x0826, 0x05a0, 0x081a];
const PACKED_ENEMY_DAMAGE_SOURCE_BYTES: usize = 0x800;
const OVERWORLD_DOOR_ANIM_TILES: [u16; 56] = [
    0x0da8, 0x0da9, 0x0daa, 0x0dab, 0x0dac, 0x0dad, 0x0dae, 0x0daf, 0x0db0, 0x0db1, 0x0db2, 0x0db3,
    0x0db6, 0x0db7, 0x0db8, 0x0db9, 0x0dba, 0x0dbb, 0x0dbc, 0x0dbd, 0x0dcd, 0x0dce, 0x0dcf, 0x0dd0,
    0x0dd3, 0x0dd4, 0x0dd5, 0x0dd6, 0x0dd7, 0x0dd8, 0x0dd9, 0x0dda, 0x0dd1, 0x0dd2, 0x0dd3, 0x0dd4,
    0x0dd1, 0x0dd2, 0x0dd7, 0x0dd8, 0x0918, 0x0919, 0x091a, 0x091b, 0x0ddb, 0x0ddc, 0x0ddd, 0x0dde,
    0x0dd1, 0x0dd2, 0x0ddb, 0x0ddc, 0x0e21, 0x0e22, 0x0e23, 0x0e24,
];

impl ZeldaState {
    pub fn parity_probe_overworld_screen(&mut self, screen: u16) -> u16 {
        self.set_indoor_flag(0);
        self.set_overworld_area_index_word(screen);
        self.set_overworld_screen_word(screen);
        self.Overworld_LoadNewScreenProperties();
        u16::from(self.game_state.world.location.overworld_screen_index())
    }

    pub fn parity_probe_overworld_screen_and_build_map(&mut self, screen: u16) -> u16 {
        self.set_indoor_flag(0);
        self.set_overworld_area_index_word(screen);
        self.set_overworld_screen_word(screen);
        self.Overworld_LoadNewScreenProperties();
        let sc = self.game_state.world.location.overworld_screen_index();
        self.DecompressAnimatedOverworldTiles(0x5e);
        self.InitializeTilesets();
        self.OverworldLoadScreensPaletteSet();
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(sc),
            self.overworld_config_table().sprite_palette(sc as usize),
        );
        self.Palette_SetOwBgColor();
        self.Overworld_LoadPalettesInner();
        for i in 0..0x100usize {
            self.ppu.cgram[i] = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + i * 2);
        }
        self.commit_palette_provenance_cgram();
        self.Overworld_LoadAndBuildScreen();
        u16::from(self.game_state.world.location.overworld_screen_index())
    }

    pub fn parity_probe_overworld_bg2_map8_entry(&self, tile_index: usize) -> u16 {
        self.game_state
            .dungeon
            .bg2_attributes
            .bg2_attr_word(tile_index * 2)
    }

    fn replay_trace_door_overlay(&self, label: &str, pos: u16) {
        if std::env::var_os("ZELDA3_REPLAY_TRACE_DOOR").is_none() {
            return;
        }
        let world_location = &self.game_state.world.location;
        let screen = world_location.overworld_screen();
        let screen_byte = world_location.overworld_screen_index();
        if screen_byte != 0x5b && pos != 0x0e2e {
            return;
        }
        let word0 = if pos < 0x2000 {
            self.game_state
                .dungeon
                .room_tilemaps
                .bg2_tile_by_byte_pos(pos)
        } else {
            0xffff
        };
        let word1 = if pos < 0x1ffe {
            self.game_state
                .dungeon
                .room_tilemaps
                .bg2_tile(((pos >> 1) + 1) as usize)
        } else {
            0xffff
        };
        eprintln!(
            "door-trace frame={} {label} main={} sub={} subsub={} screen=0x{screen:04x} screenb=0x{screen_byte:02x} event=0x{:02x} owent=0x{:04x} big=0x{:04x} pos=0x{pos:04x} bg2=0x{word0:04x}/0x{word1:04x}",
            self.game_state.frame.frame_counter,
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
            self.game_state.frame.subsubmodule,
            self.game_state.world.overworld.event_info.event_info(screen_byte as usize),
            self.game_state.world.region.ow_entrance_value(),
            self.game_state.dungeon.object_tracking.big_rock_starting_address(),
        );
    }

    pub(super) fn Module08_OverworldLoad(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.PreOverworld_LoadProperties(),
            1 => self.PreOverworld_LoadOverlays(),
            2 => self.Module08_02_LoadAndAdvance(),
            submodule => panic!("Module08_OverworldLoad invalid submodule_index: {submodule}"),
        }
    }

    pub(super) fn PreOverworld_LoadProperties(&mut self) {
        self.set_color_window_selection(0x82);
        self.dungeon_environment_mut()
            .clear_somaria_block_bg_check_flag();
        self.AdjustLinkBunnyStatus();
        if self.game_state.frame.main_module == 8 {
            self.LoadOverworldFromDungeon();
        } else {
            self.LoadOverworldFromSpecialOverworld();
        }
        self.Overworld_SetSongList();
        self.player_resources_mut().set_keys(0xff);
        self.hud_refill_logic();

        let world_location = &self.game_state.world.location;
        let sc = world_location.overworld_screen_index();
        let dr = world_location.dungeon_room_index();
        let (xt, ow_anim_tiles) = pre_overworld_music_selection(
            sc,
            dr,
            self.game_state.system_signals.queued_music_control(),
            self.game_state.inventory.save_progress.progress_indicator(),
            self.game_state
                .world
                .transient
                .savegame_has_master_sword_flags(),
            self.game_state.inventory.save_progress.dark_world_state(),
            self.game_state.inventory.items.moon_pearl(),
        );

        self.set_queued_music_control(xt);
        if self.begin_pre_overworld_properties_work(sc, ow_anim_tiles) {
            return;
        }
        self.complete_pre_overworld_load_properties(sc, ow_anim_tiles);
    }

    pub(super) fn complete_pre_overworld_load_properties(
        &mut self,
        overworld_screen: u8,
        animated_tiles: u8,
    ) {
        self.complete_pre_overworld_load_properties_through_sprite_reset(
            overworld_screen,
            animated_tiles,
        );
        self.complete_pre_overworld_load_properties_after_sprite_reset(overworld_screen);
    }

    pub(super) fn complete_pre_overworld_load_properties_through_sprite_reset(
        &mut self,
        overworld_screen: u8,
        animated_tiles: u8,
    ) {
        self.DecompressAnimatedOverworldTiles(animated_tiles);
        self.InitializeTilesets();
        self.OverworldLoadScreensPaletteSet();
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(overworld_screen),
            self.overworld_config_table()
                .sprite_palette(overworld_screen as usize),
        );
        self.Palette_SetOwBgColor();
        if self.game_state.frame.main_module == 8 {
            self.Overworld_LoadPalettesInner();
        } else {
            self.SpecialOverworld_CopyPalettesToCache();
        }
        self.Overworld_SetFixedColAndScroll();
        self.set_overworld_fixed_color_adjustment(0);
        self.follower_initialize();

        if overworld_screen & 0x3f == 0 {
            self.DecodeAnimatedSpriteTile_variable(0x1e);
        }
        self.set_saved_module_for_menu(9);
        self.sprite_begin_reload_all_overworld();
    }

    pub(super) fn complete_pre_overworld_load_properties_after_sprite_reset(
        &mut self,
        overworld_screen: u8,
    ) {
        self.complete_pre_overworld_load_properties_after_sprite_reset_with_presence(
            overworld_screen,
            false,
        );
    }

    pub(super) fn complete_pre_overworld_load_properties_after_sprite_reset_with_presence(
        &mut self,
        overworld_screen: u8,
        sprite_presence_published: bool,
    ) {
        if sprite_presence_published {
            self.sprite_activate_all_proxima();
        } else {
            self.sprite_finish_reload_all_overworld();
        }
        if overworld_screen & 0x40 == 0 {
            self.sprite_initialize_mirror_portal();
        }
        let ambient_sound_effect =
            if self.game_state.inventory.save_progress.progress_indicator() < 2 {
                1
            } else {
                5
            };
        self.set_ambient_sound_effect(ambient_sound_effect);
        if self.game_state.sprites.follower_runtime.indicator() == 6 {
            self.follower_state_mut().set_indicator(0);
        }

        self.follower_link_state_mut().clear_doorway_state();
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().clear_button_b_frames();
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_movement_velocity();
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        if self.game_state.inventory.items.moon_pearl() == 0
            && self.game_state.inventory.save_progress.dark_world_state() != 0
        {
            self.follower_link_state_mut().set_bunny_state(1);
            self.follower_link_state_mut().set_handler_state(23);
            self.LoadGearPalettes_bunny();
        }
        self.set_bg_mode(9);
        self.dungeon_torch_mut().clear_lights_out_request();
        self.dungeon_room_load_mut().set_header_collision(0);
        self.follower_link_state_mut().clear_lower_level_states();
        self.increment_submodule();
        self.increment_hud_update_flag();
        self.dungeon_savegame_state_mut()
            .clear_savegame_state_bits();
        self.LoadOWMusicIfNeeded();
    }

    pub(super) fn LoadOverworldFromDungeon(&mut self) {
        self.set_indoor_flag(0);
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
        self.set_overworld_fixed_color_adjustment(0);
        self.save_progress_mut().set_palace_index_x2(0xff);
        self.memorized_tile_mut().clear_count();

        let room = self.game_state.world.location.dungeon_room();
        if room != 0x0104 && room < 0x0180 && room >= 0x0100 {
            self.LoadCachedEntranceProperties();
        } else {
            let exit_screen = self
                .asset_raw(130)
                .expect("LoadOverworldFromDungeon missing kExitData_ScreenIndex asset")
                .to_vec();
            let exit_rooms = self
                .asset_raw(131)
                .expect("LoadOverworldFromDungeon missing kExitDataRooms asset")
                .to_vec();
            let exit_map16_src = self
                .asset_raw(132)
                .expect("LoadOverworldFromDungeon missing kExitData_Map16LoadSrcOff asset")
                .to_vec();
            let exit_scroll_x = self
                .asset_raw(133)
                .expect("LoadOverworldFromDungeon missing kExitData_ScrollX asset")
                .to_vec();
            let exit_scroll_y = self
                .asset_raw(134)
                .expect("LoadOverworldFromDungeon missing kExitData_ScrollY asset")
                .to_vec();
            let exit_x = self
                .asset_raw(135)
                .expect("LoadOverworldFromDungeon missing kExitData_XCoord asset")
                .to_vec();
            let exit_y = self
                .asset_raw(136)
                .expect("LoadOverworldFromDungeon missing kExitData_YCoord asset")
                .to_vec();
            let exit_camera_x = self
                .asset_raw(137)
                .expect("LoadOverworldFromDungeon missing kExitData_CameraXScroll asset")
                .to_vec();
            let exit_camera_y = self
                .asset_raw(138)
                .expect("LoadOverworldFromDungeon missing kExitData_CameraYScroll asset")
                .to_vec();
            let exit_normal_door = self
                .asset_raw(139)
                .expect("LoadOverworldFromDungeon missing kExitData_NormalDoor asset")
                .to_vec();
            let exit_fancy_door = self
                .asset_raw(140)
                .expect("LoadOverworldFromDungeon missing kExitData_FancyDoor asset")
                .to_vec();
            let exit_scroll_up_seed = self
                .asset_raw(141)
                .expect("LoadOverworldFromDungeon missing kExitData_Unk1 asset")
                .to_vec();
            let exit_scroll_left_seed = self
                .asset_raw(142)
                .expect("LoadOverworldFromDungeon missing kExitData_Unk3 asset")
                .to_vec();

            let k = (0..79)
                .rev()
                .find(|&k| read_word_from_slice(&exit_rooms, k * 2) == room)
                .unwrap_or_else(|| {
                    panic!("LoadOverworldFromDungeon missing exit data for room {room:#06x}")
                });

            let scroll_y = read_word_from_slice(&exit_scroll_y, k * 2);
            self.set_bg1_bg2_v_live_and_copy(scroll_y);
            let scroll_x = read_word_from_slice(&exit_scroll_x, k * 2);
            self.set_bg1_bg2_h_live_and_copy(scroll_x);

            let link_y = read_word_from_slice(&exit_y, k * 2);
            let link_x = read_word_from_slice(&exit_x, k * 2);
            self.follower_link_state_mut().set_y(link_y);
            self.follower_link_state_mut().set_x(link_x);

            let src = read_word_from_slice(&exit_map16_src, k * 2);
            self.set_overworld_map16_src_off(src);
            self.set_overworld_map16_y_unit((src.wrapping_sub(0x400) & 0x0f80) >> 7);
            self.set_overworld_map16_dst_off((src.wrapping_sub(0x10) & 0x003e) >> 1);

            let camera_y = read_word_from_slice(&exit_camera_y, k * 2);
            self.set_camera_y_coord_scroll_low(camera_y);
            self.set_camera_y_coord_scroll_hi(camera_y.wrapping_sub(2));
            let camera_x = read_word_from_slice(&exit_camera_x, k * 2);
            self.set_camera_x_coord_scroll_low(camera_x);
            self.set_camera_x_coord_scroll_hi(camera_x.wrapping_sub(2));

            // C does `write_le_u16(LINK_DIRECTION_FACING, 2)` — a 16-bit store to 0x2f that also
            // zeroes the high byte at 0x30 (= LINK_Y_VELOCITY, mode-reused). set_facing only writes
            // the 0x2f byte, so clear LINK_Y_VELOCITY too to match (else a stale Link Y-velocity
            // lingers through the special-area exit, f241475).
            self.follower_link_state_mut().set_facing(2);
            self.follower_link_state_mut().set_y_velocity(0);
            let entrance_value = read_word_from_slice(&exit_normal_door, k * 2);
            let big_rock_starting_address = read_word_from_slice(&exit_fancy_door, k * 2);
            self.set_ow_entrance_value(entrance_value);
            self.dungeon_object_tracking_mut()
                .set_big_rock_starting_address(big_rock_starting_address);
            let screen = exit_screen[k] as u16;
            self.set_overworld_area_index_word(screen);
            self.set_overworld_screen_word(screen);

            let scroll_up_seed = exit_scroll_up_seed[k] as i8 as i16 as u16;
            let scroll_left_seed = exit_scroll_left_seed[k] as i8 as i16 as u16;
            self.set_overworld_scroll_up_counter(scroll_up_seed);
            self.set_overworld_scroll_left_counter(scroll_left_seed);
            self.set_overworld_scroll_down_counter(scroll_up_seed.wrapping_neg());
            self.set_overworld_scroll_right_counter(scroll_left_seed.wrapping_neg());
        }

        self.Overworld_LoadNewScreenProperties();
    }

    pub(super) fn Overworld_EnterSpecialArea(&mut self) {
        self.memorized_tile_mut().clear_count();
        self.save_spexit_area_index();
        self.save_spexit_tm_copy();
        self.save_special_exit_bg2_live_scroll();
        self.special_exit_position_mut().store_from_player();
        self.save_spexit_camera_coords();
        let overworld_screen = self.game_state.world.location.overworld_screen();
        self.set_special_exit_screen_index(overworld_screen);
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_overworld_spexit_map16_src_off(map16.src_off);
        let top = self.game_state.world.room_bounds.packed_top();
        let bottom = self.game_state.world.room_bounds.packed_bottom();
        let left = self.game_state.world.room_bounds.packed_left();
        let right = self.game_state.world.room_bounds.packed_right();
        self.set_special_exit_room_bounds(top, bottom, left, right);
        self.copy_spexit_scroll_targets();
        self.copy_spexit_scroll_counters();
        self.world_palette_theme_mut()
            .save_special_exit_tile_themes();
        self.sprite_system_mut().save_special_exit_graphics_index();
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some() {
            println!(
                "spexit-save frame={} area=0x{:04x} screen=0x{:04x} x=0x{:04x} y=0x{:04x} bg=0x{:04x}/0x{:04x} src=0x{:04x} yunit=0x{:04x} dst=0x{:04x} cam=0x{:04x}/0x{:04x} room=0x{:04x} main={} sub={}",
                self.game_state.frame.frame_counter,
                self.game_state.world.region.spexit_area_index(),
                self.special_exit_screen_index(),
                self.game_state.player.special_exit_position.x(),
                self.game_state.player.special_exit_position.y(),
                self.game_state.display.ppu_scroll_copy.special_exit_bg2_h_copy2(),
                self.game_state.display.ppu_scroll_copy.special_exit_bg2_v_copy2(),
                self.overworld_spexit_map16_src_off(),
                self.overworld_map16_y_unit(),
                self.overworld_map16_dst_off(),
                self.game_state.world.camera_boundaries.spexit_camera_x_scroll_low(),
                self.game_state.world.camera_boundaries.spexit_camera_y_scroll_low(),
                self.game_state.world.location.dungeon_room(),
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            );
        }

        self.LoadOverworldFromDungeon();
        if self.game_state.world.location.dungeon_room() == 0x1010 {
            self.set_dungeon_room(0x182);
        }

        let room_bak = self.game_state.world.location.dungeon_room_index();
        self.decrement_dungeon_room_index_by(0x80);
        let i = self.game_state.world.location.dungeon_room_index() as usize;
        self.follower_link_state_mut()
            .set_facing(SPECIAL_EXIT_DIRECTIONS[i]);
        self.reset_incremental_vram_upload_counter();
        self.sprite_system_mut()
            .set_graphics_index(SPECIAL_EXIT_SPRITE_GRAPHICS[i]);
        self.world_palette_theme_mut()
            .set_aux_tile_theme_index(SPECIAL_EXIT_AUX_GRAPHICS[i]);
        self.Overworld_LoadPalettes(SPECIAL_EXIT_BG_PALETTES[i], SPECIAL_EXIT_SPRITE_PALETTES[i]);

        let j = (self.game_state.world.location.dungeon_room_index() & 0x3f) as usize;
        self.set_overworld_offset_base_y(SPECIAL_EXIT_TOP_BOUNDS[j]);
        self.set_overworld_offset_base_x(SPECIAL_EXIT_LEFT_EDGE_OF_MAP[j] >> 3);
        self.set_overworld_offset_mask_y(0x03f0);
        self.set_overworld_offset_mask_x(0x03f0 >> 3);

        let k = (self.game_state.world.location.dungeon_room_index() & 0x7f) as usize;
        self.room_bounds_mut().set_packed_bounds(
            SPECIAL_EXIT_TOP_BOUNDS[k],
            SPECIAL_EXIT_BOTTOM_BOUNDS[k],
            SPECIAL_EXIT_LEFT_BOUNDS[k],
            SPECIAL_EXIT_RIGHT_BOUNDS[k],
        );
        self.set_up_down_scroll_target(SPECIAL_EXIT_SCROLL_Y_START[k]);
        self.set_up_down_scroll_target_end(SPECIAL_EXIT_SCROLL_Y_END[k]);
        self.set_left_right_scroll_target(SPECIAL_EXIT_SCROLL_X_START[k]);
        self.set_left_right_scroll_target_end(SPECIAL_EXIT_SCROLL_X_END[k]);

        self.set_dungeon_room_index(room_bak);
        self.Palette_SpecialOw();
    }

    pub(super) fn GetOverworldBgPalette(&self, idx: u8) -> u8 {
        self.asset_raw(109)
            .expect("GetOverworldBgPalette missing kOverworldBgPalettes asset")[idx as usize]
    }

    pub(super) fn Overworld_SetFixedColAndScroll(&mut self) {
        self.set_sub_screen_layers(0);
        let si = self.game_state.world.location.overworld_screen_index() as u16;
        let mut p = 0x19c6;
        if si == 0x80 {
            if self.game_state.world.location.dungeon_room() == 0x181 {
                self.set_sub_screen_layers(1);
                p = if si & 0x40 != 0 { 0x2a32 } else { 0x2669 };
            }
        } else if si != 0x81 {
            p = 0;
            if si != 0x5b && (si & 0xbf) != 3 && (si & 0xbf) != 5 && (si & 0xbf) != 7 {
                p = if si & 0x40 != 0 { 0x2a32 } else { 0x2669 };
            }
        }
        self.set_main_color_constant(0, p);
        self.set_aux_color_constant(0, p);
        self.set_main_color_constant(32, p);
        self.set_aux_color_constant(32, p);
        self.set_fixed_color_red(0x20);
        self.set_fixed_color_green(0x40);
        self.set_fixed_color_blue(0x80);
        if si != 0 && si != 0x40 && si != 0x5b {
            if si == 0x70 {
                self.set_sub_screen_layers(1);
                self.increment_cgram_update_flag();
                return;
            }
            let cv = if si == 3 || si == 5 || si == 7 {
                0x8c4c26
            } else if si == 0x43 || si == 0x45 {
                0x874a26
            } else {
                self.increment_cgram_update_flag();
                return;
            };
            self.set_fixed_color_red(cv as u8);
            self.set_fixed_color_green((cv >> 8) as u8);
            self.set_fixed_color_blue((cv >> 16) as u8);
        }
        if self.game_state.frame.submodule != 4 {
            self.copy_bg2_live_to_bg1_live();
            if (si & 0x3f) == 0x1b {
                let bg2_hofs = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
                let y = (bg2_hofs.wrapping_sub(0x0778) as i16) >> 1;
                self.set_bg1_h_copy2(bg2_hofs.wrapping_sub(y as u16));

                let mut a = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
                if a >= 0x06c0 {
                    a = a.wrapping_sub(0x0600) & 0x03ff;
                    let value = if a < 0x0180 {
                        (a >> 1) | 0x0600
                    } else {
                        0x06c0
                    };
                    self.set_bg1_y(value);
                } else {
                    self.set_bg1_y(((a & 0x00ff) >> 1) | 0x0600);
                }
            }
        } else if (si & 0x3f) == 0x1b {
            let value = if self.edge_transition_direction_bits() != 8 {
                0x0838
            } else {
                self.game_state.display.ppu_scroll_copy.bg2_h_copy2()
            };
            self.set_bg1_x(value);
            self.set_bg1_y(0x06c0);
        }
        self.set_sub_screen_layers(1);
        self.increment_cgram_update_flag();
    }

    pub(super) fn Ancilla_TerminateWaterfallSplashes(&mut self) {
        if self.game_state.world.location.overworld_screen_index() == 0x0f {
            for i in (0..=4).rev() {
                if self.ancilla_slot_view(i).ancilla_type() == 0x41 {
                    self.ancilla_slot_view_mut(i).clear();
                }
            }
        }
    }

    pub(super) fn Module09_LoadAuxGFX(&mut self) {
        self.clear_overworld_event_bits(0x3b, 0x20);
        self.clear_overworld_event_bits(0x7b, 0x20);

        let saved267 = self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(267)
            & !0x0080;
        let saved40 = self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(40)
            & !0x0100;
        self.save_progress_mut()
            .set_dungeon_info_word(267, saved267);
        self.save_progress_mut().set_dungeon_info_word(40, saved40);

        if self.rom_startup_timing() {
            let timing = overworld_aux_graphics_timing(self.overworld_aux_graphics_workload());
            // The ROM's decompression time follows the actual set of nonzero
            // auxiliary packs. Carry the measured workload into the scheduler
            // rather than assigning every tileset the light eleven-slice path.
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishOverworldAuxGraphics,
                timing.load_nmi_slices,
            );
            return;
        }
        self.complete_module09_load_aux_gfx();
    }

    pub(super) fn complete_module09_load_aux_gfx(&mut self) {
        self.LoadTransAuxGFX();
        self.PrepTransAuxGfx();
        self.set_core_update_disable_flag(9);
        self.set_pending_nmi_subroutine(9);
        self.increment_submodule();
    }

    fn prepare_overworld_load_overlays(&mut self) {
        let overworld_screen = self.game_state.world.location.overworld_screen();
        self.set_prev_screen_index_word(overworld_screen);
        self.store_overworld_prev_map16_load_state(
            self.game_state.world.overworld.map16.active_load,
        );
        let screen_transition = self.screen_transition();
        self.set_previous_screen_transition(screen_transition);
        self.save_previous_screen_transition_direction_bits();

        self.clear_overlay_index_word();
        self.clear_bg1_scroll_subpixels();

        let si = u16::from(self.game_state.world.location.overworld_screen_index());
        let mut xv;
        if si >= 0x80 {
            xv = 0x97;
            let room = self.game_state.world.location.dungeon_room();
            if room == 0x0180 {
                if self.game_state.world.overworld.event_info.event_info(0x80) & 0x40 != 0 {
                    self.set_sub_screen_layers(0);
                    self.increment_submodule();
                    return;
                }
            } else if room == 0x0181 {
                xv = 0x94;
            } else if room == 0x0189 {
                xv = 0x93;
            } else {
                if room == 0x0182 || room == 0x0183 {
                    self.set_ambient_sound_effect(1);
                }
                self.set_sub_screen_layers(0);
                self.increment_submodule();
                return;
            }
        } else if (si & 0x3f) == 0 {
            xv = if (si & 0x40) == 0
                && self.game_state.world.overworld.event_info.event_info(0x80) & 0x40 != 0
            {
                0x9e
            } else {
                0x9d
            };
        } else if matches!(si, 0x03 | 0x05 | 0x07) {
            xv = 0x95;
        } else if matches!(si, 0x43 | 0x45 | 0x47) {
            xv = 0x9c;
        } else if si == 0x70 {
            xv = 0x9c;
            if self.game_state.world.overworld.event_info.event_info(0x70) & 0x20 == 0 {
                xv = 0x9f;
            }
        } else {
            xv = if self.game_state.inventory.save_progress.progress_indicator() < 2 {
                0x9f
            } else {
                0x96
            };
        }

        self.set_overworld_map16_src_off(0x0390);
        self.set_overlay_index_word(xv);
        self.set_overworld_screen_word(xv);
        let src = self.overworld_map16_src_off();
        self.set_overworld_map16_y_unit((src.wrapping_sub(0x400) & 0x0f80) >> 7);
        self.set_overworld_map16_dst_off((src.wrapping_sub(0x10) & 0x003e) >> 1);
        self.clear_screen_transition();
        self.clear_edge_transition_direction_bits();
        self.clear_screen_transition_direction_bits_word();
        self.set_color_window_selection(0x82);
        self.set_main_screen_layers(0x16);
        self.set_sub_screen_layers(1);
        let ambient_sound_effect = self
            .overworld_config_table()
            .music(self.game_state.world.location.overworld_screen_index() as usize)
            >> 4;
        self.set_ambient_sound_effect(ambient_sound_effect);

        if matches!(xv, 0x97 | 0x94 | 0x93 | 0x9d | 0x9e | 0x9f) {
            self.set_color_math_control(0x72);
        } else {
            let prev = self.game_state.world.region.prev_screen_index_byte();
            if xv == 0x95
                || xv == 0x9c
                || prev == 0x5b
                || (prev == 0x1b
                    && (self.game_state.frame.submodule == 35
                        || self.game_state.frame.submodule == 44))
            {
                self.set_color_math_control(0x20);
            } else {
                self.set_sub_screen_layers(0);
                self.set_color_math_control(0x20);
            }
        }
    }

    pub(super) fn finish_overworld_load_overlays(&mut self) {
        self.LoadOverworldOverlay();
        if self.game_state.world.region.overlay_index() == 0x94 {
            let value = self.game_state.display.ppu_scroll_copy.bg1_v_copy2() | 0x0100;
            self.set_bg1_y(value);
        }

        let overworld_screen = self.game_state.world.region.prev_screen_index_word();
        self.set_overworld_screen_word(overworld_screen);
        self.store_overworld_map16_load_state(self.game_state.world.overworld.map16.previous_load);
        let screen_transition = self.previous_screen_transition();
        self.set_screen_transition(screen_transition);
        self.restore_previous_screen_transition_direction_bits();
    }

    pub(super) fn Overworld_LoadOverlays2(&mut self) {
        self.prepare_overworld_load_overlays();
        if self.rom_startup_timing()
            && self.game_state.frame.main_module == 9
            && self.game_state.frame.submodule == 0x20
        {
            // LoadOverworldOverlay remains inside its map32 decode and
            // Map16ToMap8 conversion across the measured vblank boundaries.
            // Keep the temporary overlay screen index live until that work
            // returns instead of restoring the gameplay screen atomically.
            let schedule = self
                .module09_cpu_schedule
                .expect("Module09/$20 timing must be captured at its leading NMI");
            self.game_execution_scheduler
                .schedule_cpu_timed_work_from_current_main_iteration(
                    GameWorkContinuation::FinishWorldMapOverlayReload,
                    schedule.submodule_nmis,
                );
            return;
        }
        self.finish_overworld_load_overlays();
    }

    pub(super) fn Overworld_LoadOverlays(&mut self) {
        self.sprite_initialize_slots();
        self.sprite_reload_all_overworld();
        self.follower_link_state_mut().clear_state_bits();
        self.follower_link_state_mut().clear_picking_throw_state();
        self.set_ambient_sound_effect(5);
        self.Overworld_LoadOverlays2();
    }

    pub(super) fn PreOverworld_LoadOverlays(&mut self) {
        self.set_ambient_sound_effect(5);
        // The ROM reaches the screen-specific ambient selection two scanlines
        // later, before LoadOverworldOverlay becomes the interruptible part of
        // this call. Keep that lightweight prefix on the entry boundary so NMI
        // never observes the temporary value 5.
        self.prepare_overworld_load_overlays();
        if self.begin_pre_overworld_overlays_work() {
            return;
        }
        self.finish_overworld_load_overlays();
    }

    pub(super) fn complete_pre_overworld_load_overlays(&mut self) {
        self.finish_overworld_load_overlays();
    }

    pub(super) fn Overworld_LoadAmbientOverlay(&mut self, load_map_data: bool) {
        let bak_src_off = self.overworld_map16_src_off();
        let bak_dst_off = self.overworld_map16_dst_off();
        let bak_y_unit = self.overworld_map16_y_unit();
        if self.overworld_map_is_small() {
            self.set_small_overworld_mirror_map_position();
        }
        if load_map_data {
            self.Overworld_DrawQuadrantsAndOverlays();
        }
        self.Map16ToMap8(OverworldMap16SourcePage::Main, 0);
        self.set_overworld_map16_y_unit(bak_y_unit);
        self.set_overworld_map16_dst_off(bak_dst_off);
        self.set_overworld_map16_src_off(bak_src_off);
        self.set_pending_nmi_subroutine(4);
        self.set_core_update_disable_flag(4);
        self.increment_submodule();
        self.set_screen_brightness(0);
    }

    pub(super) fn Overworld_LoadAmbientOverlayFalse(&mut self) {
        if self.rom_startup_timing()
            && self.game_state.frame.main_module == 9
            && self.game_state.frame.submodule == 0x21
        {
            // The main-page Map16ToMap8 conversion is likewise interruptible;
            // INIDISP=0 and submodule $22 are reached only after it returns.
            let schedule = self
                .module09_cpu_schedule
                .expect("Module09/$21 timing must be captured at its leading NMI");
            self.game_execution_scheduler
                .schedule_cpu_timed_work_from_current_main_iteration(
                    GameWorkContinuation::FinishWorldMapAmbientMap8,
                    schedule.submodule_nmis,
                );
            return;
        }
        self.Overworld_LoadAmbientOverlay(false);
    }

    pub(super) fn Overworld_LoadAndBuildScreen(&mut self) {
        self.Overworld_LoadAmbientOverlay(true);
    }

    pub(super) fn LoadOverworldOverlay(&mut self) {
        self.OverworldLoad_LoadSubOverlayMap32();
        self.Map16ToMap8(OverworldMap16SourcePage::Overlay, 0x1000);
        self.set_pending_nmi_subroutine(4);
        self.set_core_update_disable_flag(4);
        self.increment_submodule();
    }

    pub(super) fn GetMap8toTileAttr(&self) -> Vec<u8> {
        self.asset_raw(163)
            .expect("GetMap8toTileAttr missing kMap8DataToTileAttr asset")
            .to_vec()
    }

    pub(super) fn GetMap16toMap8Table(&self) -> Vec<u8> {
        self.asset_raw(70)
            .expect("GetMap16toMap8Table missing kMap16ToMap8 asset")
            .to_vec()
    }

    pub(super) fn LookupInOwEntranceTab(&self, r0: u16, r2: u16) -> bool {
        for i in (0..OVERWORLD_ENTRANCE_PRIMARY_AREA_BY_INDEX.len()).rev() {
            if r0 == OVERWORLD_ENTRANCE_PRIMARY_AREA_BY_INDEX[i]
                && r2 == OVERWORLD_ENTRANCE_SECONDARY_AREA_BY_INDEX[i]
            {
                return true;
            }
        }
        false
    }

    pub(super) fn LookupInOwEntranceTab2(&self, pos: u16) -> i32 {
        let entrance_pos = self
            .asset_raw(125)
            .expect("LookupInOwEntranceTab2 missing kOverworld_Entrance_Pos asset");
        let entrance_area = self
            .asset_raw(124)
            .expect("LookupInOwEntranceTab2 missing kOverworld_Entrance_Area asset");
        for i in (0..=128).rev() {
            if pos == read_word_from_slice(entrance_pos, i * 2)
                && self.game_state.world.region.overworld_area_index_word()
                    == read_word_from_slice(entrance_area, i * 2)
            {
                return i as i32;
            }
        }
        -1
    }

    pub(super) fn CanEnterWithTagalong(&self, e: i32) -> bool {
        let t = self.game_state.sprites.follower_runtime.indicator();
        t == 0 || t == 5 || t == 14 || t == 1 || (t == 7 || t == 8) && e >= 59
    }

    pub(super) fn Module09_Overworld(&mut self) {
        self.replay_trace_submodule("module09-entry");
        let entry_frame = (
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
        );
        match self.game_state.frame.submodule {
            0 => self.Module09_00_PlayerControl(),
            1 | 15 | 26 | 38 => self.Module09_LoadAuxGFX(),
            2 | 16 | 27 | 39 => self.Overworld_FinishTransGfx(),
            3 | 17 => self.Module09_LoadNewMapAndGFX(),
            4 | 18 => self.Module09_LoadNewSprites(),
            5 | 19 => self.Overworld_StartScrollTransition(),
            6 | 20 => self.Overworld_RunScrollTransition(),
            7 | 21 => self.Overworld_EaseOffScrollTransition(),
            8 => self.Overworld_FinalizeEntryOntoScreen(),
            9 => self.Module09_09_OpenBigDoorFromExiting(),
            10 => self.Module09_0A_WalkFromExiting_FacingDown(),
            11 => self.Module09_0B_WalkFromExiting_FacingUp(),
            12 => self.Module09_0C_OpenBigDoor(),
            13 | 23 | 36 => self.Overworld_StartMosaicTransition(),
            14 => self.PreOverworld_LoadOverlays(),
            22 | 41 => self.Module09_FadeBackInFromMosaic(),
            24 => self.Overworld_Func18(),
            25 => self.Overworld_Func19(),
            28 => self.Overworld_Func1C(),
            29 => self.Overworld_Func1D(),
            30 => self.Overworld_Func1E(),
            31 => self.Overworld_Func1F(),
            32 => self.Overworld_LoadOverlays2(),
            33 => self.Overworld_LoadAmbientOverlayFalse(),
            34 => self.Overworld_Func22(),
            35 | 44 => self.Module09_MirrorWarp(),
            37 => self.Overworld_LoadOverlays(),
            40 => self.Overworld_LoadAndBuildScreen(),
            42 => self.Module09_2A_RecoverFromDrowning(),
            43 => self.Overworld_Func2B(),
            45 => self.Overworld_WeathervaneExplosion(),
            46 => self.Module09_2E_Whirlpool(),
            47 => self.Overworld_Func2F(),
            submodule => panic!("Module09_Overworld invalid submodule_index: {submodule}"),
        }
        self.replay_trace_submodule("module09-after-submodule");
        if self.rom_startup_timing() && self.game_execution_scheduler.work_is_pending() {
            return;
        }
        self.complete_module09_overworld_after_submodule();
        if matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishSpriteMain {
                boundary: crate::zelda_rtl::SpriteMainCpuBoundary::BeforeFirstSlot,
                caller: SpriteMainCpuCaller::Module09 { .. },
            })
        ) {
            let advanced = (
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            );
            if advanced != entry_frame {
                // The handler's trailing module/submodule advance belongs to
                // the host where the ROM finishes its long work and enters
                // Sprite_Main; keep the entry bytes until that resume.
                self.set_submodule(entry_frame.1);
                self.set_main_module(entry_frame.0);
                self.pending_module09_frame_advance = Some(advanced);
            }
        }
    }

    pub(super) fn complete_module09_overworld_after_submodule(&mut self) {
        self.complete_module09_sprite_and_hud_suffix_with_rain_state(false);
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            return;
        }
        self.OverworldOverlay_HandleRain();
        self.replay_trace_ram_watch("module09-after-rain");
        self.replay_trace_submodule("module09-exit");
    }

    /// Finish the caller suffix after an interrupted sprite load whose
    /// provisional display generation already advanced the rain overlay.
    pub(super) fn complete_module09_overworld_after_prepublished_rain(&mut self) {
        self.complete_module09_sprite_and_hud_suffix_with_rain_state(true);
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            return;
        }
        self.replay_trace_submodule("module09-exit");
    }

    pub(super) fn complete_module09_sprite_and_hud_suffix(&mut self) {
        self.complete_module09_sprite_and_hud_suffix_with_rain_state(false);
    }

    fn complete_module09_sprite_and_hud_suffix_with_rain_state(
        &mut self,
        rain_already_published: bool,
    ) {
        let caller = self.begin_module09_sprite_main();
        assert!(
            self.active_module09_sprite_main_return
                .replace(Module09ItemReceiptCallerReturn {
                    scroll: caller,
                    rain_already_published,
                    after_sprite_main: Module09AfterSpriteMain::Ordinary,
                })
                .is_none(),
            "Module 9 entered a second Sprite_Main caller before the first returned",
        );
        if let Some((boundary, resume_boundary)) =
            self.take_original_timing_sprite_main_boundary_for_fresh_caller()
        {
            self.arm_sprite_main_cpu_continuation(
                boundary,
                1,
                SpriteMainCpuCaller::Module09 {
                    boundary: resume_boundary,
                },
            );
        } else if let Some(boundary) = self.take_original_timing_sprite_main_progress() {
            // The host returned while the source remained inside Sprite_Main
            // without accepting an NMI (the Module07 lane's typed host-boundary
            // checkpoint, here for the overworld caller).
            self.arm_sprite_main_cpu_continuation(
                boundary,
                1,
                SpriteMainCpuCaller::Module09 {
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            );
        } else if let Some(boundary) =
            self.original_timing_fresh_iteration_interrupted_before_sprite_main()
        {
            // The fresh Module09/Module0B iteration was interrupted inside
            // Sprite_Main's shared prefix before any slot returned (route
            // hosts 165775 and 186360 in Module0B/$18, 508544 and 811514 in
            // Module09/$23). Suspend at the loop entry.
            self.arm_sprite_main_cpu_continuation(
                crate::zelda_rtl::SpriteMainCpuBoundary::BeforeFirstSlot,
                1,
                SpriteMainCpuCaller::Module09 { boundary },
            );
        }
        self.sprite_main();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            return;
        }
        let caller = self
            .active_module09_sprite_main_return
            .take()
            .expect("Module 9 Sprite_Main return frame was lost");
        if let Some(boundary) = self.take_forwarded_original_timing_main_loop_interruption(
            crate::MainLoopInterruption::LinkOam,
        ) {
            // C has restored none of Module09's four stack-local scroll values
            // yet: the next call is LinkOam_Main, where the source accepted
            // the host's trailing NMI. Preserve that caller frame and resume
            // the whole LinkOam/HUD/rain suffix after the interrupt. This is
            // the same semantic continuation used by the longer world-map
            // callers; no module, room, frame, or CPU address selects it.
            let continuation = GameWorkContinuation::FinishModule09LinkOamCallerReturn { caller };
            match boundary {
                OriginalTimingBoundary::HostReturn => {
                    self.game_execution_scheduler.schedule_work(continuation, 1)
                }
                OriginalTimingBoundary::NmiAccepted => self
                    .game_execution_scheduler
                    .schedule_after_current_trailing_nmi(continuation),
            }
            return;
        }
        self.complete_module09_after_sprite_main(caller.scroll);
    }

    pub(super) fn begin_module09_sprite_main(&mut self) -> Module09SpriteMainReturn {
        let bg2x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg2y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        let bg1x = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
        let bg1y = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
        let offx = self.game_state.world.scroll.bg1_x_offset();
        let offy = self.game_state.world.scroll.bg1_y_offset();

        let bg2x_off = bg2x.wrapping_add(offx);
        let bg2y_off = bg2y.wrapping_add(offy);
        let bg1x_off = bg1x.wrapping_add(offx);
        let bg1y_off = bg1y.wrapping_add(offy);
        self.set_bg2_h_live_and_copy(bg2x_off);
        self.set_bg2_v_live_and_copy(bg2y_off);
        self.set_bg1_h_live_and_copy(bg1x_off);
        self.set_bg1_v_live_and_copy(bg1y_off);

        self.replay_trace_ram_watch("module09-before-sprite-main");
        Module09SpriteMainReturn {
            bg2_x: bg2x,
            bg2_y: bg2y,
            bg1_x: bg1x,
            bg1_y: bg1y,
        }
    }

    fn complete_module09_after_sprite_main(&mut self, caller: Module09SpriteMainReturn) {
        self.replay_trace_ram_watch("module09-after-sprite-main");

        self.set_bg2_x(caller.bg2_x);
        self.set_bg2_y(caller.bg2_y);
        self.set_bg1_x(caller.bg1_x);
        self.set_bg1_y(caller.bg1_y);
        self.replay_trace_ram_watch("module09-after-scroll-restore");

        self.replay_trace_ram_watch("module09-before-link-oam");
        self.link_oam_main();
        self.replay_trace_ram_watch("module09-after-link-oam");
        self.hud_refill_logic();
        self.replay_trace_ram_watch("module09-after-refill");
    }

    /// Resume the ordinary Module09 caller immediately after an interrupted
    /// `Sprite_Main` returns, preserving the C statement order and its four
    /// stack-local scroll values without replaying the submodule or sprite loop.
    pub(super) fn complete_module09_overworld_after_resumed_sprite_main(
        &mut self,
        caller: Module09ItemReceiptCallerReturn,
    ) {
        self.complete_module09_after_sprite_main(caller.scroll);
        if !caller.rain_already_published {
            self.OverworldOverlay_HandleRain();
            self.replay_trace_ram_watch("module09-after-rain");
        }
        self.replay_trace_submodule("module09-exit");
    }

    /// Resume `Overworld_LoadOverlays2` through the exact first caller NMI.
    /// The original ROM reaches Sprite_Main slot 8 before that boundary and
    /// resumes at slot 7 on the following host; the four scroll locals remain
    /// live on the 65816 stack across the interrupt.
    pub(super) fn begin_world_map_overlay_module09_sprite_return(
        &mut self,
        boundary: SpriteMainCpuBoundary,
        nmi_slices: u8,
    ) {
        let module09 = self.begin_module09_sprite_main();
        self.arm_sprite_main_cpu_continuation(
            boundary,
            nmi_slices,
            SpriteMainCpuCaller::WorldMapOverlayReload { module09 },
        );
        self.sprite_main();
        debug_assert!(self
            .game_execution_scheduler
            .work_suspends_translated_call_stack());
    }

    pub(super) fn complete_module09_link_oam_caller_return(
        &mut self,
        caller: Module09ItemReceiptCallerReturn,
    ) {
        self.complete_module09_overworld_after_resumed_sprite_main(caller);
    }

    fn publish_module09_transition_sprites_without_scroll(&mut self) {
        let bg2x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg2y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        let bg1x = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
        let bg1y = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
        let offx = self.game_state.world.scroll.bg1_x_offset();
        let offy = self.game_state.world.scroll.bg1_y_offset();

        // Sprite coordinates use the offset copies, but the ROM has not
        // returned to Module09's caller suffix yet, so keep the live PPU
        // scroll publication at the preceding scanout values.
        self.set_bg2_x(bg2x.wrapping_add(offx));
        self.set_bg2_y(bg2y.wrapping_add(offy));
        self.set_bg1_x(bg1x.wrapping_add(offx));
        self.set_bg1_y(bg1y.wrapping_add(offy));
        // This Sprite_Main run only stages the provisional display
        // generation; the ROM's own Sprite_Main return happens when the
        // scheduled loader continuation completes. Keep the host's
        // source-return claim accounting owned by that real return.
        let source_claim_scope = self
            .original_timing_sprite_main_return_claims_remaining
            .take();
        // The ROM is inside Sprite_OverworldReloadAll_justLoad here and runs
        // no sprite logic: this staging walk must not leak any CPU-visible
        // mutation (a walking soldier stepped one extra time at route host
        // 115867). Keep only the staged OAM shadow; restore everything else.
        let ram_before_provisional_walk = self.ram.clone();
        let game_state_before_provisional_walk = self.game_state.clone();
        self.sprite_main();
        assert_eq!(
            self.original_timing_sprite_main_return_claims_remaining, None,
            "the provisional transition sprite build cannot open its own Sprite_Main claim scope",
        );
        self.original_timing_sprite_main_return_claims_remaining = source_claim_scope;
        let staged_oam =
            self.ram[crate::game_state::constants::OAM_BUF..][..0x220].to_vec();
        self.ram.copy_from_slice(&ram_before_provisional_walk);
        self.game_state = game_state_before_provisional_walk;
        self.ram[crate::game_state::constants::OAM_BUF..][..0x220]
            .copy_from_slice(&staged_oam);
        self.set_bg2_x(bg2x);
        self.set_bg2_y(bg2y);
        self.set_bg1_x(bg1x);
        self.set_bg1_y(bg1y);

        // These calls only build the provisional sprite generation needed
        // while the loader is interrupted. The remaining Module09 caller
        // suffix runs after the scheduled loader continuation returns. Rain is
        // the exception: the ROM reaches that weather tick before Snes9x
        // publishes the first submodule-6 scanout, so record it in this
        // provisional generation and do not advance it again on completion.
        self.link_oam_main();
        self.hud_refill_logic();
        self.OverworldOverlay_HandleRain();
    }

    pub(super) fn Module09_00_PlayerControl(&mut self) {
        self.replay_trace_submodule("module09-player-entry");
        if (self
            .game_state
            .world
            .transient
            .flag_custom_spell_anim_active()
            | self.game_state.player.follower_link.immobilized_flag()
            | self.game_state.player.follower_link.menu_block_flag()
            | self.special_entrance_trigger())
            == 0
        {
            if self.game_state.player.follower_link.filtered_joypad_h() & 0x10 != 0 {
                self.set_overworld_map_state(0);
                self.set_submodule(1);
                self.save_main_module_for_menu();
                self.set_main_module(14);
                return;
            }
            if self.DidPressButtonForMap() {
                self.set_overworld_map_state(0);
                self.set_submodule(7);
                self.save_main_module_for_menu();
                self.set_main_module(14);
                return;
            }
            if self.game_state.player.follower_link.joypad1h_last() & 0x20 != 0 {
                self.DisplaySelectMenu();
                return;
            }
            self.hud_handle_item_switch_inputs();
        }

        if self.special_entrance_trigger() != 0 {
            self.Overworld_AnimateEntrance();
        }
        self.replay_trace_ram_watch("module09-player-before-link-main");
        self.link_main();
        self.replay_trace_ram_watch("module09-player-after-link-main");
        if self.hud_state().super_bomb_indicator_timer() != 0xff {
            self.hud_super_bomb_indicator();
        }
        self.replay_trace_ram_watch("module09-player-after-super-bomb");
        let area = ((self.game_state.player.follower_link.y() & 0x1e00) >> 5)
            | ((self.game_state.player.follower_link.x() & 0x1e00) >> 8);
        self.set_current_area_of_player_word(area);
        self.Graphics_LoadChrHalfSlot();
        self.replay_trace_ram_watch("module09-player-after-chr");
        self.Overworld_OperateCameraScroll();
        self.replay_trace_ram_watch("module09-player-after-camera");
        if self.game_state.frame.main_module != 11 {
            self.Overworld_UseEntrance();
            self.replay_trace_ram_watch("module09-player-after-use-entrance");
            self.Overworld_DwDeathMountainPaletteAnimation();
            self.replay_trace_ram_watch("module09-player-after-dm-palette");
            self.OverworldHandleTransitions();
            self.replay_trace_ram_watch("module09-player-after-transitions");
        } else {
            self.ScrollAndCheckForSOWExit();
            self.replay_trace_ram_watch("module09-player-after-sow-exit");
        }
        self.replay_trace_submodule("module09-player-exit");
    }

    pub(super) fn Overworld_UseEntrance(&mut self) {
        let xc = self.game_state.player.follower_link.x() >> 3;
        let yc = self.game_state.player.follower_link.y().wrapping_add(7);
        let mut pos = ((yc.wrapping_sub(self.game_state.world.scroll.overworld_offset_base_y())
            & self.game_state.world.scroll.overworld_offset_mask_y())
            * 8)
        .wrapping_add(
            xc.wrapping_sub(self.game_state.world.scroll.overworld_offset_base_x())
                & self.game_state.world.scroll.overworld_offset_mask_x(),
        );

        let mut x = self
            .game_state
            .dungeon
            .room_tilemaps
            .bg2_tile_by_byte_pos(pos) as usize
            * 4;
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("Overworld_UseEntrance missing kMap16ToMap8 asset")
            .to_vec();
        if self.game_state.player.follower_link.facing() == 0 {
            let mut a = read_word_from_slice(&map16_to_map8, (x + 1) * 2) & 0x41ff;
            if a == 0x00e9 {
                self.overworld_draw_map16_persist(pos, 0x0da4);
                self.overworld_draw_map16_persist(pos.wrapping_add(2), 0x0da6);
                self.set_sound_effect_2(21);
                self.set_bg_vram_load_mode(1);
                return;
            }
            if a != 0x0149 && a != 0x0169 {
                x = self
                    .game_state
                    .dungeon
                    .room_tilemaps
                    .bg2_tile(((pos >> 1) + 1) as usize) as usize
                    * 4;
                a = read_word_from_slice(&map16_to_map8, x * 2) & 0x41ff;
                if a == 0x40e9 {
                    pos = pos.wrapping_sub(2);
                    self.overworld_draw_map16_persist(pos, 0x0da4);
                    self.overworld_draw_map16_persist(pos.wrapping_add(2), 0x0da6);
                    self.set_sound_effect_2(21);
                    self.set_bg_vram_load_mode(1);
                    return;
                }
                if a == 0x4149 || a == 0x4169 {
                    pos = pos.wrapping_sub(2);
                } else {
                    a = 0;
                }
            }
            if a == 0x0149 || a == 0x0169 || a == 0x4149 || a == 0x4169 {
                self.dungeon_doors_mut().clear_door_open_counter_low();
                if a & 0x20 != 0 {
                    if self.game_state.inventory.save_progress.progress_indicator() & 0x0f >= 3 {
                        // Mirror the C goto after: skip opening, continue entrance lookup.
                    } else {
                        self.dungeon_doors_mut().set_door_open_counter_low(24);
                        self.dungeon_object_tracking_mut()
                            .set_big_rock_starting_address(pos.wrapping_sub(0x80));
                        self.set_sound_effect_2(21);
                        self.set_subsubmodule(0);
                        self.set_door_animation_step(0);
                        self.set_submodule(12);
                        return;
                    }
                } else {
                    self.dungeon_object_tracking_mut()
                        .set_big_rock_starting_address(pos.wrapping_sub(0x80));
                    self.set_sound_effect_2(21);
                    self.set_subsubmodule(0);
                    self.set_door_animation_step(0);
                    self.set_submodule(12);
                    return;
                }
            }
        }

        if !self.LookupInOwEntranceTab(
            read_word_from_slice(&map16_to_map8, (x + 2) * 2) & 0x01ff,
            read_word_from_slice(&map16_to_map8, (x + 3) * 2) & 0x01ff,
        ) {
            self.set_big_key_door_message_triggered(0);
            return;
        }

        let lx = self.LookupInOwEntranceTab2(pos);
        if lx < 0 {
            return;
        }
        let entrance = self
            .asset_raw(126)
            .expect("Overworld_UseEntrance missing kOverworld_Entrance_Id asset")[lx as usize];
        if self.game_state.sprites.follower_runtime.dropped() == 0
            && (self.game_state.player.follower_link.item_hold_pose() == 1
                || !self.CanEnterWithTagalong(i32::from(entrance).wrapping_sub(1)))
        {
            if self
                .game_state
                .world
                .transient
                .big_key_door_message_triggered()
                == 0
            {
                self.set_big_key_door_message_triggered(1);
                self.dialogue_message_index_mut().set_value(5);
                self.main_show_text_message();
            }
        } else {
            self.set_which_entrance_byte(entrance);
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.set_main_module(15);
            self.set_saved_module_for_menu(6);
            self.set_submodule(0);
            self.set_subsubmodule(0);
        }
    }

    pub(super) fn Overworld_AnimateEntrance(&mut self) {
        let j = self.special_entrance_trigger();
        self.follower_link_state_mut().set_immobilized_flag(j);
        self.set_modal_pause_flag(j);
        self.set_core_update_disable_flag(j);
        match j {
            1 => self.Overworld_AnimateEntrance_PoD(),
            2 => self.Overworld_AnimateEntrance_Skull(),
            3 => self.Overworld_AnimateEntrance_Mire(),
            4 => self.Overworld_AnimateEntrance_TurtleRock(),
            5 => self.Overworld_AnimateEntrance_GanonsTower(),
            _ => panic!("Overworld_AnimateEntrance invalid trigger_special_entrance: {j}"),
        }
    }

    fn entrance_counter_inc_is(&mut self, target: u8) -> bool {
        self.increment_entrance_sequence_counter();
        self.entrance_sequence_counter() == target
    }

    fn entrance_draw_tiles(&mut self, entries: &[(u16, u16)]) {
        for &(pos, tile) in entries {
            self.overworld_draw_map16_persist(pos, tile);
        }
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn OverworldEntrance_AdvanceAndBoom(&mut self) {
        self.increment_subsubmodule();
        self.clear_entrance_sequence_counter();
        self.set_sound_effect_1(12);
        self.set_sound_effect_2(7);
    }

    pub(super) fn OverworldEntrance_PlayJingle(&mut self) {
        self.set_sound_effect_2(27);
        self.clear_special_entrance_trigger();
        self.set_subsubmodule(0);
        self.clear_core_update_disable_flag();
        self.follower_link_state_mut().clear_immobilized();
        self.clear_modal_pause_flag();
        self.set_bg1_x_offset(0);
        self.set_bg1_y_offset(0);
    }

    pub(super) fn OverworldEntrance_DrawManyTR(&mut self) {
        for (i, pos) in TURTLE_ROCK_ENTRANCE_DRAW_POSITIONS.into_iter().enumerate() {
            self.overworld_draw_map16_persist(pos, 0x0e78 + i as u16);
        }
        self.set_bg_vram_load_mode(1);
        self.set_core_update_disable_flag(1);
    }

    pub(super) fn Overworld_AnimateEntrance_PoD(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => {
                if !self.entrance_counter_inc_is(0x40) {
                    return;
                }
                self.OverworldEntrance_AdvanceAndBoom();
                self.set_overworld_event_bits(0x5e, 0x20);
                self.entrance_draw_tiles(&[
                    (0x01e6, 0x0e31),
                    (0x02ea, 0x0e30),
                    (0x026a, 0x0e26),
                    (0x02ea, 0x0e27),
                ]);
            }
            1 => {
                if !self.entrance_counter_inc_is(0x20) {
                    return;
                }
                self.OverworldEntrance_AdvanceAndBoom();
                self.entrance_draw_tiles(&[(0x026a, 0x0e28), (0x02ea, 0x0e29)]);
            }
            2 => {
                if !self.entrance_counter_inc_is(0x20) {
                    return;
                }
                self.OverworldEntrance_AdvanceAndBoom();
                self.entrance_draw_tiles(&[(0x026a, 0x0e2a), (0x02ea, 0x0e2b), (0x036a, 0x0e2c)]);
            }
            3 => {
                if !self.entrance_counter_inc_is(0x20) {
                    return;
                }
                self.OverworldEntrance_AdvanceAndBoom();
                self.entrance_draw_tiles(&[(0x026a, 0x0e2d), (0x02ea, 0x0e2e), (0x036a, 0x0e2f)]);
            }
            4 => {
                if self.entrance_counter_inc_is(0x20) {
                    self.OverworldEntrance_PlayJingle();
                }
            }
            _ => {}
        }
    }

    pub(super) fn Overworld_AnimateEntrance_Skull(&mut self) {
        let entries: &[(u16, u16)] = match self.game_state.frame.subsubmodule {
            0 => {
                if !self.entrance_counter_inc_is(4) {
                    return;
                }
                &[(0x409 * 2, 0x0e06), (0x40a * 2, 0x0e06)]
            }
            1 => {
                if !self.entrance_counter_inc_is(12) {
                    return;
                }
                &[
                    (0x3c8 * 2, 0x0e07),
                    (0x3c9 * 2, 0x0e08),
                    (0x3ca * 2, 0x0e09),
                    (0x3cb * 2, 0x0e0a),
                ]
            }
            2 => {
                if !self.entrance_counter_inc_is(12) {
                    return;
                }
                &[
                    (0x388 * 2, 0x0e07),
                    (0x389 * 2, 0x0e08),
                    (0x38a * 2, 0x0e09),
                    (0x38b * 2, 0x0e0a),
                ]
            }
            3 => {
                if !self.entrance_counter_inc_is(12) {
                    return;
                }
                &[
                    (0x2c8 * 2, 0x0e11),
                    (0x2cb * 2, 0x0e12),
                    (0x308 * 2, 0x0e0d),
                    (0x309 * 2, 0x0e0e),
                    (0x30a * 2, 0x0e0f),
                    (0x30b * 2, 0x0e10),
                    (0x349 * 2, 0x0e0b),
                    (0x34a * 2, 0x0e0c),
                ]
            }
            4 => {
                if !self.entrance_counter_inc_is(12) {
                    return;
                }
                &[
                    (0x2c8 * 2, 0x0e13),
                    (0x2cb * 2, 0x0e14),
                    (0x308 * 2, 0x0e15),
                    (0x309 * 2, 0x0e16),
                    (0x30a * 2, 0x0e17),
                    (0x30b * 2, 0x0e18),
                    (0x349 * 2, 0x0e19),
                    (0x34a * 2, 0x0e1a),
                ]
            }
            _ => return,
        };
        self.clear_entrance_sequence_counter();
        self.increment_subsubmodule();
        if self.game_state.frame.subsubmodule == 1 {
            let screen = self.game_state.world.location.overworld_screen_index() as usize;
            self.set_overworld_event_bits(screen, 0x20);
        }
        self.entrance_draw_tiles(entries);
        self.set_sound_effect_2(0x16);
        if self.game_state.frame.subsubmodule == 5 {
            self.OverworldEntrance_PlayJingle();
        }
    }

    fn draw_mire_body(&mut self, start: u16) {
        for (i, pos) in MIRE_ENTRANCE_BODY_POSITIONS.into_iter().enumerate() {
            self.overworld_draw_map16_persist(pos, start + i as u16);
        }
        self.set_bg_vram_load_mode(1);
    }

    fn draw_mire_row(&mut self, row: u16, start: u16) -> u16 {
        for i in 0..4 {
            self.overworld_draw_map16_persist(row + i * 2, start + i);
        }
        start + 4
    }

    pub(super) fn Overworld_AnimateEntrance_Mire(&mut self) {
        if self.game_state.frame.subsubmodule >= 2 {
            let x = if self.game_state.frame.frame_counter & 1 != 0 {
                (-1i16) as u16
            } else {
                1
            };
            self.set_bg1_x_offset(x);
            self.set_bg1_y_offset(x.wrapping_neg());
        }

        match self.game_state.frame.subsubmodule {
            0 => {
                self.increment_entrance_sequence_counter();
                let mut j = self.entrance_sequence_counter() as u16;
                if j < 32 {
                    return;
                }
                j -= 32;
                if j == 207 {
                    self.set_subsubmodule(1);
                    self.clear_entrance_sequence_counter();
                }
                self.set_sub_screen_layers(u8::from(
                    MIRE_ENTRANCE_MASK_BITS[(j >> 3) as usize] & (0x80 >> (j & 7)) != 0,
                ));
            }
            1 | 2 => {
                self.increment_entrance_sequence_counter();
                let j = self.entrance_sequence_counter();
                if j == 16 {
                    self.increment_subsubmodule();
                    self.set_ambient_sound_effect(7);
                }
                if j == 72 {
                    self.OverworldEntrance_AdvanceAndBoom();
                    let screen = self.game_state.world.location.overworld_screen_index() as usize;
                    self.set_overworld_event_bits(screen, 0x20);
                    self.draw_mire_body(0x0e48);
                }
            }
            3 => {
                if self.entrance_counter_inc_is(72) {
                    self.OverworldEntrance_AdvanceAndBoom();
                    let j = self.draw_mire_row(0x05a2, 0x0e54);
                    self.draw_mire_body(j);
                }
            }
            4 => {
                if self.entrance_counter_inc_is(80) {
                    self.OverworldEntrance_AdvanceAndBoom();
                    let j = self.draw_mire_row(0x0522, 0x0e64);
                    let j = self.draw_mire_row(0x05a2, j);
                    self.draw_mire_body(j);
                }
            }
            5 => {
                if self.entrance_counter_inc_is(128) {
                    self.OverworldEntrance_PlayJingle();
                    self.set_ambient_sound_effect(5);
                }
            }
            _ => {}
        }
    }

    pub(super) fn Overworld_AnimateEntrance_TurtleRock(&mut self) {
        let x = if self.game_state.frame.frame_counter & 1 != 0 {
            (-1i16) as u16
        } else {
            1
        };
        self.set_bg1_x_offset(x);
        self.set_bg1_y_offset(x.wrapping_neg());

        match self.game_state.frame.subsubmodule {
            0 => {
                let screen = self.game_state.world.location.overworld_screen_index() as usize;
                self.set_overworld_event_bits(screen, 0x20);
                self.Dungeon_ApproachFixedColor_variable(0);
                self.turtle_rock_vram_common(0x10);
            }
            1 => self.turtle_rock_vram_common(0x14),
            2 => self.turtle_rock_vram_common(0x18),
            3 => self.turtle_rock_vram_common(0x1c),
            4 => {
                for i in 0..8 {
                    self.set_main_color_constant(0x58 + i, 0);
                    self.set_aux_color_constant(0x68 + i, 0);
                }
                self.copy_bg2_live_to_bg1_live();
                self.increment_subsubmodule();
                self.increment_cgram_update_flag();
            }
            5 => {
                self.OverworldEntrance_DrawManyTR();
                self.set_sub_screen_layers(1);
                self.set_color_window_selection(2);
                self.set_color_math_control(0x22);
                let end = self.game_state.display.vram_upload_cursor_usize();
                let mut off = 0usize;
                while off != end {
                    let v0 = self.vram_upload_buffer_word(off) | 0x10;
                    self.write_vram_upload_buffer_word(off, v0);
                    for word in [2usize, 3] {
                        let offset = off + word * 2;
                        if self.vram_upload_buffer_word(offset) == 0x08aa {
                            self.write_vram_upload_buffer_word(offset, 0x01e3);
                        }
                    }
                    off += 8;
                }
                self.clear_entrance_sequence_counter();
                self.increment_subsubmodule();
            }
            6 => {
                if self.game_state.frame.frame_counter & 1 == 0 {
                    if self.entrance_sequence_counter() & 7 == 0 {
                        self.PaletteFilter_RestoreAdditive(0xb0, 0xc0);
                        self.PaletteFilter_RestoreSubtractive(0xd0, 0xe0);
                        self.increment_cgram_update_flag();
                        self.set_sound_effect_2(2);
                    }
                    self.decrement_entrance_sequence_counter();
                    if self.entrance_sequence_counter() == 0 {
                        self.set_entrance_sequence_counter(0x30);
                        self.increment_subsubmodule();
                    }
                }
            }
            7 => {
                if self.game_state.frame.frame_counter & 1 == 0
                    && self.entrance_sequence_counter() & 7 == 0
                {
                    self.set_sound_effect_2(2);
                }
                self.decrement_entrance_sequence_counter();
                if self.entrance_sequence_counter() == 0 {
                    self.OverworldEntrance_DrawManyTR();
                    self.set_sub_screen_layers(0);
                    self.set_color_window_selection(0x82);
                    self.set_color_math_control(0x20);
                    self.increment_subsubmodule();
                    self.set_ambient_sound_effect(5);
                }
            }
            8 => self.OverworldEntrance_PlayJingle(),
            _ => {}
        }
    }

    fn turtle_rock_vram_common(&mut self, first: u16) {
        self.write_vram_upload_buffer_word(0, first);
        self.write_vram_upload_buffer_word(2, 0xfe47);
        self.write_vram_upload_buffer_word(4, 0x01e3);
        self.write_vram_upload_buffer_byte(6, 0xff);
        self.increment_subsubmodule();
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn Overworld_AnimateEntrance_GanonsTower(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 | 1 => {
                let screen = self.game_state.world.location.overworld_screen_index() as usize;
                self.set_overworld_event_bits(screen, 0x20);
                self.GanonTowerEntrance_Func1();
            }
            2 => {
                self.GanonTowerEntrance_Func1();
                if self.game_state.display.sub_screen_layers == 0 {
                    self.set_sub_screen_layers(1);
                    self.increment_entrance_sequence_counter();
                    if self.entrance_sequence_counter() == 3 {
                        self.clear_entrance_sequence_counter();
                        self.set_ambient_sound_effect(7);
                    } else {
                        self.set_subsubmodule(0);
                    }
                }
            }
            3 => self.ganon_tower_draw_after(
                48,
                &[
                    (0x045e, 0x0e88),
                    (0x0460, 0x0e89),
                    (0x04de, 0x0ea2),
                    (0x04e0, 0x0ea3),
                    (0x055e, 0x0e8a),
                    (0x0560, 0x0e8b),
                ],
            ),
            4 => self.ganon_tower_draw_after(
                48,
                &[
                    (0x045e, 0x0e8c),
                    (0x0460, 0x0e8d),
                    (0x04de, 0x0e8e),
                    (0x04e0, 0x0e8f),
                    (0x055e, 0x0e90),
                    (0x0560, 0x0e91),
                ],
            ),
            5 => self.ganon_tower_draw_after(
                52,
                &[
                    (0x045e, 0x0e92),
                    (0x0460, 0x0e93),
                    (0x04de, 0x0e94),
                    (0x04e0, 0x0e94),
                    (0x055e, 0x0e95),
                    (0x0560, 0x0e95),
                ],
            ),
            6 => self.ganon_tower_draw_after(
                32,
                &[
                    (0x045e, 0x0e96),
                    (0x0460, 0x0e97),
                    (0x04de, 0x0e98),
                    (0x04e0, 0x0e99),
                ],
            ),
            7 => self.ganon_tower_draw_after(32, &[(0x04de, 0x0e9a), (0x04e0, 0x0e9b)]),
            8 => self.ganon_tower_draw_after(
                32,
                &[
                    (0x04de, 0x0e9c),
                    (0x04e0, 0x0e9d),
                    (0x055e, 0x0e9e),
                    (0x0560, 0x0e9f),
                ],
            ),
            9 => self.ganon_tower_draw_after(32, &[(0x055e, 0x0e9a), (0x0560, 0x0e9b)]),
            10 => self.ganon_tower_draw_after(
                32,
                &[
                    (0x055e, 0x0e9c),
                    (0x0560, 0x0e9d),
                    (0x05de, 0x0ea0),
                    (0x05e0, 0x0ea1),
                ],
            ),
            11 => {
                if self.entrance_counter_inc_is(32) {
                    self.set_ambient_sound_effect(5);
                    self.OverworldEntrance_AdvanceAndBoom();
                    self.entrance_draw_tiles(&[(0x05de, 0x0e9a), (0x05e0, 0x0e9b)]);
                }
            }
            12 => {
                if self.entrance_counter_inc_is(72) {
                    self.OverworldEntrance_PlayJingle();
                    self.clear_entrance_sequence_counter();
                    self.set_music_control(13);
                    self.set_ambient_sound_effect(9);
                }
            }
            _ => {}
        }
    }

    fn ganon_tower_draw_after(&mut self, target: u8, entries: &[(u16, u16)]) {
        if self.entrance_counter_inc_is(target) {
            self.OverworldEntrance_AdvanceAndBoom();
            self.entrance_draw_tiles(entries);
        }
    }

    pub(super) fn DirToEnum(&self, mut dir: i32) -> i32 {
        let mut xx = 3;
        while dir & 1 == 0 {
            xx -= 1;
            dir >>= 1;
        }
        xx
    }

    pub(super) fn Overworld_GetSignText(&self, area: i32) -> u16 {
        let signs = self
            .asset_raw(110)
            .expect("Overworld_GetSignText missing kOverworld_SignText asset");
        read_word_from_slice(signs, area as usize * 2)
    }

    pub(super) fn GetOverworldSpritePtr(&self, area: i32) -> Vec<u8> {
        let base = if self.game_state.inventory.save_progress.progress_indicator() == 3 {
            2
        } else if self.game_state.inventory.save_progress.progress_indicator() == 2 {
            1
        } else {
            0
        };
        let offsets = self
            .asset_raw(159)
            .expect("GetOverworldSpritePtr missing kOverworldSpriteOffs asset");
        let offset = read_word_from_slice(offsets, (area as usize + base * 144) * 2) as usize;
        self.asset_raw(160)
            .expect("GetOverworldSpritePtr missing kOverworldSprites asset")[offset..]
            .to_vec()
    }

    pub(super) fn GetOverworldHibytes(&self, i: i32) -> Vec<u8> {
        self.asset_memblk(105, i as usize)
            .unwrap_or_else(|| panic!("GetOverworldHibytes missing block {i}"))
            .ptr
            .to_vec()
    }

    pub(super) fn GetOverworldLobytes(&self, i: i32) -> Vec<u8> {
        self.asset_memblk(106, i as usize)
            .unwrap_or_else(|| panic!("GetOverworldLobytes missing block {i}"))
            .ptr
            .to_vec()
    }

    pub(super) fn AdjustLinkBunnyStatus(&mut self) {
        if self.game_state.inventory.items.moon_pearl() != 0 {
            self.ForceNonbunnyStatus();
        }
    }

    pub(super) fn ForceNonbunnyStatus(&mut self) {
        self.follower_link_state_mut().set_handler_state(0);
        self.follower_link_state_mut().clear_temp_bunny_timer();
        self.follower_link_state_mut().clear_bunny_transform_flags();
        if self.game_state.enhanced_features.has(4) {
            self.follower_link_state_mut().clear_running();
        }
    }

    pub(super) fn RecoverPositionAfterDrowning(&mut self) {
        self.follower_link_state_mut()
            .restore_position_from_cached();
        let cached_bounds = (
            self.game_state.world.transient.cached_room_bounds_y_start(),
            self.game_state.world.transient.cached_room_bounds_y_end(),
            self.game_state.world.transient.cached_room_bounds_x_start(),
            self.game_state.world.transient.cached_room_bounds_x_end(),
        );
        self.room_bounds_mut()
            .restore_from_cached_safe_return_bounds(
                cached_bounds.0,
                cached_bounds.1,
                cached_bounds.2,
                cached_bounds.3,
            );

        self.restore_scroll_targets_from_cached();

        if self.game_state.world.location.is_indoors() {
            self.restore_camera_y_from_cached_indoor();
            self.restore_camera_x_from_cached_indoor();
        }
        self.restore_quadrant_fullsize_from_cached();
        self.follower_link_state_mut()
            .restore_quadrants_from_cached();
        if self.game_state.world.location.is_outdoors() {
            self.update_camera_hi_outdoor();
        }

        self.follower_link_state_mut().restore_facing_from_cached();
        self.follower_link_state_mut()
            .restore_lower_level_state_from_cached();
        let doorway_state = self
            .game_state
            .world
            .transient
            .is_standing_in_doorway_cached();
        self.follower_link_state_mut()
            .set_doorway_state(doorway_state);
        self.dungeon_stair_movement_mut().restore_cached_floor();
        self.follower_link_state_mut().set_visibility_status(0);
        self.follower_link_state_mut().set_blink_countdown(0x90);
        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        self.link_reset_state_after_damaging_pit();
        self.follower_state_mut().set_appearance_none_flag(0);
        self.follower_initialize();
        self.dungeon_environment_mut()
            .clear_water_puzzle_state_changed();
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
        self.clear_screen_transition();
        self.set_submodule(0);
        if self.game_state.inventory.player_resources.current_health() == 0 {
            let main_screen_layers = self.game_state.display.main_screen_layers;
            let sub_screen_layers = self.game_state.display.sub_screen_layers;
            self.set_world_transient_map_backup_main_layer(main_screen_layers);
            self.set_world_transient_map_backup_subscreen_layer(sub_screen_layers);
            self.save_main_module_for_menu();
            self.set_main_module(18);
            self.set_submodule(1);
            self.follower_link_state_mut().clear_blink_countdown();
        }
    }

    pub(super) fn Module09_2A_RecoverFromDrowning(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => self.Module09_2A_00_ScrollToLand(),
            _ => self.RecoverPositionAfterDrowning(),
        }
    }

    pub(super) fn Module09_2A_00_ScrollToLand(&mut self) {
        let cached_x = self.game_state.player.follower_link.cached_x();
        let cached_y = self.game_state.player.follower_link.cached_y();

        let mut x = self.game_state.player.follower_link.x();
        let mut xd = 0u16;
        if x != cached_x {
            let d = if x > cached_x { -1 } else { 1 };
            x = x.wrapping_add_signed(d);
            if x != cached_x {
                x = x.wrapping_add_signed(d);
            }
            xd = x.wrapping_sub(self.game_state.player.follower_link.x());
            self.follower_link_state_mut().set_x(x);
        }

        let mut y = self.game_state.player.follower_link.y();
        let mut yd = 0u16;
        if y != cached_y {
            let d = if y > cached_y { -1 } else { 1 };
            y = y.wrapping_add_signed(d);
            if y != cached_y {
                y = y.wrapping_add_signed(d);
            }
            yd = y.wrapping_sub(self.game_state.player.follower_link.y());
            self.follower_link_state_mut().set_y(y);
        }

        self.follower_link_state_mut()
            .set_movement_velocity_from_delta(xd, yd);
        if y == cached_y && x == cached_x {
            self.increment_subsubmodule();
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.sprite_battle_mut().clear_damaging_enemies_timer();
        }
        self.Overworld_OperateCameraScroll();
        if self.has_screen_transition_direction_bits() {
            self.OverworldHandleMapScroll();
        }
    }

    pub(super) fn TakeDamageFromPit(&mut self) {
        self.replay_trace_submodule("take_damage_from_pit-entry");
        self.follower_link_state_mut().set_visibility_status(12);
        let submodule = if self.game_state.world.location.is_indoors() {
            20
        } else {
            42
        };
        self.set_submodule(submodule);
        let health = self.player_resources_mut().decrement_current_health_by(8);
        if health >= 0xa8 {
            self.player_resources_mut().set_current_health(0);
        }
        self.replay_trace_submodule("take_damage_from_pit-exit");
    }

    pub(super) fn Overworld_GetPitDestination(&mut self) {
        let x = self.game_state.player.follower_link.x() & !7;
        let y = self.game_state.player.follower_link.y() & !7;
        let ws = &self.game_state.world.scroll;
        let pos = ((y.wrapping_sub(ws.overworld_offset_base_y()) & ws.overworld_offset_mask_y())
            << 3)
            .wrapping_add(
                ((x >> 3).wrapping_sub(ws.overworld_offset_base_x()))
                    & ws.overworld_offset_mask_x(),
            );

        let fall_hole_area = self
            .asset_raw(127)
            .expect("Overworld_GetPitDestination missing kFallHole_Area asset");
        let fall_hole_pos = self
            .asset_raw(128)
            .expect("Overworld_GetPitDestination missing kFallHole_Pos asset");
        let fall_hole_entrances = self
            .asset_raw(129)
            .expect("Overworld_GetPitDestination missing kFallHole_Entrances asset");
        for i in (0..=18).rev() {
            if read_word_from_slice(fall_hole_pos, i * 2) == pos
                && read_word_from_slice(fall_hole_area, i * 2)
                    == self.game_state.world.region.overworld_area_index()
            {
                let entrance = fall_hole_entrances[i];
                self.set_which_entrance_byte(entrance);
                self.set_overworld_hole_scan_step(0);
                return;
            }
        }

        self.save_progress_mut().set_dark_world_state(0);
        self.set_which_entrance_byte(130);
        self.set_overworld_hole_scan_step(0);
    }

    pub(super) fn Overworld_ToolAndTileInteraction(&mut self, x: u16, y: u16) -> u16 {
        self.set_overworld_hole_tilemap_pos(0);
        self.tile_detect_position_mut().set_interacting_tile(0);

        let ws = &self.game_state.world.scroll;
        let pos = ((y.wrapping_sub(ws.overworld_offset_base_y()) & ws.overworld_offset_mask_y())
            .wrapping_mul(8))
        .wrapping_add(x.wrapping_sub(ws.overworld_offset_base_x()) & ws.overworld_offset_mask_x());
        let attr = self
            .game_state
            .dungeon
            .room_tilemaps
            .bg2_tile_by_byte_pos(pos);
        let mut yv = 0u16;
        let mut check_secret = false;
        let mut memoize = false;

        let item_in_hand = self.game_state.player.follower_link.item_in_hand();
        let position_mode = self.game_state.player.follower_link.position_mode();
        if item_in_hand & 2 == 0 {
            if item_in_hand & 0x40 == 0 {
                if matches!(
                    attr,
                    0x034 | 0x071 | 0x035 | 0x10d | 0x10f | 0x0e1 | 0x0e2 | 0x0da | 0x0f8 | 0x10e
                ) {
                    if position_mode != 1 {
                        return attr;
                    }
                    if self.game_state.world.location.overworld_screen_index() == 0x2a
                        && pos == 0x0492
                    {
                        self.set_overworld_hole_tilemap_pos(pos);
                    }
                    yv = 0x0dc9;
                    check_secret = true;
                    memoize = true;
                } else if attr == 0x037e {
                    if position_mode == 1 {
                        return attr;
                    }
                    self.tile_detect_position_mut()
                        .set_interaction_scratch_y(x.wrapping_mul(8).wrapping_sub(8));
                    self.tile_detect_position_mut()
                        .set_interaction_scratch_x(y.wrapping_sub(8) & !7);
                    self.tile_detect_position_mut().set_interacting_tile(3);
                    yv = 0x0dc5;
                    check_secret = true;
                    memoize = true;
                }
            }

            if !memoize {
                if attr == 0x036 || attr == 0x072a {
                    if position_mode != 1 {
                        self.tile_detect_position_mut()
                            .set_interaction_scratch_y((x & !1).wrapping_mul(8));
                        self.tile_detect_position_mut()
                            .set_interaction_scratch_x(y & !0x0f);
                        let terrain = if attr == 0x036 { 2 } else { 4 };
                        self.tile_detect_position_mut()
                            .set_interacting_tile(terrain);
                        yv = if attr == 0x072a { 0x0dc8 } else { 0x0dc7 };
                        check_secret = true;
                        memoize = true;
                    }
                } else {
                    return attr;
                }
            }
        } else if attr == 0x021b {
            self.set_sound_effect_1(17);
            self.HandlePegPuzzles(pos);
            yv = 0x0dcb;
            memoize = true;
        } else {
            self.Overworld_PickHammerSfx(attr);
            return attr;
        }

        if memoize {
            if check_secret {
                let secret = self.overworld_reveal_secret(pos);
                if secret != 0 {
                    yv = secret;
                }
            }
            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile_by_byte_pos(pos, yv);
            self.Overworld_Memorize_Map16_Change(pos, yv);
            self.overworld_draw_map16(pos, yv);
            self.set_bg_vram_load_mode(1);
        }

        let map8_index = attr as usize * 4 + (((y & 8) >> 2) | (x & 1)) as usize;
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("Overworld_ToolAndTileInteraction missing kMap16ToMap8 asset");
        let tile_attrs = self
            .asset_raw(163)
            .expect("Overworld_ToolAndTileInteraction missing kMap8DataToTileAttr asset");
        let map8 = read_word_from_slice(map16_to_map8, map8_index * 2);
        let tile_attr = tile_attrs[(map8 & 0x01ff) as usize] as u16;
        let terrain = self.game_state.player.tile_detection.interacting_tile();
        if terrain != 0 {
            let sx = self
                .game_state
                .player
                .tile_detection
                .interaction_scratch_y();
            let sy = self
                .game_state
                .player
                .tile_detection
                .interaction_scratch_x();
            self.sprite_spawn_immediately_smashed_terrain(terrain as u8, sx, sy);
            self.ancilla_add_bush_poof(sx, sy);
        }
        tile_attr
    }

    pub(super) fn Overworld_PickHammerSfx(&mut self, a: u16) {
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("Overworld_PickHammerSfx missing kMap16ToMap8 asset");
        let tile_attrs = self
            .asset_raw(163)
            .expect("Overworld_PickHammerSfx missing kMap8DataToTileAttr asset");
        let map8 = read_word_from_slice(map16_to_map8, a as usize * 8);
        let attr = tile_attrs[(map8 & 0x01ff) as usize];
        let sound_effect_1 = if attr < 0x50 {
            return;
        } else if attr < 0x52 {
            26
        } else if attr < 0x54 {
            17
        } else if attr < 0x58 {
            5
        } else {
            return;
        };
        self.set_sound_effect_1(sound_effect_1);
    }

    pub(super) fn Overworld_HandleLiftableTiles(&mut self, pt_arg: &mut Point16U) -> u8 {
        let pos = self.overworld_get_link_map16_coords(pt_arg);
        let pt = *pt_arg;
        let a = self
            .game_state
            .dungeon
            .room_tilemaps
            .bg2_tile_by_byte_pos(pos);
        if matches!(a, 0x36d | 0x23b) {
            return self.smash_rock_pile_from_lift(a, pos, 0, pt);
        }
        if matches!(a, 0x36e | 0x23c) {
            return self.smash_rock_pile_from_lift(a, pos, 1, pt);
        }
        if matches!(a, 0x374 | 0x23d) {
            return self.smash_rock_pile_from_lift(a, pos, 2, pt);
        }
        if matches!(a, 0x375 | 0x23e) {
            return self.smash_rock_pile_from_lift(a, pos, 3, pt);
        }

        let y = match a {
            0x36 => Some(0x0dc7),
            0x72a => Some(0x0dc8),
            0x20f | 0x239 => Some(0x0dca),
            0x101 => Some(0x0dc6),
            _ => None,
        };
        if let Some(y) = y {
            return self.overworld_lifting_small_obj(a, pos, y, pt);
        }

        let t =
            a as usize * 4 + if pt.x & 8 != 0 { 2 } else { 0 } + if pt.y & 8 != 0 { 1 } else { 0 };
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("Overworld_HandleLiftableTiles missing kMap16ToMap8 asset");
        let tile_attrs = self
            .asset_raw(163)
            .expect("Overworld_HandleLiftableTiles missing kMap8DataToTileAttr asset");
        let map8 = read_word_from_slice(map16_to_map8, t * 2);
        tile_attrs[(map8 & 0x01ff) as usize]
    }

    pub(super) fn Module10_00_OpenIris(&mut self) {
        self.Spotlight_open();
        self.increment_submodule();
    }

    pub(super) fn OverworldOverlay_HandleRain(&mut self) {
        if (self.game_state.world.location.overworld_screen_index() != 0x70
            && self.game_state.inventory.save_progress.progress_indicator() >= 2)
            || (self.game_state.world.overworld.event_info.event_info(0x70) & 0x20) != 0
        {
            return;
        }

        match self.game_state.frame.frame_counter {
            3 | 88 => self.set_color_math_control(0x32),
            5 | 44 | 90 => self.set_color_math_control(0x72),
            36 => {
                self.set_sound_effect_1(54);
                self.set_color_math_control(0x32);
            }
            _ => {}
        }
        if self.game_state.frame.frame_counter & 3 != 0 {
            return;
        }
        let i = self.increment_move_overlay_ctr();
        let bg1x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg1_h_copy2()
            .wrapping_add((OVERWORLD_RAIN_X_OFFSETS[i as usize] as u16) << 8);
        let bg1y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg1_v_copy2()
            .wrapping_add((OVERWORLD_RAIN_Y_OFFSETS[i as usize] as u16) << 8);
        self.set_bg1_x(bg1x);
        self.set_bg1_y(bg1y);
    }

    pub(super) fn Overworld_ResetMosaicDown(&mut self) {
        if self.game_state.display.palette_filter.countdown() & 1 != 0 {
            self.decrement_mosaic_level_by(0x10);
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(7);
    }

    pub(super) fn Overworld_Func1D(&mut self) {
        // Overworld_Func1D is an assert-only dispatch slot in the C port.
        panic!("Overworld_Func1D reached");
    }

    pub(super) fn Overworld_Func1E(&mut self) {
        // Overworld_Func1E is an assert-only dispatch slot in the C port.
        panic!("Overworld_Func1E reached");
    }

    pub(super) fn Overworld_FinishTransGfx(&mut self) {
        self.set_core_update_disable_flag(10);
        self.set_pending_nmi_subroutine(10);
        self.increment_submodule();
    }

    pub(super) fn Overworld_Func22(&mut self) {
        self.increment_screen_brightness();
        if self.game_state.display.screen_brightness == 15 {
            self.set_submodule(0);
            self.set_subsubmodule(0);
        }
    }

    pub(super) fn Overworld_Func18(&mut self) {
        self.follower_link_state_mut().clear_swim_fast_state();
        let module = self.game_state.frame.main_module;
        let submodule = self.game_state.frame.submodule;
        self.Overworld_EnterSpecialArea();
        self.Overworld_LoadOverlays();
        self.set_submodule(submodule.wrapping_add(1));
        self.set_main_module(module);
    }

    pub(super) fn Overworld_Func19(&mut self) {
        let module = self.game_state.frame.main_module;
        let submodule = self.game_state.frame.submodule;
        self.Module08_02_LoadAndAdvance();
        self.set_submodule(submodule.wrapping_add(1));
        self.set_main_module(module);
    }

    pub(super) fn Overworld_Func2B(&mut self) {
        self.Palette_AnimGetMasterSword();
    }

    pub(super) fn Overworld_WeathervaneExplosion(&mut self) {}

    pub(super) fn InitializeMirrorHDMA(&mut self) {
        self.clear_hdma_enable_mask();

        self.mirror_warp_scratch_mut().initialize_hdma_wave_state();

        self.hdma_setup(0xf2fb, 0xf2fb, 0x42, 0x0d, 0x0f, 0);

        let value = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        for i in 0..240 {
            self.set_spotlight_hdma_table_dynamic_entry(i, value);
        }
        self.set_hdma_enable_mask(0xc0);
    }

    pub(super) fn MirrorWarp_BuildWavingHDMATable(&mut self) {
        self.MirrorWarp_RunAnimationSubmodules();
        if self.game_state.frame.frame_counter & 1 != 0 {
            return;
        }

        let mut y = 240usize - 8;
        loop {
            let value = self.spotlight_hdma_table_dynamic_entry(y - 8);
            for off in [0usize, 2, 4, 6] {
                self.set_spotlight_hdma_table_dynamic_entry(y + off, value);
            }
            y -= 8;
            if y == 0 {
                break;
            }
        }

        let mirror = &self.game_state.inventory.mirror_warp;
        let target = mirror.target_offset();
        let mut t = mirror.displacement().wrapping_add(mirror.velocity_delta());
        if !sign16(t.wrapping_sub(target) ^ target) {
            t = target;
            let mut mirror = self.mirror_warp_scratch_mut();
            mirror.reset_wave_and_subpixel();
            mirror.toggle_target_index();
        }
        self.mirror_warp_scratch_mut().set_displacement(t);
        t = t.wrapping_add(self.game_state.inventory.mirror_warp.subpixel());
        self.mirror_warp_scratch_mut().set_subpixel_low_from(t);
        if sign16(t) {
            t |= 0x00ff;
        } else {
            t &= !0x00ff;
        }
        t = self
            .game_state
            .inventory
            .mirror_warp
            .wave_offset()
            .wrapping_add(t.swap_bytes());
        self.mirror_warp_scratch_mut().set_wave_offset(t);
        if self.game_state.display.palette_filter.countdown() >= 0x30 && (t & !7) == 0 {
            self.mirror_warp_scratch_mut()
                .shrink_target_offsets_for_dewaving();
            self.increment_subsubmodule();
            t = 0;
        }
        let value = t.wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        for off in [0usize, 2, 4, 6] {
            self.set_spotlight_hdma_table_dynamic_entry(off, value);
        }
    }

    pub(super) fn MirrorWarp_BuildDewavingHDMATable(&mut self) {
        self.MirrorWarp_RunAnimationSubmodules();
        if self.game_state.frame.frame_counter & 1 != 0 {
            return;
        }

        let mut y = 240usize - 8;
        loop {
            let value = self.spotlight_hdma_table_dynamic_entry(y - 8);
            for off in [0usize, 2, 4, 6] {
                self.set_spotlight_hdma_table_dynamic_entry(y + off, value);
            }
            y -= 8;
            if y == 0 {
                break;
            }
        }

        let t = self.spotlight_hdma_table_dynamic_entry(0x0c0)
            | self.spotlight_hdma_table_dynamic_entry(0x0c8)
            | self.spotlight_hdma_table_dynamic_entry(0x0d0)
            | self.spotlight_hdma_table_dynamic_entry(0x0d8);
        if t == self.game_state.display.ppu_scroll_copy.bg2_h_copy2() {
            self.clear_hdma_enable_mask();
            self.increment_subsubmodule();
            self.Overworld_SetFixedColAndScroll();
            if self.game_state.world.location.overworld_screen_index() & 0x3f != 0x1b {
                let bg2x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
                let bg2y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
                self.set_bg1_x(bg2x);
                self.set_bg1_h_copy(bg2x);
                self.set_bg2_h_copy(bg2x);
                self.set_bg1_y(bg2y);
                self.set_bg1_v_copy(bg2y);
                self.set_bg2_v_copy(bg2y);
            }
        }
    }

    pub(super) fn MirrorWarp_FinalizeAndLoadDestination(&mut self) {
        self.hdma_setup(0, 0xf2fb, 0x41, 0, 0x26, 0);
        self.IrisSpotlight_ResetTable();
        self.set_countdown_word(0);
        self.set_darkening_or_lightening_screen_word(0);
        self.ReloadPreviouslyLoadedSheets();
        self.Overworld_SetSongList();
        self.set_hdma_enable_mask(0x80);

        let screen = self.game_state.world.location.overworld_screen_index() as usize;
        let music = self.overworld_config_table().music(screen);
        self.set_music_control(music & 0x0f);
        self.set_ambient_sound_effect(music >> 4);
        if self.game_state.world.location.overworld_screen_index() >= 0x40
            && self.game_state.inventory.items.moon_pearl() == 0
        {
            self.set_music_control(4);
        }

        self.save_submodule_for_menu();
        self.set_submodule(0);
        self.set_subsubmodule(0);
        self.set_overworld_map_state(0);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn Module09_MirrorWarp(&mut self) {
        self.increment_core_update_disable_flag();
        match self.game_state.frame.subsubmodule {
            0 => {
                if self.game_state.world.location.overworld_screen_index() >= 0x80 {
                    self.set_submodule(0);
                    self.set_subsubmodule(0);
                    self.set_overworld_map_state(0);
                    return;
                }
                self.set_music_control(8);
                self.set_flag_overworld_area_changed(8);
                self.follower_link_state_mut().set_blink_countdown(0x90);
                self.InitializeMirrorHDMA();
                self.save_progress_mut().xor_dark_world_state(0x40);
                self.set_overworld_peg_puzzle_progress(0);
                let screen = (self.game_state.world.location.overworld_screen_index() & 0x3f)
                    | self.game_state.inventory.save_progress.dark_world_state();
                self.set_overworld_screen(screen);
                self.set_overworld_area_index(screen);
                self.set_overworld_map_state(0);
                self.PaletteFilter_InitializeWhiteFilter();
                self.Overworld_LoadGFXAndScreenSize();
                self.increment_subsubmodule();
            }
            1 => {
                self.increment_subsubmodule();
                self.set_hdma_enable_mask(0xc0);
                self.MirrorWarp_BuildWavingHDMATable();
            }
            2 => self.MirrorWarp_BuildWavingHDMATable(),
            3 => self.MirrorWarp_BuildDewavingHDMATable(),
            _ => self.MirrorWarp_FinalizeAndLoadDestination(),
        }
    }

    fn set_small_overworld_mirror_map_position(&mut self) {
        self.set_overworld_map16_src_off(0x0390);
        self.set_overworld_map16_y_unit((0x0390u16.wrapping_sub(0x0400) & 0x0f80) >> 7);
        self.set_overworld_map16_dst_off((0x0390u16.wrapping_sub(0x0010) & 0x003e) >> 1);
    }

    pub(super) fn Overworld_DrawScreenAtCurrentMirrorPosition(&mut self) {
        let bak_src_off = self.overworld_map16_src_off();
        let bak_dst_off = self.overworld_map16_dst_off();
        let bak_y_unit = self.overworld_map16_y_unit();
        if self.overworld_map_is_small() {
            self.set_small_overworld_mirror_map_position();
        }
        self.Overworld_DrawQuadrantsAndOverlays();
        if self.game_state.frame.submodule == 44 {
            self.MirrorBonk_RecoverChangedTiles();
        }
        self.set_overworld_map16_y_unit(bak_y_unit);
        self.set_overworld_map16_dst_off(bak_dst_off);
        self.set_overworld_map16_src_off(bak_src_off);
    }

    pub(super) fn MirrorWarp_LoadSpritesAndColors(&mut self) {
        self.follower_link_state_mut().set_blink_countdown(0x90);
        let bak_src_off = self.overworld_map16_src_off();
        let bak_dst_off = self.overworld_map16_dst_off();
        let bak_y_unit = self.overworld_map16_y_unit();
        if self.overworld_map_is_small() {
            self.set_small_overworld_mirror_map_position();
        }
        self.Map16ToMap8(OverworldMap16SourcePage::Main, 0);
        self.set_overworld_map16_y_unit(bak_y_unit);
        self.set_overworld_map16_dst_off(bak_dst_off);
        self.set_overworld_map16_src_off(bak_src_off);

        self.OverworldLoadScreensPaletteSet();
        let sc = self.game_state.world.location.overworld_screen_index() as usize;
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(sc as u8),
            self.overworld_config_table().sprite_palette(sc),
        );
        self.Palette_SpecialOw();
        self.Overworld_SetFixedColAndScroll();
        if self.game_state.world.location.overworld_screen_index() == 0x1b
            || self.game_state.world.location.overworld_screen_index() == 0x5b
        {
            self.set_sub_screen_layers(1);
        }
        for i in 0..16 * 6 {
            self.set_main_color_constant(32 + i, 0x7fff);
        }
        self.set_main_color_constant(0, 0x7fff);
        if u16::from(self.game_state.world.location.overworld_screen_index()) == 0x5b {
            self.set_main_color_constant(0, 0);
            self.set_main_color_constant(32, 0);
        }
        self.sprite_reset_all();
        self.sprite_reload_all_overworld();
        self.link_item_reset_from_overworld_things();
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        self.follower_link_state_mut().set_handler_state(20);
        if self.game_state.world.location.overworld_screen_index() & 0x40 == 0 {
            self.sprite_initialize_mirror_portal();
        }
    }

    pub(super) fn Module09_2E_Whirlpool(&mut self) {
        self.increment_core_update_disable_flag();
        match self.game_state.frame.subsubmodule {
            0 => {
                self.set_sound_effect_1(0x34);
                self.set_ambient_sound_effect(5);
                self.set_overworld_map_state(0);
                self.set_countdown(0);
                self.increment_subsubmodule();
            }
            1 => self.PaletteFilter_WhirlpoolBlue(),
            2 => self.PaletteFilter_IsolateWhirlpoolBlue(),
            3 => {
                self.set_fixed_color_blue(0x9f);
                self.clear_overworld_aux_or_main_offset();
                self.set_hud_palette(0);
                self.FindPartnerWhirlpoolExit();
                self.dungeon_room_load_mut().set_draw_width_indicator(0);
                self.Overworld_LoadOverlays2();
                self.decrement_submodule();
                self.set_pending_nmi_subroutine(12);
                self.clear_cgram_update_flag();
                self.set_fixed_color_blue(0x80);
                self.set_screen_brightness(0x0f);
                self.increment_core_update_disable_flag();
                self.increment_subsubmodule();
            }
            4 | 6 => {
                self.set_pending_nmi_subroutine(13);
                self.increment_core_update_disable_flag();
                self.increment_subsubmodule();
            }
            5 => {
                self.Overworld_LoadOverlayAndMap();
                self.set_pending_nmi_subroutine(12);
                self.set_screen_brightness(0x0f);
                self.increment_core_update_disable_flag();
                self.increment_subsubmodule();
            }
            7 => {
                self.Module09_LoadAuxGFX();
                self.decrement_submodule();
                self.increment_subsubmodule();
            }
            8 => {
                self.Overworld_FinishTransGfx();
                self.set_screen_brightness(0x0f);
                self.increment_core_update_disable_flag();
                self.decrement_submodule();
                self.increment_subsubmodule();
            }
            9 => {
                self.clear_overworld_aux_or_main_offset();
                self.Palette_Load_SpriteMain();
                self.Palette_Load_SpriteEnvironment();
                self.Palette_Load_Sp0L();
                self.Palette_Load_HUD();
                self.Palette_Load_OWBGMain();
                let sc = self.game_state.world.location.overworld_screen_index() as usize;
                self.Overworld_LoadPalettes(
                    self.GetOverworldBgPalette(sc as u8),
                    self.overworld_config_table().sprite_palette(sc),
                );
                self.Palette_SetOwBgColor();
                self.Overworld_SetFixedColAndScroll();
                self.LoadNewSpriteGFXSet();
                self.set_fixed_color_blue(0x80);
                self.set_screen_brightness(0x0f);
                self.increment_core_update_disable_flag();
                self.increment_subsubmodule();
            }
            10 => {
                self.PaletteFilter_WhirlpoolRestoreRedGreen();
                if self.game_state.display.palette_filter.countdown() != 0 {
                    self.PaletteFilter_WhirlpoolRestoreRedGreen();
                }
            }
            11 => {
                self.Graphics_IncrementalVRAMUpload();
                self.PaletteFilter_WhirlpoolRestoreBlue();
            }
            12 => {
                self.follower_link_state_mut().set_blink_countdown(144);
                self.ReloadPreviouslyLoadedSheets();
                self.set_hdma_enable_mask(0x80);
                let music = self
                    .overworld_config_table()
                    .music(self.game_state.world.location.overworld_screen_index() as usize);
                self.set_ambient_sound_effect(music >> 4);
                let music_control =
                    if self.game_state.inventory.save_progress.dark_world_state() != 0 {
                        9
                    } else {
                        2
                    };
                self.set_music_control(music_control);
                self.set_submodule(0);
                self.set_subsubmodule(0);
                self.set_overworld_map_state(0);
                self.clear_core_update_disable_flag();
            }
            _ => {}
        }
    }

    pub(super) fn Spotlight_ConfigureTableAndControl(&mut self) {
        let (caller_interrupted, _) = self.spotlight_configure_table_and_control(false);
        debug_assert!(!caller_interrupted);
    }

    pub(super) fn spotlight_configure_table_and_control(
        &mut self,
        interrupt_module0f_goal_caller: bool,
    ) -> (bool, bool) {
        let entry_main_module = self.game_state.frame.main_module;
        let reached_spotlight_goal = self.IrisSpotlight_ConfigureTable();
        let caller_interrupted = self.complete_spotlight_configure_table_and_control_after_table(
            entry_main_module,
            interrupt_module0f_goal_caller,
        );
        (caller_interrupted, reached_spotlight_goal)
    }

    fn complete_spotlight_configure_table_and_control_after_table(
        &mut self,
        entry_main_module: u8,
        interrupt_module0f_goal_caller: bool,
    ) -> bool {
        self.deactivate_nmi_thread();
        self.clear_pending_polyhedral_update();
        if self.game_state.frame.submodule != 0 {
            return false;
        }
        if interrupt_module0f_goal_caller
            && self.rom_startup_timing()
            && entry_main_module == 15
            && self.game_state.frame.main_module == 6
        {
            // IrisSpotlight_ConfigureTable has completed its goal transition,
            // including INIDISP_copy = 0x80. The ROM is interrupted before
            // Spotlight_ConfigureTableAndControl restores Link's coordinate
            // or enters OpenSpotlight_Next2, so retain that exact caller stack.
            return true;
        }
        self.complete_spotlight_configure_table_and_control_caller();
        false
    }

    fn complete_spotlight_configure_table_and_control_caller(&mut self) {
        if self.game_state.frame.main_module == 6 {
            self.follower_link_state_mut()
                .restore_y_from_overworld_exit();
        }
        self.OpenSpotlight_Next2();
    }

    pub(super) fn OpenSpotlight_Next2(&mut self) {
        if self.game_state.frame.main_module != 9 {
            self.EnableForceBlank();
            self.link_item_reset_from_overworld_things();
        }

        if self.game_state.frame.main_module == 9 {
            if self.game_state.world.location.dungeon_room() != 0x20 {
                let submodule = if self.game_state.player.follower_link.has_facing() {
                    0x0a
                } else {
                    0x0b
                };
                self.set_submodule(submodule);
            }
            self.set_overworld_transition_countdown(16);
            let big_rock = self
                .game_state
                .dungeon
                .object_tracking
                .big_rock_starting_address();
            if (self.ow_entrance_value() as u8 | big_rock as u8) != 0 && big_rock & 0xff00 != 0 {
                self.dungeon_doors_mut()
                    .set_door_open_counter_low(if big_rock & 0x8000 != 0 { 0x18 } else { 0 });
                self.dungeon_object_tracking_mut()
                    .set_big_rock_starting_address(big_rock & 0x7fff);
                self.set_door_animation_step(0);
                self.set_submodule(9);
                self.set_subsubmodule(0);
                self.set_sound_effect_2(21);
            }
        }

        self.set_bg12_window_selection(0);
        self.set_bg34_window_selection(0);
        self.set_object_color_window_selection(0);
        self.set_main_screen_window_layers(0);
        self.set_sub_screen_window_layers(0);
        self.follower_link_state_mut().clear_force_hold_sword_up();

        match self.game_state.world.location.overworld_screen_index() {
            3 | 5 | 7 => {
                self.set_fixed_color_red(0x26);
                self.set_fixed_color_green(0x4c);
                self.set_fixed_color_blue(0x8c);
            }
            0x43 | 0x45 | 0x47 => {
                self.set_fixed_color_red(0x26);
                self.set_fixed_color_green(0x4a);
                self.set_fixed_color_blue(0x87);
            }
            _ => {}
        }
    }

    pub(super) fn Module10_SpotlightOpen(&mut self) {
        let live_table_progress = self.take_original_timing_spotlight_table_build_progress();
        let live_goal_caller_returned =
            self.take_original_timing_overworld_spotlight_goal_caller_returned();
        if live_goal_caller_returned {
            assert_eq!(
                (
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule
                ),
                (0x10, 1),
                "overworld spotlight goal return belongs to recurring Module10",
            );
            assert!(
                live_table_progress.is_none(),
                "a completed overworld spotlight goal cannot also expose stale table progress",
            );
        }
        let cpu_plan = if self.rom_startup_timing()
            && !matches!(
                self.original_timing_owner(),
                crate::zelda_rtl::OriginalTimingOwner::Live
            ) {
            let (entry_earliest, entry_latest) = if self.game_state.frame.submodule == 0 {
                (
                    OVERWORLD_SPOTLIGHT_CPU_ENTRY_EARLIEST,
                    OVERWORLD_SPOTLIGHT_CPU_ENTRY_LATEST,
                )
            } else {
                self.overworld_spotlight_cpu_entry_envelope
                    .expect("recurring Module10 entry requires the preceding ROM CPU raster")
            };
            let plan = overworld_spotlight_cpu_plan(self, entry_earliest, entry_latest);
            self.overworld_spotlight_cpu_entry_envelope =
                plan.and_then(|plan| plan.next_entry_earliest.zip(plan.next_entry_latest));
            plan
        } else {
            // A continuous Live owner publishes the completed window raster
            // and main-loop progress directly. Re-running the old isolated
            // CPU/raster envelope here would make translated gameplay depend
            // on a guessed entry range and can disagree about which side of
            // one NMI a C statement occupied. In Live mode the native C
            // iteration remains the shadow implementation while the typed
            // presentation receipt owns the outgoing scanout.
            None
        };
        let iteration = SpotlightIteration::opening_from_rom_cpu_plan(cpu_plan);
        self.sprite_main();
        if let Some(receipt) = live_table_progress {
            let phase = if self.game_state.frame.submodule == 0 {
                self.spotlight_internal_before_table(0, 2);
                OverworldSpotlightBuildPhase::Entry
            } else {
                OverworldSpotlightBuildPhase::Recurring
            };
            let table_build =
                self.begin_iris_spotlight_configure_table_at_progress(receipt.progress);
            self.schedule_overworld_spotlight_build(table_build, phase, false, iteration);
            return;
        }
        if let Some(plan) = cpu_plan.filter(|plan| plan.interrupted_during_table_build_or_copy()) {
            let phase = if self.game_state.frame.submodule == 0 {
                self.spotlight_internal_before_table(0, 2);
                OverworldSpotlightBuildPhase::Entry
            } else {
                OverworldSpotlightBuildPhase::Recurring
            };
            let table_build = self.begin_iris_spotlight_configure_table(plan.iterations_before_nmi);
            let projection_completed = plan.interrupted_during_table_copy();
            if projection_completed {
                self.complete_iris_spotlight_table_projection(table_build);
            }
            self.schedule_overworld_spotlight_build(
                table_build,
                phase,
                projection_completed,
                iteration,
            );
            return;
        }
        if self.game_state.frame.submodule == 0 {
            self.Module10_00_OpenIris();
        } else {
            let (caller_interrupted, _) = self.spotlight_configure_table_and_control(false);
            debug_assert!(!caller_interrupted);
        }
        let authoritative_link_oam_interruption =
            self.take_original_timing_main_loop_interruption(crate::MainLoopInterruption::LinkOam);
        self.link_oam_main();
        if live_goal_caller_returned {
            assert_ne!(
                self.game_state.frame.main_module, 0x10,
                "authoritative overworld spotlight goal returned before the native C shadow",
            );
        }
        if cpu_plan.is_some() {
            assert!(
                !authoritative_link_oam_interruption,
                "the isolated overworld spotlight CPU plan and Live LinkOam receipt cannot both own one interruption",
            );
            self.schedule_overworld_spotlight_link_oam(iteration);
        } else {
            // LinkOam_Main is an atomic translated leaf. A replaceable timing
            // authority may stop the enclosing C call at that semantic phase;
            // consuming the receipt here proves the leaf ran once while the
            // existing FinishSpotlightIteration continuation retains only the
            // shared ZeldaRunGameLoop suffix.
            self.schedule_spotlight_iteration_return(iteration);
        }
    }

    pub(super) fn complete_overworld_spotlight_build(
        &mut self,
        table_build: SpotlightTableBuildContinuation,
        phase: OverworldSpotlightBuildPhase,
        projection_completed: bool,
    ) {
        let entry_main_module = self.game_state.frame.main_module;
        if projection_completed {
            self.complete_iris_spotlight_configure_table_after_projection();
        } else {
            self.complete_iris_spotlight_configure_table(table_build);
        }
        match phase {
            OverworldSpotlightBuildPhase::Entry => {
                self.spotlight_internal_after_table_during_active_field();
                self.increment_submodule();
            }
            OverworldSpotlightBuildPhase::Recurring => {
                let caller_interrupted = self
                    .complete_spotlight_configure_table_and_control_after_table(
                        entry_main_module,
                        false,
                    );
                debug_assert!(!caller_interrupted);
            }
        }
        self.link_oam_main();
        if self.pending_main_loop_common_suffix.is_some() {
            // A source-proven caller return carries the shared
            // ZeldaRunGameLoop suffix with it; retire the pending owner once
            // instead of leaving it to collide with the next fresh iteration.
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites_for_main_loop_once();
            self.clear_nmi_update_latch();
        }
    }

    pub(super) fn Module0F_SpotlightClose(&mut self) {
        let live_table_progress = self.take_original_timing_spotlight_table_build_progress();
        if let Some(receipt) = live_table_progress {
            assert!(
                matches!(
                    receipt.boundary,
                    crate::OriginalTimingBoundary::NmiAccepted
                        | crate::OriginalTimingBoundary::HostReturn
                ),
                "Module0F spotlight table progress must be exposed by an interrupting NMI or direct host return",
            );
        }
        let cpu_plan = if self.rom_startup_timing()
            && !matches!(
                self.original_timing_owner(),
                crate::zelda_rtl::OriginalTimingOwner::Live
            ) {
            let (entry_earliest, entry_latest) = if self.game_state.frame.submodule == 0 {
                (
                    DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_EARLIEST,
                    DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_LATEST,
                )
            } else {
                self.dungeon_exit_spotlight_cpu_entry_envelope
                    .expect("recurring Module0F entry requires the preceding ROM CPU raster")
            };
            let plan = dungeon_exit_spotlight_cpu_plan(self, entry_earliest, entry_latest);
            self.dungeon_exit_spotlight_cpu_entry_envelope =
                plan.and_then(|plan| plan.next_entry_earliest.zip(plan.next_entry_latest));
            plan
        } else {
            // The continuous authority supplies the completed scanout and
            // main-loop progress as typed receipts. Keep the isolated CPU
            // envelope only as the non-Live fallback; consulting it here
            // would make native gameplay depend on a guessed raster entry.
            None
        };
        let vertical_center = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        let phase = cpu_plan.map_or_else(
            || {
                SpotlightIterationPhase::for_close_iteration(
                    self.game_state.frame.submodule,
                    self.game_state.display.spotlight_hdma.window_radius(),
                    vertical_center,
                )
            },
            |plan| {
                if self.game_state.frame.submodule == 0 {
                    if plan.interrupted_during_table_build_or_copy() {
                        SpotlightIterationPhase::CloseEntryBeforeTablePublication
                    } else {
                        SpotlightIterationPhase::CloseEntryAfterTablePublication
                    }
                } else if plan.interrupted_during_table_build_or_copy() {
                    SpotlightIterationPhase::WholeTable
                } else {
                    SpotlightIterationPhase::WholeTableAfterTablePublication
                }
            },
        );
        let iteration = cpu_plan.map_or_else(
            || SpotlightIteration::closing(phase),
            |plan| {
                // The isolated ROM run follows channel 7 across both fields,
                // including a first-field table-build/copy interruption. Keep
                // its following-field rows on the continuation instead of
                // falling back to a geometry projection for interrupted calls.
                if plan.interrupted_during_table_build_or_copy()
                    && !plan.main_loop_sprite_preparation_completed_before_second_nmi
                {
                    return SpotlightIteration::closing(phase)
                        .with_rom_following_field_after_staged_active(plan.following_window_words);
                }
                if plan.interrupted_during_table_build_or_copy() {
                    return SpotlightIteration::closing(phase).with_rom_following_field_receipt(
                        plan.following_window_words,
                        SpotlightFollowingFieldPublication::AfterCompletionCapture,
                    );
                }
                let publication = spotlight_following_field_publication(
                    &plan.active_window_words,
                    &plan.following_window_words,
                    self.next_display_spotlight_scanout.is_some(),
                );
                SpotlightIteration::closing(phase)
                    .with_rom_following_field_receipt(plan.following_window_words, publication)
            },
        );
        let iteration = if cpu_plan.is_some_and(|plan| {
            !plan.returned_to_main_wait_before_first_nmi
                && plan.main_loop_sprite_preparation_completed_before_second_nmi
        }) {
            iteration.with_main_loop_sprite_preparation_before_second_nmi()
        } else {
            iteration
        };
        if self.game_state.frame.submodule != 0 {
            if let Some(plan) = cpu_plan {
                self.publish_or_stage_spotlight_active_field(&plan.active_window_words);
            }
        }
        self.sprite_main();
        if self.game_state.frame.submodule == 0 {
            self.Dungeon_PrepExitWithSpotlight_before_table();
            if self.begin_dungeon_exit_spotlight_entry(
                cpu_plan,
                live_table_progress.map(|receipt| receipt.progress),
                iteration,
            ) {
                // vblank interrupts the first IrisSpotlight_ConfigureTable
                // build; the table copy, radius write, submodule advance, and
                // Link/OAM suffix complete on the next host frame.
                return;
            }
            self.Dungeon_PrepExitWithSpotlight_table_and_advance();
        } else {
            if self.begin_dungeon_exit_spotlight_build(
                cpu_plan,
                live_table_progress.map(|receipt| receipt.progress),
                iteration,
            ) {
                return;
            }
            let (caller_interrupted, _) = self.spotlight_configure_table_and_control(true);
            if caller_interrupted {
                self.schedule_dungeon_exit_spotlight_goal_caller(iteration);
                return;
            }
        }
        if self.begin_module0f_spotlight_close_link_and_oam(cpu_plan, iteration) {
            return;
        }
        if cpu_plan.is_none_or(|plan| !plan.returned_to_main_wait_before_first_nmi) {
            self.schedule_spotlight_iteration_return(iteration);
        }
    }

    pub(super) fn begin_module0f_spotlight_close_link_and_oam(
        &mut self,
        cpu_plan: Option<DungeonExitSpotlightCpuPlan>,
        iteration: SpotlightIteration,
    ) -> bool {
        let authoritative_before_coordinates = self.take_original_timing_main_loop_interruption(
            crate::MainLoopInterruption::LinkPositionBeforeCoordinates,
        );
        let authoritative_link_oam_interruption =
            self.take_original_timing_main_loop_interruption(crate::MainLoopInterruption::LinkOam);
        assert!(
            !(authoritative_before_coordinates && authoritative_link_oam_interruption),
            "one Module0F source call cannot stop before Link coordinates and in LinkOam",
        );
        if authoritative_before_coordinates {
            assert!(
                cpu_plan.is_none(),
                "the isolated spotlight CPU plan and Live Link-position receipt cannot both own one interruption",
            );
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement { iteration },
                1,
            );
            return true;
        }
        if !cpu_plan.is_some_and(|plan| plan.link_position_integrated_before_first_nmi) {
            self.module0f_spotlight_close_link_and_oam();
            if authoritative_link_oam_interruption {
                // LinkOam_Main is the replaceable authority's semantic stop
                // boundary. The translated Link/OAM leaf is atomic, so run it
                // once and retain only ZeldaRunGameLoop's post-Link suffix for
                // the following host. This consumes no CPU/raster provenance
                // and is the same continuation used by the native timing path.
                self.schedule_spotlight_iteration_return(iteration);
                return true;
            }
            return false;
        }
        assert!(
            !authoritative_link_oam_interruption,
            "the isolated spotlight CPU plan and Live LinkOam receipt cannot both own one interruption",
        );
        let position_return = self
            .module0f_spotlight_close_velocity_until_position_integrated()
            .expect("Player_MovePosition1_ interruption requires Module0F's outdoor velocity path");
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity {
                position_return,
                iteration,
            },
            1,
        );
        true
    }

    pub(super) fn module0f_spotlight_close_link_and_oam(&mut self) {
        if let Some(position_return) =
            self.module0f_spotlight_close_velocity_until_position_integrated()
        {
            self.complete_link_move_position_after_coordinates(position_return);
        }
        self.module0f_spotlight_close_after_velocity();
    }

    fn module0f_spotlight_close_velocity_until_position_integrated(
        &mut self,
    ) -> Option<LinkMovePositionReturn> {
        if self.game_state.world.location.is_outdoors() {
            if self.game_state.world.location.overworld_screen_index() == 0x0f {
                self.follower_link_state_mut()
                    .set_water_ripple_or_grass_state(1);
            }
            self.follower_link_state_mut().set_speed_setting(6);
            return self.link_handle_velocity_until_position_integrated();
        }
        None
    }

    fn module0f_spotlight_close_after_velocity(&mut self) {
        if self.game_state.world.location.is_outdoors() {
            self.follower_link_state_mut().clear_movement_velocity();
        }

        let mut i = self.game_state.player.follower_link.facing_index();
        if self.game_state.world.location.is_outdoors() {
            i = if self.game_state.world.region.which_entrance() == 0x43 {
                1
            } else {
                0
            };
        }

        let dir = OVERWORLD_SCREEN_TRANSITION_DIRECTION_BITS[i];
        self.follower_link_state_mut()
            .set_direction_and_last_direction(dir);
        self.link_handle_moving_animation_full_long_entry();
        self.link_oam_main();
    }

    pub(super) fn complete_dungeon_exit_spotlight_link_velocity(
        &mut self,
        position_return: LinkMovePositionReturn,
        iteration: SpotlightIteration,
    ) {
        self.complete_link_move_position_after_coordinates(position_return);
        self.module0f_spotlight_close_after_velocity();
        if iteration.prepares_main_loop_sprites_before_second_nmi() {
            self.nmi_prepare_sprites_for_main_loop_once();
            self.clear_nmi_update_latch();
        } else {
            self.schedule_spotlight_iteration_return(iteration);
        }
    }

    pub(super) fn complete_dungeon_exit_spotlight_link_movement(
        &mut self,
        iteration: SpotlightIteration,
    ) {
        self.module0f_spotlight_close_link_and_oam();
        if iteration.prepares_main_loop_sprites_before_second_nmi() {
            self.nmi_prepare_sprites_for_main_loop_once();
            self.clear_nmi_update_latch();
        } else {
            self.schedule_spotlight_iteration_return(iteration);
        }
    }

    pub(super) fn complete_dungeon_exit_spotlight_entry(
        &mut self,
        table_build: SpotlightTableBuildContinuation,
        iteration: SpotlightIteration,
    ) {
        let vertical_center = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        let phase = SpotlightIterationPhase::for_close_iteration(
            self.game_state.frame.submodule,
            self.game_state.display.spotlight_hdma.window_radius(),
            vertical_center,
        );
        self.complete_iris_spotlight_configure_table(table_build);
        let iteration = if let Some(following) = iteration.rom_following_field_receipt() {
            // The isolated ROM run follows channel 7 from this second NMI to
            // the third. Stage those exact rows when the resumed C table
            // suffix executes, then remove the consumed receipt before the
            // later ZeldaRunGameLoop caller-return continuation.
            self.spotlight_internal_after_table_during_active_rom_field(&following.words);
            iteration.after_rom_following_field_was_staged()
        } else {
            self.spotlight_internal_after_table_during_active_field();
            iteration
        };
        self.increment_submodule();
        self.module0f_spotlight_close_link_and_oam();
        if phase == SpotlightIterationPhase::CloseEntryBeforeTablePublication {
            self.schedule_spotlight_iteration_return(iteration);
        }
        if iteration.prepares_main_loop_sprites_before_second_nmi() {
            self.nmi_prepare_sprites_for_main_loop_once();
            self.clear_nmi_update_latch();
        }
    }

    pub(super) fn complete_dungeon_exit_spotlight_build(
        &mut self,
        table_build: SpotlightTableBuildContinuation,
        projection_completed: bool,
        iteration: SpotlightIteration,
        caller_returned_to_main_wait: bool,
        caller_interrupted_in_link_oam: bool,
    ) {
        assert!(
            !(caller_returned_to_main_wait && caller_interrupted_in_link_oam),
            "one Module0F caller cannot both remain in LinkOam and reach main wait",
        );
        let iteration = if caller_returned_to_main_wait {
            // The replaceable authority observed the resumed C caller return
            // through LinkOam_Main and Main_PrepSpritesForNmi before this host
            // ended. Do not schedule an estimated extra suffix host after the
            // source call has already completed it.
            iteration.with_main_loop_sprite_preparation_before_second_nmi()
        } else {
            iteration
        };
        self.complete_dungeon_exit_spotlight_build_cpu(table_build, projection_completed);
        if caller_interrupted_in_link_oam {
            // The replaceable timing authority observed the resumed C call
            // inside LinkOam_Main. Execute the source prefix once, then retain
            // the shared post-Link-OAM/main-loop suffix for the following
            // host instead of replaying the spotlight table or pretending the
            // caller already reached main wait.
            self.schedule_dungeon_exit_spotlight_link_oam(iteration);
            return;
        }
        if iteration.prepares_main_loop_sprites_before_second_nmi() {
            self.nmi_prepare_sprites_for_main_loop_once();
            self.clear_nmi_update_latch();
        } else {
            // The ROM timing plan is still inside the ZeldaRunGameLoop caller
            // at the second boundary. Preserve that suffix as a real C
            // continuation instead of collapsing it into table completion.
            self.schedule_spotlight_iteration_return(
                iteration.after_rom_following_field_was_staged(),
            );
        }
    }

    /// Finish only the CPU body saved by `FinishDungeonExitSpotlightBuild`.
    ///
    /// This deliberately excludes scheduler ownership and ZeldaRunGameLoop's
    /// common suffix. Live terminal-return authority can therefore run the
    /// table/control/Module0F Link-OAM body after its carried NMI handler and
    /// let the shared terminal executor consume the ordinary suffix exactly
    /// once.
    pub(super) fn complete_dungeon_exit_spotlight_build_cpu(
        &mut self,
        table_build: SpotlightTableBuildContinuation,
        projection_completed: bool,
    ) {
        if projection_completed {
            self.complete_iris_spotlight_configure_table_after_projection();
        } else {
            self.complete_iris_spotlight_configure_table(table_build);
        }
        let caller_interrupted = self.complete_spotlight_configure_table_and_control_after_table(
            self.game_state.frame.main_module,
            false,
        );
        debug_assert!(!caller_interrupted);
        self.module0f_spotlight_close_link_and_oam();
    }

    pub(super) fn complete_dungeon_exit_spotlight_goal_caller(&mut self) {
        self.complete_spotlight_configure_table_and_control_caller();
        self.module0f_spotlight_close_link_and_oam();
        // This continuation resumes the suspended ZeldaRunGameLoop stack.
        // The C caller does not return directly to the host after Module0F:
        // it still packs/sorts the completed sprite shadow before clearing the
        // NMI latch. That full shadow is the operand of the following OAM DMA.
        if self.pending_main_loop_common_suffix.is_some() {
            // A source-proven caller return carries the shared
            // ZeldaRunGameLoop suffix; retire its one owner.
            self.complete_pending_main_loop_common_suffix_after_module_return();
        } else {
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
        }
    }

    pub(super) fn Dungeon_PrepExitWithSpotlight(&mut self) {
        self.Dungeon_PrepExitWithSpotlight_before_table();
        self.Dungeon_PrepExitWithSpotlight_table_and_advance();
    }

    pub(super) fn Dungeon_PrepExitWithSpotlight_table_and_advance(&mut self) {
        self.IrisSpotlight_close();
        self.increment_submodule();
    }

    pub(super) fn Dungeon_PrepExitWithSpotlight_before_table(&mut self) {
        self.deactivate_nmi_thread();
        self.clear_pending_polyhedral_update();
        if self.game_state.world.location.is_outdoors() {
            self.Ancilla_TerminateWaterfallSplashes();
            self.follower_link_state_mut()
                .store_overworld_exit_y_from_current();
        }

        let mut m = self
            .zelda_get_entrance_music_track(self.game_state.world.region.which_entrance() as i32);
        if m != 3 || {
            m = self.game_state.inventory.save_progress.progress_indicator();
            m >= 2
        } {
            if m != 0xf2 {
                m = 0xf1;
            } else if self.game_state.system_signals.current_music_control() == 12 {
                m = 7;
            }
            self.set_music_control(m);
        }

        self.clear_hud_floor_changed_timer();
        self.hud_floor_indicator();
        self.increment_hud_update_flag();
    }

    pub(super) fn SetTargetOverworldWarpToPyramid(&mut self) {
        if self.game_state.frame.main_module != 21 {
            return;
        }
        self.LoadOverworldFromDungeon();
        self.DecompressAnimatedOverworldTiles(0x5a);
        self.ResetAncillaAndCutscene();
    }

    pub(super) fn ResetAncillaAndCutscene(&mut self) {
        self.ancilla_terminate_select_interactives(0);
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        self.follower_link_state_mut().clear_button_b_frames();
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().clear_force_hold_sword_up();
        self.follower_link_state_mut().clear_immobilized();
    }

    pub(super) fn ConditionalMosaicControl(&mut self) {
        if self.game_state.display.palette_filter.countdown() & 1 != 0 {
            self.increment_mosaic_level_by(0x10);
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(7);
    }

    pub(super) fn Overworld_ResetMosaic_alwaysIncrease(&mut self) {
        self.increment_mosaic_level_by(0x10);
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(7);
    }

    pub(super) fn FluteMenu_LoadTransport(&mut self) {
        self.memorized_tile_mut().clear_count();
        let k = self.birdtravel_status() as usize;
        let bird_travel_index_x2 = self.birdtravel_status_word() << 1;
        self.set_birdtravel_status_word(bird_travel_index_x2);
        self.Overworld_LoadBirdTravelPos(k);
    }

    pub(super) fn Overworld_LoadBirdTravelPos(&mut self, k: usize) {
        let screen_index = self
            .asset_raw(113)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_ScreenIndex asset")
            .to_vec();
        let map16_src = self
            .asset_raw(114)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_Map16LoadSrcOff asset")
            .to_vec();
        let scroll_x_table = self
            .asset_raw(115)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_ScrollX asset")
            .to_vec();
        let scroll_y_table = self
            .asset_raw(116)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_ScrollY asset")
            .to_vec();
        let link_x_table = self
            .asset_raw(117)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_LinkXCoord asset")
            .to_vec();
        let link_y_table = self
            .asset_raw(118)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_LinkYCoord asset")
            .to_vec();
        let camera_x_table = self
            .asset_raw(119)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_CameraXScroll asset")
            .to_vec();
        let camera_y_table = self
            .asset_raw(120)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_CameraYScroll asset")
            .to_vec();
        let scroll_up_seed_table = self
            .asset_raw(121)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_Unk1 asset")
            .to_vec();
        let scroll_left_seed_table = self
            .asset_raw(122)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_Unk3 asset")
            .to_vec();

        let scroll_y = read_word_from_slice(&scroll_y_table, k * 2);
        let scroll_x = read_word_from_slice(&scroll_x_table, k * 2);
        self.set_bg1_y(scroll_y);
        self.set_bg2_y(scroll_y);
        self.set_bg1_v_copy(scroll_y);
        self.set_bg2_v_copy(scroll_y);
        self.set_bg1_x(scroll_x);
        self.set_bg2_x(scroll_x);
        self.set_bg1_h_copy(scroll_x);
        self.set_bg2_h_copy(scroll_x);

        let link_y = read_word_from_slice(&link_y_table, k * 2);
        let link_x = read_word_from_slice(&link_x_table, k * 2);
        self.follower_link_state_mut().set_y(link_y);
        self.follower_link_state_mut().set_x(link_x);

        let scroll_up_seed = scroll_up_seed_table[k] as i8 as i16 as u16;
        let scroll_left_seed = scroll_left_seed_table[k] as i8 as i16 as u16;
        self.set_overworld_scroll_up_counter(scroll_up_seed);
        self.set_overworld_scroll_left_counter(scroll_left_seed);
        self.set_overworld_scroll_down_counter(scroll_up_seed.wrapping_neg());
        self.set_overworld_scroll_right_counter(scroll_left_seed.wrapping_neg());

        let screen = read_word_from_slice(&screen_index, k * 2);
        self.set_overworld_area_index_word(screen);
        self.set_overworld_screen_word(screen);

        let src = read_word_from_slice(&map16_src, k * 2);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_y_unit((src.wrapping_sub(0x400) & 0x0f80) >> 7);
        self.set_overworld_map16_dst_off((src.wrapping_sub(0x10) & 0x003e) >> 1);

        let camera_y = read_word_from_slice(&camera_y_table, k * 2);
        self.set_camera_y_coord_scroll_low(camera_y);
        self.set_camera_y_coord_scroll_hi(camera_y.wrapping_sub(2));
        let camera_x = read_word_from_slice(&camera_x_table, k * 2);
        self.set_camera_x_coord_scroll_low(camera_x);
        self.set_camera_x_coord_scroll_hi(camera_x.wrapping_sub(2));

        self.set_ow_entrance_value(0);
        self.dungeon_object_tracking_mut()
            .set_big_rock_starting_address(0);
        self.Overworld_LoadNewScreenProperties();
        self.sprite_reset_all();
        self.sprite_reload_all_overworld();
        self.follower_link_state_mut().clear_doorway_state();
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
    }

    pub(super) fn FluteMenu_LoadSelectedScreenPalettes(&mut self) {
        self.OverworldLoadScreensPaletteSet();
        let sc = self.game_state.world.location.overworld_screen_index() as usize;
        let bg = self.GetOverworldBgPalette(sc as u8);
        let spr = self.overworld_config_table().sprite_palette(sc);
        self.Overworld_LoadPalettes(bg, spr);
        self.Palette_SetOwBgColor();
        self.Overworld_LoadPalettesInner();
    }

    pub(super) fn FindPartnerWhirlpoolExit(&mut self) {
        let screen = u16::from(self.game_state.world.location.overworld_screen_index());
        let whirlpool_areas = self
            .asset_raw(123)
            .expect("FindPartnerWhirlpoolExit missing kWhirlpoolAreas asset")
            .to_vec();
        let count = whirlpool_areas.len() / 2;
        for j in (0..count).rev() {
            if read_word_from_slice(&whirlpool_areas, j * 2) == screen {
                self.memorized_tile_mut().clear_count();
                self.Overworld_LoadBirdTravelPos(j + 9);
                break;
            }
        }
    }

    pub(super) fn Overworld_LoadNewScreenProperties(&mut self) {
        self.tile_detect_position_mut()
            .set_location_calc_mask(!7u16);
        self.Overworld_LoadGFXAndScreenSize();
        self.set_overworld_right_bottom_bound_low(0xe4);
        self.clear_overworld_big_area_high();
        let big = self.overworld_is_big_area();
        let area = (self.game_state.world.location.overworld_screen_index() & 0x3f) as usize;
        self.Overworld_SetCameraBoundaries(if big { 1 } else { 0 }, area as i32);
        self.follower_link_state_mut().set_quadrants(0, 2);
        self.set_fullsize_overworld_quadrants();
        self.follower_link_state_mut().disable_oam_offsets();
        {
            self.follower_link_state_mut().reset_direction_masks();
            self.follower_link_state_mut().set_z_low(0xff);
            self.follower_link_state_mut().set_actual_z_velocity(0xff);
        }
    }

    pub(super) fn LoadCachedEntranceProperties(&mut self) {
        self.restore_exit_area_index();
        self.restore_exit_layer_masks();

        self.restore_exit_bg2_scroll_to_all_layers();

        self.follower_link_state_mut()
            .restore_position_from_overworld_exit();
        if self.game_state.world.location.dungeon_room() < 0x0124 {
            let link_y = self.game_state.player.follower_link.y().wrapping_sub(0x10);
            self.follower_link_state_mut().set_y(link_y);
        }
        self.follower_link_state_mut().set_facing(2);
        if self.game_state.world.region.ow_entrance_value() == 0xffff {
            let link_y = self.game_state.player.follower_link.y().wrapping_add(0x20);
            self.follower_link_state_mut().set_y(link_y);
            self.follower_link_state_mut().set_facing(0);
        }

        let overworld_screen = self.exit_screen_index();
        self.set_overworld_screen_word(overworld_screen);
        self.set_overworld_map16_src_off(self.overworld_exit_map16_src_off());
        let src = self.overworld_map16_src_off();
        self.set_overworld_map16_y_unit((src.wrapping_sub(0x400) & 0x0f80) >> 7);
        self.set_overworld_map16_dst_off((src.wrapping_sub(0x10) & 0x003e) >> 1);

        self.restore_exit_camera_scroll();

        let exit_room_bounds = {
            let bounds = &self.game_state.world.camera_boundaries;
            (
                bounds.exit_room_bound_y_start(),
                bounds.exit_room_bound_y_end(),
                bounds.exit_room_bound_x_start(),
                bounds.exit_room_bound_x_end(),
            )
        };
        self.room_bounds_mut().set_packed_bounds(
            exit_room_bounds.0,
            exit_room_bounds.1,
            exit_room_bounds.2,
            exit_room_bounds.3,
        );
        self.restore_exit_scroll_targets();
        self.restore_exit_scroll_counters();
        self.world_palette_theme_mut().restore_exit_tile_themes();
        self.sprite_system_mut().restore_exit_graphics_index();
    }

    pub(super) fn LoadOverworldFromSpecialOverworld(&mut self) {
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some() {
            println!(
                "spexit-restore-before frame={} area=0x{:04x} screen=0x{:04x} x=0x{:04x} y=0x{:04x} bg=0x{:04x}/0x{:04x} src=0x{:04x} cam=0x{:04x}/0x{:04x} bounds={:04x},{:04x},{:04x},{:04x}",
                self.game_state.frame.frame_counter,
                self.game_state.world.region.spexit_area_index(),
                self.special_exit_screen_index(),
                self.game_state.player.special_exit_position.x(),
                self.game_state.player.special_exit_position.y(),
                self.game_state.display.ppu_scroll_copy.special_exit_bg2_h_copy2(),
                self.game_state.display.ppu_scroll_copy.special_exit_bg2_v_copy2(),
                self.overworld_spexit_map16_src_off(),
                self.game_state.world.camera_boundaries.spexit_camera_x_scroll_low(),
                self.game_state.world.camera_boundaries.spexit_camera_y_scroll_low(),
                self.game_state.world.camera_boundaries.spexit_room_bound_x_start(),
                self.game_state.world.camera_boundaries.spexit_room_bound_x_end(),
                self.game_state.world.camera_boundaries.spexit_room_bound_y_start(),
                self.game_state.world.camera_boundaries.spexit_room_bound_y_end(),
            );
        }
        self.memorized_tile_mut().clear_count();
        self.restore_spexit_area_index();
        self.restore_spexit_layer_masks();
        // TM_COPY(0x1c)/TS_COPY(0x1d) are dual-owned: world_transient models them as the
        // `tilemap_layer_copy` u16 (restored above), but DisplayState ALSO models TS_COPY as
        // `sub_screen_layers` (and TM_COPY as `main_screen_layers`) and re-stamps them via its
        // master projection. Without syncing DisplayState here, its stale sub_screen_layers
        // clobbers the restored TS_COPY at frame end (0x1d 0x00->0x01 at f273590). Keep both
        // owners coherent, matching C's single ram[TM_COPY] copy.
        let restored_layers = self.game_state.world.transient.tilemap_layer_copy;
        self.set_main_screen_layers(restored_layers as u8);
        self.set_sub_screen_layers((restored_layers >> 8) as u8);

        self.restore_special_exit_bg2_scroll_to_all_layers();

        self.special_exit_position_mut().restore_player_position();
        let overworld_screen = self.special_exit_screen_index();
        self.set_overworld_screen_word(overworld_screen);
        self.set_overworld_map16_src_off(self.overworld_spexit_map16_src_off());
        let src = self.overworld_map16_src_off();
        self.set_overworld_map16_y_unit((src.wrapping_sub(0x400) & 0x0f80) >> 7);
        self.set_overworld_map16_dst_off((src.wrapping_sub(0x10) & 0x003e) >> 1);

        self.restore_special_exit_camera_scroll();

        let special_exit_room_bounds = {
            let bounds = &self.game_state.world.camera_boundaries;
            (
                bounds.spexit_room_bound_y_start(),
                bounds.spexit_room_bound_y_end(),
                bounds.spexit_room_bound_x_start(),
                bounds.spexit_room_bound_x_end(),
            )
        };
        self.room_bounds_mut().set_packed_bounds(
            special_exit_room_bounds.0,
            special_exit_room_bounds.1,
            special_exit_room_bounds.2,
            special_exit_room_bounds.3,
        );
        self.restore_spexit_scroll_targets();
        self.restore_spexit_scroll_counters();
        self.world_palette_theme_mut()
            .restore_special_exit_tile_themes();
        self.sprite_system_mut()
            .restore_special_exit_graphics_index();

        let sc = self.game_state.world.location.overworld_screen_index() as usize;
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(sc as u8),
            self.overworld_config_table().sprite_palette(sc),
        );
        self.Palette_SpecialOw();
        self.follower_link_state_mut().set_quadrants(0, 2);
        self.set_fullsize_overworld_quadrants();
        self.follower_link_state_mut().disable_oam_offsets();
        {
            self.follower_link_state_mut().reset_direction_masks();
            self.follower_link_state_mut().set_z_low(0xff);
            self.follower_link_state_mut().set_actual_z_velocity(0xff);
        }
        self.link_reset_swimming_state();
        self.Overworld_LoadGFXAndScreenSize();
        self.set_overworld_right_bottom_bound_low(228);
        self.clear_overworld_big_area_high();
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some() {
            println!(
                "spexit-restore-after frame={} area=0x{:04x} screen=0x{:04x} x=0x{:04x} y=0x{:04x} bg=0x{:04x}/0x{:04x} base=0x{:04x}/0x{:04x} mask=0x{:04x}/0x{:04x} room=0x{:04x} main={} sub={}",
                self.game_state.frame.frame_counter,
                self.game_state.world.region.overworld_area_index(),
                u16::from(self.game_state.world.location.overworld_screen_index()),
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
                self.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
                self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
                self.game_state.world.scroll.overworld_offset_base_x(),
                self.game_state.world.scroll.overworld_offset_base_y(),
                self.game_state.world.scroll.overworld_offset_mask_x(),
                self.game_state.world.scroll.overworld_offset_mask_y(),
                self.game_state.world.location.dungeon_room(),
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            );
        }
    }

    pub(super) fn Overworld_LoadGFXAndScreenSize(&mut self) {
        let i = self.game_state.world.location.overworld_screen_index() as usize;
        self.reset_incremental_vram_upload_counter();
        let graphics_index = self.overworld_config_table().sprite_graphics(i);
        self.sprite_system_mut().set_graphics_index(graphics_index);
        let aux_tile_theme_index = self.asset_u8(108, i);
        self.world_palette_theme_mut()
            .set_aux_tile_theme_index(aux_tile_theme_index);
        self.backup_overworld_big_area_low();

        let small = self.asset_u8(107, i & 0x3f) != 0;
        self.set_overworld_big_area_low(if small { 0 } else { 0x20 });
        self.set_overworld_right_bottom_bound_high(if small { 1 } else { 3 });
        let is_dark_world_screen =
            self.game_state.world.location.overworld_screen_index() & 0x40 != 0;
        let main_tile_theme_index = if is_dark_world_screen { 0x21 } else { 0x20 };
        self.world_palette_theme_mut()
            .set_main_tile_theme_index(main_tile_theme_index);
        let packs = 6 + if is_dark_world_screen { 8 } else { 0 };
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(VARIOUS_PACKS_OVERWORLD[packs]);

        let j = (self.game_state.world.location.overworld_screen_index() & 0xbf) as usize;
        self.set_overworld_offset_base_y(overworld_offset_base_y_c_index(j));
        self.set_overworld_offset_base_x(overworld_offset_base_x_c_index(j) >> 3);
        let mask = if self.overworld_is_big_area() {
            0x03f0
        } else {
            0x01f0
        };
        self.set_overworld_offset_mask_y(mask);
        self.set_overworld_offset_mask_x(mask >> 3);
    }

    pub(super) fn Overworld_SetCameraBoundaries(&mut self, big: i32, area: i32) {
        assert!(
            (0..64).contains(&area),
            "Overworld_SetCameraBoundaries area out of range: {area}"
        );
        assert!(
            (0..=1).contains(&big),
            "Overworld_SetCameraBoundaries big out of range: {big}"
        );
        let area = area as usize;
        let big = big as usize;
        let ystart = OVERWORLD_AREA_BASE_Y[area];
        let xstart = OVERWORLD_AREA_BASE_X[area];
        self.room_bounds_mut().set_packed_bounds(
            ystart,
            ystart.wrapping_add(OVERWORLD_AREA_HEIGHTS_BY_SIZE[big]),
            xstart,
            xstart.wrapping_add(OVERWORLD_AREA_WIDTHS_BY_SIZE[big]),
        );
        let up_down = OVERWORLD_VERTICAL_SCROLL_TARGETS[area];
        self.set_up_down_scroll_target(up_down);
        self.set_up_down_scroll_target_end(
            up_down.wrapping_add(OVERWORLD_VERTICAL_SCROLL_SPANS_BY_SIZE[big]),
        );
        let left_right = OVERWORLD_HORIZONTAL_SCROLL_TARGETS[area];
        self.set_left_right_scroll_target(left_right);
        self.set_left_right_scroll_target_end(
            left_right.wrapping_add(OVERWORLD_HORIZONTAL_SCROLL_SPANS_BY_SIZE[big]),
        );
    }

    fn overworld_map_is_small(&self) -> bool {
        self.asset_u8(
            107,
            self.game_state.world.location.overworld_screen_index() as usize,
        ) != 0
    }

    fn overworld_bg2_word(&self, word_index: usize) -> u16 {
        self.game_state.dungeon.room_tilemaps.bg2_tile(word_index)
    }

    fn overworld_map16_stripe_source_word(&self, source_offset: u16) -> u16 {
        // $82:F3DB/$82:F4A5 use `LDA [$00],Y` with a $7E:2000 base.
        // The 65816 therefore keeps reading contiguous WRAM when the 16-bit
        // source offset passes the nominal $2000-byte BG2 map page. Those
        // adjacent words are intentional inputs to edge stripes; clamping to
        // the typed BG2 page changes the published tilemap.
        read_le_u16(&self.ram, DUNG_BG2 + usize::from(source_offset))
    }

    fn overworld_map16_to_map8_word(&self, map8: &[u8], map16: u16, quarter: usize) -> u16 {
        read_word_from_slice(map8, ((map16 as usize) * 4 + quarter) * 2)
    }

    fn store_overworld_map16_load_state(&mut self, state: OverworldMap16LoadState) {
        self.set_overworld_map16_load_state(state);
    }

    fn store_overworld_prev_map16_load_state(&mut self, state: OverworldMap16LoadState) {
        self.set_overworld_prev_map16_load_state(state);
    }

    fn store_overworld_spexit_map16_src_off(&mut self, src_off: u16) {
        self.set_overworld_spexit_map16_src_off(src_off);
    }

    fn store_overworld_exit_map16_src_off(&mut self, src_off: u16) {
        self.set_overworld_exit_map16_src_off(src_off);
    }

    fn store_small_overworld_map16_scroll_backup(
        &mut self,
        state: SmallOverworldMap16ScrollBackupState,
    ) {
        self.set_small_overworld_map16_scroll_backup_state(state);
    }

    fn overworld_map16_src_off(&self) -> u16 {
        self.game_state.world.overworld.map16.active_load.src_off
    }

    fn overworld_map16_dst_off(&self) -> u16 {
        self.game_state.world.overworld.map16.active_load.dst_off
    }

    fn overworld_map16_y_unit(&self) -> u16 {
        self.game_state.world.overworld.map16.active_load.y_unit
    }

    fn set_overworld_map16_src_off(&mut self, src_off: u16) {
        let mut state = self.game_state.world.overworld.map16.active_load;
        state.src_off = src_off;
        self.store_overworld_map16_load_state(state);
    }

    fn set_overworld_map16_dst_off(&mut self, dst_off: u16) {
        let mut state = self.game_state.world.overworld.map16.active_load;
        state.dst_off = dst_off;
        self.store_overworld_map16_load_state(state);
    }

    fn set_overworld_map16_y_unit(&mut self, y_unit: u16) {
        let mut state = self.game_state.world.overworld.map16.active_load;
        state.y_unit = y_unit;
        self.store_overworld_map16_load_state(state);
    }

    pub(super) fn BufferAndBuildMap16Stripes_X(&mut self, mut dst: usize) -> usize {
        let strip = OVERWORLD_MAP16_STRIP_BACKTRACK_BY_DIRECTION
            [((self.screen_transition_direction_bits() >> 1) & 1) as usize];
        let mut pos = self.overworld_map16_src_off().wrapping_sub(strip);
        let mut y_unit_index = self.overworld_map16_y_unit() as usize & 0x1f;
        for _ in 0..32 {
            let tile = self.overworld_map16_stripe_source_word(pos);
            self.set_dung_replacement_tile_state(y_unit_index, tile);
            y_unit_index = (y_unit_index + 1) & 0x1f;
            pos = pos.wrapping_add(0x80);
        }

        let map8 = self.GetMap16toMap8Table();
        let mut r0 = 0u16;
        let mut dst_unit = self.overworld_map16_dst_off();
        if dst_unit >= 0x10 {
            dst_unit &= 0x0f;
            r0 = 0x400;
        }
        r0 = r0.wrapping_add(dst_unit.wrapping_mul(2));

        let mut tmp = 0usize;
        for _ in 0..2 {
            self.write_overworld_vram_word(dst, r0);
            self.write_overworld_vram_word(dst + 33, r0.wrapping_add(1));
            dst += 1;
            for _ in 0..16 {
                let k = self
                    .game_state
                    .world
                    .transient
                    .dung_replacement_tile_state(tmp);
                tmp += 1;
                let s0 = self.overworld_map16_to_map8_word(&map8, k, 0);
                let s1 = self.overworld_map16_to_map8_word(&map8, k, 1);
                let s2 = self.overworld_map16_to_map8_word(&map8, k, 2);
                let s3 = self.overworld_map16_to_map8_word(&map8, k, 3);
                self.write_overworld_vram_word(dst, s0);
                self.write_overworld_vram_word(dst + 33, s1);
                self.write_overworld_vram_word(dst + 1, s2);
                self.write_overworld_vram_word(dst + 34, s3);
                dst += 2;
            }
            dst += 33;
            r0 = r0.wrapping_add(0x800);
        }
        dst
    }

    pub(super) fn BufferAndBuildMap16Stripes_Y(&mut self, mut dst: usize) -> usize {
        let strip_index = 1 + ((self.screen_transition_direction_bits() >> 2) & 1) as usize;
        let mut pos = self
            .overworld_map16_src_off()
            .wrapping_sub(OVERWORLD_MAP16_STRIP_BACKTRACK_BY_DIRECTION[strip_index]);
        let mut dst_unit_index = self.overworld_map16_dst_off() as usize & 0x1f;
        for _ in 0..32 {
            let tile = self.overworld_map16_stripe_source_word(pos);
            self.set_dung_replacement_tile_state(dst_unit_index, tile);
            pos = pos.wrapping_add(2);
            dst_unit_index = (dst_unit_index + 1) & 0x1f;
        }

        let map8 = self.GetMap16toMap8Table();
        let mut r0 = 0u16;
        let mut y_unit = self.overworld_map16_y_unit();
        if y_unit >= 0x10 {
            y_unit &= 0x0f;
            r0 = 0x800;
        }
        r0 = r0.wrapping_add(y_unit.wrapping_mul(64));

        let mut tmp = 0usize;
        for _ in 0..2 {
            self.write_overworld_vram_word(dst, r0);
            dst += 1;
            for _ in 0..16 {
                let k = self
                    .game_state
                    .world
                    .transient
                    .dung_replacement_tile_state(tmp);
                tmp += 1;
                let s0 = self.overworld_map16_to_map8_word(&map8, k, 0);
                let s1 = self.overworld_map16_to_map8_word(&map8, k, 1);
                let s2 = self.overworld_map16_to_map8_word(&map8, k, 2);
                let s3 = self.overworld_map16_to_map8_word(&map8, k, 3);
                self.write_overworld_vram_word(dst, s0);
                self.write_overworld_vram_word(dst + 32, s2);
                self.write_overworld_vram_word(dst + 1, s1);
                self.write_overworld_vram_word(dst + 33, s3);
                dst += 2;
            }
            dst += 32;
            r0 = r0.wrapping_add(0x400);
        }
        dst
    }

    pub(super) fn BuildFullStripeDuringTransition_North(&mut self, dst: usize) -> usize {
        self.write_overworld_vram_word(dst, 0x0080);
        let dst = self.BufferAndBuildMap16Stripes_Y(dst + 1);
        let src = self.overworld_map16_src_off().wrapping_sub(0x80);
        self.set_overworld_map16_src_off(src);
        let y_unit = self.overworld_map16_y_unit().wrapping_sub(1) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        dst
    }

    pub(super) fn BuildFullStripeDuringTransition_South(&mut self, dst: usize) -> usize {
        self.write_overworld_vram_word(dst, 0x0080);
        let dst = self.BufferAndBuildMap16Stripes_Y(dst + 1);
        let src = self.overworld_map16_src_off().wrapping_add(0x80);
        self.set_overworld_map16_src_off(src);
        let y_unit = self.overworld_map16_y_unit().wrapping_add(1) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        dst
    }

    pub(super) fn BuildFullStripeDuringTransition_West(&mut self, dst: usize) -> usize {
        self.write_overworld_vram_word(dst, 0x8040);
        let dst = self.BufferAndBuildMap16Stripes_X(dst + 1);
        let src = self.overworld_map16_src_off().wrapping_sub(2);
        self.set_overworld_map16_src_off(src);
        let off = self.overworld_map16_dst_off().wrapping_sub(1) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        dst
    }

    pub(super) fn BuildFullStripeDuringTransition_East(&mut self, dst: usize) -> usize {
        self.write_overworld_vram_word(dst, 0x8040);
        let dst = self.BufferAndBuildMap16Stripes_X(dst + 1);
        let src = self.overworld_map16_src_off().wrapping_add(2);
        self.set_overworld_map16_src_off(src);
        let off = self.overworld_map16_dst_off().wrapping_add(1) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        dst
    }

    pub(super) fn OverworldTransitionScrollAndLoadMap(&mut self) {
        let before = self.overworld_map16_src_off();
        let dst = match self.screen_transition_direction_bits() {
            1 => self.BuildFullStripeDuringTransition_East(0),
            2 => self.BuildFullStripeDuringTransition_West(0),
            4 => self.BuildFullStripeDuringTransition_South(0),
            8 => self.BuildFullStripeDuringTransition_North(0),
            _ => {
                self.set_submodule(0);
                panic!(
                    "OverworldTransitionScrollAndLoadMap invalid direction {}",
                    self.screen_transition_direction_bits()
                );
            }
        };
        self.write_overworld_vram_word(dst, 0xffff);
        self.write_overworld_vram_word(dst + 1, 0xffff);
        if dst != 0 {
            self.set_pending_nmi_subroutine(3);
        }
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some()
            && matches!(
                u16::from(self.game_state.world.location.overworld_screen_index()),
                0 | 2
            )
        {
            println!(
                "owstripe-scroll frame={} screen=0x{:04x} dir=0x{:02x} before=0x{:04x} after=0x{:04x} yunit=0x{:04x} dst=0x{:04x} sub={} subsub={}",
                self.game_state.frame.frame_counter,
                u16::from(self.game_state.world.location.overworld_screen_index()),
                self.screen_transition_direction_bits(),
                before,
                self.overworld_map16_src_off(),
                self.overworld_map16_y_unit(),
                self.overworld_map16_dst_off(),
                self.game_state.frame.submodule,
                self.game_state.frame.subsubmodule,
            );
        }
    }

    pub(super) fn TriggerAndFinishMapLoadStripe_Y(&mut self, mut n: i32) {
        self.set_screen_transition_direction_bits(8);
        self.set_pending_nmi_subroutine(3);
        let mut dst = 0usize;
        self.write_overworld_vram_word(dst, 0x0080);
        dst += 1;
        while n != 0 {
            dst = self.BufferAndBuildMap16Stripes_Y(dst);
            let src = self.overworld_map16_src_off().wrapping_sub(0x80);
            self.set_overworld_map16_src_off(src);
            let y_unit = self.overworld_map16_y_unit().wrapping_sub(1) & 0x1f;
            self.set_overworld_map16_y_unit(y_unit);
            n -= 1;
        }
        self.write_overworld_vram_word(dst, 0xffff);
    }

    pub(super) fn TriggerAndFinishMapLoadStripe_X(&mut self, mut n: i32) {
        self.set_screen_transition_direction_bits(2);
        self.set_pending_nmi_subroutine(3);
        let mut dst = 0usize;
        self.write_overworld_vram_word(dst, 0x8040);
        dst += 1;
        while n != 0 {
            dst = self.BufferAndBuildMap16Stripes_X(dst);
            let src = self.overworld_map16_src_off().wrapping_sub(2);
            self.set_overworld_map16_src_off(src);
            let off = self.overworld_map16_dst_off().wrapping_sub(1) & 0x1f;
            self.set_overworld_map16_dst_off(off);
            n -= 1;
        }
        self.write_overworld_vram_word(dst, 0xffff);
    }

    pub(super) fn CreateInitialOWScreenView_Big_North(&mut self) {
        let src = self.overworld_map16_src_off().wrapping_add(0x380);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_y_unit(31);
        self.TriggerAndFinishMapLoadStripe_Y(7);
    }

    pub(super) fn CreateInitialOWScreenView_Big_South(&mut self) {
        let mut pos = self.overworld_map16_src_off();
        while pos >= 0x80 {
            pos = pos.wrapping_sub(0x80);
        }
        self.set_overworld_map16_src_off(pos.wrapping_add(0x780));
        self.set_overworld_map16_y_unit(7);
        self.TriggerAndFinishMapLoadStripe_Y(8);
        let y_unit = self.overworld_map16_y_unit().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        let src = self.overworld_map16_src_off().wrapping_sub(0x0b80);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn CreateInitialOWScreenView_Big_West(&mut self) {
        let src = self.overworld_map16_src_off().wrapping_add(14);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_dst_off(31);
        self.TriggerAndFinishMapLoadStripe_X(7);
    }

    pub(super) fn CreateInitialOWScreenView_Big_East(&mut self) {
        let src = self
            .overworld_map16_src_off()
            .wrapping_sub(0x60)
            .wrapping_add(0x1e);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_dst_off(7);
        self.TriggerAndFinishMapLoadStripe_X(8);
        let off = self.overworld_map16_dst_off().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        let src = self.overworld_map16_src_off().wrapping_sub(0x2e);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn CreateInitialOWScreenView_Small_North(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x700),
            dst_off: map16.dst_off,
            y_unit: 10,
        });
        self.set_overworld_map16_src_off(0x1390);
        self.set_overworld_map16_dst_off(0);
        self.set_overworld_map16_y_unit(31);
        self.TriggerAndFinishMapLoadStripe_Y(7);
    }

    pub(super) fn CreateInitialOWScreenView_Small_South(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src & 0xff,
            dst_off: map16.dst_off,
            y_unit: 24,
        });
        self.set_overworld_map16_src_off(0x0790);
        self.set_overworld_map16_dst_off(0);
        self.set_overworld_map16_y_unit(7);
        self.TriggerAndFinishMapLoadStripe_Y(8);
        let y_unit = self.overworld_map16_y_unit().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        let src = self.overworld_map16_src_off().wrapping_sub(0x0b80);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn CreateInitialOWScreenView_Small_West(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x20),
            dst_off: 8,
            y_unit: map16.y_unit,
        });
        self.set_overworld_map16_src_off(0x044e);
        self.set_overworld_map16_y_unit(0);
        self.set_overworld_map16_dst_off(31);
        self.TriggerAndFinishMapLoadStripe_X(7);
    }

    pub(super) fn CreateInitialOWScreenView_Small_East(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x60),
            dst_off: 0x18,
            y_unit: map16.y_unit,
        });
        self.set_overworld_map16_src_off(0x041e);
        self.set_overworld_map16_y_unit(0);
        self.set_overworld_map16_dst_off(7);
        self.TriggerAndFinishMapLoadStripe_X(8);
        let off = self.overworld_map16_dst_off().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        let src = self.overworld_map16_src_off().wrapping_sub(0x2e);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn CreateInitialNewScreenMapToScroll(&mut self) {
        let dir = self.screen_transition_direction_bits();
        if self.overworld_map_is_small() {
            match dir {
                1 => self.CreateInitialOWScreenView_Small_East(),
                2 => self.CreateInitialOWScreenView_Small_West(),
                4 => self.CreateInitialOWScreenView_Small_South(),
                8 => self.CreateInitialOWScreenView_Small_North(),
                _ => {
                    self.set_submodule(0);
                    panic!(
                        "CreateInitialNewScreenMapToScroll small invalid direction {}",
                        dir
                    );
                }
            }
        } else {
            match dir {
                1 => self.CreateInitialOWScreenView_Big_East(),
                2 => self.CreateInitialOWScreenView_Big_West(),
                4 => self.CreateInitialOWScreenView_Big_South(),
                8 => self.CreateInitialOWScreenView_Big_North(),
                _ => {
                    self.set_submodule(0);
                    panic!(
                        "CreateInitialNewScreenMapToScroll big invalid direction {}",
                        dir
                    );
                }
            }
        }
    }

    pub(super) fn Decompress_bank02(&mut self, dst: usize, src: &[u8]) -> i32 {
        let dst_org = dst;
        let mut dst = dst;
        let mut src_pos = 0usize;
        loop {
            let mut cmd = src[src_pos];
            src_pos += 1;
            if cmd == 0xff {
                return dst.wrapping_sub(dst_org) as i32;
            }
            let mut len;
            if cmd & 0xe0 != 0xe0 {
                len = (cmd & 0x1f) as usize + 1;
                cmd &= 0xe0;
            } else {
                let len_lo = src[src_pos];
                src_pos += 1;
                len = len_lo as usize + (((cmd & 3) as usize) << 8) + 1;
                cmd = (cmd << 3) & 0xe0;
            }

            if cmd == 0 {
                while len != 0 {
                    let value = src[src_pos];
                    src_pos += 1;
                    self.write_overworld_map16_decompressed_byte(dst, value);
                    dst += 1;
                    len -= 1;
                }
            } else if cmd & 0x80 != 0 {
                let hi = src[src_pos] as usize;
                let lo = src[src_pos + 1] as usize;
                src_pos += 2;
                let mut offs = (hi << 8) | lo;
                while len != 0 {
                    self.copy_overworld_map16_decompressed_byte(dst_org, dst, offs);
                    dst += 1;
                    offs += 1;
                    len -= 1;
                }
            } else if cmd & 0x40 == 0 {
                let value = src[src_pos];
                src_pos += 1;
                while len != 0 {
                    self.write_overworld_map16_decompressed_byte(dst, value);
                    dst += 1;
                    len -= 1;
                }
            } else if cmd & 0x20 == 0 {
                let lo = src[src_pos];
                let hi = src[src_pos + 1];
                src_pos += 2;
                while len != 0 {
                    self.write_overworld_map16_decompressed_byte(dst, lo);
                    dst += 1;
                    len -= 1;
                    if len == 0 {
                        break;
                    }
                    self.write_overworld_map16_decompressed_byte(dst, hi);
                    dst += 1;
                    len -= 1;
                }
            } else {
                let mut value = src[src_pos];
                src_pos += 1;
                while len != 0 {
                    self.write_overworld_map16_decompressed_byte(dst, value);
                    dst += 1;
                    value = value.wrapping_add(1);
                    len -= 1;
                }
            }
        }
    }

    pub(super) fn Overworld_DecompressAndDrawAllQuadrants(&mut self) {
        let si = self.game_state.world.location.overworld_screen_index() as i32;
        self.Overworld_DecompressAndDrawOneQuadrant(0x2000, si);
        self.Overworld_DecompressAndDrawOneQuadrant(0x2040, si + 1);
        self.Overworld_DecompressAndDrawOneQuadrant(0x3000, si + 8);
        self.Overworld_DecompressAndDrawOneQuadrant(0x3040, si + 9);
    }

    pub(super) fn Overworld_DecompressAndDrawOneQuadrant(&mut self, mut dst: usize, screen: i32) {
        let hibytes = self.GetOverworldHibytes(screen);
        self.Decompress_bank02(OVERWORLD_DECOMP_BUFFER, &hibytes);
        self.copy_overworld_map16_scratch_to_source_words_high(256);

        let lobytes = self.GetOverworldLobytes(screen);
        self.Decompress_bank02(OVERWORLD_DECOMP_BUFFER, &lobytes);
        self.copy_overworld_map16_scratch_to_source_words_low(256);

        self.set_overworld_map16_decode_last(0xffff);
        let mut src_offset = 0usize;
        for _ in 0..16 {
            for _ in 0..16 {
                let input = self
                    .overworld_map16_decode()
                    .source_word(src_offset)
                    .wrapping_mul(2);
                src_offset += 2;
                self.Overworld_ParseMap32Definition(dst, input);
                dst += 4;
            }
            dst += 192;
        }
    }

    fn fill_map16_decode_block(&mut self, dst: usize, table: &[u8], x: usize) {
        self.fill_overworld_map16_decode_block(dst, table, x);
    }

    pub(super) fn Overworld_ParseMap32Definition(&mut self, dst: usize, input: u16) {
        let a = input & !7;
        if a != self.overworld_map16_decode().decode_last() {
            self.set_overworld_map16_decode_last(a);
            self.set_overworld_map16_decode_tmp(a >> 1);
            let x = (a >> 1) as usize + (a >> 2) as usize;
            let map0 = self
                .asset_raw(60)
                .expect("Overworld_ParseMap32Definition missing kMap32ToMap16_0 asset")
                .to_vec();
            let map1 = self
                .asset_raw(61)
                .expect("Overworld_ParseMap32Definition missing kMap32ToMap16_1 asset")
                .to_vec();
            let map2 = self
                .asset_raw(62)
                .expect("Overworld_ParseMap32Definition missing kMap32ToMap16_2 asset")
                .to_vec();
            let map3 = self
                .asset_raw(63)
                .expect("Overworld_ParseMap32Definition missing kMap32ToMap16_3 asset")
                .to_vec();
            self.fill_map16_decode_block(MAP16_DECODE_0_OVERWORLD, &map0, x);
            self.fill_map16_decode_block(MAP16_DECODE_1_OVERWORLD, &map1, x);
            self.fill_map16_decode_block(MAP16_DECODE_2_OVERWORLD, &map2, x);
            self.fill_map16_decode_block(MAP16_DECODE_3_OVERWORLD, &map3, x);
        }

        let idx = (input & 7) as usize;
        self.write_decoded_overworld_map32_to_bg2_tilemap(dst, idx);
    }

    pub(super) fn OverworldLoad_LoadSubOverlayMap32(&mut self) {
        let si = self.game_state.world.location.overworld_screen_index() as i32;
        self.Overworld_DecompressAndDrawOneQuadrant(0x4000, si);
    }

    pub(super) fn Map16ToMap8(&mut self, source_page: OverworldMap16SourcePage, r20: i32) {
        let map16_src = self.overworld_map16_src_off().wrapping_add(0x1000);
        self.set_overworld_map16_src_off(map16_src);
        let mut r14 = 0i32;
        let mut r10 = WORD_7F4000_OVERWORLD;
        for _ in 0..32 {
            self.OverworldCopyMap16ToBuffer(source_page, r20 as u16, r14, r10);
            r14 += 0x100;
            r10 += 4;
            let map16_src = self.overworld_map16_src_off().wrapping_sub(0x80);
            self.set_overworld_map16_src_off(map16_src);
            let y_unit = self.overworld_map16_y_unit().wrapping_sub(1) & 0x1f;
            self.set_overworld_map16_y_unit(y_unit);
        }
    }

    pub(super) fn OverworldCopyMap16ToBuffer(
        &mut self,
        source_page: OverworldMap16SourcePage,
        r20: u16,
        mut r14: i32,
        mut r10: usize,
    ) {
        let map8 = self.GetMap16toMap8Table();
        let mut yr = (self.overworld_map16_src_off().wrapping_sub(0x410) & 0x1fff) as usize;
        let mut xr = self.overworld_map16_dst_off() as usize & 0x1f;
        for _ in 0..32 {
            let value = self
                .overworld_map16_decode()
                .source_page_word(source_page, yr);
            self.set_dung_replacement_tile_state(xr, value);
            xr = (xr + 1) & 0x1f;
            yr = (yr + 2) & 0x1fff;
        }

        let mut r0 = 0u16;
        let mut y_unit = self.overworld_map16_y_unit();
        if y_unit >= 0x10 {
            y_unit &= 0x0f;
            r0 = 0x800;
        }
        r0 = r0.wrapping_add(y_unit.wrapping_mul(64));

        let mut tmp = 0usize;
        for _ in 0..2 {
            self.write_vram_upload_absolute_word(r10, r0 | r20);
            r10 += 2;
            for _ in 0..16 {
                let k = self
                    .game_state
                    .world
                    .transient
                    .dung_replacement_tile_state(tmp);
                tmp += 1;
                let m0 = self.overworld_map16_to_map8_word(&map8, k, 0);
                let m1 = self.overworld_map16_to_map8_word(&map8, k, 1);
                let m2 = self.overworld_map16_to_map8_word(&map8, k, 2);
                let m3 = self.overworld_map16_to_map8_word(&map8, k, 3);
                self.dungeon_bg2_attributes_mut()
                    .set_bg2_attr_word(r14 as usize, m0);
                self.dungeon_bg2_attributes_mut()
                    .set_bg2_attr_word(r14 as usize + 64, m2);
                self.dungeon_bg2_attributes_mut()
                    .set_bg2_attr_word(r14 as usize + 2, m1);
                self.dungeon_bg2_attributes_mut()
                    .set_bg2_attr_word(r14 as usize + 66, m3);
                r14 += 4;
            }
            r0 = r0.wrapping_add(0x400);
            r14 += 0x40;
        }
    }

    pub(super) fn SomeTileMapChange(&mut self) {
        self.Overworld_DecompressAndDrawAllQuadrants();
        for i in 0..64 {
            self.dungeon_room_tilemaps_mut().set_bg1_tile(i, 0x0dc4);
        }
        self.Overworld_HandleOverlaysAndBombDoors();
        self.increment_submodule();
    }

    pub(super) fn Module09_LoadNewMapAndGFX(&mut self) {
        self.set_overworld_peg_puzzle_progress(0);
        if self.rom_startup_timing() {
            let timing =
                overworld_map_and_sprite_graphics_timing(self.overworld_map_graphics_workload());
            // The first quadrant decompression begins on the caller's entry
            // slice. The ROM exposes two distinct interruptible generations:
            // map quadrants finish first (and advance the visible submodule),
            // while the initial screen map and sprite conversion remain on
            // the CPU stack for a separately named tail.
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishOverworldMapQuadrants {
                    scroll_map_and_sprite_gfx_tail_nmi_slices: timing
                        .scroll_map_and_sprite_gfx_tail_nmi_slices,
                },
                timing.quadrant_load_nmi_slices,
            );
            return;
        }
        self.complete_module09_load_new_map_and_gfx();
    }

    pub(super) fn overworld_map_graphics_workload(&self) -> OverworldMapGraphicsWorkload {
        let screen = i32::from(self.game_state.world.location.overworld_screen_index());
        let mut map32_definition_changes = 0;
        for quadrant in [screen, screen + 1, screen + 8, screen + 9] {
            let high_bytes = decompress_asset(&self.GetOverworldHibytes(quadrant));
            let low_bytes = decompress_asset(&self.GetOverworldLobytes(quadrant));
            let mut last_definition = u16::MAX;
            for (&high, &low) in high_bytes.iter().zip(&low_bytes).take(256) {
                let definition = u16::from_le_bytes([low, high]).wrapping_mul(2) & !7;
                if definition != last_definition {
                    map32_definition_changes += 1;
                    last_definition = definition;
                }
            }
        }
        OverworldMapGraphicsWorkload {
            map32_definition_changes,
        }
    }

    pub(super) fn complete_module09_load_new_map_quadrants(&mut self) {
        self.SomeTileMapChange();
        self.increment_core_update_disable_flag();
    }

    pub(super) fn complete_module09_load_new_map_and_gfx_tail(&mut self) {
        self.CreateInitialNewScreenMapToScroll();
        self.LoadNewSpriteGFXSet();
    }

    pub(super) fn complete_module09_load_new_map_and_gfx(&mut self) {
        self.complete_module09_load_new_map_quadrants();
        self.complete_module09_load_new_map_and_gfx_tail();
    }

    pub(super) fn Overworld_DrawQuadrantsAndOverlays(&mut self) {
        self.Overworld_DecompressAndDrawAllQuadrants();
        for i in 0..64 {
            self.dungeon_room_tilemaps_mut().set_bg1_tile(i, 0x0dc4);
        }
        let mut pos = self.game_state.world.region.ow_entrance_value();
        self.replay_trace_door_overlay("draw-before-entrance", pos & 0x1fff);
        if pos != 0 && pos != 0xffff {
            if pos < 0x8000 {
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile_by_byte_pos(pos, 0x0da4);
                self.Overworld_Memorize_Map16_Change(pos, 0x0da4);
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile_by_byte_pos(pos.wrapping_add(2), 0x0da6);
                self.Overworld_Memorize_Map16_Change(pos.wrapping_add(2), 0x0da6);
                self.replay_trace_door_overlay("draw-normal-door", pos);
            } else {
                pos &= 0x1fff;
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile_by_byte_pos(pos, 0x0db4);
                self.Overworld_Memorize_Map16_Change(pos, 0x0db4);
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile_by_byte_pos(pos.wrapping_add(2), 0x0db5);
                self.Overworld_Memorize_Map16_Change(pos.wrapping_add(2), 0x0db5);
                self.replay_trace_door_overlay("draw-open-door", pos);
            }
            self.set_ow_entrance_value(0);
        }
        self.Overworld_HandleOverlaysAndBombDoors();
        let screen_byte = self.game_state.world.location.overworld_screen_index() as usize;
        if screen_byte < SECONDARY_OVERLAY_BY_OVERWORLD_SCREEN.len() {
            self.replay_trace_door_overlay(
                "draw-after-overlays",
                SECONDARY_OVERLAY_BY_OVERWORLD_SCREEN[screen_byte],
            );
        }
    }

    pub(super) fn MirrorBonk_RecoverChangedTiles(&mut self) {
        let count = self.game_state.memorized_tiles.count() >> 1;
        for i in 0..count as usize {
            let memorized_tiles = &self.game_state.memorized_tiles;
            let pos = memorized_tiles.entry_addr(i);
            let value = memorized_tiles.entry_value(i);
            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile_by_byte_pos(pos, value);
        }
    }

    pub(super) fn CheckForNewlyLoadedMapAreas_North(&mut self, dst: usize) -> usize {
        let src = self.overworld_map16_src_off();
        if (src as i16).wrapping_sub(0x80) < 0 {
            return dst;
        }
        let mut dst = dst;
        if !self.overworld_map_is_small() {
            self.write_overworld_vram_word(dst, 0x0080);
            dst = self.BufferAndBuildMap16Stripes_Y(dst + 1);
        }
        self.set_overworld_map16_src_off(src.wrapping_sub(0x80));
        let y_unit = self.overworld_map16_y_unit().wrapping_sub(1) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        dst
    }

    pub(super) fn CheckForNewlyLoadedMapAreas_South(&mut self, dst: usize) -> usize {
        let src = self.overworld_map16_src_off();
        if src >= 0x1800 {
            return dst;
        }
        let mut dst = dst;
        if !self.overworld_map_is_small() {
            self.write_overworld_vram_word(dst, 0x0080);
            dst = self.BufferAndBuildMap16Stripes_Y(dst + 1);
        }
        self.set_overworld_map16_src_off(src.wrapping_add(0x80));
        let y_unit = self.overworld_map16_y_unit().wrapping_add(1) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        dst
    }

    pub(super) fn CheckForNewlyLoadedMapAreas_West(&mut self, dst: usize) -> usize {
        let mut pos = self.overworld_map16_src_off();
        while pos >= 0x80 {
            pos = pos.wrapping_sub(0x80);
        }
        if pos == 0 {
            return dst;
        }
        let mut dst = dst;
        if !self.overworld_map_is_small() {
            self.write_overworld_vram_word(dst, 0x8040);
            dst = self.BufferAndBuildMap16Stripes_X(dst + 1);
        }
        let src = self.overworld_map16_src_off().wrapping_sub(2);
        self.set_overworld_map16_src_off(src);
        let off = self.overworld_map16_dst_off().wrapping_sub(1) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        dst
    }

    pub(super) fn CheckForNewlyLoadedMapAreas_East(&mut self, dst: usize) -> usize {
        let mut pos = self.overworld_map16_src_off();
        while pos >= 0x80 {
            pos = pos.wrapping_sub(0x80);
        }
        if pos >= 0x60 {
            return dst;
        }
        let mut dst = dst;
        if !self.overworld_map_is_small() {
            self.write_overworld_vram_word(dst, 0x8040);
            dst = self.BufferAndBuildMap16Stripes_X(dst + 1);
        }
        let src = self.overworld_map16_src_off().wrapping_add(2);
        self.set_overworld_map16_src_off(src);
        let off = self.overworld_map16_dst_off().wrapping_add(1) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        dst
    }

    pub(super) fn OverworldHandleMapScroll(&mut self) {
        let before = self.overworld_map16_src_off();
        let before_y_unit = self.overworld_map16_y_unit();
        let before_dst = self.overworld_map16_dst_off();
        let dir = self.screen_transition_direction_bits();
        let dst = match dir {
            1 => {
                let dst = self.CheckForNewlyLoadedMapAreas_East(0);
                self.clear_screen_transition_direction_bits();
                dst
            }
            2 => {
                let dst = self.CheckForNewlyLoadedMapAreas_West(0);
                self.clear_screen_transition_direction_bits();
                dst
            }
            4 => {
                let dst = self.CheckForNewlyLoadedMapAreas_South(0);
                self.clear_screen_transition_direction_bits();
                dst
            }
            5 | 6 => {
                let dst = self.CheckForNewlyLoadedMapAreas_South(0);
                self.and_screen_transition_direction_bits(3);
                dst
            }
            8 => {
                let dst = self.CheckForNewlyLoadedMapAreas_North(0);
                self.clear_screen_transition_direction_bits();
                dst
            }
            9 | 10 => {
                let dst = self.CheckForNewlyLoadedMapAreas_North(0);
                self.and_screen_transition_direction_bits(3);
                dst
            }
            _ => {
                self.set_submodule(0);
                panic!("OverworldHandleMapScroll invalid direction {dir}");
            }
        };
        self.write_overworld_vram_word(dst, 0xffff);
        self.write_overworld_vram_word(dst + 1, 0xffff);
        if dst != 0 {
            self.set_pending_nmi_subroutine(3);
        }
        let screen_transition = self.screen_transition_direction_bits();
        self.set_screen_transition(screen_transition);
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some()
            && matches!(
                u16::from(self.game_state.world.location.overworld_screen_index()),
                0 | 2 | 0x80
            )
        {
            println!(
                "owlive-scroll frame={} screen=0x{:04x} dir=0x{:02x} before=0x{:04x} after=0x{:04x} yunit=0x{:04x}->0x{:04x} dst=0x{:04x}->0x{:04x} trans=0x{:02x} sub={} subsub={} x=0x{:04x} y=0x{:04x}",
                self.game_state.frame.frame_counter,
                u16::from(self.game_state.world.location.overworld_screen_index()),
                dir,
                before,
                self.overworld_map16_src_off(),
                before_y_unit,
                self.overworld_map16_y_unit(),
                before_dst,
                self.overworld_map16_dst_off(),
                self.screen_transition(),
                self.game_state.frame.submodule,
                self.game_state.frame.subsubmodule,
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
            );
        }
    }

    pub(super) fn Overworld_RunScrollTransition(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        self.Graphics_IncrementalVRAMUpload();
        let rv = self.OverworldScrollTransition();
        if rv & 0x0f == 0 {
            let direction_bits = self.edge_transition_direction_bits();
            self.set_screen_transition_direction_bits(direction_bits);
            self.OverworldTransitionScrollAndLoadMap();
            self.clear_screen_transition_direction_bits();
        }
    }

    pub(super) fn Module09_LoadNewSprites(&mut self) {
        let entry_phase = self
            .next_overworld_sprite_reload_entry_phase
            .take()
            .unwrap_or(OverworldSpriteReloadEntryPhase::OrdinaryModuleIteration);
        self.module09_load_new_sprites_from_phase(entry_phase);
    }

    fn module09_load_new_sprites_from_phase(
        &mut self,
        entry_phase: OverworldSpriteReloadEntryPhase,
    ) {
        if self.screen_transition() == 1 {
            let bg2v = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_add(2);
            self.set_bg2_y(bg2v);
            let link_y = self.game_state.player.follower_link.y().wrapping_add(2);
            self.follower_link_state_mut().set_y(link_y);
        }
        let sprite_reload_workload = self.sprite_overworld_reload_all_just_load();
        self.memorized_tile_mut().clear_count();
        if !self.rom_startup_timing()
            && self.game_state.inventory.save_progress.progress_indicator() >= 2
            && self.game_state.frame.submodule != 18
        {
            self.Overworld_SetFixedColAndScroll();
        }
        if self.rom_startup_timing() {
            // The sprite reset/load work begins while the ROM is still in
            // submodule 4 and is interrupted by several vblanks. Its rebuilt
            // sprite set must replace the stale OAM publication on this frame,
            // but the transition-control tail does not return until four more
            // NMI boundaries have passed.
            let reload_timing = overworld_sprite_reload_timing(sprite_reload_workload, entry_phase);
            let bg1_before_provisional_rain = (
                self.game_state.display.ppu_scroll_copy.bg1_h_copy2(),
                self.game_state.display.ppu_scroll_copy.bg1_v_copy2(),
            );
            self.publish_module09_transition_sprites_without_scroll();
            let bg1_after_provisional_rain = (
                self.game_state.display.ppu_scroll_copy.bg1_h_copy2(),
                self.game_state.display.ppu_scroll_copy.bg1_v_copy2(),
            );
            // The provisional caller suffix stages the transition-adjusted
            // BG2 value in the NMI register mirror. Its rain tick advances
            // only BG1's copy2 value for a later caller suffix. Record whether
            // that tick actually ran so the return boundary can retain the
            // prepublished BG1 generation without delaying an older rain tick.
            let bg1_generation = if bg1_before_provisional_rain != bg1_after_provisional_rain {
                OverworldSpriteReloadBg1Generation::RetainBeforePrepublishedRain
            } else {
                OverworldSpriteReloadBg1Generation::ComposeAtTransitionReturn
            };
            let resume_scanout = reload_timing
                .resume_boundary
                .capture_scanout(self, bg1_generation);
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishOverworldSpriteReloadTail {
                    post_return_hold_nmi_slices: reload_timing.post_return_hold_nmi_slices,
                    return_phase: reload_timing.return_phase,
                    epilogue_phase: reload_timing.epilogue_phase,
                    resume_scanout,
                },
                reload_timing.load_nmi_slices,
            );
            return;
        }
        self.complete_module09_load_new_sprites_after_reload();
    }

    pub(super) fn complete_module09_load_new_sprites_after_reload(&mut self) {
        if self.game_state.inventory.save_progress.progress_indicator() >= 2
            && self.game_state.frame.submodule != 18
        {
            self.Overworld_SetFixedColAndScroll();
        }
        self.Overworld_StartScrollTransition();
    }

    pub(super) fn Overworld_StartScrollTransition(&mut self) {
        self.increment_submodule();
        if self.edge_transition_direction_bits() >= 4 {
            let direction_bits = self.edge_transition_direction_bits();
            self.set_screen_transition_direction_bits(direction_bits);
            self.OverworldTransitionScrollAndLoadMap();
            self.clear_screen_transition_direction_bits();
        }
    }

    pub(super) fn Overworld_EaseOffScrollTransition(&mut self) {
        if self.overworld_map_is_small() {
            let direction_bits = self.edge_transition_direction_bits();
            self.set_screen_transition_direction_bits(direction_bits);
            self.OverworldTransitionScrollAndLoadMap();
            self.clear_screen_transition_direction_bits();
        }
        self.increment_subsubmodule();
        if self.game_state.frame.subsubmodule < 8 {
            return;
        }
        let dir = self.edge_transition_direction_bits();
        if (dir == 8 || dir == 2) && self.game_state.frame.subsubmodule < 9 {
            return;
        }

        self.set_subsubmodule(0);
        self.set_edge_transition_direction_bits(0);

        if self.overworld_map_is_small() {
            // RAM-resident backup (C reads orange_blue_barrier_state / word_7EC174 /
            // word_7EC176 live): a dungeon visit may legitimately have overwritten
            // the barrier-aliased src_off word.
            let backup = SmallOverworldMap16ScrollBackupState::load_from_ram(&self.ram);
            self.store_overworld_map16_load_state(OverworldMap16LoadState {
                src_off: backup.src_off,
                dst_off: backup.dst_off,
                y_unit: backup.y_unit,
            });
        }
        self.increment_submodule();
        self.follower_disable();
    }

    pub(super) fn OverworldHandleTransitions(&mut self) {
        if self.has_screen_transition_direction_bits() {
            self.OverworldHandleMapScroll();
        }

        let current_area = self.game_state.world.region.current_area_of_player_word();
        let area_half = (self.game_state.world.region.current_area_of_player() >> 1) as usize;
        let bounds = self.overworld_right_bottom_scroll_bound();
        let mut transition: Option<(u8, usize)> = None;

        if self.game_state.player.follower_link.y_velocity() != 0 {
            let dir = self.game_state.player.follower_link.direction() & 12;
            let t = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(OVERWORLD_AREA_BASE_Y[area_half]);
            if t < 4 {
                transition = Some((dir, 3));
            } else if t >= bounds {
                transition = Some((dir, 2));
            }
        }

        if transition.is_none() && self.game_state.player.follower_link.x_velocity() != 0 {
            let dir = self.game_state.player.follower_link.direction() & 3;
            let t = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_sub(OVERWORLD_AREA_BASE_X[area_half]);
            if t < 6 {
                transition = Some((dir, 1));
            } else if t >= bounds.wrapping_add(4) {
                transition = Some((dir, 0));
            }
        }

        let Some((dir, y_idx)) = transition else {
            self.Overworld_CheckSpecialSwitchArea();
            return;
        };

        let expected_dir = [1u8, 2, 4, 8][y_idx];
        if expected_dir != dir || self.link_check_for_edge_screen_transition() {
            self.Overworld_CheckSpecialSwitchArea();
            return;
        }

        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        let mut map16 = self.overworld_map16_src_off();
        let map16_before = map16;
        map16 &= SPECIAL_SWITCH_MAP16_MASKS[y_idx];
        let pushed = (current_area.wrapping_add_signed(SPECIAL_SWITCH_AREA_DELTAS[y_idx]) >> 1)
            as usize
            & 0x3f;
        let map16_add = SPECIAL_SWITCH_MAP16_OFFSETS[y_idx * 64 + pushed];
        map16 = map16.wrapping_add(map16_add);
        self.set_overworld_map16_src_off(map16);
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some() {
            println!(
                "owtrans-map16 frame={} y={} pushed=0x{:02x} cur=0x{:04x} old=0x{:04x} mask=0x{:04x} add=0x{:04x} new=0x{:04x} screen=0x{:04x} x=0x{:04x} ycoord=0x{:04x} dir=0x{:02x}",
                self.game_state.frame.frame_counter,
                y_idx,
                pushed,
                current_area,
                map16_before,
                SPECIAL_SWITCH_MAP16_MASKS[y_idx],
                map16_add,
                map16,
                u16::from(self.game_state.world.location.overworld_screen_index()),
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
                self.game_state.player.follower_link.direction(),
            );
        }

        let old_screen = self.game_state.world.location.overworld_screen_index();
        if old_screen == 0x2a {
            self.set_ambient_sound_effect(0x80);
        }

        let new_area = OVERWORLD_AREA_TILEMAP_HEADS[pushed]
            | self.game_state.inventory.save_progress.dark_world_state();
        self.set_overworld_screen(new_area);
        self.set_overworld_area_index(new_area);
        if self.game_state.inventory.save_progress.dark_world_state() == 0
            || self.game_state.inventory.items.moon_pearl() != 0
        {
            let music = self.overworld_config_table().music(new_area as usize);
            if music & 0xf0 == 0 {
                self.set_ambient_sound_effect(5);
            }
            if !self.zelda_is_playing_music_track(music & 0x0f) {
                self.set_music_control(0xf1);
            }
        }

        self.Overworld_LoadGFXAndScreenSize();
        self.set_submodule(1);
        self.set_edge_transition_direction_bits(dir);
        self.set_screen_transition_direction_bits(dir);
        let dir_enum = self.DirToEnum(dir as i32) as u8;
        self.set_transition_direction_enum(dir_enum);
        self.set_screen_transition(dir_enum);
        self.set_ow_entrance_value(0);
        // C clears only the LOW byte here (`dung_savegame_info[BIG_ROCK] = 0`, a u8 write),
        // leaving the high byte intact (e.g. 0x0100 stays 0x0100). Clearing the whole u16
        // would blank a live big-rock starting address and then re-project 0 over RAM.
        let big_rock = self
            .game_state
            .dungeon
            .object_tracking
            .big_rock_starting_address();
        self.dungeon_object_tracking_mut()
            .set_big_rock_starting_address(big_rock & 0xff00);
        self.set_transition_counter(0);

        if old_screen & 0x3f == 0
            || self.game_state.world.location.overworld_screen_index() & 0xbf == 0
        {
            self.set_subsubmodule(0);
            self.set_submodule(13);
            self.set_mosaic_copy(0);
            self.clear_mosaic_level();
        } else {
            let sc = self.game_state.world.location.overworld_screen_index() as usize;
            self.Overworld_LoadPalettes(
                self.GetOverworldBgPalette(sc as u8),
                self.overworld_config_table().sprite_palette(sc),
            );
            self.Overworld_CopyPalettesToCache();
        }
    }

    pub(super) fn Overworld_OperateCameraScroll(&mut self) {
        let z = if self.game_state.world.transient.allow_scroll_z() != 0
            && self.game_state.player.follower_link.z() != 0xffff
        {
            self.game_state.player.follower_link.z()
        } else {
            0
        };
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(z)
            .wrapping_add(12);

        let link_y_velocity = self.game_state.player.follower_link.y_velocity();
        if link_y_velocity != 0 {
            let vy = if (link_y_velocity as i8).is_negative() {
                -1
            } else {
                1
            };
            let mut av = if (link_y_velocity as i8).is_negative() {
                (!link_y_velocity).wrapping_add(1)
            } else {
                link_y_velocity
            };
            let mut r4 = 0u16;
            while av != 0 {
                if (link_y_velocity as i8).is_negative() {
                    if y <= self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_y_coord_scroll_low()
                    {
                        r4 = r4.wrapping_add(self.OverworldCameraBoundaryCheck(6, 0, vy, 0) as u16);
                    }
                } else if y
                    >= self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_y_coord_scroll_hi()
                {
                    r4 = r4.wrapping_add(self.OverworldCameraBoundaryCheck(6, 2, vy, 0) as u16);
                }
                av = av.wrapping_sub(1);
            }
            self.set_overworld_vertical_scroll_delta(r4);
            let oi = self.game_state.world.region.overlay_index();
            if oi != 0x97 && oi != 0x9d && r4 != 0 {
                let (subp, mut scroll) = if oi == 0xb5 || oi == 0xbe {
                    ((r4 & 3) << 14, r4 >> 2)
                } else {
                    ((r4 & 1) << 15, r4 >> 1)
                };
                if scroll
                    >= if oi == 0xb5 || oi == 0xbe {
                        0x3000
                    } else {
                        0x7000
                    }
                {
                    scroll |= 0xf000;
                }
                self.add_bg1_v_live_subpixel(subp, scroll);
                if self.game_state.world.location.overworld_screen_index() & 0x3f == 0x1b {
                    let bg1 = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
                    if bg1 <= 0x0600 {
                        self.set_bg1_y(0x0600);
                    } else if bg1 >= 0x06c0 {
                        self.set_bg1_y(0x06c0);
                    }
                }
            }
        }

        let x = self.game_state.player.follower_link.x().wrapping_add(8);
        let link_x_velocity = self.game_state.player.follower_link.x_velocity();
        if link_x_velocity != 0 {
            let vx = if (link_x_velocity as i8).is_negative() {
                -1
            } else {
                1
            };
            let mut ax = if (link_x_velocity as i8).is_negative() {
                (!link_x_velocity).wrapping_add(1)
            } else {
                link_x_velocity
            };
            let mut r4 = 0u16;
            while ax != 0 {
                if (link_x_velocity as i8).is_negative() {
                    if x <= self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_x_coord_scroll_low()
                    {
                        r4 = r4.wrapping_add(self.OverworldCameraBoundaryCheck(0, 4, vx, 4) as u16);
                    }
                } else if x
                    >= self
                        .game_state
                        .world
                        .camera_boundaries
                        .camera_x_coord_scroll_hi()
                {
                    r4 = r4.wrapping_add(self.OverworldCameraBoundaryCheck(0, 6, vx, 4) as u16);
                }
                ax = ax.wrapping_sub(1);
            }
            self.set_overworld_horizontal_scroll_delta(r4);
            let oi = self.game_state.world.region.overlay_index();
            if oi != 0x97 && oi != 0x9d && r4 != 0 {
                let (subp, mut scroll) = if oi == 0x95 || oi == 0x9e {
                    ((r4 & 3) << 14, r4 >> 2)
                } else {
                    ((r4 & 1) << 15, r4 >> 1)
                };
                if scroll
                    >= if oi == 0x95 || oi == 0x9e {
                        0x3000
                    } else {
                        0x7000
                    }
                {
                    scroll |= 0xf000;
                }
                self.add_bg1_h_live_subpixel(subp, scroll);
            }
        }

        if self.game_state.world.location.overworld_screen_index() != 0x47 {
            if self.game_state.world.region.overlay_index() == 0x9c {
                self.subtract_bg1_v_live_subpixel(0x2000);
                let scroll_delta = self.overworld_vertical_scroll_delta();
                let bg1_v = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg1_v_copy2()
                    .wrapping_add(scroll_delta);
                self.set_bg1_y(bg1_v);
                self.copy_bg2_h_live_to_bg1_h_live();
            } else if self.game_state.world.region.overlay_index() == 0x97
                || self.game_state.world.region.overlay_index() == 0x9d
            {
                self.add_bg1_v_live_subpixel(0x2000, 0);
                self.add_bg1_h_live_subpixel(0x2000, 0);
            }
        }

        if self.game_state.world.location.dungeon_room() == 0x0181 {
            let bg2v = self.game_state.display.ppu_scroll_copy.bg2_v_copy2() | 0x0100;
            self.set_bg1_y(bg2v);
            self.copy_bg2_h_live_to_bg1_h_live();
        }
    }

    pub(super) fn OverworldCameraBoundaryCheck(
        &mut self,
        xa: i32,
        ya: i32,
        vd: i32,
        r8: i32,
    ) -> i32 {
        let ya = (ya >> 1) as usize;
        let r8 = (r8 >> 1) as usize;
        let vertical_axis = xa != 0;
        if self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_copy2_for_axis(vertical_axis)
            == self.game_state.world.room_bounds.packed_bound(ya)
        {
            self.clear_opposed_scroll_counters(ya);
            return 0;
        }
        self.add_bg2_copy2_for_axis_signed(vertical_axis, vd as i16);

        let horizontal = r8 != 0;
        self.add_camera_scroll_for_axis(horizontal, vd as i16);

        let mut value = self
            .game_state
            .world
            .camera_boundaries
            .overworld_scroll_counter_for_axis(ya)
            .wrapping_add(1);
        if (value.wrapping_sub(0x10) as i16) >= 0 {
            value = value.wrapping_sub(0x10);
            self.or_screen_transition_direction_bits(OVERWORLD_SCROLL_DIRECTION_BITS[ya] as u8);
        }
        self.set_opposed_scroll_counter_pair(ya, value);
        vd
    }

    pub(super) fn OverworldScrollTransition(&mut self) -> i32 {
        self.increment_transition_counter();
        let y = self.screen_transition() as usize;
        let d = OVERWORLD_TRANSITION_SCROLL_DELTAS[y];
        let rv;
        if y < 2 {
            self.set_overworld_vertical_scroll_delta_low(d as u8);
            rv = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_add_signed(d);
            self.set_bg2_y(rv);
            if self.game_state.world.location.overworld_screen_index() != 0x1b
                && self.game_state.world.location.overworld_screen_index() != 0x5b
            {
                self.set_bg1_y(rv);
            }
            if self.transition_counter() >= OVERWORLD_TRANSITION_PLAYER_MOVE_FRAMES[y] {
                let link_y = self
                    .game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add_signed(d);
                self.follower_link_state_mut().set_y(link_y);
            }
            if rv
                != self
                    .game_state
                    .world
                    .camera_boundaries
                    .up_down_scroll_target(y)
            {
                return rv as i32;
            }
            if y == 0 {
                let bg2 = self
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .bg2_v_copy2()
                    .wrapping_sub(2);
                self.set_bg2_y(bg2);
            }
            let link_y = self.game_state.player.follower_link.y() & !7;
            self.follower_link_state_mut().set_y(link_y);
            let camera_hi = link_y
                .wrapping_add_signed(OVERWORLD_TRANSITION_CAMERA_OFFSETS[y])
                .wrapping_add(11);
            self.set_camera_scroll_from_link_for_axis(false, camera_hi);
            self.clear_opposed_scroll_counters(0);
        } else {
            self.set_overworld_horizontal_scroll_delta_low(d as u8);
            rv = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add_signed(d);
            self.set_bg2_x(rv);
            if self.game_state.world.location.overworld_screen_index() != 0x1b
                && self.game_state.world.location.overworld_screen_index() != 0x5b
            {
                self.set_bg1_x(rv);
            }
            if self.transition_counter() >= OVERWORLD_TRANSITION_PLAYER_MOVE_FRAMES[y] {
                let link_x = self
                    .game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add_signed(d);
                self.follower_link_state_mut().set_x(link_x);
            }
            if rv
                != self
                    .game_state
                    .world
                    .camera_boundaries
                    .up_down_scroll_target(y)
            {
                return rv as i32;
            }
            let link_x = self.game_state.player.follower_link.x() & !7;
            self.follower_link_state_mut().set_x(link_x);
            let camera_hi = link_x
                .wrapping_add_signed(OVERWORLD_TRANSITION_CAMERA_OFFSETS[y])
                .wrapping_add(11);
            self.set_camera_scroll_from_link_for_axis(true, camera_hi);
            self.clear_opposed_scroll_counters(2);
        }

        let area = ((self.game_state.world.region.current_area_of_player_word() >> 1) as i16)
            + OVERWORLD_ADJACENT_AREA_DELTAS[y];
        self.Overworld_SetCameraBoundaries(
            if self.overworld_is_big_area() { 1 } else { 0 },
            area as i32,
        );
        self.set_flag_overworld_area_changed(1);
        self.increment_submodule();
        self.set_subsubmodule(0);
        self.set_transition_counter(0);
        self.sprite_initialize_slots();
        rv as i32
    }

    pub(super) fn Overworld_FinalizeEntryOntoScreen(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        let transition_dir = self.transition_direction_enum();
        let mut d: i16 = if transition_dir & 1 != 0 { 2 } else { -2 };
        if transition_dir & 2 != 0 {
            let link_x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add_signed(d);
            self.follower_link_state_mut().set_x(link_x);
            d = link_x as i16;
        } else {
            let link_y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add_signed(d);
            self.follower_link_state_mut().set_y(link_y);
            d = link_y as i16;
        }
        if (d & 0x00fe)
            == i16::from(OVERWORLD_ENTRY_SETTLE_COORDINATES[usize::from(transition_dir)])
        {
            self.set_submodule(0);
            self.set_subsubmodule(0);
            let m = self
                .overworld_config_table()
                .music(self.game_state.world.location.overworld_screen_index() as usize);
            self.set_ambient_sound_effect(m >> 4);
            if self.game_state.system_signals.current_music_control() == 0xf1 {
                self.set_music_control(m & 0x0f);
            }
        }
        self.Overworld_OperateCameraScroll();
        if self.has_screen_transition_direction_bits() {
            self.OverworldHandleMapScroll();
        }
    }

    pub(super) fn Overworld_Func1F(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        let transition_dir = self.transition_direction_enum();
        let vel: i8 = if transition_dir & 1 != 0 { 1 } else { -1 };
        if transition_dir & 2 != 0 {
            let link_x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add_signed(vel as i16);
            self.follower_link_state_mut().set_x(link_x);
            self.follower_link_state_mut().set_x_velocity(vel as u8);
        } else {
            let link_y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add_signed(vel as i16);
            self.follower_link_state_mut().set_y(link_y);
            self.follower_link_state_mut().set_y_velocity(vel as u8);
        }
        if self.decrement_overworld_transition_countdown() == 0 {
            self.set_main_module(9);
            self.set_subsubmodule(0);
            self.set_submodule(0);
        }
        self.Overworld_OperateCameraScroll();
    }

    pub(super) fn Module08_02_LoadAndAdvance(&mut self) {
        if self.begin_pre_overworld_screen_build_work() {
            return;
        }
        self.complete_pre_overworld_screen_build();
    }

    pub(super) fn complete_pre_overworld_screen_build(&mut self) {
        self.Overworld_LoadAndBuildScreen();
        self.set_main_module(16);
        self.set_submodule(0);
        self.set_subsubmodule(0);
    }

    pub(super) fn Palette_AnimGetMasterSword2(&mut self) {
        let aux = self
            .game_state
            .display
            .palette_buffer
            .aux_full_slice()
            .to_vec();
        self.backup_overworld_palette_from_tagged(
            &aux,
            crate::game_state::PaletteSliceSource::MirrorBank(zelda3_palette::Bank::Aux),
        );
        for i in 0..256 {
            self.set_aux_color_constant(i, 0x7fff);
        }
        self.copy_color(
            (zelda3_palette::Bank::Main, 0),
            (zelda3_palette::Bank::Main, 32),
        );
        self.set_countdown(0);
        self.set_darkening_or_lightening_screen(2);
        self.increment_subsubmodule();
    }

    pub(super) fn Palette_AnimGetMasterSword(&mut self) {
        if self.game_state.frame.subsubmodule == 0 {
            self.Palette_AnimGetMasterSword2();
            return;
        }

        self.PaletteFilter_BlindingWhite();
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            == 0xff
        {
            for i in 0..8 {
                self.set_main_color_constant(0x58 + i, 0);
                self.set_aux_color_constant(0x58 + i, 0);
            }
            self.set_countdown(0);
            self.set_darkening_or_lightening_screen(0);
            self.set_submodule(0);
        } else {
            self.Palette_AnimGetMasterSword3();
        }
    }

    pub(super) fn Palette_AnimGetMasterSword3(&mut self) {
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            != 0
            || self.game_state.display.palette_filter.countdown() != 31
        {
            return;
        }
        let mapbak = self
            .game_state
            .display
            .palette_buffer
            .overworld_palette_backup()
            .to_vec();
        self.copy_aux_full_from_tagged(
            &mapbak,
            crate::game_state::PaletteSliceSource::MirrorBank(zelda3_palette::Bank::Backup),
        );
        self.set_sub_screen_layers(0);
    }

    pub(super) fn Overworld_Memorize_Map16_Change(&mut self, pos: u16, value: u16) {
        if value == 0x0dc5 || value == 0x0dc9 {
            return;
        }
        self.memorized_tile_mut().append_entry(pos, value);
    }

    fn overworld_bg2_byte_pos(&self, x: u16, y: u16) -> u16 {
        (x.wrapping_sub(self.game_state.world.scroll.overworld_offset_base_x())
            & self.game_state.world.scroll.overworld_offset_mask_x())
            | ((y.wrapping_sub(self.game_state.world.scroll.overworld_offset_base_y())
                & self.game_state.world.scroll.overworld_offset_mask_y())
                << 3)
    }

    pub(super) fn Overworld_ReadTileAttribute(&self, x: u16, y: u16) -> u8 {
        let t = self.overworld_bg2_byte_pos(x, y) as usize;
        let tile = self
            .game_state
            .dungeon
            .room_tilemaps
            .bg2_tile_by_byte_pos(t as u16) as usize;
        self.asset_raw(164)
            .expect("Overworld_ReadTileAttribute missing kSomeTileAttr asset")[tile]
    }

    pub(super) fn overworld_reveal_secret(&mut self, pos: u16) -> u16 {
        self.dungeon_secret_scratch_mut().clear_pending_kind();

        let screen = u16::from(self.game_state.world.location.overworld_screen_index()) as usize;
        if screen >= 0x80 {
            self.adjust_secret_for_powder();
            return 0;
        }

        let secret_offsets = self
            .asset_raw(157)
            .expect("overworld_reveal_secret missing kOverworldSecrets_Offs asset")
            .to_vec();
        let secrets = self
            .asset_raw(158)
            .expect("overworld_reveal_secret missing kOverworldSecrets asset")
            .to_vec();
        let ptr = u16::from(secret_offsets[screen * 2])
            | (u16::from(secret_offsets[screen * 2 + 1]) << 8);
        let mut ptr = ptr as usize;
        loop {
            let x = u16::from(secrets[ptr]) | (u16::from(secrets[ptr + 1]) << 8);
            if x == 0xffff {
                self.adjust_secret_for_powder();
                return 0;
            }
            if x & 0x7fff == pos {
                break;
            }
            ptr += 3;
        }

        let data = secrets[ptr + 2];
        if data != 0 && data < 0x80 {
            self.dungeon_secret_scratch_mut().or_pending_kind(data);
        }
        if data < 0x80 {
            self.adjust_secret_for_powder();
            return 0;
        }

        self.dungeon_secret_scratch_mut().set_pending_kind(0xff);
        if data != 0x84
            && self
                .game_state
                .world
                .overworld
                .event_info
                .event_info(screen)
                & 2
                == 0
        {
            if screen == 0x5b && self.game_state.sprites.follower_runtime.indicator() != 13 {
                self.adjust_secret_for_powder();
                return 0;
            }
            self.set_sound_effect_2(0x1b);
        } else if data == 0x82 && self.game_state.enhanced_features.has(4096) {
            self.set_sound_effect_2(0x1b);
        }

        self.adjust_secret_for_powder();
        OVERWORLD_SECRET_TILE_BY_TYPE[((data & 0x0f) >> 1) as usize]
    }

    fn adjust_secret_for_powder(&mut self) {
        if self.game_state.player.follower_link.item_in_hand_has(0x40) {
            self.dungeon_secret_scratch_mut().set_powder_pending_kind();
        }
    }

    pub(super) fn HandlePegPuzzles(&mut self, pos: u16) {
        if self.game_state.world.location.overworld_screen_index() == 7 {
            if self.game_state.world.overworld.event_info.event_info(7) & 0x20 != 0 {
                return;
            }
            let word = self
                .game_state
                .world
                .transient
                .overworld_peg_puzzle_progress();
            let idx = (word >> 1) as usize;
            if word != 0xffff && LW_TURTLE_ROCK_PEG_POSITIONS[idx] == pos {
                self.set_sound_effect_1_word(0x2d00);
                let next = word.wrapping_add(2);
                self.set_overworld_peg_puzzle_progress(next);
                if next == 6 {
                    self.set_sound_effect_1_word(0x1b00);
                    self.set_overworld_event_bits(7, 0x20);
                    self.set_submodule(47);
                }
            } else {
                self.set_sound_effect_1_word(0x003c);
                self.set_overworld_peg_puzzle_progress(0xffff);
            }
        } else if self.game_state.world.location.overworld_screen_index() == 98 {
            let next = self
                .game_state
                .world
                .transient
                .overworld_peg_puzzle_progress()
                .wrapping_add(1);
            self.set_overworld_peg_puzzle_progress(next);
            if next == 22 {
                self.set_overworld_event_bits(0x62, 0x20);
                self.set_sound_effect_2(27);
                self.dungeon_doors_mut().set_door_open_counter(0x50);
                self.dungeon_object_tracking_mut()
                    .set_big_rock_starting_address(0x0d20);
                self.Overworld_DoMapUpdate32x32_B();
            }
        }
    }

    pub(super) fn GanonTowerEntrance_Func1(&mut self) {
        if self.game_state.frame.subsubmodule == 0 {
            self.set_sound_effect_1(0x2e);
            self.Palette_AnimGetMasterSword2();
        } else {
            self.PaletteFilter_BlindingWhite();
            if self
                .game_state
                .display
                .palette_filter
                .darkening_or_lightening_screen()
                == 0xff
            {
                self.set_countdown(0xff);
                self.increment_subsubmodule();
            } else {
                self.Palette_AnimGetMasterSword3();
            }
        }
    }

    pub(super) fn Overworld_DwDeathMountainPaletteAnimation(&mut self) {
        if self.special_entrance_trigger() != 0 {
            return;
        }
        let sc = self.game_state.world.location.overworld_screen_index();
        if !matches!(sc, 0x43 | 0x45 | 0x47) {
            return;
        }

        let fc = self.game_state.frame.frame_counter;
        if matches!(fc, 5 | 44 | 90) {
            for i in 1..8 {
                for base in [0x30, 0x38, 0x48, 0x70, 0x78] {
                    self.copy_color(
                        (zelda3_palette::Bank::Aux, base + i),
                        (zelda3_palette::Bank::Main, base + i),
                    );
                }
            }
        } else if matches!(fc, 3 | 36 | 88) {
            if fc == 36 {
                self.set_sound_effect_1(54);
            }
            for i in 1..8 {
                self.set_main_color_asset(0x30 + i, DARK_WORLD_PALETTE_ANIMATION_PHASE1[i - 1]);
                self.set_main_color_asset(0x38 + i, DARK_WORLD_PALETTE_ANIMATION_PHASE1[i - 1 + 7]);
                self.set_main_color_asset(
                    0x48 + i,
                    DARK_WORLD_PALETTE_ANIMATION_PHASE1[i - 1 + 14],
                );
                self.set_main_color_asset(
                    0x70 + i,
                    DARK_WORLD_PALETTE_ANIMATION_PHASE1[i - 1 + 21],
                );
                self.set_main_color_asset(
                    0x78 + i,
                    DARK_WORLD_PALETTE_ANIMATION_PHASE1[i - 1 + 28],
                );
            }
        }

        self.increment_cgram_update_flag();
        let mut yy = 32usize;
        if sc == 0x43 || sc == 0x45 {
            if self.game_state.world.overworld.event_info.event_info(0x43) & 0x20 != 0 {
                return;
            }
            yy = ((self.game_state.frame.frame_counter & 0x0c) as usize) * 2;
        }
        for i in 0..8 {
            self.set_main_color_asset(0x68 + i, DARK_WORLD_PALETTE_ANIMATION_PHASE2[yy + i]);
        }
    }

    pub(super) fn Module09_FadeBackInFromMosaic(&mut self) {
        self.Overworld_ResetMosaicDown();
        match self.game_state.frame.subsubmodule {
            0 => {
                let sc = self.game_state.world.location.overworld_screen_index() as usize;
                let bg = self.GetOverworldBgPalette(sc as u8);
                let spr = self.overworld_config_table().sprite_palette(sc);
                self.Overworld_LoadPalettes(bg, spr);
                self.OverworldMosaicTransition_LoadSpriteGraphicsAndSetMosaic();
            }
            1 => {
                self.Graphics_IncrementalVRAMUpload();
                self.ApplyPaletteFilter_bounce();
            }
            _ => {
                self.save_current_music_as_last();
                if self.game_state.world.location.overworld_screen_index() != 0x80
                    && self.game_state.world.location.overworld_screen_index() != 0x2a
                {
                    let m = self
                        .overworld_config_table()
                        .music(self.game_state.world.location.overworld_screen_index() as usize);
                    self.set_ambient_sound_effect(if m >> 4 != 0 { m >> 4 } else { 5 });
                    if !self.zelda_is_playing_music_track(m & 0x0f) {
                        self.set_music_control(m & 0x0f);
                    }
                }
                self.set_submodule(8);
                self.set_subsubmodule(0);
                if self.game_state.frame.main_module == 11 {
                    self.set_main_module(9);
                    self.set_submodule(31);
                    self.set_overworld_transition_countdown(12);
                }
            }
        }
    }

    pub(super) fn Overworld_Func1C(&mut self) {
        self.Overworld_ResetMosaicDown();
        match self.game_state.frame.subsubmodule {
            0 => self.OverworldMosaicTransition_LoadSpriteGraphicsAndSetMosaic(),
            1 => {
                self.Graphics_IncrementalVRAMUpload();
                self.ApplyPaletteFilter_bounce();
            }
            _ => {
                if self.game_state.world.location.overworld_screen_index() < 0x80 {
                    let music_control =
                        if self.game_state.world.location.overworld_screen_index() & 0x3f != 0 {
                            2
                        } else {
                            5
                        };
                    self.set_music_control(music_control);
                }
                self.set_submodule(8);
                self.set_subsubmodule(0);
            }
        }
    }

    pub(super) fn Overworld_StartMosaicTransition(&mut self) {
        self.ConditionalMosaicControl();
        match self.game_state.frame.subsubmodule {
            0 => {
                if self.game_state.world.location.overworld_screen_index() != 0x80 {
                    let music = self
                        .overworld_config_table()
                        .music(self.game_state.world.location.overworld_screen_index() as usize);
                    if !self.zelda_is_playing_music_track(music & 0x0f) {
                        self.set_music_control(0xf1);
                    }
                }
                self.ResetTransitionPropsAndAdvance_ResetInterface();
            }
            1 => self.ApplyPaletteFilter_bounce(),
            _ => {
                self.set_screen_brightness(0x80);
                self.set_subsubmodule(0);
                if u16::from(self.game_state.world.location.overworld_screen_index()) & 0x3f == 0 {
                    self.DecodeAnimatedSpriteTile_variable(0x1e);
                }
                if self.game_state.world.region.overworld_area_index() != 0
                    && self.game_state.frame.main_module != 11
                {
                    self.set_main_screen_layers(0x16);
                    self.set_sub_screen_layers(1);
                    self.set_color_window_selection(0x82);
                    self.set_color_math_control(0x20);
                    self.increment_submodule();
                    return;
                }
                if self.game_state.frame.submodule == 36 {
                    self.LoadOverworldFromSpecialOverworld();
                    if u16::from(self.game_state.world.location.overworld_screen_index()) & 0x3f
                        == 0
                    {
                        self.DecodeAnimatedSpriteTile_variable(0x1e);
                    }
                }
                self.increment_submodule();
            }
        }
    }

    pub(super) fn OverworldMosaicTransition_LoadSpriteGraphicsAndSetMosaic(&mut self) {
        self.LoadNewSpriteGFXSet();
        self.set_screen_brightness(0x0f);
        self.set_hdma_enable_mask(0x80);
        let countdown = self.game_state.display.mosaic_target_level.wrapping_sub(1);
        self.set_countdown(countdown);
        self.clear_mosaic_target_level();
        self.set_darkening_or_lightening_screen(2);
        self.increment_subsubmodule();
    }

    pub(super) fn Overworld_SetSongList(&mut self) {
        let mut r0 = 2;
        let mut y = 0xc0usize;
        if self.game_state.inventory.save_progress.progress_indicator() < 3 {
            y = 0x80;
            if self.game_state.inventory.items.sword_type() < 2 {
                r0 = 5;
                y = 0x40;
                if self.game_state.inventory.save_progress.progress_indicator() < 2 {
                    y = 0;
                }
            }
        }
        let music_sets = self
            .asset_raw(111)
            .expect("Overworld_SetSongList missing kOwMusicSets asset")
            .to_vec();
        self.copy_overworld_music_primary(&music_sets[y..y + 64]);
        let music_sets2 = self
            .asset_raw(112)
            .expect("Overworld_SetSongList missing kOwMusicSets2 asset")
            .to_vec();
        self.copy_overworld_music_secondary(&music_sets2);
        self.set_overworld_music(128, r0);
    }

    pub(super) fn Overworld_Func2F(&mut self) {
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile_by_byte_pos(0x0720 as u16, 0x0212);
        self.Overworld_Memorize_Map16_Change(0x0720, 0x0212);
        self.overworld_draw_map16(0x0720, 0x0212);
        self.set_bg_vram_load_mode(1);
        self.set_submodule(0);
    }

    pub(super) fn OpenGargoylesDomain(&mut self) {
        self.overworld_draw_map16_persist(0x0d3e, 0x0e1b);
        self.overworld_draw_map16_persist(0x0d40, 0x0e1c);
        self.overworld_draw_map16_persist(0x0dbe, 0x0e1d);
        self.overworld_draw_map16_persist(0x0dc0, 0x0e1e);
        self.overworld_draw_map16_persist(0x0e3e, 0x0e1f);
        self.overworld_draw_map16_persist(0x0e40, 0x0e20);
        self.set_overworld_event_bits(0x58, 0x20);
        self.set_sound_effect_2(0x1b);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn CreatePyramidHole(&mut self) {
        self.overworld_draw_map16_persist(0x03bc, 0x0e3f);
        self.overworld_draw_map16_persist(0x03be, 0x0e40);
        self.overworld_draw_map16_persist(0x03c0, 0x0e41);
        self.overworld_draw_map16_persist(0x043c, 0x0e42);
        self.overworld_draw_map16_persist(0x043e, 0x0e43);
        self.overworld_draw_map16_persist(0x0440, 0x0e44);
        self.overworld_draw_map16_persist(0x04bc, 0x0e45);
        self.overworld_draw_map16_persist(0x04be, 0x0e46);
        self.overworld_draw_map16_persist(0x04c0, 0x0e47);
        self.set_ambient_sound_effect_word(0x3515);
        self.set_overworld_event_bits(0x5b, 0x20);
        self.set_sound_effect_2(3);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn Overworld_AlterTileHardcore(&mut self, pos: u16, value: u16) {
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile_by_byte_pos(pos, value);
        self.overworld_draw_map16(pos, value);
    }

    pub(super) fn Overworld_CheckSpecialSwitchArea(&mut self) {
        let map8 = self.Overworld_GetMap16OfLink_Mult8();
        let a = map8[0] & 0x01ff;
        for i in (0..4).rev() {
            if SPECIAL_SWITCH_AREA_TILE_IDS[i] == a
                && SPECIAL_SWITCH_AREA_SCREENS[i]
                    == u16::from(self.game_state.world.location.overworld_screen_index())
            {
                self.set_dungeon_room(SPECIAL_SWITCH_AREA_EXITS[i]);
                let direction = SPECIAL_SWITCH_AREA_DIRECTIONS[i];
                // C sets BOTH ow_screen_transition_dir_bits (0x410) and ..._2 (0x416). The port was
                // missing the 0x410 write (it only called set_screen_transition_direction_bits =
                // 0x416), so a special-switch overworld exit left 0x410 stale (0 instead of the
                // transition direction) — cascading into the entire post-frame-241475 divergence.
                self.set_edge_transition_direction_bits(direction);
                self.set_screen_transition_direction_bits(direction);
                self.follower_link_state_mut().set_direction(direction);
                let trans = self.DirToEnum(direction as i32) as u16;
                self.set_screen_transition_word(trans);
                self.set_transition_direction_enum(trans as u8);
                self.set_submodule(23);
                self.set_main_module(11);
                break;
            }
        }
    }

    pub(super) fn ScrollAndCheckForSOWExit(&mut self) {
        if self.has_screen_transition_direction_bits() {
            self.OverworldHandleMapScroll();
        }

        let map8 = self.Overworld_GetMap16OfLink_Mult8();
        let a = map8[0] & 0x01ff;
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some()
            && matches!(
                u16::from(self.game_state.world.location.overworld_screen_index()),
                0x0080 | 0x0081
            )
        {
            let xc = self.game_state.player.follower_link.x().wrapping_add(8) >> 3;
            let yc = self.game_state.player.follower_link.y().wrapping_add(12);
            let pos = self.overworld_bg2_byte_pos(xc, yc);
            println!(
                "spexit-check frame={} screen=0x{:04x} x=0x{:04x} y=0x{:04x} base=0x{:04x}/0x{:04x} mask=0x{:04x}/0x{:04x} pos=0x{:04x} map8=0x{:04x} dirbits2=0x{:02x} sub={} subsub={}",
                self.game_state.frame.frame_counter,
                u16::from(self.game_state.world.location.overworld_screen_index()),
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
                self.game_state.world.scroll.overworld_offset_base_x(),
                self.game_state.world.scroll.overworld_offset_base_y(),
                self.game_state.world.scroll.overworld_offset_mask_x(),
                self.game_state.world.scroll.overworld_offset_mask_y(),
                pos,
                a,
                self.screen_transition_direction_bits(),
                self.game_state.frame.submodule,
                self.game_state.frame.subsubmodule,
            );
        }
        for i in (0..3).rev() {
            if SPECIAL_SWITCH_AREA_B_TILE_IDS[i] == a
                && SPECIAL_SWITCH_AREA_B_SCREENS[i]
                    == u16::from(self.game_state.world.location.overworld_screen_index())
            {
                let direction = SPECIAL_SWITCH_AREA_B_DIRECTIONS[i];
                self.follower_link_state_mut().set_direction(direction);
                let trans = self.DirToEnum(direction as i32) as u16;
                self.set_screen_transition_word(trans);
                self.set_transition_direction_enum(trans as u8);
                self.set_submodule(36);
                self.set_subsubmodule(0);
                self.set_dungeon_room_index(0);
                if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some() {
                    println!(
                        "spexit-hit frame={} i={} screen=0x{:04x} map8=0x{:04x} dir=0x{:02x} trans=0x{:04x} x=0x{:04x} y=0x{:04x}",
                        self.game_state.frame.frame_counter,
                        i,
                        u16::from(self.game_state.world.location.overworld_screen_index()),
                        a,
                        self.game_state.player.follower_link.direction(),
                        self.screen_transition_word(),
                        self.game_state.player.follower_link.x(),
                        self.game_state.player.follower_link.y(),
                    );
                }
                break;
            }
        }
    }

    pub(super) fn Overworld_GetMap16OfLink_Mult8(&self) -> [u16; 4] {
        let xc = self.game_state.player.follower_link.x().wrapping_add(8) >> 3;
        let yc = self.game_state.player.follower_link.y().wrapping_add(12);
        let pos = self.overworld_bg2_byte_pos(xc, yc);
        let map16 = self
            .game_state
            .dungeon
            .room_tilemaps
            .bg2_tile_by_byte_pos(pos) as usize
            * 4;
        let map8 = self
            .asset_raw(70)
            .expect("Overworld_GetMap16OfLink_Mult8 missing kMap16ToMap8 asset");
        [
            read_word_from_slice(map8, map16 * 2),
            read_word_from_slice(map8, (map16 + 1) * 2),
            read_word_from_slice(map8, (map16 + 2) * 2),
            read_word_from_slice(map8, (map16 + 3) * 2),
        ]
    }

    pub(super) fn overworld_get_link_map16_coords(&self, xy: &mut Point16U) -> u16 {
        let (pos, x, y) = self.overworld_get_link_map16_coords_result();
        xy.x = x;
        xy.y = y;
        pos
    }

    pub(super) fn overworld_smash_rock_pile(
        &mut self,
        down_one_tile: bool,
        pt: &mut Point16U,
    ) -> i32 {
        if let Some((attr, x, y)) = self.overworld_smash_rock_pile_result(down_one_tile) {
            pt.x = x;
            pt.y = y;
            attr as i32
        } else {
            -1
        }
    }

    pub(super) fn overworld_lifting_small_obj(
        &mut self,
        a: u16,
        pos: u16,
        y: u16,
        pt: Point16U,
    ) -> u8 {
        self.overworld_lifting_small_obj_impl(a, pos, y, pt.x, pt.y)
    }

    pub(super) fn smash_rock_pile_from_lift(
        &mut self,
        a: u16,
        pos: u16,
        y: u16,
        pt: Point16U,
    ) -> u8 {
        self.smash_rock_pile_from_lift_impl(a, pos, y as usize, pt.x, pt.y)
    }

    pub(super) fn sprite_load_graphics_properties_light_world_only(&mut self) {
        let i = if self.game_state.inventory.save_progress.progress_indicator() < 2 {
            0
        } else if self.game_state.inventory.save_progress.progress_indicator() != 3 {
            1
        } else {
            2
        };
        let gfx = self
            .asset_raw(161)
            .expect(
                "Sprite_LoadGraphicsProperties_light_world_only missing kOverworldSpriteGfx asset",
            )
            .to_vec();
        let palettes = self
            .asset_raw(162)
            .expect(
                "Sprite_LoadGraphicsProperties_light_world_only missing kOverworldSpritePalettes asset",
            )
            .to_vec();
        self.copy_overworld_sprite_graphics_range(0, &gfx, i * 64, 64);
        self.copy_overworld_sprite_palette_range(0, &palettes, i * 64, 64);
    }

    pub(super) fn sprite_load_graphics_properties(&mut self) {
        let gfx = self
            .asset_raw(161)
            .expect("Sprite_LoadGraphicsProperties missing kOverworldSpriteGfx asset")
            .to_vec();
        let palettes = self
            .asset_raw(162)
            .expect("Sprite_LoadGraphicsProperties missing kOverworldSpritePalettes asset")
            .to_vec();
        self.copy_overworld_sprite_graphics_range(64, &gfx, 0xc0, 64);
        self.copy_overworld_sprite_palette_range(64, &palettes, 0xc0, 64);
        self.sprite_load_graphics_properties_light_world_only();
    }
}

impl ZeldaState {
    pub(super) fn decompress_enemy_damage_subclasses(&mut self) {
        let data = self
            .asset_raw(56)
            .expect("decompress_enemy_damage_subclasses missing kEnemyDamageData asset")
            .to_vec();
        self.copy_overworld_map16_decode_source_from(&data);
        let packed_damage =
            self.graphics_primary_decompression_buffer(PACKED_ENEMY_DAMAGE_SOURCE_BYTES);
        self.enemy_damage_subclass_table_mut()
            .load_from_packed_nibbles(&packed_damage);
    }

    pub(super) fn conditional_mosaic_control(&mut self) {
        if self.game_state.display.palette_filter.countdown() & 1 != 0 {
            self.increment_mosaic_level_by(0x10);
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(7);
    }

    pub(super) fn overworld_alter_weathervane(&mut self) {
        self.dungeon_doors_mut().set_door_open_counter(0x68);
        self.dungeon_object_tracking_mut()
            .set_big_rock_starting_address(0x0c3e);
        self.overworld_do_map_update32x32_b();
        self.overworld_draw_map16_persist(0x0c42, 0x0e21);
        self.overworld_draw_map16_persist(0x0cc2, 0x0e25);

        self.set_overworld_event_bits(0x18, 0x20);
        self.set_bg_vram_load_mode(1);
    }

    fn overworld_do_map_update32x32_b(&mut self) {
        self.overworld_do_map_update32x32();
        self.dungeon_doors_mut().clear_door_open_counter_low();
    }

    pub(super) fn Overworld_DoMapUpdate32x32_B(&mut self) {
        self.overworld_do_map_update32x32_b();
    }

    pub(super) fn Overworld_DoMapUpdate32x32_conditional(&mut self) {
        if self.game_state.dungeon.doors.door_open_counter_low() & 7 != 0 {
            self.dungeon_doors_mut().increment_door_open_counter_low();
        } else {
            self.overworld_do_map_update32x32();
        }
    }

    pub(super) fn Module09_09_OpenBigDoorFromExiting(&mut self) {
        if self.game_state.world.transient.door_animation_step() != 3 {
            self.Overworld_DoMapUpdate32x32_conditional();
            return;
        }
        self.set_overworld_transition_countdown(36);
        self.clear_screen_transition_direction_bits();
        self.increment_submodule();
    }

    pub(super) fn Module09_0C_OpenBigDoor(&mut self) {
        if self.game_state.world.transient.door_animation_step() != 3 {
            self.Overworld_DoMapUpdate32x32_conditional();
            return;
        }
        self.set_submodule(0);
        self.set_subsubmodule(0);
        self.clear_screen_transition_direction_bits();
    }

    pub(super) fn Module09_0A_WalkFromExiting_FacingDown(&mut self) {
        self.follower_link_state_mut().set_last_direction(4);
        self.link_handle_moving_animation_full_long_entry();
        let link_y = self.game_state.player.follower_link.y().wrapping_add(1);
        self.follower_link_state_mut().set_y(link_y);
        if self.decrement_overworld_transition_countdown() != 0 {
            return;
        }
        self.set_submodule(0);
        let link_y = self.game_state.player.follower_link.y().wrapping_add(3);
        self.follower_link_state_mut().set_y(link_y);
        self.follower_link_state_mut().set_y_velocity(3);
        self.Overworld_OperateCameraScroll();
        if self.has_screen_transition_direction_bits() {
            self.OverworldHandleMapScroll();
        }
    }

    pub(super) fn Module09_0B_WalkFromExiting_FacingUp(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        let link_y = self.game_state.player.follower_link.y().wrapping_sub(1);
        self.follower_link_state_mut().set_y(link_y);
        if self.decrement_overworld_transition_countdown() == 0 {
            self.set_submodule(0);
        }
    }

    fn overworld_do_map_update32x32(&mut self) {
        let i = self.game_state.memorized_tiles.count() as usize;
        let j = (self.game_state.dungeon.doors.door_open_counter() >> 1) as usize;
        let base = self
            .game_state
            .dungeon
            .object_tracking
            .big_rock_starting_address();
        let entries = [
            (base, OVERWORLD_DOOR_ANIM_TILES[j]),
            (base.wrapping_add(2), OVERWORLD_DOOR_ANIM_TILES[j + 1]),
            (base.wrapping_add(0x80), OVERWORLD_DOOR_ANIM_TILES[j + 2]),
            (base.wrapping_add(0x82), OVERWORLD_DOOR_ANIM_TILES[j + 3]),
        ];
        for (n, (pos, tile)) in entries.into_iter().enumerate() {
            self.memorized_tile_mut().set_entry_addr(i + n * 2, pos);
            self.memorized_tile_mut().set_entry_value(i + n * 2, tile);
            self.overworld_draw_map16_persist(pos, tile);
        }
        let upload = self.game_state.display.vram_upload_cursor_usize();
        self.write_vram_upload_buffer_word(upload, 0xffff);
        self.memorized_tile_mut().set_count((i + 8) as u16);
        let step = self
            .game_state
            .world
            .transient
            .door_animation_step()
            .wrapping_add(if self.game_state.dungeon.doors.door_open_counter() == 32 {
                2
            } else {
                1
            });
        self.set_door_animation_step_word(step);
        self.set_bg_vram_load_mode(1);
        self.dungeon_doors_mut().increment_door_open_counter_low();
    }

    fn overworld_draw_map16_persist(&mut self, pos: u16, value: u16) {
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile_by_byte_pos(pos, value);
        self.overworld_draw_map16(pos, value);
    }

    fn overworld_draw_map16(&mut self, pos: u16, value: u16) {
        let vram_pos = Self::overworld_find_map16_vram_address(pos);
        let dst = self.game_state.display.current_vram_upload_data_address();
        let src = value as usize * 4;
        let map8 = self
            .asset_raw(70)
            .expect("Overworld_DrawMap16 missing kMap16ToMap8 asset");
        let tile0 = u16::from(map8[src * 2]) | (u16::from(map8[src * 2 + 1]) << 8);
        let tile1 = u16::from(map8[(src + 1) * 2]) | (u16::from(map8[(src + 1) * 2 + 1]) << 8);
        let tile2 = u16::from(map8[(src + 2) * 2]) | (u16::from(map8[(src + 2) * 2 + 1]) << 8);
        let tile3 = u16::from(map8[(src + 3) * 2]) | (u16::from(map8[(src + 3) * 2 + 1]) << 8);
        self.write_vram_upload_map16_update_packet(dst, vram_pos, [tile0, tile1, tile2, tile3]);
        self.advance_vram_upload_cursor_by(16);
    }

    fn overworld_find_map16_vram_address(addr: u16) -> u16 {
        (if addr & 0x3f >= 0x20 { 0x0400 } else { 0 })
            + (if addr & 0x0fff >= 0x0800 { 0x0800 } else { 0 })
            + (addr & 0x001f)
            + ((addr & 0x0780) >> 1)
    }

    pub(super) fn overworld_bomb_tiles32x32(&mut self, mut x: u16, mut y: u16) {
        x = x.wrapping_sub(23) & !7;
        y = y.wrapping_sub(20) & !7;

        for _ in (1..=3).rev() {
            let mut xt = x;
            for _ in (1..=3).rev() {
                self.overworld_bomb_tile(xt, y);
                xt = xt.wrapping_add(16);
            }
            y = y.wrapping_add(16);
        }
        self.set_overworld_bomb_tile_sweep_x(x);
        self.set_overworld_bomb_tile_sweep_y_end(y);
    }

    fn overworld_bomb_tile(&mut self, x: u16, y: u16) {
        let pos = self.overworld_bg2_byte_pos(x >> 3, y);

        if self.game_state.sprites.follower_runtime.indicator() != 13 {
            let a = self
                .game_state
                .dungeon
                .room_tilemaps
                .bg2_tile_by_byte_pos(pos);
            let (k, j) = if a == 0x0036 {
                (2, 0x0dc7)
            } else if a == 0x072a {
                (4, 0x0dc8)
            } else if a == 0x037e {
                (3, 0x0dc5)
            } else {
                self.overworld_bomb_tile_label_a(pos);
                return;
            };
            let mut a = self.overworld_reveal_secret(pos);
            if a == 0 {
                a = j;
            }
            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile_by_byte_pos(pos, a);
            self.overworld_memorize_map16_change_for_smash(pos, a);
            self.overworld_draw_map16_for_smash(pos, a);
            self.sprite_spawn_immediately_smashed_terrain(k, x & !7, y & !7);
            self.set_bg_vram_load_mode(1);
            return;
        }

        self.overworld_bomb_tile_label_a(pos);
    }

    fn overworld_bomb_tile_label_a(&mut self, pos: u16) {
        let a = self.overworld_reveal_secret(pos);
        if a == 0x0db4 {
            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile_by_byte_pos(pos, a);
            self.overworld_memorize_map16_change_for_smash(pos, a);
            self.overworld_draw_map16_for_smash(pos, a);

            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile_by_byte_pos(pos.wrapping_add(2), 0x0db5);
            self.overworld_memorize_map16_change_for_smash(pos, 0x0db5);
            self.overworld_draw_map16_for_smash(pos.wrapping_add(2), 0x0db5);
            self.set_bg_vram_load_mode(1);
            let screen = self.game_state.world.location.overworld_screen_index() as usize;
            self.set_overworld_event_bits(screen, 2);
        }
    }

    pub(super) fn Overworld_HandleOverlaysAndBombDoors(&mut self) {
        let screen = u16::from(self.game_state.world.location.overworld_screen_index()) as usize;
        if screen == 0x33 {
            self.dungeon_room_tilemaps_mut().set_bg2_tile(340, 0x020f);
        } else if screen == 0x2f {
            self.dungeon_room_tilemaps_mut().set_bg2_tile(1497, 0x020f);
        }

        let screen_byte = self.game_state.world.location.overworld_screen_index() as usize;
        if screen_byte < 0x80
            && self
                .game_state
                .world
                .overworld
                .event_info
                .event_info(screen_byte)
                & 0x20
                != 0
        {
            self.Overworld_LoadEventOverlay();
        }
        if self
            .game_state
            .world
            .overworld
            .event_info
            .event_info(screen_byte)
            & 2
            != 0
        {
            let pos = (SECONDARY_OVERLAY_BY_OVERWORLD_SCREEN[screen] >> 1) as usize;
            self.replay_trace_door_overlay("secondary-before", (pos << 1) as u16);
            self.dungeon_room_tilemaps_mut().set_bg2_tile(pos, 0x0db4);
            self.dungeon_room_tilemaps_mut()
                .set_bg2_tile(pos + 1, 0x0db5);
            self.replay_trace_door_overlay("secondary-after", (pos << 1) as u16);
        }
    }

    pub(super) fn Overworld_LoadEventOverlay(&mut self) {
        match u16::from(self.game_state.world.location.overworld_screen_index()) {
            0..=2 => {
                for x in 11..=14 {
                    self.write_bg2_xy(x, 16, 0x0e32);
                }
                self.write_bg2_xy(11, 17, 0x0e32);
                self.write_bg2_xy(14, 17, 0x0e32);
                self.write_bg2_xy(12, 17, 0x0e33);
                self.write_bg2_xy(13, 17, 0x0e34);
                self.write_event_overlay_pairs(&[
                    (11, 18, 0x0e35),
                    (12, 18, 0x0e36),
                    (13, 18, 0x0e37),
                    (14, 18, 0x0e38),
                    (11, 19, 0x0e39),
                    (12, 19, 0x0e3a),
                    (13, 19, 0x0e3b),
                    (14, 19, 0x0e3c),
                    (12, 20, 0x0e3d),
                    (13, 20, 0x0e3e),
                ]);
            }
            3..=7 => self.write_bg2_xy(16, 14, 0x0212),
            8..=19 => self.write_event_overlay_2x2(3, 10),
            20 => self.write_event_overlay_pairs(&[
                (25, 10, 0x0dd1),
                (26, 10, 0x0dd2),
                (25, 11, 0x0dd7),
                (26, 11, 0x0dd8),
                (25, 12, 0x0dd9),
                (26, 12, 0x0dda),
            ]),
            21..=25 | 32 | 33 => self.write_event_overlay_pairs(&[
                (31, 24, 0x0e21),
                (33, 24, 0x0e21),
                (32, 24, 0x0e22),
                (31, 25, 0x0e23),
                (32, 25, 0x0e24),
                (33, 25, 0x0e25),
            ]),
            26..=28 | 35 | 36 => self.write_event_overlay_pairs(&[
                (30, 39, 0x0dc1),
                (31, 39, 0x0dc2),
                (30, 40, 0x0dbe),
                (31, 40, 0x0dbf),
                (32, 39, 0x0dc2),
                (33, 39, 0x0dc3),
                (32, 40, 0x0dbf),
                (33, 40, 0x0dc0),
            ]),
            29..=31 | 34 | 37..=43 | 107 => self.write_event_overlay_2x2(24, 6),
            44..=49 | 56 | 57 => self.write_event_overlay_2x2(44, 6),
            50..=55 | 119 => self.write_event_overlay_2x2(6, 8),
            58 => self.write_event_overlay_2x2(15, 20),
            59 | 123 => self.write_event_overlay_pairs(&[
                (22, 7, 0x0ddf),
                (18, 8, 0x0ddf),
                (16, 9, 0x0ddf),
                (15, 10, 0x0ddf),
                (14, 12, 0x0ddf),
                (26, 14, 0x0ddf),
                (23, 7, 0x0de0),
                (17, 9, 0x0de0),
                (24, 7, 0x0de1),
                (28, 8, 0x0de1),
                (29, 9, 0x0de1),
                (21, 11, 0x0de1),
                (29, 14, 0x0de1),
                (19, 8, 0x0de2),
                (20, 8, 0x0de2),
                (21, 8, 0x0de2),
                (25, 8, 0x0de2),
                (26, 8, 0x0de2),
                (27, 8, 0x0de2),
                (22, 8, 0x0de3),
                (18, 9, 0x0de3),
                (16, 10, 0x0de3),
                (15, 12, 0x0de3),
                (23, 8, 0x0de4),
                (19, 9, 0x0de4),
                (20, 9, 0x0de4),
                (24, 9, 0x0de4),
                (27, 9, 0x0de4),
                (17, 10, 0x0de4),
                (18, 10, 0x0de4),
                (19, 10, 0x0de4),
                (28, 10, 0x0de4),
                (16, 11, 0x0de4),
                (17, 11, 0x0de4),
                (18, 11, 0x0de4),
                (19, 11, 0x0de4),
                (16, 12, 0x0de4),
                (17, 12, 0x0de4),
                (15, 13, 0x0de4),
                (16, 13, 0x0de4),
                (15, 14, 0x0de4),
                (16, 14, 0x0de4),
                (19, 16, 0x0de4),
                (19, 17, 0x0de4),
                (20, 17, 0x0de4),
                (19, 18, 0x0de4),
                (24, 8, 0x0de5),
                (28, 9, 0x0de5),
                (20, 11, 0x0de5),
                (21, 12, 0x0de5),
                (21, 9, 0x0de6),
                (25, 9, 0x0de6),
                (20, 10, 0x0de6),
                (28, 11, 0x0de6),
                (21, 17, 0x0de6),
                (20, 18, 0x0de6),
                (22, 9, 0x0de7),
                (24, 10, 0x0de7),
                (15, 15, 0x0de7),
                (16, 15, 0x0de7),
                (19, 19, 0x0de7),
                (28, 19, 0x0de7),
                (23, 9, 0x0de8),
                (26, 9, 0x0de8),
                (27, 10, 0x0de8),
                (17, 15, 0x0de8),
                (18, 16, 0x0de8),
                (23, 10, 0x0de9),
                (26, 10, 0x0de9),
                (14, 15, 0x0de9),
                (17, 16, 0x0de9),
                (26, 18, 0x0de9),
                (27, 19, 0x0de9),
                (29, 10, 0x0dea),
                (28, 12, 0x0dea),
                (28, 13, 0x0dea),
                (29, 18, 0x0dea),
                (15, 11, 0x0deb),
                (27, 11, 0x0deb),
                (27, 12, 0x0deb),
                (14, 13, 0x0deb),
                (27, 13, 0x0deb),
                (14, 14, 0x0deb),
                (18, 17, 0x0deb),
                (18, 18, 0x0deb),
                (18, 12, 0x0dec),
                (17, 13, 0x0dec),
                (19, 12, 0x0ded),
                (20, 12, 0x0dee),
                (18, 13, 0x0def),
                (27, 15, 0x0def),
                (19, 13, 0x0df0),
                (19, 14, 0x0df0),
                (20, 14, 0x0df0),
                (21, 14, 0x0df0),
                (21, 15, 0x0df0),
                (27, 16, 0x0df0),
                (28, 16, 0x0df0),
                (20, 13, 0x0df1),
                (28, 15, 0x0df1),
                (21, 13, 0x0df2),
                (17, 14, 0x0df3),
                (18, 15, 0x0df3),
                (20, 16, 0x0df3),
                (18, 14, 0x0df4),
                (19, 15, 0x0df5),
                (20, 15, 0x0df6),
                (27, 17, 0x0df6),
                (26, 15, 0x0df7),
                (29, 15, 0x0df8),
                (21, 16, 0x0df9),
                (26, 16, 0x0dfa),
                (29, 16, 0x0dfb),
                (26, 17, 0x0dfc),
                (28, 17, 0x0dfd),
                (29, 17, 0x0dfe),
                (27, 18, 0x0dff),
                (28, 18, 0x0e00),
                (21, 10, 0x0e01),
                (25, 10, 0x0e01),
                (21, 18, 0x0e01),
                (29, 11, 0x0e02),
                (20, 19, 0x0e02),
                (29, 19, 0x0e02),
                (18, 19, 0x0e03),
                (27, 14, 0x0e04),
                (28, 14, 0x0e05),
            ]),
            60..=65 | 72 | 73 => self.write_event_overlay_pairs(&[
                (8, 11, 0x0e13),
                (11, 11, 0x0e14),
                (8, 12, 0x0e15),
                (9, 12, 0x0e16),
                (10, 12, 0x0e17),
                (11, 12, 0x0e18),
                (9, 13, 0x0e19),
                (10, 13, 0x0e1a),
                (9, 16, 0x0e06),
                (10, 16, 0x0e06),
                (8, 14, 0x0e07),
                (8, 15, 0x0e07),
                (9, 14, 0x0e08),
                (9, 15, 0x0e08),
                (10, 14, 0x0e09),
                (10, 15, 0x0e09),
                (11, 14, 0x0e0a),
                (11, 15, 0x0e0a),
            ]),
            66..=68 | 75 | 76 => self.write_event_overlay_pairs(&[
                (47, 8, 0x0e96),
                (48, 8, 0x0e97),
                (47, 9, 0x0e9c),
                (47, 10, 0x0e9c),
                (48, 9, 0x0e9d),
                (48, 10, 0x0e9d),
                (47, 11, 0x0e9a),
                (48, 11, 0x0e9b),
            ]),
            69 | 70 | 77 | 78 => self.write_event_overlay_2x2(52, 16),
            71 => self.write_event_overlay_pairs(&[
                (15, 19, 0x0e78),
                (16, 19, 0x0e79),
                (17, 19, 0x0e7a),
                (18, 19, 0x0e7b),
                (15, 20, 0x0e7c),
                (16, 20, 0x0e7d),
                (17, 20, 0x0e7e),
                (18, 20, 0x0e7f),
                (15, 21, 0x0e80),
                (16, 21, 0x0e81),
                (17, 21, 0x0e82),
                (18, 21, 0x0e83),
                (15, 22, 0x0e84),
                (16, 22, 0x0e85),
                (17, 22, 0x0e86),
                (18, 22, 0x0e87),
            ]),
            74 | 79..=89 | 96 | 97 => self.write_event_overlay_pairs(&[
                (31, 26, 0x0e1b),
                (32, 26, 0x0e1c),
                (31, 27, 0x0e1d),
                (32, 27, 0x0e1e),
                (31, 28, 0x0e1f),
                (32, 28, 0x0e20),
            ]),
            90..=92 | 99 | 100 => self.write_event_overlay_pairs(&[
                (30, 7, 0x0e3f),
                (31, 7, 0x0e40),
                (32, 7, 0x0e41),
                (30, 8, 0x0e42),
                (31, 8, 0x0e43),
                (32, 8, 0x0e44),
                (30, 9, 0x0e45),
                (31, 9, 0x0e46),
                (32, 9, 0x0e47),
            ]),
            93..=95 | 102 | 103 => self.write_event_overlay_pairs(&[
                (51, 3, 0x0e31),
                (53, 4, 0x0e2d),
                (53, 5, 0x0e2e),
                (53, 6, 0x0e2f),
            ]),
            98 => self.write_event_overlay_2x2(16, 26),
            101 | 104..=113 | 120 | 121 => self.write_event_overlay_pairs(&[
                (17, 10, 0x0e64),
                (18, 10, 0x0e65),
                (19, 10, 0x0e66),
                (20, 10, 0x0e67),
                (17, 11, 0x0e68),
                (18, 11, 0x0e69),
                (19, 11, 0x0e6a),
                (20, 11, 0x0e6b),
                (17, 12, 0x0e6c),
                (18, 12, 0x0e6d),
                (19, 12, 0x0e6e),
                (20, 12, 0x0e6f),
                (17, 13, 0x0e70),
                (18, 13, 0x0e71),
                (19, 13, 0x0e72),
                (20, 13, 0x0e73),
                (17, 14, 0x0e74),
                (18, 14, 0x0e75),
                (19, 14, 0x0e76),
                (20, 14, 0x0e77),
            ]),
            // C Overworld_LoadEventOverlay asserts for these invalid screens.
            114..=118 | 122 | 124..=127 => panic!("Overworld_LoadEventOverlay invalid screen"),
            _ => {}
        }
    }

    fn write_event_overlay_2x2(&mut self, x: usize, y: usize) {
        self.write_bg2_xy(x, y, 0x0918);
        self.write_bg2_xy(x + 1, y, 0x0919);
        self.write_bg2_xy(x, y + 1, 0x091a);
        self.write_bg2_xy(x + 1, y + 1, 0x091b);
    }

    fn write_event_overlay_pairs(&mut self, entries: &[(usize, usize, u16)]) {
        for &(x, y, value) in entries {
            self.write_bg2_xy(x, y, value);
        }
    }

    fn write_bg2_xy(&mut self, x: usize, y: usize, value: u16) {
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile(y * 64 + x, value);
    }
}

#[cfg(test)]
#[path = "overworld_tests.rs"]
mod tests;
