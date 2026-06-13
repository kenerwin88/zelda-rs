use crate::game_state::constants::{
    AUX_TILE_THEME_INDEX, DUNGEON_BG1_ATTR_TABLE, DUNGEON_BG2_ATTR_TABLE, DUNGEON_FLOOR_X_VELOCITY,
    DUNGEON_FLOOR_Y_VELOCITY, DUNGEON_HEADER_HOLE_TELEPORTER_PLANE, DUNGEON_HEADER_STAIRCASE_PLANE,
    DUNGEON_HEADER_TAG, DUNGEON_HEADER_TRAVEL_DESTINATIONS, DUNGEON_TORCH_ATTR, DUNGEON_TORCH_DATA,
    DUNGEON_WORK_R16, DUNGEON_WORK_R18, DUNG_CUR_FLOOR, DUNG_CUR_FLOOR_CACHED,
    DUNG_FLOOR_MOVE_FLAGS, DUNG_FLOOR_X_OFFS, DUNG_FLOOR_Y_OFFS, DUNG_INDEX_OF_TORCHES_START,
    DUNG_INTER_STAIRCASES, DUNG_NUM_ACTIVATED_WATER_LADDERS, DUNG_NUM_INROOM_UPNORTH_STAIRS,
    DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER,
    DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS, DUNG_NUM_STAIRS_1, DUNG_NUM_STAIRS_2, DUNG_NUM_STAIRS_WET,
    DUNG_OBJECT_POS_IN_OBJDATA, DUNG_SAVEGAME_STATE_BITS, GANON_TORCH_COUNT, MAIN_TILE_THEME_INDEX,
    MOVABLE_BLOCK_DATAS, OVERLAY_INDEX, OVERWORLD_EXIT_TILE_THEME_INDEX, OVERWORLD_SCREEN_INDEX,
    OVERWORLD_TILE_THEME_INDEX, SPRITE_GRAPHICS_INDEX, TORCH_TIMERS, WATER_SIDE_STEP_SWITCH,
};
use crate::game_state::constants::{
    COUNTDOWN_TIMER_FOR_STAIRCASES, CUR_STAIRCASE_PLANE, KIND_OF_IN_ROOM_STAIRCASE,
    STAIRCASE_LOWER_LEVEL_STATUS, STAIRCASE_MOVE_COUNTER, STAIRCASE_TILEMAP_POS_X2,
    WHICH_STAIRCASE_INDEX,
};
use crate::game_state::DungeonStairList;
use crate::types::{read_le_u16, write_le_u16};

const DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT: usize = 5;
const DUNGEON_HEADER_PLANE_SCRATCH_COUNT: usize = 5;
const DUNGEON_HEADER_TAG_COUNT: usize = 2;
const DUNGEON_TORCH_TIMER_COUNT: usize = 16;
const DUNGEON_TORCH_OBJECT_POS_COUNT: usize = 16;
const DUNGEON_BG2_ATTR_BUFFER_LEN: usize = (DUNGEON_BG1_ATTR_TABLE - DUNGEON_BG2_ATTR_TABLE) * 2;
const DUNGEON_STAIR_LIST_COUNT: usize = 21;
const DUNGEON_INTER_STAIRCASE_TABLE_WORDS: usize =
    (DUNG_STAIRS_TABLE_1 - DUNG_INTER_STAIRCASES) / 2;
const DUNGEON_STAIR_TABLE_1_WORDS: usize = (DUNG_STAIRS_TABLE_2 - DUNG_STAIRS_TABLE_1) / 2;
const DUNGEON_STAIR_TABLE_2_WORDS: usize = (DUNGEON_DOOR_DEBRIS_X - DUNG_STAIRS_TABLE_2) / 2;

const DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS_LOCAL: usize = 0x0438;
const DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS_LOCAL: usize = 0x043a;
const DUNG_NUM_INROOM_SOUTHDOWN_STAIRS_LOCAL: usize = 0x043e;
const DUNG_NUM_WATER_LADDERS_LOCAL: usize = 0x0446;
const DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_LOCAL: usize = 0x047e;
const DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_LOCAL: usize = 0x0480;
const DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2_LOCAL: usize = 0x0482;
const DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2_LOCAL: usize = 0x0484;
const DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS_LOCAL: usize = 0x04a2;
const DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS_LOCAL: usize = 0x04a4;
const DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS_LOCAL: usize = 0x04a6;
const DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS_LOCAL: usize = 0x04a8;
const DUNG_STAIRS_TABLE_1: usize = 0x06b8;
const DUNG_STAIRS_TABLE_2: usize = 0x06ec;
const DUNGEON_DOOR_DEBRIS_X: usize = 0x0728;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonState {
    pub(crate) header: DungeonHeaderState,
    pub(crate) scratch_word: DungeonScratchWordState,
    pub(crate) entrance_backup: DungeonEntranceBackupState,
    pub(crate) torch: DungeonTorchState,
    pub(crate) savegame_state: DungeonSavegameState,
    pub(crate) bg2_attributes: DungeonBg2AttributeState,
    pub(crate) stair_lists: DungeonStairListsState,
    pub(crate) stair_movement: DungeonStairMovementState,
    pub(crate) moving_floor: DungeonMovingFloorState,
}

impl DungeonState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            header: DungeonHeaderState::load_from_ram(ram),
            scratch_word: DungeonScratchWordState::load_from_ram(ram),
            entrance_backup: DungeonEntranceBackupState::load_from_ram(ram),
            torch: DungeonTorchState::load_from_ram(ram),
            savegame_state: DungeonSavegameState::load_from_ram(ram),
            bg2_attributes: DungeonBg2AttributeState::load_from_ram(ram),
            stair_lists: DungeonStairListsState::load_from_ram(ram),
            stair_movement: DungeonStairMovementState::load_from_ram(ram),
            moving_floor: DungeonMovingFloorState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.header.write_to_ram(ram);
        self.scratch_word.write_to_ram(ram);
        self.entrance_backup.write_to_ram(ram);
        self.torch.write_to_ram(ram);
        self.savegame_state.write_to_ram(ram);
        self.bg2_attributes.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonMovingFloorState {
    y_velocity: u16,
    x_velocity: u16,
    x_offset: u16,
    y_offset: u16,
    move_flags: u16,
}

