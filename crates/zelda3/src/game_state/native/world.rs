use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const BIRD_TRAVEL_DESTINATION_SLOTS: usize = 16;
const BIRD_TRAVEL_STATUS_SLOTS: usize = 16;
const OVERWORLD_EVENT_INFO_SCREENS: usize = 160;
const OVERWORLD_CONFIG_SCREENS: usize = 160;
const OVERWORLD_SPRITE_GFX_SCREENS: usize =
    OVERWORLD_SPRITE_PALETTE_TABLE - OVERWORLD_SPRITE_GFX_TABLE;
const ROOM_BOUND_COUNT: usize = 4;
const SCROLL_TARGET_COUNT: usize = 4;
const SCROLL_COUNTER_COUNT: usize = 4;
const DUNGEON_REPLACEMENT_TILE_WORDS: usize = 0x400;
pub(crate) const DOOR_ANIMATION_REPLACEMENT_TILE_INDEX: usize =
    (DOOR_ANIMATION_STEP_INDICATOR - DUNG_REPLACEMENT_TILE_STATE) / 2;

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

pub(crate) struct OverworldMap16DecodeScratch;

impl OverworldMap16DecodeScratch {
    pub(crate) fn copy_source_from(ram: &mut [u8], data: &[u8]) {
        ram[OVERWORLD_MAP16_DECODE_SRC..OVERWORLD_MAP16_DECODE_SRC + data.len()]
            .copy_from_slice(data);
    }

    pub(crate) fn copy_scratch_to_source_words_high(ram: &mut [u8], len: usize) {
        for i in 0..len {
            ram[OVERWORLD_MAP16_DECODE_SRC + 1 + i * 2] = ram[OVERWORLD_DECOMP_BUFFER + i];
        }
    }

    pub(crate) fn copy_scratch_to_source_words_low(ram: &mut [u8], len: usize) {
        for i in 0..len {
            ram[OVERWORLD_MAP16_DECODE_SRC + i * 2] = ram[OVERWORLD_DECOMP_BUFFER + i];
        }
    }

    pub(crate) fn write_decompressed_byte(ram: &mut [u8], dst: usize, value: u8) {
        ram[dst] = value;
    }

    pub(crate) fn copy_decompressed_byte(
        ram: &mut [u8],
        dst_org: usize,
        dst: usize,
        offset: usize,
    ) {
        ram[dst] = ram[dst_org + offset];
    }

    pub(crate) fn decomp_scratch_byte_mut(ram: &mut [u8], index: usize) -> &mut u8 {
        &mut ram[OVERWORLD_DECOMP_BUFFER + index]
    }

    pub(crate) fn decomp_scratch_slice_mut(ram: &mut [u8]) -> &mut [u8] {
        &mut ram[OVERWORLD_DECOMP_BUFFER..]
    }

    pub(crate) fn decode_block_fill(ram: &mut [u8], dst: usize, table: &[u8], x: usize) {
        ram[dst] = table[x];
        ram[dst + 2] = table[x + 1];
        ram[dst + 4] = table[x + 2];
        ram[dst + 6] = table[x + 3];
        let packed0 = table[x + 4];
        let packed1 = table[x + 5];
        ram[dst + 1] = packed0 >> 4;
        ram[dst + 3] = packed0 & 0x0f;
        ram[dst + 5] = packed1 >> 4;
        ram[dst + 7] = packed1 & 0x0f;
    }

    pub(crate) fn set_decode_last(ram: &mut [u8], value: u16) {
        write_le_u16(ram, MAP16_DECODE_LAST, value);
    }

    pub(crate) fn set_decode_tmp(ram: &mut [u8], value: u16) {
        write_le_u16(ram, MAP16_DECODE_WORK_WORD, value);
    }

    pub(crate) fn write_decoded_map32_to_bg2_tilemap(ram: &mut [u8], dst: usize, idx: usize) {
        let v0 = read_le_u16(ram, MAP16_DECODE_0 + idx);
        let v1 = read_le_u16(ram, MAP16_DECODE_1 + idx);
        let v2 = read_le_u16(ram, MAP16_DECODE_2 + idx);
        let v3 = read_le_u16(ram, MAP16_DECODE_3 + idx);
        write_le_u16(ram, dst, v0);
        write_le_u16(ram, dst + 128, v2);
        write_le_u16(ram, dst + 2, v1);
        write_le_u16(ram, dst + 130, v3);
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
        OverworldMap16DecodeScratch::copy_source_from(self.ram, data);
    }

    pub(crate) fn copy_scratch_to_source_words_high(&mut self, len: usize) {
        OverworldMap16DecodeScratch::copy_scratch_to_source_words_high(self.ram, len);
    }

    pub(crate) fn copy_scratch_to_source_words_low(&mut self, len: usize) {
        OverworldMap16DecodeScratch::copy_scratch_to_source_words_low(self.ram, len);
    }

    pub(crate) fn write_decompressed_byte(&mut self, dst: usize, value: u8) {
        OverworldMap16DecodeScratch::write_decompressed_byte(self.ram, dst, value);
    }

    pub(crate) fn copy_decompressed_byte(&mut self, dst_org: usize, dst: usize, offset: usize) {
        OverworldMap16DecodeScratch::copy_decompressed_byte(self.ram, dst_org, dst, offset);
    }

    pub(crate) fn decomp_scratch_byte_mut(&mut self, index: usize) -> &mut u8 {
        OverworldMap16DecodeScratch::decomp_scratch_byte_mut(self.ram, index)
    }

    pub(crate) fn decomp_scratch_slice_mut(&mut self) -> &mut [u8] {
        OverworldMap16DecodeScratch::decomp_scratch_slice_mut(self.ram)
    }

    pub(crate) fn decode_block_fill(&mut self, dst: usize, table: &[u8], x: usize) {
        OverworldMap16DecodeScratch::decode_block_fill(self.ram, dst, table, x);
    }

    pub(crate) fn set_decode_last(&mut self, value: u16) {
        OverworldMap16DecodeScratch::set_decode_last(self.ram, value);
    }

    pub(crate) fn set_decode_tmp(&mut self, value: u16) {
        OverworldMap16DecodeScratch::set_decode_tmp(self.ram, value);
    }

    pub(crate) fn write_decoded_map32_to_bg2_tilemap(&mut self, dst: usize, idx: usize) {
        OverworldMap16DecodeScratch::write_decoded_map32_to_bg2_tilemap(self.ram, dst, idx);
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

    pub(crate) fn dungeon_room(&self) -> u16 {
        self.dungeon_room
    }

    pub(crate) fn dungeon_room_index(&self) -> u8 {
        self.dungeon_room as u8
    }

    pub(crate) fn overworld_screen(&self) -> u16 {
        self.overworld_screen
    }

    pub(crate) fn overworld_screen_index(&self) -> u8 {
        self.overworld_screen as u8
    }

    pub(crate) fn indoor_flag(&self) -> u8 {
        self.indoor_flag
    }

    pub(crate) fn is_indoors(&self) -> bool {
        self.indoor_flag != 0
    }

    pub(crate) fn is_outdoors(&self) -> bool {
        !self.is_indoors()
    }

    pub(crate) fn set_dungeon_room(&mut self, value: u16) {
        self.dungeon_room = value;
    }

    pub(crate) fn set_dungeon_room_index(&mut self, value: u8) {
        self.dungeon_room = (self.dungeon_room & 0xff00) | u16::from(value);
    }

    pub(crate) fn increment_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        let next = self.dungeon_room_index().wrapping_add(value);
        self.set_dungeon_room_index(next);
        next
    }

