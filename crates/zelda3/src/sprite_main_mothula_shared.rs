use super::sprite::DrawMultipleData;
pub(super) const ARMOS_KNIGHT_REMAINING_COUNT: usize = 0x0ff8;
pub(super) const FEATURE_MISC_BUG_FIXES_MOTHULA: u32 = 4096;

// kMothula_Dmd from sprite_main.c:13776 — packed as (x:i8, y:i8, char:u16, big:u8).
pub(super) type MothulaDrawFrame = (i8, i8, u16, u8);
pub(super) const MOTHULA_DRAW_FRAMES: [MothulaDrawFrame; 24] = [
    (-24, -8, 0x0080, 2),
    (-8, -8, 0x0082, 2),
    (8, -8, 0x4082, 2),
    (24, -8, 0x4080, 2),
    (-24, 8, 0x00a0, 2),
    (-8, 8, 0x00a2, 2),
    (8, 8, 0x40a2, 2),
    (24, 8, 0x40a0, 2),
    (-24, -8, 0x0084, 2),
    (-8, -8, 0x0086, 2),
    (8, -8, 0x4086, 2),
    (24, -8, 0x4084, 2),
    (-24, 8, 0x00a4, 2),
    (-8, 8, 0x00a6, 2),
    (8, 8, 0x40a6, 2),
    (24, 8, 0x40a4, 2),
    (-8, -8, 0x0088, 2),
    (-8, -8, 0x0088, 2),
    (8, -8, 0x4088, 2),
    (8, -8, 0x4088, 2),
    (-8, 8, 0x00a8, 2),
    (-8, 8, 0x00a8, 2),
    (8, 8, 0x40a8, 2),
    (8, 8, 0x40a8, 2),
];

// kMothula_Draw_X from sprite_main.c:13809.
pub(super) const MOTHULA_DRAW_X_OFFSETS: [i8; 27] = [
    0, 3, 6, 9, 12, -3, -6, -9, -12, 0, 2, 4, 6, 8, -2, -4, -6, -8, 0, 1, 2, 3, 4, -1, -2, -3, -4,
];

// kMothula_FlapWingsGfx from sprite_main.c:22592.
pub(super) const MOTHULA_WING_FLAP_GRAPHICS: [u8; 4] = [0, 1, 2, 1];

// kMothula_XYvel from sprite_main.c:22562.
pub(super) const MOTHULA_AXIS_VELOCITIES: [i8; 10] = [-16, -12, 0, 12, 16, 12, 0, -12, -16, -12];

// kMothula_Beam_Xvel / Yvel from sprite_main.c:22600-22601.
pub(super) const MOTHULA_BEAM_X_VELOCITIES: [i8; 3] = [-16, 0, 16];
pub(super) const MOTHULA_BEAM_Y_VELOCITIES: [i8; 3] = [24, 32, 24];

// kMothula_Spike_XLo / YLo / Dir from sprite_main.c:22621-22632.
pub(super) const MOTHULA_SPIKE_X_LOW: [u8; 30] = [
    0x38, 0x48, 0x58, 0x68, 0x88, 0x98, 0xa8, 0xb8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xb8,
    0xa8, 0x98, 0x78, 0x68, 0x58, 0x48, 0x38, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28,
];
pub(super) const MOTHULA_SPIKE_Y_LOW: [u8; 30] = [
    0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x48, 0x58, 0x68, 0x78, 0x98, 0xa8, 0xb8, 0xc8,
    0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xb8, 0xa8, 0x98, 0x78, 0x68, 0x58, 0x48,
];
pub(super) const MOTHULA_SPIKE_DIRECTIONS: [u8; 30] = [
    2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0,
];
pub(super) const GIBDO_DIRECTION_TARGETS: [u8; 4] = [2, 6, 4, 0];
pub(super) const GIBDO_GRAPHICS: [u8; 8] = [4, 8, 11, 10, 0, 6, 3, 7];
pub(super) const GIBDO_AXIS_VELOCITIES: [i8; 10] = [-16, 0, 0, 0, 16, 0, 0, 0, -16, 0];
pub(super) const GIBDO_ALT_GRAPHICS: [u8; 8] = [9, 2, 0, 4, 11, 3, 1, 5];
pub(super) const PIROGUSU_DIRECTION_LOOKUP: [u8; 4] = [2, 3, 0, 1];
pub(super) const PIROGUSU_GRAPHICS_LOOKUP: [u8; 8] = [9, 11, 5, 7, 5, 11, 7, 9];
pub(super) const PIROGUSU_ANIMATION_STATE_LOOKUP: [u8; 8] = [16, 17, 18, 19, 12, 13, 14, 15];
pub(super) const PIROGUSU_AXIS_VELOCITIES: [i8; 6] = [0, 0, 4, -4, 0, 0];
pub(super) const PIROGUSU_COLLISION_AXIS_VELOCITIES: [i8; 6] = [2, -2, 0, 0, 2, -2];
pub(super) const PIROGUSU_FAST_AXIS_VELOCITIES: [i8; 6] = [24, -24, 0, 0, 24, -24];
pub(super) const PIROGUSU_DIRECTIONS: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
pub(super) const LASER_EYE_DIRECTIONS: [u8; 4] = [2, 3, 0, 1];
pub(super) const STALFOS_KNIGHT_CASE2_GRAPHICS: [u8; 2] = [0, 1];
pub(super) const STALFOS_KNIGHT_CASE2_DIRECTIONS: [u8; 16] =
    [0, 0, 0, 2, 1, 1, 1, 2, 0, 0, 0, 2, 1, 1, 1, 2];
