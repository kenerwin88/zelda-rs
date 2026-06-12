use super::*;

const DUNGEON_DRAW_OBJECT_OFFSETS_BG1: [u8; 33] = [
    0, 0x20, 0x7e, 2, 0x20, 0x7e, 4, 0x20, 0x7e, 6, 0x20, 0x7e, 0x80, 0x20, 0x7e, 0x82, 0x20, 0x7e,
    0x84, 0x20, 0x7e, 0x86, 0x20, 0x7e, 0, 0x21, 0x7e, 0x80, 0x21, 0x7e, 0, 0x22, 0x7e,
];
const DUNGEON_DRAW_OBJECT_OFFSETS_BG2: [u8; 33] = [
    0, 0x40, 0x7e, 2, 0x40, 0x7e, 4, 0x40, 0x7e, 6, 0x40, 0x7e, 0x80, 0x40, 0x7e, 0x82, 0x40, 0x7e,
    0x84, 0x40, 0x7e, 0x86, 0x40, 0x7e, 0, 0x41, 0x7e, 0x80, 0x41, 0x7e, 0, 0x42, 0x7e,
];
const DUNG_NUM_STAR_SHAPED_SWITCHES_LOCAL: usize = 0x0432;
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
const DUNG_INTER_STAIRCASE_TABLE_LOCAL: usize = 0x06b0;
const DUNG_STAIRS_TABLE_1_LOCAL: usize = 0x06b8;
const DUNG_STAIRS_TABLE_2_LOCAL: usize = 0x06ec;
const STAR_SHAPED_SWITCHES_TILE_LOCAL: usize = 0x06a0;
const POTS_REVEALED_IN_ROOM_DUNGEON_LOCAL: usize = 0x0f580;
const ROOM_BG1_TILEMAP_BASE_LOCAL: usize = 0x4000;
const ROOM_BG2_TILEMAP_BASE_LOCAL: usize = 0x2000;

#[derive(Clone, Copy)]
pub(crate) enum DungeonStairList {
    InterRoomUpNorth,
    InterRoomSouthDown,
    InRoomUpNorth,
    InRoomSouthDown,
    InterPseudoUpNorth,
    InRoomUpNorthWater,
    ActivatedWaterLadders,
    WetStairs,
    InRoomUpSouthWater,
    Stairs1,
    Stairs2,
    WaterLadders,
    WaterSideStepSwitch,
    WallUpNorthSpiral,
    WallDownNorthSpiral,
    WallUpNorthSpiralBg1,
    WallDownNorthSpiralBg1,
    InterRoomUpNorthStraight,
    InterRoomUpSouthStraight,
    InterRoomDownNorthStraight,
    InterRoomDownSouthStraight,
}

impl DungeonStairList {
    fn counter(self) -> usize {
        match self {
            Self::InterRoomUpNorth => DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS_LOCAL,
            Self::InterRoomSouthDown => DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS_LOCAL,
            Self::InRoomUpNorth => DUNG_NUM_INROOM_UPNORTH_STAIRS,
            Self::InRoomSouthDown => DUNG_NUM_INROOM_SOUTHDOWN_STAIRS_LOCAL,
            Self::InterPseudoUpNorth => DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS,
            Self::InRoomUpNorthWater => DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER,
            Self::ActivatedWaterLadders => DUNG_NUM_ACTIVATED_WATER_LADDERS,
            Self::WetStairs => DUNG_NUM_STAIRS_WET,
            Self::InRoomUpSouthWater => DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER,
            Self::Stairs1 => DUNG_NUM_STAIRS_1,
            Self::Stairs2 => DUNG_NUM_STAIRS_2,
            Self::WaterLadders => DUNG_NUM_WATER_LADDERS_LOCAL,
            Self::WaterSideStepSwitch => WATER_SIDE_STEP_SWITCH,
            Self::WallUpNorthSpiral => DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_LOCAL,
            Self::WallDownNorthSpiral => DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_LOCAL,
            Self::WallUpNorthSpiralBg1 => DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2_LOCAL,
            Self::WallDownNorthSpiralBg1 => DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2_LOCAL,
            Self::InterRoomUpNorthStraight => DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS_LOCAL,
            Self::InterRoomUpSouthStraight => DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS_LOCAL,
            Self::InterRoomDownNorthStraight => DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS_LOCAL,
            Self::InterRoomDownSouthStraight => DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS_LOCAL,
        }
    }

    fn tilemap_table(self) -> usize {
        match self {
            Self::WetStairs | Self::InRoomUpSouthWater | Self::Stairs1 | Self::Stairs2 => {
                DUNG_STAIRS_TABLE_2_LOCAL
            }
            _ => DUNG_STAIRS_TABLE_1_LOCAL,
        }
    }
}

pub(crate) struct DungeonStateView<'a> {
    ram: &'a [u8],
}

