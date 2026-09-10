//! WRAM/SRAM address map entries that no Rust code reads any more.
//!
//! They stay as `const NAME: usize = 0xADDR;` definitions because
//! `scripts/ram_ref.py` (behind `whoowns.py` and `find_dual_ownership.py`)
//! scans the repository's constant definitions to name addresses. Move an
//! entry back next to its consumer when code starts reading it again.
#![allow(dead_code)]

const FOLLOWER_KIKI_ANIM_COUNTER: usize = 0x0b69;

const AGAHNIM_PAL_SETTING: usize = 0x0c019;

const PEG_TILE_GFX_BUFFER: usize = 0xb340;

const PALETTE_SWAP_FLAG: usize = 0x0abd;

const AUX_BG_SUBSET_3: usize = 0x0c2fb;

const AUX_BG_SUBSET_2: usize = 0x0c2fa;

const AUX_BG_SUBSET_1: usize = 0x0c2f9;

const DUNGEON_MAP_CURRENT_FLOOR: usize = 0x020e;

const SCRATCH_A: usize = 0x0073;

const OVERWORLD_SCROLL_RIGHT_COUNTER_EXIT: usize = 0x0c170;

const OVERWORLD_SCROLL_LEFT_COUNTER_EXIT: usize = 0x0c16e;

const OVERWORLD_SCROLL_DOWN_COUNTER_EXIT: usize = 0x0c16c;

const LEFT_RIGHT_SCROLL_TARGET_END_EXIT: usize = 0x0c162;

const LEFT_RIGHT_SCROLL_TARGET_EXIT: usize = 0x0c160;

const UP_DOWN_SCROLL_TARGET_END_EXIT: usize = 0x0c15e;

const OVERWORLD_SCROLL_RIGHT_COUNTER_SPEXIT: usize = 0x0c130;

const OVERWORLD_SCROLL_LEFT_COUNTER_SPEXIT: usize = 0x0c12e;

const OVERWORLD_SCROLL_DOWN_COUNTER_SPEXIT: usize = 0x0c12c;

const OVERWORLD_TILE_THEME_INDEX_SPEXIT: usize = 0x0c124;

const LEFT_RIGHT_SCROLL_TARGET_END_SPEXIT: usize = 0x0c122;

const LEFT_RIGHT_SCROLL_TARGET_SPEXIT: usize = 0x0c120;

const UP_DOWN_SCROLL_TARGET_END_SPEXIT: usize = 0x0c11e;

const LEFT_RIGHT_SCROLL_TARGET_END: usize = 0x0616;

const UP_DOWN_SCROLL_TARGET_END: usize = 0x0612;

const OVERWORLD_SCROLL_RIGHT_COUNTER: usize = 0x062a;

const OVERWORLD_SCROLL_DOWN_COUNTER: usize = 0x0626;

const CAMERA_X: usize = 0x061c;

const CAMERA_Y: usize = 0x0618;

const DUNG_TORCH_DATA: usize = 0x0fb40;

const DUNG_FLOOR_1_FILLER_TILES: usize = 0x0490;

const DUNG_FLOOR_2_FILLER_TILES: usize = 0x046a;

const PUSH_BLOCK_DIRECTION_DUNGEON: usize = 0x0474;

const LINK_EQUIPPED_ITEM: usize = 0x0f340;

const BITFIELD_SPIKE_CACTUS_TILES: usize = 0x02e8;

const TEXT_BUFFER: usize = 0x11200;

const VWF_ARR: usize = 0x0c230;

const VWF_LINE_PTR: usize = 0x726;

const VWF_CURLINE: usize = 0x722;

const VWF_FLAG_NEXT_LINE: usize = 0x720;

const DIALOGUE_SCROLL_SPEED: usize = 0x1cea;

const TEXT_WAIT_COUNTDOWN2: usize = 0x1ce9;

const TEXT_TILEMAP_CUR: usize = 0x1ce2;

const TEXT_WAIT_COUNTDOWN: usize = 0x1ce0;

const DIALOGUE_MSG_SRC_OFFS: usize = 0x1cdd;

const DIALOGUE_MSG_READ_POS: usize = 0x1cd9;

const TEXT_INCREMENTAL_STATE: usize = 0x1cd7;

const VWF_LINE_SPEED: usize = 0x1cd6;

const VWF_LINE_SPEED_CUR: usize = 0x1cd5;

const TEXT_RENDER_STATE: usize = 0x1cd4;

const TEXT_MSGBOX_TOPLEFT: usize = 0x1cd2;

const TEXT_MSGBOX_TOPLEFT_COPY: usize = 0x1cd0;

const CHOICE_IN_MULTISELECT_BOX_BAK: usize = 0x1cf4;

const CHOICE_IN_MULTISELECT_BOX: usize = 0x1ce8;

const DIALOGUE_MESSAGE_INDEX: usize = 0x1cf0;

const FAINT_ANIMATION_ACTIVE: usize = 0x36b;

const LAYER_COLLISION_FLAGS: usize = 0x322;

const FALL_HOLE_SCAN_INDEX_LOCAL: usize = 0x02c9;

const FIRST_BOOT_NMI_DMA_SOURCE_BYTE_2: usize = 0x0002;

const RESERVED_GFX_CONFIG_WORD: usize = 0x0aa6;

const DUNG_EXIT_DOOR_ADDRESSES: usize = 0x19e2;

const DUNG_EXIT_DOOR_COUNT: usize = 0x19e0;

const DUNG_DOOR_DIRECTION: usize = 0x19c0;

const DUNG_DOOR_TILEMAP_ADDRESS: usize = 0x19a0;

const DOOR_TYPE_AND_SLOT: usize = 0x1980;

const POLY_RASTER_NUMFULL: usize = 0x1ffa;

const POLY_CUR_VERTEX_IDX1: usize = 0x1ff2;

const POLY_X1_STEP: usize = 0x1ff0;

const POLY_X1_FRAC: usize = 0x1fee;

const POLY_Y1_TRIG: usize = 0x1fed;

const POLY_X1_TARGET: usize = 0x1fec;

const POLY_Y1_CUR: usize = 0x1feb;

const POLY_X1_CUR: usize = 0x1fea;

const POLY_CUR_VERTEX_IDX0: usize = 0x1fe9;

const POLY_X0_STEP: usize = 0x1fe7;

const POLY_X0_FRAC: usize = 0x1fe5;

const POLY_Y0_TRIG: usize = 0x1fe4;

const POLY_X0_TARGET: usize = 0x1fe3;

const POLY_Y0_CUR: usize = 0x1fe2;

const POLY_X0_CUR: usize = 0x1fe1;

const POLY_TOTAL_NUM_STEPS: usize = 0x1fe0;

const POLY_XY_COORDS: usize = 0x1fc0;

const POLY_TMP2: usize = 0x1fbc;

const POLY_RASTER_DST_PTR: usize = 0x1fb9;

const POLY_RASTER_COLOR1: usize = 0x1fb7;

const POLY_RASTER_COLOR0: usize = 0x1fb5;

const POLY_TMP1: usize = 0x1fb2;

const POLY_TMP0: usize = 0x1fb0;

const POLY_E1: usize = 0x1f5e;

const POLY_E3: usize = 0x1f5c;

const POLY_E2: usize = 0x1f5a;

const POLY_E0: usize = 0x1f58;

const POLY_COS_B: usize = 0x1f56;

const POLY_SIN_B: usize = 0x1f54;

const POLY_COS_A: usize = 0x1f52;

const POLY_SIN_A: usize = 0x1f50;

const POLY_RASTER_COLOR_CONFIG: usize = 0x1f4f;

const POLY_NUM_VERTEX_IN_POLY: usize = 0x1f4e;

const POLY_F2: usize = 0x1f4c;

const POLY_F1: usize = 0x1f4a;

const POLY_F0: usize = 0x1f48;

const POLY_FROMLUT_X: usize = 0x1f47;

const POLY_FROMLUT_Y: usize = 0x1f46;

const POLY_FROMLUT_Z: usize = 0x1f45;

const POLY_FROMLUT_PTR4: usize = 0x1f43;

const POLY_FROMLUT_PTR2: usize = 0x1f41;

const POLY_CONFIG_NUM_POLYS: usize = 0x1f40;

const POLY_CONFIG_NUM_VERTEX: usize = 0x1f3f;

const POLY_BASE_Y: usize = 0x1f07;

const POLY_BASE_X: usize = 0x1f06;

const POLY_B: usize = 0x1f05;

const POLY_A: usize = 0x1f04;

const POLY_WHICH_MODEL: usize = 0x1f03;

const POLY_CONFIG1: usize = 0x1f02;

const POLY_CONFIG_COLOR_MODE: usize = 0x1f01;

const INTRO_SPRITE_ALLOC: usize = 0x1e08;

const INTRO_STEP_TIMER: usize = 0x1e01;

const DUNG_HDR_TRAVEL_DESTINATIONS: usize = 0x0c000;

const FOLLOWER_DROPPED: usize = 0x0f3d3;

const TEXT_DIALOGUE_POINTERS: usize = 0x171c0;

const SAVEGAME_MAP_ICONS_INDICATOR: usize = 0x0f3c7;

const LINK_ABILITY_FLAGS: usize = 0xf379;

const SPRITE_GFX_SUBSET_3: usize = 0x0c2ff;

const SPRITE_GFX_SUBSET_2: usize = 0x0c2fe;

const SPRITE_GFX_SUBSET_1: usize = 0x0c2fd;

const SPRITE_GFX_SUBSET_0: usize = 0x0c2fc;

const ORANGE_BLUE_BARRIER_STATE: usize = 0x0c172;

const DUNGEON_TRAP_TRIGGER_LATCH: usize = 0x0b9e;

const DUNG_DOOR_OPENED_INCL_ADJACENT: usize = 0x68c;

const DUNG_HDR_HOLE_TELEPORTER_PLANE: usize = 0x63c;

const DOOR_DEBRIS_Y: usize = 0x3ba;