pub(super) const STALFOS_KNIGHT_CASE6_STATES: [u8; 32] = [
    0, 4, 8, 12, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14,
    14, 14, 15, 14, 12, 8, 4, 0,
];
pub(super) const STALFOS_KNIGHT_CASE7_GRAPHICS: [u8; 2] = [1, 4];
pub(super) const TERRORPIN_X_VELOCITIES: [i8; 8] = [8, -8, 0, 0, 12, -12, 0, 0];
pub(super) const TERRORPIN_Y_VELOCITIES: [i8; 8] = [0, 0, 8, -8, 0, 0, 12, -12];
pub(super) const TERRORPIN_OAM_FLAGS: [u8; 2] = [0, 0x40];
pub(super) const TERRORPIN_OVERTURNED_X_VELOCITIES: [i8; 2] = [8, -8];
pub(super) const ARRGHUS_GRAPHICS_BY_STEP: [u8; 9] = [1, 1, 1, 2, 2, 1, 1, 0, 0];
pub(super) const ARRGHI_GRAPHICS_BY_STEP: [u8; 8] = [0, 1, 2, 2, 2, 2, 2, 1];
pub(super) const BABUSU_GRAPHICS: [u8; 6] = [5, 4, 3, 2, 1, 0];
pub(super) const BABUSU_DIRECTION_GRAPHICS: [u8; 4] = [6, 6, 0, 0];
pub(super) const BABUSU_AXIS_VELOCITIES: [i8; 6] = [32, -32, 0, 0, 32, -32];
pub(super) const BABUSU_SCURRY_GRAPHICS: [u8; 4] = [18, 14, 12, 16];
pub(super) const WIZZROBE_CLOAK_GRAPHICS: [u8; 4] = [4, 2, 0, 6];
pub(super) const WIZZROBE_ATTACK_GRAPHICS: [u8; 8] = [0, 1, 1, 1, 1, 1, 1, 0];
pub(super) const WIZZROBE_ATTACK_DIRECTION_GRAPHICS: [u8; 4] = [4, 2, 0, 6];
pub(super) const KYAMERON_COAGULATE_GRAPHICS: [u8; 8] = [4, 7, 14, 13, 12, 6, 6, 5];
pub(super) const KYAMERON_X_VELOCITIES: [i8; 4] = [32, -32, 32, -32];
pub(super) const KYAMERON_Y_VELOCITIES: [i8; 4] = [32, 32, -32, -32];
pub(super) const KYAMERON_MOVING_GRAPHICS: [u8; 4] = [3, 2, 1, 0];
pub(super) const PENGATOR_GRAPHICS_BY_DIRECTION: [u8; 4] = [5, 0, 10, 15];
pub(super) const PENGATOR_AXIS_VELOCITIES: [i8; 6] = [1, -1, 0, 0, 1, -1];
pub(super) const PENGATOR_JUMP_GRAPHICS: [u8; 4] = [4, 4, 3, 2];
pub(super) const PENGATOR_GARNISH_Y_OFFSETS: [i8; 8] = [8, 10, 12, 14, 12, 12, 12, 12];
pub(super) const PENGATOR_GARNISH_X_OFFSETS: [i8; 8] = [4, 4, 4, 4, 0, 4, 8, 12];
pub(super) const FLUTE_BOY_ANIMAL_X_VELOCITIES: [i8; 4] = [16, -16, 0, 0];
pub(super) const ZAZAK_Y_VELOCITIES: [i8; 4] = [0, 0, 16, -16];
pub(super) const GORIYA_X_VELOCITIES: [i8; 32] = [
    0, 16, -16, 0, 0, 13, -13, 0, 0, 13, -13, 0, 0, 0, 0, 0, 0, -24, 24, 0, 0, -16, 16, 0, 0, -16,
    16, 0, 0, 0, 0, 0,
];
pub(super) const GORIYA_Y_VELOCITIES: [i8; 32] = [
    0, 0, 0, 0, -16, -5, -5, 0, 16, 13, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24, -16, -16, 0, 24, 16,
    16, 0, 0, 0, 0, 0,
];
pub(super) const GORIYA_DIRECTIONS: [u8; 32] = [
    0, 0, 1, 0, 3, 3, 3, 0, 2, 2, 2, 0, 0, 0, 0, 0, 0, 1, 0, 0, 3, 3, 3, 0, 2, 2, 2, 0, 0, 0, 0, 0,
];
pub(super) const GORIYA_GRAPHICS: [u8; 16] = [8, 6, 0, 3, 9, 7, 1, 4, 8, 6, 0, 3, 9, 7, 2, 5];
pub(super) const EYEGORE_CLOSING_GRAPHICS: [u8; 8] = [0, 0, 1, 1, 2, 2, 2, 2];
pub(super) const EYEGORE_OPENING_GRAPHICS: [u8; 8] = [2, 2, 2, 2, 1, 1, 0, 0];
pub(super) const EYEGORE_CHASING_GRAPHICS: [u8; 16] =
    [7, 5, 2, 9, 8, 6, 3, 10, 7, 5, 2, 9, 8, 6, 4, 11];
