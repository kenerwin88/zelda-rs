use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const BIRD_TRAVEL_DESTINATION_SLOTS: usize = 16;
const BIRD_TRAVEL_STATUS_SLOTS: usize = 16;
const OVERWORLD_EVENT_INFO_SCREENS: usize = 160;
const OVERWORLD_CONFIG_SCREENS: usize = 160;
const ROOM_BOUND_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverworldMap16SourcePage {
    Main,
    Overlay,
}

impl OverworldMap16SourcePage {
    fn base_address(self) -> usize {
        match self {
            Self::Main => 0x2000,
            Self::Overlay => 0x4000,
        }
    }
}

pub(crate) struct OverworldMap16Decode<'a> {
    ram: &'a [u8],
}

impl<'a> OverworldMap16Decode<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn source_byte(&self, index: usize) -> u8 {
        ram_byte(self.ram, OVERWORLD_MAP16_DECODE_SRC + index)
    }

    pub(crate) fn source_word(&self, index: usize) -> u16 {
        read_le_u16(self.ram, OVERWORLD_MAP16_DECODE_SRC + index)
    }

    pub(crate) fn source_page_word(&self, page: OverworldMap16SourcePage, offset: usize) -> u16 {
        read_le_u16(self.ram, page.base_address() + offset)
    }

    pub(crate) fn decode_last(&self) -> u16 {
        read_le_u16(self.ram, MAP16_DECODE_LAST)
    }

    pub(crate) fn decode_quad(&self, idx: usize) -> (u16, u16, u16, u16) {
        (
            read_le_u16(self.ram, MAP16_DECODE_0 + idx),
            read_le_u16(self.ram, MAP16_DECODE_1 + idx),
            read_le_u16(self.ram, MAP16_DECODE_2 + idx),
            read_le_u16(self.ram, MAP16_DECODE_3 + idx),
        )
    }

    pub(crate) fn decode_block_byte(&self, base: usize, index: usize) -> u8 {
        ram_byte(self.ram, base + index)
    }
}

pub(crate) struct NativeOverworldMap16DecodeBridgeMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldMap16DecodeBridgeMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn copy_source_from(&mut self, data: &[u8]) {
        self.ram[OVERWORLD_MAP16_DECODE_SRC..OVERWORLD_MAP16_DECODE_SRC + data.len()]
            .copy_from_slice(data);
    }

    pub(crate) fn copy_scratch_to_source_words_high(&mut self, len: usize) {
        for i in 0..len {
            self.ram[OVERWORLD_MAP16_DECODE_SRC + 1 + i * 2] =
                self.ram[OVERWORLD_DECOMP_BUFFER + i];
        }
    }

    pub(crate) fn copy_scratch_to_source_words_low(&mut self, len: usize) {
        for i in 0..len {
            self.ram[OVERWORLD_MAP16_DECODE_SRC + i * 2] = self.ram[OVERWORLD_DECOMP_BUFFER + i];
        }
    }

    pub(crate) fn write_decompressed_byte(&mut self, dst: usize, value: u8) {
        self.ram[dst] = value;
    }

    pub(crate) fn copy_decompressed_byte(&mut self, dst_org: usize, dst: usize, offset: usize) {
        self.ram[dst] = self.ram[dst_org + offset];
    }

    pub(crate) fn decomp_scratch_byte_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.ram[OVERWORLD_DECOMP_BUFFER + index]
    }

    pub(crate) fn decomp_scratch_slice_mut(&mut self) -> &mut [u8] {
        &mut self.ram[OVERWORLD_DECOMP_BUFFER..]
    }

    pub(crate) fn decode_block_fill(&mut self, dst: usize, table: &[u8], x: usize) {
        self.ram[dst] = table[x];
        self.ram[dst + 2] = table[x + 1];
        self.ram[dst + 4] = table[x + 2];
        self.ram[dst + 6] = table[x + 3];
        let packed0 = table[x + 4];
        let packed1 = table[x + 5];
        self.ram[dst + 1] = packed0 >> 4;
        self.ram[dst + 3] = packed0 & 0x0f;
        self.ram[dst + 5] = packed1 >> 4;
        self.ram[dst + 7] = packed1 & 0x0f;
    }

    pub(crate) fn set_decode_last(&mut self, value: u16) {
        write_le_u16(self.ram, MAP16_DECODE_LAST, value);
    }

    pub(crate) fn set_decode_tmp(&mut self, value: u16) {
        write_le_u16(self.ram, MAP16_DECODE_WORK_WORD, value);
    }

    pub(crate) fn write_decoded_map32_to_bg2_tilemap(&mut self, dst: usize, idx: usize) {
        let v0 = read_le_u16(self.ram, MAP16_DECODE_0 + idx);
        let v1 = read_le_u16(self.ram, MAP16_DECODE_1 + idx);
        let v2 = read_le_u16(self.ram, MAP16_DECODE_2 + idx);
        let v3 = read_le_u16(self.ram, MAP16_DECODE_3 + idx);
        write_le_u16(self.ram, dst, v0);
        write_le_u16(self.ram, dst + 128, v2);
        write_le_u16(self.ram, dst + 2, v1);
        write_le_u16(self.ram, dst + 130, v3);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorldLocationState {
    pub(crate) dungeon_room: u16,
    pub(crate) overworld_screen: u16,
    pub(crate) indoor_flag: u8,
}

impl WorldLocationState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            dungeon_room: read_le_u16(ram, DUNGEON_ROOM),
            overworld_screen: read_le_u16(ram, OVERWORLD_SCREEN_INDEX),
            indoor_flag: ram_byte(ram, PLAYER_IS_INDOORS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNGEON_ROOM, self.dungeon_room);
        write_le_u16(ram, OVERWORLD_SCREEN_INDEX, self.overworld_screen);
        ram[PLAYER_IS_INDOORS] = self.indoor_flag;
    }

    pub(crate) fn dungeon_room_index(&self) -> u8 {
        self.dungeon_room as u8
    }

    pub(crate) fn overworld_screen_index(&self) -> u8 {
        self.overworld_screen as u8
    }

    pub(crate) fn is_indoors(&self) -> bool {
        self.indoor_flag != 0
    }

    pub(crate) fn is_outdoors(&self) -> bool {
        !self.is_indoors()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorldScrollState {
    pub(crate) bg1_x: u16,
    pub(crate) bg1_y: u16,
    pub(crate) bg2_x: u16,
    pub(crate) bg2_y: u16,
    pub(crate) bg1_x_offset: u16,
    pub(crate) bg1_y_offset: u16,
    pub(crate) camera_x: u16,
    pub(crate) camera_y: u16,
    pub(crate) overworld_offset_base_x: u16,
    pub(crate) overworld_offset_base_y: u16,
    pub(crate) overworld_offset_mask_x: u16,
    pub(crate) overworld_offset_mask_y: u16,
    pub(crate) scroll_x_start: u16,
    pub(crate) scroll_x_end: u16,
    pub(crate) scroll_y_end: u16,
}

impl WorldScrollState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            bg1_x: read_le_u16(ram, BG1_X_SCROLL),
            bg1_y: read_le_u16(ram, BG1_Y_SCROLL),
            bg2_x: read_le_u16(ram, BG2_X_SCROLL),
            bg2_y: read_le_u16(ram, BG2_Y_SCROLL),
            bg1_x_offset: read_le_u16(ram, BG1_X_OFFSET),
            bg1_y_offset: read_le_u16(ram, BG1_Y_OFFSET),
            camera_x: read_le_u16(ram, CAMERA_X),
            camera_y: read_le_u16(ram, CAMERA_Y),
            overworld_offset_base_x: read_le_u16(ram, OVERWORLD_OFFSET_BASE_X),
            overworld_offset_base_y: read_le_u16(ram, OVERWORLD_OFFSET_BASE_Y),
            overworld_offset_mask_x: read_le_u16(ram, OVERWORLD_OFFSET_MASK_X),
            overworld_offset_mask_y: read_le_u16(ram, OVERWORLD_OFFSET_MASK_Y),
            scroll_x_start: read_le_u16(ram, OVERWORLD_SCROLL_X_START),
            scroll_x_end: read_le_u16(ram, OVERWORLD_SCROLL_X_END),
            scroll_y_end: read_le_u16(ram, OVERWORLD_SCROLL_Y_END),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, BG1_X_SCROLL, self.bg1_x);
        write_le_u16(ram, BG1_Y_SCROLL, self.bg1_y);
        write_le_u16(ram, BG2_X_SCROLL, self.bg2_x);
        write_le_u16(ram, BG2_Y_SCROLL, self.bg2_y);
        write_le_u16(ram, BG1_X_OFFSET, self.bg1_x_offset);
        write_le_u16(ram, BG1_Y_OFFSET, self.bg1_y_offset);
        write_le_u16(ram, CAMERA_X, self.camera_x);
        write_le_u16(ram, CAMERA_Y, self.camera_y);
        write_le_u16(ram, OVERWORLD_OFFSET_BASE_X, self.overworld_offset_base_x);
        write_le_u16(ram, OVERWORLD_OFFSET_BASE_Y, self.overworld_offset_base_y);
        write_le_u16(ram, OVERWORLD_OFFSET_MASK_X, self.overworld_offset_mask_x);
        write_le_u16(ram, OVERWORLD_OFFSET_MASK_Y, self.overworld_offset_mask_y);
        write_le_u16(ram, OVERWORLD_SCROLL_X_START, self.scroll_x_start);
        write_le_u16(ram, OVERWORLD_SCROLL_X_END, self.scroll_x_end);
        write_le_u16(ram, OVERWORLD_SCROLL_Y_END, self.scroll_y_end);
    }

    pub(crate) fn bg1_x(&self) -> u16 {
        self.bg1_x
    }

    pub(crate) fn bg1_x_low(&self) -> u8 {
        self.bg1_x as u8
    }

    pub(crate) fn bg1_y(&self) -> u16 {
        self.bg1_y
    }

    pub(crate) fn bg1_y_low(&self) -> u8 {
        self.bg1_y as u8
    }

    pub(crate) fn bg2_x(&self) -> u16 {
        self.bg2_x
    }

    pub(crate) fn bg2_x_low(&self) -> u8 {
        self.bg2_x as u8
    }

    pub(crate) fn bg2_y(&self) -> u16 {
        self.bg2_y
    }

    pub(crate) fn bg2_y_low(&self) -> u8 {
        self.bg2_y as u8
    }

    pub(crate) fn bg1_x_offset(&self) -> u16 {
        self.bg1_x_offset
    }

    pub(crate) fn bg1_y_offset(&self) -> u16 {
        self.bg1_y_offset
    }

    pub(crate) fn bg1_offset_mask(&self) -> u16 {
        self.bg1_x_offset | self.bg1_y_offset
    }

    pub(crate) fn camera_x(&self) -> u16 {
        self.camera_x
    }

    pub(crate) fn camera_y(&self) -> u16 {
        self.camera_y
    }

    pub(crate) fn overworld_offset_base_x(&self) -> u16 {
        self.overworld_offset_base_x
    }

    pub(crate) fn overworld_offset_base_y(&self) -> u16 {
        self.overworld_offset_base_y
    }

    pub(crate) fn overworld_offset_mask_x(&self) -> u16 {
        self.overworld_offset_mask_x
    }

    pub(crate) fn overworld_offset_mask_y(&self) -> u16 {
        self.overworld_offset_mask_y
    }

    pub(crate) fn scroll_x_start(&self) -> u16 {
        self.scroll_x_start
    }

    pub(crate) fn scroll_x_end(&self) -> u16 {
        self.scroll_x_end
    }

    pub(crate) fn scroll_y_end(&self) -> u16 {
        self.scroll_y_end
    }
}