const DOOR_DEBRIS_X: usize = 0x3b6;

const BIG_ROCK_STARTING_ADDRESS: usize = 0x698;

const DOOR_OPEN_CLOSED_COUNTER: usize = 0x692;

const OW_ENTRANCE_VALUE: usize = 0x696;

const OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP: usize = 0x0c20c;

const OVERWORLD_PAL_AUX3_BP7_BACKUP: usize = 0x0c20b;

const OVERWORLD_PAL_MAIN_INDOORS_BACKUP: usize = 0x0c20a;

const OVERWORLD_EXIT_TILE_THEME_INDEX: usize = 0x0c164;

const LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED: usize = 0x0c1a8;

const LINK_QUADRANT_X_CACHED: usize = 0x0c19e;

const QUADRANT_FULLSIZE_X_CACHED: usize = 0x0c19c;

const CAMERA_X_COORD_SCROLL_LOW_CACHED: usize = 0x0c19a;

const LEFT_RIGHT_SCROLL_TARGET_CACHED: usize = 0x0c194;

const UP_DOWN_SCROLL_TARGET_END_CACHED: usize = 0x0c192;

const CACHED_ROOM_BOUNDS_X_END: usize = 0x0c18e;

const CACHED_ROOM_BOUNDS_X_START: usize = 0x0c18c;

const CACHED_ROOM_BOUNDS_Y_END: usize = 0x0c18a;

const CACHED_ROOM_BOUNDS_Y_START: usize = 0x0c188;

const DUNG_FLAG_STATECHANGE_WATERPUZZLE: usize = 0x642;

const DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED: usize = 0x641;

const CAMERA_X_COORD_SCROLL_HI: usize = 0x61e;

const CAMERA_X_COORD_SCROLL_LOW: usize = 0x61c;

const CAMERA_Y_COORD_SCROLL_HI: usize = 0x61a;

const CAMERA_Y_COORD_SCROLL_LOW: usize = 0x618;

// LEFT_RIGHT_SCROLL_TARGET_END is also defined (with a different value) in another module.
// const LEFT_RIGHT_SCROLL_TARGET_END: usize = 0x616;

const LEFT_RIGHT_SCROLL_TARGET: usize = 0x614;

// UP_DOWN_SCROLL_TARGET_END is also defined (with a different value) in another module.
// const UP_DOWN_SCROLL_TARGET_END: usize = 0x612;

const UP_DOWN_SCROLL_TARGET: usize = 0x610;

const OVERWORLD_MAP_FLAGS: usize = 0x636;

const MODE7_ZOOM_STEP_COUNTER: usize = 0x635;

const TIMER_FOR_MODE7_ZOOM: usize = 0x637;

const OVERWORLD_FIXED_COLOR_PLUSMINUS: usize = 0x0c017;

const LINK_DMA_TILE_OFFSET: usize = 0x0c015;

const GARNISH_TYPE: usize = 0x1f800;

const CUR_SPRITE_Y: usize = 0x0fda;

const CUR_SPRITE_X: usize = 0x0fd8;

const SPRITE_ROOM_ORIGIN_Y_HI: usize = 0x0fb1;

const RNG_SEED: usize = 0x0fa1;

const BIRDTRAVEL_STATUS: usize = 0x1af0;

const REPULSESPARK_Y_LO: usize = 0x0fae;

const REPULSESPARK_X_LO: usize = 0x0fad;

const SPRITE_BUMP_DAMAGE: usize = 0x0cd2;

const SPRITE_HIT_TIMER: usize = 0x0ef0;

const SPRITE_ANIM_CLOCK: usize = 0x0ec0;

const SPRITE_WALLCOLL: usize = 0x0e70;

const SPRITE_HEALTH: usize = 0x0e50;

const SPRITE_FLAGS: usize = 0x0b6b;

const SPRITE_G: usize = 0x0ed0;

const SPRITE_F: usize = 0x0ea0;

const SPRITE_Z_SUBPOS: usize = 0x0f90;

const SPRITE_Z_VEL: usize = 0x0f80;

const SPRITE_Z: usize = 0x0f70;

const SPRITE_FLAGS4: usize = 0x0f60;

const SPRITE_X_RECOIL: usize = 0x0f40;

const SPRITE_FLOOR: usize = 0x0f20;

const SPRITE_DELAY_AUX4: usize = 0x0f10;

const SPRITE_DELAY_AUX2: usize = 0x0e10;

const SPRITE_HEAD_DIR: usize = 0x0eb0;

const SPRITE_SUBTYPE2: usize = 0x0e80;

const SPRITE_FLAGS3: usize = 0x0e60;

const SPRITE_FLAGS2: usize = 0x0e40;

const SPRITE_SUBTYPE: usize = 0x0e30;

const SPRITE_IGNORE_PROJECTILE: usize = 0x0ba0;

const SPRITE_DELAY_AUX1: usize = 0x0e00;

const SPRITE_D: usize = 0x0de0;

const SPRITE_GRAPHICS: usize = 0x0dc0;

const SPRITE_OBJ_PRIO: usize = 0x0b89;

const SPRITE_C: usize = 0x0db0;

const SPRITE_B: usize = 0x0da0;

const SPRITE_X_SUBPIXEL: usize = 0x0d70;

const SPRITE_Y_SUBPIXEL: usize = 0x0d60;

const SPRITE_X_HI: usize = 0x0d30;

const SPRITE_Y_HI: usize = 0x0d20;

const SPRITE_Y_LO: usize = 0x0d00;

const SPRITE_DIE_ACTION: usize = 0x0cba;

const SPRITE_ROOM: usize = 0x0c9a;

const ANCILLA_OAM_IDX: usize = 0x0c86;

const OVERWORLD_TILE_THEME_INDEX: usize = 0x0aa0;

const RUPEE_SFX_SOUND_DELAY: usize = 0x0cfd;

const SPRITE_N: usize = 0x0bc0;

const HUD_INVENTORY_ORDER: usize = 0x0225;

const LINK_DEBUG_VALUE_1: usize = 0x20b;

const ATTRACT_MAIDEN_WARP_TIMER_B: usize = 0x63;

const ATTRACT_MAIDEN_WARP_TIMER_A: usize = 0x62;

const ATTRACT_SUBSTEP_DELAY_COUNTER: usize = 0x61;

const ATTRACT_FADE_IN_DONE_FLAG: usize = 0x5f;

const ATTRACT_FADE_IN_COMPLETE_FLAG: usize = 0x52;

const ATTRACT_THRONE_FADE_TIMER: usize = 0x2c;

const ATTRACT_BG2_VOFS_BACKUP: usize = 0x20;

const ATTRACT_LEGEND_CTR: usize = 0x200;

const ATTRACT_SCENE_DONE_FLAG: usize = 0x5d;

const ATTRACT_SCENE_FRAME_COUNTER: usize = 0x50;

const ATTRACT_PRISON_SOLDIER_X_LO: usize = 0x34;

const ATTRACT_SOLDIER_ANIM_STEP: usize = 0x33;

const ATTRACT_ANIM_STEP_COUNTER: usize = 0x32;

const ATTRACT_VRAM_DST: usize = 0x30;

const ATTRACT_PRISON_ZELDA_Y_BASE: usize = 0x2b;

const ATTRACT_LEGEND_FLAG: usize = 0x27;

const VWF_TILE_BUFFER: usize = 0x1300;

const ENEMY_DAMAGE_DATA: usize = 0x16000;

const ATTRIBUTES_FOR_TILE: usize = 0x0fe00;

const OVERWORLD_SPRITE_PALETTES: usize = 0x0fd40;

const OVERWORLD_SPRITE_GFX: usize = 0x0fcc0;

const LINK_DUNGEON_MAP: usize = 0x0f368;

const LINK_BIGKEY: usize = 0x0f366;

const LINK_COMPASS: usize = 0x0f364;

const LINK_KEYS_EARNED_PER_DUNGEON: usize = 0x0f37c;

const LINK_ARMOR: usize = 0x0f35b;

const LINK_HAS_CRYSTALS: usize = 0x0f37a;

const LINK_WHICH_PENDANTS: usize = 0x0f374;

const LINK_ARROW_UPGRADES: usize = 0x0f371;

const LINK_BOMB_UPGRADES: usize = 0x0f370;

const LINK_HEALTH_CURRENT: usize = 0x0f36d;

const LINK_HEALTH_CAPACITY: usize = 0x0f36c;

const LINK_HEART_PIECES: usize = 0x0f36b;

const LINK_RUPEES_ACTUAL: usize = 0x0f362;

const LINK_BOTTLE_INFO: usize = 0x0f35c;

const LINK_SHIELD_TYPE: usize = 0x0f35a;

const LINK_SWORD_TYPE: usize = 0x0f359;

const SAVEGAME_IS_DARKWORLD: usize = 0x0f3ca;

const WHICH_STARTING_POINT: usize = 0x0f3c8;

const SRAM_PROGRESS_FLAGS: usize = 0x0f3c6;

const SRAM_PROGRESS_INDICATOR: usize = 0x0f3c5;

const LINK_ITEM_MIRROR: usize = 0x0f353;

const LINK_ITEM_CAPE: usize = 0x0f352;

const LINK_ITEM_BOOTS: usize = 0x0f355;

const LINK_ITEM_GLOVES: usize = 0x0f354;

const LINK_ITEM_CANE_BYRNA: usize = 0x0f351;

const LINK_ITEM_CANE_SOMARIA: usize = 0x0f350;

const LINK_ITEM_BOTTLE_INDEX: usize = 0x0f34f;

const LINK_ITEM_BOOK: usize = 0x0f34e;

const LINK_ITEM_BUG_NET: usize = 0x0f34d;

const LINK_ITEM_FLUTE: usize = 0x0f34c;

const LINK_ITEM_HAMMER: usize = 0x0f34b;

const LINK_ITEM_TORCH: usize = 0x0f34a;