impl DungeonMovingFloorState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            y_velocity: read_le_u16(ram, DUNGEON_FLOOR_Y_VELOCITY),
            x_velocity: read_le_u16(ram, DUNGEON_FLOOR_X_VELOCITY),
            x_offset: read_le_u16(ram, DUNG_FLOOR_X_OFFS),
            y_offset: read_le_u16(ram, DUNG_FLOOR_Y_OFFS),
            move_flags: read_le_u16(ram, DUNG_FLOOR_MOVE_FLAGS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNGEON_FLOOR_Y_VELOCITY, self.y_velocity);
        write_le_u16(ram, DUNGEON_FLOOR_X_VELOCITY, self.x_velocity);
        write_le_u16(ram, DUNG_FLOOR_X_OFFS, self.x_offset);
        write_le_u16(ram, DUNG_FLOOR_Y_OFFS, self.y_offset);
        write_le_u16(ram, DUNG_FLOOR_MOVE_FLAGS, self.move_flags);
    }

    pub(crate) fn floor_y_velocity(&self) -> u16 {
        self.y_velocity
    }

    pub(crate) fn floor_y_velocity_low(&self) -> u8 {
        self.y_velocity as u8
    }

    pub(crate) fn floor_x_velocity(&self) -> u16 {
        self.x_velocity
    }

    pub(crate) fn floor_x_velocity_low(&self) -> u8 {
        self.x_velocity as u8
    }

    pub(crate) fn floor_x_offset(&self) -> u16 {
        self.x_offset
    }

    pub(crate) fn floor_y_offset(&self) -> u16 {
        self.y_offset
    }

    pub(crate) fn floor_move_flags(&self) -> u16 {
        self.move_flags
    }

    fn set_floor_y_velocity_high(&mut self, value: u8) {
        self.y_velocity = (self.y_velocity & 0x00ff) | (u16::from(value) << 8);
    }

    fn set_floor_y_velocity(&mut self, value: u16) {
        self.y_velocity = value;
    }

    fn set_floor_x_velocity(&mut self, value: u16) {
        self.x_velocity = value;
    }

    fn clear_floor_velocity(&mut self) {
        self.x_velocity = 0;
        self.y_velocity = 0;
    }

    fn set_floor_x_offset(&mut self, value: u16) {
        self.x_offset = value;
    }

    fn set_floor_y_offset(&mut self, value: u16) {
        self.y_offset = value;
    }

    fn set_floor_y_offset_low(&mut self, value: u8) {
        self.y_offset = (self.y_offset & 0xff00) | u16::from(value);
    }

    fn set_floor_offsets(&mut self, x: u16, y: u16) {
        self.x_offset = x;
        self.y_offset = y;
    }

    fn add_floor_x_offset(&mut self, delta: u16) -> u16 {
        self.x_offset = self.x_offset.wrapping_add(delta);
        self.x_offset
    }

    fn sub_floor_x_offset(&mut self, delta: u16) -> u16 {
        self.x_offset = self.x_offset.wrapping_sub(delta);
        self.x_offset
    }

    fn add_floor_y_offset(&mut self, delta: u16) -> u16 {
        self.y_offset = self.y_offset.wrapping_add(delta);
        self.y_offset
    }

    fn sub_floor_y_offset(&mut self, delta: u16) -> u16 {
        self.y_offset = self.y_offset.wrapping_sub(delta);
        self.y_offset
    }

    fn clear_floor_offsets(&mut self) {
        self.set_floor_offsets(0, 0);
    }

    fn clear_floor_move_flags(&mut self) {
        self.move_flags = 0;
    }

    fn set_floor_move_flags(&mut self, value: u16) {
        self.move_flags = value;
    }

    fn increment_floor_move_flags(&mut self) {
        self.move_flags = self.move_flags.wrapping_add(1);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonStairMovementState {
    current_floor_word: u16,
    cached_floor: u8,
    staircase_index: u16,
    move_counter: u8,
    current_plane: u8,
    lower_level_status: u8,
    tilemap_pos_x2: u16,
    in_room_kind: u16,
    countdown: u8,
}

impl DungeonStairMovementState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            current_floor_word: read_le_u16(ram, DUNG_CUR_FLOOR),
            cached_floor: ram.get(DUNG_CUR_FLOOR_CACHED).copied().unwrap_or(0),
            staircase_index: read_le_u16(ram, WHICH_STAIRCASE_INDEX),
            move_counter: ram.get(STAIRCASE_MOVE_COUNTER).copied().unwrap_or(0),
            current_plane: ram.get(CUR_STAIRCASE_PLANE).copied().unwrap_or(0),
            lower_level_status: ram.get(STAIRCASE_LOWER_LEVEL_STATUS).copied().unwrap_or(0),
            tilemap_pos_x2: read_le_u16(ram, STAIRCASE_TILEMAP_POS_X2),
            in_room_kind: read_le_u16(ram, KIND_OF_IN_ROOM_STAIRCASE),
            countdown: ram
                .get(COUNTDOWN_TIMER_FOR_STAIRCASES)
                .copied()
                .unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNG_CUR_FLOOR, self.current_floor_word);
        ram[DUNG_CUR_FLOOR_CACHED] = self.cached_floor;
        write_le_u16(ram, WHICH_STAIRCASE_INDEX, self.staircase_index);
        ram[STAIRCASE_MOVE_COUNTER] = self.move_counter;
        ram[CUR_STAIRCASE_PLANE] = self.current_plane;
        ram[STAIRCASE_LOWER_LEVEL_STATUS] = self.lower_level_status;
        write_le_u16(ram, STAIRCASE_TILEMAP_POS_X2, self.tilemap_pos_x2);
        write_le_u16(ram, KIND_OF_IN_ROOM_STAIRCASE, self.in_room_kind);
        ram[COUNTDOWN_TIMER_FOR_STAIRCASES] = self.countdown;
    }

    pub(crate) fn current_floor(&self) -> u8 {
        self.current_floor_word as u8
    }

    pub(crate) fn current_floor_word(&self) -> u16 {
        self.current_floor_word
    }

    pub(crate) fn cached_floor(&self) -> u8 {
        self.cached_floor
    }

    pub(crate) fn current_staircase_plane(&self) -> u8 {
        self.current_plane
    }

    pub(crate) fn staircase_lower_level_status(&self) -> u8 {
        self.lower_level_status
    }

    pub(crate) fn staircase_index(&self) -> u8 {
        self.staircase_index as u8
    }

    pub(crate) fn staircase_index_slot(&self) -> usize {
        usize::from(self.staircase_index() & 3)
    }

    pub(crate) fn staircase_index_has_vertical_bit(&self) -> bool {
        self.staircase_index() & 4 != 0
    }

    pub(crate) fn staircase_move_counter(&self) -> u8 {
        self.move_counter
    }

    pub(crate) fn kind_of_in_room_staircase(&self) -> u8 {
        self.in_room_kind as u8
    }

    pub(crate) fn staircase_tilemap_pos_x2(&self) -> u16 {
        self.tilemap_pos_x2
    }

    pub(crate) fn staircase_countdown(&self) -> u8 {
        self.countdown
    }

    fn set_current_floor(&mut self, value: u8) {
        self.current_floor_word = (self.current_floor_word & 0xff00) | u16::from(value);
    }

    fn decrement_current_floor(&mut self) -> u8 {
        let next = self.current_floor().wrapping_sub(1);
        self.set_current_floor(next);
        next
    }

    fn increment_current_floor(&mut self) -> u8 {
        let next = self.current_floor().wrapping_add(1);
        self.set_current_floor(next);
        next
    }

    fn cache_current_floor(&mut self) {
        self.cached_floor = self.current_floor();
    }

    fn restore_cached_floor(&mut self) {
        self.set_current_floor(self.cached_floor);
    }

    fn set_staircase_tilemap_pos_x2(&mut self, value: u16) {
        self.tilemap_pos_x2 = value;
    }

    fn set_current_staircase_plane(&mut self, value: u8) {
        self.current_plane = value;
    }

    fn set_staircase_lower_level_status(&mut self, value: u8) {
        self.lower_level_status = value;
    }

    fn set_staircase_countdown(&mut self, value: u8) {
        self.countdown = value;
    }

    fn decrement_staircase_countdown_clamped(&mut self) -> u8 {
        let value = self.countdown.wrapping_sub(1);
        self.countdown = if (value as i8).is_negative() {
            0
        } else {
            value
        };
        self.countdown
    }

    fn decrement_staircase_countdown_underflowed(&mut self) -> bool {
        let value = self.countdown.wrapping_sub(1);
        let underflowed = (value as i8).is_negative();
        self.countdown = if underflowed { 0 } else { value };
        underflowed
    }

    fn set_staircase_index(&mut self, value: u8) {
        self.staircase_index = (self.staircase_index & 0xff00) | u16::from(value);
    }

    fn set_staircase_index_high(&mut self, value: u8) {
        self.staircase_index = (self.staircase_index & 0x00ff) | (u16::from(value) << 8);
    }

    fn set_staircase_move_counter(&mut self, value: u8) {
        self.move_counter = value;
    }

    fn decrement_staircase_move_counter(&mut self) -> u8 {
        self.move_counter = self.move_counter.wrapping_sub(1);
        self.move_counter
    }

    fn set_kind_of_in_room_staircase_word(&mut self, value: u16) {
        self.in_room_kind = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonStairListsState {
    counters: [u16; DUNGEON_STAIR_LIST_COUNT],
    inter_staircases: [u16; DUNGEON_INTER_STAIRCASE_TABLE_WORDS],
    stairs_table_1: [u16; DUNGEON_STAIR_TABLE_1_WORDS],
    stairs_table_2: [u16; DUNGEON_STAIR_TABLE_2_WORDS],
}

impl DungeonStairListsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut counters = [0; DUNGEON_STAIR_LIST_COUNT];
        for list in ALL_DUNGEON_STAIR_LISTS {
            counters[stair_list_index(list)] = read_le_u16(ram, stair_list_counter_address(list));
        }

        let mut inter_staircases = [0; DUNGEON_INTER_STAIRCASE_TABLE_WORDS];
        for (index, position) in inter_staircases.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_INTER_STAIRCASES + index * 2);
        }

        let mut stairs_table_1 = [0; DUNGEON_STAIR_TABLE_1_WORDS];
        for (index, position) in stairs_table_1.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_STAIRS_TABLE_1 + index * 2);
        }

        let mut stairs_table_2 = [0; DUNGEON_STAIR_TABLE_2_WORDS];
        for (index, position) in stairs_table_2.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_STAIRS_TABLE_2 + index * 2);
        }

        Self {
            counters,
            inter_staircases,
            stairs_table_1,
            stairs_table_2,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for list in ALL_DUNGEON_STAIR_LISTS {
            write_le_u16(
                ram,
                stair_list_counter_address(list),
                self.stair_list_count(list),
            );
        }

        for (index, position) in self.inter_staircases.iter().enumerate() {
            write_le_u16(ram, DUNG_INTER_STAIRCASES + index * 2, *position);
        }

        for (index, position) in self.stairs_table_1.iter().enumerate() {
            write_le_u16(ram, DUNG_STAIRS_TABLE_1 + index * 2, *position);
        }

        for (index, position) in self.stairs_table_2.iter().enumerate() {
            write_le_u16(ram, DUNG_STAIRS_TABLE_2 + index * 2, *position);
        }
    }

    pub(crate) fn stair_list_count(&self, list: DungeonStairList) -> u16 {
        self.counters[stair_list_index(list)]
    }

    pub(crate) fn stair_list_tilemap_pos(&self, list: DungeonStairList, offset_x2: u16) -> u16 {
        let index = usize::from(offset_x2 >> 1);
        match stair_list_table(list) {
            DungeonStairTilemapTable::Stairs1 => {
                self.stairs_table_1.get(index).copied().unwrap_or(0)
            }
            DungeonStairTilemapTable::Stairs2 => {
                self.stairs_table_2.get(index).copied().unwrap_or(0)
            }
        }
    }

    pub(crate) fn inter_staircase_pos(&self, index: usize) -> u16 {
        self.inter_staircases.get(index).copied().unwrap_or(0)
    }

    fn set_stair_list_count(&mut self, list: DungeonStairList, value: u16) {
        self.counters[stair_list_index(list)] = value;
    }

    fn sync_stair_list_counts(&mut self, lists: &[DungeonStairList], value: u16) {
        for &list in lists {
            self.set_stair_list_count(list, value);
        }
    }

    fn append_interroom_staircase(&mut self, list: DungeonStairList, tilemap_pos: u16) -> u16 {
        let index = usize::from(self.stair_list_count(list)) >> 1;
        if let Some(position) = self.inter_staircases.get_mut(index) {
            *position = tilemap_pos;
        }
        self.stair_list_count(list).wrapping_add(2)
    }

    fn append_bg1_stair_table_position(&mut self, list: DungeonStairList, tilemap_pos: u16) -> u16 {
        let index = usize::from(self.stair_list_count(list)) >> 1;
        if let Some(position) = self.stairs_table_1.get_mut(index) {
            *position = tilemap_pos;
        }
        let next = self.stair_list_count(list).wrapping_add(2);
        self.set_stair_list_count(list, next);
        next
    }

    fn append_stair_table_position(&mut self, list: DungeonStairList, tilemap_pos: u16) -> u16 {
        let index = usize::from(self.stair_list_count(list)) >> 1;
        match stair_list_table(list) {
            DungeonStairTilemapTable::Stairs1 => {
                if let Some(position) = self.stairs_table_1.get_mut(index) {
                    *position = tilemap_pos;
                }
            }
            DungeonStairTilemapTable::Stairs2 => {
                if let Some(position) = self.stairs_table_2.get_mut(index) {
                    *position = tilemap_pos;
                }
            }
        }
        let next = self.stair_list_count(list).wrapping_add(2);
        self.set_stair_list_count(list, next);
        next
    }

    fn promote_water_stairs_to_active(&mut self) {
        let north_stairs = self.stair_list_count(DungeonStairList::InRoomUpNorthWater);
        let active_ladders = self.stair_list_count(DungeonStairList::ActivatedWaterLadders);
        let south_stairs = self.stair_list_count(DungeonStairList::InRoomUpSouthWater);
        self.set_stair_list_count(DungeonStairList::InterPseudoUpNorth, north_stairs);
        self.set_stair_list_count(DungeonStairList::WaterSideStepSwitch, active_ladders);
        self.set_stair_list_count(DungeonStairList::ActivatedWaterLadders, 0);
        self.set_stair_list_count(DungeonStairList::InRoomUpNorthWater, 0);
        self.set_stair_list_count(DungeonStairList::WetStairs, south_stairs);
        self.set_stair_list_count(DungeonStairList::InRoomUpSouthWater, 0);
    }

    fn set_inter_staircase_pos(&mut self, index: usize, value: u16) {
        if let Some(position) = self.inter_staircases.get_mut(index) {
            *position = value;
        }
    }
}