pub(super) const EYEGORE_OPENING_DELAYS: [u8; 4] = [0x60, 0x80, 0xa0, 0x80];
pub(super) const ARMOS_KNIGHT_WAKE_GRAPHICS: [u8; 5] = [5, 4, 3, 2, 1];
pub(super) const ARMOS_KNIGHT_CHARGE_X_VELOCITIES: [i8; 2] = [16, -16];
pub(super) const FLUTE_BOY_ANIMAL_OAM_FLAGS: [u8; 2] = [0x40, 0];
pub(super) const FLUTE_BOY_ANIMAL_GRAPHICS: [u8; 3] = [0, 1, 2];
pub(super) const FLUTE_BOY_OSTRICH_GRAPHICS: [u8; 4] = [0, 1, 0, 2];
pub(super) const FLUTE_BOY_BIRD_X_OFFSETS: [i8; 2] = [8, 0];
pub(super) const FREEZOR_X_VELOCITIES: [i8; 4] = [8, -8, 0, 0];
pub(super) const FREEZOR_Y_VELOCITIES: [i8; 4] = [0, 0, 18, -18];
pub(super) const FREEZOR_MOVING_GRAPHICS: [u8; 4] = [1, 2, 1, 3];
pub(super) const FREEZOR_SPARKLE_X_OFFSETS: [i8; 8] = [-4, -2, 0, 2, 4, 6, 8, 10];
pub(super) const FREEZOR_MELTING_GRAPHICS: [u8; 4] = [6, 5, 4, 7];
pub(super) const KODONDO_X_VELOCITIES: [i8; 4] = [1, -1, 0, 0];
pub(super) const KODONDO_Y_VELOCITIES: [i8; 4] = [0, 0, 1, -1];
pub(super) const KODONDO_GRAPHICS: [u8; 8] = [2, 2, 0, 5, 3, 3, 0, 5];
pub(super) const KODONDO_OAM_FLAGS: [u8; 8] = [0x40, 0, 0, 0, 0x40, 0, 0x40, 0x40];
pub(super) const KODONDO_FLAME_GRAPHICS: [u8; 8] = [2, 2, 0, 5, 4, 4, 1, 6];
pub(super) const KHOLDSTARE_TARGET_X_VELOCITIES: [i8; 4] = [16, 16, -16, -16];
pub(super) const KHOLDSTARE_TARGET_Y_VELOCITIES: [i8; 4] = [-16, 16, 16, -16];
pub(super) const KHOLDSTARE_SHELL_FRAGMENT_Z_VELOCITIES: [i8; 3] = [32, -32, 0];
pub(super) const KHOLDSTARE_SHELL_FRAGMENT_Z_SUBPIXELS: [i8; 3] = [-32, -32, 48];
pub(super) const BOMBER_GRAPHICS_BY_DIRECTION: [u8; 4] = [9, 10, 8, 7];
pub(super) const BOMBER_X_VELOCITIES: [i8; 8] = [16, 12, 0, -12, -16, -12, 0, 12];
pub(super) const BOMBER_Y_VELOCITIES: [i8; 8] = [0, 12, 16, 12, 0, -12, -16, -12];
pub(super) const BOMBER_SHOT_GRAPHICS_BY_DIRECTION: [u8; 4] = [0, 4, 2, 6];
pub(super) const BOMBER_PELLET_X_OFFSETS: [i8; 4] = [14, -6, 4, 4];
pub(super) const BOMBER_PELLET_Y_OFFSETS: [i8; 4] = [7, 7, 12, -4];
pub(super) const PIKIT_GRAPHICS: [u8; 24] = [
    2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2,
];
pub(super) const PIKIT_TONGUE_XY_OFFSETS: [i8; 72] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    12, 16, 24, 32, 32, 24, 16, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -12, -16, -24,
    -32, -32, -24, -16, -12, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub(super) const PIKIT_TONGUE_X_OFFSET_BASES: [u8; 8] = [24, 24, 0, 48, 48, 48, 0, 24];
