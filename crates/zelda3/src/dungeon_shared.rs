pub(super) const DUNG_REPLACEMENT_TILE_SRC_POS_X2: usize = 0x47c;
pub(super) const DUNG_NUM_STAIRS_1: usize = 0x49a;
pub(super) const DUNG_NUM_STAIRS_2: usize = 0x49c;
pub(super) const DUNG_NUM_STAIRS_WET: usize = 0x49e;
pub(super) const DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS: usize = 0x440;
pub(super) const DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER: usize = 0x442;
pub(super) const DUNG_NUM_ACTIVATED_WATER_LADDERS: usize = 0x444;
// NES_Ver2: UDSCKP6, "water-side STEP (kirikae)".
pub(super) const WATER_SIDE_STEP_SWITCH: usize = 0x448;
pub(super) const KIND_OF_IN_ROOM_STAIRCASE_DUNGEON: usize = 0x44a;
pub(super) const DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER: usize = 0x4ae;
pub(super) const DUNG_NUM_STAR_SHAPED_SWITCHES: usize = 0x432;
pub(super) const DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS: usize = 0x438;
pub(super) const DUNG_NUM_INROOM_UPNORTH_STAIRS: usize = 0x43c;
pub(super) const DUNG_NUM_INROOM_SOUTHDOWN_STAIRS: usize = 0x43e;
pub(super) const DUNG_NUM_WATER_LADDERS: usize = 0x446;
pub(super) const DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS: usize = 0x4a6;
pub(super) const DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS: usize = 0x4a8;
pub(super) const DUNG_STAIRS_TABLE_2: usize = 0x6ec;
pub(super) const DUNG_STAIRS_TABLE_1: usize = 0x6b8;
pub(super) const STAR_SHAPED_SWITCHES_TILE: usize = 0x6a0;
pub(super) const DUNG_FLOOR_MOVE_FLAGS: usize = 0x41a;
pub(super) const DUNG_FLOOR_Y_VEL_DUNGEON: usize = 0x310;
pub(super) const DUNG_FLOOR_X_OFFS: usize = 0x422;
// NES_Ver2: RSXYCKF, "reset x,y check flag".
// NES_Ver2: B1CWPT/BG1MPT, moving-wall write point and dot pointer.
pub(super) const MOVING_WALL_WRITE_POINT: usize = 0x42a;
pub(super) const MOVING_WALL_DOT_POINTER: usize = 0x41e;
pub(super) const ROOM_QUADRANT_UPLOAD_TABLE_MASK: usize = 0x0f;
pub(super) const MOVING_WALL_ARR1: usize = 0xc880;
pub(super) const INVISIBLE_DOOR_DIR_AND_INDEX_X2: usize = 0x436;
pub(super) const TRANSITION_COUNTER: usize = 0x0126;
pub(super) const DUNG_FLAG_TRAPDOORS_DOWN: usize = 0x468;
pub(super) const DUNG_FLAG_STATECHANGE_WATERPUZZLE: usize = 0x642;
// NES_Ver2: WGTPNT, water-gate pointer.
pub(super) const WATERGATE_POINTER: usize = 0x0470;
pub(super) const WATERGATE_POS: usize = 0x0472;
pub(super) const WATERGATE_SPOTLIGHT_Y_UPPER: usize = 0x0678;
// NES_Ver2 WRWP*/OYK*/WIN* water-window HDMA work RAM.
pub(super) const WATER_HDMA_WINDOW_X_DUNGEON: usize = 0x0680;
pub(super) const WATER_HDMA_WINDOW_Y_DUNGEON: usize = 0x0682;
pub(super) const WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON: usize = 0x0684;
pub(super) const WATER_HDMA_WINDOW_X_RADIUS_DUNGEON: usize = 0x0686;
pub(super) const WATER_HDMA_WINDOW_Y_TARGET_DUNGEON: usize = 0x0688;
pub(super) const WATER_HDMA_WINDOW_Y_RADIUS_ALT_DUNGEON: usize = 0x068a;
pub(super) const TURN_ON_OFF_WATER_CTR: usize = 0x0424;
pub(super) const MINIGAME_CREDITS: usize = 0x04c4;
pub(super) const DUNG_TRANSITION_LANDING_CLASS: usize = 0x004e;
pub(super) const DUNG_CUR_DOOR_POS_DUNGEON: usize = 0x068e;
pub(super) const DOOR_ANIMATION_STEP_INDICATOR_DUNGEON: usize = 0x0690;
pub(super) const DUNG_WHICH_KEY_X2_DUNGEON: usize = 0x0694;
pub(super) const DUNG_DOOR_SWITCH_TRIGGERED: usize = 0x0430;
pub(super) const DUNG_CUR_QUADRANT_UPLOAD: usize = 0x045c;
// NES_Ver2: CWLFLG/CWLPNT, crush-wall progress and doubled door index.
pub(super) const CRUSH_WALL_PROGRESS_DUNGEON: usize = 0x0454;
pub(super) const CRUSH_WALL_DOOR_INDEX_X2_DUNGEON: usize = 0x0456;
pub(super) const DUNG_DOOR_BARRIER_OR_SWITCH_FLAG: usize = 0x045e;
pub(super) const BLOCK_TRAP_CHECK_FLAG: usize = 0x0466;
pub(super) const DUNG_REPLACEMENT_TILE_DST_POS_X2: usize = 0x04b6;
pub(super) const MOVING_WALL_TORCH_BLINK_PHASE: usize = 0x04bc;
pub(super) const MOVING_WALL_TORCH_UPDATE_FLAG: usize = 0x04c2;
pub(super) const DUNG_FLAG_SOMARIA_BLOCK_SWITCH: usize = 0x0646;
pub(super) const DUNG_INTER_STAIRCASES: usize = 0x06b0;
pub(super) const STAIRCASE_TILEMAP_POS_X2: usize = 0x048c;
pub(super) const DUNG_NUM_TOGGLE_FLOOR: usize = 0x44e;
pub(super) const DUNG_NUM_TOGGLE_PALACE: usize = 0x450;
pub(super) const DUNG_TOGGLE_FLOOR_POS: usize = 0x6c0;
pub(super) const DUNG_TOGGLE_PALACE_POS: usize = 0x6d0;
pub(super) const ADJACENT_DOORS_FLAGS: usize = 0x1100;
pub(super) const ADJACENT_DOORS: usize = 0x1110;
// NES_Ver2: WRDADR, width road address.
pub(super) const DUNG_WIDTH_ROAD_ADDRESS: usize = 0x4b0;
pub(super) const ROOM_BG1_TILEMAP_BASE: usize = 0x4000;
pub(super) const ROOM_BG2_TILEMAP_BASE: usize = 0x2000;
pub(super) const DUNG_INDEX_X3: usize = 0x110;
pub(super) const PUSHEDBLOCKS_MAYBE_TIMEOUT: usize = 0x02c4;
pub(super) const PUSHEDBLOCK_FACING: usize = 0x05f8;
pub(super) const PUSH_BLOCK_DIRECTION_DUNGEON: usize = 0x0474;
pub(super) const MOVABLE_BLOCK_DATAS: usize = 0x0f940;
pub(super) const SPRITE_Y_RECOIL_DUNGEON: usize = 0x0f30;
pub(super) const DUNG_HDR_BG2_PROPERTIES_BACKUP: usize = 0xc208;
pub(super) const WHICH_STAIRCASE_INDEX: usize = 0x462;
// NES_Ver2: SPMVCT, step/staircase move counter.
pub(super) const STAIRCASE_MOVE_COUNTER: usize = 0x464;
pub(super) const CUR_STAIRCASE_PLANE: usize = 0x48a;
pub(super) const DUNG_HDR_STAIRCASE_PLANE: usize = 0x63d;
pub(super) const STAIRCASE_LOWER_LEVEL_STATUS: usize = 0x492;
pub(super) const COUNTDOWN_TIMER_FOR_STAIRCASES: usize = 0x378;
pub(super) const DOOR_DEBRIS_DIRECTION_DUNGEON: usize = 0x073c;
pub(super) const FLAG_WHICH_MUSIC_TYPE_DUNGEON: usize = 0x136;
pub(super) const MESSAGING_BUF_DUNGEON: usize = 0x10000;
pub(super) const DUNG_TORCH_TIMERS_DUNGEON: usize = 0x04f0;
pub(super) const POTS_REVEALED_IN_ROOM_DUNGEON: usize = 0x0f580;
pub(super) const UVRAM_DATA_DUNGEON: usize = 0x1100;
pub(super) const FEATURE_MISC_BUG_FIXES_DUNGEON: u32 = 4096;
pub(super) const FEATURE_BREAK_POTS_WITH_SWORD_DUNGEON: u32 = 32;
pub(super) const BIG_KEY_DOOR_MESSAGE_TRIGGERED_DUNGEON: usize = 0x04b8;
pub(super) const DUNG_LOAD_PTR: usize = 0x00b7;
pub(super) const DUNG_LOAD_PTR_BANK: usize = 0x00b9;

