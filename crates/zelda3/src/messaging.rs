// Methods ported from zelda3/src/messaging.c and included inside ZeldaState.

use super::*;
use crate::types::{sign16, Pair16U, Point16U};

const TEXT_POSITIONS: [u16; 2] = [0x6125, 0x6244];
const TEXT_INITIALIZATION_DATA: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0x39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1c, 4, 0, 0, 0,
    0, 0,
];
const TEXT_BORDER_TILES: [u16; 9] = [
    0x28f3, 0x28f4, 0x68f3, 0x28c8, 0x387f, 0x68c8, 0xa8f3, 0xa8f4, 0xe8f3,
];
const TEXT_WAIT_DURATIONS: [u16; 16] = [
    31, 63, 94, 125, 156, 188, 219, 250, 281, 313, 344, 375, 406, 438, 469, 500,
];
const VWF_RENDER_CHARACTER_SET_MASKS: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1];
const VWF_RENDER_CHARACTER_RENDER_POS: [u16; 3] = [0, 0x02a0, 0x0540];
const VWF_RENDER_CHARACTER_LINE_POSITIONS: [u16; 3] = [0, 0x0040, 0x0080];
const VWF_ROW_POSITIONS: [u16; 3] = [0, 2, 4];
const TEXT_COMMAND_START_US: u8 = 0x67;
const TEXT_DICT_BASE: u8 = 0x88;
const TEXT_CMD_NEXT_PIC: u8 = 0;
const TEXT_CMD_CHOOSE: u8 = 1;
const TEXT_CMD_ITEM: u8 = 2;
const TEXT_CMD_NAME: u8 = 3;
const TEXT_CMD_WINDOW: u8 = 4;
const TEXT_CMD_NUMBER: u8 = 5;
const TEXT_CMD_POSITION: u8 = 6;
const TEXT_CMD_SCROLL_SPD: u8 = 7;
const TEXT_CMD_SELCHG: u8 = 8;
const TEXT_CMD_CHOOSE3: u8 = 10;
const TEXT_CMD_CHOOSE2: u8 = 11;
const TEXT_CMD_SCROLL: u8 = 12;
const TEXT_CMD_1: u8 = 13;
const TEXT_CMD_2: u8 = 14;
const TEXT_CMD_3: u8 = 15;
const TEXT_CMD_COLOR: u8 = 16;
const TEXT_CMD_WAIT: u8 = 17;
const TEXT_CMD_SOUND: u8 = 18;
const TEXT_CMD_SPEED: u8 = 19;
const TEXT_CMD_WAITKEY: u8 = 23;
const TEXT_CMD_END_MESSAGE: u8 = 24;
const TEXT_CMD_IS_LETTER: u8 = 25;
const DIALOGUE_NUMBER: usize = 0x1cf2;
// NES_Ver2 MAPLEV/MAPSMK/MPLKPX/MPLKPY; call sites show scroll state and player marker position.
const DUNG_CUR_QUADRANT_UPLOAD: usize = 0x045c;
const FEATURE_EXTEND_SCREEN64_MAP: u32 = 1;
const FEATURE_CANCEL_BIRD_TRAVEL: u32 = 8192;
const LOCATION_MENU_START_POSITIONS: [u8; 3] = [0, 1, 6];
const POST_DEATH_HEALTH_BY_CAPACITY: [u8; 21] = [
    0x18, 0x18, 0x18, 0x18, 0x18, 0x20, 0x20, 0x28, 0x28, 0x30, 0x30, 0x38, 0x38, 0x38, 0x40, 0x40,
    0x40, 0x48, 0x48, 0x48, 0x50,
];
const BIRD_TRAVEL_OVERWORLD_SCREEN_BY_STOP: [u8; 8] =
    [0x7f, 0x79, 0x6c, 0x6d, 0x6e, 0x6f, 0x7c, 0x7d];
const BIRD_TRAVEL_X_LOW: [u8; 8] = [0x80, 0xcf, 0x10, 0xb8, 0x30, 0x70, 0x70, 0xf0];
const BIRD_TRAVEL_X_HIGH: [u8; 8] = [6, 0x0c, 2, 8, 0x0f, 0, 7, 0x0e];
const BIRD_TRAVEL_Y_LOW: [u8; 8] = [0x5b, 0x98, 0xc0, 0x20, 0x50, 0xb0, 0x30, 0x80];
const BIRD_TRAVEL_Y_HIGH: [u8; 8] = [3, 5, 7, 0x0b, 0x0b, 0x0f, 0x0f, 0x0f];
const OVERWORLD_MAP_ICON_TILES: [u8; 7] = [0x79, 0x6e, 0x6f, 0x6d, 0x7c, 0x6c, 0x7f];

const DUNGEON_MAP_LEVEL_LABEL_INDEX_BY_DUNGEON: [i8; 14] =
    [-1, -1, -1, -1, -1, 2, 0, 10, 4, 8, -1, 6, 12, 14];
const DUNGEON_MAP_LEVEL_LABEL_TOP_TILES: [u16; 8] = [
    0x2108, 0x2109, 0x2109, 0x210a, 0x210b, 0x210c, 0x210d, 0x211d,
];
const DUNGEON_MAP_LEVEL_LABEL_BOTTOM_TILES: [u16; 8] = [
    0x2118, 0x2119, 0xa109, 0x211a, 0x211b, 0x211c, 0x2118, 0xa11d,
];
const DUNGEON_MAP_LEVEL_LABEL_TOP_STRIPE: [u8; 14] = [
    0x60, 0x84, 0, 0x0b, 0x32, 0x21, 0x33, 0x21, 0x38, 0x21, 0x3a, 0x21, 0x7f, 0x20,
];
const DUNGEON_MAP_LEVEL_LABEL_BOTTOM_STRIPE: [u8; 14] = [
    0x60, 0xa4, 0, 0x0b, 0x42, 0x21, 0x43, 0x21, 0x49, 0x21, 0x4a, 0x21, 0x7f, 0x20,
];
const DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON: [u16; 14] = [
    0x21, 0x23, 0x20, 0x21, 0x70, 0x12, 0x11, 0x212, 2, 0x217, 0x160, 0x12, 0x113, 0x171,
];
const DUNGEON_MAP_FLOOR_LIST_HEADER_STRIPE: [u16; 21] = [
    0xaa10, 0x0100, 0x1b2f, 0xc910, 0x0300, 0x1b2f, 0x1b2e, 0xe510, 0x0b00, 0x1b2f, 0x1b2e, 0x5b2f,
    0x1b2f, 0x1b2e, 0x1b2e, 0x0311, 0x0100, 0x1b2f, 0x0411, 0x0c40, 0x1b2e,
];
const DUNGEON_MAP_FLOOR_LIST_VRAM_STARTS: [u16; 9] = [
    0x1223, 0x1263, 0x12a3, 0x12e3, 0x1323, 0x11e3, 0x11a3, 0x1163, 0x1123,
];
const DUNGEON_MAP_FLOOR_LIST_LABEL_TILES: [u16; 7] =
    [0x1b28, 0x1b29, 0x1b2a, 0x1b2b, 0x1b2c, 0x1b2d, 0x1b2e];
const DUNGEON_MAP_FLOOR_LIST_BOX_TILES: [u16; 8] = [
    0x0f26, 0x0f27, 0x4f27, 0x4f26, 0x8f26, 0x8f27, 0xcf27, 0xcf26,
];
const DUNGEON_MAP_ROOM_BORDER_CORNER_POSITIONS: [u16; 4] = [0x00e2, 0x00f8, 0x03a2, 0x03b8];
const DUNGEON_MAP_ROOM_BORDER_CORNER_TILES: [u16; 4] = [0x1f19, 0x5f19, 0x9f19, 0xdf19];
const DUNGEON_MAP_ROOM_BORDER_HORIZONTAL_POSITIONS: [u16; 2] = [0x00e4, 0x03a4];
const DUNGEON_MAP_ROOM_BORDER_HORIZONTAL_TILES: [u16; 2] = [0x1f1a, 0x9f1a];
const DUNGEON_MAP_ROOM_BORDER_VERTICAL_POSITIONS: [u16; 2] = [0x0122, 0x0138];
const DUNGEON_MAP_ROOM_BORDER_VERTICAL_TILES: [u16; 2] = [0x1f1b, 0x5f1b];
const DUNGEON_MAP_FLOOR_NUMBER_TILES: [u16; 8] = [
    0x1f1e, 0x1f1f, 0x1f20, 0x1f21, 0x1f22, 0x1f23, 0x1f24, 0x1f25,
];
const DUNGEON_MAP_ROOM_QUADRANT_TILES: &[u16] = &[
    0xb61, 0x5361, 0x8b61, 0x8b62, 0xb60, 0xb63, 0x8b60, 0xb64, 0xb00, 0xb00, 0xb65, 0xb66, 0xb67,
    0x4b67, 0x9367, 0xd367, 0xb60, 0x5360, 0x8b60, 0xcb60, 0xb6a, 0x4b6a, 0x4b6d, 0xb6d, 0x1368,
    0x1369, 0xb00, 0xb00, 0xb6a, 0x136b, 0xb6c, 0xb6d, 0x136e, 0x4b6e, 0xb00, 0xb00, 0x136f, 0xb00,
    0xb00, 0xb00, 0x1340, 0xb00, 0xb78, 0x1744, 0x536d, 0x136d, 0x4b76, 0xb76, 0xb70, 0xb71, 0xb72,
    0x8b71, 0xb75, 0xb76, 0x8b75, 0x8b76, 0xb00, 0xb53, 0xb00, 0xb55, 0x1354, 0x5354, 0xb00, 0xb00,
    0x4b53, 0xb00, 0xb56, 0xb57, 0xb00, 0xb59, 0xb00, 0x135e, 0x135a, 0x135b, 0x135f, 0x535f,
    0xb5c, 0xb5d, 0x535e, 0xcb58, 0xb50, 0x4b50, 0x1352, 0x5352, 0xb00, 0xb40, 0x1345, 0xb46,
    0x8b42, 0xb47, 0xb42, 0xb49, 0x1348, 0x5348, 0x174a, 0x574a, 0x4b47, 0xcb42, 0x4b49, 0x4b42,
    0xb00, 0xb4b, 0xb00, 0xb4d, 0xb4c, 0x4b4c, 0xb4e, 0x4b4e, 0xb51, 0xb44, 0xb00, 0xb00, 0xb4f,
    0x4b4f, 0x934f, 0xd34f, 0xb00, 0xb00, 0xb00, 0xb40, 0xb00, 0xb41, 0xb00, 0xb42, 0xb00, 0xb00,
    0xb43, 0xb43, 0xb00, 0xb00, 0x9344, 0xb00, 0x1340, 0xb00, 0x1341, 0xb00, 0x1740, 0xb40, 0xb42,
    0xb7d, 0x4b7a, 0xb7a, 0xb7e, 0x4b7e, 0xb40, 0x8b4d, 0x4bba, 0xb55, 0xb40, 0x8b55, 0x1378,
    0xcb53, 0x4b76, 0x4b75, 0x13bb, 0x53bb, 0x4b7f, 0x4b42, 0xb83, 0x13bc, 0xb00, 0xb00, 0xb79,
    0xb00, 0xb6e, 0x4b7c, 0xb00, 0xb41, 0x1340, 0x8b55, 0xb42, 0xb7b, 0x8b42, 0x9344, 0x1341,
    0xb00, 0xb53, 0x9344, 0x8b53, 0x9344, 0x8b42, 0x9344, 0xb42, 0x9344, 0x934d, 0xb00, 0x8b53,
    0x9344, 0xb00, 0xb00, 0xb40, 0xb00, 0xb41, 0xb00, 0x1384, 0xb00, 0xbb8, 0x13b9, 0x4b85, 0xcb7c,
    0xb87, 0x13b0, 0x4b7b, 0x9344, 0xb00, 0xb00, 0xb40, 0xb00, 0xb91, 0x5391, 0xb9c, 0x4b9c,
    0x8b42, 0x1392, 0xb93, 0x1394, 0xb95, 0xb96, 0x9395, 0x8b96, 0xb97, 0xb98, 0x8b97, 0x8b98,
    0x1799, 0x5799, 0x9799, 0xd799, 0x4b98, 0x4b97, 0xcb98, 0xcb97, 0x937b, 0xb00, 0xb7b, 0xb00,
    0xba6, 0x4ba6, 0xcb7a, 0x8b7a, 0xb8e, 0x4b8e, 0x938e, 0xcb8e, 0x934d, 0xb8f, 0x1390, 0x5390,
    0xb00, 0xb00, 0xb00, 0x8b48, 0xb00, 0x934e, 0xb00, 0x8b4d, 0x8b72, 0x1346, 0xb45, 0xb46,
    0x5744, 0x1744, 0xb00, 0xb00, 0x134d, 0xb00, 0x8b54, 0xb00, 0x1349, 0x1349, 0xb00, 0xb00,
    0xb4b, 0x8b48, 0xb72, 0x4b72, 0xb00, 0xb74, 0xb00, 0xbb0, 0xb71, 0x1747, 0x17af, 0xb4b, 0xb6f,
    0x1370, 0xb4b, 0xb00, 0xb6b, 0x8b6c, 0x8b6b, 0xbad, 0xb73, 0xb00, 0x13ae, 0xb46, 0x176b,
    0x576b, 0xb6a, 0x4b6a, 0x1368, 0x5368, 0x1369, 0x5369, 0x8b4e, 0xb00, 0x9354, 0xb00, 0xb00,
    0xb00, 0xb00, 0x5377, 0xb00, 0x974d, 0xb00, 0x4b7b, 0xb40, 0x8b4d, 0xb51, 0xb8d, 0x537a,
    0x137a, 0x4b42, 0x8b40, 0xb00, 0xb00, 0xb00, 0xb00, 0xb00, 0xb00, 0xb40, 0xb00, 0xcb7a, 0x576e,
    0xb00, 0xb00, 0xb6e, 0xb9f, 0xb00, 0x4ba5, 0x13a0, 0x13a1, 0xba2, 0xba3, 0xba4, 0xb00, 0xba5,
    0xb00, 0xb40, 0x8b55, 0xb42, 0xcb87, 0x8b95, 0xba7, 0x8b42, 0xbaf, 0x4b78, 0xb00, 0x4b78,
    0xb00, 0x8b42, 0xb51, 0xb78, 0x8b51, 0xba8, 0xba9, 0xbac, 0x8ba9, 0xbaa, 0x17ab, 0x13b4,
    0x8bab, 0x17b1, 0xb41, 0x4b44, 0x4b42, 0xb00, 0xbad, 0xb00, 0x13ae, 0x1340, 0xbb7, 0xb42,
    0xbb6, 0xb00, 0xb00, 0x139d, 0x139e, 0xb00, 0xb00, 0xb00, 0xb79, 0xb00, 0xb00, 0x8b42, 0xb86,
    0xb42, 0x8b7b, 0x8b42, 0xb7b, 0xb87, 0x8b7b, 0x9387, 0xb7b, 0xb40, 0x13b3, 0x1378, 0xb8d,
    0x8b42, 0xb88, 0x5378, 0xb40, 0x4b44, 0xd342, 0x97b5, 0x4b78, 0x13b3, 0x8b55, 0x4b7b, 0xb8d,
    0xb89, 0x138a, 0xb8b, 0xb8c, 0xb00, 0xb7c, 0xb00, 0xb00, 0xb00, 0x9348, 0xb00, 0xb56, 0xb00,
    0xb00, 0xb88, 0xb00, 0xb00, 0xb48, 0xb00, 0xb00, 0xb00, 0x9348, 0x1786, 0xb65, 0xb00, 0xb00,
    0xcb5a, 0xb00, 0xb00, 0x5388, 0xb00, 0xb00, 0x4b5a, 0xb00, 0xb00, 0xb00, 0xb00, 0xcb5b, 0x13ab,
    0xbac, 0xcb5a, 0xb00, 0x137e, 0xb00, 0xb00, 0x137e, 0xb00, 0xb00, 0xb00, 0x8b48, 0x1783,
    0x1384, 0xb00, 0xb00, 0x1385, 0xb00, 0xb00, 0x537e, 0xb00, 0xb00, 0xb00, 0x8b48, 0xb43, 0xcb43,
    0xb00, 0xb00, 0x1379, 0x137a, 0xb5a, 0x137b, 0xb00, 0xb00, 0xb00, 0x8b48, 0x137f, 0x1380,
    0xb00, 0xb00, 0x1381, 0x1382, 0xb00, 0xb48, 0xb00, 0xb00, 0xb00, 0xb00, 0x1387, 0x1377, 0x5746,
    0xb47, 0x1349, 0xb48, 0x1375, 0x4b42, 0x174a, 0x574a, 0xb43, 0x1344, 0xb45, 0x1746, 0x1742,
    0x5742, 0x8b42, 0xcb42, 0x1375, 0x5375, 0x8b42, 0xcb42, 0x4b40, 0x1340, 0xb41, 0x4b41, 0x4b46,
    0xb71, 0x1786, 0x8b71, 0x1347, 0xb4d, 0xb65, 0xb5b, 0xb00, 0xb00, 0x9348, 0xb00, 0xb00, 0xb00,
    0xb00, 0x8b48, 0x4b66, 0x8b65, 0x4b5b, 0xb65, 0x9365, 0xb66, 0xb63, 0x8b66, 0x4b51, 0xb5f,
    0xcb76, 0xb60, 0xb64, 0x4b4f, 0x4b60, 0x8b76, 0x4b76, 0xb61, 0xd376, 0x1362, 0x4b61, 0xb76,
    0xcb58, 0x8b51, 0xb00, 0xb00, 0x5746, 0xb5e, 0xb00, 0xb00, 0xb5e, 0xb46, 0xb00, 0xb00, 0x8b48,
    0xb00, 0xb4f, 0xb51, 0xcb76, 0x8b76, 0x5351, 0xb51, 0x8b4f, 0x8b51, 0x4b76, 0xb76, 0xcb51,
    0x8b58, 0xb54, 0xb00, 0x8b66, 0xb00, 0x9348, 0x8b48, 0xb56, 0x4b45, 0xb00, 0xb57, 0xb00, 0xb59,
    0x4b50, 0xb58, 0xcb50, 0x8b50, 0x5758, 0x1751, 0xcb58, 0x8b51, 0xb56, 0x4b56, 0xb65, 0x5756,
    0x9348, 0x8b48, 0xb4c, 0xb4b, 0xb4d, 0xb00, 0x8b54, 0xb00, 0xb4f, 0xb50, 0x8b4f, 0x8b50,
    0x4b50, 0xb51, 0xcb58, 0x8b51, 0xb52, 0xb54, 0xb53, 0x9354, 0x9748, 0x9748, 0x138d, 0x138e,
    0x1391, 0x1392, 0x138c, 0x138f, 0x1393, 0x1390, 0x9393, 0x138f, 0x1394, 0x1395, 0x138e, 0x138c,
    0x175d, 0x1399, 0x975d, 0x538f, 0x1397, 0x1398, 0x179a, 0x138c, 0x1399, 0x1766, 0x138f, 0xd75d,
    0x538e, 0x538f, 0x1391, 0x1392, 0x139b, 0x539b, 0x139c, 0x539c, 0x138f, 0x138e, 0x5392, 0x5391,
    0x138a, 0x538a, 0x138b, 0x538b, 0xb00, 0xcb5b, 0xb00, 0x8b54, 0x4b74, 0x13a6, 0xb00, 0x4b48,
    0x13a0, 0x13a1, 0x538e, 0x138e, 0xd38e, 0x53a3, 0x13a4, 0xb00, 0x97aa, 0xb00, 0x538e, 0x1399,
    0x13a4, 0xb00, 0x138e, 0xb00, 0xb00, 0x5393, 0xb00, 0x574e, 0x4b7d, 0xb00, 0x8b7d, 0x139f,
    0x97aa, 0x13a4, 0x13a9, 0x53a9, 0x13a5, 0x13a6, 0x93a5, 0xd3a5, 0xd38e, 0x938e, 0x13a4, 0x13aa,
    0xb00, 0x13a6, 0xb00, 0x8b5f, 0x139b, 0x13a6, 0x139c, 0x53a2, 0xb00, 0xb00, 0x138c, 0xb00,
    0x9394, 0x139e, 0xb00, 0xb00,
];
const DUNGEON_MAP_ROOM_REMAP_FROM: [u16; 3] = [137, 167, 79];
const DUNGEON_MAP_ROOM_REMAP_TO: [u16; 3] = [169, 119, 190];
const DUNGEON_MAP_MARKER_Y_BASES: [u16; 2] = [0x1f, 0x7f];
const DUNGEON_MAP_BOSS_ROOM_BY_DUNGEON: [u8; 14] =
    [15, 15, 200, 51, 32, 6, 90, 144, 41, 222, 7, 172, 164, 13];
