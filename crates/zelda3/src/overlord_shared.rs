//! Shared constants and tables for overlord.rs helpers.

// ---------------------------------------------------------------------------
// File-level overlord RAM offsets and shared lookup tables.
// ---------------------------------------------------------------------------
pub(super) const OVERLORD_X_HI: usize = 0x0b10;
pub(super) const OVERLORD_Y_LO: usize = 0x0b18;
pub(super) const OVERLORD_Y_HI: usize = 0x0b20;
pub(super) const OVERLORD_GEN1: usize = 0x0b28;
pub(super) const OVERLORD_GEN2: usize = 0x0b30;
pub(super) const OVERLORD_GEN3: usize = 0x0b38;
pub(super) const OVERLORD_FLOOR: usize = 0x0b40;
pub(super) const OVERLORD_OFFSET_SPRITE_POS: usize = 0x0b48;
pub(super) const SPRITE_BUMP_DAMAGE: usize = 0x0cd2;
pub(super) const ACTIVATE_BOMB_TRAP_OVERLORD: usize = 0x0cf4;
pub(super) const SPRITE_AI_STATE: usize = 0x0d80;
pub(super) const SPRITE_B: usize = 0x0da0;
pub(super) const SPRITE_C: usize = 0x0db0;
pub(super) const SPRITE_STATE: usize = 0x0dd0;
pub(super) const SPRITE_DELAY_AUX1: usize = 0x0e00;
pub(super) const SPRITE_DELAY_AUX2: usize = 0x0e10;
pub(super) const SPRITE_HEALTH: usize = 0x0e50;
pub(super) const GARNISH_ACTIVE: usize = 0x0fb4;
pub(super) const SPRITE_TILETYPE: usize = 0x0fa5;
pub(super) const SPRCOLL_Y_BASE: usize = 0x0fbe;
pub(super) const ACTIVE_OVERLORD_INDEX: usize = 0x0fde;
pub(super) const DUNG_FLOOR_MOVE_FLAGS: usize = 0x041a;
pub(super) const GARNISH_Y_LO: usize = 0x1f81e;
pub(super) const GARNISH_X_LO: usize = 0x1f83c;
pub(super) const GARNISH_Y_HI: usize = 0x1f85a;
pub(super) const GARNISH_X_HI: usize = 0x1f878;
pub(super) const GARNISH_COUNTDOWN: usize = 0x1f90e;
pub(super) const ARMOS_SINE_LOOKUP_TABLE: [u16; 256] = [
    0, 3, 6, 9, 12, 15, 18, 21, 25, 28, 31, 34, 37, 40, 40, 46, 49, 53, 56, 59, 62, 65, 68, 71, 74,
    77, 80, 83, 86, 89, 92, 95, 97, 100, 103, 106, 109, 112, 115, 117, 120, 123, 126, 128, 131,
    134, 136, 139, 142, 144, 147, 149, 152, 155, 157, 159, 162, 164, 167, 169, 171, 174, 176, 178,
    181, 183, 185, 187, 189, 191, 193, 195, 197, 199, 201, 203, 205, 207, 209, 211, 212, 214, 216,
    217, 219, 221, 222, 224, 225, 227, 228, 230, 231, 232, 234, 235, 236, 237, 238, 239, 241, 242,
    243, 244, 244, 245, 246, 247, 248, 249, 249, 250, 251, 251, 252, 252, 253, 253, 254, 254, 254,
    255, 255, 255, 255, 255, 255, 255, 256, 255, 255, 255, 255, 255, 255, 255, 254, 254, 254, 253,
    253, 252, 252, 251, 251, 250, 249, 249, 248, 247, 246, 245, 244, 244, 243, 242, 241, 239, 238,
    237, 236, 235, 234, 232, 231, 230, 228, 227, 225, 224, 222, 221, 219, 217, 216, 214, 212, 211,
    209, 207, 205, 203, 201, 199, 197, 195, 193, 191, 189, 187, 185, 183, 181, 178, 176, 174, 171,
    169, 167, 164, 162, 159, 157, 155, 152, 149, 147, 144, 142, 139, 136, 134, 131, 128, 126, 123,
    120, 117, 115, 112, 109, 106, 103, 100, 97, 95, 92, 89, 86, 83, 80, 77, 74, 71, 68, 65, 62, 59,
    56, 53, 49, 46, 43, 40, 37, 34, 31, 28, 25, 21, 18, 15, 12, 9, 6, 3,
];