impl<'a> DungeonStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn header_tag(&self, index: usize) -> u8 {
        byte(self.ram, DUNGEON_HEADER_TAG + index)
    }

    pub(crate) fn primary_header_tag(&self) -> u8 {
        self.header_tag(0)
    }

    pub(crate) fn bg2_attr(&self, offset: usize) -> u8 {
        byte(self.ram, DUNGEON_BG2_ATTR_TABLE + offset)
    }

    pub(crate) fn bg2_attr_word(&self, offset: usize) -> u16 {
        word(self.ram, DUNGEON_BG2_ATTR_TABLE + offset)
    }

    pub(crate) fn bg1_attr(&self, offset: usize) -> u8 {
        byte(self.ram, DUNGEON_BG1_ATTR_TABLE + offset)
    }

    pub(crate) fn bg1_attr_word(&self, offset: usize) -> u16 {
        word(self.ram, DUNGEON_BG1_ATTR_TABLE + offset)
    }

    pub(crate) fn attr_for_tile(&self, tile: usize) -> u8 {
        byte(self.ram, ATTRIBUTES_FOR_TILE_PLAYER + (tile & 0x03ff))
    }

    pub(crate) fn header_collision_2_mirror_high(&self) -> u8 {
        byte(self.ram, DUNGEON_HEADER_COLLISION_2_MIRROR + 1)
    }

    pub(crate) fn header_collision(&self) -> u8 {
        byte(self.ram, DUNG_HDR_COLLISION)
    }

    pub(crate) fn header_collision_2(&self) -> u8 {
        byte(self.ram, DUNG_HDR_COLLISION_2)
    }

    pub(crate) fn header_collision_2_mirror(&self) -> u8 {
        byte(self.ram, DUNGEON_HEADER_COLLISION_2_MIRROR)
    }

    pub(crate) fn game_over_check_flag(&self) -> u8 {
        byte(self.ram, GAME_OVER_CHECK_FLAG)
    }

    pub(crate) fn restart_check_flag(&self) -> u16 {
        word(self.ram, RESTART_CHECK_FLAG)
    }

    pub(crate) fn starting_point(&self) -> u8 {
        byte(self.ram, WHICH_STARTING_POINT)
    }

    pub(crate) fn bg2_properties(&self) -> u8 {
        byte(self.ram, DUNG_HDR_BG2_PROPERTIES)
    }

    pub(crate) fn current_floor(&self) -> u8 {
        byte(self.ram, DUNG_CUR_FLOOR)
    }

    pub(crate) fn current_floor_word(&self) -> u16 {
        word(self.ram, DUNG_CUR_FLOOR)
    }

    pub(crate) fn blast_wall_x_open(&self) -> bool {
        byte(self.ram, DUNG_BLASTWALL_FLAG_X) != 0
    }

    pub(crate) fn blast_wall_y_open(&self) -> bool {
        byte(self.ram, DUNG_BLASTWALL_FLAG_Y) != 0
    }

    pub(crate) fn reset_xy_check_flags(&self) -> u16 {
        word(self.ram, RESET_XY_CHECK_FLAGS)
    }

    pub(crate) fn layout_quadrant_key(&self) -> u8 {
        byte(self.ram, COMPOSITE_OF_LAYOUT_AND_QUADRANT)
    }

    pub(crate) fn quadrants_visited(&self) -> u16 {
        word(self.ram, DUNG_QUADRANTS_VISITED)
    }

    pub(crate) fn cached_floor(&self) -> u8 {
        byte(self.ram, DUNG_CUR_FLOOR_CACHED)
    }

    pub(crate) fn floor_y_velocity(&self) -> u16 {
        word(self.ram, DUNGEON_FLOOR_Y_VELOCITY)
    }

    pub(crate) fn floor_y_velocity_low(&self) -> u8 {
        byte(self.ram, DUNGEON_FLOOR_Y_VELOCITY)
    }

    pub(crate) fn floor_x_velocity(&self) -> u16 {
        word(self.ram, DUNGEON_FLOOR_X_VELOCITY)
    }

    pub(crate) fn floor_x_velocity_low(&self) -> u8 {
        byte(self.ram, DUNGEON_FLOOR_X_VELOCITY)
    }

    pub(crate) fn lit_torches(&self) -> u8 {
        byte(self.ram, DUNG_NUM_LIT_TORCHES)
    }

    pub(crate) fn orange_blue_barrier_state(&self) -> u8 {
        byte(self.ram, ORANGE_BLUE_BARRIER_STATE)
    }

    pub(crate) fn wants_lights_out(&self) -> u8 {
        byte(self.ram, DUNG_WANT_LIGHTS_OUT)
    }

    pub(crate) fn wants_lights_out_copy(&self) -> u8 {
        byte(self.ram, DUNG_WANT_LIGHTS_OUT_COPY)
    }

    pub(crate) fn any_lights_out_request(&self) -> u8 {
        self.wants_lights_out() | self.wants_lights_out_copy()
    }

    pub(crate) fn dungeon_dark_with_lantern(&self) -> bool {
        byte(self.ram, HDR_DUNGEON_DARK_WITH_LANTERN) != 0
    }

    pub(crate) fn dungeon_dark_with_lantern_raw(&self) -> u8 {
        byte(self.ram, HDR_DUNGEON_DARK_WITH_LANTERN)
    }

    pub(crate) fn quadrant_upload_index(&self) -> u8 {
        byte(self.ram, DUNG_CUR_QUADRANT_UPLOAD)
    }

    pub(crate) fn trapdoors_down(&self) -> u16 {
        word(self.ram, DUNG_FLAG_TRAPDOORS_DOWN)
    }

    pub(crate) fn trapdoors_down_low(&self) -> u8 {
        byte(self.ram, DUNG_FLAG_TRAPDOORS_DOWN)
    }

    pub(crate) fn water_puzzle_state_changed(&self) -> u8 {
        byte(self.ram, DUNG_FLAG_STATECHANGE_WATERPUZZLE)
    }

    pub(crate) fn draw_width_indicator(&self) -> u8 {
        byte(self.ram, DUNG_DRAW_WIDTH_INDICATOR)
    }

    pub(crate) fn draw_width_indicator_word(&self) -> u16 {
        word(self.ram, DUNG_DRAW_WIDTH_INDICATOR)
    }

    pub(crate) fn draw_height_indicator(&self) -> u8 {
        byte(self.ram, DUNG_DRAW_HEIGHT_INDICATOR)
    }

    pub(crate) fn draw_height_indicator_word(&self) -> u16 {
        word(self.ram, DUNG_DRAW_HEIGHT_INDICATOR)
    }

    pub(crate) fn opened_doors_including_adjacent(&self) -> u16 {
        word(self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT)
    }

    pub(crate) fn opened_doors(&self) -> u16 {
        word(self.ram, DUNG_DOOR_OPENED)
    }

    pub(crate) fn floor_x_offset(&self) -> u16 {
        word(self.ram, DUNG_FLOOR_X_OFFS)
    }

    pub(crate) fn floor_y_offset(&self) -> u16 {
        word(self.ram, DUNG_FLOOR_Y_OFFS)
    }

    pub(crate) fn crush_wall_progress(&self) -> u16 {
        word(self.ram, CRUSH_WALL_PROGRESS)
    }

    pub(crate) fn moving_wall_dot_pointer(&self) -> u8 {
        byte(self.ram, MOVING_WALL_DOT_POINTER)
    }

    pub(crate) fn moving_wall_dot_index(&self) -> usize {
        usize::from(self.moving_wall_dot_pointer() >> 1) & 7
    }

    pub(crate) fn moving_wall_write_point(&self) -> u16 {
        word(self.ram, MOVING_WALL_WRITE_POINT)
    }

    pub(crate) fn dungeon_music_type_flag(&self) -> u8 {
        byte(self.ram, FLAG_WHICH_MUSIC_TYPE_DUNGEON)
    }

    pub(crate) fn fixed_color_plusminus(&self) -> u8 {
        byte(self.ram, OVERWORLD_FIXED_COLOR_PLUSMINUS)
    }

    pub(crate) fn overlay_to_load(&self) -> u8 {
        byte(self.ram, DUNG_OVERLAY_TO_LOAD)
    }

    pub(crate) fn moving_wall_torch_should_update(&self) -> bool {
        byte(self.ram, MOVING_WALL_TORCH_UPDATE_FLAG) != 0
    }

    pub(crate) fn movable_block_was_pushed(&self) -> u8 {
        byte(self.ram, DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED)
    }

    pub(crate) fn movable_block_was_not_pushed_flag(&self) -> u8 {
        self.movable_block_was_pushed() ^ 1
    }

    pub(crate) fn somaria_block_switch_counter(&self) -> u8 {
        byte(self.ram, DUNG_FLAG_SOMARIA_BLOCK_SWITCH)
    }

    pub(crate) fn door_switch_triggered(&self) -> bool {
        byte(self.ram, DUNG_DOOR_SWITCH_TRIGGERED) != 0
    }

    pub(crate) fn block_trap_related_tile(&self) -> u16 {
        word(self.ram, BLOCK_TRAP_CHECK_FLAG)
    }

    pub(crate) fn selected_key_door_x2(&self) -> u16 {
        word(self.ram, DUNG_WHICH_KEY_X2_DUNGEON)
    }

    pub(crate) fn blast_wall_door_index_x2(&self) -> u16 {
        word(self.ram, CRUSH_WALL_DOOR_INDEX_X2)
    }

    pub(crate) fn landing_class(&self) -> u8 {
        byte(self.ram, DUNG_TRANSITION_LANDING_CLASS)
    }

    pub(crate) fn landing_class_is_pit(&self) -> bool {
        matches!(self.landing_class(), 2 | 4)
    }

    pub(crate) fn push_block_direction_index(&self) -> usize {
        usize::from((byte(self.ram, PUSH_BLOCK_DIRECTION_DUNGEON) >> 1) & 3)
    }

    pub(crate) fn minigame_credits(&self) -> u8 {
        byte(self.ram, MINIGAME_CREDITS)
    }

    pub(crate) fn water_transition_counter(&self) -> u8 {
        byte(self.ram, TURN_ON_OFF_WATER_CTR)
    }

    pub(crate) fn water_hdma_y_radius(&self) -> u16 {
        word(self.ram, WATER_HDMA_WINDOW_Y_RADIUS)
    }

    pub(crate) fn water_hdma_x_radius(&self) -> u16 {
        word(self.ram, WATER_HDMA_WINDOW_X_RADIUS)
    }

    pub(crate) fn water_hdma_y_target(&self) -> u16 {
        word(self.ram, WATER_HDMA_WINDOW_Y_TARGET)
    }

    pub(crate) fn water_hdma_y_radius_alt(&self) -> u16 {
        word(self.ram, WATER_HDMA_WINDOW_Y_RADIUS_ALT)
    }

    pub(crate) fn current_staircase_plane(&self) -> u8 {
        byte(self.ram, CUR_STAIRCASE_PLANE)
    }

    pub(crate) fn staircase_lower_level_status(&self) -> u8 {
        byte(self.ram, STAIRCASE_LOWER_LEVEL_STATUS)
    }

    pub(crate) fn line_pointer_row0(&self, index: usize) -> u16 {
        word(self.ram, DUNG_LINE_PTRS_ROW0 + index * 2)
    }

    pub(crate) fn first_line_pointer_row0(&self) -> u16 {
        word(self.ram, DUNG_LINE_PTRS_ROW0)
    }

    pub(crate) fn num_chests_x2(&self) -> u16 {
        word(self.ram, DUNG_NUM_CHESTS_X2)
    }

    pub(crate) fn num_big_key_locks_x2(&self) -> u16 {
        word(self.ram, DUNG_NUM_BIGKEY_LOCKS_X2)
    }

    pub(crate) fn chest_reveal_cursor_x2(&self) -> u16 {
        word(self.ram, OVERWORLD_MAP_STATE)
    }

    pub(crate) fn chest_reveal_cursor_reached_end(&self, cursor_x2: u16) -> bool {
        cursor_x2 == self.num_chests_x2()
    }

    pub(crate) fn replacement_tile_destination_x2(&self) -> u16 {
        word(self.ram, DUNG_REPLACEMENT_TILE_DST_POS_X2)
    }

    pub(crate) fn replacement_tile_source_x2(&self) -> u16 {
        word(self.ram, DUNG_REPLACEMENT_TILE_SRC_POS_X2)
    }

    pub(crate) fn replacement_tile_source_pos(&self) -> u16 {
        self.replacement_tile_source_x2() >> 1
    }

    pub(crate) fn chest_location_for_cursor(&self, cursor_x2: u16) -> u16 {
        self.chest_location(usize::from(cursor_x2 >> 1))
    }

    pub(crate) fn adjacent_door_flags(&self) -> u16 {
        word(self.ram, ADJACENT_DOORS_FLAGS)
    }

    pub(crate) fn exit_door_count_x2(&self) -> u16 {
        word(self.ram, DUNG_EXIT_DOOR_COUNT)
    }

    pub(crate) fn exit_door_count(&self) -> usize {
        usize::from(self.exit_door_count_x2() >> 1)
    }

    pub(crate) fn exit_door_address(&self, index: usize) -> u16 {
        word(self.ram, DUNG_EXIT_DOOR_ADDRESSES + index * 2)
    }

    pub(crate) fn has_exit_door_address(&self, address: u16) -> bool {
        (0..4).any(|index| self.exit_door_address(index) == address)
    }

    pub(crate) fn invisible_door_marker(&self) -> u16 {
        word(self.ram, INVISIBLE_DOOR_DIR_AND_INDEX_X2)
    }

    pub(crate) fn pots_revealed_in_room(&self, room: usize) -> u16 {
        word(self.ram, POTS_REVEALED_IN_ROOM_DUNGEON_LOCAL + room * 2)
    }

    pub(crate) fn toggle_floor_count_x2(&self) -> u16 {
        word(self.ram, DUNG_NUM_TOGGLE_FLOOR)
    }

    pub(crate) fn toggle_palace_count_x2(&self) -> u16 {
        word(self.ram, DUNG_NUM_TOGGLE_PALACE)
    }

    pub(crate) fn toggle_floor_pos(&self, index: usize) -> u16 {
        word(self.ram, DUNG_TOGGLE_FLOOR_POS + index * 2)
    }

    pub(crate) fn toggle_palace_pos(&self, index: usize) -> u16 {
        word(self.ram, DUNG_TOGGLE_PALACE_POS + index * 2)
    }

    pub(crate) fn active_room_load_ptr(&self) -> u16 {
        word(self.ram, DUNG_LOAD_PTR)
    }

    pub(crate) fn active_room_load_ptr_bank(&self) -> u8 {
        byte(self.ram, DUNG_LOAD_PTR_BANK)
    }

    pub(crate) fn width_road_address(&self) -> u16 {
        word(self.ram, DUNG_WIDTH_ROAD_ADDRESS)
    }

    pub(crate) fn adjacent_door(&self, index: usize) -> u16 {
        word(self.ram, ADJACENT_DOORS + index * 2)
    }

    pub(crate) fn torch_data_word(&self, offset: usize) -> u16 {
        word(self.ram, DUNG_TORCH_DATA + offset)
    }

    pub(crate) fn torch_index(&self) -> u16 {
        word(self.ram, DUNG_INDEX_OF_TORCHES)
    }

    pub(crate) fn star_switch_count_x2(&self) -> u16 {
        word(self.ram, DUNG_NUM_STAR_SHAPED_SWITCHES_LOCAL)
    }

    pub(crate) fn star_switch_tilemap_pos(&self, offset_x2: usize) -> u16 {
        word(
            self.ram,
            STAR_SHAPED_SWITCHES_TILE_LOCAL + (offset_x2 >> 1) * 2,
        )
    }

    pub(crate) fn floor_1_filler_tile_source(&self) -> usize {
        usize::from(word(self.ram, FLOOR_1_FILLER_TILES))
    }

    pub(crate) fn floor_2_filler_tile_source(&self) -> usize {
        usize::from(word(self.ram, FLOOR_2_FILLER_TILES))
    }

    pub(crate) fn object_pos_in_objdata(&self, index: usize) -> u16 {
        word(self.ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2)
    }

    pub(crate) fn room_tilemap_word(&self, base: usize, dsto: u16) -> u16 {
        word(self.ram, base + dsto as usize * 2)
    }

    pub(crate) fn room_tilemap_word_by_byte_offset(&self, base: usize, byte_offset: usize) -> u16 {
        word(self.ram, base + byte_offset)
    }

    pub(crate) fn bg1_tilemap_base(&self) -> usize {
        ROOM_BG1_TILEMAP_BASE_LOCAL
    }

    pub(crate) fn bg2_tilemap_base(&self) -> usize {
        ROOM_BG2_TILEMAP_BASE_LOCAL
    }

    pub(crate) fn ram_asset_word(&self, offset: usize, index: usize) -> u16 {
        word(self.ram, offset + index * 2)
    }

    pub(crate) fn has_opened_door_mask(&self, mask: u16) -> bool {
        self.opened_doors_including_adjacent() & mask != 0
    }

    pub(crate) fn door_animation_step(&self) -> u16 {
        word(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON)
    }

    pub(crate) fn door_animation_step_low(&self) -> u8 {
        byte(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON)
    }

    pub(crate) fn staircase_index(&self) -> u8 {
        byte(self.ram, WHICH_STAIRCASE_INDEX)
    }

    pub(crate) fn staircase_index_slot(&self) -> usize {
        usize::from(self.staircase_index() & 3)
    }

    pub(crate) fn staircase_index_has_vertical_bit(&self) -> bool {
        self.staircase_index() & 4 != 0
    }

    pub(crate) fn staircase_move_counter(&self) -> u8 {
        byte(self.ram, STAIRCASE_MOVE_COUNTER)
    }

    pub(crate) fn stair_list_count(&self, list: DungeonStairList) -> u16 {
        word(self.ram, list.counter())
    }

    pub(crate) fn stair_list_tilemap_pos(&self, list: DungeonStairList, offset_x2: u16) -> u16 {
        let index = usize::from(offset_x2 >> 1);
        word(self.ram, list.tilemap_table() + index * 2)
    }

    pub(crate) fn inter_staircase_pos(&self, index: usize) -> u16 {
        word(self.ram, DUNG_INTER_STAIRCASES + index * 2)
    }

    pub(crate) fn bg2_attr_address(&self, offset: usize) -> usize {
        DUNGEON_BG2_ATTR_TABLE + offset
    }

    pub(crate) fn bg2_attr_pair(&self, offset: usize) -> Option<(u8, u8)> {
        let address = self.bg2_attr_address(offset);
        Some((*self.ram.get(address)?, *self.ram.get(address + 1)?))
    }

    pub(crate) fn bg2_attr_slice(&self, start: usize, len: usize) -> &[u8] {
        &self.ram[DUNGEON_BG2_ATTR_TABLE + start..DUNGEON_BG2_ATTR_TABLE + start + len]
    }

    pub(crate) fn door_type_and_slot(&self, door: usize) -> u8 {
        byte(self.ram, DOOR_TYPE_AND_SLOT + door * 2)
    }

    pub(crate) fn door_type_word(&self, door: usize) -> u16 {
        word(self.ram, DOOR_TYPE_AND_SLOT + door * 2)
    }

    pub(crate) fn door_direction(&self, door: usize) -> u8 {
        byte(self.ram, DUNGEON_DOOR_DIRECTION + door * 2)
    }

    pub(crate) fn door_direction_word(&self, door: usize) -> u16 {
        word(self.ram, DUNGEON_DOOR_DIRECTION + door * 2)
    }

    pub(crate) fn changeable_object_index(&self, index: usize) -> u8 {
        byte(self.ram, CHANGEABLE_DUNGEON_OBJECT_INDEX + index)
    }

    pub(crate) fn replacement_tile_state(&self, index: usize) -> u16 {
        word(self.ram, DUNGEON_REPLACEMENT_TILE_STATE + index * 2)
    }

    pub(crate) fn savegame_state_bits(&self) -> u16 {
        word(self.ram, DUNG_SAVEGAME_STATE_BITS)
    }

    pub(crate) fn has_savegame_state_bits(&self, mask: u16) -> bool {
        self.savegame_state_bits() & mask != 0
    }

    pub(crate) fn room_index2(&self) -> u8 {
        byte(self.ram, DUNGEON_ROOM_INDEX2)
    }

    pub(crate) fn room_index2_word(&self) -> u16 {
        word(self.ram, DUNGEON_ROOM_INDEX2)
    }

    pub(crate) fn previous_room_index(&self) -> usize {
        usize::from(word(self.ram, DUNGEON_ROOM_INDEX_PREV))
    }

    pub(crate) fn previous_room_index_word(&self) -> u16 {
        word(self.ram, DUNGEON_ROOM_INDEX_PREV)
    }

    pub(crate) fn room_transitioning_flags(&self) -> u8 {
        byte(self.ram, ROOM_TRANSITIONING_FLAGS)
    }

    pub(crate) fn bg2_tile(&self, index: usize) -> u16 {
        word(self.ram, DUNG_BG2 + index * 2)
    }

    pub(crate) fn bg1_tile(&self, index: usize) -> u16 {
        word(self.ram, DUNG_BG1 + index * 2)
    }

    pub(crate) fn bg1_tile_by_byte_pos(&self, pos: u16) -> u16 {
        self.bg1_tile((pos >> 1) as usize)
    }

    pub(crate) fn bg2_tile_by_byte_pos(&self, pos: u16) -> u16 {
        self.bg2_tile((pos >> 1) as usize)
    }

    pub(crate) fn misc_object_index(&self) -> u16 {
        word(self.ram, DUNG_MISC_OBJS_INDEX)
    }

    pub(crate) fn misc_object_slot(&self) -> usize {
        (self.misc_object_index() >> 1) as usize
    }

    pub(crate) fn load_ptr_offset(&self) -> u16 {
        word(self.ram, DUNG_LOAD_PTR_OFFS)
    }

    pub(crate) fn object_tilemap_pos(&self, index: usize) -> u16 {
        word(self.ram, DUNG_OBJECT_TILEMAP_POS + index * 2)
    }

    pub(crate) fn door_tilemap_address(&self, door: usize) -> u16 {
        word(self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2)
    }

    pub(crate) fn current_door_index(&self) -> u16 {
        word(self.ram, DUNG_CUR_DOOR_IDX)
    }

    pub(crate) fn current_door_slot(&self) -> usize {
        (self.current_door_index() >> 1) as usize
    }

    pub(crate) fn current_door_pos(&self) -> u16 {
        word(self.ram, DUNG_CUR_DOOR_POS_DUNGEON)
    }

    pub(crate) fn door_open_counter(&self) -> u16 {
        word(self.ram, DOOR_OPEN_CLOSED_COUNTER)
    }

    pub(crate) fn door_open_counter_low(&self) -> u8 {
        byte(self.ram, DOOR_OPEN_CLOSED_COUNTER)
    }

    pub(crate) fn trap_trigger_latch(&self) -> u8 {
        byte(self.ram, DUNGEON_TRAP_TRIGGER_LATCH)
    }

    pub(crate) fn room_history_entry(&self, index: usize) -> u16 {
        word(self.ram, DUNGEON_ROOM_HISTORY + index * 2)
    }

    pub(crate) fn floor_move_flags(&self) -> u16 {
        word(self.ram, DUNG_FLOOR_MOVE_FLAGS)
    }

    pub(crate) fn has_bomb_trap_activation(&self) -> bool {
        byte(self.ram, ACTIVATE_BOMB_TRAP_OVERLORD) != 0
    }

    pub(crate) fn kind_of_in_room_staircase(&self) -> u8 {
        byte(self.ram, KIND_OF_IN_ROOM_STAIRCASE)
    }

    pub(crate) fn blast_wall_message_state(&self) -> u8 {
        byte(self.ram, MESSAGING_BUF_DUNGEON)
    }

    pub(crate) fn staircase_tilemap_pos_x2(&self) -> u16 {
        word(self.ram, STAIRCASE_TILEMAP_POS_X2)
    }

    pub(crate) fn staircase_countdown(&self) -> u8 {
        byte(self.ram, COUNTDOWN_TIMER_FOR_STAIRCASES)
    }

    pub(crate) fn should_run_room_tags(&self) -> bool {
        byte(self.ram, FLAG_SKIP_CALL_TAG_ROUTINES) == 0
    }

    pub(crate) fn loading_bg_offset_h(&self) -> u16 {
        word(self.ram, DUNG_LOADE_BGOFFS_H_COPY)
    }

    pub(crate) fn loading_bg_offset_v(&self) -> u16 {
        word(self.ram, DUNG_LOADE_BGOFFS_V_COPY)
    }

    pub(crate) fn movable_block_room(&self, offset: usize) -> u16 {
        word(self.ram, MOVABLE_BLOCK_DATAS + offset)
    }

    pub(crate) fn movable_block_tilemap(&self, offset: usize) -> u16 {
        word(self.ram, MOVABLE_BLOCK_DATAS + offset + 2)
    }

    pub(crate) fn big_rock_starting_address(&self) -> u16 {
        word(self.ram, BIG_ROCK_STARTING_ADDRESS)
    }

    pub(crate) fn chest_location(&self, index: usize) -> u16 {
        word(self.ram, DUNG_CHEST_LOCATIONS + index * 2)
    }

    pub(crate) fn chest_location_for_offset_x2(&self, offset_x2: usize) -> u16 {
        self.chest_location(offset_x2 >> 1)
    }

    pub(crate) fn replacement_tilemap_quad(&self, index: usize) -> [u16; 4] {
        [
            word(self.ram, REPLACEMENT_TILEMAP_UL + index * 2),
            word(self.ram, REPLACEMENT_TILEMAP_LL + index * 2),
            word(self.ram, REPLACEMENT_TILEMAP_UR + index * 2),
            word(self.ram, REPLACEMENT_TILEMAP_LR + index * 2),
        ]
    }

    pub(crate) fn moving_floor_check_flags(&self) -> u16 {
        word(self.ram, MOVING_FLOOR_BG_CHECK_FLAGS)
    }
}

