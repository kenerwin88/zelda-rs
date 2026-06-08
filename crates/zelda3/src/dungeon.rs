// Methods ported from zelda3/src/dungeon.c and included inside ZeldaState.

use super::*;
use crate::types::Point16U;
use crate::zelda_rtl::misc::DUNG_ANIMATED_TILES;
use crate::zelda_rtl::sprite::SpriteSpawnInfo;

const DUNG_REPLACEMENT_TILE_SRC_POS_X2: usize = 0x47c;
const DUNG_NUM_STAIRS_1: usize = 0x49a;
const DUNG_NUM_STAIRS_2: usize = 0x49c;
const DUNG_NUM_STAIRS_WET: usize = 0x49e;
const DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS: usize = 0x440;
const DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER: usize = 0x442;
const DUNG_NUM_ACTIVATED_WATER_LADDERS: usize = 0x444;
// NES_Ver2: UDSCKP6, "water-side STEP (kirikae)".
const WATER_SIDE_STEP_SWITCH: usize = 0x448;
const KIND_OF_IN_ROOM_STAIRCASE_DUNGEON: usize = 0x44a;
const DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER: usize = 0x4ae;
const DUNG_NUM_STAR_SHAPED_SWITCHES: usize = 0x432;
const DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS: usize = 0x438;
const DUNG_NUM_INROOM_UPNORTH_STAIRS: usize = 0x43c;
const DUNG_NUM_INROOM_SOUTHDOWN_STAIRS: usize = 0x43e;
const DUNG_NUM_WATER_LADDERS: usize = 0x446;
const DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS: usize = 0x4a6;
const DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS: usize = 0x4a8;
const DUNG_STAIRS_TABLE_2: usize = 0x6ec;
const DUNG_STAIRS_TABLE_1: usize = 0x6b8;
const STAR_SHAPED_SWITCHES_TILE: usize = 0x6a0;
const DUNG_FLOOR_MOVE_FLAGS: usize = 0x41a;
const DUNG_FLOOR_Y_VEL_DUNGEON: usize = 0x310;
const DUNG_FLOOR_X_VEL: usize = 0x312;
const DUNG_FLOOR_X_OFFS: usize = 0x422;
// NES_Ver2: RSXYCKF, "reset x,y check flag".
const RESET_XY_CHECK_FLAGS: usize = 0x00fc;
// NES_Ver2: B1CWPT/BG1MPT, moving-wall write point and dot pointer.
const MOVING_WALL_WRITE_POINT: usize = 0x42a;
const MOVING_WALL_DOT_POINTER: usize = 0x41e;
const MOVING_WALL_ARR1: usize = 0xc880;
const INVISIBLE_DOOR_DIR_AND_INDEX_X2: usize = 0x436;
const TRANSITION_COUNTER: usize = 0x0126;
const DUNG_BLASTWALL_FLAG_X: usize = 0x0452;
const DUNG_BLASTWALL_FLAG_Y: usize = 0x0453;
const DUNG_FLAG_TRAPDOORS_DOWN: usize = 0x468;
const DUNG_FLAG_STATECHANGE_WATERPUZZLE: usize = 0x642;
// NES_Ver2: WGTPNT, water-gate pointer.
const WATERGATE_POINTER: usize = 0x0470;
const WATERGATE_POS: usize = 0x0472;
const WATERGATE_SPOTLIGHT_Y_UPPER: usize = 0x0678;
// NES_Ver2 WRWP*/OYK*/WIN* water-window HDMA work RAM.
const WATER_HDMA_WINDOW_X_DUNGEON: usize = 0x0680;
const WATER_HDMA_WINDOW_Y_DUNGEON: usize = 0x0682;
const WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON: usize = 0x0684;
const WATER_HDMA_WINDOW_X_RADIUS_DUNGEON: usize = 0x0686;
const WATER_HDMA_WINDOW_Y_TARGET_DUNGEON: usize = 0x0688;
const WATER_HDMA_WINDOW_Y_RADIUS_ALT_DUNGEON: usize = 0x068a;
const TURN_ON_OFF_WATER_CTR: usize = 0x0424;
const MINIGAME_CREDITS: usize = 0x04c4;
const DUNG_TRANSITION_LANDING_CLASS: usize = 0x004e;
const DUNG_CUR_DOOR_POS_DUNGEON: usize = 0x068e;
const DOOR_ANIMATION_STEP_INDICATOR_DUNGEON: usize = 0x0690;
const DUNG_WHICH_KEY_X2_DUNGEON: usize = 0x0694;
const DUNG_DOOR_SWITCH_TRIGGERED: usize = 0x0430;
const DUNG_CUR_QUADRANT_UPLOAD: usize = 0x045c;
// NES_Ver2: CWLFLG/CWLPNT, crush-wall progress and doubled door index.
const CRUSH_WALL_PROGRESS_DUNGEON: usize = 0x0454;
const CRUSH_WALL_DOOR_INDEX_X2_DUNGEON: usize = 0x0456;
const DUNG_DOOR_BARRIER_OR_SWITCH_FLAG: usize = 0x045e;
const BLOCK_TRAP_CHECK_FLAG: usize = 0x0466;
const DUNG_REPLACEMENT_TILE_DST_POS_X2: usize = 0x04b6;
const MOVING_WALL_TORCH_BLINK_PHASE: usize = 0x04bc;
const MOVING_WALL_TORCH_UPDATE_FLAG: usize = 0x04c2;
const DUNG_FLAG_SOMARIA_BLOCK_SWITCH: usize = 0x0646;
const DUNG_INTER_STAIRCASES: usize = 0x06b0;
const STAIRCASE_TILEMAP_POS_X2: usize = 0x048c;
const DUNG_NUM_TOGGLE_FLOOR: usize = 0x44e;
const DUNG_NUM_TOGGLE_PALACE: usize = 0x450;
const DUNG_TOGGLE_FLOOR_POS: usize = 0x6c0;
const DUNG_TOGGLE_PALACE_POS: usize = 0x6d0;
const ADJACENT_DOORS_FLAGS: usize = 0x1100;
const ADJACENT_DOORS: usize = 0x1110;
// NES_Ver2: WRDADR, width road address.
const DUNG_WIDTH_ROAD_ADDRESS: usize = 0x4b0;
const DUNG_BG1_ATTR_TABLE: usize = 0x13000;
const DUNG_BG1: usize = 0x4000;
const DUNG_INDEX_X3: usize = 0x110;
const PUSHEDBLOCKS_MAYBE_TIMEOUT: usize = 0x02c4;
const PUSHEDBLOCK_FACING: usize = 0x05f8;
const PUSH_BLOCK_DIRECTION_DUNGEON: usize = 0x0474;
const MOVABLE_BLOCK_DATAS: usize = 0x0f940;
const SPRITE_Y_RECOIL_DUNGEON: usize = 0x0f30;
const DUNG_HDR_BG2_PROPERTIES_BACKUP: usize = 0xc208;
const WHICH_STAIRCASE_INDEX: usize = 0x462;
// NES_Ver2: SPMVCT, step/staircase move counter.
const STAIRCASE_MOVE_COUNTER: usize = 0x464;
const CUR_STAIRCASE_PLANE: usize = 0x48a;
const DUNG_HDR_STAIRCASE_PLANE: usize = 0x63d;
const STAIRCASE_LOWER_LEVEL_STATUS: usize = 0x492;
const COUNTDOWN_TIMER_FOR_STAIRCASES: usize = 0x378;
const DEATHS_PER_PALACE: usize = 0xf3e7;
const DEATH_SAVE_COUNTER: usize = 0xf403;
const DOOR_DEBRIS_DIRECTION_DUNGEON: usize = 0x073c;
const SAVE_OW_EVENT_INFO_DUNGEON: usize = 0x0f280;
const FLAG_WHICH_MUSIC_TYPE_DUNGEON: usize = 0x136;
const MESSAGING_BUF_DUNGEON: usize = 0x10000;
const DUNG_TORCH_TIMERS_DUNGEON: usize = 0x04f0;
const DUNG_TORCH_DATA_DUNGEON: usize = 0x0fb40;
const DUNG_MEMORIZED_TILE_ADDR: usize = 0x0f800;
const DUNG_SECRETS_UNK1_DUNGEON: usize = 0x0b9c;
const POTS_REVEALED_IN_ROOM_DUNGEON: usize = 0x0f580;
const UVRAM_DATA_DUNGEON: usize = 0x1100;
const K_FEATURES0_MISC_BUG_FIXES_DUNGEON: u32 = 4096;
const K_FEATURES0_BREAK_POTS_WITH_SWORD_DUNGEON: u32 = 32;
const BIG_KEY_DOOR_MESSAGE_TRIGGERED_DUNGEON: usize = 0x04b8;
const DUNG_LOAD_PTR: usize = 0x00b7;
const DUNG_LOAD_PTR_BANK: usize = 0x00b9;

fn parse_usize_env(value: &str) -> Option<usize> {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .and_then(|hex| usize::from_str_radix(hex, 16).ok())
        .or_else(|| value.parse::<usize>().ok())
}

fn format_optional_hex(value: Option<u8>) -> String {
    value
        .map(|value| format!("0x{value:02x}"))
        .unwrap_or_else(|| "OOB".to_string())
}

struct EntranceAssetSet {
    rooms: usize,
    relative_coords: usize,
    scroll_x: usize,
    scroll_y: usize,
    player_x: usize,
    player_y: usize,
    camera_x: usize,
    camera_y: usize,
    blockset: usize,
    floor: usize,
    palace: usize,
    doorway_orientation: usize,
    starting_bg: usize,
    quadrant1: usize,
    quadrant2: usize,
    door_settings: usize,
}

const ENTRANCE_DATA_ASSETS: EntranceAssetSet = EntranceAssetSet {
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

const STARTING_POINT_ASSETS: EntranceAssetSet = EntranceAssetSet {
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

const K_DUNGEON_QUERY_IF_TILE_LIFTABLE_X: [i16; 4] = [7, 7, -3, 16];
const K_DUNGEON_QUERY_IF_TILE_LIFTABLE_Y: [i16; 4] = [3, 24, 14, 14];
const K_DUNGEON_QUERY_IF_TILE_LIFTABLE_RV: [u16; 16] = [
    0x5252, 0x5050, 0x5454, 0, 0x2323, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const K_DUNGEON_MINIGAME_CHEST_PRIZES1: [u8; 8] = [0x40, 0x41, 0x34, 0x42, 0x43, 0x44, 0x27, 0x17];
const K_DUNGEON_RUPEE_CHEST_MINIGAME_PRIZES: [u8; 32] = [
    0x47, 0x34, 0x46, 0x34, 0x46, 0x46, 0x34, 0x47, 0x46, 0x47, 0x34, 0x46, 0x47, 0x34, 0x46, 0x47,
    0x34, 0x47, 0x41, 0x47, 0x41, 0x41, 0x47, 0x34, 0x41, 0x34, 0x47, 0x41, 0x34, 0x47, 0x41, 0x34,
];

const DOOR_TYPE_REGULAR2: u8 = 2;
const DOOR_TYPE_4: u8 = 4;
const DOOR_TYPE_ENTRANCE_DOOR: u8 = 6;
const DOOR_TYPE_WATERFALL_TUNNEL: u8 = 8;

const K_BOSS_ROOMS_DUNGEON: [u16; 12] = [200, 51, 7, 32, 6, 90, 41, 144, 222, 164, 172, 13];
const DOOR_TYPE_ENTRANCE_LARGE: u8 = 10;
const DOOR_TYPE_ENTRANCE_LARGE2: u8 = 12;
const DOOR_TYPE_ENTRANCE_CAVE: u8 = 14;
const DOOR_TYPE_ENTRANCE_CAVE2: u8 = 16;
const DOOR_TYPE_PLAYER_BG_CHANGE: u8 = 22;
const DOOR_TYPE_INVISIBLE_DOOR: u8 = 26;
const DOOR_TYPE_SMALL_KEY_DOOR: u8 = 0x1c;
const DOOR_TYPE_1E: u8 = 0x1e;
const DOOR_TYPE_STAIR_MASK_LOCKED0: u8 = 32;
const DOOR_TYPE_STAIR_MASK_LOCKED2: u8 = 36;
const K_DUNGEON_EXIT_FROM: [u8; 12] = [200, 51, 7, 32, 6, 90, 41, 144, 222, 164, 172, 13];
const K_DUNGEON_EXIT_TO: [u8; 12] = [201, 99, 119, 32, 40, 74, 89, 152, 14, 214, 219, 13];
const K_WATERGATE_LAYOUT: [u8; 17] = [
    0x1b, 0xa1, 0xc9, 0x51, 0xa1, 0xc9, 0x92, 0xa1, 0xc9, 0xa1, 0x33, 0xc9, 0xa1, 0x72, 0xc9, 0xff,
    0xff,
];
const DOOR_TYPE_STAIR_MASK_LOCKED3: u8 = 38;
const DOOR_TYPE_BREAKABLE_WALL: u8 = 0x28;
const DOOR_TYPE_LG_EXPLOSION: u8 = 48;
const DOOR_TYPE_REGULAR_DOOR33: u8 = 64;
const DOOR_TYPE_SHUTTER: u8 = 68;
const DOOR_TYPE_WARP_ROOM_DOOR: u8 = 70;
const DOOR_TYPE_SHUTTER_TRAP_UR: u8 = 72;
const DOOR_TYPE_SHUTTER_TRAP_DL: u8 = 74;
const K_DUNGEON_CRYSTAL_PENDANT_BIT: [u8; 13] = [0, 0, 4, 2, 0, 16, 2, 1, 64, 4, 1, 32, 8];
const K_STAIRCASE_TAB2: [i8; 20] = [
    12, 32, 48, 56, 72, -44, -40, -64, -64, -88, 12, 24, 40, 48, 64, -28, -40, -56, -64, -80,
];
const K_SPIRAL_TAB1: [i8; 8] = [0, 1, 1, -1, 1, 1, 1, 1];
const K_STAIRCASE_TAB3: [i8; 4] = [4, -4, 4, -4];
const K_STAIRCASE_TAB4: [u8; 4] = [52, 52, 59, 58];
const K_STAIRCASE_TAB5: [i8; 4] = [32, -64, 32, -32];
const K_TELEPORT_PIT_LEVEL1: [u8; 3] = [0, 1, 1];
const K_TELEPORT_PIT_LEVEL2: [u8; 3] = [0, 0, 1];
const K_SPIRAL_STAIRCASE_X: [i8; 4] = [-28, -28, 24, 24];
const K_SPIRAL_STAIRCASE_Y: [i8; 4] = [16, -10, -10, -32];
const K_DOOR_ANIM_UP_SRC: [u16; 5] = [0x306a, 0x306a, 0x3082, 0x309a, 0x30b2];
const K_DOOR_ANIM_DOWN_SRC: [u16; 5] = [0x30b2, 0x30ca, 0x30e2, 0x30fa, 0x3112];
const K_DOOR_ANIM_LEFT_SRC: [u16; 5] = [0x3112, 0x312a, 0x3142, 0x315a, 0x3172];
const K_DOOR_ANIM_RIGHT_SRC: [u16; 5] = [0x3172, 0x318a, 0x31a2, 0x31ba, 0x31d2];
const K_DOOR_BLAST_WALL_UP_DSTS: [u16; 6] = [0x0d8a, 0x0daa, 0x0dca, 0x02b6, 0x0ab6, 0x12b6];
const K_LAYOUT_QUADRANT_FLAGS: [u8; 32] = [
    0x0f, 0x0f, 0x0f, 0x0f, 0x0b, 0x0b, 7, 7, 0x0f, 0x0b, 0x0f, 7, 0x0b, 0x0f, 7, 0x0f, 0x0e, 0x0d,
    0x0e, 0x0d, 0x0f, 0x0f, 0x0e, 0x0d, 0x0e, 0x0d, 0x0f, 0x0f, 0x0a, 9, 6, 5,
];

const DOOR_POSITION_LEFT: [u16; 12] = [
    0x784, 0xf84, 0x1784, 0x78a, 0xf8a, 0x178a, 0x7c4, 0xfc4, 0x17c4, 0x7ca, 0xfca, 0x17ca,
];
const DOOR_POSITION_RIGHT: [u16; 12] = [
    0x7b4, 0xfb4, 0x17b4, 0x7ae, 0xfae, 0x17ae, 0x7f4, 0xff4, 0x17f4, 0x7ee, 0xfee, 0x17ee,
];
const DOOR_TYPE_SRC_LEFT: [u16; 48] = [
    0x2c6a, 0x2c82, 0x2c82, 0x2c9a, 0x2c9a, 0x2c9a, 0x2c9a, 0x2c9a, 0x2c9a, 0x2cb2, 0x2cb2, 0x2cb2,
    0x2cb2, 0x2cca, 0x2ce2, 0x2cfa, 0x2cfa, 0x2cfa, 0x2cfa, 0x2cfa, 0x2cfa, 0x2d12, 0x2d12, 0x2d2a,
    0x2d42, 0x2d42, 0x2d42, 0x2d42, 0x2d5a, 0x2d72, 0x2d72, 0x2d72, 0x2d72, 0x2d8a, 0x2da2, 0x2dba,
    0x2dd2, 0x2dea, 0x2e02, 0x2e02, 0x2e02, 0x2e1a, 0x2e32, 0x2e32, 0x2e52, 0x2e6a, 0x2e6a, 0x2e6a,
];

pub(super) struct RoomBounds {
    base: usize,
}

const ROOM_BOUNDS_Y_REF: RoomBounds = RoomBounds {
    base: ROOM_BOUNDS_Y,
};
const ROOM_BOUNDS_X_REF: RoomBounds = RoomBounds {
    base: ROOM_BOUNDS_X,
};
const DOOR_TYPE_SRC_RIGHT: [u16; 47] = [
    0x2e6a, 0x2e82, 0x2e82, 0x2e9a, 0x2e9a, 0x2e9a, 0x2e9a, 0x2e9a, 0x2e9a, 0x2eb2, 0x2eb2, 0x2eb2,
    0x2eb2, 0x2eca, 0x2ee2, 0x2efa, 0x2efa, 0x2efa, 0x2efa, 0x2efa, 0x2efa, 0x2f12, 0x2f12, 0x2f2a,
    0x2f42, 0x2f42, 0x2f42, 0x2f42, 0x2f5a, 0x2f72, 0x2f72, 0x2f72, 0x2f72, 0x2f8a, 0x2fa2, 0x2fba,
    0x2fd2, 0x2fea, 0x3002, 0x3002, 0x3002, 0x301a, 0x3032, 0x3032, 0x3052, 0x306a, 0x306a,
];
const DOOR_TYPE_REMAP: [u8; 40] = [
    0, 2, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 80, 0, 80, 80, 96, 98, 100, 102, 82, 90, 80, 82, 84, 86,
    0, 80, 80, 0, 0, 0, 64, 88, 88, 0, 88, 88, 0, 0,
];
type DungPalInfo = [u8; 4];

const DUNG_PAL_INFOS: [DungPalInfo; 41] = [
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

impl ZeldaState {
    pub(super) fn Dungeon_LoadAndDrawEntranceRoom(&mut self, room: u8) {
        self.ram[WHICH_ENTRANCE] = room;
        self.Dungeon_LoadEntrance();
        self.ram[DUNG_NUM_LIT_TORCHES] = 0;
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 0;
        self.Dungeon_LoadAndDrawRoom();
        self.Dungeon_ResetTorchBackgroundAndPlayer();
    }

    pub(super) fn Dungeon_LoadAndDrawRoom(&mut self) {
        let hdma = self.ram[HDMAEN_COPY];
        self.ram[HDMAEN_COPY] = 0;
        self.Dungeon_LoadRoom();
        self.ram[OVERWORLD_SCREEN_TRANSITION] = 0;
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.Dungeon_UploadRoomQuadrants();
        self.ram[HDMAEN_COPY] = hdma;
        self.ram[NMI_SUBROUTINE_INDEX] = 0;
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.frame_control_view_mut().set_subsubmodule(0);
    }

    pub(super) fn Dungeon_LoadEntrance(&mut self) {
        self.ram[PLAYER_IS_INDOORS] = 1;
        if self.ram[GAME_OVER_CHECK_FLAG] != 0 {
            self.ram[GAME_OVER_CHECK_FLAG] = 0;
        } else {
            copy_le_u16(&mut self.ram, 0xc140, 0x40a);
            copy_le_u16(&mut self.ram, 0xc142, TM_COPY);
            copy_le_u16(&mut self.ram, 0xc144, BG2VOFS_COPY2);
            copy_le_u16(&mut self.ram, 0xc146, BG2HOFS_COPY2);
            copy_le_u16(&mut self.ram, 0xc148, LINK_Y_COORD);
            copy_le_u16(&mut self.ram, 0xc14a, LINK_X_COORD);
            copy_le_u16(&mut self.ram, 0xc14c, OVERWORLD_SCREEN_INDEX);
            copy_le_u16(&mut self.ram, 0xc14e, 0x84);
            copy_le_u16(&mut self.ram, 0xc150, CAMERA_Y_COORD_SCROLL_LOW);
            copy_le_u16(&mut self.ram, 0xc152, CAMERA_X_COORD_SCROLL_LOW);
            self.ram.copy_within(0x600..0x608, 0xc154);
            copy_le_u16(&mut self.ram, 0xc15c, UP_DOWN_SCROLL_TARGET);
            copy_le_u16(&mut self.ram, 0xc15e, UP_DOWN_SCROLL_TARGET_END);
            copy_le_u16(&mut self.ram, 0xc160, LEFT_RIGHT_SCROLL_TARGET);
            copy_le_u16(&mut self.ram, 0xc162, LEFT_RIGHT_SCROLL_TARGET_END);
            copy_le_u16(&mut self.ram, 0xc16a, 0x624);
            copy_le_u16(&mut self.ram, 0xc16c, 0x626);
            copy_le_u16(&mut self.ram, 0xc16e, 0x628);
            copy_le_u16(&mut self.ram, 0xc170, 0x62a);
            self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX] = self.ram[OVERWORLD_TILE_THEME_INDEX];
            self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX + 1] = self.ram[MAIN_TILE_THEME_INDEX];
            self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX + 2] = self.ram[AUX_TILE_THEME_INDEX];
            self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX + 3] = self.ram[SPRITE_GRAPHICS_INDEX];
            self.ram[OVERWORLD_SCREEN_INDEX] = 0;
            self.ram[OVERWORLD_SCREEN_INDEX + 1] = 0;
            self.ram[OVERLAY_INDEX] = 0;
            self.ram[OVERLAY_INDEX + 1] = 0;
        }
        write_le_u16(&mut self.ram, BG1_Y_OFFSET, 0);
        write_le_u16(&mut self.ram, BG1_X_OFFSET, 0);
        write_le_u16(&mut self.ram, GAME_OVER_CHECK_FLAG, 0);

        if read_le_u16(&self.ram, FOLLOWER_INDICATOR) == 4
            || read_le_u16(&self.ram, RESTART_CHECK_FLAG) != 0
        {
            let i = self.ram[WHICH_STARTING_POINT] as usize;
            let entrance = self.asset_u8(44, i);
            write_le_u16(&mut self.ram, WHICH_ENTRANCE, entrance as u16);
            self.dungeon_load_entrance_fields(i, &STARTING_POINT_ASSETS);
            self.ram[LINK_DIRECTION_FACING] = 2;
            self.ram[IS_STANDING_IN_DOORWAY] = 0;
            self.ram[QUEUED_MUSIC_CONTROL] = self.asset_u8(45, i);
            if i == 0 && self.ram[SRAM_PROGRESS_INDICATOR] == 0 {
                self.ram[QUEUED_MUSIC_CONTROL] = 0xff;
            }
            self.ram[RESTART_CHECK_FLAG] = 0;
        } else {
            let i = self.ram[WHICH_ENTRANCE] as usize;
            let room = self.dungeon_load_entrance_fields(i, &ENTRANCE_DATA_ASSETS);
            write_le_u16(&mut self.ram, BIG_ROCK_STARTING_ADDRESS, 0);
            self.ram[LINK_DIRECTION_FACING] = if i == 0 || i == 0x43 { 2 } else { 0 };
            self.ram[IS_STANDING_IN_DOORWAY] =
                self.asset_u8(ENTRANCE_DATA_ASSETS.doorway_orientation, i);
            self.ram[QUEUED_MUSIC_CONTROL] = self.zelda_get_entrance_music_track(i as i32);
            if self.ram[QUEUED_MUSIC_CONTROL] == 3 && self.ram[SRAM_PROGRESS_INDICATOR] >= 2 {
                self.ram[QUEUED_MUSIC_CONTROL] = 18;
            }
            if room >= 0x100 {
                self.ram[DUNG_CUR_FLOOR] = 0;
            }
        }

        self.ram[PLAYER_OAM_Y_OFFSET] = 0x80;
        self.ram[PLAYER_OAM_X_OFFSET] = 0x80;
        self.ram[LINK_DIRECTION_MASK_A] = 0x0f;
        self.ram[LINK_DIRECTION_MASK_B] = 0x0f;
        self.ram[LINK_Z_COORD] = 0xff;
        self.ram[LINK_ACTUAL_VEL_Z] = 0xff;

        self.ram[MOVING_WALL_TORCH_BLINK_PHASE] = 0;
        write_le_u16(&mut self.ram, ORANGE_BLUE_BARRIER_STATE, 0);
        let movable_init = self
            .asset_raw(53)
            .expect("missing movable block init asset")
            .to_vec();
        self.copy_to_ram(MOVABLE_BLOCK_DATAS, &movable_init);
        let torch_init = self
            .asset_raw(54)
            .expect("missing torch init asset")
            .to_vec();
        self.copy_to_ram(DUNG_TORCH_DATA_DUNGEON, &torch_init);
        self.ram[MOVABLE_BLOCK_DATAS + 99 * 4..MOVABLE_BLOCK_DATAS + 99 * 4 + 116]
            .copy_from_slice(&torch_init[..116]);
        let torch_junk = self
            .asset_raw(55)
            .expect("missing torch junk asset")
            .to_vec();
        self.ram[DUNG_TORCH_DATA_DUNGEON + 144 * 2
            ..DUNG_TORCH_DATA_DUNGEON + 144 * 2 + torch_junk.len()]
            .copy_from_slice(&torch_junk);
        self.fill_ram(POTS_REVEALED_IN_ROOM_DUNGEON, 0x280, 0);
        self.fill_ram(DUNG_MEMORIZED_TILE_ADDR, 0x100, 0);
    }

    fn dungeon_load_entrance_fields(&mut self, i: usize, assets: &EntranceAssetSet) -> u16 {
        let room = self.asset_u16(assets.rooms, i);
        write_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX, room);
        write_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX2, room);

        let scroll_y = self.asset_u16(assets.scroll_y, i);
        write_le_u16(&mut self.ram, BG1VOFS_COPY, scroll_y);
        write_le_u16(&mut self.ram, BG2VOFS_COPY, scroll_y);
        write_le_u16(&mut self.ram, BG1VOFS_COPY2, scroll_y);
        write_le_u16(&mut self.ram, BG2VOFS_COPY2, scroll_y);

        let scroll_x = self.asset_u16(assets.scroll_x, i);
        write_le_u16(&mut self.ram, BG1HOFS_COPY, scroll_x);
        write_le_u16(&mut self.ram, BG2HOFS_COPY, scroll_x);
        write_le_u16(&mut self.ram, BG1HOFS_COPY2, scroll_x);
        write_le_u16(&mut self.ram, BG2HOFS_COPY2, scroll_x);

        if read_le_u16(&self.ram, SRAM_PROGRESS_INDICATOR) != 0 {
            let player_y = self.asset_u16(assets.player_y, i);
            let player_x = self.asset_u16(assets.player_x, i);
            self.player_state_view_mut().set_y(player_y);
            self.player_state_view_mut().set_x(player_x);
        }

        let camera_y = self.asset_u16(assets.camera_y, i);
        write_le_u16(&mut self.ram, CAMERA_Y_COORD_SCROLL_LOW, camera_y);
        write_le_u16(
            &mut self.ram,
            CAMERA_Y_COORD_SCROLL_HI,
            camera_y.wrapping_add(2),
        );
        let camera_x = self.asset_u16(assets.camera_x, i);
        write_le_u16(&mut self.ram, CAMERA_X_COORD_SCROLL_LOW, camera_x);
        write_le_u16(
            &mut self.ram,
            CAMERA_X_COORD_SCROLL_HI,
            camera_x.wrapping_add(2),
        );

        write_le_u16(&mut self.ram, TILEMAP_LOCATION_CALC_MASK, 0x01f8);
        let door_settings = self.asset_u16(assets.door_settings, i);
        write_le_u16(&mut self.ram, OW_ENTRANCE_VALUE, door_settings);
        write_le_u16(&mut self.ram, UP_DOWN_SCROLL_TARGET, 0);
        write_le_u16(&mut self.ram, UP_DOWN_SCROLL_TARGET_END, 0x0110);
        write_le_u16(&mut self.ram, LEFT_RIGHT_SCROLL_TARGET, 0);
        write_le_u16(&mut self.ram, LEFT_RIGHT_SCROLL_TARGET_END, 0x0100);

        for j in 0..4 {
            let value = (self.asset_u8(assets.relative_coords, i * 8 + j) as u16) << 8;
            let value = if j >= 2 { value | 0x10 } else { value };
            write_le_u16(&mut self.ram, ROOM_BOUNDS_Y + j * 2, value);
        }
        for j in 0..4 {
            let value = (self.asset_u8(assets.relative_coords, i * 8 + 4 + j) as u16) << 8;
            write_le_u16(&mut self.ram, ROOM_BOUNDS_X + j * 2, value);
        }

        self.ram[MAIN_TILE_THEME_INDEX] = self.asset_u8(assets.blockset, i);
        self.ram[DUNG_CUR_FLOOR] = self.asset_u8(assets.floor, i);
        self.ram[CUR_PALACE_INDEX_X2] = self.asset_u8(assets.palace, i);

        let starting_bg = self.asset_u8(assets.starting_bg, i);
        self.ram[LINK_IS_ON_LOWER_LEVEL] = starting_bg >> 4;
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = starting_bg & 0x0f;

        let quadrant1 = self.asset_u8(assets.quadrant1, i);
        self.ram[QUADRANT_FULLSIZE_X] = quadrant1 >> 4;
        self.ram[QUADRANT_FULLSIZE_Y] = quadrant1 & 0x0f;
        let quadrant2 = self.asset_u8(assets.quadrant2, i);
        self.ram[LINK_QUADRANT_X] = quadrant2 >> 4;
        self.ram[LINK_QUADRANT_Y] = quadrant2 & 0x0f;

        room
    }

    pub(super) fn Dungeon_ResetTorchBackgroundAndPlayer(&mut self) {
        const SPIRAL_BG_PROPERTIES: [i8; 8] = [0, 1, 1, -1, 1, 1, 1, 1];
        let bg_properties = self.ram[DUNG_HDR_BG2_PROPERTIES] as usize;
        let mut tm = 0x16;
        let mut ts = SPIRAL_BG_PROPERTIES[bg_properties];
        if ts < 0 {
            tm = 0x17;
            ts = 0;
        }
        if bg_properties == 2 {
            ts = 3;
        }
        self.ram[TM_COPY] = tm;
        self.ram[TS_COPY] = ts as u8;
        self.hud_restore_torch_background();
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
    }

    pub(super) fn Dungeon_ResetTorchBackgroundAndPlayerInner(&mut self) {
        self.ancilla_terminate_select_interactives(0);

        const FEATURES0_TURN_WHILE_DASHING: u32 = 4;
        if self.ram[LINK_IS_RUNNING] != 0
            && self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_TURN_WHILE_DASHING == 0
        {
            self.ram[LINK_AUXILIARY_STATE] = 0;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
            self.ram[LINK_ACTUAL_VEL_Z] = 0xff;
            self.ram[LINK_RECOIL_Z_VEL] = 0xff;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            self.ram[SWIM_ACCELERATION_MODE] = 0;
            self.ram[LINK_IS_RUNNING] = 0;
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
        }
    }

    pub(super) fn LoadOWMusicIfNeeded(&mut self) {
        if self.ram[FLAG_WHICH_MUSIC_TYPE_DUNGEON] == 0 {
            return;
        }
        self.ram[FLAG_WHICH_MUSIC_TYPE_DUNGEON] = 0;
        self.load_overworld_songs();
    }

    pub(super) fn Dungeon_LoadSongBankIfNeeded(&mut self) {
        let queued = self.ram[QUEUED_MUSIC_CONTROL];
        if queued == 0xff || queued == 0xf2 {
            return;
        }
        if queued == 3 || queued == 7 || queued == 14 {
            self.LoadOWMusicIfNeeded();
        } else {
            if self.ram[FLAG_WHICH_MUSIC_TYPE_DUNGEON] != 0 {
                return;
            }
            self.ram[FLAG_WHICH_MUSIC_TYPE_DUNGEON] = 1;
            self.load_dungeon_songs();
        }
    }

    pub(super) fn ApplyGrayscaleFixed_Incremental(&mut self) {
        let mut a = self.ram[COLDATA_COPY0] & 0x1f;
        let target = self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS];
        if a == target {
            return;
        }
        if a < target {
            a = a.wrapping_add(1);
        } else {
            a = a.wrapping_sub(1);
        }
        self.Dungeon_ApproachFixedColor_variable(a);
    }

    pub(super) fn Dungeon_ApproachFixedColor_variable(&mut self, a: u8) {
        self.ram[COLDATA_COPY0] = a | 0x20;
        self.ram[COLDATA_COPY1] = a | 0x40;
        self.ram[COLDATA_COPY2] = a | 0x80;
    }

    pub(super) fn Dungeon_DoubleApplyAndIncrementGrayscale(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.ApplyPaletteFilter_bounce();
        self.ApplyGrayscaleFixed_Incremental();
    }

    pub(super) fn Module07_0A_ChangeBrightness(&mut self) {
        self.OrientLampLightCone();
        self.ApplyGrayscaleFixed_Incremental();
        if self.ram[COLDATA_COPY0] & 0x1f != self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] {
            return;
        }
        self.frame_control_view_mut().set_submodule(0);
        self.frame_control_view_mut().set_subsubmodule(0);
    }

    pub(super) fn Module07_0B_DrainSwampPool(&mut self) {
        const TURN_OFF_WATER_TAB0: [i8; 16] =
            [-1, -1, -1, 1, -1, -1, -1, 1, -1, -1, -1, 1, -1, -1, -1, 1];

        match self.frame_control_view().subsubmodule() {
            0 => {
                if self.ram[TURN_ON_OFF_WATER_CTR] & 7 == 0 {
                    let k = ((self.ram[TURN_ON_OFF_WATER_CTR] >> 2) & 3) as usize;
                    if read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON)
                        == read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_TARGET_DUNGEON)
                    {
                        self.Dungeon_SetAttrForActivatedWaterOff();
                        return;
                    }
                    let delta = TURN_OFF_WATER_TAB0[k] as i16 as u16;
                    let var2 = read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON)
                        .wrapping_add(delta);
                    let var3 = read_le_u16(&self.ram, WATER_HDMA_WINDOW_X_RADIUS_DUNGEON)
                        .wrapping_add(delta);
                    write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON, var2);
                    write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_X_RADIUS_DUNGEON, var3);
                }
                self.ram[TURN_ON_OFF_WATER_CTR] = self.ram[TURN_ON_OFF_WATER_CTR].wrapping_add(1);
                self.AdjustWaterHDMAWindow();
            }
            1 => {
                let tile = self.tile_word(0x01e0, 0);
                for i in 0..0x1000usize {
                    write_le_u16(&mut self.ram, DUNG_BG1 + i * 2, tile);
                }
                self.ram[DUNG_CUR_QUADRANT_UPLOAD] = 0;
                self.frame_control_view_mut().increment_subsubmodule();
            }
            2..=5 => self.Dungeon_FloodSwampWater_PrepTileMap(),
            _ => {}
        }
    }

    pub(super) fn Module07_0C_FloodSwampWater(&mut self) {
        const TURN_ON_WATER_TAB2: [i8; 4] = [1, 1, 1, -1];
        const TURN_ON_WATER_TAB1: [i8; 4] = [1, 2, 1, -1];
        const TURN_ON_WATER_TAB0: [i8; 4] = [1, -1, 1, -1];

        match self.frame_control_view().subsubmodule() {
            0..=3 => self.Dungeon_FloodSwampWater_PrepTileMap(),
            4..=8 => {
                self.ram[TURN_ON_OFF_WATER_CTR] = self.ram[TURN_ON_OFF_WATER_CTR].wrapping_sub(1);
                if self.ram[TURN_ON_OFF_WATER_CTR] == 0 {
                    self.ram[TURN_ON_OFF_WATER_CTR] = 4;
                    self.frame_control_view_mut().increment_subsubmodule();
                    let depth = i32::from(self.frame_control_view().subsubmodule()) - 4;
                    write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_X_RADIUS_DUNGEON, 8);
                    write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_Y_RADIUS_ALT_DUNGEON, 0);
                    write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON, 0x30);
                    self.Dungeon_AdjustWaterVomit(0x1654 + 0x10, depth);
                }
            }
            9 => {
                self.ram[W12SEL_COPY] = 3;
                self.ram[W34SEL_COPY] = 0;
                self.ram[WOBJSEL_COPY] = 0;
                self.ram[TMW_COPY] = 22;
                self.ram[TSW_COPY] = 1;
                self.ram[TS_COPY] = 1;
                self.ram[CGWSEL_COPY] = 2;
                self.ram[CGADSUB_COPY] = 98;
                self.ram[TURN_ON_OFF_WATER_CTR] = 0;
                self.frame_control_view_mut().increment_subsubmodule();
                self.Module07_0C_FloodSwampWater_raise_window(
                    TURN_ON_WATER_TAB0,
                    TURN_ON_WATER_TAB1,
                );
            }
            10 => self
                .Module07_0C_FloodSwampWater_raise_window(TURN_ON_WATER_TAB0, TURN_ON_WATER_TAB1),
            11 => {
                if self.ram[TURN_ON_OFF_WATER_CTR] & 7 == 0 {
                    let k = ((self.ram[TURN_ON_OFF_WATER_CTR] >> 2) & 3) as usize;
                    if read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON)
                        == read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_TARGET_DUNGEON)
                    {
                        self.Dungeon_SetAttrForActivatedWater();
                        return;
                    }
                    let delta = TURN_ON_WATER_TAB2[k] as i16 as u16;
                    let var2 = read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON)
                        .wrapping_add(delta);
                    let var3 = read_le_u16(&self.ram, WATER_HDMA_WINDOW_X_RADIUS_DUNGEON)
                        .wrapping_add(delta);
                    write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON, var2);
                    write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_X_RADIUS_DUNGEON, var3);

                    let a = read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_TARGET_DUNGEON)
                        .wrapping_sub(var2);
                    if a == 0 || a == 8 {
                        self.Dungeon_AdjustWaterVomit(if a == 0 { 0x16b4 } else { 0x168c }, 5);
                    }
                }
                self.ram[TURN_ON_OFF_WATER_CTR] = self.ram[TURN_ON_OFF_WATER_CTR].wrapping_add(1);
                self.AdjustWaterHDMAWindow();
            }
            _ => {}
        }
    }

    fn Module07_0C_FloodSwampWater_raise_window(&mut self, tab0: [i8; 4], tab1: [i8; 4]) {
        let k = (self.ram[TURN_ON_OFF_WATER_CTR] & 3) as usize;
        let r0 = 0x0688u16
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
            .wrapping_sub(0x24);
        let var3 = read_le_u16(&self.ram, WATER_HDMA_WINDOW_X_RADIUS_DUNGEON)
            .wrapping_add(tab0[k] as i16 as u16);
        let var5 = read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_RADIUS_ALT_DUNGEON)
            .wrapping_add(tab1[k] as i16 as u16);
        write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_X_RADIUS_DUNGEON, var3);
        write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_Y_RADIUS_ALT_DUNGEON, var5);
        if var5 >= r0 {
            self.ram[DUNG_HDR_BG2_PROPERTIES] = 7;
            self.frame_control_view_mut().increment_subsubmodule();
        }
        self.ram[TURN_ON_OFF_WATER_CTR] = self.ram[TURN_ON_OFF_WATER_CTR].wrapping_add(1);
        let lower = 0x0688u16
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
            .wrapping_sub(read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON));
        write_le_u16(&mut self.ram, SPOTLIGHT_Y_LOWER, lower);
        let upper = lower.wrapping_add(var5);
        write_le_u16(&mut self.ram, SPOTLIGHT_Y_UPPER, upper);
        self.AdjustWaterHDMAWindow_X(upper);
    }

    pub(super) fn Module07_0D_FloodDam(&mut self) {
        self.FloodDam_PrepFloodHDMA();
        match self.frame_control_view().subsubmodule() {
            0 => self.FloodDam_PrepTiles_init(),
            1..=3 => self.Watergate_Main_State1(),
            4 => self.FloodDam_Expand(),
            5 => self.FloodDam_Fill(),
            other => panic!("invalid Module07_0D_FloodDam subsubmodule_index {other}"),
        }
    }

    pub(super) fn Module07_0E_01_HandleMusicAndResetProps(&mut self) {
        let room = self.world_state_view().dungeon_room();
        if (room == 7 || (room == 23 && !self.zelda_is_playing_music_track(17)))
            && self.ram[LINK_WHICH_PENDANTS] & 1 == 0
        {
            self.ram[MUSIC_CONTROL] = 0xf1;
        }
        self.ram[STAIRCASE_MOVE_COUNTER] = if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            106
        } else {
            88
        };
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.ResetTransitionPropsAndAdvanceSubmodule();
    }

    pub(super) fn Module07_0E_02_ApplyFilterIf(&mut self) {
        if self.ram[STAIRCASE_MOVE_COUNTER] < 9 {
            self.ApplyPaletteFilter_bounce();
            if self.ram[PALETTE_FILTER_COUNTDOWN] != 0 {
                self.ApplyPaletteFilter_bounce();
            }
        }
        if self.ram[STAIRCASE_MOVE_COUNTER] != 0 {
            self.ram[STAIRCASE_MOVE_COUNTER] = self.ram[STAIRCASE_MOVE_COUNTER].wrapping_sub(1);
            return;
        }
        self.ram[TAGALONG_APPEARANCE_NONE_FLAG] = 12;
        self.ram[LINK_VISIBILITY_STATUS] = 12;
    }

    pub(super) fn Dungeon_AdvanceThenSetBossMusicUnorthodox(&mut self) {
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        self.ram[STAIRCASE_MOVE_COUNTER] = 0x38;
        self.frame_control_view_mut().increment_subsubmodule();
        self.Dungeon_SetBossMusicUnorthodox();
    }

    pub(super) fn Dungeon_SetBossMusicUnorthodox(&mut self) {
        let room = self.world_state_view().dungeon_room();
        let mut x = 0x1c;
        if room != 16 {
            x = 0x15;
            if room != 7 {
                x = 0x11;
                if room != 23 || self.zelda_is_playing_music_track(17) {
                    return;
                }
            }
            if self.ram[CURRENT_MUSIC_CONTROL] != 0xf1 && self.ram[LINK_WHICH_PENDANTS] & 1 != 0 {
                return;
            }
        }
        self.ram[MUSIC_CONTROL] = x;
    }

    pub(super) fn Module07_0E_SpiralStairs(&mut self) {
        if self.frame_control_view().subsubmodule() >= 7 {
            self.Graphics_IncrementalVRAMUpload();
            self.Dungeon_LoadAttribute_Selectable();
        }
        self.HandleLinkOnSpiralStairs();
        match self.frame_control_view().subsubmodule() {
            0 => self.Module07_0E_00_InitPriorityAndScreens(),
            1 => self.Module07_0E_01_HandleMusicAndResetProps(),
            2 => self.Module07_0E_02_ApplyFilterIf(),
            3 => self.Dungeon_InitializeRoomFromSpecial(),
            4 => self.DungeonTransition_TriggerBGC34UpdateAndAdvance(),
            5 => self.DungeonTransition_TriggerBGC56UpdateAndAdvance(),
            6 => self.DungeonTransition_LoadSpriteGFX(),
            7 => self.Dungeon_SyncBackgroundsFromSpiralStairs(),
            8 => self.Dungeon_InterRoomTrans_State4(),
            9 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            10 => self.Dungeon_InterRoomTrans_State4(),
            11 => self.Dungeon_SpiralStaircase11(),
            12 => self.Dungeon_SpiralStaircase12(),
            13 => self.Dungeon_SpiralStaircase11(),
            14 => self.Dungeon_SpiralStaircase12(),
            15 => self.Dungeon_DoubleApplyAndIncrementGrayscale(),
            16 => self.Dungeon_AdvanceThenSetBossMusicUnorthodox(),
            17 => self.Dungeon_SpiralStaircase17(),
            18 => self.Dungeon_SpiralStaircase18(),
            19 => self.Module07_0E_13_SetRoomAndLayerAndCache(),
            other => panic!("invalid Module07_0E_SpiralStairs subsubmodule_index {other}"),
        }
    }

    pub(super) fn Dungeon_SyncBackgroundsFromSpiralStairs(&mut self) {
        if self.ram[FOLLOWER_INDICATOR] == 6 && self.ram[DUNGEON_ROOM_INDEX] == 100 {
            self.ram[FOLLOWER_INDICATOR] = 0;
        }
        let bak = self.ram[LINK_IS_ON_LOWER_LEVEL];
        let y_delta = if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            48
        } else {
            (-48i16) as u16
        };
        let y = self.player_state_view().y().wrapping_add(y_delta);
        self.player_state_view_mut().set_y(y);
        self.ram[LINK_IS_ON_LOWER_LEVEL] =
            K_TELEPORT_PIT_LEVEL2[self.ram[CUR_STAIRCASE_PLANE] as usize];
        self.SpiralStairs_MakeNearbyWallsHighPriority_Exiting();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = bak;
        let y_delta = if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            (-48i16) as u16
        } else {
            48
        };
        let y = self.player_state_view().y().wrapping_add(y_delta);
        self.player_state_view_mut().set_y(y);
        copy_le_u16(&mut self.ram, BG1HOFS_COPY2, BG2HOFS_COPY2);
        copy_le_u16(&mut self.ram, BG1VOFS_COPY2, BG2VOFS_COPY2);
        self.Dungeon_AdjustForRoomLayout();
        let mut ts = K_SPIRAL_TAB1[self.ram[DUNG_HDR_BG2_PROPERTIES] as usize];
        let mut tm = 0x16;
        if ts < 0 {
            tm = 0x17;
            ts = 0;
        }
        if self.ram[DUNG_HDR_BG2_PROPERTIES] == 2 {
            ts = 3;
        }
        self.ram[TM_COPY] = tm;
        self.ram[TS_COPY] = ts as u8;
        if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_sub(1);
        } else {
            self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_add(1);
        }
        self.ram[STAIRCASE_MOVE_COUNTER] = 24;
        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
        self.hud_restore_torch_background();
        self.Dungeon_InterRoomTrans_notDarkRoom();
    }

    pub(super) fn Dungeon_SpiralStaircase17(&mut self) {
        self.SpiralStairs_FindLandingSpot();
        self.ram[STAIRCASE_MOVE_COUNTER] = self.ram[STAIRCASE_MOVE_COUNTER].wrapping_sub(1);
        if self.ram[STAIRCASE_MOVE_COUNTER] == 0 {
            self.ram[STAIRCASE_MOVE_COUNTER] = if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
                10
            } else {
                24
            };
            self.frame_control_view_mut().increment_subsubmodule();
        }
    }

    pub(super) fn Dungeon_SpiralStaircase18(&mut self) {
        self.SpiralStairs_FindLandingSpot();
        self.ram[STAIRCASE_MOVE_COUNTER] = self.ram[STAIRCASE_MOVE_COUNTER].wrapping_sub(1);
        if self.ram[STAIRCASE_MOVE_COUNTER] == 0 {
            self.frame_control_view_mut().increment_subsubmodule();
            self.ram[OVERWORLD_MAP_STATE] = 0;
        }
    }

    pub(super) fn Module07_0E_00_InitPriorityAndScreens(&mut self) {
        self.SpiralStairs_MakeNearbyWallsHighPriority_Entering();
        if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
            self.ram[TM_COPY] &= 0x0f;
            self.ram[TS_COPY] |= 0x10;
            self.ram[LINK_IS_ON_LOWER_LEVEL] = 3;
        }
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Module07_0E_13_SetRoomAndLayerAndCache(&mut self) {
        let plane = self.ram[CUR_STAIRCASE_PLANE] as usize;
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = K_TELEPORT_PIT_LEVEL1[plane];
        self.ram[LINK_IS_ON_LOWER_LEVEL] = K_TELEPORT_PIT_LEVEL2[plane];
        self.ram[TM_COPY] |= 0x10;
        self.ram[TS_COPY] &= 0x0f;
        if self.ram[WHICH_STAIRCASE_INDEX] & 4 == 0 {
            self.SpiralStairs_MakeNearbyWallsLowPriority();
        }
        self.ram[DUNGEON_ROOM_INDEX2] = self.ram[DUNGEON_ROOM_INDEX];
        self.ResetThenCacheRoomEntryProperties();
    }

    pub(super) fn RepositionLinkAfterSpiralStairs(&mut self) {
        self.ram[LINK_VISIBILITY_STATUS] = 0;
        self.ram[TAGALONG_APPEARANCE_NONE_FLAG] = 0;
        let mut i =
            if self.ram[CUR_STAIRCASE_PLANE] == 0 && self.ram[STAIRCASE_LOWER_LEVEL_STATUS] != 0 {
                1usize
            } else {
                0usize
            };
        if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            i += 2;
        }
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(K_SPIRAL_STAIRCASE_X[i] as i16 as u16);
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(K_SPIRAL_STAIRCASE_Y[i] as i16 as u16);
        self.player_state_view_mut().set_x(x);
        self.player_state_view_mut().set_y(y);

        if self.ram[TM_COPY] & 0x10 != 0 {
            if self.ram[CUR_STAIRCASE_PLANE] == 2 {
                self.ram[LINK_IS_ON_LOWER_LEVEL] = 3;
                self.ram[TM_COPY] &= 0x0f;
                self.ram[TS_COPY] |= 0x10;
                if self.ram[STAIRCASE_LOWER_LEVEL_STATUS] != 2 {
                    let y = self.player_state_view().y().wrapping_add(24);
                    self.player_state_view_mut().set_y(y);
                }
            }
            self.follower_initialize();
        } else {
            if self.ram[CUR_STAIRCASE_PLANE] != 2 {
                self.ram[TM_COPY] |= 0x10;
                self.ram[TS_COPY] &= 0x0f;
                if self.ram[STAIRCASE_LOWER_LEVEL_STATUS] != 2 {
                    let y = self.player_state_view().y().wrapping_sub(24);
                    self.player_state_view_mut().set_y(y);
                }
            }
            self.follower_initialize();
        }
    }

    pub(super) fn Dungeon_PlayMusicIfDefeated(&mut self) {
        let room = self.world_state_view().dungeon_room();
        let mut x = 0x14;
        if room != 18 {
            x = 0x10;
            if room != 2 {
                if !K_BOSS_ROOMS_DUNGEON.contains(&room) {
                    return;
                }
                if self.sprite_check_if_screen_is_clear() {
                    return;
                }
                x = 0x15;
            }
        }
        self.ram[MUSIC_CONTROL] = x;
    }

    pub(super) fn Dungeon_LoadCustomTileAttr(&mut self) {
        let offset = self.asset_u16(51, self.ram[AUX_TILE_THEME_INDEX] as usize) as usize;
        let attrs = self.asset_raw(52).expect("missing dungeon tile attr asset");
        let custom_attrs = attrs[offset..offset + 0x80].to_vec();
        self.ram[ATTRIBUTES_FOR_TILE + 0x140..ATTRIBUTES_FOR_TILE + 0x1c0]
            .copy_from_slice(&custom_attrs);
    }

    pub(super) fn SaveDungeonKeys(&mut self) {
        let mut idx = self.ram[CUR_PALACE_INDEX_X2];
        if idx == 0xff {
            return;
        }
        if idx == 2 {
            idx = 0;
        }
        self.ram[LINK_KEYS_EARNED_PER_DUNGEON + ((idx >> 1) as usize)] = self.ram[LINK_NUM_KEYS];
    }

    pub(super) fn Dungeon_LoadRoom(&mut self) {
        self.Dungeon_LoadHeader();
        self.dungeon_load_room_reset_floor_velocity();
        self.ram[SOMARIA_BLOCK_BG_CHECK_FLAG] = 0;
        self.ram[DUNG_HDR_COLLISION_2_MIRROR] = self.ram[DUNG_HDR_COLLISION_2];
        self.ram[DUNG_HDR_COLLISION_2_MIRROR + 1] = self.ram[DUNG_HDR_TAG];
        self.ram[BG1_MOVE_CALC_BUFFER] = 0x30;
        self.ram[BG1_MOVE_CALC_BUFFER + 1] = 0xff;
        for &offset in &[
            0x41a, 0x420, 0x422, 0x424, 0x436, 0x452, 0x453, 0x454, 0x456, 0x44e, 0x450, 0x0fc,
            0x45c, 0x438, 0x43a, 0x43c, 0x43e, 0x440, 0x442, 0x4ae, 0x444, 0x446, 0x448, 0x49a,
            0x49c, 0x49e, 0x47e, 0x480, 0x482, 0x484, 0x4a2, 0x4a4, 0x4a6, 0x4a8, 0x430, 0x432,
            0x42c, 0x42e, 0x478, 0x496, 0x498, 0x4b0, 0x460,
        ] {
            write_le_u16(&mut self.ram, offset, 0);
        }
        write_le_u16(&mut self.ram, INVISIBLE_DOOR_DIR_AND_INDEX_X2, 0xffff);
        self.fill_ram(DUNG_TORCH_TIMERS_DUNGEON, 16, 0);
        self.fill_ram(DUNG_REPLACEMENT_TILE_STATE, 32, 0);
        self.fill_ram(DUNG_OBJECT_POS_IN_OBJDATA, 32, 0);
        self.fill_ram(DUNG_OBJECT_TILEMAP_POS, 32, 0);
        self.ram[DOOR_TYPE_AND_SLOT..DOOR_TYPE_AND_SLOT + 32].fill(0);
        self.ram[DUNG_DOOR_TILEMAP_ADDRESS..DUNG_DOOR_TILEMAP_ADDRESS + 32].fill(0);
        self.ram[DUNG_DOOR_DIRECTION..DUNG_DOOR_DIRECTION + 32].fill(0);
        self.ram[DUNG_EXIT_DOOR_COUNT..DUNG_EXIT_DOOR_COUNT + 10].fill(0);
        write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, 0);
        self.RoomDraw_DrawFloorsCurrentRoom();
        self.ram[DUNG_LINE_PTRS_ROW0..DUNG_LINE_PTRS_ROW0 + DUNGEON_DRAW_OBJECT_OFFSETS_BG1.len()]
            .copy_from_slice(&DUNGEON_DRAW_OBJECT_OFFSETS_BG1);
        self.RoomDraw_DrawAllObjectsCurrentRoom();
        let room = self.world_state_view().dungeon_room();
        for offset in (0..0x018c).step_by(4) {
            if read_le_u16(&self.ram, MOVABLE_BLOCK_DATAS + offset) == room {
                let tilemap = read_le_u16(&self.ram, MOVABLE_BLOCK_DATAS + offset + 2);
                self.DrawObjects_PushableBlock(tilemap, offset as u16);
            }
        }

        let misc_objs = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX);
        write_le_u16(&mut self.ram, DUNG_INDEX_OF_TORCHES_START, misc_objs);
        write_le_u16(&mut self.ram, DUNG_INDEX_OF_TORCHES, misc_objs);
        let mut i = 0usize;
        loop {
            if read_le_u16(&self.ram, DUNG_TORCH_DATA_DUNGEON + i) == room {
                i += 2;
                loop {
                    let t = read_le_u16(&self.ram, DUNG_TORCH_DATA_DUNGEON + i);
                    i += 2;
                    self.DrawObjects_LightableTorch(t, (i - 2) as u16);
                    if read_le_u16(&self.ram, DUNG_TORCH_DATA_DUNGEON + i) == 0xffff {
                        break;
                    }
                }
                break;
            }
            i += 2;
            loop {
                let t = read_le_u16(&self.ram, DUNG_TORCH_DATA_DUNGEON + i);
                i += 2;
                if t == 0xffff {
                    break;
                }
            }
            if i == 0x0120 {
                break;
            }
        }
        write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, 0x0120);
        if room == 0x51 {
            self.room_prioritize_throne_room_door_edge();
        }
    }

    fn dungeon_load_room_reset_floor_velocity(&mut self) {
        write_le_u16(&mut self.ram, DUNG_FLOOR_Y_VEL_DUNGEON, 0);
        write_le_u16(&mut self.ram, DUNG_FLOOR_X_VEL, 0);
    }

    pub(super) fn Dungeon_LoadHeader(&mut self) {
        self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = 0;
        self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH] = 0;
        self.ram[DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED] = 0;
        const ADJUSTMENT: [i16; 2] = [256, -256];

        let submodule = self.frame_control_view().submodule();
        let bg_h = read_le_u16(&self.ram, BG2HOFS_COPY2);
        let bg_v = read_le_u16(&self.ram, BG2VOFS_COPY2);
        let direction = self.ram[LINK_DIRECTION] & 0x0f;
        let (load_h, load_v) = if submodule == 0 {
            (bg_h & !0x01ff, bg_v & !0x01ff)
        } else if submodule == 21 || (submodule < 18 && submodule >= 6) {
            (
                bg_h.wrapping_add(0x20) & !0x01ff,
                bg_v.wrapping_add(0x20) & !0x01ff,
            )
        } else if (direction >> 1) < 2 {
            (
                bg_h.wrapping_add(ADJUSTMENT[(direction >> 1) as usize] as u16) & !0x01ff,
                bg_v.wrapping_add(0x20) & !0x01ff,
            )
        } else {
            (
                bg_h.wrapping_add(0x20) & !0x01ff,
                bg_v.wrapping_add(ADJUSTMENT[(direction >> 3) as usize] as u16) & !0x01ff,
            )
        };
        write_le_u16(&mut self.ram, DUNG_LOADE_BGOFFS_H_COPY, load_h);
        write_le_u16(&mut self.ram, DUNG_LOADE_BGOFFS_V_COPY, load_v);

        let room = self.world_state_view().dungeon_room() as usize;
        let header = self
            .GetRoomHeaderPtr(room)
            .expect("dungeon room must have a header")
            .to_vec();

        self.ram[DUNG_HDR_BG2_PROPERTIES_BACKUP] = self.ram[DUNG_HDR_BG2_PROPERTIES];
        self.ram[DUNG_HDR_BG2_PROPERTIES] = header[0] >> 5;
        self.ram[DUNG_HDR_COLLISION] = (header[0] >> 2) & 7;
        self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = self.ram[DUNG_WANT_LIGHTS_OUT];
        self.ram[DUNG_WANT_LIGHTS_OUT] = header[0] & 1;
        let pal = DUNG_PAL_INFOS[header[1] as usize];
        self.ram[PALETTE_MAIN_INDOORS] = pal[0];
        self.ram[PALETTE_SP0L] = pal[1];
        self.ram[PALETTE_SP5L] = pal[2];
        self.ram[PALETTE_SP6L] = pal[3];
        self.ram[AUX_TILE_THEME_INDEX] = header[2];
        self.ram[SPRITE_GRAPHICS_INDEX] = header[3].wrapping_add(0x40);
        self.ram[DUNG_HDR_COLLISION_2] = header[4];
        self.ram[DUNG_HDR_TAG] = header[5];
        self.ram[DUNG_HDR_TAG + 1] = header[6];
        self.ram[DUNG_HDR_HOLE_TELEPORTER_PLANE] = header[7] & 3;
        self.ram[DUNG_HDR_HOLE_TELEPORTER_PLANE + 1] = (header[7] >> 2) & 3;
        self.ram[DUNG_HDR_HOLE_TELEPORTER_PLANE + 2] = (header[7] >> 4) & 3;
        self.ram[DUNG_HDR_HOLE_TELEPORTER_PLANE + 3] = (header[7] >> 6) & 3;
        self.ram[DUNG_HDR_HOLE_TELEPORTER_PLANE + 4] = header[8] & 3;
        self.ram[DUNG_HDR_TRAVEL_DESTINATIONS..DUNG_HDR_TRAVEL_DESTINATIONS + 5]
            .copy_from_slice(&header[9..14]);
        write_le_u16(&mut self.ram, DUNG_FLAG_TRAPDOORS_DOWN, 1);
        self.ram[DUNG_OVERLAY_TO_LOAD] = 0;
        write_le_u16(&mut self.ram, DUNG_INDEX_X3, (room as u16).wrapping_mul(3));

        let saved = self.asset_u16_from_ram(0xf000, room);
        write_le_u16(&mut self.ram, DUNG_DOOR_OPENED, saved & 0xf000);
        write_le_u16(
            &mut self.ram,
            DUNG_DOOR_OPENED_INCL_ADJACENT,
            (saved & 0xf000) | 0x0f00,
        );
        write_le_u16(
            &mut self.ram,
            DUNG_SAVEGAME_STATE_BITS,
            (saved & 0x0ff0) << 4,
        );
        write_le_u16(&mut self.ram, DUNG_QUADRANTS_VISITED, saved & 0x000f);

        self.copy_room_door_info_to_ram(room, DUNG_DOOR_TILEMAP_ADDRESS, 16);
        if (room.wrapping_sub(1) & 0x0f) != 0x0f {
            self.Dungeon_CheckAdjacentRoomsForOpenDoors(18, room.wrapping_sub(1));
        }
        if (room.wrapping_add(1) & 0x0f) != 0 {
            self.Dungeon_CheckAdjacentRoomsForOpenDoors(12, room.wrapping_add(1));
        }
        if room >= 16 {
            self.Dungeon_CheckAdjacentRoomsForOpenDoors(6, room - 16);
        }
        if room + 16 < 0x140 {
            self.Dungeon_CheckAdjacentRoomsForOpenDoors(0, room + 16);
        }
    }

    fn copy_room_door_info_to_ram(&mut self, room: usize, dst: usize, max_words: usize) {
        let doors = self
            .GetRoomDoorInfo(room)
            .expect("dungeon room must have door info")
            .to_vec();
        for i in 0..max_words {
            let word = read_word_from_slice(&doors, i * 2);
            if word == 0xffff {
                write_le_u16(&mut self.ram, dst + i * 2, 0);
                return;
            }
            write_le_u16(&mut self.ram, dst + i * 2, word);
        }
        write_le_u16(&mut self.ram, dst + max_words * 2, 0);
    }

    fn Dungeon_CheckAdjacentRoomsForOpenDoors(&mut self, idx: usize, room: usize) {
        const LOOKUP: [u16; 24] = [
            0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x61, 0x71, 0x81, 0x91, 0xa1, 0xb1, 0x02, 0x12,
            0x22, 0x32, 0x42, 0x52, 0x63, 0x73, 0x83, 0x93, 0xa3, 0xb3,
        ];
        const LOOKUP2: [u16; 24] = [
            0x61, 0x71, 0x81, 0x91, 0xa1, 0xb1, 0x0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x63, 0x73,
            0x83, 0x93, 0xa3, 0xb3, 0x02, 0x12, 0x22, 0x32, 0x42, 0x52,
        ];

        self.Dungeon_LoadAdjacentRoomDoors(room);
        for i in 0..8 {
            let mut a = read_le_u16(&self.ram, ADJACENT_DOORS + i * 2);
            if a == 0xffff {
                break;
            }
            a &= 0x00ff;
            let mut j = idx;
            while j < idx + 6 {
                if a == LOOKUP[j] {
                    let rev = LOOKUP2[j] as u8;
                    for door in 0..8 {
                        let cur = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2);
                        if cur as u8 == rev {
                            let kind = (cur >> 8) as u8;
                            if kind == 0x30 {
                                break;
                            }
                            if kind == 0x44 || kind == 0x18 {
                                if room != read_le_u16(&self.ram, DUNGEON_ROOM_INDEX_PREV) as usize
                                {
                                    break;
                                }
                                write_le_u16(&mut self.ram, DUNG_FLAG_TRAPDOORS_DOWN, 0);
                            } else if read_le_u16(&self.ram, ADJACENT_DOORS_FLAGS)
                                & upper_bitmask(i)
                                == 0
                            {
                                break;
                            }
                            let opened = read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT)
                                | upper_bitmask(door);
                            write_le_u16(&mut self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, opened);
                            break;
                        }
                    }
                    break;
                }
                j += 1;
            }
        }
    }

    fn Dungeon_LoadAdjacentRoomDoors(&mut self, room: usize) {
        let flags = (self.asset_u16_from_ram(0xf000, room) & 0xf000) | 0x0f00;
        write_le_u16(&mut self.ram, ADJACENT_DOORS_FLAGS, flags);
        let Some(doors) = self.GetRoomDoorInfo(room).map(Vec::from) else {
            write_le_u16(&mut self.ram, ADJACENT_DOORS, 0xffff);
            return;
        };
        for i in 0..8 {
            let a = read_word_from_slice(&doors, i * 2);
            write_le_u16(&mut self.ram, ADJACENT_DOORS + i * 2, a);
            if a == 0xffff {
                break;
            }
            if (a & 0xff00) == 0x4000 || (a & 0xff00) < 0x0200 {
                let flags = read_le_u16(&self.ram, ADJACENT_DOORS_FLAGS) | upper_bitmask(i);
                write_le_u16(&mut self.ram, ADJACENT_DOORS_FLAGS, flags);
            }
        }
    }

    pub(super) fn RoomDraw_DrawFloorsCurrentRoom(&mut self) {
        let room = self.world_state_view().dungeon_room() as usize;
        let Some(room_layout) = self.dungeon_room_layout(room).map(Vec::from) else {
            return;
        };
        if room_layout.is_empty() {
            return;
        }
        self.RoomDraw_DrawFloors(&room_layout);
    }

    pub(super) fn RoomDraw_DrawFloors(&mut self, level_data: &[u8]) {
        let offs = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS) as usize;
        let floor_types = level_data.get(offs).copied().unwrap_or(0);
        self.ram[DUNG_LINE_PTRS_ROW0..DUNG_LINE_PTRS_ROW0 + DUNGEON_DRAW_OBJECT_OFFSETS_BG2.len()]
            .copy_from_slice(&DUNGEON_DRAW_OBJECT_OFFSETS_BG2);
        self.ram[DUNG_FLOOR_1_FILLER_TILES] = floor_types & 0xf0;
        self.ram[DUNG_FLOOR_1_FILLER_TILES + 1] = 0;
        self.RoomDraw_FloorChunks(0x4000, (floor_types & 0xf0) as usize);

        self.ram[DUNG_LINE_PTRS_ROW0..DUNG_LINE_PTRS_ROW0 + DUNGEON_DRAW_OBJECT_OFFSETS_BG1.len()]
            .copy_from_slice(&DUNGEON_DRAW_OBJECT_OFFSETS_BG1);
        self.ram[DUNG_FLOOR_2_FILLER_TILES] = (floor_types & 0x0f) << 4;
        self.ram[DUNG_FLOOR_2_FILLER_TILES + 1] = 0;
        self.RoomDraw_FloorChunks(0x2000, ((floor_types & 0x0f) << 4) as usize);
        write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, 1);
    }

    pub(super) fn RoomDraw_DrawAllObjectsCurrentRoom(&mut self) {
        let room = self.world_state_view().dungeon_room() as usize;
        let Some(room_layout) = self.dungeon_room_layout(room).map(Vec::from) else {
            return;
        };
        let old_offs = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS) as usize;
        let layout = room_layout.get(old_offs).copied().unwrap_or(0) as usize;
        write_le_u16(
            &mut self.ram,
            DUNG_LAYOUT_AND_STARTING_QUADRANT,
            layout as u16,
        );
        if let Some(default_layout) = self.default_room_layout(layout >> 2).map(Vec::from) {
            write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, 0);
            self.RoomData_DrawObjects_from(&default_layout);
        }

        write_le_u16(
            &mut self.ram,
            DUNG_LOAD_PTR_OFFS,
            old_offs.saturating_add(1) as u16,
        );
        self.RoomData_DrawObjects_from(&room_layout);
        let pos = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, pos);
        self.ram[DUNG_LINE_PTRS_ROW0..DUNG_LINE_PTRS_ROW0 + DUNGEON_DRAW_OBJECT_OFFSETS_BG2.len()]
            .copy_from_slice(&DUNGEON_DRAW_OBJECT_OFFSETS_BG2);
        self.RoomData_DrawObjects_from(&room_layout);
        let pos = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, pos);
        self.ram[DUNG_LINE_PTRS_ROW0..DUNG_LINE_PTRS_ROW0 + DUNGEON_DRAW_OBJECT_OFFSETS_BG1.len()]
            .copy_from_slice(&DUNGEON_DRAW_OBJECT_OFFSETS_BG1);
        self.RoomData_DrawObjects_from(&room_layout);
        write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, 0x0120);
    }

    pub(super) fn RoomDraw_DrawAllObjects(&mut self, level_data: &[u8]) {
        self.RoomData_DrawObjects_from(level_data);
    }

    pub(super) fn RoomData_DrawObjects_from(&mut self, layout: &[u8]) {
        loop {
            write_le_u16(&mut self.ram, DUNG_DRAW_WIDTH_INDICATOR, 0);
            write_le_u16(&mut self.ram, DUNG_DRAW_HEIGHT_INDICATOR, 0);
            let pos = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS) as usize;
            let raw = read_word_from_slice(layout, pos);
            if raw == 0xffff {
                return;
            }
            if raw == 0xfff0 {
                break;
            }
            let idx = layout[pos + 2];
            write_le_u16(
                &mut self.ram,
                DUNG_LOAD_PTR_OFFS,
                pos.wrapping_add(3) as u16,
            );
            self.RoomData_DrawObject(raw, idx);
        }
        loop {
            let pos = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS).wrapping_add(2) as usize;
            write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, pos as u16);
            let raw = read_word_from_slice(layout, pos);
            if raw == 0xffff {
                return;
            }
            self.RoomData_DrawObject_Door(raw);
        }
    }

    pub(super) fn RoomData_DrawObject(&mut self, raw: u16, idx: u8) {
        if raw & 0xfc != 0xfc {
            let width = (raw & 3) as u8;
            let height = ((raw >> 8) & 3) as u8;
            write_le_u16(&mut self.ram, DUNG_DRAW_WIDTH_INDICATOR, u16::from(width));
            write_le_u16(&mut self.ram, DUNG_DRAW_HEIGHT_INDICATOR, u16::from(height));
            let x = (raw as u8 >> 2) as u16;
            let y = raw >> 10;
            let dsto = y * 64 + x;
            if idx < 0xf8 {
                self.LoadType1ObjectSubtype1(idx, width, height, dsto);
            } else {
                let object = ((idx & 7) << 4) | (((raw >> 8) as u8 & 3) << 2) | (raw as u8 & 3);
                let mut dst = 0;
                self.LoadType1ObjectSubtype3(object, &mut dst, dsto);
            }
        } else {
            let x = ((raw & 3) << 4) | ((raw >> 12) & 0x0f);
            let y = (((raw >> 8) & 0x0f) << 2) | ((idx as u16) >> 6);
            let dsto = y * 64 + x;
            let mut dst = 0;
            self.LoadType1ObjectSubtype2(idx & 0x3f, &mut dst, dsto);
        }
    }

    pub(super) fn LoadType1ObjectSubtype1(&mut self, idx: u8, width: u8, height: u8, dsto: u16) {
        let Some(src) = object_subtype1_param(idx) else {
            panic!("LoadType1ObjectSubtype1 invalid object id {idx:#04x}");
        };
        match idx {
            0x00 | 0xb8 | 0xb9 => {
                let count = size_1to15_or(width, height, 32);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 2);
                }
            }
            0x01 | 0x02 | 0xb6 | 0xb7 => {
                let count = size_1to15_or(width, height, 26);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 2, 2);
                }
            }
            0x03 | 0x04 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4_both_bgs(src, dsto + i * 2, 2);
                }
            }
            0x05 | 0x06 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 6, 2);
                }
            }
            0x07 | 0x08 | 0x53 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 2);
                }
            }
            0x09 | 0x0c | 0x0d | 0x10 | 0x11 | 0x14 => {
                let count = size_a_to_a_plus_15(width, height, 6);
                let mut dst = dsto;
                for _ in 0..count {
                    self.RoomDraw_DrawObject2x2and1(src, dst);
                    dst = dst.wrapping_sub(63);
                }
            }
            0x0a | 0x0b | 0x0e | 0x0f | 0x12 | 0x13 => {
                let count = size_a_to_a_plus_15(width, height, 6);
                let mut dst = dsto;
                for _ in 0..count {
                    self.RoomDraw_DrawObject2x2and1(src, dst);
                    dst = dst.wrapping_add(65);
                }
            }
            0x15 | 0x18 | 0x19 | 0x1c | 0x1d | 0x20 => {
                let count = size_a_to_a_plus_15(width, height, 6);
                let mut dst = dsto;
                for _ in 0..count {
                    for y in 0..5 {
                        self.room_write_bg(0x2000, dst + y * 64, self.tile_word(src, y as usize));
                        self.room_write_bg(0x4000, dst + y * 64, self.tile_word(src, y as usize));
                    }
                    dst = dst.wrapping_sub(63);
                }
            }
            0x16 | 0x17 | 0x1a | 0x1b | 0x1e | 0x1f => {
                let count = size_a_to_a_plus_15(width, height, 6);
                let mut dst = dsto;
                for _ in 0..count {
                    for y in 0..5 {
                        self.room_write_bg(0x2000, dst + y * 64, self.tile_word(src, y as usize));
                        self.room_write_bg(0x4000, dst + y * 64, self.tile_word(src, y as usize));
                    }
                    dst = dst.wrapping_add(65);
                }
            }
            0x21 => {
                let mut count = (((width as u16) << 2) | height as u16) * 2 + 1;
                let mut dst = dsto;
                self.RoomDraw_1x3_rightwards(src, dst, 2);
                dst += 2;
                while count != 0 {
                    self.RoomDraw_1x3_rightwards(src + 6, dst, 1);
                    dst += 1;
                    count -= 1;
                }
                self.RoomDraw_1x3_rightwards(src + 12, dst, 1);
            }
            0x22 => {
                let count = size_a_to_a_plus_15(width, height, 2);
                if self.room_read_current(dsto) & 0x03ff != 0x00e2 {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                let tile = self.tile_word(src, 1);
                for i in 1..=count {
                    self.room_write_current(dsto + i, tile);
                }
                self.room_write_current(dsto + count + 1, self.tile_word(src, 2));
            }
            0x23..=0x2e | 0x3f..=0x46 | 0xb3 | 0xb4 => {
                let count = size_1to16(width, height);
                let tile = self.room_read_current(dsto) & 0x03ff;
                if tile != 0x01db && tile != 0x01a6 && tile != 0x01dd && tile != 0x01fc {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                let fill = self.tile_word(src, 1);
                for i in 1..=count {
                    self.room_write_current(dsto + i, fill);
                }
                self.room_write_current(dsto + count + 1, self.tile_word(src, 2));
            }
            0x2f => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst) & 0x03ff != 0x00e2 {
                    self.room_write_current(dst, self.tile_word(src, 1));
                    self.room_write_current(dst + 1, self.tile_word(src, 2));
                    self.room_write_current(dst + 64, fill);
                    self.room_write_current(dst + 65, fill);
                    dst += 2;
                }
                for _ in 0..count {
                    self.room_write_current(dst, self.tile_word(src, 3));
                    self.room_write_current(dst + 64, fill);
                    dst += 1;
                }
                self.room_write_current(dst, self.tile_word(src, 4));
                self.room_write_current(dst + 1, self.tile_word(src, 5));
                self.room_write_current(dst + 64, fill);
                self.room_write_current(dst + 65, fill);
            }
            0x30 => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst + 64) & 0x03ff != 0x00e2 {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 1, fill);
                    self.room_write_current(dst + 64, self.tile_word(src, 1));
                    self.room_write_current(dst + 65, self.tile_word(src, 2));
                    dst += 2;
                }
                for _ in 0..count {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 64, self.tile_word(src, 3));
                    dst += 1;
                }
                self.room_write_current(dst, fill);
                self.room_write_current(dst + 1, fill);
                self.room_write_current(dst + 64, self.tile_word(src, 4));
                self.room_write_current(dst + 65, self.tile_word(src, 5));
            }
            0x33 | 0xb2 | 0xba => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_4x4(src, dsto + i * 4);
                }
            }
            0x34 => {
                let count = size_a_to_a_plus_15(width, height, 4);
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i, tile);
                }
            }
            0x36 | 0x37 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_4x4(src, dsto + i * 6);
                }
            }
            0x38 => {
                let statue_src = 0x0e26;
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_1x3_rightwards(statue_src, dsto + i * 4, 2);
                }
            }
            0x39 | 0x3d => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 6, 2);
                }
            }
            0x3a | 0x3b => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_1x3_rightwards(src, dsto + i * 8, 4);
                }
            }
            0x3c => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    let dst = dsto + i * 4;
                    self.RoomDraw_Rightwards2x2(src, dst);
                    self.RoomDraw_Rightwards2x2(src + 8, dst + 6 * 64);
                }
            }
            0x3e | 0x4b => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 14);
                }
            }
            0x47 => {
                let count = size_1to16(width, height) * 2;
                let mut dst = self.RoomDraw_DrawObject2x2and1(src, dsto) + 1;
                for _ in 0..count {
                    self.RoomDraw_DrawObject2x2and1(src + 10, dst);
                    dst += 1;
                }
                self.RoomDraw_DrawObject2x2and1(src + 20, dst);
            }
            0x48 => {
                let count = size_1to16(width, height) * 2;
                let mut dst = dsto;
                self.RoomDraw_1x3_rightwards(src, dst, 1);
                dst += 1;
                for _ in 0..count {
                    self.room_write_current(dst, self.tile_word(src, 3));
                    self.room_write_current(dst + 64, self.tile_word(src, 4));
                    self.room_write_current(dst + 128, self.tile_word(src, 5));
                    dst += 1;
                }
                self.RoomDraw_1x3_rightwards(src + 12, dst, 1);
            }
            0x49 | 0x4a => {
                let count = size_1to16(width, height);
                self.RoomDraw_Downwards4x2VariableSpacing(4, src, dsto, count);
            }
            0x4c => {
                let count = size_1to16(width, height) * 2;
                let mut dst = self.RoomDraw_RightwardBarSegment(src, dsto) + 1;
                for _ in 0..count {
                    dst = self.RoomDraw_RightwardBarSegment(src + 6, dst) + 1;
                }
                self.RoomDraw_RightwardBarSegment(src + 12, dst);
            }
            0x4d..=0x4f => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                self.RoomData_DrawObject_nx4(src, dst, 1);
                dst += 1;
                for _ in 0..count {
                    self.RoomData_DrawObject_nx4(src + 8, dst, 2);
                    dst += 2;
                }
                self.RoomDraw_RightwardShelfEnd(src + 24, &mut dst);
            }
            0x50 => {
                let count = size_a_to_a_plus_15(width, height, 2);
                self.Object_Fill_Nx1(count, src, dsto);
            }
            0x51 | 0x52 | 0x5b | 0x5c => {
                let mut count = size_1to16(width, height);
                let mut dst = dsto;
                self.RoomDraw_1x3_rightwards(src, dst, 2);
                dst += 2;
                while count > 1 {
                    self.RoomDraw_1x3_rightwards(src + 12, dst, 2);
                    dst += 2;
                    count -= 1;
                }
                self.RoomDraw_1x3_rightwards(src + 24, dst, 2);
            }
            0x55 | 0x56 => {
                let count = size_1to16(width, height);
                self.RoomDraw_Downwards4x2VariableSpacing(12, src, dsto, count);
            }
            0x5d => {
                let count = size_1to16(width, height) + 1;
                let mut dst = dsto;
                self.RoomDraw_1x3_rightwards(src, dst, 2);
                dst += 2;
                for _ in 0..count {
                    self.RoomDraw_RightwardBarSegment(src + 12, dst);
                    dst += 1;
                }
                self.RoomDraw_1x3_rightwards(src + 18, dst, 2);
            }
            0x5e | 0xbb => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 4);
                }
            }
            0x5f => {
                let count = size_a_to_a_plus_15(width, height, 21);
                if self.room_read_current(dsto) & 0x03ff != 0x00e2 {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                let fill = self.tile_word(src, 1);
                for i in 1..=count {
                    self.room_write_current(dsto + i, fill);
                }
                self.room_write_current(dsto + count + 1, self.tile_word(src, 2));
            }
            0x60 | 0x92 | 0x93 => {
                let count = size_1to15_or(width, height, 32);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 64 * 2);
                }
            }
            0x61 | 0x62 | 0x90 | 0x91 => {
                let count = size_1to15_or(width, height, 26);
                if replay_room_write_trace_enabled() {
                    eprintln!(
                        "room-object idx=0x{idx:02x} src=0x{src:04x} dsto=0x{dsto:04x} count=0x{count:04x} branch=wall-vert-ud"
                    );
                }
                self.RoomDraw_Downwards4x2VariableSpacing(2 * 64, src, dsto, count);
            }
            0x63 | 0x64 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.Object_Draw_4x2_BothBgs(src, dsto + i * 2 * 64);
                }
            }
            0x65 | 0x66 => {
                let count = size_1to16(width, height);
                self.RoomDraw_Downwards4x2VariableSpacing(6 * 64, src, dsto, count);
            }
            0x67 | 0x68 | 0x7d => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 2 * 64);
                }
            }
            0x7c => {
                let count = size_1to16(width, height) + 1;
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i * 64, tile);
                }
            }
            0x7f | 0x80 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 12 * 64, 2);
                }
            }
            0x81..=0x84 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 6 * 64, 3);
                }
            }
            0x85 | 0x86 => {
                let mut count = size_1to16(width, height);
                let mut dst = dsto;
                self.Object_Draw_3x2(src, dst);
                dst = dst.wrapping_add(2 * 64);
                while {
                    count -= 1;
                    count != 0
                } {
                    self.Object_Draw_3x2(src + 12, dst);
                    dst = dst.wrapping_add(2 * 64);
                }
                self.Object_Draw_3x2(src + 24, dst);
            }
            0x69 | 0x71 => {
                let count = size_a_to_a_plus_15(width, height, if idx == 0x69 { 2 } else { 4 });
                if idx == 0x69 {
                    if self.room_read_current(dsto) & 0x03ff != 0x00e3 {
                        self.room_write_current(dsto, self.tile_word(src, 0));
                    }
                    let tile = self.tile_word(src, 1);
                    for i in 1..=count {
                        self.room_write_current(dsto + i * 64, tile);
                    }
                    self.room_write_current(dsto + (count + 1) * 64, self.tile_word(src, 2));
                } else {
                    let tile = self.tile_word(src, 0);
                    for i in 0..count {
                        self.room_write_current(dsto + i * 64, tile);
                    }
                }
            }
            0x6a | 0x6b | 0x79 | 0x7a | 0x8d | 0x8e => {
                let count = size_1to16(width, height);
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i * 64, tile);
                }
            }
            0x6c => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst) & 0x03ff != 0x00e3 {
                    self.room_write_current(dst, self.tile_word(src, 1));
                    self.room_write_current(dst + 64, self.tile_word(src, 2));
                    self.room_write_current(dst + 1, fill);
                    self.room_write_current(dst + 65, fill);
                    dst += 128;
                }
                for _ in 0..count {
                    self.room_write_current(dst, self.tile_word(src, 3));
                    self.room_write_current(dst + 1, fill);
                    dst += 64;
                }
                self.room_write_current(dst, self.tile_word(src, 4));
                self.room_write_current(dst + 64, self.tile_word(src, 5));
                self.room_write_current(dst + 1, fill);
                self.room_write_current(dst + 65, fill);
            }
            0x6d => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst + 1) & 0x03ff != 0x00e3 {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 64, fill);
                    self.room_write_current(dst + 1, self.tile_word(src, 1));
                    self.room_write_current(dst + 65, self.tile_word(src, 2));
                    dst += 128;
                }
                for _ in 0..count {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 1, self.tile_word(src, 3));
                    dst += 64;
                }
                self.room_write_current(dst, fill);
                self.room_write_current(dst + 64, fill);
                self.room_write_current(dst + 1, self.tile_word(src, 4));
                self.room_write_current(dst + 65, self.tile_word(src, 5));
            }
            0x70 | 0x94 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_4x4(src, dsto + i * 4 * 64);
                }
            }
            0x73 | 0x74 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_4x4(src, dsto + i * 6 * 64);
                }
            }
            0x75 | 0x87 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 6 * 64, 2);
                }
            }
            0x76 | 0x77 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(src, dsto + i * 8 * 64, 3);
                }
            }
            0x78 | 0x7b => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 14 * 64);
                }
            }
            0x89 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomDraw_Rightwards2x2(src, dsto + i * 4 * 64);
                }
            }
            0x8a => {
                let count = size_a_to_a_plus_15(width, height, 21);
                if self.room_read_current(dsto) & 0x03ff != 0x00e3 {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                let tile = self.tile_word(src, 1);
                for i in 1..=count {
                    self.room_write_current(dsto + i * 64, tile);
                }
                self.room_write_current(dsto + (count + 1) * 64, self.tile_word(src, 2));
            }
            0x88 => {
                let mut count = size_1to16(width, height);
                let mut dst = dsto;
                self.RoomDraw_Rightwards2x2(src, dst);
                dst = dst.wrapping_add(2 * 64);
                let src = src + 8;
                while count != 0 {
                    self.room_write_current(dst, self.tile_word(src, 0));
                    self.room_write_current(dst + 1, self.tile_word(src, 1));
                    dst = dst.wrapping_add(64);
                    count -= 1;
                }
                self.RoomDraw_1x3_rightwards(src + 4, dst, 2);
            }
            0x8b | 0x8c => {
                let count = size_a_to_a_plus_15(width, height, 8);
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i * 64, tile);
                }
            }
            0x8f => {
                let count = size_a_to_a_plus_15(width, height, 2) * 2;
                let mut dst = dsto;
                self.room_write_current(dst, self.tile_word(src, 0));
                self.room_write_current(dst + 1, self.tile_word(src, 1));
                for _ in 0..count {
                    self.room_write_current(dst + 64, self.tile_word(src, 2));
                    self.room_write_current(dst + 65, self.tile_word(src, 3));
                    dst = dst.wrapping_add(64);
                }
            }
            0x95 => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                let mut pos = dsto;
                for _ in 0..count {
                    self.RoomDraw_SinglePot(src, &mut dst, pos);
                    dst = dst.wrapping_add(2 * 64);
                    pos = pos.wrapping_add(2 * 64);
                }
            }
            0x96 => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                let mut pos = dsto;
                for _ in 0..count {
                    self.RoomDraw_HammerPegSingle(src, &mut dst, pos);
                    dst = dst.wrapping_add(2 * 64);
                    pos = pos.wrapping_add(2 * 64);
                }
            }
            0xa0 | 0xa5 | 0xa9 => {
                let mut count = size_a_to_a_plus_15(width, height, 4);
                let mut dst = dsto;
                while count != 0 {
                    self.room_fill_horizontal(dst, count, self.tile_word(src, 0));
                    dst = dst.wrapping_add(64);
                    count -= 1;
                }
            }
            0xa1 | 0xa6 | 0xaa => {
                let count = size_a_to_a_plus_15(width, height, 4);
                for y in 0..count {
                    self.room_fill_horizontal(dsto + y * 64, y + 1, self.tile_word(src, 0));
                }
            }
            0xa2 | 0xa7 | 0xab => {
                let mut count = size_a_to_a_plus_15(width, height, 4);
                let mut dst = dsto;
                while count != 0 {
                    self.room_fill_horizontal(dst, count, self.tile_word(src, 0));
                    dst = dst.wrapping_add(65);
                    count -= 1;
                }
            }
            0xa3 | 0xa8 | 0xac => {
                let mut count = size_a_to_a_plus_15(width, height, 4);
                let mut dst = dsto;
                while count != 0 {
                    self.room_fill_horizontal(dst, count, self.tile_word(src, 0));
                    dst = dst.wrapping_sub(63);
                    count -= 1;
                }
            }
            0xa4 => self.Object_Hole(src, dsto, width, height),
            0xb0 | 0xb1 => {
                let count = size_a_to_a_plus_15(width, height, 8);
                self.Object_Fill_Nx1(count, src, dsto);
            }
            0xb5 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.RoomData_DrawObject_nx4(0x0b16, dsto + i * 2, 2);
                }
            }
            0xbc => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                let mut pos = dsto;
                for _ in 0..count {
                    self.RoomDraw_SinglePot(src, &mut dst, pos);
                    dst = dst.wrapping_add(2);
                    pos = pos.wrapping_add(2);
                }
            }
            0xbd => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                let mut pos = dsto;
                for _ in 0..count {
                    self.RoomDraw_HammerPegSingle(src, &mut dst, pos);
                    dst = dst.wrapping_add(2);
                    pos = pos.wrapping_add(2);
                }
            }
            0xc5..=0xca | 0xd1 | 0xd2 | 0xd9 | 0xdf..=0xe8 => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                for y in 0..count_y {
                    let mut dst = dsto + y * 4 * 64;
                    for _ in 0..count_x {
                        self.RoomDraw_A_Many32x32Blocks(1, src, &mut dst);
                    }
                }
            }
            0xc3 | 0xd7 => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                let tile = self.tile_word(src, 0);
                for y in 0..count_y {
                    for x in 0..count_x {
                        self.room_fill_rect(dsto + y * 3 * 64 + x * 3, 3, 3, tile);
                    }
                }
            }
            0xd8 => {
                let count_x = width as u16 + 2;
                let count_y = height as u16 + 2;
                write_le_u16(
                    &mut self.ram,
                    WATER_HDMA_WINDOW_X_RADIUS_DUNGEON,
                    count_x << 4,
                );
                write_le_u16(
                    &mut self.ram,
                    WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON,
                    count_y << 4,
                );
                write_le_u16(
                    &mut self.ram,
                    WATER_HDMA_WINDOW_Y_TARGET_DUNGEON,
                    (count_y << 4).wrapping_sub(24),
                );
                let hdma0 = ((dsto & 0x003f) << 3)
                    .wrapping_add(count_x << 4)
                    .wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_H_COPY));
                let hdma1 = ((dsto & 0x0fc0) >> 3)
                    .wrapping_add(count_y << 4)
                    .wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_V_COPY));
                write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_X_DUNGEON, hdma0);
                write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_Y_DUNGEON, hdma1);
                if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x0800 != 0 {
                    self.ram[DUNG_HDR_TAG + 1] = 0;
                    self.ram[DUNG_HDR_BG2_PROPERTIES] = 0;
                    let north_stairs = read_le_u16(&self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER);
                    let active_ladders = read_le_u16(&self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS);
                    let south_stairs = read_le_u16(&self.ram, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER);
                    write_le_u16(
                        &mut self.ram,
                        DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS,
                        north_stairs,
                    );
                    write_le_u16(&mut self.ram, WATER_SIDE_STEP_SWITCH, active_ladders);
                    write_le_u16(&mut self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS, 0);
                    write_le_u16(&mut self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER, 0);
                    write_le_u16(&mut self.ram, DUNG_NUM_STAIRS_WET, south_stairs);
                    write_le_u16(&mut self.ram, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER, 0);
                    let water_dsto = dsto
                        .wrapping_add((count_x - 1) << 1)
                        .wrapping_add((count_y - 1) << 7);
                    self.DrawWaterThingBg(0x4000, water_dsto, 0x1438);
                } else {
                    let mut dst = dsto;
                    for _ in 0..count_y {
                        let mut row = dst;
                        self.RoomDraw_A_Many32x32Blocks(count_x as i32, 0x0110, &mut row);
                        dst = dst.wrapping_add(4 * 64);
                    }
                }
            }
            0xda => {
                let count_x = width as u16 + 2;
                let count_y = height as u16 + 2;
                write_le_u16(
                    &mut self.ram,
                    WATER_HDMA_WINDOW_X_RADIUS_DUNGEON,
                    (count_x << 4).wrapping_sub(24),
                );
                write_le_u16(
                    &mut self.ram,
                    WATER_HDMA_WINDOW_Y_TARGET_DUNGEON,
                    (count_y << 4).wrapping_sub(8),
                );
                write_le_u16(
                    &mut self.ram,
                    WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON,
                    (count_y << 4).wrapping_sub(32),
                );
                write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_Y_RADIUS_ALT_DUNGEON, 0);
                let hdma0 = ((dsto & 0x003f) << 3)
                    .wrapping_add(count_x << 4)
                    .wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_H_COPY));
                let hdma1 = ((dsto & 0x0fc0) >> 3)
                    .wrapping_add(count_y << 4)
                    .wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_V_COPY))
                    .wrapping_sub(8);
                write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_X_DUNGEON, hdma0);
                write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_Y_DUNGEON, hdma1);
                if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x0800 != 0 {
                    self.ram[DUNG_HDR_TAG + 1] = 0;
                } else {
                    self.ram[DUNG_HDR_BG2_PROPERTIES] = 0;
                    let north_stairs = read_le_u16(&self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER);
                    let active_ladders = read_le_u16(&self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS);
                    let south_stairs = read_le_u16(&self.ram, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER);
                    write_le_u16(
                        &mut self.ram,
                        DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS,
                        north_stairs,
                    );
                    write_le_u16(&mut self.ram, WATER_SIDE_STEP_SWITCH, active_ladders);
                    write_le_u16(&mut self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS, 0);
                    write_le_u16(&mut self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER, 0);
                    write_le_u16(&mut self.ram, DUNG_NUM_STAIRS_WET, south_stairs);
                    write_le_u16(&mut self.ram, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER, 0);
                }
                let mut dst = dsto;
                for _ in 0..(count_y * 2 - 1) {
                    let row = dst;
                    for _ in 0..count_x {
                        for y in 0..2 {
                            for x in 0..4 {
                                self.room_write_current(
                                    dst + y * 64 + x,
                                    self.tile_word(0x0110, (y * 4 + x) as usize),
                                );
                            }
                        }
                        dst = dst.wrapping_add(4);
                    }
                    dst = row.wrapping_add(2 * 64);
                }
            }
            0xc4 => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                let src = read_le_u16(&self.ram, DUNG_FLOOR_2_FILLER_TILES) as usize;
                for y in 0..count_y {
                    let mut dst = dsto + y * 4 * 64;
                    self.RoomDraw_A_Many32x32Blocks(count_x as i32, src, &mut dst);
                }
            }
            0xdb => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                let src = read_le_u16(&self.ram, DUNG_FLOOR_1_FILLER_TILES) as usize;
                for y in 0..count_y {
                    let mut dst = dsto + y * 4 * 64;
                    self.RoomDraw_A_Many32x32Blocks(count_x as i32, src, &mut dst);
                }
            }
            0xc0 | 0xc2 => {
                let tile = self.tile_word(src, 0);
                for y in 0..=height as u16 {
                    for x in 0..=width as u16 {
                        self.room_fill_rect(dsto + y * 4 * 64 + x * 4, 4, 4, tile);
                    }
                }
            }
            0xc1 => {
                let width = width as u16 + 4;
                let height = height as u16 + 1;
                let mut src = src;
                let mut dst = dsto;
                self.RoomDraw_1x3_rightwards(src, dst, 3);
                src += 18;
                dst += 3;
                for _ in 0..width {
                    self.RoomDraw_1x3_rightwards(src, dst, 2);
                    dst += 2;
                }
                self.RoomDraw_1x3_rightwards(src + 12, dst, 3);
                src += 30;

                dst = dsto + 3 * 64;
                for _ in 0..height {
                    let mut row = dst;
                    self.Object_Draw_3x2(src, row);
                    row += 3;
                    for _ in 0..width {
                        self.RoomDraw_Rightwards2x2(src + 12, row);
                        row += 2;
                    }
                    self.Object_Draw_3x2(src + 20, row);
                    dst += 2 * 64;
                }

                let bottom_start = dst;
                src += 32;
                self.RoomDraw_1x3_rightwards(src, dst, 3);
                src += 18;
                dst += 3;
                for _ in 0..width {
                    self.RoomDraw_1x3_rightwards(src, dst, 2);
                    dst += 2;
                }
                self.RoomDraw_1x3_rightwards(src + 12, dst, 3);

                self.RoomDraw_Rightwards2x2(
                    0x0590,
                    bottom_start
                        .wrapping_add(width + 2)
                        .wrapping_sub((height + 1) * 64),
                );
            }
            0xcd => self.RoomDraw_MovingWallRight(width, height, dsto),
            0xce => self.RoomDraw_MovingWallLeft(width, height, dsto),
            0x31
            | 0x32
            | 0x35
            | 0x54
            | 0x57..=0x5a
            | 0x6e
            | 0x6f
            | 0x72
            | 0x7e
            | 0x97..=0x9f
            | 0xad..=0xaf
            | 0xbe
            | 0xbf => {}
            0xdd => {
                let width = width as u16 + 1;
                let height = height as u16 * 2 + 2;
                self.Object_Table_Helper(src, dsto, width);
                for y in 1..height {
                    self.Object_Table_Helper(src + 8, dsto + y * 64, width);
                }
                self.Object_Table_Helper(src + 16, dsto + height * 64, width);
                self.Object_Table_Helper(src + 24, dsto + (height + 1) * 64, width);
            }
            0xdc => {
                let mut dst = dsto
                    | if read_le_u16(&self.ram, DUNG_LINE_PTRS_ROW0) == 0x4000 {
                        0x1000
                    } else {
                        0
                    };
                let width = width as u16 + 1;
                let height = height as u16 * 2 + 5;
                for _ in 0..height {
                    self.RoomDraw_Chest_platform_row(0x0ab4, dst, width);
                    dst += 64;
                }
                self.RoomDraw_Chest_platform_row(0x0ab4 + 2, dst, width);
                dst += 64;
                self.RoomDraw_Chest_platform_row(0x0ab4 + 4, dst, width);
            }
            0xde => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                for y in 0..count_y {
                    for x in 0..count_x {
                        self.RoomDraw_Rightwards2x2(src, dsto + y * 2 * 64 + x * 2);
                    }
                }
            }
            _ => panic!("LoadType1ObjectSubtype1 unhandled object id {idx:#04x}"),
        }
    }

    pub(super) fn LoadType1ObjectSubtype3(&mut self, idx: u8, _dst: &mut u16, dsto: u16) {
        let Some(src) = object_subtype3_param(idx) else {
            panic!("LoadType1ObjectSubtype3 invalid object id {idx:#04x}");
        };
        match idx {
            0x00 => {
                if self.ram[DUNG_HDR_TAG + 1] == 27 {
                    let room = self.world_state_view().dungeon_room() as usize;
                    if read_le_u16(&self.ram, SAVE_DUNG_INFO + room * 2) & 0x0100 != 0 {
                        self.RoomDraw_WaterHoldingObject(5, 0x162c, dsto);
                        return;
                    }
                } else if self.ram[DUNG_HDR_TAG + 1] == 25
                    && read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x0800 != 0
                {
                    self.RoomDraw_WaterHoldingObject(5, 0x162c, dsto);
                    return;
                }
                write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_SRC_POS_X2, dsto * 2);
                self.RoomDraw_WaterHoldingObject(3, src, dsto);
            }
            0x01 => self.RoomDraw_WaterHoldingObject(5, 0x162c, dsto),
            0x02 => self.RoomDraw_WaterHoldingObject(7, src, dsto),
            0x03 | 0x0e => {
                self.ram[SOMARIA_BLOCK_BG_CHECK_FLAG] =
                    self.ram[SOMARIA_BLOCK_BG_CHECK_FLAG].wrapping_add(1);
                self.room_write_current(dsto, self.tile_word(src, 0));
            }
            0x04..=0x0c | 0x0f => {
                self.room_write_current(dsto, self.tile_word(src, 0));
            }
            0x0d | 0x17 => self.RoomDraw_PrisonCell(dsto),
            0x10
            | 0x11
            | 0x13
            | 0x1a
            | 0x22..=0x25
            | 0x3e..=0x46
            | 0x49
            | 0x4a
            | 0x4f..=0x53
            | 0x56..=0x59
            | 0x5e
            | 0x5f
            | 0x63..=0x65
            | 0x75
            | 0x7c..=0x7e => {
                self.RoomDraw_Rightwards2x2(src, dsto);
            }
            0x12 => {
                if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x1000 == 0 {
                    let dst = dsto | self.room_plane_offset();
                    let src = 0x1dd6;
                    for i in 0..3 {
                        let col = i * 2;
                        self.room_write_bg(0x2000, dst + col, self.tile_word(src, 0));
                        self.room_write_bg(0x2000, dst + col + 3 * 64, self.tile_word(src, 0));
                        self.room_write_bg(0x2000, dst + col + 6 * 64, self.tile_word(src, 0));
                        self.room_write_bg(0x2000, dst + col + 64, self.tile_word(src, 1));
                        self.room_write_bg(0x2000, dst + col + 4 * 64, self.tile_word(src, 1));
                        self.room_write_bg(0x2000, dst + col + 7 * 64, self.tile_word(src, 1));
                    }
                }
            }
            0x14 | 0x4e | 0x67 | 0x68 | 0x6c | 0x6d | 0x79 => {
                self.RoomDraw_1x3_rightwards(src, dsto, 4);
            }
            0x15 => {
                if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x8000 == 0 {
                    self.RoomDraw_SomeBigDecors(10, 0x1dfa, dsto);
                }
            }
            0x16 => {
                let mut dst = dsto;
                self.RoomDraw_HammerPegSingle(src, &mut dst, dsto);
            }
            0x18 => self.RoomDraw_CellLock(dsto),
            0x19 => self.RoomDraw_Chest(dsto),
            0x1b => {
                self.write_stairs_table(DUNG_STAIRS_TABLE_1, DUNG_NUM_STAIRS_1, dsto);
                self.Object_DrawNx4_BothBgs(4, src, dsto);
            }
            0x1c => {
                self.write_stairs_table(DUNG_STAIRS_TABLE_2, DUNG_NUM_STAIRS_2, dsto);
                self.Object_DrawNx4_BothBgs(4, src, dsto);
            }
            0x1d => {
                self.write_stairs_table(DUNG_STAIRS_TABLE_2, DUNG_NUM_STAIRS_WET, dsto);
                self.RoomDraw_4x4(src, dsto);
            }
            0x1e => {
                let next = self.write_stairs_table(
                    DUNG_INTER_STARCASES,
                    DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS,
                    dsto,
                );
                for offset in [
                    DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2,
                    DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                ] {
                    write_le_u16(&mut self.ram, offset, next);
                }
                self.RoomDraw_Object_Nx4(4, src, dsto);
            }
            0x1f => {
                let next = self.write_stairs_table(
                    DUNG_INTER_STARCASES,
                    DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                    dsto,
                );
                write_le_u16(
                    &mut self.ram,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                    next,
                );
                self.RoomDraw_Object_Nx4(4, src, dsto);
            }
            0x20 => {
                let next = self.write_stairs_table(
                    DUNG_INTER_STARCASES,
                    DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS,
                    dsto,
                );
                for offset in [
                    DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2,
                    DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                ] {
                    write_le_u16(&mut self.ram, offset, next);
                }
                self.RoomDraw_Object_Nx4(4, src, dsto);
            }
            0x21 => {
                let next = self.write_stairs_table(
                    DUNG_INTER_STARCASES,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                    dsto,
                );
                write_le_u16(
                    &mut self.ram,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                    next,
                );
                self.RoomDraw_Object_Nx4(4, src, dsto);
            }
            0x26 => self.RoomDraw_LowerDoorStairsUp(src, dsto, true),
            0x27 => self.RoomDraw_LowerDoorStairsUp(src, dsto, false),
            0x28 => self.RoomDraw_LowerDoorStairsDown(src, dsto, true),
            0x29 => self.RoomDraw_LowerDoorStairsDown(src, dsto, false),
            0x2a => {
                self.RoomDraw_SingleLampCone(0x0514, 0x16dc);
                self.RoomDraw_SingleLampCone(0x0554, 0x17f6);
                self.RoomDraw_SingleLampCone(0x1514, 0x1914);
                self.RoomDraw_SingleLampCone(0x1554, 0x1a2a);
            }
            0x2b => {
                let mut dst = dsto;
                self.DrawBigGraySegment(0x1010, src, &mut dst, dsto);
            }
            0x2c => {
                let mut dst = dsto;
                self.DrawBigGraySegment(0x2020, 0x0e62, &mut dst, dsto);
                self.DrawBigGraySegment(0x2121, 0x0e6a, &mut dst, dsto + xy(2, 0) as u16);
                self.DrawBigGraySegment(0x2222, 0x0e72, &mut dst, dsto + xy(0, 2) as u16);
                self.DrawBigGraySegment(0x2323, 0x0e7a, &mut dst, dsto + xy(2, 2) as u16);
            }
            0x2d => self.RoomDraw_AgahnimAltar(dsto),
            0x2e => self.RoomDraw_AgahnimsWindows(dsto),
            0x2f => {
                let mut dst = 0;
                self.RoomDraw_SinglePot(0x0e82, &mut dst, dsto);
            }
            0x30 => {
                let mut dst = dsto;
                self.DrawBigGraySegment(0x1212, src, &mut dst, dsto);
            }
            0x31 => {
                const CHEST_OPEN_MASKS: [u16; 6] = [0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000];
                let i = read_le_u16(&self.ram, DUNG_NUM_CHESTS_X2) as usize;
                let chest = i >> 1;
                let loc = dsto * 2 | 0x8000 | self.room_plane_tilemap_bit();
                write_le_u16(&mut self.ram, DUNG_CHEST_LOCATIONS + chest * 2, loc);
                let next = (i as u16).wrapping_add(2);
                write_le_u16(&mut self.ram, DUNG_NUM_CHESTS_X2, next);
                write_le_u16(&mut self.ram, DUNG_NUM_BIGKEY_LOCKS_X2, next);
                if chest < CHEST_OPEN_MASKS.len()
                    && read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & CHEST_OPEN_MASKS[chest]
                        != 0
                {
                    write_le_u16(&mut self.ram, DUNG_CHEST_LOCATIONS + chest * 2, 0);
                    self.RoomDraw_1x3_rightwards(0x14c4, dsto, 4);
                } else {
                    self.RoomDraw_1x3_rightwards(0x14ac, dsto, 4);
                }
            }
            0x32 => self.RoomDraw_1x3_rightwards(src, dsto, 4),
            0x33 => {
                if self.ram[DUNG_HDR_TAG + 1] == 27 {
                    let room = self.world_state_view().dungeon_room() as usize;
                    if read_le_u16(&self.ram, SAVE_DUNG_INFO + room * 2) & 0x0100 == 0 {
                        self.ram[DUNG_HDR_BG2_PROPERTIES] = 0;
                        self.write_stairs_table(DUNG_STAIRS_TABLE_2, DUNG_NUM_STAIRS_WET, dsto);
                    } else {
                        self.ram[CGWSEL_COPY] = 2;
                        self.ram[CGADSUB_COPY] = 0x62;
                        self.write_stairs_table(
                            DUNG_STAIRS_TABLE_2,
                            DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER,
                            dsto,
                        );
                    }
                } else {
                    self.write_stairs_table(
                        DUNG_STAIRS_TABLE_2,
                        DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER,
                        dsto,
                    );
                }
                self.RoomDraw_4x4(src, dsto);
            }
            0x3a | 0x3b => {
                self.RoomDraw_1x3_rightwards(src, dsto, 4);
                self.RoomDraw_1x3_rightwards(src + 24, dsto + 3 * 64, 4);
            }
            0x3c | 0x3d | 0x5c => self.RoomDraw_Object_Nx4(6, src, dsto),
            0x47 => {
                let mut dst = dsto;
                self.RoomDraw_BombableFloor(src, &mut dst, dsto);
            }
            0x48 | 0x66 | 0x6b | 0x7a => self.RoomDraw_4x4(src, dsto),
            0x4b | 0x76 | 0x77 => self.RoomDraw_1x3_rightwards(src, dsto, 8),
            0x4c => self.RoomDraw_SomeBigDecors(6, 0x1f92, dsto),
            0x4d | 0x5d => self.RoomDraw_1x3_rightwards(src, dsto, 6),
            0x54 => self.RoomDraw_FortuneTellerRoom(dsto),
            0x55 | 0x5b => {
                for x in 0..3 {
                    self.room_write_current(dsto + x, self.tile_word(src, x as usize));
                }
                for y in 1..=3 {
                    for x in 0..3 {
                        self.room_write_current(
                            dsto + y * 64 + x,
                            self.tile_word(src, (3 + x) as usize),
                        );
                    }
                }
                for x in 0..3 {
                    self.room_write_current(
                        dsto + x + 4 * 64,
                        self.tile_word(src, (6 + x) as usize),
                    );
                }
            }
            0x5a => self.RoomDraw_WaterHoldingObject(2, src, dsto),
            0x60 | 0x61 => {
                self.RoomDraw_1x3_rightwards(src, dsto, 3);
                self.RoomDraw_1x3_rightwards(src + 18, dsto + 3 * 64, 3);
            }
            0x62 => {
                let mut dst = dsto;
                let mut s = 0x20f6;
                for _ in 0..22 {
                    for y in 0..11 {
                        self.room_write_bg(0x4000, dst + y * 64, self.tile_word(s, y as usize));
                    }
                    dst += 1;
                    s += 22;
                }
                dst -= 22;
                s = 0x22da;
                for i in 0..3 {
                    self.room_write_bg(0x4000, dst + 9 + 11 * 64, self.tile_word(s, i));
                    self.room_write_bg(0x4000, dst + 9 + 12 * 64, self.tile_word(s, i + 3));
                    dst += 1;
                }
            }
            0x69 | 0x6a | 0x6e | 0x6f => self.RoomDraw_Object_Nx4(3, src, dsto),
            0x70 => {
                self.RoomDraw_4x4(src, dsto);
                self.RoomDraw_4x4(0x2376, dsto + 2 * 64);
                self.RoomDraw_4x4(0x2396, dsto + 6 * 64);
            }
            0x71 => {
                if read_le_u16(&self.ram, SAVE_DUNG_INFO + 101 * 2) & 0x0100 != 0 {
                    self.Object_Draw8x8(src, dsto);
                }
            }
            0x72 => {
                if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x8000 == 0 {
                    self.RoomDraw_SomeBigDecors(10, src, dsto);
                }
            }
            0x73 => self
                .RoomDraw_FloorChunks(read_le_u16(&self.ram, DUNG_LINE_PTRS_ROW0) as usize, 0x00e0),
            0x74 => {
                self.Object_Draw8x8(src, dsto);
            }
            0x78 => {
                self.RoomDraw_4x4(src, dsto);
                self.RoomDraw_4x4(src + 32, dsto.wrapping_sub(2).wrapping_add(4 * 64));
                self.RoomDraw_4x4(src + 32, dsto + 2 + 4 * 64);
            }
            0x7b => {
                let mut dst = dsto;
                for _ in 0..5 {
                    self.RoomDraw_A_Many32x32Blocks(1, src, &mut dst);
                }
                let mut dst = dsto + 4 * 64;
                for _ in 0..5 {
                    self.RoomDraw_A_Many32x32Blocks(1, src, &mut dst);
                }
            }
            _ => panic!("LoadType1ObjectSubtype3 unhandled object id {idx:#04x}"),
        }
    }

    pub(super) fn LoadType1ObjectSubtype2(&mut self, idx: u8, _dst: &mut u16, dsto: u16) {
        let Some(src) = object_subtype2_param(idx) else {
            panic!("LoadType1ObjectSubtype2 invalid object id {idx:#04x}");
        };
        match idx {
            0x00..=0x07 | 0x1c | 0x24 | 0x25 | 0x29 => {
                self.RoomData_DrawObject_nx4(src, dsto, 4);
            }
            0x10..=0x13 => self.RoomData_DrawObject_nx4_both_bgs(src, dsto, 3),
            0x14..=0x17 => self.Object_DrawNx3_BothBgs(4, src, dsto),
            0x18..=0x1b | 0x27 | 0x2b | 0x34 => self.RoomDraw_Rightwards2x2(src, dsto),
            0x1d | 0x21 | 0x26 => self.RoomDraw_1x3_rightwards(src, dsto, 2),
            0x1e => self.RoomDraw_Rightwards2x2(src, dsto),
            0x1f => {
                let index = read_le_u16(&self.ram, DUNG_NUM_STAR_SHAPED_SWITCHES) as usize >> 1;
                let next = read_le_u16(&self.ram, DUNG_NUM_STAR_SHAPED_SWITCHES).wrapping_add(2);
                write_le_u16(&mut self.ram, DUNG_NUM_STAR_SHAPED_SWITCHES, next);
                let plane = self.room_plane_offset();
                write_le_u16(
                    &mut self.ram,
                    STAR_SHAPED_SWITCHES_TILE + index * 2,
                    dsto | plane,
                );
                self.RoomDraw_Rightwards2x2(src, dsto);
            }
            0x20 => {
                self.ram[DUNG_NUM_LIT_TORCHES] = self.ram[DUNG_NUM_LIT_TORCHES].wrapping_add(1);
                self.RoomDraw_Rightwards2x2(src, dsto);
            }
            0x22 | 0x28 => self.Object_Draw_5x4(src, dsto),
            0x23 => self.RoomDraw_1x3_rightwards(src, dsto, 4),
            0x2a => {
                self.ram[DUNG_DRAW_WIDTH_INDICATOR] = 1;
                self.RoomDraw_Downwards4x2VariableSpacing(1, src, dsto, 1);
            }
            0x2c => self.RoomDraw_1x3_rightwards(src, dsto, 6),
            0x2d => {
                let index =
                    read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS) as usize >> 1;
                let plane = self.room_plane_offset();
                write_le_u16(
                    &mut self.ram,
                    DUNG_INTER_STARCASES + index * 2,
                    dsto | plane,
                );
                let next =
                    read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS).wrapping_add(2);
                for offset in [
                    DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS,
                    DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS,
                    DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2,
                    DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2,
                    DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                ] {
                    write_le_u16(&mut self.ram, offset, next);
                }
                self.RoomDraw_4x4(0x1088, dsto);
            }
            0x2e | 0x2f => {
                let index =
                    read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS) as usize >> 1;
                let plane = self.room_plane_offset();
                write_le_u16(
                    &mut self.ram,
                    DUNG_INTER_STARCASES + index * 2,
                    dsto | plane,
                );
                let next =
                    read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS).wrapping_add(2);
                for offset in [
                    DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2,
                    DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                ] {
                    write_le_u16(&mut self.ram, offset, next);
                }
                self.RoomDraw_4x4(0x10a8, dsto);
            }
            0x08..=0x0f => self.RoomData_DrawObject_nx4_both_bgs(src, dsto, 4),
            0x31 => {
                let index = read_le_u16(&self.ram, DUNG_NUM_INROOM_SOUTHDOWN_STAIRS) as usize >> 1;
                write_le_u16(&mut self.ram, DUNG_STAIRS_TABLE_1 + index * 2, dsto);
                let next = read_le_u16(&self.ram, DUNG_NUM_INROOM_SOUTHDOWN_STAIRS).wrapping_add(2);
                write_le_u16(&mut self.ram, DUNG_NUM_INROOM_SOUTHDOWN_STAIRS, next);
                write_le_u16(&mut self.ram, DUNG_NUM_WATER_LADDERS, next);
                write_le_u16(&mut self.ram, WATER_SIDE_STEP_SWITCH, next);
                self.RoomData_DrawObject_nx4_both_bgs(src, dsto, 4);
            }
            0x32 => {
                let next = self.write_stairs_table(
                    DUNG_STAIRS_TABLE_1,
                    DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS,
                    dsto,
                );
                write_le_u16(&mut self.ram, DUNG_NUM_WATER_LADDERS, next);
                write_le_u16(&mut self.ram, WATER_SIDE_STEP_SWITCH, next);
                self.RoomDraw_4x4(src, dsto);
            }
            0x33 => {
                let room = self.world_state_view().dungeon_room() as usize;
                if self.ram[DUNG_HDR_TAG + 1] == 27
                    && read_le_u16(&self.ram, SAVE_DUNG_INFO + room * 2) & 0x0100 == 0
                {
                    self.ram[DUNG_HDR_BG2_PROPERTIES] = 0;
                    let next = self.write_stairs_table(
                        DUNG_STAIRS_TABLE_1,
                        DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS,
                        dsto,
                    );
                    write_le_u16(&mut self.ram, DUNG_NUM_WATER_LADDERS, next);
                    write_le_u16(&mut self.ram, WATER_SIDE_STEP_SWITCH, next);
                    self.RoomDraw_4x4(0x10c8, dsto);
                } else {
                    let next = self.write_stairs_table(
                        DUNG_STAIRS_TABLE_1,
                        DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER,
                        dsto,
                    );
                    write_le_u16(&mut self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS, next);
                    self.RoomDraw_4x4(0x10c8, dsto);
                }
            }
            0x35 => {
                let room = self.world_state_view().dungeon_room() as usize;
                if self.ram[DUNG_HDR_TAG + 1] == 27
                    && read_le_u16(&self.ram, SAVE_DUNG_INFO + room * 2) & 0x0100 == 0
                {
                    let next =
                        self.write_stairs_table(DUNG_STAIRS_TABLE_1, DUNG_NUM_WATER_LADDERS, dsto);
                    write_le_u16(&mut self.ram, WATER_SIDE_STEP_SWITCH, next);
                    self.Object_Draw_4x2_BothBgs(0x1108, dsto);
                } else {
                    self.write_stairs_table(
                        DUNG_STAIRS_TABLE_1,
                        DUNG_NUM_ACTIVATED_WATER_LADDERS,
                        dsto,
                    );
                    self.ram[DUNG_DRAW_WIDTH_INDICATOR] = 1;
                    self.RoomDraw_Downwards4x2VariableSpacing(1, 0x1108, dsto, 1);
                }
            }
            0x36 => {
                let next =
                    self.write_stairs_table(DUNG_STAIRS_TABLE_1, DUNG_NUM_WATER_LADDERS, dsto);
                write_le_u16(&mut self.ram, WATER_SIDE_STEP_SWITCH, next);
                self.Object_Draw_4x2_BothBgs(0x1108, dsto);
            }
            0x37 => {
                if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x0800 == 0 {
                    self.RoomDraw_Object_Nx4(10, src, dsto);
                    self.ram[WATERGATE_POINTER] = 0x0f;
                    write_le_u16(&mut self.ram, WATERGATE_POS, dsto * 2);
                } else {
                    self.RoomDraw_Object_Nx4(10, 0x13e8, dsto);
                    let load_ptr = read_le_u16(&self.ram, DUNG_LOAD_PTR);
                    let load_ptr_offs = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS);
                    let load_ptr_bank = self.ram[DUNG_LOAD_PTR_BANK];
                    self.RoomTag_OperateWaterFlooring();
                    self.ram[DUNG_LOAD_PTR_BANK] = load_ptr_bank;
                    write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, load_ptr_offs);
                    write_le_u16(&mut self.ram, DUNG_LOAD_PTR, load_ptr);
                }
            }
            0x38 => {
                let index =
                    read_le_u16(&self.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS) as usize >> 1;
                let plane = self.room_plane_offset();
                write_le_u16(
                    &mut self.ram,
                    DUNG_INTER_STARCASES + index * 2,
                    dsto.wrapping_sub(0x40) | plane,
                );
                let next =
                    read_le_u16(&self.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS).wrapping_add(2);
                for offset in [
                    DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS,
                    DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2,
                    DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2,
                    DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                ] {
                    write_le_u16(&mut self.ram, offset, next);
                }
                self.RoomDraw_1x3_rightwards(0x1148, dsto, 4);
                let left = self.room_read_bg(0x2000, dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg(0x2000, dsto.wrapping_sub(1), left);
                let right = self.room_read_bg(0x2000, dsto + 4) | 0x2000;
                self.room_write_bg(0x2000, dsto + 4, right);
            }
            0x39 => {
                let index =
                    read_le_u16(&self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS) as usize >> 1;
                let plane = self.room_plane_offset();
                write_le_u16(
                    &mut self.ram,
                    DUNG_INTER_STARCASES + index * 2,
                    dsto.wrapping_sub(0x40) | plane,
                );
                let next =
                    read_le_u16(&self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS).wrapping_add(2);
                for offset in [
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2,
                    DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                ] {
                    write_le_u16(&mut self.ram, offset, next);
                }
                self.RoomDraw_1x3_rightwards(0x1160, dsto, 4);
                let left = self.room_read_bg(0x2000, dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg(0x2000, dsto.wrapping_sub(1), left);
                let right = self.room_read_bg(0x2000, dsto + 4) | 0x2000;
                self.room_write_bg(0x2000, dsto + 4, right);
            }
            0x3a => {
                let index =
                    read_le_u16(&self.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2) as usize >> 1;
                let plane = self.room_plane_offset();
                write_le_u16(
                    &mut self.ram,
                    DUNG_INTER_STARCASES + index * 2,
                    dsto.wrapping_sub(0x40) | plane,
                );
                let next =
                    read_le_u16(&self.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2).wrapping_add(2);
                for offset in [
                    DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2,
                    DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS,
                    DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2,
                    DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                ] {
                    write_le_u16(&mut self.ram, offset, next);
                }
                self.RoomDraw_1x3_rightwards(0x1178, dsto, 4);
                let left = self.room_read_bg(0x4000, dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg(0x4000, dsto.wrapping_sub(1), left);
                let right = self.room_read_bg(0x4000, dsto + 4) | 0x2000;
                self.room_write_bg(0x4000, dsto + 4, right);
            }
            0x3b => {
                let index =
                    read_le_u16(&self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2) as usize >> 1;
                let plane = self.room_plane_offset();
                write_le_u16(
                    &mut self.ram,
                    DUNG_INTER_STARCASES + index * 2,
                    dsto.wrapping_sub(0x40) | plane,
                );
                let next =
                    read_le_u16(&self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2).wrapping_add(2);
                write_le_u16(&mut self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2, next);
                write_le_u16(
                    &mut self.ram,
                    DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                    next,
                );
                write_le_u16(
                    &mut self.ram,
                    DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                    next,
                );
                self.RoomDraw_1x3_rightwards(0x1190, dsto, 4);
                let left = self.room_read_bg(0x4000, dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg(0x4000, dsto.wrapping_sub(1), left);
                let right = self.room_read_bg(0x4000, dsto + 4) | 0x2000;
                self.room_write_bg(0x4000, dsto + 4, right);
            }
            0x3c => {
                let mut dst = dsto;
                let mut s = src;
                for _ in 0..6 {
                    let tile0 = self.tile_word(s, 0);
                    let tile6 = self.tile_word(s, 6);
                    for x in [0, 4, 8, 14, 18, 22] {
                        self.room_write_bg(0x2000, dst + x, tile0);
                    }
                    for x in [1, 5, 9, 15, 19, 23] {
                        self.room_write_bg(0x2000, dst + x, tile0 | 0x4000);
                    }
                    for x in [2, 6, 16, 20] {
                        self.room_write_bg(0x2000, dst + x, tile6);
                    }
                    for x in [3, 7, 17, 21] {
                        self.room_write_bg(0x2000, dst + x, tile6 | 0x4000);
                    }
                    dst += 64;
                    s += 2;
                }
                self.RoomDraw_1x3_rightwards(src + 24, dsto + 10, 4);
            }
            0x3e => self.RoomDraw_1x3_rightwards(src, dsto, 6),
            0x3f => {
                let mut dst = dsto | self.room_plane_offset();
                let mut s = src;
                for _ in 0..8 {
                    for y in 0..7 {
                        self.room_write_bg(0x2000, dst + y * 64, self.tile_word(s, y as usize));
                    }
                    dst += 1;
                    s += 14;
                }
            }
            _ => panic!("LoadType1ObjectSubtype2 unhandled object id {idx:#04x}"),
        }
    }

    pub(super) fn RoomData_DrawObject_Door(&mut self, raw: u16) {
        let door_type = (raw >> 8) as u8;
        let position = ((raw >> 4) & 0x0f) as usize;
        match raw & 3 {
            0 => self.RoomDraw_Door_North(door_type, position),
            1 => self.RoomDraw_Door_South(door_type, position),
            2 => self.RoomDraw_Door_West(door_type, position),
            3 => self.RoomDraw_Door_East(door_type, position),
            _ => unreachable!(),
        }
    }

    pub(super) fn RoomDraw_Door_North(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_UP[position] / 2;
        match door_type {
            DOOR_TYPE_LG_EXPLOSION => self.RoomDraw_Door_ExplodingWall(position),
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.RoomDraw_MarkLayerToggleDoor(dsto.wrapping_sub(0xfe / 2));
            }
            DOOR_TYPE_SLASHABLE => self.RoomDraw_NorthCurtainDoor(dsto),
            DOOR_TYPE_ENTRANCE_DOOR => self.Door_Up_EntranceDoor(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.RoomDraw_MarkDungeonToggleDoor(dsto.wrapping_sub(0xfe / 2))
            }
            DOOR_TYPE_REGULAR2 => {
                self.RoomDraw_MakeDoorPartsHighPriority_Y(dsto & (0xf07f / 2));
                self.RoomDraw_NormalRangedDoors_North(door_type, dsto, position);
            }
            DOOR_TYPE_EXIT_TO_OW => self.room_draw_register_exit_door(dsto),
            DOOR_TYPE_WATERFALL_TUNNEL => {
                self.RoomDraw_NormalRangedDoors_North(door_type, dsto, position);
                self.Door_PrioritizeCurDoor();
            }
            t if (DOOR_TYPE_STAIR_MASK_LOCKED0..=DOOR_TYPE_STAIR_MASK_LOCKED3).contains(&t) => {
                self.Door_Up_StairMaskLocked(door_type, dsto);
            }
            t if (DOOR_TYPE_REGULAR_DOOR33..).contains(&t) => {
                self.RoomDraw_HighRangeDoor_North(door_type, dsto, position);
            }
            _ => self.RoomDraw_NormalRangedDoors_North(door_type, dsto, position),
        }
    }

    pub(super) fn RoomDraw_Door_South(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_DOWN[position] / 2;
        match door_type {
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.RoomDraw_MarkLayerToggleDoor(dsto + xy(1, 4) as u16);
            }
            DOOR_TYPE_ENTRANCE_DOOR => self.Door_Down_EntranceDoor(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.RoomDraw_MarkDungeonToggleDoor(dsto + xy(1, 4) as u16);
            }
            DOOR_TYPE_EXIT_TO_OW => self.room_draw_register_exit_door(dsto),
            t if t >= DOOR_TYPE_REGULAR_DOOR33 => {
                self.RoomDraw_OneSidedLowerShutters_South(door_type, dsto);
            }
            DOOR_TYPE_ENTRANCE_LARGE => {
                self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, dsto);
                self.RoomDraw_SomeBigDecors(10, 0x2656, dsto.wrapping_sub(3 + 4 * 64));
            }
            DOOR_TYPE_ENTRANCE_LARGE2 => {
                let mut dsto = dsto | 0x1000;
                self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, dsto);
                dsto = dsto.wrapping_sub(3 + 4 * 64);
                self.RoomDraw_SomeBigDecors(10, 0x2656, dsto);
                dsto = dsto.wrapping_sub(0x1000).wrapping_add(7 * 64);
                for i in 0..10 {
                    let tile = self.room_read_bg(DUNG_BG1, dsto + i) | 0x2000;
                    self.room_write_bg(DUNG_BG2, dsto + i, tile);
                }
            }
            DOOR_TYPE_ENTRANCE_CAVE | DOOR_TYPE_ENTRANCE_CAVE2 => {
                if door_type == DOOR_TYPE_ENTRANCE_CAVE2 {
                    self.RoomDraw_MakeDoorPartsHighPriority_Y(dsto + xy(0, 4) as u16);
                }
                self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, dsto);
                self.RoomDraw_4x4(0x26f6, dsto);
            }
            DOOR_TYPE_4 => {
                let high_dsto = dsto | 0x1000;
                self.RoomDraw_MakeDoorPartsHighPriority_Y(high_dsto + xy(0, 4) as u16);
                self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, high_dsto);
                self.RoomDraw_4x4(0x26f6, high_dsto);
                for i in 0..4 {
                    let pos = dsto + i + xy(0, 3) as u16;
                    let tile = self.room_read_bg(DUNG_BG1, pos) | 0x2000;
                    self.room_write_bg(DUNG_BG2, pos, tile);
                }
            }
            _ => self.RoomDraw_CheckIfLowerLayerDoors_Y(door_type, dsto),
        }
    }

    pub(super) fn RoomDraw_Door_West(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_LEFT[position] / 2;
        match door_type {
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.RoomDraw_MarkLayerToggleDoor(dsto.wrapping_add(62));
            }
            DOOR_TYPE_ENTRANCE_DOOR => self.Door_Left_EntranceDoor(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.RoomDraw_MarkDungeonToggleDoor(dsto.wrapping_add(62));
            }
            DOOR_TYPE_REGULAR2 => {
                self.RoomDraw_MakeDoorPartsHighPriority_X(dsto & !0x1f);
                self.RoomDraw_NormalRangedDoors_West(door_type, dsto, position);
            }
            DOOR_TYPE_WATERFALL_TUNNEL => {
                self.RoomDraw_NormalRangedDoors_West(door_type, dsto, position);
                self.Door_PrioritizeCurDoor();
            }
            t if t < DOOR_TYPE_REGULAR_DOOR33 => {
                self.RoomDraw_NormalRangedDoors_West(door_type, dsto, position);
            }
            _ => self.RoomDraw_HighRangeDoor_West(door_type, dsto, position),
        }
    }

    pub(super) fn RoomDraw_Door_East(&mut self, door_type: u8, position: usize) {
        let dsto = DOOR_POSITION_RIGHT[position] / 2;
        match door_type {
            DOOR_TYPE_PLAYER_BG_CHANGE => {
                self.RoomDraw_MarkLayerToggleDoor(dsto + xy(4, 1) as u16);
            }
            DOOR_TYPE_ENTRANCE_DOOR => self.Door_Right_EntranceDoor(dsto),
            DOOR_TYPE_THRONE_ROOM => {
                self.RoomDraw_MarkDungeonToggleDoor(dsto + xy(4, 1) as u16);
            }
            t if t < DOOR_TYPE_REGULAR_DOOR33 => {
                self.RoomDraw_NormalRangedDoors_East(door_type, dsto)
            }
            _ => self.RoomDraw_OneSidedLowerShutters_East(door_type, dsto),
        }
    }

    pub(super) fn room_draw_register_exit_door(&mut self, dsto: u16) {
        let index = read_le_u16(&self.ram, DUNG_EXIT_DOOR_COUNT) as usize >> 1;
        if index < 16 {
            write_le_u16(
                &mut self.ram,
                DUNG_EXIT_DOOR_ADDRESSES + index * 2,
                dsto * 2,
            );
        }
        let next = read_le_u16(&self.ram, DUNG_EXIT_DOOR_COUNT).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_EXIT_DOOR_COUNT, next);
    }

    pub(super) fn RoomDraw_NormalRangedDoors_North(
        &mut self,
        door_type: u8,
        dsto: u16,
        position: usize,
    ) {
        if position >= 6 {
            if let Some(&down_dsto_bytes) = DOOR_POSITION_DOWN.get(position - 6) {
                let saved = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX);
                write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, saved | 0x10);
                self.RoomDraw_CheckIfLowerLayerDoors_Y(door_type, down_dsto_bytes / 2);
                write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, saved);
            }
        }
        self.RoomDraw_OneSidedShutters_North(door_type, dsto);
    }

    pub(super) fn RoomDraw_OneSidedShutters_North(&mut self, door_type: u8, dsto: u16) {
        let mut final_type = self.RoomDraw_FlagDoorsAndGetFinalType(0, door_type, dsto);
        if final_type & 0x100 != 0 {
            return;
        }
        if final_type as u8 == DOOR_TYPE_36 || final_type as u8 == DOOR_TYPE_38 {
            final_type = if final_type as u8 == DOOR_TYPE_36 {
                DOOR_TYPE_SHUTTERS_TWO_WAY
            } else {
                DOOR_TYPE_REGULAR
            } as u16;
            self.room_rewrite_last_door_type(final_type as u8);
        }
        if let Some(&src) = DOOR_TYPE_SRC_UP.get(final_type as usize >> 1) {
            self.RoomData_DrawObject_Door_up_4x3(src as usize, dsto);
        }
    }

    pub(super) fn Door_Up_StairMaskLocked(&mut self, door_type: u8, mut dsto: u16) {
        let door = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX) as usize >> 1;
        write_le_u16(&mut self.ram, DUNG_DOOR_DIRECTION + door * 2, 0);
        write_le_u16(
            &mut self.ram,
            DUNG_DOOR_TILEMAP_ADDRESS + door * 2,
            dsto * 2,
        );
        write_le_u16(
            &mut self.ram,
            DOOR_TYPE_AND_SLOT + door * 2,
            ((door as u16) << 8) | door_type as u16,
        );
        if read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) & upper_bitmask(door & 7) != 0 {
            let next = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX).wrapping_add(2);
            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, next);
            return;
        }
        if door_type < DOOR_TYPE_STAIR_MASK_LOCKED2 {
            self.RoomDraw_OneSidedShutters_North(door_type, dsto);
            return;
        }

        let t = self.RoomDraw_FlagDoorsAndGetFinalType(0, door_type, dsto) as usize;
        let src = DOOR_TYPE_SRC_UP.get(t >> 1).copied().unwrap_or(0) as usize;
        for i in 0..4u16 {
            self.room_write_bg(
                0x4000,
                dsto + xy(0, 0) as u16,
                self.tile_word(src, (i * 3) as usize),
            );
            self.room_write_bg(
                0x4000,
                dsto + xy(0, 1) as u16,
                self.tile_word(src, (i * 3 + 1) as usize),
            );
            self.room_write_bg(
                0x4000,
                dsto + xy(0, 2) as u16,
                self.tile_word(src, (i * 3 + 2) as usize),
            );
            dsto = dsto.wrapping_add(1);
        }
        self.Door_PrioritizeCurDoor();
    }

    pub(super) fn RoomDraw_Door_ExplodingWall(&mut self, pos_enum: usize) {
        let dsto = K_DOOR_BLAST_WALL_UP_DSTS
            .get(pos_enum)
            .copied()
            .unwrap_or(0)
            / 2;
        let door = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX) as usize >> 1;
        write_le_u16(
            &mut self.ram,
            DUNG_DOOR_TILEMAP_ADDRESS + door * 2,
            2 * (dsto + 10),
        );
        write_le_u16(
            &mut self.ram,
            DOOR_TYPE_AND_SLOT + door * 2,
            ((door as u16) << 8) | DOOR_TYPE_LG_EXPLOSION as u16,
        );
        if read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) & upper_bitmask(door & 7) == 0 {
            write_le_u16(&mut self.ram, DUNG_DOOR_DIRECTION + door * 2, 0);
            let next = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX).wrapping_add(2);
            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, next);
            return;
        }

        let slot = usize::from(
            self.ram[DUNG_HDR_TAG] != 0x20
                && self.ram[DUNG_HDR_TAG] != 0x25
                && self.ram[DUNG_HDR_TAG] != 0x28,
        );
        self.ram[DUNG_HDR_TAG + slot] = 0;
        self.ram[QUADRANT_FULLSIZE_Y] = 2;
        self.ram[DUNG_BLASTWALL_FLAG_Y] = 1;
        self.RoomDraw_ExplodingWallSegment(DOOR_TYPE_SRC_DOWN[42] as usize, dsto);
        let next = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, next);
        let unk2 = read_le_u16(&self.ram, RESET_XY_CHECK_FLAGS) | 0x0200;
        write_le_u16(&mut self.ram, RESET_XY_CHECK_FLAGS, unk2);
        self.RoomDraw_ExplodingWallSegment(DOOR_TYPE_SRC_UP[42] as usize, dsto + xy(0, 6) as u16);
    }

    pub(super) fn RoomDraw_ExplodingWallSegment(&mut self, mut src: usize, mut dsto: u16) {
        self.RoomDraw_ExplodingWallColumn(src, dsto);
        src += 24;
        dsto = dsto.wrapping_add(2);
        let fill = self.tile_word(src, 0);
        self.ram[DUNG_DRAW_WIDTH_INDICATOR] = 18;
        for x in 0..18u16 {
            for y in 0..6u16 {
                self.room_write_bg(0x2000, dsto + x + y * 64, fill);
            }
        }
        self.RoomDraw_ExplodingWallColumn(src + 2, dsto + 18);
    }

    pub(super) fn RoomDraw_ExplodingWallColumn(&mut self, src: usize, dsto: u16) {
        for i in 0..6u16 {
            self.room_write_current(dsto + i * 64, self.tile_word(src, i as usize));
            self.room_write_current(dsto + 1 + i * 64, self.tile_word(src, (i + 6) as usize));
        }
    }

    pub(super) fn RoomDraw_CheckIfLowerLayerDoors_Y(&mut self, door_type: u8, dsto: u16) {
        if door_type == DOOR_TYPE_REGULAR2 {
            self.RoomDraw_MakeDoorPartsHighPriority_Y(dsto + xy(0, 4) as u16);
            self.Door_Draw_Helper4(door_type, dsto);
        } else if door_type == DOOR_TYPE_WATERFALL_TUNNEL {
            self.Door_Draw_Helper4(door_type, dsto);
            self.Door_PrioritizeCurDoor();
        } else {
            self.Door_Draw_Helper4(door_type, dsto);
        }
    }

    pub(super) fn RoomDraw_NormalRangedDoors_West(
        &mut self,
        door_type: u8,
        dsto: u16,
        position: usize,
    ) {
        if position >= 6 {
            if let Some(&right_dsto_bytes) = DOOR_POSITION_RIGHT.get(position - 6) {
                let saved = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX);
                write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, saved | 0x10);
                self.RoomDraw_NormalRangedDoors_East(door_type, right_dsto_bytes / 2);
                write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, saved);
            }
        }

        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(2, door_type, dsto);
        if t & 0x100 != 0 {
            return;
        }
        if t as u8 == DOOR_TYPE_36 || t as u8 == DOOR_TYPE_38 {
            let new_type = if t as u8 == DOOR_TYPE_36 {
                DOOR_TYPE_SHUTTERS_TWO_WAY
            } else {
                DOOR_TYPE_REGULAR
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        if let Some(&src) = DOOR_TYPE_SRC_LEFT.get(t as usize >> 1) {
            self.RoomData_DrawObject_Door_left_3x4(src as usize, dsto);
        }
    }

    pub(super) fn RoomDraw_NormalRangedDoors_East(&mut self, door_type: u8, dsto: u16) {
        if door_type == DOOR_TYPE_REGULAR2 {
            self.RoomDraw_MakeDoorPartsHighPriority_X(dsto + xy(4, 0) as u16);
        }
        if door_type == DOOR_TYPE_WATERFALL_TUNNEL {
            self.RoomDraw_OneSidedShutters_East(door_type, dsto);
            self.Door_PrioritizeCurDoor();
        } else {
            self.RoomDraw_OneSidedShutters_East(door_type, dsto);
        }
    }

    pub(super) fn RoomDraw_OneSidedShutters_East(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(3, door_type, dsto);
        if t & 0x100 != 0 {
            return;
        }
        if t as u8 == DOOR_TYPE_36 || t as u8 == DOOR_TYPE_38 {
            let new_type = if t as u8 == DOOR_TYPE_36 {
                DOOR_TYPE_REGULAR
            } else {
                DOOR_TYPE_SHUTTERS_TWO_WAY
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        if let Some(&src) = DOOR_TYPE_SRC_RIGHT.get(t as usize >> 1) {
            self.RoomData_DrawObject_Door_right_3x4(src as usize, dsto);
        }
    }

    pub(super) fn room_rewrite_last_door_type(&mut self, door_type: u8) {
        let index = (read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX).wrapping_sub(2) >> 1) as usize;
        if index < 16 {
            write_le_u16(
                &mut self.ram,
                DOOR_TYPE_AND_SLOT + index * 2,
                ((index as u16) << 8) | door_type as u16,
            );
        }
    }

    pub(super) fn RoomDraw_FlagDoorsAndGetFinalType(
        &mut self,
        direction: u16,
        door_type: u8,
        dsto: u16,
    ) -> u16 {
        let slot = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX) as usize >> 1;
        if slot < 16 {
            write_le_u16(&mut self.ram, DUNG_DOOR_DIRECTION + slot * 2, direction);
            write_le_u16(
                &mut self.ram,
                DUNG_DOOR_TILEMAP_ADDRESS + slot * 2,
                dsto * 2,
            );
            write_le_u16(
                &mut self.ram,
                DOOR_TYPE_AND_SLOT + slot * 2,
                ((slot as u16) << 8) | door_type as u16,
            );
        }
        let mut remapped = door_type;
        if (slot & 7) < 4
            && read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) & upper_bitmask(slot & 7) != 0
        {
            let is_shutter =
                door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER;
            if !(is_shutter && read_le_u16(&self.ram, DUNG_FLAG_TRAPDOORS_DOWN) != 0) {
                remapped = DOOR_TYPE_REMAP
                    .get(door_type as usize >> 1)
                    .copied()
                    .unwrap_or(door_type);
                if !is_shutter
                    && door_type >= DOOR_TYPE_INVISIBLE_DOOR
                    && door_type != DOOR_TYPE_REGULAR_DOOR33
                    && door_type != DOOR_TYPE_WARP_ROOM_DOOR
                {
                    let opened = read_le_u16(&self.ram, DUNG_DOOR_OPENED) | upper_bitmask(slot);
                    write_le_u16(&mut self.ram, DUNG_DOOR_OPENED, opened);
                }
            }
        }
        write_le_u16(
            &mut self.ram,
            DUNG_CUR_DOOR_IDX,
            (slot as u16).wrapping_mul(2).wrapping_add(2),
        );

        if remapped == DOOR_TYPE_SLASHABLE || remapped == DOOR_TYPE_WATERFALL_TUNNEL {
            return 0x100 | remapped as u16;
        }
        if door_type != DOOR_TYPE_INVISIBLE_DOOR {
            return remapped as u16;
        }

        write_le_u16(
            &mut self.ram,
            INVISIBLE_DOOR_DIR_AND_INDEX_X2,
            (((slot as u16) << 8) | direction) * 2,
        );
        let opened = read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) | upper_bitmask(slot);
        write_le_u16(&mut self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, opened);
        DOOR_TYPE_REGULAR as u16
    }

    pub(super) fn RoomDraw_MarkDungeonToggleDoor(&mut self, dsto: u16) {
        let index = read_le_u16(&self.ram, DUNG_NUM_TOGGLE_PALACE) as usize >> 1;
        if index < 8 {
            write_le_u16(&mut self.ram, DUNG_TOGGLE_PALACE_POS + index * 2, dsto);
        }
        let next = read_le_u16(&self.ram, DUNG_NUM_TOGGLE_PALACE).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_NUM_TOGGLE_PALACE, next);
    }

    pub(super) fn RoomDraw_MarkLayerToggleDoor(&mut self, dsto: u16) {
        let index = read_le_u16(&self.ram, DUNG_NUM_TOGGLE_FLOOR) as usize >> 1;
        if index < 8 {
            write_le_u16(&mut self.ram, DUNG_TOGGLE_FLOOR_POS + index * 2, dsto);
        }
        let next = read_le_u16(&self.ram, DUNG_NUM_TOGGLE_FLOOR).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_NUM_TOGGLE_FLOOR, next);
    }

    pub(super) fn RoomData_DrawObject_Door_up_4x3(&mut self, src: usize, dsto: u16) {
        for x in 0..4 {
            for y in 0..3 {
                let tile = self.tile_word(src, (x * 3 + y) as usize);
                self.room_write_current(dsto + x + y * 64, tile);
            }
        }
    }

    pub(super) fn RoomData_DrawObject_Door_down_4x3(&mut self, src: usize, dsto: u16) {
        for x in 0..4 {
            for y in 0..3 {
                let tile = self.tile_word(src, (x * 3 + y) as usize);
                self.room_write_current(dsto + x + (y + 1) * 64, tile);
            }
        }
    }

    pub(super) fn RoomData_DrawObject_Door_left_3x4(&mut self, src: usize, dsto: u16) {
        for x in 0..3 {
            for y in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
    }

    pub(super) fn RoomData_DrawObject_Door_right_3x4(&mut self, src: usize, dsto: u16) {
        for x in 0..3 {
            for y in 0..4 {
                self.room_write_current(
                    dsto + 1 + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
    }

    pub(super) fn Door_PrioritizeCurDoor(&mut self) {
        let index = (read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX).wrapping_sub(2) >> 1) as usize;
        if index < 16 {
            let addr = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + index * 2) | 0x2000;
            write_le_u16(&mut self.ram, DUNG_DOOR_TILEMAP_ADDRESS + index * 2, addr);
        }
    }

    pub(super) fn RoomDraw_NorthCurtainDoor(&mut self, dsto: u16) {
        let rv = self.RoomDraw_FlagDoorsAndGetFinalType(0, DOOR_TYPE_SLASHABLE, dsto);
        let src = if rv & 0x100 != 0 {
            0x078a
        } else {
            DOOR_TYPE_SRC_UP
                .get(rv as usize >> 1)
                .copied()
                .unwrap_or(0x078a) as usize
        };
        self.RoomDraw_4x4(src, dsto);
    }

    pub(super) fn RoomDraw_HighRangeDoor_North(
        &mut self,
        door_type: u8,
        dsto: u16,
        position: usize,
    ) {
        if position >= 6 && door_type != DOOR_TYPE_WARP_ROOM_DOOR {
            if let Some(&down_dsto_bytes) = DOOR_POSITION_DOWN.get(position - 6) {
                let saved = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX);
                write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, saved | 0x10);
                self.RoomDraw_OneSidedLowerShutters_South(door_type, down_dsto_bytes / 2);
                write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, saved);
            }
        }
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(0, door_type, dsto);
        if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR || t as u8 == DOOR_TYPE_SHUTTER_TRAP_DL {
            let new_type = if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR {
                DOOR_TYPE_REGULAR_DOOR33
            } else {
                DOOR_TYPE_SHUTTER
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        let src = DOOR_TYPE_SRC_UP.get(t as usize >> 1).copied().unwrap_or(0) as usize;
        for x in 0..4 {
            let d = dsto + x;
            self.room_write_bg(0x2000, d, self.tile_word(src, x as usize * 3));
            self.room_write_bg(0x4000, d + 64, self.tile_word(src, x as usize * 3 + 1));
            self.room_write_bg(0x4000, d + 128, self.tile_word(src, x as usize * 3 + 2));
        }
        if door_type != DOOR_TYPE_WARP_ROOM_DOOR {
            self.RoomDraw_MakeDoorHighPriority_North(dsto);
        }
        self.Door_PrioritizeCurDoor();
    }

    pub(super) fn RoomDraw_OneSidedLowerShutters_South(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, dsto);
        if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR || t as u8 == DOOR_TYPE_SHUTTER_TRAP_DL {
            let new_type = if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR {
                DOOR_TYPE_SHUTTER
            } else {
                DOOR_TYPE_REGULAR_DOOR33
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        let src = DOOR_TYPE_SRC_DOWN
            .get(t as usize >> 1)
            .copied()
            .unwrap_or(0) as usize;
        for x in 0..4 {
            let d = dsto + x;
            self.room_write_bg(0x4000, d + 64, self.tile_word(src, x as usize * 3));
            self.room_write_bg(0x4000, d + 128, self.tile_word(src, x as usize * 3 + 1));
            self.room_write_bg(0x2000, d + 192, self.tile_word(src, x as usize * 3 + 2));
        }
        self.RoomDraw_MakeDoorHighPriority_South(dsto + xy(0, 4) as u16);
        self.Door_PrioritizeCurDoor();
    }

    pub(super) fn RoomDraw_HighRangeDoor_West(
        &mut self,
        door_type: u8,
        dsto: u16,
        position: usize,
    ) {
        if position >= 6 {
            if let Some(&right_dsto_bytes) = DOOR_POSITION_RIGHT.get(position - 6) {
                let saved = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX);
                write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, saved | 0x10);
                self.RoomDraw_OneSidedLowerShutters_East(door_type, right_dsto_bytes / 2);
                write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, saved);
            }
        }
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(2, door_type, dsto);
        if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR || t as u8 == DOOR_TYPE_SHUTTER_TRAP_DL {
            let new_type = if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR {
                DOOR_TYPE_SHUTTER
            } else {
                DOOR_TYPE_REGULAR_DOOR33
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        let src = DOOR_TYPE_SRC_LEFT
            .get(t as usize >> 1)
            .copied()
            .unwrap_or(0) as usize;
        for y in 0..4 {
            self.room_write_bg(0x2000, dsto + y * 64, self.tile_word(src, y as usize));
        }
        for x in 1..3 {
            for y in 0..4 {
                self.room_write_bg(
                    0x4000,
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
        self.RoomDraw_MakeDoorHighPriority_West(dsto);
        self.Door_PrioritizeCurDoor();
    }

    pub(super) fn RoomDraw_OneSidedLowerShutters_East(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(3, door_type, dsto);
        if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR || t as u8 == DOOR_TYPE_SHUTTER_TRAP_DL {
            let new_type = if t as u8 == DOOR_TYPE_SHUTTER_TRAP_UR {
                DOOR_TYPE_REGULAR_DOOR33
            } else {
                DOOR_TYPE_SHUTTER
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }
        let src = DOOR_TYPE_SRC_RIGHT
            .get(t as usize >> 1)
            .copied()
            .unwrap_or(0) as usize;
        for x in 0..2 {
            for y in 0..4 {
                self.room_write_bg(
                    0x4000,
                    dsto + 1 + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
        for y in 0..4 {
            self.room_write_bg(
                0x2000,
                dsto + 3 + y * 64,
                self.tile_word(src, 8 + y as usize),
            );
        }
        self.RoomDraw_MakeDoorHighPriority_East(dsto + xy(4, 0) as u16);
        self.Door_PrioritizeCurDoor();
    }

    pub(super) fn RoomDraw_MakeDoorHighPriority_North(&mut self, dsto_org: u16) {
        let mut dsto = dsto_org & (0xf07f >> 1);
        loop {
            for x in 0..4 {
                let tile = self.room_read_bg(0x2000, dsto + x) | 0x2000;
                self.room_write_bg(0x2000, dsto + x, tile);
            }
            dsto += 64;
            if dsto == dsto_org {
                break;
            }
        }
    }

    pub(super) fn RoomDraw_MakeDoorHighPriority_South(&mut self, mut dsto: u16) {
        loop {
            for x in 0..4 {
                let tile = self.room_read_bg(0x2000, dsto + x) | 0x2000;
                self.room_write_bg(0x2000, dsto + x, tile);
            }
            dsto += 64;
            if dsto & 0x07c0 == 0 {
                break;
            }
        }
    }

    pub(super) fn RoomDraw_MakeDoorHighPriority_West(&mut self, dsto_org: u16) {
        let mut dsto = dsto_org & 0xffe0;
        loop {
            for y in 0..4 {
                let pos = dsto + y * 64;
                let tile = self.room_read_bg(0x2000, pos) | 0x2000;
                self.room_write_bg(0x2000, pos, tile);
            }
            dsto += 1;
            if dsto == dsto_org {
                break;
            }
        }
    }

    pub(super) fn RoomDraw_MakeDoorHighPriority_East(&mut self, mut dsto: u16) {
        loop {
            for y in 0..4 {
                let pos = dsto + y * 64;
                let tile = self.room_read_bg(0x2000, pos) | 0x2000;
                self.room_write_bg(0x2000, pos, tile);
            }
            dsto += 1;
            if dsto & 0x1f == 0 {
                break;
            }
        }
    }

    pub(super) fn RoomDraw_MakeDoorPartsHighPriority_Y(&mut self, dsto: u16) {
        for y in 0..7 {
            for x in 0..4 {
                let pos = dsto + x + y * 64;
                let tile = self.room_read_bg(0x2000, pos) | 0x2000;
                self.room_write_bg(0x2000, pos, tile);
            }
        }
    }

    pub(super) fn RoomDraw_MakeDoorPartsHighPriority_X(&mut self, dsto: u16) {
        for x in 0..5 {
            for y in 0..4 {
                let pos = dsto + x + y * 64;
                let tile = self.room_read_bg(0x2000, pos) | 0x2000;
                self.room_write_bg(0x2000, pos, tile);
            }
        }
    }

    pub(super) fn room_prioritize_throne_room_door_edge(&mut self) {
        for dsto in [0x0ede, 0x0f1e, 0x0f5e] {
            let tile = self.room_read_bg(0x2000, dsto);
            self.room_write_bg(0x2000, dsto, tile | 0x2000);
        }
    }

    pub(super) fn room_plane_offset(&self) -> u16 {
        if read_le_u16(&self.ram, DUNG_LINE_PTRS_ROW0) == 0x4000 {
            0x1000
        } else {
            0
        }
    }

    pub(super) fn room_plane_tilemap_bit(&self) -> u16 {
        if read_le_u16(&self.ram, DUNG_LINE_PTRS_ROW0) == 0x4000 {
            0x2000
        } else {
            0
        }
    }

    pub(super) fn RoomDraw_1x3_rightwards(&mut self, src: usize, dsto: u16, columns: u16) {
        for x in 0..columns {
            for y in 0..3 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 3 + y) as usize),
                );
            }
        }
    }

    pub(super) fn Object_Draw_5x4(&mut self, src: usize, dsto: u16) {
        for y in 0..5 {
            for x in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    pub(super) fn RoomDraw_RightwardShelfEnd<'a>(
        &mut self,
        src: usize,
        dst: &'a mut u16,
    ) -> &'a mut u16 {
        let dsto = *dst;
        for y in 0..4 {
            self.room_write_current(dsto + y * 64, self.tile_word(src, y as usize));
        }
        dst
    }

    pub(super) fn RoomDraw_RightwardBarSegment(&mut self, src: usize, dsto: u16) -> u16 {
        for y in 0..3 {
            self.room_write_current(dsto + y * 64, self.tile_word(src, y as usize));
        }
        dsto
    }

    pub(super) fn RoomDraw_DrawObject2x2and1(&mut self, src: usize, dsto: u16) -> u16 {
        for y in 0..5 {
            self.room_write_current(dsto + y * 64, self.tile_word(src, y as usize));
        }
        dsto
    }

    #[track_caller]
    pub(super) fn RoomDraw_Downwards4x2VariableSpacing(
        &mut self,
        increment: u16,
        src: usize,
        dsto: u16,
        count: u16,
    ) {
        let mut dst = dsto;
        for _ in 0..count {
            for x in 0..4 {
                for y in 0..2 {
                    self.room_write_current(
                        dst + x + y * 64,
                        self.tile_word(src, (y * 4 + x) as usize),
                    );
                }
            }
            dst = dst.wrapping_add(increment);
        }
        self.ram[DUNG_DRAW_WIDTH_INDICATOR] = 0;
    }

    pub(super) fn Object_Table_Helper(&mut self, src: usize, dsto: u16, width: u16) {
        self.room_write_current(dsto, self.tile_word(src, 0));
        for segment in 0..width {
            let dst = dsto + 1 + segment * 2;
            self.room_write_current(dst, self.tile_word(src, 1));
            self.room_write_current(dst + 1, self.tile_word(src, 2));
        }
        self.room_write_current(dsto + 1 + width * 2, self.tile_word(src, 3));
    }

    pub(super) fn RoomDraw_CheckIfWallIsMoved(&mut self) -> bool {
        self.ram[BG1_MOVE_CALC_BUFFER] = 0;
        self.ram[BG1_MOVE_CALC_BUFFER + 1] = 0;
        write_le_u16(&mut self.ram, DUNG_FLOOR_MOVE_FLAGS, 0);

        let tag0 = self.ram[DUNG_HDR_TAG];
        let tag1 = self.ram[DUNG_HDR_TAG + 1];
        let i = if (0x1c..0x20).contains(&tag0) {
            Some(0usize)
        } else if (0x1c..0x20).contains(&tag1) {
            Some(1usize)
        } else {
            None
        };

        if let Some(i) = i {
            if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & (0x1000 >> i) != 0 {
                self.ram[DUNG_HDR_COLLISION] = 0;
                self.ram[DUNG_HDR_TAG + i] = 0;
                self.ram[DUNG_HDR_BG2_PROPERTIES] = 0;
                return false;
            }
        }
        true
    }

    pub(super) fn MovingWall_FillReplacementBuffer(&mut self, dsto: u16) {
        for i in 0..64 {
            write_le_u16(&mut self.ram, MOVING_WALL_ARR1 + i * 2, 0x01ec);
        }
        let value = (dsto & 0x001f) | if dsto & 0x0020 != 0 { 0x0400 } else { 0 } | 0x1000;
        write_le_u16(&mut self.ram, MOVING_WALL_WRITE_POINT, value);
    }

    pub(super) fn RoomDraw_MovingWallRight(&mut self, width: u8, height: u8, dsto: u16) {
        const SIZES0: [u16; 4] = [5, 7, 11, 15];
        const SIZES1: [u16; 4] = [8, 16, 24, 32];
        if !self.RoomDraw_CheckIfWallIsMoved() {
            return;
        }
        self.ram[DUNG_HDR_COLLISION_2_MIRROR] =
            self.ram[DUNG_HDR_COLLISION_2_MIRROR].wrapping_add(1);
        let size0 = SIZES0[width as usize];
        let size1 = SIZES1[height as usize];
        self.MovingWall_FillReplacementBuffer(dsto.wrapping_sub(size1).wrapping_sub(1));
        self.ram[MOVING_WALL_DOT_POINTER] = height.wrapping_mul(2);

        let fill_src = 0x03d8;
        let mut dst1 = dsto.wrapping_sub(size1);
        for _ in 0..size1 {
            let mut dst2 = dst1;
            self.room_write_current(dst2, self.tile_word(fill_src, 0));
            for _ in 0..(size0 * 2 + 4) {
                self.room_write_current(dst2 + 64, self.tile_word(fill_src, 1));
                dst2 += 64;
            }
            self.room_write_current(dst2 + 64, self.tile_word(fill_src, 2));
            dst1 += 1;
        }

        let src = 0x072a;
        let mut dst = dsto;
        self.RoomDraw_1x3_rightwards(src, dst, 3);
        dst += 3 * 64;
        for _ in 0..size0 {
            self.Object_Draw_3x2(src + 18, dst);
            dst += 2 * 64;
        }
        self.RoomDraw_1x3_rightwards(src + 30, dst, 3);
    }

    pub(super) fn RoomDraw_MovingWallLeft(&mut self, width: u8, height: u8, dsto: u16) {
        const SIZES0: [u16; 4] = [5, 7, 11, 15];
        const SIZES1: [u16; 4] = [8, 16, 24, 32];
        if !self.RoomDraw_CheckIfWallIsMoved() {
            return;
        }
        self.ram[DUNG_HDR_COLLISION_2_MIRROR] =
            self.ram[DUNG_HDR_COLLISION_2_MIRROR].wrapping_add(1);
        let size1 = SIZES1[height as usize];
        let size0 = SIZES0[width as usize];
        self.ram[MOVING_WALL_DOT_POINTER] = height.wrapping_mul(2);
        self.MovingWall_FillReplacementBuffer(dsto.wrapping_add(3).wrapping_add(size1));

        let src = 0x075a;
        let mut dst = dsto;
        self.RoomDraw_1x3_rightwards(src, dst, 3);
        dst += 3 * 64;
        for _ in 0..size0 {
            self.Object_Draw_3x2(src + 18, dst);
            dst += 2 * 64;
        }
        self.RoomDraw_1x3_rightwards(src + 30, dst, 3);

        let fill_src = 0x03d8;
        let mut dst1 = dsto + 3;
        for _ in 0..size1 {
            let mut dst2 = dst1;
            self.room_write_current(dst2, self.tile_word(fill_src, 0));
            for _ in 0..(size0 * 2 + 4) {
                self.room_write_current(dst2 + 64, self.tile_word(fill_src, 1));
                dst2 += 64;
            }
            self.room_write_current(dst2 + 64, self.tile_word(fill_src, 2));
            dst1 += 1;
        }
    }

    pub(super) fn Object_DrawNx3_BothBgs(&mut self, n: u16, src: usize, dsto: u16) {
        for x in 0..n {
            for y in 0..3 {
                let tile = self.tile_word(src, (x * 3 + y) as usize);
                self.room_write_bg(0x2000, dsto + x + y * 64, tile);
                self.room_write_bg(0x4000, dsto + x + y * 64, tile);
            }
        }
    }

    pub(super) fn bomb_check_for_destructibles(&mut self, x: u16, y: u16, r14: u8) {
        const DUNG_CUR_DOOR_POS_DUNGEON: usize = 0x068e;
        const K_DOOR_TYPE_BREAKABLE_WALL: u8 = 0x28;

        if self.frame_control_view().main_module() != 7 {
            self.overworld_bomb_tiles32x32(x, y);
            return;
        }

        let mut k = (((y & 0x01f8) << 3) | ((x & 0x01f8) >> 3)).wrapping_sub(0x0082) as usize;
        for _ in (0..=2).rev() {
            for step in 0..3 {
                let a = self.ram[DUNG_BG2_ATTR_TABLE + k];
                if a == 0x62 {
                    if self.world_state_view().dungeon_room() == 0x65 {
                        let bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) | 0x1000;
                        write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, bits);
                    }
                    let mut pt = Point16U { x: 0, y: 0 };
                    self.ThievesAttic_DrawLightenedHole(0, 0, &mut pt);
                    self.ram[SOUND_EFFECT_2] = 0x1b;
                    return;
                }
                if (a & 0xf0) == 0xf0 {
                    let j = (a & 0x0f) as usize;
                    let ty = self.ram[DOOR_TYPE_AND_SLOT + j * 2] & 0xfe;
                    if ty != K_DOOR_TYPE_BREAKABLE_WALL && ty != 0x2a && ty != 0x2e {
                        return;
                    }
                    write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, k as u16);
                    let addr = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + j * 2);
                    let door_x = ((addr & 0x007e) << 2)
                        .wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_H_COPY));
                    let door_y = ((addr & 0x1f80) >> 4)
                        .wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_V_COPY));
                    let r14 = r14 as usize;
                    write_le_u16(&mut self.ram, DOOR_DEBRIS_X + r14 * 2, door_x);
                    write_le_u16(&mut self.ram, DOOR_DEBRIS_Y + r14 * 2, door_y);
                    self.ram[DOOR_DEBRIS_DIRECTION_DUNGEON + r14] =
                        self.ram[DUNG_DOOR_DIRECTION + j * 2] & 3;
                    self.ram[SOUND_EFFECT_2] = 0x1b;
                    self.frame_control_view_mut().set_submodule(9);
                    return;
                }
                if step != 2 {
                    k = k.wrapping_add(2);
                }
            }
            k = k.wrapping_add(0x7c);
        }
    }

    pub(super) fn prepare_dungeon_exit_from_boss_fight(&mut self) {
        self.SavePalaceDeaths();
        self.SaveDungeonKeys_misc();
        let bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) | 0x8000;
        write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, bits);
        self.Dungeon_FlagRoomData_Quadrants();

        let room = self.ram[DUNGEON_ROOM_INDEX];
        let j = K_DUNGEON_EXIT_FROM
            .iter()
            .position(|&from| from == room)
            .expect("dungeon room must have a boss-exit mapping");
        self.ram[DUNGEON_ROOM_INDEX] = K_DUNGEON_EXIT_TO[j];
        if self.ram[DUNGEON_ROOM_INDEX] == 0x20 {
            self.ram[SRAM_PROGRESS_INDICATOR] = 3;
            self.ram[SAVE_OW_EVENT_INFO_DUNGEON + 2] |= 0x20;
            self.ram[SAVEGAME_IS_DARKWORLD] ^= 0x40;
            self.sprite_load_graphics_properties_light_world_only();
            self.ancilla_terminate_select_interactives(0);
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
            self.ram[BUTTON_B_FRAMES] = 0;
            self.ram[BUTTON_MASK_B_Y] = 0;
            self.ram[LINK_FORCE_HOLD_SWORD_UP] = 0;
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
            self.ram[SAVED_MODULE_FOR_MENU] = 8;
            self.frame_control_view_mut().set_main_module(21);
            self.frame_control_view_mut().set_submodule(0);
            self.frame_control_view_mut().set_subsubmodule(0);
        } else if self.ram[DUNGEON_ROOM_INDEX] == 0x0d {
            self.frame_control_view_mut().set_main_module(24);
            self.frame_control_view_mut().set_submodule(0);
            self.ram[OVERWORLD_MAP_STATE] = 0;
            self.ram[CGADSUB_COPY] = 0x20;
        } else {
            if j >= 3 {
                self.ram[MUSIC_CONTROL] = 0xf1;
                self.ram[CURRENT_MUSIC_CONTROL] = 0xf1;
                self.frame_control_view_mut().set_main_module(22);
            } else {
                self.frame_control_view_mut().set_main_module(19);
            }
            self.ram[SAVED_MODULE_FOR_MENU] = 8;
            self.frame_control_view_mut().set_submodule(0);
            self.frame_control_view_mut().set_subsubmodule(0);
        }
    }

    pub(super) fn Object_BombableFloorHelper(
        &mut self,
        state: u16,
        src: usize,
        src_below: usize,
        _dst: &mut u16,
        dsto: u16,
    ) {
        let index = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX) as usize >> 1;
        write_le_u16(
            &mut self.ram,
            DUNG_REPLACEMENT_TILE_STATE + index * 2,
            state,
        );
        let next = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_MISC_OBJS_INDEX, next);
        let load_ptr = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS);
        write_le_u16(
            &mut self.ram,
            DUNG_OBJECT_POS_IN_OBJDATA + index * 2,
            load_ptr,
        );
        let tilemap_pos = dsto * 2 | self.room_plane_tilemap_bit();
        let below = [
            self.tile_word(src_below, 0),
            self.tile_word(src_below, 1),
            self.tile_word(src_below, 2),
            self.tile_word(src_below, 3),
        ];
        write_le_u16(
            &mut self.ram,
            DUNG_OBJECT_TILEMAP_POS + index * 2,
            tilemap_pos,
        );
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UL + index * 2, below[0]);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LL + index * 2, below[1]);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UR + index * 2, below[2]);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LR + index * 2, below[3]);
        self.RoomDraw_Rightwards2x2(src, dsto);
    }

    pub(super) fn RoomDraw_BombableFloor(&mut self, _src: usize, dst: &mut u16, dsto: u16) {
        if self.world_state_view().dungeon_room() == 101
            && read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x1000 != 0
        {
            self.ram[DUNG_DRAW_WIDTH_INDICATOR] = 0;
            self.ram[DUNG_DRAW_HEIGHT_INDICATOR] = 0;
            self.Object_Hole(0x05aa, *dst, 0, 0);
            return;
        }

        let src = 0x0220;
        let src_below = 0x05ba;
        self.Object_BombableFloorHelper(0x3030, src, src_below, dst, dsto);
        self.Object_BombableFloorHelper(
            0x3131,
            src + 8,
            src_below + 8,
            dst,
            dsto + xy(2, 0) as u16,
        );
        self.Object_BombableFloorHelper(
            0x3232,
            src + 16,
            src_below + 16,
            dst,
            dsto + xy(0, 2) as u16,
        );
        self.Object_BombableFloorHelper(
            0x3333,
            src + 24,
            src_below + 24,
            dst,
            dsto + xy(2, 2) as u16,
        );
    }

    pub(super) fn RoomDraw_HammerPegSingle(&mut self, src: usize, _dst: &mut u16, dsto: u16) {
        let index = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX) as usize >> 1;
        let next = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_MISC_OBJS_INDEX, next);
        write_le_u16(
            &mut self.ram,
            DUNG_REPLACEMENT_TILE_STATE + index * 2,
            0x4040,
        );
        let load_ptr = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS);
        write_le_u16(
            &mut self.ram,
            DUNG_OBJECT_POS_IN_OBJDATA + index * 2,
            load_ptr,
        );
        let plane = if read_le_u16(&self.ram, DUNG_LINE_PTRS_ROW0) != 0x4000 {
            0
        } else {
            0x2000
        };
        write_le_u16(
            &mut self.ram,
            DUNG_OBJECT_TILEMAP_POS + index * 2,
            dsto * 2 | plane,
        );
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UL + index * 2, 0x19d8);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LL + index * 2, 0x19d9);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UR + index * 2, 0x59d8);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LR + index * 2, 0x59d9);
        self.RoomDraw_Rightwards2x2(src, dsto);
    }

    pub(super) fn Object_ChestPlatform_Helper(&mut self, src: usize, dsto: i32) {
        let mut dsto = dsto as usize;
        let t0 = self.tile_word(src, 0);
        let t3 = self.tile_word(src, 3);
        let t6 = self.tile_word(src, 6);
        let t9 = self.tile_word(src, 9);
        let t12 = self.tile_word(src, 12);
        let t15 = self.tile_word(src, 15);
        let t18 = self.tile_word(src, 18);

        write_le_u16(&mut self.ram, DUNG_BG2 + dsto * 2, t0);
        for _ in 0..self.ram[DUNG_DRAW_WIDTH_INDICATOR] {
            write_le_u16(&mut self.ram, DUNG_BG2 + (dsto + 1) * 2, t3);
            dsto += 1;
        }

        write_le_u16(&mut self.ram, DUNG_BG2 + (dsto + 1) * 2, t6);
        write_le_u16(&mut self.ram, DUNG_BG2 + (dsto + 2) * 2, t9);
        write_le_u16(&mut self.ram, DUNG_BG2 + (dsto + 3) * 2, t9);
        write_le_u16(&mut self.ram, DUNG_BG2 + (dsto + 4) * 2, t9);
        write_le_u16(&mut self.ram, DUNG_BG2 + (dsto + 5) * 2, t9);

        write_le_u16(&mut self.ram, DUNG_BG2 + (dsto + 6) * 2, t12);
        for _ in 0..self.ram[DUNG_DRAW_WIDTH_INDICATOR] {
            write_le_u16(&mut self.ram, DUNG_BG2 + (dsto + 7) * 2, t15);
            dsto += 1;
        }

        write_le_u16(&mut self.ram, DUNG_BG2 + (dsto + 7) * 2, t18);
    }

    pub(super) fn RoomDraw_GetObjectSize_1to16(&mut self) {
        self.Object_SizeAtoAplus15(1);
    }

    pub(super) fn Object_SizeAtoAplus15(&mut self, a: u8) {
        self.ram[DUNG_DRAW_WIDTH_INDICATOR] = (self.ram[DUNG_DRAW_WIDTH_INDICATOR] << 2
            | self.ram[DUNG_DRAW_HEIGHT_INDICATOR])
            .wrapping_add(a);
        self.ram[DUNG_DRAW_HEIGHT_INDICATOR] = 0;
    }

    pub(super) fn RoomDraw_GetObjectSize_1to15or26(&mut self) {
        let x = (self.ram[DUNG_DRAW_WIDTH_INDICATOR] << 2) | self.ram[DUNG_DRAW_HEIGHT_INDICATOR];
        self.ram[DUNG_DRAW_WIDTH_INDICATOR] = if x != 0 { x } else { 26 };
    }

    pub(super) fn RoomDraw_GetObjectSize_1to15or32(&mut self) {
        let x = (self.ram[DUNG_DRAW_WIDTH_INDICATOR] << 2) | self.ram[DUNG_DRAW_HEIGHT_INDICATOR];
        self.ram[DUNG_DRAW_WIDTH_INDICATOR] = if x != 0 { x } else { 32 };
    }

    pub(super) fn DrawWaterThing(&mut self, dsto: u16, src: usize) {
        for y in 0..4 {
            for x in 0..4 {
                self.room_write_current(
                    dsto + y * 64 + x,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    pub(super) fn DrawWaterThingBg(&mut self, base: usize, dsto: u16, src: usize) {
        for y in 0..4 {
            for x in 0..4 {
                self.room_write_bg(
                    base,
                    dsto + y * 64 + x,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    pub(super) fn RoomDraw_FortuneTellerRoom(&mut self, dsto: u16) {
        let src_org = 0x202eusize;
        let mut src = src_org;
        let mut d = dsto;

        for _ in 0..6 {
            let tile0 = self.tile_word(src, 0);
            self.room_write_bg(0x2000, d + xy(1, 0) as u16, tile0);
            self.room_write_bg(0x2000, d + xy(2, 0) as u16, tile0);
            self.room_write_bg(0x2000, d + xy(1, 1) as u16, tile0);
            self.room_write_bg(0x2000, d + xy(2, 1) as u16, tile0);
            let tile1 = self.tile_word(src, 1);
            self.room_write_bg(0x2000, d + xy(1, 2) as u16, tile1);
            self.room_write_bg(0x2000, d + xy(2, 2) as u16, tile1 | 0x4000);
            d = d.wrapping_add(xy(2, 0) as u16);
        }
        d = d.wrapping_sub((xy(2, 0) * 6) as u16);

        for _ in 0..3 {
            let tile2 = self.tile_word(src, 2);
            for &off in &[xy(0, 3), xy(2, 3), xy(10, 3), xy(12, 3)] {
                self.room_write_bg(0x2000, d + off as u16, tile2);
            }
            for &off in &[xy(1, 3), xy(3, 3), xy(11, 3), xy(13, 3)] {
                self.room_write_bg(0x2000, d + off as u16, tile2 | 0x4000);
            }
            let tile5 = self.tile_word(src, 5);
            for &off in &[xy(4, 3), xy(6, 3), xy(8, 3)] {
                self.room_write_bg(0x2000, d + off as u16, tile5);
            }
            for &off in &[xy(5, 3), xy(7, 3), xy(9, 3)] {
                self.room_write_bg(0x2000, d + off as u16, tile5 | 0x4000);
            }
            src += 2;
            d = d.wrapping_add(xy(0, 1) as u16);
        }
        d = d.wrapping_sub((xy(0, 1) * 3) as u16);

        let tile5 = self.tile_word(src, 5);
        self.room_write_bg(0x2000, d + xy(0, 0) as u16, tile5);
        self.room_write_bg(0x2000, d + xy(0, 1) as u16, tile5);
        self.room_write_bg(0x2000, d + xy(13, 0) as u16, tile5 | 0x4000);
        self.room_write_bg(0x2000, d + xy(13, 1) as u16, tile5 | 0x4000);
        let tile6 = self.tile_word(src, 6);
        self.room_write_bg(0x2000, d + xy(0, 2) as u16, tile6);
        self.room_write_bg(0x2000, d + xy(13, 2) as u16, tile6 | 0x4000);

        src = src_org;
        for _ in 0..4 {
            let tile10 = self.tile_word(src, 10);
            self.room_write_bg(0x2000, d + xy(3, 10) as u16, tile10);
            self.room_write_bg(0x2000, d + xy(10, 10) as u16, tile10 ^ 0x4000);
            let tile14 = self.tile_word(src, 14);
            self.room_write_bg(0x2000, d + xy(4, 10) as u16, tile14);
            self.room_write_bg(0x2000, d + xy(9, 10) as u16, tile14 ^ 0x4000);
            let tile18 = self.tile_word(src, 18);
            self.room_write_bg(0x2000, d + xy(5, 10) as u16, tile18);
            self.room_write_bg(0x2000, d + xy(8, 10) as u16, tile18 ^ 0x4000);
            let tile22 = self.tile_word(src, 22);
            self.room_write_bg(0x2000, d + xy(6, 10) as u16, tile22);
            self.room_write_bg(0x2000, d + xy(7, 10) as u16, tile22 ^ 0x4000);
            src += 2;
            d = d.wrapping_add(xy(0, 1) as u16);
        }
    }

    pub(super) fn RoomDraw_PrisonCell(&mut self, dsto: u16) {
        let src = 0x1488;
        let dsto = dsto | self.room_plane_offset();
        for i in 0..5 {
            let d = dsto + i;
            self.room_write_bg(0x2000, d + xy(2, 0) as u16, self.tile_word(src, 1));
            self.room_write_bg(0x2000, d + xy(9, 0) as u16, self.tile_word(src, 1));
            self.room_write_bg(0x2000, d + xy(2, 1) as u16, self.tile_word(src, 2));
            self.room_write_bg(0x2000, d + xy(9, 1) as u16, self.tile_word(src, 2) | 0x4000);
            self.room_write_bg(0x2000, d + xy(2, 2) as u16, self.tile_word(src, 4));
            self.room_write_bg(0x2000, d + xy(9, 2) as u16, self.tile_word(src, 4) | 0x4000);
            self.room_write_bg(0x2000, d + xy(2, 3) as u16, self.tile_word(src, 5));
            self.room_write_bg(0x2000, d + xy(9, 3) as u16, self.tile_word(src, 5) | 0x4000);
        }
        self.room_write_bg(0x2000, dsto, self.tile_word(src, 0));
        self.room_write_bg(
            0x2000,
            dsto + xy(15, 0) as u16,
            self.tile_word(src, 0) | 0x4000,
        );
        for offset in [xy(1, 0), xy(7, 0), xy(8, 0), xy(14, 0)] {
            self.room_write_bg(0x2000, dsto + offset as u16, self.tile_word(src, 1));
        }
        self.room_write_bg(0x2000, dsto + xy(1, 2) as u16, self.tile_word(src, 3));
        self.room_write_bg(
            0x2000,
            dsto + xy(14, 2) as u16,
            self.tile_word(src, 3) | 0x4000,
        );
    }

    pub(super) fn RoomDraw_CellLock(&mut self, dsto: u16) {
        const CHEST_OPEN_MASKS: [u16; 6] = [0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000];
        let index = read_le_u16(&self.ram, DUNG_NUM_BIGKEY_LOCKS_X2) as usize >> 1;
        write_le_u16(
            &mut self.ram,
            DUNG_NUM_BIGKEY_LOCKS_X2,
            ((index + 1) * 2) as u16,
        );
        if index < CHEST_OPEN_MASKS.len()
            && read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & CHEST_OPEN_MASKS[index] == 0
        {
            write_le_u16(&mut self.ram, DUNG_CHEST_LOCATIONS + index * 2, dsto * 2);
            self.RoomDraw_Rightwards2x2(0x1494, dsto);
        } else if index < 6 {
            write_le_u16(&mut self.ram, DUNG_CHEST_LOCATIONS + index * 2, 0);
        }
    }

    pub(super) fn RoomDraw_LowerDoorStairsUp(
        &mut self,
        mut src: usize,
        mut dsto: u16,
        from_upnorth: bool,
    ) {
        let counter = if from_upnorth {
            DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS
        } else {
            DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS
        };
        let index = read_le_u16(&self.ram, counter) as usize >> 1;
        let plane = self.room_plane_offset();
        write_le_u16(
            &mut self.ram,
            DUNG_INTER_STARCASES + index * 2,
            dsto | plane,
        );
        let next = read_le_u16(&self.ram, counter).wrapping_add(2);
        if from_upnorth {
            for offset in [
                DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS,
                DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS,
                DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS,
                DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS,
                DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2,
                DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
            ] {
                write_le_u16(&mut self.ram, offset, next);
            }
        } else {
            write_le_u16(
                &mut self.ram,
                DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                next,
            );
            write_le_u16(
                &mut self.ram,
                DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                next,
            );
        }

        for _ in 0..4 {
            self.room_write_bg(0x4000, dsto, self.tile_word(src, 0));
            self.room_write_bg(0x2000, dsto, self.tile_word(src, 0));
            self.room_write_bg(0x4000, dsto + 64, self.tile_word(src, 1));
            self.room_write_bg(0x4000, dsto + 2 * 64, self.tile_word(src, 2));
            self.room_write_bg(0x4000, dsto + 3 * 64, self.tile_word(src, 3));
            src += 8;
            dsto = dsto.wrapping_add(1);
        }
        let priority = dsto.wrapping_sub(4).wrapping_sub(4 * 64);
        self.RoomDraw_LowerDoorBg2Priority(priority);
    }

    pub(super) fn RoomDraw_LowerDoorStairsDown(
        &mut self,
        mut src: usize,
        mut dsto: u16,
        from_upsouth: bool,
    ) {
        let counter = if from_upsouth {
            DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS
        } else {
            DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS
        };
        let index = read_le_u16(&self.ram, counter) as usize >> 1;
        let plane = self.room_plane_offset();
        write_le_u16(
            &mut self.ram,
            DUNG_INTER_STARCASES + index * 2,
            dsto | plane,
        );
        let next = read_le_u16(&self.ram, counter).wrapping_add(2);
        if from_upsouth {
            for offset in [
                DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS,
                DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS,
                DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS,
                DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2,
                DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS,
                DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
            ] {
                write_le_u16(&mut self.ram, offset, next);
            }
        } else {
            write_le_u16(
                &mut self.ram,
                DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS,
                next,
            );
        }

        for _ in 0..4 {
            self.room_write_bg(0x4000, dsto, self.tile_word(src, 0));
            self.room_write_bg(0x4000, dsto + 64, self.tile_word(src, 1));
            self.room_write_bg(0x4000, dsto + 2 * 64, self.tile_word(src, 2));
            self.room_write_bg(0x4000, dsto + 3 * 64, self.tile_word(src, 3));
            self.room_write_bg(0x2000, dsto + 3 * 64, self.tile_word(src, 3));
            src += 8;
            dsto = dsto.wrapping_add(1);
        }
        let priority = dsto.wrapping_sub(4).wrapping_add(4 * 64);
        self.RoomDraw_LowerDoorBg2Priority(priority);
    }

    pub(super) fn RoomDraw_LowerDoorBg2Priority(&mut self, dsto: u16) {
        for y in 0..4 {
            let pos = dsto + y * 64;
            let tile = self.room_read_bg(0x2000, pos) | 0x2000;
            self.room_write_bg(0x2000, pos, tile);
        }
    }

    pub(super) fn RoomDraw_Chest(&mut self, dsto: u16) {
        const CHEST_OPEN_MASKS: [u16; 6] = [0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000];
        if self.frame_control_view().main_module() == 26 {
            return;
        }
        let index = read_le_u16(&self.ram, DUNG_NUM_CHESTS_X2) as usize >> 1;
        let next = ((index + 1) * 2) as u16;
        write_le_u16(&mut self.ram, DUNG_NUM_CHESTS_X2, next);
        write_le_u16(&mut self.ram, DUNG_NUM_BIGKEY_LOCKS_X2, next);
        if index >= CHEST_OPEN_MASKS.len() {
            return;
        }
        let location = 2 * (dsto | self.room_plane_offset());
        write_le_u16(&mut self.ram, DUNG_CHEST_LOCATIONS + index * 2, location);
        let tag_slot = self.chest_tag_gate_slot();
        if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & CHEST_OPEN_MASKS[index] == 0 {
            if let Some(slot) = tag_slot {
                if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & CHEST_OPEN_MASKS[slot] == 0 {
                    return;
                }
                self.ram[DUNG_HDR_TAG + slot] = 0;
            }
            self.RoomDraw_Rightwards2x2(0x149c, dsto);
        } else {
            write_le_u16(&mut self.ram, DUNG_CHEST_LOCATIONS + index * 2, 0);
            if let Some(slot) = tag_slot {
                self.ram[DUNG_HDR_TAG + slot] = 0;
            }
            self.RoomDraw_Rightwards2x2(0x14a4, dsto);
        }
    }

    pub(super) fn chest_tag_gate_slot(&self) -> Option<usize> {
        for slot in 0..2 {
            let tag = self.ram[DUNG_HDR_TAG + slot];
            if tag == 0x27 || tag == 0x3c || tag == 0x3e || (0x29..0x33).contains(&tag) {
                return Some(slot);
            }
        }
        None
    }

    pub(super) fn RoomDraw_SinglePot(&mut self, src: usize, _dst: &mut u16, dsto: u16) {
        let index = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX) as usize >> 1;
        write_le_u16(
            &mut self.ram,
            DUNG_MISC_OBJS_INDEX,
            ((index + 1) * 2) as u16,
        );
        write_le_u16(
            &mut self.ram,
            DUNG_REPLACEMENT_TILE_STATE + index * 2,
            0x1111,
        );
        let load_ptr = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS);
        write_le_u16(
            &mut self.ram,
            DUNG_OBJECT_POS_IN_OBJDATA + index * 2,
            load_ptr,
        );
        let plane_bit = self.room_plane_tilemap_bit();
        write_le_u16(
            &mut self.ram,
            DUNG_OBJECT_TILEMAP_POS + index * 2,
            dsto * 2 | plane_bit,
        );
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UL + index * 2, 0x0d0e);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LL + index * 2, 0x0d1e);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UR + index * 2, 0x4d0e);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LR + index * 2, 0x4d1e);
        let src = if self.ram[SAVEGAME_IS_DARKWORLD] != 0 {
            0x0e92
        } else {
            src
        };
        self.RoomDraw_Rightwards2x2(src, dsto);
    }

    pub(super) fn Object_Draw_4x2(&mut self, src: usize, dsto: u16) {
        for y in 0..2 {
            for x in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    pub(super) fn Object_Draw_4x2_BothBgs(&mut self, src: usize, dsto: u16) {
        for y in 0..2 {
            for x in 0..4 {
                let tile = self.tile_word(src, (y * 4 + x) as usize);
                self.room_write_bg(0x2000, dsto + x + y * 64, tile);
                self.room_write_bg(0x4000, dsto + x + y * 64, tile);
            }
        }
    }

    pub(super) fn RoomDraw_Chest_platform_row(&mut self, src: usize, dsto: u16, width: u16) {
        self.room_write_bg(0x2000, dsto, self.tile_word(src, 0));
        let left_fill = self.tile_word(src, 3);
        for x in 0..width {
            self.room_write_bg(0x2000, dsto + 1 + x, left_fill);
        }
        self.room_write_bg(0x2000, dsto + 1 + width, self.tile_word(src, 6));

        let middle_fill = self.tile_word(src, 9);
        for x in 0..4 {
            self.room_write_bg(0x2000, dsto + 2 + width + x, middle_fill);
        }
        self.room_write_bg(0x2000, dsto + 6 + width, self.tile_word(src, 12));

        let right_fill = self.tile_word(src, 15);
        for x in 0..width {
            self.room_write_bg(0x2000, dsto + 7 + width + x, right_fill);
        }
        self.room_write_bg(0x2000, dsto + 7 + width * 2, self.tile_word(src, 18));
    }

    pub(super) fn RoomDraw_4x4(&mut self, src: usize, dsto: u16) {
        self.RoomData_DrawObject_nx4(src, dsto, 4);
    }

    pub(super) fn Object_Draw8x8(&mut self, src: usize, dsto: u16) {
        self.RoomDraw_4x4(src, dsto);
        self.RoomDraw_4x4(src + 32, dsto + 4);
        self.RoomDraw_4x4(src + 64, dsto + 4 * 64);
        self.RoomDraw_4x4(src + 96, dsto + 4 + 4 * 64);
    }

    pub(super) fn Object_Draw_3x2(&mut self, src: usize, dsto: u16) {
        for y in 0..2 {
            for x in 0..3 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (y * 3 + x) as usize),
                );
            }
        }
    }

    pub(super) fn RoomDraw_WaterHoldingObject(&mut self, n: u16, src: usize, dsto: u16) {
        for y in 0..n {
            for x in 0..4 {
                self.room_write_current(
                    dsto + y * 64 + x,
                    self.tile_word(src, (y * 4 + x) as usize),
                );
            }
        }
    }

    pub(super) fn RoomDraw_SomeBigDecors(&mut self, n: u16, src: usize, dsto: u16) {
        let mut dst = dsto | self.room_plane_offset();
        for y in 0..8 {
            for x in 0..n {
                self.room_write_bg(0x2000, dst + x, self.tile_word(src, (y * n + x) as usize));
            }
            dst += 64;
        }
    }

    pub(super) fn RoomDraw_SingleLampCone(&mut self, a: u16, y: usize) {
        for row in 0..12 {
            for col in 0..12 {
                self.room_write_bg(
                    0x4000,
                    a / 2 + row * 64 + col,
                    self.tile_word(y, (row * 12 + col) as usize),
                );
            }
        }
    }

    pub(super) fn RoomDraw_AgahnimsWindows(&mut self, dsto: u16) {
        let mut d = dsto;
        let mut src = 0x1bf2;
        for _ in 0..6 {
            for x in [7, 13, 19] {
                for y in 0..4 {
                    self.room_write_bg(0x2000, d + xy(x, 4 + y) as u16, self.tile_word(src, y));
                }
            }
            src += 8;
            d += 1;
        }
        d -= 6;

        src = 0x1c22;
        for _ in 0..5 {
            let tile = self.tile_word(src, 0);
            for (x, y) in [(2, 10), (3, 9), (4, 8), (5, 7), (6, 6), (7, 5), (8, 4)] {
                self.room_write_bg(0x2000, d + xy(x, y) as u16, tile);
            }
            for (x, y) in [
                (23, 4),
                (24, 5),
                (25, 6),
                (26, 7),
                (27, 8),
                (28, 9),
                (29, 10),
            ] {
                self.room_write_bg(0x2000, d + xy(x, y) as u16, tile | 0x4000);
            }
            src += 2;
            d += 64;
        }
        d -= 64 * 5;

        src = 0x1c2c;
        for _ in 0..6 {
            for k in 0..4 {
                let tile = self.tile_word(src, k);
                for y in [11, 17, 23] {
                    self.room_write_bg(0x2000, d + xy(2 + k, y) as u16, tile);
                    self.room_write_bg(0x2000, d + xy(29 - k, y) as u16, tile | 0x4000);
                }
            }
            src += 8;
            d += 64;
        }
        d -= 64 * 6;

        src = 0x1c5c;
        for _ in 0..6 {
            let top = self.tile_word(src, 0);
            let bottom = self.tile_word(src, 6);
            self.room_write_bg(0x2000, d + xy(12, 9) as u16, top);
            self.room_write_bg(0x2000, d + xy(18, 9) as u16, top);
            self.room_write_bg(0x2000, d + xy(12, 10) as u16, bottom);
            self.room_write_bg(0x2000, d + xy(18, 10) as u16, bottom);
            src += 2;
            d += 1;
        }
        d -= 6;

        src = 0x1c74;
        for _ in 0..6 {
            self.room_write_bg(0x2000, d + xy(7, 14) as u16, self.tile_word(src, 0));
            self.room_write_bg(0x2000, d + xy(7, 20) as u16, self.tile_word(src, 0));
            self.room_write_bg(0x2000, d + xy(8, 14) as u16, self.tile_word(src, 1));
            self.room_write_bg(0x2000, d + xy(8, 20) as u16, self.tile_word(src, 1));
            src += 4;
            d += 64;
        }
        d -= 64 * 6;

        src = 0x1c8c;
        for _ in 0..5 {
            for y in 0..5 {
                self.room_write_bg(0x2000, d + xy(7, 9 + y) as u16, self.tile_word(src, y));
            }
            src += 10;
            d += 1;
        }
        d -= 5;

        for _ in 0..4 {
            let pos0 = d + xy(14, 28) as u16;
            let pos1 = d + xy(14, 29) as u16;
            let tile0 = self.room_read_bg(0x2000, pos0) | 0x2000;
            let tile1 = self.room_read_bg(0x2000, pos1) | 0x2000;
            self.room_write_bg(0x2000, pos0, tile0);
            self.room_write_bg(0x2000, pos1, tile1);
            d += 1;
        }
    }

    pub(super) fn RoomDraw_AgahnimAltar(&mut self, dsto: u16) {
        let base = 0x2000;
        for y in 0..14 {
            let row = dsto + y * 64;
            let src = 0x1b4a + y as usize * 2;
            let tile0 = self.tile_word(src, 0);
            self.room_write_bg(base, row, tile0);
            self.room_write_bg(base, row + 13, tile0 | 0x4000);

            let tile1 = self.tile_word(src, 14);
            self.room_write_bg(base, row + 1, tile1);
            self.room_write_bg(base, row + 2, tile1);
            self.room_write_bg(base, row + 11, tile1 ^ 0x4000);
            self.room_write_bg(base, row + 12, tile1 ^ 0x4000);

            for x in 3..=6 {
                let tile = self.tile_word(src, (x - 1) * 14);
                self.room_write_bg(base, row + x as u16, tile);
                self.room_write_bg(base, row + (13 - x) as u16, tile ^ 0x4000);
            }
        }
    }

    pub(super) fn RoomDraw_A_Many32x32Blocks(&mut self, mut n: i32, src: usize, dst: &mut u16) {
        loop {
            for _ in 0..2 {
                for y in 0..2 {
                    for x in 0..4 {
                        let tile = self.tile_word(src, (y * 4 + x) as usize);
                        self.room_write_current(*dst + xy(x as usize, y as usize) as u16, tile);
                    }
                }
                *dst += xy(0, 2) as u16;
            }
            *dst = dst
                .wrapping_add(xy(4, 0) as u16)
                .wrapping_sub(xy(0, 4) as u16);
            n -= 1;
            if n == 0 {
                break;
            }
        }
    }

    pub(super) fn RoomData_DrawObject_nx4(&mut self, src: usize, dsto: u16, columns: u16) {
        for x in 0..columns {
            for y in 0..4 {
                self.room_write_current(
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
    }

    pub(super) fn RoomDraw_Object_Nx4(&mut self, n: u16, src: usize, dsto: u16) {
        self.RoomData_DrawObject_nx4(src, dsto, n);
    }

    pub(super) fn RoomDraw_Object_Nx4_Bg2(&mut self, n: u16, src: usize, dsto: u16) {
        for x in 0..n {
            for y in 0..4 {
                self.room_write_bg(
                    DUNG_BG2,
                    dsto + x + y * 64,
                    self.tile_word(src, (x * 4 + y) as usize),
                );
            }
        }
    }

    pub(super) fn Object_DrawNx4_BothBgs(&mut self, n: u16, src: usize, dsto: u16) {
        self.RoomData_DrawObject_nx4_both_bgs(src, dsto, n);
    }

    pub(super) fn write_stairs_table(
        &mut self,
        table_offset: usize,
        count_offset: usize,
        dsto: u16,
    ) -> u16 {
        let index = read_le_u16(&self.ram, count_offset) as usize >> 1;
        write_le_u16(&mut self.ram, table_offset + index * 2, dsto);
        let next = read_le_u16(&self.ram, count_offset).wrapping_add(2);
        write_le_u16(&mut self.ram, count_offset, next);
        next
    }

    pub(super) fn RoomData_DrawObject_nx4_both_bgs(&mut self, src: usize, dsto: u16, columns: u16) {
        for x in 0..columns {
            for y in 0..4 {
                let tile = self.tile_word(src, (x * 4 + y) as usize);
                self.room_write_bg(0x2000, dsto + x + y * 64, tile);
                self.room_write_bg(0x4000, dsto + x + y * 64, tile);
            }
        }
    }

    pub(super) fn RoomDraw_Rightwards2x2(&mut self, src: usize, dsto: u16) {
        self.room_write_current(dsto, self.tile_word(src, 0));
        self.room_write_current(dsto + 64, self.tile_word(src, 1));
        self.room_write_current(dsto + 1, self.tile_word(src, 2));
        self.room_write_current(dsto + 65, self.tile_word(src, 3));
    }

    pub(super) fn DrawBigGraySegment(&mut self, a: u16, src: usize, _dst: &mut u16, dsto: u16) {
        let index = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX) as usize >> 1;
        write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_STATE + index * 2, a);
        let next = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_MISC_OBJS_INDEX, next);
        let load_ptr = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS);
        write_le_u16(
            &mut self.ram,
            DUNG_OBJECT_POS_IN_OBJDATA + index * 2,
            load_ptr,
        );
        let plane = if read_le_u16(&self.ram, DUNG_LINE_PTRS_ROW0) != 0x4000 {
            0
        } else {
            0x2000
        };
        write_le_u16(
            &mut self.ram,
            DUNG_OBJECT_TILEMAP_POS + index * 2,
            dsto.wrapping_mul(2) | plane,
        );
        let ul = self.room_read_current(dsto);
        let ll = self.room_read_current(dsto + 64);
        let ur = self.room_read_current(dsto + 1);
        let lr = self.room_read_current(dsto + 65);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UL + index * 2, ul);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LL + index * 2, ll);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UR + index * 2, ur);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LR + index * 2, lr);
        self.RoomDraw_Rightwards2x2(src, dsto);
    }

    pub(super) fn DrawObjects_PushableBlock(&mut self, dsto_x2: u16, slot: u16) {
        let x = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX) as usize >> 1;
        let next = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_MISC_OBJS_INDEX, next);
        write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_STATE + x * 2, 0);
        write_le_u16(&mut self.ram, DUNG_OBJECT_POS_IN_OBJDATA + x * 2, slot);
        write_le_u16(&mut self.ram, DUNG_OBJECT_TILEMAP_POS + x * 2, dsto_x2);
        let dsto = (dsto_x2 >> 1) & 0x1fff;
        let ul = self.room_read_current(dsto);
        let ll = self.room_read_current(dsto + 64);
        let ur = self.room_read_current(dsto + 1);
        let lr = self.room_read_current(dsto + 65);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UL + x * 2, ul);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LL + x * 2, ll);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UR + x * 2, ur);
        write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LR + x * 2, lr);
        self.RoomDraw_Rightwards2x2(0x0e52, dsto);
    }

    pub(super) fn DrawObjects_LightableTorch(&mut self, dsto_x2: u16, slot: u16) {
        let x = read_le_u16(&self.ram, DUNG_INDEX_OF_TORCHES) as usize >> 1;
        let next = read_le_u16(&self.ram, DUNG_INDEX_OF_TORCHES).wrapping_add(2);
        write_le_u16(&mut self.ram, DUNG_INDEX_OF_TORCHES, next);
        write_le_u16(&mut self.ram, DUNG_OBJECT_TILEMAP_POS + x * 2, dsto_x2);
        write_le_u16(&mut self.ram, DUNG_OBJECT_POS_IN_OBJDATA + x * 2, slot);
        let mut src_img = 0x0ec2;
        let dsto = (dsto_x2 >> 1) & 0x1fff;
        if dsto_x2 & 0x8000 != 0 {
            src_img = 0x0eca;
            if self.ram[DUNG_NUM_LIT_TORCHES] < 3 {
                self.ram[DUNG_NUM_LIT_TORCHES] = self.ram[DUNG_NUM_LIT_TORCHES].wrapping_add(1);
            }
        }
        self.RoomDraw_Rightwards2x2(src_img, dsto);
    }

    #[track_caller]
    pub(super) fn Object_Fill_Nx1(&mut self, count: u16, src: usize, dsto: u16) {
        let tile = self.tile_word(src, 0);
        for i in 0..count {
            self.room_write_current(dsto + i, tile);
        }
    }

    pub(super) fn Object_Hole(&mut self, src: usize, dsto: u16, width: u8, height: u8) {
        self.ram[DUNG_DRAW_WIDTH_INDICATOR] = width;
        self.ram[DUNG_DRAW_HEIGHT_INDICATOR] = height;
        self.Object_SizeAtoAplus15(4);
        let width = self.ram[DUNG_DRAW_WIDTH_INDICATOR] as u16;
        for y in 0..width {
            self.Object_Fill_Nx1(width, src, dsto + y * 64);
        }

        let edge_src = 0x063c;
        self.room_write_current(dsto, self.tile_word(edge_src, 0));
        self.Object_Fill_Nx1(width - 2, edge_src + 2, dsto + 1);
        self.room_write_current(dsto + width - 1, self.tile_word(edge_src, 2));

        let bottom = dsto + (width - 1) * 64;
        self.room_write_current(bottom, self.tile_word(edge_src, 3));
        self.Object_Fill_Nx1(width - 2, edge_src + 8, bottom + 1);
        self.room_write_current(bottom + width - 1, self.tile_word(edge_src, 5));

        let side_src = 0x0648;
        for y in 1..width - 1 {
            self.room_write_current(dsto + y * 64, self.tile_word(side_src, 0));
            self.room_write_current(dsto + width - 1 + y * 64, self.tile_word(side_src, 1));
        }
    }

    #[track_caller]
    pub(super) fn room_fill_rect(&mut self, dsto: u16, width: u16, height: u16, tile: u16) {
        for y in 0..height {
            for x in 0..width {
                self.room_write_current(dsto + x + y * 64, tile);
            }
        }
    }

    #[track_caller]
    pub(super) fn room_fill_horizontal(&mut self, dsto: u16, count: u16, tile: u16) {
        for x in 0..count {
            self.room_write_current(dsto + x, tile);
        }
    }

    #[track_caller]
    pub(super) fn room_write_current(&mut self, dsto: u16, tile: u16) {
        self.room_write_bg(
            read_le_u16(&self.ram, DUNG_LINE_PTRS_ROW0) as usize,
            dsto,
            tile,
        );
    }

    pub(super) fn room_read_current(&self, dsto: u16) -> u16 {
        let base = read_le_u16(&self.ram, DUNG_LINE_PTRS_ROW0) as usize;
        read_le_u16(&self.ram, base + dsto as usize * 2)
    }

    pub(super) fn DstoPtr(&self, d: u16) -> usize {
        read_le_u16(&self.ram, DUNG_LINE_PTRS_ROW0) as usize + d as usize * 2
    }

    pub(super) fn room_read_bg(&self, base: usize, dsto: u16) -> u16 {
        read_le_u16(&self.ram, base + dsto as usize * 2)
    }

    #[track_caller]
    pub(super) fn room_write_bg(&mut self, base: usize, dsto: u16, tile: u16) {
        let offset = base + dsto as usize * 2;
        if replay_room_write_trace_addr(offset) {
            let caller = std::panic::Location::caller();
            eprintln!(
                "room-write addr=0x{offset:05x} base=0x{base:05x} dsto=0x{dsto:04x} tile=0x{tile:04x} caller={}:{}",
                caller.file(),
                caller.line()
            );
        }
        write_le_u16(&mut self.ram, offset, tile);
    }

    pub(super) fn tile_word(&self, src: usize, index: usize) -> u16 {
        read_word_from_slice(
            self.asset_raw(69)
                .expect("missing predefined dungeon tile asset"),
            src + index * 2,
        )
    }

    pub(super) fn RoomDraw_FloorChunks(&mut self, base: usize, src_offset: usize) {
        let Some(tile_data) = self.asset_raw(69).map(Vec::from) else {
            return;
        };
        for &quadrant in &DUNGEON_QUADRANT_OFFSETS {
            let mut dst = quadrant;
            for _ in 0..8 {
                self.room_draw_many_32x32_blocks(base, src_offset, &tile_data, dst);
                dst += xy(0, 4) * 2;
            }
        }
    }

    pub(super) fn room_draw_many_32x32_blocks(
        &mut self,
        base: usize,
        src_offset: usize,
        tile_data: &[u8],
        dst: usize,
    ) {
        let mut cursor = dst;
        for _ in 0..8 {
            for _ in 0..2 {
                for y in 0..2 {
                    for x in 0..4 {
                        let src = read_word_from_slice(tile_data, src_offset + (y * 4 + x) * 2);
                        write_le_u16(&mut self.ram, base + cursor + xy(x, y) * 2, src);
                    }
                }
                cursor += xy(0, 2) * 2;
            }
            cursor = cursor.wrapping_add(xy(4, 0) * 2).wrapping_sub(xy(0, 4) * 2);
        }
    }

    pub(super) fn Dungeon_UploadRoomQuadrants(&mut self) {
        self.ram[DUNG_CUR_QUADRANT_UPLOAD] = 0;
        self.ram[OVERWORLD_MAP_STATE] = 0;
        while self.ram[DUNG_CUR_QUADRANT_UPLOAD] != 16 {
            self.TileMapPrep_NotWaterOnTag();
            self.upload_tilemap_now();
            self.Dungeon_PrepareNextRoomQuadrantUpload();
            self.upload_tilemap_now();
        }
        self.ram[NMI_SUBROUTINE_INDEX] = 0;
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.frame_control_view_mut().set_subsubmodule(0);
    }

    pub(super) fn Dungeon_PrepareNextRoomQuadrantUpload(&mut self) {
        let quadrant = self.ram[DUNG_CUR_QUADRANT_UPLOAD] as usize;
        self.Dungeon_PrepareNextRoomQuadrantUploadFrom(DUNG_BG2, quadrant, 0);
        self.ram[DUNG_CUR_QUADRANT_UPLOAD] = self.ram[DUNG_CUR_QUADRANT_UPLOAD].wrapping_add(4);
    }

    pub(super) fn Dungeon_PrepareNextRoomQuadrantUploadFrom(
        &mut self,
        source_base: usize,
        quadrant_upload: usize,
        dst_bias: u8,
    ) {
        let ofs = (self.ram[OVERWORLD_SCREEN_TRANSITION] as usize & 0x0f) + quadrant_upload;
        let mut src = UPLOAD_BG_SRCS[ofs];
        let mut p = 0usize;
        loop {
            loop {
                for y in 0..4 {
                    for x in 0..2 {
                        let value = read_le_u16(&self.ram, source_base + src + xy(x, y) * 2);
                        write_le_u16(
                            &mut self.ram,
                            VRAM_UPLOAD_OFFSET + (p + y * 32 + x) * 2,
                            value,
                        );
                    }
                }
                src += 2 * 2;
                p += 2;
                if p & 0x1f == 0 {
                    break;
                }
            }
            src += 224 * 2;
            p += 128 - 32;
            if p == 0x400 {
                break;
            }
        }
        self.ram[NMI_LOAD_TARGET_ADDR] = UPLOAD_BG_DSTS[ofs] + dst_bias;
        self.ram[NMI_SUBROUTINE_INDEX] = 1;
        self.ram[NMI_DISABLE_CORE_UPDATES] = 1;
    }

    pub(super) fn WaterFlood_BuildOneQuadrantForVRAM(&mut self) {
        assert_ne!(self.ram[DUNG_HDR_TAG], 25);
        self.TileMapPrep_NotWaterOnTag();
    }

    pub(super) fn FloodDam_PrepTiles_init(&mut self) {
        self.ram[DUNG_CUR_QUADRANT_UPLOAD] = 0;
        self.ram[OVERWORLD_SCREEN_TRANSITION] = 0;
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.ram[DUNG_CUR_QUADRANT_UPLOAD] = self.ram[DUNG_CUR_QUADRANT_UPLOAD].wrapping_add(4);
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Watergate_Main_State1(&mut self) {
        self.ram[OVERWORLD_SCREEN_TRANSITION] = 0;
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.ram[DUNG_CUR_QUADRANT_UPLOAD] = self.ram[DUNG_CUR_QUADRANT_UPLOAD].wrapping_add(4);
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Dungeon_FloodSwampWater_PrepTileMap(&mut self) {
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.ram[DUNG_CUR_QUADRANT_UPLOAD] = self.ram[DUNG_CUR_QUADRANT_UPLOAD].wrapping_add(4);
        self.frame_control_view_mut().increment_subsubmodule();
        if self.frame_control_view().subsubmodule() == 6 {
            self.ram[DUNG_CUR_QUADRANT_UPLOAD] = 0;
            self.frame_control_view_mut().set_subsubmodule(0);
            self.frame_control_view_mut().set_submodule(0);
        }
    }

    pub(super) fn Dungeon_AdjustWaterVomit(&mut self, src: usize, depth: i32) {
        let mut dsto = (read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_SRC_POS_X2) >> 1)
            .wrapping_add(xy(0, 2) as u16);
        let mut row = 0usize;
        let mut remaining = depth;
        loop {
            for x in 0..4 {
                let tile = self.tile_word(src, row * 4 + x);
                write_le_u16(&mut self.ram, DUNG_BG2 + (dsto as usize + x) * 2, tile);
            }
            dsto = dsto.wrapping_add(xy(0, 1) as u16);
            row += 1;
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }

        let base_dsto = (read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_SRC_POS_X2) >> 1)
            .wrapping_add(xy(0, 2) as u16);
        let mut upload = VRAM_UPLOAD_DATA;
        for i in 0..4u16 {
            let col = base_dsto.wrapping_add(i);
            let vram_addr = self.Dungeon_MapVramAddr(col);
            write_le_u16(&mut self.ram, upload, vram_addr);
            write_le_u16(&mut self.ram, upload + 2, 0x0980);
            for y in 0..5usize {
                let tile = read_le_u16(&self.ram, DUNG_BG2 + (col as usize + y * 64) * 2);
                write_le_u16(&mut self.ram, upload + 4 + y * 2, tile);
            }
            upload += 14;
        }
        write_le_u16(&mut self.ram, upload, 0xffff);
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn FloodDam_Expand(&mut self) {
        self.ram[WATERGATE_POINTER] = self.ram[WATERGATE_POINTER].wrapping_add(1);
        let watergate_var1 = self.ram[WATERGATE_POINTER];
        write_le_u16(
            &mut self.ram,
            WATER_HDMA_WINDOW_X_RADIUS_DUNGEON,
            u16::from(watergate_var1 >> 1),
        );
        let r0 = self.ram[WATER_HDMA_WINDOW_X_RADIUS_DUNGEON].wrapping_sub(8);
        self.ram[SPOTLIGHT_Y_UPPER] = self.ram[WATERGATE_SPOTLIGHT_Y_UPPER];
        self.ram[SPOTLIGHT_WINDOW_Y_BUFFER] = self.ram[SPOTLIGHT_WINDOW_Y_BUFFER].wrapping_add(1);
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON] =
            self.ram[SPOTLIGHT_WINDOW_Y_BUFFER].wrapping_add(r0);

        if watergate_var1 & 0x0f != 0 {
            return;
        }
        if watergate_var1 == 64 {
            self.frame_control_view_mut().increment_subsubmodule();
        }

        const WATERGATE_SRCS1: [usize; 4] = [0x12f8, 0x1348, 0x1398, 0x13e8];
        let src = WATERGATE_SRCS1[((watergate_var1 >> 4).wrapping_sub(1)) as usize];
        let dsto = read_le_u16(&self.ram, WATERGATE_POS) >> 1;
        for x in 0..10u16 {
            for y in 0..4u16 {
                let tile = self.tile_word(src, (x * 4 + y) as usize);
                write_le_u16(
                    &mut self.ram,
                    DUNG_BG2 + (dsto as usize + x as usize + y as usize * 64) * 2,
                    tile,
                );
            }
        }

        let mut pos = read_le_u16(&self.ram, WATERGATE_POS);
        let mut dma_ptr = 0usize;
        for _ in 0..3 {
            dma_ptr = self.dungeon_prep_overlay_dma_watergate(dma_ptr, pos, 0x0881, 4);
            pos = pos.wrapping_add(6);
        }
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;
    }

    pub(super) fn FloodDam_Fill(&mut self) {
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON] =
            self.ram[WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON].wrapping_add(1);
        let t =
            self.ram[WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON].wrapping_add(self.ram[SPOTLIGHT_Y_UPPER]);
        if t >= 225 {
            self.ram[DUNG_CUR_QUADRANT_UPLOAD] = 0;
            self.frame_control_view_mut().set_submodule(0);
            self.frame_control_view_mut().set_subsubmodule(0);
            self.ram[TMW_COPY] = 0;
            self.ram[TSW_COPY] = 0;
            self.IrisSpotlight_ResetTable();
        }
    }

    pub(super) fn TileMapPrep_NotWaterOnTag(&mut self) {
        self.Dungeon_PrepareNextRoomQuadrantUploadFrom(
            DUNG_BG1,
            self.ram[DUNG_CUR_QUADRANT_UPLOAD] as usize,
            0x10,
        );
    }

    pub(super) fn OrientLampLightCone(&mut self) {
        const BG_TAB0: [u16; 4] = [0, 256, 0, 256];
        const BG_TAB1: [u16; 4] = [0, 0, 256, 256];
        const BG_TAB2: [i16; 4] = [52, -2, 56, 6];
        const BG_TAB3: [i16; 4] = [64, 64, 82, -176];
        const BG_TAB4: [u16; 4] = [128, 384, 160, 160];

        if self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] == 0
            || self.frame_control_view().submodule() == 20
        {
            return;
        }

        let a = self.ram[LINK_DIRECTION_FACING] >> 1;
        let mut idx = a;
        if self.ram[IS_STANDING_IN_DOORWAY] != 0 {
            idx = self.ram[IS_STANDING_IN_DOORWAY] & 0xfe;
            if idx != 0 {
                if a < 2 {
                    idx = idx.wrapping_add(u8::from(
                        self.player_state_view().x().wrapping_add(8) as u8 >= 0x80,
                    ));
                } else {
                    idx = a;
                }
            } else if a >= 2 {
                idx = idx.wrapping_add(u8::from(self.player_state_view().y() as u8 >= 0x80));
            } else {
                idx = a;
            }
        }

        let idx = idx as usize;
        if idx >= 4 {
            return;
        }

        if idx < 2 {
            let h = read_le_u16(&self.ram, BG2HOFS_COPY2)
                .wrapping_sub(self.player_state_view().x().wrapping_sub(0x77))
                .wrapping_add(BG_TAB0[idx]);
            write_le_u16(&mut self.ram, BG1HOFS_COPY2, h);

            let t = read_le_u16(&self.ram, BG2VOFS_COPY2)
                .wrapping_sub(self.player_state_view().y().wrapping_sub(0x58))
                .wrapping_add(BG_TAB1[idx])
                .wrapping_add(BG_TAB2[idx] as u16)
                .wrapping_add(BG_TAB3[idx] as u16);
            let t = clamp_c_int16_to_u16(t, BG_TAB4[idx]);
            write_le_u16(
                &mut self.ram,
                BG1VOFS_COPY2,
                t.wrapping_sub(BG_TAB3[idx] as u16),
            );
        } else {
            let v = read_le_u16(&self.ram, BG2VOFS_COPY2)
                .wrapping_sub(self.player_state_view().y().wrapping_sub(0x72))
                .wrapping_add(BG_TAB1[idx]);
            write_le_u16(&mut self.ram, BG1VOFS_COPY2, v);

            let t = read_le_u16(&self.ram, BG2HOFS_COPY2)
                .wrapping_sub(self.player_state_view().x().wrapping_sub(0x58))
                .wrapping_add(BG_TAB0[idx])
                .wrapping_add(BG_TAB2[idx] as u16)
                .wrapping_add(BG_TAB3[idx] as u16);
            let t = clamp_c_int16_to_u16(t, BG_TAB4[idx]);
            write_le_u16(
                &mut self.ram,
                BG1HOFS_COPY2,
                t.wrapping_sub(BG_TAB3[idx] as u16),
            );
        }
    }

    pub(super) fn SavePalaceDeaths(&mut self) {
        let j = self.ram[CUR_PALACE_INDEX_X2] as usize;
        let deaths = read_le_u16(&self.ram, DEATH_SAVE_COUNTER);
        write_le_u16(&mut self.ram, DEATHS_PER_PALACE + (j >> 1) * 2, deaths);
        if j != 8 {
            write_le_u16(&mut self.ram, DEATH_SAVE_COUNTER, 0);
        }
    }

    pub(super) fn upload_tilemap_now(&mut self) {
        let target = self.ram[NMI_LOAD_TARGET_ADDR] as usize;
        let vram_page = NMI_VRAM_ADDRS[target];
        let dst = vram_page << 8;
        for i in 0..0x400 {
            self.ppu.vram[dst + i] = read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET + i * 2);
        }
        write_le_u16(&mut self.ram, VRAM_UPLOAD_OFFSET, 0);
        self.ram[NMI_DISABLE_CORE_UPDATES] = 0;
    }

    pub(super) fn dungeon_room_layout(&self, room: usize) -> Option<&[u8]> {
        let offset = self.asset_u16(4, room) as usize;
        let data = self.asset_raw(3)?;
        data.get(offset..)
    }

    pub(super) fn default_room_layout(&self, index: usize) -> Option<&[u8]> {
        let offset = self.asset_u16(47, index) as usize;
        let data = self.asset_raw(46)?;
        data.get(offset..)
    }

    pub(super) fn dungeon_room_header(&self, room: usize) -> Option<&[u8]> {
        let offset = self.asset_u16(7, room) as usize;
        let data = self.asset_raw(6)?;
        data.get(offset..)
    }

    pub(super) fn asset_u16_from_ram(&self, offset: usize, index: usize) -> u16 {
        read_le_u16(&self.ram, offset + index * 2)
    }

    pub(super) fn SetAndSaveVisitedQuadrantFlags(&mut self) {
        const QUADRANT_VISITING_FLAGS: [u16; 16] = [
            8, 4, 2, 1, 0x0c, 0x0c, 3, 3, 0x0a, 5, 0x0a, 5, 0x0f, 0x0f, 0x0f, 0x0f,
        ];
        let index = ((self.ram[QUADRANT_FULLSIZE_Y] as usize) << 2)
            + ((self.ram[QUADRANT_FULLSIZE_X] as usize) << 1)
            + self.ram[LINK_QUADRANT_Y] as usize
            + self.ram[LINK_QUADRANT_X] as usize;
        let flag = QUADRANT_VISITING_FLAGS[index];
        let visited = read_le_u16(&self.ram, DUNG_QUADRANTS_VISITED) | flag;
        write_le_u16(&mut self.ram, DUNG_QUADRANTS_VISITED, visited);

        let room = self.world_state_view().dungeon_room() as usize;
        let dst = SAVE_DUNG_INFO + room * 2;
        let saved = read_le_u16(&self.ram, dst) | visited;
        write_le_u16(&mut self.ram, dst, saved);
    }

    pub(super) fn Dungeon_PlayBlipAndCacheQuadrantVisits(&mut self) {
        self.ram[HUD_FLOOR_CHANGED_TIMER] = 1;
        self.ram[SOUND_EFFECT_2] = 36;
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn ResetTransitionPropsAndAdvance_ResetInterface(&mut self) {
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.ResetTransitionPropsAndAdvanceSubmodule();
    }

    pub(super) fn ResetTransitionPropsAndAdvanceSubmodule(&mut self) {
        write_le_u16(&mut self.ram, MOSAIC_LEVEL, 0);
        self.ram[DARKENING_OR_LIGHTENING_SCREEN] = 0;
        self.ram[PALETTE_FILTER_COUNTDOWN] = 0;
        self.ram[MOSAIC_TARGET_LEVEL] = 31;
        write_le_u16(&mut self.ram, UNUSED_CONFIG_GFX, 0);
        self.ram[DUNG_NUM_LIT_TORCHES] = 0;
        if self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] != 0 {
            self.ram[CGWSEL_COPY] = 0x02;
            self.ram[CGADSUB_COPY] = 0xb3;
        }
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 0;
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        self.Overworld_CopyPalettesToCache();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn RoomBounds_AddA(&mut self, r: &RoomBounds) {
        for offset in [0, 4] {
            let value = read_le_u16(&self.ram, r.base + offset).wrapping_add(0x0100);
            write_le_u16(&mut self.ram, r.base + offset, value);
        }
    }

    pub(super) fn RoomBounds_AddB(&mut self, r: &RoomBounds) {
        for offset in [2, 6] {
            let value = read_le_u16(&self.ram, r.base + offset).wrapping_add(0x0200);
            write_le_u16(&mut self.ram, r.base + offset, value);
        }
    }

    pub(super) fn RoomBounds_SubB(&mut self, r: &RoomBounds) {
        for offset in [2, 6] {
            let value = read_le_u16(&self.ram, r.base + offset).wrapping_sub(0x0200);
            write_le_u16(&mut self.ram, r.base + offset, value);
        }
    }

    pub(super) fn RoomBounds_SubA(&mut self, r: &RoomBounds) {
        for offset in [0, 4] {
            let value = read_le_u16(&self.ram, r.base + offset).wrapping_sub(0x0100);
            write_le_u16(&mut self.ram, r.base + offset, value);
        }
    }

    pub(super) fn AdjustQuadrantAndCamera_right(&mut self) {
        self.ram[LINK_QUADRANT_X] ^= 1;
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_AddA(&ROOM_BOUNDS_X_REF);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn AdjustQuadrantAndCamera_left(&mut self) {
        self.ram[LINK_QUADRANT_X] ^= 1;
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_SubA(&ROOM_BOUNDS_X_REF);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn AdjustQuadrantAndCamera_down(&mut self) {
        self.ram[LINK_QUADRANT_Y] ^= 2;
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_AddA(&ROOM_BOUNDS_Y_REF);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn AdjustQuadrantAndCamera_up(&mut self) {
        self.ram[LINK_QUADRANT_Y] ^= 2;
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_SubA(&ROOM_BOUNDS_Y_REF);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn Dungeon_AdjustQuadrant(&mut self) {
        self.ram[COMPOSITE_OF_LAYOUT_AND_QUADRANT] = self.ram[DUNG_LAYOUT_AND_STARTING_QUADRANT]
            | self.ram[LINK_QUADRANT_Y]
            | self.ram[LINK_QUADRANT_X];
    }

    pub(super) fn Dungeon_AdjustForRoomLayout(&mut self) {
        self.Dungeon_AdjustQuadrant();
        let flags = K_LAYOUT_QUADRANT_FLAGS[self.ram[COMPOSITE_OF_LAYOUT_AND_QUADRANT] as usize];
        self.ram[QUADRANT_FULLSIZE_X] = if self.ram[DUNG_BLASTWALL_FLAG_X] != 0
            || flags & if self.ram[LINK_QUADRANT_X] != 0 { 2 } else { 1 } == 0
        {
            2
        } else {
            0
        };
        self.ram[QUADRANT_FULLSIZE_Y] = if self.ram[DUNG_BLASTWALL_FLAG_Y] != 0
            || flags & if self.ram[LINK_QUADRANT_Y] != 0 { 8 } else { 4 } == 0
        {
            2
        } else {
            0
        };

        let unk2 = read_le_u16(&self.ram, RESET_XY_CHECK_FLAGS);
        if unk2 as u8 != 0 {
            self.ram[QUADRANT_FULLSIZE_X] = unk2 as u8;
        }
        if (unk2 >> 8) as u8 != 0 {
            self.ram[QUADRANT_FULLSIZE_Y] = (unk2 >> 8) as u8;
        }
    }

    pub(super) fn Dung_SaveDataForCurrentRoom(&mut self) {
        let saved = (read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) >> 4)
            | (read_le_u16(&self.ram, DUNG_DOOR_OPENED) & 0xf000)
            | read_le_u16(&self.ram, DUNG_QUADRANTS_VISITED);
        let room = self.world_state_view().dungeon_room() as usize;
        write_le_u16(&mut self.ram, SAVE_DUNG_INFO + room * 2, saved);
    }

    pub(super) fn SaveQuadrantsToSram(&mut self) {
        let room = self.world_state_view().dungeon_room() as usize;
        let offset = SAVE_DUNG_INFO + room * 2;
        let saved = read_le_u16(&self.ram, offset) | read_le_u16(&self.ram, DUNG_QUADRANTS_VISITED);
        write_le_u16(&mut self.ram, offset, saved);
    }

    pub(super) fn Dung_HandleExitToOverworld(&mut self) {
        self.SaveDungeonKeys();
        self.SaveQuadrantsToSram();
        self.ram[SAVED_MODULE_FOR_MENU] = 8;
        self.frame_control_view_mut().set_main_module(15);
        self.frame_control_view_mut().set_submodule(0);
        self.frame_control_view_mut().set_subsubmodule(0);
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
    }

    pub(super) fn Dungeon_FlagRoomData_Quadrants(&mut self) {
        const QUADRANT_VISITING_FLAGS: [u16; 16] = [
            8, 4, 2, 1, 0x0c, 0x0c, 3, 3, 0x0a, 5, 0x0a, 5, 0x0f, 0x0f, 0x0f, 0x0f,
        ];
        let index = ((self.ram[QUADRANT_FULLSIZE_Y] as usize) << 2)
            + ((self.ram[QUADRANT_FULLSIZE_X] as usize) << 1)
            + self.ram[LINK_QUADRANT_Y] as usize
            + self.ram[LINK_QUADRANT_X] as usize;
        let visited =
            read_le_u16(&self.ram, DUNG_QUADRANTS_VISITED) | QUADRANT_VISITING_FLAGS[index];
        write_le_u16(&mut self.ram, DUNG_QUADRANTS_VISITED, visited);
        self.Dung_SaveDataForCurrentRoom();
    }

    pub(super) fn DungeonTransition_AdjustCamera_X(&mut self, arg: u8) {
        const UP_DOWN_SCROLL: [u16; 4] = [0, 256, 256, 0];
        let index = arg as usize * 2;
        write_le_u16(
            &mut self.ram,
            LEFT_RIGHT_SCROLL_TARGET,
            UP_DOWN_SCROLL[index],
        );
        write_le_u16(
            &mut self.ram,
            LEFT_RIGHT_SCROLL_TARGET_END,
            UP_DOWN_SCROLL[index + 1],
        );
    }

    pub(super) fn DungeonTransition_AdjustCamera_Y(&mut self, arg: u8) {
        const UP_DOWN_SCROLL: [u16; 4] = [0, 272, 256, 16];
        let index = arg as usize;
        write_le_u16(&mut self.ram, UP_DOWN_SCROLL_TARGET, UP_DOWN_SCROLL[index]);
        write_le_u16(
            &mut self.ram,
            UP_DOWN_SCROLL_TARGET_END,
            UP_DOWN_SCROLL[index + 1],
        );
    }

    pub(super) fn HandleEdgeTransition_AdjustCameraBoundaries(&mut self, arg: u8) {
        const CAMERA_BOUNDS_X: [u16; 4] = [127, 383, 127, 383];
        const CAMERA_BOUNDS_Y: [u16; 4] = [120, 376, 136, 392];
        self.ram[OVERWORLD_SCREEN_TRANSITION] = arg;
        if self.ram[LINK_DIRECTION] & 3 != 0 {
            let mut index = if self.ram[LINK_DIRECTION] & 1 != 0 {
                0
            } else {
                2
            };
            if self.ram[LINK_QUADRANT_X] != 0 {
                index += 1;
            }
            write_le_u16(
                &mut self.ram,
                CAMERA_X_COORD_SCROLL_LOW,
                CAMERA_BOUNDS_X[index],
            );
            write_le_u16(
                &mut self.ram,
                CAMERA_X_COORD_SCROLL_HI,
                CAMERA_BOUNDS_X[index].wrapping_add(2),
            );
        } else {
            let mut index = if self.ram[LINK_DIRECTION] & 4 != 0 {
                0
            } else {
                2
            };
            if self.ram[LINK_QUADRANT_Y] != 0 {
                index += 1;
            }
            write_le_u16(
                &mut self.ram,
                CAMERA_Y_COORD_SCROLL_LOW,
                CAMERA_BOUNDS_Y[index],
            );
            write_le_u16(
                &mut self.ram,
                CAMERA_Y_COORD_SCROLL_HI,
                CAMERA_BOUNDS_Y[index].wrapping_add(2),
            );
        }
    }

    pub(super) fn Dungeon_StartInterRoomTrans_Left(&mut self) {
        assert_eq!(self.frame_control_view().submodule(), 0);
        self.ram[LINK_QUADRANT_X] ^= 1;
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_SubA(&ROOM_BOUNDS_X_REF);
        self.Dung_SaveDataForCurrentRoom();
        self.DungeonTransition_AdjustCamera_X(self.ram[LINK_QUADRANT_X] ^ 1);
        self.HandleEdgeTransition_AdjustCameraBoundaries(3);
        self.frame_control_view_mut().set_submodule(1);
        if self.ram[LINK_QUADRANT_X] != 0 {
            self.RoomBounds_SubB(&ROOM_BOUNDS_X_REF);
            self.ram[DUNGEON_ROOM_INDEX_PREV] = self.ram[DUNGEON_ROOM_INDEX];
            if self.ram[LINK_TILE_BELOW] & 0xcf == 0x89 {
                self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNG_HDR_TRAVEL_DESTINATIONS + 3];
                self.Dungeon_AdjustForTeleportDoors(
                    self.ram[DUNGEON_ROOM_INDEX].wrapping_add(1),
                    0xff,
                );
            } else {
                if self.ram[DUNGEON_ROOM_INDEX] != self.ram[DUNGEON_ROOM_INDEX2] {
                    self.ram[DUNGEON_ROOM_INDEX_PREV] = self.ram[DUNGEON_ROOM_INDEX2];
                    self.Dungeon_AdjustAfterSpiralStairs();
                }
                self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNGEON_ROOM_INDEX].wrapping_sub(1);
            }
            self.finish_super_tile_transition_room_side_effects();
        }
        self.ram[ROOM_TRANSITIONING_FLAGS] = 0;
        self.update_quadrant_fullsize_y_after_transition();
    }

    pub(super) fn Dung_StartInterRoomTrans_Left_Plus(&mut self) {
        let x = self.player_state_view().x().wrapping_sub(8);
        self.player_state_view_mut().set_x(x);
        self.Dungeon_StartInterRoomTrans_Left();
    }

    pub(super) fn Dungeon_StartInterRoomTrans_Right(&mut self) {
        assert_eq!(self.frame_control_view().submodule(), 0);
        self.ram[LINK_QUADRANT_X] ^= 1;
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_AddA(&ROOM_BOUNDS_X_REF);
        self.Dung_SaveDataForCurrentRoom();
        self.DungeonTransition_AdjustCamera_X(self.ram[LINK_QUADRANT_X]);
        self.HandleEdgeTransition_AdjustCameraBoundaries(2);
        self.frame_control_view_mut().set_submodule(1);
        if self.ram[LINK_QUADRANT_X] == 0 {
            self.RoomBounds_AddB(&ROOM_BOUNDS_X_REF);
            self.ram[DUNGEON_ROOM_INDEX_PREV] = self.ram[DUNGEON_ROOM_INDEX];
            if self.ram[LINK_TILE_BELOW] & 0xcf == 0x89 {
                self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNG_HDR_TRAVEL_DESTINATIONS + 4];
                self.Dungeon_AdjustForTeleportDoors(
                    self.ram[DUNGEON_ROOM_INDEX].wrapping_sub(1),
                    1,
                );
            } else {
                if self.ram[DUNGEON_ROOM_INDEX] != self.ram[DUNGEON_ROOM_INDEX2] {
                    self.ram[DUNGEON_ROOM_INDEX_PREV] = self.ram[DUNGEON_ROOM_INDEX2];
                    self.Dungeon_AdjustAfterSpiralStairs();
                }
                self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNGEON_ROOM_INDEX].wrapping_add(1);
            }
            self.finish_super_tile_transition_room_side_effects();
        }
        self.ram[ROOM_TRANSITIONING_FLAGS] = 0;
        self.update_quadrant_fullsize_y_after_transition();
    }

    pub(super) fn Dungeon_StartInterRoomTrans_Up(&mut self) {
        assert_eq!(self.frame_control_view().submodule(), 0);
        self.ram[LINK_QUADRANT_Y] ^= 2;
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_SubA(&ROOM_BOUNDS_Y_REF);
        self.Dung_SaveDataForCurrentRoom();
        self.DungeonTransition_AdjustCamera_Y(self.ram[LINK_QUADRANT_Y] ^ 2);
        self.HandleEdgeTransition_AdjustCameraBoundaries(1);
        self.frame_control_view_mut().set_submodule(1);
        if self.ram[LINK_QUADRANT_Y] != 0 {
            self.RoomBounds_SubB(&ROOM_BOUNDS_Y_REF);
            self.ram[DUNGEON_ROOM_INDEX_PREV] = self.ram[DUNGEON_ROOM_INDEX];
            if self.ram[LINK_TILE_BELOW] == 0x8e {
                self.Dung_HandleExitToOverworld();
                return;
            }
            if self.ram[DUNGEON_ROOM_INDEX] == 0 {
                self.SaveDungeonKeys();
                self.frame_control_view_mut().set_main_module(25);
                self.frame_control_view_mut().set_submodule(0);
                self.frame_control_view_mut().set_subsubmodule(0);
                return;
            }
            if self.ram[DUNGEON_ROOM_INDEX2] == self.ram[DUNGEON_ROOM_INDEX] {
                self.ram[DUNGEON_ROOM_INDEX_PREV] = self.ram[DUNGEON_ROOM_INDEX2];
                self.Dungeon_AdjustAfterSpiralStairs();
            }
            self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNGEON_ROOM_INDEX].wrapping_sub(0x10);
            self.finish_super_tile_transition_room_side_effects();
        }
        self.ram[ROOM_TRANSITIONING_FLAGS] = 0;
        self.update_quadrant_fullsize_x_after_transition();
    }

    pub(super) fn Dungeon_StartInterRoomTrans_Down(&mut self) {
        assert_eq!(self.frame_control_view().submodule(), 0);
        self.ram[LINK_QUADRANT_Y] ^= 2;
        self.Dungeon_AdjustQuadrant();
        self.RoomBounds_AddA(&ROOM_BOUNDS_Y_REF);
        self.Dung_SaveDataForCurrentRoom();
        self.DungeonTransition_AdjustCamera_Y(self.ram[LINK_QUADRANT_Y]);
        self.HandleEdgeTransition_AdjustCameraBoundaries(0);
        self.frame_control_view_mut().set_submodule(1);
        if self.ram[LINK_QUADRANT_Y] == 0 {
            self.RoomBounds_AddB(&ROOM_BOUNDS_Y_REF);
            self.ram[DUNGEON_ROOM_INDEX_PREV] = self.ram[DUNGEON_ROOM_INDEX];
            if self.ram[LINK_TILE_BELOW] == 0x8e {
                self.Dung_HandleExitToOverworld();
                return;
            }
            if self.ram[DUNGEON_ROOM_INDEX] != self.ram[DUNGEON_ROOM_INDEX2] {
                self.ram[DUNGEON_ROOM_INDEX_PREV] = self.ram[DUNGEON_ROOM_INDEX2];
                self.Dungeon_AdjustAfterSpiralStairs();
            }
            self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNGEON_ROOM_INDEX].wrapping_add(0x10);
            self.finish_super_tile_transition_room_side_effects();
        }
        self.ram[ROOM_TRANSITIONING_FLAGS] = 0;
        self.update_quadrant_fullsize_x_after_transition();
    }

    fn finish_super_tile_transition_room_side_effects(&mut self) {
        self.frame_control_view_mut().set_submodule(2);
        if self.ram[ROOM_TRANSITIONING_FLAGS] & 1 != 0 {
            self.ram[LINK_IS_ON_LOWER_LEVEL] ^= 1;
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        }
        if self.ram[ROOM_TRANSITIONING_FLAGS] & 2 != 0 {
            self.ram[CUR_PALACE_INDEX_X2] ^= 2;
        }
    }

    fn update_quadrant_fullsize_x_after_transition(&mut self) {
        let flags = K_LAYOUT_QUADRANT_FLAGS[self.ram[COMPOSITE_OF_LAYOUT_AND_QUADRANT] as usize];
        let mask = if self.ram[LINK_QUADRANT_X] != 0 { 2 } else { 1 };
        self.ram[QUADRANT_FULLSIZE_X] = if self.ram[DUNG_BLASTWALL_FLAG_X] != 0 || flags & mask == 0
        {
            2
        } else {
            0
        };
    }

    fn update_quadrant_fullsize_y_after_transition(&mut self) {
        let flags = K_LAYOUT_QUADRANT_FLAGS[self.ram[COMPOSITE_OF_LAYOUT_AND_QUADRANT] as usize];
        let mask = if self.ram[LINK_QUADRANT_Y] != 0 { 8 } else { 4 };
        self.ram[QUADRANT_FULLSIZE_Y] = if self.ram[DUNG_BLASTWALL_FLAG_Y] != 0 || flags & mask == 0
        {
            2
        } else {
            0
        };
    }

    pub(super) fn HandleEdgeTransitionMovementEast_RightBy8(&mut self) {
        let x = self.player_state_view().x().wrapping_add(8);
        self.player_state_view_mut().set_x(x);
        self.Dungeon_StartInterRoomTrans_Right();
    }

    pub(super) fn HandleEdgeTransitionMovementSouth_DownBy16(&mut self) {
        let y = self.player_state_view().y().wrapping_add(16);
        self.player_state_view_mut().set_y(y);
        self.Dungeon_StartInterRoomTrans_Down();
    }

    pub(super) fn Dungeon_Store2x2(
        &mut self,
        pos: u16,
        t0: u16,
        t1: u16,
        t2: u16,
        t3: u16,
        attr: u8,
    ) {
        let tiles = [t0, t1, t2, t3];
        let positions = [pos, pos + 64, pos + 1, pos + 65];
        for (&tile_pos, &tile) in positions.iter().zip(tiles.iter()) {
            write_le_u16(&mut self.ram, DUNG_BG2 + tile_pos as usize * 2, tile);
            self.ram[DUNG_BG2_ATTR_TABLE + tile_pos as usize] = attr;
        }

        let upload = read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET) as usize;
        let dst = VRAM_UPLOAD_DATA + upload;
        for (i, (&tile_pos, &tile)) in positions.iter().zip(tiles.iter()).enumerate() {
            let base = dst + i * 6;
            let addr = self.Dungeon_MapVramAddr(tile_pos);
            write_le_u16(&mut self.ram, base, addr);
            write_le_u16(&mut self.ram, base + 2, 0x0100);
            write_le_u16(&mut self.ram, base + 4, tile);
        }
        write_le_u16(&mut self.ram, dst + 24, 0xffff);
        write_le_u16(
            &mut self.ram,
            VRAM_UPLOAD_OFFSET,
            upload.wrapping_add(24) as u16,
        );
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn Dungeon_UpdateTileMapWithCommonTile(&mut self, x: i32, y: i32, v: u8) {
        if v == 8 {
            self.Dungeon_PrepSpriteInducedDma(x + 16, y, v + 2);
        }
        self.Dungeon_PrepSpriteInducedDma(x, y, v);
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn Dungeon_PrepSpriteInducedDma(&mut self, x: i32, y: i32, v: u8) {
        const PREP_SPRITE_INDUCED_DMA_SRCS: [usize; 10] = [
            0x0e0, 0xade, 0x5aa, 0x198, 0x210, 0x218, 0x1f3a, 0xeaa, 0xeb2, 0x140,
        ];

        let pos = ((((y + 1) as u16) & 0x01f8) << 3) | (((x as u16) & 0x01f8) >> 3);
        let src = PREP_SPRITE_INDUCED_DMA_SRCS[(v >> 1) as usize];
        let tiles = [
            self.tile_word(src, 0),
            self.tile_word(src, 1),
            self.tile_word(src, 2),
            self.tile_word(src, 3),
        ];
        let attr = self.ram[ATTRIBUTES_FOR_TILE + (tiles[3] & 0x03ff) as usize];
        let tile_positions = [pos, pos + 64, pos + 1, pos + 65];
        if std::env::var_os("ZELDA3_TRACE_SPRITE_DMA").is_some() {
            let target = std::env::var("ZELDA3_TRACE_SPRITE_DMA_POS")
                .ok()
                .and_then(|value| {
                    value
                        .strip_prefix("0x")
                        .or_else(|| value.strip_prefix("0X"))
                        .and_then(|hex| u16::from_str_radix(hex, 16).ok())
                        .or_else(|| value.parse::<u16>().ok())
                });
            if target.map_or(true, |target| tile_positions.contains(&target)) {
                eprintln!(
                    "R sprite_dma fc=0x{:02x} x=0x{:04x} y=0x{:04x} v=0x{:02x} pos=0x{:04x} src=0x{:04x} attr=0x{:02x} tiles={:04x},{:04x},{:04x},{:04x}",
                    self.ram[FRAME_COUNTER],
                    x as u16,
                    y as u16,
                    v,
                    pos,
                    src,
                    attr,
                    tiles[0],
                    tiles[1],
                    tiles[2],
                    tiles[3]
                );
            }
        }

        for &tile_pos in &tile_positions {
            self.ram[DUNG_BG2_ATTR_TABLE + tile_pos as usize] = attr;
        }

        for (&tile_pos, &tile) in tile_positions.iter().zip(tiles.iter()) {
            write_le_u16(&mut self.ram, DUNG_BG2 + tile_pos as usize * 2, tile);
        }

        let upload = read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET) as usize;
        let dst = VRAM_UPLOAD_DATA + upload;
        for (i, (&tile_pos, &tile)) in tile_positions.iter().zip(tiles.iter()).enumerate() {
            let base = dst + i * 6;
            let vram_addr = self.Dungeon_MapVramAddr(tile_pos);
            write_le_u16(&mut self.ram, base, vram_addr);
            write_le_u16(&mut self.ram, base + 2, 0x0100);
            write_le_u16(&mut self.ram, base + 4, tile);
        }
        write_le_u16(&mut self.ram, dst + 24, 0xffff);
        write_le_u16(
            &mut self.ram,
            VRAM_UPLOAD_OFFSET,
            upload.wrapping_add(24) as u16,
        );
    }

    pub(super) fn Dungeon_DeleteRupeeTile(&mut self, x: u16, y: u16) {
        let pos = ((y & 0x01f8) << 3) | ((x & 0x01f8) >> 3);
        let upload = read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET) as usize;
        let dst = VRAM_UPLOAD_DATA + upload;
        let tile = 0x190f;

        write_le_u16(&mut self.ram, DUNG_BG2 + pos as usize * 2, tile);
        write_le_u16(&mut self.ram, DUNG_BG2 + (pos + 64) as usize * 2, tile);

        let attr = u16::from(self.ram[ATTRIBUTES_FOR_TILE + (tile & 0x03ff) as usize]) * 0x0101;
        write_le_u16(&mut self.ram, DUNG_BG2_ATTR_TABLE + pos as usize, attr);
        write_le_u16(
            &mut self.ram,
            DUNG_BG2_ATTR_TABLE + (pos + 64) as usize,
            attr,
        );

        let vram_addr_0 = self.Dungeon_MapVramAddr(pos);
        let vram_addr_1 = self.Dungeon_MapVramAddr(pos + 64);
        write_le_u16(&mut self.ram, dst, vram_addr_0);
        write_le_u16(&mut self.ram, dst + 2, 0x0100);
        write_le_u16(&mut self.ram, dst + 4, tile);
        write_le_u16(&mut self.ram, dst + 6, vram_addr_1);
        write_le_u16(&mut self.ram, dst + 8, 0x0100);
        write_le_u16(&mut self.ram, dst + 10, tile);
        write_le_u16(&mut self.ram, dst + 12, 0xffff);
        write_le_u16(
            &mut self.ram,
            VRAM_UPLOAD_OFFSET,
            upload.wrapping_add(24) as u16,
        );

        let state = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) | 0x1000;
        write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, state);
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn RoomDraw_16x16Single(&mut self, index: u8) {
        let index = (index >> 1) as usize;
        let pos = (read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + index * 2) & 0x3fff) >> 1;
        let ul = read_le_u16(&self.ram, REPLACEMENT_TILEMAP_UL + index * 2);
        let ll = read_le_u16(&self.ram, REPLACEMENT_TILEMAP_LL + index * 2);
        let ur = read_le_u16(&self.ram, REPLACEMENT_TILEMAP_UR + index * 2);
        let lr = read_le_u16(&self.ram, REPLACEMENT_TILEMAP_LR + index * 2);
        let attr = self.ram[ATTRIBUTES_FOR_TILE + (lr & 0x03ff) as usize];
        self.Dungeon_Store2x2(pos, ul, ll, ur, lr, attr);
    }

    pub(super) fn Dungeon_LiftAndReplaceLiftable(&mut self, pt: &mut Point16U) -> u8 {
        let direction = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(K_DUNGEON_QUERY_IF_TILE_LIFTABLE_X[direction] as u16);
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(K_DUNGEON_QUERY_IF_TILE_LIFTABLE_Y[direction] as u16);
        pt.x = x;
        pt.y = y;
        write_le_u16(&mut self.ram, R16, y);
        write_le_u16(&mut self.ram, R18, x);

        let x = x & 0x01f8;
        let y = y & 0x01f8;
        let xy = (y << 3)
            | (x >> 3)
            | if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
                0x1000
            } else {
                0
            };
        let attr = self.ram[DUNG_BG2_ATTR_TABLE + xy as usize];
        assert_eq!(attr & 0x70, 0x70);
        let attr = attr & 0x0f;
        let rt = read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_STATE + attr as usize * 2);

        if rt & 0xf0f0 == 0x1010 {
            let misc = u16::from(attr) * 2;
            write_le_u16(&mut self.ram, DUNG_MISC_OBJS_INDEX, misc);
            let tilemap = read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + attr as usize * 2);
            self.RevealPotItem(xy, tilemap);
            self.RoomDraw_16x16Single(misc as u8);
            self.ManipBlock_Something(pt);
            K_DUNGEON_QUERY_IF_TILE_LIFTABLE_RV[(rt & 0x0f) as usize] as u8
        } else if rt & 0xf0f0 == 0x2020 {
            self.ThievesAttic_DrawLightenedHole(
                xy,
                (u16::from(attr).wrapping_sub(rt & 0x0f)).wrapping_mul(2),
                pt,
            )
        } else {
            0
        }
    }

    pub(super) fn ThievesAttic_DrawLightenedHole(
        &mut self,
        pos6: u16,
        a: u16,
        pt: &mut Point16U,
    ) -> u8 {
        write_le_u16(&mut self.ram, DUNG_MISC_OBJS_INDEX, a);
        let tilemap = read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + (a >> 1) as usize * 2);
        self.RevealPotItem(pos6, tilemap);
        self.RoomDraw_16x16Single(a as u8);
        self.RoomDraw_16x16Single(a.wrapping_add(2) as u8);
        self.RoomDraw_16x16Single(a.wrapping_add(4) as u8);
        self.RoomDraw_16x16Single(a.wrapping_add(6) as u8);
        self.ManipBlock_Something(pt);
        0x55
    }

    pub(super) fn HandleItemTileAction_Dungeon(&mut self, x: u16, y: u16) -> u8 {
        if self.ram[LINK_ITEM_IN_HAND] & 2 == 0
            && (self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_BREAK_POTS_WITH_SWORD_DUNGEON
                == 0
                || self.ram[BUTTON_B_FRAMES] == 0
                || self.ram[LINK_SWORD_TYPE] == 1)
        {
            return 0;
        }

        let pos = (y & 0x01f8).wrapping_mul(8).wrapping_add(x)
            + if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
                0x1000
            } else {
                0
            };
        let tile = self.ram[DUNG_BG2_ATTR_TABLE + pos as usize];
        if tile & 0xf0 == 0x70 {
            let tile2 = read_le_u16(
                &self.ram,
                DUNG_REPLACEMENT_TILE_STATE + (tile & 0x0f) as usize * 2,
            );
            if tile2 & 0xf0f0 == 0x4040 {
                if self.ram[LINK_ITEM_IN_HAND] & 2 == 0 {
                    return 0;
                }
                write_le_u16(
                    &mut self.ram,
                    DUNG_MISC_OBJS_INDEX,
                    u16::from(tile & 0x0f) * 2,
                );
                self.RoomDraw_16x16Single(self.ram[DUNG_MISC_OBJS_INDEX]);
                self.ram[SOUND_EFFECT_1] = 0x11;
            } else if tile2 & 0xf0f0 == 0x1010 {
                write_le_u16(
                    &mut self.ram,
                    DUNG_MISC_OBJS_INDEX,
                    u16::from(tile & 0x0f) * 2,
                );
                let tilemap = read_le_u16(
                    &self.ram,
                    DUNG_OBJECT_TILEMAP_POS + (tile & 0x0f) as usize * 2,
                );
                self.RevealPotItem(pos, tilemap);
                self.RoomDraw_16x16Single(self.ram[DUNG_MISC_OBJS_INDEX]);
                let mut pt = Point16U { x: 0, y: 0 };
                self.ManipBlock_Something(&mut pt);
                self.ram[DUNG_SECRETS_UNK1_DUNGEON] |= 0x80;
                self.sprite_spawn_immediately_smashed_terrain(1, pt.x, pt.y);
                self.ancilla_add_bush_poof(pt.x, pt.y);
            }
        }
        0
    }

    pub(super) fn ManipBlock_Something(&mut self, pt: &mut Point16U) {
        let index = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX) as usize >> 1;
        let pos = read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + index * 2);
        pt.x = (self.player_state_view().x() & 0xfe00) | ((pos & 0x007e) << 2);
        pt.y = (self.player_state_view().y() & 0xfe00) | ((pos & 0x1f80) >> 4);
    }

    pub(super) fn RevealPotItem(&mut self, pos6: u16, pos4: u16) {
        self.ram[DUNG_SECRETS_UNK1_DUNGEON] = 0;
        let room = self.world_state_view().dungeon_room() as usize;
        let secrets = self
            .asset_raw(50)
            .expect("missing dungeon secrets asset")
            .to_vec();
        let mut src = read_le_u16(&secrets, room * 2) as usize;
        let mut index = 0usize;
        loop {
            let test_pos = read_le_u16(&secrets, src);
            if test_pos == 0xffff {
                return;
            }
            assert_eq!(test_pos & 0x8000, 0);
            if test_pos == pos4 {
                break;
            }
            src += 3;
            index += 1;
        }

        let data = secrets[src + 2];
        if data == 0 {
            return;
        }
        if data < 0x80 {
            if data != 8 {
                let mask = 1u16 << index;
                let pot_addr = POTS_REVEALED_IN_ROOM_DUNGEON + room * 2;
                let revealed = read_le_u16(&self.ram, pot_addr);
                if revealed & mask != 0 {
                    return;
                }
                write_le_u16(&mut self.ram, pot_addr, revealed | mask);
            }
            self.ram[DUNG_SECRETS_UNK1_DUNGEON] |= data;
        } else if data != 0x88 {
            let j = self.ram[DUNG_BG2_ATTR_TABLE + pos6 as usize] & 0x0f;
            let mut k = (u16::from(j).wrapping_sub(
                read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_STATE + j as usize * 2) & 0x0f,
            )) as usize;
            write_le_u16(&mut self.ram, DUNG_MISC_OBJS_INDEX, (2 * k) as u16);
            self.ram[SOUND_EFFECT_2] = 0x1b;
            let src_words = self.read_predefined_tile_words(0x05ba, 16);
            for chunk in src_words.chunks_exact(4).take(4) {
                write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UL + k * 2, chunk[0]);
                write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LL + k * 2, chunk[1]);
                write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UR + k * 2, chunk[2]);
                write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LR + k * 2, chunk[3]);
                k += 1;
            }
        } else {
            let k = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX) as usize >> 1;
            write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UL + k * 2, 0x0d0b);
            write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LL + k * 2, 0x0d1b);
            write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_UR + k * 2, 0x4d0b);
            write_le_u16(&mut self.ram, REPLACEMENT_TILEMAP_LR + k * 2, 0x4d1b);
        }
    }

    pub(super) fn PushBlock_CheckForPit(&mut self, y: u8) {
        let y = (y >> 1) as usize;
        let tilemap = read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + y * 2);
        if tilemap & 0x4000 == 0 {
            self.ram[DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED] ^= 1;
        }

        let p = (tilemap & 0x3fff) >> 1;
        let attr = self.ram[DUNG_BG2_ATTR_TABLE + p as usize];
        if attr == 0x20 {
            self.ram[SOUND_EFFECT_1] = 0x20;
            let k = (read_le_u16(&self.ram, DUNG_OBJECT_POS_IN_OBJDATA + y * 2) >> 2) as usize;
            let room = u16::from(self.ram[DUNG_HDR_TRAVEL_DESTINATIONS]);
            write_le_u16(&mut self.ram, MOVABLE_BLOCK_DATAS + k * 4, room);
            write_le_u16(&mut self.ram, MOVABLE_BLOCK_DATAS + k * 4 + 2, tilemap);
            return;
        }

        let i = usize::from(self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + 1] == y as u8 + 1);
        self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + i] = 0;

        if attr == 0x23 {
            let related = read_le_u16(&self.ram, DUNG_FLAG_TRAPDOORS_DOWN) ^ 1;
            write_le_u16(&mut self.ram, BLOCK_TRAP_CHECK_FLAG, related);
            write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_STATE + y * 2, 4);
        } else {
            write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_STATE + y * 2, 0xffff);
        }
        self.Dungeon_Store2x2(p, 0x0922, 0x0932, 0x0923, 0x0933, 0x27);
    }

    pub(super) fn PushBlock_Slide(&mut self, j: u8) {
        if self.frame_control_view().submodule() != 0 {
            return;
        }
        let i = usize::from(
            (i32::from(self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + 1]) - 1) * 2 == i32::from(j),
        );
        self.ram[PUSHEDBLOCKS_MAYBE_TIMEOUT] = 9;
        self.ram[PUSHED_BLOCK_MODE] = 0;
        self.PushBlock_ApplyVelocity(i as u8);
        let y = u16::from(self.ram[PUSHEDBLOCKS_Y_LO + i * 2])
            | (u16::from(self.ram[PUSHEDBLOCKS_Y_HI + i * 2]) << 8);
        let x = u16::from(self.ram[PUSHEDBLOCKS_X_LO + i * 2])
            | (u16::from(self.ram[PUSHEDBLOCKS_X_HI + i * 2]) << 8);
        self.PushBlock_HandleCollision(i as u8, x, y);
    }

    pub(super) fn PushBlock_HandleFalling(&mut self, y: u8) {
        let y = (y >> 1) as usize;
        self.ram[PUSHEDBLOCKS_MAYBE_TIMEOUT] = self.ram[PUSHEDBLOCKS_MAYBE_TIMEOUT].wrapping_sub(1);
        if !(self.ram[PUSHEDBLOCKS_MAYBE_TIMEOUT] as i8).is_negative() {
            return;
        }
        self.ram[PUSHEDBLOCKS_MAYBE_TIMEOUT] = 9;
        self.ram[PUSHED_BLOCK_MODE] = self.ram[PUSHED_BLOCK_MODE].wrapping_add(1);
        if self.ram[PUSHED_BLOCK_MODE] == 4 {
            self.ram[DUNG_REPLACEMENT_TILE_STATE + y * 2] = 0;
            self.ram[PUSHED_BLOCK_MODE] = 0;
            let i = usize::from(
                i32::from(self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + 1]) - 1 == y as i32,
            );
            self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + i] = 0;
        }
    }

    pub(super) fn PushBlock_ApplyVelocity(&mut self, i: u8) {
        const PUSHED_BLOCK_DIR_MASK: [u8; 4] = [0x08, 0x04, 0x02, 0x01];
        const PUSH_BLOCK_TAB1: [u8; 4] = [0x00, 0x00, 0xe0, 0x20];
        const PUSH_BLOCK_TAB2: [u8; 4] = [0xe0, 0x20, 0x00, 0x00];

        let i = i as usize;
        let facing = (read_le_u16(&self.ram, PUSHEDBLOCK_FACING + i * 2) as u8) >> 1;
        let m = PUSHED_BLOCK_DIR_MASK[facing as usize];
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;

        let o;
        if m & 3 != 0 {
            let vel = if m & 2 != 0 { -12i32 } else { 12i32 };
            self.ram[LINK_ACTUAL_VEL_X] = vel as i8 as u8;
            o = (u32::from(self.ram[PUSHEDBLOCKS_SUBPIXEL + i * 2])
                | (u32::from(self.ram[PUSHEDBLOCKS_X_LO + i * 2]) << 8)
                | (u32::from(self.ram[PUSHEDBLOCKS_X_HI + i * 2]) << 16))
                .wrapping_add((vel * 16) as u32);
            self.ram[PUSHEDBLOCKS_SUBPIXEL + i * 2] = o as u8;
            self.ram[PUSHEDBLOCKS_X_LO + i * 2] = (o >> 8) as u8;
            self.ram[PUSHEDBLOCKS_X_HI + i * 2] = (o >> 16) as u8;
        } else {
            let vel = if m & 8 != 0 { -12i32 } else { 12i32 };
            self.ram[LINK_ACTUAL_VEL_Y] = vel as i8 as u8;
            o = (u32::from(self.ram[PUSHEDBLOCKS_SUBPIXEL + i * 2])
                | (u32::from(self.ram[PUSHEDBLOCKS_Y_LO + i * 2]) << 8)
                | (u32::from(self.ram[PUSHEDBLOCKS_Y_HI + i * 2]) << 16))
                .wrapping_add((vel * 16) as u32);
            self.ram[PUSHEDBLOCKS_SUBPIXEL + i * 2] = o as u8;
            self.ram[PUSHEDBLOCKS_Y_LO + i * 2] = (o >> 8) as u8;
            self.ram[PUSHEDBLOCKS_Y_HI + i * 2] = (o >> 16) as u8;
        }

        if ((o >> 8) as u8 & 0x0f) == self.ram[PUSHEDBLOCKS_TARGET + i * 2] {
            let j = self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + i].wrapping_sub(1) as usize;
            let state = read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_STATE + j * 2).wrapping_add(1);
            write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_STATE + j * 2, state);
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !0x04;
            self.ram[PLAYER_DEFENSE_FLAGS] &= !0x04;
        }

        let x = u16::from(self.ram[PUSHEDBLOCKS_X_LO + i * 2])
            | (u16::from(self.ram[PUSHEDBLOCKS_X_HI + i * 2]) << 8);
        let y = u16::from(self.ram[PUSHEDBLOCKS_Y_LO + i * 2])
            | (u16::from(self.ram[PUSHEDBLOCKS_Y_HI + i * 2]) << 8);
        for j in (0..16usize).rev() {
            if self.ram[SPRITE_STATE + j] >= 9 {
                let sx = u16::from(self.ram[SPRITE_X_LO + j])
                    | (u16::from(self.ram[SPRITE_X_HI + j]) << 8);
                let sy = u16::from(self.ram[SPRITE_Y_LO + j])
                    | (u16::from(self.ram[SPRITE_Y_HI + j]) << 8);
                if x.wrapping_sub(sx).wrapping_add(0x10) < 0x20
                    && y.wrapping_sub(sy).wrapping_add(0x10) < 0x20
                {
                    self.ram[SPRITE_F + j] = 8;
                    let k = facing as usize;
                    self.ram[SPRITE_X_RECOIL + j] = PUSH_BLOCK_TAB1[k];
                    self.ram[SPRITE_Y_RECOIL_DUNGEON + j] = PUSH_BLOCK_TAB2[k];
                }
            }
        }
    }

    pub(super) fn PushBlock_HandleCollision(&mut self, i: u8, x: u16, y: u16) {
        const PUSH_BLOCK_A: [u16; 4] = [0, 0, 8, 8];
        const PUSH_BLOCK_B: [u16; 4] = [15, 15, 23, 23];
        const PUSH_BLOCK_C: [u16; 4] = [0, 0, 0, 0];
        const PUSH_BLOCK_D: [u16; 4] = [15, 15, 15, 15];
        const PUSH_BLOCK_E: [u16; 4] = [8, 24, 0, 16];
        const PUSH_BLOCK_F: [u16; 4] = [15, 0, 15, 0];

        let i = i as usize;
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (self.player_state_view().y() >> 8) as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_HI] = (self.player_state_view().x() >> 8) as u8;

        let mut dir = 3i32;
        let mut m = self.ram[LINK_DIRECTION] & 0x0f;
        while m & 1 == 0 {
            m >>= 1;
            dir -= 1;
            if dir < 0 {
                return;
            }
        }
        let dir = dir as usize;
        let l = if dir < 2 {
            self.player_state_view().x()
        } else {
            self.player_state_view().y()
        };
        let o = if dir < 2 { x } else { y };
        let r0 = l.wrapping_add(PUSH_BLOCK_A[dir]);
        let r2 = l.wrapping_add(PUSH_BLOCK_B[dir]);
        let r4 = o.wrapping_add(PUSH_BLOCK_C[dir]);
        let r6 = o.wrapping_add(PUSH_BLOCK_D[dir]);
        let coord_addr = if dir < 2 { LINK_Y_COORD } else { LINK_X_COORD };
        let r8 = read_le_u16(&self.ram, coord_addr).wrapping_add(PUSH_BLOCK_E[dir]);
        let r10 = (if dir < 2 { y } else { x }).wrapping_add(PUSH_BLOCK_F[dir]);

        self.ram[PLAYER_DEFENSE_FLAGS] &= !4;
        if (r0 >= r4 && r0 < r6) || (r2 >= r4 && r2 < r6) {
            if self.ram[LINK_DIRECTION_FACING]
                == read_le_u16(&self.ram, PUSHEDBLOCK_FACING + i * 2) as u8
            {
                self.ram[PLAYER_DEFENSE_FLAGS] |=
                    if self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + i] != 0 {
                        4
                    } else {
                        1
                    };
            }
            let diff = r8.wrapping_sub(r10);
            if (dir & 1 != 0 && r8 >= r10 && diff < 8) || (dir & 1 == 0 && diff >= 0xfff8) {
                let coord = read_le_u16(&self.ram, coord_addr).wrapping_sub(diff);
                write_le_u16(&mut self.ram, coord_addr, coord);
                let vel_addr = if dir & 2 != 0 { LINK_X_VEL } else { LINK_Y_VEL };
                self.ram[vel_addr] = self.ram[vel_addr].wrapping_sub(diff as u8);
            }
        }
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn Dungeon_MapVramAddr(&self, pos: u16) -> u16 {
        self.Dungeon_MapVramAddrNoSwap(pos).swap_bytes()
    }

    pub(super) fn Dungeon_MapVramAddrNoSwap(&self, pos: u16) -> u16 {
        let pos = pos.wrapping_mul(2);
        ((pos & 0x40) << 4) | ((pos & 0x303f) >> 1) | ((pos & 0x0f80) >> 2)
    }

    pub(super) fn Dungeon_GetTeleMsg(&self, room: usize) -> u16 {
        self.asset_u16(9, room)
    }

    pub(super) fn GetDungPalInfo(&self, idx: usize) -> DungPalInfo {
        DUNG_PAL_INFOS.get(idx).copied().unwrap_or_default()
    }

    pub(super) fn Dungeon_IsPitThatHurtsPlayer(&self) -> bool {
        let room = self.world_state_view().dungeon_room();
        let Some(data) = self.asset_raw(10) else {
            return false;
        };
        data.chunks_exact(2)
            .any(|entry| read_word_from_slice(entry, 0) == room)
    }

    pub(super) fn Door_Up_EntranceDoor(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Up_EntranceDoor assert");
    }

    pub(super) fn Door_Down_EntranceDoor(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Down_EntranceDoor assert");
    }

    pub(super) fn Door_Left_EntranceDoor(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Left_EntranceDoor assert");
    }

    pub(super) fn Door_Right_EntranceDoor(&mut self, _dsto: u16) {
        // C asserts for entrance-door helpers because they pass the wrong
        // value to RoomDraw_FlagDoorsAndGetFinalType.
        panic!("Door_Right_EntranceDoor assert");
    }

    pub(super) fn Door_Draw_Helper4(&mut self, door_type: u8, dsto: u16) {
        let mut t = self.RoomDraw_FlagDoorsAndGetFinalType(1, door_type, dsto);
        if t & 0x100 != 0 {
            return;
        }

        if t == DOOR_TYPE_1E as u16 || t == DOOR_TYPE_36 as u16 || t == DOOR_TYPE_38 as u16 {
            let new_type = if t == DOOR_TYPE_38 as u16 {
                DOOR_TYPE_SHUTTERS_TWO_WAY
            } else {
                DOOR_TYPE_REGULAR
            };
            self.room_rewrite_last_door_type(new_type);
            t = new_type as u16;
        }

        if let Some(&src) = DOOR_TYPE_SRC_DOWN.get(t as usize >> 1) {
            for i in 0..4 {
                let d = dsto + i as u16;
                self.room_write_current(d + 64, self.tile_word(src as usize, i * 3));
                self.room_write_current(d + 128, self.tile_word(src as usize, i * 3 + 1));
                self.room_write_current(d + 192, self.tile_word(src as usize, i * 3 + 2));
            }
        }
    }

    pub(super) fn GetRoomDoorInfo(&self, room: usize) -> Option<&[u8]> {
        let offset = self.asset_u16(5, room) as usize;
        self.asset_raw(3)?.get(offset..)
    }

    pub(super) fn GetRoomHeaderPtr(&self, room: usize) -> Option<&[u8]> {
        self.dungeon_room_header(room)
    }

    pub(super) fn GetDefaultRoomLayout(&self, index: usize) -> Option<&[u8]> {
        self.default_room_layout(index)
    }

    pub(super) fn GetDungeonRoomLayout(&self, room: usize) -> Option<&[u8]> {
        self.dungeon_room_layout(room)
    }

    pub(super) fn Dung_TagRoutine_0x22_0x3B(&mut self, k: usize, j: u8) {
        if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x0100 != 0 {
            self.ram[DUNG_HDR_TAG + k] = 0;
            self.ram[DUNG_OVERLAY_TO_LOAD] = j;
            write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, 0);
            self.frame_control_view_mut().set_subsubmodule(0);
            self.ram[SOUND_EFFECT_2] = 0x1b;
            self.frame_control_view_mut().set_submodule(3);
        }
    }

    pub(super) fn Dung_TagRoutine_0x1B(&mut self, _k: usize) {}

    pub(super) fn RoomTag_NorthWestTrigger(&mut self, k: usize) {
        if self.player_state_view().x() & 0x0100 == 0 && self.player_state_view().y() & 0x0100 == 0
        {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn RoomTag_Holes0(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(1);
    }

    pub(super) fn Dung_TagRoutine_0x23(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(3);
    }

    pub(super) fn Dung_TagRoutine_0x2A(&mut self, k: usize) {
        if self.player_state_view().x() & 0x0100 != 0 && self.player_state_view().y() & 0x0100 == 0
        {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x2B(&mut self, k: usize) {
        if self.player_state_view().x() & 0x0100 == 0 && self.player_state_view().y() & 0x0100 != 0
        {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x2C(&mut self, k: usize) {
        if self.player_state_view().x() & 0x0100 != 0 && self.player_state_view().y() & 0x0100 != 0
        {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x2D(&mut self, k: usize) {
        if self.player_state_view().x() & 0x0100 == 0 {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x2E(&mut self, k: usize) {
        if self.player_state_view().x() & 0x0100 != 0 {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x2F(&mut self, k: usize) {
        if self.player_state_view().y() & 0x0100 == 0 {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x30(&mut self, k: usize) {
        if self.player_state_view().y() & 0x0100 != 0 {
            self.RoomTag_QuadrantTrigger(k);
        }
    }

    pub(super) fn Dung_TagRoutine_0x34(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(6);
    }

    pub(super) fn Dung_TagRoutine_0x35(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(8);
    }

    pub(super) fn Dung_TagRoutine_0x36(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(10);
    }

    pub(super) fn Dung_TagRoutine_0x37(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(12);
    }

    pub(super) fn Dung_TagRoutine_0x39(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(14);
    }

    pub(super) fn Dung_TagRoutine_0x3A(&mut self, _k: usize) {
        self.Dung_TagRoutine_Func2(16);
    }

    pub(super) fn Dung_TagRoutine_Func2(&mut self, mut av: u8) {
        if self.ram[DUNG_OVERLAY_TO_LOAD] == 0 {
            self.ram[DUNG_OVERLAY_TO_LOAD] = av;
        }

        let mut yv = 0;
        if self.RoomTag_CheckForPressedSwitch(&mut yv) {
            av = av.wrapping_add(yv);
            if av != self.ram[DUNG_OVERLAY_TO_LOAD] {
                self.ram[DUNG_OVERLAY_TO_LOAD] = av;
                write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, 0);
                self.frame_control_view_mut().set_subsubmodule(0);
                self.ram[SOUND_EFFECT_2] = 27;
                self.frame_control_view_mut().set_submodule(3);
                self.ram[MOVING_WALL_TORCH_BLINK_PHASE] ^= 1;
                self.Dungeon_RestoreStarTileChr();
            }
        }
    }

    pub(super) fn RoomTag_ChestHoles0(&mut self, k: usize) {
        self.Dung_TagRoutine_0x22_0x3B(k, 0);
    }

    pub(super) fn Dung_TagRoutine_0x3B(&mut self, k: usize) {
        self.Dung_TagRoutine_0x22_0x3B(k, 0x12);
    }

    pub(super) fn RoomTag_Holes2(&mut self, k: usize) {
        let mut yv = 0;
        if !self.RoomTag_CheckForPressedSwitch(&mut yv) {
            return;
        }

        self.ram[DUNG_HDR_TAG + k] = 0;
        self.ram[DUNG_OVERLAY_TO_LOAD] = 5;
        write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, 0);
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[SOUND_EFFECT_2] = 0x1b;
        self.frame_control_view_mut().set_submodule(3);
    }

    pub(super) fn RoomTag_QuadrantTrigger(&mut self, k: usize) {
        let tag = self.ram[DUNG_HDR_TAG + k];
        if tag >= 0x0b {
            if tag >= 0x29 {
                if self.sprite_check_if_screen_is_clear() {
                    self.RoomTag_OperateChestReveal(k);
                }
            } else {
                let down = self.ram[DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED] ^ 1;
                if down != self.ram[DUNG_FLAG_TRAPDOORS_DOWN] {
                    self.ram[DUNG_FLAG_TRAPDOORS_DOWN] = down;
                    self.ram[SOUND_EFFECT_2] = 37;
                    self.frame_control_view_mut().set_submodule(5);
                    write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, 0);
                    write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
                }
            }
        } else if self.sprite_check_if_screen_is_clear() {
            self.Dung_TagRoutine_TrapdoorsUp();
        }
    }

    pub(super) fn RoomTag_RoomTrigger(&mut self, k: usize) {
        if self.ram[DUNG_HDR_TAG + k] == 10 {
            if self.sprite_check_if_room_is_clear() {
                self.Dung_TagRoutine_TrapdoorsUp();
            }
        } else if self.sprite_check_if_room_is_clear() {
            self.RoomTag_OperateChestReveal(k);
        }
    }

    pub(super) fn RoomTag_RekillableBoss(&mut self, k: usize) {
        if self.sprite_check_if_room_is_clear() {
            self.ram[FLAG_BLOCK_LINK_MENU] = 0;
            self.ram[DUNG_HDR_TAG + k] = 0;
        }
    }

    pub(super) fn RoomTag_RoomTrigger_BlockDoor(&mut self, _k: usize) {
        if self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] != 0
            && read_le_u16(&self.ram, DUNG_FLAG_TRAPDOORS_DOWN) != 0
        {
            write_le_u16(&mut self.ram, DUNG_FLAG_TRAPDOORS_DOWN, 0);
            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, 0);
            write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
            self.frame_control_view_mut().set_submodule(5);
        }
    }

    pub(super) fn RoomTag_PrizeTriggerDoorDoor(&mut self, k: usize) {
        let prizes = if self.ram[SAVEGAME_IS_DARKWORLD] != 0 {
            self.ram[LINK_HAS_CRYSTALS]
        } else {
            self.ram[LINK_WHICH_PENDANTS]
        };
        let palace = (self.ram[CUR_PALACE_INDEX_X2] >> 1) as usize;
        if prizes & K_DUNGEON_CRYSTAL_PENDANT_BIT[palace] != 0 {
            write_le_u16(&mut self.ram, DUNG_FLAG_TRAPDOORS_DOWN, 0);
            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, 0);
            write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
            self.frame_control_view_mut().set_submodule(5);
            self.ram[DUNG_HDR_TAG + k] = 0;
        }
    }

    pub(super) fn RoomTag_TorchPuzzleDoor(&mut self, _k: usize) {
        let mut lit = 0;
        for i in 0..16 {
            if read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + i * 2) & 0x8000 != 0 {
                lit += 1;
            }
        }
        let down = u16::from(lit < 4);
        if down != read_le_u16(&self.ram, DUNG_FLAG_TRAPDOORS_DOWN) {
            write_le_u16(&mut self.ram, DUNG_FLAG_TRAPDOORS_DOWN, down);
            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, 0);
            write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
            self.ram[SOUND_EFFECT_2] = 0x1b;
            self.frame_control_view_mut().set_submodule(5);
        }
    }

    pub(super) fn RoomTag_Switch_ExplodingWall(&mut self, k: usize) {
        let mut yv = 0;
        if self.RoomTag_MaybeCheckShutters(&mut yv) {
            self.Dung_TagRoutine_BlastWallStuff(k);
        }
    }

    pub(super) fn RoomTag_PullSwitchExplodingWall(&mut self, k: usize) {
        if self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] != 0 {
            self.Dung_TagRoutine_BlastWallStuff(k);
        }
    }

    pub(super) fn Dung_TagRoutine_BlastWallStuff(&mut self, k: usize) {
        const BLAST_WALL_TAB0: [u8; 5] = [4, 6, 0, 0, 2];
        const BLAST_WALL_TAB1: [u16; 5] = [0, 0x0a, 0, 0, 0x0280];

        self.ram[DUNG_HDR_TAG + k] = 0;

        let mut door = 0usize;
        while self.ram[DOOR_TYPE_AND_SLOT + door * 2] & !1 != 0x30 {
            door += 1;
        }
        write_le_u16(
            &mut self.ram,
            CRUSH_WALL_DOOR_INDEX_X2_DUNGEON,
            (door * 2) as u16,
        );

        let mut i = (((self.player_state_view().y() >> 8) & 1) + 1) * 2;
        if self.ram[DUNG_DOOR_DIRECTION + door * 2] & 2 != 0 {
            i = (self.player_state_view().x() >> 8) & 1;
        }

        write_le_u16(
            &mut self.ram,
            MESSAGING_BUF_DUNGEON + 0x1c,
            u16::from(BLAST_WALL_TAB0[i as usize]),
        );
        let pos = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2)
            .wrapping_add(BLAST_WALL_TAB1[i as usize]);
        let x =
            ((pos & 0x007e) << 2).wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_H_COPY));
        let y =
            ((pos & 0x1f80) >> 4).wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_V_COPY));
        write_le_u16(&mut self.ram, MESSAGING_BUF_DUNGEON + 0x1a, x);
        write_le_u16(&mut self.ram, MESSAGING_BUF_DUNGEON + 0x18, y);
        self.ram[SOUND_EFFECT_2] = 27;
        self.ram[CRUSH_WALL_PROGRESS_DUNGEON] = 1;
        self.ancilla_add_blast_wall();
    }

    pub(super) fn RoomTag_WaterOn(&mut self, _k: usize) {
        if self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] != 0 {
            self.ram[SOUND_EFFECT_2] = 0x1b;
            self.ram[SOUND_EFFECT_1] = 0x2f;
            self.frame_control_view_mut().set_submodule(12);
            self.frame_control_view_mut().set_subsubmodule(0);
            self.ram[DUNG_FLOOR_Y_OFFS] = 1;
            self.ram[DUNG_HDR_TAG + 1] = 0;
            let save_bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) | 0x0800;
            write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, save_bits);
            self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = 0;
            self.ram[DUNG_CUR_QUADRANT_UPLOAD] = 0;
        }
    }

    pub(super) fn RoomTag_WaterOff(&mut self, _k: usize) {
        if self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] == 0 {
            return;
        }

        self.ram[W12SEL_COPY] = 3;
        self.ram[W34SEL_COPY] = 0;
        self.ram[WOBJSEL_COPY] = 0;
        self.ram[TMW_COPY] = 22;
        self.ram[TSW_COPY] = 1;
        self.ram[TURN_ON_OFF_WATER_CTR] = 1;
        self.AdjustWaterHDMAWindow();
        self.frame_control_view_mut().set_submodule(11);
        self.ram[PALETTE_FILTER_COUNTDOWN] = 0;
        self.ram[DARKENING_OR_LIGHTENING_SCREEN] = 0;
        self.ram[MOSAIC_TARGET_LEVEL] = 31;
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        self.ram[DUNG_HDR_TAG + 1] = 0;
        let save_bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) | 0x0800;
        write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, save_bits);
        self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = 0;

        let dsto = ((read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_DUNGEON) & 0x01ff)
            .wrapping_sub(0x10)
            << 3)
            | ((read_le_u16(&self.ram, WATER_HDMA_WINDOW_X_DUNGEON) & 0x01ff).wrapping_sub(0x10)
                >> 3);
        self.DrawWaterThing(dsto, 0x1438);
        self.dungeon_prep_overlay_dma_next_prep(0, dsto.wrapping_mul(2));
        self.ram[SOUND_EFFECT_2] = 0x1b;
        self.ram[SOUND_EFFECT_1] = 0x2e;
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;
    }

    pub(super) fn RoomTag_WaterGate(&mut self, _k: usize) {
        if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x0800 != 0
            || self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] == 0
        {
            return;
        }

        self.frame_control_view_mut().set_submodule(13);
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[DUNG_HDR_TAG + 1] = 0;
        let save_bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) | 0x0800;
        write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, save_bits);
        self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = 0;
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON] = 0;
        self.ram[SPOTLIGHT_WINDOW_Y_BUFFER] = 0;
        self.ram[W12SEL_COPY] = 3;
        self.ram[W34SEL_COPY] = 0;
        self.ram[WOBJSEL_COPY] = 0;
        self.ram[TMW_COPY] = 0x16;
        self.ram[TSW_COPY] = 1;
        self.ram[CGWSEL_COPY] = 2;
        self.ram[CGADSUB_COPY] = 0x62;
        self.ram[SAVE_OW_EVENT_INFO_DUNGEON + 0x3b] |= 0x20;
        self.ram[SAVE_OW_EVENT_INFO_DUNGEON + 0x7b] |= 0x20;
        let dung_info = read_le_u16(&self.ram, SAVE_DUNG_INFO + 0x28 * 2) | 0x0100;
        write_le_u16(&mut self.ram, SAVE_DUNG_INFO + 0x28 * 2, dung_info);

        self.RoomTag_OperateWaterFlooring();
        let watergate_pos = read_le_u16(&self.ram, WATERGATE_POS);
        let hdma0 = ((watergate_pos & 0x007e) << 2)
            .wrapping_add(u16::from(self.ram[DUNG_DRAW_WIDTH_INDICATOR]) * 16)
            .wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_H_COPY))
            .wrapping_add(40);
        write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_X_DUNGEON, hdma0);
        let y = (watergate_pos & 0x1f80) >> 4;
        write_le_u16(&mut self.ram, WATERGATE_SPOTLIGHT_Y_UPPER, y);
        write_le_u16(&mut self.ram, SPOTLIGHT_Y_UPPER, y);
        let hdma1 = y.wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_V_COPY));
        write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_Y_DUNGEON, hdma1);
        write_le_u16(&mut self.ram, WATER_HDMA_WINDOW_X_RADIUS_DUNGEON, 0);
        self.ram[SOUND_EFFECT_2] = 0x1b;
        self.ram[SOUND_EFFECT_1] = 0x2f;
    }

    pub(super) fn RoomTag_OperateWaterFlooring(&mut self) {
        write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, 0);
        let mut layout = 0usize;
        loop {
            write_le_u16(&mut self.ram, DUNG_DRAW_WIDTH_INDICATOR, 0);
            write_le_u16(&mut self.ram, DUNG_DRAW_HEIGHT_INDICATOR, 0);
            let t = u16::from(K_WATERGATE_LAYOUT[layout])
                | (u16::from(K_WATERGATE_LAYOUT[layout + 1]) << 8);
            if t == 0xffff {
                break;
            }
            write_le_u16(&mut self.ram, DUNG_DRAW_WIDTH_INDICATOR, (t & 3) + 1);
            write_le_u16(
                &mut self.ram,
                DUNG_DRAW_HEIGHT_INDICATOR,
                ((t >> 8) & 3) + 1,
            );
            let load = read_le_u16(&self.ram, DUNG_LOAD_PTR_OFFS).wrapping_add(3);
            write_le_u16(&mut self.ram, DUNG_LOAD_PTR_OFFS, load);
            layout += 3;

            let mut dsto2 = ((t & 0x00fc) >> 2) | ((t >> 10) << 6);
            let mut height = read_le_u16(&self.ram, DUNG_DRAW_HEIGHT_INDICATOR);
            while height != 0 {
                let mut dsto = dsto2;
                let mut width = read_le_u16(&self.ram, DUNG_DRAW_WIDTH_INDICATOR);
                while width != 0 {
                    for _ in 0..2 {
                        for y in 0..2u16 {
                            for x in 0..4u16 {
                                let tile = self.tile_word(0x0110, (y * 4 + x) as usize);
                                self.room_write_bg(0x4000, dsto + x + y * 64, tile);
                            }
                        }
                        dsto = dsto.wrapping_add(xy(0, 2) as u16);
                    }
                    dsto = dsto
                        .wrapping_add(xy(4, 0) as u16)
                        .wrapping_sub(xy(0, 4) as u16);
                    width -= 1;
                }
                dsto2 = dsto2.wrapping_add(xy(0, 4) as u16);
                height -= 1;
            }
        }
    }

    pub(super) fn RoomTag_GetHeartForPrize(&mut self, k: usize) {
        const BOSS_FINISHED_FALLING_ITEM: [u8; 13] = [0, 0, 1, 2, 0, 6, 6, 6, 6, 6, 3, 6, 6];

        if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x8000 == 0 {
            return;
        }
        let prizes = if self.ram[SAVEGAME_IS_DARKWORLD] != 0 {
            self.ram[LINK_HAS_CRYSTALS]
        } else {
            self.ram[LINK_WHICH_PENDANTS]
        };
        let palace = (self.ram[CUR_PALACE_INDEX_X2] >> 1) as usize;
        if prizes & K_DUNGEON_CRYSTAL_PENDANT_BIT[palace] == 0 {
            self.ram[MOVING_WALL_TORCH_UPDATE_FLAG] = 128;
            if self.ancilla_spawn_falling_prize(BOSS_FINISHED_FALLING_ITEM[palace]) < 0 {
                return;
            }
        }
        self.ram[DUNG_HDR_TAG + k] = 0;
    }

    pub(super) fn RoomTag_Agahnim(&mut self, _k: usize) {
        if self.ram[SAVE_OW_EVENT_INFO_DUNGEON + 0x5b] & 0x20 == 0
            && read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x8000 != 0
        {
            self.Palette_RevertTranslucencySwap();
            self.ram[DUNG_HDR_TAG] = 0;
            self.prepare_dungeon_exit_from_boss_fight();
        }
    }

    pub(super) fn RoomTag_GanonDoor(&mut self, _tagidx: usize) {
        for k in (0..16).rev() {
            if self.ram[SPRITE_STATE + k] == 4
                || (self.ram[SPRITE_FLAGS4 + k] & 64 == 0 && self.ram[SPRITE_STATE + k] != 0)
            {
                return;
            }
        }

        if self.ram[LINK_PLAYER_HANDLER_STATE] != 1 {
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = 26;
            self.frame_control_view_mut().set_submodule(26);
            self.frame_control_view_mut().set_subsubmodule(0);
            self.ram[DUNG_HDR_TAG] = 0;
            self.ram[LINK_FORCE_HOLD_SWORD_UP] = 1;
            self.ram[BUTTON_MASK_B_Y] = 0;
            self.ram[BUTTON_B_FRAMES] = 0;
            write_le_u16(&mut self.ram, R16, 0x0364);
        }
    }

    pub(super) fn RoomTag_SwitchTrigger_HoldDoor(&mut self, _k: usize) {
        let mut i = 0usize;
        let end = read_le_u16(&self.ram, DUNG_INDEX_OF_TORCHES_START) as usize;
        let down = loop {
            if i == end {
                break u16::from(
                    self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH] == 0
                        && self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] == 0
                        && {
                            let mut tmp = 0;
                            !self.RoomTag_CheckForPressedSwitch(&mut tmp)
                        },
                );
            }
            if read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_STATE + (i >> 1) * 2) == 5 {
                let value = read_le_u16(&self.ram, BLOCK_TRAP_CHECK_FLAG);
                if value != 0xffff {
                    break value;
                }
                break u16::from(
                    self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH] == 0
                        && self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] == 0
                        && {
                            let mut tmp = 0;
                            !self.RoomTag_CheckForPressedSwitch(&mut tmp)
                        },
                );
            }
            i += 2;
        };

        if down != read_le_u16(&self.ram, DUNG_FLAG_TRAPDOORS_DOWN) {
            write_le_u16(&mut self.ram, DUNG_FLAG_TRAPDOORS_DOWN, down);
            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, 0);
            write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
            if down == 0 {
                self.ram[SOUND_EFFECT_2] = 0x25;
            }
            self.frame_control_view_mut().set_submodule(5);
        }
    }

    pub(super) fn RoomTag_SwitchTrigger_ToggleDoor(&mut self, _k: usize) {
        let mut attr = 0;
        if self.ram[DUNG_DOOR_SWITCH_TRIGGERED] == 0 {
            if self.RoomTag_MaybeCheckShutters(&mut attr) {
                write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, 0);
                write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
                self.ram[SOUND_EFFECT_2] = 0x25;
                self.PushPressurePlate(attr);
                let down = read_le_u16(&self.ram, DUNG_FLAG_TRAPDOORS_DOWN) ^ 1;
                write_le_u16(&mut self.ram, DUNG_FLAG_TRAPDOORS_DOWN, down);
                self.ram[DUNG_DOOR_SWITCH_TRIGGERED] = 1;
            }
        } else if !self.RoomTag_MaybeCheckShutters(&mut attr) {
            self.ram[DUNG_DOOR_SWITCH_TRIGGERED] = 0;
        }
    }

    pub(super) fn PushPressurePlate(&mut self, attr: u8) {
        self.frame_control_view_mut().set_submodule(5);
        if attr == 0x23 || read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_DST_POS_X2) == 0 {
            return;
        }
        self.ram[SAVED_MODULE_FOR_MENU] = self.frame_control_view().submodule();
        self.frame_control_view_mut().set_submodule(23);
        self.frame_control_view_mut().set_subsubmodule(32);
        let link_y = self.player_state_view().y().wrapping_add(2);
        self.player_state_view_mut().set_y(link_y);

        let mut pos = read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_DST_POS_X2);
        if read_le_u16(&self.ram, DUNG_BG2_ATTR_TABLE + pos as usize) & 0xfe00 != 0x2400 {
            pos = pos.wrapping_add(1);
            write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_DST_POS_X2, pos);
        }
        self.Dungeon_UpdateTileMapWithCommonTile(
            i32::from((pos & 0x003f) << 3),
            i32::from((pos >> 3) & 0x01f8),
            0x10,
        );
    }

    pub(super) fn RoomTag_KillRoomBlock(&mut self, k: usize) {
        if self.player_state_view().x() & 0x0100 != 0
            && self.player_state_view().y() & 0x0100 != 0
            && self.sprite_check_if_screen_is_clear()
        {
            self.ram[SOUND_EFFECT_2] = 0x1b;
            self.ram[DUNG_HDR_TAG + k] = 0;
        }
    }

    pub(super) fn RoomTag_PushBlockForChest(&mut self, k: usize) {
        if self.ram[NMI_LOAD_BG_FROM_VRAM] == 0 && self.ram[DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED] != 0
        {
            self.RoomTag_OperateChestReveal(k);
        }
    }

    pub(super) fn RoomTag_TriggerChest(&mut self, k: usize) {
        let mut attr = 0;
        if self.ram[COUNTDOWN_FOR_BLINK] == 0 && self.RoomTag_MaybeCheckShutters(&mut attr) {
            self.RoomTag_OperateChestReveal(k);
        }
    }

    pub(super) fn RoomTag_TorchPuzzleChest(&mut self, k: usize) {
        let mut lit = 0;
        for i in 0..16 {
            if read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + i * 2) & 0x8000 != 0 {
                lit += 1;
            }
        }
        if lit >= 4 {
            self.RoomTag_OperateChestReveal(k);
        }
    }

    pub(super) fn RoomTag_OperateChestReveal(&mut self, k: usize) {
        self.ram[DUNG_HDR_TAG + k] = 0;
        write_le_u16(&mut self.ram, VRAM_UPLOAD_OFFSET, 0);
        write_le_u16(&mut self.ram, OVERWORLD_MAP_STATE, 0);

        let mut attr = 0x5858;
        loop {
            let yy = read_le_u16(&self.ram, OVERWORLD_MAP_STATE);
            let pos = (read_le_u16(&self.ram, DUNG_CHEST_LOCATIONS + (yy >> 1) as usize * 2) >> 1)
                & 0x1fff;

            write_le_u16(&mut self.ram, DUNG_BG2_ATTR_TABLE + pos as usize, attr);
            write_le_u16(
                &mut self.ram,
                DUNG_BG2_ATTR_TABLE + (pos + 64) as usize,
                attr,
            );
            attr = attr.wrapping_add(0x0101);

            let src = 0x149c;
            let tiles = [
                self.tile_word(src, 0),
                self.tile_word(src, 1),
                self.tile_word(src, 2),
                self.tile_word(src, 3),
            ];
            let positions = [pos, pos + 64, pos + 1, pos + 65];
            for (&tile_pos, &tile) in positions.iter().zip(tiles.iter()) {
                write_le_u16(&mut self.ram, DUNG_BG2 + tile_pos as usize * 2, tile);
            }

            let upload = read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET) as usize;
            let dst = VRAM_UPLOAD_DATA + upload;
            for (i, (&offset, &tile)) in [0u16, 128, 2, 130].iter().zip(tiles.iter()).enumerate() {
                let stripe = self.RoomTag_BuildChestStripes(offset, yy);
                let base = dst + i * 6;
                write_le_u16(&mut self.ram, base, stripe);
                write_le_u16(&mut self.ram, base + 2, 0x0100);
                write_le_u16(&mut self.ram, base + 4, tile);
            }
            write_le_u16(&mut self.ram, dst + 24, 0xffff);
            write_le_u16(&mut self.ram, VRAM_UPLOAD_OFFSET, (upload + 24) as u16);

            let next = yy.wrapping_add(2);
            write_le_u16(&mut self.ram, OVERWORLD_MAP_STATE, next);
            if next == read_le_u16(&self.ram, DUNG_NUM_CHESTS_X2) {
                break;
            }
        }

        write_le_u16(&mut self.ram, OVERWORLD_MAP_STATE, 0);
        self.ram[SOUND_EFFECT_2] = 26;
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn RoomTag_BuildChestStripes(&self, pos: u16, y: u16) -> u16 {
        let loc = read_le_u16(&self.ram, DUNG_CHEST_LOCATIONS + (y >> 1) as usize * 2);
        let pos = pos.wrapping_add(loc);
        (((pos & 0x0040) << 4) | ((pos & 0x303f) >> 1) | ((pos & 0x0f80) >> 2)).swap_bytes()
    }

    pub(super) fn RoomTag_GetTilemapCoords(&self) -> i32 {
        let pos = ((self.player_state_view().x().wrapping_sub(1) & 0x01f8) >> 3)
            | ((self.player_state_view().y().wrapping_add(14) & 0x01f8) << 3)
            | if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
                0x1000
            } else {
                0
            };
        i32::from(pos)
    }

    pub(super) fn RoomTag_MaybeCheckShutters(&mut self, attr_out: &mut u8) -> bool {
        write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_DST_POS_X2, 0);
        if self.ram[FLAG_IS_LINK_IMMOBILIZED] != 0 || self.ram[LINK_AUXILIARY_STATE] != 0 {
            return false;
        }

        let p = self.RoomTag_GetTilemapCoords() as u16;
        let checks = [p, p.wrapping_add(64), p.wrapping_add(1), p.wrapping_add(65)];
        for &q in &checks {
            let t = read_le_u16(&self.ram, DUNG_BG2_ATTR_TABLE + q as usize);
            if t == 0x2323 || t == 0x2424 {
                if t != read_le_u16(&self.ram, DUNG_BG2_ATTR_TABLE + (q + 64) as usize) {
                    return false;
                }
                *attr_out = t as u8;
                write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_DST_POS_X2, q);
                return true;
            }
        }
        false
    }

    pub(super) fn RoomTag_CheckForPressedSwitch(&mut self, y_out: &mut u8) -> bool {
        write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_DST_POS_X2, 0);
        if self.ram[FLAG_IS_LINK_IMMOBILIZED] != 0 || self.ram[LINK_AUXILIARY_STATE] != 0 {
            return false;
        }

        let p = self.RoomTag_GetTilemapCoords() as u16;
        let checks = [p, p.wrapping_add(64), p.wrapping_add(1), p.wrapping_add(65)];
        for &q in &checks {
            let t = read_le_u16(&self.ram, DUNG_BG2_ATTR_TABLE + q as usize);
            if t == 0x2323 || t == 0x3a3a || t == 0x3b3b {
                if t != read_le_u16(&self.ram, DUNG_BG2_ATTR_TABLE + (q + 64) as usize) {
                    return false;
                }
                *y_out = u8::from(t == 0x3b3b);
                write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_DST_POS_X2, q);
                return true;
            }
        }
        false
    }

    pub(super) fn Dungeon_SetAttrForActivatedWaterOff(&mut self) {
        self.ram[CGWSEL_COPY] = 2;
        self.ram[CGADSUB_COPY] = 0x32;
        self.ram[TS_COPY] = 0;
        self.ram[W12SEL_COPY] = 0;
        self.ram[DUNG_HDR_COLLISION] = 0;
        write_le_u16(&mut self.ram, TMW_COPY, 0);

        let mut j = 0;
        while j != read_le_u16(&self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER) {
            let dsto = read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1 + (j >> 1) as usize * 2);
            self.write_attr2(dsto as usize + xy(1, 1), 0x1d1d);
            self.write_attr2(dsto as usize + xy(1, 2), 0x1d1d);
            j += 2;
        }

        let mut j = 0;
        while j != read_le_u16(&self.ram, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER) {
            let dsto = read_le_u16(&self.ram, DUNG_STAIRS_TABLE_2 + (j >> 1) as usize * 2);
            self.write_attr2(dsto as usize + xy(1, 1), 0x1d1d);
            self.write_attr2(dsto as usize + xy(1, 2), 0x1d1d);
            j += 2;
        }

        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Dungeon_SetAttrForActivatedWater(&mut self) {
        write_le_u16(&mut self.ram, TMW_COPY, 0);

        let mut j = 0;
        while j != read_le_u16(&self.ram, DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS) {
            let dsto = read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1 + (j >> 1) as usize * 2) as usize;
            self.write_attr2(dsto, 0x0003);
            self.write_attr2(dsto + 2, 0x0300);
            self.write_attr1(dsto, 0x0a03);
            self.write_attr1(dsto + 2, 0x030a);
            self.write_attr2(dsto + xy(0, 1), 0x0808);
            self.write_attr2(dsto + xy(2, 1), 0x0808);
            self.write_attr1(dsto + xy(0, 1), 0x0808);
            self.write_attr1(dsto + xy(2, 1), 0x0808);
            self.write_attr1(dsto + xy(0, 2), 0x0808);
            self.write_attr1(dsto + xy(2, 2), 0x0808);
            self.write_attr1(dsto + xy(0, 3), 0x0808);
            self.write_attr1(dsto + xy(2, 3), 0x0808);
            j += 2;
        }

        let mut j = 0;
        while j != read_le_u16(&self.ram, DUNG_NUM_STAIRS_WET) {
            let dsto = read_le_u16(&self.ram, DUNG_STAIRS_TABLE_2 + (j >> 1) as usize * 2) as usize;
            self.write_attr2(dsto + xy(0, 3), 0x0003);
            self.write_attr2(dsto + xy(2, 3), 0x0300);
            self.write_attr1(dsto + xy(0, 3), 0x0a03);
            self.write_attr1(dsto + xy(2, 3), 0x030a);
            self.write_attr2(dsto + xy(0, 2), 0x0808);
            self.write_attr2(dsto + xy(2, 2), 0x0808);
            self.write_attr1(dsto, 0x0808);
            self.write_attr1(dsto + 2, 0x0808);
            self.write_attr1(dsto + xy(0, 1), 0x0808);
            self.write_attr1(dsto + xy(2, 1), 0x0808);
            self.write_attr1(dsto + xy(0, 2), 0x0808);
            self.write_attr1(dsto + xy(2, 2), 0x0808);
            j += 2;
        }

        self.frame_control_view_mut().set_submodule(0);
        self.ram[NMI_BOOLEAN] = 0;
        self.frame_control_view_mut().set_subsubmodule(0);
    }

    pub(super) fn Sprite_HandlePushedBlocks_One(&mut self, i: usize) {
        self.oam_allocate_from_region_b(4);

        let y = (self.ram[PUSHEDBLOCKS_Y_LO + i * 2] as u16
            | ((self.ram[PUSHEDBLOCKS_Y_HI + i * 2] as u16) << 8))
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
            .wrapping_sub(1);
        let x = (self.ram[PUSHEDBLOCKS_X_LO + i * 2] as u16
            | ((self.ram[PUSHEDBLOCKS_X_HI + i * 2] as u16) << 8))
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));

        if self.ram[PUSHED_BLOCK_MODE] < 3 {
            let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
            self.ram[oam] = x as u8;
            self.ram[oam + 1] = y as u8;
            self.ram[oam + 2] = 12;
            self.ram[oam + 3] = 0x20;
            let ext = read_le_u16(&self.ram, OAM_EXT_CUR_PTR) as usize;
            self.ram[ext] = 2;
        }
    }

    pub(super) fn Object_Draw_DoorLeft_3x4(&mut self, src: u16, door: usize) {
        let dsto = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2) >> 1;
        for i in 0..3 {
            for y in 0..4 {
                self.room_write_bg(
                    0x2000,
                    dsto + i as u16 + y * 64,
                    self.tile_word(src as usize, i * 4 + y as usize),
                );
            }
        }
    }

    pub(super) fn Object_Draw_DoorRight_3x4(&mut self, src: u16, door: usize) {
        let dsto = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2) >> 1;
        for i in 0..3 {
            for y in 0..4 {
                self.room_write_bg(
                    0x2000,
                    dsto + 1 + i as u16 + y * 64,
                    self.tile_word(src as usize, i * 4 + y as usize),
                );
            }
        }
    }

    pub(super) fn GetDoorDrawDataIndex_North_clean_door_index(&mut self, door: usize) {
        self.GetDoorDrawDataIndex_North(door, door);
    }

    pub(super) fn DoorDoorStep1_North(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x1fff) >= DOOR_POSITION_UP[6] as i32 {
            pos -= 0x500;
            if (self.ram[DOOR_TYPE_AND_SLOT + door * 2] & 0xfe) >= 0x42 {
                pos -= 0x300;
            }
            self.GetDoorDrawDataIndex_South(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.Dungeon_LoadSingleDoorAttribute(door ^ 8);
        }
        self.GetDoorDrawDataIndex_North(door, door & 7);
        dma_ptr
    }

    pub(super) fn GetDoorDrawDataIndex_North(&mut self, door: usize, r4_door: usize) {
        let door_type = self.ram[DOOR_TYPE_AND_SLOT + door * 2] & 0xfe;
        let mut x = self.ram[DOOR_OPEN_CLOSED_COUNTER] as usize;
        if x == 0 || x == 4 {
            self.DrawDoorToTileMap_North(door, r4_door);
            return;
        }
        if door_type == DOOR_TYPE_STAIR_MASK_LOCKED2
            || door_type == DOOR_TYPE_STAIR_MASK_LOCKED3
            || door_type >= 0x42
        {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = K_DOOR_ANIM_UP_SRC[x >> 1];
        self.Object_Draw_DoorUp_4x3(src, door);
    }

    pub(super) fn DrawDoorToTileMap_North(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_UP[index];
        self.Object_Draw_DoorUp_4x3(src, door);
    }

    pub(super) fn Object_Draw_DoorUp_4x3(&mut self, src: u16, door: usize) {
        let dsto = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2) >> 1;
        for i in 0..4 {
            for y in 0..3 {
                self.room_write_bg(
                    0x2000,
                    dsto + i as u16 + y * 64,
                    self.tile_word(src as usize, i * 3 + y as usize),
                );
            }
        }
    }

    pub(super) fn GetDoorDrawDataIndex_South_clean_door_index(&mut self, door: usize) {
        self.GetDoorDrawDataIndex_South(door, door);
    }

    pub(super) fn DoorDoorStep1_South(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x1fff) < DOOR_POSITION_DOWN[9] as i32 {
            pos += 0x500;
            if (self.ram[DOOR_TYPE_AND_SLOT + door * 2] & 0xfe) >= 0x42 {
                pos += 0x300;
            }
            self.GetDoorDrawDataIndex_North(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.Dungeon_LoadSingleDoorAttribute(door ^ 8);
        }
        self.GetDoorDrawDataIndex_South(door, door & 7);
        dma_ptr
    }

    pub(super) fn GetDoorDrawDataIndex_South(&mut self, door: usize, r4_door: usize) {
        let door_type = self.ram[DOOR_TYPE_AND_SLOT + door * 2] & 0xfe;
        let mut x = self.ram[DOOR_OPEN_CLOSED_COUNTER] as usize;
        if x == 0 || x == 4 {
            self.DrawDoorToTileMap_South(door, r4_door);
            return;
        }
        if door_type >= 0x42 {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = K_DOOR_ANIM_DOWN_SRC[x >> 1];
        self.Object_Draw_DoorDown_4x3(src, door);
    }

    pub(super) fn DrawDoorToTileMap_South(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_DOWN[index];
        self.Object_Draw_DoorDown_4x3(src, door);
    }

    pub(super) fn Object_Draw_DoorDown_4x3(&mut self, src: u16, door: usize) {
        let dsto = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2) >> 1;
        for i in 0..4 {
            for y in 0..3 {
                self.room_write_bg(
                    0x2000,
                    dsto + i as u16 + (y + 1) * 64,
                    self.tile_word(src as usize, i * 3 + y as usize),
                );
            }
        }
    }

    pub(super) fn GetDoorDrawDataIndex_West_clean_door_index(&mut self, door: usize) {
        self.GetDoorDrawDataIndex_West(door, door);
    }

    pub(super) fn DoorDoorStep1_West(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x7ff) >= DOOR_POSITION_LEFT[6] as i32 {
            pos -= 16;
            if (self.ram[DOOR_TYPE_AND_SLOT + door * 2] & 0xfe) >= 0x42 {
                pos -= 12;
            }
            self.GetDoorDrawDataIndex_East(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.Dungeon_LoadSingleDoorAttribute(door ^ 8);
        }
        self.GetDoorDrawDataIndex_West(door, door & 7);
        dma_ptr
    }

    pub(super) fn GetDoorDrawDataIndex_West(&mut self, door: usize, r4_door: usize) {
        let door_type = self.ram[DOOR_TYPE_AND_SLOT + door * 2] & 0xfe;
        let mut x = self.ram[DOOR_OPEN_CLOSED_COUNTER] as usize;
        if x == 0 || x == 4 {
            self.DrawDoorToTileMap_West(door, r4_door);
            return;
        }
        if door_type >= 0x42 {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = K_DOOR_ANIM_LEFT_SRC[x >> 1];
        self.Object_Draw_DoorLeft_3x4(src, door);
    }

    pub(super) fn DrawDoorToTileMap_West(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_LEFT[index];
        self.Object_Draw_DoorLeft_3x4(src, door);
    }

    pub(super) fn GetDoorDrawDataIndex_East_clean_door_index(&mut self, door: usize) {
        self.GetDoorDrawDataIndex_East(door, door);
    }

    pub(super) fn DoorDoorStep1_East(&mut self, door: usize, dma_ptr: usize) -> usize {
        let mut pos = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2) as i32;
        let mut dma_ptr = dma_ptr;
        if (pos & 0x7ff) < DOOR_POSITION_RIGHT[6] as i32 {
            pos += 16;
            if (self.ram[DOOR_TYPE_AND_SLOT + door * 2] & 0xfe) >= 0x42 {
                pos += 12;
            }
            self.GetDoorDrawDataIndex_West(door ^ 8, door & 7);
            dma_ptr = self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos as u16);
            self.Dungeon_LoadSingleDoorAttribute(door ^ 8);
        }
        self.GetDoorDrawDataIndex_East(door, door & 7);
        dma_ptr
    }

    pub(super) fn GetDoorDrawDataIndex_East(&mut self, door: usize, r4_door: usize) {
        let door_type = self.ram[DOOR_TYPE_AND_SLOT + door * 2] & 0xfe;
        let mut x = self.ram[DOOR_OPEN_CLOSED_COUNTER] as usize;
        if x == 0 || x == 4 {
            self.DrawDoorToTileMap_East(door, r4_door);
            return;
        }
        if door_type >= 0x42 {
            x += 4;
        }
        if door_type == DOOR_TYPE_SHUTTERS_TWO_WAY || door_type == DOOR_TYPE_SHUTTER {
            x += 2;
        }
        let src = K_DOOR_ANIM_RIGHT_SRC[x >> 1];
        self.Object_Draw_DoorRight_3x4(src, door);
    }

    pub(super) fn DrawDoorToTileMap_East(&mut self, door: usize, r4_door: usize) {
        let index = self.GetDoorGraphicsIndex(door, r4_door) as usize >> 1;
        let src = DOOR_TYPE_SRC_RIGHT[index];
        self.Object_Draw_DoorRight_3x4(src, door);
    }

    pub(super) fn GetDoorGraphicsIndex(&self, door: usize, r4_door: usize) -> u8 {
        let mut door_type = self.ram[DOOR_TYPE_AND_SLOT + door * 2] & 0xfe;
        if read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) & upper_bitmask(r4_door) != 0 {
            door_type = DOOR_TYPE_REMAP[door_type as usize >> 1];
        }
        door_type
    }

    pub(super) fn ClearExplodingWallFromTileMap_ClearOnePair(
        &mut self,
        mut dsto: u16,
        mut src: usize,
    ) {
        for _ in 0..2 {
            for j in 0..12u16 {
                self.room_write_bg(0x2000, dsto + j * 64, self.tile_word(src, j as usize));
            }
            dsto = dsto.wrapping_add(1);
            src += 24;
        }
    }

    pub(super) fn Door_BlastWallExploding_Draw(&mut self, dsto: usize) {
        let src = 0x31eausize;
        self.ClearExplodingWallFromTileMap_ClearOnePair(dsto as u16, src);
        let mut dst = dsto as u16 + 2;
        let fill = self.tile_word(src, 24);
        let count = read_le_u16(&self.ram, CRUSH_WALL_PROGRESS_DUNGEON).wrapping_sub(1);
        for _ in 0..count {
            for j in 0..12u16 {
                self.room_write_bg(0x2000, dst + j * 64, fill);
            }
            dst = dst.wrapping_add(1);
        }
        self.ClearExplodingWallFromTileMap_ClearOnePair(dst, src + 50);
    }

    pub(super) fn ClearAndStripeExplodingWall(&mut self, mut dsto: u16) {
        const BLAST_WALL_TAB2: [u16; 16] = [
            0x0004, 0x0008, 0x000c, 0x0010, 0x0014, 0x0018, 0x001c, 0x0020, 0x0100, 0x0200, 0x0300,
            0x0400, 0x0500, 0x0600, 0x0700, 0x0800,
        ];

        let mut r6 = 0x80u16;
        let mut r14 = 0u16;
        let mut r10 = read_le_u16(&self.ram, CRUSH_WALL_PROGRESS_DUNGEON).wrapping_add(3);
        let mut r2 = 0u16;
        if r10 >= 8 {
            r2 = r10.wrapping_sub(6);
            r14 = 1;
            r10 = 3;
        }
        let door = read_le_u16(&self.ram, DUNG_CUR_DOOR_IDX) as usize >> 1;
        if self.ram[DUNG_DOOR_DIRECTION + door * 2] & 2 == 0 {
            r6 = r6.wrapping_add(1);
        }

        let mut upload = UVRAM_DATA_DUNGEON;
        loop {
            let mut cols = r10;
            loop {
                let vram_addr = self.Dungeon_MapVramAddrNoSwap(dsto);
                write_le_u16(&mut self.ram, upload, vram_addr);
                write_le_u16(&mut self.ram, upload + 2, r6 | 0x0a00);
                for y in 0..5u16 {
                    let tile = self.room_read_bg(0x2000, dsto + y * 64);
                    write_le_u16(&mut self.ram, upload + 4 + y as usize * 2, tile);
                }
                write_le_u16(&mut self.ram, upload + 14, vram_addr.wrapping_add(0x04a0));
                write_le_u16(&mut self.ram, upload + 16, r6 | 0x0e00);
                for y in 0..7u16 {
                    let tile = self.room_read_bg(0x2000, dsto + (y + 5) * 64);
                    write_le_u16(&mut self.ram, upload + 18 + y as usize * 2, tile);
                }
                dsto = dsto.wrapping_add(1);
                upload += 32;
                cols = cols.wrapping_sub(1);
                if cols == 0 {
                    break;
                }
            }
            if r14 == 0 {
                break;
            }
            r14 = r14.wrapping_sub(1);
            let tab_index = ((r2 >> 1) + if r6 & 1 != 0 { 0 } else { 8 } - 1) as usize;
            dsto = dsto.wrapping_add(BLAST_WALL_TAB2[tab_index] >> 1);
            r10 = 3;
        }
        write_le_u16(&mut self.ram, upload, 0xffff);
    }

    pub(super) fn Dungeon_DrawRoomOverlay(&mut self, src: &[u8]) {
        let mut offset = 0usize;
        loop {
            write_le_u16(&mut self.ram, DUNG_DRAW_WIDTH_INDICATOR, 0);
            write_le_u16(&mut self.ram, DUNG_DRAW_HEIGHT_INDICATOR, 0);
            let marker = src[offset] as u16 | ((src[offset + 1] as u16) << 8);
            if marker == 0xffff {
                break;
            }
            let p = ((src[offset] as u16 >> 2) | ((src[offset + 1] as u16 >> 2) << 6)) as u16;
            let kind = src[offset + 2];
            if kind == 0xa4 {
                let mid = self.tile_word(0x05aa, 0);
                let top = self.tile_word(0x063c, 1);
                let bottom = self.tile_word(0x0642, 1);
                for x in 0..4u16 {
                    write_le_u16(&mut self.ram, DUNG_BG2 + (p + x) as usize * 2, top);
                    write_le_u16(&mut self.ram, DUNG_BG2 + (p + x + 64) as usize * 2, mid);
                    write_le_u16(&mut self.ram, DUNG_BG2 + (p + x + 128) as usize * 2, mid);
                    write_le_u16(&mut self.ram, DUNG_BG2 + (p + x + 192) as usize * 2, bottom);
                }
            } else {
                let floor = read_le_u16(&self.ram, DUNG_FLOOR_2_FILLER_TILES) as usize;
                for y in 0..4u16 {
                    for x in 0..4u16 {
                        let idx = match (x & 1, y & 1) {
                            (0, 0) => 0,
                            (1, 0) => 1,
                            (0, 1) => 4,
                            _ => 5,
                        };
                        let tile = self.tile_word(floor, idx);
                        write_le_u16(
                            &mut self.ram,
                            DUNG_BG2 + (p + x + y * 64) as usize * 2,
                            tile,
                        );
                    }
                }
            }
            offset += 3;
        }
    }

    pub(super) fn Dungeon_DrawRoomOverlay_Apply(&mut self, mut p: usize) {
        for _ in 0..4 {
            for i in 0..4 {
                let t = read_le_u16(&self.ram, DUNG_BG2 + (p + i) * 2) & 0x03fe;
                let attr = if t == 0x00ee || t == 0x00fe { 0 } else { 0x20 };
                self.ram[DUNG_BG2_ATTR_TABLE + p + i] = attr;
                if std::env::var_os("ZELDA3_TRACE_OVERLAY_ATTR").is_some() {
                    let pos = p + i;
                    let trace_pos = std::env::var("ZELDA3_TRACE_OVERLAY_ATTR_POS")
                        .ok()
                        .and_then(|value| {
                            value
                                .strip_prefix("0x")
                                .or_else(|| value.strip_prefix("0X"))
                                .and_then(|hex| usize::from_str_radix(hex, 16).ok())
                                .or_else(|| value.parse::<usize>().ok())
                        });
                    if trace_pos.map_or(true, |target| target == pos) {
                        eprintln!(
                            "R overlay_attr fc={} room=0x{:04x} overlay=0x{:02x} p=0x{:04x} pos=0x{:04x} tile=0x{:04x} attr=0x{:02x}",
                            self.ram[FRAME_COUNTER],
                            self.world_state_view().dungeon_room(),
                            self.ram[DUNG_OVERLAY_TO_LOAD],
                            p,
                            pos,
                            t,
                            attr
                        );
                    }
                }
            }
            p += 64;
        }
    }

    pub(super) fn DrawDoorOpening_Step1(&mut self, door: usize, dma_ptr: usize) -> usize {
        write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, (door * 2) as u16);
        write_le_u16(&mut self.ram, DUNG_WHICH_KEY_X2_DUNGEON, (door * 2) as u16);
        match self.ram[DUNG_DOOR_DIRECTION + door * 2] & 3 {
            0 => self.DoorDoorStep1_North(door, dma_ptr),
            1 => self.DoorDoorStep1_South(door, dma_ptr),
            2 => self.DoorDoorStep1_West(door, dma_ptr),
            3 => self.DoorDoorStep1_East(door, dma_ptr),
            _ => 0,
        }
    }

    pub(super) fn DrawShutterDoorSteps(&mut self, door: usize) {
        write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, (door * 2) as u16);
        write_le_u16(&mut self.ram, DUNG_WHICH_KEY_X2_DUNGEON, (door * 2) as u16);
        match self.ram[DUNG_DOOR_DIRECTION + door * 2] & 3 {
            0 => self.GetDoorDrawDataIndex_North_clean_door_index(door),
            1 => self.GetDoorDrawDataIndex_South_clean_door_index(door),
            2 => self.GetDoorDrawDataIndex_West_clean_door_index(door),
            3 => self.GetDoorDrawDataIndex_East_clean_door_index(door),
            _ => {}
        }
    }

    pub(super) fn DrawEyeWatchDoor(&mut self, door: usize) {
        write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, (door * 2) as u16);
        write_le_u16(&mut self.ram, DUNG_WHICH_KEY_X2_DUNGEON, (door * 2) as u16);
        match self.ram[DUNG_DOOR_DIRECTION + door * 2] & 3 {
            0 => self.DrawDoorToTileMap_North(door, door),
            1 => self.DrawDoorToTileMap_South(door, door),
            2 => self.DrawDoorToTileMap_West(door, door),
            3 => self.DrawDoorToTileMap_East(door, door),
            _ => {}
        }
    }

    pub(super) fn OperateShutterDoors(&mut self) {
        let mut anim_dst = 0usize;
        let mut y = 2u8;

        let step = read_le_u16(&self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON).wrapping_add(1);
        write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, step);
        if step != 4 {
            y = if self.ram[DUNG_FLAG_TRAPDOORS_DOWN] != 0 {
                0
            } else {
                4
            };
            if step != 8 {
                if self.ram[DOOR_ANIMATION_STEP_INDICATOR_DUNGEON] != 0x10 {
                    return;
                }
                self.frame_control_view_mut().set_submodule(0);
                self.ram[NMI_COPY_PACKETS_FLAG] = 0;
                return;
            }
        }
        write_le_u16(&mut self.ram, DOOR_OPEN_CLOSED_COUNTER, y as u16);

        let mut cur = 0usize;
        while cur != 0x18 {
            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, cur as u16);
            let j = cur >> 1;
            let door_type = self.ram[DOOR_TYPE_AND_SLOT + j * 2] & 0xfe;
            if door_type == DOOR_TYPE_SHUTTER || door_type == DOOR_TYPE_SHUTTERS_TWO_WAY {
                let mask = upper_bitmask(j);
                let mut should_draw = true;
                let mut opened = read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT);
                if self.ram[DUNG_FLAG_TRAPDOORS_DOWN] == 0 {
                    if opened & mask != 0 {
                        should_draw = false;
                    } else if step == 8 {
                        self.ram[SOUND_EFFECT_2] = 21;
                        opened ^= mask;
                        write_le_u16(&mut self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, opened);
                    }
                } else if opened & mask == 0 {
                    should_draw = false;
                } else if step == 8 {
                    self.ram[SOUND_EFFECT_2] = 22;
                    opened ^= mask;
                    write_le_u16(&mut self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, opened);
                }

                if should_draw {
                    self.DrawShutterDoorSteps(j);
                    let addr = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + j * 2);
                    anim_dst = self.dungeon_prep_overlay_dma_next_prep(anim_dst, addr);
                    if step == 8 {
                        self.Dungeon_LoadToggleDoorAttr_OtherEntry(j as i32);
                    }
                }
            }
            cur += 2;
        }
        write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, 0x16);

        if anim_dst != 0 {
            self.ram[NMI_DISABLE_CORE_UPDATES] = 1;
            self.ram[NMI_COPY_PACKETS_FLAG] = 1;
            if self.ram[DOOR_ANIMATION_STEP_INDICATOR_DUNGEON] != 0x10 {
                return;
            }
        }
        self.frame_control_view_mut().set_submodule(0);
        self.ram[NMI_COPY_PACKETS_FLAG] = 0;
    }

    pub(super) fn OpenCrackedDoor(&mut self) {
        self.Dungeon_OpeningLockedDoor_Combined(true);
    }

    pub(super) fn Dungeon_OpeningLockedDoor_Combined(&mut self, skip_anim: bool) {
        let mut ctr = 2u8;
        let step;
        if skip_anim {
            write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 16);
            step = 16;
        } else {
            step = read_le_u16(&self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON).wrapping_add(1);
            write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, step);
            if step != 4 && step != 12 {
                if step == 16 {
                    self.finish_locked_door_opening();
                }
                return;
            }
        }

        if step == 12 || skip_anim {
            let cur = read_le_u16(&self.ram, DUNG_CUR_DOOR_POS_DUNGEON) as usize;
            let mask = upper_bitmask((self.ram[DUNG_BG2_ATTR_TABLE + cur] & 7) as usize);
            let opened_adj = read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) | mask;
            let opened = read_le_u16(&self.ram, DUNG_DOOR_OPENED) | mask;
            write_le_u16(&mut self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, opened_adj);
            write_le_u16(&mut self.ram, DUNG_DOOR_OPENED, opened);
            ctr = 4;
        }

        self.ram[DOOR_OPEN_CLOSED_COUNTER] = ctr;
        let cur = read_le_u16(&self.ram, DUNG_CUR_DOOR_POS_DUNGEON) as usize;
        let k = (self.ram[DUNG_BG2_ATTR_TABLE + cur] & 0x0f) as usize;
        let dma_ptr = self.DrawDoorOpening_Step1(k, 0);
        let addr = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + k * 2);
        self.dungeon_prep_overlay_dma_next_prep(dma_ptr, addr);
        self.ram[SOUND_EFFECT_2] = 21;
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;

        if step == 16 {
            self.finish_locked_door_opening();
        }
    }

    fn finish_locked_door_opening(&mut self) {
        let cur = read_le_u16(&self.ram, DUNG_CUR_DOOR_POS_DUNGEON) as usize;
        let k = (self.ram[DUNG_BG2_ATTR_TABLE + cur] & 0x0f) as usize;
        self.Dungeon_LoadToggleDoorAttr_OtherEntry(k as i32);
        if self.ram[DUNG_BG2_ATTR_TABLE + cur] >= 0xf0 {
            let door_type = self.ram[DOOR_TYPE_AND_SLOT + k * 2];
            if (DOOR_TYPE_STAIR_MASK_LOCKED0..=DOOR_TYPE_STAIR_MASK_LOCKED3).contains(&door_type) {
                self.DrawCompletelyOpenDoor();
            }
        }
        self.frame_control_view_mut().set_submodule(0);
    }

    pub(super) fn DrawCompletelyOpenDoor(&mut self) {
        let mut i = 0usize;
        let mut attr = 0x3030u16;
        while i != read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS) as usize {
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS) as usize {
            let pos = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(pos + xy(1, 0), 0x5e5e);
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), 0);
            self.write_attr2(pos + xy(1, 3), 0);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2) as usize {
            let pos = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(pos + xy(1, 0), 0x5f5f);
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), 0);
            self.write_attr2(pos + xy(1, 3), 0);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS) as usize {
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS) as usize {
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }

        attr = (attr & 0x0707) | 0x3434;

        while i != read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS) as usize {
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS) as usize {
            let pos = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(pos + xy(1, 0), 0x5e5e);
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), 0);
            self.write_attr2(pos + xy(1, 3), 0);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2) as usize {
            let pos = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(pos + xy(1, 0), 0x5f5f);
            self.write_attr2(pos + xy(1, 1), attr);
            self.write_attr2(pos + xy(1, 2), 0);
            self.write_attr2(pos + xy(1, 3), 0);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
    }

    pub(super) fn Dungeon_LoadAttributeTable(&mut self) {
        write_le_u16(&mut self.ram, DUNG_DRAW_WIDTH_INDICATOR, 0);
        write_le_u16(&mut self.ram, DUNG_DRAW_HEIGHT_INDICATOR, 0);
        self.Dungeon_LoadBasicAttribute_full(0x1000);
        self.Dungeon_LoadObjectAttribute();
        self.Dungeon_LoadDoorAttribute();
        if self.ram[ORANGE_BLUE_BARRIER_STATE] != 0 {
            self.Dungeon_FlipCrystalPegAttribute();
        }
        self.ram[OVERWORLD_MAP_STATE] = 0;
    }

    pub(super) fn Dungeon_LoadAttribute_Selectable(&mut self) {
        match self.ram[OVERWORLD_MAP_STATE] {
            0 => {
                self.ram[OVERWORLD_MAP_STATE] = 1;
                write_le_u16(&mut self.ram, DUNG_DRAW_WIDTH_INDICATOR, 0);
                write_le_u16(&mut self.ram, DUNG_DRAW_HEIGHT_INDICATOR, 0);
                self.Dungeon_LoadBasicAttribute_full(0x40);
            }
            1 => self.Dungeon_LoadBasicAttribute_full(0x40),
            2 => self.Dungeon_LoadObjectAttribute(),
            3 => self.Dungeon_LoadDoorAttribute(),
            4 => {
                self.ram[OVERWORLD_MAP_STATE] = 5;
                if self.ram[ORANGE_BLUE_BARRIER_STATE] != 0 {
                    self.Dungeon_FlipCrystalPegAttribute();
                }
            }
            5 => {}
            // C Dungeon_LoadAttribute_Selectable asserts outside states 0..=5.
            _ => panic!(
                "Dungeon_LoadAttribute_Selectable overworld_map_state {}",
                self.ram[OVERWORLD_MAP_STATE]
            ),
        }
    }

    fn Dungeon_LoadBasicAttribute_full(&mut self, loops: usize) {
        for _ in 0..loops {
            let i = read_le_u16(&self.ram, DUNG_DRAW_WIDTH_INDICATOR) as usize / 2;
            let tile0 = read_le_u16(&self.ram, DUNG_BG2 + i * 2);
            let tile1 = read_le_u16(&self.ram, DUNG_BG2 + (i + 1) * 2);
            let a0 = self.attribute_for_bg_tile(tile0);
            let a1 = self.attribute_for_bg_tile(tile1);
            let j = read_le_u16(&self.ram, DUNG_DRAW_HEIGHT_INDICATOR) as usize;
            self.ram[DUNG_BG2_ATTR_TABLE + j] = a0;
            self.ram[DUNG_BG2_ATTR_TABLE + j + 1] = a1;
            write_le_u16(
                &mut self.ram,
                DUNG_DRAW_HEIGHT_INDICATOR,
                (j as u16).wrapping_add(2),
            );
            let width = read_le_u16(&self.ram, DUNG_DRAW_WIDTH_INDICATOR).wrapping_add(4);
            write_le_u16(&mut self.ram, DUNG_DRAW_WIDTH_INDICATOR, width);
        }
        if read_le_u16(&self.ram, DUNG_DRAW_HEIGHT_INDICATOR) == 0x2000 {
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        }
    }

    fn attribute_for_bg_tile(&self, tile: u16) -> u8 {
        let mut attr = self.ram[ATTRIBUTES_FOR_TILE + (tile as usize & 0x03ff)];
        if (0x10..0x1c).contains(&attr) {
            attr |= (tile >> 14) as u8;
        }
        attr
    }

    fn Dungeon_LoadObjectAttribute(&mut self) {
        if std::env::var_os("ZELDA3_REPLAY_DUNGEON_ATTR_STATE_DUMP").is_some() {
            eprintln!(
                "dungeon-attr-state room=0x{:04x} star=0x{:04x} inter={:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x} in1={:04x},{:04x},{:04x},{:04x},{:04x},{:04x} misc=0x{:04x} torch=0x{:04x} chest=0x{:04x} big=0x{:04x} in2={:04x},{:04x},{:04x},{:04x} table1={:04x},{:04x},{:04x},{:04x} table2={:04x},{:04x},{:04x},{:04x} obj={:04x},{:04x},{:04x},{:04x}",
                self.world_state_view().dungeon_room(),
                read_le_u16(&self.ram, DUNG_NUM_STAR_SHAPED_SWITCHES),
                read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS),
                read_le_u16(&self.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS),
                read_le_u16(&self.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2),
                read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS),
                read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS),
                read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS),
                read_le_u16(&self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS),
                read_le_u16(&self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2),
                read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS),
                read_le_u16(&self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS),
                read_le_u16(&self.ram, DUNG_NUM_INROOM_SOUTHDOWN_STAIRS),
                read_le_u16(&self.ram, DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS),
                read_le_u16(&self.ram, WATER_SIDE_STEP_SWITCH),
                read_le_u16(&self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER),
                read_le_u16(&self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS),
                read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX),
                read_le_u16(&self.ram, DUNG_INDEX_OF_TORCHES),
                read_le_u16(&self.ram, DUNG_NUM_CHESTS_X2),
                read_le_u16(&self.ram, DUNG_NUM_BIGKEY_LOCKS_X2),
                read_le_u16(&self.ram, DUNG_NUM_STAIRS_1),
                read_le_u16(&self.ram, DUNG_NUM_STAIRS_2),
                read_le_u16(&self.ram, DUNG_NUM_STAIRS_WET),
                read_le_u16(&self.ram, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER),
                read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1),
                read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1 + 2),
                read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1 + 4),
                read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1 + 6),
                read_le_u16(&self.ram, DUNG_STAIRS_TABLE_2),
                read_le_u16(&self.ram, DUNG_STAIRS_TABLE_2 + 2),
                read_le_u16(&self.ram, DUNG_STAIRS_TABLE_2 + 4),
                read_le_u16(&self.ram, DUNG_STAIRS_TABLE_2 + 6),
                read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS),
                read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + 2),
                read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + 4),
                read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + 6),
            );
        }
        let mut i = 0usize;
        while i != read_le_u16(&self.ram, DUNG_NUM_STAR_SHAPED_SWITCHES) as usize {
            let j = read_le_u16(&self.ram, STAR_SHAPED_SWITCHES_TILE + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(0, 0), 0x3b3b);
            self.write_attr2(j + xy(0, 1), 0x3b3b);
            i += 2;
        }

        i = 0;
        let mut attr = 0x3030u16;
        while i != read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS) as usize {
            let j = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(1, 2), 0);
            self.write_attr2(j + xy(1, 0), 0x2626);
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS) as usize {
            let j = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(1, 0), 0x5e5e);
            self.write_attr2(j + xy(1, 2), 0x5e5e);
            self.write_attr2(j + xy(1, 3), 0x5e5e);
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2) as usize {
            let j = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(1, 0), 0x5f5f);
            self.write_attr2(j + xy(1, 2), 0x5f5f);
            self.write_attr2(j + xy(1, 3), 0x5f5f);
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS) as usize {
            let j = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(1, 0), 0x3838);
            self.write_attr2(j + xy(1, 2), 0);
            self.write_attr2(j + xy(1, 3), 0);
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS) as usize {
            let j = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(1, 0), 0);
            self.write_attr2(j + xy(1, 1), 0);
            self.write_attr2(j + xy(1, 2), attr);
            self.write_attr2(j + xy(1, 3), 0x3939);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        attr = (attr & 0x0707) | 0x3434;
        while i != read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS) as usize {
            let j = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(1, 2), attr);
            self.write_attr2(j + xy(1, 3), 0x2626);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS) as usize {
            let j = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(1, 0), 0x5e5e);
            self.write_attr2(j + xy(1, 1), attr);
            self.write_attr2(j + xy(1, 2), 0x5e5e);
            self.write_attr2(j + xy(1, 3), 0x5e5e);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2) as usize {
            let j = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(1, 0), 0x5f5f);
            self.write_attr2(j + xy(1, 1), attr);
            self.write_attr2(j + xy(1, 2), 0x5f5f);
            self.write_attr2(j + xy(1, 3), 0x5f5f);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS) as usize {
            let j = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(1, 0), 0x3838);
            self.write_attr2(j + xy(1, 1), attr);
            self.write_attr2(j + xy(1, 2), 0);
            self.write_attr2(j + xy(1, 3), 0);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }
        while i != read_le_u16(&self.ram, DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS) as usize {
            let j = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + (i >> 1) * 2) as usize;
            self.write_attr2(j + xy(1, 0), 0);
            self.write_attr2(j + xy(1, 1), 0);
            self.write_attr2(j + xy(1, 2), attr);
            self.write_attr2(j + xy(1, 3), 0x3939);
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }

        i = 0;
        let mut stair_type = 0u16;
        let mut iend = read_le_u16(&self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS) as usize;
        attr = 0x1f1f;
        if iend == 0 {
            stair_type = 1;
            attr = 0x1e1e;
            iend = read_le_u16(&self.ram, DUNG_NUM_INROOM_SOUTHDOWN_STAIRS) as usize;
            if iend == 0 {
                stair_type = 2;
                attr = 0x1d1d;
                iend = read_le_u16(&self.ram, DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS) as usize;
            }
        }
        if iend != 0 {
            write_le_u16(&mut self.ram, KIND_OF_IN_ROOM_STAIRCASE_DUNGEON, stair_type);
            while i != iend {
                let j = read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1 + (i >> 1) * 2) as usize;
                self.write_attr2(j + xy(0, 0), 0x0002);
                self.write_attr1(j + xy(0, 3), 0x0002);
                self.write_attr2(j + xy(2, 0), 0x0200);
                self.write_attr1(j + xy(2, 3), 0x0200);
                self.write_attr2(j + xy(0, 1), 0x0001);
                self.write_attr1(j + xy(0, 2), 0x0001);
                self.write_attr2(j + xy(2, 1), 0x0100);
                self.write_attr1(j + xy(2, 2), 0x0100);
                self.write_attr2(j + xy(1, 1), attr);
                self.write_attr1(j + xy(1, 1), attr);
                self.write_attr2(j + xy(1, 2), attr);
                self.write_attr1(j + xy(1, 2), attr);
                i += 2;
            }
        }
        if i != read_le_u16(&self.ram, WATER_SIDE_STEP_SWITCH) as usize {
            write_le_u16(&mut self.ram, KIND_OF_IN_ROOM_STAIRCASE_DUNGEON, 2);
            while i != read_le_u16(&self.ram, WATER_SIDE_STEP_SWITCH) as usize {
                let j = read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1 + (i >> 1) * 2) as usize;
                self.write_attr2(j + xy(0, 0), 0x0a03);
                self.write_attr1(j + xy(0, 0), 0x0a03);
                self.write_attr2(j + xy(2, 0), 0x030a);
                self.write_attr1(j + xy(2, 0), 0x030a);
                self.write_attr2(j + xy(0, 1), 0x0803);
                self.write_attr2(j + xy(2, 1), 0x0308);
                i += 2;
            }
        }
        i = 0;
        if i != read_le_u16(&self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER) as usize {
            write_le_u16(&mut self.ram, KIND_OF_IN_ROOM_STAIRCASE_DUNGEON, 2);
            while i != read_le_u16(&self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER) as usize {
                let j = read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1 + (i >> 1) * 2) as usize;
                self.write_attr2(j + xy(0, 0), 0x0003);
                self.write_attr2(j + xy(2, 0), 0x0300);
                self.write_attr1(j + xy(0, 0), 0x0a03);
                self.write_attr1(j + xy(2, 0), 0x030a);
                self.write_attr2(j + xy(0, 1), 0x0808);
                self.write_attr2(j + xy(2, 1), 0x0808);
                i += 2;
            }
        }
        if i != read_le_u16(&self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS) as usize {
            write_le_u16(&mut self.ram, KIND_OF_IN_ROOM_STAIRCASE_DUNGEON, 2);
            while i != read_le_u16(&self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS) as usize {
                let j = read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1 + (i >> 1) * 2) as usize;
                self.write_attr2(j + xy(0, 0), 0x0003);
                self.write_attr2(j + xy(2, 0), 0x0300);
                self.write_attr1(j + xy(0, 0), 0x0a03);
                self.write_attr1(j + xy(2, 0), 0x030a);
                i += 2;
            }
        }

        let mut i = 0usize;
        let mut attr = 0x7070u16;
        let misc_end = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX) as usize;
        while i != misc_end {
            let k = read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_STATE + (i >> 1) * 2);
            if (k & 0x00f0) != 0x0030 {
                let j =
                    (read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + (i >> 1) * 2) & 0x3fff) >> 1;
                self.write_attr2(j as usize + xy(0, 0), attr);
                self.write_attr2(j as usize + xy(0, 1), attr);
            }
            i += 2;
            attr = attr.wrapping_add(0x0101);
        }

        if i != read_le_u16(&self.ram, DUNG_INDEX_OF_TORCHES) as usize {
            attr = 0xc0c0;
            while i != read_le_u16(&self.ram, DUNG_INDEX_OF_TORCHES) as usize {
                let j =
                    (read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + (i >> 1) * 2) & 0x3fff) >> 1;
                self.write_attr2(j as usize + xy(0, 0), attr);
                self.write_attr2(j as usize + xy(0, 1), attr);
                i += 2;
                attr = (attr & 0xefef).wrapping_add(0x0101);
            }
            write_le_u16(&mut self.ram, DUNG_INDEX_OF_TORCHES, 0);
        }

        let mut attr = 0x5858u16;
        let mut i = 0usize;
        let skip_big_key_locks = read_le_u16(&self.ram, DUNG_NUM_CHESTS_X2) != 0
            && self.hud_tags_suppress_big_key_locks();
        if read_le_u16(&self.ram, DUNG_NUM_CHESTS_X2) != 0 && !skip_big_key_locks {
            while i != read_le_u16(&self.ram, DUNG_NUM_CHESTS_X2) as usize {
                let k = read_le_u16(&self.ram, DUNG_CHEST_LOCATIONS + (i >> 1) * 2);
                if k != 0 {
                    let j = (k & 0x7fff) >> 1;
                    self.write_attr2(j as usize + xy(0, 0), attr);
                    self.write_attr2(j as usize + xy(0, 1), attr);
                    if k & 0x8000 != 0 {
                        write_le_u16(
                            &mut self.ram,
                            DUNG_CHEST_LOCATIONS + (i >> 1) * 2,
                            k & 0x7fff,
                        );
                        self.write_attr2(j as usize + xy(2, 1), attr);
                        self.write_attr2(j as usize + xy(0, 2), attr);
                        self.write_attr2(j as usize + xy(2, 2), attr);
                    }
                }
                i += 2;
                attr = attr.wrapping_add(0x0101);
            }
        }

        if !skip_big_key_locks {
            while i != read_le_u16(&self.ram, DUNG_NUM_BIGKEY_LOCKS_X2) as usize {
                let offset = DUNG_CHEST_LOCATIONS + (i >> 1) * 2;
                let k = read_le_u16(&self.ram, offset);
                write_le_u16(&mut self.ram, offset, k | 0x8000);
                let j = (k & 0x7fff) >> 1;
                self.write_attr2(j as usize + xy(0, 0), attr);
                self.write_attr2(j as usize + xy(0, 1), attr);
                i += 2;
                attr = attr.wrapping_add(0x0101);
            }
        }

        i = 0;
        let mut stair_type = 0u16;
        let mut iend = read_le_u16(&self.ram, DUNG_NUM_STAIRS_1) as usize;
        attr = 0x3f3f;
        if iend == 0 {
            stair_type = 1;
            attr = 0x3e3e;
            iend = read_le_u16(&self.ram, DUNG_NUM_STAIRS_2) as usize;
            if iend == 0 {
                stair_type = 2;
                attr = 0x3d3d;
                iend = read_le_u16(&self.ram, DUNG_NUM_STAIRS_WET) as usize;
            }
        }
        if iend != 0 {
            write_le_u16(&mut self.ram, KIND_OF_IN_ROOM_STAIRCASE_DUNGEON, stair_type);
            while i != iend {
                let j = read_le_u16(&self.ram, DUNG_STAIRS_TABLE_2 + (i >> 1) * 2) as usize;
                self.write_attr1(j + xy(0, 0), 0x0002);
                self.write_attr2(j + xy(0, 3), 0x0002);
                self.write_attr1(j + xy(0, 1), 0x0001);
                self.write_attr2(j + xy(0, 2), 0x0001);
                self.write_attr1(j + xy(2, 0), 0x0200);
                self.write_attr2(j + xy(2, 3), 0x0200);
                self.write_attr1(j + xy(2, 1), 0x0100);
                self.write_attr2(j + xy(2, 2), 0x0100);
                self.write_attr1(j + xy(1, 1), attr);
                self.write_attr2(j + xy(1, 1), attr);
                self.write_attr1(j + xy(1, 2), attr);
                self.write_attr2(j + xy(1, 2), attr);
                i += 2;
            }
        }

        if read_le_u16(&self.ram, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER) != 0 {
            write_le_u16(&mut self.ram, KIND_OF_IN_ROOM_STAIRCASE_DUNGEON, 2);
            i = 0;
            while i != read_le_u16(&self.ram, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER) as usize {
                let j = read_le_u16(&self.ram, DUNG_STAIRS_TABLE_2 + (i >> 1) * 2) as usize;
                self.write_attr1(j + xy(0, 3), 0x0a03);
                self.write_attr1(j + xy(2, 3), 0x030a);
                self.write_attr2(j + xy(0, 3), 0x0003);
                self.write_attr2(j + xy(2, 3), 0x0300);
                self.write_attr2(j + xy(0, 2), 0x0808);
                self.write_attr2(j + xy(2, 2), 0x0808);
                i += 2;
            }
        }
        self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
    }

    fn hud_tags_suppress_big_key_locks(&self) -> bool {
        (0..2).any(|i| {
            let tag = self.ram[DUNG_HDR_TAG + i];
            tag == 0x27 || tag == 0x3c || tag == 0x3e || (0x29..0x33).contains(&tag)
        })
    }

    fn Dungeon_LoadDoorAttribute(&mut self) {
        for k in 0..16 {
            if read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + k * 2) != 0 {
                self.Dungeon_LoadSingleDoorAttribute(k);
            }
        }
        self.Dungeon_LoadSingleDoorTileAttribute();
        self.ChangeDoorToSwitch();
        self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
    }

    pub(super) fn Dungeon_LoadToggleDoorAttr_OtherEntry(&mut self, door: i32) {
        self.Dungeon_LoadSingleDoorAttribute(door as usize);
        self.Dungeon_LoadSingleDoorTileAttribute();
    }

    fn Dungeon_LoadSingleDoorAttribute(&mut self, k: usize) {
        const TILE_ATTRS_BY_DOOR: [u16; 40] = [
            0x8080, 0x8484, 0x0000, 0x0101, 0x8484, 0x8e8e, 0x0000, 0x0000, 0x8888, 0x8e8e, 0x8080,
            0x8080, 0x8282, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8282, 0x8e8e,
            0x8080, 0x8282, 0x8080, 0x8080, 0x8080, 0x8282, 0x8282, 0x8080, 0x8080, 0x8080, 0x8484,
            0x8484, 0x8686, 0x8888, 0x8686, 0x8686, 0x8080, 0x8080,
        ];
        let t = self.ram[DOOR_TYPE_AND_SLOT + k * 2] & 0xfe;
        if std::env::var_os("ZELDA3_REPLAY_DOOR_ATTR_TRACE").is_some() {
            eprintln!(
                "door-attr frame={} entry k={} t=0x{:02x} raw=0x{:04x} opened=0x{:04x} opened_adj=0x{:04x} cur=0x{:04x} addr=0x{:04x} dir=0x{:04x} sub={} step=0x{:04x}",
                self.ram[FRAME_COUNTER],
                k,
                t,
                read_le_u16(&self.ram, DOOR_TYPE_AND_SLOT + k * 2),
                read_le_u16(&self.ram, DUNG_DOOR_OPENED),
                read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT),
                read_le_u16(&self.ram, DUNG_CUR_DOOR_POS_DUNGEON),
                read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + k * 2),
                read_le_u16(&self.ram, DUNG_DOOR_DIRECTION + k * 2),
                self.frame_control_view().submodule(),
                read_le_u16(&self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON),
            );
        }
        if !matches!(
            t,
            DOOR_TYPE_REGULAR
                | DOOR_TYPE_ENTRANCE_DOOR
                | DOOR_TYPE_EXIT_TO_OW
                | DOOR_TYPE_ENTRANCE_LARGE
                | DOOR_TYPE_ENTRANCE_CAVE
                | DOOR_TYPE_ENTRANCE_LARGE2
                | DOOR_TYPE_ENTRANCE_CAVE2
                | DOOR_TYPE_4
                | DOOR_TYPE_REGULAR2
                | DOOR_TYPE_WATERFALL_TUNNEL
        ) {
            if t == DOOR_TYPE_LG_EXPLOSION {
                return;
            }
            if t >= DOOR_TYPE_REGULAR_DOOR33 {
                if t != DOOR_TYPE_REGULAR_DOOR33
                    && t != DOOR_TYPE_WARP_ROOM_DOOR
                    && read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) & upper_bitmask(k)
                        == 0
                {
                    let j = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + k * 2) >> 1;
                    let attr = (0xf0u16.wrapping_add(k as u16)).wrapping_mul(0x0101);
                    self.write_attr2(j as usize + xy(1, 1), attr);
                    self.write_attr2(j as usize + xy(1, 2), attr);
                    return;
                }
            } else {
                let i = if t == DOOR_TYPE_SHUTTERS_TWO_WAY || t == DOOR_TYPE_SHUTTER {
                    k
                } else {
                    k & 7
                };
                if read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) & upper_bitmask(i) == 0 {
                    let j = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + k * 2) >> 1;
                    let attr = (0xf0u16.wrapping_add(k as u16)).wrapping_mul(0x0101);
                    self.write_attr2(j as usize + xy(1, 1), attr);
                    self.write_attr2(j as usize + xy(1, 2), attr);
                    return;
                }
            }
        }

        if (DOOR_TYPE_STAIR_MASK_LOCKED0..=DOOR_TYPE_STAIR_MASK_LOCKED3).contains(&t) {
            if std::env::var_os("ZELDA3_REPLAY_DOOR_ATTR_TRACE").is_some() {
                eprintln!(
                    "door-attr frame={} stairmask-return k={} t=0x{:02x}",
                    self.ram[FRAME_COUNTER], k, t,
                );
            }
            return;
        }
        let mut attr = TILE_ATTRS_BY_DOOR
            .get(t as usize >> 1)
            .copied()
            .unwrap_or(0x8080);
        if std::env::var_os("ZELDA3_REPLAY_DOOR_ATTR_TRACE").is_some() {
            eprintln!(
                "door-attr frame={} alpha k={} t=0x{:02x} attr=0x{:04x}",
                self.ram[FRAME_COUNTER], k, t, attr,
            );
        }
        let dir = self.ram[DUNG_DOOR_DIRECTION + k * 2] & 3;
        let address = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + k * 2);
        let beta = matches!(
            t,
            DOOR_TYPE_ENTRANCE_LARGE2
                | DOOR_TYPE_ENTRANCE_CAVE2
                | DOOR_TYPE_4
                | DOOR_TYPE_REGULAR2
                | DOOR_TYPE_WATERFALL_TUNNEL
                | DOOR_TYPE_REGULAR_DOOR33
                | DOOR_TYPE_WARP_ROOM_DOOR
        ) || (t >= DOOR_TYPE_REGULAR_DOOR33
            && read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) & upper_bitmask(k) != 0);

        if !beta {
            if dir == 0 {
                if self.door_address_is_exit(address) {
                    attr = 0x8e8e;
                }
                let j = ((address >> 1) & !0x07c0) as usize;
                for y in 0..=6 {
                    self.write_attr2(j + xy(1, y), attr);
                }
                self.write_attr2(j + xy(1, 7), 0);
            } else if dir == 1 {
                if t == DOOR_TYPE_ENTRANCE_LARGE
                    || t == DOOR_TYPE_ENTRANCE_CAVE
                    || self.door_address_is_exit(address)
                {
                    attr = 0x8e8e;
                }
                let j = (address >> 1) as usize;
                for y in 1..=5 {
                    self.write_attr2(j + xy(1, y), attr);
                }
            } else if dir == 2 {
                let j = ((address >> 1) & !0x001f) as usize;
                self.write_attr2(j + xy(0, 1), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(2, 1), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(0, 2), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(2, 2), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(4, 1), attr.wrapping_add(0x0101) & 0x00ff);
                self.write_attr2(j + xy(4, 2), attr.wrapping_add(0x0101) & 0x00ff);
            } else {
                let j = (address >> 1) as usize;
                self.write_attr2(j + xy(2, 1), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(4, 1), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(2, 2), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(4, 2), attr.wrapping_add(0x0101));
                self.write_attr2(j + xy(0, 1), attr.wrapping_add(0x0101) & 0xff00);
                self.write_attr2(j + xy(0, 2), attr.wrapping_add(0x0101) & 0xff00);
            }
            return;
        }

        if dir == 0 {
            let j = ((address >> 1) & !0x07c0) as usize;
            for y in 0..=9 {
                self.write_attr2(j + xy(1, y), attr);
            }
        } else if dir == 1 {
            if t == DOOR_TYPE_ENTRANCE_LARGE2
                || t == DOOR_TYPE_ENTRANCE_CAVE2
                || t == DOOR_TYPE_4
                || self.door_address_is_exit(address & 0x1fff)
            {
                attr = 0x8e8e;
            }
            let j = (address >> 1) as usize;
            for y in 1..=8 {
                self.write_attr2(j + xy(1, y), attr);
            }
        } else if dir == 2 {
            let j = ((address >> 1) & !0x001f) as usize;
            for y in 1..=2 {
                for x in [0, 2, 4, 6] {
                    self.write_attr2(j + xy(x, y), attr.wrapping_add(0x0101));
                }
            }
        } else {
            let j = (address >> 1).wrapping_add(1) as usize;
            for y in 1..=2 {
                for x in [0, 2, 4, 6] {
                    self.write_attr2(j + xy(x, y), attr.wrapping_add(0x0101));
                }
            }
        }
    }

    fn door_address_is_exit(&self, address: u16) -> bool {
        (0..4).any(|i| read_le_u16(&self.ram, DUNG_EXIT_DOOR_ADDRESSES + i * 2) == address)
    }

    fn Door_LoadBlastWallAttr(&mut self, k: usize) {
        let mut j = (read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + k * 2) >> 1) as usize;
        if self.ram[DUNG_DOOR_DIRECTION + k * 2] & 2 == 0 {
            for _ in 0..12 {
                self.write_attr2(j + xy(0, 0), 0x0102);
                for i in (2..20).step_by(2) {
                    self.write_attr2(j + xy(i, 0), 0);
                }
                self.write_attr2(j + xy(20, 0), 0x0201);
                j += xy(0, 1);
            }
        } else {
            for _ in 0..5 {
                self.write_attr2(j + xy(0, 0), 0x0101);
                self.write_attr2(j + xy(0, 21), 0x0101);
                self.write_attr2(j + xy(0, 1), 0x0202);
                self.write_attr2(j + xy(0, 20), 0x0202);
                for i in 2..20 {
                    self.write_attr2(j + xy(0, i), 0);
                }
                j += xy(2, 0);
            }
        }
    }

    fn ChangeDoorToSwitch(&self) {
        assert_eq!(read_le_u16(&self.ram, DUNG_WIDTH_ROAD_ADDRESS), 0);
    }

    fn Dungeon_FlipCrystalPegAttribute(&mut self) {
        for i in (0..=0x0fff).rev() {
            if self.ram[DUNG_BG2_ATTR_TABLE + i] & !1 == 0x66 {
                self.ram[DUNG_BG2_ATTR_TABLE + i] ^= 1;
            }
            if self.ram[DUNG_BG1_ATTR_TABLE + i] & !1 == 0x66 {
                self.ram[DUNG_BG1_ATTR_TABLE + i] ^= 1;
            }
        }
    }

    fn write_attr2(&mut self, j: usize, attr: u16) {
        let base = DUNG_BG2_ATTR_TABLE + j;
        if base + 1 >= self.ram.len() {
            if std::env::var_os("ZELDA3_REPLAY_DUNGEON_ATTR_TRACE").is_some() {
                eprintln!(
                    "attr-write-oob frame={} fn=write_attr2 j=0x{:04x} attr=0x{:04x} base=0x{:05x} ram_len=0x{:05x} stairs1=0x{:04x} stairs2=0x{:04x} inter=0x{:04x} misc=0x{:04x} chest=0x{:04x} big=0x{:04x} counts={:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x}",
                    self.state_recorder.replay_frame_counter,
                    j,
                    attr,
                    base,
                    self.ram.len(),
                    read_le_u16(&self.ram, DUNG_STAIRS_TABLE_1),
                    read_le_u16(&self.ram, DUNG_STAIRS_TABLE_2),
                    read_le_u16(&self.ram, DUNG_INTER_STAIRCASES),
                    read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX),
                    read_le_u16(&self.ram, DUNG_NUM_CHESTS_X2),
                    read_le_u16(&self.ram, DUNG_NUM_BIGKEY_LOCKS_X2),
                    read_le_u16(&self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS),
                    read_le_u16(&self.ram, DUNG_NUM_INROOM_SOUTHDOWN_STAIRS),
                    read_le_u16(&self.ram, DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS),
                    read_le_u16(&self.ram, WATER_SIDE_STEP_SWITCH),
                    read_le_u16(&self.ram, DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER),
                    read_le_u16(&self.ram, DUNG_NUM_ACTIVATED_WATER_LADDERS),
                    read_le_u16(&self.ram, DUNG_NUM_STAIRS_1),
                    read_le_u16(&self.ram, DUNG_NUM_STAIRS_2),
                );
            }
            return;
        }
        if std::env::var_os("ZELDA3_REPLAY_DUNGEON_ATTR_TRACE").is_some() {
            let frame_target = std::env::var("ZELDA3_REPLAY_DUNGEON_ATTR_FRAME")
                .ok()
                .and_then(|value| parse_usize_env(&value));
            let target = std::env::var("ZELDA3_REPLAY_DUNGEON_ATTR_POS")
                .ok()
                .and_then(|value| parse_usize_env(&value));
            let frame_matches = frame_target
                .map(|target| self.state_recorder.replay_frame_counter as usize == target)
                .unwrap_or(true);
            if frame_matches
                && match target {
                    Some(target) => j == target || j + 1 == target,
                    None => true,
                }
            {
                let before0 = self.ram.get(base).copied();
                let before1 = self.ram.get(base + 1).copied();
                eprintln!(
                    "attr-write frame={} fn=write_attr2 j=0x{:04x} attr=0x{:04x} addr=0x{:05x} before={}/{} door_open=0x{:04x} door_adj=0x{:04x} cur=0x{:04x} sub={} step=0x{:04x}",
                    self.state_recorder.replay_frame_counter,
                    j,
                    attr,
                    base,
                    format_optional_hex(before0),
                    format_optional_hex(before1),
                    read_le_u16(&self.ram, DUNG_DOOR_OPENED),
                    read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT),
                    read_le_u16(&self.ram, DUNG_CUR_DOOR_POS_DUNGEON),
                    self.frame_control_view().submodule(),
                    read_le_u16(&self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON),
                );
            }
        }
        self.ram[base] = attr as u8;
        self.ram[base + 1] = (attr >> 8) as u8;
    }

    fn write_attr1(&mut self, j: usize, attr: u16) {
        let base = DUNG_BG1_ATTR_TABLE + j;
        if base + 1 >= self.ram.len() {
            return;
        }
        self.ram[base] = attr as u8;
        self.ram[base + 1] = (attr >> 8) as u8;
    }

    fn Dungeon_LoadSingleDoorTileAttribute(&mut self) {
        let mut i = 0usize;
        while i != read_le_u16(&self.ram, DUNG_NUM_TOGGLE_FLOOR) as usize {
            let j = read_le_u16(&self.ram, DUNG_TOGGLE_FLOOR_POS + (i >> 1) * 2) as usize;
            if self.ram[DUNG_BG2_ATTR_TABLE + j] & 0xf0 == 0x80 {
                let attr = read_le_u16(&self.ram, DUNG_BG2_ATTR_TABLE + j);
                self.write_attr2(j + xy(0, 0), attr | 0x1010);
                self.write_attr2(j + xy(0, 1), attr | 0x1010);
            } else {
                let attr = read_le_u16(&self.ram, DUNG_BG1_ATTR_TABLE + j);
                self.write_attr1(j + xy(0, 0), attr | 0x1010);
                self.write_attr1(j + xy(0, 1), attr | 0x1010);
            }
            i += 2;
        }

        i = 0;
        while i != read_le_u16(&self.ram, DUNG_NUM_TOGGLE_PALACE) as usize {
            let j = read_le_u16(&self.ram, DUNG_TOGGLE_PALACE_POS + (i >> 1) * 2) as usize;
            if self.ram[DUNG_BG2_ATTR_TABLE + j] & 0xf0 == 0x80 {
                let attr = read_le_u16(&self.ram, DUNG_BG2_ATTR_TABLE + j);
                self.write_attr2(j + xy(0, 0), attr | 0x2020);
                self.write_attr2(j + xy(0, 1), attr | 0x2020);
            } else {
                let attr = read_le_u16(&self.ram, DUNG_BG1_ATTR_TABLE + j);
                self.write_attr1(j + xy(0, 0), attr | 0x2020);
                self.write_attr1(j + xy(0, 1), attr | 0x2020);
            }
            i += 2;
        }
    }

    pub(super) fn Mirror_SaveRoomData(&mut self) {
        if self.ram[CUR_PALACE_INDEX_X2] == 0xff {
            self.ram[SOUND_EFFECT_1] = 60;
            return;
        }
        self.frame_control_view_mut().set_submodule(25);
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[SOUND_EFFECT_1] = 51;
        self.Dungeon_FlagRoomData_Quadrants();
        self.SaveDungeonKeys();
    }

    pub(super) fn Dung_TagRoutine_0x00(&mut self, _k: usize) {}

    pub(super) fn Dungeon_DetectStaircase(&mut self) {
        const BUGGY_LOOKUP: [i8; 8] = [7, 24, 8, 8, 0, 0, -1, 17];
        let k = self.ram[LINK_DIRECTION] & 12;
        if k == 0 {
            return;
        }

        let lookup = BUGGY_LOOKUP[(k >> 1) as usize] as i16 as u16;
        let mut pos = (self.player_state_view().y().wrapping_add(lookup) & 0x01f8) << 3;
        pos |= (self.player_state_view().x() & 0x01f8) >> 3;
        if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
            pos |= 0x1000;
        }

        let at_pos = pos.wrapping_add(if k == 4 { 0x80 } else { 0 }) as usize;
        let at = self.ram[DUNG_BG2_ATTR_TABLE + at_pos];
        if !matches!(at, 0x26 | 0x38 | 0x39 | 0x5e | 0x5f) {
            return;
        }

        let attr2 = self.ram[DUNG_BG2_ATTR_TABLE + pos as usize + xy(0, 1)];
        if attr2 & 0xf8 != 0x30 {
            return;
        }

        if self.ram[LINK_STATE_BITS] & 0x80 != 0 {
            copy_le_u16(&mut self.ram, LINK_Y_COORD, LINK_Y_COORD_PREV);
            return;
        }

        self.ram[WHICH_STAIRCASE_INDEX] = attr2;
        self.ram[WHICH_STAIRCASE_INDEX + 1] = (pos >> 8) as u8;
        copy_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX_PREV, DUNGEON_ROOM_INDEX);
        self.Dungeon_FlagRoomData_Quadrants();

        if at == 0x38 || at == 0x39 {
            self.ram[STAIRCASE_MOVE_COUNTER] = 0x20;
            if at == 0x38 {
                self.Dungeon_StartInterRoomTrans_Up();
            } else {
                self.Dungeon_StartInterRoomTrans_Down();
            }
        }

        let j = (self.ram[WHICH_STAIRCASE_INDEX] & 3) as usize;
        self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNG_HDR_TRAVEL_DESTINATIONS + j + 1];
        self.ram[CUR_STAIRCASE_PLANE] = self.ram[DUNG_HDR_STAIRCASE_PLANE + j];
        self.ram[STAIRCASE_LOWER_LEVEL_STATUS] = if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0
            || self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] != 0
        {
            2
        } else {
            0
        };
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[BUTTON_MASK_B_Y] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;

        if at == 0x26 {
            self.frame_control_view_mut().set_submodule(6);
            self.ram[SOUND_EFFECT_1] = if self.ram[CUR_STAIRCASE_PLANE] < 0x34 {
                22
            } else {
                24
            };
        } else if at == 0x38 || at == 0x39 {
            let submodule = if at == 0x38 { 18 } else { 19 };
            self.frame_control_view_mut().set_submodule(submodule);
            self.ram[LINK_TIMER_PUSH_GET_TIRED] = 7;
        } else {
            self.UsedForStraightInterRoomStaircase();
            self.frame_control_view_mut().set_submodule(14);
        }
    }

    pub(super) fn UsedForStraightInterRoomStaircase(&mut self) {
        for i in (0..=9).rev() {
            if self.ram[ANCILLA_TYPE + i] == 13 {
                self.ram[ANCILLA_TYPE + i] = 0;
            }
        }
        if self.ram[LINK_ANIMATION_STEPS] >= 5 {
            self.ram[LINK_ANIMATION_STEPS] = 0;
        }
        self.ram[LINK_SUBPIXEL_X] = 0;
        self.ram[LINK_SUBPIXEL_Y] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = 28;
        self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] = 32;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        self.ancilla_sfx2_near(if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            0x18
        } else {
            0x16
        });

        let x = self.player_state_view().x();
        let detect_x = if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            x.wrapping_sub(15)
        } else {
            x.wrapping_add(16)
        };
        write_le_u16(&mut self.ram, TILEDETECT_WHICH_Y_POS + 2, detect_x);
        copy_le_u16(&mut self.ram, TILEDETECT_WHICH_Y_POS, LINK_Y_COORD);
    }

    pub(super) fn RoomTag_MovingWall_East(&mut self, k: usize) {
        const MOVING_WALL_TAB1: [u16; 8] = [
            (-63i16) as u16,
            (-127i16) as u16,
            (-191i16) as u16,
            (-255i16) as u16,
            (-71i16) as u16,
            (-135i16) as u16,
            (-199i16) as u16,
            (-263i16) as u16,
        ];

        if read_le_u16(&self.ram, DUNG_FLOOR_MOVE_FLAGS) == 0 {
            self.RoomTag_MovingWallTorchesCheck(k);
            write_le_u16(&mut self.ram, DUNG_FLOOR_X_VEL, 0);
        } else {
            self.ram[FLAG_UNK1] = 1;
            self.RoomTag_MovingWallShakeItUp(k);
            let vel = self.MovingWall_MoveALittle();
            write_le_u16(&mut self.ram, DUNG_FLOOR_X_VEL, vel);
        }

        let offs = read_le_u16(&self.ram, DUNG_FLOOR_X_OFFS)
            .wrapping_sub(read_le_u16(&self.ram, DUNG_FLOOR_X_VEL));
        write_le_u16(&mut self.ram, DUNG_FLOOR_X_OFFS, offs);
        let bg1 = read_le_u16(&self.ram, BG2HOFS_COPY2).wrapping_add(offs);
        write_le_u16(&mut self.ram, BG1HOFS_COPY2, bg1);

        if read_le_u16(&self.ram, DUNG_FLOOR_X_VEL) != 0 {
            let target0 = MOVING_WALL_TAB1[(self.ram[MOVING_WALL_DOT_POINTER] >> 1) as usize & 7];
            if offs < target0 {
                let target1 =
                    MOVING_WALL_TAB1[(self.RoomTag_AdvanceGiganticWall(k) >> 1) as usize & 7];
                if offs < target1 {
                    self.finish_moving_wall_tag(k);
                }
            }
            self.ram[NMI_SUBROUTINE_INDEX] = 5;
            let neg = (0u16.wrapping_sub(offs) & 0x01f8) >> 3;
            let target = read_le_u16(&self.ram, MOVING_WALL_WRITE_POINT).wrapping_sub(neg) & 0x141f;
            write_le_u16(&mut self.ram, NMI_LOAD_TARGET_ADDR, target);
        }
    }

    pub(super) fn RoomTag_MovingWallShakeItUp(&mut self, k: usize) {
        let x = if self.ram[FRAME_COUNTER] & 1 != 0 {
            -1i16
        } else {
            1
        };
        write_le_u16(&mut self.ram, BG1_X_OFFSET, x as u16);
        write_le_u16(&mut self.ram, BG1_Y_OFFSET, (-x) as u16);
        if self.ram[DUNG_HDR_TAG + k] == 0 {
            write_le_u16(&mut self.ram, BG1_X_OFFSET, 0);
            write_le_u16(&mut self.ram, BG1_Y_OFFSET, 0);
        }
    }

    pub(super) fn RoomTag_MovingWall_West(&mut self, k: usize) {
        const MOVING_WALL_TAB0: [u16; 8] = [0x42, 0x82, 0xc2, 0x102, 0x4a, 0x8a, 0xca, 0x10a];

        if read_le_u16(&self.ram, DUNG_FLOOR_MOVE_FLAGS) == 0 {
            self.RoomTag_MovingWallTorchesCheck(k);
            write_le_u16(&mut self.ram, DUNG_FLOOR_X_VEL, 0);
        } else {
            self.ram[FLAG_UNK1] = 1;
            self.RoomTag_MovingWallShakeItUp(k);
            let vel = self.MovingWall_MoveALittle();
            write_le_u16(&mut self.ram, DUNG_FLOOR_X_VEL, vel);
        }

        let offs = read_le_u16(&self.ram, DUNG_FLOOR_X_OFFS)
            .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_X_VEL));
        write_le_u16(&mut self.ram, DUNG_FLOOR_X_OFFS, offs);
        let bg1 = read_le_u16(&self.ram, BG2HOFS_COPY2).wrapping_add(offs);
        write_le_u16(&mut self.ram, BG1HOFS_COPY2, bg1);

        if read_le_u16(&self.ram, DUNG_FLOOR_X_VEL) != 0 {
            let target0 = MOVING_WALL_TAB0[(self.ram[MOVING_WALL_DOT_POINTER] >> 1) as usize & 7];
            if offs >= target0 {
                let target1 =
                    MOVING_WALL_TAB0[(self.RoomTag_AdvanceGiganticWall(k) >> 1) as usize & 7];
                if offs >= target1 {
                    self.finish_moving_wall_tag(k);
                }
            }
            self.ram[NMI_SUBROUTINE_INDEX] = 5;
            let mut target =
                read_le_u16(&self.ram, MOVING_WALL_WRITE_POINT).wrapping_add((offs & 0x01f8) >> 3);
            if target & 0x1020 != 0 {
                target = (target & 0x1020) ^ 0x0420;
            }
            write_le_u16(&mut self.ram, NMI_LOAD_TARGET_ADDR, target);
        }
    }

    fn finish_moving_wall_tag(&mut self, k: usize) {
        self.ram[SOUND_EFFECT_2] = 0x1b;
        self.ram[SOUND_EFFECT_AMBIENT] = 5;
        self.ram[DUNG_HDR_TAG + k] = 0;
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
        self.ram[FLAG_UNK1] = 0;
        write_le_u16(&mut self.ram, BG1_X_OFFSET, 0);
        write_le_u16(&mut self.ram, BG1_Y_OFFSET, 0);
    }

    pub(super) fn RoomTag_MovingWallTorchesCheck(&mut self, k: usize) {
        if read_le_u16(&self.ram, DUNG_FLAG_STATECHANGE_WATERPUZZLE) == 0 {
            let mut count = 0;
            for i in 0..16 {
                count +=
                    u8::from(read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + i * 2) & 0x8000 != 0);
            }
            if count < 4 {
                return;
            }
        }
        let flags = read_le_u16(&self.ram, DUNG_FLOOR_MOVE_FLAGS).wrapping_add(1);
        write_le_u16(&mut self.ram, DUNG_FLOOR_MOVE_FLAGS, flags);
        write_le_u16(&mut self.ram, DUNG_FLAG_STATECHANGE_WATERPUZZLE, 0);
        let save_bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) | (0x1000 >> k);
        write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, save_bits);
        self.ram[SOUND_EFFECT_AMBIENT] = 7;
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
        self.ram[FLAG_UNK1] = 1;
    }

    pub(super) fn MovingWall_MoveALittle(&mut self) -> u16 {
        let t = (self.ram[BG1_MOVE_CALC_BUFFER + 1] as u16).wrapping_add(0x22);
        self.ram[BG1_MOVE_CALC_BUFFER + 1] = t as u8;
        t >> 8
    }

    pub(super) fn RoomTag_AdvanceGiganticWall(&mut self, k: usize) -> u8 {
        let mut i = self.ram[MOVING_WALL_DOT_POINTER];
        if self.ram[DUNG_HDR_TAG + k] < 0x20 {
            self.ram[DUNG_HDR_COLLISION] = 0;
            self.ram[TM_COPY] = 0x16;
            i = i.wrapping_add(8);
        }
        i
    }

    pub(super) fn Dungeon_SaveAndLoadAllPalettes(&mut self, main_tile_theme: u8, sprite_gfx: u8) {
        self.ram[SPRITE_GRAPHICS_INDEX] = sprite_gfx;
        self.ram[MAIN_TILE_THEME_INDEX] = main_tile_theme;
        self.ram[AUX_TILE_THEME_INDEX] = main_tile_theme;
        self.initialize_tilesets();
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x200);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        self.palette_bg_and_fixed_color_black();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sprite_environment_dungeon();
        self.palette_load_hud();
        self.palette_load_dungeon_set();
    }
    pub(super) fn Dungeon_CheckForAndIDLiftableTile(&self) -> u16 {
        const X: [i8; 4] = [7, 7, -3, 16];
        const Y: [i8; 4] = [3, 24, 14, 14];
        const RV: [u16; 16] = [
            0x5252, 0x5050, 0x5454, 0x0000, 0x2323, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        ];

        let facing = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(X[facing] as i16 as u16)
            & 0x01f8;
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(Y[facing] as i16 as u16)
            & 0x01f8;
        let offset = ((y << 3) | (x >> 3)) as usize
            + if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
                0x1000
            } else {
                0
            };

        let attr = self.ram[DUNG_BG2_ATTR_TABLE + offset];
        if attr & 0xf0 != 0x70 {
            return 0xffff;
        }

        let replacement = read_le_u16(
            &self.ram,
            DUNG_REPLACEMENT_TILE_STATE + (attr & 0x0f) as usize * 2,
        );
        if replacement == 0 {
            return 0xffff;
        }
        if replacement & 0xf0f0 == 0x2020 {
            return 0x55;
        }
        RV[(replacement & 0x0f) as usize]
    }

    pub(super) fn OpenChestForItem(&mut self, tile: u8, chest_position: &mut u16) -> u8 {
        if let Some((item, position)) = self.OpenChestForItemResult(tile) {
            *chest_position = position;
            item
        } else {
            0xff
        }
    }

    pub(super) fn OpenChestForItemResult(&mut self, tile: u8) -> Option<(u8, u16)> {
        const CHEST_OPEN_MASKS: [u16; 6] = [0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000];
        if tile == 0x63 {
            return self.OpenMiniGameChestResult();
        }
        let chest_idx_org = tile.wrapping_sub(0x58) as usize;
        let loc = read_le_u16(&self.ram, DUNG_CHEST_LOCATIONS + chest_idx_org * 2);
        let palace_mask =
            upper_bitmask((read_le_u16(&self.ram, CUR_PALACE_INDEX_X2) >> 1) as usize);
        if loc >= 0x8000 {
            if read_le_u16(&self.ram, LINK_BIGKEY) & palace_mask == 0 {
                write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x007a);
                self.main_show_text_message();
                return None;
            }
            self.write_u16_ram(
                DUNG_SAVEGAME_STATE_BITS,
                self.read_u16_ram(DUNG_SAVEGAME_STATE_BITS) | CHEST_OPEN_MASKS[chest_idx_org],
            );
            self.ram[SOUND_EFFECT_1] = 0x29;
            self.ram[SOUND_EFFECT_2] = 0x15;
            let pos = (loc & 0x7fff) >> 1;
            let src = self
                .read_predefined_tile_words(read_le_u16(&self.ram, DUNG_FLOOR_2_FILLER_TILES), 4);
            let chest_position = self.apply_opened_chest_tiles(pos, loc, &src);
            return Some((0xff, chest_position));
        }

        let chest_data = self
            .asset_raw(8)
            .expect("missing dungeon room chests asset")
            .to_vec();
        let mut chest_idx = chest_idx_org as isize;
        let room = self.world_state_view().dungeon_room();
        for entry in chest_data.chunks_exact(3) {
            let chest_room = read_word_from_slice(entry, 0);
            if (chest_room & 0x7fff) == room {
                chest_idx -= 1;
                if chest_idx < 0 {
                    let item = entry[2];
                    if chest_room & 0x8000 != 0 {
                        if read_le_u16(&self.ram, LINK_BIGKEY) & palace_mask == 0 {
                            write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x007a);
                            self.main_show_text_message();
                            return None;
                        }
                        self.write_u16_ram(
                            DUNG_SAVEGAME_STATE_BITS,
                            self.read_u16_ram(DUNG_SAVEGAME_STATE_BITS)
                                | CHEST_OPEN_MASKS[chest_idx_org],
                        );
                        let chest_position = self.OpenBigChestResult(loc);
                        return Some((item, chest_position));
                    }
                    self.write_u16_ram(
                        DUNG_SAVEGAME_STATE_BITS,
                        self.read_u16_ram(DUNG_SAVEGAME_STATE_BITS)
                            | CHEST_OPEN_MASKS[chest_idx_org],
                    );
                    let src = self.read_predefined_tile_words(0x14a4, 4);
                    let chest_position = self.apply_opened_chest_tiles(loc >> 1, loc, &src);
                    return Some((item, chest_position));
                }
            }
        }
        None
    }

    pub(super) fn OpenMiniGameChest(&mut self, chest_position: &mut u16) -> u8 {
        if let Some((item, position)) = self.OpenMiniGameChestResult() {
            *chest_position = position;
            item
        } else {
            0xff
        }
    }

    pub(super) fn OpenMiniGameChestResult(&mut self) -> Option<(u8, u16)> {
        if self.ram[MINIGAME_CREDITS] == 0 {
            write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x0163);
            self.main_show_text_message();
            return None;
        }
        if self.ram[MINIGAME_CREDITS] == 0xff {
            write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x0162);
            self.main_show_text_message();
            return None;
        }
        self.ram[MINIGAME_CREDITS] = self.ram[MINIGAME_CREDITS].wrapping_sub(1);

        let mut pos = (self.player_state_view().y().wrapping_sub(4) & 0x01f8) * 8;
        pos |= (self.player_state_view().x().wrapping_add(7) & 0x01f8) >> 3;
        if read_le_u16(&self.ram, DUNG_BG2_ATTR_TABLE + pos as usize) != 0x6363 {
            pos = pos.wrapping_sub(1);
            if read_le_u16(&self.ram, DUNG_BG2_ATTR_TABLE + pos as usize) != 0x6363 {
                pos = pos.wrapping_add(2);
            }
        }

        write_le_u16(&mut self.ram, DUNG_BG2_ATTR_TABLE + pos as usize, 0x0202);
        write_le_u16(
            &mut self.ram,
            DUNG_BG2_ATTR_TABLE + (pos as usize + 64),
            0x0202,
        );

        let src = self.read_predefined_tile_words(0x14a4, 4);
        let pos_wrong = pos as usize + 128;
        write_le_u16(&mut self.ram, DUNG_BG2 + pos_wrong * 2, src[0]);
        write_le_u16(&mut self.ram, DUNG_BG2 + (pos_wrong + 64) * 2, src[1]);
        write_le_u16(&mut self.ram, DUNG_BG2 + (pos_wrong + 1) * 2, src[2]);
        write_le_u16(&mut self.ram, DUNG_BG2 + (pos_wrong + 65) * 2, src[3]);

        let upload = read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET) as usize;
        let dst = VRAM_UPLOAD_DATA + upload;
        let positions = [pos, pos + 64, pos + 1, pos + 65];
        for (i, &tile_pos) in positions.iter().enumerate() {
            let base = dst + i * 6;
            let vram_addr = self.Dungeon_MapVramAddr(tile_pos);
            write_le_u16(&mut self.ram, base, vram_addr);
            write_le_u16(&mut self.ram, base + 2, 0x0100);
            write_le_u16(&mut self.ram, base + 4, src[i]);
        }
        write_le_u16(&mut self.ram, dst + 24, 0xffff);
        let next_upload = read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET).wrapping_add(24);
        write_le_u16(&mut self.ram, VRAM_UPLOAD_OFFSET, next_upload);

        let old_choice = self.ram[R16];
        let mut choice = self.get_random_number();
        let room = self.ram[DUNGEON_ROOM_INDEX];
        let item = if room == 0 {
            choice &= 0x0f;
            K_DUNGEON_RUPEE_CHEST_MINIGAME_PRIZES[choice as usize]
        } else if room == 0x18 {
            choice = 0x10 + (choice & 0x0f);
            K_DUNGEON_RUPEE_CHEST_MINIGAME_PRIZES[(0x10 + (choice & 0x0f)) as usize]
        } else {
            choice &= 7;
            if choice >= 2 && choice == old_choice {
                choice = choice.wrapping_add(1) & 7;
            }
            if choice == 7 {
                let save_bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS);
                if save_bits & 0x4000 != 0 {
                    choice = 0;
                } else {
                    write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, save_bits | 0x4000);
                }
            }
            K_DUNGEON_MINIGAME_CHEST_PRIZES1[choice as usize]
        };
        self.ram[R16] = choice;
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
        self.ram[SOUND_EFFECT_2] = 14;
        Some((item, pos * 2))
    }

    pub(super) fn OpenBigChest(&mut self, loc: u16, chest_position: &mut u16) {
        *chest_position = self.OpenBigChestResult(loc);
    }

    pub(super) fn OpenBigChestResult(&mut self, loc: u16) -> u16 {
        let pos = loc >> 1;
        let src = self.read_predefined_tile_words(0x14c4, 12);
        for i in 0..4 {
            let dst = pos as usize + i;
            write_le_u16(&mut self.ram, DUNG_BG2 + dst * 2, src[i * 3]);
            write_le_u16(&mut self.ram, DUNG_BG2 + (dst + 64) * 2, src[i * 3 + 1]);
            write_le_u16(&mut self.ram, DUNG_BG2 + (dst + 128) * 2, src[i * 3 + 2]);
        }
        self.dungeon_prep_overlay_dma_next_prep(0, loc);
        for &tile_pos in &[pos, pos + 2, pos + 64, pos + 66, pos + 128, pos + 130] {
            write_le_u16(
                &mut self.ram,
                DUNG_BG2_ATTR_TABLE + tile_pos as usize,
                0x2727,
            );
        }
        self.Dungeon_FlagRoomData_Quadrants();
        self.ram[SOUND_EFFECT_2] = 14;
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;
        self.ram[DUNGEON_TRAP_TRIGGER_LATCH] = 1;
        loc + 2
    }

    pub(super) fn Module07_15_WarpPad(&mut self) {
        if self.frame_control_view().subsubmodule() >= 3 {
            self.Graphics_IncrementalVRAMUpload();
            self.Dungeon_LoadAttribute_Selectable();
        }
        match self.frame_control_view().subsubmodule() {
            0 => self.reset_transition_props_and_advance_reset_interface(),
            1 => self.Module07_15_01_ApplyMosaicAndFilter(),
            2 => self.Dungeon_InitializeRoomFromSpecial(),
            3 => self.DungeonTransition_LoadSpriteGFX(),
            4 => self.Module07_15_04_SyncRoomPropsAndBuildOverlay(),
            5 => self.Dungeon_InterRoomTrans_State4(),
            6 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            7 => self.Dungeon_InterRoomTrans_State4(),
            8 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            9 => self.Dungeon_InterRoomTrans_State4(),
            10 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            11 => self.Dungeon_InterRoomTrans_State4(),
            12 => self.Dungeon_Staircase14(),
            13 => self.Module07_15_0E_FadeInFromWarp(),
            14 => self.Module07_15_0F_FinalizeAndCacheEntry(),
            other => panic!("invalid Module07_15_WarpPad subsubmodule_index {other}"),
        }
    }

    pub(super) fn Dung_TagRoutine_TrapdoorsUp(&mut self) {
        if read_le_u16(&self.ram, DUNG_FLAG_TRAPDOORS_DOWN) != 0 {
            write_le_u16(&mut self.ram, DUNG_FLAG_TRAPDOORS_DOWN, 0);
            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, 0);
            write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
            self.ram[SOUND_EFFECT_2] = 0x1b;
            self.frame_control_view_mut().set_submodule(5);
        }
    }

    pub(super) fn CalculateTransitionLanding(&mut self) -> u8 {
        let mut pos = ((self.player_state_view().y().wrapping_add(12) & 0x01f8) << 3)
            | ((self.player_state_view().x().wrapping_add(8) & 0x01f8) >> 3);
        if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
            pos |= 0x1000;
        }

        let mut attr = self.ram[DUNG_BG2_ATTR_TABLE + pos as usize];
        let result = if attr == 0 || attr == 9 {
            0
        } else {
            attr &= 0x8e;
            if attr == 0x80 {
                1
            } else if attr == 0x82 {
                2
            } else if attr == 0x84 || attr == 0x88 {
                3
            } else if attr == 0x86 {
                4
            } else {
                2
            }
        };

        self.ram[DUNG_TRANSITION_LANDING_CLASS] = result;
        result
    }

    pub(super) fn MirrorBg1Bg2Offs(&mut self) {
        let h = read_le_u16(&self.ram, BG2HOFS_COPY2);
        let v = read_le_u16(&self.ram, BG2VOFS_COPY2);
        write_le_u16(&mut self.ram, BG1HOFS_COPY2, h);
        write_le_u16(&mut self.ram, BG1VOFS_COPY2, v);
    }

    pub(super) fn Dungeon_InterRoomTrans_State13(&mut self) {
        if self.ram[DUNG_WANT_LIGHTS_OUT] | self.ram[DUNG_WANT_LIGHTS_OUT_COPY] != 0 {
            self.ApplyPaletteFilter_bounce();
        }
        self.Dungeon_IntraRoomTrans_State5();
    }

    pub(super) fn Module07_01_SubtileTransition(&mut self) {
        copy_le_u16(&mut self.ram, LINK_Y_COORD_PREV, LINK_Y_COORD);
        copy_le_u16(&mut self.ram, LINK_X_COORD_PREV, LINK_X_COORD);
        self.link_handle_moving_animation_full_long_entry();
        match self.frame_control_view().subsubmodule() {
            0 => self.DungeonTransition_Subtile_PrepTransition(),
            1 => self.DungeonTransition_Subtile_ApplyFilter(),
            2 => self.DungeonTransition_Subtile_ResetShutters(),
            3 => self.DungeonTransition_ScrollRoom(),
            4 => self.DungeonTransition_FindSubtileLanding(),
            5 => self.Dungeon_IntraRoomTrans_State5(),
            6 => self.DungeonTransition_Subtile_ApplyFilter(),
            7 => self.DungeonTransition_Subtile_TriggerShutters(),
            _ => panic!("invalid dungeon subtile transition index"),
        }
    }

    pub(super) fn Module07_02_SupertileTransition(&mut self) {
        copy_le_u16(&mut self.ram, LINK_Y_COORD_PREV, LINK_Y_COORD);
        copy_le_u16(&mut self.ram, LINK_X_COORD_PREV, LINK_X_COORD);
        if self.frame_control_view().subsubmodule() != 0 {
            if self.frame_control_view().subsubmodule() >= 7 {
                self.Graphics_IncrementalVRAMUpload();
            }
            self.Dungeon_LoadAttribute_Selectable();
        }
        self.link_handle_moving_animation_full_long_entry();
        match self.frame_control_view().subsubmodule() {
            0 => self.Module07_02_00_InitializeTransition(),
            1 => self.Module07_02_01_LoadNextRoom(),
            2 => self.Module07_02_FadedFilter(),
            3 => self.Dungeon_InterRoomTrans_State3(),
            4 => self.Dungeon_InterRoomTrans_State4(),
            5 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            6 => self.Dungeon_InterRoomTrans_State4(),
            7 => self.Dungeon_InterRoomTrans_State7(),
            8 => self.DungeonTransition_ScrollRoom(),
            9 => self.Dungeon_InterRoomTrans_State9(),
            10 => self.Dungeon_InterRoomTrans_State10(),
            11 => self.Dungeon_InterRoomTrans_State9(),
            12 => self.Dungeon_InterRoomTrans_State12(),
            13 => self.Dungeon_InterRoomTrans_State13(),
            14 => self.Module07_02_FadedFilter(),
            15 => self.Dungeon_InterRoomTrans_State15(),
            _ => panic!("invalid dungeon supertile transition index"),
        }
    }

    pub(super) fn Module07_02_00_InitializeTransition(&mut self) {
        let bak = self.ram[HDR_DUNGEON_DARK_WITH_LANTERN];
        self.ResetTransitionPropsAndAdvanceSubmodule();
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = bak;
    }

    pub(super) fn Module07_02_01_LoadNextRoom(&mut self) {
        self.Dungeon_LoadRoom();
        self.ResetStarTileGraphics();
        self.LoadTransAuxGFX_sprite();
        self.frame_control_view_mut().increment_subsubmodule();
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.ram[DUNGEON_ROOM_INDEX2] = self.ram[DUNGEON_ROOM_INDEX];
        self.dungeon_reset_sprites();
        if self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] == 0 {
            self.MirrorBg1Bg2Offs();
        }
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 0;
    }

    pub(super) fn Dungeon_InterRoomTrans_State3(&mut self) {
        if self.ram[DUNG_WANT_LIGHTS_OUT] | self.ram[DUNG_WANT_LIGHTS_OUT_COPY] != 0 {
            self.ram[TS_COPY] = 0;
        }
        self.Dungeon_AdjustForRoomLayout();
        self.LoadNewSpriteGFXSet();
        self.MirrorBg1Bg2Offs();
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Module07_07_FallingTransition(&mut self) {
        if self.frame_control_view().subsubmodule() >= 6 {
            self.Graphics_IncrementalVRAMUpload();
            self.Dungeon_LoadAttribute_Selectable();
            self.ApplyGrayscaleFixed_Incremental();
        }
        match self.frame_control_view().subsubmodule() {
            0 => self.Module07_07_00_HandleMusicAndResetRoom(),
            1 => self.ApplyPaletteFilter_bounce(),
            2 => self.Dungeon_InitializeRoomFromSpecial(),
            3 => self.DungeonTransition_TriggerBGC34UpdateAndAdvance(),
            4 => self.DungeonTransition_TriggerBGC56UpdateAndAdvance(),
            5 => self.DungeonTransition_LoadSpriteGFX(),
            6 => self.Module07_07_06_SyncBG1and2(),
            7 => self.Dungeon_InterRoomTrans_State4(),
            8 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            9 => self.Dungeon_InterRoomTrans_State4(),
            10 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            11 => self.Dungeon_InterRoomTrans_State4(),
            12 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            13 => self.Dungeon_InterRoomTrans_State4(),
            14 => self.Dungeon_Staircase14(),
            15 => self.Module07_07_0F_FallingFadeIn(),
            16 => self.Module07_07_10_LandLinkFromFalling(),
            17 => self.Module07_07_11_CacheRoomAndSetMusic(),
            other => panic!("invalid Module07_07_FallingTransition subsubmodule_index {other}"),
        }
    }

    pub(super) fn Module07_07_00_HandleMusicAndResetRoom(&mut self) {
        let room = self.world_state_view().dungeon_room();
        if room == 0x10 || room == 7 || room == 0x17 {
            self.ram[MUSIC_CONTROL] = 0xf1;
        }
        self.ResetTransitionPropsAndAdvance_ResetInterface();
    }

    pub(super) fn Module07_07_06_SyncBG1and2(&mut self) {
        self.MirrorBg1Bg2Offs();
        self.Dungeon_AdjustForRoomLayout();
        let mut ts = K_SPIRAL_TAB1[self.ram[DUNG_HDR_BG2_PROPERTIES] as usize] as u8;
        let mut tm = 0x16;
        if ts & 0x80 != 0 {
            tm = 0x17;
            ts = 0;
        }
        self.ram[TM_COPY] = tm;
        self.ram[TS_COPY] = ts;
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Module07_07_0F_FallingFadeIn(&mut self) {
        self.ApplyPaletteFilter_bounce();
        if self.ram[DARKENING_OR_LIGHTENING_SCREEN] != 0 {
            return;
        }

        let link_y = self.player_state_view().y();
        let detect_y = read_le_u16(&self.ram, TILEDETECT_WHICH_Y_POS);
        let high = ((link_y >> 8) as u8).wrapping_add(u8::from((link_y as u8) >= detect_y as u8));
        self.ram[TILEDETECT_WHICH_Y_POS + 1] = high;
        self.Dungeon_SetBossMusicUnorthodox();

        let room = self.ram[DUNGEON_ROOM_INDEX];
        if room == 0x89 || room == 0x4f {
            return;
        }
        if room == 0xa7 {
            self.ram[HUD_FLOOR_CHANGED_TIMER] = 0;
            self.ram[DUNG_CUR_FLOOR] = 1;
            return;
        }
        self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_sub(1);
        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
    }

    pub(super) fn Module07_07_10_LandLinkFromFalling(&mut self) {
        self.handle_dungeon_landing_from_pit();
        if self.frame_control_view().submodule() != 0 {
            return;
        }
        self.frame_control_view_mut().set_submodule(7);
        self.frame_control_view_mut().set_subsubmodule(17);
        self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 1;
        self.Graphics_LoadChrHalfSlot();
    }

    pub(super) fn Module07_07_11_CacheRoomAndSetMusic(&mut self) {
        if self.ram[OVERWORLD_MAP_STATE] == 5 {
            self.ResetThenCacheRoomEntryProperties();
            self.Dungeon_PlayMusicIfDefeated();
            self.Graphics_LoadChrHalfSlot();
        }
    }

    pub(super) fn Module11_DungeonFallingEntrance(&mut self) {
        match self.frame_control_view().subsubmodule() {
            0 => {
                let entrance_music = self.asset_raw(27).expect("missing entrance music asset")
                    [read_le_u16(&self.ram, WHICH_ENTRANCE) as usize];
                if entrance_music != 3 || self.ram[SRAM_PROGRESS_INDICATOR] >= 2 {
                    self.ram[MUSIC_CONTROL] = 0xf1;
                }
                self.ResetTransitionPropsAndAdvance_ResetInterface();
            }
            1 => {
                if self.ram[FRAME_COUNTER] & 1 == 0 {
                    self.ApplyPaletteFilter_bounce();
                }
            }
            2 => self.Module11_02_LoadEntrance(),
            3 => self.DungeonTransition_LoadSpriteGFX(),
            4 => {
                self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_add(1) & 0x0f;
                if self.ram[INIDISP_COPY] == 15 {
                    self.frame_control_view_mut().increment_subsubmodule();
                }
                self.Module11_DungeonFallingEntrance_land();
            }
            5 => self.Module11_DungeonFallingEntrance_land(),
            _ => {}
        }
    }

    fn Module11_DungeonFallingEntrance_land(&mut self) {
        self.handle_dungeon_landing_from_pit();
        if self.frame_control_view().submodule() != 0 {
            return;
        }
        self.frame_control_view_mut().set_main_module(7);
        self.ram[FLAG_SKIP_CALL_TAG_ROUTINES] =
            self.ram[FLAG_SKIP_CALL_TAG_ROUTINES].wrapping_add(1);
        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
        self.ResetThenCacheRoomEntryProperties();
        self.ram[MUSIC_CONTROL] = self.ram[QUEUED_MUSIC_CONTROL];
        self.ram[LAST_MUSIC_CONTROL] = self.ram[CURRENT_MUSIC_CONTROL];
    }

    pub(super) fn Module11_02_LoadEntrance(&mut self) {
        self.EnableForceBlank();
        self.ram[CGWSEL_COPY] = 2;
        self.Dungeon_LoadEntrance();

        let dung = self.ram[CUR_PALACE_INDEX_X2];
        self.ram[LINK_NUM_KEYS] = if dung != 0xff {
            let idx = if dung == 2 { 0 } else { dung } >> 1;
            self.ram[LINK_KEYS_EARNED_PER_DUNGEON + idx as usize]
        } else {
            0xff
        };
        self.hud_rebuild();
        self.ram[PLAYER_PIT_DATA_INDEX] = 4;
        self.ram[PLAYER_NEAR_PIT_STATE] = 3;
        self.ram[LINK_VISIBILITY_STATUS] = 12;
        self.ram[LINK_SPEED_MODIFIER] = 16;

        let y = self.ram[LINK_Y_COORD].wrapping_sub(self.ram[BG2VOFS_COPY2]);
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[Y_BUTTON_ACTION_TIMER] = 0;
        copy_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX_PREV, DUNGEON_ROOM_INDEX);
        copy_le_u16(&mut self.ram, TILEDETECT_WHICH_Y_POS, LINK_Y_COORD);
        let new_y = self
            .player_state_view()
            .y()
            .wrapping_sub(u16::from(y).wrapping_add(16));
        self.player_state_view_mut().set_y(new_y);

        let bak = self.frame_control_view().subsubmodule();
        self.ram[DUNG_NUM_LIT_TORCHES] = 0;
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 0;
        self.Dungeon_LoadAndDrawRoom();
        self.Dungeon_LoadCustomTileAttr();
        let animated = DUNG_ANIMATED_TILES[self.ram[MAIN_TILE_THEME_INDEX] as usize];
        self.decompress_animated_dungeon_tiles(animated as usize);
        self.Dungeon_LoadAttributeTable();
        self.frame_control_view_mut()
            .set_subsubmodule(bak.wrapping_add(1));
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = 10;
        self.initialize_tilesets();
        self.ram[PALETTE_SP6R_INDOORS] = 10;
        self.dungeon_load_palettes();
        self.hud_restore_torch_background();
        self.ram[BUTTON_MASK_B_Y] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.Dungeon_ResetTorchBackgroundAndPlayer();
        if self.ram[LINK_IS_BUNNY_MIRROR] != 0 {
            self.LoadGearPalettes_bunny();
        }
        self.ram[HDMAEN_COPY] = 0x80;
        self.hud_refill_logic();
        self.module_pre_dungeon_set_ambient_sfx();
        self.frame_control_view_mut().set_submodule(7);
        self.Dungeon_LoadSongBankIfNeeded();
    }

    pub(super) fn Dungeon_InterRoomTrans_State10(&mut self) {
        if self.ram[DUNG_WANT_LIGHTS_OUT] | self.ram[DUNG_WANT_LIGHTS_OUT_COPY] != 0 {
            self.ApplyPaletteFilter_bounce();
        }
        self.Dungeon_InterRoomTrans_notDarkRoom();
    }

    pub(super) fn Dungeon_SpiralStaircase11(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Dungeon_InterRoomTrans_notDarkRoom(&mut self) {
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Dungeon_InterRoomTrans_State9(&mut self) {
        if self.ram[DUNG_WANT_LIGHTS_OUT] | self.ram[DUNG_WANT_LIGHTS_OUT_COPY] != 0 {
            self.ApplyPaletteFilter_bounce();
        }
        self.Dungeon_InterRoomTrans_State4();
    }

    pub(super) fn Dungeon_SpiralStaircase12(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.Dungeon_PrepareNextRoomQuadrantUpload();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Dungeon_InterRoomTrans_State4(&mut self) {
        self.Dungeon_PrepareNextRoomQuadrantUpload();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Dungeon_InterRoomTrans_State12(&mut self) {
        if self.frame_control_view().submodule() == 2 {
            if self.ram[OVERWORLD_MAP_STATE] != 5 {
                return;
            }
            self.SubtileTransitionCalculateLanding();
            if self.ram[DUNG_WANT_LIGHTS_OUT] | self.ram[DUNG_WANT_LIGHTS_OUT_COPY] != 0 {
                self.ApplyPaletteFilter_bounce();
            }
        }
        self.frame_control_view_mut().increment_subsubmodule();
        self.Dungeon_ResetTorchBackgroundAndPlayer();
    }

    pub(super) fn Dungeon_Staircase14(&mut self) {
        self.frame_control_view_mut().increment_subsubmodule();
        self.Dungeon_ResetTorchBackgroundAndPlayer();
    }

    pub(super) fn Dungeon_InterRoomTrans_State7(&mut self) {
        self.MirrorBg1Bg2Offs();
        if self.world_state_view().dungeon_room() != 54
            && self.world_state_view().dungeon_room() != 56
        {
            let y = if K_SPIRAL_TAB1[self.ram[DUNG_HDR_BG2_PROPERTIES] as usize] != 0 {
                0x0116
            } else {
                0x0016
            };
            let tm_ts = self.ram[TM_COPY] as u16 | ((self.ram[TS_COPY] as u16) << 8);
            if y != tm_ts
                && (self.ram[TM_COPY] == 0x17 || (self.ram[TM_COPY] | self.ram[TS_COPY]) != 0x17)
            {
                self.ram[TM_COPY] = y as u8;
                self.ram[TS_COPY] = (y >> 8) as u8;
            }
        }
        self.DungeonTransition_RunFiltering();
    }

    pub(super) fn DungeonTransition_FindSubtileLanding(&mut self) {
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        self.SubtileTransitionCalculateLanding();
        self.frame_control_view_mut().increment_subsubmodule();
        let room = self.world_state_view().dungeon_room() as usize;
        let offset = SAVE_DUNG_INFO + room * 2;
        let saved = read_le_u16(&self.ram, offset) | read_le_u16(&self.ram, DUNG_QUADRANTS_VISITED);
        write_le_u16(&mut self.ram, offset, saved);
    }

    pub(super) fn SubtileTransitionCalculateLanding(&mut self) {
        let st = self.ram[OVERWORLD_SCREEN_TRANSITION];
        let mut a = self.CalculateTransitionLanding();
        if a == 2 {
            a = 1;
        } else if a == 4 {
            a = 2;
        }
        let index = a as usize + self.ram[OVERWORLD_SCREEN_TRANSITION] as usize * 5;
        let mut v = K_STAIRCASE_TAB2[index];
        if v < 0 {
            v = v.wrapping_add(8);
        } else {
            v = v.wrapping_sub(8);
        }
        if st & 2 != 0 {
            self.ram[LINK_X_COORD] = v as u8;
        } else {
            self.ram[LINK_Y_COORD] = v as u8;
        }
        self.ram[LINK_VISIBILITY_STATUS] = 0;
    }

    pub(super) fn Dungeon_IntraRoomTrans_State5(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        if !self.DungeonTransition_MoveLinkOutDoor() {
            return;
        }
        if self.ram[DUNG_TRANSITION_LANDING_CLASS] == 2
            || self.ram[DUNG_TRANSITION_LANDING_CLASS] == 4
        {
            self.ram[IS_STANDING_IN_DOORWAY] = 0;
        }
        self.ram[FORCE_MOVE_ANY_DIRECTION] = 0;
        self.ram[DUNG_TRANSITION_LANDING_CLASS] = 0;
        self.ram[OVERWORLD_SCREEN_TRANSITION] = 0;
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn DungeonTransition_MoveLinkOutDoor(&mut self) -> bool {
        let index = self.ram[DUNG_TRANSITION_LANDING_CLASS] as usize
            + self.ram[OVERWORLD_SCREEN_TRANSITION] as usize * 5;
        let target = K_STAIRCASE_TAB2[index] as u8;
        let step = if self.ram[OVERWORLD_SCREEN_TRANSITION] & 1 != 0 {
            (-2i16) as u16
        } else {
            2
        };
        if self.ram[OVERWORLD_SCREEN_TRANSITION] & 2 == 0 {
            let y = self.player_state_view().y().wrapping_add(step);
            self.player_state_view_mut().set_y(y);
            (y as u8 & 0xfe) == target
        } else {
            let x = self.player_state_view().x().wrapping_add(step);
            self.player_state_view_mut().set_x(x);
            (x as u8 & 0xfe) == target
        }
    }

    pub(super) fn DungeonTransition_Subtile_PrepTransition(&mut self) {
        write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, 0);
        write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
        self.ram[MOSAIC_TARGET_LEVEL] = 31;
        write_le_u16(&mut self.ram, UNUSED_CONFIG_GFX, 0);
        self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH] = 0;
        self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = 0;
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn DungeonTransition_Subtile_ApplyFilter(&mut self) {
        if self.ram[DUNG_WANT_LIGHTS_OUT] == 0 {
            self.frame_control_view_mut().increment_subsubmodule();
            return;
        }
        self.ApplyPaletteFilter_bounce();
        if self.ram[PALETTE_FILTER_COUNTDOWN] != 0 {
            self.ApplyPaletteFilter_bounce();
        }
    }

    pub(super) fn DungeonTransition_Subtile_ResetShutters(&mut self) {
        self.ram[DUNG_FLAG_TRAPDOORS_DOWN] = 0;
        self.ram[DOOR_ANIMATION_STEP_INDICATOR_DUNGEON] = 7;
        let bak = self.frame_control_view().submodule();
        self.OperateShutterDoors();
        self.frame_control_view_mut().set_submodule(bak);
        self.ram[PALETTE_FILTER_COUNTDOWN] = 31;
        self.ram[MOSAIC_TARGET_LEVEL] = 0;
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn DungeonTransition_Subtile_TriggerShutters(&mut self) {
        self.ResetThenCacheRoomEntryProperties();
        if self.ram[DUNG_FLAG_TRAPDOORS_DOWN] == 0 {
            self.ram[DUNG_FLAG_TRAPDOORS_DOWN] = 1;
            self.ram[DUNG_CUR_DOOR_POS_DUNGEON] = 0;
            self.ram[DOOR_ANIMATION_STEP_INDICATOR_DUNGEON] = 0;
            self.frame_control_view_mut().set_submodule(5);
        }
    }

    pub(super) fn DungeonTransition_RunFiltering(&mut self) {
        if self.ram[DUNG_WANT_LIGHTS_OUT] | self.ram[DUNG_WANT_LIGHTS_OUT_COPY] != 0 {
            const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
            let torch = if self.ram[DUNG_WANT_LIGHTS_OUT] != 0 {
                self.ram[DUNG_NUM_LIT_TORCHES] as usize
            } else {
                3
            };
            self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = LIT_TORCHES_COLOR_PLUS[torch];
            self.Dungeon_ApproachFixedColor_variable(self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS]);
            self.ram[MOSAIC_TARGET_LEVEL] = 0;
        }
        self.Dungeon_HandleTranslucencyAndPalette();
    }

    pub(super) fn Module07_02_FadedFilter(&mut self) {
        if self.ram[DUNG_WANT_LIGHTS_OUT] | self.ram[DUNG_WANT_LIGHTS_OUT_COPY] != 0 {
            self.ApplyPaletteFilter_bounce();
            if self.ram[PALETTE_FILTER_COUNTDOWN] != 0 {
                self.ApplyPaletteFilter_bounce();
            }
        } else {
            self.frame_control_view_mut().increment_subsubmodule();
        }
    }

    pub(super) fn Dungeon_InterRoomTrans_State15(&mut self) {
        self.ResetThenCacheRoomEntryProperties();
        if self.ram[DUNG_FLAG_TRAPDOORS_DOWN] == 0
            && (self.ram[DUNGEON_ROOM_INDEX] != 172
                || read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x3000 != 0)
        {
            self.ram[DUNG_FLAG_TRAPDOORS_DOWN] = 1;
            self.ram[DUNG_CUR_DOOR_POS_DUNGEON] = 0;
            self.ram[DOOR_ANIMATION_STEP_INDICATOR_DUNGEON] = 0;
            self.frame_control_view_mut().set_submodule(5);
        }
        self.Dungeon_PlayMusicIfDefeated();
    }

    pub(super) fn DungeonTransition_LoadSpriteGFX(&mut self) {
        self.LoadNewSpriteGFXSet();
        self.dungeon_reset_sprites();
        self.DungeonTransition_RunFiltering();
    }

    pub(super) fn Module07_04_UnlockDoor(&mut self) {
        self.Dungeon_OpeningLockedDoor_Combined(false);
    }

    pub(super) fn Module07_03_OverlayChange(&mut self) {
        let overlay_index = self.ram[DUNG_OVERLAY_TO_LOAD] as usize;
        let overlay_offs = self.asset_u16(49, overlay_index) as usize;
        let overlay_data = self
            .asset_raw(48)
            .expect("missing dungeon room overlay asset")
            .to_vec();
        let overlay = &overlay_data[overlay_offs..];
        self.Dungeon_DrawRoomOverlay(overlay);
        let mut dst_pos = 0usize;
        let mut offset = 0usize;
        loop {
            let marker = overlay[offset] as u16 | ((overlay[offset + 1] as u16) << 8);
            if marker == 0xffff {
                break;
            }
            let p =
                ((overlay[offset] as u16 >> 2) | ((overlay[offset + 1] as u16 >> 2) << 6)) as usize;
            dst_pos = self.dungeon_prep_overlay_dma_next_prep(dst_pos, (p * 2) as u16);
            self.Dungeon_DrawRoomOverlay_Apply(p);
            offset += 3;
        }
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;
        self.frame_control_view_mut().set_submodule(0);
    }

    pub(super) fn Module07_05_ControlShutters(&mut self) {
        self.OperateShutterDoors();
    }

    pub(super) fn Module07_06_FatInterRoomStairs(&mut self) {
        if self.frame_control_view().subsubmodule() >= 3 {
            self.Dungeon_LoadAttribute_Selectable();
        }

        if self.frame_control_view().subsubmodule() >= 13 {
            self.Graphics_IncrementalVRAMUpload();
            if self.ram[STAIRCASE_MOVE_COUNTER] == 0 {
                self.Module07_06_FatInterRoomStairs_dispatch();
                return;
            }
            if self.ram[STAIRCASE_MOVE_COUNTER] == 0x10 {
                self.ram[LINK_SPEED_MODIFIER] = 2;
            }
            self.ram[STAIRCASE_MOVE_COUNTER] = self.ram[STAIRCASE_MOVE_COUNTER].wrapping_sub(1);
            self.ram[LINK_DIRECTION] = if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
                4
            } else {
                8
            };
            self.link_handle_velocity();
            self.dungeon_handle_camera();
        }

        self.link_handle_moving_animation_full_long_entry();
        self.Module07_06_FatInterRoomStairs_dispatch();
    }

    fn Module07_06_FatInterRoomStairs_dispatch(&mut self) {
        match self.frame_control_view().subsubmodule() {
            0 => self.ResetTransitionPropsAndAdvance_ResetInterface(),
            1 => {
                self.ApplyPaletteFilter_bounce();
                if self.ram[PALETTE_FILTER_COUNTDOWN] != 0 {
                    self.ApplyPaletteFilter_bounce();
                }
            }
            2 => self.Dungeon_InitializeRoomFromSpecial(),
            3 => self.DungeonTransition_TriggerBGC34UpdateAndAdvance(),
            4 => self.DungeonTransition_TriggerBGC56UpdateAndAdvance(),
            5 => self.DungeonTransition_LoadSpriteGFX(),
            6 => self.DungeonTransition_AdjustForFatStairScroll(),
            7 => self.Dungeon_InterRoomTrans_State4(),
            8 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            9 => self.Dungeon_InterRoomTrans_State4(),
            10 => self.Dungeon_SpiralStaircase11(),
            11 => self.Dungeon_SpiralStaircase12(),
            12 => self.Dungeon_SpiralStaircase11(),
            13 => self.Dungeon_SpiralStaircase12(),
            14 => self.Dungeon_DoubleApplyAndIncrementGrayscale(),
            15 => self.Dungeon_Staircase14(),
            16 => {
                if (self.ram[DARKENING_OR_LIGHTENING_SCREEN] | self.ram[PALETTE_FILTER_COUNTDOWN])
                    == 0
                    && self.ram[OVERWORLD_MAP_STATE] == 5
                {
                    self.ResetThenCacheRoomEntryProperties();
                }
            }
            _ => panic!("invalid fat inter-room stair index"),
        }
    }

    pub(super) fn Dungeon_InitializeRoomFromSpecial(&mut self) {
        self.Dungeon_AdjustAfterSpiralStairs();
        self.Dungeon_LoadRoom();
        self.ResetStarTileGraphics();
        self.LoadTransAuxGFX();
        self.Dungeon_LoadCustomTileAttr();
        self.ram[DUNGEON_ROOM_INDEX2] = self.ram[DUNGEON_ROOM_INDEX];
        self.follower_initialize();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn DungeonTransition_AdjustForFatStairScroll(&mut self) {
        self.MirrorBg1Bg2Offs();
        self.Dungeon_AdjustForRoomLayout();
        let mut ts = K_SPIRAL_TAB1[self.ram[DUNG_HDR_BG2_PROPERTIES] as usize];
        let mut tm = 0x16;
        if ts < 0 {
            tm = 0x17;
            ts = 0;
        }
        self.ram[TM_COPY] = tm;
        self.ram[TS_COPY] = ts as u8;

        self.ram[LINK_SPEED_MODIFIER] = 1;
        if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_sub(1);
            self.ram[STAIRCASE_MOVE_COUNTER] = 32;
            self.ram[SOUND_EFFECT_1] = 0x19;
        } else {
            self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_add(1);
            self.ram[STAIRCASE_MOVE_COUNTER] = 48;
            self.ram[SOUND_EFFECT_1] = 0x17;
        }
        self.ram[SOUND_EFFECT_2] = 0x24;
        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
        self.Dungeon_InterRoomTrans_notDarkRoom();
    }

    pub(super) fn Module07_16_UpdatePegs(&mut self) {
        self.frame_control_view_mut().increment_subsubmodule();
        if self.frame_control_view().subsubmodule() & 3 != 0 {
            return;
        }
        match self.frame_control_view().subsubmodule() >> 2 {
            0 | 1 => self.Module07_16_UpdatePegs_Step1(),
            2 => self.Module07_16_UpdatePegs_Step2(),
            3 => self.RecoverPegGFXFromMapping(),
            4 => {
                self.Dungeon_FlipCrystalPegAttribute();
                self.frame_control_view_mut().set_subsubmodule(0);
                self.frame_control_view_mut().set_submodule(0);
            }
            _ => {}
        }
    }

    pub(super) fn Module07_17_PressurePlate(&mut self) {
        self.frame_control_view_mut().decrement_subsubmodule();
        if self.frame_control_view().subsubmodule() != 0 {
            return;
        }
        let link_y = self.player_state_view().y().wrapping_sub(2);
        self.player_state_view_mut().set_y(link_y);
        let pos = read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_DST_POS_X2);
        self.Dungeon_UpdateTileMapWithCommonTile(
            i32::from((pos & 0x003f) << 3),
            i32::from((pos >> 3) & 0x01f8),
            0x0e,
        );
        let saved_module = self.ram[SAVED_MODULE_FOR_MENU];
        self.frame_control_view_mut().set_submodule(saved_module);
    }

    pub(super) fn Module07_18_RescuedMaiden(&mut self) {
        match self.frame_control_view().subsubmodule() {
            0 => {
                self.PaletteFilter_RestoreBGSubstractiveStrict();
                let c = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + 32 * 2);
                write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, c);
                if self.ram[DARKENING_OR_LIGHTENING_SCREEN] != 0xff {
                    return;
                }
                for i in 0..0x1000usize {
                    write_le_u16(&mut self.ram, DUNG_BG2 + i * 2, 0x01ec);
                    write_le_u16(&mut self.ram, DUNG_BG1 + i * 2, 0x01ec);
                }
                write_le_u16(&mut self.ram, BG1_Y_OFFSET, 0);
                write_le_u16(&mut self.ram, BG1_X_OFFSET, 0);
                write_le_u16(&mut self.ram, DUNG_FLOOR_X_OFFS, 0);
                write_le_u16(&mut self.ram, DUNG_FLOOR_Y_OFFS, 0);
                self.ram[OVERWORLD_SCREEN_TRANSITION] = 0;
                self.ram[DUNG_CUR_QUADRANT_UPLOAD] = 0;
                self.frame_control_view_mut().increment_subsubmodule();
            }
            1 => {
                const CRYSTAL_TAB0: [u16; 7] =
                    [0x1618, 0x1658, 0x1658, 0x1618, 0x0658, 0x1618, 0x1658];
                self.PaletteFilter_Crystal();
                self.ram[TS_COPY] = 1;
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 2;
                let room = self.world_state_view().dungeon_room();
                let j = K_BOSS_ROOMS_DUNGEON
                    .iter()
                    .rposition(|&r| r == room)
                    .expect("rescued maiden room must be a boss room")
                    .checked_sub(4)
                    .expect("rescued maiden boss room index must select a crystal slot");
                let mut dsto = CRYSTAL_TAB0[j] >> 1;
                let mut tile = 0u16;
                for _ in 0..4 {
                    for x in 0..8u16 {
                        self.room_write_bg(0x4000, dsto + x, 0x1f80 | tile);
                        self.room_write_bg(0x4000, dsto + x + xy(0, 4) as u16, 0x1f88 | tile);
                        tile = tile.wrapping_add(1);
                    }
                    tile = tile.wrapping_add(8);
                    dsto = dsto.wrapping_add(xy(0, 1) as u16);
                }
                self.frame_control_view_mut().increment_subsubmodule();
            }
            2 | 4 | 6 | 8 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            3 | 5 | 7 | 9 => self.Dungeon_InterRoomTrans_State4(),
            10 => {
                self.ram[IS_NMI_THREAD_ACTIVE] = self.ram[IS_NMI_THREAD_ACTIVE].wrapping_add(1);
                self.Polyhedral_InitializeThread();
                self.CrystalCutscene_Initialize();
                self.frame_control_view_mut().set_submodule(0);
                self.frame_control_view_mut().set_subsubmodule(0);
            }
            _ => {}
        }
    }

    pub(super) fn Module07_19_MirrorFade(&mut self) {
        self.Overworld_ResetMosaic_alwaysIncrease();
        self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
        if self.ram[INIDISP_COPY] != 0 {
            return;
        }
        self.frame_control_view_mut().set_main_module(5);
        self.frame_control_view_mut().set_submodule(0);
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 0;
        self.ram[LAST_MUSIC_CONTROL] = self.ram[CURRENT_MUSIC_CONTROL];
        if self.ram[PALETTE_SWAP_FLAG] != 0 {
            self.Palette_RevertTranslucencySwap();
        }
    }

    pub(super) fn Module07_1A_RoomDraw_OpenTriforceDoor_bounce(&mut self) {
        const OPEN_GANON_DOOR_TAB: [u16; 4] = [0x2556, 0x2596, 0x25d6, 0x2616];

        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
        if read_le_u16(&self.ram, R16) != 0 {
            self.ram[R16] = self.ram[R16].wrapping_sub(1);
            if self.ram[R16] != 0 {
                return;
            }
            self.ram[R16 + 1] = self.ram[R16 + 1].wrapping_sub(1);
            if self.ram[R16 + 1] != 0 {
                return;
            }
            self.ram[SOUND_EFFECT_AMBIENT] = 21;
            self.ram[LINK_FORCE_HOLD_SWORD_UP] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        }
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
        self.frame_control_view_mut().increment_subsubmodule();
        if self.frame_control_view().subsubmodule() & 3 != 0 {
            return;
        }

        let index = self.frame_control_view().subsubmodule().wrapping_sub(4) as usize >> 2;
        let src = OPEN_GANON_DOOR_TAB[index] as usize;
        for i in 0..8u16 {
            self.room_write_bg(
                0x2000,
                xy(44, 3) as u16 + i,
                self.tile_word(src, (i * 4) as usize),
            );
            self.room_write_bg(
                0x2000,
                xy(44, 4) as u16 + i,
                self.tile_word(src, (i * 4 + 1) as usize),
            );
            self.room_write_bg(
                0x2000,
                xy(44, 5) as u16 + i,
                self.tile_word(src, (i * 4 + 2) as usize),
            );
            self.room_write_bg(
                0x2000,
                xy(44, 6) as u16 + i,
                self.tile_word(src, (i * 4 + 3) as usize),
            );
        }

        self.dungeon_prep_overlay_dma_watergate(0, 0x01d8, 0x0881, 8);
        if self.frame_control_view().subsubmodule() == 16 {
            self.write_attr2(xy(44, 5), 0x0202);
            self.write_attr2(xy(44, 6), 0x0202);
            self.write_attr2(xy(50, 5), 0x0200);
            self.write_attr2(xy(50, 6), 0x0200);
            for i in (0..6).step_by(2) {
                for y in 0..7 {
                    self.write_attr2(xy(45 + i, y), 0);
                }
            }
            write_le_u16(&mut self.ram, ROOM_BOUNDS_Y, (-64i16) as u16);
            self.frame_control_view_mut().set_submodule(0);
            self.frame_control_view_mut().set_subsubmodule(0);
        }
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;
    }

    pub(super) fn DungeonTransition_ScrollRoom(&mut self) {
        self.ram[TRANSITION_COUNTER] = self.ram[TRANSITION_COUNTER].wrapping_add(1);
        let i = self.ram[OVERWORLD_SCREEN_TRANSITION] as usize;
        write_le_u16(&mut self.ram, BG1_Y_OFFSET, 0);
        write_le_u16(&mut self.ram, BG1_X_OFFSET, 0);
        let delta = K_STAIRCASE_TAB3[i] as i16 as u16;

        let t = if i >= 2 {
            let t = read_le_u16(&self.ram, BG2HOFS_COPY2).wrapping_add(delta) & !1;
            write_le_u16(&mut self.ram, BG2HOFS_COPY2, t);
            write_le_u16(&mut self.ram, BG1HOFS_COPY2, t);
            if self.ram[TRANSITION_COUNTER] >= K_STAIRCASE_TAB4[i] {
                let x = self.player_state_view().x().wrapping_add(delta);
                self.player_state_view_mut().set_x(x);
            }
            t
        } else {
            let t = read_le_u16(&self.ram, BG2VOFS_COPY2).wrapping_add(delta) & !1;
            write_le_u16(&mut self.ram, BG2VOFS_COPY2, t);
            write_le_u16(&mut self.ram, BG1VOFS_COPY2, t);
            if self.ram[TRANSITION_COUNTER] >= K_STAIRCASE_TAB4[i] {
                let y = self.player_state_view().y().wrapping_add(delta);
                self.player_state_view_mut().set_y(y);
            }
            t
        };

        if (t & 0x01fc) == read_le_u16(&self.ram, UP_DOWN_SCROLL_TARGET + i * 2) {
            self.SetAndSaveVisitedQuadrantFlags();
            self.frame_control_view_mut().increment_subsubmodule();
            self.ram[TRANSITION_COUNTER] = 0;
            if self.frame_control_view().submodule() == 2 {
                self.WaterFlood_BuildOneQuadrantForVRAM();
            }
        }
    }

    pub(super) fn DungeonTransition_TriggerBGC34UpdateAndAdvance(&mut self) {
        self.PrepTransAuxGfx();
        self.ram[NMI_SUBROUTINE_INDEX] = 9;
        self.ram[NMI_DISABLE_CORE_UPDATES] = 9;
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn DungeonTransition_TriggerBGC56UpdateAndAdvance(&mut self) {
        self.ram[NMI_SUBROUTINE_INDEX] = 10;
        self.ram[NMI_DISABLE_CORE_UPDATES] = 10;
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Dungeon_TryScreenEdgeTransition(&mut self) {
        let mut dir = None;

        if self.ram[LINK_Y_VEL] != 0 {
            let y = self.player_state_view().y() & 0x01ff;
            if y < 4 {
                dir = Some(3);
            } else if y >= 476 {
                dir = Some(2);
            }
        }

        if dir.is_none() && self.ram[LINK_X_VEL] != 0 {
            let x = self.player_state_view().x() & 0x01ff;
            if x < 8 {
                dir = Some(1);
            } else if x >= 489 {
                dir = Some(0);
            }
        }

        let Some(dir) = dir else {
            return;
        };

        if !self.link_check_for_edge_screen_transition()
            && self.frame_control_view().main_module() == 7
        {
            self.Dungeon_HandleEdgeTransitionMovement(dir);
            if self.frame_control_view().main_module() == 7 {
                self.frame_control_view_mut().set_submodule(2);
            }
        }
    }

    pub(super) fn Dungeon_HandleEdgeTransitionMovement(&mut self, dir: u8) {
        const LIMIT_DIRECTION_ON_ONE_AXIS: [u8; 4] = [0x03, 0x03, 0x0c, 0x0c];
        self.ram[LINK_DIRECTION] &= LIMIT_DIRECTION_ON_ONE_AXIS[dir as usize];
        match dir {
            0 => self.Dungeon_StartInterRoomTrans_Right(),
            1 => self.Dungeon_StartInterRoomTrans_Left(),
            2 => self.Dungeon_StartInterRoomTrans_Down(),
            3 => self.Dungeon_StartInterRoomTrans_Up(),
            _ => unreachable!(),
        }
    }

    pub(super) fn Dungeon_AdjustAfterSpiralStairs(&mut self) {
        let room = self.world_state_view().dungeon_room();
        let prev = read_le_u16(&self.ram, DUNGEON_ROOM_INDEX_PREV);
        let xd = ((room & 0x000f) as i32 - (prev & 0x000f) as i32) * 0x200;
        self.add_dungeon_room_delta_x(xd as i16 as u16);

        let yd = (((room & 0x00f0) >> 4) as i32 - ((prev & 0x00f0) >> 4) as i32) * 0x200;
        self.add_dungeon_room_delta_y(yd as i16 as u16);
    }

    pub(super) fn Dungeon_AdjustForTeleportDoors(&mut self, room: u8, flag: u8) {
        write_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX2, room as u16);
        write_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX_PREV, room as u16);

        let link_x_hi = self.player_state_view().x() >> 8;
        let xx = ((room & 0x0f) as u16)
            .wrapping_mul(2)
            .wrapping_sub(link_x_hi)
            .wrapping_add(flag as u16);
        self.add_dungeon_room_delta_x(xx << 8);

        let link_y_hi = self.player_state_view().y() >> 8;
        let yy = (((room & 0xf0) >> 3) as u16).wrapping_sub(link_y_hi);
        self.add_dungeon_room_delta_y(yy << 8);

        let y_hi = (self.player_state_view().y() >> 8) as u8;
        for i in 0..20 {
            self.ram[TAGALONG_Y_HI + i] = y_hi;
        }
    }

    fn add_dungeon_room_delta_x(&mut self, delta: u16) {
        for addr in [
            LINK_X_COORD,
            BG2HOFS_COPY2,
            ROOM_BOUNDS_X,
            ROOM_BOUNDS_X + 2,
            ROOM_BOUNDS_X + 4,
            ROOM_BOUNDS_X + 6,
        ] {
            let value = read_le_u16(&self.ram, addr).wrapping_add(delta);
            write_le_u16(&mut self.ram, addr, value);
        }
    }

    fn add_dungeon_room_delta_y(&mut self, delta: u16) {
        for addr in [
            LINK_Y_COORD,
            BG2VOFS_COPY2,
            ROOM_BOUNDS_Y,
            ROOM_BOUNDS_Y + 2,
            ROOM_BOUNDS_Y + 4,
            ROOM_BOUNDS_Y + 6,
        ] {
            let value = read_le_u16(&self.ram, addr).wrapping_add(delta);
            write_le_u16(&mut self.ram, addr, value);
        }
    }

    pub(super) fn Ganon_ExtinguishTorch_adjust_translucency(&mut self) {
        self.Palette_AssertTranslucencySwap();
        self.ram[DUNGEON_TORCH_ATTR] = 0xc0;
        self.Dungeon_ExtinguishTorch();
    }

    pub(super) fn Ganon_ExtinguishTorch(&mut self) {
        self.ram[DUNGEON_TORCH_ATTR] = 193;
        self.Dungeon_ExtinguishTorch();
    }

    pub(super) fn Dungeon_ExtinguishTorch(&mut self) {
        let y = ((self.ram[DUNGEON_TORCH_ATTR] & 0x0f) as usize) * 2
            + read_le_u16(&self.ram, DUNG_INDEX_OF_TORCHES_START) as usize;
        let idx = y >> 1;
        let mut r8 = read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + idx * 2) & 0x7fff;
        write_le_u16(&mut self.ram, DUNG_OBJECT_TILEMAP_POS + idx * 2, r8);

        let obj_pos = (read_le_u16(&self.ram, DUNG_OBJECT_POS_IN_OBJDATA + idx * 2) & 0x00ff) >> 1;
        write_le_u16(
            &mut self.ram,
            DUNG_TORCH_DATA_DUNGEON + obj_pos as usize * 2,
            r8,
        );

        r8 &= 0x3fff;
        self.room_draw_adjust_torch_lighting_change(r8, 0x0ec2, r8);
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;

        if self.ram[DUNG_WANT_LIGHTS_OUT] != 0 && self.ram[DUNG_NUM_LIT_TORCHES] != 0 {
            self.ram[DUNG_NUM_LIT_TORCHES] = self.ram[DUNG_NUM_LIT_TORCHES].wrapping_sub(1);
            if self.ram[DUNG_NUM_LIT_TORCHES] < 3 {
                if self.ram[DUNG_NUM_LIT_TORCHES] == 0 {
                    self.ram[TS_COPY] = 1;
                }
                const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
                self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] =
                    LIT_TORCHES_COLOR_PLUS[self.ram[DUNG_NUM_LIT_TORCHES] as usize];
                self.frame_control_view_mut().set_submodule(10);
                self.frame_control_view_mut().set_subsubmodule(0);
            }
        }

        let torch_timer = (self.ram[DUNGEON_TORCH_ATTR] & 0x0f) as usize;
        self.ram[DUNG_TORCH_TIMERS_DUNGEON + torch_timer] = 0;
        self.ram[DUNGEON_TORCH_ATTR] = 0;
    }

    fn set_spiral_stair_wall_priority(&mut self, pos: u16, high: bool) {
        let mask = if high { 0x2000 } else { 0xdfff };
        for i in 0..5usize {
            for y in 0..4usize {
                let addr = DUNG_BG2 + (pos as usize + i + y * 64) * 2;
                let value = read_le_u16(&self.ram, addr);
                let value = if high { value | mask } else { value & mask };
                write_le_u16(&mut self.ram, addr, value);
            }
        }
    }

    pub(super) fn SpiralStairs_MakeNearbyWallsHighPriority_Entering(&mut self) {
        let index = (self.ram[WHICH_STAIRCASE_INDEX] & 3) as usize;
        let pos = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + index * 2).wrapping_sub(4);
        write_le_u16(&mut self.ram, STAIRCASE_TILEMAP_POS_X2, pos.wrapping_mul(2));
        self.set_spiral_stair_wall_priority(pos, true);
        let dma_ptr = self.dungeon_prep_overlay_dma_next_prep(0, pos.wrapping_mul(2));
        self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos.wrapping_mul(2).wrapping_add(8));
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;
    }

    pub(super) fn SpiralStairs_MakeNearbyWallsLowPriority(&mut self) {
        let pos = read_le_u16(&self.ram, STAIRCASE_TILEMAP_POS_X2) >> 1;
        self.set_spiral_stair_wall_priority(pos, false);
        let dma_ptr = self.dungeon_prep_overlay_dma_next_prep(0, pos.wrapping_mul(2));
        self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos.wrapping_mul(2).wrapping_add(8));
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;
    }

    pub(super) fn SpiralStairs_MakeNearbyWallsHighPriority_Exiting(&mut self) {
        if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            return;
        }
        let lf = read_le_u16(&self.ram, STAIRCASE_TILEMAP_POS_X2).wrapping_add(8) & 0x007f;
        let mut x = 0usize;
        let mut p;
        loop {
            p = read_le_u16(&self.ram, DUNG_INTER_STAIRCASES + x * 2);
            if (p.wrapping_mul(2) & 0x007f) == lf {
                break;
            }
            x += 1;
        }
        p = p.wrapping_sub(4);
        write_le_u16(&mut self.ram, STAIRCASE_TILEMAP_POS_X2, p.wrapping_mul(2));
        self.set_spiral_stair_wall_priority(p, true);
    }

    pub(super) fn Module07_0F_00_InitSpotlight(&mut self) {
        self.Spotlight_open();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Module07_0F_01_OperateSpotlight(&mut self) {
        self.sprite_main();
        self.IrisSpotlight_ConfigureTable();
        if self.frame_control_view().submodule() == 0 {
            self.ram[W12SEL_COPY] = 0;
            self.ram[W34SEL_COPY] = 0;
            self.ram[WOBJSEL_COPY] = 0;
            self.ram[TMW_COPY] = 0;
            self.ram[TSW_COPY] = 0;
            self.frame_control_view_mut().set_subsubmodule(0);
            if self.ram[QUEUED_MUSIC_CONTROL] != 0xff {
                self.ram[MUSIC_CONTROL] = self.ram[QUEUED_MUSIC_CONTROL];
            }
        }
    }

    pub(super) fn Module07_0F_LandingWipe(&mut self) {
        match self.frame_control_view().subsubmodule() {
            0 => self.Module07_0F_00_InitSpotlight(),
            1 => self.Module07_0F_01_OperateSpotlight(),
            other => panic!("invalid Module07_0F_LandingWipe subsubmodule_index {other}"),
        }
        self.link_handle_moving_animation_full_long_entry();
        self.link_oam_main();
    }

    pub(super) fn Module07_10_SouthIntraRoomStairs(&mut self) {
        let t = self.ram[STAIRCASE_MOVE_COUNTER];
        if t != 0 {
            self.ram[STAIRCASE_MOVE_COUNTER] = self.ram[STAIRCASE_MOVE_COUNTER].wrapping_sub(1);
            if t == 20 {
                self.ram[LINK_SPEED_MODIFIER] = 2;
            }
            self.link_handle_velocity();
            self.apply_links_movement_to_camera();
            self.dungeon_handle_camera();
            self.link_handle_moving_animation_full_long_entry();
        }
        match self.frame_control_view().subsubmodule() {
            0 => self.Module07_10_00_InitStairs(),
            1 => self.Module07_10_01_ClimbStairs(),
            other => panic!("invalid Module07_10_SouthIntraRoomStairs subsubmodule_index {other}"),
        }
    }

    pub(super) fn Module07_09_OpenCrackedDoor(&mut self) {
        self.OpenCrackedDoor();
    }

    pub(super) fn Module07_10_00_InitStairs(&mut self) {
        let mut v1 = 0x3c;
        let mut sfx = 25;
        if self.ram[LINK_DIRECTION] & 4 != 0 {
            v1 = 0x38;
            sfx = 23;
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] ^= 1;
            if read_le_u16(&self.ram, KIND_OF_IN_ROOM_STAIRCASE) as u8 != 2 {
                self.ram[LINK_IS_ON_LOWER_LEVEL] ^= 1;
            }
        }
        self.ram[STAIRCASE_MOVE_COUNTER] = v1;
        self.ram[SOUND_EFFECT_1] = sfx;
        self.ram[LINK_SPEED_MODIFIER] = 1;
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Module07_10_01_ClimbStairs(&mut self) {
        if self.ram[STAIRCASE_MOVE_COUNTER] != 0 {
            return;
        }
        if self.ram[LINK_DIRECTION] & 8 != 0 {
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] ^= 1;
            if read_le_u16(&self.ram, KIND_OF_IN_ROOM_STAIRCASE) as u8 != 2 {
                self.ram[LINK_IS_ON_LOWER_LEVEL] ^= 1;
            }
        }
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[OVERWORLD_SCREEN_TRANSITION] = 0;
        self.frame_control_view_mut().set_submodule(0);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn Module07_08_NorthIntraRoomStairs(&mut self) {
        let t = self.ram[STAIRCASE_MOVE_COUNTER];
        if t != 0 {
            self.ram[STAIRCASE_MOVE_COUNTER] = self.ram[STAIRCASE_MOVE_COUNTER].wrapping_sub(1);
            if t == 20 {
                self.ram[LINK_SPEED_MODIFIER] = 2;
            }
            self.link_handle_velocity();
            self.apply_links_movement_to_camera();
            self.dungeon_handle_camera();
            self.link_handle_moving_animation_full_long_entry();
        }
        match self.frame_control_view().subsubmodule() {
            0 => self.Module07_08_00_InitStairs(),
            1 => self.Module07_08_01_ClimbStairs(),
            other => panic!("invalid Module07_08_NorthIntraRoomStairs subsubmodule_index {other}"),
        }
    }

    pub(super) fn Module07_08_00_InitStairs(&mut self) {
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        let mut v1 = 0x3c;
        let mut sfx = 25;
        if self.ram[LINK_DIRECTION] & 8 != 0 {
            v1 = 0x38;
            sfx = 23;
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = 0;
            if read_le_u16(&self.ram, KIND_OF_IN_ROOM_STAIRCASE) as u8 != 2 {
                self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
            }
        }
        self.ram[STAIRCASE_MOVE_COUNTER] = v1;
        self.ram[SOUND_EFFECT_1] = sfx;
        self.ram[LINK_SPEED_MODIFIER] = 1;
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Module07_08_01_ClimbStairs(&mut self) {
        if self.ram[STAIRCASE_MOVE_COUNTER] != 0 {
            return;
        }
        if self.ram[LINK_DIRECTION] & 4 != 0 {
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = 1;
            if read_le_u16(&self.ram, KIND_OF_IN_ROOM_STAIRCASE) as u8 != 2 {
                self.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
            }
        }
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[OVERWORLD_SCREEN_TRANSITION] = 0;
        self.frame_control_view_mut().set_submodule(0);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn Module07_11_StraightInterroomStairs(&mut self) {
        if self.frame_control_view().subsubmodule() >= 3 {
            self.Dungeon_LoadAttribute_Selectable();
        }
        if self.frame_control_view().subsubmodule() >= 13 {
            self.Graphics_IncrementalVRAMUpload();
        }
        if self.ram[STAIRCASE_MOVE_COUNTER] != 0 {
            if self.ram[STAIRCASE_MOVE_COUNTER] == 16 {
                self.ram[LINK_SPEED_MODIFIER] = 2;
            }
            self.ram[STAIRCASE_MOVE_COUNTER] = self.ram[STAIRCASE_MOVE_COUNTER].wrapping_sub(1);
            self.ram[LINK_DIRECTION] = if self.frame_control_view().submodule() == 18 {
                8
            } else {
                4
            };
            self.link_handle_velocity();
        }
        self.link_handle_moving_animation_full_long_entry();
        match self.frame_control_view().subsubmodule() {
            0 => self.Module07_11_00_PrepAndReset(),
            1 => self.Module07_11_01_FadeOut(),
            2 => self.Module07_11_02_LoadAndPrepRoom(),
            3 => self.Module07_11_03_FilterAndLoadBGChars(),
            4 => self.Module07_11_04_FilterDoBGAndResetSprites(),
            5 => self.Dungeon_SpiralStaircase11(),
            6 => self.Dungeon_SpiralStaircase12(),
            7 => self.Dungeon_SpiralStaircase11(),
            8 => self.Dungeon_SpiralStaircase12(),
            9 => self.Module07_11_09_LoadSpriteGraphics(),
            10 => self.Module07_11_0A_ScrollCamera(),
            11 => self.Module07_11_0B_PrepDestination(),
            12 => self.Dungeon_InterRoomTrans_State4(),
            13 => self.Dungeon_InterRoomTrans_notDarkRoom(),
            14 => self.Dungeon_InterRoomTrans_State4(),
            15 => self.Dungeon_DoubleApplyAndIncrementGrayscale(),
            16 => self.Module07_11_19_SetSongAndFilter(),
            17 => self.Module07_11_11_KeepSliding(),
            18 => self.ResetThenCacheRoomEntryProperties(),
            other => {
                panic!("invalid Module07_11_StraightInterroomStairs subsubmodule_index {other}")
            }
        }
    }

    pub(super) fn Module07_11_00_PrepAndReset(&mut self) {
        if self.ram[LINK_IS_RUNNING] != 0 {
            self.ram[LINK_IS_RUNNING] = 0;
            self.ram[LINK_SPEED_SETTING] = 2;
        }
        self.ram[SOUND_EFFECT_1] = if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            24
        } else {
            22
        };
        let room = self.world_state_view().dungeon_room();
        if room == 48 || room == 64 {
            self.ram[MUSIC_CONTROL] = 0xf1;
        }
        self.ResetTransitionPropsAndAdvance_ResetInterface();
    }

    pub(super) fn Module07_11_01_FadeOut(&mut self) {
        if self.ram[STAIRCASE_MOVE_COUNTER] < 9 {
            self.ApplyPaletteFilter_bounce();
            if self.ram[PALETTE_FILTER_COUNTDOWN] == 23 {
                self.frame_control_view_mut().increment_subsubmodule();
            }
        }
    }

    pub(super) fn Module07_11_02_LoadAndPrepRoom(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.Dungeon_LoadRoom();
        self.Dungeon_RestoreStarTileChr();
        self.LoadTransAuxGFX();
        self.Dungeon_LoadCustomTileAttr();
        self.Dungeon_AdjustForRoomLayout();
        self.follower_initialize();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Module07_11_03_FilterAndLoadBGChars(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.DungeonTransition_TriggerBGC34UpdateAndAdvance();
    }

    pub(super) fn Module07_11_04_FilterDoBGAndResetSprites(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.DungeonTransition_TriggerBGC56UpdateAndAdvance();
        self.ram[DUNGEON_ROOM_INDEX2] = self.ram[DUNGEON_ROOM_INDEX];
        self.dungeon_reset_sprites();
    }

    pub(super) fn Module07_11_09_LoadSpriteGraphics(&mut self) {
        self.ApplyPaletteFilter_bounce();
        self.frame_control_view_mut().decrement_subsubmodule();
        self.LoadNewSpriteGFXSet();
        self.Dungeon_HandleTranslucencyAndPalette();
    }

    pub(super) fn Module07_11_0A_ScrollCamera(&mut self) {
        self.ram[LINK_VISIBILITY_STATUS] = 12;
        self.ram[TAGALONG_APPEARANCE_NONE_FLAG] = 12;
        let mut i = self.ram[OVERWORLD_SCREEN_TRANSITION] as usize;
        let y = read_le_u16(&self.ram, BG2VOFS_COPY2)
            .wrapping_add(K_STAIRCASE_TAB3[i] as i16 as u16)
            & !3;
        write_le_u16(&mut self.ram, BG1VOFS_COPY2, y);
        write_le_u16(&mut self.ram, BG2VOFS_COPY2, y);
        if (y & 0x01fc) == read_le_u16(&self.ram, UP_DOWN_SCROLL_TARGET + i * 2) {
            if self.frame_control_view().submodule() >= 18 {
                i += 2;
            }
            let link_y = self
                .player_state_view()
                .y()
                .wrapping_add(K_STAIRCASE_TAB5[i] as i16 as u16);
            self.player_state_view_mut().set_y(link_y);
            self.ram[LINK_VISIBILITY_STATUS] = 0;
            self.ram[TAGALONG_APPEARANCE_NONE_FLAG] = 0;
            self.frame_control_view_mut().increment_subsubmodule();
        }
    }

    pub(super) fn Module07_11_0B_PrepDestination(&mut self) {
        let mut ts = K_SPIRAL_TAB1[self.ram[DUNG_HDR_BG2_PROPERTIES] as usize];
        let mut tm = 0x16;
        if ts < 0 {
            tm = 0x17;
            ts = 0;
        }
        self.ram[TM_COPY] = tm;
        self.ram[TS_COPY] = ts as u8;

        self.ram[LINK_SPEED_MODIFIER] = 1;
        if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_sub(1);
            self.ram[STAIRCASE_MOVE_COUNTER] = 0x32;
            self.ram[SOUND_EFFECT_1] = 25;
        } else {
            self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_add(1);
            self.ram[STAIRCASE_MOVE_COUNTER] = 0x3c;
            self.ram[SOUND_EFFECT_1] = 23;
        }

        let mut r0 = 0u8;
        let y_delta = if self.frame_control_view().submodule() == 18 {
            (-32i16) as u16
        } else {
            32
        };
        if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
            let y = self.player_state_view().y().wrapping_add(y_delta);
            self.player_state_view_mut().set_y(y);
            r0 = r0.wrapping_add(1);
        }
        let plane = self.ram[CUR_STAIRCASE_PLANE] as usize;
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = K_TELEPORT_PIT_LEVEL1[plane];
        self.ram[LINK_IS_ON_LOWER_LEVEL] = K_TELEPORT_PIT_LEVEL2[plane];
        if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
            let y = self.player_state_view().y().wrapping_add(y_delta);
            self.player_state_view_mut().set_y(y);
            r0 = r0.wrapping_add(1);
        }

        if r0 == 0 {
            let delta = if self.frame_control_view().submodule() == 18 {
                if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
                    (-24i16) as u16
                } else {
                    (-8i16) as u16
                }
            } else {
                12
            };
            let y = self.player_state_view().y().wrapping_add(delta);
            self.player_state_view_mut().set_y(y);
        }

        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
        self.hud_restore_torch_background();
        self.Dungeon_InterRoomTrans_notDarkRoom();
    }

    pub(super) fn Module07_11_19_SetSongAndFilter(&mut self) {
        if self.ram[OVERWORLD_MAP_STATE] == 5 && self.ram[DARKENING_OR_LIGHTENING_SCREEN] == 0 {
            self.frame_control_view_mut().increment_subsubmodule();
            let room = self.world_state_view().dungeon_room();
            if room == 48 {
                self.ram[MUSIC_CONTROL] = 0x1c;
            } else if room == 64 {
                self.ram[MUSIC_CONTROL] = 0x10;
            }
        }
        self.ApplyGrayscaleFixed_Incremental();
    }

    pub(super) fn Module07_11_11_KeepSliding(&mut self) {
        if self.ram[STAIRCASE_MOVE_COUNTER] == 0 {
            self.frame_control_view_mut().increment_subsubmodule();
        } else {
            self.ApplyGrayscaleFixed_Incremental();
        }
    }

    pub(super) fn Module07_14_RecoverFromFall(&mut self) {
        match self.frame_control_view().subsubmodule() {
            0 => self.Module07_14_00_ScrollCamera(),
            1 => self.RecoverPositionAfterDrowning(),
            _ => {}
        }
    }

    pub(super) fn Module07_14_00_ScrollCamera(&mut self) {
        for _ in 0..2 {
            let h = read_le_u16(&self.ram, BG2HOFS_COPY2);
            let h_cached = read_le_u16(&self.ram, BG2HOFS_COPY2_CACHED);
            if h != h_cached {
                write_le_u16(
                    &mut self.ram,
                    BG2HOFS_COPY2,
                    if h < h_cached {
                        h.wrapping_add(1)
                    } else {
                        h.wrapping_sub(1)
                    },
                );
            }
            let v = read_le_u16(&self.ram, BG2VOFS_COPY2);
            let v_cached = read_le_u16(&self.ram, BG2VOFS_COPY2_CACHED);
            if v != v_cached {
                write_le_u16(
                    &mut self.ram,
                    BG2VOFS_COPY2,
                    if v < v_cached {
                        v.wrapping_add(1)
                    } else {
                        v.wrapping_sub(1)
                    },
                );
            }
        }
        if read_le_u16(&self.ram, BG2HOFS_COPY2) == read_le_u16(&self.ram, BG2HOFS_COPY2_CACHED)
            && read_le_u16(&self.ram, BG2VOFS_COPY2) == read_le_u16(&self.ram, BG2VOFS_COPY2_CACHED)
        {
            self.frame_control_view_mut().increment_subsubmodule();
        }
        if self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] == 0 {
            self.MirrorBg1Bg2Offs();
        }
    }

    pub(super) fn HandleLinkOnSpiralStairs(&mut self) {
        copy_le_u16(&mut self.ram, LINK_X_COORD_PREV, LINK_X_COORD);
        copy_le_u16(&mut self.ram, LINK_Y_COORD_PREV, LINK_Y_COORD);
        if self.ram[Y_BUTTON_ACTION_STEP] != 0 {
            return;
        }

        self.ram[LINK_GIVE_DAMAGE] = 0;
        self.ram[LINK_INCAPACITATED_TIMER] = 0;
        self.ram[LINK_AUXILIARY_STATE] = 0;

        self.ram[LINK_ACTUAL_VEL_Y] = (-2i8) as u8;
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = self.ram[LINK_TIMER_PUSH_GET_TIRED].wrapping_sub(1);
        if (self.ram[LINK_TIMER_PUSH_GET_TIRED] as i8).is_negative() {
            self.ram[LINK_TIMER_PUSH_GET_TIRED] = 0;
            if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
                self.ram[LINK_ACTUAL_VEL_Y] = 0;
                self.ram[LINK_ACTUAL_VEL_X] = (-2i8) as u8;
            } else {
                self.ram[LINK_ACTUAL_VEL_Y] = (-2i8) as u8;
                self.ram[LINK_ACTUAL_VEL_X] = 2;
            }
        }
        self.link_move_position();
        self.link_handle_moving_animation_start_with_dash();
        if self.ram[LINK_TIMER_PUSH_GET_TIRED] == 0 {
            self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] =
                self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES].wrapping_sub(1);
            if (self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] as i8).is_negative() {
                self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] = 0;
                self.ram[LINK_DIRECTION_FACING] = if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
                    4
                } else {
                    6
                };
            }
        }

        let mut xd =
            self.ram[LINK_X_COORD].wrapping_sub(self.ram[TILEDETECT_WHICH_Y_POS + 2]) as i8;
        if xd < 0 {
            xd = xd.wrapping_neg();
        }
        if xd != 0 {
            return;
        }

        self.RepositionLinkAfterSpiralStairs();
        if self.ram[FOLLOWER_INDICATOR] != 0 {
            self.follower_initialize();
        }

        let detect_x = self.player_state_view().x().wrapping_add(
            if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
                (-8i16) as u16
            } else {
                12
            },
        );
        write_le_u16(&mut self.ram, TILEDETECT_WHICH_Y_POS + 2, detect_x);
        self.ram[Y_BUTTON_ACTION_STEP] = 1;
        self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] = 6;
        self.ancilla_sfx2_near(if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            25
        } else {
            23
        });
    }

    pub(super) fn SpiralStairs_FindLandingSpot(&mut self) {
        self.ram[LINK_GIVE_DAMAGE] = 0;
        self.ram[LINK_INCAPACITATED_TIMER] = 0;
        self.ram[LINK_AUXILIARY_STATE] = 0;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        copy_le_u16(&mut self.ram, LINK_X_COORD_PREV, LINK_X_COORD);
        copy_le_u16(&mut self.ram, LINK_Y_COORD_PREV, LINK_Y_COORD);
        self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] =
            self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES].wrapping_sub(1);
        if (self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] as i8).is_negative() {
            self.ram[COUNTDOWN_TIMER_FOR_STAIRCASES] = 0;
            self.ram[LINK_DIRECTION_FACING] = 2;
        }

        self.ram[LINK_ACTUAL_VEL_X] = 4;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        if self.ram[WHICH_STAIRCASE_INDEX] & 4 != 0 {
            self.ram[LINK_ACTUAL_VEL_X] = (-4i8) as u8;
            self.ram[LINK_ACTUAL_VEL_Y] = 2;
        }
        if self.ram[Y_BUTTON_ACTION_STEP] == 2 {
            self.ram[LINK_ACTUAL_VEL_X] = 0;
            self.ram[LINK_ACTUAL_VEL_Y] = 16;
        }
        self.link_move_position();
        self.link_handle_moving_animation_start_with_dash();
        if self.ram[LINK_X_COORD] == self.ram[TILEDETECT_WHICH_Y_POS + 2] {
            self.ram[Y_BUTTON_ACTION_STEP] = 2;
        }
    }
}

fn clamp_c_int16_to_u16(value: u16, max: u16) -> u16 {
    if (value as i16) < 0 {
        0
    } else if value > max {
        max
    } else {
        value
    }
}

fn object_subtype1_param(idx: u8) -> Option<usize> {
    const PARAMS: [usize; 256] = [
        0x03d8, 0x02e8, 0x02f8, 0x0328, 0x0338, 0x0400, 0x0410, 0x0388, 0x0390, 0x0420, 0x042a,
        0x0434, 0x043e, 0x0448, 0x0452, 0x045c, 0x0466, 0x0470, 0x047a, 0x0484, 0x048e, 0x0498,
        0x04a2, 0x04ac, 0x04b6, 0x04c0, 0x04ca, 0x04d4, 0x04de, 0x04e8, 0x04f2, 0x04fc, 0x0506,
        0x0598, 0x0600, 0x063c, 0x063c, 0x063c, 0x063c, 0x063c, 0x0642, 0x064c, 0x0652, 0x0658,
        0x065e, 0x0664, 0x066a, 0x0688, 0x0694, 0x06a8, 0x06a8, 0x06a8, 0x06c8, 0x0000, 0x078a,
        0x07aa, 0x0e26, 0x084a, 0x086a, 0x0882, 0x08ca, 0x085a, 0x08fa, 0x091a, 0x0920, 0x092a,
        0x0930, 0x0936, 0x093c, 0x0942, 0x0948, 0x094e, 0x096c, 0x097e, 0x098e, 0x0902, 0x099e,
        0x09d8, 0x09d8, 0x09d8, 0x09fa, 0x156c, 0x1590, 0x1d86, 0x0000, 0x0a14, 0x0a24, 0x0a54,
        0x0a54, 0x0a84, 0x0a84, 0x14dc, 0x1500, 0x061e, 0x0e52, 0x0600, 0x03d8, 0x02c8, 0x02d8,
        0x0308, 0x0318, 0x03e0, 0x03f0, 0x0378, 0x0380, 0x05fa, 0x0648, 0x064a, 0x0670, 0x067c,
        0x06a8, 0x06a8, 0x06a8, 0x06c8, 0x0000, 0x07aa, 0x07ca, 0x084a, 0x089a, 0x08b2, 0x090a,
        0x0926, 0x0928, 0x0912, 0x09f8, 0x1d7e, 0x0000, 0x0a34, 0x0a44, 0x0a54, 0x0a6c, 0x0a84,
        0x0a9c, 0x1524, 0x1548, 0x085a, 0x0606, 0x0e52, 0x05fa, 0x06a0, 0x06a2, 0x0b12, 0x0b14,
        0x09b0, 0x0b46, 0x0b56, 0x1f52, 0x1f5a, 0x0288, 0x0e82, 0x1df2, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x03d8, 0x03d8, 0x03d8, 0x03d8, 0x05aa,
        0x05b2, 0x05b2, 0x05b2, 0x05b2, 0x00e0, 0x00e0, 0x00e0, 0x00e0, 0x0110, 0x0000, 0x0000,
        0x06a4, 0x06a6, 0x0ae6, 0x0b06, 0x0b0c, 0x0b16, 0x0b26, 0x0b36, 0x1f52, 0x1f5a, 0x0288,
        0x0eba, 0x0e82, 0x1df2, 0x0000, 0x0000, 0x03d8, 0x0510, 0x05aa, 0x05aa, 0x0000, 0x0168,
        0x00e0, 0x0158, 0x0100, 0x0110, 0x0178, 0x072a, 0x072a, 0x072a, 0x075a, 0x0670, 0x0670,
        0x0130, 0x0148, 0x072a, 0x072a, 0x072a, 0x075a, 0x00e0, 0x0110, 0x00f0, 0x0110, 0x0000,
        0x0ab4, 0x08da, 0x0ade, 0x0188, 0x01a0, 0x01b0, 0x01c0, 0x01d0, 0x01e0, 0x01f0, 0x0200,
        0x0120, 0x02a8, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000,
    ];
    Some(PARAMS[idx as usize])
}

fn object_subtype2_param(idx: u8) -> Option<usize> {
    const PARAMS: [usize; 64] = [
        0x0b66, 0x0b86, 0x0ba6, 0x0bc6, 0x0c66, 0x0c86, 0x0ca6, 0x0cc6, 0x0be6, 0x0c06, 0x0c26,
        0x0c46, 0x0ce6, 0x0d06, 0x0d26, 0x0d46, 0x0d66, 0x0d7e, 0x0d96, 0x0dae, 0x0dc6, 0x0dde,
        0x0df6, 0x0e0e, 0x0398, 0x03a0, 0x03a8, 0x03b0, 0x0e32, 0x0e26, 0x0ea2, 0x0e9a, 0x0eca,
        0x0ed2, 0x0ede, 0x0ede, 0x0f1e, 0x0f3e, 0x0f5e, 0x0f6a, 0x0ef6, 0x0f72, 0x0f92, 0x0fa2,
        0x0fa2, 0x1088, 0x10a8, 0x10a8, 0x10c8, 0x10c8, 0x10c8, 0x10c8, 0x0e52, 0x1108, 0x1108,
        0x12a8, 0x1148, 0x1160, 0x1178, 0x1190, 0x1458, 0x1488, 0x2062, 0x2086,
    ];
    PARAMS.get(idx as usize).copied()
}

fn object_subtype3_param(idx: u8) -> Option<usize> {
    const PARAMS: [usize; 128] = [
        0x1614, 0x162c, 0x1654, 0x0a0e, 0x0a0c, 0x09fc, 0x09fe, 0x0a00, 0x0a02, 0x0a04, 0x0a06,
        0x0a08, 0x0a0a, 0x0000, 0x0a10, 0x0a12, 0x1dda, 0x1de2, 0x1dd6, 0x1dea, 0x15fc, 0x1dfa,
        0x1df2, 0x1488, 0x1494, 0x149c, 0x14a4, 0x10e8, 0x10e8, 0x10e8, 0x11a8, 0x11c8, 0x11e8,
        0x1208, 0x03b8, 0x03c0, 0x03c8, 0x03d0, 0x1228, 0x1248, 0x1268, 0x1288, 0x0000, 0x0e5a,
        0x0e62, 0x0000, 0x0000, 0x0e82, 0x0e8a, 0x14ac, 0x14c4, 0x10e8, 0x1614, 0x1614, 0x1614,
        0x1614, 0x1614, 0x1614, 0x1cbe, 0x1cee, 0x1d1e, 0x1d4e, 0x1d8e, 0x1d96, 0x1d9e, 0x1da6,
        0x1dae, 0x1db6, 0x1dbe, 0x1dc6, 0x1dce, 0x0220, 0x0260, 0x0280, 0x1f3a, 0x1f62, 0x1f92,
        0x1ff2, 0x2016, 0x1f42, 0x0eaa, 0x1f4a, 0x1f52, 0x1f5a, 0x202e, 0x2062, 0x09b8, 0x09c0,
        0x09c8, 0x09d0, 0x0fa2, 0x0fb2, 0x0fc4, 0x0ff4, 0x1018, 0x1020, 0x15b4, 0x15d8, 0x20f6,
        0x0eba, 0x22e6, 0x22ee, 0x05da, 0x281e, 0x2ae0, 0x2d2a, 0x2f2a, 0x22f6, 0x2316, 0x232e,
        0x2346, 0x235e, 0x2376, 0x23b6, 0x1e9a, 0x0000, 0x2436, 0x149c, 0x24b6, 0x24e6, 0x2516,
        0x1028, 0x1040, 0x1060, 0x1070, 0x1078, 0x1080, 0x0000,
    ];
    PARAMS.get(idx as usize).copied()
}

fn replay_room_write_trace_addr(offset: usize) -> bool {
    let Ok(raw) = std::env::var("ZELDA3_REPLAY_ROOM_WRITE_TRACE_ADDR") else {
        return false;
    };
    raw.split(',').any(|part| {
        let part = part.trim();
        if part.is_empty() {
            return false;
        }
        let parsed = part
            .strip_prefix("0x")
            .or_else(|| part.strip_prefix("0X"))
            .map_or_else(
                || part.parse::<usize>().ok(),
                |hex| usize::from_str_radix(hex, 16).ok(),
            );
        parsed == Some(offset)
    })
}

fn replay_room_write_trace_enabled() -> bool {
    std::env::var_os("ZELDA3_REPLAY_ROOM_WRITE_TRACE_ADDR").is_some()
}

impl ZeldaState {
    pub(super) fn module_pre_dungeon(&mut self) {
        self.ram[SOUND_EFFECT_AMBIENT] = 5;
        self.ram[SOUND_EFFECT_1] = 0;
        write_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX, 0);
        write_le_u16(&mut self.ram, DUNGEON_ROOM_INDEX_PREV, 0);
        self.ram[DUNG_SAVEGAME_STATE_BITS] = 0;
        self.ram[DUNG_SAVEGAME_STATE_BITS + 1] = 0;
        self.ram[AGAHNIM_PAL_SETTING..AGAHNIM_PAL_SETTING + 12].fill(0);

        self.Dungeon_LoadEntrance();
        self.load_pre_dungeon_keys();
        self.hud_rebuild();
        self.ram[DUNG_NUM_LIT_TORCHES] = 0;
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 0;
        self.Dungeon_LoadAndDrawRoom();
        self.Dungeon_LoadCustomTileAttr();

        let animated = DUNG_ANIMATED_TILES[self.ram[MAIN_TILE_THEME_INDEX] as usize];
        self.decompress_animated_dungeon_tiles(animated as usize);
        self.Dungeon_LoadAttributeTable();
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = 10;
        self.initialize_tilesets();
        self.ram[PALETTE_SP6R_INDOORS] = 10;
        self.dungeon_load_palettes();

        let room = self.world_state_view().dungeon_room();
        write_le_u16(
            &mut self.ram,
            DUNG_LOADE_BGOFFS_H_COPY,
            (room & 0x000f) << 9,
        );
        write_le_u16(
            &mut self.ram,
            DUNG_LOADE_BGOFFS_V_COPY,
            (room & 0x0ff0) << 5,
        );
        if room == 0x0104 && self.ram[SRAM_PROGRESS_FLAGS] & 0x10 != 0 {
            write_le_u16(&mut self.ram, DUNG_WANT_LIGHTS_OUT, 0);
        }
        self.SetAndSaveVisitedQuadrantFlags();

        const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
        self.ram[CGWSEL_COPY] = 2;
        let mut torch = self.ram[DUNG_NUM_LIT_TORCHES] as usize;
        self.ram[CGADSUB_COPY] = if self.ram[DUNG_WANT_LIGHTS_OUT] == 0 {
            torch = 3;
            if self.ram[DUNG_HDR_BG2_PROPERTIES] == 7 {
                0x32
            } else if self.ram[DUNG_HDR_BG2_PROPERTIES] == 4 {
                0x62
            } else {
                0x20
            }
        } else {
            0xb3
        };
        self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = LIT_TORCHES_COLOR_PLUS[torch];
        self.Dungeon_ApproachFixedColor_variable(self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS]);
        self.ram[PALETTE_FILTER_COUNTDOWN] = 0x1f;
        self.ram[MOSAIC_TARGET_LEVEL] = 0;
        self.ram[DARKENING_OR_LIGHTENING_SCREEN] = 2;
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0);
        self.ram[LINK_SPEED_MODIFIER] = 0;
        self.ram[BUTTON_MASK_B_Y] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.Dungeon_ResetTorchBackgroundAndPlayer();
        self.link_check_bunny_status();
        self.reset_then_cache_room_entry_properties();
        if self.ram[FOLLOWER_INDICATOR] == 13 {
            self.ram[FOLLOWER_INDICATOR] = 0;
            self.ram[SUPER_BOMB_INDICATOR_TIMER] = 0;
            self.hud_remove_super_bomb_indicator();
        }
        self.ram[BGMODE_COPY] = 9;
        self.follower_initialize();
        self.sprite_reset_all();
        self.dungeon_reset_sprites();
        self.ram[MESSAGE_OR_SPRITE_STATE_CACHE] = 0;
        self.ram[FLAG_SKIP_CALL_TAG_ROUTINES] =
            self.ram[FLAG_SKIP_CALL_TAG_ROUTINES].wrapping_add(1);

        if self.ram[SRAM_PROGRESS_INDICATOR] == 0 && self.ram[SRAM_PROGRESS_FLAGS] & 0x10 == 0 {
            self.ram[COLDATA_COPY0] = 0x30;
            self.ram[COLDATA_COPY1] = 0x50;
            self.ram[COLDATA_COPY2] = 0x80;
            self.ram[DUNG_WANT_LIGHTS_OUT] = 0;
            self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = 0;
            self.link_tuck_into_bed();
        }

        self.ram[SAVED_MODULE_FOR_MENU] = 7;
        self.frame_control_view_mut().set_main_module(7);
        self.frame_control_view_mut().set_submodule(15);
        self.Dungeon_LoadSongBankIfNeeded();
        self.module_pre_dungeon_set_ambient_sfx();
    }

    pub(super) fn link_check_bunny_status(&mut self) {
        if self.ram[LINK_PLAYER_HANDLER_STATE] == 2 {
            self.ram[LINK_PLAYER_HANDLER_STATE] = if self.ram[LINK_IS_BUNNY_MIRROR] == 0 {
                0
            } else if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
                28
            } else {
                23
            };
        }
    }

    pub(super) fn CrystalCutscene_Initialize(&mut self) {
        const CRYSTAL_MAIDEN_PAL: [u16; 8] =
            [0, 0x3821, 0x4463, 0x54a5, 0x5ce7, 0x6d29, 0x79ad, 0x7e10];

        self.ram[CGADSUB_COPY] = 0x33;
        self.ram[PALETTE_FILTER_COUNTDOWN] = 0;
        self.ram[DARKENING_OR_LIGHTENING_SCREEN] = 0;
        self.Palette_AssertTranslucencySwap();
        self.PaletteFilter_Crystal();
        for (i, color) in CRYSTAL_MAIDEN_PAL.iter().enumerate() {
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (112 + i) * 2, *color);
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        self.CrystalCutscene_SpawnMaiden();
        self.crystal_cutscene_initialize_polyhedral();
    }

    pub(super) fn CrystalCutscene_SpawnMaiden(&mut self) {
        self.ram[SPRITE_STATE..SPRITE_STATE + 16].fill(0);
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(0, 0xab, &mut info);
        let j = j as usize;

        self.ram[SPRITE_X_HI + j] = (self.player_state_view().x() >> 8) as u8;
        self.ram[SPRITE_Y_HI + j] = (self.player_state_view().y() >> 8) as u8;
        self.ram[SPRITE_X_LO + j] = 0x78;
        self.ram[SPRITE_Y_LO + j] = 0x7c;
        self.ram[SPRITE_D + j] = 1;
        self.ram[SPRITE_OAM_FLAGS + j] = 0x0b;
        self.ram[SPRITE_SUBTYPE2 + j] = 0;
        self.ram[SPRITE_FLOOR + j] = 0;
        self.ram[SPRITE_A + j] = self.ancilla_terminate_select_interactives(j as u8);
        self.ram[ITEM_RECEIPT_METHOD] = 0;

        if self.ram[CUR_PALACE_INDEX_X2] == 24 {
            self.ram[SPRITE_OAM_FLAGS + j] = 9;
            self.ram[FOLLOWER_INDICATOR] = 1;
        } else {
            self.ram[FOLLOWER_INDICATOR] = 6;
        }
        self.LoadFollowerGraphics();
        self.ram[FOLLOWER_INDICATOR] = 0;

        let floor_x = read_le_u16(&self.ram, BG2HOFS_COPY2)
            .wrapping_sub(self.player_state_view().x())
            .wrapping_add(0x79);
        write_le_u16(&mut self.ram, DUNG_FLOOR_X_OFFS, floor_x);
        let floor_y = 0x30u16.wrapping_sub(self.ram[BG1VOFS_COPY2] as u16);
        write_le_u16(&mut self.ram, DUNG_FLOOR_Y_OFFS, floor_y);
        self.ram[DUNG_HDR_COLLISION_2_MIRROR] = 1;
    }

    pub(super) fn reset_then_cache_room_entry_properties(&mut self) {
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[OVERWORLD_SCREEN_TRANSITION] = 0;
        self.frame_control_view_mut().set_submodule(0);
        self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = 0;
        self.ram[DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED] = 0;
        self.cache_camera_properties();
    }

    pub(super) fn ResetThenCacheRoomEntryProperties(&mut self) {
        self.reset_then_cache_room_entry_properties();
    }

    pub(super) fn module_pre_dungeon_set_ambient_sfx(&mut self) {
        if self.ram[SRAM_PROGRESS_INDICATOR] < 2 {
            self.ram[SOUND_EFFECT_AMBIENT] = 5;
            if (self.ram[DUNG_CUR_FLOOR] as i8) >= 0
                && self.world_state_view().dungeon_room() != 2
                && self.world_state_view().dungeon_room() != 18
            {
                self.ram[SOUND_EFFECT_AMBIENT] = 3;
            }
        }
    }

    pub(super) fn module07_dungeon(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        self.dungeon_handle_layer_effect();
        self.replay_trace_ram_watch("module07-after-layer-effect");
        self.run_dungeon_submodule();
        self.replay_trace_ram_watch("module07-after-submodule");

        if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES == 0
            || self.frame_control_view().main_module() == 7
        {
            self.ram[DUNG_MISC_OBJS_INDEX] = 0;
            self.dungeon_push_block_handler();
            self.replay_trace_ram_watch("module07-after-push-blocks");
            if self.frame_control_view().submodule() == 0 {
                self.graphics_load_chr_half_slot();
                self.dungeon_handle_camera();
                self.replay_trace_ram_watch("module07-after-camera");
            }
            if self.frame_control_view().submodule() == 0 {
                self.dungeon_handle_room_tags();
                self.replay_trace_ram_watch("module07-after-room-tags");
            }
            if self.frame_control_view().submodule() == 0 {
                self.dungeon_process_torches_and_doors();
                self.replay_trace_ram_watch("module07-after-torches-doors");
                if read_le_u16(&self.ram, CRUSH_WALL_PROGRESS) != 0 {
                    self.dungeon_clear_away_exploding_wall();
                    self.replay_trace_ram_watch("module07-after-blast-wall");
                }
                if self.ram[IS_STANDING_IN_DOORWAY] == 0 {
                    self.Dungeon_TryScreenEdgeTransition();
                    self.replay_trace_ram_watch("module07-after-screen-edge");
                }
            }
        }

        self.orient_lamp_light_cone();
        self.replay_trace_ram_watch("module07-after-lamp");

        let bg2x = read_le_u16(&self.ram, BG2HOFS_COPY2);
        let bg2y = read_le_u16(&self.ram, BG2VOFS_COPY2);
        let bg1x = read_le_u16(&self.ram, BG1HOFS_COPY2);
        let bg1y = read_le_u16(&self.ram, BG1VOFS_COPY2);

        let bg1_x_offset = read_le_u16(&self.ram, BG1_X_OFFSET);
        let bg1_y_offset = read_le_u16(&self.ram, BG1_Y_OFFSET);
        write_le_u16(
            &mut self.ram,
            BG2HOFS_COPY2,
            bg2x.wrapping_add(bg1_x_offset),
        );
        copy_le_u16(&mut self.ram, BG2HOFS_COPY, BG2HOFS_COPY2);
        write_le_u16(
            &mut self.ram,
            BG2VOFS_COPY2,
            bg2y.wrapping_add(bg1_y_offset),
        );
        copy_le_u16(&mut self.ram, BG2VOFS_COPY, BG2VOFS_COPY2);
        write_le_u16(
            &mut self.ram,
            BG1HOFS_COPY2,
            bg1x.wrapping_add(bg1_x_offset),
        );
        copy_le_u16(&mut self.ram, BG1HOFS_COPY, BG1HOFS_COPY2);
        write_le_u16(
            &mut self.ram,
            BG1VOFS_COPY2,
            bg1y.wrapping_add(bg1_y_offset),
        );
        copy_le_u16(&mut self.ram, BG1VOFS_COPY, BG1VOFS_COPY2);

        let mut bg1x_restore = bg1x;
        let mut bg1y_restore = bg1y;
        if self.ram[DUNG_HDR_COLLISION_2_MIRROR] != 0 {
            bg1x_restore = read_le_u16(&self.ram, BG2HOFS_COPY2)
                .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_X_OFFS));
            bg1y_restore = read_le_u16(&self.ram, BG2VOFS_COPY2)
                .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_Y_OFFS));
            write_le_u16(&mut self.ram, BG1HOFS_COPY2, bg1x_restore);
            copy_le_u16(&mut self.ram, BG1HOFS_COPY, BG1HOFS_COPY2);
            write_le_u16(&mut self.ram, BG1VOFS_COPY2, bg1y_restore);
            copy_le_u16(&mut self.ram, BG1VOFS_COPY, BG1VOFS_COPY2);
        }

        self.sprite_dungeon_draw_all_push_blocks();
        self.replay_trace_ram_watch("module07-after-draw-push-blocks");
        self.sprite_main();
        self.replay_trace_ram_watch("module07-after-sprite-main");

        write_le_u16(&mut self.ram, BG2HOFS_COPY2, bg2x);
        write_le_u16(&mut self.ram, BG2VOFS_COPY2, bg2y);
        write_le_u16(&mut self.ram, BG1HOFS_COPY2, bg1x_restore);
        write_le_u16(&mut self.ram, BG1VOFS_COPY2, bg1y_restore);

        self.link_oam_main();
        self.replay_trace_ram_watch("module07-after-link-oam");
        self.hud_refill_logic();
        self.replay_trace_ram_watch("module07-after-refill");
        self.hud_floor_indicator();
        self.replay_trace_ram_watch("module07-after-floor-indicator");
    }

    pub(super) fn module07_00_player_control(&mut self) {
        if (self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE]
            | self.ram[FLAG_IS_LINK_IMMOBILIZED]
            | self.ram[FLAG_BLOCK_LINK_MENU])
            == 0
        {
            if self.ram[FILTERED_JOYPAD_H] & 0x10 != 0 {
                self.ram[OVERWORLD_MAP_STATE] = 0;
                self.frame_control_view_mut().set_submodule(1);
                self.ram[SAVED_MODULE_FOR_MENU] = self.frame_control_view().main_module();
                self.frame_control_view_mut().set_main_module(14);
                return;
            } else if self.did_press_button_for_map() {
                if self.ram[CUR_PALACE_INDEX_X2] != 0xff && self.ram[DUNGEON_ROOM_INDEX] != 0 {
                    self.ram[OVERWORLD_MAP_STATE] = 0;
                    self.frame_control_view_mut().set_submodule(3);
                    self.ram[SAVED_MODULE_FOR_MENU] = self.frame_control_view().main_module();
                    self.frame_control_view_mut().set_main_module(14);
                    return;
                }
            } else if self.ram[JOYPAD1H_LAST] & 0x20 != 0 && self.ram[SRAM_PROGRESS_INDICATOR] != 0
            {
                self.ram[OVERWORLD_MAP_STATE] = 0;
                self.DisplaySelectMenu();
                return;
            }
            self.replay_trace_ram_watch("module07-before-hud-switch");
            self.hud_handle_item_switch_inputs();
            self.replay_trace_ram_watch("module07-after-hud-switch");
        }
        self.replay_trace_ram_watch("module07-before-link-main");
        self.link_main();
        self.replay_trace_ram_watch("module07-after-link-main");
    }

    pub(super) fn dungeon_handle_layer_effect(&mut self) {
        self.Dungeon_HandleLayerEffect();
    }

    pub(super) fn Dungeon_HandleLayerEffect(&mut self) {
        match self.ram[DUNG_HDR_COLLISION_2] {
            0 | 1 => self.LayerEffect_Nothing(),
            2 => self.LayerEffect_Scroll(),
            3 => self.LayerEffect_WaterRapids(),
            4 => self.LayerEffect_Trinexx(),
            5 => self.LayerEffect_Agahnim2(),
            6 => self.LayerEffect_InvisibleFloor(),
            7 => self.LayerEffect_Ganon(),
            _ => panic!("invalid dungeon layer effect index"),
        }
    }

    pub(super) fn LayerEffect_Nothing(&mut self) {}

    pub(super) fn LayerEffect_Scroll(&mut self) {
        if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x8000 != 0 {
            self.ram[DUNG_HDR_COLLISION_2] = 0;
            return;
        }
        write_le_u16(&mut self.ram, DUNG_FLOOR_X_VEL, 0);
        write_le_u16(&mut self.ram, DUNG_FLOOR_Y_VEL, 0);
        let flags = read_le_u16(&self.ram, DUNG_FLOOR_MOVE_FLAGS);
        if flags & 1 != 0 {
            return;
        }
        let subpixel = u16::from(self.ram[BG1_MOVE_CALC_BUFFER + 1]) + 0x80;
        self.ram[BG1_MOVE_CALC_BUFFER + 1] = subpixel as u8;
        let mut t = (subpixel >> 8) as i16;
        if flags & 2 != 0 {
            t = -t;
        }
        if flags < 4 {
            write_le_u16(&mut self.ram, DUNG_FLOOR_X_VEL, t as u16);
            let x_offs = read_le_u16(&self.ram, DUNG_FLOOR_X_OFFS).wrapping_sub(t as u16);
            write_le_u16(&mut self.ram, DUNG_FLOOR_X_OFFS, x_offs);
            let bg1 = read_le_u16(&self.ram, BG2HOFS_COPY2).wrapping_add(x_offs);
            write_le_u16(&mut self.ram, BG1HOFS_COPY2, bg1);
        } else {
            write_le_u16(&mut self.ram, DUNG_FLOOR_Y_VEL, t as u16);
            let y_offs = read_le_u16(&self.ram, DUNG_FLOOR_Y_OFFS).wrapping_sub(t as u16);
            write_le_u16(&mut self.ram, DUNG_FLOOR_Y_OFFS, y_offs);
            let bg1 = read_le_u16(&self.ram, BG2VOFS_COPY2).wrapping_add(y_offs);
            write_le_u16(&mut self.ram, BG1VOFS_COPY2, bg1);
        }
    }

    pub(super) fn LayerEffect_Trinexx(&mut self) {
        let x = read_le_u16(&self.ram, DUNG_FLOOR_X_OFFS)
            .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_X_VEL));
        let y = read_le_u16(&self.ram, DUNG_FLOOR_Y_OFFS)
            .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_Y_VEL));
        write_le_u16(&mut self.ram, DUNG_FLOOR_X_OFFS, x);
        write_le_u16(&mut self.ram, DUNG_FLOOR_Y_OFFS, y);
        write_le_u16(&mut self.ram, DUNG_FLOOR_X_VEL, 0);
        write_le_u16(&mut self.ram, DUNG_FLOOR_Y_VEL, 0);
    }

    pub(super) fn LayerEffect_Agahnim2(&mut self) {
        let j = self.ram[FRAME_COUNTER] & 0x7f;
        if j == 3 || j == 36 {
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x6d * 2, 0x1d59);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x6e * 2, 0x25ff);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x6f * 2, 0x001a);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x77 * 2, 0x001a);
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        } else if j == 5 || j == 38 {
            let p6d = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + 0x6d * 2);
            let p6e = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + 0x6e * 2);
            let p6f = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + 0x6f * 2);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x6d * 2, p6d);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x6e * 2, p6e);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x6f * 2, p6f);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x77 * 2, p6f);
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        }
        self.ram[TS_COPY] = 2;
    }

    pub(super) fn LayerEffect_InvisibleFloor(&mut self) {
        let mut count = 0;
        for i in 0..16 {
            if read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + i * 2) & 0x8000 != 0 {
                count += 1;
            }
        }
        let (x, y) = if count == 0 { (0, 0) } else { (0x2940, 0x4e60) };
        if read_le_u16(&self.ram, AUX_PALETTE_BUFFER + 0x7b * 2) != x {
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x7b * 2, x);
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + 0x7b * 2, x);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0x7c * 2, y);
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + 0x7c * 2, y);
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        }
        self.ram[TS_COPY] = 2;
    }

    pub(super) fn LayerEffect_Ganon(&mut self) {
        let mut count = 0u8;
        for i in 0..16 {
            if read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + i * 2) & 0x8000 != 0 {
                count = count.wrapping_add(1);
            }
        }
        self.ram[GANON_TORCH_COUNT] = count;
        if count == 0 {
            self.ram[TS_COPY] = 0;
            self.ram[CGADSUB_COPY] = 0xb3;
        } else if count == 1 {
            self.ram[TS_COPY] = 2;
            self.ram[CGADSUB_COPY] = 0x70;
        } else {
            self.ram[TS_COPY] = 0;
            self.ram[CGADSUB_COPY] = 0x70;
        }
    }

    pub(super) fn LayerEffect_WaterRapids(&mut self) {
        let t = u16::from(self.ram[BG1_MOVE_CALC_BUFFER + 1]) + 0x80;
        self.ram[BG1_MOVE_CALC_BUFFER + 1] = t as u8;
        write_le_u16(&mut self.ram, DUNG_FLOOR_X_VEL, (-(t as i16 >> 8)) as u16);
    }

    pub(super) fn Module07_15_01_ApplyMosaicAndFilter(&mut self) {
        self.conditional_mosaic_control();
        self.ram[MOSAIC_COPY] = self.ram[MOSAIC_LEVEL] | 3;
        self.apply_palette_filter_bounce();
    }

    pub(super) fn Module07_15_04_SyncRoomPropsAndBuildOverlay(&mut self) {
        self.ApplyGrayscaleFixed_Incremental();
        if self.world_state_view().dungeon_room() == 0x17 {
            self.ram[DUNG_CUR_FLOOR] = 4;
        }
        self.MirrorBg1Bg2Offs();
        self.Dungeon_AdjustForRoomLayout();
        let mut ts = K_SPIRAL_TAB1[self.ram[DUNG_HDR_BG2_PROPERTIES] as usize] as u8;
        let mut tm = 0x16;
        if ts & 0x80 != 0 {
            tm = 0x17;
            ts = 0;
        }
        self.ram[TM_COPY] = tm;
        self.ram[TS_COPY] = ts;
        self.WaterFlood_BuildOneQuadrantForVRAM();
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn Module07_15_0E_FadeInFromWarp(&mut self) {
        if self.ram[PALETTE_FILTER_COUNTDOWN] & 1 != 0 && self.ram[MOSAIC_LEVEL] != 0 {
            self.ram[MOSAIC_LEVEL] = self.ram[MOSAIC_LEVEL].wrapping_sub(0x10);
        }
        self.ram[BGMODE_COPY] = 9;
        self.ram[MOSAIC_COPY] = self.ram[MOSAIC_LEVEL] | 3;
        self.ApplyPaletteFilter_bounce();
    }

    pub(super) fn Module07_15_0F_FinalizeAndCacheEntry(&mut self) {
        if self.ram[OVERWORLD_MAP_STATE] == 5 {
            self.SetAndSaveVisitedQuadrantFlags();
            self.frame_control_view_mut().set_submodule(0);
            self.ResetThenCacheRoomEntryProperties();
        }
    }

    pub(super) fn dungeon_push_block_handler(&mut self) {
        const PUSH_BLOCK_MOVE_DISTANCES: [i16; 4] = [-0x100, 0x100, -0x04, 0x04];
        while read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX)
            != read_le_u16(&self.ram, DUNG_INDEX_OF_TORCHES_START)
        {
            let obj = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX);
            let k = usize::from(obj >> 1);
            match read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_STATE + k * 2) {
                1 => {
                    self.RoomDraw_16x16Single(obj as u8);
                    let dir = usize::from((self.ram[PUSH_BLOCK_DIRECTION_DUNGEON] >> 1) & 3);
                    let pos = read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + k * 2)
                        .wrapping_add_signed(PUSH_BLOCK_MOVE_DISTANCES[dir]);
                    write_le_u16(&mut self.ram, DUNG_OBJECT_TILEMAP_POS + k * 2, pos);
                    write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_STATE + k * 2, 2);
                }
                2 => {
                    self.PushBlock_Slide(obj as u8);
                    let obj = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX);
                    let k = usize::from(obj >> 1);
                    if read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_STATE + k * 2) == 3 {
                        self.PushBlock_CheckForPit(obj as u8);
                        let state = read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_STATE + k * 2)
                            .wrapping_add(1);
                        write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_STATE + k * 2, state);
                    }
                }
                4 => {
                    self.PushBlock_HandleFalling(obj as u8);
                }
                _ => {}
            }
            let next = read_le_u16(&self.ram, DUNG_MISC_OBJS_INDEX).wrapping_add(2);
            write_le_u16(&mut self.ram, DUNG_MISC_OBJS_INDEX, next);
        }
    }

    pub(super) fn dungeon_handle_camera(&mut self) {
        let link_y_vel = self.ram[LINK_Y_VEL];
        if link_y_vel != 0 {
            let z = if self.ram[ALLOW_SCROLL_Z] != 0 && self.player_state_view().z() != 0xffff {
                self.player_state_view().z()
            } else {
                0
            };
            let y = self.player_state_view().y().wrapping_sub(z) & 0x01ff;
            let y = y.wrapping_add(12);
            let moving_up = (link_y_vel as i8).is_negative();
            let scrollamt: i16 = if moving_up { -1 } else { 1 };
            let steps = if moving_up {
                (link_y_vel as i8).wrapping_neg() as u8
            } else {
                link_y_vel
            };

            for _ in 0..steps {
                let mut qm = (self.ram[QUADRANT_FULLSIZE_Y] >> 1) as usize;
                if moving_up {
                    if y > read_le_u16(&self.ram, CAMERA_Y_COORD_SCROLL_LOW) {
                        continue;
                    }
                } else {
                    if y < read_le_u16(&self.ram, CAMERA_Y_COORD_SCROLL_HI) {
                        continue;
                    }
                    qm += 2;
                }

                if read_le_u16(&self.ram, BG2VOFS_COPY2)
                    == read_le_u16(&self.ram, ROOM_BOUNDS_Y + qm * 2)
                {
                    continue;
                }

                let bg2 = read_le_u16(&self.ram, BG2VOFS_COPY2).wrapping_add(scrollamt as u16);
                write_le_u16(&mut self.ram, BG2VOFS_COPY2, bg2);
                if self.world_state_view().dungeon_room() == 0xffff {
                    continue;
                }

                let subpixel = read_le_u16(&self.ram, BG1VOFS_SUBPIXEL).wrapping_add(0x8000);
                write_le_u16(&mut self.ram, BG1VOFS_SUBPIXEL, subpixel);
                let bg1_delta = (scrollamt >> 1) + i16::from(subpixel & 0x8000 == 0);
                let bg1 = read_le_u16(&self.ram, BG1VOFS_COPY2).wrapping_add(bg1_delta as u16);
                write_le_u16(&mut self.ram, BG1VOFS_COPY2, bg1);
                let camera_low = read_le_u16(&self.ram, CAMERA_Y_COORD_SCROLL_LOW)
                    .wrapping_add(scrollamt as u16);
                write_le_u16(&mut self.ram, CAMERA_Y_COORD_SCROLL_LOW, camera_low);
                write_le_u16(
                    &mut self.ram,
                    CAMERA_Y_COORD_SCROLL_HI,
                    camera_low.wrapping_add(2),
                );
            }
        }

        let link_x_vel = self.ram[LINK_X_VEL];
        if link_x_vel != 0 {
            let x = (self.player_state_view().x() & 0x01ff).wrapping_add(8);
            let moving_left = (link_x_vel as i8).is_negative();
            let scrollamt: i16 = if moving_left { -1 } else { 1 };
            let steps = if moving_left {
                (link_x_vel as i8).wrapping_neg() as u8
            } else {
                link_x_vel
            };

            for _ in 0..steps {
                let mut qm = (self.ram[QUADRANT_FULLSIZE_X] >> 1) as usize;
                if moving_left {
                    if x > read_le_u16(&self.ram, CAMERA_X_COORD_SCROLL_LOW) {
                        continue;
                    }
                } else {
                    if x < read_le_u16(&self.ram, CAMERA_X_COORD_SCROLL_HI) {
                        continue;
                    }
                    qm += 2;
                }

                if read_le_u16(&self.ram, BG2HOFS_COPY2)
                    == read_le_u16(&self.ram, ROOM_BOUNDS_X + qm * 2)
                {
                    continue;
                }

                let bg2 = read_le_u16(&self.ram, BG2HOFS_COPY2).wrapping_add(scrollamt as u16);
                write_le_u16(&mut self.ram, BG2HOFS_COPY2, bg2);
                if self.world_state_view().dungeon_room() == 0xffff {
                    continue;
                }

                let subpixel = read_le_u16(&self.ram, BG1HOFS_SUBPIXEL).wrapping_add(0x8000);
                write_le_u16(&mut self.ram, BG1HOFS_SUBPIXEL, subpixel);
                let bg1_delta = (scrollamt >> 1) + i16::from(subpixel & 0x8000 == 0);
                let bg1 = read_le_u16(&self.ram, BG1HOFS_COPY2).wrapping_add(bg1_delta as u16);
                write_le_u16(&mut self.ram, BG1HOFS_COPY2, bg1);
                let camera_low = read_le_u16(&self.ram, CAMERA_X_COORD_SCROLL_LOW)
                    .wrapping_add(scrollamt as u16);
                write_le_u16(&mut self.ram, CAMERA_X_COORD_SCROLL_LOW, camera_low);
                write_le_u16(
                    &mut self.ram,
                    CAMERA_X_COORD_SCROLL_HI,
                    camera_low.wrapping_add(2),
                );
            }
        }

        if self.world_state_view().dungeon_room() != 0xffff {
            let bg2_properties = self.ram[DUNG_HDR_BG2_PROPERTIES];
            if bg2_properties == 0
                || bg2_properties == 2
                || bg2_properties == 3
                || bg2_properties == 4
                || bg2_properties >= 6
            {
                copy_le_u16(&mut self.ram, BG1HOFS_COPY2, BG2HOFS_COPY2);
                copy_le_u16(&mut self.ram, BG1VOFS_COPY2, BG2VOFS_COPY2);
            }
        }
    }

    pub(super) fn dungeon_handle_room_tags(&mut self) {
        if self.ram[FLAG_SKIP_CALL_TAG_ROUTINES] == 0 {
            self.Dungeon_DetectStaircase();

            if self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_MISC_BUG_FIXES_DUNGEON != 0
                && self.frame_control_view().submodule() != 0
            {
                return;
            }

            self.ram[R14] = 0;
            self.dungeon_run_tag_routine(0);
            self.ram[R14] = 1;
            self.dungeon_run_tag_routine(1);
        }
        self.ram[FLAG_SKIP_CALL_TAG_ROUTINES] = 0;
    }

    fn dungeon_run_tag_routine(&mut self, k: usize) {
        match self.ram[DUNG_HDR_TAG + k] {
            0x00 => self.Dung_TagRoutine_0x00(k),
            0x01 | 0x0b | 0x29 => self.RoomTag_NorthWestTrigger(k),
            0x02 | 0x0c | 0x2a => self.Dung_TagRoutine_0x2A(k),
            0x03 | 0x0d | 0x2b => self.Dung_TagRoutine_0x2B(k),
            0x04 | 0x0e | 0x2c => self.Dung_TagRoutine_0x2C(k),
            0x05 | 0x0f | 0x2d => self.Dung_TagRoutine_0x2D(k),
            0x06 | 0x10 | 0x2e => self.Dung_TagRoutine_0x2E(k),
            0x07 | 0x11 | 0x2f => self.Dung_TagRoutine_0x2F(k),
            0x08 | 0x12 | 0x30 => self.Dung_TagRoutine_0x30(k),
            0x09 | 0x13 | 0x31 => self.RoomTag_QuadrantTrigger(k),
            0x0a | 0x32 => self.RoomTag_RoomTrigger(k),
            0x14 => self.RoomTag_RoomTrigger_BlockDoor(k),
            0x15 => self.RoomTag_PrizeTriggerDoorDoor(k),
            0x16 => self.RoomTag_SwitchTrigger_HoldDoor(k),
            0x17 => self.RoomTag_SwitchTrigger_ToggleDoor(k),
            0x18 => self.RoomTag_WaterOff(k),
            0x19 => self.RoomTag_WaterOn(k),
            0x1a => self.RoomTag_WaterGate(k),
            0x1b => self.Dung_TagRoutine_0x1B(k),
            0x1c => self.RoomTag_MovingWall_East(k),
            0x1d => self.RoomTag_MovingWall_West(k),
            0x1e | 0x1f => self.RoomTag_MovingWallTorchesCheck(k),
            0x20 => self.RoomTag_Switch_ExplodingWall(k),
            0x21 => self.RoomTag_Holes0(k),
            0x22 => self.RoomTag_ChestHoles0(k),
            0x23 => self.Dung_TagRoutine_0x23(k),
            0x24 => self.RoomTag_Holes2(k),
            0x25 => self.RoomTag_GetHeartForPrize(k),
            0x26 => self.RoomTag_KillRoomBlock(k),
            0x27 => self.RoomTag_TriggerChest(k),
            0x28 => self.RoomTag_PullSwitchExplodingWall(k),
            0x33 => self.RoomTag_TorchPuzzleDoor(k),
            0x34 => self.Dung_TagRoutine_0x34(k),
            0x35 => self.Dung_TagRoutine_0x35(k),
            0x36 => self.Dung_TagRoutine_0x36(k),
            0x37 => self.Dung_TagRoutine_0x37(k),
            0x38 => self.RoomTag_Agahnim(k),
            0x39 => self.Dung_TagRoutine_0x39(k),
            0x3a => self.Dung_TagRoutine_0x3A(k),
            0x3b => self.Dung_TagRoutine_0x3B(k),
            0x3c => self.RoomTag_PushBlockForChest(k),
            0x3d => self.RoomTag_GanonDoor(k),
            0x3e => self.RoomTag_TorchPuzzleChest(k),
            0x3f => self.RoomTag_RekillableBoss(k),
            _ => {}
        }
    }

    pub(super) fn dungeon_process_torches_and_doors(&mut self) {
        const LINK_OFFS_X: [i32; 4] = [0, 0, -1, 17];
        const LINK_OFFS_Y: [i32; 4] = [7, 24, 8, 8];
        const LINK_OFFS_POS: [usize; 4] = [0x0002, 0x0002, 0x0080, 0x0080];
        const OPEN_DOOR_PANNING: [u8; 4] = [0x00, 0x00, 0x80, 0x40];
        const SRC_TILES_1: [u16; 4] = [0x07ea, 0x080a, 0x080a, 0x082a];

        if self.ram[FRAME_COUNTER] & 3 == 0 && self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] == 0 {
            for i in 0..16 {
                let timer = self.ram[DUNG_TORCH_TIMERS_DUNGEON + i];
                if timer != 0 {
                    let next = timer.wrapping_sub(1);
                    self.ram[DUNG_TORCH_TIMERS_DUNGEON + i] = next;
                    if next == 0 {
                        self.ram[DUNGEON_TORCH_ATTR] = 0xc0 + i as u8;
                        self.Dungeon_ExtinguishTorch();
                    }
                }
            }
        }

        if self.ram[FLAG_IS_LINK_IMMOBILIZED] == 0 {
            let dir = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
            let link_y = self.player_state_view().y() as i32;
            let link_x = self.player_state_view().x() as i32;
            let mut pos = (((link_y + LINK_OFFS_Y[dir]) & 0x01f8) << 3) as usize
                | (((link_x + LINK_OFFS_X[dir]) & 0x01f8) >> 3) as usize
                | if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
                    0x1000
                } else {
                    0
                };

            let mut openable = (self.ram[DUNG_BG2_ATTR_TABLE + pos] & 0xf0) == 0xf0;
            if !openable {
                pos += LINK_OFFS_POS[dir];
                openable = (self.ram[DUNG_BG2_ATTR_TABLE + pos] & 0xf0) == 0xf0;
            }

            if openable {
                let k = (self.ram[DUNG_BG2_ATTR_TABLE + pos] & 0x0f) as usize;
                write_le_u16(&mut self.ram, DUNG_WHICH_KEY_X2_DUNGEON, (k * 2) as u16);

                if (self.ram[DUNG_DOOR_DIRECTION + k * 2] & 3) == dir as u8 {
                    let door_type = self.ram[DOOR_TYPE_AND_SLOT + k * 2] & 0xfe;
                    if door_type == DOOR_TYPE_BREAKABLE_WALL {
                        if self.ram[LINK_IS_RUNNING] != 0 && self.ram[LINK_DASH_CTR] < 63 {
                            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, pos as u16);

                            let db = self.ancilla_add_door_debris();
                            if db >= 0 {
                                let db = db as usize;
                                self.ram[DOOR_DEBRIS_DIRECTION_DUNGEON + db] =
                                    self.ram[DUNG_DOOR_DIRECTION + k * 2] & 3;
                                let addr =
                                    read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + k * 2);
                                let door_x = read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_H_COPY)
                                    .wrapping_add((addr & 0x007e) << 2);
                                let door_y = read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_V_COPY)
                                    .wrapping_add((addr & 0x1f80) >> 4);
                                write_le_u16(&mut self.ram, DOOR_DEBRIS_X + db * 2, door_x);
                                write_le_u16(&mut self.ram, DOOR_DEBRIS_Y + db * 2, door_y);
                            }
                            self.ram[SOUND_EFFECT_2] = 27;
                            self.frame_control_view_mut().set_submodule(9);
                            self.sprite_repel_dash();
                            return;
                        }
                    } else if door_type == DOOR_TYPE_1E {
                        write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
                        write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, pos as u16);
                        let palace = (read_le_u16(&self.ram, CUR_PALACE_INDEX_X2) >> 1) as usize;
                        if read_le_u16(&self.ram, LINK_BIGKEY) & upper_bitmask(palace) != 0 {
                            write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
                            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, pos as u16);
                            self.frame_control_view_mut().set_submodule(4);
                            self.ram[SOUND_EFFECT_2] = 20
                                | OPEN_DOOR_PANNING
                                    [(self.ram[DUNG_DOOR_DIRECTION + k * 2] & 3) as usize];
                            return;
                        }
                        if read_le_u16(&self.ram, BIG_KEY_DOOR_MESSAGE_TRIGGERED_DUNGEON) == 0 {
                            write_le_u16(&mut self.ram, BIG_KEY_DOOR_MESSAGE_TRIGGERED_DUNGEON, 1);
                            write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x007a);
                            self.main_show_text_message();
                        }
                    } else if door_type >= DOOR_TYPE_SMALL_KEY_DOOR
                        && door_type < 0x2c
                        && door_type != 0x2a
                        && self.ram[LINK_NUM_KEYS] != 0
                    {
                        self.ram[LINK_NUM_KEYS] = self.ram[LINK_NUM_KEYS].wrapping_sub(1);
                        write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
                        write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, pos as u16);
                        self.frame_control_view_mut().set_submodule(4);
                        self.ram[SOUND_EFFECT_2] = 20
                            | OPEN_DOOR_PANNING
                                [(self.ram[DUNG_DOOR_DIRECTION + k * 2] & 3) as usize];
                        return;
                    }
                } else {
                    write_le_u16(&mut self.ram, BIG_KEY_DOOR_MESSAGE_TRIGGERED_DUNGEON, 0);
                }
            } else {
                write_le_u16(&mut self.ram, BIG_KEY_DOOR_MESSAGE_TRIGGERED_DUNGEON, 0);
            }
        }

        let invisible = read_le_u16(&self.ram, INVISIBLE_DOOR_DIR_AND_INDEX_X2);
        if invisible & 0x0080 == 0
            && self.ram[IS_STANDING_IN_DOORWAY] == 0
            && (self.player_state_view().x() >> 8) == 0x000c
        {
            let dir = invisible as u8;
            let j = ((invisible >> 8) >> 1) as usize;
            let mut opened = read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT);
            if dir != self.ram[LINK_DIRECTION_FACING]
                && (dir ^ 2) == self.ram[LINK_DIRECTION_FACING]
            {
                opened |= upper_bitmask(j);
            } else {
                opened &= !upper_bitmask(j);
            }
            if opened != read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) {
                write_le_u16(&mut self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, opened);
                self.DrawEyeWatchDoor(j);
                let addr = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + j * 2);
                self.dungeon_prep_overlay_dma_next_prep(0, addr);
                self.Dungeon_LoadToggleDoorAttr_OtherEntry(j as i32);
                self.ram[NMI_COPY_PACKETS_FLAG] = 1;
                self.ram[SOUND_EFFECT_2] = 21;
                return;
            }
        }

        if self.ram[BUTTON_MASK_B_Y] & 0x80 == 0 || self.ram[BUTTON_B_FRAMES] != 4 {
            return;
        }

        let link_y = self
            .player_state_view()
            .y()
            .wrapping_add(self.ram[PLAYER_OAM_Y_OFFSET] as i8 as i16 as u16);
        let link_x = self
            .player_state_view()
            .x()
            .wrapping_add(self.ram[PLAYER_OAM_X_OFFSET] as i8 as i16 as u16);
        let mut pos = (((link_y & 0x01f8) << 3) | ((link_x & 0x01f8) >> 3)) as usize;
        let mut attr = self.ram[DUNG_BG2_ATTR_TABLE + pos] & 0xfc;
        let mut y = 0x41u8;

        if attr != 0x6c && (attr & 0xf0) != 0xf0 {
            pos += 1;
            attr = self.ram[DUNG_BG2_ATTR_TABLE + pos] & 0xfc;
            y = 0x40;
            if attr != 0x6c && (attr & 0xf0) != 0xf0 {
                pos += 63;
                attr = self.ram[DUNG_BG2_ATTR_TABLE + pos] & 0xfc;
                y = 1;
                if attr != 0x6c && (attr & 0xf0) != 0xf0 {
                    pos += 1;
                    attr = self.ram[DUNG_BG2_ATTR_TABLE + pos] & 0xfc;
                    y = 0;
                    if attr != 0x6c && (attr & 0xf0) != 0xf0 {
                        return;
                    }
                }
            }
        }

        let addr;
        if attr == 0x6c {
            if y & 0x40 != 0 {
                pos -= 64;
                if self.ram[DUNG_BG2_ATTR_TABLE + pos] & 0xfc != 0x6c {
                    pos += 64;
                }
            }
            if y & 1 != 0 {
                pos -= 1;
                if self.ram[DUNG_BG2_ATTR_TABLE + pos] & 0xfc != 0x6c {
                    pos += 1;
                }
            }
            attr = self.ram[DUNG_BG2_ATTR_TABLE + pos];
            self.write_attr2(pos + xy(0, 0), 0x0202);
            self.write_attr2(pos + xy(0, 1), 0x0202);
            addr = ((pos - xy(1, 1)) * 2) as u16;
            self.RoomDraw_Object_Nx4_Bg2(4, SRC_TILES_1[(attr & 3) as usize] as usize, addr >> 1);
        } else {
            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_POS_DUNGEON, pos as u16);
            let k = (attr & 0x0f) as usize;
            if self.ram[DOOR_TYPE_AND_SLOT + k * 2] != DOOR_TYPE_SLASHABLE {
                return;
            }
            self.ram[SOUND_EFFECT_2] = 27;
            addr = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + k * 2);
            let opened_adj =
                read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) | upper_bitmask(k);
            let opened = read_le_u16(&self.ram, DUNG_DOOR_OPENED) | upper_bitmask(k);
            write_le_u16(&mut self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, opened_adj);
            write_le_u16(&mut self.ram, DUNG_DOOR_OPENED, opened);
            write_le_u16(&mut self.ram, DOOR_OPEN_CLOSED_COUNTER, 0);
            write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, (k * 2) as u16);
            write_le_u16(&mut self.ram, DUNG_WHICH_KEY_X2_DUNGEON, (k * 2) as u16);
            self.RoomDraw_Object_Nx4_Bg2(4, DOOR_TYPE_SRC_UP[0x56 / 2] as usize, addr >> 1);
            self.Dungeon_LoadToggleDoorAttr_OtherEntry(k as i32);
        }

        self.dungeon_prep_overlay_dma_next_prep(0, addr);
        self.ram[SOUND_EFFECT_1] =
            30 | self.calculate_sfx_pan_arbitrary(((addr & 0x007f) * 2) as u8);
        self.ram[NMI_COPY_PACKETS_FLAG] = 1;
    }

    pub(super) fn dungeon_clear_away_exploding_wall(&mut self) {
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 6;
        self.ram[FLAG_UNK1] = 6;
        if self.ram[MESSAGING_BUF_DUNGEON] != 6 {
            return;
        }

        write_le_u16(&mut self.ram, DUNG_DOOR_BARRIER_OR_SWITCH_FLAG, 0);
        self.ram[R12] = 0;
        write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
        let blast_door_x2 = read_le_u16(&self.ram, CRUSH_WALL_DOOR_INDEX_X2_DUNGEON);
        write_le_u16(&mut self.ram, DUNG_CUR_DOOR_IDX, blast_door_x2);

        let door = usize::from(blast_door_x2 >> 1);
        let addr = read_le_u16(&self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2).wrapping_sub(2);
        write_le_u16(&mut self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2, addr);
        let dsto = usize::from(addr >> 1);

        self.Door_BlastWallExploding_Draw(dsto);
        self.ClearAndStripeExplodingWall(dsto as u16);

        write_le_u16(&mut self.ram, NMI_DISABLE_CORE_UPDATES, 0xffff);
        let walls2 = read_le_u16(&self.ram, CRUSH_WALL_PROGRESS_DUNGEON).wrapping_add(2);
        write_le_u16(&mut self.ram, CRUSH_WALL_PROGRESS_DUNGEON, walls2);

        if walls2 == 21 {
            let mask = upper_bitmask(door);
            let opened_adj = read_le_u16(&self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) | mask;
            let opened = read_le_u16(&self.ram, DUNG_DOOR_OPENED) | mask;
            write_le_u16(&mut self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, opened_adj);
            write_le_u16(&mut self.ram, DUNG_DOOR_OPENED, opened);

            if self.ram[DUNG_DOOR_DIRECTION + door * 2] & 2 != 0 {
                self.ram[DUNG_BLASTWALL_FLAG_X] = 1;
                self.ram[QUADRANT_FULLSIZE_X] = 2;
            } else {
                self.ram[DUNG_BLASTWALL_FLAG_Y] = 1;
                self.ram[QUADRANT_FULLSIZE_Y] = 2;
            }
            let quadrant = read_le_u16(&self.ram, QUADRANT_FULLSIZE_X);
            write_le_u16(&mut self.ram, QUADRANT_FULLSIZE_X_CACHED, quadrant);
            self.Door_LoadBlastWallAttr(door);
            write_le_u16(&mut self.ram, CRUSH_WALL_PROGRESS_DUNGEON, 0);
            write_le_u16(&mut self.ram, CRUSH_WALL_DOOR_INDEX_X2_DUNGEON, 0);
            self.Dungeon_FlagRoomData_Quadrants();
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
            self.ram[FLAG_UNK1] = 0;
        }
        self.ram[NMI_COPY_PACKETS_FLAG] = 3;
    }

    pub(super) fn orient_lamp_light_cone(&mut self) {
        self.OrientLampLightCone();
    }

    pub(super) fn sprite_dungeon_draw_all_push_blocks(&mut self) {
        for i in (0..=1).rev() {
            if self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + i] != 0 {
                self.Sprite_HandlePushedBlocks_One(i);
            }
        }
    }

    pub(super) fn reset_transition_props_and_advance_reset_interface(&mut self) {
        self.ResetTransitionPropsAndAdvance_ResetInterface();
    }

    pub(super) fn reset_transition_props_and_advance_submodule(&mut self) {
        self.ResetTransitionPropsAndAdvanceSubmodule();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_completely_open_door_uses_inter_room_upnorth_count_before_wall_spirals() {
        let mut state = ZeldaState::new();
        let pos = 0x011eusize;

        state.ram[DUNG_BG2_ATTR_TABLE + pos + xy(1, 0)..DUNG_BG2_ATTR_TABLE + pos + xy(1, 3) + 2]
            .fill(0xf0);
        write_le_u16(&mut state.ram, DUNG_INTER_STAIRCASES, pos as u16);

        write_le_u16(&mut state.ram, DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS, 0);
        write_le_u16(&mut state.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS, 2);
        write_le_u16(&mut state.ram, DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2, 2);
        write_le_u16(
            &mut state.ram,
            DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS,
            2,
        );
        write_le_u16(
            &mut state.ram,
            DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS,
            2,
        );
        write_le_u16(&mut state.ram, DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS, 2);
        write_le_u16(&mut state.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS, 2);
        write_le_u16(&mut state.ram, DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2, 2);
        write_le_u16(&mut state.ram, DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS, 2);

        state.DrawCompletelyOpenDoor();

        assert_eq!(
            &state.ram
                [DUNG_BG2_ATTR_TABLE + pos + xy(1, 0)..DUNG_BG2_ATTR_TABLE + pos + xy(1, 0) + 2],
            &[0x5e, 0x5e]
        );
        assert_eq!(
            &state.ram
                [DUNG_BG2_ATTR_TABLE + pos + xy(1, 1)..DUNG_BG2_ATTR_TABLE + pos + xy(1, 1) + 2],
            &[0x30, 0x30]
        );
        assert_eq!(
            &state.ram
                [DUNG_BG2_ATTR_TABLE + pos + xy(1, 2)..DUNG_BG2_ATTR_TABLE + pos + xy(1, 2) + 2],
            &[0x00, 0x00]
        );
        assert_eq!(
            &state.ram
                [DUNG_BG2_ATTR_TABLE + pos + xy(1, 3)..DUNG_BG2_ATTR_TABLE + pos + xy(1, 3) + 2],
            &[0x00, 0x00]
        );
    }

    #[test]
    fn room_draw_all_objects_clears_width_height_before_terminator() {
        let mut state = ZeldaState::new();
        write_le_u16(&mut state.ram, DUNG_DRAW_WIDTH_INDICATOR, 0x4000);
        write_le_u16(&mut state.ram, DUNG_DRAW_HEIGHT_INDICATOR, 0x2000);
        write_le_u16(&mut state.ram, DUNG_LOAD_PTR_OFFS, 0);

        state.RoomData_DrawObjects_from(&[0xff, 0xff]);

        assert_eq!(read_le_u16(&state.ram, DUNG_DRAW_WIDTH_INDICATOR), 0);
        assert_eq!(read_le_u16(&state.ram, DUNG_DRAW_HEIGHT_INDICATOR), 0);
    }

    #[test]
    fn dungeon_load_room_resets_floor_velocity_words() {
        let mut state = ZeldaState::new();
        write_le_u16(&mut state.ram, DUNG_FLOOR_Y_VEL_DUNGEON, 0x0002);
        write_le_u16(&mut state.ram, DUNG_FLOOR_X_VEL, 0x0001);

        state.dungeon_load_room_reset_floor_velocity();

        assert_eq!(read_le_u16(&state.ram, DUNG_FLOOR_Y_VEL_DUNGEON), 0);
        assert_eq!(read_le_u16(&state.ram, DUNG_FLOOR_X_VEL), 0);
    }
}
