pub(super) const ANCILLA_Z_SUBPIXEL_PLAYER: usize = 0x02a8;
pub(super) const ANCILLA_TILE_ATTR_PLAYER: usize = 0x03e4;
pub(super) const ANCILLA_ALLOC_ROTATE_PLAYER: usize = 0x03c4;
pub(super) const ANCILLA_S_PLAYER: usize = 0x03a9;
pub(super) const ANCILLA_T_PLAYER: usize = 0x03d5;
pub(super) const ANCILLA_R_PLAYER: usize = 0x03ea;
pub(super) const DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER: usize = 0x0646;
pub(super) const ANCILLA_INTERACTIVE_RESET_FLAG: usize = 0x02f3;
pub(super) const SPRITE_TILETYPE_ANCILLA: usize = 0x0fa5;
pub(super) const CURRENT_AREA_OF_PLAYER_ANCILLA: usize = 0x0700;
pub(super) const BOOMERANG_TEMP_Y: usize = 0x0399;
pub(super) const BOOMERANG_TEMP_X: usize = 0x039b;
// Single-use coordinate scratch for arrow setup; NES_Ver2 aliases are broader shared work RAM.
pub(super) const SCRATCH_0_ANCILLA: usize = 0x0072;
pub(super) const SCRATCH_1_ANCILLA: usize = 0x0074;
pub(super) const INDEX_OF_INTERACTING_TILE_ANCILLA: usize = 0x0076;
pub(super) const SPRITE_IGNORE_PROJECTILE_ANCILLA: usize = 0x0ba0;
pub(super) const REPULSESPARK_FLOOR_STATUS_ANCILLA: usize = 0x0b68;
pub(super) const REPULSESPARK_TIMER_ANCILLA: usize = 0x0fac;
pub(super) const REPULSESPARK_X_LO_ANCILLA: usize = 0x0fad;
pub(super) const REPULSESPARK_Y_LO_ANCILLA: usize = 0x0fae;
pub(super) const REPULSESPARK_ANIM_DELAY_ANCILLA: usize = 0x0faf;
pub(super) const SPRITE_FLAGS_ANCILLA: usize = 0x0b6b;
pub(super) const DAMAGE_TYPE_DETERMINER_ANCILLA: usize = 0x0cf2;
pub(super) const SPRITE_B_ANCILLA: usize = 0x0da0;
pub(super) const SPRITE_C_ANCILLA: usize = 0x0db0;
pub(super) const SPRITE_BUMP_DAMAGE_ANCILLA: usize = 0x0cd2;
pub(super) const SPRITE_HEALTH_ANCILLA: usize = 0x0e50;
pub(super) const SPRITE_HEAD_DIR_ANCILLA: usize = 0x0eb0;
pub(super) const SPRITE_F_ANCILLA: usize = 0x0ea0;
pub(super) const SPRITE_G_ANCILLA: usize = 0x0ed0;
pub(super) const SPRITE_DELAY_AUX2_ANCILLA: usize = 0x0e10;
pub(super) const SPRITE_DELAY_AUX3_ANCILLA: usize = 0x0ee0;
pub(super) const SPRITE_HIT_TIMER_ANCILLA: usize = 0x0ef0;
pub(super) const SPRITE_Y_RECOIL_ANCILLA: usize = 0x0f30;
pub(super) const SPRITE_OAM_FLAGS_ANCILLA: usize = 0x0f50;
pub(super) const GARNISH_ACTIVE_ANCILLA: usize = 0x0fb4;
pub(super) const GARNISH_Y_LO_ANCILLA: usize = 0x1f81e;
pub(super) const GARNISH_X_LO_ANCILLA: usize = 0x1f83c;
pub(super) const GARNISH_Y_HI_ANCILLA: usize = 0x1f85a;
pub(super) const GARNISH_X_HI_ANCILLA: usize = 0x1f878;
pub(super) const GARNISH_SPRITE_ANCILLA: usize = 0x1f8b4;
pub(super) const GARNISH_COUNTDOWN_ANCILLA: usize = 0x1f90e;
pub(super) const DOOR_DEBRIS_DIRECTION: usize = 0x073c;
pub(super) const SWORDBEAM_TEMP_X: usize = 0x1580e;
pub(super) const SWORDBEAM_TEMP_Y: usize = 0x15810;
pub(super) const TAGALONG_Y_LO_ANCILLA: usize = 0x1a00;
pub(super) const TAGALONG_Y_HI_ANCILLA: usize = 0x1a14;
pub(super) const TAGALONG_X_LO_ANCILLA: usize = 0x1a28;
pub(super) const TAGALONG_X_HI_ANCILLA: usize = 0x1a3c;
pub(super) const MILESTONE_ITEM_GFX_SWAP_COUNTDOWN: usize = 0x04c2;
pub(super) const TRIGGER_SPECIAL_ENTRANCE_ANCILLA: usize = 0x04c6;
pub(super) const MAGIC_SPELL_PLAYER_LOCK_FLAG: usize = 0x0325;

pub(super) const BOMBOS_PANNED_SFX_BITS: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
pub(super) const BOMBOS_BLAST_POSITION_TABLE: [u8; 72] = [
    0xb6, 0x5d, 0xa1, 0x30, 0x69, 0xb5, 0xa3, 0x24, 0x96, 0xac, 0x73, 0x5f, 0x92, 0x48, 0x52, 0x81,
    0x39, 0x95, 0x7f, 0x20, 0x88, 0x5d, 0x34, 0x98, 0xbc, 0xd2, 0x51, 0x77, 0xa2, 0x47, 0x94, 0xb2,
    0x34, 0xda, 0x30, 0x62, 0x9f, 0x76, 0x51, 0x46, 0x98, 0x5c, 0x9b, 0x61, 0x58, 0x95, 0x4c, 0xba,
    0x7e, 0xcb, 0x12, 0xd0, 0x70, 0xa6, 0x46, 0xbf, 0x40, 0x50, 0x7e, 0x8c, 0x2d, 0x61, 0xac, 0x88,
    0x20, 0x6a, 0x72, 0x5f, 0xd2, 0x28, 0x52, 0x80,
];

#[derive(Clone, Copy)]
pub(super) struct SignedOffset {
    pub(super) y: i8,
    pub(super) x: i8,
}

#[derive(Clone, Copy)]
pub(super) struct UnsignedOffset {
    pub(super) y: u16,
    pub(super) x: u16,
}

#[derive(Clone, Copy)]
pub(super) struct OamTileAttrs {
    pub(super) char: u8,
    pub(super) flags: u8,
}

#[derive(Clone, Copy)]
pub(super) struct QuakeBoltSprite {
    pub(super) x: i8,
    pub(super) y: i8,
    pub(super) flags: u8,
}

macro_rules! quake_bolt_sprites {
    ($($x:literal, $y:literal, $flags:literal),+ $(,)?) => {
        [$(QuakeBoltSprite { x: $x, y: $y, flags: $flags as u8 },)+]
    };
}

macro_rules! signed_offsets {
    ($($y:literal, $x:literal),+ $(,)?) => {
        [$(SignedOffset { y: $y, x: $x },)+]
    };
}

macro_rules! oam_tile_attrs {
    ($($char:literal, $flags:literal),+ $(,)?) => {
        [$(OamTileAttrs { char: $char, flags: $flags },)+]
    };
}

