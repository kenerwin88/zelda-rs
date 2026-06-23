pub(super) const DOOR_ANIMATION_STEP_INDICATOR_OVERWORLD: usize = 0x0690;
pub(super) const OVERWORLD_TRANSITION_DIR_ENUM: usize = 0x069c;
pub(super) const OVERWORLD_PEG_PUZZLE_PROGRESS: usize = 0x04c8;
pub(super) const BIG_KEY_DOOR_MESSAGE_TRIGGERED_OVERWORLD: usize = 0x04b8;
pub(super) const TRIGGER_SPECIAL_ENTRANCE_OVERWORLD: usize = 0x04c6;
pub(super) const OVERWORLD_BOMB_TILE_SWEEP_X: usize = 0x0486;
pub(super) const OVERWORLD_BOMB_TILE_SWEEP_Y_END: usize = 0x0488;
pub(super) const MAPBAK_PALETTE_OVERWORLD: usize = 0x1dd80;
pub(super) const MAP16_LOAD_SRC_OFF_OVERWORLD: usize = 0x0084;
pub(super) const MAP16_LOAD_DST_OFF_OVERWORLD: usize = 0x0086;
// NES_Ver2: YWRITE, vertical unit position used while emitting Map16 stripes.
pub(super) const MAP16_LOAD_Y_UNIT_OVERWORLD: usize = 0x0088;
pub(super) const WORD_7F4000_OVERWORLD: usize = 0x14000;
pub(super) const UVRAM_DATA_OVERWORLD: usize = 0x1100;
pub(super) const OVERWORLD_MAP16_DECODE_SRC: usize = 0x14000;
pub(super) const OVERWORLD_DECOMP_BUFFER: usize = 0x14400;
pub(super) const MAP16_DECODE_0_OVERWORLD: usize = 0x14400;
pub(super) const MAP16_DECODE_1_OVERWORLD: usize = 0x14410;
pub(super) const MAP16_DECODE_2_OVERWORLD: usize = 0x14420;
pub(super) const MAP16_DECODE_3_OVERWORLD: usize = 0x14430;
pub(super) const MAP16_DECODE_LAST_OVERWORLD: usize = 0x14440;
pub(super) const MAP16_DECODE_WORK_WORD_OVERWORLD: usize = 0x14442;
pub(super) const DUNG_REPLACEMENT_TILE_STATE_OVERWORLD: usize = 0x0500;
pub(super) const ORANGE_BLUE_BARRIER_STATE_OVERWORLD: usize = 0x0c172;
pub(super) const SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF: usize = 0x0c174;
pub(super) const SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT: usize = 0x0c176;
pub(super) const OVERWORLD_AREA_INDEX_OVERWORLD: usize = 0x040a;
pub(super) const OVERWORLD_SCREEN_TRANS_DIR_BITS_OVERWORLD: usize = 0x0410;
pub(super) const OVERWORLD_SCROLL_UP_COUNTER_OVERWORLD: usize = 0x0624;
pub(super) const OVERWORLD_SCROLL_DOWN_COUNTER_OVERWORLD: usize = 0x0626;
pub(super) const OVERWORLD_SCROLL_LEFT_COUNTER_OVERWORLD: usize = 0x0628;
pub(super) const OVERWORLD_SCROLL_RIGHT_COUNTER_OVERWORLD: usize = 0x062a;
pub(super) const OW_COUNTDOWN_TRANSITION_OVERWORLD: usize = 0x069a;
pub(super) const OVERWORLD_OFFSET_BASE_Y_OVERWORLD: usize = 0x0708;
pub(super) const OVERWORLD_OFFSET_MASK_Y_OVERWORLD: usize = 0x070a;
pub(super) const OVERWORLD_OFFSET_BASE_X_OVERWORLD: usize = 0x070c;
pub(super) const OVERWORLD_OFFSET_MASK_X_OVERWORLD: usize = 0x070e;
pub(super) const OVERWORLD_AREA_INDEX_SPEXIT_OVERWORLD: usize = 0x0c100;
pub(super) const TM_COPY_SPEXIT_OVERWORLD: usize = 0x0c102;
pub(super) const OVERWORLD_SCREEN_INDEX_SPEXIT_OVERWORLD: usize = 0x0c10c;
pub(super) const MAP16_LOAD_SRC_OFF_SPEXIT_OVERWORLD: usize = 0x0c10e;
pub(super) const CAMERA_Y_COORD_SCROLL_LOW_SPEXIT_OVERWORLD: usize = 0x0c110;
pub(super) const CAMERA_X_COORD_SCROLL_LOW_SPEXIT_OVERWORLD: usize = 0x0c112;
pub(super) const SPECIAL_EXIT_ROOM_BOUNDS_Y_START: usize = 0x0c114;
pub(super) const SPECIAL_EXIT_ROOM_BOUNDS_Y_END: usize = 0x0c116;
pub(super) const SPECIAL_EXIT_ROOM_BOUNDS_X_START: usize = 0x0c118;
pub(super) const SPECIAL_EXIT_ROOM_BOUNDS_X_END: usize = 0x0c11a;
pub(super) const UP_DOWN_SCROLL_TARGET_SPEXIT_OVERWORLD: usize = 0x0c11c;
pub(super) const UP_DOWN_SCROLL_TARGET_END_SPEXIT_OVERWORLD: usize = 0x0c11e;
pub(super) const LEFT_RIGHT_SCROLL_TARGET_SPEXIT_OVERWORLD: usize = 0x0c120;
pub(super) const LEFT_RIGHT_SCROLL_TARGET_END_SPEXIT_OVERWORLD: usize = 0x0c122;
pub(super) const OVERWORLD_SPECIAL_TILE_THEME_INDEX: usize = 0x0c124;
pub(super) const MAIN_TILE_THEME_INDEX_SPEXIT_OVERWORLD: usize = 0x0c125;
pub(super) const AUX_TILE_THEME_INDEX_SPEXIT_OVERWORLD: usize = 0x0c126;
pub(super) const SPRITE_GRAPHICS_INDEX_SPEXIT_OVERWORLD: usize = 0x0c127;
pub(super) const OVERWORLD_SCROLL_UP_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c12a;
pub(super) const OVERWORLD_SCROLL_DOWN_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c12c;
pub(super) const OVERWORLD_SCROLL_LEFT_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c12e;
pub(super) const OVERWORLD_SCROLL_RIGHT_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c130;
pub(super) const OVERWORLD_AREA_INDEX_EXIT_OVERWORLD: usize = 0x0c140;
pub(super) const TM_COPY_EXIT_OVERWORLD: usize = 0x0c142;
pub(super) const OVERWORLD_SCREEN_INDEX_EXIT_OVERWORLD: usize = 0x0c14c;
pub(super) const MAP16_LOAD_SRC_OFF_EXIT_OVERWORLD: usize = 0x0c14e;
pub(super) const CAMERA_Y_COORD_SCROLL_LOW_EXIT_OVERWORLD: usize = 0x0c150;
pub(super) const CAMERA_X_COORD_SCROLL_LOW_EXIT_OVERWORLD: usize = 0x0c152;
pub(super) const OW_SCROLL_VARS0_EXIT_OVERWORLD: usize = 0x0c154;
pub(super) const UP_DOWN_SCROLL_TARGET_EXIT_OVERWORLD: usize = 0x0c15c;
pub(super) const UP_DOWN_SCROLL_TARGET_END_EXIT_OVERWORLD: usize = 0x0c15e;
pub(super) const LEFT_RIGHT_SCROLL_TARGET_EXIT_OVERWORLD: usize = 0x0c160;
pub(super) const LEFT_RIGHT_SCROLL_TARGET_END_EXIT_OVERWORLD: usize = 0x0c162;
pub(super) const OVERWORLD_EXIT_TILE_THEME_INDEX_OVERWORLD: usize = 0x0c164;
pub(super) const MAIN_TILE_THEME_INDEX_EXIT_OVERWORLD: usize = 0x0c165;
pub(super) const AUX_TILE_THEME_INDEX_EXIT_OVERWORLD: usize = 0x0c166;
pub(super) const SPRITE_GRAPHICS_INDEX_EXIT_OVERWORLD: usize = 0x0c167;
pub(super) const OVERWORLD_SCROLL_UP_COUNTER_EXIT_OVERWORLD: usize = 0x0c16a;
pub(super) const OVERWORLD_SCROLL_DOWN_COUNTER_EXIT_OVERWORLD: usize = 0x0c16c;
pub(super) const OVERWORLD_SCROLL_LEFT_COUNTER_EXIT_OVERWORLD: usize = 0x0c16e;
pub(super) const OVERWORLD_SCROLL_RIGHT_COUNTER_EXIT_OVERWORLD: usize = 0x0c170;
pub(super) const OVERWORLD_TILE_THEME_INDEX_OVERWORLD: usize = 0x0aa0;
pub(super) const MAIN_TILE_THEME_INDEX_OVERWORLD: usize = 0x0aa1;
pub(super) const AUX_TILE_THEME_INDEX_OVERWORLD: usize = 0x0aa2;
pub(super) const SPRITE_GRAPHICS_INDEX_OVERWORLD: usize = 0x0aa3;
pub(super) const MISC_SPRITES_GRAPHICS_INDEX_OVERWORLD: usize = 0x0aa4;
pub(super) const FLAG_OVERWORLD_AREA_DID_CHANGE_OVERWORLD: usize = 0x0abf;
pub(super) const BIRDTRAVEL_STATUS_OVERWORLD: usize = 0x1af0;
pub(super) const MOVE_OVERLAY_CTR_OVERWORLD: usize = 0x0494;
pub(super) const SAVEGAME_HAS_MASTER_SWORD_FLAGS_OVERWORLD: usize = 0x0f300;
pub(super) const OVERLAY_INDEX_OVERWORLD: usize = 0x008c;
pub(super) const OVERWORLD_SCREEN_INDEX_PREV_OVERWORLD: usize = 0x0c213;
pub(super) const MAP16_LOAD_SRC_OFF_PREV_OVERWORLD: usize = 0x0c215;
pub(super) const MAP16_LOAD_Y_UNIT_PREV_OVERWORLD: usize = 0x0c217;
pub(super) const MAP16_LOAD_DST_OFF_PREV_OVERWORLD: usize = 0x0c219;
pub(super) const OVERWORLD_SCREEN_TRANSITION_PREV_OVERWORLD: usize = 0x0c21b;
pub(super) const OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV_OVERWORLD: usize = 0x0c21d;
pub(super) const OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV_OVERWORLD: usize = 0x0c21f;
pub(super) const TRANSITION_COUNTER_OVERWORLD: usize = 0x0126;
pub(super) const CURRENT_AREA_OF_PLAYER_OVERWORLD: usize = 0x0700;

