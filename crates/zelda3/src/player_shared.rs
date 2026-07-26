//! Shared player tables promoted out of method bodies.
//!
//! Owner-prefixed names keep generic C table names readable at callsites while
//! preserving the original table values exactly.

pub(super) const LINK_INITIALIZE_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const LINK_RESET_PROPERTIES_C_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const LINK_SPLASH_UPON_LANDING_PLAYER_HANDLER_STATE_RECOIL_OTHER: u8 = 6;

pub(super) const LINK_HANDLE_SWIM_ACCELS_SWIM_ACCELERATION_TARGETS: [u16; 9] =
    [128, 160, 192, 224, 256, 288, 320, 352, 384];

pub(super) const LINK_SET_MOMENTUM_SWIM_STROKE_TIMERS_BY_MOVING_FLAG: [u8; 2] = [32, 8];

pub(super) const PLAYER_HANDLER_04_SWIMMING_ACTIVE_SWIM_ANIMATION_DELAYS: [u8; 4] = [2, 0, 1, 0];

pub(super) const LINK_APPLY_TILE_REBOUND_DASH_REBOUND_Y_VELOCITY_BY_DIRECTION: [u8; 4] =
    [24, (-24i8) as u8, 0, 0];

pub(super) const LINK_APPLY_TILE_REBOUND_DASH_REBOUND_X_VELOCITY_BY_DIRECTION: [u8; 4] =
    [0, 0, 24, (-24i8) as u8];

pub(super) const LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_Y_DIRECTIONS: [u8; 4] = [1, 0, 0, 0];

pub(super) const LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_X_DIRECTIONS: [u8; 4] = [0, 0, 1, 0];

pub(super) const LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_Y_ACCELERATIONS: [u16; 8] =
    [384, 384, 0, 0, 256, 256, 0, 0];

pub(super) const LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_X_ACCELERATIONS: [u16; 8] =
    [0, 0, 384, 384, 0, 0, 256, 256];

pub(super) const LINK_APPLY_TILE_REBOUND_DIRECTION_BITS_BY_FACING: [u8; 4] = [8, 4, 2, 1];

pub(super) const LINK_HANDLE_MOVING_ANIMATION_FULL_LONG_ENTRY_FACING_DIRECTION_BITS: [u8; 4] =
    [8, 4, 2, 1];

pub(super) const LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_POSE_ANIMATION_DELAYS: [u8; 16] =
    [4, 4, 4, 4, 1, 1, 1, 1, 2, 2, 2, 2, 8, 8, 8, 8];

pub(super) const LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_WALK_ANIMATION_DELAYS: [u8; 24] = [
    1, 2, 3, 2, 2, 2, 3, 2, 1, 1, 2, 1, 1, 1, 2, 1, 2, 2, 3, 2, 2, 2, 3, 2,
];

pub(super) const LINK_HANDLE_MOVING_ANIMATION_SWIMMING_FACING_DIRECTION_BITS: [u8; 4] =
    [8, 4, 2, 1];

pub(super) const LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_ANIM_THRESHOLDS: [u8; 7] =
    [48, 36, 24, 16, 12, 8, 4];