pub(super) fn parse_usize_env(value: &str) -> Option<usize> {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .and_then(|hex| usize::from_str_radix(hex, 16).ok())
        .or_else(|| value.parse::<usize>().ok())
}

pub(super) fn format_optional_hex(value: Option<u8>) -> String {
    value
        .map(|value| format!("0x{value:02x}"))
        .unwrap_or_else(|| "OOB".to_string())
}

pub(super) struct EntranceAssetSet {
    pub(super) rooms: usize,
    pub(super) relative_coords: usize,
    pub(super) scroll_x: usize,
    pub(super) scroll_y: usize,
    pub(super) player_x: usize,
    pub(super) player_y: usize,
    pub(super) camera_x: usize,
    pub(super) camera_y: usize,
    pub(super) blockset: usize,
    pub(super) floor: usize,
    pub(super) palace: usize,
    pub(super) doorway_orientation: usize,
    pub(super) starting_bg: usize,
    pub(super) quadrant1: usize,
    pub(super) quadrant2: usize,
    pub(super) door_settings: usize,
}

pub(super) const ENTRANCE_DATA_ASSETS: EntranceAssetSet = EntranceAssetSet {
    rooms: 11,
    relative_coords: 12,
    scroll_x: 13,
    scroll_y: 14,
    player_x: 15,
    player_y: 16,
    camera_x: 17,
    camera_y: 18,
    blockset: 19,
    floor: 20,
    palace: 21,
    doorway_orientation: 22,
    starting_bg: 23,
    quadrant1: 24,
    quadrant2: 25,
    door_settings: 26,
};