const ALL_DUNGEON_STAIR_LISTS: [DungeonStairList; DUNGEON_STAIR_LIST_COUNT] = [
    DungeonStairList::InterRoomUpNorth,
    DungeonStairList::InterRoomSouthDown,
    DungeonStairList::InRoomUpNorth,
    DungeonStairList::InRoomSouthDown,
    DungeonStairList::InterPseudoUpNorth,
    DungeonStairList::InRoomUpNorthWater,
    DungeonStairList::ActivatedWaterLadders,
    DungeonStairList::WetStairs,
    DungeonStairList::InRoomUpSouthWater,
    DungeonStairList::Stairs1,
    DungeonStairList::Stairs2,
    DungeonStairList::WaterLadders,
    DungeonStairList::WaterSideStepSwitch,
    DungeonStairList::WallUpNorthSpiral,
    DungeonStairList::WallDownNorthSpiral,
    DungeonStairList::WallUpNorthSpiralBg1,
    DungeonStairList::WallDownNorthSpiralBg1,
    DungeonStairList::InterRoomUpNorthStraight,
    DungeonStairList::InterRoomUpSouthStraight,
    DungeonStairList::InterRoomDownNorthStraight,
    DungeonStairList::InterRoomDownSouthStraight,
];

#[derive(Clone, Copy)]
enum DungeonStairTilemapTable {
    Stairs1,
    Stairs2,
}