const LINK_ITEM_QUAKE: usize = 0x0f349;

const LINK_ITEM_ETHER: usize = 0x0f348;

const LINK_ITEM_BOMBOS: usize = 0x0f347;

const LINK_ITEM_ICE_ROD: usize = 0x0f346;

const LINK_ITEM_FIRE_ROD: usize = 0x0f345;

const LINK_ITEM_MUSHROOM: usize = 0x0f344;

const LINK_ITEM_HOOKSHOT: usize = 0x0f342;

const LINK_ITEM_BOOMERANG: usize = 0x0f341;

const PALETTE_MAIN_INDOORS_COPY: usize = 0x0ab7;

const OVERWORLD_PALETTE_AUX1_BP2TO4_HI: usize = 0x0ab4;

const EQUIPMENT_MENU_EXIT_STATE: usize = 0x034b;

const IS_DOING_HEART_ANIMATION: usize = 0x020a;

const ANIMATE_HEART_REFILL_COUNTDOWN_SUBPOS: usize = 0x0209;

const BOTTLE_MENU_EXPAND_ROW: usize = 0x0205;

const HUD_TMP1: usize = 0x0bd;

const HUD_CUR_ITEM_R: usize = 0x0658;

const HUD_CUR_ITEM_L: usize = 0x0657;

const HUD_CUR_ITEM_X: usize = 0x0656;

const ANIMATE_HEART_REFILL_COUNTDOWN: usize = 0x0208;

const TIMER_FOR_FLASHING_CIRCLE: usize = 0x0207;

const HUD_MODULE_TICK_COUNTER: usize = 0x0206;

const HUD_CUR_ITEM: usize = 0x0202;

const MISC_SPRITES_GRAPHICS_INDEX: usize = 0x0aa4;

const SPRITE_GRAPHICS_INDEX: usize = 0x0aa3;

const AUX_TILE_THEME_INDEX: usize = 0x0aa2;

const MAIN_TILE_THEME_INDEX: usize = 0x0aa1;

const DUNG_CHEST_LOCATIONS: usize = 0x6e0;

const DUNG_STAIRS_TABLE_1: usize = 0x6b8;

const DUNG_INTER_STARCASES: usize = 0x6b0;

const REPLACEMENT_TILEMAP_LR: usize = 0x5c0;

const REPLACEMENT_TILEMAP_UR: usize = 0x5a0;

const REPLACEMENT_TILEMAP_LL: usize = 0x580;

const REPLACEMENT_TILEMAP_UL: usize = 0x560;

const DUNG_OBJECT_TILEMAP_POS: usize = 0x540;

const DUNG_OBJECT_POS_IN_OBJDATA: usize = 0x520;

const DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS: usize = 0x4a8;

const DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS: usize = 0x4a6;

const DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS: usize = 0x4a4;

const DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS: usize = 0x4a2;

const DUNG_OVERLAY_TO_LOAD: usize = 0x4ba;

const DUNG_NUM_BIGKEY_LOCKS_X2: usize = 0x498;

const DUNG_NUM_CHESTS_X2: usize = 0x496;

const DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2: usize = 0x484;

const DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2: usize = 0x482;

const DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS: usize = 0x480;

const DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS: usize = 0x47e;

const DUNG_INDEX_OF_TORCHES_START: usize = 0x478;

const SPRITE_WHERE_IN_ROOM: usize = 0x1df80;

const TAGALONG_LAYERBITS: usize = 0x1a64;

const TAGALONG_X_HI: usize = 0x1a3c;

const TAGALONG_X_LO: usize = 0x1a28;

const TAGALONG_Y_HI: usize = 0x1a14;

const TAGALONG_Y_LO: usize = 0x1a00;

const ANCILLA_NUMSPR: usize = 0x0c90;

const ANCILLA_FLOOR: usize = 0x0c7c;

const ANCILLA_DIR: usize = 0x0c72;

const ANCILLA_TIMER: usize = 0x0c68;

const ANCILLA_ITEM_TO_LINK: usize = 0x0c5e;

const ANCILLA_STEP: usize = 0x0c54;

const ANCILLA_X_SUBPIXEL: usize = 0x0c40;

const ANCILLA_Y_SUBPIXEL: usize = 0x0c36;

const ANCILLA_Y_VEL: usize = 0x0c22;

const ANCILLA_X_LO: usize = 0x0c04;

const ANCILLA_FLOOR2: usize = 0x3ca;

const ANCILLA_H: usize = 0x3c5;

const ANCILLA_Z: usize = 0x29e;

const ANCILLA_Z_VEL: usize = 0x294;

const ANCILLA_U: usize = 0x28a;

const ANCILLA_OBJPRIO: usize = 0x280;

const OAM_ALLOC_ARR1: usize = 0x0fec;

const INDEX_OF_CHANGABLE_DUNGEON_OBJS: usize = 0x5fc;

const PUSHEDBLOCKS_SUBPIXEL: usize = 0x5f4;

const PUSHEDBLOCKS_Y_LO: usize = 0x5f0;

const PUSHEDBLOCKS_Y_HI: usize = 0x5ec;

const PUSHEDBLOCKS_TARGET: usize = 0x5e8;

const PUSHEDBLOCKS_X_LO: usize = 0x5e4;

const PUSHEDBLOCKS_X_HI: usize = 0x5e0;

const ARCHERY_GAME_OUT_OF_ARROWS: usize = 0x0b9a;

const ARCHERY_GAME_ARROWS_LEFT: usize = 0x0b99;

const OVERWORLD_BOULDER_TRAP_TIMER: usize = 0x0ffe;

const OVERWORLD_BOULDER_TRAP_COUNT: usize = 0x0ffd;

const HAUNTED_GROVE_FLUTE_EVENT_LATCH: usize = 0x0fdd;

const SPRITE_ALERT_FLAG: usize = 0x0fdc;

const SPRITE_CHR_HALFSLOT_STATE: usize = 0x0fc6;

const SPRCOLL_Y_SIZE: usize = 0x0fba;

const SPRCOLL_X_SIZE: usize = 0x0fb8;

const FLAG_BLOCK_LINK_MENU: usize = 0x0ffc;

const SPRITE_SHARED_WORK_A: usize = 0x0fb6;

const SPRITE_ROOM_ORIGIN_X_HI: usize = 0x0fb0;

const DUNG_FLAG_SOMARIA_BLOCK_SWITCH: usize = 0x646;

const DUNGEON_ROOM_HISTORY: usize = 0x0b80;

const LINK_PREVENT_FROM_MOVING: usize = 0x0b7b;

const SPRITE_STUNNED: usize = 0x0b58;

const SPRITE_LIMIT_INSTANCE: usize = 0x0b6a;

const OVERWORLD_OFFSET_MASK_X: usize = 0x70e;

const OVERWORLD_OFFSET_BASE_X: usize = 0x70c;

const OVERWORLD_OFFSET_MASK_Y: usize = 0x70a;

const OVERWORLD_OFFSET_BASE_Y: usize = 0x708;

const SPOTLIGHT_WINDOW_STATE: usize = 0x67e;

const SPOTLIGHT_WINDOW_Y_BUFFER: usize = 0x67a;

const SPOTLIGHT_Y_UPPER: usize = 0x676;

const SPOTLIGHT_Y_LOWER: usize = 0x674;

const SPOTLIGHT_WINDOW_X_CENTER: usize = 0x670;

const DUNG_LOADE_BGOFFS_V_COPY: usize = 0x62e;

const DUNG_LOADE_BGOFFS_H_COPY: usize = 0x62c;

const LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP: usize = 0x04ca;

const FLAG_SKIP_CALL_TAG_ROUTINES: usize = 0x4c7;

const NUM_MEMORIZED_TILES: usize = 0x4ac;

// DUNG_FLOOR_1_FILLER_TILES is also defined (with a different value) in another module.
// const DUNG_FLOOR_1_FILLER_TILES: usize = 0x490;

// DUNG_FLOOR_2_FILLER_TILES is also defined (with a different value) in another module.
// const DUNG_FLOOR_2_FILLER_TILES: usize = 0x46a;

const CRUSH_WALL_PROGRESS: usize = 0x454;

const DUNG_CUR_QUADRANT_UPLOAD: usize = 0x45c;

const DUNG_NUM_LIT_TORCHES: usize = 0x45a;

const KIND_OF_IN_ROOM_STAIRCASE: usize = 0x44a;

const DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS: usize = 0x43a;

const DUNG_INDEX_OF_TORCHES: usize = 0x42e;

const DUNG_MISC_OBJS_INDEX: usize = 0x42c;

const HDR_DUNGEON_DARK_WITH_LANTERN: usize = 0x458;

const DUNG_HDR_BG2_PROPERTIES: usize = 0x414;

const CUR_PALACE_INDEX_X2: usize = 0x40c;

const GANON_TORCH_COUNT: usize = 0x4c5;

const OVERWORLD_HOLE_TILEMAP_POS: usize = 0x4b2;

const DUNGEON_ROOM_INDEX2: usize = 0x48e;

const DUNG_HDR_COLLISION_2_MIRROR: usize = 0x428;

const DUNG_FLOOR_Y_OFFS: usize = 0x424;

const DUNG_FLOOR_X_OFFS: usize = 0x422;

const INVISIBLE_DOOR_DIR_AND_INDEX_X2: usize = 0x436;

const DUNG_DOOR_OPENED: usize = 0x400;

const DUNG_CUR_DOOR_IDX: usize = 0x460;

const BG1_MOVE_CALC_BUFFER: usize = 0x41c;

const DUNG_LAYOUT_AND_STARTING_QUADRANT: usize = 0x40e;

const DUNG_SAVEGAME_STATE_BITS: usize = 0x402;

const PLAYER_SPECIAL_DRAW_FLAG: usize = 0x3fd;

const IS_ARCHER_OR_SHOVEL_GAME: usize = 0x3fc;

const BIT9_OF_XCOORD: usize = 0x3fa;