pub(super) const STARTING_POINT_ASSETS: EntranceAssetSet = EntranceAssetSet {
    rooms: 28,
    relative_coords: 29,
    scroll_x: 30,
    scroll_y: 31,
    player_x: 32,
    player_y: 33,
    camera_x: 34,
    camera_y: 35,
    blockset: 36,
    floor: 37,
    palace: 38,
    doorway_orientation: 39,
    starting_bg: 40,
    quadrant1: 41,
    quadrant2: 42,
    door_settings: 43,
};

pub(super) const LIFTABLE_TILE_PROBE_X_OFFSETS: [i16; 4] = [7, 7, -3, 16];
pub(super) const LIFTABLE_TILE_PROBE_Y_OFFSETS: [i16; 4] = [3, 24, 14, 14];
pub(super) const LIFTABLE_TILE_REPLACEMENT_ITEM_CODES: [u16; 16] = [
    0x5252, 0x5050, 0x5454, 0, 0x2323, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub(super) const DUNGEON_MINIGAME_CHEST_PRIZES: [u8; 8] =
    [0x40, 0x41, 0x34, 0x42, 0x43, 0x44, 0x27, 0x17];
pub(super) const RUPEE_CHEST_MINIGAME_PRIZES: [u8; 32] = [
    0x47, 0x34, 0x46, 0x34, 0x46, 0x46, 0x34, 0x47, 0x46, 0x47, 0x34, 0x46, 0x47, 0x34, 0x46, 0x47,
    0x34, 0x47, 0x41, 0x47, 0x41, 0x41, 0x47, 0x34, 0x41, 0x34, 0x47, 0x41, 0x34, 0x47, 0x41, 0x34,
];
pub(super) const DUNGEON_LIT_TORCH_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
pub(super) const DUNGEON_CRYSTAL_CUTSCENE_TILE_BASES: [u16; 7] =
    [0x1618, 0x1658, 0x1658, 0x1618, 0x0658, 0x1618, 0x1658];

pub(super) const DOOR_TYPE_REGULAR2: u8 = 2;
pub(super) const DOOR_TYPE_4: u8 = 4;
pub(super) const DOOR_TYPE_ENTRANCE_DOOR: u8 = 6;
pub(super) const DOOR_TYPE_WATERFALL_TUNNEL: u8 = 8;

pub(super) const DUNGEON_BOSS_ROOMS: [u16; 12] =
    [200, 51, 7, 32, 6, 90, 41, 144, 222, 164, 172, 13];
pub(super) const DOOR_TYPE_ENTRANCE_LARGE: u8 = 10;
pub(super) const DOOR_TYPE_ENTRANCE_LARGE2: u8 = 12;
pub(super) const DOOR_TYPE_ENTRANCE_CAVE: u8 = 14;
pub(super) const DOOR_TYPE_ENTRANCE_CAVE2: u8 = 16;
pub(super) const DOOR_TYPE_PLAYER_BG_CHANGE: u8 = 22;
pub(super) const DOOR_TYPE_INVISIBLE_DOOR: u8 = 26;
pub(super) const DOOR_TYPE_SMALL_KEY_DOOR: u8 = 0x1c;
pub(super) const DOOR_TYPE_1E: u8 = 0x1e;
pub(super) const DOOR_TYPE_STAIR_MASK_LOCKED0: u8 = 32;
pub(super) const DOOR_TYPE_STAIR_MASK_LOCKED2: u8 = 36;
pub(super) const DUNGEON_EXIT_SOURCE_ROOMS: [u8; 12] =
    [200, 51, 7, 32, 6, 90, 41, 144, 222, 164, 172, 13];
pub(super) const DUNGEON_EXIT_TARGET_ROOMS: [u8; 12] =
    [201, 99, 119, 32, 40, 74, 89, 152, 14, 214, 219, 13];
pub(super) const WATERGATE_LAYOUT_BYTES: [u8; 17] = [
    0x1b, 0xa1, 0xc9, 0x51, 0xa1, 0xc9, 0x92, 0xa1, 0xc9, 0xa1, 0x33, 0xc9, 0xa1, 0x72, 0xc9, 0xff,
    0xff,
];
pub(super) const DOOR_TYPE_STAIR_MASK_LOCKED3: u8 = 38;
pub(super) const DOOR_TYPE_BREAKABLE_WALL: u8 = 0x28;
pub(super) const DOOR_TYPE_LG_EXPLOSION: u8 = 48;
pub(super) const DOOR_TYPE_REGULAR_DOOR33: u8 = 64;
pub(super) const DOOR_TYPE_SHUTTER: u8 = 68;
pub(super) const DOOR_TYPE_WARP_ROOM_DOOR: u8 = 70;
pub(super) const DOOR_TYPE_SHUTTER_TRAP_UR: u8 = 72;
pub(super) const DOOR_TYPE_SHUTTER_TRAP_DL: u8 = 74;
pub(super) const DUNGEON_CRYSTAL_PENDANT_BITS: [u8; 13] =
    [0, 0, 4, 2, 0, 16, 2, 1, 64, 4, 1, 32, 8];
pub(super) const STAIRCASE_LANDING_COORDINATES: [i8; 20] = [
    12, 32, 48, 56, 72, -44, -40, -64, -64, -88, 12, 24, 40, 48, 64, -28, -40, -56, -64, -80,
];
pub(super) const SPIRAL_SUBSCREEN_LAYER_BY_BG2: [i8; 8] = [0, 1, 1, -1, 1, 1, 1, 1];
pub(super) const DUNGEON_TRANSITION_SCROLL_DELTAS: [i8; 4] = [4, -4, 4, -4];
pub(super) const DUNGEON_TRANSITION_PLAYER_MOVE_FRAMES: [u8; 4] = [52, 52, 59, 58];
pub(super) const STAIRCASE_CAMERA_LINK_Y_ADJUSTMENTS: [i8; 4] = [32, -64, 32, -32];
pub(super) const TELEPORT_PIT_PRIMARY_LEVELS: [u8; 3] = [0, 1, 1];
pub(super) const TELEPORT_PIT_SECONDARY_LEVELS: [u8; 3] = [0, 0, 1];
pub(super) const SPIRAL_STAIRCASE_X_OFFSETS: [i8; 4] = [-28, -28, 24, 24];
pub(super) const SPIRAL_STAIRCASE_Y_OFFSETS: [i8; 4] = [16, -10, -10, -32];
pub(super) const DOOR_ANIMATION_UP_SOURCES: [u16; 5] = [0x306a, 0x306a, 0x3082, 0x309a, 0x30b2];
pub(super) const DOOR_ANIMATION_DOWN_SOURCES: [u16; 5] = [0x30b2, 0x30ca, 0x30e2, 0x30fa, 0x3112];
pub(super) const DOOR_ANIMATION_LEFT_SOURCES: [u16; 5] = [0x3112, 0x312a, 0x3142, 0x315a, 0x3172];
pub(super) const DOOR_ANIMATION_RIGHT_SOURCES: [u16; 5] = [0x3172, 0x318a, 0x31a2, 0x31ba, 0x31d2];
pub(super) const DOOR_BLAST_WALL_UP_DESTINATIONS: [u16; 6] =
    [0x0d8a, 0x0daa, 0x0dca, 0x02b6, 0x0ab6, 0x12b6];
pub(super) const LAYOUT_QUADRANT_FLAGS: [u8; 32] = [
    0x0f, 0x0f, 0x0f, 0x0f, 0x0b, 0x0b, 7, 7, 0x0f, 0x0b, 0x0f, 7, 0x0b, 0x0f, 7, 0x0f, 0x0e, 0x0d,
    0x0e, 0x0d, 0x0f, 0x0f, 0x0e, 0x0d, 0x0e, 0x0d, 0x0f, 0x0f, 0x0a, 9, 6, 5,
];

pub(super) const DOOR_POSITION_LEFT: [u16; 12] = [
    0x784, 0xf84, 0x1784, 0x78a, 0xf8a, 0x178a, 0x7c4, 0xfc4, 0x17c4, 0x7ca, 0xfca, 0x17ca,
];
pub(super) const DOOR_POSITION_RIGHT: [u16; 12] = [
    0x7b4, 0xfb4, 0x17b4, 0x7ae, 0xfae, 0x17ae, 0x7f4, 0xff4, 0x17f4, 0x7ee, 0xfee, 0x17ee,
];
pub(super) const DOOR_TYPE_SRC_LEFT: [u16; 48] = [
    0x2c6a, 0x2c82, 0x2c82, 0x2c9a, 0x2c9a, 0x2c9a, 0x2c9a, 0x2c9a, 0x2c9a, 0x2cb2, 0x2cb2, 0x2cb2,
    0x2cb2, 0x2cca, 0x2ce2, 0x2cfa, 0x2cfa, 0x2cfa, 0x2cfa, 0x2cfa, 0x2cfa, 0x2d12, 0x2d12, 0x2d2a,
    0x2d42, 0x2d42, 0x2d42, 0x2d42, 0x2d5a, 0x2d72, 0x2d72, 0x2d72, 0x2d72, 0x2d8a, 0x2da2, 0x2dba,
    0x2dd2, 0x2dea, 0x2e02, 0x2e02, 0x2e02, 0x2e1a, 0x2e32, 0x2e32, 0x2e52, 0x2e6a, 0x2e6a, 0x2e6a,
];

pub(super) const DOOR_TYPE_SRC_RIGHT: [u16; 47] = [
    0x2e6a, 0x2e82, 0x2e82, 0x2e9a, 0x2e9a, 0x2e9a, 0x2e9a, 0x2e9a, 0x2e9a, 0x2eb2, 0x2eb2, 0x2eb2,
    0x2eb2, 0x2eca, 0x2ee2, 0x2efa, 0x2efa, 0x2efa, 0x2efa, 0x2efa, 0x2efa, 0x2f12, 0x2f12, 0x2f2a,
    0x2f42, 0x2f42, 0x2f42, 0x2f42, 0x2f5a, 0x2f72, 0x2f72, 0x2f72, 0x2f72, 0x2f8a, 0x2fa2, 0x2fba,
    0x2fd2, 0x2fea, 0x3002, 0x3002, 0x3002, 0x301a, 0x3032, 0x3032, 0x3052, 0x306a, 0x306a,
];
pub(super) const DOOR_TYPE_REMAP: [u8; 40] = [
    0, 2, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 80, 0, 80, 80, 96, 98, 100, 102, 82, 90, 80, 82, 84, 86,
    0, 80, 80, 0, 0, 0, 64, 88, 88, 0, 88, 88, 0, 0,
];
pub(super) type DungPalInfo = [u8; 4];

pub(super) const DUNG_PAL_INFOS: [DungPalInfo; 41] = [
    [0, 0, 3, 1],
    [2, 0, 3, 1],
    [4, 0, 10, 1],
    [6, 0, 1, 7],
    [10, 2, 2, 7],
    [4, 4, 3, 10],
    [12, 5, 8, 20],
    [14, 0, 3, 10],
    [2, 0, 15, 20],
    [10, 2, 0, 7],
    [2, 0, 15, 12],
    [6, 0, 6, 7],
    [0, 0, 14, 18],
    [18, 5, 5, 11],
    [18, 0, 2, 12],
    [16, 5, 10, 7],
    [16, 0, 16, 12],
    [22, 7, 2, 7],
    [22, 0, 7, 15],
    [8, 0, 4, 12],
    [8, 0, 4, 9],
    [4, 0, 3, 1],
    [20, 0, 4, 4],
    [20, 0, 20, 12],
    [24, 5, 7, 11],
    [24, 6, 16, 12],
    [26, 5, 8, 20],
    [26, 2, 0, 7],
    [6, 0, 3, 10],
    [28, 0, 3, 1],
    [30, 0, 11, 17],
    [4, 0, 11, 17],
    [14, 0, 0, 2],
    [32, 8, 19, 13],
    [10, 0, 3, 10],
    [20, 0, 4, 4],
    [26, 2, 2, 7],
    [26, 10, 0, 0],
    [0, 0, 3, 2],
    [14, 0, 3, 7],
    [26, 5, 5, 11],
];

// ---------------------------------------------------------------------------
// Promoted dungeon method-local tables. Names retain the owning helper so
// generic C table names stay readable at callsites.
// ---------------------------------------------------------------------------

pub(super) const DUNGEON_RESET_TORCH_BACKGROUND_AND_PLAYER_SPIRAL_BG_PROPERTIES: [i8; 8] =
    [0, 1, 1, -1, 1, 1, 1, 1];

pub(super) const DUNGEON_RESET_TORCH_BACKGROUND_AND_PLAYER_INNER_FEATURES0_TURN_WHILE_DASHING: u32 =
    4;

pub(super) const MODULE07_0_B_DRAIN_SWAMP_POOL_SWAMP_DRAIN_WINDOW_RADIUS_DELTAS: [i8; 16] =
    [-1, -1, -1, 1, -1, -1, -1, 1, -1, -1, -1, 1, -1, -1, -1, 1];

pub(super) const MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_FINAL_RADIUS_DELTAS: [i8; 4] =
    [1, 1, 1, -1];

pub(super) const MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_WINDOW_RIGHT_DELTAS: [i8; 4] =
    [1, 2, 1, -1];

pub(super) const MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_WINDOW_LEFT_DELTAS: [i8; 4] =
    [1, -1, 1, -1];

pub(super) const DUNGEON_LOAD_HEADER_ADJUSTMENTS: [i16; 2] = [256, -256];

pub(super) const MOVING_WALL_SIZE_TABLE0: [u16; 4] = [5, 7, 11, 15];

pub(super) const MOVING_WALL_SIZE_TABLE1: [u16; 4] = [8, 16, 24, 32];

pub(super) const DUNGEON_CHEST_OPEN_MASKS: [u16; 6] = [0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000];

pub(super) const DUNGEON_QUADRANT_VISITING_FLAGS: [u16; 16] = [
    8, 4, 2, 1, 0x0c, 0x0c, 3, 3, 0x0a, 5, 0x0a, 5, 0x0f, 0x0f, 0x0f, 0x0f,
];

pub(super) const DUNGEON_CHECK_ADJACENT_ROOMS_FOR_OPEN_DOORS_LOOKUP_TABLE: [u16; 24] = [
    0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x61, 0x71, 0x81, 0x91, 0xa1, 0xb1, 0x02, 0x12, 0x22, 0x32,
    0x42, 0x52, 0x63, 0x73, 0x83, 0x93, 0xa3, 0xb3,
];

pub(super) const DUNGEON_CHECK_ADJACENT_ROOMS_FOR_OPEN_DOORS_LOOKUP_TABLE2: [u16; 24] = [
    0x61, 0x71, 0x81, 0x91, 0xa1, 0xb1, 0x0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x63, 0x73, 0x83, 0x93,
    0xa3, 0xb3, 0x02, 0x12, 0x22, 0x32, 0x42, 0x52,
];

pub(super) const FLOOD_DAM_EXPAND_WATERGATE_SRCS1: [usize; 4] = [0x12f8, 0x1348, 0x1398, 0x13e8];

pub(super) const ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_X_BASE_OFFSETS: [u16; 4] = [0, 256, 0, 256];

pub(super) const ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_Y_BASE_OFFSETS: [u16; 4] = [0, 0, 256, 256];

pub(super) const ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_ADJUSTMENTS: [i16; 4] = [52, -2, 56, 6];

pub(super) const ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_BASELINES: [i16; 4] = [64, 64, 82, -176];

pub(super) const ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_CLAMPS: [u16; 4] = [128, 384, 160, 160];

pub(super) const DUNGEON_TRANSITION_ADJUST_CAMERA_X_UP_DOWN_SCROLL_VALUES: [u16; 4] =
    [0, 256, 256, 0];

pub(super) const DUNGEON_TRANSITION_ADJUST_CAMERA_Y_UP_DOWN_SCROLL_VALUES: [u16; 4] =
    [0, 272, 256, 16];

pub(super) const HANDLE_EDGE_TRANSITION_ADJUST_CAMERA_BOUNDARIES_CAMERA_X_BOUNDS: [u16; 4] =
    [127, 383, 127, 383];

pub(super) const HANDLE_EDGE_TRANSITION_ADJUST_CAMERA_BOUNDARIES_CAMERA_Y_BOUNDS: [u16; 4] =
    [120, 376, 136, 392];

pub(super) const DUNGEON_PREP_SPRITE_INDUCED_DMA_PREP_SPRITE_INDUCED_DMA_SRCS: [usize; 10] = [
    0x0e0, 0xade, 0x5aa, 0x198, 0x210, 0x218, 0x1f3a, 0xeaa, 0xeb2, 0x140,
];

pub(super) const PUSH_BLOCK_APPLY_VELOCITY_PUSHED_BLOCK_DIR_MASK: [u8; 4] =
    [0x08, 0x04, 0x02, 0x01];

pub(super) const PUSH_BLOCK_APPLY_VELOCITY_PUSH_BLOCK_X_RECOIL_BY_DIRECTION: [u8; 4] =
    [0x00, 0x00, 0xe0, 0x20];

pub(super) const PUSH_BLOCK_APPLY_VELOCITY_PUSH_BLOCK_Y_RECOIL_BY_DIRECTION: [u8; 4] =
    [0xe0, 0x20, 0x00, 0x00];

pub(super) const PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_A: [u16; 4] = [0, 0, 8, 8];

pub(super) const PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_B: [u16; 4] = [15, 15, 23, 23];

pub(super) const PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_C: [u16; 4] = [0, 0, 0, 0];

pub(super) const PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_D: [u16; 4] = [15, 15, 15, 15];

pub(super) const PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_E: [u16; 4] = [8, 24, 0, 16];

pub(super) const PUSH_BLOCK_HANDLE_COLLISION_PUSH_BLOCK_F: [u16; 4] = [15, 0, 15, 0];

pub(super) const DUNG_TAG_ROUTINE_BLAST_WALL_STUFF_BLAST_WALL_MESSAGE_DIRECTION_BY_QUADRANT: [u8;
    5] = [4, 6, 0, 0, 2];

pub(super) const DUNG_TAG_ROUTINE_BLAST_WALL_STUFF_BLAST_WALL_DOOR_TILEMAP_OFFSETS: [u16; 5] =
    [0, 0x0a, 0, 0, 0x0280];

pub(super) const ROOM_TAG_GET_HEART_FOR_PRIZE_BOSS_FINISHED_FALLING_ITEM: [u8; 13] =
    [0, 0, 1, 2, 0, 6, 6, 6, 6, 6, 3, 6, 6];

pub(super) const CLEAR_AND_STRIPE_EXPLODING_WALL_BLAST_WALL_STRIPE_ROW_ADVANCES: [u16; 16] = [
    0x0004, 0x0008, 0x000c, 0x0010, 0x0014, 0x0018, 0x001c, 0x0020, 0x0100, 0x0200, 0x0300, 0x0400,
    0x0500, 0x0600, 0x0700, 0x0800,
];

pub(super) const DUNGEON_LOAD_SINGLE_DOOR_ATTRIBUTE_TILE_ATTRS_BY_DOOR: [u16; 40] = [
    0x8080, 0x8484, 0x0000, 0x0101, 0x8484, 0x8e8e, 0x0000, 0x0000, 0x8888, 0x8e8e, 0x8080, 0x8080,
    0x8282, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8282, 0x8e8e, 0x8080, 0x8282,
    0x8080, 0x8080, 0x8080, 0x8282, 0x8282, 0x8080, 0x8080, 0x8080, 0x8484, 0x8484, 0x8686, 0x8888,
    0x8686, 0x8686, 0x8080, 0x8080,
];

pub(super) const DUNGEON_DETECT_STAIRCASE_BUGGY_LOOKUP: [i8; 8] = [7, 24, 8, 8, 0, 0, -1, 17];

pub(super) const ROOM_TAG_MOVING_WALL_EAST_MOVING_WALL_EAST_TARGET_OFFSETS: [u16; 8] = [
    (-63i16) as u16,
    (-127i16) as u16,
    (-191i16) as u16,
    (-255i16) as u16,
    (-71i16) as u16,
    (-135i16) as u16,
    (-199i16) as u16,
    (-263i16) as u16,
];

pub(super) const ROOM_TAG_MOVING_WALL_WEST_MOVING_WALL_WEST_TARGET_OFFSETS: [u16; 8] =
    [0x42, 0x82, 0xc2, 0x102, 0x4a, 0x8a, 0xca, 0x10a];

pub(super) const MODULE07_1_A_ROOM_DRAW_OPEN_TRIFORCE_DOOR_BOUNCE_OPEN_GANON_DOOR_TILE_SOURCES:
    [u16; 4] = [0x2556, 0x2596, 0x25d6, 0x2616];

pub(super) const DUNGEON_HANDLE_EDGE_TRANSITION_MOVEMENT_LIMIT_DIRECTION_ON_ONE_AXIS: [u8; 4] =
    [0x03, 0x03, 0x0c, 0x0c];

pub(super) const MODULE_PRE_DUNGEON_LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];