const AUX_BG_SUBSET_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorldPaletteThemeState {
    pub(crate) last_light_vs_dark_world: u8,
    pub(crate) aux_bg_subset: [u8; AUX_BG_SUBSET_COUNT],
    pub(crate) overworld_palette_aux1_hi: u8,
    pub(crate) overworld_palette_mode: u8,
    pub(crate) palette_main_indoors: u8,
    pub(crate) palette_main_indoors_copy: u8,
    pub(crate) palette_swap_flag: u8,
    pub(crate) palette_sp0l: u8,
    pub(crate) palette_sp5l: u8,
    pub(crate) palette_sp6l: u8,
    pub(crate) palette_sp6r_indoors: u8,
    pub(crate) hud_palette: u8,
    pub(crate) overworld_palette_aux2_hi: u8,
    pub(crate) overworld_palette_aux3_lo: u8,
    pub(crate) misc_sprites_graphics_index: u8,
    pub(crate) overworld_tile_theme_index: u8,
    pub(crate) main_tile_theme_index: u8,
    pub(crate) aux_tile_theme_index: u8,
    pub(crate) special_exit_overworld_tile_theme_index: u8,
    pub(crate) special_exit_main_tile_theme_index: u8,
    pub(crate) special_exit_aux_tile_theme_index: u8,
    pub(crate) exit_overworld_tile_theme_index: u8,
    pub(crate) exit_main_tile_theme_index: u8,
    pub(crate) exit_aux_tile_theme_index: u8,
}

impl WorldPaletteThemeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut aux_bg_subset = [0; AUX_BG_SUBSET_COUNT];
        for (index, subset) in aux_bg_subset.iter_mut().enumerate() {
            *subset = ram_byte(ram, AUX_BG_SUBSET_0 + index);
        }
        Self {
            last_light_vs_dark_world: ram_byte(ram, LAST_LIGHT_VS_DARK_WORLD),
            aux_bg_subset,
            overworld_palette_aux1_hi: ram_byte(ram, OVERWORLD_PALETTE_AUX1_BP2TO4_HI),
            overworld_palette_mode: ram_byte(ram, OVERWORLD_PALETTE_MODE),
            palette_main_indoors: ram_byte(ram, PALETTE_MAIN_INDOORS),
            palette_main_indoors_copy: ram_byte(ram, PALETTE_MAIN_INDOORS_COPY),
            palette_swap_flag: ram_byte(ram, PALETTE_SWAP_FLAG),
            palette_sp0l: ram_byte(ram, PALETTE_SP0L),
            palette_sp5l: ram_byte(ram, PALETTE_SP5L),
            palette_sp6l: ram_byte(ram, PALETTE_SP6L),
            palette_sp6r_indoors: ram_byte(ram, PALETTE_SP6R_INDOORS),
            hud_palette: ram_byte(ram, HUD_PALETTE),
            overworld_palette_aux2_hi: ram_byte(ram, OVERWORLD_PALETTE_AUX2_BP5TO7_HI),
            overworld_palette_aux3_lo: ram_byte(ram, OVERWORLD_PALETTE_AUX3_BP7_LO),
            misc_sprites_graphics_index: ram_byte(ram, MISC_SPRITES_GRAPHICS_INDEX),
            overworld_tile_theme_index: ram_byte(ram, OVERWORLD_TILE_THEME_INDEX),
            main_tile_theme_index: ram_byte(ram, MAIN_TILE_THEME_INDEX),
            aux_tile_theme_index: ram_byte(ram, AUX_TILE_THEME_INDEX),
            special_exit_overworld_tile_theme_index: ram_byte(
                ram,
                OVERWORLD_SPECIAL_TILE_THEME_INDEX,
            ),
            special_exit_main_tile_theme_index: ram_byte(ram, MAIN_TILE_THEME_INDEX_SPEXIT),
            special_exit_aux_tile_theme_index: ram_byte(ram, AUX_TILE_THEME_INDEX_SPEXIT),
            exit_overworld_tile_theme_index: ram_byte(ram, OVERWORLD_TILE_THEME_INDEX_EXIT),
            exit_main_tile_theme_index: ram_byte(ram, MAIN_TILE_THEME_INDEX_EXIT),
            exit_aux_tile_theme_index: ram_byte(ram, AUX_TILE_THEME_INDEX_EXIT),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[LAST_LIGHT_VS_DARK_WORLD] = self.last_light_vs_dark_world;
        for (index, subset) in self.aux_bg_subset.iter().enumerate() {
            ram[AUX_BG_SUBSET_0 + index] = *subset;
        }
        ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI] = self.overworld_palette_aux1_hi;
        ram[OVERWORLD_PALETTE_MODE] = self.overworld_palette_mode;
        ram[PALETTE_MAIN_INDOORS] = self.palette_main_indoors;
        ram[PALETTE_MAIN_INDOORS_COPY] = self.palette_main_indoors_copy;
        ram[PALETTE_SWAP_FLAG] = self.palette_swap_flag;
        ram[PALETTE_SP0L] = self.palette_sp0l;
        ram[PALETTE_SP5L] = self.palette_sp5l;
        ram[PALETTE_SP6L] = self.palette_sp6l;
        ram[PALETTE_SP6R_INDOORS] = self.palette_sp6r_indoors;
        ram[HUD_PALETTE] = self.hud_palette;
        ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = self.overworld_palette_aux2_hi;
        ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = self.overworld_palette_aux3_lo;
        ram[MISC_SPRITES_GRAPHICS_INDEX] = self.misc_sprites_graphics_index;
        ram[OVERWORLD_TILE_THEME_INDEX] = self.overworld_tile_theme_index;
        ram[MAIN_TILE_THEME_INDEX] = self.main_tile_theme_index;
        ram[AUX_TILE_THEME_INDEX] = self.aux_tile_theme_index;
        ram[OVERWORLD_SPECIAL_TILE_THEME_INDEX] = self.special_exit_overworld_tile_theme_index;
        ram[MAIN_TILE_THEME_INDEX_SPEXIT] = self.special_exit_main_tile_theme_index;
        ram[AUX_TILE_THEME_INDEX_SPEXIT] = self.special_exit_aux_tile_theme_index;
        ram[OVERWORLD_TILE_THEME_INDEX_EXIT] = self.exit_overworld_tile_theme_index;
        ram[MAIN_TILE_THEME_INDEX_EXIT] = self.exit_main_tile_theme_index;
        ram[AUX_TILE_THEME_INDEX_EXIT] = self.exit_aux_tile_theme_index;
    }

    pub(crate) fn aux_bg_subset(&self, index: usize) -> u8 {
        self.aux_bg_subset.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn last_light_vs_dark_world(&self) -> u8 {
        self.last_light_vs_dark_world
    }

    pub(crate) fn overworld_palette_aux1_hi(&self) -> u8 {
        self.overworld_palette_aux1_hi
    }

    pub(crate) fn overworld_palette_mode(&self) -> u8 {
        self.overworld_palette_mode
    }

    pub(crate) fn palette_main_indoors(&self) -> u8 {
        self.palette_main_indoors
    }

    pub(crate) fn palette_main_indoors_copy(&self) -> u8 {
        self.palette_main_indoors_copy
    }

    pub(crate) fn palette_swap_flag(&self) -> u8 {
        self.palette_swap_flag
    }

    pub(crate) fn palette_sp0l(&self) -> u8 {
        self.palette_sp0l
    }

    pub(crate) fn palette_sp5l(&self) -> u8 {
        self.palette_sp5l
    }

    pub(crate) fn palette_sp6l(&self) -> u8 {
        self.palette_sp6l
    }

    pub(crate) fn palette_sp6r_indoors(&self) -> u8 {
        self.palette_sp6r_indoors
    }

    pub(crate) fn hud_palette(&self) -> u8 {
        self.hud_palette
    }

    pub(crate) fn overworld_palette_aux2_hi(&self) -> u8 {
        self.overworld_palette_aux2_hi
    }

    pub(crate) fn overworld_palette_aux3_lo(&self) -> u8 {
        self.overworld_palette_aux3_lo
    }

    pub(crate) fn misc_sprites_graphics_index(&self) -> u8 {
        self.misc_sprites_graphics_index
    }

    pub(crate) fn main_tile_theme_index(&self) -> u8 {
        self.main_tile_theme_index
    }

    pub(crate) fn aux_tile_theme_index(&self) -> u8 {
        self.aux_tile_theme_index
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WeatherVaneState {
    pub(crate) countdown: u16,
    pub(crate) music_latch: u8,
    pub(crate) source_slot: u8,
    pub(crate) oam_offset: u8,
}

impl WeatherVaneState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            countdown: read_le_u16(ram, WEATHERVANE_COUNTDOWN),
            music_latch: ram_byte(ram, WEATHERVANE_MUSIC_LATCH),
            source_slot: ram_byte(ram, WEATHERVANE_SOURCE_SLOT),
            oam_offset: ram_byte(ram, WEATHERVANE_OAM_OFFSET),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, WEATHERVANE_COUNTDOWN, self.countdown);
        ram[WEATHERVANE_MUSIC_LATCH] = self.music_latch;
        ram[WEATHERVANE_SOURCE_SLOT] = self.source_slot;
        ram[WEATHERVANE_OAM_OFFSET] = self.oam_offset;
    }

    pub(crate) fn tick_countdown(&mut self) -> u16 {
        self.countdown = self.countdown.wrapping_sub(1);
        self.countdown
    }

    pub(crate) fn reset_oam_offset(&mut self) {
        self.oam_offset = 0;
    }

    pub(crate) fn advance_oam_offset(&mut self, value: u8) {
        self.oam_offset = self.oam_offset.wrapping_add(value);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BirdTravelStatusesState {
    slots: [u8; BIRD_TRAVEL_STATUS_SLOTS],
}

impl BirdTravelStatusesState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut slots = [0; BIRD_TRAVEL_STATUS_SLOTS];
        for (slot, status) in slots.iter_mut().enumerate() {
            *status = ram_byte(ram, BIRD_TRAVEL_STATUS + slot);
        }
        Self { slots }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (slot, status) in self.slots.iter().enumerate() {
            ram[BIRD_TRAVEL_STATUS + slot] = *status;
        }
    }

    pub(crate) fn status(&self, slot: usize) -> u8 {
        self.slots.get(slot).copied().unwrap_or_default()
    }

    pub(crate) fn set_status(&mut self, slot: usize, value: u8) {
        self.slots[slot] = value;
    }

    pub(crate) fn clear_status(&mut self, slot: usize) {
        self.set_status(slot, 0);
    }

    pub(crate) fn increment_status(&mut self, slot: usize) {
        self.set_status(slot, self.status(slot).wrapping_add(1));
    }

    pub(crate) fn status_word(&self) -> u16 {
        u16::from(self.status(0)) | (u16::from(self.status(1)) << 8)
    }

    pub(crate) fn set_status_word(&mut self, value: u16) {
        self.set_status(0, value as u8);
        self.set_status(1, (value >> 8) as u8);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldEventInfoState {
    info: Vec<u8>,
}

impl OverworldEventInfoState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut info = vec![0; OVERWORLD_EVENT_INFO_SCREENS];
        for (screen, value) in info.iter_mut().enumerate() {
            *value = ram_byte(ram, OVERWORLD_EVENT_INFO + screen);
        }
        Self { info }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (screen, value) in self.info.iter().enumerate() {
            ram[OVERWORLD_EVENT_INFO + screen] = *value;
        }
    }

    pub(crate) fn event_info(&self, screen: usize) -> u8 {
        self.info.get(screen).copied().unwrap_or_default()
    }

    pub(crate) fn has_event_bits(&self, screen: usize, mask: u8) -> bool {
        self.event_info(screen) & mask != 0
    }

    pub(crate) fn set_event_info(&mut self, screen: usize, value: u8) {
        self.info[screen] = value;
    }

    pub(crate) fn set_event_bits(&mut self, screen: usize, mask: u8) {
        self.info[screen] |= mask;
    }

    pub(crate) fn clear_event_bits(&mut self, screen: usize, mask: u8) {
        self.info[screen] &= !mask;
    }
}