fn stair_list_index(list: DungeonStairList) -> usize {
    match list {
        DungeonStairList::InterRoomUpNorth => 0,
        DungeonStairList::InterRoomSouthDown => 1,
        DungeonStairList::InRoomUpNorth => 2,
        DungeonStairList::InRoomSouthDown => 3,
        DungeonStairList::InterPseudoUpNorth => 4,
        DungeonStairList::InRoomUpNorthWater => 5,
        DungeonStairList::ActivatedWaterLadders => 6,
        DungeonStairList::WetStairs => 7,
        DungeonStairList::InRoomUpSouthWater => 8,
        DungeonStairList::Stairs1 => 9,
        DungeonStairList::Stairs2 => 10,
        DungeonStairList::WaterLadders => 11,
        DungeonStairList::WaterSideStepSwitch => 12,
        DungeonStairList::WallUpNorthSpiral => 13,
        DungeonStairList::WallDownNorthSpiral => 14,
        DungeonStairList::WallUpNorthSpiralBg1 => 15,
        DungeonStairList::WallDownNorthSpiralBg1 => 16,
        DungeonStairList::InterRoomUpNorthStraight => 17,
        DungeonStairList::InterRoomUpSouthStraight => 18,
        DungeonStairList::InterRoomDownNorthStraight => 19,
        DungeonStairList::InterRoomDownSouthStraight => 20,
    }
}

fn stair_list_counter_address(list: DungeonStairList) -> usize {
    match list {
        DungeonStairList::InterRoomUpNorth => DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS_LOCAL,
        DungeonStairList::InterRoomSouthDown => DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS_LOCAL,
        DungeonStairList::InRoomUpNorth => DUNG_NUM_INROOM_UPNORTH_STAIRS,
        DungeonStairList::InRoomSouthDown => DUNG_NUM_INROOM_SOUTHDOWN_STAIRS_LOCAL,
        DungeonStairList::InterPseudoUpNorth => DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS,
        DungeonStairList::InRoomUpNorthWater => DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER,
        DungeonStairList::ActivatedWaterLadders => DUNG_NUM_ACTIVATED_WATER_LADDERS,
        DungeonStairList::WetStairs => DUNG_NUM_STAIRS_WET,
        DungeonStairList::InRoomUpSouthWater => DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER,
        DungeonStairList::Stairs1 => DUNG_NUM_STAIRS_1,
        DungeonStairList::Stairs2 => DUNG_NUM_STAIRS_2,
        DungeonStairList::WaterLadders => DUNG_NUM_WATER_LADDERS_LOCAL,
        DungeonStairList::WaterSideStepSwitch => WATER_SIDE_STEP_SWITCH,
        DungeonStairList::WallUpNorthSpiral => DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_LOCAL,
        DungeonStairList::WallDownNorthSpiral => DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_LOCAL,
        DungeonStairList::WallUpNorthSpiralBg1 => DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2_LOCAL,
        DungeonStairList::WallDownNorthSpiralBg1 => DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2_LOCAL,
        DungeonStairList::InterRoomUpNorthStraight => {
            DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS_LOCAL
        }
        DungeonStairList::InterRoomUpSouthStraight => {
            DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS_LOCAL
        }
        DungeonStairList::InterRoomDownNorthStraight => {
            DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS_LOCAL
        }
        DungeonStairList::InterRoomDownSouthStraight => {
            DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS_LOCAL
        }
    }
}