pub(crate) struct DungeonStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DungeonStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_header_tag(&mut self, index: usize, value: u8) {
        self.ram[DUNGEON_HEADER_TAG + index] = value;
    }

    pub(crate) fn clear_header_tag(&mut self, index: usize) {
        self.set_header_tag(index, 0);
    }

    pub(crate) fn clear_header_tags(&mut self, count: usize) {
        self.ram[DUNGEON_HEADER_TAG..DUNGEON_HEADER_TAG + count].fill(0);
    }

    pub(crate) fn set_bg2_attr(&mut self, offset: usize, value: u8) {
        self.ram[DUNGEON_BG2_ATTR_TABLE + offset] = value;
    }

    pub(crate) fn set_bg2_attr_word(&mut self, offset: usize, value: u16) {
        write_le_u16(self.ram, DUNGEON_BG2_ATTR_TABLE + offset, value);
    }

    pub(crate) fn set_bg1_attr_word(&mut self, offset: usize, value: u16) {
        write_le_u16(self.ram, DUNGEON_BG1_ATTR_TABLE + offset, value);
    }

    pub(crate) fn xor_bg2_attr(&mut self, offset: usize, value: u8) {
        self.ram[DUNGEON_BG2_ATTR_TABLE + offset] ^= value;
    }

    pub(crate) fn xor_bg1_attr(&mut self, offset: usize, value: u8) {
        self.ram[DUNGEON_BG1_ATTR_TABLE + offset] ^= value;
    }

    pub(crate) fn set_floor_y_velocity_high(&mut self, value: u8) {
        self.ram[DUNGEON_FLOOR_Y_VELOCITY + 1] = value;
    }

    pub(crate) fn set_floor_y_velocity(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGEON_FLOOR_Y_VELOCITY, value);
    }

    pub(crate) fn set_floor_x_velocity(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGEON_FLOOR_X_VELOCITY, value);
    }

    pub(crate) fn clear_floor_velocity(&mut self) {
        write_le_u16(self.ram, DUNGEON_FLOOR_X_VELOCITY, 0);
        write_le_u16(self.ram, DUNGEON_FLOOR_Y_VELOCITY, 0);
    }

    pub(crate) fn set_header_collision_2_mirror_high(&mut self, value: u8) {
        self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR + 1] = value;
    }

    pub(crate) fn set_header_collision(&mut self, value: u8) {
        self.ram[DUNG_HDR_COLLISION] = value;
    }

    pub(crate) fn set_header_collision_2(&mut self, value: u8) {
        self.ram[DUNG_HDR_COLLISION_2] = value;
    }

    pub(crate) fn clear_header_collision_2(&mut self) {
        self.ram[DUNG_HDR_COLLISION_2] = 0;
    }

    pub(crate) fn set_header_collision_2_mirror(&mut self, value: u8) {
        self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR] = value;
    }

    pub(crate) fn increment_header_collision_2_mirror(&mut self) -> u8 {
        self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR] =
            self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR].wrapping_add(1);
        self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR]
    }

    pub(crate) fn copy_header_collision_2_to_mirror(&mut self) {
        self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR] = self.ram[DUNG_HDR_COLLISION_2];
    }

    pub(crate) fn set_game_over_check_flag(&mut self, value: u16) {
        write_le_u16(self.ram, GAME_OVER_CHECK_FLAG, value);
    }

    pub(crate) fn clear_game_over_check_flag(&mut self) {
        write_le_u16(self.ram, GAME_OVER_CHECK_FLAG, 0);
    }

    pub(crate) fn clear_restart_check_flag(&mut self) {
        self.ram[RESTART_CHECK_FLAG] = 0;
    }

    pub(crate) fn set_bg2_properties(&mut self, value: u8) {
        self.ram[DUNG_HDR_BG2_PROPERTIES] = value;
    }

    pub(crate) fn clear_bg2_properties(&mut self) {
        self.ram[DUNG_HDR_BG2_PROPERTIES] = 0;
    }

    pub(crate) fn set_current_floor(&mut self, value: u8) {
        self.ram[DUNG_CUR_FLOOR] = value;
    }

    pub(crate) fn decrement_current_floor(&mut self) -> u8 {
        self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_sub(1);
        self.ram[DUNG_CUR_FLOOR]
    }

    pub(crate) fn increment_current_floor(&mut self) -> u8 {
        self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_add(1);
        self.ram[DUNG_CUR_FLOOR]
    }

    pub(crate) fn cache_current_floor(&mut self) {
        self.ram[DUNG_CUR_FLOOR_CACHED] = self.ram[DUNG_CUR_FLOOR];
    }

    pub(crate) fn restore_cached_floor(&mut self) {
        self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR_CACHED];
    }

    pub(crate) fn clear_lit_torches(&mut self) {
        self.ram[DUNG_NUM_LIT_TORCHES] = 0;
    }

    pub(crate) fn set_lit_torches(&mut self, value: u8) {
        self.ram[DUNG_NUM_LIT_TORCHES] = value;
    }

    pub(crate) fn increment_lit_torches(&mut self) -> u8 {
        self.ram[DUNG_NUM_LIT_TORCHES] = self.ram[DUNG_NUM_LIT_TORCHES].wrapping_add(1);
        self.ram[DUNG_NUM_LIT_TORCHES]
    }

    pub(crate) fn decrement_lit_torches(&mut self) -> u8 {
        self.ram[DUNG_NUM_LIT_TORCHES] = self.ram[DUNG_NUM_LIT_TORCHES].wrapping_sub(1);
        self.ram[DUNG_NUM_LIT_TORCHES]
    }

    pub(crate) fn set_lights_out_request(&mut self, value: u8) {
        self.ram[DUNG_WANT_LIGHTS_OUT] = value;
    }

    pub(crate) fn clear_lights_out_request(&mut self) {
        self.ram[DUNG_WANT_LIGHTS_OUT] = 0;
    }

    pub(crate) fn set_lights_out_request_copy(&mut self, value: u8) {
        self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = value;
    }

    pub(crate) fn copy_lights_out_request(&mut self) {
        self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = self.ram[DUNG_WANT_LIGHTS_OUT];
    }

    pub(crate) fn clear_lights_out_requests(&mut self) {
        self.ram[DUNG_WANT_LIGHTS_OUT] = 0;
        self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = 0;
    }

    pub(crate) fn clear_quadrant_upload_index(&mut self) {
        self.ram[DUNG_CUR_QUADRANT_UPLOAD] = 0;
    }

    pub(crate) fn advance_quadrant_upload_index_by(&mut self, value: u8) -> u8 {
        self.ram[DUNG_CUR_QUADRANT_UPLOAD] = self.ram[DUNG_CUR_QUADRANT_UPLOAD].wrapping_add(value);
        self.ram[DUNG_CUR_QUADRANT_UPLOAD]
    }

    pub(crate) fn set_trapdoors_down(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_FLAG_TRAPDOORS_DOWN, value);
    }

    pub(crate) fn clear_trapdoors_down(&mut self) {
        write_le_u16(self.ram, DUNG_FLAG_TRAPDOORS_DOWN, 0);
    }

    pub(crate) fn set_trapdoors_down_low(&mut self, value: u8) {
        self.ram[DUNG_FLAG_TRAPDOORS_DOWN] = value;
    }

    pub(crate) fn increment_trapdoors_down_low(&mut self) -> u8 {
        self.ram[DUNG_FLAG_TRAPDOORS_DOWN] = self.ram[DUNG_FLAG_TRAPDOORS_DOWN].wrapping_add(1);
        self.ram[DUNG_FLAG_TRAPDOORS_DOWN]
    }

    pub(crate) fn clear_somaria_block_switch_counter(&mut self) {
        self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH] = 0;
    }

    pub(crate) fn increment_somaria_block_bg_check_flag(&mut self) {
        self.ram[SOMARIA_BLOCK_BG_CHECK_FLAG] =
            self.ram[SOMARIA_BLOCK_BG_CHECK_FLAG].wrapping_add(1);
    }

    pub(crate) fn increment_somaria_block_switch_counter(&mut self) {
        self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH] =
            self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH].wrapping_add(1);
    }

    pub(crate) fn set_big_rock_starting_address(&mut self, value: u16) {
        write_le_u16(self.ram, BIG_ROCK_STARTING_ADDRESS, value);
    }

    pub(crate) fn clear_water_puzzle_state_changed(&mut self) {
        self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = 0;
    }

    pub(crate) fn set_water_puzzle_state_changed(&mut self, value: u8) {
        self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = value;
    }

    pub(crate) fn increment_water_puzzle_state_changed(&mut self) -> u8 {
        self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] =
            self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE].wrapping_add(1);
        self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE]
    }

    pub(crate) fn set_draw_width_indicator(&mut self, value: u8) {
        self.ram[DUNG_DRAW_WIDTH_INDICATOR] = value;
    }

    pub(crate) fn set_draw_width_indicator_word(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_DRAW_WIDTH_INDICATOR, value);
    }

    pub(crate) fn set_draw_height_indicator(&mut self, value: u8) {
        self.ram[DUNG_DRAW_HEIGHT_INDICATOR] = value;
    }

    pub(crate) fn set_draw_height_indicator_word(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_DRAW_HEIGHT_INDICATOR, value);
    }

    pub(crate) fn clear_draw_dimensions(&mut self) {
        write_le_u16(self.ram, DUNG_DRAW_WIDTH_INDICATOR, 0);
        write_le_u16(self.ram, DUNG_DRAW_HEIGHT_INDICATOR, 0);
    }

    pub(crate) fn set_draw_dimensions(&mut self, width: u8, height: u8) {
        self.ram[DUNG_DRAW_WIDTH_INDICATOR] = width;
        self.ram[DUNG_DRAW_HEIGHT_INDICATOR] = height;
    }

    pub(crate) fn set_draw_dimensions_words(&mut self, width: u16, height: u16) {
        write_le_u16(self.ram, DUNG_DRAW_WIDTH_INDICATOR, width);
        write_le_u16(self.ram, DUNG_DRAW_HEIGHT_INDICATOR, height);
    }

    pub(crate) fn set_opened_doors_including_adjacent(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, value);
    }

    pub(crate) fn set_floor_x_offset(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_FLOOR_X_OFFS, value);
    }

    pub(crate) fn set_floor_y_offset(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_FLOOR_Y_OFFS, value);
    }

    pub(crate) fn set_floor_y_offset_low(&mut self, value: u8) {
        self.ram[DUNG_FLOOR_Y_OFFS] = value;
    }

    pub(crate) fn set_floor_offsets(&mut self, x: u16, y: u16) {
        self.set_floor_x_offset(x);
        self.set_floor_y_offset(y);
    }

    pub(crate) fn add_floor_x_offset(&mut self, delta: u16) -> u16 {
        let value = word(self.ram, DUNG_FLOOR_X_OFFS).wrapping_add(delta);
        self.set_floor_x_offset(value);
        value
    }

    pub(crate) fn sub_floor_x_offset(&mut self, delta: u16) -> u16 {
        let value = word(self.ram, DUNG_FLOOR_X_OFFS).wrapping_sub(delta);
        self.set_floor_x_offset(value);
        value
    }

    pub(crate) fn add_floor_y_offset(&mut self, delta: u16) -> u16 {
        let value = word(self.ram, DUNG_FLOOR_Y_OFFS).wrapping_add(delta);
        self.set_floor_y_offset(value);
        value
    }

    pub(crate) fn sub_floor_y_offset(&mut self, delta: u16) -> u16 {
        let value = word(self.ram, DUNG_FLOOR_Y_OFFS).wrapping_sub(delta);
        self.set_floor_y_offset(value);
        value
    }

    pub(crate) fn clear_floor_offsets(&mut self) {
        self.set_floor_offsets(0, 0);
    }

    pub(crate) fn clear_floor_move_flags(&mut self) {
        write_le_u16(self.ram, DUNG_FLOOR_MOVE_FLAGS, 0);
    }

    pub(crate) fn fill_moving_wall_replacement_buffer(&mut self, value: u16) {
        for i in 0..64 {
            write_le_u16(self.ram, MOVING_WALL_REPLACEMENT_BUFFER + i * 2, value);
        }
    }

    pub(crate) fn set_moving_wall_write_point(&mut self, value: u16) {
        write_le_u16(self.ram, MOVING_WALL_WRITE_POINT, value);
    }

    pub(crate) fn set_moving_wall_dot_pointer(&mut self, value: u8) {
        self.ram[MOVING_WALL_DOT_POINTER] = value;
    }

    pub(crate) fn clear_dungeon_music_type_flag(&mut self) {
        self.ram[FLAG_WHICH_MUSIC_TYPE_DUNGEON] = 0;
    }

    pub(crate) fn set_dungeon_music_type_flag(&mut self, value: u8) {
        self.ram[FLAG_WHICH_MUSIC_TYPE_DUNGEON] = value;
    }

    pub(crate) fn set_fixed_color_plusminus(&mut self, value: u8) {
        self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = value;
    }

    pub(crate) fn set_overlay_to_load(&mut self, value: u8) {
        self.ram[DUNG_OVERLAY_TO_LOAD] = value;
    }

    pub(crate) fn set_overlay_to_load_if_empty(&mut self, value: u8) {
        if self.ram[DUNG_OVERLAY_TO_LOAD] == 0 {
            self.set_overlay_to_load(value);
        }
    }

    pub(crate) fn toggle_moving_wall_torch_blink_phase(&mut self) {
        self.ram[MOVING_WALL_TORCH_BLINK_PHASE] ^= 1;
    }

    pub(crate) fn request_moving_wall_torch_update(&mut self) {
        self.ram[MOVING_WALL_TORCH_UPDATE_FLAG] = 0x80;
    }

    pub(crate) fn clear_moving_wall_torch_blink_phase(&mut self) {
        self.ram[MOVING_WALL_TORCH_BLINK_PHASE] = 0;
    }

    pub(crate) fn set_water_transition_counter(&mut self, value: u8) {
        self.ram[TURN_ON_OFF_WATER_CTR] = value;
    }

    pub(crate) fn increment_water_transition_counter(&mut self) -> u8 {
        self.ram[TURN_ON_OFF_WATER_CTR] = self.ram[TURN_ON_OFF_WATER_CTR].wrapping_add(1);
        self.ram[TURN_ON_OFF_WATER_CTR]
    }

    pub(crate) fn decrement_water_transition_counter(&mut self) -> u8 {
        self.ram[TURN_ON_OFF_WATER_CTR] = self.ram[TURN_ON_OFF_WATER_CTR].wrapping_sub(1);
        self.ram[TURN_ON_OFF_WATER_CTR]
    }

    pub(crate) fn set_water_hdma_y_radius(&mut self, value: u16) {
        write_le_u16(self.ram, WATER_HDMA_WINDOW_Y_RADIUS, value);
    }

    pub(crate) fn set_water_hdma_x_radius(&mut self, value: u16) {
        write_le_u16(self.ram, WATER_HDMA_WINDOW_X_RADIUS, value);
    }

    pub(crate) fn set_water_hdma_y_target(&mut self, value: u16) {
        write_le_u16(self.ram, WATER_HDMA_WINDOW_Y_TARGET, value);
    }

    pub(crate) fn set_water_hdma_y_radius_alt(&mut self, value: u16) {
        write_le_u16(self.ram, WATER_HDMA_WINDOW_Y_RADIUS_ALT, value);
    }

    pub(crate) fn set_water_window_position(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, WATER_HDMA_WINDOW_X, x);
        write_le_u16(self.ram, WATER_HDMA_WINDOW_Y, y);
    }

    pub(crate) fn set_loading_bg_offsets(&mut self, horizontal: u16, vertical: u16) {
        write_le_u16(self.ram, DUNG_LOADE_BGOFFS_H_COPY, horizontal);
        write_le_u16(self.ram, DUNG_LOADE_BGOFFS_V_COPY, vertical);
    }

    pub(crate) fn promote_water_ladders_to_saved_stair_counters(&mut self) {
        let north_stairs = word(self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER);
        let active_ladders = word(self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS);
        let south_stairs = word(self.ram, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER);

        write_le_u16(self.ram, DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS, north_stairs);
        write_le_u16(self.ram, WATER_SIDE_STEP_SWITCH, active_ladders);
        write_le_u16(self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS, 0);
        write_le_u16(self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER, 0);
        write_le_u16(self.ram, DUNG_NUM_STAIRS_WET, south_stairs);
        write_le_u16(self.ram, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER, 0);
    }

    pub(crate) fn append_star_switch_tile(&mut self, tilemap_pos: u16) -> usize {
        let index = usize::from(word(self.ram, DUNG_NUM_STAR_SHAPED_SWITCHES_LOCAL)) >> 1;
        let next = word(self.ram, DUNG_NUM_STAR_SHAPED_SWITCHES_LOCAL).wrapping_add(2);
        write_le_u16(self.ram, DUNG_NUM_STAR_SHAPED_SWITCHES_LOCAL, next);
        write_le_u16(
            self.ram,
            STAR_SHAPED_SWITCHES_TILE_LOCAL + index * 2,
            tilemap_pos,
        );
        index
    }

    pub(crate) fn stair_list_count(&self, list: DungeonStairList) -> u16 {
        word(self.ram, list.counter())
    }

    pub(crate) fn set_stair_list_count(&mut self, list: DungeonStairList, value: u16) {
        write_le_u16(self.ram, list.counter(), value);
    }

    pub(crate) fn sync_stair_list_counts(&mut self, lists: &[DungeonStairList], value: u16) {
        for &list in lists {
            self.set_stair_list_count(list, value);
        }
    }

    pub(crate) fn append_interroom_staircase(
        &mut self,
        list: DungeonStairList,
        tilemap_pos: u16,
    ) -> u16 {
        let index = usize::from(self.stair_list_count(list)) >> 1;
        write_le_u16(
            self.ram,
            DUNG_INTER_STAIRCASE_TABLE_LOCAL + index * 2,
            tilemap_pos,
        );
        self.stair_list_count(list).wrapping_add(2)
    }

    pub(crate) fn append_bg1_stair_table_position(
        &mut self,
        list: DungeonStairList,
        tilemap_pos: u16,
    ) -> u16 {
        let index = usize::from(self.stair_list_count(list)) >> 1;
        write_le_u16(self.ram, DUNG_STAIRS_TABLE_1_LOCAL + index * 2, tilemap_pos);
        let next = self.stair_list_count(list).wrapping_add(2);
        self.set_stair_list_count(list, next);
        next
    }

    pub(crate) fn append_stair_table_position(
        &mut self,
        list: DungeonStairList,
        tilemap_pos: u16,
    ) -> u16 {
        let index = usize::from(self.stair_list_count(list)) >> 1;
        write_le_u16(self.ram, list.tilemap_table() + index * 2, tilemap_pos);
        let next = self.stair_list_count(list).wrapping_add(2);
        self.set_stair_list_count(list, next);
        next
    }

    pub(crate) fn clear_room_parser_words(&mut self, offsets: &[usize]) {
        for &offset in offsets {
            write_le_u16(self.ram, offset, 0);
        }
    }

    pub(crate) fn clear_invisible_door_marker(&mut self) {
        write_le_u16(self.ram, INVISIBLE_DOOR_DIR_AND_INDEX_X2, 0xffff);
    }

    pub(crate) fn set_invisible_door_marker(&mut self, slot: usize, direction: u16) {
        write_le_u16(
            self.ram,
            INVISIBLE_DOOR_DIR_AND_INDEX_X2,
            (((slot as u16) << 8) | direction) * 2,
        );
    }

    pub(crate) fn mark_pot_revealed_in_room(&mut self, room: usize, mask: u16) -> u16 {
        let revealed = read_le_u16(self.ram, POTS_REVEALED_IN_ROOM_DUNGEON_LOCAL + room * 2) | mask;
        write_le_u16(
            self.ram,
            POTS_REVEALED_IN_ROOM_DUNGEON_LOCAL + room * 2,
            revealed,
        );
        revealed
    }

    pub(crate) fn set_room_door_info_word(&mut self, dst: usize, index: usize, value: u16) {
        write_le_u16(self.ram, dst + index * 2, value);
    }

    pub(crate) fn set_torch_index_range_start(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_INDEX_OF_TORCHES_START, value);
    }

    pub(crate) fn set_torch_index(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_INDEX_OF_TORCHES, value);
    }

    pub(crate) fn set_staircase_tilemap_pos_x2(&mut self, value: u16) {
        write_le_u16(self.ram, STAIRCASE_TILEMAP_POS_X2, value);
    }

    pub(crate) fn set_bg2_properties_backup(&mut self, value: u8) {
        self.ram[DUNG_HDR_BG2_PROPERTIES_BACKUP] = value;
    }

    pub(crate) fn copy_header_travel_destinations_from(&mut self, header: &[u8]) {
        self.ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS..DUNGEON_HEADER_TRAVEL_DESTINATIONS + 5]
            .copy_from_slice(&header[9..14]);
    }

    pub(crate) fn clear_overlay_to_load(&mut self) {
        self.ram[DUNG_OVERLAY_TO_LOAD] = 0;
    }

    pub(crate) fn set_room_index_x3(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_INDEX_X3, value);
    }

    pub(crate) fn set_current_staircase_plane(&mut self, value: u8) {
        self.ram[CUR_STAIRCASE_PLANE] = value;
    }

    pub(crate) fn set_staircase_lower_level_status(&mut self, value: u8) {
        self.ram[STAIRCASE_LOWER_LEVEL_STATUS] = value;
    }

    pub(crate) fn set_line_pointer_row0(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNG_LINE_PTRS_ROW0 + index * 2, value);
    }

    pub(crate) fn copy_line_pointer_bytes(&mut self, offsets: &[u8]) {
        self.ram[DUNG_LINE_PTRS_ROW0..DUNG_LINE_PTRS_ROW0 + offsets.len()].copy_from_slice(offsets);
    }

    pub(crate) fn clear_exit_door_count_and_flags(&mut self) {
        self.ram[DUNG_EXIT_DOOR_COUNT..DUNG_EXIT_DOOR_COUNT + 10].fill(0);
    }

    pub(crate) fn clear_movable_block_was_pushed(&mut self) {
        self.ram[DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED] = 0;
    }

    pub(crate) fn toggle_movable_block_was_pushed(&mut self) {
        self.ram[DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED] ^= 1;
    }

    pub(crate) fn set_block_trap_related_tile(&mut self, value: u16) {
        write_le_u16(self.ram, BLOCK_TRAP_CHECK_FLAG, value);
    }

    pub(crate) fn set_selected_key_door_x2(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_WHICH_KEY_X2_DUNGEON, value);
    }

    pub(crate) fn set_selected_key_door(&mut self, door: usize) {
        self.set_selected_key_door_x2((door * 2) as u16);
    }

    pub(crate) fn clear_door_barrier_or_switch_flag(&mut self) {
        write_le_u16(self.ram, DUNG_DOOR_BARRIER_OR_SWITCH_FLAG, 0);
    }

    pub(crate) fn set_blast_wall_door_index_x2(&mut self, value: u16) {
        write_le_u16(self.ram, CRUSH_WALL_DOOR_INDEX_X2, value);
    }

    pub(crate) fn set_blast_wall_door_index(&mut self, door: usize) {
        self.set_blast_wall_door_index_x2((door * 2) as u16);
    }

    pub(crate) fn clear_blast_wall_door_index(&mut self) {
        self.set_blast_wall_door_index_x2(0);
    }

    pub(crate) fn set_landing_class(&mut self, value: u8) {
        self.ram[DUNG_TRANSITION_LANDING_CLASS] = value;
    }

    pub(crate) fn clear_landing_class(&mut self) {
        self.ram[DUNG_TRANSITION_LANDING_CLASS] = 0;
    }

    pub(crate) fn mark_door_switch_triggered(&mut self) {
        self.ram[DUNG_DOOR_SWITCH_TRIGGERED] = 1;
    }

    pub(crate) fn clear_door_switch_triggered(&mut self) {
        self.ram[DUNG_DOOR_SWITCH_TRIGGERED] = 0;
    }

    pub(crate) fn set_movable_block_record(&mut self, index: usize, room: u16, tilemap: u16) {
        write_le_u16(self.ram, MOVABLE_BLOCK_DATAS + index * 4, room);
        write_le_u16(self.ram, MOVABLE_BLOCK_DATAS + index * 4 + 2, tilemap);
    }

    pub(crate) fn set_num_chests_x2(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_NUM_CHESTS_X2, value);
    }

    pub(crate) fn set_num_big_key_locks_x2(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_NUM_BIGKEY_LOCKS_X2, value);
    }

    pub(crate) fn append_chest_location_and_sync_big_key_count(&mut self, value: u16) -> usize {
        let index = self.advance_chest_and_big_key_counts();
        self.set_chest_location(index, value);
        index
    }

    pub(crate) fn advance_chest_and_big_key_counts(&mut self) -> usize {
        let index = usize::from(word(self.ram, DUNG_NUM_CHESTS_X2)) >> 1;
        let next = ((index + 1) * 2) as u16;
        self.set_num_chests_x2(next);
        self.set_num_big_key_locks_x2(next);
        index
    }

    pub(crate) fn append_big_key_lock_location(&mut self, value: u16) -> usize {
        let index = self.advance_big_key_lock_count();
        self.set_chest_location(index, value);
        index
    }

    pub(crate) fn advance_big_key_lock_count(&mut self) -> usize {
        let index = usize::from(word(self.ram, DUNG_NUM_BIGKEY_LOCKS_X2)) >> 1;
        self.set_num_big_key_locks_x2(((index + 1) * 2) as u16);
        index
    }

    pub(crate) fn set_chest_location(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNG_CHEST_LOCATIONS + index * 2, value);
    }

    pub(crate) fn set_chest_location_for_offset_x2(&mut self, offset_x2: usize, value: u16) {
        self.set_chest_location(offset_x2 >> 1, value);
    }

    pub(crate) fn set_chest_reveal_cursor_x2(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_MAP_STATE, value);
    }

    pub(crate) fn clear_chest_reveal_cursor(&mut self) {
        self.set_chest_reveal_cursor_x2(0);
    }

    pub(crate) fn set_replacement_tile_destination_x2(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_REPLACEMENT_TILE_DST_POS_X2, value);
    }

    pub(crate) fn set_replacement_tile_source_x2(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_REPLACEMENT_TILE_SRC_POS_X2, value);
    }

    pub(crate) fn clear_replacement_tile_destination(&mut self) {
        self.set_replacement_tile_destination_x2(0);
    }

    pub(crate) fn clear_chest_location(&mut self, index: usize) {
        self.set_chest_location(index, 0);
    }

    pub(crate) fn set_opened_doors(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_DOOR_OPENED, value);
    }

    pub(crate) fn or_opened_doors(&mut self, mask: u16) -> u16 {
        let opened = word(self.ram, DUNG_DOOR_OPENED) | mask;
        self.set_opened_doors(opened);
        opened
    }

    pub(crate) fn mark_door_opened(&mut self, door: usize) -> u16 {
        self.or_opened_doors(0x8000u16 >> (door & 15))
    }

    pub(crate) fn append_exit_door_address(&mut self, address: u16) -> usize {
        let count = word(self.ram, DUNG_EXIT_DOOR_COUNT);
        let index = usize::from(count >> 1);
        if index < 16 {
            write_le_u16(self.ram, DUNG_EXIT_DOOR_ADDRESSES + index * 2, address);
        }
        write_le_u16(self.ram, DUNG_EXIT_DOOR_COUNT, count.wrapping_add(2));
        index
    }

    pub(crate) fn append_toggle_palace_pos(&mut self, pos: u16) -> usize {
        let count = word(self.ram, DUNG_NUM_TOGGLE_PALACE);
        let index = usize::from(count >> 1);
        if index < 8 {
            write_le_u16(self.ram, DUNG_TOGGLE_PALACE_POS + index * 2, pos);
        }
        write_le_u16(self.ram, DUNG_NUM_TOGGLE_PALACE, count.wrapping_add(2));
        index
    }

    pub(crate) fn append_toggle_floor_pos(&mut self, pos: u16) -> usize {
        let count = word(self.ram, DUNG_NUM_TOGGLE_FLOOR);
        let index = usize::from(count >> 1);
        if index < 8 {
            write_le_u16(self.ram, DUNG_TOGGLE_FLOOR_POS + index * 2, pos);
        }
        write_le_u16(self.ram, DUNG_NUM_TOGGLE_FLOOR, count.wrapping_add(2));
        index
    }

    pub(crate) fn set_active_room_load_ptr(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_LOAD_PTR, value);
    }

    pub(crate) fn set_active_room_load_ptr_bank(&mut self, value: u8) {
        self.ram[DUNG_LOAD_PTR_BANK] = value;
    }

    pub(crate) fn set_replacement_tilemap_quad(&mut self, index: usize, quad: [u16; 4]) {
        write_le_u16(self.ram, REPLACEMENT_TILEMAP_UL + index * 2, quad[0]);
        write_le_u16(self.ram, REPLACEMENT_TILEMAP_LL + index * 2, quad[1]);
        write_le_u16(self.ram, REPLACEMENT_TILEMAP_UR + index * 2, quad[2]);
        write_le_u16(self.ram, REPLACEMENT_TILEMAP_LR + index * 2, quad[3]);
    }

    pub(crate) fn mark_opened_door_mask(&mut self, mask: u16) -> u16 {
        let opened = read_le_u16(self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) | mask;
        write_le_u16(self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, opened);
        opened
    }

    pub(crate) fn clear_door_animation_step(&mut self) {
        write_le_u16(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
    }

    pub(crate) fn set_door_animation_step(&mut self, value: u16) {
        write_le_u16(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, value);
    }

    pub(crate) fn set_door_animation_step_low(&mut self, value: u8) {
        self.ram[DOOR_ANIMATION_STEP_INDICATOR_DUNGEON] = value;
    }

    pub(crate) fn set_staircase_countdown(&mut self, value: u8) {
        self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] = value;
    }

    pub(crate) fn decrement_staircase_countdown_clamped(&mut self) -> u8 {
        let value = self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES].wrapping_sub(1);
        self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] = if (value as i8).is_negative() {
            0
        } else {
            value
        };
        self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES]
    }

    pub(crate) fn decrement_staircase_countdown_underflowed(&mut self) -> bool {
        let value = self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES].wrapping_sub(1);
        let underflowed = (value as i8).is_negative();
        self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] = if underflowed { 0 } else { value };
        underflowed
    }

    pub(crate) fn increment_door_animation_step(&mut self) -> u16 {
        let step = read_le_u16(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON).wrapping_add(1);
        write_le_u16(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, step);
        step
    }

    pub(crate) fn set_staircase_index(&mut self, value: u8) {
        self.ram[WHICH_STAIRCASE_INDEX] = value;
    }

    pub(crate) fn set_staircase_move_counter(&mut self, value: u8) {
        self.ram[STAIRCASE_MOVE_COUNTER] = value;
    }

    pub(crate) fn decrement_staircase_move_counter(&mut self) -> u8 {
        self.ram[STAIRCASE_MOVE_COUNTER] = self.ram[STAIRCASE_MOVE_COUNTER].wrapping_sub(1);
        self.ram[STAIRCASE_MOVE_COUNTER]
    }

    pub(crate) fn set_inter_staircase_pos(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNG_INTER_STAIRCASES + index * 2, value);
    }

    pub(crate) fn clear_replacement_tile_states(&mut self) {
        self.ram[DUNGEON_REPLACEMENT_TILE_STATE..DUNGEON_REPLACEMENT_TILE_STATE + 32].fill(0);
    }

    pub(crate) fn set_replacement_tile_state(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNGEON_REPLACEMENT_TILE_STATE + index * 2, value);
    }

    pub(crate) fn increment_replacement_tile_state(&mut self, index: usize) -> u16 {
        let state =
            read_le_u16(self.ram, DUNGEON_REPLACEMENT_TILE_STATE + index * 2).wrapping_add(1);
        write_le_u16(self.ram, DUNGEON_REPLACEMENT_TILE_STATE + index * 2, state);
        state
    }

    pub(crate) fn set_savegame_state_high_bits(&mut self, mask: u8) {
        self.ram[DUNG_SAVEGAME_STATE_BITS + 1] |= mask;
    }

    pub(crate) fn clear_savegame_state_bits(&mut self) {
        write_le_u16(self.ram, DUNG_SAVEGAME_STATE_BITS, 0);
    }

    pub(crate) fn set_savegame_state_bits(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_SAVEGAME_STATE_BITS, value);
    }

    pub(crate) fn or_savegame_state_bits(&mut self, mask: u16) -> u16 {
        let value = read_le_u16(self.ram, DUNG_SAVEGAME_STATE_BITS) | mask;
        write_le_u16(self.ram, DUNG_SAVEGAME_STATE_BITS, value);
        value
    }

    pub(crate) fn clear_savegame_state_high(&mut self) {
        self.ram[DUNG_SAVEGAME_STATE_BITS + 1] = 0;
    }

    pub(crate) fn clear_savegame_state_low(&mut self) {
        self.ram[DUNG_SAVEGAME_STATE_BITS] = 0;
    }

    pub(crate) fn set_room_index2(&mut self, value: u8) {
        self.ram[DUNGEON_ROOM_INDEX2] = value;
    }

    pub(crate) fn set_room_index2_word(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGEON_ROOM_INDEX2, value);
    }

    pub(crate) fn set_bg2_tile(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNG_BG2 + index * 2, value);
    }

    pub(crate) fn set_bg1_tile(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNG_BG1 + index * 2, value);
    }

    pub(crate) fn set_bg1_tile_by_byte_pos(&mut self, pos: u16, value: u16) {
        self.set_bg1_tile((pos >> 1) as usize, value);
    }

    pub(crate) fn set_bg2_tile_by_byte_pos(&mut self, pos: u16, value: u16) {
        self.set_bg2_tile((pos >> 1) as usize, value);
    }

    pub(crate) fn set_room_tilemap_word(&mut self, base: usize, dsto: u16, value: u16) {
        write_le_u16(self.ram, base + dsto as usize * 2, value);
    }

    pub(crate) fn set_room_tilemap_word_by_byte_offset(
        &mut self,
        base: usize,
        byte_offset: usize,
        value: u16,
    ) {
        write_le_u16(self.ram, base + byte_offset, value);
    }

    pub(crate) fn clear_door_tilemap_addresses(&mut self) {
        self.ram[DUNG_DOOR_TILEMAP_ADDRESS..DUNG_DOOR_TILEMAP_ADDRESS + 32].fill(0);
    }

    pub(crate) fn set_door_tilemap_address(&mut self, door: usize, value: u16) {
        write_le_u16(self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2, value);
    }

    pub(crate) fn set_object_tilemap_pos(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNG_OBJECT_TILEMAP_POS + index * 2, value);
    }

    pub(crate) fn set_object_data_pos(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2, value);
    }

    pub(crate) fn set_misc_object_index(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_MISC_OBJS_INDEX, value);
    }

    pub(crate) fn clear_misc_object_index(&mut self) {
        self.ram[DUNG_MISC_OBJS_INDEX] = 0;
    }

    pub(crate) fn advance_misc_object_index_by(&mut self, value: u16) -> u16 {
        let next = read_le_u16(self.ram, DUNG_MISC_OBJS_INDEX).wrapping_add(value);
        write_le_u16(self.ram, DUNG_MISC_OBJS_INDEX, next);
        next
    }

    pub(crate) fn set_load_ptr_offset(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_LOAD_PTR_OFFS, value);
    }

    pub(crate) fn advance_load_ptr_offset_by(&mut self, value: u16) -> u16 {
        let next = read_le_u16(self.ram, DUNG_LOAD_PTR_OFFS).wrapping_add(value);
        write_le_u16(self.ram, DUNG_LOAD_PTR_OFFS, next);
        next
    }

    pub(crate) fn set_current_door_index(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_CUR_DOOR_IDX, value);
    }

    pub(crate) fn set_quadrants_visited(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_QUADRANTS_VISITED, value);
    }

    pub(crate) fn set_hud_floor_changed_timer(&mut self, value: u8) {
        self.ram[HUD_FLOOR_CHANGED_TIMER] = value;
    }

    pub(crate) fn or_quadrants_visited(&mut self, value: u16) -> u16 {
        let visited = word(self.ram, DUNG_QUADRANTS_VISITED) | value;
        write_le_u16(self.ram, DUNG_QUADRANTS_VISITED, visited);
        visited
    }

    pub(crate) fn mark_blast_wall_x_open(&mut self) {
        self.ram[DUNG_BLASTWALL_FLAG_X] = 1;
    }

    pub(crate) fn mark_blast_wall_y_open(&mut self) {
        self.ram[DUNG_BLASTWALL_FLAG_Y] = 1;
    }

    pub(crate) fn set_crush_wall_progress(&mut self, value: u16) {
        write_le_u16(self.ram, CRUSH_WALL_PROGRESS, value);
    }

    pub(crate) fn set_crush_wall_progress_low(&mut self, value: u8) {
        self.ram[CRUSH_WALL_PROGRESS] = value;
    }

    pub(crate) fn advance_crush_wall_progress_by(&mut self, delta: u16) -> u16 {
        let value = word(self.ram, CRUSH_WALL_PROGRESS).wrapping_add(delta);
        self.set_crush_wall_progress(value);
        value
    }

    pub(crate) fn add_reset_xy_check_flags(&mut self, value: u16) -> u16 {
        let flags = word(self.ram, RESET_XY_CHECK_FLAGS) | value;
        write_le_u16(self.ram, RESET_XY_CHECK_FLAGS, flags);
        flags
    }

    pub(crate) fn set_current_door_index_for_slot(&mut self, door: usize) {
        write_le_u16(self.ram, DUNG_CUR_DOOR_IDX, (door * 2) as u16);
    }

    pub(crate) fn advance_current_door_index_by(&mut self, value: u16) -> u16 {
        let next = read_le_u16(self.ram, DUNG_CUR_DOOR_IDX).wrapping_add(value);
        write_le_u16(self.ram, DUNG_CUR_DOOR_IDX, next);
        next
    }

    pub(crate) fn set_current_door_pos(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_CUR_DOOR_POS_DUNGEON, value);
    }

    pub(crate) fn clear_current_door_pos(&mut self) {
        write_le_u16(self.ram, DUNG_CUR_DOOR_POS_DUNGEON, 0);
    }

    pub(crate) fn set_door_open_counter(&mut self, value: u16) {
        write_le_u16(self.ram, DOOR_OPEN_CLOSED_COUNTER, value);
    }

    pub(crate) fn set_door_open_counter_low(&mut self, value: u8) {
        self.ram[DOOR_OPEN_CLOSED_COUNTER] = value;
    }

    pub(crate) fn clear_door_open_counter_low(&mut self) {
        self.ram[DOOR_OPEN_CLOSED_COUNTER] = 0;
    }

    pub(crate) fn increment_door_open_counter_low(&mut self) -> u8 {
        self.ram[DOOR_OPEN_CLOSED_COUNTER] = self.ram[DOOR_OPEN_CLOSED_COUNTER].wrapping_add(1);
        self.ram[DOOR_OPEN_CLOSED_COUNTER]
    }

    pub(crate) fn clear_replacement_tile_state_low(&mut self, index: usize) {
        self.ram[DUNGEON_REPLACEMENT_TILE_STATE + index * 2] = 0;
    }

    pub(crate) fn copy_custom_tile_attrs(&mut self, attrs: &[u8]) {
        self.ram[ATTRIBUTES_FOR_TILE_PLAYER + 0x140..ATTRIBUTES_FOR_TILE_PLAYER + 0x1c0]
            .copy_from_slice(attrs);
    }

    pub(crate) fn copy_default_tile_attrs_tail(&mut self, attrs: &[u8]) {
        self.ram[ATTRIBUTES_FOR_TILE_PLAYER + 0x1c0..ATTRIBUTES_FOR_TILE_PLAYER + 0x200]
            .copy_from_slice(attrs);
    }

    pub(crate) fn set_floor_1_filler_high(&mut self, value: u8) {
        self.ram[FLOOR_1_FILLER_TILES + 1] = value;
    }

    pub(crate) fn set_floor_2_filler_high(&mut self, value: u8) {
        self.ram[FLOOR_2_FILLER_TILES + 1] = value;
    }

    pub(crate) fn set_floor_1_filler_low(&mut self, value: u8) {
        self.ram[FLOOR_1_FILLER_TILES] = value;
    }

    pub(crate) fn set_floor_2_filler_low(&mut self, value: u8) {
        self.ram[FLOOR_2_FILLER_TILES] = value;
    }

    pub(crate) fn set_room_layout_and_starting_quadrant(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_LAYOUT_AND_STARTING_QUADRANT, value);
    }

    pub(crate) fn copy_bg2_draw_line_offsets(&mut self) {
        self.copy_line_pointer_bytes(&DUNGEON_DRAW_OBJECT_OFFSETS_BG2);
    }

    pub(crate) fn copy_bg1_draw_line_offsets(&mut self) {
        self.copy_line_pointer_bytes(&DUNGEON_DRAW_OBJECT_OFFSETS_BG1);
    }

    pub(crate) fn set_staircase_index_high(&mut self, value: u8) {
        self.ram[WHICH_STAIRCASE_INDEX + 1] = value;
    }

    pub(crate) fn fill_bg2_attr_range(&mut self, start: usize, len: usize, value: u8) {
        self.ram[DUNGEON_BG2_ATTR_TABLE + start..DUNGEON_BG2_ATTR_TABLE + start + len].fill(value);
    }

    pub(crate) fn clear_door_tables(&mut self) {
        self.ram[DOOR_TYPE_AND_SLOT..DOOR_TYPE_AND_SLOT + 32].fill(0);
        self.ram[DUNGEON_DOOR_DIRECTION..DUNGEON_DOOR_DIRECTION + 32].fill(0);
    }

    pub(crate) fn set_door_type_word(&mut self, door: usize, value: u16) {
        write_le_u16(self.ram, DOOR_TYPE_AND_SLOT + door * 2, value);
    }

    pub(crate) fn set_door_direction_word(&mut self, door: usize, value: u16) {
        write_le_u16(self.ram, DUNGEON_DOOR_DIRECTION + door * 2, value);
    }

    pub(crate) fn clear_door_direction(&mut self, door: usize) {
        self.set_door_direction_word(door, 0);
    }

    pub(crate) fn set_adjacent_door_flags(&mut self, value: u16) {
        write_le_u16(self.ram, ADJACENT_DOORS_FLAGS, value);
    }

    pub(crate) fn mark_adjacent_door_flag(&mut self, index: usize) -> u16 {
        let flags = word(self.ram, ADJACENT_DOORS_FLAGS) | (0x8000u16 >> (index & 15));
        self.set_adjacent_door_flags(flags);
        flags
    }

    pub(crate) fn set_adjacent_door(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, ADJACENT_DOORS + index * 2, value);
    }

    pub(crate) fn mark_no_adjacent_doors(&mut self) {
        self.set_adjacent_door(0, 0xffff);
    }

    pub(crate) fn set_changeable_object_index(&mut self, index: usize, value: u8) {
        self.ram[CHANGEABLE_DUNGEON_OBJECT_INDEX + index] = value;
    }

    pub(crate) fn clear_changeable_object_index(&mut self, index: usize) {
        self.set_changeable_object_index(index, 0);
    }

    pub(crate) fn increment_trap_trigger_latch(&mut self) {
        self.ram[DUNGEON_TRAP_TRIGGER_LATCH] = self.ram[DUNGEON_TRAP_TRIGGER_LATCH].wrapping_add(1);
    }

    pub(crate) fn mark_trap_trigger_latched(&mut self) {
        self.ram[DUNGEON_TRAP_TRIGGER_LATCH] = 1;
    }

    pub(crate) fn clear_trap_trigger_latch(&mut self) {
        self.ram[DUNGEON_TRAP_TRIGGER_LATCH] = 0;
    }

    pub(crate) fn set_kind_of_in_room_staircase_word(&mut self, value: u16) {
        write_le_u16(self.ram, KIND_OF_IN_ROOM_STAIRCASE, value);
    }

    pub(crate) fn set_blast_wall_message_direction(&mut self, value: u16) {
        write_le_u16(self.ram, MESSAGING_BUF_DUNGEON + 0x1c, value);
    }

    pub(crate) fn set_blast_wall_message_position(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, MESSAGING_BUF_DUNGEON + 0x1a, x);
        write_le_u16(self.ram, MESSAGING_BUF_DUNGEON + 0x18, y);
    }

    pub(crate) fn set_room_history_entry(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNGEON_ROOM_HISTORY + index * 2, value);
    }

    pub(crate) fn reset_room_history(&mut self) {
        for index in 0..4 {
            write_le_u16(self.ram, DUNGEON_ROOM_HISTORY + index * 2, 0xffff);
        }
    }

    pub(crate) fn set_floor_move_flags(&mut self, value: u16) {
        write_le_u16(self.ram, DUNG_FLOOR_MOVE_FLAGS, value);
    }

    pub(crate) fn increment_floor_move_flags(&mut self) {
        let value = word(self.ram, DUNG_FLOOR_MOVE_FLAGS).wrapping_add(1);
        self.set_floor_move_flags(value);
    }

    pub(crate) fn clear_orange_blue_barrier_state(&mut self) {
        write_le_u16(self.ram, ORANGE_BLUE_BARRIER_STATE, 0);
    }

    pub(crate) fn clear_moving_floor_check_flags(&mut self) {
        write_le_u16(self.ram, MOVING_FLOOR_BG_CHECK_FLAGS, 0);
    }

    pub(crate) fn or_moving_floor_check_flags(&mut self, bits: u16) -> u16 {
        let next = read_le_u16(self.ram, MOVING_FLOOR_BG_CHECK_FLAGS) | bits;
        write_le_u16(self.ram, MOVING_FLOOR_BG_CHECK_FLAGS, next);
        next
    }

    pub(crate) fn copy_default_tile_attrs_head(&mut self, data: &[u8]) {
        self.ram[ATTRIBUTES_FOR_TILE_PLAYER..ATTRIBUTES_FOR_TILE_PLAYER + 0x140]
            .copy_from_slice(&data[..0x140]);
    }

    pub(crate) fn set_dungeon_dark_with_lantern(&mut self) {
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 1;
    }

    pub(crate) fn set_dungeon_dark_with_lantern_raw(&mut self, value: u8) {
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = value;
    }

    pub(crate) fn clear_dungeon_dark_with_lantern(&mut self) {
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 0;
    }

    pub(crate) fn toggle_orange_blue_barrier_state(&mut self) {
        self.ram[ORANGE_BLUE_BARRIER_STATE] ^= 1;
    }

    pub(crate) fn set_activate_bomb_trap_overlord(&mut self, value: u8) {
        self.ram[ACTIVATE_BOMB_TRAP_OVERLORD] = value;
    }

    pub(crate) fn set_minigame_credits(&mut self, value: u8) {
        self.ram[MINIGAME_CREDITS] = value;
    }

    pub(crate) fn decrement_minigame_credits(&mut self) -> u8 {
        self.ram[MINIGAME_CREDITS] = self.ram[MINIGAME_CREDITS].wrapping_sub(1);
        self.ram[MINIGAME_CREDITS]
    }

    pub(crate) fn clear_reserved_gfx_config(&mut self) {
        write_le_u16(self.ram, RESERVED_GFX_CONFIG_WORD, 0);
    }

    pub(crate) fn set_room_index_prev(&mut self, value: u8) {
        self.ram[DUNGEON_ROOM_INDEX_PREV] = value;
    }

    pub(crate) fn set_previous_room_index_word(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGEON_ROOM_INDEX_PREV, value);
    }

    pub(crate) fn set_layout_quadrant_key(&mut self, value: u8) {
        self.ram[COMPOSITE_OF_LAYOUT_AND_QUADRANT] = value;
    }

    pub(crate) fn update_layout_quadrant_key(&mut self, quadrant_y: u8, quadrant_x: u8) -> u8 {
        let key = self.ram[DUNG_LAYOUT_AND_STARTING_QUADRANT] | quadrant_y | quadrant_x;
        self.set_layout_quadrant_key(key);
        key
    }

    pub(crate) fn clear_room_transitioning_flags(&mut self) {
        self.ram[ROOM_TRANSITIONING_FLAGS] = 0;
    }

    pub(crate) fn skip_room_tags_once(&mut self) {
        self.ram[FLAG_SKIP_CALL_TAG_ROUTINES] =
            self.ram[FLAG_SKIP_CALL_TAG_ROUTINES].wrapping_add(1);
    }

    pub(crate) fn clear_room_tag_skip(&mut self) {
        self.ram[FLAG_SKIP_CALL_TAG_ROUTINES] = 0;
    }

    pub(crate) fn set_cached_room_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        write_le_u16(self.ram, CACHED_ROOM_BOUNDS_Y_START, y_start);
        write_le_u16(self.ram, CACHED_ROOM_BOUNDS_Y_END, y_end);
        write_le_u16(self.ram, CACHED_ROOM_BOUNDS_X_START, x_start);
        write_le_u16(self.ram, CACHED_ROOM_BOUNDS_X_END, x_end);
    }

    pub(crate) fn set_standing_in_doorway_cached(&mut self, value: u8) {
        self.ram[IS_STANDING_IN_DOORWAY_CACHED] = value;
    }

    pub(crate) fn cache_standing_in_doorway(&mut self) {
        self.ram[IS_STANDING_IN_DOORWAY_CACHED] = self.ram[IS_STANDING_IN_DOORWAY];
    }
}