impl Default for OverworldEventInfoState {
    fn default() -> Self {
        Self {
            info: vec![0; OVERWORLD_EVENT_INFO_SCREENS],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldConfigTableState {
    music: Vec<u8>,
    sprite_palette: Vec<u8>,
    sprite_graphics: Vec<u8>,
}

impl OverworldConfigTableState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut music = vec![0; OVERWORLD_CONFIG_SCREENS];
        let mut sprite_palette = vec![0; OVERWORLD_CONFIG_SCREENS];
        let mut sprite_graphics = vec![0; OVERWORLD_CONFIG_SCREENS];
        for screen in 0..OVERWORLD_CONFIG_SCREENS {
            music[screen] = ram_byte(ram, OVERWORLD_MUSIC_TABLE + screen);
            sprite_palette[screen] = ram_byte(ram, OVERWORLD_SPRITE_PALETTE_TABLE + screen);
            sprite_graphics[screen] = ram_byte(ram, OVERWORLD_SPRITE_GFX_TABLE + screen);
        }
        Self {
            music,
            sprite_palette,
            sprite_graphics,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for screen in 0..OVERWORLD_CONFIG_SCREENS {
            ram[OVERWORLD_MUSIC_TABLE + screen] = self.music(screen);
            ram[OVERWORLD_SPRITE_PALETTE_TABLE + screen] = self.sprite_palette(screen);
            ram[OVERWORLD_SPRITE_GFX_TABLE + screen] = self.sprite_graphics(screen);
        }
    }

    pub(crate) fn music(&self, screen: usize) -> u8 {
        self.music.get(screen).copied().unwrap_or_default()
    }

    pub(crate) fn sprite_palette(&self, screen: usize) -> u8 {
        self.sprite_palette.get(screen).copied().unwrap_or_default()
    }

    pub(crate) fn sprite_graphics(&self, screen: usize) -> u8 {
        self.sprite_graphics
            .get(screen)
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn set_music(&mut self, screen: usize, value: u8) {
        self.music[screen] = value;
    }

    pub(crate) fn copy_music_primary(&mut self, data: &[u8]) {
        self.music[..64].copy_from_slice(&data[..64]);
    }

    pub(crate) fn copy_music_secondary(&mut self, data: &[u8]) {
        self.music[64..160].copy_from_slice(&data[..96]);
    }
}

impl Default for OverworldConfigTableState {
    fn default() -> Self {
        Self {
            music: vec![0; OVERWORLD_CONFIG_SCREENS],
            sprite_palette: vec![0; OVERWORLD_CONFIG_SCREENS],
            sprite_graphics: vec![0; OVERWORLD_CONFIG_SCREENS],
        }
    }
}

pub(crate) struct OverworldConfigTableView<'a> {
    config_table: &'a OverworldConfigTableState,
    current_screen: usize,
}

impl<'a> OverworldConfigTableView<'a> {
    pub(crate) fn new(config_table: &'a OverworldConfigTableState, current_screen: usize) -> Self {
        Self {
            config_table,
            current_screen,
        }
    }

    pub(crate) fn music(&self, screen: usize) -> u8 {
        self.config_table.music(screen)
    }

    pub(crate) fn current_music(&self) -> u8 {
        self.music(self.current_screen)
    }

    pub(crate) fn sprite_palette(&self, screen: usize) -> u8 {
        self.config_table.sprite_palette(screen)
    }

    pub(crate) fn sprite_graphics(&self, screen: usize) -> u8 {
        self.config_table.sprite_graphics(screen)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldMapUiState {
    pub(crate) map_state: u16,
    pub(crate) map_flags: u8,
    pub(crate) bird_travel_statuses: BirdTravelStatusesState,
}

impl OverworldMapUiState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            map_state: read_le_u16(ram, OVERWORLD_MAP_STATE),
            map_flags: ram_byte(ram, OVERWORLD_MAP_FLAGS),
            bird_travel_statuses: BirdTravelStatusesState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, OVERWORLD_MAP_STATE, self.map_state);
        ram[OVERWORLD_MAP_FLAGS] = self.map_flags;
        self.bird_travel_statuses.write_to_ram(ram);
    }

    pub(crate) fn map_state(&self) -> u8 {
        self.map_state as u8
    }

    pub(crate) fn map_state_word(&self) -> u16 {
        self.map_state
    }

    pub(crate) fn birdtravel_status(&self) -> u8 {
        self.bird_travel_statuses.status(0)
    }

    pub(crate) fn birdtravel_status_word(&self) -> u16 {
        self.bird_travel_statuses.status_word()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BirdTravelDestinationState {
    pub(crate) x: u16,
    pub(crate) y: u16,
}

impl BirdTravelDestinationState {
    pub(crate) fn is_empty(&self) -> bool {
        self.x == 0 && self.y == 0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BirdTravelDestinationsState {
    slots: [BirdTravelDestinationState; BIRD_TRAVEL_DESTINATION_SLOTS],
}

impl BirdTravelDestinationsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut slots = [BirdTravelDestinationState::default(); BIRD_TRAVEL_DESTINATION_SLOTS];
        for (slot, destination) in slots.iter_mut().enumerate() {
            destination.x = u16::from(ram_byte(ram, BIRD_TRAVEL_X_LO + slot))
                | (u16::from(ram_byte(ram, BIRD_TRAVEL_X_HI + slot)) << 8);
            destination.y = u16::from(ram_byte(ram, BIRD_TRAVEL_Y_LO + slot))
                | (u16::from(ram_byte(ram, BIRD_TRAVEL_Y_HI + slot)) << 8);
        }
        Self { slots }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (slot, destination) in self.slots.iter().enumerate() {
            ram[BIRD_TRAVEL_X_LO + slot] = destination.x as u8;
            ram[BIRD_TRAVEL_X_HI + slot] = (destination.x >> 8) as u8;
            ram[BIRD_TRAVEL_Y_LO + slot] = destination.y as u8;
            ram[BIRD_TRAVEL_Y_HI + slot] = (destination.y >> 8) as u8;
        }
    }

    pub(crate) fn destination(&self, slot: usize) -> BirdTravelDestinationState {
        self.slots
            .get(slot)
            .copied()
            .unwrap_or_else(BirdTravelDestinationState::default)
    }

    pub(crate) fn destination_mut(&mut self, slot: usize) -> &mut BirdTravelDestinationState {
        &mut self.slots[slot]
    }

    pub(crate) fn set_destination(&mut self, slot: usize, x: u16, y: u16) {
        *self.destination_mut(slot) = BirdTravelDestinationState { x, y };
    }

    pub(crate) fn clear_destination(&mut self, slot: usize) {
        self.set_destination(slot, 0, 0);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldMapZoomState {
    pub(crate) step_counter: u8,
    pub(crate) timer: u8,
}

impl OverworldMapZoomState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            step_counter: ram_byte(ram, MODE7_ZOOM_STEP_COUNTER),
            timer: ram_byte(ram, TIMER_FOR_MODE7_ZOOM),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[MODE7_ZOOM_STEP_COUNTER] = self.step_counter;
        ram[TIMER_FOR_MODE7_ZOOM] = self.timer;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldScreenSizeState {
    pub(crate) big_area: u16,
    pub(crate) big_area_backup: u8,
    pub(crate) right_bottom_scroll_bound: u16,
}

impl OverworldScreenSizeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            big_area: read_le_u16(ram, OVERWORLD_AREA_IS_BIG),
            big_area_backup: ram_byte(ram, OVERWORLD_AREA_IS_BIG_BACKUP),
            right_bottom_scroll_bound: read_le_u16(ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, OVERWORLD_AREA_IS_BIG, self.big_area);
        ram[OVERWORLD_AREA_IS_BIG_BACKUP] = self.big_area_backup;
        write_le_u16(
            ram,
            OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND,
            self.right_bottom_scroll_bound,
        );
    }

    pub(crate) fn is_big_area_word(&self) -> u16 {
        self.big_area
    }

    pub(crate) fn is_big_area(&self) -> bool {
        self.is_big_area_word() != 0
    }

    pub(crate) fn right_bottom_bound_word(&self) -> u16 {
        self.right_bottom_scroll_bound
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldScrollDeltaState {
    bytes: [u8; 3],
}

impl OverworldScrollDeltaState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            bytes: [
                ram_byte(ram, OVERWORLD_SCROLL_DELTA),
                ram_byte(ram, OVERWORLD_SCROLL_DELTA + 1),
                ram_byte(ram, OVERWORLD_SCROLL_DELTA + 2),
            ],
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[OVERWORLD_SCROLL_DELTA] = self.bytes[0];
        ram[OVERWORLD_SCROLL_DELTA + 1] = self.bytes[1];
        ram[OVERWORLD_SCROLL_DELTA + 2] = self.bytes[2];
    }

    pub(crate) fn vertical_delta_low_byte(&self) -> u8 {
        self.bytes[0]
    }

    pub(crate) fn horizontal_delta_low_byte(&self) -> u8 {
        self.bytes[1]
    }

    pub(crate) fn vertical_delta_word(&self) -> u16 {
        u16::from(self.bytes[0]) | (u16::from(self.bytes[1]) << 8)
    }

    pub(crate) fn horizontal_delta_word(&self) -> u16 {
        u16::from(self.bytes[1]) | (u16::from(self.bytes[2]) << 8)
    }

    pub(crate) fn set_vertical_delta_low_byte(&mut self, value: u8) {
        self.bytes[0] = value;
    }

    pub(crate) fn set_horizontal_delta_low_byte(&mut self, value: u8) {
        self.bytes[1] = value;
    }

    pub(crate) fn set_vertical_delta_word(&mut self, value: u16) {
        self.bytes[0] = value as u8;
        self.bytes[1] = (value >> 8) as u8;
    }

    pub(crate) fn set_horizontal_delta_word(&mut self, value: u16) {
        self.bytes[1] = value as u8;
        self.bytes[2] = (value >> 8) as u8;
    }

    pub(crate) fn clear_vertical_delta_low_byte(&mut self) {
        self.set_vertical_delta_low_byte(0);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OverworldMap16LoadState {
    pub src_off: u16,
    pub dst_off: u16,
    pub y_unit: u16,
}

impl OverworldMap16LoadState {
    fn load_from_ram_at(ram: &[u8], src_off: usize, dst_off: usize, y_unit: usize) -> Self {
        Self {
            src_off: read_le_u16(ram, src_off),
            dst_off: read_le_u16(ram, dst_off),
            y_unit: read_le_u16(ram, y_unit),
        }
    }

    fn write_to_ram_at(&self, ram: &mut [u8], src_off: usize, dst_off: usize, y_unit: usize) {
        write_le_u16(ram, src_off, self.src_off);
        write_le_u16(ram, dst_off, self.dst_off);
        write_le_u16(ram, y_unit, self.y_unit);
    }

    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self::load_from_ram_at(
            ram,
            MAP16_LOAD_SRC_OFF,
            MAP16_LOAD_DST_OFF,
            MAP16_LOAD_Y_UNIT,
        )
    }

    pub(crate) fn load_previous_from_ram(ram: &[u8]) -> Self {
        Self::load_from_ram_at(
            ram,
            MAP16_LOAD_SRC_OFF_PREV,
            MAP16_LOAD_DST_OFF_PREV,
            MAP16_LOAD_Y_UNIT_PREV,
        )
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.write_to_ram_at(
            ram,
            MAP16_LOAD_SRC_OFF,
            MAP16_LOAD_DST_OFF,
            MAP16_LOAD_Y_UNIT,
        );
    }

    pub(crate) fn write_previous_to_ram(&self, ram: &mut [u8]) {
        self.write_to_ram_at(
            ram,
            MAP16_LOAD_SRC_OFF_PREV,
            MAP16_LOAD_DST_OFF_PREV,
            MAP16_LOAD_Y_UNIT_PREV,
        );
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SmallOverworldMap16ScrollBackupState {
    pub src_off: u16,
    pub dst_off: u16,
    pub y_unit: u16,
}

impl SmallOverworldMap16ScrollBackupState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            src_off: read_le_u16(ram, ORANGE_BLUE_BARRIER_STATE),
            dst_off: read_le_u16(ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF),
            y_unit: read_le_u16(ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, ORANGE_BLUE_BARRIER_STATE, self.src_off);
        write_le_u16(ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF, self.dst_off);
        write_le_u16(ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT, self.y_unit);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldMap16State {
    pub(crate) active_load: OverworldMap16LoadState,
    pub(crate) previous_load: OverworldMap16LoadState,
    pub(crate) special_exit_src_off: u16,
    pub(crate) exit_src_off: u16,
    pub(crate) small_scroll_backup: SmallOverworldMap16ScrollBackupState,
}

impl OverworldMap16State {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            active_load: OverworldMap16LoadState::load_from_ram(ram),
            previous_load: OverworldMap16LoadState::load_previous_from_ram(ram),
            special_exit_src_off: read_le_u16(ram, MAP16_LOAD_SRC_OFF_SPEXIT),
            exit_src_off: read_le_u16(ram, MAP16_LOAD_SRC_OFF_EXIT),
            small_scroll_backup: SmallOverworldMap16ScrollBackupState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.active_load.write_to_ram(ram);
        self.previous_load.write_previous_to_ram(ram);
        write_le_u16(ram, MAP16_LOAD_SRC_OFF_SPEXIT, self.special_exit_src_off);
        write_le_u16(ram, MAP16_LOAD_SRC_OFF_EXIT, self.exit_src_off);
        self.small_scroll_backup.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldEntranceState {
    pub(crate) special_entrance_trigger: u8,
    pub(crate) sequence_counter: u8,
}

impl OverworldEntranceState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            special_entrance_trigger: ram_byte(ram, TRIGGER_SPECIAL_ENTRANCE),
            sequence_counter: ram_byte(ram, OVERWORLD_ENTRANCE_SEQUENCE_COUNTER),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[TRIGGER_SPECIAL_ENTRANCE] = self.special_entrance_trigger;
        ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = self.sequence_counter;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldExitState {
    pub(crate) exit_screen: u16,
    pub(crate) special_exit_screen: u16,
}

impl OverworldExitState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            exit_screen: read_le_u16(ram, OVERWORLD_SCREEN_INDEX_EXIT),
            special_exit_screen: read_le_u16(ram, OVERWORLD_SCREEN_INDEX_SPEXIT),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, OVERWORLD_SCREEN_INDEX_EXIT, self.exit_screen);
        write_le_u16(ram, OVERWORLD_SCREEN_INDEX_SPEXIT, self.special_exit_screen);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldTransitionState {
    pub(crate) edge_direction_bits: u16,
    pub(crate) direction_bits: u16,
    pub(crate) direction_enum: u8,
    pub(crate) screen_transition: u16,
    pub(crate) transition_counter: u8,
    pub(crate) countdown: u8,
    pub(crate) previous_direction_bits: u16,
    pub(crate) previous_direction_bits2: u16,
    pub(crate) previous_screen_transition: u8,
}

impl OverworldTransitionState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            edge_direction_bits: read_le_u16(ram, OVERWORLD_SCREEN_TRANS_DIR_BITS),
            direction_bits: read_le_u16(ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2),
            direction_enum: ram_byte(ram, OVERWORLD_TRANSITION_DIR),
            screen_transition: read_le_u16(ram, OVERWORLD_SCREEN_TRANSITION),
            transition_counter: ram_byte(ram, TRANSITION_COUNTER),
            countdown: ram_byte(ram, OW_COUNTDOWN_TRANSITION),
            previous_direction_bits: read_le_u16(ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV),
            previous_direction_bits2: read_le_u16(ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV),
            previous_screen_transition: ram_byte(ram, OVERWORLD_SCREEN_TRANSITION_PREV),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(
            ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS,
            self.edge_direction_bits,
        );
        write_le_u16(ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, self.direction_bits);
        ram[OVERWORLD_TRANSITION_DIR] = self.direction_enum;
        write_le_u16(ram, OVERWORLD_SCREEN_TRANSITION, self.screen_transition);
        ram[TRANSITION_COUNTER] = self.transition_counter;
        ram[OW_COUNTDOWN_TRANSITION] = self.countdown;
        write_le_u16(
            ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV,
            self.previous_direction_bits,
        );
        write_le_u16(
            ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV,
            self.previous_direction_bits2,
        );
        ram[OVERWORLD_SCREEN_TRANSITION_PREV] = self.previous_screen_transition;
    }

    pub(crate) fn edge_direction_bits(&self) -> u8 {
        self.edge_direction_bits as u8
    }

    pub(crate) fn direction_bits(&self) -> u8 {
        self.direction_bits as u8
    }

    pub(crate) fn direction_bits_word(&self) -> u16 {
        self.direction_bits
    }

    pub(crate) fn direction_enum(&self) -> u8 {
        self.direction_enum
    }

    pub(crate) fn has_direction_bits(&self) -> bool {
        self.direction_bits() != 0
    }

    pub(crate) fn screen_transition(&self) -> u8 {
        self.screen_transition as u8
    }

    pub(crate) fn screen_transition_word(&self) -> u16 {
        self.screen_transition
    }

    pub(crate) fn countdown(&self) -> u8 {
        self.countdown
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldState {
    pub(crate) event_info: OverworldEventInfoState,
    pub(crate) config_table: OverworldConfigTableState,
    pub(crate) map_ui: OverworldMapUiState,
    pub(crate) weather_vane: WeatherVaneState,
    pub(crate) bird_travel_destinations: BirdTravelDestinationsState,
    pub(crate) map_zoom: OverworldMapZoomState,
    pub(crate) screen_size: OverworldScreenSizeState,
    pub(crate) scroll_delta: OverworldScrollDeltaState,
    pub(crate) map16: OverworldMap16State,
    pub(crate) entrance: OverworldEntranceState,
    pub(crate) exit: OverworldExitState,
    pub(crate) transition: OverworldTransitionState,
}

impl OverworldState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            event_info: OverworldEventInfoState::load_from_ram(ram),
            config_table: OverworldConfigTableState::load_from_ram(ram),
            map_ui: OverworldMapUiState::load_from_ram(ram),
            weather_vane: WeatherVaneState::load_from_ram(ram),
            bird_travel_destinations: BirdTravelDestinationsState::load_from_ram(ram),
            map_zoom: OverworldMapZoomState::load_from_ram(ram),
            screen_size: OverworldScreenSizeState::load_from_ram(ram),
            scroll_delta: OverworldScrollDeltaState::load_from_ram(ram),
            map16: OverworldMap16State::load_from_ram(ram),
            entrance: OverworldEntranceState::load_from_ram(ram),
            exit: OverworldExitState::load_from_ram(ram),
            transition: OverworldTransitionState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.event_info.write_to_ram(ram);
        self.config_table.write_to_ram(ram);
        self.map_ui.write_to_ram(ram);
        self.weather_vane.write_to_ram(ram);
        self.bird_travel_destinations.write_to_ram(ram);
        self.map_zoom.write_to_ram(ram);
        self.screen_size.write_to_ram(ram);
        self.scroll_delta.write_to_ram(ram);
        self.map16.write_to_ram(ram);
        self.entrance.write_to_ram(ram);
        self.exit.write_to_ram(ram);
        self.transition.write_to_ram(ram);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct RoomBoundsState {
    y_bounds: [u16; ROOM_BOUND_COUNT],
    x_bounds: [u16; ROOM_BOUND_COUNT],
}

impl RoomBoundsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut y_bounds = [0; ROOM_BOUND_COUNT];
        let mut x_bounds = [0; ROOM_BOUND_COUNT];
        for index in 0..ROOM_BOUND_COUNT {
            y_bounds[index] = read_le_u16(ram, ROOM_BOUNDS + index * 2);
            x_bounds[index] = read_le_u16(ram, ROOM_BOUNDS + 8 + index * 2);
        }
        Self { y_bounds, x_bounds }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for index in 0..ROOM_BOUND_COUNT {
            write_le_u16(ram, ROOM_BOUNDS + index * 2, self.y_bounds[index]);
            write_le_u16(ram, ROOM_BOUNDS + 8 + index * 2, self.x_bounds[index]);
        }
    }

    pub(crate) fn y_bound(&self, index: usize) -> u16 {
        self.y_bounds.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn x_bound(&self, index: usize) -> u16 {
        self.x_bounds.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn packed_bound(&self, index: usize) -> u16 {
        self.y_bound(index)
    }

    pub(crate) fn packed_top(&self) -> u16 {
        self.packed_bound(0)
    }

    pub(crate) fn packed_bottom(&self) -> u16 {
        self.packed_bound(1)
    }

    pub(crate) fn packed_left(&self) -> u16 {
        self.packed_bound(2)
    }

    pub(crate) fn packed_right(&self) -> u16 {
        self.packed_bound(3)
    }

    pub(crate) fn set_y_bound(&mut self, index: usize, value: u16) {
        if let Some(bound) = self.y_bounds.get_mut(index) {
            *bound = value;
        }
    }

    pub(crate) fn set_x_bound(&mut self, index: usize, value: u16) {
        if let Some(bound) = self.x_bounds.get_mut(index) {
            *bound = value;
        }
    }

    pub(crate) fn set_packed_bound(&mut self, index: usize, value: u16) {
        self.set_y_bound(index, value);
    }

    pub(crate) fn set_packed_bounds(&mut self, top: u16, bottom: u16, left: u16, right: u16) {
        self.y_bounds = [top, bottom, left, right];
    }

    pub(crate) fn add_y_bounds_a(&mut self, value: u16) {
        for index in [0, 2] {
            self.y_bounds[index] = self.y_bounds[index].wrapping_add(value);
        }
    }

    pub(crate) fn add_y_bounds_b(&mut self, value: u16) {
        for index in [1, 3] {
            self.y_bounds[index] = self.y_bounds[index].wrapping_add(value);
        }
    }

    pub(crate) fn add_x_bounds_a(&mut self, value: u16) {
        for index in [0, 2] {
            self.x_bounds[index] = self.x_bounds[index].wrapping_add(value);
        }
    }

    pub(crate) fn add_x_bounds_b(&mut self, value: u16) {
        for index in [1, 3] {
            self.x_bounds[index] = self.x_bounds[index].wrapping_add(value);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorldState {
    pub(crate) location: WorldLocationState,
    pub(crate) scroll: WorldScrollState,
    pub(crate) palette_theme: WorldPaletteThemeState,
    pub(crate) overworld: OverworldState,
    pub(crate) room_bounds: RoomBoundsState,
}

impl WorldState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            location: WorldLocationState::load_from_ram(ram),
            scroll: WorldScrollState::load_from_ram(ram),
            palette_theme: WorldPaletteThemeState::load_from_ram(ram),
            overworld: OverworldState::load_from_ram(ram),
            room_bounds: RoomBoundsState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.location.write_to_ram(ram);
        self.scroll.write_to_ram(ram);
        self.palette_theme.write_to_ram(ram);
        self.overworld.write_to_ram(ram);
        self.room_bounds.write_to_ram(ram);
    }
}

pub(crate) struct NativeWorldPaletteThemeBridgeMut<'a> {
    state: &'a mut WorldPaletteThemeState,
    ram: &'a mut [u8],
}

impl<'a> NativeWorldPaletteThemeBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut WorldPaletteThemeState, ram: &'a mut [u8]) -> Self {
        *state = WorldPaletteThemeState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, WorldPaletteThemeState::load_from_ram(self.ram));
    }

    pub(crate) fn set_last_light_vs_dark_world(&mut self, value: u8) {
        self.state.last_light_vs_dark_world = value;
        self.sync();
    }

    pub(crate) fn set_aux_bg_subset(&mut self, index: usize, value: u8) {
        if let Some(subset) = self.state.aux_bg_subset.get_mut(index) {
            *subset = value;
        }
        self.sync();
    }

    pub(crate) fn set_overworld_palette_aux1_hi(&mut self, value: u8) {
        self.state.overworld_palette_aux1_hi = value;
        self.sync();
    }

    pub(crate) fn set_hud_palette(&mut self, value: u8) {
        self.state.hud_palette = value;
        self.sync();
    }

    pub(crate) fn set_overworld_tile_theme_index(&mut self, value: u8) {
        self.state.overworld_tile_theme_index = value;
        self.sync();
    }

    pub(crate) fn set_main_tile_theme_index(&mut self, value: u8) {
        self.state.main_tile_theme_index = value;
        self.sync();
    }

    pub(crate) fn set_aux_tile_theme_index(&mut self, value: u8) {
        self.state.aux_tile_theme_index = value;
        self.sync();
    }

    pub(crate) fn set_misc_sprites_graphics_index(&mut self, value: u8) {
        self.state.misc_sprites_graphics_index = value;
        self.sync();
    }

    pub(crate) fn set_palette_sp6r_indoors(&mut self, value: u8) {
        self.state.palette_sp6r_indoors = value;
        self.sync();
    }

    pub(crate) fn restore_exit_tile_themes(&mut self) {
        self.state.overworld_tile_theme_index = self.state.exit_overworld_tile_theme_index;
        self.state.main_tile_theme_index = self.state.exit_main_tile_theme_index;
        self.state.aux_tile_theme_index = self.state.exit_aux_tile_theme_index;
        self.sync();
    }

    pub(crate) fn save_special_exit_tile_themes(&mut self) {
        self.state.special_exit_overworld_tile_theme_index = self.state.overworld_tile_theme_index;
        self.state.special_exit_main_tile_theme_index = self.state.main_tile_theme_index;
        self.state.special_exit_aux_tile_theme_index = self.state.aux_tile_theme_index;
        self.sync();
    }

    pub(crate) fn restore_special_exit_tile_themes(&mut self) {
        self.state.overworld_tile_theme_index = self.state.special_exit_overworld_tile_theme_index;
        self.state.main_tile_theme_index = self.state.special_exit_main_tile_theme_index;
        self.state.aux_tile_theme_index = self.state.special_exit_aux_tile_theme_index;
        self.sync();
    }
}

pub(crate) struct NativeWorldScrollBridgeMut<'a> {
    state: &'a mut WorldScrollState,
    ram: &'a mut [u8],
}

impl<'a> NativeWorldScrollBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut WorldScrollState, ram: &'a mut [u8]) -> Self {
        *state = WorldScrollState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, WorldScrollState::load_from_ram(self.ram));
    }

    pub(crate) fn set_bg1_x(&mut self, value: u16) {
        self.state.bg1_x = value;
        self.sync();
    }

    pub(crate) fn set_bg1_x_low(&mut self, value: u8) {
        self.state.bg1_x = (self.state.bg1_x & 0xff00) | u16::from(value);
        self.sync();
    }

    pub(crate) fn set_bg1_y(&mut self, value: u16) {
        self.state.bg1_y = value;
        self.sync();
    }

    pub(crate) fn set_bg1_y_low(&mut self, value: u8) {
        self.state.bg1_y = (self.state.bg1_y & 0xff00) | u16::from(value);
        self.sync();
    }

    pub(crate) fn set_bg2_x(&mut self, value: u16) {
        self.state.bg2_x = value;
        self.sync();
    }

    pub(crate) fn add_bg2_x(&mut self, value: u16) {
        self.state.bg2_x = self.state.bg2_x.wrapping_add(value);
        self.sync();
    }

    pub(crate) fn set_bg2_y(&mut self, value: u16) {
        self.state.bg2_y = value;
        self.sync();
    }

    pub(crate) fn set_bg1_x_offset(&mut self, value: u16) {
        self.state.bg1_x_offset = value;
        self.sync();
    }

    pub(crate) fn set_bg1_y_offset(&mut self, value: u16) {
        self.state.bg1_y_offset = value;
        self.sync();
    }

    pub(crate) fn set_bg1_offsets(&mut self, x: u16, y: u16) {
        self.state.bg1_x_offset = x;
        self.state.bg1_y_offset = y;
        self.sync();
    }

    pub(crate) fn clear_bg1_offsets(&mut self) {
        self.set_bg1_offsets(0, 0);
    }

    pub(crate) fn set_overworld_offset_base_y(&mut self, value: u16) {
        self.state.overworld_offset_base_y = value;
        self.sync();
    }

    pub(crate) fn set_overworld_offset_base_x(&mut self, value: u16) {
        self.state.overworld_offset_base_x = value;
        self.sync();
    }

    pub(crate) fn set_overworld_offset_mask_y(&mut self, value: u16) {
        self.state.overworld_offset_mask_y = value;
        self.sync();
    }

    pub(crate) fn set_overworld_offset_mask_x(&mut self, value: u16) {
        self.state.overworld_offset_mask_x = value;
        self.sync();
    }
}

