// Methods ported from zelda3/src/attract.c and included inside ZeldaState.

use super::*;

mod attract_shared;
pub(super) use attract_shared::*;

impl ZeldaState {
    pub(super) fn module14_attract(&mut self) {
        let mut state = self.game_state.ending.attract_scene.state();
        if self.game_state.display.screen_brightness != 0
            && self.game_state.display.screen_brightness != 128
            && state != 0
            && state != 2
            && state != 6
            && self.game_state.player.follower_link.filtered_joypad_h() & 0x90 != 0
        {
            self.attract_scene_mut().set_state(9);
            state = 9;
        }
        match state {
            0 => self.attract_fade(),
            1 => self.attract_init_graphics(),
            2 | 6 => self.attract_fade_out_sequence(),
            3 | 7 => self.attract_load_new_scene(),
            4 => self.attract_fade_in_sequence(),
            5 | 8 => self.attract_enact_story(),
            9 => self.attract_skip_to_file_select(),
            _ => {}
        }
    }

    pub(super) fn attract_fade(&mut self) {
        self.intro_handle_all_triforce_animations();
        self.attract_scene_mut().clear_intro_did_run_step();
        self.deactivate_nmi_thread();
        self.intro_periodic_sword_and_intro_flash();
        if self.game_state.display.screen_brightness != 0 {
            self.decrement_screen_brightness();
            return;
        }

        self.enable_force_blank();
        self.set_irq_control_flag(0xff);
        self.deactivate_nmi_thread();
        self.clear_pending_polyhedral_update();
        self.attract_scene_mut().increment_state();
    }

    pub(super) fn attract_init_graphics(&mut self) {
        self.clear_attract_low_work_area();
        if self.rom_startup_timing() {
            self.attract_init_graphics_phase = 4;
            return;
        }
        self.complete_attract_init_graphics();
    }

    pub(super) fn complete_attract_init_graphics(&mut self) {
        self.erase_tile_maps_normal();
        self.Attract_LoadBG3GFX();
        self.set_overworld_palette_mode(4);
        self.set_hud_palette(1);
        self.clear_overworld_aux_or_main_offset();
        self.palette_load_hud();
        self.select_overworld_aux_palette_offset();
        self.palette_load_ow_bg_main();
        self.palette_load_hud();
        self.palette_load_link_armor_and_gloves();
        self.set_main_color_constant(0x1d, 0x3800);
        self.increment_cgram_update_flag();
        self.set_bg3_v_copy2_low(20);
        self.attract_build_backgrounds();
        self.messaging_state_mut().clear_module();
        self.dialogue_message_index_mut().set_value(0x112);
        self.set_bg2_y(0);
        self.attract_scene_mut().set_legend_ctr(0x1010);
        self.attract_scene_mut().add_state(3);

        self.hdma_setup(0x0cfa87, 0x0cfa94, 1, 0x26, 0x28, 0);
        self.set_hdma_enable_mask(0xc0);
        self.set_bg12_window_selection(0);
        self.set_bg34_window_selection(0);
        self.set_object_color_window_selection(0xb0);
        self.set_main_screen_window_layers(3);
        self.set_sub_screen_window_layers(0);
        self.set_fixed_color_red(0x25);
        self.set_fixed_color_green(0x45);
        self.set_fixed_color_blue(0x85);
        self.set_color_window_selection(0x10);
        self.set_color_math_control(0xa3);
        self.set_music_control(6);
        self.attract_scene_mut().increment_legend_flag();
    }

    pub(super) fn attract_load_new_scene(&mut self) {
        match self.game_state.ending.attract_scene.sequence() {
            0 => self.attract_scene_polka_dots(),
            1 => self.attract_scene_world_map(),
            2 => self.attract_scene_throne_room(),
            3 => self.attract_prep_zelda_prison(),
            4 => self.attract_prep_maiden_warp(),
            5 => self.attract_scene_end_of_story(),
            _ => {}
        }
    }

    pub(super) fn attract_scene_polka_dots(&mut self) {
        self.attract_scene_mut().clear_next_legend_gfx();
        self.attract_scene_mut().increment_state();
        self.set_screen_brightness(0);
    }

    pub(super) fn attract_scene_world_map(&mut self) {
        if self.rom_startup_timing() {
            self.begin_attract_world_map_work();
            return;
        }
        self.complete_attract_scene_world_map();
    }

    pub(super) fn complete_attract_scene_world_map(&mut self) {
        self.zelda_ppu_write(0x2107, 0x13);
        self.zelda_ppu_write(0x2108, 0x03);
        self.set_color_window_selection(0x80);
        self.set_color_math_control(0x21);
        self.set_bg_mode(7);
        self.world_map_load_light_world_map();
        self.set_mode7_center_y(0x00ed);
        self.set_mode7_center_x(0x0100);
        self.set_bg1_h_copy(0x0080);
        self.set_bg1_v_copy(0x00c0);
        self.attract_scene_mut().set_mode7_zoom_timer(0xff);
        self.attract_control_map_zoom();
        self.attract_scene_mut().set_scene_timer(1);
        self.attract_scene_mut().increment_state();
        self.set_screen_brightness(0);
    }

    pub(super) fn attract_scene_throne_room(&mut self) {
        if self.rom_startup_timing() {
            self.begin_attract_throne_room_work();
            return;
        }
        self.complete_attract_scene_throne_room();
    }