pub(super) const QUAKE_BOLT_TARGET_PHASES: [u8; 5] = [0x17, 0x16, 0x17, 0x16, 0x10];
pub(super) const QUAKE_GROUND_BOLT_CHARS: [u8; 15] = [
    0x40, 0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 0x60, 0x62, 0x64, 0x66, 0x68, 0x6a, 0x63,
];
pub(super) const QUAKE_INITIAL_BOLT_SPRITES: [QuakeBoltSprite; 151] = quake_bolt_sprites![
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
pub(super) const QUAKE_SPREAD_BOLT_SPRITES: [QuakeBoltSprite; 104] = quake_bolt_sprites![
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
pub(super) const QUAKE_INITIAL_BOLT_FRAME_RANGES: [u8; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 17, 21, 25, 30, 36, 42, 48, 53, 57, 60, 62, 64, 65, 66,
    67, 68, 69, 70, 71, 72, 74, 77, 81, 85, 88, 91, 94, 97, 100, 103, 107, 111, 114, 116, 118, 119,
    120, 121, 122, 123, 124, 125, 126, 128, 130, 132, 134, 137, 141, 145, 149, 151,
];
pub(super) const QUAKE_SPREAD_BOLT_FRAME_RANGES: [u8; 56] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 18, 19, 20, 21, 22, 23, 24, 26, 28, 30, 33, 37, 41,
    45, 46, 47, 48, 49, 50, 51, 52, 53, 55, 57, 59, 62, 66, 70, 72, 73, 74, 75, 76, 78, 80, 82, 84,
    87, 91, 95, 99, 101, 104,
];
pub(super) const RECEIVE_ITEM_MILESTONE_FRAME_TIMERS: [u8; 3] = [9, 5, 5];
pub(super) const RECEIVE_ITEM_MILESTONE_GFX_SOURCES: [u8; 3] = [0x24, 0x25, 0x26];
pub(super) const RECEIVE_ITEM_CRYSTAL_FRAME_SEQUENCE: [u8; 3] = [5, 1, 4];
pub(super) const RECEIVE_ITEM_MESSAGES: [i16; 76] = [
    -1, 0x70, 0x77, 0x52, -1, 0x78, 0x78, 0x62, 0x61, 0x66, 0x69, 0x53, 0x52, 0x56, -1, 0x64, 0x63,
    0x65, 0x51, 0x54, 0x67, 0x68, 0x6b, 0x77, 0x79, 0x55, 0x6e, 0x58, 0x6d, 0x5d, 0x57, 0x5e, -1,
    0x74, 0x75, 0x76, -1, 0x5f, 0x158, -1, 0x6a, 0x5c, 0x8f, 0x71, 0x72, 0x73, 0x71, 0x72, 0x73,
    0x6a, 0x6c, 0x60, -1, -1, -1, 0x59, 0x84, 0x5a, -1, -1, -1, -1, -1, 0x159, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 0xdb, 0x67, 0x7c,
];
pub(super) const RECEIVE_ITEM_SPECIAL_MESSAGES: [i16; 2] = [0x5b, 0x83];
pub(super) const RECEIVE_ITEM_HEART_PIECE_MESSAGES: [i16; 4] = [-1, 0x155, 0x156, 0x157];
pub(super) const BOMB_PHASE_TIMERS: [u8; 11] = [0xa0, 6, 4, 4, 4, 4, 4, 6, 6, 6, 6];
pub(super) const BOMB_DRAW_FRAME_STARTS: [u8; 12] = [0, 1, 2, 3, 2, 3, 4, 5, 6, 7, 8, 9];
pub(super) const BOMB_DRAW_FRAME_COUNTS: [u8; 11] = [1, 4, 4, 4, 4, 4, 5, 4, 6, 6, 6];

pub(super) const ANCILLA_DRAW_SPRITE_COUNTS: [u8; 68] = [
    0, 8, 0x0c, 0x10, 0x10, 4, 0x10, 0x18, 8, 8, 8, 0, 0x14, 0, 0x10, 0x28, 0x18, 0x10, 0x10, 0x10,
    0x10, 0x0c, 8, 8, 0x50, 0, 0x10, 8, 0x40, 0, 0x0c, 0x24, 0x10, 0x0c, 8, 0x10, 0x10, 4, 0x0c,
    0x1c, 0, 0x10, 0x14, 0x14, 0x10, 8, 0x20, 0x10, 0x10, 0x10, 4, 0, 0x80, 0x10, 4, 0x30, 0x14,
    0x10, 0, 0x10, 0, 0, 8, 0, 0x10, 8, 0x78, 0x80,
];

pub(super) const RECEIVE_ITEM_GRAPHICS: [u8; 76] = [
    6, 0x18, 0x18, 0x18, 0x2d, 0x20, 0x2e, 9, 9, 0x0a, 8, 5, 0x10, 0x0b, 0x2c, 0x1b, 0x1a, 0x1c,
    0x14, 0x19, 0x0c, 7, 0x1d, 0x2f, 7, 0x15, 0x12, 0x0d, 0x0d, 0x0e, 0x11, 0x17, 0x28, 0x27, 4, 4,
    0x0f, 0x16, 3, 0x13, 1, 0x1e, 0x10, 0, 0, 0, 0, 0, 0, 0x30, 0x22, 0x21, 0x24, 0x24, 0x24, 0x23,
    0x23, 0x23, 0x29, 0x2a, 0x2c, 0x2b, 3, 3, 0x34, 0x35, 0x31, 0x33, 2, 0x32, 0x36, 0x37, 0x2c, 6,
    0x0c, 0x38,
];
pub(super) const RECEIVE_ITEM_OAM_EXT_SIZES: [u8; 76] = [
    0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
];
pub(super) const WISH_POND_ITEM_OAM_FLAGS: [u8; 76] = [
    5, 0xff, 5, 5, 5, 5, 5, 1, 2, 1, 1, 1, 2, 2, 2, 4, 4, 4, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1, 4, 4, 2,
    1, 6, 1, 2, 1, 2, 2, 1, 2, 2, 4, 1, 1, 4, 2, 1, 4, 2, 2, 4, 4, 4, 2, 1, 4, 1, 2, 2, 1, 2, 2, 1,
    1, 4, 4, 1, 2, 2, 4, 4, 4, 2, 5, 2, 1,
];
pub(super) const TRAVEL_BIRD_DMA_TILE_OFFSETS: [u8; 4] = [0, 0x20, 0x40, 0xe0];
pub(super) const TRAVEL_BIRD_DRAW_X_OFFSETS: [i8; 3] = [0, -9, -9];
pub(super) const TRAVEL_BIRD_DRAW_Y_OFFSETS: [i8; 3] = [0, 12, 20];
pub(super) const TRAVEL_BIRD_DRAW_CHARS: [u8; 3] = [0x0e, 0, 2];
pub(super) const TRAVEL_BIRD_DRAW_FLAGS: [u8; 3] = [0x22, 0x2e, 0x2e];

pub(super) const ANCILLA_OVERWORLD_AREA_BASE_X: [u16; 64] = [
    0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xa00, 0xe00,
    0, 0x200, 0x400, 0x600, 0x800, 0xa00, 0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00,
    0xc00, 0, 0, 0x400, 0x600, 0x600, 0xa00, 0xc00, 0xc00, 0, 0x200, 0x400, 0x600, 0x800, 0xa00,
    0xc00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00, 0xa00, 0xe00, 0, 0, 0x400, 0x600, 0x800, 0xa00,
    0xa00, 0xe00,
];

pub(super) const ANCILLA_OVERWORLD_AREA_BASE_Y: [u16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x200, 0, 0, 0, 0, 0x200, 0x400, 0x400, 0x400, 0x400, 0x400,
    0x400, 0x400, 0x400, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600, 0x600,
    0x800, 0x600, 0x600, 0x800, 0x600, 0x600, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00, 0xa00,
    0xa00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xc00, 0xe00, 0xe00,
    0xe00, 0xc00, 0xc00, 0xe00,
];

pub(super) const MAGIC_POWDER_FRAME_TIMERS: [u8; 40] = [
    13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 10, 11, 12, 0, 1, 2, 3, 4, 5, 6, 16, 17, 18, 0, 1, 2, 3, 4, 5,
    6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6,
];

#[rustfmt::skip]
pub(super) const ANCILLA_TILE_COLLISION_ATTRS: [u8; 256] = [
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
pub(super) const ANCILLA_TILE_COLLISION_ATTRS_LAYER0: [u8; 256] = [
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

pub(super) const SLOPED_TILE_HEIGHTS: [u8; 32] = [
    7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0,
];

pub(super) const FIRE_ROD_SPARK_X_VELOCITIES: [i8; 12] =
    [0, 0, -40, 40, 0, 0, -48, 48, 0, 0, -64, 64];
pub(super) const FIRE_ROD_SPARK_Y_VELOCITIES: [i8; 12] =
    [-40, 40, 0, 0, -48, 48, 0, 0, -64, 64, 0, 0];

pub(super) struct CheckPlayerCollOut {
    pub(super) r4: u16,
    pub(super) r6: u16,
    pub(super) r8: u16,
    pub(super) r10: u16,
}

pub(super) struct AncillaOamInfo {
    pub(super) x: u8,
    pub(super) y: u8,
    pub(super) flags: u8,
}

// ---------------------------------------------------------------------------
// Promoted ancilla method-local tables. Names retain the owning helper so
// generic C table names stay readable at callsites.
// ---------------------------------------------------------------------------

pub(super) const ANCILLA_WEAPON_TINK_REPULSE_SPARK_FLAGS: [u8; 4] = [0x22, 0x12, 0x22, 0x22];

pub(super) const ANCILLA_WEAPON_TINK_REPULSE_SPARK_CHAR: [u8; 3] = [0x93, 0x82, 0x81];

pub(super) const ANCILLA_ADD_HIT_STARS_SHOVEL_HIT_STARS_OFFSET: [SignedOffset; 6] = [
    SignedOffset { y: 21, x: -11 },
    SignedOffset { y: 21, x: 11 },
    SignedOffset { y: 3, x: -6 },
    SignedOffset { y: 21, x: 5 },
    SignedOffset { y: 16, x: -14 },
    SignedOffset { y: 16, x: 14 },
];

pub(super) const ANCILLA_ADD_HIT_STARS_SHOVEL_HIT_STARS_X2: [i8; 6] = [-3, 19, 2, 13, -6, 22];

pub(super) const ANCILLA_ADD_FIRE_ROD_SHOT_FIRE_ROD_X: [i8; 4] = [0, 0, -8, 16];

pub(super) const ANCILLA_ADD_FIRE_ROD_SHOT_FIRE_ROD_Y: [i8; 4] = [-8, 16, 3, 3];

pub(super) const ANCILLA_ADD_FIRE_ROD_SHOT_FIRE_ROD_XVEL: [i8; 4] = [0, 0, -64, 64];

pub(super) const ANCILLA_ADD_FIRE_ROD_SHOT_FIRE_ROD_YVEL: [i8; 4] = [-64, 64, 0, 0];

pub(super) const ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_TYPE: [u8; 7] =
    [0x10, 0x37, 0x39, 0x38, 0x26, 0x0f, 0x20];

pub(super) const ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_G: [u8; 7] = [0x40, 0, 0, 0, 0, 0xff, 0];

pub(super) const ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_X: [u16; 7] =
    [0x78, 0x78, 0x78, 0x78, 0x78, 0x80, 0x78];

pub(super) const ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_Y: [u16; 7] =
    [0x48, 0x78, 0x78, 0x78, 0x78, 0x68, 0x78];

pub(super) const ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_Z: [u8; 7] =
    [0x60, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];

pub(super) const ADD_SWORD_BEAM_SWORD_BEAM_X: [i8; 4] = [-8, -10, -22, 4];

pub(super) const ADD_SWORD_BEAM_SWORD_BEAM_Y: [i8; 4] = [-24, 8, -6, -6];

pub(super) const ADD_SWORD_BEAM_SWORD_BEAM_S: [i8; 4] = [-8, -8, -8, 8];

pub(super) const ADD_SWORD_BEAM_SWORD_BEAM_TRAILING_ANGLES: [u8; 16] = [
    0x21, 0x1d, 0x19, 0x15, 3, 0x3e, 0x3a, 0x36, 0x12, 0x0e, 0x0a, 6, 0x31, 0x2d, 0x29, 0x25,
];

pub(super) const ADD_SWORD_BEAM_SWORD_BEAM_YVEL: [i8; 4] = [-64, 64, 0, 0];

pub(super) const ADD_SWORD_BEAM_SWORD_BEAM_XVEL: [i8; 4] = [0, 0, -64, 64];

pub(super) const ANCILLA_SPAWN_SWORD_CHARGE_SPARKLE_SWORD_CHARGE_SPARKLE_A: [u8; 4] = [0, 0, 7, 7];

pub(super) const ANCILLA_SPAWN_SWORD_CHARGE_SPARKLE_SWORD_CHARGE_SPARKLE_B: [u8; 4] =
    [0x70, 0x70, 0, 0];

pub(super) const ANCILLA_SPAWN_SWORD_CHARGE_SPARKLE_SWORD_CHARGE_SPARKLE_X: [u8; 4] = [0, 3, 4, 5];

pub(super) const ANCILLA_SPAWN_SWORD_CHARGE_SPARKLE_SWORD_CHARGE_SPARKLE_Y: [u8; 4] = [5, 12, 8, 8];

pub(super) const ADD_DASHING_DUST_EX_ADD_DASHING_DUST_X: [i8; 4] = [4, 4, 6, 0];

pub(super) const ADD_DASHING_DUST_EX_ADD_DASHING_DUST_Y: [i8; 4] = [20, 4, 16, 16];

pub(super) const ANCILLA_ADD_BLAST_WALL_FIREBALL_BLAST_WALL_FIREBALL_VELOCITY: [SignedOffset; 16] = [
    SignedOffset { y: -64, x: 0 },
    SignedOffset { y: -22, x: 42 },
    SignedOffset { y: -38, x: 38 },
    SignedOffset { y: -42, x: 22 },
    SignedOffset { y: 0, x: 64 },
    SignedOffset { y: 22, x: 42 },
    SignedOffset { y: 38, x: 38 },
    SignedOffset { y: 42, x: 22 },
    SignedOffset { y: 64, x: 0 },
    SignedOffset { y: 22, x: -42 },
    SignedOffset { y: 38, x: -38 },
    SignedOffset { y: 42, x: -22 },
    SignedOffset { y: 0, x: -64 },
    SignedOffset { y: -22, x: -42 },
    SignedOffset { y: -38, x: -38 },
    SignedOffset { y: -42, x: -22 },
];

pub(super) const ANCILLA_ADD_ARROW_SHOOT_BOW_X: [i8; 4] = [4, 4, 0, 4];

pub(super) const ANCILLA_ADD_ARROW_SHOOT_BOW_Y: [i8; 4] = [-4, 3, 4, 4];

pub(super) const ANCILLA_ADD_ARROW_SHOOT_BOW_XVEL: [i8; 4] = [0, 0, -48, 48];

pub(super) const ANCILLA_ADD_ARROW_SHOOT_BOW_YVEL: [i8; 4] = [-48, 48, 0, 0];

pub(super) const BOMB_CHECK_SPRITE_AND_PLAYER_DAMAGE_BOMB_DMG_SPEED: [u8; 16] = [
    32, 32, 32, 32, 32, 32, 28, 28, 28, 28, 28, 28, 24, 24, 24, 24,
];

pub(super) const BOMB_CHECK_SPRITE_AND_PLAYER_DAMAGE_BOMB_DMG_ZVEL: [u8; 16] =
    [16, 16, 16, 16, 16, 16, 12, 12, 12, 12, 8, 8, 8, 8, 8, 8];

pub(super) const BOMB_CHECK_SPRITE_AND_PLAYER_DAMAGE_BOMB_DMG_DELAY: [u8; 16] = [
    32, 32, 32, 32, 32, 32, 24, 24, 24, 24, 24, 24, 16, 16, 16, 16,
];

pub(super) const BOMB_CHECK_SPRITE_AND_PLAYER_DAMAGE_BOMB_DMG_TO_LINK: [u8; 3] = [8, 4, 2];

pub(super) const ANCILLA05_BOOMERANG_BOOMERANG_X0: [i8; 8] = [0, 0, -8, 8, 8, 8, -8, -8];

pub(super) const ANCILLA05_BOOMERANG_BOOMERANG_Y0: [i8; 8] = [-16, 6, 0, 0, -8, 8, -8, 8];

pub(super) const ANCILLA05_BOOMERANG_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const ANCILLA01_SOMARIA_BULLET_SOMARIAN_BLAST_MASK: [u8; 6] = [7, 3, 1, 0, 0, 0];

pub(super) const BOOMERANG_DRAW_BOOMERANG_FLAGS: [u8; 8] =
    [0xa4, 0xe4, 0x64, 0x24, 0xa2, 0xe2, 0x62, 0x22];

pub(super) const BOOMERANG_DRAW_BOOMERANG_DRAW_OFFSET: [SignedOffset; 4] = [
    SignedOffset { y: 2, x: -2 },
    SignedOffset { y: 2, x: 2 },
    SignedOffset { y: -2, x: 2 },
    SignedOffset { y: -2, x: -2 },
];

pub(super) const BOOMERANG_DRAW_BOOMERANG_DRAW_OAM_IDX: [u16; 2] = [0x180, 0xd0];

pub(super) const BOOMERANG_DRAW_BOOMERANG_FRAME_RESET_BY_TYPE: [u8; 2] = [3, 2];

pub(super) const ANCILLA1_E_DASH_DUST_DASH_DUST_DRAW_X1: [i8; 4] = [0, 0, 4, -4];

pub(super) const ANCILLA1_E_DASH_DUST_DASH_DUST_DRAW_X: [i16; 30] = [
    10, 5, -1, 0, 10, 5, 0, 5, -1, 0, -1, -1, 9, -1, -1, 10, 5, -1, 0, 10, 5, 0, 5, -1, 0, -1, -1,
    9, -1, -1,
];

pub(super) const ANCILLA1_E_DASH_DUST_DASH_DUST_DRAW_Y: [i16; 30] = [
    -2, 0, -1, -3, -2, 0, -3, 0, -1, -3, -1, -1, -2, -1, -1, -2, 0, -1, -3, -2, 0, -3, 0, -1, -3,
    -1, -1, -2, -1, -1,
];

pub(super) const ANCILLA1_E_DASH_DUST_DASH_DUST_DRAW_CHAR: [u8; 30] = [
    0xcf, 0xa9, 0xff, 0xa9, 0xdf, 0xcf, 0xcf, 0xdf, 0xff, 0xdf, 0xff, 0xff, 0xa9, 0xff, 0xff, 0xcf,
    0xcf, 0xff, 0xcf, 0xdf, 0xcf, 0xcf, 0xdf, 0xff, 0xdf, 0xff, 0xff, 0xcf, 0xff, 0xff,
];

pub(super) const DASH_DUST_MOTIVE_MOTIVE_DASH_DUST_DRAW_CHAR: [u8; 3] = [0xa9, 0xcf, 0xdf];

pub(super) const WALL_HIT_DRAW_WALL_HIT_X: [i8; 32] = [
    -4, 0, 0, 0, -4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -4, 0, 0, 0, -4, 0, 0, 0,
    -8, 0, 0, 0,
];

pub(super) const WALL_HIT_DRAW_WALL_HIT_Y: [i8; 32] = [
    -4, 0, 0, 0, -4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -4, 0, 0, 0, -4, 0, 0, 0,
    -8, 0, 0, 0,
];

pub(super) const WALL_HIT_DRAW_WALL_HIT_CHAR: [u8; 32] = [
    0x80, 0, 0, 0, 0x92, 0, 0, 0, 0x81, 0x81, 0x81, 0x81, 0x82, 0x82, 0x82, 0x82, 0x93, 0x93, 0x93,
    0x93, 0x92, 0, 0, 0, 0xb9, 0, 0, 0, 0x90, 0x90, 0, 0,
];

pub(super) const WALL_HIT_DRAW_WALL_HIT_FLAGS: [u8; 32] = [
    0x32, 0, 0, 0, 0x32, 0, 0, 0, 0x32, 0x72, 0xb2, 0xf2, 0x32, 0x72, 0xb2, 0xf2, 0x32, 0x72, 0xb2,
    0xf2, 0x32, 0, 0, 0, 0x72, 0, 0, 0, 0x32, 0xf2, 0, 0,
];

pub(super) const DOOR_DEBRIS_DRAW_DOOR_DEBRIS_OFFSET: [UnsignedOffset; 32] = [
    UnsignedOffset { y: 4, x: 7 },
    UnsignedOffset { y: 3, x: 17 },
    UnsignedOffset { y: 8, x: 8 },
    UnsignedOffset { y: 7, x: 17 },
    UnsignedOffset { y: 11, x: 7 },
    UnsignedOffset { y: 10, x: 16 },
    UnsignedOffset { y: 16, x: 7 },
    UnsignedOffset { y: 17, x: 17 },
    UnsignedOffset { y: 20, x: 7 },
    UnsignedOffset { y: 21, x: 17 },
    UnsignedOffset { y: 16, x: 8 },
    UnsignedOffset { y: 17, x: 17 },
    UnsignedOffset { y: 13, x: 7 },
    UnsignedOffset { y: 14, x: 16 },
    UnsignedOffset { y: 8, x: 7 },
    UnsignedOffset { y: 7, x: 17 },
    UnsignedOffset { y: 7, x: 4 },
    UnsignedOffset { y: 17, x: 3 },
    UnsignedOffset { y: 8, x: 8 },
    UnsignedOffset { y: 17, x: 7 },
    UnsignedOffset { y: 7, x: 11 },
    UnsignedOffset { y: 16, x: 10 },
    UnsignedOffset { y: 7, x: 16 },
    UnsignedOffset { y: 17, x: 17 },
    UnsignedOffset { y: 7, x: 20 },
    UnsignedOffset { y: 17, x: 21 },
    UnsignedOffset { y: 8, x: 16 },
    UnsignedOffset { y: 17, x: 17 },
    UnsignedOffset { y: 7, x: 13 },
    UnsignedOffset { y: 16, x: 14 },
    UnsignedOffset { y: 7, x: 8 },
    UnsignedOffset { y: 17, x: 7 },
];

pub(super) const DOOR_DEBRIS_DRAW_DOOR_DEBRIS_TILE: [OamTileAttrs; 32] = [
    OamTileAttrs {
        char: 0x5e,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0xe0,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0xa0,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0xe0,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0xe0,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0xa0,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0xe0,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0x20,
    },
    OamTileAttrs {
        char: 0x5e,
        flags: 0xe0,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x60,
    },
    OamTileAttrs {
        char: 0x4f,
        flags: 0x60,
    },
];

pub(super) const ANCILLA_ADD_BOOMERANG_WALL_CLINK_BOOMERANG_WALL_HIT_X: [i8; 8] =
    [8, 8, 0, 10, 12, 8, 4, 0];

pub(super) const ANCILLA_ADD_BOOMERANG_WALL_CLINK_BOOMERANG_WALL_HIT_Y: [i8; 8] =
    [0, 8, 8, 8, 4, 8, 12, 8];

pub(super) const ANCILLA_ADD_BOOMERANG_WALL_CLINK_BOOMERANG_WALL_HIT_OFFSET_INDEX: [u8; 16] =
    [0, 6, 4, 0, 2, 10, 12, 0, 0, 8, 14, 0, 0, 0, 0, 0];

pub(super) const ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_X: [i8; 16] =
    [3, 1, 0, 0, 13, 16, 12, 12, 24, 7, -4, -10, -8, 9, 22, 26];

pub(super) const ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_Y: [i8; 16] =
    [5, 0, -3, -6, -8, -3, 12, 28, 5, 0, 8, 16, 5, 0, 8, 16];

pub(super) const ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_DRAW_X: [i8; 16] =
    [-4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0];

pub(super) const ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_DRAW_Y: [i8; 16] =
    [-4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0];

pub(super) const ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_DRAW_CHAR: [u8; 16] = [
    0x92, 0xff, 0xff, 0xff, 0x8c, 0x8c, 0x8c, 0x8c, 0xd6, 0xd6, 0xd6, 0xd6, 0x93, 0x93, 0x93, 0x93,
];

pub(super) const ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_DRAW_FLAGS: [u8; 16] = [
    0x22, 0xff, 0xff, 0xff, 0x22, 0x62, 0xa2, 0xe2, 0x24, 0x64, 0xa4, 0xe4, 0x22, 0x62, 0xa2, 0xe2,
];

pub(super) const BYRNA_WINDUP_SPARK_TRANSMUTE_TO_NORMAL_CANE_SPARK_TRAILING_ANGLES: [u8; 16] = [
    0x34, 0x33, 0x32, 0x31, 0x16, 0x15, 0x14, 0x13, 0x2a, 0x29, 0x28, 0x27, 0x10, 0x0f, 0x0e, 0x0d,
];

pub(super) const ANCILLA31_BYRNA_SPARK_CANE_SPARK_MAGIC: [u8; 3] = [4, 2, 1];

pub(super) const ANCILLA31_BYRNA_SPARK_CANE_SPARK_CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];

pub(super) const ANCILLA_ADD_MAGIC_POWDER_MAGIC_POWER_X: [i8; 4] = [-2, -2, -12, 12];

pub(super) const ANCILLA_ADD_MAGIC_POWDER_MAGIC_POWER_Y: [i8; 4] = [0, 20, 16, 16];

pub(super) const ANCILLA_ADD_MAGIC_POWDER_MAGIC_POWER_X1: [i8; 4] = [10, 10, -8, 28];

pub(super) const ANCILLA_ADD_MAGIC_POWDER_MAGIC_POWER_Y1: [i8; 4] = [1, 40, 22, 22];

pub(super) const ANCILLA_ADD_WALL_TAP_SPARK_WALL_TAP_SPARK_X: [i8; 4] = [11, 10, -12, 29];

pub(super) const ANCILLA_ADD_WALL_TAP_SPARK_WALL_TAP_SPARK_Y: [i8; 4] = [-4, 32, 17, 17];

pub(super) const ANCILLA_ADD_LAMP_FLAME_LAMP_FLAME_X: [i8; 4] = [0, 0, -20, 18];

pub(super) const ANCILLA_ADD_LAMP_FLAME_LAMP_FLAME_Y: [i8; 4] = [-16, 24, 4, 4];

pub(super) const ANCILLA_ADD_DASH_TREMOR_ADD_DASH_TREMOR_DIR: [u8; 4] = [2, 2, 0, 0];

pub(super) const ANCILLA_ADD_DASH_TREMOR_DASH_TREMOR_COORD_LIMITS: [u8; 2] = [0x80, 0x78];

pub(super) const ANCILLA_ADD_HOOKSHOT_WALL_CLINK_HOOKSHOT_WALL_HIT_X: [i8; 8] =
    [8, 8, 0, 10, 12, 8, 4, 0];

pub(super) const ANCILLA_ADD_HOOKSHOT_WALL_CLINK_HOOKSHOT_WALL_HIT_Y: [i8; 8] =
    [0, 8, 8, 8, 4, 8, 12, 8];

pub(super) const ANCILLA_ADD_BOMBOS_SPELL_BOMBOS_Y_DELTA: [i16; 4] = [16, 24, -128, -16];

pub(super) const ANCILLA_ADD_BOMBOS_SPELL_BOMBOS_X_DELTA: [i16; 4] = [-16, -128, 0, 128];

pub(super) const ANCILLA_ADD_BLAST_WALL_BLAST_WALL_FRAGMENT_X_OFFSET: [i8; 4] = [-16, 16, 0, 0];

pub(super) const ANCILLA_ADD_BLAST_WALL_BLAST_WALL_FRAGMENT_Y_OFFSET: [i8; 4] = [0, 0, -16, 16];

pub(super) const ANCILLA_ADD_BLAST_WALL_BLAST_WALL_FRAGMENT_OFFSET: [SignedOffset; 8] =
    signed_offsets![-8, 0, -8, 16, 16, 0, 16, 16, 0, -8, 16, -8, 0, 16, 16, 16,];

pub(super) const ADD_HAPPINESS_POND_RUPEES_HAPPINESS_POND_START: [i8; 4] = [0, 4, 4, 9];

pub(super) const ADD_HAPPINESS_POND_RUPEES_HAPPINESS_POND_END: [i8; 4] = [-1, 0, -1, -1];

pub(super) const ADD_HAPPINESS_POND_RUPEES_HAPPINESS_POND_XVEL: [i8; 10] =
    [0, -12, -6, 6, 12, -9, -5, 0, 5, 9];

pub(super) const ADD_HAPPINESS_POND_RUPEES_HAPPINESS_POND_YVEL: [i8; 10] =
    [-40, -40, -40, -40, -40, -32, -32, -32, -32, -32];

pub(super) const ADD_HAPPINESS_POND_RUPEES_HAPPINESS_POND_ZVEL: [i8; 10] =
    [20, 20, 20, 20, 20, 16, 16, 16, 16, 16];

pub(super) const ANCILLA_ADD_BOMB_BOMB_PLACE_X0: [i8; 4] = [8, 8, 0, 16];

pub(super) const ANCILLA_ADD_BOMB_BOMB_PLACE_Y0: [i8; 4] = [0, 24, 12, 12];

pub(super) const ANCILLA_ADD_BOMB_BOMB_PLACE_X1: [i8; 4] = [8, 8, -6, 22];

pub(super) const ANCILLA_ADD_BOMB_BOMB_PLACE_Y1: [i8; 4] = [4, 28, 12, 12];

pub(super) const ANCILLA_ADD_BOOMERANG_BOOMERANG_SPEED_BY_TYPE: [u8; 4] = [0x20, 0x18, 0x30, 0x28];

pub(super) const ANCILLA_ADD_BOOMERANG_BOOMERANG_INITIAL_STEP_BY_TYPE: [u8; 2] = [0x20, 0x60];

pub(super) const ANCILLA_ADD_BOOMERANG_BOOMERANG_FRAME_RESET_BY_TYPE: [u8; 2] = [3, 2];

pub(super) const ANCILLA_ADD_BOOMERANG_BOOMERANG_DIRECTION_BITS: [u8; 4] = [8, 4, 2, 1];

pub(super) const ANCILLA_ADD_BOOMERANG_BOOMERANG_INPUT_MASKS: [u8; 8] = [8, 4, 2, 1, 9, 5, 10, 6];

pub(super) const ANCILLA_ADD_BOOMERANG_BOOMERANG_INITIAL_SPIN_BY_INPUT: [u8; 8] =
    [2, 3, 3, 2, 2, 3, 3, 3];

pub(super) const ANCILLA_ADD_BOOMERANG_BOOMERANG_INITIAL_Y_OFFSET: [i8; 8] =
    [-10, -8, -9, -9, -10, -8, -9, -9];

pub(super) const ANCILLA_ADD_BOOMERANG_BOOMERANG_INITIAL_X_OFFSET: [i8; 8] =
    [-10, 11, 8, -8, -10, 11, 8, -8];

pub(super) const ANCILLA_ADD_BOOMERANG_BOOMERANG_CHARGED_Y_OFFSET: [i8; 8] =
    [-16, 6, 0, 0, -8, 8, -8, 8];

pub(super) const ANCILLA_ADD_BOOMERANG_BOOMERANG_CHARGED_X_OFFSET: [i8; 8] =
    [0, 0, -8, 8, 8, 8, -8, -8];

pub(super) const ANCILLA_ADD_TOSSED_POND_ITEM_WISH_POND_ITEM_X: [u8; 76] = [
    4, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 11, 0, 0, 0, 2, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 4, 4, 0, 4, 0, 0, 0, 4, 0, 0,
];

pub(super) const ANCILLA_ADD_TOSSED_POND_ITEM_WISH_POND_ITEM_Y: [i8; 76] = [
    -13, -13, -13, -13, -13, -12, -12, -13, -13, -12, -12, -12, -10, -12, -12, -12, -12, -12, -12,
    -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -13, -12, -12,
    -12, -12, -12, -12, -10, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12,
    -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -12, -13, -12, -12,
];

pub(super) const ANCILLA_ADD_EXPLODING_WEATHER_VANE_WEATHERVANE_DEBRIS_X_VELOCITY: [i8; 12] =
    [8, 10, 9, 4, 11, 12, -10, -8, 4, -6, -10, -4];

pub(super) const ANCILLA_ADD_EXPLODING_WEATHER_VANE_WEATHERVANE_DEBRIS_Y_VELOCITY: [i8; 12] =
    [20, 22, 20, 20, 22, 20, 20, 22, 20, 22, 20, 20];

pub(super) const ANCILLA_ADD_EXPLODING_WEATHER_VANE_WEATHERVANE_DEBRIS_START_Y: [u8; 12] = [
    0xb0, 0xa3, 0xa0, 0xa2, 0xa0, 0xa8, 0xa0, 0xa0, 0xa8, 0xa1, 0xb0, 0xa0,
];

pub(super) const ANCILLA_ADD_EXPLODING_WEATHER_VANE_WEATHERVANE_DEBRIS_START_X: [u8; 12] =
    [0, 2, 4, 6, 3, 8, 14, 8, 12, 7, 10, 8];

pub(super) const ANCILLA_ADD_EXPLODING_WEATHER_VANE_WEATHERVANE_DEBRIS_CHAR: [u8; 12] =
    [48, 18, 32, 20, 22, 24, 32, 20, 24, 22, 20, 32];

pub(super) const ANCILLA_PREP_OAM_COORD_TAGALONG_LAYER_BITS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];

pub(super) const ANCILLA09_ARROW_ARROW_Y: [i8; 4] = [-4, 2, 0, 0];

pub(super) const ANCILLA09_ARROW_ARROW_X: [i8; 4] = [0, 0, -4, 4];

pub(super) const ANCILLA_SWORD_BEAM_SWORD_BEAM_YVEL2: [i8; 4] = [0, 0, -6, -6];

pub(super) const ANCILLA_SWORD_BEAM_SWORD_BEAM_XVEL2: [i8; 4] = [-8, -10, 0, 0];

pub(super) const ANCILLA_SWORD_BEAM_SWORD_BEAM_CHAR: [u8; 4] = [0xd7, 0xb7, 0x80, 0x83];

pub(super) const ANCILLA_SWORD_BEAM_SWORD_BEAM_CHAR2: [u8; 3] = [0xb7, 0x80, 0x83];

pub(super) const ANCILLA0_D_SPIN_ATTACK_FULL_CHARGE_SPARK_SWORD_FULL_CHARGE_SPARK_Y: [i8; 4] =
    [-8, 27, 12, 12];

pub(super) const ANCILLA0_D_SPIN_ATTACK_FULL_CHARGE_SPARK_SWORD_FULL_CHARGE_SPARK_X: [i8; 4] =
    [4, 4, -13, 20];

pub(super) const ANCILLA0_D_SPIN_ATTACK_FULL_CHARGE_SPARK_SWORD_FULL_CHARGE_SPARK_FLAGS: [u8; 4] =
    [0x20, 0x10, 0x30, 0x20];

pub(super) const ANCILLA20_BLANKET_BEDSPREAD_CHAR: [u8; 8] =
    [0x0a, 0x0a, 0x0a, 0x0a, 0x0c, 0x0c, 0x0a, 0x0a];

pub(super) const ANCILLA20_BLANKET_BEDSPREAD_FLAGS: [u8; 8] =
    [0, 0x60, 0xa0, 0xe0, 0, 0x60, 0xa0, 0xe0];

pub(super) const ANCILLA21_SNORE_BEDSPREAD_DMA: [u8; 3] = [0x44, 0x43, 0x42];

pub(super) const ANCILLA24_GRAVESTONE_ANCILLA_GRAVESTONE_CHAR: [u8; 4] = [0xc8, 0xc8, 0xd8, 0xd8];

pub(super) const ANCILLA24_GRAVESTONE_ANCILLA_GRAVESTONE_FLAGS: [u8; 4] = [0, 0x40, 0, 0x40];

pub(super) const ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW_Y: [i8; 4] = [0, 0, 0, -3];

pub(super) const ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW_CHAR: [u8; 4] =
    [0x8e, 0xa0, 0xa2, 0xa4];

pub(super) const ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW_EXT: [u8; 4] = [2, 2, 2, 0];

pub(super) const ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_X: [i8; 24] = [
    -13, -21, -10, -1, -1, -1, -16, -27, -4, -16, -6, -25, -16, -27, -4, -16, -6, -25, -13, -5,
    -27, -11, -22, -3,
];

pub(super) const ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_Y: [i8; 24] = [
    -31, -24, -22, -1, -1, -1, -37, -32, -32, -23, -16, -14, -37, -32, -32, -23, -16, -14, -35,
    -29, -28, -20, -13, -11,
];

pub(super) const ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_CHAR: [u8; 24] = [
    0x86, 0x86, 0x86, 0xff, 0xff, 0xff, 0x86, 0x86, 0x86, 0x86, 0x86, 0x86, 0x8a, 0x8a, 0x8a, 0x8a,
    0x8a, 0x8a, 0x9b, 0x9b, 0x9b, 0x9b, 0x9b, 0x9b,
];

pub(super) const ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_FLAGS: [u8; 24] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80, 0x40, 0x40, 0x80, 0x40, 0,
];

