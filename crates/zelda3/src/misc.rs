// Methods ported from zelda3/src/misc.c and included inside ZeldaState.

use super::*;
fn calculate_sfx_pan(x: u16) -> u8 {
    ZeldaState::calculate_sfx_pan_with_scroll(x, 0)
}

fn receive_item_tab1_misc(item: u8) -> u8 {
    const RECEIVE_ITEM_OAM_EXT_SIZES: [u8; 76] = [
        0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
    ];
    RECEIVE_ITEM_OAM_EXT_SIZES
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn receive_item_tab2_misc(item: u8) -> i8 {
    const RECEIVE_ITEM_DRAW_Y_OFFSETS: [i8; 76] = [
        -5, -5, -5, -5, -5, -4, -4, -5, -5, -4, -4, -4, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4,
        -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -5, -4, -4, -4, -4, -4, -4, -2, -4, -4, -4,
        -4, -4, -4, -4, -4, -4, -2, -2, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -2, -2, -4,
        -2, -4, -4, -4, -5, -4, -4,
    ];
    RECEIVE_ITEM_DRAW_Y_OFFSETS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn receive_item_tab3_misc(item: u8) -> u8 {
    const RECEIVE_ITEM_PALETTE_BITS: [u8; 76] = [
        4, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 4, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 4, 4, 0, 4, 0, 0, 0, 4, 0, 0,
    ];
    RECEIVE_ITEM_PALETTE_BITS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn receive_item_gfx_misc(item: u8) -> u8 {
    const LOCAL_GRAPHICS: [u8; 76] = [
        6, 0x18, 0x18, 0x18, 0x2d, 0x20, 0x2e, 9, 9, 0x0a, 8, 5, 0x10, 0x0b, 0x2c, 0x1b, 0x1a,
        0x1c, 0x14, 0x19, 0x0c, 7, 0x1d, 0x2f, 7, 0x15, 0x12, 0x0d, 0x0d, 0x0e, 0x11, 0x17, 0x28,
        0x27, 4, 4, 0x0f, 0x16, 3, 0x13, 1, 0x1e, 0x10, 0, 0, 0, 0, 0, 0, 0x30, 0x22, 0x21, 0x24,
        0x24, 0x24, 0x23, 0x23, 0x23, 0x29, 0x2a, 0x2c, 0x2b, 3, 3, 0x34, 0x35, 0x31, 0x33, 2,
        0x32, 0x36, 0x37, 0x2c, 6, 0x0c, 0x38,
    ];
    LOCAL_GRAPHICS.get(item as usize).copied().unwrap_or(0)
}

fn memory_location_to_give_item_to_misc(item: u8) -> usize {
    const MEMORY_LOCATIONS: [usize; 76] = [
        0xf359, 0xf359, 0xf359, 0xf359, 0xf35a, 0xf35a, 0xf35a, 0xf345, 0xf346, 0xf34b, 0xf342,
        0xf340, 0xf341, 0xf344, 0xf35c, 0xf347, 0xf348, 0xf349, 0xf34a, 0xf34c, 0xf34c, 0xf350,
        0xf35c, 0xf36b, 0xf351, 0xf352, 0xf353, 0xf354, 0xf354, 0xf34e, 0xf356, 0xf357, 0xf37a,
        0xf34d, 0xf35b, 0xf35b, 0xf36f, 0xf364, 0xf36c, 0xf375, 0xf375, 0xf344, 0xf341, 0xf35c,
        0xf35c, 0xf35c, 0xf36d, 0xf36e, 0xf36e, 0xf375, 0xf366, 0xf368, 0xf360, 0xf360, 0xf360,
        0xf374, 0xf374, 0xf374, 0xf340, 0xf340, 0xf35c, 0xf35c, 0xf36c, 0xf36c, 0xf360, 0xf360,
        0xf372, 0xf376, 0xf376, 0xf373, 0xf360, 0xf360, 0xf35c, 0xf359, 0xf34c, 0xf355,
    ];
    MEMORY_LOCATIONS.get(item as usize).copied().unwrap_or(0)
}

fn value_to_give_item_to_misc(item: u8) -> u8 {
    const VALUES: [u8; 76] = [
        1, 2, 3, 4, 1, 2, 3, 1, 1, 1, 1, 1, 1, 2, 0xff, 1, 1, 1, 1, 1, 2, 1, 0xff, 0xff, 1, 1, 2,
        1, 2, 1, 1, 1, 0xff, 1, 0xff, 2, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 2, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfb, 0xec, 0xff, 0xff, 0xff, 1, 3, 0xff, 0xff,
        0xff, 0xff, 0x9c, 0xce, 0xff, 1, 10, 0xff, 0xff, 0xff, 0xff, 1, 3, 1,
    ];
    VALUES.get(item as usize).copied().unwrap_or(0xff)
}

impl ZeldaState {
    #[track_caller]
    pub(super) fn get_random_number(&mut self) -> u8 {
        let before = self.world_state_view().rng_seed();
        let mut t = self
            .world_state_view()
            .rng_seed()
            .wrapping_add(self.frame_state().frame_counter);
        t = if t & 1 != 0 { t >> 1 } else { (t >> 1) ^ 0xb8 };
        self.world_state_view_mut().set_rng_seed(t);
        let trace_rng = std::env::var_os("ZELDA3_TRACE_RNG").is_some();
        let trace_frame_matches = std::env::var("ZELDA3_TRACE_RNG_FRAME")
            .ok()
            .and_then(|value| {
                let trimmed = value.trim();
                if let Some(hex) = trimmed.strip_prefix("0x") {
                    u8::from_str_radix(hex, 16).ok()
                } else {
                    trimmed.parse::<u8>().ok()
                }
            })
            .is_none_or(|frame| frame == self.frame_state().frame_counter);
        if trace_rng && trace_frame_matches {
            let loc = std::panic::Location::caller();
            eprintln!(
                "R rng fc={} before=0x{:02x} after=0x{:02x} site={}:{} link=0x{:04x},0x{:04x}",
                self.frame_state().frame_counter,
                before,
                t,
                loc.file(),
                loc.line(),
                self.player_state_view().x(),
                self.player_state_view().y(),
            );
        }
        t
    }

    pub(super) fn module_reserved_12(&mut self) {
        // C Module_Unknown0 is an assert(0) main-module slot.
        panic!("module_reserved_12 reached");
    }

    pub(super) fn module_reserved_13(&mut self) {
        // C Module_Unknown1 is an assert(0) main-module slot.
        panic!("module_reserved_13 reached");
    }

    pub(super) fn sound_load_intro_song_bank(&mut self) {
        self.load_song_bank_asset(0);
    }

    pub(super) fn load_overworld_songs(&mut self) {
        self.load_song_bank_asset(0);
    }

    pub(super) fn load_dungeon_songs(&mut self) {
        self.load_song_bank_asset(1);
    }

    pub(super) fn load_credits_songs(&mut self) {
        self.load_song_bank_asset(2);
    }

    fn load_song_bank_asset(&mut self, asset: usize) {
        if let Some(bank) = self.asset_raw(asset).map(<[u8]>::to_vec) {
            self.load_song_bank(&bank);
        }
    }

    pub fn zelda_debug_song_bank_summary(&self, asset: usize) -> String {
        let Some(bank) = self.asset_raw(asset) else {
            return format!("song_bank[{asset}]=missing");
        };
        let mut ram = [0u8; 0x10000];
        let mut cursor = 0usize;
        let mut blocks = 0usize;
        let mut total_bytes = 0usize;
        let mut first_target = None;
        let mut min_target = 0xffffusize;
        let mut max_written = 0usize;
        while cursor + 4 <= bank.len() {
            let num_bytes = u16::from_le_bytes([bank[cursor], bank[cursor + 1]]) as usize;
            if num_bytes == 0 {
                cursor += 2;
                break;
            }
            let mut target = u16::from_le_bytes([bank[cursor + 2], bank[cursor + 3]]) as usize;
            cursor += 4;
            if cursor + num_bytes > bank.len() {
                return format!(
                    "song_bank[{asset}] malformed blocks={blocks} target={target:04x} len={num_bytes} remaining={}",
                    bank.len().saturating_sub(cursor)
                );
            }
            first_target.get_or_insert(target);
            min_target = min_target.min(target);
            for &byte in &bank[cursor..cursor + num_bytes] {
                ram[target & 0xffff] = byte;
                target = target.wrapping_add(1);
            }
            max_written = max_written.max(target.wrapping_sub(1) & 0xffff);
            cursor += num_bytes;
            blocks += 1;
            total_bytes += num_bytes;
        }
        let first_target = first_target.unwrap_or(0);
        let reset_vector = u16::from_le_bytes([ram[0xfffe], ram[0xffff]]);
        let code800 = &ram[0x800..0x808];
        let code878 = &ram[0x878..0x880];
        format!(
            "song_bank[{asset}] size={} blocks={blocks} total_bytes={total_bytes} first_target={first_target:04x} range={min_target:04x}-{max_written:04x} reset={reset_vector:04x} ram0800={code800:02x?} ram0878={code878:02x?} parse_end={cursor}",
            bank.len()
        )
    }

    fn kill_agahnim_load_music(&mut self) {
        self.clear_core_update_disable_flag();
        self.increment_overworld_map_state();
        self.increment_submodule();
        self.LoadOWMusicIfNeeded();
    }

    fn kill_aghanim_init(&mut self) {
        self.system_signals_view_mut().set_music_control(8);
        self.set_edge_transition_direction_bits(8);
        self.InitializeMirrorHDMA();
        self.set_overworld_map_state(0);
        self.PaletteFilter_InitializeWhiteFilter();
        self.Overworld_LoadGFXAndScreenSize();
        self.increment_submodule();
        self.player_state_view_mut().set_handler_state(20);
        self.world_state_view_mut().set_bg1_x_offset(0);
        self.world_state_view_mut().set_bg1_y_offset(0);
        self.dungeon_state_view_mut().clear_savegame_state_bits();
        self.player_state_view_mut().clear_movement_velocity();
        self.palette_buffer_view_mut().set_main_color(0, 0x7fff);
        self.palette_buffer_view_mut().set_main_color(32, 0x7fff);
        self.ancilla_terminate_select_interactives(0);
        self.link_reset_properties_a();
    }

    fn kill_aghanim_func2(&mut self) {
        self.set_hdma_enable_mask(192);
        self.MirrorWarp_BuildWavingHDMATable();
        self.increment_submodule();
        self.set_subsubmodule(0);
    }

    fn kill_aghanim_func3(&mut self) {
        self.MirrorWarp_BuildWavingHDMATable();
        if self.frame_state().subsubmodule != 0 {
            self.set_subsubmodule(0);
            self.increment_submodule();
        }
    }

    fn kill_aghanim_func4(&mut self) {
        self.MirrorWarp_BuildDewavingHDMATable();
        if self.frame_state().subsubmodule != 0 {
            self.set_subsubmodule(0);
            self.increment_submodule();
        }
    }

    fn kill_aghanim_func5(&mut self) {
        self.hdma_setup(0, 0xf2fb, 0x41, 0, 0x26, 0);
        for i in 0..240 {
            self.spotlight_hdma_view_mut()
                .set_hdma_table_dynamic_entry(i, 0xff00);
        }
        self.palette_filter_view_mut().set_countdown(0);
        self.palette_filter_view_mut()
            .set_darkening_or_lightening_screen(0);
        self.dialogue_message_index_view_mut().set_value(0x35);
        self.main_show_text_message();
        self.ReloadPreviouslyLoadedSheets();
        self.hud_rebuild_indoor();
        self.set_hdma_enable_mask(0x80);
        self.set_main_module(21);
        self.set_submodule(6);
        self.set_subsubmodule(24);
    }

    fn kill_aghanim_func6(&mut self) {
        self.decrement_subsubmodule();
        if self.frame_state().subsubmodule == 0 {
            self.increment_submodule();
            self.system_signals_view_mut().set_ambient_sound_effect(9);
        }
    }

    fn kill_aghanim_func7(&mut self) {
        self.RenderText();
        if self.frame_state().submodule == 0 {
            self.set_overworld_map_state(0);
            self.system_signals_view_mut().set_ambient_sound_effect(5);
            if !self.inventory_state_view().has_moon_pearl() {
                self.dialogue_message_index_view_mut().set_value(0x36);
                self.main_show_text_message();
                self.system_signals_view_mut().set_ambient_sound_effect(0);
                self.set_main_module(21);
                self.set_submodule(8);
            } else {
                self.set_submodule(9);
            }
        }
    }

    fn kill_aghanim_func8(&mut self) {
        self.RenderText();
        if self.frame_state().submodule == 0 {
            self.set_subsubmodule(32);
            self.set_submodule(12);
        }
    }

    fn kill_aghanim_func12(&mut self) {
        self.decrement_subsubmodule();
        if self.frame_state().subsubmodule != 0 {
            return;
        }
        self.ResetAncillaAndCutscene();
        self.Overworld_SetSongList();
        self.overworld_event_info_view_mut()
            .set_event_bits(0x1b, 32);
        self.save_progress_view_mut().set_palace_index_x2(255);
        self.set_submodule(0);
        self.set_overworld_map_state(0);
        self.clear_core_update_disable_flag();
        self.set_main_module(9);
        self.world_state_view_mut().set_bg1_y_low(0);
        let music = if self.inventory_state_view().has_moon_pearl() {
            9
        } else {
            4
        };
        self.system_signals_view_mut().set_music_control(music);
        self.save_progress_view_mut().set_map_icons_indicator(6);
    }

    pub(super) fn dungeon_light_torch(&mut self) {
        if self.dungeon_torch_state().torch_attr() & 0xf0 != 0xc0 {
            self.dungeon_torch_mut().clear_attr();
            return;
        }

        let r8 = if (self.world_location_state().dungeon_room as u8) == 0 {
            0x80
        } else {
            0xc0
        };
        let i = self.dungeon_torch_state().attr_index()
            + ((self.dungeon_torch_state().torches_start_index() >> 1) as usize);
        let opos = self.dungeon_torch_state().torch_object_data_pos(i) as usize;
        let mut tilemap_pos = self.dungeon_state_view().object_tilemap_pos(i);
        if tilemap_pos & 0x8000 != 0 {
            return;
        }
        tilemap_pos |= 0x8000;
        self.dungeon_state_view_mut()
            .set_object_tilemap_pos(i, tilemap_pos);
        if r8 == 0 {
            self.dungeon_torch_mut()
                .set_torch_data_word(opos, tilemap_pos);
        }

        let x = tilemap_pos & 0x3fff;
        self.room_draw_adjust_torch_lighting_change(x, 0x0eca, x);
        let sfx = 42 | self.calculate_sfx_pan_arbitrary(((x & 0x7f) * 2) as u8);
        self.system_signals_view_mut().set_sound_effect_1(sfx);
        self.request_nmi_copy_packets();

        if self.dungeon_state_view().wants_lights_out() != 0 {
            let lit = self.dungeon_state_view().lit_torches();
            self.dungeon_state_view_mut().increment_lit_torches();
            if lit < 3 {
                const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
                let color_plus =
                    LIT_TORCHES_COLOR_PLUS[self.dungeon_state_view().lit_torches() as usize];
                self.set_sub_screen_layers(0);
                self.set_overworld_fixed_color_adjustment(color_plus);
                self.set_submodule(10);
                self.set_subsubmodule(0);
            }
        }

        let torch_timer = self.dungeon_torch_state().attr_index();
        self.dungeon_torch_mut().set_timer(torch_timer, r8);
        self.dungeon_torch_mut().clear_attr();
    }

    pub(super) fn room_draw_adjust_torch_lighting_change(&mut self, x: u16, y: u16, r8: u16) {
        let ptr = self.src_ptr(y);
        let x = (x >> 1) as usize;
        self.set_overworld_tile_attribute_word(x, ptr[0]);
        self.set_overworld_tile_attribute_word(x + 64, ptr[1]);
        self.set_overworld_tile_attribute_word(x + 1, ptr[2]);
        self.set_overworld_tile_attribute_word(x + 65, ptr[3]);
        self.dungeon_prep_overlay_dma_next_prep(0, r8);
    }

    pub(super) fn dungeon_prep_overlay_dma_next_prep(&mut self, dst: usize, r8: u16) -> usize {
        let r6 = 0x0880 + u16::from((r8 & 0x003f) >= 0x003a);
        self.dungeon_prep_overlay_dma_watergate(dst, r8, r6, 4)
    }

    #[rustfmt::skip]
    pub(super) fn dungeon_prep_overlay_dma_watergate(&mut self, dst: usize, r8: u16, r6: u16, loops: usize) -> usize {
        let mut dst = dst;
        let mut r8 = r8;
        for _ in 0..loops {
            let x = (r8 >> 1) as usize;
            self.set_overworld_tile_upload_word(dst + 0, ((r8 & 0x40) << 4) | ((r8 & 0x303f) >> 1) | ((r8 & 0x0f80) >> 2));
            self.set_overworld_tile_upload_word(dst + 1, r6);
            let attr0 = self.overworld_tile_attribute_word(x + 0);
            self.set_overworld_tile_upload_word(dst + 2, attr0);
            if r6 & 1 == 0 {
                for j in 1..=3 {
                    let attr = self.overworld_tile_attribute_word(x + j);
                    self.set_overworld_tile_upload_word(dst + 2 + j, attr);
                }
                r8 = r8.wrapping_add(128);
            } else {
                for (j, offset) in [64usize, 128, 192].into_iter().enumerate() {
                    let attr = self.overworld_tile_attribute_word(x + offset);
                    self.set_overworld_tile_upload_word(dst + 3 + j, attr);
                }
                r8 = r8.wrapping_add(2);
            }
            dst += 6;
        }
        self.terminate_overworld_tile_upload_words(dst);
        dst
    }

    fn src_ptr(&self, src: u16) -> [u16; 4] {
        let Some(data) = self.asset_raw(69) else {
            return [0; 4];
        };
        let offset = src as usize;
        [
            read_word_from_slice(data, offset),
            read_word_from_slice(data, offset + 2),
            read_word_from_slice(data, offset + 4),
            read_word_from_slice(data, offset + 6),
        ]
    }

    pub(super) fn module05_load_file(&mut self) {
        if self.state_recorder.replay_mode
            && std::env::var_os("ZELDA3_SMV_LOADFILE_TIMING_HACKS").is_some()
            && self.frame_state().main_module == 5
            && self.frame_state().submodule == 0
            && self.frame_state().saved_module_for_menu == 0
            && self.dialogue_message_index_view().value() == 0x000a
        {
            if self.replay_loadfile_stall == 0 {
                self.replay_loadfile_stall = 75;
            }
            self.replay_loadfile_stall = self.replay_loadfile_stall.wrapping_sub(1);
            if self.replay_loadfile_stall != 0 {
                return;
            }
        } else {
            self.replay_loadfile_stall = 0;
        }

        self.enable_force_blank();
        self.set_overworld_map_state(0);
        self.player_state_view_mut()
            .clear_somaria_block_bg_check_flag();
        self.follower_state_view_mut()
            .clear_tagalong_shared_state_a();
        self.follower_state_view_mut().clear_draw_anim_frame();
        self.follower_state_view_mut().set_appearance_none_flag(0);
        self.player_state_view_mut()
            .clear_player_pose_draw_counter();
        self.player_state_view_mut()
            .clear_player_special_draw_flag();
        self.erase_tile_maps_normal();
        self.load_default_graphics();
        self.sprite_load_graphics_properties();
        self.init_load_default_tile_attr();
        self.decompress_sword_graphics();
        self.decompress_shield_graphics();
        self.link_initialize();
        self.load_follower_graphics();
        self.sprite_workspace_view_mut().set_graphics_subset(0, 70);
        self.sprite_workspace_view_mut().set_graphics_subset(1, 70);
        self.sprite_workspace_view_mut().set_graphics_subset(2, 70);
        self.sprite_workspace_view_mut().set_graphics_subset(3, 70);
        self.start_shared_message_timer(0x0200);
        self.set_vertical_irq_trigger(48);

        if self.save_progress_view().dark_world_state() != 0 {
            if self.world_location_state().is_indoors() {
                self.load_dungeon_room_rebuild_hud();
                return;
            }
            self.hud_search_for_equipped_item();
            self.hud_rebuild();
            self.hud_update_equipped_item();
            self.system_signals_view_mut().clear_game_over_check_flag();
            self.set_dungeon_room(32);
            self.set_main_module(8);
            self.set_submodule(0);
            self.set_subsubmodule(0);
            self.system_signals_view_mut().clear_restart_check_flag();
        } else if self.display_state().mosaic_level != 0
            || (self.system_signals_view().game_over_check_flag() != 0
                && self.system_signals_view().restart_check_flag() == 0)
            || self.save_progress_view().progress_indicator() < 2
            || self.save_progress_view().which_starting_point() == 5
        {
            self.load_dungeon_room_rebuild_hud();
        } else {
            let message = if self.inventory_state_view().mirror() == 2 {
                0x0185
            } else {
                0x0184
            };
            self.dialogue_message_index_view_mut().set_value(message);
            self.main_show_text_message();
            self.dungeon_load_palettes();
            self.set_screen_brightness(15);
            self.set_main_screen_layers(4);
            self.set_sub_screen_layers(0);
            self.set_main_module(27);
        }
    }

    pub(super) fn load_dungeon_room_rebuild_hud(&mut self) {
        self.clear_mosaic_level();
        self.set_mosaic_copy(7);
        self.hud_search_for_equipped_item();
        self.hud_rebuild();
        self.hud_update_equipped_item();
        self.module_pre_dungeon();
    }

    pub(super) fn patch_new_game_entrance_state(&mut self) {
        if self.save_progress_view().progress_indicator() != 0 {
            return;
        }

        self.world_state_view_mut()
            .set_camera_y_coord_scroll_low(0x017f);
        self.world_state_view_mut()
            .set_camera_y_coord_scroll_hi(0x0181);
        self.dungeon_state_view_mut().set_quadrants_visited(2);
        self.dungeon_state_view_mut().set_current_door_index(2);
        self.palette_buffer_view_mut().set_sp0l(0);
        self.palette_buffer_view_mut().set_sp5l(3);
        self.palette_buffer_view_mut().set_sp6l(1);
        self.palette_buffer_view_mut().set_palette_main_indoors(4);
        self.load_new_game_room_sprite();
    }

    fn load_new_game_room_sprite(&mut self) {
        self.sprite_slot_view_mut(0).set_y_low(0x70);
        self.sprite_slot_view_mut(0).set_x_low(0xa0);
        self.sprite_slot_view_mut(0).set_y_high(0x21);
        self.sprite_slot_view_mut(0).set_x_high(0x09);
        self.sprite_slot_view_mut(0).set_state(0x08);
        self.sprite_slot_view_mut(0).set_sprite_type(0x73);
        self.sprite_workspace_view_mut()
            .set_room_origin_x_high(0x08);
        self.sprite_workspace_view_mut()
            .set_room_origin_y_high(0x20);
        self.sprite_workspace_view_mut().set_shared_scratch_a(0x1a);
    }

    pub(super) fn load_pre_dungeon_keys(&mut self) {
        let d = self.save_progress_view().palace_index_x2_word();
        let keys = if d != 0x00ff {
            let index = if d == 2 { 0 } else { (d >> 1) as usize };
            self.dungeon_key_slots_view().keys_earned_slot(index)
        } else {
            0xff
        };
        self.player_resources_view_mut().set_keys(keys);
    }

    pub(super) fn init_load_default_tile_attr(&mut self) {
        self.dungeon_state_view_mut()
            .copy_default_tile_attrs_head(&DUNGEON_DEFAULT_ATTR[..0x140]);
        self.dungeon_state_view_mut()
            .copy_default_tile_attrs_tail(&DUNGEON_DEFAULT_ATTR[0x140..0x180]);
    }

    pub(super) fn module13_boss_victory_pendant(&mut self) {
        match self.frame_state().submodule {
            0 => self.boss_victory_heal(),
            1 => self.dungeon_start_victory_spin(),
            2 => self.dungeon_run_victory_spin(),
            3 => self.dungeon_close_victory_spin(),
            4 => self.Dungeon_PrepExitWithSpotlight(),
            5 => self.Spotlight_ConfigureTableAndControl(),
            _ => {}
        }
        self.sprite_main();
        self.link_oam_main();
    }

    pub(super) fn boss_victory_heal(&mut self) {
        if !self.hud_refill_magic_power() {
            self.increment_overworld_map_state();
        }
        if !self.hud_refill_health() {
            self.increment_overworld_map_state();
        }
        if self.overworld_map_state() == 0 {
            self.player_state_view_mut()
                .clear_button_mask_b_y_bits(0x40);
            self.Dungeon_ResetTorchBackgroundAndPlayerInner();
            self.player_state_view_mut().set_facing(2);
            self.player_state_view_mut().set_last_direction(2 << 1);
            self.system_signals_view_mut().increment_hud_update_flag();
            self.increment_submodule();
            self.set_subsubmodule(16);
            self.player_state_view_mut().increment_immobilized_flag();
        }
        self.set_overworld_map_state(0);
        self.hud_refill_logic();
    }

    pub(super) fn dungeon_start_victory_spin(&mut self) {
        self.decrement_subsubmodule();
        if self.frame_state().subsubmodule != 0 {
            return;
        }
        self.player_state_view_mut().clear_immobilized();
        self.player_state_view_mut().set_facing(2);
        self.link_animate_victory_spin();
        self.ancilla_terminate_select_interactives(0);
        self.ancilla_add_victory_spin();
        self.increment_submodule();
    }

    pub(super) fn dungeon_run_victory_spin(&mut self) {
        self.link_main();
        if self.player_state_view().handler_state() != 0 {
            return;
        }
        if self.inventory_state_view().sword_type().wrapping_add(1) & 0xfe != 0 {
            self.system_signals_view_mut().set_sound_effect_1(0x2c);
        }
        self.player_state_view_mut().force_hold_sword_up();
        self.set_subsubmodule(32);
        self.increment_submodule();
    }

    pub(super) fn dungeon_close_victory_spin(&mut self) {
        self.decrement_subsubmodule();
        if self.frame_state().subsubmodule != 0 {
            return;
        }
        self.increment_submodule();
        self.player_state_view_mut().clear_movement_velocity();
        self.set_overworld_fixed_color_adjustment(0);
    }

    pub(super) fn module15_mirror_warp_from_aga(&mut self) {
        match self.frame_state().submodule {
            0 => self.kill_agahnim_load_music(),
            1 => self.kill_aghanim_init(),
            2 => self.kill_aghanim_func2(),
            3 => self.kill_aghanim_func3(),
            4 => self.kill_aghanim_func4(),
            5 => self.kill_aghanim_func5(),
            6 => self.kill_aghanim_func6(),
            7 => self.kill_aghanim_func7(),
            8 => self.kill_aghanim_func8(),
            9 => self.boss_victory_heal(),
            10 => self.dungeon_start_victory_spin(),
            11 => self.dungeon_run_victory_spin(),
            12 => self.kill_aghanim_func12(),
            _ => {}
        }
        if self.frame_state().submodule < 2 || self.frame_state().submodule >= 5 {
            self.sprite_main();
            self.link_oam_main();
        }
    }

    pub(super) fn module16_boss_victory_crystal(&mut self) {
        match self.frame_state().submodule {
            0 => self.boss_victory_heal(),
            1 => self.dungeon_start_victory_spin(),
            2 => self.dungeon_run_victory_spin(),
            3 => self.dungeon_close_victory_spin(),
            4 => self.module16_04_fade_and_end(),
            _ => {}
        }
        self.sprite_main();
        self.link_oam_main();
    }

    pub(super) fn module16_04_fade_and_end(&mut self) {
        self.decrement_screen_brightness();
        if self.display_state().screen_brightness != 0 {
            return;
        }
        self.world_state_view_mut().set_bg1_x_offset(0);
        self.world_state_view_mut().set_bg1_y_offset(0);
        self.player_state_view_mut().set_y_velocity(0);
        self.player_state_view_mut().clear_immobilized();
        self.Palette_RevertTranslucencySwap();
        self.player_state_view_mut().set_handler_state(0);
        {
            let mut player = self.player_state_view_mut();
            player.set_receive_item_index(0);
            player.clear_item_hold_pose();
            player.clear_sprite_damage_disable_timer();
        }
        let saved_module = self.frame_state().saved_module_for_menu;
        self.set_main_module(saved_module);
        self.set_submodule(0);
        self.set_subsubmodule(0);
        self.OpenSpotlight_Next2();
    }

    pub(super) fn triforce_room_link_approach_triforce(&mut self) {
        let y = self.player_state_view().y() as u8;
        if y < 152 {
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut()
                .set_direction_and_last_direction(0);
            if self
                .player_state_view_mut()
                .decrement_spin_attack_delay_timer()
                == 0
            {
                self.player_state_view_mut().set_item_hold_pose(2);
                self.increment_subsubmodule();
            }
        } else {
            if y < 169 {
                self.player_state_view_mut().set_speed_setting(0x14);
            }
            self.player_state_view_mut()
                .set_direction_and_last_direction(8);
            self.player_state_view_mut().set_facing(0);
            self.player_state_view_mut().set_spin_attack_delay_timer(64);
        }
    }

    pub(super) fn ancilla_add_item_receipt(&mut self, ain: u8, yin: u8, chest_pos: u16) {
        let Some(k) = self.ancilla_add_simple(ain, yin) else {
            return;
        };
        let item = self.player_state_view().receive_item_index();

        self.player_state_view_mut()
            .set_immobilized_flag(if item == 0x20 { 2 } else { 1 });
        if item == 0 {
            self.inventory_state_view_mut().set_item_memory_value(
                memory_location_to_give_item_to_misc(4),
                value_to_give_item_to_misc(0),
            );
        }

        let value_addr = memory_location_to_give_item_to_misc(item);
        let value = value_to_give_item_to_misc(item);
        if (value as i8) >= 0 {
            self.inventory_state_view_mut()
                .set_item_memory_value(value_addr, value);
        }

        if item == 0x1f {
            self.player_state_view_mut().clear_bunny_body_state();
        } else if item == 0x4b || item == 0x1e {
            self.player_resources_view_mut()
                .add_ability_flags(if item == 0x4b { 4 } else { 2 });
        }

        if item == 0x1b || item == 0x1c {
            self.Palette_UpdateGlovesColor();
        } else if matches!(item, 0x37 | 0x38 | 0x39) {
            let bit = match item {
                0x37 => 4,
                0x38 => 1,
                _ => 2,
            };
            let value = self
                .inventory_state_view_mut()
                .or_item_memory_value(value_addr, bit);
            if value & 7 == 7 {
                self.save_progress_view_mut().set_map_icons_indicator(4);
            }
            self.increment_overworld_map_state();
        } else if item == 0x22 {
            self.inventory_state_view_mut()
                .set_item_memory_value_if_empty(value_addr, 1);
        } else if matches!(item, 0x25 | 0x32 | 0x33) {
            let mask = 0x8000u16 >> ((self.save_progress_view().palace_index_x2() >> 1) as u16);
            self.inventory_state_view_mut()
                .or_item_memory_word(value_addr, mask);
        } else if item == 0x3e {
            if self.player_state_view().is_lifting_or_carrying() {
                self.player_state_view_mut().set_picking_throw_state(2);
            }
        } else if item == 0x20 {
            self.increment_overworld_map_state();
            for i in (0..=4).rev() {
                if matches!(self.ancilla_slot_view(i).ancilla_type(), 7 | 0x2c) {
                    self.ancilla_slot_view_mut(i).clear();
                    let mut player = self.player_state_view_mut();
                    player.clear_state_bits();
                    player.clear_picking_throw_state();
                }
            }
            if self.player_state_view().is_cape_active() {
                self.player_state_view_mut().set_cape_transform_timer(32);
                {
                    let mut player = self.player_state_view_mut();
                    player.clear_sprite_damage_disable_timer();
                    player.clear_cape_mode();
                }
                self.ancilla_add_cape_poof(0x23, 4);
                let sfx = 0x15 | self.link_calculate_sfx_pan();
                self.system_signals_view_mut().set_sound_effect_1(sfx);
            }
        } else if item == 0x29 {
            if self.inventory_state_view().mushroom() != 2 {
                self.inventory_state_view_mut()
                    .set_item_memory_value(value_addr, 1);
                self.hud_refresh_icon();
            }
        } else if item == 0x24
            || (self.player_state_view().item_receipt_method() != 2
                && matches!(item, 0x27 | 0x28 | 0x31))
        {
            let add = match item {
                0x28 => 3,
                0x31 => 10,
                _ => 1,
            };
            self.inventory_state_view_mut()
                .add_item_memory_value_capped(value_addr, add, 99);
            self.hud_refresh_icon();
        } else if item == 0x17 {
            self.inventory_state_view_mut()
                .increment_item_memory_value_mod4(value_addr);
            let sfx = 0x2d | self.link_calculate_sfx_pan();
            self.system_signals_view_mut().set_sound_effect_2(sfx);
        } else if item == 1 {
            self.Overworld_SetSongList();
        } else {
            self.item_receipt_give_bottled_item(item);
        }

        let mut gfx = receive_item_gfx_misc(item);
        if gfx == 0xff {
            gfx = 0;
        } else if gfx == 0x20 || gfx == 0x2d || gfx == 0x2e {
            self.DecompressShieldGraphics();
            self.Palette_Load_Shield();
        }
        self.DecodeAnimatedSpriteTile_variable(gfx);
        if (gfx == 6 || gfx == 0x18) && item != 0 {
            self.DecompressSwordGraphics();
            self.Palette_Load_Sword();
        }

        let mut receipt = self.ancilla_slot_view_mut(k);
        receipt.set_item_to_link(item);
        receipt.set_work_byte_1(0);
        if item == 1 && self.player_state_view().item_receipt_method() != 2 {
            self.ancilla_slot_view_mut(k).set_timer(160);
            self.set_submodule(43);
            self.palette_filter_view_mut().set_countdown(0);
            self.ancilla_add_ms_cutscene(0x35, 4);
            self.ancilla_slot_view_mut(k).set_work_byte_3(2);
        } else {
            self.ancilla_slot_view_mut(k).set_work_byte_3(9);
        }
        let aux_timer = if matches!(item, 0x20 | 0x37 | 0x38 | 0x39) {
            0x68
        } else if item == 0x26 {
            0x02
        } else if self.player_state_view().item_receipt_method() != 0 {
            0x38
        } else {
            0x60
        };
        let item_receipt_method = self.player_state_view().item_receipt_method();
        let mut receipt = self.ancilla_slot_view_mut(k);
        receipt.set_work_byte_4(5);
        receipt.set_step(item_receipt_method);
        receipt.set_aux_timer(aux_timer);

        let (x, y) = if self.player_state_view().item_receipt_method() == 1 {
            let y = ((chest_pos & 0x1f80) >> 4)
                .wrapping_add(self.dungeon_state_view().loading_bg_offset_v() & 0xff00)
                .wrapping_add(receive_item_tab2_misc(item) as i16 as u16);
            let x = ((chest_pos & 0x007e) << 2)
                .wrapping_add(self.dungeon_state_view().loading_bg_offset_h() & 0xff00)
                .wrapping_add(receive_item_tab3_misc(item) as u16);
            (x, y)
        } else {
            if self.ancilla_slot_view(k).step() == 0 && item == 1 {
                let sfx = self.link_calculate_sfx_pan() | 0x2c;
                self.system_signals_view_mut().set_sound_effect_1(sfx);
            } else if matches!(item, 0x20 | 0x37 | 0x38 | 0x39) {
                let music = self.link_calculate_sfx_pan() | 0x13;
                self.system_signals_view_mut().set_music_control(music);
            } else if item != 0x3e && item != 0x17 {
                let sfx = self.link_calculate_sfx_pan() | 0x0f;
                self.system_signals_view_mut().set_sound_effect_2(sfx);
            }
            let method = if self.player_state_view().item_receipt_method() == 3 {
                0
            } else {
                self.player_state_view().item_receipt_method()
            };
            let x_offset = if method != 0 {
                receive_item_tab3_misc(item) as u16
            } else if receive_item_tab1_misc(item) == 0 {
                10
            } else if item == 0x20 {
                0
            } else {
                6
            };
            let y_offset = if method != 0 {
                receive_item_tab2_misc(item) as i16
            } else {
                -14
            } + if method == 2 { -8 } else { 0 };
            (
                self.player_state_view().x().wrapping_add(x_offset),
                self.player_state_view().y().wrapping_add(y_offset as u16),
            )
        };
        self.ancilla_set_xy(k, x, y);
    }

    pub(super) fn item_receipt_give_bottled_item(&mut self, item: u8) {
        const BOTTLE_LIST: [u8; 7] = [0x16, 0x2b, 0x2c, 0x2d, 0x3d, 0x3c, 0x48];
        const POTION_LIST: [u8; 5] = [0x2e, 0x2f, 0x30, 0xff, 0x0e];

        if let Some(j) = BOTTLE_LIST.iter().position(|&candidate| candidate == item) {
            if self
                .inventory_state_view_mut()
                .fill_first_empty_bottle_with(j as u8 + 2)
            {
                return;
            }
        }

        if let Some(j) = POTION_LIST.iter().position(|&candidate| candidate == item) {
            if self
                .inventory_state_view_mut()
                .replace_first_empty_bottle_with(j as u8 + 3)
            {
                return;
            }
        }
    }

    pub(super) fn module17_save_and_quit(&mut self) {
        match self.frame_state().submodule {
            0 => {
                self.increment_submodule();
            }
            1 => {}
            _ => {}
        }
        if self.frame_state().submodule == 1 {
            self.decrement_screen_brightness();
            if self.display_state().screen_brightness == 0 {
                self.set_mosaic_copy(15);
                self.set_subsubmodule(1);
                self.Death_Func15(false);
            }
        }
        self.sprite_main();
        self.link_oam_main();
    }

    pub(super) fn wall_master_send_player_to_last_entrance(&mut self) {
        self.SaveDungeonKeys_misc();
        self.Dungeon_FlagRoomData_Quadrants();
        self.sprite_reset_all();
        self.system_signals_view_mut().clear_restart_check_flag();
        self.set_main_module(17);
        self.set_submodule(0);
        self.clear_bg_vram_load_mode();
        self.reset_some_things_after_death(17);
    }

    pub(super) fn sprite_sfx_queue_sfx1_with_pan(&mut self, k: usize, a: u8) {
        if self.system_signals_view().ambient_sound_effect_is_clear() {
            let x = self.sprite_slot_view(k).x();
            let sfx = a | Self::calculate_sfx_pan_with_scroll(x, self.world_state_view().bg2_x());
            self.system_signals_view_mut().set_ambient_sound_effect(sfx);
        }
    }

    pub(super) fn SaveDungeonKeys_misc(&mut self) {
        let idx = self.save_progress_view().palace_index_x2();
        if idx == 0xff {
            return;
        }
        let slot = if idx == 2 { 0 } else { (idx >> 1) as usize };
        let keys = self.player_resources_view().keys();
        self.dungeon_key_slots_view_mut()
            .set_keys_earned_slot(slot, keys);
    }

    pub(super) fn handle_item_tile_action_overworld(&mut self, x: u16, y: u16) -> u8 {
        if self.world_location_state().is_indoors() {
            self.HandleItemTileAction_Dungeon(x, y)
        } else {
            self.Overworld_ToolAndTileInteraction(x, y) as u8
        }
    }
}

pub(super) const DUNG_ANIMATED_TILES: [u8; 24] = [
    0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5f, 0x5d, 0x5f, 0x5f, 0x5e, 0x5f, 0x5e, 0x5e, 0x5d,
    0x5d, 0x5e, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d,
];

const DUNGEON_DEFAULT_ATTR: [u8; 384] = [
    1, 1, 1, 0, 2, 1, 2, 0, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 1, 0, 0, 2, 0, 0, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 0, 0, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1, 1, 0, 0,
    0, 0, 0, 0x2a, 1, 0x20, 1, 1, 4, 1, 1, 0x18, 1, 2, 0x1c, 1, 0x28, 0x28, 0x2a, 0x2a, 1, 2, 1, 1,
    4, 0, 0, 0, 0x28, 1, 0xa, 0, 1, 1, 0xc, 0xc, 2, 2, 2, 2, 0x28, 0x2a, 0x20, 0x20, 0x20, 2, 8, 0,
    4, 4, 1, 1, 1, 2, 2, 2, 0, 0, 0x20, 0x20, 0, 2, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0x18, 0x10, 0x10, 1, 1, 1, 1, 1, 4, 4, 4, 4, 4, 4, 1, 2, 2,
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x62, 0x62, 0, 0, 0x24, 0x24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x62, 0x62, 0x27, 2, 2, 2,
    0x27, 0x27, 1, 0, 0, 0, 0, 0x24, 0, 0, 0, 0, 0x27, 0x27, 0x27, 0x27, 0x27, 0x10, 2, 1, 0, 0, 0,
    0x24, 0, 0, 0, 0, 0x27, 2, 2, 2, 0x27, 0x27, 0x27, 0x27, 2, 2, 2, 0x24, 0, 0, 0, 0, 0x27, 0x27,
    0x27, 0x27, 0x27, 0x20, 2, 2, 1, 2, 2, 0x23, 2, 0, 0, 0, 0x27, 0x27, 0x27, 0x27, 0x27, 0x20, 2,
    0x27, 2, 0x54, 0, 0, 0x27, 2, 2, 2, 0x27, 0x27, 0x27, 0x27, 0x27, 0x27, 2, 0x27, 2, 0x54, 0, 0,
    0x27, 2, 2, 2, 0x27, 0x27, 0, 0x27, 0x60, 0x60, 1, 1, 1, 1, 2, 2, 0xd, 0, 0, 0x4b, 0x67, 0x67,
    0x67, 0x67, 0x66, 0x66, 0x66, 0x66, 0, 0, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x27, 0x63, 0x27,
    0x55, 0x55, 1, 0x44, 0, 1, 0x20, 2, 2, 0x1c, 0x3a, 0x3b, 0, 0x27, 0x63, 0x27, 0x53, 0x53, 1,
    0x44, 1, 0xd, 0, 0, 0, 9, 9, 9, 9,
];

impl ZeldaState {
    pub(super) fn calculate_sfx_pan_with_scroll(x: u16, bg2hofs: u16) -> u8 {
        let relative = x.wrapping_sub(bg2hofs.wrapping_add(80));
        if relative < 80 {
            0
        } else if (relative as i16) >= 0 {
            0x40
        } else {
            0x80
        }
    }

    pub(super) fn link_calculate_sfx_pan(&self) -> u8 {
        Self::calculate_sfx_pan_with_scroll(
            self.player_state_view().x(),
            self.world_state_view().bg2_x(),
        )
    }

    pub(super) fn play_sfx_set_pan(&mut self, a: u8) -> u8 {
        self.system_signals_view_mut().set_raw_sfx_pan_value(a);
        let out = a | self.link_calculate_sfx_pan();
        self.replay_trace_sfx("play_sfx_set_pan", None, a, out);
        out
    }

    pub(super) fn ancilla_sfx2_near(&mut self, a: u8) -> u8 {
        let panned = self.play_sfx_set_pan(a);
        self.system_signals_view_mut().set_sound_effect_1(panned);
        panned
    }

    pub(super) fn ancilla_sfx3_near(&mut self, a: u8) {
        let panned = self.play_sfx_set_pan(a);
        self.system_signals_view_mut().set_sound_effect_2(panned);
    }

    pub(super) fn calculate_sfx_pan_arbitrary(&self, a: u8) -> u8 {
        const TORCH_PANS: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
        TORCH_PANS[((a.wrapping_sub(self.world_state_view().bg2_x_low()) >> 5) & 7) as usize]
    }

    pub(super) fn sprite_calculate_sfx_pan(&self, k: usize) -> u8 {
        let x = self.sprite_slot_view(k).x();
        Self::calculate_sfx_pan_with_scroll(x, self.world_state_view().bg2_x())
    }

    pub(super) fn sprite_sfx_queue_sfx2_with_pan(&mut self, k: usize, a: u8) {
        if !self.system_signals_view().has_sound_effect_1() {
            let sfx = a | self.sprite_calculate_sfx_pan(k);
            self.system_signals_view_mut().set_sound_effect_1(sfx);
        }
    }

    pub(super) fn sprite_sfx_queue_sfx3_with_pan(&mut self, k: usize, a: u8) {
        if !self.system_signals_view().has_sound_effect_2() {
            let sfx = a | self.sprite_calculate_sfx_pan(k);
            self.system_signals_view_mut().set_sound_effect_2(sfx);
        }
    }

    pub(super) fn main_show_text_message(&mut self) {
        if self.frame_state().main_module != 14 {
            self.world_state_view_mut()
                .clear_tile_interaction_shared_flag();
            self.messaging_state_view_mut().clear_module();
            self.set_submodule(2);
            self.save_main_module_for_menu();
            self.set_main_module(14);
        }
    }

    pub(super) fn nmi_prepare_sprites(&mut self) {
        const LINK_DMA_SOURCES1: [u16; 303] = [
            0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000,
            0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x9440, 0x8080, 0x8080, 0x8080, 0x9400, 0x8040,
            0x80c0, 0x80c0, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080,
            0x8080, 0x8080, 0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000, 0xa8c0,
            0xa900, 0x8000, 0xa8c0, 0xa900, 0x9100, 0x8080, 0x8080, 0x90c0, 0x8040, 0x8000, 0x8000,
            0x8000, 0x8000, 0x8000, 0x8000, 0x9a00, 0x9140, 0x9180, 0x8000, 0x9500, 0x9480, 0x94c0,
            0x94c0, 0x9ae0, 0x8080, 0x8080, 0x9a60, 0x80c0, 0x80c0, 0x9aa0, 0x8000, 0x8000, 0x9aa0,
            0x8000, 0x8000, 0x8080, 0x8080, 0x8100, 0x8100, 0x85c0, 0x8000, 0x8000, 0x85c0, 0x8000,
            0x8000, 0xadc0, 0xadc0, 0xadc0, 0xadc0, 0xadc0, 0xad40, 0xad40, 0xad40, 0xad40, 0xad40,
            0xad80, 0xad80, 0xad80, 0xad80, 0xad80, 0xad80, 0x8040, 0x9400, 0x8040, 0x8000, 0x8080,
            0x8080, 0x9440, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080, 0x8040, 0x8040, 0x8000, 0x8000,
            0x8000, 0x8000, 0x8000, 0x8000, 0xc440, 0x8140, 0x8140, 0xca40, 0x8000, 0x8000, 0x8000,
            0x8000, 0x8000, 0x8000, 0x8040, 0x85c0, 0x8040, 0x85c0, 0x8100, 0x80c0, 0x91c0, 0x8080,
            0x8080, 0x8040, 0x8040, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080, 0x8080, 0x9100, 0xa0c0,
            0xa100, 0xa100, 0xa1c0, 0xa400, 0xa440, 0xa1c0, 0xa400, 0xa440, 0x8080, 0xc480, 0x8080,
            0x8040, 0x8040, 0xca80, 0xca80, 0xca00, 0xc400, 0xca00, 0xc400, 0x81c0, 0x8080, 0x8080,
            0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040,
            0x8040, 0x8040, 0x8000, 0xa8c0, 0xa900, 0x8000, 0x8000, 0xa8c0, 0xa900, 0x8000, 0xa8c0,
            0xa900, 0x8000, 0x8000, 0xa8c0, 0xa900, 0x8040, 0x8040, 0x8040, 0x8080, 0x8080, 0x8040,
            0x8040, 0x8040, 0x8040, 0x8000, 0x8000, 0x8000, 0x8000, 0xd080, 0x8080, 0x90c0, 0xd000,
            0x9080, 0xd040, 0x9080, 0xd040, 0xd080, 0xd080, 0xd080, 0xd080, 0xd080, 0xd000, 0xd000,
            0xd000, 0xd000, 0xd000, 0xd040, 0xd040, 0xd040, 0xd040, 0xd040, 0xd040, 0x8040, 0xd000,
            0x85c0, 0x85c0, 0x85c0, 0xdc40, 0xdc40, 0xdc40, 0x85c0, 0x85c0, 0x85c0, 0xdc40, 0xdc40,
            0xdc40, 0xe1c0, 0xd000, 0x8000, 0xe400, 0xe400, 0xe440, 0x90c0, 0x90c0, 0xd000, 0x8000,
            0x8000, 0xd040, 0x8000, 0x8000, 0xd040, 0xe400, 0xe400, 0xe400, 0x9080, 0xa5c0, 0xac40,
            0xe480, 0x8180, 0x90c0, 0x80c0, 0xe180, 0xd000, 0xe4c0, 0xe4c0, 0xe840, 0xe840, 0xe840,
            0xe540, 0xe540, 0xe540, 0xe900, 0xe900, 0xe900, 0xe900, 0x8080, 0x8080, 0x8000, 0xa9c0,
            0x8080, 0x8140, 0x91c0, 0x8040, 0xa800, 0xa840,
        ];
        const LINK_DMA_SOURCES2: [u16; 303] = [
            0x8840, 0x8800, 0x8580, 0x8800, 0x8580, 0x84c0, 0x8500, 0x8540, 0x8500, 0x8540, 0x8400,
            0x8440, 0x8480, 0x8400, 0x8440, 0x8480, 0x9640, 0x8c40, 0x8c80, 0xad00, 0x9600, 0x8980,
            0x8c00, 0xacc0, 0x8880, 0x88c0, 0x8900, 0x8940, 0x8880, 0x88c0, 0x8900, 0x8940, 0xb0c0,
            0xb100, 0xb140, 0xb100, 0xb140, 0xb000, 0xb040, 0xb080, 0xec80, 0xecc0, 0xb180, 0xd440,
            0xb1c0, 0xb180, 0xd440, 0xb1c0, 0x8c80, 0xad00, 0x95c0, 0x99c0, 0xb440, 0x9580, 0xb480,
            0xb4c0, 0x9580, 0xb480, 0xb4c0, 0x9c20, 0x8000, 0x8000, 0x8000, 0x9700, 0x9680, 0x96c0,
            0x96c0, 0x9ce0, 0x8c80, 0xb540, 0x9c60, 0xb580, 0x8c00, 0x9ca0, 0x8900, 0xb500, 0x9ca0,
            0x8900, 0xb500, 0x8c40, 0xec40, 0x8c00, 0xec00, 0x8dc0, 0x9540, 0x89c0, 0x8dc0, 0x9540,
            0x89c0, 0xb940, 0xb980, 0xb9c0, 0xb980, 0xb9c0, 0xb5c0, 0xb800, 0xb840, 0xb800, 0xb840,
            0xb880, 0xb8c0, 0xb900, 0xb880, 0xb8c0, 0xb900, 0x8980, 0x9600, 0xbcc0, 0x8400, 0xbc80,
            0x8c40, 0x9640, 0xa040, 0xa080, 0xa000, 0xbc40, 0xbd40, 0x8500, 0xbd00, 0xbd80, 0xbd80,
            0x88c0, 0x8900, 0xe9c0, 0x8900, 0xc640, 0xc040, 0xc000, 0xcc40, 0x8940, 0x88c0, 0x8900,
            0xe9c0, 0x8900, 0x8940, 0x8d40, 0x8d80, 0x8d40, 0x8d80, 0xbd00, 0xb000, 0xb000, 0xa480,
            0xa480, 0xa480, 0xa480, 0xac00, 0xac00, 0xac00, 0xac00, 0xa140, 0xa180, 0xa180, 0xa4c0,
            0xa4c0, 0xa500, 0x9d40, 0x9d80, 0x9dc0, 0x9d40, 0x9d80, 0x9dc0, 0x8d00, 0xc680, 0xc180,
            0xc140, 0x8c00, 0xcc80, 0xcc80, 0xcc00, 0xc600, 0xcc00, 0xc600, 0xbd00, 0x8580, 0x8800,
            0xc9c0, 0xccc0, 0xcdc0, 0xcd00, 0xcd40, 0xcd80, 0x8500, 0x8540, 0xc940, 0xc980, 0x8540,
            0xc940, 0xc980, 0x8440, 0x8480, 0xc1c0, 0xc900, 0xc580, 0xc5c0, 0xc8c0, 0x8440, 0x8480,
            0xc1c0, 0xc900, 0xc580, 0xc5c0, 0xc8c0, 0xbd00, 0xacc0, 0xc040, 0xd540, 0xd580, 0xd4c0,
            0xd500, 0xd4c0, 0xd500, 0xd440, 0xd480, 0xd440, 0xd480, 0xd1c0, 0xd400, 0xd100, 0xd100,
            0xd140, 0xd180, 0xd140, 0xd180, 0xb0c0, 0xb100, 0xb140, 0xb100, 0xb140, 0xdd40, 0xdd80,
            0xddc0, 0xdd80, 0xddc0, 0xdc80, 0xdcc0, 0xdd00, 0xdc80, 0xdcc0, 0xdd00, 0xd100, 0xd100,
            0xe000, 0xe040, 0xe080, 0xe0c0, 0xe100, 0xe140, 0xe000, 0xe040, 0xe080, 0xe0c0, 0xe100,
            0xe140, 0x8000, 0xd0c0, 0x8000, 0xb940, 0xb980, 0xb940, 0xdd40, 0xdd80, 0xdd40, 0xdc80,
            0xdcc0, 0xc0c0, 0xdc80, 0xdcc0, 0xc0c0, 0xb9c0, 0xb980, 0xb9c0, 0xa560, 0xa5a0, 0xac80,
            0xed00, 0x8000, 0x8cc0, 0xbd00, 0xe380, 0xbdc0, 0xe500, 0xe500, 0xe880, 0xe8c0, 0xe8c0,
            0xe800, 0xe5c0, 0xe5c0, 0xe940, 0xe980, 0xe940, 0xe980, 0xbd40, 0x8c80, 0xa080, 0x8000,
            0xa980, 0xbd00, 0xbdc0, 0xb400, 0xa880, 0xedc0,
        ];
        const LINK_DMA_SOURCES3: [u16; 27] = [
            0x9a40, 0x9e00, 0x9d20, 0x9f20, 0x9b20, 0xbc20, 0xbc20, 0xbe20, 0xbe20, 0xbe00, 0xbe00,
            0xbe00, 0xbe00, 0xa540, 0xa540, 0xa540, 0xa540, 0xbc00, 0xbc00, 0xbc00, 0xbc00, 0xa740,
            0xa740, 0xa740, 0xa740, 0xe780, 0xe780,
        ];
        const LINK_DMA_SOURCES4: [u16; 8] = [
            0x9000, 0x9020, 0x9060, 0x91e0, 0x90a0, 0x90c0, 0x9100, 0x9140,
        ];
        const LINK_DMA_SOURCES5: [u16; 3] = [0x9300, 0x9340, 0x9380];
        const LINK_DMA_SOURCES6: [u16; 128] = [
            0x9480, 0x94c0, 0x94e0, 0x95c0, 0x9500, 0x9520, 0x9540, 0x9480, 0x9640, 0x9680, 0x96a0,
            0x9780, 0x96c0, 0x96e0, 0x9700, 0x9480, 0x9800, 0x9840, 0x98a0, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9ac0, 0x9b00, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9bc0,
            0x9c00, 0x9c40, 0x9c80, 0x9cc0, 0x9d00, 0x9d40, 0x9480, 0x9f40, 0x9f80, 0x9fc0, 0x9fe0,
            0xa000, 0x9480, 0x9480, 0x9480, 0xa100, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x98c0, 0x9900,
            0x99c0, 0x99e0, 0x9a00, 0x9a20, 0x9a40, 0x9a60, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9a80, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
            0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
        ];
        const LINK_DMA_SOURCES7: [u16; 16] = [
            0x00e0, 0x00e0, 0x0060, 0x0080, 0x01c0, 0x00e0, 0x0040, 0x0000, 0x0080, 0x0000, 0x0040,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        ];
        const LINK_DMA_SOURCES8: [u16; 4] = [0xa480, 0xa4c0, 0xa500, 0xa540];
        const LINK_DMA_CTRS0: [u16; 6] = [14, 4, 6, 16, 6, 8];
        const LINK_DMA_SOURCES9: [u16; 15] = [
            0, 0x20, 0x40, 0, 0x20, 0x40, 0, 0x40, 0x80, 0, 0x40, 0x80, 0xb340, 0xb400, 0xb4c0,
        ];
        fn link_dma_table_value(table: &[u16], index: usize, table_name: &str) -> u16 {
            // C indexes these static DMA tables directly; this keeps an invalid
            // translated index from becoming a silent wrong DMA source.
            table.get(index).copied().unwrap_or_else(|| {
                panic!(
                    "nmi_prepare_sprites {table_name} index {index} out of bounds; table_len={}",
                    table.len()
                )
            })
        }

        for i in 0..32 {
            let value = self.oam_state_view().packed_extended_oam_byte(i);
            self.oam_state_view_mut()
                .set_packed_extended_oam_byte(i, value);
        }

        let link_dma_graphics_index =
            (self.player_state_view().link_dma_graphics_index_word() >> 1) as usize;
        let link_dma_left_sprite_bank =
            (self.player_state_view().link_dma_left_sprite_bank_word() >> 1) as usize;
        let link_dma_right_sprite_bank =
            (self.player_state_view().link_dma_right_sprite_bank_word() >> 1) as usize;
        let link_dma_sword_sprite_bank =
            (self.player_state_view().sword_dma_graphics_index() >> 1) as usize;
        let link_dma_shield_sprite_bank =
            (self.player_state_view().shield_dma_graphics_index() >> 1) as usize;
        let link_dma_staging_index = self.player_state_view().link_dma_staging_index() as usize;
        let link_dma_staging_group = self.player_state_view().link_dma_staging_group() as usize;

        let source3 = link_dma_table_value(
            &LINK_DMA_SOURCES1,
            link_dma_graphics_index,
            "LINK_DMA_SOURCES1",
        );
        self.set_link_body_dma_sources(source3, source3.wrapping_add(0x200));

        let source4 = link_dma_table_value(
            &LINK_DMA_SOURCES2,
            link_dma_graphics_index,
            "LINK_DMA_SOURCES2",
        );
        self.set_link_head_dma_sources(source4, source4.wrapping_add(0x200));

        let left_hand_source = link_dma_table_value(
            &LINK_DMA_SOURCES3,
            link_dma_left_sprite_bank,
            "LINK_DMA_SOURCES3/left_sprite_bank",
        );
        let right_hand_source = link_dma_table_value(
            &LINK_DMA_SOURCES3,
            link_dma_right_sprite_bank,
            "LINK_DMA_SOURCES3/right_sprite_bank",
        );
        self.set_link_hand_dma_sources(left_hand_source, right_hand_source);

        let source6 = link_dma_table_value(
            &LINK_DMA_SOURCES4,
            link_dma_sword_sprite_bank,
            "LINK_DMA_SOURCES4",
        );
        self.set_link_sword_dma_sources(source6, source6.wrapping_add(0x180));

        let source7 = if self.player_state_view().shield_dma_graphics_index() == 0x8b {
            0xe099
        } else {
            link_dma_table_value(
                &LINK_DMA_SOURCES5,
                link_dma_shield_sprite_bank,
                "LINK_DMA_SOURCES5",
            )
        };
        self.set_link_shield_dma_sources(source7, source7.wrapping_add(0x00c0));

        let source8 = link_dma_table_value(
            &LINK_DMA_SOURCES6,
            link_dma_staging_index,
            "LINK_DMA_SOURCES6",
        );
        let aux_source_lower = source8.wrapping_add(link_dma_table_value(
            &LINK_DMA_SOURCES7,
            link_dma_staging_group,
            "LINK_DMA_SOURCES7",
        ));
        self.set_link_aux_dma_sources(source8, aux_source_lower);

        let source10 =
            LINK_DMA_SOURCES8[(self.player_state_view().pushed_block_mode() & 3) as usize];
        self.set_link_push_dma_sources(source10, source10.wrapping_add(0x100));

        if self.decrement_word(BG_TILE_ANIMATION_COUNTDOWN) == 0 {
            let overlay = self.world_state_view().overlay_index() as u16;
            let countdown = if overlay == 0xb5 || overlay == 0xbc {
                0x17
            } else {
                9
            };
            self.reset_bg_tile_animation_countdown(countdown);

            let source_offset = self
                .player_state_view_mut()
                .advance_link_dma_source_offset();
            self.set_animated_tile_data_source_address(0xa680u16.wrapping_add(source_offset));
        }
        if self.player_state_view_mut().decrement_link_dma_countdown() == 0 {
            let t = self.player_state_view_mut().advance_link_dma_tile_offset();

            let index = (t >> 1) as usize;
            self.player_state_view_mut()
                .set_link_dma_countdown(LINK_DMA_CTRS0[index]);
            let source9 = LINK_DMA_SOURCES9[index].wrapping_add(0xb280);
            self.set_link_animated_tile_dma_sources(source9, source9.wrapping_add(0x60));
        }

        let source16 = 0xb940u16
            .wrapping_add((self.player_state_view().dma_head_pointer() as u16).wrapping_mul(2));
        self.set_link_head_pointer_dma_sources(source16, source16.wrapping_add(0x200));

        let source17 = 0xb940u16
            .wrapping_add((self.player_state_view().dma_body_pointer() as u16).wrapping_mul(2));
        self.set_link_body_pointer_dma_sources(source17, source17.wrapping_add(0x200));

        let source20 = 0xb540u16
            .wrapping_add((self.display_state().travel_bird_tile_offset as u16).wrapping_mul(2));
        self.set_travel_bird_dma_sources(source20, source20.wrapping_add(0x200));
    }

    pub(super) fn module_main_routing(&mut self) {
        match self.frame_state().main_module {
            0 => self.Module00_Intro(),
            1 => self.module01_file_select(),
            2 => self.module02_copy_file(),
            3 => self.module03_kill_file(),
            4 => self.module04_name_file(),
            5 => self.module05_load_file(),
            6 => self.module_pre_dungeon(),
            7 => self.module07_dungeon(),
            8 | 10 => self.Module08_OverworldLoad(),
            9 | 11 => self.Module09_Overworld(),
            12 => self.module_reserved_12(),
            13 => self.module_reserved_13(),
            14 => self.Module0E_Interface(),
            15 => self.Module0F_SpotlightClose(),
            16 => self.Module10_SpotlightOpen(),
            17 => self.Module11_DungeonFallingEntrance(),
            18 => self.Module12_GameOver(),
            19 => self.module13_boss_victory_pendant(),
            20 => self.module14_attract(),
            21 => self.module15_mirror_warp_from_aga(),
            22 => self.module16_boss_victory_crystal(),
            23 => self.module17_save_and_quit(),
            24 => self.module18_ganon_emerges(),
            25 => self.module19_triforce_room(),
            26 => self.module1_a_credits(),
            27 => self.Module1B_SpawnSelect(),
            _ => {}
        }
    }
}