const LINK_NEED_FOR_PULLFORRUPEES_SPRITE: usize = 0x3f8;

const DUNG_HDR_COLLISION: usize = 0x46c;

const TILE_COLLISION_BITS_SECONDARY: usize = 0x317;

const TILE_COLLISION_BITS_PRIMARY: usize = 0x316;

const TILE_COLL_FLAG: usize = 0x315;

const SOMARIA_BLOCK_BG_CHECK_FLAG: usize = 0x3f4;

const HOOKSHOT_BG_CHECK_OFF_TIMER: usize = 0x3f9;

const MOVING_FLOOR_BG_CHECK_FLAGS: usize = 0x3f1;

const LINK_FORCE_HOLD_SWORD_UP: usize = 0x3ef;

const LINK_SOMETHING_WITH_HOOKSHOT: usize = 0x3e9;

const ANCILLA_G: usize = 0x394;

const ANCILLA_B: usize = 0x38f;

const ANCILLA_A: usize = 0x38a;

const ANCILLA_L: usize = 0x385;

const ANCILLA_K: usize = 0x380;

const PLAYER_POSE_DRAW_COUNTER: usize = 0x379;

const DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ: usize = 0x36e;

const LIFTABLE_TILE_DETECTED_INDEX_DOUBLED: usize = 0x36a;

const TILEDETECT_VERTICAL_LEDGE: usize = 0x36d;

const TILE_ACTION_INDEX: usize = 0x36c;

const LINK_PULL_ACTION_STATE: usize = 0x377;

const LINK_POSE_DURING_OPENING: usize = 0x37d;

const PLAYER_SLEEP_IN_BED_STATE: usize = 0x37c;

const LINK_TIMER_PUSH_GET_TIRED: usize = 0x371;

const LIFTABLE_TILE_ACTION_INDEX_PRIMARY: usize = 0x368;

const TILEDETECT_THICK_GRASS: usize = 0x357;

const LIFTABLE_TILE_ACTION_INDEX_SECONDARY: usize = 0x369;

const OAM_PRIORITY_VALUE_2: usize = 0x35d;

const VALUE_COMPUTED_FOR_PLAYER_OAM: usize = 0x354;

const SORT_SPRITES_OFFSET_INTO_OAM_BUFFER: usize = 0x352;

const FLAG_IS_SPRITE_TO_PICK_UP: usize = 0x314;

const LINK_PALETTE_BITS_OF_OAM: usize = 0x346;

const TILEDETECT_DEEPWATER: usize = 0x341;

const DUNGEON_TORCH_ATTR: usize = 0x333;

const LINK_DIRECTION_FACING_MIRROR: usize = 0x323;

const RELATED_TO_MOVING_FLOOR_X: usize = 0x31a;

const RELATED_TO_MOVING_FLOOR_Y: usize = 0x318;

const LINK_SPIN_OFFSETS: usize = 0x31e;

const STATE_FOR_SPIN_ATTACK: usize = 0x31c;

const Y_BUTTON_ACTION_TIMER: usize = 0x30b;

const Y_BUTTON_ACTION_STEP: usize = 0x30a;

const OVERWORLD_SCREEN_TRANSITION: usize = 0x418;

const OVERWORLD_SCREEN_TRANS_DIR_BITS2: usize = 0x416;

const OVERWORLD_SCREEN_TRANS_DIR_BITS: usize = 0x410;

const DUNG_FLOOR_Y_VEL: usize = 0x310;

const CACHED_TILE_ACTION_INDEX: usize = 0x306;

const CURRENT_ITEM_ACTIVE: usize = 0x304;

const CURRENT_ITEM_Y: usize = 0x303;

const OVERWORLD_MUSIC: usize = 0x15b00;

const TAGALONG_APPEARANCE_NONE_FLAG: usize = 0x2f9;

const TAGALONG_EVENT_FLAGS: usize = 0x2f2;

const TILEDETECT_MISC_TILES: usize = 0x2f6;

const FLAG_IS_SPRITE_TO_PICK_UP_CACHED: usize = 0x2f4;

const FLAG_IS_ANCILLA_TO_PICK_UP: usize = 0x2ec;

const TILEDETECT_INROOM_STAIRCASE: usize = 0x2c0;

const FALL_HOLE_SCAN_INDEX: usize = 0x2c9;

const PIT_CORRECTION_TIMER: usize = 0x2ca;

const LINK_SWORD_DELAY_TIMER: usize = 0x2e3;

const LINK_IS_TRANSFORMING: usize = 0x2e1;

const LINK_Y_COORD_COPY: usize = 0x2de;

const LINK_X_COORD_COPY: usize = 0x2dc;

const TAGALONG_ANIM_FRAME_COUNTER: usize = 0x2d7;

const TAGALONG_JUMP_TIMER: usize = 0x2d6;

const TAGALONG_SHARED_STATE_A: usize = 0x2d4;

const TAGALONG_DATA_INDEX: usize = 0x2cf;

const LINK_INCAPACITATED_CAMERA_TIMER: usize = 0x2c5;

const PUSHED_BLOCK_MODE: usize = 0x2c3;

const MAPBAK_HDMAEN: usize = 0x0c229;

const MAPBAK_CGWSEL: usize = 0x0c225;

const LINK_X_COORD_SPEXIT: usize = 0x0c10a;

const LINK_Y_COORD_SPEXIT: usize = 0x0c108;

const MAPBAK_TS: usize = 0x0c212;

const MAPBAK_TM: usize = 0x0c211;

const GAME_OVER_CHECK_FLAG: usize = 0x10a;

const OAM_PRIORITY_VALUE: usize = 0x64;

const OVERWORLD_HOLE_SCAN_STEP: usize = 0x10f;

const WHICH_ENTRANCE: usize = 0x10e;

const JOYPAD1L_LAST2: usize = 0xfa;

const JOYPAD1H_LAST2: usize = 0xf8;

const FILTERED_JOYPAD_L: usize = 0xf6;

const FILTERED_JOYPAD_H: usize = 0xf4;

const JOYPAD1L_LAST: usize = 0xf2;

const JOYPAD1H_LAST: usize = 0xf0;

const LINK_TILE_BELOW: usize = 0x114;

const LINK_DMA_SHIELD_GRAPHICS_INDEX: usize = 0x108;

const LINK_DMA_SWORD_GRAPHICS_INDEX: usize = 0x107;

const LINK_DMA_RIGHT_SPRITE_BANK_INDEX: usize = 0x104;

const LINK_DMA_LEFT_SPRITE_BANK_INDEX: usize = 0x102;

const INTRO_SWORD_24: usize = 0xd0;

const INTRO_SWORD_21: usize = 0xcd;

const INTRO_SWORD_20: usize = 0xcc;

const INTRO_SWORD_19: usize = 0xcb;

const INTRO_SWORD_18: usize = 0xca;

const INTRO_SWORD_YPOS: usize = 0xc8;

const LINK_RECOIL_Z_VEL: usize = 0xc7;

const ROOM_TRANSITIONING_FLAGS: usize = 0xef;

const TILEMAP_LOCATION_CALC_MASK: usize = 0xec;

const DUNG_HDR_TAG: usize = 0xae;

const COMPOSITE_OF_LAYOUT_AND_QUADRANT: usize = 0xa8;

const DUNG_CUR_FLOOR: usize = 0xa4;

const DUNG_DRAW_HEIGHT_INDICATOR: usize = 0xb4;

const DUNG_DRAW_WIDTH_INDICATOR: usize = 0xb2;

const OVERLAY_INDEX: usize = 0x8c;

const OAM_EXT_CUR_PTR: usize = 0x92;

const FLAG_CUSTOM_SPELL_ANIM_ACTIVE: usize = 0x112;

const ALLOW_SCROLL_Z: usize = 0x78;

const INDEX_OF_INTERACTING_TILE: usize = 0x76;

const MOVING_AGAINST_DIAG_DEADLOCKED: usize = 0x6d;

const LINK_NUM_ORTHOGONAL_DIRECTIONS: usize = 0x6a;

const OVERWORLD_SCROLL_DELTA: usize = 0x69e;

const LINK_X_PAGE_MOVEMENT_DELTA: usize = 0x69;

const LINK_Y_PAGE_MOVEMENT_DELTA: usize = 0x68;

const FLAG_IS_LINK_IMMOBILIZED: usize = 0x2e4;

const GRAVESTONE_PUSH_TIMEOUT: usize = 0x61;

const LINK_SPEED_MODIFIER: usize = 0x57;

const TILEDETECT_WHICH_Y_POS: usize = 0x51;

const INDEX_OF_DASHING_SFX: usize = 0x4f;

const LINK_VISIBILITY_STATUS: usize = 0x4b;

const FORCE_MOVE_ANY_DIRECTION: usize = 0x49;

const SET_WHEN_DAMAGING_ENEMIES: usize = 0x47;

const LINK_DIRECTION_MASK_B: usize = 0x43;

const LINK_DIRECTION_MASK_A: usize = 0x42;

const BUTTON_B_FRAMES: usize = 0x3c;

const Y_BUTTON_ACTION_FLAGS: usize = 0x3b;

const BUTTON_MASK_B_Y: usize = 0x3a;

const LINK_Y_COORD_ORIGINAL: usize = 0x32;

const LINK_ANIMATION_STEPS: usize = 0x2e;

const LINK_SUBPIXEL_Z: usize = 0x2c;

const SCRATCH_1: usize = 0x74;

// SCRATCH_A is also defined (with a different value) in another module.
// const SCRATCH_A: usize = 0x73;

const SCRATCH_0: usize = 0x72;

const FLAG_UPDATE_HUD_IN_NMI: usize = 0x16;

const ATTRACT_NEXT_LEGEND_GFX: usize = 0x26;

const MAGIC_SPELL_PLAYER_LOCK_FLAG: usize = 0x0325;

const TRIGGER_SPECIAL_ENTRANCE_ANCILLA: usize = 0x04c6;

const TAGALONG_X_HI_ANCILLA: usize = 0x1a3c;