pub(crate) struct DungeonEntranceBackupViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DungeonEntranceBackupViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn cache_exit_tile_themes(&mut self) {
        self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX] = self.ram[OVERWORLD_TILE_THEME_INDEX];
        self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX + 1] = self.ram[MAIN_TILE_THEME_INDEX];
        self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX + 2] = self.ram[AUX_TILE_THEME_INDEX];
        self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX + 3] = self.ram[SPRITE_GRAPHICS_INDEX];
    }

    pub(crate) fn clear_overworld_screen_high(&mut self) {
        self.ram[OVERWORLD_SCREEN_INDEX + 1] = 0;
    }

    pub(crate) fn clear_overlay_high(&mut self) {
        self.ram[OVERLAY_INDEX + 1] = 0;
    }
}

pub(crate) struct DungeonHeaderView<'a> {
    ram: &'a [u8],
}

impl<'a> DungeonHeaderView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn travel_destination(&self, index: usize) -> u8 {
        byte(self.ram, DUNGEON_HEADER_TRAVEL_DESTINATIONS + index)
    }

    pub(crate) fn hole_teleporter_plane(&self, index: usize) -> u8 {
        byte(self.ram, DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + index)
    }

    pub(crate) fn staircase_plane(&self, index: usize) -> u8 {
        byte(self.ram, DUNGEON_HEADER_STAIRCASE_PLANE + index)
    }
}

