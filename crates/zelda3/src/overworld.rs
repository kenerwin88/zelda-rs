// Methods ported from zelda3/src/overworld.c and included inside ZeldaState.

use super::*;
use crate::types::{sign16, Point16U};

const DOOR_ANIMATION_STEP_INDICATOR_OVERWORLD: usize = 0x0690;
const OVERWORLD_TRANSITION_DIR_ENUM: usize = 0x069c;
const MEMORIZED_TILE_ADDR_OVERWORLD: usize = 0x0f800;
const MEMORIZED_TILE_VALUE_OVERWORLD: usize = 0x0fa00;
const OVERWORLD_PEG_PUZZLE_PROGRESS: usize = 0x04c8;
const BIG_KEY_DOOR_MESSAGE_TRIGGERED_OVERWORLD: usize = 0x04b8;
const TRIGGER_SPECIAL_ENTRANCE_OVERWORLD: usize = 0x04c6;
const OVERWORLD_BOMB_TILE_SWEEP_X: usize = 0x0486;
const OVERWORLD_BOMB_TILE_SWEEP_Y_END: usize = 0x0488;
const MAPBAK_PALETTE_OVERWORLD: usize = 0x1dd80;
const MAP16_LOAD_SRC_OFF_OVERWORLD: usize = 0x0084;
const MAP16_LOAD_DST_OFF_OVERWORLD: usize = 0x0086;
// NES_Ver2: YWRITE, vertical unit position used while emitting Map16 stripes.
const MAP16_LOAD_Y_UNIT_OVERWORLD: usize = 0x0088;
const WORD_7F4000_OVERWORLD: usize = 0x14000;
const UVRAM_DATA_OVERWORLD: usize = 0x1100;
const OVERWORLD_MAP16_DECODE_SRC: usize = 0x14000;
const OVERWORLD_DECOMP_BUFFER: usize = 0x14400;
const MAP16_DECODE_0_OVERWORLD: usize = 0x14400;
const MAP16_DECODE_1_OVERWORLD: usize = 0x14410;
const MAP16_DECODE_2_OVERWORLD: usize = 0x14420;
const MAP16_DECODE_3_OVERWORLD: usize = 0x14430;
const MAP16_DECODE_LAST_OVERWORLD: usize = 0x14440;
const MAP16_DECODE_WORK_WORD_OVERWORLD: usize = 0x14442;
const DUNG_REPLACEMENT_TILE_STATE_OVERWORLD: usize = 0x0500;
const ORANGE_BLUE_BARRIER_STATE_OVERWORLD: usize = 0x0c172;
const SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF: usize = 0x0c174;
const SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT: usize = 0x0c176;
const OVERWORLD_AREA_INDEX_OVERWORLD: usize = 0x040a;
const INCREMENTAL_COUNTER_FOR_VRAM_OVERWORLD: usize = 0x0412;
const OVERWORLD_SCREEN_TRANS_DIR_BITS_OVERWORLD: usize = 0x0410;
const OVERWORLD_SCROLL_UP_COUNTER_OVERWORLD: usize = 0x0624;
const OVERWORLD_SCROLL_DOWN_COUNTER_OVERWORLD: usize = 0x0626;
const OVERWORLD_SCROLL_LEFT_COUNTER_OVERWORLD: usize = 0x0628;
const OVERWORLD_SCROLL_RIGHT_COUNTER_OVERWORLD: usize = 0x062a;
const OW_COUNTDOWN_TRANSITION_OVERWORLD: usize = 0x069a;
const OVERWORLD_OFFSET_BASE_Y_OVERWORLD: usize = 0x0708;
const OVERWORLD_OFFSET_MASK_Y_OVERWORLD: usize = 0x070a;
const OVERWORLD_OFFSET_BASE_X_OVERWORLD: usize = 0x070c;
const OVERWORLD_OFFSET_MASK_X_OVERWORLD: usize = 0x070e;
const OVERWORLD_AREA_INDEX_SPEXIT_OVERWORLD: usize = 0x0c100;
const TM_COPY_SPEXIT_OVERWORLD: usize = 0x0c102;
const OVERWORLD_SCREEN_INDEX_SPEXIT_OVERWORLD: usize = 0x0c10c;
const MAP16_LOAD_SRC_OFF_SPEXIT_OVERWORLD: usize = 0x0c10e;
const CAMERA_Y_COORD_SCROLL_LOW_SPEXIT_OVERWORLD: usize = 0x0c110;
const CAMERA_X_COORD_SCROLL_LOW_SPEXIT_OVERWORLD: usize = 0x0c112;
const SPECIAL_EXIT_ROOM_BOUNDS_Y_START: usize = 0x0c114;
const SPECIAL_EXIT_ROOM_BOUNDS_Y_END: usize = 0x0c116;
const SPECIAL_EXIT_ROOM_BOUNDS_X_START: usize = 0x0c118;
const SPECIAL_EXIT_ROOM_BOUNDS_X_END: usize = 0x0c11a;
const UP_DOWN_SCROLL_TARGET_SPEXIT_OVERWORLD: usize = 0x0c11c;
const UP_DOWN_SCROLL_TARGET_END_SPEXIT_OVERWORLD: usize = 0x0c11e;
const LEFT_RIGHT_SCROLL_TARGET_SPEXIT_OVERWORLD: usize = 0x0c120;
const LEFT_RIGHT_SCROLL_TARGET_END_SPEXIT_OVERWORLD: usize = 0x0c122;
const OVERWORLD_SPECIAL_TILE_THEME_INDEX: usize = 0x0c124;
const MAIN_TILE_THEME_INDEX_SPEXIT_OVERWORLD: usize = 0x0c125;
const AUX_TILE_THEME_INDEX_SPEXIT_OVERWORLD: usize = 0x0c126;
const SPRITE_GRAPHICS_INDEX_SPEXIT_OVERWORLD: usize = 0x0c127;
const OVERWORLD_SCROLL_UP_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c12a;
const OVERWORLD_SCROLL_DOWN_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c12c;
const OVERWORLD_SCROLL_LEFT_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c12e;
const OVERWORLD_SCROLL_RIGHT_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c130;
const OVERWORLD_AREA_INDEX_EXIT_OVERWORLD: usize = 0x0c140;
const TM_COPY_EXIT_OVERWORLD: usize = 0x0c142;
const OVERWORLD_SCREEN_INDEX_EXIT_OVERWORLD: usize = 0x0c14c;
const MAP16_LOAD_SRC_OFF_EXIT_OVERWORLD: usize = 0x0c14e;
const CAMERA_Y_COORD_SCROLL_LOW_EXIT_OVERWORLD: usize = 0x0c150;
const CAMERA_X_COORD_SCROLL_LOW_EXIT_OVERWORLD: usize = 0x0c152;
const OW_SCROLL_VARS0_EXIT_OVERWORLD: usize = 0x0c154;
const UP_DOWN_SCROLL_TARGET_EXIT_OVERWORLD: usize = 0x0c15c;
const UP_DOWN_SCROLL_TARGET_END_EXIT_OVERWORLD: usize = 0x0c15e;
const LEFT_RIGHT_SCROLL_TARGET_EXIT_OVERWORLD: usize = 0x0c160;
const LEFT_RIGHT_SCROLL_TARGET_END_EXIT_OVERWORLD: usize = 0x0c162;
const OVERWORLD_EXIT_TILE_THEME_INDEX_OVERWORLD: usize = 0x0c164;
const MAIN_TILE_THEME_INDEX_EXIT_OVERWORLD: usize = 0x0c165;
const AUX_TILE_THEME_INDEX_EXIT_OVERWORLD: usize = 0x0c166;
const SPRITE_GRAPHICS_INDEX_EXIT_OVERWORLD: usize = 0x0c167;
const OVERWORLD_SCROLL_UP_COUNTER_EXIT_OVERWORLD: usize = 0x0c16a;
const OVERWORLD_SCROLL_DOWN_COUNTER_EXIT_OVERWORLD: usize = 0x0c16c;
const OVERWORLD_SCROLL_LEFT_COUNTER_EXIT_OVERWORLD: usize = 0x0c16e;
const OVERWORLD_SCROLL_RIGHT_COUNTER_EXIT_OVERWORLD: usize = 0x0c170;
const OVERWORLD_TILE_THEME_INDEX_OVERWORLD: usize = 0x0aa0;
const MAIN_TILE_THEME_INDEX_OVERWORLD: usize = 0x0aa1;
const AUX_TILE_THEME_INDEX_OVERWORLD: usize = 0x0aa2;
const SPRITE_GRAPHICS_INDEX_OVERWORLD: usize = 0x0aa3;
const MISC_SPRITES_GRAPHICS_INDEX_OVERWORLD: usize = 0x0aa4;
const FLAG_OVERWORLD_AREA_DID_CHANGE_OVERWORLD: usize = 0x0abf;
const BIRDTRAVEL_STATUS_OVERWORLD: usize = 0x1af0;
const MOVE_OVERLAY_CTR_OVERWORLD: usize = 0x0494;
const SAVEGAME_HAS_MASTER_SWORD_FLAGS_OVERWORLD: usize = 0x0f300;
const OVERLAY_INDEX_OVERWORLD: usize = 0x008c;
const OVERWORLD_SCREEN_INDEX_PREV_OVERWORLD: usize = 0x0c213;
const MAP16_LOAD_SRC_OFF_PREV_OVERWORLD: usize = 0x0c215;
const MAP16_LOAD_Y_UNIT_PREV_OVERWORLD: usize = 0x0c217;
const MAP16_LOAD_DST_OFF_PREV_OVERWORLD: usize = 0x0c219;
const OVERWORLD_SCREEN_TRANSITION_PREV_OVERWORLD: usize = 0x0c21b;
const OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV_OVERWORLD: usize = 0x0c21d;
const OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV_OVERWORLD: usize = 0x0c21f;
const TRANSITION_COUNTER_OVERWORLD: usize = 0x0126;
const CURRENT_AREA_OF_PLAYER_OVERWORLD: usize = 0x0700;

const OVERWORLD_ENTRANCE_PRIMARY_AREA_BY_INDEX: [u16; 44] = [
    0xfe, 0xc5, 0xfe, 0x114, 0x115, 0x175, 0x156, 0xf5, 0xe2, 0x1ef, 0x119, 0xfe, 0x172, 0x177,
    0x13f, 0x172, 0x112, 0x161, 0x172, 0x14c, 0x156, 0x1ef, 0xfe, 0xfe, 0xfe, 0x10b, 0x173, 0x143,
    0x149, 0x175, 0x103, 0x100, 0x1cc, 0x15e, 0x167, 0x128, 0x131, 0x112, 0x16d, 0x163, 0x173,
    0xfe, 0x113, 0x177,
];
const OVERWORLD_ENTRANCE_SECONDARY_AREA_BY_INDEX: [u16; 44] = [
    0x14a, 0xc4, 0x14f, 0x115, 0x114, 0x174, 0x155, 0xf5, 0xee, 0x1eb, 0x118, 0x146, 0x171, 0x155,
    0x137, 0x174, 0x173, 0x121, 0x164, 0x155, 0x157, 0x128, 0x114, 0x123, 0x113, 0x109, 0x118,
    0x161, 0x149, 0x117, 0x174, 0x101, 0x1cc, 0x131, 0x51, 0x14e, 0x131, 0x112, 0x17a, 0x163,
    0x172, 0x1bd, 0x152, 0x167,
];
const OVERWORLD_AREA_BASE_X: [u16; 64] = [
    0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00,
    0, 0x200, 0x400, 0x600, 0x800, 0xa00, 0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00,
    0xc00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0x200, 0x400, 0x600, 0x800, 0xa00,
    0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00,
    0xa00, 0xe00,
];
const OVERWORLD_AREA_BASE_Y: [u16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x200, 0, 0, 0, 0, 0x200, 0x400, 0x400, 0x400, 0x400, 0x400,
    0x400, 0x400, 0x400, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600,
    0x800, 0x600, 0x600, 0x800, 0x600, 0x600, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00,
    0xa00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xe00, 0xe00,
    0xe00, 0xc00, 0xc00, 0xe00,
];
const OVERWORLD_VERTICAL_SCROLL_TARGETS: [u16; 64] = [
    0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0x120, 0xff20,
    0xff20, 0xff20, 0xff20, 0x120, 0x320, 0x320, 0x320, 0x320, 0x320, 0x320, 0x320, 0x320, 0x520,
    0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x720, 0x520, 0x520, 0x720,
    0x520, 0x520, 0x920, 0x920, 0x920, 0x920, 0x920, 0x920, 0x920, 0x920, 0xb20, 0xb20, 0xb20,
    0xb20, 0xb20, 0xb20, 0xb20, 0xb20, 0xb20, 0xb20, 0xd20, 0xd20, 0xd20, 0xb20, 0xb20, 0xd20,
];
const OVERWORLD_HORIZONTAL_SCROLL_TARGETS: [u16; 64] = [
    0xff00, 0xff00, 0x300, 0x500, 0x500, 0x900, 0x900, 0xd00, 0xff00, 0xff00, 0x300, 0x500, 0x500,
    0x900, 0x900, 0xd00, 0xff00, 0x100, 0x300, 0x500, 0x700, 0x900, 0xb00, 0xd00, 0xff00, 0xff00,
    0x300, 0x500, 0x500, 0x900, 0xb00, 0xb00, 0xff00, 0xff00, 0x300, 0x500, 0x500, 0x900, 0xb00,
    0xb00, 0xff00, 0x100, 0x300, 0x500, 0x700, 0x900, 0xb00, 0xd00, 0xff00, 0xff00, 0x300, 0x500,
    0x700, 0x900, 0x900, 0xd00, 0xff00, 0xff00, 0x300, 0x500, 0x700, 0x900, 0x900, 0xd00,
];

fn pre_overworld_music_selection(
    sc: u8,
    dr: u8,
    queued_music_control: u8,
    sram_progress_indicator: u8,
    savegame_has_master_sword_flags: u16,
    savegame_is_darkworld: u8,
    link_item_moon_pearl: u8,
) -> (u8, u8) {
    let mut ow_anim_tiles = 0x58;
    let mut xt;
    let mut skip_darkworld_override = false;

    if matches!(sc, 3 | 5 | 7) {
        xt = 2;
    } else if matches!(sc, 0x43 | 0x45 | 0x47) {
        xt = 9;
    } else {
        ow_anim_tiles = 0x5a;
        if sc >= 0x40 {
            xt = 0xf3;
            if queued_music_control == 0xf2 {
                skip_darkworld_override = true;
            } else {
                xt = if sram_progress_indicator < 2 { 3 } else { 2 };
            }
        } else if dr == 0xe3 || dr == 0x18 || dr == 0x2f || (dr == 0x1f && sc == 0x18) {
            xt = if sram_progress_indicator < 3 { 7 } else { 2 };
        } else {
            xt = if savegame_has_master_sword_flags & 0x40 != 0 {
                2
            } else {
                5
            };
            if dr != 0 && dr != 0xe1 {
                xt = 0xf3;
                if queued_music_control == 0xf2 {
                    skip_darkworld_override = true;
                } else {
                    xt = if sram_progress_indicator < 2 { 3 } else { 2 };
                }
            }
        }
    }

    if !skip_darkworld_override && savegame_is_darkworld != 0 {
        xt = if matches!(sc, 0x40 | 0x43 | 0x45 | 0x47) {
            13
        } else {
            9
        };
        if link_item_moon_pearl == 0 {
            xt = 4;
        }
    }

    (xt, ow_anim_tiles)
}

fn overworld_offset_base_x_c_index(index: usize) -> u16 {
    if index < OVERWORLD_AREA_BASE_X.len() {
        OVERWORLD_AREA_BASE_X[index]
    } else if index < OVERWORLD_AREA_BASE_X.len() + OVERWORLD_AREA_BASE_Y.len() {
        OVERWORLD_AREA_BASE_Y[index - OVERWORLD_AREA_BASE_X.len()]
    } else {
        OVERWORLD_VERTICAL_SCROLL_TARGETS
            [index - OVERWORLD_AREA_BASE_X.len() - OVERWORLD_AREA_BASE_Y.len()]
    }
}

fn overworld_offset_base_y_c_index(index: usize) -> u16 {
    if index < OVERWORLD_AREA_BASE_Y.len() {
        OVERWORLD_AREA_BASE_Y[index]
    } else if index < OVERWORLD_AREA_BASE_Y.len() + OVERWORLD_VERTICAL_SCROLL_TARGETS.len() {
        OVERWORLD_VERTICAL_SCROLL_TARGETS[index - OVERWORLD_AREA_BASE_Y.len()]
    } else {
        OVERWORLD_HORIZONTAL_SCROLL_TARGETS
            [index - OVERWORLD_AREA_BASE_Y.len() - OVERWORLD_VERTICAL_SCROLL_TARGETS.len()]
    }
}

const OVERWORLD_AREA_HEIGHTS_BY_SIZE: [u16; 2] = [0x11e, 0x31e];
const OVERWORLD_AREA_WIDTHS_BY_SIZE: [u16; 2] = [0x100, 0x300];
const OVERWORLD_VERTICAL_SCROLL_SPANS_BY_SIZE: [u16; 2] = [0x2e0, 0x4e0];
const OVERWORLD_HORIZONTAL_SCROLL_SPANS_BY_SIZE: [u16; 2] = [0x300, 0x500];
const OVERWORLD_MAP16_STRIP_BACKTRACK_BY_DIRECTION: [u16; 3] = [0x03d0, 0x0410, 0xf410];
const SPECIAL_EXIT_TOP_BOUNDS: [u16; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0x200, 0x200, 0, 0, 0, 0, 0, 0];
const SPECIAL_EXIT_BOTTOM_BOUNDS: [u16; 16] = [
    0x120, 0x20, 0x320, 0x20, 0, 0, 0x320, 0x320, 0x320, 0x220, 0, 0, 0, 0, 0x320, 0x320,
];
const SPECIAL_EXIT_LEFT_BOUNDS: [u16; 16] = [
    0, 0x100, 0x200, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0x100, 0x200, 0x600, 0x600, 0xa00,
    0xc00, 0xc00,
];
const SPECIAL_EXIT_RIGHT_BOUNDS: [u16; 16] = [
    0, 0x100, 0x500, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0x100, 0x400, 0x600, 0x600, 0xa00,
    0xc00, 0xc00,
];
const SPECIAL_EXIT_SCROLL_Y_START: [u16; 16] = [
    0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0x120, 0xff20,
    0xff20, 0xff20, 0xff20, 0x120,
];
const SPECIAL_EXIT_SCROLL_Y_END: [u16; 16] = [
    0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0x400, 0x400, 0xff20, 0xff20, 0x120, 0xff20,
    0xff20, 0xff20, 0x400, 0x400,
];
const SPECIAL_EXIT_SCROLL_X_START: [u16; 16] = [
    0xfffc, 0x100, 0x300, 0x100, 0x500, 0x900, 0xb00, 0xb00, 0xfffc, 0x100, 0x300, 0x500, 0x500,
    0x900, 0xb00, 0xb00,
];
const SPECIAL_EXIT_SCROLL_X_END: [u16; 16] = [
    4, 0x104, 0x300, 0x100, 0x500, 0x900, 0xb00, 0xb00, 4, 0x104, 0x300, 0x100, 0x500, 0x900,
    0xb00, 0xb00,
];
const SPECIAL_EXIT_LEFT_EDGE_OF_MAP: [u16; 16] = [
    0, 0, 0x200, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0, 0x200, 0x600, 0x600, 0xa00, 0xc00, 0xc00,
];
const SPECIAL_EXIT_DIRECTIONS: [u8; 16] = [0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const SPECIAL_EXIT_SPRITE_GRAPHICS: [u8; 16] = [
    0x0c, 0x0c, 0x0e, 0x0e, 0x0e, 0x10, 0x10, 0x10, 0x0e, 0x0e, 0x0e, 0x0e, 0x10, 0x10, 0x10, 0x10,
];
const SPECIAL_EXIT_AUX_GRAPHICS: [u8; 16] = [0x2f; 16];
const SPECIAL_EXIT_BG_PALETTES: [u8; 16] = [
    0x0a, 0x0a, 0x0a, 0x0a, 2, 2, 2, 0x0a, 2, 2, 0x0a, 2, 2, 2, 2, 0x0a,
];
const SPECIAL_EXIT_SPRITE_PALETTES: [u8; 16] = [1, 8, 8, 8, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 2];
const VARIOUS_PACKS_OVERWORLD: [u8; 16] = [
    0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x5b, 0x01, 0x5a, 0x42, 0x43, 0x44, 0x45, 0x3f, 0x59, 0x0b, 0x5a,
];
const SECONDARY_OVERLAY_BY_OVERWORLD_SCREEN: [u16; 128] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1c0c, 0x1c0c, 0, 0,
    0, 0, 0, 0, 0x1c0c, 0x1c0c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x03b0,
    0x180c, 0x180c, 0x0288, 0, 0, 0, 0, 0, 0x180c, 0x180c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1ab6, 0x1ab6, 0, 0x0e2e, 0x0e2e, 0, 0, 0, 0x1ab6, 0x1ab6,
    0, 0x0e2e, 0x0e2e, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x03b0, 0, 0, 0x0288, 0, 0, 0,
    0, 0, 0, 0, 0,
];
const DARK_WORLD_PALETTE_ANIMATION_PHASE1: [u16; 35] = [
    0x0884, 0x0cc7, 0x150a, 0x154d, 0x7ff6, 0x5944, 0x7ad1, 0x0884, 0x0cc7, 0x150a, 0x154d, 0x5bff,
    0x7ad1, 0x21af, 0x1084, 0x48c0, 0x6186, 0x7e6d, 0x7fe0, 0x5944, 0x7e20, 0x1084, 0x000e, 0x1059,
    0x291f, 0x7fe0, 0x5944, 0x7e20, 0x1084, 0x1508, 0x196c, 0x21af, 0x7ff6, 0x1d4c, 0x7ad1,
];
const DARK_WORLD_PALETTE_ANIMATION_PHASE2: [u16; 40] = [
    0x7fff, 0x0884, 0x1cc8, 0x1dce, 0x3694, 0x4718, 0x1d4a, 0x18ac, 0x7fff, 0x1908, 0x2d2f, 0x3614,
    0x4eda, 0x471f, 0x1d4a, 0x390f, 0x7fff, 0x34cd, 0x5971, 0x5635, 0x7f1b, 0x7fff, 0x1d4a, 0x3d54,
    0x7fff, 0x1908, 0x2d2f, 0x3614, 0x4eda, 0x471f, 0x1d4a, 0x390f, 0x7fff, 0x0884, 0x052a, 0x21ef,
    0x3ab5, 0x4b39, 0x1d4c, 0x18ac,
];
const SPECIAL_SWITCH_AREA_TILE_IDS: [u16; 4] = [0x0105, 0x01e4, 0x00ad, 0x00b9];
const SPECIAL_SWITCH_AREA_SCREENS: [u16; 4] = [0, 45, 15, 129];
const SPECIAL_SWITCH_AREA_DIRECTIONS: [u8; 4] = [8, 2, 8, 8];
const SPECIAL_SWITCH_AREA_EXITS: [u16; 4] = [0x0180, 0x0181, 0x0182, 0x0189];
const SPECIAL_SWITCH_AREA_B_TILE_IDS: [u16; 3] = [0x017c, 0x01e4, 0x00ad];
const SPECIAL_SWITCH_AREA_B_SCREENS: [u16; 3] = [0x0080, 0x0080, 0x0081];
const SPECIAL_SWITCH_AREA_B_DIRECTIONS: [u8; 3] = [4, 1, 4];
const SPECIAL_SWITCH_MAP16_MASKS: [u16; 4] = [0x0f80, 0x0f80, 0x003f, 0x003f];
const SPECIAL_SWITCH_MAP16_OFFSETS: [u16; 256] = [
    0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x1060,
    0x1060, 0x1060, 0x1060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060,
    0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x1060,
    0x1060, 0x0060, 0x1060, 0x1060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060,
    0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060, 0x0060,
    0x0060, 0x1060, 0x1060, 0x0060, 0x0080, 0x0080, 0x0040, 0x0080, 0x0080, 0x0080, 0x0080, 0x0040,
    0x1080, 0x1080, 0x0040, 0x1080, 0x1080, 0x1080, 0x1080, 0x0040, 0x0040, 0x0040, 0x0040, 0x0040,
    0x0040, 0x0040, 0x0040, 0x0040, 0x0080, 0x0080, 0x0040, 0x0080, 0x0080, 0x0040, 0x0080, 0x0080,
    0x1080, 0x1080, 0x0040, 0x1080, 0x1080, 0x0040, 0x1080, 0x1080, 0x0040, 0x0040, 0x0040, 0x0040,
    0x0040, 0x0040, 0x0040, 0x0040, 0x0080, 0x0080, 0x0040, 0x0040, 0x0040, 0x0080, 0x0080, 0x0040,
    0x1080, 0x1080, 0x0040, 0x0040, 0x0040, 0x1080, 0x1080, 0x0040, 0x1800, 0x1840, 0x1800, 0x1800,
    0x1840, 0x1800, 0x1840, 0x1800, 0x1800, 0x1840, 0x1800, 0x1800, 0x1840, 0x1800, 0x1840, 0x1800,
    0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1840, 0x1800, 0x1800,
    0x1840, 0x1800, 0x1800, 0x1840, 0x1800, 0x1840, 0x1800, 0x1800, 0x1840, 0x1800, 0x1800, 0x1840,
    0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1800, 0x1840, 0x1800, 0x1800,
    0x1800, 0x1800, 0x1840, 0x1800, 0x1800, 0x1840, 0x1800, 0x1800, 0x1800, 0x1800, 0x1840, 0x1800,
    0x2000, 0x2040, 0x1000, 0x2000, 0x2040, 0x2000, 0x2040, 0x1000, 0x2000, 0x2040, 0x1000, 0x2000,
    0x2040, 0x2000, 0x2040, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000,
    0x2000, 0x2040, 0x1000, 0x2000, 0x2040, 0x1000, 0x2000, 0x2040, 0x2000, 0x2040, 0x1000, 0x2000,
    0x2040, 0x1000, 0x2000, 0x2040, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000,
    0x2000, 0x2040, 0x1000, 0x1000, 0x1000, 0x2000, 0x2040, 0x1000, 0x2000, 0x2040, 0x1000, 0x1000,
    0x1000, 0x2000, 0x2040, 0x1000,
];
const SPECIAL_SWITCH_AREA_DELTAS: [i16; 4] = [2, -2, 16, -16];
const OVERWORLD_AREA_TILEMAP_HEADS: [u8; 64] = [
    0, 0, 2, 3, 3, 5, 5, 7, 0, 0, 10, 3, 3, 5, 5, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 24, 26,
    27, 27, 29, 30, 30, 24, 24, 34, 27, 27, 37, 30, 30, 40, 41, 42, 43, 44, 45, 46, 47, 48, 48, 50,
    51, 52, 53, 53, 55, 48, 48, 58, 59, 60, 53, 53, 63,
];
const OVERWORLD_SCROLL_DIRECTION_BITS: [u16; 4] = [8, 4, 2, 1];
const OVERWORLD_TRANSITION_SCROLL_DELTAS: [i16; 4] = [-8, 8, -8, 8];
const OVERWORLD_TRANSITION_PLAYER_MOVE_FRAMES: [u8; 4] = [27, 27, 30, 30];
const OVERWORLD_TRANSITION_CAMERA_OFFSETS: [i16; 4] = [-0x70, 0x70, -0x70, 0x70];
const OVERWORLD_ADJACENT_AREA_DELTAS: [i16; 4] = [-8, 8, -1, 1];
const OVERWORLD_ENTRY_SETTLE_COORDINATES: [u8; 4] = [0xe0, 8, 0xe0, 0x10];

impl ZeldaState {
    fn replay_trace_door_overlay(&self, label: &str, pos: u16) {
        if std::env::var_os("ZELDA3_REPLAY_TRACE_DOOR").is_none() {
            return;
        }
        let world_location = self.world_location_state();
        let screen = world_location.overworld_screen;
        let screen_byte = world_location.overworld_screen_index();
        if screen_byte != 0x5b && pos != 0x0e2e {
            return;
        }
        let word0 = if pos < 0x2000 {
            self.dungeon_state_view().bg2_tile_by_byte_pos(pos)
        } else {
            0xffff
        };
        let word1 = if pos < 0x1ffe {
            self.dungeon_state_view()
                .bg2_tile(((pos >> 1) + 1) as usize)
        } else {
            0xffff
        };
        eprintln!(
            "door-trace frame={} {label} main={} sub={} subsub={} screen=0x{screen:04x} screenb=0x{screen_byte:02x} event=0x{:02x} owent=0x{:04x} big=0x{:04x} pos=0x{pos:04x} bg2=0x{word0:04x}/0x{word1:04x}",
            self.frame_state().frame_counter,
            self.frame_state().main_module,
            self.frame_state().submodule,
            self.frame_state().subsubmodule,
            self.overworld_event_info_view().event_info(screen_byte as usize),
            self.world_state_view().ow_entrance_value(),
            self.dungeon_state_view().big_rock_starting_address(),
        );
    }

    pub(super) fn Module08_OverworldLoad(&mut self) {
        match self.frame_state().submodule {
            0 => self.PreOverworld_LoadProperties(),
            1 => self.PreOverworld_LoadOverlays(),
            2 => self.Module08_02_LoadAndAdvance(),
            submodule => panic!("Module08_OverworldLoad invalid submodule_index: {submodule}"),
        }
    }

    pub(super) fn PreOverworld_LoadProperties(&mut self) {
        self.palette_filter_view_mut()
            .set_color_window_selection(0x82);
        self.player_state_view_mut()
            .clear_somaria_block_bg_check_flag();
        self.AdjustLinkBunnyStatus();
        if self.frame_state().main_module == 8 {
            self.LoadOverworldFromDungeon();
        } else {
            self.LoadOverworldFromSpecialOverworld();
        }
        self.Overworld_SetSongList();
        self.player_resources_view_mut().set_keys(0xff);
        self.hud_refill_logic();

        let world_location = self.world_location_state();
        let sc = world_location.overworld_screen_index();
        let dr = world_location.dungeon_room_index();
        let (xt, ow_anim_tiles) = pre_overworld_music_selection(
            sc,
            dr,
            self.system_signals_view().queued_music_control(),
            self.save_progress_view().progress_indicator(),
            self.world_state_view().savegame_has_master_sword_flags(),
            self.save_progress_view().dark_world_state(),
            self.inventory_state_view().moon_pearl(),
        );

        self.system_signals_view_mut().set_queued_music_control(xt);
        self.DecompressAnimatedOverworldTiles(ow_anim_tiles);
        self.InitializeTilesets();
        self.OverworldLoadScreensPaletteSet();
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(sc),
            self.overworld_config_table_view()
                .sprite_palette(sc as usize),
        );
        self.Palette_SetOwBgColor();
        if self.frame_state().main_module == 8 {
            self.Overworld_LoadPalettesInner();
        } else {
            self.SpecialOverworld_CopyPalettesToCache();
        }
        self.Overworld_SetFixedColAndScroll();
        self.set_overworld_fixed_color_adjustment(0);
        self.follower_initialize();