const TAGALONG_X_LO_ANCILLA: usize = 0x1a28;

const TAGALONG_Y_HI_ANCILLA: usize = 0x1a14;

const TAGALONG_Y_LO_ANCILLA: usize = 0x1a00;

const SWORDBEAM_TEMP_Y: usize = 0x15810;

const SWORDBEAM_TEMP_X: usize = 0x1580e;

const DOOR_DEBRIS_DIRECTION: usize = 0x03be;

const GARNISH_COUNTDOWN_ANCILLA: usize = 0x1f90e;

const GARNISH_SPRITE_ANCILLA: usize = 0x1f8b4;

const GARNISH_X_HI_ANCILLA: usize = 0x1f878;

const GARNISH_Y_HI_ANCILLA: usize = 0x1f85a;

const GARNISH_X_LO_ANCILLA: usize = 0x1f83c;

const GARNISH_Y_LO_ANCILLA: usize = 0x1f81e;

const GARNISH_ACTIVE_ANCILLA: usize = 0x0fb4;

const SPRITE_OAM_FLAGS_ANCILLA: usize = 0x0f50;

const SPRITE_Y_RECOIL_ANCILLA: usize = 0x0f30;

const SPRITE_HIT_TIMER_ANCILLA: usize = 0x0ef0;

const SPRITE_DELAY_AUX3_ANCILLA: usize = 0x0ee0;

const SPRITE_DELAY_AUX2_ANCILLA: usize = 0x0e10;

const SPRITE_G_ANCILLA: usize = 0x0ed0;

const SPRITE_F_ANCILLA: usize = 0x0ea0;

const SPRITE_HEAD_DIR_ANCILLA: usize = 0x0eb0;

const SPRITE_HEALTH_ANCILLA: usize = 0x0e50;

const SPRITE_BUMP_DAMAGE_ANCILLA: usize = 0x0cd2;

const SPRITE_C_ANCILLA: usize = 0x0db0;

const SPRITE_B_ANCILLA: usize = 0x0da0;

const DAMAGE_TYPE_DETERMINER_ANCILLA: usize = 0x0cf2;

const SPRITE_FLAGS_ANCILLA: usize = 0x0b6b;

const REPULSESPARK_ANIM_DELAY_ANCILLA: usize = 0x0faf;

const REPULSESPARK_Y_LO_ANCILLA: usize = 0x0fae;

const REPULSESPARK_X_LO_ANCILLA: usize = 0x0fad;

const REPULSESPARK_TIMER_ANCILLA: usize = 0x0fac;

const REPULSESPARK_FLOOR_STATUS_ANCILLA: usize = 0x0b68;

const SPRITE_IGNORE_PROJECTILE_ANCILLA: usize = 0x0ba0;

const INDEX_OF_INTERACTING_TILE_ANCILLA: usize = 0x0076;

const SCRATCH_1_ANCILLA: usize = 0x0074;

const SCRATCH_0_ANCILLA: usize = 0x0072;

const BOOMERANG_TEMP_X: usize = 0x039b;

const BOOMERANG_TEMP_Y: usize = 0x0399;

const CURRENT_AREA_OF_PLAYER_ANCILLA: usize = 0x0700;

const SPRITE_TILETYPE_ANCILLA: usize = 0x0fa5;

const ANCILLA_INTERACTIVE_RESET_FLAG: usize = 0x02f3;

const DUNG_FLAG_SOMARIA_BLOCK_SWITCH_PLAYER: usize = 0x0646;

const ANCILLA_R_PLAYER: usize = 0x03ea;

const ANCILLA_T_PLAYER: usize = 0x03d5;

const ANCILLA_S_PLAYER: usize = 0x03a9;

const ANCILLA_ALLOC_ROTATE_PLAYER: usize = 0x03c4;

const ANCILLA_TILE_ATTR_PLAYER: usize = 0x03e4;

const ANCILLA_Z_SUBPIXEL_PLAYER: usize = 0x02a8;

const DUNG_LOAD_PTR_BANK: usize = 0x00b9;

const DUNG_LOAD_PTR: usize = 0x00b7;

const BIG_KEY_DOOR_MESSAGE_TRIGGERED_DUNGEON: usize = 0x04b8;

const POTS_REVEALED_IN_ROOM_DUNGEON: usize = 0x0f580;

const DUNG_TORCH_TIMERS_DUNGEON: usize = 0x04f0;

const FLAG_WHICH_MUSIC_TYPE_DUNGEON: usize = 0x136;

const DOOR_DEBRIS_DIRECTION_DUNGEON: usize = 0x03be;

const COUNTDOWN_TIMER_FOR_STAIRCASES: usize = 0x378;

const STAIRCASE_LOWER_LEVEL_STATUS: usize = 0x492;

const DUNG_HDR_STAIRCASE_PLANE: usize = 0x63d;

const CUR_STAIRCASE_PLANE: usize = 0x48a;

const STAIRCASE_MOVE_COUNTER: usize = 0x464;

const WHICH_STAIRCASE_INDEX: usize = 0x462;

const DUNG_HDR_BG2_PROPERTIES_BACKUP: usize = 0xc208;

const SPRITE_Y_RECOIL_DUNGEON: usize = 0x0f30;

const MOVABLE_BLOCK_DATAS: usize = 0x0f940;

const PUSHEDBLOCK_FACING: usize = 0x05f8;

const PUSHEDBLOCKS_MAYBE_TIMEOUT: usize = 0x02c4;

const DUNG_INDEX_X3: usize = 0x110;

const DUNG_WIDTH_ROAD_ADDRESS: usize = 0x4b0;

const ADJACENT_DOORS: usize = 0x1110;

const ADJACENT_DOORS_FLAGS: usize = 0x1100;

const DUNG_TOGGLE_PALACE_POS: usize = 0x6d0;

const DUNG_TOGGLE_FLOOR_POS: usize = 0x6c0;

const DUNG_NUM_TOGGLE_PALACE: usize = 0x450;

const DUNG_NUM_TOGGLE_FLOOR: usize = 0x44e;

const STAIRCASE_TILEMAP_POS_X2: usize = 0x048c;

const DUNG_INTER_STAIRCASES: usize = 0x06b0;

// DUNG_FLAG_SOMARIA_BLOCK_SWITCH is also defined (with a different value) in another module.
// const DUNG_FLAG_SOMARIA_BLOCK_SWITCH: usize = 0x0646;


const MESSAGING_BUF_DUNGEON: usize = 0x10000;

const DUNG_REPLACEMENT_TILE_DST_POS_X2: usize = 0x04b6;

const BLOCK_TRAP_CHECK_FLAG: usize = 0x0466;

const DUNG_DOOR_BARRIER_OR_SWITCH_FLAG: usize = 0x045e;

const CRUSH_WALL_DOOR_INDEX_X2_DUNGEON: usize = 0x0456;

const CRUSH_WALL_PROGRESS_DUNGEON: usize = 0x0454;

// DUNG_CUR_QUADRANT_UPLOAD is also defined (with a different value) in another module.
// const DUNG_CUR_QUADRANT_UPLOAD: usize = 0x045c;

const DUNG_DOOR_SWITCH_TRIGGERED: usize = 0x0430;

const DUNG_WHICH_KEY_X2_DUNGEON: usize = 0x0694;

const DOOR_ANIMATION_STEP_INDICATOR_DUNGEON: usize = 0x0690;

const DUNG_CUR_DOOR_POS_DUNGEON: usize = 0x068e;

const DUNG_TRANSITION_LANDING_CLASS: usize = 0x004e;

const MINIGAME_CREDITS: usize = 0x04c4;

const TURN_ON_OFF_WATER_CTR: usize = 0x0424;

const WATER_HDMA_WINDOW_Y_RADIUS_ALT_DUNGEON: usize = 0x068a;

const WATER_HDMA_WINDOW_Y_TARGET_DUNGEON: usize = 0x0688;

const WATER_HDMA_WINDOW_X_RADIUS_DUNGEON: usize = 0x0686;

const WATER_HDMA_WINDOW_Y_RADIUS_DUNGEON: usize = 0x0684;

const WATER_HDMA_WINDOW_Y_DUNGEON: usize = 0x0682;

const WATER_HDMA_WINDOW_X_DUNGEON: usize = 0x0680;

const WATERGATE_SPOTLIGHT_Y_UPPER: usize = 0x0678;

const WATERGATE_POS: usize = 0x0472;

const WATERGATE_POINTER: usize = 0x0470;

const DUNG_FLAG_TRAPDOORS_DOWN: usize = 0x468;

const TRANSITION_COUNTER: usize = 0x0126;

const MOVING_WALL_ARR1: usize = 0xc880;

const MOVING_WALL_DOT_POINTER: usize = 0x41e;

const MOVING_WALL_WRITE_POINT: usize = 0x42a;

const DUNG_FLOOR_MOVE_FLAGS: usize = 0x41a;

const STAR_SHAPED_SWITCHES_TILE: usize = 0x6a0;

const DUNG_STAIRS_TABLE_2: usize = 0x6ec;

const DUNG_NUM_WATER_LADDERS: usize = 0x446;

const DUNG_NUM_INROOM_SOUTHDOWN_STAIRS: usize = 0x43e;

const DUNG_NUM_INROOM_UPNORTH_STAIRS: usize = 0x43c;

const DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS: usize = 0x438;

const DUNG_NUM_STAR_SHAPED_SWITCHES: usize = 0x432;

const DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER: usize = 0x4ae;

const KIND_OF_IN_ROOM_STAIRCASE_DUNGEON: usize = 0x44a;

const WATER_SIDE_STEP_SWITCH: usize = 0x448;

const DUNG_NUM_ACTIVATED_WATER_LADDERS: usize = 0x444;

const DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER: usize = 0x442;

const DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS: usize = 0x440;

const DUNG_NUM_STAIRS_WET: usize = 0x49e;

const DUNG_NUM_STAIRS_2: usize = 0x49c;