pub(super) const OVERWORLD_ENTRANCE_PRIMARY_AREA_BY_INDEX: [u16; 44] = [
    0xfe, 0xc5, 0xfe, 0x114, 0x115, 0x175, 0x156, 0xf5, 0xe2, 0x1ef, 0x119, 0xfe, 0x172, 0x177,
    0x13f, 0x172, 0x112, 0x161, 0x172, 0x14c, 0x156, 0x1ef, 0xfe, 0xfe, 0xfe, 0x10b, 0x173, 0x143,
    0x149, 0x175, 0x103, 0x100, 0x1cc, 0x15e, 0x167, 0x128, 0x131, 0x112, 0x16d, 0x163, 0x173,
    0xfe, 0x113, 0x177,
];
pub(super) const OVERWORLD_ENTRANCE_SECONDARY_AREA_BY_INDEX: [u16; 44] = [
    0x14a, 0xc4, 0x14f, 0x115, 0x114, 0x174, 0x155, 0xf5, 0xee, 0x1eb, 0x118, 0x146, 0x171, 0x155,
    0x137, 0x174, 0x173, 0x121, 0x164, 0x155, 0x157, 0x128, 0x114, 0x123, 0x113, 0x109, 0x118,
    0x161, 0x149, 0x117, 0x174, 0x101, 0x1cc, 0x131, 0x51, 0x14e, 0x131, 0x112, 0x17a, 0x163,
    0x172, 0x1bd, 0x152, 0x167,
];
pub(super) const OVERWORLD_AREA_BASE_X: [u16; 64] = [
    0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00,
    0, 0x200, 0x400, 0x600, 0x800, 0xa00, 0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00,
    0xc00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0x200, 0x400, 0x600, 0x800, 0xa00,
    0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00,
    0xa00, 0xe00,
];
pub(super) const OVERWORLD_AREA_BASE_Y: [u16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x200, 0, 0, 0, 0, 0x200, 0x400, 0x400, 0x400, 0x400, 0x400,
    0x400, 0x400, 0x400, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600,
    0x800, 0x600, 0x600, 0x800, 0x600, 0x600, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00,
    0xa00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xe00, 0xe00,
    0xe00, 0xc00, 0xc00, 0xe00,
];
pub(super) const OVERWORLD_VERTICAL_SCROLL_TARGETS: [u16; 64] = [
    0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0x120, 0xff20,
    0xff20, 0xff20, 0xff20, 0x120, 0x320, 0x320, 0x320, 0x320, 0x320, 0x320, 0x320, 0x320, 0x520,
    0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x520, 0x720, 0x520, 0x520, 0x720,
    0x520, 0x520, 0x920, 0x920, 0x920, 0x920, 0x920, 0x920, 0x920, 0x920, 0xb20, 0xb20, 0xb20,
    0xb20, 0xb20, 0xb20, 0xb20, 0xb20, 0xb20, 0xb20, 0xd20, 0xd20, 0xd20, 0xb20, 0xb20, 0xd20,
];
pub(super) const OVERWORLD_HORIZONTAL_SCROLL_TARGETS: [u16; 64] = [
    0xff00, 0xff00, 0x300, 0x500, 0x500, 0x900, 0x900, 0xd00, 0xff00, 0xff00, 0x300, 0x500, 0x500,
    0x900, 0x900, 0xd00, 0xff00, 0x100, 0x300, 0x500, 0x700, 0x900, 0xb00, 0xd00, 0xff00, 0xff00,
    0x300, 0x500, 0x500, 0x900, 0xb00, 0xb00, 0xff00, 0xff00, 0x300, 0x500, 0x500, 0x900, 0xb00,
    0xb00, 0xff00, 0x100, 0x300, 0x500, 0x700, 0x900, 0xb00, 0xd00, 0xff00, 0xff00, 0x300, 0x500,
    0x700, 0x900, 0x900, 0xd00, 0xff00, 0xff00, 0x300, 0x500, 0x700, 0x900, 0x900, 0xd00,
];