pub(super) const LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_ANIM_DELAYS: [u8; 56] = [
    3, 3, 5, 3, 3, 3, 5, 3, 2, 2, 4, 2, 2, 2, 4, 2, 2, 2, 3, 2, 2, 2, 3, 2, 1, 1, 2, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(super) const LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_MIN_ANIM_STEPS: [u8; 7] =
    [1, 2, 2, 2, 2, 2, 2];

pub(super) const LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG: [u8; 4] = [8, 4, 2, 1];

pub(super) const LINK_APPLY_CONVEYOR_MOVING_BELT_Y: [i8; 4] = [-8, 8, 0, 0];

pub(super) const LINK_APPLY_CONVEYOR_MOVING_BELT_X: [i8; 4] = [0, 0, -8, 8];

pub(super) const LINK_HOP_IN_OR_OUT_OF_WATER_Y_RECOIL_VEL_Y: [u8; 3] = [24, 16, 16];

pub(super) const LINK_HOP_IN_OR_OUT_OF_WATER_Y_RECOIL_VEL_Z: [u8; 3] = [36, 24, 24];

pub(super) const LINK_HOP_IN_OR_OUT_OF_WATER_X_RECOIL_VEL_X: [u8; 3] = [28, 24, 16];

pub(super) const LINK_HOP_IN_OR_OUT_OF_WATER_X_RECOIL_VEL_Z: [u8; 3] = [32, 24, 24];

pub(super) const FLAG_MOVING_INTO_SLOPES_Y_AVOID_JUDDER: [i8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7,
];

pub(super) const FLAG_MOVING_INTO_SLOPES_X_AVOID_JUDDER: [i8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7,
];

pub(super) const PLAYER_LIMIT_DIRECTIONS_INNER_MASKS: [u8; 4] = [0x07, 0x0b, 0x0d, 0x0e];

pub(super) const LINK_HANDLE_VELOCITY_SPEED_MOD: [u8; 27] = [
    24, 16, 10, 24, 16, 8, 8, 4, 12, 16, 9, 25, 20, 13, 16, 8, 64, 42, 16, 8, 4, 2, 48, 24, 32, 21,
    0,
];

pub(super) const HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_ACCELERATION_DELTAS: [i8; 12] =
    [8, -12, -8, -16, 4, -6, -12, -6, 10, -16, -12, -6];

pub(super) const HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_AXIS_DIRECTION_CLEAR_MASKS: [u8; 2] =
    [!0x0c, !0x03];

pub(super) const HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_DIRECTION_BITS_BY_AXIS: [u8; 4] =
    [8, 4, 2, 1];

pub(super) const HANDLE_NUDGING_NUDGE_PROBE_Y0_OFFSETS: [u8; 8] = [8, 8, 23, 23, 8, 23, 8, 23];

pub(super) const HANDLE_NUDGING_NUDGE_PROBE_X0_OFFSETS: [u8; 8] = [0, 15, 0, 15, 0, 0, 15, 15];

pub(super) const HANDLE_NUDGING_NUDGE_PROBE_Y1_OFFSETS: [u8; 8] = [23, 23, 8, 8, 8, 23, 8, 23];

pub(super) const HANDLE_NUDGING_NUDGE_PROBE_X1_OFFSETS: [u8; 8] = [0, 15, 0, 15, 15, 15, 0, 0];

pub(super) const PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_Y_OFFSETS_0: [i8; 4] = [-4, 20, 4, 4];

pub(super) const PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_Y_OFFSETS_1: [i8; 4] = [-4, 20, 12, 12];

pub(super) const PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_X_OFFSETS_0: [i8; 4] = [4, 4, -4, 20];

pub(super) const PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_X_OFFSETS_1: [i8; 4] = [12, 12, -4, 20];

pub(super) const LINK_FIND_VALID_LANDING_TILE_NORTH_Y_DELTAS: [u8; 32] = [
    16, 16, 20, 20, 24, 24, 28, 28, 32, 32, 36, 36, 40, 40, 44, 44, 48, 48, 48, 48, 48, 48, 48, 48,
    48, 48, 48, 48, 48, 48, 48, 48,
];

pub(super) const LINK_FIND_VALID_LANDING_TILE_NORTH_Z_DELTAS: [u8; 32] = [
    24, 24, 24, 24, 28, 28, 28, 28, 32, 32, 32, 32, 36, 36, 36, 36, 40, 40, 40, 40, 44, 44, 44, 44,
    48, 48, 48, 48, 52, 52, 52, 52,
];

pub(super) const LINK_FIND_VALID_LANDING_TILE_NORTH_TIMERS: [u8; 32] = [
    16, 16, 20, 20, 24, 24, 28, 28, 32, 32, 36, 36, 40, 40, 44, 44, 48, 48, 48, 48, 48, 48, 48, 48,
    48, 48, 48, 48, 48, 48, 48, 48,
];

pub(super) const LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_X_DELTAS: [u8; 32] = [
    8, 8, 8, 8, 16, 16, 16, 16, 24, 24, 24, 24, 16, 16, 16, 16, 8, 20, 20, 20, 24, 24, 24, 24, 28,
    28, 28, 28, 32, 32, 32, 32,
];

pub(super) const LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_Y_DELTAS: [u8; 32] = [
    8, 8, 8, 8, 16, 16, 20, 20, 24, 24, 24, 24, 32, 32, 32, 32, 8, 20, 20, 20, 24, 24, 24, 24, 28,
    28, 28, 28, 32, 32, 32, 32,
];

pub(super) const LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_Z_DELTAS: [u8; 32] = [
    32, 32, 32, 32, 32, 32, 32, 32, 36, 36, 36, 36, 40, 40, 40, 40, 32, 40, 40, 40, 44, 44, 44, 44,
    48, 48, 48, 48, 52, 52, 52, 52,
];

pub(super) const TILE_DETECT_MAIN_HANDLER_SPIN_OFFSETS: [u8; 8] = [10, 6, 14, 2, 12, 4, 8, 0];

pub(super) const TILE_DETECT_MAIN_HANDLER_X_OFFSETS: [i8; 40] = [
    8, 8, 8, 8, 6, 8, -1, 22, 19, 19, 0, 19, 6, 8, -1, 22, 8, 8, 8, 8, 8, 8, 0, 15, 6, 8, -10, 29,
    6, 8, -6, 22, 6, 8, -4, 22, -4, 22, -4, 22,
];

pub(super) const TILE_DETECT_MAIN_HANDLER_Y_OFFSETS: [i8; 40] = [
    20, 20, 20, 20, 4, 28, 16, 16, 22, 22, 22, 22, 4, 24, 16, 16, 16, 16, 16, 16, 20, 20, 23, 23,
    -4, 36, 16, 16, 4, 28, 16, 16, 4, 28, 16, 16, 4, 4, 28, 28,
];

pub(super) const LINK_HANDLE_LIFTABLES_ACTION_FOR_GLOVES: [u8; 7] = [0, 1, 0, 0, 2, 1, 2];

pub(super) const LINK_HANDLE_LIFTABLES_ACTION_FOR_TILE: [u8; 7] = [2, 3, 1, 4, 0, 5, 6];

pub(super) const LINK_APRESS_BASIC_ABILITY_BITMASKS: [u8; 8] =
    [0xe0, 0x40, 4, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0];

pub(super) const LINK_HANDLE_LIFTABLES_ACTION_X: [i8; 4] = [7, 7, -3, 16];

pub(super) const LINK_HANDLE_LIFTABLES_ACTION_Y: [i8; 4] = [6, 24, 12, 12];

pub(super) const LINK_BONK_AND_SMASH_LIFTABLE_TILE_ATTR_TO_TERRAIN_TYPE: [u8; 9] =
    [0x54, 0x52, 0x50, 0xff, 0x51, 0x53, 0x55, 0x56, 0x57];

pub(super) const FINISH_INDOOR_COLLISION_COMMON_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const FINISH_INDOOR_COLLISION_COMMON_RUPEE_Y_OFFSETS: [u8; 4] = [8, 24, 16, 16];

pub(super) const FINISH_INDOOR_COLLISION_COMMON_RUPEE_X_OFFSETS: [u8; 4] = [8, 8, 0, 15];

pub(super) const OVERWORLD_GET_LINK_MAP16_COORDS_RESULT_X_OFFSETS: [i16; 4] = [7, 7, -3, 16];

pub(super) const OVERWORLD_GET_LINK_MAP16_COORDS_RESULT_Y_OFFSETS: [i16; 4] = [6, 24, 12, 12];

pub(super) const SMASH_ROCK_PILE_FROM_LIFT_IMPL_BIG_ROCK_MAP16_QUADRANT_OFFSETS: [i16; 4] =
    [0, -1, -64, -65];

pub(super) const SMASH_ROCK_PILE_FROM_LIFT_IMPL_BIG_ROCK_QUADRANT_Y_OFFSETS: [i16; 4] =
    [0, 0, -64, -64];

pub(super) const SMASH_ROCK_PILE_FROM_LIFT_IMPL_BIG_ROCK_QUADRANT_X_OFFSETS: [i16; 4] =
    [0, -1, 0, -1];

pub(super) const OVERWORLD_DO_MAP_UPDATE32X32_FOR_SMASH_DOOR_ANIM_TILES: [u16; 56] = [
    0x0da8, 0x0da9, 0x0daa, 0x0dab, 0x0dac, 0x0dad, 0x0dae, 0x0daf, 0x0db0, 0x0db1, 0x0db2, 0x0db3,
    0x0db6, 0x0db7, 0x0db8, 0x0db9, 0x0dba, 0x0dbb, 0x0dbc, 0x0dbd, 0x0dcd, 0x0dce, 0x0dcf, 0x0dd0,
    0x0dd3, 0x0dd4, 0x0dd5, 0x0dd6, 0x0dd7, 0x0dd8, 0x0dd9, 0x0dda, 0x0dd1, 0x0dd2, 0x0dd3, 0x0dd4,
    0x0dd1, 0x0dd2, 0x0dd7, 0x0dd8, 0x0918, 0x0919, 0x091a, 0x091b, 0x0ddb, 0x0ddc, 0x0ddd, 0x0dde,
    0x0dd1, 0x0dd2, 0x0ddb, 0x0ddc, 0x0e21, 0x0e22, 0x0e23, 0x0e24,
];

pub(super) const LINK_HANDLE_DIAGONAL_KICKBACK_X_OFFSETS_0: [i8; 10] =
    [0, 1, 1, 1, 2, 2, 2, 3, 3, 3];

pub(super) const LINK_HANDLE_DIAGONAL_KICKBACK_X_OFFSETS_1: [i8; 10] =
    [0, -1, -1, -1, -2, -2, -2, -3, -3, -3];

pub(super) const LINK_HANDLE_DIAGONAL_KICKBACK_Y_OFFSETS_0: [i8; 10] =
    [0, 0, 0, 1, 1, 1, 2, 2, 2, 3];

pub(super) const LINK_HANDLE_DIAGONAL_KICKBACK_Y_OFFSETS_1: [i8; 16] =
    [0, 1, 1, 2, 2, 2, 3, 3, 3, 3, -91, 48, -16, 4, -91, 49];

pub(super) const LINK_HANDLE_CAPE_PASSIVE_LIFT_CHECK_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const PLAYER_CHECK_HANDLE_CAPE_STUFF_CAPE_DEPLETION_TIMERS: [u8; 3] = [4, 8, 8];

pub(super) const LINK_CHECK_MAGIC_COST_LINK_ITEM_MAGIC_COSTS: [u8; 27] = [
    16, 8, 4, 32, 16, 8, 8, 4, 2, 8, 4, 2, 8, 4, 2, 16, 8, 4, 4, 2, 2, 8, 4, 2, 16, 8, 4,
];

pub(super) const REFUND_MAGIC_LINK_ITEM_MAGIC_COSTS: [u8; 27] = [
    16, 8, 4, 32, 16, 8, 8, 4, 2, 8, 4, 2, 8, 4, 2, 16, 8, 4, 4, 2, 2, 8, 4, 2, 16, 8, 4,
];

pub(super) const REFUND_MAGIC_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const LINK_ITEM_CAPE_CAPE_DEPLETION_TIMERS: [u8; 3] = [4, 8, 8];

pub(super) const LINK_ITEM_CAPE_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const LINK_ITEM_ROD_ROD_ANIM_DELAYS: [u8; 3] = [3, 3, 5];

pub(super) const LINK_ITEM_HAMMER_HAMMER_ANIM_DELAYS: [u8; 3] = [3, 3, 16];

pub(super) const LINK_ITEM_BOW_BOW_DELAYS: [u8; 3] = [3, 3, 8];

pub(super) const LINK_PERFORM_OPEN_CHEST_RECEIVE_ITEM_ALTERNATES: [u8; 76] = [
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 68, 255, 255, 255, 255, 255, 53,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 70, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
];

pub(super) const LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_ACTION_TIMERS: [u8; 10] =
    [8, 24, 8, 24, 8, 32, 6, 8, 13, 13];

pub(super) const LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_ACTION_STEPS: [u8; 10] =
    [0, 1, 0, 1, 0, 1, 0, 1, 2, 3];

pub(super) const LINK_A_PRESS_LIFT_CARRY_THROW_LIFT_THROW_SEQUENCE_TIMERS: [u8; 29] = [
    6, 7, 7, 5, 10, 0, 23, 0, 18, 0, 18, 0, 8, 0, 8, 0, 254, 255, 17, 0, 0x54, 0x52, 0x50, 0xff,
    0x51, 0x53, 0x55, 0x56, 0x57,
];

pub(super) const LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_DIRS: [u8; 4] = [4, 8, 1, 2];

pub(super) const LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_ANIM_STEPS: [u8; 7] = [0, 1, 2, 3, 1, 2, 3];

pub(super) const LINK_A_PRESS_PULL_OBJECT_GRAB_WALL_ANIM_TIMER: [u8; 7] = [0, 5, 5, 12, 5, 5, 12];

pub(super) const LINK_A_PRESS_STATUE_DRAG_GRAB_WALL_DIRS: [u8; 4] = [4, 8, 1, 2];

pub(super) const LINK_A_PRESS_STATUE_DRAG_GRAB_WALL_ANIM_STEPS: [u8; 7] = [0, 1, 2, 3, 1, 2, 3];

pub(super) const LINK_A_PRESS_STATUE_DRAG_GRAB_WALL_ANIM_TIMER: [u8; 7] = [0, 5, 5, 12, 5, 5, 12];

pub(super) const LINK_ITEM_BOMBS_FEATURES0_MORE_ACTIVE_BOMBS: u32 = 1 << 2;

pub(super) const LINK_ITEM_POWDER_MUSHROOM_TIMER: [u8; 10] = [2, 1, 1, 3, 2, 2, 2, 2, 6, 0];

pub(super) const LINK_ITEM_SHOVEL_SHOVEL_ANIM_DELAY: [u8; 6] = [7, 18, 16, 7, 18, 16];

pub(super) const LINK_ITEM_SHOVEL_SHOVEL_ANIM_DELAY2: [u8; 6] = [0, 1, 2, 0, 1, 2];

pub(super) const LINK_ITEM_ETHER_ETHER_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 3, 3];