const DUNG_NUM_STAIRS_1: usize = 0x49a;

const DUNG_REPLACEMENT_TILE_SRC_POS_X2: usize = 0x47c;

// OVERWORLD_SCROLL_RIGHT_COUNTER is also defined (with a different value) in another module.
// const OVERWORLD_SCROLL_RIGHT_COUNTER: usize = 0x62a;

const OVERWORLD_SCROLL_LEFT_COUNTER: usize = 0x628;

// OVERWORLD_SCROLL_DOWN_COUNTER is also defined (with a different value) in another module.
// const OVERWORLD_SCROLL_DOWN_COUNTER: usize = 0x626;

const OVERWORLD_SCROLL_UP_COUNTER: usize = 0x624;

// DUNG_CUR_QUADRANT_UPLOAD is also defined (with a different value) in another module.
// const DUNG_CUR_QUADRANT_UPLOAD: usize = 0x045c;

const DIALOGUE_NUMBER: usize = 0x1cf2;

const GARNISH_COUNTDOWN: usize = 0x1f90e;

const GARNISH_X_HI: usize = 0x1f878;

const GARNISH_Y_HI: usize = 0x1f85a;

const GARNISH_X_LO: usize = 0x1f83c;

const GARNISH_Y_LO: usize = 0x1f81e;

// DUNG_FLOOR_MOVE_FLAGS is also defined (with a different value) in another module.
// const DUNG_FLOOR_MOVE_FLAGS: usize = 0x041a;

const ACTIVE_OVERLORD_INDEX: usize = 0x0fde;

const SPRCOLL_Y_BASE: usize = 0x0fbe;

const SPRITE_TILETYPE: usize = 0x0fa5;

const GARNISH_ACTIVE: usize = 0x0fb4;

const SPRITE_STATE: usize = 0x0dd0;

const SPRITE_AI_STATE: usize = 0x0d80;

const ACTIVATE_BOMB_TRAP_OVERLORD: usize = 0x0cf4;

const OVERLORD_OFFSET_SPRITE_POS: usize = 0x0b48;

const OVERLORD_FLOOR: usize = 0x0b40;

const OVERLORD_GEN3: usize = 0x0b38;

const OVERLORD_GEN2: usize = 0x0b30;

const OVERLORD_GEN1: usize = 0x0b28;

const OVERLORD_Y_HI: usize = 0x0b20;

const OVERLORD_Y_LO: usize = 0x0b18;

const OVERLORD_X_HI: usize = 0x0b10;

const CURRENT_AREA_OF_PLAYER_OVERWORLD: usize = 0x0700;

const TRANSITION_COUNTER_OVERWORLD: usize = 0x0126;

const OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV_OVERWORLD: usize = 0x0c21f;

const OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV_OVERWORLD: usize = 0x0c21d;

const OVERWORLD_SCREEN_TRANSITION_PREV_OVERWORLD: usize = 0x0c21b;

const MAP16_LOAD_DST_OFF_PREV_OVERWORLD: usize = 0x0c219;

const MAP16_LOAD_Y_UNIT_PREV_OVERWORLD: usize = 0x0c217;

const MAP16_LOAD_SRC_OFF_PREV_OVERWORLD: usize = 0x0c215;

const OVERWORLD_SCREEN_INDEX_PREV_OVERWORLD: usize = 0x0c213;

const OVERLAY_INDEX_OVERWORLD: usize = 0x008c;

const SAVEGAME_HAS_MASTER_SWORD_FLAGS_OVERWORLD: usize = 0x0f300;

const MOVE_OVERLAY_CTR_OVERWORLD: usize = 0x0494;

const BIRDTRAVEL_STATUS_OVERWORLD: usize = 0x1af0;

const FLAG_OVERWORLD_AREA_DID_CHANGE_OVERWORLD: usize = 0x0abf;

const MISC_SPRITES_GRAPHICS_INDEX_OVERWORLD: usize = 0x0aa4;

const SPRITE_GRAPHICS_INDEX_OVERWORLD: usize = 0x0aa3;

const AUX_TILE_THEME_INDEX_OVERWORLD: usize = 0x0aa2;

const MAIN_TILE_THEME_INDEX_OVERWORLD: usize = 0x0aa1;

const OVERWORLD_TILE_THEME_INDEX_OVERWORLD: usize = 0x0aa0;

const OVERWORLD_SCROLL_RIGHT_COUNTER_EXIT_OVERWORLD: usize = 0x0c170;

const OVERWORLD_SCROLL_LEFT_COUNTER_EXIT_OVERWORLD: usize = 0x0c16e;

const OVERWORLD_SCROLL_DOWN_COUNTER_EXIT_OVERWORLD: usize = 0x0c16c;

const OVERWORLD_SCROLL_UP_COUNTER_EXIT_OVERWORLD: usize = 0x0c16a;

const SPRITE_GRAPHICS_INDEX_EXIT_OVERWORLD: usize = 0x0c167;

const AUX_TILE_THEME_INDEX_EXIT_OVERWORLD: usize = 0x0c166;

const MAIN_TILE_THEME_INDEX_EXIT_OVERWORLD: usize = 0x0c165;

const OVERWORLD_EXIT_TILE_THEME_INDEX_OVERWORLD: usize = 0x0c164;

const LEFT_RIGHT_SCROLL_TARGET_END_EXIT_OVERWORLD: usize = 0x0c162;

const LEFT_RIGHT_SCROLL_TARGET_EXIT_OVERWORLD: usize = 0x0c160;

const UP_DOWN_SCROLL_TARGET_END_EXIT_OVERWORLD: usize = 0x0c15e;

const UP_DOWN_SCROLL_TARGET_EXIT_OVERWORLD: usize = 0x0c15c;

const OW_SCROLL_VARS0_EXIT_OVERWORLD: usize = 0x0c154;

const CAMERA_X_COORD_SCROLL_LOW_EXIT_OVERWORLD: usize = 0x0c152;

const CAMERA_Y_COORD_SCROLL_LOW_EXIT_OVERWORLD: usize = 0x0c150;

const MAP16_LOAD_SRC_OFF_EXIT_OVERWORLD: usize = 0x0c14e;

const OVERWORLD_SCREEN_INDEX_EXIT_OVERWORLD: usize = 0x0c14c;

const TM_COPY_EXIT_OVERWORLD: usize = 0x0c142;

const OVERWORLD_AREA_INDEX_EXIT_OVERWORLD: usize = 0x0c140;

const OVERWORLD_SCROLL_RIGHT_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c130;

const OVERWORLD_SCROLL_LEFT_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c12e;

const OVERWORLD_SCROLL_DOWN_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c12c;

const OVERWORLD_SCROLL_UP_COUNTER_SPEXIT_OVERWORLD: usize = 0x0c12a;

const SPRITE_GRAPHICS_INDEX_SPEXIT_OVERWORLD: usize = 0x0c127;

const AUX_TILE_THEME_INDEX_SPEXIT_OVERWORLD: usize = 0x0c126;

const MAIN_TILE_THEME_INDEX_SPEXIT_OVERWORLD: usize = 0x0c125;

const OVERWORLD_SPECIAL_TILE_THEME_INDEX: usize = 0x0c124;

const LEFT_RIGHT_SCROLL_TARGET_END_SPEXIT_OVERWORLD: usize = 0x0c122;

const LEFT_RIGHT_SCROLL_TARGET_SPEXIT_OVERWORLD: usize = 0x0c120;

const UP_DOWN_SCROLL_TARGET_END_SPEXIT_OVERWORLD: usize = 0x0c11e;

const UP_DOWN_SCROLL_TARGET_SPEXIT_OVERWORLD: usize = 0x0c11c;

const SPECIAL_EXIT_ROOM_BOUNDS_X_END: usize = 0x0c11a;

const SPECIAL_EXIT_ROOM_BOUNDS_X_START: usize = 0x0c118;

const SPECIAL_EXIT_ROOM_BOUNDS_Y_END: usize = 0x0c116;

const SPECIAL_EXIT_ROOM_BOUNDS_Y_START: usize = 0x0c114;

const CAMERA_X_COORD_SCROLL_LOW_SPEXIT_OVERWORLD: usize = 0x0c112;

const CAMERA_Y_COORD_SCROLL_LOW_SPEXIT_OVERWORLD: usize = 0x0c110;

const MAP16_LOAD_SRC_OFF_SPEXIT_OVERWORLD: usize = 0x0c10e;

const OVERWORLD_SCREEN_INDEX_SPEXIT_OVERWORLD: usize = 0x0c10c;

const TM_COPY_SPEXIT_OVERWORLD: usize = 0x0c102;

const OVERWORLD_AREA_INDEX_SPEXIT_OVERWORLD: usize = 0x0c100;

const OVERWORLD_OFFSET_MASK_X_OVERWORLD: usize = 0x070e;

const OVERWORLD_OFFSET_BASE_X_OVERWORLD: usize = 0x070c;

const OVERWORLD_OFFSET_MASK_Y_OVERWORLD: usize = 0x070a;

const OVERWORLD_OFFSET_BASE_Y_OVERWORLD: usize = 0x0708;

const OW_COUNTDOWN_TRANSITION_OVERWORLD: usize = 0x069a;

const OVERWORLD_SCROLL_RIGHT_COUNTER_OVERWORLD: usize = 0x062a;

const OVERWORLD_SCROLL_LEFT_COUNTER_OVERWORLD: usize = 0x0628;

const OVERWORLD_SCROLL_DOWN_COUNTER_OVERWORLD: usize = 0x0626;

const OVERWORLD_SCROLL_UP_COUNTER_OVERWORLD: usize = 0x0624;

const OVERWORLD_SCREEN_TRANS_DIR_BITS_OVERWORLD: usize = 0x0410;

const OVERWORLD_AREA_INDEX_OVERWORLD: usize = 0x040a;

const SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT: usize = 0x0c176;

const SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF: usize = 0x0c174;