pub(crate) struct DungeonHeaderViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DungeonHeaderViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_hole_teleporter_planes(&mut self, packed: u8, extra: u8) {
        self.ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE] = packed & 3;
        self.ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 1] = (packed >> 2) & 3;
        self.ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 2] = (packed >> 4) & 3;
        self.ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 3] = (packed >> 6) & 3;
        self.ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 4] = extra & 3;
    }
}

pub(crate) struct DungeonKeySlotsRawView<'a> {
    ram: &'a [u8],
}

impl<'a> DungeonKeySlotsRawView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn keys_earned(&self, palace_index_x2: u8) -> u8 {
        byte(
            self.ram,
            LINK_KEYS_EARNED_PER_DUNGEON + usize::from(palace_index_x2 >> 1),
        )
    }

    pub(crate) fn keys_earned_slot(&self, slot: usize) -> u8 {
        byte(self.ram, LINK_KEYS_EARNED_PER_DUNGEON + slot)
    }
}

pub(crate) struct DungeonKeySlotsRawViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DungeonKeySlotsRawViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_keys_earned(&mut self, palace_index_x2: u8, keys: u8) {
        self.ram[LINK_KEYS_EARNED_PER_DUNGEON + usize::from(palace_index_x2 >> 1)] = keys;
    }

    pub(crate) fn set_keys_earned_slot(&mut self, slot: usize, keys: u8) {
        self.ram[LINK_KEYS_EARNED_PER_DUNGEON + slot] = keys;
    }
}