pub(super) const LINK_ITEM_ETHER_ETHER_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 6, 7];

pub(super) const LINK_ITEM_BOMBOS_BOMBOS_ANIM_DELAYS: [u8; 20] =
    [5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3, 3, 7, 1, 1, 1, 1, 1, 13];

pub(super) const LINK_ITEM_BOMBOS_BOMBOS_ANIM_STATES: [u8; 20] = [
    0, 1, 2, 3, 0, 1, 2, 3, 8, 9, 10, 11, 12, 10, 8, 13, 14, 15, 16, 17,
];

pub(super) const LINK_ITEM_QUAKE_QUAKE_ANIM_DELAYS: [u8; 12] =
    [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 19];

pub(super) const LINK_ITEM_QUAKE_QUAKE_ANIM_STATES: [u8; 12] =
    [0, 1, 2, 3, 0, 1, 2, 3, 18, 19, 20, 22];

pub(super) const LINK_STATE_USING_ETHER_ETHER_ANIM_DELAYS: [u8; 12] =
    [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 3, 3];

pub(super) const LINK_STATE_USING_ETHER_ETHER_ANIM_DELAYS_NO_FLASH: [u8; 12] =
    [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 24, 24];

pub(super) const LINK_STATE_USING_ETHER_ETHER_ANIM_STATES: [u8; 12] =
    [0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 6, 7];