const ORANGE_BLUE_BARRIER_STATE_OVERWORLD: usize = 0x0c172;

const DUNG_REPLACEMENT_TILE_STATE_OVERWORLD: usize = 0x0500;

const MAP16_DECODE_WORK_WORD_OVERWORLD: usize = 0x14442;

const MAP16_DECODE_LAST_OVERWORLD: usize = 0x14440;

const OVERWORLD_MAP16_DECODE_SRC: usize = 0x14000;

const MAP16_LOAD_Y_UNIT_OVERWORLD: usize = 0x0088;

const MAP16_LOAD_DST_OFF_OVERWORLD: usize = 0x0086;

const MAP16_LOAD_SRC_OFF_OVERWORLD: usize = 0x0084;

const MAPBAK_PALETTE_OVERWORLD: usize = 0x1dd80;

const OVERWORLD_BOMB_TILE_SWEEP_Y_END: usize = 0x0488;

const OVERWORLD_BOMB_TILE_SWEEP_X: usize = 0x0486;

const TRIGGER_SPECIAL_ENTRANCE_OVERWORLD: usize = 0x04c6;

const BIG_KEY_DOOR_MESSAGE_TRIGGERED_OVERWORLD: usize = 0x04b8;

const OVERWORLD_PEG_PUZZLE_PROGRESS: usize = 0x04c8;

const OVERWORLD_TRANSITION_DIR_ENUM: usize = 0x069c;

const DOOR_ANIMATION_STEP_INDICATOR_OVERWORLD: usize = 0x0690;

const SPRITE_C_PLAYER: usize = 0x0db0;

const PUSH_BLOCK_DIRECTION_PLAYER: usize = 0x0474;

const DOOR_ANIMATION_STEP_INDICATOR_PLAYER: usize = 0x0690;

const PRIMARY_WATER_GRASS_TIMER: usize = 0x356;

const SECONDARY_WATER_GRASS_TIMER: usize = 0x355;

const TURTLE_ROCK_OAM_PRIORITY_FLAG: usize = 0x34e;

const ALT_SPRITE_IGNORE_PROJECTILE_SPRITE: usize = 0x1fadc;

const ALT_SPRITE_I_SPRITE: usize = 0x1facc;

const ALT_SPRITE_DELAY_MAIN_SPRITE: usize = 0x1faac;

const ALT_SPRITE_HEIGHT_ABOVE_SHADOW_SPRITE: usize = 0x1fa9c;

const ALT_SPRITE_SUBTYPE2_SPRITE: usize = 0x1fa8c;

const ALT_SPRITE_E_SPRITE: usize = 0x1fa7c;

const ALT_SPRITE_C_SPRITE: usize = 0x1fa6c;

const ALT_SPRITE_B_SPRITE: usize = 0x1fa5c;

const ALT_SPRITE_FLAGS3_SPRITE: usize = 0x1df0;

const ALT_SPRITE_SPAWNED_FLAG_SPRITE: usize = 0x1de0;

const ALT_SPRITE_FLOOR_SPRITE: usize = 0x1dd0;

const ALT_SPRITE_FLAGS2_SPRITE: usize = 0x1dc0;

const ALT_SPRITE_D_SPRITE: usize = 0x1db0;

const ALT_SPRITE_OBJ_PRIO_SPRITE: usize = 0x1da0;

const ALT_SPRITE_OAM_FLAGS_SPRITE: usize = 0x1d90;

const ALT_SPRITE_HEAD_DIR_SPRITE: usize = 0x1d80;

const ALT_SPRITE_A_SPRITE: usize = 0x1d70;

const ALT_SPRITE_GRAPHICS_SPRITE: usize = 0x1d60;

const ALT_SPRITE_Y_HI_SPRITE: usize = 0x1d50;

const ALT_SPRITE_Y_LO_SPRITE: usize = 0x1d40;

const ALT_SPRITE_X_HI_SPRITE: usize = 0x1d30;

const ALT_SPRITE_X_LO_SPRITE: usize = 0x1d20;

const ALT_SPRITE_TYPE_SPRITE: usize = 0x1d10;

const ALT_SPRITE_STATE_SPRITE: usize = 0x1d00;

const NUM_SPRITES_KILLED_SPRITE: usize = 0x0cfb;

const LUCK_KILL_COUNTER_SPRITE: usize = 0x0cfa;

const ITEM_DROP_LUCK_SPRITE: usize = 0x0cf9;

const SPRITE_TILETYPE_SPR: usize = 0x0fa5;

const BYTE_7FFABC: usize = 0x1fabc;

const SPRITE_N_WORD: usize = 0x0bc0;

const SPRITE_OVERLORD_Y_HI: usize = 0x0b20;

const SPRITE_OVERLORD_Y_LO: usize = 0x0b18;

const SPRITE_OVERLORD_X_HI: usize = 0x0b10;

const OVERLORD_OFFSET_SPRITE_POS_SPRITE: usize = 0x0b48;

const SPRITE_WHERE_IN_OVERWORLD: usize = 0x1df80;

const SPRCOLL_Y_BASE_SPRITE: usize = 0x0fbe;

const SPRCOLL_X_BASE_SPRITE: usize = 0x0fbc;

const SPR_RANGED_BASED_TOGGLER: usize = 0x0fb7;

const OAM_REGION_BASE_SPRITE: usize = 0x0fe0;

const ACTIVATE_BOMB_TRAP_OVERLORD_SPRITE: usize = 0x0cf4;

const SPRITE_RESET_WORK_B: usize = 0x0ffb;

const SPRITE_RESET_WORK_A: usize = 0x0ff8;

const SRAM_PROGRESS_INDICATOR_SPRITE: usize = 0x0f3c5;

const OVERWORLD_AREA_INDEX_SPRITE: usize = 0x040a;

const GARNISH_OAM_FLAGS_SPRITE: usize = 0x1f9fe;

const GARNISH_X_SUBPIXEL_SPRITE: usize = 0x1f8f0;

const GARNISH_Y_SUBPIXEL_SPRITE: usize = 0x1f8d2;

const GARNISH_X_VEL_SPRITE: usize = 0x1f8b4;

const GARNISH_Y_VEL_SPRITE: usize = 0x1f896;

const IS_IN_DARK_WORLD_SPRITE: usize = 0x0fff;

const SPRITE_GIVE_DAMAGE_SPRITE: usize = 0x0ce2;

const DAMAGE_TYPE_DETERMINER_SPRITE: usize = 0x0cf2;

const SPRITE_FLAGS_SPRITE: usize = 0x0b6b;

const ANCILLA_Y_LO_SPRITE: usize = 0x0bfa;

const ANCILLA_X_LO_SPRITE: usize = 0x0c04;

const SPRITE_AI_STATE_SPRITE: usize = 0x0d80;

const SPRITE_PICKUP_SLOT_CACHE: usize = 0x0fb2;

const SPRITE_DRAW_PRIORITY_OVERRIDE: usize = 0x0cfe;

const SPRITE_Y_RECOIL: usize = 0x0f30;

const SPRITE_DELAY_AUX2_SPRITE: usize = 0x0e10;

const SPRITE_DELAY_AUX3_SPRITE: usize = 0x0ee0;

const SPRITE_HEALTH_SPRITE: usize = 0x0e50;

const SPRITE_DELAY_AUX1_SPRITE: usize = 0x0e00;

const SPRITE_C_SPRITE: usize = 0x0db0;

const LINK_ITEM_QUAKE_MEDALLION_DRAW: usize = 0x0f349;

const LINK_ITEM_BOMBOS_MEDALLION_DRAW: usize = 0x0f347;

const ENHANCED_FEATURES0_DRAW: usize = 0x064c;

const FLAG_OVERWORLD_AREA_DID_CHANGE_DRAW: usize = 0x0abf;

const MINIGAME_CREDITS_DRAW: usize = 0x04c4;

const ANCILLA_Z_DRAW: usize = 0x29e;

const ANCILLA_Y_VEL_DRAW: usize = 0x0c22;

const ANCILLA_X_VEL_DRAW: usize = 0x0c2c;

const ANCILLA_X_HI_DRAW: usize = 0x0c18;

const ANCILLA_Y_HI_DRAW: usize = 0x0c0e;

const ANCILLA_X_LO_DRAW: usize = 0x0c04;

const ANCILLA_Y_LO_DRAW: usize = 0x0bfa;

const SPRITE_I_DRAW: usize = 0x1f9c2;

const ACTIVATE_BOMB_TRAP_OVERLORD_DRAW: usize = 0x0cf4;

const GARNISH_OAM_FLAGS_DRAW: usize = 0x1f9fe;

const GARNISH_FLOOR_DRAW: usize = 0x1f968;

const GARNISH_SPRITE_DRAW: usize = 0x1f92c;

const GARNISH_COUNTDOWN_DRAW: usize = 0x1f90e;

const GARNISH_X_HI_DRAW: usize = 0x1f878;

const GARNISH_Y_HI_DRAW: usize = 0x1f85a;

const GARNISH_X_LO_DRAW: usize = 0x1f83c;

const GARNISH_Y_LO_DRAW: usize = 0x1f81e;

const GARNISH_TYPE_DRAW: usize = 0x1f800;

const SRAM_PROGRESS_INDICATOR_3_DRAW: usize = 0x0f3c9;

const SPRITE_DRAW_WORK_Y_OR_FLAGS: usize = 0x0fb6;

const GARNISH_ACTIVE_DRAW: usize = 0x0fb4;

const ACTIVE_OVERLORD_INDEX_DRAW: usize = 0x0fde;

const SPRITE_TILETYPE_DRAW: usize = 0x0fa5;

const REPULSESPARK_Y_LO_DRAW: usize = 0x0fae;

const REPULSESPARK_X_LO_DRAW: usize = 0x0fad;

const REPULSESPARK_TIMER_DRAW: usize = 0x0fac;

const IS_IN_DARK_WORLD_DRAW: usize = 0x0fff;

