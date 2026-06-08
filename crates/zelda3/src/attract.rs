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
        let mut state = self.ram[ATTRACT_STATE];
        if self.ram[INIDISP_COPY] != 0
            && self.ram[INIDISP_COPY] != 128
            && state != 0
            && state != 2
            && state != 6
            && self.ram[FILTERED_JOYPAD_H] & 0x90 != 0
        {
            self.ram[ATTRACT_STATE] = 9;
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
        self.ram[INTRO_DID_RUN_STEP] = 0;
        self.ram[IS_NMI_THREAD_ACTIVE] = 0;
        self.intro_periodic_sword_and_intro_flash();
        if self.ram[INIDISP_COPY] != 0 {
            self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
            return;
        }

        self.enable_force_blank();
        self.ram[IRQ_FLAG] = 0xff;
        self.ram[IS_NMI_THREAD_ACTIVE] = 0;
        self.ram[NMI_FLAG_UPDATE_POLYHEDRAL] = 0;
        self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(1);
    }

    pub(super) fn attract_init_graphics(&mut self) {
        self.fill_ram(ATTRACT_LOW_RAM_CLEAR_START, ATTRACT_LOW_RAM_CLEAR_LEN, 0);
        self.erase_tile_maps_normal();
        self.Attract_LoadBG3GFX();
        self.ram[OVERWORLD_PALETTE_MODE] = 4;
        self.ram[HUD_PALETTE] = 1;
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0);
        self.palette_load_hud();
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x200);
        self.palette_load_ow_bg_main();
        self.palette_load_hud();
        self.palette_load_link_armor_and_gloves();
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x1d * 2, 0x3800);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        self.ram[BG3VOFS_COPY2] = 20;
        self.attract_build_backgrounds();
        self.ram[MESSAGING_MODULE] = 0;
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x112);
        write_le_u16(&mut self.ram, BG2VOFS_COPY2, 0);
        write_le_u16(&mut self.ram, ATTRACT_LEGEND_CTR, 0x1010);
        self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(3);

        self.hdma_setup(0x0cfa87, 0x0cfa94, 1, 0x26, 0x28, 0);
        self.ram[HDMAEN_COPY] = 0xc0;
        self.ram[W12SEL_COPY] = 0;
        self.ram[W34SEL_COPY] = 0;
        self.ram[WOBJSEL_COPY] = 0xb0;
        self.ram[TMW_COPY] = 3;
        self.ram[TSW_COPY] = 0;
        self.ram[COLDATA_COPY0] = 0x25;
        self.ram[COLDATA_COPY1] = 0x45;
        self.ram[COLDATA_COPY2] = 0x85;
        self.ram[CGWSEL_COPY] = 0x10;
        self.ram[CGADSUB_COPY] = 0xa3;
        self.ram[MUSIC_CONTROL] = 6;
        self.ram[ATTRACT_LEGEND_FLAG] = self.ram[ATTRACT_LEGEND_FLAG].wrapping_add(1);
    }

    pub(super) fn attract_load_new_scene(&mut self) {
        match self.ram[ATTRACT_SEQUENCE] {
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
        self.ram[ATTRACT_NEXT_LEGEND_GFX] = 0;
        self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(1);
        self.ram[INIDISP_COPY] = 0;
    }

    pub(super) fn attract_scene_world_map(&mut self) {
        self.zelda_ppu_write(0x2107, 0x13);
        self.zelda_ppu_write(0x2108, 0x03);
        self.ram[CGWSEL_COPY] = 0x80;
        self.ram[CGADSUB_COPY] = 0x21;
        self.ram[BGMODE_COPY] = 7;
        self.world_map_load_light_world_map();
        write_le_u16(&mut self.ram, M7Y_COPY, 0x00ed);
        write_le_u16(&mut self.ram, M7X_COPY, 0x0100);
        write_le_u16(&mut self.ram, BG1HOFS_COPY, 0x0080);
        write_le_u16(&mut self.ram, BG1VOFS_COPY, 0x00c0);
        self.ram[TIMER_FOR_MODE7_ZOOM] = 0xff;
        self.attract_control_map_zoom();
        self.ram[ATTRACT_SCENE_TIMER] = 1;
        self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(1);
        self.ram[INIDISP_COPY] = 0;
    }

    pub(super) fn attract_scene_throne_room(&mut self) {
        self.ram[HDMAEN_COPY] = 0;
        self.ram[CGWSEL_COPY] = 2;
        self.ram[CGADSUB_COPY] = 0x20;
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = 10;
        self.load_common_sprites();

        let attract_bg2_vofs_backup = read_le_u16(&self.ram, ATTRACT_BG2_VOFS_BACKUP);
        let attract_state = read_le_u16(&self.ram, ATTRACT_STATE);
        self.dungeon_load_and_draw_entrance_room(0x74);
        write_le_u16(&mut self.ram, ATTRACT_STATE, attract_state);
        write_le_u16(
            &mut self.ram,
            ATTRACT_BG2_VOFS_BACKUP,
            attract_bg2_vofs_backup,
        );

        self.ram[PALETTE_MAIN_INDOORS] = 0;
        self.ram[PALETTE_SP0L] = 0;
        self.ram[PALETTE_SP5L] = 14;
        self.ram[PALETTE_SP6L] = 3;
        self.dungeon_save_and_load_load_all_palettes(0, 0x7e);

        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x1d * 2, 0x3800);
        self.ram[MESSAGING_MODULE] = 0;
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x113);
        self.ram[ATTRACT_SCENE_TIMER] = 2;
        self.ram[ATTRACT_THRONE_FADE_TIMER] = 0xe0;
        write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, 0x210);

        self.attract_prep_finish();
    }

    pub(super) fn attract_prep_zelda_prison(&mut self) {
        self.ram[CGWSEL_COPY] = 0;
        self.ram[CGADSUB_COPY] = 0;

        let attract_bg2_vofs_backup = read_le_u16(&self.ram, ATTRACT_BG2_VOFS_BACKUP);
        let attract_state = read_le_u16(&self.ram, ATTRACT_STATE);
        self.dungeon_load_and_draw_entrance_room(0x73);
        write_le_u16(&mut self.ram, ATTRACT_STATE, attract_state);
        write_le_u16(
            &mut self.ram,
            ATTRACT_BG2_VOFS_BACKUP,
            attract_bg2_vofs_backup,
        );

        self.ram[PALETTE_MAIN_INDOORS] = 2;
        self.ram[PALETTE_SP0L] = 0;
        self.ram[PALETTE_SP5L] = 14;
        self.ram[PALETTE_SP6L] = 3;
        self.dungeon_save_and_load_load_all_palettes(1, 0x7f);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x1d * 2, 0x3800);

        self.ram[MESSAGING_MODULE] = 0;
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x114);

        self.ram[ATTRACT_PRISON_ZELDA_Y_BASE] = 148;
        self.ram[ATTRACT_VRAM_DST] = 0x68;
        self.ram[ATTRACT_VRAM_DST + 1] = 0;
        self.ram[ATTRACT_ANIM_STEP_COUNTER] = 0;
        self.ram[ATTRACT_SOLDIER_ANIM_STEP] = 0;
        self.ram[ATTRACT_X_BASE_HI] = 0;
        self.ram[ATTRACT_SCENE_FRAME_COUNTER] = 0;
        self.ram[ATTRACT_FADE_IN_DONE_FLAG] = 0;
        self.ram[ATTRACT_SCENE_TIMER] = 255;
        write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, 0x240);
        self.attract_prep_finish();
    }

    pub(super) fn attract_prep_maiden_warp(&mut self) {
        let attract_bg2_vofs_backup = read_le_u16(&self.ram, ATTRACT_BG2_VOFS_BACKUP);
        let attract_state = read_le_u16(&self.ram, ATTRACT_STATE);
        self.dungeon_load_and_draw_entrance_room(0x75);
        write_le_u16(&mut self.ram, ATTRACT_STATE, attract_state);
        write_le_u16(
            &mut self.ram,
            ATTRACT_BG2_VOFS_BACKUP,
            attract_bg2_vofs_backup,
        );

        self.ram[PALETTE_MAIN_INDOORS] = 0;
        self.ram[PALETTE_SP0L] = 0;
        self.ram[PALETTE_SP5L] = 14;
        self.ram[PALETTE_SP6L] = 3;
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0);
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sprite_environment_dungeon();
        self.palette_load_hud();
        self.palette_load_dungeon_set();
        self.dungeon_save_and_load_load_all_palettes(2, 0x7f);
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + 0x1d * 2, 0x3800);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x1d * 2, 0x3800);

        self.ram[MESSAGING_MODULE] = 0;
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x115);
        self.ram[ATTRACT_SCENE_TIMER] = 255;
        self.ram[ATTRACT_VRAM_DST] = 112;
        self.ram[ATTRACT_MAIDEN_WARP_TIMER_A] = 112;
        self.ram[ATTRACT_MAIDEN_WARP_TIMER_B] = 112;
        self.ram[ATTRACT_ANIM_STEP_COUNTER] = 8;
        self.ram[ATTRACT_SCENE_FRAME_COUNTER] = 0;
        self.ram[ATTRACT_MAIDEN_WARP_STEP] = 0;
        self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG] = 0;
        self.ram[ATTRACT_FADE_IN_DONE_FLAG] = 0;
        self.ram[ATTRACT_SCENE_SUBSTEP] = 0;
        self.ram[ATTRACT_SUBSTEP_DELAY_COUNTER] = 0;
        write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, 0x00c0);
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
        self.ram[HDMAEN_COPY] = 0x80;
        self.ram[BGMODE_COPY] = 9;
        self.ram[NMI_DISABLE_CORE_UPDATES] = 0;
    }

    pub(super) fn death_func31(&mut self) {
        self.ram[NMI_DISABLE_CORE_UPDATES] = self.ram[NMI_DISABLE_CORE_UPDATES].wrapping_add(1);
        self.intro_initialize_memory_darken();
        self.overworld_load_all_palettes();
        self.ram[BG3VOFS_COPY2] = 0;
        write_le_u16(&mut self.ram, M7Y_COPY, 0);
        write_le_u16(&mut self.ram, M7X_COPY, 0);
        write_le_u16(&mut self.ram, BG1HOFS_COPY, 0);
        write_le_u16(&mut self.ram, BG1VOFS_COPY, 0);
        write_le_u16(&mut self.ram, BG2HOFS_COPY, 0);
        write_le_u16(&mut self.ram, BG2VOFS_COPY, 0);
        self.ram[MUSIC_CONTROL] = 0xf1;
        self.ram[ATTRACT_SEQUENCE] = 0;
        self.frame_control_view_mut().set_main_module(0);
        self.frame_control_view_mut().set_submodule(10);
        self.frame_control_view_mut().set_subsubmodule(10);
    }

    pub(super) fn attract_skip_to_file_select(&mut self) {
        self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
        if self.ram[INIDISP_COPY] != 0 {
            return;
        }
        self.enable_force_blank();
        self.zelda_ppu_write(0x2107, 0x13);
        self.zelda_ppu_write(0x2108, 0x03);
        self.attract_setup_conclusion_hdma();
        write_le_u16(&mut self.ram, M7Y_COPY, 0);
        write_le_u16(&mut self.ram, M7X_COPY, 0);
        write_le_u16(&mut self.ram, BG1HOFS_COPY, 0);
        write_le_u16(&mut self.ram, BG1VOFS_COPY, 0);
        self.ram[BG3VOFS_COPY2] = 0;
        self.fade_music_and_reset_sram_mirror();
    }

    pub(super) fn attract_prep_finish(&mut self) {
        self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(1);
        self.ram[INIDISP_COPY] = 0;
        self.ram[BG3VOFS_COPY2] = 0;
        let bg2_hofs = read_le_u16(&self.ram, BG2HOFS_COPY) & 0x01ff;
        let bg2_vofs = read_le_u16(&self.ram, BG2VOFS_COPY) & 0x01ff;
        let bg2_hofs2 = read_le_u16(&self.ram, BG2HOFS_COPY2) & 0x01ff;
        let bg2_vofs2 = read_le_u16(&self.ram, BG2VOFS_COPY2) & 0x01ff;
        write_le_u16(&mut self.ram, BG2HOFS_COPY, bg2_hofs);
        write_le_u16(&mut self.ram, BG2VOFS_COPY, bg2_vofs);
        write_le_u16(&mut self.ram, BG2HOFS_COPY2, bg2_hofs2);
        write_le_u16(&mut self.ram, BG2VOFS_COPY2, bg2_vofs2);
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
        let zoom = self.ram[TIMER_FOR_MODE7_ZOOM] as u16;
        for (i, value) in MAP_MODE_ZOOMS1.iter().enumerate() {
            write_le_u16(
                &mut self.ram,
                HDMA_TABLE_DYNAMIC + i * 2,
                ((*value as u32 * zoom as u32) >> 8) as u16,
            );
        }
    }

    pub(super) fn attract_fade_in_sequence(&mut self) {
        if self.ram[INIDISP_COPY] != 15 {
            self.ram[LINK_SPEED_SETTING] = self.ram[LINK_SPEED_SETTING].wrapping_sub(1);
            if (self.ram[LINK_SPEED_SETTING] as i8) < 0 {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_add(1);
                self.ram[LINK_SPEED_SETTING] = 1;
            }
        } else {
            self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(1);
        }
    }

    pub(super) fn attract_fade_out_sequence(&mut self) {
        if self.ram[INIDISP_COPY] != 0 {
            self.ram[LINK_SPEED_SETTING] = self.ram[LINK_SPEED_SETTING].wrapping_sub(1);
            if (self.ram[LINK_SPEED_SETTING] as i8) < 0 {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
                self.ram[LINK_SPEED_SETTING] = 1;
            }
        } else {
            self.enable_force_blank();
            self.erase_tile_maps_normal();
            self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(1);
        }
    }

    pub(super) fn attract_enact_story(&mut self) {
        match self.ram[ATTRACT_SEQUENCE] {
            0 => self.attract_dramatize_polka_dots(),
            1 => self.attract_dramatize_world_map(),
            2 => self.attract_throne_room(),
            3 => self.attract_dramatize_prison(),
            4 => self.attract_dramatize_agahnim_altar(),
            _ => {}
        }
    }

    pub(super) fn attract_dramatize_world_map(&mut self) {
        if self.ram[TIMER_FOR_MODE7_ZOOM] != 0 {
            if self.ram[TIMER_FOR_MODE7_ZOOM] < 15 {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
            }
            self.ram[ATTRACT_SCENE_TIMER] = self.ram[ATTRACT_SCENE_TIMER].wrapping_sub(1);
            if self.ram[ATTRACT_SCENE_TIMER] == 0 {
                self.ram[ATTRACT_SCENE_TIMER] = 1;
                self.ram[TIMER_FOR_MODE7_ZOOM] = self.ram[TIMER_FOR_MODE7_ZOOM].wrapping_sub(1);
                self.attract_control_map_zoom();
            }
        } else {
            self.enable_force_blank();
            self.ram[BGMODE_COPY] = 9;
            self.erase_tile_maps_normal();
            self.ram[ATTRACT_SEQUENCE] = self.ram[ATTRACT_SEQUENCE].wrapping_add(1);
            self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_sub(2);
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

        self.ram[ATTRACT_OAM_IDX] = 0;
        if self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG] == 0 {
            if self.ram[INIDISP_COPY] != 15 {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_add(1);
            } else {
                self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG] =
                    self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG].wrapping_add(1);
            }
        }

        if read_le_u16(&self.ram, BG2VOFS_COPY) == 0 {
            self.attract_show_timed_text_message();
            if read_le_u16(&self.ram, OAM_PRIORITY_VALUE) == 0 {
                if self.ram[ATTRACT_THRONE_FADE_TIMER] < 31
                    && self.ram[ATTRACT_THRONE_FADE_TIMER] & 1 == 0
                {
                    self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
                }
                self.ram[ATTRACT_THRONE_FADE_TIMER] =
                    self.ram[ATTRACT_THRONE_FADE_TIMER].wrapping_sub(1);
                if self.ram[ATTRACT_THRONE_FADE_TIMER] == 0 {
                    self.ram[ATTRACT_SEQUENCE] = self.ram[ATTRACT_SEQUENCE].wrapping_add(1);
                    self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(1);
                    return;
                }
            }
        } else {
            let bg2 = read_le_u16(&self.ram, BG2VOFS_COPY).wrapping_sub(1);
            let bg1 = read_le_u16(&self.ram, BG1VOFS_COPY).wrapping_sub(1);
            write_le_u16(&mut self.ram, BG2VOFS_COPY, bg2);
            write_le_u16(&mut self.ram, BG1VOFS_COPY, bg1);
        }

        let bg2_vofs = read_le_u16(&self.ram, BG2VOFS_COPY);
        for i in (0..=1).rev() {
            let y = (Y_BASE[i] as u16).wrapping_sub(bg2_vofs);
            if (y.wrapping_add(32) as i16) >= 0 {
                self.ram[ATTRACT_X_BASE] = X_BASE[i];
                self.ram[ATTRACT_Y_BASE] = y as u8;
                self.attract_draw_sprite_set2_slice(
                    &THRONE_ROOM_OAMS[OAM_OFFS[i]..OAM_OFFS[i + 1]],
                );
            }
        }

        write_le_u16(&mut self.ram, ATTRACT_VAR7, 0xf8a7);
    }

    pub(super) fn attract_show_timed_text_message(&mut self) {
        let bg2_vofs2 = read_le_u16(&self.ram, BG2VOFS_COPY2);
        write_le_u16(&mut self.ram, ATTRACT_BG2_VOFS_BACKUP, bg2_vofs2);
        self.ram[JOYPAD1L_LAST] = 0;
        self.ram[FILTERED_JOYPAD_L] = 0;
        self.ram[FILTERED_JOYPAD_H] = 0;
        self.RenderText();
        let priority = read_le_u16(&self.ram, OAM_PRIORITY_VALUE);
        if priority != 0 {
            write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, priority.wrapping_sub(1));
        }
    }

    #[rustfmt::skip]
    pub(super) fn attract_draw_sprite_set2(&mut self, p: *const AttractOamInfo, n: i32) {
        self.attract_draw_sprite_set2_slice(unsafe {
            std::slice::from_raw_parts(p, n as usize)
        });
    }

    pub(super) fn attract_draw_sprite_set2_slice(&mut self, entries: &[AttractOamInfo]) {
        let start = self.ram[ATTRACT_OAM_IDX] as usize;
        self.ram[ATTRACT_OAM_IDX] = self.ram[ATTRACT_OAM_IDX].wrapping_add(entries.len() as u8);
        for (slot, &(x, y, charnum, flags, big)) in entries.iter().rev().enumerate() {
            let obj_x = self.ram[ATTRACT_X_BASE].wrapping_add_signed(x);
            let obj_y = self.ram[ATTRACT_Y_BASE].wrapping_add_signed(y);
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

        if self.ram[ATTRACT_SCENE_DONE_FLAG] != 0 {
            self.ram[ATTRACT_SEQUENCE] = self.ram[ATTRACT_SEQUENCE].wrapping_add(1);
            self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_sub(2);
            return;
        }

        self.ram[ATTRACT_OAM_IDX] = 0;
        self.handle_screen_flash();
        if self.ram[ATTRACT_FADE_IN_DONE_FLAG] == 0 {
            self.attract_fade_in_step();
        }
        if self.ram[ATTRACT_SCENE_FRAME_COUNTER] != 0xff {
            self.ram[ATTRACT_SCENE_FRAME_COUNTER] =
                self.ram[ATTRACT_SCENE_FRAME_COUNTER].wrapping_add(1);
        }
        if self.ram[INTRO_TIMES_PAL_FLASH] & 4 != 0 {
            self.ram[SOUND_EFFECT_2] = 0x2b;
        }

        match self.ram[ATTRACT_SCENE_SUBSTEP] {
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

        if self.ram[ATTRACT_SCENE_FRAME_COUNTER] >= 0xa0 {
            if self.ram[ATTRACT_VRAM_DST] != 0x60 {
                self.ram[ATTRACT_ANIM_STEP_COUNTER] =
                    self.ram[ATTRACT_ANIM_STEP_COUNTER].wrapping_sub(1);
                if self.ram[ATTRACT_ANIM_STEP_COUNTER] == 0 {
                    self.ram[ATTRACT_VRAM_DST] = self.ram[ATTRACT_VRAM_DST].wrapping_sub(1);
                    self.ram[ATTRACT_ANIM_STEP_COUNTER] = 8;
                }
            } else {
                self.ram[ATTRACT_SUBSTEP_DELAY_COUNTER] =
                    self.ram[ATTRACT_SUBSTEP_DELAY_COUNTER].wrapping_add(1);
            }
        }

        if self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG] == 0 {
            self.ram[ATTRACT_X_BASE] = 116;
            self.ram[ATTRACT_Y_BASE] = self.ram[ATTRACT_VRAM_DST];
            let start = if self.ram[ATTRACT_VRAM_DST] == 0x70 {
                0
            } else {
                2
            };
            self.attract_draw_sprite_set2_slice(&MAIDEN_WARP0[start..start + 2]);

            let mut k = 7usize;
            if self.ram[ATTRACT_VRAM_DST] < 0x68 {
                k = self.ram[ATTRACT_VRAM_DST].wrapping_sub(0x68) as usize & 7;
            }
            self.ram[ATTRACT_X_BASE] = 0x74 + MAIDEN_WARP_XBASE[k];
            self.ram[ATTRACT_Y_BASE] = 0x76;
            self.attract_draw_sprite_set2_slice(&MAIDEN_WARP1[k * 2..k * 2 + 2]);
        }

        let k = (self.ram[ATTRACT_SCENE_FRAME_COUNTER] >> 5) as usize & 7;
        self.ram[ATTRACT_X_BASE] = 112;
        self.ram[ATTRACT_Y_BASE] = 70;
        self.attract_draw_sprite_set2_slice(&MAIDEN_WARP2[k * 6..k * 6 + 6]);
    }

    pub(super) fn attract_maiden_warp_case0(&mut self) {
        if self.ram[ATTRACT_SUBSTEP_DELAY_COUNTER] != 0 {
            self.ram[ATTRACT_SCENE_SUBSTEP] = self.ram[ATTRACT_SCENE_SUBSTEP].wrapping_add(1);
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

        let k = (self.ram[FRAME_COUNTER] >> 2) as usize & 1;
        let n = NUM[(self.ram[ATTRACT_MAIDEN_WARP_STEP] >> 1) as usize & 7];
        self.ram[ATTRACT_X_BASE] = 110;
        self.ram[ATTRACT_Y_BASE] = 72;
        self.attract_draw_sprite_set2_slice(&OAMS[k * 14..k * 14 + n]);

        if self.ram[ATTRACT_MAIDEN_WARP_STEP] == 0 && self.ram[ATTRACT_MAIDEN_WARP_TIMER_B] == 0x70
        {
            self.ram[SOUND_EFFECT_2] = 0x27;
        }

        if self.ram[ATTRACT_MAIDEN_WARP_STEP] == 15 {
            self.ram[ATTRACT_SCENE_SUBSTEP] = self.ram[ATTRACT_SCENE_SUBSTEP].wrapping_add(1);
        } else {
            if self.ram[ATTRACT_MAIDEN_WARP_STEP] == 6 {
                self.ram[INTRO_TIMES_PAL_FLASH] = 0x90;
                self.ram[SOUND_EFFECT_2] = 0x2b;
            }
            if self.ram[ATTRACT_MAIDEN_WARP_TIMER_B] != 0 {
                self.ram[ATTRACT_MAIDEN_WARP_TIMER_B] =
                    self.ram[ATTRACT_MAIDEN_WARP_TIMER_B].wrapping_sub(1);
            } else {
                self.ram[ATTRACT_MAIDEN_WARP_STEP] =
                    self.ram[ATTRACT_MAIDEN_WARP_STEP].wrapping_add(1);
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

        self.ram[ATTRACT_X_BASE] = 110;
        self.ram[ATTRACT_Y_BASE] = 72;
        let k = (self.ram[FRAME_COUNTER] >> 2) as usize & 1;
        let n = NUM[(self.ram[ATTRACT_MAIDEN_WARP_STEP] >> 1) as usize & 7];
        self.attract_draw_sprite_set2_slice(&OAMS[k * 14 + (14 - n)..k * 14 + 14]);

        if self.ram[ATTRACT_MAIDEN_WARP_STEP] == 0 {
            self.ram[ATTRACT_MAIDEN_WARP_TIMER_A] =
                self.ram[ATTRACT_MAIDEN_WARP_TIMER_A].wrapping_sub(1);
            if self.ram[ATTRACT_MAIDEN_WARP_TIMER_A] == 0 {
                self.ram[ATTRACT_SCENE_SUBSTEP] = self.ram[ATTRACT_SCENE_SUBSTEP].wrapping_add(1);
            }
        } else {
            self.ram[ATTRACT_MAIDEN_WARP_STEP] = self.ram[ATTRACT_MAIDEN_WARP_STEP].wrapping_sub(1);
        }
    }

    pub(super) fn attract_maiden_warp_case3(&mut self) {
        const OAMS: [(i8, i8, u8, u8, u8); 3] = [
            (0, 0, 0xc6, 0x3d, 2),
            (0, 0, 0x24, 0x35, 2),
            (16, 0, 0x24, 0x75, 2),
        ];
        const X_BASE: [u8; 2] = [0x78, 0x70];

        if self.ram[ATTRACT_MAIDEN_WARP_STEP] == 6 {
            self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG] =
                self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG].wrapping_add(1);
            self.ram[SOUND_EFFECT_1] = 51;
        } else if self.ram[ATTRACT_MAIDEN_WARP_STEP] == 0x40 {
            self.ram[ATTRACT_MAIDEN_WARP_STEP] = 224;
            self.ram[ATTRACT_SCENE_SUBSTEP] = self.ram[ATTRACT_SCENE_SUBSTEP].wrapping_add(1);
        } else if self.ram[ATTRACT_MAIDEN_WARP_STEP] < 0x0f {
            let k = (self.ram[ATTRACT_MAIDEN_WARP_STEP] >> 3) as usize & 1;
            self.ram[ATTRACT_X_BASE] = X_BASE[k];
            self.ram[ATTRACT_Y_BASE] = 0x60;
            let n = if k != 0 { 2 } else { 1 };
            self.attract_draw_sprite_set2_slice(&OAMS[k..k + n]);
        }
        self.ram[ATTRACT_MAIDEN_WARP_STEP] = self.ram[ATTRACT_MAIDEN_WARP_STEP].wrapping_add(1);
    }

    pub(super) fn attract_maiden_warp_case4(&mut self) {
        self.attract_show_timed_text_message();
        if read_le_u16(&self.ram, OAM_PRIORITY_VALUE) == 0 {
            if self.ram[ATTRACT_MAIDEN_WARP_STEP] < 31
                && self.ram[ATTRACT_MAIDEN_WARP_STEP] & 1 == 0
            {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
            }
            self.ram[ATTRACT_MAIDEN_WARP_STEP] = self.ram[ATTRACT_MAIDEN_WARP_STEP].wrapping_sub(1);
            if self.ram[ATTRACT_MAIDEN_WARP_STEP] == 0 {
                self.ram[ATTRACT_SCENE_DONE_FLAG] =
                    self.ram[ATTRACT_SCENE_DONE_FLAG].wrapping_add(1);
            }
        }
    }

    pub(super) fn attract_dramatize_prison(&mut self) {
        const TAB0: [u8; 16] = [0, 1, 2, 3, 4, 5, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1];
        const SOLDIER_X: [i16; 2] = [32, -12];
        const SOLDIER_Y: [u16; 2] = [24, 24];
        const SOLDIER_DIR: [u8; 2] = [1, 1];
        const SOLDIER_FLAGS: [u8; 2] = [9, 7];

        self.ram[ATTRACT_OAM_IDX] = 0;
        if self.ram[ATTRACT_FADE_IN_DONE_FLAG] == 0 {
            self.attract_fade_in_step();
        }

        self.ram[ATTRACT_X_BASE] = 56;
        self.attract_draw_zelda();

        if self.ram[ATTRACT_SCENE_TIMER] >= 192 {
            self.ram[ATTRACT_Y_BASE] = 112;
            self.ram[ATTRACT_SCENE_FRAME_COUNTER] =
                self.ram[ATTRACT_SCENE_FRAME_COUNTER].wrapping_sub(1);
            if (self.ram[ATTRACT_SCENE_FRAME_COUNTER] as i8) < 0 {
                self.ram[ATTRACT_SCENE_FRAME_COUNTER] = 0x0f;
            }

            let t = read_le_u16(&self.ram, ATTRACT_VRAM_DST)
                .wrapping_add(TAB0[self.ram[ATTRACT_SCENE_FRAME_COUNTER] as usize] as u16);
            self.ram[ATTRACT_X_BASE_HI] = (t >> 8) as u8;
            self.ram[ATTRACT_X_BASE] = t as u8;
            self.attract_zelda_prison_draw_a();

            for k in (0..=1).rev() {
                let sprite = k * 2;
                self.sprite_prep_reset_properties(sprite);
                let x = (read_le_u16(&self.ram, ATTRACT_VRAM_DST) as i16)
                    .wrapping_add(0x100)
                    .wrapping_add(SOLDIER_X[k]) as u16;
                self.ram[ATTRACT_PRISON_SOLDIER_X_LO] = x as u8;
                self.sprite_simulate_soldier(
                    sprite,
                    x,
                    (self.ram[ATTRACT_Y_BASE] as u16).wrapping_add(SOLDIER_Y[k]),
                    SOLDIER_DIR[k],
                    SOLDIER_FLAGS[k],
                    self.ram[ATTRACT_SOLDIER_ANIM_STEP],
                );
            }

            self.ram[ATTRACT_ANIM_STEP_COUNTER] =
                self.ram[ATTRACT_ANIM_STEP_COUNTER].wrapping_add(1);
            if self.ram[ATTRACT_ANIM_STEP_COUNTER] & 7 == 0 {
                if self.ram[ATTRACT_SOLDIER_ANIM_STEP] == 2 {
                    self.ram[ATTRACT_SOLDIER_ANIM_STEP] = 0xff;
                    if self.ram[ATTRACT_VRAM_DST + 1] == 0
                        && self.ram[ATTRACT_ANIM_STEP_COUNTER] & 8 != 0
                    {
                        self.ram[SOUND_EFFECT_2] = 4;
                    }
                }
                self.ram[ATTRACT_SOLDIER_ANIM_STEP] =
                    self.ram[ATTRACT_SOLDIER_ANIM_STEP].wrapping_add(1);
            }
        }

        match self.ram[ATTRACT_SCENE_SUBSTEP] {
            0 => self.attract_zelda_prison_case0(),
            1 => self.attract_zelda_prison_case1(),
            _ => {}
        }
    }

    pub(super) fn attract_fade_in_step(&mut self) {
        if self.ram[INIDISP_COPY] != 15 {
            self.ram[LINK_SPEED_SETTING] = self.ram[LINK_SPEED_SETTING].wrapping_sub(1);
            if (self.ram[LINK_SPEED_SETTING] as i8) < 0 {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_add(1);
                self.ram[LINK_SPEED_SETTING] = 1;
            }
        } else {
            self.ram[ATTRACT_FADE_IN_DONE_FLAG] =
                self.ram[ATTRACT_FADE_IN_DONE_FLAG].wrapping_add(1);
        }
    }

    pub(super) fn attract_draw_zelda(&mut self) {
        let start = self.ram[ATTRACT_OAM_IDX] as usize;
        self.set_oam_plain(64 + start, 0x60, self.ram[ATTRACT_X_BASE], 0x28, 0x29, 2);
        self.set_oam_plain(
            65 + start,
            0x60,
            self.ram[ATTRACT_X_BASE].wrapping_add(10),
            0x2a,
            0x29,
            2,
        );
        self.ram[ATTRACT_OAM_IDX] = self.ram[ATTRACT_OAM_IDX].wrapping_add(2);
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
        let start = self.ram[ATTRACT_OAM_IDX] as usize;
        self.ram[ATTRACT_OAM_IDX] = self.ram[ATTRACT_OAM_IDX]
            .wrapping_add(n as u8)
            .wrapping_add(1);
        for i in (0..=n).rev() {
            self.set_oam_plain(
                64 + start + (n - i),
                self.ram[ATTRACT_X_BASE].wrapping_add(xp[i]),
                self.ram[ATTRACT_Y_BASE].wrapping_add(yp[i]),
                cp[i],
                fp[i],
                ep[i],
            );
        }
    }

    pub(super) fn attract_zelda_prison_draw_a(&mut self) {
        let start = self.ram[ATTRACT_OAM_IDX] as usize;
        let ext = if self.ram[ATTRACT_X_BASE_HI] != 0 {
            3
        } else {
            2
        };
        let j = (self.ram[ATTRACT_ANIM_STEP_COUNTER] >> 3) & 1;
        self.set_oam_plain(
            64 + start,
            self.ram[ATTRACT_X_BASE],
            self.ram[ATTRACT_Y_BASE].wrapping_add(j),
            6,
            0x3d,
            ext,
        );
        self.set_oam_plain(
            65 + start,
            self.ram[ATTRACT_X_BASE],
            self.ram[ATTRACT_Y_BASE].wrapping_add(10),
            if j != 0 { 10 } else { 8 },
            0x3d,
            ext,
        );
        self.ram[ATTRACT_OAM_IDX] = self.ram[ATTRACT_OAM_IDX].wrapping_add(2);
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
        if self.ram[ATTRACT_PRISON_SOLDIER_X_LO] == 0 {
            self.ram[ATTRACT_SCENE_SUBSTEP] = self.ram[ATTRACT_SCENE_SUBSTEP].wrapping_add(1);
        }
        if self.ram[FRAME_COUNTER] & 1 != 0 {
            let vram = read_le_u16(&self.ram, ATTRACT_VRAM_DST).wrapping_sub(1);
            write_le_u16(&mut self.ram, ATTRACT_VRAM_DST, vram);
        }
        self.ram[ATTRACT_X_BASE] = 0x58;
        self.ram[ATTRACT_Y_BASE] = self.ram[ATTRACT_PRISON_ZELDA_Y_BASE];
        self.attract_draw_sprite_set2_slice(&OAMS0);
        write_le_u16(&mut self.ram, ATTRACT_VAR7, 0xf8d9);
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
        let Some(k) = (if self.ram[ATTRACT_SCENE_TIMER] < 0x80 {
            self.attract_show_timed_text_message();
            if read_le_u16(&self.ram, OAM_PRIORITY_VALUE) != 0 {
                Some(4)
            } else {
                self.attract_zelda_prison_case1_step()
            }
        } else {
            self.attract_zelda_prison_case1_step()
        }) else {
            return;
        };

        if self.ram[FRAME_COUNTER] & 1 != 0 {
            let vram = read_le_u16(&self.ram, ATTRACT_VRAM_DST).wrapping_sub(1);
            write_le_u16(&mut self.ram, ATTRACT_VRAM_DST, vram);
        }
        self.ram[ATTRACT_X_BASE] = 0x58;
        self.ram[ATTRACT_Y_BASE] = self.ram[ATTRACT_PRISON_ZELDA_Y_BASE];
        self.attract_draw_sprite_set2_slice(&OAMS1[k * 6..k * 6 + 6]);
    }

    fn attract_zelda_prison_case1_step(&mut self) -> Option<usize> {
        if self.ram[ATTRACT_PRISON_ZELDA_Y_BASE] != 0x6e {
            self.ram[ATTRACT_PRISON_ZELDA_Y_BASE] =
                self.ram[ATTRACT_PRISON_ZELDA_Y_BASE].wrapping_sub(1);
            return Some(0);
        }

        if self.ram[ATTRACT_SCENE_TIMER] < 31 && self.ram[ATTRACT_SCENE_TIMER] & 1 == 0 {
            self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
        }
        self.ram[ATTRACT_SCENE_TIMER] = self.ram[ATTRACT_SCENE_TIMER].wrapping_sub(1);
        if self.ram[ATTRACT_SCENE_TIMER] == 0 {
            self.ram[ATTRACT_SEQUENCE] = self.ram[ATTRACT_SEQUENCE].wrapping_add(1);
            self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_sub(2);
            return None;
        }

        Some(match self.ram[ATTRACT_SCENE_TIMER] {
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
        self.ram[SPRITE_Z + k] = 0;
        self.sprite_get_16_bit_coords(k);
        self.ram[SPRITE_D + k] = dir;
        self.ram[SPRITE_HEAD_DIR + k] = dir;
        self.ram[SPRITE_GRAPHICS + k] = SIMULATE_SOLDIER_GFX[dir as usize].wrapping_add(gfx);
        self.ram[SPRITE_FLAGS3 + k] = 16;
        self.ram[SPRITE_OBJ_PRIO + k] = 0;
        self.ram[SPRITE_OAM_FLAGS + k] = flags | 0x30;
        self.ram[SPRITE_TYPE + k] = if flags == 9 { 0x41 } else { 0x43 };
        self.ram[SPRITE_FLAGS2 + k] = 7;
        let oam_idx = k * 8;
        write_le_u16(&mut self.ram, OAM_CUR_PTR, (OAM_BUF + oam_idx * 4) as u16);
        write_le_u16(
            &mut self.ram,
            OAM_EXT_CUR_PTR,
            (BYTEWISE_EXTENDED_OAM + oam_idx) as u16,
        );
        self.guard_handle_all_animation(k);
    }

    fn sprite_get_16_bit_coords(&mut self, k: usize) {
        let x = self.ram[SPRITE_X_LO + k] as u16 | ((self.ram[SPRITE_X_HI + k] as u16) << 8);
        let y = self.ram[SPRITE_Y_LO + k] as u16 | ((self.ram[SPRITE_Y_HI + k] as u16) << 8);
        write_le_u16(&mut self.ram, CUR_SPRITE_X, x);
        write_le_u16(&mut self.ram, CUR_SPRITE_Y, y);
    }

    pub(super) fn sprite_draw_shadow_custom_attract(
        &mut self,
        k: usize,
        poc: (u16, u16, u8),
        y_offset: u8,
    ) {
        let sprite_y = self.ram[SPRITE_Y_LO + k] as u16 | ((self.ram[SPRITE_Y_HI + k] as u16) << 8);
        let y = sprite_y
            .wrapping_add(y_offset as u16)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        if y.wrapping_add(0x10) >= 0x100 {
            return;
        }
        let oam_offs = (self.ram[SPRITE_FLAGS2 + k] & 0x1f) as usize;
        self.set_guard_oam(oam_offs, poc.0, y, 0x6c, (poc.2 & 0x30) | 8, 2);
    }

    fn set_guard_oam(&mut self, offset: usize, x: u16, y: u16, charnum: u8, flags: u8, big: u8) {
        let oam_cur = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let index = (oam_cur - OAM_BUF) / 4 + offset;
        self.set_oam_helper0_index(index, x, y, charnum, flags, big);
    }

    pub(super) fn attract_dramatize_polka_dots(&mut self) {
        if self.ram[FRAME_COUNTER] & 3 == 0 {
            self.ram[BG1VOFS_COPY] = self.ram[BG1VOFS_COPY].wrapping_add(1);
            self.ram[BG1HOFS_COPY] = self.ram[BG1HOFS_COPY].wrapping_add(1);
            self.ram[BG2VOFS_COPY] = self.ram[BG2VOFS_COPY].wrapping_add(1);
            self.ram[BG2HOFS_COPY] = self.ram[BG2HOFS_COPY].wrapping_sub(1);
        }

        if self.ram[ATTRACT_LEGEND_FLAG] != 0 {
            self.attract_build_next_image_tile_map();
            self.ram[ATTRACT_LEGEND_FLAG] = 0;
            self.ram[ATTRACT_NEXT_LEGEND_GFX] = self.ram[ATTRACT_NEXT_LEGEND_GFX].wrapping_add(2);
        }

        self.ram[JOYPAD1L_LAST] = 0;
        self.ram[FILTERED_JOYPAD_L] = 0;
        self.ram[FILTERED_JOYPAD_H] = 0;
        self.RenderText();

        let legend_ctr = read_le_u16(&self.ram, ATTRACT_LEGEND_CTR).wrapping_sub(1);
        write_le_u16(&mut self.ram, ATTRACT_LEGEND_CTR, legend_ctr);
        if legend_ctr == 0 {
            self.ram[ATTRACT_SEQUENCE] = self.ram[ATTRACT_SEQUENCE].wrapping_add(1);
            self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_sub(3);
        } else if legend_ctr < 0x18 && legend_ctr & 1 != 0 {
            self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
        }
    }

    pub(super) fn attract_build_next_image_tile_map(&mut self) {
        let image = self.ram[ATTRACT_NEXT_LEGEND_GFX] >> 1;
        let data = match image {
            0 => &K_ATTRACT_LEGEND_GRAPHICS_0[..],
            1 => &K_ATTRACT_LEGEND_GRAPHICS_1[..],
            2 => &K_ATTRACT_LEGEND_GRAPHICS_2[..],
            3 => &K_ATTRACT_LEGEND_GRAPHICS_3[..],
            _ => &[],
        };
        let len = data
            .len()
            .min(self.ram.len().saturating_sub(VRAM_UPLOAD_DATA));
        self.copy_to_ram(VRAM_UPLOAD_DATA, &data[..len]);
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn attract_build_backgrounds(&mut self) {
        const TAB0: [u16; 16] = [
            0x01a0, 0x09a6, 0x89a5, 0x01a0, 0x09a5, 0x01a0, 0x01a0, 0x89a6, 0x49a5, 0x01a0, 0x01a0,
            0x49a5, 0x01a0, 0x89a5, 0xc9a5, 0x01a0,
        ];
        const TAB1: [u16; 4] = [0x09a1, 0x09a2, 0x09a3, 0x09a4];

        self.ram[BGMODE_COPY] = 9;
        self.ram[TM_COPY] = 0x17;
        self.ram[TS_COPY] = 0;
        self.zelda_ppu_write(0x2107, 0x10);
        self.zelda_ppu_write(0x2108, 0x00);

        let mut k = 0usize;
        let mut p = 0usize;
        loop {
            let mut j = k & 3;
            loop {
                write_le_u16(&mut self.ram, VRAM_UPLOAD_DATA + 4 + k * 2, TAB0[p + j]);
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
                write_le_u16(&mut self.ram, VRAM_UPLOAD_DATA + 4 + k * 2, TAB1[p + j]);
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
        write_le_u16(&mut self.ram, ATTRACT_VRAM_DST, 0);
    }

    pub(super) fn attract_trigger_bgdma(&mut self, dstv: usize) {
        for row in 0..8 {
            let dst = dstv + row * 0x80;
            for i in 0..0x80 {
                self.ppu.vram[dst + i] = read_le_u16(&self.ram, VRAM_UPLOAD_DATA + 4 + i * 2);
            }
        }
    }
}