pub(super) const LINK_STATE_USING_ETHER_FEATURES0_DIM_FLASHES: u32 = 65536;

pub(super) const LINK_STATE_USING_BOMBOS_BOMBOS_ANIM_DELAYS: [u8; 20] =
    [5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3, 3, 7, 1, 1, 1, 1, 1, 13];

pub(super) const LINK_STATE_USING_BOMBOS_BOMBOS_ANIM_STATES: [u8; 20] = [
    0, 1, 2, 3, 0, 1, 2, 3, 8, 9, 10, 11, 12, 10, 8, 13, 14, 15, 16, 17,
];

pub(super) const LINK_STATE_USING_QUAKE_QUAKE_ANIM_DELAYS: [u8; 12] =
    [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 19];

pub(super) const LINK_STATE_USING_QUAKE_QUAKE_ANIM_STATES: [u8; 12] =
    [0, 1, 2, 3, 0, 1, 2, 3, 18, 19, 20, 22];

pub(super) const LINK_ITEM_MIRROR_FEATURES0_MIRROR_TO_DARKWORLD: u32 = 8;

pub(super) const LINK_STATE_HOOKSHOTTING_HOOKSHOT_TARGET_Y_OFFSETS: [i8; 4] = [-8, -16, 0, 0];

pub(super) const LINK_STATE_HOOKSHOTTING_HOOKSHOT_TARGET_X_OFFSETS: [i8; 4] = [0, 0, 4, -12];