pub(super) const ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_EXT: [u8; 24] = [
    2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0,
];

pub(super) const MORPH_POOF_DRAW_MORPH_POOF_OFFSET: [SignedOffset; 12] = signed_offsets![
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 0, 8, 8, -4, -4, -4, 12, 12, -4, 12, 12,
];

pub(super) const MORPH_POOF_DRAW_MORPH_POOF_FLAGS: [u8; 12] = [
    0, 0xff, 0xff, 0xff, 0x40, 0, 0xc0, 0x80, 0, 0x40, 0x80, 0xc0,
];

pub(super) const MORPH_POOF_DRAW_MORPH_POOF_CHAR: [u8; 3] = [0x86, 0xa9, 0x9b];

pub(super) const MORPH_POOF_DRAW_MORPH_POOF_EXT: [u8; 3] = [2, 0, 0];

pub(super) const ANCILLA3_F_BUSH_POOF_BUSH_POOF_DRAW_X: [i8; 16] =
    [0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, -2, 10, -2, 10];

pub(super) const ANCILLA3_F_BUSH_POOF_BUSH_POOF_DRAW_Y: [i8; 16] =
    [0, 0, 8, 8, 0, 0, 8, 8, 0, 0, 8, 8, -2, -2, 10, 10];

pub(super) const ANCILLA3_F_BUSH_POOF_BUSH_POOF_DRAW_CHAR: [u8; 16] = [
    0x86, 0x87, 0x96, 0x97, 0xa9, 0xa9, 0xa9, 0xa9, 0x8a, 0x8b, 0x9a, 0x9b, 0x9b, 0x9b, 0x9b, 0x9b,
];

