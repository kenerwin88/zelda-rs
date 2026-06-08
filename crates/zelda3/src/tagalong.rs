// Methods ported from zelda3/src/tagalong.c and included inside ZeldaState.

use super::*;
use crate::types::{abs16, sign16, sign8, OamEnt, ProjectSpeedRet};

const MAIN_MODULE_INDEX_TAGALONG: usize = 0x10;
const SUBMODULE_INDEX_TAGALONG: usize = 0x11;
const LINK_Y_COORD_TAGALONG: usize = 0x20;
const LINK_X_COORD_TAGALONG: usize = 0x22;
const LINK_Z_COORD_TAGALONG: usize = 0x24;
const LINK_Y_VEL_TAGALONG: usize = 0x30;
const LINK_X_VEL_TAGALONG: usize = 0x31;
const BUTTON_MASK_B_Y_TAGALONG: usize = 0x3a;
// NES_Ver2: HIKUFG, "pull set flag".
const LINK_PULL_ACTION_STATE_TAGALONG: usize = 0x377;
const LINK_STATE_BITS_TAGALONG: usize = 0x308;
const LINK_GRABBING_WALL_TAGALONG: usize = 0x376;
const LINK_SPEED_SETTING_TAGALONG: usize = 0x5e;
const LINK_DIRECTION_FACING_TAGALONG: usize = 0x2f;
const LINK_IS_RUNNING_TAGALONG: usize = 0x372;
const PLAYER_IS_INDOORS_TAGALONG: usize = 0x1b;
const LINK_IS_ON_LOWER_LEVEL_TAGALONG: usize = 0xee;
const LINK_PLAYER_HANDLER_STATE_TAGALONG: usize = 0x5d;
const LINK_AUXILIARY_STATE_TAGALONG: usize = 0x4d;
const LINK_POSITION_MODE_TAGALONG: usize = 0x37a;
const LINK_ITEM_IN_HAND_TAGALONG: usize = 0x301;
const FLAG_IS_LINK_IMMOBILIZED_TAGALONG: usize = 0x2e4;
const LINK_DISABLE_SPRITE_DAMAGE_TAGALONG: usize = 0x37b;
const PLAYER_NEAR_PIT_STATE_TAGALONG: usize = 0x5b;
const FILTERED_JOYPAD_L_TAGALONG: usize = 0xf6;
const BG2HOFS_COPY2_TAGALONG: usize = 0xe2;
const BG2VOFS_COPY2_TAGALONG: usize = 0xe8;
const FRAME_COUNTER_TAGALONG: usize = 0x1a;
const TAGALONG_MESSAGE_TIMER: usize = 0x2cd;
const TAGALONG_DATA_INDEX_TAGALONG: usize = 0x2cf;
const TAGALONG_VAR3_TAGALONG: usize = 0x2d0;
const TAGALONG_VAR7_TAGALONG: usize = 0x2d1;
const TIMER_TAGALONG_REACQUIRE_TAGALONG: usize = 0x2d2;
const TAGALONG_VAR1_TAGALONG: usize = 0x2d3;
const TAGALONG_JUMP_TIMER_TAGALONG: usize = 0x2d6;
const TAGALONG_DRAW_ANIM_FRAME: usize = 0x2d7;
const FLAG_IS_ANCILLA_TO_PICK_UP_TAGALONG: usize = 0x2ec;
const TAGALONG_EVENT_FLAGS_TAGALONG: usize = 0x2f2;
const FLAG_IS_SPRITE_TO_PICK_UP_TAGALONG: usize = 0x314;
const TAGALONG_APPEARANCE_NONE_FLAG_TAGALONG: usize = 0x2f9;
const COUNTDOWN_FOR_BLINK_TAGALONG: usize = 0x31f;
const RELATED_TO_HOOKSHOT_TAGALONG: usize = 0x37e;
const DRAW_WATER_RIPPLES_OR_GRASS_TAGALONG: usize = 0x351;
const SWIM_ACCELERATION_TAGALONG: usize = 0x33c;
const OAM_PRIORITY_VALUE_TAGALONG: usize = 0x64;
const OAM_CUR_PTR_TAGALONG: usize = 0x90;
const OAM_EXT_CUR_PTR_TAGALONG: usize = 0x92;
const SOUND_EFFECT_1_TAGALONG: usize = 0x12e;
const MUSIC_CONTROL_TAGALONG: usize = 0x12c;
const DIALOGUE_MESSAGE_INDEX_TAGALONG: usize = 0x1cf0;
const SAVED_MODULE_FOR_MENU_TAGALONG: usize = 0x10c;
const TAGALONG_MESSAGE_RESET_FLAG: usize = 0x223;
const MESSAGING_MODULE_TAGALONG: usize = 0x1cd8;
const TAGALONG_DMA_HEAD_POINTER: usize = 0x0ae8;
const TAGALONG_DMA_BODY_POINTER: usize = 0x0aea;
const PALETTE_SWAP_FLAG_TAGALONG: usize = 0x0abd;
const SORT_SPRITES_SETTING_TAGALONG: usize = 0x0fb3;
const SUPER_BOMB_INDICATOR_TIMER_TAGALONG: usize = 0x4b4;
const SUPER_BOMB_INDICATOR_COUNTER_TAGALONG: usize = 0x4b5;
const DUNGEON_ROOM_INDEX_TAGALONG: usize = 0xa0;
const DUNG_SAVEGAME_STATE_BITS_TAGALONG: usize = 0x402;
const DUNG_FLAG_TRAPDOORS_DOWN_TAGALONG: usize = 0x468;
const DUNG_CUR_DOOR_POS_TAGALONG: usize = 0x68e;
const DOOR_ANIMATION_STEP_INDICATOR_TAGALONG: usize = 0x690;
const KIKI_ANIM_COUNTER_TAGALONG: usize = 0x0b69;
const SPRITE_IGNORE_PROJECTILE_TAGALONG: usize = 0x0ba0;
const SPRITE_STATE_TAGALONG: usize = 0x0dd0;
const SPRITE_TYPE_TAGALONG: usize = 0x0e20;
const SPRITE_Y_LO_TAGALONG: usize = 0x0d00;
const SPRITE_X_LO_TAGALONG: usize = 0x0d10;
const SPRITE_Y_HI_TAGALONG: usize = 0x0d20;
const SPRITE_X_HI_TAGALONG: usize = 0x0d30;
const SPRITE_Y_VEL_TAGALONG: usize = 0x0d40;
const SPRITE_X_VEL_TAGALONG: usize = 0x0d50;
const SPRITE_AI_STATE_TAGALONG: usize = 0x0d80;
const SPRITE_HEAD_DIR_TAGALONG: usize = 0x0eb0;
const SPRITE_D_TAGALONG: usize = 0x0de0;
const SPRITE_DELAY_AUX2_TAGALONG: usize = 0x0e10;
const SPRITE_GRAPHICS_TAGALONG: usize = 0x0dc0;
const SPRITE_FLOOR_TAGALONG: usize = 0x0f20;
const SPRITE_Z_TAGALONG: usize = 0x0f70;
const SPRITE_Z_VEL_TAGALONG: usize = 0x0f80;
const SPRITE_SUBTYPE2_TAGALONG: usize = 0x0e80;
const SPRITE_N_TAGALONG: usize = 0x0bc0;
const SPRITE_N_WORD_TAGALONG: usize = 0x0bc0;
const SPRITE_DIE_ACTION_TAGALONG: usize = 0x0cba;
const SPRITE_SUBTYPE_TAGALONG: usize = 0x0e30;
const ANCILLA_Y_LO_TAGALONG: usize = 0x0bfa;
const ANCILLA_X_LO_TAGALONG: usize = 0x0c04;
const ANCILLA_Y_HI_TAGALONG: usize = 0x0c0e;
const ANCILLA_X_HI_TAGALONG: usize = 0x0c18;
const ANCILLA_Y_VEL_TAGALONG: usize = 0x0c22;
const ANCILLA_X_VEL_TAGALONG: usize = 0x0c2c;
const ANCILLA_Y_SUBPIXEL_TAGALONG: usize = 0x0c36;
const ANCILLA_X_SUBPIXEL_TAGALONG: usize = 0x0c40;
const ANCILLA_TYPE_TAGALONG: usize = 0x0c4a;
const ANCILLA_STEP_TAGALONG: usize = 0x0c54;
const ANCILLA_ITEM_TO_LINK_TAGALONG: usize = 0x0c5e;
const ANCILLA_ARR3_TAGALONG: usize = 0x39f;
const ANCILLA_L_TAGALONG: usize = 0x385;
const ANCILLA_R_TAGALONG: usize = 0x03ea;
const OAM_BUF_TAGALONG: usize = 0x0800;
const BYTEWISE_EXTENDED_OAM_TAGALONG: usize = 0x0a20;
const ANCILLA_ARR25_TAGALONG: usize = 0x0746;
const SAVE_DUNG_INFO_TAGALONG: usize = 0x0f000;
const SAVE_OW_EVENT_INFO_TAGALONG: usize = 0x0f280;
const FOLLOWER_INDICATOR_TAGALONG: usize = 0x0f3cc;
const SAVED_TAGALONG_Y_TAGALONG: usize = 0x0f3cd;
const SAVED_TAGALONG_X_TAGALONG: usize = 0x0f3cf;
const SAVED_TAGALONG_INDOORS_TAGALONG: usize = 0x0f3d1;
const SAVED_TAGALONG_FLOOR_TAGALONG: usize = 0x0f3d2;
const FOLLOWER_DROPPED_TAGALONG: usize = 0x0f3d3;
const TAGALONG_Y_LO_TAGALONG: usize = 0x1a00;
const TAGALONG_Y_HI_TAGALONG: usize = 0x1a14;
const TAGALONG_X_LO_TAGALONG: usize = 0x1a28;
const TAGALONG_X_HI_TAGALONG: usize = 0x1a3c;
const TAGALONG_Z_TAGALONG: usize = 0x1a50;
const TAGALONG_LAYERBITS_TAGALONG: usize = 0x1a64;
const OVERWORLD_SCREEN_INDEX_TAGALONG: usize = 0x8a;
const ENHANCED_FEATURES0_TAGALONG: usize = 0x064c;

