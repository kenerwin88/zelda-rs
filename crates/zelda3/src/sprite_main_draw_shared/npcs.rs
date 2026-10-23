//! Villager, story character, fairy, and minigame NPC draw and behavior tables.

use crate::zelda_rtl::sprite::DrawMultipleData;
// hud.h:9 — kHudItem_Flute.
pub(in crate::zelda_rtl::sprite_main_draw) const HUD_ITEM_FLUTE_DRAW: u8 = 13;
// hud.h:7 — kHudItem_Mushroom.
pub(in crate::zelda_rtl::sprite_main_draw) const HUD_ITEM_MUSHROOM_DRAW: u8 = 5;

pub(in crate::zelda_rtl::sprite_main_draw) const FLUTE_BOY_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -1,
        y: -1,
        char_flags: 0x0abe,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: -1,
        y: -1,
        char_flags: 0x0abe,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0abf,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: -1,
        y: -1,
        char_flags: 0x0abe,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: -1,
        y: -1,
        char_flags: 0x0abe,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0abf,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FLUTE_AARDVARK_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: -16,
        char_flags: 0x06e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x06c8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -16,
        char_flags: 0x06e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x06ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -16,
        char_flags: 0x06e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x06ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -16,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00dc,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const BUG_NET_KID_DRAW_FRAMES: [DrawMultipleData; 18] = [
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x0027,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 6,
        char_flags: 0x040a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 6,
        char_flags: 0x440a,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 14,
        char_flags: 0x840a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 14,
        char_flags: 0xc40a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 6,
        char_flags: 0x040a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 6,
        char_flags: 0x440a,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 14,
        char_flags: 0x840a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 14,
        char_flags: 0xc40a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x002e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x002e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x040a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x440a,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 14,
        char_flags: 0x840a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 14,
        char_flags: 0xc40a,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const BULLY_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x46e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x46e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x46e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x46c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x06e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x06e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x06e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x06c4,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const RUNNING_MAN_RECOIL_X_VELOCITIES: [i8; 2] = [-24, 24];

pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_EYE_BASE_X_OFFSETS: [i8; 2] = [9, -9];

pub(in crate::zelda_rtl::sprite_main_draw) const LADY_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x00e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x00e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x00e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x40e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x40e2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40e6,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const YOUNG_SNITCH_LADY_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0026,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0026,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0024,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0024,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0028,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0028,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x00e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4028,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40e4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x4028,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x40e6,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const HOBO_DRAW_FRAMES: [DrawMultipleData; 12] = [
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00ab,
        ext: 0,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x00a7,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: -11,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00ab,
        ext: 0,
    },
    DrawMultipleData {
        x: 3,
        y: 3,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 3,
        char_flags: 0x00a6,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const RUNNING_MAN_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x002c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x08ee,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x002c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x48ee,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x002a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x08ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x002a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x48ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x002e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x08cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x002e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x08ce,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x402e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x48cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x402e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x48ce,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const ELDER_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40a4,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SHOPKEEPER_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0c00,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0c10,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0c00,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4c10,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FLUTE_BOY_FATHER_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -6,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0088,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FLUTE_BOY_OSTRICH_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x0081,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x00a3,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x0081,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x00a1,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -8,
        char_flags: 0x0081,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 8,
        char_flags: 0x0083,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 8,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -7,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -7,
        char_flags: 0x0081,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 9,
        char_flags: 0x00a3,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 9,
        char_flags: 0x00a4,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const OLD_MOUNTAIN_MAN_DRAW_FRAMES_0: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x00ae,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const OLD_MOUNTAIN_MAN_DRAW_FRAMES_1: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: 0,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: 1,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: 0,
        char_flags: 0x4120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: 1,
        char_flags: 0x4120,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x4122,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const INN_KEEPER_DRAW_FRAMES: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ca,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const ELDER_WIFE_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 5,
        char_flags: 0x0028,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -4,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 5,
        char_flags: 0x4028,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const MIDDLE_AGED_MAN_DRAW_FRAMES: [DrawMultipleData; 2] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x00ea,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ec,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const BLIND_HIDEOUT_GUY_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x000c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x000c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x000c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x000c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x400e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ca,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x400e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x40ca,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SWEEPING_LADY_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 5,
        char_flags: 0x008a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x008e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 4,
        char_flags: 0x008c,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FORTUNE_TELLER_DRAW_FRAMES: [DrawMultipleData; 12] = [
    DrawMultipleData {
        x: 0,
        y: -48,
        char_flags: 0x000c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -32,
        char_flags: 0x002c,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -32,
        char_flags: 0x402c,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -48,
        char_flags: 0x000a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -32,
        char_flags: 0x002a,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -32,
        char_flags: 0x402a,
        ext: 0,
    },
    DrawMultipleData {
        x: -4,
        y: -40,
        char_flags: 0x0066,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -40,
        char_flags: 0x4066,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -40,
        char_flags: 0x0066,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -40,
        char_flags: 0x0068,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: -40,
        char_flags: 0x4068,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -40,
        char_flags: 0x0068,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const MAZE_GAME_GUY_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x4002,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x4002,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0002,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -10,
        char_flags: 0x0002,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0020,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const DRINKING_GUY_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: 8,
        y: 2,
        char_flags: 0x00ae,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0822,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0006,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: 0,
        char_flags: 0x00af,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -9,
        char_flags: 0x0822,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0006,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_DRAW_FRAMES: [DrawMultipleData; 12] = [
    DrawMultipleData {
        x: 1,
        y: -1,
        char_flags: 0x00e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: 7,
        char_flags: 0x00f8,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: -1,
        char_flags: 0x40e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 7,
        y: 7,
        char_flags: 0x40f8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -1,
        char_flags: 0x00e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 7,
        char_flags: 0x00f8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: -1,
        char_flags: 0x40e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x40f8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 7,
        char_flags: 0x00f8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x40e8,
        ext: 0,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x40f8,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const STORY_TELLER_1_DRAW_FRAMES: [DrawMultipleData; 10] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a4a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4a6e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a24,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4a24,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0804,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4804,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a6a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a6c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a0e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a2e,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SMITHY_FROG_DRAW_FRAMES: [DrawMultipleData; 1] = [DrawMultipleData {
    x: 0,
    y: 0,
    char_flags: 0x00c8,
    ext: 2,
}];

pub(in crate::zelda_rtl::sprite_main_draw) const SMITHY_SPARK_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: 0,
        y: 3,
        char_flags: 0x41aa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -1,
        char_flags: 0x41aa,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x0190,
        ext: 0,
    },
    DrawMultipleData {
        x: 12,
        y: 0,
        char_flags: 0x4190,
        ext: 0,
    },
    DrawMultipleData {
        x: -5,
        y: -2,
        char_flags: 0x0191,
        ext: 0,
    },
    DrawMultipleData {
        x: 13,
        y: -2,
        char_flags: 0x0191,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const QUARREL_BROS_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 0,
        y: -12,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -11,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x400a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -12,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -11,
        char_flags: 0x0004,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x400a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -12,
        char_flags: 0x0008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -11,
        char_flags: 0x0008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x400a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -12,
        char_flags: 0x4008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x000a,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -11,
        char_flags: 0x4008,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x400a,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const LUMBERJACKS_DRAW_FRAMES: [DrawMultipleData; 33] = [
    DrawMultipleData {
        x: -23,
        y: 5,
        char_flags: 0x02be,
        ext: 0,
    },
    DrawMultipleData {
        x: -15,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: -7,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 1,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 9,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 17,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 25,
        y: 5,
        char_flags: 0x42be,
        ext: 0,
    },
    DrawMultipleData {
        x: -32,
        y: -8,
        char_flags: 0x40a8,
        ext: 2,
    },
    DrawMultipleData {
        x: -32,
        y: 4,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 30,
        y: -8,
        char_flags: 0x00a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 31,
        y: 4,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 5,
        char_flags: 0x02be,
        ext: 0,
    },
    DrawMultipleData {
        x: -11,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 13,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 21,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 29,
        y: 5,
        char_flags: 0x42be,
        ext: 0,
    },
    DrawMultipleData {
        x: -31,
        y: -8,
        char_flags: 0x40a8,
        ext: 2,
    },
    DrawMultipleData {
        x: -32,
        y: 4,
        char_flags: 0x40a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 31,
        y: -8,
        char_flags: 0x00a8,
        ext: 2,
    },
    DrawMultipleData {
        x: 31,
        y: 4,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 5,
        char_flags: 0x02be,
        ext: 0,
    },
    DrawMultipleData {
        x: -11,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: -3,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 13,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 21,
        y: 5,
        char_flags: 0x02bf,
        ext: 0,
    },
    DrawMultipleData {
        x: 29,
        y: 5,
        char_flags: 0x42be,
        ext: 0,
    },
    DrawMultipleData {
        x: -32,
        y: -8,
        char_flags: 0x400e,
        ext: 2,
    },
    DrawMultipleData {
        x: -32,
        y: 4,
        char_flags: 0x40a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 32,
        y: -8,
        char_flags: 0x000e,
        ext: 2,
    },
    DrawMultipleData {
        x: 31,
        y: 4,
        char_flags: 0x00a6,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const TROUGH_BOY_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0882,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0882,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x4880,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x0880,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0aaa,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const RETREAT_BAT_DRAW_FRAMES: [DrawMultipleData; 18] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x044b,
        ext: 0,
    },
    DrawMultipleData {
        x: 5,
        y: -4,
        char_flags: 0x045b,
        ext: 0,
    },
    DrawMultipleData {
        x: -2,
        y: -4,
        char_flags: 0x0464,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: -4,
        char_flags: 0x0449,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -9,
        char_flags: 0x046c,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -9,
        char_flags: 0x446c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -7,
        char_flags: 0x044c,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -7,
        char_flags: 0x444c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -9,
        char_flags: 0x0444,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -9,
        char_flags: 0x4444,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0462,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4462,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -7,
        char_flags: 0x0460,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -7,
        char_flags: 0x4460,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x044e,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x444e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x046e,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 16,
        char_flags: 0x446e,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const GANON_BAT_DRAW_FRAMES: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0560,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4560,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0562,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4562,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0544,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4544,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const EVIL_BARRIER_DRAW_FRAMES: [DrawMultipleData; 45] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: 3,
        char_flags: 0x00ca,
        ext: 0,
    },
    DrawMultipleData {
        x: -29,
        y: 11,
        char_flags: 0x00da,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 3,
        char_flags: 0x40ca,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40da,
        ext: 0,
    },
    DrawMultipleData {
        x: -24,
        y: -2,
        char_flags: 0x00e6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -2,
        char_flags: 0x00e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -2,
        char_flags: 0x40e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: -2,
        char_flags: 0x40e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: 3,
        char_flags: 0x00cb,
        ext: 0,
    },
    DrawMultipleData {
        x: -29,
        y: 11,
        char_flags: 0x00db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 3,
        char_flags: 0x40cb,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00cc,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: 3,
        char_flags: 0x00cb,
        ext: 0,
    },
    DrawMultipleData {
        x: -29,
        y: 11,
        char_flags: 0x00db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 3,
        char_flags: 0x40cb,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: -24,
        y: -2,
        char_flags: 0x80e6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -2,
        char_flags: 0x80e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -2,
        char_flags: 0xc0e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: -2,
        char_flags: 0xc0e6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: 3,
        char_flags: 0x00ca,
        ext: 0,
    },
    DrawMultipleData {
        x: -29,
        y: 11,
        char_flags: 0x00da,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 3,
        char_flags: 0x40ca,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40da,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e8,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: 3,
        char_flags: 0x00cb,
        ext: 0,
    },
    DrawMultipleData {
        x: -29,
        y: 11,
        char_flags: 0x00db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 3,
        char_flags: 0x40cb,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
    DrawMultipleData {
        x: 37,
        y: 11,
        char_flags: 0x40db,
        ext: 0,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const CHATTY_AGAHNIM_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0b82,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4b82,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x0ba2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4ba2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0b80,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4b80,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x0ba0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4ba0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0b80,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4b82,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x0ba0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4ba2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0b82,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4b80,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x0ba2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4ba0,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const FAERIE_QUEEN_DRAW_FRAMES: [DrawMultipleData; 20] = [
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x40e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x40e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x40e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x00eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 16,
        char_flags: 0x40eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 32,
        char_flags: 0x00ed,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 32,
        char_flags: 0x40ed,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00ef,
        ext: 0,
    },
    DrawMultipleData {
        x: 24,
        y: 0,
        char_flags: 0x40ef,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 8,
        char_flags: 0x00ff,
        ext: 0,
    },
    DrawMultipleData {
        x: 24,
        y: 8,
        char_flags: 0x40ff,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 0,
        char_flags: 0x40e9,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x00eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 16,
        char_flags: 0x40eb,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 32,
        char_flags: 0x00ed,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 32,
        char_flags: 0x40ed,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const CRYSTAL_MAIDEN_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x0120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x0122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x4120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x4122,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: -7,
        char_flags: 0x4120,
        ext: 2,
    },
    DrawMultipleData {
        x: 1,
        y: 3,
        char_flags: 0x4122,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_BAT_CRASH_X_POSITIONS: [u16; 4] = [0x07dc, 0x07f0, 0x0820, 0x0818];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_BAT_CRASH_Y_POSITIONS: [u16; 4] = [0x062e, 0x0636, 0x0630, 0x05e0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_BAT_CRASH_DELAY: [u8; 5] = [4, 3, 4, 6, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const BAT_CRASH_SPAWN_DEBRIS_X_OFFSETS: [i8; 30] = [
    -8, 0, 8, 16, 24, 32, -8, 0, 8, 16, 24, 32, -8, 0, 8, 16, 24, 32, -8, 0, 8, 16, 24, 32, -8, 0,
    8, 16, 24, 32,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BAT_CRASH_SPAWN_DEBRIS_Y_OFFSETS: [i8; 30] = [
    0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x20, 0x20, 0x20, 0x20,
    0x20, 0x20, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BAT_CRASH_SPAWN_DEBRIS_LOCAL_X_VELOCITIES: [i8; 30] = [
    -30, -25, -8, 8, 25, 30, -50, -45, -20, 20, 45, 50, -50, -35, -25, 25, 35, 50, -45, -50, -60,
    60, 50, 45, -30, -35, -40, 40, 35, 30,
];

pub(in crate::zelda_rtl::sprite_main_draw) const BAT_CRASH_SPAWN_DEBRIS_LOCAL_Y_VELOCITIES: [i8; 30] = [
    2, 5, 10, 10, 5, 2, 5, 20, 30, 30, 20, 5, 10, 30, 40, 40, 30, 10, -20, -40, -60, -60, -40, -20,
    -10, -20, -40, -40, -20, -10,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_OLD_SNITCH_LADY_XD: [i8; 2] = [-32, 32];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_OLD_SNITCH_LADY_LOCAL_X_VELOCITIES: [i8; 4] = [0, 0, -9, 9];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_OLD_SNITCH_LADY_LOCAL_Y_VELOCITIES: [i8; 4] = [-9, 9, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_HOBO_SMOKE_OAM_FLAGS: [u8; 4] = [0, 64, 128, 192];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_RUNNING_MAN_LOCAL_X_VELOCITIES: [i8; 4] = [0, 0, -54, 54];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_RUNNING_MAN_LOCAL_Y_VELOCITIES: [i8; 4] = [-54, 54, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_RUNNING_MAN_DIRECTIONS: [i8; 4] = [3, 1, 3, -1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_RUNNING_MAN_A: [u8; 4] = [120, 24, 128, 3];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_RUNNING_MAN_PLAYER_STATE_RECOIL_WALL_LOCAL: u8 = 13;

pub(in crate::zelda_rtl::sprite_main_draw) const OLD_MOUNTAIN_MAN_DRAW_DMA: [u8; 16] = [
    0x20, 0xc0, 0x20, 0xc0, 0, 0xa0, 0, 0xa0, 0x40, 0x80, 0x40, 0x60, 0x40, 0x80, 0x40, 0x60,
];

pub(in crate::zelda_rtl::sprite_main_draw) const WITCH_DRAW_DATA_A: [(i8, i8, u8, u8); 16] = [
    (-3, 8, 0xae, 0x00),
    (-3, 16, 0xbe, 0x00),
    (-2, 8, 0xae, 0x00),
    (-2, 16, 0xbe, 0x00),
    (-1, 8, 0xaf, 0x00),
    (-1, 16, 0xbf, 0x00),
    (0, 9, 0xaf, 0x00),
    (0, 17, 0xbf, 0x00),
    (1, 10, 0xaf, 0x00),
    (1, 18, 0xbf, 0x00),
    (0, 11, 0xaf, 0x00),
    (0, 18, 0xbf, 0x00),
    (-1, 10, 0xae, 0x00),
    (-1, 18, 0xbe, 0x00),
    (-3, 9, 0xae, 0x00),
    (-3, 17, 0xbe, 0x00),
];

pub(in crate::zelda_rtl::sprite_main_draw) const WITCH_DRAW_DATA_B: [(i8, i8, u8, u8); 3] = [
    (0, -4, 0x80, 0x00),
    (-11, 15, 0x86, 0x04),
    (-3, 15, 0x86, 0x44),
];

pub(in crate::zelda_rtl::sprite_main_draw) const WITCH_DRAW_DATA_C: [(i8, i8, u8, u8); 2] =
    [(0, 4, 0x84, 0x00), (0, 4, 0x82, 0x00)];

pub(in crate::zelda_rtl::sprite_main_draw) const FORTUNE_TELLER_LIGHT_OR_DARK_WORLD_PRICES: [u8; 4] = [10, 15, 20, 30];

pub(in crate::zelda_rtl::sprite_main_draw) const FORTUNE_TELLER_PERFORM_PSEUDO_SCIENCE_READINGS: [u8; 16] = [
    0xea, 0xeb, 0xec, 0xed, 0xee, 0xef, 0xf0, 0xf1, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_MOUTH_SECONDARY_GRAPHICS: [u8; 4] = [0, 2, 3, 1];
/// ROM $9D:F9B0: `kTalkingTree_Gfx2` followed by the state-2 code bytes. The
/// ROM computes Y = delay>>1 but then loads the table with X, the sprite
/// slot (`LDA $F9B0,X`), so slots 4..15 read the instruction stream that
/// follows the four-entry table (route host 440910: slot 12 reads $9D).
pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_MOUTH_SECONDARY_GRAPHICS_BY_SLOT: [u8; 16] = [
    0x00, 0x02, 0x03, 0x01, 0xbd, 0xf0, 0x0d, 0x4a, 0xa8, 0xbd, 0xb0, 0xf9, 0x9d, 0xc0, 0x0d, 0xbd,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_MOUTH_MSGS_2: [u16; 2] = [0x0082, 0x007d];

pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_MOUTH_MSGS: [u16; 4] = [0x007e, 0x007f, 0x0080, 0x0081];

pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_MOUTH_SCREENS: [u8; 4] = [0x58, 0x5d, 0x72, 0x6b];

pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_MOUTH_LOCAL_GRAPHICS: [u8; 8] = [1, 2, 3, 1, 3, 1, 2, 3];

pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_MOUTH_DELAY: [u8; 8] = [13, 13, 13, 11, 11, 6, 16, 8];

pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_EYE_X_OFFSETS_1: [i8; 5] = [-2, -1, 0, 1, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const TALKING_TREE_EYE_Y_OFFSETS_1: [i8; 5] = [-1, 0, 0, 0, -1];

pub(in crate::zelda_rtl::sprite_main_draw) const GERUDO_MAN_DRAW_X_OFFSETS: [i8; 18] =
    [4, 4, 4, 4, 4, 4, -8, 8, 8, -8, 8, 8, -16, 0, 16, -16, 0, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const GERUDO_MAN_DRAW_Y_OFFSETS: [i8; 18] =
    [8, 8, 8, 8, 8, 8, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const GERUDO_MAN_DRAW_CHARS: [u8; 18] = [
    0xb8, 0xb8, 0xb8, 0xb8, 0xb8, 0xb8, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa4, 0xa2, 0xa0, 0xa0,
    0xa2, 0xa4,
];

pub(in crate::zelda_rtl::sprite_main_draw) const GERUDO_MAN_DRAW_FL: [u8; 18] = [
    0, 0, 0, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const GERUDO_MAN_DRAW_BIG: [u8; 18] =
    [0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const RECRUIT_DRAW_SOLDIER_CH: [u8; 4] = [0x42, 0x42, 0x40, 0x44];

pub(in crate::zelda_rtl::sprite_main_draw) const RECRUIT_DRAW_SOLDIER_FL: [u8; 4] = [0x40, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const RECRUIT_DRAW_X_OFFSETS: [i16; 8] = [2, 2, -2, -2, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const RECRUIT_DRAW_CHARS: [u8; 8] = [0x8a, 0x8c, 0x8a, 0x8c, 0x86, 0x88, 0x8e, 0xa0];

pub(in crate::zelda_rtl::sprite_main_draw) const RECRUIT_DRAW_FL: [u8; 8] = [0x40, 0x40, 0, 0, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_LUMBERJACKS_MESSAGES: [u16; 4] = [0x012c, 0x012d, 0x012e, 0x012d];

pub(in crate::zelda_rtl::sprite_main_draw) const RETREAT_BAT_DRAW_OFFSETS: [usize; 20] = [
    0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 6, 6, 8, 10, 12, 10, 14, 14, 14, 14,
];

pub(in crate::zelda_rtl::sprite_main_draw) const RETREAT_BAT_DRAW_COUNT: [usize; 20] =
    [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 4, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const FAERIE_QUEEN_DRAW_X_OFFSETS: [u8; 24] = [
    0, 16, 0, 8, 16, 24, 0, 8, 16, 24, 0, 16, 0, 16, 0, 8, 16, 24, 0, 8, 16, 24, 0, 16,
];

pub(in crate::zelda_rtl::sprite_main_draw) const FAERIE_QUEEN_DRAW_Y_OFFSETS: [u8; 24] = [
    0, 0, 16, 16, 16, 16, 24, 24, 24, 24, 32, 32, 0, 0, 16, 16, 16, 16, 24, 24, 24, 24, 32, 32,
];

pub(in crate::zelda_rtl::sprite_main_draw) const FAERIE_QUEEN_DRAW_CHARS: [u8; 24] = [
    0xc7, 0xc7, 0xcf, 0xca, 0xca, 0xcf, 0xdf, 0xda, 0xda, 0xdf, 0xcb, 0xcb, 0xcd, 0xcd, 0xc9, 0xca,
    0xca, 0xc9, 0xd9, 0xda, 0xda, 0xd9, 0xcb, 0xcb,
];

pub(in crate::zelda_rtl::sprite_main_draw) const FAERIE_QUEEN_DRAW_FL: [u8; 24] = [
    0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x40, 0x40, 0, 0x40, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x40,
    0x40, 0, 0x40,
];

pub(in crate::zelda_rtl::sprite_main_draw) const FAERIE_QUEEN_DRAW_BIG: [u8; 24] = [
    2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2,
];

pub(in crate::zelda_rtl::sprite_main_draw) const CRYSTAL_MAIDEN_DRAW_DMA: [u8; 16] = [
    0x20, 0xc0, 0x20, 0xc0, 0, 0xa0, 0, 0xa0, 0x40, 0x80, 0x40, 0x60, 0x40, 0x80, 0x40, 0x60,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_1_F_SICK_KID_LOCAL_GRAPHICS: [i8; 8] = [0, 1, 0, 1, 0, 1, 2, -1];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_1_F_SICK_KID_DELAY: [u8; 7] = [8, 12, 8, 12, 8, 96, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const BAT_CRASH_DRAW_HARDCODED_GARBAGE_OAMS: [(i8, i8, u8, u8); 8] = [
    (104, -105, 0x57, 0x01),
    (120, -105, 0x57, 0x01),
    (-120, -105, 0x57, 0x01),
    (104, -89, 0x57, 0x01),
    (120, -89, 0x57, 0x01),
    (-120, -89, 0x57, 0x01),
    (101, -112, 0x57, 0x01),
    (-117, -112, 0x57, 0x01),
];

pub(in crate::zelda_rtl::sprite_main_draw) const ALTAR_ZELDA_DRAW_BODY_X_OFFS: [u8; 16] =
    [4, 4, 3, 3, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_HOBO_BUM_LOCAL_GRAPHICS: [u8; 7] = [0, 1, 0, 1, 0, 1, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_HOBO_BUM_DELAY: [u8; 7] = [6, 2, 6, 6, 2, 100, 30];