const DUNGEON_MAP_FLOOR_SCROLL_TARGET_DELTAS: [i16; 2] = [0x60, -0x60];
const DUNGEON_MAP_BOSS_FLOOR_OFFSETS: [i16; 14] = [
    -1, -1, 1, 1, 6, 0xff, 0xff, 0xff, 0xfe, 0xf9, 5, 0xff, 0xfd, 6,
];
const DUNGEON_MAP_LOCATION_MARKER_X_OFFSETS: [i8; 4] = [-9, 8, -9, 8];
const DUNGEON_MAP_LOCATION_MARKER_Y_OFFSETS: [i8; 4] = [-8, -8, 9, 9];
const DUNGEON_MAP_LOCATION_MARKER_OAM_FLAGS: [u8; 4] = [0xf1, 0xb1, 0x71, 0x31];
const DUNGEON_MAP_LOCATION_MARKER_CHARS: [u8; 4] = [0x0c, 0x0c, 8, 0x0a];
const DUNGEON_MAP_FLOOR_Y_POSITIONS: [u8; 8] = [187, 171, 155, 139, 123, 107, 91, 75];
const DUNGEON_MAP_FLOOR_DIGIT_CHARS: [u8; 8] = [0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25];
const DUNGEON_MAP_FLOOR_BLINKER_SPRITE_OFFSETS: [u8; 2] = [0, 8];
const DUNGEON_MAP_FLOOR_BLINKER_CHARS: [u8; 4] = [0x37, 0x38, 0x38, 0x37];
const DUNGEON_MAP_BOSS_ICON_XY_BY_DUNGEON: [u16; 14] = [
    0xffff, 0xffff, 0x0808, 8, 0, 8, 0x0808, 8, 0x0808, 0x0800, 0x0404, 0x0808, 8, 8,
];
const DUNGEON_MAP_PLAYER_MARKER_OAM_FLAGS: [u8; 4] = [0x39, 0x3b, 0x3d, 0x3b];
const DUNGEON_MAP_SCROLL_MARKER_Y_DELTAS: [i8; 2] = [-4, 4];
const DUNGEON_MAP_SCROLL_BG_Y_DELTAS: [i8; 2] = [4, -4];
const DUNG_MAP_UPPER_BITMASKS: [u16; 16] = [
    0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020, 0x0010,
    0x0008, 0x0004, 0x0002, 0x0001,
];

const DEATH_ANIM_CTR0: [u8; 15] = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 5];
const DEATH_ANIM_CTR1: [u8; 15] = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 0x62];
const DEATH_SPR_FLAGS: [u8; 2] = [0x20, 0x10];
const DEATH_SPR_CHAR0: [u8; 2] = [0xea, 0xec];
const DEATH_SPR_Y0: [u8; 3] = [0x7f, 0x8f, 0x9f];

fn text_decode_cmd(a: u8, src: *const u8) -> u32 {
    if a < TEXT_COMMAND_START_US || a >= 0x80 {
        let param = if a >= 0x80 { 26 } else { a };
        return ((param as u32) << 16) | ((TEXT_CMD_IS_LETTER as u32) << 8);
    }
    const COMMAND_LENGTHS_US: [u8; 25] = [
        0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0,
    ];
    let cmd = a - TEXT_COMMAND_START_US;
    let param = if COMMAND_LENGTHS_US[cmd as usize] != 0 {
        unsafe { src.as_ref().copied().unwrap_or(0) }
    } else {
        0
    };
    ((param as u32) << 16) | ((cmd as u32) << 8)
}

impl ZeldaState {
    pub(super) fn Module0E_Interface(&mut self) {
        let mut skip_run = false;
        if self.game_state.world.location.is_indoors() {
            if self.game_state.frame.submodule == 3 {
                skip_run = self.overworld_map_state() != 0 && self.overworld_map_state() != 7;
            } else {
                self.dungeon_push_block_handler();
            }
        } else {
            skip_run = (self.game_state.frame.submodule == 7
                || self.game_state.frame.submodule == 10)
                && self.overworld_map_state() != 0;
        }
        if !skip_run {
            self.sprite_main();
            self.link_oam_main();
            if self.game_state.world.location.is_outdoors() {
                self.OverworldOverlay_HandleRain();
            }
            self.hud_refill_logic();
            if self.game_state.frame.submodule != 2 {
                self.orient_lamp_light_cone();
            }
        }
        self.replay_trace_ram_watch("module0e-before-run-interface");
        self.RunInterface();
        self.replay_trace_ram_watch("module0e-after-run-interface");
        let bg1_x_offset = self.game_state.world.scroll.bg1_x_offset();
        let bg1_y_offset = self.game_state.world.scroll.bg1_y_offset();
        let bg2x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_add(bg1_x_offset);
        let bg2y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add(bg1_y_offset);
        let bg1x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg1_h_copy2()
            .wrapping_add(bg1_x_offset);
        let bg1y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg1_v_copy2()
            .wrapping_add(bg1_y_offset);
        self.set_bg2_h_copy(bg2x);
        self.set_bg2_v_copy(bg2y);
        self.set_bg1_h_copy(bg1x);
        self.set_bg1_v_copy(bg1y);
        self.replay_trace_ram_watch("module0e-after-scroll-copy");
    }

    pub(super) fn Module_Messaging_0(&mut self) {
        // C Module_Messaging_0 is an assert(0) dispatch slot.
        panic!("Module_Messaging_0 hit unsupported C assert(0) path");
    }