fn stair_list_table(list: DungeonStairList) -> DungeonStairTilemapTable {
    match list {
        DungeonStairList::WetStairs
        | DungeonStairList::InRoomUpSouthWater
        | DungeonStairList::Stairs1
        | DungeonStairList::Stairs2 => DungeonStairTilemapTable::Stairs2,
        _ => DungeonStairTilemapTable::Stairs1,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonBg2AttributeState {
    attrs: Vec<u8>,
}

impl Default for DungeonBg2AttributeState {
    fn default() -> Self {
        Self {
            attrs: vec![0; DUNGEON_BG2_ATTR_BUFFER_LEN],
        }
    }
}

impl DungeonBg2AttributeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut attrs = vec![0; DUNGEON_BG2_ATTR_BUFFER_LEN];
        let available = ram.len().saturating_sub(DUNGEON_BG2_ATTR_TABLE);
        let len = attrs.len().min(available);
        attrs[..len].copy_from_slice(&ram[DUNGEON_BG2_ATTR_TABLE..DUNGEON_BG2_ATTR_TABLE + len]);
        Self { attrs }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        let available = ram.len().saturating_sub(DUNGEON_BG2_ATTR_TABLE);
        let len = self.attrs.len().min(available);
        ram[DUNGEON_BG2_ATTR_TABLE..DUNGEON_BG2_ATTR_TABLE + len]
            .copy_from_slice(&self.attrs[..len]);
    }

    pub(crate) fn bg2_attr(&self, offset: usize) -> u8 {
        self.attrs.get(offset).copied().unwrap_or(0)
    }

    pub(crate) fn bg2_attr_word(&self, offset: usize) -> u16 {
        u16::from(self.bg2_attr(offset)) | (u16::from(self.bg2_attr(offset + 1)) << 8)
    }

    pub(crate) fn bg2_attr_address(&self, offset: usize) -> usize {
        DUNGEON_BG2_ATTR_TABLE + offset
    }

    pub(crate) fn bg2_attr_pair(&self, offset: usize) -> Option<(u8, u8)> {
        Some((
            *self.attrs.get(offset)?,
            *self.attrs.get(offset.wrapping_add(1))?,
        ))
    }

    pub(crate) fn bg2_attr_slice(&self, start: usize, len: usize) -> &[u8] {
        &self.attrs[start..start + len]
    }

    fn set_bg2_attr(&mut self, offset: usize, value: u8) {
        self.attrs[offset] = value;
    }

    fn set_bg2_attr_word(&mut self, offset: usize, value: u16) {
        self.attrs[offset] = value as u8;
        self.attrs[offset + 1] = (value >> 8) as u8;
    }

    fn xor_bg2_attr(&mut self, offset: usize, value: u8) {
        self.attrs[offset] ^= value;
    }

    fn fill_bg2_attr_range(&mut self, start: usize, len: usize, value: u8) {
        self.attrs[start..start + len].fill(value);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonSavegameState {
    state_bits: u16,
}

impl DungeonSavegameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            state_bits: read_le_u16(ram, DUNG_SAVEGAME_STATE_BITS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNG_SAVEGAME_STATE_BITS, self.state_bits);
    }

    pub(crate) fn savegame_state_bits(&self) -> u16 {
        self.state_bits
    }

    pub(crate) fn has_savegame_state_bits(&self, mask: u16) -> bool {
        self.state_bits & mask != 0
    }

    fn set_savegame_state_bits(&mut self, value: u16) {
        self.state_bits = value;
    }

    fn clear_savegame_state_bits(&mut self) {
        self.state_bits = 0;
    }

    fn or_savegame_state_bits(&mut self, mask: u16) -> u16 {
        self.state_bits |= mask;
        self.state_bits
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonTorchState {
    timers: [u8; DUNGEON_TORCH_TIMER_COUNT],
    attr: u8,
    ganon_torch_count: u8,
    torches_start_index: u16,
    object_data_positions: [u16; DUNGEON_TORCH_OBJECT_POS_COUNT],
}

impl DungeonTorchState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut timers = [0; DUNGEON_TORCH_TIMER_COUNT];
        for (index, timer) in timers.iter_mut().enumerate() {
            *timer = ram.get(TORCH_TIMERS + index).copied().unwrap_or(0);
        }

        let mut object_data_positions = [0; DUNGEON_TORCH_OBJECT_POS_COUNT];
        for (index, position) in object_data_positions.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2);
        }

        Self {
            timers,
            attr: ram.get(DUNGEON_TORCH_ATTR).copied().unwrap_or(0),
            ganon_torch_count: ram.get(GANON_TORCH_COUNT).copied().unwrap_or(0),
            torches_start_index: read_le_u16(ram, DUNG_INDEX_OF_TORCHES_START),
            object_data_positions,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[TORCH_TIMERS..TORCH_TIMERS + DUNGEON_TORCH_TIMER_COUNT].copy_from_slice(&self.timers);
        ram[DUNGEON_TORCH_ATTR] = self.attr;
        ram[GANON_TORCH_COUNT] = self.ganon_torch_count;
        write_le_u16(ram, DUNG_INDEX_OF_TORCHES_START, self.torches_start_index);
        for (index, position) in self.object_data_positions.iter().enumerate() {
            write_le_u16(ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2, *position);
        }
    }

    pub(crate) fn timer(&self, index: usize) -> u8 {
        self.timers.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn attr_index(&self) -> usize {
        usize::from(self.attr & 0x0f)
    }

    pub(crate) fn torch_attr(&self) -> u8 {
        self.attr
    }

    pub(crate) fn ganon_torch_count(&self) -> u8 {
        self.ganon_torch_count
    }

    pub(crate) fn torches_start_index(&self) -> u16 {
        self.torches_start_index
    }

    pub(crate) fn torch_object_data_pos(&self, index: usize) -> u16 {
        self.object_data_positions.get(index).copied().unwrap_or(0)
    }

    fn clear_timer(&mut self, index: usize) {
        if let Some(timer) = self.timers.get_mut(index) {
            *timer = 0;
        }
    }

    fn set_timer(&mut self, index: usize, value: u8) {
        if let Some(timer) = self.timers.get_mut(index) {
            *timer = value;
        }
    }

    fn set_attr(&mut self, value: u8) {
        self.attr = value;
    }

    fn clear_attr(&mut self) {
        self.attr = 0;
    }

    fn set_ganon_torch_count(&mut self, value: u8) {
        self.ganon_torch_count = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonEntranceBackupState {
    exit_tile_themes: [u8; 4],
    overworld_screen_high: u8,
    overlay_high: u8,
}

impl DungeonEntranceBackupState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut exit_tile_themes = [0; 4];
        for (index, theme) in exit_tile_themes.iter_mut().enumerate() {
            *theme = ram
                .get(OVERWORLD_EXIT_TILE_THEME_INDEX + index)
                .copied()
                .unwrap_or(0);
        }
        Self {
            exit_tile_themes,
            overworld_screen_high: ram.get(OVERWORLD_SCREEN_INDEX + 1).copied().unwrap_or(0),
            overlay_high: ram.get(OVERLAY_INDEX + 1).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[OVERWORLD_EXIT_TILE_THEME_INDEX..OVERWORLD_EXIT_TILE_THEME_INDEX + 4]
            .copy_from_slice(&self.exit_tile_themes);
        ram[OVERWORLD_SCREEN_INDEX + 1] = self.overworld_screen_high;
        ram[OVERLAY_INDEX + 1] = self.overlay_high;
    }

    pub(crate) fn exit_tile_theme(&self, index: usize) -> u8 {
        self.exit_tile_themes.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn overworld_screen_high(&self) -> u8 {
        self.overworld_screen_high
    }

    pub(crate) fn overlay_high(&self) -> u8 {
        self.overlay_high
    }

    pub(crate) fn cache_exit_tile_themes(&mut self, overworld: u8, main: u8, aux: u8, sprite: u8) {
        self.exit_tile_themes = [overworld, main, aux, sprite];
    }

    pub(crate) fn clear_overworld_screen_high(&mut self) {
        self.overworld_screen_high = 0;
    }

    pub(crate) fn clear_overlay_high(&mut self) {
        self.overlay_high = 0;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonHeaderState {
    tags: [u8; DUNGEON_HEADER_TAG_COUNT],
    travel_destinations: [u8; DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT],
    plane_scratch: [u8; DUNGEON_HEADER_PLANE_SCRATCH_COUNT],
}

impl DungeonHeaderState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut tags = [0; DUNGEON_HEADER_TAG_COUNT];
        for (index, tag) in tags.iter_mut().enumerate() {
            *tag = ram.get(DUNGEON_HEADER_TAG + index).copied().unwrap_or(0);
        }

        let mut travel_destinations = [0; DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT];
        for (index, destination) in travel_destinations.iter_mut().enumerate() {
            *destination = ram
                .get(DUNGEON_HEADER_TRAVEL_DESTINATIONS + index)
                .copied()
                .unwrap_or(0);
        }

        let mut plane_scratch = [0; DUNGEON_HEADER_PLANE_SCRATCH_COUNT];
        for (index, plane) in plane_scratch.iter_mut().enumerate() {
            *plane = ram
                .get(DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + index)
                .copied()
                .unwrap_or(0);
        }

        Self {
            tags,
            travel_destinations,
            plane_scratch,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DUNGEON_HEADER_TAG..DUNGEON_HEADER_TAG + DUNGEON_HEADER_TAG_COUNT]
            .copy_from_slice(&self.tags);
        ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS
            ..DUNGEON_HEADER_TRAVEL_DESTINATIONS + DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT]
            .copy_from_slice(&self.travel_destinations);
        ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE
            ..DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + DUNGEON_HEADER_PLANE_SCRATCH_COUNT]
            .copy_from_slice(&self.plane_scratch);
    }

    pub(crate) fn header_tag(&self, index: usize) -> u8 {
        self.tags.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn primary_header_tag(&self) -> u8 {
        self.header_tag(0)
    }

    pub(crate) fn travel_destination(&self, index: usize) -> u8 {
        self.travel_destinations.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn hole_teleporter_plane(&self, index: usize) -> u8 {
        self.plane_scratch.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn staircase_plane(&self, index: usize) -> u8 {
        self.plane_scratch
            .get(DUNGEON_HEADER_STAIRCASE_PLANE - DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + index)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn set_hole_teleporter_planes(&mut self, packed: u8, extra: u8) {
        self.plane_scratch[0] = packed & 3;
        self.plane_scratch[1] = (packed >> 2) & 3;
        self.plane_scratch[2] = (packed >> 4) & 3;
        self.plane_scratch[3] = (packed >> 6) & 3;
        self.plane_scratch[4] = extra & 3;
    }

    fn set_header_tag(&mut self, index: usize, value: u8) {
        self.tags[index] = value;
    }

    fn clear_header_tag(&mut self, index: usize) {
        self.set_header_tag(index, 0);
    }

    fn clear_header_tags(&mut self, count: usize) {
        self.tags[..count].fill(0);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonScratchWordState {
    r16: u16,
    r18: u16,
}

impl DungeonScratchWordState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            r16: read_le_u16(ram, DUNGEON_WORK_R16),
            r18: read_le_u16(ram, DUNGEON_WORK_R18),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNGEON_WORK_R16, self.r16);
        write_le_u16(ram, DUNGEON_WORK_R18, self.r18);
    }

    pub(crate) fn high(&self) -> u8 {
        (self.r16 >> 8) as u8
    }

    pub(crate) fn word(&self) -> u16 {
        self.r16
    }

    pub(crate) fn minigame_previous_chest_choice(&self) -> u8 {
        self.r16 as u8
    }

    pub(crate) fn primary_word(&self) -> u16 {
        self.r16
    }

    pub(crate) fn secondary_word(&self) -> u16 {
        self.r18
    }

    pub(crate) fn primary_low(&self) -> u8 {
        self.r16 as u8
    }

    pub(crate) fn secondary_low(&self) -> u8 {
        self.r18 as u8
    }

    pub(crate) fn decrement_high(&mut self) -> u8 {
        let next = self.high().wrapping_sub(1);
        self.r16 = (self.r16 & 0x00ff) | (u16::from(next) << 8);
        next
    }

    pub(crate) fn set_word(&mut self, value: u16) {
        self.r16 = value;
    }

    pub(crate) fn clear_word(&mut self) {
        self.set_word(0);
    }

    pub(crate) fn set_liftable_tile_probe_position(&mut self, y: u16, x: u16) {
        self.r16 = y;
        self.r18 = x;
    }

    pub(crate) fn set_ganon_door_bounce_countdown(&mut self, value: u16) {
        self.set_word(value);
    }

    pub(crate) fn decrement_ganon_door_bounce_low(&mut self) -> u8 {
        let next = (self.r16 as u8).wrapping_sub(1);
        self.r16 = (self.r16 & 0xff00) | u16::from(next);
        next
    }

    pub(crate) fn clear_module_transition_counter(&mut self) {
        self.r16 = (self.r16 & 0xff00) | 0;
    }

    pub(crate) fn set_minigame_previous_chest_choice(&mut self, value: u8) {
        self.r16 = (self.r16 & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_primary_word(&mut self, value: u16) {
        self.r16 = value;
    }

    pub(crate) fn set_secondary_word(&mut self, value: u16) {
        self.r18 = value;
    }

    pub(crate) fn clear_primary_word(&mut self) {
        self.set_primary_word(0);
    }

    pub(crate) fn set_primary_low(&mut self, value: u8) {
        self.r16 = (self.r16 & 0xff00) | u16::from(value);
    }

    pub(crate) fn decrement_primary_low(&mut self) -> u8 {
        let next = self.primary_low().wrapping_sub(1);
        self.set_primary_low(next);
        next
    }

    pub(crate) fn increment_secondary_low(&mut self) -> u8 {
        let next = self.secondary_low().wrapping_add(1);
        self.r18 = (self.r18 & 0xff00) | u16::from(next);
        next
    }
}

pub(crate) struct NativeDungeonEntranceBackupBridgeMut<'a> {
    state: &'a mut DungeonEntranceBackupState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonEntranceBackupBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonEntranceBackupState, ram: &'a mut [u8]) -> Self {
        *state = DungeonEntranceBackupState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonEntranceBackupState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn cache_exit_tile_themes(&mut self) {
        self.state.cache_exit_tile_themes(
            self.ram[OVERWORLD_TILE_THEME_INDEX],
            self.ram[MAIN_TILE_THEME_INDEX],
            self.ram[AUX_TILE_THEME_INDEX],
            self.ram[SPRITE_GRAPHICS_INDEX],
        );
        self.sync();
    }

    pub(crate) fn clear_overworld_screen_high(&mut self) {
        self.state.clear_overworld_screen_high();
        self.sync();
    }

    pub(crate) fn clear_overlay_high(&mut self) {
        self.state.clear_overlay_high();
        self.sync();
    }
}

pub(crate) struct NativeDungeonScratchWordBridgeMut<'a> {
    scratch: &'a mut DungeonScratchWordState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonScratchWordBridgeMut<'a> {
    pub(crate) fn new(scratch: &'a mut DungeonScratchWordState, ram: &'a mut [u8]) -> Self {
        *scratch = DungeonScratchWordState::load_from_ram(ram);
        Self { scratch, ram }
    }

    fn sync(&mut self) {
        self.scratch.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.scratch,
            DungeonScratchWordState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn decrement_high(&mut self) -> u8 {
        let next = self.scratch.decrement_high();
        self.sync();
        next
    }

    pub(crate) fn set_word(&mut self, value: u16) {
        self.scratch.set_word(value);
        self.sync();
    }

    pub(crate) fn clear_word(&mut self) {
        self.scratch.clear_word();
        self.sync();
    }

    pub(crate) fn set_liftable_tile_probe_position(&mut self, y: u16, x: u16) {
        self.scratch.set_liftable_tile_probe_position(y, x);
        self.sync();
    }

    pub(crate) fn set_ganon_door_bounce_countdown(&mut self, value: u16) {
        self.scratch.set_ganon_door_bounce_countdown(value);
        self.sync();
    }

    pub(crate) fn decrement_ganon_door_bounce_low(&mut self) -> u8 {
        let next = self.scratch.decrement_ganon_door_bounce_low();
        self.sync();
        next
    }

    pub(crate) fn clear_module_transition_counter(&mut self) {
        self.scratch.clear_module_transition_counter();
        self.sync();
    }

    pub(crate) fn set_minigame_previous_chest_choice(&mut self, value: u8) {
        self.scratch.set_minigame_previous_chest_choice(value);
        self.sync();
    }

    pub(crate) fn set_primary_word(&mut self, value: u16) {
        self.scratch.set_primary_word(value);
        self.sync();
    }

    pub(crate) fn set_secondary_word(&mut self, value: u16) {
        self.scratch.set_secondary_word(value);
        self.sync();
    }

    pub(crate) fn clear_primary_word(&mut self) {
        self.scratch.clear_primary_word();
        self.sync();
    }

    pub(crate) fn set_primary_low(&mut self, value: u8) {
        self.scratch.set_primary_low(value);
        self.sync();
    }

    pub(crate) fn decrement_primary_low(&mut self) -> u8 {
        let next = self.scratch.decrement_primary_low();
        self.sync();
        next
    }

    pub(crate) fn increment_secondary_low(&mut self) -> u8 {
        let next = self.scratch.increment_secondary_low();
        self.sync();
        next
    }
}

pub(crate) struct NativeDungeonSavegameBridgeMut<'a> {
    state: &'a mut DungeonSavegameState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonSavegameBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonSavegameState, ram: &'a mut [u8]) -> Self {
        *state = DungeonSavegameState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DungeonSavegameState::load_from_ram(self.ram));
    }

    pub(crate) fn set_savegame_state_bits(&mut self, value: u16) {
        self.state.set_savegame_state_bits(value);
        self.sync();
    }

    pub(crate) fn clear_savegame_state_bits(&mut self) {
        self.state.clear_savegame_state_bits();
        self.sync();
    }

    pub(crate) fn or_savegame_state_bits(&mut self, mask: u16) -> u16 {
        let value = self.state.or_savegame_state_bits(mask);
        self.sync();
        value
    }
}

pub(crate) struct NativeDungeonBg2AttributeBridgeMut<'a> {
    state: &'a mut DungeonBg2AttributeState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonBg2AttributeBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonBg2AttributeState, ram: &'a mut [u8]) -> Self {
        *state = DungeonBg2AttributeState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonBg2AttributeState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_bg2_attr(&mut self, offset: usize, value: u8) {
        self.state.set_bg2_attr(offset, value);
        self.sync();
    }

    pub(crate) fn set_bg2_attr_word(&mut self, offset: usize, value: u16) {
        self.state.set_bg2_attr_word(offset, value);
        self.sync();
    }

    pub(crate) fn xor_bg2_attr(&mut self, offset: usize, value: u8) {
        self.state.xor_bg2_attr(offset, value);
        self.sync();
    }

    pub(crate) fn fill_bg2_attr_range(&mut self, start: usize, len: usize, value: u8) {
        self.state.fill_bg2_attr_range(start, len, value);
        self.sync();
    }
}

pub(crate) struct NativeDungeonStairListsBridgeMut<'a> {
    state: &'a mut DungeonStairListsState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonStairListsBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonStairListsState, ram: &'a mut [u8]) -> Self {
        *state = DungeonStairListsState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DungeonStairListsState::load_from_ram(self.ram));
    }

    pub(crate) fn set_stair_list_count(&mut self, list: DungeonStairList, value: u16) {
        self.state.set_stair_list_count(list, value);
        self.sync();
    }

    pub(crate) fn sync_stair_list_counts(&mut self, lists: &[DungeonStairList], value: u16) {
        self.state.sync_stair_list_counts(lists, value);
        self.sync();
    }

    pub(crate) fn append_interroom_staircase(
        &mut self,
        list: DungeonStairList,
        tilemap_pos: u16,
    ) -> u16 {
        let next = self.state.append_interroom_staircase(list, tilemap_pos);
        self.sync();
        next
    }

    pub(crate) fn append_bg1_stair_table_position(
        &mut self,
        list: DungeonStairList,
        tilemap_pos: u16,
    ) -> u16 {
        let next = self
            .state
            .append_bg1_stair_table_position(list, tilemap_pos);
        self.sync();
        next
    }

    pub(crate) fn append_stair_table_position(
        &mut self,
        list: DungeonStairList,
        tilemap_pos: u16,
    ) -> u16 {
        let next = self.state.append_stair_table_position(list, tilemap_pos);
        self.sync();
        next
    }

    pub(crate) fn promote_water_stairs_to_active(&mut self) {
        self.state.promote_water_stairs_to_active();
        self.sync();
    }

    pub(crate) fn set_inter_staircase_pos(&mut self, index: usize, value: u16) {
        self.state.set_inter_staircase_pos(index, value);
        self.sync();
    }
}