pub(super) const ANCILLA3_F_BUSH_POOF_BUSH_POOF_DRAW_FLAGS: [u8; 16] = [
    0, 0, 0, 0, 0, 0x40, 0x80, 0xc0, 0, 0, 0, 0, 0xc0, 0x80, 0x40, 0,
];

pub(super) const ANCILLA26_SWORD_SWING_SPARKLE_SWORD_SWING_SPARKLE_X: [i8; 48] = [
    5, 10, -1, 5, 10, -4, 5, 10, -4, -4, -1, -1, 0, 5, -1, 0, 5, 14, 0, 5, 14, 14, -1, -1, -23,
    -27, -1, -23, -27, -22, -23, -27, -22, -22, -1, -1, 32, 35, -1, 32, 35, 30, 32, 35, 30, 30, -1,
    -1,
];

pub(super) const ANCILLA26_SWORD_SWING_SPARKLE_SWORD_SWING_SPARKLE_Y: [i8; 48] = [
    -22, -18, -1, -22, -18, -17, -22, -18, -17, -17, -1, -1, 35, 40, -1, 35, 40, 37, 35, 40, 37,
    37, -1, -1, 2, 7, -1, 2, 7, 19, 2, 7, 19, 19, -1, -1, 2, 7, -1, 2, 7, 19, 2, 7, 19, 19, -1, -1,
];

pub(super) const ANCILLA26_SWORD_SWING_SPARKLE_SWORD_SWING_SPARKLE_CHAR: [u8; 48] = [
    0xb7, 0xb7, 0xff, 0x80, 0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7, 0xff, 0x80,
    0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7, 0xff, 0x80, 0x80, 0xb7, 0x83, 0x83,
    0x80, 0x83, 0xff, 0xff, 0xb7, 0xb7, 0xff, 0x80, 0x80, 0xb7, 0x83, 0x83, 0x80, 0x83, 0xff, 0xff,
];

pub(super) const ANCILLA26_SWORD_SWING_SPARKLE_SWORD_SWING_SPARKLE_FLAGS: [u8; 48] = [
    0, 0, 0xff, 0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff, 0, 0, 0xff, 0, 0, 0, 0x80, 0x80, 0, 0x80,
    0xff, 0xff, 0, 0, 0xff, 0, 0, 0, 0x80, 0x80, 0, 0x80, 0xff, 0xff, 0, 0, 0xff, 0, 0, 0, 0x80,
    0x80, 0, 0x80, 0xff, 0xff,
];