    pub(super) fn complete_attract_scene_throne_room(&mut self) {
        self.clear_hdma_enable_mask();
        self.set_color_window_selection(2);
        self.set_color_math_control(0x20);
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(10);
        self.load_common_sprites();

        let attract_bg2_vofs_backup = self.game_state.ending.attract_scene.bg2_vofs_backup();
        let attract_state = self.game_state.ending.attract_scene.state_word();
        self.dungeon_load_and_draw_entrance_room(0x74);
        self.attract_scene_mut().set_state_word(attract_state);
        self.attract_scene_mut()
            .set_bg2_vofs_backup(attract_bg2_vofs_backup);

        self.set_palette_main_indoors(0);
        self.set_sp0l(0);
        self.set_sp5l(14);
        self.set_sp6l(3);
        self.dungeon_save_and_load_load_all_palettes(0, 0x7e);

        self.set_main_color_constant(0x1d, 0x3800);
        self.messaging_state_mut().clear_module();
        self.dialogue_message_index_mut().set_value(0x113);
        self.attract_scene_mut().set_scene_timer(2);
        self.attract_scene_mut().set_throne_fade_timer(0xe0);
        self.oam_state_mut().set_priority_word(0x210);

        self.attract_prep_finish();
    }

    pub(super) fn attract_prep_zelda_prison(&mut self) {
        if self.rom_startup_timing() {
            self.begin_attract_zelda_prison_work();
            return;
        }
        self.complete_attract_prep_zelda_prison();
    }

    pub(super) fn complete_attract_prep_zelda_prison(&mut self) {
        self.set_color_window_selection(0);
        self.set_color_math_control(0);

        let attract_bg2_vofs_backup = self.game_state.ending.attract_scene.bg2_vofs_backup();
        let attract_state = self.game_state.ending.attract_scene.state_word();
        self.dungeon_load_and_draw_entrance_room(0x73);
        self.attract_scene_mut().set_state_word(attract_state);
        self.attract_scene_mut()
            .set_bg2_vofs_backup(attract_bg2_vofs_backup);

        self.set_palette_main_indoors(2);
        self.set_sp0l(0);
        self.set_sp5l(14);
        self.set_sp6l(3);
        self.dungeon_save_and_load_load_all_palettes(1, 0x7f);
        self.set_main_color_constant(0x1d, 0x3800);

        self.messaging_state_mut().clear_module();
        self.dialogue_message_index_mut().set_value(0x114);

        self.attract_scene_mut().set_prison_zelda_y_base(148);
        self.set_attract_vram_destination_address(0x0068);
        self.attract_scene_mut().set_anim_step_counter(0);
        self.attract_scene_mut().set_soldier_anim_step(0);
        self.attract_scene_mut().set_x_base_high(0);
        self.attract_scene_mut().set_scene_frame_counter(0);
        self.attract_scene_mut().clear_fade_in_done_flag();
        self.attract_scene_mut().set_scene_timer(255);
        self.oam_state_mut().set_priority_word(0x240);
        self.attract_prep_finish();
    }

    pub(super) fn attract_prep_maiden_warp(&mut self) {
        if self.rom_startup_timing() {
            self.begin_attract_maiden_warp_work();
            return;
        }
        self.complete_attract_prep_maiden_warp();
    }

    pub(super) fn complete_attract_prep_maiden_warp(&mut self) {
        let attract_bg2_vofs_backup = self.game_state.ending.attract_scene.bg2_vofs_backup();
        let attract_state = self.game_state.ending.attract_scene.state_word();
        self.dungeon_load_and_draw_entrance_room(0x75);
        self.attract_scene_mut().set_state_word(attract_state);
        self.attract_scene_mut()
            .set_bg2_vofs_backup(attract_bg2_vofs_backup);

        self.set_palette_main_indoors(0);
        self.set_sp0l(0);
        self.set_sp5l(14);
        self.set_sp6l(3);
        self.clear_overworld_aux_or_main_offset();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sprite_environment_dungeon();
        self.palette_load_hud();
        self.palette_load_dungeon_set();
        self.dungeon_save_and_load_load_all_palettes(2, 0x7f);
        self.set_aux_color_constant(0x1d, 0x3800);
        self.set_main_color_constant(0x1d, 0x3800);

        self.messaging_state_mut().clear_module();
        self.dialogue_message_index_mut().set_value(0x115);
        self.attract_scene_mut().set_scene_timer(255);
        self.set_attract_vram_destination_page_offset(112);
        self.attract_scene_mut().set_maiden_warp_timer_a(112);
        self.attract_scene_mut().set_maiden_warp_timer_b(112);
        self.attract_scene_mut().set_anim_step_counter(8);
        self.attract_scene_mut().set_scene_frame_counter(0);
        self.attract_scene_mut().set_maiden_warp_step(0);
        self.attract_scene_mut().set_fade_in_complete_flag(0);
        self.attract_scene_mut().clear_fade_in_done_flag();
        self.attract_scene_mut().set_scene_substep(0);
        self.attract_scene_mut().clear_substep_delay_counter();
        self.oam_state_mut().set_priority_word(0x00c0);
        self.attract_prep_finish();
    }

    pub(super) fn dungeon_load_and_draw_entrance_room(&mut self, room: u8) {
        self.Dungeon_LoadAndDrawEntranceRoom(room);
    }

    #[rustfmt::skip]
    pub(super) fn dungeon_save_and_load_load_all_palettes(&mut self, a: u8, k: u8) {
        self.Dungeon_SaveAndLoadAllPalettes(a, k);
    }

    pub(super) fn attract_scene_end_of_story(&mut self) {
        if self.rom_startup_timing() {
            self.begin_attract_end_of_story_work();
            return;
        }
        self.complete_attract_scene_end_of_story();
    }