pub(super) const PIKIT_TONGUE_Y_OFFSET_BASES: [u8; 8] = [0, 24, 24, 24, 0, 48, 48, 48];
pub(super) const STALFOS_DIRECTION_ANIMATION_STATES: [u8; 4] = [8, 9, 10, 11];
pub(super) const STALFOS_CHECK_DIRECTIONS: [u8; 4] = [3, 2, 1, 0];
pub(super) const STALFOS_PRIMARY_ANIMATION_STATES: [u8; 8] = [6, 4, 0, 2, 7, 5, 1, 3];
pub(super) const STALFOS_HOP_DELAYS: [u8; 4] = [16, 32, 64, 32];
pub(super) const ZAZAK_ALT_DIRECTIONS: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
pub(super) const FIREBALL_JUNCTION_X_OFFSETS: [i8; 4] = [12, -12, 0, 0];
pub(super) const FIREBALL_JUNCTION_Y_OFFSETS: [i8; 4] = [0, 0, 12, -12];
pub(super) const FIREBALL_JUNCTION_AXIS_VELOCITIES: [i8; 6] = [0, 0, 40, -40, 0, 0];
pub(super) const GIBO_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];
pub(super) const GIBO_X_VELOCITIES: [i8; 8] = [16, 16, 0, -16, -16, -16, 0, 16];
pub(super) const GIBO_Y_VELOCITIES: [i8; 8] = [0, 0, 16, -16, 16, 16, -16, -16];
pub(super) const TEKITE_DIRECTIONS: [u8; 4] = [3, 2, 1, 0];
pub(super) const TEKITE_X_VELOCITIES: [i8; 4] = [16, -16, 16, -16];
pub(super) const TEKITE_Y_VELOCITIES: [i8; 4] = [16, 16, -16, -16];
pub(super) const HOVER_OAM_FLAGS: [u8; 4] = [0x40, 0, 0x40, 0];
pub(super) const HOVER_PRIMARY_X_ACCELERATIONS: [i8; 4] = [1, -1, 1, -1];
pub(super) const HOVER_PRIMARY_Y_ACCELERATIONS: [i8; 4] = [1, 1, -1, -1];
pub(super) const HOVER_SECONDARY_X_ACCELERATIONS: [i8; 4] = [-1, 1, -1, 1];
pub(super) const HOVER_SECONDARY_Y_ACCELERATIONS: [i8; 4] = [-1, -1, 1, 1];
pub(super) const CHAIN_CHOMP_X_VELOCITIES: [i8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
];
pub(super) const CHAIN_CHOMP_Y_VELOCITIES: [i8; 16] = [
    -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
];
pub(super) const HOKBOK_SEGMENT_STATES: [u8; 8] = [8, 7, 6, 5, 4, 5, 6, 7];
pub(super) const BOULDER_Z_VELOCITIES: [i8; 2] = [32, 48];
pub(super) const BOULDER_Y_VELOCITIES: [i8; 2] = [8, 32];
pub(super) const BOULDER_X_VELOCITIES: [i8; 4] = [24, 16, -24, -16];
pub(super) const MAD_BATTER_BOLT_X_OFFSETS: [u16; 8] = [0, 4, 8, 12, 12, 4, 8, 0];
pub(super) const MAD_BATTER_BOLT_Y_OFFSETS: [u16; 8] = [0, 4, 8, 12, 12, 4, 8, 0];
pub(super) const FLOPPING_FISH_X_VELOCITIES: [i8; 8] = [0, 12, 16, 12, 0, -12, -16, -12];
pub(super) const FLOPPING_FISH_Y_VELOCITIES: [i8; 8] = [-16, -12, 0, 12, 16, 12, 0, -12];
pub(super) const FLOPPING_FISH_A_TARGET_BY_DIRECTION: [u8; 2] = [2, 0];
pub(super) const FLOPPING_FISH_AIR_GRAPHICS: [u8; 3] = [1, 5, 3];
pub(super) const FLOPPING_FISH_GROUND_GRAPHICS: [u8; 17] =
    [5, 5, 6, 6, 5, 5, 4, 4, 3, 7, 7, 8, 8, 7, 7, 8, 8];