pub(super) const ANCILLA2_D_SOMARIA_BLOCK_FIZZ_SOMARIA_BLOCK_FIZZLE_X: [i8; 6] =
    [-4, -1, -8, 0, -6, -2];

pub(super) const ANCILLA2_D_SOMARIA_BLOCK_FIZZ_SOMARIA_BLOCK_FIZZLE_Y: [i8; 6] =
    [-4, -1, -4, -4, -4, -4];

pub(super) const ANCILLA2_D_SOMARIA_BLOCK_FIZZ_SOMARIA_BLOCK_FIZZLE_CHAR: [u8; 6] =
    [0x92, 0xff, 0xf9, 0xf9, 0xf9, 0xf9];

pub(super) const ANCILLA2_D_SOMARIA_BLOCK_FIZZ_SOMARIA_BLOCK_FIZZLE_FLAGS: [u8; 6] =
    [6, 0xff, 0x86, 0xc6, 0x86, 0xc6];

pub(super) const ANCILLA39_SOMARIA_PLATFORM_POOF_SOMARIAN_PLATFORM_POOF_DIRECTION_BY_OPEN_SIDE:
    [u8; 4] = [1, 0, 3, 2];

pub(super) const ANCILLA3_A_BIG_BOMB_EXPLOSION_SUPER_BOMB_EXPLODE_X: [i8; 9] =
    [0, -16, 0, 16, -24, 24, -16, 0, 16];

pub(super) const ANCILLA3_A_BIG_BOMB_EXPLOSION_SUPER_BOMB_EXPLODE_Y: [i8; 9] =
    [0, -16, -24, -16, 0, 0, 16, 24, 16];

pub(super) const ANCILLA3_A_BIG_BOMB_EXPLOSION_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const ANCILLA3_B_SWORD_UP_SPARKLE_ANCILLA_VICTORY_SPARKLE_X: [i8; 16] =
    [16, 0, 0, 0, 8, 16, 8, 16, 9, 15, 0, 0, 12, 0, 0, 0];

pub(super) const ANCILLA3_B_SWORD_UP_SPARKLE_ANCILLA_VICTORY_SPARKLE_Y: [i8; 16] =
    [-7, 0, 0, 0, -11, -11, -3, -3, -7, -7, 0, 0, -7, 0, 0, 0];

pub(super) const ANCILLA3_B_SWORD_UP_SPARKLE_ANCILLA_VICTORY_SPARKLE_CHAR: [u8; 16] = [
    0x92, 0xff, 0xff, 0xff, 0x93, 0x93, 0x93, 0x93, 0xf9, 0xf9, 0xff, 0xff, 0x80, 0xff, 0xff, 0xff,
];

pub(super) const ANCILLA3_B_SWORD_UP_SPARKLE_ANCILLA_VICTORY_SPARKLE_FLAGS: [u8; 16] = [
    0, 0xff, 0xff, 0xff, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0xff, 0xff, 0, 0xff, 0xff, 0xff,
];

pub(super) const SPIN_SPARK_DRAW_INITIAL_SPIN_SPARK_CHAR: [u8; 32] = [
    0x92, 0xff, 0xff, 0xff, 0x8c, 0x8c, 0x8c, 0x8c, 0xd6, 0xd6, 0xd6, 0xd6, 0x93, 0x93, 0x93, 0x93,
    0xd6, 0xd6, 0xd6, 0xd6, 0xd7, 0xff, 0xff, 0xff, 0x80, 0xff, 0xff, 0xff, 0x22, 0xff, 0xff, 0xff,
];

pub(super) const SPIN_SPARK_DRAW_INITIAL_SPIN_SPARK_FLAGS: [u8; 29] = [
    0x22, 0xff, 0xff, 0xff, 0x22, 0x62, 0xa2, 0xe2, 0x24, 0x64, 0xa4, 0xe4, 0x22, 0x62, 0xa2, 0xe2,
    0x22, 0x62, 0xa2, 0xe2, 0x22, 0xff, 0xff, 0xff, 0x22, 0xff, 0xff, 0xff, 0xfc,
];

pub(super) const SPIN_SPARK_DRAW_INITIAL_SPIN_SPARK_Y: [i8; 29] = [
    -4, 0, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -8, -8, 0, 0, -4, 0, 0, 0, -4, 0, 0, 0,
    -4,
];

pub(super) const SPIN_SPARK_DRAW_INITIAL_SPIN_SPARK_X: [i16; 29] = [
    -4, 0, 0, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -8, 0, -4, 0, 0, 0, -4, 0, 0, 0,
    0x11a5,
];

pub(super) const ANCILLA2_A_SPIN_ATTACK_SPARKLE_A_INITIAL_SPIN_SPARK_TIMER: [u8; 6] =
    [4, 2, 3, 3, 2, 1];

pub(super) const SPIN_ATTACK_SPARKLE_A_TRANSMUTE_TO_NEXT_SPARK_TRANSMUTE_SPIN_SPARK_ARR: [u8; 16] = [
    0x21, 0x20, 0x1f, 0x1e, 3, 2, 1, 0, 0x12, 0x11, 0x10, 0x0f, 0x31, 0x30, 0x2f, 0x2e,
];

pub(super) const SPIN_ATTACK_SPARKLE_A_TRANSMUTE_TO_NEXT_SPARK_TRANSMUTE_SPIN_SPARK_X: [i8; 4] =
    [-3, 21, 25, -8];

pub(super) const SPIN_ATTACK_SPARKLE_A_TRANSMUTE_TO_NEXT_SPARK_TRANSMUTE_SPIN_SPARK_Y: [i8; 4] =
    [28, -2, 24, 6];

pub(super) const ANCILLA2_B_SPIN_ATTACK_SPARKLE_B_SPIN_SPARK_CHAR: [u8; 4] =
    [0xd7, 0xb7, 0x80, 0x83];

pub(super) const ANCILLA35_MASTER_SWORD_RECEIPT_SWORD_CEREMONY_X: [i8; 8] =
    [-1, 8, -1, 8, 0, 7, 0, 7];

pub(super) const ANCILLA35_MASTER_SWORD_RECEIPT_SWORD_CEREMONY_Y: [i8; 8] =
    [1, 1, 9, 9, 1, 1, 9, 9];

pub(super) const ANCILLA35_MASTER_SWORD_RECEIPT_SWORD_CEREMONY_CHAR: [u8; 8] =
    [0x86, 0x86, 0x96, 0x96, 0x87, 0x87, 0x97, 0x97];

pub(super) const ANCILLA35_MASTER_SWORD_RECEIPT_SWORD_CEREMONY_FLAGS: [u8; 8] =
    [1, 0x41, 1, 0x41, 1, 0x41, 1, 0x41];

pub(super) const ANCILLA36_FLUTE_FLUTE_VELS: [u8; 4] = [0x18, 0x10, 0x0a, 0];

pub(super) const ANCILLA2_C_SOMARIA_BLOCK_SOMARIAN_BLOCK_COLL_X: [i8; 12] =
    [0, 0, -8, 8, 0, 0, 0, 0, 8, -8, -8, 8];

pub(super) const ANCILLA2_C_SOMARIA_BLOCK_SOMARIAN_BLOCK_COLL_Y: [i8; 12] =
    [-8, 8, 0, 0, 0, 0, 0, 0, -8, 8, -8, 8];

pub(super) const ANCILLA_DRAW_QUAKE_INITIAL_BOLTS_QUAKE_GROUND_BOLT_OAM_STARTS: [u8; 5] =
    [0, 0x18, 0, 0x18, 0x2f];

pub(super) const ANCILLA1_F_HOOKSHOT_HOOKSHOT_MOVE_X: [i8; 4] = [0, 0, 8, -8];

pub(super) const ANCILLA1_F_HOOKSHOT_HOOKSHOT_MOVE_Y: [i8; 4] = [8, -9, 0, 0];

pub(super) const ANCILLA1_F_HOOKSHOT_HOOKSHOT_DRAW_FLAGS: [u8; 12] =
    [0, 0, 0xff, 0x80, 0x80, 0xff, 0x40, 0xff, 0x40, 0, 0xff, 0];

pub(super) const ANCILLA1_F_HOOKSHOT_HOOKSHOT_DRAW_CHAR: [u8; 12] =
    [9, 0x0a, 0xff, 9, 0x0a, 0xff, 9, 0xff, 0x0a, 9, 0xff, 0x0a];

pub(super) const ANCILLA_DRAW_ETHER_BLITZ_BALL_ETHER_BLITZ_BALL_CHAR: [u8; 2] = [0x68, 0x6a];

pub(super) const ANCILLA_DRAW_ETHER_BLITZ_SEGMENT_ETHER_SPLITTING_BLITZ_SEGMENT_X: [i8; 16] = [
    -8, -16, -24, -16, -8, 0, 8, -16, -8, -16, -24, -16, -8, 0, 8, 0,
];

pub(super) const ANCILLA_DRAW_ETHER_BLITZ_SEGMENT_ETHER_SPLITTING_BLITZ_SEGMENT_Y: [i8; 16] = [
    8, 0, -8, -16, -24, -16, -8, -16, 8, 0, -8, -16, -24, -16, -8, 0,
];

pub(super) const ANCILLA_DRAW_ETHER_BLITZ_SEGMENT_ETHER_SPLITTING_BLITZ_SEGMENT_CHAR: [u8; 32] = [
    0x40, 0x42, 0x66, 0x64, 0x62, 0x60, 0x64, 0x66, 0x42, 0x40, 0x66, 0x64, 0x60, 0x62, 0x64, 0x66,
    0x68, 0x42, 0x68, 0x64, 0x68, 0x60, 0x68, 0x64, 0x68, 0x40, 0x68, 0x66, 0x68, 0x62, 0x68, 0x64,
];

pub(super) const ANCILLA_DRAW_ETHER_BLITZ_SEGMENT_ETHER_SPLITTING_BLITZ_SEGMENT_FLAGS: [u8; 32] = [
    0x3c, 0x3c, 0xfc, 0xfc, 0x3c, 0x3c, 0xbc, 0xbc, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x7c, 0x7c,
    0x3c, 0x7c, 0x3c, 0x3c, 0x3c, 0xbc, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0xfc, 0x3c, 0xbc, 0x3c, 0xbc,
];

pub(super) const ANCILLA_DRAW_ETHER_BLITZ_ETHER_BLITZ_ORB_FLAGS: [u8; 8] =
    [0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c];

pub(super) const ANCILLA_DRAW_ETHER_BLITZ_ETHER_BLITZ_SEGMENT_CHAR: [u8; 4] =
    [0x40, 0x42, 0x44, 0x46];

pub(super) const ANCILLA_DRAW_ETHER_ORB_ETHER_BLITZ_ORB_CHAR: [u8; 8] =
    [0x48, 0x48, 0x4a, 0x4a, 0x4c, 0x4c, 0x4e, 0x4e];

pub(super) const ANCILLA_DRAW_ETHER_ORB_ETHER_BLITZ_ORB_FLAGS: [u8; 8] =
    [0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c, 0x3c, 0x7c];

pub(super) const ANCILLA_DRAW_BOMBOS_FIRE_COLUMN_BOMBOS_SPELL_FIRE_COLUMN_X: [i8; 39] = [
    0, -1, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, -1, 1, -1, -1, 2, -1, -1,
];

pub(super) const ANCILLA_DRAW_BOMBOS_FIRE_COLUMN_BOMBOS_SPELL_FIRE_COLUMN_Y: [i8; 39] = [
    0, -1, -1, 0, -4, -1, 0, -8, -1, 0, -12, -1, 0, -16, -1, 0, -4, -20, 0, -8, -24, 0, -12, -28,
    0, -16, -32, 0, -16, -32, -18, -34, -1, -35, -1, -1, -36, -1, -1,
];