pub(crate) struct NativeDungeonMovingFloorBridgeMut<'a> {
    state: &'a mut DungeonMovingFloorState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonMovingFloorBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonMovingFloorState, ram: &'a mut [u8]) -> Self {
        *state = DungeonMovingFloorState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonMovingFloorState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_floor_y_velocity_high(&mut self, value: u8) {
        self.state.set_floor_y_velocity_high(value);
        self.sync();
    }

    pub(crate) fn set_floor_y_velocity(&mut self, value: u16) {
        self.state.set_floor_y_velocity(value);
        self.sync();
    }

    pub(crate) fn set_floor_x_velocity(&mut self, value: u16) {
        self.state.set_floor_x_velocity(value);
        self.sync();
    }

    pub(crate) fn clear_floor_velocity(&mut self) {
        self.state.clear_floor_velocity();
        self.sync();
    }

    pub(crate) fn set_floor_x_offset(&mut self, value: u16) {
        self.state.set_floor_x_offset(value);
        self.sync();
    }

    pub(crate) fn set_floor_y_offset(&mut self, value: u16) {
        self.state.set_floor_y_offset(value);
        self.sync();
    }

    pub(crate) fn set_floor_y_offset_low(&mut self, value: u8) {
        self.state.set_floor_y_offset_low(value);
        self.sync();
    }

    pub(crate) fn set_floor_offsets(&mut self, x: u16, y: u16) {
        self.state.set_floor_offsets(x, y);
        self.sync();
    }

    pub(crate) fn add_floor_x_offset(&mut self, delta: u16) -> u16 {
        let value = self.state.add_floor_x_offset(delta);
        self.sync();
        value
    }

    pub(crate) fn sub_floor_x_offset(&mut self, delta: u16) -> u16 {
        let value = self.state.sub_floor_x_offset(delta);
        self.sync();
        value
    }

    pub(crate) fn add_floor_y_offset(&mut self, delta: u16) -> u16 {
        let value = self.state.add_floor_y_offset(delta);
        self.sync();
        value
    }

    pub(crate) fn sub_floor_y_offset(&mut self, delta: u16) -> u16 {
        let value = self.state.sub_floor_y_offset(delta);
        self.sync();
        value
    }

    pub(crate) fn clear_floor_offsets(&mut self) {
        self.state.clear_floor_offsets();
        self.sync();
    }

    pub(crate) fn clear_floor_move_flags(&mut self) {
        self.state.clear_floor_move_flags();
        self.sync();
    }

    pub(crate) fn set_floor_move_flags(&mut self, value: u16) {
        self.state.set_floor_move_flags(value);
        self.sync();
    }

    pub(crate) fn increment_floor_move_flags(&mut self) {
        self.state.increment_floor_move_flags();
        self.sync();
    }
}