pub(crate) struct DungeonTorchView<'a> {
    ram: &'a [u8],
}

impl<'a> DungeonTorchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn timer(&self, index: usize) -> u8 {
        byte(self.ram, TORCH_TIMERS + index)
    }

    pub(crate) fn attr_index(&self) -> usize {
        usize::from(byte(self.ram, DUNGEON_TORCH_ATTR) & 0x0f)
    }

    pub(crate) fn torch_attr(&self) -> u8 {
        byte(self.ram, DUNGEON_TORCH_ATTR)
    }

    pub(crate) fn ganon_torch_count(&self) -> u8 {
        byte(self.ram, GANON_TORCH_COUNT)
    }

    pub(crate) fn torches_start_index(&self) -> u16 {
        word(self.ram, DUNG_INDEX_OF_TORCHES_START)
    }

    pub(crate) fn torch_object_data_pos(&self, index: usize) -> u16 {
        word(self.ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2)
    }
}

pub(crate) struct DungeonTorchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DungeonTorchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
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
        self.ram[TORCH_TIMERS + index] = 0;
    }

    pub(crate) fn set_timer(&mut self, index: usize, value: u8) {
        self.ram[TORCH_TIMERS + index] = value;
    }

    pub(crate) fn set_torch_data_word(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNGEON_TORCH_DATA + index * 2, value);
    }

    pub(crate) fn set_attr(&mut self, value: u8) {
        self.ram[DUNGEON_TORCH_ATTR] = value;
    }

    pub(crate) fn set_ganon_torch_count(&mut self, value: u8) {
        self.ram[GANON_TORCH_COUNT] = value;
    }

    pub(crate) fn clear_attr(&mut self) {
        self.set_attr(0);
    }
}