const K_PLAYER_STATE_GROUND: u8 = 0;
const K_PLAYER_STATE_SWIMMING: u8 = 4;
const K_PLAYER_STATE_RECOIL_OTHER: u8 = 6;
const K_PLAYER_STATE_START_DASH: u8 = 17;
const K_PLAYER_STATE_HOOKSHOT: u8 = 19;
const K_PLAYER_STATE_ETHER: u8 = 8;
const K_PLAYER_STATE_BOMBOS: u8 = 9;
const K_PLAYER_STATE_QUAKE: u8 = 10;
const FEATURES0_MISC_BUG_FIXES_TAGALONG: u32 = 4096;
const FEATURES0_TURN_WHILE_DASHING_TAGALONG: u32 = 4;

const TAGALONG_FLAGS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
const TAGALONG_TAB5: [u8; 15] = [0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const TAGALONG_TAB4: [u8; 15] = [0, 0, 3, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const TAGALONG_TAB0: [u8; 3] = [5, 9, 0x0a];
const TAGALONG_TAB1: [u16; 3] = [0x0df3, 0x06f9, 0x0df3];
const TAGALONG_MSG: [u16; 3] = [0x20, 0x108, 0x11d];

#[derive(Clone, Copy)]
pub(super) struct TagalongMessageInfo {
    y: u16,
    x: u16,
    bit: u8,
    msg: u16,
    tagalong: u8,
}

#[derive(Clone, Copy)]
struct TagalongSprXY {
    y1: i8,
    x1: i8,
    y2: i8,
    x2: i8,
}

#[derive(Clone, Copy)]
struct TagalongDmaFlags {
    dma6: u8,
    dma7: u8,
    flags: u8,
}

struct SpriteSpawnInfo {
    r0_x: u16,
    r2_y: u16,
}

const TAGALONG_INDOOR_INFOS: [TagalongMessageInfo; 12] = [
    TagalongMessageInfo {
        y: 0x1ef0,
        x: 0x288,
        bit: 1,
        msg: 0x99,
        tagalong: 4,
    },
    TagalongMessageInfo {
        y: 0x1e58,
        x: 0x2f0,
        bit: 2,
        msg: 0x9a,
        tagalong: 4,
    },
    TagalongMessageInfo {
        y: 0x1ea8,
        x: 0x3b8,
        bit: 4,
        msg: 0x9b,
        tagalong: 4,
    },
    TagalongMessageInfo {
        y: 0x0cf8,
        x: 0x25b,
        bit: 1,
        msg: 0x21,
        tagalong: 1,
    },
    TagalongMessageInfo {
        y: 0x0cf8,
        x: 0x39d,
        bit: 2,
        msg: 0x21,
        tagalong: 2,
    },
    TagalongMessageInfo {
        y: 0x0c78,
        x: 0x238,
        bit: 4,
        msg: 0x21,
        tagalong: 4,
    },
    TagalongMessageInfo {
        y: 0x0a30,
        x: 0x2f8,
        bit: 1,
        msg: 0x22,
        tagalong: 1,
    },
    TagalongMessageInfo {
        y: 0x0178,
        x: 0x550,
        bit: 1,
        msg: 0x23,
        tagalong: 1,
    },
    TagalongMessageInfo {
        y: 0x0168,
        x: 0x4f8,
        bit: 2,
        msg: 0x2a,
        tagalong: 1,
    },
    TagalongMessageInfo {
        y: 0x1bd8,
        x: 0x16fc,
        bit: 1,
        msg: 0x124,
        tagalong: 6,
    },
    TagalongMessageInfo {
        y: 0x1520,
        x: 0x167c,
        bit: 1,
        msg: 0x124,
        tagalong: 6,
    },
    TagalongMessageInfo {
        y: 0x05ac,
        x: 0x4fc,
        bit: 1,
        msg: 0x29,
        tagalong: 1,
    },
];

const TAGALONG_OUTDOOR_INFOS: [TagalongMessageInfo; 5] = [
    TagalongMessageInfo {
        y: 0x03c0,
        x: 0x0730,
        bit: 1,
        msg: 0x9d,
        tagalong: 4,
    },
    TagalongMessageInfo {
        y: 0x0648,
        x: 0x0f50,
        bit: 0,
        msg: 0xffff,
        tagalong: 0x0a,
    },
    TagalongMessageInfo {
        y: 0x06c8,
        x: 0x0d78,
        bit: 1,
        msg: 0xffff,
        tagalong: 0x0a,
    },
    TagalongMessageInfo {
        y: 0x0688,
        x: 0x0c78,
        bit: 2,
        msg: 0xffff,
        tagalong: 0x0a,
    },
    TagalongMessageInfo {
        y: 0x00e8,
        x: 0x0090,
        bit: 0,
        msg: 0x28,
        tagalong: 0x0e,
    },
];

const TAGALONG_INDOOR_OFFSETS: [u8; 8] = [0, 3, 6, 7, 9, 10, 11, 12];
const TAGALONG_OUTDOOR_OFFSETS: [u8; 4] = [0, 1, 4, 5];
const TAGALONG_INDOOR_ROOMS: [u16; 7] = [0xf1, 0x61, 0x51, 2, 0xdb, 0xab, 0x22];
const TAGALONG_OUTDOOR_ROOMS: [u16; 3] = [3, 0x5e, 0];

const TAGALONG_DRAW_SPR_XY: [TagalongSprXY; 56] = [
    TagalongSprXY {
        y1: -2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: -3,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 3,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: -3,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 1,
        x1: 3,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: -1,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 0,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 2,
        x1: 0,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 2,
        x1: -1,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 2,
        x1: 1,
        y2: 0,
        x2: 0,
    },
    TagalongSprXY {
        y1: 3,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 3,
        x1: 0,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 3,
        x1: -1,
        y2: 1,
        x2: 0,
    },
    TagalongSprXY {
        y1: 3,
        x1: 1,
        y2: 1,
        x2: 0,
    },
];

const TAGALONG_DMA_AND_FLAGS: [TagalongDmaFlags; 16] = [
    TagalongDmaFlags {
        dma6: 0x20,
        dma7: 0xc0,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x00,
        dma7: 0xa0,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0x60,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0x60,
        flags: 0x44,
    },
    TagalongDmaFlags {
        dma6: 0x20,
        dma7: 0xc0,
        flags: 0x04,
    },
    TagalongDmaFlags {
        dma6: 0x00,
        dma7: 0xa0,
        flags: 0x04,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0x80,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0x80,
        flags: 0x44,
    },
    TagalongDmaFlags {
        dma6: 0x20,
        dma7: 0xe0,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x00,
        dma7: 0xe0,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0xe0,
        flags: 0x00,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0xe0,
        flags: 0x44,
    },
    TagalongDmaFlags {
        dma6: 0x20,
        dma7: 0xe0,
        flags: 0x04,
    },
    TagalongDmaFlags {
        dma6: 0x00,
        dma7: 0xe0,
        flags: 0x04,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0xe0,
        flags: 0x04,
    },
    TagalongDmaFlags {
        dma6: 0x40,
        dma7: 0xe0,
        flags: 0x40,
    },
];

const TAGALONG_DRAW_PALS: [u8; 14] = [0, 4, 4, 4, 4, 0, 7, 4, 4, 3, 4, 4, 4, 4];
const TAGALONG_DRAW_OFFS: [u16; 14] = [
    0, 0, 0x80, 0x80, 0x80, 0, 0, 0xc0, 0xc0, 0x100, 0x180, 0x180, 0x140, 0x140,
];
const TAGALONG_DRAW_SPR_INFO0: [u8; 24] = [
    0xd8, 0x24, 0xd8, 0x64, 0xd9, 0x24, 0xd9, 0x64, 0xda, 0x24, 0xda, 0x64, 0xc8, 0x22, 0xc8, 0x62,
    0xc9, 0x22, 0xc9, 0x62, 0xca, 0x22, 0xca, 0x62,
];
const TAGALONG_DRAW_SPR_OFFS0: [u16; 2] = [0x170, 0xc0];
const TAGALONG_DRAW_SPR_OFFS1: [u16; 2] = [0x1c0, 0x110];

impl ZeldaState {
    pub(super) fn tagalong_is_following(&self) -> bool {
        let main = self.ram[MAIN_MODULE_INDEX_TAGALONG];
        let sub = self.ram[SUBMODULE_INDEX_TAGALONG];
        self.ram[FLAG_IS_LINK_IMMOBILIZED_TAGALONG] == 0
            && sub != 10
            && !(main == 9 && sub == 0x23)
            && !(main == 14 && (sub == 1 || sub == 2))
    }

    pub(super) fn follower_validate_message_freedom(&self) -> bool {
        let ps = self.ram[LINK_PLAYER_HANDLER_STATE_TAGALONG];
        if ps != K_PLAYER_STATE_GROUND
            && ps != K_PLAYER_STATE_SWIMMING
            && ps != K_PLAYER_STATE_START_DASH
        {
            return false;
        }
        let t = (self.ram[BUTTON_MASK_B_Y_TAGALONG] & 0x80)
            | self.ram[LINK_PULL_ACTION_STATE_TAGALONG]
            | self.ram[LINK_ITEM_IN_HAND_TAGALONG]
            | self.ram[LINK_POSITION_MODE_TAGALONG]
            | self.ram[FLAG_IS_ANCILLA_TO_PICK_UP_TAGALONG]
            | self.ram[FLAG_IS_SPRITE_TO_PICK_UP_TAGALONG]
            | self.ram[LINK_STATE_BITS_TAGALONG]
            | self.ram[LINK_GRABBING_WALL_TAGALONG];
        t == 0
    }

    pub(super) fn follower_move_towards_link(&mut self) {
        loop {
            let k = 9;
            let j = self.ram[TAGALONG_VAR1_TAGALONG] as usize;
            self.ram[ANCILLA_Y_LO_TAGALONG + k] = self.ram[TAGALONG_Y_LO_TAGALONG + j];
            self.ram[ANCILLA_Y_HI_TAGALONG + k] = self.ram[TAGALONG_Y_HI_TAGALONG + j];
            self.ram[ANCILLA_X_LO_TAGALONG + k] = self.ram[TAGALONG_X_LO_TAGALONG + j];
            self.ram[ANCILLA_X_HI_TAGALONG + k] = self.ram[TAGALONG_X_HI_TAGALONG + j];

            let pt = self.Ancilla_ProjectSpeedTowardsPlayer(k, 24);
            self.ram[ANCILLA_X_VEL_TAGALONG + k] = pt.x;
            self.ram[ANCILLA_Y_VEL_TAGALONG + k] = pt.y;
            self.Ancilla_MoveY(k);
            self.Ancilla_MoveX(k);

            let x = self.Ancilla_GetX(k);
            let y = self.Ancilla_GetY(k);
            if abs16(x.wrapping_sub(read_le_u16(&self.ram, LINK_X_COORD_TAGALONG))) < 2
                && abs16(y.wrapping_sub(read_le_u16(&self.ram, LINK_Y_COORD_TAGALONG))) < 2
            {
                return;
            }
            self.ram[TAGALONG_VAR1_TAGALONG] = self.ram[TAGALONG_VAR1_TAGALONG].wrapping_add(1);
            let k = self.ram[TAGALONG_VAR1_TAGALONG] as usize;
            if k == 18 {
                return;
            }
            self.ram[TAGALONG_Y_LO_TAGALONG + k] = y as u8;
            self.ram[TAGALONG_Y_HI_TAGALONG + k] = (y >> 8) as u8;
            self.ram[TAGALONG_X_LO_TAGALONG + k] = x as u8;
            self.ram[TAGALONG_X_HI_TAGALONG + k] = (x >> 8) as u8;
            self.ram[TAGALONG_LAYERBITS_TAGALONG + k] =
                (TAGALONG_FLAGS[self.ram[LINK_IS_ON_LOWER_LEVEL_TAGALONG] as usize] >> 2) | 1;
        }
    }

    pub(super) fn follower_check_blind_trigger(&self) -> bool {
        let k = self.ram[TAGALONG_DATA_INDEX_TAGALONG] as usize;
        let mut x = self.tagalong_x(k);
        let mut y = self.tagalong_y(k);
        let z = self.ram[TAGALONG_Z_TAGALONG + k] as i8 as i16 as u16;
        y = y.wrapping_add(z).wrapping_add(12);
        x = x.wrapping_add(8);
        abs16(0x1568u16.wrapping_sub(y)) < 24 && abs16(0x1980u16.wrapping_sub(x)) < 24
    }

    pub(super) fn follower_initialize(&mut self) {
        let y = read_le_u16(&self.ram, LINK_Y_COORD_TAGALONG);
        let x = read_le_u16(&self.ram, LINK_X_COORD_TAGALONG);
        self.set_tagalong_y(0, y);
        self.set_tagalong_x(0, x);
        self.ram[TAGALONG_LAYERBITS_TAGALONG] =
            (TAGALONG_FLAGS[self.ram[LINK_IS_ON_LOWER_LEVEL_TAGALONG] as usize] >> 2)
                | (self.ram[LINK_DIRECTION_FACING_TAGALONG] >> 1);
        self.ram[TIMER_TAGALONG_REACQUIRE_TAGALONG] = 64;
        self.ram[TAGALONG_DATA_INDEX_TAGALONG] = 0;
        self.ram[TAGALONG_VAR1_TAGALONG] = 0;
        self.ram[TAGALONG_VAR3_TAGALONG] = 0;
        self.ram[TAGALONG_JUMP_TIMER_TAGALONG] = 0;
        self.ram[LINK_SPEED_SETTING_TAGALONG] = 0;
        if self.read_u32_ram(ENHANCED_FEATURES0_TAGALONG) & FEATURES0_TURN_WHILE_DASHING_TAGALONG
            != 0
        {
            self.ram[LINK_PLAYER_HANDLER_STATE_TAGALONG] = K_PLAYER_STATE_GROUND;
            self.ram[LINK_IS_RUNNING_TAGALONG] = 0;
        }
    }

    pub(super) fn sprite_become_follower(&mut self, k: usize) {
        self.ram[TAGALONG_APPEARANCE_NONE_FLAG_TAGALONG] = 0;
        let y = self.Tagalong_Sprite_GetY(k).wrapping_sub(6);
        self.set_tagalong_y(0, y);
        let x = self.Tagalong_Sprite_GetX(k).wrapping_add(1);
        self.set_tagalong_x(0, x);
        self.ram[TAGALONG_LAYERBITS_TAGALONG] =
            (TAGALONG_FLAGS[self.ram[LINK_IS_ON_LOWER_LEVEL_TAGALONG] as usize] >> 2) | 1;
        self.ram[TIMER_TAGALONG_REACQUIRE_TAGALONG] = 64;
        self.ram[TAGALONG_VAR1_TAGALONG] = 0;
        self.ram[TAGALONG_DATA_INDEX_TAGALONG] = 0;
        self.ram[TAGALONG_VAR3_TAGALONG] = 0;
        self.ram[TAGALONG_JUMP_TIMER_TAGALONG] = 0;
        self.ram[LINK_SPEED_SETTING_TAGALONG] = 0;
        self.ram[TAGALONG_APPEARANCE_NONE_FLAG_TAGALONG] = 0;
        self.ram[FOLLOWER_DROPPED_TAGALONG] = 0;
        self.follower_move_towards_link();
    }

    pub(super) fn follower_main(&mut self) {
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 0 {
            return;
        }
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 0x0e {
            self.follower_handle_trigger();
            return;
        }
        let j = TAGALONG_TAB0
            .iter()
            .position(|&v| v == self.ram[FOLLOWER_INDICATOR_TAGALONG])
            .map(|v| v as i32)
            .unwrap_or(-1);
        if j >= 0
            && self.ram[SUBMODULE_INDEX_TAGALONG] == 0
            && !(j == 2 && self.ram[OVERWORLD_SCREEN_INDEX_TAGALONG] & 0x40 != 0)
        {
            let timer = read_le_u16(&self.ram, TAGALONG_MESSAGE_TIMER).wrapping_sub(1);
            write_le_u16(&mut self.ram, TAGALONG_MESSAGE_TIMER, timer);
            if sign16(timer) {
                if !self.follower_validate_message_freedom() {
                    write_le_u16(&mut self.ram, TAGALONG_MESSAGE_TIMER, 0);
                } else {
                    let j = j as usize;
                    write_le_u16(&mut self.ram, TAGALONG_MESSAGE_TIMER, TAGALONG_TAB1[j]);
                    write_le_u16(
                        &mut self.ram,
                        DIALOGUE_MESSAGE_INDEX_TAGALONG,
                        TAGALONG_MSG[j],
                    );
                    self.Tagalong_Main_ShowTextMessage();
                }
            }
        }
        if j != 0 {
            self.follower_no_timed_message();
        }
    }

    pub(super) fn follower_no_timed_message(&mut self) {
        if self.ram[FOLLOWER_DROPPED_TAGALONG] != 0 {
            self.follower_not_following();
            return;
        }
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 12 {
            if self.ram[LINK_AUXILIARY_STATE_TAGALONG] == 0 {
                if self.follower_can_drop() {
                    self.follower_drop();
                    return;
                }
            }
        } else if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 13 {
            if self.ram[LINK_AUXILIARY_STATE_TAGALONG] == 2
                || self.ram[PLAYER_NEAR_PIT_STATE_TAGALONG] == 2
            {
                self.follower_drop();
                return;
            }
            if self.follower_can_drop() {
                self.follower_drop();
                return;
            }
        }
        self.follower_check_game_mode();
    }

    fn follower_can_drop(&self) -> bool {
        self.ram[SUBMODULE_INDEX_TAGALONG] == 0
            && self.ram[LINK_AUXILIARY_STATE_TAGALONG] != 1
            && self.ram[LINK_STATE_BITS_TAGALONG] & 0x80 == 0
            && self.ram[TAGALONG_APPEARANCE_NONE_FLAG_TAGALONG] == 0
            && self.ram[TAGALONG_VAR3_TAGALONG] == 0
            && (self.ram[TAGALONG_Z_TAGALONG + self.ram[TAGALONG_DATA_INDEX_TAGALONG] as usize]
                as i8)
                <= 0
            && self.ram[FILTERED_JOYPAD_L_TAGALONG] & 0x80 != 0
    }

    fn follower_drop(&mut self) {
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 13 && self.ram[PLAYER_IS_INDOORS_TAGALONG] == 0
        {
            let ps = self.ram[LINK_PLAYER_HANDLER_STATE_TAGALONG];
            if ps == K_PLAYER_STATE_ETHER
                || ps == K_PLAYER_STATE_BOMBOS
                || ps == K_PLAYER_STATE_QUAKE
            {
                self.follower_check_game_mode();
                return;
            }
            self.ram[SUPER_BOMB_INDICATOR_TIMER_TAGALONG] = 3;
            self.ram[SUPER_BOMB_INDICATOR_COUNTER_TAGALONG] = 0xbb;
        }
        self.ram[FOLLOWER_DROPPED_TAGALONG] = 128;
        self.ram[TIMER_TAGALONG_REACQUIRE_TAGALONG] = 64;
        let k = self.ram[TAGALONG_DATA_INDEX_TAGALONG] as usize;
        let y = self.tagalong_y(k);
        let x = self.tagalong_x(k);
        write_le_u16(&mut self.ram, SAVED_TAGALONG_Y_TAGALONG, y);
        write_le_u16(&mut self.ram, SAVED_TAGALONG_X_TAGALONG, x);
        self.ram[SAVED_TAGALONG_FLOOR_TAGALONG] = self.ram[LINK_IS_ON_LOWER_LEVEL_TAGALONG];
        self.ram[SAVED_TAGALONG_INDOORS_TAGALONG] = self.ram[PLAYER_IS_INDOORS_TAGALONG];
        self.follower_not_following();
    }

    pub(super) fn follower_check_game_mode(&mut self) {
        if self.tagalong_is_following()
            && (self.ram[LINK_X_VEL_TAGALONG] | self.ram[LINK_Y_VEL_TAGALONG]) != 0
        {
            let mut k = self.ram[TAGALONG_VAR1_TAGALONG].wrapping_add(1);
            if k == 20 {
                k = 0;
            }
            self.ram[TAGALONG_VAR1_TAGALONG] = k;
            let mut z = self.ram[LINK_Z_COORD_TAGALONG];
            if z >= 0xf0 {
                z = 0;
            }
            let k = k as usize;
            self.ram[TAGALONG_Z_TAGALONG + k] = z;
            let y = read_le_u16(&self.ram, LINK_Y_COORD_TAGALONG).wrapping_sub(z as u16);
            self.set_tagalong_y(k, y);
            self.set_tagalong_x(k, read_le_u16(&self.ram, LINK_X_COORD_TAGALONG));
            let mut layerbits = (self.ram[LINK_DIRECTION_FACING_TAGALONG] >> 1)
                | (TAGALONG_FLAGS[self.ram[LINK_IS_ON_LOWER_LEVEL_TAGALONG] as usize] >> 2);
            if self.ram[LINK_PLAYER_HANDLER_STATE_TAGALONG] == K_PLAYER_STATE_SWIMMING {
                layerbits |= 0x20;
            } else {
                if self.ram[LINK_PLAYER_HANDLER_STATE_TAGALONG] == K_PLAYER_STATE_HOOKSHOT
                    && self.ram[RELATED_TO_HOOKSHOT_TAGALONG] != 0
                {
                    layerbits |= 0x10;
                }
                if self.ram[DRAW_WATER_RIPPLES_OR_GRASS_TAGALONG] != 0 {
                    layerbits |= if self.ram[DRAW_WATER_RIPPLES_OR_GRASS_TAGALONG] == 1 {
                        0x80
                    } else {
                        0x40
                    };
                }
            }
            self.ram[TAGALONG_LAYERBITS_TAGALONG + k] = layerbits;
        }
        match self.ram[FOLLOWER_INDICATOR_TAGALONG] {
            2 | 4 => self.follower_old_man(),
            3 | 11 => self.follower_old_man_unused(),
            5 | 14 => self.follower_basic_mover(),
            _ => self.follower_basic_mover(),
        }
    }

    pub(super) fn follower_basic_mover(&mut self) {
        if !self.tagalong_is_following() {
            self.tagalong_draw();
            return;
        }
        self.follower_handle_trigger();
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 10
            && self.ram[LINK_AUXILIARY_STATE_TAGALONG] != 0
            && self.ram[COUNTDOWN_FOR_BLINK_TAGALONG] != 0
        {
            let k = if self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1) == 20 {
                0
            } else {
                self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1)
            };
            self.kiki_spawn_handler_b(k as usize);
            self.ram[FOLLOWER_INDICATOR_TAGALONG] = 0;
            return;
        }
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 6
            && read_le_u16(&self.ram, DUNGEON_ROOM_INDEX_TAGALONG) == 0x0ac
            && read_le_u16(&self.ram, SAVE_DUNG_INFO_TAGALONG + 101 * 2) & 0x100 != 0
            && self.follower_check_blind_trigger()
        {
            let k = self.ram[TAGALONG_DATA_INDEX_TAGALONG] as usize;
            let x = self.tagalong_x(k);
            let y = self.tagalong_y(k);
            self.ram[FOLLOWER_INDICATOR_TAGALONG] = 0;
            self.blind_spawn_from_maiden(x, y);
            self.ram[DUNG_FLAG_TRAPDOORS_DOWN_TAGALONG] =
                self.ram[DUNG_FLAG_TRAPDOORS_DOWN_TAGALONG].wrapping_add(1);
            self.ram[DUNG_CUR_DOOR_POS_TAGALONG] = 0;
            self.ram[DOOR_ANIMATION_STEP_INDICATOR_TAGALONG] = 0;
            self.ram[SUBMODULE_INDEX_TAGALONG] = 5;
            self.ram[MUSIC_CONTROL_TAGALONG] = 21;
            return;
        }
        if self.ram[TAGALONG_VAR3_TAGALONG] == 0 {
            if self.ram[LINK_PLAYER_HANDLER_STATE_TAGALONG] == K_PLAYER_STATE_HOOKSHOT
                && self.ram[RELATED_TO_HOOKSHOT_TAGALONG] != 0
            {
                self.ram[TAGALONG_VAR3_TAGALONG] = 1;
                self.advance_follower_tail();
                self.tagalong_draw();
                return;
            }
        } else {
            if self.ram[LINK_PLAYER_HANDLER_STATE_TAGALONG] == K_PLAYER_STATE_HOOKSHOT {
                self.advance_follower_tail();
                self.tagalong_draw();
                return;
            }
            if self.ram[TAGALONG_VAR7_TAGALONG] != self.ram[TAGALONG_DATA_INDEX_TAGALONG] {
                self.ram[TAGALONG_DATA_INDEX_TAGALONG] =
                    if self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1) == 20 {
                        0
                    } else {
                        self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1)
                    };
                self.tagalong_draw();
                return;
            }
            self.ram[TAGALONG_VAR3_TAGALONG] = 0;
        }
        let k = self.ram[TAGALONG_DATA_INDEX_TAGALONG] as usize;
        if sign8(self.ram[TAGALONG_Z_TAGALONG + k]) == false
            && self.ram[TAGALONG_Z_TAGALONG + k] > 0
        {
            if self.ram[TAGALONG_VAR1_TAGALONG] != k as u8 {
                self.ram[TAGALONG_DATA_INDEX_TAGALONG] =
                    if self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1) == 20 {
                        0
                    } else {
                        self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1)
                    };
                self.tagalong_draw();
                return;
            }
            self.ram[TAGALONG_Z_TAGALONG + k] = 0;
            self.set_tagalong_y(k, read_le_u16(&self.ram, LINK_Y_COORD_TAGALONG));
            self.set_tagalong_x(k, read_le_u16(&self.ram, LINK_X_COORD_TAGALONG));
        }
        if (self.ram[LINK_X_VEL_TAGALONG] | self.ram[LINK_Y_VEL_TAGALONG]) != 0 {
            self.advance_follower_tail();
        }
        self.tagalong_draw();
    }

    fn advance_follower_tail(&mut self) {
        let mut t = self.ram[TAGALONG_VAR1_TAGALONG].wrapping_sub(15);
        if sign8(t) {
            t = t.wrapping_add(20);
        }
        if t == self.ram[TAGALONG_DATA_INDEX_TAGALONG] {
            self.ram[TAGALONG_DATA_INDEX_TAGALONG] =
                if self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1) == 20 {
                    0
                } else {
                    self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1)
                };
        }
    }

    pub(super) fn follower_not_following(&mut self) {
        if self.ram[SAVED_TAGALONG_INDOORS_TAGALONG] != self.ram[PLAYER_IS_INDOORS_TAGALONG] {
            return;
        }
        if self.ram[LINK_IS_RUNNING_TAGALONG] == 0 && !self.follower_check_proximity_to_link() {
            self.follower_initialize();
            self.ram[SAVED_TAGALONG_INDOORS_TAGALONG] = self.ram[PLAYER_IS_INDOORS_TAGALONG];
            if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 13 {
                self.ram[SUPER_BOMB_INDICATOR_TIMER_TAGALONG] = 254;
                self.ram[SUPER_BOMB_INDICATOR_COUNTER_TAGALONG] = 0;
            }
            self.ram[FOLLOWER_DROPPED_TAGALONG] = 0;
            self.tagalong_draw();
        } else {
            if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 13
                && self.ram[PLAYER_IS_INDOORS_TAGALONG] == 0
                && self.ram[SUPER_BOMB_INDICATOR_TIMER_TAGALONG] == 0
            {
                if self.AncillaAdd_SuperBombExplosion(0x3a, 0).is_some() {
                    self.ram[FOLLOWER_DROPPED_TAGALONG] = 0;
                    if self.read_u32_ram(ENHANCED_FEATURES0_TAGALONG)
                        & FEATURES0_MISC_BUG_FIXES_TAGALONG
                        != 0
                    {
                        self.ram[FOLLOWER_INDICATOR_TAGALONG] = 0;
                        return;
                    }
                } else {
                    self.ram[SUPER_BOMB_INDICATOR_COUNTER_TAGALONG] = 1;
                }
            }
            self.follower_do_layers();
        }
    }

    pub(super) fn follower_old_man(&mut self) {
        if !self.tagalong_is_following() {
            self.tagalong_draw();
            return;
        }
        if self.ram[LINK_SPEED_SETTING_TAGALONG] != 4 {
            self.ram[LINK_SPEED_SETTING_TAGALONG] = 12;
        }
        self.follower_handle_trigger();
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 0 {
            return;
        } else if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 4 {
            let k = self.ram[TAGALONG_DATA_INDEX_TAGALONG] as usize;
            if sign8(self.ram[TAGALONG_Z_TAGALONG + k]) == false
                && self.ram[TAGALONG_Z_TAGALONG + k] > 0
                && self.ram[TAGALONG_VAR1_TAGALONG] != self.ram[TAGALONG_DATA_INDEX_TAGALONG]
            {
                self.ram[TAGALONG_DATA_INDEX_TAGALONG] =
                    if self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1) >= 20 {
                        0
                    } else {
                        self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1)
                    };
                self.tagalong_draw();
                return;
            }
        } else {
            if (self.ram[LINK_AUXILIARY_STATE_TAGALONG] & 1) != 0
                && self.ram[LINK_PLAYER_HANDLER_STATE_TAGALONG] == K_PLAYER_STATE_RECOIL_OTHER
            {
                if self.ram[TAGALONG_VAR1_TAGALONG] == self.ram[TAGALONG_DATA_INDEX_TAGALONG] {
                    // C asserts here because the follower X value is undefined.
                    panic!("follower_old_man assert");
                }
                self.transform_old_man();
                return;
            }
            if self.ram[LINK_AUXILIARY_STATE_TAGALONG] & 2 != 0 {
                self.transform_old_man();
                return;
            }
        }
        if (self.ram[LINK_X_VEL_TAGALONG] | self.ram[LINK_Y_VEL_TAGALONG]) != 0 {
            let mut t = self.ram[TAGALONG_VAR1_TAGALONG].wrapping_sub(20);
            if sign8(t) {
                t = t.wrapping_add(20);
            }
            if t == self.ram[TAGALONG_DATA_INDEX_TAGALONG] {
                self.ram[TAGALONG_DATA_INDEX_TAGALONG] =
                    if self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1) >= 20 {
                        0
                    } else {
                        self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1)
                    };
            }
        } else if self.ram[FRAME_COUNTER_TAGALONG] & 3 == 0
            && self.ram[TAGALONG_VAR1_TAGALONG] != self.ram[TAGALONG_DATA_INDEX_TAGALONG]
        {
            let mut t = self.ram[TAGALONG_VAR1_TAGALONG].wrapping_sub(9);
            if sign8(t) {
                t = t.wrapping_add(20);
            }
            if t != self.ram[TAGALONG_DATA_INDEX_TAGALONG] {
                self.ram[TAGALONG_DATA_INDEX_TAGALONG] =
                    if self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1) >= 20 {
                        0
                    } else {
                        self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1)
                    };
            }
        }
        self.tagalong_draw();
    }

    fn transform_old_man(&mut self) {
        self.ram[FOLLOWER_INDICATOR_TAGALONG] =
            TAGALONG_TAB4[self.ram[FOLLOWER_INDICATOR_TAGALONG] as usize];
        self.ram[TIMER_TAGALONG_REACQUIRE_TAGALONG] = 64;
        let k = self.ram[TAGALONG_DATA_INDEX_TAGALONG] as usize;
        let y = self.tagalong_y(k);
        let x = self.tagalong_x(k);
        write_le_u16(&mut self.ram, SAVED_TAGALONG_Y_TAGALONG, y);
        write_le_u16(&mut self.ram, SAVED_TAGALONG_X_TAGALONG, x);
        self.ram[SAVED_TAGALONG_FLOOR_TAGALONG] = self.ram[LINK_IS_ON_LOWER_LEVEL_TAGALONG];
        self.follower_old_man_unused();
    }

    pub(super) fn follower_old_man_unused(&mut self) {
        self.ram[LINK_SPEED_SETTING_TAGALONG] = 16;
        if self.ram[LINK_IS_RUNNING_TAGALONG] == 0
            && self.ram[LINK_AUXILIARY_STATE_TAGALONG] == 0
            && self.ram[LINK_PLAYER_HANDLER_STATE_TAGALONG] != K_PLAYER_STATE_SWIMMING
        {
            self.ram[LINK_SPEED_SETTING_TAGALONG] = 0;
            if self.ram[LINK_PLAYER_HANDLER_STATE_TAGALONG] != K_PLAYER_STATE_HOOKSHOT
                && !self.follower_check_proximity_to_link()
            {
                self.follower_initialize();
                self.ram[FOLLOWER_INDICATOR_TAGALONG] =
                    TAGALONG_TAB5[self.ram[FOLLOWER_INDICATOR_TAGALONG] as usize];
                return;
            }
        }
        self.follower_do_layers();
    }

    pub(super) fn follower_do_layers(&mut self) {
        let floor = self.ram[SAVED_TAGALONG_FLOOR_TAGALONG] as usize;
        write_le_u16(
            &mut self.ram,
            OAM_PRIORITY_VALUE_TAGALONG,
            (TAGALONG_FLAGS[floor] as u16) << 8,
        );
        let a = if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 12
            || self.ram[FOLLOWER_INDICATOR_TAGALONG] == 13
        {
            2
        } else {
            1
        };
        self.follower_animate_movement_preserved(
            a,
            read_le_u16(&self.ram, SAVED_TAGALONG_X_TAGALONG),
            read_le_u16(&self.ram, SAVED_TAGALONG_Y_TAGALONG),
        );
    }

    pub(super) fn follower_check_proximity_to_link(&mut self) -> bool {
        self.ram[TIMER_TAGALONG_REACQUIRE_TAGALONG] =
            self.ram[TIMER_TAGALONG_REACQUIRE_TAGALONG].wrapping_sub(1);
        if !sign8(self.ram[TIMER_TAGALONG_REACQUIRE_TAGALONG]) {
            return true;
        }
        self.ram[TIMER_TAGALONG_REACQUIRE_TAGALONG] = 0;
        let y = read_le_u16(&self.ram, SAVED_TAGALONG_Y_TAGALONG);
        let x = read_le_u16(&self.ram, SAVED_TAGALONG_X_TAGALONG);
        let ly = read_le_u16(&self.ram, LINK_Y_COORD_TAGALONG);
        let lx = read_le_u16(&self.ram, LINK_X_COORD_TAGALONG);
        y.wrapping_sub(1) >= ly
            || y.wrapping_add(19) < ly
            || x.wrapping_sub(1) >= lx
            || x.wrapping_add(19) < lx
    }

    pub(super) fn follower_handle_trigger(&mut self) {
        if self.ram[SUBMODULE_INDEX_TAGALONG] != 0 {
            return;
        }
        let (infos, start, end) = if self.ram[PLAYER_IS_INDOORS_TAGALONG] != 0 {
            let room = read_le_u16(&self.ram, DUNGEON_ROOM_INDEX_TAGALONG);
            let Some(j) = TAGALONG_INDOOR_ROOMS.iter().position(|&v| v == room) else {
                return;
            };
            (
                &TAGALONG_INDOOR_INFOS[..],
                TAGALONG_INDOOR_OFFSETS[j] as usize,
                TAGALONG_INDOOR_OFFSETS[j + 1] as usize,
            )
        } else {
            let screen = read_le_u16(&self.ram, OVERWORLD_SCREEN_INDEX_TAGALONG);
            let Some(j) = TAGALONG_OUTDOOR_ROOMS.iter().position(|&v| v == screen) else {
                return;
            };
            (
                &TAGALONG_OUTDOOR_INFOS[..],
                TAGALONG_OUTDOOR_OFFSETS[j] as usize,
                TAGALONG_OUTDOOR_OFFSETS[j + 1] as usize,
            )
        };
        let st = if self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1) >= 20 {
            0
        } else {
            self.ram[TAGALONG_DATA_INDEX_TAGALONG].wrapping_add(1)
        };
        for info in &infos[start..end] {
            if info.tagalong == self.ram[FOLLOWER_INDICATOR_TAGALONG]
                && self.follower_check_for_trigger(info)
            {
                if info.bit & self.ram[TAGALONG_EVENT_FLAGS_TAGALONG] != 0 {
                    return;
                }
                self.ram[TAGALONG_EVENT_FLAGS_TAGALONG] |= info.bit;
                write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX_TAGALONG, info.msg);
                if info.msg == 0xffff {
                    if info.bit & 3 == 0 {
                        self.kiki_revert_to_sprite(st as usize);
                    } else if self.ram[SAVE_OW_EVENT_INFO_TAGALONG
                        + self.ram[OVERWORLD_SCREEN_INDEX_TAGALONG] as usize]
                        & 1
                        == 0
                    {
                        self.kiki_spawn_handler_a(st as usize);
                    }
                    return;
                }
                if info.msg == 0x9d {
                    self.OldMan_RevertToSprite(st as usize);
                } else if info.msg == 0x28 {
                    self.ram[FOLLOWER_INDICATOR_TAGALONG] = 0;
                }
                self.Tagalong_Main_ShowTextMessage();
                return;
            }
        }
    }

    pub(super) fn tagalong_draw(&mut self) {
        if self.ram[TAGALONG_APPEARANCE_NONE_FLAG_TAGALONG] != 0 {
            return;
        }
        let priority =
            if self.ram[TAGALONG_Z_TAGALONG + self.ram[TAGALONG_DATA_INDEX_TAGALONG] as usize] != 0
                && self.ram[PLAYER_IS_INDOORS_TAGALONG] == 0
            {
                0x20
            } else if self.ram[SUBMODULE_INDEX_TAGALONG] == 14 {
                TAGALONG_FLAGS[self.ram[LINK_IS_ON_LOWER_LEVEL_TAGALONG] as usize]
            } else {
                (self.ram
                    [TAGALONG_LAYERBITS_TAGALONG + self.ram[TAGALONG_DATA_INDEX_TAGALONG] as usize]
                    & 0x0c)
                    << 2
            };
        write_le_u16(
            &mut self.ram,
            OAM_PRIORITY_VALUE_TAGALONG,
            (priority as u16) << 8,
        );
        let k = if sign8(self.ram[TAGALONG_DATA_INDEX_TAGALONG]) {
            0
        } else {
            self.ram[TAGALONG_DATA_INDEX_TAGALONG] as usize
        };
        let x = self.tagalong_x(k);
        let y = self.tagalong_y(k);
        let a = self.ram[TAGALONG_LAYERBITS_TAGALONG + k];
        self.follower_animate_movement_preserved(a, x, y);
    }

    #[rustfmt::skip]
    pub(super) fn set_oam_follower(&mut self, oam: &mut OamEnt, x: u16, y: u16, charnum: u8, flags: u8, mut big: u8) {
        oam.x = x as u8;
        let visible = x.wrapping_add(0x80) < 0x180 && {
            big |= ((x >> 8) & 1) as u8;
            y.wrapping_add(0x10) < 0x100
        };
        oam.y = if visible { y as u8 } else { 0xf0 };
        oam.charnum = charnum;
        oam.flags = flags;
        let _ = big;
    }

    #[rustfmt::skip]
    fn set_oam_follower_at(&mut self, oam: usize, x: u16, y: u16, charnum: u8, flags: u8, mut big: u8) {
        self.ram[oam] = x as u8;
        let visible = x.wrapping_add(0x80) < 0x180 && {
            big |= ((x >> 8) & 1) as u8;
            y.wrapping_add(0x10) < 0x100
        };
        self.ram[oam + 1] = if visible { y as u8 } else { 0xf0 };
        self.ram[oam + 2] = charnum;
        self.ram[oam + 3] = flags;
        self.ram[BYTEWISE_EXTENDED_OAM_TAGALONG + (oam - OAM_BUF_TAGALONG) / 4] = big;
    }

    pub(super) fn follower_animate_movement_preserved(&mut self, ain: u8, xin: u16, yin: u16) {
        let mut yt = 0;
        let av;
        let mut sc = 0;
        if (ain >> 2 & 8) != 0
            && (self.ram[FOLLOWER_INDICATOR_TAGALONG] == 6
                || self.ram[FOLLOWER_INDICATOR_TAGALONG] == 1)
        {
            yt = 8;
            av = if self.ram[SWIM_ACCELERATION_TAGALONG] | self.ram[SWIM_ACCELERATION_TAGALONG + 1]
                != 0
            {
                (self.ram[FRAME_COUNTER_TAGALONG] >> 1) & 4
            } else {
                (self.ram[FRAME_COUNTER_TAGALONG] >> 2) & 4
            };
        } else if self.ram[SUBMODULE_INDEX_TAGALONG] == 8
            || self.ram[SUBMODULE_INDEX_TAGALONG] == 14
            || self.ram[SUBMODULE_INDEX_TAGALONG] == 16
        {
            av = if self.ram[LINK_IS_RUNNING_TAGALONG] != 0 {
                self.ram[FRAME_COUNTER_TAGALONG] & 4
            } else {
                (self.ram[FRAME_COUNTER_TAGALONG] >> 1) & 4
            };
        } else if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 11 {
            av = (self.ram[FRAME_COUNTER_TAGALONG] >> 1) & 4;
        } else if ((self.ram[FOLLOWER_INDICATOR_TAGALONG] == 12
            || self.ram[FOLLOWER_INDICATOR_TAGALONG] == 13)
            && self.ram[FOLLOWER_DROPPED_TAGALONG] != 0)
            || self.ram[FLAG_IS_LINK_IMMOBILIZED_TAGALONG] != 0
            || self.ram[SUBMODULE_INDEX_TAGALONG] == 10
            || (self.ram[MAIN_MODULE_INDEX_TAGALONG] == 9
                && self.ram[SUBMODULE_INDEX_TAGALONG] == 0x23)
            || (self.ram[MAIN_MODULE_INDEX_TAGALONG] == 14
                && (self.ram[SUBMODULE_INDEX_TAGALONG] == 1
                    || self.ram[SUBMODULE_INDEX_TAGALONG] == 2))
            || (self.ram[LINK_Y_VEL_TAGALONG] | self.ram[LINK_X_VEL_TAGALONG]) == 0
        {
            av = 4;
            sc = 4;
        } else {
            av = if self.ram[LINK_IS_RUNNING_TAGALONG] != 0 {
                self.ram[FRAME_COUNTER_TAGALONG] & 4
            } else {
                (self.ram[FRAME_COUNTER_TAGALONG] >> 1) & 4
            };
        }
        let frame = (ain & 3).wrapping_add(av).wrapping_add(yt) as usize;
        let spr_offs = if (read_le_u16(&self.ram, LINK_Y_COORD_TAGALONG) == yin && (ain & 3) == 0)
            || read_le_u16(&self.ram, LINK_Y_COORD_TAGALONG) < yin
        {
            TAGALONG_DRAW_SPR_OFFS0[self.ram[SORT_SPRITES_SETTING_TAGALONG] as usize] >> 2
        } else {
            TAGALONG_DRAW_SPR_OFFS1[self.ram[SORT_SPRITES_SETTING_TAGALONG] as usize] >> 2
        } as usize;
        write_le_u16(
            &mut self.ram,
            OAM_EXT_CUR_PTR_TAGALONG,
            0x0a20 + spr_offs as u16,
        );
        write_le_u16(
            &mut self.ram,
            OAM_CUR_PTR_TAGALONG,
            0x0800 + (spr_offs as u16) * 4,
        );
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR_TAGALONG) as usize;
        let scrolly = yin.wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2_TAGALONG));
        let scrollx = xin.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2_TAGALONG));
        let mut skip_first_sprites = false;
        let mut sk_index = 0usize;
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 1
            || self.ram[FOLLOWER_INDICATOR_TAGALONG] == 6
            || (ain & 0x20) == 0
        {
            if ain & 0xc0 == 0 {
                skip_first_sprites = true;
            } else if (ain & 0x80) == 0 {
                sk_index += 12;
                if sc != 0 {
                    self.ram[TAGALONG_DRAW_ANIM_FRAME] = 0;
                } else if self.ram[FRAME_COUNTER_TAGALONG] & 7 == 0 {
                    self.ram[TAGALONG_DRAW_ANIM_FRAME] =
                        self.ram[TAGALONG_DRAW_ANIM_FRAME].wrapping_add(1);
                    if self.ram[TAGALONG_DRAW_ANIM_FRAME] == 3 {
                        self.ram[TAGALONG_DRAW_ANIM_FRAME] = 0;
                    }
                }
            } else if self.ram[FRAME_COUNTER_TAGALONG] & 7 == 0 {
                self.ram[TAGALONG_DRAW_ANIM_FRAME] =
                    self.ram[TAGALONG_DRAW_ANIM_FRAME].wrapping_add(1);
                if self.ram[TAGALONG_DRAW_ANIM_FRAME] == 3 {
                    self.ram[TAGALONG_DRAW_ANIM_FRAME] = 0;
                }
            }
        } else if self.ram[FRAME_COUNTER_TAGALONG] & 7 == 0 {
            self.ram[TAGALONG_DRAW_ANIM_FRAME] = self.ram[TAGALONG_DRAW_ANIM_FRAME].wrapping_add(1);
            if self.ram[TAGALONG_DRAW_ANIM_FRAME] == 3 {
                self.ram[TAGALONG_DRAW_ANIM_FRAME] = 0;
            }
        }
        if !skip_first_sprites {
            sk_index += self.ram[TAGALONG_DRAW_ANIM_FRAME] as usize * 4;
            self.set_oam_follower_at(
                oam,
                scrollx,
                scrolly.wrapping_add(16),
                TAGALONG_DRAW_SPR_INFO0[sk_index],
                TAGALONG_DRAW_SPR_INFO0[sk_index + 1],
                0,
            );
            self.set_oam_follower_at(
                oam + 4,
                scrollx.wrapping_add(8),
                scrolly.wrapping_add(16),
                TAGALONG_DRAW_SPR_INFO0[sk_index + 2],
                TAGALONG_DRAW_SPR_INFO0[sk_index + 3],
                0,
            );
            oam += 8;
        }
        let mut pal = TAGALONG_DRAW_PALS[self.ram[FOLLOWER_INDICATOR_TAGALONG] as usize];
        if pal == 7 && self.ram[PALETTE_SWAP_FLAG_TAGALONG] != 0 {
            pal = 0;
        }
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 13 {
            let colorful = if self.read_u32_ram(ENHANCED_FEATURES0_TAGALONG)
                & FEATURES0_MISC_BUG_FIXES_TAGALONG
                != 0
            {
                self.ram[SUPER_BOMB_INDICATOR_TIMER_TAGALONG] <= 1
            } else {
                self.ram[SUPER_BOMB_INDICATOR_TIMER_TAGALONG] == 1
            };
            if colorful {
                pal = self.ram[FRAME_COUNTER_TAGALONG] & 7;
            }
        }
        let sprd = TAGALONG_DRAW_SPR_XY[frame
            + (TAGALONG_DRAW_OFFS[self.ram[FOLLOWER_INDICATOR_TAGALONG] as usize] >> 3) as usize];
        let sprf = TAGALONG_DMA_AND_FLAGS[frame];
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] != 12
            && self.ram[FOLLOWER_INDICATOR_TAGALONG] != 13
        {
            self.set_oam_follower_at(
                oam,
                scrollx.wrapping_add(sprd.x1 as i16 as u16),
                scrolly.wrapping_add(sprd.y1 as i16 as u16),
                0x20,
                (sprf.flags & 0xf0)
                    | (pal << 1)
                    | (read_le_u16(&self.ram, OAM_PRIORITY_VALUE_TAGALONG) >> 8) as u8,
                2,
            );
            oam += 4;
            self.ram[TAGALONG_DMA_HEAD_POINTER] = sprf.dma6;
        }
        self.set_oam_follower_at(
            oam,
            scrollx.wrapping_add(sprd.x2 as i16 as u16),
            scrolly.wrapping_add(sprd.y2 as i16 as u16).wrapping_add(8),
            0x22,
            ((sprf.flags & 0x0f) << 4)
                | (pal << 1)
                | (read_le_u16(&self.ram, OAM_PRIORITY_VALUE_TAGALONG) >> 8) as u8,
            2,
        );
        self.ram[TAGALONG_DMA_BODY_POINTER] = sprf.dma7;
    }

    pub(super) fn follower_check_for_trigger(&self, info: &TagalongMessageInfo) -> bool {
        let mut x = read_le_u16(&self.ram, LINK_X_COORD_TAGALONG)
            .wrapping_add(12)
            .wrapping_sub(info.x.wrapping_add(8));
        let mut y = read_le_u16(&self.ram, LINK_Y_COORD_TAGALONG)
            .wrapping_add(12)
            .wrapping_sub(info.y.wrapping_add(8));
        if sign16(x) {
            x = 0u16.wrapping_sub(x);
        }
        if sign16(y) {
            y = 0u16.wrapping_sub(y);
        }
        x < 24 && y < 28
    }

    pub(super) fn follower_disable(&mut self) {
        if self.ram[FOLLOWER_INDICATOR_TAGALONG] == 9 || self.ram[FOLLOWER_INDICATOR_TAGALONG] == 10
        {
            self.ram[FOLLOWER_INDICATOR_TAGALONG] = 0;
        }
    }

    pub(super) fn blind_spawn_from_maiden(&mut self, x: u16, y: u16) {
        let k = 0;
        self.ram[SPRITE_STATE_TAGALONG + k] = 9;
        self.ram[SPRITE_TYPE_TAGALONG + k] = 206;
        self.Tagalong_Sprite_SetX(k, x);
        self.Tagalong_Sprite_SetY(k, y.wrapping_sub(16));
        self.SpritePrep_LoadProperties(k);
        self.ram[SPRITE_DELAY_AUX2_TAGALONG + k] = 192;
        self.ram[SPRITE_GRAPHICS_TAGALONG + k] = 21;
        self.ram[SPRITE_D_TAGALONG + k] = 2;
        self.ram[SPRITE_IGNORE_PROJECTILE_TAGALONG + k] = 2;
        let dung_bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS_TAGALONG) | 0x2000;
        write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS_TAGALONG, dung_bits);
        self.ram[KIKI_ANIM_COUNTER_TAGALONG] = 0;
    }

    pub(super) fn kiki_revert_to_sprite(&mut self, k: usize) {
        if let Some(j) = self.kiki_spawn_handler_monke(k) {
            self.ram[SPRITE_SUBTYPE2_TAGALONG + j] = 1;
        }
        self.ram[FOLLOWER_INDICATOR_TAGALONG] = 0;
    }

    pub(super) fn kiki_spawn_handler_monke(&mut self, k: usize) -> Option<usize> {
        let Some((j, _info)) = self.Tagalong_Sprite_SpawnDynamically(k, 0xb6) else {
            return None;
        };
        self.ram[SPRITE_HEAD_DIR_TAGALONG + j] = self.ram[TAGALONG_LAYERBITS_TAGALONG + k] & 3;
        self.ram[SPRITE_D_TAGALONG + j] = self.ram[TAGALONG_LAYERBITS_TAGALONG + k] & 3;
        let x = self.tagalong_x(k);
        let y = self.tagalong_y(k);
        self.Tagalong_Sprite_SetX(j, x.wrapping_add(2));
        self.Tagalong_Sprite_SetY(j, y.wrapping_add(2));
        self.ram[SPRITE_FLOOR_TAGALONG + j] = self.ram[LINK_IS_ON_LOWER_LEVEL_TAGALONG];
        self.ram[SPRITE_IGNORE_PROJECTILE_TAGALONG + j] = 1;
        self.ram[SPRITE_FLOOR_TAGALONG + j] = 2;
        self.ram[LINK_SPEED_SETTING_TAGALONG] = 0;
        Some(j)
    }

    pub(super) fn kiki_spawn_handler_a(&mut self, k: usize) {
        if let Some(j) = self.kiki_spawn_handler_monke(k) {
            self.ram[SPRITE_SUBTYPE2_TAGALONG + j] = 2;
        }
    }

    pub(super) fn kiki_spawn_handler_b(&mut self, k: usize) {
        if let Some(j) = self.kiki_spawn_handler_monke(k) {
            self.ram[SPRITE_Z_TAGALONG + j] = 1;
            self.ram[SPRITE_Z_VEL_TAGALONG + j] = 16;
            self.ram[SPRITE_SUBTYPE2_TAGALONG + j] = 3;
        }
        self.ram[FOLLOWER_INDICATOR_TAGALONG] = 0;
    }

    fn tagalong_x(&self, k: usize) -> u16 {
        self.ram[TAGALONG_X_LO_TAGALONG + k] as u16
            | ((self.ram[TAGALONG_X_HI_TAGALONG + k] as u16) << 8)
    }

    fn tagalong_y(&self, k: usize) -> u16 {
        self.ram[TAGALONG_Y_LO_TAGALONG + k] as u16
            | ((self.ram[TAGALONG_Y_HI_TAGALONG + k] as u16) << 8)
    }

    fn set_tagalong_x(&mut self, k: usize, x: u16) {
        self.ram[TAGALONG_X_LO_TAGALONG + k] = x as u8;
        self.ram[TAGALONG_X_HI_TAGALONG + k] = (x >> 8) as u8;
    }

    fn set_tagalong_y(&mut self, k: usize, y: u16) {
        self.ram[TAGALONG_Y_LO_TAGALONG + k] = y as u8;
        self.ram[TAGALONG_Y_HI_TAGALONG + k] = (y >> 8) as u8;
    }

    fn Tagalong_Main_ShowTextMessage(&mut self) {
        if self.ram[MAIN_MODULE_INDEX_TAGALONG] != 14 {
            self.ram[TAGALONG_MESSAGE_RESET_FLAG] = 0;
            self.ram[MESSAGING_MODULE_TAGALONG] = 0;
            self.ram[SUBMODULE_INDEX_TAGALONG] = 2;
            self.ram[SAVED_MODULE_FOR_MENU_TAGALONG] = self.ram[MAIN_MODULE_INDEX_TAGALONG];
            self.ram[MAIN_MODULE_INDEX_TAGALONG] = 14;
        }
    }

    fn Ancilla_GetX(&self, k: usize) -> u16 {
        self.ram[ANCILLA_X_LO_TAGALONG + k] as u16
            | ((self.ram[ANCILLA_X_HI_TAGALONG + k] as u16) << 8)
    }

    fn Ancilla_GetY(&self, k: usize) -> u16 {
        self.ram[ANCILLA_Y_LO_TAGALONG + k] as u16
            | ((self.ram[ANCILLA_Y_HI_TAGALONG + k] as u16) << 8)
    }

    fn Ancilla_SetXY(&mut self, k: usize, x: u16, y: u16) {
        self.ram[ANCILLA_X_LO_TAGALONG + k] = x as u8;
        self.ram[ANCILLA_X_HI_TAGALONG + k] = (x >> 8) as u8;
        self.ram[ANCILLA_Y_LO_TAGALONG + k] = y as u8;
        self.ram[ANCILLA_Y_HI_TAGALONG + k] = (y >> 8) as u8;
    }

    fn Ancilla_MoveX(&mut self, k: usize) {
        let pos = self.ram[ANCILLA_X_SUBPIXEL_TAGALONG + k] as u32
            | ((self.ram[ANCILLA_X_LO_TAGALONG + k] as u32) << 8)
            | ((self.ram[ANCILLA_X_HI_TAGALONG + k] as u32) << 16);
        let delta = ((self.ram[ANCILLA_X_VEL_TAGALONG + k] as i8 as i32) << 4) as u32;
        let moved = pos.wrapping_add(delta);
        self.ram[ANCILLA_X_SUBPIXEL_TAGALONG + k] = moved as u8;
        self.ram[ANCILLA_X_LO_TAGALONG + k] = (moved >> 8) as u8;
        self.ram[ANCILLA_X_HI_TAGALONG + k] = (moved >> 16) as u8;
    }

    fn Ancilla_MoveY(&mut self, k: usize) {
        let pos = self.ram[ANCILLA_Y_SUBPIXEL_TAGALONG + k] as u32
            | ((self.ram[ANCILLA_Y_LO_TAGALONG + k] as u32) << 8)
            | ((self.ram[ANCILLA_Y_HI_TAGALONG + k] as u32) << 16);
        let delta = ((self.ram[ANCILLA_Y_VEL_TAGALONG + k] as i8 as i32) << 4) as u32;
        let moved = pos.wrapping_add(delta);
        self.ram[ANCILLA_Y_SUBPIXEL_TAGALONG + k] = moved as u8;
        self.ram[ANCILLA_Y_LO_TAGALONG + k] = (moved >> 8) as u8;
        self.ram[ANCILLA_Y_HI_TAGALONG + k] = (moved >> 16) as u8;
    }

    fn Ancilla_ProjectSpeedTowardsPlayer(&self, k: usize, vel: u8) -> ProjectSpeedRet {
        if vel == 0 {
            return ProjectSpeedRet {
                x: 0,
                y: 0,
                xdiff: 0,
                ydiff: 0,
            };
        }
        let below_b =
            read_le_u16(&self.ram, LINK_Y_COORD_TAGALONG).wrapping_sub(self.Ancilla_GetY(k)) as u8;
        let below_a = sign16(
            read_le_u16(&self.ram, LINK_Y_COORD_TAGALONG).wrapping_sub(self.Ancilla_GetY(k)),
        ) as u8;
        let right_b =
            read_le_u16(&self.ram, LINK_X_COORD_TAGALONG).wrapping_sub(self.Ancilla_GetX(k)) as u8;
        let right_a = sign16(
            read_le_u16(&self.ram, LINK_X_COORD_TAGALONG).wrapping_sub(self.Ancilla_GetX(k)),
        ) as u8;
        let mut r12 = if sign8(below_b) {
            0u8.wrapping_sub(below_b)
        } else {
            below_b
        };
        let mut r13 = if sign8(right_b) {
            0u8.wrapping_sub(right_b)
        } else {
            right_b
        };
        let mut swapped = false;
        if r13 < r12 {
            swapped = true;
            core::mem::swap(&mut r12, &mut r13);
        }
        let mut xvel = vel;
        let mut yvel = 0u8;
        let mut t = 0u8;
        let mut n = vel;
        loop {
            t = t.wrapping_add(r12);
            if t >= r13 {
                t = t.wrapping_sub(r13);
                yvel = yvel.wrapping_add(1);
            }
            n = n.wrapping_sub(1);
            if n == 0 {
                break;
            }
        }
        if swapped {
            core::mem::swap(&mut xvel, &mut yvel);
        }
        ProjectSpeedRet {
            x: if right_a != 0 {
                0u8.wrapping_sub(xvel)
            } else {
                xvel
            },
            y: if below_a != 0 {
                0u8.wrapping_sub(yvel)
            } else {
                yvel
            },
            xdiff: right_b,
            ydiff: below_b,
        }
    }

    fn AncillaAdd_SuperBombExplosion(&mut self, a: u8, y: u8) -> Option<usize> {
        let k = self.ancilla_add_simple(a, y)?;
        self.ram[ANCILLA_R_TAGALONG + k] = 0;
        self.ram[ANCILLA_STEP_TAGALONG + k] = 0;
        self.ram[ANCILLA_ARR25_TAGALONG + k] = 0;
        self.ram[ANCILLA_L_TAGALONG + k] = 0;
        self.ram[ANCILLA_ARR3_TAGALONG + k] = 6;
        self.ram[ANCILLA_ITEM_TO_LINK_TAGALONG + k] = 1;
        let j = read_le_u16(&self.ram, TAGALONG_DATA_INDEX_TAGALONG) as usize;
        let y = self.tagalong_y(j);
        let x = self.tagalong_x(j);
        self.Ancilla_SetXY(k, x.wrapping_add(8), y.wrapping_add(16));
        Some(k)
    }

    fn Tagalong_Sprite_GetX(&self, k: usize) -> u16 {
        self.ram[SPRITE_X_LO_TAGALONG + k] as u16
            | ((self.ram[SPRITE_X_HI_TAGALONG + k] as u16) << 8)
    }

    fn Tagalong_Sprite_GetY(&self, k: usize) -> u16 {
        self.ram[SPRITE_Y_LO_TAGALONG + k] as u16
            | ((self.ram[SPRITE_Y_HI_TAGALONG + k] as u16) << 8)
    }

    fn Tagalong_Sprite_SetX(&mut self, k: usize, x: u16) {
        self.ram[SPRITE_X_LO_TAGALONG + k] = x as u8;
        self.ram[SPRITE_X_HI_TAGALONG + k] = (x >> 8) as u8;
    }

    fn Tagalong_Sprite_SetY(&mut self, k: usize, y: u16) {
        self.ram[SPRITE_Y_LO_TAGALONG + k] = y as u8;
        self.ram[SPRITE_Y_HI_TAGALONG + k] = (y >> 8) as u8;
    }

    fn Tagalong_Sprite_SpawnDynamically(
        &mut self,
        k: usize,
        sprite: u8,
    ) -> Option<(usize, SpriteSpawnInfo)> {
        let j = (0..16)
            .rev()
            .find(|&j| self.ram[SPRITE_STATE_TAGALONG + j] == 0)?;
        self.ram[SPRITE_STATE_TAGALONG + j] = 9;
        self.ram[SPRITE_TYPE_TAGALONG + j] = sprite;
        self.SpritePrep_LoadProperties(j);
        if self.ram[PLAYER_IS_INDOORS_TAGALONG] == 0 {
            write_le_u16(&mut self.ram, SPRITE_N_WORD_TAGALONG + j * 2, 0xffff);
        } else {
            self.ram[SPRITE_N_TAGALONG + j] = 0xff;
        }
        self.ram[SPRITE_FLOOR_TAGALONG + j] = self.ram[SPRITE_FLOOR_TAGALONG + k];
        self.ram[SPRITE_D_TAGALONG + j] = self.ram[SPRITE_D_TAGALONG + k];
        self.ram[SPRITE_DIE_ACTION_TAGALONG + j] = 0;
        self.ram[SPRITE_SUBTYPE_TAGALONG + j] = 0;
        Some((
            j,
            SpriteSpawnInfo {
                r0_x: self.tagalong_x(k),
                r2_y: self.tagalong_y(k),
            },
        ))
    }

    fn SpritePrep_LoadProperties(&mut self, k: usize) {
        self.sprite_prep_load_properties(k);
    }

    fn OldMan_RevertToSprite(&mut self, k: usize) {
        if let Some((j, _info)) = self.Tagalong_Sprite_SpawnDynamically(k, 0xad) {
            self.ram[SPRITE_D_TAGALONG + j] = self.ram[TAGALONG_LAYERBITS_TAGALONG + k] & 3;
            self.ram[SPRITE_HEAD_DIR_TAGALONG + j] = self.ram[TAGALONG_LAYERBITS_TAGALONG + k] & 3;
            self.Tagalong_Sprite_SetY(j, self.tagalong_y(k).wrapping_add(2));
            self.Tagalong_Sprite_SetX(j, self.tagalong_x(k).wrapping_add(2));
            self.ram[SPRITE_FLOOR_TAGALONG + j] = self.ram[LINK_IS_ON_LOWER_LEVEL_TAGALONG];
            self.ram[SPRITE_IGNORE_PROJECTILE_TAGALONG + j] = 1;
            self.ram[SPRITE_SUBTYPE2_TAGALONG + j] = 1;
        }
        self.OldMan_EnableCutscene();
        self.ram[FOLLOWER_INDICATOR_TAGALONG] = 0;
        self.ram[LINK_SPEED_SETTING_TAGALONG] = 0;
    }

    fn OldMan_EnableCutscene(&mut self) {
        self.ram[FLAG_IS_LINK_IMMOBILIZED_TAGALONG] = 1;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE_TAGALONG] = 1;
    }
}