pub(crate) struct NativeDungeonStairMovementBridgeMut<'a> {
    state: &'a mut DungeonStairMovementState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonStairMovementBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonStairMovementState, ram: &'a mut [u8]) -> Self {
        *state = DungeonStairMovementState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonStairMovementState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_current_floor(&mut self, value: u8) {
        self.state.set_current_floor(value);
        self.sync();
    }

    pub(crate) fn decrement_current_floor(&mut self) -> u8 {
        let value = self.state.decrement_current_floor();
        self.sync();
        value
    }

    pub(crate) fn increment_current_floor(&mut self) -> u8 {
        let value = self.state.increment_current_floor();
        self.sync();
        value
    }

    pub(crate) fn cache_current_floor(&mut self) {
        self.state.cache_current_floor();
        self.sync();
    }

    pub(crate) fn restore_cached_floor(&mut self) {
        self.state.restore_cached_floor();
        self.sync();
    }

    pub(crate) fn set_staircase_tilemap_pos_x2(&mut self, value: u16) {
        self.state.set_staircase_tilemap_pos_x2(value);
        self.sync();
    }

    pub(crate) fn set_current_staircase_plane(&mut self, value: u8) {
        self.state.set_current_staircase_plane(value);
        self.sync();
    }

    pub(crate) fn set_staircase_lower_level_status(&mut self, value: u8) {
        self.state.set_staircase_lower_level_status(value);
        self.sync();
    }

    pub(crate) fn set_staircase_countdown(&mut self, value: u8) {
        self.state.set_staircase_countdown(value);
        self.sync();
    }

    pub(crate) fn decrement_staircase_countdown_clamped(&mut self) -> u8 {
        let value = self.state.decrement_staircase_countdown_clamped();
        self.sync();
        value
    }

    pub(crate) fn decrement_staircase_countdown_underflowed(&mut self) -> bool {
        let underflowed = self.state.decrement_staircase_countdown_underflowed();
        self.sync();
        underflowed
    }

    pub(crate) fn set_staircase_index(&mut self, value: u8) {
        self.state.set_staircase_index(value);
        self.sync();
    }

    pub(crate) fn set_staircase_index_high(&mut self, value: u8) {
        self.state.set_staircase_index_high(value);
        self.sync();
    }

    pub(crate) fn set_staircase_move_counter(&mut self, value: u8) {
        self.state.set_staircase_move_counter(value);
        self.sync();
    }

    pub(crate) fn decrement_staircase_move_counter(&mut self) -> u8 {
        let value = self.state.decrement_staircase_move_counter();
        self.sync();
        value
    }

    pub(crate) fn set_kind_of_in_room_staircase_word(&mut self, value: u16) {
        self.state.set_kind_of_in_room_staircase_word(value);
        self.sync();
    }
}

