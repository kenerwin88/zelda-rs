// Methods ported from zelda3/src/attract.c and included inside ZeldaState.

use super::*;

const ATTRACT_LOW_RAM_CLEAR_START: usize = 0x20;
const ATTRACT_LOW_RAM_CLEAR_LEN: usize = 0x51;

pub(super) const SOLDIER_DRAW1_CHAR: [u8; 4] = [0x42, 0x42, 0x40, 0x44];
pub(super) const SOLDIER_DRAW1_FLAGS: [u8; 4] = [0x40, 0, 0, 0];
pub(super) const SOLDIER_DRAW1_YD: [i8; 26] = [
    7, 8, 7, 8, 8, 7, 8, 7, 8, 7, 8, 8, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
];
pub(super) const SOLDIER_DRAW2_XD: [i8; 104] = [
    -4, 4, 10, 10, -4, 4, 10, 10, -4, 4, 10, 10, -4, 4, 10, 10, -4, -4, 0, 0, -4, -4, 0, 0, -3, -3,
    0, 0, -3, -3, -4, 4, -3, -3, -4, 4, -3, -3, -4, 4, -3, -3, -4, 4, 12, 12, 0, 0, 12, 12, 0, 0,
    11, 11, 0, 0, -4, 4, 0, 0, -4, 4, 0, 0, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -4, 4,
    0, 0, -4, 4, 0, 0, -4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub(super) const SOLDIER_DRAW2_YD: [i8; 104] = [
    0, 0, 2, 10, 0, 0, 2, 10, 0, 0, 1, 9, 0, 0, 2, 10, -2, 6, 1, 1, -2, 6, 2, 2, -2, 6, 1, 1, -5,
    3, 0, 0, -4, 4, 0, 0, -4, 4, 0, 0, -5, 3, 0, 0, -2, 6, 1, 1, -2, 6, 2, 2, -2, 6, 1, 1, 0, 0, 8,
    8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8,
    8, 0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8,
];
pub(super) const SOLDIER_DRAW2_CHAR: [u8; 104] = [
    0x48, 0x49, 0x6d, 0x7d, 0x49, 0x48, 0x6d, 0x7d, 0x46, 0x46, 0x6d, 0x7d, 0x4b, 0x46, 0x6d, 0x7d,
    0x4d, 0x5d, 0x4e, 0x4e, 0x4d, 0x5d, 0x60, 0x60, 0x4d, 0x5d, 0x62, 0x62, 0x6d, 0x7d, 0x64, 0x64,
    0x6d, 0x7d, 0x66, 0x67, 0x6d, 0x7d, 0x67, 0x66, 0x6d, 0x7d, 0x64, 0x69, 0x4d, 0x5d, 0x4e, 0x4e,
    0x4d, 0x5d, 0x60, 0x60, 0x4d, 0x5d, 0x62, 0x62, 2, 3, 0x20, 0x20, 2, 0x0c, 0x20, 0x20, 2, 0x0c,
    0x20, 0x20, 8, 8, 0x20, 0x20, 0x0e, 0x0e, 0x20, 0x20, 0x0e, 0x0e, 0x20, 0x20, 5, 6, 0x20, 0x20,
    0x22, 6, 0x20, 0x20, 0x22, 6, 0x20, 0x20, 8, 8, 0x20, 0x20, 0x0e, 0x0e, 0x20, 0x20, 0x0e, 0x0e,
    0x20, 0x20,
];
pub(super) const SOLDIER_DRAW2_FLAGS: [u8; 104] = [
    0, 0, 0, 0, 0x40, 0x40, 0, 0, 0, 0x40, 0, 0, 0, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x40, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40,
];
pub(super) const SOLDIER_DRAW2_BIG: [u8; 104] = [
    2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2,
    0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2,
];
pub(super) const SOLDIER_DRAW2_OAM_IDX: [u8; 4] = [12, 12, 12, 4];
type AttractOamInfo = (i8, i8, u8, u8, u8);
pub(super) const SOLDIER_DRAW3_XD: [i8; 28] = [
    -3, -3, -4, -4, -4, -4, -4, -4, -11, -3, -11, -3, -16, -8, 12, 12, 12, 12, 12, 12, 12, 12, 21,
    13, 21, 13, 24, 16,
];
pub(super) const SOLDIER_DRAW3_YD: [i8; 28] = [
    11, 19, 11, 19, 10, 18, 14, 22, 8, 8, 8, 8, 6, 6, -10, -2, -9, -1, -9, -1, -16, -8, 8, 8, 8, 8,
    6, 6,
];
pub(super) const SOLDIER_DRAW3_CHAR: [u8; 28] = [
    0x7b, 0x6b, 0x7b, 0x6b, 0x7b, 0x6b, 0x7b, 0x6b, 0x6c, 0x7c, 0x6c, 0x7c, 0x6c, 0x7c, 0x6b, 0x7b,
    0x6b, 0x7b, 0x6b, 0x7b, 0x6b, 0x7b, 0x6c, 0x7c, 0x6c, 0x7c, 0x6c, 0x7c,
];
pub(super) const SOLDIER_DRAW3_FLAGS: [u8; 28] = [
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40,
];
pub(super) const SOLDIER_DRAW3_OAM_IDX: [u8; 4] = [4, 4, 4, 20];
pub(super) const SOLDIER_DRAW_SHADOW: [u8; 4] = [0x0c, 0x0c, 0x0a, 0x0a];

impl ZeldaState {
    pub(super) fn module14_attract(&mut self) {
        let mut state = self.attract_state_view().state();
        if self.display_state().screen_brightness != 0
            && self.display_state().screen_brightness != 128
            && state != 0
            && state != 2
            && state != 6
            && self.player_state_view().filtered_joypad_h() & 0x90 != 0
        {
            self.attract_state_view_mut().set_state(9);
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
        self.attract_state_view_mut().clear_intro_did_run_step();
        self.deactivate_nmi_thread();
        self.intro_periodic_sword_and_intro_flash();
        if self.display_state().screen_brightness != 0 {
            self.decrement_screen_brightness();
            return;
        }

        self.enable_force_blank();
        self.set_irq_control_flag(0xff);
        self.deactivate_nmi_thread();
        self.clear_pending_polyhedral_update();
        self.attract_state_view_mut().increment_state();
    }

    pub(super) fn attract_init_graphics(&mut self) {
        self.fill_ram(ATTRACT_LOW_RAM_CLEAR_START, ATTRACT_LOW_RAM_CLEAR_LEN, 0);
        self.erase_tile_maps_normal();
        self.Attract_LoadBG3GFX();
        self.palette_buffer_view_mut().set_overworld_palette_mode(4);
        self.palette_buffer_view_mut().set_hud_palette(1);
        self.palette_buffer_view_mut()
            .clear_overworld_aux_or_main_offset();
        self.palette_load_hud();
        self.palette_buffer_view_mut()
            .select_overworld_aux_palette_offset();
        self.palette_load_ow_bg_main();
        self.palette_load_hud();
        self.palette_load_link_armor_and_gloves();
        self.palette_buffer_view_mut().set_main_color(0x1d, 0x3800);
        self.system_signals_view_mut().increment_cgram_update_flag();
        self.ppu_scroll_copy_view_mut().set_bg3_v_copy2_low(20);
        self.attract_build_backgrounds();
        self.messaging_state_view_mut().clear_module();
        self.dialogue_message_index_view_mut().set_value(0x112);
        self.world_state_view_mut().set_bg2_y(0);
        self.attract_state_view_mut().set_legend_ctr(0x1010);
        self.attract_state_view_mut().add_state(3);

        self.hdma_setup(0x0cfa87, 0x0cfa94, 1, 0x26, 0x28, 0);
        self.set_hdma_enable_mask(0xc0);
        self.set_bg12_window_selection(0);
        self.set_bg34_window_selection(0);
        self.set_object_color_window_selection(0xb0);
        self.set_main_screen_window_layers(3);
        self.set_sub_screen_window_layers(0);
        self.palette_filter_view_mut().set_fixed_color_red(0x25);
        self.palette_filter_view_mut().set_fixed_color_green(0x45);
        self.palette_filter_view_mut().set_fixed_color_blue(0x85);
        self.palette_filter_view_mut()
            .set_color_window_selection(0x10);
        self.palette_filter_view_mut().set_color_math_control(0xa3);
        self.system_signals_view_mut().set_music_control(6);
        self.attract_state_view_mut().increment_legend_flag();
    }

    pub(super) fn attract_load_new_scene(&mut self) {
        match self.attract_state_view().sequence() {
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
        self.attract_state_view_mut().clear_next_legend_gfx();
        self.attract_state_view_mut().increment_state();
        self.set_screen_brightness(0);
    }

    pub(super) fn attract_scene_world_map(&mut self) {
        self.zelda_ppu_write(0x2107, 0x13);
        self.zelda_ppu_write(0x2108, 0x03);
        self.palette_filter_view_mut()
            .set_color_window_selection(0x80);
        self.palette_filter_view_mut().set_color_math_control(0x21);
        self.set_bg_mode(7);
        self.world_map_load_light_world_map();
        self.ppu_scroll_copy_view_mut().set_mode7_center_y(0x00ed);
        self.ppu_scroll_copy_view_mut().set_mode7_center_x(0x0100);
        self.ppu_scroll_copy_view_mut().set_bg1_h_copy(0x0080);
        self.ppu_scroll_copy_view_mut().set_bg1_v_copy(0x00c0);
        self.attract_state_view_mut().set_mode7_zoom_timer(0xff);
        self.attract_control_map_zoom();
        self.attract_state_view_mut().set_scene_timer(1);
        self.attract_state_view_mut().increment_state();
        self.set_screen_brightness(0);
    }

    pub(super) fn attract_scene_throne_room(&mut self) {
        self.clear_hdma_enable_mask();
        self.palette_filter_view_mut().set_color_window_selection(2);
        self.palette_filter_view_mut().set_color_math_control(0x20);
        self.sprite_system_view_mut()
            .set_misc_sprites_graphics_index(10);
        self.load_common_sprites();

        let attract_bg2_vofs_backup = self.attract_state_view().bg2_vofs_backup();
        let attract_state = self.attract_state_view().state_word();
        self.dungeon_load_and_draw_entrance_room(0x74);
        self.attract_state_view_mut().set_state_word(attract_state);
        self.attract_state_view_mut()
            .set_bg2_vofs_backup(attract_bg2_vofs_backup);

        self.palette_buffer_view_mut().set_palette_main_indoors(0);
        self.palette_buffer_view_mut().set_sp0l(0);
        self.palette_buffer_view_mut().set_sp5l(14);
        self.palette_buffer_view_mut().set_sp6l(3);
        self.dungeon_save_and_load_load_all_palettes(0, 0x7e);

        self.palette_buffer_view_mut().set_main_color(0x1d, 0x3800);
        self.messaging_state_view_mut().clear_module();
        self.dialogue_message_index_view_mut().set_value(0x113);
        self.attract_state_view_mut().set_scene_timer(2);
        self.attract_state_view_mut().set_throne_fade_timer(0xe0);
        self.oam_state_view_mut().set_priority_word(0x210);

        self.attract_prep_finish();
    }

    pub(super) fn attract_prep_zelda_prison(&mut self) {
        self.palette_filter_view_mut().set_color_window_selection(0);
        self.palette_filter_view_mut().set_color_math_control(0);

        let attract_bg2_vofs_backup = self.attract_state_view().bg2_vofs_backup();
        let attract_state = self.attract_state_view().state_word();
        self.dungeon_load_and_draw_entrance_room(0x73);
        self.attract_state_view_mut().set_state_word(attract_state);
        self.attract_state_view_mut()
            .set_bg2_vofs_backup(attract_bg2_vofs_backup);

        self.palette_buffer_view_mut().set_palette_main_indoors(2);
        self.palette_buffer_view_mut().set_sp0l(0);
        self.palette_buffer_view_mut().set_sp5l(14);
        self.palette_buffer_view_mut().set_sp6l(3);
        self.dungeon_save_and_load_load_all_palettes(1, 0x7f);
        self.palette_buffer_view_mut().set_main_color(0x1d, 0x3800);

        self.messaging_state_view_mut().clear_module();
        self.dialogue_message_index_view_mut().set_value(0x114);

        self.attract_state_view_mut().set_prison_zelda_y_base(148);
        {
            let mut target = self.attract_vram_target_view_mut();
            target.set_low(0x68);
            target.clear_high();
        }
        self.attract_state_view_mut().set_anim_step_counter(0);
        self.attract_state_view_mut().set_soldier_anim_step(0);
        self.attract_state_view_mut().set_x_base_high(0);
        self.attract_state_view_mut().set_scene_frame_counter(0);
        self.attract_state_view_mut().clear_fade_in_done_flag();
        self.attract_state_view_mut().set_scene_timer(255);
        self.oam_state_view_mut().set_priority_word(0x240);
        self.attract_prep_finish();
    }

    pub(super) fn attract_prep_maiden_warp(&mut self) {
        let attract_bg2_vofs_backup = self.attract_state_view().bg2_vofs_backup();
        let attract_state = self.attract_state_view().state_word();
        self.dungeon_load_and_draw_entrance_room(0x75);
        self.attract_state_view_mut().set_state_word(attract_state);
        self.attract_state_view_mut()
            .set_bg2_vofs_backup(attract_bg2_vofs_backup);

        self.palette_buffer_view_mut().set_palette_main_indoors(0);
        self.palette_buffer_view_mut().set_sp0l(0);
        self.palette_buffer_view_mut().set_sp5l(14);
        self.palette_buffer_view_mut().set_sp6l(3);
        self.palette_buffer_view_mut()
            .clear_overworld_aux_or_main_offset();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sprite_environment_dungeon();
        self.palette_load_hud();
        self.palette_load_dungeon_set();
        self.dungeon_save_and_load_load_all_palettes(2, 0x7f);
        self.palette_buffer_view_mut().set_aux_color(0x1d, 0x3800);
        self.palette_buffer_view_mut().set_main_color(0x1d, 0x3800);

        self.messaging_state_view_mut().clear_module();
        self.dialogue_message_index_view_mut().set_value(0x115);
        self.attract_state_view_mut().set_scene_timer(255);
        self.attract_state_view_mut().set_vram_dst_byte(112);
        self.attract_state_view_mut().set_maiden_warp_timer_a(112);
        self.attract_state_view_mut().set_maiden_warp_timer_b(112);
        self.attract_state_view_mut().set_anim_step_counter(8);
        self.attract_state_view_mut().set_scene_frame_counter(0);
        self.attract_state_view_mut().set_maiden_warp_step(0);
        self.attract_state_view_mut().set_fade_in_complete_flag(0);
        self.attract_state_view_mut().clear_fade_in_done_flag();
        self.attract_state_view_mut().set_scene_substep(0);
        self.attract_state_view_mut().clear_substep_delay_counter();
        self.oam_state_view_mut().set_priority_word(0x00c0);
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
        self.display_nmi_view_mut()
            .increment_core_update_disable_flag();
        self.intro_initialize_memory_darken();
        self.overworld_load_all_palettes();
        self.ppu_scroll_copy_view_mut().set_bg3_v_copy2_low(0);
        self.ppu_scroll_copy_view_mut().set_mode7_center_y(0);
        self.ppu_scroll_copy_view_mut().set_mode7_center_x(0);
        self.ppu_scroll_copy_view_mut().set_bg1_h_copy(0);
        self.ppu_scroll_copy_view_mut().set_bg1_v_copy(0);
        self.ppu_scroll_copy_view_mut().set_bg2_h_copy(0);
        self.ppu_scroll_copy_view_mut().set_bg2_v_copy(0);
        self.system_signals_view_mut().set_music_control(0xf1);
        self.attract_state_view_mut().set_sequence(0);
        self.frame_control_view_mut().set_main_module(0);
        self.frame_control_view_mut().set_submodule(10);
        self.frame_control_view_mut().set_subsubmodule(10);
    }

    pub(super) fn attract_skip_to_file_select(&mut self) {
        self.decrement_screen_brightness();
        if self.display_state().screen_brightness != 0 {
            return;
        }
        self.enable_force_blank();
        self.zelda_ppu_write(0x2107, 0x13);
        self.zelda_ppu_write(0x2108, 0x03);
        self.attract_setup_conclusion_hdma();
        self.ppu_scroll_copy_view_mut().set_mode7_center_y(0);
        self.ppu_scroll_copy_view_mut().set_mode7_center_x(0);
        self.ppu_scroll_copy_view_mut().set_bg1_h_copy(0);
        self.ppu_scroll_copy_view_mut().set_bg1_v_copy(0);
        self.ppu_scroll_copy_view_mut().set_bg3_v_copy2_low(0);
        self.fade_music_and_reset_sram_mirror();
    }

    pub(super) fn attract_prep_finish(&mut self) {
        self.attract_state_view_mut().increment_state();
        self.set_screen_brightness(0);
        self.ppu_scroll_copy_view_mut().set_bg3_v_copy2_low(0);
        let bg2_hofs = self.ppu_scroll_copy_view().bg2_h_copy() & 0x01ff;
        let bg2_vofs = self.ppu_scroll_copy_view().bg2_v_copy() & 0x01ff;
        let bg2_hofs2 = self.world_state_view().bg2_x() & 0x01ff;
        let bg2_vofs2 = self.world_state_view().bg2_y() & 0x01ff;
        self.ppu_scroll_copy_view_mut().set_bg2_h_copy(bg2_hofs);
        self.ppu_scroll_copy_view_mut().set_bg2_v_copy(bg2_vofs);
        self.world_state_view_mut().set_bg2_x(bg2_hofs2);
        self.world_state_view_mut().set_bg2_y(bg2_vofs2);
    }

    pub(super) fn attract_control_map_zoom(&mut self) {
        const MAP_MODE_ZOOMS1: [u16; 240] = [
            375, 374, 373, 373, 372, 371, 371, 370, 369, 369, 368, 367, 367, 366, 365, 365, 364,
            363, 363, 361, 361, 360, 359, 359, 358, 357, 357, 356, 355, 355, 354, 354, 353, 352,
            352, 351, 351, 350, 349, 349, 348, 348, 347, 346, 346, 345, 345, 344, 343, 343, 342,
            342, 341, 341, 340, 339, 339, 338, 338, 337, 337, 336, 335, 335, 334, 334, 333, 333,
            332, 332, 331, 331, 330, 330, 328, 327, 327, 326, 326, 325, 325, 324, 324, 323, 323,
            322, 322, 321, 321, 320, 320, 319, 319, 318, 318, 317, 317, 316, 316, 315, 315, 314,
            314, 313, 313, 312, 312, 311, 311, 310, 310, 309, 309, 309, 308, 308, 307, 307, 306,
            306, 305, 305, 304, 304, 303, 303, 303, 302, 302, 301, 301, 300, 300, 299, 299, 299,
            298, 298, 297, 297, 295, 295, 294, 294, 294, 293, 293, 292, 292, 292, 291, 291, 290,
            290, 289, 289, 289, 288, 288, 287, 287, 287, 286, 286, 285, 285, 285, 284, 284, 283,
            283, 283, 282, 282, 281, 281, 281, 280, 280, 279, 279, 279, 278, 278, 278, 277, 277,
            276, 276, 276, 275, 275, 275, 274, 274, 273, 273, 273, 272, 272, 272, 271, 271, 271,
            270, 270, 269, 269, 269, 268, 268, 268, 267, 267, 267, 266, 266, 266, 265, 265, 265,
            264, 264, 264, 263, 263, 262, 262, 262, 261, 261, 261, 260, 260, 260, 259, 259, 259,
            258, 258,
        ];
        let zoom = self.attract_state_view().mode7_zoom_timer() as u16;
        for (i, value) in MAP_MODE_ZOOMS1.iter().enumerate() {
            self.spotlight_hdma_view_mut()
                .set_hdma_table_dynamic_entry(i, ((*value as u32 * zoom as u32) >> 8) as u16);
        }
    }

    pub(super) fn attract_fade_in_sequence(&mut self) {
        if self.display_state().screen_brightness != 15 {
            if (self.player_state_view_mut().decrement_speed_setting() as i8) < 0 {
                self.increment_screen_brightness();
                self.player_state_view_mut().set_speed_setting(1);
            }
        } else {
            self.attract_state_view_mut().increment_state();
        }
    }

    pub(super) fn attract_fade_out_sequence(&mut self) {
        if self.display_state().screen_brightness != 0 {
            if (self.player_state_view_mut().decrement_speed_setting() as i8) < 0 {
                self.decrement_screen_brightness();
                self.player_state_view_mut().set_speed_setting(1);
            }
        } else {
            self.enable_force_blank();
            self.erase_tile_maps_normal();
            self.attract_state_view_mut().increment_state();
        }
    }

    pub(super) fn attract_enact_story(&mut self) {
        match self.attract_state_view().sequence() {
            0 => self.attract_dramatize_polka_dots(),
            1 => self.attract_dramatize_world_map(),
            2 => self.attract_throne_room(),
            3 => self.attract_dramatize_prison(),
            4 => self.attract_dramatize_agahnim_altar(),
            _ => {}
        }
    }

    pub(super) fn attract_dramatize_world_map(&mut self) {
        if self.attract_state_view().mode7_zoom_timer() != 0 {
            if self.attract_state_view().mode7_zoom_timer() < 15 {
                self.decrement_screen_brightness();
            }
            self.attract_state_view_mut().decrement_scene_timer();
            if self.attract_state_view().scene_timer() == 0 {
                self.attract_state_view_mut().set_scene_timer(1);
                self.attract_state_view_mut().decrement_mode7_zoom_timer();
                self.attract_control_map_zoom();
            }
        } else {
            self.enable_force_blank();
            self.set_bg_mode(9);
            self.erase_tile_maps_normal();
            self.attract_state_view_mut().increment_sequence();
            self.attract_state_view_mut().subtract_state(2);
        }
    }

    pub(super) fn attract_throne_room(&mut self) {
        const THRONE_ROOM_OAMS: [(i8, i8, u8, u8, u8); 10] = [
            (16, 16, 0x2a, 0x7b, 2),
            (0, 16, 0x2a, 0x3b, 2),
            (16, 0, 0x0a, 0x7b, 2),
            (0, 0, 0x0a, 0x3b, 2),
            (0, 0, 0x0c, 0x31, 2),
            (16, 0, 0x0e, 0x31, 2),
            (32, 0, 0x0c, 0x71, 2),
            (0, 16, 0x2c, 0x31, 2),
            (16, 16, 0x2e, 0x31, 2),
            (32, 16, 0x2c, 0x71, 2),
        ];
        const OAM_OFFS: [usize; 3] = [0, 4, 10];
        const X_BASE: [u8; 2] = [80, 104];
        const Y_BASE: [i16; 2] = [88, 32];

        self.attract_state_view_mut().set_oam_index(0);
        if self.attract_state_view().fade_in_complete_flag() == 0 {
            if self.display_state().screen_brightness != 15 {
                self.increment_screen_brightness();
            } else {
                self.attract_state_view_mut()
                    .increment_fade_in_complete_flag();
            }
        }

        if self.ppu_scroll_copy_view().bg2_v_copy() == 0 {
            self.attract_show_timed_text_message();
            if self.oam_state_view().priority_word() == 0 {
                if self.attract_state_view().throne_fade_timer() < 31
                    && self.attract_state_view().throne_fade_timer() & 1 == 0
                {
                    self.decrement_screen_brightness();
                }
                let throne_fade = self.attract_state_view_mut().decrement_throne_fade_timer();
                if throne_fade == 0 {
                    self.attract_state_view_mut().increment_sequence();
                    self.attract_state_view_mut().increment_state();
                    return;
                }
            }
        } else {
            let bg2 = self.ppu_scroll_copy_view().bg2_v_copy().wrapping_sub(1);
            let bg1 = self.ppu_scroll_copy_view().bg1_v_copy().wrapping_sub(1);
            self.ppu_scroll_copy_view_mut().set_bg2_v_copy(bg2);
            self.ppu_scroll_copy_view_mut().set_bg1_v_copy(bg1);
        }

        let bg2_vofs = self.ppu_scroll_copy_view().bg2_v_copy();
        for i in (0..=1).rev() {
            let y = (Y_BASE[i] as u16).wrapping_sub(bg2_vofs);
            if (y.wrapping_add(32) as i16) >= 0 {
                self.attract_state_view_mut().set_x_base(X_BASE[i]);
                self.attract_state_view_mut().set_y_base(y as u8);
                self.attract_draw_sprite_set2_slice(
                    &THRONE_ROOM_OAMS[OAM_OFFS[i]..OAM_OFFS[i + 1]],
                );
            }
        }

        self.attract_state_view_mut().set_story_text_pointer(0xf8a7);
    }

    pub(super) fn attract_show_timed_text_message(&mut self) {
        let bg2_vofs2 = self.world_state_view().bg2_y();
        self.attract_state_view_mut().set_bg2_vofs_backup(bg2_vofs2);
        self.player_state_view_mut().set_joypad1l_last(0);
        self.player_state_view_mut().set_filtered_joypad_l(0);
        self.player_state_view_mut().set_filtered_joypad_h(0);
        self.RenderText();
        let priority = self.oam_state_view().priority_word();
        if priority != 0 {
            self.oam_state_view_mut()
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
        let start = self.attract_state_view().oam_index() as usize;
        self.attract_state_view_mut()
            .advance_oam_index_by(entries.len() as u8);
        for (slot, &(x, y, charnum, flags, big)) in entries.iter().rev().enumerate() {
            let obj_x = self.attract_state_view().x_base().wrapping_add_signed(x);
            let obj_y = self.attract_state_view().y_base().wrapping_add_signed(y);
            self.set_oam_plain(64 + start + slot, obj_x, obj_y, charnum, flags, big);
        }
    }

    pub(super) fn attract_dramatize_agahnim_altar(&mut self) {
        const SOLDIER_X: [u16; 6] = [48, 192, 48, 192, 80, 160];
        const SOLDIER_Y: [u16; 6] = [112, 112, 152, 152, 192, 192];
        const SOLDIER_DIR: [u8; 6] = [0, 1, 0, 1, 3, 3];
        const SOLDIER_FLAGS: [u8; 6] = [9, 9, 9, 9, 7, 9];
        const MAIDEN_WARP0: [(i8, i8, u8, u8, u8); 4] = [
            (0, 0, 0x03, 0x3d, 2),
            (8, 0, 0x04, 0x3d, 2),
            (0, 0, 0x00, 0x3d, 2),
            (8, 0, 0x01, 0x3d, 2),
        ];
        const MAIDEN_WARP_XBASE: [u8; 8] = [4, 4, 3, 3, 2, 2, 1, 0];
        const MAIDEN_WARP1: [(i8, i8, u8, u8, u8); 16] = [
            (0, 0, 0x6c, 0x38, 2),
            (0, 0, 0x6c, 0x38, 2),
            (0, 0, 0x6c, 0x38, 2),
            (0, 0, 0x6c, 0x38, 2),
            (0, 0, 0x6c, 0x38, 2),
            (2, 0, 0x6c, 0x38, 2),
            (0, 0, 0x6c, 0x38, 2),
            (2, 0, 0x6c, 0x38, 2),
            (0, 0, 0x6c, 0x38, 2),
            (4, 0, 0x6c, 0x38, 2),
            (0, 0, 0x6c, 0x38, 2),
            (4, 0, 0x6c, 0x38, 2),
            (0, 0, 0x6c, 0x38, 2),
            (6, 0, 0x6c, 0x38, 2),
            (0, 0, 0x6c, 0x38, 2),
            (8, 0, 0x6c, 0x38, 2),
        ];
        const MAIDEN_WARP2: [(i8, i8, u8, u8, u8); 48] = [
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x82, 0x3b, 2),
            (16, 0, 0x82, 0x7b, 2),
            (0, 16, 0xa2, 0x3b, 2),
            (16, 16, 0xa2, 0x7b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x80, 0x3b, 2),
            (16, 0, 0x82, 0x7b, 2),
            (0, 16, 0xa0, 0x3b, 2),
            (16, 16, 0xa2, 0x7b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x82, 0x3b, 2),
            (16, 0, 0x82, 0x7b, 2),
            (0, 16, 0xa2, 0x3b, 2),
            (16, 16, 0xa2, 0x7b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x82, 0x3b, 2),
            (16, 0, 0x80, 0x7b, 2),
            (0, 16, 0xa2, 0x3b, 2),
            (16, 16, 0xa0, 0x7b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x82, 0x3b, 2),
            (16, 0, 0x82, 0x7b, 2),
            (0, 16, 0xa2, 0x3b, 2),
            (16, 16, 0xa2, 0x7b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x80, 0x3b, 2),
            (16, 0, 0x82, 0x7b, 2),
            (0, 16, 0xa0, 0x3b, 2),
            (16, 16, 0xa2, 0x7b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x82, 0x3b, 2),
            (16, 0, 0x82, 0x7b, 2),
            (0, 16, 0xa2, 0x3b, 2),
            (16, 16, 0xa2, 0x7b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x80, 0x3b, 2),
            (16, 0, 0x80, 0x7b, 2),
            (0, 16, 0xa0, 0x3b, 2),
            (16, 16, 0xa0, 0x7b, 2),
        ];

        if self.attract_state_view().scene_done_flag() != 0 {
            self.attract_state_view_mut().increment_sequence();
            self.attract_state_view_mut().subtract_state(2);
            return;
        }

        self.attract_state_view_mut().set_oam_index(0);
        self.handle_screen_flash();
        if self.attract_state_view().fade_in_done_flag() == 0 {
            self.attract_fade_in_step();
        }
        if self.attract_state_view().scene_frame_counter() != 0xff {
            self.attract_state_view_mut()
                .increment_scene_frame_counter();
        }
        if self.attract_state_view().intro_palette_flash_count() & 4 != 0 {
            self.system_signals_view_mut().set_sound_effect_2(0x2b);
        }

        match self.attract_state_view().scene_substep() {
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
                SOLDIER_X[k],
                SOLDIER_Y[k],
                SOLDIER_DIR[k],
                SOLDIER_FLAGS[k],
                0,
            );
        }

        if self.attract_state_view().scene_frame_counter() >= 0xa0 {
            if self.attract_state_view().vram_dst_byte() != 0x60 {
                let step = self.attract_state_view_mut().decrement_anim_step_counter();
                if step == 0 {
                    self.attract_state_view_mut().decrement_vram_dst_byte();
                    self.attract_state_view_mut().set_anim_step_counter(8);
                }
            } else {
                self.attract_state_view_mut()
                    .increment_substep_delay_counter();
            }
        }

        if self.attract_state_view().fade_in_complete_flag() == 0 {
            self.attract_state_view_mut().set_x_base(116);
            let y_base = self.attract_state_view().vram_dst_byte();
            self.attract_state_view_mut().set_y_base(y_base);
            let start = if self.attract_state_view().vram_dst_byte() == 0x70 {
                0
            } else {
                2
            };
            self.attract_draw_sprite_set2_slice(&MAIDEN_WARP0[start..start + 2]);

            let mut k = 7usize;
            if self.attract_state_view().vram_dst_byte() < 0x68 {
                k = self.attract_state_view().vram_dst_byte().wrapping_sub(0x68) as usize & 7;
            }
            self.attract_state_view_mut()
                .set_x_base(0x74 + MAIDEN_WARP_XBASE[k]);
            self.attract_state_view_mut().set_y_base(0x76);
            self.attract_draw_sprite_set2_slice(&MAIDEN_WARP1[k * 2..k * 2 + 2]);
        }

        let k = (self.attract_state_view().scene_frame_counter() >> 5) as usize & 7;
        self.attract_state_view_mut().set_x_base(112);
        self.attract_state_view_mut().set_y_base(70);
        self.attract_draw_sprite_set2_slice(&MAIDEN_WARP2[k * 6..k * 6 + 6]);
    }

    pub(super) fn attract_maiden_warp_case0(&mut self) {
        if self.attract_state_view().substep_delay_counter() != 0 {
            self.attract_state_view_mut().increment_scene_substep();
        }
    }

    pub(super) fn attract_maiden_warp_case1(&mut self) {
        const OAMS: [(i8, i8, u8, u8, u8); 28] = [
            (0, 0, 0xce, 0x35, 0),
            (28, 0, 0xce, 0x35, 0),
            (-2, 3, 0x26, 0x75, 0),
            (30, 3, 0x26, 0x35, 0),
            (-2, 11, 0x36, 0x75, 0),
            (30, 11, 0x36, 0x35, 0),
            (0, 16, 0x26, 0x75, 0),
            (28, 16, 0x26, 0x35, 0),
            (0, 24, 0x36, 0x75, 0),
            (28, 24, 0x36, 0x35, 0),
            (2, 16, 0x20, 0x35, 2),
            (18, 16, 0x20, 0x75, 2),
            (2, 32, 0x20, 0xb5, 2),
            (18, 32, 0x20, 0xf5, 2),
            (0, 0, 0xce, 0x37, 0),
            (28, 0, 0xce, 0x37, 0),
            (-2, 3, 0x26, 0x77, 0),
            (30, 3, 0x26, 0x37, 0),
            (-2, 11, 0x36, 0x77, 0),
            (30, 11, 0x36, 0x37, 0),
            (0, 16, 0x26, 0x77, 0),
            (28, 16, 0x26, 0x37, 0),
            (0, 24, 0x36, 0x77, 0),
            (28, 24, 0x36, 0x37, 0),
            (2, 16, 0x22, 0x37, 2),
            (18, 16, 0x22, 0x77, 2),
            (2, 32, 0x22, 0xb7, 2),
            (18, 32, 0x22, 0xf7, 2),
        ];
        const NUM: [usize; 8] = [2, 2, 2, 6, 6, 10, 10, 14];

        let k = (self.frame_state().frame_counter >> 2) as usize & 1;
        let n = NUM[(self.attract_state_view().maiden_warp_step() >> 1) as usize & 7];
        self.attract_state_view_mut().set_x_base(110);
        self.attract_state_view_mut().set_y_base(72);
        self.attract_draw_sprite_set2_slice(&OAMS[k * 14..k * 14 + n]);

        if self.attract_state_view().maiden_warp_step() == 0
            && self.attract_state_view().maiden_warp_timer_b() == 0x70
        {
            self.system_signals_view_mut().set_sound_effect_2(0x27);
        }

        if self.attract_state_view().maiden_warp_step() == 15 {
            self.attract_state_view_mut().increment_scene_substep();
        } else {
            if self.attract_state_view().maiden_warp_step() == 6 {
                self.attract_state_view_mut()
                    .set_intro_palette_flash_count(0x90);
                self.system_signals_view_mut().set_sound_effect_2(0x2b);
            }
            if self.attract_state_view().maiden_warp_timer_b() != 0 {
                self.attract_state_view_mut()
                    .decrement_maiden_warp_timer_b();
            } else {
                self.attract_state_view_mut().increment_maiden_warp_step();
            }
        }
    }

    pub(super) fn attract_maiden_warp_case2(&mut self) {
        const NUM: [usize; 8] = [4, 4, 8, 8, 12, 12, 14, 14];
        const OAMS: [(i8, i8, u8, u8, u8); 28] = [
            (0, 0, 0xce, 0x35, 0),
            (28, 0, 0xce, 0x35, 0),
            (-2, 3, 0x26, 0x75, 0),
            (30, 3, 0x26, 0x35, 0),
            (-2, 11, 0x36, 0x75, 0),
            (30, 11, 0x36, 0x35, 0),
            (0, 16, 0x26, 0x75, 0),
            (28, 16, 0x26, 0x35, 0),
            (0, 24, 0x36, 0x75, 0),
            (28, 24, 0x36, 0x35, 0),
            (2, 16, 0x20, 0x35, 2),
            (18, 16, 0x20, 0x75, 2),
            (2, 32, 0x20, 0xb5, 2),
            (18, 32, 0x20, 0xf5, 2),
            (0, 0, 0xce, 0x37, 0),
            (28, 0, 0xce, 0x37, 0),
            (-2, 3, 0x26, 0x77, 0),
            (30, 3, 0x26, 0x37, 0),
            (-2, 11, 0x36, 0x77, 0),
            (30, 11, 0x36, 0x37, 0),
            (0, 16, 0x26, 0x77, 0),
            (28, 16, 0x26, 0x37, 0),
            (0, 24, 0x36, 0x77, 0),
            (28, 24, 0x36, 0x37, 0),
            (2, 16, 0x22, 0x37, 2),
            (18, 16, 0x22, 0x77, 2),
            (2, 32, 0x22, 0xb7, 2),
            (18, 32, 0x22, 0xf7, 2),
        ];

        self.attract_state_view_mut().set_x_base(110);
        self.attract_state_view_mut().set_y_base(72);
        let k = (self.frame_state().frame_counter >> 2) as usize & 1;
        let n = NUM[(self.attract_state_view().maiden_warp_step() >> 1) as usize & 7];
        self.attract_draw_sprite_set2_slice(&OAMS[k * 14 + (14 - n)..k * 14 + 14]);

        if self.attract_state_view().maiden_warp_step() == 0 {
            let timer = self
                .attract_state_view_mut()
                .decrement_maiden_warp_timer_a();
            if timer == 0 {
                self.attract_state_view_mut().increment_scene_substep();
            }
        } else {
            self.attract_state_view_mut().decrement_maiden_warp_step();
        }
    }

    pub(super) fn attract_maiden_warp_case3(&mut self) {
        const OAMS: [(i8, i8, u8, u8, u8); 3] = [
            (0, 0, 0xc6, 0x3d, 2),
            (0, 0, 0x24, 0x35, 2),
            (16, 0, 0x24, 0x75, 2),
        ];
        const X_BASE: [u8; 2] = [0x78, 0x70];

        if self.attract_state_view().maiden_warp_step() == 6 {
            self.attract_state_view_mut()
                .increment_fade_in_complete_flag();
            self.system_signals_view_mut().set_sound_effect_1(51);
        } else if self.attract_state_view().maiden_warp_step() == 0x40 {
            self.attract_state_view_mut().set_maiden_warp_step(224);
            self.attract_state_view_mut().increment_scene_substep();
        } else if self.attract_state_view().maiden_warp_step() < 0x0f {
            let k = (self.attract_state_view().maiden_warp_step() >> 3) as usize & 1;
            self.attract_state_view_mut().set_x_base(X_BASE[k]);
            self.attract_state_view_mut().set_y_base(0x60);
            let n = if k != 0 { 2 } else { 1 };
            self.attract_draw_sprite_set2_slice(&OAMS[k..k + n]);
        }
        self.attract_state_view_mut().increment_maiden_warp_step();
    }

    pub(super) fn attract_maiden_warp_case4(&mut self) {
        self.attract_show_timed_text_message();
        if self.oam_state_view().priority_word() == 0 {
            if self.attract_state_view().maiden_warp_step() < 31
                && self.attract_state_view().maiden_warp_step() & 1 == 0
            {
                self.decrement_screen_brightness();
            }
            self.attract_state_view_mut().decrement_maiden_warp_step();
            if self.attract_state_view().maiden_warp_step() == 0 {
                self.attract_state_view_mut().increment_scene_done_flag();
            }
        }
    }

    pub(super) fn attract_dramatize_prison(&mut self) {
        const PRISON_ZELDA_ANIMATION_FRAMES: [u8; 16] =
            [0, 1, 2, 3, 4, 5, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1];
        const SOLDIER_X: [i16; 2] = [32, -12];
        const SOLDIER_Y: [u16; 2] = [24, 24];
        const SOLDIER_DIR: [u8; 2] = [1, 1];
        const SOLDIER_FLAGS: [u8; 2] = [9, 7];

        self.attract_state_view_mut().set_oam_index(0);
        if self.attract_state_view().fade_in_done_flag() == 0 {
            self.attract_fade_in_step();
        }

        self.attract_state_view_mut().set_x_base(56);
        self.attract_draw_zelda();

        if self.attract_state_view().scene_timer() >= 192 {
            self.attract_state_view_mut().set_y_base(112);
            let frame_ctr = self
                .attract_state_view_mut()
                .decrement_scene_frame_counter();
            let frame_ctr = if (frame_ctr as i8) < 0 {
                self.attract_state_view_mut().set_scene_frame_counter(0x0f);
                0x0f
            } else {
                frame_ctr
            };

            let t = self
                .attract_state_view()
                .vram_dst_word()
                .wrapping_add(PRISON_ZELDA_ANIMATION_FRAMES[frame_ctr as usize] as u16);
            self.attract_state_view_mut()
                .set_x_base_high((t >> 8) as u8);
            self.attract_state_view_mut().set_x_base(t as u8);
            self.attract_zelda_prison_draw_a();

            for k in (0..=1).rev() {
                let sprite = k * 2;
                self.sprite_prep_reset_properties(sprite);
                let x = (self.attract_state_view().vram_dst_word() as i16)
                    .wrapping_add(0x100)
                    .wrapping_add(SOLDIER_X[k]) as u16;
                self.attract_state_view_mut()
                    .set_prison_soldier_x_lo(x as u8);
                let soldier_anim = self.attract_state_view().soldier_anim_step();
                self.sprite_simulate_soldier(
                    sprite,
                    x,
                    (self.attract_state_view().y_base() as u16).wrapping_add(SOLDIER_Y[k]),
                    SOLDIER_DIR[k],
                    SOLDIER_FLAGS[k],
                    soldier_anim,
                );
            }

            let step_ctr = self.attract_state_view_mut().increment_anim_step_counter();
            if step_ctr & 7 == 0 {
                if self.attract_state_view().soldier_anim_step() == 2 {
                    self.attract_state_view_mut().set_soldier_anim_step(0xff);
                    if self.attract_vram_target_view().high() == 0 && step_ctr & 8 != 0 {
                        self.system_signals_view_mut().set_sound_effect_2(4);
                    }
                }
                self.attract_state_view_mut().increment_soldier_anim_step();
            }
        }

        match self.attract_state_view().scene_substep() {
            0 => self.attract_zelda_prison_case0(),
            1 => self.attract_zelda_prison_case1(),
            _ => {}
        }
    }

    pub(super) fn attract_fade_in_step(&mut self) {
        if self.display_state().screen_brightness != 15 {
            if (self.player_state_view_mut().decrement_speed_setting() as i8) < 0 {
                self.increment_screen_brightness();
                self.player_state_view_mut().set_speed_setting(1);
            }
        } else {
            self.attract_state_view_mut().increment_fade_in_done_flag();
        }
    }

    pub(super) fn attract_draw_zelda(&mut self) {
        let start = self.attract_state_view().oam_index() as usize;
        self.set_oam_plain(
            64 + start,
            0x60,
            self.attract_state_view().x_base(),
            0x28,
            0x29,
            2,
        );
        self.set_oam_plain(
            65 + start,
            0x60,
            self.attract_state_view().x_base().wrapping_add(10),
            0x2a,
            0x29,
            2,
        );
        self.attract_state_view_mut().advance_oam_index_by(2);
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
        let start = self.attract_state_view().oam_index() as usize;
        self.attract_state_view_mut()
            .advance_oam_index_by((n as u8).wrapping_add(1));
        for i in (0..=n).rev() {
            self.set_oam_plain(
                64 + start + (n - i),
                self.attract_state_view().x_base().wrapping_add(xp[i]),
                self.attract_state_view().y_base().wrapping_add(yp[i]),
                cp[i],
                fp[i],
                ep[i],
            );
        }
    }

    pub(super) fn attract_zelda_prison_draw_a(&mut self) {
        let start = self.attract_state_view().oam_index() as usize;
        let ext = if self.attract_state_view().x_base_high() != 0 {
            3
        } else {
            2
        };
        let j = (self.attract_state_view().anim_step_counter() >> 3) & 1;
        self.set_oam_plain(
            64 + start,
            self.attract_state_view().x_base(),
            self.attract_state_view().y_base().wrapping_add(j),
            6,
            0x3d,
            ext,
        );
        self.set_oam_plain(
            65 + start,
            self.attract_state_view().x_base(),
            self.attract_state_view().y_base().wrapping_add(10),
            if j != 0 { 10 } else { 8 },
            0x3d,
            ext,
        );
        self.attract_state_view_mut().advance_oam_index_by(2);
    }

    pub(super) fn attract_zelda_prison_case0(&mut self) {
        const OAMS0: [(i8, i8, u8, u8, u8); 6] = [
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x84, 0x3b, 2),
            (16, 0, 0x84, 0x7b, 2),
            (0, 16, 0xa4, 0x3b, 2),
            (16, 16, 0xa4, 0x7b, 2),
        ];
        if self.attract_state_view().prison_soldier_x_lo() == 0 {
            self.attract_state_view_mut().increment_scene_substep();
        }
        if self.frame_state().frame_counter & 1 != 0 {
            self.attract_state_view_mut().decrement_vram_dst_word();
        }
        self.attract_state_view_mut().set_x_base(0x58);
        let y_base = self.attract_state_view().prison_zelda_y_base();
        self.attract_state_view_mut().set_y_base(y_base);
        self.attract_draw_sprite_set2_slice(&OAMS0);
        self.attract_state_view_mut().set_story_text_pointer(0xf8d9);
    }

    pub(super) fn attract_zelda_prison_case1(&mut self) {
        const OAMS1: [(i8, i8, u8, u8, u8); 30] = [
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x84, 0x3b, 2),
            (16, 0, 0x84, 0x7b, 2),
            (0, 16, 0xa4, 0x3b, 2),
            (16, 16, 0xa4, 0x7b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0xc4, 0x3b, 2),
            (16, 0, 0xc2, 0x3b, 2),
            (0, 16, 0xe4, 0x3b, 2),
            (16, 16, 0xe6, 0x3b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x88, 0x3b, 2),
            (16, 0, 0x8a, 0x3b, 2),
            (0, 16, 0xa8, 0x3b, 2),
            (16, 16, 0xaa, 0x3b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x82, 0x3b, 2),
            (16, 0, 0x82, 0x7b, 2),
            (0, 16, 0xa2, 0x3b, 2),
            (16, 16, 0xa2, 0x7b, 2),
            (5, 25, 0x6c, 0x38, 2),
            (11, 25, 0x6c, 0x38, 2),
            (0, 0, 0x80, 0x3b, 2),
            (16, 0, 0x80, 0x7b, 2),
            (0, 16, 0xa0, 0x3b, 2),
            (16, 16, 0xa0, 0x7b, 2),
        ];
        let Some(k) = (if self.attract_state_view().scene_timer() < 0x80 {
            self.attract_show_timed_text_message();
            if self.oam_state_view().priority_word() != 0 {
                Some(4)
            } else {
                self.attract_zelda_prison_case1_step()
            }
        } else {
            self.attract_zelda_prison_case1_step()
        }) else {
            return;
        };

        if self.frame_state().frame_counter & 1 != 0 {
            self.attract_state_view_mut().decrement_vram_dst_word();
        }
        self.attract_state_view_mut().set_x_base(0x58);
        let y_base = self.attract_state_view().prison_zelda_y_base();
        self.attract_state_view_mut().set_y_base(y_base);
        self.attract_draw_sprite_set2_slice(&OAMS1[k * 6..k * 6 + 6]);
    }

    fn attract_zelda_prison_case1_step(&mut self) -> Option<usize> {
        if self.attract_state_view().prison_zelda_y_base() != 0x6e {
            self.attract_state_view_mut()
                .decrement_prison_zelda_y_base();
            return Some(0);
        }

        if self.attract_state_view().scene_timer() < 31
            && self.attract_state_view().scene_timer() & 1 == 0
        {
            self.decrement_screen_brightness();
        }
        self.attract_state_view_mut().decrement_scene_timer();
        if self.attract_state_view().scene_timer() == 0 {
            self.attract_state_view_mut().increment_sequence();
            self.attract_state_view_mut().subtract_state(2);
            return None;
        }

        Some(match self.attract_state_view().scene_timer() {
            0xc0..=0xff => 0,
            0xb8..=0xbf => 1,
            0xb0..=0xb7 => 2,
            0xa0..=0xaf => 3,
            _ => 4,
        })
    }

    #[rustfmt::skip]
    pub(super) fn sprite_simulate_soldier(&mut self, k: usize, x: u16, y: u16, dir: u8, flags: u8, gfx: u8) {
        const SIMULATE_SOLDIER_GFX: [u8; 4] = [11, 4, 0, 7];
        self.sprite_set_x(k, x);
        self.sprite_set_y(k, y);
        self.sprite_slot_view_mut(k).set_z(0);
        self.sprite_get_16_bit_coords(k);
        let mut soldier = self.sprite_slot_view_mut(k);
        soldier.set_direction(dir);
        soldier.set_head_direction(dir);
        soldier.set_graphics(SIMULATE_SOLDIER_GFX[dir as usize].wrapping_add(gfx));
        soldier.set_flags3(16);
        soldier.set_object_priority(0);
        soldier.set_oam_flags(flags | 0x30);
        soldier.set_sprite_type(if flags == 9 { 0x41 } else { 0x43 });
        soldier.set_flags2(7);
        let oam_idx = k * 8;
        self.oam_state_view_mut()
            .set_current_pointer((OAM_BUF + oam_idx * 4) as u16);
        self.oam_state_view_mut()
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
            .wrapping_sub(self.world_state_view().bg2_y());
        if y.wrapping_add(0x10) >= 0x100 {
            return;
        }
        let oam_offs = (self.sprite_slot_view(k).flags2() & 0x1f) as usize;
        self.set_guard_oam(oam_offs, poc.0, y, 0x6c, (poc.2 & 0x30) | 8, 2);
    }

    fn set_guard_oam(&mut self, offset: usize, x: u16, y: u16, charnum: u8, flags: u8, big: u8) {
        let oam_cur = self.oam_state_view().current_pointer_usize();
        let index = (oam_cur - OAM_BUF) / 4 + offset;
        self.set_oam_helper0_index(index, x, y, charnum, flags, big);
    }

    pub(super) fn attract_dramatize_polka_dots(&mut self) {
        if self.frame_state().frame_counter & 3 == 0 {
            self.ppu_scroll_copy_view_mut().add_bg1_v_copy_low(1);
            self.ppu_scroll_copy_view_mut().add_bg1_h_copy_low(1);
            self.ppu_scroll_copy_view_mut().add_bg2_v_copy_low(1);
            self.ppu_scroll_copy_view_mut().subtract_bg2_h_copy_low(1);
        }

        if self.attract_state_view().legend_flag() != 0 {
            self.attract_build_next_image_tile_map();
            self.attract_state_view_mut().clear_legend_flag();
            self.attract_state_view_mut().advance_next_legend_gfx();
        }

        self.player_state_view_mut().set_joypad1l_last(0);
        self.player_state_view_mut().set_filtered_joypad_l(0);
        self.player_state_view_mut().set_filtered_joypad_h(0);
        self.RenderText();

        let legend_ctr = self.attract_state_view_mut().decrement_legend_ctr();
        if legend_ctr == 0 {
            self.attract_state_view_mut().increment_sequence();
            self.attract_state_view_mut().subtract_state(3);
        } else if legend_ctr < 0x18 && legend_ctr & 1 != 0 {
            self.decrement_screen_brightness();
        }
    }

    pub(super) fn attract_build_next_image_tile_map(&mut self) {
        let image = self.attract_state_view().next_legend_image();
        let data = match image {
            0 => &ATTRACT_LEGEND_TILEMAP_BYTES_0[..],
            1 => &ATTRACT_LEGEND_TILEMAP_BYTES_1[..],
            2 => &ATTRACT_LEGEND_TILEMAP_BYTES_2[..],
            3 => &ATTRACT_LEGEND_TILEMAP_BYTES_3[..],
            _ => &[],
        };
        let len = data.len().min(
            self.ram
                .len()
                .saturating_sub(self.vram_upload_data_view().data_base()),
        );
        self.vram_upload_data_view_mut().copy_bytes(0, &data[..len]);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn attract_build_backgrounds(&mut self) {
        const BACKGROUND_TILE_PATTERN: [u16; 16] = [
            0x01a0, 0x09a6, 0x89a5, 0x01a0, 0x09a5, 0x01a0, 0x01a0, 0x89a6, 0x49a5, 0x01a0, 0x01a0,
            0x49a5, 0x01a0, 0x89a5, 0xc9a5, 0x01a0,
        ];
        const BACKGROUND_CORNER_TILES: [u16; 4] = [0x09a1, 0x09a2, 0x09a3, 0x09a4];

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
                self.vram_upload_data_view_mut()
                    .set_word(4 + k * 2, BACKGROUND_TILE_PATTERN[p + j]);
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
                self.vram_upload_data_view_mut()
                    .set_word(4 + k * 2, BACKGROUND_CORNER_TILES[p + j]);
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
        self.attract_state_view_mut().set_vram_dst_word(0);
    }

    pub(super) fn attract_trigger_bgdma(&mut self, dstv: usize) {
        for row in 0..8 {
            let dst = dstv + row * 0x80;
            for i in 0..0x80 {
                self.ppu.vram[dst + i] = self.vram_upload_data_view().word(4 + i * 2);
            }
        }
    }
}
