// Methods ported from zelda3/src/ancilla.c and included inside ZeldaState.

use super::*;
use crate::types::{
    abs16, abs8, sign16, sign8, AncillaRadialProjection, PairU8, Point16U, ProjectSpeedRet,
    SpriteHitBox,
};
use crate::zelda_rtl::sprite::SpriteSpawnInfo;

const ANCILLA_Z_SUBPIXEL_PLAYER: usize = 0x02a8;
const ANCILLA_TILE_ATTR_PLAYER: usize = 0x03e4;
const ANCILLA_ALLOC_ROTATE_PLAYER: usize = 0x03c4;
const ANCILLA_S_PLAYER: usize = 0x03a9;
const ANCILLA_T_PLAYER: usize = 0x03d5;
const ANCILLA_R_PLAYER: usize = 0x03ea;
const DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER: usize = 0x0646;
const ANCILLA_INTERACTIVE_RESET_FLAG: usize = 0x02f3;
const SPRITE_TILETYPE_ANCILLA: usize = 0x0fa5;
const TMP_COUNTER_ANCILLA: usize = 0x0fb5;
const OW_SCROLL_VARS0_YSTART: usize = 0x0600;
const OW_SCROLL_VARS0_YEND: usize = 0x0602;
const OW_SCROLL_VARS0_XSTART: usize = 0x0604;
const OW_SCROLL_VARS0_XEND: usize = 0x0606;
const CURRENT_AREA_OF_PLAYER_ANCILLA: usize = 0x0700;
const OVERWORLD_RIGHT_BOTTOM_BOUND_FOR_SCROLL_ANCILLA: usize = 0x0716;
const ANCILLA_ARR25: usize = 0x0746;
const ANCILLA_ARR22: usize = 0x074b;
const ANCILLA_ARR23: usize = 0x03cf;
const BOOMERANG_TEMP_Y: usize = 0x0399;
const BOOMERANG_TEMP_X: usize = 0x039b;
// Single-use coordinate scratch for arrow setup; NES_Ver2 aliases are broader shared work RAM.
const SCRATCH_0_ANCILLA: usize = 0x0072;
const SCRATCH_1_ANCILLA: usize = 0x0074;
const INDEX_OF_INTERACTING_TILE_ANCILLA: usize = 0x0076;
const SPRITE_IGNORE_PROJECTILE_ANCILLA: usize = 0x0ba0;
const REPULSESPARK_FLOOR_STATUS_ANCILLA: usize = 0x0b68;
const REPULSESPARK_TIMER_ANCILLA: usize = 0x0fac;
const REPULSESPARK_X_LO_ANCILLA: usize = 0x0fad;
const REPULSESPARK_Y_LO_ANCILLA: usize = 0x0fae;
const REPULSESPARK_ANIM_DELAY_ANCILLA: usize = 0x0faf;
const SPRITE_FLAGS_ANCILLA: usize = 0x0b6b;
const SPRITE_UNK2_ANCILLA: usize = 0x0bb0;
const SPRITE_GIVE_DAMAGE_ANCILLA: usize = 0x0ce2;
const DAMAGE_TYPE_DETERMINER_ANCILLA: usize = 0x0cf2;
const SPRITE_B_ANCILLA: usize = 0x0da0;
const SPRITE_C_ANCILLA: usize = 0x0db0;
const SPRITE_BUMP_DAMAGE_ANCILLA: usize = 0x0cd2;
const SPRITE_HEALTH_ANCILLA: usize = 0x0e50;
const SPRITE_HEAD_DIR_ANCILLA: usize = 0x0eb0;
const SPRITE_F_ANCILLA: usize = 0x0ea0;
const SPRITE_G_ANCILLA: usize = 0x0ed0;
const SPRITE_DELAY_AUX2_ANCILLA: usize = 0x0e10;
const SPRITE_DELAY_AUX3_ANCILLA: usize = 0x0ee0;
const SPRITE_HIT_TIMER_ANCILLA: usize = 0x0ef0;
const SPRITE_Y_RECOIL_ANCILLA: usize = 0x0f30;
const SPRITE_OAM_FLAGS_ANCILLA: usize = 0x0f50;
const GARNISH_ACTIVE_ANCILLA: usize = 0x0fb4;
const GARNISH_Y_LO_ANCILLA: usize = 0x1f81e;
const GARNISH_X_LO_ANCILLA: usize = 0x1f83c;
const GARNISH_Y_HI_ANCILLA: usize = 0x1f85a;
const GARNISH_X_HI_ANCILLA: usize = 0x1f878;
const GARNISH_SPRITE_ANCILLA: usize = 0x1f8b4;
const GARNISH_COUNTDOWN_ANCILLA: usize = 0x1f90e;
const DOOR_DEBRIS_DIRECTION: usize = 0x073c;
const HAPPINESS_POND_Y_VEL: usize = 0x15800;
const HAPPINESS_POND_X_VEL: usize = 0x1580c;
const HAPPINESS_POND_Z_VEL: usize = 0x15818;
const HAPPINESS_POND_Y_LO: usize = 0x15824;
const HAPPINESS_POND_Y_HI: usize = 0x15830;
const HAPPINESS_POND_X_LO: usize = 0x1583c;
const HAPPINESS_POND_X_HI: usize = 0x15848;
const HAPPINESS_POND_Z: usize = 0x15854;
const HAPPINESS_POND_TIMER: usize = 0x15860;
const HAPPINESS_POND_ARR1: usize = 0x1586c;
const HAPPINESS_POND_ITEM_TO_LINK: usize = 0x1587a;
const HAPPINESS_POND_Y_SUBPIXEL: usize = 0x15886;
const HAPPINESS_POND_X_SUBPIXEL: usize = 0x15892;
const HAPPINESS_POND_Z_SUBPIXEL: usize = 0x1589e;
const HAPPINESS_POND_STEP: usize = 0x158aa;
const SWORDBEAM_ARR: usize = 0x15800;
const SWORDBEAM_VAR1: usize = 0x15804;
const SWORDBEAM_VAR2: usize = 0x15808;
const SWORDBEAM_TEMP_X: usize = 0x1580e;
const SWORDBEAM_TEMP_Y: usize = 0x15810;
const QUAKE_ARR1: usize = 0x15800;
const QUAKE_ARR2: usize = 0x15805;
const QUAKE_VAR5: usize = 0x1580a;
const QUAKE_VAR1: usize = 0x1580b;
const QUAKE_VAR2: usize = 0x1580d;
const QUAKE_VAR4: usize = 0x1580f;
const QUAKE_VAR3: usize = 0x1581e;
const ETHER_ARR1: usize = 0x15800;
const ETHER_VAR2: usize = 0x15808;
const ETHER_Y2: usize = 0x1580a;
const ETHER_Y_ADJUSTED: usize = 0x1580c;
const ETHER_X2: usize = 0x1580e;
const ETHER_Y3: usize = 0x15810;
const ETHER_VAR1: usize = 0x15812;
const ETHER_Y: usize = 0x15813;
const ETHER_X: usize = 0x15815;
const BOMBOS_ARR1: usize = 0x15800;
const BOMBOS_ARR2: usize = 0x15810;
const BOMBOS_ARR7: usize = 0x15820;
const BOMBOS_Y_LO: usize = 0x15824;
const BOMBOS_Y_HI: usize = 0x15864;
const BOMBOS_X_LO: usize = 0x158a4;
const BOMBOS_X_HI: usize = 0x158e4;
const BOMBOS_Y_COORD2: usize = 0x15924;
const BOMBOS_X_COORD2: usize = 0x1592c;
const BOMBOS_VAR4: usize = 0x15934;
const BOMBOS_ARR3: usize = 0x15935;
const BOMBOS_ARR4: usize = 0x15945;
const BOMBOS_Y_COORD: usize = 0x15955;
const BOMBOS_X_COORD: usize = 0x159d5;
const BOMBOS_VAR3: usize = 0x15a55;
const BOMBOS_VAR2: usize = 0x15a56;
const BOMBOS_VAR1: usize = 0x15a57;
const BREAKTOWERSEAL_VAR3: usize = 0x15800;
const BREAKTOWERSEAL_VAR4: usize = 0x15808;
const BREAKTOWERSEAL_X: usize = 0x1580e;
const BREAKTOWERSEAL_Y: usize = 0x15810;
const BREAKTOWERSEAL_VAR5: usize = 0x15812;
const BREAKTOWERSEAL_BASE_SPARKLE_Y_LO: usize = 0x15817;
const BREAKTOWERSEAL_BASE_SPARKLE_Y_HI: usize = 0x1581f;
const BREAKTOWERSEAL_BASE_SPARKLE_X_LO: usize = 0x15827;
const BREAKTOWERSEAL_BASE_SPARKLE_X_HI: usize = 0x1582f;
const BREAKTOWERSEAL_SPARKLE_VAR1: usize = 0x15837;
const BREAKTOWERSEAL_SPARKLE_Y_LO: usize = 0x1584f;
const BREAKTOWERSEAL_SPARKLE_Y_HI: usize = 0x15867;
const BREAKTOWERSEAL_SPARKLE_X_LO: usize = 0x1587f;
const BREAKTOWERSEAL_SPARKLE_X_HI: usize = 0x15897;
const BREAKTOWERSEAL_SPARKLE_VAR2: usize = 0x158af;
const BLASTWALL_VAR5: usize = 0x10000;
const BLASTWALL_VAR6: usize = 0x10008;
const BLASTWALL_VAR1: usize = 0x10010;
const BLASTWALL_VAR4: usize = 0x10011;
const BLASTWALL_VAR8: usize = 0x10018;
const BLASTWALL_VAR9: usize = 0x1001a;
const BLASTWALL_VAR7: usize = 0x1001c;
const BLASTWALL_VAR10: usize = 0x10020;
const BLASTWALL_VAR11: usize = 0x10030;
const BLASTWALL_VAR12: usize = 0x10040;
const SKULLWOODSFIRE_VAR0: usize = 0x10000;
const SKULLWOODSFIRE_VAR5: usize = 0x10008;
const SKULLWOODSFIRE_VAR4: usize = 0x10010;
const SKULLWOODSFIRE_VAR9: usize = 0x10018;
const SKULLWOODSFIRE_VAR11: usize = 0x1001a;
const SKULLWOODSFIRE_VAR10: usize = 0x10026;
const SKULLWOODSFIRE_VAR12: usize = 0x10036;
const SKULLWOODSFIRE_Y_ARR: usize = 0x10020;
const SKULLWOODSFIRE_X_ARR: usize = 0x10030;
const ANCILLA_ARR26: usize = 0x0741;
const SAVE_OW_EVENT_INFO_ANCILLA: usize = 0x0f280;
const TAGALONG_DATA_INDEX_ANCILLA: usize = 0x02cf;
const TAGALONG_APPEARANCE_NONE_FLAG_ANCILLA: usize = 0x02f9;
const TAGALONG_Y_LO_ANCILLA: usize = 0x1a00;
const TAGALONG_Y_HI_ANCILLA: usize = 0x1a14;
const TAGALONG_X_LO_ANCILLA: usize = 0x1a28;
const TAGALONG_X_HI_ANCILLA: usize = 0x1a3c;
const FOLLOWER_INDICATOR_ANCILLA: usize = 0x0f3cc;
const FLAG_TRAVEL_BIRD: usize = 0x0af4;
const MILESTONE_ITEM_GFX_SWAP_COUNTDOWN: usize = 0x04c2;
const TRIGGER_SPECIAL_ENTRANCE_ANCILLA: usize = 0x04c6;
const WEATHERVANE_ARR3: usize = 0x15800;
const WEATHERVANE_ARR4: usize = 0x1580c;
const WEATHERVANE_ARR5: usize = 0x15818;
const WEATHERVANE_ARR6: usize = 0x15824;
const WEATHERVANE_ARR7: usize = 0x15830;
const WEATHERVANE_ARR8: usize = 0x1583c;
const WEATHERVANE_ARR9: usize = 0x15848;
const WEATHERVANE_ARR10: usize = 0x15854;
const WEATHERVANE_ARR11: usize = 0x15860;
const WEATHERVANE_ARR12: usize = 0x1586c;
const WEATHERVANE_VAR13: usize = 0x15878;
const WEATHERVANE_VAR14: usize = 0x15879;
const WEATHERVANE_VAR2: usize = 0x158b6;
const WEATHERVANE_VAR1: usize = 0x158b8;
const MAGIC_SPELL_PLAYER_LOCK_FLAG: usize = 0x0325;

const K_BOMBOS_SFX: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
const K_BOMBOS_BLASTS_TAB: [u8; 72] = [
    0xb6, 0x5d, 0xa1, 0x30, 0x69, 0xb5, 0xa3, 0x24, 0x96, 0xac, 0x73, 0x5f, 0x92, 0x48, 0x52, 0x81,
    0x39, 0x95, 0x7f, 0x20, 0x88, 0x5d, 0x34, 0x98, 0xbc, 0xd2, 0x51, 0x77, 0xa2, 0x47, 0x94, 0xb2,
    0x34, 0xda, 0x30, 0x62, 0x9f, 0x76, 0x51, 0x46, 0x98, 0x5c, 0x9b, 0x61, 0x58, 0x95, 0x4c, 0xba,
    0x7e, 0xcb, 0x12, 0xd0, 0x70, 0xa6, 0x46, 0xbf, 0x40, 0x50, 0x7e, 0x8c, 0x2d, 0x61, 0xac, 0x88,
    0x20, 0x6a, 0x72, 0x5f, 0xd2, 0x28, 0x52, 0x80,
];
const K_QUAKE_TAB1: [u8; 5] = [0x17, 0x16, 0x17, 0x16, 0x10];
const K_QUAKE_DRAW_GROUND_BOLTS_CHAR: [u8; 15] = [
    0x40, 0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 0x60, 0x62, 0x64, 0x66, 0x68, 0x6a, 0x63,
];
const K_QUAKE_ITEMS: [i16; 453] = [
    0, -16, 0, 0, -16, 1, 0, -16, 2, 0, -16, 3, 0, -16, 67, 0, -16, 66, 0, -16, 65, 0, -16, 64, 0,
    -16, 64, 14, -8, 132, 29, -8, 68, 13, -7, 132, 31, -7, 68, 47, -4, 132, 49, -11, 6, 63, -5, 68,
    47, -4, 132, 36, -17, 8, 49, -11, 6, 63, -5, 68, 78, 4, 8, 22, -31, 8, 36, -17, 8, 78, 4, 8,
    93, 20, 8, 7, -46, 8, 23, -45, 72, 22, -31, 8, 93, 20, 8, 93, 36, 72, -7, -61, 8, 37, -59, 72,
    7, -46, 8, 23, -45, 72, 93, 36, 72, 93, 52, 8, -22, -75, 8, 47, -74, 1, -8, -61, 8, 36, -60,
    72, 93, 52, 8, 108, 67, 8, -37, -90, 8, -22, -75, 8, 47, -74, 1, 59, -62, 129, 108, 67, 8, 121,
    80, 8, -44, -104, 201, -37, -90, 8, 73, -74, 72, 59, -62, 129, 121, 80, 8, -44, -120, 9, -44,
    -104, 201, 87, -89, 72, 73, -74, 72, -44, -120, 9, 102, -104, 72, 87, -89, 72, 102, -104, 72,
    87, -89, 72, 112, -116, 72, 102, -104, 72, 112, -116, 72, -13, -16, 0, -13, -16, 1, -13, -16,
    2, -13, -16, 3, -11, -16, 67, -11, -16, 66, -11, -16, 65, -11, -16, 64, -24, -10, 4, -38, -18,
    8, -24, -10, 4, -40, -7, 196, -45, -33, 201, -38, -18, 8, -57, -7, 4, -40, -7, 196, -48, -45,
    7, -45, -33, 201, -57, -7, 4, -71, 2, 72, -48, -45, 6, -71, 2, 72, -70, 18, 8, -48, -45, 5,
    -70, 18, 8, -56, 33, 8, -48, -45, 7, -54, 34, 8, -54, 49, 136, -48, -45, 6, -54, 49, 136, -69,
    64, 136, -48, -45, 7, -69, 64, 136, -85, 73, 196, -48, -45, 5, -101, 73, 4, -85, 73, 196, -60,
    -53, 8, -48, -45, 6, -101, 73, 4, -116, 77, 196, -75, -67, 8, -60, -53, 8, -128, 76, 4, -116,
    77, 196, -90, -82, 8, -75, -67, 8, -128, 76, 4, -105, -97, 8, -90, -82, 8, -120, -111, 8, -105,
    -97, 8, -120, -111, 8, 0, -5, 10, 0, -5, 11, 2, -3, 12, 1, -3, 13, 0, -3, 141, 1, -3, 140, 1,
    -3, 139, 1, -3, 138, -6, 12, 137, -6, 12, 137, -10, 28, 201, -10, 28, 73, -8, 44, 137, -8, 44,
    137, -10, 56, 2, -10, 56, 2, -23, 70, 72, 5, 70, 8, -23, 70, 72, 5, 70, 8, -38, 85, 72, 19, 85,
    8, -38, 85, 72, 19, 85, 8, -52, 99, 72, 33, 101, 8, -52, 99, 72, 33, 101, 8, -66, 113, 72, 47,
    115, 8, -66, 113, 72, 47, 115, 8,
];
const K_QUAKE_ITEMS2: [i16; 312] = [
    -96, 112, 32, -96, 112, 33, -96, 112, 102, -96, 112, 34, -96, 112, 35, -96, 112, 99, -96, 112,
    98, -96, 112, 38, -96, 112, 39, -86, 124, 40, -86, 124, 40, -72, -117, 40, -72, -117, 40, -59,
    -102, 161, -59, -102, 161, -44, -116, 104, -44, -116, 104, -29, 126, 104, -29, 126, 104, -19,
    125, 197, -112, 96, 42, -112, 96, 43, -112, 96, 44, -112, 96, 45, -119, 82, 41, -112, 96, 42,
    -123, 66, 233, -119, 82, 41, -121, 50, 41, -123, 66, 233, 126, 34, 40, -115, 34, 104, -121, 50,
    41, -106, 18, 169, 111, 19, 40, 126, 34, 40, -115, 34, 104, -100, 2, 104, 102, 4, 233, -106,
    18, 169, 111, 19, 40, -91, -14, 169, 95, -11, 40, -100, 2, 104, 102, 4, 233, 96, 112, 96, 96,
    112, 97, 96, 112, 38, 96, 112, 98, 96, 112, 99, 96, 112, 35, 96, 112, 34, 96, 112, 102, 85,
    111, 232, 96, 112, 103, 70, 104, 36, 85, 111, 232, 70, 104, 36, 54, 108, 228, 40, 100, 40, 38,
    107, 36, 54, 108, 228, 25, 85, 40, 40, 100, 40, 38, 107, 36, 22, 110, 228, 11, 70, 40, 25, 85,
    40, 7, 108, 36, 22, 110, 228, 11, 70, 40, 7, 108, 36, 112, 112, 42, 112, 112, 43, 112, 112, 44,
    112, 112, 45, 112, 112, 42, 108, 125, 41, 108, 125, 41, 114, -116, 40, 114, -116, 40, 124,
    -100, 41, 124, -100, 41, 123, -84, 233, 123, -84, 233, 117, -74, 228, -124, -69, 40, 117, -74,
    228, -124, -69, 40, 103, -67, 104, -110, -54, 40, 103, -67, 104, -110, -54, 40, 95, -52, 105,
    -102, -39, 41, 95, -52, 105, -102, -39, 41, 96, -36, 233, -102, -24, 233, 96, -36, 233, -102,
    -24, 233, -123, -14, 41, -115, -14, 46, 49, -12, 40,
];
const K_QUAKE_ITEM_POS: [u8; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 17, 21, 25, 30, 36, 42, 48, 53, 57, 60, 62, 64, 65, 66,
    67, 68, 69, 70, 71, 72, 74, 77, 81, 85, 88, 91, 94, 97, 100, 103, 107, 111, 114, 116, 118, 119,
    120, 121, 122, 123, 124, 125, 126, 128, 130, 132, 134, 137, 141, 145, 149, 151,
];
const K_QUAKE_ITEM_POS2: [u8; 56] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 18, 19, 20, 21, 22, 23, 24, 26, 28, 30, 33, 37, 41,
    45, 46, 47, 48, 49, 50, 51, 52, 53, 55, 57, 59, 62, 66, 70, 72, 73, 74, 75, 76, 78, 80, 82, 84,
    87, 91, 95, 99, 101, 104,
];
const K_RECEIVE_ITEM_TAB4: [u8; 3] = [9, 5, 5];
const K_RECEIVE_ITEM_TAB5: [u8; 3] = [0x24, 0x25, 0x26];
const K_RECEIVE_ITEM_TAB0: [u8; 3] = [5, 1, 4];
const K_RECEIVE_ITEM_MSGS: [i16; 76] = [
    -1, 0x70, 0x77, 0x52, -1, 0x78, 0x78, 0x62, 0x61, 0x66, 0x69, 0x53, 0x52, 0x56, -1, 0x64, 0x63,
    0x65, 0x51, 0x54, 0x67, 0x68, 0x6b, 0x77, 0x79, 0x55, 0x6e, 0x58, 0x6d, 0x5d, 0x57, 0x5e, -1,
    0x74, 0x75, 0x76, -1, 0x5f, 0x158, -1, 0x6a, 0x5c, 0x8f, 0x71, 0x72, 0x73, 0x71, 0x72, 0x73,
    0x6a, 0x6c, 0x60, -1, -1, -1, 0x59, 0x84, 0x5a, -1, -1, -1, -1, -1, 0x159, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 0xdb, 0x67, 0x7c,
];
const K_RECEIVE_ITEM_MSGS2: [i16; 2] = [0x5b, 0x83];
const K_RECEIVE_ITEM_MSGS3: [i16; 4] = [-1, 0x155, 0x156, 0x157];
const K_BOMB_TAB0: [u8; 11] = [0xa0, 6, 4, 4, 4, 4, 4, 6, 6, 6, 6];
const K_BOMB_DRAW_TAB0: [u8; 12] = [0, 1, 2, 3, 2, 3, 4, 5, 6, 7, 8, 9];
const K_BOMB_DRAW_TAB2: [u8; 11] = [1, 4, 4, 4, 4, 4, 5, 4, 6, 6, 6];

const K_ANCILLA_PFLAGS: [u8; 68] = [
    0, 8, 0x0c, 0x10, 0x10, 4, 0x10, 0x18, 8, 8, 8, 0, 0x14, 0, 0x10, 0x28, 0x18, 0x10, 0x10, 0x10,
    0x10, 0x0c, 8, 8, 0x50, 0, 0x10, 8, 0x40, 0, 0x0c, 0x24, 0x10, 0x0c, 8, 0x10, 0x10, 4, 0x0c,
    0x1c, 0, 0x10, 0x14, 0x14, 0x10, 8, 0x20, 0x10, 0x10, 0x10, 4, 0, 0x80, 0x10, 4, 0x30, 0x14,
    0x10, 0, 0x10, 0, 0, 8, 0, 0x10, 8, 0x78, 0x80,
];

const K_RECEIVE_ITEM_GFX: [u8; 76] = [
    6, 0x18, 0x18, 0x18, 0x2d, 0x20, 0x2e, 9, 9, 0x0a, 8, 5, 0x10, 0x0b, 0x2c, 0x1b, 0x1a, 0x1c,
    0x14, 0x19, 0x0c, 7, 0x1d, 0x2f, 7, 0x15, 0x12, 0x0d, 0x0d, 0x0e, 0x11, 0x17, 0x28, 0x27, 4, 4,
    0x0f, 0x16, 3, 0x13, 1, 0x1e, 0x10, 0, 0, 0, 0, 0, 0, 0x30, 0x22, 0x21, 0x24, 0x24, 0x24, 0x23,
    0x23, 0x23, 0x29, 0x2a, 0x2c, 0x2b, 3, 3, 0x34, 0x35, 0x31, 0x33, 2, 0x32, 0x36, 0x37, 0x2c, 6,
    0x0c, 0x38,
];
const K_RECEIVE_ITEM_TAB1: [u8; 76] = [
    0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
];
const K_WISH_POND2_OAM_FLAGS: [u8; 76] = [
    5, 0xff, 5, 5, 5, 5, 5, 1, 2, 1, 1, 1, 2, 2, 2, 4, 4, 4, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1, 4, 4, 2,
    1, 6, 1, 2, 1, 2, 2, 1, 2, 2, 4, 1, 1, 4, 2, 1, 4, 2, 2, 4, 4, 4, 2, 1, 4, 1, 2, 2, 1, 2, 2, 1,
    1, 4, 4, 1, 2, 2, 4, 4, 4, 2, 5, 2, 1,
];
const K_TRAVEL_BIRD_DMA_STUFFS: [u8; 4] = [0, 0x20, 0x40, 0xe0];
const K_TRAVEL_BIRD_DRAW_X: [i8; 3] = [0, -9, -9];
const K_TRAVEL_BIRD_DRAW_Y: [i8; 3] = [0, 12, 20];
const K_TRAVEL_BIRD_DRAW_CHAR: [u8; 3] = [0x0e, 0, 2];
const K_TRAVEL_BIRD_DRAW_FLAGS: [u8; 3] = [0x22, 0x2e, 0x2e];

const K_OVERWORLD_OFFSET_BASE_X_ANCILLA: [u16; 64] = [
    0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00,
    0, 0x200, 0x400, 0x600, 0x800, 0xa00, 0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00,
    0xc00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0x200, 0x400, 0x600, 0x800, 0xa00,
    0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00,
    0xa00, 0xe00,
];

const K_OVERWORLD_OFFSET_BASE_Y_ANCILLA: [u16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x200, 0, 0, 0, 0, 0x200, 0x400, 0x400, 0x400, 0x400, 0x400,
    0x400, 0x400, 0x400, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600,
    0x800, 0x600, 0x600, 0x800, 0x600, 0x600, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00,
    0xa00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xe00, 0xe00,
    0xe00, 0xc00, 0xc00, 0xe00,
];

const K_MAGIC_POWDER_TAB0: [u8; 40] = [
    13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 10, 11, 12, 0, 1, 2, 3, 4, 5, 6, 16, 17, 18, 0, 1, 2, 3, 4, 5,
    6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6,
];

#[rustfmt::skip]
const K_ANCILLA_TILE_COLL_ATTRS: [u8; 256] = [
    0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4,
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 3, 3, 3,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

#[rustfmt::skip]
const K_ANCILLA_TILE_COLL0_ATTRS: [u8; 256] = [
    0, 1, 0, 3, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4,
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 3, 3, 3,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

const K_SLOPED_TILE: [u8; 32] = [
    7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0,
];

const K_FIRE_ROD_XVEL2: [i8; 12] = [0, 0, -40, 40, 0, 0, -48, 48, 0, 0, -64, 64];
const K_FIRE_ROD_YVEL2: [i8; 12] = [-40, 40, 0, 0, -48, 48, 0, 0, -64, 64, 0, 0];

struct CheckPlayerCollOut {
    r4: u16,
    r6: u16,
    r8: u16,
    r10: u16,
}

struct AncillaOamInfo {
    x: u8,
    y: u8,
    flags: u8,
}

impl ZeldaState {
    fn replay_ancilla_trace_enabled(&self) -> bool {
        if std::env::var_os("ZELDA3_REPLAY_ANCILLA_TRACE").is_none() {
            return false;
        }
        std::env::var("ZELDA3_REPLAY_ANCILLA_TRACE_FRAME")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .is_none_or(|target| self.state_recorder.replay_frame_counter == target)
    }

    pub(super) fn ancilla_main(&mut self) {
        self.ancilla_weapon_tink();
        self.ancilla_execute_all();
    }

    fn ancilla_weapon_tink(&mut self) {
        if self.ram[REPULSESPARK_TIMER_ANCILLA] == 0 {
            return;
        }
        self.ram[SPRITE_ALERT_FLAG] = 2;
        self.ram[REPULSESPARK_ANIM_DELAY_ANCILLA] =
            self.ram[REPULSESPARK_ANIM_DELAY_ANCILLA].wrapping_sub(1);
        if sign8(self.ram[REPULSESPARK_ANIM_DELAY_ANCILLA]) {
            self.ram[REPULSESPARK_TIMER_ANCILLA] =
                self.ram[REPULSESPARK_TIMER_ANCILLA].wrapping_sub(1);
            self.ram[REPULSESPARK_ANIM_DELAY_ANCILLA] = 1;
        }

        if self.ram[SORT_SPRITES_SETTING] != 0 {
            if self.ram[REPULSESPARK_FLOOR_STATUS_ANCILLA] != 0 {
                self.oam_allocate_from_region_f(0x10);
            } else {
                self.oam_allocate_from_region_d(0x10);
            }
        } else {
            self.oam_allocate_from_region_a(0x10);
        }

        let x = self.ram[REPULSESPARK_X_LO_ANCILLA].wrapping_sub(self.ram[BG2HOFS_COPY2]);
        let y = self.ram[REPULSESPARK_Y_LO_ANCILLA].wrapping_sub(self.ram[BG2VOFS_COPY2]);
        if x >= 0xf8 || y >= 0xf0 {
            self.ram[REPULSESPARK_TIMER_ANCILLA] = 0;
            return;
        }

        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let oam_idx = (oam - OAM_BUF) / 4;
        const REPULSE_SPARK_FLAGS: [u8; 4] = [0x22, 0x12, 0x22, 0x22];
        let flags = REPULSE_SPARK_FLAGS[self.ram[REPULSESPARK_FLOOR_STATUS_ANCILLA] as usize];
        if self.ram[REPULSESPARK_TIMER_ANCILLA] >= 3 {
            self.set_oam_plain(
                oam_idx,
                x,
                y,
                if self.ram[REPULSESPARK_TIMER_ANCILLA] < 9 {
                    0x92
                } else {
                    0x80
                },
                flags,
                0,
            );
            return;
        }

        const REPULSE_SPARK_CHAR: [u8; 3] = [0x93, 0x82, 0x81];
        let c = REPULSE_SPARK_CHAR[self.ram[REPULSESPARK_TIMER_ANCILLA] as usize];
        self.set_oam_plain(oam_idx, x.wrapping_sub(4), y.wrapping_sub(4), c, flags, 0);
        self.set_oam_plain(
            oam_idx + 1,
            x.wrapping_add(4),
            y.wrapping_sub(4),
            c,
            flags | 0x40,
            0,
        );
        self.set_oam_plain(
            oam_idx + 2,
            x.wrapping_sub(4),
            y.wrapping_add(4),
            c,
            flags | 0x80,
            0,
        );
        self.set_oam_plain(
            oam_idx + 3,
            x.wrapping_add(4),
            y.wrapping_add(4),
            c,
            flags | 0xc0,
            0,
        );
    }

    fn ancilla_empty(&mut self, _k: usize) {}

    fn ancilla_unused_14(&mut self, _k: usize) {
        // Ancilla_Unused_14 is an assert-only dispatch slot in the C port.
        panic!("Ancilla_Unused_14");
    }

    fn ancilla_unused_25(&mut self, _k: usize) {
        // Ancilla_Unused_25 is an assert-only dispatch slot in the C port.
        panic!("Ancilla_Unused_25");
    }

    fn ancilla_execute_all(&mut self) {
        for i in (0..10).rev() {
            self.ram[CUR_OBJECT_INDEX] = i as u8;
            let ty = self.ram[ANCILLA_TYPE + i];
            if ty != 0 {
                self.replay_trace_ram_watch(&format!(
                    "ancilla-before-execute-one ancilla={i} type=0x{:02x} timer=0x{:02x} item=0x{:02x} arr3=0x{:02x} floor=0x{:02x} num={}",
                    ty,
                    self.ram[ANCILLA_TIMER + i],
                    self.ram[ANCILLA_ITEM_TO_LINK + i],
                    self.ram[ANCILLA_ARR3 + i],
                    self.ram[ANCILLA_FLOOR + i],
                    self.ram[ANCILLA_NUMSPR + i],
                ));
                self.ancilla_execute_one(ty, i);
                self.replay_trace_ram_watch(&format!(
                    "ancilla-after-execute-one ancilla={i} type=0x{:02x} timer=0x{:02x} item=0x{:02x} arr3=0x{:02x} floor=0x{:02x} num={}",
                    self.ram[ANCILLA_TYPE + i],
                    self.ram[ANCILLA_TIMER + i],
                    self.ram[ANCILLA_ITEM_TO_LINK + i],
                    self.ram[ANCILLA_ARR3 + i],
                    self.ram[ANCILLA_FLOOR + i],
                    self.ram[ANCILLA_NUMSPR + i],
                ));
            }
        }
    }

    fn ancilla_execute_one(&mut self, ty: u8, k: usize) {
        if k < 6 {
            let idx =
                self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, self.ram[ANCILLA_NUMSPR + k]);
            self.ram[ANCILLA_OAM_IDX + k] = idx as u8;
        }

        if self.frame_control_view().submodule() == 0 && self.ram[ANCILLA_TIMER + k] != 0 {
            self.ram[ANCILLA_TIMER + k] = self.ram[ANCILLA_TIMER + k].wrapping_sub(1);
        }

        match ty {
            0x01 => self.ancilla01_somaria_bullet(k),
            0x02 => self.ancilla02_fire_rod_shot(k),
            0x03 => self.ancilla_empty(k),
            0x04 => self.ancilla04_beam_hit(k),
            0x05 => self.ancilla05_boomerang(k),
            0x06 => self.ancilla06_wall_hit(k),
            0x07 => self.ancilla07_bomb(k),
            0x08 => self.ancilla08_door_debris(k),
            0x09 => self.ancilla09_arrow(k),
            0x0a => self.ancilla0_a_arrow_in_the_wall(k),
            0x0b => self.ancilla0_b_ice_rod_shot(k),
            0x0c => self.ancilla_sword_beam(k),
            0x0d => self.ancilla0_d_spin_attack_full_charge_spark(k),
            0x0e..=0x10 => self.ancilla33_blast_wall_explosion(k),
            0x11 => self.ancilla11_ice_rod_wall_hit(k),
            0x12 => self.ancilla33_blast_wall_explosion(k),
            0x13 => self.ancilla13_ice_rod_sparkle(k),
            0x14 => self.ancilla_unused_14(k),
            0x15 => self.ancilla15_jump_splash(k),
            0x16 => self.ancilla16_hit_stars(k),
            0x17 => self.ancilla17_shovel_dirt(k),
            0x18 => self.ancilla18_ether_spell(k),
            0x19 => self.ancilla19_bombos_spell(k),
            0x1a => self.ancilla1_a_powder_dust(k),
            0x1b => self.ancilla_sword_wall_hit(k),
            0x1c => self.ancilla1_c_quake_spell(k),
            0x1d => self.ancilla1_d_screen_shake(k),
            0x1e => self.ancilla1_e_dash_dust(k),
            0x1f => self.ancilla1_f_hookshot(k),
            0x20 => self.ancilla20_blanket(k),
            0x21 => self.ancilla21_snore(k),
            0x22 => self.ancilla22_item_receipt(k),
            0x23 => self.ancilla23_link_poof(k),
            0x24 => self.ancilla24_gravestone(k),
            0x25 => self.ancilla_unused_25(k),
            0x26 => self.ancilla26_sword_swing_sparkle(k),
            0x27 => self.ancilla27_duck(k),
            0x28 => self.ancilla28_wish_pond_item(k),
            0x29 => self.ancilla29_milestone_item_receipt(k),
            0x2a => self.ancilla2_a_spin_attack_sparkle_a(k),
            0x2b => self.ancilla2_b_spin_attack_sparkle_b(k),
            0x2c => self.ancilla2_c_somaria_block(k),
            0x2d => self.ancilla2_d_somaria_block_fizz(k),
            0x2e => self.ancilla2_e_somaria_block_fission(k),
            0x2f => self.ancilla2_f_lamp_flame(k),
            0x30 => self.ancilla30_byrna_windup_spark(k),
            0x31 => self.ancilla31_byrna_spark(k),
            0x32 => self.ancilla32_blast_wall_fireball(k),
            0x33 => self.ancilla33_blast_wall_explosion(k),
            0x34 => self.ancilla34_skull_woods_fire(k),
            0x35 => self.ancilla35_master_sword_receipt(k),
            0x36 => self.ancilla36_flute(k),
            0x37 => self.ancilla37_weathervane_explosion(k),
            0x39 => self.ancilla39_somaria_platform_poof(k),
            0x38 => self.ancilla38_cutscene_duck(k),
            0x3a => self.ancilla3_a_big_bomb_explosion(k),
            0x3b => self.ancilla3_b_sword_up_sparkle(k),
            0x3c => self.ancilla3_c_spin_attack_charge_sparkle(k),
            0x3d => self.ancilla3_d_item_splash(k),
            0x3e => self.ancilla_rising_crystal(k),
            0x3f => self.ancilla3_f_bush_poof(k),
            0x40 => self.ancilla40_dwarf_poof(k),
            0x41 => self.ancilla41_waterfall_splash(k),
            0x42 => self.ancilla42_happiness_pond_rupees(k),
            0x43 => self.ancilla43_ganons_tower_cutscene(k),
            _ => {}
        }
    }

    pub(super) fn ancilla_add_blanket(&mut self, a: u8) {
        let k = 0;
        self.ram[ANCILLA_TYPE + k] = a;
        self.ram[ANCILLA_NUMSPR + k] = K_ANCILLA_PFLAGS[a as usize];
        self.ram[ANCILLA_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[ANCILLA_FLOOR2 + k] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
        self.ram[ANCILLA_OBJPRIO + k] = 0;
        self.ancilla_set_xy(k, 0x0938, 0x2162);
    }

    pub(super) fn ancilla_add_cape_poof(&mut self, ty: u8, limit: u8) {
        if let Some(k) = self.ancilla_add_simple(ty, limit) {
            self.ram[ANCILLA_STEP + k] = 1;
            self.ram[LINK_IS_TRANSFORMING] = 1;
            self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
            self.ram[LINK_DIRECTION] = 0;
            self.ram[LINK_DIRECTION_LAST] = 0;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_AUX_TIMER + k] = 7;
            let x = self.player_state_view().x();
            let y = self.player_state_view().y().wrapping_add(4);
            self.ancilla_set_xy(k, x, y);
        }
    }

    pub(super) fn ancilla_add_hit_stars(&mut self, a: u8, y: u8) {
        const SHOVEL_HIT_STARS_XY: [i8; 12] = [21, -11, 21, 11, 3, -6, 21, 5, 16, -14, 16, 14];
        const SHOVEL_HIT_STARS_X2: [i8; 6] = [-3, 19, 2, 13, -6, 22];

        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_AUX_TIMER + k] = 2;
            self.ram[ANCILLA_ARR3 + k] = 1;
            self.ram[ANCILLA_Y_VEL + k] = 0;
            self.ram[ANCILLA_X_VEL + k] = 0;

            let mut j = a;
            if self.ram[LINK_ITEM_IN_HAND] != 0 {
                j = (self.ram[LINK_DIRECTION_FACING] >> 1).wrapping_add(2);
            } else if self.ram[LINK_POSITION_MODE] != 0 {
                j = if self.ram[LINK_DIRECTION_FACING] != 4 {
                    1
                } else {
                    0
                };
            }

            self.ram[ANCILLA_STEP + k] = j;
            let j = j as usize;
            let link_x = self.player_state_view().x();
            let link_y = self.player_state_view().y();
            let t = link_x.wrapping_add(SHOVEL_HIT_STARS_X2[j] as i16 as u16);
            self.ram[ANCILLA_A + k] = t as u8;
            self.ram[ANCILLA_B + k] = (t >> 8) as u8;
            self.ancilla_set_xy(
                k,
                link_x.wrapping_add(SHOVEL_HIT_STARS_XY[j * 2 + 1] as i16 as u16),
                link_y.wrapping_add(SHOVEL_HIT_STARS_XY[j * 2] as i16 as u16),
            );
        }
    }

    pub(super) fn ancilla_add_fire_rod_shot(&mut self, type_: u8, _y: u8) {
        const FIRE_ROD_X: [i8; 4] = [0, 0, -8, 16];
        const FIRE_ROD_Y: [i8; 4] = [-8, 16, 3, 3];
        const FIRE_ROD_XVEL: [i8; 4] = [0, 0, -64, 64];
        const FIRE_ROD_YVEL: [i8; 4] = [-64, 64, 0, 0];

        let y = 1;
        let Some(mut j) = self.ancilla_alloc_init(type_, y) else {
            if type_ != 1 {
                self.refund_magic(0);
            }
            return;
        };

        if type_ != 1 {
            self.ancilla_sfx2_near(0x0e);
        }

        self.ram[ANCILLA_TYPE + j] = type_;
        self.ram[ANCILLA_NUMSPR + j] = K_ANCILLA_PFLAGS[type_ as usize];
        self.ram[ANCILLA_TIMER + j] = 3;
        self.ram[ANCILLA_STEP + j] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK + j] = 0;
        self.ram[ANCILLA_OBJPRIO + j] = 0;
        self.ram[ANCILLA_U + j] = 0;
        let mut i = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        self.ram[ANCILLA_DIR + j] = i as u8;

        if self.ancilla_check_initial_tile_a(j) < 0 {
            self.ancilla_set_xy(
                j,
                self.player_state_view()
                    .x()
                    .wrapping_add(FIRE_ROD_X[i] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(FIRE_ROD_Y[i] as i16 as u16),
            );
            if type_ != 1 {
                self.ram[ANCILLA_X_VEL + j] = FIRE_ROD_XVEL[i] as u8;
                self.ram[ANCILLA_Y_VEL + j] = FIRE_ROD_YVEL[i] as u8;
            } else {
                i += self.ram[LINK_SWORD_TYPE].wrapping_sub(2) as usize * 4;
                self.ram[ANCILLA_X_VEL + j] = K_FIRE_ROD_XVEL2[i] as u8;
                self.ram[ANCILLA_Y_VEL + j] = K_FIRE_ROD_YVEL2[i] as u8;
            }
            self.ram[ANCILLA_FLOOR + j] = self.ram[LINK_IS_ON_LOWER_LEVEL];
            self.ram[ANCILLA_FLOOR2 + j] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
        } else if type_ == 1 {
            self.ram[ANCILLA_TYPE + j] = 4;
            self.ram[ANCILLA_TIMER + j] = 7;
            self.ram[ANCILLA_NUMSPR + j] = 16;
        } else {
            self.ram[ANCILLA_STEP + j] = 1;
            self.ram[ANCILLA_TIMER + j] = 31;
            self.ram[ANCILLA_NUMSPR + j] = 8;
            j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
            self.ancilla_sfx2_pan(j, 0x2a);
        }
    }

    pub(super) fn ancilla_add_falling_prize(&mut self, a: u8, item_idx: u8, yv: u8) -> i32 {
        const FALLING_ITEM_TYPE: [u8; 7] = [0x10, 0x37, 0x39, 0x38, 0x26, 0x0f, 0x20];
        const FALLING_ITEM_G: [u8; 7] = [0x40, 0, 0, 0, 0, 0xff, 0];
        const FALLING_ITEM_X: [u16; 7] = [0x78, 0x78, 0x78, 0x78, 0x78, 0x80, 0x78];
        const FALLING_ITEM_Y: [u16; 7] = [0x48, 0x78, 0x78, 0x78, 0x78, 0x68, 0x78];
        const FALLING_ITEM_Z: [u8; 7] = [0x60, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];

        self.ram[LINK_RECEIVEITEM_INDEX] = item_idx;
        let Some(k) = self.ancilla_add_simple(a, yv) else {
            return -1;
        };
        let item_type = FALLING_ITEM_TYPE[item_idx as usize];
        self.ram[ANCILLA_ITEM_TO_LINK + k] = item_type;
        if item_type == 0x10 || item_type == 0x0f {
            self.DecodeAnimatedSpriteTile_variable(K_RECEIVE_ITEM_GFX[item_type as usize]);
        }

        self.ram[ANCILLA_Z_VEL + k] = (-48i8) as u8;
        self.ram[ANCILLA_Y_VEL + k] = 0;
        self.ram[ANCILLA_X_VEL + k] = 0;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_Z + k] = FALLING_ITEM_Z[item_idx as usize];
        self.ram[ANCILLA_AUX_TIMER + k] = 9;
        self.ram[ANCILLA_ARR3 + k] = 0;
        self.ram[ANCILLA_L + k] = 0;
        self.ram[ANCILLA_G + k] = FALLING_ITEM_G[item_idx as usize];
        self.ram[LINK_RECEIVEITEM_INDEX] = item_type;

        let (x, y) = if item_idx != 0 && item_idx != 5 {
            if self.ram[CUR_PALACE_INDEX_X2] == 20 {
                (
                    (self.player_state_view().x() & 0xff00) | 0x0100,
                    (self.player_state_view().y() & 0xff00) | 0x0100,
                )
            } else {
                (
                    FALLING_ITEM_X[item_idx as usize]
                        .wrapping_add(read_le_u16(&self.ram, BG2HOFS_COPY2)),
                    FALLING_ITEM_Y[item_idx as usize]
                        .wrapping_add(read_le_u16(&self.ram, BG2VOFS_COPY2)),
                )
            }
        } else {
            (
                self.player_state_view().x(),
                FALLING_ITEM_Y[item_idx as usize]
                    .wrapping_add(read_le_u16(&self.ram, BG2VOFS_COPY2)),
            )
        };
        self.ancilla_set_xy(k, x, y);
        k as i32
    }

    pub(super) fn add_sword_beam(&mut self, y: u8) {
        const SWORD_BEAM_X: [i8; 4] = [-8, -10, -22, 4];
        const SWORD_BEAM_Y: [i8; 4] = [-24, 8, -6, -6];
        const SWORD_BEAM_S: [i8; 4] = [-8, -8, -8, 8];
        const SWORD_BEAM_TAB: [u8; 16] = [
            0x21, 0x1d, 0x19, 0x15, 3, 0x3e, 0x3a, 0x36, 0x12, 0x0e, 0x0a, 6, 0x31, 0x2d, 0x29,
            0x25,
        ];
        const SWORD_BEAM_YVEL: [i8; 4] = [-64, 64, 0, 0];
        const SWORD_BEAM_XVEL: [i8; 4] = [0, 0, -64, 64];

        let Some(k) = self.ancilla_add_simple(0x0c, y) else {
            return;
        };
        let mut j = self.ram[LINK_DIRECTION_FACING] as usize * 2;
        self.ram[SWORDBEAM_ARR] = SWORD_BEAM_TAB[j];
        self.ram[SWORDBEAM_ARR + 1] = SWORD_BEAM_TAB[j + 1];
        self.ram[SWORDBEAM_ARR + 2] = SWORD_BEAM_TAB[j + 2];
        self.ram[SWORDBEAM_ARR + 3] = SWORD_BEAM_TAB[j + 3];
        self.ram[SWORDBEAM_VAR1] = SWORD_BEAM_TAB[j + 3];
        self.ram[ANCILLA_AUX_TIMER + k] = 2;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0x4c;
        self.ram[ANCILLA_ARR3 + k] = 8;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_L + k] = 0;
        self.ram[ANCILLA_G + k] = 0;
        self.ram[ANCILLA_ARR1 + k] = 0;
        self.ram[SWORDBEAM_VAR2] = 14;
        j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        self.ram[ANCILLA_DIR + k] = j as u8;
        self.ram[ANCILLA_Y_VEL + k] = SWORD_BEAM_YVEL[j] as u8;
        self.ram[ANCILLA_X_VEL + k] = SWORD_BEAM_XVEL[j] as u8;
        self.ram[ANCILLA_S_PLAYER + k] = SWORD_BEAM_S[j] as u8;

        let swordbeam_temp_y = self.player_state_view().y().wrapping_add(12);
        let swordbeam_temp_x = self.player_state_view().x().wrapping_add(8);
        write_le_u16(&mut self.ram, SWORDBEAM_TEMP_Y, swordbeam_temp_y);
        write_le_u16(&mut self.ram, SWORDBEAM_TEMP_X, swordbeam_temp_x);

        if self.ancilla_check_initial_tile_a(k) >= 0 {
            self.ancilla_set_xy(
                k,
                swordbeam_temp_x.wrapping_add(SWORD_BEAM_X[j] as i16 as u16),
                swordbeam_temp_y.wrapping_add(SWORD_BEAM_Y[j] as i16 as u16),
            );
            self.ram[SOUND_EFFECT_2] = 1 | self.ancilla_calculate_sfx_pan(k);
            self.ram[ANCILLA_TYPE + k] = 4;
            self.ram[ANCILLA_TIMER + k] = 7;
            self.ram[ANCILLA_NUMSPR + k] = 16;
        }
    }

    pub(super) fn ancilla_spawn_sword_charge_sparkle(&mut self) {
        const SWORD_CHARGE_SPARKLE_A: [u8; 4] = [0, 0, 7, 7];
        const SWORD_CHARGE_SPARKLE_B: [u8; 4] = [0x70, 0x70, 0, 0];
        const SWORD_CHARGE_SPARKLE_X: [u8; 4] = [0, 3, 4, 5];
        const SWORD_CHARGE_SPARKLE_Y: [u8; 4] = [5, 12, 8, 8];

        let Some(k) = self.ancilla_alloc_high() else {
            return;
        };
        self.ram[ANCILLA_TYPE + k] = 0x3c;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_TIMER + k] = 4;
        self.ram[ANCILLA_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        let mut x = 0i8;
        let mut y = 0i8;
        let m0 = SWORD_CHARGE_SPARKLE_A[j];
        if m0 == 0 {
            y = (self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] >> 2) as i8;
            if j == 0 {
                y = -y;
            }
        }
        let m1 = SWORD_CHARGE_SPARKLE_B[j];
        if m1 == 0 {
            x = (self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] >> 2) as i8;
            if j == 2 {
                x = -x;
            }
        }
        let r = self.get_random_number();
        let dst_x = self
            .player_state_view()
            .x()
            .wrapping_add(x as i16 as u16)
            .wrapping_add(SWORD_CHARGE_SPARKLE_X[j] as u16)
            .wrapping_add(((r & m1) >> 4) as u16);
        let dst_y = self
            .player_state_view()
            .y()
            .wrapping_add(y as i16 as u16)
            .wrapping_add(SWORD_CHARGE_SPARKLE_Y[j] as u16)
            .wrapping_add((r & m0) as u16);
        if self.replay_ancilla_trace_enabled() {
            println!(
                "ancilla-trace kind=spawn-charge abs={} fc=0x{:02x} dst={} rng=0x{:02x} j={} off=0x{:02x}/0x{:02x} mask=0x{:02x}/0x{:02x} xy=0x{:04x}/0x{:04x} link=0x{:04x}/0x{:04x} face=0x{:02x} spin=0x{:02x} speed=0x{:02x}/0x{:02x} type=0x{:02x} timer=0x{:02x} floor=0x{:02x}",
                self.state_recorder.replay_frame_counter,
                self.ram[FRAME_COUNTER],
                k,
                r,
                j,
                x as u8,
                y as u8,
                m1,
                m0,
                dst_x,
                dst_y,
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.ram[LINK_DIRECTION_FACING],
                self.ram[LINK_SPIN_ATTACK_STEP_COUNTER],
                self.ram[LINK_ACTUAL_VEL_X],
                self.ram[LINK_ACTUAL_VEL_Y],
                self.ram[ANCILLA_TYPE + k],
                self.ram[ANCILLA_TIMER + k],
                self.ram[ANCILLA_FLOOR + k],
            );
        }
        self.ancilla_set_xy(k, dst_x, dst_y);
    }

    pub(super) fn ancilla_add_sword_charge_sparkle_from_ancilla(&mut self, source: usize) {
        let Some(k) = self.ancilla_alloc_high() else {
            return;
        };
        self.ram[ANCILLA_TYPE + k] = 60;
        self.ram[ANCILLA_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_TIMER + k] = 4;

        let rand = self.get_random_number();
        let mut z = self.ram[ANCILLA_Z + source];
        if z >= 0xf8 {
            z = 0;
        }
        self.ancilla_set_xy(
            k,
            self.ancilla_get_x(source)
                .wrapping_add(2)
                .wrapping_add(u16::from(rand >> 5)),
            self.ancilla_get_y(source)
                .wrapping_sub(2)
                .wrapping_sub(u16::from(z))
                .wrapping_add(u16::from(rand & 0x0f)),
        );
    }

    fn add_dashing_dust_ex(&mut self, a: u8, y: u8, flag: u8) {
        const ADD_DASHING_DUST_X: [i8; 4] = [4, 4, 6, 0];
        const ADD_DASHING_DUST_Y: [i8; 4] = [20, 4, 16, 16];
        if let Some(k) = self.ancilla_add_simple(a, y) {
            self.ram[ANCILLA_STEP + k] = flag;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_TIMER + k] = 3;
            let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
            self.ram[ANCILLA_DIR + k] = j as u8;
            if flag == 0 {
                self.ancilla_set_xy(
                    k,
                    self.player_state_view().x(),
                    self.player_state_view().y().wrapping_add(20),
                );
            } else {
                self.ancilla_set_xy(
                    k,
                    self.player_state_view()
                        .x()
                        .wrapping_add(ADD_DASHING_DUST_X[j] as i16 as u16),
                    self.player_state_view()
                        .y()
                        .wrapping_add(ADD_DASHING_DUST_Y[j] as i16 as u16),
                );
            }
        }
    }

    pub(super) fn ancilla_add_dash_dust(&mut self, a: u8, y: u8) {
        self.add_dashing_dust_ex(a, y, 1);
    }

    pub(super) fn ancilla_add_dash_dust_charging(&mut self, a: u8, y: u8) {
        self.add_dashing_dust_ex(a, y, 0);
    }

    fn ancilla_add_blast_wall_fireball(&mut self, _a: u8, _y: u8, r4: usize) {
        const BLAST_WALL_XY: [i8; 32] = [
            -64, 0, -22, 42, -38, 38, -42, 22, 0, 64, 22, 42, 38, 38, 42, 22, 64, 0, 22, -42, 38,
            -38, 42, -22, 0, -64, -22, -42, -38, -38, -42, -22,
        ];

        for k in (5..=10).rev() {
            if self.ram[ANCILLA_TYPE + k] == 0 {
                self.ram[ANCILLA_TYPE + k] = 0x32;
                self.ram[ANCILLA_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
                self.ram[BLASTWALL_VAR12 + k] = 16;
                let j = (self.ram[FRAME_COUNTER] & 15) as usize;
                self.ram[ANCILLA_Y_VEL + k] = BLAST_WALL_XY[j * 2] as u8;
                self.ram[ANCILLA_X_VEL + k] = BLAST_WALL_XY[j * 2 + 1] as u8;
                self.ancilla_set_xy(
                    k,
                    read_le_u16(&self.ram, BLASTWALL_VAR11 + r4 * 2).wrapping_add(16),
                    read_le_u16(&self.ram, BLASTWALL_VAR10 + r4 * 2).wrapping_add(8),
                );
                return;
            }
        }
    }

    pub(super) fn ancilla_add_arrow(
        &mut self,
        a: u8,
        ax: u8,
        ay: u8,
        xcoord: u16,
        ycoord: u16,
    ) -> i32 {
        const SHOOT_BOW_X: [i8; 4] = [4, 4, 0, 4];
        const SHOOT_BOW_Y: [i8; 4] = [-4, 3, 4, 4];
        const SHOOT_BOW_XVEL: [i8; 4] = [0, 0, -48, 48];
        const SHOOT_BOW_YVEL: [i8; 4] = [-48, 48, 0, 0];

        write_le_u16(&mut self.ram, SCRATCH_0_ANCILLA, ycoord);
        write_le_u16(&mut self.ram, SCRATCH_1_ANCILLA, xcoord);
        self.ram[INDEX_OF_INTERACTING_TILE_ANCILLA] = ax;

        if self.ancilla_add_check_for_presence(a) {
            return -1;
        }

        let k = self.ancilla_add_arrow_find_slot(a, ay);

        if k >= 0 {
            let k = k as usize;
            self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 7;
            self.ram[ANCILLA_H + k] = 0;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 8;
            let j = (ax >> 1) as usize;
            self.ram[ANCILLA_DIR + k] = j as u8 | 4;
            self.ram[ANCILLA_Y_VEL + k] = SHOOT_BOW_YVEL[j] as u8;
            self.ram[ANCILLA_X_VEL + k] = SHOOT_BOW_XVEL[j] as u8;
            self.ancilla_set_xy(
                k,
                xcoord.wrapping_add(SHOOT_BOW_X[j] as i16 as u16),
                ycoord
                    .wrapping_add(8)
                    .wrapping_add(SHOOT_BOW_Y[j] as i16 as u16),
            );
        }
        k
    }

    fn ancilla_add_arrow_find_slot(&mut self, type_: u8, ay: u8) -> i32 {
        let mut n = 0;
        for k in (0..=4).rev() {
            if self.ram[ANCILLA_TYPE + k] == 10 {
                n += 1;
            }
        }

        let mut k = -1;
        if n != ay.wrapping_add(1) {
            for i in (0..=4).rev() {
                if self.ram[ANCILLA_TYPE + i] == 0 {
                    k = i as i32;
                    break;
                }
            }
        } else {
            loop {
                self.ram[ANCILLA_ALLOC_ROTATE_PLAYER] =
                    self.ram[ANCILLA_ALLOC_ROTATE_PLAYER].wrapping_sub(1);
                if sign8(self.ram[ANCILLA_ALLOC_ROTATE_PLAYER]) {
                    self.ram[ANCILLA_ALLOC_ROTATE_PLAYER] = 4;
                }
                k = self.ram[ANCILLA_ALLOC_ROTATE_PLAYER] as i32;
                if self.ram[ANCILLA_TYPE + k as usize] == 10 {
                    break;
                }
            }
        }

        if k >= 0 {
            let k = k as usize;
            self.ram[ANCILLA_TYPE + k] = type_;
            self.ram[ANCILLA_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
            self.ram[ANCILLA_FLOOR2 + k] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
            self.ram[ANCILLA_Y_VEL + k] = 0;
            self.ram[ANCILLA_X_VEL + k] = 0;
            self.ram[ANCILLA_OBJPRIO + k] = 0;
            self.ram[ANCILLA_U + k] = 0;
            self.ram[ANCILLA_NUMSPR + k] = K_ANCILLA_PFLAGS[type_ as usize];
        }
        k
    }

    fn add_bird_common(&mut self, k: usize) {
        self.ram[ANCILLA_Y_VEL + k] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_AUX_TIMER + k] = 1;
        self.ram[ANCILLA_X_VEL + k] = 56;
        self.ram[ANCILLA_ARR3 + k] = 3;
        self.ram[ANCILLA_K + k] = 0;
        self.ram[ANCILLA_G + k] = 0;

        let xt: u16 = if self.read_u32_ram(ENHANCED_FEATURES0) & 1 != 0 {
            0x40
        } else {
            0
        };
        self.ancilla_set_xy(
            k,
            read_le_u16(&self.ram, BG2HOFS_COPY2)
                .wrapping_sub(16)
                .wrapping_sub(xt),
            self.player_state_view().y().wrapping_sub(8),
        );
    }

    fn bomb_project_speed_towards_player(
        &mut self,
        _k: usize,
        x: u16,
        y: u16,
        vel: u8,
    ) -> ProjectSpeedRet {
        let old_x = self.sprite_get_x(0);
        let old_y = self.sprite_get_y(0);
        let old_z = self.ram[SPRITE_Z];
        self.sprite_set_x(0, x);
        self.sprite_set_y(0, y);
        self.ram[SPRITE_Z] = 0;
        let pt = self.sprite_project_speed_towards_link(0, vel);
        self.ram[SPRITE_Z] = old_z;
        self.sprite_set_x(0, old_x);
        self.sprite_set_y(0, old_y);
        pt
    }

    fn bomb_check_sprite_damage(&mut self, k: usize) {
        for j in (0..16).rev() {
            if (((j as u8 ^ self.ram[FRAME_COUNTER]) & 3)
                | self.ram[SPRITE_HIT_TIMER_ANCILLA + j]
                | self.ram[SPRITE_IGNORE_PROJECTILE_ANCILLA + j])
                != 0
            {
                continue;
            }
            if self.ram[SPRITE_FLOOR + j] != self.ram[ANCILLA_FLOOR + k]
                || self.ram[SPRITE_STATE + j] < 9
            {
                continue;
            }
            let ax = self.ancilla_get_x(k).wrapping_sub(24);
            let ay = self
                .ancilla_get_y(k)
                .wrapping_sub(24)
                .wrapping_sub(self.ram[ANCILLA_Z + k] as u16);
            let mut hb = SpriteHitBox {
                r0_xlo: ax as u8,
                r8_xhi: (ax >> 8) as u8,
                r1_ylo: ay as u8,
                r9_yhi: (ay >> 8) as u8,
                r2: 48,
                r3: 48,
                r4_spr_xlo: 0,
                r10_spr_xhi: 0,
                r5_spr_ylo: 0,
                r11_spr_yhi: 0,
                r6_spr_xsize: 0,
                r7_spr_ysize: 0,
            };
            self.sprite_setup_hit_box(j, &mut hb);
            if !self.check_if_hit_boxes_overlap(&hb) {
                continue;
            }
            if self.ram[SPRITE_TYPE + j] == 0x92 && self.ram[SPRITE_C_ANCILLA + j] >= 3 {
                continue;
            }
            self.ancilla_check_damage_to_sprite(j, self.ram[ANCILLA_TYPE + k]);
            let pt = self.ancilla_project_reflexive_speed_onto_sprite(
                j,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                64,
            );
            self.ram[SPRITE_X_RECOIL + j] = 0u8.wrapping_sub(pt.x);
            self.ram[SPRITE_Y_RECOIL_ANCILLA + j] = 0u8.wrapping_sub(pt.y);
        }
    }

    fn bomb_check_sprite_and_player_damage(&mut self, k: usize) {
        const BOMB_DMG_SPEED: [u8; 16] = [
            32, 32, 32, 32, 32, 32, 28, 28, 28, 28, 28, 28, 24, 24, 24, 24,
        ];
        const BOMB_DMG_ZVEL: [u8; 16] = [16, 16, 16, 16, 16, 16, 12, 12, 12, 12, 8, 8, 8, 8, 8, 8];
        const BOMB_DMG_DELAY: [u8; 16] = [
            32, 32, 32, 32, 32, 32, 24, 24, 24, 24, 24, 24, 16, 16, 16, 16,
        ];
        const BOMB_DMG_TO_LINK: [u8; 3] = [8, 4, 2];

        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0 || self.ram[ANCILLA_ITEM_TO_LINK + k] >= 9 {
            return;
        }
        self.bomb_check_sprite_damage(k);
        if self.ram[LINK_DISABLE_SPRITE_DAMAGE] != 0 {
            if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize
                && self.ram[LINK_STATE_BITS] & 0x80 != 0
            {
                self.ram[LINK_STATE_BITS] &= !0x80;
                self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
            }
            return;
        }

        if self.ram[LINK_AUXILIARY_STATE] != 0
            || self.ram[LINK_INCAPACITATED_TIMER] != 0
            || self.ram[ANCILLA_FLOOR + k] != self.ram[LINK_IS_ON_LOWER_LEVEL]
        {
            return;
        }

        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        let ax = self.ancilla_get_x(k).wrapping_sub(16);
        let ay = self.ancilla_get_y(k).wrapping_sub(16);
        let hb = SpriteHitBox {
            r0_xlo: link_x as u8,
            r8_xhi: (link_x >> 8) as u8,
            r1_ylo: link_y as u8,
            r9_yhi: (link_y >> 8) as u8,
            r2: 0x10,
            r3: 0x18,
            r4_spr_xlo: ax as u8,
            r10_spr_xhi: (ax >> 8) as u8,
            r5_spr_ylo: ay as u8,
            r11_spr_yhi: (ay >> 8) as u8,
            r6_spr_xsize: 32,
            r7_spr_ysize: 32,
        };

        if !self.check_if_hit_boxes_overlap(&hb) {
            return;
        }

        let x = self.ancilla_get_x(k).wrapping_sub(8);
        let y = self.ancilla_get_y(k).wrapping_sub(12);
        let j = self.bomb_get_displacement_from_link(k) as usize;
        let pt = self.bomb_project_speed_towards_player(k, x, y, BOMB_DMG_SPEED[j]);
        if self.ram[COUNTDOWN_FOR_BLINK] != 0 || self.ram[FLAG_BLOCK_LINK_MENU] == 2 {
            return;
        }
        self.ram[LINK_ACTUAL_VEL_X] = pt.x;
        self.ram[LINK_ACTUAL_VEL_Y] = pt.y;
        self.ram[LINK_ACTUAL_VEL_Z] = BOMB_DMG_ZVEL[j];
        self.ram[LINK_ACTUAL_VEL_Z_COPY] = BOMB_DMG_ZVEL[j];
        self.ram[LINK_INCAPACITATED_TIMER] = BOMB_DMG_DELAY[j];
        self.ram[LINK_AUXILIARY_STATE] = 1;
        self.ram[COUNTDOWN_FOR_BLINK] = 58;
        if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x8000 == 0 {
            self.ram[LINK_GIVE_DAMAGE] = BOMB_DMG_TO_LINK[self.ram[LINK_ARMOR] as usize];
        }
    }

    fn ancilla07_bomb(&mut self, k: usize) {
        if self.frame_control_view().submodule() != 0 {
            if self.frame_control_view().submodule() == 8
                || self.frame_control_view().submodule() == 16
            {
                self.ancilla_handle_lift_logic(k);
            } else if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize
                && self.ram[ANCILLA_K + k] != 0
            {
                if self.ram[ANCILLA_K + k] != 3 {
                    self.ancilla_latch_link_coordinates(k, 3);
                    self.ancilla_latch_altitude_above_link(k);
                    self.ram[ANCILLA_K + k] = 3;
                }
                self.ancilla_latch_carried_position(k);
            }
            self.bomb_draw(k);
            return;
        }
        self.ancilla_handle_lift_logic(k);

        let mut old_y = self.ancilla_latch_y_coord_to_z(k);
        let s1a = self.ram[ANCILLA_DIR + k];
        let s1b = self.ram[ANCILLA_OBJPRIO + k];
        self.ram[ANCILLA_OBJPRIO + k] = 0;
        let mut flag = self.ancilla_check_tile_collision_class2(k);

        if self.ram[PLAYER_IS_INDOORS] != 0
            && self.ram[ANCILLA_L + k] != 0
            && self.ram[ANCILLA_TILE_ATTR_PLAYER + k] == 0x1c
        {
            self.ram[ANCILLA_T_PLAYER + k] = 1;
        }

        loop {
            if flag
                && (self.ram[LINK_STATE_BITS] & 0x80 == 0
                    || self.ram[LINK_PICKING_THROW_STATE] != 0)
            {
                if s1b == 0 && self.ram[ANCILLA_ARR4 + k] == 0 {
                    self.ram[ANCILLA_ARR4 + k] = 1;
                    let qq = if self.ram[ANCILLA_DIR + k] == 1 {
                        16
                    } else {
                        4
                    };
                    if self.ram[ANCILLA_Y_VEL + k] != 0 {
                        self.ram[ANCILLA_Y_VEL + k] = if sign8(self.ram[ANCILLA_Y_VEL + k]) {
                            qq
                        } else {
                            (-(qq as i8)) as u8
                        };
                    }
                    if self.ram[ANCILLA_X_VEL + k] != 0 {
                        self.ram[ANCILLA_X_VEL + k] = if sign8(self.ram[ANCILLA_X_VEL + k]) {
                            4
                        } else {
                            (-4i8) as u8
                        };
                    }
                    if self.ram[ANCILLA_DIR + k] == 1 && self.ram[ANCILLA_Z + k] != 0 {
                        self.ram[ANCILLA_Y_VEL + k] = (-4i8) as u8;
                        self.ram[ANCILLA_L + k] = 2;
                    }
                }
            } else if !(k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize
                && self.ram[LINK_STATE_BITS] & 0x80 != 0)
                && (self.ram[ANCILLA_Z + k] == 0 || self.ram[ANCILLA_Z + k] == 0xff)
            {
                self.ram[ANCILLA_DIR + k] = 16;
                let bak0 = self.ram[ANCILLA_OBJPRIO + k];
                self.ancilla_check_tile_collision(k);
                self.ram[ANCILLA_OBJPRIO + k] = bak0;
                let a = self.ram[ANCILLA_TILE_ATTR_PLAYER + k];
                if a == 0x26 {
                    flag = true;
                    continue;
                } else if a == 0x0c || a == 0x1c {
                    if self.ram[DUNG_HDR_COLLISION] != 3 {
                        if self.ram[ANCILLA_FLOOR + k] == 0
                            && self.ram[ANCILLA_Z + k] != 0
                            && self.ram[ANCILLA_Z + k] != 0xff
                        {
                            self.ram[ANCILLA_FLOOR + k] = 1;
                        }
                    } else {
                        old_y = self
                            .ancilla_get_y(k)
                            .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_Y_VEL));
                        self.ancilla_set_x(
                            k,
                            self.ancilla_get_x(k)
                                .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_X_VEL)),
                        );
                    }
                } else if a == 0x20 || (a & 0xf0) == 0xb0 && a != 0xb6 && a != 0xbc {
                    if self.ram[LINK_STATE_BITS] & 0x80 == 0 {
                        if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
                            self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                        }
                        if self.ram[ANCILLA_TIMER + k] == 0 {
                            self.ram[ANCILLA_TYPE + k] = 0;
                            return;
                        }
                    }
                } else if a == 8 {
                    if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
                        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                    }
                    if self.ram[ANCILLA_TIMER + k] == 0 {
                        self.ancilla_set_y(k, self.ancilla_get_y(k).wrapping_sub(24));
                        self.ancilla_transmute_to_splash(k);
                        return;
                    }
                } else if matches!(a, 0x68 | 0x69 | 0x6a | 0x6b) {
                    self.ancilla_apply_conveyor(k);
                    old_y = self.ancilla_get_y(k);
                } else {
                    self.ram[ANCILLA_TIMER + k] = if self.ram[ANCILLA_L + k] != 0 { 0 } else { 2 };
                }
            }
            break;
        }

        self.ancilla_set_y(k, old_y);
        self.ram[ANCILLA_DIR + k] = s1a;
        self.ram[ANCILLA_OBJPRIO + k] |= s1b;
        self.bomb_check_sprite_and_player_damage(k);
        self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
        if self.ram[ANCILLA_ARR3 + k] == 0 {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 1 {
                self.ancilla_sfx2_pan(k, 0x0c);
                if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
                    self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                    if self.ram[LINK_STATE_BITS] & 0x80 != 0 {
                        self.ram[LINK_STATE_BITS] = 0;
                        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
                    }
                }
            }

            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 11 {
                self.ram[ANCILLA_TYPE + k] = if self.ram[ANCILLA_STEP + k] != 0 {
                    8
                } else {
                    0
                };
                return;
            }
            self.ram[ANCILLA_ARR3 + k] = K_BOMB_TAB0[self.ram[ANCILLA_ITEM_TO_LINK + k] as usize];
        }

        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 7 && self.ram[ANCILLA_ARR3 + k] == 2 {
            write_le_u16(&mut self.ram, DOOR_DEBRIS_X + k * 2, 0);
            self.bomb_check_for_destructibles(
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                k as u8,
            );
            if read_le_u16(&self.ram, DOOR_DEBRIS_X + k * 2) != 0 {
                self.ram[ANCILLA_STEP + k] = 1;
            }
        }
        self.bomb_draw(k);
    }

    fn boomerang_cheat_when_no_ones_looking(&self, k: usize, pt: &mut ProjectSpeedRet) {
        let x = self
            .player_state_view()
            .x()
            .wrapping_sub(self.ancilla_get_x(k))
            .wrapping_add(0xf0);
        let y = self
            .player_state_view()
            .y()
            .wrapping_sub(self.ancilla_get_y(k))
            .wrapping_add(0xf0);
        if x >= 0x1e0 {
            pt.x = if sign16(x.wrapping_sub(0x1e0)) {
                0x90
            } else {
                0x70
            };
        } else if y >= 0x1e0 {
            pt.y = if sign16(y.wrapping_sub(0x1e0)) {
                0x90
            } else {
                0x70
            };
        }
    }

    fn boomerang_screen_edge(&self, k: usize) -> bool {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        if self.ram[HOOKSHOT_EFFECT_INDEX] & 3 != 0 {
            let t = x
                .wrapping_add(if self.ram[HOOKSHOT_EFFECT_INDEX] & 1 != 0 {
                    16
                } else {
                    0
                })
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
            if t >= 0x100 {
                return true;
            }
        }
        if self.ram[HOOKSHOT_EFFECT_INDEX] & 12 != 0 {
            let t = y
                .wrapping_add(if self.ram[HOOKSHOT_EFFECT_INDEX] & 4 != 0 {
                    16
                } else {
                    0
                })
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
            if t >= 0xe2 {
                return true;
            }
        }
        false
    }

    fn boomerang_stop_off_screen(&mut self, k: usize) {
        let x = self.ancilla_get_x(k).wrapping_add(8);
        let y = self.ancilla_get_y(k).wrapping_add(8);
        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        if x >= link_x && x < link_x.wrapping_add(16) && y >= link_y && y < link_y.wrapping_add(24)
        {
            self.boomerang_terminate(k);
        }
    }

    fn boomerang_terminate(&mut self, k: usize) {
        self.ram[ANCILLA_TYPE + k] = 0;
        self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 0;
        if self.ram[LINK_ITEM_IN_HAND] & 0x80 != 0 {
            self.ram[LINK_ITEM_IN_HAND] = 0;
            self.ram[BUTTON_MASK_B_Y] &= !0x40;
            if self.ram[BUTTON_MASK_B_Y] & 0x80 == 0 {
                self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            }
        }
    }

    fn ancilla05_boomerang(&mut self, k: usize) {
        const BOOMERANG_X0: [i8; 8] = [0, 0, -8, 8, 8, 8, -8, -8];
        const BOOMERANG_Y0: [i8; 8] = [-16, 6, 0, 0, -8, 8, -8, 8];
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        for j in (0..=4).rev() {
            if self.ram[ANCILLA_TYPE + j] == 0x22 {
                self.boomerang_draw(k);
                return;
            }
        }
        if self.frame_control_view().submodule() != 0 {
            self.boomerang_draw(k);
            return;
        }

        if self.ram[FRAME_COUNTER] & 7 == 0 {
            self.ancilla_sfx2_pan(k, 9);
        }

        if self.ram[ANCILLA_AUX_TIMER + k] == 0 {
            if self.ram[BUTTON_B_FRAMES] < 9 && self.ram[PLAYER_HANDLER_TIMER] == 0 {
                if self.ram[LINK_IS_BUNNY_MIRROR] != 0
                    || self.ram[LINK_AUXILIARY_STATE] != 0
                    || self.ram[LINK_ITEM_IN_HAND] == 0
                        && self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0
                {
                    self.boomerang_terminate(k);
                    return;
                }
                self.boomerang_draw(k);
                return;
            }
            let j = (self.ram[ANCILLA_ARR23 + k] >> 1) as usize;
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(BOOMERANG_X0[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(8)
                    .wrapping_add(BOOMERANG_Y0[j] as i16 as u16),
            );
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_add(1);
        }

        if self.ram[ANCILLA_G + k] != 0 && self.ram[FRAME_COUNTER] & 1 == 0 {
            self.ancilla_add_sword_charge_sparkle(k);
        }

        if self.ram[ANCILLA_ITEM_TO_LINK + k] != 0 {
            if self.ram[ANCILLA_K + k] != 0 {
                self.ram[ANCILLA_K + k] = self.ram[ANCILLA_K + k].wrapping_add(1);
            }
            let link_y = self.player_state_view().y();
            write_le_u16(&mut self.ram, ANCILLA_A + k, link_y);
            self.player_state_view_mut().set_y(link_y.wrapping_add(8));
            let mut pt = self.ancilla_project_speed_towards_player(k, self.ram[ANCILLA_H + k]);
            self.boomerang_cheat_when_no_ones_looking(k, &mut pt);
            self.ram[ANCILLA_X_VEL + k] = pt.x;
            self.ram[ANCILLA_Y_VEL + k] = pt.y;
            copy_le_u16(&mut self.ram, LINK_Y_COORD, ANCILLA_A + k);
        }

        if self.ram[ANCILLA_Y_VEL + k] != 0 {
            self.ram[ANCILLA_Y_VEL + k] =
                self.ram[ANCILLA_Y_VEL + k].wrapping_add(self.ram[ANCILLA_K + k]);
        }
        self.ancilla_move_y(k);

        if self.ram[ANCILLA_X_VEL + k] != 0 {
            self.ram[ANCILLA_X_VEL + k] =
                self.ram[ANCILLA_X_VEL + k].wrapping_add(self.ram[ANCILLA_K + k]);
        }
        self.ancilla_move_x(k);
        if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
            && k == 4
            && self.ram[FRAME_COUNTER] >= 140
            && self.ram[FRAME_COUNTER] <= 210
        {
            eprintln!(
                "R boomerang-tick fc={} k={} x={:04x} y={:04x} xv={:02x} yv={:02x} step={:02x} aux={:02x} item={:02x} K={:02x} H={:02x} dir={:02x} arr23={:02x} link={:04x}/{:04x} hook={:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                self.ram[ANCILLA_X_VEL + k],
                self.ram[ANCILLA_Y_VEL + k],
                self.ram[ANCILLA_STEP + k],
                self.ram[ANCILLA_AUX_TIMER + k],
                self.ram[ANCILLA_ITEM_TO_LINK + k],
                self.ram[ANCILLA_K + k],
                self.ram[ANCILLA_H + k],
                self.ram[ANCILLA_DIR + k],
                self.ram[ANCILLA_ARR23 + k],
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.ram[HOOKSHOT_EFFECT_INDEX],
            );
        }
        let hit_spr = self.ancilla_check_sprite_collision(k);
        let trace_pre_item = self.ram[ANCILLA_ITEM_TO_LINK + k];
        let trace_pre_step = self.ram[ANCILLA_STEP + k];
        let trace_pre_k = self.ram[ANCILLA_K + k];

        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0 {
            if hit_spr.is_some() {
                self.ram[ANCILLA_ITEM_TO_LINK + k] ^= 1;
                if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
                    && k == 4
                    && self.ram[FRAME_COUNTER] >= 130
                    && self.ram[FRAME_COUNTER] <= 150
                {
                    eprintln!(
                        "R boomerang-branch fc={} reason=hit hit={} pre_item={:02x} item={:02x} pre_step={:02x} step={:02x} preK={:02x} K={:02x} x={:04x} y={:04x}",
                        self.ram[FRAME_COUNTER],
                        hit_spr.unwrap(),
                        trace_pre_item,
                        self.ram[ANCILLA_ITEM_TO_LINK + k],
                        trace_pre_step,
                        self.ram[ANCILLA_STEP + k],
                        trace_pre_k,
                        self.ram[ANCILLA_K + k],
                        self.ancilla_get_x(k),
                        self.ancilla_get_y(k),
                    );
                }
            } else if self.ancilla_check_tile_collision(k) != 0 {
                self.ancilla_add_boomerang_wall_clink(k);
                self.ancilla_sfx2_pan(
                    k,
                    if self.ram[ANCILLA_TILE_ATTR_PLAYER + k] == 0xf0 {
                        6
                    } else {
                        5
                    },
                );
                self.ram[ANCILLA_ITEM_TO_LINK + k] ^= 1;
                if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
                    && k == 4
                    && self.ram[FRAME_COUNTER] >= 130
                    && self.ram[FRAME_COUNTER] <= 150
                {
                    eprintln!(
                        "R boomerang-branch fc={} reason=tile attr={:02x} pre_item={:02x} item={:02x} pre_step={:02x} step={:02x} preK={:02x} K={:02x} x={:04x} y={:04x}",
                        self.ram[FRAME_COUNTER],
                        self.ram[ANCILLA_TILE_ATTR_PLAYER + k],
                        trace_pre_item,
                        self.ram[ANCILLA_ITEM_TO_LINK + k],
                        trace_pre_step,
                        self.ram[ANCILLA_STEP + k],
                        trace_pre_k,
                        self.ram[ANCILLA_K + k],
                        self.ancilla_get_x(k),
                        self.ancilla_get_y(k),
                    );
                }
            } else {
                let reached_edge = self.boomerang_screen_edge(k);
                if !reached_edge {
                    self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_sub(1);
                }
                if reached_edge || self.ram[ANCILLA_STEP + k] == 0 {
                    self.ram[ANCILLA_ITEM_TO_LINK + k] ^= 1;
                    if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
                        && k == 4
                        && self.ram[FRAME_COUNTER] >= 130
                        && self.ram[FRAME_COUNTER] <= 150
                    {
                        eprintln!(
                            "R boomerang-branch fc={} reason=edge-step pre_item={:02x} item={:02x} pre_step={:02x} step={:02x} preK={:02x} K={:02x} x={:04x} y={:04x}",
                            self.ram[FRAME_COUNTER],
                            trace_pre_item,
                            self.ram[ANCILLA_ITEM_TO_LINK + k],
                            trace_pre_step,
                            self.ram[ANCILLA_STEP + k],
                            trace_pre_k,
                            self.ram[ANCILLA_K + k],
                            self.ancilla_get_x(k),
                            self.ancilla_get_y(k),
                        );
                    }
                } else if self.ram[ANCILLA_STEP + k] < 5 {
                    self.ram[ANCILLA_K + k] = self.ram[ANCILLA_K + k].wrapping_sub(1);
                    if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
                        && k == 4
                        && self.ram[FRAME_COUNTER] >= 130
                        && self.ram[FRAME_COUNTER] <= 150
                    {
                        eprintln!(
                            "R boomerang-branch fc={} reason=outbound pre_item={:02x} item={:02x} pre_step={:02x} step={:02x} preK={:02x} K={:02x} x={:04x} y={:04x}",
                            self.ram[FRAME_COUNTER],
                            trace_pre_item,
                            self.ram[ANCILLA_ITEM_TO_LINK + k],
                            trace_pre_step,
                            self.ram[ANCILLA_STEP + k],
                            trace_pre_k,
                            self.ram[ANCILLA_K + k],
                            self.ancilla_get_x(k),
                            self.ancilla_get_y(k),
                        );
                    }
                } else if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
                    && k == 4
                    && self.ram[FRAME_COUNTER] >= 130
                    && self.ram[FRAME_COUNTER] <= 150
                {
                    eprintln!(
                        "R boomerang-branch fc={} reason=outbound pre_item={:02x} item={:02x} pre_step={:02x} step={:02x} preK={:02x} K={:02x} x={:04x} y={:04x}",
                        self.ram[FRAME_COUNTER],
                        trace_pre_item,
                        self.ram[ANCILLA_ITEM_TO_LINK + k],
                        trace_pre_step,
                        self.ram[ANCILLA_STEP + k],
                        trace_pre_k,
                        self.ram[ANCILLA_K + k],
                        self.ancilla_get_x(k),
                        self.ancilla_get_y(k),
                    );
                }
            }
        } else {
            let bak0 = self.ram[ANCILLA_OBJPRIO + k];
            let bak1 = self.ram[ANCILLA_FLOOR + k];
            self.ram[ANCILLA_FLOOR + k] = 0;
            self.ancilla_check_tile_collision(k);
            self.ram[ANCILLA_FLOOR + k] = bak1;
            self.ram[ANCILLA_OBJPRIO + k] = bak0;
            self.boomerang_stop_off_screen(k);
        }

        self.boomerang_draw(k);
    }

    fn ancilla01_somaria_bullet(&mut self, k: usize) {
        const SOMARIAN_BLAST_MASK: [u8; 6] = [7, 3, 1, 0, 0, 0];

        if self.frame_control_view().submodule() == 0 {
            if self.ram[FRAME_COUNTER] & SOMARIAN_BLAST_MASK[self.ram[ANCILLA_STEP + k] as usize]
                == 0
            {
                self.ancilla_move_x(k);
                self.ancilla_move_y(k);
            }
            if self.ram[ANCILLA_TIMER + k] == 0 {
                self.ram[ANCILLA_TIMER + k] = 3;
                let mut a = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                if a >= 6 {
                    a = 4;
                }
                self.ram[ANCILLA_STEP + k] = a;
            }
            if self.ancilla_check_sprite_collision(k).is_some()
                || self.ancilla_check_tile_collision_staggered(k) != 0
            {
                self.ram[ANCILLA_TYPE + k] = 4;
                self.ram[ANCILLA_TIMER + k] = 7;
                self.ram[ANCILLA_NUMSPR + k] = 16;
            }
        }
        self.somarian_blast_draw(k);
    }

    fn bomb_get_displacement_from_link(&self, k: usize) -> i32 {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        (((abs16(self.player_state_view().x().wrapping_add(8).wrapping_sub(x))
            + abs16(
                self.player_state_view()
                    .y()
                    .wrapping_add(12)
                    .wrapping_sub(y),
            ))
            & 0xfc)
            >> 2) as i32
    }

    fn hookshot_check_proximity_to_link(&self, x: i32, y: i32) -> bool {
        abs16(
            self.player_state_view()
                .y()
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
                .wrapping_add(12)
                .wrapping_sub(y as u16)
                .wrapping_sub(4),
        ) < 12
            && abs16(
                self.player_state_view()
                    .x()
                    .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2))
                    .wrapping_add(8)
                    .wrapping_sub(x as u16)
                    .wrapping_sub(4),
            ) < 12
    }

    fn hookshot_should_i_even_bother_with_tiles(&self, k: usize) -> bool {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            let area = (self.ram[CURRENT_AREA_OF_PLAYER_ANCILLA] >> 1) as usize;
            let bound = read_le_u16(&self.ram, OVERWORLD_RIGHT_BOTTOM_BOUND_FOR_SCROLL_ANCILLA);
            if self.ram[ANCILLA_DIR + k] & 2 == 0 {
                let t = y.wrapping_sub(K_OVERWORLD_OFFSET_BASE_Y_ANCILLA[area]);
                return t < 4 || t >= bound;
            } else {
                let t = x.wrapping_sub(K_OVERWORLD_OFFSET_BASE_X_ANCILLA[area]);
                return t < 6 || t >= bound;
            }
        }
        if self.ram[ANCILLA_DIR + k] & 2 == 0 {
            (y & 0x1ff) < 4
                || (y & 0x1ff) >= 0x1e8
                || (y & 0x200) != (self.player_state_view().y() & 0x200)
        } else {
            (x & 0x1ff) < 4
                || (x & 0x1ff) >= 0x1f0
                || (x & 0x200) != (self.player_state_view().x() & 0x200)
        }
    }

    fn boomerang_draw(&mut self, k: usize) {
        const BOOMERANG_FLAGS: [u8; 8] = [0xa4, 0xe4, 0x64, 0x24, 0xa2, 0xe2, 0x62, 0x22];
        const BOOMERANG_DRAW_XY: [i8; 8] = [2, -2, 2, 2, -2, 2, -2, -2];
        const BOOMERANG_DRAW_OAM_IDX: [u16; 2] = [0x180, 0xd0];
        const BOOMERANG_DRAW_TAB0: [u8; 2] = [3, 2];
        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);

        if self.ram[ANCILLA_ITEM_TO_LINK + k] != 0 {
            self.ram[ANCILLA_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
            const TAGALONG_LAYER_BITS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
            let priority =
                (TAGALONG_LAYER_BITS[self.ram[LINK_IS_ON_LOWER_LEVEL] as usize] as u16) << 8;
            write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, priority);
        }

        if self.ram[ANCILLA_OBJPRIO + k] != 0 {
            write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, 0x3000);
        }

        if self.frame_control_view().submodule() == 0 && self.ram[ANCILLA_AUX_TIMER + k] != 0 {
            self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_ARR3 + k]) {
                self.ram[ANCILLA_ARR3 + k] = BOOMERANG_DRAW_TAB0[self.ram[ANCILLA_G + k] as usize];
                self.ram[ANCILLA_ARR1 + k] = self.ram[ANCILLA_ARR1 + k].wrapping_add(
                    if self.ram[ANCILLA_S_PLAYER + k] != 0 {
                        0xff
                    } else {
                        1
                    },
                ) & 3;
            }
        }

        let j = self.ram[ANCILLA_ARR1 + k] as usize;
        let x = info_x.wrapping_add(BOOMERANG_DRAW_XY[j * 2 + 1] as i16 as u16);
        let y = info_y.wrapping_add(BOOMERANG_DRAW_XY[j * 2] as i16 as u16);
        if self.ram[ANCILLA_AUX_TIMER + k] == 0 {
            let i = BOOMERANG_DRAW_OAM_IDX[self.ram[SORT_SPRITES_SETTING] as usize];
            write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, (i >> 2) + 0xa20);
            write_le_u16(&mut self.ram, OAM_CUR_PTR, i + 0x800);
        }
        self.ancilla_set_oam_safe(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            x,
            y,
            0x26,
            (BOOMERANG_FLAGS[self.ram[ANCILLA_G + k] as usize * 4 + j] & !0x30)
                | self.ram[OAM_PRIORITY_VALUE + 1],
            2,
        );
    }

    fn ancilla06_wall_hit(&mut self, k: usize) {
        self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_ARR3 + k]) {
            let t = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if t == 5 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
            self.ram[ANCILLA_ITEM_TO_LINK + k] = t;
            self.ram[ANCILLA_ARR3 + k] = 1;
        }
        self.wall_hit_draw(k);
    }

    fn ancilla_sword_wall_hit(&mut self, k: usize) {
        self.ram[SPRITE_ALERT_FLAG] = 3;
        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
            let t = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if t == 8 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
            self.ram[ANCILLA_ITEM_TO_LINK + k] = t;
            self.ram[ANCILLA_AUX_TIMER + k] = 1;
        }
        self.wall_hit_draw(k);
    }

    fn ancilla1_d_screen_shake(&mut self, k: usize) {
        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_ITEM_TO_LINK + k]) {
                write_le_u16(&mut self.ram, BG1_X_OFFSET, 0);
                write_le_u16(&mut self.ram, BG1_Y_OFFSET, 0);
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
            let offs = self.dash_tremor_twiddle_offset(k);
            let j = self.ram[ANCILLA_DIR + k];
            if j == 0 {
                write_le_u16(&mut self.ram, BG1_X_OFFSET, offs as u16);
                self.ram[LINK_X_VEL] = self.ram[LINK_X_VEL].wrapping_add(offs as u8);
            } else {
                write_le_u16(&mut self.ram, BG1_Y_OFFSET, offs as u16);
                self.ram[LINK_Y_VEL] = self.ram[LINK_Y_VEL].wrapping_add(offs as u8);
            }
        }
        self.ram[SPRITE_ALERT_FLAG] = 3;
    }

    fn ancilla1_e_dash_dust(&mut self, k: usize) {
        if self.ram[ANCILLA_STEP + k] != 0 {
            self.dash_dust_motive(k);
            return;
        }
        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TIMER + k] = 3;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 5 {
                return;
            }
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 6 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }
        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 5 {
            return;
        }

        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;

        const DASH_DUST_DRAW_X1: [i8; 4] = [0, 0, 4, -4];
        const DASH_DUST_DRAW_X: [i16; 30] = [
            10, 5, -1, 0, 10, 5, 0, 5, -1, 0, -1, -1, 9, -1, -1, 10, 5, -1, 0, 10, 5, 0, 5, -1, 0,
            -1, -1, 9, -1, -1,
        ];
        const DASH_DUST_DRAW_Y: [i16; 30] = [
            -2, 0, -1, -3, -2, 0, -3, 0, -1, -3, -1, -1, -2, -1, -1, -2, 0, -1, -3, -2, 0, -3, 0,
            -1, -3, -1, -1, -2, -1, -1,
        ];
        const DASH_DUST_DRAW_CHAR: [u8; 30] = [
            0xcf, 0xa9, 0xff, 0xa9, 0xdf, 0xcf, 0xcf, 0xdf, 0xff, 0xdf, 0xff, 0xff, 0xa9, 0xff,
            0xff, 0xcf, 0xcf, 0xff, 0xcf, 0xdf, 0xcf, 0xcf, 0xdf, 0xff, 0xdf, 0xff, 0xff, 0xcf,
            0xff, 0xff,
        ];
        let r12 = DASH_DUST_DRAW_X1[(self.ram[LINK_DIRECTION_FACING] >> 1) as usize] as i16;
        let mut t = 3
            * (self.ram[ANCILLA_ITEM_TO_LINK + k] as usize
                + if self.ram[DRAW_WATER_RIPPLES_OR_GRASS] == 1 {
                    5
                } else {
                    0
                });

        for _ in (0..=2).rev() {
            if DASH_DUST_DRAW_CHAR[t] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    info_x
                        .wrapping_add(r12 as u16)
                        .wrapping_add(DASH_DUST_DRAW_X[t] as u16),
                    info_y.wrapping_add(DASH_DUST_DRAW_Y[t] as u16),
                    DASH_DUST_DRAW_CHAR[t],
                    4 | self.ram[OAM_PRIORITY_VALUE + 1],
                    0,
                );
                oam += 4;
            }
            t += 1;
        }
    }

    fn dash_dust_motive(&mut self, k: usize) {
        const MOTIVE_DASH_DUST_DRAW_CHAR: [u8; 3] = [0xa9, 0xcf, 0xdf];
        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TIMER + k] = 3;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 3 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }
        if self.ram[LINK_DIRECTION_FACING] == 2 {
            self.oam_allocate_from_region_b(4);
        }
        let frame = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
        if frame >= MOTIVE_DASH_DUST_DRAW_CHAR.len() {
            self.ram[ANCILLA_TYPE + k] = 0;
            return;
        }
        let (x, y) = self.ancilla_prep_oam_coord(k);
        self.ancilla_set_oam(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            x,
            y,
            MOTIVE_DASH_DUST_DRAW_CHAR[frame],
            4 | self.ram[OAM_PRIORITY_VALUE + 1],
            0,
        );
    }

    fn wall_hit_draw(&mut self, k: usize) {
        const WALL_HIT_X: [i8; 32] = [
            -4, 0, 0, 0, -4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -4, 0, 0, 0, -4, 0,
            0, 0, -8, 0, 0, 0,
        ];
        const WALL_HIT_Y: [i8; 32] = [
            -4, 0, 0, 0, -4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -4, 0, 0, 0, -4, 0,
            0, 0, -8, 0, 0, 0,
        ];
        const WALL_HIT_CHAR: [u8; 32] = [
            0x80, 0, 0, 0, 0x92, 0, 0, 0, 0x81, 0x81, 0x81, 0x81, 0x82, 0x82, 0x82, 0x82, 0x93,
            0x93, 0x93, 0x93, 0x92, 0, 0, 0, 0xb9, 0, 0, 0, 0x90, 0x90, 0, 0,
        ];
        const WALL_HIT_FLAGS: [u8; 32] = [
            0x32, 0, 0, 0, 0x32, 0, 0, 0, 0x32, 0x72, 0xb2, 0xf2, 0x32, 0x72, 0xb2, 0xf2, 0x32,
            0x72, 0xb2, 0xf2, 0x32, 0, 0, 0, 0x72, 0, 0, 0, 0x32, 0xf2, 0, 0,
        ];
        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        let mut t = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 4;

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for _ in (0..=3).rev() {
            if WALL_HIT_CHAR[t] != 0 {
                self.ancilla_set_oam(
                    oam,
                    info_x.wrapping_add(WALL_HIT_X[t] as i16 as u16),
                    info_y.wrapping_add(WALL_HIT_Y[t] as i16 as u16),
                    WALL_HIT_CHAR[t],
                    (WALL_HIT_FLAGS[t] & !0x30) | self.ram[OAM_PRIORITY_VALUE + 1],
                    0,
                );
                oam += 4;
            }
            oam = self.ancilla_allocate_oam_from_custom_region(oam);
            t += 1;
        }
    }

    fn ancilla08_door_debris(&mut self, k: usize) {
        self.door_debris_draw(k);
        self.ram[ANCILLA_ARR26 + k] = self.ram[ANCILLA_ARR26 + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_ARR26 + k]) {
            self.ram[ANCILLA_ARR26 + k] = 7;
            self.ram[ANCILLA_ARR25 + k] = self.ram[ANCILLA_ARR25 + k].wrapping_add(1);
            if self.ram[ANCILLA_ARR25 + k] == 4 {
                self.ram[ANCILLA_TYPE + k] = 0;
            }
        }
    }

    fn door_debris_draw(&mut self, k: usize) {
        const DOOR_DEBRIS_XY: [u16; 64] = [
            4, 7, 3, 17, 8, 8, 7, 17, 11, 7, 10, 16, 16, 7, 17, 17, 20, 7, 21, 17, 16, 8, 17, 17,
            13, 7, 14, 16, 8, 7, 7, 17, 7, 4, 17, 3, 8, 8, 17, 7, 7, 11, 16, 10, 7, 16, 17, 17, 7,
            20, 17, 21, 8, 16, 17, 17, 7, 13, 16, 14, 7, 8, 17, 7,
        ];
        const DOOR_DEBRIS_CHAR_FLAGS: [u16; 32] = [
            0x205e, 0xe05e, 0xa05e, 0x605e, 0x204f, 0x204f, 0x204f, 0x204f, 0x605e, 0x605e, 0x205e,
            0xe05e, 0x604f, 0x604f, 0x604f, 0x604f, 0x205e, 0xe05e, 0xa05e, 0x605e, 0x204f, 0xe04f,
            0x204f, 0x204f, 0x605e, 0x605e, 0x205e, 0xe05e, 0x604f, 0x604f, 0x604f, 0x604f,
        ];

        self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let y = read_le_u16(&self.ram, DOOR_DEBRIS_Y + k * 2)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        let x = read_le_u16(&self.ram, DOOR_DEBRIS_X + k * 2)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let j =
            self.ram[ANCILLA_ARR25 + k] as usize + self.ram[DOOR_DEBRIS_DIRECTION + k] as usize * 4;

        for i in 0..2 {
            let t = j * 2 + i;
            let d = DOOR_DEBRIS_CHAR_FLAGS[t];
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(DOOR_DEBRIS_XY[t * 2 + 1]),
                y.wrapping_add(DOOR_DEBRIS_XY[t * 2]),
                d as u8,
                ((d >> 8) as u8 & 0xc0) | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
        }
    }

    fn ancilla_add_boomerang_wall_clink(&mut self, k: usize) {
        const BOOMERANG_WALL_HIT_X: [i8; 8] = [8, 8, 0, 10, 12, 8, 4, 0];
        const BOOMERANG_WALL_HIT_Y: [i8; 8] = [0, 8, 8, 8, 4, 8, 12, 8];
        const BOOMERANG_WALL_HIT_TAB0: [u8; 16] =
            [0, 6, 4, 0, 2, 10, 12, 0, 0, 8, 14, 0, 0, 0, 0, 0];
        let temp_x = self.ancilla_get_x(k);
        let temp_y = self.ancilla_get_y(k);
        write_le_u16(&mut self.ram, BOOMERANG_TEMP_X, temp_x);
        write_le_u16(&mut self.ram, BOOMERANG_TEMP_Y, temp_y);
        if let Some(k) = self.ancilla_add_ancilla(6, 1) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_ARR3 + k] = 1;
            let j =
                (BOOMERANG_WALL_HIT_TAB0[self.ram[HOOKSHOT_EFFECT_INDEX] as usize] >> 1) as usize;
            self.ancilla_set_xy(
                k,
                read_le_u16(&self.ram, BOOMERANG_TEMP_X)
                    .wrapping_add(BOOMERANG_WALL_HIT_X[j] as i16 as u16),
                read_le_u16(&self.ram, BOOMERANG_TEMP_Y)
                    .wrapping_add(BOOMERANG_WALL_HIT_Y[j] as i16 as u16),
            );
        }
    }

    pub(super) fn call_for_duck_indoors(&mut self) {
        self.ancilla_sfx2_near(0x13);
        self.ancilla_add_duck_take_off(0x27, 4);
    }

    pub(super) fn ancilla_add_duck_take_off(&mut self, a: u8, y: u8) {
        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_TIMER + k] = 0x78;
            self.ram[ANCILLA_L + k] = 0;
            self.ram[ANCILLA_Z_VEL + k] = 0;
            self.ram[ANCILLA_Z + k] = 0;
            self.ram[ANCILLA_STEP + k] = 0;
            self.add_bird_common(k);
        }
    }

    fn ancilla30_byrna_windup_spark(&mut self, k: usize) {
        const INITIAL_CANE_SPARK_X: [i8; 16] =
            [3, 1, 0, 0, 13, 16, 12, 12, 24, 7, -4, -10, -8, 9, 22, 26];
        const INITIAL_CANE_SPARK_Y: [i8; 16] =
            [5, 0, -3, -6, -8, -3, 12, 28, 5, 0, 8, 16, 5, 0, 8, 16];
        const INITIAL_CANE_SPARK_DRAW_X: [i8; 16] =
            [-4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0];
        const INITIAL_CANE_SPARK_DRAW_Y: [i8; 16] =
            [-4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0];
        const INITIAL_CANE_SPARK_DRAW_CHAR: [u8; 16] = [
            0x92, 0xff, 0xff, 0xff, 0x8c, 0x8c, 0x8c, 0x8c, 0xd6, 0xd6, 0xd6, 0xd6, 0x93, 0x93,
            0x93, 0x93,
        ];
        const INITIAL_CANE_SPARK_DRAW_FLAGS: [u8; 16] = [
            0x22, 0xff, 0xff, 0xff, 0x22, 0x62, 0xa2, 0xe2, 0x24, 0x64, 0xa4, 0xe4, 0x22, 0x62,
            0xa2, 0xe2,
        ];

        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                self.ram[ANCILLA_AUX_TIMER + k] = 1;
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                if self.ram[ANCILLA_ITEM_TO_LINK + k] == 17 {
                    self.byrna_windup_spark_transmute_to_normal(k);
                    return;
                }
            }
        }
        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0 {
            return;
        }

        let mut j = self.ram[PLAYER_HANDLER_TIMER];
        if j == 2 {
            let mut a = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
            if sign8(a) {
                a = 0;
                j = 3;
            }
            self.ram[ANCILLA_ARR3 + k] = a;
        }
        let j = j.wrapping_add(self.ram[LINK_DIRECTION_FACING].wrapping_mul(2)) as usize;
        self.ancilla_set_xy(
            k,
            self.player_state_view()
                .x()
                .wrapping_add(INITIAL_CANE_SPARK_X[j] as i16 as u16),
            self.player_state_view()
                .y()
                .wrapping_add(INITIAL_CANE_SPARK_Y[j] as i16 as u16),
        );
        let (x, y) = self.ancilla_prep_oam_coord(k);

        let a = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_sub(1) & 0x0f;
        let mut j = 0usize;
        if a != 0 {
            j = 4 * if a != 15 { ((a & 1) + 1) as usize } else { 3 };
        }

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for _ in 0..4 {
            if INITIAL_CANE_SPARK_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(INITIAL_CANE_SPARK_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(INITIAL_CANE_SPARK_DRAW_Y[j] as i16 as u16),
                    INITIAL_CANE_SPARK_DRAW_CHAR[j],
                    INITIAL_CANE_SPARK_DRAW_FLAGS[j] & !0x30 | self.ram[OAM_PRIORITY_VALUE + 1],
                    0,
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn byrna_windup_spark_transmute_to_normal(&mut self, k: usize) {
        const CANE_SPARK_TRANSMUTE_TAB: [u8; 16] = [
            0x34, 0x33, 0x32, 0x31, 0x16, 0x15, 0x14, 0x13, 0x2a, 0x29, 0x28, 0x27, 0x10, 0x0f,
            0x0e, 0x0d,
        ];
        self.ram[ANCILLA_TYPE + k] = 0x31;
        let j = (self.ram[LINK_DIRECTION_FACING] << 1) as usize;
        self.ram[SWORDBEAM_ARR] = CANE_SPARK_TRANSMUTE_TAB[j];
        self.ram[SWORDBEAM_ARR + 1] = CANE_SPARK_TRANSMUTE_TAB[j + 1];
        self.ram[SWORDBEAM_ARR + 2] = CANE_SPARK_TRANSMUTE_TAB[j + 2];
        self.ram[SWORDBEAM_ARR + 3] = CANE_SPARK_TRANSMUTE_TAB[j + 3];
        self.ram[ANCILLA_AUX_TIMER + k] = 0x17;
        self.ram[ANCILLA_G + k] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_ARR3 + k] = 8;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_L + k] = 0;
        self.ram[ANCILLA_ARR1 + k] = 2;
        self.ram[ANCILLA_TIMER + k] = 21;
        self.ram[SWORDBEAM_VAR2] = 20;
        self.ancilla_sfx3_near(0x30);
        self.ancilla31_byrna_spark(k);
    }

    fn ancilla31_byrna_spark(&mut self, k: usize) {
        const CANE_SPARK_MAGIC: [u8; 3] = [4, 2, 1];
        const CANE_SPARK_CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];

        let mut flags = 2;
        if self.frame_control_view().submodule() == 0 {
            if self.ram[CURRENT_ITEM_Y] != 13 {
                self.kill_byrna_spark(k);
                return;
            }
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if self.ram[ANCILLA_AUX_TIMER + k] == 0 {
                self.ram[ANCILLA_AUX_TIMER + k] = 1;
                let r0 = CANE_SPARK_MAGIC[self.ram[LINK_MAGIC_CONSUMPTION] as usize];
                let r0 = self.ram[LINK_MAGIC_POWER].wrapping_sub(r0);
                if self.ram[LINK_MAGIC_POWER] == 0 || r0 >= 0x80 {
                    self.kill_byrna_spark(k);
                    return;
                }

                self.ram[ANCILLA_G + k] = self.ram[ANCILLA_G + k].wrapping_sub(1);
                if sign8(self.ram[ANCILLA_G + k]) {
                    self.ram[ANCILLA_G + k] = 0x17;
                    self.ram[LINK_MAGIC_POWER] = r0;
                }
                if self.ram[FILTERED_JOYPAD_H] & 0x40 != 0 {
                    self.kill_byrna_spark(k);
                    return;
                }
            }
            if self.ram[ANCILLA_STEP + k] != 3 {
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                let a = self.ram[ANCILLA_ITEM_TO_LINK + k];
                self.ram[ANCILLA_STEP + k] = if a >= 4 {
                    3
                } else if a == 2 {
                    1
                } else if a == 3 {
                    2
                } else {
                    0
                };
            }
            self.ram[ANCILLA_ARR1 + k] = self.ram[ANCILLA_ARR1 + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_ARR1 + k]) {
                self.ram[ANCILLA_ARR1 + k] = 2;
                flags = 4;
            }
        }

        let mut z = self.ram[LINK_Z_COORD] as i8 as i16;
        if z == -1 {
            z = 0;
        }
        let swordbeam_temp_y = self
            .player_state_view()
            .y()
            .wrapping_add(12)
            .wrapping_sub(z as u16);
        let swordbeam_temp_x = self.player_state_view().x().wrapping_add(8);
        write_le_u16(&mut self.ram, SWORDBEAM_TEMP_Y, swordbeam_temp_y);
        write_le_u16(&mut self.ram, SWORDBEAM_TEMP_X, swordbeam_temp_x);
        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TIMER + k] = 21;
            self.ancilla_sfx3_near(0x30);
        }
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut i = self.ram[ANCILLA_STEP + k] as usize;
        loop {
            if self.frame_control_view().submodule() == 0 {
                self.ram[SWORDBEAM_ARR + i] = self.ram[SWORDBEAM_ARR + i].wrapping_add(3) & 0x3f;
            }
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                self.ram[SWORDBEAM_ARR + i],
                self.ram[SWORDBEAM_VAR2],
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                CANE_SPARK_CHAR[i],
                flags | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
            self.ancilla_set_xy(
                k,
                pt.x.wrapping_add(read_le_u16(&self.ram, BG2HOFS_COPY2)),
                pt.y.wrapping_add(read_le_u16(&self.ram, BG2VOFS_COPY2)),
            );
            self.ram[ANCILLA_DIR + k] = 0;
            self.ancilla_check_sprite_collision(k);
            oam += 4;
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    fn kill_byrna_spark(&mut self, k: usize) {
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ram[ANCILLA_TYPE + k] = 0;
        self.ram[LINK_GIVE_DAMAGE] = 0;
    }

    pub(super) fn configure_revival_ancillae(&mut self) {
        self.ram[LINK_DMA_VAR5] = 80;
        let mut k = 0usize;

        self.ram[ANCILLA_ARR3 + k] = 64;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_Z_VEL + k] = 8;
        self.ram[ANCILLA_L + k] = 0;
        self.ram[ANCILLA_G + k] = 5;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_K + k] = 0;
        self.ancilla_set_xy(
            k,
            self.player_state_view().x(),
            self.player_state_view().y(),
        );
        self.ram[ANCILLA_Z + k] = 0;
        k += 1;

        self.ram[ANCILLA_Z + k] = 0;
        self.ram[ANCILLA_ARR3 + k] = 240;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_K + k] = 0;
        k += 1;

        self.ram[ANCILLA_ITEM_TO_LINK + k] = 2;
        self.ram[ANCILLA_AUX_TIMER + k] = 3;
        self.ram[ANCILLA_ARR3 + k] = 8;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_DIR + k] = 3;
        self.ram[ANCILLA_ARR25 + k] =
            K_MAGIC_POWDER_TAB0[30 + self.ram[ANCILLA_ITEM_TO_LINK + k] as usize];

        self.ancilla_set_xy(
            k,
            self.player_state_view().x().wrapping_add(20),
            self.player_state_view().y().wrapping_add(2),
        );
    }

    pub(super) fn ancilla_add_bunny_poof(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[LINK_VISIBILITY_STATUS] = 0x0c;
            self.ram[ANCILLA_STEP + k] = 0;
            self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan()
                | if self.ram[LINK_IS_BUNNY_MIRROR] == 0 {
                    0x14
                } else {
                    0x15
                };
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_AUX_TIMER + k] = 7;
            self.ancilla_set_xy(
                k,
                self.player_state_view().x(),
                self.player_state_view().y().wrapping_add(4),
            );
        }
    }

    pub(super) fn ancilla_add_dwarf_poof(&mut self, ain: u8, yin: u8) {
        let Some(k) = self.ancilla_add_ancilla(ain, yin) else {
            return;
        };
        self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan()
            | if self.ram[FOLLOWER_INDICATOR_ANCILLA] == 8 {
                0x14
            } else {
                0x15
            };

        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_AUX_TIMER + k] = 7;
        self.ram[TAGALONG_APPEARANCE_NONE_FLAG_ANCILLA] = 1;
        let j = self.ram[TAGALONG_DATA_INDEX_ANCILLA] as usize;
        let x = self.ram[TAGALONG_X_LO_ANCILLA + j] as u16
            | ((self.ram[TAGALONG_X_HI_ANCILLA + j] as u16) << 8);
        let y = self.ram[TAGALONG_Y_LO_ANCILLA + j] as u16
            | ((self.ram[TAGALONG_Y_HI_ANCILLA + j] as u16) << 8);
        self.ancilla_set_xy(k, x, y.wrapping_add(4));
    }

    pub(super) fn ancilla_add_bush_poof(&mut self, x: u16, y: u16) {
        if self.ram[LINK_ITEM_IN_HAND] & 0x40 == 0 {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(0x3f, 4) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_TIMER + k] = 7;
            self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 21;
            self.ancilla_set_xy(k, x, y.wrapping_sub(2));
        }
    }

    pub(super) fn ancilla_add_victory_spin(&mut self) {
        if self.ram[LINK_SWORD_TYPE].wrapping_add(1) & 0xfe != 0 {
            if let Some(k) = self.ancilla_add_ancilla(0x3b, 0) {
                self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
                self.ram[ANCILLA_ARR3 + k] = 1;
                self.ram[ANCILLA_AUX_TIMER + k] = 34;
            }
        }
    }

    pub(super) fn ancilla_add_magic_powder(&mut self, a: u8, y: u8) {
        const MAGIC_POWER_X: [i8; 4] = [-2, -2, -12, 12];
        const MAGIC_POWER_Y: [i8; 4] = [0, 20, 16, 16];
        const MAGIC_POWER_X1: [i8; 4] = [10, 10, -8, 28];
        const MAGIC_POWER_Y1: [i8; 4] = [1, 40, 22, 22];

        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_Z + k] = 0;
            self.ram[ANCILLA_AUX_TIMER + k] = 1;
            self.ram[LINK_DMA_VAR5] = 80;
            let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
            self.ram[ANCILLA_DIR + k] = j as u8;
            self.ram[ANCILLA_ARR25 + k] = K_MAGIC_POWDER_TAB0[j * 10];
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(MAGIC_POWER_X[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(MAGIC_POWER_Y[j] as i16 as u16),
            );
            self.ancilla_check_tile_collision(k);
            self.ram[DUNGEON_TORCH_ATTR] = self.ram[ANCILLA_TILE_ATTR_PLAYER + k];
            if self.ram[CURRENT_ITEM_ACTIVE] == 9 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
            self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 0x0d;
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(MAGIC_POWER_X1[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(MAGIC_POWER_Y1[j] as i16 as u16),
            );
        }
    }

    pub(super) fn ancilla_add_wall_tap_spark(&mut self, a: u8, y: u8) {
        const WALL_TAP_SPARK_X: [i8; 4] = [11, 10, -12, 29];
        const WALL_TAP_SPARK_Y: [i8; 4] = [-4, 32, 17, 17];
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 5;
            self.ram[ANCILLA_AUX_TIMER + k] = 1;
            let i = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(WALL_TAP_SPARK_X[i] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(WALL_TAP_SPARK_Y[i] as i16 as u16),
            );
        }
    }

    pub(super) fn ancilla_add_lamp_flame(&mut self, a: u8, y: u8) {
        const LAMP_FLAME_X: [i8; 4] = [0, 0, -20, 18];
        const LAMP_FLAME_Y: [i8; 4] = [-16, 24, 4, 4];
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_AUX_TIMER + k] = 0;
            self.ram[ANCILLA_TIMER + k] = 23;
            let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
            self.ram[ANCILLA_DIR + k] = j as u8;
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(LAMP_FLAME_X[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(LAMP_FLAME_Y[j] as i16 as u16),
            );
            self.ram[SOUND_EFFECT_1] = self.ancilla_calculate_sfx_pan(k) | 42;
        }
    }

    pub(super) fn ancilla_add_ms_cutscene(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_AUX_TIMER + k] = 2;
            self.ram[ANCILLA_TIMER + k] = 64;
            self.ancilla_set_xy(
                k,
                self.player_state_view().x().wrapping_add(8),
                self.player_state_view().y().wrapping_sub(8),
            );
        }
    }

    pub(super) fn ancilla_add_dash_tremor(&mut self, a: u8, y: u8) {
        const ADD_DASH_TREMOR_DIR: [u8; 4] = [2, 2, 0, 0];
        const ADD_DASH_TREMOR_TAB: [u8; 2] = [0x80, 0x78];

        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 16;
            self.ram[ANCILLA_L + k] = 0;
            let mut j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
            j = ADD_DASH_TREMOR_DIR[j] as usize;
            self.ram[ANCILLA_DIR + k] = j as u8;
            let y = self
                .player_state_view()
                .y()
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)) as u8;
            let x = self
                .player_state_view()
                .x()
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)) as u8;
            let coord = if j != 0 { y } else { x };
            self.ancilla_set_y(
                k,
                if coord < ADD_DASH_TREMOR_TAB[j >> 1] {
                    3
                } else {
                    (-3i8) as u16
                },
            );
        }
    }

    fn ancilla_add_hookshot_wall_clink(&mut self, kin: usize, a: u8, y: u8) {
        const HOOKSHOT_WALL_HIT_X: [i8; 8] = [8, 8, 0, 10, 12, 8, 4, 0];
        const HOOKSHOT_WALL_HIT_Y: [i8; 8] = [0, 8, 8, 8, 4, 8, 12, 8];

        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_ARR3 + k] = 1;
            let j = self.ram[ANCILLA_DIR + kin] as usize;
            self.ancilla_set_xy(
                k,
                self.ancilla_get_x(kin)
                    .wrapping_add(HOOKSHOT_WALL_HIT_X[j] as i16 as u16),
                self.ancilla_get_y(kin)
                    .wrapping_add(HOOKSHOT_WALL_HIT_Y[j] as i16 as u16),
            );
        }
    }

    pub(super) fn ancilla_add_quake_spell(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_STEP + k] = 0;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 13;
            self.ram[SOUND_EFFECT_1] = 0x35;
            for i in 0..5 {
                self.ram[QUAKE_ARR2 + i] = 0;
            }
            self.ram[QUAKE_VAR5] = 0;
            for i in 0..5 {
                self.ram[QUAKE_ARR1 + i] = 1;
            }
            self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 1;
            self.ram[ANCILLA_TIMER + k] = 2;
            let quake_var1 = self.player_state_view().y().wrapping_add(26);
            let quake_var2 = self.player_state_view().x().wrapping_add(8);
            write_le_u16(&mut self.ram, QUAKE_VAR1, quake_var1);
            write_le_u16(&mut self.ram, QUAKE_VAR2, quake_var2);
            write_le_u16(&mut self.ram, QUAKE_VAR3, 3);
        }
    }

    pub(super) fn ancilla_add_ether_spell(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_ARR25 + k] = 0;
            self.ram[ANCILLA_STEP + k] = 0;
            self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 1;
            self.ram[ANCILLA_AUX_TIMER + k] = 2;
            self.ram[ANCILLA_ARR3 + k] = 3;
            self.ram[ANCILLA_Y_VEL + k] = 127;
            self.ram[ETHER_VAR2] = 40;
            self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 9;
            self.ram[ETHER_VAR1] = 0x40;
            self.ram[SOUND_EFFECT_2] = self.link_calculate_sfx_pan() | 0x26;
            for i in 0..8 {
                self.ram[ETHER_ARR1 + i] = (i * 8) as u8;
            }
            let ether_y = self.player_state_view().y();
            write_le_u16(&mut self.ram, ETHER_Y, ether_y);
            let y = read_le_u16(&self.ram, BG2VOFS_COPY2).wrapping_sub(16);
            write_le_u16(&mut self.ram, ETHER_Y_ADJUSTED, y & 0x00f0);
            let ether_x = self.player_state_view().x();
            write_le_u16(&mut self.ram, ETHER_X, ether_x);
            write_le_u16(&mut self.ram, ETHER_X2, ether_x.wrapping_add(8));
            let ether_y2 = ether_y.wrapping_sub(16);
            write_le_u16(&mut self.ram, ETHER_Y2, ether_y2);
            write_le_u16(&mut self.ram, ETHER_Y3, ether_y2.wrapping_add(0x24));
            self.ancilla_set_xy(k, ether_x, y);
        }
    }

    fn ancilla18_ether_spell(&mut self, k: usize) {
        if self.frame_control_view().submodule() != 0 {
            return;
        }

        if self.ram[ANCILLA_STEP + k] != 0 {
            let flag = if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 0 {
                self.ram[ANCILLA_ARR4 + k] = self.ram[ANCILLA_ARR4 + k].wrapping_add(1);
                self.ram[ANCILLA_ARR4 + k] & 4 == 0
            } else {
                self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 11
            };
            if flag {
                self.palette_electro_themed_gear();
                self.filter_majorly_whiten_bg();
            } else {
                self.load_actual_gear_palettes();
                self.palette_restore_bg_from_flash();
            }
        }

        if self.ram[ANCILLA_STEP + k] == 2 {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                self.ram[ANCILLA_AUX_TIMER + k] = 2;
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                if self.ram[ANCILLA_ITEM_TO_LINK + k] == 2 {
                    self.ram[ANCILLA_ITEM_TO_LINK + k] =
                        self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_sub(1);
                    self.ram[ANCILLA_X_VEL + k] = 16;
                    self.ram[ANCILLA_STEP + k] = 3;
                }
            }
            self.ram[ANCILLA_X_VEL + k] = self.ram[ANCILLA_X_VEL + k].wrapping_add(1);
            self.ether_spell_handle_radial_spin(k);
            return;
        }

        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
            self.ram[ANCILLA_AUX_TIMER + k] = 2;
            self.ram[ANCILLA_ITEM_TO_LINK + k] ^= 1;
        }
        if self.ram[ANCILLA_STEP + k] == 0 {
            self.ether_spell_handle_lightning_stroke(k);
        } else if self.ram[ANCILLA_STEP + k] == 1 {
            self.ether_spell_handle_orb_pulse(k);
        } else if self.ram[ANCILLA_STEP + k] == 3 {
            self.ether_spell_handle_radial_spin(k);
        } else if self.ram[ANCILLA_STEP + k] == 4 {
            self.ram[ETHER_VAR1] = self.ram[ETHER_VAR1].wrapping_sub(1);
            if self.ram[ETHER_VAR1] == 0 {
                self.ram[ANCILLA_STEP + k] = 5;
            }
            self.ether_spell_handle_radial_spin(k);
        } else {
            let mut vel = self.ram[ANCILLA_X_VEL + k].wrapping_add(0x10);
            if sign8(vel) {
                vel = 0x7f;
            }
            self.ram[ANCILLA_X_VEL + k] = vel;
            self.ether_spell_handle_radial_spin(k);
        }
    }

    fn ether_spell_handle_lightning_stroke(&mut self, k: usize) {
        self.ancilla_move_y(k);
        let y = self.ancilla_get_y(k);

        if self.ram[ETHER_Y_ADJUSTED] != (y & 0xf0) as u8 {
            self.ram[ETHER_Y_ADJUSTED] = (y & 0xf0) as u8;
            self.ram[ANCILLA_ARR25 + k] = self.ram[ANCILLA_ARR25 + k].wrapping_add(1);
        }
        if y < 0xe000
            && read_le_u16(&self.ram, ETHER_Y2) < 0xe000
            && read_le_u16(&self.ram, ETHER_Y2) <= y
        {
            self.ram[ANCILLA_STEP + k] = 1;
        }
        self.ancilla_draw_ether_blitz(k);
    }

    fn ether_spell_handle_orb_pulse(&mut self, k: usize) {
        if !sign8(self.ram[ANCILLA_ARR25 + k]) {
            self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
            if !sign8(self.ram[ANCILLA_ARR3 + k]) {
                self.ancilla_draw_ether_blitz(k);
                return;
            }
            self.ram[ANCILLA_ARR3 + k] = 3;
            self.ram[ANCILLA_ARR25 + k] = self.ram[ANCILLA_ARR25 + k].wrapping_sub(1);
            if !sign8(self.ram[ANCILLA_ARR25 + k]) {
                self.ancilla_draw_ether_blitz(k);
                return;
            }
            self.ram[ANCILLA_ARR3 + k] = 9;
        }
        self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_ARR3 + k]) {
            self.ram[ANCILLA_STEP + k] = 2;
            self.ram[ANCILLA_Y_VEL + k] = 0;
            self.ram[ANCILLA_X_VEL + k] = 16;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_AUX_TIMER + k] = 2;
            if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] != 0 {
                self.medallion_check_sprite_damage(k);
            }
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        self.ancilla_draw_ether_orb(k, oam);
    }

    fn ether_spell_handle_radial_spin(&mut self, k: usize) {
        if self.ram[ANCILLA_STEP + k] == 4 {
            if self.ram[FRAME_COUNTER] & 7 == 0 {
                self.ram[SOUND_EFFECT_2] = 0x2a;
            } else if self.ram[FRAME_COUNTER] & 7 == 4 {
                self.ram[SOUND_EFFECT_2] = 0xaa;
            } else if self.ram[FRAME_COUNTER] & 7 == 7 {
                self.ram[SOUND_EFFECT_2] = 0x6a;
            }
        } else {
            self.ram[ANCILLA_X_LO + k] = self.ram[ETHER_VAR2];
            self.ram[ANCILLA_X_HI + k] = 0;
            self.ancilla_move_x(k);
            self.ram[ETHER_VAR2] = self.ram[ANCILLA_X_LO + k];
            if self.ram[ETHER_VAR2] == 0x40 {
                self.ram[ANCILLA_STEP + k] = 4;
            }
        }

        let sb = self.ram[ANCILLA_STEP + k];
        let sa = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for i in (0..=7).rev() {
            if sb != 2 && sb != 5 {
                self.ram[ETHER_ARR1 + i] = self.ram[ETHER_ARR1 + i].wrapping_add(1) & 0x3f;
            }
            let arp =
                self.ancilla_get_radial_projection(self.ram[ETHER_ARR1 + i], self.ram[ETHER_VAR2]);
            if sb != 2 {
                oam = self.ancilla_draw_ether_blitz_ball(oam, &arp, sa);
            } else {
                oam = self.ancilla_draw_ether_blitz_segment(oam, &arp, sa, i);
            }
        }
        if self.ram[ETHER_VAR2] < 0xf0 {
            let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
            for i in 0..8 {
                if self.ram[oam + i * 4 + 1] != 0xf0 {
                    return;
                }
            }
        }

        self.ram[ANCILLA_TYPE + k] = 0;
        self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 1;
        self.ram[SPIN_ATTACK_SOUND_LATCH] = 0;
        self.ram[STATE_FOR_SPIN_ATTACK] = 0;
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[FLAG_UNK1] = 0;

        if self.ram[OVERWORLD_SCREEN_INDEX] == 0x70
            && self.ram[SAVE_OW_EVENT_INFO_ANCILLA + 0x70] & 0x20 == 0
            && self.ancilla_check_for_entrance_trigger(2)
        {
            self.ram[TRIGGER_SPECIAL_ENTRANCE_ANCILLA] = 3;
            self.frame_control_view_mut().set_subsubmodule(0);
            self.ram[R16] = 0;
        }

        if self.ram[LINK_PLAYER_HANDLER_STATE] != 25 {
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
            self.ram[BUTTON_MASK_B_Y] = if self.ram[BUTTON_B_FRAMES] != 0 {
                self.ram[JOYPAD1H_LAST] & 0x80
            } else {
                0
            };
        }
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[MAGIC_SPELL_PLAYER_LOCK_FLAG] = 0;
        self.load_actual_gear_palettes();
        self.palette_restore_bg_and_hud();
    }

    pub(super) fn ancilla_add_bombos_spell(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_add_ancilla_bank08(a, y) else {
            return;
        };
        for i in 0..10 {
            self.ram[BOMBOS_ARR2 + i] = 0;
            self.ram[BOMBOS_ARR1 + i] = 3;
        }
        for i in 0..8 {
            self.ram[BOMBOS_ARR3 + i] = 0;
            self.ram[BOMBOS_ARR4 + i] = 3;
        }
        self.ram[BOMBOS_VAR4] = 0;
        self.ram[BOMBOS_VAR2] = 0;
        self.ram[BOMBOS_VAR3] = 0x80;
        self.ram[BOMBOS_ARR7] = 0x10;
        self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 11;
        self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 1;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ancilla_sfx2_near(0x2a);

        let mut t = self.asset_u8(72, self.ram[FRAME_COUNTER] as usize);
        t = if t < 0xe0 { t } else { t & 0x7f };
        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        write_le_u16(
            &mut self.ram,
            BOMBOS_X_COORD,
            (link_x & !0xff) | u16::from(t),
        );
        write_le_u16(
            &mut self.ram,
            BOMBOS_Y_COORD,
            (link_y & !0xff) | u16::from(t),
        );

        const BOMBOS_Y_DELTA: [i16; 4] = [16, 24, -128, -16];
        const BOMBOS_X_DELTA: [i16; 4] = [-16, -128, 0, 128];

        for i in 0..1 {
            let bombos_x_coord2 = link_x.wrapping_add(BOMBOS_X_DELTA[i] as u16);
            let bombos_y_coord2 = link_y.wrapping_add(BOMBOS_Y_DELTA[i] as u16);
            write_le_u16(&mut self.ram, BOMBOS_X_COORD2 + i * 2, bombos_x_coord2);
            write_le_u16(&mut self.ram, BOMBOS_Y_COORD2 + i * 2, bombos_y_coord2);
            self.ram[BOMBOS_VAR1] = 16;
            let arp = self.ancilla_get_radial_projection(self.ram[BOMBOS_ARR7 + i], 16);
            let x = (if arp.r6 != 0 {
                -(arp.r4 as i32)
            } else {
                arp.r4 as i32
            }) + i32::from(bombos_x_coord2);
            let y = (if arp.r2 != 0 {
                -(arp.r0 as i32)
            } else {
                arp.r0 as i32
            }) + i32::from(bombos_y_coord2);
            self.ram[BOMBOS_X_LO + i] = x as u8;
            self.ram[BOMBOS_X_HI + i] = ((x as u16) >> 8) as u8;
            self.ram[BOMBOS_Y_LO + i] = y as u8;
            self.ram[BOMBOS_Y_HI + i] = ((y as u16) >> 8) as u8;
        }
    }

    fn ancilla19_bombos_spell(&mut self, k: usize) {
        if self.ram[BOMBOS_VAR4] == 0 {
            if self.frame_control_view().submodule() == 0 {
                self.bombos_spell_control_fire_columns(k);
                return;
            }
            for i in (0..=9).rev() {
                self.ancilla_draw_bombos_fire_column(i);
            }
        } else if self.ram[BOMBOS_VAR4] != 2 {
            if self.frame_control_view().submodule() == 0 {
                self.bombos_spell_finish_fire_columns(k);
                return;
            }
            for i in (0..=9).rev() {
                self.ancilla_draw_bombos_fire_column(i);
            }
        } else {
            if self.frame_control_view().submodule() == 0 {
                self.bombos_spell_control_blasting(k);
                return;
            }
            let mut i = self.ram[ANCILLA_STEP + k] as i32;
            loop {
                self.ancilla_draw_bombos_blast(i as usize);
                i -= 1;
                if i < 0 {
                    break;
                }
            }
        }
    }

    fn bombos_spell_control_fire_columns(&mut self, k: usize) {
        let sa = self.ram[ANCILLA_ITEM_TO_LINK + k];
        let mut sb = self.ram[ANCILLA_STEP + k];

        let mut i = sb as i32;
        loop {
            let ui = i as usize;
            if self.ram[BOMBOS_ARR2 + ui] != 13 {
                let arr1 = self.ram[BOMBOS_ARR1 + ui].wrapping_sub(1);
                self.ram[BOMBOS_ARR1 + ui] = arr1;
                if sign8(arr1) {
                    self.ram[BOMBOS_ARR1 + ui] = 3;
                    self.ram[BOMBOS_ARR2 + ui] = self.ram[BOMBOS_ARR2 + ui].wrapping_add(1);
                    if self.ram[BOMBOS_ARR2 + ui] != 13 {
                        if self.ram[BOMBOS_ARR2 + ui] == 2 && sa == 0 {
                            let j = if sb == 9 {
                                let mut found: Option<usize> = None;
                                for candidate in (0..=9).rev() {
                                    if self.ram[BOMBOS_ARR2 + candidate] == 13 {
                                        self.ram[BOMBOS_ARR2 + candidate] = 0;
                                        found = Some(candidate);
                                        break;
                                    }
                                }
                                found.unwrap_or(9)
                            } else {
                                sb = if sb.wrapping_add(1) != 10 {
                                    sb.wrapping_add(1)
                                } else {
                                    9
                                };
                                sb as usize
                            };
                            self.ram[BOMBOS_VAR1] = if self.ram[BOMBOS_VAR1].wrapping_add(3) >= 207
                            {
                                207
                            } else {
                                self.ram[BOMBOS_VAR1].wrapping_add(3)
                            };
                            self.ram[BOMBOS_ARR7] = self.ram[BOMBOS_ARR7].wrapping_add(6);
                            let arp = self.ancilla_get_radial_projection(
                                self.ram[BOMBOS_ARR7] & 0x3f,
                                self.ram[BOMBOS_VAR1],
                            );
                            let x = (if arp.r6 != 0 {
                                -(arp.r4 as i32)
                            } else {
                                arp.r4 as i32
                            }) + i32::from(read_le_u16(&self.ram, BOMBOS_X_COORD2));
                            let y = (if arp.r2 != 0 {
                                -(arp.r0 as i32)
                            } else {
                                arp.r0 as i32
                            }) + i32::from(read_le_u16(&self.ram, BOMBOS_Y_COORD2));
                            self.ram[BOMBOS_X_LO + j] = x as u8;
                            self.ram[BOMBOS_X_HI + j] = ((x as u16) >> 8) as u8;
                            self.ram[BOMBOS_Y_LO + j] = y as u8;
                            self.ram[BOMBOS_Y_HI + j] = ((y as u16) >> 8) as u8;

                            let t = (x as u16)
                                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2))
                                .wrapping_add(8);
                            if t < 256 {
                                self.ram[SOUND_EFFECT_1] = K_BOMBOS_SFX[(t >> 5) as usize] | 0x2a;
                            }
                        }
                    } else {
                        i -= 1;
                        if i < 0 {
                            break;
                        }
                        continue;
                    }
                }
                self.ancilla_draw_bombos_fire_column(ui);
            }

            i -= 1;
            if i < 0 {
                break;
            }
        }
        if self.ram[BOMBOS_ARR7] >= 0x80 {
            self.ram[BOMBOS_VAR4] = 1;
        }
        self.ram[ANCILLA_STEP + k] = sb;
    }

    fn bombos_spell_finish_fire_columns(&mut self, kk: usize) {
        let mut k = self.ram[ANCILLA_STEP + kk] as i32;
        loop {
            let uk = k as usize;
            let arr1 = self.ram[BOMBOS_ARR1 + uk].wrapping_sub(1);
            self.ram[BOMBOS_ARR1 + uk] = arr1;
            if sign8(arr1) {
                self.ram[BOMBOS_ARR1 + uk] = 3;
                self.ram[BOMBOS_ARR2 + uk] = self.ram[BOMBOS_ARR2 + uk].wrapping_add(1);
                if self.ram[BOMBOS_ARR2 + uk] >= 13 {
                    self.ram[BOMBOS_ARR2 + uk] = 13;
                }
            }
            self.ancilla_draw_bombos_fire_column(uk);
            k -= 1;
            if k < 0 {
                break;
            }
        }
        for k in (0..=9).rev() {
            if self.ram[BOMBOS_ARR2 + k] != 13 {
                return;
            }
        }
        self.ram[BOMBOS_VAR4] = 2;
        self.medallion_check_sprite_damage(kk);
        self.ram[ANCILLA_STEP + kk] = 0;
    }

    fn bombos_spell_control_blasting(&mut self, kk: usize) {
        let mut k = self.ram[ANCILLA_STEP + kk] as i32;
        let mut sb = k;
        while k >= 0 {
            let uk = k as usize;
            if self.ram[BOMBOS_ARR3 + uk] != 8 {
                let arr4 = self.ram[BOMBOS_ARR4 + uk].wrapping_sub(1);
                self.ram[BOMBOS_ARR4 + uk] = arr4;
                if sign8(arr4) {
                    self.ram[BOMBOS_ARR4 + uk] = 3;
                    self.ram[BOMBOS_ARR3 + uk] = self.ram[BOMBOS_ARR3 + uk].wrapping_add(1);
                    if self.ram[BOMBOS_ARR3 + uk] == 1 && self.ram[BOMBOS_VAR2] == 0 {
                        let mut j = sb;
                        if j != 15 {
                            sb += 1;
                            j = sb;
                        } else {
                            while j >= 0 && self.ram[BOMBOS_ARR3 + j as usize] != 8 {
                                j -= 1;
                            }
                        }
                        let uj = j as usize;
                        self.ram[BOMBOS_ARR3 + uj] = 0;
                        self.ram[BOMBOS_ARR4 + uj] = 3;

                        let idx = (self.ram[FRAME_COUNTER] & 0x3f) as usize;
                        let y = u16::from(K_BOMBOS_BLASTS_TAB[idx]);
                        let x = u16::from(K_BOMBOS_BLASTS_TAB[idx + 3]);
                        let bg2vofs_copy2 = read_le_u16(&self.ram, BG2VOFS_COPY2);
                        let bg2hofs_copy2 = read_le_u16(&self.ram, BG2HOFS_COPY2);
                        write_le_u16(
                            &mut self.ram,
                            BOMBOS_Y_COORD + uj * 2,
                            y.wrapping_add(bg2vofs_copy2),
                        );
                        write_le_u16(
                            &mut self.ram,
                            BOMBOS_X_COORD + uj * 2,
                            x.wrapping_add(bg2hofs_copy2),
                        );
                        let bombos_x = read_le_u16(&self.ram, BOMBOS_X_COORD + uj * 2);
                        self.ram[SOUND_EFFECT_1] =
                            0x0c | K_BOMBOS_SFX[((bombos_x >> 5) & 7) as usize];
                    }
                }
            }
            self.ancilla_draw_bombos_blast(uk);
            k -= 1;
        }

        for j in (0..=15).rev() {
            if self.ram[BOMBOS_ARR3 + j] != 8 {
                self.ram[ANCILLA_STEP + kk] = sb as u8;
                let var3 = self.ram[BOMBOS_VAR3].wrapping_sub(1);
                self.ram[BOMBOS_VAR3] = var3;
                if var3 == 0 {
                    self.ram[BOMBOS_VAR3] = 1;
                    self.ram[BOMBOS_VAR2] = 1;
                }
                return;
            }
        }
        self.ram[ANCILLA_TYPE + kk] = 0;
        self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 1;
        self.ram[SPIN_ATTACK_SOUND_LATCH] = 0;
        self.ram[STATE_FOR_SPIN_ATTACK] = 0;
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[FLAG_UNK1] = 0;
        if self.ram[LINK_PLAYER_HANDLER_STATE] != 26 {
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
            self.ram[BUTTON_MASK_B_Y] = if self.ram[BUTTON_B_FRAMES] != 0 {
                self.ram[JOYPAD1H_LAST] & 0x80
            } else {
                0
            };
        }
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[MAGIC_SPELL_PLAYER_LOCK_FLAG] = 0;

        let var3 = self.ram[BOMBOS_VAR3].wrapping_sub(1);
        self.ram[BOMBOS_VAR3] = var3;
        if var3 == 0 {
            self.ram[BOMBOS_VAR3] = 1;
            self.ram[BOMBOS_VAR2] = 1;
        }
    }

    pub(super) fn ancilla_add_gt_cutscene(&mut self) {
        if (self.ram[LINK_STATE_BITS] & 0x80 | self.ram[LINK_AUXILIARY_STATE]) != 0
            || self.ram[LINK_HAS_CRYSTALS] & 0x7f != 0x7f
            || self.ram[SAVE_OW_EVENT_INFO_ANCILLA + 0x43] & 0x20 != 0
        {
            return;
        }

        self.ancilla_terminate_sparkle_objects_for_ancilla();

        if self.ancilla_add_check_for_presence(0x43) {
            return;
        }

        let Some(k) = self.ancilla_add_ancilla(0x43, 4) else {
            return;
        };

        for i in (0..=15).rev() {
            if self.ram[SPRITE_TYPE + i] == 0x37 {
                self.ram[SPRITE_STATE + i] = 0;
            }
        }

        for i in (0..=0x17).rev() {
            self.ram[BREAKTOWERSEAL_SPARKLE_VAR1 + i] = 0xff;
        }
        self.DecodeAnimatedSpriteTile_variable(0x28);
        self.ram[PALETTE_SP6R_INDOORS] = 4;
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x0200);
        self.palette_load_sprite_environment_dungeon();
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
        self.ram[ANCILLA_Y_SUBPIXEL + k] = 0;
        self.ram[ANCILLA_X_SUBPIXEL + k] = 0;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[BREAKTOWERSEAL_VAR5] = 240;
        self.ram[BREAKTOWERSEAL_VAR4] = 0;

        self.ram[BREAKTOWERSEAL_VAR3] = 0;
        self.ram[BREAKTOWERSEAL_VAR3 + 1] = 10;
        self.ram[BREAKTOWERSEAL_VAR3 + 2] = 22;
        self.ram[BREAKTOWERSEAL_VAR3 + 3] = 32;
        self.ram[BREAKTOWERSEAL_VAR3 + 4] = 42;
        self.ram[BREAKTOWERSEAL_VAR3 + 5] = 54;

        self.ancilla_set_xy(
            k,
            self.player_state_view().x(),
            self.player_state_view().y().wrapping_sub(16),
        );
    }

    fn ancilla_terminate_sparkle_objects_for_ancilla(&mut self) {
        for i in (0..=4).rev() {
            let t = self.ram[ANCILLA_TYPE + i];
            if t == 0x2a
                || t == 0x2b
                || t == 0x30
                || t == 0x31
                || t == 0x18
                || t == 0x19
                || t == 0x0c
            {
                self.ram[ANCILLA_TYPE + i] = 0;
            }
        }
    }

    pub(super) fn ancilla_add_blast_wall(&mut self) {
        const BLAST_WALL_TAB3: [i8; 4] = [-16, 16, 0, 0];
        const BLAST_WALL_TAB4: [i8; 4] = [0, 0, -16, 16];
        const BLAST_WALL_TAB5: [i8; 16] =
            [-8, 0, -8, 16, 16, 0, 16, 16, 0, -8, 16, -8, 0, 16, 16, 16];

        self.ram[ANCILLA_TYPE] = 0x33;
        self.ram[ANCILLA_TYPE + 1] = 0x33;
        self.ram[ANCILLA_TYPE + 2] = 0;
        self.ram[ANCILLA_TYPE + 3] = 0;
        self.ram[ANCILLA_TYPE + 4] = 0;
        self.ram[ANCILLA_TYPE + 5] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK] = 0;
        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[ANCILLA_K] = 0;
        self.ram[ANCILLA_FLOOR] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[ANCILLA_FLOOR + 1] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[ANCILLA_FLOOR2] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
        self.ram[BLASTWALL_VAR1] = 0;
        self.ram[BLASTWALL_VAR6 + 1] = 0;
        self.ram[BLASTWALL_VAR5 + 1] = 0;
        self.ram[BLASTWALL_VAR4] = 0;
        self.ram[BLASTWALL_VAR5] = 1;
        self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 1;
        self.ram[BLASTWALL_VAR6] = 3;

        let mut j = self.ram[BLASTWALL_VAR7] as usize;
        let blastwall_var8 =
            read_le_u16(&self.ram, BLASTWALL_VAR8).wrapping_add(BLAST_WALL_TAB3[j] as i16 as u16);
        let blastwall_var9 =
            read_le_u16(&self.ram, BLASTWALL_VAR9).wrapping_add(BLAST_WALL_TAB4[j] as i16 as u16);
        write_le_u16(&mut self.ram, BLASTWALL_VAR8, blastwall_var8);
        write_le_u16(&mut self.ram, BLASTWALL_VAR9, blastwall_var9);
        j = if j < 4 { 4 } else { 0 };
        for k in (0..=3).rev() {
            let y = blastwall_var8.wrapping_add(BLAST_WALL_TAB5[j * 2] as i16 as u16);
            let x = blastwall_var9.wrapping_add(BLAST_WALL_TAB5[j * 2 + 1] as i16 as u16);
            write_le_u16(&mut self.ram, BLASTWALL_VAR10 + k * 2, y);
            write_le_u16(&mut self.ram, BLASTWALL_VAR11 + k * 2, x);
            let x = x.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
            if x < 256 {
                self.ram[SOUND_EFFECT_1] = K_BOMBOS_SFX[(x >> 5) as usize] | 0x0c;
            }
            j += 1;
        }
    }

    pub(super) fn add_bird_travel_something(&mut self, a: u8, y: u8) {
        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_simple(a, y) {
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            self.ram[BUTTON_MASK_B_Y] &= !0x81;
            self.ram[BUTTON_B_FRAMES] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            self.ram[ANCILLA_L + k] = 1;

            if self.read_u32_ram(ENHANCED_FEATURES0) & 1 != 0 {
                self.ram[ANCILLA_Z_VEL + k] = 58;
                self.ram[ANCILLA_Z + k] = (-105i8) as u8;
            } else {
                self.ram[ANCILLA_Z_VEL + k] = 40;
                self.ram[ANCILLA_Z + k] = (-51i8) as u8;
            }
            self.ram[ANCILLA_STEP + k] = 2;
            self.add_bird_common(k);
        }
    }

    fn ancilla_add_check_for_presence(&self, a: u8) -> bool {
        (0..=5).rev().any(|k| self.ram[ANCILLA_TYPE + k] == a)
    }

    pub(super) fn add_happiness_pond_rupees(&mut self, arg: u8) {
        let Some(_) = self.ancilla_add_simple(0x42, 9) else {
            return;
        };
        self.ram[SOUND_EFFECT_2] = self.link_calculate_sfx_pan() | 0x13;
        self.DecodeAnimatedSpriteTile_variable(0x24);
        self.ram[LINK_STATE_BITS] = 0x80;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_DIRECTION_FACING] = 0;
        self.ram[LINK_ANIMATION_STEPS] = 0;

        for i in 0..10 {
            self.ram[HAPPINESS_POND_ARR1 + i] = 0;
        }

        const HAPPINESS_POND_START: [i8; 4] = [0, 4, 4, 9];
        const HAPPINESS_POND_END: [i8; 4] = [-1, 0, -1, -1];
        const HAPPINESS_POND_XVEL: [i8; 10] = [0, -12, -6, 6, 12, -9, -5, 0, 5, 9];
        const HAPPINESS_POND_YVEL: [i8; 10] = [-40, -40, -40, -40, -40, -32, -32, -32, -32, -32];
        const HAPPINESS_POND_ZVEL: [i8; 10] = [20, 20, 20, 20, 20, 16, 16, 16, 16, 16];

        let mut j = HAPPINESS_POND_START[arg as usize];
        let j_end = HAPPINESS_POND_END[arg as usize];
        let mut k = 9usize;
        loop {
            self.ram[HAPPINESS_POND_ARR1 + k] = 1;
            self.ram[HAPPINESS_POND_Z_VEL + k] = HAPPINESS_POND_ZVEL[j as usize] as u8;
            self.ram[HAPPINESS_POND_Y_VEL + k] = HAPPINESS_POND_YVEL[j as usize] as u8;
            self.ram[HAPPINESS_POND_X_VEL + k] = HAPPINESS_POND_XVEL[j as usize] as u8;
            self.ram[HAPPINESS_POND_Z + k] = 0;
            self.ram[HAPPINESS_POND_STEP + k] = 0;
            self.ram[HAPPINESS_POND_TIMER + k] = 16;
            self.ram[HAPPINESS_POND_ITEM_TO_LINK + k] = 53;
            let x = self.player_state_view().x().wrapping_add(4);
            let y = self.player_state_view().y().wrapping_sub(12);
            self.ram[HAPPINESS_POND_X_LO + k] = x as u8;
            self.ram[HAPPINESS_POND_X_HI + k] = (x >> 8) as u8;
            self.ram[HAPPINESS_POND_Y_LO + k] = y as u8;
            self.ram[HAPPINESS_POND_Y_HI + k] = (y >> 8) as u8;
            if k == 0 {
                break;
            }
            k -= 1;
            j -= 1;
            if j == j_end {
                break;
            }
        }
    }

    pub(super) fn ancilla_add_snoring(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_simple(a, y) else {
            return;
        };
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_Y_VEL + k] = (-8i8) as u8;
        self.ram[ANCILLA_AUX_TIMER + k] = 7;
        self.ram[ANCILLA_X_VEL + k] = 8;
        self.ram[ANCILLA_STEP + k] = 255;
        self.ancilla_set_xy(
            k,
            self.player_state_view().x().wrapping_add(16),
            self.player_state_view().y().wrapping_add(4),
        );
    }

    pub(super) fn ancilla_add_bomb(&mut self, a: u8, y: u8) {
        const BOMB_PLACE_X0: [i8; 4] = [8, 8, 0, 16];
        const BOMB_PLACE_Y0: [i8; 4] = [0, 24, 12, 12];
        const BOMB_PLACE_X1: [i8; 4] = [8, 8, -6, 22];
        const BOMB_PLACE_Y1: [i8; 4] = [4, 28, 12, 12];

        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return;
        };
        if self.ram[LINK_ITEM_BOMBS] == 0 {
            self.ram[ANCILLA_TYPE + k] = 0;
            return;
        }

        self.ram[LINK_ITEM_BOMBS] = self.ram[LINK_ITEM_BOMBS].wrapping_sub(1);
        if self.ram[LINK_ITEM_BOMBS] == 0 {
            self.hud_refresh_icon();
        }

        self.ram[ANCILLA_R_PLAYER + k] = 0;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_L + k] = 0;
        self.ram[ANCILLA_ARR3 + k] = K_BOMB_TAB0[0];
        self.ram[ANCILLA_ARR25 + k] = 0;
        self.ram[ANCILLA_ARR26 + k] = 7;
        self.ram[ANCILLA_Z + k] = 0;
        self.ram[ANCILLA_TIMER + k] = 8;
        self.ram[ANCILLA_DIR + k] = self.ram[LINK_DIRECTION_FACING] >> 1;
        self.ram[ANCILLA_T_PLAYER + k] = 0;
        self.ram[ANCILLA_ARR23 + k] = 0;
        self.ram[ANCILLA_ARR22 + k] = 0;
        let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        if self.ancilla_check_initial_tile_collision_class2(k) {
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(BOMB_PLACE_X0[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(BOMB_PLACE_Y0[j] as i16 as u16),
            );
        } else {
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(BOMB_PLACE_X1[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(BOMB_PLACE_Y1[j] as i16 as u16),
            );
        }
        self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 0x0b;
    }

    pub(super) fn ancilla_add_boomerang(&mut self, a: u8, y: u8) -> u8 {
        const BOOMERANG_TAB0: [u8; 4] = [0x20, 0x18, 0x30, 0x28];
        const BOOMERANG_TAB1: [u8; 2] = [0x20, 0x60];
        const BOOMERANG_TAB2: [u8; 2] = [3, 2];
        const BOOMERANG_TAB3: [u8; 4] = [8, 4, 2, 1];
        const BOOMERANG_TAB4: [u8; 8] = [8, 4, 2, 1, 9, 5, 10, 6];
        const BOOMERANG_TAB5: [u8; 8] = [2, 3, 3, 2, 2, 3, 3, 3];
        const BOOMERANG_TAB6: [i8; 8] = [-10, -8, -9, -9, -10, -8, -9, -9];
        const BOOMERANG_TAB7: [i8; 8] = [-10, 11, 8, -8, -10, 11, 8, -8];
        const BOOMERANG_TAB8: [i8; 8] = [-16, 6, 0, 0, -8, 8, -8, 8];
        const BOOMERANG_TAB9: [i8; 8] = [0, 0, -8, 8, 8, 8, -8, -8];

        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return 0;
        };
        self.ram[ANCILLA_AUX_TIMER + k] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_K + k] = 0;
        self.ram[ANCILLA_Z + k] = 0;
        self.ram[ANCILLA_L + k] = self.ram[ANCILLA_NUMSPR + k];
        self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 1;
        let mut j = self.ram[LINK_ITEM_BOOMERANG].wrapping_sub(1) as usize;
        self.ram[ANCILLA_G + k] = j as u8;
        self.ram[ANCILLA_STEP + k] = BOOMERANG_TAB1[j];
        self.ram[ANCILLA_ARR3 + k] = BOOMERANG_TAB2[j];

        let s = self.ram[ANCILLA_G + k] as usize * 2
            + if self.ram[JOYPAD1H_LAST] & 0x0c != 0 && self.ram[JOYPAD1H_LAST] & 3 != 0 {
                1
            } else {
                0
            };
        let r0 = BOOMERANG_TAB0[s];
        self.ram[ANCILLA_H + k] = r0;

        let r1 = if self.ram[JOYPAD1H_LAST] & 0x0f != 0 {
            self.ram[JOYPAD1H_LAST] & 0x0f
        } else {
            BOOMERANG_TAB3[(self.ram[LINK_DIRECTION_FACING] >> 1) as usize]
        };
        self.ram[HOOKSHOT_EFFECT_INDEX] = 0;

        if r1 & 0x0c != 0 {
            self.ram[ANCILLA_Y_VEL + k] = if r1 & 8 != 0 { (-(r0 as i8)) as u8 } else { r0 };
            let i = if sign8(self.ram[ANCILLA_Y_VEL + k]) {
                0
            } else {
                1
            };
            self.ram[ANCILLA_DIR + k] = i;
            self.ram[HOOKSHOT_EFFECT_INDEX] = BOOMERANG_TAB3[i as usize];
        }
        self.ram[ANCILLA_S_PLAYER + k] = 0;

        if r1 & 3 != 0 {
            if r1 & 2 == 0 {
                self.ram[ANCILLA_S_PLAYER + k] = 1;
            }
            self.ram[ANCILLA_X_VEL + k] = if r1 & 2 != 0 { (-(r0 as i8)) as u8 } else { r0 };
            let i = if sign8(self.ram[ANCILLA_X_VEL + k]) {
                2
            } else {
                3
            };
            self.ram[ANCILLA_DIR + k] = i;
            self.ram[HOOKSHOT_EFFECT_INDEX] |= BOOMERANG_TAB3[i as usize];
        }

        j = BOOMERANG_TAB4.iter().position(|&v| v == r1).unwrap_or(0);
        self.ram[ANCILLA_ARR1 + k] = BOOMERANG_TAB5[j];
        self.ram[ANCILLA_ARR23 + k] = (j << 1) as u8;
        if self.ram[BUTTON_B_FRAMES] >= 9 {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_add(1);
        } else if s != 0 || self.ram[JOYPAD1H_LAST] & 0x0f == 0 {
            j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        }

        let s = self.ancilla_check_initial_tile_a(k);
        if s < 0 {
            if self.ram[ANCILLA_AUX_TIMER + k] != 0 {
                self.ancilla_set_xy(
                    k,
                    self.player_state_view()
                        .x()
                        .wrapping_add(BOOMERANG_TAB9[j] as i16 as u16),
                    self.player_state_view()
                        .y()
                        .wrapping_add(8)
                        .wrapping_add(BOOMERANG_TAB8[j] as i16 as u16),
                );
            } else {
                self.ancilla_set_xy(
                    k,
                    self.player_state_view()
                        .x()
                        .wrapping_add(BOOMERANG_TAB7[j] as i16 as u16),
                    self.player_state_view()
                        .y()
                        .wrapping_add(8)
                        .wrapping_add(BOOMERANG_TAB6[j] as i16 as u16),
                );
            }
        } else {
            self.ram[ANCILLA_TYPE + k] = 0;
            self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 0;
            self.ram[SOUND_EFFECT_1] = self.ancilla_calculate_sfx_pan(k)
                | if self.ram[ANCILLA_TILE_ATTR_PLAYER + k] != 0xf0 {
                    5
                } else {
                    6
                };
            self.ancilla_add_boomerang_wall_clink(k);
        }
        if std::env::var_os("ZELDA3_TRACE_BOOMERANG").is_some()
            && k == 4
            && self.ram[FRAME_COUNTER] >= 140
            && self.ram[FRAME_COUNTER] <= 210
        {
            eprintln!(
                "R boomerang-add fc={} k={} s={} type=0x{:02x} x={:04x} y={:04x} xv={:02x} yv={:02x} step={:02x} aux={:02x} item={:02x} K={:02x} dir={:02x} arr23={:02x} link={:04x}/{:04x} joy={:02x} bframes={:02x}",
                self.ram[FRAME_COUNTER],
                k,
                s,
                self.ram[ANCILLA_TYPE + k],
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                self.ram[ANCILLA_X_VEL + k],
                self.ram[ANCILLA_Y_VEL + k],
                self.ram[ANCILLA_STEP + k],
                self.ram[ANCILLA_AUX_TIMER + k],
                self.ram[ANCILLA_ITEM_TO_LINK + k],
                self.ram[ANCILLA_K + k],
                self.ram[ANCILLA_DIR + k],
                self.ram[ANCILLA_ARR23 + k],
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.ram[JOYPAD1H_LAST],
                self.ram[BUTTON_B_FRAMES],
            );
        }
        s as u8
    }

    pub(super) fn ancilla_add_tossed_pond_item(&mut self, a: u8, xin: u8, yin: u8) {
        const WISH_POND_ITEM_X: [u8; 76] = [
            4, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 4, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 2, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 0, 4, 0, 0, 0, 4, 0, 0,
        ];
        const WISH_POND_ITEM_Y: [i8; 76] = [
            -13, -13, -13, -13, -13, -12, -12, -13, -13, -12, -12, -12, -10, -12, -12, -12, -12,
            -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12,
            -12, -13, -12, -12, -12, -12, -12, -12, -10, -12, -12, -12, -12, -12, -12, -12, -12,
            -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12,
            -12, -12, -12, -12, -12, -13, -12, -12,
        ];

        self.ram[LINK_RECEIVEITEM_INDEX] = xin;
        if let Some(k) = self.ancilla_add_ancilla(a, yin) {
            self.ram[SOUND_EFFECT_2] = self.link_calculate_sfx_pan() | 0x13;
            let sb = K_RECEIVE_ITEM_GFX[xin as usize];
            if sb != 0xff {
                if sb == 0x20 {
                    self.DecompressShieldGraphics();
                }
                self.DecodeAnimatedSpriteTile_variable(sb);
            } else {
                self.DecodeAnimatedSpriteTile_variable(0);
            }
            if sb == 6 {
                self.DecompressSwordGraphics();
            }

            self.ram[LINK_STATE_BITS] = 0x80;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_DIRECTION_FACING] = 0;
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[ANCILLA_Z_VEL + k] = 20;
            self.ram[ANCILLA_Y_VEL + k] = (-40i8) as u8;
            self.ram[ANCILLA_X_VEL + k] = 0;
            self.ram[ANCILLA_Z + k] = 0;
            self.ram[ANCILLA_TIMER + k] = 16;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[LINK_RECEIVEITEM_INDEX];
            self.ancilla_set_xy(
                k,
                self.player_state_view().x().wrapping_add(
                    WISH_POND_ITEM_X[self.ram[LINK_RECEIVEITEM_INDEX] as usize] as u16,
                ),
                self.player_state_view().y().wrapping_add(
                    WISH_POND_ITEM_Y[self.ram[LINK_RECEIVEITEM_INDEX] as usize] as i16 as u16,
                ),
            );
        }
    }

    fn ancilla_add_cutscene_duck(&mut self, a: u8, y: u8) {
        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ram[ANCILLA_DIR + k] = 2;
            self.ram[ANCILLA_ARR3 + k] = 3;
            self.ram[ANCILLA_STEP + k] = 0;
            self.ram[ANCILLA_AUX_TIMER + k] = 32;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 116;
            self.ram[ANCILLA_Z_VEL + k] = 0;
            self.ram[ANCILLA_L + k] = 0;
            self.ram[ANCILLA_Z + k] = 0;
            self.ram[ANCILLA_S_PLAYER + k] = 0;
            self.ancilla_set_xy(k, 0x0200, 0x0788);
        }
    }

    pub(super) fn ancilla_add_exploding_weather_vane(&mut self, a: u8, y: u8) {
        const K_WEATHERVANE_TAB4: [i8; 12] = [8, 10, 9, 4, 11, 12, -10, -8, 4, -6, -10, -4];
        const K_WEATHERVANE_TAB5: [i8; 12] = [20, 22, 20, 20, 22, 20, 20, 22, 20, 22, 20, 20];
        const K_WEATHERVANE_TAB6: [u8; 12] = [
            0xb0, 0xa3, 0xa0, 0xa2, 0xa0, 0xa8, 0xa0, 0xa0, 0xa8, 0xa1, 0xb0, 0xa0,
        ];
        const K_WEATHERVANE_TAB8: [u8; 12] = [0, 2, 4, 6, 3, 8, 14, 8, 12, 7, 10, 8];
        const K_WEATHERVANE_TAB10: [u8; 12] = [48, 18, 32, 20, 22, 24, 32, 20, 24, 22, 20, 32];

        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return;
        };

        self.ram[ANCILLA_AUX_TIMER + k] = 10;
        self.ram[ANCILLA_G + k] = 128;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_ARR3 + k] = 0;
        self.ram[SOUND_EFFECT_1] = 0;
        self.ram[MUSIC_CONTROL] = 0xf2;
        self.ram[SOUND_EFFECT_AMBIENT] = 0x17;

        self.ram[WEATHERVANE_VAR1] = 0;
        write_le_u16(&mut self.ram, WEATHERVANE_VAR2, 0x0280);

        for i in (0..=11).rev() {
            self.ram[WEATHERVANE_ARR3 + i] = 0;
            self.ram[WEATHERVANE_ARR4 + i] = K_WEATHERVANE_TAB4[i] as u8;
            self.ram[WEATHERVANE_ARR5 + i] = K_WEATHERVANE_TAB5[i] as u8;
            self.ram[WEATHERVANE_ARR6 + i] = K_WEATHERVANE_TAB6[i];
            self.ram[WEATHERVANE_ARR7 + i] = 7;
            self.ram[WEATHERVANE_ARR8 + i] = K_WEATHERVANE_TAB8[i];
            self.ram[WEATHERVANE_ARR9 + i] = 2;
            self.ram[WEATHERVANE_ARR10 + i] = K_WEATHERVANE_TAB10[i];
            self.ram[WEATHERVANE_ARR11 + i] = 1;
            self.ram[WEATHERVANE_ARR12 + i] = (i & 1) as u8;
        }
    }

    fn ancilla_add_super_bomb_explosion(&mut self, a: u8, y: u8) -> i32 {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return -1;
        };
        self.ram[ANCILLA_R_PLAYER + k] = 0;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_ARR25 + k] = 0;
        self.ram[ANCILLA_L + k] = 0;
        self.ram[ANCILLA_ARR3 + k] = K_BOMB_TAB0[1];
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 1;
        let j = self.ram[TAGALONG_DATA_INDEX_ANCILLA] as usize;
        let y = self.ram[TAGALONG_Y_LO_ANCILLA + j] as u16
            | ((self.ram[TAGALONG_Y_HI_ANCILLA + j] as u16) << 8);
        let x = self.ram[TAGALONG_X_LO_ANCILLA + j] as u16
            | ((self.ram[TAGALONG_X_HI_ANCILLA + j] as u16) << 8);
        self.ancilla_set_xy(k, x.wrapping_add(8), y.wrapping_add(16));
        k as i32
    }

    pub(super) fn ancilla_add_somaria_block(&mut self, ty: u8, y: u8) -> Option<usize> {
        let k = self.ancilla_add_add_ancilla_bank08(ty, y)?;
        for j in (0..=4).rev() {
            if j == k || self.ram[ANCILLA_TYPE + j] != 0x2c {
                continue;
            }
            if j == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP].wrapping_sub(1) as usize {
                self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
            }
            self.ancilla_add_exploding_somaria_block(j);
            self.ram[ANCILLA_TYPE + k] = 0;
            self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER] = 0;
            if self.ram[LINK_SPEED_SETTING] == 0x12 {
                self.ram[PLAYER_DEFENSE_FLAGS] = 0;
                self.ram[LINK_SPEED_SETTING] = 0;
            }
            return Some(k);
        }

        self.ancilla_sfx3_near(0x2a);
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_Y_VEL + k] = 0;
        self.ram[ANCILLA_X_VEL + k] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_AUX_TIMER + k] = 0;
        self.ram[ANCILLA_ARR3 + k] = 0;
        self.ram[ANCILLA_ARR1 + k] = 0;
        self.ram[ANCILLA_H + k] = 0;
        self.ram[ANCILLA_G + k] = 12;
        self.ram[ANCILLA_TIMER + k] = 18;
        self.ram[ANCILLA_L + k] = 0;
        self.ram[ANCILLA_Z + k] = 0;
        self.ram[ANCILLA_K + k] = 0;
        self.ram[ANCILLA_R_PLAYER + k] = 0;
        self.ram[ANCILLA_ARR4 + k] = 0;
        self.ram[ANCILLA_S_PLAYER + k] = 9;
        self.ram[ANCILLA_T_PLAYER + k] = 0;
        self.ram[ANCILLA_DIR + k] = self.ram[LINK_DIRECTION_FACING] >> 1;
        if self.ancilla_check_initial_tile_collision_class2(k) {
            self.ancilla_set_xy(
                k,
                self.player_state_view().x().wrapping_add(8),
                self.player_state_view().y().wrapping_add(16),
            );
        } else {
            const CANE_OF_SOMARIA_Y: [i8; 4] = [-8, 31, 17, 17];
            const CANE_OF_SOMARIA_X: [i8; 4] = [8, 8, -8, 23];
            let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(CANE_OF_SOMARIA_X[j] as i16 as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(CANE_OF_SOMARIA_Y[j] as i16 as u16),
            );
            self.somaria_block_check_for_transit_tile(k);
        }
        Some(k)
    }

    pub(super) fn ancilla_get_y(&self, k: usize) -> u16 {
        u16::from(self.ram[ANCILLA_Y_LO + k]) | (u16::from(self.ram[ANCILLA_Y_HI + k]) << 8)
    }

    pub(super) fn ancilla_get_x(&self, k: usize) -> u16 {
        u16::from(self.ram[ANCILLA_X_LO + k]) | (u16::from(self.ram[ANCILLA_X_HI + k]) << 8)
    }

    fn ancilla_project_reflexive_speed_onto_sprite(
        &mut self,
        k: usize,
        x: u16,
        y: u16,
        vel: u8,
    ) -> ProjectSpeedRet {
        let old_x = self.player_state_view().x();
        let old_y = self.player_state_view().y();
        self.player_state_view_mut().set_x(x);
        self.player_state_view_mut().set_y(y);
        let pt = self.sprite_project_speed_towards_link(k, vel);
        self.player_state_view_mut().set_x(old_x);
        self.player_state_view_mut().set_y(old_y);
        pt
    }

    pub(super) fn ancilla_terminate_select_interactives(&mut self, mut y: u8) -> u8 {
        for i in (0..=5).rev() {
            if self.ram[ANCILLA_TYPE + i] == 0x3e {
                y = i as u8;
            } else if self.ram[ANCILLA_TYPE + i] == 0x2c {
                self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER] = 0;
                if self.ram[PLAYER_DEFENSE_FLAGS] & 0x80 != 0 {
                    self.ram[PLAYER_DEFENSE_FLAGS] = 0;
                    self.ram[LINK_SPEED_SETTING] = 0;
                }
            }

            if sign8(self.ram[LINK_STATE_BITS]) {
                if i + 1 != self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
                    self.ram[ANCILLA_TYPE + i] = 0;
                }
            } else {
                if i + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
                    self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                }
                self.ram[ANCILLA_TYPE + i] = 0;
            }
        }

        if self.ram[LINK_POSITION_MODE] & 0x10 != 0 {
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
            self.ram[LINK_POSITION_MODE] = 0;
        }
        self.ram[FLUTE_COUNTDOWN] = 0;
        self.ram[TAGALONG_EVENT_FLAGS] = 0;
        self.ram[ANCILLA_INTERACTIVE_RESET_FLAG] = 0;
        self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 0;
        self.ram[IS_ARCHER_OR_SHOVEL_GAME] = 0;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ram[PLAYER_SPECIAL_DRAW_FLAG] = 0;
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        if self.ram[LINK_PLAYER_HANDLER_STATE] == 19 {
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.ram[BUTTON_MASK_B_Y] &= !0x40;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            self.ram[LINK_POSITION_MODE] &= !4;
            self.ram[RELATED_TO_HOOKSHOT] = 0;
        }
        y
    }

    pub(super) fn ancilla_allocate_oam_from_region_a_or_d_or_f(
        &mut self,
        k: usize,
        size: u8,
    ) -> u16 {
        if self.ram[SORT_SPRITES_SETTING] != 0 {
            if self.ram[ANCILLA_FLOOR + k] != 0 {
                self.oam_allocate_from_region_f(size)
            } else {
                self.oam_allocate_from_region_d(size)
            }
        } else {
            self.oam_allocate_from_region_a(size)
        }
    }

    pub(super) fn ancilla_prep_oam_coord(&mut self, k: usize) -> (u16, u16) {
        const TAGALONG_LAYER_BITS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
        let floor = self.ram[ANCILLA_FLOOR + k] as usize;
        write_le_u16(
            &mut self.ram,
            OAM_PRIORITY_VALUE,
            (TAGALONG_LAYER_BITS[floor] as u16) << 8,
        );
        (
            self.ancilla_x(k)
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)),
            self.ancilla_y(k)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)),
        )
    }

    pub(super) fn ancilla_move_x(&mut self, k: usize) {
        let pos = self.ram[ANCILLA_X_SUBPIXEL + k] as u32
            | ((self.ram[ANCILLA_X_LO + k] as u32) << 8)
            | ((self.ram[ANCILLA_X_HI + k] as u32) << 16);
        let delta = ((self.ram[ANCILLA_X_VEL + k] as i8 as i32) << 4) as u32;
        let moved = pos.wrapping_add(delta);
        self.ram[ANCILLA_X_SUBPIXEL + k] = moved as u8;
        self.ram[ANCILLA_X_LO + k] = (moved >> 8) as u8;
        self.ram[ANCILLA_X_HI + k] = (moved >> 16) as u8;
    }

    pub(super) fn ancilla_move_y(&mut self, k: usize) {
        let pos = self.ram[ANCILLA_Y_SUBPIXEL + k] as u32
            | ((self.ram[ANCILLA_Y_LO + k] as u32) << 8)
            | ((self.ram[ANCILLA_Y_HI + k] as u32) << 16);
        let delta = ((self.ram[ANCILLA_Y_VEL + k] as i8 as i32) << 4) as u32;
        let moved = pos.wrapping_add(delta);
        self.ram[ANCILLA_Y_SUBPIXEL + k] = moved as u8;
        self.ram[ANCILLA_Y_LO + k] = (moved >> 8) as u8;
        self.ram[ANCILLA_Y_HI + k] = (moved >> 16) as u8;
    }

    pub(super) fn ancilla_move_z(&mut self, k: usize) {
        let pos = self.ram[ANCILLA_Z_SUBPIXEL_PLAYER + k] as u32
            | ((self.ram[ANCILLA_Z + k] as u32) << 8);
        let delta = ((self.ram[ANCILLA_Z_VEL + k] as i8 as i32) << 4) as u32;
        let moved = pos.wrapping_add(delta);
        self.ram[ANCILLA_Z_SUBPIXEL_PLAYER + k] = moved as u8;
        self.ram[ANCILLA_Z + k] = (moved >> 8) as u8;
    }

    fn ancilla02_fire_rod_shot(&mut self, k: usize) {
        if self.ram[ANCILLA_STEP + k] == 0 {
            if self.frame_control_view().submodule() == 0 {
                self.ram[ANCILLA_L + k] = 0;
                self.ancilla_move_x(k);
                self.ancilla_move_y(k);
                let mut coll = self.ancilla_check_sprite_collision(k).is_some();
                if !coll {
                    self.ram[ANCILLA_DIR + k] |= 8;
                    coll = self.ancilla_check_tile_collision(k) != 0;
                    self.ram[ANCILLA_L + k] = self.ram[ANCILLA_TILE_ATTR_PLAYER + k];
                    if !coll {
                        self.ram[ANCILLA_DIR + k] |= 12;
                        let bak = self.ram[ANCILLA_U + k];
                        coll = self.ancilla_check_tile_collision(k) != 0;
                        self.ram[ANCILLA_U + k] = bak;
                    }
                }
                if coll {
                    self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                    self.ram[ANCILLA_TIMER + k] = 31;
                    self.ram[ANCILLA_NUMSPR + k] = 8;
                    self.ancilla_sfx2_pan(k, 0x2a);
                }
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                self.ram[ANCILLA_DIR + k] &= !0x0c;
                self.ram[DUNGEON_TORCH_ATTR] = self.ram[ANCILLA_L + k];
                if self.ram[DUNGEON_TORCH_ATTR] & 0xf0 == 0xc0 {
                    self.dungeon_light_torch();
                } else {
                    self.ram[DUNGEON_TORCH_ATTR] = self.ram[ANCILLA_TILE_ATTR_PLAYER + k];
                    if self.ram[DUNGEON_TORCH_ATTR] & 0xf0 == 0xc0 {
                        self.dungeon_light_torch();
                    }
                }
            }
            self.fire_shot_draw(k);
        } else {
            self.ancilla_check_basic_sprite_collision(k);
            let Some(info) = self.ancilla_return_if_outside_bounds(k) else {
                return;
            };
            let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
            if self.ram[ANCILLA_TIMER + k] == 0 {
                let old_type = self.ram[ANCILLA_TYPE + k];
                self.ram[ANCILLA_TYPE + k] = 0;
                if old_type != 0x2f
                    && self.ram[OVERWORLD_SCREEN_INDEX] == 64
                    && self.ram[ANCILLA_TILE_ATTR_PLAYER + k] == 0x43
                {
                    self.fire_rod_shot_become_skull_woods_fire(k);
                }
                return;
            }
            let j = self.ram[ANCILLA_TIMER + k] >> 3;
            if j != 0 {
                const FIRE_SHOT_DRAW_CHAR: [u8; 3] = [0xa2, 0xa0, 0x8e];
                self.ancilla_set_oam_plain(
                    oam,
                    info.x as u16,
                    info.y as u16,
                    FIRE_SHOT_DRAW_CHAR[j as usize - 1],
                    info.flags | 2,
                    2,
                );
            } else {
                self.ancilla_set_oam_plain(
                    oam,
                    info.x as u16,
                    info.y.wrapping_sub(3) as u16,
                    0xa4,
                    info.flags | 2,
                    0,
                );
                self.ancilla_set_oam_plain(
                    oam + 4,
                    info.x.wrapping_add(8) as u16,
                    info.y.wrapping_sub(3) as u16,
                    0xa5,
                    info.flags | 2,
                    0,
                );
            }
        }
    }

    fn fire_rod_shot_become_skull_woods_fire(&mut self, _k: usize) {
        if self.ram[PLAYER_IS_INDOORS] != 0 || self.ram[OVERWORLD_SCREEN_INDEX] & 0x40 == 0 {
            return;
        }

        self.ram[ANCILLA_TYPE] = 0x34;
        for i in 1..=5 {
            self.ram[ANCILLA_TYPE + i] = 0;
        }
        self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 0;
        self.ram[ANCILLA_NUMSPR] = K_ANCILLA_PFLAGS[0x34];
        self.ram[SKULLWOODSFIRE_VAR0] = 253;
        self.ram[SKULLWOODSFIRE_VAR0 + 1] = 254;
        self.ram[SKULLWOODSFIRE_VAR0 + 2] = 255;
        self.ram[SKULLWOODSFIRE_VAR0 + 3] = 0;
        self.ram[SKULLWOODSFIRE_VAR4] = 0;
        for i in 0..4 {
            self.ram[SKULLWOODSFIRE_VAR5 + i] = 5;
        }
        self.ram[ANCILLA_AUX_TIMER] = 5;
        write_le_u16(&mut self.ram, SKULLWOODSFIRE_VAR9, 0x0100);
        write_le_u16(&mut self.ram, SKULLWOODSFIRE_VAR10, 0x0100);
        write_le_u16(&mut self.ram, SKULLWOODSFIRE_VAR11, 0x0098);
        write_le_u16(&mut self.ram, SKULLWOODSFIRE_VAR12, 0x0098);
        self.ram[TRIGGER_SPECIAL_ENTRANCE_ANCILLA] = 2;
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[R16] = 0;
        self.ram[ANCILLA_FLOOR] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[ANCILLA_FLOOR2] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
        self.ram[ANCILLA_ITEM_TO_LINK] = 0;
        self.ram[ANCILLA_STEP] = 0;
    }

    fn ancilla0_b_ice_rod_shot(&mut self, k: usize) {
        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                if self.ram[ANCILLA_ITEM_TO_LINK + k] & !1 != 0 {
                    self.ram[ANCILLA_STEP + k] = 1;
                    self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k] & 7 | 4;
                }
                self.ram[ANCILLA_AUX_TIMER + k] = 3;
            }
            if self.ram[ANCILLA_STEP + k] != 0 {
                if self.ancilla_return_if_outside_bounds(k).is_none() {
                    return;
                }
                self.ancilla_move_y(k);
                self.ancilla_move_x(k);
                if self.ancilla_check_sprite_collision(k).is_some()
                    || self.ancilla_check_tile_collision(k) != 0
                {
                    self.ram[ANCILLA_TYPE + k] = 0x11;
                    self.ram[ANCILLA_NUMSPR + k] = K_ANCILLA_PFLAGS[0x11];
                    self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
                    self.ram[ANCILLA_AUX_TIMER + k] = 4;
                }
            }
        }
        self.ancilla_add_ice_rod_sparkle(k);
    }

    fn ancilla09_arrow(&mut self, k: usize) {
        const ARROW_Y: [i8; 4] = [-4, 2, 0, 0];
        const ARROW_X: [i8; 4] = [0, 0, -4, 4];

        if self.frame_control_view().submodule() != 0 {
            self.arrow_draw(k);
            return;
        }

        self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_sub(1);
        if !sign8(self.ram[ANCILLA_ITEM_TO_LINK + k]) {
            if self.ram[ANCILLA_ITEM_TO_LINK + k] >= 4 {
                return;
            }
        } else {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0xff;
        }
        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
        if self.ram[LINK_ITEM_BOW] & 4 != 0 && self.ram[FRAME_COUNTER] & 1 == 0 {
            self.ancilla_add_silver_arrow_sparkle(k);
        }
        self.ram[ANCILLA_S_PLAYER + k] = 255;
        let j;
        if let Some(sprite) = self.ancilla_check_sprite_collision(k) {
            j = sprite;
            self.ram[ANCILLA_X_VEL + k] =
                self.ram[ANCILLA_X_LO + k].wrapping_sub(self.ram[SPRITE_X_LO + sprite]);
            self.ram[ANCILLA_Y_VEL + k] = self.ram[ANCILLA_Y_LO + k]
                .wrapping_sub(self.ram[SPRITE_Y_LO + sprite])
                .wrapping_add(self.ram[SPRITE_Z + sprite]);
            self.ram[ANCILLA_S_PLAYER + k] = sprite as u8;
            if self.ram[SPRITE_TYPE + sprite] == 0x65 {
                if self.ram[SPRITE_A + sprite] == 1 {
                    self.ram[SOUND_EFFECT_2] = 0x2d;
                    self.ram[SPRITE_DELAY_AUX2_ANCILLA + sprite] = 0x80;
                    self.ram[SPRITE_DELAY_AUX4] = 128;
                    if self.ram[ARCHERY_GAME_HIT_COUNTER] < 9 {
                        self.ram[ARCHERY_GAME_HIT_COUNTER] =
                            self.ram[ARCHERY_GAME_HIT_COUNTER].wrapping_add(1);
                    }
                    self.ram[SPRITE_B_ANCILLA + sprite] = self.ram[ARCHERY_GAME_HIT_COUNTER];
                    self.ram[SPRITE_G_ANCILLA + sprite] =
                        self.ram[SPRITE_G_ANCILLA + sprite].wrapping_add(1);
                } else {
                    self.ram[SPRITE_DELAY_AUX3_ANCILLA + sprite] = 4;
                    self.ram[ARCHERY_GAME_HIT_COUNTER] = 0;
                }
            } else {
                self.ram[ARCHERY_GAME_HIT_COUNTER] = 0;
            }
        } else {
            let coll = self.ancilla_check_tile_collision(k);
            if coll != 0 {
                self.ram[ANCILLA_H + k] = coll >> 1;
                let dir = (self.ram[ANCILLA_DIR + k] & 3) as usize;
                self.ancilla_set_xy(
                    k,
                    self.ancilla_get_x(k)
                        .wrapping_add(ARROW_X[dir] as i16 as u16),
                    self.ancilla_get_y(k)
                        .wrapping_add(ARROW_Y[dir] as i16 as u16),
                );
                self.ram[ARCHERY_GAME_HIT_COUNTER] = 0;
                j = dir;
            } else {
                self.arrow_draw(k);
                return;
            }
        }
        if self.ram[SPRITE_TYPE + j] != 0x1b {
            self.ancilla_sfx2_pan(k, 8);
        }
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_TYPE + k] = 10;
        self.ram[ANCILLA_AUX_TIMER + k] = 1;
        if self.ram[ANCILLA_H + k] != 0 {
            self.ram[ANCILLA_X_LO + k] = self.ram[ANCILLA_X_LO + k]
                .wrapping_add(self.ram[BG1HOFS_COPY2])
                .wrapping_sub(self.ram[BG2HOFS_COPY2]);
            self.ram[ANCILLA_Y_LO + k] = self.ram[ANCILLA_Y_LO + k]
                .wrapping_add(self.ram[BG1VOFS_COPY2])
                .wrapping_sub(self.ram[BG2VOFS_COPY2]);
        }
        self.arrow_draw(k);
    }

    fn ancilla_sword_beam(&mut self, k: usize) {
        const SWORD_BEAM_YVEL2: [i8; 4] = [0, 0, -6, -6];
        const SWORD_BEAM_XVEL2: [i8; 4] = [-8, -10, 0, 0];
        const SWORD_BEAM_CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];
        const SWORD_BEAM_CHAR2: [u8; 3] = [0xb7, 0x80, 0x83];

        let mut flags = 2;

        if self.frame_control_view().submodule() == 0 {
            self.ancilla_set_xy(
                k,
                read_le_u16(&self.ram, SWORDBEAM_TEMP_X),
                read_le_u16(&self.ram, SWORDBEAM_TEMP_Y),
            );
            self.ancilla_move_x(k);
            self.ancilla_move_y(k);
            let x = self.ancilla_get_x(k);
            let y = self.ancilla_get_y(k);
            write_le_u16(&mut self.ram, SWORDBEAM_TEMP_X, x);
            write_le_u16(&mut self.ram, SWORDBEAM_TEMP_Y, y);

            let g = self.ram[ANCILLA_G + k];
            self.ram[ANCILLA_G + k] = g.wrapping_add(1);
            if g & 0x0f == 0 {
                self.ram[SOUND_EFFECT_2] = self.ancilla_calculate_sfx_pan(k) | 1;
            }

            if self.ancilla_check_sprite_collision(k).is_some()
                || self.ancilla_check_tile_collision(k) != 0
            {
                let j = self.ram[ANCILLA_DIR + k] as usize;
                self.ancilla_set_xy(
                    k,
                    self.ancilla_get_x(k)
                        .wrapping_add(SWORD_BEAM_XVEL2[j] as i16 as u16),
                    self.ancilla_get_y(k)
                        .wrapping_add(SWORD_BEAM_YVEL2[j] as i16 as u16),
                );
                self.ram[ANCILLA_TYPE + k] = 4;
                self.ram[ANCILLA_TIMER + k] = 7;
                self.ram[ANCILLA_NUMSPR + k] = 0x10;
                return;
            }
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                flags = 4;
                self.ram[ANCILLA_AUX_TIMER + k] = 2;
            }
        }

        let oam_org = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut oam = oam_org;
        let s = self.ram[ANCILLA_S_PLAYER + k];
        for i in (0..=3).rev() {
            if self.frame_control_view().submodule() == 0 {
                self.ram[SWORDBEAM_ARR + i] = self.ram[SWORDBEAM_ARR + i].wrapping_add(s) & 0x3f;
            }
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                self.ram[SWORDBEAM_ARR + i],
                self.ram[SWORDBEAM_VAR2],
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                SWORD_BEAM_CHAR[i],
                flags | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
            oam += 4;
        }

        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
            if !sign8(self.ram[ANCILLA_ARR3 + k]) {
                self.ancilla_sword_beam_check_offscreen(k, oam_org);
                return;
            }

            self.ram[ANCILLA_ARR3 + k] = 0;
            self.ram[ANCILLA_ARR1 + k] = self.ram[ANCILLA_ARR1 + k].wrapping_add(1) & 3;
            if self.ram[ANCILLA_ARR1 + k] == 3 {
                self.ram[SWORDBEAM_VAR1] = self.ram[SWORDBEAM_VAR1].wrapping_add(s) & 0x3f;
            }
        }

        let t = self.ram[ANCILLA_ARR1 + k];
        if t != 3 {
            let pt =
                self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                    self.ram[SWORDBEAM_VAR1],
                    self.ram[SWORDBEAM_VAR2],
                ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                SWORD_BEAM_CHAR2[t as usize],
                4 | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
        }

        self.ancilla_sword_beam_check_offscreen(k, oam_org);
    }

    fn ancilla_sword_beam_check_offscreen(&mut self, k: usize, oam_org: usize) {
        for i in 0..4 {
            if self.ram[oam_org + i * 4 + 1] != 0xf0 {
                return;
            }
        }
        self.ram[ANCILLA_TYPE + k] = 0;
    }

    fn ancilla0_d_spin_attack_full_charge_spark(&mut self, k: usize) {
        const SWORD_FULL_CHARGE_SPARK_Y: [i8; 4] = [-8, 27, 12, 12];
        const SWORD_FULL_CHARGE_SPARK_X: [i8; 4] = [4, 4, -13, 20];
        const SWORD_FULL_CHARGE_SPARK_FLAGS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];

        self.ram[ANCILLA_OAM_IDX + k] =
            self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 4) as u8;

        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TYPE + k] = 0;
            return;
        }

        let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;

        let x = self
            .player_state_view()
            .x()
            .wrapping_add(SWORD_FULL_CHARGE_SPARK_X[j] as i16 as u16)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(SWORD_FULL_CHARGE_SPARK_Y[j] as i16 as u16)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));

        let flags = SWORD_FULL_CHARGE_SPARK_FLAGS[self.ram[ANCILLA_FLOOR + k] as usize];
        write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, (flags as u16) << 8);
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        self.ancilla_set_oam(oam, x, y, 0xd7, flags | 2, 0);
    }

    fn ancilla20_blanket(&mut self, k: usize) {
        const BEDSPREAD_CHAR: [u8; 8] = [0x0a, 0x0a, 0x0a, 0x0a, 0x0c, 0x0c, 0x0a, 0x0a];
        const BEDSPREAD_FLAGS: [u8; 8] = [0, 0x60, 0xa0, 0xe0, 0, 0x60, 0xa0, 0xe0];

        let (mut x, mut y) = self.ancilla_prep_oam_coord(k);

        if self.ram[LINK_POSE_DURING_OPENING] == 0 {
            self.oam_allocate_from_region_b(0x10);
        } else {
            self.oam_allocate_from_region_a(0x10);
        }

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut j = if self.ram[LINK_POSE_DURING_OPENING] != 0 {
            4
        } else {
            0
        };
        for i in (0..=3).rev() {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                BEDSPREAD_CHAR[j],
                BEDSPREAD_FLAGS[j] | 0x0d | self.ram[OAM_PRIORITY_VALUE + 1],
                2,
            );
            x = x.wrapping_add(16);
            if i == 2 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(8);
            }
            j += 1;
            oam += 4;
        }
    }

    fn ancilla21_snore(&mut self, k: usize) {
        const BEDSPREAD_DMA: [u8; 3] = [0x44, 0x43, 0x42];

        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if (self.ram[ANCILLA_AUX_TIMER + k] as i8).is_negative() {
            if self.ram[ANCILLA_ITEM_TO_LINK + k] != 2 {
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            }
            self.ram[ANCILLA_AUX_TIMER + k] = 7;
        }

        self.ram[ANCILLA_X_VEL + k] =
            self.ram[ANCILLA_X_VEL + k].wrapping_add(self.ram[ANCILLA_STEP + k]);
        if abs8(self.ram[ANCILLA_X_VEL + k]) >= 8 {
            self.ram[ANCILLA_STEP + k] = (-(self.ram[ANCILLA_STEP + k] as i8)) as u8;
        }

        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
        if self.ancilla_y(k) <= self.player_state_view().y().wrapping_sub(24) {
            self.ram[ANCILLA_TYPE + k] = 0;
        }

        self.ram[LINK_DMA_VAR5] = BEDSPREAD_DMA[self.ram[ANCILLA_ITEM_TO_LINK + k] as usize];
        let (x, y) = self.ancilla_prep_oam_coord(k);
        self.ancilla_set_oam(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            x,
            y,
            9,
            0x24,
            0,
        );
    }

    fn ancilla23_link_poof(&mut self, k: usize) {
        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
            self.ram[ANCILLA_AUX_TIMER + k] = 7;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 3 {
                self.ram[ANCILLA_TYPE + k] = 0;
                self.ram[LINK_IS_TRANSFORMING] = 0;
                self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
                if self.ram[ANCILLA_STEP + k] == 0 {
                    self.ram[LINK_ANIMATION_STEPS] = 0;
                    self.ram[LINK_VISIBILITY_STATUS] = 0;
                    let bunny = if self.ram[OVERWORLD_SCREEN_INDEX] & 0x40 != 0 {
                        1
                    } else {
                        0
                    };
                    self.ram[LINK_IS_BUNNY_MIRROR] = bunny;
                    self.ram[LINK_IS_BUNNY] = bunny;
                    if self.ram[LINK_IS_BUNNY] != 0 {
                        self.LoadGearPalettes_bunny();
                    } else {
                        self.LoadActualGearPalettes();
                    }
                }
                return;
            }
        }
        self.morph_poof_draw(k);
    }

    fn ancilla24_gravestone(&mut self, k: usize) {
        const ANCILLA_GRAVESTONE_CHAR: [u8; 4] = [0xc8, 0xc8, 0xd8, 0xd8];
        const ANCILLA_GRAVESTONE_FLAGS: [u8; 4] = [0, 0x40, 0, 0x40];

        let (mut x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.oam_allocate_from_region_b(16);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for i in 0..4 {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ANCILLA_GRAVESTONE_CHAR[i],
                ANCILLA_GRAVESTONE_FLAGS[i] | 0x3d,
                2,
            );
            x = x.wrapping_add(16);
            if i == 1 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(8);
            }
            oam += 4;
        }
    }

    fn ancilla34_skull_woods_fire(&mut self, k: usize) {
        const SKULL_WOODS_FIRE_DRAW_Y: [i8; 4] = [0, 0, 0, -3];
        const SKULL_WOODS_FIRE_DRAW_CHAR: [u8; 4] = [0x8e, 0xa0, 0xa2, 0xa4];
        const SKULL_WOODS_FIRE_DRAW_EXT: [u8; 4] = [2, 2, 2, 0];
        const SKULL_WOODS_FIRE_DRAW2_X: [i8; 24] = [
            -13, -21, -10, -1, -1, -1, -16, -27, -4, -16, -6, -25, -16, -27, -4, -16, -6, -25, -13,
            -5, -27, -11, -22, -3,
        ];
        const SKULL_WOODS_FIRE_DRAW2_Y: [i8; 24] = [
            -31, -24, -22, -1, -1, -1, -37, -32, -32, -23, -16, -14, -37, -32, -32, -23, -16, -14,
            -35, -29, -28, -20, -13, -11,
        ];
        const SKULL_WOODS_FIRE_DRAW2_CHAR: [u8; 24] = [
            0x86, 0x86, 0x86, 0xff, 0xff, 0xff, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x8a, 0x8a,
            0x8a, 0x8a, 0x8a, 0x8a, 0x9b, 0x9b, 0x9b, 0x9b, 0x9b, 0x9b,
        ];
        const SKULL_WOODS_FIRE_DRAW2_FLAGS: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80, 0x40, 0x40, 0x80, 0x40, 0,
        ];
        const SKULL_WOODS_FIRE_DRAW2_EXT: [u8; 24] = [
            2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0,
        ];

        if self.ram[SKULLWOODSFIRE_VAR4] != 0 && self.ram[ANCILLA_ITEM_TO_LINK + k] != 4 && {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            sign8(self.ram[ANCILLA_AUX_TIMER + k])
        } {
            self.ram[ANCILLA_AUX_TIMER + k] = 5;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
        }

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for i in (0..=3).rev() {
            self.ram[SKULLWOODSFIRE_VAR5 + i] = self.ram[SKULLWOODSFIRE_VAR5 + i].wrapping_sub(1);
            if sign8(self.ram[SKULLWOODSFIRE_VAR5 + i]) {
                self.ram[SKULLWOODSFIRE_VAR5 + i] = 5;
                if self.ram[SKULLWOODSFIRE_VAR0 + i] != 128 {
                    self.ram[SKULLWOODSFIRE_VAR0 + i] =
                        self.ram[SKULLWOODSFIRE_VAR0 + i].wrapping_add(1);
                    if self.ram[SKULLWOODSFIRE_VAR0 + i] == 0
                        || self.ram[SKULLWOODSFIRE_VAR0 + i] == 4
                    {
                        self.ram[SKULLWOODSFIRE_VAR0 + i] = 0;
                        let var9 = read_le_u16(&self.ram, SKULLWOODSFIRE_VAR9).wrapping_sub(8);
                        write_le_u16(&mut self.ram, SKULLWOODSFIRE_VAR9, var9);
                        if var9 < 200 && self.ram[SKULLWOODSFIRE_VAR4] != 1 {
                            self.ram[SKULLWOODSFIRE_VAR4] = 1;
                            let pan = (0x98u16.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2))
                                as u8)
                                >> 5;
                            self.ram[SOUND_EFFECT_1] = K_BOMBOS_SFX[pan as usize] | 0x0c;
                        }
                        if var9 < 168 {
                            self.ram[SKULLWOODSFIRE_VAR0 + i] = 128;
                        }
                        let var11 = read_le_u16(&self.ram, SKULLWOODSFIRE_VAR11);
                        write_le_u16(&mut self.ram, SKULLWOODSFIRE_X_ARR + i * 2, var11);
                        write_le_u16(&mut self.ram, SKULLWOODSFIRE_Y_ARR + i * 2, var9);
                        if self.ram[SOUND_EFFECT_1] == 0 {
                            let pan = (read_le_u16(&self.ram, SKULLWOODSFIRE_VAR11)
                                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2))
                                as u8)
                                >> 5;
                            self.ram[SOUND_EFFECT_1] = K_BOMBOS_SFX[pan as usize] | 0x2a;
                        }
                    }
                }
            }

            if !sign8(self.ram[SKULLWOODSFIRE_VAR0 + i]) {
                let j = self.ram[SKULLWOODSFIRE_VAR0 + i] as usize;
                let x = read_le_u16(&self.ram, SKULLWOODSFIRE_X_ARR + i * 2)
                    .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
                let y = read_le_u16(&self.ram, SKULLWOODSFIRE_Y_ARR + i * 2)
                    .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
                    .wrapping_add(SKULL_WOODS_FIRE_DRAW_Y[j] as i16 as u16);
                self.ancilla_set_oam(
                    oam,
                    x,
                    y,
                    SKULL_WOODS_FIRE_DRAW_CHAR[j],
                    0x32,
                    SKULL_WOODS_FIRE_DRAW_EXT[j],
                );
                oam += 4;
                if SKULL_WOODS_FIRE_DRAW_EXT[j] != 2 {
                    self.ancilla_set_oam(
                        oam,
                        x.wrapping_add(8),
                        y,
                        SKULL_WOODS_FIRE_DRAW_CHAR[j].wrapping_add(1),
                        0x32,
                        SKULL_WOODS_FIRE_DRAW_EXT[j],
                    );
                    oam += 4;
                }
            }
        }

        let mut i = 3i32;
        while sign8(self.ram[SKULLWOODSFIRE_VAR0 + i as usize]) {
            i -= 1;
            if i < 0 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }

        if self.ram[SKULLWOODSFIRE_VAR4] == 0 || self.ram[ANCILLA_ITEM_TO_LINK + k] == 4 {
            return;
        }

        let mut j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 6;
        for _ in 0..6 {
            if SKULL_WOODS_FIRE_DRAW2_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    168u16
                        .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2))
                        .wrapping_add(SKULL_WOODS_FIRE_DRAW2_X[j] as i16 as u16),
                    200u16
                        .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
                        .wrapping_add(SKULL_WOODS_FIRE_DRAW2_Y[j] as i16 as u16),
                    SKULL_WOODS_FIRE_DRAW2_CHAR[j],
                    SKULL_WOODS_FIRE_DRAW2_FLAGS[j] | 0x32,
                    SKULL_WOODS_FIRE_DRAW2_EXT[j],
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn morph_poof_draw(&mut self, k: usize) {
        const MORPH_POOF_X: [i8; 12] = [0, 0, 0, 0, 0, 8, 0, 8, -4, 12, -4, 12];
        const MORPH_POOF_Y: [i8; 12] = [0, 0, 0, 0, 0, 0, 8, 8, -4, -4, 12, 12];
        const MORPH_POOF_FLAGS: [u8; 12] = [
            0, 0xff, 0xff, 0xff, 0x40, 0, 0xc0, 0x80, 0, 0x40, 0x80, 0xc0,
        ];
        const MORPH_POOF_CHAR: [u8; 3] = [0x86, 0xa9, 0x9b];
        const MORPH_POOF_EXT: [u8; 3] = [2, 0, 0];
        if self.ram[SORT_SPRITES_SETTING] != 0
            && self.ram[ANCILLA_FLOOR + k] != 0
            && (self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] == 0 || self.ram[FRAME_COUNTER] & 1 == 0)
        {
            write_le_u16(&mut self.ram, OAM_CUR_PTR, 0x08d0);
            write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0x0a20 + (0x0d0 >> 2));
        }
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
        let ext = MORPH_POOF_EXT[j];
        let chr = MORPH_POOF_CHAR[j];
        for i in 0..4 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(MORPH_POOF_X[j * 4 + i] as i16 as u16),
                y.wrapping_add(MORPH_POOF_Y[j * 4 + i] as i16 as u16),
                chr,
                MORPH_POOF_FLAGS[j * 4 + i] | 4 | self.ram[OAM_PRIORITY_VALUE + 1],
                ext,
            );
            if ext == 2 {
                break;
            }
            oam += 4;
        }
    }

    fn ancilla40_dwarf_poof(&mut self, k: usize) {
        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
            self.ram[ANCILLA_AUX_TIMER + k] = 7;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 3 {
                self.ram[ANCILLA_TYPE + k] = 0;
                self.ram[TAGALONG_APPEARANCE_NONE_FLAG_ANCILLA] = 0;
                return;
            }
        }
        self.morph_poof_draw(k);
    }

    fn ancilla3_f_bush_poof(&mut self, k: usize) {
        const BUSH_POOF_DRAW_X: [i8; 16] = [0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, -2, 10, -2, 10];
        const BUSH_POOF_DRAW_Y: [i8; 16] = [0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, -2, -2, 10, 10];
        const BUSH_POOF_DRAW_CHAR: [u8; 16] = [
            0x86, 0x87, 0x96, 0x97, 0xa9, 0xa9, 0xa9, 0xa9, 0x8a, 0x8b, 0x9a, 0x9b, 0x9b, 0x9b,
            0x9b, 0x9b,
        ];
        const BUSH_POOF_DRAW_FLAGS: [u8; 16] = [
            0, 0, 0, 0, 0, 0x40, 0x80, 0xc0, 0, 0, 0, 0, 0xc0, 0x80, 0x40, 0,
        ];

        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TIMER + k] = 7;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 4 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }
        self.oam_get_buffer_position(0x10, 4);
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;

        let mut j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 4;
        for _ in 0..4 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(BUSH_POOF_DRAW_X[j] as i16 as u16),
                y.wrapping_add(BUSH_POOF_DRAW_Y[j] as i16 as u16),
                BUSH_POOF_DRAW_CHAR[j],
                BUSH_POOF_DRAW_FLAGS[j] | 4 | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
            j += 1;
            oam += 4;
        }
    }

    fn ancilla26_sword_swing_sparkle(&mut self, k: usize) {
        const SWORD_SWING_SPARKLE_X: [i8; 48] = [
            5, 10, -1, 5, 10, -4, 5, 10, -4, -4, -1, -1, 0, 5, -1, 0, 5, 14, 0, 5, 14, 14, -1, -1,
            -23, -27, -1, -23, -27, -22, -23, -27, -22, -22, -1, -1, 32, 35, -1, 32, 35, 30, 32,
            35, 30, 30, -1, -1,
        ];
        const SWORD_SWING_SPARKLE_Y: [i8; 48] = [
            -22, -18, -1, -22, -18, -17, -22, -18, -17, -17, -1, -1, 35, 40, -1, 35, 40, 37, 35,
            40, 37, 37, -1, -1, 2, 7, -1, 2, 7, 19, 2, 7, 19, 19, -1, -1, 2, 7, -1, 2, 7, 19, 2, 7,
            19, 19, -1, -1,
        ];
        const SWORD_SWING_SPARKLE_CHAR: [u8; 48] = [
            0xb7, 0xb7, 0xff, 0x80, 0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7,
            0xff, 0x80, 0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7, 0xff, 0x80,
            0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7, 0xff, 0x80, 0x80, 0xb7,
            0x83, 0x83, 0x80, 0x83, 0xff, 0xff,
        ];
        const SWORD_SWING_SPARKLE_FLAGS: [u8; 48] = [
            0, 0, 0xff, 0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff, 0, 0, 0xff, 0, 0, 0, 0x80, 0x80,
            0, 0x80, 0xff, 0xff, 0, 0, 0xff, 0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff, 0, 0, 0xff,
            0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff,
        ];
        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
            self.ram[ANCILLA_AUX_TIMER + k] = 0;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 4 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }
        self.ancilla_set_xy(
            k,
            self.player_state_view().x(),
            self.player_state_view().y(),
        );

        let (x, y) = self.ancilla_prep_oam_coord(k);

        let mut j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 3
            + self.ram[ANCILLA_DIR + k] as usize * 12;

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for _ in (0..=2).rev() {
            let chr = SWORD_SWING_SPARKLE_CHAR[j];
            if chr != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(SWORD_SWING_SPARKLE_X[j] as i16 as u16),
                    y.wrapping_add(SWORD_SWING_SPARKLE_Y[j] as i16 as u16),
                    chr,
                    SWORD_SWING_SPARKLE_FLAGS[j] | 0x4 | self.ram[OAM_PRIORITY_VALUE + 1],
                    0,
                );
            }
            j += 1;
            oam += 4;
        }
    }

    fn ancilla2_d_somaria_block_fizz(&mut self, k: usize) {
        const SOMARIA_BLOCK_FIZZLE_X: [i8; 6] = [-4, -1, -8, 0, -6, -2];
        const SOMARIA_BLOCK_FIZZLE_Y: [i8; 6] = [-4, -1, -4, -4, -4, -4];
        const SOMARIA_BLOCK_FIZZLE_CHAR: [u8; 6] = [0x92, 0xff, 0xf9, 0xf9, 0xf9, 0xf9];
        const SOMARIA_BLOCK_FIZZLE_FLAGS: [u8; 6] = [6, 0xff, 0x86, 0xc6, 0x86, 0xc6];

        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if (self.ram[ANCILLA_AUX_TIMER + k] as i8) < 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = 3;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 3 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut z = self.ram[ANCILLA_Z + k];
        if z == 0xff {
            z = 0;
        }
        let y = y.wrapping_sub(z as i8 as i16 as u16);
        let mut j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 2;
        for _ in 0..2 {
            if SOMARIA_BLOCK_FIZZLE_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(SOMARIA_BLOCK_FIZZLE_X[j] as i16 as u16),
                    y.wrapping_add(SOMARIA_BLOCK_FIZZLE_Y[j] as i16 as u16),
                    SOMARIA_BLOCK_FIZZLE_CHAR[j],
                    SOMARIA_BLOCK_FIZZLE_FLAGS[j] & !0x30 | self.ram[OAM_PRIORITY_VALUE + 1],
                    0,
                );
            }
            j += 1;
            oam += 4;
        }
    }

    fn ancilla39_somaria_platform_poof(&mut self, k: usize) {
        const SOMARIAN_PLATFORM_POOF_TAB0: [u8; 4] = [1, 0, 3, 2];
        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if (self.ram[ANCILLA_AUX_TIMER + k] as i8) >= 0 {
            return;
        }
        self.ram[ANCILLA_TYPE + k] = 0;
        let x = self.ancilla_get_x(k) & !7 | 4;
        let y = self.ancilla_get_y(k) & !7 | 4;
        let floor = self.ram[ANCILLA_FLOOR + k];
        if let Some(j) = self.sprite_spawn_dynamically_for_ancilla(k, 0xed) {
            self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
            self.sprite_set_x(j, x);
            self.sprite_set_y(j, y);

            let pos = (((x & 0x01f8) >> 3) + ((y & 0x01f8) << 3)) as usize
                + if floor >= 1 { 0x1000 } else { 0 };

            let mut t = 0usize;
            if self.ram[DUNG_BG2_ATTR_TABLE + pos.wrapping_sub(0x40)] & 0xf0 != 0xb0 {
                t += 1;
                if self.ram[DUNG_BG2_ATTR_TABLE + pos + 0x40] & 0xf0 != 0xb0 {
                    t += 1;
                    if self.ram[DUNG_BG2_ATTR_TABLE + pos.wrapping_sub(1)] & 0xf0 != 0xb0 {
                        t += 1;
                    }
                }
            }
            self.ram[SPRITE_D + j] = SOMARIAN_PLATFORM_POOF_TAB0[t];
            self.ram[SPRITE_FLOOR + j] = 0;
        } else {
            self.ancilla_draw_somaria_block(k);
        }
    }

    fn ancilla3_a_big_bomb_explosion(&mut self, k: usize) {
        const SUPER_BOMB_EXPLODE_X: [i8; 9] = [0, -16, 0, 16, -24, 24, -16, 0, 16];
        const SUPER_BOMB_EXPLODE_Y: [i8; 9] = [0, -16, -24, -16, 0, 0, 16, 24, 16];
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
            if self.ram[ANCILLA_ARR3 + k] == 0 {
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                if self.ram[ANCILLA_ITEM_TO_LINK + k] == 2 {
                    self.ancilla_sfx2_pan(k, 0x0c);
                }
                if self.ram[ANCILLA_ITEM_TO_LINK + k] == 11 {
                    self.ram[ANCILLA_TYPE + k] = 0;
                    return;
                }
                self.ram[ANCILLA_ARR3 + k] =
                    K_BOMB_TAB0[self.ram[ANCILLA_ITEM_TO_LINK + k] as usize];
            }
        }

        write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, 0x3000);
        let numframes = K_BOMB_DRAW_TAB2[self.ram[ANCILLA_ITEM_TO_LINK + k] as usize] as usize;
        let j = K_BOMB_DRAW_TAB0[self.ram[ANCILLA_ITEM_TO_LINK + k] as usize] as usize * 6;
        self.ram[ANCILLA_STEP + k] = (j * 2) as u8;

        let mut yy = 0usize;
        for i in (0..=8).rev() {
            let x = self
                .ancilla_get_x(k)
                .wrapping_add(SUPER_BOMB_EXPLODE_X[i] as i16 as u16)
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
            let y = self
                .ancilla_get_y(k)
                .wrapping_add(SUPER_BOMB_EXPLODE_Y[i] as i16 as u16)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
            if x < 256 && y < 256 {
                self.ancilla_allocate_oam_from_region_a_or_d_or_f((j * 2) as usize, 0x18);
                let base_oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
                let oam = base_oam + yy;
                let next_oam = self.ancilla_draw_explosion(oam, j, 0, numframes, 0x32, x, y);
                yy += next_oam - oam;
            }
        }

        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 3 && self.ram[ANCILLA_ARR3 + k] == 1 {
            let old = if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0 {
                self.ram[FOLLOWER_INDICATOR]
            } else {
                0
            };
            self.ram[FOLLOWER_INDICATOR] = 13;
            self.bomb_check_for_destructibles(self.ancilla_get_x(k), self.ancilla_get_y(k), 0);
            self.ram[FOLLOWER_INDICATOR] = old;
        }
    }

    fn ancilla3_b_sword_up_sparkle(&mut self, k: usize) {
        const ANCILLA_VICTORY_SPARKLE_X: [i8; 16] =
            [16, 0, 0, 0, 8, 16, 8, 16, 9, 15, 0, 0, 12, 0, 0, 0];
        const ANCILLA_VICTORY_SPARKLE_Y: [i8; 16] =
            [-7, 0, 0, 0, -11, -11, -3, -3, -7, -7, 0, 0, -7, 0, 0, 0];
        const ANCILLA_VICTORY_SPARKLE_CHAR: [u8; 16] = [
            0x92, 0xff, 0xff, 0xff, 0x93, 0x93, 0x93, 0x93, 0xf9, 0xf9, 0xff, 0xff, 0x80, 0xff,
            0xff, 0xff,
        ];
        const ANCILLA_VICTORY_SPARKLE_FLAGS: [u8; 16] = [
            0, 0xff, 0xff, 0xff, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0xff, 0xff, 0, 0xff, 0xff, 0xff,
        ];

        if self.ram[ANCILLA_AUX_TIMER + k] != 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            return;
        }

        self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_ARR3 + k]) {
            self.ram[ANCILLA_ARR3 + k] = 1;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 4 {
                self.ram[ANCILLA_TYPE + k] = 0;
                self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
                return;
            }
        }
        self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 4;
        for _ in 0..4 {
            if ANCILLA_VICTORY_SPARKLE_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    self.player_state_view()
                        .x()
                        .wrapping_add(ANCILLA_VICTORY_SPARKLE_X[j] as i16 as u16)
                        .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)),
                    self.player_state_view()
                        .y()
                        .wrapping_add(ANCILLA_VICTORY_SPARKLE_Y[j] as i16 as u16)
                        .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)),
                    ANCILLA_VICTORY_SPARKLE_CHAR[j],
                    ANCILLA_VICTORY_SPARKLE_FLAGS[j] | 4 | self.ram[OAM_PRIORITY_VALUE + 1],
                    0,
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn spin_spark_draw(&mut self, k: usize, offs: i32) {
        const INITIAL_SPIN_SPARK_CHAR: [u8; 32] = [
            0x92, 0xff, 0xff, 0xff, 0x8c, 0x8c, 0x8c, 0x8c, 0xd6, 0xd6, 0xd6, 0xd6, 0x93, 0x93,
            0x93, 0x93, 0xd6, 0xd6, 0xd6, 0xd6, 0xd7, 0xff, 0xff, 0xff, 0x80, 0xff, 0xff, 0xff,
            0x22, 0xff, 0xff, 0xff,
        ];
        const INITIAL_SPIN_SPARK_FLAGS: [u8; 29] = [
            0x22, 0xff, 0xff, 0xff, 0x22, 0x62, 0xa2, 0xe2, 0x24, 0x64, 0xa4, 0xe4, 0x22, 0x62,
            0xa2, 0xe2, 0x22, 0x62, 0xa2, 0xe2, 0x22, 0xff, 0xff, 0xff, 0x22, 0xff, 0xff, 0xff,
            0xfc,
        ];
        const INITIAL_SPIN_SPARK_Y: [i8; 29] = [
            -4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -4, 0, 0, 0, -4,
            0, 0, 0, -4,
        ];
        const INITIAL_SPIN_SPARK_X: [i16; 29] = [
            -4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -4, 0, 0, 0, -4,
            0, 0, 0, 0x11a5,
        ];

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut t = (i32::from(self.ram[ANCILLA_ITEM_TO_LINK + k]) + offs) * 4;
        assert!(t < 32);
        for _ in 0..4 {
            let idx = t as usize;
            if INITIAL_SPIN_SPARK_CHAR[idx] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(INITIAL_SPIN_SPARK_X[idx] as u16),
                    y.wrapping_add(INITIAL_SPIN_SPARK_Y[idx] as i16 as u16),
                    INITIAL_SPIN_SPARK_CHAR[idx],
                    INITIAL_SPIN_SPARK_FLAGS[idx] & !0x30 | self.ram[OAM_PRIORITY_VALUE + 1],
                    0,
                );
                oam += 4;
            }
            t += 1;
        }
    }

    fn ancilla2_a_spin_attack_sparkle_a(&mut self, k: usize) {
        const INITIAL_SPIN_SPARK_TIMER: [u8; 6] = [4, 2, 3, 3, 2, 1];

        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                self.ram[ANCILLA_AUX_TIMER + k] = 0;
                if self.ram[ANCILLA_TIMER + k] == 0 {
                    let j = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                    self.ram[ANCILLA_ITEM_TO_LINK + k] = j;
                    self.ram[ANCILLA_TIMER + k] = INITIAL_SPIN_SPARK_TIMER[j as usize];
                    if j == 5 {
                        if self.ram[ANCILLA_STEP + k] != 0 {
                            self.add_sword_beam(j);
                        } else {
                            self.spin_attack_sparkle_a_transmute_to_next_spark(k);
                        }
                        return;
                    }
                }
            }
        }
        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0 {
            return;
        }
        self.spin_spark_draw(k, -1);
    }

    fn spin_attack_sparkle_a_transmute_to_next_spark(&mut self, k: usize) {
        const TRANSMUTE_SPIN_SPARK_ARR: [u8; 16] = [
            0x21, 0x20, 0x1f, 0x1e, 3, 2, 1, 0, 0x12, 0x11, 0x10, 0x0f, 0x31, 0x30, 0x2f, 0x2e,
        ];
        const TRANSMUTE_SPIN_SPARK_X: [i8; 4] = [-3, 21, 25, -8];
        const TRANSMUTE_SPIN_SPARK_Y: [i8; 4] = [28, -2, 24, 6];

        self.ram[ANCILLA_TYPE + k] = 0x2b;
        let mut j = self.ram[LINK_DIRECTION_FACING] as usize * 2;
        self.ram[SWORDBEAM_ARR] = TRANSMUTE_SPIN_SPARK_ARR[j];
        self.ram[SWORDBEAM_ARR + 1] = TRANSMUTE_SPIN_SPARK_ARR[j + 1];
        self.ram[SWORDBEAM_ARR + 2] = TRANSMUTE_SPIN_SPARK_ARR[j + 2];
        self.ram[SWORDBEAM_ARR + 3] = TRANSMUTE_SPIN_SPARK_ARR[j + 3];
        self.ram[SWORDBEAM_VAR1] = TRANSMUTE_SPIN_SPARK_ARR[j + 3];
        self.ram[ANCILLA_AUX_TIMER + k] = 2;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0x4c;
        self.ram[ANCILLA_ARR3 + k] = 8;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_L + k] = 0;
        self.ram[ANCILLA_ARR1 + k] = 255;
        self.ram[SWORDBEAM_VAR2] = 20;

        let swordbeam_temp_x = self.player_state_view().x().wrapping_add(8);
        let swordbeam_temp_y = self.player_state_view().y().wrapping_add(12);
        write_le_u16(&mut self.ram, SWORDBEAM_TEMP_X, swordbeam_temp_x);
        write_le_u16(&mut self.ram, SWORDBEAM_TEMP_Y, swordbeam_temp_y);

        j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        self.ancilla_set_xy(
            k,
            self.player_state_view()
                .x()
                .wrapping_add(TRANSMUTE_SPIN_SPARK_X[j] as i16 as u16),
            self.player_state_view()
                .y()
                .wrapping_add(TRANSMUTE_SPIN_SPARK_Y[j] as i16 as u16),
        );
        self.ancilla2_b_spin_attack_sparkle_b(k);
    }

    fn ancilla2_b_spin_attack_sparkle_b(&mut self, k: usize) {
        const SPIN_SPARK_CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];

        if self.ram[ANCILLA_L + k] != 0 {
            self.spin_attack_sparkle_b_closer(k);
            return;
        }
        let mut flags = 2;
        if self.frame_control_view().submodule() == 0 {
            let t = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_sub(3);
            self.ram[ANCILLA_ITEM_TO_LINK + k] = t;
            if t < 13 {
                self.ram[ANCILLA_AUX_TIMER + k] = 1;
                self.ram[ANCILLA_L + k] = 1;
                self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
                self.spin_attack_sparkle_b_closer(k);
                return;
            }
            self.ram[ANCILLA_STEP + k] = if t < 0x42 {
                3
            } else if t == 0x46 {
                1
            } else if t == 0x43 {
                2
            } else {
                0
            };
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                flags = 4;
                self.ram[ANCILLA_AUX_TIMER + k] = 2;
            }
        }

        let oam_org = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut oam = oam_org;
        let mut i = self.ram[ANCILLA_STEP + k] as usize;
        loop {
            if self.frame_control_view().submodule() == 0 {
                self.ram[SWORDBEAM_ARR + i] = self.ram[SWORDBEAM_ARR + i].wrapping_add(4) & 0x3f;
            }
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                self.ram[SWORDBEAM_ARR + i],
                self.ram[SWORDBEAM_VAR2],
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                SPIN_SPARK_CHAR[i],
                flags | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
            oam += 4;
            if i == 0 {
                break;
            }
            i -= 1;
        }

        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
            if !sign8(self.ram[ANCILLA_ARR3 + k]) {
                if self.ram[ANCILLA_ITEM_TO_LINK + k] == 7 {
                    self.ram[BYTEWISE_EXTENDED_OAM + (oam_org - OAM_BUF) / 4 + 3] = 1;
                }
                return;
            }

            self.ram[ANCILLA_ARR3 + k] = 0;
            self.ram[ANCILLA_ARR1 + k] = self.ram[ANCILLA_ARR1 + k].wrapping_add(1) & 3;
            if self.ram[ANCILLA_ARR1 + k] == 3 {
                self.ram[SWORDBEAM_VAR1] = self.ram[SWORDBEAM_VAR1].wrapping_add(9) & 0x3f;
            }
        }

        let t = self.ram[ANCILLA_ARR1 + k];
        if t != 3 {
            const SPIN_SPARK_CHAR2: [u8; 3] = [0xb7, 0x80, 0x83];
            let pt =
                self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                    self.ram[SWORDBEAM_VAR1],
                    self.ram[SWORDBEAM_VAR2],
                ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                SPIN_SPARK_CHAR2[t as usize],
                4 | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
        }
        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 7 {
            self.ram[BYTEWISE_EXTENDED_OAM + (oam_org - OAM_BUF) / 4 + 3] = 1;
        }
    }

    fn spin_attack_sparkle_b_closer(&mut self, k: usize) {
        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
            self.ram[ANCILLA_AUX_TIMER + k] = 1;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 3 {
                self.ram[ANCILLA_TYPE + k] = 0;
            }
        }
        self.spin_spark_draw(k, 4);
    }

    fn ancilla35_master_sword_receipt(&mut self, k: usize) {
        const SWORD_CEREMONY_X: [i8; 8] = [-1, 8, -1, 8, 0, 7, 0, 7];
        const SWORD_CEREMONY_Y: [i8; 8] = [1, 1, 9, 9, 1, 1, 9, 9];
        const SWORD_CEREMONY_CHAR: [u8; 8] = [0x86, 0x86, 0x96, 0x96, 0x87, 0x87, 0x97, 0x97];
        const SWORD_CEREMONY_FLAGS: [u8; 8] = [1, 0x41, 1, 0x41, 1, 0x41, 1, 0x41];

        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TYPE + k] = 0;
            return;
        }
        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = if self.ram[ANCILLA_ITEM_TO_LINK + k] == 2 {
                0
            } else {
                self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1)
            };
        }

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let item = self.ram[ANCILLA_ITEM_TO_LINK + k];
        if item == 0 {
            return;
        }

        let mut j = item.wrapping_sub(1) as usize * 4;
        for _ in 0..4 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(SWORD_CEREMONY_X[j] as i16 as u16),
                y.wrapping_add(SWORD_CEREMONY_Y[j] as i16 as u16),
                SWORD_CEREMONY_CHAR[j],
                SWORD_CEREMONY_FLAGS[j] & !0x30 | 4 | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
            j += 1;
            oam += 4;
        }
    }

    fn ancilla36_flute(&mut self, k: usize) {
        const FLUTE_VELS: [u8; 4] = [0x18, 0x10, 0x0a, 0];

        if self.frame_control_view().submodule() == 0 {
            if self.ram[ANCILLA_STEP + k] != 3 {
                self.ram[ANCILLA_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + k].wrapping_sub(2);
                self.ancilla_move_x(k);
                self.ancilla_move_z(k);
                if sign8(self.ram[ANCILLA_Z + k]) || self.ram[ANCILLA_Z + k] >= 0xf0 {
                    self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                    self.ram[ANCILLA_Z_VEL + k] = FLUTE_VELS[self.ram[ANCILLA_STEP + k] as usize];
                    self.ram[ANCILLA_Z + k] = 0;
                }
            } else if self.ancilla_check_link_collision(k, 2)
                && self.ram[RELATED_TO_HOOKSHOT] == 0
                && self.ram[LINK_AUXILIARY_STATE] == 0
            {
                self.ram[ANCILLA_TYPE + k] = 0;
                self.ram[ITEM_RECEIPT_METHOD] = 0;
                self.link_receive_item(0x14, 0);
                return;
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        self.ancilla_set_oam(
            oam,
            x,
            y.wrapping_sub(self.ram[ANCILLA_Z + k] as i8 as i16 as u16),
            0x24,
            self.ram[OAM_PRIORITY_VALUE + 1] | 4,
            2,
        );
        if self.ram[oam + 1] == 0xf0 {
            self.ram[ANCILLA_TYPE + k] = 0;
        }
    }

    fn ancilla37_weathervane_explosion(&mut self, k: usize) {
        let var2 = read_le_u16(&self.ram, WEATHERVANE_VAR2).wrapping_sub(1);
        write_le_u16(&mut self.ram, WEATHERVANE_VAR2, var2);
        if var2 != 0 {
            return;
        }
        write_le_u16(&mut self.ram, WEATHERVANE_VAR2, 1);
        if self.ram[WEATHERVANE_VAR1] == 0 {
            self.ram[WEATHERVANE_VAR1] = 1;
            self.ram[MUSIC_CONTROL] = 0xf3;
        }
        self.ram[ANCILLA_G + k] = self.ram[ANCILLA_G + k].wrapping_sub(1);
        if self.ram[ANCILLA_G + k] != 0 {
            return;
        }
        self.ram[ANCILLA_G + k] = 1;
        if self.ram[ANCILLA_ARR3 + k] == 0 {
            self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_add(1);
            self.ancilla_sfx2_near(0x0c);
        }
        if self.ram[ANCILLA_STEP + k] == 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                self.ram[ANCILLA_STEP + k] = 1;
                self.overworld_alter_weathervane();
                self.ancilla_add_cutscene_duck(0x38, 0);
            }
        }
        self.ram[WEATHERVANE_VAR13] = k as u8;
        self.ram[WEATHERVANE_VAR14] = 0;
        for i in (0..=11).rev() {
            if self.ram[WEATHERVANE_ARR12 + i] == 0xff {
                continue;
            }
            self.ram[WEATHERVANE_ARR11 + i] = self.ram[WEATHERVANE_ARR11 + i].wrapping_sub(1);
            if sign8(self.ram[WEATHERVANE_ARR11 + i]) {
                self.ram[WEATHERVANE_ARR11 + i] = 1;
                self.ram[WEATHERVANE_ARR12 + i] ^= 1;
            }

            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[WEATHERVANE_ARR12 + i];
            self.ram[ANCILLA_Y_LO + k] = self.ram[WEATHERVANE_ARR6 + i];
            self.ram[ANCILLA_Y_HI + k] = self.ram[WEATHERVANE_ARR7 + i];
            self.ram[ANCILLA_X_LO + k] = self.ram[WEATHERVANE_ARR8 + i];
            self.ram[ANCILLA_X_HI + k] = self.ram[WEATHERVANE_ARR9 + i];
            self.ram[ANCILLA_Z + k] = self.ram[WEATHERVANE_ARR10 + i];
            self.ram[ANCILLA_Y_VEL + k] = self.ram[WEATHERVANE_ARR3 + i];
            self.ram[ANCILLA_X_VEL + k] = self.ram[WEATHERVANE_ARR4 + i];
            self.ram[WEATHERVANE_ARR5 + i] = self.ram[WEATHERVANE_ARR5 + i].wrapping_sub(1);
            self.ram[ANCILLA_Z_VEL + k] = self.ram[WEATHERVANE_ARR5 + i];

            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            self.ancilla_move_z(k);

            let c = if self.ram[ANCILLA_Z + k] < 0xf0 {
                0
            } else {
                0xff
            };
            self.ancilla_draw_weathervane_explosion_wood_debris(k);
            if sign8(c) {
                self.ram[WEATHERVANE_ARR12 + i] = c;
            }
            self.ram[WEATHERVANE_ARR6 + i] = self.ram[ANCILLA_Y_LO + k];
            self.ram[WEATHERVANE_ARR7 + i] = self.ram[ANCILLA_Y_HI + k];
            self.ram[WEATHERVANE_ARR8 + i] = self.ram[ANCILLA_X_LO + k];
            self.ram[WEATHERVANE_ARR9 + i] = self.ram[ANCILLA_X_HI + k];
            self.ram[WEATHERVANE_ARR10 + i] = self.ram[ANCILLA_Z + k];
        }
        for i in (0..=11).rev() {
            if self.ram[WEATHERVANE_ARR12 + i] != 0xff {
                return;
            }
        }
        self.ram[ANCILLA_TYPE + k] = 0;
    }

    fn ancilla2_c_somaria_block(&mut self, k: usize) {
        const SOMARIAN_BLOCK_COLL_X: [i8; 12] = [0, 0, -8, 8, 0, 0, 0, 0, 8, -8, -8, 8];
        const SOMARIAN_BLOCK_COLL_Y: [i8; 12] = [-8, 8, 0, 0, 0, 0, 0, 0, -8, 8, -8, 8];

        self.ram[ANCILLA_G + k] = self.ram[ANCILLA_G + k].wrapping_sub(1);
        if !sign8(self.ram[ANCILLA_G + k]) {
            return;
        }
        self.ram[ANCILLA_G + k] = 0;

        if self.ram[ANCILLA_H + k] == 0 {
            if matches!(self.frame_control_view().submodule(), 0 | 8 | 16) {
                self.ancilla_handle_lift_logic(k);
            } else if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize
                && self.ram[ANCILLA_K + k] != 0
            {
                if self.ram[ANCILLA_K + k] != 3 {
                    self.ancilla_latch_link_coordinates(k, 3);
                    self.ancilla_latch_altitude_above_link(k);
                    self.ram[ANCILLA_K + k] = 3;
                }
                self.ancilla_latch_carried_position(k);
            }
            if self.ram[PLAYER_IS_INDOORS] != 0 {
                if self.ram[ANCILLA_K + k] == 0
                    && self.ram[LINK_STATE_BITS] & 0x80 == 0
                    && (self.ram[ANCILLA_Z + k] == 0 || self.ram[ANCILLA_Z + k] == 0xff)
                {
                    if self.ram[SOMARIA_BLOCK_BG_CHECK_FLAG] != 0 {
                        let mut j = (self.ram[FRAME_COUNTER] & 3) as usize;
                        loop {
                            let bak = self.ram[ANCILLA_OBJPRIO + k];
                            let x = self
                                .ancilla_get_x(k)
                                .wrapping_add(SOMARIAN_BLOCK_COLL_X[j] as i16 as u16);
                            let y = self
                                .ancilla_get_y(k)
                                .wrapping_add(SOMARIAN_BLOCK_COLL_Y[j] as i16 as u16);
                            self.ancilla_check_tile_collision_targeted(k, x, y);
                            self.ram[ANCILLA_OBJPRIO + k] = bak;
                            if matches!(self.ram[ANCILLA_TILE_ATTR_PLAYER + k], 0xb6 | 0xbc) {
                                self.ancilla_set_xy(k, x, y);
                                self.ancilla_add_somaria_platform_poof(k);
                                if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
                                    self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                                }
                                return;
                            }
                            j += 4;
                            if j >= 12 {
                                break;
                            }
                        }
                    } else if !self.somaria_block_check_for_switch(k)
                        && (self.ram[ANCILLA_Z + k] == 0 || self.ram[ANCILLA_Z + k] == 0xff)
                    {
                        self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER] =
                            self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER].wrapping_add(1);
                    }
                } else if self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] == k as u8 + 1 {
                    self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER] = 0;
                }
            }
        } else if self.ram[PLAYER_IS_INDOORS] != 0
            && self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] == k as u8 + 1
        {
            self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER] = 0;
        }

        let mut old_y = self.ancilla_latch_y_coord_to_z(k);
        let s1a = self.ram[ANCILLA_DIR + k];
        let mut s1b = self.ram[ANCILLA_OBJPRIO + k];
        self.ram[ANCILLA_OBJPRIO + k] = 0;
        let mut flag = self.ancilla_check_tile_collision_class2(k);

        if self.ram[PLAYER_IS_INDOORS] != 0
            && self.ram[ANCILLA_L + k] != 0
            && self.ram[ANCILLA_TILE_ATTR_PLAYER + k] == 0x1c
        {
            self.ram[ANCILLA_T_PLAYER + k] = 1;
        }

        loop {
            if flag
                && (self.ram[LINK_STATE_BITS] & 0x80 == 0
                    || self.ram[LINK_PICKING_THROW_STATE] != 0)
            {
                if s1b == 0 && self.ram[ANCILLA_ARR4 + k] == 0 && self.ram[ANCILLA_Z + k] != 0 {
                    self.ram[ANCILLA_ARR4 + k] = 1;
                    let qq = if self.ram[ANCILLA_DIR + k] == 1 {
                        16
                    } else {
                        4
                    };
                    if self.ram[ANCILLA_Y_VEL + k] != 0 {
                        self.ram[ANCILLA_Y_VEL + k] = if sign8(self.ram[ANCILLA_Y_VEL + k]) {
                            qq
                        } else {
                            (-(qq as i8)) as u8
                        };
                    }
                    if self.ram[ANCILLA_X_VEL + k] != 0 {
                        self.ram[ANCILLA_X_VEL + k] = if sign8(self.ram[ANCILLA_X_VEL + k]) {
                            4
                        } else {
                            (-4i8) as u8
                        };
                    }
                    if self.ram[ANCILLA_DIR + k] == 1 && self.ram[ANCILLA_Z + k] != 0 {
                        self.ram[ANCILLA_Y_VEL + k] = (-4i8) as u8;
                        self.ram[ANCILLA_L + k] = 2;
                    }
                }
            } else if self.ram[LINK_STATE_BITS] & 0x80 == 0
                && (self.ram[ANCILLA_Z + k] == 0 || self.ram[ANCILLA_Z + k] == 0xff)
            {
                self.ram[ANCILLA_DIR + k] = 16;
                let bak0 = self.ram[ANCILLA_OBJPRIO + k];
                self.ancilla_check_tile_collision(k);
                self.ram[ANCILLA_OBJPRIO + k] = bak0;
                let a = self.ram[ANCILLA_TILE_ATTR_PLAYER + k];
                if a == 0x26 {
                    flag = true;
                    continue;
                } else if a == 0x0c || a == 0x1c {
                    if self.ram[DUNG_HDR_COLLISION] != 3 {
                        if self.ram[ANCILLA_FLOOR + k] == 0
                            && self.ram[ANCILLA_Z + k] != 0
                            && self.ram[ANCILLA_Z + k] != 0xff
                        {
                            self.ram[ANCILLA_FLOOR + k] = 1;
                        }
                    } else {
                        old_y = self
                            .ancilla_get_y(k)
                            .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_Y_VEL));
                        self.ancilla_set_x(
                            k,
                            self.ancilla_get_x(k)
                                .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_X_VEL)),
                        );
                    }
                } else if a == 0x20 || (a & 0xf0) == 0xb0 && a != 0xb6 && a != 0xbc {
                    if self.ram[LINK_STATE_BITS] & 0x80 == 0 {
                        if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
                            self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                        }
                        if self.ram[ANCILLA_TIMER + k] == 0 {
                            if self.ram[LINK_SPEED_SETTING] == 18 {
                                self.ram[LINK_SPEED_SETTING] = 0;
                                self.ram[PLAYER_DEFENSE_FLAGS] = 0;
                            }
                            self.ram[ANCILLA_TYPE + k] = 0;
                            return;
                        }
                    }
                } else if a == 8 {
                    if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
                        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                    }
                    if self.ram[ANCILLA_TIMER + k] == 0 {
                        self.ancilla_set_y(k, self.ancilla_get_y(k).wrapping_sub(24));
                        self.ancilla_transmute_to_splash(k);
                        return;
                    }
                } else if matches!(a, 0x68 | 0x69 | 0x6a | 0x6b) {
                    self.ancilla_apply_conveyor(k);
                    old_y = self.ancilla_get_y(k);
                } else {
                    self.ram[ANCILLA_TIMER + k] =
                        if (self.ram[ANCILLA_L + k] | self.ram[ANCILLA_H + k]) != 0 {
                            0
                        } else {
                            2
                        };
                }
            }
            break;
        }

        s1b |= self.ram[ANCILLA_OBJPRIO + k];

        if self.ram[LINK_STATE_BITS] & 0x80 == 0 {
            self.ram[ANCILLA_S_PLAYER + k] = self.ram[ANCILLA_S_PLAYER + k].wrapping_sub(1);
            if self.ram[ANCILLA_S_PLAYER + k] == 0 {
                self.ram[ANCILLA_S_PLAYER + k] = 1;
                self.ram[ANCILLA_OBJPRIO + k] = 0;
                if self.ancilla_check_basic_sprite_collision(k).is_some() {
                    self.ram[ANCILLA_S_PLAYER + k] = 7;
                    self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                    if self.ram[ANCILLA_STEP + k] == 5 {
                        self.somaria_block_fizzle_away(k);
                        return;
                    }
                }
            }
        }
        self.ancilla_set_y(k, old_y);
        self.ram[ANCILLA_DIR + k] = s1a;
        self.ram[ANCILLA_OBJPRIO + k] = s1b;

        self.ancilla_draw_somaria_block(k);
    }

    fn quake_spell_shake_screen(&mut self, _k: usize) {
        let quake_var3 = read_le_u16(&self.ram, QUAKE_VAR3);
        write_le_u16(&mut self.ram, BG1_Y_OFFSET, quake_var3);
        write_le_u16(&mut self.ram, QUAKE_VAR3, 0u16.wrapping_sub(quake_var3));
        self.ram[LINK_Y_VEL] = self.ram[LINK_Y_VEL].wrapping_add(quake_var3 as u8);
    }

    fn ancilla1_c_quake_spell(&mut self, k: usize) {
        if self.frame_control_view().submodule() != 0 {
            if self.ram[QUAKE_ARR2 + 4] != K_QUAKE_TAB1[4] {
                self.ancilla_draw_quake_initial_bolts(k);
            }
            return;
        }
        if self.ram[ANCILLA_STEP + k] != 2 {
            self.quake_spell_shake_screen(k);
            self.quake_spell_control_bolts(k);
            self.quake_spell_spread_bolts(k);
            return;
        }
        self.medallion_check_sprite_damage(k);
        self.prepare_apply_rumble_to_sprites();
        self.ram[ANCILLA_TYPE + k] = 0;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
        self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 1;
        self.ram[SPIN_ATTACK_SOUND_LATCH] = 0;
        self.ram[STATE_FOR_SPIN_ATTACK] = 0;
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[FLAG_UNK1] = 0;
        write_le_u16(&mut self.ram, BG1_X_OFFSET, 0);
        write_le_u16(&mut self.ram, BG1_Y_OFFSET, 0);
        if self.ram[OVERWORLD_SCREEN_INDEX] == 0x47
            && self.ram[SAVE_OW_EVENT_INFO_ANCILLA + 0x47] & 0x20 == 0
            && self.ancilla_check_for_entrance_trigger(3)
        {
            self.ram[TRIGGER_SPECIAL_ENTRANCE_ANCILLA] = 4;
            self.frame_control_view_mut().set_subsubmodule(0);
            self.ram[R16] = 0;
        }
        self.ram[BUTTON_MASK_B_Y] = if self.ram[BUTTON_B_FRAMES] != 0 {
            self.ram[JOYPAD1H_LAST] & 0x80
        } else {
            0
        };
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[MAGIC_SPELL_PLAYER_LOCK_FLAG] = 0;
    }

    fn quake_spell_control_bolts(&mut self, k: usize) {
        self.ram[QUAKE_VAR4] = self.ram[ANCILLA_STEP + k];
        let mut j = self.ram[QUAKE_VAR5] as i32;
        loop {
            let uj = j as usize;
            if self.ram[QUAKE_ARR2 + uj] != K_QUAKE_TAB1[uj] {
                self.ram[QUAKE_ARR1 + uj] = self.ram[QUAKE_ARR1 + uj].wrapping_sub(1);
                if sign8(self.ram[QUAKE_ARR1 + uj]) {
                    self.ram[QUAKE_ARR1 + uj] = 1;
                    self.ram[QUAKE_ARR2 + uj] = self.ram[QUAKE_ARR2 + uj].wrapping_add(1);
                    if self.ram[QUAKE_ARR2 + uj] != K_QUAKE_TAB1[uj] {
                        if j == 0 && self.ram[QUAKE_ARR2 + uj] == 2 {
                            self.ancilla_sfx2_near(0x0c);
                            self.ram[QUAKE_VAR5] = 1;
                        } else if j == 1 && self.ram[QUAKE_ARR2 + uj] == 2 {
                            self.ram[QUAKE_VAR5] = 4;
                        } else if j == 4 && self.ram[QUAKE_ARR2 + uj] == 7 {
                            self.ram[QUAKE_VAR4] = 1;
                        }
                        self.ancilla_draw_quake_initial_bolts(uj);
                    }
                } else {
                    self.ancilla_draw_quake_initial_bolts(uj);
                }
            }
            j -= 1;
            if j < 0 {
                break;
            }
        }
        self.ram[ANCILLA_STEP + k] = self.ram[QUAKE_VAR4];
    }

    fn quake_item(table: &[i16], idx: usize) -> (i16, i16, u8) {
        let base = idx * 3;
        (table[base], table[base + 1], table[base + 2] as u8)
    }

    fn ancilla_draw_quake_initial_bolts(&mut self, k: usize) {
        const QUAKE_DRAW_GROUND_BOLTS_TAB: [u8; 5] = [0, 0x18, 0, 0x18, 0x2f];

        let t = self.ram[QUAKE_ARR2 + k].wrapping_add(QUAKE_DRAW_GROUND_BOLTS_TAB[k]) as usize;
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let idx = K_QUAKE_ITEM_POS[t] as usize;
        let end = K_QUAKE_ITEM_POS[t + 1] as usize;
        for item_idx in idx..end {
            let (ix, iy, f) = Self::quake_item(&K_QUAKE_ITEMS, item_idx);
            let x = read_le_u16(&self.ram, QUAKE_VAR2)
                .wrapping_add(ix as u16)
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
            let y = read_le_u16(&self.ram, QUAKE_VAR1)
                .wrapping_add(iy as u16)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));

            let mut xval = self.ram[oam];
            let mut yval = 0xf0;
            if x < 256 && y < 256 {
                xval = x as u8;
                if y < 0xf0 {
                    yval = y as u8;
                }
            }
            self.ram[oam] = xval;
            self.ram[oam + 1] = yval;
            self.ram[oam + 2] = K_QUAKE_DRAW_GROUND_BOLTS_CHAR[(f & 0x0f) as usize];
            self.ram[oam + 3] = (f & 0xc0) | 0x3c;
            self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4] = 2;
            oam += 4;
            let cur = read_le_u16(&self.ram, OAM_CUR_PTR).wrapping_add(4);
            let ext = read_le_u16(&self.ram, OAM_EXT_CUR_PTR).wrapping_add(1);
            write_le_u16(&mut self.ram, OAM_CUR_PTR, cur);
            write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, ext);
        }
    }

    fn quake_spell_spread_bolts(&mut self, k: usize) {
        if self.ram[ANCILLA_STEP + k] != 1 {
            return;
        }
        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TIMER + k] = 2;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 55 {
                self.ram[ANCILLA_STEP + k] = 2;
                return;
            }
        }
        let t = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
        let idx = K_QUAKE_ITEM_POS2[t] as usize;
        let end = K_QUAKE_ITEM_POS2[t + 1] as usize;
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for item_idx in idx..end {
            let (x, y, f) = Self::quake_item(&K_QUAKE_ITEMS2, item_idx);
            self.ram[oam] = x as u8;
            self.ram[oam + 1] = y as u8;
            self.ram[oam + 2] = K_QUAKE_DRAW_GROUND_BOLTS_CHAR[(f & 0x0f) as usize];
            self.ram[oam + 3] = (f & 0xc0) | 0x3c;
            self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4] = (f >> 4) & 3;
            let cur = read_le_u16(&self.ram, OAM_CUR_PTR).wrapping_add(4);
            let ext = read_le_u16(&self.ram, OAM_EXT_CUR_PTR).wrapping_add(1);
            write_le_u16(&mut self.ram, OAM_CUR_PTR, cur);
            write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, ext);
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
        }
    }

    fn ancilla1_f_hookshot(&mut self, k: usize) {
        const HOOKSHOT_MOVE_X: [i8; 4] = [0, 0, 8, -8];
        const HOOKSHOT_MOVE_Y: [i8; 4] = [8, -9, 0, 0];
        const HOOKSHOT_DRAW_FLAGS: [u8; 12] =
            [0, 0, 0xff, 0x80, 0x80, 0xff, 0x40, 0xff, 0x40, 0, 0xff, 0];
        const HOOKSHOT_DRAW_CHAR: [u8; 12] =
            [9, 0x0a, 0xff, 9, 0x0a, 0xff, 9, 0xff, 0x0a, 9, 0xff, 0x0a];

        if self.frame_control_view().submodule() == 0 {
            if self.ram[ANCILLA_TIMER + k] == 0 {
                self.ram[ANCILLA_TIMER + k] = 7;
                self.ancilla_sfx2_pan(k, 0x0a);
            }

            if self.ram[RELATED_TO_HOOKSHOT] == 0 {
                self.ancilla_move_y(k);
                self.ancilla_move_x(k);
                if self.ram[ANCILLA_STEP + k] != 0 {
                    self.ram[ANCILLA_ITEM_TO_LINK + k] =
                        self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_sub(1);
                    if sign8(self.ram[ANCILLA_ITEM_TO_LINK + k]) {
                        self.ram[ANCILLA_TYPE + k] = 0;
                        return;
                    }
                } else {
                    self.ram[ANCILLA_ITEM_TO_LINK + k] =
                        self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                    if self.ram[ANCILLA_ITEM_TO_LINK + k] == 32 {
                        self.ram[ANCILLA_STEP + k] = 1;
                        self.ram[ANCILLA_X_VEL + k] = self.ram[ANCILLA_X_VEL + k].wrapping_neg();
                        self.ram[ANCILLA_Y_VEL + k] = self.ram[ANCILLA_Y_VEL + k].wrapping_neg();
                    }

                    if !self.hookshot_should_i_even_bother_with_tiles(k) {
                        if self.ram[ANCILLA_L + k] == 0
                            && self.ram[ANCILLA_STEP + k] == 0
                            && self.ancilla_check_sprite_collision(k).is_some()
                            && self.ram[ANCILLA_STEP + k] == 0
                        {
                            self.ram[ANCILLA_STEP + k] = 1;
                            self.ram[ANCILLA_Y_VEL + k] =
                                self.ram[ANCILLA_Y_VEL + k].wrapping_neg();
                            self.ram[ANCILLA_X_VEL + k] =
                                self.ram[ANCILLA_X_VEL + k].wrapping_neg();
                        }

                        self.hookshot_check_tile_collision(k as i32);

                        let mut r0 = 0u8;
                        let contact = if self.ram[PLAYER_IS_INDOORS] != 0 {
                            if self.ram[ANCILLA_DIR + k] & 2 == 0 {
                                r0 = (self.ram[TILEDETECT_VERTICAL_LEDGE]
                                    | (self.ram[TILEDETECT_VERTICAL_LEDGE] >> 4))
                                    & 3;
                            } else {
                                r0 = self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 3;
                            }
                            r0 != 0
                        } else {
                            (self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 3
                                | self.ram[TILEDETECT_VERTICAL_LEDGE]
                                | self.ram[DETECTION_OF_UNKNOWN_TILE_TYPES])
                                & 0x33
                                != 0
                        };

                        if contact {
                            self.ram[ANCILLA_G + k] = self.ram[ANCILLA_G + k].wrapping_sub(1);
                        }
                        if contact && sign8(self.ram[ANCILLA_G + k]) {
                            if self.ram[ANCILLA_K + k] != 0
                                && ((r0 & 3) != 0
                                    || self.ram[ANCILLA_K + k]
                                        != self.ram[INDEX_OF_INTERACTING_TILE_ANCILLA])
                            {
                                self.ram[ANCILLA_G + k] = 2;
                                self.ram[ANCILLA_L + k] = self.ram[ANCILLA_L + k].wrapping_sub(1);
                                if sign8(self.ram[ANCILLA_L + k]) {
                                    self.ram[ANCILLA_L + k] = 0;
                                }
                            } else {
                                self.ram[ANCILLA_L + k] = self.ram[ANCILLA_L + k].wrapping_add(1);
                                self.ram[ANCILLA_K + k] =
                                    self.ram[INDEX_OF_INTERACTING_TILE_ANCILLA];
                                self.ram[ANCILLA_G + k] = 1;
                            }
                        }

                        if self.ram[ANCILLA_L + k] == 0 {
                            if !sign8(self.ram[ANCILLA_G + k]) {
                                self.ram[ANCILLA_G + k] = self.ram[ANCILLA_G + k].wrapping_sub(1);
                            } else {
                                let blocked = (((read_le_u16(&self.ram, R14) >> 4)
                                    | read_le_u16(&self.ram, R14)
                                    | self.ram[TILEDETECT_STAIR_TILE] as u16
                                    | read_le_u16(&self.ram, R12))
                                    & 3)
                                    != 0;
                                if blocked && self.ram[ANCILLA_STEP + k] == 0 {
                                    self.ram[ANCILLA_STEP + k] = 1;
                                    self.ram[ANCILLA_Y_VEL + k] =
                                        self.ram[ANCILLA_Y_VEL + k].wrapping_neg();
                                    self.ram[ANCILLA_X_VEL + k] =
                                        self.ram[ANCILLA_X_VEL + k].wrapping_neg();
                                    if read_le_u16(&self.ram, TILEDETECT_MISC_TILES) & 3 == 0 {
                                        self.ancilla_add_hookshot_wall_clink(k, 6, 1);
                                        self.ancilla_sfx2_pan(
                                            k,
                                            if read_le_u16(&self.ram, TILEDETECT_MISC_TILES) & 0x30
                                                != 0
                                            {
                                                6
                                            } else {
                                                5
                                            },
                                        );
                                    }
                                }

                                if read_le_u16(&self.ram, TILEDETECT_MISC_TILES) & 3 != 0 {
                                    if self.ram[ANCILLA_ITEM_TO_LINK + k] < 4 {
                                        self.ram[ANCILLA_TYPE + k] = 0;
                                        return;
                                    }
                                    self.ram[RELATED_TO_HOOKSHOT] = 1;
                                    self.ram[HOOKSHOT_EFFECT_INDEX] = k as u8;
                                }
                            }
                        }
                    }
                }
            }
        }

        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        if self.ram[ANCILLA_L + k] != 0 {
            write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, 0x3000);
        }
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;

        let mut j = self.ram[ANCILLA_DIR + k] as usize * 3;
        let mut x = info_x;
        let mut y = info_y;
        for i in (0..=2).rev() {
            if HOOKSHOT_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x,
                    y,
                    HOOKSHOT_DRAW_CHAR[j],
                    HOOKSHOT_DRAW_FLAGS[j] | 2 | self.ram[OAM_PRIORITY_VALUE + 1],
                    0,
                );
                oam += 4;
            }
            if i == 1 {
                x = x.wrapping_sub(8);
                y = y.wrapping_add(8);
            } else {
                x = x.wrapping_add(8);
            }
            j += 1;
        }

        let mut r10 = 0i32;
        let mut n = (self.ram[ANCILLA_ITEM_TO_LINK + k] >> 1) as i32;
        if n >= 7 {
            r10 = n - 7;
            n = 6;
        }
        if n == 0 {
            return;
        }
        if self.ram[ANCILLA_DIR + k] & 1 != 0 {
            r10 = -r10;
        }
        let mut x = info_x;
        let mut y = info_y;
        let j = self.ram[ANCILLA_DIR + k] as usize;
        if HOOKSHOT_MOVE_Y[j] == 0 {
            y = y.wrapping_add(4);
        }
        if HOOKSHOT_MOVE_X[j] == 0 {
            x = x.wrapping_add(4);
        }
        loop {
            if HOOKSHOT_MOVE_Y[j] != 0 {
                y = y.wrapping_add((HOOKSHOT_MOVE_Y[j] as i32 + r10) as i16 as u16);
            }
            if HOOKSHOT_MOVE_X[j] != 0 {
                x = x.wrapping_add((HOOKSHOT_MOVE_X[j] as i32 + r10) as i16 as u16);
            }
            if !self.hookshot_check_proximity_to_link(x as i32, y as i32) {
                self.ancilla_set_oam(
                    oam,
                    x,
                    y,
                    0x19,
                    (self.ram[FRAME_COUNTER] & 2) << 6 | 2 | self.ram[OAM_PRIORITY_VALUE + 1],
                    0,
                );
                oam += 4;
            }
            n -= 1;
            if n < 0 {
                break;
            }
        }
    }

    fn ancilla_draw_ether_blitz_ball(
        &mut self,
        oam: usize,
        arp: &AncillaRadialProjection,
        s: usize,
    ) -> usize {
        const ETHER_BLITZ_BALL_CHAR: [u8; 2] = [0x68, 0x6a];

        let x = read_le_u16(&self.ram, ETHER_X2)
            .wrapping_add(if arp.r6 != 0 {
                0u16.wrapping_sub(arp.r4 as u16)
            } else {
                arp.r4 as u16
            })
            .wrapping_sub(8)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let y = read_le_u16(&self.ram, ETHER_Y3)
            .wrapping_add(if arp.r2 != 0 {
                0u16.wrapping_sub(arp.r0 as u16)
            } else {
                arp.r0 as u16
            })
            .wrapping_sub(8)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        self.ancilla_set_oam(oam, x, y, ETHER_BLITZ_BALL_CHAR[s], 0x3c, 2);
        self.ancilla_allocate_oam_from_custom_region(oam + 4)
    }

    fn ancilla_draw_ether_blitz_segment(
        &mut self,
        oam: usize,
        arp: &AncillaRadialProjection,
        s: usize,
        k: usize,
    ) -> usize {
        const ETHER_SPLITTING_BLITZ_SEGMENT_X: [i8; 16] = [
            -8, -16, -24, -16, -8, 0, 8, -16, -8, -16, -24, -16, -8, 0, 8, 0,
        ];
        const ETHER_SPLITTING_BLITZ_SEGMENT_Y: [i8; 16] = [
            8, 0, -8, -16, -24, -16, -8, -16, 8, 0, -8, -16, -24, -16, -8, 0,
        ];
        const ETHER_SPLITTING_BLITZ_SEGMENT_CHAR: [u8; 32] = [
            0x40, 0x42, 0x66, 0x64, 0x62, 0x60, 0x64, 0x66, 0x42, 0x40, 0x66, 0x64, 0x60, 0x62,
            0x64, 0x66, 0x68, 0x42, 0x68, 0x64, 0x68, 0x60, 0x68, 0x64, 0x68, 0x40, 0x68, 0x66,
            0x68, 0x62, 0x68, 0x64,
        ];
        const ETHER_SPLITTING_BLITZ_SEGMENT_FLAGS: [u8; 32] = [
            0x3c, 0x3c, 0xfc, 0xfc, 0x3c, 0x3c, 0xbc, 0xbc, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c,
            0x7c, 0x7c, 0x3c, 0x7c, 0x3c, 0x3c, 0x3c, 0xbc, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0xfc,
            0x3c, 0xbc, 0x3c, 0xbc,
        ];

        let x = if arp.r6 != 0 {
            0u16.wrapping_sub(arp.r4 as u16)
        } else {
            arp.r4 as u16
        };
        let y = if arp.r2 != 0 {
            0u16.wrapping_sub(arp.r0 as u16)
        } else {
            arp.r0 as u16
        };
        let t = s * 8 + k;
        let base_x = x
            .wrapping_add(read_le_u16(&self.ram, ETHER_X2))
            .wrapping_sub(8)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let base_y = y
            .wrapping_add(read_le_u16(&self.ram, ETHER_Y3))
            .wrapping_sub(8)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        self.ancilla_set_oam(
            oam,
            base_x,
            base_y,
            ETHER_SPLITTING_BLITZ_SEGMENT_CHAR[t * 2],
            ETHER_SPLITTING_BLITZ_SEGMENT_FLAGS[t * 2],
            2,
        );
        self.ancilla_set_oam(
            oam + 4,
            x.wrapping_add(read_le_u16(&self.ram, ETHER_X2))
                .wrapping_add(ETHER_SPLITTING_BLITZ_SEGMENT_X[t] as i16 as u16)
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)),
            y.wrapping_add(read_le_u16(&self.ram, ETHER_Y3))
                .wrapping_add(ETHER_SPLITTING_BLITZ_SEGMENT_Y[t] as i16 as u16)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)),
            ETHER_SPLITTING_BLITZ_SEGMENT_CHAR[t * 2 + 1],
            ETHER_SPLITTING_BLITZ_SEGMENT_FLAGS[t * 2 + 1],
            2,
        );
        self.ancilla_allocate_oam_from_custom_region(oam + 8)
    }

    fn ancilla_draw_ether_blitz(&mut self, k: usize) {
        const ETHER_BLITZ_ORB_FLAGS: [u8; 8] = [0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c];
        const ETHER_BLITZ_SEGMENT_CHAR: [u8; 4] = [0x40, 0x42, 0x44, 0x46];

        let (x, mut y) = self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let t = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
        let mut i = self.ram[ANCILLA_ARR25 + k];
        let mut m = 0usize;
        loop {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ETHER_BLITZ_SEGMENT_CHAR[t * 2 + m],
                ETHER_BLITZ_ORB_FLAGS[0] | self.ram[OAM_PRIORITY_VALUE + 1],
                2,
            );
            y = y.wrapping_sub(16);
            oam += 4;
            m ^= 1;
            i = i.wrapping_sub(1);
            if sign8(i) {
                break;
            }
        }
        if self.ram[ANCILLA_STEP + k] == 1 {
            self.ancilla_draw_ether_orb(k, oam);
        }
    }

    fn ancilla_draw_ether_orb(&mut self, k: usize, mut oam: usize) {
        const ETHER_BLITZ_ORB_CHAR: [u8; 8] = [0x48, 0x48, 0x4a, 0x4a, 0x4c, 0x4c, 0x4e, 0x4e];
        const ETHER_BLITZ_ORB_FLAGS: [u8; 8] = [0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c];

        let mut y = read_le_u16(&self.ram, ETHER_Y)
            .wrapping_sub(1)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        let mut x = read_le_u16(&self.ram, ETHER_X)
            .wrapping_sub(8)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let t = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 4;

        for i in 0..4 {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ETHER_BLITZ_ORB_CHAR[t + i],
                ETHER_BLITZ_ORB_FLAGS[t + i],
                2,
            );
            oam += 4;
            oam = self.ancilla_allocate_oam_from_custom_region(oam);
            x = x.wrapping_add(16);
            if i == 1 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(16);
            }
        }
    }

    fn ancilla_draw_bombos_fire_column(&mut self, kk: usize) {
        const BOMBOS_SPELL_FIRE_COLUMN_X: [i8; 39] = [
            0, -1, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, -1, 1, -1, -1, 2, -1, -1,
        ];
        const BOMBOS_SPELL_FIRE_COLUMN_Y: [i8; 39] = [
            0, -1, -1, 0, -4, -1, 0, -8, -1, 0, -12, -1, 0, -16, -1, 0, -4, -20, 0, -8, -24, 0,
            -12, -28, 0, -16, -32, 0, -16, -32, -18, -34, -1, -35, -1, -1, -36, -1, -1,
        ];
        const BOMBOS_SPELL_FIRE_COLUMN_FLAGS: [u8; 39] = [
            0x3c, 0xff, 0xff, 0x3c, 0x3c, 0xff, 0x3c, 0x3c, 0xff, 0x7c, 0x7c, 0xff, 0x3c, 0x7c,
            0xff, 0x3c, 0x3c, 0x3c, 0xbc, 0x3c, 0x3c, 0x7c, 0x3c, 0x3c, 0x3c, 0x3c, 0x7c, 0x3c,
            0x3c, 0x3c, 0x3c, 0x3c, 0xff, 0x3c, 0xff, 0xff, 0x3c, 0xff, 0xff,
        ];
        const BOMBOS_SPELL_FIRE_COLUMN_CHAR: [u8; 39] = [
            0x40, 0xff, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44,
            0xff, 0x40, 0x46, 0x44, 0x4a, 0x4a, 0x48, 0x4c, 0x4c, 0x4a, 0x4e, 0x4c, 0x4a, 0x4e,
            0x6a, 0x4c, 0x4e, 0x68, 0xff, 0x6a, 0xff, 0xff, 0x4e, 0xff, 0xff,
        ];

        self.ancilla_allocate_oam_from_region_a_or_d_or_f(kk, 0x10);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for _ in 0..1 {
            let mut k = self.ram[BOMBOS_ARR2 + kk] as usize;
            if k == 13 {
                continue;
            }
            k = k * 3 + 2;
            for _ in 0..3 {
                if BOMBOS_SPELL_FIRE_COLUMN_CHAR[k] != 0xff {
                    let x = self.ram[BOMBOS_X_LO + kk] as u16
                        | ((self.ram[BOMBOS_X_HI + kk] as u16) << 8);
                    let y = self.ram[BOMBOS_Y_LO + kk] as u16
                        | ((self.ram[BOMBOS_Y_HI + kk] as u16) << 8);
                    self.ancilla_set_oam(
                        oam,
                        x.wrapping_add(BOMBOS_SPELL_FIRE_COLUMN_X[k] as i16 as u16)
                            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)),
                        y.wrapping_add(BOMBOS_SPELL_FIRE_COLUMN_Y[k] as i16 as u16)
                            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)),
                        BOMBOS_SPELL_FIRE_COLUMN_CHAR[k],
                        BOMBOS_SPELL_FIRE_COLUMN_FLAGS[k],
                        2,
                    );
                    oam += 4;
                }
                oam = self.ancilla_allocate_oam_from_custom_region(oam);
                k = k.wrapping_sub(1);
            }
        }
    }

    fn ancilla_draw_bombos_blast(&mut self, k: usize) {
        const BOMBOS_SPELL_DRAW_BLAST_X: [i8; 32] = [
            -8, -1, -1, -1, -12, -4, -12, -4, -16, 0, -16, 0, -16, 0, -16, 0, -17, 1, -17, 1, -19,
            3, -19, 3, -19, 3, -19, 3, -19, 3, -19, 3,
        ];
        const BOMBOS_SPELL_DRAW_BLAST_Y: [i8; 32] = [
            -8, -1, -1, -1, -12, -12, -4, -4, -16, -16, 0, 0, -16, -16, 0, 0, -17, -17, 1, 1, -19,
            -19, 3, 3, -19, -19, 3, 3, -19, -19, 3, 3,
        ];
        const BOMBOS_SPELL_DRAW_BLAST_FLAGS: [u8; 32] = [
            0x3c, 0xff, 0xff, 0xff, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c,
            0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc,
            0x3c, 0x7c, 0xbc, 0xfc,
        ];
        const BOMBOS_SPELL_DRAW_BLAST_CHAR: [u8; 32] = [
            0x60, 0xff, 0xff, 0xff, 0x62, 0x62, 0x62, 0x62, 0x64, 0x64, 0x64, 0x64, 0x66, 0x66,
            0x66, 0x66, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x6a, 0x6a, 0x6a, 0x6a,
            0x4e, 0x4e, 0x4e, 0x4e,
        ];

        let x = read_le_u16(&self.ram, BOMBOS_X_COORD + k * 2);
        let y = read_le_u16(&self.ram, BOMBOS_Y_COORD + k * 2);
        if self.ram[BOMBOS_ARR3 + k] == 8 {
            return;
        }

        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 0x10);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;

        let mut t = self.ram[BOMBOS_ARR3 + k] as usize * 4 + 3;
        for _ in 0..4 {
            if BOMBOS_SPELL_DRAW_BLAST_CHAR[t] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(BOMBOS_SPELL_DRAW_BLAST_X[t] as i16 as u16)
                        .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)),
                    y.wrapping_add(BOMBOS_SPELL_DRAW_BLAST_Y[t] as i16 as u16)
                        .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)),
                    BOMBOS_SPELL_DRAW_BLAST_CHAR[t],
                    BOMBOS_SPELL_DRAW_BLAST_FLAGS[t],
                    2,
                );
                oam += 4;
            }
            oam = self.ancilla_allocate_oam_from_custom_region(oam);
            t = t.wrapping_sub(1);
        }
    }

    fn ancilla27_duck(&mut self, k: usize) {
        if self.frame_control_view().submodule() == 0 {
            if self.ram[ANCILLA_TIMER + k] != 0 {
                let xt: u16 = if self.read_u32_ram(ENHANCED_FEATURES0) & 1 != 0 {
                    0x40
                } else {
                    0
                };
                self.ancilla_set_xy(
                    k,
                    read_le_u16(&self.ram, BG2HOFS_COPY2)
                        .wrapping_sub(16)
                        .wrapping_sub(xt),
                    self.player_state_view().y().wrapping_sub(8),
                );
                return;
            }

            self.ram[ANCILLA_G + k] = self.ram[ANCILLA_G + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_G + k]) {
                self.ram[ANCILLA_G + k] = 0x28;
                self.ancilla_sfx3_pan(k, 0x1e);
            }

            if self.ram[ANCILLA_L + k] != 0 || self.ram[ANCILLA_STEP + k] != 0 {
                if self.ram[ANCILLA_L + k] == 0 && self.ram[ANCILLA_STEP + k] != 0 {
                    self.ram[FLAG_UNK1] = self.ram[FLAG_UNK1].wrapping_add(1);
                }
                self.ram[ANCILLA_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + k].wrapping_sub(1);
                self.ancilla_move_z(k);
            }
            self.ancilla_move_x(k);

            if self.ram[ANCILLA_L + k] != 0 {
                let x = self.ancilla_get_x(k);
                if self.ram[ANCILLA_STEP + k] != 0 {
                    self.ram[FLAG_UNK1] = self.ram[FLAG_UNK1].wrapping_add(1);
                }
                if !sign16(x) && x >= self.player_state_view().x() {
                    if self.ram[ANCILLA_STEP + k] != 0 {
                        self.ram[ANCILLA_STEP + k] = 0;
                        self.ram[LINK_VISIBILITY_STATUS] = 0;
                        self.ram[TAGALONG_APPEARANCE_NONE_FLAG_ANCILLA] = 0;
                        self.ram[LINK_POSE_FOR_ITEM] = 0;
                        self.ram[ANCILLA_Y_VEL + k] = 0;
                        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
                        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
                        self.ram[PLAYER_SPECIAL_DRAW_FLAG] = 0;
                        self.ram[COUNTDOWN_FOR_BLINK] = 144;
                        if !((self.ram[FOLLOWER_INDICATOR_ANCILLA] == 12
                            || self.ram[FOLLOWER_INDICATOR_ANCILLA] == 13)
                            && self.ram[FOLLOWER_DROPPED] != 0)
                        {
                            self.follower_initialize();
                        }
                    }
                } else if self.player_state_view().x().wrapping_sub(x) < 48 {
                    self.draw_duck(k, 3);
                    return;
                }
            } else if self.ancilla_check_link_collision(k, 1)
                && self.frame_control_view().main_module() != 15
            {
                if self.ram[PLAYER_IS_INDOORS] == 0 {
                    if self.ram[LINK_PLAYER_HANDLER_STATE] == 8
                        || self.ram[LINK_PLAYER_HANDLER_STATE] == 9
                        || self.ram[LINK_PLAYER_HANDLER_STATE] == 10
                        || self.ram[PLAYER_NEAR_PIT_STATE] == 2
                        || (self.ram[LINK_POSE_FOR_ITEM]
                            | self.ram[RELATED_TO_HOOKSHOT]
                            | self.ram[LINK_FORCE_HOLD_SWORD_UP]
                            | self.ram[LINK_DISABLE_SPRITE_DAMAGE])
                            != 0
                        || (self.ram[LINK_STATE_BITS] & 0x80) != 0
                    {
                        self.draw_duck_default(k);
                        return;
                    }
                    for i in (0..5).rev() {
                        let a = self.ram[ANCILLA_TYPE + i];
                        if a == 0x2a || a == 0x1f || a == 0x30 || a == 0x31 || a == 0x41 {
                            self.ram[ANCILLA_TYPE + i] = 0;
                        }
                    }
                    if self.ram[FOLLOWER_INDICATOR_ANCILLA] == 9 {
                        self.ram[FOLLOWER_INDICATOR_ANCILLA] = 0;
                        self.ram[TAGALONG_APPEARANCE_NONE_FLAG_ANCILLA] = 0;
                    }
                }
                self.ram[LINK_STATE_BITS] = 0;
                self.ram[LINK_PICKING_THROW_STATE] = 0;

                self.ram[BG1_X_OFFSET] = 0;
                self.ram[BG1_Y_OFFSET] = 0;
                self.link_reset_properties_a();
                self.ram[LINK_IS_IN_DEEP_WATER] = 0;
                self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
                self.ram[LINK_VISIBILITY_STATUS] = 12;
                self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
                self.ram[LINK_POSE_FOR_ITEM] = 1;
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                self.ram[TAGALONG_APPEARANCE_NONE_FLAG_ANCILLA] = 1;
                self.ram[ANCILLA_STEP + k] = 2;
                self.ram[FLAG_UNK1] = self.ram[FLAG_UNK1].wrapping_add(1);
                self.ram[LINK_GIVE_DAMAGE] = 0;
                if self.ram[PLAYER_IS_INDOORS] != 0 {
                    self.ram[PLAYER_SPECIAL_DRAW_FLAG] = self.ram[PLAYER_IS_INDOORS];
                }
            }
        }
        self.draw_duck_default(k);
    }

    fn draw_duck_default(&mut self, k: usize) {
        self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_ARR3 + k]) {
            self.ram[ANCILLA_ARR3 + k] = 3;
            self.ram[ANCILLA_K + k] = self.ram[ANCILLA_K + k].wrapping_add(1);
            if self.ram[ANCILLA_K + k] == 3 {
                self.ram[ANCILLA_K + k] = 0;
            }
        }
        self.draw_duck(k, self.ram[ANCILLA_K + k]);
    }

    fn draw_duck(&mut self, k: usize, j: u8) {
        self.ram[FLAG_TRAVEL_BIRD] = K_TRAVEL_BIRD_DMA_STUFFS[j as usize];

        let (x, y) = self.ancilla_prep_oam_coord(k);

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let z = if self.ram[ANCILLA_Z + k] != 0 {
            self.ram[ANCILLA_Z + k] as i8 as i16 as u16
        } else {
            0
        };
        let n = self.ram[ANCILLA_STEP + k] as usize + 1;
        for i in 0..n {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(K_TRAVEL_BIRD_DRAW_X[i] as i16 as u16),
                y.wrapping_add(z)
                    .wrapping_add(K_TRAVEL_BIRD_DRAW_Y[i] as i16 as u16),
                K_TRAVEL_BIRD_DRAW_CHAR[i],
                K_TRAVEL_BIRD_DRAW_FLAGS[i] | 0x30,
                2,
            );
            oam += 4;
        }

        self.ancilla_draw_shadow(oam, 1, x, y.wrapping_add(28), 0x30);
        oam += 8;
        if self.ram[ANCILLA_STEP + k] != 0 {
            self.ancilla_draw_shadow(oam, 1, x.wrapping_sub(7), y.wrapping_add(28), 0x30);
        }

        if !sign16(x) && x >= 0x0130 {
            self.ram[ANCILLA_TYPE + k] = 0;
            if self.ram[ANCILLA_L + k] == 0 && self.ram[ANCILLA_STEP + k] != 0 {
                let main_module = self.frame_control_view().main_module();
                self.frame_control_view_mut().set_submodule(10);
                self.ram[SAVED_MODULE_FOR_MENU] = main_module;
                self.frame_control_view_mut().set_main_module(14);
            }
        }
    }

    fn ancilla_draw_weathervane_explosion_wood_debris(&mut self, k: usize) {
        const WEATHERVANE_EXPLODE_CHAR: [u8; 2] = [0x4e, 0x4f];

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let y = y.wrapping_sub(self.ram[ANCILLA_Z + k] as i8 as i16 as u16);
        let i = self.ram[ANCILLA_ITEM_TO_LINK + k];
        if sign8(i) {
            return;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize
            + ((self.ram[WEATHERVANE_VAR14] >> 2) as usize) * 4;
        self.ancilla_set_oam(oam, x, y, WEATHERVANE_EXPLODE_CHAR[i as usize], 0x3c, 0);
        self.ram[WEATHERVANE_VAR14] = self.ram[WEATHERVANE_VAR14].wrapping_add(4);
    }

    fn ancilla38_cutscene_duck(&mut self, k: usize) {
        const TRAVEL_BIRD_INTRO_TAB0: [u8; 2] = [0x40, 0];
        const TRAVEL_BIRD_INTRO_TAB1: [u8; 2] = [28, 60];

        if self.ram[FRAME_COUNTER] & 31 == 0 {
            self.ancilla_sfx3_pan(k, 0x1e);
        }

        self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_ARR3 + k]) {
            self.ram[ANCILLA_ARR3 + k] = 3;
            self.ram[ANCILLA_K + k] ^= 1;
        }

        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if self.ram[ANCILLA_AUX_TIMER + k] == 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = 1;
            if self.ram[ANCILLA_L + k] == 0 {
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_sub(1);
                if !sign8(self.ram[ANCILLA_ITEM_TO_LINK + k]) {
                    self.ram[ANCILLA_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + k].wrapping_add(
                        if self.ram[ANCILLA_STEP + k] != 0 {
                            1
                        } else {
                            (-1i8) as u8
                        },
                    );
                    if abs8(self.ram[ANCILLA_Z_VEL + k]) >= 12 {
                        self.ram[ANCILLA_STEP + k] ^= 1;
                    }
                    self.ancilla38_cutscene_duck_after_stuff(k);
                    return;
                }
                self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
                self.ram[ANCILLA_STEP + k] = 0;
                self.ram[ANCILLA_X_VEL + k] = TRAVEL_BIRD_INTRO_TAB1[0];
                self.ram[ANCILLA_Z_VEL + k] = (-16i8) as u8;
                self.ram[ANCILLA_L + k] = self.ram[ANCILLA_L + k].wrapping_add(1);
                self.ram[ANCILLA_STEP + k] = 3;
            }
            self.ram[ANCILLA_X_VEL + k] =
                self.ram[ANCILLA_X_VEL + k].wrapping_add(if self.ram[ANCILLA_STEP + k] & 1 == 0 {
                    1
                } else {
                    (-1i8) as u8
                });
            let absx = abs8(self.ram[ANCILLA_X_VEL + k]);
            if absx == 0 {
                self.ram[ANCILLA_L + k] = self.ram[ANCILLA_L + k].wrapping_add(1);
                if self.ram[ANCILLA_L + k] == 7 {
                    self.ram[ANCILLA_S_PLAYER + k] = 1;
                }
            }
            if absx >= TRAVEL_BIRD_INTRO_TAB1[self.ram[ANCILLA_S_PLAYER + k] as usize] {
                self.ram[ANCILLA_STEP + k] ^= 3;
            }
            self.ram[ANCILLA_DIR + k] = if sign8(self.ram[ANCILLA_X_VEL + k]) {
                2
            } else {
                3
            };
            let t = TRAVEL_BIRD_INTRO_TAB1[self.ram[ANCILLA_S_PLAYER + k] as usize]
                .wrapping_sub(absx)
                >> 1;
            self.ram[ANCILLA_Z_VEL + k] = if self.ram[ANCILLA_STEP + k] & 2 != 0 {
                0u8.wrapping_sub(t)
            } else {
                t
            };
        }
        self.ancilla38_cutscene_duck_after_stuff(k);
    }

    fn ancilla38_cutscene_duck_after_stuff(&mut self, k: usize) {
        const TRAVEL_BIRD_INTRO_TAB0: [u8; 2] = [0x40, 0];

        self.ancilla_move_x(k);
        self.ancilla_move_z(k);
        self.ram[FLAG_TRAVEL_BIRD] = K_TRAVEL_BIRD_DMA_STUFFS[self.ram[ANCILLA_K + k] as usize + 1];
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        self.ancilla_set_oam(
            oam,
            x.wrapping_add(K_TRAVEL_BIRD_DRAW_X[0] as i16 as u16),
            y.wrapping_add(self.ram[ANCILLA_Z + k] as i8 as i16 as u16)
                .wrapping_add(K_TRAVEL_BIRD_DRAW_Y[0] as i16 as u16),
            K_TRAVEL_BIRD_DRAW_CHAR[0],
            K_TRAVEL_BIRD_DRAW_FLAGS[0]
                | 0x30
                | TRAVEL_BIRD_INTRO_TAB0[(self.ram[ANCILLA_DIR + k] & 1) as usize],
            2,
        );
        self.ancilla_draw_shadow(oam + 4, 1, x, y.wrapping_add(48), 0x30);
        if !sign16(x) && x >= 248 {
            self.ram[ANCILLA_TYPE + k] = 0;
            self.frame_control_view_mut().set_submodule(0);
            self.ram[LINK_ITEM_FLUTE] = 3;
        }
    }

    fn ancilla16_hit_stars(&mut self, k: usize) {
        const ANCILLA_HIT_STARS_CHAR: [u8; 2] = [0x90, 0x91];

        self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
        if !sign8(self.ram[ANCILLA_ARR3 + k]) {
            return;
        }

        self.ram[ANCILLA_ARR3 + k] = 0;
        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                self.ram[ANCILLA_AUX_TIMER + k] = 0;
                self.ram[ANCILLA_ITEM_TO_LINK + k] = 1;
            }
            if self.ram[ANCILLA_ITEM_TO_LINK + k] != 0 {
                self.ram[ANCILLA_Y_VEL + k] = self.ram[ANCILLA_Y_VEL + k].wrapping_sub(4);
                self.ram[ANCILLA_X_VEL + k] = self.ram[ANCILLA_Y_VEL + k];
                if self.ram[ANCILLA_Y_VEL + k] < 232 {
                    self.ram[ANCILLA_TYPE + k] = 0;
                    return;
                }
                self.ancilla_move_y(k);
                self.ancilla_move_x(k);
            }
        }
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let ax = self.ancilla_get_x(k);
        let tt = u16::from(self.ram[ANCILLA_A + k]) | (u16::from(self.ram[ANCILLA_B + k]) << 8);
        let r8 = tt
            .wrapping_mul(2)
            .wrapping_sub(ax)
            .wrapping_sub(8)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));

        if self.ram[ANCILLA_STEP + k] == 2 {
            self.ancilla_allocate_oam_from_region_b_or_e(8);
        }

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut x = x;
        let mut flags = 0;
        for _ in (0..=1).rev() {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ANCILLA_HIT_STARS_CHAR[self.ram[ANCILLA_ITEM_TO_LINK + k] as usize],
                self.ram[OAM_PRIORITY_VALUE + 1] | 4 | flags,
                0,
            );
            flags = 0x40;
            x = (x & 0xff00) | (r8 & 0x00ff);
            oam = self.hit_stars_update_oam_buffer_position(oam + 4);
        }
    }

    fn ancilla17_shovel_dirt(&mut self, k: usize) {
        const SHOVEL_DIRT_XY: [i8; 8] = [18, -13, -9, 4, 18, 13, -9, -11];
        const SHOVEL_DIRT_CHAR: [u8; 2] = [0x40, 0x50];

        let (mut x, mut y) = self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TIMER + k] = 8;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 2 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }
        let b = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
        let j = b + if self.ram[LINK_DIRECTION_FACING] == 4 {
            0
        } else {
            2
        };
        x = x.wrapping_add(SHOVEL_DIRT_XY[j * 2 + 1] as i16 as u16);
        y = y.wrapping_add(SHOVEL_DIRT_XY[j * 2] as i16 as u16);
        for i in 0..2 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add((i * 8) as u16),
                y,
                SHOVEL_DIRT_CHAR[b].wrapping_add(i as u8),
                4 | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
        }
    }

    fn ancilla_magic_powder_draw(&mut self, k: usize) {
        const MAGIC_POWDER_DRAW_X: [i8; 76] = [
            -5, -12, 2, -9, -7, -10, -6, -2, -6, -12, 1, -6, -6, -12, 1, -6, -6, -12, 1, -6, -6,
            -12, 1, -6, -6, -12, 1, -6, -17, -23, -14, -19, -11, -18, -9, -13, -4, -13, -1, -8, -3,
            -9, 0, -5, -3, -10, -1, -5, -4, -13, -1, -8, -3, -9, 0, -5, -3, -10, -1, -5, -3, -13,
            -1, -8, 9, 15, 6, 11, 3, 10, 1, 5, -4, 5, -7, 0,
        ];
        const MAGIC_POWDER_DRAW_Y: [i8; 76] = [
            -20, -15, -13, -7, -18, -13, -13, -13, -20, -13, -13, -8, -20, -13, -13, -8, -19, -12,
            -12, -7, -18, -11, -11, -6, -17, -10, -10, -5, -16, -14, -12, -9, -17, -14, -12, -8,
            -18, -14, -13, -6, -33, -31, -29, -26, -28, -25, -23, -19, -22, -18, -17, -10, -2, 0,
            2, 5, -9, -6, -4, 0, -16, -12, -11, -4, -16, -14, -12, -9, -17, -14, -12, -8, -18, -14,
            -13, -6,
        ];
        const MAGIC_POWDER_DRAW_CHAR: [u8; 19] =
            [9, 10, 10, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9];
        const MAGIC_POWDER_DRAW_FLAGS: [u8; 76] = [
            0x68, 0x24, 0xa2, 0x28, 0x68, 0xe2, 0x28, 0xa4, 0x68, 0xe2, 0xa4, 0x28, 0x22, 0xa4,
            0xe8, 0x62, 0x24, 0xa8, 0xe2, 0x64, 0x28, 0xa2, 0xe4, 0x68, 0x22, 0xa4, 0xe8, 0x62,
            0xe2, 0xa4, 0xe8, 0x64, 0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4,
            0xe8, 0x64, 0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4, 0xe8, 0x64,
            0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4, 0xe8, 0x64, 0xe8, 0xa8,
            0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68,
        ];

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let b = self.ram[ANCILLA_ARR25 + k] as usize;
        let mut j = b * 4;
        for _ in 0..4 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(MAGIC_POWDER_DRAW_X[j] as i16 as u16),
                y.wrapping_add(MAGIC_POWDER_DRAW_Y[j] as i16 as u16),
                MAGIC_POWDER_DRAW_CHAR[b],
                MAGIC_POWDER_DRAW_FLAGS[j] & !0x30 | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
            oam += 4;
            j += 1;
        }
    }

    fn ancilla1_a_powder_dust(&mut self, k: usize) {
        if self.frame_control_view().submodule() == 0 {
            self.powder_apply_damage_to_sprites(k);
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                self.ram[ANCILLA_AUX_TIMER + k] = 1;
                let j = self.ram[ANCILLA_DIR + k] as usize;
                if self.ram[ANCILLA_ITEM_TO_LINK + k] == 9 {
                    self.ram[ANCILLA_TYPE + k] = 0;
                    self.ram[DUNGEON_TORCH_ATTR] = 0;
                    return;
                }
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                self.ram[ANCILLA_ARR25 + k] =
                    K_MAGIC_POWDER_TAB0[self.ram[ANCILLA_ITEM_TO_LINK + k] as usize + j * 10];
            }
        }
        self.ancilla_allocate_oam_from_region_b_or_e(self.ram[ANCILLA_NUMSPR + k]);
        self.ancilla_magic_powder_draw(k);
    }

    fn powder_apply_damage_to_sprites(&mut self, k: usize) {
        for j in (0..16).rev() {
            if ((self.ram[FRAME_COUNTER] ^ j as u8) & 3) != 0
                || self.ram[SPRITE_STATE + j] != 9
                || (self.ram[SPRITE_BUMP_DAMAGE_ANCILLA + j] & 0x20) != 0
            {
                continue;
            }

            let mut hb = self.ancilla_setup_basic_hit_box(k);
            self.sprite_setup_hit_box(j, &mut hb);
            if !self.check_if_hit_boxes_overlap(&hb) {
                continue;
            }

            let mut a = self.ram[SPRITE_TYPE + j];
            if a != 0x0b
                || {
                    a = self.ram[PLAYER_IS_INDOORS];
                    a == 0
                }
                || {
                    a = self.ram[DUNGEON_ROOM_INDEX2].wrapping_sub(1);
                    a != 0
                }
            {
                if a != 0x0d {
                    self.ancilla_check_damage_to_sprite_preset(j, 10);
                    continue;
                }
                if self.ram[SPRITE_HEAD_DIR_ANCILLA + j] != 0 {
                    continue;
                }
            }
            self.ram[SPRITE_HEAD_DIR_ANCILLA + j] = 1;
            self.sprite_spawn_poof_garnish_for_ancilla(j);
        }
    }

    fn garnish_alloc_force_for_ancilla(&self) -> usize {
        (0..30)
            .rev()
            .find(|&k| self.ram[GARNISH_TYPE + k] == 0)
            .unwrap_or(0)
    }

    fn sprite_spawn_poof_garnish_for_ancilla(&mut self, j: usize) {
        let k = self.garnish_alloc_force_for_ancilla();
        self.ram[GARNISH_TYPE + k] = 10;
        self.ram[GARNISH_ACTIVE_ANCILLA] = 10;
        self.ram[GARNISH_X_LO_ANCILLA + k] = self.ram[SPRITE_X_LO + j];
        self.ram[GARNISH_X_HI_ANCILLA + k] = self.ram[SPRITE_X_HI + j];
        let y = self.sprite_get_y(j).wrapping_add(16);
        self.ram[GARNISH_Y_LO_ANCILLA + k] = y as u8;
        self.ram[GARNISH_Y_HI_ANCILLA + k] = (y >> 8) as u8;
        self.ram[GARNISH_SPRITE_ANCILLA + k] = self.ram[SPRITE_FLOOR + j];
        self.ram[GARNISH_COUNTDOWN_ANCILLA + k] = 15;
    }

    fn wish_pond_item_draw(&mut self, k: usize) {
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);

        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 1 {
            self.ram[ANCILLA_ARR4 + k] = 5;
        }

        let oam = self.ancilla_receive_item_draw(
            k,
            x,
            y.wrapping_sub(self.ram[ANCILLA_Z + k] as i8 as i16 as u16),
        );

        if self.ram[LINK_PICKING_THROW_STATE] != 2
            || (!sign8(self.ram[ANCILLA_Z_VEL + k]) && self.ram[ANCILLA_Z_VEL + k] >= 2)
        {
            return;
        }

        let xx = self.asset_u8(71, self.ram[ANCILLA_ITEM_TO_LINK + k] as usize);
        self.ancilla_draw_shadow(
            oam,
            if xx == 2 { 1 } else { 2 },
            x.wrapping_sub(if xx == 2 { 0 } else { 4 }),
            y.wrapping_add(40),
            self.ram[OAM_PRIORITY_VALUE + 1],
        );
    }

    fn ancilla_receive_item_draw(&mut self, k: usize, x: u16, y: u16) -> usize {
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
        let mut a = K_WISH_POND2_OAM_FLAGS[j];
        if sign8(a) {
            a = self.ram[ANCILLA_ARR4 + k];
        }
        self.ancilla_set_oam(
            oam,
            x,
            y,
            0x24,
            a.wrapping_mul(2) | 0x30,
            K_RECEIVE_ITEM_TAB1[j],
        );
        oam += 4;
        if K_RECEIVE_ITEM_TAB1[j] == 0 {
            self.ancilla_set_oam(oam, x, y.wrapping_add(8), 0x34, a.wrapping_mul(2) | 0x30, 0);
            oam += 4;
        }
        oam
    }

    fn item_receipt_transmute_to_rising_crystal(&mut self, k: usize) {
        self.ram[ANCILLA_TYPE + k] = 0x3e;
        self.ram[ANCILLA_Y_VEL + k] = 0;
        self.ram[ANCILLA_X_VEL + k] = 0;
        self.ram[ANCILLA_Y_SUBPIXEL + k] = 0;
        self.ancilla_rising_crystal(k);
    }

    fn ancilla22_item_receipt(&mut self, k: usize) {
        if self.ram[FLAG_IS_LINK_IMMOBILIZED] != 2 {
            if self.frame_control_view().submodule() != 0
                && self.frame_control_view().submodule() != 43
                && self.frame_control_view().submodule() != 9
            {
                if self.frame_control_view().submodule() == 2 {
                    self.ram[ANCILLA_TIMER + k] = 16;
                }
            } else {
                self.ram[FLAG_UNK1] = self.ram[FLAG_UNK1].wrapping_add(1);

                if self.ram[ANCILLA_STEP + k] != 0 && self.ram[ANCILLA_STEP + k] != 3 {
                    self.ram[ANCILLA_AUX_TIMER + k] =
                        self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
                    if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                        self.ancilla22_item_receipt_finish(k);
                        return;
                    }
                    if self.ram[ANCILLA_AUX_TIMER + k] == 0 {
                        self.ancilla22_item_receipt_show_message(k);
                    } else {
                        if self.ram[ANCILLA_AUX_TIMER + k] == 40
                            && self.ram[ANCILLA_STEP + k] != 2
                            && (self.ancilla_add_rupees(k)
                                || self.ram[ANCILLA_ITEM_TO_LINK + k] != 0x17)
                        {
                            self.ancilla_sfx3_near(0x0f);
                        }
                        self.ancilla22_item_receipt_move_label_b(k);
                    }
                } else if self.ram[ANCILLA_ITEM_TO_LINK + k] == 1 && self.ram[ANCILLA_STEP + k] != 2
                {
                    if self.ram[ANCILLA_TIMER + k] == 0 {
                        self.ancilla22_item_receipt_label_a(k);
                        return;
                    }
                    if self.ram[ANCILLA_TIMER + k] == 17 {
                        write_le_u16(&mut self.ram, SHARED_MESSAGE_TIMER, 0x0df3);
                        self.ram[FOLLOWER_INDICATOR] = 0x0e;
                        self.ancilla22_item_receipt_show_message(k);
                    }
                } else {
                    self.ram[ANCILLA_AUX_TIMER + k] =
                        self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
                    let a = self.ram[ANCILLA_AUX_TIMER + k];
                    if a == 0 {
                        self.ancilla22_item_receipt_label_a(k);
                        return;
                    }
                    if a == 1 {
                        let item = self.ram[ANCILLA_ITEM_TO_LINK + k];
                        if (item == 0x37 || item == 0x38 || item == 0x39)
                            && self.zelda_read_apui00() != 0
                        {
                            self.ram[ANCILLA_AUX_TIMER + k] =
                                self.ram[ANCILLA_AUX_TIMER + k].wrapping_add(1);
                        } else {
                            self.ancilla22_item_receipt_show_message(k);
                        }
                    }
                }
            }
        }

        self.ancilla22_item_receipt_draw_and_update(k);
    }

    fn ancilla22_item_receipt_label_a(&mut self, k: usize) {
        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 1 && self.ram[ANCILLA_STEP + k] == 0 {
            self.ram[SOUND_EFFECT_AMBIENT] = 5;
            self.ram[MUSIC_CONTROL] = 2;
        }
        self.ram[LINK_PLAYER_HANDLER_STATE] = if self.ram[LINK_IS_IN_DEEP_WATER] != 0 {
            4
        } else {
            0
        };
        self.ram[LINK_RECEIVEITEM_INDEX] = 0;
        self.ram[LINK_POSE_FOR_ITEM] = 0;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ancilla_add_rupees(k);
        self.ancilla22_item_receipt_finish(k);
    }

    fn ancilla22_item_receipt_finish(&mut self, k: usize) {
        self.ram[ITEM_RECEIPT_METHOD] = 0;
        let a = self.ram[ANCILLA_ITEM_TO_LINK + k];
        if a == 23 && self.ram[LINK_HEART_PIECES] == 0 {
            self.link_receive_item(0x26, 0);
            self.ram[ANCILLA_TYPE + k] = 0;
            self.ram[FLAG_UNK1] = 0;
            return;
        }

        if a == 0x26 || a == 0x3f {
            if self.ram[LINK_HEALTH_CAPACITY] != 0xa0 {
                self.ram[LINK_HEALTH_CAPACITY] = self.ram[LINK_HEALTH_CAPACITY].wrapping_add(8);
                self.ram[LINK_HEARTS_FILLER] = self.ram[LINK_HEARTS_FILLER].wrapping_add(
                    self.ram[LINK_HEALTH_CAPACITY].wrapping_sub(self.ram[LINK_HEALTH_CURRENT]),
                );
                self.ancilla_sfx3_near(0x0d);
            }
        } else if a == 0x3e {
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
            if self.ram[LINK_HEALTH_CAPACITY] != 0xa0 {
                self.ram[LINK_HEALTH_CAPACITY] = self.ram[LINK_HEALTH_CAPACITY].wrapping_add(8);
                self.ram[LINK_HEARTS_FILLER] = self.ram[LINK_HEARTS_FILLER].wrapping_add(8);
                self.ancilla_sfx3_near(0x0d);
            }
        } else if a == 0x42 {
            self.ram[LINK_HEARTS_FILLER] = self.ram[LINK_HEARTS_FILLER].wrapping_add(8);
        } else if a == 0x45 {
            self.ram[LINK_MAGIC_FILLER] = self.ram[LINK_MAGIC_FILLER].wrapping_add(16);
        } else if a == 0x22 || a == 0x23 {
            self.Palette_Load_LinkArmorAndGloves();
        }

        self.ram[ANCILLA_TYPE + k] = 0;
        self.ram[FLAG_UNK1] = 0;
        let a = self.ram[ANCILLA_ITEM_TO_LINK + k];
        if self.ram[ANCILLA_STEP + k] == 3 && a != 0x10 && a != 0x26 && a != 0x0f && a != 0x20 {
            self.prepare_dungeon_exit_from_boss_fight();
        }

        if self.ram[ANCILLA_STEP + k] != 2 {
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
        }
    }

    fn ancilla22_item_receipt_show_message(&mut self, k: usize) {
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            let room = self.world_state_view().dungeon_room();
            if room == 0x00ff
                || room == 0x010f
                || room == 0x0110
                || room == 0x0112
                || room == 0x011f
            {
                self.ancilla22_item_receipt_move_label_b(k);
                return;
            }
        }
        let item = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
        let mut msg = -1i16;
        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0x38 || self.ram[ANCILLA_ITEM_TO_LINK + k] == 0x39
        {
            if self.ram[LINK_WHICH_PENDANTS] & 7 == 7 {
                msg = K_RECEIVE_ITEM_MSGS2[item - 0x38];
            } else {
                msg = K_RECEIVE_ITEM_MSGS[item];
            }
        } else if self.ram[ANCILLA_STEP + k] != 2 {
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0x17 {
                msg = K_RECEIVE_ITEM_MSGS3[self.ram[LINK_HEART_PIECES] as usize];
            } else {
                msg = K_RECEIVE_ITEM_MSGS[item];
            }
        }
        if msg != -1 {
            write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, msg as u16);
            if msg == 0x70 {
                self.ram[SOUND_EFFECT_AMBIENT] = 9;
            }
            self.main_show_text_message();
        }
    }

    fn ancilla22_item_receipt_move_label_b(&mut self, k: usize) {
        if self.ram[ANCILLA_AUX_TIMER + k] >= 24 {
            let a = self.ram[ANCILLA_Y_VEL + k].wrapping_sub(1);
            if a >= 248 {
                self.ram[ANCILLA_Y_VEL + k] = a;
            }
            self.ancilla_move_y(k);
        }
    }

    fn ancilla22_item_receipt_draw_and_update(&mut self, k: usize) {
        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0x20 {
            self.ram[ANCILLA_Z + k] = 0;
            self.ancilla_add_occasional_sparkle(k);
            if self.zelda_read_apui00() == 0 {
                self.ram[MUSIC_CONTROL] = 0x1a;
                self.item_receipt_transmute_to_rising_crystal(k);
                return;
            }
        } else if self.ram[ANCILLA_ITEM_TO_LINK + k] == 1 {
            self.ram[ANCILLA_ARR4 + k] = K_RECEIVE_ITEM_TAB0[0];
            if self.ram[ANCILLA_STEP + k] != 2 {
                if self.ram[ANCILLA_TIMER + k] < 16 {
                    self.ram[ANCILLA_ARR1 + k] = 0;
                    self.ram[ANCILLA_ARR4 + k] = K_RECEIVE_ITEM_TAB0[0];
                } else {
                    self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
                    if sign8(self.ram[ANCILLA_ARR3 + k]) {
                        self.ram[ANCILLA_ARR3 + k] = 2;
                        let mut a = self.ram[ANCILLA_ARR1 + k].wrapping_add(1);
                        if a == 3 {
                            a = 0;
                        }
                        self.ram[ANCILLA_ARR1 + k] = a;
                        self.ram[ANCILLA_ARR4 + k] = K_RECEIVE_ITEM_TAB0[a as usize];
                    }
                }
            }
        }

        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0x34
            || self.ram[ANCILLA_ITEM_TO_LINK + k] == 0x35
            || self.ram[ANCILLA_ITEM_TO_LINK + k] == 0x36
        {
            self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_ARR3 + k]) {
                let mut a = self.ram[ANCILLA_ARR1 + k].wrapping_add(1);
                if a == 3 {
                    a = 0;
                }
                self.ram[ANCILLA_ARR1 + k] = a;
                self.ram[ANCILLA_ARR3 + k] = K_RECEIVE_ITEM_TAB4[a as usize];
                self.WriteTo4BPPBuffer_at_7F4000(K_RECEIVE_ITEM_TAB5[a as usize]);
            }
        }
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.ancilla_receive_item_draw(k, x, y);
    }

    fn ancilla_rising_crystal(&mut self, k: usize) {
        const DUNGEON_CRYSTAL_PENDANT_BIT: [u8; 13] = [0, 0, 4, 2, 0, 16, 2, 1, 64, 4, 1, 32, 8];

        self.ram[ANCILLA_Z + k] = 0;
        self.ancilla_add_occasional_sparkle(k);
        let mut yy = self.ram[ANCILLA_Y_VEL + k].wrapping_sub(1);
        if yy < 0xf0 {
            yy = 0xf0;
        }
        self.ram[ANCILLA_Y_VEL + k] = yy;
        self.ancilla_move_y(k);

        let y = self
            .ancilla_get_y(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY));
        if y < 0x49 {
            self.ancilla_set_y(
                k,
                0x49u16.wrapping_add(read_le_u16(&self.ram, BG2VOFS_COPY)),
            );
            if self.frame_control_view().submodule() == 0 {
                let i = (self.ram[CUR_PALACE_INDEX_X2] >> 1) as usize;
                self.ram[LINK_HAS_CRYSTALS] |= DUNGEON_CRYSTAL_PENDANT_BIT[i];
                self.frame_control_view_mut().set_submodule(0x18);
                self.frame_control_view_mut().set_subsubmodule(0);
                self.ram[AUX_PALETTE_BUFFER + 0x20 * 2..AUX_PALETTE_BUFFER + 0x80 * 2].fill(0);
                write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
                write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, 0);
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.ancilla_receive_item_draw(k, x, y);
    }

    fn ancilla29_milestone_item_receipt(&mut self, k: usize) {
        if self.ram[ANCILLA_ITEM_TO_LINK + k] != 0x10 && self.ram[ANCILLA_ITEM_TO_LINK + k] != 0x0f
        {
            let dung_savegame_state_bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS);
            if dung_savegame_state_bits & 0x4000 != 0 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }

            if dung_savegame_state_bits & 0x8000 == 0 {
                return;
            }

            if self.ram[MILESTONE_ITEM_GFX_SWAP_COUNTDOWN] != 0 {
                if self.ram[MILESTONE_ITEM_GFX_SWAP_COUNTDOWN] == 1 {
                    if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0x20 {
                        self.ram[SOUND_EFFECT_AMBIENT] = 0x0f;
                        self.DecodeAnimatedSpriteTile_variable(0x28);
                    } else {
                        self.DecodeAnimatedSpriteTile_variable(0x23);
                    }
                }
                self.ram[MILESTONE_ITEM_GFX_SWAP_COUNTDOWN] =
                    self.ram[MILESTONE_ITEM_GFX_SWAP_COUNTDOWN].wrapping_sub(1);
                return;
            }
            if self.ram[ANCILLA_ARR3 + k] == 0 && self.ram[ANCILLA_ITEM_TO_LINK + k] == 0x20 {
                self.ram[ANCILLA_ARR3 + k] = 1;
                self.ram[PALETTE_SP6R_INDOORS] = 4;
                write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x0200);
                self.Palette_Load_SpriteEnvironment_Dungeon();
                self.ram[FLAG_UPDATE_CGRAM_IN_NMI] =
                    self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
            }
        } else if self.ram[ANCILLA_G + k] != 0 {
            self.ram[ANCILLA_G + k] = self.ram[ANCILLA_G + k].wrapping_sub(1);
            return;
        }

        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0x20 {
            self.ancilla_add_occasional_sparkle(k);
        }

        if self.frame_control_view().submodule() == 0 {
            if self.ram[ANCILLA_Z + k] < 24
                && self.ancilla_check_link_collision(k, 2)
                && self.ram[RELATED_TO_HOOKSHOT] == 0
                && self.ram[LINK_AUXILIARY_STATE] == 0
            {
                self.ram[ANCILLA_TYPE + k] = 0;
                if self.ram[LINK_PLAYER_HANDLER_STATE] == 25
                    || self.ram[LINK_PLAYER_HANDLER_STATE] == 26
                {
                    self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 0;
                    self.ram[LINK_FORCE_HOLD_SWORD_UP] = 0;
                    self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
                }
                self.ram[ITEM_RECEIPT_METHOD] = 3;
                self.link_receive_item(self.ram[ANCILLA_ITEM_TO_LINK + k], 0);
                return;
            }

            if self.ram[ANCILLA_STEP + k] != 2 {
                if self.ram[ANCILLA_STEP + k] != 0 {
                    self.ram[ANCILLA_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + k].wrapping_sub(1);
                }
                self.ancilla_move_z(k);
                if self.ram[ANCILLA_Z + k] >= 0xf8 {
                    self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                    self.ram[ANCILLA_Z_VEL + k] = 0x18;
                    self.ram[ANCILLA_Z + k] = 0;
                }
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let oam =
            self.ancilla_receive_item_draw(k, x, y.wrapping_sub(self.ram[ANCILLA_Z + k] as u16));

        let aux_timer = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        self.ram[ANCILLA_AUX_TIMER + k] = aux_timer;
        if sign8(aux_timer) {
            self.ram[ANCILLA_AUX_TIMER + k] = 9;
            self.ram[ANCILLA_L + k] = self.ram[ANCILLA_L + k].wrapping_add(1);
            if self.ram[ANCILLA_L + k] == 3 {
                self.ram[ANCILLA_L + k] = 0;
            }
        }

        let t = if self.ram[ANCILLA_Z + k] == 0 {
            if self.world_state_view().dungeon_room() == 6 {
                self.ram[ANCILLA_L + k].wrapping_add(4)
            } else {
                0
            }
        } else if self.ram[ANCILLA_Z + k] < 0x20 {
            1
        } else {
            2
        };
        self.ancilla_draw_shadow(oam, t as usize, x, y.wrapping_add(12), 0x20);
    }

    fn ancilla28_wish_pond_item(&mut self, k: usize) {
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 0x10);

        if self.frame_control_view().submodule() == 0 && self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[LINK_PICKING_THROW_STATE] = 2;
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[ANCILLA_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + k].wrapping_sub(2);
            self.ancilla_move_z(k);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            if sign8(self.ram[ANCILLA_Z + k]) && self.ram[ANCILLA_Z + k] < 228 {
                self.ram[ANCILLA_Z + k] = 228;
                let j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
                self.ancilla_set_xy(
                    k,
                    self.ancilla_get_x(k)
                        .wrapping_add(if self.asset_u8(71, j) != 0 { 8 } else { 4 }),
                    self.ancilla_get_y(k).wrapping_add(18),
                );
                self.ancilla_transmute_to_splash(k);
                return;
            }
        }
        self.wish_pond_item_draw(k);
    }

    fn ancilla42_happiness_pond_rupees(&mut self, k: usize) {
        self.ram[LINK_PICKING_THROW_STATE] = 2;
        self.ram[LINK_STATE_BITS] = 0;
        for i in (0..=9).rev() {
            if self.ram[HAPPINESS_POND_ARR1 + i] != 0 {
                self.hapiness_pond_rupees_execute_rupee(k, i);
                if self.ram[HAPPINESS_POND_STEP + i] == 2 {
                    self.ram[HAPPINESS_POND_ARR1 + i] = 0;
                }
            }
        }
        for i in (0..=9).rev() {
            if self.ram[HAPPINESS_POND_ARR1 + i] != 0 {
                return;
            }
        }
        self.ram[ANCILLA_TYPE + k] = 0;
    }

    fn hapiness_pond_rupees_execute_rupee(&mut self, k: usize, i: usize) {
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 0x10);
        self.hapiness_pond_rupees_get_state(k, i);

        if self.ram[ANCILLA_STEP + k] != 0 {
            if self.frame_control_view().submodule() == 0 && self.ram[ANCILLA_TIMER + k] == 0 {
                self.ram[ANCILLA_TIMER + k] = 6;
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                if self.ram[ANCILLA_ITEM_TO_LINK + k] == 5 {
                    self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                } else {
                    self.object_splash_draw(k);
                }
            } else {
                self.object_splash_draw(k);
            }
        } else if self.frame_control_view().submodule() == 0 && self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + k].wrapping_sub(2);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            self.ancilla_move_z(k);
            if sign8(self.ram[ANCILLA_Z + k]) && self.ram[ANCILLA_Z + k] < 0xe4 {
                self.ram[ANCILLA_Z + k] = 0xe4;
                self.ancilla_set_xy(
                    k,
                    self.ancilla_get_x(k).wrapping_sub(4),
                    self.ancilla_get_y(k).wrapping_add(30),
                );
                self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
                self.ram[ANCILLA_TIMER + k] = 6;
                self.ancilla_sfx2_pan(k, 0x28);
                self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                self.object_splash_draw(k);
            } else {
                self.ram[ANCILLA_ARR4 + k] = 2;
                self.ram[ANCILLA_FLOOR + k] = 0;
                self.wish_pond_item_draw(k);
            }
        } else {
            self.ram[ANCILLA_ARR4 + k] = 2;
            self.ram[ANCILLA_FLOOR + k] = 0;
            self.wish_pond_item_draw(k);
        }
        self.hapiness_pond_rupees_save_state(i, k);
    }

    fn hapiness_pond_rupees_get_state(&mut self, j: usize, k: usize) {
        self.ram[ANCILLA_Y_LO + j] = self.ram[HAPPINESS_POND_Y_LO + k];
        self.ram[ANCILLA_Y_HI + j] = self.ram[HAPPINESS_POND_Y_HI + k];
        self.ram[ANCILLA_X_LO + j] = self.ram[HAPPINESS_POND_X_LO + k];
        self.ram[ANCILLA_X_HI + j] = self.ram[HAPPINESS_POND_X_HI + k];
        self.ram[ANCILLA_Z + j] = self.ram[HAPPINESS_POND_Z + k];
        self.ram[ANCILLA_Y_VEL + j] = self.ram[HAPPINESS_POND_Y_VEL + k];
        self.ram[ANCILLA_X_VEL + j] = self.ram[HAPPINESS_POND_X_VEL + k];
        self.ram[ANCILLA_Z_VEL + j] = self.ram[HAPPINESS_POND_Z_VEL + k];
        self.ram[ANCILLA_Y_SUBPIXEL + j] = self.ram[HAPPINESS_POND_Y_SUBPIXEL + k];
        self.ram[ANCILLA_X_SUBPIXEL + j] = self.ram[HAPPINESS_POND_X_SUBPIXEL + k];
        self.ram[ANCILLA_Z_SUBPIXEL_PLAYER + j] = self.ram[HAPPINESS_POND_Z_SUBPIXEL + k];
        self.ram[ANCILLA_ITEM_TO_LINK + j] = self.ram[HAPPINESS_POND_ITEM_TO_LINK + k];
        self.ram[ANCILLA_STEP + j] = self.ram[HAPPINESS_POND_STEP + k];
        self.ram[ANCILLA_TIMER + j] = self.ram[HAPPINESS_POND_TIMER + k].saturating_sub(1);
    }

    fn hapiness_pond_rupees_save_state(&mut self, k: usize, j: usize) {
        self.ram[HAPPINESS_POND_Y_LO + k] = self.ram[ANCILLA_Y_LO + j];
        self.ram[HAPPINESS_POND_Y_HI + k] = self.ram[ANCILLA_Y_HI + j];
        self.ram[HAPPINESS_POND_X_LO + k] = self.ram[ANCILLA_X_LO + j];
        self.ram[HAPPINESS_POND_X_HI + k] = self.ram[ANCILLA_X_HI + j];
        self.ram[HAPPINESS_POND_Z + k] = self.ram[ANCILLA_Z + j];
        self.ram[HAPPINESS_POND_Y_VEL + k] = self.ram[ANCILLA_Y_VEL + j];
        self.ram[HAPPINESS_POND_X_VEL + k] = self.ram[ANCILLA_X_VEL + j];
        self.ram[HAPPINESS_POND_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + j];
        self.ram[HAPPINESS_POND_Y_SUBPIXEL + k] = self.ram[ANCILLA_Y_SUBPIXEL + j];
        self.ram[HAPPINESS_POND_X_SUBPIXEL + k] = self.ram[ANCILLA_X_SUBPIXEL + j];
        self.ram[HAPPINESS_POND_Z_SUBPIXEL + k] = self.ram[ANCILLA_Z_SUBPIXEL_PLAYER + j];
        self.ram[HAPPINESS_POND_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + j];
        self.ram[HAPPINESS_POND_TIMER + k] = self.ram[ANCILLA_TIMER + j];
        self.ram[HAPPINESS_POND_STEP + k] = self.ram[ANCILLA_STEP + j];
    }

    fn ancilla3_c_spin_attack_charge_sparkle(&mut self, k: usize) {
        const SWORD_CHARGE_SPARK_CHAR: [u8; 3] = [0xb7, 0x80, 0x83];
        const SWORD_CHARGE_SPARK_FLAGS: [u8; 3] = [4, 4, 0x84];

        if self.frame_control_view().submodule() == 0 && self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TIMER + k] = 4;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 3 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }
        self.ram[ANCILLA_OAM_IDX + k] =
            self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 4) as u8;
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
        self.ancilla_set_oam(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            x,
            y,
            SWORD_CHARGE_SPARK_CHAR[j],
            SWORD_CHARGE_SPARK_FLAGS[j] | self.ram[OAM_PRIORITY_VALUE + 1],
            0,
        );
    }

    fn ancilla2_e_somaria_block_fission(&mut self, k: usize) {
        const SOMARIAN_BLOCK_DIVIDE_X: [i8; 16] =
            [-8, 0, -8, 0, -10, -10, 2, 2, -8, 0, -8, 0, -12, -12, 4, 4];
        const SOMARIAN_BLOCK_DIVIDE_Y: [i8; 16] =
            [-10, -10, 2, 2, -8, 0, -8, 0, -12, -12, 4, 4, -8, 0, -8, 0];
        const SOMARIAN_BLOCK_DIVIDE_CHAR: [u8; 16] = [
            0xc6, 0xc6, 0xc6, 0xc6, 0xc4, 0xc4, 0xc4, 0xc4, 0xd2, 0xd2, 0xd2, 0xd2, 0xc5, 0xc5,
            0xc5, 0xc5,
        ];
        const SOMARIAN_BLOCK_DIVIDE_FLAGS: [u8; 16] = [
            0xc6, 0x86, 0x46, 6, 0x46, 0xc6, 6, 0x86, 0xc6, 0x86, 0x46, 6, 0x46, 0xc6, 6, 0x86,
        ];

        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if (self.ram[ANCILLA_AUX_TIMER + k] as i8) < 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = 3;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 2 {
                self.ram[ANCILLA_TYPE + k] = 0;
                self.somaria_block_spawn_bullets(k);
                return;
            }
        }
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;

        let z = self.ram[ANCILLA_Z + k].wrapping_add(
            if self.ram[ANCILLA_K + k] == 3 && self.ram[LINK_Z_COORD] != 0xff {
                self.ram[LINK_Z_COORD]
            } else {
                0
            },
        );
        let mut j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 8;
        for _ in 0..8 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(SOMARIAN_BLOCK_DIVIDE_X[j] as i16 as u16),
                y.wrapping_add(SOMARIAN_BLOCK_DIVIDE_Y[j] as i16 as u16)
                    .wrapping_sub(z as i8 as i16 as u16),
                SOMARIAN_BLOCK_DIVIDE_CHAR[j],
                SOMARIAN_BLOCK_DIVIDE_FLAGS[j] & !0x30 | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
            j += 1;
            oam += 4;
        }
    }

    fn ancilla2_f_lamp_flame(&mut self, k: usize) {
        const LAMP_FLAME_DRAW_CHAR: [u8; 12] = [
            0x9c, 0x9c, 0xff, 0xff, 0xa4, 0xa5, 0xb2, 0xb3, 0xe3, 0xf3, 0xff, 0xff,
        ];
        const LAMP_FLAME_DRAW_Y: [i8; 12] = [-3, 0, 0, 0, 0, 0, 8, 8, 0, 8, 0, 0];
        const LAMP_FLAME_DRAW_X: [i8; 12] = [4, 10, 0, 0, 1, 9, 2, 7, 4, 4, 0, 0];

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TYPE + k] = 0;
            return;
        }
        let mut j = ((self.ram[ANCILLA_TIMER + k] & 0xf8) >> 1) as usize;
        loop {
            if LAMP_FLAME_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(LAMP_FLAME_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(LAMP_FLAME_DRAW_Y[j] as i16 as u16),
                    LAMP_FLAME_DRAW_CHAR[j],
                    self.ram[OAM_PRIORITY_VALUE + 1] | 2,
                    0,
                );
                oam += 4;
            }
            j += 1;
            if j & 3 == 0 {
                break;
            }
        }
    }

    fn ancilla41_waterfall_splash(&mut self, k: usize) {
        const WATERFALL_SPLASH_X: [i8; 8] = [0, 0, -4, 4, -7, 7, -9, 17];
        const WATERFALL_SPLASH_Y: [i8; 8] = [-4, 0, -5, -5, -3, -3, 12, 12];
        const WATERFALL_SPLASH_CHAR: [u8; 8] = [0xc0, 0xff, 0xac, 0xac, 0xae, 0xae, 0xbf, 0xbf];
        const WATERFALL_SPLASH_FLAGS: [u8; 8] = [0x84, 0xff, 0x84, 0xc4, 0x84, 0xc4, 0x84, 0xc4];
        const WATERFALL_SPLASH_EXT: [u8; 8] = [2, 0xff, 2, 2, 2, 2, 0, 0];

        if !self.ancilla_check_for_entrance_trigger(if self.ram[PLAYER_IS_INDOORS] != 0 {
            0
        } else {
            1
        }) {
            self.ram[ANCILLA_TYPE + k] = 0;
            return;
        }

        if self.frame_control_view().submodule() == 0 && self.ram[FRAME_COUNTER] & 7 == 0 {
            self.ancilla_sfx2_near(0x1c);
        }

        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 1;
        if !sign8(self.ram[LINK_ANIMATION_STEPS].wrapping_sub(6)) {
            self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_sub(6);
        }

        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TIMER + k] = 2;
            self.ram[ANCILLA_ITEM_TO_LINK + k] =
                self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1) & 3;
        }

        if self.ram[PLAYER_IS_INDOORS] != 0 && self.ram[LINK_Y_COORD] < 0x38 {
            self.ancilla_set_y(k, 0x0d38);
        } else {
            self.ancilla_set_y(k, self.player_state_view().y());
        }
        self.ancilla_set_x(k, self.player_state_view().x());

        let (x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let z = self.ram[LINK_Z_COORD];
        y = y.wrapping_sub(if sign8(z) { 0 } else { z } as u16);

        let mut j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 2;
        for _ in 0..2 {
            if WATERFALL_SPLASH_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(WATERFALL_SPLASH_X[j] as i16 as u16),
                    y.wrapping_add(WATERFALL_SPLASH_Y[j] as i16 as u16),
                    WATERFALL_SPLASH_CHAR[j],
                    WATERFALL_SPLASH_FLAGS[j] | 0x30,
                    WATERFALL_SPLASH_EXT[j],
                );
            }
            j += 1;
            oam += 4;
        }
    }

    fn ancilla3_d_item_splash(&mut self, k: usize) {
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 8);
        if self.frame_control_view().submodule() == 0 && self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TIMER + k] = 6;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 5 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }
        self.object_splash_draw(k);
    }

    fn ancilla15_jump_splash(&mut self, k: usize) {
        const ANCILLA_JUMP_SPLASH_CHAR: [u8; 2] = [0xac, 0xae];

        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
                self.ram[ANCILLA_AUX_TIMER + k] = 0;
                self.ram[ANCILLA_ITEM_TO_LINK + k] = 1;
            }
            if self.ram[ANCILLA_ITEM_TO_LINK + k] != 0 {
                self.ram[ANCILLA_Y_VEL + k] = self.ram[ANCILLA_Y_VEL + k].wrapping_sub(4);
                self.ram[ANCILLA_X_VEL + k] = self.ram[ANCILLA_Y_VEL + k];
                if self.ram[ANCILLA_Y_VEL + k] < 232 {
                    self.ram[ANCILLA_TYPE + k] = 0;
                    if (self.ram[LINK_IS_BUNNY_MIRROR] != 0
                        || self.ram[LINK_PLAYER_HANDLER_STATE] == 4)
                        && self.ram[LINK_IS_IN_DEEP_WATER] != 0
                    {
                        self.check_ability_to_swim();
                    }
                    return;
                }
                self.ancilla_move_x(k);
                self.ancilla_move_y(k);
            }
        }

        let (mut x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let ax = self.ancilla_get_x(k);
        let x8 = self
            .player_state_view()
            .x()
            .wrapping_mul(2)
            .wrapping_sub(ax)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let x6 = ax
            .wrapping_add(12)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize;
        let mut flags = 0;
        for _ in 0..2 {
            self.ancilla_set_oam(oam, x, y, ANCILLA_JUMP_SPLASH_CHAR[j], 0x24 | flags, 2);
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
            x = x8;
            flags = 0x40;
        }
        self.ancilla_set_oam(oam, x6, y, 0xc0, 0x24, if j == 1 { 1 } else { 2 });
    }

    fn ancilla04_beam_hit(&mut self, k: usize) {
        const BEAM_HIT_X: [i8; 16] = [-12, 20, -12, 20, -8, 16, -8, 16, -4, 12, -4, 12, 0, 8, 0, 8];
        const BEAM_HIT_Y: [i8; 16] = [-12, -12, 20, 20, -8, -8, 16, 16, -4, -4, 12, 12, 0, 0, 8, 8];
        const BEAM_HIT_CHAR: [u8; 16] = [
            0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x54, 0x54,
            0x54, 0x54,
        ];
        const BEAM_HIT_FLAGS: [u8; 16] = [
            0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0, 0x40, 0x80, 0xc0,
        ];

        let Some(info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };
        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TYPE + k] = 0;
            return;
        }

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = (self.ram[ANCILLA_TIMER + k] >> 1) as usize;
        let ancilla_x = self.ancilla_get_x(k);
        let ancilla_y = self.ancilla_get_y(k);
        let r7 = ancilla_x.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)) as u8;
        let r6 = ancilla_y.wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)) as u8;
        for i in (0..=3).rev() {
            let m = j * 4 + i;
            let x = info.x.wrapping_add(BEAM_HIT_X[m] as u8);
            let y = info.y.wrapping_add(BEAM_HIT_Y[m] as u8);
            let x_adj = ancilla_x
                .wrapping_add(x.wrapping_sub(r7) as i8 as i16 as u16)
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
            let y_adj = ancilla_y
                .wrapping_add(y.wrapping_sub(r6) as i8 as i16 as u16)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
                .wrapping_add(0x10);
            self.ram[oam] = x;
            self.ram[oam + 1] = if y_adj >= 0x100 { 0xf0 } else { y };
            self.ram[oam + 2] = BEAM_HIT_CHAR[m].wrapping_add(0x82);
            self.ram[oam + 3] = BEAM_HIT_FLAGS[m] | 2 | info.flags;
            self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4] =
                if x_adj >= 0x100 { 1 } else { 0 };
            oam += 4;
        }
    }

    fn ancilla13_ice_rod_sparkle(&mut self, k: usize) {
        const ICE_SHOT_SPARKLE_X: [u8; 16] = [2, 7, 6, 1, 1, 7, 7, 1, 0, 7, 8, 1, 4, 9, 4, 0xff];
        const ICE_SHOT_SPARKLE_Y: [u8; 16] = [2, 3, 8, 7, 1, 1, 7, 7, 1, 0, 7, 8, 0xff, 4, 9, 4];
        const ICE_SHOT_SPARKLE_CHAR: [u8; 16] = [
            0x83, 0x83, 0x83, 0x83, 0xb6, 0x80, 0xb6, 0x80, 0xb7, 0xb6, 0xb7, 0xb6, 0xb7, 0xb6,
            0xb7, 0xb6,
        ];

        if self.ram[ANCILLA_TIMER + k] == 0 {
            self.ram[ANCILLA_TYPE + k] = 0;
        }
        if self.frame_control_view().submodule() == 0 {
            self.ancilla_move_x(k);
            self.ancilla_move_y(k);
        }
        let Some(mut info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };

        let mut j = 4i32;
        while j >= 0 && self.ram[ANCILLA_TYPE + j as usize] != 0x0b {
            j -= 1;
        }
        if j >= 0 && self.ram[ANCILLA_OBJPRIO + j as usize] != 0 {
            info.flags = 0x30;
        }

        if self.ram[SORT_SPRITES_SETTING] != 0 {
            if self.ram[ANCILLA_FLOOR + k] != 0 {
                self.oam_allocate_from_region_e(0x10);
            } else {
                self.oam_allocate_from_region_d(0x10);
            }
        } else {
            self.oam_allocate_from_region_a(0x10);
        }

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        j = (self.ram[ANCILLA_TIMER + k] & 0x1c) as i32;
        for i in (0..=3).rev() {
            let n = i + j as usize;
            self.ram[oam] = info.x.wrapping_add(ICE_SHOT_SPARKLE_X[n]);
            self.ram[oam + 1] = info.y.wrapping_add(ICE_SHOT_SPARKLE_Y[n]);
            self.ram[oam + 2] = ICE_SHOT_SPARKLE_CHAR[n];
            self.ram[oam + 3] = info.flags | 4;
            self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4] = 0;
            oam += 4;
        }
    }

    fn ancilla_add_ice_rod_sparkle(&mut self, k: usize) {
        const ICE_SHOT_SPARKLE_XVEL: [i8; 4] = [0, 0, -4, 4];
        const ICE_SHOT_SPARKLE_YVEL: [i8; 4] = [-4, 4, 0, 0];

        if self.frame_control_view().submodule() != 0 {
            return;
        }
        self.ram[ANCILLA_ARR4 + k] = self.ram[ANCILLA_ARR4 + k].wrapping_sub(1);
        if !sign8(self.ram[ANCILLA_ARR4 + k]) {
            return;
        }

        self.ram[ANCILLA_ARR4 + k] = 5;
        if let Some(j) = self.ancilla_alloc_high() {
            self.ram[ANCILLA_TYPE + j] = 0x13;
            self.ram[ANCILLA_TIMER + j] = 15;

            let i = self.ram[ANCILLA_DIR + k] as usize;
            self.ram[ANCILLA_X_VEL + j] = ICE_SHOT_SPARKLE_XVEL[i] as u8;
            self.ram[ANCILLA_Y_VEL + j] = ICE_SHOT_SPARKLE_YVEL[i] as u8;

            self.ram[ANCILLA_X_LO + j] = self.ram[ANCILLA_X_LO + k];
            self.ram[ANCILLA_Y_LO + j] = self.ram[ANCILLA_Y_LO + k];
            self.ram[ANCILLA_FLOOR + j] = self.ram[ANCILLA_FLOOR + k];
            self.ram[ANCILLA_NUMSPR + j] = 0;
        }
    }

    pub(super) fn ancilla_add_simple(&mut self, ty: u8, limit: u8) -> Option<usize> {
        self.ancilla_add_ancilla(ty, limit)
    }

    fn ancilla_add_ancilla(&mut self, a: u8, y: u8) -> Option<usize> {
        let k = self.ancilla_alloc_init(a, y)?;
        self.ram[ANCILLA_TYPE + k] = a;
        self.ram[ANCILLA_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[ANCILLA_FLOOR2 + k] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
        self.ram[ANCILLA_Y_VEL + k] = 0;
        self.ram[ANCILLA_X_VEL + k] = 0;
        self.ram[ANCILLA_OBJPRIO + k] = 0;
        self.ram[ANCILLA_U + k] = 0;
        self.ram[ANCILLA_NUMSPR + k] = K_ANCILLA_PFLAGS[a as usize];
        Some(k)
    }

    fn ancilla_alloc_high(&self) -> Option<usize> {
        (0..=9).rev().find(|&k| self.ram[ANCILLA_TYPE + k] == 0)
    }

    pub(super) fn ancilla_alloc_init(&mut self, ty: u8, limit: u8) -> Option<usize> {
        if self.ram[RAM_BUGS_FIXED] >= BUGFIX_POLY_RENDERER {
            self.ram[R14] = limit.wrapping_add(1);
        }

        let n = (0..5).filter(|&i| self.ram[ANCILLA_TYPE + i] == ty).count();
        if limit as usize + 1 == n {
            return None;
        }

        let start = if ty == 7 || ty == 8 {
            limit as usize
        } else {
            4
        };
        for j in (0..=start).rev() {
            if self.ram[ANCILLA_TYPE + j] == 0 {
                return Some(j);
            }
        }

        let mut k = self.ram[ANCILLA_ALLOC_ROTATE_PLAYER] as i8;
        loop {
            k -= 1;
            if k < 0 {
                k = limit as i8;
            }
            let old_type = self.ram[ANCILLA_TYPE + k as usize];
            if old_type == 0x3c || old_type == 0x13 || old_type == 0x0a {
                self.ram[ANCILLA_ALLOC_ROTATE_PLAYER] = k as u8;
                return Some(k as usize);
            }
            if k == 0 {
                break;
            }
        }
        self.ram[ANCILLA_ALLOC_ROTATE_PLAYER] = 0;
        None
    }

    fn ancilla_add_add_ancilla_bank08(&mut self, ty: u8, y: u8) -> Option<usize> {
        self.ancilla_add_simple(ty, y)
    }

    pub(super) fn ancilla_check_link_collision(&self, k: usize, j: usize) -> bool {
        self.ancilla_check_link_collision_out(k, j).is_some()
    }

    fn ancilla_check_link_collision_out(&self, k: usize, j: usize) -> Option<CheckPlayerCollOut> {
        const YOFFS: [u16; 5] = [0, 8, 8, 8, 0];
        const XOFFS: [u16; 5] = [0, 8, 8, 8, 0];
        const H: [u16; 5] = [20, 20, 8, 28, 14];
        const W: [u16; 5] = [20, 3, 8, 24, 14];
        const LINK_YOFFS: [u16; 5] = [12, 12, 12, 12, 12];
        const LINK_XOFFS: [u16; 5] = [8, 8, 8, 12, 8];

        let y = self
            .ancilla_y(k)
            .wrapping_add(YOFFS[j])
            .wrapping_add(self.ram[ANCILLA_Z + k] as i8 as i16 as u16);
        let x = self.ancilla_x(k).wrapping_add(XOFFS[j]);
        let r4 = self
            .player_state_view()
            .y()
            .wrapping_add(LINK_YOFFS[j])
            .wrapping_sub(y);
        let r6 = self
            .player_state_view()
            .x()
            .wrapping_add(LINK_XOFFS[j])
            .wrapping_sub(x);
        let r8 = abs16(r4);
        let r10 = abs16(r6);
        if r8 < H[j] && r10 < W[j] {
            Some(CheckPlayerCollOut { r4, r6, r8, r10 })
        } else {
            None
        }
    }

    fn ancilla_check_tile_collision(&mut self, k: usize) -> u8 {
        if self.ram[PLAYER_IS_INDOORS] == 0 && self.ram[ANCILLA_OBJPRIO + k] != 0 {
            self.ram[ANCILLA_TILE_ATTR_PLAYER + k] = 0;
            return 0;
        }
        if self.ram[DUNG_HDR_COLLISION] == 0 {
            return self.ancilla_check_tile_collision_one_floor(k) as u8;
        }

        let mut x = 0u16;
        let mut y = 0u16;
        if self.ram[DUNG_HDR_COLLISION] < 3 {
            x = read_le_u16(&self.ram, BG1HOFS_COPY2)
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
            y = read_le_u16(&self.ram, BG1VOFS_COPY2)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        }
        let oldx = self.ancilla_get_x(k);
        let oldy = self.ancilla_get_y(k);
        self.ancilla_set_xy(k, oldx.wrapping_add(x), oldy.wrapping_add(y));
        self.ram[ANCILLA_FLOOR + k] = 1;
        let b = self.ancilla_check_tile_collision_one_floor(k) as u8;
        self.ram[ANCILLA_FLOOR + k] = 0;
        self.ancilla_set_xy(k, oldx, oldy);
        (b << 1) | self.ancilla_check_tile_collision_one_floor(k) as u8
    }

    fn ancilla_check_tile_collision_staggered(&mut self, k: usize) -> u8 {
        if (self.ram[FRAME_COUNTER] ^ k as u8) & 1 != 0 {
            self.ancilla_check_tile_collision(k)
        } else {
            0
        }
    }

    fn ancilla_check_tile_collision_one_floor(&mut self, k: usize) -> bool {
        const CHECK_TILE_COLL0_X: [i8; 20] = [
            8, 8, 0, 16, 4, 4, 0, 16, 4, 4, 4, 12, 12, 12, 4, 12, 0, 0, 0, 0,
        ];
        const CHECK_TILE_COLL0_Y: [i8; 20] = [
            0, 16, 5, 5, 0, 16, 4, 4, 4, 12, 5, 5, 4, 12, 12, 12, 0, 0, 0, 0,
        ];
        let j = self.ram[ANCILLA_DIR + k] as usize;
        let x = self
            .ancilla_get_x(k)
            .wrapping_add(CHECK_TILE_COLL0_X[j] as i16 as u16);
        let y = self
            .ancilla_get_y(k)
            .wrapping_add(CHECK_TILE_COLL0_Y[j] as i16 as u16);
        self.ancilla_check_tile_collision_targeted(k, x, y)
    }

    fn ancilla_check_initial_tile_a(&mut self, k: usize) -> i32 {
        const YOFFS_HB: [i8; 12] = [8, 0, -8, 8, 16, 24, 8, 8, 8, 8, 8, 8];
        const XOFFS_HB: [i8; 12] = [0, 0, 0, 0, 0, 0, 0, -8, -16, 0, 8, 16];

        let mut j = self.ram[ANCILLA_DIR + k] as usize * 3;
        let mut i = 2i32;
        loop {
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(XOFFS_HB[j] as i16 as u16);
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(YOFFS_HB[j] as i16 as u16);
            self.ancilla_set_xy(k, x, y);
            if self.ancilla_check_tile_collision(k) != 0 {
                break;
            }
            i -= 1;
            if i < 0 {
                break;
            }
            j += 1;
        }
        i
    }

    fn ancilla_return_if_outside_bounds(&mut self, k: usize) -> Option<AncillaOamInfo> {
        const ANCILLA_FLOOR_FLAGS: [u8; 2] = [0x20, 0x10];
        let info = AncillaOamInfo {
            x: self.ram[ANCILLA_X_LO + k].wrapping_sub(self.ram[BG2HOFS_COPY2]),
            y: self.ram[ANCILLA_Y_LO + k].wrapping_sub(self.ram[BG2VOFS_COPY2]),
            flags: ANCILLA_FLOOR_FLAGS[self.ram[ANCILLA_FLOOR + k] as usize],
        };
        if info.x >= 0xf4 || info.y >= 0xf0 {
            self.ram[ANCILLA_TYPE + k] = 0;
            None
        } else {
            Some(info)
        }
    }

    fn ancilla_apply_conveyor(&mut self, k: usize) {
        const ANCILLA_BELT_XVEL: [i8; 4] = [0, 0, -8, 8];
        const ANCILLA_BELT_YVEL: [i8; 4] = [-8, 8, 0, 0];
        let j = self.ram[ANCILLA_TILE_ATTR_PLAYER + k].wrapping_sub(0x68) as usize;
        self.ram[ANCILLA_Y_VEL + k] = ANCILLA_BELT_YVEL[j] as u8;
        self.ram[ANCILLA_X_VEL + k] = ANCILLA_BELT_XVEL[j] as u8;
        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
    }

    fn ancilla_project_speed_towards_player(&self, k: usize, mut vel: u8) -> ProjectSpeedRet {
        if vel == 0 {
            return ProjectSpeedRet {
                x: 0,
                y: 0,
                xdiff: 0,
                ydiff: 0,
            };
        }
        let below = self.ancilla_is_below_link(k);
        let mut r12 = if (below.b as i8).is_negative() {
            0u8.wrapping_sub(below.b)
        } else {
            below.b
        };

        let right = self.ancilla_is_right_of_link(k);
        let mut r13 = if (right.b as i8).is_negative() {
            0u8.wrapping_sub(right.b)
        } else {
            right.b
        };
        let mut swapped = false;
        if r13 < r12 {
            swapped = true;
            std::mem::swap(&mut r12, &mut r13);
        }
        let mut xvel = vel;
        let mut yvel = 0u8;
        let mut t = 0u8;
        loop {
            t = t.wrapping_add(r12);
            if t >= r13 {
                t = t.wrapping_sub(r13);
                yvel = yvel.wrapping_add(1);
            }
            vel = vel.wrapping_sub(1);
            if vel == 0 {
                break;
            }
        }
        if swapped {
            std::mem::swap(&mut xvel, &mut yvel);
        }
        ProjectSpeedRet {
            x: if right.a != 0 {
                0u8.wrapping_sub(xvel)
            } else {
                xvel
            },
            y: if below.a != 0 {
                0u8.wrapping_sub(yvel)
            } else {
                yvel
            },
            xdiff: right.b,
            ydiff: below.b,
        }
    }

    fn ancilla_get_radial_projection(&self, a: u8, r8: u8) -> AncillaRadialProjection {
        const RADIAL_PROJECTION_TAB0: [u8; 64] = [
            255, 254, 251, 244, 236, 225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49,
            74, 97, 120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254, 251, 244, 236,
            225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49, 74, 97, 120, 142, 162,
            181, 197, 212, 225, 236, 244, 251, 254,
        ];
        const RADIAL_PROJECTION_TAB2: [u8; 64] = [
            0, 25, 49, 74, 97, 120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254,
            251, 244, 236, 225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49, 74, 97,
            120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254, 251, 244, 236, 225,
            212, 197, 181, 162, 142, 120, 97, 74, 49, 25,
        ];
        const RADIAL_PROJECTION_TAB1: [u8; 64] = [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        const RADIAL_PROJECTION_TAB3: [u8; 64] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        let a = a as usize;
        let p0 = u16::from(RADIAL_PROJECTION_TAB0[a]) * u16::from(r8);
        let p1 = u16::from(RADIAL_PROJECTION_TAB2[a]) * u16::from(r8);
        AncillaRadialProjection {
            r0: ((p0 >> 8) + ((p0 >> 7) & 1)) as u8,
            r2: RADIAL_PROJECTION_TAB1[a],
            r4: ((p1 >> 8) + ((p1 >> 7) & 1)) as u8,
            r6: RADIAL_PROJECTION_TAB3[a],
        }
    }

    fn sparkle_prep_oam_from_radial(&self, p: AncillaRadialProjection) -> Point16U {
        Point16U {
            y: read_le_u16(&self.ram, SWORDBEAM_TEMP_Y)
                .wrapping_add(if p.r2 != 0 {
                    -(p.r0 as i16)
                } else {
                    p.r0 as i16
                } as u16)
                .wrapping_sub(4)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)),
            x: read_le_u16(&self.ram, SWORDBEAM_TEMP_X)
                .wrapping_add(if p.r6 != 0 {
                    -(p.r4 as i16)
                } else {
                    p.r4 as i16
                } as u16)
                .wrapping_sub(4)
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)),
        }
    }

    fn ancilla_is_right_of_link(&self, k: usize) -> PairU8 {
        let x = self
            .player_state_view()
            .x()
            .wrapping_sub(self.ancilla_get_x(k));
        PairU8 {
            a: u8::from((x as i16).is_negative()),
            b: x as u8,
        }
    }

    fn ancilla_is_below_link(&self, k: usize) -> PairU8 {
        let y = self
            .player_state_view()
            .y()
            .wrapping_sub(self.ancilla_get_y(k));
        PairU8 {
            a: u8::from((y as i16).is_negative()),
            b: y as u8,
        }
    }

    fn ancilla_transmute_to_splash(&mut self, k: usize) {
        self.ram[ANCILLA_TYPE + k] = 0x3d;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_TIMER + k] = 6;
        self.ancilla_set_xy(
            k,
            self.ancilla_get_x(k).wrapping_sub(8),
            self.ancilla_get_y(k).wrapping_add(12),
        );
        self.ancilla_sfx2_pan(k, 0x28);
        self.ancilla3_d_item_splash(k);
    }

    fn object_splash_draw(&mut self, k: usize) {
        const OBJECT_SPLASH_DRAW_X: [i8; 10] = [0, 0, 0, 0, 11, -3, 15, -7, 15, -7];
        const OBJECT_SPLASH_DRAW_Y: [i8; 10] = [0, 0, -6, 0, -13, -8, -17, -4, -17, -4];
        const OBJECT_SPLASH_DRAW_CHAR: [u8; 10] =
            [0xc0, 0xff, 0xe7, 0xff, 0xaf, 0xbf, 0x80, 0x80, 0x83, 0x83];
        const OBJECT_SPLASH_DRAW_FLAGS: [u8; 10] = [0, 0xff, 0, 0xff, 0x40, 0, 0x40, 0, 0xc0, 0x80];
        const OBJECT_SPLASH_DRAW_EXT: [u8; 10] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0];
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 2;
        for _ in 0..2 {
            if OBJECT_SPLASH_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(OBJECT_SPLASH_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(OBJECT_SPLASH_DRAW_Y[j] as i16 as u16),
                    OBJECT_SPLASH_DRAW_CHAR[j],
                    OBJECT_SPLASH_DRAW_FLAGS[j] | 0x24,
                    OBJECT_SPLASH_DRAW_EXT[j],
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn ancilla_handle_lift_logic(&mut self, k: usize) {
        const ANCILLA_LIFTABLE_DELAY: [u8; 3] = [16, 8, 9];

        if self.ram[ANCILLA_R_PLAYER + k] != 0 {
            self.ancilla_handle_lift_logic_label_6(k);
            return;
        }
        if self.ram[ANCILLA_L + k] == 0 {
            if self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] == 0 {
                if self.ancilla_handle_lift_logic_clear_pickup_item(k, &ANCILLA_LIFTABLE_DELAY) {
                    return;
                }
            } else {
                if self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] != k as u8 + 1 {
                    return;
                }
                if (self.ram[LINK_DISABLE_SPRITE_DAMAGE] == 0
                    && self.ram[LINK_INCAPACITATED_TIMER] != 0)
                    || self.ram[PLAYER_SPECIAL_DRAW_FLAG] != 0
                    || self.ram[LINK_AUXILIARY_STATE] == 1
                {
                    self.ram[ANCILLA_R_PLAYER + k] = 1;
                    self.ram[ANCILLA_Z_VEL + k] = 0;
                    self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                    self.ram[ANCILLA_ARR4 + k] = 0;
                    self.ancilla_handle_lift_logic_label_6(k);
                    return;
                }
                if self.ram[LINK_STATE_BITS] & 0x80 == 0 {
                    if self.ancilla_handle_lift_logic_clear_pickup_item(k, &ANCILLA_LIFTABLE_DELAY)
                    {
                        return;
                    }
                } else {
                    let mut j = self.ram[ANCILLA_K + k];
                    if self.ram[LINK_PICKING_THROW_STATE] != 2
                        && self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] != 0
                        && j != 3
                    {
                        if j == 0 && self.ram[ANCILLA_AUX_TIMER + k] == 16 {
                            self.ancilla_sfx2_pan(k, 0x1d);
                        }
                        self.ram[ANCILLA_AUX_TIMER + k] =
                            self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
                        if (self.ram[ANCILLA_AUX_TIMER + k] as i8).is_negative() {
                            j = j.wrapping_add(1);
                            self.ram[ANCILLA_K + k] = j;
                            self.ram[ANCILLA_AUX_TIMER + k] = if j == 3 {
                                (-2i8) as u8
                            } else {
                                ANCILLA_LIFTABLE_DELAY[j as usize]
                            };
                            if j == 3 {
                                self.ancilla_latch_altitude_above_link(k);
                                return;
                            }
                        }
                        self.ancilla_latch_link_coordinates(k, j as usize);
                        return;
                    }
                    if j != 3 {
                        return;
                    }

                    if self.ram[LINK_PICKING_THROW_STATE] != 2
                        && (self.frame_control_view().submodule() != 0
                            || ((self.ram[FILTERED_JOYPAD_L] | self.ram[FILTERED_JOYPAD_H]) & 0x80)
                                == 0)
                    {
                        if self.ram[ANCILLA_ITEM_TO_LINK + k] != 0 {
                            return;
                        }
                        if self.ram[PLAYER_NEAR_PIT_STATE] >= 2 {
                            self.ram[LINK_SPEED_SETTING] = 0;
                            if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
                                self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                                self.ram[ANCILLA_TYPE + k] = 0;
                            }
                            return;
                        }
                        if (self.ram[LINK_IS_IN_DEEP_WATER] | self.ram[LINK_IS_BUNNY_MIRROR]) == 0 {
                            self.ancilla_latch_carried_position(k);
                            return;
                        }
                        self.ram[LINK_STATE_BITS] = 0;
                    }
                    const ANCILLA_LIFTABLE_YVEL: [i8; 4] = [-32, 32, 0, 0];
                    const ANCILLA_LIFTABLE_XVEL: [i8; 4] = [0, 0, -32, 32];
                    let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
                    self.ram[ANCILLA_DIR + k] = j as u8;
                    self.ram[ANCILLA_Z_VEL + k] = 24;
                    self.ram[ANCILLA_Y_VEL + k] = ANCILLA_LIFTABLE_YVEL[j] as u8;
                    self.ram[ANCILLA_X_VEL + k] = ANCILLA_LIFTABLE_XVEL[j] as u8;
                    self.ram[LINK_PICKING_THROW_STATE] = 2;
                    self.ram[ANCILLA_L + k] = 1;
                    self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                    self.ram[ANCILLA_ARR4 + k] = 0;
                    self.ram[ANCILLA_K + k] = 0;
                    self.ram[ANCILLA_OBJPRIO + k] = 0;
                    self.ancilla_sfx3_pan(k, 0x13);
                }
            }
        }

        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0 {
            self.ram[ANCILLA_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + k].wrapping_sub(2);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            let old_z = self.ram[ANCILLA_Z + k];
            self.ancilla_move_z(k);
            if self.ram[ANCILLA_ARR4 + k] != 0
                && self.ram[ANCILLA_DIR + k] == 1
                && !(self.ram[ANCILLA_Z + k] as i8).is_negative()
            {
                self.ancilla_set_y(
                    k,
                    self.ancilla_get_y(k).wrapping_add(
                        self.ram[ANCILLA_Z + k].wrapping_sub(old_z) as i8 as i16 as u16
                    ),
                );
            }
            if !(self.ram[ANCILLA_Z + k] as i8).is_negative() || self.ram[ANCILLA_Z + k] == 0xff {
                return;
            }
            self.ram[ANCILLA_Z + k] = 0;
            self.ancilla_sfx2_pan(k, 0x21);
            self.ram[ANCILLA_L + k] = self.ram[ANCILLA_L + k].wrapping_add(1);
            if self.ram[ANCILLA_L + k] != 3 {
                self.ram[ANCILLA_Y_VEL + k] = ((self.ram[ANCILLA_Y_VEL + k] as i8) / 2) as u8;
                self.ram[ANCILLA_X_VEL + k] = ((self.ram[ANCILLA_X_VEL + k] as i8) / 2) as u8;
                self.ram[ANCILLA_Z_VEL + k] = 16;
                self.ram[ANCILLA_ARR4 + k] = 0;
            } else {
                self.ram[ANCILLA_Z + k] = 0;
                self.ram[ANCILLA_L + k] = 0;
                self.ram[ANCILLA_ARR4 + k] = 0;
                self.ram[LINK_SPEED_SETTING] = 0;
                self.ram[ANCILLA_Y_VEL + k] = 0;
                self.ram[ANCILLA_X_VEL + k] = 0;
                self.ram[ANCILLA_Z_VEL + k] = 0;
                if self.ram[ANCILLA_T_PLAYER + k] != 0 {
                    self.ram[ANCILLA_FLOOR + k] = self.ram[ANCILLA_T_PLAYER + k];
                    self.ram[ANCILLA_T_PLAYER + k] = 0;
                }
            }
        }
    }

    fn ancilla_handle_lift_logic_clear_pickup_item(
        &mut self,
        k: usize,
        liftable_delay: &[u8; 3],
    ) -> bool {
        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
        if self.ram[ANCILLA_ITEM_TO_LINK + k] != 0 || self.ram[LINK_STATE_BITS] != 0 {
            return true;
        }
        let Some(coll) = self.ancilla_check_link_collision_out(k, 0) else {
            return true;
        };
        if self.ram[ANCILLA_FLOOR + k] != self.ram[LINK_IS_ON_LOWER_LEVEL] {
            return true;
        }
        if coll.r8 >= 16 || coll.r10 >= 12 {
            let j = if coll.r8 >= coll.r10 {
                if (coll.r4 as i16).is_negative() {
                    1
                } else {
                    0
                }
            } else if (coll.r6 as i16).is_negative() {
                3
            } else {
                2
            };
            if j * 2 != self.ram[LINK_DIRECTION_FACING] {
                return true;
            }
        }
        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = k as u8 + 1;
        self.ram[ANCILLA_K + k] = 0;
        self.ram[ANCILLA_AUX_TIMER + k] = liftable_delay[0];
        self.ram[ANCILLA_L + k] = 0;
        self.ram[ANCILLA_Z + k] = 0;
        true
    }

    fn ancilla_handle_lift_logic_label_6(&mut self, k: usize) {
        if self.ram[ANCILLA_ITEM_TO_LINK + k] != 0 {
            return;
        }
        if self.ram[ANCILLA_K + k] == 3 {
            self.ram[ANCILLA_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + k].wrapping_sub(2);
            self.ancilla_move_z(k);
            if self.ram[ANCILLA_Z + k] != 0 && self.ram[ANCILLA_Z + k] < 252 {
                return;
            }
            self.ram[ANCILLA_Z + k] = 0;
            self.ram[ANCILLA_R_PLAYER + k] = self.ram[ANCILLA_R_PLAYER + k].wrapping_add(1);
            if self.ram[ANCILLA_R_PLAYER + k] != 3 {
                self.ram[ANCILLA_Z_VEL + k] = 24;
                return;
            }
            self.ram[ANCILLA_K + k] = 0;
        }
        self.ram[ANCILLA_R_PLAYER + k] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
    }

    fn ancilla_latch_altitude_above_link(&mut self, k: usize) {
        self.ram[ANCILLA_Z + k] = 17;
        self.ancilla_set_y(k, self.ancilla_get_y(k).wrapping_add(17));
        self.ram[ANCILLA_OBJPRIO + k] = 0;
    }

    fn ancilla_latch_link_coordinates(&mut self, k: usize, mut j: usize) {
        const ANCILLA_FUNC3_X: [i8; 12] = [8, 8, -4, 20, 8, 8, 8, 8, 8, 8, 8, 8];
        const ANCILLA_FUNC3_Y: [i8; 12] = [16, 8, 4, 4, 8, 2, -1, -1, 2, 2, -1, -1];
        j = j * 4 + (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        self.ancilla_set_xy(
            k,
            self.player_state_view()
                .x()
                .wrapping_add(ANCILLA_FUNC3_X[j] as i16 as u16),
            self.player_state_view()
                .y()
                .wrapping_add(ANCILLA_FUNC3_Y[j] as i16 as u16),
        );
    }

    fn ancilla_latch_carried_position(&mut self, k: usize) {
        const ANCILLA_FUNC2_Y: [i8; 6] = [-2, -1, 0, -2, -1, 0];
        self.ram[LINK_SPEED_SETTING] = 12;
        self.ram[ANCILLA_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[ANCILLA_FLOOR2 + k] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
        let mut z = self.player_state_view().z();
        if z == 0xffff {
            z = 0;
        }
        self.ancilla_set_xy(
            k,
            self.player_state_view().x().wrapping_add(8),
            self.player_state_view()
                .y()
                .wrapping_sub(z)
                .wrapping_add(18)
                .wrapping_add(
                    ANCILLA_FUNC2_Y[self.ram[LINK_ANIMATION_STEPS] as usize] as i16 as u16,
                ),
        );
    }

    fn ancilla_latch_y_coord_to_z(&mut self, k: usize) -> u16 {
        let y = self.ancilla_get_y(k);
        let z = self.ram[ANCILLA_Z + k];
        if self.ram[ANCILLA_DIR + k] == 1 && z != 0xff {
            self.ancilla_set_y(k, y.wrapping_sub(z as i8 as i16 as u16));
        }
        y
    }

    pub(super) fn ancilla_check_tile_collision_class2(&mut self, k: usize) -> bool {
        if self.ram[DUNG_HDR_COLLISION] == 0 {
            return self.ancilla_check_tile_collision_class2_inner(k);
        }

        let mut x = 0u16;
        let mut y = 0u16;
        if self.ram[DUNG_HDR_COLLISION] < 3 {
            x = read_le_u16(&self.ram, BG1HOFS_COPY2)
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
            y = read_le_u16(&self.ram, BG1VOFS_COPY2)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        }

        let oldx = self.ancilla_x(k);
        let oldy = self.ancilla_y(k);
        self.ancilla_set_xy(k, oldx.wrapping_add(x), oldy.wrapping_add(y));
        self.ram[ANCILLA_FLOOR + k] = 1;
        let b = self.ancilla_check_tile_collision_class2_inner(k);
        self.ram[ANCILLA_FLOOR + k] = 0;
        self.ancilla_set_xy(k, oldx, oldy);
        b | self.ancilla_check_tile_collision_class2_inner(k)
    }

    fn ancilla_check_tile_collision_class2_inner(&mut self, k: usize) -> bool {
        const Y: [i8; 4] = [-8, 8, 0, 0];
        const X: [i8; 4] = [0, 0, -8, 8];

        let dir = self.ram[ANCILLA_DIR + k] as usize;
        let mut x = self.ancilla_x(k).wrapping_add(X[dir] as i16 as u16);
        let y = self.ancilla_y(k).wrapping_add(Y[dir] as i16 as u16);

        if y.wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)) >= 224
            || x.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)) >= 256
        {
            return false;
        }

        let tile_attr = if self.ram[PLAYER_IS_INDOORS] == 0 {
            x >>= 3;
            self.overworld_get_tile_attribute_at_location(x, y)
        } else {
            self.get_tile_attribute_for_ancilla(self.ram[ANCILLA_FLOOR + k], x, y)
        };

        self.ram[ANCILLA_TILE_ATTR_PLAYER + k] = tile_attr;
        if tile_attr == 3 && self.ram[ANCILLA_FLOOR2 + k] != 0 {
            return false;
        }

        match K_ANCILLA_TILE_COLL_ATTRS[tile_attr as usize] {
            0 => false,
            2 => self.entity_check_sloped_tile_collision_for_ancilla(x, y),
            3 => self.ram[ANCILLA_FLOOR2 + k] != 0,
            4 => {
                if self.ram[ANCILLA_FLOOR2 + k] != 0 {
                    true
                } else {
                    self.ram[ANCILLA_OBJPRIO + k] = 1;
                    false
                }
            }
            _ => true,
        }
    }

    fn ancilla_check_initial_tile_collision_class2(&mut self, k: usize) -> bool {
        const INITIAL_TILE_COLL_Y: [i16; 9] = [15, 16, 28, 24, 12, 12, 12, 12, 8];
        const INITIAL_TILE_COLL_X: [i16; 9] = [8, 8, 8, 8, -1, 0, 17, 16, 0x4b8b];
        let mut j = self.ram[ANCILLA_DIR + k] as usize * 2;
        for _ in (0..=2).rev() {
            self.ancilla_set_xy(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(INITIAL_TILE_COLL_X[j] as u16),
                self.player_state_view()
                    .y()
                    .wrapping_add(INITIAL_TILE_COLL_Y[j] as u16),
            );
            if self.ancilla_check_tile_collision_class2(k) {
                return true;
            }
            j += 1;
        }
        false
    }

    fn ancilla_check_tile_collision_targeted(&mut self, k: usize, mut x: u16, y: u16) -> bool {
        let trace_x = x;
        let trace_y = y;
        if y.wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)) >= 224
            || x.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)) >= 256
        {
            if std::env::var_os("ZELDA3_TRACE_TILE_COLL").is_some()
                && k == 4
                && self.ram[FRAME_COUNTER] >= 140
                && self.ram[FRAME_COUNTER] <= 150
            {
                eprintln!(
                    "R tile-target fc={} k={} offscreen x={:04x} y={:04x} bg2={:04x}/{:04x} floor={:02x} type={:02x}",
                    self.ram[FRAME_COUNTER],
                    k,
                    trace_x,
                    trace_y,
                    read_le_u16(&self.ram, BG2HOFS_COPY2),
                    read_le_u16(&self.ram, BG2VOFS_COPY2),
                    self.ram[ANCILLA_FLOOR + k],
                    self.ram[ANCILLA_TYPE + k],
                );
            }
            return false;
        }
        let tile_attr = if self.ram[PLAYER_IS_INDOORS] == 0 {
            x >>= 3;
            self.overworld_get_tile_attribute_at_location(x, y)
        } else {
            self.get_tile_attribute_for_ancilla(self.ram[ANCILLA_FLOOR + k], x, y)
        };

        self.ram[ANCILLA_TILE_ATTR_PLAYER + k] = tile_attr;
        if tile_attr == 3 && self.ram[ANCILLA_FLOOR2 + k] != 0 {
            return false;
        }

        let mut t = K_ANCILLA_TILE_COLL0_ATTRS[tile_attr as usize];
        if self.ram[ANCILLA_TYPE + k] == 2 && tile_attr & 0xf0 == 0xc0 {
            t = 0;
        }
        if std::env::var_os("ZELDA3_TRACE_TILE_COLL").is_some()
            && k == 4
            && self.ram[FRAME_COUNTER] >= 140
            && self.ram[FRAME_COUNTER] <= 150
        {
            eprintln!(
                "R tile-target fc={} k={} x={:04x} y={:04x} lookup={:04x}/{:04x} floor={:02x} floor2={:02x} obj={:02x} type={:02x} attr={:02x} t={:02x} u={:02x} indoors={:02x} hdr={:02x} bg1={:04x}/{:04x} bg2={:04x}/{:04x}",
                self.ram[FRAME_COUNTER],
                k,
                trace_x,
                trace_y,
                x,
                y,
                self.ram[ANCILLA_FLOOR + k],
                self.ram[ANCILLA_FLOOR2 + k],
                self.ram[ANCILLA_OBJPRIO + k],
                self.ram[ANCILLA_TYPE + k],
                tile_attr,
                t,
                self.ram[ANCILLA_U + k],
                self.ram[PLAYER_IS_INDOORS],
                self.ram[DUNG_HDR_COLLISION],
                read_le_u16(&self.ram, BG1HOFS_COPY2),
                read_le_u16(&self.ram, BG1VOFS_COPY2),
                read_le_u16(&self.ram, BG2HOFS_COPY2),
                read_le_u16(&self.ram, BG2VOFS_COPY2),
            );
        }

        if self.ram[ANCILLA_OBJPRIO + k] == 0 {
            if t == 0 {
                return false;
            }
            if t == 1 {
                self.ram[SPRITE_ALERT_FLAG] = 3;
                return true;
            }
            if t == 2 {
                return self.entity_check_sloped_tile_collision_for_ancilla(x, y);
            }
            if t == 3 {
                if self.ram[ANCILLA_FLOOR2 + k] != 0 {
                    self.ram[SPRITE_ALERT_FLAG] = 3;
                    return true;
                }
                return false;
            }
        }
        self.ram[ANCILLA_U + k] = self.ram[ANCILLA_U + k].wrapping_sub(1);
        if (self.ram[ANCILLA_U + k] as i8) < 0 {
            self.ram[ANCILLA_U + k] = 0;
            if t == 4 {
                self.ram[ANCILLA_U + k] = 6;
                self.ram[ANCILLA_OBJPRIO + k] ^= 1;
            }
        }
        false
    }

    fn somaria_block_check_for_transit_tile(&mut self, k: usize) {
        const SOMARIA_TRANSIT_LINE_X: [i8; 12] = [-8, 0, 8, -8, 0, 8, -16, -16, -16, 16, 16, 16];
        const SOMARIA_TRANSIT_LINE_Y: [i8; 12] = [-16, -16, -16, 16, 16, 16, -8, 0, 8, -8, 0, 8];
        if self.ram[SOMARIA_BLOCK_BG_CHECK_FLAG] == 0 {
            return;
        }
        for j in (0..=11).rev() {
            let x = self
                .ancilla_get_x(k)
                .wrapping_add(SOMARIA_TRANSIT_LINE_X[j] as i16 as u16);
            let y = self
                .ancilla_get_y(k)
                .wrapping_add(SOMARIA_TRANSIT_LINE_Y[j] as i16 as u16);
            let bak = self.ram[ANCILLA_OBJPRIO + k];
            self.ancilla_check_tile_collision_targeted(k, x, y);
            self.ram[ANCILLA_OBJPRIO + k] = bak;
            if matches!(self.ram[ANCILLA_TILE_ATTR_PLAYER + k], 0xb6 | 0xbc) {
                self.ancilla_set_xy(k, x, y);
                self.ancilla_add_somaria_platform_poof(k);
                return;
            }
        }
    }

    fn ancilla_add_somaria_platform_poof(&mut self, k: usize) {
        self.ram[ANCILLA_TYPE + k] = 0x39;
        self.ram[ANCILLA_AUX_TIMER + k] = 7;
        for j in (0..=15).rev() {
            if self.ram[SPRITE_TYPE + j] == 0xed {
                self.ram[SPRITE_STATE + j] = 0;
                self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
            }
        }
        self.player_tile_detect_nearby();
    }

    fn ancilla_add_exploding_somaria_block(&mut self, k: usize) {
        self.ram[ANCILLA_TYPE + k] = 0x2e;
        self.ram[ANCILLA_NUMSPR + k] = K_ANCILLA_PFLAGS[0x2e];
        self.ram[ANCILLA_AUX_TIMER + k] = 3;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_ARR3 + k] = 0;
        self.ram[ANCILLA_ARR1 + k] = 0;
        self.ram[ANCILLA_R_PLAYER + k] = 0;
        self.ram[ANCILLA_OBJPRIO + k] = 0;
        self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER] = 0;
        self.ram[SOUND_EFFECT_2] = self.ancilla_calculate_sfx_pan(k) | 1;
    }

    pub(super) fn ancilla_add_charged_spin_attack_sparkle(&mut self) {
        for k in (0..10).rev() {
            if self.ram[ANCILLA_TYPE + k] == 0 || self.ram[ANCILLA_TYPE + k] == 0x3c {
                self.ram[ANCILLA_TYPE + k] = 13;
                self.ram[ANCILLA_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
                self.ram[ANCILLA_TIMER + k] = 6;
                break;
            }
        }
    }

    pub(super) fn ancilla_add_sword_swing_sparkle(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return;
        };
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_AUX_TIMER + k] = 1;
        self.ram[ANCILLA_DIR + k] = self.ram[LINK_DIRECTION_FACING] >> 1;
        self.ancilla_set_xy(
            k,
            self.player_state_view().x(),
            self.player_state_view().y(),
        );
    }

    pub(super) fn ancilla_add_spin_attack_init_spark(&mut self, a: u8, x: u8, y: u8) {
        const SPIN_ATTACK_START_SPARKLE_Y: [i8; 4] = [32, -8, 10, 20];
        const SPIN_ATTACK_START_SPARKLE_X: [i8; 4] = [10, 7, 28, -10];

        let k = self.ancilla_add_ancilla(a, y);
        for i in (0..=4).rev() {
            if self.ram[ANCILLA_TYPE + i] == 0x31 {
                self.ram[ANCILLA_TYPE + i] = 0;
            }
        }
        let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        let spark_x = self
            .player_state_view()
            .x()
            .wrapping_add(SPIN_ATTACK_START_SPARKLE_X[j] as i16 as u16);
        let spark_y = self
            .player_state_view()
            .y()
            .wrapping_add(SPIN_ATTACK_START_SPARKLE_Y[j] as i16 as u16);
        let Some(k) = k else {
            // C writes through k = -1 on allocation failure; preserve those
            // aliasing writes explicitly instead of silently returning.
            self.ram[ANCILLA_ITEM_TO_LINK - 1] = 0;
            self.ram[ANCILLA_STEP - 1] = x;
            self.ram[ANCILLA_TIMER - 1] = 4;
            self.ram[ANCILLA_AUX_TIMER - 1] = 3;
            self.ram[ANCILLA_X_LO - 1] = spark_x as u8;
            self.ram[ANCILLA_X_HI - 1] = (spark_x >> 8) as u8;
            self.ram[ANCILLA_Y_LO - 1] = spark_y as u8;
            self.ram[ANCILLA_Y_HI - 1] = (spark_y >> 8) as u8;
            return;
        };
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_STEP + k] = x;
        self.ram[ANCILLA_TIMER + k] = 4;
        self.ram[ANCILLA_AUX_TIMER + k] = 3;
        self.ancilla_set_xy(k, spark_x, spark_y);
    }

    fn ancilla_add_sword_charge_sparkle(&mut self, k: usize) {
        let mut j = 9usize;
        while self.ram[ANCILLA_TYPE + j] != 0 {
            if j == 0 {
                return;
            }
            j -= 1;
        }
        self.ram[ANCILLA_TYPE + j] = 60;
        self.ram[ANCILLA_FLOOR + j] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[ANCILLA_ITEM_TO_LINK + j] = 0;
        self.ram[ANCILLA_TIMER + j] = 4;

        let rand = self.get_random_number();

        let mut z = self.ram[ANCILLA_Z + k];
        if z >= 0xf8 {
            z = 0;
        }
        let dst_x = self
            .ancilla_get_x(k)
            .wrapping_add(2)
            .wrapping_add((rand >> 5) as u16);
        let dst_y = self
            .ancilla_get_y(k)
            .wrapping_sub(2)
            .wrapping_sub(z as u16)
            .wrapping_add((rand & 0xf) as u16);
        if self.replay_ancilla_trace_enabled() {
            println!(
                "ancilla-trace kind=child-charge abs={} fc=0x{:02x} src={} dst={} rng=0x{:02x} base=0x{:04x}/0x{:04x} z=0x{:02x} xy=0x{:04x}/0x{:04x} type=0x{:02x} timer=0x{:02x} floor=0x{:02x} link=0x{:04x}/0x{:04x} face=0x{:02x} spin=0x{:02x} speed=0x{:02x}/0x{:02x}",
                self.state_recorder.replay_frame_counter,
                self.ram[FRAME_COUNTER],
                k,
                j,
                rand,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                z,
                dst_x,
                dst_y,
                self.ram[ANCILLA_TYPE + j],
                self.ram[ANCILLA_TIMER + j],
                self.ram[ANCILLA_FLOOR + j],
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.ram[LINK_DIRECTION_FACING],
                self.ram[LINK_SPIN_ATTACK_STEP_COUNTER],
                self.ram[LINK_ACTUAL_VEL_X],
                self.ram[LINK_ACTUAL_VEL_Y],
            );
        }
        self.ancilla_set_xy(j, dst_x, dst_y);
    }

    fn ancilla_add_silver_arrow_sparkle(&mut self, kin: usize) {
        const SILVER_ARROW_SPARKLE_X: [i8; 4] = [-4, -4, 0, 2];
        const SILVER_ARROW_SPARKLE_Y: [i8; 4] = [0, 2, -4, -4];

        if let Some(k) = self.ancilla_alloc_high() {
            self.ram[ANCILLA_TYPE + k] = 0x3c;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_TIMER + k] = 4;
            self.ram[ANCILLA_FLOOR + k] = self.ram[LINK_IS_ON_LOWER_LEVEL];
            let m = self.get_random_number();
            let j = (self.ram[ANCILLA_DIR + kin] & 3) as usize;
            self.ancilla_set_xy(
                k,
                self.ancilla_get_x(kin)
                    .wrapping_add(SILVER_ARROW_SPARKLE_X[j] as i16 as u16)
                    .wrapping_add(((m >> 4) & 7) as u16),
                self.ancilla_get_y(kin)
                    .wrapping_add(SILVER_ARROW_SPARKLE_Y[j] as i16 as u16)
                    .wrapping_add((m & 7) as u16),
            );
        }
    }

    pub(super) fn ancilla_add_ice_rod_shot(&mut self, a: u8, y: u8) {
        const ICE_ROD_X: [i8; 4] = [0, 0, -20, 20];
        const ICE_ROD_Y: [i8; 4] = [-16, 24, 8, 8];
        const ICE_ROD_XVEL: [i8; 4] = [0, 0, -48, 48];
        const ICE_ROD_YVEL: [i8; 4] = [-48, 48, 0, 0];

        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            self.refund_magic(0);
            return;
        };
        self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 15;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_ARR25 + k] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 255;
        self.ram[ANCILLA_L + k] = 1;
        self.ram[ANCILLA_AUX_TIMER + k] = 3;
        self.ram[ANCILLA_ARR3 + k] = 6;
        let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        self.ram[ANCILLA_DIR + k] = j as u8;
        self.ram[ANCILLA_Y_VEL + k] = ICE_ROD_YVEL[j] as u8;
        self.ram[ANCILLA_X_VEL + k] = ICE_ROD_XVEL[j] as u8;

        if self.ancilla_check_initial_tile_a(k) < 0 {
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(ICE_ROD_X[j] as i16 as u16);
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(ICE_ROD_Y[j] as i16 as u16);

            if (x.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2))
                | y.wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)))
                & 0xff00
                != 0
            {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
            self.ancilla_set_xy(k, x, y);
        } else {
            self.ram[ANCILLA_TYPE + k] = 0x11;
            self.ram[ANCILLA_NUMSPR + k] = K_ANCILLA_PFLAGS[0x11];
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_AUX_TIMER + k] = 4;
        }
    }

    #[track_caller]
    pub(super) fn ancilla_add_splash(&mut self, a: u8, y: u8) -> bool {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            if std::env::var_os("ZELDA3_REPLAY_SPLASH_TRACE").is_some() {
                let caller = std::panic::Location::caller();
                println!(
                    "splash-trace abs={} fc=0x{:02x} a=0x{:02x} yarg=0x{:02x} slot=-1 caller={}:{} link=0x{:04x}/0x{:04x} state=0x{:02x} deep=0x{:04x} inwater=0x{:02x} indoors={} lower=0x{:02x} aux=0x{:02x} z=0x{:02x} vz=0x{:02x} tile=0x{:04x} normal=0x{:04x} joy=0x{:02x}/0x{:02x}",
                    self.state_recorder.replay_frame_counter,
                    self.ram[FRAME_COUNTER],
                    a,
                    y,
                    caller.file(),
                    caller.line(),
                    self.player_state_view().x(),
                    self.player_state_view().y(),
                    self.ram[LINK_PLAYER_HANDLER_STATE],
                    read_le_u16(&self.ram, TILEDETECT_DEEPWATER),
                    self.ram[LINK_IS_IN_DEEP_WATER],
                    self.ram[PLAYER_IS_INDOORS],
                    self.ram[LINK_IS_ON_LOWER_LEVEL],
                    self.ram[LINK_AUXILIARY_STATE],
                    self.ram[LINK_Z_COORD],
                    self.ram[LINK_ACTUAL_VEL_Z],
                    read_le_u16(&self.ram, TILEDETECT_TILE_TYPE),
                    read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES),
                    self.ram[JOYPAD1H_LAST],
                    self.ram[JOYPAD1L_LAST],
                );
            }
            return true;
        };
        if std::env::var_os("ZELDA3_REPLAY_SPLASH_TRACE").is_some() {
            let caller = std::panic::Location::caller();
            println!(
                "splash-trace abs={} fc=0x{:02x} a=0x{:02x} yarg=0x{:02x} slot={} caller={}:{} link=0x{:04x}/0x{:04x} state=0x{:02x} deep=0x{:04x} inwater=0x{:02x} indoors={} lower=0x{:02x} aux=0x{:02x} z=0x{:02x} vz=0x{:02x} tile=0x{:04x} normal=0x{:04x} joy=0x{:02x}/0x{:02x}",
                self.state_recorder.replay_frame_counter,
                self.ram[FRAME_COUNTER],
                a,
                y,
                k,
                caller.file(),
                caller.line(),
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.ram[LINK_PLAYER_HANDLER_STATE],
                read_le_u16(&self.ram, TILEDETECT_DEEPWATER),
                self.ram[LINK_IS_IN_DEEP_WATER],
                self.ram[PLAYER_IS_INDOORS],
                self.ram[LINK_IS_ON_LOWER_LEVEL],
                self.ram[LINK_AUXILIARY_STATE],
                self.ram[LINK_Z_COORD],
                self.ram[LINK_ACTUAL_VEL_Z],
                read_le_u16(&self.ram, TILEDETECT_TILE_TYPE),
                read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES),
                self.ram[JOYPAD1H_LAST],
                self.ram[JOYPAD1L_LAST],
            );
        }
        self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 0x24;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_AUX_TIMER + k] = 2;
        if self.ram[PLAYER_IS_INDOORS] != 0 && self.ram[LINK_IS_IN_DEEP_WATER] == 0 {
            self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
        }
        self.ancilla_set_xy(
            k,
            self.player_state_view().x().wrapping_sub(11),
            self.player_state_view().y().wrapping_add(8),
        );
        false
    }

    pub(super) fn ancilla_add_grave_stone(&mut self, ain: u8, yin: u8) {
        const MOVE_GRAVESTONE_Y: [u16; 8] =
            [0x550, 0x540, 0x530, 0x520, 0x500, 0x4e0, 0x4c0, 0x4b0];
        const MOVE_GRAVESTONE_X: [u16; 15] = [
            0x8b0, 0x8f0, 0x910, 0x950, 0x970, 0x9a0, 0x850, 0x870, 0x8b0, 0x8f0, 0x920, 0x950,
            0x880, 0x990, 0x840,
        ];
        const MOVE_GRAVESTONE_Y1: [u16; 15] = [
            0x540, 0x530, 0x530, 0x530, 0x520, 0x520, 0x510, 0x510, 0x4f0, 0x4f0, 0x4f0, 0x4f0,
            0x4d0, 0x4b0, 0x4a0,
        ];
        const MOVE_GRAVESTONE_X1: [u16; 15] = [
            0x8b0, 0x8f0, 0x910, 0x950, 0x970, 0x9a0, 0x850, 0x870, 0x8b0, 0x8f0, 0x920, 0x950,
            0x880, 0x990, 0x840,
        ];
        const MOVE_GRAVESTONE_POS: [u16; 15] = [
            0xa16, 0x99e, 0x9a2, 0x9aa, 0x92e, 0x934, 0x88a, 0x88e, 0x796, 0x79e, 0x7a4, 0x7aa,
            0x690, 0x5b2, 0x508,
        ];
        const MOVE_GRAVESTONE_CTR: [u8; 15] = [
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x38,
            0x58,
        ];
        const MOVE_GRAVESTONE_IDX: [u8; 9] = [0, 1, 4, 6, 8, 12, 13, 14, 15];

        let Some(k) = self.ancilla_add_ancilla(ain, yin) else {
            return;
        };
        let link_y = self.player_state_view().y();
        let t = if link_y & 0x0f < 7 {
            link_y
        } else {
            link_y.wrapping_add(16)
        } & !0x0f;

        let mut i = 7usize;
        while MOVE_GRAVESTONE_Y[i] != t {
            if i == 0 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
            i -= 1;
        }

        let mut j = MOVE_GRAVESTONE_IDX[i] as usize;
        let end = MOVE_GRAVESTONE_IDX[i + 1] as usize;
        loop {
            let x = MOVE_GRAVESTONE_X[j];
            let link_x = self.player_state_view().x();
            if x < link_x && x.wrapping_add(15) >= link_x {
                if (j == 13) == (self.ram[LINK_IS_RUNNING] == 0) {
                    break;
                }

                let pos = MOVE_GRAVESTONE_POS[j];
                write_le_u16(&mut self.ram, BIG_ROCK_STARTING_ADDRESS, pos);
                write_le_u16(
                    &mut self.ram,
                    DOOR_OPEN_CLOSED_COUNTER,
                    MOVE_GRAVESTONE_CTR[j] as u16,
                );
                if self.ram[DOOR_OPEN_CLOSED_COUNTER] == 0x58 {
                    self.ram[SOUND_EFFECT_2] = self.link_calculate_sfx_pan() | 0x1b;
                } else if self.ram[DOOR_OPEN_CLOSED_COUNTER] == 0x38 {
                    let screen = self.ram[OVERWORLD_SCREEN_INDEX] as usize;
                    self.ram[SAVE_OW_EVENT_INFO_ANCILLA + screen] |= 0x20;
                    self.ram[SOUND_EFFECT_2] = self.link_calculate_sfx_pan() | 0x1b;
                }

                let debris = pos.wrapping_sub(0x80);
                self.ram[DOOR_DEBRIS_Y + k] = debris as u8;
                self.ram[DOOR_DEBRIS_X + k] = (debris >> 8) as u8;

                self.Overworld_DoMapUpdate32x32_B();

                if self.ram[SOUND_EFFECT_2] & 0x3f != 0x1b {
                    self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 0x22;
                }

                let yy = MOVE_GRAVESTONE_Y1[j];
                let xx = MOVE_GRAVESTONE_X1[j];
                self.ram[PLAYER_DEFENSE_FLAGS] = 4;
                self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 1;
                let ancilla_a = yy.wrapping_sub(18);
                self.ram[ANCILLA_A + k] = ancilla_a as u8;
                self.ram[ANCILLA_B + k] = (ancilla_a >> 8) as u8;
                self.ancilla_set_xy(k, xx, yy.wrapping_sub(2));
                return;
            }
            j += 1;
            if j == end {
                break;
            }
        }
        self.ram[ANCILLA_TYPE + k] = 0;
    }

    pub(super) fn ancilla_add_waterfall_splash(&mut self) {
        if self.ancilla_add_check_for_presence(0x41) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(0x41, 4) {
            self.ram[ANCILLA_TIMER + k] = 2;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        }
    }

    pub(super) fn ancilla_add_door_debris(&mut self) -> i32 {
        let Some(k) = self.ancilla_add_ancilla(8, 1) else {
            return -1;
        };
        self.ram[ANCILLA_ARR25 + k] = 0;
        self.ram[ANCILLA_ARR26 + k] = 7;
        k as i32
    }

    fn ancilla_add_occasional_sparkle(&mut self, k: usize) {
        if self.ram[FRAME_COUNTER] & 7 == 0 {
            self.ancilla_add_sword_charge_sparkle(k);
        }
    }

    fn ancilla43_ganons_tower_cutscene(&mut self, k: usize) {
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut draw_ring = true;

        if self.ram[ANCILLA_STEP + k] == 0 {
            let yy = self.ram[ANCILLA_Y_VEL + k].wrapping_sub(1);
            self.ram[ANCILLA_Y_VEL + k] = if yy < 0xf0 { 0xf0 } else { yy };
            self.ancilla_move_y(k);
            let x = self.ancilla_get_x(k);
            let y = self.ancilla_get_y(k);
            let bg2vofs = read_le_u16(&self.ram, BG2VOFS_COPY);
            if y.wrapping_sub(bg2vofs) < 0x38 {
                write_le_u16(
                    &mut self.ram,
                    BREAKTOWERSEAL_Y,
                    0x38u16.wrapping_add(8).wrapping_add(bg2vofs),
                );
                write_le_u16(&mut self.ram, BREAKTOWERSEAL_X, x.wrapping_add(8));
                self.ancilla_set_y(k, 0x38u16.wrapping_add(bg2vofs));
                self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                self.ram[SOUND_EFFECT_AMBIENT] = 5;
                self.ram[MUSIC_CONTROL] = 0xf1;
                write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x013b);
                self.main_show_text_message();
                draw_ring = false;
            } else if self.frame_control_view().submodule() == 0 {
                draw_ring = false;
            }
        }

        if draw_ring {
            if self.ram[ANCILLA_STEP + k] == 1 && self.frame_control_view().submodule() == 0 {
                self.ram[ANCILLA_X_VEL + k] = 16;
                let bak0 = self.ram[ANCILLA_X_LO + k];
                let bak1 = self.ram[ANCILLA_X_HI + k];
                self.ram[ANCILLA_X_LO + k] = self.ram[BREAKTOWERSEAL_VAR4];
                self.ram[ANCILLA_X_HI + k] = 0;
                self.ancilla_move_x(k);
                self.ram[BREAKTOWERSEAL_VAR4] = self.ram[ANCILLA_X_LO + k];
                self.ram[ANCILLA_X_LO + k] = bak0;
                self.ram[ANCILLA_X_HI + k] = bak1;
                if self.ram[BREAKTOWERSEAL_VAR4] >= 48 {
                    self.ram[BREAKTOWERSEAL_VAR4] = 48;
                    self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                }
            }

            if self.frame_control_view().submodule() == 0
                && self.ram[ANCILLA_STEP + k] != 0
                && self.ram[ANCILLA_STEP + k] != 1
            {
                if self.ram[ANCILLA_STEP + k] == 2 {
                    self.ram[BREAKTOWERSEAL_VAR5] = self.ram[BREAKTOWERSEAL_VAR5].wrapping_sub(1);
                    if self.ram[BREAKTOWERSEAL_VAR5] == 0 {
                        self.ram[TRIGGER_SPECIAL_ENTRANCE_ANCILLA] = 5;
                        self.frame_control_view_mut().set_subsubmodule(0);
                        self.ram[R16] = 0;
                        self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                    }
                } else {
                    self.ram[ANCILLA_X_VEL + k] = 48;
                    let bak0 = self.ram[ANCILLA_X_LO + k];
                    let bak1 = self.ram[ANCILLA_X_HI + k];
                    self.ram[ANCILLA_X_LO + k] = self.ram[BREAKTOWERSEAL_VAR4];
                    self.ram[ANCILLA_X_HI + k] = 0;
                    self.ancilla_move_x(k);
                    self.ram[BREAKTOWERSEAL_VAR4] = self.ram[ANCILLA_X_LO + k];
                    self.ram[ANCILLA_X_LO + k] = bak0;
                    self.ram[ANCILLA_X_HI + k] = bak1;
                    if self.ram[BREAKTOWERSEAL_VAR4] >= 240 {
                        self.ram[PALETTE_SP6R_INDOORS] = 0;
                        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x0200);
                        self.Palette_Load_SpriteEnvironment_Dungeon();
                        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] =
                            self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
                        self.ram[ANCILLA_TYPE + k] = 0;
                        return;
                    }
                }
            }

            let astep = self.ram[ANCILLA_STEP + k];
            if astep != 0 {
                oam = self.gt_cutscene_sparkle_a_lot(oam);
            }

            for j in (0..=6).rev() {
                if self.frame_control_view().submodule() == 0
                    && astep != 1
                    && self.ram[FRAME_COUNTER] & 1 == 0
                {
                    self.ram[BREAKTOWERSEAL_VAR3 + j] =
                        self.ram[BREAKTOWERSEAL_VAR3 + j].wrapping_add(1) & 63;
                }
                let arp = self.ancilla_get_radial_projection(
                    self.ram[BREAKTOWERSEAL_VAR3 + j],
                    self.ram[BREAKTOWERSEAL_VAR4],
                );
                let x = (if arp.r6 != 0 {
                    -(arp.r4 as i32)
                } else {
                    arp.r4 as i32
                }) + i32::from(read_le_u16(&self.ram, BREAKTOWERSEAL_X))
                    - 8
                    - i32::from(read_le_u16(&self.ram, BG2HOFS_COPY));
                let y = (if arp.r2 != 0 {
                    -(arp.r0 as i32)
                } else {
                    arp.r0 as i32
                }) + i32::from(read_le_u16(&self.ram, BREAKTOWERSEAL_Y))
                    - 8
                    - i32::from(read_le_u16(&self.ram, BG2VOFS_COPY));

                self.ram[BREAKTOWERSEAL_BASE_SPARKLE_X_LO + j] = x as u8;
                self.ram[BREAKTOWERSEAL_BASE_SPARKLE_X_HI + j] = ((x as u16) >> 8) as u8;
                self.ram[BREAKTOWERSEAL_BASE_SPARKLE_Y_LO + j] = y as u8;
                self.ram[BREAKTOWERSEAL_BASE_SPARKLE_Y_HI + j] = ((y as u16) >> 8) as u8;

                self.ancilla_draw_gt_cutscene_crystal(oam, x as u16, y as u16);
                oam += 4;
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.ram[BREAKTOWERSEAL_BASE_SPARKLE_X_LO + 7] = x as u8;
        self.ram[BREAKTOWERSEAL_BASE_SPARKLE_X_HI + 7] = (x >> 8) as u8;
        self.ram[BREAKTOWERSEAL_BASE_SPARKLE_Y_LO + 7] = y as u8;
        self.ram[BREAKTOWERSEAL_BASE_SPARKLE_Y_HI + 7] = (y >> 8) as u8;

        self.ancilla_draw_gt_cutscene_crystal(oam, x, y);

        if self.ram[ANCILLA_STEP + k] == 0 {
            self.ancilla_add_occasional_sparkle(k);
        } else if self.frame_control_view().submodule() == 0 {
            self.gt_cutscene_activate_sparkle();
        }
    }

    fn ancilla_draw_gt_cutscene_crystal(&mut self, oam: usize, x: u16, y: u16) {
        self.ancilla_set_oam_safe(oam, x, y, 0x24, 0x3c, 2);
    }

    fn fire_shot_draw(&mut self, k: usize) {
        const FIRE_SHOT_DRAW_X2: [u8; 16] = [7, 0, 8, 0, 8, 4, 0, 0, 2, 8, 0, 0, 1, 4, 9, 0];
        const FIRE_SHOT_DRAW_Y2: [u8; 16] = [1, 4, 9, 0, 7, 0, 8, 0, 8, 4, 0, 0, 2, 8, 0, 0];
        const FIRE_SHOT_DRAW_CHAR2: [u8; 3] = [0x8d, 0x9d, 0x9c];

        let Some(mut info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };
        if self.ram[ANCILLA_OBJPRIO + k] != 0 {
            info.flags |= 0x30;
        }

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = (self.ram[ANCILLA_ITEM_TO_LINK + k] & 0x0c) as usize;
        for i in (0..=2).rev() {
            self.ancilla_set_oam_plain(
                oam,
                u16::from(info.x.wrapping_add(FIRE_SHOT_DRAW_X2[j + i])),
                u16::from(info.y.wrapping_add(FIRE_SHOT_DRAW_Y2[j + i])),
                FIRE_SHOT_DRAW_CHAR2[i],
                info.flags | 2,
                0,
            );
            oam += 4;
        }
    }

    fn ice_shot_spread_draw(&mut self, k: usize) {
        const ICE_SHOT_SPREAD_CHAR_FLAGS: [u8; 16] = [
            0xcf, 0x24, 0xcf, 0x24, 0xcf, 0x24, 0xcf, 0x24, 0xdf, 0x24, 0xdf, 0x24, 0xdf, 0x24,
            0xdf, 0x24,
        ];
        const ICE_SHOT_SPREAD_XY: [u8; 16] = [
            0, 0, 0, 8, 8, 0, 8, 8, 0xf8, 0xf8, 0xf8, 0x10, 0x10, 0xf8, 0x10, 0x10,
        ];

        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, self.ram[ANCILLA_NUMSPR + k]);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let oam_org = oam;
        let mut j = self.ram[ANCILLA_ITEM_TO_LINK + k] as usize * 4;
        for _ in 0..4 {
            let y = info_y.wrapping_add(ICE_SHOT_SPREAD_XY[j * 2] as i8 as i16 as u16);
            let x = info_x.wrapping_add(ICE_SHOT_SPREAD_XY[j * 2 + 1] as i8 as i16 as u16);
            let mut yv = 0xf0;
            if x < 256 && y < 256 {
                self.ram[oam] = x as u8;
                if y < 224 {
                    yv = y as u8;
                }
            }
            self.ram[oam + 1] = yv;
            self.ram[oam + 2] = ICE_SHOT_SPREAD_CHAR_FLAGS[j * 2];
            self.ram[oam + 3] =
                ICE_SHOT_SPREAD_CHAR_FLAGS[j * 2 + 1] & !0x30 | self.ram[OAM_PRIORITY_VALUE + 1];
            self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4] = 0;
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
            j += 1;
        }
        if self.ram[oam_org + 1] == 0xf0 && self.ram[oam_org + 5] == 0xf0 {
            self.ram[ANCILLA_TYPE + k] = 0;
        }
    }

    fn ancilla11_ice_rod_wall_hit(&mut self, k: usize) {
        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
            self.ram[ANCILLA_AUX_TIMER + k] = 7;
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 2 {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }
        self.ice_shot_spread_draw(k);
    }

    fn ancilla0_a_arrow_in_the_wall(&mut self, k: usize) {
        let j = self.ram[ANCILLA_S_PLAYER + k];
        if !sign8(j) {
            let j = j as usize;
            if self.ram[SPRITE_STATE + j] < 9
                || sign8(self.ram[SPRITE_Z + j])
                || self.ram[SPRITE_IGNORE_PROJECTILE_ANCILLA + j] != 0
                || self.ram[SPRITE_DEFL_BITS + j] & 2 != 0
            {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
            self.ancilla_set_x(
                k,
                self.sprite_get_x(j)
                    .wrapping_add(self.ram[ANCILLA_X_VEL + k] as i8 as i16 as u16),
            );
            self.ancilla_set_y(
                k,
                self.sprite_get_y(j)
                    .wrapping_add(self.ram[ANCILLA_Y_VEL + k] as i8 as i16 as u16)
                    .wrapping_sub(u16::from(self.ram[SPRITE_Z + j])),
            );
        }
        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
            if self.ram[ANCILLA_AUX_TIMER + k] == 0 {
                self.ram[ANCILLA_AUX_TIMER + k] = 2;
                self.ram[ANCILLA_ITEM_TO_LINK + k] =
                    self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                if self.ram[ANCILLA_ITEM_TO_LINK + k] == 9 {
                    self.ram[ANCILLA_TYPE + k] = 0;
                    return;
                } else if self.ram[ANCILLA_ITEM_TO_LINK + k] & 8 != 0 {
                    self.ram[ANCILLA_AUX_TIMER + k] = 0x80;
                }
            }
        }
        self.arrow_draw(k);
    }

    fn somarian_blast_draw(&mut self, k: usize) {
        const SOMARIAN_BLAST_FLAGS: [u8; 2] = [2, 6];
        const SOMARIAN_BLAST_DRAW_X0: [i8; 24] = [
            0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const SOMARIAN_BLAST_DRAW_X1: [i8; 24] = [
            8, 8, 8, 8, 4, 4, 8, 8, 8, 8, 4, 4, 0, 0, 0, 0, 8, 8, 0, 0, 0, 0, 8, 8,
        ];
        const SOMARIAN_BLAST_DRAW_Y0: [u8; 24] = [
            0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 4, 4,
        ];
        const SOMARIAN_BLAST_DRAW_Y1: [u8; 24] = [
            0, 0, 0, 0, 8, 8, 0x80, 0, 0, 0, 8, 8, 0x80, 8, 8, 8, 4, 4, 0x80, 8, 8, 8, 4, 4,
        ];
        const SOMARIAN_BLAST_DRAW_FLAGS0: [u8; 24] = [
            0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0xc0, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0xc0, 0, 0, 0, 0, 0, 0x80,
        ];
        const SOMARIAN_BLAST_DRAW_FLAGS1: [u8; 24] = [
            0x80, 0x80, 0x80, 0x80, 0x80, 0xc0, 0, 0, 0, 0, 0, 0x40, 0xc0, 0xc0, 0xc0, 0xc0, 0x40,
            0xc0, 0x80, 0x80, 0x80, 0x80, 0, 0x80,
        ];
        const SOMARIAN_BLAST_DRAW_CHAR0: [u8; 24] = [
            0x50, 0x50, 0x44, 0x44, 0x52, 0x52, 0x50, 0x50, 0x44, 0x44, 0x51, 0x51, 0x43, 0x43,
            0x42, 0x42, 0x41, 0x41, 0x43, 0x43, 0x42, 0x42, 0x40, 0x40,
        ];
        const SOMARIAN_BLAST_DRAW_CHAR1: [u8; 24] = [
            0x50, 0x50, 0x44, 0x44, 0x51, 0x51, 0x50, 0x50, 0x44, 0x44, 0x52, 0x52, 0x43, 0x43,
            0x42, 0x42, 0x40, 0x40, 0x43, 0x43, 0x42, 0x42, 0x41, 0x41,
        ];

        let Some(mut info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };
        info.flags |= SOMARIAN_BLAST_FLAGS[self.ram[ANCILLA_ITEM_TO_LINK + k] as usize];
        if self.ram[ANCILLA_OBJPRIO + k] != 0 {
            info.flags |= 0x30;
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = self.ram[ANCILLA_DIR + k] as usize * 6 + self.ram[ANCILLA_STEP + k] as usize;
        self.ram[oam] = info.x.wrapping_add(SOMARIAN_BLAST_DRAW_X0[j] as u8);
        self.ram[oam + 1] = if sign8(SOMARIAN_BLAST_DRAW_Y0[j]) {
            0xf0
        } else {
            info.y.wrapping_add(SOMARIAN_BLAST_DRAW_Y0[j])
        };
        self.ram[oam + 2] = 0x82u8.wrapping_add(SOMARIAN_BLAST_DRAW_CHAR0[j]);
        self.ram[oam + 3] = info.flags | SOMARIAN_BLAST_DRAW_FLAGS0[j];
        self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4] = 0;
        self.ram[oam + 4] = info.x.wrapping_add(SOMARIAN_BLAST_DRAW_X1[j] as u8);
        self.ram[oam + 5] = if sign8(SOMARIAN_BLAST_DRAW_Y1[j]) {
            0xf0
        } else {
            info.y.wrapping_add(SOMARIAN_BLAST_DRAW_Y1[j])
        };
        self.ram[oam + 6] = 0x82u8.wrapping_add(SOMARIAN_BLAST_DRAW_CHAR1[j]);
        self.ram[oam + 7] = info.flags | SOMARIAN_BLAST_DRAW_FLAGS1[j];
        self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4 + 1] = 0;
    }

    fn arrow_draw(&mut self, k: usize) {
        const ARROW_DRAW_CHAR: [u8; 48] = [
            0x2b, 0x2a, 0x2a, 0x2b, 0x3d, 0x3a, 0x3a, 0x3d, 0x2b, 0xff, 0x2b, 0xff, 0x3d, 0xff,
            0x3d, 0xff, 0x3c, 0x2c, 0x3c, 0x2a, 0x3c, 0x2c, 0x3c, 0x2a, 0x2c, 0x3c, 0x2a, 0x3c,
            0x2c, 0x3c, 0x2a, 0x3c, 0x3b, 0x2d, 0x3b, 0x3a, 0x3b, 0x2d, 0x3b, 0x3a, 0x2d, 0x3b,
            0x3a, 0x3b, 0x2d, 0x3b, 0x3a, 0x3b,
        ];
        const ARROW_DRAW_FLAGS: [u8; 48] = [
            0xa4, 0xa4, 0x24, 0x24, 0x64, 0x64, 0x24, 0x24, 0xa4, 0xff, 0x24, 0xff, 0x64, 0xff,
            0x24, 0xff, 0xa4, 0xa4, 0xa4, 0xa4, 0xa4, 0xe4, 0xa4, 0xa4, 0x24, 0x24, 0x24, 0x24,
            0x64, 0x24, 0x24, 0x24, 0x64, 0x64, 0x64, 0xe4, 0x64, 0xe4, 0x64, 0xe4, 0x24, 0x24,
            0x24, 0xa4, 0xa4, 0x24, 0x24, 0xa4,
        ];
        const ARROW_DRAW_Y: [i8; 48] = [
            0, 8, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0,
            8, 0, 8, -1, -1, 0, 0, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 0, 0,
        ];
        const ARROW_DRAW_X: [i8; 48] = [
            0, 0, 0, 0, 0, 8, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, -2, 0, 0, 1, 1, 0, 0,
            -2, -1, 0, 0, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8,
        ];

        let (mut x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        if self.ram[ANCILLA_OBJPRIO + k] != 0 {
            self.ram[OAM_PRIORITY_VALUE + 1] = 0x30;
        }
        if self.ram[ANCILLA_H + k] != 0 {
            x = x.wrapping_add(
                read_le_u16(&self.ram, BG2VOFS_COPY2)
                    .wrapping_sub(read_le_u16(&self.ram, BG1VOFS_COPY2)),
            );
            y = y.wrapping_add(
                read_le_u16(&self.ram, BG2HOFS_COPY2)
                    .wrapping_sub(read_le_u16(&self.ram, BG1HOFS_COPY2)),
            );
        }

        let r7 = self.ram[ANCILLA_ITEM_TO_LINK + k];
        let mut j = self.ram[ANCILLA_DIR + k] & !4;
        if self.ram[ANCILLA_TYPE + k] == 0x0a {
            j = j
                .wrapping_mul(4)
                .wrapping_add(8)
                .wrapping_add(if r7 & 8 != 0 { 1 } else { r7 & 3 });
        } else if !sign8(r7) {
            j |= 4;
        }
        let mut j = j as usize * 2;

        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let oam_org = oam;
        let flags = if self.ram[LINK_ITEM_BOW] & 4 != 0 {
            2
        } else {
            4
        };
        for _ in 0..2 {
            if ARROW_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(ARROW_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(ARROW_DRAW_Y[j] as i16 as u16),
                    ARROW_DRAW_CHAR[j],
                    ARROW_DRAW_FLAGS[j] & !0x3e | flags | self.ram[OAM_PRIORITY_VALUE + 1],
                    0,
                );
                oam += 4;
            }
            j += 1;
        }

        if self.ram[oam_org + 1] == 0xf0 && self.ram[oam_org + 5] == 0xf0 {
            self.ram[ANCILLA_TYPE + k] = 0;
        }
    }

    fn revival_fairy_monitor_hp(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        if (self.ram[LINK_HEALTH_CURRENT] == self.ram[LINK_HEALTH_CAPACITY]
            || self.ram[LINK_HEALTH_CURRENT] == 0x38)
            && self.ram[IS_DOING_HEART_ANIMATION] == 0
        {
            if self.ram[LINK_IS_IN_DEEP_WATER] != 0 {
                self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = 4;
                self.ram[LINK_PLAYER_HANDLER_STATE] = 4;
            } else if self.ram[LINK_IS_BUNNY] != 0 {
                self.ram[LINK_PLAYER_HANDLER_STATE] = 23;
                self.ram[LINK_IS_BUNNY_MIRROR] = 1;
                if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0 {
                    self.LoadGearPalettes_bunny();
                }
            } else {
                self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            }
            self.ram[LINK_AUXILIARY_STATE] = 0;
            self.ram[LINK_FAINT_ANIMATION_ACTIVE] = 0;
            self.ram[LINK_VAR30D] = 0;
            self.ram[Y_BUTTON_ACTION_STEP] = 0;
            self.ram[LINK_Z_COORD] = 0;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
            for i in 0..5 {
                self.ram[ANCILLA_TYPE + i] = 0;
            }
            return;
        }

        let k = 1;
        if self.ram[ANCILLA_STEP + k] == 0 {
            self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
            if self.ram[ANCILLA_ARR3 + k] == 0 {
                self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_add(1);
                self.ram[ANCILLA_Z_VEL + k] = 4;
                self.ancilla_move_z(k);
                if self.ram[ANCILLA_Z + k] >= 16 {
                    self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                    self.ram[ANCILLA_Z_VEL + k] = 2;
                }
            }
        } else {
            self.ram[ANCILLA_K + k] = self.ram[ANCILLA_K + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_K + k]) {
                self.ram[ANCILLA_K + k] = 32;
                self.ram[ANCILLA_Z_VEL + k] = 0u8.wrapping_sub(self.ram[ANCILLA_Z_VEL + k]);
            }
            self.ancilla_move_z(k);
        }
        self.ram[LINK_Z_COORD] = self.ram[ANCILLA_Z + k];
    }

    fn revival_fairy_dust(&mut self) {
        let k = 2;
        if self.ram[ANCILLA_STEP] == 0 || self.ram[ANCILLA_STEP + k] == 2 {
            return;
        }
        self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
        if !sign8(self.ram[ANCILLA_ARR3 + k]) {
            return;
        }
        self.ram[ANCILLA_ARR3 + k] = 0;
        if self.ram[SORT_SPRITES_SETTING] == 0 {
            self.oam_allocate_from_region_a(16);
        } else {
            self.oam_allocate_from_region_d(16);
        }
        self.ram[ANCILLA_AUX_TIMER + k] = self.ram[ANCILLA_AUX_TIMER + k].wrapping_sub(1);
        if sign8(self.ram[ANCILLA_AUX_TIMER + k]) {
            self.ram[ANCILLA_AUX_TIMER + k] = 3;
            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 9 {
                self.ram[ANCILLA_ARR3 + k] = 32;
                self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                self.ram[ANCILLA_ITEM_TO_LINK + k] = 2;
                return;
            }
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
            self.ram[ANCILLA_ARR25 + k] =
                K_MAGIC_POWDER_TAB0[30 + self.ram[ANCILLA_ITEM_TO_LINK + k] as usize];
        }
        self.ancilla_magic_powder_draw(k);
    }

    pub(super) fn revival_fairy_main(&mut self) {
        const ANCILLA_REVIVAL_FAERIE_TAB0: [u8; 2] = [0, 0x90];
        const ANCILLA_REVIVAL_FAERIE_TAB1: [u8; 5] = [0x4b, 0x4d, 0x49, 0x47, 0x49];

        let k = 0;
        let skip_draw = match self.ram[ANCILLA_STEP + k] {
            0 => {
                self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
                if self.ram[ANCILLA_ARR3 + k] == 0 {
                    self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                    self.ram[ANCILLA_ARR3 + k] =
                        ANCILLA_REVIVAL_FAERIE_TAB0[self.ram[ANCILLA_STEP + k] as usize];
                    self.ram[ANCILLA_K + k] = 0;
                    self.ram[ANCILLA_Z_VEL + k] = 0;
                } else {
                    self.ancilla_move_z(k);
                }
                false
            }
            1 => {
                self.ram[ANCILLA_ARR3 + k] = self.ram[ANCILLA_ARR3 + k].wrapping_sub(1);
                if self.ram[ANCILLA_ARR3 + k] == 0 {
                    self.ram[ANCILLA_STEP + k] = self.ram[ANCILLA_STEP + k].wrapping_add(1);
                    self.ram[ANCILLA_Z_VEL + k] = 0;
                    self.ram[ANCILLA_X_VEL + k] = 0;
                } else {
                    if self.ram[ANCILLA_ARR3 + k] == 0x4f || self.ram[ANCILLA_ARR3 + k] == 0x8f {
                        self.ram[ANCILLA_L + k] = self.ram[ANCILLA_L + k].wrapping_add(1);
                        self.ancilla_sfx2_pan(k, 0x31);
                    }
                    if self.ram[ANCILLA_L + k] != 0 {
                        self.ram[ANCILLA_G + k] = self.ram[ANCILLA_G + k].wrapping_sub(1);
                        if sign8(self.ram[ANCILLA_G + k]) {
                            self.ram[ANCILLA_G + k] = 5;
                            self.ram[ANCILLA_ITEM_TO_LINK + k] =
                                self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1);
                            if self.ram[ANCILLA_ITEM_TO_LINK + k] == 3 {
                                self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
                                self.ram[ANCILLA_L + k] = 0;
                            }
                        }
                    }
                    self.ram[ANCILLA_Z_VEL + k] =
                        self.ram[ANCILLA_Z_VEL + k].wrapping_add(if self.ram[ANCILLA_K + k] != 0 {
                            1
                        } else {
                            0xff
                        });
                    if abs8(self.ram[ANCILLA_Z_VEL + k]) == 8 {
                        self.ram[ANCILLA_K + k] ^= 1;
                    }
                    self.ancilla_move_z(k);
                }
                false
            }
            2 => {
                if self.ram[ANCILLA_Z_VEL + k] < 24 {
                    self.ram[ANCILLA_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + k].wrapping_add(1);
                }
                if self.ram[ANCILLA_X_VEL + k] < 16 {
                    self.ram[ANCILLA_X_VEL + k] = self.ram[ANCILLA_X_VEL + k].wrapping_add(1);
                }
                self.ancilla_move_x(k);
                self.ancilla_move_z(k);
                false
            }
            3 => true,
            _ => false,
        };

        if !skip_draw {
            self.oam_allocate_from_region_c(12);
            let (x, y) = self.ancilla_prep_oam_coord(k);
            let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
            let mut t = if self.ram[ANCILLA_STEP + k] == 1 && self.ram[ANCILLA_L + k] != 0 {
                self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(1)
            } else {
                0
            };
            if t != 0 {
                t = t.wrapping_add(1);
            } else {
                t = (self.ram[FRAME_COUNTER] >> 2) & 1;
            }
            self.ancilla_set_oam(
                oam,
                x,
                y.wrapping_sub(self.ram[ANCILLA_Z + k] as i8 as i16 as u16),
                ANCILLA_REVIVAL_FAERIE_TAB1[t as usize],
                0x74,
                2,
            );
            if self.ram[oam + 1] == 0xf0 {
                self.ram[ANCILLA_STEP + k] = 3;
                self.frame_control_view_mut().increment_submodule();
                self.ram[TM_COPY] = self.ram[MAPBAK_TM];
            }
        }

        self.revival_fairy_dust();
        self.revival_fairy_monitor_hp();
    }

    fn gt_cutscene_activate_sparkle(&mut self) {
        for k in (0..=0x17).rev() {
            if self.ram[BREAKTOWERSEAL_SPARKLE_VAR1 + k] == 0xff {
                self.ram[BREAKTOWERSEAL_SPARKLE_VAR1 + k] = 0;
                self.ram[BREAKTOWERSEAL_SPARKLE_VAR2 + k] = 4;
                let r = self.get_random_number();
                let base = k & 7;
                let mut x = u16::from(self.ram[BREAKTOWERSEAL_BASE_SPARKLE_X_LO + base])
                    | (u16::from(self.ram[BREAKTOWERSEAL_BASE_SPARKLE_X_HI + base]) << 8);
                let mut y = u16::from(self.ram[BREAKTOWERSEAL_BASE_SPARKLE_Y_LO + base])
                    | (u16::from(self.ram[BREAKTOWERSEAL_BASE_SPARKLE_Y_HI + base]) << 8);
                x = x.wrapping_add((r >> 4) as u16);
                y = y.wrapping_add((r & 0x0f) as u16);
                self.ram[BREAKTOWERSEAL_SPARKLE_X_LO + k] = x as u8;
                self.ram[BREAKTOWERSEAL_SPARKLE_X_HI + k] = (x >> 8) as u8;
                self.ram[BREAKTOWERSEAL_SPARKLE_Y_LO + k] = y as u8;
                self.ram[BREAKTOWERSEAL_SPARKLE_Y_HI + k] = (y >> 8) as u8;
                return;
            }
        }
    }

    fn gt_cutscene_sparkle_a_lot(&mut self, mut oam: usize) -> usize {
        const SWORD_CHARGE_SPARK_CHAR: [u8; 3] = [0xb7, 0x80, 0x83];
        const SWORD_CHARGE_SPARK_FLAGS: [u8; 3] = [4, 4, 0x84];

        for k in (0..=0x17).rev() {
            if self.ram[BREAKTOWERSEAL_SPARKLE_VAR1 + k] == 0xff {
                continue;
            }

            let timer = self.ram[BREAKTOWERSEAL_SPARKLE_VAR2 + k].wrapping_sub(1);
            self.ram[BREAKTOWERSEAL_SPARKLE_VAR2 + k] = timer;
            if sign8(timer) {
                self.ram[BREAKTOWERSEAL_SPARKLE_VAR2 + k] = 4;
                self.ram[BREAKTOWERSEAL_SPARKLE_VAR1 + k] =
                    self.ram[BREAKTOWERSEAL_SPARKLE_VAR1 + k].wrapping_add(1);
                if self.ram[BREAKTOWERSEAL_SPARKLE_VAR1 + k] == 3 {
                    self.ram[BREAKTOWERSEAL_SPARKLE_VAR1 + k] = 0xff;
                    continue;
                }
            }

            let x = u16::from(self.ram[BREAKTOWERSEAL_SPARKLE_X_LO + k])
                | (u16::from(self.ram[BREAKTOWERSEAL_SPARKLE_X_HI + k]) << 8);
            let y = u16::from(self.ram[BREAKTOWERSEAL_SPARKLE_Y_LO + k])
                | (u16::from(self.ram[BREAKTOWERSEAL_SPARKLE_Y_HI + k]) << 8);
            let j = self.ram[BREAKTOWERSEAL_SPARKLE_VAR1 + k] as usize;
            self.ancilla_set_oam(
                oam,
                x,
                y,
                SWORD_CHARGE_SPARK_CHAR[j],
                SWORD_CHARGE_SPARK_FLAGS[j] | 0x30,
                0,
            );
            oam += 4;
        }
        oam
    }

    fn ancilla_add_rupees(&mut self, k: usize) -> bool {
        const GIVE_RUPEE_GIFT_TAB: [u16; 5] = [1, 5, 20, 100, 50];
        let a = self.ram[ANCILLA_ITEM_TO_LINK + k];
        let amount = if (0x34..=0x36).contains(&a) {
            GIVE_RUPEE_GIFT_TAB[(a - 0x34) as usize]
        } else if a == 0x40 || a == 0x41 {
            GIVE_RUPEE_GIFT_TAB[(a - 0x40 + 3) as usize]
        } else if a == 0x46 {
            300
        } else if a == 0x47 {
            20
        } else {
            return false;
        };
        let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL).wrapping_add(amount);
        write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, rupees);
        true
    }

    fn somaria_block_spawn_bullets(&mut self, k: usize) {
        const SPAWN_CENTRIFUGAL_QUAD_X: [i8; 4] = [-8, -8, -9, -4];
        const SPAWN_CENTRIFUGAL_QUAD_Y: [i8; 4] = [-15, -4, -8, -8];

        let z = if self.ram[ANCILLA_Z + k] == 0xff {
            0
        } else {
            self.ram[ANCILLA_Z + k]
        };
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k).wrapping_sub(z as u16);

        for i in (0..=3).rev() {
            if let Some(j) = self.ancilla_alloc_init(1, 4) {
                self.ram[ANCILLA_TYPE + j] = 1;
                self.ram[ANCILLA_NUMSPR + j] = K_ANCILLA_PFLAGS[1];
                self.ram[ANCILLA_STEP + j] = 4;
                self.ram[ANCILLA_ITEM_TO_LINK + j] = 0;
                self.ram[ANCILLA_OBJPRIO + j] = 0;
                self.ram[ANCILLA_DIR + j] = i as u8;
                self.ancilla_set_xy(
                    j,
                    x.wrapping_add(SPAWN_CENTRIFUGAL_QUAD_X[i] as i16 as u16),
                    y.wrapping_add(SPAWN_CENTRIFUGAL_QUAD_Y[i] as i16 as u16),
                );
                self.ancilla_terminate_if_offscreen(j);
                self.ram[ANCILLA_X_VEL + j] = K_FIRE_ROD_XVEL2[i] as u8;
                self.ram[ANCILLA_Y_VEL + j] = K_FIRE_ROD_YVEL2[i] as u8;
                self.ram[ANCILLA_FLOOR + j] = self.ram[ANCILLA_FLOOR + k];
                self.ram[ANCILLA_FLOOR2 + j] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
            }
        }
        self.ram[TMP_COUNTER_ANCILLA] = 0xff;
    }

    fn ancilla_terminate_if_offscreen(&mut self, j: usize) {
        let xt: u16 = if self.read_u32_ram(ENHANCED_FEATURES0) & 1 != 0 {
            0x40
        } else {
            0
        };
        let x = self
            .ancilla_get_x(j)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2))
            .wrapping_add(xt);
        let y = self
            .ancilla_get_y(j)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        if x >= 244 + xt * 2 || y >= 240 {
            self.ram[ANCILLA_TYPE + j] = 0;
        }
    }

    fn ancilla_draw_somaria_block(&mut self, k: usize) {
        const SOMARIAN_BLOCK_DRAW_X: [i8; 12] = [-8, 0, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        const SOMARIAN_BLOCK_DRAW_Y: [i8; 12] = [-8, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        const SOMARIAN_BLOCK_DRAW_FLAGS: [u8; 12] = [
            0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0,
        ];

        if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize
            && self.ram[LINK_STATE_BITS] & 0x80 != 0
            && self.ram[ANCILLA_K + k] != 3
            && self.ram[LINK_DIRECTION_FACING] == 0
        {
            self.ancilla_allocate_oam_from_region_b_or_e(self.ram[ANCILLA_NUMSPR + k]);
        } else if self.ram[SORT_SPRITES_SETTING] != 0
            && self.ram[ANCILLA_FLOOR + k] != 0
            && (self.ram[ANCILLA_L + k] != 0
                || k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize
                    && self.ram[LINK_STATE_BITS] & 0x80 != 0)
        {
            write_le_u16(&mut self.ram, OAM_CUR_PTR, 0x08d0);
            write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0x0a20 + 0x34);
        }

        let (x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        let oam_org = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut oam = oam_org;
        let z = self.ram[ANCILLA_Z + k] as i8;
        if z != 0 && z != -1 && self.ram[ANCILLA_K + k] != 3 && self.ram[ANCILLA_OBJPRIO + k] != 0 {
            write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, 0x3000);
        }
        y = y.wrapping_sub(z as i16 as u16);
        let mut j = self.ram[ANCILLA_ARR1 + k] as usize * 4;
        for _ in 0..4 {
            self.ancilla_set_oam_safe(
                oam,
                x.wrapping_add(SOMARIAN_BLOCK_DRAW_X[j] as i16 as u16),
                y.wrapping_add(SOMARIAN_BLOCK_DRAW_Y[j] as i16 as u16),
                0xe9,
                SOMARIAN_BLOCK_DRAW_FLAGS[j] & !0x30 | 2 | self.ram[OAM_PRIORITY_VALUE + 1],
                0,
            );
            j += 1;
            oam += 4;
        }

        if self.somarian_block_check_empty(oam_org) {
            self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER] = 0;
            self.ram[ANCILLA_TYPE + k] = 0;
            if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
                self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
                if self.ram[LINK_STATE_BITS] & 0x80 != 0 {
                    self.ram[LINK_STATE_BITS] = 0;
                }
            }
        }
    }

    fn somaria_block_check_for_switch(&mut self, k: usize) -> bool {
        const SOMARIAN_BLOCK_CHECK_COVER_X: [i8; 4] = [0, 0, -4, 4];
        const SOMARIAN_BLOCK_CHECK_COVER_Y: [i8; 4] = [-4, 4, 0, 0];
        self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER] = 0;
        self.ram[ANCILLA_ARR24 + k] = 0;
        for j in (0..=3).rev() {
            let y = self
                .ancilla_get_y(k)
                .wrapping_add(SOMARIAN_BLOCK_CHECK_COVER_Y[j] as i16 as u16);
            let x = self
                .ancilla_get_x(k)
                .wrapping_add(SOMARIAN_BLOCK_CHECK_COVER_X[j] as i16 as u16);
            let bak = self.ram[ANCILLA_OBJPRIO + k];
            self.ancilla_check_tile_collision_targeted(k, x, y);
            self.ram[ANCILLA_OBJPRIO + k] = bak;
            if matches!(
                self.ram[ANCILLA_TILE_ATTR_PLAYER + k],
                0x23 | 0x24 | 0x25 | 0x3b
            ) {
                self.ram[ANCILLA_ARR24 + k] = self.ram[ANCILLA_ARR24 + k].wrapping_add(1);
            }
        }
        self.ram[ANCILLA_ARR24 + k] != 4
    }

    fn somaria_block_fizzle_away(&mut self, k: usize) {
        if self.ram[LINK_SPEED_SETTING] == 18 {
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
        }
        self.ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER] = 0;
        self.ram[ANCILLA_TYPE + k] = 0x2d;
        self.ram[ANCILLA_AUX_TIMER + k] = 0;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_ARR3 + k] = 0;
        self.ram[ANCILLA_ARR1 + k] = 0;
        self.ram[ANCILLA_R_PLAYER + k] = 0;
        if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize {
            self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
            self.ram[LINK_STATE_BITS] &= 0x80;
        }
        self.ancilla2_d_somaria_block_fizz(k);
    }

    fn ancilla_setup_basic_hit_box(&self, k: usize) -> SpriteHitBox {
        let x = self.ancilla_get_x(k).wrapping_sub(8);
        let y = self
            .ancilla_get_y(k)
            .wrapping_sub(8)
            .wrapping_sub(self.ram[ANCILLA_Z + k] as u16);
        SpriteHitBox {
            r0_xlo: x as u8,
            r8_xhi: (x >> 8) as u8,
            r1_ylo: y as u8,
            r9_yhi: (y >> 8) as u8,
            r2: 15,
            r3: 15,
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        }
    }

    fn ancilla_setup_hit_box(&self, k: usize) -> SpriteHitBox {
        const ANCILLA_HIT_BOX_X: [i8; 12] = [4, 4, 4, 4, 3, 3, 2, 11, -16, -16, -1, -8];
        const ANCILLA_HIT_BOX_Y: [i8; 12] = [4, 4, 4, 4, 2, 11, 3, 3, -1, -8, -16, -16];
        const ANCILLA_HIT_BOX_W: [u8; 12] = [8, 8, 8, 8, 1, 1, 1, 1, 32, 32, 8, 8];
        const ANCILLA_HIT_BOX_H: [u8; 12] = [8, 8, 8, 8, 1, 1, 1, 1, 8, 8, 32, 32];
        let mut j = self.ram[ANCILLA_DIR + k] as usize;
        if self.ram[ANCILLA_TYPE + k] == 0x0c {
            j |= 8;
        }
        let x = self
            .ancilla_get_x(k)
            .wrapping_add(ANCILLA_HIT_BOX_X[j] as i16 as u16);
        let y = self
            .ancilla_get_y(k)
            .wrapping_add(ANCILLA_HIT_BOX_Y[j] as i16 as u16);
        SpriteHitBox {
            r0_xlo: x as u8,
            r8_xhi: (x >> 8) as u8,
            r1_ylo: y as u8,
            r9_yhi: (y >> 8) as u8,
            r2: ANCILLA_HIT_BOX_W[j],
            r3: ANCILLA_HIT_BOX_H[j],
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        }
    }

    fn somarian_block_check_empty(&self, oam: usize) -> bool {
        for i in 0..4 {
            if self.ram[oam + i * 4 + 1] == 0xf0 {
                continue;
            }
            for i in 0..4 {
                if self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4 + i] & 1 == 0 {
                    return false;
                }
            }
            break;
        }
        true
    }

    fn ancilla_prep_adjusted_oam_coord(&mut self, k: usize) -> (u16, u16) {
        const TAGALONG_LAYER_BITS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
        let floor = self.ram[ANCILLA_FLOOR + k] as usize;
        write_le_u16(
            &mut self.ram,
            OAM_PRIORITY_VALUE,
            (TAGALONG_LAYER_BITS[floor] as u16) << 8,
        );
        (
            self.ancilla_get_x(k)
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY)),
            self.ancilla_get_y(k)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY)),
        )
    }

    fn ancilla_allocate_oam_from_region_b_or_e(&mut self, size: u8) {
        if self.ram[SORT_SPRITES_SETTING] == 0 {
            self.oam_allocate_from_region_b(size);
        } else {
            self.oam_allocate_from_region_e(size);
        }
    }

    fn ancilla_allocate_oam_from_custom_region(&mut self, oam: usize) -> usize {
        let mut a = oam;
        if self.ram[SORT_SPRITES_SETTING] != 0 {
            if a < 0x900 {
                if a < 0x8e0 {
                    return oam;
                }
                a = 0x820;
            } else {
                if a < 0x9d0 {
                    return oam;
                }
                a = 0x940;
            }
        } else {
            if a < 0x990 {
                return oam;
            }
            a = 0x820;
        }
        write_le_u16(&mut self.ram, OAM_CUR_PTR, a as u16);
        write_le_u16(
            &mut self.ram,
            OAM_EXT_CUR_PTR,
            (((a - 0x800) >> 2) + 0xa20) as u16,
        );
        read_le_u16(&self.ram, OAM_CUR_PTR) as usize
    }

    fn hit_stars_update_oam_buffer_position(&mut self, oam: usize) -> usize {
        let mut oam = oam;
        if self.ram[SORT_SPRITES_SETTING] == 0 && oam >= 0x9d0 {
            write_le_u16(&mut self.ram, OAM_CUR_PTR, 0x820);
            write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0xa20 + (0x20 >> 2));
            oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        }
        oam
    }

    fn ancilla_check_for_entrance_trigger(&self, what: usize) -> bool {
        const ENTRANCE_TRIGGER_BASE_Y: [u16; 4] = [0x0d40, 0x0210, 0x0cfc, 0x0100];
        const ENTRANCE_TRIGGER_BASE_X: [u16; 4] = [0x0d80, 0x0e68, 0x0130, 0x0f10];
        const ENTRANCE_TRIGGER_SIZE_Y: [u16; 4] = [11, 32, 16, 12];
        const ENTRANCE_TRIGGER_SIZE_X: [u16; 4] = [16, 16, 16, 16];

        abs16(
            self.player_state_view()
                .y()
                .wrapping_add(12)
                .wrapping_sub(ENTRANCE_TRIGGER_BASE_Y[what]),
        ) < ENTRANCE_TRIGGER_SIZE_Y[what]
            && abs16(
                self.player_state_view()
                    .x()
                    .wrapping_add(8)
                    .wrapping_sub(ENTRANCE_TRIGGER_BASE_X[what]),
            ) < ENTRANCE_TRIGGER_SIZE_X[what]
    }

    fn game_over_text_draw(&mut self) {
        const GAME_OVER_TEXT_CHARS: [u8; 16] = [
            0x40, 0x50, 0x41, 0x51, 0x42, 0x52, 0x43, 0x53, 0x44, 0x54, 0x45, 0x55, 0x43, 0x53,
            0x46, 0x56,
        ];

        write_le_u16(&mut self.ram, OAM_CUR_PTR, 0x0800);
        write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0x0a20);
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut k = self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] as i32;
        loop {
            let j = k as usize * 2;
            let x = self.ancilla_get_x(k as usize);
            self.ancilla_set_oam(oam, x, 0x57, GAME_OVER_TEXT_CHARS[j], 0x3c, 0);
            self.ancilla_set_oam(oam + 4, x, 0x5f, GAME_OVER_TEXT_CHARS[j + 1], 0x3c, 0);
            oam += 8;
            k -= 1;
            if k < 0 {
                break;
            }
        }
    }

    fn ancilla_draw_shadow(&mut self, oam: usize, k: usize, mut x: u16, y: u16, pal: u8) {
        const ANCILLA_DRAW_SHADOW_CHAR: [u8; 14] = [
            0x6c, 0x6c, 0x28, 0x28, 0x38, 0xff, 0xc8, 0xc8, 0xd8, 0xd8, 0xd9, 0xd9, 0xda, 0xda,
        ];
        const ANCILLA_DRAW_SHADOW_FLAGS: [u8; 14] = [
            0x28, 0x68, 0x28, 0x68, 0x28, 0xff, 0x22, 0x22, 0x24, 0x64, 0x24, 0x64, 0x24, 0x64,
        ];

        if k == 2 {
            x = x.wrapping_add(4);
        }
        self.ancilla_set_oam_safe(
            oam,
            x,
            y,
            ANCILLA_DRAW_SHADOW_CHAR[k * 2],
            ANCILLA_DRAW_SHADOW_FLAGS[k * 2] & !0x30 | pal,
            0,
        );
        let ch = ANCILLA_DRAW_SHADOW_CHAR[k * 2 + 1];
        if ch != 0xff {
            x = x.wrapping_add(8);
            self.ancilla_set_oam_safe(
                oam + 4,
                x,
                y,
                ch,
                ANCILLA_DRAW_SHADOW_FLAGS[k * 2 + 1] & !0x30 | pal,
                0,
            );
        }
    }

    fn sprite_spawn_dynamically_for_ancilla(&mut self, k: usize, sprite: u8) -> Option<usize> {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, sprite, &mut info);
        if j >= 0 {
            Some(j as usize)
        } else {
            None
        }
    }

    fn sprite_place_rupulse_spark_2_for_ancilla(&mut self, k: usize) {
        let x = self
            .sprite_get_x(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let y = self
            .sprite_get_y(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        if x & !0xff != 0 || y & !0xff != 0 {
            return;
        }
        self.ram[REPULSESPARK_X_LO_ANCILLA] = self.ram[SPRITE_X_LO + k];
        self.ram[REPULSESPARK_Y_LO_ANCILLA] = self.ram[SPRITE_Y_LO + k];
        self.ram[REPULSESPARK_TIMER_ANCILLA] = 5;
        self.ram[REPULSESPARK_FLOOR_STATUS_ANCILLA] = self.ram[SPRITE_FLOOR + k];
    }

    fn sprite_place_weapon_tink_for_ancilla(&mut self, k: usize) {
        if self.ram[REPULSESPARK_TIMER_ANCILLA] != 0 {
            return;
        }
        self.sprite_sfx_queue_sfx2_with_pan(k, 5);
        self.sprite_place_rupulse_spark_2_for_ancilla(k);
    }

    fn sprite_create_deflected_arrow(&mut self, k: usize) {
        self.ram[ANCILLA_TYPE + k] = 0;
        if let Some(j) = self.sprite_spawn_dynamically_for_ancilla(k, 0x1b) {
            self.ram[SPRITE_X_LO + j] = self.ram[ANCILLA_X_LO + k];
            self.ram[SPRITE_X_HI + j] = self.ram[ANCILLA_X_HI + k];
            self.ram[SPRITE_Y_LO + j] = self.ram[ANCILLA_Y_LO + k];
            self.ram[SPRITE_Y_HI + j] = self.ram[ANCILLA_Y_HI + k];
            self.ram[SPRITE_STATE + j] = 6;
            self.ram[SPRITE_DELAY_MAIN + j] = 31;
            self.ram[SPRITE_X_VEL + j] = self.ram[ANCILLA_X_VEL + k];
            self.ram[SPRITE_Y_VEL + j] = self.ram[ANCILLA_Y_VEL + k];
            self.ram[SPRITE_FLOOR + j] = self.ram[LINK_IS_ON_LOWER_LEVEL];
            self.sprite_place_weapon_tink_for_ancilla(j);
        }
    }

    fn ancilla_check_damage_to_sprite(&mut self, k: usize, ty: u8) {
        if !sign8(self.ram[SPRITE_HIT_TIMER_ANCILLA + k]) {
            self.ancilla_check_damage_to_sprite_aggressive(k, ty);
        }
    }

    fn ancilla_check_damage_to_sprite_aggressive(&mut self, k: usize, ty: u8) {
        const ANCILLA_DAMAGE: [u8; 57] = [
            6, 1, 11, 0, 0, 0, 0, 8, 0, 6, 0, 12, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 14, 13, 0, 0,
            15, 0, 0, 7, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 11, 0, 1, 1, 1, 1, 1, 1, 1,
            1,
        ];

        let mut dmg = ANCILLA_DAMAGE[ty as usize];
        if dmg == 6 && self.ram[LINK_ITEM_BOW] >= 3 {
            if self.ram[SPRITE_TYPE + k] == 0xd7 {
                self.ram[SPRITE_DELAY_AUX4 + k] = 32;
            }
            dmg = 9;
        }
        self.ancilla_check_damage_to_sprite_preset(k, dmg);
    }

    fn medallion_check_sprite_damage(&mut self, k: usize) {
        self.ram[TMP_COUNTER_ANCILLA] = self.ram[ANCILLA_TYPE + k];
        for j in (0..16).rev() {
            if self.ram[SPRITE_STATE + j] >= 9
                && (self.ram[SPRITE_IGNORE_PROJECTILE_ANCILLA + j] | self.ram[SPRITE_PAUSE + j])
                    == 0
            {
                self.ancilla_check_damage_to_sprite_aggressive(j, self.ram[TMP_COUNTER_ANCILLA]);
            }
        }
    }

    fn sprite_func15_for_ancilla(&mut self, k: usize, a: u8) {
        self.ram[DAMAGE_TYPE_DETERMINER_ANCILLA] = a;
        self.sprite_apply_calculated_damage_for_ancilla(k, if a == 8 { 0x35 } else { 0x20 });
    }

    fn sprite_apply_calculated_damage_for_ancilla(&mut self, k: usize, a: u8) {
        const ENEMY_DAMAGES: [u8; 128] = [
            0, 1, 32, 255, 252, 251, 0, 0, 0, 2, 64, 4, 0, 0, 0, 0, 0, 4, 64, 2, 3, 0, 0, 0, 0, 8,
            64, 4, 0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 4, 64, 16, 0,
            0, 0, 0, 0, 255, 64, 255, 252, 251, 0, 0, 0, 4, 64, 255, 252, 251, 32, 0, 0, 100, 24,
            100, 0, 0, 0, 0, 0, 249, 250, 255, 100, 0, 0, 0, 0, 8, 64, 253, 4, 16, 0, 0, 0, 8, 64,
            254, 4, 0, 0, 0, 0, 16, 64, 253, 0, 0, 0, 0, 0, 254, 64, 16, 0, 0, 0, 0, 0, 32, 64,
            255, 0, 0, 0, 250,
        ];

        if self.ram[SPRITE_FLAGS3 + k] & 0x40 != 0 || self.ram[SPRITE_TYPE + k] >= 0xd8 {
            return;
        }
        let damage_type = self.ram[DAMAGE_TYPE_DETERMINER_ANCILLA] as usize;
        let enemy_damage_index = self.ram[SPRITE_TYPE + k] as usize * 16
            + self.ram[DAMAGE_TYPE_DETERMINER_ANCILLA] as usize;
        let dmg = ENEMY_DAMAGES
            [damage_type * 8 | self.ram[ENEMY_DAMAGE_DATA + enemy_damage_index] as usize];
        self.sprite_give_damage_for_ancilla(k, dmg, a);
    }

    fn sprite_give_damage_for_ancilla(&mut self, k: usize, dmg: u8, r0_hit_timer: u8) {
        if dmg == 249 {
            self.sprite_func18_for_ancilla(k, 0xe3);
            return;
        }
        if dmg == 250 {
            self.sprite_func18_for_ancilla(k, 0x8f);
            self.ram[SPRITE_AI_STATE + k] = 2;
            self.ram[SPRITE_Z_VEL + k] = 32;
            self.ram[SPRITE_OAM_FLAGS_ANCILLA + k] = 8;
            self.ram[SPRITE_F_ANCILLA + k] = 0;
            self.ram[SPRITE_HIT_TIMER_ANCILLA + k] = 0;
            self.ram[SPRITE_HEALTH_ANCILLA + k] = 0;
            self.ram[SPRITE_BUMP_DAMAGE_ANCILLA + k] = 1;
            self.ram[SPRITE_FLAGS5 + k] = 1;
            return;
        }
        if dmg >= self.ram[SPRITE_GIVE_DAMAGE_ANCILLA + k] {
            self.ram[SPRITE_GIVE_DAMAGE_ANCILLA + k] = dmg;
        }
        if dmg == 0 {
            if self.ram[DAMAGE_TYPE_DETERMINER_ANCILLA] != 10 {
                if self.ram[SPRITE_FLAGS_ANCILLA + k] & 4 != 0 {
                    self.sprite_set_damage_stun_for_ancilla(k);
                    return;
                }
                self.ram[LINK_SWORD_DELAY_TIMER] = 0;
            }
            self.ram[SPRITE_HIT_TIMER_ANCILLA + k] = 0;
            self.ram[SPRITE_GIVE_DAMAGE_ANCILLA + k] = 0;
            return;
        }
        if dmg >= 254 && self.ram[SPRITE_STATE + k] == 11 {
            self.ram[SPRITE_HIT_TIMER_ANCILLA + k] = 0;
            self.ram[SPRITE_GIVE_DAMAGE_ANCILLA + k] = 0;
            return;
        }
        if self.ram[SPRITE_TYPE + k] == 0x9a && self.ram[SPRITE_GIVE_DAMAGE_ANCILLA + k] < 0xf0 {
            self.ram[SPRITE_STATE + k] = 9;
            self.ram[SPRITE_AI_STATE + k] = 4;
            self.ram[SPRITE_DELAY_MAIN + k] = 15;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
            return;
        }
        if self.ram[SPRITE_TYPE + k] == 0x1b {
            self.sprite_sfx_queue_sfx2_with_pan(k, 5);
            self.sprite_schedule_for_breakage_for_ancilla(k);
            self.sprite_place_weapon_tink_for_ancilla(k);
            return;
        }
        self.ram[SPRITE_HIT_TIMER_ANCILLA + k] = r0_hit_timer;
        if self.ram[SPRITE_TYPE + k] != 0x92 || self.ram[SPRITE_C_ANCILLA + k] >= 3 {
            let sfx = if self.ram[SPRITE_FLAGS_ANCILLA + k] & 2 != 0 {
                0x21
            } else if self.ram[SPRITE_FLAGS5 + k] & 0x10 != 0 {
                0x1c
            } else {
                8
            };
            self.ram[SOUND_EFFECT_2] = sfx | self.sprite_calculate_sfx_pan(k);
        }
        self.sprite_set_damage_stun_for_ancilla(k);
    }

    fn sprite_set_damage_stun_for_ancilla(&mut self, k: usize) {
        let ty = self.ram[SPRITE_TYPE + k];
        self.ram[SPRITE_F_ANCILLA + k] = if self.ram[DAMAGE_TYPE_DETERMINER_ANCILLA] >= 13 {
            0
        } else if ty == 9 {
            20
        } else if ty == 0x53 || ty == 0x18 {
            11
        } else {
            15
        };
    }

    fn sprite_schedule_for_breakage_for_ancilla(&mut self, k: usize) {
        self.ram[SPRITE_DELAY_MAIN + k] = 31;
        self.ram[SPRITE_STATE + k] = 6;
        self.ram[SPRITE_FLAGS2 + k] = self.ram[SPRITE_FLAGS2 + k].wrapping_add(4);
    }

    fn sprite_func18_for_ancilla(&mut self, k: usize, new_type: u8) {
        self.ram[SPRITE_TYPE + k] = new_type;
        self.sprite_prep_load_properties(k);
        self.ram[SOUND_EFFECT_2] = 0;
    }

    fn ancilla_check_sprite_collision(&mut self, k: usize) -> Option<usize> {
        for j in (0..16).rev() {
            if (self.ram[ANCILLA_TYPE + k] == 9
                || self.ram[ANCILLA_TYPE + k] == 0x1f
                || (((j as u8 ^ self.ram[FRAME_COUNTER]) & 3) | self.ram[SPRITE_PAUSE + j]) == 0)
                && self.ram[SPRITE_STATE + j] >= 9
                && (self.ram[SPRITE_DEFL_BITS + j] & 2 != 0 || self.ram[ANCILLA_OBJPRIO + k] == 0)
                && self.ram[ANCILLA_FLOOR + k] == self.ram[SPRITE_FLOOR + j]
                && self.ancilla_check_sprite_collision_single(k, j)
            {
                return Some(j);
            }
        }
        None
    }

    fn ancilla_check_sprite_collision_single(&mut self, k: usize, j: usize) -> bool {
        let mut hb = self.ancilla_setup_hit_box(k);
        self.sprite_setup_hit_box(j, &mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if std::env::var_os("ZELDA3_TRACE_ANCILLA_COLL").is_some()
            && j == 2
            && self.ram[FRAME_COUNTER] >= 160
            && self.ram[FRAME_COUNTER] <= 210
        {
            eprintln!(
                "R ancilla-coll fc={} k={} atype=0x{:02x} j={} stype=0x{:02x} overlap={} ax={:04x} ay={:04x} az={:02x} hb={:02x}/{:02x} {:02x}/{:02x} size={:02x}/{:02x} spr={:02x}/{:02x} {:02x}/{:02x} ssize={:02x}/{:02x} dir={:02x} hit={:02x} pause={:02x} floor={:02x}/{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ram[ANCILLA_TYPE + k],
                j,
                self.ram[SPRITE_TYPE + j],
                overlap,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                self.ram[ANCILLA_Z + k],
                hb.r0_xlo,
                hb.r8_xhi,
                hb.r1_ylo,
                hb.r9_yhi,
                hb.r2,
                hb.r3,
                hb.r4_spr_xlo,
                hb.r10_spr_xhi,
                hb.r5_spr_ylo,
                hb.r11_spr_yhi,
                hb.r6_spr_xsize,
                hb.r7_spr_ysize,
                self.ram[ANCILLA_DIR + k],
                self.ram[SPRITE_HIT_TIMER_ANCILLA + j],
                self.ram[SPRITE_PAUSE + j],
                self.ram[ANCILLA_FLOOR + k],
                self.ram[SPRITE_FLOOR + j],
            );
        }
        if !overlap {
            return false;
        }

        let mut return_value = true;
        if self.ram[SPRITE_FLAGS_ANCILLA + j] & 8 != 0 && self.ram[ANCILLA_TYPE + k] == 9 {
            if self.ram[SPRITE_TYPE + j] != 0x1b {
                self.sprite_create_deflected_arrow(k);
                return false;
            }
            if self.ram[LINK_ITEM_BOW] < 3 {
                self.sprite_create_deflected_arrow(k);
            } else {
                return_value = false;
            }
        }

        let mut return_true_set_alert = false;
        if self.ram[SPRITE_DEFL_BITS + j] & 0x10 != 0 {
            const ANCILLA_CHECK_SPRITE_COLL_DIR: [u8; 4] = [2, 3, 0, 1];
            self.ram[ANCILLA_DIR + k] &= 3;
            if self.ram[ANCILLA_DIR + k]
                == ANCILLA_CHECK_SPRITE_COLL_DIR[self.ram[ANCILLA_DIR + k] as usize]
            {
                return_true_set_alert = true;
            }
        }

        if !return_true_set_alert
            && (self.ram[ANCILLA_TYPE + k] == 5 || self.ram[ANCILLA_TYPE + k] == 0x1f)
        {
            let skip = self.ram[ANCILLA_TYPE + k] == 0x1f && self.ram[SPRITE_TYPE + j] == 0x8d;
            if !skip && self.ram[SPRITE_HIT_TIMER_ANCILLA + j] != 0 {
                return_true_set_alert = true;
            } else if skip || self.ram[SPRITE_DEFL_BITS + j] & 2 != 0 {
                self.ram[SPRITE_B_ANCILLA + j] = k as u8 + 1;
                self.ram[SPRITE_UNK2_ANCILLA + j] = self.ram[ANCILLA_TYPE + k];
                return_true_set_alert = true;
            }
        }

        if !return_true_set_alert && self.ram[SPRITE_IGNORE_PROJECTILE_ANCILLA + j] == 0 {
            const ANCILLA_CHECK_SPRITE_COLL_RECOIL_X: [u8; 4] = [0, 0, 0xc0, 0x40];
            const ANCILLA_CHECK_SPRITE_COLL_RECOIL_Y: [u8; 4] = [0xc0, 0x40, 0, 0];
            if self.ram[SPRITE_TYPE + j] == 0x92 && self.ram[SPRITE_C_ANCILLA + j] < 3 {
                return_true_set_alert = true;
            } else {
                let i = (self.ram[ANCILLA_DIR + k] & 3) as usize;
                self.ram[SPRITE_X_RECOIL + j] = ANCILLA_CHECK_SPRITE_COLL_RECOIL_X[i];
                self.ram[SPRITE_Y_RECOIL_ANCILLA + j] = ANCILLA_CHECK_SPRITE_COLL_RECOIL_Y[i];
                self.ram[SPRITE_SHARED_SCRATCH_A] = k as u8;
                self.ancilla_check_damage_to_sprite(j, self.ram[ANCILLA_TYPE + k]);
                return_true_set_alert = true;
            }
        } else if !return_true_set_alert {
            return false;
        }

        if return_true_set_alert {
            self.ram[SPRITE_UNK2_ANCILLA + j] = self.ram[ANCILLA_TYPE + k];
            self.ram[SPRITE_ALERT_FLAG] = 3;
            return return_value;
        }
        false
    }

    fn ancilla_check_basic_sprite_collision(&mut self, k: usize) -> Option<usize> {
        for j in (0..16).rev() {
            if (((j as u8 ^ self.ram[FRAME_COUNTER]) & 3)
                | self.ram[SPRITE_PAUSE + j]
                | self.ram[SPRITE_HIT_TIMER_ANCILLA + j])
                != 0
            {
                continue;
            }
            if self.ram[SPRITE_STATE + j] < 9
                || (self.ram[SPRITE_DEFL_BITS + j] & 2 == 0 && self.ram[ANCILLA_OBJPRIO + k] != 0)
                || self.ram[ANCILLA_FLOOR + k] != self.ram[SPRITE_FLOOR + j]
                || self.ram[ANCILLA_TYPE + k] == 0x2c
                    && (self.ram[SPRITE_TYPE + j] == 0x1e || self.ram[SPRITE_TYPE + j] == 0x90)
            {
                continue;
            }
            if self.ancilla_check_basic_sprite_collision_single(k, j) {
                return Some(j);
            }
        }
        None
    }

    fn ancilla_check_basic_sprite_collision_single(&mut self, k: usize, j: usize) -> bool {
        let mut hb = self.ancilla_setup_basic_hit_box(k);
        self.sprite_setup_hit_box(j, &mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if std::env::var_os("ZELDA3_TRACE_ANCILLA_COLL").is_some()
            && j == 2
            && self.ram[FRAME_COUNTER] >= 160
            && self.ram[FRAME_COUNTER] <= 210
        {
            eprintln!(
                "R ancilla-basic-coll fc={} k={} atype=0x{:02x} j={} stype=0x{:02x} overlap={} ax={:04x} ay={:04x} az={:02x} hb={:02x}/{:02x} {:02x}/{:02x} size={:02x}/{:02x} spr={:02x}/{:02x} {:02x}/{:02x} ssize={:02x}/{:02x} dir={:02x} hit={:02x} pause={:02x} floor={:02x}/{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.ram[ANCILLA_TYPE + k],
                j,
                self.ram[SPRITE_TYPE + j],
                overlap,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                self.ram[ANCILLA_Z + k],
                hb.r0_xlo,
                hb.r8_xhi,
                hb.r1_ylo,
                hb.r9_yhi,
                hb.r2,
                hb.r3,
                hb.r4_spr_xlo,
                hb.r10_spr_xhi,
                hb.r5_spr_ylo,
                hb.r11_spr_yhi,
                hb.r6_spr_xsize,
                hb.r7_spr_ysize,
                self.ram[ANCILLA_DIR + k],
                self.ram[SPRITE_HIT_TIMER_ANCILLA + j],
                self.ram[SPRITE_PAUSE + j],
                self.ram[ANCILLA_FLOOR + k],
                self.ram[SPRITE_FLOOR + j],
            );
        }
        if !overlap {
            return false;
        }
        if self.ram[SPRITE_TYPE + j] == 0x92 && self.ram[SPRITE_C_ANCILLA + j] < 3 {
            return true;
        }
        if self.ram[SPRITE_TYPE + j] == 0x80 && self.ram[SPRITE_DELAY_AUX4 + j] == 0 {
            self.ram[SPRITE_DELAY_AUX4 + j] = 24;
            self.ram[SPRITE_D + j] ^= 1;
        }
        if self.ram[SPRITE_IGNORE_PROJECTILE_ANCILLA + j] != 0 {
            return false;
        }

        let x = self.ancilla_get_x(k).wrapping_sub(8);
        let y = self
            .ancilla_get_y(k)
            .wrapping_sub(8)
            .wrapping_sub(self.ram[ANCILLA_Z + k] as u16);
        let pt = self.sprite_project_speed_towards_location(j, x, y, 80);
        self.ram[SPRITE_Y_RECOIL_ANCILLA + j] = !pt.y;
        self.ram[SPRITE_X_RECOIL + j] = !pt.x;
        self.ancilla_check_damage_to_sprite(j, self.ram[ANCILLA_TYPE + k]);
        true
    }

    fn bomb_check_underside_sprite_status(&mut self, k: usize, pt: &mut Point16U) -> Option<u8> {
        if self.ram[ANCILLA_ITEM_TO_LINK + k] != 0 {
            return None;
        }

        let mut r10 = 0;
        if self.ram[ANCILLA_TILE_ATTR_PLAYER + k] == 9 {
            self.ram[ANCILLA_ARR22 + k] = self.ram[ANCILLA_ARR22 + k].wrapping_sub(1);
            if sign8(self.ram[ANCILLA_ARR22 + k]) {
                self.ram[ANCILLA_ARR22 + k] = 3;
                self.ram[ANCILLA_ARR23 + k] = self.ram[ANCILLA_ARR23 + k].wrapping_add(1);
                if self.ram[ANCILLA_ARR23 + k] == 3 {
                    self.ram[ANCILLA_ARR23 + k] = 0;
                }
            }
            r10 = self.ram[ANCILLA_ARR23 + k].wrapping_add(4);
            if self.ram[SOUND_EFFECT_1] & 0x3f == 0x0b || self.ram[SOUND_EFFECT_1] & 0x3f == 0x21 {
                self.ram[SOUND_EFFECT_1] = self.ancilla_calculate_sfx_pan(k) | 0x28;
            }
        } else if self.ram[ANCILLA_TILE_ATTR_PLAYER + k] == 0x40 {
            r10 = 3;
        }

        if self.ram[ANCILLA_Z + k] >= 2 && self.ram[ANCILLA_Z + k] < 252 {
            r10 = 2;
        }
        if k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize
            && self.ram[LINK_STATE_BITS] & 0x80 != 0
        {
            return None;
        }
        let z = self.ram[ANCILLA_Z + k] as i8;
        pt.y = pt.y.wrapping_add(z as i16 as u16).wrapping_add(2);
        pt.x = pt.x.wrapping_sub(8);
        Some(r10)
    }

    fn ancilla_draw_explosion(
        &mut self,
        mut oam: usize,
        mut frame: usize,
        mut idx: usize,
        idx_end: usize,
        r11: u8,
        x: u16,
        y: u16,
    ) -> usize {
        const BOMB_DRAW_EXPLOSION_XY: [i8; 108] = [
            -8, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -8, -8, -8, 0, 0, -8, 0, 0, 0, 0, 0, 0, -16, -16,
            -16, 0, 0, -16, 0, 0, 0, 0, 0, 0, -16, -16, -16, 0, 0, -16, 0, 0, 0, 0, 0, 0, -8, -8,
            -21, -22, -21, 8, 9, -22, 9, 8, 0, 0, -6, -15, 0, -1, -16, -2, -8, -7, 0, 0, 0, 0, -9,
            -4, -21, -5, -12, -18, -11, 7, 0, -15, 4, -2, -9, -4, -22, -5, -13, -20, -11, 8, 1,
            -16, 5, -2, -20, 4, -12, -19, -9, 16, -5, -2, 2, -9, 10, 6,
        ];
        const BOMB_DRAW_EXPLOSION_CHAR_FLAGS: [u8; 108] = [
            0x6e, 0x26, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x8c, 0x22,
            0x8c, 0x62, 0x8c, 0xa2, 0x8c, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x84, 0x22, 0x84, 0x62,
            0x84, 0xa2, 0x84, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x88, 0x22, 0x88, 0x62, 0x88, 0xa2,
            0x88, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x86, 0x22, 0x88, 0x22, 0x88, 0x62, 0x88, 0xa2,
            0x88, 0xe2, 0xff, 0xff, 0x86, 0x22, 0x86, 0x62, 0x86, 0xe2, 0x86, 0xe2, 0xff, 0xff,
            0xff, 0xff, 0x86, 0xe2, 0x86, 0x22, 0x86, 0x22, 0x86, 0x62, 0x86, 0xa2, 0x86, 0xa2,
            0x8a, 0xa2, 0x8a, 0x62, 0x8a, 0x22, 0x8a, 0x62, 0x8a, 0x62, 0x8a, 0xe2, 0x9b, 0x22,
            0x9b, 0xa2, 0x9b, 0x62, 0x9b, 0xe2, 0x9b, 0xa2, 0x9b, 0x22,
        ];
        const BOMB_DRAW_EXPLOSION_EXT: [u8; 54] = [
            2, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2,
            1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0,
        ];

        let base_frame = frame;
        loop {
            if BOMB_DRAW_EXPLOSION_CHAR_FLAGS[frame * 2] != 0xff {
                let i = idx + base_frame;
                self.ancilla_set_oam_safe(
                    oam,
                    x.wrapping_add(BOMB_DRAW_EXPLOSION_XY[i * 2 + 1] as i16 as u16),
                    y.wrapping_add(BOMB_DRAW_EXPLOSION_XY[i * 2] as i16 as u16),
                    BOMB_DRAW_EXPLOSION_CHAR_FLAGS[frame * 2],
                    BOMB_DRAW_EXPLOSION_CHAR_FLAGS[frame * 2 + 1] & !0x3e
                        | self.ram[OAM_PRIORITY_VALUE + 1]
                        | r11,
                    BOMB_DRAW_EXPLOSION_EXT[frame],
                );
                oam += 4;
            }
            frame += 1;
            idx += 1;
            if idx == idx_end {
                break;
            }
        }
        oam
    }

    fn bomb_draw(&mut self, k: usize) {
        let (x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        let z = self.ram[ANCILLA_Z + k] as i8;
        if z != 0 && z != -1 && self.ram[ANCILLA_K + k] != 3 && self.ram[ANCILLA_OBJPRIO + k] != 0 {
            write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, 0x3000);
        }
        y = y.wrapping_sub(z as i16 as u16);
        let j = K_BOMB_DRAW_TAB0[self.ram[ANCILLA_ITEM_TO_LINK + k] as usize] as usize * 6;

        let mut r11 = 2;
        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0 {
            r11 = if self.ram[ANCILLA_ARR3 + k] < 0x20 {
                self.ram[ANCILLA_ARR3 + k] & 0x0e
            } else {
                4
            };
        }

        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0 {
            if self.ram[ANCILLA_L + k] == 0
                && (self.ram[SPRITE_TYPE] == 0x92
                    || k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize)
                && (self.ram[LINK_STATE_BITS] & 0x80 == 0
                    || self.ram[ANCILLA_K + k] != 3 && self.ram[LINK_DIRECTION_FACING] == 0)
            {
                self.ancilla_allocate_oam_from_region_b_or_e(12);
            } else if self.ram[SORT_SPRITES_SETTING] != 0
                && self.ram[ANCILLA_FLOOR + k] != 0
                && (self.ram[ANCILLA_L + k] != 0
                    || k + 1 == self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] as usize
                        && self.ram[LINK_STATE_BITS] & 0x80 != 0)
            {
                write_le_u16(&mut self.ram, OAM_CUR_PTR, 0x0800 + 0x34 * 4);
                write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0x0a20 + 0x34);
            }
        }

        let oam_org = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let numframes = K_BOMB_DRAW_TAB2[self.ram[ANCILLA_ITEM_TO_LINK + k] as usize] as usize;
        let mut oam = oam_org;
        if self.ram[ANCILLA_ITEM_TO_LINK + k] == 0
            && (self.ram[ANCILLA_TILE_ATTR_PLAYER + k] == 9
                || self.ram[ANCILLA_TILE_ATTR_PLAYER + k] == 0x40)
        {
            oam += 8;
        }

        self.ancilla_draw_explosion(oam, j, 0, numframes, r11, x, y);
        oam += numframes * 4;

        let mut pt = Point16U { x, y };
        if let Some(r10) = self.bomb_check_underside_sprite_status(k, &mut pt) {
            if oam != oam_org + 4 {
                oam = oam_org;
            }
            self.ancilla_draw_shadow(
                oam,
                r10 as usize,
                pt.x,
                pt.y,
                self.ram[OAM_PRIORITY_VALUE + 1],
            );
        }
    }

    fn ancilla32_blast_wall_fireball(&mut self, k: usize) {
        const BLAST_WALL_FIREBALL_CHAR: [u8; 3] = [0x9d, 0x9c, 0x8d];

        if self.frame_control_view().submodule() == 0 {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = self.ram[ANCILLA_ITEM_TO_LINK + k].wrapping_add(2);
            self.ram[ANCILLA_Y_VEL + k] =
                self.ram[ANCILLA_Y_VEL + k].wrapping_add(self.ram[ANCILLA_ITEM_TO_LINK + k]);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            self.ram[BLASTWALL_VAR12 + k] = self.ram[BLASTWALL_VAR12 + k].wrapping_sub(1);
            if sign8(self.ram[BLASTWALL_VAR12 + k]) {
                self.ram[ANCILLA_TYPE + k] = 0;
                return;
            }
        }

        if self.ram[SORT_SPRITES_SETTING] != 0 {
            self.oam_allocate_from_region_d(4);
        } else {
            self.oam_allocate_from_region_a(4);
        }

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let j = if self.ram[BLASTWALL_VAR12 + k] & 8 != 0 {
            0
        } else if self.ram[BLASTWALL_VAR12 + k] & 4 != 0 {
            1
        } else {
            2
        };
        self.ancilla_set_oam(
            read_le_u16(&self.ram, OAM_CUR_PTR) as usize,
            x,
            y,
            BLAST_WALL_FIREBALL_CHAR[j],
            0x22,
            0,
        );
    }

    fn ancilla33_blast_wall_explosion(&mut self, k: usize) {
        if self.frame_control_view().submodule() == 0 {
            if self.ram[BLASTWALL_VAR5 + k] != 0 {
                self.ram[BLASTWALL_VAR6 + k] = self.ram[BLASTWALL_VAR6 + k].wrapping_sub(1);
                if self.ram[BLASTWALL_VAR6 + k] == 0 {
                    self.ram[BLASTWALL_VAR5 + k] = self.ram[BLASTWALL_VAR5 + k].wrapping_add(1);
                    if self.ram[BLASTWALL_VAR5 + k] != 0 && self.ram[BLASTWALL_VAR5 + k] < 9 {
                        self.ancilla_add_blast_wall_fireball(0x32, 10, k * 4);
                    }
                    if self.ram[BLASTWALL_VAR5 + k] == 11 {
                        self.ram[BLASTWALL_VAR5 + k] = 0;
                        self.ram[BLASTWALL_VAR6 + k] = 0;
                    } else {
                        self.ram[BLASTWALL_VAR6 + k] = 3;
                    }
                }
            } else {
                let k = k ^ 1;
                if self.ram[BLASTWALL_VAR5 + k] == 6
                    && self.ram[BLASTWALL_VAR6 + k] == 2
                    && self.ram[ANCILLA_ITEM_TO_LINK].wrapping_add(1) < 7
                {
                    self.ram[ANCILLA_ITEM_TO_LINK] = self.ram[ANCILLA_ITEM_TO_LINK].wrapping_add(1);
                    self.ram[BLASTWALL_VAR5 + k] = 1;
                    self.ram[BLASTWALL_VAR6 + k] = 3;
                    for i in (0..=3).rev() {
                        let mut arr = [0i8, 0i8];
                        let j = if self.ram[BLASTWALL_VAR7] < 4 { 1 } else { 0 };
                        arr[j] = if i & 2 != 0 { -13 } else { 13 };
                        let j = k * 4 + i;
                        let y = read_le_u16(&self.ram, BLASTWALL_VAR10 + j * 2)
                            .wrapping_add(arr[0] as i16 as u16);
                        let x = read_le_u16(&self.ram, BLASTWALL_VAR11 + j * 2)
                            .wrapping_add(arr[1] as i16 as u16);
                        write_le_u16(&mut self.ram, BLASTWALL_VAR10 + j * 2, y);
                        write_le_u16(&mut self.ram, BLASTWALL_VAR11 + j * 2, x);
                        let x = x.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
                        if x < 256 {
                            self.ram[SOUND_EFFECT_1] = K_BOMBOS_SFX[(x >> 5) as usize] | 0x0c;
                        }
                    }
                }
            }
        }

        let k = self.ram[ANCILLA_K] as usize;
        if self.ram[BLASTWALL_VAR5 + k] != 0 {
            let first_i = if k == 1 { 7 } else { 3 };
            for i in (first_i - 3..=first_i).rev() {
                self.ancilla_draw_blast_wall_blast(
                    k,
                    read_le_u16(&self.ram, BLASTWALL_VAR11 + i * 2),
                    read_le_u16(&self.ram, BLASTWALL_VAR10 + i * 2),
                );
            }
        }
        if self.ram[ANCILLA_ITEM_TO_LINK] == 6
            && self.ram[BLASTWALL_VAR5] == 0
            && self.ram[BLASTWALL_VAR5 + 1] == 0
        {
            self.ram[ANCILLA_TYPE] = 0;
            self.ram[ANCILLA_TYPE + 1] = 0;
            self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 0;
        }
    }

    fn ancilla_draw_blast_wall_blast(&mut self, k: usize, x: u16, y: u16) {
        write_le_u16(&mut self.ram, OAM_PRIORITY_VALUE, 0x3000);
        if self.ram[SORT_SPRITES_SETTING] != 0 {
            self.oam_allocate_from_region_d(0x18);
        } else {
            self.oam_allocate_from_region_a(0x18);
        }
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let i = self.ram[BLASTWALL_VAR5 + k] as usize;
        self.ancilla_draw_explosion(
            oam,
            K_BOMB_DRAW_TAB0[i] as usize * 6,
            0,
            K_BOMB_DRAW_TAB2[i] as usize,
            0x32,
            x.wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)),
            y.wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)),
        );
    }

    fn ancilla_calculate_sfx_pan(&self, k: usize) -> u8 {
        Self::calculate_sfx_pan_with_scroll(
            self.ancilla_get_x(k),
            read_le_u16(&self.ram, BG2HOFS_COPY2),
        )
    }

    fn get_tile_attribute_for_ancilla(&mut self, floor: u8, mut x: u16, y: u16) -> u8 {
        let tiletype = if self.ram[PLAYER_IS_INDOORS] != 0 {
            let mut t = if floor >= 1 { 0x1000 } else { 0 };
            t += ((x & 0x01f8) >> 3) as usize;
            t += ((y & 0x01f8) << 3) as usize;
            self.ram[DUNG_BG2_ATTR_TABLE + t]
        } else {
            x >>= 3;
            self.overworld_get_tile_attribute_at_location(x, y)
        };
        self.ram[SPRITE_TILETYPE_ANCILLA] = tiletype;
        tiletype
    }

    fn entity_check_sloped_tile_collision_for_ancilla(&self, x: u16, y: u16) -> bool {
        let a = (y & 7) as u8;
        let r6 = self.ram[SPRITE_TILETYPE_ANCILLA].wrapping_sub(0x10);
        let b = K_SLOPED_TILE[(r6 as usize) * 8 + (x as usize & 7)];
        if r6 < 2 {
            b >= a
        } else {
            a >= b
        }
    }

    fn ancilla_set_oam(&mut self, oam: usize, x: u16, y: u16, charnum: u8, flags: u8, mut big: u8) {
        let mut yval = 0xf0;
        let xt: u16 = if self.read_u32_ram(ENHANCED_FEATURES0) & 1 != 0 {
            0x40
        } else {
            0
        };
        if x.wrapping_add(xt) < 256 + xt * 2 && y < 256 {
            big |= ((x >> 8) as u8) & 1;
            self.ram[oam] = x as u8;
            if y < 0xf0 {
                yval = y as u8;
            }
        }
        self.ram[oam + 1] = yval;
        self.ram[oam + 2] = charnum;
        self.ram[oam + 3] = flags;
        self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4] = big;
    }

    fn ancilla_set_oam_plain(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.ram[oam] = x as u8;
        self.ram[oam + 1] = y as u8;
        self.ram[oam + 2] = charnum;
        self.ram[oam + 3] = flags;
        self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4] = big;
    }

    fn ancilla_set_oam_safe(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        mut big: u8,
    ) {
        let mut yval = 0xf0;
        self.ram[oam] = x as u8;
        let xt: u16 = if self.read_u32_ram(ENHANCED_FEATURES0) & 1 != 0 {
            0x48
        } else {
            0
        };
        if x.wrapping_add(0x80) < 0x180 + xt {
            big |= ((x >> 8) as u8) & 1;
            if y.wrapping_add(0x10) < 0x100 {
                yval = y as u8;
            }
        }
        self.ram[oam + 1] = yval;
        self.ram[oam + 2] = charnum;
        self.ram[oam + 3] = flags;
        self.ram[BYTEWISE_EXTENDED_OAM + (oam - OAM_BUF) / 4] = big;
    }

    pub(super) fn ancilla_sfx2_pan(&mut self, k: usize, sfx: u8) {
        self.ram[RAW_SFX_PAN_VALUE] = sfx;
        let out = sfx | self.ancilla_calculate_sfx_pan(k);
        self.ram[SOUND_EFFECT_1] = out;
        self.replay_trace_sfx("ancilla_sfx2_pan", Some(k), sfx, out);
    }

    pub(super) fn ancilla_sfx1_pan(&mut self, k: usize, sfx: u8) {
        self.ram[RAW_SFX_PAN_VALUE] = sfx;
        let out = sfx | self.ancilla_calculate_sfx_pan(k);
        self.ram[SOUND_EFFECT_AMBIENT] = out;
        self.replay_trace_sfx("ancilla_sfx1_pan", Some(k), sfx, out);
    }

    pub(super) fn ancilla_sfx3_pan(&mut self, k: usize, sfx: u8) {
        self.ram[RAW_SFX_PAN_VALUE] = sfx;
        let out = sfx | self.ancilla_calculate_sfx_pan(k);
        self.ram[SOUND_EFFECT_2] = out;
        self.replay_trace_sfx("ancilla_sfx3_pan", Some(k), sfx, out);
    }

    pub(super) fn ancilla_set_xy(&mut self, k: usize, x: u16, y: u16) {
        self.ancilla_set_x(k, x);
        self.ancilla_set_y(k, y);
    }

    fn dash_tremor_twiddle_offset(&mut self, k: usize) -> i32 {
        let j = self.ram[ANCILLA_DIR + k];
        let y = 0u16.wrapping_sub(self.ancilla_get_y(k));
        self.ancilla_set_y(k, y);
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            return y as i32;
        }
        if j == 2 {
            let start = read_le_u16(&self.ram, OW_SCROLL_VARS0_YSTART).wrapping_add(1);
            let end = read_le_u16(&self.ram, OW_SCROLL_VARS0_YEND).wrapping_sub(1);
            let a = y.wrapping_add(read_le_u16(&self.ram, BG2VOFS_COPY2));
            if a <= start || a >= end {
                0
            } else {
                y as i32
            }
        } else {
            let start = read_le_u16(&self.ram, OW_SCROLL_VARS0_XSTART).wrapping_add(1);
            let end = read_le_u16(&self.ram, OW_SCROLL_VARS0_XEND).wrapping_sub(1);
            let a = y.wrapping_add(read_le_u16(&self.ram, BG2HOFS_COPY2));
            if a <= start || a >= end {
                0
            } else {
                y as i32
            }
        }
    }

    fn ancilla_set_x(&mut self, k: usize, x: u16) {
        self.ram[ANCILLA_X_LO + k] = x as u8;
        self.ram[ANCILLA_X_HI + k] = (x >> 8) as u8;
    }

    fn ancilla_set_y(&mut self, k: usize, y: u16) {
        self.ram[ANCILLA_Y_LO + k] = y as u8;
        self.ram[ANCILLA_Y_HI + k] = (y >> 8) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dash_dust_motive_expires_out_of_range_frame() {
        let mut state = ZeldaState::new();
        state.ram[ANCILLA_TYPE] = 0x1e;
        state.ram[ANCILLA_TIMER] = 1;
        state.ram[ANCILLA_ITEM_TO_LINK] = 3;

        state.dash_dust_motive(0);

        assert_eq!(state.ram[ANCILLA_TYPE], 0);
    }
}