pub(super) const LINK_STATE_HOOKSHOTTING_HOOKSHOT_PULL_Y_VELOCITIES: [u8; 4] = [0xc0, 0x40, 0, 0];

pub(super) const LINK_STATE_HOOKSHOTTING_HOOKSHOT_PULL_X_VELOCITIES: [u8; 4] = [0, 0, 0xc0, 0x40];

pub(super) const LINK_ITEM_CANE_OF_SOMARIA_ROD_ANIM_DELAYS: [u8; 3] = [3, 3, 5];

pub(super) const LINK_ITEM_CANE_OF_SOMARIA_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const LINK_ITEM_CANE_OF_BYRNA_BYRNA_DELAYS: [u8; 4] = [19, 7, 13, 32];

pub(super) const LINK_ITEM_NET_BUG_NET_TIMERS: [u8; 40] = [
    11, 6, 7, 8, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 9, 4, 5, 6, 7, 8, 1, 2, 3, 4, 10,
    8, 1, 2, 3, 4, 5, 6, 7, 8,
];

pub(super) const LINK_ITEM_Y_BUTTON_BUTTON_INDEX_KEYS: [u8; 4] = [0, 0x40, 0x20, 0x10];

pub(super) const ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_Y_VEL: [u8; 4] = [0xc0, 0x40, 0, 0];