pub(super) const LARGE_WATER_TURBULENCE_DRAW_DATA: [DrawMultipleData; 6] = [
    DrawMultipleData {
        x: -10,
        y: 14,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 16,
        char_flags: 0x40c0,
        ext: 2,
    },
    DrawMultipleData {
        x: -2,
        y: 18,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 2,
        y: 18,
        char_flags: 0x40c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 16,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 10,
        y: 14,
        char_flags: 0x40c0,
        ext: 2,
    },
];
pub(super) const ARRGHUS_DRAW_DATA: [DrawMultipleData; 5] = [
    DrawMultipleData {
        x: -8,
        y: -4,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -4,
        char_flags: 0x4080,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 12,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 12,
        char_flags: 0x40a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 24,
        char_flags: 0x00a8,
        ext: 2,
    },
];
pub(super) const BLOB_POPPING_OUT_GRAPHICS: [u8; 16] =
    [0, 1, 7, 7, 6, 6, 5, 5, 6, 6, 5, 5, 4, 4, 4, 4];
pub(super) const BLOB_FALLING_X_VELOCITIES: [i8; 2] = [-8, 8];
pub(super) const BLOB_FALLING_GRAPHICS: [u8; 2] = [0, 1];
pub(super) const SPIKE_BLOCK_ATTACK_X_VELOCITY_TARGETS: [i8; 4] = [32, -32, 0, 0];
pub(super) const SPIKE_BLOCK_ATTACK_Y_VELOCITY_TARGETS: [i8; 4] = [0, 0, 32, -32];
pub(super) const SPIKE_BLOCK_ATTACK_X_VELOCITY_DELTAS: [i8; 4] = [1, -1, 0, 0];
pub(super) const SPIKE_BLOCK_ATTACK_Y_VELOCITY_DELTAS: [i8; 4] = [0, 0, 1, -1];
pub(super) const SPIKE_BLOCK_RETURN_X_VELOCITIES: [i8; 4] = [-16, 16, 0, 0];
pub(super) const SPIKE_BLOCK_RETURN_Y_VELOCITIES: [i8; 4] = [0, 0, -16, 16];
pub(super) const BIG_SPIKE_ATTACK_X_VELOCITIES: [i8; 4] = [32, -32, 0, 0];
pub(super) const BIG_SPIKE_RETURN_X_VELOCITIES: [i8; 4] = [-16, 16, 0, 0];
pub(super) const BIG_SPIKE_ATTACK_Y_VELOCITIES: [i8; 4] = [0, 0, 32, -32];
pub(super) const BIG_SPIKE_RETURN_Y_VELOCITIES: [i8; 4] = [0, 0, -16, 16];
pub(super) const BIG_SPIKE_ATTACK_DELAYS: [u8; 4] = [0x40, 0x40, 0x38, 0x38];