pub(crate) struct ScratchWordView<'a> {
    ram: &'a [u8],
}

impl<'a> ScratchWordView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn high(&self) -> u8 {
        byte(self.ram, DUNGEON_WORK_R16 + 1)
    }

    pub(crate) fn word(&self) -> u16 {
        word(self.ram, DUNGEON_WORK_R16)
    }

    pub(crate) fn minigame_previous_chest_choice(&self) -> u8 {
        byte(self.ram, DUNGEON_WORK_R16)
    }
}

pub(crate) struct ScratchWordViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> ScratchWordViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn decrement_high(&mut self) -> u8 {
        let next = self.ram[DUNGEON_WORK_R16 + 1].wrapping_sub(1);
        self.ram[DUNGEON_WORK_R16 + 1] = next;
        next
    }

    pub(crate) fn set_word(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGEON_WORK_R16, value);
    }

    pub(crate) fn clear_word(&mut self) {
        self.set_word(0);
    }

    pub(crate) fn set_liftable_tile_probe_position(&mut self, y: u16, x: u16) {
        write_le_u16(self.ram, DUNGEON_WORK_R16, y);
        write_le_u16(self.ram, DUNGEON_WORK_R18, x);
    }

    pub(crate) fn set_ganon_door_bounce_countdown(&mut self, value: u16) {
        self.set_word(value);
    }

    pub(crate) fn decrement_ganon_door_bounce_low(&mut self) -> u8 {
        let next = self.ram[DUNGEON_WORK_R16].wrapping_sub(1);
        self.ram[DUNGEON_WORK_R16] = next;
        next
    }

    pub(crate) fn clear_module_transition_counter(&mut self) {
        self.ram[DUNGEON_WORK_R16] = 0;
    }

    pub(crate) fn set_minigame_previous_chest_choice(&mut self, value: u8) {
        self.ram[DUNGEON_WORK_R16] = value;
    }
}