pub(super) const CRYSTAL_CUTSCENE_INITIALIZE_CRYSTAL_MAIDEN_PAL: [u16; 8] =
    [0, 0x3821, 0x4463, 0x54a5, 0x5ce7, 0x6d29, 0x79ad, 0x7e10];

pub(super) const DUNGEON_PUSH_BLOCK_HANDLER_PUSH_BLOCK_MOVE_DISTANCES: [i16; 4] =
    [-0x100, 0x100, -0x04, 0x04];

pub(super) const DUNGEON_PROCESS_TORCHES_AND_DOORS_LINK_X_OFFSETS: [i32; 4] = [0, 0, -1, 17];

pub(super) const DUNGEON_PROCESS_TORCHES_AND_DOORS_LINK_Y_OFFSETS: [i32; 4] = [7, 24, 8, 8];

pub(super) const DUNGEON_PROCESS_TORCHES_AND_DOORS_LINK_POSITION_OFFSETS: [usize; 4] =
    [0x0002, 0x0002, 0x0080, 0x0080];

pub(super) const DUNGEON_PROCESS_TORCHES_AND_DOORS_OPEN_DOOR_PANNING: [u8; 4] =
    [0x00, 0x00, 0x80, 0x40];

pub(super) const DUNGEON_PROCESS_TORCHES_AND_DOORS_SOURCE_TILES1: [u16; 4] =
    [0x07ea, 0x080a, 0x080a, 0x082a];