pub(super) const ANCILLA_DRAW_BOMBOS_FIRE_COLUMN_BOMBOS_SPELL_FIRE_COLUMN_FLAGS: [u8; 39] = [
    0x3c, 0xff, 0xff, 0x3c, 0x3c, 0xff, 0x3c, 0x3c, 0xff, 0x7c, 0x7c, 0xff, 0x3c, 0x7c, 0xff, 0x3c,
    0x3c, 0x3c, 0xbc, 0x3c, 0x3c, 0x7c, 0x3c, 0x3c, 0x3c, 0x3c, 0x7c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c,
    0xff, 0x3c, 0xff, 0xff, 0x3c, 0xff, 0xff,
];

pub(super) const ANCILLA_DRAW_BOMBOS_FIRE_COLUMN_BOMBOS_SPELL_FIRE_COLUMN_CHAR: [u8; 39] = [
    0x40, 0xff, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44, 0xff, 0x42, 0x44, 0xff, 0x40,
    0x46, 0x44, 0x4a, 0x4a, 0x48, 0x4c, 0x4c, 0x4a, 0x4e, 0x4c, 0x4a, 0x4e, 0x6a, 0x4c, 0x4e, 0x68,
    0xff, 0x6a, 0xff, 0xff, 0x4e, 0xff, 0xff,
];

pub(super) const ANCILLA_DRAW_BOMBOS_BLAST_BOMBOS_SPELL_DRAW_BLAST_X: [i8; 32] = [
    -8, -1, -1, -1, -12, -4, -12, -4, -16, 0, -16, 0, -16, 0, -16, 0, -17, 1, -17, 1, -19, 3, -19,
    3, -19, 3, -19, 3, -19, 3, -19, 3,
];

pub(super) const ANCILLA_DRAW_BOMBOS_BLAST_BOMBOS_SPELL_DRAW_BLAST_Y: [i8; 32] = [
    -8, -1, -1, -1, -12, -12, -4, -4, -16, -16, 0, 0, -16, -16, 0, 0, -17, -17, 1, 1, -19, -19, 3,
    3, -19, -19, 3, 3, -19, -19, 3, 3,
];

pub(super) const ANCILLA_DRAW_BOMBOS_BLAST_BOMBOS_SPELL_DRAW_BLAST_FLAGS: [u8; 32] = [
    0x3c, 0xff, 0xff, 0xff, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc,
    0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc, 0x3c, 0x7c, 0xbc, 0xfc,
];

pub(super) const ANCILLA_DRAW_BOMBOS_BLAST_BOMBOS_SPELL_DRAW_BLAST_CHAR: [u8; 32] = [
    0x60, 0xff, 0xff, 0xff, 0x62, 0x62, 0x62, 0x62, 0x64, 0x64, 0x64, 0x64, 0x66, 0x66, 0x66, 0x66,
    0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x6a, 0x6a, 0x6a, 0x6a, 0x4e, 0x4e, 0x4e, 0x4e,
];

pub(super) const ANCILLA_DRAW_WEATHERVANE_EXPLOSION_WOOD_DEBRIS_WEATHERVANE_EXPLODE_CHAR: [u8; 2] =
    [0x4e, 0x4f];

pub(super) const ANCILLA38_CUTSCENE_DUCK_TRAVEL_BIRD_INTRO_FLAGS_BY_DIRECTION: [u8; 2] = [0x40, 0];

pub(super) const ANCILLA38_CUTSCENE_DUCK_TRAVEL_BIRD_INTRO_X_SPEED_LIMITS: [u8; 2] = [28, 60];

pub(super) const ANCILLA38_CUTSCENE_DUCK_AFTER_STUFF_TRAVEL_BIRD_INTRO_FLAGS_BY_DIRECTION: [u8; 2] =
    [0x40, 0];

pub(super) const ANCILLA16_HIT_STARS_ANCILLA_HIT_STARS_CHAR: [u8; 2] = [0x90, 0x91];

pub(super) const ANCILLA17_SHOVEL_DIRT_SHOVEL_DIRT_XY: [i8; 8] = [18, -13, -9, 4, 18, 13, -9, -11];

pub(super) const ANCILLA17_SHOVEL_DIRT_SHOVEL_DIRT_CHAR: [u8; 2] = [0x40, 0x50];

pub(super) const ANCILLA_MAGIC_POWDER_DRAW_MAGIC_POWDER_DRAW_X: [i8; 76] = [
    -5, -12, 2, -9, -7, -10, -6, -2, -6, -12, 1, -6, -6, -12, 1, -6, -6, -12, 1, -6, -6, -12, 1,
    -6, -6, -12, 1, -6, -17, -23, -14, -19, -11, -18, -9, -13, -4, -13, -1, -8, -3, -9, 0, -5, -3,
    -10, -1, -5, -4, -13, -1, -8, -3, -9, 0, -5, -3, -10, -1, -5, -3, -13, -1, -8, 9, 15, 6, 11, 3,
    10, 1, 5, -4, 5, -7, 0,
];

pub(super) const ANCILLA_MAGIC_POWDER_DRAW_MAGIC_POWDER_DRAW_Y: [i8; 76] = [
    -20, -15, -13, -7, -18, -13, -13, -13, -20, -13, -13, -8, -20, -13, -13, -8, -19, -12, -12, -7,
    -18, -11, -11, -6, -17, -10, -10, -5, -16, -14, -12, -9, -17, -14, -12, -8, -18, -14, -13, -6,
    -33, -31, -29, -26, -28, -25, -23, -19, -22, -18, -17, -10, -2, 0, 2, 5, -9, -6, -4, 0, -16,
    -12, -11, -4, -16, -14, -12, -9, -17, -14, -12, -8, -18, -14, -13, -6,
];

pub(super) const ANCILLA_MAGIC_POWDER_DRAW_MAGIC_POWDER_DRAW_CHAR: [u8; 19] =
    [9, 10, 10, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9];

pub(super) const ANCILLA_MAGIC_POWDER_DRAW_MAGIC_POWDER_DRAW_FLAGS: [u8; 76] = [
    0x68, 0x24, 0xa2, 0x28, 0x68, 0xe2, 0x28, 0xa4, 0x68, 0xe2, 0xa4, 0x28, 0x22, 0xa4, 0xe8, 0x62,
    0x24, 0xa8, 0xe2, 0x64, 0x28, 0xa2, 0xe4, 0x68, 0x22, 0xa4, 0xe8, 0x62, 0xe2, 0xa4, 0xe8, 0x64,
    0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4, 0xe8, 0x64, 0xe8, 0xa8, 0xe4, 0x62,
    0xe4, 0xa8, 0xe2, 0x68, 0xe2, 0xa4, 0xe8, 0x64, 0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68,
    0xe2, 0xa4, 0xe8, 0x64, 0xe8, 0xa8, 0xe4, 0x62, 0xe4, 0xa8, 0xe2, 0x68,
];

pub(super) const ANCILLA_RISING_CRYSTAL_DUNGEON_CRYSTAL_PENDANT_BIT: [u8; 13] =
    [0, 0, 4, 2, 0, 16, 2, 1, 64, 4, 1, 32, 8];

pub(super) const ANCILLA3_C_SPIN_ATTACK_CHARGE_SPARKLE_SWORD_CHARGE_SPARK_CHAR: [u8; 3] =
    [0xb7, 0x80, 0x83];

pub(super) const ANCILLA3_C_SPIN_ATTACK_CHARGE_SPARKLE_SWORD_CHARGE_SPARK_FLAGS: [u8; 3] =
    [4, 4, 0x84];

pub(super) const ANCILLA2_E_SOMARIA_BLOCK_FISSION_SOMARIAN_BLOCK_DIVIDE_X: [i8; 16] =
    [-8, 0, -8, 0, -10, -10, 2, 2, -8, 0, -8, 0, -12, -12, 4, 4];

pub(super) const ANCILLA2_E_SOMARIA_BLOCK_FISSION_SOMARIAN_BLOCK_DIVIDE_Y: [i8; 16] =
    [-10, -10, 2, 2, -8, 0, -8, 0, -12, -12, 4, 4, -8, 0, -8, 0];

pub(super) const ANCILLA2_E_SOMARIA_BLOCK_FISSION_SOMARIAN_BLOCK_DIVIDE_CHAR: [u8; 16] = [
    0xc6, 0xc6, 0xc6, 0xc6, 0xc4, 0xc4, 0xc4, 0xc4, 0xd2, 0xd2, 0xd2, 0xd2, 0xc5, 0xc5, 0xc5, 0xc5,
];

pub(super) const ANCILLA2_E_SOMARIA_BLOCK_FISSION_SOMARIAN_BLOCK_DIVIDE_FLAGS: [u8; 16] = [
    0xc6, 0x86, 0x46, 6, 0x46, 0xc6, 6, 0x86, 0xc6, 0x86, 0x46, 6, 0x46, 0xc6, 6, 0x86,
];

pub(super) const ANCILLA2_F_LAMP_FLAME_LAMP_FLAME_DRAW_CHAR: [u8; 12] = [
    0x9c, 0x9c, 0xff, 0xff, 0xa4, 0xa5, 0xb2, 0xb3, 0xe3, 0xf3, 0xff, 0xff,
];

pub(super) const ANCILLA2_F_LAMP_FLAME_LAMP_FLAME_DRAW_Y: [i8; 12] =
    [-3, 0, 0, 0, 0, 0, 8, 8, 0, 8, 0, 0];

pub(super) const ANCILLA2_F_LAMP_FLAME_LAMP_FLAME_DRAW_X: [i8; 12] =
    [4, 10, 0, 0, 1, 9, 2, 7, 4, 4, 0, 0];

pub(super) const ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_X: [i8; 8] =
    [0, 0, -4, 4, -7, 7, -9, 17];

pub(super) const ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_Y: [i8; 8] =
    [-4, 0, -5, -5, -3, -3, 12, 12];

pub(super) const ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_CHAR: [u8; 8] =
    [0xc0, 0xff, 0xac, 0xac, 0xae, 0xae, 0xbf, 0xbf];

pub(super) const ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_FLAGS: [u8; 8] =
    [0x84, 0xff, 0x84, 0xc4, 0x84, 0xc4, 0x84, 0xc4];

pub(super) const ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_EXT: [u8; 8] =
    [2, 0xff, 2, 2, 2, 2, 0, 0];

pub(super) const ANCILLA15_JUMP_SPLASH_ANCILLA_JUMP_SPLASH_CHAR: [u8; 2] = [0xac, 0xae];

pub(super) const ANCILLA04_BEAM_HIT_BEAM_HIT_X: [i8; 16] =
    [-12, 20, -12, 20, -8, 16, -8, 16, -4, 12, -4, 12, 0, 8, 0, 8];

pub(super) const ANCILLA04_BEAM_HIT_BEAM_HIT_Y: [i8; 16] =
    [-12, -12, 20, 20, -8, -8, 16, 16, -4, -4, 12, 12, 0, 0, 8, 8];

pub(super) const ANCILLA04_BEAM_HIT_BEAM_HIT_CHAR: [u8; 16] = [
    0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x53, 0x54, 0x54, 0x54, 0x54,
];

pub(super) const ANCILLA04_BEAM_HIT_BEAM_HIT_FLAGS: [u8; 16] = [
    0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0, 0x40, 0x80, 0xc0,
];

pub(super) const ANCILLA13_ICE_ROD_SPARKLE_ICE_SHOT_SPARKLE_X: [u8; 16] =
    [2, 7, 6, 1, 1, 7, 7, 1, 0, 7, 8, 1, 4, 9, 4, 0xff];

pub(super) const ANCILLA13_ICE_ROD_SPARKLE_ICE_SHOT_SPARKLE_Y: [u8; 16] =
    [2, 3, 8, 7, 1, 1, 7, 7, 1, 0, 7, 8, 0xff, 4, 9, 4];

pub(super) const ANCILLA13_ICE_ROD_SPARKLE_ICE_SHOT_SPARKLE_CHAR: [u8; 16] = [
    0x83, 0x83, 0x83, 0x83, 0xb6, 0x80, 0xb6, 0x80, 0xb7, 0xb6, 0xb7, 0xb6, 0xb7, 0xb6, 0xb7, 0xb6,
];

pub(super) const ANCILLA_ADD_ICE_ROD_SPARKLE_ICE_SHOT_SPARKLE_XVEL: [i8; 4] = [0, 0, -4, 4];

pub(super) const ANCILLA_ADD_ICE_ROD_SPARKLE_ICE_SHOT_SPARKLE_YVEL: [i8; 4] = [-4, 4, 0, 0];