pub(crate) struct EndingScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> EndingScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn primary_word(&self) -> u16 {
        word(self.ram, ENDING_WORK_PRIMARY)
    }

    pub(crate) fn secondary_word(&self) -> u16 {
        word(self.ram, ENDING_WORK_SECONDARY)
    }

    pub(crate) fn primary_low(&self) -> u8 {
        byte(self.ram, ENDING_WORK_PRIMARY)
    }

    pub(crate) fn secondary_low(&self) -> u8 {
        byte(self.ram, ENDING_WORK_SECONDARY)
    }
}

pub(crate) struct EndingScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> EndingScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_primary_word(&mut self, value: u16) {
        write_le_u16(self.ram, ENDING_WORK_PRIMARY, value);
    }

    pub(crate) fn set_secondary_word(&mut self, value: u16) {
        write_le_u16(self.ram, ENDING_WORK_SECONDARY, value);
    }

    pub(crate) fn clear_primary_word(&mut self) {
        self.set_primary_word(0);
    }

    pub(crate) fn set_primary_low(&mut self, value: u8) {
        self.ram[ENDING_WORK_PRIMARY] = value;
    }

    pub(crate) fn decrement_primary_low(&mut self) -> u8 {
        self.ram[ENDING_WORK_PRIMARY] = self.ram[ENDING_WORK_PRIMARY].wrapping_sub(1);
        self.ram[ENDING_WORK_PRIMARY]
    }

    pub(crate) fn increment_secondary_low(&mut self) -> u8 {
        self.ram[ENDING_WORK_SECONDARY] = self.ram[ENDING_WORK_SECONDARY].wrapping_add(1);
        self.ram[ENDING_WORK_SECONDARY]
    }
}

pub(crate) struct SaveLoadScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> SaveLoadScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn source_offset(&self) -> u16 {
        word(self.ram, SAVE_LOAD_SOURCE_OFFSET)
    }

    pub(crate) fn source_offset_usize(&self) -> usize {
        usize::from(self.source_offset())
    }
}

pub(crate) struct SaveLoadScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> SaveLoadScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_source_offset(&mut self, value: u16) {
        write_le_u16(self.ram, SAVE_LOAD_SOURCE_OFFSET, value);
    }
}

pub(crate) struct DungeonMapScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> DungeonMapScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn scroll_draw_offset(&self) -> u16 {
        word(self.ram, DUNGEON_MAP_SCROLL_DRAW_OFFSET)
    }

    pub(crate) fn scroll_input(&self) -> u16 {
        word(self.ram, DUNGEON_MAP_SCROLL_INPUT)
    }

    pub(crate) fn scroll_input_direction_index(&self) -> usize {
        usize::from((self.scroll_input() >> 3) & 1)
    }

    pub(crate) fn marker_x_offset(&self) -> u16 {
        word(self.ram, DUNGEON_MAP_MARKER_X_OFFSET)
    }

    pub(crate) fn marker_y_offset(&self) -> u16 {
        word(self.ram, DUNGEON_MAP_MARKER_Y_OFFSET)
    }

    pub(crate) fn location_marker_base_y(&self) -> u8 {
        byte(self.ram, DUNGEON_MAP_LOCATION_MARKER_BASE_Y)
    }

    pub(crate) fn dungmap_init_state(&self) -> u8 {
        byte(self.ram, DUNGMAP_INIT_STATE)
    }

    pub(crate) fn dungmap_cur_floor(&self) -> u16 {
        word(self.ram, DUNGMAP_CUR_FLOOR)
    }

    pub(crate) fn dungmap_cur_floor_byte(&self) -> u8 {
        byte(self.ram, DUNGMAP_CUR_FLOOR)
    }

    pub(crate) fn dungmap_floor_scroll_step(&self) -> u8 {
        byte(self.ram, DUNGMAP_FLOOR_SCROLL_STEP)
    }

    pub(crate) fn dungmap_idx(&self) -> u16 {
        word(self.ram, DUNGMAP_IDX)
    }

    pub(crate) fn dungmap_scroll_target_y(&self) -> u16 {
        word(self.ram, DUNGMAP_SCROLL_TARGET_Y)
    }

    pub(crate) fn dungmap_player_marker_x(&self) -> u16 {
        word(self.ram, DUNGMAP_PLAYER_MARKER_X)
    }

    pub(crate) fn dungmap_player_marker_x_byte(&self) -> u8 {
        byte(self.ram, DUNGMAP_PLAYER_MARKER_X)
    }

    pub(crate) fn dungmap_player_marker_y(&self) -> u16 {
        word(self.ram, DUNGMAP_PLAYER_MARKER_Y)
    }
}

pub(crate) struct DungeonMapScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DungeonMapScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_scroll_state(&mut self) {
        write_le_u16(self.ram, DUNGEON_MAP_SCROLL_DRAW_OFFSET, 0);
        write_le_u16(self.ram, DUNGEON_MAP_SCROLL_INPUT, 0);
    }

    pub(crate) fn set_scroll_draw_offset(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGEON_MAP_SCROLL_DRAW_OFFSET, value);
    }

    pub(crate) fn set_scroll_input(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGEON_MAP_SCROLL_INPUT, value);
    }

    pub(crate) fn reset_marker_offsets(&mut self) {
        write_le_u16(self.ram, DUNGEON_MAP_MARKER_X_OFFSET, 0x0040);
        write_le_u16(self.ram, DUNGEON_MAP_MARKER_Y_OFFSET, 0x0040);
    }

    pub(crate) fn set_marker_x_offset(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGEON_MAP_MARKER_X_OFFSET, value);
    }

    pub(crate) fn set_marker_y_offset(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGEON_MAP_MARKER_Y_OFFSET, value);
    }

    pub(crate) fn set_location_marker_base_y(&mut self, value: u8) {
        self.ram[DUNGEON_MAP_LOCATION_MARKER_BASE_Y] = value;
        self.ram[DUNGEON_MAP_LOCATION_MARKER_BASE_Y + 1] = 0;
    }

    pub(crate) fn shift_marker_x_left(&mut self) -> u16 {
        let value = word(self.ram, DUNGEON_MAP_MARKER_X_OFFSET).wrapping_sub(0x10);
        write_le_u16(self.ram, DUNGEON_MAP_MARKER_X_OFFSET, value);
        value
    }

    pub(crate) fn reset_marker_x_offset(&mut self) {
        write_le_u16(self.ram, DUNGEON_MAP_MARKER_X_OFFSET, 0x0040);
    }

    pub(crate) fn shift_marker_y_low_up(&mut self) {
        self.ram[DUNGEON_MAP_MARKER_Y_OFFSET] =
            self.ram[DUNGEON_MAP_MARKER_Y_OFFSET].wrapping_sub(0x10);
    }

    pub(crate) fn add_marker_y_offset_signed(&mut self, value: i16) -> u16 {
        let value = word(self.ram, DUNGEON_MAP_MARKER_Y_OFFSET).wrapping_add_signed(value);
        write_le_u16(self.ram, DUNGEON_MAP_MARKER_Y_OFFSET, value);
        value
    }

    pub(crate) fn increment_dungmap_init_state(&mut self) {
        self.ram[DUNGMAP_INIT_STATE] = self.ram[DUNGMAP_INIT_STATE].wrapping_add(1);
    }

    pub(crate) fn clear_dungmap_init_state(&mut self) {
        self.ram[DUNGMAP_INIT_STATE] = 0;
    }

    pub(crate) fn set_dungmap_cur_floor(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGMAP_CUR_FLOOR, value);
    }

    pub(crate) fn decrement_dungmap_cur_floor_byte(&mut self) {
        self.ram[DUNGMAP_CUR_FLOOR] = self.ram[DUNGMAP_CUR_FLOOR].wrapping_sub(1);
    }

    pub(crate) fn increment_dungmap_cur_floor(&mut self) -> u16 {
        let value = word(self.ram, DUNGMAP_CUR_FLOOR).wrapping_add(1);
        write_le_u16(self.ram, DUNGMAP_CUR_FLOOR, value);
        value
    }

    pub(crate) fn increment_dungmap_cur_floor_byte(&mut self) {
        self.ram[DUNGMAP_CUR_FLOOR] = self.ram[DUNGMAP_CUR_FLOOR].wrapping_add(1);
    }

    pub(crate) fn set_dungmap_floor_scroll_step(&mut self, value: u8) {
        self.ram[DUNGMAP_FLOOR_SCROLL_STEP] = value;
    }

    pub(crate) fn clear_dungmap_floor_scroll_step(&mut self) {
        self.ram[DUNGMAP_FLOOR_SCROLL_STEP] = 0;
    }

    pub(crate) fn increment_dungmap_floor_scroll_step(&mut self) {
        self.ram[DUNGMAP_FLOOR_SCROLL_STEP] = self.ram[DUNGMAP_FLOOR_SCROLL_STEP].wrapping_add(1);
    }

    pub(crate) fn set_dungmap_idx(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGMAP_IDX, value);
    }

    pub(crate) fn clear_dungmap_idx(&mut self) {
        write_le_u16(self.ram, DUNGMAP_IDX, 0);
    }

    pub(crate) fn set_dungmap_scroll_target_y(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGMAP_SCROLL_TARGET_Y, value);
    }

    pub(crate) fn set_dungmap_player_marker_x(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGMAP_PLAYER_MARKER_X, value);
    }

    pub(crate) fn set_dungmap_player_marker_y(&mut self, value: u16) {
        write_le_u16(self.ram, DUNGMAP_PLAYER_MARKER_Y, value);
    }
}

pub(crate) struct TempCounterView<'a> {
    ram: &'a [u8],
}

impl<'a> TempCounterView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn value(&self) -> u8 {
        byte(self.ram, TEMP_COUNTER)
    }

    pub(crate) fn as_usize(&self) -> usize {
        usize::from(self.value())
    }

    pub(crate) fn is_negative(&self) -> bool {
        (self.value() as i8).is_negative()
    }
}

pub(crate) struct TempCounterViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> TempCounterViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set(&mut self, value: u8) {
        self.ram[TEMP_COUNTER] = value;
    }

    pub(crate) fn decrement(&mut self) -> u8 {
        let next = self.ram[TEMP_COUNTER].wrapping_sub(1);
        self.ram[TEMP_COUNTER] = next;
        next
    }
}

pub(crate) struct DungeonSecretScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> DungeonSecretScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn pending_kind(&self) -> u8 {
        byte(self.ram, DUNGEON_SECRET_PENDING_KIND)
    }

    pub(crate) fn overworld_subst_counter(&self) -> u8 {
        byte(self.ram, OVERWORLD_SECRET_SUBST_CTR)
    }

    pub(crate) fn has_pending_kind(&self) -> bool {
        self.pending_kind() != 0
    }

    pub(crate) fn is_available(&self) -> bool {
        self.pending_kind() != 0xff
    }

    pub(crate) fn graphics_kind(&self) -> Option<u8> {
        let value = self.pending_kind();
        if value & 0x80 != 0 {
            Some(value & 0x7f)
        } else {
            None
        }
    }
}

pub(crate) struct DungeonSecretScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DungeonSecretScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_pending_kind(&mut self) {
        self.ram[DUNGEON_SECRET_PENDING_KIND] = 0;
    }

    pub(crate) fn set_pending_kind(&mut self, value: u8) {
        self.ram[DUNGEON_SECRET_PENDING_KIND] = value;
    }

    pub(crate) fn increment_overworld_subst_counter(&mut self) {
        self.ram[OVERWORLD_SECRET_SUBST_CTR] = self.ram[OVERWORLD_SECRET_SUBST_CTR].wrapping_add(1);
    }

    pub(crate) fn set_powder_pending_kind(&mut self) {
        write_le_u16(self.ram, DUNGEON_SECRET_PENDING_KIND, 4);
    }

    pub(crate) fn or_pending_kind(&mut self, value: u8) {
        self.ram[DUNGEON_SECRET_PENDING_KIND] |= value;
    }

    pub(crate) fn mark_graphics_kind(&mut self) {
        self.ram[DUNGEON_SECRET_PENDING_KIND] |= 0x80;
    }
}
