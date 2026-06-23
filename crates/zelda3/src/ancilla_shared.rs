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