// ---------------------------------------------------------------------------
// Promoted overlord method-local tables. Names retain the owning helper so
// generic C table names stay readable at callsites.
// ---------------------------------------------------------------------------
pub(super) const ARMOS_COORDINATOR_BACK_WALL_X_OFFSETS: [u8; 6] = [49, 77, 105, 131, 159, 187];
pub(super) const INVISIBLE_STALFOS_TRAP_X_OFFSETS: [i8; 4] = [0, 0, -48, 48];
pub(super) const INVISIBLE_STALFOS_TRAP_Y_OFFSETS: [i8; 4] = [-40, 56, 8, 8];
pub(super) const INVISIBLE_STALFOS_TRAP_DELAYS: [u8; 4] = [0x30, 0x50, 0x70, 0x90];
pub(super) const ZORO_SPAWNER_X_OFFSETS: [i8; 8] = [-4, -2, 0, 2, 4, 6, 8, 12];
pub(super) const WIZZROBE_SPAWNER_X_OFFSETS: [i8; 4] = [48, -48, 0, 0];
pub(super) const WIZZROBE_SPAWNER_Y_OFFSETS: [i8; 4] = [16, 16, 64, -32];
pub(super) const WIZZROBE_SPAWNER_DELAYS: [u8; 4] = [0, 16, 32, 48];
pub(super) const TILE_ROOM_FLYING_TILE_X_POSITIONS: [u8; 22] = [
    0x70, 0x80, 0x60, 0x90, 0x90, 0x60, 0x70, 0x80, 0x80, 0x70, 0x50, 0xa0, 0xa0, 0x50, 0x50, 0xa0,
    0xa0, 0x50, 0x70, 0x80, 0x80, 0x70,
];
pub(super) const TILE_ROOM_FLYING_TILE_Y_POSITIONS: [u8; 22] = [
    0x80, 0x80, 0x70, 0x90, 0x70, 0x90, 0x60, 0xa0, 0x60, 0xa0, 0x60, 0xb0, 0x60, 0xb0, 0x80, 0x90,
    0x80, 0x90, 0x70, 0x90, 0x70, 0x90,
];
pub(super) const PIROGUSU_SPAWNER_DIRECTIONS: [u8; 4] = [2, 3, 0, 1];
pub(super) const FALLING_SQUARE_CRUMBLE_PATH_DIRECTIONS: [u8; 109] = [
    2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 3, 1, 3, 0, 3, 1,
    3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1,
    3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0xff,
];
pub(super) const FALLING_SQUARE_CRUMBLE_PATH_OFFSETS: [u8; 7] = [0, 25, 66, 77, 87, 98, 108];
pub(super) const FALLING_SQUARE_CRUMBLE_X_DELTAS: [i8; 4] = [16, -16, 0, 0];
pub(super) const FALLING_SQUARE_CRUMBLE_Y_DELTAS: [i8; 4] = [0, 0, 16, -16];
pub(super) const BLOB_SPAWNER_ZOL_X_OFFSETS: [i8; 4] = [0, 0, -48, 48];
pub(super) const BLOB_SPAWNER_ZOL_Y_OFFSETS: [i8; 4] = [-40, 56, 8, 8];
pub(super) const FALLING_STALFOS_TRAP_TRIGGER_TIMERS: [u8; 8] =
    [255, 224, 192, 160, 128, 96, 64, 32];
pub(super) const SNAKE_TRAP_SPAWN_TIMERS_BY_SLOT: [u8; 8] =
    [0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90];
pub(super) const FULL_ROOM_CANNON_BALL_DIRECTIONS: [u8; 16] =
    [2, 2, 2, 2, 1, 1, 1, 1, 3, 3, 3, 3, 0, 0, 0, 0];
pub(super) const FULL_ROOM_CANNON_BALL_X_POSITIONS: [u8; 16] = [
    64, 96, 144, 176, 240, 240, 240, 240, 176, 144, 96, 64, 0, 0, 0, 0,
];
pub(super) const FULL_ROOM_CANNON_BALL_Y_POSITIONS: [u8; 16] = [
    16, 16, 16, 16, 64, 96, 160, 192, 240, 240, 240, 240, 192, 160, 96, 64,
];
pub(super) const CANNON_BALL_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];
pub(super) const CANNON_BALL_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];
pub(super) const OVERLORD_ACTIVE_RANGE_OFFSETS: [u16; 2] = [0x0130, (-0x40i16) as u16];
pub(super) const ARMOS_KNIGHT_RING_ANGLE_OFFSETS: [u16; 6] = [0, 425, 340, 255, 170, 85];