        if sc & 0x3f == 0 {
            self.DecodeAnimatedSpriteTile_variable(0x1e);
        }
        self.set_saved_module_for_menu(9);
        self.sprite_reload_all_overworld();
        if sc & 0x40 == 0 {
            self.sprite_initialize_mirror_portal();
        }
        let ambient_sound_effect = if self.save_progress_view().progress_indicator() < 2 {
            1
        } else {
            5
        };
        self.system_signals_view_mut()
            .set_ambient_sound_effect(ambient_sound_effect);
        if self.follower_state_view().indicator() == 6 {
            self.follower_state_view_mut().set_indicator(0);
        }

        self.player_state_view_mut().clear_doorway_state();
        self.player_state_view_mut().set_button_mask_b_y(0);
        self.player_state_view_mut().clear_button_b_frames();
        self.player_state_view_mut().clear_direction_lock();
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().clear_movement_velocity();
        self.player_state_view_mut()
            .clear_water_ripple_or_grass_state();
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        if self.inventory_state_view().moon_pearl() == 0
            && self.save_progress_view().dark_world_state() != 0
        {
            self.player_state_view_mut().set_bunny_state(1);
            self.player_state_view_mut().set_handler_state(23);
            self.LoadGearPalettes_bunny();
        }
        self.set_bg_mode(9);
        self.dungeon_state_view_mut().clear_lights_out_request();
        self.dungeon_state_view_mut().set_header_collision(0);
        self.player_state_view_mut().clear_lower_level_states();
        self.increment_submodule();
        self.system_signals_view_mut().increment_hud_update_flag();
        self.dungeon_state_view_mut().clear_savegame_state_bits();
        self.LoadOWMusicIfNeeded();
    }

    pub(super) fn LoadOverworldFromDungeon(&mut self) {
        self.set_indoor_flag(0);
        self.dungeon_state_view_mut()
            .clear_dungeon_dark_with_lantern();
        self.set_overworld_fixed_color_adjustment(0);
        self.save_progress_view_mut().set_palace_index_x2(0xff);
        self.memorized_tile_view_mut().clear_count();

        let room = self.world_location_state().dungeon_room;
        if room != 0x0104 && room < 0x0180 && room >= 0x0100 {
            self.LoadCachedEntranceProperties();
        } else {
            let exit_screen = self
                .asset_raw(130)
                .expect("LoadOverworldFromDungeon missing kExitData_ScreenIndex asset")
                .to_vec();
            let exit_rooms = self
                .asset_raw(131)
                .expect("LoadOverworldFromDungeon missing kExitDataRooms asset")
                .to_vec();
            let exit_map16_src = self
                .asset_raw(132)
                .expect("LoadOverworldFromDungeon missing kExitData_Map16LoadSrcOff asset")
                .to_vec();
            let exit_scroll_x = self
                .asset_raw(133)
                .expect("LoadOverworldFromDungeon missing kExitData_ScrollX asset")
                .to_vec();
            let exit_scroll_y = self
                .asset_raw(134)
                .expect("LoadOverworldFromDungeon missing kExitData_ScrollY asset")
                .to_vec();
            let exit_x = self
                .asset_raw(135)
                .expect("LoadOverworldFromDungeon missing kExitData_XCoord asset")
                .to_vec();
            let exit_y = self
                .asset_raw(136)
                .expect("LoadOverworldFromDungeon missing kExitData_YCoord asset")
                .to_vec();
            let exit_camera_x = self
                .asset_raw(137)
                .expect("LoadOverworldFromDungeon missing kExitData_CameraXScroll asset")
                .to_vec();
            let exit_camera_y = self
                .asset_raw(138)
                .expect("LoadOverworldFromDungeon missing kExitData_CameraYScroll asset")
                .to_vec();
            let exit_normal_door = self
                .asset_raw(139)
                .expect("LoadOverworldFromDungeon missing kExitData_NormalDoor asset")
                .to_vec();
            let exit_fancy_door = self
                .asset_raw(140)
                .expect("LoadOverworldFromDungeon missing kExitData_FancyDoor asset")
                .to_vec();
            let exit_scroll_up_seed = self
                .asset_raw(141)
                .expect("LoadOverworldFromDungeon missing kExitData_Unk1 asset")
                .to_vec();
            let exit_scroll_left_seed = self
                .asset_raw(142)
                .expect("LoadOverworldFromDungeon missing kExitData_Unk3 asset")
                .to_vec();

            let k = (0..79)
                .rev()
                .find(|&k| read_word_from_slice(&exit_rooms, k * 2) == room)
                .unwrap_or_else(|| {
                    panic!("LoadOverworldFromDungeon missing exit data for room {room:#06x}")
                });

            let scroll_y = read_word_from_slice(&exit_scroll_y, k * 2);
            self.ppu_scroll_copy_view_mut()
                .set_bg1_bg2_v_live_and_copy(scroll_y);
            let scroll_x = read_word_from_slice(&exit_scroll_x, k * 2);
            self.ppu_scroll_copy_view_mut()
                .set_bg1_bg2_h_live_and_copy(scroll_x);

            let link_y = read_word_from_slice(&exit_y, k * 2);
            let link_x = read_word_from_slice(&exit_x, k * 2);
            self.player_state_view_mut().set_y(link_y);
            self.player_state_view_mut().set_x(link_x);

            let src = read_word_from_slice(&exit_map16_src, k * 2);
            self.set_overworld_map16_src_off(src);
            self.set_overworld_map16_y_unit((src.wrapping_sub(0x400) & 0x0f80) >> 7);
            self.set_overworld_map16_dst_off((src.wrapping_sub(0x10) & 0x003e) >> 1);

            let camera_y = read_word_from_slice(&exit_camera_y, k * 2);
            self.world_state_view_mut()
                .set_camera_y_coord_scroll_low(camera_y);
            self.world_state_view_mut()
                .set_camera_y_coord_scroll_hi(camera_y.wrapping_sub(2));
            let camera_x = read_word_from_slice(&exit_camera_x, k * 2);
            self.world_state_view_mut()
                .set_camera_x_coord_scroll_low(camera_x);
            self.world_state_view_mut()
                .set_camera_x_coord_scroll_hi(camera_x.wrapping_sub(2));

            self.player_state_view_mut().set_facing(2);
            let entrance_value = read_word_from_slice(&exit_normal_door, k * 2);
            let big_rock_starting_address = read_word_from_slice(&exit_fancy_door, k * 2);
            self.world_state_view_mut()
                .set_ow_entrance_value(entrance_value);
            self.dungeon_state_view_mut()
                .set_big_rock_starting_address(big_rock_starting_address);
            let screen = exit_screen[k] as u16;
            self.world_state_view_mut()
                .set_overworld_area_index_word(screen);
            self.set_overworld_screen_word(screen);

            let scroll_up_seed = exit_scroll_up_seed[k] as i8 as i16 as u16;
            let scroll_left_seed = exit_scroll_left_seed[k] as i8 as i16 as u16;
            self.world_state_view_mut()
                .set_overworld_scroll_up_counter(scroll_up_seed);
            self.world_state_view_mut()
                .set_overworld_scroll_left_counter(scroll_left_seed);
            self.world_state_view_mut()
                .set_overworld_scroll_down_counter(scroll_up_seed.wrapping_neg());
            self.world_state_view_mut()
                .set_overworld_scroll_right_counter(scroll_left_seed.wrapping_neg());
        }

        self.Overworld_LoadNewScreenProperties();
    }

    pub(super) fn Overworld_EnterSpecialArea(&mut self) {
        self.memorized_tile_view_mut().clear_count();
        self.world_state_view_mut().save_spexit_area_index();
        self.world_state_view_mut().save_spexit_tm_copy();
        self.ppu_scroll_copy_view_mut()
            .save_special_exit_bg2_live_scroll();
        self.special_exit_position_view_mut().store_from_player();
        self.world_state_view_mut().save_spexit_camera_coords();
        let overworld_screen = self.world_location_state().overworld_screen;
        self.world_state_view_mut()
            .set_spexit_screen_index(overworld_screen);
        let map16 = self.overworld_map16_load_state();
        self.store_overworld_spexit_map16_src_off(map16.src_off);
        let top = self.room_bounds_view().packed_top();
        let bottom = self.room_bounds_view().packed_bottom();
        let left = self.room_bounds_view().packed_left();
        let right = self.room_bounds_view().packed_right();
        self.world_state_view_mut()
            .set_special_exit_room_bounds(top, bottom, left, right);
        self.world_state_view_mut().copy_spexit_scroll_targets();
        self.world_state_view_mut().copy_spexit_scroll_counters();
        self.world_state_view_mut().save_spexit_tile_themes();
        self.sprite_system_view_mut()
            .save_special_exit_graphics_index();
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some() {
            println!(
                "spexit-save frame={} area=0x{:04x} screen=0x{:04x} x=0x{:04x} y=0x{:04x} bg=0x{:04x}/0x{:04x} src=0x{:04x} yunit=0x{:04x} dst=0x{:04x} cam=0x{:04x}/0x{:04x} room=0x{:04x} main={} sub={}",
                self.frame_state().frame_counter,
                self.world_state_view().spexit_area_index(),
                self.world_state_view().spexit_screen_index(),
                self.special_exit_position_view().x(),
                self.special_exit_position_view().y(),
                self.ppu_scroll_copy_view().special_exit_bg2_h_copy2(),
                self.ppu_scroll_copy_view().special_exit_bg2_v_copy2(),
                self.overworld_spexit_map16_src_off(),
                self.overworld_map16_y_unit(),
                self.overworld_map16_dst_off(),
                self.world_state_view().spexit_camera_x_scroll_low(),
                self.world_state_view().spexit_camera_y_scroll_low(),
                self.world_location_state().dungeon_room,
                self.frame_state().main_module,
                self.frame_state().submodule,
            );
        }

        self.LoadOverworldFromDungeon();
        if self.world_location_state().dungeon_room == 0x1010 {
            self.set_dungeon_room(0x182);
        }

        let room_bak = self.world_location_state().dungeon_room_index();
        self.decrement_dungeon_room_index_by(0x80);
        let i = self.world_location_state().dungeon_room_index() as usize;
        self.player_state_view_mut()
            .set_facing(SPECIAL_EXIT_DIRECTIONS[i]);
        self.reset_incremental_vram_upload_counter();
        self.sprite_system_view_mut()
            .set_graphics_index(SPECIAL_EXIT_SPRITE_GRAPHICS[i]);
        self.world_state_view_mut()
            .set_aux_tile_theme_index(SPECIAL_EXIT_AUX_GRAPHICS[i]);
        self.Overworld_LoadPalettes(SPECIAL_EXIT_BG_PALETTES[i], SPECIAL_EXIT_SPRITE_PALETTES[i]);

        let j = (self.world_location_state().dungeon_room_index() & 0x3f) as usize;
        self.world_state_view_mut()
            .set_overworld_offset_base_y(SPECIAL_EXIT_TOP_BOUNDS[j]);
        self.world_state_view_mut()
            .set_overworld_offset_base_x(SPECIAL_EXIT_LEFT_EDGE_OF_MAP[j] >> 3);
        self.world_state_view_mut()
            .set_overworld_offset_mask_y(0x03f0);
        self.world_state_view_mut()
            .set_overworld_offset_mask_x(0x03f0 >> 3);

        let k = (self.world_location_state().dungeon_room_index() & 0x7f) as usize;
        self.room_bounds_view_mut().set_packed_bounds(
            SPECIAL_EXIT_TOP_BOUNDS[k],
            SPECIAL_EXIT_BOTTOM_BOUNDS[k],
            SPECIAL_EXIT_LEFT_BOUNDS[k],
            SPECIAL_EXIT_RIGHT_BOUNDS[k],
        );
        self.world_state_view_mut()
            .set_up_down_scroll_target(SPECIAL_EXIT_SCROLL_Y_START[k]);
        self.world_state_view_mut()
            .set_up_down_scroll_target_end(SPECIAL_EXIT_SCROLL_Y_END[k]);
        self.world_state_view_mut()
            .set_left_right_scroll_target(SPECIAL_EXIT_SCROLL_X_START[k]);
        self.world_state_view_mut()
            .set_left_right_scroll_target_end(SPECIAL_EXIT_SCROLL_X_END[k]);

        self.set_dungeon_room_index(room_bak);
        self.Palette_SpecialOw();
    }

    pub(super) fn GetOverworldBgPalette(&self, idx: u8) -> u8 {
        self.asset_raw(109)
            .expect("GetOverworldBgPalette missing kOverworldBgPalettes asset")[idx as usize]
    }

    pub(super) fn Overworld_SetFixedColAndScroll(&mut self) {
        self.set_sub_screen_layers(0);
        let si = self.world_location_state().overworld_screen_index() as u16;
        let mut p = 0x19c6;
        if si == 0x80 {
            if self.world_location_state().dungeon_room == 0x181 {
                self.set_sub_screen_layers(1);
                p = if si & 0x40 != 0 { 0x2a32 } else { 0x2669 };
            }
        } else if si != 0x81 {
            p = 0;
            if si != 0x5b && (si & 0xbf) != 3 && (si & 0xbf) != 5 && (si & 0xbf) != 7 {
                p = if si & 0x40 != 0 { 0x2a32 } else { 0x2669 };
            }
        }
        self.palette_buffer_view_mut().set_main_color(0, p);
        self.palette_buffer_view_mut().set_aux_color(0, p);
        self.palette_buffer_view_mut().set_main_color(32, p);
        self.palette_buffer_view_mut().set_aux_color(32, p);
        self.palette_filter_view_mut().set_fixed_color_red(0x20);
        self.palette_filter_view_mut().set_fixed_color_green(0x40);
        self.palette_filter_view_mut().set_fixed_color_blue(0x80);
        if si != 0 && si != 0x40 && si != 0x5b {
            if si == 0x70 {
                self.set_sub_screen_layers(1);
                self.system_signals_view_mut().increment_cgram_update_flag();
                return;
            }
            let cv = if si == 3 || si == 5 || si == 7 {
                0x8c4c26
            } else if si == 0x43 || si == 0x45 {
                0x874a26
            } else {
                self.system_signals_view_mut().increment_cgram_update_flag();
                return;
            };
            self.palette_filter_view_mut().set_fixed_color_red(cv as u8);
            self.palette_filter_view_mut()
                .set_fixed_color_green((cv >> 8) as u8);
            self.palette_filter_view_mut()
                .set_fixed_color_blue((cv >> 16) as u8);
        }
        if self.frame_state().submodule != 4 {
            self.ppu_scroll_copy_view_mut().copy_bg2_live_to_bg1_live();
            if (si & 0x3f) == 0x1b {
                let bg2_hofs = self.world_state_view().bg2_x();
                let y = (bg2_hofs.wrapping_sub(0x0778) as i16) >> 1;
                self.ppu_scroll_copy_view_mut()
                    .set_bg1_h_copy2(bg2_hofs.wrapping_sub(y as u16));

                let mut a = self.world_state_view().bg1_y();
                if a >= 0x06c0 {
                    a = a.wrapping_sub(0x0600) & 0x03ff;
                    let value = if a < 0x0180 {
                        (a >> 1) | 0x0600
                    } else {
                        0x06c0
                    };
                    self.world_state_view_mut().set_bg1_y(value);
                } else {
                    self.world_state_view_mut()
                        .set_bg1_y(((a & 0x00ff) >> 1) | 0x0600);
                }
            }
        } else if (si & 0x3f) == 0x1b {
            let value = if self.world_state_view().overworld_screen_trans_dir_bits() != 8 {
                0x0838
            } else {
                self.world_state_view().bg2_x()
            };
            self.world_state_view_mut().set_bg1_x(value);
            self.world_state_view_mut().set_bg1_y(0x06c0);
        }
        self.set_sub_screen_layers(1);
        self.system_signals_view_mut().increment_cgram_update_flag();
    }

    pub(super) fn Ancilla_TerminateWaterfallSplashes(&mut self) {
        if self.world_location_state().overworld_screen_index() == 0x0f {
            for i in (0..=4).rev() {
                if self.ancilla_slot_view(i).ancilla_type() == 0x41 {
                    self.ancilla_slot_view_mut(i).clear();
                }
            }
        }
    }

    pub(super) fn Module09_LoadAuxGFX(&mut self) {
        self.overworld_event_info_view_mut()
            .clear_event_bits(0x3b, 0x20);
        self.overworld_event_info_view_mut()
            .clear_event_bits(0x7b, 0x20);

        let saved267 = self.save_progress_view().dungeon_info_word(267) & !0x0080;
        let saved40 = self.save_progress_view().dungeon_info_word(40) & !0x0100;
        self.save_progress_view_mut()
            .set_dungeon_info_word(267, saved267);
        self.save_progress_view_mut()
            .set_dungeon_info_word(40, saved40);

        self.LoadTransAuxGFX();
        self.PrepTransAuxGfx();
        self.set_core_update_disable_flag(9);
        self.set_pending_nmi_subroutine(9);
        self.increment_submodule();
    }

    pub(super) fn Overworld_LoadOverlays2(&mut self) {
        let overworld_screen = self.world_location_state().overworld_screen;
        self.world_state_view_mut()
            .set_prev_screen_index_word(overworld_screen);
        self.store_overworld_prev_map16_load_state(self.overworld_map16_load_state());
        let screen_transition = self.world_state_view().screen_transition();
        self.world_state_view_mut()
            .set_prev_screen_transition(screen_transition);
        self.world_state_view_mut().save_prev_screen_trans_bits();

        self.world_state_view_mut().clear_overlay_index_word();
        self.ppu_scroll_copy_view_mut().clear_bg1_scroll_subpixels();

        let si = u16::from(self.world_location_state().overworld_screen_index());
        let mut xv;
        if si >= 0x80 {
            xv = 0x97;
            let room = self.world_location_state().dungeon_room;
            if room == 0x0180 {
                if self.overworld_event_info_view().event_info(0x80) & 0x40 != 0 {
                    self.set_sub_screen_layers(0);
                    self.increment_submodule();
                    return;
                }
            } else if room == 0x0181 {
                xv = 0x94;
            } else if room == 0x0189 {
                xv = 0x93;
            } else {
                if room == 0x0182 || room == 0x0183 {
                    self.system_signals_view_mut().set_ambient_sound_effect(1);
                }
                self.set_sub_screen_layers(0);
                self.increment_submodule();
                return;
            }
        } else if (si & 0x3f) == 0 {
            xv = if (si & 0x40) == 0
                && self.overworld_event_info_view().event_info(0x80) & 0x40 != 0
            {
                0x9e
            } else {
                0x9d
            };
        } else if matches!(si, 0x03 | 0x05 | 0x07) {
            xv = 0x95;
        } else if matches!(si, 0x43 | 0x45 | 0x47) {
            xv = 0x9c;
        } else if si == 0x70 {
            xv = 0x9c;
            if self.overworld_event_info_view().event_info(0x70) & 0x20 == 0 {
                xv = 0x9f;
            }
        } else {
            xv = if self.save_progress_view().progress_indicator() < 2 {
                0x9f
            } else {
                0x96
            };
        }

        self.set_overworld_map16_src_off(0x0390);
        self.world_state_view_mut().set_overlay_index_word(xv);
        self.set_overworld_screen_word(xv);
        let src = self.overworld_map16_src_off();
        self.set_overworld_map16_y_unit((src.wrapping_sub(0x400) & 0x0f80) >> 7);
        self.set_overworld_map16_dst_off((src.wrapping_sub(0x10) & 0x003e) >> 1);
        self.world_state_view_mut().clear_screen_transition();
        self.world_state_view_mut()
            .clear_overworld_screen_trans_dir_bits();
        self.world_state_view_mut()
            .clear_screen_transition_direction_bits_word();
        self.palette_filter_view_mut()
            .set_color_window_selection(0x82);
        self.set_main_screen_layers(0x16);
        self.set_sub_screen_layers(1);
        let ambient_sound_effect = self
            .overworld_config_table_view()
            .music(self.world_location_state().overworld_screen_index() as usize)
            >> 4;
        self.system_signals_view_mut()
            .set_ambient_sound_effect(ambient_sound_effect);

        if matches!(xv, 0x97 | 0x94 | 0x93 | 0x9d | 0x9e | 0x9f) {
            self.palette_filter_view_mut().set_color_math_control(0x72);
        } else {
            let prev = self.world_state_view().prev_screen_index_byte();
            if xv == 0x95
                || xv == 0x9c
                || prev == 0x5b
                || (prev == 0x1b
                    && (self.frame_state().submodule == 35 || self.frame_state().submodule == 44))
            {
                self.palette_filter_view_mut().set_color_math_control(0x20);
            } else {
                self.set_sub_screen_layers(0);
                self.palette_filter_view_mut().set_color_math_control(0x20);
            }
        }

        self.LoadOverworldOverlay();
        if self.world_state_view().overlay_index() == 0x94 {
            let value = self.world_state_view().bg1_y() | 0x0100;
            self.world_state_view_mut().set_bg1_y(value);
        }

        let overworld_screen = self.world_state_view().prev_screen_index_word();
        self.set_overworld_screen_word(overworld_screen);
        self.store_overworld_map16_load_state(self.overworld_prev_map16_load_state());
        let screen_transition = self.world_state_view().prev_screen_transition();
        self.world_state_view_mut()
            .set_screen_transition(screen_transition);
        self.world_state_view_mut().restore_prev_screen_trans_bits();
    }

    pub(super) fn Overworld_LoadOverlays(&mut self) {
        self.sprite_initialize_slots();
        self.sprite_reload_all_overworld();
        let mut player = self.player_state_view_mut();
        player.clear_state_bits();
        player.clear_picking_throw_state();
        self.system_signals_view_mut().set_ambient_sound_effect(5);
        self.Overworld_LoadOverlays2();
    }

    pub(super) fn PreOverworld_LoadOverlays(&mut self) {
        self.system_signals_view_mut().set_ambient_sound_effect(5);
        self.Overworld_LoadOverlays2();
    }

    pub(super) fn Overworld_LoadAmbientOverlay(&mut self, load_map_data: bool) {
        let bak_src_off = self.overworld_map16_src_off();
        let bak_dst_off = self.overworld_map16_dst_off();
        let bak_y_unit = self.overworld_map16_y_unit();
        if self.overworld_map_is_small() {
            self.set_small_overworld_mirror_map_position();
        }
        if load_map_data {
            self.Overworld_DrawQuadrantsAndOverlays();
        }
        self.Map16ToMap8(0x2000, 0);
        self.set_overworld_map16_y_unit(bak_y_unit);
        self.set_overworld_map16_dst_off(bak_dst_off);
        self.set_overworld_map16_src_off(bak_src_off);
        self.set_pending_nmi_subroutine(4);
        self.set_core_update_disable_flag(4);
        self.increment_submodule();
        self.set_screen_brightness(0);
    }

    pub(super) fn Overworld_LoadAmbientOverlayFalse(&mut self) {
        self.Overworld_LoadAmbientOverlay(false);
    }

    pub(super) fn Overworld_LoadAndBuildScreen(&mut self) {
        self.Overworld_LoadAmbientOverlay(true);
    }

    pub(super) fn LoadOverworldOverlay(&mut self) {
        self.OverworldLoad_LoadSubOverlayMap32();
        self.Map16ToMap8(0x4000, 0x1000);
        self.set_pending_nmi_subroutine(4);
        self.set_core_update_disable_flag(4);
        self.increment_submodule();
    }

    pub(super) fn GetMap8toTileAttr(&self) -> Vec<u8> {
        self.asset_raw(163)
            .expect("GetMap8toTileAttr missing kMap8DataToTileAttr asset")
            .to_vec()
    }

    pub(super) fn GetMap16toMap8Table(&self) -> Vec<u8> {
        self.asset_raw(70)
            .expect("GetMap16toMap8Table missing kMap16ToMap8 asset")
            .to_vec()
    }

    pub(super) fn LookupInOwEntranceTab(&self, r0: u16, r2: u16) -> bool {
        for i in (0..OVERWORLD_ENTRANCE_PRIMARY_AREA_BY_INDEX.len()).rev() {
            if r0 == OVERWORLD_ENTRANCE_PRIMARY_AREA_BY_INDEX[i]
                && r2 == OVERWORLD_ENTRANCE_SECONDARY_AREA_BY_INDEX[i]
            {
                return true;
            }
        }
        false
    }

    pub(super) fn LookupInOwEntranceTab2(&self, pos: u16) -> i32 {
        let entrance_pos = self
            .asset_raw(125)
            .expect("LookupInOwEntranceTab2 missing kOverworld_Entrance_Pos asset");
        let entrance_area = self
            .asset_raw(124)
            .expect("LookupInOwEntranceTab2 missing kOverworld_Entrance_Area asset");
        for i in (0..=128).rev() {
            if pos == read_word_from_slice(entrance_pos, i * 2)
                && self.world_state_view().overworld_area_index_word()
                    == read_word_from_slice(entrance_area, i * 2)
            {
                return i as i32;
            }
        }
        -1
    }

    pub(super) fn CanEnterWithTagalong(&self, e: i32) -> bool {
        let t = self.follower_state_view().indicator();
        t == 0 || t == 5 || t == 14 || t == 1 || (t == 7 || t == 8) && e >= 59
    }

    pub(super) fn Module09_Overworld(&mut self) {
        self.replay_trace_submodule("module09-entry");
        match self.frame_state().submodule {
            0 => self.Module09_00_PlayerControl(),
            1 | 15 | 26 | 38 => self.Module09_LoadAuxGFX(),
            2 | 16 | 27 | 39 => self.Overworld_FinishTransGfx(),
            3 | 17 => self.Module09_LoadNewMapAndGFX(),
            4 | 18 => self.Module09_LoadNewSprites(),
            5 | 19 => self.Overworld_StartScrollTransition(),
            6 | 20 => self.Overworld_RunScrollTransition(),
            7 | 21 => self.Overworld_EaseOffScrollTransition(),
            8 => self.Overworld_FinalizeEntryOntoScreen(),
            9 => self.Module09_09_OpenBigDoorFromExiting(),
            10 => self.Module09_0A_WalkFromExiting_FacingDown(),
            11 => self.Module09_0B_WalkFromExiting_FacingUp(),
            12 => self.Module09_0C_OpenBigDoor(),
            13 | 23 | 36 => self.Overworld_StartMosaicTransition(),
            14 => self.PreOverworld_LoadOverlays(),
            22 | 41 => self.Module09_FadeBackInFromMosaic(),
            24 => self.Overworld_Func18(),
            25 => self.Overworld_Func19(),
            28 => self.Overworld_Func1C(),
            29 => self.Overworld_Func1D(),
            30 => self.Overworld_Func1E(),
            31 => self.Overworld_Func1F(),
            32 => self.Overworld_LoadOverlays2(),
            33 => self.Overworld_LoadAmbientOverlayFalse(),
            34 => self.Overworld_Func22(),
            35 | 44 => self.Module09_MirrorWarp(),
            37 => self.Overworld_LoadOverlays(),
            40 => self.Overworld_LoadAndBuildScreen(),
            42 => self.Module09_2A_RecoverFromDrowning(),
            43 => self.Overworld_Func2B(),
            45 => self.Overworld_WeathervaneExplosion(),
            46 => self.Module09_2E_Whirlpool(),
            47 => self.Overworld_Func2F(),
            submodule => panic!("Module09_Overworld invalid submodule_index: {submodule}"),
        }
        self.replay_trace_submodule("module09-after-submodule");

        let bg2x = self.world_state_view().bg2_x();
        let bg2y = self.world_state_view().bg2_y();
        let bg1x = self.world_state_view().bg1_x();
        let bg1y = self.world_state_view().bg1_y();
        let offx = self.world_state_view().bg1_x_offset();
        let offy = self.world_state_view().bg1_y_offset();

        let bg2x_off = bg2x.wrapping_add(offx);
        let bg2y_off = bg2y.wrapping_add(offy);
        let bg1x_off = bg1x.wrapping_add(offx);
        let bg1y_off = bg1y.wrapping_add(offy);
        self.ppu_scroll_copy_view_mut()
            .set_bg2_h_live_and_copy(bg2x_off);
        self.ppu_scroll_copy_view_mut()
            .set_bg2_v_live_and_copy(bg2y_off);
        self.ppu_scroll_copy_view_mut()
            .set_bg1_h_live_and_copy(bg1x_off);
        self.ppu_scroll_copy_view_mut()
            .set_bg1_v_live_and_copy(bg1y_off);

        self.replay_trace_ram_watch("module09-before-sprite-main");
        self.sprite_main();
        self.replay_trace_ram_watch("module09-after-sprite-main");

        self.world_state_view_mut().set_bg2_x(bg2x);
        self.world_state_view_mut().set_bg2_y(bg2y);
        self.world_state_view_mut().set_bg1_x(bg1x);
        self.world_state_view_mut().set_bg1_y(bg1y);
        self.replay_trace_ram_watch("module09-after-scroll-restore");

        self.replay_trace_ram_watch("module09-before-link-oam");
        self.link_oam_main();
        self.replay_trace_ram_watch("module09-after-link-oam");
        self.hud_refill_logic();
        self.replay_trace_ram_watch("module09-after-refill");
        self.OverworldOverlay_HandleRain();
        self.replay_trace_ram_watch("module09-after-rain");
        self.replay_trace_submodule("module09-exit");
    }

    pub(super) fn Module09_00_PlayerControl(&mut self) {
        self.replay_trace_submodule("module09-player-entry");
        if (self.world_state_view().flag_custom_spell_anim_active()
            | self.player_state_view().immobilized_flag()
            | self.player_state_view().menu_block_flag()
            | self.world_state_view().trigger_special_entrance())
            == 0
        {
            if self.player_state_view().filtered_joypad_h() & 0x10 != 0 {
                self.world_state_view_mut().set_overworld_map_state(0);
                self.set_submodule(1);
                self.save_main_module_for_menu();
                self.set_main_module(14);
                return;
            }
            if self.DidPressButtonForMap() {
                self.world_state_view_mut().set_overworld_map_state(0);
                self.set_submodule(7);
                self.save_main_module_for_menu();
                self.set_main_module(14);
                return;
            }
            if self.player_state_view().joypad1h_last() & 0x20 != 0 {
                self.DisplaySelectMenu();
                return;
            }
            self.hud_handle_item_switch_inputs();
        }

        if self.world_state_view().trigger_special_entrance() != 0 {
            self.Overworld_AnimateEntrance();
        }
        self.replay_trace_ram_watch("module09-player-before-link-main");
        self.link_main();
        self.replay_trace_ram_watch("module09-player-after-link-main");
        if self.world_state_view().super_bomb_indicator_timer() != 0xff {
            self.hud_super_bomb_indicator();
        }
        self.replay_trace_ram_watch("module09-player-after-super-bomb");
        let area = ((self.player_state_view().y() & 0x1e00) >> 5)
            | ((self.player_state_view().x() & 0x1e00) >> 8);
        self.world_state_view_mut()
            .set_current_area_of_player_word(area);
        self.Graphics_LoadChrHalfSlot();
        self.replay_trace_ram_watch("module09-player-after-chr");
        self.Overworld_OperateCameraScroll();
        self.replay_trace_ram_watch("module09-player-after-camera");
        if self.frame_state().main_module != 11 {
            self.Overworld_UseEntrance();
            self.replay_trace_ram_watch("module09-player-after-use-entrance");
            self.Overworld_DwDeathMountainPaletteAnimation();
            self.replay_trace_ram_watch("module09-player-after-dm-palette");
            self.OverworldHandleTransitions();
            self.replay_trace_ram_watch("module09-player-after-transitions");
        } else {
            self.ScrollAndCheckForSOWExit();
            self.replay_trace_ram_watch("module09-player-after-sow-exit");
        }
        self.replay_trace_submodule("module09-player-exit");
    }

    pub(super) fn Overworld_UseEntrance(&mut self) {
        let xc = self.player_state_view().x() >> 3;
        let yc = self.player_state_view().y().wrapping_add(7);
        let mut pos = ((yc.wrapping_sub(self.world_state_view().overworld_offset_base_y())
            & self.world_state_view().overworld_offset_mask_y())
            * 8)
        .wrapping_add(
            xc.wrapping_sub(self.world_state_view().overworld_offset_base_x())
                & self.world_state_view().overworld_offset_mask_x(),
        );

        let mut x = self.dungeon_state_view().bg2_tile_by_byte_pos(pos) as usize * 4;
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("Overworld_UseEntrance missing kMap16ToMap8 asset")
            .to_vec();
        if self.player_state_view().facing() == 0 {
            let mut a = read_word_from_slice(&map16_to_map8, (x + 1) * 2) & 0x41ff;
            if a == 0x00e9 {
                self.overworld_draw_map16_persist(pos, 0x0da4);
                self.overworld_draw_map16_persist(pos.wrapping_add(2), 0x0da6);
                self.system_signals_view_mut().set_sound_effect_2(21);
                self.set_bg_vram_load_mode(1);
                return;
            }
            if a != 0x0149 && a != 0x0169 {
                x = self
                    .dungeon_state_view()
                    .bg2_tile(((pos >> 1) + 1) as usize) as usize
                    * 4;
                a = read_word_from_slice(&map16_to_map8, x * 2) & 0x41ff;
                if a == 0x40e9 {
                    pos = pos.wrapping_sub(2);
                    self.overworld_draw_map16_persist(pos, 0x0da4);
                    self.overworld_draw_map16_persist(pos.wrapping_add(2), 0x0da6);
                    self.system_signals_view_mut().set_sound_effect_2(21);
                    self.set_bg_vram_load_mode(1);
                    return;
                }
                if a == 0x4149 || a == 0x4169 {
                    pos = pos.wrapping_sub(2);
                } else {
                    a = 0;
                }
            }
            if a == 0x0149 || a == 0x0169 || a == 0x4149 || a == 0x4169 {
                self.dungeon_state_view_mut().clear_door_open_counter_low();
                if a & 0x20 != 0 {
                    if self.save_progress_view().progress_indicator() & 0x0f >= 3 {
                        // Mirror the C goto after: skip opening, continue entrance lookup.
                    } else {
                        self.dungeon_state_view_mut().set_door_open_counter_low(24);
                        self.dungeon_state_view_mut()
                            .set_big_rock_starting_address(pos.wrapping_sub(0x80));
                        self.system_signals_view_mut().set_sound_effect_2(21);
                        self.set_subsubmodule(0);
                        self.world_state_view_mut().set_door_animation_step(0);
                        self.set_submodule(12);
                        return;
                    }
                } else {
                    self.dungeon_state_view_mut()
                        .set_big_rock_starting_address(pos.wrapping_sub(0x80));
                    self.system_signals_view_mut().set_sound_effect_2(21);
                    self.set_subsubmodule(0);
                    self.world_state_view_mut().set_door_animation_step(0);
                    self.set_submodule(12);
                    return;
                }
            }
        }

        if !self.LookupInOwEntranceTab(
            read_word_from_slice(&map16_to_map8, (x + 2) * 2) & 0x01ff,
            read_word_from_slice(&map16_to_map8, (x + 3) * 2) & 0x01ff,
        ) {
            self.world_state_view_mut()
                .set_big_key_door_message_triggered(0);
            return;
        }

        let lx = self.LookupInOwEntranceTab2(pos);
        if lx < 0 {
            return;
        }
        let entrance = self
            .asset_raw(126)
            .expect("Overworld_UseEntrance missing kOverworld_Entrance_Id asset")[lx as usize];
        if self.follower_state_view().dropped() == 0
            && (self.player_state_view().item_hold_pose() == 1
                || !self.CanEnterWithTagalong(i32::from(entrance).wrapping_sub(1)))
        {
            if self.world_state_view().big_key_door_message_triggered() == 0 {
                self.world_state_view_mut()
                    .set_big_key_door_message_triggered(1);
                self.dialogue_message_index_view_mut().set_value(5);
                self.main_show_text_message();
            }
        } else {
            self.world_state_view_mut()
                .set_which_entrance_byte(entrance);
            self.player_state_view_mut().clear_auxiliary_state();
            self.player_state_view_mut().set_incapacitated_timer(0);
            self.set_main_module(15);
            self.set_saved_module_for_menu(6);
            self.set_submodule(0);
            self.set_subsubmodule(0);
        }
    }

    pub(super) fn Overworld_AnimateEntrance(&mut self) {
        let j = self.world_state_view().trigger_special_entrance();
        self.player_state_view_mut().set_immobilized_flag(j);
        self.set_modal_pause_flag(j);
        self.set_core_update_disable_flag(j);
        match j {
            1 => self.Overworld_AnimateEntrance_PoD(),
            2 => self.Overworld_AnimateEntrance_Skull(),
            3 => self.Overworld_AnimateEntrance_Mire(),
            4 => self.Overworld_AnimateEntrance_TurtleRock(),
            5 => self.Overworld_AnimateEntrance_GanonsTower(),
            _ => panic!("Overworld_AnimateEntrance invalid trigger_special_entrance: {j}"),
        }
    }

    fn entrance_counter_inc_is(&mut self, target: u8) -> bool {
        self.world_state_view_mut()
            .increment_entrance_sequence_counter();
        self.world_state_view().entrance_sequence_counter() == target
    }

    fn entrance_draw_tiles(&mut self, entries: &[(u16, u16)]) {
        for &(pos, tile) in entries {
            self.overworld_draw_map16_persist(pos, tile);
        }
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn OverworldEntrance_AdvanceAndBoom(&mut self) {
        self.increment_subsubmodule();
        self.world_state_view_mut()
            .clear_entrance_sequence_counter();
        self.system_signals_view_mut().set_sound_effect_1(12);
        self.system_signals_view_mut().set_sound_effect_2(7);
    }

    pub(super) fn OverworldEntrance_PlayJingle(&mut self) {
        self.system_signals_view_mut().set_sound_effect_2(27);
        self.world_state_view_mut().clear_trigger_special_entrance();
        self.set_subsubmodule(0);
        self.clear_core_update_disable_flag();
        self.player_state_view_mut().clear_immobilized();
        self.clear_modal_pause_flag();
        self.world_state_view_mut().set_bg1_x_offset(0);
        self.world_state_view_mut().set_bg1_y_offset(0);
    }

    pub(super) fn OverworldEntrance_DrawManyTR(&mut self) {
        const POS: [u16; 16] = [
            0x099e, 0x09a0, 0x09a2, 0x09a4, 0x0a1e, 0x0a20, 0x0a22, 0x0a24, 0x0a9e, 0x0aa0, 0x0aa2,
            0x0aa4, 0x0b1e, 0x0b20, 0x0b22, 0x0b24,
        ];
        for (i, pos) in POS.into_iter().enumerate() {
            self.overworld_draw_map16_persist(pos, 0x0e78 + i as u16);
        }
        self.set_bg_vram_load_mode(1);
        self.set_core_update_disable_flag(1);
    }

    pub(super) fn Overworld_AnimateEntrance_PoD(&mut self) {
        match self.frame_state().subsubmodule {
            0 => {
                if !self.entrance_counter_inc_is(0x40) {
                    return;
                }
                self.OverworldEntrance_AdvanceAndBoom();
                self.overworld_event_info_view_mut()
                    .set_event_bits(0x5e, 0x20);
                self.entrance_draw_tiles(&[
                    (0x01e6, 0x0e31),
                    (0x02ea, 0x0e30),
                    (0x026a, 0x0e26),
                    (0x02ea, 0x0e27),
                ]);
            }
            1 => {
                if !self.entrance_counter_inc_is(0x20) {
                    return;
                }
                self.OverworldEntrance_AdvanceAndBoom();
                self.entrance_draw_tiles(&[(0x026a, 0x0e28), (0x02ea, 0x0e29)]);
            }
            2 => {
                if !self.entrance_counter_inc_is(0x20) {
                    return;
                }
                self.OverworldEntrance_AdvanceAndBoom();
                self.entrance_draw_tiles(&[(0x026a, 0x0e2a), (0x02ea, 0x0e2b), (0x036a, 0x0e2c)]);
            }
            3 => {
                if !self.entrance_counter_inc_is(0x20) {
                    return;
                }
                self.OverworldEntrance_AdvanceAndBoom();
                self.entrance_draw_tiles(&[(0x026a, 0x0e2d), (0x02ea, 0x0e2e), (0x036a, 0x0e2f)]);
            }
            4 => {
                if self.entrance_counter_inc_is(0x20) {
                    self.OverworldEntrance_PlayJingle();
                }
            }
            _ => {}
        }
    }

    pub(super) fn Overworld_AnimateEntrance_Skull(&mut self) {
        let entries: &[(u16, u16)] = match self.frame_state().subsubmodule {
            0 => {
                if !self.entrance_counter_inc_is(4) {
                    return;
                }
                &[(0x409 * 2, 0x0e06), (0x40a * 2, 0x0e06)]
            }
            1 => {
                if !self.entrance_counter_inc_is(12) {
                    return;
                }
                &[
                    (0x3c8 * 2, 0x0e07),
                    (0x3c9 * 2, 0x0e08),
                    (0x3ca * 2, 0x0e09),
                    (0x3cb * 2, 0x0e0a),
                ]
            }
            2 => {
                if !self.entrance_counter_inc_is(12) {
                    return;
                }
                &[
                    (0x388 * 2, 0x0e07),
                    (0x389 * 2, 0x0e08),
                    (0x38a * 2, 0x0e09),
                    (0x38b * 2, 0x0e0a),
                ]
            }
            3 => {
                if !self.entrance_counter_inc_is(12) {
                    return;
                }
                &[
                    (0x2c8 * 2, 0x0e11),
                    (0x2cb * 2, 0x0e12),
                    (0x308 * 2, 0x0e0d),
                    (0x309 * 2, 0x0e0e),
                    (0x30a * 2, 0x0e0f),
                    (0x30b * 2, 0x0e10),
                    (0x349 * 2, 0x0e0b),
                    (0x34a * 2, 0x0e0c),
                ]
            }
            4 => {
                if !self.entrance_counter_inc_is(12) {
                    return;
                }
                &[
                    (0x2c8 * 2, 0x0e13),
                    (0x2cb * 2, 0x0e14),
                    (0x308 * 2, 0x0e15),
                    (0x309 * 2, 0x0e16),
                    (0x30a * 2, 0x0e17),
                    (0x30b * 2, 0x0e18),
                    (0x349 * 2, 0x0e19),
                    (0x34a * 2, 0x0e1a),
                ]
            }
            _ => return,
        };
        self.world_state_view_mut()
            .clear_entrance_sequence_counter();
        self.increment_subsubmodule();
        if self.frame_state().subsubmodule == 1 {
            let screen = self.world_location_state().overworld_screen_index() as usize;
            self.overworld_event_info_view_mut()
                .set_event_bits(screen, 0x20);
        }
        self.entrance_draw_tiles(entries);
        self.system_signals_view_mut().set_sound_effect_2(0x16);
        if self.frame_state().subsubmodule == 5 {
            self.OverworldEntrance_PlayJingle();
        }
    }

    fn draw_mire_body(&mut self, start: u16) {
        const POS: [u16; 12] = [
            0x0622, 0x0624, 0x0626, 0x0628, 0x06a2, 0x06a4, 0x06a6, 0x06a8, 0x0722, 0x0724, 0x0726,
            0x0728,
        ];
        for (i, pos) in POS.into_iter().enumerate() {
            self.overworld_draw_map16_persist(pos, start + i as u16);
        }
        self.set_bg_vram_load_mode(1);
    }

    fn draw_mire_row(&mut self, row: u16, start: u16) -> u16 {
        for i in 0..4 {
            self.overworld_draw_map16_persist(row + i * 2, start + i);
        }
        start + 4
    }

    pub(super) fn Overworld_AnimateEntrance_Mire(&mut self) {
        const BITS: [u8; 26] = [
            0xff, 0xf7, 0xf7, 0xfb, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xaa, 0xaa, 0xaa, 0xaa,
            0xaa, 0xaa, 0xaa, 0x88, 0x88, 0x88, 0x88, 0x80, 0x80, 0x80, 0x80, 0x80,
        ];

        if self.frame_state().subsubmodule >= 2 {
            let x = if self.frame_state().frame_counter & 1 != 0 {
                (-1i16) as u16
            } else {
                1
            };
            self.world_state_view_mut().set_bg1_x_offset(x);
            self.world_state_view_mut()
                .set_bg1_y_offset(x.wrapping_neg());
        }

        match self.frame_state().subsubmodule {
            0 => {
                self.world_state_view_mut()
                    .increment_entrance_sequence_counter();
                let mut j = self.world_state_view().entrance_sequence_counter() as u16;
                if j < 32 {
                    return;
                }
                j -= 32;
                if j == 207 {
                    self.set_subsubmodule(1);
                    self.world_state_view_mut()
                        .clear_entrance_sequence_counter();
                }
                self.set_sub_screen_layers(u8::from(
                    BITS[(j >> 3) as usize] & (0x80 >> (j & 7)) != 0,
                ));
            }
            1 | 2 => {
                self.world_state_view_mut()
                    .increment_entrance_sequence_counter();
                let j = self.world_state_view().entrance_sequence_counter();
                if j == 16 {
                    self.increment_subsubmodule();
                    self.system_signals_view_mut().set_ambient_sound_effect(7);
                }
                if j == 72 {
                    self.OverworldEntrance_AdvanceAndBoom();
                    let screen = self.world_location_state().overworld_screen_index() as usize;
                    self.overworld_event_info_view_mut()
                        .set_event_bits(screen, 0x20);
                    self.draw_mire_body(0x0e48);
                }
            }
            3 => {
                if self.entrance_counter_inc_is(72) {
                    self.OverworldEntrance_AdvanceAndBoom();
                    let j = self.draw_mire_row(0x05a2, 0x0e54);
                    self.draw_mire_body(j);
                }
            }
            4 => {
                if self.entrance_counter_inc_is(80) {
                    self.OverworldEntrance_AdvanceAndBoom();
                    let j = self.draw_mire_row(0x0522, 0x0e64);
                    let j = self.draw_mire_row(0x05a2, j);
                    self.draw_mire_body(j);
                }
            }
            5 => {
                if self.entrance_counter_inc_is(128) {
                    self.OverworldEntrance_PlayJingle();
                    self.system_signals_view_mut().set_ambient_sound_effect(5);
                }
            }
            _ => {}
        }
    }

    pub(super) fn Overworld_AnimateEntrance_TurtleRock(&mut self) {
        let x = if self.frame_state().frame_counter & 1 != 0 {
            (-1i16) as u16
        } else {
            1
        };
        self.world_state_view_mut().set_bg1_x_offset(x);
        self.world_state_view_mut()
            .set_bg1_y_offset(x.wrapping_neg());

        match self.frame_state().subsubmodule {
            0 => {
                let screen = self.world_location_state().overworld_screen_index() as usize;
                self.overworld_event_info_view_mut()
                    .set_event_bits(screen, 0x20);
                self.Dungeon_ApproachFixedColor_variable(0);
                self.turtle_rock_vram_common(0x10);
            }
            1 => self.turtle_rock_vram_common(0x14),
            2 => self.turtle_rock_vram_common(0x18),
            3 => self.turtle_rock_vram_common(0x1c),
            4 => {
                for i in 0..8 {
                    self.palette_buffer_view_mut().set_main_color(0x58 + i, 0);
                    self.palette_buffer_view_mut().set_aux_color(0x68 + i, 0);
                }
                self.ppu_scroll_copy_view_mut().copy_bg2_live_to_bg1_live();
                self.increment_subsubmodule();
                self.system_signals_view_mut().increment_cgram_update_flag();
            }
            5 => {
                self.OverworldEntrance_DrawManyTR();
                self.set_sub_screen_layers(1);
                self.palette_filter_view_mut().set_color_window_selection(2);
                self.palette_filter_view_mut().set_color_math_control(0x22);
                let end = self.display_state().vram_upload_cursor_usize();
                let mut off = 0usize;
                while off != end {
                    let v0 = self.vram_upload_data_view().word(off) | 0x10;
                    self.vram_upload_data_view_mut().set_word(off, v0);
                    for word in [2usize, 3] {
                        let offset = off + word * 2;
                        if self.vram_upload_data_view().word(offset) == 0x08aa {
                            self.vram_upload_data_view_mut().set_word(offset, 0x01e3);
                        }
                    }
                    off += 8;
                }
                self.world_state_view_mut()
                    .clear_entrance_sequence_counter();
                self.increment_subsubmodule();
            }
            6 => {
                if self.frame_state().frame_counter & 1 == 0 {
                    if self.world_state_view().entrance_sequence_counter() & 7 == 0 {
                        self.PaletteFilter_RestoreAdditive(0xb0, 0xc0);
                        self.PaletteFilter_RestoreSubtractive(0xd0, 0xe0);
                        self.system_signals_view_mut().increment_cgram_update_flag();
                        self.system_signals_view_mut().set_sound_effect_2(2);
                    }
                    self.world_state_view_mut()
                        .decrement_entrance_sequence_counter();
                    if self.world_state_view().entrance_sequence_counter() == 0 {
                        self.world_state_view_mut()
                            .set_entrance_sequence_counter(0x30);
                        self.increment_subsubmodule();
                    }
                }
            }
            7 => {
                if self.frame_state().frame_counter & 1 == 0
                    && self.world_state_view().entrance_sequence_counter() & 7 == 0
                {
                    self.system_signals_view_mut().set_sound_effect_2(2);
                }
                self.world_state_view_mut()
                    .decrement_entrance_sequence_counter();
                if self.world_state_view().entrance_sequence_counter() == 0 {
                    self.OverworldEntrance_DrawManyTR();
                    self.set_sub_screen_layers(0);
                    self.palette_filter_view_mut()
                        .set_color_window_selection(0x82);
                    self.palette_filter_view_mut().set_color_math_control(0x20);
                    self.increment_subsubmodule();
                    self.system_signals_view_mut().set_ambient_sound_effect(5);
                }
            }
            8 => self.OverworldEntrance_PlayJingle(),
            _ => {}
        }
    }

    fn turtle_rock_vram_common(&mut self, first: u16) {
        self.vram_upload_data_view_mut().set_word(0, first);
        self.vram_upload_data_view_mut().set_word(2, 0xfe47);
        self.vram_upload_data_view_mut().set_word(4, 0x01e3);
        self.vram_upload_data_view_mut().set_byte(6, 0xff);
        self.increment_subsubmodule();
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn Overworld_AnimateEntrance_GanonsTower(&mut self) {
        match self.frame_state().subsubmodule {
            0 | 1 => {
                let screen = self.world_location_state().overworld_screen_index() as usize;
                self.overworld_event_info_view_mut()
                    .set_event_bits(screen, 0x20);
                self.GanonTowerEntrance_Func1();
            }
            2 => {
                self.GanonTowerEntrance_Func1();
                if self.display_state().sub_screen_layers == 0 {
                    self.set_sub_screen_layers(1);
                    self.world_state_view_mut()
                        .increment_entrance_sequence_counter();
                    if self.world_state_view().entrance_sequence_counter() == 3 {
                        self.world_state_view_mut()
                            .clear_entrance_sequence_counter();
                        self.system_signals_view_mut().set_ambient_sound_effect(7);
                    } else {
                        self.set_subsubmodule(0);
                    }
                }
            }
            3 => self.ganon_tower_draw_after(
                48,
                &[
                    (0x045e, 0x0e88),
                    (0x0460, 0x0e89),
                    (0x04de, 0x0ea2),
                    (0x04e0, 0x0ea3),
                    (0x055e, 0x0e8a),
                    (0x0560, 0x0e8b),
                ],
            ),
            4 => self.ganon_tower_draw_after(
                48,
                &[
                    (0x045e, 0x0e8c),
                    (0x0460, 0x0e8d),
                    (0x04de, 0x0e8e),
                    (0x04e0, 0x0e8f),
                    (0x055e, 0x0e90),
                    (0x0560, 0x0e91),
                ],
            ),
            5 => self.ganon_tower_draw_after(
                52,
                &[
                    (0x045e, 0x0e92),
                    (0x0460, 0x0e93),
                    (0x04de, 0x0e94),
                    (0x04e0, 0x0e94),
                    (0x055e, 0x0e95),
                    (0x0560, 0x0e95),
                ],
            ),
            6 => self.ganon_tower_draw_after(
                32,
                &[
                    (0x045e, 0x0e96),
                    (0x0460, 0x0e97),
                    (0x04de, 0x0e98),
                    (0x04e0, 0x0e99),
                ],
            ),
            7 => self.ganon_tower_draw_after(32, &[(0x04de, 0x0e9a), (0x04e0, 0x0e9b)]),
            8 => self.ganon_tower_draw_after(
                32,
                &[
                    (0x04de, 0x0e9c),
                    (0x04e0, 0x0e9d),
                    (0x055e, 0x0e9e),
                    (0x0560, 0x0e9f),
                ],
            ),
            9 => self.ganon_tower_draw_after(32, &[(0x055e, 0x0e9a), (0x0560, 0x0e9b)]),
            10 => self.ganon_tower_draw_after(
                32,
                &[
                    (0x055e, 0x0e9c),
                    (0x0560, 0x0e9d),
                    (0x05de, 0x0ea0),
                    (0x05e0, 0x0ea1),
                ],
            ),
            11 => {
                if self.entrance_counter_inc_is(32) {
                    self.system_signals_view_mut().set_ambient_sound_effect(5);
                    self.OverworldEntrance_AdvanceAndBoom();
                    self.entrance_draw_tiles(&[(0x05de, 0x0e9a), (0x05e0, 0x0e9b)]);
                }
            }
            12 => {
                if self.entrance_counter_inc_is(72) {
                    self.OverworldEntrance_PlayJingle();
                    self.world_state_view_mut()
                        .clear_entrance_sequence_counter();
                    self.system_signals_view_mut().set_music_control(13);
                    self.system_signals_view_mut().set_ambient_sound_effect(9);
                }
            }
            _ => {}
        }
    }

    fn ganon_tower_draw_after(&mut self, target: u8, entries: &[(u16, u16)]) {
        if self.entrance_counter_inc_is(target) {
            self.OverworldEntrance_AdvanceAndBoom();
            self.entrance_draw_tiles(entries);
        }
    }

    pub(super) fn DirToEnum(&self, mut dir: i32) -> i32 {
        let mut xx = 3;
        while dir & 1 == 0 {
            xx -= 1;
            dir >>= 1;
        }
        xx
    }

    pub(super) fn Overworld_GetSignText(&self, area: i32) -> u16 {
        let signs = self
            .asset_raw(110)
            .expect("Overworld_GetSignText missing kOverworld_SignText asset");
        read_word_from_slice(signs, area as usize * 2)
    }

    pub(super) fn GetOverworldSpritePtr(&self, area: i32) -> Vec<u8> {
        let base = if self.save_progress_view().progress_indicator() == 3 {
            2
        } else if self.save_progress_view().progress_indicator() == 2 {
            1
        } else {
            0
        };
        let offsets = self
            .asset_raw(159)
            .expect("GetOverworldSpritePtr missing kOverworldSpriteOffs asset");
        let offset = read_word_from_slice(offsets, (area as usize + base * 144) * 2) as usize;
        self.asset_raw(160)
            .expect("GetOverworldSpritePtr missing kOverworldSprites asset")[offset..]
            .to_vec()
    }

    pub(super) fn GetOverworldHibytes(&self, i: i32) -> Vec<u8> {
        self.asset_memblk(105, i as usize)
            .unwrap_or_else(|| panic!("GetOverworldHibytes missing block {i}"))
            .ptr
            .to_vec()
    }

    pub(super) fn GetOverworldLobytes(&self, i: i32) -> Vec<u8> {
        self.asset_memblk(106, i as usize)
            .unwrap_or_else(|| panic!("GetOverworldLobytes missing block {i}"))
            .ptr
            .to_vec()
    }

    pub(super) fn AdjustLinkBunnyStatus(&mut self) {
        if self.inventory_state_view().moon_pearl() != 0 {
            self.ForceNonbunnyStatus();
        }
    }

    pub(super) fn ForceNonbunnyStatus(&mut self) {
        self.player_state_view_mut().set_handler_state(0);
        self.player_state_view_mut().clear_temp_bunny_timer();
        self.player_state_view_mut().clear_bunny_transform_flags();
        if self.enhanced_features_view().has(4) {
            self.player_state_view_mut().clear_running();
        }
    }

    pub(super) fn RecoverPositionAfterDrowning(&mut self) {
        self.player_state_view_mut().restore_position_from_cached();
        self.room_bounds_view_mut()
            .copy_y_bound_from(0, CACHED_ROOM_BOUNDS_Y_START);
        self.room_bounds_view_mut()
            .copy_y_bound_from(2, CACHED_ROOM_BOUNDS_Y_END);
        self.room_bounds_view_mut()
            .copy_x_bound_from(0, CACHED_ROOM_BOUNDS_X_START);
        self.room_bounds_view_mut()
            .copy_x_bound_from(2, CACHED_ROOM_BOUNDS_X_END);

        self.world_state_view_mut()
            .restore_scroll_targets_from_cached();

        if self.world_location_state().is_indoors() {
            self.world_state_view_mut()
                .restore_camera_y_from_cached_indoor();
            self.world_state_view_mut()
                .restore_camera_x_from_cached_indoor();
        }
        self.world_state_view_mut()
            .restore_quadrant_fullsize_from_cached();
        self.player_state_view_mut().restore_quadrants_from_cached();
        if self.world_location_state().is_outdoors() {
            self.world_state_view_mut().update_camera_hi_outdoor();
        }

        self.player_state_view_mut().restore_facing_from_cached();
        self.player_state_view_mut()
            .restore_lower_level_state_from_cached();
        let doorway_state = self.world_state_view().is_standing_in_doorway_cached();
        self.player_state_view_mut()
            .set_doorway_state(doorway_state);
        self.dungeon_state_view_mut().restore_cached_floor();
        self.player_state_view_mut().set_visibility_status(0);
        self.player_state_view_mut().set_blink_countdown(0x90);
        self.Dungeon_PlayBlipAndCacheQuadrantVisits();
        self.player_state_view_mut()
            .clear_sprite_damage_disable_timer();
        self.link_reset_state_after_damaging_pit();
        self.follower_state_view_mut().set_appearance_none_flag(0);
        self.follower_initialize();
        self.dungeon_state_view_mut()
            .clear_water_puzzle_state_changed();
        self.world_state_view_mut().set_overworld_map_state(0);
        self.set_subsubmodule(0);
        self.world_state_view_mut().clear_screen_transition();
        self.set_submodule(0);
        if self.player_resources_view().current_health() == 0 {
            let main_screen_layers = self.display_state().main_screen_layers;
            let sub_screen_layers = self.display_state().sub_screen_layers;
            self.world_state_view_mut()
                .set_mapbak_tm(main_screen_layers);
            self.world_state_view_mut().set_mapbak_ts(sub_screen_layers);
            self.save_main_module_for_menu();
            self.set_main_module(18);
            self.set_submodule(1);
            self.player_state_view_mut().clear_blink_countdown();
        }
    }

    pub(super) fn Module09_2A_RecoverFromDrowning(&mut self) {
        match self.frame_state().subsubmodule {
            0 => self.Module09_2A_00_ScrollToLand(),
            _ => self.RecoverPositionAfterDrowning(),
        }
    }

    pub(super) fn Module09_2A_00_ScrollToLand(&mut self) {
        let cached_x = self.player_state_view().cached_x();
        let cached_y = self.player_state_view().cached_y();

        let mut x = self.player_state_view().x();
        let mut xd = 0u16;
        if x != cached_x {
            let d = if x > cached_x { -1 } else { 1 };
            x = x.wrapping_add_signed(d);
            if x != cached_x {
                x = x.wrapping_add_signed(d);
            }
            xd = x.wrapping_sub(self.player_state_view().x());
            self.player_state_view_mut().set_x(x);
        }

        let mut y = self.player_state_view().y();
        let mut yd = 0u16;
        if y != cached_y {
            let d = if y > cached_y { -1 } else { 1 };
            y = y.wrapping_add_signed(d);
            if y != cached_y {
                y = y.wrapping_add_signed(d);
            }
            yd = y.wrapping_sub(self.player_state_view().y());
            self.player_state_view_mut().set_y(y);
        }

        self.player_state_view_mut()
            .set_movement_velocity_from_delta(xd, yd);
        if y == cached_y && x == cached_x {
            self.increment_subsubmodule();
            self.player_state_view_mut().set_incapacitated_timer(0);
            self.world_state_view_mut()
                .clear_set_when_damaging_enemies();
        }
        self.Overworld_OperateCameraScroll();
        if self
            .world_state_view()
            .has_screen_transition_direction_bits()
        {
            self.OverworldHandleMapScroll();
        }
    }

    pub(super) fn TakeDamageFromPit(&mut self) {
        self.replay_trace_submodule("take_damage_from_pit-entry");
        self.player_state_view_mut().set_visibility_status(12);
        let submodule = if self.world_location_state().is_indoors() {
            20
        } else {
            42
        };
        self.set_submodule(submodule);
        let health = self
            .player_resources_view_mut()
            .decrement_current_health_by(8);
        if health >= 0xa8 {
            self.player_resources_view_mut().set_current_health(0);
        }
        self.replay_trace_submodule("take_damage_from_pit-exit");
    }

    pub(super) fn Overworld_GetPitDestination(&mut self) {
        let x = self.player_state_view().x() & !7;
        let y = self.player_state_view().y() & !7;
        let ws = self.world_state_view();
        let pos = ((y.wrapping_sub(ws.overworld_offset_base_y()) & ws.overworld_offset_mask_y())
            << 3)
            .wrapping_add(
                ((x >> 3).wrapping_sub(ws.overworld_offset_base_x()))
                    & ws.overworld_offset_mask_x(),
            );

        let fall_hole_area = self
            .asset_raw(127)
            .expect("Overworld_GetPitDestination missing kFallHole_Area asset");
        let fall_hole_pos = self
            .asset_raw(128)
            .expect("Overworld_GetPitDestination missing kFallHole_Pos asset");
        let fall_hole_entrances = self
            .asset_raw(129)
            .expect("Overworld_GetPitDestination missing kFallHole_Entrances asset");
        for i in (0..=18).rev() {
            if read_word_from_slice(fall_hole_pos, i * 2) == pos
                && read_word_from_slice(fall_hole_area, i * 2)
                    == self.world_state_view().overworld_area_index()
            {
                let entrance = fall_hole_entrances[i];
                self.world_state_view_mut()
                    .set_which_entrance_byte(entrance);
                self.world_state_view_mut().set_overworld_hole_scan_step(0);
                return;
            }
        }

        self.save_progress_view_mut().set_dark_world_state(0);
        self.world_state_view_mut().set_which_entrance_byte(130);
        self.world_state_view_mut().set_overworld_hole_scan_step(0);
    }

    pub(super) fn Overworld_ToolAndTileInteraction(&mut self, x: u16, y: u16) -> u16 {
        self.world_state_view_mut()
            .set_overworld_hole_tilemap_pos(0);
        self.tile_detect_position_view_mut().set_interacting_tile(0);

        let ws = self.world_state_view();
        let pos = ((y.wrapping_sub(ws.overworld_offset_base_y()) & ws.overworld_offset_mask_y())
            .wrapping_mul(8))
        .wrapping_add(x.wrapping_sub(ws.overworld_offset_base_x()) & ws.overworld_offset_mask_x());
        let attr = self.dungeon_state_view().bg2_tile_by_byte_pos(pos);
        let mut yv = 0u16;
        let mut reveal = false;

        let item_in_hand = self.player_state_view().item_in_hand();
        let position_mode = self.player_state_view().position_mode();
        if item_in_hand & 2 == 0 {
            if item_in_hand & 0x40 == 0 {
                if matches!(
                    attr,
                    0x034 | 0x071 | 0x035 | 0x10d | 0x10f | 0x0e1 | 0x0e2 | 0x0da | 0x0f8 | 0x10e
                ) {
                    if position_mode != 1 {
                        return attr;
                    }
                    if self.world_location_state().overworld_screen_index() == 0x2a && pos == 0x0492
                    {
                        self.world_state_view_mut()
                            .set_overworld_hole_tilemap_pos(pos);
                    }
                    yv = 0x0dc9;
                    reveal = true;
                } else if attr == 0x037e {
                    if position_mode == 1 {
                        return attr;
                    }
                    self.tile_detect_position_view_mut()
                        .set_interaction_scratch_y(x.wrapping_mul(8).wrapping_sub(8));
                    self.tile_detect_position_view_mut()
                        .set_interaction_scratch_x(y.wrapping_sub(8) & !7);
                    self.tile_detect_position_view_mut().set_interacting_tile(3);
                    yv = 0x0dc5;
                    reveal = true;
                }
            }

            if !reveal {
                if attr == 0x036 || attr == 0x072a {
                    if position_mode != 1 {
                        self.tile_detect_position_view_mut()
                            .set_interaction_scratch_y((x & !1).wrapping_mul(8));
                        self.tile_detect_position_view_mut()
                            .set_interaction_scratch_x(y & !0x0f);
                        let terrain = if attr == 0x036 { 2 } else { 4 };
                        self.tile_detect_position_view_mut()
                            .set_interacting_tile(terrain);
                        yv = if attr == 0x072a { 0x0dc8 } else { 0x0dc7 };
                        reveal = true;
                    }
                } else {
                    return attr;
                }
            }
        } else if attr == 0x021b {
            self.system_signals_view_mut().set_sound_effect_1(17);
            self.HandlePegPuzzles(pos);
            yv = 0x0dcb;
            reveal = true;
        } else {
            self.Overworld_PickHammerSfx(attr);
            return attr;
        }

        if reveal {
            let secret = self.Overworld_RevealSecret(pos);
            if secret != 0 {
                yv = secret;
            }
            self.dungeon_state_view_mut()
                .set_bg2_tile_by_byte_pos(pos, yv);
            self.Overworld_Memorize_Map16_Change(pos, yv);
            self.overworld_draw_map16(pos, yv);
            self.set_bg_vram_load_mode(1);
        }

        let map8_index = attr as usize * 4 + (((y & 8) >> 2) | (x & 1)) as usize;
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("Overworld_ToolAndTileInteraction missing kMap16ToMap8 asset");
        let tile_attrs = self
            .asset_raw(163)
            .expect("Overworld_ToolAndTileInteraction missing kMap8DataToTileAttr asset");
        let map8 = read_word_from_slice(map16_to_map8, map8_index * 2);
        let tile_attr = tile_attrs[(map8 & 0x01ff) as usize] as u16;
        let terrain = self.tile_detect_position_view().interacting_tile();
        if terrain != 0 {
            let sx = self.tile_detect_position_view().interaction_scratch_y();
            let sy = self.tile_detect_position_view().interaction_scratch_x();
            self.sprite_spawn_immediately_smashed_terrain(terrain as u8, sx, sy);
            self.ancilla_add_bush_poof(sx, sy);
        }
        tile_attr
    }

    pub(super) fn Overworld_PickHammerSfx(&mut self, a: u16) {
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("Overworld_PickHammerSfx missing kMap16ToMap8 asset");
        let tile_attrs = self
            .asset_raw(163)
            .expect("Overworld_PickHammerSfx missing kMap8DataToTileAttr asset");
        let map8 = read_word_from_slice(map16_to_map8, a as usize * 8);
        let attr = tile_attrs[(map8 & 0x01ff) as usize];
        let sound_effect_1 = if attr < 0x50 {
            return;
        } else if attr < 0x52 {
            26
        } else if attr < 0x54 {
            17
        } else if attr < 0x58 {
            5
        } else {
            return;
        };
        self.system_signals_view_mut()
            .set_sound_effect_1(sound_effect_1);
    }

    pub(super) fn Overworld_HandleLiftableTiles(&mut self, pt_arg: &mut Point16U) -> u8 {
        let pos = self.overworld_get_link_map16_coords(pt_arg);
        let pt = *pt_arg;
        let a = self.dungeon_state_view().bg2_tile_by_byte_pos(pos);
        if matches!(a, 0x36d | 0x23b) {
            return self.smash_rock_pile_from_lift(a, pos, 0, pt);
        }
        if matches!(a, 0x36e | 0x23c) {
            return self.smash_rock_pile_from_lift(a, pos, 1, pt);
        }
        if matches!(a, 0x374 | 0x23d) {
            return self.smash_rock_pile_from_lift(a, pos, 2, pt);
        }
        if matches!(a, 0x375 | 0x23e) {
            return self.smash_rock_pile_from_lift(a, pos, 3, pt);
        }

        let y = match a {
            0x36 => Some(0x0dc7),
            0x72a => Some(0x0dc8),
            0x20f | 0x239 => Some(0x0dca),
            0x101 => Some(0x0dc6),
            _ => None,
        };
        if let Some(y) = y {
            return self.overworld_lifting_small_obj(a, pos, y, pt);
        }

        let t =
            a as usize * 4 + if pt.x & 8 != 0 { 2 } else { 0 } + if pt.y & 8 != 0 { 1 } else { 0 };
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("Overworld_HandleLiftableTiles missing kMap16ToMap8 asset");
        let tile_attrs = self
            .asset_raw(163)
            .expect("Overworld_HandleLiftableTiles missing kMap8DataToTileAttr asset");
        let map8 = read_word_from_slice(map16_to_map8, t * 2);
        tile_attrs[(map8 & 0x01ff) as usize]
    }

    pub(super) fn Module10_00_OpenIris(&mut self) {
        self.Spotlight_open();
        self.increment_submodule();
    }

    pub(super) fn OverworldOverlay_HandleRain(&mut self) {
        const X: [u8; 4] = [1, 0, 1, 0];
        const Y: [u8; 4] = [0, 17, 0, 17];

        if (self.world_location_state().overworld_screen_index() != 0x70
            && self.save_progress_view().progress_indicator() >= 2)
            || (self.overworld_event_info_view().event_info(0x70) & 0x20) != 0
        {
            return;
        }

        match self.frame_state().frame_counter {
            3 | 88 => self.palette_filter_view_mut().set_color_math_control(0x32),
            5 | 44 | 90 => self.palette_filter_view_mut().set_color_math_control(0x72),
            36 => {
                self.system_signals_view_mut().set_sound_effect_1(54);
                self.palette_filter_view_mut().set_color_math_control(0x32);
            }
            _ => {}
        }
        if self.frame_state().frame_counter & 3 != 0 {
            return;
        }
        let i = self.world_state_view_mut().increment_move_overlay_ctr();
        let bg1x = self
            .world_state_view()
            .bg1_x()
            .wrapping_add((X[i as usize] as u16) << 8);
        let bg1y = self
            .world_state_view()
            .bg1_y()
            .wrapping_add((Y[i as usize] as u16) << 8);
        self.world_state_view_mut().set_bg1_x(bg1x);
        self.world_state_view_mut().set_bg1_y(bg1y);
    }

    pub(super) fn Overworld_ResetMosaicDown(&mut self) {
        if self.palette_filter_view().countdown() & 1 != 0 {
            self.decrement_mosaic_level_by(0x10);
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(7);
    }

    pub(super) fn Overworld_Func1D(&mut self) {
        // Overworld_Func1D is an assert-only dispatch slot in the C port.
        panic!("Overworld_Func1D reached");
    }

    pub(super) fn Overworld_Func1E(&mut self) {
        // Overworld_Func1E is an assert-only dispatch slot in the C port.
        panic!("Overworld_Func1E reached");
    }

    pub(super) fn Overworld_FinishTransGfx(&mut self) {
        self.set_core_update_disable_flag(10);
        self.set_pending_nmi_subroutine(10);
        self.increment_submodule();
    }

    pub(super) fn Overworld_Func22(&mut self) {
        self.increment_screen_brightness();
        if self.display_state().screen_brightness == 15 {
            self.set_submodule(0);
            self.set_subsubmodule(0);
        }
    }

    pub(super) fn Overworld_Func18(&mut self) {
        self.player_state_view_mut().clear_swim_fast_state();
        let module = self.frame_state().main_module;
        let submodule = self.frame_state().submodule;
        self.Overworld_EnterSpecialArea();
        self.Overworld_LoadOverlays();
        self.set_submodule(submodule.wrapping_add(1));
        self.set_main_module(module);
    }

    pub(super) fn Overworld_Func19(&mut self) {
        let module = self.frame_state().main_module;
        let submodule = self.frame_state().submodule;
        self.Module08_02_LoadAndAdvance();
        self.set_submodule(submodule.wrapping_add(1));
        self.set_main_module(module);
    }

    pub(super) fn Overworld_Func2B(&mut self) {
        self.Palette_AnimGetMasterSword();
    }

    pub(super) fn Overworld_WeathervaneExplosion(&mut self) {}

    pub(super) fn InitializeMirrorHDMA(&mut self) {
        self.clear_hdma_enable_mask();

        self.mirror_warp_scratch_view_mut()
            .initialize_hdma_wave_state();

        self.hdma_setup(0xf2fb, 0xf2fb, 0x42, 0x0d, 0x0f, 0);

        let value = self.world_state_view().bg2_x();
        for i in 0..240 {
            self.spotlight_hdma_view_mut()
                .set_hdma_table_dynamic_entry(i, value);
        }
        self.set_hdma_enable_mask(0xc0);
    }

    pub(super) fn MirrorWarp_BuildWavingHDMATable(&mut self) {
        self.MirrorWarp_RunAnimationSubmodules();
        if self.frame_state().frame_counter & 1 != 0 {
            return;
        }

        let mut y = 240usize - 8;
        loop {
            let value = self
                .spotlight_hdma_view_mut()
                .hdma_table_dynamic_entry(y - 8);
            for off in [0usize, 2, 4, 6] {
                self.spotlight_hdma_view_mut()
                    .set_hdma_table_dynamic_entry(y + off, value);
            }
            y -= 8;
            if y == 0 {
                break;
            }
        }

        let mirror = self.mirror_warp_scratch_view();
        let target = mirror.target_offset();
        let mut t = mirror.displacement().wrapping_add(mirror.velocity_delta());
        if !sign16(t.wrapping_sub(target) ^ target) {
            t = target;
            let mut mirror = self.mirror_warp_scratch_view_mut();
            mirror.reset_wave_and_subpixel();
            mirror.toggle_target_index();
        }
        self.mirror_warp_scratch_view_mut().set_displacement(t);
        t = t.wrapping_add(self.mirror_warp_scratch_view().subpixel());
        self.mirror_warp_scratch_view_mut().set_subpixel_low_from(t);
        if sign16(t) {
            t |= 0x00ff;
        } else {
            t &= !0x00ff;
        }
        t = self
            .mirror_warp_scratch_view()
            .wave_offset()
            .wrapping_add(t.swap_bytes());
        self.mirror_warp_scratch_view_mut().set_wave_offset(t);
        if self.palette_filter_view().countdown() >= 0x30 && (t & !7) == 0 {
            self.mirror_warp_scratch_view_mut()
                .shrink_target_offsets_for_dewaving();
            self.increment_subsubmodule();
            t = 0;
        }
        let value = t.wrapping_add(self.world_state_view().bg2_x());
        for off in [0usize, 2, 4, 6] {
            self.spotlight_hdma_view_mut()
                .set_hdma_table_dynamic_entry(off, value);
        }
    }

    pub(super) fn MirrorWarp_BuildDewavingHDMATable(&mut self) {
        self.MirrorWarp_RunAnimationSubmodules();
        if self.frame_state().frame_counter & 1 != 0 {
            return;
        }

        let mut y = 240usize - 8;
        loop {
            let value = self
                .spotlight_hdma_view_mut()
                .hdma_table_dynamic_entry(y - 8);
            for off in [0usize, 2, 4, 6] {
                self.spotlight_hdma_view_mut()
                    .set_hdma_table_dynamic_entry(y + off, value);
            }
            y -= 8;
            if y == 0 {
                break;
            }
        }

        let t = self
            .spotlight_hdma_view_mut()
            .hdma_table_dynamic_entry(0x0c0)
            | self
                .spotlight_hdma_view_mut()
                .hdma_table_dynamic_entry(0x0c8)
            | self
                .spotlight_hdma_view_mut()
                .hdma_table_dynamic_entry(0x0d0)
            | self
                .spotlight_hdma_view_mut()
                .hdma_table_dynamic_entry(0x0d8);
        if t == self.world_state_view().bg2_x() {
            self.clear_hdma_enable_mask();
            self.increment_subsubmodule();
            self.Overworld_SetFixedColAndScroll();
            if self.world_location_state().overworld_screen_index() & 0x3f != 0x1b {
                let bg2x = self.world_state_view().bg2_x();
                let bg2y = self.world_state_view().bg2_y();
                self.world_state_view_mut().set_bg1_x(bg2x);
                self.ppu_scroll_copy_view_mut().set_bg1_h_copy(bg2x);
                self.ppu_scroll_copy_view_mut().set_bg2_h_copy(bg2x);
                self.world_state_view_mut().set_bg1_y(bg2y);
                self.ppu_scroll_copy_view_mut().set_bg1_v_copy(bg2y);
                self.ppu_scroll_copy_view_mut().set_bg2_v_copy(bg2y);
            }
        }
    }

    pub(super) fn MirrorWarp_FinalizeAndLoadDestination(&mut self) {
        self.hdma_setup(0, 0xf2fb, 0x41, 0, 0x26, 0);
        self.IrisSpotlight_ResetTable();
        self.palette_filter_view_mut().set_countdown_word(0);
        self.palette_filter_view_mut()
            .set_darkening_or_lightening_screen_word(0);
        self.ReloadPreviouslyLoadedSheets();
        self.Overworld_SetSongList();
        self.set_hdma_enable_mask(0x80);

        let screen = self.world_location_state().overworld_screen_index() as usize;
        let music = self.overworld_config_table_view().music(screen);
        self.system_signals_view_mut()
            .set_music_control(music & 0x0f);
        self.system_signals_view_mut()
            .set_ambient_sound_effect(music >> 4);
        if self.world_location_state().overworld_screen_index() >= 0x40
            && self.inventory_state_view().moon_pearl() == 0
        {
            self.system_signals_view_mut().set_music_control(4);
        }

        self.save_submodule_for_menu();
        self.set_submodule(0);
        self.set_subsubmodule(0);
        self.world_state_view_mut().set_overworld_map_state(0);
        self.clear_core_update_disable_flag();
    }

    pub(super) fn Module09_MirrorWarp(&mut self) {
        self.display_nmi_view_mut()
            .increment_core_update_disable_flag();
        match self.frame_state().subsubmodule {
            0 => {
                if self.world_location_state().overworld_screen_index() >= 0x80 {
                    self.set_submodule(0);
                    self.set_subsubmodule(0);
                    self.world_state_view_mut().set_overworld_map_state(0);
                    return;
                }
                self.system_signals_view_mut().set_music_control(8);
                self.world_state_view_mut()
                    .set_flag_overworld_area_changed(8);
                self.player_state_view_mut().set_blink_countdown(0x90);
                self.InitializeMirrorHDMA();
                self.save_progress_view_mut().xor_dark_world_state(0x40);
                self.world_state_view_mut()
                    .set_overworld_peg_puzzle_progress(0);
                let screen = (self.world_location_state().overworld_screen_index() & 0x3f)
                    | self.save_progress_view().dark_world_state();
                self.set_overworld_screen(screen);
                self.world_state_view_mut().set_overworld_area_index(screen);
                self.world_state_view_mut().set_overworld_map_state(0);
                self.PaletteFilter_InitializeWhiteFilter();
                self.Overworld_LoadGFXAndScreenSize();
                self.increment_subsubmodule();
            }
            1 => {
                self.increment_subsubmodule();
                self.set_hdma_enable_mask(0xc0);
                self.MirrorWarp_BuildWavingHDMATable();
            }
            2 => self.MirrorWarp_BuildWavingHDMATable(),
            3 => self.MirrorWarp_BuildDewavingHDMATable(),
            _ => self.MirrorWarp_FinalizeAndLoadDestination(),
        }
    }

    fn set_small_overworld_mirror_map_position(&mut self) {
        self.set_overworld_map16_src_off(0x0390);
        self.set_overworld_map16_y_unit((0x0390u16.wrapping_sub(0x0400) & 0x0f80) >> 7);
        self.set_overworld_map16_dst_off((0x0390u16.wrapping_sub(0x0010) & 0x003e) >> 1);
    }

    pub(super) fn Overworld_DrawScreenAtCurrentMirrorPosition(&mut self) {
        let bak_src_off = self.overworld_map16_src_off();
        let bak_dst_off = self.overworld_map16_dst_off();
        let bak_y_unit = self.overworld_map16_y_unit();
        if self.overworld_map_is_small() {
            self.set_small_overworld_mirror_map_position();
        }
        self.Overworld_DrawQuadrantsAndOverlays();
        if self.frame_state().submodule == 44 {
            self.MirrorBonk_RecoverChangedTiles();
        }
        self.set_overworld_map16_y_unit(bak_y_unit);
        self.set_overworld_map16_dst_off(bak_dst_off);
        self.set_overworld_map16_src_off(bak_src_off);
    }

    pub(super) fn MirrorWarp_LoadSpritesAndColors(&mut self) {
        self.player_state_view_mut().set_blink_countdown(0x90);
        let bak_src_off = self.overworld_map16_src_off();
        let bak_dst_off = self.overworld_map16_dst_off();
        let bak_y_unit = self.overworld_map16_y_unit();
        if self.overworld_map_is_small() {
            self.set_small_overworld_mirror_map_position();
        }
        self.Map16ToMap8(0x2000, 0);
        self.set_overworld_map16_y_unit(bak_y_unit);
        self.set_overworld_map16_dst_off(bak_dst_off);
        self.set_overworld_map16_src_off(bak_src_off);

        self.OverworldLoadScreensPaletteSet();
        let sc = self.world_location_state().overworld_screen_index() as usize;
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(sc as u8),
            self.overworld_config_table_view().sprite_palette(sc),
        );
        self.Palette_SpecialOw();
        self.Overworld_SetFixedColAndScroll();
        if self.world_location_state().overworld_screen_index() == 0x1b
            || self.world_location_state().overworld_screen_index() == 0x5b
        {
            self.set_sub_screen_layers(1);
        }
        for i in 0..16 * 6 {
            self.palette_buffer_view_mut()
                .set_main_color(32 + i, 0x7fff);
        }
        self.palette_buffer_view_mut().set_main_color(0, 0x7fff);
        if u16::from(self.world_location_state().overworld_screen_index()) == 0x5b {
            self.palette_buffer_view_mut().set_main_color(0, 0);
            self.palette_buffer_view_mut().set_main_color(32, 0);
        }
        self.sprite_reset_all();
        self.sprite_reload_all_overworld();
        self.link_item_reset_from_overworld_things();
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        self.player_state_view_mut().set_handler_state(20);
        if self.world_location_state().overworld_screen_index() & 0x40 == 0 {
            self.sprite_initialize_mirror_portal();
        }
    }

    pub(super) fn Module09_2E_Whirlpool(&mut self) {
        self.display_nmi_view_mut()
            .increment_core_update_disable_flag();
        match self.frame_state().subsubmodule {
            0 => {
                self.system_signals_view_mut().set_sound_effect_1(0x34);
                self.system_signals_view_mut().set_ambient_sound_effect(5);
                self.world_state_view_mut().set_overworld_map_state(0);
                self.palette_filter_view_mut().set_countdown(0);
                self.increment_subsubmodule();
            }
            1 => self.PaletteFilter_WhirlpoolBlue(),
            2 => self.PaletteFilter_IsolateWhirlpoolBlue(),
            3 => {
                self.palette_filter_view_mut().set_fixed_color_blue(0x9f);
                self.palette_buffer_view_mut()
                    .clear_overworld_aux_or_main_offset();
                self.world_state_view_mut().set_hud_palette(0);
                self.FindPartnerWhirlpoolExit();
                self.dungeon_state_view_mut().set_draw_width_indicator(0);
                self.Overworld_LoadOverlays2();
                self.decrement_submodule();
                self.set_pending_nmi_subroutine(12);
                self.system_signals_view_mut().clear_cgram_update_flag();
                self.palette_filter_view_mut().set_fixed_color_blue(0x80);
                self.set_screen_brightness(0x0f);
                self.display_nmi_view_mut()
                    .increment_core_update_disable_flag();
                self.increment_subsubmodule();
            }
            4 | 6 => {
                self.set_pending_nmi_subroutine(13);
                self.display_nmi_view_mut()
                    .increment_core_update_disable_flag();
                self.increment_subsubmodule();
            }
            5 => {
                self.Overworld_LoadOverlayAndMap();
                self.set_pending_nmi_subroutine(12);
                self.set_screen_brightness(0x0f);
                self.display_nmi_view_mut()
                    .increment_core_update_disable_flag();
                self.increment_subsubmodule();
            }
            7 => {
                self.Module09_LoadAuxGFX();
                self.decrement_submodule();
                self.increment_subsubmodule();
            }
            8 => {
                self.Overworld_FinishTransGfx();
                self.set_screen_brightness(0x0f);
                self.display_nmi_view_mut()
                    .increment_core_update_disable_flag();
                self.decrement_submodule();
                self.increment_subsubmodule();
            }
            9 => {
                self.palette_buffer_view_mut()
                    .clear_overworld_aux_or_main_offset();
                self.Palette_Load_SpriteMain();
                self.Palette_Load_SpriteEnvironment();
                self.Palette_Load_Sp0L();
                self.Palette_Load_HUD();
                self.Palette_Load_OWBGMain();
                let sc = self.world_location_state().overworld_screen_index() as usize;
                self.Overworld_LoadPalettes(
                    self.GetOverworldBgPalette(sc as u8),
                    self.overworld_config_table_view().sprite_palette(sc),
                );
                self.Palette_SetOwBgColor();
                self.Overworld_SetFixedColAndScroll();
                self.LoadNewSpriteGFXSet();
                self.palette_filter_view_mut().set_fixed_color_blue(0x80);
                self.set_screen_brightness(0x0f);
                self.display_nmi_view_mut()
                    .increment_core_update_disable_flag();
                self.increment_subsubmodule();
            }
            10 => {
                self.PaletteFilter_WhirlpoolRestoreRedGreen();
                if self.palette_filter_view().countdown() != 0 {
                    self.PaletteFilter_WhirlpoolRestoreRedGreen();
                }
            }
            11 => {
                self.Graphics_IncrementalVRAMUpload();
                self.PaletteFilter_WhirlpoolRestoreBlue();
            }
            12 => {
                self.player_state_view_mut().set_blink_countdown(144);
                self.ReloadPreviouslyLoadedSheets();
                self.set_hdma_enable_mask(0x80);
                let music = self
                    .overworld_config_table_view()
                    .music(self.world_location_state().overworld_screen_index() as usize);
                self.system_signals_view_mut()
                    .set_ambient_sound_effect(music >> 4);
                let music_control = if self.save_progress_view().dark_world_state() != 0 {
                    9
                } else {
                    2
                };
                self.system_signals_view_mut()
                    .set_music_control(music_control);
                self.set_submodule(0);
                self.set_subsubmodule(0);
                self.world_state_view_mut().set_overworld_map_state(0);
                self.clear_core_update_disable_flag();
            }
            _ => {}
        }
    }

    pub(super) fn Spotlight_ConfigureTableAndControl(&mut self) {
        self.IrisSpotlight_ConfigureTable();
        self.world_state_view_mut().set_nmi_thread_active(0);
        self.clear_pending_polyhedral_update();
        if self.frame_state().submodule != 0 {
            return;
        }
        if self.frame_state().main_module == 6 {
            self.player_state_view_mut().restore_y_from_overworld_exit();
        }
        self.OpenSpotlight_Next2();
    }

    pub(super) fn OpenSpotlight_Next2(&mut self) {
        if self.frame_state().main_module != 9 {
            self.EnableForceBlank();
            self.link_item_reset_from_overworld_things();
        }

        if self.frame_state().main_module == 9 {
            if self.world_location_state().dungeon_room != 0x20 {
                let submodule = if self.player_state_view().has_facing() {
                    0x0a
                } else {
                    0x0b
                };
                self.set_submodule(submodule);
            }
            self.world_state_view_mut().set_ow_countdown_transition(16);
            let big_rock = self.dungeon_state_view().big_rock_starting_address();
            if (self.world_state_view_mut().ow_entrance_value() as u8 | big_rock as u8) != 0
                && big_rock & 0xff00 != 0
            {
                self.dungeon_state_view_mut()
                    .set_door_open_counter_low(if big_rock & 0x8000 != 0 { 0x18 } else { 0 });
                self.dungeon_state_view_mut()
                    .set_big_rock_starting_address(big_rock & 0x7fff);
                self.world_state_view_mut().set_door_animation_step(0);
                self.set_submodule(9);
                self.set_subsubmodule(0);
                self.system_signals_view_mut().set_sound_effect_2(21);
            }
        }

        self.set_bg12_window_selection(0);
        self.set_bg34_window_selection(0);
        self.set_object_color_window_selection(0);
        self.set_main_screen_window_layers(0);
        self.set_sub_screen_window_layers(0);
        self.player_state_view_mut().clear_force_hold_sword_up();

        match self.world_location_state().overworld_screen_index() {
            3 | 5 | 7 => {
                self.palette_filter_view_mut().set_fixed_color_red(0x26);
                self.palette_filter_view_mut().set_fixed_color_green(0x4c);
                self.palette_filter_view_mut().set_fixed_color_blue(0x8c);
            }
            0x43 | 0x45 | 0x47 => {
                self.palette_filter_view_mut().set_fixed_color_red(0x26);
                self.palette_filter_view_mut().set_fixed_color_green(0x4a);
                self.palette_filter_view_mut().set_fixed_color_blue(0x87);
            }
            _ => {}
        }
    }

    pub(super) fn Module10_SpotlightOpen(&mut self) {
        self.sprite_main();
        if self.frame_state().submodule == 0 {
            self.Module10_00_OpenIris();
        } else {
            self.Spotlight_ConfigureTableAndControl();
        }
        self.link_oam_main();
    }

    pub(super) fn Module0F_SpotlightClose(&mut self) {
        const SCREEN_TRANSITION_DIRECTION_BITS: [u8; 4] = [8, 4, 2, 1];

        self.sprite_main();
        if self.frame_state().submodule == 0 {
            self.Dungeon_PrepExitWithSpotlight();
        } else {
            self.Spotlight_ConfigureTableAndControl();
        }

        if self.world_location_state().is_outdoors() {
            if self.world_location_state().overworld_screen_index() == 0x0f {
                self.player_state_view_mut()
                    .set_water_ripple_or_grass_state(1);
            }
            self.player_state_view_mut().set_speed_setting(6);
            self.link_handle_velocity();
            self.player_state_view_mut().clear_movement_velocity();
        }

        let mut i = self.player_state_view().facing_index();
        if self.world_location_state().is_outdoors() {
            i = if self.world_state_view().which_entrance() == 0x43 {
                1
            } else {
                0
            };
        }

        let dir = SCREEN_TRANSITION_DIRECTION_BITS[i];
        self.player_state_view_mut()
            .set_direction_and_last_direction(dir);
        self.link_handle_moving_animation_full_long_entry();
        self.link_oam_main();
    }

    pub(super) fn Dungeon_PrepExitWithSpotlight(&mut self) {
        self.world_state_view_mut().set_nmi_thread_active(0);
        self.clear_pending_polyhedral_update();
        if self.world_location_state().is_outdoors() {
            self.Ancilla_TerminateWaterfallSplashes();
            self.player_state_view_mut()
                .store_overworld_exit_y_from_current();
        }

        let mut m =
            self.zelda_get_entrance_music_track(self.world_state_view().which_entrance() as i32);
        if m != 3 || {
            m = self.save_progress_view().progress_indicator();
            m >= 2
        } {
            if m != 0xf2 {
                m = 0xf1;
            } else if self.system_signals_view().current_music_control() == 12 {
                m = 7;
            }
            self.system_signals_view_mut().set_music_control(m);
        }

        self.world_state_view_mut().clear_hud_floor_changed_timer();
        self.hud_floor_indicator();
        self.system_signals_view_mut().increment_hud_update_flag();
        self.IrisSpotlight_close();
        self.increment_submodule();
    }

    pub(super) fn SetTargetOverworldWarpToPyramid(&mut self) {
        if self.frame_state().main_module != 21 {
            return;
        }
        self.LoadOverworldFromDungeon();
        self.DecompressAnimatedOverworldTiles(0x5a);
        self.ResetAncillaAndCutscene();
    }

    pub(super) fn ResetAncillaAndCutscene(&mut self) {
        self.ancilla_terminate_select_interactives(0);
        self.player_state_view_mut()
            .clear_sprite_damage_disable_timer();
        self.player_state_view_mut().clear_button_b_frames();
        self.player_state_view_mut().set_button_mask_b_y(0);
        self.player_state_view_mut().clear_force_hold_sword_up();
        self.player_state_view_mut().clear_immobilized();
    }

    pub(super) fn ConditionalMosaicControl(&mut self) {
        if self.palette_filter_view().countdown() & 1 != 0 {
            self.increment_mosaic_level_by(0x10);
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(7);
    }

    pub(super) fn Overworld_ResetMosaic_alwaysIncrease(&mut self) {
        self.increment_mosaic_level_by(0x10);
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(7);
    }

    pub(super) fn FluteMenu_LoadTransport(&mut self) {
        self.memorized_tile_view_mut().clear_count();
        let k = self.world_state_view().birdtravel_status() as usize;
        let bird_travel_index_x2 = self.world_state_view().birdtravel_status_word() << 1;
        self.world_state_view_mut()
            .set_birdtravel_status_word(bird_travel_index_x2);
        self.Overworld_LoadBirdTravelPos(k);
    }

    pub(super) fn Overworld_LoadBirdTravelPos(&mut self, k: usize) {
        let screen_index = self
            .asset_raw(113)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_ScreenIndex asset")
            .to_vec();
        let map16_src = self
            .asset_raw(114)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_Map16LoadSrcOff asset")
            .to_vec();
        let scroll_x_table = self
            .asset_raw(115)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_ScrollX asset")
            .to_vec();
        let scroll_y_table = self
            .asset_raw(116)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_ScrollY asset")
            .to_vec();
        let link_x_table = self
            .asset_raw(117)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_LinkXCoord asset")
            .to_vec();
        let link_y_table = self
            .asset_raw(118)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_LinkYCoord asset")
            .to_vec();
        let camera_x_table = self
            .asset_raw(119)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_CameraXScroll asset")
            .to_vec();
        let camera_y_table = self
            .asset_raw(120)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_CameraYScroll asset")
            .to_vec();
        let scroll_up_seed_table = self
            .asset_raw(121)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_Unk1 asset")
            .to_vec();
        let scroll_left_seed_table = self
            .asset_raw(122)
            .expect("Overworld_LoadBirdTravelPos missing kBirdTravel_Unk3 asset")
            .to_vec();

        let scroll_y = read_word_from_slice(&scroll_y_table, k * 2);
        let scroll_x = read_word_from_slice(&scroll_x_table, k * 2);
        self.world_state_view_mut().set_bg1_y(scroll_y);
        self.world_state_view_mut().set_bg2_y(scroll_y);
        self.ppu_scroll_copy_view_mut().set_bg1_v_copy(scroll_y);
        self.ppu_scroll_copy_view_mut().set_bg2_v_copy(scroll_y);
        self.world_state_view_mut().set_bg1_x(scroll_x);
        self.world_state_view_mut().set_bg2_x(scroll_x);
        self.ppu_scroll_copy_view_mut().set_bg1_h_copy(scroll_x);
        self.ppu_scroll_copy_view_mut().set_bg2_h_copy(scroll_x);

        let link_y = read_word_from_slice(&link_y_table, k * 2);
        let link_x = read_word_from_slice(&link_x_table, k * 2);
        self.player_state_view_mut().set_y(link_y);
        self.player_state_view_mut().set_x(link_x);

        let scroll_up_seed = scroll_up_seed_table[k] as i8 as i16 as u16;
        let scroll_left_seed = scroll_left_seed_table[k] as i8 as i16 as u16;
        self.world_state_view_mut()
            .set_overworld_scroll_up_counter(scroll_up_seed);
        self.world_state_view_mut()
            .set_overworld_scroll_left_counter(scroll_left_seed);
        self.world_state_view_mut()
            .set_overworld_scroll_down_counter(scroll_up_seed.wrapping_neg());
        self.world_state_view_mut()
            .set_overworld_scroll_right_counter(scroll_left_seed.wrapping_neg());

        let screen = read_word_from_slice(&screen_index, k * 2);
        self.world_state_view_mut()
            .set_overworld_area_index_word(screen);
        self.set_overworld_screen_word(screen);

        let src = read_word_from_slice(&map16_src, k * 2);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_y_unit((src.wrapping_sub(0x400) & 0x0f80) >> 7);
        self.set_overworld_map16_dst_off((src.wrapping_sub(0x10) & 0x003e) >> 1);

        let camera_y = read_word_from_slice(&camera_y_table, k * 2);
        self.world_state_view_mut()
            .set_camera_y_coord_scroll_low(camera_y);
        self.world_state_view_mut()
            .set_camera_y_coord_scroll_hi(camera_y.wrapping_sub(2));
        let camera_x = read_word_from_slice(&camera_x_table, k * 2);
        self.world_state_view_mut()
            .set_camera_x_coord_scroll_low(camera_x);
        self.world_state_view_mut()
            .set_camera_x_coord_scroll_hi(camera_x.wrapping_sub(2));

        self.world_state_view_mut().set_ow_entrance_value(0);
        self.dungeon_state_view_mut()
            .set_big_rock_starting_address(0);
        self.Overworld_LoadNewScreenProperties();
        self.sprite_reset_all();
        self.sprite_reload_all_overworld();
        self.player_state_view_mut().clear_doorway_state();
        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
    }

    pub(super) fn FluteMenu_LoadSelectedScreenPalettes(&mut self) {
        self.OverworldLoadScreensPaletteSet();
        let sc = self.world_location_state().overworld_screen_index() as usize;
        let bg = self.GetOverworldBgPalette(sc as u8);
        let spr = self.overworld_config_table_view().sprite_palette(sc);
        self.Overworld_LoadPalettes(bg, spr);
        self.Palette_SetOwBgColor();
        self.Overworld_LoadPalettesInner();
    }

    pub(super) fn FindPartnerWhirlpoolExit(&mut self) {
        let screen = u16::from(self.world_location_state().overworld_screen_index());
        let whirlpool_areas = self
            .asset_raw(123)
            .expect("FindPartnerWhirlpoolExit missing kWhirlpoolAreas asset")
            .to_vec();
        let count = whirlpool_areas.len() / 2;
        for j in (0..count).rev() {
            if read_word_from_slice(&whirlpool_areas, j * 2) == screen {
                self.memorized_tile_view_mut().clear_count();
                self.Overworld_LoadBirdTravelPos(j + 9);
                break;
            }
        }
    }

    pub(super) fn Overworld_LoadNewScreenProperties(&mut self) {
        self.tile_detect_position_view_mut()
            .set_location_calc_mask(!7u16);
        self.Overworld_LoadGFXAndScreenSize();
        self.overworld_screen_size_view_mut()
            .set_right_bottom_bound_low(0xe4);
        self.overworld_screen_size_view_mut().clear_big_area_high();
        let big = self.overworld_screen_size_view().is_big_area_word() != 0;
        let area = (self.world_location_state().overworld_screen_index() & 0x3f) as usize;
        self.Overworld_SetCameraBoundaries(if big { 1 } else { 0 }, area as i32);
        self.player_state_view_mut().set_quadrants(0, 2);
        self.world_state_view_mut()
            .set_fullsize_overworld_quadrants();
        self.player_state_view_mut().disable_oam_offsets();
        {
            let mut player = self.player_state_view_mut();
            player.reset_direction_masks();
            player.set_z_low(0xff);
            player.set_actual_z_velocity(0xff);
        }
    }

    pub(super) fn LoadCachedEntranceProperties(&mut self) {
        self.world_state_view_mut().restore_exit_area_index();
        self.world_state_view_mut().restore_exit_layer_masks();

        self.ppu_scroll_copy_view_mut()
            .restore_exit_bg2_scroll_to_all_layers();

        self.player_state_view_mut()
            .restore_position_from_overworld_exit();
        if self.world_location_state().dungeon_room < 0x0124 {
            let link_y = self.player_state_view().y().wrapping_sub(0x10);
            self.player_state_view_mut().set_y(link_y);
        }
        self.player_state_view_mut().set_facing(2);
        if self.world_state_view().ow_entrance_value() == 0xffff {
            let link_y = self.player_state_view().y().wrapping_add(0x20);
            self.player_state_view_mut().set_y(link_y);
            self.player_state_view_mut().set_facing(0);
        }

        let overworld_screen = self.world_state_view().exit_screen_index();
        self.set_overworld_screen_word(overworld_screen);
        self.set_overworld_map16_src_off(self.overworld_exit_map16_src_off());
        let src = self.overworld_map16_src_off();
        self.set_overworld_map16_y_unit((src.wrapping_sub(0x400) & 0x0f80) >> 7);
        self.set_overworld_map16_dst_off((src.wrapping_sub(0x10) & 0x003e) >> 1);

        self.world_state_view_mut().restore_exit_camera_scroll();

        self.room_bounds_view_mut()
            .copy_y_bounds_from(OW_SCROLL_VARS0_EXIT_OVERWORLD, 8);
        copy_le_u16(
            &mut self.ram,
            UP_DOWN_SCROLL_TARGET,
            UP_DOWN_SCROLL_TARGET_EXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            UP_DOWN_SCROLL_TARGET_END,
            UP_DOWN_SCROLL_TARGET_END_EXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            LEFT_RIGHT_SCROLL_TARGET,
            LEFT_RIGHT_SCROLL_TARGET_EXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            LEFT_RIGHT_SCROLL_TARGET_END,
            LEFT_RIGHT_SCROLL_TARGET_END_EXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            OVERWORLD_SCROLL_UP_COUNTER_OVERWORLD,
            OVERWORLD_SCROLL_UP_COUNTER_EXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            OVERWORLD_SCROLL_DOWN_COUNTER_OVERWORLD,
            OVERWORLD_SCROLL_DOWN_COUNTER_EXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            OVERWORLD_SCROLL_LEFT_COUNTER_OVERWORLD,
            OVERWORLD_SCROLL_LEFT_COUNTER_EXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            OVERWORLD_SCROLL_RIGHT_COUNTER_OVERWORLD,
            OVERWORLD_SCROLL_RIGHT_COUNTER_EXIT_OVERWORLD,
        );
        self.world_state_view_mut().restore_exit_tile_themes();
        self.sprite_system_view_mut().restore_exit_graphics_index();
    }

    pub(super) fn LoadOverworldFromSpecialOverworld(&mut self) {
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some() {
            println!(
                "spexit-restore-before frame={} area=0x{:04x} screen=0x{:04x} x=0x{:04x} y=0x{:04x} bg=0x{:04x}/0x{:04x} src=0x{:04x} cam=0x{:04x}/0x{:04x} bounds={:04x},{:04x},{:04x},{:04x}",
                self.frame_state().frame_counter,
                self.world_state_view().spexit_area_index(),
                self.world_state_view().spexit_screen_index(),
                self.special_exit_position_view().x(),
                self.special_exit_position_view().y(),
                self.ppu_scroll_copy_view().special_exit_bg2_h_copy2(),
                self.ppu_scroll_copy_view().special_exit_bg2_v_copy2(),
                self.overworld_spexit_map16_src_off(),
                self.world_state_view().spexit_camera_x_scroll_low(),
                self.world_state_view().spexit_camera_y_scroll_low(),
                self.world_state_view().spexit_room_bound_x_start(),
                self.world_state_view().spexit_room_bound_x_end(),
                self.world_state_view().spexit_room_bound_y_start(),
                self.world_state_view().spexit_room_bound_y_end(),
            );
        }
        self.memorized_tile_view_mut().clear_count();
        self.world_state_view_mut().restore_spexit_area_index();
        self.world_state_view_mut().restore_spexit_layer_masks();

        self.ppu_scroll_copy_view_mut()
            .restore_special_exit_bg2_scroll_to_all_layers();

        self.special_exit_position_view_mut()
            .restore_player_position();
        let overworld_screen = self.world_state_view().spexit_screen_index();
        self.set_overworld_screen_word(overworld_screen);
        self.set_overworld_map16_src_off(self.overworld_spexit_map16_src_off());
        let src = self.overworld_map16_src_off();
        self.set_overworld_map16_y_unit((src.wrapping_sub(0x400) & 0x0f80) >> 7);
        self.set_overworld_map16_dst_off((src.wrapping_sub(0x10) & 0x003e) >> 1);

        self.world_state_view_mut()
            .restore_special_exit_camera_scroll();

        self.room_bounds_view_mut()
            .copy_packed_bound_from(0, SPECIAL_EXIT_ROOM_BOUNDS_Y_START);
        self.room_bounds_view_mut()
            .copy_packed_bound_from(1, SPECIAL_EXIT_ROOM_BOUNDS_Y_END);
        self.room_bounds_view_mut()
            .copy_packed_bound_from(2, SPECIAL_EXIT_ROOM_BOUNDS_X_START);
        self.room_bounds_view_mut()
            .copy_packed_bound_from(3, SPECIAL_EXIT_ROOM_BOUNDS_X_END);
        copy_le_u16(
            &mut self.ram,
            UP_DOWN_SCROLL_TARGET,
            UP_DOWN_SCROLL_TARGET_SPEXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            UP_DOWN_SCROLL_TARGET_END,
            UP_DOWN_SCROLL_TARGET_END_SPEXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            LEFT_RIGHT_SCROLL_TARGET,
            LEFT_RIGHT_SCROLL_TARGET_SPEXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            LEFT_RIGHT_SCROLL_TARGET_END,
            LEFT_RIGHT_SCROLL_TARGET_END_SPEXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            OVERWORLD_SCROLL_UP_COUNTER_OVERWORLD,
            OVERWORLD_SCROLL_UP_COUNTER_SPEXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            OVERWORLD_SCROLL_DOWN_COUNTER_OVERWORLD,
            OVERWORLD_SCROLL_DOWN_COUNTER_SPEXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            OVERWORLD_SCROLL_LEFT_COUNTER_OVERWORLD,
            OVERWORLD_SCROLL_LEFT_COUNTER_SPEXIT_OVERWORLD,
        );
        copy_le_u16(
            &mut self.ram,
            OVERWORLD_SCROLL_RIGHT_COUNTER_OVERWORLD,
            OVERWORLD_SCROLL_RIGHT_COUNTER_SPEXIT_OVERWORLD,
        );
        self.world_state_view_mut().restore_spexit_tile_themes();
        self.sprite_system_view_mut()
            .restore_special_exit_graphics_index();

        let sc = self.world_location_state().overworld_screen_index() as usize;
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(sc as u8),
            self.overworld_config_table_view().sprite_palette(sc),
        );
        self.Palette_SpecialOw();
        self.player_state_view_mut().set_quadrants(0, 2);
        self.world_state_view_mut()
            .set_fullsize_overworld_quadrants();
        self.player_state_view_mut().disable_oam_offsets();
        {
            let mut player = self.player_state_view_mut();
            player.reset_direction_masks();
            player.set_z_low(0xff);
            player.set_actual_z_velocity(0xff);
        }
        self.link_reset_swimming_state();
        self.Overworld_LoadGFXAndScreenSize();
        self.overworld_screen_size_view_mut()
            .set_right_bottom_bound_low(228);
        self.overworld_screen_size_view_mut().clear_big_area_high();
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some() {
            println!(
                "spexit-restore-after frame={} area=0x{:04x} screen=0x{:04x} x=0x{:04x} y=0x{:04x} bg=0x{:04x}/0x{:04x} base=0x{:04x}/0x{:04x} mask=0x{:04x}/0x{:04x} room=0x{:04x} main={} sub={}",
                self.frame_state().frame_counter,
                self.world_state_view().overworld_area_index(),
                u16::from(self.world_location_state().overworld_screen_index()),
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.world_state_view().bg2_x(),
                self.world_state_view().bg2_y(),
                self.world_state_view().overworld_offset_base_x(),
                self.world_state_view().overworld_offset_base_y(),
                self.world_state_view().overworld_offset_mask_x(),
                self.world_state_view().overworld_offset_mask_y(),
                self.world_location_state().dungeon_room,
                self.frame_state().main_module,
                self.frame_state().submodule,
            );
        }
    }

    pub(super) fn Overworld_LoadGFXAndScreenSize(&mut self) {
        let i = self.world_location_state().overworld_screen_index() as usize;
        self.reset_incremental_vram_upload_counter();
        let graphics_index = self.overworld_config_table_view().sprite_graphics(i);
        self.sprite_system_view_mut()
            .set_graphics_index(graphics_index);
        let aux_tile_theme_index = self.asset_u8(108, i);
        self.world_state_view_mut()
            .set_aux_tile_theme_index(aux_tile_theme_index);
        self.overworld_screen_size_view_mut().backup_big_area_low();

        let small = self.asset_u8(107, i & 0x3f) != 0;
        self.overworld_screen_size_view_mut()
            .set_big_area_low(if small { 0 } else { 0x20 });
        self.overworld_screen_size_view_mut()
            .set_right_bottom_bound_high(if small { 1 } else { 3 });
        let is_dark_world_screen = self.world_location_state().overworld_screen_index() & 0x40 != 0;
        let main_tile_theme_index = if is_dark_world_screen { 0x21 } else { 0x20 };
        self.world_state_view_mut()
            .set_main_tile_theme_index(main_tile_theme_index);
        let packs = 6 + if is_dark_world_screen { 8 } else { 0 };
        self.world_state_view_mut()
            .set_misc_sprites_graphics_index(VARIOUS_PACKS_OVERWORLD[packs]);

        let j = (self.world_location_state().overworld_screen_index() & 0xbf) as usize;
        self.world_state_view_mut()
            .set_overworld_offset_base_y(overworld_offset_base_y_c_index(j));
        self.world_state_view_mut()
            .set_overworld_offset_base_x(overworld_offset_base_x_c_index(j) >> 3);
        let mask = if self.overworld_screen_size_view().is_big_area_word() != 0 {
            0x03f0
        } else {
            0x01f0
        };
        self.world_state_view_mut()
            .set_overworld_offset_mask_y(mask);
        self.world_state_view_mut()
            .set_overworld_offset_mask_x(mask >> 3);
    }

    pub(super) fn Overworld_SetCameraBoundaries(&mut self, big: i32, area: i32) {
        assert!(
            (0..64).contains(&area),
            "Overworld_SetCameraBoundaries area out of range: {area}"
        );
        assert!(
            (0..=1).contains(&big),
            "Overworld_SetCameraBoundaries big out of range: {big}"
        );
        let area = area as usize;
        let big = big as usize;
        let ystart = OVERWORLD_AREA_BASE_Y[area];
        let xstart = OVERWORLD_AREA_BASE_X[area];
        self.room_bounds_view_mut().set_packed_bounds(
            ystart,
            ystart.wrapping_add(OVERWORLD_AREA_HEIGHTS_BY_SIZE[big]),
            xstart,
            xstart.wrapping_add(OVERWORLD_AREA_WIDTHS_BY_SIZE[big]),
        );
        let up_down = OVERWORLD_VERTICAL_SCROLL_TARGETS[area];
        self.world_state_view_mut()
            .set_up_down_scroll_target(up_down);
        self.world_state_view_mut().set_up_down_scroll_target_end(
            up_down.wrapping_add(OVERWORLD_VERTICAL_SCROLL_SPANS_BY_SIZE[big]),
        );
        let left_right = OVERWORLD_HORIZONTAL_SCROLL_TARGETS[area];
        self.world_state_view_mut()
            .set_left_right_scroll_target(left_right);
        self.world_state_view_mut()
            .set_left_right_scroll_target_end(
                left_right.wrapping_add(OVERWORLD_HORIZONTAL_SCROLL_SPANS_BY_SIZE[big]),
            );
    }

    fn overworld_map_is_small(&self) -> bool {
        self.asset_u8(
            107,
            self.world_location_state().overworld_screen_index() as usize,
        ) != 0
    }

    fn write_overworld_vram_word(&mut self, word_index: usize, value: u16) {
        self.vram_upload_data_view_mut()
            .write_overworld_vram_word(word_index, value);
    }

    fn overworld_bg2_word(&self, word_index: usize) -> u16 {
        self.dungeon_state_view().bg2_tile(word_index)
    }

    fn overworld_map16_to_map8_word(&self, map8: &[u8], map16: u16, quarter: usize) -> u16 {
        read_word_from_slice(map8, ((map16 as usize) * 4 + quarter) * 2)
    }

    fn store_overworld_map16_load_state(&mut self, state: OverworldMap16LoadState) {
        self.set_overworld_map16_load_state(state);
        self.world_state_view_mut()
            .set_map16_load_src(state.src_off);
        self.world_state_view_mut()
            .set_map16_load_dst(state.dst_off);
        self.world_state_view_mut()
            .set_map16_load_y_unit(state.y_unit);
    }

    fn store_overworld_prev_map16_load_state(&mut self, state: OverworldMap16LoadState) {
        self.set_overworld_prev_map16_load_state(state);
        self.world_state_view_mut().set_prev_map16_load_state(
            state.src_off,
            state.dst_off,
            state.y_unit,
        );
    }

    fn store_overworld_spexit_map16_src_off(&mut self, src_off: u16) {
        self.set_overworld_spexit_map16_src_off(src_off);
        self.world_state_view_mut()
            .set_spexit_map16_src_off(src_off);
    }

    fn store_overworld_exit_map16_src_off(&mut self, src_off: u16) {
        self.set_overworld_exit_map16_src_off(src_off);
        self.world_state_view_mut().set_exit_map16_src_off(src_off);
    }

    fn store_small_overworld_map16_scroll_backup(
        &mut self,
        state: SmallOverworldMap16ScrollBackupState,
    ) {
        self.set_small_overworld_map16_scroll_backup_state(state);
        self.world_state_view_mut().set_small_ow_scroll_backup(
            state.src_off,
            state.dst_off,
            state.y_unit,
        );
    }

    fn overworld_map16_src_off(&self) -> u16 {
        self.overworld_map16_load_state().src_off
    }

    fn overworld_map16_dst_off(&self) -> u16 {
        self.overworld_map16_load_state().dst_off
    }

    fn overworld_map16_y_unit(&self) -> u16 {
        self.overworld_map16_load_state().y_unit
    }

    fn set_overworld_map16_src_off(&mut self, src_off: u16) {
        let mut state = self.overworld_map16_load_state();
        state.src_off = src_off;
        self.store_overworld_map16_load_state(state);
    }

    fn set_overworld_map16_dst_off(&mut self, dst_off: u16) {
        let mut state = self.overworld_map16_load_state();
        state.dst_off = dst_off;
        self.store_overworld_map16_load_state(state);
    }

    fn set_overworld_map16_y_unit(&mut self, y_unit: u16) {
        let mut state = self.overworld_map16_load_state();
        state.y_unit = y_unit;
        self.store_overworld_map16_load_state(state);
    }

    pub(super) fn BufferAndBuildMap16Stripes_X(&mut self, mut dst: usize) -> usize {
        let strip = OVERWORLD_MAP16_STRIP_BACKTRACK_BY_DIRECTION
            [((self.world_state_view().screen_transition_direction_bits() >> 1) & 1) as usize];
        let mut pos = self.overworld_map16_src_off().wrapping_sub(strip);
        let mut y_unit_index = self.overworld_map16_y_unit() as usize & 0x1f;
        for _ in 0..32 {
            let tile = if pos >= 0x2000 {
                0
            } else {
                self.overworld_bg2_word((pos >> 1) as usize)
            };
            self.world_state_view_mut()
                .set_dung_replacement_tile_state(y_unit_index, tile);
            y_unit_index = (y_unit_index + 1) & 0x1f;
            pos = pos.wrapping_add(0x80);
        }

        let map8 = self.GetMap16toMap8Table();
        let mut r0 = 0u16;
        let mut dst_unit = self.overworld_map16_dst_off();
        if dst_unit >= 0x10 {
            dst_unit &= 0x0f;
            r0 = 0x400;
        }
        r0 = r0.wrapping_add(dst_unit.wrapping_mul(2));

        let mut tmp = 0usize;
        for _ in 0..2 {
            self.write_overworld_vram_word(dst, r0);
            self.write_overworld_vram_word(dst + 33, r0.wrapping_add(1));
            dst += 1;
            for _ in 0..16 {
                let k = self.world_state_view().dung_replacement_tile_state(tmp);
                tmp += 1;
                let s0 = self.overworld_map16_to_map8_word(&map8, k, 0);
                let s1 = self.overworld_map16_to_map8_word(&map8, k, 1);
                let s2 = self.overworld_map16_to_map8_word(&map8, k, 2);
                let s3 = self.overworld_map16_to_map8_word(&map8, k, 3);
                self.write_overworld_vram_word(dst, s0);
                self.write_overworld_vram_word(dst + 33, s1);
                self.write_overworld_vram_word(dst + 1, s2);
                self.write_overworld_vram_word(dst + 34, s3);
                dst += 2;
            }
            dst += 33;
            r0 = r0.wrapping_add(0x800);
        }
        dst
    }

    pub(super) fn BufferAndBuildMap16Stripes_Y(&mut self, mut dst: usize) -> usize {
        let strip_index =
            1 + ((self.world_state_view().screen_transition_direction_bits() >> 2) & 1) as usize;
        let mut pos = self
            .overworld_map16_src_off()
            .wrapping_sub(OVERWORLD_MAP16_STRIP_BACKTRACK_BY_DIRECTION[strip_index]);
        let mut dst_unit_index = self.overworld_map16_dst_off() as usize & 0x1f;
        for _ in 0..32 {
            let tile = if pos >= 0x2000 {
                0
            } else {
                self.overworld_bg2_word((pos >> 1) as usize)
            };
            self.world_state_view_mut()
                .set_dung_replacement_tile_state(dst_unit_index, tile);
            pos = pos.wrapping_add(2);
            dst_unit_index = (dst_unit_index + 1) & 0x1f;
        }

        let map8 = self.GetMap16toMap8Table();
        let mut r0 = 0u16;
        let mut y_unit = self.overworld_map16_y_unit();
        if y_unit >= 0x10 {
            y_unit &= 0x0f;
            r0 = 0x800;
        }
        r0 = r0.wrapping_add(y_unit.wrapping_mul(64));

        let mut tmp = 0usize;
        for _ in 0..2 {
            self.write_overworld_vram_word(dst, r0);
            dst += 1;
            for _ in 0..16 {
                let k = self.world_state_view().dung_replacement_tile_state(tmp);
                tmp += 1;
                let s0 = self.overworld_map16_to_map8_word(&map8, k, 0);
                let s1 = self.overworld_map16_to_map8_word(&map8, k, 1);
                let s2 = self.overworld_map16_to_map8_word(&map8, k, 2);
                let s3 = self.overworld_map16_to_map8_word(&map8, k, 3);
                self.write_overworld_vram_word(dst, s0);
                self.write_overworld_vram_word(dst + 32, s2);
                self.write_overworld_vram_word(dst + 1, s1);
                self.write_overworld_vram_word(dst + 33, s3);
                dst += 2;
            }
            dst += 32;
            r0 = r0.wrapping_add(0x400);
        }
        dst
    }

    pub(super) fn BuildFullStripeDuringTransition_North(&mut self, dst: usize) -> usize {
        self.write_overworld_vram_word(dst, 0x0080);
        let dst = self.BufferAndBuildMap16Stripes_Y(dst + 1);
        let src = self.overworld_map16_src_off().wrapping_sub(0x80);
        self.set_overworld_map16_src_off(src);
        let y_unit = self.overworld_map16_y_unit().wrapping_sub(1) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        dst
    }

    pub(super) fn BuildFullStripeDuringTransition_South(&mut self, dst: usize) -> usize {
        self.write_overworld_vram_word(dst, 0x0080);
        let dst = self.BufferAndBuildMap16Stripes_Y(dst + 1);
        let src = self.overworld_map16_src_off().wrapping_add(0x80);
        self.set_overworld_map16_src_off(src);
        let y_unit = self.overworld_map16_y_unit().wrapping_add(1) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        dst
    }

    pub(super) fn BuildFullStripeDuringTransition_West(&mut self, dst: usize) -> usize {
        self.write_overworld_vram_word(dst, 0x8040);
        let dst = self.BufferAndBuildMap16Stripes_X(dst + 1);
        let src = self.overworld_map16_src_off().wrapping_sub(2);
        self.set_overworld_map16_src_off(src);
        let off = self.overworld_map16_dst_off().wrapping_sub(1) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        dst
    }

    pub(super) fn BuildFullStripeDuringTransition_East(&mut self, dst: usize) -> usize {
        self.write_overworld_vram_word(dst, 0x8040);
        let dst = self.BufferAndBuildMap16Stripes_X(dst + 1);
        let src = self.overworld_map16_src_off().wrapping_add(2);
        self.set_overworld_map16_src_off(src);
        let off = self.overworld_map16_dst_off().wrapping_add(1) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        dst
    }

    pub(super) fn OverworldTransitionScrollAndLoadMap(&mut self) {
        let before = self.overworld_map16_src_off();
        let dst = match self.world_state_view().screen_transition_direction_bits() {
            1 => self.BuildFullStripeDuringTransition_East(0),
            2 => self.BuildFullStripeDuringTransition_West(0),
            4 => self.BuildFullStripeDuringTransition_South(0),
            8 => self.BuildFullStripeDuringTransition_North(0),
            _ => {
                self.set_submodule(0);
                panic!(
                    "OverworldTransitionScrollAndLoadMap invalid direction {}",
                    self.world_state_view().screen_transition_direction_bits()
                );
            }
        };
        self.write_overworld_vram_word(dst, 0xffff);
        self.write_overworld_vram_word(dst + 1, 0xffff);
        if dst != 0 {
            self.set_pending_nmi_subroutine(3);
        }
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some()
            && matches!(
                u16::from(self.world_location_state().overworld_screen_index()),
                0 | 2
            )
        {
            println!(
                "owstripe-scroll frame={} screen=0x{:04x} dir=0x{:02x} before=0x{:04x} after=0x{:04x} yunit=0x{:04x} dst=0x{:04x} sub={} subsub={}",
                self.frame_state().frame_counter,
                u16::from(self.world_location_state().overworld_screen_index()),
                self.world_state_view().screen_transition_direction_bits(),
                before,
                self.overworld_map16_src_off(),
                self.overworld_map16_y_unit(),
                self.overworld_map16_dst_off(),
                self.frame_state().submodule,
                self.frame_state().subsubmodule,
            );
        }
    }

    pub(super) fn TriggerAndFinishMapLoadStripe_Y(&mut self, mut n: i32) {
        self.world_state_view_mut()
            .set_screen_transition_direction_bits(8);
        self.set_pending_nmi_subroutine(3);
        let mut dst = 0usize;
        self.write_overworld_vram_word(dst, 0x0080);
        dst += 1;
        while n != 0 {
            dst = self.BufferAndBuildMap16Stripes_Y(dst);
            let src = self.overworld_map16_src_off().wrapping_sub(0x80);
            self.set_overworld_map16_src_off(src);
            let y_unit = self.overworld_map16_y_unit().wrapping_sub(1) & 0x1f;
            self.set_overworld_map16_y_unit(y_unit);
            n -= 1;
        }
        self.write_overworld_vram_word(dst, 0xffff);
    }

    pub(super) fn TriggerAndFinishMapLoadStripe_X(&mut self, mut n: i32) {
        self.world_state_view_mut()
            .set_screen_transition_direction_bits(2);
        self.set_pending_nmi_subroutine(3);
        let mut dst = 0usize;
        self.write_overworld_vram_word(dst, 0x8040);
        dst += 1;
        while n != 0 {
            dst = self.BufferAndBuildMap16Stripes_X(dst);
            let src = self.overworld_map16_src_off().wrapping_sub(2);
            self.set_overworld_map16_src_off(src);
            let off = self.overworld_map16_dst_off().wrapping_sub(1) & 0x1f;
            self.set_overworld_map16_dst_off(off);
            n -= 1;
        }
        self.write_overworld_vram_word(dst, 0xffff);
    }

    pub(super) fn CreateInitialOWScreenView_Big_North(&mut self) {
        let src = self.overworld_map16_src_off().wrapping_add(0x380);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_y_unit(31);
        self.TriggerAndFinishMapLoadStripe_Y(7);
    }

    pub(super) fn CreateInitialOWScreenView_Big_South(&mut self) {
        let mut pos = self.overworld_map16_src_off();
        while pos >= 0x80 {
            pos = pos.wrapping_sub(0x80);
        }
        self.set_overworld_map16_src_off(pos.wrapping_add(0x780));
        self.set_overworld_map16_y_unit(7);
        self.TriggerAndFinishMapLoadStripe_Y(8);
        let y_unit = self.overworld_map16_y_unit().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        let src = self.overworld_map16_src_off().wrapping_sub(0x0b80);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn CreateInitialOWScreenView_Big_West(&mut self) {
        let src = self.overworld_map16_src_off().wrapping_add(14);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_dst_off(31);
        self.TriggerAndFinishMapLoadStripe_X(7);
    }

    pub(super) fn CreateInitialOWScreenView_Big_East(&mut self) {
        let src = self
            .overworld_map16_src_off()
            .wrapping_sub(0x60)
            .wrapping_add(0x1e);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_dst_off(7);
        self.TriggerAndFinishMapLoadStripe_X(8);
        let off = self.overworld_map16_dst_off().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        let src = self.overworld_map16_src_off().wrapping_sub(0x2e);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn CreateInitialOWScreenView_Small_North(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.overworld_map16_load_state();
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x700),
            dst_off: map16.dst_off,
            y_unit: 10,
        });
        self.set_overworld_map16_src_off(0x1390);
        self.set_overworld_map16_dst_off(0);
        self.set_overworld_map16_y_unit(31);
        self.TriggerAndFinishMapLoadStripe_Y(7);
    }

    pub(super) fn CreateInitialOWScreenView_Small_South(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.overworld_map16_load_state();
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src & 0xff,
            dst_off: map16.dst_off,
            y_unit: 24,
        });
        self.set_overworld_map16_src_off(0x0790);
        self.set_overworld_map16_dst_off(0);
        self.set_overworld_map16_y_unit(7);
        self.TriggerAndFinishMapLoadStripe_Y(8);
        let y_unit = self.overworld_map16_y_unit().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        let src = self.overworld_map16_src_off().wrapping_sub(0x0b80);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn CreateInitialOWScreenView_Small_West(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.overworld_map16_load_state();
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x20),
            dst_off: 8,
            y_unit: map16.y_unit,
        });
        self.set_overworld_map16_src_off(0x044e);
        self.set_overworld_map16_y_unit(0);
        self.set_overworld_map16_dst_off(31);
        self.TriggerAndFinishMapLoadStripe_X(7);
    }

    pub(super) fn CreateInitialOWScreenView_Small_East(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.overworld_map16_load_state();
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x60),
            dst_off: 0x18,
            y_unit: map16.y_unit,
        });
        self.set_overworld_map16_src_off(0x041e);
        self.set_overworld_map16_y_unit(0);
        self.set_overworld_map16_dst_off(7);
        self.TriggerAndFinishMapLoadStripe_X(8);
        let off = self.overworld_map16_dst_off().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        let src = self.overworld_map16_src_off().wrapping_sub(0x2e);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn CreateInitialNewScreenMapToScroll(&mut self) {
        let dir = self.world_state_view().screen_transition_direction_bits();
        if self.overworld_map_is_small() {
            match dir {
                1 => self.CreateInitialOWScreenView_Small_East(),
                2 => self.CreateInitialOWScreenView_Small_West(),
                4 => self.CreateInitialOWScreenView_Small_South(),
                8 => self.CreateInitialOWScreenView_Small_North(),
                _ => {
                    self.set_submodule(0);
                    panic!(
                        "CreateInitialNewScreenMapToScroll small invalid direction {}",
                        dir
                    );
                }
            }
        } else {
            match dir {
                1 => self.CreateInitialOWScreenView_Big_East(),
                2 => self.CreateInitialOWScreenView_Big_West(),
                4 => self.CreateInitialOWScreenView_Big_South(),
                8 => self.CreateInitialOWScreenView_Big_North(),
                _ => {
                    self.set_submodule(0);
                    panic!(
                        "CreateInitialNewScreenMapToScroll big invalid direction {}",
                        dir
                    );
                }
            }
        }
    }

    pub(super) fn Decompress_bank02(&mut self, dst: usize, src: &[u8]) -> i32 {
        let dst_org = dst;
        let mut dst = dst;
        let mut src_pos = 0usize;
        loop {
            let mut cmd = src[src_pos];
            src_pos += 1;
            if cmd == 0xff {
                return dst.wrapping_sub(dst_org) as i32;
            }
            let mut len;
            if cmd & 0xe0 != 0xe0 {
                len = (cmd & 0x1f) as usize + 1;
                cmd &= 0xe0;
            } else {
                let len_lo = src[src_pos];
                src_pos += 1;
                len = len_lo as usize + (((cmd & 3) as usize) << 8) + 1;
                cmd = (cmd << 3) & 0xe0;
            }

            if cmd == 0 {
                while len != 0 {
                    let value = src[src_pos];
                    src_pos += 1;
                    self.overworld_map16_decode_view_mut()
                        .write_decompressed_byte(dst, value);
                    dst += 1;
                    len -= 1;
                }
            } else if cmd & 0x80 != 0 {
                let hi = src[src_pos] as usize;
                let lo = src[src_pos + 1] as usize;
                src_pos += 2;
                let mut offs = (hi << 8) | lo;
                while len != 0 {
                    self.overworld_map16_decode_view_mut()
                        .copy_decompressed_byte(dst_org, dst, offs);
                    dst += 1;
                    offs += 1;
                    len -= 1;
                }
            } else if cmd & 0x40 == 0 {
                let value = src[src_pos];
                src_pos += 1;
                while len != 0 {
                    self.overworld_map16_decode_view_mut()
                        .write_decompressed_byte(dst, value);
                    dst += 1;
                    len -= 1;
                }
            } else if cmd & 0x20 == 0 {
                let lo = src[src_pos];
                let hi = src[src_pos + 1];
                src_pos += 2;
                while len != 0 {
                    self.overworld_map16_decode_view_mut()
                        .write_decompressed_byte(dst, lo);
                    dst += 1;
                    len -= 1;
                    if len == 0 {
                        break;
                    }
                    self.overworld_map16_decode_view_mut()
                        .write_decompressed_byte(dst, hi);
                    dst += 1;
                    len -= 1;
                }
            } else {
                let mut value = src[src_pos];
                src_pos += 1;
                while len != 0 {
                    self.overworld_map16_decode_view_mut()
                        .write_decompressed_byte(dst, value);
                    dst += 1;
                    value = value.wrapping_add(1);
                    len -= 1;
                }
            }
        }
    }

    pub(super) fn Overworld_DecompressAndDrawAllQuadrants(&mut self) {
        let si = self.world_location_state().overworld_screen_index() as i32;
        self.Overworld_DecompressAndDrawOneQuadrant(0x2000, si);
        self.Overworld_DecompressAndDrawOneQuadrant(0x2040, si + 1);
        self.Overworld_DecompressAndDrawOneQuadrant(0x3000, si + 8);
        self.Overworld_DecompressAndDrawOneQuadrant(0x3040, si + 9);
    }

    pub(super) fn Overworld_DecompressAndDrawOneQuadrant(&mut self, mut dst: usize, screen: i32) {
        let hibytes = self.GetOverworldHibytes(screen);
        self.Decompress_bank02(OVERWORLD_DECOMP_BUFFER, &hibytes);
        self.overworld_map16_decode_view_mut()
            .copy_scratch_to_source_words_high(256);

        let lobytes = self.GetOverworldLobytes(screen);
        self.Decompress_bank02(OVERWORLD_DECOMP_BUFFER, &lobytes);
        self.overworld_map16_decode_view_mut()
            .copy_scratch_to_source_words_low(256);

        self.overworld_map16_decode_view_mut()
            .set_decode_last(0xffff);
        let mut src_offset = 0usize;
        for _ in 0..16 {
            for _ in 0..16 {
                let input = self
                    .overworld_map16_decode_view()
                    .source_word(src_offset)
                    .wrapping_mul(2);
                src_offset += 2;
                self.Overworld_ParseMap32Definition(dst, input);
                dst += 4;
            }
            dst += 192;
        }
    }

    fn fill_map16_decode_block(&mut self, dst: usize, table: &[u8], x: usize) {
        self.overworld_map16_decode_view_mut()
            .decode_block_fill(dst, table, x);
    }

    pub(super) fn Overworld_ParseMap32Definition(&mut self, dst: usize, input: u16) {
        let a = input & !7;
        if a != self.overworld_map16_decode_view().decode_last() {
            self.overworld_map16_decode_view_mut().set_decode_last(a);
            self.overworld_map16_decode_view_mut()
                .set_decode_tmp(a >> 1);
            let x = (a >> 1) as usize + (a >> 2) as usize;
            let map0 = self
                .asset_raw(60)
                .expect("Overworld_ParseMap32Definition missing kMap32ToMap16_0 asset")
                .to_vec();
            let map1 = self
                .asset_raw(61)
                .expect("Overworld_ParseMap32Definition missing kMap32ToMap16_1 asset")
                .to_vec();
            let map2 = self
                .asset_raw(62)
                .expect("Overworld_ParseMap32Definition missing kMap32ToMap16_2 asset")
                .to_vec();
            let map3 = self
                .asset_raw(63)
                .expect("Overworld_ParseMap32Definition missing kMap32ToMap16_3 asset")
                .to_vec();
            self.fill_map16_decode_block(MAP16_DECODE_0_OVERWORLD, &map0, x);
            self.fill_map16_decode_block(MAP16_DECODE_1_OVERWORLD, &map1, x);
            self.fill_map16_decode_block(MAP16_DECODE_2_OVERWORLD, &map2, x);
            self.fill_map16_decode_block(MAP16_DECODE_3_OVERWORLD, &map3, x);
        }

        let idx = (input & 7) as usize;
        self.overworld_map16_decode_view_mut()
            .write_decoded_map32_to_bg2_tilemap(dst, idx);
    }

    pub(super) fn OverworldLoad_LoadSubOverlayMap32(&mut self) {
        let si = self.world_location_state().overworld_screen_index() as i32;
        self.Overworld_DecompressAndDrawOneQuadrant(0x4000, si);
    }

    pub(super) fn Map16ToMap8(&mut self, src: usize, r20: i32) {
        let map16_src = self.overworld_map16_src_off().wrapping_add(0x1000);
        self.set_overworld_map16_src_off(map16_src);
        let mut r14 = 0i32;
        let mut r10 = WORD_7F4000_OVERWORLD;
        for _ in 0..32 {
            self.OverworldCopyMap16ToBuffer(src, r20 as u16, r14, r10);
            r14 += 0x100;
            r10 += 4;
            let map16_src = self.overworld_map16_src_off().wrapping_sub(0x80);
            self.set_overworld_map16_src_off(map16_src);
            let y_unit = self.overworld_map16_y_unit().wrapping_sub(1) & 0x1f;
            self.set_overworld_map16_y_unit(y_unit);
        }
    }

    pub(super) fn OverworldCopyMap16ToBuffer(
        &mut self,
        src: usize,
        r20: u16,
        mut r14: i32,
        mut r10: usize,
    ) {
        let map8 = self.GetMap16toMap8Table();
        let mut yr = (self.overworld_map16_src_off().wrapping_sub(0x410) & 0x1fff) as usize;
        let mut xr = self.overworld_map16_dst_off() as usize & 0x1f;
        for _ in 0..32 {
            let value = self.display_nmi_view().word_at(src + yr);
            self.world_state_view_mut()
                .set_dung_replacement_tile_state(xr, value);
            xr = (xr + 1) & 0x1f;
            yr = (yr + 2) & 0x1fff;
        }

        let mut r0 = 0u16;
        let mut y_unit = self.overworld_map16_y_unit();
        if y_unit >= 0x10 {
            y_unit &= 0x0f;
            r0 = 0x800;
        }
        r0 = r0.wrapping_add(y_unit.wrapping_mul(64));

        let mut tmp = 0usize;
        for _ in 0..2 {
            self.vram_upload_data_view_mut()
                .write_le_u16_at(r10, r0 | r20);
            r10 += 2;
            for _ in 0..16 {
                let k = self.world_state_view().dung_replacement_tile_state(tmp);
                tmp += 1;
                let m0 = self.overworld_map16_to_map8_word(&map8, k, 0);
                let m1 = self.overworld_map16_to_map8_word(&map8, k, 1);
                let m2 = self.overworld_map16_to_map8_word(&map8, k, 2);
                let m3 = self.overworld_map16_to_map8_word(&map8, k, 3);
                self.dungeon_state_view_mut()
                    .set_bg2_attr_word(r14 as usize, m0);
                self.dungeon_state_view_mut()
                    .set_bg2_attr_word(r14 as usize + 64, m2);
                self.dungeon_state_view_mut()
                    .set_bg2_attr_word(r14 as usize + 2, m1);
                self.dungeon_state_view_mut()
                    .set_bg2_attr_word(r14 as usize + 66, m3);
                r14 += 4;
            }
            r0 = r0.wrapping_add(0x400);
            r14 += 0x40;
        }
    }

    pub(super) fn SomeTileMapChange(&mut self) {
        self.Overworld_DecompressAndDrawAllQuadrants();
        for i in 0..64 {
            self.dungeon_state_view_mut().set_bg1_tile(i, 0x0dc4);
        }
        self.Overworld_HandleOverlaysAndBombDoors();
        self.increment_submodule();
    }

    pub(super) fn Module09_LoadNewMapAndGFX(&mut self) {
        self.world_state_view_mut()
            .set_overworld_peg_puzzle_progress(0);
        self.SomeTileMapChange();
        self.display_nmi_view_mut()
            .increment_core_update_disable_flag();
        self.CreateInitialNewScreenMapToScroll();
        self.LoadNewSpriteGFXSet();
    }

    pub(super) fn Overworld_DrawQuadrantsAndOverlays(&mut self) {
        self.Overworld_DecompressAndDrawAllQuadrants();
        for i in 0..64 {
            self.dungeon_state_view_mut().set_bg1_tile(i, 0x0dc4);
        }
        let mut pos = self.world_state_view().ow_entrance_value();
        self.replay_trace_door_overlay("draw-before-entrance", pos & 0x1fff);
        if pos != 0 && pos != 0xffff {
            if pos < 0x8000 {
                self.dungeon_state_view_mut()
                    .set_bg2_tile_by_byte_pos(pos, 0x0da4);
                self.Overworld_Memorize_Map16_Change(pos, 0x0da4);
                self.dungeon_state_view_mut()
                    .set_bg2_tile_by_byte_pos(pos.wrapping_add(2), 0x0da6);
                self.Overworld_Memorize_Map16_Change(pos.wrapping_add(2), 0x0da6);
                self.replay_trace_door_overlay("draw-normal-door", pos);
            } else {
                pos &= 0x1fff;
                self.dungeon_state_view_mut()
                    .set_bg2_tile_by_byte_pos(pos, 0x0db4);
                self.Overworld_Memorize_Map16_Change(pos, 0x0db4);
                self.dungeon_state_view_mut()
                    .set_bg2_tile_by_byte_pos(pos.wrapping_add(2), 0x0db5);
                self.Overworld_Memorize_Map16_Change(pos.wrapping_add(2), 0x0db5);
                self.replay_trace_door_overlay("draw-open-door", pos);
            }
            self.world_state_view_mut().set_ow_entrance_value(0);
        }
        self.Overworld_HandleOverlaysAndBombDoors();
        let screen_byte = self.world_location_state().overworld_screen_index() as usize;
        if screen_byte < SECONDARY_OVERLAY_BY_OVERWORLD_SCREEN.len() {
            self.replay_trace_door_overlay(
                "draw-after-overlays",
                SECONDARY_OVERLAY_BY_OVERWORLD_SCREEN[screen_byte],
            );
        }
    }

    pub(super) fn MirrorBonk_RecoverChangedTiles(&mut self) {
        let count = self.memorized_tile_view().count() >> 1;
        for i in 0..count as usize {
            let memorized_tiles = self.memorized_tile_view();
            let pos = memorized_tiles.entry_addr(i);
            let value = memorized_tiles.entry_value(i);
            self.dungeon_state_view_mut()
                .set_bg2_tile_by_byte_pos(pos, value);
        }
    }

    pub(super) fn CheckForNewlyLoadedMapAreas_North(&mut self, dst: usize) -> usize {
        let src = self.overworld_map16_src_off();
        if (src as i16).wrapping_sub(0x80) < 0 {
            return dst;
        }
        let mut dst = dst;
        if !self.overworld_map_is_small() {
            self.write_overworld_vram_word(dst, 0x0080);
            dst = self.BufferAndBuildMap16Stripes_Y(dst + 1);
        }
        self.set_overworld_map16_src_off(src.wrapping_sub(0x80));
        let y_unit = self.overworld_map16_y_unit().wrapping_sub(1) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        dst
    }

    pub(super) fn CheckForNewlyLoadedMapAreas_South(&mut self, dst: usize) -> usize {
        let src = self.overworld_map16_src_off();
        if src >= 0x1800 {
            return dst;
        }
        let mut dst = dst;
        if !self.overworld_map_is_small() {
            self.write_overworld_vram_word(dst, 0x0080);
            dst = self.BufferAndBuildMap16Stripes_Y(dst + 1);
        }
        self.set_overworld_map16_src_off(src.wrapping_add(0x80));
        let y_unit = self.overworld_map16_y_unit().wrapping_add(1) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        dst
    }

    pub(super) fn CheckForNewlyLoadedMapAreas_West(&mut self, dst: usize) -> usize {
        let mut pos = self.overworld_map16_src_off();
        while pos >= 0x80 {
            pos = pos.wrapping_sub(0x80);
        }
        if pos == 0 {
            return dst;
        }
        let mut dst = dst;
        if !self.overworld_map_is_small() {
            self.write_overworld_vram_word(dst, 0x8040);
            dst = self.BufferAndBuildMap16Stripes_X(dst + 1);
        }
        let src = self.overworld_map16_src_off().wrapping_sub(2);
        self.set_overworld_map16_src_off(src);
        let off = self.overworld_map16_dst_off().wrapping_sub(1) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        dst
    }

    pub(super) fn CheckForNewlyLoadedMapAreas_East(&mut self, dst: usize) -> usize {
        let mut pos = self.overworld_map16_src_off();
        while pos >= 0x80 {
            pos = pos.wrapping_sub(0x80);
        }
        if pos >= 0x60 {
            return dst;
        }
        let mut dst = dst;
        if !self.overworld_map_is_small() {
            self.write_overworld_vram_word(dst, 0x8040);
            dst = self.BufferAndBuildMap16Stripes_X(dst + 1);
        }
        let src = self.overworld_map16_src_off().wrapping_add(2);
        self.set_overworld_map16_src_off(src);
        let off = self.overworld_map16_dst_off().wrapping_add(1) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        dst
    }

    pub(super) fn OverworldHandleMapScroll(&mut self) {
        let before = self.overworld_map16_src_off();
        let before_y_unit = self.overworld_map16_y_unit();
        let before_dst = self.overworld_map16_dst_off();
        let dir = self.world_state_view().screen_transition_direction_bits();
        let dst = match dir {
            1 => {
                let dst = self.CheckForNewlyLoadedMapAreas_East(0);
                self.world_state_view_mut()
                    .clear_screen_transition_direction_bits();
                dst
            }
            2 => {
                let dst = self.CheckForNewlyLoadedMapAreas_West(0);
                self.world_state_view_mut()
                    .clear_screen_transition_direction_bits();
                dst
            }
            4 => {
                let dst = self.CheckForNewlyLoadedMapAreas_South(0);
                self.world_state_view_mut()
                    .clear_screen_transition_direction_bits();
                dst
            }
            5 | 6 => {
                let dst = self.CheckForNewlyLoadedMapAreas_South(0);
                self.world_state_view_mut()
                    .and_screen_transition_direction_bits(3);
                dst
            }
            8 => {
                let dst = self.CheckForNewlyLoadedMapAreas_North(0);
                self.world_state_view_mut()
                    .clear_screen_transition_direction_bits();
                dst
            }
            9 | 10 => {
                let dst = self.CheckForNewlyLoadedMapAreas_North(0);
                self.world_state_view_mut()
                    .and_screen_transition_direction_bits(3);
                dst
            }
            _ => {
                self.set_submodule(0);
                panic!("OverworldHandleMapScroll invalid direction {dir}");
            }
        };
        self.write_overworld_vram_word(dst, 0xffff);
        self.write_overworld_vram_word(dst + 1, 0xffff);
        if dst != 0 {
            self.set_pending_nmi_subroutine(3);
        }
        let screen_transition = self.world_state_view().screen_transition_direction_bits();
        self.world_state_view_mut()
            .set_screen_transition(screen_transition);
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some()
            && matches!(
                u16::from(self.world_location_state().overworld_screen_index()),
                0 | 2 | 0x80
            )
        {
            println!(
                "owlive-scroll frame={} screen=0x{:04x} dir=0x{:02x} before=0x{:04x} after=0x{:04x} yunit=0x{:04x}->0x{:04x} dst=0x{:04x}->0x{:04x} trans=0x{:02x} sub={} subsub={} x=0x{:04x} y=0x{:04x}",
                self.frame_state().frame_counter,
                u16::from(self.world_location_state().overworld_screen_index()),
                dir,
                before,
                self.overworld_map16_src_off(),
                before_y_unit,
                self.overworld_map16_y_unit(),
                before_dst,
                self.overworld_map16_dst_off(),
                self.world_state_view().screen_transition(),
                self.frame_state().submodule,
                self.frame_state().subsubmodule,
                self.player_state_view().x(),
                self.player_state_view().y(),
            );
        }
    }

    pub(super) fn Overworld_RunScrollTransition(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        self.Graphics_IncrementalVRAMUpload();
        let rv = self.OverworldScrollTransition();
        if rv & 0x0f == 0 {
            let direction_bits = self.world_state_view().overworld_screen_trans_dir_bits();
            self.world_state_view_mut()
                .set_screen_transition_direction_bits(direction_bits);
            self.OverworldTransitionScrollAndLoadMap();
            self.world_state_view_mut()
                .clear_screen_transition_direction_bits();
        }
    }

    pub(super) fn Module09_LoadNewSprites(&mut self) {
        if self.world_state_view().screen_transition() == 1 {
            let bg2v = self.world_state_view().bg2_y().wrapping_add(2);
            self.world_state_view_mut().set_bg2_y(bg2v);
            let link_y = self.player_state_view().y().wrapping_add(2);
            self.player_state_view_mut().set_y(link_y);
        }
        self.sprite_overworld_reload_all_just_load();
        self.memorized_tile_view_mut().clear_count();
        if self.save_progress_view().progress_indicator() >= 2 && self.frame_state().submodule != 18
        {
            self.Overworld_SetFixedColAndScroll();
        }
        self.Overworld_StartScrollTransition();
    }

    pub(super) fn Overworld_StartScrollTransition(&mut self) {
        self.increment_submodule();
        if self.world_state_view().overworld_screen_trans_dir_bits() >= 4 {
            let direction_bits = self.world_state_view().overworld_screen_trans_dir_bits();
            self.world_state_view_mut()
                .set_screen_transition_direction_bits(direction_bits);
            self.OverworldTransitionScrollAndLoadMap();
            self.world_state_view_mut()
                .clear_screen_transition_direction_bits();
        }
    }

    pub(super) fn Overworld_EaseOffScrollTransition(&mut self) {
        if self.overworld_map_is_small() {
            let direction_bits = self.world_state_view().overworld_screen_trans_dir_bits();
            self.world_state_view_mut()
                .set_screen_transition_direction_bits(direction_bits);
            self.OverworldTransitionScrollAndLoadMap();
            self.world_state_view_mut()
                .clear_screen_transition_direction_bits();
        }
        self.increment_subsubmodule();
        if self.frame_state().subsubmodule < 8 {
            return;
        }
        let dir = self.world_state_view().overworld_screen_trans_dir_bits();
        if (dir == 8 || dir == 2) && self.frame_state().subsubmodule < 9 {
            return;
        }

        self.set_subsubmodule(0);
        self.world_state_view_mut()
            .set_overworld_screen_trans_dir_bits(0);

        if self.overworld_map_is_small() {
            let backup = self.small_overworld_map16_scroll_backup_state();
            self.store_overworld_map16_load_state(OverworldMap16LoadState {
                src_off: backup.src_off,
                dst_off: backup.dst_off,
                y_unit: backup.y_unit,
            });
        }
        self.increment_submodule();
        self.follower_disable();
    }

    pub(super) fn OverworldHandleTransitions(&mut self) {
        if self
            .world_state_view()
            .has_screen_transition_direction_bits()
        {
            self.OverworldHandleMapScroll();
        }

        let current_area = self.world_state_view().current_area_of_player_word();
        let area_half = (self.world_state_view().current_area_of_player() >> 1) as usize;
        let bounds = self.overworld_screen_size_view().right_bottom_bound_word();
        let mut transition: Option<(u8, usize)> = None;

        if self.player_state_view().y_velocity() != 0 {
            let dir = self.player_state_view().direction() & 12;
            let t = self
                .player_state_view()
                .y()
                .wrapping_sub(OVERWORLD_AREA_BASE_Y[area_half]);
            if t < 4 {
                transition = Some((dir, 3));
            } else if t >= bounds {
                transition = Some((dir, 2));
            }
        }

        if transition.is_none() && self.player_state_view().x_velocity() != 0 {
            let dir = self.player_state_view().direction() & 3;
            let t = self
                .player_state_view()
                .x()
                .wrapping_sub(OVERWORLD_AREA_BASE_X[area_half]);
            if t < 6 {
                transition = Some((dir, 1));
            } else if t >= bounds.wrapping_add(4) {
                transition = Some((dir, 0));
            }
        }

        let Some((dir, y_idx)) = transition else {
            self.Overworld_CheckSpecialSwitchArea();
            return;
        };

        let expected_dir = [1u8, 2, 4, 8][y_idx];
        if expected_dir != dir || self.link_check_for_edge_screen_transition() {
            self.Overworld_CheckSpecialSwitchArea();
            return;
        }

        self.Dungeon_ResetTorchBackgroundAndPlayerInner();
        let mut map16 = self.overworld_map16_src_off();
        let map16_before = map16;
        map16 &= SPECIAL_SWITCH_MAP16_MASKS[y_idx];
        let pushed = (current_area.wrapping_add_signed(SPECIAL_SWITCH_AREA_DELTAS[y_idx]) >> 1)
            as usize
            & 0x3f;
        let map16_add = SPECIAL_SWITCH_MAP16_OFFSETS[y_idx * 64 + pushed];
        map16 = map16.wrapping_add(map16_add);
        self.set_overworld_map16_src_off(map16);
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some() {
            println!(
                "owtrans-map16 frame={} y={} pushed=0x{:02x} cur=0x{:04x} old=0x{:04x} mask=0x{:04x} add=0x{:04x} new=0x{:04x} screen=0x{:04x} x=0x{:04x} ycoord=0x{:04x} dir=0x{:02x}",
                self.frame_state().frame_counter,
                y_idx,
                pushed,
                current_area,
                map16_before,
                SPECIAL_SWITCH_MAP16_MASKS[y_idx],
                map16_add,
                map16,
                u16::from(self.world_location_state().overworld_screen_index()),
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.player_state_view().direction(),
            );
        }

        let old_screen = self.world_location_state().overworld_screen_index();
        if old_screen == 0x2a {
            self.system_signals_view_mut()
                .set_ambient_sound_effect(0x80);
        }

        let new_area =
            OVERWORLD_AREA_TILEMAP_HEADS[pushed] | self.save_progress_view().dark_world_state();
        self.set_overworld_screen(new_area);
        self.world_state_view_mut()
            .set_overworld_area_index(new_area);
        if self.save_progress_view().dark_world_state() == 0
            || self.inventory_state_view().moon_pearl() != 0
        {
            let music = self.overworld_config_table_view().music(new_area as usize);
            if music & 0xf0 == 0 {
                self.system_signals_view_mut().set_ambient_sound_effect(5);
            }
            if !self.zelda_is_playing_music_track(music & 0x0f) {
                self.system_signals_view_mut().set_music_control(0xf1);
            }
        }

        self.Overworld_LoadGFXAndScreenSize();
        self.set_submodule(1);
        self.world_state_view_mut()
            .set_overworld_screen_trans_dir_bits(dir);
        self.world_state_view_mut()
            .set_screen_transition_direction_bits(dir);
        let dir_enum = self.DirToEnum(dir as i32) as u8;
        self.world_state_view_mut()
            .set_transition_dir_enum(dir_enum);
        self.world_state_view_mut().set_screen_transition(dir_enum);
        self.world_state_view_mut().set_ow_entrance_value(0);
        self.dungeon_state_view_mut()
            .set_big_rock_starting_address(0);
        self.world_state_view_mut().set_transition_counter(0);

        if old_screen & 0x3f == 0
            || self.world_location_state().overworld_screen_index() & 0xbf == 0
        {
            self.set_subsubmodule(0);
            self.set_submodule(13);
            self.set_mosaic_copy(0);
            self.clear_mosaic_level();
        } else {
            let sc = self.world_location_state().overworld_screen_index() as usize;
            self.Overworld_LoadPalettes(
                self.GetOverworldBgPalette(sc as u8),
                self.overworld_config_table_view().sprite_palette(sc),
            );
            self.Overworld_CopyPalettesToCache();
        }
    }

    pub(super) fn Overworld_OperateCameraScroll(&mut self) {
        let z = if self.world_state_view().allow_scroll_z() != 0
            && self.player_state_view().z() != 0xffff
        {
            self.player_state_view().z()
        } else {
            0
        };
        let y = self
            .player_state_view()
            .y()
            .wrapping_sub(z)
            .wrapping_add(12);

        let link_y_velocity = self.player_state_view().y_velocity();
        if link_y_velocity != 0 {
            let vy = if (link_y_velocity as i8).is_negative() {
                -1
            } else {
                1
            };
            let mut av = if (link_y_velocity as i8).is_negative() {
                (!link_y_velocity).wrapping_add(1)
            } else {
                link_y_velocity
            };
            let mut r4 = 0u16;
            while av != 0 {
                if (link_y_velocity as i8).is_negative() {
                    if y <= self.world_state_view().camera_y_coord_scroll_low() {
                        r4 = r4.wrapping_add(self.OverworldCameraBoundaryCheck(6, 0, vy, 0) as u16);
                    }
                } else if y >= self.world_state_view().camera_y_coord_scroll_hi() {
                    r4 = r4.wrapping_add(self.OverworldCameraBoundaryCheck(6, 2, vy, 0) as u16);
                }
                av = av.wrapping_sub(1);
            }
            self.overworld_scroll_delta_view_mut().set_low_word(r4);
            let oi = self.world_state_view().overlay_index();
            if oi != 0x97 && oi != 0x9d && r4 != 0 {
                let (subp, mut scroll) = if oi == 0xb5 || oi == 0xbe {
                    ((r4 & 3) << 14, r4 >> 2)
                } else {
                    ((r4 & 1) << 15, r4 >> 1)
                };
                if scroll
                    >= if oi == 0xb5 || oi == 0xbe {
                        0x3000
                    } else {
                        0x7000
                    }
                {
                    scroll |= 0xf000;
                }
                self.ppu_scroll_copy_view_mut()
                    .add_bg1_v_live_subpixel(subp, scroll);
                if self.world_location_state().overworld_screen_index() & 0x3f == 0x1b {
                    let bg1 = self.world_state_view().bg1_y();
                    if bg1 <= 0x0600 {
                        self.world_state_view_mut().set_bg1_y(0x0600);
                    } else if bg1 >= 0x06c0 {
                        self.world_state_view_mut().set_bg1_y(0x06c0);
                    }
                }
            }
        }

        let x = self.player_state_view().x().wrapping_add(8);
        let link_x_velocity = self.player_state_view().x_velocity();
        if link_x_velocity != 0 {
            let vx = if (link_x_velocity as i8).is_negative() {
                -1
            } else {
                1
            };
            let mut ax = if (link_x_velocity as i8).is_negative() {
                (!link_x_velocity).wrapping_add(1)
            } else {
                link_x_velocity
            };
            let mut r4 = 0u16;
            while ax != 0 {
                if (link_x_velocity as i8).is_negative() {
                    if x <= self.world_state_view().camera_x_coord_scroll_low() {
                        r4 = r4.wrapping_add(self.OverworldCameraBoundaryCheck(0, 4, vx, 4) as u16);
                    }
                } else if x >= self.world_state_view().camera_x_coord_scroll_hi() {
                    r4 = r4.wrapping_add(self.OverworldCameraBoundaryCheck(0, 6, vx, 4) as u16);
                }
                ax = ax.wrapping_sub(1);
            }
            self.overworld_scroll_delta_view_mut().set_high_word(r4);
            let oi = self.world_state_view().overlay_index();
            if oi != 0x97 && oi != 0x9d && r4 != 0 {
                let (subp, mut scroll) = if oi == 0x95 || oi == 0x9e {
                    ((r4 & 3) << 14, r4 >> 2)
                } else {
                    ((r4 & 1) << 15, r4 >> 1)
                };
                if scroll
                    >= if oi == 0x95 || oi == 0x9e {
                        0x3000
                    } else {
                        0x7000
                    }
                {
                    scroll |= 0xf000;
                }
                self.ppu_scroll_copy_view_mut()
                    .add_bg1_h_live_subpixel(subp, scroll);
            }
        }

        if self.world_location_state().overworld_screen_index() != 0x47 {
            if self.world_state_view().overlay_index() == 0x9c {
                self.ppu_scroll_copy_view_mut()
                    .subtract_bg1_v_live_subpixel(0x2000);
                let scroll_delta = self.overworld_scroll_delta_view().word();
                let bg1_v = self.world_state_view().bg1_y().wrapping_add(scroll_delta);
                self.world_state_view_mut().set_bg1_y(bg1_v);
                self.ppu_scroll_copy_view_mut()
                    .copy_bg2_h_live_to_bg1_h_live();
            } else if self.world_state_view().overlay_index() == 0x97
                || self.world_state_view().overlay_index() == 0x9d
            {
                self.ppu_scroll_copy_view_mut()
                    .add_bg1_v_live_subpixel(0x2000, 0);
                self.ppu_scroll_copy_view_mut()
                    .add_bg1_h_live_subpixel(0x2000, 0);
            }
        }

        if self.world_location_state().dungeon_room == 0x0181 {
            let bg2v = self.world_state_view().bg2_y() | 0x0100;
            self.world_state_view_mut().set_bg1_y(bg2v);
            self.ppu_scroll_copy_view_mut()
                .copy_bg2_h_live_to_bg1_h_live();
        }
    }

    pub(super) fn OverworldCameraBoundaryCheck(
        &mut self,
        xa: i32,
        ya: i32,
        vd: i32,
        r8: i32,
    ) -> i32 {
        let ya = (ya >> 1) as usize;
        let r8 = (r8 >> 1) as usize;
        let vertical_axis = xa != 0;
        if self
            .ppu_scroll_copy_view()
            .bg2_copy2_for_axis(vertical_axis)
            == self.room_bounds_view().packed_bound(ya)
        {
            self.world_state_view_mut()
                .clear_opposed_scroll_counters(ya);
            return 0;
        }
        self.ppu_scroll_copy_view_mut()
            .add_bg2_copy2_for_axis_signed(vertical_axis, vd as i16);

        let horizontal = r8 != 0;
        self.world_state_view_mut()
            .add_camera_scroll_for_axis(horizontal, vd as i16);

        let mut value = self
            .world_state_view()
            .overworld_scroll_counter_for_axis(ya)
            .wrapping_add(1);
        if (value.wrapping_sub(0x10) as i16) >= 0 {
            value = value.wrapping_sub(0x10);
            self.world_state_view_mut()
                .or_screen_transition_direction_bits(OVERWORLD_SCROLL_DIRECTION_BITS[ya] as u8);
        }
        self.world_state_view_mut()
            .set_opposed_scroll_counter_pair(ya, value);
        vd
    }

    pub(super) fn OverworldScrollTransition(&mut self) -> i32 {
        self.world_state_view_mut().increment_transition_counter();
        let y = self.world_state_view().screen_transition() as usize;
        let d = OVERWORLD_TRANSITION_SCROLL_DELTAS[y];
        let rv;
        if y < 2 {
            self.overworld_scroll_delta_view_mut().set_low(d as u8);
            rv = self.world_state_view().bg2_y().wrapping_add_signed(d);
            self.world_state_view_mut().set_bg2_y(rv);
            if self.world_location_state().overworld_screen_index() != 0x1b
                && self.world_location_state().overworld_screen_index() != 0x5b
            {
                self.world_state_view_mut().set_bg1_y(rv);
            }
            if self.world_state_view().transition_counter()
                >= OVERWORLD_TRANSITION_PLAYER_MOVE_FRAMES[y]
            {
                let link_y = self.player_state_view().y().wrapping_add_signed(d);
                self.player_state_view_mut().set_y(link_y);
            }
            if rv != self.world_state_view().up_down_scroll_target(y) {
                return rv as i32;
            }
            if y == 0 {
                let bg2 = self.world_state_view().bg2_y().wrapping_sub(2);
                self.world_state_view_mut().set_bg2_y(bg2);
            }
            let link_y = self.player_state_view().y() & !7;
            self.player_state_view_mut().set_y(link_y);
            let camera_hi = link_y
                .wrapping_add_signed(OVERWORLD_TRANSITION_CAMERA_OFFSETS[y])
                .wrapping_add(11);
            self.world_state_view_mut()
                .set_camera_scroll_from_link_for_axis(false, camera_hi);
            self.world_state_view_mut().clear_opposed_scroll_counters(0);
        } else {
            self.overworld_scroll_delta_view_mut().set_high(d as u8);
            rv = self.world_state_view().bg2_x().wrapping_add_signed(d);
            self.world_state_view_mut().set_bg2_x(rv);
            if self.world_location_state().overworld_screen_index() != 0x1b
                && self.world_location_state().overworld_screen_index() != 0x5b
            {
                self.world_state_view_mut().set_bg1_x(rv);
            }
            if self.world_state_view().transition_counter()
                >= OVERWORLD_TRANSITION_PLAYER_MOVE_FRAMES[y]
            {
                let link_x = self.player_state_view().x().wrapping_add_signed(d);
                self.player_state_view_mut().set_x(link_x);
            }
            if rv != self.world_state_view().up_down_scroll_target(y) {
                return rv as i32;
            }
            let link_x = self.player_state_view().x() & !7;
            self.player_state_view_mut().set_x(link_x);
            let camera_hi = link_x
                .wrapping_add_signed(OVERWORLD_TRANSITION_CAMERA_OFFSETS[y])
                .wrapping_add(11);
            self.world_state_view_mut()
                .set_camera_scroll_from_link_for_axis(true, camera_hi);
            self.world_state_view_mut().clear_opposed_scroll_counters(2);
        }

        let area = ((self.world_state_view().current_area_of_player_word() >> 1) as i16)
            + OVERWORLD_ADJACENT_AREA_DELTAS[y];
        self.Overworld_SetCameraBoundaries(
            if self.overworld_screen_size_view().is_big_area_word() != 0 {
                1
            } else {
                0
            },
            area as i32,
        );
        self.world_state_view_mut()
            .set_flag_overworld_area_changed(1);
        self.increment_submodule();
        self.set_subsubmodule(0);
        self.world_state_view_mut().set_transition_counter(0);
        self.sprite_initialize_slots();
        rv as i32
    }

    pub(super) fn Overworld_FinalizeEntryOntoScreen(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        let transition_dir = self.world_state_view().transition_dir_enum();
        let mut d: i16 = if transition_dir & 1 != 0 { 2 } else { -2 };
        if transition_dir & 2 != 0 {
            let link_x = self.player_state_view().x().wrapping_add_signed(d);
            self.player_state_view_mut().set_x(link_x);
            d = link_x as i16;
        } else {
            let link_y = self.player_state_view().y().wrapping_add_signed(d);
            self.player_state_view_mut().set_y(link_y);
            d = link_y as i16;
        }
        if (d & 0x00fe)
            == i16::from(OVERWORLD_ENTRY_SETTLE_COORDINATES[usize::from(transition_dir)])
        {
            self.set_submodule(0);
            self.set_subsubmodule(0);
            let m = self
                .overworld_config_table_view()
                .music(self.world_location_state().overworld_screen_index() as usize);
            self.system_signals_view_mut()
                .set_ambient_sound_effect(m >> 4);
            if self.system_signals_view().current_music_control() == 0xf1 {
                self.system_signals_view_mut().set_music_control(m & 0x0f);
            }
        }
        self.Overworld_OperateCameraScroll();
        if self
            .world_state_view()
            .has_screen_transition_direction_bits()
        {
            self.OverworldHandleMapScroll();
        }
    }

    pub(super) fn Overworld_Func1F(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        let transition_dir = self.world_state_view().transition_dir_enum();
        let vel: i8 = if transition_dir & 1 != 0 { 1 } else { -1 };
        if transition_dir & 2 != 0 {
            let link_x = self.player_state_view().x().wrapping_add_signed(vel as i16);
            self.player_state_view_mut().set_x(link_x);
            self.player_state_view_mut().set_x_velocity(vel as u8);
        } else {
            let link_y = self.player_state_view().y().wrapping_add_signed(vel as i16);
            self.player_state_view_mut().set_y(link_y);
            self.player_state_view_mut().set_y_velocity(vel as u8);
        }
        if self
            .world_state_view_mut()
            .decrement_ow_countdown_transition()
            == 0
        {
            self.set_main_module(9);
            self.set_subsubmodule(0);
            self.set_submodule(0);
        }
        self.Overworld_OperateCameraScroll();
    }

    pub(super) fn Module08_02_LoadAndAdvance(&mut self) {
        self.Overworld_LoadAndBuildScreen();
        self.set_main_module(16);
        self.set_submodule(0);
        self.set_subsubmodule(0);
    }

    pub(super) fn Palette_AnimGetMasterSword2(&mut self) {
        let aux = self.palette_buffer_view().aux_full_slice().to_vec();
        self.palette_buffer_view_mut()
            .backup_overworld_palette_from(&aux);
        for i in 0..256 {
            self.palette_buffer_view_mut().set_aux_color(i, 0x7fff);
        }
        copy_le_u16(
            &mut self.ram,
            MAIN_PALETTE_BUFFER + 32 * 2,
            MAIN_PALETTE_BUFFER,
        );
        self.palette_filter_view_mut().set_countdown(0);
        self.palette_filter_view_mut()
            .set_darkening_or_lightening_screen(2);
        self.increment_subsubmodule();
    }

    pub(super) fn Palette_AnimGetMasterSword(&mut self) {
        if self.frame_state().subsubmodule == 0 {
            self.Palette_AnimGetMasterSword2();
            return;
        }

        self.PaletteFilter_BlindingWhite();
        if self.palette_filter_view().darkening_or_lightening_screen() == 0xff {
            for i in 0..8 {
                self.palette_buffer_view_mut().set_main_color(0x58 + i, 0);
                self.palette_buffer_view_mut().set_aux_color(0x58 + i, 0);
            }
            self.palette_filter_view_mut().set_countdown(0);
            self.palette_filter_view_mut()
                .set_darkening_or_lightening_screen(0);
            self.set_submodule(0);
        } else {
            self.Palette_AnimGetMasterSword3();
        }
    }

    pub(super) fn Palette_AnimGetMasterSword3(&mut self) {
        if self.palette_filter_view().darkening_or_lightening_screen() != 0
            || self.palette_filter_view().countdown() != 31
        {
            return;
        }
        let mapbak = self
            .palette_buffer_view()
            .overworld_palette_backup()
            .to_vec();
        self.palette_buffer_view_mut().copy_aux_full_from(&mapbak);
        self.set_sub_screen_layers(0);
    }

    pub(super) fn Overworld_Memorize_Map16_Change(&mut self, pos: u16, value: u16) {
        if value == 0x0dc5 || value == 0x0dc9 {
            return;
        }
        self.memorized_tile_view_mut().append_entry(pos, value);
    }

    fn overworld_bg2_byte_pos(&self, x: u16, y: u16) -> u16 {
        (x.wrapping_sub(self.world_state_view().overworld_offset_base_x())
            & self.world_state_view().overworld_offset_mask_x())
            | ((y.wrapping_sub(self.world_state_view().overworld_offset_base_y())
                & self.world_state_view().overworld_offset_mask_y())
                << 3)
    }

    pub(super) fn Overworld_ReadTileAttribute(&self, x: u16, y: u16) -> u8 {
        let t = self.overworld_bg2_byte_pos(x, y) as usize;
        let tile = self.dungeon_state_view().bg2_tile_by_byte_pos(t as u16) as usize;
        self.asset_raw(164)
            .expect("Overworld_ReadTileAttribute missing kSomeTileAttr asset")[tile]
    }

    pub(super) fn Overworld_RevealSecret(&mut self, pos: u16) -> u16 {
        self.dungeon_secret_scratch_view_mut().clear_pending_kind();

        let screen = u16::from(self.world_location_state().overworld_screen_index()) as usize;
        if screen >= 0x80 {
            self.AdjustSecretForPowder();
            return 0;
        }

        let secret_offsets = self
            .asset_raw(157)
            .expect("Overworld_RevealSecret missing kOverworldSecrets_Offs asset")
            .to_vec();
        let secrets = self
            .asset_raw(158)
            .expect("Overworld_RevealSecret missing kOverworldSecrets asset")
            .to_vec();
        let ptr = u16::from(secret_offsets[screen * 2])
            | (u16::from(secret_offsets[screen * 2 + 1]) << 8);
        let mut ptr = ptr as usize;
        loop {
            let x = u16::from(secrets[ptr]) | (u16::from(secrets[ptr + 1]) << 8);
            if x == 0xffff {
                self.AdjustSecretForPowder();
                return 0;
            }
            if x & 0x7fff == pos {
                break;
            }
            ptr += 3;
        }

        let data = secrets[ptr + 2];
        if data != 0 && data < 0x80 {
            self.dungeon_secret_scratch_view_mut().or_pending_kind(data);
        }
        if data < 0x80 {
            self.AdjustSecretForPowder();
            return 0;
        }

        self.dungeon_secret_scratch_view_mut()
            .set_pending_kind(0xff);
        if data != 0x84 && self.overworld_event_info_view().event_info(screen) & 2 == 0 {
            if screen == 0x5b && self.follower_state_view().indicator() != 13 {
                self.AdjustSecretForPowder();
                return 0;
            }
            self.system_signals_view_mut().set_sound_effect_2(0x1b);
        } else if data == 0x82 && self.enhanced_features_view().has(4096) {
            self.system_signals_view_mut().set_sound_effect_2(0x1b);
        }

        const TILE_BELOW: [u16; 4] = [0x0dcc, 0x0212, 0xffff, 0x0db4];
        self.AdjustSecretForPowder();
        TILE_BELOW[((data & 0x0f) >> 1) as usize]
    }

    pub(super) fn AdjustSecretForPowder(&mut self) {
        if self.player_state_view().item_in_hand_has(0x40) {
            self.dungeon_secret_scratch_view_mut()
                .set_powder_pending_kind();
        }
    }

    pub(super) fn HandlePegPuzzles(&mut self, pos: u16) {
        const LW_TURTLE_ROCK_PEG_POSITIONS: [u16; 3] = [0x0826, 0x05a0, 0x081a];

        if self.world_location_state().overworld_screen_index() == 7 {
            if self.overworld_event_info_view().event_info(7) & 0x20 != 0 {
                return;
            }
            let word = self.world_state_view().overworld_peg_puzzle_progress();
            let idx = (word >> 1) as usize;
            if word != 0xffff && LW_TURTLE_ROCK_PEG_POSITIONS[idx] == pos {
                self.system_signals_view_mut()
                    .set_sound_effect_1_word(0x2d00);
                let next = word.wrapping_add(2);
                self.world_state_view_mut()
                    .set_overworld_peg_puzzle_progress(next);
                if next == 6 {
                    self.system_signals_view_mut()
                        .set_sound_effect_1_word(0x1b00);
                    self.overworld_event_info_view_mut().set_event_bits(7, 0x20);
                    self.set_submodule(47);
                }
            } else {
                self.system_signals_view_mut()
                    .set_sound_effect_1_word(0x003c);
                self.world_state_view_mut()
                    .set_overworld_peg_puzzle_progress(0xffff);
            }
        } else if self.world_location_state().overworld_screen_index() == 98 {
            let next = self
                .world_state_view()
                .overworld_peg_puzzle_progress()
                .wrapping_add(1);
            self.world_state_view_mut()
                .set_overworld_peg_puzzle_progress(next);
            if next == 22 {
                self.overworld_event_info_view_mut()
                    .set_event_bits(0x62, 0x20);
                self.system_signals_view_mut().set_sound_effect_2(27);
                self.dungeon_state_view_mut().set_door_open_counter(0x50);
                self.dungeon_state_view_mut()
                    .set_big_rock_starting_address(0x0d20);
                self.Overworld_DoMapUpdate32x32_B();
            }
        }
    }

    pub(super) fn GanonTowerEntrance_Func1(&mut self) {
        if self.frame_state().subsubmodule == 0 {
            self.system_signals_view_mut().set_sound_effect_1(0x2e);
            self.Palette_AnimGetMasterSword2();
        } else {
            self.PaletteFilter_BlindingWhite();
            if self.palette_filter_view().darkening_or_lightening_screen() == 0xff {
                self.palette_filter_view_mut().set_countdown(0xff);
                self.increment_subsubmodule();
            } else {
                self.Palette_AnimGetMasterSword3();
            }
        }
    }

    pub(super) fn Overworld_DwDeathMountainPaletteAnimation(&mut self) {
        if self.world_state_view().trigger_special_entrance() != 0 {
            return;
        }
        let sc = self.world_location_state().overworld_screen_index();
        if !matches!(sc, 0x43 | 0x45 | 0x47) {
            return;
        }

        let fc = self.frame_state().frame_counter;
        if matches!(fc, 5 | 44 | 90) {
            for i in 1..8 {
                copy_le_u16(
                    &mut self.ram,
                    MAIN_PALETTE_BUFFER + (0x30 + i) * 2,
                    AUX_PALETTE_BUFFER + (0x30 + i) * 2,
                );
                copy_le_u16(
                    &mut self.ram,
                    MAIN_PALETTE_BUFFER + (0x38 + i) * 2,
                    AUX_PALETTE_BUFFER + (0x38 + i) * 2,
                );
                copy_le_u16(
                    &mut self.ram,
                    MAIN_PALETTE_BUFFER + (0x48 + i) * 2,
                    AUX_PALETTE_BUFFER + (0x48 + i) * 2,
                );
                copy_le_u16(
                    &mut self.ram,
                    MAIN_PALETTE_BUFFER + (0x70 + i) * 2,
                    AUX_PALETTE_BUFFER + (0x70 + i) * 2,
                );
                copy_le_u16(
                    &mut self.ram,
                    MAIN_PALETTE_BUFFER + (0x78 + i) * 2,
                    AUX_PALETTE_BUFFER + (0x78 + i) * 2,
                );
            }
        } else if matches!(fc, 3 | 36 | 88) {
            if fc == 36 {
                self.system_signals_view_mut().set_sound_effect_1(54);
            }
            for i in 1..8 {
                self.palette_buffer_view_mut()
                    .set_main_color(0x30 + i, DARK_WORLD_PALETTE_ANIMATION_PHASE1[i - 1]);
                self.palette_buffer_view_mut()
                    .set_main_color(0x38 + i, DARK_WORLD_PALETTE_ANIMATION_PHASE1[i - 1 + 7]);
                self.palette_buffer_view_mut()
                    .set_main_color(0x48 + i, DARK_WORLD_PALETTE_ANIMATION_PHASE1[i - 1 + 14]);
                self.palette_buffer_view_mut()
                    .set_main_color(0x70 + i, DARK_WORLD_PALETTE_ANIMATION_PHASE1[i - 1 + 21]);
                self.palette_buffer_view_mut()
                    .set_main_color(0x78 + i, DARK_WORLD_PALETTE_ANIMATION_PHASE1[i - 1 + 28]);
            }
        }

        self.system_signals_view_mut().increment_cgram_update_flag();
        let mut yy = 32usize;
        if sc == 0x43 || sc == 0x45 {
            if self.overworld_event_info_view().event_info(0x43) & 0x20 != 0 {
                return;
            }
            yy = ((self.frame_state().frame_counter & 0x0c) as usize) * 2;
        }
        for i in 0..8 {
            self.palette_buffer_view_mut()
                .set_main_color(0x68 + i, DARK_WORLD_PALETTE_ANIMATION_PHASE2[yy + i]);
        }
    }

    pub(super) fn Module09_FadeBackInFromMosaic(&mut self) {
        self.Overworld_ResetMosaicDown();
        match self.frame_state().subsubmodule {
            0 => {
                let sc = self.world_location_state().overworld_screen_index() as usize;
                let bg = self.GetOverworldBgPalette(sc as u8);
                let spr = self.overworld_config_table_view().sprite_palette(sc);
                self.Overworld_LoadPalettes(bg, spr);
                self.OverworldMosaicTransition_LoadSpriteGraphicsAndSetMosaic();
            }
            1 => {
                self.Graphics_IncrementalVRAMUpload();
                self.ApplyPaletteFilter_bounce();
            }
            _ => {
                self.system_signals_view_mut().save_current_music_as_last();
                if self.world_location_state().overworld_screen_index() != 0x80
                    && self.world_location_state().overworld_screen_index() != 0x2a
                {
                    let m = self
                        .overworld_config_table_view()
                        .music(self.world_location_state().overworld_screen_index() as usize);
                    self.system_signals_view_mut()
                        .set_ambient_sound_effect(if m >> 4 != 0 { m >> 4 } else { 5 });
                    if !self.zelda_is_playing_music_track(m & 0x0f) {
                        self.system_signals_view_mut().set_music_control(m & 0x0f);
                    }
                }
                self.set_submodule(8);
                self.set_subsubmodule(0);
                if self.frame_state().main_module == 11 {
                    self.set_main_module(9);
                    self.set_submodule(31);
                    self.world_state_view_mut().set_ow_countdown_transition(12);
                }
            }
        }
    }

    pub(super) fn Overworld_Func1C(&mut self) {
        self.Overworld_ResetMosaicDown();
        match self.frame_state().subsubmodule {
            0 => self.OverworldMosaicTransition_LoadSpriteGraphicsAndSetMosaic(),
            1 => {
                self.Graphics_IncrementalVRAMUpload();
                self.ApplyPaletteFilter_bounce();
            }
            _ => {
                if self.world_location_state().overworld_screen_index() < 0x80 {
                    let music_control =
                        if self.world_location_state().overworld_screen_index() & 0x3f != 0 {
                            2
                        } else {
                            5
                        };
                    self.system_signals_view_mut()
                        .set_music_control(music_control);
                }
                self.set_submodule(8);
                self.set_subsubmodule(0);
            }
        }
    }

    pub(super) fn Overworld_StartMosaicTransition(&mut self) {
        self.ConditionalMosaicControl();
        match self.frame_state().subsubmodule {
            0 => {
                if self.world_location_state().overworld_screen_index() != 0x80 {
                    let music = self
                        .overworld_config_table_view()
                        .music(self.world_location_state().overworld_screen_index() as usize);
                    if !self.zelda_is_playing_music_track(music & 0x0f) {
                        self.system_signals_view_mut().set_music_control(0xf1);
                    }
                }
                self.ResetTransitionPropsAndAdvance_ResetInterface();
            }
            1 => self.ApplyPaletteFilter_bounce(),
            _ => {
                self.set_screen_brightness(0x80);
                self.set_subsubmodule(0);
                if u16::from(self.world_location_state().overworld_screen_index()) & 0x3f == 0 {
                    self.DecodeAnimatedSpriteTile_variable(0x1e);
                }
                if self.world_state_view().overworld_area_index() != 0
                    && self.frame_state().main_module != 11
                {
                    self.set_main_screen_layers(0x16);
                    self.set_sub_screen_layers(1);
                    self.palette_filter_view_mut()
                        .set_color_window_selection(0x82);
                    self.palette_filter_view_mut().set_color_math_control(0x20);
                    self.increment_submodule();
                    return;
                }
                if self.frame_state().submodule == 36 {
                    self.LoadOverworldFromSpecialOverworld();
                    if u16::from(self.world_location_state().overworld_screen_index()) & 0x3f == 0 {
                        self.DecodeAnimatedSpriteTile_variable(0x1e);
                    }
                }
                self.increment_submodule();
            }
        }
    }

    pub(super) fn OverworldMosaicTransition_LoadSpriteGraphicsAndSetMosaic(&mut self) {
        self.LoadNewSpriteGFXSet();
        self.set_screen_brightness(0x0f);
        self.set_hdma_enable_mask(0x80);
        let countdown = self.display_state().mosaic_target_level.wrapping_sub(1);
        self.palette_filter_view_mut().set_countdown(countdown);
        self.clear_mosaic_target_level();
        self.palette_filter_view_mut()
            .set_darkening_or_lightening_screen(2);
        self.increment_subsubmodule();
    }

    pub(super) fn Overworld_SetSongList(&mut self) {
        let mut r0 = 2;
        let mut y = 0xc0usize;
        if self.save_progress_view().progress_indicator() < 3 {
            y = 0x80;
            if self.inventory_state_view().sword_type() < 2 {
                r0 = 5;
                y = 0x40;
                if self.save_progress_view().progress_indicator() < 2 {
                    y = 0;
                }
            }
        }
        let music_sets = self
            .asset_raw(111)
            .expect("Overworld_SetSongList missing kOwMusicSets asset")
            .to_vec();
        self.overworld_config_table_view_mut()
            .copy_music_primary(&music_sets[y..y + 64]);
        let music_sets2 = self
            .asset_raw(112)
            .expect("Overworld_SetSongList missing kOwMusicSets2 asset")
            .to_vec();
        self.overworld_config_table_view_mut()
            .copy_music_secondary(&music_sets2);
        self.overworld_config_table_view_mut().set_music(128, r0);
    }

    pub(super) fn Overworld_Func2F(&mut self) {
        self.dungeon_state_view_mut()
            .set_bg2_tile_by_byte_pos(0x0720 as u16, 0x0212);
        self.Overworld_Memorize_Map16_Change(0x0720, 0x0212);
        self.overworld_draw_map16(0x0720, 0x0212);
        self.set_bg_vram_load_mode(1);
        self.set_submodule(0);
    }

    pub(super) fn OpenGargoylesDomain(&mut self) {
        self.overworld_draw_map16_persist(0x0d3e, 0x0e1b);
        self.overworld_draw_map16_persist(0x0d40, 0x0e1c);
        self.overworld_draw_map16_persist(0x0dbe, 0x0e1d);
        self.overworld_draw_map16_persist(0x0dc0, 0x0e1e);
        self.overworld_draw_map16_persist(0x0e3e, 0x0e1f);
        self.overworld_draw_map16_persist(0x0e40, 0x0e20);
        self.overworld_event_info_view_mut()
            .set_event_bits(0x58, 0x20);
        self.system_signals_view_mut().set_sound_effect_2(0x1b);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn CreatePyramidHole(&mut self) {
        self.overworld_draw_map16_persist(0x03bc, 0x0e3f);
        self.overworld_draw_map16_persist(0x03be, 0x0e40);
        self.overworld_draw_map16_persist(0x03c0, 0x0e41);
        self.overworld_draw_map16_persist(0x043c, 0x0e42);
        self.overworld_draw_map16_persist(0x043e, 0x0e43);
        self.overworld_draw_map16_persist(0x0440, 0x0e44);
        self.overworld_draw_map16_persist(0x04bc, 0x0e45);
        self.overworld_draw_map16_persist(0x04be, 0x0e46);
        self.overworld_draw_map16_persist(0x04c0, 0x0e47);
        self.system_signals_view_mut()
            .set_ambient_sound_effect_word(0x3515);
        self.overworld_event_info_view_mut()
            .set_event_bits(0x5b, 0x20);
        self.system_signals_view_mut().set_sound_effect_2(3);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn Overworld_AlterTileHardcore(&mut self, pos: u16, value: u16) {
        self.dungeon_state_view_mut()
            .set_bg2_tile_by_byte_pos(pos, value);
        self.overworld_draw_map16(pos, value);
    }

    pub(super) fn Overworld_CheckSpecialSwitchArea(&mut self) {
        let map8 = self.Overworld_GetMap16OfLink_Mult8();
        let a = map8[0] & 0x01ff;
        for i in (0..4).rev() {
            if SPECIAL_SWITCH_AREA_TILE_IDS[i] == a
                && SPECIAL_SWITCH_AREA_SCREENS[i]
                    == u16::from(self.world_location_state().overworld_screen_index())
            {
                self.set_dungeon_room(SPECIAL_SWITCH_AREA_EXITS[i]);
                self.world_state_view_mut()
                    .set_screen_transition_direction_bits(SPECIAL_SWITCH_AREA_DIRECTIONS[i]);
                let direction = SPECIAL_SWITCH_AREA_DIRECTIONS[i];
                self.player_state_view_mut().set_direction(direction);
                let trans = self.DirToEnum(direction as i32) as u16;
                self.world_state_view_mut()
                    .set_screen_transition_word(trans);
                self.world_state_view_mut()
                    .set_transition_dir_enum(trans as u8);
                self.set_submodule(23);
                self.set_main_module(11);
                break;
            }
        }
    }

    pub(super) fn ScrollAndCheckForSOWExit(&mut self) {
        if self
            .world_state_view()
            .has_screen_transition_direction_bits()
        {
            self.OverworldHandleMapScroll();
        }

        let map8 = self.Overworld_GetMap16OfLink_Mult8();
        let a = map8[0] & 0x01ff;
        if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some()
            && matches!(
                u16::from(self.world_location_state().overworld_screen_index()),
                0x0080 | 0x0081
            )
        {
            let xc = self.player_state_view().x().wrapping_add(8) >> 3;
            let yc = self.player_state_view().y().wrapping_add(12);
            let pos = self.overworld_bg2_byte_pos(xc, yc);
            println!(
                "spexit-check frame={} screen=0x{:04x} x=0x{:04x} y=0x{:04x} base=0x{:04x}/0x{:04x} mask=0x{:04x}/0x{:04x} pos=0x{:04x} map8=0x{:04x} dirbits2=0x{:02x} sub={} subsub={}",
                self.frame_state().frame_counter,
                u16::from(self.world_location_state().overworld_screen_index()),
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.world_state_view().overworld_offset_base_x(),
                self.world_state_view().overworld_offset_base_y(),
                self.world_state_view().overworld_offset_mask_x(),
                self.world_state_view().overworld_offset_mask_y(),
                pos,
                a,
                self.world_state_view().screen_transition_direction_bits(),
                self.frame_state().submodule,
                self.frame_state().subsubmodule,
            );
        }
        for i in (0..3).rev() {
            if SPECIAL_SWITCH_AREA_B_TILE_IDS[i] == a
                && SPECIAL_SWITCH_AREA_B_SCREENS[i]
                    == u16::from(self.world_location_state().overworld_screen_index())
            {
                let direction = SPECIAL_SWITCH_AREA_B_DIRECTIONS[i];
                self.player_state_view_mut().set_direction(direction);
                let trans = self.DirToEnum(direction as i32) as u16;
                self.world_state_view_mut()
                    .set_screen_transition_word(trans);
                self.world_state_view_mut()
                    .set_transition_dir_enum(trans as u8);
                self.set_submodule(36);
                self.set_subsubmodule(0);
                self.set_dungeon_room_index(0);
                if std::env::var_os("ZELDA3_REPLAY_SPEXIT_DUMP").is_some() {
                    println!(
                        "spexit-hit frame={} i={} screen=0x{:04x} map8=0x{:04x} dir=0x{:02x} trans=0x{:04x} x=0x{:04x} y=0x{:04x}",
                        self.frame_state().frame_counter,
                        i,
                        u16::from(self.world_location_state().overworld_screen_index()),
                        a,
                        self.player_state_view().direction(),
                        self.world_state_view().screen_transition_word(),
                        self.player_state_view().x(),
                        self.player_state_view().y(),
                    );
                }
                break;
            }
        }
    }

    pub(super) fn Overworld_GetMap16OfLink_Mult8(&self) -> [u16; 4] {
        let xc = self.player_state_view().x().wrapping_add(8) >> 3;
        let yc = self.player_state_view().y().wrapping_add(12);
        let pos = self.overworld_bg2_byte_pos(xc, yc);
        let map16 = self.dungeon_state_view().bg2_tile_by_byte_pos(pos) as usize * 4;
        let map8 = self
            .asset_raw(70)
            .expect("Overworld_GetMap16OfLink_Mult8 missing kMap16ToMap8 asset");
        [
            u16::from(map8[map16 * 2]) | (u16::from(map8[map16 * 2 + 1]) << 8),
            u16::from(map8[(map16 + 1) * 2]) | (u16::from(map8[(map16 + 1) * 2 + 1]) << 8),
            u16::from(map8[(map16 + 2) * 2]) | (u16::from(map8[(map16 + 2) * 2 + 1]) << 8),
            u16::from(map8[(map16 + 3) * 2]) | (u16::from(map8[(map16 + 3) * 2 + 1]) << 8),
        ]
    }

    pub(super) fn overworld_get_link_map16_coords(&self, xy: &mut Point16U) -> u16 {
        let (pos, x, y) = self.overworld_get_link_map16_coords_result();
        xy.x = x;
        xy.y = y;
        pos
    }

    pub(super) fn overworld_smash_rock_pile(
        &mut self,
        down_one_tile: bool,
        pt: &mut Point16U,
    ) -> i32 {
        if let Some((attr, x, y)) = self.overworld_smash_rock_pile_result(down_one_tile) {
            pt.x = x;
            pt.y = y;
            attr as i32
        } else {
            -1
        }
    }

    pub(super) fn overworld_lifting_small_obj(
        &mut self,
        a: u16,
        pos: u16,
        y: u16,
        pt: Point16U,
    ) -> u8 {
        self.overworld_lifting_small_obj_impl(a, pos, y, pt.x, pt.y)
    }

    pub(super) fn smash_rock_pile_from_lift(
        &mut self,
        a: u16,
        pos: u16,
        y: u16,
        pt: Point16U,
    ) -> u8 {
        self.smash_rock_pile_from_lift_impl(a, pos, y as usize, pt.x, pt.y)
    }

    pub(super) fn sprite_load_graphics_properties_light_world_only(&mut self) {
        let i = if self.save_progress_view().progress_indicator() < 2 {
            0
        } else if self.save_progress_view().progress_indicator() != 3 {
            1
        } else {
            2
        };
        let gfx = self
            .asset_raw(161)
            .expect(
                "Sprite_LoadGraphicsProperties_light_world_only missing kOverworldSpriteGfx asset",
            )
            .to_vec();
        let palettes = self
            .asset_raw(162)
            .expect(
                "Sprite_LoadGraphicsProperties_light_world_only missing kOverworldSpritePalettes asset",
            )
            .to_vec();
        copy_asset_range(&mut self.ram, OVERWORLD_SPRITE_GFX, &gfx, i * 64, 64);
        copy_asset_range(
            &mut self.ram,
            OVERWORLD_SPRITE_PALETTES,
            &palettes,
            i * 64,
            64,
        );
    }

    pub(super) fn sprite_load_graphics_properties(&mut self) {
        let gfx = self
            .asset_raw(161)
            .expect("Sprite_LoadGraphicsProperties missing kOverworldSpriteGfx asset")
            .to_vec();
        let palettes = self
            .asset_raw(162)
            .expect("Sprite_LoadGraphicsProperties missing kOverworldSpritePalettes asset")
            .to_vec();
        copy_asset_range(&mut self.ram, OVERWORLD_SPRITE_GFX + 64, &gfx, 0xc0, 64);
        copy_asset_range(
            &mut self.ram,
            OVERWORLD_SPRITE_PALETTES + 64,
            &palettes,
            0xc0,
            64,
        );
        self.sprite_load_graphics_properties_light_world_only();
    }
}

fn copy_asset_range(ram: &mut [u8], dst: usize, data: &[u8], src: usize, len: usize) {
    ram[dst..dst + len].copy_from_slice(&data[src..src + len]);
}

impl ZeldaState {
    pub(super) fn decompress_enemy_damage_subclasses(&mut self) {
        let data = self
            .asset_raw(56)
            .expect("decompress_enemy_damage_subclasses missing kEnemyDamageData asset")
            .to_vec();
        self.overworld_map16_decode_view_mut()
            .copy_source_from(&data);
        for i in (0..0x1000).step_by(2) {
            let t = self.overworld_map16_decode_view().source_byte(i >> 1);
            let mut enemy_damage = self.enemy_damage_data_view_mut();
            enemy_damage.set_entry(i, t >> 4);
            enemy_damage.set_entry(i + 1, t & 0x0f);
        }
    }

    pub(super) fn conditional_mosaic_control(&mut self) {
        if self.palette_filter_view().countdown() & 1 != 0 {
            self.increment_mosaic_level_by(0x10);
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(7);
    }

    pub(super) fn overworld_alter_weathervane(&mut self) {
        self.dungeon_state_view_mut().set_door_open_counter(0x68);
        self.dungeon_state_view_mut()
            .set_big_rock_starting_address(0x0c3e);
        self.overworld_do_map_update32x32_b();
        self.overworld_draw_map16_persist(0x0c42, 0x0e21);
        self.overworld_draw_map16_persist(0x0cc2, 0x0e25);

        self.overworld_event_info_view_mut()
            .set_event_bits(0x18, 0x20);
        self.set_bg_vram_load_mode(1);
    }

    fn overworld_do_map_update32x32_b(&mut self) {
        self.overworld_do_map_update32x32();
        self.dungeon_state_view_mut().clear_door_open_counter_low();
    }

    pub(super) fn Overworld_DoMapUpdate32x32_B(&mut self) {
        self.overworld_do_map_update32x32_b();
    }

    pub(super) fn Overworld_DoMapUpdate32x32_conditional(&mut self) {
        if self.dungeon_state_view().door_open_counter_low() & 7 != 0 {
            self.dungeon_state_view_mut()
                .increment_door_open_counter_low();
        } else {
            self.overworld_do_map_update32x32();
        }
    }

    pub(super) fn Module09_09_OpenBigDoorFromExiting(&mut self) {
        if self.world_state_view().door_animation_step() != 3 {
            self.Overworld_DoMapUpdate32x32_conditional();
            return;
        }
        self.world_state_view_mut().set_ow_countdown_transition(36);
        self.world_state_view_mut()
            .clear_screen_transition_direction_bits();
        self.increment_submodule();
    }

    pub(super) fn Module09_0C_OpenBigDoor(&mut self) {
        if self.world_state_view().door_animation_step() != 3 {
            self.Overworld_DoMapUpdate32x32_conditional();
            return;
        }
        self.set_submodule(0);
        self.set_subsubmodule(0);
        self.world_state_view_mut()
            .clear_screen_transition_direction_bits();
    }

    pub(super) fn Module09_0A_WalkFromExiting_FacingDown(&mut self) {
        self.player_state_view_mut().set_last_direction(4);
        self.link_handle_moving_animation_full_long_entry();
        let link_y = self.player_state_view().y().wrapping_add(1);
        self.player_state_view_mut().set_y(link_y);
        if self
            .world_state_view_mut()
            .decrement_ow_countdown_transition()
            != 0
        {
            return;
        }
        self.set_submodule(0);
        let link_y = self.player_state_view().y().wrapping_add(3);
        self.player_state_view_mut().set_y(link_y);
        self.player_state_view_mut().set_y_velocity(3);
        self.Overworld_OperateCameraScroll();
        if self
            .world_state_view()
            .has_screen_transition_direction_bits()
        {
            self.OverworldHandleMapScroll();
        }
    }

    pub(super) fn Module09_0B_WalkFromExiting_FacingUp(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        let link_y = self.player_state_view().y().wrapping_sub(1);
        self.player_state_view_mut().set_y(link_y);
        if self
            .world_state_view_mut()
            .decrement_ow_countdown_transition()
            == 0
        {
            self.set_submodule(0);
        }
    }

    fn overworld_do_map_update32x32(&mut self) {
        const DOOR_ANIM_TILES: [u16; 56] = [
            0x0da8, 0x0da9, 0x0daa, 0x0dab, 0x0dac, 0x0dad, 0x0dae, 0x0daf, 0x0db0, 0x0db1, 0x0db2,
            0x0db3, 0x0db6, 0x0db7, 0x0db8, 0x0db9, 0x0dba, 0x0dbb, 0x0dbc, 0x0dbd, 0x0dcd, 0x0dce,
            0x0dcf, 0x0dd0, 0x0dd3, 0x0dd4, 0x0dd5, 0x0dd6, 0x0dd7, 0x0dd8, 0x0dd9, 0x0dda, 0x0dd1,
            0x0dd2, 0x0dd3, 0x0dd4, 0x0dd1, 0x0dd2, 0x0dd7, 0x0dd8, 0x0918, 0x0919, 0x091a, 0x091b,
            0x0ddb, 0x0ddc, 0x0ddd, 0x0dde, 0x0dd1, 0x0dd2, 0x0ddb, 0x0ddc, 0x0e21, 0x0e22, 0x0e23,
            0x0e24,
        ];

        let i = self.memorized_tile_view().count() as usize;
        let j = (self.dungeon_state_view().door_open_counter() >> 1) as usize;
        let base = self.dungeon_state_view().big_rock_starting_address();
        let entries = [
            (base, DOOR_ANIM_TILES[j]),
            (base.wrapping_add(2), DOOR_ANIM_TILES[j + 1]),
            (base.wrapping_add(0x80), DOOR_ANIM_TILES[j + 2]),
            (base.wrapping_add(0x82), DOOR_ANIM_TILES[j + 3]),
        ];
        for (n, (pos, tile)) in entries.into_iter().enumerate() {
            self.memorized_tile_view_mut()
                .set_entry_addr(i + n * 2, pos);
            self.memorized_tile_view_mut()
                .set_entry_value(i + n * 2, tile);
            self.overworld_draw_map16_persist(pos, tile);
        }
        let upload = self.display_state().vram_upload_cursor_usize();
        self.vram_upload_data_view_mut().set_word(upload, 0xffff);
        self.memorized_tile_view_mut().set_count((i + 8) as u16);
        let step = self.world_state_view().door_animation_step().wrapping_add(
            if self.dungeon_state_view().door_open_counter() == 32 {
                2
            } else {
                1
            },
        );
        self.world_state_view_mut()
            .set_door_animation_step_word(step);
        self.set_bg_vram_load_mode(1);
        self.dungeon_state_view_mut()
            .increment_door_open_counter_low();
    }

    fn overworld_draw_map16_persist(&mut self, pos: u16, value: u16) {
        self.dungeon_state_view_mut()
            .set_bg2_tile_by_byte_pos(pos, value);
        self.overworld_draw_map16(pos, value);
    }

    fn overworld_draw_map16(&mut self, pos: u16, value: u16) {
        let vram_pos = Self::overworld_find_map16_vram_address(pos);
        let upload = self.display_state().vram_upload_cursor_usize();
        let dst = self.vram_upload_data_view().data_address(upload);
        let src = value as usize * 4;
        let map8 = self
            .asset_raw(70)
            .expect("Overworld_DrawMap16 missing kMap16ToMap8 asset");
        let tile0 = u16::from(map8[src * 2]) | (u16::from(map8[src * 2 + 1]) << 8);
        let tile1 = u16::from(map8[(src + 1) * 2]) | (u16::from(map8[(src + 1) * 2 + 1]) << 8);
        let tile2 = u16::from(map8[(src + 2) * 2]) | (u16::from(map8[(src + 2) * 2 + 1]) << 8);
        let tile3 = u16::from(map8[(src + 3) * 2]) | (u16::from(map8[(src + 3) * 2 + 1]) << 8);
        self.vram_upload_data_view_mut().write_map16_update_packet(
            dst,
            vram_pos,
            [tile0, tile1, tile2, tile3],
        );
        self.vram_upload_data_view_mut()
            .set_offset((upload + 16) as u16);
    }

    fn overworld_find_map16_vram_address(addr: u16) -> u16 {
        (if addr & 0x3f >= 0x20 { 0x0400 } else { 0 })
            + (if addr & 0x0fff >= 0x0800 { 0x0800 } else { 0 })
            + (addr & 0x001f)
            + ((addr & 0x0780) >> 1)
    }

    pub(super) fn overworld_bomb_tiles32x32(&mut self, mut x: u16, mut y: u16) {
        x = x.wrapping_sub(23) & !7;
        y = y.wrapping_sub(20) & !7;

        for _ in (1..=3).rev() {
            let mut xt = x;
            for _ in (1..=3).rev() {
                self.overworld_bomb_tile(xt, y);
                xt = xt.wrapping_add(16);
            }
            y = y.wrapping_add(16);
        }
        self.world_state_view_mut()
            .set_overworld_bomb_tile_sweep_x(x);
        self.world_state_view_mut()
            .set_overworld_bomb_tile_sweep_y_end(y);
    }

    fn overworld_bomb_tile(&mut self, x: u16, y: u16) {
        let pos = self.overworld_bg2_byte_pos(x >> 3, y);

        if self.follower_state_view().indicator() != 13 {
            let a = self.dungeon_state_view().bg2_tile_by_byte_pos(pos);
            let (k, j) = if a == 0x0036 {
                (2, 0x0dc7)
            } else if a == 0x072a {
                (4, 0x0dc8)
            } else if a == 0x037e {
                (3, 0x0dc5)
            } else {
                self.overworld_bomb_tile_label_a(pos);
                return;
            };
            let mut a = self.overworld_reveal_secret_for_smash(pos);
            if a == 0 {
                a = j;
            }
            self.dungeon_state_view_mut()
                .set_bg2_tile_by_byte_pos(pos, a);
            self.overworld_memorize_map16_change_for_smash(pos, a);
            self.overworld_draw_map16_for_smash(pos, a);
            self.sprite_spawn_immediately_smashed_terrain(k, x & !7, y & !7);
            self.set_bg_vram_load_mode(1);
            return;
        }

        self.overworld_bomb_tile_label_a(pos);
    }

    fn overworld_bomb_tile_label_a(&mut self, pos: u16) {
        let a = self.overworld_reveal_secret_for_smash(pos);
        if a == 0x0db4 {
            self.dungeon_state_view_mut()
                .set_bg2_tile_by_byte_pos(pos, a);
            self.overworld_memorize_map16_change_for_smash(pos, a);
            self.overworld_draw_map16_for_smash(pos, a);

            self.dungeon_state_view_mut()
                .set_bg2_tile_by_byte_pos(pos.wrapping_add(2), 0x0db5);
            self.overworld_memorize_map16_change_for_smash(pos, 0x0db5);
            self.overworld_draw_map16_for_smash(pos.wrapping_add(2), 0x0db5);
            self.set_bg_vram_load_mode(1);
            let screen = self.world_location_state().overworld_screen_index() as usize;
            self.overworld_event_info_view_mut()
                .set_event_bits(screen, 2);
        }
    }

    pub(super) fn Overworld_HandleOverlaysAndBombDoors(&mut self) {
        let screen = u16::from(self.world_location_state().overworld_screen_index()) as usize;
        if screen == 0x33 {
            self.dungeon_state_view_mut().set_bg2_tile(340, 0x020f);
        } else if screen == 0x2f {
            self.dungeon_state_view_mut().set_bg2_tile(1497, 0x020f);
        }

        let screen_byte = self.world_location_state().overworld_screen_index() as usize;
        if screen_byte < 0x80
            && self.overworld_event_info_view().event_info(screen_byte) & 0x20 != 0
        {
            self.Overworld_LoadEventOverlay();
        }
        if self.overworld_event_info_view().event_info(screen_byte) & 2 != 0 {
            let pos = (SECONDARY_OVERLAY_BY_OVERWORLD_SCREEN[screen] >> 1) as usize;
            self.replay_trace_door_overlay("secondary-before", (pos << 1) as u16);
            self.dungeon_state_view_mut().set_bg2_tile(pos, 0x0db4);
            self.dungeon_state_view_mut().set_bg2_tile(pos + 1, 0x0db5);
            self.replay_trace_door_overlay("secondary-after", (pos << 1) as u16);
        }
    }

    pub(super) fn Overworld_LoadEventOverlay(&mut self) {
        match u16::from(self.world_location_state().overworld_screen_index()) {
            0..=2 => {
                for x in 11..=14 {
                    self.write_bg2_xy(x, 16, 0x0e32);
                }
                self.write_bg2_xy(11, 17, 0x0e32);
                self.write_bg2_xy(14, 17, 0x0e32);
                self.write_bg2_xy(12, 17, 0x0e33);
                self.write_bg2_xy(13, 17, 0x0e34);
                self.write_event_overlay_pairs(&[
                    (11, 18, 0x0e35),
                    (12, 18, 0x0e36),
                    (13, 18, 0x0e37),
                    (14, 18, 0x0e38),
                    (11, 19, 0x0e39),
                    (12, 19, 0x0e3a),
                    (13, 19, 0x0e3b),
                    (14, 19, 0x0e3c),
                    (12, 20, 0x0e3d),
                    (13, 20, 0x0e3e),
                ]);
            }
            3..=7 => self.write_bg2_xy(16, 14, 0x0212),
            8..=19 => self.write_event_overlay_2x2(3, 10),
            20 => self.write_event_overlay_pairs(&[
                (25, 10, 0x0dd1),
                (26, 10, 0x0dd2),
                (25, 11, 0x0dd7),
                (26, 11, 0x0dd8),
                (25, 12, 0x0dd9),
                (26, 12, 0x0dda),
            ]),
            21..=25 | 32 | 33 => self.write_event_overlay_pairs(&[
                (31, 24, 0x0e21),
                (33, 24, 0x0e21),
                (32, 24, 0x0e22),
                (31, 25, 0x0e23),
                (32, 25, 0x0e24),
                (33, 25, 0x0e25),
            ]),
            26..=28 | 35 | 36 => self.write_event_overlay_pairs(&[
                (30, 39, 0x0dc1),
                (31, 39, 0x0dc2),
                (30, 40, 0x0dbe),
                (31, 40, 0x0dbf),
                (32, 39, 0x0dc2),
                (33, 39, 0x0dc3),
                (32, 40, 0x0dbf),
                (33, 40, 0x0dc0),
            ]),
            29..=31 | 34 | 37..=43 | 107 => self.write_event_overlay_2x2(24, 6),
            44..=49 | 56 | 57 => self.write_event_overlay_2x2(44, 6),
            50..=55 | 119 => self.write_event_overlay_2x2(6, 8),
            58 => self.write_event_overlay_2x2(15, 20),
            59 | 123 => self.write_event_overlay_pairs(&[
                (22, 7, 0x0ddf),
                (18, 8, 0x0ddf),
                (16, 9, 0x0ddf),
                (15, 10, 0x0ddf),
                (14, 12, 0x0ddf),
                (26, 14, 0x0ddf),
                (23, 7, 0x0de0),
                (17, 9, 0x0de0),
                (24, 7, 0x0de1),
                (28, 8, 0x0de1),
                (29, 9, 0x0de1),
                (21, 11, 0x0de1),
                (29, 14, 0x0de1),
                (19, 8, 0x0de2),
                (20, 8, 0x0de2),
                (21, 8, 0x0de2),
                (25, 8, 0x0de2),
                (26, 8, 0x0de2),
                (27, 8, 0x0de2),
                (22, 8, 0x0de3),
                (18, 9, 0x0de3),
                (16, 10, 0x0de3),
                (15, 12, 0x0de3),
                (23, 8, 0x0de4),
                (19, 9, 0x0de4),
                (20, 9, 0x0de4),
                (24, 9, 0x0de4),
                (27, 9, 0x0de4),
                (17, 10, 0x0de4),
                (18, 10, 0x0de4),
                (19, 10, 0x0de4),
                (28, 10, 0x0de4),
                (16, 11, 0x0de4),
                (17, 11, 0x0de4),
                (18, 11, 0x0de4),
                (19, 11, 0x0de4),
                (16, 12, 0x0de4),
                (17, 12, 0x0de4),
                (15, 13, 0x0de4),
                (16, 13, 0x0de4),
                (15, 14, 0x0de4),
                (16, 14, 0x0de4),
                (19, 16, 0x0de4),
                (19, 17, 0x0de4),
                (20, 17, 0x0de4),
                (19, 18, 0x0de4),
                (24, 8, 0x0de5),
                (28, 9, 0x0de5),
                (20, 11, 0x0de5),
                (21, 12, 0x0de5),
                (21, 9, 0x0de6),
                (25, 9, 0x0de6),
                (20, 10, 0x0de6),
                (28, 11, 0x0de6),
                (21, 17, 0x0de6),
                (20, 18, 0x0de6),
                (22, 9, 0x0de7),
                (24, 10, 0x0de7),
                (15, 15, 0x0de7),
                (16, 15, 0x0de7),
                (19, 19, 0x0de7),
                (28, 19, 0x0de7),
                (23, 9, 0x0de8),
                (26, 9, 0x0de8),
                (27, 10, 0x0de8),
                (17, 15, 0x0de8),
                (18, 16, 0x0de8),
                (23, 10, 0x0de9),
                (26, 10, 0x0de9),
                (14, 15, 0x0de9),
                (17, 16, 0x0de9),
                (26, 18, 0x0de9),
                (27, 19, 0x0de9),
                (29, 10, 0x0dea),
                (28, 12, 0x0dea),
                (28, 13, 0x0dea),
                (29, 18, 0x0dea),
                (15, 11, 0x0deb),
                (27, 11, 0x0deb),
                (27, 12, 0x0deb),
                (14, 13, 0x0deb),
                (27, 13, 0x0deb),
                (14, 14, 0x0deb),
                (18, 17, 0x0deb),
                (18, 18, 0x0deb),
                (18, 12, 0x0dec),
                (17, 13, 0x0dec),
                (19, 12, 0x0ded),
                (20, 12, 0x0dee),
                (18, 13, 0x0def),
                (27, 15, 0x0def),
                (19, 13, 0x0df0),
                (19, 14, 0x0df0),
                (20, 14, 0x0df0),
                (21, 14, 0x0df0),
                (21, 15, 0x0df0),
                (27, 16, 0x0df0),
                (28, 16, 0x0df0),
                (20, 13, 0x0df1),
                (28, 15, 0x0df1),
                (21, 13, 0x0df2),
                (17, 14, 0x0df3),
                (18, 15, 0x0df3),
                (20, 16, 0x0df3),
                (18, 14, 0x0df4),
                (19, 15, 0x0df5),
                (20, 15, 0x0df6),
                (27, 17, 0x0df6),
                (26, 15, 0x0df7),
                (29, 15, 0x0df8),
                (21, 16, 0x0df9),
                (26, 16, 0x0dfa),
                (29, 16, 0x0dfb),
                (26, 17, 0x0dfc),
                (28, 17, 0x0dfd),
                (29, 17, 0x0dfe),
                (27, 18, 0x0dff),
                (28, 18, 0x0e00),
                (21, 10, 0x0e01),
                (25, 10, 0x0e01),
                (21, 18, 0x0e01),
                (29, 11, 0x0e02),
                (20, 19, 0x0e02),
                (29, 19, 0x0e02),
                (18, 19, 0x0e03),
                (27, 14, 0x0e04),
                (28, 14, 0x0e05),
            ]),
            60..=65 | 72 | 73 => self.write_event_overlay_pairs(&[
                (8, 11, 0x0e13),
                (11, 11, 0x0e14),
                (8, 12, 0x0e15),
                (9, 12, 0x0e16),
                (10, 12, 0x0e17),
                (11, 12, 0x0e18),
                (9, 13, 0x0e19),
                (10, 13, 0x0e1a),
                (9, 16, 0x0e06),
                (10, 16, 0x0e06),
                (8, 14, 0x0e07),
                (8, 15, 0x0e07),
                (9, 14, 0x0e08),
                (9, 15, 0x0e08),
                (10, 14, 0x0e09),
                (10, 15, 0x0e09),
                (11, 14, 0x0e0a),
                (11, 15, 0x0e0a),
            ]),
            66..=68 | 75 | 76 => self.write_event_overlay_pairs(&[
                (47, 8, 0x0e96),
                (48, 8, 0x0e97),
                (47, 9, 0x0e9c),
                (47, 10, 0x0e9c),
                (48, 9, 0x0e9d),
                (48, 10, 0x0e9d),
                (47, 11, 0x0e9a),
                (48, 11, 0x0e9b),
            ]),
            69 | 70 | 77 | 78 => self.write_event_overlay_2x2(52, 16),
            71 => self.write_event_overlay_pairs(&[
                (15, 19, 0x0e78),
                (16, 19, 0x0e79),
                (17, 19, 0x0e7a),
                (18, 19, 0x0e7b),
                (15, 20, 0x0e7c),
                (16, 20, 0x0e7d),
                (17, 20, 0x0e7e),
                (18, 20, 0x0e7f),
                (15, 21, 0x0e80),
                (16, 21, 0x0e81),
                (17, 21, 0x0e82),
                (18, 21, 0x0e83),
                (15, 22, 0x0e84),
                (16, 22, 0x0e85),
                (17, 22, 0x0e86),
                (18, 22, 0x0e87),
            ]),
            74 | 79..=89 | 96 | 97 => self.write_event_overlay_pairs(&[
                (31, 26, 0x0e1b),
                (32, 26, 0x0e1c),
                (31, 27, 0x0e1d),
                (32, 27, 0x0e1e),
                (31, 28, 0x0e1f),
                (32, 28, 0x0e20),
            ]),
            90..=92 | 99 | 100 => self.write_event_overlay_pairs(&[
                (30, 7, 0x0e3f),
                (31, 7, 0x0e40),
                (32, 7, 0x0e41),
                (30, 8, 0x0e42),
                (31, 8, 0x0e43),
                (32, 8, 0x0e44),
                (30, 9, 0x0e45),
                (31, 9, 0x0e46),
                (32, 9, 0x0e47),
            ]),
            93..=95 | 102 | 103 => self.write_event_overlay_pairs(&[
                (51, 3, 0x0e31),
                (53, 4, 0x0e2d),
                (53, 5, 0x0e2e),
                (53, 6, 0x0e2f),
            ]),
            98 => self.write_event_overlay_2x2(16, 26),
            101 | 104..=113 | 120 | 121 => self.write_event_overlay_pairs(&[
                (17, 10, 0x0e64),
                (18, 10, 0x0e65),
                (19, 10, 0x0e66),
                (20, 10, 0x0e67),
                (17, 11, 0x0e68),
                (18, 11, 0x0e69),
                (19, 11, 0x0e6a),
                (20, 11, 0x0e6b),
                (17, 12, 0x0e6c),
                (18, 12, 0x0e6d),
                (19, 12, 0x0e6e),
                (20, 12, 0x0e6f),
                (17, 13, 0x0e70),
                (18, 13, 0x0e71),
                (19, 13, 0x0e72),
                (20, 13, 0x0e73),
                (17, 14, 0x0e74),
                (18, 14, 0x0e75),
                (19, 14, 0x0e76),
                (20, 14, 0x0e77),
            ]),
            // C Overworld_LoadEventOverlay asserts for these invalid screens.
            114..=118 | 122 | 124..=127 => panic!("Overworld_LoadEventOverlay invalid screen"),
            _ => {}
        }
    }

    fn write_event_overlay_2x2(&mut self, x: usize, y: usize) {
        self.write_bg2_xy(x, y, 0x0918);
        self.write_bg2_xy(x + 1, y, 0x0919);
        self.write_bg2_xy(x, y + 1, 0x091a);
        self.write_bg2_xy(x + 1, y + 1, 0x091b);
    }

    fn write_event_overlay_pairs(&mut self, entries: &[(usize, usize, u16)]) {
        for &(x, y, value) in entries {
            self.write_bg2_xy(x, y, value);
        }
    }

    fn write_bg2_xy(&mut self, x: usize, y: usize, value: u16) {
        self.dungeon_state_view_mut()
            .set_bg2_tile(y * 64 + x, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_overworld_music_selection_preserves_f2_goto_setsong_path() {
        let (xt, ow_anim_tiles) = pre_overworld_music_selection(0x6c, 0x1c, 0xf2, 2, 0, 0x40, 1);

        assert_eq!(xt, 0xf3);
        assert_eq!(ow_anim_tiles, 0x5a);
    }

    #[test]
    fn pre_overworld_music_selection_applies_darkworld_override_without_f2() {
        let (xt, ow_anim_tiles) = pre_overworld_music_selection(0x6c, 0x1c, 0x09, 2, 0, 0x40, 1);

        assert_eq!(xt, 9);
        assert_eq!(ow_anim_tiles, 0x5a);
    }

    #[test]
    fn turtle_rock_vram_common_terminates_nmi_upload_data() {
        let mut state = ZeldaState::new();
        state.vram_upload_data_view_mut().set_byte(6, 0);
        state.ram[UVRAM_DATA_OVERWORLD + 6] = 0;

        state.turtle_rock_vram_common(0x10);

        assert_eq!(state.vram_upload_data_view().byte(6), 0xff);
        assert_eq!(state.ram[UVRAM_DATA_OVERWORLD + 6], 0);
    }
}