pub(super) fn pre_overworld_music_selection(
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

pub(super) fn overworld_offset_base_x_c_index(index: usize) -> u16 {
    if index < OVERWORLD_AREA_BASE_X.len() {
        OVERWORLD_AREA_BASE_X[index]
    } else if index < OVERWORLD_AREA_BASE_X.len() + OVERWORLD_AREA_BASE_Y.len() {
        OVERWORLD_AREA_BASE_Y[index - OVERWORLD_AREA_BASE_X.len()]
    } else {
        OVERWORLD_VERTICAL_SCROLL_TARGETS
            [index - OVERWORLD_AREA_BASE_X.len() - OVERWORLD_AREA_BASE_Y.len()]
    }
}

pub(super) fn overworld_offset_base_y_c_index(index: usize) -> u16 {
    if index < OVERWORLD_AREA_BASE_Y.len() {
        OVERWORLD_AREA_BASE_Y[index]
    } else if index < OVERWORLD_AREA_BASE_Y.len() + OVERWORLD_VERTICAL_SCROLL_TARGETS.len() {
        OVERWORLD_VERTICAL_SCROLL_TARGETS[index - OVERWORLD_AREA_BASE_Y.len()]
    } else {
        OVERWORLD_HORIZONTAL_SCROLL_TARGETS
            [index - OVERWORLD_AREA_BASE_Y.len() - OVERWORLD_VERTICAL_SCROLL_TARGETS.len()]
    }
}

pub(super) const OVERWORLD_AREA_HEIGHTS_BY_SIZE: [u16; 2] = [0x11e, 0x31e];
pub(super) const OVERWORLD_AREA_WIDTHS_BY_SIZE: [u16; 2] = [0x100, 0x300];
pub(super) const OVERWORLD_VERTICAL_SCROLL_SPANS_BY_SIZE: [u16; 2] = [0x2e0, 0x4e0];
pub(super) const OVERWORLD_HORIZONTAL_SCROLL_SPANS_BY_SIZE: [u16; 2] = [0x300, 0x500];
pub(super) const OVERWORLD_MAP16_STRIP_BACKTRACK_BY_DIRECTION: [u16; 3] = [0x03d0, 0x0410, 0xf410];
pub(super) const SPECIAL_EXIT_TOP_BOUNDS: [u16; 16] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0x200, 0x200, 0, 0, 0, 0, 0, 0];
pub(super) const SPECIAL_EXIT_BOTTOM_BOUNDS: [u16; 16] = [
    0x120, 0x20, 0x320, 0x20, 0, 0, 0x320, 0x320, 0x320, 0x220, 0, 0, 0, 0, 0x320, 0x320,
];
pub(super) const SPECIAL_EXIT_LEFT_BOUNDS: [u16; 16] = [
    0, 0x100, 0x200, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0x100, 0x200, 0x600, 0x600, 0xa00,
    0xc00, 0xc00,
];
pub(super) const SPECIAL_EXIT_RIGHT_BOUNDS: [u16; 16] = [
    0, 0x100, 0x500, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0x100, 0x400, 0x600, 0x600, 0xa00,
    0xc00, 0xc00,
];
pub(super) const SPECIAL_EXIT_SCROLL_Y_START: [u16; 16] = [
    0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0x120, 0xff20,
    0xff20, 0xff20, 0xff20, 0x120,
];
pub(super) const SPECIAL_EXIT_SCROLL_Y_END: [u16; 16] = [
    0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0xff20, 0x400, 0x400, 0xff20, 0xff20, 0x120, 0xff20,
    0xff20, 0xff20, 0x400, 0x400,
];
pub(super) const SPECIAL_EXIT_SCROLL_X_START: [u16; 16] = [
    0xfffc, 0x100, 0x300, 0x100, 0x500, 0x900, 0xb00, 0xb00, 0xfffc, 0x100, 0x300, 0x500, 0x500,
    0x900, 0xb00, 0xb00,
];
pub(super) const SPECIAL_EXIT_SCROLL_X_END: [u16; 16] = [
    4, 0x104, 0x300, 0x100, 0x500, 0x900, 0xb00, 0xb00, 4, 0x104, 0x300, 0x100, 0x500, 0x900,
    0xb00, 0xb00,
];
pub(super) const SPECIAL_EXIT_LEFT_EDGE_OF_MAP: [u16; 16] = [
    0, 0, 0x200, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0, 0x200, 0x600, 0x600, 0xa00, 0xc00, 0xc00,
];
pub(super) const SPECIAL_EXIT_DIRECTIONS: [u8; 16] =
    [0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub(super) const SPECIAL_EXIT_SPRITE_GRAPHICS: [u8; 16] = [
    0x0c, 0x0c, 0x0e, 0x0e, 0x0e, 0x10, 0x10, 0x10, 0x0e, 0x0e, 0x0e, 0x0e, 0x10, 0x10, 0x10, 0x10,
];
pub(super) const SPECIAL_EXIT_AUX_GRAPHICS: [u8; 16] = [0x2f; 16];
pub(super) const SPECIAL_EXIT_BG_PALETTES: [u8; 16] = [
    0x0a, 0x0a, 0x0a, 0x0a, 2, 2, 2, 0x0a, 2, 2, 0x0a, 2, 2, 2, 2, 0x0a,
];
pub(super) const SPECIAL_EXIT_SPRITE_PALETTES: [u8; 16] =
    [1, 8, 8, 8, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 2];
pub(super) const VARIOUS_PACKS_OVERWORLD: [u8; 16] = [
    0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x5b, 0x01, 0x5a, 0x42, 0x43, 0x44, 0x45, 0x3f, 0x59, 0x0b, 0x5a,
];
pub(super) const SECONDARY_OVERLAY_BY_OVERWORLD_SCREEN: [u16; 128] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1c0c, 0x1c0c, 0, 0,
    0, 0, 0, 0, 0x1c0c, 0x1c0c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x03b0,
    0x180c, 0x180c, 0x0288, 0, 0, 0, 0, 0, 0x180c, 0x180c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1ab6, 0x1ab6, 0, 0x0e2e, 0x0e2e, 0, 0, 0, 0x1ab6, 0x1ab6,
    0, 0x0e2e, 0x0e2e, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x03b0, 0, 0, 0x0288, 0, 0, 0,
    0, 0, 0, 0, 0,
];
pub(super) const DARK_WORLD_PALETTE_ANIMATION_PHASE1: [u16; 35] = [
    0x0884, 0x0cc7, 0x150a, 0x154d, 0x7ff6, 0x5944, 0x7ad1, 0x0884, 0x0cc7, 0x150a, 0x154d, 0x5bff,
    0x7ad1, 0x21af, 0x1084, 0x48c0, 0x6186, 0x7e6d, 0x7fe0, 0x5944, 0x7e20, 0x1084, 0x000e, 0x1059,
    0x291f, 0x7fe0, 0x5944, 0x7e20, 0x1084, 0x1508, 0x196c, 0x21af, 0x7ff6, 0x1d4c, 0x7ad1,
];
pub(super) const DARK_WORLD_PALETTE_ANIMATION_PHASE2: [u16; 40] = [
    0x7fff, 0x0884, 0x1cc8, 0x1dce, 0x3694, 0x4718, 0x1d4a, 0x18ac, 0x7fff, 0x1908, 0x2d2f, 0x3614,
    0x4eda, 0x471f, 0x1d4a, 0x390f, 0x7fff, 0x34cd, 0x5971, 0x5635, 0x7f1b, 0x7fff, 0x1d4a, 0x3d54,
    0x7fff, 0x1908, 0x2d2f, 0x3614, 0x4eda, 0x471f, 0x1d4a, 0x390f, 0x7fff, 0x0884, 0x052a, 0x21ef,
    0x3ab5, 0x4b39, 0x1d4c, 0x18ac,
];
pub(super) const SPECIAL_SWITCH_AREA_TILE_IDS: [u16; 4] = [0x0105, 0x01e4, 0x00ad, 0x00b9];
pub(super) const SPECIAL_SWITCH_AREA_SCREENS: [u16; 4] = [0, 45, 15, 129];
pub(super) const SPECIAL_SWITCH_AREA_DIRECTIONS: [u8; 4] = [8, 2, 8, 8];
pub(super) const SPECIAL_SWITCH_AREA_EXITS: [u16; 4] = [0x0180, 0x0181, 0x0182, 0x0189];
pub(super) const SPECIAL_SWITCH_AREA_B_TILE_IDS: [u16; 3] = [0x017c, 0x01e4, 0x00ad];
pub(super) const SPECIAL_SWITCH_AREA_B_SCREENS: [u16; 3] = [0x0080, 0x0080, 0x0081];
pub(super) const SPECIAL_SWITCH_AREA_B_DIRECTIONS: [u8; 3] = [4, 1, 4];
pub(super) const SPECIAL_SWITCH_MAP16_MASKS: [u16; 4] = [0x0f80, 0x0f80, 0x003f, 0x003f];
pub(super) const SPECIAL_SWITCH_MAP16_OFFSETS: [u16; 256] = [
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
pub(super) const SPECIAL_SWITCH_AREA_DELTAS: [i16; 4] = [2, -2, 16, -16];
pub(super) const OVERWORLD_AREA_TILEMAP_HEADS: [u8; 64] = [
    0, 0, 2, 3, 3, 5, 5, 7, 0, 0, 10, 3, 3, 5, 5, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 24, 26,
    27, 27, 29, 30, 30, 24, 24, 34, 27, 27, 37, 30, 30, 40, 41, 42, 43, 44, 45, 46, 47, 48, 48, 50,
    51, 52, 53, 53, 55, 48, 48, 58, 59, 60, 53, 53, 63,
];
pub(super) const OVERWORLD_SCROLL_DIRECTION_BITS: [u16; 4] = [8, 4, 2, 1];
pub(super) const OVERWORLD_TRANSITION_SCROLL_DELTAS: [i16; 4] = [-8, 8, -8, 8];
pub(super) const OVERWORLD_TRANSITION_PLAYER_MOVE_FRAMES: [u8; 4] = [27, 27, 30, 30];
pub(super) const OVERWORLD_TRANSITION_CAMERA_OFFSETS: [i16; 4] = [-0x70, 0x70, -0x70, 0x70];
pub(super) const OVERWORLD_ADJACENT_AREA_DELTAS: [i16; 4] = [-8, 8, -1, 1];
pub(super) const OVERWORLD_ENTRY_SETTLE_COORDINATES: [u8; 4] = [0xe0, 8, 0xe0, 0x10];