pub(super) const ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_X_VEL: [u8; 4] = [0, 0, 0xc0, 0x40];

pub(super) const ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_Y_DELTA: [i16; 4] = [4, 20, 8, 8];

pub(super) const ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_X_DELTA: [i16; 4] = [0, 0, -4, 11];

pub(super) const HANDLE_DUNGEON_LANDING_FROM_PIT_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const LINK_STATE_DASHING_DASH_SFX_TRIGGER_MASKS: [u8; 3] = [7, 15, 15];

pub(super) const LINK_STATE_DASHING_DASH_DIRECTION_BITS_BY_FACING: [u8; 4] = [8, 4, 2, 1];

pub(super) const LINK_STATE_DASHING_FEATURES0_TURN_WHILE_DASHING: u32 = 4;

pub(super) const LINK_STATE_DASHING_DASH_CONTROLS_TO_DIRECTION: [u8; 16] =
    [0, 1, 2, 0, 4, 4, 4, 0, 8, 8, 8, 0, 0, 0, 0, 0];

pub(super) const LINK_STATE_PITS_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const SPRITE_DUNGEON_DRAW_SINGLE_PUSH_BLOCK_PUSH_BLOCK_CHAR_INDEX_BY_MODE: [usize; 9] =
    [0, 1, 2, 3, 4, 0, 0, 0, 0];