    pub(super) fn complete_attract_scene_end_of_story(&mut self) {
        self.attract_setup_conclusion_hdma();
        self.death_func31();
    }

    pub(super) fn attract_setup_conclusion_hdma(&mut self) {
        self.hdma_setup(0x0abddd, 0x0abddd, 0x42, 0x1b, 0x1e, 0);
        self.set_hdma_enable_mask(0x80);
        self.set_bg_mode(9);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn death_func31(&mut self) {
        self.increment_core_update_disable_flag();
        self.intro_initialize_memory_darken();
        self.overworld_load_all_palettes();
        self.set_bg3_v_copy2_low(0);
        self.set_mode7_center_y(0);
        self.set_mode7_center_x(0);
        self.set_bg1_h_copy(0);
        self.set_bg1_v_copy(0);
        self.set_bg2_h_copy(0);
        self.set_bg2_v_copy(0);
        self.set_music_control(0xf1);
        self.attract_scene_mut().set_sequence(0);
        self.set_main_module(0);
        self.set_submodule(10);
        self.set_subsubmodule(10);
    }

    pub(super) fn attract_skip_to_file_select(&mut self) {
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        self.enable_force_blank();
        self.zelda_ppu_write(0x2107, 0x13);
        self.zelda_ppu_write(0x2108, 0x03);
        self.attract_setup_conclusion_hdma();
        self.set_mode7_center_y(0);
        self.set_mode7_center_x(0);
        self.set_bg1_h_copy(0);
        self.set_bg1_v_copy(0);
        self.set_bg3_v_copy2_low(0);
        self.fade_music_and_reset_sram_mirror();
    }

    pub(super) fn attract_prep_finish(&mut self) {
        self.attract_scene_mut().increment_state();
        self.set_screen_brightness(0);
        self.set_bg3_v_copy2_low(0);
        let bg2_hofs = self.game_state.display.ppu_scroll_copy.bg2_h_copy() & 0x01ff;
        let bg2_vofs = self.game_state.display.ppu_scroll_copy.bg2_v_copy() & 0x01ff;
        let bg2_hofs2 = self.game_state.display.ppu_scroll_copy.bg2_h_copy2() & 0x01ff;
        let bg2_vofs2 = self.game_state.display.ppu_scroll_copy.bg2_v_copy2() & 0x01ff;
        self.set_bg2_h_copy(bg2_hofs);
        self.set_bg2_v_copy(bg2_vofs);
        self.set_bg2_x(bg2_hofs2);
        self.set_bg2_y(bg2_vofs2);
    }

    pub(super) fn attract_control_map_zoom(&mut self) {
        let zoom = self.game_state.ending.attract_scene.mode7_zoom_timer() as u16;
        if self.rom_startup_timing() {
            self.attract_map_hdma_projection_before = Some(self.hdma_dynamic_table_bytes());
        }
        // The ROM loop starts at X=$01BE and stops after word 223. The final
        // sixteen constants are outside the 224-line HDMA scanout.
        for (i, value) in ATTRACT_MAP_ZOOM_HDMA_BASES[..ATTRACT_MAP_PROJECTION_WORDS]
            .iter()
            .enumerate()
        {
            self.set_spotlight_hdma_table_dynamic_entry(
                i,
                ((*value as u32 * zoom as u32) >> 8) as u16,
            );
        }
    }

    pub(super) fn attract_fade_in_sequence(&mut self) {
        if self.game_state.display.screen_brightness != 15 {
            if (self.follower_link_state_mut().decrement_speed_setting() as i8) < 0 {
                self.increment_screen_brightness();
                self.follower_link_state_mut().set_speed_setting(1);
            }
        } else {
            self.attract_scene_mut().increment_state();
            if self.rom_startup_timing() {
                // Sequence 1 begins the map story on the fade-completion CPU
                // slice. Its display publication still crosses later NMI
                // boundaries, which is modeled separately by the renderer
                // snapshot schedule.
                if self.game_state.ending.attract_scene.sequence() == 1 {
                    self.attract_enact_story();
                } else {
                    self.attract_first_story_render_delay = rom_attract_story_render_nmi_slices(
                        self.game_state.ending.attract_scene.sequence(),
                    );
                }
            }
        }
    }

    pub(super) fn attract_fade_out_sequence(&mut self) {
        if self.game_state.display.screen_brightness != 0 {
            if (self.follower_link_state_mut().decrement_speed_setting() as i8) < 0 {
                self.decrement_screen_brightness();
                self.follower_link_state_mut().set_speed_setting(1);
            }
        } else {
            self.enable_force_blank();
            self.erase_tile_maps_normal();
            self.attract_scene_mut().increment_state();
        }
    }

    pub(super) fn attract_enact_story(&mut self) {
        match self.game_state.ending.attract_scene.sequence() {
            0 => self.attract_dramatize_polka_dots(),
            1 => self.attract_dramatize_world_map(),
            2 => self.attract_throne_room(),
            3 => self.attract_dramatize_prison(),
            4 => self.attract_dramatize_agahnim_altar(),
            _ => {}
        }
    }

    pub(super) fn attract_dramatize_world_map(&mut self) {
        if self.game_state.ending.attract_scene.mode7_zoom_timer() != 0 {
            if self.game_state.ending.attract_scene.mode7_zoom_timer() < 15 {
                self.decrement_screen_brightness();
            }
            self.attract_scene_mut().decrement_scene_timer();
            if self.game_state.ending.attract_scene.scene_timer() == 0 {
                self.attract_scene_mut().set_scene_timer(1);
                self.attract_scene_mut().decrement_mode7_zoom_timer();
                self.attract_control_map_zoom();
            }
        } else {
            self.enable_force_blank();
            self.set_bg_mode(9);
            self.erase_tile_maps_normal();
            self.attract_scene_mut().increment_sequence();
            self.attract_scene_mut().subtract_state(2);
        }
    }

    pub(super) fn attract_throne_room(&mut self) {
        self.attract_scene_mut().set_oam_index(0);
        if self.game_state.ending.attract_scene.fade_in_complete_flag() == 0 {
            if self.game_state.display.screen_brightness != 15 {
                self.increment_screen_brightness();
            } else {
                self.attract_scene_mut().increment_fade_in_complete_flag();
            }
        }

        if self.game_state.display.ppu_scroll_copy.bg2_v_copy() == 0 {
            self.attract_show_timed_text_message();
            if self.game_state.oam.priority_word() == 0 {
                if self.game_state.ending.attract_scene.throne_fade_timer() < 31
                    && self.game_state.ending.attract_scene.throne_fade_timer() & 1 == 0
                {
                    self.decrement_screen_brightness();
                }
                let throne_fade = self.attract_scene_mut().decrement_throne_fade_timer();
                if throne_fade == 0 {
                    self.attract_scene_mut().increment_sequence();
                    self.attract_scene_mut().increment_state();
                    return;
                }
            }
        } else {
            let bg2 = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy()
                .wrapping_sub(1);
            let bg1 = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg1_v_copy()
                .wrapping_sub(1);
            self.set_bg2_v_copy(bg2);
            self.set_bg1_v_copy(bg1);
        }

        let bg2_vofs = self.game_state.display.ppu_scroll_copy.bg2_v_copy();
        for i in (0..=1).rev() {
            let y = (ATTRACT_THRONE_ROOM_Y_BASES[i] as u16).wrapping_sub(bg2_vofs);
            if (y.wrapping_add(32) as i16) >= 0 {
                self.attract_scene_mut()
                    .set_x_base(ATTRACT_THRONE_ROOM_X_BASES[i]);
                self.attract_scene_mut().set_y_base(y as u8);
                self.attract_draw_sprite_set2_slice(
                    &ATTRACT_THRONE_ROOM_SPRITE_ENTRIES[ATTRACT_THRONE_ROOM_SPRITE_ENTRY_STARTS[i]
                        ..ATTRACT_THRONE_ROOM_SPRITE_ENTRY_STARTS[i + 1]],
                );
            }
        }

        self.attract_scene_mut().set_story_text_pointer(0xf8a7);
    }

    pub(super) fn attract_show_timed_text_message(&mut self) {
        let bg2_vofs2 = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.attract_scene_mut().set_bg2_vofs_backup(bg2_vofs2);
        self.follower_link_state_mut().set_joypad1l_last(0);
        self.follower_link_state_mut().set_filtered_joypad_l(0);
        self.follower_link_state_mut().set_filtered_joypad_h(0);
        self.RenderText();
        self.tick_attract_timed_text_priority();
    }

    fn tick_attract_timed_text_priority(&mut self) {
        let priority = self.game_state.oam.priority_word();
        if priority != 0 {
            self.oam_state_mut()
                .set_priority_word(priority.wrapping_sub(1));
        }
    }

    #[rustfmt::skip]
    pub(super) fn attract_draw_sprite_set2(&mut self, p: *const AttractOamInfo, n: i32) {
        self.attract_draw_sprite_set2_slice(unsafe {
            std::slice::from_raw_parts(p, n as usize)
        });
    }

    pub(super) fn attract_draw_sprite_set2_slice(&mut self, entries: &[AttractOamInfo]) {
        let start = self.game_state.ending.attract_scene.oam_index() as usize;
        self.attract_scene_mut()
            .advance_oam_index_by(entries.len() as u8);
        for (slot, &(x, y, charnum, flags, big)) in entries.iter().rev().enumerate() {
            let obj_x = self
                .game_state
                .ending
                .attract_scene
                .x_base()
                .wrapping_add_signed(x);
            let obj_y = self
                .game_state
                .ending
                .attract_scene
                .y_base()
                .wrapping_add_signed(y);
            self.set_oam_plain(64 + start + slot, obj_x, obj_y, charnum, flags, big);
        }
    }

    pub(super) fn attract_dramatize_agahnim_altar(&mut self) {
        if self.game_state.ending.attract_scene.scene_done_flag() != 0 {
            self.attract_scene_mut().increment_sequence();
            self.attract_scene_mut().subtract_state(2);
            return;
        }

        self.attract_scene_mut().set_oam_index(0);
        self.handle_screen_flash();
        if self.game_state.ending.attract_scene.fade_in_done_flag() == 0 {
            self.attract_fade_in_step();
        }
        if self.game_state.ending.attract_scene.scene_frame_counter() != 0xff {
            self.attract_scene_mut().increment_scene_frame_counter();
        }
        if self
            .game_state
            .ending
            .attract_scene
            .intro_palette_flash_count()
            & 4
            != 0
        {
            self.set_sound_effect_2(0x2b);
        }

        match self.game_state.ending.attract_scene.scene_substep() {
            0 => self.attract_maiden_warp_case0(),
            1 => self.attract_maiden_warp_case1(),
            2 => self.attract_maiden_warp_case2(),
            3 => self.attract_maiden_warp_case3(),
            4 => self.attract_maiden_warp_case4(),
            _ => {}
        }

        for k in (0..=5).rev() {
            self.sprite_prep_reset_properties(k);
            self.sprite_simulate_soldier(
                k,
                ATTRACT_DRAMATIZE_AGAHNIM_ALTAR_SOLDIER_X[k],
                ATTRACT_DRAMATIZE_AGAHNIM_ALTAR_SOLDIER_Y[k],
                ATTRACT_DRAMATIZE_AGAHNIM_ALTAR_SOLDIER_DIR[k],
                ATTRACT_DRAMATIZE_AGAHNIM_ALTAR_SOLDIER_FLAGS[k],
                0,
            );
        }

        if self.game_state.ending.attract_scene.scene_frame_counter() >= 0xa0 {
            if self.attract_vram_destination_page_offset() != 0x60 {
                let step = self.attract_scene_mut().decrement_anim_step_counter();
                if step == 0 {
                    self.decrement_attract_vram_destination_page_offset();
                    self.attract_scene_mut().set_anim_step_counter(8);
                }
            } else {
                self.attract_scene_mut().increment_substep_delay_counter();
            }
        }

        if self.game_state.ending.attract_scene.fade_in_complete_flag() == 0 {
            self.attract_scene_mut().set_x_base(116);
            let y_base = self.attract_vram_destination_page_offset();
            self.attract_scene_mut().set_y_base(y_base);
            let start = if self.attract_vram_destination_page_offset() == 0x70 {
                0
            } else {
                2
            };
            self.attract_draw_sprite_set2_slice(
                &ATTRACT_AGAHNIM_ALTAR_MAIDEN_CORE_ENTRIES[start..start + 2],
            );

            let mut k = 7usize;
            if self.attract_vram_destination_page_offset() < 0x68 {
                k = self
                    .attract_vram_destination_page_offset()
                    .wrapping_sub(0x68) as usize
                    & 7;
            }
            self.attract_scene_mut()
                .set_x_base(0x74 + ATTRACT_AGAHNIM_ALTAR_MAIDEN_X_BASE_OFFSETS[k]);
            self.attract_scene_mut().set_y_base(0x76);
            self.attract_draw_sprite_set2_slice(
                &ATTRACT_AGAHNIM_ALTAR_MAIDEN_SHIMMER_ENTRIES[k * 2..k * 2 + 2],
            );
        }

        let k = (self.game_state.ending.attract_scene.scene_frame_counter() >> 5) as usize & 7;
        self.attract_scene_mut().set_x_base(112);
        self.attract_scene_mut().set_y_base(70);
        self.attract_draw_sprite_set2_slice(
            &ATTRACT_AGAHNIM_ALTAR_MAIDEN_WARP_ENTRIES[k * 6..k * 6 + 6],
        );
    }

    pub(super) fn attract_maiden_warp_case0(&mut self) {
        if self.game_state.ending.attract_scene.substep_delay_counter() != 0 {
            self.attract_scene_mut().increment_scene_substep();
        }
    }

    pub(super) fn attract_maiden_warp_case1(&mut self) {
        let k = (self.game_state.frame.frame_counter >> 2) as usize & 1;
        let n = ATTRACT_MAIDEN_WARP_CASE1_ENTRY_COUNTS
            [(self.game_state.ending.attract_scene.maiden_warp_step() >> 1) as usize & 7];
        self.attract_scene_mut().set_x_base(110);
        self.attract_scene_mut().set_y_base(72);
        self.attract_draw_sprite_set2_slice(
            &ATTRACT_MAIDEN_WARP_CASE1_SPRITE_ENTRIES[k * 14..k * 14 + n],
        );

        if self.game_state.ending.attract_scene.maiden_warp_step() == 0
            && self.game_state.ending.attract_scene.maiden_warp_timer_b() == 0x70
        {
            self.set_sound_effect_2(0x27);
        }

        if self.game_state.ending.attract_scene.maiden_warp_step() == 15 {
            self.attract_scene_mut().increment_scene_substep();
        } else {
            if self.game_state.ending.attract_scene.maiden_warp_step() == 6 {
                self.attract_scene_mut().set_intro_palette_flash_count(0x90);
                self.set_sound_effect_2(0x2b);
            }
            if self.game_state.ending.attract_scene.maiden_warp_timer_b() != 0 {
                self.attract_scene_mut().decrement_maiden_warp_timer_b();
            } else {
                self.attract_scene_mut().increment_maiden_warp_step();
            }
        }
    }

    pub(super) fn attract_maiden_warp_case2(&mut self) {
        self.attract_scene_mut().set_x_base(110);
        self.attract_scene_mut().set_y_base(72);
        let k = (self.game_state.frame.frame_counter >> 2) as usize & 1;
        let n = ATTRACT_MAIDEN_WARP_CASE2_ENTRY_COUNTS
            [(self.game_state.ending.attract_scene.maiden_warp_step() >> 1) as usize & 7];
        self.attract_draw_sprite_set2_slice(
            &ATTRACT_MAIDEN_WARP_CASE2_SPRITE_ENTRIES[k * 14 + (14 - n)..k * 14 + 14],
        );

        if self.game_state.ending.attract_scene.maiden_warp_step() == 0 {
            let timer = self.attract_scene_mut().decrement_maiden_warp_timer_a();
            if timer == 0 {
                self.attract_scene_mut().increment_scene_substep();
            }
        } else {
            self.attract_scene_mut().decrement_maiden_warp_step();
        }
    }

    pub(super) fn attract_maiden_warp_case3(&mut self) {
        if self.game_state.ending.attract_scene.maiden_warp_step() == 6 {
            self.attract_scene_mut().increment_fade_in_complete_flag();
            self.set_sound_effect_1(51);
        } else if self.game_state.ending.attract_scene.maiden_warp_step() == 0x40 {
            self.attract_scene_mut().set_maiden_warp_step(224);
            self.attract_scene_mut().increment_scene_substep();
        } else if self.game_state.ending.attract_scene.maiden_warp_step() < 0x0f {
            let k = (self.game_state.ending.attract_scene.maiden_warp_step() >> 3) as usize & 1;
            self.attract_scene_mut()
                .set_x_base(ATTRACT_MAIDEN_WARP_CASE3_X_BASES[k]);
            self.attract_scene_mut().set_y_base(0x60);
            let n = if k != 0 { 2 } else { 1 };
            self.attract_draw_sprite_set2_slice(
                &ATTRACT_MAIDEN_WARP_CASE3_SPRITE_ENTRIES[k..k + n],
            );
        }
        self.attract_scene_mut().increment_maiden_warp_step();
    }

    pub(super) fn attract_maiden_warp_case4(&mut self) {
        self.attract_show_timed_text_message();
        if self.game_state.oam.priority_word() == 0 {
            if self.game_state.ending.attract_scene.maiden_warp_step() < 31
                && self.game_state.ending.attract_scene.maiden_warp_step() & 1 == 0
            {
                self.decrement_screen_brightness();
            }
            self.attract_scene_mut().decrement_maiden_warp_step();
            if self.game_state.ending.attract_scene.maiden_warp_step() == 0 {
                self.attract_scene_mut().increment_scene_done_flag();
            }
        }
    }

    pub(super) fn attract_dramatize_prison(&mut self) {
        self.attract_scene_mut().set_oam_index(0);
        if self.game_state.ending.attract_scene.fade_in_done_flag() == 0 {
            self.attract_fade_in_step();
        }

        self.attract_scene_mut().set_x_base(56);
        self.attract_draw_zelda();

        if self.game_state.ending.attract_scene.scene_timer() >= 192 {
            self.attract_scene_mut().set_y_base(112);
            let frame_ctr = self.attract_scene_mut().decrement_scene_frame_counter();
            let frame_ctr = if (frame_ctr as i8) < 0 {
                self.attract_scene_mut().set_scene_frame_counter(0x0f);
                0x0f
            } else {
                frame_ctr
            };

            let t = self.attract_vram_destination_address().wrapping_add(
                ATTRACT_DRAMATIZE_PRISON_ZELDA_ANIMATION_FRAMES[frame_ctr as usize] as u16,
            );
            self.attract_scene_mut().set_x_base_high((t >> 8) as u8);
            self.attract_scene_mut().set_x_base(t as u8);
            self.attract_zelda_prison_draw_a();

            for k in (0..=1).rev() {
                let sprite = k * 2;
                self.sprite_prep_reset_properties(sprite);
                let x = (self.attract_vram_destination_address() as i16)
                    .wrapping_add(0x100)
                    .wrapping_add(ATTRACT_DRAMATIZE_PRISON_SOLDIER_X[k])
                    as u16;
                self.attract_scene_mut().set_prison_soldier_x_lo(x as u8);
                let soldier_anim = self.game_state.ending.attract_scene.soldier_anim_step();
                self.sprite_simulate_soldier(
                    sprite,
                    x,
                    (self.game_state.ending.attract_scene.y_base() as u16)
                        .wrapping_add(ATTRACT_DRAMATIZE_PRISON_SOLDIER_Y[k]),
                    ATTRACT_DRAMATIZE_PRISON_SOLDIER_DIR[k],
                    ATTRACT_DRAMATIZE_PRISON_SOLDIER_FLAGS[k],
                    soldier_anim,
                );
            }

            let step_ctr = self.attract_scene_mut().increment_anim_step_counter();
            if step_ctr & 7 == 0 {
                if self.game_state.ending.attract_scene.soldier_anim_step() == 2 {
                    self.attract_scene_mut().set_soldier_anim_step(0xff);
                    if self.attract_vram_destination_high_is_clear() && step_ctr & 8 != 0 {
                        self.set_sound_effect_2(4);
                    }
                }
                self.attract_scene_mut().increment_soldier_anim_step();
            }
        }

        match self.game_state.ending.attract_scene.scene_substep() {
            0 => self.attract_zelda_prison_case0(),
            1 => self.attract_zelda_prison_case1(),
            _ => {}
        }
    }

    pub(super) fn attract_fade_in_step(&mut self) {
        if self.game_state.display.screen_brightness != 15 {
            if (self.follower_link_state_mut().decrement_speed_setting() as i8) < 0 {
                self.increment_screen_brightness();
                self.follower_link_state_mut().set_speed_setting(1);
            }
        } else {
            self.attract_scene_mut().increment_fade_in_done_flag();
        }
    }

    pub(super) fn attract_draw_zelda(&mut self) {
        let start = self.game_state.ending.attract_scene.oam_index() as usize;
        self.set_oam_plain(
            64 + start,
            0x60,
            self.game_state.ending.attract_scene.x_base(),
            0x28,
            0x29,
            2,
        );
        self.set_oam_plain(
            65 + start,
            0x60,
            self.game_state
                .ending
                .attract_scene
                .x_base()
                .wrapping_add(10),
            0x2a,
            0x29,
            2,
        );
        self.attract_scene_mut().advance_oam_index_by(2);
    }

    #[rustfmt::skip]
    pub(super) fn attract_draw_preloaded_sprite(&mut self, xp: *const u8, yp: *const u8, cp: *const u8, fp: *const u8, ep: *const u8, n: i32) {
        let len = n as usize + 1;
        self.attract_draw_preloaded_sprite_slice(
            unsafe { std::slice::from_raw_parts(xp, len) },
            unsafe { std::slice::from_raw_parts(yp, len) },
            unsafe { std::slice::from_raw_parts(cp, len) },
            unsafe { std::slice::from_raw_parts(fp, len) },
            unsafe { std::slice::from_raw_parts(ep, len) },
            n as usize,
        );
    }

    pub(super) fn attract_draw_preloaded_sprite_slice(
        &mut self,
        xp: &[u8],
        yp: &[u8],
        cp: &[u8],
        fp: &[u8],
        ep: &[u8],
        n: usize,
    ) {
        let start = self.game_state.ending.attract_scene.oam_index() as usize;
        self.attract_scene_mut()
            .advance_oam_index_by((n as u8).wrapping_add(1));
        for i in (0..=n).rev() {
            self.set_oam_plain(
                64 + start + (n - i),
                self.game_state
                    .ending
                    .attract_scene
                    .x_base()
                    .wrapping_add(xp[i]),
                self.game_state
                    .ending
                    .attract_scene
                    .y_base()
                    .wrapping_add(yp[i]),
                cp[i],
                fp[i],
                ep[i],
            );
        }
    }

    pub(super) fn attract_zelda_prison_draw_a(&mut self) {
        let start = self.game_state.ending.attract_scene.oam_index() as usize;
        let ext = if self.game_state.ending.attract_scene.x_base_high() != 0 {
            3
        } else {
            2
        };
        let j = (self.game_state.ending.attract_scene.anim_step_counter() >> 3) & 1;
        self.set_oam_plain(
            64 + start,
            self.game_state.ending.attract_scene.x_base(),
            self.game_state
                .ending
                .attract_scene
                .y_base()
                .wrapping_add(j),
            6,
            0x3d,
            ext,
        );
        self.set_oam_plain(
            65 + start,
            self.game_state.ending.attract_scene.x_base(),
            self.game_state
                .ending
                .attract_scene
                .y_base()
                .wrapping_add(10),
            if j != 0 { 10 } else { 8 },
            0x3d,
            ext,
        );
        self.attract_scene_mut().advance_oam_index_by(2);
    }

    pub(super) fn attract_zelda_prison_case0(&mut self) {
        if self.game_state.ending.attract_scene.prison_soldier_x_lo() == 0 {
            self.attract_scene_mut().increment_scene_substep();
        }
        if self.game_state.frame.frame_counter & 1 != 0 {
            self.decrement_attract_vram_destination_address();
        }
        self.attract_scene_mut().set_x_base(0x58);
        let y_base = self.game_state.ending.attract_scene.prison_zelda_y_base();
        self.attract_scene_mut().set_y_base(y_base);
        self.attract_draw_sprite_set2_slice(&ATTRACT_ZELDA_PRISON_CASE0_SPRITE_ENTRIES);
        self.attract_scene_mut().set_story_text_pointer(0xf8d9);
    }

    pub(super) fn attract_zelda_prison_case1(&mut self) {
        let Some(k) = (if self.game_state.ending.attract_scene.scene_timer() < 0x80 {
            self.attract_show_timed_text_message();
            if self.game_state.oam.priority_word() != 0 {
                Some(4)
            } else {
                self.attract_zelda_prison_case1_step()
            }
        } else {
            self.attract_zelda_prison_case1_step()
        }) else {
            return;
        };

        if self.game_state.frame.frame_counter & 1 != 0 {
            self.decrement_attract_vram_destination_address();
        }
        self.attract_scene_mut().set_x_base(0x58);
        let y_base = self.game_state.ending.attract_scene.prison_zelda_y_base();
        self.attract_scene_mut().set_y_base(y_base);
        self.attract_draw_sprite_set2_slice(
            &ATTRACT_ZELDA_PRISON_CASE1_SPRITE_ENTRIES[k * 6..k * 6 + 6],
        );
    }

    fn attract_zelda_prison_case1_step(&mut self) -> Option<usize> {
        if self.game_state.ending.attract_scene.prison_zelda_y_base() != 0x6e {
            self.attract_scene_mut().decrement_prison_zelda_y_base();
            return Some(0);
        }

        if self.game_state.ending.attract_scene.scene_timer() < 31
            && self.game_state.ending.attract_scene.scene_timer() & 1 == 0
        {
            self.decrement_screen_brightness();
        }
        self.attract_scene_mut().decrement_scene_timer();
        if self.game_state.ending.attract_scene.scene_timer() == 0 {
            self.attract_scene_mut().increment_sequence();
            self.attract_scene_mut().subtract_state(2);
            return None;
        }

        Some(match self.game_state.ending.attract_scene.scene_timer() {
            0xc0..=0xff => 0,
            0xb8..=0xbf => 1,
            0xb0..=0xb7 => 2,
            0xa0..=0xaf => 3,
            _ => 4,
        })
    }

    #[rustfmt::skip]
    pub(super) fn sprite_simulate_soldier(&mut self, k: usize, x: u16, y: u16, dir: u8, flags: u8, gfx: u8) {

        self.sprite_set_x(k, x);
        self.sprite_set_y(k, y);
        self.sprite_slot_view_mut(k).set_z(0);
        self.sprite_get_16_bit_coords(k);
        let mut soldier = self.sprite_slot_view_mut(k);
        soldier.set_direction(dir);
        soldier.set_head_direction(dir);
        soldier.set_graphics(SPRITE_SIMULATE_SOLDIER_GRAPHICS_BY_DIRECTION[dir as usize].wrapping_add(gfx));
        soldier.set_flags3(16);
        soldier.set_object_priority(0);
        soldier.set_oam_flags(flags | 0x30);
        soldier.set_sprite_type(if flags == 9 { 0x41 } else { 0x43 });
        soldier.set_flags2(7);
        let oam_idx = k * 8;
        self.oam_state_mut()
            .set_current_pointer((OAM_BUF + oam_idx * 4) as u16);
        self.oam_state_mut()
            .set_current_extended_pointer((BYTEWISE_EXTENDED_OAM + oam_idx) as u16);
        self.guard_handle_all_animation(k);
    }

    fn sprite_get_16_bit_coords(&mut self, k: usize) {
        self.sprite_get16_bit_coords(k);
    }

    pub(super) fn sprite_draw_shadow_custom_attract(
        &mut self,
        k: usize,
        poc: (u16, u16, u8),
        y_offset: u8,
    ) {
        let y = self
            .sprite_slot_view(k)
            .y()
            .wrapping_add(y_offset as u16)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if y.wrapping_add(0x10) >= 0x100 {
            return;
        }
        let oam_offs = (self.sprite_slot_view(k).flags2() & 0x1f) as usize;
        self.set_guard_oam(oam_offs, poc.0, y, 0x6c, (poc.2 & 0x30) | 8, 2);
    }

    fn set_guard_oam(&mut self, offset: usize, x: u16, y: u16, charnum: u8, flags: u8, big: u8) {
        let oam_cur = self.game_state.oam.current_pointer_usize();
        let index = (oam_cur - OAM_BUF) / 4 + offset;
        self.set_oam_helper0_index(index, x, y, charnum, flags, big);
    }

    pub(super) fn attract_dramatize_polka_dots(&mut self) {
        if self.game_state.frame.frame_counter & 3 == 0 {
            self.add_bg1_v_copy_low(1);
            self.add_bg1_h_copy_low(1);
            self.add_bg2_v_copy_low(1);
            self.subtract_bg2_h_copy_low(1);
        }

        if self.game_state.ending.attract_scene.legend_flag() != 0 {
            self.attract_build_next_image_tile_map();
            self.attract_scene_mut().clear_legend_flag();
            self.attract_scene_mut().advance_next_legend_gfx();
        }

        self.follower_link_state_mut().set_joypad1l_last(0);
        self.follower_link_state_mut().set_filtered_joypad_l(0);
        self.follower_link_state_mut().set_filtered_joypad_h(0);
        self.RenderText();

        let legend_ctr = self.attract_scene_mut().decrement_legend_ctr();
        if legend_ctr == 0 {
            self.attract_scene_mut().increment_sequence();
            self.attract_scene_mut().subtract_state(3);
        } else if legend_ctr < 0x18 && legend_ctr & 1 != 0 {
            self.decrement_screen_brightness();
        }
    }

    pub(super) fn attract_build_next_image_tile_map(&mut self) {
        let image = self.game_state.ending.attract_scene.next_legend_image();
        self.attract_build_legend_image_tile_map(image);
    }

    pub(super) fn attract_build_legend_image_tile_map(&mut self, image: u8) {
        let data = match image {
            0 => &ATTRACT_LEGEND_TILEMAP_BYTES_0[..],
            1 => &ATTRACT_LEGEND_TILEMAP_BYTES_1[..],
            2 => &ATTRACT_LEGEND_TILEMAP_BYTES_2[..],
            3 => &ATTRACT_LEGEND_TILEMAP_BYTES_3[..],
            _ => &[],
        };
        let len = data.len().min(self.vram_upload_buffer_remaining_len());
        self.copy_vram_upload_buffer_bytes(0, &data[..len]);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn attract_build_backgrounds(&mut self) {
        self.set_bg_mode(9);
        self.set_main_screen_layers(0x17);
        self.set_sub_screen_layers(0);
        self.zelda_ppu_write(0x2107, 0x10);
        self.zelda_ppu_write(0x2108, 0x00);

        let mut k = 0usize;
        let mut p = 0usize;
        loop {
            let mut j = k & 3;
            loop {
                self.write_vram_upload_buffer_word(
                    4 + k * 2,
                    ATTRACT_BACKGROUND_TILE_PATTERN[p + j],
                );
                k += 1;
                j += 1;
                if j & 3 == 0 {
                    break;
                }
            }
            if k & 0x1f == 0 {
                p += 4;
            }
            if k == 0x80 {
                break;
            }
        }
        self.attract_trigger_bgdma(0x1000);

        k = 0;
        loop {
            let mut j = k & 1;
            let p = (k & 0x20) >> 4;
            loop {
                self.write_vram_upload_buffer_word(
                    4 + k * 2,
                    ATTRACT_BACKGROUND_CORNER_TILES[p + j],
                );
                k += 1;
                j += 1;
                if j & 1 == 0 {
                    break;
                }
            }
            if k == 0x80 {
                break;
            }
        }
        self.attract_trigger_bgdma(0);
        self.clear_attract_vram_destination_address();
    }

    pub(super) fn attract_trigger_bgdma(&mut self, dstv: usize) {
        for row in 0..8 {
            let dst = dstv + row * 0x80;
            for i in 0..0x80 {
                self.ppu.vram[dst + i] = self.vram_upload_buffer_word(4 + i * 2);
            }
        }
    }
}
