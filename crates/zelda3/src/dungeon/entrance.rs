//! Dungeon entrance selection, initialization, and pre-dungeon setup.

use super::*;
use crate::zelda_rtl::misc::DUNG_ANIMATED_TILES;

impl ZeldaState {

    /// Source-address bindings for immutable entrance data already owned by
    /// the native asset pack. The continuation uses live WRAM for selectors.
    pub(crate) fn native_dungeon_entrance_table_byte(&self, address: u32) -> Option<u8> {
        let (asset, start) = match address {
            0x02c813..=0x02c91c => (ENTRANCE_DATA_ASSETS.rooms, 0x02c813),
            0x02c91d..=0x02cd44 => (ENTRANCE_DATA_ASSETS.relative_coords, 0x02c91d),
            0x02cd45..=0x02ce4e => (ENTRANCE_DATA_ASSETS.scroll_x, 0x02cd45),
            0x02ce4f..=0x02cf58 => (ENTRANCE_DATA_ASSETS.scroll_y, 0x02ce4f),
            0x02cf59..=0x02d062 => (ENTRANCE_DATA_ASSETS.player_y, 0x02cf59),
            0x02d063..=0x02d16c => (ENTRANCE_DATA_ASSETS.player_x, 0x02d063),
            0x02d16d..=0x02d276 => (ENTRANCE_DATA_ASSETS.camera_y, 0x02d16d),
            0x02d277..=0x02d380 => (ENTRANCE_DATA_ASSETS.camera_x, 0x02d277),
            0x02d381..=0x02d405 => (ENTRANCE_DATA_ASSETS.blockset, 0x02d381),
            0x02d406..=0x02d48a => (ENTRANCE_DATA_ASSETS.floor, 0x02d406),
            0x02d48b..=0x02d50f => (ENTRANCE_DATA_ASSETS.palace, 0x02d48b),
            0x02d510..=0x02d594 => (ENTRANCE_DATA_ASSETS.doorway_orientation, 0x02d510),
            0x02d595..=0x02d619 => (ENTRANCE_DATA_ASSETS.starting_bg, 0x02d595),
            0x02d61a..=0x02d69e => (ENTRANCE_DATA_ASSETS.quadrant1, 0x02d61a),
            0x02d69f..=0x02d723 => (ENTRANCE_DATA_ASSETS.quadrant2, 0x02d69f),
            0x02d724..=0x02d82d => (ENTRANCE_DATA_ASSETS.door_settings, 0x02d724),
            0x02d82e..=0x02d8b2 => (27, 0x02d82e),
            0x02db6e..=0x02db7b => (STARTING_POINT_ASSETS.rooms, 0x02db6e),
            0x02db7c..=0x02dbb3 => (STARTING_POINT_ASSETS.relative_coords, 0x02db7c),
            0x02dbb4..=0x02dbc1 => (STARTING_POINT_ASSETS.scroll_x, 0x02dbb4),
            0x02dbc2..=0x02dbcf => (STARTING_POINT_ASSETS.scroll_y, 0x02dbc2),
            0x02dbd0..=0x02dbdd => (STARTING_POINT_ASSETS.player_y, 0x02dbd0),
            0x02dbde..=0x02dbeb => (STARTING_POINT_ASSETS.player_x, 0x02dbde),
            0x02dbec..=0x02dbf9 => (STARTING_POINT_ASSETS.camera_y, 0x02dbec),
            0x02dbfa..=0x02dc07 => (STARTING_POINT_ASSETS.camera_x, 0x02dbfa),
            0x02dc08..=0x02dc0e => (STARTING_POINT_ASSETS.blockset, 0x02dc08),
            0x02dc0f..=0x02dc15 => (STARTING_POINT_ASSETS.floor, 0x02dc0f),
            0x02dc16..=0x02dc1c => (STARTING_POINT_ASSETS.palace, 0x02dc16),
            0x02dc1d..=0x02dc23 => (STARTING_POINT_ASSETS.starting_bg, 0x02dc1d),
            0x02dc24..=0x02dc2a => (STARTING_POINT_ASSETS.quadrant1, 0x02dc24),
            0x02dc2b..=0x02dc31 => (STARTING_POINT_ASSETS.quadrant2, 0x02dc2b),
            0x02dc32..=0x02dc3f => (STARTING_POINT_ASSETS.door_settings, 0x02dc32),
            0x02dc40..=0x02dc4d => {
                let offset = (address - 0x02dc40) as usize;
                let value = *self.asset_raw(44)?.get(offset / 2)?;
                return Some(if offset & 1 == 0 { value } else { 0 });
            }
            0x02dc4e..=0x02dc54 => (45, 0x02dc4e),
            0x04f1de..=0x04f369 => (53, 0x04f1de),
            0x04f36a..=0x04f489 => (54, 0x04f36a),
            0x04f48a..=0x04f4e9 => (55, 0x04f48a),
            _ => return None,
        };
        self.asset_raw(asset)?.get((address - start) as usize).copied()
    }