    pub(super) fn RunInterface(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.Module_Messaging_0(),
            1 => self.hud_module_run(),
            2 => self.RenderText(),
            3 => self.Module0E_03_DungeonMap(),
            4 => self.Module0E_04_RedPotion(),
            5 => self.Module0E_05_DesertPrayer(),
            6 => self.Module_Messaging_6(),
            7 => self.Messaging_OverworldMap(),
            8 => self.Module0E_08_GreenPotion(),
            9 => self.Module0E_09_BluePotion(),
            10 => self.Module0E_0A_FluteMenu(),
            11 => self.Module0E_0B_SaveMenu(),
            // Master Sword item receipt sets submodule 43. C dispatches through
            // the 12-entry messaging table without bounds checks, landing on
            // kModule_BossVictory[3] in the adjacent static table.
            43 => self.dungeon_close_victory_spin(),
            // C indexes kMessagingSubmodules directly; this is the Rust
            // bounds guard for the same dispatch table.
            submodule => panic!("RunInterface invalid submodule {submodule}"),
        }
    }

    pub(super) fn Module_Messaging_6(&mut self) {
        // C Module_Messaging_6 is an assert(0) dispatch slot.
        panic!("Module_Messaging_6 hit unsupported C assert(0) path");
    }

    pub(super) fn GetDungmapFloorLayout(&self) -> Vec<u8> {
        let idx = (self.game_state.inventory.save_progress.palace_index_x2() >> 1) as usize;
        self.asset_memblk(97, idx)
            .map(|blk| blk.ptr.to_vec())
            .unwrap_or_default()
    }

    pub(super) fn GetOtherDungmapInfo(&self, count: usize) -> u8 {
        let idx = (self.game_state.inventory.save_progress.palace_index_x2() >> 1) as usize;
        self.asset_memblk(98, idx)
            .and_then(|blk| blk.ptr.get(count).copied())
            .unwrap_or(0)
    }

    pub(super) fn GetLightOverworldTilemap(&self) -> Vec<u8> {
        self.asset_raw(67)
            .map(|tilemap| tilemap.to_vec())
            .unwrap_or_default()
    }

    pub(super) fn Module0E_05_DesertPrayer(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => self.ResetTransitionPropsAndAdvance_ResetInterface(),
            1 => self.ApplyPaletteFilter_bounce(),
            2 => {
                self.DesertPrayer_InitializeIrisHDMA();
                let countdown = self.game_state.display.mosaic_target_level.wrapping_sub(1);
                self.set_countdown(countdown);
                self.clear_mosaic_target_level();
                self.set_darkening_or_lightening_screen(2);
            }
            3 => {
                self.ApplyPaletteFilter_bounce();
                self.DesertPrayer_BuildIrisHDMATable();
            }
            4 => self.DesertPrayer_BuildIrisHDMATable(),
            _ => {}
        }
    }

    pub(super) fn Module0E_04_RedPotion(&mut self) {
        if self.hud_refill_health() {
            self.finish_potion_refill();
        }
    }

    pub(super) fn Module0E_08_GreenPotion(&mut self) {
        if self.hud_refill_magic_power() {
            self.finish_potion_refill();
        }
    }

    pub(super) fn Module0E_09_BluePotion(&mut self) {
        if self.hud_refill_health() {
            self.set_submodule(8);
        }
        if self.hud_refill_magic_power() {
            self.set_submodule(4);
        }
    }

    fn finish_potion_refill(&mut self) {
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.increment_hud_update_flag();
        self.set_submodule(0);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
    }

    pub(super) fn Module0E_0B_SaveMenu(&mut self) {
        if self.game_state.world.location.is_outdoors() {
            self.Overworld_DwDeathMountainPaletteAnimation();
        }
        self.RenderText();
        self.clear_hud_update_flag();
        self.clear_core_update_disable_flag();
        if self.game_state.frame.subsubmodule < 3 {
            self.increment_subsubmodule();
        } else {
            self.clear_bg_vram_load_mode();
        }
        if self.game_state.frame.submodule == 0 {
            self.set_subsubmodule(0);
            self.set_bg_vram_load_mode(1);
            if self.multiselect_choice().value() != 0 {
                self.set_ambient_sound_effect(15);
                self.set_main_module(23);
                self.set_submodule(1);
                self.dungeon_object_tracking_mut()
                    .clear_changeable_object_index(0);
                self.dungeon_object_tracking_mut()
                    .clear_changeable_object_index(1);
            } else {
                self.multiselect_choice_mut().restore_backup();
            }
        }
    }

    pub(super) fn Module1B_SpawnSelect(&mut self) {
        self.RenderText();
        if self.game_state.frame.submodule != 0 {
            return;
        }
        self.clear_bg_vram_load_mode();
        self.EnableForceBlank();
        self.EraseTileMaps_normal();
        let bak = self
            .game_state
            .inventory
            .save_progress
            .which_starting_point();
        let choice = self.multiselect_choice().value();
        self.save_progress_mut()
            .set_which_starting_point(LOCATION_MENU_START_POSITIONS[choice as usize]);
        self.set_subsubmodule(0);
        self.load_dungeon_room_rebuild_hud();
        self.save_progress_mut().set_which_starting_point(bak);
    }

    pub(super) fn CleanUpAndPrepDesertPrayerHDMA(&mut self) {
        self.hdma_setup(0, 0x02c80c, 0x41, 0, 0x26, 0);
        let main_layers = self.game_state.display.main_screen_layers;
        let sub_layers = self.game_state.display.sub_screen_layers;
        self.set_window_layer_masks(0x33, 3, 0x33, main_layers, sub_layers);
        self.set_hdma_enable_mask(0x80);
        self.clear_spotlight_hdma_table_dynamic(240);
    }

    pub(super) fn DesertPrayer_InitializeIrisHDMA(&mut self) {
        self.CleanUpAndPrepDesertPrayerHDMA();
        self.set_spotlight_window_radius_byte(0x26);
        self.set_spotlight_window_state_byte(0);
        self.DesertPrayer_BuildIrisHDMATable();
        self.increment_subsubmodule();
    }

    pub(super) fn DesertPrayer_BuildIrisHDMATable(&mut self) {
        let r14 = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
            .wrapping_add(12);
        let radius = self.game_state.display.spotlight_hdma.window_radius_byte();
        let mut spotlight_y_lower = r14.wrapping_sub(u16::from(radius));
        self.set_spotlight_y_lower(spotlight_y_lower);
        let mut r4 = if sign16(spotlight_y_lower) {
            spotlight_y_lower
        } else {
            0
        };
        let spotlight_y_upper = spotlight_y_lower.wrapping_add(u16::from(radius) * 2);
        self.set_spotlight_y_upper(spotlight_y_upper);
        let spotlight_x_center = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
            .wrapping_add(8);
        self.set_spotlight_window_x_center(spotlight_x_center);
        self.set_spotlight_window_y_buffer_byte(1);

        loop {
            let mut r0 = 0x0100u16;
            let mut r2 = 0x0100u16;
            let in_window =
                sign16(spotlight_y_lower) || (r4 >= spotlight_y_lower && r4 < spotlight_y_upper);
            let radius = self.game_state.display.spotlight_hdma.window_radius_byte();
            let y_buffer = self
                .game_state
                .display
                .spotlight_hdma
                .window_y_buffer_byte();
            let k = if !in_window {
                r4.wrapping_sub(1)
            } else if radius < y_buffer {
                self.set_spotlight_window_y_buffer_byte(1);
                spotlight_y_lower = 0;
                self.set_spotlight_y_lower(0);
                r4 = spotlight_y_upper;
                if r4 >= 225 {
                    break;
                }
                r4.wrapping_sub(1)
            } else {
                let pair = self.DesertHDMA_CalculateIrisShapeLine();
                if pair.a == 0 {
                    spotlight_y_lower = 0;
                    self.set_spotlight_y_lower(0);
                } else {
                    r2 = spotlight_x_center.wrapping_add(pair.b);
                    r0 = spotlight_x_center.wrapping_sub(pair.b);
                }
                r14.wrapping_sub(u16::from(
                    self.game_state
                        .display
                        .spotlight_hdma
                        .window_y_buffer_byte(),
                ))
                .wrapping_sub(1)
            };

            let t6 = if r0 < 256 {
                r0 as u8
            } else if r0 < 512 {
                255
            } else {
                0
            };
            let t7 = if r2 < 256 { r2 as u8 } else { 255 };
            let r6 = (u16::from(t7) << 8) | u16::from(t6);
            if k < 240 {
                self.set_spotlight_hdma_table_dynamic_entry(
                    k as usize,
                    if r6 == 0xffff { 0x00ff } else { r6 },
                );
            }

            if sign16(spotlight_y_lower) || (r4 >= spotlight_y_lower && r4 < spotlight_y_upper) {
                let k = u16::from(
                    self.game_state
                        .display
                        .spotlight_hdma
                        .window_y_buffer_byte(),
                )
                .wrapping_sub(2)
                .wrapping_add(r14);
                if k < 240 {
                    self.set_spotlight_hdma_table_dynamic_entry(
                        k as usize,
                        if r6 == 0xffff { 0x00ff } else { r6 },
                    );
                }
                self.increment_spotlight_window_y_buffer_byte();
            }

            r4 = r4.wrapping_add(1);
            if !sign16(r4) && r4 >= 225 {
                break;
            }
        }

        if self.game_state.frame.subsubmodule != 4 {
            return;
        }
        if self.game_state.display.spotlight_hdma.window_state_byte() != 1
            && (self.game_state.player.follower_link.filtered_joypad_h()
                | self.game_state.player.follower_link.filtered_joypad_l())
                & 0xc0
                != 0
        {
            self.set_spotlight_window_state_byte(1);
            self.shr_spotlight_window_radius_byte(1);
        }
        if self.game_state.display.spotlight_hdma.window_state_byte() != 0 {
            self.add_spotlight_window_radius_byte(8);
            if self.game_state.display.spotlight_hdma.window_radius_byte() >= 0xc0 {
                self.messaging_state_mut()
                    .xor_message_or_sprite_state_cache(1);
                self.set_music_control(0xf3);
                self.set_ambient_sound_effect(0);
                self.clear_modal_pause_flag();
                self.follower_link_state_mut().set_y_button_action_step(0);
                self.follower_link_state_mut().set_button_mask_b_y(0);
                self.follower_link_state_mut().clear_state_bits();
                self.follower_link_state_mut().clear_direction_lock_bits(1);
                self.set_subsubmodule(0);
                self.set_submodule(0);
                let saved_module = self.game_state.frame.saved_module_for_menu;
                self.set_main_module(saved_module);
                self.clear_window_layer_masks();
                self.IrisSpotlight_ResetTable();
                return;
            }
        }

        const PRAYING_SCENE_DELAYS: [u8; 5] = [22, 22, 22, 64, 1];
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            .is_negative()
        {
            let i = self
                .game_state
                .player
                .follower_link
                .y_button_action_step()
                .wrapping_add(1);
            if i != 4 {
                self.follower_link_state_mut().set_y_button_action_step(i);
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(PRAYING_SCENE_DELAYS[i as usize]);
        }
    }

    pub(super) fn DesertHDMA_CalculateIrisShapeLine(&self) -> Pair16U {
        const PRAYING_IRIS_OPEN_RADIUS_LOOKUP: [u8; 129] = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xfe,
            0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0xfc, 0xfc, 0xfc, 0xfb, 0xfb, 0xfb, 0xfa, 0xfa,
            0xf9, 0xf9, 0xf8, 0xf8, 0xf7, 0xf7, 0xf6, 0xf6, 0xf5, 0xf5, 0xf4, 0xf3, 0xf3, 0xf2,
            0xf1, 0xf1, 0xf0, 0xef, 0xee, 0xee, 0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe9, 0xe8, 0xe7,
            0xe6, 0xe5, 0xe4, 0xe3, 0xe2, 0xe1, 0xdf, 0xde, 0xdd, 0xdc, 0xdb, 0xda, 0xd8, 0xd7,
            0xd6, 0xd5, 0xd3, 0xd2, 0xd0, 0xcf, 0xcd, 0xcc, 0xca, 0xc9, 0xc7, 0xc6, 0xc4, 0xc2,
            0xc1, 0xbf, 0xbd, 0xbb, 0xb9, 0xb7, 0xb6, 0xb4, 0xb1, 0xaf, 0xad, 0xab, 0xa9, 0xa7,
            0xa4, 0xa2, 0x9f, 0x9d, 0x9a, 0x97, 0x95, 0x92, 0x8f, 0x8c, 0x89, 0x86, 0x82, 0x7f,
            0x7b, 0x78, 0x74, 0x70, 0x6c, 0x67, 0x63, 0x5e, 0x59, 0x53, 0x4d, 0x46, 0x3f, 0x37,
            0x2d, 0x1f, 0,
        ];
        const PRAYING_IRIS_CLOSED_RADIUS_LOOKUP: [u8; 129] = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xfe, 0xfd, 0xfd, 0xfc, 0xfc, 0xfb, 0xfa,
            0xf9, 0xf8, 0xf7, 0xf6, 0xf5, 0xf4, 0xf3, 0xf1, 0xf0, 0xee, 0xed, 0xeb, 0xe9, 0xe8,
            0xe6, 0xe4, 0xe2, 0xdf, 0xdd, 0xdb, 0xd8, 0xd6, 0xd3, 0xd0, 0xcd, 0xca, 0xc7, 0xc4,
            0xc1, 0xbd, 0xb9, 0xb6, 0xb1, 0xad, 0xa9, 0xa4, 0x9f, 0x9a, 0x95, 0x8f, 0x89, 0x82,
            0x7b, 0x74, 0x6c, 0x63, 0x59, 0x4d, 0x3f, 0x2d, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let y_buffer = self
            .game_state
            .display
            .spotlight_hdma
            .window_y_buffer_byte();
        let radius = self
            .game_state
            .display
            .spotlight_hdma
            .window_radius_byte()
            .max(1);
        let t = (self.snes_divide(u16::from(y_buffer) << 8, radius) >> 1) as usize;
        let r6 = if self.game_state.display.spotlight_hdma.window_state_byte() != 0 {
            PRAYING_IRIS_OPEN_RADIUS_LOOKUP[t.min(128)]
        } else {
            PRAYING_IRIS_CLOSED_RADIUS_LOOKUP[t.min(128)]
        };
        let mut r8 = (u16::from(r6) * u16::from(radius)) >> 8;
        if self.game_state.display.spotlight_hdma.window_state_byte() != 0 {
            r8 <<= 1;
        }
        Pair16U {
            a: u16::from(r6),
            b: r8,
        }
    }

    pub(super) fn OverworldMap_SetupHdma(&mut self) {
        const HDMA_ADDR: [u32; 2] = [0x0abdcf, 0x0abdd6];
        let addr = HDMA_ADDR[self.overworld_map_flags() as usize];
        self.hdma_setup(addr, addr, 0x42, 0x1b, 0x1e, 10);
    }

    pub(super) fn SaveGameFile(&mut self) {
        let offs = self.selected_save_slot_offset();
        // C copies the LIVE save block from WRAM (ram[SAVE_DUNG_INFO..+0x500]) and
        // checksums it from WRAM. The SaveProgress native model keeps a `dungeon_info`
        // shadow of this block, but that shadow goes stale for bytes owned by OTHER
        // native states (e.g. LINK_HEALTH_CURRENT 0xf36d, owned by player_resources,
        // whose set_current_health write-throughs ram but not this shadow). Reading the
        // shadow saved a stale health to SRAM (and a checksum over the stale block),
        // surfacing on the next LoadFile. Mirror C: copy + checksum from live ram.
        let dung_info = self.ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + 0x500].to_vec();
        if offs + 0x500 <= self.sram.len() {
            self.sram[offs..offs + 0x500].copy_from_slice(&dung_info);
        }
        if offs + 0xf00 + 0x500 <= self.sram.len() {
            self.sram[offs + 0xf00..offs + 0xf00 + 0x500].copy_from_slice(&dung_info);
        }
        let mut checksum = 0x5a5au16;
        for i in (0..0x4fe).step_by(2) {
            checksum = checksum.wrapping_sub(read_le_u16(&self.ram, SAVE_DUNG_INFO + i));
        }
        // Keep the shadow + ram[SAVE_DUNG_INFO+0x4fe] coherent so the frame-end bulk
        // projection of dungeon_info doesn't re-stamp a stale checksum over ram.
        self.save_progress_mut().set_dungeon_info_checksum(checksum);
        if offs + 0x500 <= self.sram.len() {
            write_le_u16(&mut self.sram, offs + 0x4fe, checksum);
        }
        if offs + 0xf00 + 0x500 <= self.sram.len() {
            write_le_u16(&mut self.sram, offs + 0x4fe + 0xf00, checksum);
        }
        self.zelda_write_sram();
    }

    pub(super) fn TransferMode7Characters(&mut self) {
        self.transfer_mode7_characters();
    }

    pub(super) fn Animate_GAMEOVER_Letters(&mut self) {
        match self.game_state.sprites.ancilla_slots.slot(0).ancilla_type() {
            0 => self.increment_submodule(),
            1 => self.GameOverText_SweepLeft(),
            2 => self.GameOverText_UnfurlRight(),
            3 => self.GameOverText_Draw(),
            _ => {}
        }
    }

    pub(super) fn GameOverText_SweepLeft(&mut self) {
        const GAME_OVER_SWEEP_LEFT_X_TARGETS: [u8; 8] =
            [0x40, 0x50, 0x60, 0x70, 0x88, 0x98, 0xa8, 0x40];

        let mut k = self.game_state.minigame.flag_boomerang_in_place() as usize;
        self.sprite_system_mut().set_cur_object_index(k as u8);
        self.game_state
            .sprites
            .ancilla_slots
            .slot_mut(&mut self.ram, k)
            .set_x_velocity(0x80);
        self.ancilla_move_x(k);
        if self.ancilla_get_x(k) < u16::from(GAME_OVER_SWEEP_LEFT_X_TARGETS[k]) {
            self.game_state
                .sprites
                .ancilla_slots
                .slot_mut(&mut self.ram, k)
                .set_x_low(GAME_OVER_SWEEP_LEFT_X_TARGETS[k]);
            k += 1;
            self.minigame_state_mut()
                .set_flag_boomerang_in_place(k as u8);
            if k == 8 {
                self.minigame_state_mut().set_flag_boomerang_in_place(7);
                self.game_state
                    .sprites
                    .ancilla_slots
                    .slot_mut(&mut self.ram, 0)
                    .increment_ancilla_type();
                self.messaging_state_mut().clear_game_over_letter_cursor();
                self.set_sound_effect_2(38);
                self.GameOverText_Draw();
                return;
            }
        }
        if k == 7 {
            let mut j = 6i32;
            let x = self.game_state.sprites.ancilla_slots.slot(k).x_low();
            while j != i32::from(self.game_state.messaging.runtime.game_over_letter_cursor()) {
                self.game_state
                    .sprites
                    .ancilla_slots
                    .slot_mut(&mut self.ram, j as usize)
                    .set_x_low(x);
                j -= 1;
            }
            let hookshot = self.game_state.messaging.runtime.game_over_letter_cursor() as usize;
            if self.ancilla_get_x(k) < u16::from(GAME_OVER_SWEEP_LEFT_X_TARGETS[hookshot]) {
                self.messaging_state_mut()
                    .decrement_game_over_letter_cursor();
            }
        }
        self.GameOverText_Draw();
    }

    pub(super) fn GameOverText_UnfurlRight(&mut self) {
        const GAME_OVER_UNFURL_RIGHT_X_TARGETS: [u8; 8] =
            [0x58, 0x60, 0x68, 0x70, 0x88, 0x90, 0x98, 0xa0];

        let mut k = self.game_state.minigame.flag_boomerang_in_place() as usize;
        self.sprite_system_mut().set_cur_object_index(k as u8);
        self.game_state
            .sprites
            .ancilla_slots
            .slot_mut(&mut self.ram, k)
            .set_x_velocity(0x60);
        self.ancilla_move_x(k);
        let j = self.game_state.messaging.runtime.game_over_letter_cursor() as usize;
        if self.game_state.sprites.ancilla_slots.slot(k).x()
            >= u16::from(GAME_OVER_UNFURL_RIGHT_X_TARGETS[j])
        {
            self.game_state
                .sprites
                .ancilla_slots
                .slot_mut(&mut self.ram, j)
                .set_x_low(GAME_OVER_UNFURL_RIGHT_X_TARGETS[j]);
            self.messaging_state_mut()
                .increment_game_over_letter_cursor();
            if self.game_state.messaging.runtime.game_over_letter_cursor() == 8 {
                self.increment_submodule();
                self.game_state
                    .sprites
                    .ancilla_slots
                    .slot_mut(&mut self.ram, 0)
                    .increment_ancilla_type();
                self.GameOverText_Draw();
                return;
            }
        }
        let end =
            i32::from(self.game_state.messaging.runtime.game_over_letter_cursor()).wrapping_sub(1);
        k = self.game_state.minigame.flag_boomerang_in_place() as usize;
        let mut j = k as i32;
        let x = self.game_state.sprites.ancilla_slots.slot(k).x_low();
        loop {
            self.game_state
                .sprites
                .ancilla_slots
                .slot_mut(&mut self.ram, j as usize)
                .set_x_low(x);
            j -= 1;
            if j == end {
                break;
            }
        }
        self.GameOverText_Draw();
    }

    pub(super) fn GameOverText_Draw(&mut self) {
        self.set_pending_nmi_subroutine(0x12);
    }

    pub(super) fn Module12_GameOver(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.GameOver_AdvanceImmediately(),
            1 => self.Death_Func1(),
            2 => self.GameOver_DelayBeforeIris(),
            3 => self.GameOver_IrisWipe(),
            4 => self.Death_Func4(),
            5 => self.GameOver_SplatAndFade(),
            6 => self.Death_Func6(),
            7 => self.Animate_GAMEOVER_Letters_bounce(),
            8 => self.GameOver_Finalize_GAMEOVR(),
            9 => self.GameOver_SaveAndOrContinue(),
            10 => self.GameOver_InitializeRevivalFairy(),
            11 => self.RevivalFairy_Main_bounce(),
            12 => self.GameOver_RiseALittle(),
            13 => self.GameOver_Restore0D(),
            14 => self.GameOver_Restore0E(),
            15 => self.GameOver_ResituateLink(),
            _ => {}
        }
        if self.game_state.frame.submodule != 9 {
            self.link_oam_main();
        }
    }

    pub(super) fn GameOver_AdvanceImmediately(&mut self) {
        self.increment_submodule();
        self.Death_Func1();
    }

    pub(super) fn Death_Func1(&mut self) {
        let current_music = self.game_state.system_signals.current_music_control();
        let ambient_sound = self.game_state.system_signals.last_ambient_sound_effect();
        self.set_death_backup_current_music(current_music);
        self.set_death_backup_ambient_sound(ambient_sound);
        self.set_music_control(0xf1);
        self.set_ambient_sound_effect(5);
        self.set_overworld_map_state(5);
        self.follower_link_state_mut().clear_conveyor_belt_state();
        self.tile_detect_position_mut().set_layer_collision_flags(0);
        self.follower_link_state_mut().clear_cape_mode();
        let palette_filter_countdown = self.game_state.display.palette_filter.countdown_word();
        let darkening_or_lightening_screen = self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word();
        self.set_mapbak_bg1_x_offset(palette_filter_countdown);
        self.set_mapbak_bg1_y_offset(darkening_or_lightening_screen);
        let palette = self
            .game_state
            .display
            .palette_buffer
            .aux_visible_slice()
            .to_vec();
        self.copy_mapbak_palette_from(&palette[..palette.len().min(256)]);
        self.clear_aux_visible_subpalettes();
        self.set_countdown_word(0);
        self.set_darkening_or_lightening_screen_word(0);
        self.set_bg1_x_offset(0);
        self.set_bg1_y_offset(0);
        let cgwsel = self
            .game_state
            .display
            .palette_filter
            .color_window_and_math_word();
        self.set_mapbak_cgwsel_word(cgwsel);
        self.messaging_state_mut().set_menu_animation_timer(32);
        self.clear_floor_changed_timer_low();
        self.hud_floor_indicator();
        self.increment_hud_update_flag();
        self.set_ambient_sound_effect(5);
        self.increment_submodule();
    }

    pub(super) fn GameOver_DelayBeforeIris(&mut self) {
        self.messaging_state_mut().decrement_menu_animation_timer();
        if self.game_state.messaging.runtime.menu_animation_timer() != 0 {
            return;
        }
        self.Death_InitializeGameOverLetters();
        self.IrisSpotlight_close();
        self.set_object_color_window_selection(0x30);
        self.set_bg34_window_selection(0);
        self.increment_submodule();
    }

    pub(super) fn GameOver_IrisWipe(&mut self) {
        self.PaletteFilter_RestoreBGSubstractiveStrict();
        let bg = self.game_state.display.palette_buffer.main_color(32);
        self.set_main_color(0, bg);
        let bak = self.game_state.frame.main_module;
        self.IrisSpotlight_ConfigureTable();
        self.set_main_module(bak);
        if self.game_state.frame.submodule != 0 {
            return;
        }
        for base in [0x20usize, 0x30, 0x40, 0x50, 0x60, 0x70] {
            for i in 0..16 {
                self.set_main_color(base + i, 0x18);
            }
        }
        self.set_main_color(0, 0x18);
        self.set_main_color(32, 0x18);
        self.IrisSpotlight_ResetTable();
        self.set_fixed_color_red(32);
        self.set_fixed_color_green(64);
        self.set_fixed_color_blue(128);
        self.set_bg12_window_selection(0);
        self.set_bg34_window_selection(0);
        self.set_object_color_window_selection(0);
        self.set_submodule(4);
        self.increment_cgram_update_flag();
        self.set_screen_brightness(15);
        self.set_main_screen_layers(20);
        self.set_sub_screen_layers(0);
        self.set_color_math_control(32);
        self.messaging_state_mut().set_menu_animation_timer(64);
        self.set_countdown(0);
        self.set_darkening_or_lightening_screen(0);
        self.Death_PrepFaint();
    }

    pub(super) fn GameOver_SplatAndFade(&mut self) {
        if self.game_state.messaging.runtime.menu_animation_timer() != 0 {
            self.messaging_state_mut().decrement_menu_animation_timer();
            return;
        }
        self.PaletteFilter_RestoreBGSubstractiveStrict();
        let bg = self.game_state.display.palette_buffer.main_color(32);
        self.set_main_color(0, bg);
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            != 0xff
        {
            return;
        }
        self.clear_mosaic_level();
        self.clear_mosaic_direction();
        self.set_mosaic_copy(3);
        for i in 0..4 {
            if self.game_state.inventory.items.bottle(i) == 6 {
                let value = 2;
                self.inventory_items_mut().set_bottle(i, value);
                self.messaging_state_mut().set_menu_animation_timer(12);
                self.set_chr_halfslot_request(15);
                self.Graphics_LoadChrHalfSlot();
                self.clear_chr_halfslot_request();
                self.set_submodule(10);
                return;
            }
        }
        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(0);
        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(1);
        self.set_pending_nmi_subroutine(22);
        self.set_core_update_disable_flag(22);
        self.increment_submodule();
    }

    pub(super) fn Death_Func6(&mut self) {
        self.messaging_state_mut().set_menu_animation_timer(12);
        self.set_chr_halfslot_request(15);
        self.Graphics_LoadChrHalfSlot();
        self.clear_chr_halfslot_request();
        self.set_sp6r_indoors(5);
        self.select_overworld_aux_palette_offset();
        self.Palette_Load_SpriteEnvironment_Dungeon();
        self.Palette_Load_SpriteMain();
        self.increment_cgram_update_flag();
        self.increment_submodule();
        self.Death_PlayerSwoon();
    }

    pub(super) fn Death_Func4(&mut self) {
        self.Death_PlayerSwoon();
    }

    pub(super) fn Animate_GAMEOVER_Letters_bounce(&mut self) {
        self.Animate_GAMEOVER_Letters();
    }

    pub(super) fn GameOver_Finalize_GAMEOVR(&mut self) {
        self.Animate_GAMEOVER_Letters();
        let bak1 = self.game_state.frame.main_module;
        let bak2 = self.game_state.frame.submodule;
        self.messaging_state_mut().set_module(2);
        self.RenderText();
        self.set_submodule(bak2.wrapping_add(1));
        self.set_main_module(bak1);
        self.messaging_state_mut().set_menu_animation_timer(2);
        self.set_music_control(11);
    }

    pub(super) fn GameOver_SaveAndOrContinue(&mut self) {
        self.GameOver_AnimateChoiceFairy();
        self.Animate_GAMEOVER_Letters();

        if self.game_state.player.follower_link.filtered_joypad_h() & 0x20 != 0 {
            self.increment_subsubmodule();
            if self.game_state.frame.subsubmodule >= 3 {
                self.set_subsubmodule(0);
            }
            self.messaging_state_mut().set_menu_animation_timer(12);
            self.set_sound_effect_2(32);
        } else {
            self.messaging_state_mut().decrement_menu_animation_timer();
            if self.game_state.messaging.runtime.menu_animation_timer() == 0 {
                self.messaging_state_mut().set_menu_animation_timer(1);
                if self.game_state.player.follower_link.joypad1h_last() & 12 != 0 {
                    if self.game_state.player.follower_link.joypad1h_last() & 4 != 0 {
                        self.increment_subsubmodule();
                        if self.game_state.frame.subsubmodule >= 3 {
                            self.set_subsubmodule(0);
                        }
                    } else {
                        self.decrement_subsubmodule();
                        if (self.game_state.frame.subsubmodule as i8).is_negative() {
                            self.set_subsubmodule(2);
                        }
                    }
                    self.messaging_state_mut().set_menu_animation_timer(12);
                    self.set_sound_effect_2(32);
                }
            }
        }

        if ((self.game_state.player.follower_link.filtered_joypad_l() & 0xc0)
            | self.game_state.player.follower_link.filtered_joypad_h())
            & 0xd0
            == 0
        {
            return;
        }
        self.set_sound_effect_1(44);
        self.Death_Func15(self.game_state.frame.subsubmodule != 2);
    }

    pub(super) fn Death_Func15(&mut self, count_as_death: bool) {
        self.set_music_control(0xf1);
        if self.game_state.world.location.is_indoors() {
            self.Dungeon_FlagRoomData_Quadrants();
        }
        self.AdjustLinkBunnyStatus();
        if self.game_state.inventory.save_progress.progress_indicator() < 3 {
            self.save_progress_mut().set_dark_world_state(0);
            if !self.game_state.inventory.items.has_moon_pearl() {
                self.ForceNonbunnyStatus();
            }
        }
        if self.game_state.world.location.dungeon_room() == 0 {
            self.set_indoor_flag(0);
        }

        self.reset_some_things_after_death(self.game_state.world.location.dungeon_room() as u8);
        if matches!(
            self.game_state.sprites.follower_runtime.indicator(),
            6 | 9 | 10 | 13
        ) {
            self.follower_state_mut().set_indicator(0);
        }

        let health = POST_DEATH_HEALTH_BY_CAPACITY
            [(self.game_state.inventory.player_resources.health_capacity() >> 3) as usize];
        self.set_restart_check_flag(health);
        self.player_resources_mut().set_current_health(health);
        let palace = self.game_state.inventory.save_progress.palace_index_x2();
        if palace != 0xff {
            let slot = if palace == 2 { 0 } else { palace } >> 1;
            let keys = self.game_state.inventory.player_resources.keys();
            self.dungeon_key_slots_mut()
                .set_keys_earned_slot(slot as usize, keys);
        }
        self.sprite_reset_all();
        if self
            .game_state
            .inventory
            .save_progress
            .total_death_save_counter_is_uninitialized()
            && (!self.game_state.enhanced_features.has(4096) || count_as_death)
        {
            self.save_progress_mut()
                .increment_pending_death_save_counter();
        }
        self.increment_game_over_check_flag();
        if self.game_state.frame.subsubmodule != 1 {
            if self.game_state.world.location.is_indoors() {
                if self.game_state.sprites.follower_runtime.indicator() != 1
                    && self.game_state.inventory.save_progress.palace_index_x2() != 0xff
                {
                    self.clear_restart_check_flag();
                } else {
                    self.set_queued_music_control(0);
                    self.set_indoor_flag(0);
                    if self.game_state.inventory.save_progress.dark_world_state() != 0 {
                        self.set_dungeon_room(32);
                    }
                }
            } else if self.game_state.inventory.save_progress.dark_world_state() != 0 {
                self.set_dungeon_room(32);
            }

            if self.game_state.inventory.save_progress.progress_indicator() != 0 {
                if self.game_state.frame.subsubmodule == 0 {
                    self.SaveGameFile();
                }
                self.set_main_module(5);
                self.set_submodule(0);
                self.clear_bg_vram_load_mode();
            } else {
                let offset = self.selected_save_slot_source_offset();
                self.save_load_scratch_mut().set_source_offset(offset);
                self.clear_game_over_check_flag();
                self.CopySaveToWRAM();
            }
        } else {
            if self.game_state.inventory.save_progress.progress_indicator() != 0 {
                self.SaveGameFile();
            }
            self.set_main_screen_layers(16);
            self.set_indoor_flag(0);
            self.death_func31();
            self.clear_restart_check_flag();
            self.clear_game_over_check_flag();
            self.set_queued_music_control(0);
            self.set_bg1_x(0);
            self.set_bg2_x(0);
            self.set_bg3_h_copy2(0);
            self.set_bg1_y(0);
            self.set_bg2_y(0);
            self.set_bg3_v_copy2(0);
            self.set_bg1_h_copy(0);
            self.set_bg2_h_copy(0);
            self.set_bg1_v_copy(0);
            self.set_bg2_v_copy(0);
            self.save_progress_mut().clear_dungeon_info();
            self.messaging_state_mut()
                .clear_flag_which_music_type_messaging();
            self.load_overworld_songs();
        }
    }

    pub(super) fn GameOver_AnimateChoiceFairy(&mut self) {
        self.set_oam_plain(
            0x14,
            0x34,
            DEATH_SPR_Y0[self.game_state.frame.subsubmodule as usize],
            DEATH_SPR_CHAR0[(self.game_state.frame.frame_counter >> 3 & 1) as usize],
            0x78,
            2,
        );
    }

    pub(super) fn GameOver_InitializeRevivalFairy(&mut self) {
        self.configure_revival_ancillae();
        self.player_resources_mut().set_heart_filler(56);
        self.increment_submodule();
        self.set_overworld_map_state(0);
    }

    pub(super) fn RevivalFairy_Main_bounce(&mut self) {
        self.revival_fairy_main();
    }

    pub(super) fn GameOver_RiseALittle(&mut self) {
        if self.game_state.inventory.player_resources.heart_filler() == 0 {
            let palette = self
                .game_state
                .display
                .ppu_scroll_copy
                .mapbak_palette_slice()[..256]
                .to_vec();
            self.copy_aux_visible_from(&palette);
            self.clear_main_visible_subpalettes();
            self.set_main_color(0, 0);
            self.set_countdown_word(0);
            self.set_darkening_or_lightening_screen_word(2);
            let cgwsel = self.game_state.display.ppu_scroll_copy.mapbak_cgwsel_word();
            self.set_color_window_and_math_word(cgwsel);
            self.increment_submodule();
        }
        self.revival_fairy_main();
        self.hud_refill_logic();
    }

    pub(super) fn GameOver_Restore0D(&mut self) {
        if !self.hud_state().is_doing_heart_animation() {
            self.set_chr_halfslot_request(1);
            self.Graphics_LoadChrHalfSlot();
            let fixed_color = self.game_state.display.overworld_fixed_color_adjustment;
            self.Dungeon_ApproachFixedColor_variable(fixed_color);
            self.increment_submodule();
        }
        self.revival_fairy_main();
        self.hud_refill_logic();
    }

    pub(super) fn GameOver_Restore0E(&mut self) {
        self.Graphics_LoadChrHalfSlot();
        let sub_screen_layers = self.game_state.display.ppu_scroll_copy.mapbak_ts();
        self.set_sub_screen_layers(sub_screen_layers);
        self.increment_submodule();
    }

    pub(super) fn GameOver_ResituateLink(&mut self) {
        self.PaletteFilter_RestoreBGAdditiveStrict();
        let bg = self.game_state.display.palette_buffer.main_color(32);
        self.set_main_color(0, bg);
        if self.game_state.display.palette_filter.countdown() != 32 {
            return;
        }
        if self.game_state.world.location.is_outdoors() {
            self.Overworld_SetFixedColAndScroll();
        }
        let sub_screen_layers = self.game_state.display.ppu_scroll_copy.mapbak_ts();
        self.set_sub_screen_layers(sub_screen_layers);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
        self.set_submodule(0);
        self.follower_link_state_mut().set_blink_countdown(144);
        let music = self.game_state.system_signals.death_backup_current_music();
        self.set_music_control(music);
        let ambient = self.game_state.system_signals.death_backup_ambient_sound();
        self.set_ambient_sound_effect(ambient);
        let palette_filter_countdown = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_bg1_x_offset();
        let darkening_or_lightening_screen = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_bg1_y_offset();
        self.set_countdown_word(palette_filter_countdown);
        self.set_darkening_or_lightening_screen_word(darkening_or_lightening_screen);
    }

    pub(super) fn Module0E_0A_FluteMenu(&mut self) {
        match self.overworld_map_state() {
            0 => self.WorldMap_FadeOut(),
            1 => {
                self.set_birdtravel_status(0);
                self.WorldMap_LoadLightWorldMap();
            }
            2 => self.WorldMap_LoadSpriteGFX(),
            3 => self.WorldMap_Brighten(),
            4 => {
                self.messaging_state_mut().set_menu_animation_timer(0x10);
                self.increment_overworld_map_state();
            }
            5 => self.FluteMenu_HandleSelection(),
            6 => self.WorldMap_RestoreGraphics(),
            7 => self.FluteMenu_LoadSelectedScreen(),
            8 => self.Overworld_LoadOverlayAndMap(),
            9 => self.FluteMenu_FadeInAndQuack(),
            // C Module0E_0A_FluteMenu asserts outside states 0..=9.
            state => panic!("Module0E_0A_FluteMenu invalid overworld_map_state {state}"),
        }
    }

    pub(super) fn FluteMenu_HandleSelection(&mut self) {
        if self.game_state.messaging.runtime.menu_animation_timer() == 0 {
            if (self.game_state.player.follower_link.joypad1l_last()
                | self.game_state.player.follower_link.joypad1h_last())
                & 0xc0
                != 0
            {
                if self
                    .game_state
                    .enhanced_features
                    .has(FEATURE_CANCEL_BIRD_TRAVEL)
                {
                    let joypad = self.game_state.player.follower_link.joypad1l_last();
                    self.messaging_state_mut().set_menu_animation_timer(joypad);
                }
                self.increment_overworld_map_state();
                return;
            }
        } else {
            self.messaging_state_mut().decrement_menu_animation_timer();
        }

        if self.game_state.player.follower_link.filtered_joypad_h() & 10 != 0 {
            self.decrement_birdtravel_status();
            self.set_sound_effect_2(32);
        }
        if self.game_state.player.follower_link.filtered_joypad_h() & 5 != 0 {
            self.increment_birdtravel_status();
            self.set_sound_effect_2(32);
        }
        self.and_birdtravel_status(7);

        let mut pt = Point16U { x: 0, y: 0 };
        if self.game_state.frame.frame_counter & 0x10 != 0
            && self.WorldMap_CalculateOamCoordinates(&mut pt)
        {
            self.WorldMap_AddSprite(16, 2, 0x3e, 0, pt.x.wrapping_sub(4), pt.y.wrapping_sub(4));
        }

        let ybak = self.game_state.player.special_exit_position.y();
        let xbak = self.game_state.player.special_exit_position.x();
        for i in (0..8).rev() {
            let bird_x = u16::from(BIRD_TRAVEL_X_HIGH[i]) << 8 | u16::from(BIRD_TRAVEL_X_LOW[i]);
            let bird_y = u16::from(BIRD_TRAVEL_Y_HIGH[i]) << 8 | u16::from(BIRD_TRAVEL_Y_LOW[i]);
            self.set_bird_travel_destination(i, bird_x, bird_y);
            self.special_exit_position_mut()
                .set_position(bird_x, bird_y);

            if self.WorldMap_CalculateOamCoordinates(&mut pt) {
                self.WorldMap_AddSprite(
                    i,
                    0,
                    if i == usize::from(self.birdtravel_status()) {
                        0x30 + (self.game_state.frame.frame_counter & 6)
                    } else {
                        0x32
                    },
                    BIRD_TRAVEL_OVERWORLD_SCREEN_BY_STOP[i],
                    pt.x,
                    pt.y,
                );
            }
        }
        self.special_exit_position_mut().set_position(xbak, ybak);
    }

    pub(super) fn FluteMenu_LoadSelectedScreen(&mut self) {
        self.clear_overworld_event_bits(0x3b, 0x20);
        self.clear_overworld_event_bits(0x7b, 0x20);
        let dung_267 = self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(267)
            & !0x0080;
        let dung_40 = self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(40)
            & !0x0100;
        self.save_progress_mut()
            .set_dungeon_info_word(267, dung_267);
        self.save_progress_mut().set_dungeon_info_word(40, dung_40);

        if self.game_state.messaging.runtime.menu_animation_timer() & 0x40 == 0 {
            self.FluteMenu_LoadTransport();
        }

        self.FluteMenu_LoadSelectedScreenPalettes();
        let t = self.game_state.world.location.overworld_screen_index() & 0xbf;
        self.DecompressAnimatedOverworldTiles(if t == 3 || t == 5 || t == 7 {
            0x58
        } else {
            0x5a
        });
        self.Overworld_SetFixedColAndScroll();
        self.clear_overworld_aux_or_main_offset();
        self.set_hud_palette(0);
        self.InitializeTilesets();
        self.increment_overworld_map_state();
        self.dungeon_room_load_mut().set_draw_width_indicator(0);
        self.Overworld_LoadOverlays2();
        self.decrement_submodule();
        self.set_sound_effect_2(16);
        let m = self.overworld_config_table().current_music();
        self.set_ambient_sound_effect(m >> 4);
        let track = m & 0x0f;
        let music = if self.zelda_is_playing_music_track(track) {
            0xf3
        } else {
            track
        };
        self.set_music_control(music);
    }

    pub(super) fn Overworld_LoadOverlayAndMap(&mut self) {
        let bak1 = self.game_state.frame.main_module_word();
        let bak2 = self.overworld_map_state_word();
        self.Overworld_LoadAndBuildScreen();
        self.set_overworld_map_state_word(bak2.wrapping_add(1));
        self.set_main_module_word(bak1);
    }

    pub(super) fn FluteMenu_FadeInAndQuack(&mut self) {
        self.increment_screen_brightness();
        if self.game_state.display.screen_brightness == 15 {
            self.BirdTravel_Finish_Doit();
        } else {
            self.sprite_main();
        }
    }

    pub(super) fn BirdTravel_Finish_Doit(&mut self) {
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
        self.set_submodule(0);
        let hdma_enable_mask = self.game_state.display.ppu_scroll_copy.mapbak_hdmaen();
        self.set_hdma_enable_mask(hdma_enable_mask);
        self.add_bird_travel_something(0x27, 4);
        self.sprite_main();
    }

    pub(super) fn Messaging_OverworldMap(&mut self) {
        match self.overworld_map_state() {
            0 => self.WorldMap_FadeOut(),
            1 => self.WorldMap_LoadLightWorldMap(),
            2 => self.WorldMap_LoadDarkWorldMap(),
            3 => self.WorldMap_LoadSpriteGFX(),
            4 => self.WorldMap_Brighten(),
            5 => self.WorldMap_PlayerControl(),
            6 => self.WorldMap_RestoreGraphics(),
            7 => self.WorldMap_ExitMap(),
            _ => {}
        }
    }

    pub(super) fn WorldMap_FadeOut(&mut self) {
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        let hdmaen = self.game_state.display.hdma_enable_mask;
        self.set_mapbak_hdmaen(hdmaen);
        self.EnableForceBlank();
        self.set_mosaic_copy(3);
        self.increment_overworld_map_state();
        // C backs up MAPBAK_TM:MAPBAK_TS from the RAM word TM_COPY:TS_COPY (the last-projected
        // values), not the live native layer masks. The native sub_screen_layers can be
        // transiently ahead of RAM TS_COPY this frame (dark-room sub-screen path), so reading
        // layer_masks_word() backed up a stale 1 into MAPBAK_TS (0xc212) vs old-rust's 0.
        let tm_ts = read_le_u16(&self.ram, crate::game_state::constants::TM_COPY);
        self.set_mapbak_tm_word(tm_ts);
        let bg1hofs = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
        let bg2hofs = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg1vofs = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
        let bg2vofs = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.set_map_backup_scrolls(bg1hofs, bg2hofs, bg1vofs, bg2vofs);
        self.set_bg1_x(0);
        self.set_bg2_x(0);
        self.set_bg3_h_copy2(0);
        self.set_bg1_y(0);
        self.set_bg2_y(0);
        self.set_bg3_v_copy2(0);
        let cgwsel_cgadsub = self
            .game_state
            .display
            .palette_filter
            .color_window_and_math_word();
        self.set_mapbak_cgwsel_word(cgwsel_cgadsub);
        self.follower_link_state_mut()
            .set_link_dma_graphics_index_word(0x01fc);
        if self.game_state.world.location.overworld_screen_index() < 0x80 {
            self.special_exit_position_mut().store_from_player();
        }
        if self.game_state.inventory.save_progress.progress_indicator() < 2 {
            self.set_color_window_selection(0x80);
            self.set_color_math_control(0x61);
        }
        self.set_sound_effect_2(16);
        self.set_ambient_sound_effect(5);
        self.set_music_control(0xf2);
        self.set_bg_mode(7);
    }

    pub(super) fn WorldMap_LoadLightWorldMap(&mut self) {
        self.world_map_load_light_world_map();
    }

    pub(super) fn WorldMap_LoadDarkWorldMap(&mut self) {
        if u16::from(self.game_state.world.location.overworld_screen_index()) & 0x40 != 0 {
            if let Some(tilemap) = self.asset_raw(68).map(Vec::from) {
                let len = tilemap.len().min(1024);
                self.copy_tilemap_upload_stripe_bytes(&tilemap[..len]);
            }
            self.set_pending_nmi_subroutine(21);
        }
        self.increment_overworld_map_state();
    }

    pub(super) fn WorldMap_LoadSpriteGFX(&mut self) {
        self.set_chr_halfslot_request(0x10);
        self.Graphics_LoadChrHalfSlot();
        self.clear_chr_halfslot_request();
        self.increment_overworld_map_state();
    }

    pub(super) fn WorldMap_Brighten(&mut self) {
        self.increment_screen_brightness();
        if self.game_state.display.screen_brightness == 15 {
            self.increment_overworld_map_state();
        }
    }

    pub(super) fn DidPressButtonForMap(&self) -> bool {
        if self.game_state.world.transient.hud_cur_item_x() != 0 {
            self.game_state.player.follower_link.filtered_joypad_h() & 0x20 != 0
        } else {
            self.game_state.player.follower_link.filtered_joypad_l() & 0x40 != 0
        }
    }

    pub(super) fn WorldMap_PlayerControl(&mut self) {
        const OVERWORLD_MAP_TIMER: [u8; 2] = [33, 12];
        const OVERWORLD_MAP_SCROLL_DELTAS: [i16; 8] = [0, 0, 1, 2, -1, -2, 1, 2];
        const OVERWORLD_MAP_SCROLL_TARGETS: [u16; 8] =
            [0, 0, 224, 480, (-72i16) as u16, (-224i16) as u16, 0, 0];

        if self.overworld_map_flags() & 0x80 != 0 {
            self.and_overworld_map_flags(!0x80);
            self.OverworldMap_SetupHdma();
        }

        if self.overworld_map_flags() == 0 && self.DidPressButtonForMap() {
            self.increment_overworld_map_state();
            return;
        }

        if self.game_state.dungeon.room_load.draw_width_indicator() != 0 {
            let draw_width = self
                .game_state
                .dungeon
                .room_load
                .draw_width_indicator()
                .wrapping_sub(1);
            self.dungeon_room_load_mut()
                .set_draw_width_indicator(draw_width);
        } else if self.game_state.player.follower_link.filtered_joypad_l() & 0x30 != 0
            || self.DidPressButtonForMap()
        {
            self.set_sound_effect_2(36);
            self.dungeon_room_load_mut().set_draw_width_indicator(8);
            let t = (self.overworld_map_flags() ^ 1) & 1;
            self.set_overworld_map_flags(t | 0x80);
            self.set_mode7_zoom_timer(OVERWORLD_MAP_TIMER[t as usize]);
            if self.mode7_zoom_timer() == 12 {
                let y = self.game_state.player.special_exit_position.map_zoom_y();
                self.set_bg1_y(y);
                self.set_mode7_center_y(y.wrapping_add(0x100));
                let t0 = self
                    .game_state
                    .player
                    .special_exit_position
                    .map_zoom_x_offset();
                let abs_t0 = if (t0 as i16) < 0 {
                    0u16.wrapping_sub(t0)
                } else {
                    t0
                };
                let t1 = abs_t0.wrapping_mul(5) >> 1;
                let t2 = if (t0 as i16) < 0 {
                    0u16.wrapping_sub(t1)
                } else {
                    t1
                };
                self.set_bg1_x(t2.wrapping_add(0x80) & !1);
            } else {
                self.set_bg1_y(200);
                self.set_mode7_center_y(200 + 256);
                self.set_bg1_x(128);
            }
        }

        if self.overworld_map_flags() != 0 {
            let k = ((self.game_state.player.follower_link.joypad1h_last() & 12) >> 1) as usize;
            let bg1vofs = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
            if bg1vofs != OVERWORLD_MAP_SCROLL_TARGETS[k] {
                let next = bg1vofs.wrapping_add(OVERWORLD_MAP_SCROLL_DELTAS[k] as u16);
                self.set_bg1_y(next);
                self.set_mode7_center_y(next.wrapping_add(0x100));
            }
            let k = ((self.game_state.player.follower_link.joypad1h_last() & 3) * 2 + 1) as usize;
            let bg1hofs = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
            if bg1hofs != OVERWORLD_MAP_SCROLL_TARGETS[k] {
                self.set_bg1_h_copy2(bg1hofs.wrapping_add(OVERWORLD_MAP_SCROLL_DELTAS[k] as u16));
            }
        }

        self.WorldMap_HandleSprites();
    }

    pub(super) fn WorldMap_RestoreGraphics(&mut self) {
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        self.EnableForceBlank();
        self.increment_overworld_map_state();
        let aux = self
            .game_state
            .display
            .palette_buffer
            .aux_full_slice()
            .to_vec();
        self.copy_main_full_from(&aux);
        let cgwsel_cgadsub = self.game_state.display.ppu_scroll_copy.mapbak_cgwsel_word();
        self.set_color_window_and_math_word(cgwsel_cgadsub);
        self.set_bg3_h_copy2(0);
        self.set_bg3_v_copy2(0);
        let bg1hofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg1_h_copy2();
        let bg2hofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg2_h_copy2();
        let bg1vofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg1_v_copy2();
        let bg2vofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg2_v_copy2();
        self.set_bg1_x(bg1hofs);
        self.set_bg2_x(bg2hofs);
        self.set_bg1_y(bg1vofs);
        self.set_bg2_y(bg2vofs);
        let tm_ts = self.game_state.display.ppu_scroll_copy.mapbak_tm_word();
        self.set_layer_masks_word(tm_ts);
        self.Attract_SetUpConclusionHDMA();
    }

    pub(super) fn Attract_SetUpConclusionHDMA(&mut self) {
        self.hdma_setup(0x0abddd, 0x0abddd, 0x42, 0x1b, 0x1e, 0);
        self.set_hdma_enable_mask(0x80);
        self.set_bg_mode(9);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn WorldMap_ExitMap(&mut self) {
        self.clear_overworld_aux_or_main_offset();
        self.set_hud_palette(0);
        self.InitializeTilesets();
        self.increment_cgram_update_flag();
        self.dungeon_room_load_mut().set_draw_width_indicator(0);
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
        self.set_submodule(32);
        self.clear_vram_upload_cursor();
        let hdma_enable_mask = self.game_state.display.ppu_scroll_copy.mapbak_hdmaen();
        self.set_hdma_enable_mask(hdma_enable_mask);
        let music = self.overworld_config_table().current_music();
        self.set_ambient_sound_effect(music >> 4);
        self.set_sound_effect_2(0x10);
        self.set_music_control(0xf3);
    }

    pub(super) fn WorldMap_SetUpHDMA(&mut self) {
        self.world_map_setup_hdma();
    }

    pub(super) fn WorldMap_FillTilemapWithEF(&mut self) {
        self.world_map_fill_tilemap_with_ef();
    }

    pub(super) fn WorldMap_HandleSprites(&mut self) {
        let ybak = self.game_state.player.special_exit_position.y();
        let xbak = self.game_state.player.special_exit_position.x();

        if self.game_state.frame.frame_counter & 0x10 != 0 {
            if let Some((x, y)) = self.WorldMap_CalculateCurrentOamCoordinates() {
                self.WorldMap_AddSprite(0, 2, 0x3e, 0, x.wrapping_sub(4), y.wrapping_sub(4));
            }
        }

        let k = 15;
        if self.game_state.world.location.overworld_screen_index() < 0x40
            && !self
                .game_state
                .world
                .overworld
                .bird_travel_destinations
                .destination(k)
                .is_empty()
        {
            if self.game_state.frame.frame_counter == 0 {
                self.increment_bird_travel_stop_status(k);
            }
            let bird = self
                .game_state
                .world
                .overworld
                .bird_travel_destinations
                .destination(k);
            let bird_x = bird.x;
            let bird_y = bird.y;
            self.special_exit_position_mut()
                .set_position(bird_x, bird_y);
            if let Some((x, y)) = self.WorldMap_CalculateCurrentOamCoordinates() {
                const OVERWORLD_MAP_BIRD_FRAME_CHARS: [u8; 4] = [0x34, 0x74, 0xf4, 0xb4];
                self.WorldMap_AddSprite(
                    15,
                    2,
                    OVERWORLD_MAP_BIRD_FRAME_CHARS
                        [(self.game_state.frame.frame_counter >> 1 & 3) as usize],
                    0x6a,
                    x,
                    y,
                );
            }
        }

        if self.game_state.world.overworld.event_info.event_info(0x5b) & 0x20 == 0
            && (((self
                .game_state
                .inventory
                .save_progress
                .map_icons_indicator()
                >= 6) as u8
                ^ self.game_state.world.region.is_in_dark_world() as u8)
                & 1)
                == 0
        {
            self.WorldMap_HandleCrystalSprites();
        }

        self.special_exit_position_mut().set_position(xbak, ybak);
    }

    fn WorldMap_HandleCrystalSprites(&mut self) {
        const OVERWORLD_MAP_CRYSTAL_ICON_FRAMES: [u8; 4] = [0x68, 0x69, 0x78, 0x69];
        const X: [[u16; 9]; 7] = [
            [0x7ff, 0x2c0, 0xd00, 0xf31, 0x6d, 0x7e0, 0xf40, 0xf40, 0x8dc],
            [
                0xff00, 0xff00, 0xff00, 0x8d0, 0xff00, 0xff00, 0xff00, 0x82, 0xff00,
            ],
            [
                0xff00, 0xff00, 0xff00, 0x108, 0xff00, 0xff00, 0xff00, 0xf11, 0xff00,
            ],
            [
                0xff00, 0xff00, 0xff00, 0x6d, 0xff00, 0xff00, 0xff00, 0x1d0, 0xff00,
            ],
            [
                0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0x100, 0xff00,
            ],
            [
                0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xca0, 0xff00,
            ],
            [
                0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0x759, 0xff00,
            ],
        ];
        const Y: [[u16; 9]; 7] = [
            [0x730, 0x6a0, 0x710, 0x620, 0x70, 0x640, 0x620, 0x620, 0x30],
            [
                0xff00, 0xff00, 0xff00, 0x80, 0xff00, 0xff00, 0xff00, 0xb0, 0xff00,
            ],
            [
                0xff00, 0xff00, 0xff00, 0xd70, 0xff00, 0xff00, 0xff00, 0x103, 0xff00,
            ],
            [
                0xff00, 0xff00, 0xff00, 0x70, 0xff00, 0xff00, 0xff00, 0x780, 0xff00,
            ],
            [
                0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xca0, 0xff00,
            ],
            [
                0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xda0, 0xff00,
            ],
            [
                0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xff00, 0xed0, 0xff00,
            ],
        ];
        const INFO: [[u16; 9]; 7] = [
            [0, 0, 0, 0x6038, 0x6234, 0x6632, 0x6434, 0x6434, 0x6632],
            [0, 0, 0, 0x6032, 0, 0, 0, 0x6434, 0],
            [0, 0, 0, 0x6034, 0, 0, 0, 0x6434, 0],
            [0, 0, 0, 0x6234, 0, 0, 0, 0x6434, 0],
            [0, 0, 0, 0, 0, 0, 0, 0x6434, 0],
            [0, 0, 0, 0, 0, 0, 0, 0x6434, 0],
            [0, 0, 0, 0, 0, 0, 0, 0x6434, 0],
        ];
        let k = self
            .game_state
            .inventory
            .save_progress
            .map_icons_indicator() as usize;
        if k >= 9 {
            return;
        }
        for crystal in 0..7 {
            let have_marker = if crystal < 3 {
                self.OverworldMap_CheckForPendant(crystal)
                    || self.OverworldMap_CheckForCrystal(crystal)
            } else {
                self.OverworldMap_CheckForCrystal(crystal)
            };
            if have_marker || (X[crystal][k] as i16) < 0 {
                continue;
            }
            self.special_exit_position_mut()
                .set_position(X[crystal][k], Y[crystal][k]);
            let mut info = INFO[crystal][k];
            let t = (info >> 8) as u8;
            if t != 0 {
                if t != 100 && self.game_state.frame.frame_counter & 0x10 != 0 {
                    continue;
                }
                self.special_exit_position_mut()
                    .offset_position(0u16.wrapping_sub(4), 0u16.wrapping_sub(4));
            }
            if let Some((x, y)) = self.WorldMap_CalculateCurrentOamCoordinates() {
                let mut ext = 2;
                if info >> 8 == 0 {
                    info = u16::from(
                        OVERWORLD_MAP_CRYSTAL_ICON_FRAMES
                            [(self.game_state.frame.frame_counter >> 3 & 3) as usize],
                    ) << 8
                        | 0x32;
                    ext = 0;
                }
                self.WorldMap_AddSprite(14 - crystal, ext, info as u8, (info >> 8) as u8, x, y);
            }
        }
    }

    pub(super) fn WorldMap_CalculateOamCoordinates(&mut self, pt: &mut Point16U) -> bool {
        if let Some((x, y)) = self.WorldMap_CalculateCurrentOamCoordinates() {
            pt.x = x;
            pt.y = y;
            true
        } else {
            false
        }
    }

    fn WorldMap_CalculateCurrentOamCoordinates(&self) -> Option<(u16, u16)> {
        const OVERWORLD_MAP_PROJECTION_CURVE: [u8; 333] = [
            0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0,
            0xe0, 0xdf, 0xde, 0xdd, 0xdc, 0xdb, 0xda, 0xd8, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2,
            0xd1, 0xd0, 0xcf, 0xce, 0xcd, 0xcc, 0xcb, 0xca, 0xc9, 0xc7, 0xc6, 0xc5, 0xc4, 0xc3,
            0xc2, 0xc1, 0xc0, 0xbf, 0xbe, 0xbd, 0xbc, 0xbb, 0xba, 0xb9, 0xb8, 0xb7, 0xb6, 0xb5,
            0xb4, 0xb3, 0xb2, 0xb1, 0xb0, 0xaf, 0xae, 0xad, 0xac, 0xab, 0xaa, 0xa9, 0xa8, 0xa7,
            0xa6, 0xa5, 0xa4, 0xa3, 0xa2, 0xa1, 0xa0, 0x9f, 0x9e, 0x9d, 0x9c, 0x9b, 0x9b, 0x9a,
            0x99, 0x98, 0x97, 0x96, 0x95, 0x94, 0x93, 0x92, 0x91, 0x90, 0x8f, 0x8e, 0x8d, 0x8c,
            0x8b, 0x8b, 0x8a, 0x89, 0x88, 0x87, 0x86, 0x85, 0x84, 0x83, 0x82, 0x81, 0x81, 0x80,
            0x7f, 0x7e, 0x7d, 0x7c, 0x7b, 0x7a, 0x79, 0x79, 0x78, 0x77, 0x76, 0x75, 0x74, 0x73,
            0x72, 0x72, 0x71, 0x70, 0x6f, 0x6e, 0x6d, 0x6c, 0x6c, 0x6b, 0x6a, 0x69, 0x68, 0x67,
            0x67, 0x66, 0x65, 0x64, 0x63, 0x62, 0x62, 0x61, 0x60, 0x5f, 0x5e, 0x5d, 0x5d, 0x5c,
            0x5b, 0x5a, 0x59, 0x59, 0x58, 0x57, 0x56, 0x55, 0x55, 0x54, 0x53, 0x52, 0x51, 0x51,
            0x50, 0x4f, 0x4e, 0x4e, 0x4d, 0x4c, 0x4b, 0x4a, 0x4a, 0x49, 0x48, 0x47, 0x47, 0x46,
            0x45, 0x44, 0x44, 0x43, 0x42, 0x41, 0x41, 0x40, 0x3f, 0x3e, 0x3e, 0x3d, 0x3c, 0x3c,
            0x3b, 0x3a, 0x39, 0x39, 0x38, 0x37, 0x36, 0x36, 0x35, 0x34, 0x34, 0x33, 0x32, 0x32,
            0x31, 0x30, 0x2f, 0x2f, 0x2e, 0x2d, 0x2d, 0x2c, 0x2b, 0x2b, 0x2a, 0x29, 0x29, 0x28,
            0x27, 0x27, 0x26, 0x25, 0x25, 0x24, 0x23, 0x23, 0x22, 0x21, 0x21, 0x20, 0x1f, 0x1f,
            0x1e, 0x1d, 0x1d, 0x1c, 0x1c, 0x1b, 0x1a, 0x1a, 0x19, 0x18, 0x18, 0x17, 0x17, 0x16,
            0x15, 0x15, 0x14, 0x14, 0x13, 0x12, 0x12, 0x11, 0x10, 0x10, 0x0f, 0x0f, 0x0e, 0x0e,
            0x0d, 0x0c, 0x0c, 0x0b, 0x0b, 0x0a, 9, 9, 8, 8, 7, 7, 6, 5, 5, 4, 4, 3, 3, 2, 1, 1, 0,
            0, 0, 0, 0xff, 0xfe, 0xfe, 0xfd, 0xfc, 0xfc, 0xfb, 0xfb, 0xfa, 0xf9, 0xf9, 0xf8, 0xf7,
            0xf7, 0xf6, 0xf5, 0xf4, 0xf4, 0xf3, 0xf2, 0xf2, 0xf1, 0xf0, 0xef, 0xee, 0xee, 0xed,
            0xec, 0xeb, 0xea, 0xe9, 0xe8, 0xe8, 0xe7, 0xe6, 0xe5, 0xe4, 0xe3, 0xe2, 0xe1, 0xe0,
        ];
        let spexit = &self.game_state.player.special_exit_position;
        let y_spexit = spexit.y();
        let x_spexit = spexit.x();
        if self.overworld_map_flags() == 0 {
            let j = (-(i32::from(y_spexit >> 4))
                + i32::from(self.game_state.display.ppu_scroll_copy.mode7_center_y())
                + i32::from((y_spexit >> 3) & 1)
                - 0xc0) as usize;
            let yval = 13u16.wrapping_mul(*OVERWORLD_MAP_PROJECTION_CURVE.get(j)? as u16) >> 4;
            let mut at = (x_spexit >> 4) as u8;
            let below = at < 0x80;
            at = at.wrapping_sub(0x80);
            if (at as i8) < 0 {
                at = !at;
            }
            let t1 = (((if yval < 224 { yval } else { 0 }) * 0x54) >> 8) as u8 + 0xb2;
            let t2 = ((u16::from(at) * u16::from(t1)) >> 8) as u8;
            let x = if below {
                0x80u16.wrapping_sub(u16::from(t2))
            } else {
                u16::from(t2).wrapping_add(0x80)
            };
            Some((
                x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_h_copy2())
                    .wrapping_add(0x80),
                yval + 12,
            ))
        } else {
            let t0 = (-(i32::from(y_spexit >> 4))
                + i32::from(self.game_state.display.ppu_scroll_copy.mode7_center_y())
                - 0x80) as u16;
            if t0 >= 0x100 {
                return None;
            }
            let t1 = (t0 * 37) >> 4;
            let yval = *OVERWORLD_MAP_PROJECTION_CURVE.get(t1 as usize)? as u16;
            let mut t2 = x_spexit;
            let below = t2 < 0x7f8;
            t2 = t2.wrapping_sub(0x7f8);
            if (t2 as i16) < 0 {
                t2 = (!t2).wrapping_add(1);
            }
            let t3 = if yval < 226 { yval } else { 0 };
            let t4 = ((t3 * 84) >> 8) + 178;
            let t5 = (((t2 as u8 as u16) * t4) >> 8) as u16;
            let t6 = ((t2 >> 8) * t4).wrapping_add(t5);
            let mut t7 = if below {
                0x800u16.wrapping_sub(t6)
            } else {
                t6.wrapping_add(0x800)
            };
            let below2 = t7 < 0x800;
            t7 = t7.wrapping_sub(0x800);
            let t8 = if below2 { (!t7).wrapping_add(1) } else { t7 };
            let t9 = (((t8 as u8 as u16) * 45) >> 8) as u16;
            let t10 = ((t8 >> 8) * 45).wrapping_add(t9);
            let t11 = if below2 {
                0x80u16.wrapping_sub(t10)
            } else {
                t10.wrapping_add(0x80)
            };
            let xval = t11.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_h_copy2());
            let xt = if self
                .game_state
                .enhanced_features
                .has(FEATURE_EXTEND_SCREEN64_MAP)
            {
                0x48
            } else {
                0
            };
            if xval.wrapping_add(0x80).wrapping_add(xt) >= 0x100 + xt * 2 {
                return None;
            }
            Some((xval.wrapping_add(0x81), yval.wrapping_add(16)))
        }
    }

    pub(super) fn WorldMap_AddSprite(
        &mut self,
        spr: usize,
        big: u8,
        flags: u8,
        ch: u8,
        x: u16,
        y: u16,
    ) {
        let mut big = big;
        let mut flags = flags;
        let mut ch = ch;
        let mut x = x;
        let mut y = y;

        if self.game_state.frame.frame_counter & 0x10 == 0 && ch == 100 {
            assert!(spr >= 8);
            ch = OVERWORLD_MAP_ICON_TILES[spr - 8];
            flags = 0x32;
            big = 0;
        } else {
            x = x.wrapping_sub(4);
            y = y.wrapping_sub(4);
        }
        if self
            .game_state
            .enhanced_features
            .has(FEATURE_EXTEND_SCREEN64_MAP)
        {
            big |= ((x >> 8) as u8) & 1;
        }
        self.set_oam_plain(spr, x as u8, y as u8, ch, flags, big);
    }

    pub(super) fn OverworldMap_CheckForPendant(&self, k: usize) -> bool {
        const PENDANT_BIT_MASK: [u8; 3] = [4, 1, 2];
        self.game_state
            .inventory
            .save_progress
            .map_icons_indicator()
            == 3
            && self.game_state.inventory.player_resources.pendant_flags() & PENDANT_BIT_MASK[k] != 0
    }

    pub(super) fn OverworldMap_CheckForCrystal(&self, k: usize) -> bool {
        const CRYSTAL_BIT_MASK: [u8; 7] = [2, 0x40, 8, 0x20, 1, 4, 0x10];
        self.game_state
            .inventory
            .save_progress
            .map_icons_indicator()
            == 7
            && self.game_state.inventory.player_resources.crystal_flags() & CRYSTAL_BIT_MASK[k] != 0
    }

    pub(super) fn Module0E_03_DungeonMap(&mut self) {
        self.replay_trace_ram_watch("dungmap-before-submodule");
        match self.overworld_map_state() {
            0 => self.DungMap_Backup(),
            1 => self.Module0E_03_01_DrawMap(),
            2 => self.DungMap_LightenUpMap(),
            3 => self.DungeonMap_HandleInputAndSprites(),
            4 => self.DungMap_4(),
            5 => self.DungMap_FadeMapToBlack(),
            6 => self.DungeonMap_RecoverGFX(),
            7 => self.ToggleStarTilesAndAdvance(),
            _ => self.DungMap_RestoreOld(),
        }
        self.replay_trace_ram_watch("dungmap-after-submodule");
    }

    pub(super) fn Module0E_03_01_DrawMap(&mut self) {
        self.replay_trace_ram_watch("dungmap-draw-before-init");
        match self.game_state.dungeon_map_display.dungmap_init_state() {
            0 => self.Module0E_03_01_00_PrepMapGraphics(),
            1 => self.Module0E_03_01_01_DrawLEVEL(),
            2 => self.Module0E_03_01_02_DrawFloorsBackdrop(),
            3 => self.Module0E_03_01_03_DrawRooms(),
            4 => self.DungeonMap_DrawRoomMarkers(),
            // C dispatches through kDungMapInit and asserts in debug if this
            // state is outside the initialized table.
            state => panic!("Module0E_03_01_DrawMap invalid dungmap_init_state {state}"),
        }
        self.replay_trace_ram_watch("dungmap-draw-after-init");
    }

    pub(super) fn Module0E_03_01_00_PrepMapGraphics(&mut self) {
        self.replay_trace_ram_watch("dungmap-prep-entry");
        let hdmaen_bak = self.game_state.display.hdma_enable_mask;
        self.clear_hdma_enable_mask();
        let main_tile_theme = self.game_state.world.palette_theme.main_tile_theme_index();
        let sprite_gfx = self.game_state.sprites.system.graphics_index();
        let aux_tile_theme = self.game_state.world.palette_theme.aux_tile_theme_index();
        let main_layers = self.game_state.display.main_screen_layers;
        let sub_layers = self.game_state.display.sub_screen_layers;
        self.set_mapbak_main_tile_theme_index(main_tile_theme);
        self.set_mapbak_sprite_graphics_index(sprite_gfx);
        self.set_mapbak_aux_tile_theme_index(aux_tile_theme);
        self.set_mapbak_tm(main_layers);
        self.set_mapbak_ts(sub_layers);
        self.world_palette_theme_mut().set_main_tile_theme_index(32);
        let graphics_index =
            0x80 | (self.game_state.inventory.save_progress.palace_index_x2() >> 1);
        self.sprite_system_mut().set_graphics_index(graphics_index);
        self.world_palette_theme_mut().set_aux_tile_theme_index(64);
        self.set_main_screen_layers(0x16);
        self.set_sub_screen_layers(1);
        self.EraseTileMaps_dungeonmap();
        self.InitializeTilesets();
        self.select_overworld_aux_palette_offset();
        self.replay_trace_ram_watch("dungmap-prep-before-bg-palette");
        self.Palette_Load_DungeonMapBG();
        self.replay_trace_ram_watch("dungmap-prep-after-bg-palette");
        self.Palette_Load_DungeonMapSprite();
        self.replay_trace_ram_watch("dungmap-prep-after-sprite-palette");
        self.set_hud_palette(1);
        self.Palette_Load_HUD();
        self.replay_trace_ram_watch("dungmap-prep-after-hud-palette");
        self.LoadActualGearPalettes();
        self.replay_trace_ram_watch("dungmap-prep-after-gear-palette");
        self.increment_cgram_update_flag();
        self.increment_dungeon_map_init_state();
        self.set_hdma_enable_mask(hdmaen_bak);
        self.set_bg_vram_load_mode(9);
        self.set_core_update_disable_flag(9);
        self.replay_trace_ram_watch("dungmap-prep-exit");
    }

    pub(super) fn Module0E_03_01_01_DrawLEVEL(&mut self) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_LEVEL_LABEL_INDEX_BY_DUNGEON.len() - 1);
        let i = DUNGEON_MAP_LEVEL_LABEL_INDEX_BY_DUNGEON[dung] >> 1;
        if i >= 0 {
            let i = i as usize;
            self.write_vram_upload_level_label_tiles(
                &DUNGEON_MAP_LEVEL_LABEL_TOP_STRIPE,
                &DUNGEON_MAP_LEVEL_LABEL_BOTTOM_STRIPE,
            );
            self.write_vram_upload_buffer_word(14, DUNGEON_MAP_LEVEL_LABEL_TOP_TILES[i]);
            self.write_vram_upload_buffer_word(30, DUNGEON_MAP_LEVEL_LABEL_BOTTOM_TILES[i]);
            self.set_bg_vram_load_mode(1);
        }
        self.increment_dungeon_map_init_state();
    }

    pub(super) fn Module0E_03_01_02_DrawFloorsBackdrop(&mut self) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung];
        let mut offs = 0usize;

        if t5 & 0x0100 != 0 {
            for &tile in &DUNGEON_MAP_FLOOR_LIST_HEADER_STRIPE {
                self.write_vram_upload_buffer_word(offs * 2, tile);
                offs += 1;
            }
            let mut t = 0x1123u16;
            for _ in 0..16 {
                self.write_vram_upload_buffer_word(offs * 2, t.swap_bytes());
                self.write_vram_upload_buffer_word((offs + 1) * 2, 0x0e40);
                self.write_vram_upload_buffer_word((offs + 2) * 2, 0x1b2e);
                t = t.wrapping_add(0x20);
                offs += 3;
            }
        }

        let t5_low = t5 as u8;
        let tab7_index = if t5_low >= 0x50 {
            usize::from((t5_low >> 4).wrapping_sub(4))
        } else if t5 & 0x0f >= 5 {
            usize::from(t5 & 0x0f)
        } else {
            0
        };
        let mut t7 = DUNGEON_MAP_FLOOR_LIST_VRAM_STARTS[tab7_index];
        let t7_org = t7;
        let mut j = 0usize;
        loop {
            self.write_vram_upload_buffer_word(offs * 2, t7.swap_bytes());
            offs += 1;
            self.write_vram_upload_buffer_word(offs * 2, 0x0e40);
            offs += 1;
            self.write_vram_upload_buffer_word(
                offs * 2,
                DUNGEON_MAP_FLOOR_LIST_LABEL_TILES[j] + if t5 & 0x0200 != 0 { 0x0400 } else { 0 },
            );
            offs += 1;
            if j != 6 {
                j += 1;
            }
            t7 = t7.wrapping_add(0x20);
            if t7 >= 0x1360 {
                break;
            }
        }
        self.set_vram_upload_cursor((offs * 2) as u16);
        self.DungeonMap_BuildFloorListBoxes(t5 as u8, t7_org);
        let offset = self.game_state.display.vram_upload_cursor_usize();
        self.terminate_vram_upload_buffer_at(offset);
        self.increment_dungeon_map_init_state();
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn DungeonMap_BuildFloorListBoxes(&mut self, t5: u8, mut r14: u16) {
        let n = usize::from((t5 & 0x0f).wrapping_add(t5 >> 4)).max(1);
        r14 = r14
            .wrapping_sub(0x40 - 2)
            .wrapping_add(u16::from(t5 & 0x0f) * 0x40);
        let mut offs = self.game_state.display.vram_upload_cursor_usize() >> 1;
        for _ in 0..n {
            self.write_vram_upload_buffer_word(offs * 2, r14.swap_bytes());
            offs += 1;
            self.write_vram_upload_buffer_word(offs * 2, 0x0700);
            offs += 1;
            for (x, &tile) in DUNGEON_MAP_FLOOR_LIST_BOX_TILES.iter().enumerate() {
                self.write_vram_upload_buffer_word(offs * 2, tile);
                offs += 1;
                if x == 3 {
                    r14 = r14.wrapping_add(0x20);
                    self.write_vram_upload_buffer_word(offs * 2, r14.swap_bytes());
                    offs += 1;
                    self.write_vram_upload_buffer_word(offs * 2, 0x0700);
                    offs += 1;
                }
            }
            r14 = r14.wrapping_sub(0x40 + 0x20);
        }
        self.set_vram_upload_cursor((offs * 2) as u16);
    }

    pub(super) fn Module0E_03_01_03_DrawRooms(&mut self) {
        self.clear_dungeon_map_floor_scroll_step();
        self.clear_dungeon_map_idx();
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t =
            (-(i16::from((DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung] & 0x0f) as u8)) as u16) as u8;
        if self.game_state.dungeon.stair_movement.current_floor_word() != u16::from(t) {
            let dung_cur_floor = u16::from(self.game_state.dungeon.stair_movement.current_floor());
            self.set_dungeon_map_current_floor(dung_cur_floor);
        } else {
            let dung_cur_floor = self
                .game_state
                .dungeon
                .stair_movement
                .current_floor_word()
                .wrapping_add(1);
            let dungmap_idx = self
                .game_state
                .dungeon_map_display
                .dungmap_idx()
                .wrapping_add(2);
            self.set_dungeon_map_current_floor(dung_cur_floor);
            self.set_dungeon_map_idx(dungmap_idx);
        }
        self.DungeonMap_DrawFloorNumbersByRoom(0, !0x1000);
        self.DungeonMap_DrawBorderForRooms(0, !0x1000);
        self.DungeonMap_DrawDungeonLayout(0);
        self.decrement_dungeon_map_current_floor_byte();
        self.DungeonMap_DrawFloorNumbersByRoom(0x0300, !0x1000);
        self.DungeonMap_DrawBorderForRooms(0x0300, !0x1000);
        self.DungeonMap_DrawDungeonLayout(0x0300);
        let dungmap_cur_floor = self
            .game_state
            .dungeon_map_display
            .dungmap_cur_floor()
            .wrapping_add(1);
        self.set_dungeon_map_current_floor(dungmap_cur_floor);
        self.clear_dungeon_map_scroll_state();
        self.set_pending_nmi_subroutine(8);
        self.set_nmi_load_target_page(0x22);
        self.increment_dungeon_map_init_state();
    }

    pub(super) fn DungeonMap_DrawBorderForRooms(&mut self, pd: u16, mask: u16) {
        for i in 0..4 {
            let idx = (((DUNGEON_MAP_ROOM_BORDER_CORNER_POSITIONS[i].wrapping_add(pd)) & 0x0fff)
                >> 1) as usize;
            self.set_messaging_render_buffer_word(
                idx,
                DUNGEON_MAP_ROOM_BORDER_CORNER_TILES[i] & mask,
            );
        }
        for i in 0..2 {
            let r4 = DUNGEON_MAP_ROOM_BORDER_HORIZONTAL_POSITIONS[i].wrapping_add(pd);
            for j in (0..20u16).step_by(2) {
                let idx = (((r4.wrapping_add(j)) & 0x0fff) >> 1) as usize;
                self.set_messaging_render_buffer_word(
                    idx,
                    DUNGEON_MAP_ROOM_BORDER_HORIZONTAL_TILES[i] & mask,
                );
            }
        }
        for i in 0..2 {
            let r4 = DUNGEON_MAP_ROOM_BORDER_VERTICAL_POSITIONS[i].wrapping_add(pd);
            for j in (0..0x280u16).step_by(0x40) {
                let idx = (((r4.wrapping_add(j)) & 0x0fff) >> 1) as usize;
                self.set_messaging_render_buffer_word(
                    idx,
                    DUNGEON_MAP_ROOM_BORDER_VERTICAL_TILES[i] & mask,
                );
            }
        }
    }

    pub(super) fn DungeonMap_DrawFloorNumbersByRoom(&mut self, pd: u16, r8: u16) {
        let mut p = 0x00deu16;
        loop {
            let t = (((p.wrapping_add(pd)) & 0x0fff) >> 1) as usize;
            self.set_messaging_render_buffer_word(t, 0x0f00);
            self.set_messaging_render_buffer_word(t + 1, 0x0f00);
            p = p.wrapping_add(0x40);
            if p == 0x039e {
                break;
            }
        }
        let t = (((0x035eu16.wrapping_add(pd)) & 0x0fff) >> 1) as usize;
        let floor = self.game_state.dungeon_map_display.dungmap_cur_floor();
        let (q1, q2) = if (floor & 0x80) != 0 {
            (
                0x1f1c,
                DUNGEON_MAP_FLOOR_NUMBER_TILES[usize::from((!(floor as u8)) & 0x07)],
            )
        } else {
            (
                DUNGEON_MAP_FLOOR_NUMBER_TILES[usize::from(floor & 0x0f)],
                0x1f1d,
            )
        };
        self.set_messaging_render_buffer_word(t, q1 & r8);
        self.set_messaging_render_buffer_word(t + 1, q2 & r8);
    }

    pub(super) fn DungeonMap_DrawDungeonLayout(&mut self, pd: i32) {
        for i in 0..5 {
            let arg_x = ((292 + 128 * i + pd) & 0x0fff) >> 1;
            self.DungeonMap_DrawSingleRowOfRooms(i, arg_x);
        }
    }

    pub(super) fn DungeonMap_DrawSingleRowOfRooms(&mut self, i: i32, mut arg_x: i32) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung];
        let dungmask = DUNG_MAP_UPPER_BITMASKS[dung & 0x0f];
        let curp = self.GetDungmapFloorLayout();
        let has_map = self
            .game_state
            .inventory
            .player_resources
            .has_dungeon_map_mask(dungmask);

        for j in 0..5 {
            let mut r14 = self
                .game_state
                .dungeon_map_display
                .dungmap_cur_floor_byte()
                .wrapping_add((t5 & 0x0f) as u8);
            let room_index = usize::from(r14) * 25 + (i as usize) * 5 + j as usize;
            let v = curp.get(room_index).copied().unwrap_or(0x0f);
            let yv = if v == 0x0f {
                0x51
            } else {
                r14 = (self
                    .game_state
                    .inventory
                    .save_progress
                    .dungeon_info_word(usize::from(v))
                    & 0x0f) as u8;
                let mut k = 0usize;
                let mut count = 0usize;
                while k < curp.len() && curp[k] != v {
                    count += usize::from(curp[k] != 0x0f);
                    k += 1;
                }
                self.GetOtherDungmapInfo(count)
            };

            let base = usize::from(yv) * 4;
            let av0 =
                self.dungeon_map_room_tile(DUNGEON_MAP_ROOM_QUADRANT_TILES[base], r14, 8, has_map);
            let av1 = self.dungeon_map_room_tile(
                DUNGEON_MAP_ROOM_QUADRANT_TILES[base + 1],
                r14,
                4,
                has_map,
            );
            let av2 = self.dungeon_map_room_tile(
                DUNGEON_MAP_ROOM_QUADRANT_TILES[base + 2],
                r14,
                2,
                has_map,
            );
            let av3 = self.dungeon_map_room_tile(
                DUNGEON_MAP_ROOM_QUADRANT_TILES[base + 3],
                r14,
                1,
                has_map,
            );
            let idx = arg_x as usize;
            self.set_messaging_render_buffer_word(idx, av0);
            self.set_messaging_render_buffer_word(idx + 1, av1);
            self.set_messaging_render_buffer_word(idx + 32, av2);
            self.set_messaging_render_buffer_word(idx + 33, av3);
            arg_x += 2;
        }
    }

    fn dungeon_map_room_tile(&self, mut r12: u16, r14: u8, bit: u8, has_map: bool) -> u16 {
        let r12_org = r12;
        if r12 != 0x0b00 && (r14 & bit) == 0 {
            if (r12 & 0x1000) == 0 {
                r12 = 0x0400;
            } else if has_map {
                return (r12 & !0x1c00) | 0x0c00;
            } else {
                r12 = 0;
            }
        } else {
            r12 = 0;
        }
        if has_map || (r14 & bit) != 0 {
            r12.wrapping_add(r12_org)
        } else {
            0x0b00
        }
    }

    pub(super) fn DungeonMap_DrawRoomMarkers(&mut self) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t5 = (DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung] & 0x0f) as u8;
        let floor1 = t5.wrapping_add(self.game_state.dungeon.stair_movement.current_floor());

        let mut room = self.game_state.world.location.dungeon_room();
        for i in 0..3 {
            if room == DUNGEON_MAP_ROOM_REMAP_FROM[i] {
                room = DUNGEON_MAP_ROOM_REMAP_TO[i];
            }
        }

        let roomp = self.GetDungmapFloorLayout();
        let mut xcoord = 0u8;
        let mut ycoord = 0u8;
        let base = usize::from(floor1) * 25;
        for i in 0..25 {
            if roomp.get(base + i).copied().unwrap_or(0x0f) == room as u8 {
                break;
            }
            if xcoord < 64 {
                xcoord = xcoord.wrapping_add(16);
            } else {
                xcoord = 0;
                ycoord = ycoord.wrapping_add(16);
            }
        }

        let marker_x = u16::from(xcoord)
            .wrapping_add(0x90)
            .wrapping_add((self.game_state.player.follower_link.x() & 0x01e0) >> 5);
        self.set_dungeon_map_player_marker_x(marker_x);
        self.set_dungeon_map_location_marker_base_y(ycoord);

        let idx = usize::from((self.game_state.dungeon_map_display.dungmap_idx() >> 1) & 1);
        let marker_y = u16::from(ycoord)
            .wrapping_add(DUNGEON_MAP_MARKER_Y_BASES[idx])
            .wrapping_add((self.game_state.player.follower_link.y() & 0x01e0) >> 5);
        self.set_dungeon_map_player_marker_y(marker_y);

        let floor2 = t5.wrapping_add(DUNGEON_MAP_BOSS_FLOOR_OFFSETS[dung] as u8);
        let marker_base = usize::from(floor2) * 25;
        self.reset_dungeon_map_marker_offsets();

        let lookfor = DUNGEON_MAP_BOSS_ROOM_BY_DUNGEON[dung];
        for j in (0..25).rev() {
            let value = roomp.get(marker_base + j).copied().unwrap_or(0x0f);
            if value != 0x0f && value == lookfor {
                break;
            }
            let marker_x_offset = self.shift_dungeon_map_marker_x_left();
            if (marker_x_offset as i16) < 0 {
                self.reset_dungeon_map_marker_x_and_shift_marker_y_low_up();
            }
        }

        let floor3 = (self.game_state.dungeon_map_display.dungmap_cur_floor_byte() as i8)
            .wrapping_sub(DUNGEON_MAP_BOSS_FLOOR_OFFSETS[dung] as i8);
        let marker_y_offset = self
            .game_state
            .dungeon_map_display
            .marker_y_offset()
            .wrapping_add_signed(i16::from(floor3) * 0x60)
            .wrapping_add(DUNGEON_MAP_MARKER_Y_BASES[0]);
        self.set_dungeon_map_marker_y_offset(marker_y_offset);
        self.increment_overworld_map_state();
        self.set_screen_brightness(0);
        self.clear_dungeon_map_init_state();
    }

    pub(super) fn DungeonMap_HandleInputAndSprites(&mut self) {
        self.DungeonMap_HandleInput();
        self.DungeonMap_DrawSprites();
    }

    pub(super) fn DungeonMap_HandleInput(&mut self) {
        if self.WantExitDungeonMap() {
            let overworld_map_state = self.overworld_map_state().wrapping_add(2);
            self.set_overworld_map_state(overworld_map_state);
            self.clear_dungeon_map_init_state();
        } else {
            self.DungeonMap_HandleMovementInput();
        }
    }

    fn WantExitDungeonMap(&self) -> bool {
        if self.game_state.world.transient.hud_cur_item_x() != 0 {
            self.game_state.player.follower_link.filtered_joypad_h() & 0x20 != 0
        } else {
            self.game_state.player.follower_link.filtered_joypad_l() & 0x40 != 0
        }
    }

    pub(super) fn DungeonMap_HandleMovementInput(&mut self) {
        self.DungeonMap_HandleFloorSelect();
        if self
            .game_state
            .dungeon_map_display
            .dungmap_floor_scroll_step()
            != 0
        {
            self.DungeonMap_ScrollFloors();
        }
    }

    pub(super) fn DungeonMap_HandleFloorSelect(&mut self) {
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[usize::from(
            self.game_state.inventory.save_progress.palace_index_x2() >> 1,
        )
        .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1)];
        let r2 = ((t5 >> 4) & 0x0f) as u8;
        let r3 = (t5 & 0x0f) as u8;
        if r2.wrapping_add(r3) < 3
            || self
                .game_state
                .dungeon_map_display
                .dungmap_floor_scroll_step()
                != 0
            || (self.game_state.player.follower_link.joypad1h_last() & 0x0c) == 0
        {
            return;
        }

        self.dungeon_map_mut().clear_current_floor_high();
        let mut scroll_draw_offset = self.game_state.dungeon_map_display.scroll_draw_offset();
        if (self.game_state.player.follower_link.joypad1h_last() & 8) != 0 {
            if r2.wrapping_sub(1) == self.game_state.dungeon_map_display.dungmap_cur_floor_byte() {
                return;
            }
            self.increment_dungeon_map_current_floor_byte();
            scroll_draw_offset = scroll_draw_offset.wrapping_sub(0x300) & 0x0fff;
        } else {
            if (!r3).wrapping_add(1) == self.game_state.dungeon_map_display.dungmap_cur_floor_byte()
            {
                return;
            }
            let new_floor = self
                .game_state
                .dungeon_map_display
                .dungmap_cur_floor()
                .wrapping_sub(2);
            self.set_dungeon_map_current_floor(new_floor);
            scroll_draw_offset = scroll_draw_offset.wrapping_add(0x600) & 0x0fff;
        }

        self.DungeonMap_DrawFloorNumbersByRoom(scroll_draw_offset, !0x1000);
        self.DungeonMap_DrawBorderForRooms(scroll_draw_offset, !0x1000);
        self.DungeonMap_DrawDungeonLayout(scroll_draw_offset as i32);
        self.increment_dungeon_map_floor_scroll_step();
        let joypad_h = self.game_state.player.follower_link.joypad1h_last();
        self.set_dungeon_map_scroll_input(u16::from(joypad_h));
        let x = usize::from((joypad_h >> 3) & 1);
        let target = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add_signed(DUNGEON_MAP_FLOOR_SCROLL_TARGET_DELTAS[x]);
        self.set_dungeon_map_scroll_target_y(target);
        if x == 0 {
            scroll_draw_offset = scroll_draw_offset.wrapping_sub(0x300) & 0x0fff;
            self.increment_dungeon_map_current_floor_byte();
        }
        self.set_dungeon_map_scroll_draw_offset(scroll_draw_offset);
        self.set_pending_nmi_subroutine(8);
    }

    pub(super) fn DungeonMap_ScrollFloors(&mut self) {
        let x = self
            .game_state
            .dungeon_map_display
            .scroll_input_direction_index();
        let marker_y = self
            .game_state
            .dungeon_map_display
            .dungmap_player_marker_y()
            .wrapping_add_signed(i16::from(DUNGEON_MAP_SCROLL_MARKER_Y_DELTAS[x]));
        self.set_dungeon_map_player_marker_y(marker_y);
        self.add_dungeon_map_marker_y_offset_signed(i16::from(
            DUNGEON_MAP_SCROLL_MARKER_Y_DELTAS[x],
        ));
        let bg2 = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add_signed(i16::from(DUNGEON_MAP_SCROLL_BG_Y_DELTAS[x]));
        self.set_bg2_y(bg2);
        if bg2
            == self
                .game_state
                .dungeon_map_display
                .dungmap_scroll_target_y()
        {
            self.clear_dungeon_map_floor_scroll_step();
        }
    }

    pub(super) fn DungeonMap_DrawSprites(&mut self) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let r2 = (DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung] & 0x0f) as u8;
        let floor = r2.wrapping_add(self.game_state.dungeon.stair_movement.current_floor());

        let mut spr_pos = 0usize;
        let mut r14 = 0u16;
        self.DungeonMap_DrawLinkPointing(spr_pos, r2, floor);
        spr_pos += 1;
        loop {
            spr_pos = self.DungeonMap_DrawLocationMarker(spr_pos, r14);
            r14 = r14.wrapping_add(1);
            if spr_pos == 9 {
                break;
            }
        }
        spr_pos = self.DungeonMap_DrawBlinkingIndicator(spr_pos);
        spr_pos = self.DungeonMap_DrawBossIcon(spr_pos);
        let _ = self.DungeonMap_DrawFloorNumberObjects(spr_pos);
        self.DungeonMap_DrawFloorBlinker();
    }

    pub(super) fn DungeonMap_DrawLinkPointing(&mut self, spr_pos: usize, r2: u8, mut r3: u8) {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung] as u8;
        if 4i8.wrapping_sub(r2 as i8) >= 0 {
            r3 = r3.wrapping_add(4u8.wrapping_sub(r2));
            let a = ((t5 >> 4) as i8).wrapping_sub(4);
            if a >= 0 {
                r3 = r3.wrapping_sub(a as u8);
            }
        }
        let y = DUNGEON_MAP_FLOOR_Y_POSITIONS[usize::from(r3)].wrapping_sub(4);
        let flags = if self.ram[crate::game_state::constants::PALETTE_SWAP_FLAG] != 0 {
            0x30
        } else {
            0x3e
        };
        self.set_oam_plain(spr_pos, 0x19, y, 0, flags, 2);
    }

    pub(super) fn DungeonMap_DrawBlinkingIndicator(&mut self, spr_pos: usize) -> usize {
        let marker_y = self
            .game_state
            .dungeon_map_display
            .dungmap_player_marker_y();
        let y = if marker_y < 256 { marker_y as u8 } else { 0xf0 }.wrapping_sub(3);
        self.set_oam_plain(
            spr_pos,
            self.game_state
                .dungeon_map_display
                .dungmap_player_marker_x_byte()
                .wrapping_sub(3),
            y,
            0x34,
            DUNGEON_MAP_PLAYER_MARKER_OAM_FLAGS
                [usize::from((self.game_state.frame.frame_counter >> 2) & 3)],
            0,
        );
        spr_pos + 1
    }

    pub(super) fn DungeonMap_DrawLocationMarker(&mut self, mut spr_pos: usize, r14: u16) -> usize {
        for i in (0..4).rev() {
            let r15 = self
                .game_state
                .dungeon_map_display
                .location_marker_base_y()
                .wrapping_add(DUNGEON_MAP_MARKER_Y_BASES[usize::from(r14)] as u8);
            let mut fr = (self.game_state.frame.frame_counter >> 2) & 1;
            let marker_y = self
                .game_state
                .dungeon_map_display
                .dungmap_player_marker_y();
            if ((marker_y.wrapping_add(1)) & 0x00f0) == u16::from(r15.wrapping_add(1))
                && marker_y < 256
            {
                fr = fr.wrapping_add(2);
            }
            let x = (self
                .game_state
                .dungeon_map_display
                .dungmap_player_marker_x()
                & 0x00f0)
                .wrapping_add_signed(i16::from(DUNGEON_MAP_LOCATION_MARKER_X_OFFSETS[i]))
                as u8;
            let y = u16::from(r15)
                .wrapping_add_signed(i16::from(DUNGEON_MAP_LOCATION_MARKER_Y_OFFSETS[i]))
                as u8;
            self.set_oam_plain(
                spr_pos,
                x,
                y,
                0,
                DUNGEON_MAP_LOCATION_MARKER_CHARS[usize::from(fr)]
                    | DUNGEON_MAP_LOCATION_MARKER_OAM_FLAGS[i],
                2,
            );
            spr_pos += 1;
        }
        spr_pos
    }

    pub(super) fn DungeonMap_DrawFloorNumberObjects(&mut self, spr_pos: usize) -> usize {
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[usize::from(
            self.game_state.inventory.save_progress.palace_index_x2() >> 1,
        )
        .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1)];
        let mut r2 = ((t5 >> 4) & 0x0f) as u8;
        let mut r3 = (t5 & 0x0f) as u8;
        let mut yv = 7u8;
        if r2.wrapping_add(r3) != 8 && r2 < 4 {
            yv = 6;
            let mut i = 3u8;
            while i != 0 && i != r2 {
                yv = yv.wrapping_sub(1);
                i = i.wrapping_sub(1);
            }
            if r3 >= 5 {
                let mut i = 5u8;
                while i != r3 && r3 != 8 {
                    yv = yv.wrapping_add(1);
                    i = i.wrapping_add(1);
                }
            }
        }

        let mut r4 = DUNGEON_MAP_FLOOR_Y_POSITIONS[usize::from(yv)].wrapping_add(1);
        r2 = r2.wrapping_sub(1);
        r3 = 0u8.wrapping_sub(r3);
        let mut pos = spr_pos;
        loop {
            let left = if (r2 as i8) < 0 {
                0x1c
            } else {
                DUNGEON_MAP_FLOOR_DIGIT_CHARS[usize::from(r2)]
            };
            let right = if (r2 as i8) < 0 {
                DUNGEON_MAP_FLOOR_DIGIT_CHARS[usize::from(r2 ^ 0xff)]
            } else {
                0x1d
            };
            self.set_oam_plain(pos, 0x30, r4, left, 0x3d, 0);
            self.set_oam_plain(pos + 1, 0x38, r4, right, 0x3d, 0);
            r4 = r4.wrapping_add(16);
            let done = r2 == r3;
            pos += 2;
            r2 = r2.wrapping_sub(1);
            if done {
                break;
            }
        }
        pos
    }

    pub(super) fn DungeonMap_DrawFloorBlinker(&mut self) {
        let mut floor = self.game_state.dungeon_map_display.dungmap_cur_floor_byte();
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[usize::from(
            self.game_state.inventory.save_progress.palace_index_x2() >> 1,
        )
        .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1)] as u8;
        let mut flag = u8::from(((t5 >> 4) & 0x0f).wrapping_add(t5 & 0x0f) != 1);
        floor = floor.wrapping_sub(flag);
        let mut r0;
        let mut i = flag;
        loop {
            r0 = floor.wrapping_add(t5 & 0x0f);
            let a = 4i8.wrapping_sub((t5 & 0x0f) as i8);
            if a >= 0 {
                r0 = r0.wrapping_add(a as u8);
                let a = ((t5 >> 4) as i8).wrapping_sub(4);
                if a >= 0 {
                    r0 = r0.wrapping_sub(a as u8);
                }
            }
            floor = floor.wrapping_add(1);
            if i == 0 {
                break;
            }
            i = i.wrapping_sub(1);
        }
        if (self.game_state.frame.frame_counter & 0x10) == 0 {
            return;
        }
        let y = DUNGEON_MAP_FLOOR_Y_POSITIONS[usize::from(r0)].wrapping_sub(4);
        loop {
            let mut x = 40u8;
            let mut spr_pos =
                0x40 + usize::from(DUNGEON_MAP_FLOOR_BLINKER_SPRITE_OFFSETS[usize::from(flag)]);
            for i in (0..4).rev() {
                let t = 0x3d | if i != 0 { 0 } else { 0x40 };
                self.set_oam_plain(
                    spr_pos,
                    x,
                    y.wrapping_add(flag.wrapping_mul(16)),
                    DUNGEON_MAP_FLOOR_BLINKER_CHARS[i],
                    t,
                    0,
                );
                self.set_oam_plain(
                    spr_pos + 4,
                    x,
                    y.wrapping_add(flag.wrapping_mul(16)).wrapping_add(8),
                    DUNGEON_MAP_FLOOR_BLINKER_CHARS[i],
                    t | 0x80,
                    0,
                );
                x = x.wrapping_add(8);
                spr_pos += 1;
            }
            if flag == 0 {
                break;
            }
            flag = flag.wrapping_sub(1);
        }
    }

    pub(super) fn DungeonMap_DrawBossIcon(&mut self, spr_pos: usize) -> usize {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        if (self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(usize::from(DUNGEON_MAP_BOSS_ROOM_BY_DUNGEON[dung]))
            & 0x0800)
            != 0
            || !self
                .game_state
                .inventory
                .player_resources
                .has_compass_mask(DUNG_MAP_UPPER_BITMASKS[dung & 0x0f])
            || DUNGEON_MAP_BOSS_FLOOR_OFFSETS[dung] < 0
        {
            return spr_pos;
        }
        let spr_pos = self.DungeonMap_DrawBossIconByFloor(spr_pos);
        if (self.game_state.frame.frame_counter & 0x0f) >= 10 {
            return spr_pos;
        }
        let xy = DUNGEON_MAP_BOSS_ICON_XY_BY_DUNGEON[dung];
        let x = (xy >> 8)
            .wrapping_add(self.game_state.dungeon_map_display.marker_x_offset())
            .wrapping_add(0x90) as u8;
        let marker_y_offset = self.game_state.dungeon_map_display.marker_y_offset();
        let y = if marker_y_offset < 256 {
            xy.wrapping_add(marker_y_offset) as u8
        } else {
            0xf0
        };
        self.set_oam_plain(spr_pos, x, y, 0x31, 0x33, 0);
        spr_pos + 1
    }

    pub(super) fn DungeonMap_DrawBossIconByFloor(&mut self, spr_pos: usize) -> usize {
        let dung = usize::from(self.game_state.inventory.save_progress.palace_index_x2() >> 1)
            .min(DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON.len() - 1);
        let t5 = DUNGEON_MAP_FLOOR_RANGE_BY_DUNGEON[dung] as u8;
        let r2 = t5 & 0x0f;
        let mut r3 = r2.wrapping_add(DUNGEON_MAP_BOSS_FLOOR_OFFSETS[dung] as u8);
        if 4i8.wrapping_sub(r2 as i8) >= 0 {
            r3 = r3.wrapping_add(4u8.wrapping_sub(r2));
            let a = ((t5 >> 4) as i8).wrapping_sub(4);
            if a >= 0 {
                r3 = r3.wrapping_sub(a as u8);
            }
        }
        if (self.game_state.frame.frame_counter & 0x0f) >= 10 {
            return spr_pos;
        }
        self.set_oam_plain(
            spr_pos,
            0x4c,
            DUNGEON_MAP_FLOOR_Y_POSITIONS[usize::from(r3)],
            0x31,
            0x33,
            0,
        );
        spr_pos + 1
    }

    pub(super) fn DungeonMap_RecoverGFX(&mut self) {
        let hdmaen_bak = self.game_state.display.hdma_enable_mask;
        self.clear_hdma_enable_mask();
        self.EraseTileMaps_normal();

        let main_screen_layers = self.game_state.display.ppu_scroll_copy.mapbak_tm();
        let sub_screen_layers = self.game_state.display.ppu_scroll_copy.mapbak_ts();
        self.set_main_screen_layers(main_screen_layers);
        self.set_sub_screen_layers(sub_screen_layers);
        let main_tile_theme = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_main_tile_theme_index();
        self.world_palette_theme_mut()
            .set_main_tile_theme_index(main_tile_theme);
        let graphics_index = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_sprite_graphics_index();
        self.sprite_system_mut().set_graphics_index(graphics_index);
        let aux_tile_theme = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_aux_tile_theme_index();
        self.world_palette_theme_mut()
            .set_aux_tile_theme_index(aux_tile_theme);
        self.InitializeTilesets();
        self.clear_overworld_aux_or_main_offset();
        self.set_hud_palette(0);
        self.hud_rebuild();

        self.clear_screen_transition();
        self.dungeon_room_load_mut().clear_quadrant_upload_index();
        loop {
            self.WaterFlood_BuildOneQuadrantForVRAM();
            self.upload_tilemap_now();
            self.Dungeon_PrepareNextRoomQuadrantUpload();
            self.upload_tilemap_now();
            if self.game_state.dungeon.room_load.quadrant_upload_index() == 0x10 {
                break;
            }
        }

        self.clear_pending_nmi_subroutine();
        self.set_subsubmodule(0);
        self.set_hdma_enable_mask(hdmaen_bak);
        let mapbak_palette = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_palette_slice()
            .to_vec();
        self.copy_main_full_from(&mapbak_palette);
        let fixed_color_plusminus = self.game_state.display.overworld_fixed_color_adjustment;
        self.or_fixed_color_red(fixed_color_plusminus);
        self.or_fixed_color_green(fixed_color_plusminus);
        self.or_fixed_color_blue(fixed_color_plusminus);

        self.set_sound_effect_2(16);
        self.set_music_control(0xf3);
        self.RecoverPegGFXFromMapping();
        self.increment_cgram_update_flag();
        self.increment_overworld_map_state();
        self.set_screen_brightness(0);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn ToggleStarTilesAndAdvance(&mut self) {
        self.ResetStarTileGraphics();
        self.increment_overworld_map_state();
    }

    pub(super) fn DungMap_4(&mut self) {
        let scroll_target = self
            .game_state
            .dungeon_map_display
            .dungmap_scroll_target_y();
        let y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add(scroll_target);
        self.set_bg2_y(y);
        let marker_y = self
            .game_state
            .dungeon_map_display
            .dungmap_player_marker_y()
            .wrapping_sub(scroll_target);
        self.set_dungeon_map_player_marker_y(marker_y);
        let new_row = self.decrement_bottle_menu_row();
        if new_row == 0 {
            let overworld_map_state = self.overworld_map_state().wrapping_sub(1);
            self.set_overworld_map_state(overworld_map_state);
        }
    }

    pub(super) fn DungMap_LightenUpMap(&mut self) {
        self.increment_screen_brightness();
        if self.game_state.display.screen_brightness == 0x0f {
            self.increment_overworld_map_state();
        }
    }

    pub(super) fn DungMap_Backup(&mut self) {
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        self.set_mosaic_copy(3);
        let hdmaen = self.game_state.display.hdma_enable_mask;
        self.set_mapbak_hdmaen(hdmaen);
        self.EnableForceBlank();
        self.increment_overworld_map_state();
        self.clear_dungeon_map_init_state();
        self.set_fixed_color_red(0x20);
        self.set_fixed_color_green(0x40);
        self.set_fixed_color_blue(0x80);
        self.follower_link_state_mut()
            .set_link_dma_graphics_index_word(0x0250);
        let palette = self
            .game_state
            .display
            .palette_buffer
            .main_full_slice()
            .to_vec();
        self.copy_mapbak_palette_from(&palette);
        let bg1_x_offset = self.game_state.world.scroll.bg1_x_offset();
        let bg1_y_offset = self.game_state.world.scroll.bg1_y_offset();
        self.set_mapbak_bg1_x_offset(bg1_x_offset);
        self.set_mapbak_bg1_y_offset(bg1_y_offset);
        self.set_bg1_x_offset(0);
        self.set_bg1_y_offset(0);
        let bg1hofs = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
        let bg2hofs = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg1vofs = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
        let bg2vofs = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.set_map_backup_scrolls(bg1hofs, bg2hofs, bg1vofs, bg2vofs);
        self.set_bg1_x(0);
        self.set_bg1_y(0);
        self.set_bg2_x(0);
        self.set_bg2_y(0);
        self.set_bg3_h_copy2(0);
        self.set_bg3_v_copy2(0);
        let cgwsel = self
            .game_state
            .display
            .palette_filter
            .color_window_and_math_word();
        self.set_mapbak_cgwsel_word(cgwsel);
        self.set_color_window_selection(0x02);
        self.set_color_math_control(0x20);
        self.fill_messaging_render_buffer_word_range(0, 2048, 0x0300);
        self.set_sound_effect_2(16);
        self.set_music_control(0xf2);
    }

    pub(super) fn DungMap_FadeMapToBlack(&mut self) {
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        self.EnableForceBlank();
        self.increment_overworld_map_state();
        let cgwsel = self.game_state.display.ppu_scroll_copy.mapbak_cgwsel_word();
        let bg1hofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg1_h_copy2();
        let bg2hofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg2_h_copy2();
        let bg1vofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg1_v_copy2();
        let bg2vofs = self
            .game_state
            .display
            .ppu_scroll_copy
            .map_backup_bg2_v_copy2();
        self.set_color_window_and_math_word(cgwsel);
        self.set_bg1_x(bg1hofs);
        self.set_bg2_x(bg2hofs);
        self.set_bg1_y(bg1vofs);
        self.set_bg2_y(bg2vofs);
        self.set_bg3_v_copy2(0);
        self.set_bg3_h_copy2(0);
        let bg1_x_offset = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_bg1_x_offset();
        let bg1_y_offset = self
            .game_state
            .display
            .ppu_scroll_copy
            .mapbak_bg1_y_offset();
        self.set_bg1_x_offset(bg1_x_offset);
        self.set_bg1_y_offset(bg1_y_offset);
        self.increment_cgram_update_flag();
    }

    pub(super) fn DungMap_RestoreOld(&mut self) {
        self.OrientLampLightCone();
        self.increment_screen_brightness();
        if self.game_state.display.screen_brightness != 0x0f {
            return;
        }
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
        self.set_submodule(0);
        self.set_overworld_map_state(0);
        self.set_subsubmodule(0);
        self.set_screen_brightness(0x0f);
        let hdma_enable_mask = self.game_state.display.ppu_scroll_copy.mapbak_hdmaen();
        self.set_hdma_enable_mask(hdma_enable_mask);
    }

    pub(super) fn Death_InitializeGameOverLetters(&mut self) {
        self.minigame_state_mut().set_flag_boomerang_in_place(0);
        for i in 0..8 {
            self.game_state
                .sprites
                .ancilla_slots
                .slot_mut(&mut self.ram, i)
                .set_x(0xb0);
        }
        self.game_state
            .sprites
            .ancilla_slots
            .slot_mut(&mut self.ram, 0)
            .set_ancilla_type(1);
        self.messaging_state_mut().set_game_over_letter_cursor(6);
    }

    pub(super) fn CopySaveToWRAM(&mut self) {
        let k = 0x0f;
        self.clear_bird_travel_destination(k);
        self.clear_bird_travel_stop_status(k);

        let save_offset = self.game_state.save_load_transfer.source_offset_usize();
        if save_offset + 0x500 <= self.sram.len() {
            let save = self.sram[save_offset..save_offset + 0x500].to_vec();
            self.save_progress_mut().copy_dungeon_info_from(&save);
        }

        self.set_bg_tile_animation_countdown(7);
        self.follower_link_state_mut()
            .reset_link_dma_animation_cycle(7);
        self.set_message_dma_destination_address(0x6040);
        self.set_message_dma_tile_base(0x4841);
        self.set_message_dma_tile_limit(0x007f);
        self.set_message_dma_tile_sentinel(0xffff);

        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self
            .game_state
            .enhanced_features
            .has(FEATURES0_MISC_BUG_FIXES)
        {
            self.clear_mosaic_level();
        }

        self.save_progress_mut().request_post_message_refresh();
        self.set_main_module(5);
        self.set_submodule(0);
        self.set_which_entrance(0);
        self.clear_core_update_disable_flag();
        self.set_hud_palette(0);
    }

    pub(super) fn RenderText(&mut self) {
        match self.game_state.messaging.runtime.module() {
            0 => self.Text_Initialize(),
            1 => self.Text_Render(),
            2 => self.RenderText_PostDeathSaveOptions(),
            _ => {}
        }
    }

    pub(super) fn RenderText_PostDeathSaveOptions(&mut self) {
        self.dialogue_message_index_mut().set_value(3);
        self.Text_Initialize_initModuleStateLoop();
        self.messaging_state_mut().set_text_msgbox_topleft(0x61e8);
        self.messaging_state_mut().set_text_render_state(2);
        for _ in 0..5 {
            self.Text_Render();
        }
    }

    pub(super) fn Text_Initialize(&mut self) {
        if self.game_state.frame.main_module == 20 {
            self.ResetHUDPalettes4and5();
        }
        self.Attract_DecompressStoryGFX();
        self.Text_Initialize_initModuleStateLoop();
    }

    pub(super) fn Text_Initialize_initModuleStateLoop(&mut self) {
        // C copies all 32 bytes of TEXT_INITIALIZATION_DATA into the message-state struct at
        // TEXT_MSGBOX_TOPLEFT_COPY (0x1cd0..0x1cf0). init_msgbox_state_from only models a
        // subset, leaving unmodeled bytes (notably DIALOGUE_MSG_SRC_OFFS 0x1cdd-0x1cde, a dead
        // message-DMA-pointer scratch) stale. Mirror C's raw copy so those bytes match too;
        // the native fields re-project their (identical) values afterward.
        self.ram[crate::game_state::constants::TEXT_MSGBOX_TOPLEFT_COPY
            ..crate::game_state::constants::TEXT_MSGBOX_TOPLEFT_COPY + TEXT_INITIALIZATION_DATA.len()]
            .copy_from_slice(&TEXT_INITIALIZATION_DATA);
        self.messaging_state_mut()
            .init_msgbox_state_from(&TEXT_INITIALIZATION_DATA);
        self.Text_InitVwfState();
        self.RenderText_SetDefaultWindowPosition();
        self.messaging_state_mut().set_text_tilemap_cur(0x3980);
        self.Text_LoadCharacterBuffer();
        self.clear_messaging_render_buffer_range(0x7e0);
        self.set_pending_nmi_subroutine(2);
        self.set_core_update_disable_flag(2);
    }

    pub(super) fn Text_InitVwfState(&mut self) {
        self.set_vwf_current_line(0);
        self.clear_vwf_next_line_request();
        self.clear_vwf_glyph_cursor();
        self.set_vwf_line_render_offset(0);
    }

    pub(super) fn Text_DecodeCmd(&self, a: u8, src: &[u8]) -> u32 {
        let (param, cmd, multibyte) = self.text_decode_cmd(a, src.first().copied());
        ((param as u32) << 6) | ((cmd as u32) << 1) | u32::from(multibyte)
    }

    fn text_decode_cmd(&self, a: u8, next: Option<u8>) -> (u8, u8, bool) {
        if self.dialogue_flags & 1 == 0 {
            if a < TEXT_COMMAND_START_US || a >= 0x80 {
                return (if a >= 0x80 { 26 } else { a }, TEXT_CMD_IS_LETTER, false);
            }
            const COMMAND_LENGTHS_US: [u8; 25] = [
                0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0,
            ];
            let cmd = a - TEXT_COMMAND_START_US;
            if COMMAND_LENGTHS_US[cmd as usize] != 0 {
                (next.unwrap_or(0), cmd, true)
            } else {
                (0, cmd, false)
            }
        } else if a < 0x7f {
            (a, TEXT_CMD_IS_LETTER, false)
        } else if a < 0x87 {
            const SIMPLE: [(u8, u8, bool); 8] = [
                (0, TEXT_CMD_END_MESSAGE, false),
                (0, TEXT_CMD_SCROLL, false),
                (0, TEXT_CMD_WAITKEY, false),
                (0, TEXT_CMD_1, false),
                (0, TEXT_CMD_2, false),
                (0, TEXT_CMD_3, false),
                (0, TEXT_CMD_NAME, false),
                (0, TEXT_CMD_NAME, false),
            ];
            SIMPLE[(a - 0x7f) as usize]
        } else {
            match next.unwrap_or(0) {
                b @ 0x00..=0x0f => (b & 0x0f, TEXT_CMD_WAIT, true),
                b @ 0x10..=0x1f => (b & 0x0f, TEXT_CMD_COLOR, true),
                b @ 0x20..=0x2f => (b & 0x0f, TEXT_CMD_NUMBER, true),
                b @ 0x30..=0x3f => (b & 0x0f, TEXT_CMD_SPEED, true),
                b @ 0x40..=0x4f => {
                    const SOUND_LUT: [u8; 1] = [45];
                    (SOUND_LUT[(b & 0x0f) as usize], TEXT_CMD_SOUND, true)
                }
                0x80 => (0, TEXT_CMD_CHOOSE, true),
                0x81 => (0, TEXT_CMD_CHOOSE2, true),
                0x82 => (0, TEXT_CMD_CHOOSE3, true),
                0x83 => (0, TEXT_CMD_SELCHG, true),
                0x84 => (0, TEXT_CMD_ITEM, true),
                0x85 => (0, TEXT_CMD_NEXT_PIC, true),
                0x86 => (2, TEXT_CMD_WINDOW, true),
                0x87 => (0, TEXT_CMD_POSITION, true),
                0x88 => (1, TEXT_CMD_POSITION, true),
                _ => (26, TEXT_CMD_IS_LETTER, false),
            }
        }
    }

    pub(super) fn Text_LoadCharacterBuffer(&mut self) {
        let Some(dialogue_blk) = self.asset_memblk(94, self.dialogue_blk_index) else {
            return;
        };
        let dictionary = find_index_in_memblk(dialogue_blk, 0).ptr.to_vec();
        let dialogue = find_index_in_memblk(dialogue_blk, 1).ptr.to_vec();
        let text_index = self.game_state.messaging.dialogue_message_index.value() as usize;
        let text_str = find_index_in_memblk(MemBlk { ptr: &dialogue }, text_index)
            .ptr
            .to_vec();

        let mut src = 0usize;
        let mut decoded = Vec::new();
        while src < text_str.len() {
            let c = text_str[src];
            src += 1;
            if c >= TEXT_DICT_BASE {
                let blk = find_index_in_memblk(
                    MemBlk { ptr: &dictionary },
                    (c - TEXT_DICT_BASE) as usize,
                );
                decoded.extend_from_slice(blk.ptr);
                continue;
            }
            let (param, cmd, multibyte) = self.text_decode_cmd(c, text_str.get(src).copied());
            match cmd {
                TEXT_CMD_NAME => self.text_write_player_name_vec(&mut decoded),
                TEXT_CMD_WINDOW => self.messaging_state_mut().set_text_render_state(param),
                TEXT_CMD_NUMBER => {
                    let v = self
                        .game_state
                        .messaging
                        .dialogue_number
                        .packed_digits((param >> 1) as usize);
                    decoded.push(0x34 + if param & 1 != 0 { v >> 4 } else { v & 0x0f });
                }
                TEXT_CMD_POSITION => {
                    self.messaging_state_mut()
                        .set_text_msgbox_topleft(TEXT_POSITIONS[param as usize & 1]);
                }
                TEXT_CMD_COLOR => {
                    let value = ((0x387f & 0xe300) | 0x180) | (((param as u16) << 10) & 0x3c00);
                    self.messaging_state_mut().set_text_tilemap_cur(value);
                }
                _ => {
                    decoded.push(c);
                    if multibyte {
                        if let Some(next) = text_str.get(src) {
                            decoded.push(*next);
                        }
                    }
                }
            }
            if multibyte {
                src += 1;
            }
        }
        decoded.push(0x7f);
        self.messaging_text_mut().load_decoded_dialogue(&decoded);
        self.messaging_state_mut().clear_dialogue_msg_read_pos();
    }

    pub(super) fn Text_WritePlayerName(&mut self, dst: usize) -> usize {
        let mut decoded = Vec::new();
        self.text_write_player_name_vec(&mut decoded);
        let len = self
            .messaging_text_mut()
            .write_decoded_text_at(dst, &decoded);
        dst + len
    }

    fn text_write_player_name_vec(&self, decoded: &mut Vec<u8>) {
        let slot = self.selected_save_slot_byte();
        let offs = (((slot >> 1) as isize) - 1) * 0x500;
        let start = 0x3d9isize + offs;
        let mut name = [0u8; 6];
        for (i, ch) in name.iter_mut().enumerate() {
            let p = start + (i as isize) * 2;
            let a = if p >= 0 && (p as usize) + 1 < self.sram.len() {
                read_le_u16(&self.sram, p as usize)
            } else {
                0
            };
            *ch = self.Text_FilterPlayerNameCharacters((a & 0x0f | (a >> 1) & 0xf0) as u8);
        }
        let mut len = name.len();
        while len != 0 && name[len - 1] == 0x59 {
            len -= 1;
        }
        decoded.extend_from_slice(&name[..len]);
    }

    pub(super) fn Text_FilterPlayerNameCharacters(&self, mut a: u8) -> u8 {
        if a >= 0x5f {
            if a >= 0x76 {
                a = a.wrapping_sub(0x42);
            } else if a == 0x5f {
                a = 8;
            } else if a == 0x60 {
                a = 0x22;
            } else if a == 0x61 {
                a = 0x3e;
            }
        }
        a
    }

    pub(super) fn Text_Render(&mut self) {
        match self.game_state.messaging.runtime.text_render_state() {
            0 => self.RenderText_Draw_Border(),
            1 => self.RenderText_Draw_BorderIncremental(),
            2 => self.RenderText_Draw_CharacterTilemap(),
            3 => self.RenderText_Draw_MessageCharacters(),
            4 => self.RenderText_Draw_Finish(),
            _ => {}
        }
    }

    pub(super) fn RenderText_Draw_Border(&mut self) {
        self.RenderText_DrawBorderInitialize();
        let mut d = self.RenderText_DrawBorderRow(0x1002, 0);
        for _ in 0..6 {
            d = self.RenderText_DrawBorderRow(d, 6);
        }
        self.RenderText_DrawBorderRow(d, 12);
        self.set_bg_vram_load_mode(1);
        self.messaging_state_mut().set_text_render_state(2);
    }

    pub(super) fn RenderText_Draw_BorderIncremental(&mut self) {
        self.set_bg_vram_load_mode(1);
        let mut a = self.game_state.messaging.runtime.text_incremental_state();
        let d = 0x1002;
        if a != 0 {
            a = if a < 7 { 1 } else { 2 };
        }
        match a {
            0 => {
                self.RenderText_DrawBorderInitialize();
                self.RenderText_DrawBorderRow(d, 0);
                self.messaging_state_mut()
                    .increment_text_incremental_state();
            }
            1 => {
                self.RenderText_DrawBorderRow(d, 6);
                self.messaging_state_mut()
                    .increment_text_incremental_state();
            }
            2 => {
                self.messaging_state_mut().set_text_render_state(2);
                self.RenderText_DrawBorderRow(d, 12);
                self.messaging_state_mut()
                    .increment_text_incremental_state();
            }
            _ => {}
        }
    }

    pub(super) fn RenderText_Draw_CharacterTilemap(&mut self) {
        self.Text_BuildCharacterTilemap();
    }

    pub(super) fn RenderText_Draw_MessageCharacters(&mut self) {
        loop {
            let read_pos = self.game_state.messaging.runtime.dialogue_msg_read_pos() as usize;
            let c = self.game_state.messaging.decoded_text.byte(read_pos);
            let (param, cmd, multibyte) = self.text_decode_cmd(
                c,
                self.game_state.messaging.decoded_text.next_byte(read_pos),
            );
            let mut command_done = false;
            let mut restart_if_zero_speed = false;
            match cmd {
                TEXT_CMD_IS_LETTER => {
                    if self.game_state.messaging.runtime.vwf_line_speed_cur() >= 2 {
                        self.messaging_state_mut().decrement_vwf_line_speed_cur();
                    } else {
                        self.VWF_RenderSingle(param as i32);
                        command_done = true;
                        restart_if_zero_speed =
                            self.game_state.messaging.runtime.vwf_line_speed_cur() == 0;
                    }
                }
                TEXT_CMD_NEXT_PIC => {
                    if self.game_state.frame.main_module == 20 {
                        self.PaletteFilterHistory();
                        command_done = self.game_state.display.palette_filter.countdown() == 0;
                    } else {
                        command_done = true;
                    }
                }
                TEXT_CMD_SCROLL_SPD => {
                    self.messaging_state_mut().set_dialogue_scroll_speed(param);
                    command_done = true;
                }
                TEXT_CMD_SCROLL => command_done = self.RenderText_Draw_Scroll(),
                TEXT_CMD_1 | TEXT_CMD_2 | TEXT_CMD_3 => {
                    let idx = (cmd - TEXT_CMD_1) as usize;
                    self.set_vwf_current_line(VWF_ROW_POSITIONS[idx]);
                    self.request_vwf_next_line(1);
                    command_done = true;
                }
                TEXT_CMD_WAIT => {
                    let wait = if self.game_state.player.follower_link.joypad1l_last() & 0x80 != 0 {
                        1
                    } else {
                        self.game_state.messaging.runtime.text_wait_countdown()
                    };
                    match wait {
                        0 => self.messaging_state_mut().set_text_wait_countdown(
                            TEXT_WAIT_DURATIONS[param as usize].wrapping_sub(1),
                        ),
                        1 => {
                            self.messaging_state_mut().clear_text_wait_countdown();
                            command_done = true;
                        }
                        _ => self
                            .messaging_state_mut()
                            .set_text_wait_countdown(wait.wrapping_sub(1)),
                    }
                }
                TEXT_CMD_SOUND => {
                    self.set_sound_effect_2(param);
                    command_done = true;
                }
                TEXT_CMD_SPEED => {
                    self.messaging_state_mut().set_vwf_line_speed(param);
                    self.messaging_state_mut().set_vwf_line_speed_cur(param);
                    command_done = true;
                }
                TEXT_CMD_CHOOSE => self.RenderText_Draw_Choose2LowOr3(),
                TEXT_CMD_ITEM => self.RenderText_Draw_ChooseItem(),
                TEXT_CMD_SELCHG => self.RenderText_Draw_Choose2HiOr3(),
                TEXT_CMD_CHOOSE3 => self.RenderText_Draw_Choose3(),
                TEXT_CMD_CHOOSE2 => self.RenderText_Draw_Choose1Or2(),
                TEXT_CMD_WAITKEY | TEXT_CMD_END_MESSAGE => {
                    if cmd == TEXT_CMD_END_MESSAGE
                        && std::env::var_os("ZELDA3_SMV_MESSAGING_TIMING_HACKS").is_some()
                        && self.game_state.messaging.dialogue_message_index.value() == 0x0070
                        && self.game_state.messaging.runtime.dialogue_msg_read_pos() == 0x0115
                        && self.game_state.messaging.runtime.text_wait_countdown2() <= 3
                    {
                        self.messaging_state_mut().clear_text_wait_countdown2();
                    }
                    if self.game_state.messaging.runtime.text_wait_countdown2() != 0 {
                        self.messaging_state_mut().decrement_text_wait_countdown2();
                        if self.game_state.messaging.runtime.text_wait_countdown2() == 1 {
                            self.set_sound_effect_2(36);
                        }
                    } else if (self.game_state.player.follower_link.filtered_joypad_h()
                        | self.game_state.player.follower_link.filtered_joypad_l())
                        & if cmd == TEXT_CMD_WAITKEY { 0xc0 } else { 0xff }
                        != 0
                    {
                        self.messaging_state_mut().set_text_wait_countdown2(28);
                        command_done = cmd == TEXT_CMD_WAITKEY;
                        if cmd == TEXT_CMD_END_MESSAGE {
                            self.messaging_state_mut().set_text_render_state(4);
                        }
                    }
                }
                _ => {
                    panic!("RenderText_Draw_MessageCharacters unsupported cmd {cmd} param {param}")
                }
            }
            if command_done {
                self.messaging_state_mut().set_dialogue_msg_read_pos(
                    (read_pos as u16).wrapping_add(1 + u16::from(multibyte)),
                );
            }
            if !restart_if_zero_speed {
                break;
            }
        }
        self.set_pending_nmi_subroutine(2);
        self.set_core_update_disable_flag(2);
    }

    pub(super) fn RenderText_Draw_Finish(&mut self) {
        self.RenderText_DrawBorderInitialize();
        let top_left = self.game_state.messaging.runtime.text_msgbox_topleft_copy();
        self.write_vram_upload_buffer_word(0, top_left.swap_bytes());
        self.write_vram_upload_buffer_word(2, 0x2e42);
        self.write_vram_upload_buffer_word(4, 0x387f);
        self.write_vram_upload_buffer_word(6, 0xffff);
        self.set_bg_vram_load_mode(1);
        self.messaging_state_mut().clear_module();
        self.set_submodule(0);
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
    }

    pub(super) fn VWF_RenderSingle(&mut self, c: i32) {
        let c = c as u8;
        if c != 0x59 {
            self.set_sound_effect_2(12);
        }
        let speed = self.game_state.messaging.runtime.vwf_line_speed();
        self.messaging_state_mut().set_vwf_line_speed_cur(speed);
        if self.game_state.messaging.vwf_render.next_line_requested() != 0 {
            let line = (self.game_state.messaging.vwf_render.current_line() >> 1) as usize;
            self.set_vwf_line_render_offset(VWF_RENDER_CHARACTER_RENDER_POS[line]);
            self.set_vwf_glyph_cursor(VWF_RENDER_CHARACTER_LINE_POSITIONS[line]);
            self.clear_vwf_next_line_request();
        }
        let Some(dialogue_font) = self.asset_memblk(95, self.dialogue_font_blk_index) else {
            return;
        };
        let font_data = find_index_in_memblk(dialogue_font, 0).ptr.to_vec();
        let widths = find_index_in_memblk(dialogue_font, 1).ptr.to_vec();
        let width = widths.get(c as usize).copied().unwrap_or(0);
        assert!(width <= 8);
        let i = self.game_state.messaging.vwf_render.glyph_cursor_usize();
        self.increment_vwf_glyph_cursor();
        // C: arrval = vwf_arr[i]; vwf_arr[i + 1] = arrval + width (vwf_arr = raw g_ram).
        let arrval = self.vwf_glyph_advance_prefix_sum(i);
        self.set_vwf_next_glyph_advance_prefix_sum(i, arrval.wrapping_add(width));
        let r10 = ((c as usize & 0x70) * 2) + (c as usize & 0x0f);
        let r0 = arrval as usize * 2;
        let line_ptr = self.game_state.messaging.vwf_render.line_render_offset() as usize;
        self.messaging_vwf_render_half(&font_data, r10, r0, line_ptr, width);
        self.messaging_vwf_render_half(&font_data, r10 + 16, r0, line_ptr + 0x150, width);
    }

    fn messaging_vwf_render_half(
        &mut self,
        font_data: &[u8],
        r10: usize,
        r0: usize,
        line_ptr: usize,
        width: u8,
    ) {
        let mut src = r10 * 16;
        for i in (0..16).step_by(2) {
            if src + 1 >= font_data.len() {
                return;
            }
            let mut r4 = u16::from_le_bytes([font_data[src], font_data[src + 1]]);
            src += 2;
            let y_base = r0 + line_ptr;
            let mut x = (y_base & 0xff0) + i;
            let mut y = (y_base >> 1) & 7;
            let mut r3 = width;
            while r3 != 0 {
                if r4 & 0x0080 != 0 {
                    self.xor_messaging_render_buffer_mask(x, VWF_RENDER_CHARACTER_SET_MASKS[y]);
                } else {
                    self.clear_messaging_render_buffer_mask(x, VWF_RENDER_CHARACTER_SET_MASKS[y]);
                }
                if r4 & 0x8000 != 0 {
                    self.xor_messaging_render_buffer_mask(x + 1, VWF_RENDER_CHARACTER_SET_MASKS[y]);
                } else {
                    self.clear_messaging_render_buffer_mask(
                        x + 1,
                        VWF_RENDER_CHARACTER_SET_MASKS[y],
                    );
                }
                r4 = (r4 & !0x8080) << 1;
                r3 -= 1;
                y += 1;
                if y == 8 {
                    break;
                }
            }
            x += 16;
            if r4 != 0 {
                self.set_messaging_render_buffer_word_at_byte_offset(x, r4);
            }
        }
    }

    pub(super) fn RenderText_Draw_Choose2LowOr3(&mut self) {
        self.RenderText_Draw_Choose2(1);
    }

    pub(super) fn RenderText_Draw_ChooseItem(&mut self) {
        if self.game_state.messaging.runtime.text_wait_countdown2() != 0 {
            self.messaging_state_mut().decrement_text_wait_countdown2();
            if self.game_state.messaging.runtime.text_wait_countdown2() == 1 {
                self.RenderText_FindYItem_Next();
            }
        } else if (self.game_state.player.follower_link.filtered_joypad_h()
            | self.game_state.player.follower_link.filtered_joypad_l())
            & 0xc0
            != 0
        {
            self.messaging_state_mut().set_text_render_state(4);
        } else {
            if self.game_state.player.follower_link.filtered_joypad_h() & 5 != 0 {
                self.multiselect_choice_mut().increment_value();
            } else if self.game_state.player.follower_link.filtered_joypad_h() & 10 != 0 {
                self.multiselect_choice_mut().decrement_value();
                self.RenderText_FindYItem_Previous();
                self.RenderText_Refresh();
                return;
            }
            self.RenderText_FindYItem_Next();
            self.RenderText_Refresh();
        }
    }

    pub(super) fn RenderText_FindYItem_Previous(&mut self) {
        loop {
            let mut x = self.multiselect_choice().value();
            if (x as i8) < 0 {
                self.multiselect_choice_mut().set_value(31);
                x = 31;
            }
            // Raw RAM (not bounded inventory_item) — same fix as RenderText_FindYItem_Next.
            if x != 15
                && (self.ram[LINK_ITEM_BOW + x as usize] != 0
                    || (x == 32 && self.ram[LINK_ITEM_BOW + x as usize + 1] != 0))
            {
                break;
            }
            self.multiselect_choice_mut().decrement_value();
        }
        self.RenderText_DrawSelectedYItem();
    }

    pub(super) fn RenderText_FindYItem_Next(&mut self) {
        loop {
            let mut x = self.multiselect_choice().value();
            if x >= 32 {
                self.multiselect_choice_mut().set_value(0);
                x = 0;
            }
            // C reads raw inventory bytes ram[LINK_ITEM_BOW + x] for x up to 32. The native
            // item_slots model only covers 28 slots, so inventory_item(x) returns 0 for x>=28 —
            // which made this scan skip owned items 28-31 and wrap to 0 (NEW landed on item 0, OLD
            // on item 28), cascading the menu/text-render divergence. Read raw RAM to match C.
            if x != 15
                && (self.ram[LINK_ITEM_BOW + x as usize] != 0
                    || (x == 32 && self.ram[LINK_ITEM_BOW + x as usize + 1] != 0))
            {
                break;
            }
            self.multiselect_choice_mut().increment_value();
        }
        self.RenderText_DrawSelectedYItem();
    }

    pub(super) fn RenderText_DrawSelectedYItem(&mut self) {
        let item = self.multiselect_choice().value();
        // Raw RAM (not inventory_item, which is bounded to 28 slots) — matches C and covers the
        // multiselect index range 0..=32 (see RenderText_FindYItem_Next).
        let variant = if item == 3 || item == 32 {
            1
        } else {
            self.ram[LINK_ITEM_BOW + item as usize] as usize
        };
        let p = self.hud_get_item_box_table(item)[variant];
        self.set_vwf_tile_word_at_byte_offset(0x0c2, p[0]);
        self.set_vwf_tile_word_at_byte_offset(0x0c4, p[1]);
        self.set_vwf_tile_word_at_byte_offset(0x0ec, p[2]);
        self.set_vwf_tile_word_at_byte_offset(0x0ee, p[3]);
    }

    pub(super) fn RenderText_Draw_Choose2HiOr3(&mut self) {
        self.RenderText_Draw_Choose2(11);
    }

    fn RenderText_Draw_Choose2(&mut self, message_base: u16) {
        if self.game_state.messaging.runtime.text_wait_countdown2() != 0 {
            self.messaging_state_mut().decrement_text_wait_countdown2();
            if self.game_state.messaging.runtime.text_wait_countdown2() == 1 {
                self.set_sound_effect_2(36);
            }
        } else if (self.game_state.player.follower_link.filtered_joypad_h()
            | self.game_state.player.follower_link.filtered_joypad_l())
            & 0xc0
            != 0
        {
            self.set_sound_effect_1(43);
            self.messaging_state_mut().set_text_render_state(4);
        } else if self.game_state.player.follower_link.filtered_joypad_h() & 12 != 0 {
            let t = if self.game_state.player.follower_link.filtered_joypad_h() & 8 != 0 {
                0
            } else {
                1
            };
            if self.multiselect_choice().value() == t {
                return;
            }
            self.multiselect_choice_mut().set_value(t);
            self.set_sound_effect_2(32);
            self.dialogue_message_index_mut()
                .set_value(message_base + u16::from(t));
            self.Text_LoadCharacterBuffer();
            self.Text_InitVwfState();
        }
    }

    pub(super) fn RenderText_Draw_Choose3(&mut self) {
        if self.game_state.messaging.runtime.text_wait_countdown2() != 0 {
            self.messaging_state_mut().decrement_text_wait_countdown2();
            if self.game_state.messaging.runtime.text_wait_countdown2() == 1 {
                self.set_sound_effect_2(36);
            }
            return;
        }
        let y = (self.game_state.player.follower_link.filtered_joypad_l() & 0xc0)
            | self.game_state.player.follower_link.filtered_joypad_h();
        if y & 0xd0 != 0 {
            self.set_sound_effect_1(43);
            self.messaging_state_mut().set_text_render_state(4);
        } else if y & 12 != 0 {
            let mut choice = self.multiselect_choice().value();
            choice = if y & 8 != 0 {
                if choice == 0 {
                    2
                } else {
                    choice - 1
                }
            } else if choice == 2 {
                0
            } else {
                choice + 1
            };
            self.multiselect_choice_mut().set_value(choice);
            self.set_sound_effect_2(32);
            self.dialogue_message_index_mut()
                .set_value(u16::from(choice) + 6);
            self.Text_LoadCharacterBuffer();
            self.Text_InitVwfState();
        }
    }

    pub(super) fn RenderText_Draw_Choose1Or2(&mut self) {
        if self.game_state.messaging.runtime.text_wait_countdown2() != 0 {
            self.messaging_state_mut().decrement_text_wait_countdown2();
            if self.game_state.messaging.runtime.text_wait_countdown2() == 1 {
                self.set_sound_effect_2(36);
            }
            return;
        }
        let y = (self.game_state.player.follower_link.filtered_joypad_l() & 0xc0)
            | self.game_state.player.follower_link.filtered_joypad_h();
        if y & 0xd0 != 0 {
            self.set_sound_effect_1(43);
            self.messaging_state_mut().set_text_render_state(4);
        } else if y & 12 != 0 {
            let t = if y & 8 != 0 { 0 } else { 1 };
            if self.multiselect_choice().value() == t {
                return;
            }
            self.multiselect_choice_mut().set_value(t);
            self.set_sound_effect_2(32);
            self.dialogue_message_index_mut()
                .set_value(u16::from(t) + 9);
            self.Text_LoadCharacterBuffer();
            self.Text_InitVwfState();
        }
    }

    pub(super) fn RenderText_Draw_Scroll(&mut self) -> bool {
        let mut r2 = self.game_state.messaging.runtime.dialogue_scroll_speed();
        loop {
            for i in (0..0x7e0).step_by(16) {
                for j in 0..7 {
                    let value = self
                        .game_state
                        .messaging
                        .render_buffer
                        .word_at_byte_offset(i + (j + 1) * 2);
                    self.set_messaging_render_buffer_word_at_byte_offset(i + j * 2, value);
                }
                let value = self
                    .game_state
                    .messaging
                    .render_buffer
                    .word_at_byte_offset(i + 168 * 2);
                self.set_messaging_render_buffer_word_at_byte_offset(i + 7 * 2, value);
            }
            for i in (0x34f..=0x3ef).step_by(8) {
                self.set_messaging_render_buffer_word_at_byte_offset(i * 2, 0);
            }
            let source_bank_offset = self
                .dialogue_source_offset_mut()
                .increment_bank_offset_low_nibble();
            if source_bank_offset & 0x0f == 0 {
                self.set_vwf_current_line(4);
                self.request_vwf_next_line(1);
                return true;
            }
            let old = r2;
            r2 = r2.wrapping_sub(1);
            if old == 0 {
                return false;
            }
        }
    }

    pub(super) fn RenderText_SetDefaultWindowPosition(&mut self) {
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let flag = usize::from(y < 0x78);
        self.messaging_state_mut()
            .set_text_msgbox_topleft(TEXT_POSITIONS[flag]);
    }

    pub(super) fn RenderText_DrawBorderInitialize(&mut self) {
        let top_left = self.game_state.messaging.runtime.text_msgbox_topleft();
        self.messaging_state_mut()
            .set_text_msgbox_topleft_copy(top_left);
    }

    pub(super) fn RenderText_DrawBorderRow(&mut self, mut d: usize, y: usize) -> usize {
        let y = y >> 1;
        let top_left = self.game_state.messaging.runtime.text_msgbox_topleft_copy();
        self.write_vram_upload_absolute_word(d, top_left.swap_bytes());
        d += 2;
        self.messaging_state_mut()
            .set_text_msgbox_topleft_copy(top_left.wrapping_add(0x20));
        self.write_vram_upload_absolute_word(d, 0x2f00);
        d += 2;
        self.write_vram_upload_absolute_word(d, TEXT_BORDER_TILES[y]);
        d += 2;
        for _ in 0..22 {
            self.write_vram_upload_absolute_word(d, TEXT_BORDER_TILES[y + 1]);
            d += 2;
        }
        self.write_vram_upload_absolute_word(d, TEXT_BORDER_TILES[y + 2]);
        d += 2;
        self.write_vram_upload_absolute_word(d, 0xffff);
        d
    }

    pub(super) fn Text_BuildCharacterTilemap(&mut self) {
        let mut tile = self.game_state.messaging.runtime.text_tilemap_cur();
        for i in 0..126 {
            self.set_vwf_tile_word_at_byte_offset(i * 2, tile);
            tile = tile.wrapping_add(1);
        }
        self.messaging_state_mut().set_text_tilemap_cur(tile);
        self.RenderText_Refresh();
        self.messaging_state_mut().increment_text_render_state();
    }

    pub(super) fn RenderText_Refresh(&mut self) {
        self.RenderText_DrawBorderInitialize();
        let top_left = self
            .game_state
            .messaging
            .runtime
            .text_msgbox_topleft_copy()
            .wrapping_add(0x21);
        self.messaging_state_mut()
            .set_text_msgbox_topleft_copy(top_left);
        let mut d = 0x1002usize;
        let mut s = 0usize; // offset into VWF_TILE_BUFFER
        for _ in 0..6 {
            let row_top_left = self.game_state.messaging.runtime.text_msgbox_topleft_copy();
            self.write_vram_upload_absolute_word(d, row_top_left.swap_bytes());
            d += 2;
            self.messaging_state_mut()
                .set_text_msgbox_topleft_copy(row_top_left.wrapping_add(0x20));
            self.write_vram_upload_absolute_word(d, 0x2900);
            d += 2;
            for _ in 0..21 {
                let tile = self
                    .game_state
                    .messaging
                    .vwf_render
                    .tile_word_at_byte_offset(s);
                self.write_vram_upload_absolute_word(d, tile);
                d += 2;
                s += 2;
            }
        }
        self.write_vram_upload_absolute_word(d, 0xffff);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn Text_GenerateMessagePointers(&mut self) {
        let Some(dialogue_blk) = self.asset_memblk(94, self.dialogue_blk_index) else {
            return;
        };
        let dialogue = find_index_in_memblk(dialogue_blk, 1).ptr.to_vec();
        let mut p = 0x1c8000u32;
        for i in 0..398 {
            if i == 359 {
                p = 0x0edf40;
            }
            self.messaging_text_mut().set_dialogue_pointer(i, p);
            let entry = find_index_in_memblk(MemBlk { ptr: &dialogue }, i);
            p = p.wrapping_add(entry.ptr.len() as u32 + 1);
        }
    }

    pub(super) fn Death_PlayerSwoon(&mut self) {
        let mut k = self.game_state.player.follower_link.item_action_step_var() as usize;
        self.follower_link_state_mut()
            .decrement_y_button_action_timer();
        if (self.game_state.player.follower_link.y_button_action_timer() as i8) < 0 {
            k += 1;
            if k == 15 {
                return;
            }
            if k == 14 {
                self.increment_submodule();
            }
            self.follower_link_state_mut()
                .set_item_action_step_var(k as u8);
            self.follower_link_state_mut()
                .set_y_button_action_step(DEATH_ANIM_CTR0[k]);
            self.follower_link_state_mut()
                .set_y_button_action_timer(DEATH_ANIM_CTR1[k]);
        }
        if k != 13 || self.game_state.player.follower_link.visibility_status() == 12 {
            return;
        }
        let y = (self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(16)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())) as u8;
        let x = (self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(7)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())) as u8;
        let flags = DEATH_SPR_FLAGS
            [self.game_state.player.follower_link.lower_level_state() as usize & 1]
            | 2;
        self.set_oam_plain(0x74, x, y, 0xaa, flags, 2);
    }

    pub(super) fn Death_PrepFaint(&mut self) {
        self.follower_link_state_mut().set_facing(2);
        self.follower_link_state_mut().set_faint_animation_active(1);
        self.follower_link_state_mut().clear_item_action_step_var();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_timer(5);
        {
            let mut resources = self.player_resources_mut();
            resources.set_heart_filler(0);
            resources.set_current_health(0);
        }
        self.link_reset_properties_c();
        self.follower_link_state_mut()
            .clear_somaria_platform_state();
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        self.follower_link_state_mut().clear_bunny_mirror();
        self.follower_link_state_mut().clear_defense_flags();
        self.follower_link_state_mut().clear_ancilla_pickup_flag();
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().clear_transforming();
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut()
            .clear_transform_poof_need_and_temp_bunny_timer();
        if self.game_state.inventory.items.has_moon_pearl() {
            self.follower_link_state_mut().clear_bunny_body_state();
        }
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self
            .game_state
            .enhanced_features
            .has(FEATURES0_MISC_BUG_FIXES)
        {
            self.LoadActualGearPalettes();
        }
        let sfx = 0x27 | self.link_calculate_sfx_pan();
        self.set_sound_effect_1(sfx);
        for i in 0..4 {
            if self.game_state.inventory.items.bottle(i) == 6 {
                return;
            }
        }
        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(0);
        self.dungeon_object_tracking_mut()
            .clear_changeable_object_index(1);
    }

    pub(super) fn DisplaySelectMenu(&mut self) {
        self.multiselect_choice_mut().save_backup();
        self.dialogue_message_index_mut().set_value(0x0186);
        let bak = self.game_state.frame.main_module;
        self.main_show_text_message();
        self.set_main_module(bak);
        self.set_subsubmodule(0);
        self.set_submodule(11);
        self.save_main_module_for_menu();
        self.set_main_module(14);
    }
}