    pub(crate) fn decrement_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        let next = self.dungeon_room_index().wrapping_sub(value);
        self.set_dungeon_room_index(next);
        next
    }

    pub(crate) fn set_overworld_screen(&mut self, value: u8) {
        self.overworld_screen = (self.overworld_screen & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_overworld_screen_word(&mut self, value: u16) {
        self.overworld_screen = value;
    }

    pub(crate) fn set_indoor_flag(&mut self, value: u8) {
        self.indoor_flag = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorldScrollState {
    // BG scroll copy2 (0xe0/0xe2/0xe6/0xe8) now lives solely in PpuScrollCopyState.
    pub(crate) bg1_x_offset: u16,
    pub(crate) bg1_y_offset: u16,
    // camera_x/camera_y (0x61c/0x618) now live solely in WorldCameraBoundariesState
    // (camera_x_coord_scroll_low/camera_y_coord_scroll_low), the authoritative
    // owner written by the dungeon/overworld camera logic.
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
            bg1_x_offset: read_le_u16(ram, BG1_X_OFFSET),
            bg1_y_offset: read_le_u16(ram, BG1_Y_OFFSET),
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
        write_le_u16(ram, BG1_X_OFFSET, self.bg1_x_offset);
        write_le_u16(ram, BG1_Y_OFFSET, self.bg1_y_offset);
        write_le_u16(ram, OVERWORLD_OFFSET_BASE_X, self.overworld_offset_base_x);
        write_le_u16(ram, OVERWORLD_OFFSET_BASE_Y, self.overworld_offset_base_y);
        write_le_u16(ram, OVERWORLD_OFFSET_MASK_X, self.overworld_offset_mask_x);
        write_le_u16(ram, OVERWORLD_OFFSET_MASK_Y, self.overworld_offset_mask_y);
        write_le_u16(ram, OVERWORLD_SCROLL_X_START, self.scroll_x_start);
        write_le_u16(ram, OVERWORLD_SCROLL_X_END, self.scroll_x_end);
        write_le_u16(ram, OVERWORLD_SCROLL_Y_END, self.scroll_y_end);
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

    pub(crate) fn set_bg1_x_offset(&mut self, value: u16) {
        self.bg1_x_offset = value;
    }

    pub(crate) fn set_bg1_y_offset(&mut self, value: u16) {
        self.bg1_y_offset = value;
    }

    pub(crate) fn set_bg1_offsets(&mut self, x: u16, y: u16) {
        self.bg1_x_offset = x;
        self.bg1_y_offset = y;
    }

    pub(crate) fn clear_bg1_offsets(&mut self) {
        self.set_bg1_offsets(0, 0);
    }

    pub(crate) fn set_overworld_offset_base_y(&mut self, value: u16) {
        self.overworld_offset_base_y = value;
    }

    pub(crate) fn set_overworld_offset_base_x(&mut self, value: u16) {
        self.overworld_offset_base_x = value;
    }

    pub(crate) fn set_overworld_offset_mask_y(&mut self, value: u16) {
        self.overworld_offset_mask_y = value;
    }

    pub(crate) fn set_overworld_offset_mask_x(&mut self, value: u16) {
        self.overworld_offset_mask_x = value;
    }

    pub(crate) fn set_scroll_x_start(&mut self, value: u16) {
        self.scroll_x_start = value;
    }

    pub(crate) fn set_scroll_x_end(&mut self, value: u16) {
        self.scroll_x_end = value;
    }

    pub(crate) fn set_scroll_y_end(&mut self, value: u16) {
        self.scroll_y_end = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorldCameraBoundariesState {
    pub(crate) camera_y_low: u16,
    pub(crate) camera_y_hi: u16,
    pub(crate) camera_x_low: u16,
    pub(crate) camera_x_hi: u16,
    pub(crate) scroll_targets: [u16; SCROLL_TARGET_COUNT],
    pub(crate) cached_scroll_targets: [u16; SCROLL_TARGET_COUNT],
    pub(crate) special_exit_scroll_targets: [u16; SCROLL_TARGET_COUNT],
    pub(crate) exit_scroll_targets: [u16; SCROLL_TARGET_COUNT],
    pub(crate) scroll_counters: [u16; SCROLL_COUNTER_COUNT],
    pub(crate) special_exit_scroll_counters: [u16; SCROLL_COUNTER_COUNT],
    pub(crate) exit_scroll_counters: [u16; SCROLL_COUNTER_COUNT],
    pub(crate) special_exit_camera_y_low: u16,
    pub(crate) special_exit_camera_x_low: u16,
    pub(crate) exit_camera_y_low: u16,
    pub(crate) exit_camera_x_low: u16,
    pub(crate) cached_camera_y_low: u16,
    pub(crate) cached_camera_x_low: u16,
    pub(crate) exit_room_bounds_y_start: u16,
    pub(crate) exit_room_bounds_y_end: u16,
    pub(crate) exit_room_bounds_x_start: u16,
    pub(crate) exit_room_bounds_x_end: u16,
    pub(crate) special_exit_room_bounds_y_start: u16,
    pub(crate) special_exit_room_bounds_y_end: u16,
    pub(crate) special_exit_room_bounds_x_start: u16,
    pub(crate) special_exit_room_bounds_x_end: u16,
}

impl WorldCameraBoundariesState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            camera_y_low: read_le_u16(ram, CAMERA_Y_COORD_SCROLL_LOW),
            camera_y_hi: read_le_u16(ram, CAMERA_Y_COORD_SCROLL_HI),
            camera_x_low: read_le_u16(ram, CAMERA_X_COORD_SCROLL_LOW),
            camera_x_hi: read_le_u16(ram, CAMERA_X_COORD_SCROLL_HI),
            scroll_targets: read_scroll_targets(ram, UP_DOWN_SCROLL_TARGET),
            cached_scroll_targets: [
                read_le_u16(ram, UP_DOWN_SCROLL_TARGET_CACHED),
                read_le_u16(ram, UP_DOWN_SCROLL_TARGET_END_CACHED),
                read_le_u16(ram, LEFT_RIGHT_SCROLL_TARGET_CACHED),
                read_le_u16(ram, LEFT_RIGHT_SCROLL_TARGET_END_CACHED),
            ],
            special_exit_scroll_targets: read_scroll_targets(ram, UP_DOWN_SCROLL_TARGET_SPEXIT),
            exit_scroll_targets: read_scroll_targets(ram, UP_DOWN_SCROLL_TARGET_EXIT),
            scroll_counters: read_scroll_counters(ram, OVERWORLD_SCROLL_UP_COUNTER),
            special_exit_scroll_counters: read_scroll_counters(
                ram,
                OVERWORLD_SCROLL_UP_COUNTER_SPEXIT,
            ),
            exit_scroll_counters: read_scroll_counters(ram, OVERWORLD_SCROLL_UP_COUNTER_EXIT),
            special_exit_camera_y_low: read_le_u16(ram, CAMERA_Y_COORD_SCROLL_LOW_SPEXIT),
            special_exit_camera_x_low: read_le_u16(ram, CAMERA_X_COORD_SCROLL_LOW_SPEXIT),
            exit_camera_y_low: read_le_u16(ram, CAMERA_Y_COORD_SCROLL_LOW_EXIT),
            exit_camera_x_low: read_le_u16(ram, CAMERA_X_COORD_SCROLL_LOW_EXIT),
            cached_camera_y_low: read_le_u16(ram, CAMERA_Y_COORD_SCROLL_LOW_CACHED),
            cached_camera_x_low: read_le_u16(ram, CAMERA_X_COORD_SCROLL_LOW_CACHED),
            exit_room_bounds_y_start: read_le_u16(ram, OW_SCROLL_VARS0_EXIT),
            exit_room_bounds_y_end: read_le_u16(ram, OW_SCROLL_VARS0_EXIT + 2),
            exit_room_bounds_x_start: read_le_u16(ram, OW_SCROLL_VARS0_EXIT + 4),
            exit_room_bounds_x_end: read_le_u16(ram, OW_SCROLL_VARS0_EXIT + 6),
            special_exit_room_bounds_y_start: read_le_u16(ram, SPECIAL_EXIT_ROOM_BOUNDS_Y_START),
            special_exit_room_bounds_y_end: read_le_u16(ram, SPECIAL_EXIT_ROOM_BOUNDS_Y_END),
            special_exit_room_bounds_x_start: read_le_u16(ram, SPECIAL_EXIT_ROOM_BOUNDS_X_START),
            special_exit_room_bounds_x_end: read_le_u16(ram, SPECIAL_EXIT_ROOM_BOUNDS_X_END),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, CAMERA_Y_COORD_SCROLL_LOW, self.camera_y_low);
        write_le_u16(ram, CAMERA_Y_COORD_SCROLL_HI, self.camera_y_hi);
        write_le_u16(ram, CAMERA_X_COORD_SCROLL_LOW, self.camera_x_low);
        write_le_u16(ram, CAMERA_X_COORD_SCROLL_HI, self.camera_x_hi);
        write_scroll_targets(ram, UP_DOWN_SCROLL_TARGET, self.scroll_targets);
        write_le_u16(
            ram,
            UP_DOWN_SCROLL_TARGET_CACHED,
            self.cached_scroll_targets[0],
        );
        write_le_u16(
            ram,
            UP_DOWN_SCROLL_TARGET_END_CACHED,
            self.cached_scroll_targets[1],
        );
        write_le_u16(
            ram,
            LEFT_RIGHT_SCROLL_TARGET_CACHED,
            self.cached_scroll_targets[2],
        );
        write_le_u16(
            ram,
            LEFT_RIGHT_SCROLL_TARGET_END_CACHED,
            self.cached_scroll_targets[3],
        );
        write_scroll_targets(
            ram,
            UP_DOWN_SCROLL_TARGET_SPEXIT,
            self.special_exit_scroll_targets,
        );
        write_scroll_targets(ram, UP_DOWN_SCROLL_TARGET_EXIT, self.exit_scroll_targets);
        write_scroll_counters(ram, OVERWORLD_SCROLL_UP_COUNTER, self.scroll_counters);
        write_scroll_counters(
            ram,
            OVERWORLD_SCROLL_UP_COUNTER_SPEXIT,
            self.special_exit_scroll_counters,
        );
        write_scroll_counters(
            ram,
            OVERWORLD_SCROLL_UP_COUNTER_EXIT,
            self.exit_scroll_counters,
        );
        write_le_u16(
            ram,
            CAMERA_Y_COORD_SCROLL_LOW_SPEXIT,
            self.special_exit_camera_y_low,
        );
        write_le_u16(
            ram,
            CAMERA_X_COORD_SCROLL_LOW_SPEXIT,
            self.special_exit_camera_x_low,
        );
        write_le_u16(ram, CAMERA_Y_COORD_SCROLL_LOW_EXIT, self.exit_camera_y_low);
        write_le_u16(ram, CAMERA_X_COORD_SCROLL_LOW_EXIT, self.exit_camera_x_low);
        write_le_u16(
            ram,
            CAMERA_Y_COORD_SCROLL_LOW_CACHED,
            self.cached_camera_y_low,
        );
        write_le_u16(
            ram,
            CAMERA_X_COORD_SCROLL_LOW_CACHED,
            self.cached_camera_x_low,
        );
        write_le_u16(ram, OW_SCROLL_VARS0_EXIT, self.exit_room_bounds_y_start);
        write_le_u16(ram, OW_SCROLL_VARS0_EXIT + 2, self.exit_room_bounds_y_end);
        write_le_u16(ram, OW_SCROLL_VARS0_EXIT + 4, self.exit_room_bounds_x_start);
        write_le_u16(ram, OW_SCROLL_VARS0_EXIT + 6, self.exit_room_bounds_x_end);
        write_le_u16(
            ram,
            SPECIAL_EXIT_ROOM_BOUNDS_Y_START,
            self.special_exit_room_bounds_y_start,
        );
        write_le_u16(
            ram,
            SPECIAL_EXIT_ROOM_BOUNDS_Y_END,
            self.special_exit_room_bounds_y_end,
        );
        write_le_u16(
            ram,
            SPECIAL_EXIT_ROOM_BOUNDS_X_START,
            self.special_exit_room_bounds_x_start,
        );
        write_le_u16(
            ram,
            SPECIAL_EXIT_ROOM_BOUNDS_X_END,
            self.special_exit_room_bounds_x_end,
        );
    }

    pub(crate) fn camera_y_coord_scroll_low(&self) -> u16 {
        self.camera_y_low
    }

    pub(crate) fn camera_y_coord_scroll_hi(&self) -> u16 {
        self.camera_y_hi
    }

    pub(crate) fn camera_x_coord_scroll_low(&self) -> u16 {
        self.camera_x_low
    }

    pub(crate) fn camera_x_coord_scroll_hi(&self) -> u16 {
        self.camera_x_hi
    }

    pub(crate) fn camera_scroll_low_for_axis(&self, horizontal: bool) -> u16 {
        if horizontal {
            self.camera_x_low
        } else {
            self.camera_y_low
        }
    }

    pub(crate) fn camera_scroll_hi_for_axis(&self, horizontal: bool) -> u16 {
        if horizontal {
            self.camera_x_hi
        } else {
            self.camera_y_hi
        }
    }

    pub(crate) fn up_down_scroll_target(&self, index: usize) -> u16 {
        self.scroll_targets[index]
    }

    pub(crate) fn overworld_scroll_counter_for_axis(&self, ya: usize) -> u16 {
        self.scroll_counters[ya]
    }

    pub(crate) fn spexit_camera_y_scroll_low(&self) -> u16 {
        self.special_exit_camera_y_low
    }

    pub(crate) fn spexit_camera_x_scroll_low(&self) -> u16 {
        self.special_exit_camera_x_low
    }

    pub(crate) fn spexit_room_bound_y_start(&self) -> u16 {
        self.special_exit_room_bounds_y_start
    }

    pub(crate) fn spexit_room_bound_y_end(&self) -> u16 {
        self.special_exit_room_bounds_y_end
    }

    pub(crate) fn spexit_room_bound_x_start(&self) -> u16 {
        self.special_exit_room_bounds_x_start
    }

    pub(crate) fn spexit_room_bound_x_end(&self) -> u16 {
        self.special_exit_room_bounds_x_end
    }

    pub(crate) fn exit_room_bound_y_start(&self) -> u16 {
        self.exit_room_bounds_y_start
    }

    pub(crate) fn exit_room_bound_y_end(&self) -> u16 {
        self.exit_room_bounds_y_end
    }

    pub(crate) fn exit_room_bound_x_start(&self) -> u16 {
        self.exit_room_bounds_x_start
    }

    pub(crate) fn exit_room_bound_x_end(&self) -> u16 {
        self.exit_room_bounds_x_end
    }

    pub(crate) fn set_camera_y_coord_scroll_low(&mut self, value: u16) {
        self.camera_y_low = value;
    }

    pub(crate) fn set_camera_y_coord_scroll_hi(&mut self, value: u16) {
        self.camera_y_hi = value;
    }

    pub(crate) fn set_camera_x_coord_scroll_low(&mut self, value: u16) {
        self.camera_x_low = value;
    }

    pub(crate) fn set_camera_x_coord_scroll_hi(&mut self, value: u16) {
        self.camera_x_hi = value;
    }

    pub(crate) fn add_camera_scroll_for_axis(&mut self, horizontal: bool, delta: i16) -> u16 {
        if horizontal {
            self.camera_x_hi = self.camera_x_hi.wrapping_add_signed(delta);
            self.camera_x_low = self.camera_x_hi.wrapping_add(2);
            self.camera_x_hi
        } else {
            self.camera_y_hi = self.camera_y_hi.wrapping_add_signed(delta);
            self.camera_y_low = self.camera_y_hi.wrapping_add(2);
            self.camera_y_hi
        }
    }

    pub(crate) fn set_camera_scroll_from_link_for_axis(&mut self, horizontal: bool, value: u16) {
        if horizontal {
            self.camera_x_hi = value;
            self.camera_x_low = value.wrapping_add(2);
        } else {
            self.camera_y_hi = value;
            self.camera_y_low = value.wrapping_add(2);
        }
    }

    pub(crate) fn set_scroll_target(&mut self, index: usize, value: u16) {
        self.scroll_targets[index] = value;
    }

    pub(crate) fn set_up_down_scroll_target(&mut self, value: u16) {
        self.set_scroll_target(0, value);
    }

    pub(crate) fn set_up_down_scroll_target_end(&mut self, value: u16) {
        self.set_scroll_target(1, value);
    }

    pub(crate) fn set_left_right_scroll_target(&mut self, value: u16) {
        self.set_scroll_target(2, value);
    }

    pub(crate) fn set_left_right_scroll_target_end(&mut self, value: u16) {
        self.set_scroll_target(3, value);
    }

    pub(crate) fn cache_scroll_targets(&mut self) {
        self.cached_scroll_targets = self.scroll_targets;
    }

    pub(crate) fn cache_camera_scroll(&mut self) {
        self.cached_camera_y_low = self.camera_y_low;
        self.cached_camera_x_low = self.camera_x_low;
    }

    pub(crate) fn restore_scroll_targets_from_cached(&mut self) {
        self.scroll_targets = self.cached_scroll_targets;
    }

    pub(crate) fn set_overworld_scroll_counter_for_axis(&mut self, ya: usize, value: u16) {
        self.scroll_counters[ya] = value;
    }

    pub(crate) fn set_overworld_scroll_up_counter(&mut self, value: u16) {
        self.set_overworld_scroll_counter_for_axis(0, value);
    }

    pub(crate) fn set_overworld_scroll_down_counter(&mut self, value: u16) {
        self.set_overworld_scroll_counter_for_axis(1, value);
    }

    pub(crate) fn set_overworld_scroll_left_counter(&mut self, value: u16) {
        self.set_overworld_scroll_counter_for_axis(2, value);
    }

    pub(crate) fn set_overworld_scroll_right_counter(&mut self, value: u16) {
        self.set_overworld_scroll_counter_for_axis(3, value);
    }

    pub(crate) fn clear_opposed_scroll_counters(&mut self, ya: usize) {
        self.scroll_counters[ya] = 0;
        self.scroll_counters[ya ^ 1] = 0;
    }

    pub(crate) fn set_opposed_scroll_counter_pair(&mut self, ya: usize, value: u16) {
        self.scroll_counters[ya] = value;
        self.scroll_counters[ya ^ 1] = (0u16).wrapping_sub(value);
    }

    pub(crate) fn set_special_exit_room_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        self.special_exit_room_bounds_y_start = y_start;
        self.special_exit_room_bounds_y_end = y_end;
        self.special_exit_room_bounds_x_start = x_start;
        self.special_exit_room_bounds_x_end = x_end;
    }

    pub(crate) fn save_exit_room_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        self.exit_room_bounds_y_start = y_start;
        self.exit_room_bounds_y_end = y_end;
        self.exit_room_bounds_x_start = x_start;
        self.exit_room_bounds_x_end = x_end;
    }

    pub(crate) fn copy_spexit_scroll_targets(&mut self) {
        self.special_exit_scroll_targets = self.scroll_targets;
    }

    pub(crate) fn copy_spexit_scroll_counters(&mut self) {
        self.special_exit_scroll_counters = self.scroll_counters;
    }

    pub(crate) fn restore_spexit_scroll_targets(&mut self) {
        self.scroll_targets = self.special_exit_scroll_targets;
    }

    pub(crate) fn restore_spexit_scroll_counters(&mut self) {
        self.scroll_counters = self.special_exit_scroll_counters;
    }

    pub(crate) fn copy_exit_scroll_targets(&mut self) {
        self.exit_scroll_targets = self.scroll_targets;
    }

    pub(crate) fn copy_exit_scroll_counters(&mut self) {
        self.exit_scroll_counters = self.scroll_counters;
    }

    pub(crate) fn restore_exit_scroll_targets(&mut self) {
        self.scroll_targets = self.exit_scroll_targets;
    }

    pub(crate) fn restore_exit_scroll_counters(&mut self) {
        self.scroll_counters = self.exit_scroll_counters;
    }

    pub(crate) fn save_spexit_camera_coords(&mut self) {
        self.special_exit_camera_y_low = self.camera_y_low;
        self.special_exit_camera_x_low = self.camera_x_low;
    }

    pub(crate) fn save_exit_camera_coords(&mut self) {
        self.exit_camera_y_low = self.camera_y_low;
        self.exit_camera_x_low = self.camera_x_low;
    }

    pub(crate) fn restore_exit_camera_scroll(&mut self) {
        self.camera_y_low = self.exit_camera_y_low;
        self.camera_y_hi = self.exit_camera_y_low.wrapping_sub(2);
        self.camera_x_low = self.exit_camera_x_low;
        self.camera_x_hi = self.exit_camera_x_low.wrapping_sub(2);
    }

    pub(crate) fn restore_special_exit_camera_scroll(&mut self) {
        self.camera_y_low = self.special_exit_camera_y_low;
        self.camera_y_hi = self.special_exit_camera_y_low.wrapping_sub(2);
        self.camera_x_low = self.special_exit_camera_x_low;
        self.camera_x_hi = self.special_exit_camera_x_low.wrapping_sub(2);
    }

    pub(crate) fn restore_camera_y_from_cached_indoor(&mut self) {
        self.camera_y_low = self.cached_camera_y_low;
        self.camera_y_hi = self.cached_camera_y_low.wrapping_add(2);
    }

    pub(crate) fn restore_camera_x_from_cached_indoor(&mut self) {
        self.camera_x_low = self.cached_camera_x_low;
        self.camera_x_hi = self.cached_camera_x_low.wrapping_add(2);
    }

    pub(crate) fn update_camera_hi_outdoor(&mut self) {
        self.camera_y_hi = self.camera_y_low.wrapping_sub(2);
        self.camera_x_hi = self.camera_x_low.wrapping_sub(2);
    }
}