pub(super) const ANCILLA_CHECK_LINK_COLLISION_OUT_YOFFS: [u16; 5] = [0, 8, 8, 8, 0];

pub(super) const ANCILLA_CHECK_LINK_COLLISION_OUT_XOFFS: [u16; 5] = [0, 8, 8, 8, 0];

pub(super) const ANCILLA_CHECK_LINK_COLLISION_OUT_H: [u16; 5] = [20, 20, 8, 28, 14];

pub(super) const ANCILLA_CHECK_LINK_COLLISION_OUT_W: [u16; 5] = [20, 3, 8, 24, 14];

pub(super) const ANCILLA_CHECK_LINK_COLLISION_OUT_LINK_YOFFS: [u16; 5] = [12, 12, 12, 12, 12];

pub(super) const ANCILLA_CHECK_LINK_COLLISION_OUT_LINK_XOFFS: [u16; 5] = [8, 8, 8, 12, 8];

pub(super) const ANCILLA_CHECK_TILE_COLLISION_ONE_FLOOR_CHECK_TILE_COLL0_X: [i8; 20] = [
    8, 8, 0, 16, 4, 4, 0, 16, 4, 4, 4, 12, 12, 12, 4, 12, 0, 0, 0, 0,
];

pub(super) const ANCILLA_CHECK_TILE_COLLISION_ONE_FLOOR_CHECK_TILE_COLL0_Y: [i8; 20] = [
    0, 16, 5, 5, 0, 16, 4, 4, 4, 12, 5, 5, 4, 12, 12, 12, 0, 0, 0, 0,
];

pub(super) const ANCILLA_CHECK_INITIAL_TILE_A_YOFFS_HB: [i8; 12] =
    [8, 0, -8, 8, 16, 24, 8, 8, 8, 8, 8, 8];

pub(super) const ANCILLA_CHECK_INITIAL_TILE_A_XOFFS_HB: [i8; 12] =
    [0, 0, 0, 0, 0, 0, 0, -8, -16, 0, 8, 16];

pub(super) const ANCILLA_RETURN_IF_OUTSIDE_BOUNDS_ANCILLA_FLOOR_FLAGS: [u8; 2] = [0x20, 0x10];

pub(super) const ANCILLA_APPLY_CONVEYOR_ANCILLA_BELT_XVEL: [i8; 4] = [0, 0, -8, 8];

pub(super) const ANCILLA_APPLY_CONVEYOR_ANCILLA_BELT_YVEL: [i8; 4] = [-8, 8, 0, 0];

pub(super) const ANCILLA_GET_RADIAL_PROJECTION_RADIAL_PROJECTION_PRIMARY_MAGNITUDE: [u8; 64] = [
    255, 254, 251, 244, 236, 225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49, 74, 97,
    120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254, 251, 244, 236, 225, 212, 197,
    181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49, 74, 97, 120, 142, 162, 181, 197, 212, 225, 236,
    244, 251, 254,
];

pub(super) const ANCILLA_GET_RADIAL_PROJECTION_RADIAL_PROJECTION_SECONDARY_MAGNITUDE: [u8; 64] = [
    0, 25, 49, 74, 97, 120, 142, 162, 181, 197, 212, 225, 236, 244, 251, 254, 255, 254, 251, 244,
    236, 225, 212, 197, 181, 162, 142, 120, 97, 74, 49, 25, 0, 25, 49, 74, 97, 120, 142, 162, 181,
    197, 212, 225, 236, 244, 251, 254, 255, 254, 251, 244, 236, 225, 212, 197, 181, 162, 142, 120,
    97, 74, 49, 25,
];

pub(super) const ANCILLA_GET_RADIAL_PROJECTION_RADIAL_PROJECTION_PRIMARY_SIGN: [u8; 64] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