    /// ROM: `Dungeon_LoadEntrance`
    pub(in crate::zelda_rtl) fn dungeon_load_entrance(&mut self) {
        let selection = self.dungeon_load_entrance_before_fields();
        self.dungeon_load_entrance_after_selection(selection, false);
    }

    /// Route retained entrance stores before generic startup RAM ranges. The
    /// value comes from the live native caller; no source state is imported.
    pub(crate) fn native_dungeon_entrance_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let offset = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        match offset {
            0x10a => {
                assert!(matches!(width, 1 | 2));
                self.game_state.system_signals.set_game_over_check_flag(value as u8);
            }
            0x10e => {
                if width == 1 { self.set_which_entrance(value as u8); }
                else { assert_eq!(width, 2); self.set_which_entrance_word(value); }
            }
            0x11a => { assert_eq!(width, 2); self.set_bg1_x_offset(value); }
            0x11c => { assert_eq!(width, 2); self.set_bg1_y_offset(value); }
            0x132 => { assert_eq!(width, 1); self.game_state.system_signals.set_queued_music_control(value as u8); }
            0x4aa => { assert_eq!(width, 1); self.game_state.system_signals.set_restart_check_flag(value as u8); }
            0x40c => { assert_eq!(width, 1); self.save_progress_mut().set_palace_index_x2(value as u8); }
            0xec => { assert_eq!(width, 2); self.tile_detect_position_mut().set_location_calc_mask(value); }
            0x1b | 0x20..=0x25 | 0x29 | 0x2f | 0x42..=0x45 | 0x6c |
            0x8a..=0x8d | 0xa0..=0xa1 | 0xa4..=0xa7 | 0xa9..=0xaa |
            0xe0..=0xe3 | 0xe6..=0xe9 | 0xee | 0x11e..=0x125 |
            0x476 | 0x48e..=0x48f | 0x4bc..=0x4bd | 0x600..=0x61f |
            0x696..=0x699 | 0xaa1 | 0xc140..=0xc167 | 0xc16a..=0xc173 |
            0xf580..=0xf8ff | 0xf940..=0xfcbf => {
                assert!(matches!(width, 1 | 2));
                for byte in 0..usize::from(width) {
                    let at = offset + byte;
                    let v = (value >> (byte * 8)) as u8;
                    // Keep both owners updated where the cartridge overlays
                    // fields. Each publisher changes only this native byte.
                    let mut owned = self.game_state.world.write_entrance_byte(at, v);
                    owned |= self.game_state.dungeon.write_entrance_byte(at, v);
                    owned |= self.game_state.player.follower_link.write_entrance_byte(at, v);
                    owned |= self.game_state.display.ppu_scroll_copy.write_entrance_byte(at, v);
                    if (0xf800..=0xf8ff).contains(&at) {
                        self.game_state.memorized_tiles.write_address_byte(at - 0xf800, v);
                        owned = true;
                    }
                    assert!(owned, "unowned dungeon entrance byte {at:04x}");
                }
            }
            _ => return false,
        }
        self.ram[offset..offset + usize::from(width)]
            .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        true
    }

    fn dungeon_load_entrance_before_fields(&mut self) -> (usize, bool) {
        self.dungeon_load_entrance_before_selection();
        self.dungeon_load_entrance_select()
    }

    fn dungeon_load_entrance_before_selection(&mut self) {
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
            self.clear_overlay_index_word();
        }
        self.set_bg1_y_offset(0);
        self.set_bg1_x_offset(0);
        self.clear_game_over_check_flag();
    }

    fn dungeon_load_entrance_select(&mut self) -> (usize, bool) {
        if self.game_state.sprites.follower_runtime.indicator_word() == 4
            || self.game_state.system_signals.restart_check_flag() != 0
        {
            let i = self
                .game_state
                .inventory
                .save_progress
                .which_starting_point() as usize;
            let entrance = self.asset_u8(44, i);
            // The original stores the starting-point entrance as a word.
            self.set_which_entrance_word(u16::from(entrance));
            (i, true)
        } else {
            (
                self.game_state.world.region.which_entrance() as usize,
                false,
            )
        }
    }

    fn dungeon_load_entrance_after_selection(
        &mut self,
        (i, starting): (usize, bool),
        scroll_published: bool,
    ) {
        let assets = if starting {
            &STARTING_POINT_ASSETS
        } else {
            &ENTRANCE_DATA_ASSETS
        };
        if !scroll_published {
            self.dungeon_load_entrance_room_and_scroll(i, assets);
        }
        let room = self.dungeon_load_entrance_fields_after_scroll(i, assets);
        if starting {
            self.follower_link_state_mut().set_facing(2);
            self.follower_link_state_mut().clear_doorway_state();
            let queued_music_control = self.asset_u8(45, i);
            self.set_queued_music_control(queued_music_control);
            if i == 0 && self.game_state.inventory.save_progress.progress_indicator() == 0 {
                self.set_queued_music_control(0xff);
            }
            self.clear_restart_check_flag();
        } else {
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

        self.dungeon_room_effects_mut().clear_star_tile_phase();
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

    fn dungeon_load_entrance_room_and_scroll(&mut self, i: usize, assets: &EntranceAssetSet) {
        let room = self.asset_u16(assets.rooms, i);
        self.set_dungeon_room(room);
        self.dungeon_room_tracking_mut().set_room_index2_word(room);

        let scroll_y = self.asset_u16(assets.scroll_y, i);
        self.set_bg1_y(scroll_y);
        self.set_bg2_y(scroll_y);
    }

    fn dungeon_load_entrance_fields_after_scroll(
        &mut self,
        i: usize,
        assets: &EntranceAssetSet,
    ) -> u16 {
        let room = self.asset_u16(assets.rooms, i);
        let scroll_y = self.asset_u16(assets.scroll_y, i);
        self.set_bg1_v_copy(scroll_y);
        self.set_bg2_v_copy(scroll_y);

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

    pub(crate) fn native_dungeon_spiral_background(index: usize) -> Option<u8> {
        DUNGEON_RESET_TORCH_BACKGROUND_AND_PLAYER_SPIRAL_BG_PROPERTIES
            .get(index)
            .map(|&value| value as u8)
    }

    pub(crate) fn native_load_file_table_byte(address: u32) -> Option<u8> {
        match address {
            0x02811e..=0x028135 => DUNG_ANIMATED_TILES.get((address - 0x02811e) as usize).copied(),
            0x02a1e5..=0x02a1e8 => DUNGEON_LIT_TORCH_COLOR_PLUS.get((address - 0x02a1e5) as usize).copied(),
            _ => None,
        }
    }

    /// ROM: `Dungeon_ResetTorchBackgroundAndPlayer`
    pub(super) fn dungeon_reset_torch_background_and_player(&mut self) {
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
        self.dungeon_reset_torch_background_and_player_inner();
    }

    /// ROM: `Dungeon_ResetTorchBackgroundAndPlayerInner`
    pub(in crate::zelda_rtl) fn dungeon_reset_torch_background_and_player_inner(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x02_8b0c);
        crate::cycle_ledger::charge(62); // JSL interactive cleanup.
        self.ancilla_terminate_select_interactives(0);
        self.dungeon_reset_player_after_interactive_cleanup();
    }

    pub(in crate::zelda_rtl) fn dungeon_reset_player_after_interactive_cleanup(&mut self) {
        crate::cycle_ledger::charge(32 + 16); // LDA running; BEQ.
        if self.game_state.player.follower_link.is_running()
            && !self
                .game_state
                .enhanced_features
                .has(DUNGEON_RESET_TORCH_BACKGROUND_AND_PLAYER_INNER_FEATURES0_TURN_WHILE_DASHING)
        {
            crate::cycle_ledger::charge(264); // $8B15-$8B2B: reset the running player.
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
        } else {
            crate::cycle_ledger::charge(6); // BEQ taken.
        }
        crate::cycle_ledger::charge(42); // RTS.
    }
    pub(in crate::zelda_rtl) fn module_pre_dungeon(&mut self) {
        let measured_nmis = self.spotlight_successor_cpu_entry_envelope.take()
            .map(|entry| crate::zelda_rtl::pre_dungeon_load_nmi_slices(self, entry));
        self.module_pre_dungeon_audio_prefix();
        self.module_pre_dungeon_entrance_prefix();
        if let Some(progress) = self.take_deferred_original_timing_pre_dungeon_return() {
            assert_eq!(
                progress,
                crate::MainLoopProgress::CallStackContinued,
                "deferred Module_PreDungeon completion lost its source caller phase",
            );
            self.complete_module_pre_dungeon_before_return_suffix();
            self.complete_module_pre_dungeon_authoritative_return();
            return;
        }
        if self.begin_pre_dungeon_entrance_load_work(measured_nmis) {
            return;
        }
        self.complete_module_pre_dungeon_before_return_suffix();
        self.complete_module_pre_dungeon_entrance_load();
    }

    pub(in crate::zelda_rtl) fn module_pre_dungeon_audio_prefix(&mut self) {
        self.set_ambient_sound_effect(5);
        self.set_sound_effect_1(0);
    }

    pub(super) fn module_pre_dungeon_entrance_prefix(&mut self) {
        // Module_PreDungeon commits the entrance selection and Link placement
        // before Dungeon_LoadAndDrawRoom enters its interruptible construction
        // stack. Keep those visible mutations on the initiating host call.
        self.module_pre_dungeon_initial_entrance();
        self.module_pre_dungeon_after_entrance_prefix();
    }

    pub(in crate::zelda_rtl) fn module_pre_dungeon_initial_entrance(&mut self) {
        if let Some(continuation) = self.pending_selected_game_entrance.take() {
            let (selection, scroll_published) = match continuation {
                SelectedGameEntranceContinuation::BeforeSelection => {
                    (self.dungeon_load_entrance_select(), false)
                }
                SelectedGameEntranceContinuation::AfterScroll { selection } => (selection, true),
            };
            self.dungeon_load_entrance_after_selection(selection, scroll_published);
            return;
        }
        self.module_pre_dungeon_before_initial_entrance();
        self.dungeon_load_entrance();
    }

    pub(in crate::zelda_rtl) fn begin_selected_game_entrance_scroll_prefix(&mut self) {
        assert!(self.pending_selected_game_entrance.is_none());
        self.module_pre_dungeon_before_initial_entrance();
        let selection = self.dungeon_load_entrance_before_fields();
        assert!(
            selection.1,
            "starting-point source checkpoint reached an ordinary entrance"
        );
        self.dungeon_load_entrance_room_and_scroll(selection.0, &STARTING_POINT_ASSETS);
        self.pending_selected_game_entrance =
            Some(SelectedGameEntranceContinuation::AfterScroll { selection });
    }

    pub(in crate::zelda_rtl) fn begin_selected_game_entrance_before_selection(&mut self) {
        assert!(self.pending_selected_game_entrance.is_none());
        self.module_pre_dungeon_before_initial_entrance();
        self.dungeon_load_entrance_before_selection();
        self.pending_selected_game_entrance =
            Some(SelectedGameEntranceContinuation::BeforeSelection);
    }

    fn module_pre_dungeon_before_initial_entrance(&mut self) {
        self.set_dungeon_room(0);
        self.dungeon_room_tracking_mut()
            .set_previous_room_index_word(0);
        self.dungeon_savegame_state_mut().clear_savegame_state_low();
        self.dungeon_savegame_state_mut()
            .clear_savegame_state_high();
        self.clear_agahnim_palette_settings(12);
    }

    pub(in crate::zelda_rtl) fn module_pre_dungeon_after_entrance_prefix(&mut self) {
        self.load_pre_dungeon_keys();
        self.hud_rebuild();
        self.dungeon_torch_mut().clear_lit_torches();
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
    }

    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_entrance_load(&mut self) {
        self.module_pre_dungeon_after_sprite_reset_with_song_bank_timing(true);
    }

    /// Complete the `Module_PreDungeon` suffix after a continuous timing
    /// authority has observed the original C caller return. The source call
    /// already includes `Dungeon_LoadSongBankIfNeeded`, so scheduling a second
    /// translated song-bank wait here would replay that synchronous work.
    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_authoritative_return(&mut self) {
        self.module_pre_dungeon_after_sprite_reset_with_song_bank_timing(false);
    }

    /// Complete `Module_PreDungeon` after a source-tracked
    /// `Dungeon_ResetSprites` call has already published and then resumed its
    /// exact room-load prefix.
    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_after_dungeon_reset_authoritative_return(&mut self) {
        self.module_pre_dungeon_after_dungeon_reset_with_song_bank_timing(false);
    }

    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_after_dungeon_reset_entrance_load(&mut self) {
        self.module_pre_dungeon_after_dungeon_reset_with_song_bank_timing(true);
    }

    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_before_return_suffix(&mut self) {
        self.complete_module_pre_dungeon_before_sprite_reset();
        self.sprite_reset_all();
    }

    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_through_sprite_disable_all(&mut self) {
        self.complete_module_pre_dungeon_before_sprite_reset();
        self.sprite_disable_all();
    }

    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_after_sprite_disable_all(&mut self) {
        self.sprite_reset_all_no_disable();
    }

    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_before_animated_tiles(&mut self) {
        if !self.finish_early_dungeon_room_load(EarlyDungeonRoomOwner::PreDungeon) {
            self.dungeon_load_and_draw_room();
        }
        self.dungeon_load_custom_tile_attr();
    }

    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_before_tilesets(&mut self) {
        if self.native_pre_dungeon_animated_graphics.is_none() {
            self.complete_module_pre_dungeon_before_animated_tiles();
        }
        let animated = DUNG_ANIMATED_TILES
            [self.game_state.world.palette_theme.main_tile_theme_index() as usize];
        self.decompress_animated_dungeon_tiles(animated as usize);
        self.dungeon_load_attribute_table();
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(10);
    }

    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_before_palette_cache(&mut self) {
        if self.native_pre_dungeon_tileset_publication.is_none() {
            self.complete_module_pre_dungeon_before_tilesets();
        }
        self.initialize_tilesets();
        self.native_pre_dungeon_tileset_publication = None;
        // PALETTE_SP6R_INDOORS (0xab1) is read by palette_load_sprite_environment_dungeon
        // off display.palette_buffer, not world.palette_theme. Set the display owner so
        // the immediately-following dungeon_load_palettes() loads the selector-10 sprite
        // environment palette instead of a stale selector (matches raw-RAM single-byte).
        self.set_sp6r_indoors(10);
        self.dungeon_load_palettes_before_cache();
    }

    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_before_sprite_reset(&mut self) {
        self.advance_pre_dungeon_palette_cache(257);
        self.native_pre_dungeon_palette_cache = None;

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
            .set_fixed_color_plusminus(DUNGEON_LIT_TORCH_COLOR_PLUS[torch]);
        self.dungeon_approach_fixed_color_variable(
            self.game_state.dungeon.room_effects.fixed_color_plusminus(),
        );
        self.set_countdown(0x1f);
        self.clear_mosaic_target_level();
        self.set_darkening_or_lightening_screen(2);
        self.clear_overworld_aux_or_main_offset();
        self.follower_link_state_mut().clear_speed_modifier();
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().clear_button_b_frames();
        self.dungeon_reset_torch_background_and_player();
        self.link_check_bunny_status();
        self.reset_then_cache_room_entry_properties();
        if self.game_state.sprites.follower_runtime.indicator() == 13 {
            self.follower_state_mut().set_indicator(0);
            self.set_super_bomb_indicator_timer(0);
            self.hud_remove_super_bomb_indicator();
        }
        self.set_bg_mode(9);
        self.follower_initialize();
    }

    /// `Module_PreDungeon` from `Sprite_ResetAll`'s return through the start
    /// of `Dungeon_LoadSongBankIfNeeded`: the module/submodule bytes are
    /// written before the song-bank upload holds the wire. Returns whether a
    /// song-bank transfer started.
    fn module_pre_dungeon_after_sprite_reset_prefix(&mut self) -> bool {
        self.dungeon_reset_sprites();
        self.module_pre_dungeon_after_dungeon_reset_prefix()
    }

    fn module_pre_dungeon_after_dungeon_reset_prefix(&mut self) -> bool {
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
        self.dungeon_load_song_bank_if_needed()
    }

    fn module_pre_dungeon_after_dungeon_reset_with_song_bank_timing(
        &mut self,
        defer_song_bank: bool,
    ) {
        let song_bank_transfer_started = self.module_pre_dungeon_after_dungeon_reset_prefix();
        if defer_song_bank
            && song_bank_transfer_started
            && self.begin_pre_dungeon_song_bank_transfer_work()
        {
            return;
        }
        self.complete_module_pre_dungeon_after_song_bank_transfer();
    }

    fn module_pre_dungeon_after_sprite_reset_with_song_bank_timing(
        &mut self,
        defer_song_bank: bool,
    ) {
        let song_bank_transfer_started = self.module_pre_dungeon_after_sprite_reset_prefix();
        if defer_song_bank
            && song_bank_transfer_started
            && self.begin_pre_dungeon_song_bank_transfer_work()
        {
            return;
        }
        self.complete_module_pre_dungeon_after_song_bank_transfer();
    }

    pub(in crate::zelda_rtl) fn complete_module_pre_dungeon_after_song_bank_transfer(&mut self) {
        self.module_pre_dungeon_set_ambient_sfx();
    }

    pub(super) fn link_check_bunny_status(&mut self) {
        if self.game_state.player.follower_link.handler_state() == 2 {
            let handler_state = if !self.game_state.player.follower_link.is_bunny_mirror() {
                0
            } else if self.game_state.inventory.items.has_moon_pearl() {
                28
            } else {
                23
            };
            self.follower_link_state_mut()
                .set_handler_state(handler_state);
        }
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
        // $02:8D71..$8E0E: six byte clears, width changes, sixteen word
        // copies, five byte copies, and the near return. Cache helpers are
        // translations of this inlined body, not separately called work.
        crate::cycle_ledger::charge_routine(0x02_8d71, 1950);
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
}