pub(crate) struct NativeDungeonTorchBridgeMut<'a> {
    torch: &'a mut DungeonTorchState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonTorchBridgeMut<'a> {
    pub(crate) fn new(torch: &'a mut DungeonTorchState, ram: &'a mut [u8]) -> Self {
        *torch = DungeonTorchState::load_from_ram(ram);
        Self { torch, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.torch, DungeonTorchState::load_from_ram(self.ram));
    }

    pub(crate) fn copy_torch_init_to_movable_blocks(&mut self, torch_init: &[u8]) {
        self.ram[MOVABLE_BLOCK_DATAS + 99 * 4..MOVABLE_BLOCK_DATAS + 99 * 4 + 116]
            .copy_from_slice(&torch_init[..116]);
    }

    pub(crate) fn copy_torch_junk(&mut self, torch_junk: &[u8]) {
        self.ram[DUNGEON_TORCH_DATA + 144 * 2..DUNGEON_TORCH_DATA + 144 * 2 + torch_junk.len()]
            .copy_from_slice(torch_junk);
    }

    pub(crate) fn clear_timer(&mut self, index: usize) {
        self.torch.clear_timer(index);
        if index < DUNGEON_TORCH_TIMER_COUNT {
            self.ram[TORCH_TIMERS + index] = 0;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_timer(&mut self, index: usize, value: u8) {
        self.torch.set_timer(index, value);
        if index < DUNGEON_TORCH_TIMER_COUNT {
            self.ram[TORCH_TIMERS + index] = value;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_torch_data_word(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNGEON_TORCH_DATA + index * 2, value);
    }

    pub(crate) fn set_attr(&mut self, value: u8) {
        self.torch.set_attr(value);
        self.ram[DUNGEON_TORCH_ATTR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_ganon_torch_count(&mut self, value: u8) {
        self.torch.set_ganon_torch_count(value);
        self.ram[GANON_TORCH_COUNT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_attr(&mut self) {
        self.torch.clear_attr();
        self.ram[DUNGEON_TORCH_ATTR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn refresh_object_data_positions(&mut self) {
        *self.torch = DungeonTorchState::load_from_ram(self.ram);
    }
}

pub(crate) struct NativeDungeonHeaderBridgeMut<'a> {
    header: &'a mut DungeonHeaderState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonHeaderBridgeMut<'a> {
    pub(crate) fn new(header: &'a mut DungeonHeaderState, ram: &'a mut [u8]) -> Self {
        *header = DungeonHeaderState::load_from_ram(ram);
        Self { header, ram }
    }

    fn sync(&mut self) {
        self.header.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.header, DungeonHeaderState::load_from_ram(self.ram));
    }

    pub(crate) fn set_hole_teleporter_planes(&mut self, packed: u8, extra: u8) {
        self.header.set_hole_teleporter_planes(packed, extra);
        self.sync();
    }

    pub(crate) fn set_header_tag(&mut self, index: usize, value: u8) {
        self.header.set_header_tag(index, value);
        self.sync();
    }

    pub(crate) fn clear_header_tag(&mut self, index: usize) {
        self.header.clear_header_tag(index);
        self.sync();
    }

    pub(crate) fn clear_header_tags(&mut self, count: usize) {
        self.header.clear_header_tags(count);
        self.sync();
    }
}