pub(super) const ANCILLA_GET_RADIAL_PROJECTION_RADIAL_PROJECTION_SECONDARY_SIGN: [u8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

pub(super) const OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_X: [i8; 10] =
    [0, 0, 0, 0, 11, -3, 15, -7, 15, -7];

pub(super) const OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_Y: [i8; 10] =
    [0, 0, -6, 0, -13, -8, -17, -4, -17, -4];

pub(super) const OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_CHAR: [u8; 10] =
    [0xc0, 0xff, 0xe7, 0xff, 0xaf, 0xbf, 0x80, 0x80, 0x83, 0x83];

pub(super) const OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_FLAGS: [u8; 10] =
    [0, 0xff, 0, 0xff, 0x40, 0, 0x40, 0, 0xc0, 0x80];

pub(super) const OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_EXT: [u8; 10] =
    [2, 0, 2, 0, 0, 0, 0, 0, 0, 0];

pub(super) const ANCILLA_HANDLE_LIFT_LOGIC_ANCILLA_LIFTABLE_DELAY: [u8; 3] = [16, 8, 9];

pub(super) const ANCILLA_LATCH_LINK_COORDINATES_ANCILLA_FUNC3_X: [i8; 12] =
    [8, 8, -4, 20, 8, 8, 8, 8, 8, 8, 8, 8];

pub(super) const ANCILLA_LATCH_LINK_COORDINATES_ANCILLA_FUNC3_Y: [i8; 12] =
    [16, 8, 4, 4, 8, 2, -1, -1, 2, 2, -1, -1];

pub(super) const ANCILLA_LATCH_CARRIED_POSITION_ANCILLA_FUNC2_Y: [i8; 6] = [-2, -1, 0, -2, -1, 0];

pub(super) const ANCILLA_CHECK_TILE_COLLISION_CLASS2_INNER_Y_OFFSETS: [i8; 4] = [-8, 8, 0, 0];

pub(super) const ANCILLA_CHECK_TILE_COLLISION_CLASS2_INNER_X_OFFSETS: [i8; 4] = [0, 0, -8, 8];

pub(super) const ANCILLA_CHECK_INITIAL_TILE_COLLISION_CLASS2_INITIAL_TILE_COLL_Y: [i16; 9] =
    [15, 16, 28, 24, 12, 12, 12, 12, 8];

pub(super) const ANCILLA_CHECK_INITIAL_TILE_COLLISION_CLASS2_INITIAL_TILE_COLL_X: [i16; 9] =
    [8, 8, 8, 8, -1, 0, 17, 16, 0x4b8b];

pub(super) const SOMARIA_BLOCK_CHECK_FOR_TRANSIT_TILE_SOMARIA_TRANSIT_LINE_X: [i8; 12] =
    [-8, 0, 8, -8, 0, 8, -16, -16, -16, 16, 16, 16];

pub(super) const SOMARIA_BLOCK_CHECK_FOR_TRANSIT_TILE_SOMARIA_TRANSIT_LINE_Y: [i8; 12] =
    [-16, -16, -16, 16, 16, 16, -8, 0, 8, -8, 0, 8];

pub(super) const ANCILLA_ADD_SPIN_ATTACK_INIT_SPARK_SPIN_ATTACK_START_SPARKLE_Y: [i8; 4] =
    [32, -8, 10, 20];

pub(super) const ANCILLA_ADD_SPIN_ATTACK_INIT_SPARK_SPIN_ATTACK_START_SPARKLE_X: [i8; 4] =
    [10, 7, 28, -10];

pub(super) const ANCILLA_ADD_SILVER_ARROW_SPARKLE_SILVER_ARROW_SPARKLE_X: [i8; 4] = [-4, -4, 0, 2];

pub(super) const ANCILLA_ADD_SILVER_ARROW_SPARKLE_SILVER_ARROW_SPARKLE_Y: [i8; 4] = [0, 2, -4, -4];

pub(super) const ANCILLA_ADD_ICE_ROD_SHOT_ICE_ROD_X: [i8; 4] = [0, 0, -20, 20];

pub(super) const ANCILLA_ADD_ICE_ROD_SHOT_ICE_ROD_Y: [i8; 4] = [-16, 24, 8, 8];

pub(super) const ANCILLA_ADD_ICE_ROD_SHOT_ICE_ROD_XVEL: [i8; 4] = [0, 0, -48, 48];

pub(super) const ANCILLA_ADD_ICE_ROD_SHOT_ICE_ROD_YVEL: [i8; 4] = [-48, 48, 0, 0];

pub(super) const ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_Y: [u16; 8] =
    [0x550, 0x540, 0x530, 0x520, 0x500, 0x4e0, 0x4c0, 0x4b0];

pub(super) const ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_X: [u16; 15] = [
    0x8b0, 0x8f0, 0x910, 0x950, 0x970, 0x9a0, 0x850, 0x870, 0x8b0, 0x8f0, 0x920, 0x950, 0x880,
    0x990, 0x840,
];

pub(super) const ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_Y1: [u16; 15] = [
    0x540, 0x530, 0x530, 0x530, 0x520, 0x520, 0x510, 0x510, 0x4f0, 0x4f0, 0x4f0, 0x4f0, 0x4d0,
    0x4b0, 0x4a0,
];

pub(super) const ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_X1: [u16; 15] = [
    0x8b0, 0x8f0, 0x910, 0x950, 0x970, 0x9a0, 0x850, 0x870, 0x8b0, 0x8f0, 0x920, 0x950, 0x880,
    0x990, 0x840,
];

pub(super) const ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_POS: [u16; 15] = [
    0xa16, 0x99e, 0x9a2, 0x9aa, 0x92e, 0x934, 0x88a, 0x88e, 0x796, 0x79e, 0x7a4, 0x7aa, 0x690,
    0x5b2, 0x508,
];

pub(super) const ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_CTR: [u8; 15] = [
    0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x38, 0x58,
];

pub(super) const ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_IDX: [u8; 9] =
    [0, 1, 4, 6, 8, 12, 13, 14, 15];

pub(super) const FIRE_SHOT_DRAW_FIRE_SHOT_DRAW_X2: [u8; 16] =
    [7, 0, 8, 0, 8, 4, 0, 0, 2, 8, 0, 0, 1, 4, 9, 0];

pub(super) const FIRE_SHOT_DRAW_FIRE_SHOT_DRAW_Y2: [u8; 16] =
    [1, 4, 9, 0, 7, 0, 8, 0, 8, 4, 0, 0, 2, 8, 0, 0];

pub(super) const FIRE_SHOT_DRAW_FIRE_SHOT_DRAW_CHAR2: [u8; 3] = [0x8d, 0x9d, 0x9c];

pub(super) const ICE_SHOT_SPREAD_DRAW_ICE_SHOT_SPREAD_TILE: [OamTileAttrs; 8] = oam_tile_attrs![
    0xcf, 0x24, 0xcf, 0x24, 0xcf, 0x24, 0xcf, 0x24, 0xdf, 0x24, 0xdf, 0x24, 0xdf, 0x24, 0xdf, 0x24,
];

pub(super) const ICE_SHOT_SPREAD_DRAW_ICE_SHOT_SPREAD_OFFSET: [SignedOffset; 8] =
    signed_offsets![0, 0, 0, 8, 8, 0, 8, 8, -8, -8, -8, 16, 16, -8, 16, 16,];

pub(super) const SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_FLAGS: [u8; 2] = [2, 6];

pub(super) const SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_X0: [i8; 24] = [
    0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(super) const SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_X1: [i8; 24] = [
    8, 8, 8, 8, 4, 4, 8, 8, 8, 8, 4, 4, 0, 0, 0, 0, 8, 8, 0, 0, 0, 0, 8, 8,
];

pub(super) const SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_Y0: [u8; 24] = [
    0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 4, 4,
];

pub(super) const SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_Y1: [u8; 24] = [
    0, 0, 0, 0, 8, 8, 0x80, 0, 0, 0, 8, 8, 0x80, 8, 8, 8, 4, 4, 0x80, 8, 8, 8, 4, 4,
];

pub(super) const SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_FLAGS0: [u8; 24] = [
    0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0xc0, 0x40, 0x40, 0x40, 0x40, 0, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0xc0, 0, 0, 0, 0, 0, 0x80,
];

pub(super) const SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_FLAGS1: [u8; 24] = [
    0x80, 0x80, 0x80, 0x80, 0x80, 0xc0, 0, 0, 0, 0, 0, 0x40, 0xc0, 0xc0, 0xc0, 0xc0, 0x40, 0xc0,
    0x80, 0x80, 0x80, 0x80, 0, 0x80,
];

pub(super) const SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_CHAR0: [u8; 24] = [
    0x50, 0x50, 0x44, 0x44, 0x52, 0x52, 0x50, 0x50, 0x44, 0x44, 0x51, 0x51, 0x43, 0x43, 0x42, 0x42,
    0x41, 0x41, 0x43, 0x43, 0x42, 0x42, 0x40, 0x40,
];

pub(super) const SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_CHAR1: [u8; 24] = [
    0x50, 0x50, 0x44, 0x44, 0x51, 0x51, 0x50, 0x50, 0x44, 0x44, 0x52, 0x52, 0x43, 0x43, 0x42, 0x42,
    0x40, 0x40, 0x43, 0x43, 0x42, 0x42, 0x41, 0x41,
];

pub(super) const ARROW_DRAW_ARROW_DRAW_CHAR: [u8; 48] = [
    0x2b, 0x2a, 0x2a, 0x2b, 0x3d, 0x3a, 0x3a, 0x3d, 0x2b, 0xff, 0x2b, 0xff, 0x3d, 0xff, 0x3d, 0xff,
    0x3c, 0x2c, 0x3c, 0x2a, 0x3c, 0x2c, 0x3c, 0x2a, 0x2c, 0x3c, 0x2a, 0x3c, 0x2c, 0x3c, 0x2a, 0x3c,
    0x3b, 0x2d, 0x3b, 0x3a, 0x3b, 0x2d, 0x3b, 0x3a, 0x2d, 0x3b, 0x3a, 0x3b, 0x2d, 0x3b, 0x3a, 0x3b,
];

pub(super) const ARROW_DRAW_ARROW_DRAW_FLAGS: [u8; 48] = [
    0xa4, 0xa4, 0x24, 0x24, 0x64, 0x64, 0x24, 0x24, 0xa4, 0xff, 0x24, 0xff, 0x64, 0xff, 0x24, 0xff,
    0xa4, 0xa4, 0xa4, 0xa4, 0xa4, 0xe4, 0xa4, 0xa4, 0x24, 0x24, 0x24, 0x24, 0x64, 0x24, 0x24, 0x24,
    0x64, 0x64, 0x64, 0xe4, 0x64, 0xe4, 0x64, 0xe4, 0x24, 0x24, 0x24, 0xa4, 0xa4, 0x24, 0x24, 0xa4,
];

pub(super) const ARROW_DRAW_ARROW_DRAW_Y: [i8; 48] = [
    0, 8, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8,
    -1, -1, 0, 0, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 0, 0,
];

pub(super) const ARROW_DRAW_ARROW_DRAW_X: [i8; 48] = [
    0, 0, 0, 0, 0, 8, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, -2, 0, 0, 1, 1, 0, 0, -2, -1,
    0, 0, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8,
];

pub(super) const REVIVAL_FAIRY_MONITOR_HP_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const REVIVAL_FAIRY_MAIN_REVIVAL_FAIRY_STEP_TIMERS: [u8; 2] = [0, 0x90];

pub(super) const REVIVAL_FAIRY_MAIN_REVIVAL_FAIRY_TILE_CHARS: [u8; 5] =
    [0x4b, 0x4d, 0x49, 0x47, 0x49];

pub(super) const GT_CUTSCENE_SPARKLE_A_LOT_SWORD_CHARGE_SPARK_CHAR: [u8; 3] = [0xb7, 0x80, 0x83];

pub(super) const GT_CUTSCENE_SPARKLE_A_LOT_SWORD_CHARGE_SPARK_FLAGS: [u8; 3] = [4, 4, 0x84];

pub(super) const ANCILLA_ADD_RUPEES_RUPEE_GIFT_AMOUNTS: [u16; 5] = [1, 5, 20, 100, 50];

pub(super) const SOMARIA_BLOCK_SPAWN_BULLETS_SPAWN_CENTRIFUGAL_QUAD_X: [i8; 4] = [-8, -8, -9, -4];

pub(super) const SOMARIA_BLOCK_SPAWN_BULLETS_SPAWN_CENTRIFUGAL_QUAD_Y: [i8; 4] = [-15, -4, -8, -8];

pub(super) const ANCILLA_DRAW_SOMARIA_BLOCK_SOMARIAN_BLOCK_DRAW_X: [i8; 12] =
    [-8, 0, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub(super) const ANCILLA_DRAW_SOMARIA_BLOCK_SOMARIAN_BLOCK_DRAW_Y: [i8; 12] =
    [-8, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub(super) const ANCILLA_DRAW_SOMARIA_BLOCK_SOMARIAN_BLOCK_DRAW_FLAGS: [u8; 12] = [
    0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0,
];

pub(super) const SOMARIA_BLOCK_CHECK_FOR_SWITCH_SOMARIAN_BLOCK_CHECK_COVER_X: [i8; 4] =
    [0, 0, -4, 4];

pub(super) const SOMARIA_BLOCK_CHECK_FOR_SWITCH_SOMARIAN_BLOCK_CHECK_COVER_Y: [i8; 4] =
    [-4, 4, 0, 0];

pub(super) const ANCILLA_SETUP_HIT_BOX_ANCILLA_HIT_BOX_X: [i8; 12] =
    [4, 4, 4, 4, 3, 3, 2, 11, -16, -16, -1, -8];

pub(super) const ANCILLA_SETUP_HIT_BOX_ANCILLA_HIT_BOX_Y: [i8; 12] =
    [4, 4, 4, 4, 2, 11, 3, 3, -1, -8, -16, -16];

pub(super) const ANCILLA_SETUP_HIT_BOX_ANCILLA_HIT_BOX_W: [u8; 12] =
    [8, 8, 8, 8, 1, 1, 1, 1, 32, 32, 8, 8];

pub(super) const ANCILLA_SETUP_HIT_BOX_ANCILLA_HIT_BOX_H: [u8; 12] =
    [8, 8, 8, 8, 1, 1, 1, 1, 8, 8, 32, 32];

pub(super) const ANCILLA_PREP_ADJUSTED_OAM_COORD_TAGALONG_LAYER_BITS: [u8; 4] =
    [0x20, 0x10, 0x30, 0x20];

pub(super) const ANCILLA_CHECK_FOR_ENTRANCE_TRIGGER_ENTRANCE_TRIGGER_BASE_Y: [u16; 4] =
    [0x0d40, 0x0210, 0x0cfc, 0x0100];

pub(super) const ANCILLA_CHECK_FOR_ENTRANCE_TRIGGER_ENTRANCE_TRIGGER_BASE_X: [u16; 4] =
    [0x0d80, 0x0e68, 0x0130, 0x0f10];

pub(super) const ANCILLA_CHECK_FOR_ENTRANCE_TRIGGER_ENTRANCE_TRIGGER_SIZE_Y: [u16; 4] =
    [11, 32, 16, 12];

pub(super) const ANCILLA_CHECK_FOR_ENTRANCE_TRIGGER_ENTRANCE_TRIGGER_SIZE_X: [u16; 4] =
    [16, 16, 16, 16];

pub(super) const GAME_OVER_TEXT_DRAW_GAME_OVER_TEXT_CHARS: [u8; 16] = [
    0x40, 0x50, 0x41, 0x51, 0x42, 0x52, 0x43, 0x53, 0x44, 0x54, 0x45, 0x55, 0x43, 0x53, 0x46, 0x56,
];

pub(super) const ANCILLA_DRAW_SHADOW_ANCILLA_DRAW_SHADOW_CHAR: [u8; 14] = [
    0x6c, 0x6c, 0x28, 0x28, 0x38, 0xff, 0xc8, 0xc8, 0xd8, 0xd8, 0xd9, 0xd9, 0xda, 0xda,
];

pub(super) const ANCILLA_DRAW_SHADOW_ANCILLA_DRAW_SHADOW_FLAGS: [u8; 14] = [
    0x28, 0x68, 0x28, 0x68, 0x28, 0xff, 0x22, 0x22, 0x24, 0x64, 0x24, 0x64, 0x24, 0x64,
];

pub(super) const ANCILLA_CHECK_DAMAGE_TO_SPRITE_AGGRESSIVE_ANCILLA_DAMAGE: [u8; 57] = [
    6, 1, 11, 0, 0, 0, 0, 8, 0, 6, 0, 12, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 14, 13, 0, 0, 15, 0,
    0, 7, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 11, 0, 1, 1, 1, 1, 1, 1, 1, 1,
];

pub(super) const SPRITE_APPLY_CALCULATED_DAMAGE_FOR_ANCILLA_ENEMY_DAMAGES: [u8; 128] = [
    0, 1, 32, 255, 252, 251, 0, 0, 0, 2, 64, 4, 0, 0, 0, 0, 0, 4, 64, 2, 3, 0, 0, 0, 0, 8, 64, 4,
    0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 16, 64, 8, 0, 0, 0, 0, 0, 4, 64, 16, 0, 0, 0, 0, 0,
    255, 64, 255, 252, 251, 0, 0, 0, 4, 64, 255, 252, 251, 32, 0, 0, 100, 24, 100, 0, 0, 0, 0, 0,
    249, 250, 255, 100, 0, 0, 0, 0, 8, 64, 253, 4, 16, 0, 0, 0, 8, 64, 254, 4, 0, 0, 0, 0, 16, 64,
    253, 0, 0, 0, 0, 0, 254, 64, 16, 0, 0, 0, 0, 0, 32, 64, 255, 0, 0, 0, 250,
];

pub(super) const ANCILLA_DRAW_EXPLOSION_BOMB_DRAW_EXPLOSION_OFFSET: [SignedOffset; 54] = signed_offsets![
    -8, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -8, -8, -8, 0, 0, -8, 0, 0, 0, 0, 0, 0, -16, -16, -16, 0,
    0, -16, 0, 0, 0, 0, 0, 0, -16, -16, -16, 0, 0, -16, 0, 0, 0, 0, 0, 0, -8, -8, -21, -22, -21, 8,
    9, -22, 9, 8, 0, 0, -6, -15, 0, -1, -16, -2, -8, -7, 0, 0, 0, 0, -9, -4, -21, -5, -12, -18,
    -11, 7, 0, -15, 4, -2, -9, -4, -22, -5, -13, -20, -11, 8, 1, -16, 5, -2, -20, 4, -12, -19, -9,
    16, -5, -2, 2, -9, 10, 6,
];

pub(super) const ANCILLA_DRAW_EXPLOSION_BOMB_DRAW_EXPLOSION_TILE: [OamTileAttrs; 54] = oam_tile_attrs![
    0x6e, 0x26, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x8c, 0x22, 0x8c, 0x62,
    0x8c, 0xa2, 0x8c, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x84, 0x22, 0x84, 0x62, 0x84, 0xa2, 0x84, 0xe2,
    0xff, 0xff, 0xff, 0xff, 0x88, 0x22, 0x88, 0x62, 0x88, 0xa2, 0x88, 0xe2, 0xff, 0xff, 0xff, 0xff,
    0x86, 0x22, 0x88, 0x22, 0x88, 0x62, 0x88, 0xa2, 0x88, 0xe2, 0xff, 0xff, 0x86, 0x22, 0x86, 0x62,
    0x86, 0xe2, 0x86, 0xe2, 0xff, 0xff, 0xff, 0xff, 0x86, 0xe2, 0x86, 0x22, 0x86, 0x22, 0x86, 0x62,
    0x86, 0xa2, 0x86, 0xa2, 0x8a, 0xa2, 0x8a, 0x62, 0x8a, 0x22, 0x8a, 0x62, 0x8a, 0x62, 0x8a, 0xe2,
    0x9b, 0x22, 0x9b, 0xa2, 0x9b, 0x62, 0x9b, 0xe2, 0x9b, 0xa2, 0x9b, 0x22,
];

pub(super) const ANCILLA_DRAW_EXPLOSION_BOMB_DRAW_EXPLOSION_EXT: [u8; 54] = [
    2, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 1, 2, 2,
    2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0,
];

pub(super) const ANCILLA32_BLAST_WALL_FIREBALL_BLAST_WALL_FIREBALL_CHAR: [u8; 3] =
    [0x9d, 0x9c, 0x8d];
