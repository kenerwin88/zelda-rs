//! Internal RAM semantics grouped by subsystem.

use crate::types::{read_le_u16, write_le_u16};

pub(crate) mod semantic {
    use super::{read_le_u16, write_le_u16};

    fn copy_word(ram: &mut [u8], dst: usize, src: usize) {
        let value = read_le_u16(ram, src);
        write_le_u16(ram, dst, value);
    }

    // Source addresses for semantic parity snapshots. Keep these constants close
    // to the typed views so every semantic field has an auditable WRAM source.
    const MAIN_MODULE: usize = 0x0010;
    const SUBMODULE: usize = 0x0011;
    const SUBSUBMODULE: usize = 0x00b0;
    const INIDISP_COPY: usize = 0x0013;
    const NMI_LOAD_BG_FROM_VRAM: usize = 0x0014;
    const NMI_SUBROUTINE_INDEX: usize = 0x0017;
    const FRAME_COUNTER: usize = 0x001a;
    const FLAG_UPDATE_CGRAM_IN_NMI: usize = 0x0015;
    const FLAG_UPDATE_HUD_IN_NMI: usize = 0x0016;
    const NMI_COPY_PACKETS_FLAG: usize = 0x0018;
    const TM_COPY: usize = 0x001c;
    const TS_COPY: usize = 0x001d;
    const BGMODE_COPY: usize = 0x0094;
    const MOSAIC_COPY: usize = 0x0095;
    const W12SEL_COPY: usize = 0x0096;
    const W34SEL_COPY: usize = 0x0097;
    const WOBJSEL_COPY: usize = 0x0098;
    const HDMAEN_COPY: usize = 0x009b;
    const NMI_BOOLEAN: usize = 0x0012;
    const NMI_UPDATE_TILEMAP_DST: usize = 0x0019;
    const TMW_COPY: usize = 0x001e;
    const TSW_COPY: usize = 0x001f;
    const NMI_THREAD_ACTIVE: usize = 0x012a;
    const NMI_LOAD_TARGET_ADDR: usize = 0x0116;
    const NMI_UPDATE_TILEMAP_SRC: usize = 0x0118;
    const ANIMATED_TILE_VRAM_ADDR: usize = 0x0134;
    const NMI_DISABLE_CORE_UPDATES: usize = 0x0710;
    const LOAD_CHR_HALFSLOT_EVEN_ODD: usize = 0x0aaa;
    const MOSAIC_TARGET_LEVEL: usize = 0x0c00b;
    const MUSIC_CONTROL: usize = 0x012c;
    const SOUND_EFFECT_AMBIENT: usize = 0x012d;
    const SOUND_EFFECT_1: usize = 0x012e;
    const SOUND_EFFECT_2: usize = 0x012f;
    const CURRENT_MUSIC_CONTROL: usize = 0x0130;
    const SOUND_EFFECT_AMBIENT_LAST: usize = 0x0131;
    const QUEUED_MUSIC_CONTROL: usize = 0x0132;
    const LAST_MUSIC_CONTROL: usize = 0x0133;
    const RAM_APUI00: usize = 0x0648;
    const ANIMATED_TILE_DATA_SRC: usize = 0x0adc;
    const FLAG_TRAVEL_BIRD: usize = 0x0af4;
    const HUD_TILE_INDICES_BUFFER: usize = 0x0c700;
    const NMI_FLAG_UPDATE_POLYHEDRAL: usize = 0x1f0c;
    const POLY_THREAD_STACK: usize = 0x1f0a;
    const IRQ_FLAG: usize = 0x128;
    const RUN_MAIN_THREAD: u8 = 1;
    const RUN_POLY_THREAD: u8 = 2;

    const ATTRACT_STATE: usize = 0x0022;
    const ATTRACT_SEQUENCE: usize = 0x0023;
    const ATTRACT_SCENE_TIMER: usize = 0x0025;
    const ATTRACT_X_BASE: usize = 0x0028;
    const ATTRACT_Y_BASE: usize = 0x0029;
    const ATTRACT_OAM_IDX: usize = 0x002a;
    const ATTRACT_X_BASE_HI: usize = 0x0040;
    const ATTRACT_MAIDEN_WARP_STEP: usize = 0x0051;
    const ATTRACT_SCENE_SUBSTEP: usize = 0x0060;
    const LINK_Y_COORD: usize = 0x0020;
    const LINK_X_COORD: usize = 0x0022;
    const LINK_Z_COORD: usize = 0x0024;
    const LINK_LAST_DIRECTION: usize = 0x0026;
    const LINK_ACTUAL_Y_VELOCITY: usize = 0x0027;
    const LINK_ACTUAL_X_VELOCITY: usize = 0x0028;
    const LINK_Z_VELOCITY: usize = 0x0029;
    const LINK_Y_SUBPIXEL: usize = 0x002a;
    const LINK_X_SUBPIXEL: usize = 0x002b;
    const LINK_Z_SUBPIXEL: usize = 0x002c;
    const LINK_FRAME_CHANGE_COUNTER: usize = 0x002d;
    const LINK_Y_COORD_ORIGINAL: usize = 0x0032;
    const LINK_ANIMATION_STEPS: usize = 0x002e;
    const LINK_FACING: usize = 0x002f;
    const LINK_Y_VELOCITY: usize = 0x0030;
    const LINK_X_VELOCITY: usize = 0x0031;
    const BUTTON_MASK_B_Y: usize = 0x003a;
    const Y_BUTTON_ACTION_FLAGS: usize = 0x003b;
    const BUTTON_B_FRAMES: usize = 0x003c;
    const TILEMAP_LOCATION_CALC_MASK: usize = 0x00ec;
    const JOYPAD1H_LAST: usize = 0x00f0;
    const JOYPAD1L_LAST: usize = 0x00f2;
    const JOYPAD1H_LAST2: usize = 0x00f8;
    const JOYPAD1L_LAST2: usize = 0x00fa;
    const FILTERED_JOYPAD_H: usize = 0x00f4;
    const FILTERED_JOYPAD_L: usize = 0x00f6;
    const LINK_DELAY_TIMER_SPIN_ATTACK: usize = 0x003d;
    const LINK_Y_COORD_SAFE_RETURN_LO: usize = 0x003e;
    const LINK_X_COORD_SAFE_RETURN_LO: usize = 0x003f;
    const PLAYER_OAM_Y_OFFSET: usize = 0x0044;
    const PLAYER_OAM_X_OFFSET: usize = 0x0045;
    const LINK_INCAPACITATED_TIMER: usize = 0x0046;
    const PLAYER_DEFENSE_FLAGS: usize = 0x0048;
    const LINK_VISIBILITY_STATUS: usize = 0x004b;
    const LINK_AUXILIARY_STATE: usize = 0x004d;
    const LINK_CANT_CHANGE_DIRECTION: usize = 0x0050;
    const LINK_CAPE_MODE: usize = 0x0055;
    const LINK_IS_BUNNY: usize = 0x0056;
    const TILEDETECT_PIT_TILE: usize = 0x0059;
    const PLAYER_PIT_DATA_INDEX: usize = 0x005a;
    const LINK_SPRITE_OAM_STATE_TIMER: usize = 0x005c;
    const PLAYER_NEAR_PIT_STATE: usize = 0x005b;
    const LINK_SPEED_SETTING: usize = 0x005e;
    const LINK_HANDLER_STATE: usize = 0x005d;
    const LINK_LAST_DIRECTION_MOVED_TOWARDS: usize = 0x0066;
    const LINK_DIRECTION: usize = 0x0067;
    const LINK_Y_PAGE_MOVEMENT_DELTA: usize = 0x0068;
    const LINK_X_PAGE_MOVEMENT_DELTA: usize = 0x0069;
    const LINK_MOVING_AGAINST_DIAG_TILE: usize = 0x006b;
    const IS_STANDING_IN_DOORWAY: usize = 0x006c;
    const TILEDETECT_DIAG_STATE: usize = 0x006e;
    const INDEX_OF_INTERACTING_TILE: usize = 0x0076;
    const LINK_RECOIL_Z_VELOCITY_DUNGEON: usize = 0x00c7;
    const LINK_DMA_GRAPHICS_INDEX: usize = 0x0100;
    const LINK_DMA_LEFT_SPRITE_BANK_INDEX: usize = 0x0102;
    const LINK_DMA_RIGHT_SPRITE_BANK_INDEX: usize = 0x0104;
    const LINK_QUADRANT_X: usize = 0x00a9;
    const LINK_QUADRANT_Y: usize = 0x00aa;
    const LINK_SPIN_ATTACK_STEP_COUNTER: usize = 0x0079;
    const LINK_TILE_BELOW: usize = 0x0114;
    const TILEDETECT_STAIRCASE_CACHE: usize = 0x02c2;
    const FLAG_CUSTOM_SPELL_ANIM_ACTIVE: usize = 0x0112;
    const LINK_DEBUG_VALUE_1: usize = 0x020b;
    const LINK_IS_BUNNY_MIRROR: usize = 0x02e0;
    const LINK_IS_TRANSFORMING: usize = 0x02e1;
    const LINK_BUNNY_TRANSFORM_TIMER: usize = 0x02e2;
    const LINK_SWORD_DELAY_TIMER: usize = 0x02e3;
    const FLAG_IS_LINK_IMMOBILIZED: usize = 0x02e4;
    const LINK_POSE_FOR_ITEM: usize = 0x02da;
    const LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE: usize = 0x02db;
    const ITEM_RECEIPT_METHOD: usize = 0x02e9;
    const TILEDETECT_TILE_TYPE: usize = 0x02ea;
    const TILEDETECT_CHEST: usize = 0x02e5;
    const TILEDETECT_KEY_LOCK_GRAVESTONES: usize = 0x02e7;
    const BITFIELD_SPIKE_CACTUS_TILES: usize = 0x02e8;
    const TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS: usize = 0x02ee;
    const BITMASK_FOR_DASHABLE_TILES: usize = 0x02ef;
    const FLAG_IS_ANCILLA_TO_PICK_UP: usize = 0x02ec;
    const LINK_DMA_SWORD_GRAPHICS_INDEX: usize = 0x0107;
    const LINK_DMA_SHIELD_GRAPHICS_INDEX: usize = 0x0108;
    const LINK_DMA_STAGING_INDEX: usize = 0x0109;
    const LINK_INCAPACITATED_CAMERA_TIMER: usize = 0x02c5;
    const LINK_RECOIL_TIMER: usize = 0x02c6;
    const LINK_Z_VELOCITY_COPY: usize = 0x02c7;
    const LINK_RECEIVE_ITEM_INDEX: usize = 0x02d8;
    const LINK_ITEM_HOLDING_TIMER: usize = 0x02d9;
    const LINK_X_COORD_COPY: usize = 0x02dc;
    const LINK_Y_COORD_COPY: usize = 0x02de;
    const TAGALONG_EVENT_FLAGS: usize = 0x02f2;
    const FLAG_IS_SPRITE_TO_PICK_UP_CACHED: usize = 0x02f4;
    const LINK_DASH_COUNTER: usize = 0x02f1;
    const PLAYER_ON_SOMARIA_PLATFORM: usize = 0x02f5;
    const TILEDETECT_MISC_TILES: usize = 0x02f6;
    const LINK_WANT_MAKE_NOISE_WHEN_DASHED: usize = 0x02f8;
    const LINK_IS_NEAR_MOVEABLE_STATUE: usize = 0x02fa;
    const PLAYER_HANDLER_TIMER: usize = 0x0300;
    const LINK_ITEM_IN_HAND: usize = 0x0301;
    const PIT_CORRECTION_ACTIVE_FLAG: usize = 0x0302;
    const LINK_CURRENT_ITEM_Y: usize = 0x0303;
    const EQ_SELECTED_ROD: usize = 0x0307;
    const LINK_CURRENT_ITEM_ACTIVE: usize = 0x0304;
    const LINK_STATE_BITS: usize = 0x0308;
    const LINK_PICKING_THROW_STATE: usize = 0x0309;
    const Y_BUTTON_ACTION_STEP: usize = 0x030a;
    const Y_BUTTON_ACTION_TIMER: usize = 0x030b;
    const LINK_FACING_MIRROR: usize = 0x0323;
    const LINK_ITEM_ACTION_STEP_SCRATCH: usize = 0x030d;
    const LINK_THROW_OAM_STATE_INDEX: usize = 0x030e;
    const RELATED_TO_MOVING_FLOOR_Y: usize = 0x0318;
    const RELATED_TO_MOVING_FLOOR_X: usize = 0x031a;
    const TILEDETECT_MOVING_FLOOR_TILES: usize = 0x0320;
    const FLAG_IS_SPRITE_TO_PICK_UP: usize = 0x0314;
    const TILE_COLL_FLAG: usize = 0x0315;
    const STATE_FOR_SPIN_ATTACK: usize = 0x031c;
    const TURTLE_ROCK_OAM_PRIORITY_FLAG: usize = 0x034e;
    const SORT_SPRITES_OFFSET_INTO_OAM_BUFFER: usize = 0x0352;
    const VALUE_COMPUTED_FOR_PLAYER_OAM: usize = 0x0354;
    const SECONDARY_WATER_GRASS_TIMER: usize = 0x0355;
    const PRIMARY_WATER_GRASS_TIMER: usize = 0x0356;
    const OAM_PRIORITY_VALUE_2: usize = 0x035d;
    const STEP_COUNTER_FOR_SPIN_ATTACK: usize = 0x031d;
    const LINK_SPIN_OFFSETS: usize = 0x031e;
    const COUNTDOWN_FOR_BLINK: usize = 0x031f;
    const LINK_MAYBE_SWIM_FASTER: usize = 0x032a;
    const SWIM_PLAYER_DIRECTION_FLAGS: usize = 0x0340;
    const TILEDETECT_ICY_FLOOR: usize = 0x0348;
    const TILEDETECT_WATER_STAIRCASE: usize = 0x034c;
    const LINK_IS_IN_DEEP_WATER: usize = 0x0345;
    const LINK_FLAG_MOVING: usize = 0x034a;
    const LINK_SWIM_HARD_STROKE: usize = 0x034f;
    const LINK_DEBUG_VALUE_2: usize = 0x0350;
    const DRAW_WATER_RIPPLES_OR_GRASS: usize = 0x0351;
    const TILEDETECT_SHALLOW_WATER: usize = 0x0359;
    const TILEDETECT_DESTRUCTION_AFTERMATH: usize = 0x035b;
    const LINK_ELECTROCUTE_ON_TOUCH: usize = 0x0360;
    const LINK_Z_VELOCITY_MIRROR: usize = 0x0362;
    const LINK_Z_VELOCITY_COPY_MIRROR: usize = 0x0363;
    const LINK_Z_COORD_MIRROR: usize = 0x0364;
    const LINK_FAINT_ANIMATION_ACTIVE: usize = 0x036b;
    const TILE_ACTION_INDEX: usize = 0x036c;
    const TILEDETECT_READ_SOMETHING: usize = 0x0366;
    const TILEDETECT_LEDGES_DOWN_LEFTRIGHT: usize = 0x036f;
    const TILEDETECT_DIAGONAL_LEDGE_TILES: usize = 0x0370;
    const LINK_GIVE_DAMAGE: usize = 0x0373;
    const LINK_COUNTDOWN_FOR_DASH: usize = 0x0374;
    const LINK_TIMER_JUMP_LEDGE: usize = 0x0375;
    const LINK_GRABBING_WALL: usize = 0x0376;
    const LINK_PULL_ACTION_STATE: usize = 0x0377;
    const LINK_POSITION_MODE: usize = 0x037a;
    const LINK_DISABLE_SPRITE_DAMAGE: usize = 0x037b;
    const PLAYER_SLEEP_IN_BED_STATE: usize = 0x037c;
    const LINK_POSE_DURING_OPENING: usize = 0x037d;
    const RELATED_TO_HOOKSHOT: usize = 0x037e;
    const PLAYER_RESET_ANCILLA_WORK_BYTE_24: usize = 0x03db;
    const LINK_SOMETHING_WITH_HOOKSHOT: usize = 0x03e9;
    const LINK_FORCE_HOLD_SWORD_UP: usize = 0x03ef;
    const LINK_ON_CONVEYOR_BELT: usize = 0x03f3;
    const LINK_TIMER_TEMPBUNNY: usize = 0x03f5;
    const LINK_NEED_FOR_POOF_FOR_TRANSFORM: usize = 0x03f7;
    const LINK_NEED_FOR_PULLFORRUPEES_SPRITE: usize = 0x03f8;
    const BIT9_OF_XCOORD: usize = 0x03fa;
    const IS_ARCHER_OR_SHOVEL_GAME: usize = 0x03fc;
    const ABOUT_TO_JUMP_OFF_LEDGE: usize = 0x047a;
    const LINK_Y_COORD_SAFE_RETURN_HI: usize = 0x0040;
    const LINK_X_COORD_SAFE_RETURN_HI: usize = 0x0041;
    const LINK_X_COORD_PREV: usize = 0x0fc2;
    const LINK_Y_COORD_PREV: usize = 0x0fc4;
    const LINK_Y_COORD_CACHED: usize = 0x0c184;
    const LINK_X_COORD_CACHED: usize = 0x0c186;
    const LINK_QUADRANT_X_CACHED: usize = 0x0c19e;
    const LINK_QUADRANT_Y_CACHED: usize = 0x0c19f;
    const LINK_Y_COORD_SPEXIT: usize = 0x0c108;
    const LINK_X_COORD_SPEXIT: usize = 0x0c10a;
    const LINK_Y_COORD_EXIT_OVERWORLD: usize = 0x0c148;
    const LINK_X_COORD_EXIT_OVERWORLD: usize = 0x0c14a;
    const LINK_FACING_CACHED: usize = 0x0c1a6;
    const LINK_IS_ON_LOWER_LEVEL_CACHED: usize = 0x0c1a7;
    const LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED: usize = 0x0c1a8;
    const LINK_DIRECTION_MASK_A: usize = 0x0042;
    const LINK_DIRECTION_MASK_B: usize = 0x0043;
    const TILEDETECT_SLOPE_COLLISION_BITS: usize = 0x000c;
    const TILEDETECT_COLLISION_BITS: usize = 0x000e;
    const TILEDETECT_WHICH_Y_POS: usize = 0x0051;
    const TILEDETECT_DIAGONAL_TILE: usize = 0x0038;
    const TILEDETECT_STAIR_TILE: usize = 0x0058;
    const TILEDETECT_BLOCK_FLAGS_LO: usize = 0x005f;
    const TILEDETECT_DOOR_DIRECTION_FLAGS: usize = 0x0062;
    const LINK_SPEED_MODIFIER: usize = 0x0057;
    const LINK_NUM_ORTHOGONAL_DIRECTIONS: usize = 0x006a;
    const LINK_IS_ON_LOWER_LEVEL: usize = 0x00ee;
    const SWIMMING_COUNTDOWN: usize = 0x02cb;
    const SWIM_STROKE_ANIM_STEP: usize = 0x02cc;
    const LINK_TIMER_PUSH_GET_TIRED: usize = 0x0371;
    const LINK_IS_RUNNING: usize = 0x0372;
    const LINK_PREVENT_FROM_MOVING: usize = 0x0b7b;
    const LINK_IS_ON_LOWER_LEVEL_MIRROR: usize = 0x0476;
    const LINK_CURRENT_HEALTH: usize = 0x0f36d;
    const LINK_ITEM_FLIPPERS: usize = 0x0f356;
    const LINK_ITEM_MOON_PEARL: usize = 0x0f357;
    const LINK_MAGIC_POWER: usize = 0x0f36e;
    const LINK_MAGIC_CONSUMPTION: usize = 0x0f37b;
    const LINK_EQUIPPED_ITEM: usize = 0x0f340;
    const LINK_ITEM_BOW: usize = 0x0f340;
    const LINK_BOTTLE_INFO: usize = 0x0f35c;
    const LINK_ITEM_BOMBS: usize = 0x0f343;
    const LINK_ITEM_BOTTLE_INDEX: usize = 0x0f34f;
    const LINK_RUPEES_GOAL: usize = 0x0f360;
    const LINK_RUPEES_ACTUAL: usize = 0x0f362;
    const LINK_COMPASS: usize = 0x0f364;
    const LINK_BIGKEY: usize = 0x0f366;
    const LINK_DUNGEON_MAP: usize = 0x0f368;
    const LINK_HEART_PIECES: usize = 0x0f36b;
    const LINK_HEALTH_CAPACITY: usize = 0x0f36c;
    const LINK_NUM_KEYS: usize = 0x0f36f;
    const LINK_RUPEES_IN_POND: usize = 0x0f36a;
    const LINK_BOMB_UPGRADES: usize = 0x0f370;
    const LINK_ARROW_UPGRADES: usize = 0x0f371;
    const LINK_HEARTS_FILLER: usize = 0x0f372;
    const LINK_MAGIC_FILLER: usize = 0x0f373;
    const LINK_WHICH_PENDANTS: usize = 0x0f374;
    const LINK_BOMB_FILLER: usize = 0x0f375;
    const LINK_ARROW_REFILL_COUNTER: usize = 0x0f376;
    const LINK_NUM_ARROWS: usize = 0x0f377;
    const LINK_ABILITY_FLAGS: usize = 0x0f379;
    const LINK_HAS_CRYSTALS: usize = 0x0f37a;
    const LINK_KEYS_EARNED_PER_DUNGEON: usize = 0x0f37c;
    const DEATHS_PER_PALACE: usize = 0x0f3e7;
    const PENDING_DEATH_SAVE_COUNTER: usize = 0x0f403;
    const TOTAL_DEATH_SAVE_COUNTER: usize = 0x0f405;
    const LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP: usize = 0x04ca;
    const SAVE_DUNG_INFO: usize = 0x0f000;
    const MIRROR_WARP_TARGET_INDEX: usize = 0x06a0;
    const MIRROR_WARP_TARGET_OFFSETS: usize = 0x06a2;
    const MIRROR_WARP_VELOCITY_DELTAS: usize = 0x06a6;
    const MIRROR_WARP_WAVE_OFFSET: usize = 0x06aa;
    const MIRROR_WARP_DISPLACEMENT: usize = 0x06ac;
    const MIRROR_WARP_SUBPIXEL: usize = 0x06ae;
    const MIRROR_WARP_RESERVED: usize = 0x06b0;
    const MIRROR_WARP_WAVE_LENGTH: usize = 0x06b2;
    const MIRROR_WARP_SPACING_A: usize = 0x06b4;
    const MIRROR_WARP_SPACING_B: usize = 0x06b6;
    const MIRROR_WARP_LOAD_STEP_COUNTER: usize = 0x06ba;
    const MIRROR_WARP_ANIMATION_COUNTER: usize = 0x06bb;
    const CUR_PALACE_INDEX_X2: usize = 0x040c;
    const HUD_CUR_ITEM: usize = 0x0202;
    const HUD_CUR_ITEM_X: usize = 0x0656;
    const HUD_CUR_ITEM_L: usize = 0x0657;
    const HUD_CUR_ITEM_R: usize = 0x0658;
    const HUD_POST_MESSAGE_REFRESH_FLAG: usize = 0x0204;
    const SRAM_PROGRESS_INDICATOR: usize = 0x0f3c5;
    const SRAM_PROGRESS_FLAGS: usize = 0x0f3c6;
    const SRAM_PROGRESS_INDICATOR_3: usize = 0x0f3c9;
    const SAVEGAME_MAP_ICONS_INDICATOR: usize = 0x0f3c7;
    const SAVEGAME_IS_DARKWORLD: usize = 0x0f3ca;
    const MODAL_PAUSE_FLAG: usize = 0x0fc1;
    const FLAG_BLOCK_LINK_MENU: usize = 0x0ffc;

    const DUNGEON_ROOM: usize = 0x00a0;
    const PLAYER_IS_INDOORS: usize = 0x001b;
    const OVERWORLD_MAP_STATE: usize = 0x0200;
    const OVERWORLD_ENTRANCE_SEQUENCE_COUNTER: usize = 0x00c8;
    const OVERWORLD_SCREEN_INDEX: usize = 0x008a;
    const OVERWORLD_AREA_INDEX: usize = 0x040a;
    const OVERWORLD_SCREEN_TRANS_DIR_BITS2: usize = 0x0416;
    const OVERWORLD_SCREEN_TRANSITION: usize = 0x0418;
    const OVERWORLD_TRANSITION_DIR: usize = 0x069c;
    const OVERWORLD_EVENT_INFO: usize = 0x0f280;
    const OVERLAY_INDEX: usize = 0x008c;
    const DUNGEON_HEADER_TAG: usize = 0x00ae;
    const DUNG_CUR_FLOOR: usize = 0x00a4;
    const DUNG_HDR_COLLISION_2: usize = 0x00ad;
    const DUNG_DRAW_WIDTH_INDICATOR: usize = 0x00b2;
    const DUNG_DRAW_HEIGHT_INDICATOR: usize = 0x00b4;
    const DUNGEON_HEADER_COLLISION_2_MIRROR: usize = 0x0428;
    const DUNGEON_HEADER_HOLE_TELEPORTER_PLANE: usize = 0x063c;
    const DUNGEON_HEADER_STAIRCASE_PLANE: usize = 0x063d;
    const DUNGEON_HEADER_TRAVEL_DESTINATIONS: usize = 0x0c000;
    const DUNG_LOAD_PTR_OFFS: usize = 0x00ba;
    const DUNG_SAVEGAME_STATE_BITS: usize = 0x0402;
    const BG1_MOVE_CALC_BUFFER: usize = 0x041c;
    const DUNG_MISC_OBJS_INDEX: usize = 0x042c;
    const DUNG_CUR_DOOR_IDX: usize = 0x0460;
    const DUNG_FLAG_TRAPDOORS_DOWN: usize = 0x0468;
    const DUNG_HDR_COLLISION: usize = 0x046c;
    const DUNG_HDR_BG2_PROPERTIES: usize = 0x0414;
    const CHANGEABLE_DUNGEON_OBJECT_INDEX: usize = 0x05fc;
    const DUNGEON_ROOM_INDEX2: usize = 0x048e;
    const DUNGEON_FLOOR_Y_VELOCITY: usize = 0x0310;
    const DUNGEON_FLOOR_X_VELOCITY: usize = 0x0312;
    const DUNG_OBJECT_TILEMAP_POS: usize = 0x0540;
    const DUNG_OBJECT_POS_IN_OBJDATA: usize = 0x0520;
    const DUNG_NUM_LIT_TORCHES: usize = 0x045a;
    const DUNG_CUR_QUADRANT_UPLOAD: usize = 0x045c;
    const DUNG_FLAG_STATECHANGE_WATERPUZZLE: usize = 0x0642;
    const DUNG_DOOR_OPENED: usize = 0x0400;
    const DUNG_DOOR_OPENED_INCL_ADJACENT: usize = 0x068c;
    const DUNG_CUR_DOOR_POS_DUNGEON: usize = 0x068e;
    const DOOR_ANIMATION_STEP_INDICATOR_DUNGEON: usize = 0x0690;
    const DOOR_OPEN_CLOSED_COUNTER: usize = 0x0692;
    const DUNG_INTER_STAIRCASES: usize = 0x06b0;
    const DUNGEON_REPLACEMENT_TILE_STATE: usize = 0x0500;
    const DUNG_DOOR_TILEMAP_ADDRESS: usize = 0x19a0;
    const DUNG_BG2: usize = 0x2000;
    const DUNG_BG1: usize = 0x4000;
    const DUNG_WANT_LIGHTS_OUT: usize = 0x0c005;
    const DUNG_WANT_LIGHTS_OUT_COPY: usize = 0x0c006;
    const DUNG_CUR_FLOOR_CACHED: usize = 0x0c1aa;
    const ATTRIBUTES_FOR_TILE_PLAYER: usize = 0x0fe00;
    const MAP16_LOAD_SRC_OFF: usize = 0x0084;
    const MAP16_LOAD_DST_OFF: usize = 0x0086;
    const MAP16_LOAD_Y_UNIT: usize = 0x0088;
    const BG1_X_SCROLL: usize = 0x00e0;
    const BG2_X_SCROLL: usize = 0x00e2;
    const BG1_Y_SCROLL: usize = 0x00e6;
    const BG2_Y_SCROLL: usize = 0x00e8;
    const BG1_X_OFFSET: usize = 0x011a;
    const BG1_Y_OFFSET: usize = 0x011c;
    const SAVED_MODULE_FOR_MENU: usize = 0x010c;
    const OAM_CUR_PTR: usize = 0x0090;
    const OAM_EXT_CUR_PTR: usize = 0x0092;
    const CAMERA_Y: usize = 0x0618;
    const CAMERA_X: usize = 0x061c;
    const RNG_SEED: usize = 0x0fa1;
    const SWIM_ACCELERATION_MODE: usize = 0x032b;
    const SWIM_SPEED_ACTIVE_FLAG: usize = 0x032f;
    const SWIM_MAX_SPEED: usize = 0x0334;
    const SWIM_ACCELERATION_DIRECTION: usize = 0x0338;
    const SWIM_ACCELERATION: usize = 0x033c;
    const PUSHEDBLOCKS_X_HI: usize = 0x05e0;
    const PUSHEDBLOCKS_X_LO: usize = 0x05e4;
    const PUSHEDBLOCKS_TARGET: usize = 0x05e8;
    const PUSHEDBLOCKS_Y_HI: usize = 0x05ec;
    const PUSHEDBLOCKS_Y_LO: usize = 0x05f0;
    const PUSHEDBLOCKS_SUBPIXEL: usize = 0x05f4;
    const PUSHEDBLOCK_FACING_PLAYER: usize = 0x05f8;
    const DOOR_DEBRIS_X: usize = 0x0728;
    const DOOR_DEBRIS_Y: usize = 0x0732;
    const DOOR_DEBRIS_DIRECTION: usize = 0x073c;
    const TORCH_TIMERS: usize = 0x04f0;
    const DUNGEON_TORCH_ATTR: usize = 0x0333;
    const DUNGEON_TORCH_DATA: usize = 0x0fb40;
    const DUNG_INDEX_OF_TORCHES_START: usize = 0x0478;
    const GANON_TORCH_COUNT: usize = 0x04c5;
    const TRIGGER_SPECIAL_ENTRANCE: usize = 0x04c6;
    const WHICH_STAIRCASE_INDEX: usize = 0x0462;
    const STAIRCASE_MOVE_COUNTER: usize = 0x0464;
    const MOVABLE_BLOCK_DATAS: usize = 0x0f940;
    const OVERWORLD_TILE_THEME_INDEX: usize = 0x0aa0;
    const MAIN_TILE_THEME_INDEX: usize = 0x0aa1;
    const AUX_TILE_THEME_INDEX: usize = 0x0aa2;
    const SPRITE_GRAPHICS_INDEX: usize = 0x0aa3;
    const SPRITE_GRAPHICS_INDEX_SPEXIT: usize = 0x0c127;
    const SPRITE_GRAPHICS_INDEX_EXIT: usize = 0x0c167;
    const DRAG_PLAYER_X: usize = 0x0b7c;
    const DRAG_PLAYER_Y: usize = 0x0b7e;
    const BLIND_HEAD_ANIM_COUNTER: usize = 0x0b69;
    const SPRITE_LIMIT_INSTANCE: usize = 0x0b6a;
    const SPRITE_ROOM_ORIGIN_X_HI: usize = 0x0fb0;
    const SPRITE_ROOM_ORIGIN_Y_HI: usize = 0x0fb1;
    const SPRITE_PICKUP_SLOT_CACHE: usize = 0x0fb2;
    const SPRITE_SHARED_SCRATCH_A: usize = 0x0fb6;
    const SPRITE_TILETYPE: usize = 0x0fa5;
    const CUR_OBJECT_INDEX: usize = 0x0fa0;
    const SPRITE_CHR_HALFSLOT_STATE: usize = 0x0fc6;
    const SPRITE_ALERT_FLAG: usize = 0x0fdc;
    const HAUNTED_GROVE_FLUTE_EVENT_LATCH: usize = 0x0fdd;
    const CUR_SPRITE_X: usize = 0x0fd8;
    const CUR_SPRITE_Y: usize = 0x0fda;
    const SPRITE_RESET_SCRATCH_A: usize = 0x0ff8;
    const SPRITE_RESET_SCRATCH_B: usize = 0x0ffb;
    const REPULSESPARK_TIMER: usize = 0x0fac;
    const REPULSESPARK_X_LO: usize = 0x0fad;
    const REPULSESPARK_Y_LO: usize = 0x0fae;
    const REPULSESPARK_FLOOR_STATUS: usize = 0x0b68;
    const GARNISH_ACTIVE: usize = 0x0fb4;
    const SPR_RANGED_BASED_TOGGLER: usize = 0x0fb7;
    const SPRCOLL_Y_BASE: usize = 0x0fbe;
    const ACTIVE_OVERLORD_INDEX: usize = 0x0fde;
    const OVERWORLD_BOULDER_TRAP_COUNT: usize = 0x0ffd;
    const OVERWORLD_BOULDER_TRAP_TIMER: usize = 0x0ffe;
    const DUNGEON_TRAP_TRIGGER_LATCH: usize = 0x0b9e;
    const DUNGEON_ROOM_HISTORY: usize = 0x0b80;
    const DUNG_FLOOR_MOVE_FLAGS: usize = 0x041a;
    const DUNG_FLOOR_X_OFFS: usize = 0x0422;
    const DUNG_FLOOR_Y_OFFS: usize = 0x0424;
    const ACTIVATE_BOMB_TRAP_OVERLORD: usize = 0x0cf4;
    const OVERLORD_OFFSET_SPRITE_POS: usize = 0x0b48;
    const OVERWORLD_OFFSET_BASE_Y: usize = 0x0708;
    const OVERWORLD_OFFSET_MASK_Y: usize = 0x070a;
    const OVERWORLD_OFFSET_BASE_X: usize = 0x070c;
    const OVERWORLD_OFFSET_MASK_X: usize = 0x070e;
    const TILEDETECT_INROOM_STAIRCASE: usize = 0x02c0;
    const SCRATCH_1: usize = 0x0074;
    const LIFTABLE_TILE_DETECTED_INDEX_DOUBLED: usize = 0x036a;
    const KIND_OF_IN_ROOM_STAIRCASE: usize = 0x044a;
    const DUNG_CHEST_LOCATIONS: usize = 0x06e0;
    const MOVING_FLOOR_BG_CHECK_FLAGS: usize = 0x03f1;
    const FORCE_MOVE_ANY_DIRECTION: usize = 0x0049;
    const CHEAT_WALK_THROUGH_WALLS: usize = 0x037f;
    const ROOM_TRANSITIONING_FLAGS: usize = 0x00ef;
    const SPRITE_WHERE_IN_ROOM: usize = 0x1df80;
    const SPRITE_DRAW_PRIORITY_OVERRIDE: usize = 0x0cfe;
    const SPRITE_GFX_SUBSET_0: usize = 0x0c2fc;
    const OVERWORLD_EXIT_TILE_THEME_INDEX: usize = 0x0c164;
    const SCRATCH_R16: usize = 0x00c8;
    const SCRATCH_R18: usize = 0x00ca;
    const TEMP_COUNTER: usize = 0x0fb5;
    const FLOOR_1_FILLER_TILES: usize = 0x0490;
    const FLOOR_2_FILLER_TILES: usize = 0x046a;
    const AUX_PALETTE_BUFFER: usize = 0x0c300;
    const MAIN_PALETTE_BUFFER: usize = 0x0c500;
    const PALETTE_FILTER_COUNTDOWN: usize = 0x0c007;
    const OVERWORLD_PALETTE_AUX_OR_MAIN: usize = 0x0aa8;
    const DARKENING_OR_LIGHTENING_SCREEN: usize = 0x0c009;
    const MOSAIC_LEVEL: usize = 0x0c011;
    const CGWSEL_COPY: usize = 0x0099;
    const CGADSUB_COPY: usize = 0x009a;
    const COLDATA_COPY0: usize = 0x009c;
    const COLDATA_COPY1: usize = 0x009d;
    const COLDATA_COPY2: usize = 0x009e;
    const BIRD_TRAVEL_STATUS: usize = 0x1af0;
    const DUNGEON_MAP_CURRENT_FLOOR: usize = 0x020e;
    const MESSAGING_MODULE: usize = 0x1cd8;
    const MESSAGE_OR_SPRITE_STATE_CACHE: usize = 0x02f0;
    const TEXT_RENDER_STATE: usize = 0x1cd4;
    const TEXT_WAIT_COUNTDOWN2: usize = 0x1ce9;
    const MENU_ANIMATION_TIMER: usize = 0x00c8;
    const GAME_OVER_LETTER_CURSOR: usize = 0x039d;
    const MESSAGING_RENDER_BUFFER: usize = 0x10000;
    const MESSAGING_TEXT_BUFFER: usize = 0x11200;
    const DIALOGUE_MESSAGE_INDEX: usize = 0x1cf0;
    const MULTISELECT_CHOICE: usize = 0x1ce8;
    const MULTISELECT_CHOICE_BACKUP: usize = 0x1cf4;
    const DIALOGUE_MSG_SRC_OFFS: usize = 0x1cdd;
    const VWF_GLYPH_CURSOR: usize = 0x0724;
    const VWF_ARR: usize = 0x0c230;
    const ETHER_ANGLE: usize = 0x15800;
    const ETHER_RADIUS: usize = 0x15808;
    const ETHER_BEAM_Y: usize = 0x1580a;
    const ETHER_BEAM_TOP_BUCKET: usize = 0x1580c;
    const ETHER_ORBIT_X: usize = 0x1580e;
    const ETHER_ORBIT_Y: usize = 0x15810;
    const ETHER_SPIN_COUNTDOWN: usize = 0x15812;
    const ETHER_ORB_Y: usize = 0x15813;
    const ETHER_ORB_X: usize = 0x15815;
    const HUD_INVENTORY_ORDER: usize = 0x0225;
    const SELECT_FILE_TRANSITION_SCRATCH: usize = 0x00c9;
    const SELECT_FILE_CHOICE_SCRATCH: usize = 0x00ca;
    const SELECTFILE_SAVE_SLOT_FLAGS: usize = 0x00bf;
    const SELECT_FILE_CURSOR_SCRATCH: usize = 0x00c8;
    const SELECT_FILE_TARGET_SCRATCH: usize = 0x00ca;
    const SELECT_FILE_REMEMBERED_CURSOR: usize = 0x0b9d;
    const OVERWORLD_SCREEN_TRANS_DIR_BITS: usize = 0x0410;
    const SOMARIA_BLOCK_BG_CHECK_FLAG: usize = 0x03f4;
    const TAGALONG_SHARED_STATE_A: usize = 0x02d4;
    const TAGALONG_ANIM_FRAME_COUNTER: usize = 0x02d7;
    const PLAYER_POSE_DRAW_COUNTER: usize = 0x0379;
    const PLAYER_SPECIAL_DRAW_FLAG: usize = 0x03fd;
    const VIRQ_TRIGGER: usize = 0x00ff;
    const GAME_OVER_CHECK_FLAG: usize = 0x010a;
    const RESTART_CHECK_FLAG: usize = 0x04aa;
    const WHICH_STARTING_POINT: usize = 0x0f3c8;
    const PALETTE_SP0L: usize = 0x0aac;
    const PALETTE_SP5L: usize = 0x0aad;
    const PALETTE_SP6L: usize = 0x0aae;
    const PALETTE_SP6R_INDOORS: usize = 0x0ab1;
    const HUD_PALETTE: usize = 0x0ab2;
    const OVERWORLD_PALETTE_AUX2_BP5TO7_HI: usize = 0x0ab5;
    const PALETTE_MAIN_INDOORS: usize = 0x0ab6;
    const OVERWORLD_PALETTE_AUX3_BP7_LO: usize = 0x0ab8;
    const MISC_SPRITES_GRAPHICS_INDEX: usize = 0x0aa4;
    const BIRDTRAVEL_STATUS: usize = 0x1af0;
    const WHICH_ENTRANCE: usize = 0x10e;
    const SPRCOLL_X_SIZE: usize = 0x0fb8;
    const SPRCOLL_Y_SIZE: usize = 0x0fba;
    const OAM_REGION_BASE: usize = 0x0fe0;
    const OAM_REGION_ALLOC: usize = 0x0fec;
    const INTRO_WANT_DOUBLE_RET: usize = 0x1e02;
    const INTRO_SPRITE_ALLOC: usize = 0x1e08;
    const TRIFORCE_CTR: usize = 0x1e0c;
    const ENDING_WHICH_DUNG: usize = 0x0cc;
    const ENDING_CREDIT_DIGIT_CHAR: usize = 0x0ce;
    const BG_TILE_ANIMATION_COUNTDOWN: usize = 0x0c00d;
    const INTRO_SWORD_YPOS: usize = 0xc8;
    const INTRO_SWORD_SPARKLE_TIMER: usize = 0xca;
    const INTRO_SWORD_SPARKLE_STEP: usize = 0xcb;
    const INTRO_SWORD_ANIM_STEP: usize = 0xcc;
    const INTRO_SWORD_SPARKLE_Y_OFFSET: usize = 0xcd;
    const INTRO_SWORD_FLASH_RGB_CHANNEL: usize = 0xd0;
    const RAW_SFX_PAN_VALUE: usize = 0x0cf8;
    const TILE_INTERACTION_SHARED_FLAG: usize = 0x0223;
    const PUSHED_BLOCK_MODE: usize = 0x02c3;
    const DMA_HEAD_POINTER: usize = 0x0ae8;
    const DMA_BODY_POINTER: usize = 0x0aea;
    const OVERWORLD_FIXED_COLOR_PLUSMINUS: usize = 0x0c017;
    const HUD_FLOOR_CHANGED_TIMER: usize = 0x04a0;
    const SUPER_BOMB_INDICATOR_TIMER: usize = 0x4b4;
    const SUPER_BOMB_INDICATOR_COUNTER: usize = 0x4b5;
    const HDR_DUNGEON_DARK_WITH_LANTERN: usize = 0x458;
    const HUD_MODULE_TICK_COUNTER: usize = 0x0206;
    const FLASHING_CIRCLE_TIMER: usize = 0x0207;
    const HEART_REFILL_COUNTDOWN: usize = 0x0208;
    const HEART_REFILL_ANIM_SUBPOS: usize = 0x0209;
    const IS_DOING_HEART_ANIMATION: usize = 0x020a;
    const MENU_PREV_JOYPAD_H: usize = 0x0bd;
    const BOTTLE_MENU_ROW: usize = 0x0205;
    const EQUIPMENT_MENU_EXIT_STATE: usize = 0x034b;
    const RUPEE_SFX_SOUND_DELAY: usize = 0x0cfd;
    const IS_IN_DARK_WORLD_FLAG: usize = 0x0fff;
    const FLAG_OVERWORLD_AREA_CHANGED: usize = 0x0abf;
    const LAST_LIGHT_VS_DARK_WORLD: usize = 0x007b;
    const SPRCOLL_X_BASE: usize = 0x0fbc;
    const ARCHERY_GAME_HIT_COUNTER: usize = 0x0b88;
    const ARCHERY_GAME_ARROWS_LEFT: usize = 0x0b99;
    const ARCHERY_GAME_OUT_OF_ARROWS: usize = 0x0b9a;
    const ITEM_DROP_COUNTER: usize = 0x0b9b;
    const MINIGAME_CREDITS: usize = 0x04c4;
    const FLAG_FOR_BOOMERANG_IN_PLACE: usize = 0x35f;
    const ORANGE_BLUE_BARRIER_STATE: usize = 0x0c172;
    const SHARED_MESSAGE_TIMER: usize = 0x2cd;
    const ITEM_DROP_LUCK: usize = 0x0cf9;
    const LUCK_KILL_COUNTER: usize = 0x0cfa;
    const NUM_SPRITES_KILLED: usize = 0x0cfb;
    const DAMAGE_TYPE_DETERMINER: usize = 0x0cf2;
    const SET_WHEN_DAMAGING_ENEMIES: usize = 0x0047;
    const TIMES_HURT_BY_SPRITES: usize = 0x0cfc;
    const SELECT_FILE_COPY_SOURCE_SLOT_X2: usize = 0x00cc;
    const SELECT_FILE_NAME_SCROLL_X: usize = 0x0630;
    const SELECT_FILE_NAME_COLUMN: usize = 0x0b10;
    const SELECT_FILE_NAME_CURSOR_Y: usize = 0x0b11;
    const SELECT_FILE_NAME_SLOT: usize = 0x0b12;
    const SELECT_FILE_NAME_SCROLL_X_STEP: usize = 0x0b13;
    const SELECT_FILE_NAME_SCROLL_Y_STEP: usize = 0x0b14;
    const SELECT_FILE_NAME_ROW: usize = 0x0b15;
    const SELECT_FILE_NAME_SCROLL_X_DIRECTION: usize = 0x0b16;
    const ENDING_SCRATCH_PRIMARY: usize = 0x00c8;
    const ENDING_SCRATCH_SECONDARY: usize = 0x00ca;
    const SAVE_LOAD_SOURCE_OFFSET: usize = 0x0000;
    const DUNGEON_MAP_SCROLL_DRAW_OFFSET: usize = 0x0006;
    const DUNGEON_MAP_SCROLL_INPUT: usize = 0x000a;
    const DUNGEON_MAP_MARKER_X_OFFSET: usize = 0x0fa8;
    const DUNGEON_MAP_MARKER_Y_OFFSET: usize = 0x0faa;
    const DUNGEON_MAP_LOCATION_MARKER_BASE_Y: usize = 0x0cf5;
    const DUNGEON_SECRET_PENDING_KIND: usize = 0x0b9c;
    const OVERWORLD_SECRET_SUBST_CTR: usize = 0x0cf7;
    const SPRITE_OAM_PREP_X: usize = 0x0000;
    const SPRITE_OAM_PREP_Y: usize = 0x0002;
    const SPRITE_LOAD_BLOCK_SCRATCH: usize = 0x0000;
    const SPRITE_LAST_GARNISH_INDEX: usize = 0x000f;
    const BG1_H_SCROLL_COPY: usize = 0x0120;
    const BG1_V_SCROLL_COPY: usize = 0x0124;
    const BG2_H_SCROLL_COPY: usize = 0x011e;
    const BG2_V_SCROLL_COPY: usize = 0x0122;
    const BG3_H_SCROLL_COPY2: usize = 0x00e4;
    const BG3_V_SCROLL_COPY2: usize = 0x00ea;
    const BG2_H_SCROLL_COPY2_CACHED: usize = 0x0c180;
    const BG2_V_SCROLL_COPY2_CACHED: usize = 0x0c182;
    const MAP_BACKUP_BG1_H_SCROLL_COPY2: usize = 0x0c200;
    const MAP_BACKUP_BG2_H_SCROLL_COPY2: usize = 0x0c202;
    const MAP_BACKUP_BG1_V_SCROLL_COPY2: usize = 0x0c204;
    const MAP_BACKUP_BG2_V_SCROLL_COPY2: usize = 0x0c206;
    const BG2_V_SCROLL_COPY2_SPECIAL_EXIT: usize = 0x0c104;
    const BG2_H_SCROLL_COPY2_SPECIAL_EXIT: usize = 0x0c106;
    const BG2_V_SCROLL_COPY2_EXIT: usize = 0x0c144;
    const BG2_H_SCROLL_COPY2_EXIT: usize = 0x0c146;
    const BG1_H_SCROLL_SUBPIXEL: usize = 0x0620;
    const BG1_V_SCROLL_SUBPIXEL: usize = 0x0622;
    const MODE7_CENTER_X_COPY: usize = 0x0638;
    const MODE7_CENTER_Y_COPY: usize = 0x063a;
    const ATTRACT_VRAM_DST: usize = 0x0030;
    const ATTRACT_STORY_TEXT_POINTER: usize = 0x002d;
    const ATTRACT_BG2_VOFS_BACKUP: usize = 0x0020;
    const ATTRACT_NEXT_LEGEND_GFX: usize = 0x0026;
    const ATTRACT_LEGEND_FLAG: usize = 0x0027;
    const ATTRACT_PRISON_ZELDA_Y_BASE: usize = 0x002b;
    const ATTRACT_THRONE_FADE_TIMER: usize = 0x002c;
    const ATTRACT_ANIM_STEP_COUNTER: usize = 0x0032;
    const ATTRACT_SOLDIER_ANIM_STEP: usize = 0x0033;
    const ATTRACT_PRISON_SOLDIER_X_LO: usize = 0x0034;
    const ATTRACT_SCENE_FRAME_COUNTER: usize = 0x0050;
    const ATTRACT_FADE_IN_COMPLETE_FLAG: usize = 0x0052;
    const ATTRACT_SCENE_DONE_FLAG: usize = 0x005d;
    const ATTRACT_FADE_IN_DONE_FLAG: usize = 0x005f;
    const ATTRACT_SUBSTEP_DELAY_COUNTER: usize = 0x0061;
    const ATTRACT_MAIDEN_WARP_TIMER_A: usize = 0x0062;
    const ATTRACT_MAIDEN_WARP_TIMER_B: usize = 0x0063;
    const ATTRACT_LEGEND_CTR: usize = 0x0200;
    const TIMER_FOR_MODE7_ZOOM: usize = 0x0637;
    const OVERWORLD_PALETTE_MODE: usize = 0x0ab3;
    const INTRO_TIMES_PAL_FLASH: usize = 0x0ff9;
    const INTRO_STEP_INDEX: usize = 0x1e00;
    const INTRO_STEP_TIMER: usize = 0x1e01;
    const INTRO_FRAME_CTR: usize = 0x1e0a;
    const INTRO_DID_RUN_STEP: usize = 0x1f00;
    const TILEDETECT_DEEPWATER: usize = 0x0341;
    const TILEDETECT_NORMAL_TILES: usize = 0x0343;
    const TILEDETECT_THICK_GRASS: usize = 0x0357;
    const TILEDETECT_VERTICAL_LEDGE: usize = 0x036d;
    const DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ: usize = 0x036e;
    const LINK_PALETTE_BITS_OF_OAM: usize = 0x0346;
    const LINK_DMA_SOURCE_OFFSET: usize = 0x0c00f;
    const LINK_DMA_COUNTDOWN: usize = 0x0c013;
    const LINK_DMA_TILE_OFFSET: usize = 0x0c015;
    const AUX_BG_SUBSET_0: usize = 0x0c2f8;
    const AGAHNIM_PAL_SETTING: usize = 0x0c019;
    const PRIMARY_DECOMP_BUFFER_LOAD_GFX: usize = 0x14000;
    const SECONDARY_DECOMP_BUFFER_LOAD_GFX: usize = PRIMARY_DECOMP_BUFFER_LOAD_GFX + 0x600;
    const BG_DECOMP_BUFFER_LOAD_GFX: usize = 0x6000;
    const SPRITE_DECOMP_BUFFER_LOAD_GFX: usize = 0x7800;
    const GRAPHICS_DECOMP_BUFFER_END: usize = 0x9000;

    const SPRITE_STATE: usize = 0x0dd0;
    const SPRITE_TYPE: usize = 0x0e20;
    const SPRITE_Y_LO: usize = 0x0d00;
    const SPRITE_X_LO: usize = 0x0d10;
    const SPRITE_Y_HI: usize = 0x0d20;
    const SPRITE_X_HI: usize = 0x0d30;
    const SPRITE_Y_VELOCITY: usize = 0x0d40;
    const SPRITE_X_VELOCITY: usize = 0x0d50;
    const SPRITE_Y_SUBPIXEL: usize = 0x0d60;
    const SPRITE_X_SUBPIXEL: usize = 0x0d70;
    const SPRITE_AI_STATE: usize = 0x0d80;
    const SPRITE_A: usize = 0x0d90;
    const SPRITE_B: usize = 0x0da0;
    const SPRITE_C: usize = 0x0db0;
    const SPRITE_GRAPHICS: usize = 0x0dc0;
    const SPRITE_D: usize = 0x0de0;
    const SPRITE_DELAY_AUX1: usize = 0x0e00;
    const SPRITE_DELAY_MAIN: usize = 0x0df0;
    const SPRITE_FLAGS2: usize = 0x0e40;
    const SPRITE_HEALTH: usize = 0x0e50;
    const SPRITE_FLAGS3: usize = 0x0e60;
    const SPRITE_WALL_COLLISION: usize = 0x0e70;
    const SPRITE_ANIM_CLOCK: usize = 0x0ec0;
    const SPRITE_G: usize = 0x0ed0;
    const SPRITE_DELAY_AUX3: usize = 0x0ee0;
    const SPRITE_SUBTYPE2: usize = 0x0e80;
    const SPRITE_F: usize = 0x0ea0;
    const SPRITE_HEAD_DIR: usize = 0x0eb0;
    const SPRITE_HIT_TIMER: usize = 0x0ef0;
    const SPRITE_PAUSE: usize = 0x0f00;
    const SPRITE_FLOOR: usize = 0x0f20;
    const SPRITE_Y_RECOIL: usize = 0x0f30;
    const SPRITE_X_RECOIL: usize = 0x0f40;
    const SPRITE_OAM_FLAGS: usize = 0x0f50;
    const SPRITE_FLAGS4: usize = 0x0f60;
    const SPRITE_FLAGS5: usize = 0x0be0;
    const SPRITE_Z: usize = 0x0f70;
    const SPRITE_Z_VELOCITY: usize = 0x0f80;
    const SPRITE_Z_SUBPIXEL: usize = 0x0f90;
    const SPRITE_OBJ_PRIO: usize = 0x0b89;
    const SPRITE_STUNNED: usize = 0x0b58;
    const SPRITE_FLAGS: usize = 0x0b6b;
    const SPRITE_IGNORE_PROJECTILE: usize = 0x0ba0;
    const SPRITE_DRAW_WORK_BYTE_2: usize = 0x0bb0;
    const SPRITE_N: usize = 0x0bc0;
    const SPRITE_ROOM: usize = 0x0c9a;
    const SPRITE_DIE_ACTION: usize = 0x0cba;
    const SPRITE_DEFL_BITS: usize = 0x0caa;
    const SPRITE_BUMP_DAMAGE: usize = 0x0cd2;
    const SPRITE_INCOMING_DAMAGE: usize = 0x0ce2;
    const SPRITE_E: usize = 0x0e90;
    const SPRITE_SUBTYPE: usize = 0x0e30;
    const SPRITE_DELAY_AUX2: usize = 0x0e10;
    const SPRITE_DELAY_AUX4: usize = 0x0f10;
    const SPRITE_DRAW_I: usize = 0x1f9c2;
    const SPRITE_DRAW_WORK_BYTE_3: usize = 0x1fa1c;
    const SPRITE_DRAW_WORK_BYTE_4: usize = 0x1fa2c;
    const SPRITE_DRAW_WORK_BYTE_5: usize = 0x1fa3c;
    const SPRITE_DRAW_WORK_BYTE_1: usize = 0x1fa4c;

    const ANCILLA_Z_VELOCITY: usize = 0x0294;
    const ANCILLA_Z: usize = 0x029e;
    const ANCILLA_Z_SUBPIXEL_PLAYER: usize = 0x02a8;
    const ANCILLA_Y_LO: usize = 0x0bfa;
    const ANCILLA_X_LO: usize = 0x0c04;
    const ANCILLA_Y_HI: usize = 0x0c0e;
    const ANCILLA_X_HI: usize = 0x0c18;
    const ANCILLA_Y_VELOCITY: usize = 0x0c22;
    const ANCILLA_X_VELOCITY: usize = 0x0c2c;
    const ANCILLA_Y_SUBPIXEL: usize = 0x0c36;
    const ANCILLA_X_SUBPIXEL: usize = 0x0c40;
    const ANCILLA_TYPE: usize = 0x0c4a;
    const ANCILLA_ITEM_TO_LINK: usize = 0x0c5e;
    const ANCILLA_TIMER: usize = 0x0c68;
    const ANCILLA_DIRECTION: usize = 0x0c72;
    const ANCILLA_FLOOR: usize = 0x0c7c;
    const ANCILLA_OAM_IDX: usize = 0x0c86;
    const ANCILLA_NUMSPR: usize = 0x0c90;
    const ANCILLA_STEP: usize = 0x0c54;
    const ANCILLA_OBJPRIO: usize = 0x0280;
    const ANCILLA_U: usize = 0x028a;
    const ANCILLA_A: usize = 0x038a;
    const ANCILLA_B: usize = 0x038f;
    const ANCILLA_K: usize = 0x0380;
    const ANCILLA_H: usize = 0x03c5;
    const ANCILLA_R: usize = 0x03ea;
    const ANCILLA_AUX_TIMER: usize = 0x03b1;
    const ANCILLA_FLOOR2: usize = 0x03ca;
    const ANCILLA_WORK_BYTE_23: usize = 0x03cf;
    const ANCILLA_T_PLAYER: usize = 0x03d5;
    const ANCILLA_TILE_ATTRIBUTE: usize = 0x03e4;
    const ANCILLA_WORK_BYTE_3: usize = 0x039f;
    const ANCILLA_WORK_BYTE_1: usize = 0x03a4;
    const ANCILLA_S_PLAYER: usize = 0x03a9;
    const ANCILLA_L: usize = 0x0385;
    const ANCILLA_G: usize = 0x0394;
    const ANCILLA_WORK_BYTE_4: usize = 0x0bf0;
    const ANCILLA_WORK_BYTE_25: usize = 0x0746;
    const ANCILLA_WORK_BYTE_22: usize = 0x074b;
    const ANCILLA_WORK_BYTE_24: usize = 0x03db;
    const ANCILLA_WORK_BYTE_26: usize = 0x0741;
    const TAGALONG_DATA_INDEX: usize = 0x02cf;
    const TAGALONG_HOOKSHOT_INTERLOCK: usize = 0x02d0;
    const FOLLOWER_TAIL_WRITE_INDEX: usize = 0x02d3;
    const FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX: usize = 0x02d1;
    const TIMER_TAGALONG_REACQUIRE: usize = 0x02d2;
    const TAGALONG_APPEARANCE_NONE_FLAG: usize = 0x02f9;
    const TAGALONG_Y_LO: usize = 0x1a00;
    const TAGALONG_Y_HI: usize = 0x1a14;
    const TAGALONG_X_LO: usize = 0x1a28;
    const TAGALONG_X_HI: usize = 0x1a3c;
    const TAGALONG_Z: usize = 0x1a50;
    const TAGALONG_LAYERBITS: usize = 0x1a64;
    const FOLLOWER_INDICATOR: usize = 0x0f3cc;
    const FOLLOWER_SAVED_Y: usize = 0x0f3cd;
    const FOLLOWER_SAVED_X: usize = 0x0f3cf;
    const FOLLOWER_SAVED_INDOORS: usize = 0x0f3d1;
    const FOLLOWER_SAVED_FLOOR: usize = 0x0f3d2;
    const FOLLOWER_DROPPED: usize = 0x0f3d3;
    const FOLLOWER_JUMP_TIMER: usize = 0x02d6;
    const FOLLOWER_KIKI_ANIM_COUNTER: usize = 0x0b69;
    const FOLLOWER_PALETTE_SWAP_FLAG: usize = 0x0abd;
    const ZELDA_RESCUE_CUTSCENE_STATE: usize = 0x1fe01;

    const OVERLORD_X_LO: usize = 0x0b08;
    const OVERLORD_X_HI: usize = 0x0b10;
    const OVERLORD_Y_LO: usize = 0x0b18;
    const OVERLORD_Y_HI: usize = 0x0b20;
    const OVERLORD_GEN1: usize = 0x0b28;
    const OVERLORD_GEN2: usize = 0x0b30;
    const OVERLORD_GEN3: usize = 0x0b38;
    const OVERLORD_FLOOR: usize = 0x0b40;
    const OVERLORD_TYPE: usize = 0x0b00;
    const OVERLORD_SPAWNED_AREA: usize = 0x0cca;
    const OVERWORLD_SCROLL_DELTA: usize = 0x069e;
    const OVERWORLD_SPRITE_PRESENCE: usize = 0x1df80;
    const OVERWORLD_SPRITE_WAS_LOADED: usize = 0x1ef80;
    const OVERWORLD_MUSIC_TABLE: usize = 0x15b00;
    const OVERWORLD_SPRITE_GFX_TABLE: usize = 0x0fcc0;
    const OVERWORLD_SPRITE_PALETTE_TABLE: usize = 0x0fd40;
    const OVERWORLD_AREA_IS_BIG: usize = 0x0712;
    const OVERWORLD_AREA_IS_BIG_BACKUP: usize = 0x0714;
    const OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND: usize = 0x0716;
    const ROOM_BOUNDS: usize = 0x0600;
    const VRAM_UPLOAD_OFFSET: usize = 0x1000;
    const VRAM_UPLOAD_DATA: usize = 0x1002;
    const OVERWORLD_MAP16_DECODE_SRC: usize = 0x14000;
    const OVERWORLD_DECOMP_SCRATCH: usize = 0x14400;
    const POLY_CONFIG_COLOR_MODE: usize = 0x1f01;
    const POLY_CONFIG1: usize = 0x1f02;
    const POLY_WHICH_MODEL: usize = 0x1f03;
    const POLY_A: usize = 0x1f04;
    const POLY_B: usize = 0x1f05;
    const POLY_BASE_X: usize = 0x1f06;
    const POLY_BASE_Y: usize = 0x1f07;
    const POLY_SHAPE_DEPTH_BIAS: usize = 0x1f08;
    const POLY_CONFIG_NUM_VERTEX: usize = 0x1f3f;
    const POLY_CONFIG_NUM_POLYS: usize = 0x1f40;
    const POLY_FROMLUT_Z: usize = 0x1f45;
    const POLY_FROMLUT_Y: usize = 0x1f46;
    const POLY_FROMLUT_X: usize = 0x1f47;
    const POLY_F0: usize = 0x1f48;
    const POLY_F1: usize = 0x1f4a;
    const POLY_NUM_VERTEX_IN_POLY: usize = 0x1f4e;
    const POLY_RASTER_COLOR_CONFIG: usize = 0x1f4f;
    const POLY_TMP0: usize = 0x1fb0;
    const POLY_TMP2: usize = 0x1fbc;
    const POLYHEDRAL_BUFFER: usize = 0xe800;
    const POLY_PROJECTED_X: usize = 0x1f60;
    const POLY_PROJECTED_Y: usize = 0x1f88;
    const POLY_FACE_COORDS: usize = 0x1fc0;
    const POLY_X0_CUR: usize = 0x1fe1;
    const POLY_Y0_CUR: usize = 0x1fe2;
    const POLY_X0_TARGET: usize = 0x1fe3;
    const POLY_Y0_TRIG: usize = 0x1fe4;
    const POLY_X1_CUR: usize = 0x1fea;
    const POLY_Y1_CUR: usize = 0x1feb;
    const POLY_X1_TARGET: usize = 0x1fec;
    const POLY_Y1_TRIG: usize = 0x1fed;
    const POLY_TOTAL_NUM_STEPS: usize = 0x1fe0;
    const POLY_CUR_VERTEX_IDX0: usize = 0x1fe9;
    const POLY_CUR_VERTEX_IDX1: usize = 0x1ff2;
    const GARNISH_TYPE: usize = 0x1f800;
    const GARNISH_Y_LO: usize = 0x1f81e;
    const GARNISH_X_LO: usize = 0x1f83c;
    const GARNISH_Y_HI: usize = 0x1f85a;
    const GARNISH_X_HI: usize = 0x1f878;
    const GARNISH_Y_VELOCITY: usize = 0x1f896;
    const GARNISH_X_VELOCITY: usize = 0x1f8b4;
    const GARNISH_Y_SUBPIXEL: usize = 0x1f8d2;
    const GARNISH_X_SUBPIXEL: usize = 0x1f8f0;
    const GARNISH_COUNTDOWN: usize = 0x1f90e;
    const GARNISH_SPRITE: usize = 0x1f92c;
    const GARNISH_FLOOR: usize = 0x1f968;
    const GARNISH_OAM_FLAGS: usize = 0x1f9fe;
    const OAM_PRIORITY_VALUE: usize = 0x0064;
    const EXTENDED_OAM: usize = 0x0a00;
    const OAM_BUF: usize = 0x0800;
    const BYTEWISE_EXTENDED_OAM: usize = 0x0a20;
    const SORT_SPRITES_SETTING: usize = 0x0fb3;
    const DOOR_TYPE_AND_SLOT: usize = 0x1980;
    const DUNGEON_DOOR_DIRECTION: usize = 0x19c0;
    const DUNGEON_BG2_ATTR_TABLE: usize = 0x12000;
    const DUNGEON_BG1_ATTR_TABLE: usize = 0x13000;
    const INTRO_SPRITE_IS_INITED: usize = 0x1e10;
    const INTRO_SPRITE_SUBTYPE: usize = 0x1e18;
    const INTRO_SPRITE_STATE: usize = 0x1e20;
    const INTRO_X_SUBPIXEL: usize = 0x1e28;
    const INTRO_X_LO: usize = 0x1e30;
    const INTRO_X_HI: usize = 0x1e38;
    const INTRO_Y_SUBPIXEL: usize = 0x1e40;
    const INTRO_Y_LO: usize = 0x1e48;
    const INTRO_Y_HI: usize = 0x1e50;
    const INTRO_X_VEL: usize = 0x1e58;
    const INTRO_Y_VEL: usize = 0x1e60;
    const ALT_SPRITE_STATE: usize = 0x1d00;
    const ALT_SPRITE_TYPE: usize = 0x1d10;
    const ALT_SPRITE_X_LO: usize = 0x1d20;
    const ALT_SPRITE_X_HI: usize = 0x1d30;
    const ALT_SPRITE_Y_LO: usize = 0x1d40;
    const ALT_SPRITE_Y_HI: usize = 0x1d50;
    const ALT_SPRITE_GRAPHICS: usize = 0x1d60;
    const ALT_SPRITE_A: usize = 0x1d70;
    const ALT_SPRITE_HEAD_DIR: usize = 0x1d80;
    const ALT_SPRITE_OAM_FLAGS: usize = 0x1d90;
    const ALT_SPRITE_OBJ_PRIO: usize = 0x1da0;
    const ALT_SPRITE_D: usize = 0x1db0;
    const ALT_SPRITE_FLAGS2: usize = 0x1dc0;
    const ALT_SPRITE_FLOOR: usize = 0x1dd0;
    const ALT_SPRITE_SPAWNED_FLAG: usize = 0x1de0;
    const ALT_SPRITES_FLAG: usize = 0x0ffa;
    const ALT_SPRITE_FLAGS3: usize = 0x1df0;
    const ALT_SPRITE_B: usize = 0x1fa5c;
    const ALT_SPRITE_C: usize = 0x1fa6c;
    const ALT_SPRITE_E: usize = 0x1fa7c;
    const ALT_SPRITE_SUBTYPE2: usize = 0x1fa8c;
    const ALT_SPRITE_HEIGHT_ABOVE_SHADOW: usize = 0x1fa9c;
    const ALT_SPRITE_DELAY_MAIN: usize = 0x1faac;
    const ALT_SPRITE_I: usize = 0x1facc;
    const ALT_SPRITE_IGNORE_PROJECTILE: usize = 0x1fadc;
    const EFFECT_ANGLE_SCRATCH: usize = 0x15800;
    const QUAKE_BOLT_TIMER: usize = 0x15800;
    const QUAKE_BOLT_PHASE: usize = 0x15805;
    const QUAKE_ACTIVE_BOLT_LIMIT: usize = 0x1580a;
    const QUAKE_ORIGIN_Y: usize = 0x1580b;
    const QUAKE_ORIGIN_X: usize = 0x1580d;
    const QUAKE_PENDING_STEP: usize = 0x1580f;
    const QUAKE_SCREEN_SHAKE_Y: usize = 0x1581e;
    const BOMBOS_FIRE_COLUMN_TIMER: usize = 0x15800;
    const BOMBOS_FIRE_COLUMN_PHASE: usize = 0x15810;
    const BOMBOS_FIRE_COLUMN_RADIAL_ANGLE: usize = 0x15820;
    const BOMBOS_FIRE_COLUMN_Y_LO: usize = 0x15824;
    const BOMBOS_FIRE_COLUMN_Y_HI: usize = 0x15864;
    const BOMBOS_FIRE_COLUMN_X_LO: usize = 0x158a4;
    const BOMBOS_FIRE_COLUMN_X_HI: usize = 0x158e4;
    const BOMBOS_FIRE_COLUMN_SEED_Y: usize = 0x15924;
    const BOMBOS_FIRE_COLUMN_SEED_X: usize = 0x1592c;
    const BOMBOS_MODE: usize = 0x15934;
    const BOMBOS_BLAST_PHASE: usize = 0x15935;
    const BOMBOS_BLAST_TIMER: usize = 0x15945;
    const BOMBOS_BLAST_Y: usize = 0x15955;
    const BOMBOS_BLAST_X: usize = 0x159d5;
    const BOMBOS_BLAST_RELEASE_COUNTDOWN: usize = 0x15a55;
    const BOMBOS_BLAST_RELEASE_LOCKED: usize = 0x15a56;
    const BOMBOS_FIRE_COLUMN_RADIUS: usize = 0x15a57;
    const TOWER_SEAL_ORBIT_ANGLE: usize = 0x15800;
    const TOWER_SEAL_RING_RADIUS: usize = 0x15808;
    const TOWER_SEAL_CENTER_X: usize = 0x1580e;
    const TOWER_SEAL_CENTER_Y: usize = 0x15810;
    const TOWER_SEAL_WAIT_COUNTDOWN: usize = 0x15812;
    const TOWER_SEAL_BASE_SPARKLE_Y_LO: usize = 0x15817;
    const TOWER_SEAL_BASE_SPARKLE_Y_HI: usize = 0x1581f;
    const TOWER_SEAL_BASE_SPARKLE_X_LO: usize = 0x15827;
    const TOWER_SEAL_BASE_SPARKLE_X_HI: usize = 0x1582f;
    const TOWER_SEAL_SPARKLE_PHASE: usize = 0x15837;
    const TOWER_SEAL_SPARKLE_Y_LO: usize = 0x1584f;
    const TOWER_SEAL_SPARKLE_Y_HI: usize = 0x15867;
    const TOWER_SEAL_SPARKLE_X_LO: usize = 0x1587f;
    const TOWER_SEAL_SPARKLE_X_HI: usize = 0x15897;
    const TOWER_SEAL_SPARKLE_TIMER: usize = 0x158af;
    const BLAST_WALL_EXPLOSION_PHASE: usize = 0x10000;
    const BLAST_WALL_EXPLOSION_TIMER: usize = 0x10008;
    const BLAST_WALL_ENTRY_STATE: usize = 0x10010;
    const BLAST_WALL_SECONDARY_STATE: usize = 0x10011;
    const BLAST_WALL_CENTER_Y: usize = 0x10018;
    const BLAST_WALL_CENTER_X: usize = 0x1001a;
    const BLAST_WALL_DIRECTION: usize = 0x1001c;
    const BLAST_WALL_FRAGMENT_Y: usize = 0x10020;
    const BLAST_WALL_FRAGMENT_X: usize = 0x10030;
    const BLAST_WALL_FIREBALL_TIMER: usize = 0x10040;
    const SKULL_WOODS_FIRE_PHASE: usize = 0x10000;
    const SKULL_WOODS_FIRE_TIMER: usize = 0x10008;
    const SKULL_WOODS_FIRE_STARTED: usize = 0x10010;
    const SKULL_WOODS_FIRE_INNER_Y: usize = 0x10018;
    const SKULL_WOODS_FIRE_INNER_X: usize = 0x1001a;
    const SKULL_WOODS_FIRE_OUTER_Y: usize = 0x10026;
    const SKULL_WOODS_FIRE_OUTER_X: usize = 0x10036;
    const SKULL_WOODS_FIRE_Y: usize = 0x10020;
    const SKULL_WOODS_FIRE_X: usize = 0x10030;
    const HAPPINESS_POND_Y_VEL: usize = 0x15800;
    const HAPPINESS_POND_X_VEL: usize = 0x1580c;
    const HAPPINESS_POND_Z_VEL: usize = 0x15818;
    const HAPPINESS_POND_Y_LO: usize = 0x15824;
    const HAPPINESS_POND_Y_HI: usize = 0x15830;
    const HAPPINESS_POND_X_LO: usize = 0x1583c;
    const HAPPINESS_POND_X_HI: usize = 0x15848;
    const HAPPINESS_POND_Z: usize = 0x15854;
    const HAPPINESS_POND_TIMER: usize = 0x15860;
    const HAPPINESS_POND_ACTIVE: usize = 0x1586c;
    const HAPPINESS_POND_ITEM_TO_LINK: usize = 0x1587a;
    const HAPPINESS_POND_Y_SUBPIXEL: usize = 0x15886;
    const HAPPINESS_POND_X_SUBPIXEL: usize = 0x15892;
    const HAPPINESS_POND_Z_SUBPIXEL: usize = 0x1589e;
    const HAPPINESS_POND_STEP: usize = 0x158aa;
    const WEATHERVANE_Y_VELOCITY: usize = 0x15800;
    const WEATHERVANE_X_VELOCITY: usize = 0x1580c;
    const WEATHERVANE_Z_VELOCITY: usize = 0x15818;
    const WEATHERVANE_Y_LO: usize = 0x15824;
    const WEATHERVANE_Y_HI: usize = 0x15830;
    const WEATHERVANE_X_LO: usize = 0x1583c;
    const WEATHERVANE_X_HI: usize = 0x15848;
    const WEATHERVANE_Z: usize = 0x15854;
    const WEATHERVANE_ANIM_TIMER: usize = 0x15860;
    const WEATHERVANE_DRAW_STATE: usize = 0x1586c;
    const WEATHERVANE_SOURCE_SLOT: usize = 0x15878;
    const WEATHERVANE_OAM_OFFSET: usize = 0x15879;
    const WEATHERVANE_COUNTDOWN: usize = 0x158b6;
    const WEATHERVANE_MUSIC_LATCH: usize = 0x158b8;
    const BIRD_TRAVEL_X_LO: usize = 0x1ab0;
    const BIRD_TRAVEL_X_HI: usize = 0x1ac0;
    const BIRD_TRAVEL_Y_LO: usize = 0x1ad0;
    const BIRD_TRAVEL_Y_HI: usize = 0x1ae0;
    const MOLDORM_HISTORY_X_LO: usize = 0x1fc00;
    const MOLDORM_HISTORY_X_HI: usize = 0x1fc80;
    const MOLDORM_HISTORY_Y_LO: usize = 0x1fd00;
    const MOLDORM_HISTORY_Y_HI: usize = 0x1fd80;
    const SWAMOLA_HISTORY_X_LO: usize = 0x1fa5c;
    const SWAMOLA_HISTORY_X_HI: usize = 0x1fb1c;
    const SWAMOLA_HISTORY_Y_LO: usize = 0x1fbdc;
    const SWAMOLA_HISTORY_Y_HI: usize = 0x1fc9c;
    const SWAMOLA_TARGET_X_LO: usize = 0x1fd5c;
    const SWAMOLA_TARGET_X_HI: usize = 0x1fd62;
    const SWAMOLA_TARGET_Y_LO: usize = 0x1fd68;
    const SWAMOLA_TARGET_Y_HI: usize = 0x1fd6e;
    const BEAMOS_LASER_HISTORY_X_LO: usize = 0x1fd80;
    const BEAMOS_LASER_HISTORY_X_HI: usize = 0x1fe00;
    const BEAMOS_LASER_HISTORY_Y_LO: usize = 0x1fe80;
    const BEAMOS_LASER_HISTORY_Y_HI: usize = 0x1ff00;
    const DIGGING_GAME_PRIZE_SPAWNED: usize = BEAMOS_LASER_HISTORY_X_HI;
    const DIGGING_GAME_PRIZE_ATTEMPTS: usize = BEAMOS_LASER_HISTORY_X_HI + 1;
    const DRAW_SCRATCH_POSITION_X: usize = 0x0fa8;
    const DRAW_SCRATCH_POSITION_Y: usize = 0x0fa9;
    const DRAW_SCRATCH_FLAGS_HI: usize = 0x0fab;
    const HITBOX_SCRATCH_Y_OFFSET: usize = 0x0faa;
    const HITBOX_SCRATCH_X_OFFSET: usize = 0x0fab;
    const DIALOGUE_NUMBER_LO: usize = 0x1cf2;
    const DIALOGUE_NUMBER_HI: usize = 0x1cf3;
    const ENEMY_DAMAGE_DATA: usize = 0x16000;
    const PRIZE_DROP_CYCLE: usize = 0x0fc7;
    const DUAL_LAYER_TILE_CACHE: usize = 0x1fabc;

    const CACHED_SPRITE_LIVE_FIELDS: [usize; 24] = [
        SPRITE_STATE,
        SPRITE_TYPE,
        SPRITE_X_LO,
        SPRITE_X_HI,
        SPRITE_Y_LO,
        SPRITE_Y_HI,
        SPRITE_GRAPHICS,
        SPRITE_A,
        SPRITE_HEAD_DIR,
        SPRITE_OAM_FLAGS,
        SPRITE_OBJ_PRIO,
        SPRITE_D,
        SPRITE_FLAGS2,
        SPRITE_FLOOR,
        SPRITE_AI_STATE,
        SPRITE_FLAGS3,
        SPRITE_B,
        SPRITE_C,
        SPRITE_E,
        SPRITE_SUBTYPE2,
        SPRITE_Z,
        SPRITE_DELAY_MAIN,
        SPRITE_DRAW_I,
        SPRITE_IGNORE_PROJECTILE,
    ];

    const CACHED_SPRITE_ALT_FIELDS: [usize; 24] = [
        ALT_SPRITE_STATE,
        ALT_SPRITE_TYPE,
        ALT_SPRITE_X_LO,
        ALT_SPRITE_X_HI,
        ALT_SPRITE_Y_LO,
        ALT_SPRITE_Y_HI,
        ALT_SPRITE_GRAPHICS,
        ALT_SPRITE_A,
        ALT_SPRITE_HEAD_DIR,
        ALT_SPRITE_OAM_FLAGS,
        ALT_SPRITE_OBJ_PRIO,
        ALT_SPRITE_D,
        ALT_SPRITE_FLAGS2,
        ALT_SPRITE_FLOOR,
        ALT_SPRITE_SPAWNED_FLAG,
        ALT_SPRITE_FLAGS3,
        ALT_SPRITE_B,
        ALT_SPRITE_C,
        ALT_SPRITE_E,
        ALT_SPRITE_SUBTYPE2,
        ALT_SPRITE_HEIGHT_ABOVE_SHADOW,
        ALT_SPRITE_DELAY_MAIN,
        ALT_SPRITE_I,
        ALT_SPRITE_IGNORE_PROJECTILE,
    ];

    pub(crate) struct FrameControlView<'a> {
        ram: &'a [u8],
    }

    impl<'a> FrameControlView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn main_module(&self) -> u8 {
            byte(self.ram, MAIN_MODULE)
        }

        pub(crate) fn submodule(&self) -> u8 {
            byte(self.ram, SUBMODULE)
        }

        pub(crate) fn subsubmodule(&self) -> u8 {
            byte(self.ram, SUBSUBMODULE)
        }

        pub(crate) fn frame_counter(&self) -> u8 {
            byte(self.ram, FRAME_COUNTER)
        }

        pub(crate) fn saved_module_for_menu(&self) -> u8 {
            byte(self.ram, SAVED_MODULE_FOR_MENU)
        }

        pub(crate) fn modal_pause_flag(&self) -> u8 {
            byte(self.ram, MODAL_PAUSE_FLAG)
        }

        pub(crate) fn nmi_thread_active(&self) -> bool {
            byte(self.ram, NMI_THREAD_ACTIVE) != 0
        }

        pub(crate) fn selected_run_thread(&self) -> u8 {
            if self.nmi_thread_active() && word(self.ram, POLY_THREAD_STACK) != 0x1f31 {
                RUN_POLY_THREAD
            } else {
                RUN_MAIN_THREAD
            }
        }
    }

    pub(crate) struct FrameControlViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> FrameControlViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_main_module(&mut self, value: u8) {
            self.ram[MAIN_MODULE] = value;
        }

        pub(crate) fn set_submodule(&mut self, value: u8) {
            self.ram[SUBMODULE] = value;
        }

        pub(crate) fn set_subsubmodule(&mut self, value: u8) {
            self.ram[SUBSUBMODULE] = value;
        }

        pub(crate) fn increment_submodule(&mut self) {
            self.ram[SUBMODULE] = self.ram[SUBMODULE].wrapping_add(1);
        }

        pub(crate) fn decrement_submodule(&mut self) {
            self.ram[SUBMODULE] = self.ram[SUBMODULE].wrapping_sub(1);
        }

        pub(crate) fn increment_subsubmodule(&mut self) {
            self.ram[SUBSUBMODULE] = self.ram[SUBSUBMODULE].wrapping_add(1);
        }

        pub(crate) fn decrement_subsubmodule(&mut self) {
            self.ram[SUBSUBMODULE] = self.ram[SUBSUBMODULE].wrapping_sub(1);
        }

        pub(crate) fn set_frame_counter(&mut self, value: u8) {
            self.ram[FRAME_COUNTER] = value;
        }

        pub(crate) fn set_saved_module_for_menu(&mut self, value: u8) {
            self.ram[SAVED_MODULE_FOR_MENU] = value;
        }

        pub(crate) fn clear_saved_module_for_menu(&mut self) {
            self.set_saved_module_for_menu(0);
        }

        pub(crate) fn save_main_module_for_menu(&mut self) {
            self.ram[SAVED_MODULE_FOR_MENU] = self.ram[MAIN_MODULE];
        }

        pub(crate) fn save_submodule_for_menu(&mut self) {
            self.ram[SAVED_MODULE_FOR_MENU] = self.ram[SUBMODULE];
        }

        pub(crate) fn increment_frame_counter(&mut self) {
            self.ram[FRAME_COUNTER] = self.ram[FRAME_COUNTER].wrapping_add(1);
        }

        pub(crate) fn clear_modal_pause_flag(&mut self) {
            self.ram[MODAL_PAUSE_FLAG] = 0;
        }

        pub(crate) fn set_modal_pause_flag(&mut self, value: u8) {
            self.ram[MODAL_PAUSE_FLAG] = value;
        }

        pub(crate) fn increment_modal_pause_flag(&mut self) -> u8 {
            self.ram[MODAL_PAUSE_FLAG] = self.ram[MODAL_PAUSE_FLAG].wrapping_add(1);
            self.ram[MODAL_PAUSE_FLAG]
        }
    }

    pub(crate) struct SystemSignalsView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SystemSignalsView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn apui00_offset() -> usize {
            RAM_APUI00
        }

        pub(crate) fn music_control(&self) -> u8 {
            byte(self.ram, MUSIC_CONTROL)
        }

        pub(crate) fn current_music_control(&self) -> u8 {
            byte(self.ram, CURRENT_MUSIC_CONTROL)
        }

        pub(crate) fn last_music_control(&self) -> u8 {
            byte(self.ram, LAST_MUSIC_CONTROL)
        }

        pub(crate) fn queued_music_control(&self) -> u8 {
            byte(self.ram, QUEUED_MUSIC_CONTROL)
        }

        pub(crate) fn sound_effect_1(&self) -> u8 {
            byte(self.ram, SOUND_EFFECT_1)
        }

        pub(crate) fn sound_effect_2(&self) -> u8 {
            byte(self.ram, SOUND_EFFECT_2)
        }

        pub(crate) fn ambient_sound_effect(&self) -> u8 {
            byte(self.ram, SOUND_EFFECT_AMBIENT)
        }

        pub(crate) fn last_ambient_sound_effect(&self) -> u8 {
            byte(self.ram, SOUND_EFFECT_AMBIENT_LAST)
        }

        pub(crate) fn apui00(&self) -> u8 {
            byte(self.ram, RAM_APUI00)
        }

        pub(crate) fn has_sound_effect_1(&self) -> bool {
            self.sound_effect_1() != 0
        }

        pub(crate) fn has_sound_effect_2(&self) -> bool {
            self.sound_effect_2() != 0
        }

        pub(crate) fn ambient_sound_effect_is_clear(&self) -> bool {
            self.ambient_sound_effect() == 0
        }

        pub(crate) fn should_update_cgram(&self) -> bool {
            byte(self.ram, FLAG_UPDATE_CGRAM_IN_NMI) != 0
        }

        pub(crate) fn should_update_hud(&self) -> bool {
            byte(self.ram, FLAG_UPDATE_HUD_IN_NMI) != 0
        }

        pub(crate) fn game_over_check_flag(&self) -> u8 {
            byte(self.ram, GAME_OVER_CHECK_FLAG)
        }

        pub(crate) fn restart_check_flag(&self) -> u8 {
            byte(self.ram, RESTART_CHECK_FLAG)
        }
    }

    pub(crate) struct SystemSignalsViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SystemSignalsViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_music_control(&mut self, value: u8) {
            self.ram[MUSIC_CONTROL] = value;
        }

        pub(crate) fn set_current_music_control(&mut self, value: u8) {
            self.ram[CURRENT_MUSIC_CONTROL] = value;
        }

        pub(crate) fn set_last_music_control(&mut self, value: u8) {
            self.ram[LAST_MUSIC_CONTROL] = value;
        }

        pub(crate) fn set_queued_music_control(&mut self, value: u8) {
            self.ram[QUEUED_MUSIC_CONTROL] = value;
        }

        pub(crate) fn set_ambient_sound_effect(&mut self, value: u8) {
            self.ram[SOUND_EFFECT_AMBIENT] = value;
        }

        pub(crate) fn set_sound_effect_1(&mut self, value: u8) {
            self.ram[SOUND_EFFECT_1] = value;
        }

        pub(crate) fn set_sound_effect_2(&mut self, value: u8) {
            self.ram[SOUND_EFFECT_2] = value;
        }

        pub(crate) fn set_apui00(&mut self, value: u8) {
            self.ram[RAM_APUI00] = value;
        }

        pub(crate) fn set_sound_effect_1_word(&mut self, value: u16) {
            write_le_u16(self.ram, SOUND_EFFECT_1, value);
        }

        pub(crate) fn set_ambient_sound_effect_word(&mut self, value: u16) {
            write_le_u16(self.ram, SOUND_EFFECT_AMBIENT, value);
        }

        pub(crate) fn clear_sound_effect_1(&mut self) {
            self.set_sound_effect_1(0);
        }

        pub(crate) fn clear_sound_effect_2(&mut self) {
            self.set_sound_effect_2(0);
        }

        pub(crate) fn clear_ambient_sound_effect(&mut self) {
            self.set_ambient_sound_effect(0);
        }

        pub(crate) fn queue_sound_effect_1_if_empty(&mut self, value: u8) -> bool {
            if self.ram[SOUND_EFFECT_1] == 0 {
                self.ram[SOUND_EFFECT_1] = value;
                true
            } else {
                false
            }
        }

        pub(crate) fn queue_sound_effect_2_if_empty(&mut self, value: u8) -> bool {
            if self.ram[SOUND_EFFECT_2] == 0 {
                self.ram[SOUND_EFFECT_2] = value;
                true
            } else {
                false
            }
        }

        pub(crate) fn increment_hud_update_flag(&mut self) -> u8 {
            self.ram[FLAG_UPDATE_HUD_IN_NMI] = self.ram[FLAG_UPDATE_HUD_IN_NMI].wrapping_add(1);
            self.ram[FLAG_UPDATE_HUD_IN_NMI]
        }

        pub(crate) fn clear_hud_update_flag(&mut self) {
            self.ram[FLAG_UPDATE_HUD_IN_NMI] = 0;
        }

        pub(crate) fn increment_cgram_update_flag(&mut self) -> u8 {
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI]
        }

        pub(crate) fn clear_cgram_update_flag(&mut self) {
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = 0;
        }

        pub(crate) fn save_current_music_as_last(&mut self) {
            self.ram[LAST_MUSIC_CONTROL] = self.ram[CURRENT_MUSIC_CONTROL];
        }

        pub(crate) fn save_ambient_sound_effect_as_last(&mut self) {
            self.ram[SOUND_EFFECT_AMBIENT_LAST] = self.ram[SOUND_EFFECT_AMBIENT];
        }

        pub(crate) fn clear_game_over_check_flag(&mut self) {
            self.ram[GAME_OVER_CHECK_FLAG] = 0;
        }

        pub(crate) fn clear_restart_check_flag(&mut self) {
            self.ram[RESTART_CHECK_FLAG] = 0;
        }

        pub(crate) fn set_restart_check_flag(&mut self, value: u8) {
            self.ram[RESTART_CHECK_FLAG] = value;
        }

        pub(crate) fn set_raw_sfx_pan_value(&mut self, value: u8) {
            self.ram[RAW_SFX_PAN_VALUE] = value;
        }
    }

    pub(crate) struct DisplayNmiView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DisplayNmiView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn screen_brightness(&self) -> u8 {
            byte(self.ram, INIDISP_COPY)
        }

        pub(crate) fn bg_vram_load_mode(&self) -> u8 {
            byte(self.ram, NMI_LOAD_BG_FROM_VRAM)
        }

        pub(crate) fn has_bg_vram_load(&self) -> bool {
            self.bg_vram_load_mode() != 0
        }

        pub(crate) fn core_updates_disabled(&self) -> bool {
            byte(self.ram, NMI_DISABLE_CORE_UPDATES) != 0
        }

        pub(crate) fn core_update_disable_flag(&self) -> u8 {
            byte(self.ram, NMI_DISABLE_CORE_UPDATES)
        }

        pub(crate) fn subroutine_index(&self) -> u8 {
            byte(self.ram, NMI_SUBROUTINE_INDEX)
        }

        pub(crate) fn load_target_addr(&self) -> u8 {
            byte(self.ram, NMI_LOAD_TARGET_ADDR)
        }

        pub(crate) fn load_target_addr_word(&self) -> u16 {
            word(self.ram, NMI_LOAD_TARGET_ADDR)
        }

        pub(crate) fn main_screen_layers(&self) -> u8 {
            byte(self.ram, TM_COPY)
        }

        pub(crate) fn sub_screen_layers(&self) -> u8 {
            byte(self.ram, TS_COPY)
        }

        pub(crate) fn layer_masks_word(&self) -> u16 {
            word(self.ram, TM_COPY)
        }

        pub(crate) fn bg_mode(&self) -> u8 {
            byte(self.ram, BGMODE_COPY)
        }

        pub(crate) fn mosaic_copy(&self) -> u8 {
            byte(self.ram, MOSAIC_COPY)
        }

        pub(crate) fn hdma_enable_mask(&self) -> u8 {
            byte(self.ram, HDMAEN_COPY)
        }

        pub(crate) fn is_hdma_channel_enabled(&self, channel: usize) -> bool {
            self.hdma_enable_mask() & (1 << channel) != 0
        }

        pub(crate) fn mosaic_level(&self) -> u8 {
            byte(self.ram, MOSAIC_LEVEL)
        }

        pub(crate) fn nmi_copy_packets_flag(&self) -> u8 {
            byte(self.ram, NMI_COPY_PACKETS_FLAG)
        }

        pub(crate) fn has_nmi_copy_packets(&self) -> bool {
            self.nmi_copy_packets_flag() != 0
        }

        pub(crate) fn chr_halfslot_state(&self) -> u8 {
            byte(self.ram, LOAD_CHR_HALFSLOT_EVEN_ODD)
        }

        pub(crate) fn mosaic_target_level(&self) -> u8 {
            byte(self.ram, MOSAIC_TARGET_LEVEL)
        }

        pub(crate) fn nmi_boolean(&self) -> u8 {
            byte(self.ram, NMI_BOOLEAN)
        }

        pub(crate) fn is_nmi_thread_active(&self) -> bool {
            byte(self.ram, NMI_THREAD_ACTIVE) != 0
        }

        pub(crate) fn nmi_flag_update_polyhedral(&self) -> u8 {
            byte(self.ram, NMI_FLAG_UPDATE_POLYHEDRAL)
        }

        pub(crate) fn thread_other_stack(&self) -> u16 {
            word(self.ram, POLY_THREAD_STACK)
        }

        pub(crate) fn update_tilemap_dst(&self) -> u8 {
            byte(self.ram, NMI_UPDATE_TILEMAP_DST)
        }

        pub(crate) fn update_tilemap_src_data(&self) -> &[u8] {
            let offset = word(self.ram, NMI_UPDATE_TILEMAP_SRC) as usize;
            let start = super::nmi::BG_CHAR_BUFFER + offset;
            &self.ram[start.min(self.ram.len())..]
        }

        pub(crate) fn animated_tile_data_src(&self) -> u16 {
            word(self.ram, ANIMATED_TILE_DATA_SRC)
        }

        pub(crate) fn animated_tile_vram_addr(&self) -> u16 {
            word(self.ram, ANIMATED_TILE_VRAM_ADDR)
        }

        pub(crate) fn animated_tile_data(&self) -> &[u8] {
            let src = word(self.ram, ANIMATED_TILE_DATA_SRC) as usize;
            &self.ram[src.min(self.ram.len())..]
        }

        pub(crate) fn message_dma_dst_addr(&self) -> u16 {
            word(self.ram, super::messaging::MESSAGE_DMA_DST_ADDR)
        }

        pub(crate) fn hud_tile_indices_buffer(&self) -> &[u8] {
            &self.ram[HUD_TILE_INDICES_BUFFER..]
        }

        pub(crate) fn oam_buf(&self) -> &[u8] {
            &self.ram[OAM_BUF..]
        }

        pub(crate) fn tilemap_upload_buffer(&self) -> &[u8] {
            &self.ram[super::nmi::TILEMAP_UPLOAD_BUFFER..]
        }

        pub(crate) fn stripe_buffer_021b(&self) -> &[u8] {
            &self.ram[super::nmi::STRIPE_BUFFER_021B..]
        }

        pub(crate) fn vram_upload_tile_buf(&self) -> &[u8] {
            &self.ram[super::nmi::VRAM_UPLOAD_TILE_BUF..]
        }

        pub(crate) fn bg1_wall_top_buffer(&self) -> &[u8] {
            &self.ram[super::nmi::BG1_WALL_TOP_BUFFER..]
        }

        pub(crate) fn bg1_wall_bottom_buffer(&self) -> &[u8] {
            &self.ram[super::nmi::BG1_WALL_BOTTOM_BUFFER..]
        }

        pub(crate) fn bg_char_buffer(&self) -> &[u8] {
            &self.ram[super::nmi::BG_CHAR_BUFFER..]
        }

        pub(crate) fn bg_char_buffer_1(&self) -> &[u8] {
            &self.ram[super::nmi::BG_CHAR_BUFFER_1..]
        }

        pub(crate) fn bg_char_half_buffer(&self) -> &[u8] {
            &self.ram[super::nmi::BG_CHAR_HALF_BUFFER..]
        }

        pub(crate) fn game_over_text_buffer(&self) -> &[u8] {
            &self.ram[super::nmi::GAME_OVER_TEXT_BUFFER..]
        }

        pub(crate) fn game_over_text_tail_buffer(&self) -> &[u8] {
            &self.ram[super::nmi::GAME_OVER_TEXT_TAIL_BUFFER..]
        }

        pub(crate) fn polyhedral_buffer(&self) -> &[u8] {
            &self.ram[POLYHEDRAL_BUFFER..]
        }

        pub(crate) fn arbitrary_tilemap_dst(&self, slot: usize) -> u16 {
            word(
                self.ram,
                super::nmi::ARBITRARY_TILEMAP_DST_BUFFER + slot * 2,
            )
        }

        pub(crate) fn dungeon_bg2_attr_table(&self) -> &[u8] {
            &self.ram[DUNGEON_BG2_ATTR_TABLE..]
        }

        pub(crate) fn dungeon_bg1_attr_table(&self) -> &[u8] {
            &self.ram[DUNGEON_BG1_ATTR_TABLE..]
        }

        pub(crate) fn flag_travel_bird(&self) -> bool {
            byte(self.ram, FLAG_TRAVEL_BIRD) != 0
        }

        pub(crate) fn travel_bird_tile_offset(&self) -> u8 {
            byte(self.ram, FLAG_TRAVEL_BIRD)
        }

        pub(crate) fn w12sel_copy(&self) -> u8 {
            byte(self.ram, W12SEL_COPY)
        }

        pub(crate) fn w34sel_copy(&self) -> u8 {
            byte(self.ram, W34SEL_COPY)
        }

        pub(crate) fn wobjsel_copy(&self) -> u8 {
            byte(self.ram, WOBJSEL_COPY)
        }

        pub(crate) fn tmw_copy(&self) -> u8 {
            byte(self.ram, TMW_COPY)
        }

        pub(crate) fn tsw_copy(&self) -> u8 {
            byte(self.ram, TSW_COPY)
        }

        pub(crate) fn word_at(&self, addr: usize) -> u16 {
            word(self.ram, addr)
        }

        pub(crate) fn ram_slice_at(&self, addr: usize, len: usize) -> &[u8] {
            let start = addr.min(self.ram.len());
            let end = (addr + len).min(self.ram.len());
            &self.ram[start..end]
        }
    }

    pub(crate) struct DisplayNmiViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DisplayNmiViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_screen_brightness(&mut self, value: u8) {
            self.ram[INIDISP_COPY] = value;
        }

        pub(crate) fn increment_screen_brightness(&mut self) -> u8 {
            self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_add(1);
            self.ram[INIDISP_COPY]
        }

        pub(crate) fn decrement_screen_brightness(&mut self) -> u8 {
            self.ram[INIDISP_COPY] = self.ram[INIDISP_COPY].wrapping_sub(1);
            self.ram[INIDISP_COPY]
        }

        pub(crate) fn set_bg_vram_load_mode(&mut self, value: u8) {
            self.ram[NMI_LOAD_BG_FROM_VRAM] = value;
        }

        pub(crate) fn clear_bg_vram_load_mode(&mut self) {
            self.set_bg_vram_load_mode(0);
        }

        pub(crate) fn set_core_update_disable_flag(&mut self, value: u8) {
            self.ram[NMI_DISABLE_CORE_UPDATES] = value;
        }

        pub(crate) fn set_core_update_disable_flag_word(&mut self, value: u16) {
            write_le_u16(self.ram, NMI_DISABLE_CORE_UPDATES, value);
        }

        pub(crate) fn clear_core_update_disable_flag(&mut self) {
            self.set_core_update_disable_flag(0);
        }

        pub(crate) fn increment_core_update_disable_flag(&mut self) -> u8 {
            self.ram[NMI_DISABLE_CORE_UPDATES] = self.ram[NMI_DISABLE_CORE_UPDATES].wrapping_add(1);
            self.ram[NMI_DISABLE_CORE_UPDATES]
        }

        pub(crate) fn set_subroutine_index(&mut self, value: u8) {
            self.ram[NMI_SUBROUTINE_INDEX] = value;
        }

        pub(crate) fn clear_subroutine_index(&mut self) {
            self.set_subroutine_index(0);
        }

        pub(crate) fn take_subroutine_index(&mut self) -> u8 {
            let subroutine_index = self.ram[NMI_SUBROUTINE_INDEX];
            self.ram[NMI_SUBROUTINE_INDEX] = 0;
            subroutine_index
        }

        pub(crate) fn set_load_target_addr(&mut self, value: u8) {
            self.ram[NMI_LOAD_TARGET_ADDR] = value;
        }

        pub(crate) fn set_load_target_addr_word(&mut self, value: u16) {
            write_le_u16(self.ram, NMI_LOAD_TARGET_ADDR, value);
        }

        pub(crate) fn set_main_screen_layers(&mut self, value: u8) {
            self.ram[TM_COPY] = value;
        }

        pub(crate) fn and_main_screen_layers(&mut self, value: u8) {
            self.ram[TM_COPY] &= value;
        }

        pub(crate) fn or_main_screen_layers(&mut self, value: u8) {
            self.ram[TM_COPY] |= value;
        }

        pub(crate) fn set_sub_screen_layers(&mut self, value: u8) {
            self.ram[TS_COPY] = value;
        }

        pub(crate) fn clear_sub_screen_layers_word(&mut self) {
            write_le_u16(self.ram, TS_COPY, 0);
        }

        pub(crate) fn and_sub_screen_layers(&mut self, value: u8) {
            self.ram[TS_COPY] &= value;
        }

        pub(crate) fn or_sub_screen_layers(&mut self, value: u8) {
            self.ram[TS_COPY] |= value;
        }

        pub(crate) fn set_layer_masks_word(&mut self, value: u16) {
            write_le_u16(self.ram, TM_COPY, value);
        }

        pub(crate) fn set_bg_mode(&mut self, value: u8) {
            self.ram[BGMODE_COPY] = value;
        }

        pub(crate) fn set_mosaic_copy(&mut self, value: u8) {
            self.ram[MOSAIC_COPY] = value;
        }

        pub(crate) fn set_mosaic_copy_from_level_or(&mut self, mask: u8) {
            self.ram[MOSAIC_COPY] = self.ram[MOSAIC_LEVEL] | mask;
        }

        pub(crate) fn set_hdma_enable_mask(&mut self, value: u8) {
            self.ram[HDMAEN_COPY] = value;
        }

        pub(crate) fn clear_hdma_enable_mask(&mut self) {
            self.ram[HDMAEN_COPY] = 0;
        }

        pub(crate) fn set_mosaic_level(&mut self, value: u8) {
            self.ram[MOSAIC_LEVEL] = value;
        }

        pub(crate) fn clear_mosaic_level(&mut self) {
            self.ram[MOSAIC_LEVEL] = 0;
        }

        pub(crate) fn clear_mosaic_level_word(&mut self) {
            write_le_u16(self.ram, MOSAIC_LEVEL, 0);
        }

        pub(crate) fn set_mosaic_target_level(&mut self, value: u8) {
            self.ram[MOSAIC_TARGET_LEVEL] = value;
        }

        pub(crate) fn set_mosaic_target_level_word(&mut self, value: u16) {
            write_le_u16(self.ram, MOSAIC_TARGET_LEVEL, value);
        }

        pub(crate) fn clear_mosaic_target_level(&mut self) {
            self.ram[MOSAIC_TARGET_LEVEL] = 0;
        }

        pub(crate) fn clear_mosaic_target_level_word(&mut self) {
            write_le_u16(self.ram, MOSAIC_TARGET_LEVEL, 0);
        }

        pub(crate) fn set_nmi_copy_packets_flag(&mut self, value: u8) {
            self.ram[NMI_COPY_PACKETS_FLAG] = value;
        }

        pub(crate) fn request_nmi_copy_packets(&mut self) {
            self.ram[NMI_COPY_PACKETS_FLAG] = 1;
        }

        pub(crate) fn clear_nmi_copy_packets_flag(&mut self) {
            self.ram[NMI_COPY_PACKETS_FLAG] = 0;
        }

        pub(crate) fn set_chr_halfslot_state(&mut self, value: u8) {
            self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = value;
        }

        pub(crate) fn clear_chr_halfslot_state(&mut self) {
            self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 0;
        }

        pub(crate) fn set_nmi_boolean(&mut self, value: u8) {
            self.ram[NMI_BOOLEAN] = value;
        }

        pub(crate) fn set_nmi_flag_update_polyhedral(&mut self, value: u8) {
            self.ram[NMI_FLAG_UPDATE_POLYHEDRAL] = value;
        }

        pub(crate) fn clear_nmi_flag_update_polyhedral(&mut self) {
            self.ram[NMI_FLAG_UPDATE_POLYHEDRAL] = 0;
        }

        pub(crate) fn set_thread_other_stack(&mut self, value: u16) {
            write_le_u16(self.ram, POLY_THREAD_STACK, value);
        }

        pub(crate) fn clear_update_tilemap_dst(&mut self) {
            self.ram[NMI_UPDATE_TILEMAP_DST] = 0;
        }

        pub(crate) fn increment_chr_halfslot_state(&mut self) -> u8 {
            self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] =
                self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD].wrapping_add(1);
            self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD]
        }

        pub(crate) fn increment_mosaic_level_by(&mut self, value: u8) -> u8 {
            self.ram[MOSAIC_LEVEL] = self.ram[MOSAIC_LEVEL].wrapping_add(value);
            self.ram[MOSAIC_LEVEL]
        }

        pub(crate) fn decrement_mosaic_level_by(&mut self, value: u8) -> u8 {
            self.ram[MOSAIC_LEVEL] = self.ram[MOSAIC_LEVEL].wrapping_sub(value);
            self.ram[MOSAIC_LEVEL]
        }

        pub(crate) fn set_overworld_fixed_color_plusminus(&mut self, value: u8) {
            self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = value;
        }

        pub(crate) fn set_virq_trigger(&mut self, value: u8) {
            self.ram[VIRQ_TRIGGER] = value;
        }

        pub(crate) fn set_dma_head_pointer(&mut self, value: u8) {
            self.ram[DMA_HEAD_POINTER] = value;
        }

        pub(crate) fn set_dma_body_pointer(&mut self, value: u8) {
            self.ram[DMA_BODY_POINTER] = value;
        }

        pub(crate) fn set_nmi_thread_active(&mut self, value: u8) {
            self.ram[NMI_THREAD_ACTIVE] = value;
        }

        pub(crate) fn set_irq_flag(&mut self, value: u8) {
            self.ram[IRQ_FLAG] = value;
        }

        pub(crate) fn set_w12sel_copy(&mut self, value: u8) {
            self.ram[W12SEL_COPY] = value;
        }

        pub(crate) fn set_w34sel_copy(&mut self, value: u8) {
            self.ram[W34SEL_COPY] = value;
        }

        pub(crate) fn set_wobjsel_copy(&mut self, value: u8) {
            self.ram[WOBJSEL_COPY] = value;
        }

        pub(crate) fn set_tmw_copy(&mut self, value: u8) {
            self.ram[TMW_COPY] = value;
        }

        pub(crate) fn set_tsw_copy(&mut self, value: u8) {
            self.ram[TSW_COPY] = value;
        }
    }

    pub(crate) struct PlayerStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PlayerStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn x(&self) -> u16 {
            word(self.ram, LINK_X_COORD)
        }

        pub(crate) fn y(&self) -> u16 {
            word(self.ram, LINK_Y_COORD)
        }

        pub(crate) fn z(&self) -> u16 {
            word(self.ram, LINK_Z_COORD)
        }

        pub(crate) fn z_low(&self) -> u8 {
            byte(self.ram, LINK_Z_COORD)
        }

        pub(crate) fn z_low_signed(&self) -> i8 {
            self.z_low() as i8
        }

        pub(crate) fn is_z_low_negative(&self) -> bool {
            self.z_low_signed().is_negative()
        }

        pub(crate) fn z_mirror(&self) -> u16 {
            word(self.ram, LINK_Z_COORD_MIRROR)
        }

        pub(crate) fn z_mirror_low(&self) -> u8 {
            byte(self.ram, LINK_Z_COORD_MIRROR)
        }

        pub(crate) fn z_mirror_delta_low(&self) -> u8 {
            self.z_mirror_low().wrapping_sub(self.z_low())
        }

        pub(crate) fn is_landing_at_or_above_ground(&self) -> bool {
            self.z() >= 0xfff0
        }

        pub(crate) fn is_low_z_landing_at_or_above_ground(&self) -> bool {
            self.z_low() >= 0xf0
        }

        pub(crate) fn is_recoil_landing_z_window(&self) -> bool {
            ((self.z_low() & 0xfe) as i8) <= 0
        }

        pub(crate) fn should_probe_recoil_landing_tile(&self) -> bool {
            self.z_low() == 0 || self.z_low() >= 0xe0
        }

        pub(crate) fn z_for_oam(&self) -> u8 {
            if self.z() < 0x8000 || byte(self.ram, LINK_Z_COORD) < 0xf0 {
                byte(self.ram, LINK_Z_COORD)
            } else {
                0
            }
        }

        pub(crate) fn is_grounded_or_z_sentinel(&self) -> bool {
            matches!(byte(self.ram, LINK_Z_COORD), 0 | 0xff)
        }

        pub(crate) fn cached_x(&self) -> u16 {
            word(self.ram, LINK_X_COORD_CACHED)
        }

        pub(crate) fn cached_y(&self) -> u16 {
            word(self.ram, LINK_Y_COORD_CACHED)
        }

        pub(crate) fn oam_x_offset(&self) -> u8 {
            byte(self.ram, PLAYER_OAM_X_OFFSET)
        }

        pub(crate) fn oam_y_offset(&self) -> u8 {
            byte(self.ram, PLAYER_OAM_Y_OFFSET)
        }

        pub(crate) fn oam_x_offset_signed(&self) -> i8 {
            self.oam_x_offset() as i8
        }

        pub(crate) fn oam_y_offset_signed(&self) -> i8 {
            self.oam_y_offset() as i8
        }

        pub(crate) fn has_disabled_oam_offsets(&self) -> bool {
            self.oam_y_offset() == 0x80
        }

        pub(crate) fn x_high(&self) -> u8 {
            byte(self.ram, LINK_X_COORD + 1)
        }

        pub(crate) fn y_high(&self) -> u8 {
            byte(self.ram, LINK_Y_COORD + 1)
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, LINK_X_COORD)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, LINK_Y_COORD)
        }

        pub(crate) fn safe_return_x_high(&self) -> u8 {
            byte(self.ram, LINK_X_COORD_SAFE_RETURN_HI)
        }

        pub(crate) fn safe_return_y_high(&self) -> u8 {
            byte(self.ram, LINK_Y_COORD_SAFE_RETURN_HI)
        }

        pub(crate) fn safe_return_y_low(&self) -> u8 {
            byte(self.ram, LINK_Y_COORD_SAFE_RETURN_LO)
        }

        pub(crate) fn y_low_delta_from_safe_return(&self) -> u8 {
            byte(self.ram, LINK_Y_COORD).wrapping_sub(byte(self.ram, LINK_Y_COORD_SAFE_RETURN_LO))
        }

        pub(crate) fn safe_return_x(&self) -> u16 {
            byte(self.ram, LINK_X_COORD_SAFE_RETURN_LO) as u16
                | ((byte(self.ram, LINK_X_COORD_SAFE_RETURN_HI) as u16) << 8)
        }

        pub(crate) fn safe_return_y(&self) -> u16 {
            byte(self.ram, LINK_Y_COORD_SAFE_RETURN_LO) as u16
                | ((byte(self.ram, LINK_Y_COORD_SAFE_RETURN_HI) as u16) << 8)
        }

        pub(crate) fn hop_origin_coord(&self) -> u16 {
            word(self.ram, LINK_Y_COORD_ORIGINAL)
        }

        pub(crate) fn copied_x(&self) -> u16 {
            word(self.ram, LINK_X_COORD_COPY)
        }

        pub(crate) fn copied_y(&self) -> u16 {
            word(self.ram, LINK_Y_COORD_COPY)
        }

        pub(crate) fn temp_bunny_timer(&self) -> u16 {
            word(self.ram, LINK_TIMER_TEMPBUNNY)
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            byte(self.ram, LINK_X_VELOCITY)
        }

        pub(crate) fn x_velocity_signed(&self) -> i8 {
            self.x_velocity() as i8
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            byte(self.ram, LINK_Y_VELOCITY)
        }

        pub(crate) fn y_velocity_signed(&self) -> i8 {
            self.y_velocity() as i8
        }

        pub(crate) fn x_subpixel(&self) -> u8 {
            byte(self.ram, LINK_X_SUBPIXEL)
        }

        pub(crate) fn y_subpixel(&self) -> u8 {
            byte(self.ram, LINK_Y_SUBPIXEL)
        }

        pub(crate) fn x_page_movement_delta(&self) -> u8 {
            byte(self.ram, LINK_X_PAGE_MOVEMENT_DELTA)
        }

        pub(crate) fn y_page_movement_delta(&self) -> u8 {
            byte(self.ram, LINK_Y_PAGE_MOVEMENT_DELTA)
        }

        pub(crate) fn x_page_movement_delta_signed(&self) -> i8 {
            self.x_page_movement_delta() as i8
        }

        pub(crate) fn y_page_movement_delta_signed(&self) -> i8 {
            self.y_page_movement_delta() as i8
        }

        pub(crate) fn z_velocity(&self) -> u8 {
            byte(self.ram, LINK_Z_VELOCITY)
        }

        pub(crate) fn actual_x_velocity(&self) -> u8 {
            byte(self.ram, LINK_ACTUAL_X_VELOCITY)
        }

        pub(crate) fn actual_x_velocity_signed(&self) -> i8 {
            self.actual_x_velocity() as i8
        }

        pub(crate) fn actual_y_velocity(&self) -> u8 {
            byte(self.ram, LINK_ACTUAL_Y_VELOCITY)
        }

        pub(crate) fn actual_y_velocity_signed(&self) -> i8 {
            self.actual_y_velocity() as i8
        }

        pub(crate) fn actual_z_velocity(&self) -> u8 {
            byte(self.ram, LINK_Z_VELOCITY)
        }

        pub(crate) fn actual_z_velocity_copy(&self) -> u8 {
            byte(self.ram, LINK_Z_VELOCITY_COPY)
        }

        pub(crate) fn actual_z_velocity_mirror(&self) -> u8 {
            byte(self.ram, LINK_Z_VELOCITY_MIRROR)
        }

        pub(crate) fn recoil_z_velocity_for_dungeon_reset(&self) -> u8 {
            byte(self.ram, LINK_RECOIL_Z_VELOCITY_DUNGEON)
        }

        pub(crate) fn recoil_timer(&self) -> u8 {
            byte(self.ram, LINK_RECOIL_TIMER)
        }

        pub(crate) fn direction(&self) -> u8 {
            byte(self.ram, LINK_DIRECTION)
        }

        pub(crate) fn direction_lock(&self) -> u8 {
            byte(self.ram, LINK_CANT_CHANGE_DIRECTION)
        }

        pub(crate) fn direction_lock_has(&self, mask: u8) -> bool {
            byte(self.ram, LINK_CANT_CHANGE_DIRECTION) & mask != 0
        }

        pub(crate) fn moving_against_diag_tile(&self) -> u8 {
            byte(self.ram, LINK_MOVING_AGAINST_DIAG_TILE)
        }

        pub(crate) fn is_moving_against_diag_tile_on_both_axes(&self) -> bool {
            self.moving_against_diag_tile() & 0x0c != 0 && self.moving_against_diag_tile() & 3 != 0
        }

        pub(crate) fn has_swim_axis_drag(&self) -> bool {
            (byte(self.ram, LINK_NUM_ORTHOGONAL_DIRECTIONS)
                | byte(self.ram, LINK_MOVING_AGAINST_DIAG_TILE))
                != 0
        }

        pub(crate) fn num_orthogonal_directions(&self) -> u8 {
            byte(self.ram, LINK_NUM_ORTHOGONAL_DIRECTIONS)
        }

        pub(crate) fn last_direction_moved_towards(&self) -> u8 {
            byte(self.ram, LINK_LAST_DIRECTION_MOVED_TOWARDS)
        }

        pub(crate) fn last_direction_moved_towards_index(&self) -> usize {
            usize::from(self.last_direction_moved_towards())
        }

        pub(crate) fn last_direction(&self) -> u8 {
            byte(self.ram, LINK_LAST_DIRECTION)
        }

        pub(crate) fn facing(&self) -> u8 {
            byte(self.ram, LINK_FACING)
        }

        pub(crate) fn has_facing(&self) -> bool {
            byte(self.ram, LINK_FACING) != 0
        }

        pub(crate) fn facing_index(&self) -> usize {
            usize::from(byte(self.ram, LINK_FACING) >> 1)
        }

        pub(crate) fn facing_mirror_index(&self) -> usize {
            usize::from(byte(self.ram, LINK_FACING_MIRROR) >> 1)
        }

        pub(crate) fn handler_state(&self) -> u8 {
            byte(self.ram, LINK_HANDLER_STATE)
        }

        pub(crate) fn is_edge_transition_blocked_by_handler_state(&self) -> bool {
            matches!(byte(self.ram, LINK_HANDLER_STATE), 3 | 8 | 9 | 10)
        }

        pub(crate) fn auxiliary_state(&self) -> u8 {
            byte(self.ram, LINK_AUXILIARY_STATE)
        }

        pub(crate) fn is_in_auxiliary_state(&self, value: u8) -> bool {
            byte(self.ram, LINK_AUXILIARY_STATE) == value
        }

        pub(crate) fn has_auxiliary_state(&self) -> bool {
            byte(self.ram, LINK_AUXILIARY_STATE) != 0
        }

        pub(crate) fn incapacitated_timer(&self) -> u8 {
            byte(self.ram, LINK_INCAPACITATED_TIMER)
        }

        pub(crate) fn is_in_deep_water(&self) -> bool {
            byte(self.ram, LINK_IS_IN_DEEP_WATER) != 0
        }

        pub(crate) fn deep_water_state(&self) -> u8 {
            byte(self.ram, LINK_IS_IN_DEEP_WATER)
        }

        pub(crate) fn flag_moving(&self) -> u8 {
            byte(self.ram, LINK_FLAG_MOVING)
        }

        pub(crate) fn swim_direction_flags(&self) -> u8 {
            byte(self.ram, SWIM_PLAYER_DIRECTION_FLAGS)
        }

        pub(crate) fn hard_swim_stroke(&self) -> u8 {
            byte(self.ram, LINK_SWIM_HARD_STROKE)
        }

        pub(crate) fn is_running(&self) -> bool {
            byte(self.ram, LINK_IS_RUNNING) != 0
        }

        pub(crate) fn running_state(&self) -> u8 {
            byte(self.ram, LINK_IS_RUNNING)
        }

        pub(crate) fn speed_setting(&self) -> u8 {
            byte(self.ram, LINK_SPEED_SETTING)
        }

        pub(crate) fn speed_modifier(&self) -> u8 {
            byte(self.ram, LINK_SPEED_MODIFIER)
        }

        pub(crate) fn dash_counter(&self) -> u8 {
            byte(self.ram, LINK_DASH_COUNTER)
        }

        pub(crate) fn quadrant_x(&self) -> u8 {
            byte(self.ram, LINK_QUADRANT_X)
        }

        pub(crate) fn quadrant_y(&self) -> u8 {
            byte(self.ram, LINK_QUADRANT_Y)
        }

        pub(crate) fn quadrant_visit_index(&self, fullsize_y: u8, fullsize_x: u8) -> usize {
            ((fullsize_y as usize) << 2)
                + ((fullsize_x as usize) << 1)
                + self.quadrant_y() as usize
                + self.quadrant_x() as usize
        }

        pub(crate) fn quadrant_x_mask(&self) -> u8 {
            if self.quadrant_x() != 0 {
                2
            } else {
                1
            }
        }

        pub(crate) fn quadrant_y_mask(&self) -> u8 {
            if self.quadrant_y() != 0 {
                8
            } else {
                4
            }
        }

        pub(crate) fn dash_countdown(&self) -> u8 {
            byte(self.ram, LINK_COUNTDOWN_FOR_DASH)
        }

        pub(crate) fn jump_ledge_timer(&self) -> u8 {
            byte(self.ram, LINK_TIMER_JUMP_LEDGE)
        }

        pub(crate) fn immobilized_flag(&self) -> u8 {
            byte(self.ram, FLAG_IS_LINK_IMMOBILIZED)
        }

        pub(crate) fn is_immobilized(&self) -> bool {
            self.immobilized_flag() != 0
        }

        pub(crate) fn menu_block_flag(&self) -> u8 {
            byte(self.ram, FLAG_BLOCK_LINK_MENU)
        }

        pub(crate) fn is_menu_blocked(&self) -> bool {
            self.menu_block_flag() != 0
        }

        pub(crate) fn has_menu_block_flag(&self, value: u8) -> bool {
            self.menu_block_flag() == value
        }

        pub(crate) fn push_fatigue_timer(&self) -> u8 {
            byte(self.ram, LINK_TIMER_PUSH_GET_TIRED)
        }

        pub(crate) fn palette_bits_of_oam(&self) -> u8 {
            byte(self.ram, LINK_PALETTE_BITS_OF_OAM)
        }

        pub(crate) fn palette_bits_of_oam_word(&self) -> u16 {
            read_le_u16(self.ram, LINK_PALETTE_BITS_OF_OAM)
        }

        pub(crate) fn visibility_status(&self) -> u8 {
            byte(self.ram, LINK_VISIBILITY_STATUS)
        }

        pub(crate) fn electrocute_on_touch(&self) -> u8 {
            byte(self.ram, LINK_ELECTROCUTE_ON_TOUCH)
        }

        pub(crate) fn is_cape_active(&self) -> bool {
            byte(self.ram, LINK_CAPE_MODE) != 0
        }

        pub(crate) fn sprite_damage_disable_timer(&self) -> u8 {
            byte(self.ram, LINK_DISABLE_SPRITE_DAMAGE)
        }

        pub(crate) fn sprite_oam_state_timer(&self) -> u8 {
            byte(self.ram, LINK_SPRITE_OAM_STATE_TIMER)
        }

        pub(crate) fn action_handler_timer(&self) -> u8 {
            byte(self.ram, PLAYER_HANDLER_TIMER)
        }

        pub(crate) fn doorway_state(&self) -> u8 {
            byte(self.ram, IS_STANDING_IN_DOORWAY)
        }

        pub(crate) fn blink_countdown(&self) -> u8 {
            byte(self.ram, COUNTDOWN_FOR_BLINK)
        }

        pub(crate) fn item_receipt_method(&self) -> u8 {
            byte(self.ram, ITEM_RECEIPT_METHOD)
        }

        pub(crate) fn ancilla_pickup_flag(&self) -> u8 {
            byte(self.ram, FLAG_IS_ANCILLA_TO_PICK_UP)
        }

        pub(crate) fn sprite_pickup_flag(&self) -> u8 {
            byte(self.ram, FLAG_IS_SPRITE_TO_PICK_UP)
        }

        pub(crate) fn sprite_pickup_flag_cached(&self) -> u8 {
            byte(self.ram, FLAG_IS_SPRITE_TO_PICK_UP_CACHED)
        }

        pub(crate) fn spin_attack_delay_timer(&self) -> u8 {
            byte(self.ram, LINK_DELAY_TIMER_SPIN_ATTACK)
        }

        pub(crate) fn sword_delay_timer(&self) -> u8 {
            byte(self.ram, LINK_SWORD_DELAY_TIMER)
        }

        pub(crate) fn spin_attack_step_counter(&self) -> u8 {
            byte(self.ram, LINK_SPIN_ATTACK_STEP_COUNTER)
        }

        pub(crate) fn spin_animation_step_counter(&self) -> u8 {
            byte(self.ram, STEP_COUNTER_FOR_SPIN_ATTACK)
        }

        pub(crate) fn spin_offsets(&self) -> u8 {
            byte(self.ram, LINK_SPIN_OFFSETS)
        }

        pub(crate) fn given_damage(&self) -> u8 {
            byte(self.ram, LINK_GIVE_DAMAGE)
        }

        pub(crate) fn needs_transform_poof(&self) -> bool {
            byte(self.ram, LINK_NEED_FOR_POOF_FOR_TRANSFORM) != 0
        }

        pub(crate) fn hookshot_grave_latch(&self) -> bool {
            byte(self.ram, LINK_SOMETHING_WITH_HOOKSHOT) != 0
        }

        pub(crate) fn hookshot_interlock(&self) -> u8 {
            byte(self.ram, RELATED_TO_HOOKSHOT)
        }

        pub(crate) fn has_hookshot_interlock(&self) -> bool {
            self.hookshot_interlock() != 0
        }

        pub(crate) fn hookshot_interlock_has(&self, mask: u8) -> bool {
            self.hookshot_interlock() & mask != 0
        }

        pub(crate) fn dash_noise_requested(&self) -> bool {
            byte(self.ram, LINK_WANT_MAKE_NOISE_WHEN_DASHED) != 0
        }

        pub(crate) fn has_pull_action_state(&self) -> bool {
            byte(self.ram, LINK_PULL_ACTION_STATE) != 0
        }

        pub(crate) fn pull_action_state(&self) -> u8 {
            byte(self.ram, LINK_PULL_ACTION_STATE)
        }

        pub(crate) fn is_transforming(&self) -> bool {
            byte(self.ram, LINK_IS_TRANSFORMING) != 0
        }

        pub(crate) fn item_action_step_var(&self) -> u8 {
            byte(self.ram, LINK_ITEM_ACTION_STEP_SCRATCH)
        }

        pub(crate) fn throw_oam_state_index(&self) -> u8 {
            byte(self.ram, LINK_THROW_OAM_STATE_INDEX)
        }

        pub(crate) fn needs_pull_for_rupees_sprite(&self) -> bool {
            byte(self.ram, LINK_NEED_FOR_PULLFORRUPEES_SPRITE) != 0
        }

        pub(crate) fn is_near_moveable_statue(&self) -> bool {
            byte(self.ram, LINK_IS_NEAR_MOVEABLE_STATUE) != 0
        }

        pub(crate) fn is_prevented_from_moving(&self) -> bool {
            byte(self.ram, LINK_PREVENT_FROM_MOVING) != 0
        }

        pub(crate) fn button_b_frames(&self) -> u8 {
            byte(self.ram, BUTTON_B_FRAMES)
        }

        pub(crate) fn button_b_frames_word(&self) -> u16 {
            word(self.ram, BUTTON_B_FRAMES)
        }

        pub(crate) fn button_mask_b_y(&self) -> u8 {
            byte(self.ram, BUTTON_MASK_B_Y)
        }

        pub(crate) fn y_button_action_flags(&self) -> u8 {
            byte(self.ram, Y_BUTTON_ACTION_FLAGS)
        }

        pub(crate) fn y_button_action_step(&self) -> u8 {
            byte(self.ram, Y_BUTTON_ACTION_STEP)
        }

        pub(crate) fn y_button_action_timer(&self) -> u8 {
            byte(self.ram, Y_BUTTON_ACTION_TIMER)
        }

        pub(crate) fn filtered_joypad_h(&self) -> u8 {
            byte(self.ram, FILTERED_JOYPAD_H)
        }

        pub(crate) fn filtered_joypad_l(&self) -> u8 {
            byte(self.ram, FILTERED_JOYPAD_L)
        }

        pub(crate) fn joypad1h_last(&self) -> u8 {
            byte(self.ram, JOYPAD1H_LAST)
        }

        pub(crate) fn joypad1l_last(&self) -> u8 {
            byte(self.ram, JOYPAD1L_LAST)
        }

        pub(crate) fn joypad1h_last2(&self) -> u8 {
            byte(self.ram, JOYPAD1H_LAST2)
        }

        pub(crate) fn joypad1l_last2(&self) -> u8 {
            byte(self.ram, JOYPAD1L_LAST2)
        }

        pub(crate) fn button_b_frames_index(&self) -> usize {
            usize::from(self.button_b_frames())
        }

        pub(crate) fn opening_pose(&self) -> u8 {
            byte(self.ram, LINK_POSE_DURING_OPENING)
        }

        pub(crate) fn defense_flags(&self) -> u8 {
            byte(self.ram, PLAYER_DEFENSE_FLAGS)
        }

        pub(crate) fn on_somaria_platform(&self) -> u8 {
            byte(self.ram, PLAYER_ON_SOMARIA_PLATFORM)
        }

        pub(crate) fn has_somaria_platform_state(&self) -> bool {
            self.on_somaria_platform() != 0
        }

        pub(crate) fn near_pit_state(&self) -> u8 {
            byte(self.ram, PLAYER_NEAR_PIT_STATE)
        }

        pub(crate) fn is_near_pit(&self) -> bool {
            self.near_pit_state() != 0
        }

        pub(crate) fn near_pit_state_is(&self, value: u8) -> bool {
            self.near_pit_state() == value
        }

        pub(crate) fn near_pit_state_at_least(&self, value: u8) -> bool {
            self.near_pit_state() >= value
        }

        pub(crate) fn pit_data_index(&self) -> u8 {
            byte(self.ram, PLAYER_PIT_DATA_INDEX)
        }

        pub(crate) fn conveyor_belt_state(&self) -> u8 {
            byte(self.ram, LINK_ON_CONVEYOR_BELT)
        }

        pub(crate) fn tile_below(&self) -> u8 {
            byte(self.ram, LINK_TILE_BELOW)
        }

        pub(crate) fn is_on_lower_level(&self) -> bool {
            byte(self.ram, LINK_IS_ON_LOWER_LEVEL) != 0
        }

        pub(crate) fn lower_level_tilemap_offset(&self) -> u16 {
            if self.is_on_lower_level() {
                0x1000
            } else {
                0
            }
        }

        pub(crate) fn has_lower_level_state_or_mirror(&self) -> bool {
            byte(self.ram, LINK_IS_ON_LOWER_LEVEL) | byte(self.ram, LINK_IS_ON_LOWER_LEVEL_MIRROR)
                != 0
        }

        pub(crate) fn lower_level_state(&self) -> u8 {
            byte(self.ram, LINK_IS_ON_LOWER_LEVEL)
        }

        pub(crate) fn lower_level_mirror_state(&self) -> u8 {
            byte(self.ram, LINK_IS_ON_LOWER_LEVEL_MIRROR)
        }

        pub(crate) fn cached_lower_level_state(&self) -> u8 {
            byte(self.ram, LINK_IS_ON_LOWER_LEVEL_CACHED)
        }

        pub(crate) fn cached_lower_level_mirror_state(&self) -> u8 {
            byte(self.ram, LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED)
        }

        pub(crate) fn water_ripple_or_grass_state(&self) -> u8 {
            byte(self.ram, DRAW_WATER_RIPPLES_OR_GRASS)
        }

        pub(crate) fn animation_step(&self) -> u8 {
            byte(self.ram, LINK_ANIMATION_STEPS)
        }

        pub(crate) fn animation_step_index(&self) -> usize {
            usize::from(self.animation_step())
        }

        pub(crate) fn has_flippers(&self) -> bool {
            byte(self.ram, LINK_ITEM_FLIPPERS) != 0
        }

        pub(crate) fn flippers(&self) -> u8 {
            byte(self.ram, LINK_ITEM_FLIPPERS)
        }

        pub(crate) fn moon_pearl(&self) -> u8 {
            byte(self.ram, LINK_ITEM_MOON_PEARL)
        }

        pub(crate) fn has_moon_pearl(&self) -> bool {
            byte(self.ram, LINK_ITEM_MOON_PEARL) != 0
        }

        pub(crate) fn is_bunny(&self) -> bool {
            byte(self.ram, LINK_IS_BUNNY) != 0
        }

        pub(crate) fn is_bunny_mirror(&self) -> bool {
            byte(self.ram, LINK_IS_BUNNY_MIRROR) != 0
        }

        pub(crate) fn is_darkworld_save(&self) -> bool {
            byte(self.ram, SAVEGAME_IS_DARKWORLD) != 0
        }

        pub(crate) fn current_health(&self) -> u8 {
            byte(self.ram, LINK_CURRENT_HEALTH)
        }

        pub(crate) fn magic_power(&self) -> u8 {
            byte(self.ram, LINK_MAGIC_POWER)
        }

        pub(crate) fn magic_consumption_level(&self) -> u8 {
            byte(self.ram, LINK_MAGIC_CONSUMPTION)
        }

        pub(crate) fn item_in_hand(&self) -> u8 {
            byte(self.ram, LINK_ITEM_IN_HAND)
        }

        pub(crate) fn receive_item_index(&self) -> u8 {
            byte(self.ram, LINK_RECEIVE_ITEM_INDEX)
        }

        pub(crate) fn item_hold_pose(&self) -> u8 {
            byte(self.ram, LINK_POSE_FOR_ITEM)
        }

        pub(crate) fn swim_fast_state(&self) -> u8 {
            byte(self.ram, LINK_MAYBE_SWIM_FASTER)
        }

        pub(crate) fn faint_animation_active(&self) -> u8 {
            byte(self.ram, LINK_FAINT_ANIMATION_ACTIVE)
        }

        pub(crate) fn force_hold_sword_up_state(&self) -> u8 {
            byte(self.ram, LINK_FORCE_HOLD_SWORD_UP)
        }

        pub(crate) fn link_dma_staging_index(&self) -> u8 {
            byte(self.ram, LINK_DMA_STAGING_INDEX)
        }

        pub(crate) fn link_dma_graphics_index_word(&self) -> u16 {
            read_le_u16(self.ram, LINK_DMA_GRAPHICS_INDEX)
        }

        pub(crate) fn link_dma_left_sprite_bank_word(&self) -> u16 {
            read_le_u16(self.ram, LINK_DMA_LEFT_SPRITE_BANK_INDEX)
        }

        pub(crate) fn link_dma_right_sprite_bank_word(&self) -> u16 {
            read_le_u16(self.ram, LINK_DMA_RIGHT_SPRITE_BANK_INDEX)
        }

        pub(crate) fn link_dma_source_offset(&self) -> u16 {
            read_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET)
        }

        pub(crate) fn link_dma_tile_offset(&self) -> u16 {
            read_le_u16(self.ram, LINK_DMA_TILE_OFFSET)
        }

        pub(crate) fn sword_dma_graphics_index(&self) -> u8 {
            byte(self.ram, LINK_DMA_SWORD_GRAPHICS_INDEX)
        }

        pub(crate) fn shield_dma_graphics_index(&self) -> u8 {
            byte(self.ram, LINK_DMA_SHIELD_GRAPHICS_INDEX)
        }

        pub(crate) fn link_dma_staging_group(&self) -> u8 {
            self.link_dma_staging_index() >> 3
        }

        pub(crate) fn has_item_in_hand(&self) -> bool {
            byte(self.ram, LINK_ITEM_IN_HAND) != 0
        }

        pub(crate) fn has_item_or_position_mode(&self) -> bool {
            byte(self.ram, LINK_ITEM_IN_HAND) | byte(self.ram, LINK_POSITION_MODE) != 0
        }

        pub(crate) fn has_position_mode(&self) -> bool {
            byte(self.ram, LINK_POSITION_MODE) != 0
        }

        pub(crate) fn position_mode(&self) -> u8 {
            byte(self.ram, LINK_POSITION_MODE)
        }

        pub(crate) fn position_mode_has(&self, mask: u8) -> bool {
            byte(self.ram, LINK_POSITION_MODE) & mask != 0
        }

        pub(crate) fn item_in_hand_has(&self, mask: u8) -> bool {
            byte(self.ram, LINK_ITEM_IN_HAND) & mask != 0
        }

        pub(crate) fn state_bits(&self) -> u8 {
            byte(self.ram, LINK_STATE_BITS)
        }

        pub(crate) fn state_bits_has(&self, mask: u8) -> bool {
            byte(self.ram, LINK_STATE_BITS) & mask != 0
        }

        pub(crate) fn has_action_state(&self) -> bool {
            byte(self.ram, LINK_STATE_BITS) != 0
        }

        pub(crate) fn has_non_lift_action_state(&self) -> bool {
            byte(self.ram, LINK_STATE_BITS) & 0x7f != 0
        }

        pub(crate) fn is_lift_throw_primed(&self) -> bool {
            byte(self.ram, LINK_PICKING_THROW_STATE) & 1 != 0
        }

        pub(crate) fn picking_throw_state(&self) -> u8 {
            byte(self.ram, LINK_PICKING_THROW_STATE)
        }

        pub(crate) fn picking_throw_state_has(&self, mask: u8) -> bool {
            byte(self.ram, LINK_PICKING_THROW_STATE) & mask != 0
        }

        pub(crate) fn has_picking_throw_state(&self) -> bool {
            byte(self.ram, LINK_PICKING_THROW_STATE) != 0
        }

        pub(crate) fn is_lifting_or_carrying(&self) -> bool {
            byte(self.ram, LINK_STATE_BITS) & 0x80 != 0
        }

        pub(crate) fn is_ready_to_start_ground_movement(&self) -> bool {
            (byte(self.ram, LINK_GRABBING_WALL) & !2) == 0
                && !self.has_non_lift_action_state()
                && (!self.is_lifting_or_carrying()
                    || byte(self.ram, LINK_PICKING_THROW_STATE) & 1 == 0)
                && !self.has_item_or_position_mode()
        }

        pub(crate) fn has_grabbing_wall_state(&self) -> bool {
            byte(self.ram, LINK_GRABBING_WALL) != 0
        }

        pub(crate) fn grabbing_wall(&self) -> u8 {
            byte(self.ram, LINK_GRABBING_WALL)
        }

        pub(crate) fn grabbing_wall_has(&self, mask: u8) -> bool {
            byte(self.ram, LINK_GRABBING_WALL) & mask != 0
        }

        pub(crate) fn current_item_y(&self) -> u8 {
            byte(self.ram, LINK_CURRENT_ITEM_Y)
        }

        pub(crate) fn selected_rod(&self) -> u8 {
            byte(self.ram, EQ_SELECTED_ROD)
        }

        pub(crate) fn swim_stroke_anim_step(&self) -> u8 {
            byte(self.ram, SWIM_STROKE_ANIM_STEP)
        }

        pub(crate) fn state_for_spin_attack(&self) -> u8 {
            byte(self.ram, STATE_FOR_SPIN_ATTACK)
        }

        pub(crate) fn bit9_of_xcoord(&self) -> u8 {
            byte(self.ram, BIT9_OF_XCOORD)
        }

        pub(crate) fn primary_water_grass_timer(&self) -> u8 {
            byte(self.ram, PRIMARY_WATER_GRASS_TIMER)
        }

        pub(crate) fn secondary_water_grass_timer(&self) -> u8 {
            byte(self.ram, SECONDARY_WATER_GRASS_TIMER)
        }

        pub(crate) fn item_debug_value_1(&self) -> u8 {
            byte(self.ram, LINK_DEBUG_VALUE_1)
        }

        pub(crate) fn current_item_active(&self) -> u8 {
            byte(self.ram, LINK_CURRENT_ITEM_ACTIVE)
        }

        pub(crate) fn equipped_item(&self) -> u8 {
            byte(self.ram, LINK_EQUIPPED_ITEM)
        }

        pub(crate) fn force_move_any_direction_lo(&self) -> u16 {
            word(self.ram, FORCE_MOVE_ANY_DIRECTION) & 0x00ff
        }

        pub(crate) fn cheat_walk_through_walls(&self) -> u8 {
            byte(self.ram, CHEAT_WALK_THROUGH_WALLS)
        }

        pub(crate) fn drag_player_x(&self) -> u16 {
            word(self.ram, DRAG_PLAYER_X)
        }

        pub(crate) fn drag_player_y(&self) -> u16 {
            word(self.ram, DRAG_PLAYER_Y)
        }

        pub(crate) fn pushed_block_mode(&self) -> u8 {
            byte(self.ram, PUSHED_BLOCK_MODE)
        }

        pub(crate) fn dma_head_pointer(&self) -> u8 {
            byte(self.ram, DMA_HEAD_POINTER)
        }

        pub(crate) fn dma_body_pointer(&self) -> u8 {
            byte(self.ram, DMA_BODY_POINTER)
        }
    }

    pub(crate) struct PlayerStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PlayerStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_X_COORD, value);
        }

        pub(crate) fn set_x_low(&mut self, value: u8) {
            self.ram[LINK_X_COORD] = value;
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_Y_COORD, value);
        }

        pub(crate) fn set_y_low(&mut self, value: u8) {
            self.ram[LINK_Y_COORD] = value;
        }

        pub(crate) fn set_oam_x_offset(&mut self, value: u8) {
            self.ram[PLAYER_OAM_X_OFFSET] = value;
        }

        pub(crate) fn set_oam_y_offset(&mut self, value: u8) {
            self.ram[PLAYER_OAM_Y_OFFSET] = value;
        }

        pub(crate) fn set_oam_offsets(&mut self, x: u8, y: u8) {
            self.set_oam_x_offset(x);
            self.set_oam_y_offset(y);
        }

        pub(crate) fn disable_oam_offsets(&mut self) {
            self.set_oam_offsets(0x80, 0x80);
        }

        pub(crate) fn set_z(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_Z_COORD, value);
        }

        pub(crate) fn set_z_low(&mut self, value: u8) {
            self.ram[LINK_Z_COORD] = value;
        }

        pub(crate) fn restore_z_low_from_mirror(&mut self) {
            self.ram[LINK_Z_COORD] = self.ram[LINK_Z_COORD_MIRROR];
        }

        pub(crate) fn restore_z_from_mirror(&mut self) {
            copy_word(self.ram, LINK_Z_COORD, LINK_Z_COORD_MIRROR);
        }

        pub(crate) fn cache_z_low_to_mirror(&mut self) {
            self.ram[LINK_Z_COORD_MIRROR] = self.ram[LINK_Z_COORD];
        }

        pub(crate) fn cache_z_to_mirror(&mut self) {
            copy_word(self.ram, LINK_Z_COORD_MIRROR, LINK_Z_COORD);
        }

        pub(crate) fn set_z_mirror(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_Z_COORD_MIRROR, value);
        }

        pub(crate) fn clear_z_mirror_low(&mut self) {
            self.ram[LINK_Z_COORD_MIRROR] = 0;
        }

        pub(crate) fn clear_z_mirror_word_low(&mut self) {
            let value = word(self.ram, LINK_Z_COORD_MIRROR) & !0x00ff;
            write_le_u16(self.ram, LINK_Z_COORD_MIRROR, value);
        }

        pub(crate) fn set_z_and_mirror(&mut self, value: u16) {
            self.set_z(value);
            self.set_z_mirror(value);
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            self.set_x(x);
            self.set_y(y);
        }

        pub(crate) fn clear_z_high(&mut self) {
            self.ram[LINK_Z_COORD + 1] = 0;
        }

        pub(crate) fn restore_position_from_cached(&mut self) {
            copy_word(self.ram, LINK_X_COORD, LINK_X_COORD_CACHED);
            copy_word(self.ram, LINK_Y_COORD, LINK_Y_COORD_CACHED);
        }

        pub(crate) fn cache_current_position(&mut self) {
            copy_word(self.ram, LINK_Y_COORD_CACHED, LINK_Y_COORD);
            copy_word(self.ram, LINK_X_COORD_CACHED, LINK_X_COORD);
        }

        pub(crate) fn cache_copied_position_from_current(&mut self) {
            copy_word(self.ram, LINK_Y_COORD_COPY, LINK_Y_COORD);
            copy_word(self.ram, LINK_X_COORD_COPY, LINK_X_COORD);
        }

        pub(crate) fn cache_current_quadrants(&mut self) {
            self.ram[LINK_QUADRANT_X_CACHED] = self.ram[LINK_QUADRANT_X];
            self.ram[LINK_QUADRANT_Y_CACHED] = self.ram[LINK_QUADRANT_Y];
        }

        pub(crate) fn restore_quadrants_from_cached(&mut self) {
            self.ram[LINK_QUADRANT_X] = self.ram[LINK_QUADRANT_X_CACHED];
            self.ram[LINK_QUADRANT_Y] = self.ram[LINK_QUADRANT_Y_CACHED];
        }

        pub(crate) fn restore_y_from_previous_position(&mut self) {
            copy_word(self.ram, LINK_Y_COORD, LINK_Y_COORD_PREV);
        }

        pub(crate) fn restore_position_from_previous(&mut self) {
            copy_word(self.ram, LINK_X_COORD, LINK_X_COORD_PREV);
            copy_word(self.ram, LINK_Y_COORD, LINK_Y_COORD_PREV);
        }

        pub(crate) fn cache_safe_return_high_from_current(&mut self) {
            self.ram[LINK_X_COORD_SAFE_RETURN_HI] = self.ram[LINK_X_COORD + 1];
            self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = self.ram[LINK_Y_COORD + 1];
        }

        pub(crate) fn cache_previous_position_from_current(&mut self) {
            copy_word(self.ram, LINK_Y_COORD_PREV, LINK_Y_COORD);
            copy_word(self.ram, LINK_X_COORD_PREV, LINK_X_COORD);
        }

        pub(crate) fn cache_previous_position_from_current_xy_order(&mut self) {
            copy_word(self.ram, LINK_X_COORD_PREV, LINK_X_COORD);
            copy_word(self.ram, LINK_Y_COORD_PREV, LINK_Y_COORD);
        }

        pub(crate) fn set_previous_position(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, LINK_X_COORD_PREV, x);
            write_le_u16(self.ram, LINK_Y_COORD_PREV, y);
        }

        pub(crate) fn move_x_by_velocity(&mut self, velocity: u8) -> u16 {
            move_link_axis_by_velocity(self.ram, LINK_X_SUBPIXEL, LINK_X_COORD, velocity)
        }

        pub(crate) fn move_y_by_velocity(&mut self, velocity: u8) -> u16 {
            move_link_axis_by_velocity(self.ram, LINK_Y_SUBPIXEL, LINK_Y_COORD, velocity)
        }

        pub(crate) fn move_z_by_velocity(&mut self, velocity: u8) -> u16 {
            move_link_axis_by_velocity(self.ram, LINK_Z_SUBPIXEL, LINK_Z_COORD, velocity)
        }

        pub(crate) fn move_x_by_subpixel_delta(&mut self, delta: u16) -> u16 {
            move_link_axis_by_subpixel_delta(self.ram, LINK_X_SUBPIXEL, LINK_X_COORD, delta)
        }

        pub(crate) fn move_y_by_subpixel_delta(&mut self, delta: u16) -> u16 {
            move_link_axis_by_subpixel_delta(self.ram, LINK_Y_SUBPIXEL, LINK_Y_COORD, delta)
        }

        pub(crate) fn store_overworld_exit_position_from_current(&mut self) {
            copy_word(self.ram, LINK_Y_COORD_EXIT_OVERWORLD, LINK_Y_COORD);
            copy_word(self.ram, LINK_X_COORD_EXIT_OVERWORLD, LINK_X_COORD);
        }

        pub(crate) fn store_overworld_exit_y_from_current(&mut self) {
            copy_word(self.ram, LINK_Y_COORD_EXIT_OVERWORLD, LINK_Y_COORD);
        }

        pub(crate) fn restore_y_from_overworld_exit(&mut self) {
            copy_word(self.ram, LINK_Y_COORD, LINK_Y_COORD_EXIT_OVERWORLD);
        }

        pub(crate) fn restore_position_from_overworld_exit(&mut self) {
            copy_word(self.ram, LINK_X_COORD, LINK_X_COORD_EXIT_OVERWORLD);
            copy_word(self.ram, LINK_Y_COORD, LINK_Y_COORD_EXIT_OVERWORLD);
        }

        pub(crate) fn restore_lower_level_state_from_cached(&mut self) {
            self.ram[LINK_IS_ON_LOWER_LEVEL] = self.ram[LINK_IS_ON_LOWER_LEVEL_CACHED];
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] =
                self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED];
        }

        pub(crate) fn restore_facing_from_cached(&mut self) {
            self.ram[LINK_FACING] = self.ram[LINK_FACING_CACHED];
        }

        pub(crate) fn store_safe_return_position(&mut self, x: u16, y: u16) {
            self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
            self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
            self.ram[LINK_X_COORD_SAFE_RETURN_LO] = x as u8;
            self.ram[LINK_X_COORD_SAFE_RETURN_HI] = (x >> 8) as u8;
        }

        pub(crate) fn restore_position_from_safe_return(&mut self) {
            self.ram[LINK_Y_COORD] = self.ram[LINK_Y_COORD_SAFE_RETURN_LO];
            self.ram[LINK_Y_COORD + 1] = self.ram[LINK_Y_COORD_SAFE_RETURN_HI];
            self.ram[LINK_X_COORD] = self.ram[LINK_X_COORD_SAFE_RETURN_LO];
            self.ram[LINK_X_COORD + 1] = self.ram[LINK_X_COORD_SAFE_RETURN_HI];
        }

        pub(crate) fn store_safe_return_low_from_current(&mut self) {
            self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = self.ram[LINK_Y_COORD];
            self.ram[LINK_X_COORD_SAFE_RETURN_LO] = self.ram[LINK_X_COORD];
        }

        pub(crate) fn store_safe_return_y(&mut self, y: u16) {
            self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
            self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
        }

        pub(crate) fn set_hop_origin_coord(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_Y_COORD_ORIGINAL, value);
        }

        pub(crate) fn set_hop_origin_delta_from_y(&mut self, y: u16) -> u16 {
            let diff = word(self.ram, LINK_Y_COORD_ORIGINAL).wrapping_sub(y);
            write_le_u16(self.ram, LINK_Y_COORD_ORIGINAL, diff);
            diff
        }

        pub(crate) fn restore_y_from_hop_origin(&mut self) {
            copy_word(self.ram, LINK_Y_COORD, LINK_Y_COORD_ORIGINAL);
        }

        pub(crate) fn clear_temp_bunny_timer(&mut self) {
            write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, 0);
        }

        pub(crate) fn set_temp_bunny_timer(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, value);
        }

        pub(crate) fn decrement_temp_bunny_timer(&mut self) -> u16 {
            let timer = word(self.ram, LINK_TIMER_TEMPBUNNY).wrapping_sub(1);
            write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, timer);
            timer
        }

        pub(crate) fn set_safe_return_y_low(&mut self, value: u8) {
            self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = value;
        }

        pub(crate) fn set_movement_velocity_from_position_delta(
            &mut self,
            x: u16,
            y: u16,
            old_x: u16,
            old_y: u16,
        ) {
            self.ram[LINK_Y_VELOCITY] = y.wrapping_sub(old_y) as u8;
            self.ram[LINK_X_VELOCITY] = x.wrapping_sub(old_x) as u8;
        }

        pub(crate) fn set_movement_velocity_from_delta(&mut self, x_delta: u16, y_delta: u16) {
            self.ram[LINK_Y_VELOCITY] = y_delta as u8;
            self.ram[LINK_X_VELOCITY] = x_delta as u8;
        }

        pub(crate) fn subtract_axis_velocity_delta(&mut self, horizontal: bool, delta: u8) {
            if horizontal {
                self.ram[LINK_X_VELOCITY] = self.ram[LINK_X_VELOCITY].wrapping_sub(delta);
            } else {
                self.ram[LINK_Y_VELOCITY] = self.ram[LINK_Y_VELOCITY].wrapping_sub(delta);
            }
        }

        pub(crate) fn add_movement_velocity_delta(&mut self, x_delta: u16, y_delta: u16) {
            self.ram[LINK_X_VELOCITY] = self.ram[LINK_X_VELOCITY].wrapping_add(x_delta as u8);
            self.ram[LINK_Y_VELOCITY] = self.ram[LINK_Y_VELOCITY].wrapping_add(y_delta as u8);
        }

        pub(crate) fn add_y_velocity_delta(&mut self, y_delta: u8) {
            self.ram[LINK_Y_VELOCITY] = self.ram[LINK_Y_VELOCITY].wrapping_add(y_delta);
        }

        pub(crate) fn set_y_velocity_from_safe_return_delta_unless_ledge_hopping(&mut self) {
            if self.ram[LINK_HANDLER_STATE] != 11 {
                self.ram[LINK_Y_VELOCITY] =
                    self.ram[LINK_Y_COORD].wrapping_sub(self.ram[LINK_Y_COORD_SAFE_RETURN_LO]);
            }
        }

        pub(crate) fn set_x_velocity_from_safe_return_delta(&mut self) {
            self.ram[LINK_X_VELOCITY] =
                self.ram[LINK_X_COORD].wrapping_sub(self.ram[LINK_X_COORD_SAFE_RETURN_LO]);
        }

        pub(crate) fn update_vertical_direction_from_movement_velocity(&mut self) {
            if self.ram[LINK_Y_VELOCITY] != 0 {
                self.ram[LINK_DIRECTION] = (self.ram[LINK_DIRECTION] & 3)
                    | if (self.ram[LINK_Y_VELOCITY] as i8).is_negative() {
                        8
                    } else {
                        4
                    };
            }
        }

        pub(crate) fn update_horizontal_direction_from_movement_velocity(&mut self) {
            if self.ram[LINK_X_VELOCITY] != 0 {
                self.ram[LINK_DIRECTION] = (self.ram[LINK_DIRECTION] & 0x0c)
                    | if (self.ram[LINK_X_VELOCITY] as i8).is_negative() {
                        2
                    } else {
                        1
                    };
            }
        }

        pub(crate) fn refresh_direction_from_safe_return_delta(&mut self) {
            self.set_y_velocity_from_safe_return_delta_unless_ledge_hopping();
            self.update_vertical_direction_from_movement_velocity();
            self.set_x_velocity_from_safe_return_delta();
            self.update_horizontal_direction_from_movement_velocity();
        }

        pub(crate) fn set_x_velocity(&mut self, value: u8) {
            self.ram[LINK_X_VELOCITY] = value;
        }

        pub(crate) fn set_y_velocity(&mut self, value: u8) {
            self.ram[LINK_Y_VELOCITY] = value;
        }

        pub(crate) fn clear_movement_velocity_and_direction(&mut self) {
            self.ram[LINK_X_VELOCITY] = 0;
            self.ram[LINK_Y_VELOCITY] = 0;
            self.ram[LINK_DIRECTION] = 0;
        }

        pub(crate) fn clear_movement_velocity(&mut self) {
            self.ram[LINK_X_VELOCITY] = 0;
            self.ram[LINK_Y_VELOCITY] = 0;
        }

        pub(crate) fn clear_movement_subpixels(&mut self) {
            self.ram[LINK_X_SUBPIXEL] = 0;
            self.ram[LINK_Y_SUBPIXEL] = 0;
        }

        pub(crate) fn clear_link_state_block_for_ending(&mut self) {
            self.ram[LINK_Y_COORD..LINK_Y_COORD + 0x70].fill(0);
        }

        pub(crate) fn clear_page_movement_deltas(&mut self) {
            self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = 0;
            self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = 0;
        }

        pub(crate) fn set_page_movement_deltas(&mut self, y_delta: u8, x_delta: u8) {
            self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = y_delta;
            self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = x_delta;
        }

        pub(crate) fn set_y_page_movement_delta_from_high_position(&mut self, high: u8) {
            self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] =
                high.wrapping_sub(self.ram[LINK_Y_COORD_SAFE_RETURN_HI]);
        }

        pub(crate) fn set_x_page_movement_delta_from_high_position(&mut self, high: u8) {
            self.ram[LINK_X_PAGE_MOVEMENT_DELTA] =
                high.wrapping_sub(self.ram[LINK_X_COORD_SAFE_RETURN_HI]);
        }

        pub(crate) fn clear_actual_velocity_and_page_movement_deltas(&mut self) {
            self.clear_actual_velocity_xy();
            self.clear_page_movement_deltas();
        }

        pub(crate) fn set_moving_against_diag_tile(&mut self, value: u8) {
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = value;
        }

        pub(crate) fn add_moving_against_diag_tile_flags(&mut self, value: u8) {
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] |= value;
        }

        pub(crate) fn clear_moving_against_diag_tile(&mut self) {
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        }

        pub(crate) fn reset_direction_limits(&mut self) {
            self.ram[LINK_DIRECTION_MASK_A] = 0x0f;
            self.ram[LINK_DIRECTION_MASK_B] = 0x0f;
            self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;
        }

        pub(crate) fn reset_direction_masks(&mut self) {
            self.ram[LINK_DIRECTION_MASK_A] = 0x0f;
            self.ram[LINK_DIRECTION_MASK_B] = 0x0f;
        }

        pub(crate) fn set_quadrants_from_packed_nibbles(&mut self, value: u8) {
            self.ram[LINK_QUADRANT_X] = value >> 4;
            self.ram[LINK_QUADRANT_Y] = value & 0x0f;
        }

        pub(crate) fn set_quadrants(&mut self, x: u8, y: u8) {
            self.ram[LINK_QUADRANT_X] = x;
            self.ram[LINK_QUADRANT_Y] = y;
        }

        pub(crate) fn toggle_quadrant_x(&mut self) -> u8 {
            self.ram[LINK_QUADRANT_X] ^= 1;
            self.ram[LINK_QUADRANT_X]
        }

        pub(crate) fn toggle_quadrant_y(&mut self) -> u8 {
            self.ram[LINK_QUADRANT_Y] ^= 2;
            self.ram[LINK_QUADRANT_Y]
        }

        pub(crate) fn increment_orthogonal_direction_count(&mut self) {
            self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] =
                self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS].wrapping_add(1);
        }

        pub(crate) fn clear_orthogonal_direction_count(&mut self) {
            self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;
        }

        pub(crate) fn set_last_direction_moved_towards(&mut self, value: u8) {
            self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = value;
        }

        pub(crate) fn set_last_direction_from_current_direction(&mut self) {
            self.ram[LINK_LAST_DIRECTION] = self.ram[LINK_DIRECTION];
        }

        pub(crate) fn set_last_direction(&mut self, value: u8) {
            self.ram[LINK_LAST_DIRECTION] = value;
        }

        pub(crate) fn mask_last_direction(&mut self, mask: u8) {
            self.ram[LINK_LAST_DIRECTION] &= mask;
        }

        pub(crate) fn set_last_direction_from_swim_flags(&mut self) {
            self.ram[LINK_LAST_DIRECTION] = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
        }

        pub(crate) fn set_swim_flags_from_last_direction(&mut self) {
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_LAST_DIRECTION];
        }

        pub(crate) fn set_direction(&mut self, value: u8) {
            self.ram[LINK_DIRECTION] = value;
        }

        pub(crate) fn set_direction_and_last_direction(&mut self, value: u8) {
            self.ram[LINK_DIRECTION] = value;
            self.ram[LINK_LAST_DIRECTION] = value;
        }

        pub(crate) fn set_direction_and_swim_flags(&mut self, value: u8) {
            self.ram[LINK_DIRECTION] = value;
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = value;
        }

        pub(crate) fn mask_direction(&mut self, mask: u8) {
            self.ram[LINK_DIRECTION] &= mask;
        }

        pub(crate) fn clear_cardinal_direction(&mut self) {
            self.ram[LINK_DIRECTION] &= !0x0f;
        }

        pub(crate) fn add_direction_flags(&mut self, flags: u8) {
            self.ram[LINK_DIRECTION] |= flags;
        }

        pub(crate) fn clear_direction_flags(&mut self, flags: u8) {
            self.ram[LINK_DIRECTION] &= !flags;
        }

        pub(crate) fn set_direction_lock(&mut self, value: u8) {
            self.ram[LINK_CANT_CHANGE_DIRECTION] = value;
        }

        pub(crate) fn clear_direction_lock(&mut self) {
            self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        }

        pub(crate) fn set_direction_lock_bits(&mut self, mask: u8) {
            self.ram[LINK_CANT_CHANGE_DIRECTION] |= mask;
        }

        pub(crate) fn clear_direction_lock_bits(&mut self, mask: u8) {
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !mask;
        }

        pub(crate) fn set_direction_mask_a(&mut self, value: u8) {
            self.ram[LINK_DIRECTION_MASK_A] = value;
        }

        pub(crate) fn set_direction_mask_b(&mut self, value: u8) {
            self.ram[LINK_DIRECTION_MASK_B] = value;
        }

        pub(crate) fn apply_direction_masks(&mut self) {
            self.ram[LINK_DIRECTION] &=
                self.ram[LINK_DIRECTION_MASK_A] & self.ram[LINK_DIRECTION_MASK_B];
        }

        pub(crate) fn force_direction_from_diag_tile_if_needed(&mut self) {
            if self.ram[LINK_DIRECTION] & 0x0f != 0
                && self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0f != 0
            {
                self.ram[LINK_DIRECTION] = self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0f;
            }
        }

        pub(crate) fn resolve_orthogonal_direction_count_from_facing(&mut self) {
            self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] =
                if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 2 {
                    if self.ram[LINK_FACING] & 4 != 0 {
                        2
                    } else {
                        1
                    }
                } else {
                    0
                };
        }

        pub(crate) fn mark_moving_floor_direction(&mut self, floor_y: u16, floor_x: u16) {
            if floor_y != 0 {
                self.ram[LINK_DIRECTION] |= if (floor_y as i16).is_negative() { 8 } else { 4 };
            }
            if floor_x != 0 {
                self.ram[LINK_DIRECTION] |= if (floor_x as i16).is_negative() { 2 } else { 1 };
            }
        }

        pub(crate) fn cache_moving_floor_position(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, RELATED_TO_MOVING_FLOOR_Y, y);
            write_le_u16(self.ram, RELATED_TO_MOVING_FLOOR_X, x);
        }

        pub(crate) fn mark_lower_level(&mut self) {
            self.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
        }

        pub(crate) fn mark_lower_level_mirror(&mut self) {
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = 1;
        }

        pub(crate) fn set_lower_level_state(&mut self, value: u8) {
            self.ram[LINK_IS_ON_LOWER_LEVEL] = value;
        }

        pub(crate) fn set_lower_level_mirror_state(&mut self, value: u8) {
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = value;
        }

        pub(crate) fn set_lower_level_states(&mut self, state: u8, mirror: u8) {
            self.ram[LINK_IS_ON_LOWER_LEVEL] = state;
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = mirror;
        }

        pub(crate) fn clear_lower_level(&mut self) {
            self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
        }

        pub(crate) fn clear_lower_level_states(&mut self) {
            self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = 0;
        }

        pub(crate) fn set_water_ripple_or_grass_state(&mut self, value: u8) {
            self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = value;
        }

        pub(crate) fn clear_water_ripple_or_grass_state(&mut self) {
            self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        }

        pub(crate) fn increment_water_ripple_or_grass_state(&mut self) -> u8 {
            self.ram[DRAW_WATER_RIPPLES_OR_GRASS] =
                self.ram[DRAW_WATER_RIPPLES_OR_GRASS].wrapping_add(1);
            self.ram[DRAW_WATER_RIPPLES_OR_GRASS]
        }

        pub(crate) fn toggle_lower_level_state(&mut self) {
            self.ram[LINK_IS_ON_LOWER_LEVEL] ^= 1;
        }

        pub(crate) fn toggle_lower_level_mirror_state(&mut self) {
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] ^= 1;
        }

        pub(crate) fn mirror_lower_level_state(&mut self) {
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        }

        pub(crate) fn set_actual_z_velocity(&mut self, value: u8) {
            self.ram[LINK_Z_VELOCITY] = value;
        }

        pub(crate) fn set_recoil_z_velocity_for_dungeon_reset(&mut self, value: u8) {
            self.ram[LINK_RECOIL_Z_VELOCITY_DUNGEON] = value;
        }

        pub(crate) fn set_recoil_z_velocity(&mut self, value: u8) {
            self.ram[LINK_RECOIL_Z_VELOCITY_DUNGEON] = value;
        }

        pub(crate) fn set_actual_x_velocity(&mut self, value: u8) {
            self.ram[LINK_ACTUAL_X_VELOCITY] = value;
        }

        pub(crate) fn set_actual_y_velocity(&mut self, value: u8) {
            self.ram[LINK_ACTUAL_Y_VELOCITY] = value;
        }

        pub(crate) fn clear_actual_x_velocity(&mut self) {
            self.ram[LINK_ACTUAL_X_VELOCITY] = 0;
        }

        pub(crate) fn clear_actual_y_velocity(&mut self) {
            self.ram[LINK_ACTUAL_Y_VELOCITY] = 0;
        }

        pub(crate) fn set_actual_velocity_xy(&mut self, x: u8, y: u8) {
            self.ram[LINK_ACTUAL_X_VELOCITY] = x;
            self.ram[LINK_ACTUAL_Y_VELOCITY] = y;
        }

        pub(crate) fn invert_actual_velocity_xy(&mut self) {
            self.ram[LINK_ACTUAL_X_VELOCITY] = (-(self.ram[LINK_ACTUAL_X_VELOCITY] as i8)) as u8;
            self.ram[LINK_ACTUAL_Y_VELOCITY] = (-(self.ram[LINK_ACTUAL_Y_VELOCITY] as i8)) as u8;
        }

        pub(crate) fn xor_actual_velocity_xy(&mut self, mask: u8) {
            self.ram[LINK_ACTUAL_X_VELOCITY] ^= mask;
            self.ram[LINK_ACTUAL_Y_VELOCITY] ^= mask;
        }

        pub(crate) fn derive_direction_from_actual_velocity(&mut self) {
            self.ram[LINK_DIRECTION] = 0;
            if self.ram[LINK_ACTUAL_Y_VELOCITY] != 0 {
                self.ram[LINK_DIRECTION] |=
                    if (self.ram[LINK_ACTUAL_Y_VELOCITY] as i8).is_negative() {
                        8
                    } else {
                        4
                    };
            }
            if self.ram[LINK_ACTUAL_X_VELOCITY] != 0 {
                self.ram[LINK_DIRECTION] |=
                    if (self.ram[LINK_ACTUAL_X_VELOCITY] as i8).is_negative() {
                        2
                    } else {
                        1
                    };
            }
        }

        pub(crate) fn set_actual_velocity_from_direction(&mut self, direction: u8, velocity: u8) {
            self.ram[LINK_ACTUAL_X_VELOCITY] = if direction & 0x03 != 0 {
                if direction & 0x02 != 0 {
                    0u8.wrapping_sub(velocity)
                } else {
                    velocity
                }
            } else {
                0
            };
            self.ram[LINK_ACTUAL_Y_VELOCITY] = if direction & 0x0c != 0 {
                if direction & 0x08 != 0 {
                    0u8.wrapping_sub(velocity)
                } else {
                    velocity
                }
            } else {
                0
            };
        }

        pub(crate) fn clear_actual_velocity_xy(&mut self) {
            self.set_actual_velocity_xy(0, 0);
        }

        pub(crate) fn set_actual_z_velocity_and_copy(&mut self, value: u8) {
            self.ram[LINK_Z_VELOCITY] = value;
            self.ram[LINK_Z_VELOCITY_COPY] = value;
        }

        pub(crate) fn set_actual_z_velocity_mirror_and_copy(&mut self, value: u8) {
            self.ram[LINK_Z_VELOCITY_MIRROR] = value;
            self.ram[LINK_Z_VELOCITY_COPY_MIRROR] = value;
        }

        pub(crate) fn restore_actual_z_velocity_from_mirror(&mut self) {
            self.ram[LINK_Z_VELOCITY] = self.ram[LINK_Z_VELOCITY_MIRROR];
            self.ram[LINK_Z_VELOCITY_COPY] = self.ram[LINK_Z_VELOCITY_COPY_MIRROR];
        }

        pub(crate) fn cache_actual_z_velocity_to_mirror(&mut self) {
            self.ram[LINK_Z_VELOCITY_MIRROR] = self.ram[LINK_Z_VELOCITY];
            self.ram[LINK_Z_VELOCITY_COPY_MIRROR] = self.ram[LINK_Z_VELOCITY_COPY];
        }

        pub(crate) fn prime_airborne_z_velocity(&mut self) {
            self.ram[LINK_Z_VELOCITY] = 0xff;
            write_le_u16(self.ram, LINK_Z_COORD, 0xffff);
            self.ram[LINK_Z_SUBPIXEL] = 0;
        }

        pub(crate) fn decrement_actual_z_velocity(&mut self, delta: u8) {
            self.ram[LINK_Z_VELOCITY] = self.ram[LINK_Z_VELOCITY].wrapping_sub(delta);
        }

        pub(crate) fn set_incapacitated_timer(&mut self, value: u8) {
            self.ram[LINK_INCAPACITATED_TIMER] = value;
        }

        pub(crate) fn decrement_incapacitated_timer(&mut self) -> u8 {
            self.ram[LINK_INCAPACITATED_TIMER] = self.ram[LINK_INCAPACITATED_TIMER].wrapping_sub(1);
            self.ram[LINK_INCAPACITATED_TIMER]
        }

        pub(crate) fn reset_elapsed_incapacitated_timer(&mut self) {
            if self.ram[LINK_INCAPACITATED_TIMER] == 0 {
                self.ram[LINK_INCAPACITATED_TIMER] = 1;
            }
        }

        pub(crate) fn set_recoil_timer(&mut self, value: u8) {
            self.ram[LINK_RECOIL_TIMER] = value;
        }

        pub(crate) fn increment_recoil_timer(&mut self) -> u8 {
            self.ram[LINK_RECOIL_TIMER] = self.ram[LINK_RECOIL_TIMER].wrapping_add(1);
            self.ram[LINK_RECOIL_TIMER]
        }

        pub(crate) fn clear_speed_modifier(&mut self) {
            self.ram[LINK_SPEED_MODIFIER] = 0;
        }

        pub(crate) fn set_speed_modifier(&mut self, value: u8) {
            self.ram[LINK_SPEED_MODIFIER] = value;
        }

        pub(crate) fn set_tile_below(&mut self, value: u8) {
            self.ram[LINK_TILE_BELOW] = value;
        }

        pub(crate) fn advance_frame_change_counter(&mut self, delay: u8) -> bool {
            self.ram[LINK_FRAME_CHANGE_COUNTER] =
                self.ram[LINK_FRAME_CHANGE_COUNTER].wrapping_add(1);
            if self.ram[LINK_FRAME_CHANGE_COUNTER] >= delay {
                self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
                true
            } else {
                false
            }
        }

        pub(crate) fn set_visibility_status(&mut self, value: u8) {
            self.ram[LINK_VISIBILITY_STATUS] = value;
        }

        pub(crate) fn set_sprite_damage_disable_timer(&mut self, value: u8) {
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = value;
        }

        pub(crate) fn clear_sprite_damage_disable_timer(&mut self) {
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        }

        pub(crate) fn set_somaria_platform_state(&mut self, value: u8) {
            self.ram[PLAYER_ON_SOMARIA_PLATFORM] = value;
        }

        pub(crate) fn clear_somaria_platform_state(&mut self) {
            self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
        }

        pub(crate) fn set_near_pit_state(&mut self, value: u8) {
            self.ram[PLAYER_NEAR_PIT_STATE] = value;
        }

        pub(crate) fn clear_near_pit_state(&mut self) {
            self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        }

        pub(crate) fn set_pit_data_index(&mut self, value: u8) {
            self.ram[PLAYER_PIT_DATA_INDEX] = value;
        }

        pub(crate) fn clear_pit_data_index(&mut self) {
            self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        }

        pub(crate) fn advance_pit_data_index(&mut self) -> u8 {
            self.ram[PLAYER_PIT_DATA_INDEX] = self.ram[PLAYER_PIT_DATA_INDEX].wrapping_add(1);
            self.ram[PLAYER_PIT_DATA_INDEX]
        }

        pub(crate) fn begin_pit_check(&mut self) {
            self.clear_pit_data_index();
            self.set_near_pit_state(1);
        }

        pub(crate) fn clear_pit_state(&mut self) {
            self.clear_pit_data_index();
            self.clear_near_pit_state();
        }

        pub(crate) fn set_hookshot_interlock(&mut self, value: u8) {
            self.ram[RELATED_TO_HOOKSHOT] = value;
        }

        pub(crate) fn clear_hookshot_interlock(&mut self) {
            self.ram[RELATED_TO_HOOKSHOT] = 0;
        }

        pub(crate) fn xor_hookshot_interlock(&mut self, mask: u8) {
            self.ram[RELATED_TO_HOOKSHOT] ^= mask;
        }

        pub(crate) fn increment_sprite_damage_disable_timer(&mut self) {
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] =
                self.ram[LINK_DISABLE_SPRITE_DAMAGE].wrapping_add(1);
        }

        pub(crate) fn clear_electrocute_on_touch(&mut self) {
            self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        }

        pub(crate) fn set_electrocute_on_touch(&mut self, value: u8) {
            self.ram[LINK_ELECTROCUTE_ON_TOUCH] = value;
        }

        pub(crate) fn clear_conveyor_belt_state(&mut self) {
            self.ram[LINK_ON_CONVEYOR_BELT] = 0;
        }

        pub(crate) fn clear_faint_animation_active(&mut self) {
            self.ram[LINK_FAINT_ANIMATION_ACTIVE] = 0;
        }

        pub(crate) fn clear_hookshot_grave_latch(&mut self) {
            self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 0;
        }

        pub(crate) fn set_hookshot_grave_latch(&mut self) {
            self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 1;
        }

        pub(crate) fn set_conveyor_belt_state(&mut self, value: u8) {
            self.ram[LINK_ON_CONVEYOR_BELT] = value;
        }

        pub(crate) fn set_deep_water_state(&mut self, value: u8) {
            self.ram[LINK_IS_IN_DEEP_WATER] = value;
        }

        pub(crate) fn enter_deep_water_state(&mut self) {
            self.ram[LINK_IS_IN_DEEP_WATER] = 1;
        }

        pub(crate) fn clear_deep_water_state(&mut self) {
            self.ram[LINK_IS_IN_DEEP_WATER] = 0;
        }

        pub(crate) fn clear_whirlpool_trigger(&mut self) {
            self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 0;
        }

        pub(crate) fn set_whirlpool_trigger(&mut self) {
            self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 1;
        }

        pub(crate) fn whirlpool_triggered(&self) -> bool {
            byte(self.ram, LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE) != 0
        }

        pub(crate) fn set_dash_noise_request(&mut self) {
            self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 1;
        }

        pub(crate) fn clear_dash_noise_request(&mut self) {
            self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 0;
        }

        pub(crate) fn decrement_incapacitated_camera_timer(&mut self) -> u8 {
            self.ram[LINK_INCAPACITATED_CAMERA_TIMER] =
                self.ram[LINK_INCAPACITATED_CAMERA_TIMER].wrapping_sub(1);
            self.ram[LINK_INCAPACITATED_CAMERA_TIMER]
        }

        pub(crate) fn reset_incapacitated_camera_timer_from_incapacitated(&mut self) {
            self.ram[LINK_INCAPACITATED_CAMERA_TIMER] = self.ram[LINK_INCAPACITATED_TIMER] >> 4;
        }

        pub(crate) fn tick_jump_ledge_timer_or_reset(&mut self) -> bool {
            self.ram[LINK_TIMER_JUMP_LEDGE] = self.ram[LINK_TIMER_JUMP_LEDGE].wrapping_sub(1);
            if (self.ram[LINK_TIMER_JUMP_LEDGE] as i8).is_negative() {
                self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
                true
            } else {
                false
            }
        }

        pub(crate) fn reset_jump_ledge_timer(&mut self) {
            self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
        }

        pub(crate) fn set_spin_attack_delay_timer(&mut self, value: u8) {
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = value;
        }

        pub(crate) fn decrement_spin_attack_delay_timer(&mut self) -> u8 {
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK]
        }

        pub(crate) fn decrement_sword_delay_timer(&mut self) -> u8 {
            self.ram[LINK_SWORD_DELAY_TIMER] = self.ram[LINK_SWORD_DELAY_TIMER].wrapping_sub(1);
            self.ram[LINK_SWORD_DELAY_TIMER]
        }

        pub(crate) fn set_sword_delay_timer(&mut self, value: u8) {
            self.ram[LINK_SWORD_DELAY_TIMER] = value;
        }

        pub(crate) fn clear_sword_delay_timer(&mut self) {
            self.ram[LINK_SWORD_DELAY_TIMER] = 0;
        }

        pub(crate) fn set_dash_countdown(&mut self, value: u8) {
            self.ram[LINK_COUNTDOWN_FOR_DASH] = value;
        }

        pub(crate) fn set_dash_counter(&mut self, value: u8) {
            self.ram[LINK_DASH_COUNTER] = value;
        }

        pub(crate) fn prime_dash_counter(&mut self) {
            self.ram[LINK_DASH_COUNTER] = 64;
        }

        pub(crate) fn decrement_dash_counter_clamped_to_minimum(&mut self, minimum: u8) {
            self.ram[LINK_DASH_COUNTER] = self.ram[LINK_DASH_COUNTER].wrapping_sub(1);
            if self.ram[LINK_DASH_COUNTER] < minimum {
                self.ram[LINK_DASH_COUNTER] = minimum;
            }
        }

        pub(crate) fn increment_dash_countdown(&mut self) -> u8 {
            self.ram[LINK_COUNTDOWN_FOR_DASH] = self.ram[LINK_COUNTDOWN_FOR_DASH].wrapping_add(1);
            self.ram[LINK_COUNTDOWN_FOR_DASH]
        }

        pub(crate) fn decrement_dash_countdown(&mut self) -> u8 {
            self.ram[LINK_COUNTDOWN_FOR_DASH] = self.ram[LINK_COUNTDOWN_FOR_DASH].wrapping_sub(1);
            self.ram[LINK_COUNTDOWN_FOR_DASH]
        }

        pub(crate) fn set_cape_mode(&mut self, value: u8) {
            self.ram[LINK_CAPE_MODE] = value;
        }

        pub(crate) fn clear_cape_mode(&mut self) {
            self.ram[LINK_CAPE_MODE] = 0;
        }

        pub(crate) fn increment_opening_pose(&mut self) {
            self.ram[LINK_POSE_DURING_OPENING] = self.ram[LINK_POSE_DURING_OPENING].wrapping_add(1);
        }

        pub(crate) fn set_item_action_debug_value_2(&mut self, value: u8) {
            self.ram[LINK_DEBUG_VALUE_2] = value;
        }

        pub(crate) fn clear_spin_attack_step_counter(&mut self) {
            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
        }

        pub(crate) fn increment_spin_attack_step_counter(&mut self) -> u8 {
            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] =
                self.ram[LINK_SPIN_ATTACK_STEP_COUNTER].wrapping_add(1);
            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER]
        }

        pub(crate) fn increment_spin_animation_step_counter(&mut self) -> u8 {
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] =
                self.ram[STEP_COUNTER_FOR_SPIN_ATTACK].wrapping_add(1);
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK]
        }

        pub(crate) fn clear_spin_animation_step_counter(&mut self) {
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
        }

        pub(crate) fn set_spin_offsets(&mut self, value: u8) {
            self.ram[LINK_SPIN_OFFSETS] = value;
        }

        pub(crate) fn clear_button_b_frames(&mut self) {
            self.ram[BUTTON_B_FRAMES] = 0;
        }

        pub(crate) fn clear_button_mask_b_y(&mut self) {
            self.ram[BUTTON_MASK_B_Y] = 0;
        }

        pub(crate) fn set_button_mask_b_y(&mut self, value: u8) {
            self.ram[BUTTON_MASK_B_Y] = value;
        }

        pub(crate) fn add_button_mask_b_y_bits(&mut self, bits: u8) {
            self.ram[BUTTON_MASK_B_Y] |= bits;
        }

        pub(crate) fn clear_button_mask_b_y_bits(&mut self, bits: u8) {
            self.ram[BUTTON_MASK_B_Y] &= !bits;
        }

        pub(crate) fn set_button_b_frames(&mut self, value: u8) {
            self.ram[BUTTON_B_FRAMES] = value;
        }

        pub(crate) fn set_button_b_frames_word(&mut self, value: u16) {
            write_le_u16(self.ram, BUTTON_B_FRAMES, value);
        }

        pub(crate) fn decrement_button_b_frames_word(&mut self) -> u16 {
            let frames = read_le_u16(self.ram, BUTTON_B_FRAMES).wrapping_sub(1);
            write_le_u16(self.ram, BUTTON_B_FRAMES, frames);
            frames
        }

        pub(crate) fn increment_button_b_frames(&mut self) -> u8 {
            self.ram[BUTTON_B_FRAMES] = self.ram[BUTTON_B_FRAMES].wrapping_add(1);
            self.ram[BUTTON_B_FRAMES]
        }

        pub(crate) fn set_y_button_action_flags(&mut self, value: u8) {
            self.ram[Y_BUTTON_ACTION_FLAGS] = value;
        }

        pub(crate) fn add_y_button_action_flag_bits(&mut self, bits: u8) {
            self.ram[Y_BUTTON_ACTION_FLAGS] |= bits;
        }

        pub(crate) fn clear_y_button_action_flags(&mut self) {
            self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        }

        pub(crate) fn set_y_button_action_step(&mut self, value: u8) {
            self.ram[Y_BUTTON_ACTION_STEP] = value;
        }

        pub(crate) fn clear_y_button_action_step(&mut self) {
            self.ram[Y_BUTTON_ACTION_STEP] = 0;
        }

        pub(crate) fn set_y_button_action_timer(&mut self, value: u8) {
            self.ram[Y_BUTTON_ACTION_TIMER] = value;
        }

        pub(crate) fn decrement_y_button_action_timer(&mut self) -> u8 {
            self.ram[Y_BUTTON_ACTION_TIMER] = self.ram[Y_BUTTON_ACTION_TIMER].wrapping_sub(1);
            self.ram[Y_BUTTON_ACTION_TIMER]
        }

        pub(crate) fn set_filtered_joypad_h(&mut self, value: u8) {
            self.ram[FILTERED_JOYPAD_H] = value;
        }

        pub(crate) fn set_filtered_joypad_l(&mut self, value: u8) {
            self.ram[FILTERED_JOYPAD_L] = value;
        }

        pub(crate) fn clear_filtered_joypad_l_bits(&mut self, bits: u8) {
            self.ram[FILTERED_JOYPAD_L] &= !bits;
        }

        pub(crate) fn set_joypad1h_last(&mut self, value: u8) {
            self.ram[JOYPAD1H_LAST] = value;
        }

        pub(crate) fn set_joypad1l_last(&mut self, value: u8) {
            self.ram[JOYPAD1L_LAST] = value;
        }

        pub(crate) fn set_joypad1h_last2(&mut self, value: u8) {
            self.ram[JOYPAD1H_LAST2] = value;
        }

        pub(crate) fn set_joypad1l_last2(&mut self, value: u8) {
            self.ram[JOYPAD1L_LAST2] = value;
        }

        pub(crate) fn set_item_action_step_var(&mut self, value: u8) {
            self.ram[LINK_ITEM_ACTION_STEP_SCRATCH] = value;
        }

        pub(crate) fn set_throw_oam_state_index(&mut self, value: u8) {
            self.ram[LINK_THROW_OAM_STATE_INDEX] = value;
        }

        pub(crate) fn clear_item_action_step_var(&mut self) {
            self.ram[LINK_ITEM_ACTION_STEP_SCRATCH] = 0;
        }

        pub(crate) fn increment_item_action_step_var(&mut self) -> u8 {
            self.ram[LINK_ITEM_ACTION_STEP_SCRATCH] =
                self.ram[LINK_ITEM_ACTION_STEP_SCRATCH].wrapping_add(1);
            self.ram[LINK_ITEM_ACTION_STEP_SCRATCH]
        }

        pub(crate) fn advance_item_action_step_var_wrapping_7_to_1(&mut self) -> u8 {
            self.ram[LINK_ITEM_ACTION_STEP_SCRATCH] =
                if self.ram[LINK_ITEM_ACTION_STEP_SCRATCH].wrapping_add(1) == 7 {
                    1
                } else {
                    self.ram[LINK_ITEM_ACTION_STEP_SCRATCH].wrapping_add(1)
                };
            self.ram[LINK_ITEM_ACTION_STEP_SCRATCH]
        }

        pub(crate) fn clear_near_moveable_statue(&mut self) {
            self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 0;
        }

        pub(crate) fn mark_near_moveable_statue(&mut self) {
            self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 1;
        }

        pub(crate) fn clear_pull_for_rupees_sprite_need(&mut self) {
            self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
        }

        pub(crate) fn set_pull_for_rupees_sprite_need(&mut self) {
            self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 1;
        }

        pub(crate) fn set_pull_action_state(&mut self, value: u8) {
            self.ram[LINK_PULL_ACTION_STATE] = value;
        }

        pub(crate) fn increment_pull_action_state(&mut self) {
            self.ram[LINK_PULL_ACTION_STATE] = self.ram[LINK_PULL_ACTION_STATE].wrapping_add(1);
        }

        pub(crate) fn prevent_movement(&mut self) {
            self.ram[LINK_PREVENT_FROM_MOVING] = 1;
        }

        pub(crate) fn clear_prevent_movement(&mut self) {
            self.ram[LINK_PREVENT_FROM_MOVING] = 0;
        }

        pub(crate) fn clear_frame_change_counter(&mut self) {
            self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
        }

        pub(crate) fn set_faint_animation_active(&mut self, value: u8) {
            self.ram[LINK_FAINT_ANIMATION_ACTIVE] = value;
        }

        pub(crate) fn clear_given_damage(&mut self) {
            self.ram[LINK_GIVE_DAMAGE] = 0;
        }

        pub(crate) fn set_given_damage(&mut self, value: u8) {
            self.ram[LINK_GIVE_DAMAGE] = value;
        }

        pub(crate) fn force_hold_sword_up(&mut self) {
            self.ram[LINK_FORCE_HOLD_SWORD_UP] = 1;
        }

        pub(crate) fn clear_force_hold_sword_up(&mut self) {
            self.ram[LINK_FORCE_HOLD_SWORD_UP] = 0;
        }

        pub(crate) fn clear_transforming(&mut self) {
            self.ram[LINK_IS_TRANSFORMING] = 0;
        }

        pub(crate) fn set_transforming(&mut self) {
            self.ram[LINK_IS_TRANSFORMING] = 1;
        }

        pub(crate) fn set_sprite_oam_state_timer(&mut self, value: u8) {
            self.ram[LINK_SPRITE_OAM_STATE_TIMER] = value;
        }

        pub(crate) fn mark_pit_landing_oam_state(&mut self) {
            self.ram[LINK_SPRITE_OAM_STATE_TIMER] = 9;
        }

        pub(crate) fn set_receive_item_index(&mut self, value: u8) {
            self.ram[LINK_RECEIVE_ITEM_INDEX] = value;
        }

        pub(crate) fn set_item_holding_timer(&mut self, value: u8) {
            self.ram[LINK_ITEM_HOLDING_TIMER] = value;
        }

        pub(crate) fn set_item_hold_pose(&mut self, value: u8) {
            self.ram[LINK_POSE_FOR_ITEM] = value;
        }

        pub(crate) fn clear_item_hold_pose(&mut self) {
            self.ram[LINK_POSE_FOR_ITEM] = 0;
        }

        pub(crate) fn set_link_dma_staging_index(&mut self, value: u8) {
            self.ram[LINK_DMA_STAGING_INDEX] = value;
        }

        pub(crate) fn set_immobilized_flag(&mut self, value: u8) {
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = value;
        }

        pub(crate) fn immobilize(&mut self) {
            self.set_immobilized_flag(1);
        }

        pub(crate) fn clear_immobilized(&mut self) {
            self.set_immobilized_flag(0);
        }

        pub(crate) fn increment_immobilized_flag(&mut self) -> u8 {
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = self.ram[FLAG_IS_LINK_IMMOBILIZED].wrapping_add(1);
            self.ram[FLAG_IS_LINK_IMMOBILIZED]
        }

        pub(crate) fn set_menu_block_flag(&mut self, value: u8) {
            self.ram[FLAG_BLOCK_LINK_MENU] = value;
        }

        pub(crate) fn clear_menu_block(&mut self) {
            self.set_menu_block_flag(0);
        }

        pub(crate) fn increment_menu_block_flag(&mut self) -> u8 {
            self.ram[FLAG_BLOCK_LINK_MENU] = self.ram[FLAG_BLOCK_LINK_MENU].wrapping_add(1);
            self.ram[FLAG_BLOCK_LINK_MENU]
        }

        pub(crate) fn set_link_dma_graphics_index_word(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_DMA_GRAPHICS_INDEX, value);
        }

        pub(crate) fn set_link_dma_left_sprite_bank_word(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_DMA_LEFT_SPRITE_BANK_INDEX, value);
        }

        pub(crate) fn set_link_dma_right_sprite_bank_word(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_DMA_RIGHT_SPRITE_BANK_INDEX, value);
        }

        pub(crate) fn clear_link_dma_sprite_banks(&mut self) {
            self.set_link_dma_left_sprite_bank_word(0);
            self.set_link_dma_right_sprite_bank_word(0);
        }

        pub(crate) fn set_palette_bits_of_oam_word(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_PALETTE_BITS_OF_OAM, value);
        }

        pub(crate) fn advance_link_dma_source_offset(&mut self) -> u16 {
            let mut source_offset =
                read_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET).wrapping_add(0x400);
            if source_offset == 0x0c00 {
                source_offset = 0;
            }
            write_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET, source_offset);
            source_offset
        }

        pub(crate) fn advance_link_dma_tile_offset(&mut self) -> u16 {
            let mut tile_offset = read_le_u16(self.ram, LINK_DMA_TILE_OFFSET).wrapping_add(2);
            if tile_offset == 12 {
                tile_offset = 0;
            }
            write_le_u16(self.ram, LINK_DMA_TILE_OFFSET, tile_offset);
            tile_offset
        }

        pub(crate) fn set_link_dma_countdown(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_DMA_COUNTDOWN, value);
        }

        pub(crate) fn decrement_link_dma_countdown(&mut self) -> u16 {
            let countdown = read_le_u16(self.ram, LINK_DMA_COUNTDOWN).wrapping_sub(1);
            write_le_u16(self.ram, LINK_DMA_COUNTDOWN, countdown);
            countdown
        }

        pub(crate) fn reset_link_dma_animation_cycle(&mut self, countdown: u16) {
            self.set_link_dma_countdown(countdown);
            write_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET, 0);
            write_le_u16(self.ram, LINK_DMA_TILE_OFFSET, 0);
        }

        pub(crate) fn set_sword_dma_graphics_index(&mut self, value: u8) {
            self.ram[LINK_DMA_SWORD_GRAPHICS_INDEX] = value;
        }

        pub(crate) fn set_shield_dma_graphics_index(&mut self, value: u8) {
            self.ram[LINK_DMA_SHIELD_GRAPHICS_INDEX] = value;
        }

        pub(crate) fn decrement_sprite_oam_state_timer(&mut self) -> u8 {
            self.ram[LINK_SPRITE_OAM_STATE_TIMER] =
                self.ram[LINK_SPRITE_OAM_STATE_TIMER].wrapping_sub(1);
            self.ram[LINK_SPRITE_OAM_STATE_TIMER]
        }

        pub(crate) fn set_speed_setting(&mut self, value: u8) {
            self.ram[LINK_SPEED_SETTING] = value;
        }

        pub(crate) fn decrement_speed_setting(&mut self) -> u8 {
            self.ram[LINK_SPEED_SETTING] = self.ram[LINK_SPEED_SETTING].wrapping_sub(1);
            self.ram[LINK_SPEED_SETTING]
        }

        pub(crate) fn set_flag_moving(&mut self, value: u8) {
            self.ram[LINK_FLAG_MOVING] = value;
        }

        pub(crate) fn start_running(&mut self) {
            self.ram[LINK_IS_RUNNING] = 1;
        }

        pub(crate) fn set_running_state(&mut self, value: u8) {
            self.ram[LINK_IS_RUNNING] = value;
        }

        pub(crate) fn clear_running(&mut self) {
            self.ram[LINK_IS_RUNNING] = 0;
        }

        pub(crate) fn arm_stair_speed_modifier(&mut self) {
            self.ram[LINK_SPEED_SETTING] = 2;
            self.ram[LINK_SPEED_MODIFIER] = 1;
        }

        pub(crate) fn resolve_dash_speed_setting(&mut self) {
            if self.ram[LINK_SPEED_SETTING] == 2 {
                self.ram[LINK_SPEED_SETTING] = if self.ram[LINK_IS_RUNNING] != 0 {
                    16
                } else {
                    0
                };
            }
        }

        pub(crate) fn promote_pending_speed_modifier(&mut self) {
            if self.ram[LINK_SPEED_MODIFIER] == 1 {
                self.ram[LINK_SPEED_MODIFIER] = 2;
            }
        }

        pub(crate) fn increase_near_pit_speed_modifier(&mut self) {
            self.ram[LINK_SPEED_MODIFIER] = if self.ram[LINK_SPEED_MODIFIER] < 48 {
                self.ram[LINK_SPEED_MODIFIER].wrapping_add(8)
            } else {
                32
            };
        }

        pub(crate) fn advance_dash_deceleration(&mut self) {
            self.ram[LINK_SPEED_MODIFIER] = self.ram[LINK_SPEED_MODIFIER].wrapping_add(1);
        }

        pub(crate) fn enter_water_hop_state(&mut self) {
            if self.ram[LINK_AUXILIARY_STATE] != 2 {
                self.ram[LINK_AUXILIARY_STATE] = 1;
                self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
            }
            self.ram[LINK_HANDLER_STATE] = 6;
        }

        pub(crate) fn clear_bunny_mirror(&mut self) {
            self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        }

        pub(crate) fn clear_bunny_body_state(&mut self) {
            self.ram[LINK_IS_BUNNY] = 0;
        }

        pub(crate) fn set_bunny_state(&mut self, value: u8) {
            self.ram[LINK_IS_BUNNY] = value;
            self.ram[LINK_IS_BUNNY_MIRROR] = value;
        }

        pub(crate) fn start_bunny_transform_poof(&mut self) {
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 1;
            self.ram[LINK_VISIBILITY_STATUS] = 12;
        }

        pub(crate) fn finish_bunny_transform_poof(&mut self) {
            self.ram[LINK_IS_BUNNY_MIRROR] = 1;
            self.ram[LINK_IS_BUNNY] = 1;
            self.ram[LINK_VISIBILITY_STATUS] = 0;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        }

        pub(crate) fn clear_bunny_transform_flags(&mut self) {
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
            self.ram[LINK_IS_BUNNY] = 0;
            self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        }

        pub(crate) fn clear_bunny_transform_after_moon_pearl(&mut self) {
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
            self.ram[LINK_IS_BUNNY] = 0;
            self.ram[LINK_IS_BUNNY_MIRROR] = 0;
            self.ram[LINK_TIMER_TEMPBUNNY] = 0;
        }

        pub(crate) fn clear_transform_poof_need_and_temp_bunny_timer(&mut self) {
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
            write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, 0);
        }

        pub(crate) fn clear_auxiliary_state(&mut self) {
            self.ram[LINK_AUXILIARY_STATE] = 0;
        }

        pub(crate) fn set_auxiliary_state(&mut self, value: u8) {
            self.ram[LINK_AUXILIARY_STATE] = value;
        }

        pub(crate) fn clear_handler_state(&mut self) {
            self.ram[LINK_HANDLER_STATE] = 0;
        }

        pub(crate) fn set_handler_state(&mut self, value: u8) {
            self.ram[LINK_HANDLER_STATE] = value;
        }

        pub(crate) fn set_facing(&mut self, value: u8) {
            self.ram[LINK_FACING] = value;
        }

        pub(crate) fn set_facing_mirror(&mut self, value: u8) {
            self.ram[LINK_FACING_MIRROR] = value;
        }

        pub(crate) fn cache_facing_to_mirror(&mut self) {
            self.ram[LINK_FACING_MIRROR] = self.ram[LINK_FACING];
        }

        pub(crate) fn cache_facing(&mut self) {
            self.ram[LINK_FACING_CACHED] = self.ram[LINK_FACING];
        }

        pub(crate) fn cache_lower_level_states(&mut self) {
            self.ram[LINK_IS_ON_LOWER_LEVEL_CACHED] = self.ram[LINK_IS_ON_LOWER_LEVEL];
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED] =
                self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
        }

        pub(crate) fn land_after_splash(&mut self) {
            self.ram[LINK_HANDLER_STATE] = if self.ram[LINK_IS_BUNNY_MIRROR] != 0 {
                if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
                    3
                } else {
                    23
                }
            } else if self.ram[LINK_IS_IN_DEEP_WATER] != 0 {
                4
            } else {
                0
            };
        }

        pub(crate) fn interrupt_swimming_for_auxiliary_state(&mut self) {
            self.ram[LINK_HANDLER_STATE] = 2;
            self.clear_z_high();
            self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
            self.ram[LINK_SWIM_HARD_STROKE] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        }

        pub(crate) fn clear_swimming_action_state(&mut self) {
            self.ram[BUTTON_MASK_B_Y] = 0;
            self.ram[BUTTON_B_FRAMES] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
        }

        pub(crate) fn clear_swim_fast_state(&mut self) {
            self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
        }

        pub(crate) fn advance_idle_swim_animation(&mut self) {
            self.ram[LINK_ANIMATION_STEPS] &= 1;
            self.ram[LINK_FRAME_CHANGE_COUNTER] =
                self.ram[LINK_FRAME_CHANGE_COUNTER].wrapping_add(1);
            if self.ram[LINK_FRAME_CHANGE_COUNTER] >= 16 {
                self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
                self.ram[SWIM_STROKE_ANIM_STEP] = 0;
                self.ram[LINK_ANIMATION_STEPS] = (self.ram[LINK_ANIMATION_STEPS] & 1) ^ 1;
            }
        }

        pub(crate) fn advance_active_swim_animation(&mut self, stroke_steps: &[u8; 4]) {
            self.ram[LINK_FRAME_CHANGE_COUNTER] =
                self.ram[LINK_FRAME_CHANGE_COUNTER].wrapping_add(1);
            if self.ram[LINK_FRAME_CHANGE_COUNTER] >= 8 {
                self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
                self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1) & 3;
                self.ram[SWIM_STROKE_ANIM_STEP] =
                    stroke_steps[self.ram[LINK_ANIMATION_STEPS] as usize];
            }
        }

        pub(crate) fn start_hard_swim_stroke(&mut self, hard_stroke: u8) {
            self.ram[LINK_SWIM_HARD_STROKE] = hard_stroke;
            self.ram[LINK_MAYBE_SWIM_FASTER] = 1;
            self.ram[SWIMMING_COUNTDOWN] = 7;
        }

        pub(crate) fn tick_hard_swim_stroke(&mut self) {
            self.ram[SWIMMING_COUNTDOWN] = self.ram[SWIMMING_COUNTDOWN].wrapping_sub(1);
            if (self.ram[SWIMMING_COUNTDOWN] as i8).is_negative() {
                self.ram[SWIMMING_COUNTDOWN] = 7;
                self.ram[LINK_MAYBE_SWIM_FASTER] = self.ram[LINK_MAYBE_SWIM_FASTER].wrapping_add(1);
                if self.ram[LINK_MAYBE_SWIM_FASTER] == 5 {
                    self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
                    self.ram[LINK_SWIM_HARD_STROKE] &= !0xc0;
                }
            }
        }

        pub(crate) fn clear_swim_movement_velocity(&mut self) {
            self.ram[LINK_Y_VELOCITY] = 0;
            self.ram[LINK_X_VELOCITY] = 0;
        }

        pub(crate) fn reset_idle_swim_animation_if_out_of_water(&mut self) {
            if self.ram[LINK_HANDLER_STATE] != 4 {
                self.ram[LINK_ANIMATION_STEPS] = 0;
            }
        }

        pub(crate) fn set_swim_direction_flags(&mut self, direction: u8) {
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = direction;
        }

        pub(crate) fn reset_swim_subpixel_and_defense_state(&mut self) {
            self.ram[LINK_X_SUBPIXEL] = 0;
            self.ram[LINK_Y_SUBPIXEL] = 0;
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        }

        pub(crate) fn clear_defense_flags(&mut self) {
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        }

        pub(crate) fn set_defense_flags(&mut self, value: u8) {
            self.ram[PLAYER_DEFENSE_FLAGS] = value;
        }

        pub(crate) fn or_defense_flags(&mut self, value: u8) {
            self.ram[PLAYER_DEFENSE_FLAGS] |= value;
        }

        pub(crate) fn and_defense_flags(&mut self, value: u8) {
            self.ram[PLAYER_DEFENSE_FLAGS] &= value;
        }

        pub(crate) fn clear_action_handler_timer(&mut self) {
            self.ram[PLAYER_HANDLER_TIMER] = 0;
        }

        pub(crate) fn set_action_handler_timer(&mut self, value: u8) {
            self.ram[PLAYER_HANDLER_TIMER] = value;
        }

        pub(crate) fn increment_action_handler_timer(&mut self) -> u8 {
            self.ram[PLAYER_HANDLER_TIMER] = self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1);
            self.ram[PLAYER_HANDLER_TIMER]
        }

        pub(crate) fn clear_doorway_state(&mut self) {
            self.ram[IS_STANDING_IN_DOORWAY] = 0;
        }

        pub(crate) fn set_doorway_state(&mut self, value: u8) {
            self.ram[IS_STANDING_IN_DOORWAY] = value;
        }

        pub(crate) fn clear_blink_countdown(&mut self) {
            self.ram[COUNTDOWN_FOR_BLINK] = 0;
        }

        pub(crate) fn set_blink_countdown(&mut self, value: u8) {
            self.ram[COUNTDOWN_FOR_BLINK] = value;
        }

        pub(crate) fn decrement_blink_countdown(&mut self) -> u8 {
            self.ram[COUNTDOWN_FOR_BLINK] = self.ram[COUNTDOWN_FOR_BLINK].wrapping_sub(1);
            self.ram[COUNTDOWN_FOR_BLINK]
        }

        pub(crate) fn set_item_receipt_method(&mut self, value: u8) {
            self.ram[ITEM_RECEIPT_METHOD] = value;
        }

        pub(crate) fn clear_ancilla_pickup_flag(&mut self) {
            self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
        }

        pub(crate) fn set_ancilla_pickup_flag(&mut self, value: u8) {
            self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = value;
        }

        pub(crate) fn set_spin_attack_step_counter(&mut self, value: u8) {
            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = value;
        }

        pub(crate) fn set_spin_animation_step_counter(&mut self, value: u8) {
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = value;
        }

        pub(crate) fn clear_pit_correction(&mut self) {
            self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
        }

        pub(crate) fn cancel_dash_state(&mut self) {
            self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            self.ram[LINK_IS_RUNNING] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
            write_le_u16(self.ram, SWIM_ACCELERATION_MODE, 0);
        }

        pub(crate) fn set_last_direction_moved_towards_from_facing(&mut self) {
            self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = self.ram[LINK_FACING] >> 1;
        }

        pub(crate) fn clear_animation_step(&mut self) {
            self.ram[LINK_ANIMATION_STEPS] = 0;
        }

        pub(crate) fn set_animation_step(&mut self, value: u8) {
            self.ram[LINK_ANIMATION_STEPS] = value;
        }

        pub(crate) fn clear_animation_step_if_at_least(&mut self, threshold: u8) {
            if self.ram[LINK_ANIMATION_STEPS] >= threshold {
                self.clear_animation_step();
            }
        }

        pub(crate) fn subtract_animation_step_if_at_least(&mut self, threshold: u8, delta: u8) {
            if self.ram[LINK_ANIMATION_STEPS] >= threshold {
                self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_sub(delta);
            }
        }

        pub(crate) fn set_item_in_hand(&mut self, value: u8) {
            self.ram[LINK_ITEM_IN_HAND] = value;
        }

        pub(crate) fn clear_item_in_hand(&mut self) {
            self.ram[LINK_ITEM_IN_HAND] = 0;
        }

        pub(crate) fn clear_item_in_hand_bits(&mut self, mask: u8) {
            self.ram[LINK_ITEM_IN_HAND] &= !mask;
        }

        pub(crate) fn clear_position_mode(&mut self) {
            self.ram[LINK_POSITION_MODE] = 0;
        }

        pub(crate) fn set_position_mode(&mut self, value: u8) {
            self.ram[LINK_POSITION_MODE] = value;
        }

        pub(crate) fn set_position_mode_bits(&mut self, mask: u8) {
            self.ram[LINK_POSITION_MODE] |= mask;
        }

        pub(crate) fn clear_position_mode_bits(&mut self, mask: u8) {
            self.ram[LINK_POSITION_MODE] &= !mask;
        }

        pub(crate) fn set_state_bits(&mut self, value: u8) {
            self.ram[LINK_STATE_BITS] = value;
        }

        pub(crate) fn clear_state_bits(&mut self) {
            self.ram[LINK_STATE_BITS] = 0;
        }

        pub(crate) fn clear_lifting_or_carrying_state(&mut self) {
            self.ram[LINK_STATE_BITS] &= !0x80;
        }

        pub(crate) fn keep_only_lifting_or_carrying_state(&mut self) {
            self.ram[LINK_STATE_BITS] &= 0x80;
        }

        pub(crate) fn enter_item_hold_pose(&mut self) {
            self.ram[LINK_STATE_BITS] = 0x80;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_FACING] = 0;
            self.ram[LINK_ANIMATION_STEPS] = 0;
        }

        pub(crate) fn clear_state_item_and_grab_flags(&mut self) {
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
        }

        pub(crate) fn clear_picking_throw_state(&mut self) {
            self.ram[LINK_PICKING_THROW_STATE] = 0;
        }

        pub(crate) fn set_picking_throw_state(&mut self, value: u8) {
            self.ram[LINK_PICKING_THROW_STATE] = value;
        }

        pub(crate) fn clear_grabbing_wall(&mut self) {
            self.ram[LINK_GRABBING_WALL] = 0;
        }

        pub(crate) fn set_grabbing_wall(&mut self, value: u8) {
            self.ram[LINK_GRABBING_WALL] = value;
        }

        pub(crate) fn start_lift_throw_state(&mut self) {
            self.ram[LINK_PICKING_THROW_STATE] = 1;
            self.ram[LINK_STATE_BITS] = 0x80;
        }

        pub(crate) fn set_cape_transform_timer(&mut self, value: u8) {
            self.ram[LINK_BUNNY_TRANSFORM_TIMER] = value;
        }

        pub(crate) fn decrement_push_fatigue_timer(&mut self) -> u8 {
            self.ram[LINK_TIMER_PUSH_GET_TIRED] =
                self.ram[LINK_TIMER_PUSH_GET_TIRED].wrapping_sub(1);
            self.ram[LINK_TIMER_PUSH_GET_TIRED]
        }

        pub(crate) fn set_push_fatigue_timer(&mut self, value: u8) {
            self.ram[LINK_TIMER_PUSH_GET_TIRED] = value;
        }

        pub(crate) fn reset_push_fatigue_timer(&mut self) {
            self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
        }

        pub(crate) fn tick_cape_transform_timer(&mut self) -> u8 {
            self.ram[LINK_BUNNY_TRANSFORM_TIMER] =
                self.ram[LINK_BUNNY_TRANSFORM_TIMER].wrapping_sub(1);
            self.ram[LINK_BUNNY_TRANSFORM_TIMER]
        }

        pub(crate) fn clear_cape_transform_timer(&mut self) {
            self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 0;
        }

        pub(crate) fn set_current_item_y(&mut self, value: u8) {
            self.ram[LINK_CURRENT_ITEM_Y] = value;
        }

        pub(crate) fn increment_sleep_in_bed_state(&mut self) {
            self.ram[PLAYER_SLEEP_IN_BED_STATE] =
                self.ram[PLAYER_SLEEP_IN_BED_STATE].wrapping_add(1);
        }

        pub(crate) fn set_bit9_of_xcoord_word(&mut self, value: u16) {
            write_le_u16(self.ram, BIT9_OF_XCOORD, value);
        }

        /// Stashes the selected Link body sprite table index in the shared
        /// scratch word at 0x74 for the player OAM routines.
        pub(crate) fn set_link_sprite_index_scratch(&mut self, value: u16) {
            write_le_u16(self.ram, SCRATCH_1, value);
        }

        pub(crate) fn set_primary_water_grass_timer(&mut self, value: u8) {
            self.ram[PRIMARY_WATER_GRASS_TIMER] = value;
        }

        pub(crate) fn set_secondary_water_grass_timer(&mut self, value: u8) {
            self.ram[SECONDARY_WATER_GRASS_TIMER] = value;
        }

        pub(crate) fn clear_item_debug_value_1(&mut self) {
            self.ram[LINK_DEBUG_VALUE_1] = 0;
        }

        pub(crate) fn clear_action_scratch_state(&mut self) {
            self.ram[LINK_DEBUG_VALUE_1] = 0;
            self.ram[LINK_DEBUG_VALUE_2] = 0;
            self.ram[LINK_ITEM_ACTION_STEP_SCRATCH] = 0;
            self.ram[LINK_THROW_OAM_STATE_INDEX] = 0;
        }

        pub(crate) fn clear_lift_throw_scratch_state(&mut self) {
            self.ram[LINK_ITEM_ACTION_STEP_SCRATCH] = 0;
            self.ram[LINK_THROW_OAM_STATE_INDEX] = 0;
        }

        pub(crate) fn spend_magic(&mut self, cost: u8) -> bool {
            let new_magic = self.ram[LINK_MAGIC_POWER].wrapping_sub(cost);
            if self.ram[LINK_MAGIC_POWER] != 0 && new_magic < 0x80 {
                self.ram[LINK_MAGIC_POWER] = new_magic;
                true
            } else {
                false
            }
        }

        pub(crate) fn refund_magic(&mut self, cost: u8, clamp_full: bool) {
            let mut new_magic = self.ram[LINK_MAGIC_POWER] as u16 + cost as u16;
            if clamp_full && new_magic >= 128 {
                new_magic = 128;
            }
            self.ram[LINK_MAGIC_POWER] = new_magic as u8;
        }

        pub(crate) fn decrement_magic_power(&mut self) -> u8 {
            self.ram[LINK_MAGIC_POWER] = self.ram[LINK_MAGIC_POWER].wrapping_sub(1);
            self.ram[LINK_MAGIC_POWER]
        }

        pub(crate) fn advance_animation_step(&mut self, wrap_at: u8, wrap_to: u8) {
            self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1);
            if self.ram[LINK_ANIMATION_STEPS] == wrap_at {
                self.ram[LINK_ANIMATION_STEPS] = wrap_to;
            }
        }

        pub(crate) fn advance_animation_step_at_least(&mut self, wrap_at: u8, wrap_to: u8) {
            self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1);
            if self.ram[LINK_ANIMATION_STEPS] >= wrap_at {
                self.ram[LINK_ANIMATION_STEPS] = wrap_to;
            }
        }

        pub(crate) fn initialize_link_action_state(&mut self) {
            self.ram[LINK_FACING] = 2;
            self.ram[LINK_LAST_DIRECTION] = 0;
            self.ram[LINK_ITEM_IN_HAND] = 0;
            self.ram[LINK_POSITION_MODE] = 0;
            self.ram[LINK_DEBUG_VALUE_1] = 0;
            self.ram[LINK_DEBUG_VALUE_2] = 0;
            self.ram[LINK_ITEM_ACTION_STEP_SCRATCH] = 0;
            self.ram[LINK_THROW_OAM_STATE_INDEX] = 0;
            self.ram[Y_BUTTON_ACTION_STEP] = 0;
            self.ram[LINK_IS_TRANSFORMING] = 0;
            self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
            self.ram[BUTTON_MASK_B_Y] &= !0x40;
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
        }

        pub(crate) fn finish_link_action_state_initialization(&mut self) {
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            self.clear_z_high();
            self.ram[LINK_AUXILIARY_STATE] = 0;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
            self.ram[COUNTDOWN_FOR_BLINK] = 0;
            self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
            self.ram[LINK_POSE_FOR_ITEM] = 0;
            self.ram[LINK_CAPE_MODE] = 0;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_DIRECTION] &= !0x0f;
            self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
        }

        pub(crate) fn clear_misc_bugfix_movement_state(&mut self) {
            self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 0;
            self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 0;
            self.ram[LINK_ON_CONVEYOR_BELT] = 0;
            self.ram[LINK_FLAG_MOVING] = 0;
        }

        pub(crate) fn become_bunny_handler(&mut self) {
            self.ram[LINK_HANDLER_STATE] = 23;
            self.ram[LINK_IS_BUNNY] = 1;
            self.ram[LINK_IS_BUNNY_MIRROR] = 1;
        }

        pub(crate) fn reset_properties_a_fields(&mut self) {
            self.ram[LINK_LAST_DIRECTION] = 0;
            self.ram[LINK_DIRECTION] = 0;
            self.ram[LINK_FLAG_MOVING] = 0;
            self.ram[LINK_IS_TRANSFORMING] = 0;
            self.ram[COUNTDOWN_FOR_BLINK] = 0;
            self.ram[PLAYER_RESET_ANCILLA_WORK_BYTE_24] = 0;
            self.ram[LINK_IS_BUNNY] = 0;
            self.ram[LINK_IS_BUNNY_MIRROR] = 0;
            self.ram[LINK_TIMER_TEMPBUNNY] = 0;
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
            self.ram[IS_ARCHER_OR_SHOVEL_GAME] = 0;
            self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
            self.ram[BIT9_OF_XCOORD] = 0;
            self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 0;
            self.ram[LINK_GIVE_DAMAGE] = 0;
            self.ram[LINK_SPIN_OFFSETS] = 0;
            self.ram[TAGALONG_EVENT_FLAGS] = 0;
            self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 0;
            self.ram[TILEDETECT_TILE_TYPE] = 0;
            self.ram[ITEM_RECEIPT_METHOD] = 0;
            self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 0;
        }

        pub(crate) fn reset_properties_b_fields(&mut self) {
            self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
            self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
            self.ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED] = 0;
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
            self.ram[PLAYER_PIT_DATA_INDEX] = 0;
            self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        }

        pub(crate) fn clear_custom_spell_animation(&mut self) {
            self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 0;
        }

        pub(crate) fn reset_properties_c_fields(&mut self) {
            self.ram[TILE_ACTION_INDEX] = 0;
            self.ram[STATE_FOR_SPIN_ATTACK] = 0;
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
            self.ram[TILE_COLL_FLAG] = 0;
            self.ram[LINK_FORCE_HOLD_SWORD_UP] = 0;
            self.ram[LINK_SWORD_DELAY_TIMER] = 0;
            write_le_u16(self.ram, TILEDETECT_MISC_TILES, 0);
            self.ram[LINK_ITEM_IN_HAND] = 0;
            self.ram[LINK_POSITION_MODE] = 0;
            self.ram[LINK_DEBUG_VALUE_1] = 0;
            self.ram[LINK_DEBUG_VALUE_2] = 0;
            self.ram[LINK_ITEM_ACTION_STEP_SCRATCH] = 0;
            self.ram[LINK_THROW_OAM_STATE_INDEX] = 0;
            self.ram[Y_BUTTON_ACTION_STEP] = 0;
            self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
            self.ram[BUTTON_MASK_B_Y] = 0;
            self.ram[BUTTON_B_FRAMES] = 0;
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
            self.ram[LINK_AUXILIARY_STATE] = 0;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
            self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
            self.ram[LINK_POSE_FOR_ITEM] = 0;
            self.ram[LINK_CAPE_MODE] = 0;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[RELATED_TO_HOOKSHOT] = 0;
            self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
            self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = 0;
            self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
            self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 0;
        }

        pub(crate) fn setup_bed_pose(&mut self) {
            self.ram[LINK_HANDLER_STATE] = 0x16;
            self.ram[PLAYER_SLEEP_IN_BED_STATE] = 0;
            self.ram[LINK_POSE_DURING_OPENING] = 0;
            self.ram[LINK_COUNTDOWN_FOR_DASH] = 3;
        }

        pub(crate) fn reset_swimming_state_fields(&mut self) {
            self.ram[SWIMMING_COUNTDOWN] = 0;
            self.ram[LINK_SWIM_HARD_STROKE] = 0;
            self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
        }

        pub(crate) fn reset_after_damaging_pit(&mut self) {
            self.ram[LINK_HANDLER_STATE] =
                if self.ram[LINK_IS_BUNNY] != 0 && self.ram[LINK_ITEM_MOON_PEARL] == 0 {
                    23
                } else {
                    0
                };
            self.ram[LINK_LAST_DIRECTION] = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
            self.ram[LINK_IS_IN_DEEP_WATER] = 0;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
            self.ram[PLAYER_PIT_DATA_INDEX] = 0;
            self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        }

        pub(crate) fn recache_bunny_state(&mut self) {
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
            write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, 0);
            if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
                self.ram[LINK_IS_BUNNY] = 0;
                self.ram[LINK_AUXILIARY_STATE] = 0;
            }
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[LINK_IS_TRANSFORMING] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        }

        pub(crate) fn enter_deep_water(&mut self) {
            self.ram[LINK_IS_IN_DEEP_WATER] = 1;
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_LAST_DIRECTION];
            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
        }

        pub(crate) fn cache_safe_return_position_from_current(&mut self) {
            self.ram[LINK_X_COORD_SAFE_RETURN_LO] = self.ram[LINK_X_COORD];
            self.ram[LINK_X_COORD_SAFE_RETURN_HI] = self.ram[LINK_X_COORD + 1];
            self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = self.ram[LINK_Y_COORD];
            self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = self.ram[LINK_Y_COORD + 1];
        }

        pub(crate) fn clear_force_move_high_byte(&mut self) {
            let lo = self.ram[FORCE_MOVE_ANY_DIRECTION];
            write_le_u16(self.ram, FORCE_MOVE_ANY_DIRECTION, lo as u16);
        }

        pub(crate) fn set_sprite_pickup_flag(&mut self, value: u8) {
            self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = value;
        }

        pub(crate) fn clear_sprite_pickup_flag(&mut self) {
            self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = 0;
        }

        pub(crate) fn set_drag_player_x(&mut self, value: u16) {
            write_le_u16(self.ram, DRAG_PLAYER_X, value);
        }

        pub(crate) fn set_drag_player_y(&mut self, value: u16) {
            write_le_u16(self.ram, DRAG_PLAYER_Y, value);
        }

        pub(crate) fn add_drag_player_x(&mut self, delta: u16) {
            let cur = word(self.ram, DRAG_PLAYER_X);
            write_le_u16(self.ram, DRAG_PLAYER_X, cur.wrapping_add(delta));
        }

        pub(crate) fn add_drag_player_y(&mut self, delta: u16) {
            let cur = word(self.ram, DRAG_PLAYER_Y);
            write_le_u16(self.ram, DRAG_PLAYER_Y, cur.wrapping_add(delta));
        }

        pub(crate) fn clear_somaria_block_bg_check_flag(&mut self) {
            self.ram[SOMARIA_BLOCK_BG_CHECK_FLAG] = 0;
        }

        pub(crate) fn clear_player_pose_draw_counter(&mut self) {
            self.ram[PLAYER_POSE_DRAW_COUNTER] = 0;
        }

        pub(crate) fn increment_player_pose_draw_counter(&mut self) {
            self.ram[PLAYER_POSE_DRAW_COUNTER] = self.ram[PLAYER_POSE_DRAW_COUNTER].wrapping_add(1);
        }

        pub(crate) fn clear_player_special_draw_flag(&mut self) {
            self.ram[PLAYER_SPECIAL_DRAW_FLAG] = 0;
        }
    }

    pub(crate) struct SpecialExitPositionView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SpecialExitPositionView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn x(&self) -> u16 {
            word(self.ram, LINK_X_COORD_SPEXIT)
        }

        pub(crate) fn y(&self) -> u16 {
            word(self.ram, LINK_Y_COORD_SPEXIT)
        }

        pub(crate) fn map_zoom_y(&self) -> u16 {
            ((self.y() >> 4).wrapping_sub(0x48)) & !1
        }

        pub(crate) fn map_zoom_x_offset(&self) -> u16 {
            (self.x() >> 4).wrapping_sub(0x80)
        }
    }

    pub(crate) struct SpecialExitPositionViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SpecialExitPositionViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_X_COORD_SPEXIT, value);
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_Y_COORD_SPEXIT, value);
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            self.set_x(x);
            self.set_y(y);
        }

        pub(crate) fn offset_position(&mut self, x_delta: u16, y_delta: u16) {
            let x = word(self.ram, LINK_X_COORD_SPEXIT).wrapping_add(x_delta);
            let y = word(self.ram, LINK_Y_COORD_SPEXIT).wrapping_add(y_delta);
            self.set_position(x, y);
        }

        pub(crate) fn store_from_player(&mut self) {
            copy_word(self.ram, LINK_Y_COORD_SPEXIT, LINK_Y_COORD);
            copy_word(self.ram, LINK_X_COORD_SPEXIT, LINK_X_COORD);
        }

        pub(crate) fn restore_player_position(&mut self) {
            copy_word(self.ram, LINK_X_COORD, LINK_X_COORD_SPEXIT);
            copy_word(self.ram, LINK_Y_COORD, LINK_Y_COORD_SPEXIT);
        }
    }

    pub(crate) struct SwimAccelerationView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SwimAccelerationView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn mode(&self, offset: usize) -> u16 {
            word(self.ram, SWIM_ACCELERATION_MODE + offset)
        }

        pub(crate) fn mode_low(&self, axis: usize) -> u8 {
            byte(self.ram, SWIM_ACCELERATION_MODE + axis * 2)
        }

        pub(crate) fn speed_active_flag(&self, offset: usize) -> u16 {
            word(self.ram, SWIM_SPEED_ACTIVE_FLAG + offset)
        }

        pub(crate) fn max_speed(&self, offset: usize) -> u16 {
            word(self.ram, SWIM_MAX_SPEED + offset)
        }

        pub(crate) fn acceleration_direction(&self, offset: usize) -> u16 {
            word(self.ram, SWIM_ACCELERATION_DIRECTION + offset)
        }

        pub(crate) fn acceleration(&self, offset: usize) -> u16 {
            word(self.ram, SWIM_ACCELERATION + offset)
        }

        pub(crate) fn has_any_acceleration(&self) -> bool {
            self.acceleration(0) | self.acceleration(2) != 0
        }
    }

    pub(crate) struct SwimAccelerationViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SwimAccelerationViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_mode(&mut self, offset: usize, value: u16) {
            write_le_u16(self.ram, SWIM_ACCELERATION_MODE + offset, value);
        }

        pub(crate) fn clear_mode_low_axis(&mut self) {
            write_le_u16(self.ram, SWIM_ACCELERATION_MODE, 0);
        }

        pub(crate) fn set_speed_active_flag(&mut self, offset: usize, value: u16) {
            write_le_u16(self.ram, SWIM_SPEED_ACTIVE_FLAG + offset, value);
        }

        pub(crate) fn set_max_speed(&mut self, offset: usize, value: u16) {
            write_le_u16(self.ram, SWIM_MAX_SPEED + offset, value);
        }

        pub(crate) fn set_max_speed_both_axes(&mut self, value: u16) {
            self.set_max_speed(0, value);
            self.set_max_speed(2, value);
        }

        pub(crate) fn set_acceleration_direction(&mut self, offset: usize, value: u16) {
            write_le_u16(self.ram, SWIM_ACCELERATION_DIRECTION + offset, value);
        }

        pub(crate) fn set_acceleration(&mut self, offset: usize, value: u16) {
            write_le_u16(self.ram, SWIM_ACCELERATION + offset, value);
        }

        pub(crate) fn clear_axis_motion(&mut self, offset: usize) {
            self.set_speed_active_flag(offset, 0);
            self.set_mode(offset, 0);
            self.set_acceleration(offset, 0);
            self.set_max_speed(offset, 0);
        }
    }

    pub(crate) struct Bg1MoveCalcView<'a> {
        ram: &'a [u8],
    }

    impl<'a> Bg1MoveCalcView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn x_subpixel(&self) -> u8 {
            byte(self.ram, BG1_MOVE_CALC_BUFFER + 1)
        }
    }

    pub(crate) struct Bg1MoveCalcViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> Bg1MoveCalcViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_x_subpixel(&mut self, value: u8) {
            self.ram[BG1_MOVE_CALC_BUFFER + 1] = value;
        }

        pub(crate) fn advance_x_subpixel(&mut self, delta: u16) -> u16 {
            let next = u16::from(self.ram[BG1_MOVE_CALC_BUFFER + 1]).wrapping_add(delta);
            self.set_x_subpixel(next as u8);
            next
        }
    }

    pub(crate) struct TileDetectPositionView<'a> {
        ram: &'a [u8],
    }

    impl<'a> TileDetectPositionView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn y_low_at(&self, offset: usize) -> u8 {
            byte(self.ram, TILEDETECT_WHICH_Y_POS + offset)
        }

        pub(crate) fn y(&self) -> u16 {
            word(self.ram, TILEDETECT_WHICH_Y_POS)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, TILEDETECT_WHICH_Y_POS)
        }

        pub(crate) fn x(&self) -> u16 {
            word(self.ram, TILEDETECT_WHICH_Y_POS + 2)
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, TILEDETECT_WHICH_Y_POS + 2)
        }

        pub(crate) fn location_calc_mask(&self) -> u16 {
            word(self.ram, TILEMAP_LOCATION_CALC_MASK)
        }

        pub(crate) fn interacting_tile(&self) -> u16 {
            word(self.ram, INDEX_OF_INTERACTING_TILE)
        }

        pub(crate) fn pit_tile(&self) -> u8 {
            byte(self.ram, TILEDETECT_PIT_TILE)
        }

        pub(crate) fn pit_tile_word(&self) -> u16 {
            word(self.ram, TILEDETECT_PIT_TILE)
        }

        pub(crate) fn deepwater(&self) -> u16 {
            word(self.ram, TILEDETECT_DEEPWATER)
        }

        pub(crate) fn deepwater_high(&self) -> u8 {
            byte(self.ram, TILEDETECT_DEEPWATER + 1)
        }

        pub(crate) fn normal_tiles(&self) -> u16 {
            word(self.ram, TILEDETECT_NORMAL_TILES)
        }

        pub(crate) fn normal_tiles_high(&self) -> u8 {
            byte(self.ram, TILEDETECT_NORMAL_TILES + 1)
        }

        pub(crate) fn misc_tiles(&self) -> u16 {
            word(self.ram, TILEDETECT_MISC_TILES)
        }

        pub(crate) fn thick_grass(&self) -> u16 {
            word(self.ram, TILEDETECT_THICK_GRASS)
        }

        pub(crate) fn thick_grass_low(&self) -> u8 {
            byte(self.ram, TILEDETECT_THICK_GRASS)
        }

        pub(crate) fn diagonal_tile(&self) -> u16 {
            word(self.ram, TILEDETECT_DIAGONAL_TILE)
        }

        pub(crate) fn stair_tile(&self) -> u8 {
            byte(self.ram, TILEDETECT_STAIR_TILE)
        }

        pub(crate) fn block_flags(&self) -> u16 {
            word(self.ram, TILEDETECT_BLOCK_FLAGS_LO)
        }

        pub(crate) fn door_direction_flags(&self) -> u16 {
            word(self.ram, TILEDETECT_DOOR_DIRECTION_FLAGS)
        }

        pub(crate) fn diag_state(&self) -> u16 {
            word(self.ram, TILEDETECT_DIAG_STATE)
        }

        pub(crate) fn moving_floor_tiles(&self) -> u16 {
            word(self.ram, TILEDETECT_MOVING_FLOOR_TILES)
        }

        pub(crate) fn icy_floor(&self) -> u16 {
            word(self.ram, TILEDETECT_ICY_FLOOR)
        }

        pub(crate) fn water_staircase(&self) -> u16 {
            word(self.ram, TILEDETECT_WATER_STAIRCASE)
        }

        pub(crate) fn shallow_water(&self) -> u16 {
            word(self.ram, TILEDETECT_SHALLOW_WATER)
        }

        pub(crate) fn shallow_water_low(&self) -> u8 {
            byte(self.ram, TILEDETECT_SHALLOW_WATER)
        }

        pub(crate) fn destruction_aftermath(&self) -> u16 {
            word(self.ram, TILEDETECT_DESTRUCTION_AFTERMATH)
        }

        pub(crate) fn destruction_aftermath_low(&self) -> u8 {
            byte(self.ram, TILEDETECT_DESTRUCTION_AFTERMATH)
        }

        pub(crate) fn read_something(&self) -> u16 {
            word(self.ram, TILEDETECT_READ_SOMETHING)
        }

        pub(crate) fn vertical_ledge(&self) -> u8 {
            byte(self.ram, TILEDETECT_VERTICAL_LEDGE)
        }

        pub(crate) fn horizontal_ledge(&self) -> u8 {
            byte(self.ram, DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ)
        }

        pub(crate) fn ledge_mask(&self) -> u8 {
            self.vertical_ledge() | self.horizontal_ledge()
        }

        pub(crate) fn ledges_down_leftright(&self) -> u8 {
            byte(self.ram, TILEDETECT_LEDGES_DOWN_LEFTRIGHT)
        }

        pub(crate) fn diagonal_ledge_tiles(&self) -> u8 {
            byte(self.ram, TILEDETECT_DIAGONAL_LEDGE_TILES)
        }

        pub(crate) fn chest(&self) -> u16 {
            word(self.ram, TILEDETECT_CHEST)
        }

        pub(crate) fn key_lock_gravestones(&self) -> u16 {
            word(self.ram, TILEDETECT_KEY_LOCK_GRAVESTONES)
        }

        pub(crate) fn key_lock_gravestones_low(&self) -> u8 {
            byte(self.ram, TILEDETECT_KEY_LOCK_GRAVESTONES)
        }

        pub(crate) fn spike_cactus_tiles(&self) -> u8 {
            byte(self.ram, BITFIELD_SPIKE_CACTUS_TILES)
        }

        pub(crate) fn tile_type(&self) -> u16 {
            word(self.ram, TILEDETECT_TILE_TYPE)
        }

        pub(crate) fn spike_floor_and_triggers(&self) -> u8 {
            byte(self.ram, TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS)
        }

        pub(crate) fn dashable_tiles(&self) -> u8 {
            byte(self.ram, BITMASK_FOR_DASHABLE_TILES)
        }

        pub(crate) fn staircase_cache(&self) -> u8 {
            byte(self.ram, TILEDETECT_STAIRCASE_CACHE)
        }

        pub(crate) fn slope_collision_bits(&self) -> u16 {
            word(self.ram, TILEDETECT_SLOPE_COLLISION_BITS)
        }

        pub(crate) fn collision_bits(&self) -> u16 {
            word(self.ram, TILEDETECT_COLLISION_BITS)
        }

        pub(crate) fn collision_bits_low(&self) -> u8 {
            byte(self.ram, TILEDETECT_COLLISION_BITS)
        }

        pub(crate) fn bonk_bits_low(&self) -> u8 {
            byte(self.ram, TILEDETECT_SLOPE_COLLISION_BITS)
                | byte(self.ram, TILEDETECT_COLLISION_BITS)
        }

        pub(crate) fn has_collision_bits(&self, mask: u16) -> bool {
            self.collision_bits() & mask != 0
        }

        pub(crate) fn has_slope_collision_bits(&self, mask: u16) -> bool {
            self.slope_collision_bits() & mask != 0
        }

        pub(crate) fn palette_bits_high(&self) -> u8 {
            byte(self.ram, LINK_PALETTE_BITS_OF_OAM + 1)
        }

        pub(crate) fn inroom_staircase(&self) -> u16 {
            word(self.ram, TILEDETECT_INROOM_STAIRCASE)
        }
    }

    pub(crate) struct TileDetectPositionViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> TileDetectPositionViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_y_high(&mut self, value: u8) {
            self.ram[TILEDETECT_WHICH_Y_POS + 1] = value;
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_WHICH_Y_POS, value);
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_WHICH_Y_POS + 2, value);
        }

        pub(crate) fn set_location_calc_mask(&mut self, value: u16) {
            write_le_u16(self.ram, TILEMAP_LOCATION_CALC_MASK, value);
        }

        pub(crate) fn set_interacting_tile(&mut self, value: u16) {
            write_le_u16(self.ram, INDEX_OF_INTERACTING_TILE, value);
        }

        pub(crate) fn set_diagonal_tile(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_DIAGONAL_TILE, value);
        }

        pub(crate) fn clear_diagonal_tile(&mut self) {
            self.set_diagonal_tile(0);
        }

        pub(crate) fn or_diagonal_tile(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_DIAGONAL_TILE) | value;
            write_le_u16(self.ram, TILEDETECT_DIAGONAL_TILE, next);
            next
        }

        pub(crate) fn set_stair_tile(&mut self, value: u8) {
            self.ram[TILEDETECT_STAIR_TILE] = value;
        }

        pub(crate) fn clear_stair_tile(&mut self) {
            self.set_stair_tile(0);
        }

        pub(crate) fn or_stair_tile(&mut self, value: u8) {
            self.ram[TILEDETECT_STAIR_TILE] |= value;
        }

        pub(crate) fn set_block_flags(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_BLOCK_FLAGS_LO, value);
        }

        pub(crate) fn clear_block_flags(&mut self) {
            self.set_block_flags(0);
        }

        pub(crate) fn or_block_flags(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_BLOCK_FLAGS_LO) | value;
            write_le_u16(self.ram, TILEDETECT_BLOCK_FLAGS_LO, next);
            next
        }

        pub(crate) fn set_door_direction_flags(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_DOOR_DIRECTION_FLAGS, value);
        }

        pub(crate) fn clear_door_direction_flags(&mut self) {
            self.set_door_direction_flags(0);
        }

        pub(crate) fn set_diag_state(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_DIAG_STATE, value);
        }

        pub(crate) fn clear_diag_state(&mut self) {
            self.set_diag_state(0);
        }

        pub(crate) fn clear_pit_tile(&mut self) {
            self.ram[TILEDETECT_PIT_TILE] = 0;
        }

        pub(crate) fn or_pit_tile(&mut self, value: u8) {
            self.ram[TILEDETECT_PIT_TILE] |= value;
        }

        pub(crate) fn set_deepwater(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_DEEPWATER, value);
        }

        pub(crate) fn clear_deepwater(&mut self) {
            self.set_deepwater(0);
        }

        pub(crate) fn or_deepwater(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_DEEPWATER) | value;
            write_le_u16(self.ram, TILEDETECT_DEEPWATER, next);
            next
        }

        pub(crate) fn set_normal_tiles(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_NORMAL_TILES, value);
        }

        pub(crate) fn clear_normal_tiles(&mut self) {
            self.set_normal_tiles(0);
        }

        pub(crate) fn or_normal_tiles(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_NORMAL_TILES) | value;
            write_le_u16(self.ram, TILEDETECT_NORMAL_TILES, next);
            next
        }

        pub(crate) fn set_misc_tiles(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_MISC_TILES, value);
        }

        pub(crate) fn clear_misc_tiles(&mut self) {
            self.set_misc_tiles(0);
        }

        pub(crate) fn or_misc_tiles(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_MISC_TILES) | value;
            write_le_u16(self.ram, TILEDETECT_MISC_TILES, next);
            next
        }

        pub(crate) fn set_thick_grass(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_THICK_GRASS, value);
        }

        pub(crate) fn clear_thick_grass(&mut self) {
            self.set_thick_grass(0);
        }

        pub(crate) fn or_thick_grass(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_THICK_GRASS) | value;
            write_le_u16(self.ram, TILEDETECT_THICK_GRASS, next);
            next
        }

        pub(crate) fn clear_vertical_ledge(&mut self) {
            self.ram[TILEDETECT_VERTICAL_LEDGE] = 0;
        }

        pub(crate) fn or_vertical_ledge(&mut self, value: u8) {
            self.ram[TILEDETECT_VERTICAL_LEDGE] |= value;
        }

        pub(crate) fn clear_horizontal_ledge(&mut self) {
            self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] = 0;
        }

        pub(crate) fn or_horizontal_ledge(&mut self, value: u8) {
            self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] |= value;
        }

        pub(crate) fn set_moving_floor_tiles(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_MOVING_FLOOR_TILES, value);
        }

        pub(crate) fn clear_moving_floor_tiles(&mut self) {
            self.set_moving_floor_tiles(0);
        }

        pub(crate) fn or_moving_floor_tiles(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_MOVING_FLOOR_TILES) | value;
            write_le_u16(self.ram, TILEDETECT_MOVING_FLOOR_TILES, next);
            next
        }

        pub(crate) fn set_icy_floor(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_ICY_FLOOR, value);
        }

        pub(crate) fn clear_icy_floor(&mut self) {
            self.set_icy_floor(0);
        }

        pub(crate) fn or_icy_floor(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_ICY_FLOOR) | value;
            write_le_u16(self.ram, TILEDETECT_ICY_FLOOR, next);
            next
        }

        pub(crate) fn set_water_staircase(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_WATER_STAIRCASE, value);
        }

        pub(crate) fn clear_water_staircase(&mut self) {
            self.set_water_staircase(0);
        }

        pub(crate) fn or_water_staircase(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_WATER_STAIRCASE) | value;
            write_le_u16(self.ram, TILEDETECT_WATER_STAIRCASE, next);
            next
        }

        pub(crate) fn set_shallow_water(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_SHALLOW_WATER, value);
        }

        pub(crate) fn clear_shallow_water(&mut self) {
            self.set_shallow_water(0);
        }

        pub(crate) fn or_shallow_water(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_SHALLOW_WATER) | value;
            write_le_u16(self.ram, TILEDETECT_SHALLOW_WATER, next);
            next
        }

        pub(crate) fn set_destruction_aftermath(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_DESTRUCTION_AFTERMATH, value);
        }

        pub(crate) fn clear_destruction_aftermath(&mut self) {
            self.set_destruction_aftermath(0);
        }

        pub(crate) fn or_destruction_aftermath(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_DESTRUCTION_AFTERMATH) | value;
            write_le_u16(self.ram, TILEDETECT_DESTRUCTION_AFTERMATH, next);
            next
        }

        pub(crate) fn set_read_something(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_READ_SOMETHING, value);
        }

        pub(crate) fn clear_read_something(&mut self) {
            self.set_read_something(0);
        }

        pub(crate) fn or_read_something(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_READ_SOMETHING) | value;
            write_le_u16(self.ram, TILEDETECT_READ_SOMETHING, next);
            next
        }

        pub(crate) fn set_ledges_down_leftright(&mut self, value: u8) {
            self.ram[TILEDETECT_LEDGES_DOWN_LEFTRIGHT] = value;
        }

        pub(crate) fn clear_ledges_down_leftright(&mut self) {
            self.set_ledges_down_leftright(0);
        }

        pub(crate) fn or_ledges_down_leftright(&mut self, value: u8) {
            self.ram[TILEDETECT_LEDGES_DOWN_LEFTRIGHT] |= value;
        }

        pub(crate) fn set_diagonal_ledge_tiles(&mut self, value: u8) {
            self.ram[TILEDETECT_DIAGONAL_LEDGE_TILES] = value;
        }

        pub(crate) fn clear_diagonal_ledge_tiles(&mut self) {
            self.set_diagonal_ledge_tiles(0);
        }

        pub(crate) fn or_diagonal_ledge_tiles(&mut self, value: u8) {
            self.ram[TILEDETECT_DIAGONAL_LEDGE_TILES] |= value;
        }

        pub(crate) fn set_chest(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_CHEST, value);
        }

        pub(crate) fn clear_chest(&mut self) {
            self.set_chest(0);
        }

        pub(crate) fn or_chest(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_CHEST) | value;
            write_le_u16(self.ram, TILEDETECT_CHEST, next);
            next
        }

        pub(crate) fn set_key_lock_gravestones(&mut self, value: u8) {
            self.ram[TILEDETECT_KEY_LOCK_GRAVESTONES] = value;
        }

        pub(crate) fn clear_key_lock_gravestones(&mut self) {
            self.set_key_lock_gravestones(0);
        }

        pub(crate) fn or_key_lock_gravestones(&mut self, value: u8) {
            self.ram[TILEDETECT_KEY_LOCK_GRAVESTONES] |= value;
        }

        pub(crate) fn set_spike_cactus_tiles(&mut self, value: u8) {
            self.ram[BITFIELD_SPIKE_CACTUS_TILES] = value;
        }

        pub(crate) fn clear_spike_cactus_tiles(&mut self) {
            self.set_spike_cactus_tiles(0);
        }

        pub(crate) fn or_spike_cactus_tiles(&mut self, value: u8) {
            self.ram[BITFIELD_SPIKE_CACTUS_TILES] |= value;
        }

        pub(crate) fn set_tile_type(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_TILE_TYPE, value);
        }

        pub(crate) fn clear_tile_type(&mut self) {
            self.set_tile_type(0);
        }

        pub(crate) fn set_spike_floor_and_triggers(&mut self, value: u8) {
            self.ram[TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS] = value;
        }

        pub(crate) fn clear_spike_floor_and_triggers(&mut self) {
            self.set_spike_floor_and_triggers(0);
        }

        pub(crate) fn or_spike_floor_and_triggers(&mut self, value: u8) {
            self.ram[TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS] |= value;
        }

        pub(crate) fn set_dashable_tiles(&mut self, value: u8) {
            self.ram[BITMASK_FOR_DASHABLE_TILES] = value;
        }

        pub(crate) fn clear_dashable_tiles(&mut self) {
            self.set_dashable_tiles(0);
        }

        pub(crate) fn or_dashable_tiles(&mut self, value: u8) {
            self.ram[BITMASK_FOR_DASHABLE_TILES] |= value;
        }

        pub(crate) fn set_staircase_cache(&mut self, value: u8) {
            self.ram[TILEDETECT_STAIRCASE_CACHE] = value;
        }

        pub(crate) fn set_slope_collision_bits(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_SLOPE_COLLISION_BITS, value);
        }

        pub(crate) fn clear_slope_collision_bits(&mut self) {
            self.set_slope_collision_bits(0);
        }

        pub(crate) fn or_slope_collision_bits(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_SLOPE_COLLISION_BITS) | value;
            write_le_u16(self.ram, TILEDETECT_SLOPE_COLLISION_BITS, next);
            next
        }

        pub(crate) fn set_collision_bits(&mut self, value: u16) {
            write_le_u16(self.ram, TILEDETECT_COLLISION_BITS, value);
        }

        pub(crate) fn clear_collision_bits(&mut self) {
            self.set_collision_bits(0);
        }

        pub(crate) fn or_collision_bits(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_COLLISION_BITS) | value;
            write_le_u16(self.ram, TILEDETECT_COLLISION_BITS, next);
            next
        }

        pub(crate) fn set_tile_probe_anchor(&mut self, value: u16) {
            write_le_u16(self.ram, SCRATCH_1, value);
        }

        pub(crate) fn clear_inroom_staircase(&mut self) {
            write_le_u16(self.ram, TILEDETECT_INROOM_STAIRCASE, 0);
        }

        pub(crate) fn or_inroom_staircase(&mut self, bits: u16) -> u16 {
            let next = read_le_u16(self.ram, TILEDETECT_INROOM_STAIRCASE) | bits;
            write_le_u16(self.ram, TILEDETECT_INROOM_STAIRCASE, next);
            next
        }

        pub(crate) fn set_liftable_tile_index(&mut self, value: u8) {
            self.ram[LIFTABLE_TILE_DETECTED_INDEX_DOUBLED] = value;
        }
    }

    pub(crate) struct PushedBlockView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PushedBlockView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn x(&self, slot: usize) -> u16 {
            u16::from(byte(self.ram, PUSHEDBLOCKS_X_LO + slot * 2))
                | (u16::from(byte(self.ram, PUSHEDBLOCKS_X_HI + slot * 2)) << 8)
        }

        pub(crate) fn y(&self, slot: usize) -> u16 {
            u16::from(byte(self.ram, PUSHEDBLOCKS_Y_LO + slot * 2))
                | (u16::from(byte(self.ram, PUSHEDBLOCKS_Y_HI + slot * 2)) << 8)
        }

        pub(crate) fn x_low(&self, slot: usize) -> u8 {
            byte(self.ram, PUSHEDBLOCKS_X_LO + slot * 2)
        }

        pub(crate) fn y_low(&self, slot: usize) -> u8 {
            byte(self.ram, PUSHEDBLOCKS_Y_LO + slot * 2)
        }

        pub(crate) fn subpixel(&self, slot: usize) -> u8 {
            byte(self.ram, PUSHEDBLOCKS_SUBPIXEL + slot * 2)
        }

        pub(crate) fn target_low(&self, slot: usize) -> u8 {
            byte(self.ram, PUSHEDBLOCKS_TARGET + slot * 2)
        }

        pub(crate) fn facing_player(&self, slot: usize) -> u8 {
            byte(self.ram, PUSHEDBLOCK_FACING_PLAYER + slot * 2)
        }

        pub(crate) fn x_fixed24(&self, slot: usize) -> u32 {
            u32::from(self.subpixel(slot))
                | (u32::from(self.x_low(slot)) << 8)
                | (u32::from(byte(self.ram, PUSHEDBLOCKS_X_HI + slot * 2)) << 16)
        }

        pub(crate) fn y_fixed24(&self, slot: usize) -> u32 {
            u32::from(self.subpixel(slot))
                | (u32::from(self.y_low(slot)) << 8)
                | (u32::from(byte(self.ram, PUSHEDBLOCKS_Y_HI + slot * 2)) << 16)
        }
    }

    pub(crate) struct PushedBlockViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PushedBlockViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_facing_player(&mut self, slot: usize, value: u8) {
            self.ram[PUSHEDBLOCK_FACING_PLAYER + slot * 2] = value;
        }

        pub(crate) fn set_target_low(&mut self, slot: usize, value: u8) {
            self.ram[PUSHEDBLOCKS_TARGET + slot * 2] = value;
        }

        pub(crate) fn set_x_fixed24(&mut self, slot: usize, value: u32) {
            self.ram[PUSHEDBLOCKS_SUBPIXEL + slot * 2] = value as u8;
            self.ram[PUSHEDBLOCKS_X_LO + slot * 2] = (value >> 8) as u8;
            self.ram[PUSHEDBLOCKS_X_HI + slot * 2] = (value >> 16) as u8;
        }

        pub(crate) fn set_y_fixed24(&mut self, slot: usize, value: u32) {
            self.ram[PUSHEDBLOCKS_SUBPIXEL + slot * 2] = value as u8;
            self.ram[PUSHEDBLOCKS_Y_LO + slot * 2] = (value >> 8) as u8;
            self.ram[PUSHEDBLOCKS_Y_HI + slot * 2] = (value >> 16) as u8;
        }
    }

    pub(crate) struct PlayerTileAttributeView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PlayerTileAttributeView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn attr_for_tile(&self, tile: usize) -> u8 {
            byte(self.ram, ATTRIBUTES_FOR_TILE_PLAYER + (tile & 0x03ff))
        }
    }

    pub(crate) struct InventoryStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> InventoryStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn inventory_item(&self, index: usize) -> u8 {
            byte(self.ram, LINK_ITEM_BOW + index)
        }

        pub(crate) fn has_inventory_item(&self, index: usize) -> bool {
            self.inventory_item(index) != 0
        }

        pub(crate) fn bow(&self) -> u8 {
            self.inventory_item(0)
        }

        pub(crate) fn has_silver_arrows(&self) -> bool {
            self.bow() & 4 != 0
        }

        pub(crate) fn has_upgraded_bow(&self) -> bool {
            self.bow() >= 3
        }

        pub(crate) fn boomerang(&self) -> u8 {
            self.inventory_item(1)
        }

        pub(crate) fn hookshot(&self) -> u8 {
            self.inventory_item(2)
        }

        pub(crate) fn mushroom(&self) -> u8 {
            self.inventory_item(4)
        }

        pub(crate) fn fire_rod(&self) -> u8 {
            self.inventory_item(5)
        }

        pub(crate) fn ice_rod(&self) -> u8 {
            self.inventory_item(6)
        }

        pub(crate) fn bombos(&self) -> u8 {
            self.inventory_item(7)
        }

        pub(crate) fn ether(&self) -> u8 {
            self.inventory_item(8)
        }

        pub(crate) fn quake(&self) -> u8 {
            self.inventory_item(9)
        }

        pub(crate) fn torch(&self) -> u8 {
            self.inventory_item(10)
        }

        pub(crate) fn hammer(&self) -> u8 {
            self.inventory_item(11)
        }

        pub(crate) fn flute(&self) -> u8 {
            self.inventory_item(12)
        }

        pub(crate) fn bug_net(&self) -> u8 {
            self.inventory_item(13)
        }

        pub(crate) fn book(&self) -> u8 {
            self.inventory_item(14)
        }

        pub(crate) fn cane_somaria(&self) -> u8 {
            self.inventory_item(15)
        }

        pub(crate) fn cane_byrna(&self) -> u8 {
            self.inventory_item(17)
        }

        pub(crate) fn cape(&self) -> u8 {
            self.inventory_item(18)
        }

        pub(crate) fn mirror(&self) -> u8 {
            self.inventory_item(19)
        }

        pub(crate) fn gloves(&self) -> u8 {
            self.inventory_item(20)
        }

        pub(crate) fn boots(&self) -> u8 {
            self.inventory_item(21)
        }

        pub(crate) fn has_boots(&self) -> bool {
            self.boots() != 0
        }

        pub(crate) fn flippers(&self) -> u8 {
            self.inventory_item(22)
        }

        pub(crate) fn moon_pearl(&self) -> u8 {
            self.inventory_item(23)
        }

        pub(crate) fn has_moon_pearl(&self) -> bool {
            self.moon_pearl() != 0
        }

        pub(crate) fn sword_type(&self) -> u8 {
            self.inventory_item(25)
        }

        pub(crate) fn shield_type(&self) -> u8 {
            self.inventory_item(26)
        }

        pub(crate) fn armor(&self) -> u8 {
            self.inventory_item(27)
        }

        pub(crate) fn bottle(&self, index: usize) -> u8 {
            byte(self.ram, LINK_BOTTLE_INFO + index)
        }

        pub(crate) fn has_bottle(&self, index: usize) -> bool {
            self.bottle(index) != 0
        }

        pub(crate) fn bottle_contents_or(&self) -> u8 {
            self.bottle(0) | self.bottle(1) | self.bottle(2) | self.bottle(3)
        }

        pub(crate) fn has_bottle_at_least(&self, value: u8) -> bool {
            (0..4).any(|index| self.bottle(index) >= value)
        }
    }

    pub(crate) struct InventoryStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> InventoryStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_inventory_item(&mut self, index: usize, value: u8) {
            self.ram[LINK_ITEM_BOW + index] = value;
        }

        pub(crate) fn set_mushroom(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOW + 4] = value;
        }

        pub(crate) fn set_ice_rod(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOW + 6] = value;
        }

        pub(crate) fn set_bombos(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOW + 7] = value;
        }

        pub(crate) fn set_ether(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOW + 8] = value;
        }

        pub(crate) fn set_flute(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOW + 12] = value;
        }

        pub(crate) fn set_mirror(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOW + 19] = value;
        }

        pub(crate) fn set_boots(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOW + 21] = value;
        }

        pub(crate) fn set_moon_pearl(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOW + 23] = value;
        }

        pub(crate) fn add_ability_flags(&mut self, flags: u8) {
            self.ram[LINK_ABILITY_FLAGS] |= flags;
        }

        pub(crate) fn set_sword_type(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOW + 25] = value;
        }

        pub(crate) fn set_shield_type(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOW + 26] = value;
        }

        pub(crate) fn set_bottle(&mut self, index: usize, value: u8) {
            self.ram[LINK_BOTTLE_INFO + index] = value;
        }

        pub(crate) fn fill_first_empty_bottle_with(&mut self, value: u8) -> bool {
            for i in 0..4 {
                if self.ram[LINK_BOTTLE_INFO + i] < 2 {
                    self.ram[LINK_BOTTLE_INFO + i] = value;
                    return true;
                }
            }
            false
        }

        pub(crate) fn replace_first_empty_bottle_with(&mut self, value: u8) -> bool {
            for i in 0..4 {
                if self.ram[LINK_BOTTLE_INFO + i] == 2 {
                    self.ram[LINK_BOTTLE_INFO + i] = value;
                    return true;
                }
            }
            false
        }
    }

    pub(crate) struct SaveProgressView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SaveProgressView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn palace_index_x2(&self) -> u8 {
            byte(self.ram, CUR_PALACE_INDEX_X2)
        }

        pub(crate) fn palace_index_x2_word(&self) -> u16 {
            word(self.ram, CUR_PALACE_INDEX_X2)
        }

        pub(crate) fn palace_index(&self) -> usize {
            usize::from(self.palace_index_x2() >> 1)
        }

        pub(crate) fn progress_indicator(&self) -> u8 {
            byte(self.ram, SRAM_PROGRESS_INDICATOR)
        }

        pub(crate) fn progress_indicator_word(&self) -> u16 {
            word(self.ram, SRAM_PROGRESS_INDICATOR)
        }

        pub(crate) fn progress_flags(&self) -> u8 {
            byte(self.ram, SRAM_PROGRESS_FLAGS)
        }

        pub(crate) fn progress_flags_has(&self, mask: u8) -> bool {
            self.progress_flags() & mask != 0
        }

        pub(crate) fn map_icons_indicator(&self) -> u8 {
            byte(self.ram, SAVEGAME_MAP_ICONS_INDICATOR)
        }

        pub(crate) fn dark_world_state(&self) -> u8 {
            byte(self.ram, SAVEGAME_IS_DARKWORLD)
        }

        pub(crate) fn is_dark_world(&self) -> bool {
            self.dark_world_state() != 0
        }

        pub(crate) fn dark_world_bit6(&self) -> u8 {
            (self.dark_world_state() >> 6) & 1
        }

        pub(crate) fn hud_current_item(&self) -> u8 {
            byte(self.ram, HUD_CUR_ITEM)
        }

        pub(crate) fn hud_current_item_slot(&self, slot: usize) -> u8 {
            let address = match slot {
                1 => HUD_CUR_ITEM_X,
                2 => HUD_CUR_ITEM_L,
                3 => HUD_CUR_ITEM_R,
                _ => HUD_CUR_ITEM,
            };
            byte(self.ram, address)
        }

        pub(crate) fn dungeon_info_word(&self, room: usize) -> u16 {
            word(self.ram, SAVE_DUNG_INFO + room * 2)
        }

        pub(crate) fn death_count_for_palace(&self, palace: usize) -> u16 {
            word(self.ram, DEATHS_PER_PALACE + palace * 2)
        }

        pub(crate) fn pending_death_save_counter(&self) -> u16 {
            word(self.ram, PENDING_DEATH_SAVE_COUNTER)
        }

        pub(crate) fn total_death_save_counter(&self) -> u16 {
            word(self.ram, TOTAL_DEATH_SAVE_COUNTER)
        }

        pub(crate) fn total_death_save_counter_is_uninitialized(&self) -> bool {
            self.total_death_save_counter() == 0xffff
        }

        pub(crate) fn dungeon_info_slice(&self) -> &[u8] {
            &self.ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + 0x500]
        }

        pub(crate) fn which_starting_point(&self) -> u8 {
            byte(self.ram, WHICH_STARTING_POINT)
        }

        pub(crate) fn progress_indicator_3(&self) -> u8 {
            byte(self.ram, SRAM_PROGRESS_INDICATOR_3)
        }
    }

    pub(crate) struct SaveProgressViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SaveProgressViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_palace_index_x2(&mut self, value: u8) {
            self.ram[CUR_PALACE_INDEX_X2] = value;
        }

        pub(crate) fn set_which_starting_point(&mut self, value: u8) {
            self.ram[WHICH_STARTING_POINT] = value;
        }

        pub(crate) fn xor_palace_index_x2(&mut self, value: u8) {
            self.ram[CUR_PALACE_INDEX_X2] ^= value;
        }

        pub(crate) fn set_progress_indicator(&mut self, value: u8) {
            self.ram[SRAM_PROGRESS_INDICATOR] = value;
        }

        pub(crate) fn or_progress_flags(&mut self, value: u8) {
            self.ram[SRAM_PROGRESS_FLAGS] |= value;
        }

        pub(crate) fn or_progress_indicator_3(&mut self, bits: u8) {
            self.ram[SRAM_PROGRESS_INDICATOR_3] |= bits;
        }

        pub(crate) fn clear_progress_indicator_3_bits(&mut self, bits: u8) {
            self.ram[SRAM_PROGRESS_INDICATOR_3] &= !bits;
        }

        pub(crate) fn xor_progress_flags(&mut self, value: u8) {
            self.ram[SRAM_PROGRESS_FLAGS] ^= value;
        }

        pub(crate) fn set_progress_flags(&mut self, value: u8) {
            self.ram[SRAM_PROGRESS_FLAGS] = value;
        }

        pub(crate) fn set_map_icons_indicator(&mut self, value: u8) {
            self.ram[SAVEGAME_MAP_ICONS_INDICATOR] = value;
        }

        pub(crate) fn set_dark_world_state(&mut self, value: u8) {
            self.ram[SAVEGAME_IS_DARKWORLD] = value;
        }

        pub(crate) fn xor_dark_world_state(&mut self, value: u8) {
            self.ram[SAVEGAME_IS_DARKWORLD] ^= value;
        }

        pub(crate) fn set_hud_current_item(&mut self, value: u8) {
            self.ram[HUD_CUR_ITEM] = value;
        }

        pub(crate) fn set_hud_current_item_slot(&mut self, slot: usize, value: u8) {
            let address = match slot {
                1 => HUD_CUR_ITEM_X,
                2 => HUD_CUR_ITEM_L,
                3 => HUD_CUR_ITEM_R,
                _ => HUD_CUR_ITEM,
            };
            self.ram[address] = value;
        }

        pub(crate) fn set_death_count_for_palace(&mut self, palace: usize, value: u16) {
            write_le_u16(self.ram, DEATHS_PER_PALACE + palace * 2, value);
        }

        pub(crate) fn increment_pending_death_save_counter(&mut self) -> u16 {
            let deaths = word(self.ram, PENDING_DEATH_SAVE_COUNTER).wrapping_add(1);
            write_le_u16(self.ram, PENDING_DEATH_SAVE_COUNTER, deaths);
            deaths
        }

        pub(crate) fn clear_pending_death_save_counter(&mut self) {
            write_le_u16(self.ram, PENDING_DEATH_SAVE_COUNTER, 0);
        }

        pub(crate) fn set_total_death_save_counter(&mut self, value: u16) {
            write_le_u16(self.ram, TOTAL_DEATH_SAVE_COUNTER, value);
        }

        pub(crate) fn clear_post_message_refresh_flag(&mut self) {
            self.ram[HUD_POST_MESSAGE_REFRESH_FLAG] = 0;
        }

        pub(crate) fn request_post_message_refresh(&mut self) {
            self.ram[HUD_POST_MESSAGE_REFRESH_FLAG] = 0x80;
        }

        pub(crate) fn clear_dungeon_info(&mut self) {
            self.ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + 0x500].fill(0);
        }

        pub(crate) fn copy_dungeon_info_from(&mut self, source: &[u8]) {
            self.ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + 0x500].copy_from_slice(source);
        }

        pub(crate) fn set_dungeon_info_word(&mut self, room: usize, value: u16) {
            write_le_u16(self.ram, SAVE_DUNG_INFO + room * 2, value);
        }

        pub(crate) fn or_dungeon_info_word(&mut self, room: usize, value: u16) -> u16 {
            let word = read_le_u16(self.ram, SAVE_DUNG_INFO + room * 2) | value;
            write_le_u16(self.ram, SAVE_DUNG_INFO + room * 2, word);
            word
        }
    }

    pub(crate) struct PlayerResourcesView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PlayerResourcesView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn magic_power(&self) -> u8 {
            byte(self.ram, LINK_MAGIC_POWER)
        }

        pub(crate) fn magic_filler(&self) -> u8 {
            byte(self.ram, LINK_MAGIC_FILLER)
        }

        pub(crate) fn magic_consumption_level(&self) -> u8 {
            byte(self.ram, LINK_MAGIC_CONSUMPTION)
        }

        pub(crate) fn bomb_filler(&self) -> u8 {
            byte(self.ram, LINK_BOMB_FILLER)
        }

        pub(crate) fn bombs(&self) -> u8 {
            byte(self.ram, LINK_ITEM_BOMBS)
        }

        pub(crate) fn bomb_upgrade_level(&self) -> u8 {
            byte(self.ram, LINK_BOMB_UPGRADES)
        }

        pub(crate) fn next_bomb_upgrade_level(&self) -> u8 {
            self.bomb_upgrade_level().wrapping_add(1)
        }

        pub(crate) fn arrow_filler(&self) -> u8 {
            byte(self.ram, LINK_ARROW_REFILL_COUNTER)
        }

        pub(crate) fn arrows(&self) -> u8 {
            byte(self.ram, LINK_NUM_ARROWS)
        }

        pub(crate) fn arrow_upgrade_level(&self) -> u8 {
            byte(self.ram, LINK_ARROW_UPGRADES)
        }

        pub(crate) fn next_arrow_upgrade_level(&self) -> u8 {
            self.arrow_upgrade_level().wrapping_add(1)
        }

        pub(crate) fn has_bomb_or_arrow_upgrade(&self) -> bool {
            byte(self.ram, LINK_BOMB_UPGRADES) | byte(self.ram, LINK_ARROW_UPGRADES) != 0
        }

        pub(crate) fn current_health(&self) -> u8 {
            byte(self.ram, LINK_CURRENT_HEALTH)
        }

        pub(crate) fn health_capacity(&self) -> u8 {
            byte(self.ram, LINK_HEALTH_CAPACITY)
        }

        pub(crate) fn heart_filler(&self) -> u8 {
            byte(self.ram, LINK_HEARTS_FILLER)
        }

        pub(crate) fn low_health_beep_timer(&self) -> u8 {
            byte(self.ram, LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP)
        }

        pub(crate) fn equipped_bottle_index(&self) -> u8 {
            byte(self.ram, LINK_ITEM_BOTTLE_INDEX)
        }

        pub(crate) fn rupees_goal(&self) -> u16 {
            read_le_u16(self.ram, LINK_RUPEES_GOAL)
        }

        pub(crate) fn rupees_actual(&self) -> u16 {
            read_le_u16(self.ram, LINK_RUPEES_ACTUAL)
        }

        pub(crate) fn compass_flags(&self) -> u16 {
            read_le_u16(self.ram, LINK_COMPASS)
        }

        pub(crate) fn big_key_flags(&self) -> u16 {
            read_le_u16(self.ram, LINK_BIGKEY)
        }

        pub(crate) fn dungeon_map_flags(&self) -> u16 {
            read_le_u16(self.ram, LINK_DUNGEON_MAP)
        }

        pub(crate) fn has_compass_mask(&self, mask: u16) -> bool {
            self.compass_flags() & mask != 0
        }

        pub(crate) fn has_big_key_mask(&self, mask: u16) -> bool {
            self.big_key_flags() & mask != 0
        }

        pub(crate) fn lacks_big_key_mask(&self, mask: u16) -> bool {
            !self.has_big_key_mask(mask)
        }

        pub(crate) fn has_dungeon_map_mask(&self, mask: u16) -> bool {
            self.dungeon_map_flags() & mask != 0
        }

        pub(crate) fn has_big_key_at_shift(&self, shift: u8) -> bool {
            (self.big_key_flags() << shift) & 0x8000 != 0
        }

        pub(crate) fn has_dungeon_map_at_shift(&self, shift: u8) -> bool {
            (self.dungeon_map_flags() << shift) & 0x8000 != 0
        }

        pub(crate) fn has_compass_at_shift(&self, shift: u8) -> bool {
            (self.compass_flags() << shift) & 0x8000 != 0
        }

        pub(crate) fn ability_flags(&self) -> u8 {
            byte(self.ram, LINK_ABILITY_FLAGS)
        }

        pub(crate) fn pendant_flags(&self) -> u8 {
            byte(self.ram, LINK_WHICH_PENDANTS)
        }

        pub(crate) fn crystal_flags(&self) -> u8 {
            byte(self.ram, LINK_HAS_CRYSTALS)
        }

        pub(crate) fn heart_pieces(&self) -> u8 {
            byte(self.ram, LINK_HEART_PIECES)
        }

        pub(crate) fn keys(&self) -> u8 {
            byte(self.ram, LINK_NUM_KEYS)
        }

        pub(crate) fn rupees_in_pond(&self) -> u8 {
            byte(self.ram, LINK_RUPEES_IN_POND)
        }
    }

    pub(crate) struct PlayerResourcesViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PlayerResourcesViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_magic_power(&mut self, value: u8) {
            self.ram[LINK_MAGIC_POWER] = value;
        }

        pub(crate) fn set_magic_consumption_level(&mut self, value: u8) {
            self.ram[LINK_MAGIC_CONSUMPTION] = value;
        }

        pub(crate) fn increment_magic_power(&mut self) {
            self.ram[LINK_MAGIC_POWER] = self.ram[LINK_MAGIC_POWER].wrapping_add(1);
        }

        pub(crate) fn set_magic_filler(&mut self, value: u8) {
            self.ram[LINK_MAGIC_FILLER] = value;
        }

        pub(crate) fn clear_magic_filler(&mut self) {
            self.ram[LINK_MAGIC_FILLER] = 0;
        }

        pub(crate) fn decrement_magic_filler(&mut self) {
            self.ram[LINK_MAGIC_FILLER] = self.ram[LINK_MAGIC_FILLER].wrapping_sub(1);
        }

        pub(crate) fn decrement_bomb_filler(&mut self) {
            self.ram[LINK_BOMB_FILLER] = self.ram[LINK_BOMB_FILLER].wrapping_sub(1);
        }

        pub(crate) fn set_bomb_filler(&mut self, value: u8) {
            self.ram[LINK_BOMB_FILLER] = value;
        }

        pub(crate) fn increment_bomb_filler_by(&mut self, value: u8) {
            self.ram[LINK_BOMB_FILLER] = self.ram[LINK_BOMB_FILLER].wrapping_add(value);
        }

        pub(crate) fn increment_bombs(&mut self) {
            self.ram[LINK_ITEM_BOMBS] = self.ram[LINK_ITEM_BOMBS].wrapping_add(1);
        }

        pub(crate) fn decrement_bombs(&mut self) -> u8 {
            self.ram[LINK_ITEM_BOMBS] = self.ram[LINK_ITEM_BOMBS].wrapping_sub(1);
            self.ram[LINK_ITEM_BOMBS]
        }

        pub(crate) fn increment_health_capacity_by(&mut self, value: u8) -> u8 {
            self.ram[LINK_HEALTH_CAPACITY] = self.ram[LINK_HEALTH_CAPACITY].wrapping_add(value);
            self.ram[LINK_HEALTH_CAPACITY]
        }

        pub(crate) fn increment_heart_filler_by(&mut self, value: u8) {
            self.ram[LINK_HEARTS_FILLER] = self.ram[LINK_HEARTS_FILLER].wrapping_add(value);
        }

        pub(crate) fn increment_heart_filler_word_by(&mut self, value: u16) -> u16 {
            let hearts = read_le_u16(self.ram, LINK_HEARTS_FILLER).wrapping_add(value);
            write_le_u16(self.ram, LINK_HEARTS_FILLER, hearts);
            hearts
        }

        pub(crate) fn increment_magic_filler_by(&mut self, value: u8) {
            self.ram[LINK_MAGIC_FILLER] = self.ram[LINK_MAGIC_FILLER].wrapping_add(value);
        }

        pub(crate) fn add_crystal_flags(&mut self, flags: u8) {
            self.ram[LINK_HAS_CRYSTALS] |= flags;
        }

        pub(crate) fn set_crystal_flags(&mut self, flags: u8) {
            self.ram[LINK_HAS_CRYSTALS] = flags;
        }

        pub(crate) fn set_pendant_flags(&mut self, flags: u8) {
            self.ram[LINK_WHICH_PENDANTS] = flags;
        }

        pub(crate) fn decrement_arrow_filler(&mut self) {
            self.ram[LINK_ARROW_REFILL_COUNTER] =
                self.ram[LINK_ARROW_REFILL_COUNTER].wrapping_sub(1);
        }

        pub(crate) fn set_arrow_filler(&mut self, value: u8) {
            self.ram[LINK_ARROW_REFILL_COUNTER] = value;
        }

        pub(crate) fn increment_arrow_filler_by(&mut self, value: u8) {
            self.ram[LINK_ARROW_REFILL_COUNTER] =
                self.ram[LINK_ARROW_REFILL_COUNTER].wrapping_add(value);
        }

        pub(crate) fn increment_arrows(&mut self) {
            self.ram[LINK_NUM_ARROWS] = self.ram[LINK_NUM_ARROWS].wrapping_add(1);
        }

        pub(crate) fn increment_arrows_by(&mut self, value: u8) {
            self.ram[LINK_NUM_ARROWS] = self.ram[LINK_NUM_ARROWS].wrapping_add(value);
        }

        pub(crate) fn set_arrows(&mut self, value: u8) {
            self.ram[LINK_NUM_ARROWS] = value;
        }

        pub(crate) fn decrement_arrows(&mut self) -> u8 {
            self.ram[LINK_NUM_ARROWS] = self.ram[LINK_NUM_ARROWS].wrapping_sub(1);
            self.ram[LINK_NUM_ARROWS]
        }

        pub(crate) fn set_current_health(&mut self, value: u8) {
            self.ram[LINK_CURRENT_HEALTH] = value;
        }

        pub(crate) fn increment_current_health_by(&mut self, value: u8) {
            self.ram[LINK_CURRENT_HEALTH] = self.ram[LINK_CURRENT_HEALTH].wrapping_add(value);
        }

        pub(crate) fn decrement_current_health_by(&mut self, value: u8) -> u8 {
            self.ram[LINK_CURRENT_HEALTH] = self.ram[LINK_CURRENT_HEALTH].wrapping_sub(value);
            self.ram[LINK_CURRENT_HEALTH]
        }

        pub(crate) fn set_heart_filler(&mut self, value: u8) {
            self.ram[LINK_HEARTS_FILLER] = value;
        }

        pub(crate) fn decrement_heart_filler_by(&mut self, value: u8) {
            self.ram[LINK_HEARTS_FILLER] = self.ram[LINK_HEARTS_FILLER].wrapping_sub(value);
        }

        pub(crate) fn decrement_low_health_beep_timer(&mut self) {
            self.ram[LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP] =
                self.ram[LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP].wrapping_sub(1);
        }

        pub(crate) fn set_low_health_beep_timer(&mut self, value: u8) {
            self.ram[LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP] = value;
        }

        pub(crate) fn set_equipped_bottle_index(&mut self, value: u8) {
            self.ram[LINK_ITEM_BOTTLE_INDEX] = value;
        }

        pub(crate) fn set_rupees_goal(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_RUPEES_GOAL, value);
        }

        pub(crate) fn set_rupees_actual(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_RUPEES_ACTUAL, value);
        }

        pub(crate) fn add_rupees_goal(&mut self, value: u16) -> u16 {
            let rupees = read_le_u16(self.ram, LINK_RUPEES_GOAL).wrapping_add(value);
            write_le_u16(self.ram, LINK_RUPEES_GOAL, rupees);
            rupees
        }

        pub(crate) fn subtract_rupees_goal(&mut self, value: u16) -> u16 {
            let rupees = read_le_u16(self.ram, LINK_RUPEES_GOAL).wrapping_sub(value);
            write_le_u16(self.ram, LINK_RUPEES_GOAL, rupees);
            rupees
        }

        pub(crate) fn set_keys(&mut self, value: u8) {
            self.ram[LINK_NUM_KEYS] = value;
        }

        pub(crate) fn increment_keys(&mut self) -> u8 {
            self.ram[LINK_NUM_KEYS] = self.ram[LINK_NUM_KEYS].wrapping_add(1);
            self.ram[LINK_NUM_KEYS]
        }

        pub(crate) fn decrement_keys(&mut self) -> u8 {
            self.ram[LINK_NUM_KEYS] = self.ram[LINK_NUM_KEYS].wrapping_sub(1);
            self.ram[LINK_NUM_KEYS]
        }

        pub(crate) fn advance_heart_piece_count(&mut self) -> u8 {
            self.ram[LINK_HEART_PIECES] = self.ram[LINK_HEART_PIECES].wrapping_add(1) & 3;
            self.ram[LINK_HEART_PIECES]
        }

        pub(crate) fn add_rupees_to_pond(&mut self, value: u8) -> u8 {
            self.ram[LINK_RUPEES_IN_POND] = self.ram[LINK_RUPEES_IN_POND].wrapping_add(value);
            self.ram[LINK_RUPEES_IN_POND]
        }

        pub(crate) fn subtract_pond_reward_threshold(&mut self) -> u8 {
            self.ram[LINK_RUPEES_IN_POND] = self.ram[LINK_RUPEES_IN_POND].wrapping_sub(100);
            self.ram[LINK_RUPEES_IN_POND]
        }

        pub(crate) fn set_bomb_upgrade_level(&mut self, value: u8) {
            self.ram[LINK_BOMB_UPGRADES] = value;
        }

        pub(crate) fn set_arrow_upgrade_level(&mut self, value: u8) {
            self.ram[LINK_ARROW_UPGRADES] = value;
        }
    }

    pub(crate) struct MirrorWarpScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> MirrorWarpScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn target_index(&self) -> usize {
            usize::from(word(self.ram, MIRROR_WARP_TARGET_INDEX) >> 1)
        }

        pub(crate) fn target_offset(&self) -> u16 {
            word(
                self.ram,
                MIRROR_WARP_TARGET_OFFSETS + self.target_index() * 2,
            )
        }

        pub(crate) fn velocity_delta(&self) -> u16 {
            word(
                self.ram,
                MIRROR_WARP_VELOCITY_DELTAS + self.target_index() * 2,
            )
        }

        pub(crate) fn wave_offset(&self) -> u16 {
            word(self.ram, MIRROR_WARP_WAVE_OFFSET)
        }

        pub(crate) fn displacement(&self) -> u16 {
            word(self.ram, MIRROR_WARP_DISPLACEMENT)
        }

        pub(crate) fn subpixel(&self) -> u16 {
            word(self.ram, MIRROR_WARP_SUBPIXEL)
        }

        pub(crate) fn animation_counter(&self) -> u8 {
            byte(self.ram, MIRROR_WARP_ANIMATION_COUNTER)
        }
    }

    pub(crate) struct MirrorWarpScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> MirrorWarpScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn initialize_hdma_wave_state(&mut self) {
            for addr in [
                MIRROR_WARP_TARGET_INDEX,
                MIRROR_WARP_WAVE_OFFSET,
                MIRROR_WARP_DISPLACEMENT,
                MIRROR_WARP_SUBPIXEL,
                MIRROR_WARP_RESERVED,
            ] {
                write_le_u16(self.ram, addr, 0);
            }
            write_le_u16(self.ram, MIRROR_WARP_SPACING_A, 8);
            write_le_u16(self.ram, MIRROR_WARP_SPACING_B, 8);
            write_le_u16(self.ram, MIRROR_WARP_WAVE_LENGTH, 21);
            write_le_u16(self.ram, MIRROR_WARP_TARGET_OFFSETS, 0xfe00);
            write_le_u16(self.ram, MIRROR_WARP_TARGET_OFFSETS + 2, 0x0200);
            write_le_u16(self.ram, MIRROR_WARP_VELOCITY_DELTAS, 0xffc0);
            write_le_u16(self.ram, MIRROR_WARP_VELOCITY_DELTAS + 2, 0x0040);
        }

        pub(crate) fn reset_wave_and_subpixel(&mut self) {
            write_le_u16(self.ram, MIRROR_WARP_WAVE_OFFSET, 0);
            write_le_u16(self.ram, MIRROR_WARP_SUBPIXEL, 0);
        }

        pub(crate) fn toggle_target_index(&mut self) {
            let value = word(self.ram, MIRROR_WARP_TARGET_INDEX) ^ 2;
            write_le_u16(self.ram, MIRROR_WARP_TARGET_INDEX, value);
        }

        pub(crate) fn set_displacement(&mut self, value: u16) {
            write_le_u16(self.ram, MIRROR_WARP_DISPLACEMENT, value);
        }

        pub(crate) fn set_subpixel_low_from(&mut self, value: u16) {
            write_le_u16(self.ram, MIRROR_WARP_SUBPIXEL, value & 0x00ff);
        }

        pub(crate) fn set_wave_offset(&mut self, value: u16) {
            write_le_u16(self.ram, MIRROR_WARP_WAVE_OFFSET, value);
        }

        pub(crate) fn shrink_target_offsets_for_dewaving(&mut self) {
            write_le_u16(self.ram, MIRROR_WARP_TARGET_OFFSETS, 0xff00);
            write_le_u16(self.ram, MIRROR_WARP_TARGET_OFFSETS + 2, 0x0100);
        }

        pub(crate) fn increment_load_step_counter(&mut self) -> u8 {
            self.ram[MIRROR_WARP_LOAD_STEP_COUNTER] =
                self.ram[MIRROR_WARP_LOAD_STEP_COUNTER].wrapping_add(1);
            self.ram[MIRROR_WARP_LOAD_STEP_COUNTER]
        }

        pub(crate) fn reset_load_step_counter(&mut self) {
            self.ram[MIRROR_WARP_LOAD_STEP_COUNTER] = 0;
        }

        pub(crate) fn set_animation_counter(&mut self, value: u8) {
            self.ram[MIRROR_WARP_ANIMATION_COUNTER] = value;
        }

        pub(crate) fn decrement_animation_counter(&mut self) -> u8 {
            self.ram[MIRROR_WARP_ANIMATION_COUNTER] =
                self.ram[MIRROR_WARP_ANIMATION_COUNTER].wrapping_sub(1);
            self.ram[MIRROR_WARP_ANIMATION_COUNTER]
        }
    }

    pub(crate) struct WorldStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> WorldStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn dungeon_room(&self) -> u16 {
            word(self.ram, DUNGEON_ROOM)
        }

        pub(crate) fn dungeon_room_index(&self) -> u8 {
            byte(self.ram, DUNGEON_ROOM)
        }

        pub(crate) fn overworld_screen(&self) -> u8 {
            byte(self.ram, OVERWORLD_SCREEN_INDEX)
        }

        pub(crate) fn overworld_screen_word(&self) -> u16 {
            word(self.ram, OVERWORLD_SCREEN_INDEX)
        }

        pub(crate) fn is_indoors(&self) -> bool {
            byte(self.ram, PLAYER_IS_INDOORS) != 0
        }

        pub(crate) fn indoor_flag(&self) -> u8 {
            byte(self.ram, PLAYER_IS_INDOORS)
        }

        pub(crate) fn is_outdoors(&self) -> bool {
            !self.is_indoors()
        }

        pub(crate) fn overworld_map_state(&self) -> u8 {
            byte(self.ram, OVERWORLD_MAP_STATE)
        }

        pub(crate) fn entrance_sequence_counter(&self) -> u8 {
            byte(self.ram, OVERWORLD_ENTRANCE_SEQUENCE_COUNTER)
        }

        pub(crate) fn overworld_area(&self) -> u16 {
            word(self.ram, OVERWORLD_AREA_INDEX)
        }

        pub(crate) fn overworld_area_low(&self) -> u8 {
            byte(self.ram, OVERWORLD_AREA_INDEX)
        }

        pub(crate) fn transition_direction(&self) -> u8 {
            byte(self.ram, OVERWORLD_TRANSITION_DIR)
        }

        pub(crate) fn screen_transition_direction_bits(&self) -> u8 {
            byte(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2)
        }

        pub(crate) fn screen_transition_direction_bits_word(&self) -> u16 {
            word(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2)
        }

        pub(crate) fn has_screen_transition_direction_bits(&self) -> bool {
            self.screen_transition_direction_bits() != 0
        }

        pub(crate) fn screen_transition(&self) -> u8 {
            byte(self.ram, OVERWORLD_SCREEN_TRANSITION)
        }

        pub(crate) fn screen_transition_word(&self) -> u16 {
            word(self.ram, OVERWORLD_SCREEN_TRANSITION)
        }

        pub(crate) fn overlay_index(&self) -> u8 {
            byte(self.ram, OVERLAY_INDEX)
        }

        pub(crate) fn map16_load_src(&self) -> u16 {
            word(self.ram, MAP16_LOAD_SRC_OFF)
        }

        pub(crate) fn map16_load_dst(&self) -> u16 {
            word(self.ram, MAP16_LOAD_DST_OFF)
        }

        pub(crate) fn map16_load_y_unit(&self) -> u16 {
            word(self.ram, MAP16_LOAD_Y_UNIT)
        }

        pub(crate) fn bg1_x(&self) -> u16 {
            word(self.ram, BG1_X_SCROLL)
        }

        pub(crate) fn bg1_x_low(&self) -> u8 {
            byte(self.ram, BG1_X_SCROLL)
        }

        pub(crate) fn bg1_y(&self) -> u16 {
            word(self.ram, BG1_Y_SCROLL)
        }

        pub(crate) fn bg1_y_low(&self) -> u8 {
            byte(self.ram, BG1_Y_SCROLL)
        }

        pub(crate) fn bg2_x(&self) -> u16 {
            word(self.ram, BG2_X_SCROLL)
        }

        pub(crate) fn bg2_x_low(&self) -> u8 {
            byte(self.ram, BG2_X_SCROLL)
        }

        pub(crate) fn bg2_y(&self) -> u16 {
            word(self.ram, BG2_Y_SCROLL)
        }

        pub(crate) fn bg2_y_low(&self) -> u8 {
            byte(self.ram, BG2_Y_SCROLL)
        }

        pub(crate) fn bg1_x_offset(&self) -> u16 {
            word(self.ram, BG1_X_OFFSET)
        }

        pub(crate) fn bg1_y_offset(&self) -> u16 {
            word(self.ram, BG1_Y_OFFSET)
        }

        pub(crate) fn bg1_offset_mask(&self) -> u16 {
            self.bg1_x_offset() | self.bg1_y_offset()
        }

        pub(crate) fn camera_x(&self) -> u16 {
            word(self.ram, CAMERA_X)
        }

        pub(crate) fn camera_y(&self) -> u16 {
            word(self.ram, CAMERA_Y)
        }

        pub(crate) fn rng_seed(&self) -> u8 {
            byte(self.ram, RNG_SEED)
        }

        pub(crate) fn overworld_offset_base_x(&self) -> u16 {
            word(self.ram, OVERWORLD_OFFSET_BASE_X)
        }

        pub(crate) fn overworld_offset_base_y(&self) -> u16 {
            word(self.ram, OVERWORLD_OFFSET_BASE_Y)
        }

        pub(crate) fn overworld_offset_mask_x(&self) -> u16 {
            word(self.ram, OVERWORLD_OFFSET_MASK_X)
        }

        pub(crate) fn overworld_offset_mask_y(&self) -> u16 {
            word(self.ram, OVERWORLD_OFFSET_MASK_Y)
        }

        pub(crate) fn dark_world_region_index(&self) -> u8 {
            byte(self.ram, IS_IN_DARK_WORLD_FLAG)
        }

        pub(crate) fn is_in_dark_world(&self) -> bool {
            byte(self.ram, IS_IN_DARK_WORLD_FLAG) != 0
        }

        pub(crate) fn flag_overworld_area_changed(&self) -> bool {
            byte(self.ram, FLAG_OVERWORLD_AREA_CHANGED) != 0
        }

        pub(crate) fn overworld_area_index(&self) -> u16 {
            word(self.ram, OVERWORLD_AREA_INDEX)
        }
    }

    pub(crate) struct WorldStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> WorldStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_overlay_high(&mut self, value: u8) {
            self.ram[OVERLAY_INDEX + 1] = value;
        }

        pub(crate) fn set_dungeon_room(&mut self, value: u16) {
            write_le_u16(self.ram, DUNGEON_ROOM, value);
        }

        pub(crate) fn set_dungeon_room_index(&mut self, value: u8) {
            self.ram[DUNGEON_ROOM] = value;
        }

        pub(crate) fn increment_dungeon_room_index_by(&mut self, value: u8) -> u8 {
            self.ram[DUNGEON_ROOM] = self.ram[DUNGEON_ROOM].wrapping_add(value);
            self.ram[DUNGEON_ROOM]
        }

        pub(crate) fn decrement_dungeon_room_index_by(&mut self, value: u8) -> u8 {
            self.ram[DUNGEON_ROOM] = self.ram[DUNGEON_ROOM].wrapping_sub(value);
            self.ram[DUNGEON_ROOM]
        }

        pub(crate) fn set_overworld_screen(&mut self, value: u8) {
            self.ram[OVERWORLD_SCREEN_INDEX] = value;
        }

        pub(crate) fn set_overworld_screen_word(&mut self, value: u16) {
            write_le_u16(self.ram, OVERWORLD_SCREEN_INDEX, value);
        }

        pub(crate) fn set_indoor_flag(&mut self, value: u8) {
            self.ram[PLAYER_IS_INDOORS] = value;
        }

        pub(crate) fn set_overworld_map_state(&mut self, value: u8) {
            self.ram[OVERWORLD_MAP_STATE] = value;
        }

        pub(crate) fn increment_overworld_map_state(&mut self) {
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        }

        pub(crate) fn set_entrance_sequence_counter(&mut self, value: u8) {
            self.ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = value;
        }

        pub(crate) fn clear_entrance_sequence_counter(&mut self) {
            self.ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = 0;
        }

        pub(crate) fn increment_entrance_sequence_counter(&mut self) -> u8 {
            self.ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] =
                self.ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER].wrapping_add(1);
            self.ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER]
        }

        pub(crate) fn decrement_entrance_sequence_counter(&mut self) -> u8 {
            self.ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] =
                self.ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER].wrapping_sub(1);
            self.ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER]
        }

        pub(crate) fn set_screen_transition_direction_bits(&mut self, value: u8) {
            self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS2] = value;
        }

        pub(crate) fn set_screen_transition_direction_bits_word(&mut self, value: u16) {
            write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, value);
        }

        pub(crate) fn clear_screen_transition_direction_bits(&mut self) {
            self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS2] = 0;
        }

        pub(crate) fn clear_screen_transition_direction_bits_word(&mut self) {
            write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, 0);
        }

        pub(crate) fn and_screen_transition_direction_bits(&mut self, value: u8) {
            self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS2] &= value;
        }

        pub(crate) fn or_screen_transition_direction_bits(&mut self, value: u8) {
            self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS2] |= value;
        }

        pub(crate) fn or_screen_transition_direction_bits_word(&mut self, value: u16) -> u16 {
            let bits = read_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2) | value;
            write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, bits);
            bits
        }

        pub(crate) fn set_screen_transition(&mut self, value: u8) {
            self.ram[OVERWORLD_SCREEN_TRANSITION] = value;
        }

        pub(crate) fn set_screen_transition_word(&mut self, value: u16) {
            write_le_u16(self.ram, OVERWORLD_SCREEN_TRANSITION, value);
        }

        pub(crate) fn clear_screen_transition(&mut self) {
            self.ram[OVERWORLD_SCREEN_TRANSITION] = 0;
        }

        pub(crate) fn set_bg1_x(&mut self, value: u16) {
            write_le_u16(self.ram, BG1_X_SCROLL, value);
        }

        pub(crate) fn set_bg1_x_low(&mut self, value: u8) {
            self.ram[BG1_X_SCROLL] = value;
        }

        pub(crate) fn set_bg1_y(&mut self, value: u16) {
            write_le_u16(self.ram, BG1_Y_SCROLL, value);
        }

        pub(crate) fn set_bg1_y_low(&mut self, value: u8) {
            self.ram[BG1_Y_SCROLL] = value;
        }

        pub(crate) fn set_bg2_x(&mut self, value: u16) {
            write_le_u16(self.ram, BG2_X_SCROLL, value);
        }

        pub(crate) fn set_bg2_y(&mut self, value: u16) {
            write_le_u16(self.ram, BG2_Y_SCROLL, value);
        }

        pub(crate) fn set_bg1_x_offset(&mut self, value: u16) {
            write_le_u16(self.ram, BG1_X_OFFSET, value);
        }

        pub(crate) fn set_bg1_y_offset(&mut self, value: u16) {
            write_le_u16(self.ram, BG1_Y_OFFSET, value);
        }

        pub(crate) fn set_bg1_offsets(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, BG1_X_OFFSET, x);
            write_le_u16(self.ram, BG1_Y_OFFSET, y);
        }

        pub(crate) fn clear_bg1_offsets(&mut self) {
            self.set_bg1_offsets(0, 0);
        }

        pub(crate) fn add_bg2_x(&mut self, value: u16) {
            let x = read_le_u16(self.ram, BG2_X_SCROLL);
            write_le_u16(self.ram, BG2_X_SCROLL, x.wrapping_add(value));
        }

        pub(crate) fn set_room_transitioning_flags(&mut self, value: u8) {
            self.ram[ROOM_TRANSITIONING_FLAGS] = value;
        }

        pub(crate) fn set_rng_seed(&mut self, value: u8) {
            self.ram[RNG_SEED] = value;
        }

        pub(crate) fn set_trigger_special_entrance(&mut self, value: u8) {
            self.ram[TRIGGER_SPECIAL_ENTRANCE] = value;
        }

        pub(crate) fn set_overworld_screen_trans_dir_bits(&mut self, value: u8) {
            self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS] = value;
        }

        pub(crate) fn clear_tile_interaction_shared_flag(&mut self) {
            self.ram[TILE_INTERACTION_SHARED_FLAG] = 0;
        }

        pub(crate) fn set_dark_world_region_index(&mut self, value: u8) {
            self.ram[IS_IN_DARK_WORLD_FLAG] = value;
        }

        pub(crate) fn set_which_entrance(&mut self, value: u16) {
            write_le_u16(self.ram, WHICH_ENTRANCE, value);
        }

        pub(crate) fn set_birdtravel_status(&mut self, value: u8) {
            self.ram[BIRDTRAVEL_STATUS] = value;
        }

        pub(crate) fn set_flag_travel_bird(&mut self, value: u8) {
            self.ram[FLAG_TRAVEL_BIRD] = value;
        }

        pub(crate) fn clear_flag_overworld_area_changed(&mut self) {
            self.ram[FLAG_OVERWORLD_AREA_CHANGED] = 0;
        }

        pub(crate) fn set_last_light_vs_dark_world(&mut self, value: u8) {
            self.ram[LAST_LIGHT_VS_DARK_WORLD] = value;
        }
    }

    pub(crate) struct DungeonStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DungeonStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn header_tag(&self, index: usize) -> u8 {
            byte(self.ram, DUNGEON_HEADER_TAG + index)
        }

        pub(crate) fn primary_header_tag(&self) -> u8 {
            self.header_tag(0)
        }

        pub(crate) fn bg2_attr(&self, offset: usize) -> u8 {
            byte(self.ram, DUNGEON_BG2_ATTR_TABLE + offset)
        }

        pub(crate) fn bg2_attr_word(&self, offset: usize) -> u16 {
            word(self.ram, DUNGEON_BG2_ATTR_TABLE + offset)
        }

        pub(crate) fn bg1_attr(&self, offset: usize) -> u8 {
            byte(self.ram, DUNGEON_BG1_ATTR_TABLE + offset)
        }

        pub(crate) fn bg1_attr_word(&self, offset: usize) -> u16 {
            word(self.ram, DUNGEON_BG1_ATTR_TABLE + offset)
        }

        pub(crate) fn attr_for_tile(&self, tile: usize) -> u8 {
            byte(self.ram, ATTRIBUTES_FOR_TILE_PLAYER + (tile & 0x03ff))
        }

        pub(crate) fn header_collision_2_mirror_high(&self) -> u8 {
            byte(self.ram, DUNGEON_HEADER_COLLISION_2_MIRROR + 1)
        }

        pub(crate) fn header_collision(&self) -> u8 {
            byte(self.ram, DUNG_HDR_COLLISION)
        }

        pub(crate) fn header_collision_2(&self) -> u8 {
            byte(self.ram, DUNG_HDR_COLLISION_2)
        }

        pub(crate) fn header_collision_2_mirror(&self) -> u8 {
            byte(self.ram, DUNGEON_HEADER_COLLISION_2_MIRROR)
        }

        pub(crate) fn bg2_properties(&self) -> u8 {
            byte(self.ram, DUNG_HDR_BG2_PROPERTIES)
        }

        pub(crate) fn current_floor(&self) -> u8 {
            byte(self.ram, DUNG_CUR_FLOOR)
        }

        pub(crate) fn current_floor_word(&self) -> u16 {
            word(self.ram, DUNG_CUR_FLOOR)
        }

        pub(crate) fn cached_floor(&self) -> u8 {
            byte(self.ram, DUNG_CUR_FLOOR_CACHED)
        }

        pub(crate) fn floor_y_velocity(&self) -> u16 {
            word(self.ram, DUNGEON_FLOOR_Y_VELOCITY)
        }

        pub(crate) fn floor_y_velocity_low(&self) -> u8 {
            byte(self.ram, DUNGEON_FLOOR_Y_VELOCITY)
        }

        pub(crate) fn floor_x_velocity(&self) -> u16 {
            word(self.ram, DUNGEON_FLOOR_X_VELOCITY)
        }

        pub(crate) fn floor_x_velocity_low(&self) -> u8 {
            byte(self.ram, DUNGEON_FLOOR_X_VELOCITY)
        }

        pub(crate) fn lit_torches(&self) -> u8 {
            byte(self.ram, DUNG_NUM_LIT_TORCHES)
        }

        pub(crate) fn orange_blue_barrier_state(&self) -> u8 {
            byte(self.ram, ORANGE_BLUE_BARRIER_STATE)
        }

        pub(crate) fn wants_lights_out(&self) -> u8 {
            byte(self.ram, DUNG_WANT_LIGHTS_OUT)
        }

        pub(crate) fn wants_lights_out_copy(&self) -> u8 {
            byte(self.ram, DUNG_WANT_LIGHTS_OUT_COPY)
        }

        pub(crate) fn any_lights_out_request(&self) -> u8 {
            self.wants_lights_out() | self.wants_lights_out_copy()
        }

        pub(crate) fn quadrant_upload_index(&self) -> u8 {
            byte(self.ram, DUNG_CUR_QUADRANT_UPLOAD)
        }

        pub(crate) fn trapdoors_down(&self) -> u16 {
            word(self.ram, DUNG_FLAG_TRAPDOORS_DOWN)
        }

        pub(crate) fn trapdoors_down_low(&self) -> u8 {
            byte(self.ram, DUNG_FLAG_TRAPDOORS_DOWN)
        }

        pub(crate) fn water_puzzle_state_changed(&self) -> u8 {
            byte(self.ram, DUNG_FLAG_STATECHANGE_WATERPUZZLE)
        }

        pub(crate) fn draw_width_indicator(&self) -> u8 {
            byte(self.ram, DUNG_DRAW_WIDTH_INDICATOR)
        }

        pub(crate) fn draw_width_indicator_word(&self) -> u16 {
            word(self.ram, DUNG_DRAW_WIDTH_INDICATOR)
        }

        pub(crate) fn draw_height_indicator(&self) -> u8 {
            byte(self.ram, DUNG_DRAW_HEIGHT_INDICATOR)
        }

        pub(crate) fn draw_height_indicator_word(&self) -> u16 {
            word(self.ram, DUNG_DRAW_HEIGHT_INDICATOR)
        }

        pub(crate) fn opened_doors_including_adjacent(&self) -> u16 {
            word(self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT)
        }

        pub(crate) fn opened_doors(&self) -> u16 {
            word(self.ram, DUNG_DOOR_OPENED)
        }

        pub(crate) fn floor_x_offset(&self) -> u16 {
            word(self.ram, DUNG_FLOOR_X_OFFS)
        }

        pub(crate) fn floor_y_offset(&self) -> u16 {
            word(self.ram, DUNG_FLOOR_Y_OFFS)
        }

        pub(crate) fn object_pos_in_objdata(&self, index: usize) -> u16 {
            word(self.ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2)
        }

        pub(crate) fn has_opened_door_mask(&self, mask: u16) -> bool {
            self.opened_doors_including_adjacent() & mask != 0
        }

        pub(crate) fn door_animation_step(&self) -> u16 {
            word(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON)
        }

        pub(crate) fn door_animation_step_low(&self) -> u8 {
            byte(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON)
        }

        pub(crate) fn staircase_index(&self) -> u8 {
            byte(self.ram, WHICH_STAIRCASE_INDEX)
        }

        pub(crate) fn staircase_index_slot(&self) -> usize {
            usize::from(self.staircase_index() & 3)
        }

        pub(crate) fn staircase_index_has_vertical_bit(&self) -> bool {
            self.staircase_index() & 4 != 0
        }

        pub(crate) fn staircase_move_counter(&self) -> u8 {
            byte(self.ram, STAIRCASE_MOVE_COUNTER)
        }

        pub(crate) fn inter_staircase_pos(&self, index: usize) -> u16 {
            word(self.ram, DUNG_INTER_STAIRCASES + index * 2)
        }

        pub(crate) fn bg2_attr_address(&self, offset: usize) -> usize {
            DUNGEON_BG2_ATTR_TABLE + offset
        }

        pub(crate) fn bg2_attr_pair(&self, offset: usize) -> Option<(u8, u8)> {
            let address = self.bg2_attr_address(offset);
            Some((*self.ram.get(address)?, *self.ram.get(address + 1)?))
        }

        pub(crate) fn bg2_attr_slice(&self, start: usize, len: usize) -> &[u8] {
            &self.ram[DUNGEON_BG2_ATTR_TABLE + start..DUNGEON_BG2_ATTR_TABLE + start + len]
        }

        pub(crate) fn door_type_and_slot(&self, door: usize) -> u8 {
            byte(self.ram, DOOR_TYPE_AND_SLOT + door * 2)
        }

        pub(crate) fn door_type_word(&self, door: usize) -> u16 {
            word(self.ram, DOOR_TYPE_AND_SLOT + door * 2)
        }

        pub(crate) fn door_direction(&self, door: usize) -> u8 {
            byte(self.ram, DUNGEON_DOOR_DIRECTION + door * 2)
        }

        pub(crate) fn door_direction_word(&self, door: usize) -> u16 {
            word(self.ram, DUNGEON_DOOR_DIRECTION + door * 2)
        }

        pub(crate) fn changeable_object_index(&self, index: usize) -> u8 {
            byte(self.ram, CHANGEABLE_DUNGEON_OBJECT_INDEX + index)
        }

        pub(crate) fn replacement_tile_state(&self, index: usize) -> u16 {
            word(self.ram, DUNGEON_REPLACEMENT_TILE_STATE + index * 2)
        }

        pub(crate) fn savegame_state_bits(&self) -> u16 {
            word(self.ram, DUNG_SAVEGAME_STATE_BITS)
        }

        pub(crate) fn has_savegame_state_bits(&self, mask: u16) -> bool {
            self.savegame_state_bits() & mask != 0
        }

        pub(crate) fn room_index2(&self) -> u8 {
            byte(self.ram, DUNGEON_ROOM_INDEX2)
        }

        pub(crate) fn room_index2_word(&self) -> u16 {
            word(self.ram, DUNGEON_ROOM_INDEX2)
        }

        pub(crate) fn bg2_tile(&self, index: usize) -> u16 {
            word(self.ram, DUNG_BG2 + index * 2)
        }

        pub(crate) fn bg1_tile(&self, index: usize) -> u16 {
            word(self.ram, DUNG_BG1 + index * 2)
        }

        pub(crate) fn bg1_tile_by_byte_pos(&self, pos: u16) -> u16 {
            self.bg1_tile((pos >> 1) as usize)
        }

        pub(crate) fn bg2_tile_by_byte_pos(&self, pos: u16) -> u16 {
            self.bg2_tile((pos >> 1) as usize)
        }

        pub(crate) fn misc_object_index(&self) -> u16 {
            word(self.ram, DUNG_MISC_OBJS_INDEX)
        }

        pub(crate) fn misc_object_slot(&self) -> usize {
            (self.misc_object_index() >> 1) as usize
        }

        pub(crate) fn load_ptr_offset(&self) -> u16 {
            word(self.ram, DUNG_LOAD_PTR_OFFS)
        }

        pub(crate) fn object_tilemap_pos(&self, index: usize) -> u16 {
            word(self.ram, DUNG_OBJECT_TILEMAP_POS + index * 2)
        }

        pub(crate) fn door_tilemap_address(&self, door: usize) -> u16 {
            word(self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2)
        }

        pub(crate) fn current_door_index(&self) -> u16 {
            word(self.ram, DUNG_CUR_DOOR_IDX)
        }

        pub(crate) fn current_door_slot(&self) -> usize {
            (self.current_door_index() >> 1) as usize
        }

        pub(crate) fn current_door_pos(&self) -> u16 {
            word(self.ram, DUNG_CUR_DOOR_POS_DUNGEON)
        }

        pub(crate) fn door_open_counter(&self) -> u16 {
            word(self.ram, DOOR_OPEN_CLOSED_COUNTER)
        }

        pub(crate) fn door_open_counter_low(&self) -> u8 {
            byte(self.ram, DOOR_OPEN_CLOSED_COUNTER)
        }

        pub(crate) fn trap_trigger_latch(&self) -> u8 {
            byte(self.ram, DUNGEON_TRAP_TRIGGER_LATCH)
        }

        pub(crate) fn room_history_entry(&self, index: usize) -> u16 {
            word(self.ram, DUNGEON_ROOM_HISTORY + index * 2)
        }

        pub(crate) fn floor_move_flags(&self) -> u8 {
            byte(self.ram, DUNG_FLOOR_MOVE_FLAGS)
        }

        pub(crate) fn has_bomb_trap_activation(&self) -> bool {
            byte(self.ram, ACTIVATE_BOMB_TRAP_OVERLORD) != 0
        }

        pub(crate) fn kind_of_in_room_staircase(&self) -> u8 {
            byte(self.ram, KIND_OF_IN_ROOM_STAIRCASE)
        }

        pub(crate) fn chest_location(&self, index: usize) -> u16 {
            word(self.ram, DUNG_CHEST_LOCATIONS + index * 2)
        }

        pub(crate) fn moving_floor_check_flags(&self) -> u16 {
            word(self.ram, MOVING_FLOOR_BG_CHECK_FLAGS)
        }
    }

    pub(crate) struct DungeonStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DungeonStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_header_tag(&mut self, index: usize, value: u8) {
            self.ram[DUNGEON_HEADER_TAG + index] = value;
        }

        pub(crate) fn clear_header_tag(&mut self, index: usize) {
            self.set_header_tag(index, 0);
        }

        pub(crate) fn clear_header_tags(&mut self, count: usize) {
            self.ram[DUNGEON_HEADER_TAG..DUNGEON_HEADER_TAG + count].fill(0);
        }

        pub(crate) fn set_bg2_attr(&mut self, offset: usize, value: u8) {
            self.ram[DUNGEON_BG2_ATTR_TABLE + offset] = value;
        }

        pub(crate) fn set_bg2_attr_word(&mut self, offset: usize, value: u16) {
            write_le_u16(self.ram, DUNGEON_BG2_ATTR_TABLE + offset, value);
        }

        pub(crate) fn set_bg1_attr_word(&mut self, offset: usize, value: u16) {
            write_le_u16(self.ram, DUNGEON_BG1_ATTR_TABLE + offset, value);
        }

        pub(crate) fn xor_bg2_attr(&mut self, offset: usize, value: u8) {
            self.ram[DUNGEON_BG2_ATTR_TABLE + offset] ^= value;
        }

        pub(crate) fn xor_bg1_attr(&mut self, offset: usize, value: u8) {
            self.ram[DUNGEON_BG1_ATTR_TABLE + offset] ^= value;
        }

        pub(crate) fn set_floor_y_velocity_high(&mut self, value: u8) {
            self.ram[DUNGEON_FLOOR_Y_VELOCITY + 1] = value;
        }

        pub(crate) fn set_floor_y_velocity(&mut self, value: u16) {
            write_le_u16(self.ram, DUNGEON_FLOOR_Y_VELOCITY, value);
        }

        pub(crate) fn set_floor_x_velocity(&mut self, value: u16) {
            write_le_u16(self.ram, DUNGEON_FLOOR_X_VELOCITY, value);
        }

        pub(crate) fn clear_floor_velocity(&mut self) {
            write_le_u16(self.ram, DUNGEON_FLOOR_X_VELOCITY, 0);
            write_le_u16(self.ram, DUNGEON_FLOOR_Y_VELOCITY, 0);
        }

        pub(crate) fn set_header_collision_2_mirror_high(&mut self, value: u8) {
            self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR + 1] = value;
        }

        pub(crate) fn set_header_collision(&mut self, value: u8) {
            self.ram[DUNG_HDR_COLLISION] = value;
        }

        pub(crate) fn set_header_collision_2(&mut self, value: u8) {
            self.ram[DUNG_HDR_COLLISION_2] = value;
        }

        pub(crate) fn clear_header_collision_2(&mut self) {
            self.ram[DUNG_HDR_COLLISION_2] = 0;
        }

        pub(crate) fn set_header_collision_2_mirror(&mut self, value: u8) {
            self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR] = value;
        }

        pub(crate) fn increment_header_collision_2_mirror(&mut self) -> u8 {
            self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR] =
                self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR].wrapping_add(1);
            self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR]
        }

        pub(crate) fn copy_header_collision_2_to_mirror(&mut self) {
            self.ram[DUNGEON_HEADER_COLLISION_2_MIRROR] = self.ram[DUNG_HDR_COLLISION_2];
        }

        pub(crate) fn set_bg2_properties(&mut self, value: u8) {
            self.ram[DUNG_HDR_BG2_PROPERTIES] = value;
        }

        pub(crate) fn clear_bg2_properties(&mut self) {
            self.ram[DUNG_HDR_BG2_PROPERTIES] = 0;
        }

        pub(crate) fn set_current_floor(&mut self, value: u8) {
            self.ram[DUNG_CUR_FLOOR] = value;
        }

        pub(crate) fn decrement_current_floor(&mut self) -> u8 {
            self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_sub(1);
            self.ram[DUNG_CUR_FLOOR]
        }

        pub(crate) fn increment_current_floor(&mut self) -> u8 {
            self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_add(1);
            self.ram[DUNG_CUR_FLOOR]
        }

        pub(crate) fn cache_current_floor(&mut self) {
            self.ram[DUNG_CUR_FLOOR_CACHED] = self.ram[DUNG_CUR_FLOOR];
        }

        pub(crate) fn restore_cached_floor(&mut self) {
            self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR_CACHED];
        }

        pub(crate) fn clear_lit_torches(&mut self) {
            self.ram[DUNG_NUM_LIT_TORCHES] = 0;
        }

        pub(crate) fn set_lit_torches(&mut self, value: u8) {
            self.ram[DUNG_NUM_LIT_TORCHES] = value;
        }

        pub(crate) fn increment_lit_torches(&mut self) -> u8 {
            self.ram[DUNG_NUM_LIT_TORCHES] = self.ram[DUNG_NUM_LIT_TORCHES].wrapping_add(1);
            self.ram[DUNG_NUM_LIT_TORCHES]
        }

        pub(crate) fn decrement_lit_torches(&mut self) -> u8 {
            self.ram[DUNG_NUM_LIT_TORCHES] = self.ram[DUNG_NUM_LIT_TORCHES].wrapping_sub(1);
            self.ram[DUNG_NUM_LIT_TORCHES]
        }

        pub(crate) fn set_lights_out_request(&mut self, value: u8) {
            self.ram[DUNG_WANT_LIGHTS_OUT] = value;
        }

        pub(crate) fn clear_lights_out_request(&mut self) {
            self.ram[DUNG_WANT_LIGHTS_OUT] = 0;
        }

        pub(crate) fn set_lights_out_request_copy(&mut self, value: u8) {
            self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = value;
        }

        pub(crate) fn copy_lights_out_request(&mut self) {
            self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = self.ram[DUNG_WANT_LIGHTS_OUT];
        }

        pub(crate) fn clear_lights_out_requests(&mut self) {
            self.ram[DUNG_WANT_LIGHTS_OUT] = 0;
            self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = 0;
        }

        pub(crate) fn clear_quadrant_upload_index(&mut self) {
            self.ram[DUNG_CUR_QUADRANT_UPLOAD] = 0;
        }

        pub(crate) fn advance_quadrant_upload_index_by(&mut self, value: u8) -> u8 {
            self.ram[DUNG_CUR_QUADRANT_UPLOAD] =
                self.ram[DUNG_CUR_QUADRANT_UPLOAD].wrapping_add(value);
            self.ram[DUNG_CUR_QUADRANT_UPLOAD]
        }

        pub(crate) fn set_trapdoors_down(&mut self, value: u16) {
            write_le_u16(self.ram, DUNG_FLAG_TRAPDOORS_DOWN, value);
        }

        pub(crate) fn clear_trapdoors_down(&mut self) {
            write_le_u16(self.ram, DUNG_FLAG_TRAPDOORS_DOWN, 0);
        }

        pub(crate) fn set_trapdoors_down_low(&mut self, value: u8) {
            self.ram[DUNG_FLAG_TRAPDOORS_DOWN] = value;
        }

        pub(crate) fn increment_trapdoors_down_low(&mut self) -> u8 {
            self.ram[DUNG_FLAG_TRAPDOORS_DOWN] = self.ram[DUNG_FLAG_TRAPDOORS_DOWN].wrapping_add(1);
            self.ram[DUNG_FLAG_TRAPDOORS_DOWN]
        }

        pub(crate) fn clear_water_puzzle_state_changed(&mut self) {
            self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = 0;
        }

        pub(crate) fn set_water_puzzle_state_changed(&mut self, value: u8) {
            self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = value;
        }

        pub(crate) fn increment_water_puzzle_state_changed(&mut self) -> u8 {
            self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] =
                self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE].wrapping_add(1);
            self.ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE]
        }

        pub(crate) fn set_draw_width_indicator(&mut self, value: u8) {
            self.ram[DUNG_DRAW_WIDTH_INDICATOR] = value;
        }

        pub(crate) fn set_draw_width_indicator_word(&mut self, value: u16) {
            write_le_u16(self.ram, DUNG_DRAW_WIDTH_INDICATOR, value);
        }

        pub(crate) fn set_draw_height_indicator(&mut self, value: u8) {
            self.ram[DUNG_DRAW_HEIGHT_INDICATOR] = value;
        }

        pub(crate) fn set_draw_height_indicator_word(&mut self, value: u16) {
            write_le_u16(self.ram, DUNG_DRAW_HEIGHT_INDICATOR, value);
        }

        pub(crate) fn clear_draw_dimensions(&mut self) {
            write_le_u16(self.ram, DUNG_DRAW_WIDTH_INDICATOR, 0);
            write_le_u16(self.ram, DUNG_DRAW_HEIGHT_INDICATOR, 0);
        }

        pub(crate) fn set_draw_dimensions(&mut self, width: u8, height: u8) {
            self.ram[DUNG_DRAW_WIDTH_INDICATOR] = width;
            self.ram[DUNG_DRAW_HEIGHT_INDICATOR] = height;
        }

        pub(crate) fn set_draw_dimensions_words(&mut self, width: u16, height: u16) {
            write_le_u16(self.ram, DUNG_DRAW_WIDTH_INDICATOR, width);
            write_le_u16(self.ram, DUNG_DRAW_HEIGHT_INDICATOR, height);
        }

        pub(crate) fn set_opened_doors_including_adjacent(&mut self, value: u16) {
            write_le_u16(self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, value);
        }

        pub(crate) fn set_floor_x_offset(&mut self, value: u16) {
            write_le_u16(self.ram, DUNG_FLOOR_X_OFFS, value);
        }

        pub(crate) fn mark_opened_door_mask(&mut self, mask: u16) -> u16 {
            let opened = read_le_u16(self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT) | mask;
            write_le_u16(self.ram, DUNG_DOOR_OPENED_INCL_ADJACENT, opened);
            opened
        }

        pub(crate) fn clear_door_animation_step(&mut self) {
            write_le_u16(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, 0);
        }

        pub(crate) fn set_door_animation_step(&mut self, value: u16) {
            write_le_u16(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, value);
        }

        pub(crate) fn set_door_animation_step_low(&mut self, value: u8) {
            self.ram[DOOR_ANIMATION_STEP_INDICATOR_DUNGEON] = value;
        }

        pub(crate) fn increment_door_animation_step(&mut self) -> u16 {
            let step = read_le_u16(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON).wrapping_add(1);
            write_le_u16(self.ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, step);
            step
        }

        pub(crate) fn set_staircase_index(&mut self, value: u8) {
            self.ram[WHICH_STAIRCASE_INDEX] = value;
        }

        pub(crate) fn set_staircase_move_counter(&mut self, value: u8) {
            self.ram[STAIRCASE_MOVE_COUNTER] = value;
        }

        pub(crate) fn decrement_staircase_move_counter(&mut self) -> u8 {
            self.ram[STAIRCASE_MOVE_COUNTER] = self.ram[STAIRCASE_MOVE_COUNTER].wrapping_sub(1);
            self.ram[STAIRCASE_MOVE_COUNTER]
        }

        pub(crate) fn set_inter_staircase_pos(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, DUNG_INTER_STAIRCASES + index * 2, value);
        }

        pub(crate) fn clear_replacement_tile_states(&mut self) {
            self.ram[DUNGEON_REPLACEMENT_TILE_STATE..DUNGEON_REPLACEMENT_TILE_STATE + 32].fill(0);
        }

        pub(crate) fn set_replacement_tile_state(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, DUNGEON_REPLACEMENT_TILE_STATE + index * 2, value);
        }

        pub(crate) fn increment_replacement_tile_state(&mut self, index: usize) -> u16 {
            let state =
                read_le_u16(self.ram, DUNGEON_REPLACEMENT_TILE_STATE + index * 2).wrapping_add(1);
            write_le_u16(self.ram, DUNGEON_REPLACEMENT_TILE_STATE + index * 2, state);
            state
        }

        pub(crate) fn set_savegame_state_high_bits(&mut self, mask: u8) {
            self.ram[DUNG_SAVEGAME_STATE_BITS + 1] |= mask;
        }

        pub(crate) fn clear_savegame_state_bits(&mut self) {
            write_le_u16(self.ram, DUNG_SAVEGAME_STATE_BITS, 0);
        }

        pub(crate) fn set_savegame_state_bits(&mut self, value: u16) {
            write_le_u16(self.ram, DUNG_SAVEGAME_STATE_BITS, value);
        }

        pub(crate) fn or_savegame_state_bits(&mut self, mask: u16) -> u16 {
            let value = read_le_u16(self.ram, DUNG_SAVEGAME_STATE_BITS) | mask;
            write_le_u16(self.ram, DUNG_SAVEGAME_STATE_BITS, value);
            value
        }

        pub(crate) fn clear_savegame_state_high(&mut self) {
            self.ram[DUNG_SAVEGAME_STATE_BITS + 1] = 0;
        }

        pub(crate) fn clear_savegame_state_low(&mut self) {
            self.ram[DUNG_SAVEGAME_STATE_BITS] = 0;
        }

        pub(crate) fn set_room_index2(&mut self, value: u8) {
            self.ram[DUNGEON_ROOM_INDEX2] = value;
        }

        pub(crate) fn set_room_index2_word(&mut self, value: u16) {
            write_le_u16(self.ram, DUNGEON_ROOM_INDEX2, value);
        }

        pub(crate) fn set_bg2_tile(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, DUNG_BG2 + index * 2, value);
        }

        pub(crate) fn set_bg1_tile(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, DUNG_BG1 + index * 2, value);
        }

        pub(crate) fn set_bg1_tile_by_byte_pos(&mut self, pos: u16, value: u16) {
            self.set_bg1_tile((pos >> 1) as usize, value);
        }

        pub(crate) fn set_bg2_tile_by_byte_pos(&mut self, pos: u16, value: u16) {
            self.set_bg2_tile((pos >> 1) as usize, value);
        }

        pub(crate) fn clear_door_tilemap_addresses(&mut self) {
            self.ram[DUNG_DOOR_TILEMAP_ADDRESS..DUNG_DOOR_TILEMAP_ADDRESS + 32].fill(0);
        }

        pub(crate) fn set_door_tilemap_address(&mut self, door: usize, value: u16) {
            write_le_u16(self.ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2, value);
        }

        pub(crate) fn set_object_tilemap_pos(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, DUNG_OBJECT_TILEMAP_POS + index * 2, value);
        }

        pub(crate) fn set_misc_object_index(&mut self, value: u16) {
            write_le_u16(self.ram, DUNG_MISC_OBJS_INDEX, value);
        }

        pub(crate) fn clear_misc_object_index(&mut self) {
            self.ram[DUNG_MISC_OBJS_INDEX] = 0;
        }

        pub(crate) fn advance_misc_object_index_by(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, DUNG_MISC_OBJS_INDEX).wrapping_add(value);
            write_le_u16(self.ram, DUNG_MISC_OBJS_INDEX, next);
            next
        }

        pub(crate) fn set_load_ptr_offset(&mut self, value: u16) {
            write_le_u16(self.ram, DUNG_LOAD_PTR_OFFS, value);
        }

        pub(crate) fn advance_load_ptr_offset_by(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, DUNG_LOAD_PTR_OFFS).wrapping_add(value);
            write_le_u16(self.ram, DUNG_LOAD_PTR_OFFS, next);
            next
        }

        pub(crate) fn set_current_door_index(&mut self, value: u16) {
            write_le_u16(self.ram, DUNG_CUR_DOOR_IDX, value);
        }

        pub(crate) fn set_current_door_index_for_slot(&mut self, door: usize) {
            write_le_u16(self.ram, DUNG_CUR_DOOR_IDX, (door * 2) as u16);
        }

        pub(crate) fn advance_current_door_index_by(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, DUNG_CUR_DOOR_IDX).wrapping_add(value);
            write_le_u16(self.ram, DUNG_CUR_DOOR_IDX, next);
            next
        }

        pub(crate) fn set_current_door_pos(&mut self, value: u16) {
            write_le_u16(self.ram, DUNG_CUR_DOOR_POS_DUNGEON, value);
        }

        pub(crate) fn clear_current_door_pos(&mut self) {
            write_le_u16(self.ram, DUNG_CUR_DOOR_POS_DUNGEON, 0);
        }

        pub(crate) fn set_door_open_counter(&mut self, value: u16) {
            write_le_u16(self.ram, DOOR_OPEN_CLOSED_COUNTER, value);
        }

        pub(crate) fn set_door_open_counter_low(&mut self, value: u8) {
            self.ram[DOOR_OPEN_CLOSED_COUNTER] = value;
        }

        pub(crate) fn clear_door_open_counter_low(&mut self) {
            self.ram[DOOR_OPEN_CLOSED_COUNTER] = 0;
        }

        pub(crate) fn increment_door_open_counter_low(&mut self) -> u8 {
            self.ram[DOOR_OPEN_CLOSED_COUNTER] = self.ram[DOOR_OPEN_CLOSED_COUNTER].wrapping_add(1);
            self.ram[DOOR_OPEN_CLOSED_COUNTER]
        }

        pub(crate) fn clear_replacement_tile_state_low(&mut self, index: usize) {
            self.ram[DUNGEON_REPLACEMENT_TILE_STATE + index * 2] = 0;
        }

        pub(crate) fn copy_custom_tile_attrs(&mut self, attrs: &[u8]) {
            self.ram[ATTRIBUTES_FOR_TILE_PLAYER + 0x140..ATTRIBUTES_FOR_TILE_PLAYER + 0x1c0]
                .copy_from_slice(attrs);
        }

        pub(crate) fn copy_default_tile_attrs_tail(&mut self, attrs: &[u8]) {
            self.ram[ATTRIBUTES_FOR_TILE_PLAYER + 0x1c0..ATTRIBUTES_FOR_TILE_PLAYER + 0x200]
                .copy_from_slice(attrs);
        }

        pub(crate) fn set_floor_1_filler_high(&mut self, value: u8) {
            self.ram[FLOOR_1_FILLER_TILES + 1] = value;
        }

        pub(crate) fn set_floor_2_filler_high(&mut self, value: u8) {
            self.ram[FLOOR_2_FILLER_TILES + 1] = value;
        }

        pub(crate) fn set_staircase_index_high(&mut self, value: u8) {
            self.ram[WHICH_STAIRCASE_INDEX + 1] = value;
        }

        pub(crate) fn fill_bg2_attr_range(&mut self, start: usize, len: usize, value: u8) {
            self.ram[DUNGEON_BG2_ATTR_TABLE + start..DUNGEON_BG2_ATTR_TABLE + start + len]
                .fill(value);
        }

        pub(crate) fn clear_door_tables(&mut self) {
            self.ram[DOOR_TYPE_AND_SLOT..DOOR_TYPE_AND_SLOT + 32].fill(0);
            self.ram[DUNGEON_DOOR_DIRECTION..DUNGEON_DOOR_DIRECTION + 32].fill(0);
        }

        pub(crate) fn set_door_type_word(&mut self, door: usize, value: u16) {
            write_le_u16(self.ram, DOOR_TYPE_AND_SLOT + door * 2, value);
        }

        pub(crate) fn set_door_direction_word(&mut self, door: usize, value: u16) {
            write_le_u16(self.ram, DUNGEON_DOOR_DIRECTION + door * 2, value);
        }

        pub(crate) fn clear_door_direction(&mut self, door: usize) {
            self.set_door_direction_word(door, 0);
        }

        pub(crate) fn set_changeable_object_index(&mut self, index: usize, value: u8) {
            self.ram[CHANGEABLE_DUNGEON_OBJECT_INDEX + index] = value;
        }

        pub(crate) fn clear_changeable_object_index(&mut self, index: usize) {
            self.set_changeable_object_index(index, 0);
        }

        pub(crate) fn increment_trap_trigger_latch(&mut self) {
            self.ram[DUNGEON_TRAP_TRIGGER_LATCH] =
                self.ram[DUNGEON_TRAP_TRIGGER_LATCH].wrapping_add(1);
        }

        pub(crate) fn clear_trap_trigger_latch(&mut self) {
            self.ram[DUNGEON_TRAP_TRIGGER_LATCH] = 0;
        }

        pub(crate) fn set_room_history_entry(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, DUNGEON_ROOM_HISTORY + index * 2, value);
        }

        pub(crate) fn reset_room_history(&mut self) {
            for index in 0..4 {
                write_le_u16(self.ram, DUNGEON_ROOM_HISTORY + index * 2, 0xffff);
            }
        }

        pub(crate) fn set_floor_move_flags(&mut self, value: u8) {
            self.ram[DUNG_FLOOR_MOVE_FLAGS] = value;
        }

        pub(crate) fn increment_floor_move_flags(&mut self) {
            self.ram[DUNG_FLOOR_MOVE_FLAGS] = self.ram[DUNG_FLOOR_MOVE_FLAGS].wrapping_add(1);
        }

        pub(crate) fn clear_orange_blue_barrier_state(&mut self) {
            write_le_u16(self.ram, ORANGE_BLUE_BARRIER_STATE, 0);
        }

        pub(crate) fn clear_moving_floor_check_flags(&mut self) {
            write_le_u16(self.ram, MOVING_FLOOR_BG_CHECK_FLAGS, 0);
        }

        pub(crate) fn or_moving_floor_check_flags(&mut self, bits: u16) -> u16 {
            let next = read_le_u16(self.ram, MOVING_FLOOR_BG_CHECK_FLAGS) | bits;
            write_le_u16(self.ram, MOVING_FLOOR_BG_CHECK_FLAGS, next);
            next
        }

        pub(crate) fn copy_default_tile_attrs_head(&mut self, data: &[u8]) {
            self.ram[ATTRIBUTES_FOR_TILE_PLAYER..ATTRIBUTES_FOR_TILE_PLAYER + 0x140]
                .copy_from_slice(&data[..0x140]);
        }

        pub(crate) fn set_dungeon_dark_with_lantern(&mut self) {
            self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 1;
        }

        pub(crate) fn clear_dungeon_dark_with_lantern(&mut self) {
            self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 0;
        }

        pub(crate) fn toggle_orange_blue_barrier_state(&mut self) {
            self.ram[ORANGE_BLUE_BARRIER_STATE] ^= 1;
        }

        pub(crate) fn set_activate_bomb_trap_overlord(&mut self, value: u8) {
            self.ram[ACTIVATE_BOMB_TRAP_OVERLORD] = value;
        }
    }

    pub(crate) struct DungeonEntranceBackupViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DungeonEntranceBackupViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn cache_exit_tile_themes(&mut self) {
            self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX] = self.ram[OVERWORLD_TILE_THEME_INDEX];
            self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX + 1] = self.ram[MAIN_TILE_THEME_INDEX];
            self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX + 2] = self.ram[AUX_TILE_THEME_INDEX];
            self.ram[OVERWORLD_EXIT_TILE_THEME_INDEX + 3] = self.ram[SPRITE_GRAPHICS_INDEX];
        }

        pub(crate) fn clear_overworld_screen_high(&mut self) {
            self.ram[OVERWORLD_SCREEN_INDEX + 1] = 0;
        }

        pub(crate) fn clear_overlay_high(&mut self) {
            self.ram[OVERLAY_INDEX + 1] = 0;
        }
    }

    pub(crate) struct DungeonHeaderView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DungeonHeaderView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn travel_destination(&self, index: usize) -> u8 {
            byte(self.ram, DUNGEON_HEADER_TRAVEL_DESTINATIONS + index)
        }

        pub(crate) fn staircase_plane(&self, index: usize) -> u8 {
            byte(self.ram, DUNGEON_HEADER_STAIRCASE_PLANE + index)
        }
    }

    pub(crate) struct DungeonHeaderViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DungeonHeaderViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_hole_teleporter_planes(&mut self, packed: u8, extra: u8) {
            self.ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE] = packed & 3;
            self.ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 1] = (packed >> 2) & 3;
            self.ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 2] = (packed >> 4) & 3;
            self.ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 3] = (packed >> 6) & 3;
            self.ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 4] = extra & 3;
        }
    }

    pub(crate) struct DungeonKeySlotsView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DungeonKeySlotsView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn keys_earned(&self, palace_index_x2: u8) -> u8 {
            byte(
                self.ram,
                LINK_KEYS_EARNED_PER_DUNGEON + usize::from(palace_index_x2 >> 1),
            )
        }

        pub(crate) fn keys_earned_slot(&self, slot: usize) -> u8 {
            byte(self.ram, LINK_KEYS_EARNED_PER_DUNGEON + slot)
        }
    }

    pub(crate) struct DungeonKeySlotsViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DungeonKeySlotsViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_keys_earned(&mut self, palace_index_x2: u8, keys: u8) {
            self.ram[LINK_KEYS_EARNED_PER_DUNGEON + usize::from(palace_index_x2 >> 1)] = keys;
        }

        pub(crate) fn set_keys_earned_slot(&mut self, slot: usize, keys: u8) {
            self.ram[LINK_KEYS_EARNED_PER_DUNGEON + slot] = keys;
        }
    }

    pub(crate) struct DungeonTorchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DungeonTorchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn timer(&self, index: usize) -> u8 {
            byte(self.ram, TORCH_TIMERS + index)
        }

        pub(crate) fn attr_index(&self) -> usize {
            usize::from(byte(self.ram, DUNGEON_TORCH_ATTR) & 0x0f)
        }

        pub(crate) fn torch_attr(&self) -> u8 {
            byte(self.ram, DUNGEON_TORCH_ATTR)
        }

        pub(crate) fn ganon_torch_count(&self) -> u8 {
            byte(self.ram, GANON_TORCH_COUNT)
        }

        pub(crate) fn torches_start_index(&self) -> u16 {
            word(self.ram, DUNG_INDEX_OF_TORCHES_START)
        }
    }

    pub(crate) struct DungeonTorchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DungeonTorchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn copy_torch_init_to_movable_blocks(&mut self, torch_init: &[u8]) {
            self.ram[MOVABLE_BLOCK_DATAS + 99 * 4..MOVABLE_BLOCK_DATAS + 99 * 4 + 116]
                .copy_from_slice(&torch_init[..116]);
        }

        pub(crate) fn copy_torch_junk(&mut self, torch_junk: &[u8]) {
            self.ram[DUNGEON_TORCH_DATA + 144 * 2..DUNGEON_TORCH_DATA + 144 * 2 + torch_junk.len()]
                .copy_from_slice(torch_junk);
        }

        pub(crate) fn clear_timer(&mut self, index: usize) {
            self.ram[TORCH_TIMERS + index] = 0;
        }

        pub(crate) fn set_timer(&mut self, index: usize, value: u8) {
            self.ram[TORCH_TIMERS + index] = value;
        }

        pub(crate) fn set_torch_data_word(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, DUNGEON_TORCH_DATA + index * 2, value);
        }

        pub(crate) fn set_attr(&mut self, value: u8) {
            self.ram[DUNGEON_TORCH_ATTR] = value;
        }

        pub(crate) fn clear_attr(&mut self) {
            self.set_attr(0);
        }
    }

    pub(crate) struct ScratchWordView<'a> {
        ram: &'a [u8],
    }

    impl<'a> ScratchWordView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn high(&self) -> u8 {
            byte(self.ram, SCRATCH_R16 + 1)
        }

        pub(crate) fn word(&self) -> u16 {
            word(self.ram, SCRATCH_R16)
        }

        pub(crate) fn minigame_previous_chest_choice(&self) -> u8 {
            byte(self.ram, SCRATCH_R16)
        }
    }

    pub(crate) struct ScratchWordViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> ScratchWordViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn decrement_high(&mut self) -> u8 {
            let next = self.ram[SCRATCH_R16 + 1].wrapping_sub(1);
            self.ram[SCRATCH_R16 + 1] = next;
            next
        }

        pub(crate) fn set_word(&mut self, value: u16) {
            write_le_u16(self.ram, SCRATCH_R16, value);
        }

        pub(crate) fn clear_word(&mut self) {
            self.set_word(0);
        }

        pub(crate) fn set_liftable_tile_probe_position(&mut self, y: u16, x: u16) {
            write_le_u16(self.ram, SCRATCH_R16, y);
            write_le_u16(self.ram, SCRATCH_R18, x);
        }

        pub(crate) fn set_ganon_door_bounce_countdown(&mut self, value: u16) {
            self.set_word(value);
        }

        pub(crate) fn decrement_ganon_door_bounce_low(&mut self) -> u8 {
            let next = self.ram[SCRATCH_R16].wrapping_sub(1);
            self.ram[SCRATCH_R16] = next;
            next
        }

        pub(crate) fn clear_module_transition_counter(&mut self) {
            self.ram[SCRATCH_R16] = 0;
        }

        pub(crate) fn set_minigame_previous_chest_choice(&mut self, value: u8) {
            self.ram[SCRATCH_R16] = value;
        }
    }

    pub(crate) struct EndingScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> EndingScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn primary_word(&self) -> u16 {
            word(self.ram, ENDING_SCRATCH_PRIMARY)
        }

        pub(crate) fn secondary_word(&self) -> u16 {
            word(self.ram, ENDING_SCRATCH_SECONDARY)
        }

        pub(crate) fn primary_low(&self) -> u8 {
            byte(self.ram, ENDING_SCRATCH_PRIMARY)
        }

        pub(crate) fn secondary_low(&self) -> u8 {
            byte(self.ram, ENDING_SCRATCH_SECONDARY)
        }
    }

    pub(crate) struct EndingScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> EndingScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_primary_word(&mut self, value: u16) {
            write_le_u16(self.ram, ENDING_SCRATCH_PRIMARY, value);
        }

        pub(crate) fn set_secondary_word(&mut self, value: u16) {
            write_le_u16(self.ram, ENDING_SCRATCH_SECONDARY, value);
        }

        pub(crate) fn clear_primary_word(&mut self) {
            self.set_primary_word(0);
        }

        pub(crate) fn set_primary_low(&mut self, value: u8) {
            self.ram[ENDING_SCRATCH_PRIMARY] = value;
        }

        pub(crate) fn decrement_primary_low(&mut self) -> u8 {
            self.ram[ENDING_SCRATCH_PRIMARY] = self.ram[ENDING_SCRATCH_PRIMARY].wrapping_sub(1);
            self.ram[ENDING_SCRATCH_PRIMARY]
        }

        pub(crate) fn increment_secondary_low(&mut self) -> u8 {
            self.ram[ENDING_SCRATCH_SECONDARY] = self.ram[ENDING_SCRATCH_SECONDARY].wrapping_add(1);
            self.ram[ENDING_SCRATCH_SECONDARY]
        }
    }

    pub(crate) struct SaveLoadScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SaveLoadScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn source_offset(&self) -> u16 {
            word(self.ram, SAVE_LOAD_SOURCE_OFFSET)
        }

        pub(crate) fn source_offset_usize(&self) -> usize {
            usize::from(self.source_offset())
        }
    }

    pub(crate) struct SaveLoadScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SaveLoadScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_source_offset(&mut self, value: u16) {
            write_le_u16(self.ram, SAVE_LOAD_SOURCE_OFFSET, value);
        }
    }

    pub(crate) struct DungeonMapScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DungeonMapScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn scroll_draw_offset(&self) -> u16 {
            word(self.ram, DUNGEON_MAP_SCROLL_DRAW_OFFSET)
        }

        pub(crate) fn scroll_input(&self) -> u16 {
            word(self.ram, DUNGEON_MAP_SCROLL_INPUT)
        }

        pub(crate) fn scroll_input_direction_index(&self) -> usize {
            usize::from((self.scroll_input() >> 3) & 1)
        }

        pub(crate) fn marker_x_offset(&self) -> u16 {
            word(self.ram, DUNGEON_MAP_MARKER_X_OFFSET)
        }

        pub(crate) fn marker_y_offset(&self) -> u16 {
            word(self.ram, DUNGEON_MAP_MARKER_Y_OFFSET)
        }

        pub(crate) fn location_marker_base_y(&self) -> u8 {
            byte(self.ram, DUNGEON_MAP_LOCATION_MARKER_BASE_Y)
        }
    }

    pub(crate) struct DungeonMapScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DungeonMapScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear_scroll_state(&mut self) {
            write_le_u16(self.ram, DUNGEON_MAP_SCROLL_DRAW_OFFSET, 0);
            write_le_u16(self.ram, DUNGEON_MAP_SCROLL_INPUT, 0);
        }

        pub(crate) fn set_scroll_draw_offset(&mut self, value: u16) {
            write_le_u16(self.ram, DUNGEON_MAP_SCROLL_DRAW_OFFSET, value);
        }

        pub(crate) fn set_scroll_input(&mut self, value: u16) {
            write_le_u16(self.ram, DUNGEON_MAP_SCROLL_INPUT, value);
        }

        pub(crate) fn reset_marker_offsets(&mut self) {
            write_le_u16(self.ram, DUNGEON_MAP_MARKER_X_OFFSET, 0x0040);
            write_le_u16(self.ram, DUNGEON_MAP_MARKER_Y_OFFSET, 0x0040);
        }

        pub(crate) fn set_marker_x_offset(&mut self, value: u16) {
            write_le_u16(self.ram, DUNGEON_MAP_MARKER_X_OFFSET, value);
        }

        pub(crate) fn set_marker_y_offset(&mut self, value: u16) {
            write_le_u16(self.ram, DUNGEON_MAP_MARKER_Y_OFFSET, value);
        }

        pub(crate) fn set_location_marker_base_y(&mut self, value: u8) {
            self.ram[DUNGEON_MAP_LOCATION_MARKER_BASE_Y] = value;
            self.ram[DUNGEON_MAP_LOCATION_MARKER_BASE_Y + 1] = 0;
        }

        pub(crate) fn shift_marker_x_left(&mut self) -> u16 {
            let value = word(self.ram, DUNGEON_MAP_MARKER_X_OFFSET).wrapping_sub(0x10);
            write_le_u16(self.ram, DUNGEON_MAP_MARKER_X_OFFSET, value);
            value
        }

        pub(crate) fn reset_marker_x_offset(&mut self) {
            write_le_u16(self.ram, DUNGEON_MAP_MARKER_X_OFFSET, 0x0040);
        }

        pub(crate) fn shift_marker_y_low_up(&mut self) {
            self.ram[DUNGEON_MAP_MARKER_Y_OFFSET] =
                self.ram[DUNGEON_MAP_MARKER_Y_OFFSET].wrapping_sub(0x10);
        }

        pub(crate) fn add_marker_y_offset_signed(&mut self, value: i16) -> u16 {
            let value = word(self.ram, DUNGEON_MAP_MARKER_Y_OFFSET).wrapping_add_signed(value);
            write_le_u16(self.ram, DUNGEON_MAP_MARKER_Y_OFFSET, value);
            value
        }
    }

    pub(crate) struct TempCounterView<'a> {
        ram: &'a [u8],
    }

    impl<'a> TempCounterView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn value(&self) -> u8 {
            byte(self.ram, TEMP_COUNTER)
        }

        pub(crate) fn as_usize(&self) -> usize {
            usize::from(self.value())
        }

        pub(crate) fn is_negative(&self) -> bool {
            (self.value() as i8).is_negative()
        }
    }

    pub(crate) struct TempCounterViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> TempCounterViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set(&mut self, value: u8) {
            self.ram[TEMP_COUNTER] = value;
        }

        pub(crate) fn decrement(&mut self) -> u8 {
            let next = self.ram[TEMP_COUNTER].wrapping_sub(1);
            self.ram[TEMP_COUNTER] = next;
            next
        }
    }

    pub(crate) struct DungeonSecretScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DungeonSecretScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn pending_kind(&self) -> u8 {
            byte(self.ram, DUNGEON_SECRET_PENDING_KIND)
        }

        pub(crate) fn overworld_subst_counter(&self) -> u8 {
            byte(self.ram, OVERWORLD_SECRET_SUBST_CTR)
        }

        pub(crate) fn has_pending_kind(&self) -> bool {
            self.pending_kind() != 0
        }

        pub(crate) fn is_available(&self) -> bool {
            self.pending_kind() != 0xff
        }

        pub(crate) fn graphics_kind(&self) -> Option<u8> {
            let value = self.pending_kind();
            if value & 0x80 != 0 {
                Some(value & 0x7f)
            } else {
                None
            }
        }
    }

    pub(crate) struct DungeonSecretScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DungeonSecretScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear_pending_kind(&mut self) {
            self.ram[DUNGEON_SECRET_PENDING_KIND] = 0;
        }

        pub(crate) fn set_pending_kind(&mut self, value: u8) {
            self.ram[DUNGEON_SECRET_PENDING_KIND] = value;
        }

        pub(crate) fn increment_overworld_subst_counter(&mut self) {
            self.ram[OVERWORLD_SECRET_SUBST_CTR] =
                self.ram[OVERWORLD_SECRET_SUBST_CTR].wrapping_add(1);
        }

        pub(crate) fn set_powder_pending_kind(&mut self) {
            write_le_u16(self.ram, DUNGEON_SECRET_PENDING_KIND, 4);
        }

        pub(crate) fn or_pending_kind(&mut self, value: u8) {
            self.ram[DUNGEON_SECRET_PENDING_KIND] |= value;
        }

        pub(crate) fn mark_graphics_kind(&mut self) {
            self.ram[DUNGEON_SECRET_PENDING_KIND] |= 0x80;
        }
    }

    pub(crate) struct OverworldEventInfoView<'a> {
        ram: &'a [u8],
    }

    impl<'a> OverworldEventInfoView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn event_info(&self, screen: usize) -> u8 {
            byte(self.ram, OVERWORLD_EVENT_INFO + screen)
        }

        pub(crate) fn has_event_bits(&self, screen: usize, mask: u8) -> bool {
            self.event_info(screen) & mask != 0
        }
    }

    pub(crate) struct OverworldEventInfoViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> OverworldEventInfoViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_event_info(&mut self, screen: usize, value: u8) {
            self.ram[OVERWORLD_EVENT_INFO + screen] = value;
        }

        pub(crate) fn set_event_bits(&mut self, screen: usize, mask: u8) {
            self.ram[OVERWORLD_EVENT_INFO + screen] |= mask;
        }

        pub(crate) fn clear_event_bits(&mut self, screen: usize, mask: u8) {
            self.ram[OVERWORLD_EVENT_INFO + screen] &= !mask;
        }
    }

    pub(crate) struct OverworldConfigTableView<'a> {
        ram: &'a [u8],
    }

    impl<'a> OverworldConfigTableView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn music(&self, screen: usize) -> u8 {
            byte(self.ram, OVERWORLD_MUSIC_TABLE + screen)
        }

        pub(crate) fn current_music(&self) -> u8 {
            self.music(usize::from(byte(self.ram, OVERWORLD_SCREEN_INDEX)))
        }

        pub(crate) fn sprite_palette(&self, screen: usize) -> u8 {
            byte(self.ram, OVERWORLD_SPRITE_PALETTE_TABLE + screen)
        }

        pub(crate) fn sprite_graphics(&self, screen: usize) -> u8 {
            byte(self.ram, OVERWORLD_SPRITE_GFX_TABLE + screen)
        }
    }

    pub(crate) struct PaletteBufferView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PaletteBufferView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn main_color(&self, index: usize) -> u16 {
            word(self.ram, MAIN_PALETTE_BUFFER + index * 2)
        }

        pub(crate) fn aux_color(&self, index: usize) -> u16 {
            word(self.ram, AUX_PALETTE_BUFFER + index * 2)
        }

        pub(crate) fn aux_visible_slice(&self) -> &[u8] {
            &self.ram[AUX_PALETTE_BUFFER..AUX_PALETTE_BUFFER + 256]
        }

        pub(crate) fn main_full_slice(&self) -> &[u8] {
            &self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 512]
        }

        pub(crate) fn aux_full_slice(&self) -> &[u8] {
            &self.ram[AUX_PALETTE_BUFFER..AUX_PALETTE_BUFFER + 512]
        }

        pub(crate) fn overworld_aux_or_main_offset(&self) -> u16 {
            word(self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN)
        }
    }

    pub(crate) struct PaletteBufferViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PaletteBufferViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear_aux_visible_subpalettes(&mut self) {
            self.ram[AUX_PALETTE_BUFFER + 32 * 2..AUX_PALETTE_BUFFER + 32 * 2 + 192].fill(0);
        }

        pub(crate) fn clear_main_visible_subpalettes(&mut self) {
            self.ram[MAIN_PALETTE_BUFFER + 32 * 2..MAIN_PALETTE_BUFFER + 32 * 2 + 192].fill(0);
        }

        pub(crate) fn clear_aux_sprite_subpalettes(&mut self) {
            self.ram[AUX_PALETTE_BUFFER + 0x180..AUX_PALETTE_BUFFER + 0x200].fill(0);
        }

        pub(crate) fn set_main_color(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, MAIN_PALETTE_BUFFER + index * 2, value);
        }

        pub(crate) fn set_aux_color(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, AUX_PALETTE_BUFFER + index * 2, value);
        }

        pub(crate) fn set_overworld_aux_or_main_offset(&mut self, value: u16) {
            write_le_u16(self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, value);
        }

        pub(crate) fn clear_overworld_aux_or_main_offset(&mut self) {
            self.set_overworld_aux_or_main_offset(0);
        }

        pub(crate) fn select_overworld_aux_palette_offset(&mut self) {
            self.set_overworld_aux_or_main_offset(0x0200);
        }

        pub(crate) fn keep_overworld_aux_or_main_low_byte(&mut self) {
            let value = word(self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN) & 0x00ff;
            self.set_overworld_aux_or_main_offset(value);
        }

        pub(crate) fn clear_main_full(&mut self) {
            self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 0x200].fill(0);
        }

        pub(crate) fn copy_aux_visible_from(&mut self, palette: &[u8]) {
            self.ram[AUX_PALETTE_BUFFER..AUX_PALETTE_BUFFER + 256].copy_from_slice(palette);
        }

        pub(crate) fn copy_aux_full_from(&mut self, palette: &[u8]) {
            self.ram[AUX_PALETTE_BUFFER..AUX_PALETTE_BUFFER + 512].copy_from_slice(palette);
        }

        pub(crate) fn copy_main_full_from(&mut self, palette: &[u8]) {
            self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 512].copy_from_slice(palette);
        }

        pub(crate) fn set_sp0l(&mut self, value: u8) {
            self.ram[PALETTE_SP0L] = value;
        }

        pub(crate) fn set_sp5l(&mut self, value: u8) {
            self.ram[PALETTE_SP5L] = value;
        }

        pub(crate) fn set_sp6l(&mut self, value: u8) {
            self.ram[PALETTE_SP6L] = value;
        }

        pub(crate) fn set_palette_main_indoors(&mut self, value: u8) {
            self.ram[PALETTE_MAIN_INDOORS] = value;
        }

        pub(crate) fn set_hud_palette(&mut self, value: u8) {
            self.ram[HUD_PALETTE] = value;
        }

        pub(crate) fn set_sp6r_indoors(&mut self, value: u8) {
            self.ram[PALETTE_SP6R_INDOORS] = value;
        }

        pub(crate) fn set_overworld_palette_aux2_hi(&mut self, value: u8) {
            self.ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = value;
        }

        pub(crate) fn set_overworld_palette_aux3_lo(&mut self, value: u8) {
            self.ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = value;
        }

        pub(crate) fn set_bg_tile_animation_countdown(&mut self, value: u16) {
            write_le_u16(self.ram, BG_TILE_ANIMATION_COUNTDOWN, value);
        }

        pub(crate) fn set_overworld_palette_mode(&mut self, value: u8) {
            self.ram[OVERWORLD_PALETTE_MODE] = value;
        }
    }

    pub(crate) struct PaletteFilterView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PaletteFilterView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn countdown(&self) -> u8 {
            byte(self.ram, PALETTE_FILTER_COUNTDOWN)
        }

        pub(crate) fn countdown_word(&self) -> u16 {
            word(self.ram, PALETTE_FILTER_COUNTDOWN)
        }

        pub(crate) fn darkening_or_lightening_screen(&self) -> u8 {
            byte(self.ram, DARKENING_OR_LIGHTENING_SCREEN)
        }

        pub(crate) fn darkening_or_lightening_screen_word(&self) -> u16 {
            word(self.ram, DARKENING_OR_LIGHTENING_SCREEN)
        }

        pub(crate) fn color_window_selection(&self) -> u8 {
            byte(self.ram, CGWSEL_COPY)
        }

        pub(crate) fn color_window_and_math_word(&self) -> u16 {
            word(self.ram, CGWSEL_COPY)
        }

        pub(crate) fn color_math_control(&self) -> u8 {
            byte(self.ram, CGADSUB_COPY)
        }

        pub(crate) fn color_math_control_word(&self) -> u16 {
            word(self.ram, CGADSUB_COPY)
        }

        pub(crate) fn fixed_color_red(&self) -> u8 {
            byte(self.ram, COLDATA_COPY0)
        }

        pub(crate) fn fixed_color_green(&self) -> u8 {
            byte(self.ram, COLDATA_COPY1)
        }

        pub(crate) fn fixed_color_blue(&self) -> u8 {
            byte(self.ram, COLDATA_COPY2)
        }

        pub(crate) fn fixed_color_component(&self, index: usize) -> u8 {
            byte(self.ram, COLDATA_COPY0 + index)
        }
    }

    pub(crate) struct PaletteFilterViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PaletteFilterViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_countdown(&mut self, value: u8) {
            self.ram[PALETTE_FILTER_COUNTDOWN] = value;
        }

        pub(crate) fn increment_countdown(&mut self) {
            self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_add(1);
        }

        pub(crate) fn decrement_countdown(&mut self) {
            self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_sub(1);
        }

        pub(crate) fn set_countdown_word(&mut self, value: u16) {
            write_le_u16(self.ram, PALETTE_FILTER_COUNTDOWN, value);
        }

        pub(crate) fn set_darkening_or_lightening_screen(&mut self, value: u8) {
            self.ram[DARKENING_OR_LIGHTENING_SCREEN] = value;
        }

        pub(crate) fn xor_darkening_or_lightening_screen(&mut self, value: u8) {
            self.ram[DARKENING_OR_LIGHTENING_SCREEN] ^= value;
        }

        pub(crate) fn set_darkening_or_lightening_screen_word(&mut self, value: u16) {
            write_le_u16(self.ram, DARKENING_OR_LIGHTENING_SCREEN, value);
        }

        pub(crate) fn set_color_window_selection(&mut self, value: u8) {
            self.ram[CGWSEL_COPY] = value;
        }

        pub(crate) fn set_color_window_and_math_word(&mut self, value: u16) {
            write_le_u16(self.ram, CGWSEL_COPY, value);
        }

        pub(crate) fn set_color_math_control(&mut self, value: u8) {
            self.ram[CGADSUB_COPY] = value;
        }

        pub(crate) fn set_fixed_color_red(&mut self, value: u8) {
            self.ram[COLDATA_COPY0] = value;
        }

        pub(crate) fn or_fixed_color_red(&mut self, value: u8) {
            self.ram[COLDATA_COPY0] |= value;
        }

        pub(crate) fn subtract_fixed_color_red(&mut self, value: u8) {
            self.ram[COLDATA_COPY0] = self.ram[COLDATA_COPY0].wrapping_sub(value);
        }

        pub(crate) fn set_fixed_color_green(&mut self, value: u8) {
            self.ram[COLDATA_COPY1] = value;
        }

        pub(crate) fn or_fixed_color_green(&mut self, value: u8) {
            self.ram[COLDATA_COPY1] |= value;
        }

        pub(crate) fn subtract_fixed_color_green(&mut self, value: u8) {
            self.ram[COLDATA_COPY1] = self.ram[COLDATA_COPY1].wrapping_sub(value);
        }

        pub(crate) fn set_fixed_color_blue(&mut self, value: u8) {
            self.ram[COLDATA_COPY2] = value;
        }

        pub(crate) fn or_fixed_color_blue(&mut self, value: u8) {
            self.ram[COLDATA_COPY2] |= value;
        }

        pub(crate) fn subtract_fixed_color_blue(&mut self, value: u8) {
            self.ram[COLDATA_COPY2] = self.ram[COLDATA_COPY2].wrapping_sub(value);
        }

        pub(crate) fn set_fixed_color_component(&mut self, index: usize, value: u8) {
            self.ram[COLDATA_COPY0 + index] = value;
        }

        pub(crate) fn or_fixed_color_component(&mut self, index: usize, value: u8) {
            self.ram[COLDATA_COPY0 + index] |= value;
        }
    }

    pub(crate) struct HudStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> HudStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn floor_changed_timer_low(&self) -> u8 {
            byte(self.ram, HUD_FLOOR_CHANGED_TIMER)
        }

        pub(crate) fn super_bomb_indicator_timer(&self) -> u8 {
            byte(self.ram, SUPER_BOMB_INDICATOR_TIMER)
        }

        pub(crate) fn super_bomb_indicator_counter(&self) -> u8 {
            byte(self.ram, SUPER_BOMB_INDICATOR_COUNTER)
        }

        pub(crate) fn rupee_sfx_sound_delay(&self) -> u8 {
            byte(self.ram, RUPEE_SFX_SOUND_DELAY)
        }

        pub(crate) fn is_doing_heart_animation(&self) -> bool {
            byte(self.ram, IS_DOING_HEART_ANIMATION) != 0
        }

        pub(crate) fn is_doing_heart_animation_raw(&self) -> u8 {
            byte(self.ram, IS_DOING_HEART_ANIMATION)
        }

        pub(crate) fn heart_refill_countdown(&self) -> u8 {
            byte(self.ram, HEART_REFILL_COUNTDOWN)
        }

        pub(crate) fn heart_refill_anim_subpos(&self) -> u8 {
            byte(self.ram, HEART_REFILL_ANIM_SUBPOS)
        }

        pub(crate) fn flashing_circle_timer(&self) -> u8 {
            byte(self.ram, FLASHING_CIRCLE_TIMER)
        }

        pub(crate) fn prev_joypad_h(&self) -> u8 {
            byte(self.ram, MENU_PREV_JOYPAD_H)
        }

        pub(crate) fn equipment_menu_exit_state(&self) -> u8 {
            byte(self.ram, EQUIPMENT_MENU_EXIT_STATE)
        }

        pub(crate) fn bottle_menu_row(&self) -> u8 {
            byte(self.ram, BOTTLE_MENU_ROW)
        }

        pub(crate) fn dungeon_dark_with_lantern(&self) -> bool {
            byte(self.ram, HDR_DUNGEON_DARK_WITH_LANTERN) != 0
        }

        pub(crate) fn tick_counter(&self) -> u8 {
            byte(self.ram, HUD_MODULE_TICK_COUNTER)
        }
    }

    pub(crate) struct HudStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> HudStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_floor_changed_timer(&mut self, value: u16) {
            write_le_u16(self.ram, HUD_FLOOR_CHANGED_TIMER, value);
        }

        pub(crate) fn set_super_bomb_indicator_timer(&mut self, value: u8) {
            self.ram[SUPER_BOMB_INDICATOR_TIMER] = value;
        }

        pub(crate) fn set_super_bomb_indicator_counter(&mut self, value: u8) {
            self.ram[SUPER_BOMB_INDICATOR_COUNTER] = value;
        }

        pub(crate) fn set_rupee_sfx_sound_delay(&mut self, value: u8) {
            self.ram[RUPEE_SFX_SOUND_DELAY] = value;
        }

        pub(crate) fn set_is_doing_heart_animation(&mut self, value: u8) {
            self.ram[IS_DOING_HEART_ANIMATION] = value;
        }

        pub(crate) fn clear_is_doing_heart_animation(&mut self) {
            self.ram[IS_DOING_HEART_ANIMATION] = 0;
        }

        pub(crate) fn set_heart_refill_countdown(&mut self, value: u8) {
            self.ram[HEART_REFILL_COUNTDOWN] = value;
        }

        pub(crate) fn set_heart_refill_anim_subpos(&mut self, value: u8) {
            self.ram[HEART_REFILL_ANIM_SUBPOS] = value;
        }

        pub(crate) fn set_flashing_circle_timer(&mut self, value: u8) {
            self.ram[FLASHING_CIRCLE_TIMER] = value;
        }

        pub(crate) fn set_prev_joypad_h(&mut self, value: u8) {
            self.ram[MENU_PREV_JOYPAD_H] = value;
        }

        pub(crate) fn clear_prev_joypad_h(&mut self) {
            self.ram[MENU_PREV_JOYPAD_H] = 0;
        }

        pub(crate) fn set_equipment_menu_exit_state(&mut self, value: u8) {
            self.ram[EQUIPMENT_MENU_EXIT_STATE] = value;
        }

        pub(crate) fn set_bottle_menu_row(&mut self, value: u8) {
            self.ram[BOTTLE_MENU_ROW] = value;
        }

        pub(crate) fn set_dungeon_dark_with_lantern(&mut self) {
            self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 1;
        }

        pub(crate) fn set_tick_counter(&mut self, value: u8) {
            self.ram[HUD_MODULE_TICK_COUNTER] = value;
        }
    }

    pub(crate) struct HudInventoryOrderView<'a> {
        ram: &'a [u8],
    }

    impl<'a> HudInventoryOrderView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn is_custom(&self) -> bool {
            byte(self.ram, HUD_INVENTORY_ORDER) != 0
        }

        pub(crate) fn item(&self, index: usize) -> u8 {
            byte(self.ram, HUD_INVENTORY_ORDER + index)
        }
    }

    pub(crate) struct HudInventoryOrderViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> HudInventoryOrderViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn initialize_default_order(&mut self, count: usize) {
            for i in 0..count {
                self.ram[HUD_INVENTORY_ORDER + i] = i as u8 + 1;
            }
        }

        pub(crate) fn swap_items(&mut self, old_pos: usize, new_pos: usize) {
            self.ram
                .swap(HUD_INVENTORY_ORDER + old_pos, HUD_INVENTORY_ORDER + new_pos);
        }
    }

    pub(crate) struct GraphicsScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> GraphicsScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_aux_bg_subset_pack(&mut self, index: usize, value: u8) {
            self.ram[AUX_BG_SUBSET_0 + index] = value;
        }

        pub(crate) fn primary_decomp_buffer_offset() -> usize {
            PRIMARY_DECOMP_BUFFER_LOAD_GFX
        }

        pub(crate) fn secondary_decomp_buffer_offset() -> usize {
            SECONDARY_DECOMP_BUFFER_LOAD_GFX
        }

        pub(crate) fn primary_decomp_buffer(&self, len: usize) -> Vec<u8> {
            self.ram[PRIMARY_DECOMP_BUFFER_LOAD_GFX..PRIMARY_DECOMP_BUFFER_LOAD_GFX + len].to_vec()
        }

        pub(crate) fn combined_decomp_buffers(&self) -> Vec<u8> {
            self.primary_decomp_buffer(0x0c00)
        }

        pub(crate) fn copy_to_primary_decomp_buffer(&mut self, data: &[u8]) {
            let len = data.len().min(
                self.ram
                    .len()
                    .saturating_sub(PRIMARY_DECOMP_BUFFER_LOAD_GFX),
            );
            self.ram[PRIMARY_DECOMP_BUFFER_LOAD_GFX..PRIMARY_DECOMP_BUFFER_LOAD_GFX + len]
                .copy_from_slice(&data[..len]);
        }

        pub(crate) fn copy_message_rows(
            &mut self,
            dst: usize,
            src0: usize,
            src1: usize,
            len: usize,
        ) {
            for i in 0..len {
                self.ram[MESSAGING_RENDER_BUFFER + dst + i] = self.ram[src0 + i];
                self.ram[MESSAGING_RENDER_BUFFER + dst + len + i] = self.ram[src1 + i];
            }
        }

        pub(crate) fn clear_agahnim_palette_settings(&mut self, len: usize) {
            self.ram[AGAHNIM_PAL_SETTING..AGAHNIM_PAL_SETTING + len].fill(0);
        }

        pub(crate) fn sprite_decomp_buffer_tail(&self) -> Vec<u8> {
            self.ram[SPRITE_DECOMP_BUFFER_LOAD_GFX..GRAPHICS_DECOMP_BUFFER_END].to_vec()
        }

        pub(crate) fn staged_bg_and_sprite_decomp_buffers(&self) -> Vec<u8> {
            self.ram[BG_DECOMP_BUFFER_LOAD_GFX..GRAPHICS_DECOMP_BUFFER_END].to_vec()
        }
    }

    pub(crate) struct DungeonMapViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DungeonMapViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear_current_floor_high(&mut self) {
            self.ram[DUNGEON_MAP_CURRENT_FLOOR + 1] = 0;
        }
    }

    pub(crate) struct OverworldConfigTableViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> OverworldConfigTableViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn copy_music_primary(&mut self, data: &[u8]) {
            self.ram[OVERWORLD_MUSIC_TABLE..OVERWORLD_MUSIC_TABLE + 64]
                .copy_from_slice(&data[..64]);
        }

        pub(crate) fn copy_music_secondary(&mut self, data: &[u8]) {
            self.ram[OVERWORLD_MUSIC_TABLE + 64..OVERWORLD_MUSIC_TABLE + 160]
                .copy_from_slice(&data[..96]);
        }

        pub(crate) fn set_music(&mut self, screen: usize, value: u8) {
            self.ram[OVERWORLD_MUSIC_TABLE + screen] = value;
        }
    }

    pub(crate) struct OverworldScreenSizeView<'a> {
        ram: &'a [u8],
    }

    impl<'a> OverworldScreenSizeView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn is_big_area_word(&self) -> u16 {
            word(self.ram, OVERWORLD_AREA_IS_BIG)
        }

        pub(crate) fn right_bottom_bound_word(&self) -> u16 {
            word(self.ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND)
        }
    }

    pub(crate) struct OverworldScreenSizeViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> OverworldScreenSizeViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear_big_area_high(&mut self) {
            self.ram[OVERWORLD_AREA_IS_BIG + 1] = 0;
        }

        pub(crate) fn set_big_area_low(&mut self, value: u8) {
            self.ram[OVERWORLD_AREA_IS_BIG] = value;
        }

        pub(crate) fn backup_big_area_low(&mut self) {
            self.ram[OVERWORLD_AREA_IS_BIG_BACKUP] = self.ram[OVERWORLD_AREA_IS_BIG];
        }

        pub(crate) fn set_right_bottom_bound_low(&mut self, value: u8) {
            self.ram[OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND] = value;
        }

        pub(crate) fn set_right_bottom_bound_high(&mut self, value: u8) {
            self.ram[OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND + 1] = value;
        }
    }

    pub(crate) struct OverworldMap16DecodeView<'a> {
        ram: &'a [u8],
    }

    impl<'a> OverworldMap16DecodeView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn source_byte(&self, index: usize) -> u8 {
            byte(self.ram, OVERWORLD_MAP16_DECODE_SRC + index)
        }
    }

    pub(crate) struct OverworldMap16DecodeViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> OverworldMap16DecodeViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn copy_source_from(&mut self, data: &[u8]) {
            self.ram[OVERWORLD_MAP16_DECODE_SRC..OVERWORLD_MAP16_DECODE_SRC + data.len()]
                .copy_from_slice(data);
        }

        pub(crate) fn copy_scratch_to_source_words_high(&mut self, len: usize) {
            for i in 0..len {
                self.ram[OVERWORLD_MAP16_DECODE_SRC + 1 + i * 2] =
                    self.ram[OVERWORLD_DECOMP_SCRATCH + i];
            }
        }

        pub(crate) fn copy_scratch_to_source_words_low(&mut self, len: usize) {
            for i in 0..len {
                self.ram[OVERWORLD_MAP16_DECODE_SRC + i * 2] =
                    self.ram[OVERWORLD_DECOMP_SCRATCH + i];
            }
        }
    }

    pub(crate) struct RoomBoundsView<'a> {
        ram: &'a [u8],
    }

    impl<'a> RoomBoundsView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn y_bound(&self, index: usize) -> u16 {
            word(self.ram, ROOM_BOUNDS + index * 2)
        }

        pub(crate) fn x_bound(&self, index: usize) -> u16 {
            word(self.ram, ROOM_BOUNDS + 8 + index * 2)
        }

        pub(crate) fn packed_bound(&self, index: usize) -> u16 {
            word(self.ram, ROOM_BOUNDS + index * 2)
        }

        pub(crate) fn packed_top(&self) -> u16 {
            self.packed_bound(0)
        }

        pub(crate) fn packed_bottom(&self) -> u16 {
            self.packed_bound(1)
        }

        pub(crate) fn packed_left(&self) -> u16 {
            self.packed_bound(2)
        }

        pub(crate) fn packed_right(&self) -> u16 {
            self.packed_bound(3)
        }
    }

    pub(crate) struct RoomBoundsViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> RoomBoundsViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_y_bound(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, ROOM_BOUNDS + index * 2, value);
        }

        pub(crate) fn set_x_bound(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, ROOM_BOUNDS + 8 + index * 2, value);
        }

        pub(crate) fn set_packed_bound(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, ROOM_BOUNDS + index * 2, value);
        }

        pub(crate) fn set_packed_bounds(&mut self, top: u16, bottom: u16, left: u16, right: u16) {
            self.set_packed_bound(0, top);
            self.set_packed_bound(1, bottom);
            self.set_packed_bound(2, left);
            self.set_packed_bound(3, right);
        }

        pub(crate) fn copy_y_bound_from(&mut self, index: usize, src: usize) {
            let value = word(self.ram, src);
            self.set_y_bound(index, value);
        }

        pub(crate) fn copy_x_bound_from(&mut self, index: usize, src: usize) {
            let value = word(self.ram, src);
            self.set_x_bound(index, value);
        }

        pub(crate) fn copy_packed_bound_from(&mut self, index: usize, src: usize) {
            let value = word(self.ram, src);
            self.set_packed_bound(index, value);
        }

        pub(crate) fn add_y_bounds_a(&mut self, value: u16) {
            for index in [0, 2] {
                let next = word(self.ram, ROOM_BOUNDS + index * 2).wrapping_add(value);
                self.set_y_bound(index, next);
            }
        }

        pub(crate) fn add_y_bounds_b(&mut self, value: u16) {
            for index in [1, 3] {
                let next = word(self.ram, ROOM_BOUNDS + index * 2).wrapping_add(value);
                self.set_y_bound(index, next);
            }
        }

        pub(crate) fn add_x_bounds_a(&mut self, value: u16) {
            for index in [0, 2] {
                let next = word(self.ram, ROOM_BOUNDS + 8 + index * 2).wrapping_add(value);
                self.set_x_bound(index, next);
            }
        }

        pub(crate) fn add_x_bounds_b(&mut self, value: u16) {
            for index in [1, 3] {
                let next = word(self.ram, ROOM_BOUNDS + 8 + index * 2).wrapping_add(value);
                self.set_x_bound(index, next);
            }
        }

        pub(crate) fn copy_y_bounds_from(&mut self, src: usize, count: usize) {
            for i in 0..count {
                self.ram[ROOM_BOUNDS + i] = self.ram[src + i];
            }
        }
    }

    pub(crate) struct VramUploadDataView<'a> {
        ram: &'a [u8],
    }

    impl<'a> VramUploadDataView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn offset(&self) -> u16 {
            word(self.ram, VRAM_UPLOAD_OFFSET)
        }

        pub(crate) fn offset_usize(&self) -> usize {
            usize::from(self.offset())
        }

        pub(crate) fn data_base(&self) -> usize {
            VRAM_UPLOAD_DATA
        }

        pub(crate) fn data_address(&self, offset: usize) -> usize {
            VRAM_UPLOAD_DATA + offset
        }

        pub(crate) fn current_data_address(&self) -> usize {
            self.data_address(self.offset_usize())
        }

        pub(crate) fn word(&self, offset: usize) -> u16 {
            word(self.ram, VRAM_UPLOAD_DATA + offset)
        }

        pub(crate) fn tilemap_word(&self, offset: usize) -> u16 {
            word(self.ram, VRAM_UPLOAD_OFFSET + offset)
        }

        pub(crate) fn byte(&self, offset: usize) -> u8 {
            byte(self.ram, VRAM_UPLOAD_DATA + offset)
        }

        pub(crate) fn remaining_data(&self) -> &[u8] {
            &self.ram[VRAM_UPLOAD_DATA..]
        }
    }

    pub(crate) struct VramUploadDataViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> VramUploadDataViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_offset(&mut self, value: u16) {
            write_le_u16(self.ram, VRAM_UPLOAD_OFFSET, value);
        }

        pub(crate) fn clear_offset(&mut self) {
            self.set_offset(0);
        }

        pub(crate) fn advance_offset_by(&mut self, value: u16) -> u16 {
            let next = read_le_u16(self.ram, VRAM_UPLOAD_OFFSET).wrapping_add(value);
            write_le_u16(self.ram, VRAM_UPLOAD_OFFSET, next);
            next
        }

        pub(crate) fn set_byte(&mut self, offset: usize, value: u8) {
            self.ram[VRAM_UPLOAD_DATA + offset] = value;
        }

        pub(crate) fn set_word(&mut self, offset: usize, value: u16) {
            write_le_u16(self.ram, VRAM_UPLOAD_DATA + offset, value);
        }

        pub(crate) fn set_tilemap_word(&mut self, offset: usize, value: u16) {
            write_le_u16(self.ram, VRAM_UPLOAD_OFFSET + offset, value);
        }

        pub(crate) fn set_level_label_tiles(&mut self, left: &[u8; 14], right: &[u8; 14]) {
            self.ram[VRAM_UPLOAD_DATA + 32] = 0xff;
            for i in (0..14).rev() {
                self.ram[VRAM_UPLOAD_DATA + i] = left[i];
                self.ram[VRAM_UPLOAD_DATA + i + 16] = right[i];
            }
        }

        pub(crate) fn terminate_at(&mut self, offset: usize) {
            self.ram[VRAM_UPLOAD_DATA + offset] = 0xff;
        }

        pub(crate) fn copy_bytes(&mut self, offset: usize, data: &[u8]) {
            self.ram[VRAM_UPLOAD_DATA + offset..VRAM_UPLOAD_DATA + offset + data.len()]
                .copy_from_slice(data);
        }
    }

    pub(crate) struct PolyStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PolyStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn config1(&self) -> u8 {
            byte(self.ram, POLY_CONFIG1)
        }

        pub(crate) fn color_mode(&self) -> u8 {
            byte(self.ram, POLY_CONFIG_COLOR_MODE)
        }

        pub(crate) fn model(&self) -> u8 {
            byte(self.ram, POLY_WHICH_MODEL)
        }

        pub(crate) fn angle_a(&self) -> u8 {
            byte(self.ram, POLY_A)
        }

        pub(crate) fn angle_b(&self) -> u8 {
            byte(self.ram, POLY_B)
        }

        pub(crate) fn base_x(&self) -> u8 {
            byte(self.ram, POLY_BASE_X)
        }

        pub(crate) fn base_y(&self) -> u8 {
            byte(self.ram, POLY_BASE_Y)
        }

        pub(crate) fn shape_depth_bias_low(&self) -> u8 {
            byte(self.ram, POLY_SHAPE_DEPTH_BIAS)
        }

        pub(crate) fn shape_depth_bias(&self) -> u16 {
            word(self.ram, POLY_SHAPE_DEPTH_BIAS)
        }

        pub(crate) fn num_vertices(&self) -> u8 {
            byte(self.ram, POLY_CONFIG_NUM_VERTEX)
        }

        pub(crate) fn num_polys(&self) -> u8 {
            byte(self.ram, POLY_CONFIG_NUM_POLYS)
        }

        pub(crate) fn fromlut_x(&self) -> i8 {
            byte(self.ram, POLY_FROMLUT_X) as i8
        }

        pub(crate) fn fromlut_y(&self) -> i8 {
            byte(self.ram, POLY_FROMLUT_Y) as i8
        }

        pub(crate) fn fromlut_z(&self) -> i8 {
            byte(self.ram, POLY_FROMLUT_Z) as i8
        }

        pub(crate) fn f0(&self) -> u8 {
            byte(self.ram, POLY_F0)
        }

        pub(crate) fn f1(&self) -> u8 {
            byte(self.ram, POLY_F1)
        }

        pub(crate) fn num_vertex_in_poly(&self) -> u8 {
            byte(self.ram, POLY_NUM_VERTEX_IN_POLY)
        }

        pub(crate) fn raster_color_config(&self) -> u8 {
            byte(self.ram, POLY_RASTER_COLOR_CONFIG)
        }

        pub(crate) fn tmp0(&self) -> u8 {
            byte(self.ram, POLY_TMP0)
        }

        pub(crate) fn tmp2(&self) -> u8 {
            byte(self.ram, POLY_TMP2)
        }
    }

    pub(crate) struct PolyStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PolyStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_config1(&mut self, value: u8) {
            self.ram[POLY_CONFIG1] = value;
        }

        pub(crate) fn clear_config1(&mut self) {
            self.ram[POLY_CONFIG1] = 0;
        }

        pub(crate) fn increment_config1(&mut self) -> u8 {
            self.ram[POLY_CONFIG1] = self.ram[POLY_CONFIG1].wrapping_add(1);
            self.ram[POLY_CONFIG1]
        }

        pub(crate) fn subtract_config1(&mut self, value: u8) -> u8 {
            self.ram[POLY_CONFIG1] = self.ram[POLY_CONFIG1].wrapping_sub(value);
            self.ram[POLY_CONFIG1]
        }

        pub(crate) fn set_color_mode(&mut self, value: u8) {
            self.ram[POLY_CONFIG_COLOR_MODE] = value;
        }

        pub(crate) fn set_model(&mut self, value: u8) {
            self.ram[POLY_WHICH_MODEL] = value;
        }

        pub(crate) fn set_angle_a(&mut self, value: u8) {
            self.ram[POLY_A] = value;
        }

        pub(crate) fn set_angle_b(&mut self, value: u8) {
            self.ram[POLY_B] = value;
        }

        pub(crate) fn clear_angles(&mut self) {
            self.ram[POLY_A] = 0;
            self.ram[POLY_B] = 0;
        }

        pub(crate) fn add_angle_a(&mut self, value: u8) -> u8 {
            self.ram[POLY_A] = self.ram[POLY_A].wrapping_add(value);
            self.ram[POLY_A]
        }

        pub(crate) fn add_angle_b(&mut self, value: u8) -> u8 {
            self.ram[POLY_B] = self.ram[POLY_B].wrapping_add(value);
            self.ram[POLY_B]
        }

        pub(crate) fn add_angles(&mut self, angle_a: u8, angle_b: u8) {
            self.add_angle_a(angle_a);
            self.add_angle_b(angle_b);
        }

        pub(crate) fn set_base_x(&mut self, value: u8) {
            self.ram[POLY_BASE_X] = value;
        }

        pub(crate) fn set_base_y(&mut self, value: u8) {
            self.ram[POLY_BASE_Y] = value;
        }

        pub(crate) fn set_base_position(&mut self, x: u8, y: u8) {
            self.ram[POLY_BASE_X] = x;
            self.ram[POLY_BASE_Y] = y;
        }

        pub(crate) fn set_shape_depth_bias_low(&mut self, value: u8) {
            self.ram[POLY_SHAPE_DEPTH_BIAS] = value;
        }

        pub(crate) fn set_shape_depth_bias(&mut self, value: u16) {
            write_le_u16(self.ram, POLY_SHAPE_DEPTH_BIAS, value);
        }

        pub(crate) fn set_num_vertices(&mut self, value: u8) {
            self.ram[POLY_CONFIG_NUM_VERTEX] = value;
        }

        pub(crate) fn set_num_polys(&mut self, value: u8) {
            self.ram[POLY_CONFIG_NUM_POLYS] = value;
        }

        pub(crate) fn decrement_num_polys(&mut self) -> u8 {
            self.ram[POLY_CONFIG_NUM_POLYS] = self.ram[POLY_CONFIG_NUM_POLYS].wrapping_sub(1);
            self.ram[POLY_CONFIG_NUM_POLYS]
        }

        pub(crate) fn set_fromlut_position(&mut self, x: u8, y: u8, z: u8) {
            self.ram[POLY_FROMLUT_X] = x;
            self.ram[POLY_FROMLUT_Y] = y;
            self.ram[POLY_FROMLUT_Z] = z;
        }

        pub(crate) fn set_num_vertex_in_poly(&mut self, value: u8) {
            self.ram[POLY_NUM_VERTEX_IN_POLY] = value;
        }

        pub(crate) fn set_raster_color_config(&mut self, value: u8) {
            self.ram[POLY_RASTER_COLOR_CONFIG] = value;
        }

        pub(crate) fn set_tmp0(&mut self, value: u8) {
            self.ram[POLY_TMP0] = value;
        }

        pub(crate) fn decrement_tmp0(&mut self) -> u8 {
            self.ram[POLY_TMP0] = self.ram[POLY_TMP0].wrapping_sub(1);
            self.ram[POLY_TMP0]
        }

        pub(crate) fn set_tmp2(&mut self, value: u8) {
            self.ram[POLY_TMP2] = value;
        }

        pub(crate) fn clear_poly_buffer(&mut self) {
            self.ram[POLYHEDRAL_BUFFER..POLYHEDRAL_BUFFER + 0x800].fill(0);
        }
    }

    pub(crate) struct PolyProjectedVertexView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PolyProjectedVertexView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn x(&self, vertex: usize) -> u8 {
            byte(self.ram, POLY_PROJECTED_X + vertex)
        }

        pub(crate) fn y(&self, vertex: usize) -> u8 {
            byte(self.ram, POLY_PROJECTED_Y + vertex)
        }
    }

    pub(crate) struct PolyProjectedVertexViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PolyProjectedVertexViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_position(&mut self, vertex: usize, x: u8, y: u8) {
            self.ram[POLY_PROJECTED_X + vertex] = x;
            self.ram[POLY_PROJECTED_Y + vertex] = y;
        }
    }

    pub(crate) struct PolyFaceCoordsView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PolyFaceCoordsView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn coord(&self, offset: usize) -> u8 {
            byte(self.ram, POLY_FACE_COORDS + offset)
        }

        pub(crate) fn xy_coords_count(&self) -> u8 {
            byte(self.ram, POLY_FACE_COORDS)
        }
    }

    pub(crate) struct PolyFaceCoordsViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PolyFaceCoordsViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_coord(&mut self, offset: usize, value: u8) {
            self.ram[POLY_FACE_COORDS + offset] = value;
        }

        pub(crate) fn set_xy_coords_count(&mut self, value: u8) {
            self.ram[POLY_FACE_COORDS] = value;
        }
    }

    pub(crate) struct PolyRasterEdgeView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PolyRasterEdgeView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn x0_cur(&self) -> u8 {
            byte(self.ram, POLY_X0_CUR)
        }

        pub(crate) fn y0_cur(&self) -> u8 {
            byte(self.ram, POLY_Y0_CUR)
        }

        pub(crate) fn x1_cur(&self) -> u8 {
            byte(self.ram, POLY_X1_CUR)
        }

        pub(crate) fn y1_cur(&self) -> u8 {
            byte(self.ram, POLY_Y1_CUR)
        }

        pub(crate) fn x0_target(&self) -> u8 {
            byte(self.ram, POLY_X0_TARGET)
        }

        pub(crate) fn y0_trigger(&self) -> u8 {
            byte(self.ram, POLY_Y0_TRIG)
        }

        pub(crate) fn x1_target(&self) -> u8 {
            byte(self.ram, POLY_X1_TARGET)
        }

        pub(crate) fn y1_trigger(&self) -> u8 {
            byte(self.ram, POLY_Y1_TRIG)
        }

        pub(crate) fn total_num_steps(&self) -> u8 {
            byte(self.ram, POLY_TOTAL_NUM_STEPS)
        }

        pub(crate) fn total_num_steps_signed(&self) -> i8 {
            byte(self.ram, POLY_TOTAL_NUM_STEPS) as i8
        }

        pub(crate) fn cur_vertex_idx0(&self) -> u8 {
            byte(self.ram, POLY_CUR_VERTEX_IDX0)
        }

        pub(crate) fn cur_vertex_idx1(&self) -> u8 {
            byte(self.ram, POLY_CUR_VERTEX_IDX1)
        }
    }

    pub(crate) struct PolyRasterEdgeViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PolyRasterEdgeViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_left_current(&mut self, x: u8, y: u8) {
            self.ram[POLY_X0_CUR] = x;
            self.ram[POLY_Y0_CUR] = y;
        }

        pub(crate) fn set_right_current(&mut self, x: u8, y: u8) {
            self.ram[POLY_X1_CUR] = x;
            self.ram[POLY_Y1_CUR] = y;
        }

        pub(crate) fn set_left_target(&mut self, x: u8, y: u8) {
            self.ram[POLY_X0_TARGET] = x;
            self.ram[POLY_Y0_TRIG] = y;
        }

        pub(crate) fn set_right_target(&mut self, x: u8, y: u8) {
            self.ram[POLY_X1_TARGET] = x;
            self.ram[POLY_Y1_TRIG] = y;
        }

        pub(crate) fn set_left_current_x(&mut self, x: u8) {
            self.ram[POLY_X0_CUR] = x;
        }

        pub(crate) fn set_right_current_x(&mut self, x: u8) {
            self.ram[POLY_X1_CUR] = x;
        }

        pub(crate) fn set_total_num_steps(&mut self, value: u8) {
            self.ram[POLY_TOTAL_NUM_STEPS] = value;
        }

        pub(crate) fn decrement_total_num_steps(&mut self) -> i8 {
            self.ram[POLY_TOTAL_NUM_STEPS] = self.ram[POLY_TOTAL_NUM_STEPS].wrapping_sub(1);
            self.ram[POLY_TOTAL_NUM_STEPS] as i8
        }

        pub(crate) fn set_both_cur_vertex_idx(&mut self, value: u8) {
            self.ram[POLY_CUR_VERTEX_IDX0] = value;
            self.ram[POLY_CUR_VERTEX_IDX1] = value;
        }

        pub(crate) fn set_cur_vertex_idx0(&mut self, value: u8) {
            self.ram[POLY_CUR_VERTEX_IDX0] = value;
        }

        pub(crate) fn set_cur_vertex_idx1(&mut self, value: u8) {
            self.ram[POLY_CUR_VERTEX_IDX1] = value;
        }

        pub(crate) fn increment_y0_cur(&mut self) {
            self.ram[POLY_Y0_CUR] = self.ram[POLY_Y0_CUR].wrapping_add(1);
        }

        pub(crate) fn increment_y1_cur(&mut self) {
            self.ram[POLY_Y1_CUR] = self.ram[POLY_Y1_CUR].wrapping_add(1);
        }
    }

    pub(crate) struct IntroActorView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> IntroActorView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn x(&self) -> u16 {
            byte(self.ram, INTRO_X_LO + self.slot) as u16
                | ((byte(self.ram, INTRO_X_HI + self.slot) as u16) << 8)
        }

        pub(crate) fn y(&self) -> u16 {
            byte(self.ram, INTRO_Y_LO + self.slot) as u16
                | ((byte(self.ram, INTRO_Y_HI + self.slot) as u16) << 8)
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, INTRO_X_LO + self.slot)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, INTRO_Y_LO + self.slot)
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            byte(self.ram, INTRO_X_VEL + self.slot)
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            byte(self.ram, INTRO_Y_VEL + self.slot)
        }

        pub(crate) fn init_phase(&self) -> u8 {
            byte(self.ram, INTRO_SPRITE_IS_INITED + self.slot)
        }

        pub(crate) fn subtype(&self) -> u8 {
            byte(self.ram, INTRO_SPRITE_SUBTYPE + self.slot)
        }

        pub(crate) fn state(&self) -> u8 {
            byte(self.ram, INTRO_SPRITE_STATE + self.slot)
        }
    }

    pub(crate) struct IntroActorViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> IntroActorViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_x(&mut self, value: i16) {
            self.ram[INTRO_X_LO + self.slot] = value as u8;
            self.ram[INTRO_X_HI + self.slot] = (value >> 8) as u8;
        }

        pub(crate) fn set_y(&mut self, value: i16) {
            self.ram[INTRO_Y_LO + self.slot] = value as u8;
            self.ram[INTRO_Y_HI + self.slot] = (value >> 8) as u8;
        }

        pub(crate) fn set_x_low(&mut self, value: u8) {
            self.ram[INTRO_X_LO + self.slot] = value;
        }

        pub(crate) fn set_y_low(&mut self, value: u8) {
            self.ram[INTRO_Y_LO + self.slot] = value;
        }

        pub(crate) fn set_x_velocity(&mut self, value: u8) {
            self.ram[INTRO_X_VEL + self.slot] = value;
        }

        pub(crate) fn set_y_velocity(&mut self, value: u8) {
            self.ram[INTRO_Y_VEL + self.slot] = value;
        }

        pub(crate) fn add_x_velocity(&mut self, value: u8) {
            self.ram[INTRO_X_VEL + self.slot] =
                self.ram[INTRO_X_VEL + self.slot].wrapping_add(value);
        }

        pub(crate) fn add_y_velocity(&mut self, value: u8) {
            self.ram[INTRO_Y_VEL + self.slot] =
                self.ram[INTRO_Y_VEL + self.slot].wrapping_add(value);
        }

        pub(crate) fn set_init_phase(&mut self, value: u8) {
            self.ram[INTRO_SPRITE_IS_INITED + self.slot] = value;
        }

        pub(crate) fn increment_init_phase(&mut self) {
            self.ram[INTRO_SPRITE_IS_INITED + self.slot] =
                self.ram[INTRO_SPRITE_IS_INITED + self.slot].wrapping_add(1);
        }

        pub(crate) fn set_subtype(&mut self, value: u8) {
            self.ram[INTRO_SPRITE_SUBTYPE + self.slot] = value;
        }

        pub(crate) fn set_state(&mut self, value: u8) {
            self.ram[INTRO_SPRITE_STATE + self.slot] = value;
        }

        pub(crate) fn increment_state(&mut self) {
            self.ram[INTRO_SPRITE_STATE + self.slot] =
                self.ram[INTRO_SPRITE_STATE + self.slot].wrapping_add(1);
        }

        pub(crate) fn move_x(&mut self) {
            move_axis24(
                self.ram,
                INTRO_X_SUBPIXEL + self.slot,
                INTRO_X_LO + self.slot,
                INTRO_X_HI + self.slot,
                INTRO_X_VEL + self.slot,
            );
        }

        pub(crate) fn move_y(&mut self) {
            move_axis24(
                self.ram,
                INTRO_Y_SUBPIXEL + self.slot,
                INTRO_Y_LO + self.slot,
                INTRO_Y_HI + self.slot,
                INTRO_Y_VEL + self.slot,
            );
        }
    }

    pub(crate) struct EffectAngleScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> EffectAngleScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn angle(&self, slot: usize) -> u8 {
            byte(self.ram, EFFECT_ANGLE_SCRATCH + slot)
        }

        pub(crate) fn trailing_angle(&self) -> u8 {
            byte(self.ram, EFFECT_ANGLE_SCRATCH + 4)
        }

        pub(crate) fn radial_radius(&self) -> u8 {
            byte(self.ram, EFFECT_ANGLE_SCRATCH + 8)
        }
    }

    pub(crate) struct EffectAngleScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> EffectAngleScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_angle(&mut self, slot: usize, value: u8) {
            self.ram[EFFECT_ANGLE_SCRATCH + slot] = value;
        }

        pub(crate) fn set_angles4(&mut self, values: &[u8], start: usize) {
            for slot in 0..4 {
                self.set_angle(slot, values[start + slot]);
            }
        }

        pub(crate) fn add_angle_mod64(&mut self, slot: usize, value: u8) -> u8 {
            let angle = self.ram[EFFECT_ANGLE_SCRATCH + slot].wrapping_add(value) & 0x3f;
            self.ram[EFFECT_ANGLE_SCRATCH + slot] = angle;
            angle
        }

        pub(crate) fn set_trailing_angle(&mut self, value: u8) {
            self.ram[EFFECT_ANGLE_SCRATCH + 4] = value;
        }

        pub(crate) fn add_trailing_angle_mod64(&mut self, value: u8) -> u8 {
            let angle = self.ram[EFFECT_ANGLE_SCRATCH + 4].wrapping_add(value) & 0x3f;
            self.ram[EFFECT_ANGLE_SCRATCH + 4] = angle;
            angle
        }

        pub(crate) fn set_radial_radius(&mut self, value: u8) {
            self.ram[EFFECT_ANGLE_SCRATCH + 8] = value;
        }
    }

    pub(crate) struct QuakeBoltView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> QuakeBoltView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn timer(&self) -> u8 {
            byte(self.ram, QUAKE_BOLT_TIMER + self.slot)
        }

        pub(crate) fn phase(&self) -> u8 {
            byte(self.ram, QUAKE_BOLT_PHASE + self.slot)
        }
    }

    pub(crate) struct QuakeBoltViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> QuakeBoltViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_timer(&mut self, value: u8) {
            self.ram[QUAKE_BOLT_TIMER + self.slot] = value;
        }

        pub(crate) fn tick_timer(&mut self) -> u8 {
            let value = self.ram[QUAKE_BOLT_TIMER + self.slot].wrapping_sub(1);
            self.ram[QUAKE_BOLT_TIMER + self.slot] = value;
            value
        }

        pub(crate) fn set_phase(&mut self, value: u8) {
            self.ram[QUAKE_BOLT_PHASE + self.slot] = value;
        }

        pub(crate) fn advance_phase(&mut self) -> u8 {
            let value = self.ram[QUAKE_BOLT_PHASE + self.slot].wrapping_add(1);
            self.ram[QUAKE_BOLT_PHASE + self.slot] = value;
            value
        }
    }

    pub(crate) struct QuakeSpellScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> QuakeSpellScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn active_bolt_limit(&self) -> u8 {
            byte(self.ram, QUAKE_ACTIVE_BOLT_LIMIT)
        }

        pub(crate) fn pending_step(&self) -> u8 {
            byte(self.ram, QUAKE_PENDING_STEP)
        }

        pub(crate) fn origin_x(&self) -> u16 {
            read_le_u16(self.ram, QUAKE_ORIGIN_X)
        }

        pub(crate) fn origin_y(&self) -> u16 {
            read_le_u16(self.ram, QUAKE_ORIGIN_Y)
        }

        pub(crate) fn screen_shake_y(&self) -> u16 {
            read_le_u16(self.ram, QUAKE_SCREEN_SHAKE_Y)
        }
    }

    pub(crate) struct QuakeSpellScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> QuakeSpellScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_active_bolt_limit(&mut self, value: u8) {
            self.ram[QUAKE_ACTIVE_BOLT_LIMIT] = value;
        }

        pub(crate) fn set_pending_step(&mut self, value: u8) {
            self.ram[QUAKE_PENDING_STEP] = value;
        }

        pub(crate) fn set_origin(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, QUAKE_ORIGIN_X, x);
            write_le_u16(self.ram, QUAKE_ORIGIN_Y, y);
        }

        pub(crate) fn set_screen_shake_y(&mut self, value: u16) {
            write_le_u16(self.ram, QUAKE_SCREEN_SHAKE_Y, value);
        }

        pub(crate) fn invert_screen_shake_y(&mut self) -> u16 {
            let value = read_le_u16(self.ram, QUAKE_SCREEN_SHAKE_Y);
            write_le_u16(self.ram, QUAKE_SCREEN_SHAKE_Y, 0u16.wrapping_sub(value));
            value
        }
    }

    pub(crate) struct BombosFireColumnView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> BombosFireColumnView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn timer(&self) -> u8 {
            byte(self.ram, BOMBOS_FIRE_COLUMN_TIMER + self.slot)
        }

        pub(crate) fn phase(&self) -> u8 {
            byte(self.ram, BOMBOS_FIRE_COLUMN_PHASE + self.slot)
        }

        pub(crate) fn radial_angle(&self) -> u8 {
            byte(self.ram, BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + self.slot)
        }

        pub(crate) fn x(&self) -> u16 {
            byte(self.ram, BOMBOS_FIRE_COLUMN_X_LO + self.slot) as u16
                | ((byte(self.ram, BOMBOS_FIRE_COLUMN_X_HI + self.slot) as u16) << 8)
        }

        pub(crate) fn y(&self) -> u16 {
            byte(self.ram, BOMBOS_FIRE_COLUMN_Y_LO + self.slot) as u16
                | ((byte(self.ram, BOMBOS_FIRE_COLUMN_Y_HI + self.slot) as u16) << 8)
        }
    }

    pub(crate) struct BombosFireColumnViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> BombosFireColumnViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_timer(&mut self, value: u8) {
            self.ram[BOMBOS_FIRE_COLUMN_TIMER + self.slot] = value;
        }

        pub(crate) fn tick_timer(&mut self) -> u8 {
            let value = self.ram[BOMBOS_FIRE_COLUMN_TIMER + self.slot].wrapping_sub(1);
            self.ram[BOMBOS_FIRE_COLUMN_TIMER + self.slot] = value;
            value
        }

        pub(crate) fn set_phase(&mut self, value: u8) {
            self.ram[BOMBOS_FIRE_COLUMN_PHASE + self.slot] = value;
        }

        pub(crate) fn advance_phase(&mut self) -> u8 {
            let value = self.ram[BOMBOS_FIRE_COLUMN_PHASE + self.slot].wrapping_add(1);
            self.ram[BOMBOS_FIRE_COLUMN_PHASE + self.slot] = value;
            value
        }

        pub(crate) fn add_radial_angle(&mut self, value: u8) -> u8 {
            let angle = self.ram[BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + self.slot].wrapping_add(value);
            self.ram[BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + self.slot] = angle;
            angle
        }

        pub(crate) fn set_radial_angle(&mut self, value: u8) {
            self.ram[BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + self.slot] = value;
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            self.ram[BOMBOS_FIRE_COLUMN_X_LO + self.slot] = x as u8;
            self.ram[BOMBOS_FIRE_COLUMN_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[BOMBOS_FIRE_COLUMN_Y_LO + self.slot] = y as u8;
            self.ram[BOMBOS_FIRE_COLUMN_Y_HI + self.slot] = (y >> 8) as u8;
        }
    }

    pub(crate) struct BombosBlastView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> BombosBlastView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn phase(&self) -> u8 {
            byte(self.ram, BOMBOS_BLAST_PHASE + self.slot)
        }
    }

    pub(crate) struct BombosBlastViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> BombosBlastViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_phase(&mut self, value: u8) {
            self.ram[BOMBOS_BLAST_PHASE + self.slot] = value;
        }

        pub(crate) fn advance_phase(&mut self) -> u8 {
            let value = self.ram[BOMBOS_BLAST_PHASE + self.slot].wrapping_add(1);
            self.ram[BOMBOS_BLAST_PHASE + self.slot] = value;
            value
        }

        pub(crate) fn set_timer(&mut self, value: u8) {
            self.ram[BOMBOS_BLAST_TIMER + self.slot] = value;
        }

        pub(crate) fn tick_timer(&mut self) -> u8 {
            let value = self.ram[BOMBOS_BLAST_TIMER + self.slot].wrapping_sub(1);
            self.ram[BOMBOS_BLAST_TIMER + self.slot] = value;
            value
        }
    }

    pub(crate) struct BombosSpellScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> BombosSpellScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn mode(&self) -> u8 {
            byte(self.ram, BOMBOS_MODE)
        }

        pub(crate) fn fire_column_radius(&self) -> u8 {
            byte(self.ram, BOMBOS_FIRE_COLUMN_RADIUS)
        }

        pub(crate) fn blast_release_locked(&self) -> bool {
            byte(self.ram, BOMBOS_BLAST_RELEASE_LOCKED) != 0
        }

        pub(crate) fn fire_column_seed_x(&self, slot: usize) -> u16 {
            read_le_u16(self.ram, BOMBOS_FIRE_COLUMN_SEED_X + slot * 2)
        }

        pub(crate) fn fire_column_seed_y(&self, slot: usize) -> u16 {
            read_le_u16(self.ram, BOMBOS_FIRE_COLUMN_SEED_Y + slot * 2)
        }

        pub(crate) fn blast_x(&self, slot: usize) -> u16 {
            read_le_u16(self.ram, BOMBOS_BLAST_X + slot * 2)
        }

        pub(crate) fn blast_y(&self, slot: usize) -> u16 {
            read_le_u16(self.ram, BOMBOS_BLAST_Y + slot * 2)
        }
    }

    pub(crate) struct BombosSpellScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> BombosSpellScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_mode(&mut self, value: u8) {
            self.ram[BOMBOS_MODE] = value;
        }

        pub(crate) fn set_fire_column_radius(&mut self, value: u8) {
            self.ram[BOMBOS_FIRE_COLUMN_RADIUS] = value;
        }

        pub(crate) fn grow_fire_column_radius(&mut self, value: u8, limit: u8) -> u8 {
            let next = self.ram[BOMBOS_FIRE_COLUMN_RADIUS].wrapping_add(value);
            let radius = if next >= limit { limit } else { next };
            self.ram[BOMBOS_FIRE_COLUMN_RADIUS] = radius;
            radius
        }

        pub(crate) fn set_blast_release_locked(&mut self, value: bool) {
            self.ram[BOMBOS_BLAST_RELEASE_LOCKED] = u8::from(value);
        }

        pub(crate) fn set_blast_release_countdown(&mut self, value: u8) {
            self.ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = value;
        }

        pub(crate) fn tick_blast_release_countdown(&mut self) -> u8 {
            let value = self.ram[BOMBOS_BLAST_RELEASE_COUNTDOWN].wrapping_sub(1);
            self.ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = value;
            value
        }

        pub(crate) fn set_fire_column_seed_position(&mut self, slot: usize, x: u16, y: u16) {
            write_le_u16(self.ram, BOMBOS_FIRE_COLUMN_SEED_X + slot * 2, x);
            write_le_u16(self.ram, BOMBOS_FIRE_COLUMN_SEED_Y + slot * 2, y);
        }

        pub(crate) fn set_blast_position(&mut self, slot: usize, x: u16, y: u16) {
            write_le_u16(self.ram, BOMBOS_BLAST_X + slot * 2, x);
            write_le_u16(self.ram, BOMBOS_BLAST_Y + slot * 2, y);
        }
    }

    pub(crate) struct TowerSealOrbitView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> TowerSealOrbitView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn angle(&self) -> u8 {
            byte(self.ram, TOWER_SEAL_ORBIT_ANGLE + self.slot)
        }
    }

    pub(crate) struct TowerSealOrbitViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> TowerSealOrbitViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_angle(&mut self, value: u8) {
            self.ram[TOWER_SEAL_ORBIT_ANGLE + self.slot] = value;
        }

        pub(crate) fn advance_angle_mod64(&mut self) -> u8 {
            let angle = self.ram[TOWER_SEAL_ORBIT_ANGLE + self.slot].wrapping_add(1) & 0x3f;
            self.ram[TOWER_SEAL_ORBIT_ANGLE + self.slot] = angle;
            angle
        }

        pub(crate) fn set_base_sparkle_position(&mut self, x: u16, y: u16) {
            self.ram[TOWER_SEAL_BASE_SPARKLE_X_LO + self.slot] = x as u8;
            self.ram[TOWER_SEAL_BASE_SPARKLE_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[TOWER_SEAL_BASE_SPARKLE_Y_LO + self.slot] = y as u8;
            self.ram[TOWER_SEAL_BASE_SPARKLE_Y_HI + self.slot] = (y >> 8) as u8;
        }
    }

    pub(crate) struct TowerSealSparkleView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> TowerSealSparkleView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn phase(&self) -> u8 {
            byte(self.ram, TOWER_SEAL_SPARKLE_PHASE + self.slot)
        }

        pub(crate) fn is_free(&self) -> bool {
            self.phase() == 0xff
        }

        pub(crate) fn x(&self) -> u16 {
            byte(self.ram, TOWER_SEAL_SPARKLE_X_LO + self.slot) as u16
                | ((byte(self.ram, TOWER_SEAL_SPARKLE_X_HI + self.slot) as u16) << 8)
        }

        pub(crate) fn y(&self) -> u16 {
            byte(self.ram, TOWER_SEAL_SPARKLE_Y_LO + self.slot) as u16
                | ((byte(self.ram, TOWER_SEAL_SPARKLE_Y_HI + self.slot) as u16) << 8)
        }
    }

    pub(crate) struct TowerSealSparkleViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> TowerSealSparkleViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_phase(&mut self, value: u8) {
            self.ram[TOWER_SEAL_SPARKLE_PHASE + self.slot] = value;
        }

        pub(crate) fn set_timer(&mut self, value: u8) {
            self.ram[TOWER_SEAL_SPARKLE_TIMER + self.slot] = value;
        }

        pub(crate) fn tick_timer(&mut self) -> u8 {
            let timer = self.ram[TOWER_SEAL_SPARKLE_TIMER + self.slot].wrapping_sub(1);
            self.ram[TOWER_SEAL_SPARKLE_TIMER + self.slot] = timer;
            timer
        }

        pub(crate) fn advance_phase(&mut self) -> u8 {
            let phase = self.ram[TOWER_SEAL_SPARKLE_PHASE + self.slot].wrapping_add(1);
            self.ram[TOWER_SEAL_SPARKLE_PHASE + self.slot] = phase;
            phase
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            self.ram[TOWER_SEAL_SPARKLE_X_LO + self.slot] = x as u8;
            self.ram[TOWER_SEAL_SPARKLE_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[TOWER_SEAL_SPARKLE_Y_LO + self.slot] = y as u8;
            self.ram[TOWER_SEAL_SPARKLE_Y_HI + self.slot] = (y >> 8) as u8;
        }

        pub(crate) fn base_sparkle_position(&self, base: usize) -> (u16, u16) {
            let x = self.ram[TOWER_SEAL_BASE_SPARKLE_X_LO + base] as u16
                | ((self.ram[TOWER_SEAL_BASE_SPARKLE_X_HI + base] as u16) << 8);
            let y = self.ram[TOWER_SEAL_BASE_SPARKLE_Y_LO + base] as u16
                | ((self.ram[TOWER_SEAL_BASE_SPARKLE_Y_HI + base] as u16) << 8);
            (x, y)
        }
    }

    pub(crate) struct TowerSealScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> TowerSealScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn ring_radius(&self) -> u8 {
            byte(self.ram, TOWER_SEAL_RING_RADIUS)
        }

        pub(crate) fn center_x(&self) -> u16 {
            read_le_u16(self.ram, TOWER_SEAL_CENTER_X)
        }

        pub(crate) fn center_y(&self) -> u16 {
            read_le_u16(self.ram, TOWER_SEAL_CENTER_Y)
        }
    }

    pub(crate) struct TowerSealScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> TowerSealScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_ring_radius(&mut self, value: u8) {
            self.ram[TOWER_SEAL_RING_RADIUS] = value;
        }

        pub(crate) fn set_center(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, TOWER_SEAL_CENTER_X, x);
            write_le_u16(self.ram, TOWER_SEAL_CENTER_Y, y);
        }

        pub(crate) fn tick_wait_countdown(&mut self) -> u8 {
            let value = self.ram[TOWER_SEAL_WAIT_COUNTDOWN].wrapping_sub(1);
            self.ram[TOWER_SEAL_WAIT_COUNTDOWN] = value;
            value
        }

        pub(crate) fn set_wait_countdown(&mut self, value: u8) {
            self.ram[TOWER_SEAL_WAIT_COUNTDOWN] = value;
        }
    }

    pub(crate) struct BlastWallExplosionView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> BlastWallExplosionView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn phase(&self) -> u8 {
            byte(self.ram, BLAST_WALL_EXPLOSION_PHASE + self.slot)
        }

        pub(crate) fn timer(&self) -> u8 {
            byte(self.ram, BLAST_WALL_EXPLOSION_TIMER + self.slot)
        }
    }

    pub(crate) struct BlastWallExplosionViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> BlastWallExplosionViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_phase(&mut self, value: u8) {
            self.ram[BLAST_WALL_EXPLOSION_PHASE + self.slot] = value;
        }

        pub(crate) fn advance_phase(&mut self) -> u8 {
            let phase = self.ram[BLAST_WALL_EXPLOSION_PHASE + self.slot].wrapping_add(1);
            self.ram[BLAST_WALL_EXPLOSION_PHASE + self.slot] = phase;
            phase
        }

        pub(crate) fn set_timer(&mut self, value: u8) {
            self.ram[BLAST_WALL_EXPLOSION_TIMER + self.slot] = value;
        }

        pub(crate) fn tick_timer(&mut self) -> u8 {
            let timer = self.ram[BLAST_WALL_EXPLOSION_TIMER + self.slot].wrapping_sub(1);
            self.ram[BLAST_WALL_EXPLOSION_TIMER + self.slot] = timer;
            timer
        }
    }

    pub(crate) struct BlastWallFragmentView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> BlastWallFragmentView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn y(&self) -> u16 {
            read_le_u16(self.ram, BLAST_WALL_FRAGMENT_Y + self.slot * 2)
        }

        pub(crate) fn x(&self) -> u16 {
            read_le_u16(self.ram, BLAST_WALL_FRAGMENT_X + self.slot * 2)
        }
    }

    pub(crate) struct BlastWallFragmentViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> BlastWallFragmentViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, BLAST_WALL_FRAGMENT_X + self.slot * 2, x);
            write_le_u16(self.ram, BLAST_WALL_FRAGMENT_Y + self.slot * 2, y);
        }

        pub(crate) fn offset(&mut self, x_delta: i16, y_delta: i16) -> (u16, u16) {
            let x = read_le_u16(self.ram, BLAST_WALL_FRAGMENT_X + self.slot * 2)
                .wrapping_add(x_delta as u16);
            let y = read_le_u16(self.ram, BLAST_WALL_FRAGMENT_Y + self.slot * 2)
                .wrapping_add(y_delta as u16);
            self.set_position(x, y);
            (x, y)
        }
    }

    pub(crate) struct BlastWallFireballView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> BlastWallFireballView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn timer(&self) -> u8 {
            byte(self.ram, BLAST_WALL_FIREBALL_TIMER + self.slot)
        }
    }

    pub(crate) struct BlastWallFireballViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> BlastWallFireballViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_timer(&mut self, value: u8) {
            self.ram[BLAST_WALL_FIREBALL_TIMER + self.slot] = value;
        }

        pub(crate) fn tick_timer(&mut self) -> u8 {
            let timer = self.ram[BLAST_WALL_FIREBALL_TIMER + self.slot].wrapping_sub(1);
            self.ram[BLAST_WALL_FIREBALL_TIMER + self.slot] = timer;
            timer
        }
    }

    pub(crate) struct BlastWallScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> BlastWallScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn direction(&self) -> u8 {
            byte(self.ram, BLAST_WALL_DIRECTION)
        }

        pub(crate) fn center_x(&self) -> u16 {
            read_le_u16(self.ram, BLAST_WALL_CENTER_X)
        }

        pub(crate) fn center_y(&self) -> u16 {
            read_le_u16(self.ram, BLAST_WALL_CENTER_Y)
        }
    }

    pub(crate) struct BlastWallScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> BlastWallScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear_entry_state(&mut self) {
            self.ram[BLAST_WALL_ENTRY_STATE] = 0;
        }

        pub(crate) fn clear_secondary_state(&mut self) {
            self.ram[BLAST_WALL_SECONDARY_STATE] = 0;
        }

        pub(crate) fn offset_center(&mut self, x_delta: i8, y_delta: i8) -> (u16, u16) {
            let y = read_le_u16(self.ram, BLAST_WALL_CENTER_Y).wrapping_add(y_delta as i16 as u16);
            let x = read_le_u16(self.ram, BLAST_WALL_CENTER_X).wrapping_add(x_delta as i16 as u16);
            write_le_u16(self.ram, BLAST_WALL_CENTER_Y, y);
            write_le_u16(self.ram, BLAST_WALL_CENTER_X, x);
            (x, y)
        }
    }

    pub(crate) struct SkullWoodsFireView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> SkullWoodsFireView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn phase(&self) -> u8 {
            byte(self.ram, SKULL_WOODS_FIRE_PHASE + self.slot)
        }

        pub(crate) fn is_finished(&self) -> bool {
            (self.phase() as i8).is_negative()
        }

        pub(crate) fn x(&self) -> u16 {
            read_le_u16(self.ram, SKULL_WOODS_FIRE_X + self.slot * 2)
        }

        pub(crate) fn y(&self) -> u16 {
            read_le_u16(self.ram, SKULL_WOODS_FIRE_Y + self.slot * 2)
        }
    }

    pub(crate) struct SkullWoodsFireViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> SkullWoodsFireViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_phase(&mut self, value: u8) {
            self.ram[SKULL_WOODS_FIRE_PHASE + self.slot] = value;
        }

        pub(crate) fn advance_phase(&mut self) -> u8 {
            let phase = self.ram[SKULL_WOODS_FIRE_PHASE + self.slot].wrapping_add(1);
            self.ram[SKULL_WOODS_FIRE_PHASE + self.slot] = phase;
            phase
        }

        pub(crate) fn set_timer(&mut self, value: u8) {
            self.ram[SKULL_WOODS_FIRE_TIMER + self.slot] = value;
        }

        pub(crate) fn tick_timer(&mut self) -> u8 {
            let timer = self.ram[SKULL_WOODS_FIRE_TIMER + self.slot].wrapping_sub(1);
            self.ram[SKULL_WOODS_FIRE_TIMER + self.slot] = timer;
            timer
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, SKULL_WOODS_FIRE_X + self.slot * 2, x);
            write_le_u16(self.ram, SKULL_WOODS_FIRE_Y + self.slot * 2, y);
        }
    }

    pub(crate) struct SkullWoodsFireScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SkullWoodsFireScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn has_started_entrance_opening(&self) -> bool {
            byte(self.ram, SKULL_WOODS_FIRE_STARTED) != 0
        }

        pub(crate) fn inner_x(&self) -> u16 {
            read_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_X)
        }

        pub(crate) fn inner_y(&self) -> u16 {
            read_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_Y)
        }
    }

    pub(crate) struct SkullWoodsFireScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SkullWoodsFireScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear_entrance_opening_started(&mut self) {
            self.ram[SKULL_WOODS_FIRE_STARTED] = 0;
        }

        pub(crate) fn set_entrance_opening_started(&mut self) {
            self.ram[SKULL_WOODS_FIRE_STARTED] = 1;
        }

        pub(crate) fn set_inner_position(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_X, x);
            write_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_Y, y);
        }

        pub(crate) fn set_outer_position(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, SKULL_WOODS_FIRE_OUTER_X, x);
            write_le_u16(self.ram, SKULL_WOODS_FIRE_OUTER_Y, y);
        }

        pub(crate) fn retreat_inner_y(&mut self, value: u16) -> u16 {
            let y = read_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_Y).wrapping_sub(value);
            write_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_Y, y);
            y
        }
    }

    pub(crate) struct HappinessPondRupeeState {
        pub(crate) y_low: u8,
        pub(crate) y_high: u8,
        pub(crate) x_low: u8,
        pub(crate) x_high: u8,
        pub(crate) z: u8,
        pub(crate) y_velocity: u8,
        pub(crate) x_velocity: u8,
        pub(crate) z_velocity: u8,
        pub(crate) y_subpixel: u8,
        pub(crate) x_subpixel: u8,
        pub(crate) z_subpixel: u8,
        pub(crate) item_to_link: u8,
        pub(crate) timer: u8,
        pub(crate) step: u8,
    }

    pub(crate) struct HappinessPondRupeeView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> HappinessPondRupeeView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn is_active(&self) -> bool {
            byte(self.ram, HAPPINESS_POND_ACTIVE + self.slot) != 0
        }

        pub(crate) fn step(&self) -> u8 {
            byte(self.ram, HAPPINESS_POND_STEP + self.slot)
        }

        pub(crate) fn snapshot(&self) -> HappinessPondRupeeState {
            HappinessPondRupeeState {
                y_low: byte(self.ram, HAPPINESS_POND_Y_LO + self.slot),
                y_high: byte(self.ram, HAPPINESS_POND_Y_HI + self.slot),
                x_low: byte(self.ram, HAPPINESS_POND_X_LO + self.slot),
                x_high: byte(self.ram, HAPPINESS_POND_X_HI + self.slot),
                z: byte(self.ram, HAPPINESS_POND_Z + self.slot),
                y_velocity: byte(self.ram, HAPPINESS_POND_Y_VEL + self.slot),
                x_velocity: byte(self.ram, HAPPINESS_POND_X_VEL + self.slot),
                z_velocity: byte(self.ram, HAPPINESS_POND_Z_VEL + self.slot),
                y_subpixel: byte(self.ram, HAPPINESS_POND_Y_SUBPIXEL + self.slot),
                x_subpixel: byte(self.ram, HAPPINESS_POND_X_SUBPIXEL + self.slot),
                z_subpixel: byte(self.ram, HAPPINESS_POND_Z_SUBPIXEL + self.slot),
                item_to_link: byte(self.ram, HAPPINESS_POND_ITEM_TO_LINK + self.slot),
                timer: byte(self.ram, HAPPINESS_POND_TIMER + self.slot).saturating_sub(1),
                step: byte(self.ram, HAPPINESS_POND_STEP + self.slot),
            }
        }
    }

    pub(crate) struct HappinessPondRupeeViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> HappinessPondRupeeViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_active(&mut self, active: bool) {
            self.ram[HAPPINESS_POND_ACTIVE + self.slot] = u8::from(active);
        }

        pub(crate) fn clear(&mut self) {
            self.ram[HAPPINESS_POND_ACTIVE + self.slot] = 0;
        }

        pub(crate) fn initialize(
            &mut self,
            x: u16,
            y: u16,
            x_velocity: u8,
            y_velocity: u8,
            z_velocity: u8,
        ) {
            self.set_active(true);
            self.ram[HAPPINESS_POND_Z_VEL + self.slot] = z_velocity;
            self.ram[HAPPINESS_POND_Y_VEL + self.slot] = y_velocity;
            self.ram[HAPPINESS_POND_X_VEL + self.slot] = x_velocity;
            self.ram[HAPPINESS_POND_Z + self.slot] = 0;
            self.ram[HAPPINESS_POND_STEP + self.slot] = 0;
            self.ram[HAPPINESS_POND_TIMER + self.slot] = 16;
            self.ram[HAPPINESS_POND_ITEM_TO_LINK + self.slot] = 53;
            self.ram[HAPPINESS_POND_X_LO + self.slot] = x as u8;
            self.ram[HAPPINESS_POND_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[HAPPINESS_POND_Y_LO + self.slot] = y as u8;
            self.ram[HAPPINESS_POND_Y_HI + self.slot] = (y >> 8) as u8;
        }

        pub(crate) fn store_snapshot(&mut self, state: HappinessPondRupeeState) {
            self.ram[HAPPINESS_POND_Y_LO + self.slot] = state.y_low;
            self.ram[HAPPINESS_POND_Y_HI + self.slot] = state.y_high;
            self.ram[HAPPINESS_POND_X_LO + self.slot] = state.x_low;
            self.ram[HAPPINESS_POND_X_HI + self.slot] = state.x_high;
            self.ram[HAPPINESS_POND_Z + self.slot] = state.z;
            self.ram[HAPPINESS_POND_Y_VEL + self.slot] = state.y_velocity;
            self.ram[HAPPINESS_POND_X_VEL + self.slot] = state.x_velocity;
            self.ram[HAPPINESS_POND_Z_VEL + self.slot] = state.z_velocity;
            self.ram[HAPPINESS_POND_Y_SUBPIXEL + self.slot] = state.y_subpixel;
            self.ram[HAPPINESS_POND_X_SUBPIXEL + self.slot] = state.x_subpixel;
            self.ram[HAPPINESS_POND_Z_SUBPIXEL + self.slot] = state.z_subpixel;
            self.ram[HAPPINESS_POND_ITEM_TO_LINK + self.slot] = state.item_to_link;
            self.ram[HAPPINESS_POND_TIMER + self.slot] = state.timer;
            self.ram[HAPPINESS_POND_STEP + self.slot] = state.step;
        }
    }

    pub(crate) struct WeatherVaneStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> WeatherVaneStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn oam_offset(&self) -> u8 {
            byte(self.ram, WEATHERVANE_OAM_OFFSET)
        }
    }

    pub(crate) struct WeatherVaneStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> WeatherVaneStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_countdown(&mut self, value: u16) {
            write_le_u16(self.ram, WEATHERVANE_COUNTDOWN, value);
        }

        pub(crate) fn tick_countdown(&mut self) -> u16 {
            let value = read_le_u16(self.ram, WEATHERVANE_COUNTDOWN).wrapping_sub(1);
            write_le_u16(self.ram, WEATHERVANE_COUNTDOWN, value);
            value
        }

        pub(crate) fn set_music_latch(&mut self, value: u8) {
            self.ram[WEATHERVANE_MUSIC_LATCH] = value;
        }

        pub(crate) fn music_latch(&self) -> u8 {
            byte(self.ram, WEATHERVANE_MUSIC_LATCH)
        }

        pub(crate) fn set_source_slot(&mut self, slot: u8) {
            self.ram[WEATHERVANE_SOURCE_SLOT] = slot;
        }

        pub(crate) fn reset_oam_offset(&mut self) {
            self.ram[WEATHERVANE_OAM_OFFSET] = 0;
        }

        pub(crate) fn advance_oam_offset(&mut self, value: u8) {
            self.ram[WEATHERVANE_OAM_OFFSET] = self.ram[WEATHERVANE_OAM_OFFSET].wrapping_add(value);
        }
    }

    pub(crate) struct WeatherVaneDebrisState {
        pub(crate) y: u16,
        pub(crate) x: u16,
        pub(crate) z: u8,
        pub(crate) y_velocity: u8,
        pub(crate) x_velocity: u8,
        pub(crate) z_velocity: u8,
        pub(crate) draw_state: u8,
    }

    pub(crate) struct WeatherVaneDebrisView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> WeatherVaneDebrisView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn is_finished(&self) -> bool {
            byte(self.ram, WEATHERVANE_DRAW_STATE + self.slot) == 0xff
        }

        pub(crate) fn snapshot(&self) -> WeatherVaneDebrisState {
            WeatherVaneDebrisState {
                y: packed_position(
                    self.ram,
                    WEATHERVANE_Y_LO + self.slot,
                    WEATHERVANE_Y_HI + self.slot,
                ),
                x: packed_position(
                    self.ram,
                    WEATHERVANE_X_LO + self.slot,
                    WEATHERVANE_X_HI + self.slot,
                ),
                z: byte(self.ram, WEATHERVANE_Z + self.slot),
                y_velocity: byte(self.ram, WEATHERVANE_Y_VELOCITY + self.slot),
                x_velocity: byte(self.ram, WEATHERVANE_X_VELOCITY + self.slot),
                z_velocity: byte(self.ram, WEATHERVANE_Z_VELOCITY + self.slot),
                draw_state: byte(self.ram, WEATHERVANE_DRAW_STATE + self.slot),
            }
        }
    }

    pub(crate) struct WeatherVaneDebrisViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> WeatherVaneDebrisViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn initialize(
            &mut self,
            x: u16,
            y: u16,
            x_velocity: u8,
            y_velocity: u8,
            z_velocity: u8,
            z: u8,
            draw_state: u8,
        ) {
            self.ram[WEATHERVANE_Y_VELOCITY + self.slot] = y_velocity;
            self.ram[WEATHERVANE_X_VELOCITY + self.slot] = x_velocity;
            self.ram[WEATHERVANE_Z_VELOCITY + self.slot] = z_velocity;
            self.ram[WEATHERVANE_Y_LO + self.slot] = y as u8;
            self.ram[WEATHERVANE_Y_HI + self.slot] = (y >> 8) as u8;
            self.ram[WEATHERVANE_X_LO + self.slot] = x as u8;
            self.ram[WEATHERVANE_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[WEATHERVANE_Z + self.slot] = z;
            self.ram[WEATHERVANE_ANIM_TIMER + self.slot] = 1;
            self.ram[WEATHERVANE_DRAW_STATE + self.slot] = draw_state;
        }

        pub(crate) fn tick_animation(&mut self) -> u8 {
            let timer = self.ram[WEATHERVANE_ANIM_TIMER + self.slot].wrapping_sub(1);
            self.ram[WEATHERVANE_ANIM_TIMER + self.slot] = timer;
            if (timer as i8).is_negative() {
                self.ram[WEATHERVANE_ANIM_TIMER + self.slot] = 1;
                self.ram[WEATHERVANE_DRAW_STATE + self.slot] ^= 1;
            }
            self.ram[WEATHERVANE_DRAW_STATE + self.slot]
        }

        pub(crate) fn tick_z_velocity(&mut self) -> u8 {
            let z_velocity = self.ram[WEATHERVANE_Z_VELOCITY + self.slot].wrapping_sub(1);
            self.ram[WEATHERVANE_Z_VELOCITY + self.slot] = z_velocity;
            z_velocity
        }

        pub(crate) fn load_into_ancilla(&self, ancilla: &mut AncillaSlotViewMut<'_>) {
            let y = packed_position(
                self.ram,
                WEATHERVANE_Y_LO + self.slot,
                WEATHERVANE_Y_HI + self.slot,
            );
            let x = packed_position(
                self.ram,
                WEATHERVANE_X_LO + self.slot,
                WEATHERVANE_X_HI + self.slot,
            );
            ancilla.set_item_to_link(byte(self.ram, WEATHERVANE_DRAW_STATE + self.slot));
            ancilla.set_y(y);
            ancilla.set_x(x);
            ancilla.set_z(byte(self.ram, WEATHERVANE_Z + self.slot));
            ancilla.set_y_velocity(byte(self.ram, WEATHERVANE_Y_VELOCITY + self.slot));
            ancilla.set_x_velocity(byte(self.ram, WEATHERVANE_X_VELOCITY + self.slot));
            ancilla.set_z_velocity(byte(self.ram, WEATHERVANE_Z_VELOCITY + self.slot));
        }

        pub(crate) fn mark_finished_if_landed(&mut self, z: u8) {
            if z >= 0xf0 {
                self.ram[WEATHERVANE_DRAW_STATE + self.slot] = 0xff;
            }
        }

        pub(crate) fn save_position(&mut self, x: u16, y: u16, z: u8) {
            self.ram[WEATHERVANE_Y_LO + self.slot] = y as u8;
            self.ram[WEATHERVANE_Y_HI + self.slot] = (y >> 8) as u8;
            self.ram[WEATHERVANE_X_LO + self.slot] = x as u8;
            self.ram[WEATHERVANE_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[WEATHERVANE_Z + self.slot] = z;
        }
    }

    pub(crate) struct BirdTravelDestinationView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> BirdTravelDestinationView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(
                self.ram,
                BIRD_TRAVEL_X_LO + self.slot,
                BIRD_TRAVEL_X_HI + self.slot,
            )
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(
                self.ram,
                BIRD_TRAVEL_Y_LO + self.slot,
                BIRD_TRAVEL_Y_HI + self.slot,
            )
        }

        pub(crate) fn is_empty(&self) -> bool {
            self.ram[BIRD_TRAVEL_X_LO + self.slot]
                | self.ram[BIRD_TRAVEL_X_HI + self.slot]
                | self.ram[BIRD_TRAVEL_Y_LO + self.slot]
                | self.ram[BIRD_TRAVEL_Y_HI + self.slot]
                == 0
        }
    }

    pub(crate) struct BirdTravelDestinationViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> BirdTravelDestinationViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            self.ram[BIRD_TRAVEL_X_LO + self.slot] = x as u8;
            self.ram[BIRD_TRAVEL_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[BIRD_TRAVEL_Y_LO + self.slot] = y as u8;
            self.ram[BIRD_TRAVEL_Y_HI + self.slot] = (y >> 8) as u8;
        }

        pub(crate) fn clear(&mut self) {
            self.set_position(0, 0);
        }
    }

    pub(crate) struct BirdTravelStatusViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> BirdTravelStatusViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear(&mut self, slot: usize) {
            self.ram[BIRD_TRAVEL_STATUS + slot] = 0;
        }

        pub(crate) fn increment(&mut self, slot: usize) {
            self.ram[BIRD_TRAVEL_STATUS + slot] =
                self.ram[BIRD_TRAVEL_STATUS + slot].wrapping_add(1);
        }
    }

    pub(crate) struct MoldormHistoryView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> MoldormHistoryView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(
                self.ram,
                MOLDORM_HISTORY_X_LO + self.slot,
                MOLDORM_HISTORY_X_HI + self.slot,
            )
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(
                self.ram,
                MOLDORM_HISTORY_Y_LO + self.slot,
                MOLDORM_HISTORY_Y_HI + self.slot,
            )
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, MOLDORM_HISTORY_X_LO + self.slot)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, MOLDORM_HISTORY_Y_LO + self.slot)
        }
    }

    pub(crate) struct MoldormHistoryViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> MoldormHistoryViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            self.ram[MOLDORM_HISTORY_X_LO + self.slot] = x as u8;
            self.ram[MOLDORM_HISTORY_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[MOLDORM_HISTORY_Y_LO + self.slot] = y as u8;
            self.ram[MOLDORM_HISTORY_Y_HI + self.slot] = (y >> 8) as u8;
        }

        pub(crate) fn set_low_position(&mut self, x_low: u8, y_low: u8) {
            self.ram[MOLDORM_HISTORY_X_LO + self.slot] = x_low;
            self.ram[MOLDORM_HISTORY_Y_LO + self.slot] = y_low;
        }
    }

    pub(crate) struct SwamolaTargetView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> SwamolaTargetView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(
                self.ram,
                SWAMOLA_TARGET_X_LO + self.slot,
                SWAMOLA_TARGET_X_HI + self.slot,
            )
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(
                self.ram,
                SWAMOLA_TARGET_Y_LO + self.slot,
                SWAMOLA_TARGET_Y_HI + self.slot,
            )
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, SWAMOLA_TARGET_X_LO + self.slot)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, SWAMOLA_TARGET_Y_LO + self.slot)
        }
    }

    pub(crate) struct SwamolaTargetViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> SwamolaTargetViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            self.ram[SWAMOLA_TARGET_X_LO + self.slot] = x as u8;
            self.ram[SWAMOLA_TARGET_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[SWAMOLA_TARGET_Y_LO + self.slot] = y as u8;
            self.ram[SWAMOLA_TARGET_Y_HI + self.slot] = (y >> 8) as u8;
        }

        pub(crate) fn set_x_low(&mut self, value: u8) {
            self.ram[SWAMOLA_TARGET_X_LO + self.slot] = value;
        }

        pub(crate) fn set_y_low(&mut self, value: u8) {
            self.ram[SWAMOLA_TARGET_Y_LO + self.slot] = value;
        }
    }

    pub(crate) struct SwamolaHistoryView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> SwamolaHistoryView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(
                self.ram,
                SWAMOLA_HISTORY_X_LO + self.slot,
                SWAMOLA_HISTORY_X_HI + self.slot,
            )
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(
                self.ram,
                SWAMOLA_HISTORY_Y_LO + self.slot,
                SWAMOLA_HISTORY_Y_HI + self.slot,
            )
        }
    }

    pub(crate) struct SwamolaHistoryViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> SwamolaHistoryViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            self.ram[SWAMOLA_HISTORY_X_LO + self.slot] = x as u8;
            self.ram[SWAMOLA_HISTORY_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[SWAMOLA_HISTORY_Y_LO + self.slot] = y as u8;
            self.ram[SWAMOLA_HISTORY_Y_HI + self.slot] = (y >> 8) as u8;
        }
    }

    pub(crate) struct BeamosLaserHistoryView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> BeamosLaserHistoryView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(
                self.ram,
                BEAMOS_LASER_HISTORY_X_LO + self.slot,
                BEAMOS_LASER_HISTORY_X_HI + self.slot,
            )
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(
                self.ram,
                BEAMOS_LASER_HISTORY_Y_LO + self.slot,
                BEAMOS_LASER_HISTORY_Y_HI + self.slot,
            )
        }
    }

    pub(crate) struct BeamosLaserHistoryViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> BeamosLaserHistoryViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            self.ram[BEAMOS_LASER_HISTORY_X_LO + self.slot] = x as u8;
            self.ram[BEAMOS_LASER_HISTORY_X_HI + self.slot] = (x >> 8) as u8;
            self.ram[BEAMOS_LASER_HISTORY_Y_LO + self.slot] = y as u8;
            self.ram[BEAMOS_LASER_HISTORY_Y_HI + self.slot] = (y >> 8) as u8;
        }
    }

    pub(crate) struct LanmolaSegmentMotionView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> LanmolaSegmentMotionView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn z_offset(&self) -> u8 {
            byte(self.ram, BEAMOS_LASER_HISTORY_X_HI + self.slot)
        }

        pub(crate) fn direction(&self) -> u8 {
            byte(self.ram, BEAMOS_LASER_HISTORY_Y_HI + self.slot)
        }
    }

    pub(crate) struct LanmolaSegmentMotionViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> LanmolaSegmentMotionViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_z_offset(&mut self, value: u8) {
            self.ram[BEAMOS_LASER_HISTORY_X_HI + self.slot] = value;
        }

        pub(crate) fn set_direction(&mut self, value: u8) {
            self.ram[BEAMOS_LASER_HISTORY_Y_HI + self.slot] = value;
        }
    }

    pub(crate) struct DoorDebrisView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DoorDebrisView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn x(&self, slot: usize) -> u8 {
            byte(self.ram, DOOR_DEBRIS_X + slot)
        }

        pub(crate) fn y(&self, slot: usize) -> u8 {
            byte(self.ram, DOOR_DEBRIS_Y + slot)
        }

        pub(crate) fn direction(&self, slot: usize) -> u8 {
            byte(self.ram, DOOR_DEBRIS_DIRECTION + slot)
        }

        pub(crate) fn x_word(&self, slot: usize) -> u16 {
            word(self.ram, DOOR_DEBRIS_X + slot * 2)
        }

        pub(crate) fn y_word(&self, slot: usize) -> u16 {
            word(self.ram, DOOR_DEBRIS_Y + slot * 2)
        }
    }

    pub(crate) struct DoorDebrisViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DoorDebrisViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_direction(&mut self, slot: usize, value: u8) {
            self.ram[DOOR_DEBRIS_DIRECTION + slot] = value;
        }

        pub(crate) fn set_y_low_and_x_low_from_word(&mut self, slot: usize, value: u16) {
            self.ram[DOOR_DEBRIS_Y + slot] = value as u8;
            self.ram[DOOR_DEBRIS_X + slot] = (value >> 8) as u8;
        }
    }

    pub(crate) struct DiggingGamePrizeView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DiggingGamePrizeView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn attempts(&self) -> u8 {
            byte(self.ram, DIGGING_GAME_PRIZE_ATTEMPTS)
        }

        pub(crate) fn spawned_marker(&self) -> u8 {
            byte(self.ram, DIGGING_GAME_PRIZE_SPAWNED)
        }
    }

    pub(crate) struct DiggingGamePrizeViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DiggingGamePrizeViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn increment_attempts(&mut self) {
            self.ram[DIGGING_GAME_PRIZE_ATTEMPTS] =
                self.ram[DIGGING_GAME_PRIZE_ATTEMPTS].wrapping_add(1);
        }

        pub(crate) fn mark_spawned(&mut self) {
            self.ram[DIGGING_GAME_PRIZE_SPAWNED] = 0xeb;
        }

        pub(crate) fn clear_prize_spawned(&mut self) {
            self.ram[DIGGING_GAME_PRIZE_SPAWNED] = 0;
        }
    }

    pub(crate) struct DrawScratchPositionView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DrawScratchPositionView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, DRAW_SCRATCH_POSITION_X)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, DRAW_SCRATCH_POSITION_Y)
        }

        pub(crate) fn low_position_word(&self) -> u16 {
            word(self.ram, DRAW_SCRATCH_POSITION_X)
        }
    }

    pub(crate) struct DrawScratchPositionViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DrawScratchPositionViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_low_position(&mut self, x: u8, y: u8) {
            self.ram[DRAW_SCRATCH_POSITION_X] = x;
            self.ram[DRAW_SCRATCH_POSITION_Y] = y;
        }

        pub(crate) fn set_low_position_word(&mut self, value: u16) {
            write_le_u16(self.ram, DRAW_SCRATCH_POSITION_X, value);
        }

        pub(crate) fn set_word_bytes(&mut self, low: u8, high: u8) {
            self.ram[DRAW_SCRATCH_POSITION_X] = low;
            self.ram[DRAW_SCRATCH_POSITION_Y] = high;
        }

        pub(crate) fn offset_low_position(&mut self, dx: u8, dy: u8) -> (u8, u8) {
            let x = self.ram[DRAW_SCRATCH_POSITION_X].wrapping_add(dx);
            let y = self.ram[DRAW_SCRATCH_POSITION_Y].wrapping_add(dy);
            self.set_low_position(x, y);
            (x, y)
        }

        pub(crate) fn set_flags_high(&mut self, value: u8) {
            self.ram[DRAW_SCRATCH_FLAGS_HI] = value;
        }
    }

    pub(crate) struct HitboxScratchOffsetView<'a> {
        ram: &'a [u8],
    }

    impl<'a> HitboxScratchOffsetView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn x_high_offset(&self) -> u8 {
            byte(self.ram, HITBOX_SCRATCH_X_OFFSET)
        }

        pub(crate) fn y_low_offset(&self) -> u8 {
            byte(self.ram, HITBOX_SCRATCH_Y_OFFSET)
        }
    }

    pub(crate) struct HitboxScratchOffsetViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> HitboxScratchOffsetViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_x_high_offset(&mut self, value: u8) {
            self.ram[HITBOX_SCRATCH_X_OFFSET] = value;
        }

        pub(crate) fn set_y_low_offset(&mut self, value: u8) {
            self.ram[HITBOX_SCRATCH_Y_OFFSET] = value;
        }

        pub(crate) fn set_offsets(&mut self, y_low: u8, x_high: u8) {
            self.ram[HITBOX_SCRATCH_Y_OFFSET] = y_low;
            self.ram[HITBOX_SCRATCH_X_OFFSET] = x_high;
        }
    }

    pub(crate) struct OverworldScrollDeltaView<'a> {
        ram: &'a [u8],
    }

    impl<'a> OverworldScrollDeltaView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn low(&self) -> u8 {
            byte(self.ram, OVERWORLD_SCROLL_DELTA)
        }

        pub(crate) fn high(&self) -> u8 {
            byte(self.ram, OVERWORLD_SCROLL_DELTA + 1)
        }

        pub(crate) fn word(&self) -> u16 {
            word(self.ram, OVERWORLD_SCROLL_DELTA)
        }
    }

    pub(crate) struct OverworldScrollDeltaViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> OverworldScrollDeltaViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_high(&mut self, value: u8) {
            self.ram[OVERWORLD_SCROLL_DELTA + 1] = value;
        }

        pub(crate) fn set_high_word(&mut self, value: u16) {
            write_le_u16(self.ram, OVERWORLD_SCROLL_DELTA + 1, value);
        }

        pub(crate) fn set_y_delta(&mut self, value: u16) {
            write_le_u16(self.ram, OVERWORLD_SCROLL_DELTA, value);
        }

        pub(crate) fn clear_low(&mut self) {
            self.ram[OVERWORLD_SCROLL_DELTA] = 0;
        }
    }

    pub(crate) struct PpuScrollCopyView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PpuScrollCopyView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn bg2_h_copy2_offset() -> usize {
            BG2_X_SCROLL
        }

        pub(crate) fn bg1_h_high(&self) -> u8 {
            byte(self.ram, BG1_H_SCROLL_COPY + 1)
        }

        pub(crate) fn bg1_h_copy(&self) -> u16 {
            word(self.ram, BG1_H_SCROLL_COPY)
        }

        pub(crate) fn bg1_h_copy_low(&self) -> u8 {
            byte(self.ram, BG1_H_SCROLL_COPY)
        }

        pub(crate) fn bg1_v_high(&self) -> u8 {
            byte(self.ram, BG1_V_SCROLL_COPY + 1)
        }

        pub(crate) fn bg1_v_copy(&self) -> u16 {
            word(self.ram, BG1_V_SCROLL_COPY)
        }

        pub(crate) fn bg1_v_copy_low(&self) -> u8 {
            byte(self.ram, BG1_V_SCROLL_COPY)
        }

        pub(crate) fn bg2_h_high(&self) -> u8 {
            byte(self.ram, BG2_H_SCROLL_COPY + 1)
        }

        pub(crate) fn bg2_h_copy(&self) -> u16 {
            word(self.ram, BG2_H_SCROLL_COPY)
        }

        pub(crate) fn bg2_h_copy_low(&self) -> u8 {
            byte(self.ram, BG2_H_SCROLL_COPY)
        }

        pub(crate) fn bg2_v_high(&self) -> u8 {
            byte(self.ram, BG2_V_SCROLL_COPY + 1)
        }

        pub(crate) fn bg2_v_copy(&self) -> u16 {
            word(self.ram, BG2_V_SCROLL_COPY)
        }

        pub(crate) fn bg2_v_copy_low(&self) -> u8 {
            byte(self.ram, BG2_V_SCROLL_COPY)
        }

        pub(crate) fn bg1_h_copy2(&self) -> u16 {
            word(self.ram, BG1_X_SCROLL)
        }

        pub(crate) fn bg1_v_copy2(&self) -> u16 {
            word(self.ram, BG1_Y_SCROLL)
        }

        pub(crate) fn bg2_h_copy2(&self) -> u16 {
            word(self.ram, BG2_X_SCROLL)
        }

        pub(crate) fn bg2_v_copy2(&self) -> u16 {
            word(self.ram, BG2_Y_SCROLL)
        }

        pub(crate) fn bg2_copy2_for_axis(&self, vertical: bool) -> u16 {
            if vertical {
                self.bg2_v_copy2()
            } else {
                self.bg2_h_copy2()
            }
        }

        pub(crate) fn bg3_h_high(&self) -> u8 {
            byte(self.ram, BG3_H_SCROLL_COPY2 + 1)
        }

        pub(crate) fn bg3_h_copy2(&self) -> u16 {
            word(self.ram, BG3_H_SCROLL_COPY2)
        }

        pub(crate) fn bg3_h_copy2_low(&self) -> u8 {
            byte(self.ram, BG3_H_SCROLL_COPY2)
        }

        pub(crate) fn bg3_v_high(&self) -> u8 {
            byte(self.ram, BG3_V_SCROLL_COPY2 + 1)
        }

        pub(crate) fn bg3_v_copy2(&self) -> u16 {
            word(self.ram, BG3_V_SCROLL_COPY2)
        }

        pub(crate) fn bg3_v_copy2_low(&self) -> u8 {
            byte(self.ram, BG3_V_SCROLL_COPY2)
        }

        pub(crate) fn bg2_h_copy2_cached(&self) -> u16 {
            word(self.ram, BG2_H_SCROLL_COPY2_CACHED)
        }

        pub(crate) fn bg2_v_copy2_cached(&self) -> u16 {
            word(self.ram, BG2_V_SCROLL_COPY2_CACHED)
        }

        pub(crate) fn map_backup_bg1_h_copy2(&self) -> u16 {
            word(self.ram, MAP_BACKUP_BG1_H_SCROLL_COPY2)
        }

        pub(crate) fn map_backup_bg2_h_copy2(&self) -> u16 {
            word(self.ram, MAP_BACKUP_BG2_H_SCROLL_COPY2)
        }

        pub(crate) fn map_backup_bg1_v_copy2(&self) -> u16 {
            word(self.ram, MAP_BACKUP_BG1_V_SCROLL_COPY2)
        }

        pub(crate) fn map_backup_bg2_v_copy2(&self) -> u16 {
            word(self.ram, MAP_BACKUP_BG2_V_SCROLL_COPY2)
        }

        pub(crate) fn special_exit_bg2_h_copy2(&self) -> u16 {
            word(self.ram, BG2_H_SCROLL_COPY2_SPECIAL_EXIT)
        }

        pub(crate) fn special_exit_bg2_v_copy2(&self) -> u16 {
            word(self.ram, BG2_V_SCROLL_COPY2_SPECIAL_EXIT)
        }

        pub(crate) fn exit_bg2_h_copy2(&self) -> u16 {
            word(self.ram, BG2_H_SCROLL_COPY2_EXIT)
        }

        pub(crate) fn exit_bg2_v_copy2(&self) -> u16 {
            word(self.ram, BG2_V_SCROLL_COPY2_EXIT)
        }

        pub(crate) fn mode7_center_x_high(&self) -> u8 {
            byte(self.ram, MODE7_CENTER_X_COPY + 1)
        }

        pub(crate) fn mode7_center_x(&self) -> u16 {
            word(self.ram, MODE7_CENTER_X_COPY)
        }

        pub(crate) fn mode7_center_y_high(&self) -> u8 {
            byte(self.ram, MODE7_CENTER_Y_COPY + 1)
        }

        pub(crate) fn mode7_center_y(&self) -> u16 {
            word(self.ram, MODE7_CENTER_Y_COPY)
        }

        pub(crate) fn bg1_h_subpixel(&self) -> u16 {
            word(self.ram, BG1_H_SCROLL_SUBPIXEL)
        }

        pub(crate) fn bg1_v_subpixel(&self) -> u16 {
            word(self.ram, BG1_V_SCROLL_SUBPIXEL)
        }
    }

    pub(crate) struct PpuScrollCopyViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PpuScrollCopyViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_bg1_h_high(&mut self, value: u8) {
            self.ram[BG1_X_SCROLL + 1] = value;
        }

        pub(crate) fn set_bg1_h_copy(&mut self, value: u16) {
            write_le_u16(self.ram, BG1_H_SCROLL_COPY, value);
        }

        pub(crate) fn set_bg1_v_copy(&mut self, value: u16) {
            write_le_u16(self.ram, BG1_V_SCROLL_COPY, value);
        }

        pub(crate) fn set_bg2_h_copy(&mut self, value: u16) {
            write_le_u16(self.ram, BG2_H_SCROLL_COPY, value);
        }

        pub(crate) fn set_bg2_v_copy(&mut self, value: u16) {
            write_le_u16(self.ram, BG2_V_SCROLL_COPY, value);
        }

        pub(crate) fn set_bg1_h_copy_low(&mut self, value: u8) {
            self.ram[BG1_H_SCROLL_COPY] = value;
        }

        pub(crate) fn set_bg1_v_copy_low(&mut self, value: u8) {
            self.ram[BG1_V_SCROLL_COPY] = value;
        }

        pub(crate) fn set_bg2_h_copy_low(&mut self, value: u8) {
            self.ram[BG2_H_SCROLL_COPY] = value;
        }

        pub(crate) fn set_bg2_v_copy_low(&mut self, value: u8) {
            self.ram[BG2_V_SCROLL_COPY] = value;
        }

        pub(crate) fn set_bg1_h_copy2(&mut self, value: u16) {
            write_le_u16(self.ram, BG1_X_SCROLL, value);
        }

        pub(crate) fn set_bg1_v_copy2(&mut self, value: u16) {
            write_le_u16(self.ram, BG1_Y_SCROLL, value);
        }

        pub(crate) fn set_bg2_h_copy2(&mut self, value: u16) {
            write_le_u16(self.ram, BG2_X_SCROLL, value);
        }

        pub(crate) fn set_bg2_v_copy2(&mut self, value: u16) {
            write_le_u16(self.ram, BG2_Y_SCROLL, value);
        }

        pub(crate) fn set_bg3_h_copy2(&mut self, value: u16) {
            write_le_u16(self.ram, BG3_H_SCROLL_COPY2, value);
        }

        pub(crate) fn set_bg3_v_copy2(&mut self, value: u16) {
            write_le_u16(self.ram, BG3_V_SCROLL_COPY2, value);
        }

        pub(crate) fn set_bg3_v_copy2_low(&mut self, value: u8) {
            self.ram[BG3_V_SCROLL_COPY2] = value;
        }

        pub(crate) fn set_mode7_center_x(&mut self, value: u16) {
            write_le_u16(self.ram, MODE7_CENTER_X_COPY, value);
        }

        pub(crate) fn set_mode7_center_y(&mut self, value: u16) {
            write_le_u16(self.ram, MODE7_CENTER_Y_COPY, value);
        }

        pub(crate) fn set_mode7_center(&mut self, x: u16, y: u16) {
            self.set_mode7_center_x(x);
            self.set_mode7_center_y(y);
        }

        pub(crate) fn set_bg1_h_live_and_copy(&mut self, value: u16) {
            self.set_bg1_h_copy2(value);
            self.set_bg1_h_copy(value);
        }

        pub(crate) fn set_bg1_v_live_and_copy(&mut self, value: u16) {
            self.set_bg1_v_copy2(value);
            self.set_bg1_v_copy(value);
        }

        pub(crate) fn set_bg2_h_live_and_copy(&mut self, value: u16) {
            self.set_bg2_h_copy2(value);
            self.set_bg2_h_copy(value);
        }

        pub(crate) fn set_bg2_v_live_and_copy(&mut self, value: u16) {
            self.set_bg2_v_copy2(value);
            self.set_bg2_v_copy(value);
        }

        pub(crate) fn set_bg1_bg2_h_live_and_copy(&mut self, value: u16) {
            self.set_bg2_h_live_and_copy(value);
            self.set_bg1_h_live_and_copy(value);
        }

        pub(crate) fn set_bg1_bg2_v_live_and_copy(&mut self, value: u16) {
            self.set_bg2_v_live_and_copy(value);
            self.set_bg1_v_live_and_copy(value);
        }

        pub(crate) fn set_bg1_bg2_live_and_copy(
            &mut self,
            bg2_h: u16,
            bg2_v: u16,
            bg1_h: u16,
            bg1_v: u16,
        ) {
            self.set_bg2_h_live_and_copy(bg2_h);
            self.set_bg2_v_live_and_copy(bg2_v);
            self.set_bg1_h_live_and_copy(bg1_h);
            self.set_bg1_v_live_and_copy(bg1_v);
        }

        pub(crate) fn set_bg2_h_copy2_cached(&mut self, value: u16) {
            write_le_u16(self.ram, BG2_H_SCROLL_COPY2_CACHED, value);
        }

        pub(crate) fn set_bg2_v_copy2_cached(&mut self, value: u16) {
            write_le_u16(self.ram, BG2_V_SCROLL_COPY2_CACHED, value);
        }

        pub(crate) fn cache_bg2_live_scroll(&mut self) {
            copy_word(self.ram, BG2_H_SCROLL_COPY2_CACHED, BG2_X_SCROLL);
            copy_word(self.ram, BG2_V_SCROLL_COPY2_CACHED, BG2_Y_SCROLL);
        }

        pub(crate) fn save_special_exit_bg2_live_scroll(&mut self) {
            copy_word(self.ram, BG2_H_SCROLL_COPY2_SPECIAL_EXIT, BG2_X_SCROLL);
            copy_word(self.ram, BG2_V_SCROLL_COPY2_SPECIAL_EXIT, BG2_Y_SCROLL);
        }

        pub(crate) fn save_exit_bg2_live_scroll(&mut self) {
            copy_word(self.ram, BG2_H_SCROLL_COPY2_EXIT, BG2_X_SCROLL);
            copy_word(self.ram, BG2_V_SCROLL_COPY2_EXIT, BG2_Y_SCROLL);
        }

        pub(crate) fn restore_special_exit_bg2_scroll_to_all_layers(&mut self) {
            let h = read_le_u16(self.ram, BG2_H_SCROLL_COPY2_SPECIAL_EXIT);
            let v = read_le_u16(self.ram, BG2_V_SCROLL_COPY2_SPECIAL_EXIT);
            self.set_all_layer_h_scrolls(h);
            self.set_all_layer_v_scrolls(v);
        }

        pub(crate) fn restore_exit_bg2_scroll_to_all_layers(&mut self) {
            let h = read_le_u16(self.ram, BG2_H_SCROLL_COPY2_EXIT);
            let v = read_le_u16(self.ram, BG2_V_SCROLL_COPY2_EXIT);
            self.set_all_layer_h_scrolls(h);
            self.set_all_layer_v_scrolls(v);
        }

        pub(crate) fn set_all_layer_h_scrolls(&mut self, value: u16) {
            self.set_bg2_h_copy2(value);
            self.set_bg2_h_copy(value);
            self.set_bg1_h_copy2(value);
            self.set_bg1_h_copy(value);
        }

        pub(crate) fn set_all_layer_v_scrolls(&mut self, value: u16) {
            self.set_bg2_v_copy2(value);
            self.set_bg2_v_copy(value);
            self.set_bg1_v_copy2(value);
            self.set_bg1_v_copy(value);
        }

        pub(crate) fn set_map_backup_scrolls(
            &mut self,
            bg1_h: u16,
            bg2_h: u16,
            bg1_v: u16,
            bg2_v: u16,
        ) {
            write_le_u16(self.ram, MAP_BACKUP_BG1_H_SCROLL_COPY2, bg1_h);
            write_le_u16(self.ram, MAP_BACKUP_BG2_H_SCROLL_COPY2, bg2_h);
            write_le_u16(self.ram, MAP_BACKUP_BG1_V_SCROLL_COPY2, bg1_v);
            write_le_u16(self.ram, MAP_BACKUP_BG2_V_SCROLL_COPY2, bg2_v);
        }

        pub(crate) fn clear_bg3_h_copy2(&mut self) {
            self.set_bg3_h_copy2(0);
        }

        pub(crate) fn clear_bg3_v_copy2(&mut self) {
            self.set_bg3_v_copy2(0);
        }

        pub(crate) fn add_bg1_h_copy_low(&mut self, value: u8) {
            self.ram[BG1_H_SCROLL_COPY] = self.ram[BG1_H_SCROLL_COPY].wrapping_add(value);
        }

        pub(crate) fn add_bg1_v_copy_low(&mut self, value: u8) {
            self.ram[BG1_V_SCROLL_COPY] = self.ram[BG1_V_SCROLL_COPY].wrapping_add(value);
        }

        pub(crate) fn add_bg2_v_copy_low(&mut self, value: u8) {
            self.ram[BG2_V_SCROLL_COPY] = self.ram[BG2_V_SCROLL_COPY].wrapping_add(value);
        }

        pub(crate) fn subtract_bg2_h_copy_low(&mut self, value: u8) {
            self.ram[BG2_H_SCROLL_COPY] = self.ram[BG2_H_SCROLL_COPY].wrapping_sub(value);
        }

        pub(crate) fn add_bg2_h_copy2_signed(&mut self, value: i8) {
            let next = read_le_u16(self.ram, BG2_X_SCROLL).wrapping_add(value as i16 as u16);
            write_le_u16(self.ram, BG2_X_SCROLL, next);
        }

        pub(crate) fn add_bg2_v_copy2_signed(&mut self, value: i8) {
            let next = read_le_u16(self.ram, BG2_Y_SCROLL).wrapping_add(value as i16 as u16);
            write_le_u16(self.ram, BG2_Y_SCROLL, next);
        }

        pub(crate) fn add_bg3_v_copy2_signed(&mut self, value: i8) {
            let next = read_le_u16(self.ram, BG3_V_SCROLL_COPY2).wrapping_add(value as i16 as u16);
            write_le_u16(self.ram, BG3_V_SCROLL_COPY2, next);
        }

        fn add_subpixel_scroll(&mut self, subpixel_addr: usize, scroll_addr: usize, value: u32) {
            let current = (read_le_u16(self.ram, subpixel_addr) as u32)
                | ((read_le_u16(self.ram, scroll_addr) as u32) << 16);
            let next = current.wrapping_add(value);
            write_le_u16(self.ram, subpixel_addr, next as u16);
            write_le_u16(self.ram, scroll_addr, (next >> 16) as u16);
        }

        fn subtract_subpixel_scroll(
            &mut self,
            subpixel_addr: usize,
            scroll_addr: usize,
            value: u32,
        ) {
            let current = (read_le_u16(self.ram, subpixel_addr) as u32)
                | ((read_le_u16(self.ram, scroll_addr) as u32) << 16);
            let next = current.wrapping_sub(value);
            write_le_u16(self.ram, subpixel_addr, next as u16);
            write_le_u16(self.ram, scroll_addr, (next >> 16) as u16);
        }

        pub(crate) fn clear_bg1_scroll_subpixels(&mut self) {
            write_le_u16(self.ram, BG1_H_SCROLL_SUBPIXEL, 0);
            write_le_u16(self.ram, BG1_V_SCROLL_SUBPIXEL, 0);
        }

        pub(crate) fn add_bg1_h_live_subpixel(&mut self, subpixel: u16, scroll: u16) {
            self.add_subpixel_scroll(
                BG1_H_SCROLL_SUBPIXEL,
                BG1_X_SCROLL,
                (subpixel as u32) | ((scroll as u32) << 16),
            );
        }

        pub(crate) fn add_bg1_v_live_subpixel(&mut self, subpixel: u16, scroll: u16) {
            self.add_subpixel_scroll(
                BG1_V_SCROLL_SUBPIXEL,
                BG1_Y_SCROLL,
                (subpixel as u32) | ((scroll as u32) << 16),
            );
        }

        pub(crate) fn subtract_bg1_v_live_subpixel(&mut self, value: u32) {
            self.subtract_subpixel_scroll(BG1_V_SCROLL_SUBPIXEL, BG1_Y_SCROLL, value);
        }

        pub(crate) fn add_bg1_h_copy2_subpixel(&mut self, subpixel: u16, scroll: u16) {
            self.add_bg1_h_live_subpixel(subpixel, scroll);
        }

        pub(crate) fn add_bg1_v_copy2_subpixel(&mut self, subpixel: u16, scroll: u16) {
            self.add_bg1_v_live_subpixel(subpixel, scroll);
        }

        pub(crate) fn subtract_bg1_v_copy2_subpixel(&mut self, subpixel: u16, scroll: u16) {
            self.subtract_subpixel_scroll(
                BG1_V_SCROLL_SUBPIXEL,
                BG1_Y_SCROLL,
                (subpixel as u32) | ((scroll as u32) << 16),
            );
        }

        pub(crate) fn set_bg1_h_subpixel(&mut self, value: u16) {
            write_le_u16(self.ram, BG1_H_SCROLL_SUBPIXEL, value);
        }

        pub(crate) fn set_bg1_v_subpixel(&mut self, value: u16) {
            write_le_u16(self.ram, BG1_V_SCROLL_SUBPIXEL, value);
        }

        pub(crate) fn step_bg2_h_copy2_toward_cached(&mut self) {
            let h = read_le_u16(self.ram, BG2_X_SCROLL);
            let cached = read_le_u16(self.ram, BG2_H_SCROLL_COPY2_CACHED);
            if h != cached {
                write_le_u16(
                    self.ram,
                    BG2_X_SCROLL,
                    if h < cached {
                        h.wrapping_add(1)
                    } else {
                        h.wrapping_sub(1)
                    },
                );
            }
        }

        pub(crate) fn step_bg2_v_copy2_toward_cached(&mut self) {
            let v = read_le_u16(self.ram, BG2_Y_SCROLL);
            let cached = read_le_u16(self.ram, BG2_V_SCROLL_COPY2_CACHED);
            if v != cached {
                write_le_u16(
                    self.ram,
                    BG2_Y_SCROLL,
                    if v < cached {
                        v.wrapping_add(1)
                    } else {
                        v.wrapping_sub(1)
                    },
                );
            }
        }

        pub(crate) fn add_bg2_h_copy2(&mut self, value: u16) {
            let next = read_le_u16(self.ram, BG2_X_SCROLL).wrapping_add(value);
            write_le_u16(self.ram, BG2_X_SCROLL, next);
        }

        pub(crate) fn add_bg2_v_copy2(&mut self, value: u16) {
            let next = read_le_u16(self.ram, BG2_Y_SCROLL).wrapping_add(value);
            write_le_u16(self.ram, BG2_Y_SCROLL, next);
        }

        pub(crate) fn add_bg2_copy2_for_axis_signed(&mut self, vertical: bool, value: i16) {
            if vertical {
                let next = read_le_u16(self.ram, BG2_Y_SCROLL).wrapping_add_signed(value);
                write_le_u16(self.ram, BG2_Y_SCROLL, next);
            } else {
                let next = read_le_u16(self.ram, BG2_X_SCROLL).wrapping_add_signed(value);
                write_le_u16(self.ram, BG2_X_SCROLL, next);
            }
        }

        pub(crate) fn copy_bg1_live_to_ppu_copy(&mut self) {
            copy_word(self.ram, BG1_H_SCROLL_COPY, BG1_X_SCROLL);
            copy_word(self.ram, BG1_V_SCROLL_COPY, BG1_Y_SCROLL);
        }

        pub(crate) fn copy_bg2_live_to_ppu_copy(&mut self) {
            copy_word(self.ram, BG2_H_SCROLL_COPY, BG2_X_SCROLL);
            copy_word(self.ram, BG2_V_SCROLL_COPY, BG2_Y_SCROLL);
        }

        pub(crate) fn copy_live_to_ppu_copy(&mut self) {
            self.copy_bg1_live_to_ppu_copy();
            self.copy_bg2_live_to_ppu_copy();
        }

        pub(crate) fn copy_bg2_live_to_bg1_live(&mut self) {
            copy_word(self.ram, BG1_X_SCROLL, BG2_X_SCROLL);
            copy_word(self.ram, BG1_Y_SCROLL, BG2_Y_SCROLL);
        }

        pub(crate) fn copy_bg2_h_live_to_bg1_h_live(&mut self) {
            copy_word(self.ram, BG1_X_SCROLL, BG2_X_SCROLL);
        }

        pub(crate) fn copy_bg2_v_live_to_bg1_v_live(&mut self) {
            copy_word(self.ram, BG1_Y_SCROLL, BG2_Y_SCROLL);
        }
    }

    pub(crate) struct AttractStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> AttractStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn state(&self) -> u8 {
            byte(self.ram, ATTRACT_STATE)
        }

        pub(crate) fn state_word(&self) -> u16 {
            word(self.ram, ATTRACT_STATE)
        }

        pub(crate) fn sequence(&self) -> u8 {
            byte(self.ram, ATTRACT_SEQUENCE)
        }

        pub(crate) fn scene_timer(&self) -> u8 {
            byte(self.ram, ATTRACT_SCENE_TIMER)
        }

        pub(crate) fn scene_substep(&self) -> u8 {
            byte(self.ram, ATTRACT_SCENE_SUBSTEP)
        }

        pub(crate) fn x_base(&self) -> u8 {
            byte(self.ram, ATTRACT_X_BASE)
        }

        pub(crate) fn x_base_word(&self) -> u16 {
            word(self.ram, ATTRACT_X_BASE)
        }

        pub(crate) fn x_base_high(&self) -> u8 {
            byte(self.ram, ATTRACT_X_BASE_HI)
        }

        pub(crate) fn y_base(&self) -> u8 {
            byte(self.ram, ATTRACT_Y_BASE)
        }

        pub(crate) fn oam_index(&self) -> u8 {
            byte(self.ram, ATTRACT_OAM_IDX)
        }

        pub(crate) fn maiden_warp_step(&self) -> u8 {
            byte(self.ram, ATTRACT_MAIDEN_WARP_STEP)
        }

        pub(crate) fn intro_step_index(&self) -> u8 {
            byte(self.ram, INTRO_STEP_INDEX)
        }

        pub(crate) fn intro_step_timer(&self) -> u8 {
            byte(self.ram, INTRO_STEP_TIMER)
        }

        pub(crate) fn intro_frame_counter(&self) -> u8 {
            byte(self.ram, INTRO_FRAME_CTR)
        }

        pub(crate) fn intro_did_run_step(&self) -> u8 {
            byte(self.ram, INTRO_DID_RUN_STEP)
        }

        pub(crate) fn intro_palette_flash_count(&self) -> u8 {
            byte(self.ram, INTRO_TIMES_PAL_FLASH)
        }

        pub(crate) fn legend_flag(&self) -> u8 {
            byte(self.ram, ATTRACT_LEGEND_FLAG)
        }
        pub(crate) fn next_legend_gfx(&self) -> u8 {
            byte(self.ram, ATTRACT_NEXT_LEGEND_GFX)
        }
        pub(crate) fn next_legend_image(&self) -> u8 {
            byte(self.ram, ATTRACT_NEXT_LEGEND_GFX) >> 1
        }
        pub(crate) fn bg2_vofs_backup(&self) -> u16 {
            word(self.ram, ATTRACT_BG2_VOFS_BACKUP)
        }
        pub(crate) fn throne_fade_timer(&self) -> u8 {
            byte(self.ram, ATTRACT_THRONE_FADE_TIMER)
        }
        pub(crate) fn prison_zelda_y_base(&self) -> u8 {
            byte(self.ram, ATTRACT_PRISON_ZELDA_Y_BASE)
        }
        pub(crate) fn anim_step_counter(&self) -> u8 {
            byte(self.ram, ATTRACT_ANIM_STEP_COUNTER)
        }
        pub(crate) fn soldier_anim_step(&self) -> u8 {
            byte(self.ram, ATTRACT_SOLDIER_ANIM_STEP)
        }
        pub(crate) fn prison_soldier_x_lo(&self) -> u8 {
            byte(self.ram, ATTRACT_PRISON_SOLDIER_X_LO)
        }
        pub(crate) fn scene_frame_counter(&self) -> u8 {
            byte(self.ram, ATTRACT_SCENE_FRAME_COUNTER)
        }
        pub(crate) fn scene_done_flag(&self) -> u8 {
            byte(self.ram, ATTRACT_SCENE_DONE_FLAG)
        }
        pub(crate) fn legend_ctr(&self) -> u16 {
            word(self.ram, ATTRACT_LEGEND_CTR)
        }
        pub(crate) fn fade_in_complete_flag(&self) -> u8 {
            byte(self.ram, ATTRACT_FADE_IN_COMPLETE_FLAG)
        }
        pub(crate) fn fade_in_done_flag(&self) -> u8 {
            byte(self.ram, ATTRACT_FADE_IN_DONE_FLAG)
        }
        pub(crate) fn substep_delay_counter(&self) -> u8 {
            byte(self.ram, ATTRACT_SUBSTEP_DELAY_COUNTER)
        }
        pub(crate) fn maiden_warp_timer_a(&self) -> u8 {
            byte(self.ram, ATTRACT_MAIDEN_WARP_TIMER_A)
        }
        pub(crate) fn maiden_warp_timer_b(&self) -> u8 {
            byte(self.ram, ATTRACT_MAIDEN_WARP_TIMER_B)
        }
        pub(crate) fn vram_dst_byte(&self) -> u8 {
            byte(self.ram, ATTRACT_VRAM_DST)
        }
        pub(crate) fn vram_dst_word(&self) -> u16 {
            word(self.ram, ATTRACT_VRAM_DST)
        }
        pub(crate) fn mode7_zoom_timer(&self) -> u8 {
            byte(self.ram, TIMER_FOR_MODE7_ZOOM)
        }
    }

    pub(crate) struct AttractStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> AttractStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_state(&mut self, value: u8) {
            self.ram[ATTRACT_STATE] = value;
        }

        pub(crate) fn set_state_word(&mut self, value: u16) {
            write_le_u16(self.ram, ATTRACT_STATE, value);
        }

        pub(crate) fn increment_state(&mut self) -> u8 {
            self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(1);
            self.ram[ATTRACT_STATE]
        }

        pub(crate) fn add_state(&mut self, value: u8) -> u8 {
            self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(value);
            self.ram[ATTRACT_STATE]
        }

        pub(crate) fn subtract_state(&mut self, value: u8) -> u8 {
            self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_sub(value);
            self.ram[ATTRACT_STATE]
        }

        pub(crate) fn set_sequence(&mut self, value: u8) {
            self.ram[ATTRACT_SEQUENCE] = value;
        }

        pub(crate) fn increment_sequence(&mut self) -> u8 {
            self.ram[ATTRACT_SEQUENCE] = self.ram[ATTRACT_SEQUENCE].wrapping_add(1);
            self.ram[ATTRACT_SEQUENCE]
        }

        pub(crate) fn set_scene_timer(&mut self, value: u8) {
            self.ram[ATTRACT_SCENE_TIMER] = value;
        }

        pub(crate) fn decrement_scene_timer(&mut self) -> u8 {
            self.ram[ATTRACT_SCENE_TIMER] = self.ram[ATTRACT_SCENE_TIMER].wrapping_sub(1);
            self.ram[ATTRACT_SCENE_TIMER]
        }

        pub(crate) fn set_scene_substep(&mut self, value: u8) {
            self.ram[ATTRACT_SCENE_SUBSTEP] = value;
        }

        pub(crate) fn increment_scene_substep(&mut self) -> u8 {
            self.ram[ATTRACT_SCENE_SUBSTEP] = self.ram[ATTRACT_SCENE_SUBSTEP].wrapping_add(1);
            self.ram[ATTRACT_SCENE_SUBSTEP]
        }

        pub(crate) fn set_x_base(&mut self, value: u8) {
            self.ram[ATTRACT_X_BASE] = value;
        }

        pub(crate) fn set_x_base_high(&mut self, value: u8) {
            self.ram[ATTRACT_X_BASE_HI] = value;
        }

        pub(crate) fn set_y_base(&mut self, value: u8) {
            self.ram[ATTRACT_Y_BASE] = value;
        }

        pub(crate) fn set_story_text_pointer(&mut self, value: u16) {
            write_le_u16(self.ram, ATTRACT_STORY_TEXT_POINTER, value);
        }

        pub(crate) fn set_oam_index(&mut self, value: u8) {
            self.ram[ATTRACT_OAM_IDX] = value;
        }

        pub(crate) fn advance_oam_index_by(&mut self, value: u8) -> u8 {
            self.ram[ATTRACT_OAM_IDX] = self.ram[ATTRACT_OAM_IDX].wrapping_add(value);
            self.ram[ATTRACT_OAM_IDX]
        }

        pub(crate) fn set_maiden_warp_step(&mut self, value: u8) {
            self.ram[ATTRACT_MAIDEN_WARP_STEP] = value;
        }

        pub(crate) fn increment_maiden_warp_step(&mut self) -> u8 {
            self.ram[ATTRACT_MAIDEN_WARP_STEP] = self.ram[ATTRACT_MAIDEN_WARP_STEP].wrapping_add(1);
            self.ram[ATTRACT_MAIDEN_WARP_STEP]
        }

        pub(crate) fn decrement_maiden_warp_step(&mut self) -> u8 {
            self.ram[ATTRACT_MAIDEN_WARP_STEP] = self.ram[ATTRACT_MAIDEN_WARP_STEP].wrapping_sub(1);
            self.ram[ATTRACT_MAIDEN_WARP_STEP]
        }

        pub(crate) fn set_intro_step_index(&mut self, value: u8) {
            self.ram[INTRO_STEP_INDEX] = value;
        }

        pub(crate) fn clear_intro_step_state_block(&mut self) {
            self.ram[INTRO_STEP_INDEX..INTRO_STEP_INDEX + 7 * 16].fill(0);
        }

        pub(crate) fn increment_intro_step_index(&mut self) -> u8 {
            self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
            self.ram[INTRO_STEP_INDEX]
        }

        pub(crate) fn set_intro_step_timer(&mut self, value: u8) {
            self.ram[INTRO_STEP_TIMER] = value;
        }

        pub(crate) fn increment_intro_step_timer(&mut self) -> u8 {
            self.ram[INTRO_STEP_TIMER] = self.ram[INTRO_STEP_TIMER].wrapping_add(1);
            self.ram[INTRO_STEP_TIMER]
        }

        pub(crate) fn decrement_intro_step_timer(&mut self) -> u8 {
            self.ram[INTRO_STEP_TIMER] = self.ram[INTRO_STEP_TIMER].wrapping_sub(1);
            self.ram[INTRO_STEP_TIMER]
        }

        pub(crate) fn increment_intro_frame_counter(&mut self) -> u8 {
            self.ram[INTRO_FRAME_CTR] = self.ram[INTRO_FRAME_CTR].wrapping_add(1);
            self.ram[INTRO_FRAME_CTR]
        }

        pub(crate) fn set_intro_did_run_step(&mut self, value: u8) {
            self.ram[INTRO_DID_RUN_STEP] = value;
        }

        pub(crate) fn clear_intro_did_run_step(&mut self) {
            self.ram[INTRO_DID_RUN_STEP] = 0;
        }

        pub(crate) fn mark_intro_did_run_step(&mut self) {
            self.ram[INTRO_DID_RUN_STEP] = 1;
        }

        pub(crate) fn set_intro_palette_flash_count(&mut self, value: u8) {
            self.ram[INTRO_TIMES_PAL_FLASH] = value;
        }

        pub(crate) fn clear_intro_palette_flash_count(&mut self) {
            self.ram[INTRO_TIMES_PAL_FLASH] = 0;
        }

        pub(crate) fn decrement_intro_palette_flash_count(&mut self) -> u8 {
            self.ram[INTRO_TIMES_PAL_FLASH] = self.ram[INTRO_TIMES_PAL_FLASH].wrapping_sub(1);
            self.ram[INTRO_TIMES_PAL_FLASH]
        }

        pub(crate) fn increment_legend_flag(&mut self) {
            self.ram[ATTRACT_LEGEND_FLAG] = self.ram[ATTRACT_LEGEND_FLAG].wrapping_add(1);
        }
        pub(crate) fn clear_legend_flag(&mut self) {
            self.ram[ATTRACT_LEGEND_FLAG] = 0;
        }
        pub(crate) fn clear_next_legend_gfx(&mut self) {
            self.ram[ATTRACT_NEXT_LEGEND_GFX] = 0;
        }
        pub(crate) fn advance_next_legend_gfx(&mut self) {
            self.ram[ATTRACT_NEXT_LEGEND_GFX] = self.ram[ATTRACT_NEXT_LEGEND_GFX].wrapping_add(2);
        }
        pub(crate) fn set_bg2_vofs_backup(&mut self, value: u16) {
            write_le_u16(self.ram, ATTRACT_BG2_VOFS_BACKUP, value);
        }
        pub(crate) fn set_throne_fade_timer(&mut self, value: u8) {
            self.ram[ATTRACT_THRONE_FADE_TIMER] = value;
        }
        pub(crate) fn decrement_throne_fade_timer(&mut self) -> u8 {
            self.ram[ATTRACT_THRONE_FADE_TIMER] =
                self.ram[ATTRACT_THRONE_FADE_TIMER].wrapping_sub(1);
            self.ram[ATTRACT_THRONE_FADE_TIMER]
        }
        pub(crate) fn set_prison_zelda_y_base(&mut self, value: u8) {
            self.ram[ATTRACT_PRISON_ZELDA_Y_BASE] = value;
        }
        pub(crate) fn decrement_prison_zelda_y_base(&mut self) {
            self.ram[ATTRACT_PRISON_ZELDA_Y_BASE] =
                self.ram[ATTRACT_PRISON_ZELDA_Y_BASE].wrapping_sub(1);
        }
        pub(crate) fn set_anim_step_counter(&mut self, value: u8) {
            self.ram[ATTRACT_ANIM_STEP_COUNTER] = value;
        }
        pub(crate) fn decrement_anim_step_counter(&mut self) -> u8 {
            self.ram[ATTRACT_ANIM_STEP_COUNTER] =
                self.ram[ATTRACT_ANIM_STEP_COUNTER].wrapping_sub(1);
            self.ram[ATTRACT_ANIM_STEP_COUNTER]
        }
        pub(crate) fn increment_anim_step_counter(&mut self) -> u8 {
            self.ram[ATTRACT_ANIM_STEP_COUNTER] =
                self.ram[ATTRACT_ANIM_STEP_COUNTER].wrapping_add(1);
            self.ram[ATTRACT_ANIM_STEP_COUNTER]
        }
        pub(crate) fn set_soldier_anim_step(&mut self, value: u8) {
            self.ram[ATTRACT_SOLDIER_ANIM_STEP] = value;
        }
        pub(crate) fn increment_soldier_anim_step(&mut self) {
            self.ram[ATTRACT_SOLDIER_ANIM_STEP] =
                self.ram[ATTRACT_SOLDIER_ANIM_STEP].wrapping_add(1);
        }
        pub(crate) fn set_prison_soldier_x_lo(&mut self, value: u8) {
            self.ram[ATTRACT_PRISON_SOLDIER_X_LO] = value;
        }
        pub(crate) fn set_scene_frame_counter(&mut self, value: u8) {
            self.ram[ATTRACT_SCENE_FRAME_COUNTER] = value;
        }
        pub(crate) fn increment_scene_frame_counter(&mut self) -> u8 {
            self.ram[ATTRACT_SCENE_FRAME_COUNTER] =
                self.ram[ATTRACT_SCENE_FRAME_COUNTER].wrapping_add(1);
            self.ram[ATTRACT_SCENE_FRAME_COUNTER]
        }
        pub(crate) fn decrement_scene_frame_counter(&mut self) -> u8 {
            self.ram[ATTRACT_SCENE_FRAME_COUNTER] =
                self.ram[ATTRACT_SCENE_FRAME_COUNTER].wrapping_sub(1);
            self.ram[ATTRACT_SCENE_FRAME_COUNTER]
        }
        pub(crate) fn increment_scene_done_flag(&mut self) {
            self.ram[ATTRACT_SCENE_DONE_FLAG] = self.ram[ATTRACT_SCENE_DONE_FLAG].wrapping_add(1);
        }
        pub(crate) fn set_legend_ctr(&mut self, value: u16) {
            write_le_u16(self.ram, ATTRACT_LEGEND_CTR, value);
        }
        pub(crate) fn decrement_legend_ctr(&mut self) -> u16 {
            let v = read_le_u16(self.ram, ATTRACT_LEGEND_CTR).wrapping_sub(1);
            write_le_u16(self.ram, ATTRACT_LEGEND_CTR, v);
            v
        }
        pub(crate) fn set_fade_in_complete_flag(&mut self, value: u8) {
            self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG] = value;
        }
        pub(crate) fn increment_fade_in_complete_flag(&mut self) {
            self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG] =
                self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG].wrapping_add(1);
        }
        pub(crate) fn clear_fade_in_done_flag(&mut self) {
            self.ram[ATTRACT_FADE_IN_DONE_FLAG] = 0;
        }
        pub(crate) fn increment_fade_in_done_flag(&mut self) {
            self.ram[ATTRACT_FADE_IN_DONE_FLAG] =
                self.ram[ATTRACT_FADE_IN_DONE_FLAG].wrapping_add(1);
        }
        pub(crate) fn clear_substep_delay_counter(&mut self) {
            self.ram[ATTRACT_SUBSTEP_DELAY_COUNTER] = 0;
        }
        pub(crate) fn increment_substep_delay_counter(&mut self) {
            self.ram[ATTRACT_SUBSTEP_DELAY_COUNTER] =
                self.ram[ATTRACT_SUBSTEP_DELAY_COUNTER].wrapping_add(1);
        }
        pub(crate) fn set_maiden_warp_timer_a(&mut self, value: u8) {
            self.ram[ATTRACT_MAIDEN_WARP_TIMER_A] = value;
        }
        pub(crate) fn decrement_maiden_warp_timer_a(&mut self) -> u8 {
            self.ram[ATTRACT_MAIDEN_WARP_TIMER_A] =
                self.ram[ATTRACT_MAIDEN_WARP_TIMER_A].wrapping_sub(1);
            self.ram[ATTRACT_MAIDEN_WARP_TIMER_A]
        }
        pub(crate) fn set_maiden_warp_timer_b(&mut self, value: u8) {
            self.ram[ATTRACT_MAIDEN_WARP_TIMER_B] = value;
        }
        pub(crate) fn decrement_maiden_warp_timer_b(&mut self) -> u8 {
            self.ram[ATTRACT_MAIDEN_WARP_TIMER_B] =
                self.ram[ATTRACT_MAIDEN_WARP_TIMER_B].wrapping_sub(1);
            self.ram[ATTRACT_MAIDEN_WARP_TIMER_B]
        }
        pub(crate) fn set_vram_dst_byte(&mut self, value: u8) {
            self.ram[ATTRACT_VRAM_DST] = value;
        }
        pub(crate) fn decrement_vram_dst_byte(&mut self) {
            self.ram[ATTRACT_VRAM_DST] = self.ram[ATTRACT_VRAM_DST].wrapping_sub(1);
        }
        pub(crate) fn set_vram_dst_word(&mut self, value: u16) {
            write_le_u16(self.ram, ATTRACT_VRAM_DST, value);
        }
        pub(crate) fn decrement_vram_dst_word(&mut self) -> u16 {
            let v = read_le_u16(self.ram, ATTRACT_VRAM_DST).wrapping_sub(1);
            write_le_u16(self.ram, ATTRACT_VRAM_DST, v);
            v
        }
        pub(crate) fn set_mode7_zoom_timer(&mut self, value: u8) {
            self.ram[TIMER_FOR_MODE7_ZOOM] = value;
        }
        pub(crate) fn decrement_mode7_zoom_timer(&mut self) {
            self.ram[TIMER_FOR_MODE7_ZOOM] = self.ram[TIMER_FOR_MODE7_ZOOM].wrapping_sub(1);
        }
    }

    pub(crate) struct AttractVramTargetView<'a> {
        ram: &'a [u8],
    }

    impl<'a> AttractVramTargetView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn high(&self) -> u8 {
            byte(self.ram, ATTRACT_VRAM_DST + 1)
        }
    }

    pub(crate) struct AttractVramTargetViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> AttractVramTargetViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_low(&mut self, value: u8) {
            self.ram[ATTRACT_VRAM_DST] = value;
        }

        pub(crate) fn clear_high(&mut self) {
            self.ram[ATTRACT_VRAM_DST + 1] = 0;
        }
    }

    pub(crate) struct DialogueMessageIndexView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DialogueMessageIndexView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn value(&self) -> u16 {
            word(self.ram, DIALOGUE_MESSAGE_INDEX)
        }
    }

    pub(crate) struct DialogueMessageIndexViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DialogueMessageIndexViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_value(&mut self, value: u16) {
            write_le_u16(self.ram, DIALOGUE_MESSAGE_INDEX, value);
        }
    }

    pub(crate) struct MultiselectChoiceView<'a> {
        ram: &'a [u8],
    }

    impl<'a> MultiselectChoiceView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn value(&self) -> u8 {
            byte(self.ram, MULTISELECT_CHOICE)
        }

        pub(crate) fn value_word(&self) -> u16 {
            word(self.ram, MULTISELECT_CHOICE)
        }

        pub(crate) fn backup(&self) -> u8 {
            byte(self.ram, MULTISELECT_CHOICE_BACKUP)
        }
    }

    pub(crate) struct MultiselectChoiceViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> MultiselectChoiceViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_value(&mut self, value: u8) {
            self.ram[MULTISELECT_CHOICE] = value;
        }

        pub(crate) fn increment_value(&mut self) {
            self.ram[MULTISELECT_CHOICE] = self.ram[MULTISELECT_CHOICE].wrapping_add(1);
        }

        pub(crate) fn decrement_value(&mut self) {
            self.ram[MULTISELECT_CHOICE] = self.ram[MULTISELECT_CHOICE].wrapping_sub(1);
        }

        pub(crate) fn restore_backup(&mut self) {
            self.ram[MULTISELECT_CHOICE] = self.ram[MULTISELECT_CHOICE_BACKUP];
        }

        pub(crate) fn save_backup(&mut self) {
            self.ram[MULTISELECT_CHOICE_BACKUP] = self.ram[MULTISELECT_CHOICE];
        }
    }

    pub(crate) struct DialogueNumberView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DialogueNumberView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn packed_digits(&self, pair_index: usize) -> u8 {
            byte(self.ram, DIALOGUE_NUMBER_LO + pair_index)
        }
    }

    pub(crate) struct DialogueNumberViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DialogueNumberViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_packed_digits(&mut self, low_pair: u8, high_pair: u8) {
            self.ram[DIALOGUE_NUMBER_LO] = low_pair;
            self.ram[DIALOGUE_NUMBER_HI] = high_pair;
        }

        pub(crate) fn set_low_pair(&mut self, value: u8) {
            self.ram[DIALOGUE_NUMBER_LO] = value;
        }

        pub(crate) fn set_high_pair(&mut self, value: u8) {
            self.ram[DIALOGUE_NUMBER_HI] = value;
        }
    }

    pub(crate) struct MessagingStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> MessagingStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn module(&self) -> u8 {
            byte(self.ram, MESSAGING_MODULE)
        }

        pub(crate) fn text_render_state(&self) -> u8 {
            byte(self.ram, TEXT_RENDER_STATE)
        }

        pub(crate) fn text_wait_countdown2(&self) -> u8 {
            byte(self.ram, TEXT_WAIT_COUNTDOWN2)
        }

        pub(crate) fn menu_animation_timer(&self) -> u8 {
            byte(self.ram, MENU_ANIMATION_TIMER)
        }

        pub(crate) fn game_over_letter_cursor(&self) -> u8 {
            byte(self.ram, GAME_OVER_LETTER_CURSOR)
        }

        pub(crate) fn effect_index(&self) -> u8 {
            byte(self.ram, GAME_OVER_LETTER_CURSOR)
        }
    }

    pub(crate) struct MessagingStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> MessagingStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_module(&mut self, value: u8) {
            self.ram[MESSAGING_MODULE] = value;
        }

        pub(crate) fn clear_module(&mut self) {
            self.ram[MESSAGING_MODULE] = 0;
        }

        pub(crate) fn clear_message_or_sprite_state_cache(&mut self) {
            self.ram[MESSAGE_OR_SPRITE_STATE_CACHE] = 0;
        }

        pub(crate) fn set_text_render_state(&mut self, value: u8) {
            self.ram[TEXT_RENDER_STATE] = value;
        }

        pub(crate) fn increment_text_render_state(&mut self) -> u8 {
            self.ram[TEXT_RENDER_STATE] = self.ram[TEXT_RENDER_STATE].wrapping_add(1);
            self.ram[TEXT_RENDER_STATE]
        }

        pub(crate) fn set_text_wait_countdown2(&mut self, value: u8) {
            self.ram[TEXT_WAIT_COUNTDOWN2] = value;
        }

        pub(crate) fn clear_text_wait_countdown2(&mut self) {
            self.ram[TEXT_WAIT_COUNTDOWN2] = 0;
        }

        pub(crate) fn decrement_text_wait_countdown2(&mut self) -> u8 {
            self.ram[TEXT_WAIT_COUNTDOWN2] = self.ram[TEXT_WAIT_COUNTDOWN2].wrapping_sub(1);
            self.ram[TEXT_WAIT_COUNTDOWN2]
        }

        pub(crate) fn set_menu_animation_timer(&mut self, value: u8) {
            self.ram[MENU_ANIMATION_TIMER] = value;
        }

        pub(crate) fn decrement_menu_animation_timer(&mut self) -> u8 {
            self.ram[MENU_ANIMATION_TIMER] = self.ram[MENU_ANIMATION_TIMER].wrapping_sub(1);
            self.ram[MENU_ANIMATION_TIMER]
        }

        pub(crate) fn set_game_over_letter_cursor(&mut self, value: u8) {
            self.ram[GAME_OVER_LETTER_CURSOR] = value;
        }

        pub(crate) fn set_effect_index(&mut self, value: u8) {
            self.ram[GAME_OVER_LETTER_CURSOR] = value;
        }

        pub(crate) fn or_effect_index(&mut self, value: u8) {
            self.ram[GAME_OVER_LETTER_CURSOR] |= value;
        }

        pub(crate) fn clear_game_over_letter_cursor(&mut self) {
            self.ram[GAME_OVER_LETTER_CURSOR] = 0;
        }

        pub(crate) fn clear_effect_index(&mut self) {
            self.ram[GAME_OVER_LETTER_CURSOR] = 0;
        }

        pub(crate) fn increment_game_over_letter_cursor(&mut self) -> u8 {
            self.ram[GAME_OVER_LETTER_CURSOR] = self.ram[GAME_OVER_LETTER_CURSOR].wrapping_add(1);
            self.ram[GAME_OVER_LETTER_CURSOR]
        }

        pub(crate) fn decrement_game_over_letter_cursor(&mut self) -> u8 {
            self.ram[GAME_OVER_LETTER_CURSOR] = self.ram[GAME_OVER_LETTER_CURSOR].wrapping_sub(1);
            self.ram[GAME_OVER_LETTER_CURSOR]
        }
    }

    pub(crate) struct MessagingTextView<'a> {
        ram: &'a [u8],
    }

    impl<'a> MessagingTextView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn byte(&self, offset: usize) -> u8 {
            byte(self.ram, MESSAGING_TEXT_BUFFER + offset)
        }

        pub(crate) fn next_byte(&self, offset: usize) -> Option<u8> {
            self.ram.get(MESSAGING_TEXT_BUFFER + offset + 1).copied()
        }
    }

    pub(crate) struct MessagingRenderBufferViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> MessagingRenderBufferViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn xor_mask(&mut self, offset: usize, mask: u8) {
            self.ram[MESSAGING_RENDER_BUFFER + offset] ^= mask;
        }

        pub(crate) fn clear_mask(&mut self, offset: usize, mask: u8) {
            self.ram[MESSAGING_RENDER_BUFFER + offset] &= !mask;
        }
    }

    pub(crate) struct VwfGlyphSpacingView<'a> {
        ram: &'a [u8],
    }

    impl<'a> VwfGlyphSpacingView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn offset(&self, index: usize) -> u8 {
            byte(self.ram, VWF_ARR + index)
        }

        pub(crate) fn cursor(&self) -> u16 {
            word(self.ram, VWF_GLYPH_CURSOR)
        }

        pub(crate) fn cursor_usize(&self) -> usize {
            usize::from(self.cursor())
        }
    }

    pub(crate) struct VwfGlyphSpacingViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> VwfGlyphSpacingViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_next_offset(&mut self, index: usize, value: u8) {
            self.ram[VWF_ARR + index + 1] = value;
        }

        pub(crate) fn set_cursor(&mut self, value: u16) {
            write_le_u16(self.ram, VWF_GLYPH_CURSOR, value);
        }

        pub(crate) fn clear_cursor(&mut self) {
            self.set_cursor(0);
        }

        pub(crate) fn increment_cursor(&mut self) -> u16 {
            let value = word(self.ram, VWF_GLYPH_CURSOR).wrapping_add(1);
            self.set_cursor(value);
            value
        }
    }

    pub(crate) struct DialogueSourceOffsetViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DialogueSourceOffsetViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn increment_bank_offset_low_nibble(&mut self) -> u8 {
            let next = self.ram[DIALOGUE_MSG_SRC_OFFS + 2].wrapping_add(1);
            self.ram[DIALOGUE_MSG_SRC_OFFS + 2] = next;
            next
        }
    }

    pub(crate) struct SelectFileScratchView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SelectFileScratchView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn choice(&self, index: usize) -> u8 {
            byte(self.ram, SELECT_FILE_CHOICE_SCRATCH + index)
        }

        pub(crate) fn cursor(&self) -> u8 {
            byte(self.ram, SELECT_FILE_CURSOR_SCRATCH)
        }

        pub(crate) fn cursor_usize(&self) -> usize {
            usize::from(self.cursor())
        }

        pub(crate) fn remembered_cursor(&self) -> u8 {
            byte(self.ram, SELECT_FILE_REMEMBERED_CURSOR)
        }

        pub(crate) fn target_word(&self) -> u16 {
            word(self.ram, SELECT_FILE_TARGET_SCRATCH)
        }

        pub(crate) fn copy_source_slot_x2(&self) -> u16 {
            word(self.ram, SELECT_FILE_COPY_SOURCE_SLOT_X2)
        }

        pub(crate) fn copy_source_slot(&self) -> usize {
            usize::from(self.copy_source_slot_x2() >> 1)
        }

        pub(crate) fn name_scroll_x(&self) -> u16 {
            word(self.ram, SELECT_FILE_NAME_SCROLL_X)
        }

        pub(crate) fn name_column(&self) -> u8 {
            byte(self.ram, SELECT_FILE_NAME_COLUMN)
        }

        pub(crate) fn name_column_usize(&self) -> usize {
            usize::from(self.name_column())
        }

        pub(crate) fn name_cursor_y(&self) -> u8 {
            byte(self.ram, SELECT_FILE_NAME_CURSOR_Y)
        }

        pub(crate) fn name_slot(&self) -> u8 {
            byte(self.ram, SELECT_FILE_NAME_SLOT)
        }

        pub(crate) fn name_slot_usize(&self) -> usize {
            usize::from(self.name_slot())
        }

        pub(crate) fn name_scroll_x_step(&self) -> u8 {
            byte(self.ram, SELECT_FILE_NAME_SCROLL_X_STEP)
        }

        pub(crate) fn name_scroll_y_step(&self) -> u8 {
            byte(self.ram, SELECT_FILE_NAME_SCROLL_Y_STEP)
        }

        pub(crate) fn is_name_scrolling(&self) -> bool {
            (self.name_scroll_x_step() | self.name_scroll_y_step()) != 0
        }

        pub(crate) fn name_row(&self) -> u8 {
            byte(self.ram, SELECT_FILE_NAME_ROW)
        }

        pub(crate) fn name_row_usize(&self) -> usize {
            usize::from(self.name_row())
        }

        pub(crate) fn name_scroll_x_direction(&self) -> u8 {
            byte(self.ram, SELECT_FILE_NAME_SCROLL_X_DIRECTION)
        }

        pub(crate) fn save_slot_flag(&self, slot: usize) -> u16 {
            word(self.ram, SELECTFILE_SAVE_SLOT_FLAGS + slot * 2)
        }

        pub(crate) fn save_slot_flags(&self) -> [u16; 3] {
            [
                self.save_slot_flag(0),
                self.save_slot_flag(1),
                self.save_slot_flag(2),
            ]
        }

        pub(crate) fn any_save_slot_flag(&self) -> bool {
            self.save_slot_flags().into_iter().any(|flag| flag != 0)
        }
    }

    pub(crate) struct SelectFileScratchViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SelectFileScratchViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_choice(&mut self, index: usize, value: u8) {
            self.ram[SELECT_FILE_CHOICE_SCRATCH + index] = value;
        }

        pub(crate) fn set_cursor(&mut self, value: u8) {
            self.ram[SELECT_FILE_CURSOR_SCRATCH] = value;
        }

        pub(crate) fn clear_cursor(&mut self) {
            self.set_cursor(0);
        }

        pub(crate) fn clear_transition_scratch(&mut self) {
            self.ram[SELECT_FILE_TRANSITION_SCRATCH] = 0;
        }

        pub(crate) fn increment_cursor(&mut self) -> u8 {
            self.ram[SELECT_FILE_CURSOR_SCRATCH] =
                self.ram[SELECT_FILE_CURSOR_SCRATCH].wrapping_add(1);
            self.ram[SELECT_FILE_CURSOR_SCRATCH]
        }

        pub(crate) fn decrement_cursor(&mut self) -> u8 {
            self.ram[SELECT_FILE_CURSOR_SCRATCH] =
                self.ram[SELECT_FILE_CURSOR_SCRATCH].wrapping_sub(1);
            self.ram[SELECT_FILE_CURSOR_SCRATCH]
        }

        pub(crate) fn set_remembered_cursor(&mut self, value: u8) {
            self.ram[SELECT_FILE_REMEMBERED_CURSOR] = value;
        }

        pub(crate) fn clear_remembered_cursor(&mut self) {
            self.set_remembered_cursor(0);
        }

        pub(crate) fn remember_current_cursor(&mut self) {
            self.ram[SELECT_FILE_REMEMBERED_CURSOR] = self.ram[SELECT_FILE_CURSOR_SCRATCH];
        }

        pub(crate) fn restore_remembered_cursor(&mut self) {
            self.ram[SELECT_FILE_CURSOR_SCRATCH] = self.ram[SELECT_FILE_REMEMBERED_CURSOR];
        }

        pub(crate) fn set_target_word(&mut self, value: u16) {
            write_le_u16(self.ram, SELECT_FILE_TARGET_SCRATCH, value);
        }

        pub(crate) fn set_copy_source_slot_x2(&mut self, value: u16) {
            write_le_u16(self.ram, SELECT_FILE_COPY_SOURCE_SLOT_X2, value);
        }

        pub(crate) fn set_copy_source_slot(&mut self, slot: u8) {
            self.set_copy_source_slot_x2(u16::from(slot) * 2);
        }

        pub(crate) fn set_name_scroll_x(&mut self, value: u16) {
            write_le_u16(self.ram, SELECT_FILE_NAME_SCROLL_X, value);
        }

        pub(crate) fn clear_name_entry_state(&mut self) {
            self.ram[SELECT_FILE_NAME_COLUMN] = 0;
            self.ram[SELECT_FILE_NAME_SLOT] = 0;
            self.ram[SELECT_FILE_NAME_ROW] = 0;
            self.ram[SELECT_FILE_CHOICE_SCRATCH] = 0;
            self.ram[SELECT_FILE_COPY_SOURCE_SLOT_X2] = 0;
            self.ram[SELECT_FILE_NAME_CURSOR_Y] = 0x83;
            write_le_u16(self.ram, SELECT_FILE_NAME_SCROLL_X, 0x01f0);
        }

        pub(crate) fn set_name_column(&mut self, value: u8) {
            self.ram[SELECT_FILE_NAME_COLUMN] = value;
        }

        pub(crate) fn set_name_cursor_y(&mut self, value: u8) {
            self.ram[SELECT_FILE_NAME_CURSOR_Y] = value;
        }

        pub(crate) fn step_name_cursor_y_toward(&mut self, target_y: u8) -> bool {
            let diff = self.ram[SELECT_FILE_NAME_CURSOR_Y].wrapping_sub(target_y);
            if diff == 0 {
                return false;
            }
            self.ram[SELECT_FILE_NAME_CURSOR_Y] = if diff & 0x80 != 0 {
                self.ram[SELECT_FILE_NAME_CURSOR_Y].wrapping_add(2)
            } else {
                self.ram[SELECT_FILE_NAME_CURSOR_Y].wrapping_sub(2)
            };
            true
        }

        pub(crate) fn set_name_slot(&mut self, value: u8) {
            self.ram[SELECT_FILE_NAME_SLOT] = value;
        }

        pub(crate) fn move_name_slot_left_wrapped(&mut self) -> u8 {
            let next = if self.ram[SELECT_FILE_NAME_SLOT] == 0 {
                5
            } else {
                self.ram[SELECT_FILE_NAME_SLOT].wrapping_sub(1)
            };
            self.ram[SELECT_FILE_NAME_SLOT] = next;
            next
        }

        pub(crate) fn move_name_slot_right_wrapped(&mut self) -> u8 {
            self.ram[SELECT_FILE_NAME_SLOT] = self.ram[SELECT_FILE_NAME_SLOT].wrapping_add(1);
            if self.ram[SELECT_FILE_NAME_SLOT] == 6 {
                self.ram[SELECT_FILE_NAME_SLOT] = 0;
            }
            self.ram[SELECT_FILE_NAME_SLOT]
        }

        pub(crate) fn set_name_scroll_x_step(&mut self, value: u8) {
            self.ram[SELECT_FILE_NAME_SCROLL_X_STEP] = value;
        }

        pub(crate) fn advance_name_scroll_x_step_by(&mut self, value: u8) -> u8 {
            self.ram[SELECT_FILE_NAME_SCROLL_X_STEP] =
                self.ram[SELECT_FILE_NAME_SCROLL_X_STEP].wrapping_add(value);
            self.ram[SELECT_FILE_NAME_SCROLL_X_STEP]
        }

        pub(crate) fn set_name_scroll_y_step(&mut self, value: u8) {
            self.ram[SELECT_FILE_NAME_SCROLL_Y_STEP] = value;
        }

        pub(crate) fn clear_name_scroll_y_step(&mut self) {
            self.ram[SELECT_FILE_NAME_SCROLL_Y_STEP] = 0;
        }

        pub(crate) fn increment_name_scroll_y_step(&mut self) -> u8 {
            self.ram[SELECT_FILE_NAME_SCROLL_Y_STEP] =
                self.ram[SELECT_FILE_NAME_SCROLL_Y_STEP].wrapping_add(1);
            self.ram[SELECT_FILE_NAME_SCROLL_Y_STEP]
        }

        pub(crate) fn set_name_row(&mut self, value: u8) {
            self.ram[SELECT_FILE_NAME_ROW] = value;
        }

        pub(crate) fn set_name_scroll_x_direction(&mut self, value: u8) {
            self.ram[SELECT_FILE_NAME_SCROLL_X_DIRECTION] = value;
        }

        pub(crate) fn set_save_slot_flag(&mut self, slot: usize, value: u16) {
            write_le_u16(self.ram, SELECTFILE_SAVE_SLOT_FLAGS + slot * 2, value);
        }

        pub(crate) fn mark_save_slot_present(&mut self, slot: usize) {
            self.set_save_slot_flag(slot, 1);
        }

        pub(crate) fn clear_save_slot_flag(&mut self, slot: usize) {
            self.set_save_slot_flag(slot, 0);
        }

        pub(crate) fn clear_save_slot_flags(&mut self) {
            self.ram[SELECTFILE_SAVE_SLOT_FLAGS..SELECTFILE_SAVE_SLOT_FLAGS + 6].fill(0);
        }
    }

    pub(crate) struct FollowerStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> FollowerStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn indicator(&self) -> u8 {
            byte(self.ram, FOLLOWER_INDICATOR)
        }

        pub(crate) fn indicator_word(&self) -> u16 {
            word(self.ram, FOLLOWER_INDICATOR)
        }

        pub(crate) fn data_index(&self) -> u8 {
            byte(self.ram, TAGALONG_DATA_INDEX)
        }

        pub(crate) fn data_index_word(&self) -> u16 {
            word(self.ram, TAGALONG_DATA_INDEX)
        }

        pub(crate) fn appearance_none_flag(&self) -> u8 {
            byte(self.ram, TAGALONG_APPEARANCE_NONE_FLAG)
        }

        pub(crate) fn dropped(&self) -> u8 {
            byte(self.ram, FOLLOWER_DROPPED)
        }

        pub(crate) fn hookshot_interlock(&self) -> u8 {
            byte(self.ram, TAGALONG_HOOKSHOT_INTERLOCK)
        }

        pub(crate) fn hookshot_interlock_is_clear(&self) -> bool {
            self.hookshot_interlock() == 0
        }

        pub(crate) fn tail_write_index(&self) -> u8 {
            byte(self.ram, FOLLOWER_TAIL_WRITE_INDEX)
        }

        pub(crate) fn hookshot_release_tail_index(&self) -> u8 {
            byte(self.ram, FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX)
        }

        pub(crate) fn event_flags(&self) -> u8 {
            byte(self.ram, TAGALONG_EVENT_FLAGS)
        }

        pub(crate) fn reacquire_timer_low(&self) -> u8 {
            byte(self.ram, TIMER_TAGALONG_REACQUIRE)
        }

        pub(crate) fn reacquire_timer(&self) -> u16 {
            word(self.ram, TIMER_TAGALONG_REACQUIRE)
        }

        pub(crate) fn draw_anim_frame(&self) -> u8 {
            byte(self.ram, TAGALONG_ANIM_FRAME_COUNTER)
        }

        pub(crate) fn saved_y(&self) -> u16 {
            word(self.ram, FOLLOWER_SAVED_Y)
        }

        pub(crate) fn saved_x(&self) -> u16 {
            word(self.ram, FOLLOWER_SAVED_X)
        }

        pub(crate) fn saved_indoor_flag(&self) -> u8 {
            byte(self.ram, FOLLOWER_SAVED_INDOORS)
        }

        pub(crate) fn saved_floor(&self) -> u8 {
            byte(self.ram, FOLLOWER_SAVED_FLOOR)
        }

        pub(crate) fn palette_swap_flag(&self) -> u8 {
            byte(self.ram, FOLLOWER_PALETTE_SWAP_FLAG)
        }

        /// Cutscene progress byte shared by the Priest and rescued Zelda
        /// during the opening rescue sequence.
        pub(crate) fn zelda_rescue_cutscene_state(&self) -> u8 {
            byte(self.ram, ZELDA_RESCUE_CUTSCENE_STATE)
        }
    }

    pub(crate) struct FollowerStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> FollowerStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_indicator(&mut self, value: u8) {
            self.ram[FOLLOWER_INDICATOR] = value;
        }

        pub(crate) fn set_data_index(&mut self, value: u8) {
            self.ram[TAGALONG_DATA_INDEX] = value;
        }

        pub(crate) fn advance_data_index_wrapping_at_20(&mut self) {
            self.ram[TAGALONG_DATA_INDEX] = if self.ram[TAGALONG_DATA_INDEX].wrapping_add(1) >= 20 {
                0
            } else {
                self.ram[TAGALONG_DATA_INDEX].wrapping_add(1)
            };
        }

        pub(crate) fn xor_indicator(&mut self, value: u8) {
            self.ram[FOLLOWER_INDICATOR] ^= value;
        }

        pub(crate) fn set_appearance_none_flag(&mut self, value: u8) {
            self.ram[TAGALONG_APPEARANCE_NONE_FLAG] = value;
        }

        pub(crate) fn set_dropped(&mut self, value: u8) {
            self.ram[FOLLOWER_DROPPED] = value;
        }

        pub(crate) fn clear_hookshot_interlock(&mut self) {
            self.ram[TAGALONG_HOOKSHOT_INTERLOCK] = 0;
        }

        pub(crate) fn set_hookshot_interlock(&mut self) {
            self.ram[TAGALONG_HOOKSHOT_INTERLOCK] = 1;
        }

        pub(crate) fn clear_event_flags(&mut self) {
            self.ram[TAGALONG_EVENT_FLAGS] = 0;
        }

        pub(crate) fn or_event_flags(&mut self, value: u8) {
            self.ram[TAGALONG_EVENT_FLAGS] |= value;
        }

        pub(crate) fn and_event_flags(&mut self, value: u8) {
            self.ram[TAGALONG_EVENT_FLAGS] &= value;
        }

        pub(crate) fn set_hookshot_release_tail_index_from_tail_write_index(&mut self) {
            self.ram[FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX] = self.ram[FOLLOWER_TAIL_WRITE_INDEX];
        }

        pub(crate) fn set_tail_write_index(&mut self, value: u8) {
            self.ram[FOLLOWER_TAIL_WRITE_INDEX] = value;
        }

        pub(crate) fn increment_tail_write_index(&mut self) {
            self.ram[FOLLOWER_TAIL_WRITE_INDEX] =
                self.ram[FOLLOWER_TAIL_WRITE_INDEX].wrapping_add(1);
        }

        pub(crate) fn set_hookshot_release_tail_index(&mut self, value: u8) {
            self.ram[FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX] = value;
        }

        pub(crate) fn set_reacquire_timer_low(&mut self, value: u8) {
            self.ram[TIMER_TAGALONG_REACQUIRE] = value;
        }

        pub(crate) fn decrement_reacquire_timer_low(&mut self) {
            self.ram[TIMER_TAGALONG_REACQUIRE] = self.ram[TIMER_TAGALONG_REACQUIRE].wrapping_sub(1);
        }

        pub(crate) fn set_reacquire_timer(&mut self, value: u16) {
            write_le_u16(self.ram, TIMER_TAGALONG_REACQUIRE, value);
        }

        pub(crate) fn clear_tagalong_shared_state_a(&mut self) {
            self.ram[TAGALONG_SHARED_STATE_A] = 0;
        }

        pub(crate) fn clear_draw_anim_frame(&mut self) {
            self.ram[TAGALONG_ANIM_FRAME_COUNTER] = 0;
        }

        pub(crate) fn increment_and_cycle_draw_anim_frame(&mut self) {
            self.ram[TAGALONG_ANIM_FRAME_COUNTER] =
                self.ram[TAGALONG_ANIM_FRAME_COUNTER].wrapping_add(1);
            if self.ram[TAGALONG_ANIM_FRAME_COUNTER] == 3 {
                self.ram[TAGALONG_ANIM_FRAME_COUNTER] = 0;
            }
        }

        pub(crate) fn clear_jump_timer(&mut self) {
            self.ram[FOLLOWER_JUMP_TIMER] = 0;
        }

        pub(crate) fn set_saved_y(&mut self, value: u16) {
            write_le_u16(self.ram, FOLLOWER_SAVED_Y, value);
        }

        pub(crate) fn set_saved_x(&mut self, value: u16) {
            write_le_u16(self.ram, FOLLOWER_SAVED_X, value);
        }

        pub(crate) fn set_saved_indoor_flag(&mut self, value: u8) {
            self.ram[FOLLOWER_SAVED_INDOORS] = value;
        }

        pub(crate) fn set_saved_floor(&mut self, value: u8) {
            self.ram[FOLLOWER_SAVED_FLOOR] = value;
        }

        pub(crate) fn clear_kiki_anim_counter(&mut self) {
            self.ram[FOLLOWER_KIKI_ANIM_COUNTER] = 0;
        }

        pub(crate) fn set_zelda_rescue_cutscene_state(&mut self, value: u8) {
            self.ram[ZELDA_RESCUE_CUTSCENE_STATE] = value;
        }
    }

    pub(crate) struct TagalongSlotView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> TagalongSlotView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(
                self.ram,
                TAGALONG_X_LO + self.slot,
                TAGALONG_X_HI + self.slot,
            )
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(
                self.ram,
                TAGALONG_Y_LO + self.slot,
                TAGALONG_Y_HI + self.slot,
            )
        }

        pub(crate) fn z(&self) -> u8 {
            byte(self.ram, TAGALONG_Z + self.slot)
        }

        pub(crate) fn z_signed(&self) -> i8 {
            self.z() as i8
        }

        pub(crate) fn is_above_ground(&self) -> bool {
            self.z_signed() > 0
        }

        pub(crate) fn layer_bits(&self) -> u8 {
            byte(self.ram, TAGALONG_LAYERBITS + self.slot)
        }

        pub(crate) fn direction(&self) -> u8 {
            self.layer_bits() & 3
        }
    }

    pub(crate) struct TagalongSlotViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> TagalongSlotViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_position(
                self.ram,
                TAGALONG_X_LO + self.slot,
                TAGALONG_X_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_position(
                self.ram,
                TAGALONG_Y_LO + self.slot,
                TAGALONG_Y_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_y_high(&mut self, value: u8) {
            self.ram[TAGALONG_Y_HI + self.slot] = value;
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            self.set_x(x);
            self.set_y(y);
        }

        pub(crate) fn set_z(&mut self, value: u8) {
            self.ram[TAGALONG_Z + self.slot] = value;
        }

        pub(crate) fn set_layer_bits(&mut self, value: u8) {
            self.ram[TAGALONG_LAYERBITS + self.slot] = value;
        }
    }

    pub(crate) struct ArrghusPuffHomeView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> ArrghusPuffHomeView<'a> {
        pub(crate) fn new(ram: &'a [u8], puff_slot: usize) -> Self {
            Self {
                ram,
                slot: puff_slot + 7,
            }
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(
                self.ram,
                OVERLORD_X_LO + self.slot,
                OVERLORD_Y_LO + self.slot,
            )
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(
                self.ram,
                OVERLORD_GEN1 + self.slot,
                OVERLORD_GEN3 + self.slot,
            )
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, OVERLORD_X_LO + self.slot)
        }

        pub(crate) fn x_high(&self) -> u8 {
            byte(self.ram, OVERLORD_Y_LO + self.slot)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, OVERLORD_GEN1 + self.slot)
        }

        pub(crate) fn y_high(&self) -> u8 {
            byte(self.ram, OVERLORD_GEN3 + self.slot)
        }
    }

    pub(crate) struct ArmosKnightHomeView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> ArmosKnightHomeView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(
                self.ram,
                OVERLORD_X_HI + self.slot,
                OVERLORD_Y_HI + self.slot,
            )
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(
                self.ram,
                OVERLORD_GEN2 + self.slot,
                OVERLORD_FLOOR + self.slot,
            )
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, OVERLORD_X_HI + self.slot)
        }

        pub(crate) fn x_high(&self) -> u8 {
            byte(self.ram, OVERLORD_Y_HI + self.slot)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, OVERLORD_GEN2 + self.slot)
        }

        pub(crate) fn y_high(&self) -> u8 {
            byte(self.ram, OVERLORD_FLOOR + self.slot)
        }
    }

    pub(crate) struct ArmosKnightHomeViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> ArmosKnightHomeViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_position(&mut self, x: u16, y: u16) {
            write_position(
                self.ram,
                OVERLORD_X_HI + self.slot,
                OVERLORD_Y_HI + self.slot,
                x,
            );
            write_position(
                self.ram,
                OVERLORD_GEN2 + self.slot,
                OVERLORD_FLOOR + self.slot,
                y,
            );
        }
    }

    pub(crate) struct CachedSpriteSlotView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> CachedSpriteSlotView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn state(&self) -> u8 {
            byte(self.ram, ALT_SPRITE_STATE + self.slot)
        }

        pub(crate) fn is_active(&self) -> bool {
            self.state() != 0
        }

        pub(crate) fn type_byte(&self) -> u8 {
            byte(self.ram, ALT_SPRITE_TYPE + self.slot)
        }

        pub(crate) fn y_high(&self) -> u8 {
            byte(self.ram, ALT_SPRITE_Y_HI + self.slot)
        }
    }

    pub(crate) struct CachedSpriteSlotViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> CachedSpriteSlotViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn clear_state(&mut self) {
            self.ram[ALT_SPRITE_STATE + self.slot] = 0;
        }

        pub(crate) fn initialize_trinexx_component(&mut self) {
            self.ram[ALT_SPRITE_TYPE + self.slot] = 0x40;
            self.ram[ALT_SPRITE_X_HI + self.slot] = 0;
            self.ram[ALT_SPRITE_Y_HI + self.slot] = 0;
        }

        pub(crate) fn set_type_byte(&mut self, value: u8) {
            self.ram[ALT_SPRITE_TYPE + self.slot] = value;
        }

        pub(crate) fn set_y_high(&mut self, value: u8) {
            self.ram[ALT_SPRITE_Y_HI + self.slot] = value;
        }

        pub(crate) fn cache_sprite_header(
            &mut self,
            sprite_type: u8,
            x_low: u8,
            x_high: u8,
            y_low: u8,
            y_high: u8,
            graphics: u8,
        ) {
            self.ram[ALT_SPRITE_STATE + self.slot] = 0;
            self.ram[ALT_SPRITE_TYPE + self.slot] = sprite_type;
            self.ram[ALT_SPRITE_X_LO + self.slot] = x_low;
            self.ram[ALT_SPRITE_X_HI + self.slot] = x_high;
            self.ram[ALT_SPRITE_Y_LO + self.slot] = y_low;
            self.ram[ALT_SPRITE_Y_HI + self.slot] = y_high;
            self.ram[ALT_SPRITE_GRAPHICS + self.slot] = graphics;
        }

        pub(crate) fn cache_live_fields(&mut self) {
            for i in 0..CACHED_SPRITE_LIVE_FIELDS.len() {
                self.ram[CACHED_SPRITE_ALT_FIELDS[i] + self.slot] =
                    self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot];
            }
        }

        pub(crate) fn load_cached_into_live(&mut self, backup: &mut [u8; 24]) {
            for i in 0..CACHED_SPRITE_LIVE_FIELDS.len() {
                backup[i] = self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot];
                self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] =
                    self.ram[CACHED_SPRITE_ALT_FIELDS[i] + self.slot];
            }
        }

        pub(crate) fn restore_live_from_backup(&mut self, backup: &[u8; 24]) {
            for i in (0..CACHED_SPRITE_LIVE_FIELDS.len()).rev() {
                self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] = backup[i];
            }
        }
    }

    pub(crate) type AltSpriteSlotViewMut<'a> = CachedSpriteSlotViewMut<'a>;

    pub(crate) struct SpriteSystemView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SpriteSystemView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn limit_instance(&self) -> u8 {
            byte(self.ram, SPRITE_LIMIT_INSTANCE)
        }

        /// Animation counter shared by Blind's head and the tutorial guards.
        pub(crate) fn blind_head_anim_counter(&self) -> u8 {
            byte(self.ram, BLIND_HEAD_ANIM_COUNTER)
        }

        pub(crate) fn chr_halfslot_state(&self) -> u8 {
            byte(self.ram, SPRITE_CHR_HALFSLOT_STATE)
        }

        pub(crate) fn alert_flag(&self) -> u8 {
            byte(self.ram, SPRITE_ALERT_FLAG)
        }

        pub(crate) fn graphics_index(&self) -> u8 {
            byte(self.ram, SPRITE_GRAPHICS_INDEX)
        }

        pub(crate) fn saved_special_exit_graphics_index(&self) -> u8 {
            byte(self.ram, SPRITE_GRAPHICS_INDEX_SPEXIT)
        }

        pub(crate) fn saved_exit_graphics_index(&self) -> u8 {
            byte(self.ram, SPRITE_GRAPHICS_INDEX_EXIT)
        }

        pub(crate) fn alt_sprite_spawned_flag(&self) -> u8 {
            byte(self.ram, ALT_SPRITE_SPAWNED_FLAG)
        }

        pub(crate) fn cur_object_index(&self) -> u8 {
            byte(self.ram, CUR_OBJECT_INDEX)
        }

        pub(crate) fn alt_sprites_flag(&self) -> u8 {
            byte(self.ram, ALT_SPRITES_FLAG)
        }

        pub(crate) fn ranged_based_toggler(&self) -> u8 {
            byte(self.ram, SPR_RANGED_BASED_TOGGLER)
        }

        pub(crate) fn main_tile_theme(&self) -> u8 {
            byte(self.ram, MAIN_TILE_THEME_INDEX)
        }
    }

    pub(crate) struct SpriteSystemViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SpriteSystemViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_limit_instance(&mut self, value: u8) {
            self.ram[SPRITE_LIMIT_INSTANCE] = value;
        }

        pub(crate) fn set_blind_head_anim_counter(&mut self, value: u8) {
            self.ram[BLIND_HEAD_ANIM_COUNTER] = value;
        }

        pub(crate) fn increment_blind_head_anim_counter(&mut self) {
            self.ram[BLIND_HEAD_ANIM_COUNTER] = self.ram[BLIND_HEAD_ANIM_COUNTER].wrapping_add(1);
        }

        pub(crate) fn increment_limit_instance(&mut self) -> u8 {
            self.ram[SPRITE_LIMIT_INSTANCE] = self.ram[SPRITE_LIMIT_INSTANCE].wrapping_add(1);
            self.ram[SPRITE_LIMIT_INSTANCE]
        }

        pub(crate) fn decrement_limit_instance(&mut self) -> u8 {
            self.ram[SPRITE_LIMIT_INSTANCE] = self.ram[SPRITE_LIMIT_INSTANCE].wrapping_sub(1);
            self.ram[SPRITE_LIMIT_INSTANCE]
        }

        pub(crate) fn set_chr_halfslot_state(&mut self, value: u8) {
            self.ram[SPRITE_CHR_HALFSLOT_STATE] = value;
        }

        pub(crate) fn set_alert_flag(&mut self, value: u8) {
            self.ram[SPRITE_ALERT_FLAG] = value;
        }

        pub(crate) fn decrement_alert_flag(&mut self) -> u8 {
            self.ram[SPRITE_ALERT_FLAG] = self.ram[SPRITE_ALERT_FLAG].wrapping_sub(1);
            self.ram[SPRITE_ALERT_FLAG]
        }

        pub(crate) fn set_graphics_index(&mut self, value: u8) {
            self.ram[SPRITE_GRAPHICS_INDEX] = value;
        }

        pub(crate) fn save_special_exit_graphics_index(&mut self) {
            self.ram[SPRITE_GRAPHICS_INDEX_SPEXIT] = self.ram[SPRITE_GRAPHICS_INDEX];
        }

        pub(crate) fn restore_special_exit_graphics_index(&mut self) {
            self.ram[SPRITE_GRAPHICS_INDEX] = self.ram[SPRITE_GRAPHICS_INDEX_SPEXIT];
        }

        pub(crate) fn restore_exit_graphics_index(&mut self) {
            self.ram[SPRITE_GRAPHICS_INDEX] = self.ram[SPRITE_GRAPHICS_INDEX_EXIT];
        }

        pub(crate) fn fill_live_states(&mut self, value: u8) {
            self.ram[SPRITE_STATE..SPRITE_STATE + 16].fill(value);
        }

        pub(crate) fn clear_live_table_pages(&mut self) {
            self.ram[SPRITE_Y_LO..SPRITE_Y_LO + 256 * 3].fill(0);
        }

        pub(crate) fn set_alt_sprite_spawned_flag(&mut self, value: u8) {
            self.ram[ALT_SPRITE_SPAWNED_FLAG] = value;
        }

        pub(crate) fn set_main_tile_theme(&mut self, value: u8) {
            self.ram[MAIN_TILE_THEME_INDEX] = value;
        }

        pub(crate) fn set_aux_tile_theme(&mut self, value: u8) {
            self.ram[AUX_TILE_THEME_INDEX] = value;
        }

        pub(crate) fn set_misc_sprites_graphics_index(&mut self, value: u8) {
            self.ram[MISC_SPRITES_GRAPHICS_INDEX] = value;
        }

        pub(crate) fn set_cur_object_index(&mut self, value: u8) {
            self.ram[CUR_OBJECT_INDEX] = value;
        }

        pub(crate) fn set_alt_sprites_flag(&mut self, value: u8) {
            self.ram[ALT_SPRITES_FLAG] = value;
        }

        pub(crate) fn clear_alt_sprites_flag(&mut self) {
            self.ram[ALT_SPRITES_FLAG] = 0;
        }

        pub(crate) fn increment_ranged_based_toggler(&mut self) {
            self.ram[SPR_RANGED_BASED_TOGGLER] = self.ram[SPR_RANGED_BASED_TOGGLER].wrapping_add(1);
        }
    }

    pub(crate) struct SpriteWorkspaceView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SpriteWorkspaceView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn room_origin_x_high(&self) -> u8 {
            byte(self.ram, SPRITE_ROOM_ORIGIN_X_HI)
        }

        pub(crate) fn room_origin_y_high(&self) -> u8 {
            byte(self.ram, SPRITE_ROOM_ORIGIN_Y_HI)
        }

        pub(crate) fn pickup_slot_cache(&self) -> u8 {
            byte(self.ram, SPRITE_PICKUP_SLOT_CACHE)
        }

        pub(crate) fn shared_scratch_a(&self) -> u8 {
            byte(self.ram, SPRITE_SHARED_SCRATCH_A)
        }

        pub(crate) fn tile_type(&self) -> u8 {
            byte(self.ram, SPRITE_TILETYPE)
        }

        pub(crate) fn prep_shared_counter(&self) -> u8 {
            byte(self.ram, SPRITE_RESET_SCRATCH_A)
        }

        pub(crate) fn reset_scratch_a(&self) -> u8 {
            byte(self.ram, SPRITE_RESET_SCRATCH_A)
        }

        /// Alias of `reset_scratch_a`: the Armos Knights fight reuses the
        /// shared scratch byte as the remaining-knight counter.
        pub(crate) fn armos_knight_remaining_count(&self) -> u8 {
            byte(self.ram, SPRITE_RESET_SCRATCH_A)
        }

        pub(crate) fn reset_scratch_b(&self) -> u8 {
            byte(self.ram, SPRITE_RESET_SCRATCH_B)
        }

        pub(crate) fn graphics_subset(&self, slot: usize) -> u8 {
            byte(self.ram, SPRITE_GFX_SUBSET_0 + slot)
        }

        pub(crate) fn draw_priority_override(&self) -> u16 {
            read_le_u16(self.ram, SPRITE_DRAW_PRIORITY_OVERRIDE)
        }

        pub(crate) fn current_sprite_x(&self) -> u16 {
            read_le_u16(self.ram, CUR_SPRITE_X)
        }

        pub(crate) fn current_sprite_x_low(&self) -> u8 {
            byte(self.ram, CUR_SPRITE_X)
        }

        pub(crate) fn current_sprite_y(&self) -> u16 {
            read_le_u16(self.ram, CUR_SPRITE_Y)
        }

        pub(crate) fn current_sprite_y_low(&self) -> u8 {
            byte(self.ram, CUR_SPRITE_Y)
        }

        pub(crate) fn oam_prep_x(&self) -> u16 {
            read_le_u16(self.ram, SPRITE_OAM_PREP_X)
        }

        pub(crate) fn oam_prep_y(&self) -> u16 {
            read_le_u16(self.ram, SPRITE_OAM_PREP_Y)
        }

        pub(crate) fn where_in_room(&self, room: usize) -> u16 {
            read_le_u16(self.ram, SPRITE_WHERE_IN_ROOM + room * 2)
        }
    }

    pub(crate) struct SpriteWorkspaceViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SpriteWorkspaceViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_room_origin_x_high(&mut self, value: u8) {
            self.ram[SPRITE_ROOM_ORIGIN_X_HI] = value;
        }

        pub(crate) fn set_room_origin_y_high(&mut self, value: u8) {
            self.ram[SPRITE_ROOM_ORIGIN_Y_HI] = value;
        }

        pub(crate) fn set_pickup_slot_cache(&mut self, value: u8) {
            self.ram[SPRITE_PICKUP_SLOT_CACHE] = value;
        }

        pub(crate) fn set_shared_scratch_a(&mut self, value: u8) {
            self.ram[SPRITE_SHARED_SCRATCH_A] = value;
        }

        pub(crate) fn set_tile_type(&mut self, value: u8) {
            self.ram[SPRITE_TILETYPE] = value;
        }

        pub(crate) fn set_prep_shared_counter(&mut self, value: u8) {
            self.ram[SPRITE_RESET_SCRATCH_A] = value;
        }

        pub(crate) fn increment_prep_shared_counter(&mut self) -> u8 {
            self.ram[SPRITE_RESET_SCRATCH_A] = self.ram[SPRITE_RESET_SCRATCH_A].wrapping_add(1);
            self.ram[SPRITE_RESET_SCRATCH_A]
        }

        pub(crate) fn decrement_prep_shared_counter(&mut self) -> u8 {
            self.ram[SPRITE_RESET_SCRATCH_A] = self.ram[SPRITE_RESET_SCRATCH_A].wrapping_sub(1);
            self.ram[SPRITE_RESET_SCRATCH_A]
        }

        /// Alias of `decrement_prep_shared_counter` for the Armos Knights
        /// fight, which reuses the shared scratch byte as the
        /// remaining-knight counter. Returns the new value.
        pub(crate) fn decrement_armos_knight_remaining_count(&mut self) -> u8 {
            self.ram[SPRITE_RESET_SCRATCH_A] = self.ram[SPRITE_RESET_SCRATCH_A].wrapping_sub(1);
            self.ram[SPRITE_RESET_SCRATCH_A]
        }

        /// Alias of `set_prep_shared_counter(0)` for the Vitreous fight,
        /// which reuses the shared scratch byte as the eyeball release
        /// counter.
        pub(crate) fn clear_vitreous_eyeball_release_count(&mut self) {
            self.ram[SPRITE_RESET_SCRATCH_A] = 0;
        }

        pub(crate) fn set_reset_scratch_a(&mut self, value: u8) {
            self.ram[SPRITE_RESET_SCRATCH_A] = value;
        }

        pub(crate) fn set_reset_scratch_b(&mut self, value: u8) {
            self.ram[SPRITE_RESET_SCRATCH_B] = value;
        }

        pub(crate) fn set_graphics_subset(&mut self, slot: usize, value: u8) {
            self.ram[SPRITE_GFX_SUBSET_0 + slot] = value;
        }

        pub(crate) fn clear_where_in_room(&mut self) {
            self.ram[SPRITE_WHERE_IN_ROOM..SPRITE_WHERE_IN_ROOM + 0x1000].fill(0);
        }

        pub(crate) fn clear_draw_priority_override(&mut self) {
            write_le_u16(self.ram, SPRITE_DRAW_PRIORITY_OVERRIDE, 0);
        }

        pub(crate) fn set_draw_priority_override_low(&mut self, value: u8) {
            self.ram[SPRITE_DRAW_PRIORITY_OVERRIDE] = value;
        }

        pub(crate) fn set_current_sprite_x(&mut self, value: u16) {
            write_le_u16(self.ram, CUR_SPRITE_X, value);
        }

        pub(crate) fn set_current_sprite_x_low(&mut self, value: u8) {
            self.ram[CUR_SPRITE_X] = value;
        }

        pub(crate) fn add_current_sprite_x_low(&mut self, value: u8) {
            self.ram[CUR_SPRITE_X] = self.ram[CUR_SPRITE_X].wrapping_add(value);
        }

        pub(crate) fn set_current_sprite_y(&mut self, value: u16) {
            write_le_u16(self.ram, CUR_SPRITE_Y, value);
        }

        pub(crate) fn set_current_sprite_y_low(&mut self, value: u8) {
            self.ram[CUR_SPRITE_Y] = value;
        }

        pub(crate) fn add_current_sprite_y_low(&mut self, value: u8) {
            self.ram[CUR_SPRITE_Y] = self.ram[CUR_SPRITE_Y].wrapping_add(value);
        }

        pub(crate) fn subtract_current_sprite_y_low(&mut self, value: u8) {
            self.ram[CUR_SPRITE_Y] = self.ram[CUR_SPRITE_Y].wrapping_sub(value);
        }

        pub(crate) fn set_current_sprite_position(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, CUR_SPRITE_X, x);
            write_le_u16(self.ram, CUR_SPRITE_Y, y);
        }

        pub(crate) fn set_oam_prep_coords(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, SPRITE_OAM_PREP_X, x);
            write_le_u16(self.ram, SPRITE_OAM_PREP_Y, y);
        }

        pub(crate) fn set_killed_sprite_load_block(&mut self, block: u16) {
            self.ram[SPRITE_LOAD_BLOCK_SCRATCH] = block as u8;
            write_le_u16(
                self.ram,
                SPRITE_LOAD_BLOCK_SCRATCH + 1,
                (block >> 3).wrapping_add(0xef80),
            );
        }

        pub(crate) fn set_last_garnish_index(&mut self, index: i32) {
            self.ram[SPRITE_LAST_GARNISH_INDEX] = index as u8;
        }

        pub(crate) fn set_where_in_room(&mut self, room: usize, value: u16) {
            write_le_u16(self.ram, SPRITE_WHERE_IN_ROOM + room * 2, value);
        }
    }

    pub(crate) struct EnemyDamageDataView<'a> {
        ram: &'a [u8],
    }

    impl<'a> EnemyDamageDataView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn entry(&self, index: usize) -> u8 {
            byte(self.ram, ENEMY_DAMAGE_DATA + index)
        }
    }

    pub(crate) struct EnemyDamageDataViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> EnemyDamageDataViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_entry(&mut self, index: usize, value: u8) {
            self.ram[ENEMY_DAMAGE_DATA + index] = value;
        }
    }

    pub(crate) struct EtherOrbitView<'a> {
        ram: &'a [u8],
    }

    impl<'a> EtherOrbitView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn angle(&self, slot: usize) -> u8 {
            byte(self.ram, ETHER_ANGLE + slot)
        }

        pub(crate) fn radius(&self) -> u8 {
            byte(self.ram, ETHER_RADIUS)
        }

        pub(crate) fn beam_top_bucket(&self) -> u8 {
            byte(self.ram, ETHER_BEAM_TOP_BUCKET)
        }

        pub(crate) fn beam_y(&self) -> u16 {
            read_le_u16(self.ram, ETHER_BEAM_Y)
        }

        pub(crate) fn orbit_x(&self) -> u16 {
            read_le_u16(self.ram, ETHER_ORBIT_X)
        }

        pub(crate) fn orbit_y(&self) -> u16 {
            read_le_u16(self.ram, ETHER_ORBIT_Y)
        }

        pub(crate) fn orb_x(&self) -> u16 {
            read_le_u16(self.ram, ETHER_ORB_X)
        }

        pub(crate) fn orb_y(&self) -> u16 {
            read_le_u16(self.ram, ETHER_ORB_Y)
        }
    }

    pub(crate) struct EtherOrbitViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> EtherOrbitViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_angle(&mut self, slot: usize, value: u8) {
            self.ram[ETHER_ANGLE + slot] = value;
        }

        pub(crate) fn advance_angle(&mut self, slot: usize) -> u8 {
            let next = self.ram[ETHER_ANGLE + slot].wrapping_add(1) & 0x3f;
            self.ram[ETHER_ANGLE + slot] = next;
            next
        }

        pub(crate) fn set_radius(&mut self, value: u8) {
            self.ram[ETHER_RADIUS] = value;
        }

        pub(crate) fn tick_spin_countdown(&mut self) -> u8 {
            let value = self.ram[ETHER_SPIN_COUNTDOWN].wrapping_sub(1);
            self.ram[ETHER_SPIN_COUNTDOWN] = value;
            value
        }

        pub(crate) fn set_spin_countdown(&mut self, value: u8) {
            self.ram[ETHER_SPIN_COUNTDOWN] = value;
        }

        pub(crate) fn set_beam_top_bucket(&mut self, value: u8) {
            self.ram[ETHER_BEAM_TOP_BUCKET] = value;
        }

        pub(crate) fn initialize_beam_adjusted_y(&mut self, value: u16) {
            write_le_u16(self.ram, ETHER_BEAM_TOP_BUCKET, value);
        }

        pub(crate) fn set_orb_position(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, ETHER_ORB_X, x);
            write_le_u16(self.ram, ETHER_ORB_Y, y);
        }

        pub(crate) fn set_orbit_position(&mut self, x: u16, y: u16) {
            write_le_u16(self.ram, ETHER_ORBIT_X, x);
            write_le_u16(self.ram, ETHER_ORBIT_Y, y);
        }

        pub(crate) fn set_beam_y(&mut self, value: u16) {
            write_le_u16(self.ram, ETHER_BEAM_Y, value);
        }
    }

    pub(crate) struct PrizeDropCycleViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PrizeDropCycleViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn take_next_index(&mut self, slot: usize) -> u8 {
            let index = byte(self.ram, PRIZE_DROP_CYCLE + slot);
            self.ram[PRIZE_DROP_CYCLE + slot] = index.wrapping_add(1) & 7;
            index
        }
    }

    pub(crate) struct DualLayerTileCacheView<'a> {
        ram: &'a [u8],
    }

    impl<'a> DualLayerTileCacheView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn tile_type(&self, slot: usize) -> u8 {
            byte(self.ram, DUAL_LAYER_TILE_CACHE + slot)
        }
    }

    pub(crate) struct DualLayerTileCacheViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> DualLayerTileCacheViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_tile_type(&mut self, slot: usize, value: u8) {
            self.ram[DUAL_LAYER_TILE_CACHE + slot] = value;
        }
    }

    pub(crate) struct SpriteSlotView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> SpriteSlotView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn slot(&self) -> u8 {
            self.slot as u8
        }

        pub(crate) fn sprite_type(&self) -> u8 {
            byte(self.ram, SPRITE_TYPE + self.slot)
        }

        pub(crate) fn state(&self) -> u8 {
            byte(self.ram, SPRITE_STATE + self.slot)
        }

        pub(crate) fn is_active(&self) -> bool {
            self.sprite_type() != 0 || self.state() != 0
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(self.ram, SPRITE_X_LO + self.slot, SPRITE_X_HI + self.slot)
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, SPRITE_X_LO + self.slot)
        }

        pub(crate) fn x_high(&self) -> u8 {
            byte(self.ram, SPRITE_X_HI + self.slot)
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(self.ram, SPRITE_Y_LO + self.slot, SPRITE_Y_HI + self.slot)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, SPRITE_Y_LO + self.slot)
        }

        pub(crate) fn y_high(&self) -> u8 {
            byte(self.ram, SPRITE_Y_HI + self.slot)
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            byte(self.ram, SPRITE_X_VELOCITY + self.slot)
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            byte(self.ram, SPRITE_Y_VELOCITY + self.slot)
        }

        pub(crate) fn z_velocity(&self) -> u8 {
            byte(self.ram, SPRITE_Z_VELOCITY + self.slot)
        }

        pub(crate) fn x_recoil(&self) -> u8 {
            byte(self.ram, SPRITE_X_RECOIL + self.slot)
        }

        pub(crate) fn y_recoil(&self) -> u8 {
            byte(self.ram, SPRITE_Y_RECOIL + self.slot)
        }

        pub(crate) fn x_subpixel(&self) -> u8 {
            byte(self.ram, SPRITE_X_SUBPIXEL + self.slot)
        }

        pub(crate) fn y_subpixel(&self) -> u8 {
            byte(self.ram, SPRITE_Y_SUBPIXEL + self.slot)
        }

        pub(crate) fn z(&self) -> u8 {
            byte(self.ram, SPRITE_Z + self.slot)
        }

        pub(crate) fn z_subpixel(&self) -> u8 {
            byte(self.ram, SPRITE_Z_SUBPIXEL + self.slot)
        }

        pub(crate) fn ai_state(&self) -> u8 {
            byte(self.ram, SPRITE_AI_STATE + self.slot)
        }

        pub(crate) fn a(&self) -> u8 {
            byte(self.ram, SPRITE_A + self.slot)
        }

        pub(crate) fn c(&self) -> u8 {
            byte(self.ram, SPRITE_C + self.slot)
        }

        pub(crate) fn b(&self) -> u8 {
            byte(self.ram, SPRITE_B + self.slot)
        }

        pub(crate) fn e(&self) -> u8 {
            byte(self.ram, SPRITE_E + self.slot)
        }

        pub(crate) fn f(&self) -> u8 {
            byte(self.ram, SPRITE_F + self.slot)
        }

        pub(crate) fn g(&self) -> u8 {
            byte(self.ram, SPRITE_G + self.slot)
        }

        pub(crate) fn graphics(&self) -> u8 {
            byte(self.ram, SPRITE_GRAPHICS + self.slot)
        }

        pub(crate) fn direction(&self) -> u8 {
            byte(self.ram, SPRITE_D + self.slot)
        }

        pub(crate) fn subtype(&self) -> u8 {
            byte(self.ram, SPRITE_SUBTYPE + self.slot)
        }

        pub(crate) fn delay_main(&self) -> u8 {
            byte(self.ram, SPRITE_DELAY_MAIN + self.slot)
        }

        pub(crate) fn delay_aux1(&self) -> u8 {
            byte(self.ram, SPRITE_DELAY_AUX1 + self.slot)
        }

        pub(crate) fn delay_aux4(&self) -> u8 {
            byte(self.ram, SPRITE_DELAY_AUX4 + self.slot)
        }

        pub(crate) fn delay_aux2(&self) -> u8 {
            byte(self.ram, SPRITE_DELAY_AUX2 + self.slot)
        }

        pub(crate) fn flags2(&self) -> u8 {
            byte(self.ram, SPRITE_FLAGS2 + self.slot)
        }

        pub(crate) fn flags(&self) -> u8 {
            byte(self.ram, SPRITE_FLAGS + self.slot)
        }

        pub(crate) fn flags3(&self) -> u8 {
            byte(self.ram, SPRITE_FLAGS3 + self.slot)
        }

        pub(crate) fn wall_collision(&self) -> u8 {
            byte(self.ram, SPRITE_WALL_COLLISION + self.slot)
        }

        pub(crate) fn anim_clock(&self) -> u8 {
            byte(self.ram, SPRITE_ANIM_CLOCK + self.slot)
        }

        pub(crate) fn delay_aux3(&self) -> u8 {
            byte(self.ram, SPRITE_DELAY_AUX3 + self.slot)
        }

        pub(crate) fn flags4(&self) -> u8 {
            byte(self.ram, SPRITE_FLAGS4 + self.slot)
        }

        pub(crate) fn flags5(&self) -> u8 {
            byte(self.ram, SPRITE_FLAGS5 + self.slot)
        }

        pub(crate) fn health(&self) -> u8 {
            byte(self.ram, SPRITE_HEALTH + self.slot)
        }

        pub(crate) fn hit_timer(&self) -> u8 {
            byte(self.ram, SPRITE_HIT_TIMER + self.slot)
        }

        pub(crate) fn pause(&self) -> u8 {
            byte(self.ram, SPRITE_PAUSE + self.slot)
        }

        pub(crate) fn stunned(&self) -> u8 {
            byte(self.ram, SPRITE_STUNNED + self.slot)
        }

        pub(crate) fn ignore_projectile(&self) -> u8 {
            byte(self.ram, SPRITE_IGNORE_PROJECTILE + self.slot)
        }

        pub(crate) fn draw_work_byte_2(&self) -> u8 {
            byte(self.ram, SPRITE_DRAW_WORK_BYTE_2 + self.slot)
        }

        pub(crate) fn n(&self) -> u8 {
            byte(self.ram, SPRITE_N + self.slot)
        }

        pub(crate) fn n_word(&self) -> u16 {
            read_le_u16(self.ram, SPRITE_N + self.slot * 2)
        }

        pub(crate) fn deflection_bits(&self) -> u8 {
            byte(self.ram, SPRITE_DEFL_BITS + self.slot)
        }

        pub(crate) fn bump_damage(&self) -> u8 {
            byte(self.ram, SPRITE_BUMP_DAMAGE + self.slot)
        }

        pub(crate) fn incoming_damage(&self) -> u8 {
            byte(self.ram, SPRITE_INCOMING_DAMAGE + self.slot)
        }

        pub(crate) fn floor(&self) -> u8 {
            byte(self.ram, SPRITE_FLOOR + self.slot)
        }

        pub(crate) fn room(&self) -> u8 {
            byte(self.ram, SPRITE_ROOM + self.slot)
        }

        pub(crate) fn die_action(&self) -> u8 {
            byte(self.ram, SPRITE_DIE_ACTION + self.slot)
        }

        pub(crate) fn draw_i(&self) -> u8 {
            byte(self.ram, SPRITE_DRAW_I + self.slot)
        }

        pub(crate) fn draw_work_byte_3(&self) -> u8 {
            byte(self.ram, SPRITE_DRAW_WORK_BYTE_3 + self.slot)
        }

        pub(crate) fn draw_work_byte_4(&self) -> u8 {
            byte(self.ram, SPRITE_DRAW_WORK_BYTE_4 + self.slot)
        }

        pub(crate) fn draw_work_byte_5(&self) -> u8 {
            byte(self.ram, SPRITE_DRAW_WORK_BYTE_5 + self.slot)
        }

        pub(crate) fn draw_work_byte_1(&self) -> u8 {
            byte(self.ram, SPRITE_DRAW_WORK_BYTE_1 + self.slot)
        }

        pub(crate) fn head_direction(&self) -> u8 {
            byte(self.ram, SPRITE_HEAD_DIR + self.slot)
        }

        pub(crate) fn oam_flags(&self) -> u8 {
            byte(self.ram, SPRITE_OAM_FLAGS + self.slot)
        }

        pub(crate) fn object_priority(&self) -> u8 {
            byte(self.ram, SPRITE_OBJ_PRIO + self.slot)
        }

        pub(crate) fn subtype2(&self) -> u8 {
            byte(self.ram, SPRITE_SUBTYPE2 + self.slot)
        }
    }

    pub(crate) struct SpriteSlotViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> SpriteSlotViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_sprite_type(&mut self, value: u8) {
            self.ram[SPRITE_TYPE + self.slot] = value;
        }

        pub(crate) fn set_state(&mut self, value: u8) {
            self.ram[SPRITE_STATE + self.slot] = value;
        }

        pub(crate) fn increment_state(&mut self) {
            self.ram[SPRITE_STATE + self.slot] = self.ram[SPRITE_STATE + self.slot].wrapping_add(1);
        }

        pub(crate) fn clear(&mut self) {
            self.set_state(0);
        }

        pub(crate) fn clear_prep_runtime_state(&mut self) {
            for base in [
                SPRITE_PAUSE,
                SPRITE_E,
                SPRITE_X_VELOCITY,
                SPRITE_Y_VELOCITY,
                SPRITE_Z_VELOCITY,
                SPRITE_X_SUBPIXEL,
                SPRITE_Y_SUBPIXEL,
                SPRITE_Z_SUBPIXEL,
                SPRITE_AI_STATE,
                SPRITE_GRAPHICS,
                SPRITE_D,
                SPRITE_DELAY_MAIN,
                SPRITE_DELAY_AUX1,
                SPRITE_DELAY_AUX2,
                SPRITE_DELAY_AUX4,
                SPRITE_HEAD_DIR,
                SPRITE_ANIM_CLOCK,
                SPRITE_G,
                SPRITE_HIT_TIMER,
                SPRITE_WALL_COLLISION,
                SPRITE_Z,
                SPRITE_HEALTH,
                SPRITE_F,
                SPRITE_X_RECOIL,
                SPRITE_Y_RECOIL,
                SPRITE_A,
                SPRITE_B,
                SPRITE_C,
                SPRITE_DRAW_WORK_BYTE_2,
                SPRITE_SUBTYPE2,
                SPRITE_IGNORE_PROJECTILE,
                SPRITE_OBJ_PRIO,
                SPRITE_OAM_FLAGS,
                SPRITE_STUNNED,
                SPRITE_INCOMING_DAMAGE,
                SPRITE_DRAW_WORK_BYTE_3,
                SPRITE_DRAW_WORK_BYTE_4,
                SPRITE_DRAW_WORK_BYTE_5,
                SPRITE_DRAW_WORK_BYTE_1,
                SPRITE_DRAW_I,
            ] {
                self.ram[base + self.slot] = 0;
            }
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_position(
                self.ram,
                SPRITE_X_LO + self.slot,
                SPRITE_X_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_x_low(&mut self, value: u8) {
            self.ram[SPRITE_X_LO + self.slot] = value;
        }

        pub(crate) fn set_x_high(&mut self, value: u8) {
            self.ram[SPRITE_X_HI + self.slot] = value;
        }

        pub(crate) fn add_x_low(&mut self, value: u8) {
            self.ram[SPRITE_X_LO + self.slot] =
                self.ram[SPRITE_X_LO + self.slot].wrapping_add(value);
        }

        pub(crate) fn subtract_x_low(&mut self, value: u8) {
            self.ram[SPRITE_X_LO + self.slot] =
                self.ram[SPRITE_X_LO + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_position(
                self.ram,
                SPRITE_Y_LO + self.slot,
                SPRITE_Y_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_y_high(&mut self, value: u8) {
            self.ram[SPRITE_Y_HI + self.slot] = value;
        }

        pub(crate) fn add_y_low(&mut self, value: u8) {
            self.ram[SPRITE_Y_LO + self.slot] =
                self.ram[SPRITE_Y_LO + self.slot].wrapping_add(value);
        }

        pub(crate) fn subtract_y_low(&mut self, value: u8) {
            self.ram[SPRITE_Y_LO + self.slot] =
                self.ram[SPRITE_Y_LO + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_y_low(&mut self, value: u8) {
            self.ram[SPRITE_Y_LO + self.slot] = value;
        }

        pub(crate) fn set_x_velocity(&mut self, value: u8) {
            self.ram[SPRITE_X_VELOCITY + self.slot] = value;
        }

        pub(crate) fn set_y_velocity(&mut self, value: u8) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] = value;
        }

        pub(crate) fn set_x_subpixel(&mut self, value: u8) {
            self.ram[SPRITE_X_SUBPIXEL + self.slot] = value;
        }

        pub(crate) fn set_y_subpixel(&mut self, value: u8) {
            self.ram[SPRITE_Y_SUBPIXEL + self.slot] = value;
        }

        pub(crate) fn set_z_velocity(&mut self, value: u8) {
            self.ram[SPRITE_Z_VELOCITY + self.slot] = value;
        }

        pub(crate) fn decrement_z_velocity(&mut self) {
            self.ram[SPRITE_Z_VELOCITY + self.slot] =
                self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_sub(1);
        }

        pub(crate) fn set_delay_aux4(&mut self, value: u8) {
            self.ram[SPRITE_DELAY_AUX4 + self.slot] = value;
        }

        pub(crate) fn set_delay_aux1(&mut self, value: u8) {
            self.ram[SPRITE_DELAY_AUX1 + self.slot] = value;
        }

        pub(crate) fn set_delay_aux2(&mut self, value: u8) {
            self.ram[SPRITE_DELAY_AUX2 + self.slot] = value;
        }

        pub(crate) fn set_delay_aux3(&mut self, value: u8) {
            self.ram[SPRITE_DELAY_AUX3 + self.slot] = value;
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            self.ram[SPRITE_X_VELOCITY + self.slot]
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            self.ram[SPRITE_Y_VELOCITY + self.slot]
        }

        pub(crate) fn add_x_velocity(&mut self, value: u8) {
            self.ram[SPRITE_X_VELOCITY + self.slot] =
                self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_add(value);
        }

        pub(crate) fn add_y_velocity(&mut self, value: u8) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] =
                self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_add(value);
        }

        pub(crate) fn subtract_x_velocity(&mut self, value: u8) {
            self.ram[SPRITE_X_VELOCITY + self.slot] =
                self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_sub(value);
        }

        pub(crate) fn subtract_y_velocity(&mut self, value: u8) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] =
                self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_sub(value);
        }

        pub(crate) fn xor_x_velocity(&mut self, value: u8) {
            self.ram[SPRITE_X_VELOCITY + self.slot] ^= value;
        }

        pub(crate) fn xor_y_velocity(&mut self, value: u8) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] ^= value;
        }

        pub(crate) fn add_z_velocity(&mut self, value: u8) {
            self.ram[SPRITE_Z_VELOCITY + self.slot] =
                self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_add(value);
        }

        pub(crate) fn negate_z_velocity(&mut self) {
            self.ram[SPRITE_Z_VELOCITY + self.slot] =
                self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_neg();
        }

        pub(crate) fn and_x_velocity(&mut self, value: u8) {
            self.ram[SPRITE_X_VELOCITY + self.slot] &= value;
        }

        pub(crate) fn and_y_velocity(&mut self, value: u8) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] &= value;
        }

        pub(crate) fn shift_x_velocity_left(&mut self, amount: u32) {
            self.ram[SPRITE_X_VELOCITY + self.slot] =
                self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_shl(amount);
        }

        pub(crate) fn shift_y_velocity_left(&mut self, amount: u32) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] =
                self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_shl(amount);
        }

        pub(crate) fn negate_x_velocity(&mut self) {
            self.ram[SPRITE_X_VELOCITY + self.slot] =
                self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_neg();
        }

        pub(crate) fn negate_y_velocity(&mut self) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] =
                self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_neg();
        }

        pub(crate) fn subtract_z_velocity(&mut self, value: u8) {
            self.ram[SPRITE_Z_VELOCITY + self.slot] =
                self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_sub(value);
        }

        pub(crate) fn halve_x_velocity(&mut self) {
            self.ram[SPRITE_X_VELOCITY + self.slot] =
                ((self.ram[SPRITE_X_VELOCITY + self.slot] as i8) >> 1) as u8;
        }

        pub(crate) fn halve_y_velocity(&mut self) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] =
                ((self.ram[SPRITE_Y_VELOCITY + self.slot] as i8) >> 1) as u8;
        }

        pub(crate) fn set_z(&mut self, value: u8) {
            self.ram[SPRITE_Z + self.slot] = value;
        }

        pub(crate) fn add_z(&mut self, value: u8) {
            self.ram[SPRITE_Z + self.slot] = self.ram[SPRITE_Z + self.slot].wrapping_add(value);
        }

        pub(crate) fn set_z_subpixel(&mut self, value: u8) {
            self.ram[SPRITE_Z_SUBPIXEL + self.slot] = value;
        }

        pub(crate) fn negate_z_subpixel(&mut self) {
            self.ram[SPRITE_Z_SUBPIXEL + self.slot] =
                self.ram[SPRITE_Z_SUBPIXEL + self.slot].wrapping_neg();
        }

        pub(crate) fn increment_z_subpixel(&mut self) -> u8 {
            self.ram[SPRITE_Z_SUBPIXEL + self.slot] =
                self.ram[SPRITE_Z_SUBPIXEL + self.slot].wrapping_add(1);
            self.ram[SPRITE_Z_SUBPIXEL + self.slot]
        }

        pub(crate) fn decrement_z_subpixel(&mut self) -> u8 {
            self.ram[SPRITE_Z_SUBPIXEL + self.slot] =
                self.ram[SPRITE_Z_SUBPIXEL + self.slot].wrapping_sub(1);
            self.ram[SPRITE_Z_SUBPIXEL + self.slot]
        }

        pub(crate) fn set_ai_state(&mut self, value: u8) {
            self.ram[SPRITE_AI_STATE + self.slot] = value;
        }

        pub(crate) fn increment_ai_state(&mut self) {
            self.ram[SPRITE_AI_STATE + self.slot] =
                self.ram[SPRITE_AI_STATE + self.slot].wrapping_add(1);
        }

        pub(crate) fn decrement_ai_state(&mut self) {
            self.ram[SPRITE_AI_STATE + self.slot] =
                self.ram[SPRITE_AI_STATE + self.slot].wrapping_sub(1);
        }

        pub(crate) fn add_ai_state(&mut self, value: u8) {
            self.ram[SPRITE_AI_STATE + self.slot] =
                self.ram[SPRITE_AI_STATE + self.slot].wrapping_add(value);
        }

        pub(crate) fn subtract_ai_state(&mut self, value: u8) {
            self.ram[SPRITE_AI_STATE + self.slot] =
                self.ram[SPRITE_AI_STATE + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_a(&mut self, value: u8) {
            self.ram[SPRITE_A + self.slot] = value;
        }

        pub(crate) fn increment_a(&mut self) {
            self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_add(1);
        }

        pub(crate) fn xor_a(&mut self, value: u8) {
            self.ram[SPRITE_A + self.slot] ^= value;
        }

        pub(crate) fn add_a(&mut self, value: u8) {
            self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_add(value);
        }

        pub(crate) fn decrement_a(&mut self) {
            self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_sub(1);
        }

        pub(crate) fn subtract_a(&mut self, value: u8) {
            self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_b(&mut self, value: u8) {
            self.ram[SPRITE_B + self.slot] = value;
        }

        pub(crate) fn increment_b(&mut self) {
            self.ram[SPRITE_B + self.slot] = self.ram[SPRITE_B + self.slot].wrapping_add(1);
        }

        pub(crate) fn add_b(&mut self, value: u8) {
            self.ram[SPRITE_B + self.slot] = self.ram[SPRITE_B + self.slot].wrapping_add(value);
        }

        pub(crate) fn subtract_b(&mut self, value: u8) {
            self.ram[SPRITE_B + self.slot] = self.ram[SPRITE_B + self.slot].wrapping_sub(value);
        }

        pub(crate) fn xor_b(&mut self, value: u8) {
            self.ram[SPRITE_B + self.slot] ^= value;
        }

        pub(crate) fn set_c(&mut self, value: u8) {
            self.ram[SPRITE_C + self.slot] = value;
        }

        pub(crate) fn increment_c(&mut self) {
            self.ram[SPRITE_C + self.slot] = self.ram[SPRITE_C + self.slot].wrapping_add(1);
        }

        pub(crate) fn add_c(&mut self, value: u8) {
            self.ram[SPRITE_C + self.slot] = self.ram[SPRITE_C + self.slot].wrapping_add(value);
        }

        pub(crate) fn xor_c(&mut self, value: u8) {
            self.ram[SPRITE_C + self.slot] ^= value;
        }

        pub(crate) fn decrement_c(&mut self) {
            self.ram[SPRITE_C + self.slot] = self.ram[SPRITE_C + self.slot].wrapping_sub(1);
        }

        pub(crate) fn set_e(&mut self, value: u8) {
            self.ram[SPRITE_E + self.slot] = value;
        }

        pub(crate) fn increment_e(&mut self) -> u8 {
            self.ram[SPRITE_E + self.slot] = self.ram[SPRITE_E + self.slot].wrapping_add(1);
            self.ram[SPRITE_E + self.slot]
        }

        pub(crate) fn add_e(&mut self, value: u8) {
            self.ram[SPRITE_E + self.slot] = self.ram[SPRITE_E + self.slot].wrapping_add(value);
        }

        pub(crate) fn decrement_e(&mut self) -> u8 {
            self.ram[SPRITE_E + self.slot] = self.ram[SPRITE_E + self.slot].wrapping_sub(1);
            self.ram[SPRITE_E + self.slot]
        }

        pub(crate) fn set_direction(&mut self, value: u8) {
            self.ram[SPRITE_D + self.slot] = value;
        }

        pub(crate) fn add_direction(&mut self, value: u8) {
            self.ram[SPRITE_D + self.slot] = self.ram[SPRITE_D + self.slot].wrapping_add(value);
        }

        pub(crate) fn and_direction(&mut self, value: u8) {
            self.ram[SPRITE_D + self.slot] &= value;
        }

        pub(crate) fn increment_direction(&mut self) {
            self.ram[SPRITE_D + self.slot] = self.ram[SPRITE_D + self.slot].wrapping_add(1);
        }

        pub(crate) fn xor_direction(&mut self, value: u8) {
            self.ram[SPRITE_D + self.slot] ^= value;
        }

        pub(crate) fn set_delay_main(&mut self, value: u8) {
            self.ram[SPRITE_DELAY_MAIN + self.slot] = value;
        }

        pub(crate) fn increment_delay_main(&mut self) {
            self.ram[SPRITE_DELAY_MAIN + self.slot] =
                self.ram[SPRITE_DELAY_MAIN + self.slot].wrapping_add(1);
        }

        pub(crate) fn add_delay_main(&mut self, value: u8) {
            self.ram[SPRITE_DELAY_MAIN + self.slot] =
                self.ram[SPRITE_DELAY_MAIN + self.slot].wrapping_add(value);
        }

        pub(crate) fn subtract_delay_main(&mut self, value: u8) {
            self.ram[SPRITE_DELAY_MAIN + self.slot] =
                self.ram[SPRITE_DELAY_MAIN + self.slot].wrapping_sub(value);
        }

        pub(crate) fn halve_delay_main(&mut self) {
            self.ram[SPRITE_DELAY_MAIN + self.slot] >>= 1;
        }

        pub(crate) fn set_head_direction(&mut self, value: u8) {
            self.ram[SPRITE_HEAD_DIR + self.slot] = value;
        }

        pub(crate) fn increment_head_direction(&mut self) {
            self.ram[SPRITE_HEAD_DIR + self.slot] =
                self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_add(1);
        }

        pub(crate) fn add_head_direction(&mut self, value: u8) {
            self.ram[SPRITE_HEAD_DIR + self.slot] =
                self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_add(value);
        }

        pub(crate) fn decrement_head_direction(&mut self) {
            self.ram[SPRITE_HEAD_DIR + self.slot] =
                self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_sub(1);
        }

        pub(crate) fn increment_head_direction_mod16(&mut self) {
            self.ram[SPRITE_HEAD_DIR + self.slot] =
                self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_add(1) & 15;
        }

        pub(crate) fn set_graphics(&mut self, value: u8) {
            self.ram[SPRITE_GRAPHICS + self.slot] = value;
        }

        pub(crate) fn increment_graphics(&mut self) {
            self.ram[SPRITE_GRAPHICS + self.slot] =
                self.ram[SPRITE_GRAPHICS + self.slot].wrapping_add(1);
        }

        pub(crate) fn add_graphics(&mut self, value: u8) {
            self.ram[SPRITE_GRAPHICS + self.slot] =
                self.ram[SPRITE_GRAPHICS + self.slot].wrapping_add(value);
        }

        pub(crate) fn decrement_graphics(&mut self) {
            self.ram[SPRITE_GRAPHICS + self.slot] =
                self.ram[SPRITE_GRAPHICS + self.slot].wrapping_sub(1);
        }

        pub(crate) fn xor_graphics(&mut self, value: u8) {
            self.ram[SPRITE_GRAPHICS + self.slot] ^= value;
        }

        pub(crate) fn set_flags2(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS2 + self.slot] = value;
        }

        pub(crate) fn add_flags2(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS2 + self.slot] =
                self.ram[SPRITE_FLAGS2 + self.slot].wrapping_add(value);
        }

        pub(crate) fn subtract_flags2(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS2 + self.slot] =
                self.ram[SPRITE_FLAGS2 + self.slot].wrapping_sub(value);
        }

        pub(crate) fn and_flags2(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS2 + self.slot] &= value;
        }

        pub(crate) fn or_flags2(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS2 + self.slot] |= value;
        }

        pub(crate) fn masked_or_flags2(&mut self, mask: u8, value: u8) {
            self.ram[SPRITE_FLAGS2 + self.slot] =
                (self.ram[SPRITE_FLAGS2 + self.slot] & mask) | value;
        }

        pub(crate) fn set_flags(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS + self.slot] = value;
        }

        pub(crate) fn masked_or_flags(&mut self, mask: u8, value: u8) {
            self.ram[SPRITE_FLAGS + self.slot] =
                (self.ram[SPRITE_FLAGS + self.slot] & mask) | value;
        }

        pub(crate) fn set_flags3(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS3 + self.slot] = value;
        }

        pub(crate) fn and_flags3(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS3 + self.slot] &= value;
        }

        pub(crate) fn clear_flags3_bits(&mut self, mask: u8) {
            self.ram[SPRITE_FLAGS3 + self.slot] &= !mask;
        }

        pub(crate) fn set_flags4(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS4 + self.slot] = value;
        }

        pub(crate) fn increment_flags4(&mut self) {
            self.ram[SPRITE_FLAGS4 + self.slot] =
                self.ram[SPRITE_FLAGS4 + self.slot].wrapping_add(1);
        }

        pub(crate) fn or_flags4(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS4 + self.slot] |= value;
        }

        pub(crate) fn set_flags5(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS5 + self.slot] = value;
        }

        pub(crate) fn and_flags5(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS5 + self.slot] &= value;
        }

        pub(crate) fn or_flags3(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS3 + self.slot] |= value;
        }

        pub(crate) fn xor_flags3(&mut self, value: u8) {
            self.ram[SPRITE_FLAGS3 + self.slot] ^= value;
        }

        pub(crate) fn masked_or_flags3(&mut self, mask: u8, value: u8) {
            self.ram[SPRITE_FLAGS3 + self.slot] =
                (self.ram[SPRITE_FLAGS3 + self.slot] & mask) | value;
        }

        pub(crate) fn set_subtype2(&mut self, value: u8) {
            self.ram[SPRITE_SUBTYPE2 + self.slot] = value;
        }

        pub(crate) fn increment_subtype2(&mut self) {
            self.ram[SPRITE_SUBTYPE2 + self.slot] =
                self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_add(1);
        }

        pub(crate) fn add_subtype2(&mut self, value: u8) {
            self.ram[SPRITE_SUBTYPE2 + self.slot] =
                self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_add(value);
        }

        pub(crate) fn decrement_subtype2(&mut self) -> u8 {
            self.ram[SPRITE_SUBTYPE2 + self.slot] =
                self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_sub(1);
            self.ram[SPRITE_SUBTYPE2 + self.slot]
        }

        pub(crate) fn subtract_subtype2(&mut self, value: u8) {
            self.ram[SPRITE_SUBTYPE2 + self.slot] =
                self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_subtype(&mut self, value: u8) {
            self.ram[SPRITE_SUBTYPE + self.slot] = value;
        }

        pub(crate) fn increment_subtype(&mut self) {
            self.ram[SPRITE_SUBTYPE + self.slot] =
                self.ram[SPRITE_SUBTYPE + self.slot].wrapping_add(1);
        }

        pub(crate) fn add_subtype(&mut self, value: u8) {
            self.ram[SPRITE_SUBTYPE + self.slot] =
                self.ram[SPRITE_SUBTYPE + self.slot].wrapping_add(value);
        }

        pub(crate) fn and_subtype(&mut self, value: u8) {
            self.ram[SPRITE_SUBTYPE + self.slot] &= value;
        }

        pub(crate) fn decrement_subtype(&mut self) {
            self.ram[SPRITE_SUBTYPE + self.slot] =
                self.ram[SPRITE_SUBTYPE + self.slot].wrapping_sub(1);
        }

        pub(crate) fn set_f(&mut self, value: u8) {
            self.ram[SPRITE_F + self.slot] = value;
        }

        pub(crate) fn subtract_f(&mut self, value: u8) {
            self.ram[SPRITE_F + self.slot] = self.ram[SPRITE_F + self.slot].wrapping_sub(value);
        }

        pub(crate) fn decrement_f(&mut self) {
            self.ram[SPRITE_F + self.slot] = self.ram[SPRITE_F + self.slot].wrapping_sub(1);
        }

        pub(crate) fn set_g(&mut self, value: u8) {
            self.ram[SPRITE_G + self.slot] = value;
        }

        pub(crate) fn increment_g(&mut self) {
            self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_add(1);
        }

        pub(crate) fn add_g(&mut self, value: u8) {
            self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_add(value);
        }

        pub(crate) fn subtract_g(&mut self, value: u8) {
            self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_sub(value);
        }

        pub(crate) fn decrement_g(&mut self) {
            self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_sub(1);
        }

        pub(crate) fn set_floor(&mut self, value: u8) {
            self.ram[SPRITE_FLOOR + self.slot] = value;
        }

        pub(crate) fn set_room(&mut self, value: u8) {
            self.ram[SPRITE_ROOM + self.slot] = value;
        }

        pub(crate) fn set_x_recoil(&mut self, value: u8) {
            self.ram[SPRITE_X_RECOIL + self.slot] = value;
        }

        pub(crate) fn set_y_recoil(&mut self, value: u8) {
            self.ram[SPRITE_Y_RECOIL + self.slot] = value;
        }

        pub(crate) fn set_oam_flags(&mut self, value: u8) {
            self.ram[SPRITE_OAM_FLAGS + self.slot] = value;
        }

        pub(crate) fn and_oam_flags(&mut self, value: u8) {
            self.ram[SPRITE_OAM_FLAGS + self.slot] &= value;
        }

        pub(crate) fn xor_oam_flags(&mut self, value: u8) {
            self.ram[SPRITE_OAM_FLAGS + self.slot] ^= value;
        }

        pub(crate) fn or_oam_flags(&mut self, value: u8) {
            self.ram[SPRITE_OAM_FLAGS + self.slot] |= value;
        }

        pub(crate) fn masked_or_oam_flags(&mut self, mask: u8, value: u8) {
            self.ram[SPRITE_OAM_FLAGS + self.slot] =
                (self.ram[SPRITE_OAM_FLAGS + self.slot] & mask) | value;
        }

        pub(crate) fn set_stunned(&mut self, value: u8) {
            self.ram[SPRITE_STUNNED + self.slot] = value;
        }

        pub(crate) fn decrement_stunned(&mut self) {
            self.ram[SPRITE_STUNNED + self.slot] =
                self.ram[SPRITE_STUNNED + self.slot].wrapping_sub(1);
        }

        pub(crate) fn set_pause(&mut self, value: u8) {
            self.ram[SPRITE_PAUSE + self.slot] = value;
        }

        pub(crate) fn set_health(&mut self, value: u8) {
            self.ram[SPRITE_HEALTH + self.slot] = value;
        }

        pub(crate) fn decrement_health(&mut self) {
            self.ram[SPRITE_HEALTH + self.slot] =
                self.ram[SPRITE_HEALTH + self.slot].wrapping_sub(1);
        }

        pub(crate) fn set_ignore_projectile(&mut self, value: u8) {
            self.ram[SPRITE_IGNORE_PROJECTILE + self.slot] = value;
        }

        pub(crate) fn set_draw_work_byte_2(&mut self, value: u8) {
            self.ram[SPRITE_DRAW_WORK_BYTE_2 + self.slot] = value;
        }

        pub(crate) fn set_draw_i(&mut self, value: u8) {
            self.ram[SPRITE_DRAW_I + self.slot] = value;
        }

        pub(crate) fn set_draw_work_byte_3(&mut self, value: u8) {
            self.ram[SPRITE_DRAW_WORK_BYTE_3 + self.slot] = value;
        }

        pub(crate) fn increment_draw_work_byte_3(&mut self) {
            self.ram[SPRITE_DRAW_WORK_BYTE_3 + self.slot] =
                self.ram[SPRITE_DRAW_WORK_BYTE_3 + self.slot].wrapping_add(1);
        }

        pub(crate) fn set_draw_work_byte_4(&mut self, value: u8) {
            self.ram[SPRITE_DRAW_WORK_BYTE_4 + self.slot] = value;
        }

        pub(crate) fn set_draw_work_byte_5(&mut self, value: u8) {
            self.ram[SPRITE_DRAW_WORK_BYTE_5 + self.slot] = value;
        }

        pub(crate) fn set_draw_work_byte_1(&mut self, value: u8) {
            self.ram[SPRITE_DRAW_WORK_BYTE_1 + self.slot] = value;
        }

        pub(crate) fn increment_ignore_projectile(&mut self) {
            self.ram[SPRITE_IGNORE_PROJECTILE + self.slot] =
                self.ram[SPRITE_IGNORE_PROJECTILE + self.slot].wrapping_add(1);
        }

        pub(crate) fn set_deflection_bits(&mut self, value: u8) {
            self.ram[SPRITE_DEFL_BITS + self.slot] = value;
        }

        pub(crate) fn or_deflection_bits(&mut self, value: u8) {
            self.ram[SPRITE_DEFL_BITS + self.slot] |= value;
        }

        pub(crate) fn clear_deflection_bits(&mut self, mask: u8) {
            self.ram[SPRITE_DEFL_BITS + self.slot] &= !mask;
        }

        pub(crate) fn and_deflection_bits(&mut self, value: u8) {
            self.ram[SPRITE_DEFL_BITS + self.slot] &= value;
        }

        pub(crate) fn set_bump_damage(&mut self, value: u8) {
            self.ram[SPRITE_BUMP_DAMAGE + self.slot] = value;
        }

        pub(crate) fn and_bump_damage(&mut self, value: u8) {
            self.ram[SPRITE_BUMP_DAMAGE + self.slot] &= value;
        }

        pub(crate) fn set_incoming_damage(&mut self, value: u8) {
            self.ram[SPRITE_INCOMING_DAMAGE + self.slot] = value;
        }

        pub(crate) fn set_n(&mut self, value: u8) {
            self.ram[SPRITE_N + self.slot] = value;
        }

        pub(crate) fn set_n_word(&mut self, value: u16) {
            write_le_u16(self.ram, SPRITE_N + self.slot * 2, value);
        }

        pub(crate) fn set_die_action(&mut self, value: u8) {
            self.ram[SPRITE_DIE_ACTION + self.slot] = value;
        }

        pub(crate) fn increment_die_action(&mut self) {
            self.ram[SPRITE_DIE_ACTION + self.slot] =
                self.ram[SPRITE_DIE_ACTION + self.slot].wrapping_add(1);
        }

        pub(crate) fn set_object_priority(&mut self, value: u8) {
            self.ram[SPRITE_OBJ_PRIO + self.slot] = value;
        }

        pub(crate) fn clear_object_priority_bits(&mut self, mask: u8) {
            self.ram[SPRITE_OBJ_PRIO + self.slot] &= !mask;
        }

        pub(crate) fn or_object_priority_bits(&mut self, value: u8) {
            self.ram[SPRITE_OBJ_PRIO + self.slot] |= value;
        }

        pub(crate) fn or_object_priority(&mut self, value: u8) {
            self.ram[SPRITE_OBJ_PRIO + self.slot] |= value;
        }

        pub(crate) fn and_object_priority(&mut self, value: u8) {
            self.ram[SPRITE_OBJ_PRIO + self.slot] &= value;
        }

        pub(crate) fn set_hit_timer(&mut self, value: u8) {
            self.ram[SPRITE_HIT_TIMER + self.slot] = value;
        }

        pub(crate) fn or_hit_timer(&mut self, value: u8) {
            self.ram[SPRITE_HIT_TIMER + self.slot] |= value;
        }

        pub(crate) fn set_anim_clock(&mut self, value: u8) {
            self.ram[SPRITE_ANIM_CLOCK + self.slot] = value;
        }

        pub(crate) fn add_anim_clock(&mut self, value: u8) {
            self.ram[SPRITE_ANIM_CLOCK + self.slot] =
                self.ram[SPRITE_ANIM_CLOCK + self.slot].wrapping_add(value);
        }

        pub(crate) fn increment_anim_clock(&mut self) {
            self.ram[SPRITE_ANIM_CLOCK + self.slot] =
                self.ram[SPRITE_ANIM_CLOCK + self.slot].wrapping_add(1);
        }

        pub(crate) fn decrement_anim_clock(&mut self) {
            self.ram[SPRITE_ANIM_CLOCK + self.slot] =
                self.ram[SPRITE_ANIM_CLOCK + self.slot].wrapping_sub(1);
        }

        pub(crate) fn set_wall_collision(&mut self, value: u8) {
            self.ram[SPRITE_WALL_COLLISION + self.slot] = value;
        }

        pub(crate) fn or_wall_collision(&mut self, value: u8) {
            self.ram[SPRITE_WALL_COLLISION + self.slot] |= value;
        }

        pub(crate) fn decrement_wall_collision(&mut self) {
            self.ram[SPRITE_WALL_COLLISION + self.slot] =
                self.ram[SPRITE_WALL_COLLISION + self.slot].wrapping_sub(1);
        }

        pub(crate) fn move_x(&mut self) {
            if self.ram[SPRITE_X_VELOCITY + self.slot] == 0 {
                return;
            }
            move_axis24(
                self.ram,
                SPRITE_X_SUBPIXEL + self.slot,
                SPRITE_X_LO + self.slot,
                SPRITE_X_HI + self.slot,
                SPRITE_X_VELOCITY + self.slot,
            );
        }

        pub(crate) fn move_y(&mut self) {
            if self.ram[SPRITE_Y_VELOCITY + self.slot] == 0 {
                return;
            }
            move_axis24(
                self.ram,
                SPRITE_Y_SUBPIXEL + self.slot,
                SPRITE_Y_LO + self.slot,
                SPRITE_Y_HI + self.slot,
                SPRITE_Y_VELOCITY + self.slot,
            );
        }

        pub(crate) fn move_z(&mut self) {
            move_axis16(
                self.ram,
                SPRITE_Z_SUBPIXEL + self.slot,
                SPRITE_Z + self.slot,
                SPRITE_Z_VELOCITY + self.slot,
            );
        }
    }

    pub(crate) struct AncillaSlotView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> AncillaSlotView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn slot(&self) -> u8 {
            self.slot as u8
        }

        pub(crate) fn ancilla_type(&self) -> u8 {
            byte(self.ram, ANCILLA_TYPE + self.slot)
        }

        pub(crate) fn is_active(&self) -> bool {
            self.ancilla_type() != 0
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(self.ram, ANCILLA_X_LO + self.slot, ANCILLA_X_HI + self.slot)
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, ANCILLA_X_LO + self.slot)
        }

        pub(crate) fn x_high(&self) -> u8 {
            byte(self.ram, ANCILLA_X_HI + self.slot)
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(self.ram, ANCILLA_Y_LO + self.slot, ANCILLA_Y_HI + self.slot)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, ANCILLA_Y_LO + self.slot)
        }

        pub(crate) fn y_high(&self) -> u8 {
            byte(self.ram, ANCILLA_Y_HI + self.slot)
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            byte(self.ram, ANCILLA_X_VELOCITY + self.slot)
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            byte(self.ram, ANCILLA_Y_VELOCITY + self.slot)
        }

        pub(crate) fn z_velocity(&self) -> u8 {
            byte(self.ram, ANCILLA_Z_VELOCITY + self.slot)
        }

        pub(crate) fn x_subpixel(&self) -> u8 {
            byte(self.ram, ANCILLA_X_SUBPIXEL + self.slot)
        }

        pub(crate) fn y_subpixel(&self) -> u8 {
            byte(self.ram, ANCILLA_Y_SUBPIXEL + self.slot)
        }

        pub(crate) fn z(&self) -> u8 {
            byte(self.ram, ANCILLA_Z + self.slot)
        }

        pub(crate) fn z_subpixel(&self) -> u8 {
            byte(self.ram, ANCILLA_Z_SUBPIXEL_PLAYER + self.slot)
        }

        pub(crate) fn item_to_link(&self) -> u8 {
            byte(self.ram, ANCILLA_ITEM_TO_LINK + self.slot)
        }

        pub(crate) fn timer(&self) -> u8 {
            byte(self.ram, ANCILLA_TIMER + self.slot)
        }

        pub(crate) fn floor(&self) -> u8 {
            byte(self.ram, ANCILLA_FLOOR + self.slot)
        }

        pub(crate) fn floor2(&self) -> u8 {
            byte(self.ram, ANCILLA_FLOOR2 + self.slot)
        }

        pub(crate) fn object_priority(&self) -> u8 {
            byte(self.ram, ANCILLA_OBJPRIO + self.slot)
        }

        pub(crate) fn u(&self) -> u8 {
            byte(self.ram, ANCILLA_U + self.slot)
        }

        pub(crate) fn num_sprites(&self) -> u8 {
            byte(self.ram, ANCILLA_NUMSPR + self.slot)
        }

        pub(crate) fn direction(&self) -> u8 {
            byte(self.ram, ANCILLA_DIRECTION + self.slot)
        }

        pub(crate) fn tile_attribute(&self) -> u8 {
            byte(self.ram, ANCILLA_TILE_ATTRIBUTE + self.slot)
        }

        pub(crate) fn step(&self) -> u8 {
            byte(self.ram, ANCILLA_STEP + self.slot)
        }

        pub(crate) fn aux_timer(&self) -> u8 {
            byte(self.ram, ANCILLA_AUX_TIMER + self.slot)
        }

        pub(crate) fn work_byte_3(&self) -> u8 {
            byte(self.ram, ANCILLA_WORK_BYTE_3 + self.slot)
        }

        pub(crate) fn work_byte_1(&self) -> u8 {
            byte(self.ram, ANCILLA_WORK_BYTE_1 + self.slot)
        }

        pub(crate) fn s_player(&self) -> u8 {
            byte(self.ram, ANCILLA_S_PLAYER + self.slot)
        }

        pub(crate) fn t_player(&self) -> u8 {
            byte(self.ram, ANCILLA_T_PLAYER + self.slot)
        }

        pub(crate) fn a(&self) -> u8 {
            byte(self.ram, ANCILLA_A + self.slot)
        }

        pub(crate) fn b(&self) -> u8 {
            byte(self.ram, ANCILLA_B + self.slot)
        }

        pub(crate) fn ab_word(&self) -> u16 {
            u16::from(self.a()) | (u16::from(self.b()) << 8)
        }

        pub(crate) fn l(&self) -> u8 {
            byte(self.ram, ANCILLA_L + self.slot)
        }

        pub(crate) fn h(&self) -> u8 {
            byte(self.ram, ANCILLA_H + self.slot)
        }

        pub(crate) fn k(&self) -> u8 {
            byte(self.ram, ANCILLA_K + self.slot)
        }

        pub(crate) fn g(&self) -> u8 {
            byte(self.ram, ANCILLA_G + self.slot)
        }

        pub(crate) fn r(&self) -> u8 {
            byte(self.ram, ANCILLA_R + self.slot)
        }

        pub(crate) fn work_byte_22(&self) -> u8 {
            byte(self.ram, ANCILLA_WORK_BYTE_22 + self.slot)
        }

        pub(crate) fn work_byte_23(&self) -> u8 {
            byte(self.ram, ANCILLA_WORK_BYTE_23 + self.slot)
        }

        pub(crate) fn work_byte_24(&self) -> u8 {
            byte(self.ram, ANCILLA_WORK_BYTE_24 + self.slot)
        }

        pub(crate) fn work_byte_4(&self) -> u8 {
            byte(self.ram, ANCILLA_WORK_BYTE_4 + self.slot)
        }

        pub(crate) fn work_byte_25(&self) -> u8 {
            byte(self.ram, ANCILLA_WORK_BYTE_25 + self.slot)
        }

        pub(crate) fn work_byte_26(&self) -> u8 {
            byte(self.ram, ANCILLA_WORK_BYTE_26 + self.slot)
        }
    }

    pub(crate) struct AncillaSlotViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> AncillaSlotViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_ancilla_type(&mut self, value: u8) {
            self.ram[ANCILLA_TYPE + self.slot] = value;
        }

        pub(crate) fn increment_ancilla_type(&mut self) -> u8 {
            self.ram[ANCILLA_TYPE + self.slot] = self.ram[ANCILLA_TYPE + self.slot].wrapping_add(1);
            self.ram[ANCILLA_TYPE + self.slot]
        }

        pub(crate) fn clear(&mut self) {
            self.set_ancilla_type(0);
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_position(
                self.ram,
                ANCILLA_X_LO + self.slot,
                ANCILLA_X_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_x_low(&mut self, value: u8) {
            self.ram[ANCILLA_X_LO + self.slot] = value;
        }

        pub(crate) fn set_x_high(&mut self, value: u8) {
            self.ram[ANCILLA_X_HI + self.slot] = value;
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_position(
                self.ram,
                ANCILLA_Y_LO + self.slot,
                ANCILLA_Y_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_y_low(&mut self, value: u8) {
            self.ram[ANCILLA_Y_LO + self.slot] = value;
        }

        pub(crate) fn set_y_high(&mut self, value: u8) {
            self.ram[ANCILLA_Y_HI + self.slot] = value;
        }

        pub(crate) fn set_x_subpixel(&mut self, value: u8) {
            self.ram[ANCILLA_X_SUBPIXEL + self.slot] = value;
        }

        pub(crate) fn set_y_subpixel(&mut self, value: u8) {
            self.ram[ANCILLA_Y_SUBPIXEL + self.slot] = value;
        }

        pub(crate) fn set_x_velocity(&mut self, value: u8) {
            self.ram[ANCILLA_X_VELOCITY + self.slot] = value;
        }

        pub(crate) fn set_y_velocity(&mut self, value: u8) {
            self.ram[ANCILLA_Y_VELOCITY + self.slot] = value;
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            self.ram[ANCILLA_X_VELOCITY + self.slot]
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            self.ram[ANCILLA_Y_VELOCITY + self.slot]
        }

        pub(crate) fn add_x_velocity(&mut self, value: u8) -> u8 {
            let velocity = self.ram[ANCILLA_X_VELOCITY + self.slot].wrapping_add(value);
            self.set_x_velocity(velocity);
            velocity
        }

        pub(crate) fn add_y_velocity(&mut self, value: u8) -> u8 {
            let velocity = self.ram[ANCILLA_Y_VELOCITY + self.slot].wrapping_add(value);
            self.set_y_velocity(velocity);
            velocity
        }

        pub(crate) fn subtract_y_velocity(&mut self, value: u8) {
            self.ram[ANCILLA_Y_VELOCITY + self.slot] =
                self.ram[ANCILLA_Y_VELOCITY + self.slot].wrapping_sub(value);
        }

        pub(crate) fn negate_x_velocity(&mut self) {
            self.ram[ANCILLA_X_VELOCITY + self.slot] =
                self.ram[ANCILLA_X_VELOCITY + self.slot].wrapping_neg();
        }

        pub(crate) fn negate_y_velocity(&mut self) {
            self.ram[ANCILLA_Y_VELOCITY + self.slot] =
                self.ram[ANCILLA_Y_VELOCITY + self.slot].wrapping_neg();
        }

        pub(crate) fn set_z_velocity(&mut self, value: u8) {
            self.ram[ANCILLA_Z_VELOCITY + self.slot] = value;
        }

        pub(crate) fn add_z_velocity(&mut self, value: u8) {
            self.ram[ANCILLA_Z_VELOCITY + self.slot] =
                self.ram[ANCILLA_Z_VELOCITY + self.slot].wrapping_add(value);
        }

        pub(crate) fn tick_z_velocity(&mut self) -> u8 {
            let value = self.ram[ANCILLA_Z_VELOCITY + self.slot].wrapping_sub(1);
            self.set_z_velocity(value);
            value
        }

        pub(crate) fn set_z(&mut self, value: u8) {
            self.ram[ANCILLA_Z + self.slot] = value;
        }

        pub(crate) fn set_z_subpixel(&mut self, value: u8) {
            self.ram[ANCILLA_Z_SUBPIXEL_PLAYER + self.slot] = value;
        }

        pub(crate) fn move_x(&mut self) {
            move_axis24(
                self.ram,
                ANCILLA_X_SUBPIXEL + self.slot,
                ANCILLA_X_LO + self.slot,
                ANCILLA_X_HI + self.slot,
                ANCILLA_X_VELOCITY + self.slot,
            );
        }

        pub(crate) fn move_y(&mut self) {
            move_axis24(
                self.ram,
                ANCILLA_Y_SUBPIXEL + self.slot,
                ANCILLA_Y_LO + self.slot,
                ANCILLA_Y_HI + self.slot,
                ANCILLA_Y_VELOCITY + self.slot,
            );
        }

        pub(crate) fn move_z(&mut self) {
            move_axis16(
                self.ram,
                ANCILLA_Z_SUBPIXEL_PLAYER + self.slot,
                ANCILLA_Z + self.slot,
                ANCILLA_Z_VELOCITY + self.slot,
            );
        }

        pub(crate) fn set_item_to_link(&mut self, value: u8) {
            self.ram[ANCILLA_ITEM_TO_LINK + self.slot] = value;
        }

        pub(crate) fn advance_item_to_link(&mut self) -> u8 {
            let value = self.ram[ANCILLA_ITEM_TO_LINK + self.slot].wrapping_add(1);
            self.set_item_to_link(value);
            value
        }

        pub(crate) fn add_item_to_link(&mut self, value: u8) {
            self.ram[ANCILLA_ITEM_TO_LINK + self.slot] =
                self.ram[ANCILLA_ITEM_TO_LINK + self.slot].wrapping_add(value);
        }

        pub(crate) fn retreat_item_to_link(&mut self) -> u8 {
            let value = self.ram[ANCILLA_ITEM_TO_LINK + self.slot].wrapping_sub(1);
            self.set_item_to_link(value);
            value
        }

        pub(crate) fn toggle_item_to_link_bit0(&mut self) -> u8 {
            let value = self.ram[ANCILLA_ITEM_TO_LINK + self.slot] ^ 1;
            self.set_item_to_link(value);
            value
        }

        pub(crate) fn set_timer(&mut self, value: u8) {
            self.ram[ANCILLA_TIMER + self.slot] = value;
        }

        pub(crate) fn tick_timer(&mut self) -> u8 {
            let value = self.ram[ANCILLA_TIMER + self.slot].wrapping_sub(1);
            self.set_timer(value);
            value
        }

        pub(crate) fn set_floor(&mut self, value: u8) {
            self.ram[ANCILLA_FLOOR + self.slot] = value;
        }

        pub(crate) fn set_floor2(&mut self, value: u8) {
            self.ram[ANCILLA_FLOOR2 + self.slot] = value;
        }

        pub(crate) fn set_oam_index(&mut self, value: u8) {
            self.ram[ANCILLA_OAM_IDX + self.slot] = value;
        }

        pub(crate) fn set_num_sprites(&mut self, value: u8) {
            self.ram[ANCILLA_NUMSPR + self.slot] = value;
        }

        pub(crate) fn set_object_priority(&mut self, value: u8) {
            self.ram[ANCILLA_OBJPRIO + self.slot] = value;
        }

        pub(crate) fn xor_object_priority(&mut self, value: u8) {
            self.ram[ANCILLA_OBJPRIO + self.slot] ^= value;
        }

        pub(crate) fn set_direction(&mut self, value: u8) {
            self.ram[ANCILLA_DIRECTION + self.slot] = value;
        }

        pub(crate) fn or_direction(&mut self, value: u8) {
            self.ram[ANCILLA_DIRECTION + self.slot] |= value;
        }

        pub(crate) fn and_direction(&mut self, value: u8) {
            self.ram[ANCILLA_DIRECTION + self.slot] &= value;
        }

        pub(crate) fn add_direction(&mut self, value: u8) {
            self.ram[ANCILLA_DIRECTION + self.slot] =
                self.ram[ANCILLA_DIRECTION + self.slot].wrapping_add(value);
        }

        pub(crate) fn set_tile_attribute(&mut self, value: u8) {
            self.ram[ANCILLA_TILE_ATTRIBUTE + self.slot] = value;
        }

        pub(crate) fn set_step(&mut self, value: u8) {
            self.ram[ANCILLA_STEP + self.slot] = value;
        }

        pub(crate) fn advance_step(&mut self) -> u8 {
            let value = self.ram[ANCILLA_STEP + self.slot].wrapping_add(1);
            self.set_step(value);
            value
        }

        pub(crate) fn add_step(&mut self, value: u8) {
            self.ram[ANCILLA_STEP + self.slot] =
                self.ram[ANCILLA_STEP + self.slot].wrapping_add(value);
        }

        pub(crate) fn retreat_step(&mut self) -> u8 {
            let value = self.ram[ANCILLA_STEP + self.slot].wrapping_sub(1);
            self.set_step(value);
            value
        }

        pub(crate) fn set_aux_timer(&mut self, value: u8) {
            self.ram[ANCILLA_AUX_TIMER + self.slot] = value;
        }

        pub(crate) fn advance_aux_timer(&mut self) -> u8 {
            let value = self.ram[ANCILLA_AUX_TIMER + self.slot].wrapping_add(1);
            self.set_aux_timer(value);
            value
        }

        pub(crate) fn add_aux_timer(&mut self, value: u8) {
            self.ram[ANCILLA_AUX_TIMER + self.slot] =
                self.ram[ANCILLA_AUX_TIMER + self.slot].wrapping_add(value);
        }

        pub(crate) fn tick_aux_timer(&mut self) -> u8 {
            let value = self.ram[ANCILLA_AUX_TIMER + self.slot].wrapping_sub(1);
            self.set_aux_timer(value);
            value
        }

        pub(crate) fn set_work_byte_3(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_3 + self.slot] = value;
        }

        pub(crate) fn add_work_byte_3(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_3 + self.slot] =
                self.ram[ANCILLA_WORK_BYTE_3 + self.slot].wrapping_add(value);
        }

        pub(crate) fn set_work_byte_1(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_1 + self.slot] = value;
        }

        pub(crate) fn subtract_work_byte_1(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_1 + self.slot] =
                self.ram[ANCILLA_WORK_BYTE_1 + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_a(&mut self, value: u8) {
            self.ram[ANCILLA_A + self.slot] = value;
        }

        pub(crate) fn advance_a(&mut self) -> u8 {
            let value = self.ram[ANCILLA_A + self.slot].wrapping_add(1);
            self.set_a(value);
            value
        }

        pub(crate) fn set_b(&mut self, value: u8) {
            self.ram[ANCILLA_B + self.slot] = value;
        }

        pub(crate) fn set_l(&mut self, value: u8) {
            self.ram[ANCILLA_L + self.slot] = value;
        }

        pub(crate) fn advance_l(&mut self) -> u8 {
            let value = self.ram[ANCILLA_L + self.slot].wrapping_add(1);
            self.set_l(value);
            value
        }

        pub(crate) fn add_l(&mut self, value: u8) {
            self.ram[ANCILLA_L + self.slot] = self.ram[ANCILLA_L + self.slot].wrapping_add(value);
        }

        pub(crate) fn retreat_l(&mut self) -> u8 {
            let value = self.ram[ANCILLA_L + self.slot].wrapping_sub(1);
            self.set_l(value);
            value
        }

        pub(crate) fn set_h(&mut self, value: u8) {
            self.ram[ANCILLA_H + self.slot] = value;
        }

        pub(crate) fn set_k(&mut self, value: u8) {
            self.ram[ANCILLA_K + self.slot] = value;
        }

        pub(crate) fn toggle_k_bit0(&mut self) -> u8 {
            let value = self.ram[ANCILLA_K + self.slot] ^ 1;
            self.set_k(value);
            value
        }

        pub(crate) fn advance_k(&mut self) -> u8 {
            let value = self.ram[ANCILLA_K + self.slot].wrapping_add(1);
            self.set_k(value);
            value
        }

        pub(crate) fn add_k(&mut self, value: u8) {
            self.ram[ANCILLA_K + self.slot] = self.ram[ANCILLA_K + self.slot].wrapping_add(value);
        }

        pub(crate) fn retreat_k(&mut self) -> u8 {
            let value = self.ram[ANCILLA_K + self.slot].wrapping_sub(1);
            self.set_k(value);
            value
        }

        pub(crate) fn tick_k(&mut self) -> u8 {
            let value = self.ram[ANCILLA_K + self.slot].wrapping_sub(1);
            self.set_k(value);
            value
        }

        pub(crate) fn set_g(&mut self, value: u8) {
            self.ram[ANCILLA_G + self.slot] = value;
        }

        pub(crate) fn subtract_g(&mut self, value: u8) {
            self.ram[ANCILLA_G + self.slot] = self.ram[ANCILLA_G + self.slot].wrapping_sub(value);
        }

        pub(crate) fn tick_g(&mut self) -> u8 {
            let value = self.ram[ANCILLA_G + self.slot].wrapping_sub(1);
            self.set_g(value);
            value
        }

        pub(crate) fn set_s_player(&mut self, value: u8) {
            self.ram[ANCILLA_S_PLAYER + self.slot] = value;
        }

        pub(crate) fn set_t_player(&mut self, value: u8) {
            self.ram[ANCILLA_T_PLAYER + self.slot] = value;
        }

        pub(crate) fn set_r(&mut self, value: u8) {
            self.ram[ANCILLA_R + self.slot] = value;
        }

        pub(crate) fn advance_r(&mut self) -> u8 {
            let value = self.ram[ANCILLA_R + self.slot].wrapping_add(1);
            self.set_r(value);
            value
        }

        pub(crate) fn add_r(&mut self, value: u8) {
            self.ram[ANCILLA_R + self.slot] = self.ram[ANCILLA_R + self.slot].wrapping_add(value);
        }

        pub(crate) fn tick_s_player(&mut self) -> u8 {
            let value = self.ram[ANCILLA_S_PLAYER + self.slot].wrapping_sub(1);
            self.set_s_player(value);
            value
        }

        pub(crate) fn set_u(&mut self, value: u8) {
            self.ram[ANCILLA_U + self.slot] = value;
        }

        pub(crate) fn subtract_u(&mut self, value: u8) {
            self.ram[ANCILLA_U + self.slot] = self.ram[ANCILLA_U + self.slot].wrapping_sub(value);
        }

        pub(crate) fn advance_work_byte_1_mod4(&mut self) -> u8 {
            let value = self.ram[ANCILLA_WORK_BYTE_1 + self.slot].wrapping_add(1) & 3;
            self.set_work_byte_1(value);
            value
        }

        pub(crate) fn add_work_byte_1_mod4(&mut self, value: u8) -> u8 {
            let next = self.ram[ANCILLA_WORK_BYTE_1 + self.slot].wrapping_add(value) & 3;
            self.set_work_byte_1(next);
            next
        }

        pub(crate) fn tick_work_byte_3(&mut self) -> u8 {
            let value = self.ram[ANCILLA_WORK_BYTE_3 + self.slot].wrapping_sub(1);
            self.set_work_byte_3(value);
            value
        }

        pub(crate) fn advance_work_byte_3(&mut self) -> u8 {
            let value = self.ram[ANCILLA_WORK_BYTE_3 + self.slot].wrapping_add(1);
            self.set_work_byte_3(value);
            value
        }

        pub(crate) fn set_work_byte_4(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_4 + self.slot] = value;
        }

        pub(crate) fn subtract_work_byte_4(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_4 + self.slot] =
                self.ram[ANCILLA_WORK_BYTE_4 + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_work_byte_22(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_22 + self.slot] = value;
        }

        pub(crate) fn tick_work_byte_22(&mut self) -> u8 {
            let value = self.ram[ANCILLA_WORK_BYTE_22 + self.slot].wrapping_sub(1);
            self.set_work_byte_22(value);
            value
        }

        pub(crate) fn subtract_work_byte_22(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_22 + self.slot] =
                self.ram[ANCILLA_WORK_BYTE_22 + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_work_byte_23(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_23 + self.slot] = value;
        }

        pub(crate) fn advance_work_byte_23(&mut self) -> u8 {
            let value = self.ram[ANCILLA_WORK_BYTE_23 + self.slot].wrapping_add(1);
            self.set_work_byte_23(value);
            value
        }

        pub(crate) fn add_work_byte_23(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_23 + self.slot] =
                self.ram[ANCILLA_WORK_BYTE_23 + self.slot].wrapping_add(value);
        }

        pub(crate) fn set_work_byte_24(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_24 + self.slot] = value;
        }

        pub(crate) fn advance_work_byte_24(&mut self) -> u8 {
            let value = self.ram[ANCILLA_WORK_BYTE_24 + self.slot].wrapping_add(1);
            self.set_work_byte_24(value);
            value
        }

        pub(crate) fn add_work_byte_24(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_24 + self.slot] =
                self.ram[ANCILLA_WORK_BYTE_24 + self.slot].wrapping_add(value);
        }

        pub(crate) fn set_work_byte_25(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_25 + self.slot] = value;
        }

        pub(crate) fn set_work_byte_26(&mut self, value: u8) {
            self.ram[ANCILLA_WORK_BYTE_26 + self.slot] = value;
        }

        pub(crate) fn advance_work_byte_25(&mut self) -> u8 {
            let value = self.ram[ANCILLA_WORK_BYTE_25 + self.slot].wrapping_add(1);
            self.set_work_byte_25(value);
            value
        }

        pub(crate) fn retreat_work_byte_25(&mut self) -> u8 {
            let value = self.ram[ANCILLA_WORK_BYTE_25 + self.slot].wrapping_sub(1);
            self.set_work_byte_25(value);
            value
        }

        pub(crate) fn advance_work_byte_4(&mut self) -> u8 {
            let value = self.ram[ANCILLA_WORK_BYTE_4 + self.slot].wrapping_add(1);
            self.set_work_byte_4(value);
            value
        }
    }

    pub(crate) struct OverlordSlotView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> OverlordSlotView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(
                self.ram,
                OVERLORD_X_LO + self.slot,
                OVERLORD_X_HI + self.slot,
            )
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(
                self.ram,
                OVERLORD_Y_LO + self.slot,
                OVERLORD_Y_HI + self.slot,
            )
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, OVERLORD_X_LO + self.slot)
        }

        pub(crate) fn adjacent_x_low_word(&self) -> u16 {
            word(self.ram, OVERLORD_X_LO + self.slot)
        }

        pub(crate) fn x_high(&self) -> u8 {
            byte(self.ram, OVERLORD_X_HI + self.slot)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, OVERLORD_Y_LO + self.slot)
        }

        pub(crate) fn y_high(&self) -> u8 {
            byte(self.ram, OVERLORD_Y_HI + self.slot)
        }

        pub(crate) fn overlord_type(&self) -> u8 {
            byte(self.ram, OVERLORD_TYPE + self.slot)
        }

        pub(crate) fn is_active(&self) -> bool {
            self.overlord_type() != 0
        }

        pub(crate) fn gen1(&self) -> u8 {
            byte(self.ram, OVERLORD_GEN1 + self.slot)
        }

        pub(crate) fn gen1_word(&self) -> u16 {
            packed_position(
                self.ram,
                OVERLORD_GEN1 + self.slot,
                OVERLORD_GEN1 + self.slot + 1,
            )
        }

        pub(crate) fn gen2(&self) -> u8 {
            byte(self.ram, OVERLORD_GEN2 + self.slot)
        }

        pub(crate) fn gen2_word(&self) -> u16 {
            packed_position(
                self.ram,
                OVERLORD_GEN2 + self.slot,
                OVERLORD_GEN2 + self.slot + 1,
            )
        }

        pub(crate) fn gen3(&self) -> u8 {
            byte(self.ram, OVERLORD_GEN3 + self.slot)
        }

        pub(crate) fn floor(&self) -> u8 {
            byte(self.ram, OVERLORD_FLOOR + self.slot)
        }

        pub(crate) fn spawned_area(&self) -> u8 {
            byte(self.ram, OVERLORD_SPAWNED_AREA + self.slot)
        }

        pub(crate) fn sprite_block_pos(&self) -> u16 {
            word(self.ram, OVERLORD_OFFSET_SPRITE_POS + self.slot * 2)
        }
    }

    pub(crate) struct OverlordSlotViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> OverlordSlotViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_position(
                self.ram,
                OVERLORD_X_LO + self.slot,
                OVERLORD_X_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_x_low(&mut self, value: u8) {
            self.ram[OVERLORD_X_LO + self.slot] = value;
        }

        pub(crate) fn set_adjacent_x_low_word(&mut self, value: u16) {
            write_le_u16(self.ram, OVERLORD_X_LO + self.slot, value);
        }

        pub(crate) fn subtract_adjacent_x_low_word(&mut self, value: u16) -> u16 {
            let updated = word(self.ram, OVERLORD_X_LO + self.slot).wrapping_sub(value);
            self.set_adjacent_x_low_word(updated);
            updated
        }

        pub(crate) fn x_low(&self) -> u8 {
            self.ram[OVERLORD_X_LO + self.slot]
        }

        pub(crate) fn set_x_high(&mut self, value: u8) {
            self.ram[OVERLORD_X_HI + self.slot] = value;
        }

        pub(crate) fn increment_x_high(&mut self) {
            self.ram[OVERLORD_X_HI + self.slot] =
                self.ram[OVERLORD_X_HI + self.slot].wrapping_add(1);
        }

        pub(crate) fn add_x_low(&mut self, value: u8) {
            self.ram[OVERLORD_X_LO + self.slot] =
                self.ram[OVERLORD_X_LO + self.slot].wrapping_add(value);
        }

        pub(crate) fn set_circle_x(&mut self, value: u16) {
            self.ram[OVERLORD_X_HI + self.slot] = value as u8;
            self.ram[OVERLORD_Y_HI + self.slot] = (value >> 8) as u8;
        }

        pub(crate) fn set_circle_y(&mut self, value: u16) {
            self.ram[OVERLORD_GEN2 + self.slot] = value as u8;
            self.ram[OVERLORD_FLOOR + self.slot] = (value >> 8) as u8;
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_position(
                self.ram,
                OVERLORD_Y_LO + self.slot,
                OVERLORD_Y_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_y_low(&mut self, value: u8) {
            self.ram[OVERLORD_Y_LO + self.slot] = value;
        }

        pub(crate) fn set_y_high(&mut self, value: u8) {
            self.ram[OVERLORD_Y_HI + self.slot] = value;
        }

        pub(crate) fn subtract_x_low(&mut self, value: u8) {
            self.ram[OVERLORD_X_LO + self.slot] =
                self.ram[OVERLORD_X_LO + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_overlord_type(&mut self, value: u8) {
            self.ram[OVERLORD_TYPE + self.slot] = value;
        }

        pub(crate) fn clear(&mut self) {
            self.set_overlord_type(0);
        }

        pub(crate) fn set_gen1(&mut self, value: u8) {
            self.ram[OVERLORD_GEN1 + self.slot] = value;
        }

        pub(crate) fn add_gen1(&mut self, value: u8) {
            self.ram[OVERLORD_GEN1 + self.slot] =
                self.ram[OVERLORD_GEN1 + self.slot].wrapping_add(value);
        }

        pub(crate) fn add_gen1_word(&mut self, value: u16) {
            let next = read_le_u16(self.ram, OVERLORD_GEN1 + self.slot).wrapping_add(value);
            write_le_u16(self.ram, OVERLORD_GEN1 + self.slot, next);
        }

        pub(crate) fn subtract_gen1(&mut self, value: u8) {
            self.ram[OVERLORD_GEN1 + self.slot] =
                self.ram[OVERLORD_GEN1 + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_gen2(&mut self, value: u8) {
            self.ram[OVERLORD_GEN2 + self.slot] = value;
        }

        pub(crate) fn add_gen2(&mut self, value: u8) {
            self.ram[OVERLORD_GEN2 + self.slot] =
                self.ram[OVERLORD_GEN2 + self.slot].wrapping_add(value);
        }

        pub(crate) fn add_gen2_word(&mut self, value: u16) {
            let next = read_le_u16(self.ram, OVERLORD_GEN2 + self.slot).wrapping_add(value);
            write_le_u16(self.ram, OVERLORD_GEN2 + self.slot, next);
        }

        pub(crate) fn subtract_gen2(&mut self, value: u8) {
            self.ram[OVERLORD_GEN2 + self.slot] =
                self.ram[OVERLORD_GEN2 + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_gen3(&mut self, value: u8) {
            self.ram[OVERLORD_GEN3 + self.slot] = value;
        }

        pub(crate) fn add_gen3(&mut self, value: u8) {
            self.ram[OVERLORD_GEN3 + self.slot] =
                self.ram[OVERLORD_GEN3 + self.slot].wrapping_add(value);
        }

        pub(crate) fn set_floor(&mut self, value: u8) {
            self.ram[OVERLORD_FLOOR + self.slot] = value;
        }

        pub(crate) fn set_sprite_block_pos(&mut self, value: u16) {
            write_le_u16(self.ram, OVERLORD_OFFSET_SPRITE_POS + self.slot * 2, value);
        }

        pub(crate) fn set_spawned_area(&mut self, value: u8) {
            self.ram[OVERLORD_SPAWNED_AREA + self.slot] = value;
        }
    }

    pub(crate) struct OverworldSpritePresenceView<'a> {
        ram: &'a [u8],
    }

    impl<'a> OverworldSpritePresenceView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn marker(&self, index: usize) -> u8 {
            byte(self.ram, OVERWORLD_SPRITE_PRESENCE + index)
        }
    }

    pub(crate) struct OverworldSpritePresenceViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> OverworldSpritePresenceViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_marker(&mut self, index: usize, value: u8) {
            self.ram[OVERWORLD_SPRITE_PRESENCE + index] = value;
        }
    }

    pub(crate) struct OverworldSpriteLoadedView<'a> {
        ram: &'a [u8],
    }

    impl<'a> OverworldSpriteLoadedView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn is_loaded(&self, block: u16, loaded_mask: u8) -> bool {
            byte(
                self.ram,
                OVERWORLD_SPRITE_WAS_LOADED + usize::from(block >> 3),
            ) & loaded_mask
                != 0
        }
    }

    pub(crate) struct OverworldSpriteLoadedViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> OverworldSpriteLoadedViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
            self.ram[OVERWORLD_SPRITE_WAS_LOADED + usize::from(block >> 3)] &= !loaded_mask;
        }

        /// Same as `clear_loaded_mask`, but wraps the byte index to the low
        /// 128 KiB of RAM exactly like the original code did.
        pub(crate) fn clear_loaded_mask_wrapped(&mut self, block: u16, loaded_mask: u8) {
            self.ram[(OVERWORLD_SPRITE_WAS_LOADED + usize::from(block >> 3)) & 0x1ffff] &=
                !loaded_mask;
        }

        pub(crate) fn set_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
            self.ram[OVERWORLD_SPRITE_WAS_LOADED + usize::from(block >> 3)] |= loaded_mask;
        }

        pub(crate) fn clear_all(&mut self) {
            self.ram[OVERWORLD_SPRITE_WAS_LOADED..OVERWORLD_SPRITE_WAS_LOADED + 0x200].fill(0);
        }
    }

    pub(crate) struct GarnishSlotView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> GarnishSlotView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn garnish_type(&self) -> u8 {
            byte(self.ram, GARNISH_TYPE + self.slot)
        }

        pub(crate) fn is_empty(&self) -> bool {
            self.garnish_type() == 0
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(self.ram, GARNISH_X_LO + self.slot, GARNISH_X_HI + self.slot)
        }

        pub(crate) fn x_low(&self) -> u8 {
            byte(self.ram, GARNISH_X_LO + self.slot)
        }

        pub(crate) fn x_high(&self) -> u8 {
            byte(self.ram, GARNISH_X_HI + self.slot)
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(self.ram, GARNISH_Y_LO + self.slot, GARNISH_Y_HI + self.slot)
        }

        pub(crate) fn y_low(&self) -> u8 {
            byte(self.ram, GARNISH_Y_LO + self.slot)
        }

        pub(crate) fn y_high(&self) -> u8 {
            byte(self.ram, GARNISH_Y_HI + self.slot)
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            byte(self.ram, GARNISH_X_VELOCITY + self.slot)
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            byte(self.ram, GARNISH_Y_VELOCITY + self.slot)
        }

        pub(crate) fn x_subpixel(&self) -> u8 {
            byte(self.ram, GARNISH_X_SUBPIXEL + self.slot)
        }

        pub(crate) fn y_subpixel(&self) -> u8 {
            byte(self.ram, GARNISH_Y_SUBPIXEL + self.slot)
        }

        pub(crate) fn countdown(&self) -> u8 {
            byte(self.ram, GARNISH_COUNTDOWN + self.slot)
        }

        pub(crate) fn sprite(&self) -> u8 {
            byte(self.ram, GARNISH_SPRITE + self.slot)
        }

        pub(crate) fn floor(&self) -> u8 {
            byte(self.ram, GARNISH_FLOOR + self.slot)
        }

        pub(crate) fn oam_flags(&self) -> u8 {
            byte(self.ram, GARNISH_OAM_FLAGS + self.slot)
        }
    }

    pub(crate) struct GarnishSlotViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> GarnishSlotViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_garnish_type(&mut self, value: u8) {
            self.ram[GARNISH_TYPE + self.slot] = value;
        }

        pub(crate) fn clear(&mut self) {
            self.set_garnish_type(0);
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_position(
                self.ram,
                GARNISH_X_LO + self.slot,
                GARNISH_X_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_x_low(&mut self, value: u8) {
            self.ram[GARNISH_X_LO + self.slot] = value;
        }

        pub(crate) fn set_x_high(&mut self, value: u8) {
            self.ram[GARNISH_X_HI + self.slot] = value;
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_position(
                self.ram,
                GARNISH_Y_LO + self.slot,
                GARNISH_Y_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_y_low(&mut self, value: u8) {
            self.ram[GARNISH_Y_LO + self.slot] = value;
        }

        pub(crate) fn add_y_low(&mut self, value: u8) {
            self.ram[GARNISH_Y_LO + self.slot] =
                self.ram[GARNISH_Y_LO + self.slot].wrapping_add(value);
        }

        pub(crate) fn subtract_y_low(&mut self, value: u8) {
            self.ram[GARNISH_Y_LO + self.slot] =
                self.ram[GARNISH_Y_LO + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_y_high(&mut self, value: u8) {
            self.ram[GARNISH_Y_HI + self.slot] = value;
        }

        pub(crate) fn set_x_velocity(&mut self, value: u8) {
            self.ram[GARNISH_X_VELOCITY + self.slot] = value;
        }

        pub(crate) fn set_y_velocity(&mut self, value: u8) {
            self.ram[GARNISH_Y_VELOCITY + self.slot] = value;
        }

        pub(crate) fn add_y_velocity(&mut self, value: u8) {
            self.ram[GARNISH_Y_VELOCITY + self.slot] =
                self.ram[GARNISH_Y_VELOCITY + self.slot].wrapping_add(value);
        }

        pub(crate) fn set_x_subpixel(&mut self, value: u8) {
            self.ram[GARNISH_X_SUBPIXEL + self.slot] = value;
        }

        pub(crate) fn set_y_subpixel(&mut self, value: u8) {
            self.ram[GARNISH_Y_SUBPIXEL + self.slot] = value;
        }

        pub(crate) fn set_countdown(&mut self, value: u8) {
            self.ram[GARNISH_COUNTDOWN + self.slot] = value;
        }

        pub(crate) fn subtract_countdown(&mut self, value: u8) {
            self.ram[GARNISH_COUNTDOWN + self.slot] =
                self.ram[GARNISH_COUNTDOWN + self.slot].wrapping_sub(value);
        }

        pub(crate) fn set_sprite(&mut self, value: u8) {
            self.ram[GARNISH_SPRITE + self.slot] = value;
        }

        pub(crate) fn set_floor(&mut self, value: u8) {
            self.ram[GARNISH_FLOOR + self.slot] = value;
        }

        pub(crate) fn set_oam_flags(&mut self, value: u8) {
            self.ram[GARNISH_OAM_FLAGS + self.slot] = value;
        }
    }

    pub(crate) struct GarnishStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> GarnishStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn active_type(&self) -> u8 {
            byte(self.ram, GARNISH_ACTIVE)
        }

        pub(crate) fn boulder_trap_count(&self) -> u8 {
            byte(self.ram, OVERWORLD_BOULDER_TRAP_COUNT)
        }

        pub(crate) fn boulder_trap_timer(&self) -> u8 {
            byte(self.ram, OVERWORLD_BOULDER_TRAP_TIMER)
        }

        pub(crate) fn sprcoll_y_hi(&self) -> u8 {
            byte(self.ram, SPRCOLL_Y_BASE + 1)
        }

        pub(crate) fn sprcoll_x_word(&self) -> u16 {
            word(self.ram, SPRCOLL_X_BASE)
        }

        pub(crate) fn sprcoll_y_word(&self) -> u16 {
            word(self.ram, SPRCOLL_Y_BASE)
        }

        pub(crate) fn active_overlord_index(&self) -> u8 {
            byte(self.ram, ACTIVE_OVERLORD_INDEX)
        }

        pub(crate) fn haunted_grove_flute_event_latch(&self) -> u8 {
            byte(self.ram, HAUNTED_GROVE_FLUTE_EVENT_LATCH)
        }

        pub(crate) fn repulsespark_timer(&self) -> u8 {
            byte(self.ram, REPULSESPARK_TIMER)
        }

        pub(crate) fn sprcoll_x_size(&self) -> u16 {
            word(self.ram, SPRCOLL_X_SIZE)
        }

        pub(crate) fn sprcoll_y_size(&self) -> u16 {
            word(self.ram, SPRCOLL_Y_SIZE)
        }
    }

    pub(crate) struct GarnishStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> GarnishStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_active_type(&mut self, value: u8) {
            self.ram[GARNISH_ACTIVE] = value;
        }

        pub(crate) fn clear_active_type(&mut self) {
            self.ram[GARNISH_ACTIVE] = 0;
        }

        pub(crate) fn increment_boulder_trap_timer(&mut self) -> u8 {
            let val = self.ram[OVERWORLD_BOULDER_TRAP_TIMER].wrapping_add(1);
            self.ram[OVERWORLD_BOULDER_TRAP_TIMER] = val;
            val
        }

        pub(crate) fn set_active_overlord_index(&mut self, value: u8) {
            self.ram[ACTIVE_OVERLORD_INDEX] = value;
        }

        pub(crate) fn increment_haunted_grove_flute_event_latch(&mut self) {
            self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] =
                self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH].wrapping_add(1);
        }

        pub(crate) fn set_repulsespark_timer(&mut self, value: u8) {
            self.ram[REPULSESPARK_TIMER] = value;
        }

        pub(crate) fn set_repulsespark_x_lo(&mut self, value: u8) {
            self.ram[REPULSESPARK_X_LO] = value;
        }

        pub(crate) fn set_repulsespark_y_lo(&mut self, value: u8) {
            self.ram[REPULSESPARK_Y_LO] = value;
        }

        pub(crate) fn set_sprcoll_x_size(&mut self, value: u16) {
            write_le_u16(self.ram, SPRCOLL_X_SIZE, value);
        }

        pub(crate) fn set_sprcoll_y_size(&mut self, value: u16) {
            write_le_u16(self.ram, SPRCOLL_Y_SIZE, value);
        }

        pub(crate) fn set_sprcoll_x_base(&mut self, value: u16) {
            write_le_u16(self.ram, SPRCOLL_X_BASE, value);
        }

        pub(crate) fn set_sprcoll_y_base(&mut self, value: u16) {
            write_le_u16(self.ram, SPRCOLL_Y_BASE, value);
        }

        pub(crate) fn set_repulsespark_floor_status(&mut self, value: u8) {
            self.ram[REPULSESPARK_FLOOR_STATUS] = value;
        }

        pub(crate) fn clear_boulder_trap_count(&mut self) {
            self.ram[OVERWORLD_BOULDER_TRAP_COUNT] = 0;
        }

        pub(crate) fn increment_boulder_trap_count(&mut self) {
            self.ram[OVERWORLD_BOULDER_TRAP_COUNT] =
                self.ram[OVERWORLD_BOULDER_TRAP_COUNT].wrapping_add(1);
        }

        pub(crate) fn clear_haunted_grove_flute_event_latch(&mut self) {
            self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] = 0;
        }
    }

    pub(crate) struct OamStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> OamStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn priority_word(&self) -> u16 {
            word(self.ram, OAM_PRIORITY_VALUE)
        }

        pub(crate) fn priority_high(&self) -> u8 {
            byte(self.ram, OAM_PRIORITY_VALUE + 1)
        }

        pub(crate) fn current_pointer(&self) -> u16 {
            word(self.ram, OAM_CUR_PTR)
        }

        pub(crate) fn current_pointer_usize(&self) -> usize {
            usize::from(self.current_pointer())
        }

        pub(crate) fn current_extended_pointer(&self) -> u16 {
            word(self.ram, OAM_EXT_CUR_PTR)
        }

        pub(crate) fn current_extended_pointer_usize(&self) -> usize {
            usize::from(self.current_extended_pointer())
        }

        pub(crate) fn extended_byte(&self, index: usize) -> u8 {
            byte(self.ram, BYTEWISE_EXTENDED_OAM + index)
        }

        pub(crate) fn sprite_sorting_setting(&self) -> u8 {
            byte(self.ram, SORT_SPRITES_SETTING)
        }

        pub(crate) fn sprite_sorting_offset_index(&self) -> usize {
            usize::from(self.sprite_sorting_setting())
        }

        pub(crate) fn has_sprite_sorting(&self) -> bool {
            self.sprite_sorting_setting() != 0
        }

        pub(crate) fn packed_extended_oam_byte(&self, index: usize) -> u8 {
            (self.extended_byte(3 + index * 4) << 6)
                | (self.extended_byte(2 + index * 4) << 4)
                | (self.extended_byte(1 + index * 4) << 2)
                | self.extended_byte(index * 4)
        }

        pub(crate) fn priority_value_2(&self) -> u16 {
            word(self.ram, OAM_PRIORITY_VALUE_2)
        }

        pub(crate) fn sort_sprites_offset(&self) -> u16 {
            word(self.ram, SORT_SPRITES_OFFSET_INTO_OAM_BUFFER)
        }

        pub(crate) fn player_oam_computed_value(&self) -> u8 {
            byte(self.ram, VALUE_COMPUTED_FOR_PLAYER_OAM)
        }

        pub(crate) fn turtle_rock_priority_flag(&self) -> u8 {
            byte(self.ram, TURTLE_ROCK_OAM_PRIORITY_FLAG)
        }

        pub(crate) fn entry_x(&self, addr: usize) -> u8 {
            byte(self.ram, addr)
        }

        pub(crate) fn entry_y(&self, addr: usize) -> u8 {
            byte(self.ram, addr + 1)
        }

        pub(crate) fn entry_char(&self, addr: usize) -> u8 {
            byte(self.ram, addr + 2)
        }

        pub(crate) fn entry_flags(&self, addr: usize) -> u8 {
            byte(self.ram, addr + 3)
        }

        /// Reads a byte from the bytewise extended OAM region at a byte
        /// address.
        pub(crate) fn extended_byte_at(&self, addr: usize) -> u8 {
            byte(self.ram, addr)
        }

        pub(crate) fn region_base_word(&self, region: usize) -> u16 {
            word(self.ram, OAM_REGION_BASE + region * 2)
        }

        pub(crate) fn region_alloc_counter(&self, region: usize) -> u16 {
            word(self.ram, OAM_REGION_ALLOC + region * 2)
        }
    }

    pub(crate) struct OamStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> OamStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_priority_word(&mut self, value: u16) {
            write_le_u16(self.ram, OAM_PRIORITY_VALUE, value);
        }

        pub(crate) fn subtract_priority_word(&mut self, value: u16) {
            let priority = read_le_u16(self.ram, OAM_PRIORITY_VALUE);
            write_le_u16(self.ram, OAM_PRIORITY_VALUE, priority.wrapping_sub(value));
        }

        pub(crate) fn set_priority_high(&mut self, value: u8) {
            self.ram[OAM_PRIORITY_VALUE + 1] = value;
        }

        pub(crate) fn set_current_pointer(&mut self, value: u16) {
            write_le_u16(self.ram, OAM_CUR_PTR, value);
        }

        pub(crate) fn add_current_pointer(&mut self, value: u16) {
            let pointer = read_le_u16(self.ram, OAM_CUR_PTR);
            write_le_u16(self.ram, OAM_CUR_PTR, pointer.wrapping_add(value));
        }

        pub(crate) fn subtract_current_pointer(&mut self, value: u16) {
            let pointer = read_le_u16(self.ram, OAM_CUR_PTR);
            write_le_u16(self.ram, OAM_CUR_PTR, pointer.wrapping_sub(value));
        }

        pub(crate) fn set_current_extended_pointer(&mut self, value: u16) {
            write_le_u16(self.ram, OAM_EXT_CUR_PTR, value);
        }

        pub(crate) fn set_sprite_sorting_setting(&mut self, value: u8) {
            self.ram[SORT_SPRITES_SETTING] = value;
        }

        pub(crate) fn set_priority_value_2(&mut self, value: u16) {
            write_le_u16(self.ram, OAM_PRIORITY_VALUE_2, value);
        }

        pub(crate) fn set_sort_sprites_offset(&mut self, value: u16) {
            write_le_u16(self.ram, SORT_SPRITES_OFFSET_INTO_OAM_BUFFER, value);
        }

        pub(crate) fn clear_sort_sprites_offset(&mut self) {
            write_le_u16(self.ram, SORT_SPRITES_OFFSET_INTO_OAM_BUFFER, 0);
        }

        pub(crate) fn set_player_oam_computed_value(&mut self, value: u8) {
            self.ram[VALUE_COMPUTED_FOR_PLAYER_OAM] = value;
        }

        pub(crate) fn clear_sprite_sorting_setting(&mut self) {
            self.set_sprite_sorting_setting(0);
        }

        pub(crate) fn add_current_extended_pointer(&mut self, value: u16) {
            let pointer = read_le_u16(self.ram, OAM_EXT_CUR_PTR);
            write_le_u16(self.ram, OAM_EXT_CUR_PTR, pointer.wrapping_add(value));
        }

        pub(crate) fn subtract_current_extended_pointer(&mut self, value: u16) {
            let pointer = read_le_u16(self.ram, OAM_EXT_CUR_PTR);
            write_le_u16(self.ram, OAM_EXT_CUR_PTR, pointer.wrapping_sub(value));
        }

        pub(crate) fn set_extended_byte(&mut self, index: usize, value: u8) {
            self.ram[BYTEWISE_EXTENDED_OAM + index] = value;
        }

        /// Writes a byte into the bytewise extended OAM region at a byte
        /// address.
        pub(crate) fn set_extended_byte_at(&mut self, addr: usize, value: u8) {
            self.ram[addr] = value;
        }

        pub(crate) fn set_extended_word(&mut self, index: usize, value: u16) {
            write_le_u16(self.ram, BYTEWISE_EXTENDED_OAM + index, value);
        }

        pub(crate) fn set_packed_extended_oam_byte(&mut self, index: usize, value: u8) {
            self.ram[EXTENDED_OAM + index] = value;
        }

        pub(crate) fn hide_sprite_row(&mut self, oam_index: usize) {
            self.ram[OAM_BUF + oam_index * 4 + 1] = 0xf0;
        }

        /// Writes a full 4-byte OAM entry (x, y, char, flags) at a byte
        /// address into the OAM buffer.
        pub(crate) fn write_entry(&mut self, addr: usize, x: u8, y: u8, charnum: u8, flags: u8) {
            self.ram[addr] = x;
            self.ram[addr + 1] = y;
            self.ram[addr + 2] = charnum;
            self.ram[addr + 3] = flags;
        }

        pub(crate) fn set_entry_x(&mut self, addr: usize, x: u8) {
            self.ram[addr] = x;
        }

        pub(crate) fn set_entry_y(&mut self, addr: usize, y: u8) {
            self.ram[addr + 1] = y;
        }

        pub(crate) fn set_entry_xy(&mut self, addr: usize, x: u8, y: u8) {
            self.ram[addr] = x;
            self.ram[addr + 1] = y;
        }

        /// Writes the char (low byte) and flags (high byte) of an entry as a
        /// single little-endian word.
        pub(crate) fn set_entry_char_flags(&mut self, addr: usize, value: u16) {
            write_le_u16(self.ram, addr + 2, value);
        }

        /// Moves the entry off-screen by setting its y coordinate below the
        /// visible area.
        pub(crate) fn hide_entry(&mut self, addr: usize) {
            self.ram[addr + 1] = 0xf0;
        }

        pub(crate) fn set_entry_char(&mut self, addr: usize, charnum: u8) {
            self.ram[addr + 2] = charnum;
        }

        pub(crate) fn set_entry_flags(&mut self, addr: usize, flags: u8) {
            self.ram[addr + 3] = flags;
        }

        pub(crate) fn or_entry_flags(&mut self, addr: usize, bits: u8) {
            self.ram[addr + 3] |= bits;
        }

        /// Keeps the flag bits selected by `keep_mask` and ors in `bits`.
        pub(crate) fn merge_entry_flags(&mut self, addr: usize, keep_mask: u8, bits: u8) {
            self.ram[addr + 3] = (self.ram[addr + 3] & keep_mask) | bits;
        }

        /// Writes consecutive bytes into the bytewise extended OAM region at
        /// a byte address.
        pub(crate) fn set_extended_bytes_at(&mut self, addr: usize, values: &[u8]) {
            self.ram[addr..addr + values.len()].copy_from_slice(values);
        }

        pub(crate) fn init_credits_region_base(&mut self) {
            write_le_u16(self.ram, OAM_REGION_BASE, 0x30);
            write_le_u16(self.ram, OAM_REGION_BASE + 2, 0x1d0);
            write_le_u16(self.ram, OAM_REGION_BASE + 4, 0);
        }

        pub(crate) fn set_region_base_word(&mut self, region: usize, value: u16) {
            write_le_u16(self.ram, OAM_REGION_BASE + region * 2, value);
        }

        pub(crate) fn set_region_alloc_counter(&mut self, region: usize, value: u16) {
            write_le_u16(self.ram, OAM_REGION_ALLOC + region * 2, value);
        }
    }

    pub(crate) struct ArcheryGameView<'a> {
        ram: &'a [u8],
    }

    impl<'a> ArcheryGameView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn hit_counter(&self) -> u8 {
            byte(self.ram, ARCHERY_GAME_HIT_COUNTER)
        }

        pub(crate) fn arrows_left(&self) -> u8 {
            byte(self.ram, ARCHERY_GAME_ARROWS_LEFT)
        }

        pub(crate) fn out_of_arrows(&self) -> u8 {
            byte(self.ram, ARCHERY_GAME_OUT_OF_ARROWS)
        }
    }

    pub(crate) struct ArcheryGameViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> ArcheryGameViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear_hit_counter(&mut self) {
            self.ram[ARCHERY_GAME_HIT_COUNTER] = 0;
        }

        pub(crate) fn set_arrows_left(&mut self, value: u8) {
            self.ram[ARCHERY_GAME_ARROWS_LEFT] = value;
        }

        pub(crate) fn increment_out_of_arrows(&mut self) {
            self.ram[ARCHERY_GAME_OUT_OF_ARROWS] =
                self.ram[ARCHERY_GAME_OUT_OF_ARROWS].wrapping_add(1);
        }

        pub(crate) fn clear_out_of_arrows(&mut self) {
            self.ram[ARCHERY_GAME_OUT_OF_ARROWS] = 0;
        }
    }

    pub(crate) struct MinigameStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> MinigameStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn is_archer_or_shovel_game(&self) -> u8 {
            byte(self.ram, IS_ARCHER_OR_SHOVEL_GAME)
        }

        pub(crate) fn credits(&self) -> u8 {
            byte(self.ram, MINIGAME_CREDITS)
        }

        pub(crate) fn flag_boomerang_in_place(&self) -> u8 {
            byte(self.ram, FLAG_FOR_BOOMERANG_IN_PLACE)
        }
    }

    pub(crate) struct MinigameStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> MinigameStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_is_archer_or_shovel_game(&mut self, value: u8) {
            self.ram[IS_ARCHER_OR_SHOVEL_GAME] = value;
        }

        pub(crate) fn clear_is_archer_or_shovel_game(&mut self) {
            self.ram[IS_ARCHER_OR_SHOVEL_GAME] = 0;
        }

        pub(crate) fn set_credits(&mut self, value: u8) {
            self.ram[MINIGAME_CREDITS] = value;
        }

        pub(crate) fn clear_flag_boomerang_in_place(&mut self) {
            self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 0;
        }
    }

    pub(crate) struct SpriteBattleView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SpriteBattleView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn sprites_killed(&self) -> u8 {
            byte(self.ram, NUM_SPRITES_KILLED)
        }

        pub(crate) fn times_hurt_by_sprites(&self) -> u8 {
            byte(self.ram, TIMES_HURT_BY_SPRITES)
        }

        pub(crate) fn item_drop_luck(&self) -> u8 {
            byte(self.ram, ITEM_DROP_LUCK)
        }

        pub(crate) fn luck_kill_counter(&self) -> u8 {
            byte(self.ram, LUCK_KILL_COUNTER)
        }

        pub(crate) fn item_drop_counter(&self) -> u8 {
            byte(self.ram, ITEM_DROP_COUNTER)
        }

        pub(crate) fn damage_type_determiner(&self) -> u8 {
            byte(self.ram, DAMAGE_TYPE_DETERMINER)
        }

        pub(crate) fn damaging_enemies_timer(&self) -> u8 {
            byte(self.ram, SET_WHEN_DAMAGING_ENEMIES)
        }
    }

    pub(crate) struct SpriteBattleViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SpriteBattleViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn clear_sprites_killed(&mut self) {
            self.ram[NUM_SPRITES_KILLED] = 0;
        }

        pub(crate) fn clear_times_hurt_by_sprites(&mut self) {
            self.ram[TIMES_HURT_BY_SPRITES] = 0;
        }

        pub(crate) fn set_item_drop_luck(&mut self, value: u8) {
            self.ram[ITEM_DROP_LUCK] = value;
        }

        pub(crate) fn clear_luck_kill_counter(&mut self) {
            self.ram[LUCK_KILL_COUNTER] = 0;
        }

        pub(crate) fn clear_item_drop_counter(&mut self) {
            self.ram[ITEM_DROP_COUNTER] = 0;
        }

        pub(crate) fn increment_sprites_killed(&mut self) {
            self.ram[NUM_SPRITES_KILLED] = self.ram[NUM_SPRITES_KILLED].wrapping_add(1);
        }

        pub(crate) fn increment_luck_kill_counter(&mut self) {
            self.ram[LUCK_KILL_COUNTER] = self.ram[LUCK_KILL_COUNTER].wrapping_add(1);
        }

        pub(crate) fn set_damage_type_determiner(&mut self, value: u8) {
            self.ram[DAMAGE_TYPE_DETERMINER] = value;
        }

        pub(crate) fn set_damaging_enemies_timer(&mut self, value: u8) {
            self.ram[SET_WHEN_DAMAGING_ENEMIES] = value;
        }

        pub(crate) fn clear_damaging_enemies_timer(&mut self) {
            self.ram[SET_WHEN_DAMAGING_ENEMIES] = 0;
        }

        pub(crate) fn tick_damaging_enemies_timer(&mut self) {
            if self.ram[SET_WHEN_DAMAGING_ENEMIES] & 0x7f != 0 {
                self.ram[SET_WHEN_DAMAGING_ENEMIES] =
                    self.ram[SET_WHEN_DAMAGING_ENEMIES].wrapping_sub(1);
            } else {
                self.ram[SET_WHEN_DAMAGING_ENEMIES] = 0;
            }
        }

        pub(crate) fn increment_item_drop_counter(&mut self) -> u8 {
            let v = self.ram[ITEM_DROP_COUNTER].wrapping_add(1);
            self.ram[ITEM_DROP_COUNTER] = v;
            v
        }
    }

    pub(crate) struct SharedMessageTimerView<'a> {
        ram: &'a [u8],
    }

    impl<'a> SharedMessageTimerView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn get(&self) -> u16 {
            word(self.ram, SHARED_MESSAGE_TIMER)
        }
    }

    pub(crate) struct SharedMessageTimerViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> SharedMessageTimerViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set(&mut self, value: u16) {
            write_le_u16(self.ram, SHARED_MESSAGE_TIMER, value);
        }
    }

    pub(crate) struct IntroStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> IntroStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn want_double_ret(&self) -> u8 {
            byte(self.ram, INTRO_WANT_DOUBLE_RET)
        }

        pub(crate) fn sprite_alloc(&self) -> u16 {
            word(self.ram, INTRO_SPRITE_ALLOC)
        }

        pub(crate) fn triforce_ctr(&self) -> u16 {
            word(self.ram, TRIFORCE_CTR)
        }
    }

    pub(crate) struct IntroStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> IntroStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_want_double_ret(&mut self, value: u8) {
            self.ram[INTRO_WANT_DOUBLE_RET] = value;
        }

        pub(crate) fn set_sprite_alloc(&mut self, value: u16) {
            write_le_u16(self.ram, INTRO_SPRITE_ALLOC, value);
        }

        pub(crate) fn set_triforce_ctr(&mut self, value: u16) {
            write_le_u16(self.ram, TRIFORCE_CTR, value);
        }

        pub(crate) fn decrement_triforce_ctr(&mut self) {
            let v = read_le_u16(self.ram, TRIFORCE_CTR).wrapping_sub(1);
            write_le_u16(self.ram, TRIFORCE_CTR, v);
        }
    }

    pub(crate) struct EndingCreditStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> EndingCreditStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn which_dung(&self) -> u16 {
            word(self.ram, ENDING_WHICH_DUNG)
        }
    }

    pub(crate) struct EndingCreditStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> EndingCreditStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_which_dung(&mut self, value: u16) {
            write_le_u16(self.ram, ENDING_WHICH_DUNG, value);
        }

        pub(crate) fn clear_which_dung(&mut self) {
            write_le_u16(self.ram, ENDING_WHICH_DUNG, 0);
        }

        pub(crate) fn set_credit_digit_char(&mut self, value: u16) {
            write_le_u16(self.ram, ENDING_CREDIT_DIGIT_CHAR, value);
        }
    }

    pub(crate) struct IntroSwordView<'a> {
        ram: &'a [u8],
    }

    impl<'a> IntroSwordView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn ypos(&self) -> u16 {
            word(self.ram, INTRO_SWORD_YPOS)
        }

        pub(crate) fn sparkle_timer(&self) -> u8 {
            byte(self.ram, INTRO_SWORD_SPARKLE_TIMER)
        }

        pub(crate) fn sparkle_step(&self) -> u8 {
            byte(self.ram, INTRO_SWORD_SPARKLE_STEP)
        }

        pub(crate) fn anim_phase(&self) -> u8 {
            byte(self.ram, INTRO_SWORD_ANIM_STEP) >> 1
        }

        pub(crate) fn anim_step_raw(&self) -> u8 {
            byte(self.ram, INTRO_SWORD_ANIM_STEP)
        }

        pub(crate) fn sparkle_y_offset(&self) -> u8 {
            byte(self.ram, INTRO_SWORD_SPARKLE_Y_OFFSET)
        }

        pub(crate) fn flash_rgb_channel(&self) -> usize {
            byte(self.ram, INTRO_SWORD_FLASH_RGB_CHANNEL) as usize
        }
    }

    pub(crate) struct IntroSwordViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> IntroSwordViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn reset_sword_state(&mut self) {
            self.ram[INTRO_SWORD_SPARKLE_STEP] = 7;
            self.ram[INTRO_SWORD_ANIM_STEP] = 0;
            self.ram[INTRO_SWORD_SPARKLE_Y_OFFSET] = 0;
            write_le_u16(self.ram, INTRO_SWORD_YPOS, (-130i16) as u16);
        }

        pub(crate) fn set_ypos(&mut self, value: u16) {
            write_le_u16(self.ram, INTRO_SWORD_YPOS, value);
        }

        pub(crate) fn advance_ypos(&mut self) {
            let y = read_le_u16(self.ram, INTRO_SWORD_YPOS).wrapping_add(16);
            write_le_u16(self.ram, INTRO_SWORD_YPOS, y);
        }

        pub(crate) fn decrement_sparkle_timer(&mut self) {
            self.ram[INTRO_SWORD_SPARKLE_TIMER] =
                self.ram[INTRO_SWORD_SPARKLE_TIMER].wrapping_sub(1);
        }

        pub(crate) fn set_sparkle_timer(&mut self, value: u8) {
            self.ram[INTRO_SWORD_SPARKLE_TIMER] = value;
        }

        pub(crate) fn set_sparkle_step(&mut self, value: u8) {
            self.ram[INTRO_SWORD_SPARKLE_STEP] = value;
        }

        pub(crate) fn decrement_sparkle_step_check_negative(&mut self) -> bool {
            self.ram[INTRO_SWORD_SPARKLE_STEP] = self.ram[INTRO_SWORD_SPARKLE_STEP].wrapping_sub(1);
            (self.ram[INTRO_SWORD_SPARKLE_STEP] as i8) < 0
        }

        pub(crate) fn advance_anim_step(&mut self) {
            self.ram[INTRO_SWORD_ANIM_STEP] = self.ram[INTRO_SWORD_ANIM_STEP].wrapping_add(2);
        }

        pub(crate) fn set_sparkle_y_offset(&mut self, value: u8) {
            self.ram[INTRO_SWORD_SPARKLE_Y_OFFSET] = value;
        }

        pub(crate) fn advance_sparkle_y_offset(&mut self) {
            self.ram[INTRO_SWORD_SPARKLE_Y_OFFSET] =
                self.ram[INTRO_SWORD_SPARKLE_Y_OFFSET].wrapping_add(4);
        }

        pub(crate) fn set_flash_rgb_channel_word(&mut self, value: u16) {
            write_le_u16(self.ram, INTRO_SWORD_FLASH_RGB_CHANNEL, value);
        }

        pub(crate) fn cycle_flash_rgb_channel(&mut self) {
            self.ram[INTRO_SWORD_FLASH_RGB_CHANNEL] =
                if self.ram[INTRO_SWORD_FLASH_RGB_CHANNEL] == 2 {
                    0
                } else {
                    self.ram[INTRO_SWORD_FLASH_RGB_CHANNEL].wrapping_add(1)
                };
        }
    }

    fn byte(ram: &[u8], offset: usize) -> u8 {
        ram.get(offset).copied().unwrap_or(0)
    }

    fn word(ram: &[u8], offset: usize) -> u16 {
        if offset + 1 < ram.len() {
            read_le_u16(ram, offset)
        } else {
            0
        }
    }

    fn packed_position(ram: &[u8], low_offset: usize, high_offset: usize) -> u16 {
        u16::from(byte(ram, low_offset)) | (u16::from(byte(ram, high_offset)) << 8)
    }

    fn write_position(ram: &mut [u8], low_offset: usize, high_offset: usize, value: u16) {
        ram[low_offset] = value as u8;
        ram[high_offset] = (value >> 8) as u8;
    }

    fn move_axis24(
        ram: &mut [u8],
        subpixel_offset: usize,
        low_offset: usize,
        high_offset: usize,
        velocity_offset: usize,
    ) {
        let pos = u32::from(ram[subpixel_offset])
            | (u32::from(ram[low_offset]) << 8)
            | (u32::from(ram[high_offset]) << 16);
        let delta = ((ram[velocity_offset] as i8 as i32) << 4) as u32;
        let moved = pos.wrapping_add(delta);
        ram[subpixel_offset] = moved as u8;
        ram[low_offset] = (moved >> 8) as u8;
        ram[high_offset] = (moved >> 16) as u8;
    }

    fn move_axis16(ram: &mut [u8], subpixel_offset: usize, offset: usize, velocity_offset: usize) {
        let pos = (u16::from(ram[offset]) << 8) | u16::from(ram[subpixel_offset]);
        let delta = ((ram[velocity_offset] as i8 as i32) << 4) as u16;
        let moved = pos.wrapping_add(delta);
        ram[subpixel_offset] = moved as u8;
        ram[offset] = (moved >> 8) as u8;
    }

    fn move_link_axis_by_velocity(
        ram: &mut [u8],
        subpixel_offset: usize,
        coord_offset: usize,
        velocity: u8,
    ) -> u16 {
        let pos =
            u32::from(ram[subpixel_offset]) | (u32::from(read_le_u16(ram, coord_offset)) << 8);
        let delta = ((velocity as i8 as i32) << 4) as u32;
        let moved = pos.wrapping_add(delta);
        ram[subpixel_offset] = moved as u8;
        write_le_u16(ram, coord_offset, (moved >> 8) as u16);
        (moved >> 8) as u16
    }

    fn move_link_axis_by_subpixel_delta(
        ram: &mut [u8],
        subpixel_offset: usize,
        coord_offset: usize,
        delta: u16,
    ) -> u16 {
        let pos =
            u32::from(ram[subpixel_offset]) | (u32::from(read_le_u16(ram, coord_offset)) << 8);
        let moved = pos.wrapping_add(delta as i16 as i32 as u32);
        ram[subpixel_offset] = moved as u8;
        write_le_u16(ram, coord_offset, (moved >> 8) as u16);
        (moved >> 8) as u16
    }
}

pub(crate) mod player {
    pub(crate) const LAYER_COLLISION_FLAGS: usize = 0x322;
    pub(crate) const FAINT_ANIMATION_ACTIVE: usize = 0x36b;

    pub(crate) const LAYER_COLLISION_BG1: u8 = 0x01;
    pub(crate) const LAYER_COLLISION_BG2: u8 = 0x02;
    pub(crate) const LAYER_COLLISION_BOTH: u8 = LAYER_COLLISION_BG1 | LAYER_COLLISION_BG2;
}

pub(crate) mod nmi {
    pub(crate) const TILEMAP_UPLOAD_BUFFER: usize = 0x1000;
    pub(crate) const VRAM_UPLOAD_DATA: usize = 0x1002;
    pub(crate) const VRAM_UPLOAD_OFFSET: usize = 0x1000;
    pub(crate) const VRAM_UPLOAD_TILE_BUF: usize = 0x1100;
    pub(crate) const BG_CHAR_BUFFER: usize = 0x10000;
    pub(crate) const BG_CHAR_BUFFER_1: usize = 0x10800;
    pub(crate) const BG_CHAR_HALF_BUFFER: usize = 0x11000;
    pub(crate) const BG1_WALL_TOP_BUFFER: usize = 0x0c880;
    pub(crate) const BG1_WALL_BOTTOM_BUFFER: usize = 0x0c8c0;
    pub(crate) const GAME_OVER_TEXT_BUFFER: usize = 0x2000;
    pub(crate) const GAME_OVER_TEXT_TAIL_BUFFER: usize = 0x3400;
    pub(crate) const STRIPE_BUFFER_021B: usize = 0x021b;
    pub(crate) const ARBITRARY_TILEMAP_DST_BUFFER: usize = 0x14000;
}

pub(crate) mod messaging {
    pub(crate) const MODULE: usize = 0x1cd8;
    pub(crate) const DIALOGUE_MESSAGE_INDEX: usize = 0x1cf0;
    pub(crate) const CHOICE_IN_MULTISELECT_BOX: usize = 0x1ce8;
    pub(crate) const CHOICE_IN_MULTISELECT_BOX_BAK: usize = 0x1cf4;
    pub(crate) const TEXT_MSGBOX_TOPLEFT_COPY: usize = 0x1cd0;
    pub(crate) const TEXT_MSGBOX_TOPLEFT: usize = 0x1cd2;
    pub(crate) const TEXT_RENDER_STATE: usize = 0x1cd4;
    pub(crate) const VWF_LINE_SPEED_CUR: usize = 0x1cd5;
    pub(crate) const VWF_LINE_SPEED: usize = 0x1cd6;
    pub(crate) const TEXT_INCREMENTAL_STATE: usize = 0x1cd7;
    pub(crate) const DIALOGUE_MSG_READ_POS: usize = 0x1cd9;
    pub(crate) const DIALOGUE_TEXT_COLOR: usize = 0x1cdc;
    pub(crate) const DIALOGUE_MSG_SRC_OFFS: usize = 0x1cdd;
    pub(crate) const TEXT_WAIT_COUNTDOWN: usize = 0x1ce0;
    pub(crate) const TEXT_TILEMAP_CUR: usize = 0x1ce2;
    pub(crate) const TEXT_WAIT_COUNTDOWN2: usize = 0x1ce9;
    pub(crate) const DIALOGUE_SCROLL_SPEED: usize = 0x1cea;
    pub(crate) const MESSAGE_DMA_DST_ADDR: usize = 0x219;
    pub(crate) const MESSAGE_DMA_TILE_BASE: usize = 0x21d;
    pub(crate) const MESSAGE_DMA_TILE_LIMIT: usize = 0x21f;
    pub(crate) const MESSAGE_DMA_TILE_SENTINEL: usize = 0x221;
    pub(crate) const VWF_FLAG_NEXT_LINE: usize = 0x720;
    pub(crate) const VWF_CURLINE: usize = 0x722;
    pub(crate) const VWF_LINE_PTR: usize = 0x726;
    pub(crate) const VWF_ARR: usize = 0x0c230;
    pub(crate) const TEXT_BUFFER: usize = 0x11200;
}