const SPRITE_Y_RECOIL_DRAW: usize = 0x0f30;

const SPRITE_DELAY_AUX3_DRAW: usize = 0x0ee0;

const SPRITE_G_DRAW: usize = 0x0ed0;

const SPRITE_ANIM_CLOCK_DRAW: usize = 0x0ec0;

const SPRITE_F_DRAW: usize = 0x0ea0;

const SPRITE_DELAY_AUX2_DRAW: usize = 0x0e10;

const SPRITE_DELAY_AUX1_DRAW: usize = 0x0e00;

const SPRITE_C_DRAW: usize = 0x0db0;

const SPRITE_B_DRAW: usize = 0x0da0;

const SPRITE_IGNORE_PROJECTILE_DRAW: usize = 0x0ba0;

const BYTE_7FFE01: usize = 0x1fe01;

const FLAG_UPDATE_HUD_NEXT_FRAME: usize = 0xf2;

const PLAYER_HANDLER_STATE_DN: usize = 0x5d;

const TILE_ACTION_INDEX_DN: usize = 0x36c;

const MESSAGING_MODULE: usize = 0x0e2;

const TILE_INTERACTION_SHARED_FLAG: usize = 0x0223;

const TRIGGER_SPECIAL_ENTRANCE: usize = 0x4c6;

const LINK_DISABLE_SPRITE_DAMAGE_DN: usize = 0x37b;

const FLAG_OVERWORLD_AREA_DID_CHANGE: usize = 0xabf;

const SRAM_PROGRESS_INDICATOR_AUX: usize = 0x0f3c9; // alias used by Smithy_Homecoming

const GARNISH_COUNTDOWN_GANON: usize = 0x1f90e;

const SPRITE_HIT_TIMER_GANON: usize = 0x0ef0;

const SPRITE_G_GANON: usize = 0x0ed0;

const SPRITE_ANIM_CLOCK_GANON: usize = 0x0ec0;

const SPRITE_HEALTH_GANON: usize = 0x0e50;

const SPRITE_DELAY_AUX2_GANON: usize = 0x0e10;

const SPRITE_DELAY_AUX1_GANON: usize = 0x0e00;

const SPRITE_C_GANON: usize = 0x0db0;

const SPRITE_B_GANON: usize = 0x0da0;

const SPRITE_BUMP_DAMAGE_GANON: usize = 0x0cd2;

const SPRITE_IGNORE_PROJECTILE_GANON: usize = 0x0ba0;

const SPRITE_OBJ_PRIO_GANON: usize = 0x0b89;

const OVERLORD_FLOOR_GANON: usize = 0x0b40;

const DUNG_TORCH_DATA_GANON: usize = 0x0fb40;

const DUNG_TORCH_TIMERS_GANON: usize = 0x04f0;

const SPRITE_Y_RECOIL_GUARD: usize = 0x0f30;

const SPRITE_DELAY_AUX3_GUARD: usize = 0x0ee0;

const SPRITE_TILETYPE_GUARD: usize = 0x0fa5;

const OVERWORLD_AREA_INDEX_GUARD: usize = 0x40a;

const ARMOS_KNIGHT_REMAINING_COUNT: usize = 0x0ff8;

#[cfg(test)]
const OVERLORD_GEN3_PREP: usize = 0x0b38;

#[cfg(test)]
const OVERLORD_GEN1_PREP: usize = 0x0b28;

#[cfg(test)]
const OVERLORD_Y_LO_PREP: usize = 0x0b18;

#[cfg(test)]
const BEAMOS_Y_HI_PREP: usize = 0x1ff00;

#[cfg(test)]
const BEAMOS_Y_LO_PREP: usize = 0x1fe80;

#[cfg(test)]
const BEAMOS_X_LO_PREP: usize = 0x1fd80;

const SPRCOLL_Y_BASE_PREP: usize = 0x0fbe;

const SPRCOLL_X_BASE_PREP: usize = 0x0fbc;

const SRAM_PROGRESS_INDICATOR_3_PREP: usize = 0x0f3c9;

const FLAG_OVERWORLD_AREA_DID_CHANGE_PREP: usize = 0x0abf;

const SPRITE_DELAY_AUX3_PREP: usize = 0x0ee0;

const LUCK_KILL_COUNTER_PREP: usize = 0x0cfa;

const ITEM_DROP_LUCK_PREP: usize = 0x0cf9;

const LINK_RUPEES_IN_POND_PREP: usize = 0x0f36a;

const SPRITE_PREP_SHARED_COUNTER: usize = 0x0ff8;

const ACTIVE_OVERLORD_INDEX_PREP: usize = 0x0fde;

const DUNG_FLOOR_MOVE_FLAGS_PREP: usize = 0x041a;

const IS_IN_DARK_WORLD_PREP: usize = 0x0fff;

const OVERLORD_X_HI_SB: usize = 0x0b10;

const VITREOUS_EYEBALL_RELEASE_COUNT: usize = 0x0ff8;

const SPRITE_DELAY_AUX3_SB: usize = 0x0ee0;

const ALT_SPRITE_SPAWNED_FLAG_WORLD: usize = 0x1de0;

const SRAM_PROGRESS_INDICATOR_3: usize = 0x0f3c9;

const ENHANCED_FEATURES0_TAGALONG: usize = 0x064c;

const SAVED_TAGALONG_FLOOR_TAGALONG: usize = 0x0f3d2;

const SAVED_TAGALONG_INDOORS_TAGALONG: usize = 0x0f3d1;

const SAVED_TAGALONG_X_TAGALONG: usize = 0x0f3cf;

const SAVED_TAGALONG_Y_TAGALONG: usize = 0x0f3cd;

const SAVE_DUNG_INFO_TAGALONG: usize = 0x0f000;

const ANCILLA_R_TAGALONG: usize = 0x03ea;

const ANCILLA_L_TAGALONG: usize = 0x385;

const ANCILLA_ITEM_TO_LINK_TAGALONG: usize = 0x0c5e;

const ANCILLA_STEP_TAGALONG: usize = 0x0c54;

const ANCILLA_X_SUBPIXEL_TAGALONG: usize = 0x0c40;

const ANCILLA_Y_SUBPIXEL_TAGALONG: usize = 0x0c36;

const ANCILLA_X_VEL_TAGALONG: usize = 0x0c2c;

const ANCILLA_Y_VEL_TAGALONG: usize = 0x0c22;

const ANCILLA_X_HI_TAGALONG: usize = 0x0c18;

const ANCILLA_Y_HI_TAGALONG: usize = 0x0c0e;

const ANCILLA_X_LO_TAGALONG: usize = 0x0c04;

const ANCILLA_Y_LO_TAGALONG: usize = 0x0bfa;

const SPRITE_SUBTYPE_TAGALONG: usize = 0x0e30;

const SPRITE_DIE_ACTION_TAGALONG: usize = 0x0cba;

const SPRITE_N_TAGALONG: usize = 0x0bc0;

const SPRITE_SUBTYPE2_TAGALONG: usize = 0x0e80;

const SPRITE_Z_VEL_TAGALONG: usize = 0x0f80;

const SPRITE_Z_TAGALONG: usize = 0x0f70;

const SPRITE_FLOOR_TAGALONG: usize = 0x0f20;

const SPRITE_GRAPHICS_TAGALONG: usize = 0x0dc0;

const SPRITE_DELAY_AUX2_TAGALONG: usize = 0x0e10;

const SPRITE_D_TAGALONG: usize = 0x0de0;

const SPRITE_HEAD_DIR_TAGALONG: usize = 0x0eb0;

const SPRITE_AI_STATE_TAGALONG: usize = 0x0d80;

const SPRITE_X_VEL_TAGALONG: usize = 0x0d50;

const SPRITE_Y_VEL_TAGALONG: usize = 0x0d40;

const SPRITE_X_HI_TAGALONG: usize = 0x0d30;

const SPRITE_Y_HI_TAGALONG: usize = 0x0d20;

const SPRITE_X_LO_TAGALONG: usize = 0x0d10;

const SPRITE_Y_LO_TAGALONG: usize = 0x0d00;

const SPRITE_TYPE_TAGALONG: usize = 0x0e20;

const SPRITE_STATE_TAGALONG: usize = 0x0dd0;

const SPRITE_IGNORE_PROJECTILE_TAGALONG: usize = 0x0ba0;

const KIKI_ANIM_COUNTER_TAGALONG: usize = 0x0b69;

const DOOR_ANIMATION_STEP_INDICATOR_TAGALONG: usize = 0x690;

const DUNG_CUR_DOOR_POS_TAGALONG: usize = 0x68e;

const DUNG_FLAG_TRAPDOORS_DOWN_TAGALONG: usize = 0x468;

const SUPER_BOMB_INDICATOR_COUNTER_TAGALONG: usize = 0x4b5;

const SUPER_BOMB_INDICATOR_TIMER_TAGALONG: usize = 0x4b4;

const PALETTE_SWAP_FLAG_TAGALONG: usize = 0x0abd;

const MESSAGING_MODULE_TAGALONG: usize = 0x1cd8;

const TAGALONG_MESSAGE_RESET_FLAG: usize = 0x223;

const DIALOGUE_MESSAGE_INDEX_TAGALONG: usize = 0x1cf0;

const TAGALONG_MIRROR_BGM_COMMAND: usize = 0x12c;

const OAM_EXT_CUR_PTR_TAGALONG: usize = 0x92;

const OAM_CUR_PTR_TAGALONG: usize = 0x90;

const OAM_PRIORITY_VALUE_TAGALONG: usize = 0x64;

const COUNTDOWN_FOR_BLINK_TAGALONG: usize = 0x31f;

const TAGALONG_DRAW_ANIM_FRAME: usize = 0x2d7;

const TAGALONG_JUMP_TIMER_TAGALONG: usize = 0x2d6;

const TAGALONG_MESSAGE_TIMER: usize = 0x2cd;

const FILTERED_JOYPAD_L_TAGALONG: usize = 0xf6;