pub(super) const SPRITE_DUNGEON_DRAW_SINGLE_PUSH_BLOCK_CHARS: [u8; 4] = [0x0c, 0x0c, 0x0c, 0xff];

pub(super) const PLAYER_HANDLER_00_GROUND_3_FEATURES0_MISC_BUG_FIXES: u32 = 4096;

pub(super) const PLAYER_MEMORY_LOCATION_TO_GIVE_ITEM_TO_MEMORY_LOCATIONS: [usize; 76] = [
    0xf359, 0xf359, 0xf359, 0xf359, 0xf35a, 0xf35a, 0xf35a, 0xf345, 0xf346, 0xf34b, 0xf342, 0xf340,
    0xf341, 0xf344, 0xf35c, 0xf347, 0xf348, 0xf349, 0xf34a, 0xf34c, 0xf34c, 0xf350, 0xf35c, 0xf36b,
    0xf351, 0xf352, 0xf353, 0xf354, 0xf354, 0xf34e, 0xf356, 0xf357, 0xf37a, 0xf34d, 0xf35b, 0xf35b,
    0xf36f, 0xf364, 0xf36c, 0xf375, 0xf375, 0xf344, 0xf341, 0xf35c, 0xf35c, 0xf35c, 0xf36d, 0xf36e,
    0xf36e, 0xf375, 0xf366, 0xf368, 0xf360, 0xf360, 0xf360, 0xf374, 0xf374, 0xf374, 0xf340, 0xf340,
    0xf35c, 0xf35c, 0xf36c, 0xf36c, 0xf360, 0xf360, 0xf372, 0xf376, 0xf376, 0xf373, 0xf360, 0xf360,
    0xf35c, 0xf359, 0xf34c, 0xf355,
];

pub(super) const LINK_PERFORM_THROW_LIFTABLE_TILE_ATTR_TO_TERRAIN_TYPE: [u8; 9] =
    [0x54, 0x52, 0x50, 0xff, 0x51, 0x53, 0x55, 0x56, 0x57];

pub(super) const LINK_ITEM_CANE_OF_SOMARIA_BLOCK_Y_VELOCITIES: [u8; 4] = [(-40i8) as u8, 40, 0, 0];

pub(super) const LINK_ITEM_CANE_OF_SOMARIA_BLOCK_X_VELOCITIES: [u8; 4] = [0, 0, (-40i8) as u8, 40];

pub(super) const LINK_ITEM_CANE_OF_SOMARIA_BLOCK_Z_VELOCITIES: [u8; 4] = [48, 24, 16, 8];

pub(super) const SPAWN_HAMMER_WATER_SPLASH_HAMMER_WATER_X: [i8; 4] = [0, 12, -8, 24];

pub(super) const SPAWN_HAMMER_WATER_SPLASH_HAMMER_WATER_Y: [i8; 4] = [8, 32, 24, 24];

pub(super) const DIGGING_GAME_GUY_ATTEMPT_PRIZE_SPAWN_DIGGING_GAME_XVEL: [u8; 2] =
    [(-16i8) as u8, 16];

pub(super) const DIGGING_GAME_GUY_ATTEMPT_PRIZE_SPAWN_DIGGING_GAME_X: [i8; 2] = [0, 19];

pub(super) const DIGGING_GAME_GUY_ATTEMPT_PRIZE_SPAWN_DIGGING_GAME_ITEMS: [u8; 4] =
    [0xdb, 0xda, 0xd9, 0xdf];

pub(super) const HANDLE_DOOR_TRANSITIONS_FEATURES0_MISC_BUG_FIXES: u32 = 4096;