fn read_scroll_targets(ram: &[u8], base: usize) -> [u16; SCROLL_TARGET_COUNT] {
    [
        read_le_u16(ram, base),
        read_le_u16(ram, base + 2),
        read_le_u16(ram, base + 4),
        read_le_u16(ram, base + 6),
    ]
}

fn write_scroll_targets(ram: &mut [u8], base: usize, targets: [u16; SCROLL_TARGET_COUNT]) {
    for (index, value) in targets.iter().enumerate() {
        write_le_u16(ram, base + index * 2, *value);
    }
}

fn read_scroll_counters(ram: &[u8], base: usize) -> [u16; SCROLL_COUNTER_COUNT] {
    [
        read_le_u16(ram, base),
        read_le_u16(ram, base + 2),
        read_le_u16(ram, base + 4),
        read_le_u16(ram, base + 6),
    ]
}

fn write_scroll_counters(ram: &mut [u8], base: usize, counters: [u16; SCROLL_COUNTER_COUNT]) {
    for (index, value) in counters.iter().enumerate() {
        write_le_u16(ram, base + index * 2, *value);
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
        // NOTE: OVERWORLD/MAIN/AUX_TILE_THEME_INDEX_EXIT (0xc164-0xc166) are owned
        // and projected by DungeonEntranceBackupState (the `Dungeon_LoadEntrance`
        // save, matching C dungeon.c:8475). These fields are a load-only mirror
        // read by `restore_exit_tile_themes`; projecting them here would clobber
        // the authoritative save (and cascade into the sprite byte 0xc167).
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

    pub(crate) fn set_last_light_vs_dark_world(&mut self, value: u8) {
        self.last_light_vs_dark_world = value;
    }

    pub(crate) fn set_aux_bg_subset(&mut self, index: usize, value: u8) {
        if let Some(subset) = self.aux_bg_subset.get_mut(index) {
            *subset = value;
        }
    }

    pub(crate) fn set_overworld_palette_aux1_hi(&mut self, value: u8) {
        self.overworld_palette_aux1_hi = value;
    }

    pub(crate) fn set_hud_palette(&mut self, value: u8) {
        self.hud_palette = value;
    }

    pub(crate) fn set_overworld_tile_theme_index(&mut self, value: u8) {
        self.overworld_tile_theme_index = value;
    }

    pub(crate) fn set_main_tile_theme_index(&mut self, value: u8) {
        self.main_tile_theme_index = value;
    }

    pub(crate) fn set_aux_tile_theme_index(&mut self, value: u8) {
        self.aux_tile_theme_index = value;
    }

    pub(crate) fn set_misc_sprites_graphics_index(&mut self, value: u8) {
        self.misc_sprites_graphics_index = value;
    }

    pub(crate) fn set_palette_sp6r_indoors(&mut self, value: u8) {
        self.palette_sp6r_indoors = value;
    }

    pub(crate) fn sync_shared_palette_aliases_from_ram(
        &mut self,
        ram: &[u8],
        preserve_hud_palette: bool,
        preserve_sp6r: bool,
    ) {
        self.overworld_palette_mode = ram_byte(ram, OVERWORLD_PALETTE_MODE);
        self.palette_main_indoors = ram_byte(ram, PALETTE_MAIN_INDOORS);
        self.palette_sp0l = ram_byte(ram, PALETTE_SP0L);
        self.palette_sp5l = ram_byte(ram, PALETTE_SP5L);
        self.palette_sp6l = ram_byte(ram, PALETTE_SP6L);
        self.overworld_palette_aux2_hi = ram_byte(ram, OVERWORLD_PALETTE_AUX2_BP5TO7_HI);
        self.overworld_palette_aux3_lo = ram_byte(ram, OVERWORLD_PALETTE_AUX3_BP7_LO);
        if preserve_hud_palette {
            self.hud_palette = ram_byte(ram, HUD_PALETTE);
        }
        if preserve_sp6r {
            self.palette_sp6r_indoors = ram_byte(ram, PALETTE_SP6R_INDOORS);
        }
    }

    pub(crate) fn restore_exit_tile_themes(&mut self) {
        self.overworld_tile_theme_index = self.exit_overworld_tile_theme_index;
        self.main_tile_theme_index = self.exit_main_tile_theme_index;
        self.aux_tile_theme_index = self.exit_aux_tile_theme_index;
    }

    pub(crate) fn save_special_exit_tile_themes(&mut self) {
        self.special_exit_overworld_tile_theme_index = self.overworld_tile_theme_index;
        self.special_exit_main_tile_theme_index = self.main_tile_theme_index;
        self.special_exit_aux_tile_theme_index = self.aux_tile_theme_index;
    }

    pub(crate) fn restore_special_exit_tile_themes(&mut self) {
        self.overworld_tile_theme_index = self.special_exit_overworld_tile_theme_index;
        self.main_tile_theme_index = self.special_exit_main_tile_theme_index;
        self.aux_tile_theme_index = self.special_exit_aux_tile_theme_index;
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

    pub(crate) fn set_countdown(&mut self, value: u16) {
        self.countdown = value;
    }

    pub(crate) fn set_music_latch(&mut self, value: u8) {
        self.music_latch = value;
    }

    pub(crate) fn set_source_slot(&mut self, value: u8) {
        self.source_slot = value;
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
        let mut sprite_graphics = vec![0; OVERWORLD_SPRITE_GFX_SCREENS];
        for screen in 0..OVERWORLD_CONFIG_SCREENS {
            music[screen] = ram_byte(ram, OVERWORLD_MUSIC_TABLE + screen);
            sprite_palette[screen] = ram_byte(ram, OVERWORLD_SPRITE_PALETTE_TABLE + screen);
            if screen < OVERWORLD_SPRITE_GFX_SCREENS {
                sprite_graphics[screen] = ram_byte(ram, OVERWORLD_SPRITE_GFX_TABLE + screen);
            }
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
            if screen < OVERWORLD_SPRITE_GFX_SCREENS {
                ram[OVERWORLD_SPRITE_GFX_TABLE + screen] = self.sprite_graphics(screen);
            }
        }
    }

    pub(crate) fn music(&self, screen: usize) -> u8 {
        self.music.get(screen).copied().unwrap_or_default()
    }

    pub(crate) fn sprite_palette(&self, screen: usize) -> u8 {
        self.sprite_palette.get(screen).copied().unwrap_or_default()
    }

    pub(crate) fn sprite_graphics(&self, screen: usize) -> u8 {
        if screen < OVERWORLD_SPRITE_GFX_SCREENS {
            self.sprite_graphics
                .get(screen)
                .copied()
                .unwrap_or_default()
        } else {
            self.sprite_palette(screen - OVERWORLD_SPRITE_GFX_SCREENS)
        }
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

    pub(crate) fn copy_sprite_graphics_range(
        &mut self,
        dst: usize,
        data: &[u8],
        src: usize,
        len: usize,
    ) {
        self.sprite_graphics[dst..dst + len].copy_from_slice(&data[src..src + len]);
    }

    pub(crate) fn copy_sprite_palette_range(
        &mut self,
        dst: usize,
        data: &[u8],
        src: usize,
        len: usize,
    ) {
        self.sprite_palette[dst..dst + len].copy_from_slice(&data[src..src + len]);
    }
}

impl Default for OverworldConfigTableState {
    fn default() -> Self {
        Self {
            music: vec![0; OVERWORLD_CONFIG_SCREENS],
            sprite_palette: vec![0; OVERWORLD_CONFIG_SCREENS],
            sprite_graphics: vec![0; OVERWORLD_SPRITE_GFX_SCREENS],
        }
    }
}

pub(crate) struct OverworldConfigTableRead<'a> {
    config_table: &'a OverworldConfigTableState,
    current_screen: usize,
}

impl<'a> OverworldConfigTableRead<'a> {
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

    pub(crate) fn set_map_state(&mut self, value: u8) {
        self.map_state = (self.map_state & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_map_state_word(&mut self, value: u16) {
        self.map_state = value;
    }

    pub(crate) fn increment_map_state(&mut self) {
        self.set_map_state(self.map_state().wrapping_add(1));
    }

    pub(crate) fn set_map_flags(&mut self, value: u8) {
        self.map_flags = value;
    }

    pub(crate) fn and_map_flags(&mut self, value: u8) {
        self.map_flags &= value;
    }

    pub(crate) fn or_map_flags(&mut self, value: u8) {
        self.map_flags |= value;
    }

    pub(crate) fn set_birdtravel_status(&mut self, value: u8) {
        self.bird_travel_statuses.set_status(0, value);
    }

    pub(crate) fn set_birdtravel_status_word(&mut self, value: u16) {
        self.bird_travel_statuses.set_status_word(value);
    }

    pub(crate) fn and_birdtravel_status(&mut self, value: u8) {
        self.set_birdtravel_status(self.birdtravel_status() & value);
    }

    pub(crate) fn decrement_birdtravel_status(&mut self) {
        self.set_birdtravel_status(self.birdtravel_status().wrapping_sub(1));
    }

    pub(crate) fn increment_birdtravel_status(&mut self) {
        self.set_birdtravel_status(self.birdtravel_status().wrapping_add(1));
    }

    pub(crate) fn clear_bird_travel_stop_status(&mut self, slot: usize) {
        self.bird_travel_statuses.clear_status(slot);
    }

    pub(crate) fn increment_bird_travel_stop_status(&mut self, slot: usize) {
        self.bird_travel_statuses.increment_status(slot);
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

    pub(crate) fn set_step_counter(&mut self, value: u8) {
        self.step_counter = value;
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.timer = value;
    }

    pub(crate) fn decrement_timer(&mut self) -> u8 {
        self.timer = self.timer.wrapping_sub(1);
        self.timer
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

    pub(crate) fn clear_big_area_high(&mut self) {
        self.big_area &= 0x00ff;
    }

    pub(crate) fn set_big_area_low(&mut self, value: u8) {
        self.big_area = (self.big_area & 0xff00) | u16::from(value);
    }

    pub(crate) fn backup_big_area_low(&mut self) {
        self.big_area_backup = self.big_area as u8;
    }

    pub(crate) fn set_right_bottom_bound_low(&mut self, value: u8) {
        self.right_bottom_scroll_bound =
            (self.right_bottom_scroll_bound & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_right_bottom_bound_high(&mut self, value: u8) {
        self.right_bottom_scroll_bound =
            (self.right_bottom_scroll_bound & 0x00ff) | (u16::from(value) << 8);
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

    pub(crate) fn set_special_entrance_trigger(&mut self, value: u8) {
        self.special_entrance_trigger = value;
    }

    pub(crate) fn clear_special_entrance_trigger(&mut self) {
        self.set_special_entrance_trigger(0);
    }

    pub(crate) fn set_sequence_counter(&mut self, value: u8) {
        self.sequence_counter = value;
    }

    pub(crate) fn clear_sequence_counter(&mut self) {
        self.set_sequence_counter(0);
    }

    pub(crate) fn increment_sequence_counter(&mut self) -> u8 {
        self.sequence_counter = self.sequence_counter.wrapping_add(1);
        self.sequence_counter
    }

    pub(crate) fn decrement_sequence_counter(&mut self) -> u8 {
        self.sequence_counter = self.sequence_counter.wrapping_sub(1);
        self.sequence_counter
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

    pub(crate) fn set_exit_screen(&mut self, value: u16) {
        self.exit_screen = value;
    }

    pub(crate) fn set_special_exit_screen(&mut self, value: u16) {
        self.special_exit_screen = value;
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

    pub(crate) fn set_direction_bits(&mut self, value: u8) {
        self.direction_bits = (self.direction_bits & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_direction_bits_word(&mut self, value: u16) {
        self.direction_bits = value;
    }

    pub(crate) fn clear_direction_bits(&mut self) {
        self.set_direction_bits(0);
    }

    pub(crate) fn clear_direction_bits_word(&mut self) {
        self.set_direction_bits_word(0);
    }

    pub(crate) fn and_direction_bits(&mut self, value: u8) {
        self.set_direction_bits(self.direction_bits() & value);
    }

    pub(crate) fn or_direction_bits(&mut self, value: u8) {
        self.set_direction_bits(self.direction_bits() | value);
    }

    pub(crate) fn or_direction_bits_word(&mut self, value: u16) -> u16 {
        let next = self.direction_bits_word() | value;
        self.set_direction_bits_word(next);
        next
    }

    pub(crate) fn set_edge_direction_bits(&mut self, value: u8) {
        self.edge_direction_bits = (self.edge_direction_bits & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_edge_direction_bits_word(&mut self, value: u16) {
        self.edge_direction_bits = value;
    }

    pub(crate) fn clear_edge_direction_bits(&mut self) {
        self.set_edge_direction_bits(0);
    }

    pub(crate) fn set_direction_enum(&mut self, value: u8) {
        self.direction_enum = value;
    }

    pub(crate) fn set_screen_transition(&mut self, value: u8) {
        self.screen_transition = (self.screen_transition & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_screen_transition_word(&mut self, value: u16) {
        self.screen_transition = value;
    }

    pub(crate) fn clear_screen_transition(&mut self) {
        self.set_screen_transition(0);
    }

    pub(crate) fn set_transition_counter(&mut self, value: u8) {
        self.transition_counter = value;
    }

    pub(crate) fn increment_transition_counter(&mut self) -> u8 {
        self.transition_counter = self.transition_counter.wrapping_add(1);
        self.transition_counter
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.countdown = value;
    }

    pub(crate) fn decrement_countdown(&mut self) -> u8 {
        self.countdown = self.countdown.wrapping_sub(1);
        self.countdown
    }

    pub(crate) fn save_previous_direction_bits(&mut self) {
        self.previous_direction_bits = u16::from(self.edge_direction_bits());
        self.previous_direction_bits2 = self.direction_bits_word();
    }

    pub(crate) fn restore_previous_direction_bits(&mut self) {
        self.set_edge_direction_bits_word(self.previous_direction_bits);
        self.set_direction_bits_word(self.previous_direction_bits2);
    }

    pub(crate) fn set_previous_screen_transition(&mut self, value: u8) {
        self.previous_screen_transition = value;
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorldRegionState {
    pub(crate) current_area_of_player: u16,
    pub(crate) overworld_area_index: u16,
    pub(crate) special_exit_area_index: u16,
    pub(crate) exit_area_index: u16,
    pub(crate) previous_screen_index: u16,
    pub(crate) overlay_index: u16,
    pub(crate) rng_seed: u8,
    pub(crate) dark_world_region_index: u8,
    pub(crate) area_changed_flag: u8,
    pub(crate) entrance_id: u16,
    pub(crate) overworld_entrance_value: u16,
}

impl WorldRegionState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            current_area_of_player: read_le_u16(ram, CURRENT_AREA_OF_PLAYER),
            overworld_area_index: read_le_u16(ram, OVERWORLD_AREA_INDEX),
            special_exit_area_index: read_le_u16(ram, OVERWORLD_AREA_INDEX_SPEXIT),
            exit_area_index: read_le_u16(ram, OVERWORLD_AREA_INDEX_EXIT),
            previous_screen_index: read_le_u16(ram, OVERWORLD_SCREEN_INDEX_PREV),
            overlay_index: read_le_u16(ram, OVERLAY_INDEX),
            rng_seed: ram_byte(ram, RNG_SEED),
            dark_world_region_index: ram_byte(ram, IS_IN_DARK_WORLD_FLAG),
            area_changed_flag: ram_byte(ram, FLAG_OVERWORLD_AREA_CHANGED),
            entrance_id: read_le_u16(ram, WHICH_ENTRANCE),
            overworld_entrance_value: read_le_u16(ram, OW_ENTRANCE_VALUE),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, CURRENT_AREA_OF_PLAYER, self.current_area_of_player);
        write_le_u16(ram, OVERWORLD_AREA_INDEX, self.overworld_area_index);
        write_le_u16(
            ram,
            OVERWORLD_AREA_INDEX_SPEXIT,
            self.special_exit_area_index,
        );
        write_le_u16(ram, OVERWORLD_AREA_INDEX_EXIT, self.exit_area_index);
        write_le_u16(ram, OVERWORLD_SCREEN_INDEX_PREV, self.previous_screen_index);
        write_le_u16(ram, OVERLAY_INDEX, self.overlay_index);
        ram[RNG_SEED] = self.rng_seed;
        ram[IS_IN_DARK_WORLD_FLAG] = self.dark_world_region_index;
        ram[FLAG_OVERWORLD_AREA_CHANGED] = self.area_changed_flag;
        write_le_u16(ram, WHICH_ENTRANCE, self.entrance_id);
        write_le_u16(ram, OW_ENTRANCE_VALUE, self.overworld_entrance_value);
    }

    pub(crate) fn current_area_of_player(&self) -> u8 {
        self.current_area_of_player as u8
    }

    pub(crate) fn current_area_of_player_word(&self) -> u16 {
        self.current_area_of_player
    }

    pub(crate) fn overworld_area(&self) -> u16 {
        self.overworld_area_index
    }

    pub(crate) fn overworld_area_low(&self) -> u8 {
        self.overworld_area_index as u8
    }

    pub(crate) fn overworld_area_index(&self) -> u16 {
        self.overworld_area_index
    }

    pub(crate) fn overworld_area_index_word(&self) -> u16 {
        self.overworld_area_index
    }

    pub(crate) fn spexit_area_index(&self) -> u16 {
        self.special_exit_area_index
    }

    pub(crate) fn prev_screen_index_word(&self) -> u16 {
        self.previous_screen_index
    }

    pub(crate) fn prev_screen_index_byte(&self) -> u8 {
        self.previous_screen_index as u8
    }

    pub(crate) fn overlay_index(&self) -> u8 {
        self.overlay_index as u8
    }

    pub(crate) fn rng_seed(&self) -> u8 {
        self.rng_seed
    }

    pub(crate) fn dark_world_region_index(&self) -> u8 {
        self.dark_world_region_index
    }

    pub(crate) fn is_in_dark_world(&self) -> bool {
        self.dark_world_region_index != 0
    }

    pub(crate) fn flag_overworld_area_changed(&self) -> bool {
        self.area_changed_flag != 0
    }

    pub(crate) fn which_entrance(&self) -> u16 {
        self.entrance_id
    }

    pub(crate) fn ow_entrance_value(&self) -> u16 {
        self.overworld_entrance_value
    }

    pub(crate) fn set_rng_seed(&mut self, value: u8) {
        self.rng_seed = value;
    }

    pub(crate) fn set_dark_world_region_index(&mut self, value: u8) {
        self.dark_world_region_index = value;
    }

    pub(crate) fn set_which_entrance(&mut self, value: u16) {
        self.entrance_id = value;
    }

    pub(crate) fn set_which_entrance_byte(&mut self, value: u8) {
        self.entrance_id = (self.entrance_id & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_overworld_area_index(&mut self, value: u8) {
        self.overworld_area_index = (self.overworld_area_index & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_overworld_area_index_word(&mut self, value: u16) {
        self.overworld_area_index = value;
    }

    pub(crate) fn set_current_area_of_player_word(&mut self, value: u16) {
        self.current_area_of_player = value;
    }

    pub(crate) fn set_flag_overworld_area_changed(&mut self, value: u8) {
        self.area_changed_flag = value;
    }

    pub(crate) fn clear_flag_overworld_area_changed(&mut self) {
        self.area_changed_flag = 0;
    }

    pub(crate) fn clear_overlay_index_word(&mut self) {
        self.overlay_index = 0;
    }

    pub(crate) fn set_overlay_index_word(&mut self, value: u16) {
        self.overlay_index = value;
    }

    pub(crate) fn set_overlay_high(&mut self, value: u8) {
        self.overlay_index = (self.overlay_index & 0x00ff) | (u16::from(value) << 8);
    }

    pub(crate) fn set_prev_screen_index_word(&mut self, value: u16) {
        self.previous_screen_index = value;
    }

    pub(crate) fn save_spexit_area_index(&mut self) {
        self.special_exit_area_index = self.overworld_area_index;
    }

    pub(crate) fn restore_spexit_area_index(&mut self) {
        self.overworld_area_index = self.special_exit_area_index;
    }

    pub(crate) fn save_exit_area_index(&mut self) {
        self.exit_area_index = self.overworld_area_index;
    }

    pub(crate) fn restore_exit_area_index(&mut self) {
        self.overworld_area_index = self.exit_area_index;
    }

    pub(crate) fn set_ow_entrance_value(&mut self, value: u16) {
        self.overworld_entrance_value = value;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorldTransientState {
    pub(crate) custom_spell_animation_flag: u8,
    pub(crate) allow_scroll_z: u8,
    pub(crate) milestone_item_graphics_countdown: u8,
    pub(crate) big_key_door_message_triggered: u16,
    pub(crate) savegame_master_sword_flags: u16,
    pub(crate) super_bomb_indicator_timer: u8,
    pub(crate) standing_in_doorway_cached: u8,
    pub(crate) cached_room_bounds_y_start: u16,
    pub(crate) cached_room_bounds_y_end: u16,
    pub(crate) cached_room_bounds_x_start: u16,
    pub(crate) cached_room_bounds_x_end: u16,
    pub(crate) overworld_peg_puzzle_progress: u16,
    pub(crate) overworld_hole_tilemap_position: u8,
    pub(crate) overworld_bomb_tile_sweep_x: u16,
    pub(crate) overworld_bomb_tile_sweep_y_end: u16,
    pub(crate) hud_current_item_x: u8,
    pub(crate) door_animation_step: u16,
    pub(crate) room_transitioning_flags: u8,
    pub(crate) travel_bird_flag: u8,
    pub(crate) tile_interaction_shared_flag: u8,
    pub(crate) hud_floor_changed_timer: u8,
    pub(crate) quadrant_fullsize_x: u8,
    pub(crate) quadrant_fullsize_y: u8,
    pub(crate) cached_quadrant_fullsize_x: u8,
    pub(crate) cached_quadrant_fullsize_y: u8,
    pub(crate) tilemap_layer_copy: u16,
    pub(crate) special_exit_tilemap_layer_copy: u16,
    pub(crate) exit_tilemap_layer_copy: u16,
    pub(crate) map_backup_main_layer: u8,
    pub(crate) map_backup_subscreen_layer: u8,
    pub(crate) move_overlay_counter: u8,
    pub(crate) overworld_hole_scan_step: u8,
    pub(crate) dungeon_replacement_tiles: Vec<u16>,
}

impl Default for WorldTransientState {
    fn default() -> Self {
        Self {
            custom_spell_animation_flag: 0,
            allow_scroll_z: 0,
            milestone_item_graphics_countdown: 0,
            big_key_door_message_triggered: 0,
            savegame_master_sword_flags: 0,
            super_bomb_indicator_timer: 0,
            standing_in_doorway_cached: 0,
            cached_room_bounds_y_start: 0,
            cached_room_bounds_y_end: 0,
            cached_room_bounds_x_start: 0,
            cached_room_bounds_x_end: 0,
            overworld_peg_puzzle_progress: 0,
            overworld_hole_tilemap_position: 0,
            overworld_bomb_tile_sweep_x: 0,
            overworld_bomb_tile_sweep_y_end: 0,
            hud_current_item_x: 0,
            door_animation_step: 0,
            room_transitioning_flags: 0,
            travel_bird_flag: 0,
            tile_interaction_shared_flag: 0,
            hud_floor_changed_timer: 0,
            quadrant_fullsize_x: 0,
            quadrant_fullsize_y: 0,
            cached_quadrant_fullsize_x: 0,
            cached_quadrant_fullsize_y: 0,
            tilemap_layer_copy: 0,
            special_exit_tilemap_layer_copy: 0,
            exit_tilemap_layer_copy: 0,
            map_backup_main_layer: 0,
            map_backup_subscreen_layer: 0,
            move_overlay_counter: 0,
            overworld_hole_scan_step: 0,
            dungeon_replacement_tiles: vec![0; DUNGEON_REPLACEMENT_TILE_WORDS],
        }
    }
}

impl WorldTransientState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut dungeon_replacement_tiles = vec![0; DUNGEON_REPLACEMENT_TILE_WORDS];
        for (index, tile) in dungeon_replacement_tiles.iter_mut().enumerate() {
            *tile = read_le_u16(ram, DUNG_REPLACEMENT_TILE_STATE + index * 2);
        }
        Self {
            custom_spell_animation_flag: ram_byte(ram, FLAG_CUSTOM_SPELL_ANIM_ACTIVE),
            allow_scroll_z: ram_byte(ram, ALLOW_SCROLL_Z),
            milestone_item_graphics_countdown: ram_byte(ram, MILESTONE_ITEM_GFX_SWAP_COUNTDOWN),
            big_key_door_message_triggered: read_le_u16(ram, BIG_KEY_DOOR_MESSAGE_TRIGGERED),
            savegame_master_sword_flags: read_le_u16(ram, SAVEGAME_HAS_MASTER_SWORD_FLAGS),
            super_bomb_indicator_timer: ram_byte(ram, SUPER_BOMB_INDICATOR_TIMER),
            standing_in_doorway_cached: ram_byte(ram, IS_STANDING_IN_DOORWAY_CACHED),
            cached_room_bounds_y_start: read_le_u16(ram, CACHED_ROOM_BOUNDS_Y_START),
            cached_room_bounds_y_end: read_le_u16(ram, CACHED_ROOM_BOUNDS_Y_END),
            cached_room_bounds_x_start: read_le_u16(ram, CACHED_ROOM_BOUNDS_X_START),
            cached_room_bounds_x_end: read_le_u16(ram, CACHED_ROOM_BOUNDS_X_END),
            overworld_peg_puzzle_progress: read_le_u16(ram, OVERWORLD_PEG_PUZZLE_PROGRESS),
            overworld_hole_tilemap_position: ram_byte(ram, OVERWORLD_HOLE_TILEMAP_POS),
            overworld_bomb_tile_sweep_x: read_le_u16(ram, OVERWORLD_BOMB_TILE_SWEEP_X),
            overworld_bomb_tile_sweep_y_end: read_le_u16(ram, OVERWORLD_BOMB_TILE_SWEEP_Y_END),
            hud_current_item_x: ram_byte(ram, HUD_CUR_ITEM_X),
            door_animation_step: read_le_u16(ram, DOOR_ANIMATION_STEP_INDICATOR),
            room_transitioning_flags: ram_byte(ram, ROOM_TRANSITIONING_FLAGS),
            travel_bird_flag: ram_byte(ram, FLAG_TRAVEL_BIRD),
            tile_interaction_shared_flag: ram_byte(ram, TILE_INTERACTION_SHARED_FLAG),
            hud_floor_changed_timer: ram_byte(ram, HUD_FLOOR_CHANGED_TIMER),
            quadrant_fullsize_x: ram_byte(ram, QUADRANT_FULLSIZE_X),
            quadrant_fullsize_y: ram_byte(ram, QUADRANT_FULLSIZE_Y),
            cached_quadrant_fullsize_x: ram_byte(ram, QUADRANT_FULLSIZE_X_CACHED),
            cached_quadrant_fullsize_y: ram_byte(ram, QUADRANT_FULLSIZE_X_CACHED + 1),
            tilemap_layer_copy: read_le_u16(ram, TM_COPY),
            special_exit_tilemap_layer_copy: read_le_u16(ram, TM_COPY_SPEXIT),
            exit_tilemap_layer_copy: read_le_u16(ram, TM_COPY_EXIT),
            map_backup_main_layer: ram_byte(ram, MAPBAK_TM),
            map_backup_subscreen_layer: ram_byte(ram, MAPBAK_TS),
            move_overlay_counter: ram_byte(ram, MOVE_OVERLAY_CTR),
            overworld_hole_scan_step: ram_byte(ram, OVERWORLD_HOLE_SCAN_STEP),
            dungeon_replacement_tiles,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (index, tile) in self.dungeon_replacement_tiles.iter().enumerate() {
            write_le_u16(ram, DUNG_REPLACEMENT_TILE_STATE + index * 2, *tile);
        }
        self.write_scalar_fields_to_ram(ram);
    }

    pub(crate) fn write_scalar_fields_to_ram(&self, ram: &mut [u8]) {
        ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = self.custom_spell_animation_flag;
        ram[ALLOW_SCROLL_Z] = self.allow_scroll_z;
        ram[MILESTONE_ITEM_GFX_SWAP_COUNTDOWN] = self.milestone_item_graphics_countdown;
        write_le_u16(
            ram,
            BIG_KEY_DOOR_MESSAGE_TRIGGERED,
            self.big_key_door_message_triggered,
        );
        write_le_u16(
            ram,
            SAVEGAME_HAS_MASTER_SWORD_FLAGS,
            self.savegame_master_sword_flags,
        );
        ram[SUPER_BOMB_INDICATOR_TIMER] = self.super_bomb_indicator_timer;
        ram[IS_STANDING_IN_DOORWAY_CACHED] = self.standing_in_doorway_cached;
        write_le_u16(
            ram,
            CACHED_ROOM_BOUNDS_Y_START,
            self.cached_room_bounds_y_start,
        );
        write_le_u16(ram, CACHED_ROOM_BOUNDS_Y_END, self.cached_room_bounds_y_end);
        write_le_u16(
            ram,
            CACHED_ROOM_BOUNDS_X_START,
            self.cached_room_bounds_x_start,
        );
        write_le_u16(ram, CACHED_ROOM_BOUNDS_X_END, self.cached_room_bounds_x_end);
        write_le_u16(
            ram,
            OVERWORLD_PEG_PUZZLE_PROGRESS,
            self.overworld_peg_puzzle_progress,
        );
        ram[OVERWORLD_HOLE_TILEMAP_POS] = self.overworld_hole_tilemap_position;
        write_le_u16(
            ram,
            OVERWORLD_BOMB_TILE_SWEEP_X,
            self.overworld_bomb_tile_sweep_x,
        );
        write_le_u16(
            ram,
            OVERWORLD_BOMB_TILE_SWEEP_Y_END,
            self.overworld_bomb_tile_sweep_y_end,
        );
        ram[HUD_CUR_ITEM_X] = self.hud_current_item_x;
        write_le_u16(ram, DOOR_ANIMATION_STEP_INDICATOR, self.door_animation_step);
        ram[ROOM_TRANSITIONING_FLAGS] = self.room_transitioning_flags;
        ram[FLAG_TRAVEL_BIRD] = self.travel_bird_flag;
        ram[TILE_INTERACTION_SHARED_FLAG] = self.tile_interaction_shared_flag;
        ram[HUD_FLOOR_CHANGED_TIMER] = self.hud_floor_changed_timer;
        ram[QUADRANT_FULLSIZE_X] = self.quadrant_fullsize_x;
        ram[QUADRANT_FULLSIZE_Y] = self.quadrant_fullsize_y;
        ram[QUADRANT_FULLSIZE_X_CACHED] = self.cached_quadrant_fullsize_x;
        ram[QUADRANT_FULLSIZE_X_CACHED + 1] = self.cached_quadrant_fullsize_y;
        write_le_u16(ram, TM_COPY, self.tilemap_layer_copy);
        write_le_u16(ram, TM_COPY_SPEXIT, self.special_exit_tilemap_layer_copy);
        write_le_u16(ram, TM_COPY_EXIT, self.exit_tilemap_layer_copy);
        ram[MAPBAK_TM] = self.map_backup_main_layer;
        ram[MAPBAK_TS] = self.map_backup_subscreen_layer;
        ram[MOVE_OVERLAY_CTR] = self.move_overlay_counter;
        ram[OVERWORLD_HOLE_SCAN_STEP] = self.overworld_hole_scan_step;
    }

    pub(crate) fn flag_custom_spell_anim_active(&self) -> u8 {
        self.custom_spell_animation_flag
    }

    pub(crate) fn allow_scroll_z(&self) -> u8 {
        self.allow_scroll_z
    }

    pub(crate) fn milestone_item_gfx_swap_countdown(&self) -> u8 {
        self.milestone_item_graphics_countdown
    }

    pub(crate) fn big_key_door_message_triggered(&self) -> u16 {
        self.big_key_door_message_triggered
    }

    pub(crate) fn savegame_has_master_sword_flags(&self) -> u16 {
        self.savegame_master_sword_flags
    }

    pub(crate) fn super_bomb_indicator_timer(&self) -> u8 {
        self.super_bomb_indicator_timer
    }

    pub(crate) fn is_standing_in_doorway_cached(&self) -> u8 {
        self.standing_in_doorway_cached
    }

    pub(crate) fn cached_room_bounds_y_start(&self) -> u16 {
        self.cached_room_bounds_y_start
    }

    pub(crate) fn cached_room_bounds_y_end(&self) -> u16 {
        self.cached_room_bounds_y_end
    }

    pub(crate) fn cached_room_bounds_x_start(&self) -> u16 {
        self.cached_room_bounds_x_start
    }

    pub(crate) fn cached_room_bounds_x_end(&self) -> u16 {
        self.cached_room_bounds_x_end
    }

    pub(crate) fn room_transitioning_flags(&self) -> u8 {
        self.room_transitioning_flags
    }

    pub(crate) fn overworld_peg_puzzle_progress(&self) -> u16 {
        self.overworld_peg_puzzle_progress
    }

    pub(crate) fn overworld_hole_tilemap_pos(&self) -> u8 {
        self.overworld_hole_tilemap_position
    }

    pub(crate) fn hud_cur_item_x(&self) -> u8 {
        self.hud_current_item_x
    }

    pub(crate) fn door_animation_step(&self) -> u16 {
        self.door_animation_step
    }

    pub(crate) fn quadrant_fullsize_x(&self) -> u8 {
        self.quadrant_fullsize_x
    }

    pub(crate) fn quadrant_fullsize_y(&self) -> u8 {
        self.quadrant_fullsize_y
    }

    pub(crate) fn horizontal_room_bounds_base_index(&self) -> usize {
        (self.quadrant_fullsize_x >> 1) as usize
    }

    pub(crate) fn vertical_room_bounds_base_index(&self) -> usize {
        (self.quadrant_fullsize_y >> 1) as usize
    }

    pub(crate) fn dungeon_quadrant_visit_index(
        &self,
        player_quadrant_y: u8,
        player_quadrant_x: u8,
    ) -> usize {
        ((self.quadrant_fullsize_y as usize) << 2)
            + ((self.quadrant_fullsize_x as usize) << 1)
            + player_quadrant_y as usize
            + player_quadrant_x as usize
    }

    pub(crate) fn dung_replacement_tile_state(&self, index: usize) -> u16 {
        self.dungeon_replacement_tiles[index]
    }

    pub(crate) fn set_room_transitioning_flags(&mut self, value: u8) {
        self.room_transitioning_flags = value;
    }

    pub(crate) fn clear_custom_spell_animation(&mut self) {
        self.custom_spell_animation_flag = 0;
    }

    pub(crate) fn set_custom_spell_animation_active(&mut self) {
        self.custom_spell_animation_flag = 1;
    }

    pub(crate) fn set_allow_scroll_z(&mut self, value: u8) {
        self.allow_scroll_z = value;
    }

    pub(crate) fn set_cached_room_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        self.cached_room_bounds_y_start = y_start;
        self.cached_room_bounds_y_end = y_end;
        self.cached_room_bounds_x_start = x_start;
        self.cached_room_bounds_x_end = x_end;
    }

    pub(crate) fn set_standing_in_doorway_cached(&mut self, value: u8) {
        self.standing_in_doorway_cached = value;
    }

    pub(crate) fn set_flag_travel_bird(&mut self, value: u8) {
        self.travel_bird_flag = value;
    }

    pub(crate) fn clear_tile_interaction_shared_flag(&mut self) {
        self.tile_interaction_shared_flag = 0;
    }

    pub(crate) fn set_door_animation_step(&mut self, value: u8) {
        self.set_door_animation_step_word((self.door_animation_step & 0xff00) | u16::from(value));
    }

    pub(crate) fn set_door_animation_step_word(&mut self, value: u16) {
        self.door_animation_step = value;
        self.dungeon_replacement_tiles[DOOR_ANIMATION_REPLACEMENT_TILE_INDEX] = value;
    }

    pub(crate) fn clear_hud_floor_changed_timer(&mut self) {
        self.hud_floor_changed_timer = 0;
    }

    pub(crate) fn cache_quadrant_fullsize_state(&mut self) {
        self.cached_quadrant_fullsize_x = self.quadrant_fullsize_x;
        self.cached_quadrant_fullsize_y = self.quadrant_fullsize_y;
    }

    pub(crate) fn restore_quadrant_fullsize_from_cached(&mut self) {
        self.quadrant_fullsize_x = self.cached_quadrant_fullsize_x;
        self.quadrant_fullsize_y = self.cached_quadrant_fullsize_y;
    }

    pub(crate) fn set_quadrant_fullsize_x(&mut self, value: u8) {
        self.quadrant_fullsize_x = value;
    }

    pub(crate) fn set_quadrant_fullsize_y(&mut self, value: u8) {
        self.quadrant_fullsize_y = value;
    }

    pub(crate) fn set_fullsize_overworld_quadrants(&mut self) {
        self.quadrant_fullsize_x = 2;
        self.quadrant_fullsize_y = 2;
    }

    pub(crate) fn set_horizontal_room_fullsize_state(&mut self, value: u8) {
        self.quadrant_fullsize_x = value;
    }

    pub(crate) fn set_vertical_room_fullsize_state(&mut self, value: u8) {
        self.quadrant_fullsize_y = value;
    }

    pub(crate) fn apply_dungeon_layout_quadrant_fullsize(
        &mut self,
        layout_flags: u8,
        horizontal_mask: u8,
        vertical_mask: u8,
        blast_wall_x_open: bool,
        blast_wall_y_open: bool,
    ) {
        self.apply_dungeon_layout_horizontal_fullsize(
            layout_flags,
            horizontal_mask,
            blast_wall_x_open,
        );
        self.apply_dungeon_layout_vertical_fullsize(layout_flags, vertical_mask, blast_wall_y_open);
    }

    pub(crate) fn apply_dungeon_layout_horizontal_fullsize(
        &mut self,
        layout_flags: u8,
        horizontal_mask: u8,
        blast_wall_x_open: bool,
    ) {
        self.quadrant_fullsize_x = if blast_wall_x_open || layout_flags & horizontal_mask == 0 {
            2
        } else {
            0
        };
    }

    pub(crate) fn apply_dungeon_layout_vertical_fullsize(
        &mut self,
        layout_flags: u8,
        vertical_mask: u8,
        blast_wall_y_open: bool,
    ) {
        self.quadrant_fullsize_y = if blast_wall_y_open || layout_flags & vertical_mask == 0 {
            2
        } else {
            0
        };
    }

    pub(crate) fn apply_reset_xy_quadrant_overrides(&mut self, reset_xy_flags: u16) {
        if reset_xy_flags as u8 != 0 {
            self.quadrant_fullsize_x = reset_xy_flags as u8;
        }
        if (reset_xy_flags >> 8) as u8 != 0 {
            self.quadrant_fullsize_y = (reset_xy_flags >> 8) as u8;
        }
    }

    pub(crate) fn force_horizontal_fullsize_for_blast_wall(&mut self) {
        self.quadrant_fullsize_x = 2;
    }

    pub(crate) fn force_vertical_fullsize_for_blast_wall(&mut self) {
        self.quadrant_fullsize_y = 2;
    }

    pub(crate) fn set_tilemap_layer_copy(&mut self, value: u16) {
        self.tilemap_layer_copy = value;
    }

    pub(crate) fn save_spexit_tm_copy(&mut self) {
        self.special_exit_tilemap_layer_copy = self.tilemap_layer_copy;
    }

    pub(crate) fn restore_spexit_layer_masks(&mut self) {
        self.tilemap_layer_copy = self.special_exit_tilemap_layer_copy;
    }

    pub(crate) fn save_exit_tm_copy(&mut self) {
        self.exit_tilemap_layer_copy = self.tilemap_layer_copy;
    }

    pub(crate) fn restore_exit_layer_masks(&mut self) {
        self.tilemap_layer_copy = self.exit_tilemap_layer_copy;
    }

    pub(crate) fn set_mapbak_ts(&mut self, value: u8) {
        self.map_backup_subscreen_layer = value;
    }

    pub(crate) fn set_mapbak_tm(&mut self, value: u8) {
        self.map_backup_main_layer = value;
    }

    pub(crate) fn increment_move_overlay_ctr(&mut self) -> u8 {
        self.move_overlay_counter = self.move_overlay_counter.wrapping_add(1) & 3;
        self.move_overlay_counter
    }

    pub(crate) fn set_overworld_hole_scan_step(&mut self, value: u8) {
        self.overworld_hole_scan_step = value;
    }

    pub(crate) fn set_overworld_peg_puzzle_progress(&mut self, value: u16) {
        self.overworld_peg_puzzle_progress = value;
    }

    pub(crate) fn set_overworld_hole_tilemap_pos(&mut self, value: u16) {
        self.overworld_hole_tilemap_position = value as u8;
    }

    pub(crate) fn set_overworld_bomb_tile_sweep_x(&mut self, value: u16) {
        self.overworld_bomb_tile_sweep_x = value;
    }

    pub(crate) fn set_overworld_bomb_tile_sweep_y_end(&mut self, value: u16) {
        self.overworld_bomb_tile_sweep_y_end = value;
    }

    pub(crate) fn set_big_key_door_message_triggered(&mut self, value: u16) {
        self.big_key_door_message_triggered = value;
    }

    pub(crate) fn set_savegame_has_master_sword_flags(&mut self, value: u16) {
        self.savegame_master_sword_flags = value;
    }

    pub(crate) fn set_dung_replacement_tile_state(&mut self, index: usize, value: u16) {
        self.dungeon_replacement_tiles[index] = value;
    }

    pub(crate) fn decrement_milestone_item_gfx_swap_countdown(&mut self) {
        self.milestone_item_graphics_countdown =
            self.milestone_item_graphics_countdown.wrapping_sub(1);
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

    pub(crate) fn set_packed_bounds(&mut self, top: u16, bottom: u16, left: u16, right: u16) {
        self.y_bounds = [top, bottom, left, right];
    }

    pub(crate) fn restore_from_cached_safe_return_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        self.set_y_bound(0, y_start);
        self.set_y_bound(2, y_end);
        self.set_x_bound(0, x_start);
        self.set_x_bound(2, x_end);
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
    pub(crate) camera_boundaries: WorldCameraBoundariesState,
    pub(crate) palette_theme: WorldPaletteThemeState,
    pub(crate) region: WorldRegionState,
    pub(crate) transient: WorldTransientState,
    pub(crate) overworld: OverworldState,
    pub(crate) room_bounds: RoomBoundsState,
}

impl WorldState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            location: WorldLocationState::load_from_ram(ram),
            scroll: WorldScrollState::load_from_ram(ram),
            camera_boundaries: WorldCameraBoundariesState::load_from_ram(ram),
            palette_theme: WorldPaletteThemeState::load_from_ram(ram),
            region: WorldRegionState::load_from_ram(ram),
            transient: WorldTransientState::load_from_ram(ram),
            overworld: OverworldState::load_from_ram(ram),
            room_bounds: RoomBoundsState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.location.write_to_ram(ram);
        self.scroll.write_to_ram(ram);
        self.camera_boundaries.write_to_ram(ram);
        self.palette_theme.write_to_ram(ram);
        self.region.write_to_ram(ram);
        self.transient.write_to_ram(ram);
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
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, WorldPaletteThemeState::load_from_ram(self.ram));
    }

    fn preserve_shared_palette_aliases(&mut self, preserve_hud_palette: bool, preserve_sp6r: bool) {
        self.state.sync_shared_palette_aliases_from_ram(
            self.ram,
            preserve_hud_palette,
            preserve_sp6r,
        );
    }

    fn sync_preserving_shared_palette_aliases(&mut self) {
        self.preserve_shared_palette_aliases(true, true);
        self.sync();
    }

    pub(crate) fn set_last_light_vs_dark_world(&mut self, value: u8) {
        self.state.set_last_light_vs_dark_world(value);
        self.sync_preserving_shared_palette_aliases();
    }

    pub(crate) fn set_aux_bg_subset(&mut self, index: usize, value: u8) {
        self.state.set_aux_bg_subset(index, value);
        self.sync_preserving_shared_palette_aliases();
    }

    pub(crate) fn set_overworld_palette_aux1_hi(&mut self, value: u8) {
        self.state.set_overworld_palette_aux1_hi(value);
        self.sync_preserving_shared_palette_aliases();
    }

    pub(crate) fn set_hud_palette(&mut self, value: u8) {
        self.state.set_hud_palette(value);
        self.preserve_shared_palette_aliases(false, true);
        self.sync();
    }

    pub(crate) fn set_overworld_tile_theme_index(&mut self, value: u8) {
        self.state.set_overworld_tile_theme_index(value);
        self.sync_preserving_shared_palette_aliases();
    }

    pub(crate) fn set_main_tile_theme_index(&mut self, value: u8) {
        self.state.set_main_tile_theme_index(value);
        self.sync_preserving_shared_palette_aliases();
    }

    pub(crate) fn set_aux_tile_theme_index(&mut self, value: u8) {
        self.state.set_aux_tile_theme_index(value);
        self.sync_preserving_shared_palette_aliases();
    }

    pub(crate) fn set_misc_sprites_graphics_index(&mut self, value: u8) {
        self.state.set_misc_sprites_graphics_index(value);
        self.sync_preserving_shared_palette_aliases();
    }

    pub(crate) fn set_palette_sp6r_indoors(&mut self, value: u8) {
        self.state.set_palette_sp6r_indoors(value);
        self.preserve_shared_palette_aliases(true, false);
        self.sync();
    }

    pub(crate) fn restore_exit_tile_themes(&mut self) {
        self.state.restore_exit_tile_themes();
        self.sync_preserving_shared_palette_aliases();
    }

    pub(crate) fn save_special_exit_tile_themes(&mut self) {
        self.state.save_special_exit_tile_themes();
        self.sync_preserving_shared_palette_aliases();
    }

    pub(crate) fn restore_special_exit_tile_themes(&mut self) {
        self.state.restore_special_exit_tile_themes();
        self.sync_preserving_shared_palette_aliases();
    }
}

pub(crate) struct NativeWorldScrollBridgeMut<'a> {
    state: &'a mut WorldScrollState,
    ram: &'a mut [u8],
}

impl<'a> NativeWorldScrollBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut WorldScrollState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, WorldScrollState::load_from_ram(self.ram));
    }

    pub(crate) fn set_bg1_x_offset(&mut self, value: u16) {
        self.state.set_bg1_x_offset(value);
        self.sync();
    }

    pub(crate) fn set_bg1_y_offset(&mut self, value: u16) {
        self.state.set_bg1_y_offset(value);
        self.sync();
    }

    pub(crate) fn set_bg1_offsets(&mut self, x: u16, y: u16) {
        self.state.set_bg1_offsets(x, y);
        self.sync();
    }

    pub(crate) fn clear_bg1_offsets(&mut self) {
        self.state.clear_bg1_offsets();
        self.sync();
    }

    pub(crate) fn set_overworld_offset_base_y(&mut self, value: u16) {
        self.state.set_overworld_offset_base_y(value);
        self.sync();
    }

    pub(crate) fn set_overworld_offset_base_x(&mut self, value: u16) {
        self.state.set_overworld_offset_base_x(value);
        self.sync();
    }

    pub(crate) fn set_overworld_offset_mask_y(&mut self, value: u16) {
        self.state.set_overworld_offset_mask_y(value);
        self.sync();
    }

    pub(crate) fn set_overworld_offset_mask_x(&mut self, value: u16) {
        self.state.set_overworld_offset_mask_x(value);
        self.sync();
    }
}

pub(crate) struct NativeWorldCameraBoundariesBridgeMut<'a> {
    state: &'a mut WorldCameraBoundariesState,
    ram: &'a mut [u8],
}

impl<'a> NativeWorldCameraBoundariesBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut WorldCameraBoundariesState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            WorldCameraBoundariesState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_camera_y_coord_scroll_low(&mut self, value: u16) {
        self.state.set_camera_y_coord_scroll_low(value);
        self.sync();
    }

    pub(crate) fn set_camera_y_coord_scroll_hi(&mut self, value: u16) {
        self.state.set_camera_y_coord_scroll_hi(value);
        self.sync();
    }

    pub(crate) fn set_camera_x_coord_scroll_low(&mut self, value: u16) {
        self.state.set_camera_x_coord_scroll_low(value);
        self.sync();
    }

    pub(crate) fn set_camera_x_coord_scroll_hi(&mut self, value: u16) {
        self.state.set_camera_x_coord_scroll_hi(value);
        self.sync();
    }

    pub(crate) fn add_camera_scroll_for_axis(&mut self, horizontal: bool, delta: i16) -> u16 {
        let hi = self.state.add_camera_scroll_for_axis(horizontal, delta);
        self.sync();
        hi
    }

    pub(crate) fn set_camera_scroll_from_link_for_axis(&mut self, horizontal: bool, value: u16) {
        self.state
            .set_camera_scroll_from_link_for_axis(horizontal, value);
        self.sync();
    }

    pub(crate) fn set_up_down_scroll_target(&mut self, value: u16) {
        self.state.set_up_down_scroll_target(value);
        self.sync();
    }

    pub(crate) fn set_up_down_scroll_target_end(&mut self, value: u16) {
        self.state.set_up_down_scroll_target_end(value);
        self.sync();
    }

    pub(crate) fn set_left_right_scroll_target(&mut self, value: u16) {
        self.state.set_left_right_scroll_target(value);
        self.sync();
    }

    pub(crate) fn set_left_right_scroll_target_end(&mut self, value: u16) {
        self.state.set_left_right_scroll_target_end(value);
        self.sync();
    }

    pub(crate) fn cache_scroll_targets(&mut self) {
        self.state.cache_scroll_targets();
        self.sync();
    }

    pub(crate) fn cache_camera_scroll(&mut self) {
        self.state.cache_camera_scroll();
        self.sync();
    }

    pub(crate) fn restore_scroll_targets_from_cached(&mut self) {
        self.state.restore_scroll_targets_from_cached();
        self.sync();
    }

    pub(crate) fn set_overworld_scroll_up_counter(&mut self, value: u16) {
        self.state.set_overworld_scroll_up_counter(value);
        self.sync();
    }

    pub(crate) fn set_overworld_scroll_down_counter(&mut self, value: u16) {
        self.state.set_overworld_scroll_down_counter(value);
        self.sync();
    }

    pub(crate) fn set_overworld_scroll_left_counter(&mut self, value: u16) {
        self.state.set_overworld_scroll_left_counter(value);
        self.sync();
    }

    pub(crate) fn set_overworld_scroll_right_counter(&mut self, value: u16) {
        self.state.set_overworld_scroll_right_counter(value);
        self.sync();
    }

    pub(crate) fn set_overworld_scroll_counter_for_axis(&mut self, ya: usize, value: u16) {
        self.state.set_overworld_scroll_counter_for_axis(ya, value);
        self.sync();
    }

    pub(crate) fn clear_opposed_scroll_counters(&mut self, ya: usize) {
        self.state.clear_opposed_scroll_counters(ya);
        self.sync();
    }

    pub(crate) fn set_opposed_scroll_counter_pair(&mut self, ya: usize, value: u16) {
        self.state.set_opposed_scroll_counter_pair(ya, value);
        self.sync();
    }

    pub(crate) fn set_special_exit_room_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        self.state
            .set_special_exit_room_bounds(y_start, y_end, x_start, x_end);
        self.sync();
    }

    pub(crate) fn save_exit_room_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        self.state
            .save_exit_room_bounds(y_start, y_end, x_start, x_end);
        self.sync();
    }

    pub(crate) fn copy_spexit_scroll_targets(&mut self) {
        self.state.copy_spexit_scroll_targets();
        self.sync();
    }

    pub(crate) fn copy_spexit_scroll_counters(&mut self) {
        self.state.copy_spexit_scroll_counters();
        self.sync();
    }

    pub(crate) fn restore_spexit_scroll_targets(&mut self) {
        self.state.restore_spexit_scroll_targets();
        self.sync();
    }

    pub(crate) fn restore_spexit_scroll_counters(&mut self) {
        self.state.restore_spexit_scroll_counters();
        self.sync();
    }

    pub(crate) fn copy_exit_scroll_targets(&mut self) {
        self.state.copy_exit_scroll_targets();
        self.sync();
    }

    pub(crate) fn copy_exit_scroll_counters(&mut self) {
        self.state.copy_exit_scroll_counters();
        self.sync();
    }

    pub(crate) fn restore_exit_scroll_targets(&mut self) {
        self.state.restore_exit_scroll_targets();
        self.sync();
    }

    pub(crate) fn restore_exit_scroll_counters(&mut self) {
        self.state.restore_exit_scroll_counters();
        self.sync();
    }

    pub(crate) fn save_spexit_camera_coords(&mut self) {
        self.state.save_spexit_camera_coords();
        self.sync();
    }

    pub(crate) fn save_exit_camera_coords(&mut self) {
        self.state.save_exit_camera_coords();
        self.sync();
    }

    pub(crate) fn restore_exit_camera_scroll(&mut self) {
        self.state.restore_exit_camera_scroll();
        self.sync();
    }

    pub(crate) fn restore_special_exit_camera_scroll(&mut self) {
        self.state.restore_special_exit_camera_scroll();
        self.sync();
    }

    pub(crate) fn restore_camera_y_from_cached_indoor(&mut self) {
        self.state.restore_camera_y_from_cached_indoor();
        self.sync();
    }

    pub(crate) fn restore_camera_x_from_cached_indoor(&mut self) {
        self.state.restore_camera_x_from_cached_indoor();
        self.sync();
    }

    pub(crate) fn update_camera_hi_outdoor(&mut self) {
        self.state.update_camera_hi_outdoor();
        self.sync();
    }
}

pub(crate) struct NativeWorldRegionBridgeMut<'a> {
    state: &'a mut WorldRegionState,
    ram: &'a mut [u8],
}

impl<'a> NativeWorldRegionBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut WorldRegionState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, WorldRegionState::load_from_ram(self.ram));
    }

    pub(crate) fn set_rng_seed(&mut self, value: u8) {
        self.state.set_rng_seed(value);
        self.sync();
    }

    pub(crate) fn set_dark_world_region_index(&mut self, value: u8) {
        self.state.set_dark_world_region_index(value);
        self.sync();
    }

    pub(crate) fn set_which_entrance(&mut self, value: u16) {
        self.state.set_which_entrance(value);
        self.sync();
    }

    pub(crate) fn set_which_entrance_byte(&mut self, value: u8) {
        self.state.set_which_entrance_byte(value);
        self.sync();
    }

    pub(crate) fn set_overworld_area_index(&mut self, value: u8) {
        self.state.set_overworld_area_index(value);
        self.sync();
    }

    pub(crate) fn set_overworld_area_index_word(&mut self, value: u16) {
        self.state.set_overworld_area_index_word(value);
        self.sync();
    }

    pub(crate) fn set_current_area_of_player_word(&mut self, value: u16) {
        self.state.set_current_area_of_player_word(value);
        self.sync();
    }

    pub(crate) fn set_flag_overworld_area_changed(&mut self, value: u8) {
        self.state.set_flag_overworld_area_changed(value);
        self.sync();
    }

    pub(crate) fn clear_flag_overworld_area_changed(&mut self) {
        self.state.clear_flag_overworld_area_changed();
        self.sync();
    }

    pub(crate) fn clear_overlay_index_word(&mut self) {
        self.state.clear_overlay_index_word();
        self.sync();
    }

    pub(crate) fn set_overlay_index_word(&mut self, value: u16) {
        self.state.set_overlay_index_word(value);
        self.sync();
    }

    pub(crate) fn set_overlay_high(&mut self, value: u8) {
        self.state.set_overlay_high(value);
        self.sync();
    }

    pub(crate) fn set_prev_screen_index_word(&mut self, value: u16) {
        self.state.set_prev_screen_index_word(value);
        self.sync();
    }

    pub(crate) fn save_spexit_area_index(&mut self) {
        self.state.save_spexit_area_index();
        self.sync();
    }

    pub(crate) fn restore_spexit_area_index(&mut self) {
        self.state.restore_spexit_area_index();
        self.sync();
    }

    pub(crate) fn save_exit_area_index(&mut self) {
        self.state.save_exit_area_index();
        self.sync();
    }

    pub(crate) fn restore_exit_area_index(&mut self) {
        self.state.restore_exit_area_index();
        self.sync();
    }

    pub(crate) fn set_ow_entrance_value(&mut self, value: u16) {
        self.state.set_ow_entrance_value(value);
        self.sync();
    }

    pub(crate) fn ow_entrance_value(&self) -> u16 {
        self.state.overworld_entrance_value
    }
}

pub(crate) struct NativeWorldTransientBridgeMut<'a> {
    state: &'a mut WorldTransientState,
    ram: &'a mut [u8],
}

impl<'a> NativeWorldTransientBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut WorldTransientState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_scalar_fields_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn sync_preserving_projected_door_animation_step(&mut self) {
        self.state
            .set_door_animation_step_word(read_le_u16(self.ram, DOOR_ANIMATION_STEP_INDICATOR));
        self.sync();
    }

    fn debug_assert_matches_ram(&self) {
        let mut loaded = WorldTransientState::load_from_ram(self.ram);
        loaded.dungeon_replacement_tiles = self.state.dungeon_replacement_tiles.clone();
        debug_assert_eq!(*self.state, loaded);
    }

    pub(crate) fn set_room_transitioning_flags(&mut self, value: u8) {
        self.state.set_room_transitioning_flags(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn clear_custom_spell_animation(&mut self) {
        self.state.clear_custom_spell_animation();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_custom_spell_animation_active(&mut self) {
        self.state.set_custom_spell_animation_active();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_allow_scroll_z(&mut self, value: u8) {
        self.state.set_allow_scroll_z(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_cached_room_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        self.state
            .set_cached_room_bounds(y_start, y_end, x_start, x_end);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_standing_in_doorway_cached(&mut self, value: u8) {
        self.state.set_standing_in_doorway_cached(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn cache_standing_in_doorway(&mut self, doorway_state: u8) {
        self.state.set_standing_in_doorway_cached(doorway_state);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_flag_travel_bird(&mut self, value: u8) {
        self.state.set_flag_travel_bird(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn clear_tile_interaction_shared_flag(&mut self) {
        self.state.clear_tile_interaction_shared_flag();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_door_animation_step(&mut self, value: u8) {
        self.state.set_door_animation_step(value);
        self.sync();
    }

    pub(crate) fn set_door_animation_step_word(&mut self, value: u16) {
        self.state.set_door_animation_step_word(value);
        self.sync();
    }

    pub(crate) fn clear_hud_floor_changed_timer(&mut self) {
        self.state.clear_hud_floor_changed_timer();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn cache_quadrant_fullsize_state(&mut self) {
        self.state.cache_quadrant_fullsize_state();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn restore_quadrant_fullsize_from_cached(&mut self) {
        self.state.restore_quadrant_fullsize_from_cached();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_quadrant_fullsize_x(&mut self, value: u8) {
        self.state.set_quadrant_fullsize_x(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_quadrant_fullsize_y(&mut self, value: u8) {
        self.state.set_quadrant_fullsize_y(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_fullsize_overworld_quadrants(&mut self) {
        self.state.set_fullsize_overworld_quadrants();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_horizontal_room_fullsize_state(&mut self, value: u8) {
        self.state.set_horizontal_room_fullsize_state(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_vertical_room_fullsize_state(&mut self, value: u8) {
        self.state.set_vertical_room_fullsize_state(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn apply_dungeon_layout_quadrant_fullsize(
        &mut self,
        layout_flags: u8,
        horizontal_mask: u8,
        vertical_mask: u8,
        blast_wall_x_open: bool,
        blast_wall_y_open: bool,
    ) {
        self.state.apply_dungeon_layout_quadrant_fullsize(
            layout_flags,
            horizontal_mask,
            vertical_mask,
            blast_wall_x_open,
            blast_wall_y_open,
        );
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn apply_dungeon_layout_horizontal_fullsize(
        &mut self,
        layout_flags: u8,
        horizontal_mask: u8,
        blast_wall_x_open: bool,
    ) {
        self.state.apply_dungeon_layout_horizontal_fullsize(
            layout_flags,
            horizontal_mask,
            blast_wall_x_open,
        );
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn apply_dungeon_layout_vertical_fullsize(
        &mut self,
        layout_flags: u8,
        vertical_mask: u8,
        blast_wall_y_open: bool,
    ) {
        self.state.apply_dungeon_layout_vertical_fullsize(
            layout_flags,
            vertical_mask,
            blast_wall_y_open,
        );
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn apply_reset_xy_quadrant_overrides(&mut self, reset_xy_flags: u16) {
        self.state.apply_reset_xy_quadrant_overrides(reset_xy_flags);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn force_horizontal_fullsize_for_blast_wall(&mut self) {
        self.state.force_horizontal_fullsize_for_blast_wall();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn force_vertical_fullsize_for_blast_wall(&mut self) {
        self.state.force_vertical_fullsize_for_blast_wall();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn save_spexit_tm_copy(&mut self) {
        self.state.save_spexit_tm_copy();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn restore_spexit_layer_masks(&mut self) {
        self.state.restore_spexit_layer_masks();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn save_exit_tm_copy(&mut self) {
        self.state.save_exit_tm_copy();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn restore_exit_layer_masks(&mut self) {
        self.state.restore_exit_layer_masks();
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_mapbak_ts(&mut self, value: u8) {
        self.state.set_mapbak_ts(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_mapbak_tm(&mut self, value: u8) {
        self.state.set_mapbak_tm(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn increment_move_overlay_ctr(&mut self) -> u8 {
        let value = self.state.increment_move_overlay_ctr();
        self.sync_preserving_projected_door_animation_step();
        value
    }

    pub(crate) fn set_overworld_hole_scan_step(&mut self, value: u8) {
        self.state.set_overworld_hole_scan_step(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_overworld_peg_puzzle_progress(&mut self, value: u16) {
        self.state.set_overworld_peg_puzzle_progress(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_overworld_hole_tilemap_pos(&mut self, value: u16) {
        self.state.set_overworld_hole_tilemap_pos(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_overworld_bomb_tile_sweep_x(&mut self, value: u16) {
        self.state.set_overworld_bomb_tile_sweep_x(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_overworld_bomb_tile_sweep_y_end(&mut self, value: u16) {
        self.state.set_overworld_bomb_tile_sweep_y_end(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_big_key_door_message_triggered(&mut self, value: u16) {
        self.state.set_big_key_door_message_triggered(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_savegame_has_master_sword_flags(&mut self, value: u16) {
        self.state.set_savegame_has_master_sword_flags(value);
        self.sync_preserving_projected_door_animation_step();
    }

    pub(crate) fn set_dung_replacement_tile_state(&mut self, index: usize, value: u16) {
        self.state.set_dung_replacement_tile_state(index, value);
        write_le_u16(self.ram, DUNG_REPLACEMENT_TILE_STATE + index * 2, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_milestone_item_gfx_swap_countdown(&mut self) {
        self.state.decrement_milestone_item_gfx_swap_countdown();
        self.sync_preserving_projected_door_animation_step();
    }
}

pub(crate) struct NativeRoomBoundsBridgeMut<'a> {
    state: &'a mut RoomBoundsState,
    ram: &'a mut [u8],
}

impl<'a> NativeRoomBoundsBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut RoomBoundsState, ram: &'a mut [u8]) -> Self {
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

    pub(crate) fn set_packed_bounds(&mut self, top: u16, bottom: u16, left: u16, right: u16) {
        self.state.set_packed_bounds(top, bottom, left, right);
        self.sync();
    }

    pub(crate) fn restore_from_cached_safe_return_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        self.state
            .restore_from_cached_safe_return_bounds(y_start, y_end, x_start, x_end);
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
        self.world_location.set_dungeon_room(value);
        self.sync();
    }

    pub(crate) fn set_dungeon_room_index(&mut self, value: u8) {
        self.world_location.set_dungeon_room_index(value);
        self.sync();
    }

    pub(crate) fn increment_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        let next = self.world_location.increment_dungeon_room_index_by(value);
        self.sync();
        next
    }

    pub(crate) fn decrement_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        let next = self.world_location.decrement_dungeon_room_index_by(value);
        self.sync();
        next
    }

    pub(crate) fn set_overworld_screen(&mut self, value: u8) {
        self.world_location.set_overworld_screen(value);
        self.sync();
    }

    pub(crate) fn set_overworld_screen_word(&mut self, value: u16) {
        self.world_location.set_overworld_screen_word(value);
        self.sync();
    }

    pub(crate) fn set_indoor_flag(&mut self, value: u8) {
        self.world_location.set_indoor_flag(value);
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
        Self { event_info, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.event_info,
            OverworldEventInfoState::load_from_ram(self.ram)
        );
    }

    fn sync(&mut self) {
        self.event_info.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_event_info(&mut self, screen: usize, value: u8) {
        self.event_info.set_event_info(screen, value);
        self.sync();
    }

    pub(crate) fn set_event_bits(&mut self, screen: usize, mask: u8) {
        self.event_info.set_event_bits(screen, mask);
        self.sync();
    }

    pub(crate) fn clear_event_bits(&mut self, screen: usize, mask: u8) {
        self.event_info.clear_event_bits(screen, mask);
        self.sync();
    }
}

pub(crate) struct NativeOverworldConfigTableBridgeMut<'a> {
    config_table: &'a mut OverworldConfigTableState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldConfigTableBridgeMut<'a> {
    pub(crate) fn new(config_table: &'a mut OverworldConfigTableState, ram: &'a mut [u8]) -> Self {
        Self { config_table, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.config_table,
            OverworldConfigTableState::load_from_ram(self.ram)
        );
    }

    fn sync(&mut self) {
        self.config_table.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn copy_music_primary(&mut self, data: &[u8]) {
        self.config_table.copy_music_primary(data);
        self.sync();
    }

    pub(crate) fn copy_music_secondary(&mut self, data: &[u8]) {
        self.config_table.copy_music_secondary(data);
        self.sync();
    }

    pub(crate) fn copy_sprite_graphics_range(
        &mut self,
        dst: usize,
        data: &[u8],
        src: usize,
        len: usize,
    ) {
        self.config_table
            .copy_sprite_graphics_range(dst, data, src, len);
        self.sync();
    }

    pub(crate) fn copy_sprite_palette_range(
        &mut self,
        dst: usize,
        data: &[u8],
        src: usize,
        len: usize,
    ) {
        self.config_table
            .copy_sprite_palette_range(dst, data, src, len);
        self.sync();
    }

    pub(crate) fn set_music(&mut self, screen: usize, value: u8) {
        self.config_table.set_music(screen, value);
        self.sync();
    }
}

pub(crate) struct NativeOverworldMapUiBridgeMut<'a> {
    map_ui: &'a mut OverworldMapUiState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldMapUiBridgeMut<'a> {
    pub(crate) fn new(map_ui: &'a mut OverworldMapUiState, ram: &'a mut [u8]) -> Self {
        Self { map_ui, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.map_ui, OverworldMapUiState::load_from_ram(self.ram));
    }

    fn sync(&mut self) {
        self.map_ui.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_map_state(&mut self, value: u8) {
        self.map_ui.set_map_state(value);
        self.sync();
    }

    pub(crate) fn set_map_state_word(&mut self, value: u16) {
        self.map_ui.set_map_state_word(value);
        self.sync();
    }

    pub(crate) fn increment_map_state(&mut self) {
        self.map_ui.increment_map_state();
        self.sync();
    }

    pub(crate) fn set_map_flags(&mut self, value: u8) {
        self.map_ui.set_map_flags(value);
        self.sync();
    }

    pub(crate) fn and_map_flags(&mut self, value: u8) {
        self.map_ui.and_map_flags(value);
        self.sync();
    }

    pub(crate) fn or_map_flags(&mut self, value: u8) {
        self.map_ui.or_map_flags(value);
        self.sync();
    }

    pub(crate) fn set_birdtravel_status(&mut self, value: u8) {
        self.map_ui.set_birdtravel_status(value);
        self.sync();
    }

    pub(crate) fn set_birdtravel_status_word(&mut self, value: u16) {
        self.map_ui.set_birdtravel_status_word(value);
        self.sync();
    }

    pub(crate) fn and_birdtravel_status(&mut self, value: u8) {
        self.map_ui.and_birdtravel_status(value);
        self.sync();
    }

    pub(crate) fn decrement_birdtravel_status(&mut self) {
        self.map_ui.decrement_birdtravel_status();
        self.sync();
    }

    pub(crate) fn increment_birdtravel_status(&mut self) {
        self.map_ui.increment_birdtravel_status();
        self.sync();
    }

    pub(crate) fn clear_bird_travel_stop_status(&mut self, slot: usize) {
        self.map_ui.clear_bird_travel_stop_status(slot);
        self.sync();
    }

    pub(crate) fn increment_bird_travel_stop_status(&mut self, slot: usize) {
        self.map_ui.increment_bird_travel_stop_status(slot);
        self.sync();
    }
}

pub(crate) struct NativeWeatherVaneBridgeMut<'a> {
    weather_vane: &'a mut WeatherVaneState,
    ram: &'a mut [u8],
}

impl<'a> NativeWeatherVaneBridgeMut<'a> {
    pub(crate) fn new(weather_vane: &'a mut WeatherVaneState, ram: &'a mut [u8]) -> Self {
        Self { weather_vane, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.weather_vane,
            WeatherVaneState::load_from_ram(self.ram)
        );
    }

    fn sync(&mut self) {
        self.weather_vane.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_countdown(&mut self, value: u16) {
        self.weather_vane.set_countdown(value);
        self.sync();
    }

    pub(crate) fn tick_countdown(&mut self) -> u16 {
        let value = self.weather_vane.tick_countdown();
        self.sync();
        value
    }

    pub(crate) fn set_music_latch(&mut self, value: u8) {
        self.weather_vane.set_music_latch(value);
        self.sync();
    }

    pub(crate) fn set_source_slot(&mut self, value: u8) {
        self.weather_vane.set_source_slot(value);
        self.sync();
    }

    pub(crate) fn reset_oam_offset(&mut self) {
        self.weather_vane.reset_oam_offset();
        self.sync();
    }

    pub(crate) fn advance_oam_offset(&mut self, value: u8) {
        self.weather_vane.advance_oam_offset(value);
        self.sync();
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
        Self { destinations, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.destinations,
            BirdTravelDestinationsState::load_from_ram(self.ram)
        );
    }

    fn sync(&mut self) {
        self.destinations.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_destination(&mut self, slot: usize, x: u16, y: u16) {
        self.destinations.set_destination(slot, x, y);
        self.sync();
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
        Self { zoom, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.zoom, OverworldMapZoomState::load_from_ram(self.ram));
    }

    fn sync(&mut self) {
        self.zoom.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_step_counter(&mut self, value: u8) {
        self.zoom.set_step_counter(value);
        self.sync();
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.zoom.set_timer(value);
        self.sync();
    }

    pub(crate) fn decrement_timer(&mut self) {
        self.zoom.decrement_timer();
        self.sync();
    }
}

pub(crate) struct NativeOverworldScreenSizeBridgeMut<'a> {
    screen_size: &'a mut OverworldScreenSizeState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldScreenSizeBridgeMut<'a> {
    pub(crate) fn new(screen_size: &'a mut OverworldScreenSizeState, ram: &'a mut [u8]) -> Self {
        Self { screen_size, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.screen_size,
            OverworldScreenSizeState::load_from_ram(self.ram)
        );
    }

    fn sync(&mut self) {
        self.screen_size.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_big_area_high(&mut self) {
        self.screen_size.clear_big_area_high();
        self.sync();
    }

    pub(crate) fn set_big_area_low(&mut self, value: u8) {
        self.screen_size.set_big_area_low(value);
        self.sync();
    }

    pub(crate) fn backup_big_area_low(&mut self) {
        self.screen_size.backup_big_area_low();
        self.sync();
    }

    pub(crate) fn set_right_bottom_bound_low(&mut self, value: u8) {
        self.screen_size.set_right_bottom_bound_low(value);
        self.sync();
    }

    pub(crate) fn set_right_bottom_bound_high(&mut self, value: u8) {
        self.screen_size.set_right_bottom_bound_high(value);
        self.sync();
    }
}

pub(crate) struct NativeOverworldScrollDeltaBridgeMut<'a> {
    scroll_delta: &'a mut OverworldScrollDeltaState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldScrollDeltaBridgeMut<'a> {
    pub(crate) fn new(scroll_delta: &'a mut OverworldScrollDeltaState, ram: &'a mut [u8]) -> Self {
        Self { scroll_delta, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.scroll_delta,
            OverworldScrollDeltaState::load_from_ram(self.ram)
        );
    }

    fn sync(&mut self) {
        self.scroll_delta.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_vertical_delta_low_byte(&mut self, value: u8) {
        self.scroll_delta.set_vertical_delta_low_byte(value);
        self.sync();
    }

    pub(crate) fn set_horizontal_delta_low_byte(&mut self, value: u8) {
        self.scroll_delta.set_horizontal_delta_low_byte(value);
        self.sync();
    }

    pub(crate) fn set_vertical_delta_word(&mut self, value: u16) {
        self.scroll_delta.set_vertical_delta_word(value);
        self.sync();
    }

    pub(crate) fn set_horizontal_delta_word(&mut self, value: u16) {
        self.scroll_delta.set_horizontal_delta_word(value);
        self.sync();
    }

    pub(crate) fn clear_vertical_delta_low_byte(&mut self) {
        self.scroll_delta.clear_vertical_delta_low_byte();
        self.sync();
    }
}

pub(crate) struct NativeOverworldMap16BridgeMut<'a> {
    map16: &'a mut OverworldMap16State,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldMap16BridgeMut<'a> {
    pub(crate) fn new(map16: &'a mut OverworldMap16State, ram: &'a mut [u8]) -> Self {
        Self { map16, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.map16, OverworldMap16State::load_from_ram(self.ram));
    }

    fn sync(&mut self) {
        self.map16.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn sync_from_ram(&mut self) {
        *self.map16 = OverworldMap16State::load_from_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_active_load(&mut self, state: OverworldMap16LoadState) {
        self.map16.active_load = state;
        self.sync();
    }

    pub(crate) fn set_previous_load(&mut self, state: OverworldMap16LoadState) {
        self.map16.previous_load = state;
        self.sync();
    }

    pub(crate) fn set_special_exit_src_off(&mut self, src_off: u16) {
        self.map16.special_exit_src_off = src_off;
        self.sync();
    }

    pub(crate) fn set_exit_src_off(&mut self, src_off: u16) {
        self.map16.exit_src_off = src_off;
        self.sync();
    }

    pub(crate) fn set_small_scroll_backup(&mut self, state: SmallOverworldMap16ScrollBackupState) {
        self.map16.small_scroll_backup = state;
        self.sync();
    }
}

pub(crate) struct NativeOverworldEntranceBridgeMut<'a> {
    entrance: &'a mut OverworldEntranceState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldEntranceBridgeMut<'a> {
    pub(crate) fn new(entrance: &'a mut OverworldEntranceState, ram: &'a mut [u8]) -> Self {
        Self { entrance, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.entrance,
            OverworldEntranceState::load_from_ram(self.ram)
        );
    }

    fn sync(&mut self) {
        self.entrance.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_special_entrance_trigger(&mut self, value: u8) {
        self.entrance.set_special_entrance_trigger(value);
        self.sync();
    }

    pub(crate) fn clear_special_entrance_trigger(&mut self) {
        self.entrance.clear_special_entrance_trigger();
        self.sync();
    }

    pub(crate) fn set_sequence_counter(&mut self, value: u8) {
        self.entrance.set_sequence_counter(value);
        self.sync();
    }

    pub(crate) fn clear_sequence_counter(&mut self) {
        self.entrance.clear_sequence_counter();
        self.sync();
    }

    pub(crate) fn increment_sequence_counter(&mut self) -> u8 {
        let next = self.entrance.increment_sequence_counter();
        self.sync();
        next
    }

    pub(crate) fn decrement_sequence_counter(&mut self) -> u8 {
        let next = self.entrance.decrement_sequence_counter();
        self.sync();
        next
    }
}

pub(crate) struct NativeOverworldExitBridgeMut<'a> {
    exit: &'a mut OverworldExitState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldExitBridgeMut<'a> {
    pub(crate) fn new(exit: &'a mut OverworldExitState, ram: &'a mut [u8]) -> Self {
        Self { exit, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.exit, OverworldExitState::load_from_ram(self.ram));
    }

    fn sync(&mut self) {
        self.exit.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_exit_screen(&mut self, value: u16) {
        self.exit.set_exit_screen(value);
        self.sync();
    }

    pub(crate) fn set_special_exit_screen(&mut self, value: u16) {
        self.exit.set_special_exit_screen(value);
        self.sync();
    }
}

pub(crate) struct NativeOverworldTransitionBridgeMut<'a> {
    transition: &'a mut OverworldTransitionState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldTransitionBridgeMut<'a> {
    pub(crate) fn new(transition: &'a mut OverworldTransitionState, ram: &'a mut [u8]) -> Self {
        Self { transition, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.transition,
            OverworldTransitionState::load_from_ram(self.ram)
        );
    }

    fn sync(&mut self) {
        self.transition.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_bits(&mut self, value: u8) {
        self.transition.set_direction_bits(value);
        self.sync();
    }

    pub(crate) fn set_direction_bits_word(&mut self, value: u16) {
        self.transition.set_direction_bits_word(value);
        self.sync();
    }

    pub(crate) fn clear_direction_bits(&mut self) {
        self.transition.clear_direction_bits();
        self.sync();
    }

    pub(crate) fn clear_direction_bits_word(&mut self) {
        self.transition.clear_direction_bits_word();
        self.sync();
    }

    pub(crate) fn and_direction_bits(&mut self, value: u8) {
        self.transition.and_direction_bits(value);
        self.sync();
    }

    pub(crate) fn or_direction_bits(&mut self, value: u8) {
        self.transition.or_direction_bits(value);
        self.sync();
    }

    pub(crate) fn or_direction_bits_word(&mut self, value: u16) -> u16 {
        let next = self.transition.or_direction_bits_word(value);
        self.sync();
        next
    }

    pub(crate) fn set_edge_direction_bits(&mut self, value: u8) {
        self.transition.set_edge_direction_bits(value);
        self.sync();
    }

    pub(crate) fn set_edge_direction_bits_word(&mut self, value: u16) {
        self.transition.set_edge_direction_bits_word(value);
        self.sync();
    }

    pub(crate) fn clear_edge_direction_bits(&mut self) {
        self.transition.clear_edge_direction_bits();
        self.sync();
    }

    pub(crate) fn set_direction_enum(&mut self, value: u8) {
        self.transition.set_direction_enum(value);
        self.sync();
    }

    pub(crate) fn set_screen_transition(&mut self, value: u8) {
        self.transition.set_screen_transition(value);
        self.sync();
    }

    pub(crate) fn set_screen_transition_word(&mut self, value: u16) {
        self.transition.set_screen_transition_word(value);
        self.sync();
    }

    pub(crate) fn clear_screen_transition(&mut self) {
        self.transition.clear_screen_transition();
        self.sync();
    }

    pub(crate) fn set_transition_counter(&mut self, value: u8) {
        self.transition.set_transition_counter(value);
        self.sync();
    }

    pub(crate) fn increment_transition_counter(&mut self) -> u8 {
        let next = self.transition.increment_transition_counter();
        self.sync();
        next
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.transition.set_countdown(value);
        self.sync();
    }

    pub(crate) fn decrement_countdown(&mut self) -> u8 {
        let next = self.transition.decrement_countdown();
        self.sync();
        next
    }

    pub(crate) fn save_previous_direction_bits(&mut self) {
        self.transition.save_previous_direction_bits();
        self.sync();
    }

    pub(crate) fn restore_previous_direction_bits(&mut self) {
        self.transition.restore_previous_direction_bits();
        self.sync();
    }

    pub(crate) fn set_previous_screen_transition(&mut self, value: u8) {
        self.transition.set_previous_screen_transition(value);
        self.sync();
    }
}