pub(crate) struct NativeRoomBoundsBridgeMut<'a> {
    state: &'a mut RoomBoundsState,
    ram: &'a mut [u8],
}

impl<'a> NativeRoomBoundsBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut RoomBoundsState, ram: &'a mut [u8]) -> Self {
        *state = RoomBoundsState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, RoomBoundsState::load_from_ram(self.ram));
    }

    pub(crate) fn set_y_bound(&mut self, index: usize, value: u16) {
        self.state.set_y_bound(index, value);
        self.sync();
    }

    pub(crate) fn set_x_bound(&mut self, index: usize, value: u16) {
        self.state.set_x_bound(index, value);
        self.sync();
    }

    pub(crate) fn set_packed_bound(&mut self, index: usize, value: u16) {
        self.state.set_packed_bound(index, value);
        self.sync();
    }

    pub(crate) fn set_packed_bounds(&mut self, top: u16, bottom: u16, left: u16, right: u16) {
        self.state.set_packed_bounds(top, bottom, left, right);
        self.sync();
    }

    pub(crate) fn copy_y_bound_from(&mut self, index: usize, src: usize) {
        self.state.set_y_bound(index, read_le_u16(self.ram, src));
        self.sync();
    }

    pub(crate) fn copy_x_bound_from(&mut self, index: usize, src: usize) {
        self.state.set_x_bound(index, read_le_u16(self.ram, src));
        self.sync();
    }

    pub(crate) fn copy_packed_bound_from(&mut self, index: usize, src: usize) {
        self.state
            .set_packed_bound(index, read_le_u16(self.ram, src));
        self.sync();
    }

    pub(crate) fn add_y_bounds_a(&mut self, value: u16) {
        self.state.add_y_bounds_a(value);
        self.sync();
    }

    pub(crate) fn add_y_bounds_b(&mut self, value: u16) {
        self.state.add_y_bounds_b(value);
        self.sync();
    }

    pub(crate) fn add_x_bounds_a(&mut self, value: u16) {
        self.state.add_x_bounds_a(value);
        self.sync();
    }

    pub(crate) fn add_x_bounds_b(&mut self, value: u16) {
        self.state.add_x_bounds_b(value);
        self.sync();
    }

    pub(crate) fn copy_y_bounds_from(&mut self, src: usize, count: usize) {
        self.sync();
        for offset in 0..count {
            self.ram[ROOM_BOUNDS + offset] = self.ram[src + offset];
        }
        *self.state = RoomBoundsState::load_from_ram(self.ram);
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeWorldLocationBridgeMut<'a> {
    world_location: &'a mut WorldLocationState,
    ram: &'a mut [u8],
}