impl ZeldaState {
    pub(super) fn world_map_load_light_world_map(&mut self) {
        self.world_map_fill_tilemap_with_ef();
        self.set_main_screen_layers(0x11);
        self.set_sub_screen_layers(0);
        self.transfer_mode7_characters();
        self.world_map_setup_hdma();
        self.load_overworld_map_palette();
        self.load_actual_gear_palettes();
        self.increment_cgram_update_flag();
        self.set_pending_nmi_subroutine(7);
        self.set_screen_brightness(0);
        self.increment_core_update_disable_flag();
        self.increment_overworld_map_state();
    }

    pub(super) fn world_map_fill_tilemap_with_ef(&mut self) {
        for i in 0..0x4000 {
            self.ppu.vram[i] = (self.ppu.vram[i] & 0xff00) | 0x00ef;
        }
    }

    pub(super) fn transfer_mode7_characters(&mut self) {
        if let Some(gfx) = self.asset_raw(66).map(Vec::from) {
            for i in 0..0x4000.min(gfx.len()).min(self.ppu.vram.len()) {
                self.ppu.vram[i] = (self.ppu.vram[i] & 0x00ff) | ((gfx[i] as u16) << 8);
            }
        }
    }

    pub(super) fn did_press_button_for_map(&self) -> bool {
        if self.game_state.world.transient.hud_cur_item_x() != 0 {
            self.game_state.player.follower_link.filtered_joypad_h() & 0x20 != 0
        } else {
            self.game_state.player.follower_link.filtered_joypad_l() & 0x40 != 0
        }
    }
}