impl<'a> NativeWorldLocationBridgeMut<'a> {
    pub(crate) fn new(world_location: &'a mut WorldLocationState, ram: &'a mut [u8]) -> Self {
        *world_location = WorldLocationState::load_from_ram(ram);
        Self {
            world_location,
            ram,
        }
    }

    fn sync(&mut self) {
        self.world_location.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.world_location,
            WorldLocationState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_dungeon_room(&mut self, value: u16) {
        self.world_location.dungeon_room = value;
        self.sync();
    }

    pub(crate) fn set_dungeon_room_index(&mut self, value: u8) {
        self.world_location.dungeon_room =
            (self.world_location.dungeon_room & 0xff00) | u16::from(value);
        self.sync();
    }

    pub(crate) fn increment_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        let next = self.world_location.dungeon_room_index().wrapping_add(value);
        self.set_dungeon_room_index(next);
        next
    }

    pub(crate) fn decrement_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        let next = self.world_location.dungeon_room_index().wrapping_sub(value);
        self.set_dungeon_room_index(next);
        next
    }

    pub(crate) fn set_overworld_screen(&mut self, value: u8) {
        self.world_location.overworld_screen =
            (self.world_location.overworld_screen & 0xff00) | u16::from(value);
        self.sync();
    }

    pub(crate) fn set_overworld_screen_word(&mut self, value: u16) {
        self.world_location.overworld_screen = value;
        self.sync();
    }

    pub(crate) fn set_indoor_flag(&mut self, value: u8) {
        self.world_location.indoor_flag = value;
        self.sync();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overworld_map16_source_pages_read_named_wram_pages() {
        let mut ram = vec![0; 0x8000];
        write_le_u16(&mut ram, 0x2000 + 0x010, 0x1234);
        write_le_u16(&mut ram, 0x4000 + 0x010, 0xabcd);

        let decode = OverworldMap16Decode::new(&ram);
        assert_eq!(
            decode.source_page_word(OverworldMap16SourcePage::Main, 0x010),
            0x1234
        );
        assert_eq!(
            decode.source_page_word(OverworldMap16SourcePage::Overlay, 0x010),
            0xabcd
        );
    }
}

pub(crate) struct NativeOverworldEventInfoBridgeMut<'a> {
    event_info: &'a mut OverworldEventInfoState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldEventInfoBridgeMut<'a> {
    pub(crate) fn new(event_info: &'a mut OverworldEventInfoState, ram: &'a mut [u8]) -> Self {
        *event_info = OverworldEventInfoState::load_from_ram(ram);
        Self { event_info, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.event_info,
            OverworldEventInfoState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_event_info(&mut self, screen: usize, value: u8) {
        self.event_info.set_event_info(screen, value);
        self.ram[OVERWORLD_EVENT_INFO + screen] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_event_bits(&mut self, screen: usize, mask: u8) {
        self.event_info.set_event_bits(screen, mask);
        self.ram[OVERWORLD_EVENT_INFO + screen] |= mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_event_bits(&mut self, screen: usize, mask: u8) {
        self.event_info.clear_event_bits(screen, mask);
        self.ram[OVERWORLD_EVENT_INFO + screen] &= !mask;
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeOverworldConfigTableBridgeMut<'a> {
    config_table: &'a mut OverworldConfigTableState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldConfigTableBridgeMut<'a> {
    pub(crate) fn new(config_table: &'a mut OverworldConfigTableState, ram: &'a mut [u8]) -> Self {
        *config_table = OverworldConfigTableState::load_from_ram(ram);
        Self { config_table, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.config_table,
            OverworldConfigTableState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn copy_music_primary(&mut self, data: &[u8]) {
        self.config_table.copy_music_primary(data);
        self.ram[OVERWORLD_MUSIC_TABLE..OVERWORLD_MUSIC_TABLE + 64].copy_from_slice(&data[..64]);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn copy_music_secondary(&mut self, data: &[u8]) {
        self.config_table.copy_music_secondary(data);
        self.ram[OVERWORLD_MUSIC_TABLE + 64..OVERWORLD_MUSIC_TABLE + 160]
            .copy_from_slice(&data[..96]);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_music(&mut self, screen: usize, value: u8) {
        self.config_table.set_music(screen, value);
        self.ram[OVERWORLD_MUSIC_TABLE + screen] = value;
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeOverworldMapUiBridgeMut<'a> {
    map_ui: &'a mut OverworldMapUiState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldMapUiBridgeMut<'a> {
    pub(crate) fn new(map_ui: &'a mut OverworldMapUiState, ram: &'a mut [u8]) -> Self {
        *map_ui = OverworldMapUiState::load_from_ram(ram);
        Self { map_ui, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.map_ui, OverworldMapUiState::load_from_ram(self.ram));
    }

    pub(crate) fn set_map_state(&mut self, value: u8) {
        self.map_ui.map_state = (self.map_ui.map_state & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_MAP_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_map_state_word(&mut self, value: u16) {
        self.map_ui.map_state = value;
        write_le_u16(self.ram, OVERWORLD_MAP_STATE, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_map_state(&mut self) {
        let next = self.map_ui.map_state().wrapping_add(1);
        self.set_map_state(next);
    }

    pub(crate) fn set_map_flags(&mut self, value: u8) {
        self.map_ui.map_flags = value;
        self.ram[OVERWORLD_MAP_FLAGS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn and_map_flags(&mut self, value: u8) {
        let next = self.map_ui.map_flags & value;
        self.set_map_flags(next);
    }

    pub(crate) fn or_map_flags(&mut self, value: u8) {
        let next = self.map_ui.map_flags | value;
        self.set_map_flags(next);
    }

    pub(crate) fn set_birdtravel_status(&mut self, value: u8) {
        self.map_ui.bird_travel_statuses.set_status(0, value);
        self.ram[BIRDTRAVEL_STATUS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_birdtravel_status_word(&mut self, value: u16) {
        self.map_ui.bird_travel_statuses.set_status_word(value);
        write_le_u16(self.ram, BIRDTRAVEL_STATUS, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn and_birdtravel_status(&mut self, value: u8) {
        let next = self.map_ui.birdtravel_status() & value;
        self.set_birdtravel_status(next);
    }

    pub(crate) fn decrement_birdtravel_status(&mut self) {
        let next = self.map_ui.birdtravel_status().wrapping_sub(1);
        self.set_birdtravel_status(next);
    }

    pub(crate) fn increment_birdtravel_status(&mut self) {
        let next = self.map_ui.birdtravel_status().wrapping_add(1);
        self.set_birdtravel_status(next);
    }

    pub(crate) fn clear_bird_travel_stop_status(&mut self, slot: usize) {
        self.map_ui.bird_travel_statuses.clear_status(slot);
        self.ram[BIRD_TRAVEL_STATUS + slot] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_bird_travel_stop_status(&mut self, slot: usize) {
        self.map_ui.bird_travel_statuses.increment_status(slot);
        self.ram[BIRD_TRAVEL_STATUS + slot] = self.ram[BIRD_TRAVEL_STATUS + slot].wrapping_add(1);
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeWeatherVaneBridgeMut<'a> {
    weather_vane: &'a mut WeatherVaneState,
    ram: &'a mut [u8],
}

impl<'a> NativeWeatherVaneBridgeMut<'a> {
    pub(crate) fn new(weather_vane: &'a mut WeatherVaneState, ram: &'a mut [u8]) -> Self {
        *weather_vane = WeatherVaneState::load_from_ram(ram);
        Self { weather_vane, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.weather_vane,
            WeatherVaneState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_countdown(&mut self, value: u16) {
        self.weather_vane.countdown = value;
        write_le_u16(self.ram, WEATHERVANE_COUNTDOWN, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn tick_countdown(&mut self) -> u16 {
        let value = self.weather_vane.tick_countdown();
        write_le_u16(self.ram, WEATHERVANE_COUNTDOWN, value);
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_music_latch(&mut self, value: u8) {
        self.weather_vane.music_latch = value;
        self.ram[WEATHERVANE_MUSIC_LATCH] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_source_slot(&mut self, value: u8) {
        self.weather_vane.source_slot = value;
        self.ram[WEATHERVANE_SOURCE_SLOT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_oam_offset(&mut self) {
        self.weather_vane.reset_oam_offset();
        self.ram[WEATHERVANE_OAM_OFFSET] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_oam_offset(&mut self, value: u8) {
        self.weather_vane.advance_oam_offset(value);
        self.ram[WEATHERVANE_OAM_OFFSET] = self.ram[WEATHERVANE_OAM_OFFSET].wrapping_add(value);
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeBirdTravelDestinationBridgeMut<'a> {
    destinations: &'a mut BirdTravelDestinationsState,
    ram: &'a mut [u8],
}

impl<'a> NativeBirdTravelDestinationBridgeMut<'a> {
    pub(crate) fn new(
        destinations: &'a mut BirdTravelDestinationsState,
        ram: &'a mut [u8],
    ) -> Self {
        *destinations = BirdTravelDestinationsState::load_from_ram(ram);
        Self { destinations, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.destinations,
            BirdTravelDestinationsState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_destination(&mut self, slot: usize, x: u16, y: u16) {
        self.destinations.set_destination(slot, x, y);
        self.ram[BIRD_TRAVEL_X_LO + slot] = x as u8;
        self.ram[BIRD_TRAVEL_X_HI + slot] = (x >> 8) as u8;
        self.ram[BIRD_TRAVEL_Y_LO + slot] = y as u8;
        self.ram[BIRD_TRAVEL_Y_HI + slot] = (y >> 8) as u8;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_destination(&mut self, slot: usize) {
        self.set_destination(slot, 0, 0);
    }
}

pub(crate) struct NativeOverworldMapZoomBridgeMut<'a> {
    zoom: &'a mut OverworldMapZoomState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldMapZoomBridgeMut<'a> {
    pub(crate) fn new(zoom: &'a mut OverworldMapZoomState, ram: &'a mut [u8]) -> Self {
        *zoom = OverworldMapZoomState::load_from_ram(ram);
        Self { zoom, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.zoom, OverworldMapZoomState::load_from_ram(self.ram));
    }

    pub(crate) fn set_step_counter(&mut self, value: u8) {
        self.zoom.step_counter = value;
        self.ram[MODE7_ZOOM_STEP_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.zoom.timer = value;
        self.ram[TIMER_FOR_MODE7_ZOOM] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_timer(&mut self) {
        let next = self.zoom.timer.wrapping_sub(1);
        self.set_timer(next);
    }
}

pub(crate) struct NativeOverworldScreenSizeBridgeMut<'a> {
    screen_size: &'a mut OverworldScreenSizeState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldScreenSizeBridgeMut<'a> {
    pub(crate) fn new(screen_size: &'a mut OverworldScreenSizeState, ram: &'a mut [u8]) -> Self {
        *screen_size = OverworldScreenSizeState::load_from_ram(ram);
        Self { screen_size, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.screen_size,
            OverworldScreenSizeState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_big_area_high(&mut self) {
        self.screen_size.big_area &= 0x00ff;
        self.ram[OVERWORLD_AREA_IS_BIG + 1] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_big_area_low(&mut self, value: u8) {
        self.screen_size.big_area = (self.screen_size.big_area & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_AREA_IS_BIG] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn backup_big_area_low(&mut self) {
        self.screen_size.big_area_backup = self.screen_size.big_area as u8;
        self.ram[OVERWORLD_AREA_IS_BIG_BACKUP] = self.screen_size.big_area_backup;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_right_bottom_bound_low(&mut self, value: u8) {
        self.screen_size.right_bottom_scroll_bound =
            (self.screen_size.right_bottom_scroll_bound & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_right_bottom_bound_high(&mut self, value: u8) {
        self.screen_size.right_bottom_scroll_bound =
            (self.screen_size.right_bottom_scroll_bound & 0x00ff) | (u16::from(value) << 8);
        self.ram[OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND + 1] = value;
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeOverworldScrollDeltaBridgeMut<'a> {
    scroll_delta: &'a mut OverworldScrollDeltaState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldScrollDeltaBridgeMut<'a> {
    pub(crate) fn new(scroll_delta: &'a mut OverworldScrollDeltaState, ram: &'a mut [u8]) -> Self {
        *scroll_delta = OverworldScrollDeltaState::load_from_ram(ram);
        Self { scroll_delta, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.scroll_delta,
            OverworldScrollDeltaState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_vertical_delta_low_byte(&mut self, value: u8) {
        self.scroll_delta.set_vertical_delta_low_byte(value);
        self.ram[OVERWORLD_SCROLL_DELTA] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_horizontal_delta_low_byte(&mut self, value: u8) {
        self.scroll_delta.set_horizontal_delta_low_byte(value);
        self.ram[OVERWORLD_SCROLL_DELTA + 1] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_vertical_delta_word(&mut self, value: u16) {
        self.scroll_delta.set_vertical_delta_word(value);
        write_le_u16(self.ram, OVERWORLD_SCROLL_DELTA, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_horizontal_delta_word(&mut self, value: u16) {
        self.scroll_delta.set_horizontal_delta_word(value);
        write_le_u16(self.ram, OVERWORLD_SCROLL_DELTA + 1, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_vertical_delta_low_byte(&mut self) {
        self.scroll_delta.clear_vertical_delta_low_byte();
        self.ram[OVERWORLD_SCROLL_DELTA] = 0;
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeOverworldMap16BridgeMut<'a> {
    map16: &'a mut OverworldMap16State,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldMap16BridgeMut<'a> {
    pub(crate) fn new(map16: &'a mut OverworldMap16State, ram: &'a mut [u8]) -> Self {
        *map16 = OverworldMap16State::load_from_ram(ram);
        Self { map16, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.map16, OverworldMap16State::load_from_ram(self.ram));
    }

    pub(crate) fn set_active_load(&mut self, state: OverworldMap16LoadState) {
        self.map16.active_load = state;
        state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_previous_load(&mut self, state: OverworldMap16LoadState) {
        self.map16.previous_load = state;
        state.write_previous_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_special_exit_src_off(&mut self, src_off: u16) {
        self.map16.special_exit_src_off = src_off;
        write_le_u16(self.ram, MAP16_LOAD_SRC_OFF_SPEXIT, src_off);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_exit_src_off(&mut self, src_off: u16) {
        self.map16.exit_src_off = src_off;
        write_le_u16(self.ram, MAP16_LOAD_SRC_OFF_EXIT, src_off);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_small_scroll_backup(&mut self, state: SmallOverworldMap16ScrollBackupState) {
        self.map16.small_scroll_backup = state;
        state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeOverworldEntranceBridgeMut<'a> {
    entrance: &'a mut OverworldEntranceState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldEntranceBridgeMut<'a> {
    pub(crate) fn new(entrance: &'a mut OverworldEntranceState, ram: &'a mut [u8]) -> Self {
        *entrance = OverworldEntranceState::load_from_ram(ram);
        Self { entrance, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.entrance,
            OverworldEntranceState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_special_entrance_trigger(&mut self, value: u8) {
        self.entrance.special_entrance_trigger = value;
        self.ram[TRIGGER_SPECIAL_ENTRANCE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_special_entrance_trigger(&mut self) {
        self.set_special_entrance_trigger(0);
    }

    pub(crate) fn set_sequence_counter(&mut self, value: u8) {
        self.entrance.sequence_counter = value;
        self.ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_sequence_counter(&mut self) {
        self.set_sequence_counter(0);
    }

    pub(crate) fn increment_sequence_counter(&mut self) -> u8 {
        let next = self.entrance.sequence_counter.wrapping_add(1);
        self.set_sequence_counter(next);
        next
    }

    pub(crate) fn decrement_sequence_counter(&mut self) -> u8 {
        let next = self.entrance.sequence_counter.wrapping_sub(1);
        self.set_sequence_counter(next);
        next
    }
}

pub(crate) struct NativeOverworldExitBridgeMut<'a> {
    exit: &'a mut OverworldExitState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldExitBridgeMut<'a> {
    pub(crate) fn new(exit: &'a mut OverworldExitState, ram: &'a mut [u8]) -> Self {
        *exit = OverworldExitState::load_from_ram(ram);
        Self { exit, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.exit, OverworldExitState::load_from_ram(self.ram));
    }

    pub(crate) fn set_exit_screen(&mut self, value: u16) {
        self.exit.exit_screen = value;
        write_le_u16(self.ram, OVERWORLD_SCREEN_INDEX_EXIT, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_special_exit_screen(&mut self, value: u16) {
        self.exit.special_exit_screen = value;
        write_le_u16(self.ram, OVERWORLD_SCREEN_INDEX_SPEXIT, value);
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeOverworldTransitionBridgeMut<'a> {
    transition: &'a mut OverworldTransitionState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldTransitionBridgeMut<'a> {
    pub(crate) fn new(transition: &'a mut OverworldTransitionState, ram: &'a mut [u8]) -> Self {
        *transition = OverworldTransitionState::load_from_ram(ram);
        Self { transition, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.transition,
            OverworldTransitionState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_direction_bits(&mut self, value: u8) {
        self.transition.direction_bits =
            (self.transition.direction_bits & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS2] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_bits_word(&mut self, value: u16) {
        self.transition.direction_bits = value;
        write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_direction_bits(&mut self) {
        self.set_direction_bits(0);
    }

    pub(crate) fn clear_direction_bits_word(&mut self) {
        self.set_direction_bits_word(0);
    }

    pub(crate) fn and_direction_bits(&mut self, value: u8) {
        let next = self.transition.direction_bits() & value;
        self.set_direction_bits(next);
    }

    pub(crate) fn or_direction_bits(&mut self, value: u8) {
        let next = self.transition.direction_bits() | value;
        self.set_direction_bits(next);
    }

    pub(crate) fn or_direction_bits_word(&mut self, value: u16) -> u16 {
        let next = self.transition.direction_bits_word() | value;
        self.set_direction_bits_word(next);
        next
    }

    pub(crate) fn set_edge_direction_bits(&mut self, value: u8) {
        self.transition.edge_direction_bits =
            (self.transition.edge_direction_bits & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_edge_direction_bits_word(&mut self, value: u16) {
        self.transition.edge_direction_bits = value;
        write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_edge_direction_bits(&mut self) {
        self.set_edge_direction_bits(0);
    }

    pub(crate) fn set_direction_enum(&mut self, value: u8) {
        self.transition.direction_enum = value;
        self.ram[OVERWORLD_TRANSITION_DIR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_screen_transition(&mut self, value: u8) {
        self.transition.screen_transition =
            (self.transition.screen_transition & 0xff00) | u16::from(value);
        self.ram[OVERWORLD_SCREEN_TRANSITION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_screen_transition_word(&mut self, value: u16) {
        self.transition.screen_transition = value;
        write_le_u16(self.ram, OVERWORLD_SCREEN_TRANSITION, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_screen_transition(&mut self) {
        self.set_screen_transition(0);
    }

    pub(crate) fn set_transition_counter(&mut self, value: u8) {
        self.transition.transition_counter = value;
        self.ram[TRANSITION_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_transition_counter(&mut self) -> u8 {
        let next = self.transition.transition_counter.wrapping_add(1);
        self.set_transition_counter(next);
        next
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.transition.countdown = value;
        self.ram[OW_COUNTDOWN_TRANSITION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_countdown(&mut self) -> u8 {
        let next = self.transition.countdown.wrapping_sub(1);
        self.set_countdown(next);
        next
    }

    pub(crate) fn save_previous_direction_bits(&mut self) {
        self.transition.previous_direction_bits = u16::from(self.transition.edge_direction_bits());
        self.transition.previous_direction_bits2 = self.transition.direction_bits_word();
        write_le_u16(
            self.ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV,
            self.transition.previous_direction_bits,
        );
        write_le_u16(
            self.ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV,
            self.transition.previous_direction_bits2,
        );
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_previous_direction_bits(&mut self) {
        self.set_edge_direction_bits_word(self.transition.previous_direction_bits);
        self.set_direction_bits_word(self.transition.previous_direction_bits2);
    }

    pub(crate) fn set_previous_screen_transition(&mut self, value: u8) {
        self.transition.previous_screen_transition = value;
        self.ram[OVERWORLD_SCREEN_TRANSITION_PREV] = value;
        self.debug_assert_matches_ram();
    }
}
