use crate::game_state::constants::{
    ACTIVATE_BOMB_TRAP_OVERLORD, ADJACENT_DOORS, ADJACENT_DOORS_FLAGS, ATTRIBUTES_FOR_TILE_PLAYER,
    AUX_TILE_THEME_INDEX, BIG_ROCK_STARTING_ADDRESS, BLOCK_TRAP_CHECK_FLAG,
    CHANGEABLE_DUNGEON_OBJECT_INDEX, COMPOSITE_OF_LAYOUT_AND_QUADRANT, CRUSH_WALL_DOOR_INDEX_X2,
    CRUSH_WALL_PROGRESS, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, DOOR_OPEN_CLOSED_COUNTER,
    DOOR_TYPE_AND_SLOT, DUNGEON_BG1_ATTR_TABLE, DUNGEON_BG2_ATTR_TABLE, DUNGEON_DOOR_DIRECTION,
    DUNGEON_FLOOR_X_VELOCITY, DUNGEON_FLOOR_Y_VELOCITY, DUNGEON_HEADER_COLLISION_2_MIRROR,
    DUNGEON_HEADER_HOLE_TELEPORTER_PLANE, DUNGEON_HEADER_STAIRCASE_PLANE, DUNGEON_HEADER_TAG,
    DUNGEON_HEADER_TRAVEL_DESTINATIONS, DUNGEON_REPLACEMENT_TILE_STATE, DUNGEON_TORCH_ATTR,
    DUNGEON_TORCH_DATA, DUNGEON_TRAP_TRIGGER_LATCH, DUNGEON_WORK_R16, DUNGEON_WORK_R18, DUNG_BG1,
    DUNG_BG2, DUNG_BLASTWALL_FLAG_X, DUNG_BLASTWALL_FLAG_Y, DUNG_CHEST_LOCATIONS,
    DUNG_CUR_DOOR_IDX, DUNG_CUR_DOOR_POS_DUNGEON, DUNG_CUR_FLOOR, DUNG_CUR_FLOOR_CACHED,
    DUNG_CUR_QUADRANT_UPLOAD, DUNG_DOOR_BARRIER_OR_SWITCH_FLAG, DUNG_DOOR_OPENED,
    DUNG_DOOR_OPENED_INCL_ADJACENT, DUNG_DOOR_SWITCH_TRIGGERED, DUNG_DOOR_TILEMAP_ADDRESS,
    DUNG_DRAW_HEIGHT_INDICATOR, DUNG_DRAW_WIDTH_INDICATOR, DUNG_EXIT_DOOR_ADDRESSES,
    DUNG_EXIT_DOOR_COUNT, DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED, DUNG_FLAG_SOMARIA_BLOCK_SWITCH,
    DUNG_FLAG_STATECHANGE_WATERPUZZLE, DUNG_FLAG_TRAPDOORS_DOWN, DUNG_FLOOR_MOVE_FLAGS,
    DUNG_FLOOR_X_OFFS, DUNG_FLOOR_Y_OFFS, DUNG_HDR_BG2_PROPERTIES, DUNG_HDR_BG2_PROPERTIES_BACKUP,
    DUNG_HDR_COLLISION, DUNG_HDR_COLLISION_2, DUNG_INDEX_OF_TORCHES, DUNG_INDEX_OF_TORCHES_START,
    DUNG_INDEX_X3, DUNG_INTER_STAIRCASES, DUNG_LAYOUT_AND_STARTING_QUADRANT, DUNG_LINE_PTRS_ROW0,
    DUNG_LOADE_BGOFFS_H_COPY, DUNG_LOADE_BGOFFS_V_COPY, DUNG_LOAD_PTR, DUNG_LOAD_PTR_BANK,
    DUNG_LOAD_PTR_OFFS, DUNG_MISC_OBJS_INDEX, DUNG_NUM_ACTIVATED_WATER_LADDERS,
    DUNG_NUM_BIGKEY_LOCKS_X2, DUNG_NUM_CHESTS_X2, DUNG_NUM_INROOM_UPNORTH_STAIRS,
    DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER, DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER,
    DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS, DUNG_NUM_LIT_TORCHES, DUNG_NUM_STAIRS_1,
    DUNG_NUM_STAIRS_2, DUNG_NUM_STAIRS_WET, DUNG_NUM_TOGGLE_FLOOR, DUNG_NUM_TOGGLE_PALACE,
    DUNG_OBJECT_POS_IN_OBJDATA, DUNG_OBJECT_TILEMAP_POS, DUNG_OVERLAY_TO_LOAD,
    DUNG_QUADRANTS_VISITED, DUNG_REPLACEMENT_TILE_DST_POS_X2, DUNG_REPLACEMENT_TILE_SRC_POS_X2,
    DUNG_SAVEGAME_STATE_BITS, DUNG_TOGGLE_FLOOR_POS, DUNG_TOGGLE_PALACE_POS,
    DUNG_TRANSITION_LANDING_CLASS, DUNG_WANT_LIGHTS_OUT, DUNG_WANT_LIGHTS_OUT_COPY,
    DUNG_WHICH_KEY_X2_DUNGEON, DUNG_WIDTH_ROAD_ADDRESS, FLAG_SKIP_CALL_TAG_ROUTINES,
    FLAG_WHICH_MUSIC_TYPE_DUNGEON, FLOOR_1_FILLER_TILES, FLOOR_2_FILLER_TILES, GANON_TORCH_COUNT,
    HDR_DUNGEON_DARK_WITH_LANTERN, INVISIBLE_DOOR_DIR_AND_INDEX_X2, MAIN_TILE_THEME_INDEX,
    MESSAGING_BUF_DUNGEON, MOVABLE_BLOCK_DATAS, MOVING_FLOOR_BG_CHECK_FLAGS,
    MOVING_WALL_DOT_POINTER, MOVING_WALL_REPLACEMENT_BUFFER, MOVING_WALL_TORCH_BLINK_PHASE,
    MOVING_WALL_TORCH_UPDATE_FLAG, MOVING_WALL_WRITE_POINT, ORANGE_BLUE_BARRIER_STATE,
    OVERLAY_INDEX, OVERWORLD_EXIT_TILE_THEME_INDEX, OVERWORLD_FIXED_COLOR_PLUSMINUS,
    OVERWORLD_MAP_STATE, OVERWORLD_SCREEN_INDEX, OVERWORLD_TILE_THEME_INDEX,
    REPLACEMENT_TILEMAP_LL, REPLACEMENT_TILEMAP_LR, REPLACEMENT_TILEMAP_UL, REPLACEMENT_TILEMAP_UR,
    RESERVED_GFX_CONFIG_WORD, RESET_XY_CHECK_FLAGS, SOMARIA_BLOCK_BG_CHECK_FLAG,
    SPRITE_GRAPHICS_INDEX, TORCH_TIMERS, TURN_ON_OFF_WATER_CTR, WATER_HDMA_WINDOW_X,
    WATER_HDMA_WINDOW_X_RADIUS, WATER_HDMA_WINDOW_Y, WATER_HDMA_WINDOW_Y_RADIUS,
    WATER_HDMA_WINDOW_Y_RADIUS_ALT, WATER_HDMA_WINDOW_Y_TARGET, WATER_SIDE_STEP_SWITCH,
};
use crate::game_state::constants::{
    COUNTDOWN_TIMER_FOR_STAIRCASES, CUR_STAIRCASE_PLANE, KIND_OF_IN_ROOM_STAIRCASE,
    STAIRCASE_LOWER_LEVEL_STATUS, STAIRCASE_MOVE_COUNTER, STAIRCASE_TILEMAP_POS_X2,
    WHICH_STAIRCASE_INDEX,
};
use crate::game_state::constants::{
    DUNGEON_ROOM_HISTORY, DUNGEON_ROOM_INDEX2, DUNGEON_ROOM_INDEX_PREV,
};
use crate::types::{read_le_u16, write_le_u16};

const DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT: usize = 5;
const DUNGEON_HEADER_PLANE_SCRATCH_COUNT: usize = 5;
const DUNGEON_HEADER_TAG_COUNT: usize = 2;
const DUNGEON_TORCH_TIMER_COUNT: usize = 16;
const DUNGEON_TORCH_OBJECT_POS_COUNT: usize = 16;
const DUNGEON_TORCH_DATA_SCAN_BYTES: usize = 0x0120;
const DUNGEON_TORCH_DATA_SCAN_WORDS: usize = DUNGEON_TORCH_DATA_SCAN_BYTES / 2;
const DUNGEON_ROOM_HISTORY_COUNT: usize = 4;
const DUNGEON_OBJECT_SLOT_COUNT: usize = 16;
const DUNGEON_ROOM_ITEM_SLOT_COUNT: usize = 16;
// dung_chest_locations (0x6e0) is bounded by dung_stairs_table_2 (0x6ec) = 6 words; the
// game caps chests at 6 (CHEST_OPEN_MASKS). Sizing the native array to the shared
// SLOT_COUNT (16) made its write_to_ram project 0x6e0..0x700, clobbering stairs_table_2
// + toggle_floor on every chest sync (RoomDraw_Chest). Cap it to the C span.
const DUNGEON_CHEST_LOCATION_COUNT: usize = (DUNG_STAIRS_TABLE_2 - DUNG_CHEST_LOCATIONS) / 2;
const MOVING_WALL_REPLACEMENT_WORDS: usize = 64;
// star_shaped_switches_tile (0x6a0) is bounded by dung_inter_staircases (0x6b0): only
// 8 u16 slots (0x6a0..0x6b0). Modeling 16 made the parser's write_to_ram project
// 0x6a0..0x6bf, clobbering DungeonStairListsState's inter_staircases (0x6b0) and
// stairs_table_1 (0x6b8) every room-load sync (e.g. wiping a room's stair positions
// recorded the same frame — frame 7804 room 0x61).
const DUNGEON_ROOM_PARSER_SMALL_TABLE_COUNT: usize =
    (DUNG_INTER_STAIRCASES - STAR_SHAPED_SWITCHES_TILE_LOCAL) / 2;
const DUNGEON_ROOM_TOGGLE_SLOT_COUNT: usize = 8;
const DUNGEON_POT_REVEAL_ROOM_COUNT: usize = 0x140;
const DUNGEON_ADJACENT_DOOR_COUNT: usize = 8;
const DUNGEON_EXIT_DOOR_COUNT: usize = 4;
const DUNGEON_MOVABLE_BLOCK_RECORD_COUNT: usize = 0x018c / 4;
const CHANGEABLE_DUNGEON_OBJECT_SLOT_COUNT: usize = 2;
const DUNGEON_DOOR_SLOT_COUNT: usize = 16;
const DUNGEON_ROOM_TILEMAP_WORDS: usize = (DUNG_BG1 - DUNG_BG2) / 2;
const DUNGEON_DRAW_LINE_POINTER_BYTES: usize = 33;
const DUNGEON_BG2_ATTR_BUFFER_LEN: usize = (DUNGEON_BG1_ATTR_TABLE - DUNGEON_BG2_ATTR_TABLE) * 2;
const DUNGEON_BG1_ATTR_BUFFER_OFFSET: usize = DUNGEON_BG1_ATTR_TABLE - DUNGEON_BG2_ATTR_TABLE;
const DUNGEON_STAIR_LIST_COUNT: usize = 21;
const DUNGEON_INTER_STAIRCASE_TABLE_WORDS: usize =
    (DUNG_STAIRS_TABLE_1 - DUNG_INTER_STAIRCASES) / 2;
// dung_stairs_table_1 (0x6b8) is physically followed by dung_chest_locations
// (0x6e0) and then dung_stairs_table_2 (0x6ec) — a SNES byte-reuse: the nominal
// gap to stairs_table_2 spans OVER chest_locations. Sizing this table to that full
// gap made DungeonStairListsState project a stale 0x6e0..0x6eb over chest_locations
// on every stair sync (clobbering DungeonRoomItemState.chest_locations). Cap it at
// the chest_locations boundary so chest_locations stays the sole owner of 0x6e0+.
const DUNGEON_STAIR_TABLE_1_WORDS: usize = (DUNG_CHEST_LOCATIONS - DUNG_STAIRS_TABLE_1) / 2;
const DUNGEON_STAIR_TABLE_2_WORDS: usize = (DUNGEON_DOOR_DEBRIS_X - DUNG_STAIRS_TABLE_2) / 2;

pub(crate) fn loaded_room_data_word(ram: &[u8], offset: usize, index: usize) -> u16 {
    read_le_u16(ram, offset + index * 2)
}

fn door_info_word(door_info: &[u8], offset: usize) -> u16 {
    u16::from(door_info.get(offset).copied().unwrap_or(0))
        | (u16::from(door_info.get(offset + 1).copied().unwrap_or(0)) << 8)
}

#[derive(Clone, Copy)]
pub(crate) enum DungeonStairList {
    InterRoomUpNorth,
    InterRoomSouthDown,
    InRoomUpNorth,
    InRoomSouthDown,
    InterPseudoUpNorth,
    InRoomUpNorthWater,
    ActivatedWaterLadders,
    WetStairs,
    InRoomUpSouthWater,
    Stairs1,
    Stairs2,
    WaterLadders,
    WaterSideStepSwitch,
    WallUpNorthSpiral,
    WallDownNorthSpiral,
    WallUpNorthSpiralBg1,
    WallDownNorthSpiralBg1,
    InterRoomUpNorthStraight,
    InterRoomUpSouthStraight,
    InterRoomDownNorthStraight,
    InterRoomDownSouthStraight,
}

const DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS_LOCAL: usize = 0x0438;
const DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS_LOCAL: usize = 0x043a;
const DUNG_NUM_INROOM_SOUTHDOWN_STAIRS_LOCAL: usize = 0x043e;
const DUNG_NUM_WATER_LADDERS_LOCAL: usize = 0x0446;
const DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_LOCAL: usize = 0x047e;
const DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_LOCAL: usize = 0x0480;
const DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2_LOCAL: usize = 0x0482;
const DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2_LOCAL: usize = 0x0484;
const DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS_LOCAL: usize = 0x04a2;
const DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS_LOCAL: usize = 0x04a4;
const DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS_LOCAL: usize = 0x04a6;
const DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS_LOCAL: usize = 0x04a8;
const DUNG_NUM_STAR_SHAPED_SWITCHES_LOCAL: usize = 0x0432;
const STAR_SHAPED_SWITCHES_TILE_LOCAL: usize = 0x06a0;
const POTS_REVEALED_IN_ROOM_DUNGEON_LOCAL: usize = 0x0f580;
const DUNG_STAIRS_TABLE_1: usize = 0x06b8;
const DUNG_STAIRS_TABLE_2: usize = 0x06ec;
const DUNGEON_DOOR_DEBRIS_X: usize = 0x0728;

const DUNGEON_DRAW_OBJECT_OFFSETS_BG1: [u8; DUNGEON_DRAW_LINE_POINTER_BYTES] = [
    0, 0x20, 0x7e, 2, 0x20, 0x7e, 4, 0x20, 0x7e, 6, 0x20, 0x7e, 0x80, 0x20, 0x7e, 0x82, 0x20, 0x7e,
    0x84, 0x20, 0x7e, 0x86, 0x20, 0x7e, 0, 0x21, 0x7e, 0x80, 0x21, 0x7e, 0, 0x22, 0x7e,
];
const DUNGEON_DRAW_OBJECT_OFFSETS_BG2: [u8; DUNGEON_DRAW_LINE_POINTER_BYTES] = [
    0, 0x40, 0x7e, 2, 0x40, 0x7e, 4, 0x40, 0x7e, 6, 0x40, 0x7e, 0x80, 0x40, 0x7e, 0x82, 0x40, 0x7e,
    0x84, 0x40, 0x7e, 0x86, 0x40, 0x7e, 0, 0x41, 0x7e, 0x80, 0x41, 0x7e, 0, 0x42, 0x7e,
];

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonState {
    pub(crate) header: DungeonHeaderState,
    pub(crate) scratch_word: DungeonScratchWordState,
    pub(crate) entrance_backup: DungeonEntranceBackupState,
    pub(crate) torch: DungeonTorchState,
    pub(crate) savegame_state: DungeonSavegameState,
    pub(crate) bg2_attributes: DungeonBg2AttributeState,
    pub(crate) stair_lists: DungeonStairListsState,
    pub(crate) stair_movement: DungeonStairMovementState,
    pub(crate) moving_floor: DungeonMovingFloorState,
    pub(crate) room_tracking: DungeonRoomTrackingState,
    pub(crate) object_tracking: DungeonObjectTrackingState,
    pub(crate) doors: DungeonDoorState,
    pub(crate) room_load: DungeonRoomLoadState,
    pub(crate) environment: DungeonEnvironmentState,
    pub(crate) room_tilemaps: DungeonRoomTilemapState,
    pub(crate) room_items: DungeonRoomItemState,
    pub(crate) room_effects: DungeonRoomEffectsState,
    pub(crate) room_parser: DungeonRoomParserState,
    pub(crate) door_setup: DungeonRoomDoorSetupState,
    pub(crate) room_runtime: DungeonRoomRuntimeState,
    pub(crate) movable_blocks: DungeonMovableBlockState,
}

impl DungeonState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            header: DungeonHeaderState::load_from_ram(ram),
            scratch_word: DungeonScratchWordState::load_from_ram(ram),
            entrance_backup: DungeonEntranceBackupState::load_from_ram(ram),
            torch: DungeonTorchState::load_from_ram(ram),
            savegame_state: DungeonSavegameState::load_from_ram(ram),
            bg2_attributes: DungeonBg2AttributeState::load_from_ram(ram),
            stair_lists: DungeonStairListsState::load_from_ram(ram),
            stair_movement: DungeonStairMovementState::load_from_ram(ram),
            moving_floor: DungeonMovingFloorState::load_from_ram(ram),
            room_tracking: DungeonRoomTrackingState::load_from_ram(ram),
            object_tracking: DungeonObjectTrackingState::load_from_ram(ram),
            doors: DungeonDoorState::load_from_ram(ram),
            room_load: DungeonRoomLoadState::load_from_ram(ram),
            environment: DungeonEnvironmentState::load_from_ram(ram),
            room_tilemaps: DungeonRoomTilemapState::load_from_ram(ram),
            room_items: DungeonRoomItemState::load_from_ram(ram),
            room_effects: DungeonRoomEffectsState::load_from_ram(ram),
            room_parser: DungeonRoomParserState::load_from_ram(ram),
            door_setup: DungeonRoomDoorSetupState::load_from_ram(ram),
            room_runtime: DungeonRoomRuntimeState::load_from_ram(ram),
            movable_blocks: DungeonMovableBlockState::load_from_ram(ram),
        }
    }

    /// Leaf-level coherence drill-down (see GameState::report_incoherent_with_ram):
    /// returns "dungeon.<leaf>" for each sub-state that has drifted out of sync with RAM.
    pub(crate) fn report_incoherent_with_ram(&self, ram: &[u8]) -> Vec<&'static str> {
        let fresh = Self::load_from_ram(ram);
        let mut out = Vec::new();
        macro_rules! check {
            ($field:ident) => {
                if self.$field != fresh.$field {
                    out.push(concat!("dungeon.", stringify!($field)));
                }
            };
        }
        check!(header);
        check!(scratch_word);
        check!(entrance_backup);
        check!(torch);
        check!(savegame_state);
        check!(bg2_attributes);
        check!(stair_lists);
        check!(stair_movement);
        check!(moving_floor);
        check!(room_tracking);
        check!(object_tracking);
        check!(doors);
        check!(room_load);
        check!(environment);
        check!(room_tilemaps);
        check!(room_items);
        check!(room_effects);
        check!(room_parser);
        check!(door_setup);
        check!(room_runtime);
        check!(movable_blocks);
        out
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.header.write_to_ram(ram);
        self.scratch_word.write_to_ram(ram);
        self.entrance_backup.write_to_ram(ram);
        self.torch.write_to_ram(ram);
        self.savegame_state.write_to_ram(ram);
        self.bg2_attributes.write_to_ram(ram);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonRoomTilemapState {
    bg1_tiles: Vec<u16>,
    bg2_tiles: Vec<u16>,
    line_pointer_bytes: Vec<u8>,
}

impl Default for DungeonRoomTilemapState {
    fn default() -> Self {
        Self {
            bg1_tiles: vec![0; DUNGEON_ROOM_TILEMAP_WORDS],
            bg2_tiles: vec![0; DUNGEON_ROOM_TILEMAP_WORDS],
            line_pointer_bytes: vec![0; DUNGEON_DRAW_LINE_POINTER_BYTES],
        }
    }
}

impl DungeonRoomTilemapState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for index in 0..DUNGEON_ROOM_TILEMAP_WORDS {
            state.bg1_tiles[index] = read_le_u16(ram, DUNG_BG1 + index * 2);
            state.bg2_tiles[index] = read_le_u16(ram, DUNG_BG2 + index * 2);
        }
        for (index, byte) in state.line_pointer_bytes.iter_mut().enumerate() {
            *byte = ram.get(DUNG_LINE_PTRS_ROW0 + index).copied().unwrap_or(0);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.write_tilemaps_to_ram(ram);
        self.write_line_pointers_to_ram(ram);
    }

    /// Project only the BG1/BG2 tilemaps. The line-pointer buffer
    /// (`DUNG_LINE_PTRS_ROW0` = 0xbf, 33 bytes → 0xbf-0xdf) overlaps the dungeon
    /// work registers R16/R18 (0xc8-0xcb) and the intro-sword bytes by SNES byte
    /// reuse, so it must NOT be re-stamped on every tile write — that would
    /// clobber the live scratch a lifted-tile/probe just set.
    pub(crate) fn write_tilemaps_to_ram(&self, ram: &mut [u8]) {
        for (index, tile) in self.bg1_tiles.iter().enumerate() {
            write_le_u16(ram, DUNG_BG1 + index * 2, *tile);
        }
        for (index, tile) in self.bg2_tiles.iter().enumerate() {
            write_le_u16(ram, DUNG_BG2 + index * 2, *tile);
        }
    }

    /// Project the line-pointer buffer (0xbf-0xdf). C writes these bytes only
    /// when room draw actually computes line pointers, so only the dedicated
    /// line-pointer setters call this — matching the C site that overwrites the
    /// overlapping work registers at exactly that moment.
    pub(crate) fn write_line_pointers_to_ram(&self, ram: &mut [u8]) {
        for (index, byte) in self.line_pointer_bytes.iter().enumerate() {
            ram[DUNG_LINE_PTRS_ROW0 + index] = *byte;
        }
    }

    /// Re-read the line-pointer cache from RAM so it tracks whatever currently
    /// occupies the overlapping scratch bytes. Used after a tilemap-only sync to
    /// keep the native cache coherent with RAM without writing the bytes back.
    pub(crate) fn reload_line_pointers_from_ram(&mut self, ram: &[u8]) {
        for (index, byte) in self.line_pointer_bytes.iter_mut().enumerate() {
            *byte = ram.get(DUNG_LINE_PTRS_ROW0 + index).copied().unwrap_or(0);
        }
    }

    /// Mirror the four words written by an overworld map32 decode (`dst`,
    /// `dst+2`, `dst+128`, `dst+130`) from RAM back into the live bg2 cache.
    ///
    /// The overworld map16 decode writes the BG2 tilemap as raw RAM, bypassing
    /// this cache. Without mirroring, the cache (loaded at frame start) stays
    /// stale: overworld readers (`overworld_bg2_word`) would read pre-decode
    /// tiles and the frame-end projection would clobber the decoded RAM.
    pub(crate) fn mirror_decoded_map32_from_ram(&mut self, ram: &[u8], dst: usize) {
        for offset in [0usize, 2, 128, 130] {
            let addr = dst + offset;
            if addr >= DUNG_BG2 && addr + 1 < DUNG_BG2 + DUNGEON_ROOM_TILEMAP_WORDS * 2 {
                self.bg2_tiles[(addr - DUNG_BG2) / 2] = read_le_u16(ram, addr);
            }
        }
    }

    pub(crate) fn bg1_tilemap_base(&self) -> usize {
        DUNG_BG1
    }

    pub(crate) fn bg2_tilemap_base(&self) -> usize {
        DUNG_BG2
    }

    pub(crate) fn bg1_tile(&self, index: usize) -> u16 {
        self.bg1_tiles.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn bg2_tile(&self, index: usize) -> u16 {
        // Mirror set_bg2_tile's spill: an index past the BG2 word count reads the contiguous
        // BG1 span (C reads dung_bg2[index] flat).
        if index < DUNGEON_ROOM_TILEMAP_WORDS {
            self.bg2_tiles.get(index).copied().unwrap_or(0)
        } else {
            self.bg1_tiles
                .get(index - DUNGEON_ROOM_TILEMAP_WORDS)
                .copied()
                .unwrap_or(0)
        }
    }

    /// Tile source for the basic-attribute fill, which scans 0x2000 words. C
    /// reads `dung_bg2[index]`; dung_bg2 (0x2000) and dung_bg1 (0x4000) are
    /// contiguous in WRAM, so for index >= the BG2 tilemap length it actually
    /// reads the BG1 tilemap — that is how the BG1 attribute table's half is
    /// generated (the Rust buffers are separate, so fall through explicitly).
    pub(crate) fn attr_source_tile(&self, index: usize) -> u16 {
        if index < DUNGEON_ROOM_TILEMAP_WORDS {
            self.bg2_tile(index)
        } else {
            self.bg1_tile(index - DUNGEON_ROOM_TILEMAP_WORDS)
        }
    }

    pub(crate) fn bg1_tile_by_byte_pos(&self, pos: u16) -> u16 {
        self.bg1_tile((pos >> 1) as usize)
    }

    pub(crate) fn bg2_tile_by_byte_pos(&self, pos: u16) -> u16 {
        self.bg2_tile((pos >> 1) as usize)
    }

    pub(crate) fn line_pointer_row0(&self, index: usize) -> u16 {
        self.line_pointer_word(index * 2)
    }

    pub(crate) fn first_line_pointer_row0(&self) -> u16 {
        self.line_pointer_row0(0)
    }

    pub(crate) fn room_tilemap_word(&self, base: usize, dsto: u16) -> u16 {
        self.tile_at_abs(base + dsto as usize * 2)
    }

    pub(crate) fn room_tilemap_word_by_byte_offset(&self, base: usize, byte_offset: usize) -> u16 {
        self.tile_at_abs(base + byte_offset)
    }

    /// Read a tilemap word by absolute WRAM byte offset, mirroring the raw-RAM
    /// semantics: a BG2 access whose offset overruns the BG2 span (the `0x1000`
    /// lower-level bit on a door position) spills into the contiguous BG1 span,
    /// exactly as `base + dsto*2` does against flat RAM.
    fn tile_at_abs(&self, abs: usize) -> u16 {
        let span = DUNGEON_ROOM_TILEMAP_WORDS * 2;
        if abs >= DUNG_BG2 && abs < DUNG_BG2 + span {
            self.bg2_tile((abs - DUNG_BG2) / 2)
        } else if abs >= DUNG_BG1 && abs < DUNG_BG1 + span {
            self.bg1_tile((abs - DUNG_BG1) / 2)
        } else {
            0
        }
    }

    fn set_tile_at_abs(&mut self, abs: usize, value: u16) {
        let span = DUNGEON_ROOM_TILEMAP_WORDS * 2;
        if abs >= DUNG_BG2 && abs < DUNG_BG2 + span {
            self.set_bg2_tile((abs - DUNG_BG2) / 2, value);
        } else if abs >= DUNG_BG1 && abs < DUNG_BG1 + span {
            self.set_bg1_tile((abs - DUNG_BG1) / 2, value);
        }
    }

    fn line_pointer_word(&self, byte_offset: usize) -> u16 {
        let lo = self
            .line_pointer_bytes
            .get(byte_offset)
            .copied()
            .unwrap_or(0);
        let hi = self
            .line_pointer_bytes
            .get(byte_offset + 1)
            .copied()
            .unwrap_or(0);
        u16::from(lo) | (u16::from(hi) << 8)
    }

    fn set_bg1_tile(&mut self, index: usize, value: u16) {
        if let Some(tile) = self.bg1_tiles.get_mut(index) {
            *tile = value;
        }
    }

    fn set_bg2_tile(&mut self, index: usize, value: u16) {
        // dung_bg2 (0x2000) and dung_bg1 (0x4000) are contiguous in WRAM; C writes
        // `dung_bg2[index]` flat, so an index past the BG2 word count addresses the BG1 span
        // (e.g. a lifted/opened 2x2 object beyond the lower-level boundary). Spill into BG1
        // instead of dropping the OOB write. (set_tile_at_abs only calls this with index <
        // the word count, so this never recurses.)
        if index < DUNGEON_ROOM_TILEMAP_WORDS {
            if let Some(tile) = self.bg2_tiles.get_mut(index) {
                *tile = value;
            }
        } else if let Some(tile) = self.bg1_tiles.get_mut(index - DUNGEON_ROOM_TILEMAP_WORDS) {
            *tile = value;
        }
    }

    fn set_bg1_tile_by_byte_pos(&mut self, pos: u16, value: u16) {
        self.set_bg1_tile((pos >> 1) as usize, value);
    }

    fn set_bg2_tile_by_byte_pos(&mut self, pos: u16, value: u16) {
        self.set_bg2_tile((pos >> 1) as usize, value);
    }

    fn set_room_tilemap_word(&mut self, base: usize, dsto: u16, value: u16) {
        self.set_tile_at_abs(base + dsto as usize * 2, value);
    }

    fn set_room_tilemap_word_by_byte_offset(
        &mut self,
        base: usize,
        byte_offset: usize,
        value: u16,
    ) {
        self.set_tile_at_abs(base + byte_offset, value);
    }

    fn set_line_pointer_row0(&mut self, index: usize, value: u16) {
        let offset = index * 2;
        if offset + 1 < self.line_pointer_bytes.len() {
            self.line_pointer_bytes[offset] = value as u8;
            self.line_pointer_bytes[offset + 1] = (value >> 8) as u8;
        }
    }

    fn copy_line_pointer_bytes(&mut self, offsets: &[u8]) {
        let len = offsets.len().min(self.line_pointer_bytes.len());
        self.line_pointer_bytes[..len].copy_from_slice(&offsets[..len]);
    }

    fn copy_bg2_draw_line_offsets(&mut self) {
        self.copy_line_pointer_bytes(&DUNGEON_DRAW_OBJECT_OFFSETS_BG2);
    }

    fn copy_bg1_draw_line_offsets(&mut self) {
        self.copy_line_pointer_bytes(&DUNGEON_DRAW_OBJECT_OFFSETS_BG1);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonEnvironmentState {
    water_transition_counter: u8,
    water_hdma_y_radius: u16,
    water_hdma_x_radius: u16,
    water_hdma_y_target: u16,
    water_hdma_y_radius_alt: u16,
    water_window_x: u16,
    water_window_y: u16,
    water_puzzle_state_changed: u8,
    trapdoors_down: u16,
    somaria_block_switch_counter: u8,
    somaria_block_bg_check_flag: u8,
    orange_blue_barrier_state: u8,
    moving_floor_check_flags: u16,
    movable_block_was_pushed: u8,
    block_trap_related_tile: u16,
}

impl DungeonEnvironmentState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            water_transition_counter: ram.get(TURN_ON_OFF_WATER_CTR).copied().unwrap_or(0),
            water_hdma_y_radius: read_le_u16(ram, WATER_HDMA_WINDOW_Y_RADIUS),
            water_hdma_x_radius: read_le_u16(ram, WATER_HDMA_WINDOW_X_RADIUS),
            water_hdma_y_target: read_le_u16(ram, WATER_HDMA_WINDOW_Y_TARGET),
            water_hdma_y_radius_alt: read_le_u16(ram, WATER_HDMA_WINDOW_Y_RADIUS_ALT),
            water_window_x: read_le_u16(ram, WATER_HDMA_WINDOW_X),
            water_window_y: read_le_u16(ram, WATER_HDMA_WINDOW_Y),
            water_puzzle_state_changed: ram
                .get(DUNG_FLAG_STATECHANGE_WATERPUZZLE)
                .copied()
                .unwrap_or(0),
            trapdoors_down: read_le_u16(ram, DUNG_FLAG_TRAPDOORS_DOWN),
            somaria_block_switch_counter: ram
                .get(DUNG_FLAG_SOMARIA_BLOCK_SWITCH)
                .copied()
                .unwrap_or(0),
            somaria_block_bg_check_flag: ram.get(SOMARIA_BLOCK_BG_CHECK_FLAG).copied().unwrap_or(0),
            orange_blue_barrier_state: ram.get(ORANGE_BLUE_BARRIER_STATE).copied().unwrap_or(0),
            moving_floor_check_flags: read_le_u16(ram, MOVING_FLOOR_BG_CHECK_FLAGS),
            movable_block_was_pushed: ram
                .get(DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED)
                .copied()
                .unwrap_or(0),
            block_trap_related_tile: read_le_u16(ram, BLOCK_TRAP_CHECK_FLAG),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[TURN_ON_OFF_WATER_CTR] = self.water_transition_counter;
        write_le_u16(ram, WATER_HDMA_WINDOW_Y_RADIUS, self.water_hdma_y_radius);
        write_le_u16(ram, WATER_HDMA_WINDOW_X_RADIUS, self.water_hdma_x_radius);
        write_le_u16(ram, WATER_HDMA_WINDOW_Y_TARGET, self.water_hdma_y_target);
        write_le_u16(
            ram,
            WATER_HDMA_WINDOW_Y_RADIUS_ALT,
            self.water_hdma_y_radius_alt,
        );
        write_le_u16(ram, WATER_HDMA_WINDOW_X, self.water_window_x);
        write_le_u16(ram, WATER_HDMA_WINDOW_Y, self.water_window_y);
        ram[DUNG_FLAG_STATECHANGE_WATERPUZZLE] = self.water_puzzle_state_changed;
        write_le_u16(ram, DUNG_FLAG_TRAPDOORS_DOWN, self.trapdoors_down);
        ram[DUNG_FLAG_SOMARIA_BLOCK_SWITCH] = self.somaria_block_switch_counter;
        ram[SOMARIA_BLOCK_BG_CHECK_FLAG] = self.somaria_block_bg_check_flag;
        ram[ORANGE_BLUE_BARRIER_STATE] = self.orange_blue_barrier_state;
        write_le_u16(
            ram,
            MOVING_FLOOR_BG_CHECK_FLAGS,
            self.moving_floor_check_flags,
        );
        ram[DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED] = self.movable_block_was_pushed;
        write_le_u16(ram, BLOCK_TRAP_CHECK_FLAG, self.block_trap_related_tile);
    }

    pub(crate) fn water_transition_counter(&self) -> u8 {
        self.water_transition_counter
    }

    pub(crate) fn water_hdma_y_radius(&self) -> u16 {
        self.water_hdma_y_radius
    }

    pub(crate) fn water_hdma_x_radius(&self) -> u16 {
        self.water_hdma_x_radius
    }

    pub(crate) fn water_hdma_y_target(&self) -> u16 {
        self.water_hdma_y_target
    }

    pub(crate) fn water_hdma_y_radius_alt(&self) -> u16 {
        self.water_hdma_y_radius_alt
    }

    pub(crate) fn water_puzzle_state_changed(&self) -> u8 {
        self.water_puzzle_state_changed
    }

    pub(crate) fn trapdoors_down(&self) -> u16 {
        self.trapdoors_down
    }

    pub(crate) fn trapdoors_down_low(&self) -> u8 {
        self.trapdoors_down as u8
    }

    pub(crate) fn somaria_block_switch_counter(&self) -> u8 {
        self.somaria_block_switch_counter
    }

    pub(crate) fn orange_blue_barrier_state(&self) -> u8 {
        self.orange_blue_barrier_state
    }

    pub(crate) fn moving_floor_check_flags(&self) -> u16 {
        self.moving_floor_check_flags
    }

    pub(crate) fn movable_block_was_pushed(&self) -> u8 {
        self.movable_block_was_pushed
    }

    pub(crate) fn movable_block_was_not_pushed_flag(&self) -> u8 {
        self.movable_block_was_pushed ^ 1
    }

    pub(crate) fn block_trap_related_tile(&self) -> u16 {
        self.block_trap_related_tile
    }

    fn set_water_transition_counter(&mut self, value: u8) {
        self.water_transition_counter = value;
    }

    fn increment_water_transition_counter(&mut self) -> u8 {
        self.water_transition_counter = self.water_transition_counter.wrapping_add(1);
        self.water_transition_counter
    }

    fn decrement_water_transition_counter(&mut self) -> u8 {
        self.water_transition_counter = self.water_transition_counter.wrapping_sub(1);
        self.water_transition_counter
    }

    fn set_water_hdma_y_radius(&mut self, value: u16) {
        self.water_hdma_y_radius = value;
    }

    fn set_water_hdma_x_radius(&mut self, value: u16) {
        self.water_hdma_x_radius = value;
    }

    fn set_water_hdma_y_target(&mut self, value: u16) {
        self.water_hdma_y_target = value;
    }

    fn set_water_hdma_y_radius_alt(&mut self, value: u16) {
        self.water_hdma_y_radius_alt = value;
    }

    fn set_water_window_position(&mut self, x: u16, y: u16) {
        self.water_window_x = x;
        self.water_window_y = y;
    }

    fn clear_water_puzzle_state_changed(&mut self) {
        self.water_puzzle_state_changed = 0;
    }

    fn set_water_puzzle_state_changed(&mut self, value: u8) {
        self.water_puzzle_state_changed = value;
    }

    fn increment_water_puzzle_state_changed(&mut self) -> u8 {
        self.water_puzzle_state_changed = self.water_puzzle_state_changed.wrapping_add(1);
        self.water_puzzle_state_changed
    }

    fn set_trapdoors_down(&mut self, value: u16) {
        self.trapdoors_down = value;
    }

    fn clear_trapdoors_down(&mut self) {
        self.trapdoors_down = 0;
    }

    fn set_trapdoors_down_low(&mut self, value: u8) {
        self.trapdoors_down = (self.trapdoors_down & 0xff00) | u16::from(value);
    }

    fn increment_trapdoors_down_low(&mut self) -> u8 {
        let value = self.trapdoors_down_low().wrapping_add(1);
        self.set_trapdoors_down_low(value);
        value
    }

    fn clear_somaria_block_switch_counter(&mut self) {
        self.somaria_block_switch_counter = 0;
    }

    fn increment_somaria_block_switch_counter(&mut self) {
        self.somaria_block_switch_counter = self.somaria_block_switch_counter.wrapping_add(1);
    }

    fn increment_somaria_block_bg_check_flag(&mut self) {
        self.somaria_block_bg_check_flag = self.somaria_block_bg_check_flag.wrapping_add(1);
    }

    fn clear_orange_blue_barrier_state(&mut self) {
        self.orange_blue_barrier_state = 0;
    }

    fn toggle_orange_blue_barrier_state(&mut self) {
        self.orange_blue_barrier_state ^= 1;
    }

    fn clear_moving_floor_check_flags(&mut self) {
        self.moving_floor_check_flags = 0;
    }

    fn or_moving_floor_check_flags(&mut self, bits: u16) -> u16 {
        self.moving_floor_check_flags |= bits;
        self.moving_floor_check_flags
    }

    fn clear_movable_block_was_pushed(&mut self) {
        self.movable_block_was_pushed = 0;
    }

    fn toggle_movable_block_was_pushed(&mut self) {
        self.movable_block_was_pushed ^= 1;
    }

    fn set_block_trap_related_tile(&mut self, value: u16) {
        self.block_trap_related_tile = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonRoomLoadState {
    header_collision: u8,
    header_collision_2: u8,
    header_collision_2_mirror: u16,
    bg2_properties: u8,
    bg2_properties_backup: u8,
    // NOTE: dung_layout_and_starting_quadrant (0x40e) is NOT owned here — it is a
    // uint16 owned solely by DungeonRoomParserState.room_layout_and_starting_quadrant.
    layout_quadrant_key: u8,
    quadrants_visited: u16,
    quadrant_upload_index: u8,
    draw_width_indicator: u16,
    draw_height_indicator: u16,
    overlay_to_load: u8,
    selected_key_door_x2: u16,
    load_ptr_offset: u16,
    loading_bg_offset_h: u16,
    loading_bg_offset_v: u16,
}

impl DungeonRoomLoadState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            header_collision: ram.get(DUNG_HDR_COLLISION).copied().unwrap_or(0),
            header_collision_2: ram.get(DUNG_HDR_COLLISION_2).copied().unwrap_or(0),
            header_collision_2_mirror: read_le_u16(ram, DUNGEON_HEADER_COLLISION_2_MIRROR),
            bg2_properties: ram.get(DUNG_HDR_BG2_PROPERTIES).copied().unwrap_or(0),
            bg2_properties_backup: ram
                .get(DUNG_HDR_BG2_PROPERTIES_BACKUP)
                .copied()
                .unwrap_or(0),
            layout_quadrant_key: ram
                .get(COMPOSITE_OF_LAYOUT_AND_QUADRANT)
                .copied()
                .unwrap_or(0),
            quadrants_visited: read_le_u16(ram, DUNG_QUADRANTS_VISITED),
            quadrant_upload_index: ram.get(DUNG_CUR_QUADRANT_UPLOAD).copied().unwrap_or(0),
            draw_width_indicator: read_le_u16(ram, DUNG_DRAW_WIDTH_INDICATOR),
            draw_height_indicator: read_le_u16(ram, DUNG_DRAW_HEIGHT_INDICATOR),
            overlay_to_load: ram.get(DUNG_OVERLAY_TO_LOAD).copied().unwrap_or(0),
            selected_key_door_x2: read_le_u16(ram, DUNG_WHICH_KEY_X2_DUNGEON),
            load_ptr_offset: read_le_u16(ram, DUNG_LOAD_PTR_OFFS),
            loading_bg_offset_h: read_le_u16(ram, DUNG_LOADE_BGOFFS_H_COPY),
            loading_bg_offset_v: read_le_u16(ram, DUNG_LOADE_BGOFFS_V_COPY),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DUNG_HDR_COLLISION] = self.header_collision;
        ram[DUNG_HDR_COLLISION_2] = self.header_collision_2;
        write_le_u16(
            ram,
            DUNGEON_HEADER_COLLISION_2_MIRROR,
            self.header_collision_2_mirror,
        );
        ram[DUNG_HDR_BG2_PROPERTIES] = self.bg2_properties;
        ram[DUNG_HDR_BG2_PROPERTIES_BACKUP] = self.bg2_properties_backup;
        ram[COMPOSITE_OF_LAYOUT_AND_QUADRANT] = self.layout_quadrant_key;
        write_le_u16(ram, DUNG_QUADRANTS_VISITED, self.quadrants_visited);
        ram[DUNG_CUR_QUADRANT_UPLOAD] = self.quadrant_upload_index;
        write_le_u16(ram, DUNG_DRAW_WIDTH_INDICATOR, self.draw_width_indicator);
        write_le_u16(ram, DUNG_DRAW_HEIGHT_INDICATOR, self.draw_height_indicator);
        ram[DUNG_OVERLAY_TO_LOAD] = self.overlay_to_load;
        write_le_u16(ram, DUNG_WHICH_KEY_X2_DUNGEON, self.selected_key_door_x2);
        write_le_u16(ram, DUNG_LOAD_PTR_OFFS, self.load_ptr_offset);
        write_le_u16(ram, DUNG_LOADE_BGOFFS_H_COPY, self.loading_bg_offset_h);
        write_le_u16(ram, DUNG_LOADE_BGOFFS_V_COPY, self.loading_bg_offset_v);
    }

    pub(crate) fn header_collision(&self) -> u8 {
        self.header_collision
    }

    pub(crate) fn header_collision_2(&self) -> u8 {
        self.header_collision_2
    }

    pub(crate) fn header_collision_2_mirror(&self) -> u8 {
        self.header_collision_2_mirror as u8
    }

    pub(crate) fn header_collision_2_mirror_high(&self) -> u8 {
        (self.header_collision_2_mirror >> 8) as u8
    }

    pub(crate) fn bg2_properties(&self) -> u8 {
        self.bg2_properties
    }

    pub(crate) fn layout_quadrant_key(&self) -> u8 {
        self.layout_quadrant_key
    }

    pub(crate) fn quadrants_visited(&self) -> u16 {
        self.quadrants_visited
    }

    pub(crate) fn quadrant_upload_index(&self) -> u8 {
        self.quadrant_upload_index
    }

    pub(crate) fn draw_width_indicator(&self) -> u8 {
        self.draw_width_indicator as u8
    }

    pub(crate) fn draw_width_indicator_word(&self) -> u16 {
        self.draw_width_indicator
    }

    pub(crate) fn draw_height_indicator(&self) -> u8 {
        self.draw_height_indicator as u8
    }

    pub(crate) fn draw_height_indicator_word(&self) -> u16 {
        self.draw_height_indicator
    }

    pub(crate) fn overlay_to_load(&self) -> u8 {
        self.overlay_to_load
    }

    pub(crate) fn selected_key_door_x2(&self) -> u16 {
        self.selected_key_door_x2
    }

    pub(crate) fn load_ptr_offset(&self) -> u16 {
        self.load_ptr_offset
    }

    pub(crate) fn loading_bg_offset_h(&self) -> u16 {
        self.loading_bg_offset_h
    }

    pub(crate) fn loading_bg_offset_v(&self) -> u16 {
        self.loading_bg_offset_v
    }

    fn set_header_collision(&mut self, value: u8) {
        self.header_collision = value;
    }

    fn set_header_collision_2(&mut self, value: u8) {
        self.header_collision_2 = value;
    }

    fn clear_header_collision_2(&mut self) {
        self.header_collision_2 = 0;
    }

    fn set_header_collision_2_mirror(&mut self, value: u8) {
        self.header_collision_2_mirror =
            (self.header_collision_2_mirror & 0xff00) | u16::from(value);
    }

    fn set_header_collision_2_mirror_high(&mut self, value: u8) {
        self.header_collision_2_mirror =
            (self.header_collision_2_mirror & 0x00ff) | (u16::from(value) << 8);
    }

    fn increment_header_collision_2_mirror(&mut self) -> u8 {
        let value = self.header_collision_2_mirror().wrapping_add(1);
        self.set_header_collision_2_mirror(value);
        value
    }

    fn copy_header_collision_2_to_mirror(&mut self) {
        self.set_header_collision_2_mirror(self.header_collision_2);
    }

    fn set_bg2_properties(&mut self, value: u8) {
        self.bg2_properties = value;
    }

    fn clear_bg2_properties(&mut self) {
        self.bg2_properties = 0;
    }

    fn set_bg2_properties_backup(&mut self, value: u8) {
        self.bg2_properties_backup = value;
    }

    fn set_layout_quadrant_key(&mut self, value: u8) {
        self.layout_quadrant_key = value;
    }

    fn update_layout_quadrant_key(&mut self, layout: u8, quadrant_y: u8, quadrant_x: u8) -> u8 {
        let key = layout | quadrant_y | quadrant_x;
        self.set_layout_quadrant_key(key);
        key
    }

    fn set_quadrants_visited(&mut self, value: u16) {
        self.quadrants_visited = value;
    }

    fn or_quadrants_visited(&mut self, value: u16) -> u16 {
        self.quadrants_visited |= value;
        self.quadrants_visited
    }

    fn clear_quadrant_upload_index(&mut self) {
        self.quadrant_upload_index = 0;
    }

    fn advance_quadrant_upload_index_by(&mut self, value: u8) -> u8 {
        self.quadrant_upload_index = self.quadrant_upload_index.wrapping_add(value);
        self.quadrant_upload_index
    }

    fn set_draw_width_indicator(&mut self, value: u8) {
        self.draw_width_indicator = (self.draw_width_indicator & 0xff00) | u16::from(value);
    }

    fn set_draw_width_indicator_word(&mut self, value: u16) {
        self.draw_width_indicator = value;
    }

    fn set_draw_height_indicator(&mut self, value: u8) {
        self.draw_height_indicator = (self.draw_height_indicator & 0xff00) | u16::from(value);
    }

    fn set_draw_height_indicator_word(&mut self, value: u16) {
        self.draw_height_indicator = value;
    }

    fn set_draw_dimensions(&mut self, width: u8, height: u8) {
        self.set_draw_width_indicator(width);
        self.set_draw_height_indicator(height);
    }

    fn set_draw_dimensions_words(&mut self, width: u16, height: u16) {
        self.draw_width_indicator = width;
        self.draw_height_indicator = height;
    }

    fn set_overlay_to_load(&mut self, value: u8) {
        self.overlay_to_load = value;
    }

    fn set_overlay_to_load_if_empty(&mut self, value: u8) {
        if self.overlay_to_load == 0 {
            self.overlay_to_load = value;
        }
    }

    fn clear_overlay_to_load(&mut self) {
        self.overlay_to_load = 0;
    }

    fn set_selected_key_door_x2(&mut self, value: u16) {
        self.selected_key_door_x2 = value;
    }

    fn set_selected_key_door(&mut self, door: usize) {
        self.selected_key_door_x2 = (door * 2) as u16;
    }

    fn set_load_ptr_offset(&mut self, value: u16) {
        self.load_ptr_offset = value;
    }

    fn advance_load_ptr_offset_by(&mut self, value: u16) -> u16 {
        self.load_ptr_offset = self.load_ptr_offset.wrapping_add(value);
        self.load_ptr_offset
    }

    fn set_loading_bg_offsets(&mut self, horizontal: u16, vertical: u16) {
        self.loading_bg_offset_h = horizontal;
        self.loading_bg_offset_v = vertical;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonObjectTrackingState {
    misc_object_index: u16,
    replacement_tile_states: [u16; DUNGEON_OBJECT_SLOT_COUNT],
    object_data_positions: [u16; DUNGEON_OBJECT_SLOT_COUNT],
    object_tilemap_positions: [u16; DUNGEON_OBJECT_SLOT_COUNT],
    changeable_object_indices: [u8; CHANGEABLE_DUNGEON_OBJECT_SLOT_COUNT],
    big_rock_starting_address: u16,
}

impl DungeonObjectTrackingState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut replacement_tile_states = [0; DUNGEON_OBJECT_SLOT_COUNT];
        for (index, state) in replacement_tile_states.iter_mut().enumerate() {
            *state = read_le_u16(ram, DUNGEON_REPLACEMENT_TILE_STATE + index * 2);
        }

        let mut object_data_positions = [0; DUNGEON_OBJECT_SLOT_COUNT];
        for (index, position) in object_data_positions.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2);
        }

        let mut object_tilemap_positions = [0; DUNGEON_OBJECT_SLOT_COUNT];
        for (index, position) in object_tilemap_positions.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_OBJECT_TILEMAP_POS + index * 2);
        }

        let mut changeable_object_indices = [0; CHANGEABLE_DUNGEON_OBJECT_SLOT_COUNT];
        for (index, object_index) in changeable_object_indices.iter_mut().enumerate() {
            *object_index = ram
                .get(CHANGEABLE_DUNGEON_OBJECT_INDEX + index)
                .copied()
                .unwrap_or(0);
        }

        Self {
            misc_object_index: read_le_u16(ram, DUNG_MISC_OBJS_INDEX),
            replacement_tile_states,
            object_data_positions,
            object_tilemap_positions,
            changeable_object_indices,
            big_rock_starting_address: read_le_u16(ram, BIG_ROCK_STARTING_ADDRESS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNG_MISC_OBJS_INDEX, self.misc_object_index);
        for (index, state) in self.replacement_tile_states.iter().enumerate() {
            write_le_u16(ram, DUNGEON_REPLACEMENT_TILE_STATE + index * 2, *state);
        }
        for (index, position) in self.object_data_positions.iter().enumerate() {
            write_le_u16(ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2, *position);
        }
        for (index, position) in self.object_tilemap_positions.iter().enumerate() {
            write_le_u16(ram, DUNG_OBJECT_TILEMAP_POS + index * 2, *position);
        }
        for (index, object_index) in self.changeable_object_indices.iter().enumerate() {
            ram[CHANGEABLE_DUNGEON_OBJECT_INDEX + index] = *object_index;
        }
        write_le_u16(
            ram,
            BIG_ROCK_STARTING_ADDRESS,
            self.big_rock_starting_address,
        );
    }

    pub(crate) fn misc_object_index(&self) -> u16 {
        self.misc_object_index
    }

    pub(crate) fn misc_object_slot(&self) -> usize {
        (self.misc_object_index >> 1) as usize
    }

    pub(crate) fn replacement_tile_state(&self, index: usize) -> u16 {
        self.replacement_tile_states
            .get(index)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn object_pos_in_objdata(&self, index: usize) -> u16 {
        self.object_data_positions.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn object_tilemap_pos(&self, index: usize) -> u16 {
        self.object_tilemap_positions
            .get(index)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn changeable_object_index(&self, index: usize) -> u8 {
        self.changeable_object_indices
            .get(index)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn big_rock_starting_address(&self) -> u16 {
        self.big_rock_starting_address
    }

    fn set_misc_object_index(&mut self, value: u16) {
        self.misc_object_index = value;
    }

    fn clear_misc_object_index(&mut self) {
        self.misc_object_index &= 0xff00;
    }

    fn advance_misc_object_index_by(&mut self, value: u16) -> u16 {
        self.misc_object_index = self.misc_object_index.wrapping_add(value);
        self.misc_object_index
    }

    fn clear_replacement_tile_states(&mut self) {
        self.replacement_tile_states.fill(0);
    }

    fn clear_object_data_positions(&mut self) {
        self.object_data_positions.fill(0);
    }

    fn set_replacement_tile_state(&mut self, index: usize, value: u16) {
        if let Some(state) = self.replacement_tile_states.get_mut(index) {
            *state = value;
        }
    }

    fn increment_replacement_tile_state(&mut self, index: usize) -> u16 {
        let Some(state) = self.replacement_tile_states.get_mut(index) else {
            return 0;
        };
        *state = state.wrapping_add(1);
        *state
    }

    fn clear_replacement_tile_state_low(&mut self, index: usize) {
        if let Some(state) = self.replacement_tile_states.get_mut(index) {
            *state &= 0xff00;
        }
    }

    fn set_object_data_pos(&mut self, index: usize, value: u16) {
        if let Some(position) = self.object_data_positions.get_mut(index) {
            *position = value;
        }
    }

    fn set_object_tilemap_pos(&mut self, index: usize, value: u16) {
        if let Some(position) = self.object_tilemap_positions.get_mut(index) {
            *position = value;
        }
    }

    fn set_changeable_object_index(&mut self, index: usize, value: u8) {
        if let Some(object_index) = self.changeable_object_indices.get_mut(index) {
            *object_index = value;
        }
    }

    fn clear_changeable_object_index(&mut self, index: usize) {
        self.set_changeable_object_index(index, 0);
    }

    fn set_big_rock_starting_address(&mut self, value: u16) {
        self.big_rock_starting_address = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonDoorState {
    opened_doors: u16,
    opened_doors_including_adjacent: u16,
    current_door_index: u16,
    current_door_pos: u16,
    animation_step: u16,
    open_counter: u16,
    door_tilemap_addresses: [u16; DUNGEON_DOOR_SLOT_COUNT],
    door_types: [u16; DUNGEON_DOOR_SLOT_COUNT],
    door_directions: [u16; DUNGEON_DOOR_SLOT_COUNT],
    switch_triggered: u8,
    barrier_or_switch_flag: u16,
}

impl DungeonDoorState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut door_tilemap_addresses = [0; DUNGEON_DOOR_SLOT_COUNT];
        for (door, address) in door_tilemap_addresses.iter_mut().enumerate() {
            *address = read_le_u16(ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2);
        }

        let mut door_types = [0; DUNGEON_DOOR_SLOT_COUNT];
        for (door, door_type) in door_types.iter_mut().enumerate() {
            *door_type = read_le_u16(ram, DOOR_TYPE_AND_SLOT + door * 2);
        }

        let mut door_directions = [0; DUNGEON_DOOR_SLOT_COUNT];
        for (door, direction) in door_directions.iter_mut().enumerate() {
            *direction = read_le_u16(ram, DUNGEON_DOOR_DIRECTION + door * 2);
        }

        Self {
            opened_doors: read_le_u16(ram, DUNG_DOOR_OPENED),
            opened_doors_including_adjacent: read_le_u16(ram, DUNG_DOOR_OPENED_INCL_ADJACENT),
            current_door_index: read_le_u16(ram, DUNG_CUR_DOOR_IDX),
            current_door_pos: read_le_u16(ram, DUNG_CUR_DOOR_POS_DUNGEON),
            animation_step: read_le_u16(ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON),
            open_counter: read_le_u16(ram, DOOR_OPEN_CLOSED_COUNTER),
            door_tilemap_addresses,
            door_types,
            door_directions,
            switch_triggered: ram.get(DUNG_DOOR_SWITCH_TRIGGERED).copied().unwrap_or(0),
            barrier_or_switch_flag: read_le_u16(ram, DUNG_DOOR_BARRIER_OR_SWITCH_FLAG),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNG_DOOR_OPENED, self.opened_doors);
        write_le_u16(
            ram,
            DUNG_DOOR_OPENED_INCL_ADJACENT,
            self.opened_doors_including_adjacent,
        );
        write_le_u16(ram, DUNG_CUR_DOOR_IDX, self.current_door_index);
        write_le_u16(ram, DUNG_CUR_DOOR_POS_DUNGEON, self.current_door_pos);
        write_le_u16(
            ram,
            DOOR_ANIMATION_STEP_INDICATOR_DUNGEON,
            self.animation_step,
        );
        write_le_u16(ram, DOOR_OPEN_CLOSED_COUNTER, self.open_counter);
        for (door, address) in self.door_tilemap_addresses.iter().enumerate() {
            write_le_u16(ram, DUNG_DOOR_TILEMAP_ADDRESS + door * 2, *address);
        }
        for (door, door_type) in self.door_types.iter().enumerate() {
            write_le_u16(ram, DOOR_TYPE_AND_SLOT + door * 2, *door_type);
        }
        for (door, direction) in self.door_directions.iter().enumerate() {
            write_le_u16(ram, DUNGEON_DOOR_DIRECTION + door * 2, *direction);
        }
        ram[DUNG_DOOR_SWITCH_TRIGGERED] = self.switch_triggered;
        write_le_u16(
            ram,
            DUNG_DOOR_BARRIER_OR_SWITCH_FLAG,
            self.barrier_or_switch_flag,
        );
    }

    pub(crate) fn opened_doors(&self) -> u16 {
        self.opened_doors
    }

    pub(crate) fn opened_doors_including_adjacent(&self) -> u16 {
        self.opened_doors_including_adjacent
    }

    pub(crate) fn has_opened_door_mask(&self, mask: u16) -> bool {
        self.opened_doors_including_adjacent & mask != 0
    }

    pub(crate) fn door_tilemap_address(&self, door: usize) -> u16 {
        self.door_tilemap_addresses.get(door).copied().unwrap_or(0)
    }

    pub(crate) fn door_type_and_slot(&self, door: usize) -> u8 {
        self.door_type_word(door) as u8
    }

    pub(crate) fn door_type_word(&self, door: usize) -> u16 {
        self.door_types.get(door).copied().unwrap_or(0)
    }

    pub(crate) fn door_direction(&self, door: usize) -> u8 {
        self.door_direction_word(door) as u8
    }

    pub(crate) fn door_direction_word(&self, door: usize) -> u16 {
        self.door_directions.get(door).copied().unwrap_or(0)
    }

    pub(crate) fn current_door_index(&self) -> u16 {
        self.current_door_index
    }

    pub(crate) fn current_door_slot(&self) -> usize {
        (self.current_door_index >> 1) as usize
    }

    pub(crate) fn current_door_pos(&self) -> u16 {
        self.current_door_pos
    }

    pub(crate) fn door_animation_step(&self) -> u16 {
        self.animation_step
    }

    pub(crate) fn door_animation_step_low(&self) -> u8 {
        self.animation_step as u8
    }

    pub(crate) fn door_open_counter(&self) -> u16 {
        self.open_counter
    }

    pub(crate) fn door_open_counter_low(&self) -> u8 {
        self.open_counter as u8
    }

    pub(crate) fn door_switch_triggered(&self) -> bool {
        self.switch_triggered != 0
    }

    fn set_opened_doors(&mut self, value: u16) {
        self.opened_doors = value;
    }

    fn or_opened_doors(&mut self, mask: u16) -> u16 {
        self.opened_doors |= mask;
        self.opened_doors
    }

    fn mark_door_opened(&mut self, door: usize) -> u16 {
        self.or_opened_doors(0x8000u16 >> (door & 15))
    }

    fn set_opened_doors_including_adjacent(&mut self, value: u16) {
        self.opened_doors_including_adjacent = value;
    }

    fn mark_opened_door_mask(&mut self, mask: u16) -> u16 {
        self.opened_doors_including_adjacent |= mask;
        self.opened_doors_including_adjacent
    }

    fn clear_door_tilemap_addresses(&mut self) {
        self.door_tilemap_addresses.fill(0);
    }

    fn set_door_tilemap_address(&mut self, door: usize, value: u16) {
        if let Some(address) = self.door_tilemap_addresses.get_mut(door) {
            *address = value;
        }
    }

    fn load_room_door_tilemap_addresses_from_info(&mut self, door_info: &[u8]) {
        for door in 0..DUNGEON_DOOR_SLOT_COUNT {
            let address = door_info_word(door_info, door * 2);
            if address == 0xffff {
                self.set_door_tilemap_address(door, 0);
                return;
            }
            self.set_door_tilemap_address(door, address);
        }
    }

    fn clear_door_tables(&mut self) {
        self.door_types.fill(0);
        self.door_directions.fill(0);
    }

    fn set_door_type_word(&mut self, door: usize, value: u16) {
        if let Some(door_type) = self.door_types.get_mut(door) {
            *door_type = value;
        }
    }

    fn set_door_direction_word(&mut self, door: usize, value: u16) {
        if let Some(direction) = self.door_directions.get_mut(door) {
            *direction = value;
        }
    }

    fn clear_door_direction(&mut self, door: usize) {
        self.set_door_direction_word(door, 0);
    }

    fn set_current_door_index(&mut self, value: u16) {
        self.current_door_index = value;
    }

    fn set_current_door_index_for_slot(&mut self, door: usize) {
        self.current_door_index = (door * 2) as u16;
    }

    fn advance_current_door_index_by(&mut self, value: u16) -> u16 {
        self.current_door_index = self.current_door_index.wrapping_add(value);
        self.current_door_index
    }

    fn set_current_door_pos(&mut self, value: u16) {
        self.current_door_pos = value;
    }

    fn clear_current_door_pos(&mut self) {
        self.current_door_pos = 0;
    }

    fn clear_current_door_pos_low_byte(&mut self) {
        self.current_door_pos &= 0xff00;
    }

    fn clear_door_animation_step(&mut self) {
        self.animation_step = 0;
    }

    fn set_door_animation_step(&mut self, value: u16) {
        self.animation_step = value;
    }

    fn sync_door_animation_step_from_ram(&mut self, ram: &[u8]) {
        self.animation_step = read_le_u16(ram, DOOR_ANIMATION_STEP_INDICATOR_DUNGEON);
    }

    fn set_door_animation_step_low(&mut self, value: u8) {
        self.animation_step = (self.animation_step & 0xff00) | u16::from(value);
    }

    fn increment_door_animation_step(&mut self) -> u16 {
        self.animation_step = self.animation_step.wrapping_add(1);
        self.animation_step
    }

    fn set_door_open_counter(&mut self, value: u16) {
        self.open_counter = value;
    }

    fn set_door_open_counter_low(&mut self, value: u8) {
        self.open_counter = (self.open_counter & 0xff00) | u16::from(value);
    }

    fn clear_door_open_counter_low(&mut self) {
        self.open_counter &= 0xff00;
    }

    fn increment_door_open_counter_low(&mut self) -> u8 {
        let value = self.door_open_counter_low().wrapping_add(1);
        self.set_door_open_counter_low(value);
        value
    }

    fn mark_door_switch_triggered(&mut self) {
        self.switch_triggered = 1;
    }

    fn clear_door_switch_triggered(&mut self) {
        self.switch_triggered = 0;
    }

    fn clear_door_barrier_or_switch_flag(&mut self) {
        self.barrier_or_switch_flag = 0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonRoomTrackingState {
    room_index2: u16,
    previous_room_index: u16,
    history: [u16; DUNGEON_ROOM_HISTORY_COUNT],
}

impl Default for DungeonRoomTrackingState {
    fn default() -> Self {
        Self {
            room_index2: 0,
            previous_room_index: 0,
            history: [0xffff; DUNGEON_ROOM_HISTORY_COUNT],
        }
    }
}

impl DungeonRoomTrackingState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut history = [0xffff; DUNGEON_ROOM_HISTORY_COUNT];
        for (index, entry) in history.iter_mut().enumerate() {
            *entry = read_le_u16(ram, DUNGEON_ROOM_HISTORY + index * 2);
        }

        Self {
            room_index2: read_le_u16(ram, DUNGEON_ROOM_INDEX2),
            previous_room_index: read_le_u16(ram, DUNGEON_ROOM_INDEX_PREV),
            history,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNGEON_ROOM_INDEX2, self.room_index2);
        write_le_u16(ram, DUNGEON_ROOM_INDEX_PREV, self.previous_room_index);
        for (index, entry) in self.history.iter().enumerate() {
            write_le_u16(ram, DUNGEON_ROOM_HISTORY + index * 2, *entry);
        }
    }

    pub(crate) fn room_index2(&self) -> u8 {
        self.room_index2 as u8
    }

    pub(crate) fn room_index2_word(&self) -> u16 {
        self.room_index2
    }

    pub(crate) fn previous_room_index(&self) -> usize {
        usize::from(self.previous_room_index)
    }

    pub(crate) fn previous_room_index_word(&self) -> u16 {
        self.previous_room_index
    }

    pub(crate) fn room_history_entry(&self, index: usize) -> u16 {
        self.history.get(index).copied().unwrap_or(0xffff)
    }

    fn set_room_index2(&mut self, value: u8) {
        self.room_index2 = (self.room_index2 & 0xff00) | u16::from(value);
    }

    fn set_room_index2_word(&mut self, value: u16) {
        self.room_index2 = value;
    }

    fn set_room_index_prev(&mut self, value: u8) {
        self.previous_room_index = (self.previous_room_index & 0xff00) | u16::from(value);
    }

    fn set_previous_room_index_word(&mut self, value: u16) {
        self.previous_room_index = value;
    }

    fn set_room_history_entry(&mut self, index: usize, value: u16) {
        if let Some(entry) = self.history.get_mut(index) {
            *entry = value;
        }
    }

    fn reset_room_history(&mut self) {
        self.history.fill(0xffff);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonMovingFloorState {
    y_velocity: u16,
    x_velocity: u16,
    x_offset: u16,
    y_offset: u16,
    move_flags: u16,
}

impl DungeonMovingFloorState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            y_velocity: read_le_u16(ram, DUNGEON_FLOOR_Y_VELOCITY),
            x_velocity: read_le_u16(ram, DUNGEON_FLOOR_X_VELOCITY),
            x_offset: read_le_u16(ram, DUNG_FLOOR_X_OFFS),
            y_offset: read_le_u16(ram, DUNG_FLOOR_Y_OFFS),
            move_flags: read_le_u16(ram, DUNG_FLOOR_MOVE_FLAGS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNGEON_FLOOR_Y_VELOCITY, self.y_velocity);
        write_le_u16(ram, DUNGEON_FLOOR_X_VELOCITY, self.x_velocity);
        write_le_u16(ram, DUNG_FLOOR_X_OFFS, self.x_offset);
        write_le_u16(ram, DUNG_FLOOR_Y_OFFS, self.y_offset);
        write_le_u16(ram, DUNG_FLOOR_MOVE_FLAGS, self.move_flags);
    }

    pub(crate) fn floor_y_velocity(&self) -> u16 {
        self.y_velocity
    }

    pub(crate) fn floor_y_velocity_low(&self) -> u8 {
        self.y_velocity as u8
    }

    pub(crate) fn floor_x_velocity(&self) -> u16 {
        self.x_velocity
    }

    pub(crate) fn floor_x_velocity_low(&self) -> u8 {
        self.x_velocity as u8
    }

    pub(crate) fn floor_x_offset(&self) -> u16 {
        self.x_offset
    }

    pub(crate) fn floor_y_offset(&self) -> u16 {
        self.y_offset
    }

    pub(crate) fn floor_move_flags(&self) -> u16 {
        self.move_flags
    }

    fn set_floor_y_velocity_high(&mut self, value: u8) {
        self.y_velocity = (self.y_velocity & 0x00ff) | (u16::from(value) << 8);
    }

    fn set_floor_y_velocity(&mut self, value: u16) {
        self.y_velocity = value;
    }

    fn set_floor_x_velocity(&mut self, value: u16) {
        self.x_velocity = value;
    }

    fn clear_floor_velocity(&mut self) {
        self.x_velocity = 0;
        self.y_velocity = 0;
    }

    fn set_floor_x_offset(&mut self, value: u16) {
        self.x_offset = value;
    }

    fn set_floor_y_offset(&mut self, value: u16) {
        self.y_offset = value;
    }

    fn set_floor_y_offset_low(&mut self, value: u8) {
        self.y_offset = (self.y_offset & 0xff00) | u16::from(value);
    }

    fn set_floor_offsets(&mut self, x: u16, y: u16) {
        self.x_offset = x;
        self.y_offset = y;
    }

    fn add_floor_x_offset(&mut self, delta: u16) -> u16 {
        self.x_offset = self.x_offset.wrapping_add(delta);
        self.x_offset
    }

    fn sub_floor_x_offset(&mut self, delta: u16) -> u16 {
        self.x_offset = self.x_offset.wrapping_sub(delta);
        self.x_offset
    }

    fn add_floor_y_offset(&mut self, delta: u16) -> u16 {
        self.y_offset = self.y_offset.wrapping_add(delta);
        self.y_offset
    }

    fn sub_floor_y_offset(&mut self, delta: u16) -> u16 {
        self.y_offset = self.y_offset.wrapping_sub(delta);
        self.y_offset
    }

    fn clear_floor_offsets(&mut self) {
        self.set_floor_offsets(0, 0);
    }

    fn clear_floor_move_flags(&mut self) {
        self.move_flags = 0;
    }

    fn set_floor_move_flags(&mut self, value: u16) {
        self.move_flags = value;
    }

    fn increment_floor_move_flags(&mut self) {
        self.move_flags = self.move_flags.wrapping_add(1);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonStairMovementState {
    current_floor_word: u16,
    cached_floor: u8,
    staircase_index: u16,
    move_counter: u8,
    current_plane: u8,
    lower_level_status: u8,
    tilemap_pos_x2: u16,
    in_room_kind: u16,
    countdown: u8,
}

impl DungeonStairMovementState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            current_floor_word: read_le_u16(ram, DUNG_CUR_FLOOR),
            cached_floor: ram.get(DUNG_CUR_FLOOR_CACHED).copied().unwrap_or(0),
            staircase_index: read_le_u16(ram, WHICH_STAIRCASE_INDEX),
            move_counter: ram.get(STAIRCASE_MOVE_COUNTER).copied().unwrap_or(0),
            current_plane: ram.get(CUR_STAIRCASE_PLANE).copied().unwrap_or(0),
            lower_level_status: ram.get(STAIRCASE_LOWER_LEVEL_STATUS).copied().unwrap_or(0),
            tilemap_pos_x2: read_le_u16(ram, STAIRCASE_TILEMAP_POS_X2),
            in_room_kind: read_le_u16(ram, KIND_OF_IN_ROOM_STAIRCASE),
            countdown: ram
                .get(COUNTDOWN_TIMER_FOR_STAIRCASES)
                .copied()
                .unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNG_CUR_FLOOR, self.current_floor_word);
        ram[DUNG_CUR_FLOOR_CACHED] = self.cached_floor;
        write_le_u16(ram, WHICH_STAIRCASE_INDEX, self.staircase_index);
        ram[STAIRCASE_MOVE_COUNTER] = self.move_counter;
        ram[CUR_STAIRCASE_PLANE] = self.current_plane;
        ram[STAIRCASE_LOWER_LEVEL_STATUS] = self.lower_level_status;
        write_le_u16(ram, STAIRCASE_TILEMAP_POS_X2, self.tilemap_pos_x2);
        write_le_u16(ram, KIND_OF_IN_ROOM_STAIRCASE, self.in_room_kind);
        ram[COUNTDOWN_TIMER_FOR_STAIRCASES] = self.countdown;
    }

    pub(crate) fn current_floor(&self) -> u8 {
        self.current_floor_word as u8
    }

    pub(crate) fn current_floor_word(&self) -> u16 {
        self.current_floor_word
    }

    pub(crate) fn cached_floor(&self) -> u8 {
        self.cached_floor
    }

    pub(crate) fn current_staircase_plane(&self) -> u8 {
        self.current_plane
    }

    pub(crate) fn staircase_lower_level_status(&self) -> u8 {
        self.lower_level_status
    }

    pub(crate) fn staircase_index(&self) -> u8 {
        self.staircase_index as u8
    }

    pub(crate) fn staircase_index_slot(&self) -> usize {
        usize::from(self.staircase_index() & 3)
    }

    pub(crate) fn staircase_index_has_vertical_bit(&self) -> bool {
        self.staircase_index() & 4 != 0
    }

    pub(crate) fn staircase_move_counter(&self) -> u8 {
        self.move_counter
    }

    pub(crate) fn kind_of_in_room_staircase(&self) -> u8 {
        self.in_room_kind as u8
    }

    pub(crate) fn staircase_tilemap_pos_x2(&self) -> u16 {
        self.tilemap_pos_x2
    }

    pub(crate) fn staircase_countdown(&self) -> u8 {
        self.countdown
    }

    fn set_current_floor(&mut self, value: u8) {
        self.current_floor_word = (self.current_floor_word & 0xff00) | u16::from(value);
    }

    fn decrement_current_floor(&mut self) -> u8 {
        let next = self.current_floor().wrapping_sub(1);
        self.set_current_floor(next);
        next
    }

    fn increment_current_floor(&mut self) -> u8 {
        let next = self.current_floor().wrapping_add(1);
        self.set_current_floor(next);
        next
    }

    fn cache_current_floor(&mut self) {
        self.cached_floor = self.current_floor();
    }

    fn restore_cached_floor(&mut self) {
        self.set_current_floor(self.cached_floor);
    }

    fn set_staircase_tilemap_pos_x2(&mut self, value: u16) {
        self.tilemap_pos_x2 = value;
    }

    fn set_current_staircase_plane(&mut self, value: u8) {
        self.current_plane = value;
    }

    fn set_staircase_lower_level_status(&mut self, value: u8) {
        self.lower_level_status = value;
    }

    fn set_staircase_countdown(&mut self, value: u8) {
        self.countdown = value;
    }

    fn decrement_staircase_countdown_clamped(&mut self) -> u8 {
        let value = self.countdown.wrapping_sub(1);
        self.countdown = if (value as i8).is_negative() {
            0
        } else {
            value
        };
        self.countdown
    }

    fn decrement_staircase_countdown_underflowed(&mut self) -> bool {
        let value = self.countdown.wrapping_sub(1);
        let underflowed = (value as i8).is_negative();
        self.countdown = if underflowed { 0 } else { value };
        underflowed
    }

    fn set_staircase_index(&mut self, value: u8) {
        self.staircase_index = (self.staircase_index & 0xff00) | u16::from(value);
    }

    fn set_staircase_index_high(&mut self, value: u8) {
        self.staircase_index = (self.staircase_index & 0x00ff) | (u16::from(value) << 8);
    }

    fn set_staircase_move_counter(&mut self, value: u8) {
        self.move_counter = value;
    }

    fn decrement_staircase_move_counter(&mut self) -> u8 {
        self.move_counter = self.move_counter.wrapping_sub(1);
        self.move_counter
    }

    fn set_kind_of_in_room_staircase_word(&mut self, value: u16) {
        self.in_room_kind = value;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonMovableBlockState {
    records: Vec<[u16; 2]>,
}

impl Default for DungeonMovableBlockState {
    fn default() -> Self {
        Self {
            records: vec![[0; 2]; DUNGEON_MOVABLE_BLOCK_RECORD_COUNT],
        }
    }
}

impl DungeonMovableBlockState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut records = vec![[0; 2]; DUNGEON_MOVABLE_BLOCK_RECORD_COUNT];
        for (index, record) in records.iter_mut().enumerate() {
            let base = MOVABLE_BLOCK_DATAS + index * 4;
            record[0] = read_le_u16(ram, base);
            record[1] = read_le_u16(ram, base + 2);
        }
        Self { records }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (index, record) in self.records.iter().enumerate() {
            let base = MOVABLE_BLOCK_DATAS + index * 4;
            write_le_u16(ram, base, record[0]);
            write_le_u16(ram, base + 2, record[1]);
        }
    }

    pub(crate) fn movable_block_room_for_offset(&self, offset: usize) -> u16 {
        self.records
            .get(offset / 4)
            .map(|record| record[0])
            .unwrap_or(0)
    }

    pub(crate) fn movable_block_tilemap_for_offset(&self, offset: usize) -> u16 {
        self.records
            .get(offset / 4)
            .map(|record| record[1])
            .unwrap_or(0)
    }

    fn set_movable_block_record(&mut self, index: usize, room: u16, tilemap: u16) {
        if let Some(record) = self.records.get_mut(index) {
            *record = [room, tilemap];
        }
    }

    fn copy_records_from_bytes(&mut self, data: &[u8]) {
        for (index, record) in self.records.iter_mut().enumerate() {
            let base = index * 4;
            if base + 3 >= data.len() {
                break;
            }
            record[0] = read_le_u16(data, base);
            record[1] = read_le_u16(data, base + 2);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonRoomRuntimeState {
    dungeon_music_type_flag: u8,
    room_tag_skip_count: u8,
    landing_class: u8,
    room_index_x3: u16,
    reserved_gfx_config_word: u16,
}

impl DungeonRoomRuntimeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            dungeon_music_type_flag: ram.get(FLAG_WHICH_MUSIC_TYPE_DUNGEON).copied().unwrap_or(0),
            room_tag_skip_count: ram.get(FLAG_SKIP_CALL_TAG_ROUTINES).copied().unwrap_or(0),
            landing_class: ram.get(DUNG_TRANSITION_LANDING_CLASS).copied().unwrap_or(0),
            room_index_x3: read_le_u16(ram, DUNG_INDEX_X3),
            reserved_gfx_config_word: read_le_u16(ram, RESERVED_GFX_CONFIG_WORD),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[FLAG_WHICH_MUSIC_TYPE_DUNGEON] = self.dungeon_music_type_flag;
        ram[FLAG_SKIP_CALL_TAG_ROUTINES] = self.room_tag_skip_count;
        ram[DUNG_TRANSITION_LANDING_CLASS] = self.landing_class;
        write_le_u16(ram, DUNG_INDEX_X3, self.room_index_x3);
        write_le_u16(ram, RESERVED_GFX_CONFIG_WORD, self.reserved_gfx_config_word);
    }

    pub(crate) fn dungeon_music_type_flag(&self) -> u8 {
        self.dungeon_music_type_flag
    }

    pub(crate) fn landing_class(&self) -> u8 {
        self.landing_class
    }

    pub(crate) fn landing_class_is_pit(&self) -> bool {
        // C (Dungeon_IntraRoomTrans_State5): `byte_7E004E == 2 || == 4`, NOT a
        // bitmask — `& 2 != 0` wrongly matches 3/6/7 and misses 4.
        self.landing_class == 2 || self.landing_class == 4
    }

    pub(crate) fn should_run_room_tags(&self) -> bool {
        self.room_tag_skip_count == 0
    }

    fn clear_dungeon_music_type_flag(&mut self) {
        self.dungeon_music_type_flag = 0;
    }

    fn set_dungeon_music_type_flag(&mut self, value: u8) {
        self.dungeon_music_type_flag = value;
    }

    fn set_room_index_x3(&mut self, value: u16) {
        self.room_index_x3 = value;
    }

    fn clear_reserved_gfx_config(&mut self) {
        self.reserved_gfx_config_word = 0;
    }

    fn skip_room_tags_once(&mut self) {
        self.room_tag_skip_count = self.room_tag_skip_count.wrapping_add(1);
    }

    fn clear_room_tag_skip(&mut self) {
        self.room_tag_skip_count = 0;
    }

    fn set_landing_class(&mut self, value: u8) {
        self.landing_class = value;
    }

    fn clear_landing_class(&mut self) {
        self.landing_class = 0;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonRoomDoorSetupState {
    adjacent_door_flags: u16,
    adjacent_doors: [u16; DUNGEON_ADJACENT_DOOR_COUNT],
    exit_door_count_x2: u16,
    exit_door_addresses: [u16; DUNGEON_EXIT_DOOR_COUNT],
    invisible_door_marker: u16,
    active_room_load_ptr: u16,
    active_room_load_ptr_bank: u8,
    width_road_address: u16,
    reset_xy_check_flags: u16,
}

impl DungeonRoomDoorSetupState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut adjacent_doors = [0; DUNGEON_ADJACENT_DOOR_COUNT];
        for (index, door) in adjacent_doors.iter_mut().enumerate() {
            *door = read_le_u16(ram, ADJACENT_DOORS + index * 2);
        }

        let mut exit_door_addresses = [0; DUNGEON_EXIT_DOOR_COUNT];
        for (index, address) in exit_door_addresses.iter_mut().enumerate() {
            *address = read_le_u16(ram, DUNG_EXIT_DOOR_ADDRESSES + index * 2);
        }

        Self {
            adjacent_door_flags: read_le_u16(ram, ADJACENT_DOORS_FLAGS),
            adjacent_doors,
            exit_door_count_x2: read_le_u16(ram, DUNG_EXIT_DOOR_COUNT),
            exit_door_addresses,
            invisible_door_marker: read_le_u16(ram, INVISIBLE_DOOR_DIR_AND_INDEX_X2),
            active_room_load_ptr: read_le_u16(ram, DUNG_LOAD_PTR),
            active_room_load_ptr_bank: ram.get(DUNG_LOAD_PTR_BANK).copied().unwrap_or(0),
            width_road_address: read_le_u16(ram, DUNG_WIDTH_ROAD_ADDRESS),
            reset_xy_check_flags: read_le_u16(ram, RESET_XY_CHECK_FLAGS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, ADJACENT_DOORS_FLAGS, self.adjacent_door_flags);
        for (index, &door) in self.adjacent_doors.iter().enumerate() {
            write_le_u16(ram, ADJACENT_DOORS + index * 2, door);
        }
        write_le_u16(ram, DUNG_EXIT_DOOR_COUNT, self.exit_door_count_x2);
        for (index, &address) in self.exit_door_addresses.iter().enumerate() {
            write_le_u16(ram, DUNG_EXIT_DOOR_ADDRESSES + index * 2, address);
        }
        write_le_u16(
            ram,
            INVISIBLE_DOOR_DIR_AND_INDEX_X2,
            self.invisible_door_marker,
        );
        write_le_u16(ram, DUNG_LOAD_PTR, self.active_room_load_ptr);
        ram[DUNG_LOAD_PTR_BANK] = self.active_room_load_ptr_bank;
        write_le_u16(ram, DUNG_WIDTH_ROAD_ADDRESS, self.width_road_address);
        write_le_u16(ram, RESET_XY_CHECK_FLAGS, self.reset_xy_check_flags);
    }

    pub(crate) fn adjacent_door_flags(&self) -> u16 {
        self.adjacent_door_flags
    }

    pub(crate) fn adjacent_door(&self, index: usize) -> u16 {
        self.adjacent_doors.get(index).copied().unwrap_or(0xffff)
    }

    pub(crate) fn has_exit_door_address(&self, address: u16) -> bool {
        self.exit_door_addresses
            .iter()
            .any(|&existing| existing == address)
    }

    pub(crate) fn invisible_door_marker(&self) -> u16 {
        self.invisible_door_marker
    }

    pub(crate) fn active_room_load_ptr(&self) -> u16 {
        self.active_room_load_ptr
    }

    pub(crate) fn active_room_load_ptr_bank(&self) -> u8 {
        self.active_room_load_ptr_bank
    }

    pub(crate) fn width_road_address(&self) -> u16 {
        self.width_road_address
    }

    pub(crate) fn reset_xy_check_flags(&self) -> u16 {
        self.reset_xy_check_flags
    }

    fn clear_invisible_door_marker(&mut self) {
        self.invisible_door_marker = 0xffff;
    }

    fn set_invisible_door_marker(&mut self, slot: usize, direction: u16) {
        self.invisible_door_marker = (((slot as u16) << 8) | direction) * 2;
    }

    fn clear_exit_door_count_and_flags(&mut self) {
        self.exit_door_count_x2 = 0;
        self.exit_door_addresses = [0; DUNGEON_EXIT_DOOR_COUNT];
    }

    fn append_exit_door_address(&mut self, address: u16) -> usize {
        let index = usize::from(self.exit_door_count_x2 >> 1);
        if let Some(slot) = self.exit_door_addresses.get_mut(index) {
            *slot = address;
        }
        self.exit_door_count_x2 = self.exit_door_count_x2.wrapping_add(2);
        index
    }

    fn set_active_room_load_ptr(&mut self, value: u16) {
        self.active_room_load_ptr = value;
    }

    fn set_active_room_load_ptr_bank(&mut self, value: u8) {
        self.active_room_load_ptr_bank = value;
    }

    fn add_reset_xy_check_flags(&mut self, value: u16) -> u16 {
        self.reset_xy_check_flags |= value;
        self.reset_xy_check_flags
    }

    fn set_adjacent_door_flags(&mut self, value: u16) {
        self.adjacent_door_flags = value;
    }

    fn mark_adjacent_door_flag(&mut self, index: usize) -> u16 {
        self.adjacent_door_flags |= 0x8000u16 >> (index & 15);
        self.adjacent_door_flags
    }

    fn set_adjacent_door(&mut self, index: usize, value: u16) {
        if let Some(door) = self.adjacent_doors.get_mut(index) {
            *door = value;
        }
    }

    fn mark_no_adjacent_doors(&mut self) {
        self.set_adjacent_door(0, 0xffff);
    }

    fn load_adjacent_doors_from_room_info(&mut self, door_info: &[u8]) {
        for index in 0..DUNGEON_ADJACENT_DOOR_COUNT {
            let door = door_info_word(door_info, index * 2);
            self.set_adjacent_door(index, door);
            if door == 0xffff {
                break;
            }
            if (door & 0xff00) == 0x4000 || (door & 0xff00) < 0x0200 {
                self.mark_adjacent_door_flag(index);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonRoomParserState {
    star_switch_count_x2: u16,
    star_switch_tilemap_positions: [u16; DUNGEON_ROOM_PARSER_SMALL_TABLE_COUNT],
    toggle_floor_count_x2: u16,
    toggle_palace_count_x2: u16,
    toggle_floor_positions: [u16; DUNGEON_ROOM_TOGGLE_SLOT_COUNT],
    toggle_palace_positions: [u16; DUNGEON_ROOM_TOGGLE_SLOT_COUNT],
    floor_1_filler_tiles: u16,
    floor_2_filler_tiles: u16,
    room_layout_and_starting_quadrant: u16,
    pot_reveal_masks: Vec<u16>,
    tile_attributes: Vec<u8>,
}

impl Default for DungeonRoomParserState {
    fn default() -> Self {
        Self {
            star_switch_count_x2: 0,
            star_switch_tilemap_positions: [0; DUNGEON_ROOM_PARSER_SMALL_TABLE_COUNT],
            toggle_floor_count_x2: 0,
            toggle_palace_count_x2: 0,
            toggle_floor_positions: [0; DUNGEON_ROOM_TOGGLE_SLOT_COUNT],
            toggle_palace_positions: [0; DUNGEON_ROOM_TOGGLE_SLOT_COUNT],
            floor_1_filler_tiles: 0,
            floor_2_filler_tiles: 0,
            room_layout_and_starting_quadrant: 0,
            pot_reveal_masks: vec![0; DUNGEON_POT_REVEAL_ROOM_COUNT],
            tile_attributes: vec![0; 0x200],
        }
    }
}

impl DungeonRoomParserState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut star_switch_tilemap_positions = [0; DUNGEON_ROOM_PARSER_SMALL_TABLE_COUNT];
        for (index, pos) in star_switch_tilemap_positions.iter_mut().enumerate() {
            *pos = read_le_u16(ram, STAR_SHAPED_SWITCHES_TILE_LOCAL + index * 2);
        }

        let mut toggle_floor_positions = [0; DUNGEON_ROOM_TOGGLE_SLOT_COUNT];
        let mut toggle_palace_positions = [0; DUNGEON_ROOM_TOGGLE_SLOT_COUNT];
        for index in 0..DUNGEON_ROOM_TOGGLE_SLOT_COUNT {
            toggle_floor_positions[index] = read_le_u16(ram, DUNG_TOGGLE_FLOOR_POS + index * 2);
            toggle_palace_positions[index] = read_le_u16(ram, DUNG_TOGGLE_PALACE_POS + index * 2);
        }

        let mut pot_reveal_masks = vec![0; DUNGEON_POT_REVEAL_ROOM_COUNT];
        for (room, mask) in pot_reveal_masks.iter_mut().enumerate() {
            *mask = read_le_u16(ram, POTS_REVEALED_IN_ROOM_DUNGEON_LOCAL + room * 2);
        }

        // ATTRIBUTES_FOR_TILE owns exactly 0x200 bytes (0xfe00..0x10000). A 0x400 array
        // overran the load/projection into 0x10000-0x101ff (BG_CHAR_BUFFER), re-stamping a
        // stale copy over the star-tile graphics written by Dungeon_RestoreStarTileChr.
        let mut tile_attributes = vec![0; 0x200];
        let available = ram.len().saturating_sub(ATTRIBUTES_FOR_TILE_PLAYER);
        let len = tile_attributes.len().min(available);
        tile_attributes[..len]
            .copy_from_slice(&ram[ATTRIBUTES_FOR_TILE_PLAYER..ATTRIBUTES_FOR_TILE_PLAYER + len]);

        Self {
            star_switch_count_x2: read_le_u16(ram, DUNG_NUM_STAR_SHAPED_SWITCHES_LOCAL),
            star_switch_tilemap_positions,
            toggle_floor_count_x2: read_le_u16(ram, DUNG_NUM_TOGGLE_FLOOR),
            toggle_palace_count_x2: read_le_u16(ram, DUNG_NUM_TOGGLE_PALACE),
            toggle_floor_positions,
            toggle_palace_positions,
            floor_1_filler_tiles: read_le_u16(ram, FLOOR_1_FILLER_TILES),
            floor_2_filler_tiles: read_le_u16(ram, FLOOR_2_FILLER_TILES),
            room_layout_and_starting_quadrant: read_le_u16(ram, DUNG_LAYOUT_AND_STARTING_QUADRANT),
            pot_reveal_masks,
            tile_attributes,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(
            ram,
            DUNG_NUM_STAR_SHAPED_SWITCHES_LOCAL,
            self.star_switch_count_x2,
        );
        for (index, &pos) in self.star_switch_tilemap_positions.iter().enumerate() {
            write_le_u16(ram, STAR_SHAPED_SWITCHES_TILE_LOCAL + index * 2, pos);
        }
        write_le_u16(ram, DUNG_NUM_TOGGLE_FLOOR, self.toggle_floor_count_x2);
        write_le_u16(ram, DUNG_NUM_TOGGLE_PALACE, self.toggle_palace_count_x2);
        for (index, &pos) in self.toggle_floor_positions.iter().enumerate() {
            write_le_u16(ram, DUNG_TOGGLE_FLOOR_POS + index * 2, pos);
        }
        for (index, &pos) in self.toggle_palace_positions.iter().enumerate() {
            write_le_u16(ram, DUNG_TOGGLE_PALACE_POS + index * 2, pos);
        }
        write_le_u16(ram, FLOOR_1_FILLER_TILES, self.floor_1_filler_tiles);
        write_le_u16(ram, FLOOR_2_FILLER_TILES, self.floor_2_filler_tiles);
        write_le_u16(
            ram,
            DUNG_LAYOUT_AND_STARTING_QUADRANT,
            self.room_layout_and_starting_quadrant,
        );
        for (room, &mask) in self.pot_reveal_masks.iter().enumerate() {
            write_le_u16(ram, POTS_REVEALED_IN_ROOM_DUNGEON_LOCAL + room * 2, mask);
        }
        let len = self
            .tile_attributes
            .len()
            .min(ram.len().saturating_sub(ATTRIBUTES_FOR_TILE_PLAYER));
        ram[ATTRIBUTES_FOR_TILE_PLAYER..ATTRIBUTES_FOR_TILE_PLAYER + len]
            .copy_from_slice(&self.tile_attributes[..len]);
    }

    pub(crate) fn pots_revealed_in_room(&self, room: usize) -> u16 {
        self.pot_reveal_masks.get(room).copied().unwrap_or(0)
    }

    fn clear_pot_reveal_masks(&mut self) {
        self.pot_reveal_masks.fill(0);
    }

    pub(crate) fn toggle_floor_count_x2(&self) -> u16 {
        self.toggle_floor_count_x2
    }

    pub(crate) fn toggle_palace_count_x2(&self) -> u16 {
        self.toggle_palace_count_x2
    }

    pub(crate) fn toggle_floor_pos(&self, index: usize) -> u16 {
        self.toggle_floor_positions.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn toggle_palace_pos(&self, index: usize) -> u16 {
        self.toggle_palace_positions
            .get(index)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn star_switch_count_x2(&self) -> u16 {
        self.star_switch_count_x2
    }

    pub(crate) fn star_switch_tilemap_pos(&self, offset_x2: usize) -> u16 {
        self.star_switch_tilemap_positions
            .get(offset_x2 >> 1)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn floor_1_filler_tile_source(&self) -> usize {
        usize::from(self.floor_1_filler_tiles)
    }

    pub(crate) fn floor_2_filler_tile_source(&self) -> usize {
        usize::from(self.floor_2_filler_tiles)
    }

    fn append_star_switch_tile(&mut self, tilemap_pos: u16) -> usize {
        let index = usize::from(self.star_switch_count_x2) >> 1;
        self.star_switch_count_x2 = self.star_switch_count_x2.wrapping_add(2);
        if let Some(pos) = self.star_switch_tilemap_positions.get_mut(index) {
            *pos = tilemap_pos;
        }
        index
    }

    fn mark_pot_revealed_in_room(&mut self, room: usize, mask: u16) -> u16 {
        let Some(revealed) = self.pot_reveal_masks.get_mut(room) else {
            return 0;
        };
        *revealed |= mask;
        *revealed
    }

    fn append_toggle_palace_pos(&mut self, pos: u16) -> usize {
        let index = usize::from(self.toggle_palace_count_x2 >> 1);
        if let Some(slot) = self.toggle_palace_positions.get_mut(index) {
            *slot = pos;
        }
        self.toggle_palace_count_x2 = self.toggle_palace_count_x2.wrapping_add(2);
        index
    }

    fn append_toggle_floor_pos(&mut self, pos: u16) -> usize {
        let index = usize::from(self.toggle_floor_count_x2 >> 1);
        if let Some(slot) = self.toggle_floor_positions.get_mut(index) {
            *slot = pos;
        }
        self.toggle_floor_count_x2 = self.toggle_floor_count_x2.wrapping_add(2);
        index
    }

    fn copy_custom_tile_attrs(&mut self, attrs: &[u8]) {
        self.tile_attributes[0x140..0x1c0].copy_from_slice(attrs);
    }

    fn copy_default_tile_attrs_tail(&mut self, attrs: &[u8]) {
        self.tile_attributes[0x1c0..0x200].copy_from_slice(attrs);
    }

    fn copy_default_tile_attrs_head(&mut self, data: &[u8]) {
        self.tile_attributes[..0x140].copy_from_slice(&data[..0x140]);
    }

    fn set_floor_1_filler_high(&mut self, value: u8) {
        self.floor_1_filler_tiles = (self.floor_1_filler_tiles & 0x00ff) | (u16::from(value) << 8);
    }

    fn set_floor_2_filler_high(&mut self, value: u8) {
        self.floor_2_filler_tiles = (self.floor_2_filler_tiles & 0x00ff) | (u16::from(value) << 8);
    }

    fn set_floor_1_filler_low(&mut self, value: u8) {
        self.floor_1_filler_tiles = (self.floor_1_filler_tiles & 0xff00) | u16::from(value);
    }

    fn set_floor_2_filler_low(&mut self, value: u8) {
        self.floor_2_filler_tiles = (self.floor_2_filler_tiles & 0xff00) | u16::from(value);
    }

    fn set_room_layout_and_starting_quadrant(&mut self, value: u16) {
        self.room_layout_and_starting_quadrant = value;
    }

    pub(crate) fn room_layout_and_starting_quadrant(&self) -> u16 {
        self.room_layout_and_starting_quadrant
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonRoomEffectsState {
    blast_wall_x_open: u8,
    blast_wall_y_open: u8,
    crush_wall_progress: u16,
    blast_wall_door_index_x2: u16,
    moving_wall_dot_pointer: u8,
    moving_wall_write_point: u16,
    moving_wall_torch_blink_phase: u8,
    moving_wall_torch_update_flag: u8,
    fixed_color_plusminus: u8,
    trap_trigger_latch: u8,
    bomb_trap_activation: u8,
    blast_wall_message_state: u8,
    blast_wall_message_x: u16,
    blast_wall_message_y: u16,
    blast_wall_message_direction: u16,
    moving_wall_replacement_buffer: Vec<u16>,
}

impl Default for DungeonRoomEffectsState {
    fn default() -> Self {
        Self {
            blast_wall_x_open: 0,
            blast_wall_y_open: 0,
            crush_wall_progress: 0,
            blast_wall_door_index_x2: 0,
            moving_wall_dot_pointer: 0,
            moving_wall_write_point: 0,
            moving_wall_torch_blink_phase: 0,
            moving_wall_torch_update_flag: 0,
            fixed_color_plusminus: 0,
            trap_trigger_latch: 0,
            bomb_trap_activation: 0,
            blast_wall_message_state: 0,
            blast_wall_message_x: 0,
            blast_wall_message_y: 0,
            blast_wall_message_direction: 0,
            moving_wall_replacement_buffer: vec![0; MOVING_WALL_REPLACEMENT_WORDS],
        }
    }
}

impl DungeonRoomEffectsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut moving_wall_replacement_buffer = vec![0; MOVING_WALL_REPLACEMENT_WORDS];
        for (index, value) in moving_wall_replacement_buffer.iter_mut().enumerate() {
            *value = read_le_u16(ram, MOVING_WALL_REPLACEMENT_BUFFER + index * 2);
        }
        Self {
            blast_wall_x_open: ram.get(DUNG_BLASTWALL_FLAG_X).copied().unwrap_or(0),
            blast_wall_y_open: ram.get(DUNG_BLASTWALL_FLAG_Y).copied().unwrap_or(0),
            crush_wall_progress: read_le_u16(ram, CRUSH_WALL_PROGRESS),
            blast_wall_door_index_x2: read_le_u16(ram, CRUSH_WALL_DOOR_INDEX_X2),
            moving_wall_dot_pointer: ram.get(MOVING_WALL_DOT_POINTER).copied().unwrap_or(0),
            moving_wall_write_point: read_le_u16(ram, MOVING_WALL_WRITE_POINT),
            moving_wall_torch_blink_phase: ram
                .get(MOVING_WALL_TORCH_BLINK_PHASE)
                .copied()
                .unwrap_or(0),
            moving_wall_torch_update_flag: ram
                .get(MOVING_WALL_TORCH_UPDATE_FLAG)
                .copied()
                .unwrap_or(0),
            fixed_color_plusminus: ram
                .get(OVERWORLD_FIXED_COLOR_PLUSMINUS)
                .copied()
                .unwrap_or(0),
            trap_trigger_latch: ram.get(DUNGEON_TRAP_TRIGGER_LATCH).copied().unwrap_or(0),
            bomb_trap_activation: ram.get(ACTIVATE_BOMB_TRAP_OVERLORD).copied().unwrap_or(0),
            blast_wall_message_state: ram.get(MESSAGING_BUF_DUNGEON).copied().unwrap_or(0),
            blast_wall_message_x: read_le_u16(ram, MESSAGING_BUF_DUNGEON + 0x1a),
            blast_wall_message_y: read_le_u16(ram, MESSAGING_BUF_DUNGEON + 0x18),
            blast_wall_message_direction: read_le_u16(ram, MESSAGING_BUF_DUNGEON + 0x1c),
            moving_wall_replacement_buffer,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DUNG_BLASTWALL_FLAG_X] = self.blast_wall_x_open;
        ram[DUNG_BLASTWALL_FLAG_Y] = self.blast_wall_y_open;
        write_le_u16(ram, CRUSH_WALL_PROGRESS, self.crush_wall_progress);
        write_le_u16(ram, CRUSH_WALL_DOOR_INDEX_X2, self.blast_wall_door_index_x2);
        ram[MOVING_WALL_DOT_POINTER] = self.moving_wall_dot_pointer;
        write_le_u16(ram, MOVING_WALL_WRITE_POINT, self.moving_wall_write_point);
        ram[MOVING_WALL_TORCH_BLINK_PHASE] = self.moving_wall_torch_blink_phase;
        ram[MOVING_WALL_TORCH_UPDATE_FLAG] = self.moving_wall_torch_update_flag;
        ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = self.fixed_color_plusminus;
        ram[DUNGEON_TRAP_TRIGGER_LATCH] = self.trap_trigger_latch;
        ram[ACTIVATE_BOMB_TRAP_OVERLORD] = self.bomb_trap_activation;
        // MESSAGING_BUF_DUNGEON (0x10000) is SNES byte-reused: it is the BG-char / message
        // gfx-staging buffer normally, and only holds the blast-wall message (state/x/y/dir)
        // while a blast wall is open. Projecting these fields unconditionally clobbered the
        // regenerated gfx buffer with stale values (page 0x10000 transient). Only project them
        // when a blast wall is actually open, matching C (which writes 0x10000 raw, only then).
        if self.blast_wall_x_open != 0 || self.blast_wall_y_open != 0 {
            ram[MESSAGING_BUF_DUNGEON] = self.blast_wall_message_state;
            write_le_u16(ram, MESSAGING_BUF_DUNGEON + 0x1a, self.blast_wall_message_x);
            write_le_u16(ram, MESSAGING_BUF_DUNGEON + 0x18, self.blast_wall_message_y);
            write_le_u16(
                ram,
                MESSAGING_BUF_DUNGEON + 0x1c,
                self.blast_wall_message_direction,
            );
        }
        for (index, &value) in self.moving_wall_replacement_buffer.iter().enumerate() {
            write_le_u16(ram, MOVING_WALL_REPLACEMENT_BUFFER + index * 2, value);
        }
    }

    pub(crate) fn blast_wall_x_open(&self) -> bool {
        self.blast_wall_x_open != 0
    }

    pub(crate) fn blast_wall_y_open(&self) -> bool {
        self.blast_wall_y_open != 0
    }

    pub(crate) fn crush_wall_progress(&self) -> u16 {
        self.crush_wall_progress
    }

    pub(crate) fn moving_wall_dot_pointer(&self) -> u8 {
        self.moving_wall_dot_pointer
    }

    pub(crate) fn moving_wall_dot_index(&self) -> usize {
        usize::from(self.moving_wall_dot_pointer >> 1) & 7
    }

    pub(crate) fn moving_wall_write_point(&self) -> u16 {
        self.moving_wall_write_point
    }

    pub(crate) fn fixed_color_plusminus(&self) -> u8 {
        self.fixed_color_plusminus
    }

    pub(crate) fn blast_wall_door_index_x2(&self) -> u16 {
        self.blast_wall_door_index_x2
    }

    pub(crate) fn trap_trigger_latch(&self) -> u8 {
        self.trap_trigger_latch
    }

    pub(crate) fn has_bomb_trap_activation(&self) -> bool {
        self.bomb_trap_activation != 0
    }

    pub(crate) fn blast_wall_message_state(&self) -> u8 {
        self.blast_wall_message_state
    }

    fn fill_moving_wall_replacement_buffer(&mut self, value: u16) {
        self.moving_wall_replacement_buffer.fill(value);
    }

    fn set_moving_wall_write_point(&mut self, value: u16) {
        self.moving_wall_write_point = value;
    }

    fn set_moving_wall_dot_pointer(&mut self, value: u8) {
        self.moving_wall_dot_pointer = value;
    }

    fn set_fixed_color_plusminus(&mut self, value: u8) {
        self.fixed_color_plusminus = value;
    }

    fn toggle_moving_wall_torch_blink_phase(&mut self) {
        self.moving_wall_torch_blink_phase ^= 1;
    }

    fn request_moving_wall_torch_update(&mut self) {
        self.moving_wall_torch_update_flag = 0x80;
    }

    fn clear_moving_wall_torch_blink_phase(&mut self) {
        self.moving_wall_torch_blink_phase = 0;
    }

    fn set_blast_wall_door_index_x2(&mut self, value: u16) {
        self.blast_wall_door_index_x2 = value;
    }

    fn set_blast_wall_door_index(&mut self, door: usize) {
        self.set_blast_wall_door_index_x2((door * 2) as u16);
    }

    fn clear_blast_wall_door_index(&mut self) {
        self.set_blast_wall_door_index_x2(0);
    }

    fn mark_blast_wall_x_open(&mut self) {
        self.blast_wall_x_open = 1;
    }

    fn mark_blast_wall_y_open(&mut self) {
        self.blast_wall_y_open = 1;
    }

    fn set_crush_wall_progress(&mut self, value: u16) {
        self.crush_wall_progress = value;
    }

    fn set_crush_wall_progress_low(&mut self, value: u8) {
        self.crush_wall_progress = (self.crush_wall_progress & 0xff00) | u16::from(value);
    }

    fn increment_trap_trigger_latch(&mut self) {
        self.trap_trigger_latch = self.trap_trigger_latch.wrapping_add(1);
    }

    fn mark_trap_trigger_latched(&mut self) {
        self.trap_trigger_latch = 1;
    }

    fn clear_trap_trigger_latch(&mut self) {
        self.trap_trigger_latch = 0;
    }

    fn set_blast_wall_message_direction(&mut self, value: u16) {
        self.blast_wall_message_direction = value;
    }

    fn set_blast_wall_message_position(&mut self, x: u16, y: u16) {
        self.blast_wall_message_x = x;
        self.blast_wall_message_y = y;
    }

    fn set_activate_bomb_trap_overlord(&mut self, value: u8) {
        self.bomb_trap_activation = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonRoomItemState {
    num_chests_x2: u16,
    num_big_key_locks_x2: u16,
    chest_reveal_cursor_x2: u16,
    replacement_tile_destination_x2: u16,
    replacement_tile_source_x2: u16,
    chest_locations: [u16; DUNGEON_CHEST_LOCATION_COUNT],
    replacement_tilemap_quads: [[u16; 4]; DUNGEON_ROOM_ITEM_SLOT_COUNT],
}

impl DungeonRoomItemState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut chest_locations = [0; DUNGEON_CHEST_LOCATION_COUNT];
        for (index, slot) in chest_locations.iter_mut().enumerate() {
            *slot = read_le_u16(ram, DUNG_CHEST_LOCATIONS + index * 2);
        }
        let mut replacement_tilemap_quads = [[0; 4]; DUNGEON_ROOM_ITEM_SLOT_COUNT];
        for index in 0..DUNGEON_ROOM_ITEM_SLOT_COUNT {
            replacement_tilemap_quads[index] = [
                read_le_u16(ram, REPLACEMENT_TILEMAP_UL + index * 2),
                read_le_u16(ram, REPLACEMENT_TILEMAP_LL + index * 2),
                read_le_u16(ram, REPLACEMENT_TILEMAP_UR + index * 2),
                read_le_u16(ram, REPLACEMENT_TILEMAP_LR + index * 2),
            ];
        }
        Self {
            num_chests_x2: read_le_u16(ram, DUNG_NUM_CHESTS_X2),
            num_big_key_locks_x2: read_le_u16(ram, DUNG_NUM_BIGKEY_LOCKS_X2),
            chest_reveal_cursor_x2: read_le_u16(ram, OVERWORLD_MAP_STATE),
            replacement_tile_destination_x2: read_le_u16(ram, DUNG_REPLACEMENT_TILE_DST_POS_X2),
            replacement_tile_source_x2: read_le_u16(ram, DUNG_REPLACEMENT_TILE_SRC_POS_X2),
            chest_locations,
            replacement_tilemap_quads,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNG_NUM_CHESTS_X2, self.num_chests_x2);
        write_le_u16(ram, DUNG_NUM_BIGKEY_LOCKS_X2, self.num_big_key_locks_x2);
        write_le_u16(ram, OVERWORLD_MAP_STATE, self.chest_reveal_cursor_x2);
        write_le_u16(
            ram,
            DUNG_REPLACEMENT_TILE_DST_POS_X2,
            self.replacement_tile_destination_x2,
        );
        write_le_u16(
            ram,
            DUNG_REPLACEMENT_TILE_SRC_POS_X2,
            self.replacement_tile_source_x2,
        );
        for (index, &location) in self.chest_locations.iter().enumerate() {
            write_le_u16(ram, DUNG_CHEST_LOCATIONS + index * 2, location);
        }
        for (index, quad) in self.replacement_tilemap_quads.iter().enumerate() {
            write_le_u16(ram, REPLACEMENT_TILEMAP_UL + index * 2, quad[0]);
            write_le_u16(ram, REPLACEMENT_TILEMAP_LL + index * 2, quad[1]);
            write_le_u16(ram, REPLACEMENT_TILEMAP_UR + index * 2, quad[2]);
            write_le_u16(ram, REPLACEMENT_TILEMAP_LR + index * 2, quad[3]);
        }
    }

    pub(crate) fn num_chests_x2(&self) -> u16 {
        self.num_chests_x2
    }

    pub(crate) fn num_big_key_locks_x2(&self) -> u16 {
        self.num_big_key_locks_x2
    }

    pub(crate) fn chest_reveal_cursor_x2(&self) -> u16 {
        self.chest_reveal_cursor_x2
    }

    pub(crate) fn chest_reveal_cursor_reached_end(&self, cursor_x2: u16) -> bool {
        cursor_x2 == self.num_chests_x2
    }

    pub(crate) fn replacement_tile_destination_x2(&self) -> u16 {
        self.replacement_tile_destination_x2
    }

    pub(crate) fn replacement_tile_source_x2(&self) -> u16 {
        self.replacement_tile_source_x2
    }

    pub(crate) fn replacement_tile_source_pos(&self) -> u16 {
        self.replacement_tile_source_x2 >> 1
    }

    pub(crate) fn chest_location(&self, index: usize) -> u16 {
        self.chest_locations.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn chest_location_for_cursor(&self, cursor_x2: u16) -> u16 {
        self.chest_location(usize::from(cursor_x2 >> 1))
    }

    pub(crate) fn chest_location_for_offset_x2(&self, offset_x2: usize) -> u16 {
        self.chest_location(offset_x2 >> 1)
    }

    pub(crate) fn replacement_tilemap_quad(&self, index: usize) -> [u16; 4] {
        self.replacement_tilemap_quads
            .get(index)
            .copied()
            .unwrap_or([0; 4])
    }

    fn set_num_chests_x2(&mut self, value: u16) {
        self.num_chests_x2 = value;
    }

    fn set_num_big_key_locks_x2(&mut self, value: u16) {
        self.num_big_key_locks_x2 = value;
    }

    fn append_chest_location_and_sync_big_key_count(&mut self, value: u16) -> usize {
        let index = self.advance_chest_and_big_key_counts();
        self.set_chest_location(index, value);
        index
    }

    fn advance_chest_and_big_key_counts(&mut self) -> usize {
        let index = usize::from(self.num_chests_x2) >> 1;
        let next = ((index + 1) * 2) as u16;
        self.set_num_chests_x2(next);
        self.set_num_big_key_locks_x2(next);
        index
    }

    fn advance_big_key_lock_count(&mut self) -> usize {
        let index = usize::from(self.num_big_key_locks_x2) >> 1;
        self.set_num_big_key_locks_x2(((index + 1) * 2) as u16);
        index
    }

    fn set_chest_location(&mut self, index: usize, value: u16) {
        if let Some(location) = self.chest_locations.get_mut(index) {
            *location = value;
        }
    }

    fn set_chest_location_for_offset_x2(&mut self, offset_x2: usize, value: u16) {
        self.set_chest_location(offset_x2 >> 1, value);
    }

    fn set_chest_reveal_cursor_x2(&mut self, value: u16) {
        self.chest_reveal_cursor_x2 = value;
    }

    fn clear_chest_reveal_cursor(&mut self) {
        self.set_chest_reveal_cursor_x2(0);
    }

    fn set_replacement_tile_destination_x2(&mut self, value: u16) {
        self.replacement_tile_destination_x2 = value;
    }

    fn set_replacement_tile_source_x2(&mut self, value: u16) {
        self.replacement_tile_source_x2 = value;
    }

    fn clear_replacement_tile_destination(&mut self) {
        self.set_replacement_tile_destination_x2(0);
    }

    fn clear_chest_location(&mut self, index: usize) {
        self.set_chest_location(index, 0);
    }

    fn set_replacement_tilemap_quad(&mut self, index: usize, quad: [u16; 4]) {
        if let Some(slot) = self.replacement_tilemap_quads.get_mut(index) {
            *slot = quad;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonStairListsState {
    counters: [u16; DUNGEON_STAIR_LIST_COUNT],
    inter_staircases: [u16; DUNGEON_INTER_STAIRCASE_TABLE_WORDS],
    stairs_table_1: [u16; DUNGEON_STAIR_TABLE_1_WORDS],
    stairs_table_2: [u16; DUNGEON_STAIR_TABLE_2_WORDS],
}

impl DungeonStairListsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut counters = [0; DUNGEON_STAIR_LIST_COUNT];
        for list in ALL_DUNGEON_STAIR_LISTS {
            counters[stair_list_index(list)] = read_le_u16(ram, stair_list_counter_address(list));
        }

        let mut inter_staircases = [0; DUNGEON_INTER_STAIRCASE_TABLE_WORDS];
        for (index, position) in inter_staircases.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_INTER_STAIRCASES + index * 2);
        }

        let mut stairs_table_1 = [0; DUNGEON_STAIR_TABLE_1_WORDS];
        for (index, position) in stairs_table_1.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_STAIRS_TABLE_1 + index * 2);
        }

        let mut stairs_table_2 = [0; DUNGEON_STAIR_TABLE_2_WORDS];
        for (index, position) in stairs_table_2.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_STAIRS_TABLE_2 + index * 2);
        }

        Self {
            counters,
            inter_staircases,
            stairs_table_1,
            stairs_table_2,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for list in ALL_DUNGEON_STAIR_LISTS {
            write_le_u16(
                ram,
                stair_list_counter_address(list),
                self.stair_list_count(list),
            );
        }

        for (index, position) in self.inter_staircases.iter().enumerate() {
            write_le_u16(ram, DUNG_INTER_STAIRCASES + index * 2, *position);
        }

        for (index, position) in self.stairs_table_1.iter().enumerate() {
            write_le_u16(ram, DUNG_STAIRS_TABLE_1 + index * 2, *position);
        }

        for (index, position) in self.stairs_table_2.iter().enumerate() {
            write_le_u16(ram, DUNG_STAIRS_TABLE_2 + index * 2, *position);
        }
    }

    pub(crate) fn stair_list_count(&self, list: DungeonStairList) -> u16 {
        self.counters[stair_list_index(list)]
    }

    pub(crate) fn stair_list_tilemap_pos(&self, list: DungeonStairList, offset_x2: u16) -> u16 {
        let index = usize::from(offset_x2 >> 1);
        match stair_list_table(list) {
            DungeonStairTilemapTable::Stairs1 => {
                self.stairs_table_1.get(index).copied().unwrap_or(0)
            }
            DungeonStairTilemapTable::Stairs2 => {
                self.stairs_table_2.get(index).copied().unwrap_or(0)
            }
        }
    }

    pub(crate) fn inter_staircase_pos(&self, index: usize) -> u16 {
        self.inter_staircases.get(index).copied().unwrap_or(0)
    }

    fn set_stair_list_count(&mut self, list: DungeonStairList, value: u16) {
        self.counters[stair_list_index(list)] = value;
    }

    fn clear_all_counts(&mut self) {
        self.counters.fill(0);
    }

    fn sync_stair_list_counts(&mut self, lists: &[DungeonStairList], value: u16) {
        for &list in lists {
            self.set_stair_list_count(list, value);
        }
    }

    fn append_interroom_staircase(&mut self, list: DungeonStairList, tilemap_pos: u16) -> u16 {
        let index = usize::from(self.stair_list_count(list)) >> 1;
        if let Some(position) = self.inter_staircases.get_mut(index) {
            *position = tilemap_pos;
        }
        self.stair_list_count(list).wrapping_add(2)
    }

    fn append_bg1_stair_table_position(&mut self, list: DungeonStairList, tilemap_pos: u16) -> u16 {
        let index = usize::from(self.stair_list_count(list)) >> 1;
        if let Some(position) = self.stairs_table_1.get_mut(index) {
            *position = tilemap_pos;
        }
        let next = self.stair_list_count(list).wrapping_add(2);
        self.set_stair_list_count(list, next);
        next
    }

    fn append_stair_table_position(&mut self, list: DungeonStairList, tilemap_pos: u16) -> u16 {
        let index = usize::from(self.stair_list_count(list)) >> 1;
        match stair_list_table(list) {
            DungeonStairTilemapTable::Stairs1 => {
                if let Some(position) = self.stairs_table_1.get_mut(index) {
                    *position = tilemap_pos;
                }
            }
            DungeonStairTilemapTable::Stairs2 => {
                if let Some(position) = self.stairs_table_2.get_mut(index) {
                    *position = tilemap_pos;
                }
            }
        }
        let next = self.stair_list_count(list).wrapping_add(2);
        self.set_stair_list_count(list, next);
        next
    }

    fn promote_water_stairs_to_active(&mut self) {
        let north_stairs = self.stair_list_count(DungeonStairList::InRoomUpNorthWater);
        let active_ladders = self.stair_list_count(DungeonStairList::ActivatedWaterLadders);
        let south_stairs = self.stair_list_count(DungeonStairList::InRoomUpSouthWater);
        self.set_stair_list_count(DungeonStairList::InterPseudoUpNorth, north_stairs);
        self.set_stair_list_count(DungeonStairList::WaterSideStepSwitch, active_ladders);
        self.set_stair_list_count(DungeonStairList::ActivatedWaterLadders, 0);
        self.set_stair_list_count(DungeonStairList::InRoomUpNorthWater, 0);
        self.set_stair_list_count(DungeonStairList::WetStairs, south_stairs);
        self.set_stair_list_count(DungeonStairList::InRoomUpSouthWater, 0);
    }

    fn set_inter_staircase_pos(&mut self, index: usize, value: u16) {
        if let Some(position) = self.inter_staircases.get_mut(index) {
            *position = value;
        }
    }
}

const ALL_DUNGEON_STAIR_LISTS: [DungeonStairList; DUNGEON_STAIR_LIST_COUNT] = [
    DungeonStairList::InterRoomUpNorth,
    DungeonStairList::InterRoomSouthDown,
    DungeonStairList::InRoomUpNorth,
    DungeonStairList::InRoomSouthDown,
    DungeonStairList::InterPseudoUpNorth,
    DungeonStairList::InRoomUpNorthWater,
    DungeonStairList::ActivatedWaterLadders,
    DungeonStairList::WetStairs,
    DungeonStairList::InRoomUpSouthWater,
    DungeonStairList::Stairs1,
    DungeonStairList::Stairs2,
    DungeonStairList::WaterLadders,
    DungeonStairList::WaterSideStepSwitch,
    DungeonStairList::WallUpNorthSpiral,
    DungeonStairList::WallDownNorthSpiral,
    DungeonStairList::WallUpNorthSpiralBg1,
    DungeonStairList::WallDownNorthSpiralBg1,
    DungeonStairList::InterRoomUpNorthStraight,
    DungeonStairList::InterRoomUpSouthStraight,
    DungeonStairList::InterRoomDownNorthStraight,
    DungeonStairList::InterRoomDownSouthStraight,
];

#[derive(Clone, Copy)]
enum DungeonStairTilemapTable {
    Stairs1,
    Stairs2,
}

fn stair_list_index(list: DungeonStairList) -> usize {
    match list {
        DungeonStairList::InterRoomUpNorth => 0,
        DungeonStairList::InterRoomSouthDown => 1,
        DungeonStairList::InRoomUpNorth => 2,
        DungeonStairList::InRoomSouthDown => 3,
        DungeonStairList::InterPseudoUpNorth => 4,
        DungeonStairList::InRoomUpNorthWater => 5,
        DungeonStairList::ActivatedWaterLadders => 6,
        DungeonStairList::WetStairs => 7,
        DungeonStairList::InRoomUpSouthWater => 8,
        DungeonStairList::Stairs1 => 9,
        DungeonStairList::Stairs2 => 10,
        DungeonStairList::WaterLadders => 11,
        DungeonStairList::WaterSideStepSwitch => 12,
        DungeonStairList::WallUpNorthSpiral => 13,
        DungeonStairList::WallDownNorthSpiral => 14,
        DungeonStairList::WallUpNorthSpiralBg1 => 15,
        DungeonStairList::WallDownNorthSpiralBg1 => 16,
        DungeonStairList::InterRoomUpNorthStraight => 17,
        DungeonStairList::InterRoomUpSouthStraight => 18,
        DungeonStairList::InterRoomDownNorthStraight => 19,
        DungeonStairList::InterRoomDownSouthStraight => 20,
    }
}

fn stair_list_counter_address(list: DungeonStairList) -> usize {
    match list {
        DungeonStairList::InterRoomUpNorth => DUNG_NUM_INTER_ROOM_UPNORTH_STAIRS_LOCAL,
        DungeonStairList::InterRoomSouthDown => DUNG_NUM_INTER_ROOM_SOUTHDOWN_STAIRS_LOCAL,
        DungeonStairList::InRoomUpNorth => DUNG_NUM_INROOM_UPNORTH_STAIRS,
        DungeonStairList::InRoomSouthDown => DUNG_NUM_INROOM_SOUTHDOWN_STAIRS_LOCAL,
        DungeonStairList::InterPseudoUpNorth => DUNG_NUM_INTERPSEUDO_UPNORTH_STAIRS,
        DungeonStairList::InRoomUpNorthWater => DUNG_NUM_INROOM_UPNORTH_STAIRS_WATER,
        DungeonStairList::ActivatedWaterLadders => DUNG_NUM_ACTIVATED_WATER_LADDERS,
        DungeonStairList::WetStairs => DUNG_NUM_STAIRS_WET,
        DungeonStairList::InRoomUpSouthWater => DUNG_NUM_INROOM_UPSOUTH_STAIRS_WATER,
        DungeonStairList::Stairs1 => DUNG_NUM_STAIRS_1,
        DungeonStairList::Stairs2 => DUNG_NUM_STAIRS_2,
        DungeonStairList::WaterLadders => DUNG_NUM_WATER_LADDERS_LOCAL,
        DungeonStairList::WaterSideStepSwitch => WATER_SIDE_STEP_SWITCH,
        DungeonStairList::WallUpNorthSpiral => DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_LOCAL,
        DungeonStairList::WallDownNorthSpiral => DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_LOCAL,
        DungeonStairList::WallUpNorthSpiralBg1 => DUNG_NUM_WALL_UPNORTH_SPIRAL_STAIRS_2_LOCAL,
        DungeonStairList::WallDownNorthSpiralBg1 => DUNG_NUM_WALL_DOWNNORTH_SPIRAL_STAIRS_2_LOCAL,
        DungeonStairList::InterRoomUpNorthStraight => {
            DUNG_NUM_INTER_ROOM_UPNORTH_STRAIGHT_STAIRS_LOCAL
        }
        DungeonStairList::InterRoomUpSouthStraight => {
            DUNG_NUM_INTER_ROOM_UPSOUTH_STRAIGHT_STAIRS_LOCAL
        }
        DungeonStairList::InterRoomDownNorthStraight => {
            DUNG_NUM_INTER_ROOM_DOWNNORTH_STRAIGHT_STAIRS_LOCAL
        }
        DungeonStairList::InterRoomDownSouthStraight => {
            DUNG_NUM_INTER_ROOM_DOWNSOUTH_STRAIGHT_STAIRS_LOCAL
        }
    }
}

fn stair_list_table(list: DungeonStairList) -> DungeonStairTilemapTable {
    match list {
        DungeonStairList::WetStairs
        | DungeonStairList::InRoomUpSouthWater
        | DungeonStairList::Stairs1
        | DungeonStairList::Stairs2 => DungeonStairTilemapTable::Stairs2,
        _ => DungeonStairTilemapTable::Stairs1,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonBg2AttributeState {
    attrs: Vec<u8>,
}

impl Default for DungeonBg2AttributeState {
    fn default() -> Self {
        Self {
            attrs: vec![0; DUNGEON_BG2_ATTR_BUFFER_LEN],
        }
    }
}

impl DungeonBg2AttributeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut attrs = vec![0; DUNGEON_BG2_ATTR_BUFFER_LEN];
        let available = ram.len().saturating_sub(DUNGEON_BG2_ATTR_TABLE);
        let len = attrs.len().min(available);
        attrs[..len].copy_from_slice(&ram[DUNGEON_BG2_ATTR_TABLE..DUNGEON_BG2_ATTR_TABLE + len]);
        Self { attrs }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        let available = ram.len().saturating_sub(DUNGEON_BG2_ATTR_TABLE);
        let len = self.attrs.len().min(available);
        ram[DUNGEON_BG2_ATTR_TABLE..DUNGEON_BG2_ATTR_TABLE + len]
            .copy_from_slice(&self.attrs[..len]);
    }

    pub(crate) fn bg2_attr(&self, offset: usize) -> u8 {
        self.attrs.get(offset).copied().unwrap_or(0)
    }

    pub(crate) fn bg2_attr_word(&self, offset: usize) -> u16 {
        u16::from(self.bg2_attr(offset)) | (u16::from(self.bg2_attr(offset + 1)) << 8)
    }

    pub(crate) fn bg1_attr(&self, offset: usize) -> u8 {
        self.attrs
            .get(DUNGEON_BG1_ATTR_BUFFER_OFFSET + offset)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn bg1_attr_word(&self, offset: usize) -> u16 {
        u16::from(self.bg1_attr(offset)) | (u16::from(self.bg1_attr(offset + 1)) << 8)
    }

    pub(crate) fn attr_for_tile(&self, ram: &[u8], tile: usize) -> u8 {
        ram.get(ATTRIBUTES_FOR_TILE_PLAYER + (tile & 0x03ff))
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn bg2_attr_address(&self, offset: usize) -> usize {
        DUNGEON_BG2_ATTR_TABLE + offset
    }

    pub(crate) fn bg2_attr_pair(&self, offset: usize) -> Option<(u8, u8)> {
        Some((
            *self.attrs.get(offset)?,
            *self.attrs.get(offset.wrapping_add(1))?,
        ))
    }

    pub(crate) fn bg2_attr_slice(&self, start: usize, len: usize) -> &[u8] {
        &self.attrs[start..start + len]
    }

    fn set_bg2_attr(&mut self, offset: usize, value: u8) {
        self.attrs[offset] = value;
    }

    fn set_bg2_attr_word(&mut self, offset: usize, value: u16) {
        self.attrs[offset] = value as u8;
        self.attrs[offset + 1] = (value >> 8) as u8;
    }

    fn set_bg1_attr_word(&mut self, offset: usize, value: u16) {
        let offset = DUNGEON_BG1_ATTR_BUFFER_OFFSET + offset;
        self.attrs[offset] = value as u8;
        self.attrs[offset + 1] = (value >> 8) as u8;
    }

    fn xor_bg2_attr(&mut self, offset: usize, value: u8) {
        self.attrs[offset] ^= value;
    }

    fn xor_bg1_attr(&mut self, offset: usize, value: u8) {
        self.attrs[DUNGEON_BG1_ATTR_BUFFER_OFFSET + offset] ^= value;
    }

    fn fill_bg2_attr_range(&mut self, start: usize, len: usize, value: u8) {
        self.attrs[start..start + len].fill(value);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonSavegameState {
    state_bits: u16,
}

impl DungeonSavegameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            state_bits: read_le_u16(ram, DUNG_SAVEGAME_STATE_BITS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNG_SAVEGAME_STATE_BITS, self.state_bits);
    }

    pub(crate) fn savegame_state_bits(&self) -> u16 {
        self.state_bits
    }

    pub(crate) fn has_savegame_state_bits(&self, mask: u16) -> bool {
        self.state_bits & mask != 0
    }

    fn set_savegame_state_bits(&mut self, value: u16) {
        self.state_bits = value;
    }

    fn clear_savegame_state_bits(&mut self) {
        self.state_bits = 0;
    }

    fn clear_savegame_state_high(&mut self) {
        self.state_bits &= 0x00ff;
    }

    fn clear_savegame_state_low(&mut self) {
        self.state_bits &= 0xff00;
    }

    fn set_savegame_state_high_bits(&mut self, mask: u8) {
        self.state_bits |= u16::from(mask) << 8;
    }

    fn or_savegame_state_bits(&mut self, mask: u16) -> u16 {
        self.state_bits |= mask;
        self.state_bits
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonTorchState {
    timers: [u8; DUNGEON_TORCH_TIMER_COUNT],
    attr: u8,
    lit_torches: u8,
    lights_out_request: u8,
    lights_out_request_copy: u8,
    dark_with_lantern: u8,
    ganon_torch_count: u8,
    torches_start_index: u16,
    torch_index: u16,
    object_data_positions: [u16; DUNGEON_TORCH_OBJECT_POS_COUNT],
    torch_data_words: Vec<u16>,
}

impl DungeonTorchState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut timers = [0; DUNGEON_TORCH_TIMER_COUNT];
        for (index, timer) in timers.iter_mut().enumerate() {
            *timer = ram.get(TORCH_TIMERS + index).copied().unwrap_or(0);
        }

        let mut object_data_positions = [0; DUNGEON_TORCH_OBJECT_POS_COUNT];
        for (index, position) in object_data_positions.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2);
        }

        let mut torch_data_words = vec![0; DUNGEON_TORCH_DATA_SCAN_WORDS];
        for (index, word) in torch_data_words.iter_mut().enumerate() {
            *word = read_le_u16(ram, DUNGEON_TORCH_DATA + index * 2);
        }

        Self {
            timers,
            attr: ram.get(DUNGEON_TORCH_ATTR).copied().unwrap_or(0),
            lit_torches: ram.get(DUNG_NUM_LIT_TORCHES).copied().unwrap_or(0),
            lights_out_request: ram.get(DUNG_WANT_LIGHTS_OUT).copied().unwrap_or(0),
            lights_out_request_copy: ram.get(DUNG_WANT_LIGHTS_OUT_COPY).copied().unwrap_or(0),
            dark_with_lantern: ram.get(HDR_DUNGEON_DARK_WITH_LANTERN).copied().unwrap_or(0),
            ganon_torch_count: ram.get(GANON_TORCH_COUNT).copied().unwrap_or(0),
            torches_start_index: read_le_u16(ram, DUNG_INDEX_OF_TORCHES_START),
            torch_index: read_le_u16(ram, DUNG_INDEX_OF_TORCHES),
            object_data_positions,
            torch_data_words,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[TORCH_TIMERS..TORCH_TIMERS + DUNGEON_TORCH_TIMER_COUNT].copy_from_slice(&self.timers);
        ram[DUNGEON_TORCH_ATTR] = self.attr;
        ram[DUNG_NUM_LIT_TORCHES] = self.lit_torches;
        ram[DUNG_WANT_LIGHTS_OUT] = self.lights_out_request;
        ram[DUNG_WANT_LIGHTS_OUT_COPY] = self.lights_out_request_copy;
        ram[HDR_DUNGEON_DARK_WITH_LANTERN] = self.dark_with_lantern;
        ram[GANON_TORCH_COUNT] = self.ganon_torch_count;
        write_le_u16(ram, DUNG_INDEX_OF_TORCHES_START, self.torches_start_index);
        write_le_u16(ram, DUNG_INDEX_OF_TORCHES, self.torch_index);
        for (index, position) in self.object_data_positions.iter().enumerate() {
            write_le_u16(ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2, *position);
        }
        for (index, word) in self.torch_data_words.iter().enumerate() {
            write_le_u16(ram, DUNGEON_TORCH_DATA + index * 2, *word);
        }
    }

    pub(crate) fn timer(&self, index: usize) -> u8 {
        self.timers.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn attr_index(&self) -> usize {
        usize::from(self.attr & 0x0f)
    }

    pub(crate) fn torch_attr(&self) -> u8 {
        self.attr
    }

    pub(crate) fn lit_torches(&self) -> u8 {
        self.lit_torches
    }

    pub(crate) fn wants_lights_out(&self) -> u8 {
        self.lights_out_request
    }

    pub(crate) fn wants_lights_out_copy(&self) -> u8 {
        self.lights_out_request_copy
    }

    pub(crate) fn any_lights_out_request(&self) -> u8 {
        self.lights_out_request | self.lights_out_request_copy
    }

    pub(crate) fn dungeon_dark_with_lantern(&self) -> bool {
        self.dark_with_lantern != 0
    }

    pub(crate) fn dungeon_dark_with_lantern_raw(&self) -> u8 {
        self.dark_with_lantern
    }

    pub(crate) fn ganon_torch_count(&self) -> u8 {
        self.ganon_torch_count
    }

    pub(crate) fn torches_start_index(&self) -> u16 {
        self.torches_start_index
    }

    pub(crate) fn torch_index(&self) -> u16 {
        self.torch_index
    }

    pub(crate) fn torch_data_word_at_byte_offset(&self, byte_offset: usize) -> u16 {
        self.torch_data_words
            .get(byte_offset >> 1)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn torch_object_data_pos(&self, index: usize) -> u16 {
        self.object_data_positions.get(index).copied().unwrap_or(0)
    }

    fn clear_timer(&mut self, index: usize) {
        if let Some(timer) = self.timers.get_mut(index) {
            *timer = 0;
        }
    }

    fn clear_timers(&mut self) {
        self.timers.fill(0);
    }

    fn refresh_object_data_positions_from_ram(&mut self, ram: &[u8]) {
        for (index, position) in self.object_data_positions.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2);
        }
    }

    fn set_timer(&mut self, index: usize, value: u8) {
        if let Some(timer) = self.timers.get_mut(index) {
            *timer = value;
        }
    }

    fn clear_lit_torches(&mut self) {
        self.lit_torches = 0;
    }

    fn set_lit_torches(&mut self, value: u8) {
        self.lit_torches = value;
    }

    fn increment_lit_torches(&mut self) -> u8 {
        self.lit_torches = self.lit_torches.wrapping_add(1);
        self.lit_torches
    }

    fn decrement_lit_torches(&mut self) -> u8 {
        self.lit_torches = self.lit_torches.wrapping_sub(1);
        self.lit_torches
    }

    fn set_lights_out_request(&mut self, value: u8) {
        self.lights_out_request = value;
    }

    fn clear_lights_out_request(&mut self) {
        self.lights_out_request = 0;
    }

    fn set_lights_out_request_copy(&mut self, value: u8) {
        self.lights_out_request_copy = value;
    }

    fn copy_lights_out_request(&mut self) {
        self.lights_out_request_copy = self.lights_out_request;
    }

    fn clear_lights_out_requests(&mut self) {
        self.lights_out_request = 0;
        self.lights_out_request_copy = 0;
    }

    fn set_dungeon_dark_with_lantern(&mut self) {
        self.dark_with_lantern = 1;
    }

    fn set_dungeon_dark_with_lantern_raw(&mut self, value: u8) {
        self.dark_with_lantern = value;
    }

    fn clear_dungeon_dark_with_lantern(&mut self) {
        self.dark_with_lantern = 0;
    }

    fn set_torch_index_range_start(&mut self, value: u16) {
        self.torches_start_index = value;
    }

    fn set_torch_index(&mut self, value: u16) {
        self.torch_index = value;
    }

    fn set_torch_data_word_index(&mut self, index: usize, value: u16) {
        if let Some(word) = self.torch_data_words.get_mut(index) {
            *word = value;
        }
    }

    fn set_attr(&mut self, value: u8) {
        self.attr = value;
    }

    fn clear_attr(&mut self) {
        self.attr = 0;
    }

    fn set_ganon_torch_count(&mut self, value: u8) {
        self.ganon_torch_count = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonEntranceBackupState {
    exit_tile_themes: [u8; 4],
    overworld_screen_high: u8,
    overlay_high: u8,
}

impl DungeonEntranceBackupState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut exit_tile_themes = [0; 4];
        for (index, theme) in exit_tile_themes.iter_mut().enumerate() {
            *theme = ram
                .get(OVERWORLD_EXIT_TILE_THEME_INDEX + index)
                .copied()
                .unwrap_or(0);
        }
        Self {
            exit_tile_themes,
            overworld_screen_high: ram.get(OVERWORLD_SCREEN_INDEX + 1).copied().unwrap_or(0),
            overlay_high: ram.get(OVERLAY_INDEX + 1).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[OVERWORLD_EXIT_TILE_THEME_INDEX..OVERWORLD_EXIT_TILE_THEME_INDEX + 4]
            .copy_from_slice(&self.exit_tile_themes);
        ram[OVERWORLD_SCREEN_INDEX + 1] = self.overworld_screen_high;
        ram[OVERLAY_INDEX + 1] = self.overlay_high;
    }

    pub(crate) fn exit_tile_theme(&self, index: usize) -> u8 {
        self.exit_tile_themes.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn overworld_screen_high(&self) -> u8 {
        self.overworld_screen_high
    }

    pub(crate) fn overlay_high(&self) -> u8 {
        self.overlay_high
    }

    pub(crate) fn cache_exit_tile_themes(&mut self, overworld: u8, main: u8, aux: u8, sprite: u8) {
        self.exit_tile_themes = [overworld, main, aux, sprite];
    }

    pub(crate) fn clear_overworld_screen_high(&mut self) {
        self.overworld_screen_high = 0;
    }

    pub(crate) fn clear_overlay_high(&mut self) {
        self.overlay_high = 0;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonHeaderState {
    tags: [u8; DUNGEON_HEADER_TAG_COUNT],
    travel_destinations: [u8; DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT],
    plane_scratch: [u8; DUNGEON_HEADER_PLANE_SCRATCH_COUNT],
}

impl DungeonHeaderState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut tags = [0; DUNGEON_HEADER_TAG_COUNT];
        for (index, tag) in tags.iter_mut().enumerate() {
            *tag = ram.get(DUNGEON_HEADER_TAG + index).copied().unwrap_or(0);
        }

        let mut travel_destinations = [0; DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT];
        for (index, destination) in travel_destinations.iter_mut().enumerate() {
            *destination = ram
                .get(DUNGEON_HEADER_TRAVEL_DESTINATIONS + index)
                .copied()
                .unwrap_or(0);
        }

        let mut plane_scratch = [0; DUNGEON_HEADER_PLANE_SCRATCH_COUNT];
        for (index, plane) in plane_scratch.iter_mut().enumerate() {
            *plane = ram
                .get(DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + index)
                .copied()
                .unwrap_or(0);
        }

        Self {
            tags,
            travel_destinations,
            plane_scratch,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DUNGEON_HEADER_TAG..DUNGEON_HEADER_TAG + DUNGEON_HEADER_TAG_COUNT]
            .copy_from_slice(&self.tags);
        ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS
            ..DUNGEON_HEADER_TRAVEL_DESTINATIONS + DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT]
            .copy_from_slice(&self.travel_destinations);
        ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE
            ..DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + DUNGEON_HEADER_PLANE_SCRATCH_COUNT]
            .copy_from_slice(&self.plane_scratch);
    }

    pub(crate) fn header_tag(&self, index: usize) -> u8 {
        self.tags.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn primary_header_tag(&self) -> u8 {
        self.header_tag(0)
    }

    pub(crate) fn travel_destination(&self, index: usize) -> u8 {
        self.travel_destinations.get(index).copied().unwrap_or(0)
    }

    fn copy_travel_destinations_from_header(&mut self, header: &[u8]) {
        self.travel_destinations.copy_from_slice(&header[9..14]);
    }

    pub(crate) fn hole_teleporter_plane(&self, index: usize) -> u8 {
        self.plane_scratch.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn staircase_plane(&self, index: usize) -> u8 {
        self.plane_scratch
            .get(DUNGEON_HEADER_STAIRCASE_PLANE - DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + index)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn set_hole_teleporter_planes(&mut self, packed: u8, extra: u8) {
        self.plane_scratch[0] = packed & 3;
        self.plane_scratch[1] = (packed >> 2) & 3;
        self.plane_scratch[2] = (packed >> 4) & 3;
        self.plane_scratch[3] = (packed >> 6) & 3;
        self.plane_scratch[4] = extra & 3;
    }

    fn set_header_tag(&mut self, index: usize, value: u8) {
        self.tags[index] = value;
    }

    fn clear_header_tag(&mut self, index: usize) {
        self.set_header_tag(index, 0);
    }

    fn clear_header_tags(&mut self, count: usize) {
        self.tags[..count].fill(0);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonScratchWordState {
    r16: u16,
    r18: u16,
}

impl DungeonScratchWordState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            r16: read_le_u16(ram, DUNGEON_WORK_R16),
            r18: read_le_u16(ram, DUNGEON_WORK_R18),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNGEON_WORK_R16, self.r16);
        write_le_u16(ram, DUNGEON_WORK_R18, self.r18);
    }

    pub(crate) fn high(&self) -> u8 {
        (self.r16 >> 8) as u8
    }

    pub(crate) fn word(&self) -> u16 {
        self.r16
    }

    pub(crate) fn minigame_previous_chest_choice(&self) -> u8 {
        self.r16 as u8
    }

    pub(crate) fn primary_word(&self) -> u16 {
        self.r16
    }

    pub(crate) fn secondary_word(&self) -> u16 {
        self.r18
    }

    pub(crate) fn primary_low(&self) -> u8 {
        self.r16 as u8
    }

    pub(crate) fn secondary_low(&self) -> u8 {
        self.r18 as u8
    }

    pub(crate) fn decrement_high(&mut self) -> u8 {
        let next = self.high().wrapping_sub(1);
        self.r16 = (self.r16 & 0x00ff) | (u16::from(next) << 8);
        next
    }

    pub(crate) fn set_word(&mut self, value: u16) {
        self.r16 = value;
    }

    pub(crate) fn clear_word(&mut self) {
        self.set_word(0);
    }

    pub(crate) fn set_liftable_tile_probe_position(&mut self, y: u16, x: u16) {
        self.r16 = y;
        self.r18 = x;
    }

    pub(crate) fn set_ganon_door_bounce_countdown(&mut self, value: u16) {
        self.set_word(value);
    }

    pub(crate) fn decrement_ganon_door_bounce_low(&mut self) -> u8 {
        let next = (self.r16 as u8).wrapping_sub(1);
        self.r16 = (self.r16 & 0xff00) | u16::from(next);
        next
    }

    pub(crate) fn clear_module_transition_counter(&mut self) {
        self.r16 = (self.r16 & 0xff00) | 0;
    }

    pub(crate) fn set_minigame_previous_chest_choice(&mut self, value: u8) {
        self.r16 = (self.r16 & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_primary_word(&mut self, value: u16) {
        self.r16 = value;
    }

    pub(crate) fn set_secondary_word(&mut self, value: u16) {
        self.r18 = value;
    }

    pub(crate) fn clear_primary_word(&mut self) {
        self.set_primary_word(0);
    }

    pub(crate) fn set_primary_low(&mut self, value: u8) {
        self.r16 = (self.r16 & 0xff00) | u16::from(value);
    }

    pub(crate) fn decrement_primary_low(&mut self) -> u8 {
        let next = self.primary_low().wrapping_sub(1);
        self.set_primary_low(next);
        next
    }

    pub(crate) fn increment_secondary_low(&mut self) -> u8 {
        let next = self.secondary_low().wrapping_add(1);
        self.r18 = (self.r18 & 0xff00) | u16::from(next);
        next
    }
}

pub(crate) struct NativeDungeonEntranceBackupBridgeMut<'a> {
    state: &'a mut DungeonEntranceBackupState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonEntranceBackupBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonEntranceBackupState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonEntranceBackupState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn cache_exit_tile_themes(&mut self) {
        self.state.cache_exit_tile_themes(
            self.ram[OVERWORLD_TILE_THEME_INDEX],
            self.ram[MAIN_TILE_THEME_INDEX],
            self.ram[AUX_TILE_THEME_INDEX],
            self.ram[SPRITE_GRAPHICS_INDEX],
        );
        self.sync();
    }

    pub(crate) fn clear_overworld_screen_high(&mut self) {
        self.state.clear_overworld_screen_high();
        self.sync();
    }

    pub(crate) fn clear_overlay_high(&mut self) {
        self.state.clear_overlay_high();
        self.sync();
    }
}

pub(crate) struct NativeDungeonScratchWordBridgeMut<'a> {
    scratch: &'a mut DungeonScratchWordState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonScratchWordBridgeMut<'a> {
    pub(crate) fn new(scratch: &'a mut DungeonScratchWordState, ram: &'a mut [u8]) -> Self {
        Self { scratch, ram }
    }

    fn sync(&mut self) {
        self.scratch.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.scratch,
            DungeonScratchWordState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn decrement_high(&mut self) -> u8 {
        let next = self.scratch.decrement_high();
        self.sync();
        next
    }

    pub(crate) fn set_word(&mut self, value: u16) {
        self.scratch.set_word(value);
        self.sync();
    }

    pub(crate) fn clear_word(&mut self) {
        self.scratch.clear_word();
        self.sync();
    }

    pub(crate) fn set_liftable_tile_probe_position(&mut self, y: u16, x: u16) {
        self.scratch.set_liftable_tile_probe_position(y, x);
        self.sync();
    }

    pub(crate) fn set_ganon_door_bounce_countdown(&mut self, value: u16) {
        self.scratch.set_ganon_door_bounce_countdown(value);
        self.sync();
    }

    pub(crate) fn decrement_ganon_door_bounce_low(&mut self) -> u8 {
        let next = self.scratch.decrement_ganon_door_bounce_low();
        self.sync();
        next
    }

    pub(crate) fn clear_module_transition_counter(&mut self) {
        self.scratch.clear_module_transition_counter();
        self.sync();
    }

    pub(crate) fn set_minigame_previous_chest_choice(&mut self, value: u8) {
        self.scratch.set_minigame_previous_chest_choice(value);
        self.sync();
    }

    pub(crate) fn set_primary_word(&mut self, value: u16) {
        self.scratch.set_primary_word(value);
        self.sync();
    }

    pub(crate) fn set_secondary_word(&mut self, value: u16) {
        self.scratch.set_secondary_word(value);
        self.sync();
    }

    pub(crate) fn clear_primary_word(&mut self) {
        self.scratch.clear_primary_word();
        self.sync();
    }

    pub(crate) fn set_primary_low(&mut self, value: u8) {
        self.scratch.set_primary_low(value);
        self.sync();
    }

    pub(crate) fn decrement_primary_low(&mut self) -> u8 {
        let next = self.scratch.decrement_primary_low();
        self.sync();
        next
    }

    pub(crate) fn increment_secondary_low(&mut self) -> u8 {
        let next = self.scratch.increment_secondary_low();
        self.sync();
        next
    }
}

pub(crate) struct NativeDungeonSavegameBridgeMut<'a> {
    state: &'a mut DungeonSavegameState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonSavegameBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonSavegameState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DungeonSavegameState::load_from_ram(self.ram));
    }

    pub(crate) fn set_savegame_state_bits(&mut self, value: u16) {
        self.state.set_savegame_state_bits(value);
        self.sync();
    }

    pub(crate) fn clear_savegame_state_bits(&mut self) {
        self.state.clear_savegame_state_bits();
        self.sync();
    }

    pub(crate) fn clear_savegame_state_high(&mut self) {
        self.state.clear_savegame_state_high();
        self.sync();
    }

    pub(crate) fn clear_savegame_state_low(&mut self) {
        self.state.clear_savegame_state_low();
        self.sync();
    }

    pub(crate) fn set_savegame_state_high_bits(&mut self, mask: u8) {
        self.state.set_savegame_state_high_bits(mask);
        self.sync();
    }

    pub(crate) fn or_savegame_state_bits(&mut self, mask: u16) -> u16 {
        let value = self.state.or_savegame_state_bits(mask);
        self.sync();
        value
    }
}

pub(crate) struct NativeDungeonBg2AttributeBridgeMut<'a> {
    state: &'a mut DungeonBg2AttributeState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonBg2AttributeBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonBg2AttributeState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonBg2AttributeState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_bg2_attr(&mut self, offset: usize, value: u8) {
        self.state.set_bg2_attr(offset, value);
        self.sync();
    }

    pub(crate) fn set_bg2_attr_word(&mut self, offset: usize, value: u16) {
        self.state.set_bg2_attr_word(offset, value);
        self.sync();
    }

    pub(crate) fn set_bg1_attr_word(&mut self, offset: usize, value: u16) {
        self.state.set_bg1_attr_word(offset, value);
        self.sync();
    }

    pub(crate) fn xor_bg2_attr(&mut self, offset: usize, value: u8) {
        self.state.xor_bg2_attr(offset, value);
        self.sync();
    }

    pub(crate) fn xor_bg1_attr(&mut self, offset: usize, value: u8) {
        self.state.xor_bg1_attr(offset, value);
        self.sync();
    }

    pub(crate) fn fill_bg2_attr_range(&mut self, start: usize, len: usize, value: u8) {
        self.state.fill_bg2_attr_range(start, len, value);
        self.sync();
    }
}

pub(crate) struct NativeDungeonStairListsBridgeMut<'a> {
    state: &'a mut DungeonStairListsState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonStairListsBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonStairListsState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DungeonStairListsState::load_from_ram(self.ram));
    }

    pub(crate) fn set_stair_list_count(&mut self, list: DungeonStairList, value: u16) {
        self.state.set_stair_list_count(list, value);
        self.sync();
    }

    pub(crate) fn clear_all_counts(&mut self) {
        self.state.clear_all_counts();
        self.sync();
    }

    pub(crate) fn sync_stair_list_counts(&mut self, lists: &[DungeonStairList], value: u16) {
        self.state.sync_stair_list_counts(lists, value);
        self.sync();
    }

    pub(crate) fn append_interroom_staircase(
        &mut self,
        list: DungeonStairList,
        tilemap_pos: u16,
    ) -> u16 {
        let next = self.state.append_interroom_staircase(list, tilemap_pos);
        self.sync();
        next
    }

    pub(crate) fn append_bg1_stair_table_position(
        &mut self,
        list: DungeonStairList,
        tilemap_pos: u16,
    ) -> u16 {
        let next = self
            .state
            .append_bg1_stair_table_position(list, tilemap_pos);
        self.sync();
        next
    }

    pub(crate) fn append_stair_table_position(
        &mut self,
        list: DungeonStairList,
        tilemap_pos: u16,
    ) -> u16 {
        let next = self.state.append_stair_table_position(list, tilemap_pos);
        self.sync();
        next
    }

    pub(crate) fn promote_water_stairs_to_active(&mut self) {
        self.state.promote_water_stairs_to_active();
        self.sync();
    }

    pub(crate) fn set_inter_staircase_pos(&mut self, index: usize, value: u16) {
        self.state.set_inter_staircase_pos(index, value);
        self.sync();
    }
}

pub(crate) struct NativeDungeonMovingFloorBridgeMut<'a> {
    state: &'a mut DungeonMovingFloorState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonMovingFloorBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonMovingFloorState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonMovingFloorState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_floor_y_velocity_high(&mut self, value: u8) {
        self.state.set_floor_y_velocity_high(value);
        self.sync();
    }

    pub(crate) fn set_floor_y_velocity(&mut self, value: u16) {
        self.state.set_floor_y_velocity(value);
        self.sync();
    }

    pub(crate) fn set_floor_x_velocity(&mut self, value: u16) {
        self.state.set_floor_x_velocity(value);
        self.sync();
    }

    pub(crate) fn clear_floor_velocity(&mut self) {
        self.state.clear_floor_velocity();
        self.sync();
    }

    pub(crate) fn set_floor_x_offset(&mut self, value: u16) {
        self.state.set_floor_x_offset(value);
        self.sync();
    }

    pub(crate) fn set_floor_y_offset(&mut self, value: u16) {
        self.state.set_floor_y_offset(value);
        self.sync();
    }

    pub(crate) fn set_floor_y_offset_low(&mut self, value: u8) {
        self.state.set_floor_y_offset_low(value);
        self.sync();
    }

    pub(crate) fn set_floor_offsets(&mut self, x: u16, y: u16) {
        self.state.set_floor_offsets(x, y);
        self.sync();
    }

    pub(crate) fn add_floor_x_offset(&mut self, delta: u16) -> u16 {
        let value = self.state.add_floor_x_offset(delta);
        self.sync();
        value
    }

    pub(crate) fn sub_floor_x_offset(&mut self, delta: u16) -> u16 {
        let value = self.state.sub_floor_x_offset(delta);
        self.sync();
        value
    }

    pub(crate) fn add_floor_y_offset(&mut self, delta: u16) -> u16 {
        let value = self.state.add_floor_y_offset(delta);
        self.sync();
        value
    }

    pub(crate) fn sub_floor_y_offset(&mut self, delta: u16) -> u16 {
        let value = self.state.sub_floor_y_offset(delta);
        self.sync();
        value
    }

    pub(crate) fn clear_floor_offsets(&mut self) {
        self.state.clear_floor_offsets();
        self.sync();
    }

    pub(crate) fn clear_floor_move_flags(&mut self) {
        self.state.clear_floor_move_flags();
        self.sync();
    }

    pub(crate) fn set_floor_move_flags(&mut self, value: u16) {
        self.state.set_floor_move_flags(value);
        self.sync();
    }

    pub(crate) fn increment_floor_move_flags(&mut self) {
        self.state.increment_floor_move_flags();
        self.sync();
    }
}

pub(crate) struct NativeDungeonRoomTrackingBridgeMut<'a> {
    state: &'a mut DungeonRoomTrackingState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonRoomTrackingBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonRoomTrackingState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonRoomTrackingState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_room_index2(&mut self, value: u8) {
        self.state.set_room_index2(value);
        self.sync();
    }

    pub(crate) fn set_room_index2_word(&mut self, value: u16) {
        self.state.set_room_index2_word(value);
        self.sync();
    }

    pub(crate) fn set_room_index_prev(&mut self, value: u8) {
        self.state.set_room_index_prev(value);
        self.sync();
    }

    pub(crate) fn set_previous_room_index_word(&mut self, value: u16) {
        self.state.set_previous_room_index_word(value);
        self.sync();
    }

    pub(crate) fn set_room_history_entry(&mut self, index: usize, value: u16) {
        self.state.set_room_history_entry(index, value);
        self.sync();
    }

    pub(crate) fn reset_room_history(&mut self) {
        self.state.reset_room_history();
        self.sync();
    }
}

pub(crate) struct NativeDungeonObjectTrackingBridgeMut<'a> {
    state: &'a mut DungeonObjectTrackingState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonObjectTrackingBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonObjectTrackingState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonObjectTrackingState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_big_rock_starting_address(&mut self, value: u16) {
        self.state.set_big_rock_starting_address(value);
        self.sync();
    }

    pub(crate) fn clear_replacement_tile_states(&mut self) {
        self.state.clear_replacement_tile_states();
        self.sync();
    }

    pub(crate) fn clear_object_data_positions(&mut self) {
        self.state.clear_object_data_positions();
        self.sync();
    }

    pub(crate) fn set_replacement_tile_state(&mut self, index: usize, value: u16) {
        self.state.set_replacement_tile_state(index, value);
        self.sync();
    }

    pub(crate) fn increment_replacement_tile_state(&mut self, index: usize) -> u16 {
        let value = self.state.increment_replacement_tile_state(index);
        self.sync();
        value
    }

    pub(crate) fn clear_replacement_tile_state_low(&mut self, index: usize) {
        self.state.clear_replacement_tile_state_low(index);
        self.sync();
    }

    pub(crate) fn set_object_data_pos(&mut self, index: usize, value: u16) {
        self.state.set_object_data_pos(index, value);
        self.sync();
    }

    pub(crate) fn set_object_tilemap_pos(&mut self, index: usize, value: u16) {
        self.state.set_object_tilemap_pos(index, value);
        self.sync();
    }

    pub(crate) fn set_misc_object_index(&mut self, value: u16) {
        self.state.set_misc_object_index(value);
        self.sync();
    }

    pub(crate) fn clear_misc_object_index(&mut self) {
        self.state.clear_misc_object_index();
        self.sync();
    }

    pub(crate) fn advance_misc_object_index_by(&mut self, value: u16) -> u16 {
        let next = self.state.advance_misc_object_index_by(value);
        self.sync();
        next
    }

    pub(crate) fn set_changeable_object_index(&mut self, index: usize, value: u8) {
        self.state.set_changeable_object_index(index, value);
        self.sync();
    }

    pub(crate) fn clear_changeable_object_index(&mut self, index: usize) {
        self.state.clear_changeable_object_index(index);
        self.sync();
    }
}

pub(crate) struct NativeDungeonDoorBridgeMut<'a> {
    state: &'a mut DungeonDoorState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonDoorBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonDoorState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn sync_preserving_animation_step(&mut self) {
        self.state.sync_door_animation_step_from_ram(self.ram);
        self.sync();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DungeonDoorState::load_from_ram(self.ram));
    }

    pub(crate) fn set_opened_doors(&mut self, value: u16) {
        self.state.set_opened_doors(value);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn or_opened_doors(&mut self, mask: u16) -> u16 {
        let opened = self.state.or_opened_doors(mask);
        self.sync_preserving_animation_step();
        opened
    }

    pub(crate) fn mark_door_opened(&mut self, door: usize) -> u16 {
        let opened = self.state.mark_door_opened(door);
        self.sync_preserving_animation_step();
        opened
    }

    pub(crate) fn set_opened_doors_including_adjacent(&mut self, value: u16) {
        self.state.set_opened_doors_including_adjacent(value);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn mark_opened_door_mask(&mut self, mask: u16) -> u16 {
        let opened = self.state.mark_opened_door_mask(mask);
        self.sync_preserving_animation_step();
        opened
    }

    pub(crate) fn clear_door_tilemap_addresses(&mut self) {
        self.state.clear_door_tilemap_addresses();
        self.sync_preserving_animation_step();
    }

    pub(crate) fn set_door_tilemap_address(&mut self, door: usize, value: u16) {
        self.state.set_door_tilemap_address(door, value);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn load_room_door_tilemap_addresses_from_info(&mut self, door_info: &[u8]) {
        self.state
            .load_room_door_tilemap_addresses_from_info(door_info);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn clear_door_tables(&mut self) {
        self.state.clear_door_tables();
        self.sync_preserving_animation_step();
    }

    pub(crate) fn set_door_type_word(&mut self, door: usize, value: u16) {
        self.state.set_door_type_word(door, value);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn set_door_direction_word(&mut self, door: usize, value: u16) {
        self.state.set_door_direction_word(door, value);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn clear_door_direction(&mut self, door: usize) {
        self.state.clear_door_direction(door);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn set_current_door_index(&mut self, value: u16) {
        self.state.set_current_door_index(value);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn set_current_door_index_for_slot(&mut self, door: usize) {
        self.state.set_current_door_index_for_slot(door);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn advance_current_door_index_by(&mut self, value: u16) -> u16 {
        let next = self.state.advance_current_door_index_by(value);
        self.sync_preserving_animation_step();
        next
    }

    pub(crate) fn set_current_door_pos(&mut self, value: u16) {
        self.state.set_current_door_pos(value);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn clear_current_door_pos(&mut self) {
        self.state.clear_current_door_pos();
        self.sync_preserving_animation_step();
    }

    /// Clears only the LOW byte of DUNG_CUR_DOOR_POS (0x68e), preserving the high byte (0x68f).
    /// C's InterRoomTrans_State15 writes `ram[DUNG_CUR_DOOR_POS_DUNGEON] = 0` as a BYTE, leaving
    /// the high byte (a stale leftover) intact; a full word-clear here diverges scratch (0x68f).
    pub(crate) fn clear_current_door_pos_low_byte(&mut self) {
        self.state.clear_current_door_pos_low_byte();
        self.sync_preserving_animation_step();
    }

    pub(crate) fn clear_door_animation_step(&mut self) {
        self.state.clear_door_animation_step();
        self.sync();
    }

    pub(crate) fn set_door_animation_step(&mut self, value: u16) {
        self.state.set_door_animation_step(value);
        self.sync();
    }

    pub(crate) fn set_door_animation_step_low(&mut self, value: u8) {
        self.state.set_door_animation_step_low(value);
        self.sync();
    }

    pub(crate) fn increment_door_animation_step(&mut self) -> u16 {
        let step = self.state.increment_door_animation_step();
        self.sync();
        step
    }

    pub(crate) fn set_door_open_counter(&mut self, value: u16) {
        self.state.set_door_open_counter(value);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn set_door_open_counter_low(&mut self, value: u8) {
        self.state.set_door_open_counter_low(value);
        self.sync_preserving_animation_step();
    }

    pub(crate) fn clear_door_open_counter_low(&mut self) {
        self.state.clear_door_open_counter_low();
        self.sync_preserving_animation_step();
    }

    pub(crate) fn increment_door_open_counter_low(&mut self) -> u8 {
        let value = self.state.increment_door_open_counter_low();
        self.sync_preserving_animation_step();
        value
    }

    pub(crate) fn mark_door_switch_triggered(&mut self) {
        self.state.mark_door_switch_triggered();
        self.sync_preserving_animation_step();
    }

    pub(crate) fn clear_door_switch_triggered(&mut self) {
        self.state.clear_door_switch_triggered();
        self.sync_preserving_animation_step();
    }

    pub(crate) fn clear_door_barrier_or_switch_flag(&mut self) {
        self.state.clear_door_barrier_or_switch_flag();
        self.sync_preserving_animation_step();
    }
}

pub(crate) struct NativeDungeonRoomLoadBridgeMut<'a> {
    state: &'a mut DungeonRoomLoadState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonRoomLoadBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonRoomLoadState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DungeonRoomLoadState::load_from_ram(self.ram));
    }

    pub(crate) fn set_header_collision(&mut self, value: u8) {
        self.state.set_header_collision(value);
        self.sync();
    }

    pub(crate) fn set_header_collision_2(&mut self, value: u8) {
        self.state.set_header_collision_2(value);
        self.sync();
    }

    pub(crate) fn clear_header_collision_2(&mut self) {
        self.state.clear_header_collision_2();
        self.sync();
    }

    pub(crate) fn set_header_collision_2_mirror(&mut self, value: u8) {
        self.state.set_header_collision_2_mirror(value);
        self.sync();
    }

    pub(crate) fn set_header_collision_2_mirror_high(&mut self, value: u8) {
        self.state.set_header_collision_2_mirror_high(value);
        self.sync();
    }

    pub(crate) fn increment_header_collision_2_mirror(&mut self) -> u8 {
        let value = self.state.increment_header_collision_2_mirror();
        self.sync();
        value
    }

    pub(crate) fn copy_header_collision_2_to_mirror(&mut self) {
        self.state.copy_header_collision_2_to_mirror();
        self.sync();
    }

    pub(crate) fn set_bg2_properties(&mut self, value: u8) {
        self.state.set_bg2_properties(value);
        self.sync();
    }

    pub(crate) fn clear_bg2_properties(&mut self) {
        self.state.clear_bg2_properties();
        self.sync();
    }

    pub(crate) fn set_bg2_properties_backup(&mut self, value: u8) {
        self.state.set_bg2_properties_backup(value);
        self.sync();
    }

    pub(crate) fn set_layout_quadrant_key(&mut self, value: u8) {
        self.state.set_layout_quadrant_key(value);
        self.sync();
    }

    pub(crate) fn update_layout_quadrant_key(
        &mut self,
        layout: u8,
        quadrant_y: u8,
        quadrant_x: u8,
    ) -> u8 {
        let key = self
            .state
            .update_layout_quadrant_key(layout, quadrant_y, quadrant_x);
        self.sync();
        key
    }

    pub(crate) fn set_quadrants_visited(&mut self, value: u16) {
        self.state.set_quadrants_visited(value);
        self.sync();
    }

    pub(crate) fn or_quadrants_visited(&mut self, value: u16) -> u16 {
        let visited = self.state.or_quadrants_visited(value);
        self.sync();
        visited
    }

    pub(crate) fn clear_quadrant_upload_index(&mut self) {
        self.state.clear_quadrant_upload_index();
        self.sync();
    }

    pub(crate) fn advance_quadrant_upload_index_by(&mut self, value: u8) -> u8 {
        let index = self.state.advance_quadrant_upload_index_by(value);
        self.sync();
        index
    }

    pub(crate) fn set_draw_width_indicator(&mut self, value: u8) {
        self.state.set_draw_width_indicator(value);
        self.sync();
    }

    pub(crate) fn set_draw_width_indicator_word(&mut self, value: u16) {
        self.state.set_draw_width_indicator_word(value);
        self.sync();
    }

    pub(crate) fn set_draw_height_indicator(&mut self, value: u8) {
        self.state.set_draw_height_indicator(value);
        self.sync();
    }

    pub(crate) fn set_draw_height_indicator_word(&mut self, value: u16) {
        self.state.set_draw_height_indicator_word(value);
        self.sync();
    }

    pub(crate) fn set_draw_dimensions(&mut self, width: u8, height: u8) {
        self.state.set_draw_dimensions(width, height);
        self.sync();
    }

    pub(crate) fn set_draw_dimensions_words(&mut self, width: u16, height: u16) {
        self.state.set_draw_dimensions_words(width, height);
        self.sync();
    }

    pub(crate) fn set_overlay_to_load(&mut self, value: u8) {
        self.state.set_overlay_to_load(value);
        self.sync();
    }

    pub(crate) fn set_overlay_to_load_if_empty(&mut self, value: u8) {
        self.state.set_overlay_to_load_if_empty(value);
        self.sync();
    }

    pub(crate) fn clear_overlay_to_load(&mut self) {
        self.state.clear_overlay_to_load();
        self.sync();
    }

    pub(crate) fn set_selected_key_door_x2(&mut self, value: u16) {
        self.state.set_selected_key_door_x2(value);
        self.sync();
    }

    pub(crate) fn set_selected_key_door(&mut self, door: usize) {
        self.state.set_selected_key_door(door);
        self.sync();
    }

    pub(crate) fn set_load_ptr_offset(&mut self, value: u16) {
        self.state.set_load_ptr_offset(value);
        self.sync();
    }

    pub(crate) fn advance_load_ptr_offset_by(&mut self, value: u16) -> u16 {
        let offset = self.state.advance_load_ptr_offset_by(value);
        self.sync();
        offset
    }

    pub(crate) fn set_loading_bg_offsets(&mut self, horizontal: u16, vertical: u16) {
        self.state.set_loading_bg_offsets(horizontal, vertical);
        self.sync();
    }
}

pub(crate) struct NativeDungeonEnvironmentBridgeMut<'a> {
    state: &'a mut DungeonEnvironmentState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonEnvironmentBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonEnvironmentState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonEnvironmentState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_water_transition_counter(&mut self, value: u8) {
        self.state.set_water_transition_counter(value);
        self.sync();
    }

    pub(crate) fn increment_water_transition_counter(&mut self) -> u8 {
        let value = self.state.increment_water_transition_counter();
        self.sync();
        value
    }

    pub(crate) fn decrement_water_transition_counter(&mut self) -> u8 {
        let value = self.state.decrement_water_transition_counter();
        self.sync();
        value
    }

    pub(crate) fn set_water_hdma_y_radius(&mut self, value: u16) {
        self.state.set_water_hdma_y_radius(value);
        self.sync();
    }

    pub(crate) fn set_water_hdma_x_radius(&mut self, value: u16) {
        self.state.set_water_hdma_x_radius(value);
        self.sync();
    }

    pub(crate) fn set_water_hdma_y_target(&mut self, value: u16) {
        self.state.set_water_hdma_y_target(value);
        self.sync();
    }

    pub(crate) fn set_water_hdma_y_radius_alt(&mut self, value: u16) {
        self.state.set_water_hdma_y_radius_alt(value);
        self.sync();
    }

    pub(crate) fn set_water_window_position(&mut self, x: u16, y: u16) {
        self.state.set_water_window_position(x, y);
        self.sync();
    }

    pub(crate) fn clear_water_puzzle_state_changed(&mut self) {
        self.state.clear_water_puzzle_state_changed();
        self.sync();
    }

    pub(crate) fn set_water_puzzle_state_changed(&mut self, value: u8) {
        self.state.set_water_puzzle_state_changed(value);
        self.sync();
    }

    pub(crate) fn increment_water_puzzle_state_changed(&mut self) -> u8 {
        let value = self.state.increment_water_puzzle_state_changed();
        self.sync();
        value
    }

    pub(crate) fn set_trapdoors_down(&mut self, value: u16) {
        self.state.set_trapdoors_down(value);
        self.sync();
    }

    pub(crate) fn clear_trapdoors_down(&mut self) {
        self.state.clear_trapdoors_down();
        self.sync();
    }

    pub(crate) fn set_trapdoors_down_low(&mut self, value: u8) {
        self.state.set_trapdoors_down_low(value);
        self.sync();
    }

    pub(crate) fn increment_trapdoors_down_low(&mut self) -> u8 {
        let value = self.state.increment_trapdoors_down_low();
        self.sync();
        value
    }

    pub(crate) fn clear_somaria_block_switch_counter(&mut self) {
        self.state.clear_somaria_block_switch_counter();
        self.sync();
    }

    pub(crate) fn increment_somaria_block_switch_counter(&mut self) {
        self.state.increment_somaria_block_switch_counter();
        self.sync();
    }

    pub(crate) fn increment_somaria_block_bg_check_flag(&mut self) {
        self.state.increment_somaria_block_bg_check_flag();
        self.sync();
    }

    pub(crate) fn clear_orange_blue_barrier_state(&mut self) {
        self.state.clear_orange_blue_barrier_state();
        self.sync();
        // C clears ORANGE_BLUE_BARRIER_STATE with a 16-bit store (Dungeon_LoadEntrance),
        // zeroing the adjacent high byte 0xc173 too. That byte is unmodeled here and is
        // mode-reused as the overworld map16 src-offset backup, so leaving it stale leaks
        // into the overworld decode after a dungeon visit (divergence @rf 46000 → cascade).
        self.ram[ORANGE_BLUE_BARRIER_STATE + 1] = 0;
    }

    pub(crate) fn toggle_orange_blue_barrier_state(&mut self) {
        self.state.toggle_orange_blue_barrier_state();
        self.sync();
    }

    pub(crate) fn clear_moving_floor_check_flags(&mut self) {
        self.state.clear_moving_floor_check_flags();
        self.sync();
    }

    pub(crate) fn or_moving_floor_check_flags(&mut self, bits: u16) -> u16 {
        let flags = self.state.or_moving_floor_check_flags(bits);
        self.sync();
        flags
    }

    pub(crate) fn clear_movable_block_was_pushed(&mut self) {
        self.state.clear_movable_block_was_pushed();
        self.sync();
    }

    pub(crate) fn toggle_movable_block_was_pushed(&mut self) {
        self.state.toggle_movable_block_was_pushed();
        self.sync();
    }

    pub(crate) fn set_block_trap_related_tile(&mut self, value: u16) {
        self.state.set_block_trap_related_tile(value);
        self.sync();
    }
}

pub(crate) struct NativeDungeonRoomTilemapBridgeMut<'a> {
    state: &'a mut DungeonRoomTilemapState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonRoomTilemapBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonRoomTilemapState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    /// Tile-only sync: project the BG1/BG2 tilemaps and re-read the line-pointer
    /// cache from RAM. The line-pointer buffer overlaps the R16/R18 work
    /// registers (0xc8-0xcb), so re-stamping it here would clobber the live
    /// scratch a lift/probe set this frame.
    fn sync(&mut self) {
        self.state.write_tilemaps_to_ram(self.ram);
        self.state.reload_line_pointers_from_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    /// Sync for the dedicated line-pointer setters: project tilemaps AND the
    /// line-pointer bytes (0xbf-0xdf), matching the C room-draw site that writes
    /// those bytes over the overlapping work registers at draw time.
    fn sync_with_line_pointers(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonRoomTilemapState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_bg1_tile(&mut self, index: usize, value: u16) {
        self.state.set_bg1_tile(index, value);
        self.sync();
    }

    pub(crate) fn set_bg2_tile(&mut self, index: usize, value: u16) {
        self.state.set_bg2_tile(index, value);
        self.sync();
    }

    pub(crate) fn set_bg1_tile_by_byte_pos(&mut self, pos: u16, value: u16) {
        self.state.set_bg1_tile_by_byte_pos(pos, value);
        self.sync();
    }

    pub(crate) fn set_bg2_tile_by_byte_pos(&mut self, pos: u16, value: u16) {
        self.state.set_bg2_tile_by_byte_pos(pos, value);
        self.sync();
    }

    pub(crate) fn set_room_tilemap_word(&mut self, base: usize, dsto: u16, value: u16) {
        match base {
            DUNG_BG1 | DUNG_BG2 => {
                self.state.set_room_tilemap_word(base, dsto, value);
                self.sync();
            }
            _ => {
                write_le_u16(self.ram, base + dsto as usize * 2, value);
                *self.state = DungeonRoomTilemapState::load_from_ram(self.ram);
            }
        }
    }

    pub(crate) fn set_room_tilemap_word_by_byte_offset(
        &mut self,
        base: usize,
        byte_offset: usize,
        value: u16,
    ) {
        match base {
            DUNG_BG1 | DUNG_BG2 => {
                self.state
                    .set_room_tilemap_word_by_byte_offset(base, byte_offset, value);
                self.sync();
            }
            _ => {
                write_le_u16(self.ram, base + byte_offset, value);
                *self.state = DungeonRoomTilemapState::load_from_ram(self.ram);
            }
        }
    }

    pub(crate) fn set_line_pointer_row0(&mut self, index: usize, value: u16) {
        self.state.set_line_pointer_row0(index, value);
        self.sync_with_line_pointers();
    }

    pub(crate) fn copy_line_pointer_bytes(&mut self, offsets: &[u8]) {
        self.state.copy_line_pointer_bytes(offsets);
        self.sync_with_line_pointers();
    }

    pub(crate) fn copy_bg2_draw_line_offsets(&mut self) {
        self.state.copy_bg2_draw_line_offsets();
        self.sync_with_line_pointers();
    }

    pub(crate) fn copy_bg1_draw_line_offsets(&mut self) {
        self.state.copy_bg1_draw_line_offsets();
        self.sync_with_line_pointers();
    }
}

pub(crate) struct NativeDungeonRoomItemBridgeMut<'a> {
    state: &'a mut DungeonRoomItemState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonRoomItemBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonRoomItemState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DungeonRoomItemState::load_from_ram(self.ram));
    }

    /// Reset the chest / big-key-lock counters at room load (C: dung_num_chests_x2
    /// = dung_num_bigkey_locks_x2 = 0). The clear_room_parser_words loop only
    /// zeroes RAM; these native fields would otherwise stay stale and miscount.
    pub(crate) fn clear_item_counts(&mut self) {
        self.state.set_num_chests_x2(0);
        self.state.set_num_big_key_locks_x2(0);
        self.sync();
    }

    pub(crate) fn append_chest_location_and_sync_big_key_count(&mut self, value: u16) -> usize {
        let index = self
            .state
            .append_chest_location_and_sync_big_key_count(value);
        self.sync();
        index
    }

    pub(crate) fn advance_chest_and_big_key_counts(&mut self) -> usize {
        let index = self.state.advance_chest_and_big_key_counts();
        self.sync();
        index
    }

    pub(crate) fn advance_big_key_lock_count(&mut self) -> usize {
        let index = self.state.advance_big_key_lock_count();
        self.sync();
        index
    }

    pub(crate) fn set_chest_location(&mut self, index: usize, value: u16) {
        self.state.set_chest_location(index, value);
        self.sync();
    }

    pub(crate) fn set_chest_location_for_offset_x2(&mut self, offset_x2: usize, value: u16) {
        self.state
            .set_chest_location_for_offset_x2(offset_x2, value);
        self.sync();
    }

    pub(crate) fn set_chest_reveal_cursor_x2(&mut self, value: u16) {
        self.state.set_chest_reveal_cursor_x2(value);
        self.sync();
    }

    pub(crate) fn clear_chest_reveal_cursor(&mut self) {
        self.state.clear_chest_reveal_cursor();
        self.sync();
    }

    pub(crate) fn set_replacement_tile_destination_x2(&mut self, value: u16) {
        self.state.set_replacement_tile_destination_x2(value);
        self.sync();
    }

    pub(crate) fn set_replacement_tile_source_x2(&mut self, value: u16) {
        self.state.set_replacement_tile_source_x2(value);
        self.sync();
    }

    pub(crate) fn clear_replacement_tile_destination(&mut self) {
        self.state.clear_replacement_tile_destination();
        self.sync();
    }

    pub(crate) fn clear_chest_location(&mut self, index: usize) {
        self.state.clear_chest_location(index);
        self.sync();
    }

    pub(crate) fn set_replacement_tilemap_quad(&mut self, index: usize, quad: [u16; 4]) {
        self.state.set_replacement_tilemap_quad(index, quad);
        self.sync();
    }
}

pub(crate) struct NativeDungeonRoomEffectsBridgeMut<'a> {
    state: &'a mut DungeonRoomEffectsState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonRoomEffectsBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonRoomEffectsState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonRoomEffectsState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn fill_moving_wall_replacement_buffer(&mut self, value: u16) {
        self.state.fill_moving_wall_replacement_buffer(value);
        self.sync();
    }

    pub(crate) fn set_moving_wall_write_point(&mut self, value: u16) {
        self.state.set_moving_wall_write_point(value);
        self.sync();
    }

    pub(crate) fn set_moving_wall_dot_pointer(&mut self, value: u8) {
        self.state.set_moving_wall_dot_pointer(value);
        self.sync();
    }

    pub(crate) fn set_fixed_color_plusminus(&mut self, value: u8) {
        self.state.set_fixed_color_plusminus(value);
        self.sync();
    }

    /// Set only fixed_color_plusminus and its single owned byte (0xc017), without a full
    /// state sync. Used by the overworld fixed-color setter, which must not re-project the
    /// rest of room_effects (blast-wall/messaging bytes the messaging system owns in that
    /// mode) over their current RAM values.
    pub(crate) fn set_fixed_color_plusminus_value_only(&mut self, value: u8) {
        self.state.set_fixed_color_plusminus(value);
        self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = value;
    }

    pub(crate) fn toggle_moving_wall_torch_blink_phase(&mut self) {
        self.state.toggle_moving_wall_torch_blink_phase();
        self.sync();
    }

    pub(crate) fn request_moving_wall_torch_update(&mut self) {
        self.state.request_moving_wall_torch_update();
        self.sync();
    }

    pub(crate) fn clear_moving_wall_torch_blink_phase(&mut self) {
        self.state.clear_moving_wall_torch_blink_phase();
        self.sync();
    }

    pub(crate) fn set_blast_wall_door_index(&mut self, door: usize) {
        self.state.set_blast_wall_door_index(door);
        self.sync();
    }

    pub(crate) fn clear_blast_wall_door_index(&mut self) {
        self.state.clear_blast_wall_door_index();
        self.sync();
    }

    pub(crate) fn mark_blast_wall_x_open(&mut self) {
        self.state.mark_blast_wall_x_open();
        self.sync();
    }

    pub(crate) fn mark_blast_wall_y_open(&mut self) {
        self.state.mark_blast_wall_y_open();
        self.sync();
    }

    pub(crate) fn set_crush_wall_progress(&mut self, value: u16) {
        self.state.set_crush_wall_progress(value);
        self.sync();
    }

    pub(crate) fn set_crush_wall_progress_low(&mut self, value: u8) {
        self.state.set_crush_wall_progress_low(value);
        self.sync();
    }

    pub(crate) fn increment_trap_trigger_latch(&mut self) {
        self.state.increment_trap_trigger_latch();
        self.sync();
    }

    pub(crate) fn mark_trap_trigger_latched(&mut self) {
        self.state.mark_trap_trigger_latched();
        self.sync();
    }

    pub(crate) fn clear_trap_trigger_latch(&mut self) {
        self.state.clear_trap_trigger_latch();
        self.sync();
    }

    pub(crate) fn set_blast_wall_message_direction(&mut self, value: u16) {
        self.state.set_blast_wall_message_direction(value);
        self.sync();
    }

    pub(crate) fn set_blast_wall_message_position(&mut self, x: u16, y: u16) {
        self.state.set_blast_wall_message_position(x, y);
        self.sync();
    }

    pub(crate) fn set_activate_bomb_trap_overlord(&mut self, value: u8) {
        self.state.set_activate_bomb_trap_overlord(value);
        self.sync();
    }
}

pub(crate) struct NativeDungeonRoomParserBridgeMut<'a> {
    state: &'a mut DungeonRoomParserState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonRoomParserBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonRoomParserState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DungeonRoomParserState::load_from_ram(self.ram));
    }

    pub(crate) fn append_star_switch_tile(&mut self, tilemap_pos: u16) -> usize {
        let index = self.state.append_star_switch_tile(tilemap_pos);
        self.sync();
        index
    }

    pub(crate) fn clear_room_parser_words(&mut self, offsets: &[usize]) {
        for &offset in offsets {
            write_le_u16(self.ram, offset, 0);
        }
        *self.state = DungeonRoomParserState::load_from_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mark_pot_revealed_in_room(&mut self, room: usize, mask: u16) -> u16 {
        let revealed = self.state.mark_pot_revealed_in_room(room, mask);
        self.sync();
        revealed
    }

    pub(crate) fn clear_pot_reveal_masks(&mut self) {
        self.state.clear_pot_reveal_masks();
        self.sync();
    }

    pub(crate) fn append_toggle_palace_pos(&mut self, pos: u16) -> usize {
        let index = self.state.append_toggle_palace_pos(pos);
        self.sync();
        index
    }

    pub(crate) fn append_toggle_floor_pos(&mut self, pos: u16) -> usize {
        let index = self.state.append_toggle_floor_pos(pos);
        self.sync();
        index
    }

    pub(crate) fn copy_custom_tile_attrs(&mut self, attrs: &[u8]) {
        self.state.copy_custom_tile_attrs(attrs);
        self.sync();
    }

    pub(crate) fn copy_default_tile_attrs_tail(&mut self, attrs: &[u8]) {
        self.state.copy_default_tile_attrs_tail(attrs);
        self.sync();
    }

    pub(crate) fn copy_default_tile_attrs_head(&mut self, data: &[u8]) {
        self.state.copy_default_tile_attrs_head(data);
        self.sync();
    }

    pub(crate) fn set_floor_1_filler_high(&mut self, value: u8) {
        self.state.set_floor_1_filler_high(value);
        self.sync();
    }

    pub(crate) fn set_floor_2_filler_high(&mut self, value: u8) {
        self.state.set_floor_2_filler_high(value);
        self.sync();
    }

    pub(crate) fn set_floor_1_filler_low(&mut self, value: u8) {
        self.state.set_floor_1_filler_low(value);
        self.sync();
    }

    pub(crate) fn set_floor_2_filler_low(&mut self, value: u8) {
        self.state.set_floor_2_filler_low(value);
        self.sync();
    }

    pub(crate) fn set_room_layout_and_starting_quadrant(&mut self, value: u16) {
        self.state.set_room_layout_and_starting_quadrant(value);
        self.sync();
    }
}

pub(crate) struct NativeDungeonRoomDoorSetupBridgeMut<'a> {
    state: &'a mut DungeonRoomDoorSetupState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonRoomDoorSetupBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonRoomDoorSetupState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonRoomDoorSetupState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_invisible_door_marker(&mut self) {
        self.state.clear_invisible_door_marker();
        self.sync();
    }

    pub(crate) fn set_invisible_door_marker(&mut self, slot: usize, direction: u16) {
        self.state.set_invisible_door_marker(slot, direction);
        self.sync();
    }

    pub(crate) fn clear_exit_door_count_and_flags(&mut self) {
        self.state.clear_exit_door_count_and_flags();
        self.sync();
    }

    pub(crate) fn append_exit_door_address(&mut self, address: u16) -> usize {
        let index = self.state.append_exit_door_address(address);
        self.sync();
        index
    }

    pub(crate) fn set_active_room_load_ptr(&mut self, value: u16) {
        self.state.set_active_room_load_ptr(value);
        self.sync();
    }

    pub(crate) fn set_active_room_load_ptr_bank(&mut self, value: u8) {
        self.state.set_active_room_load_ptr_bank(value);
        self.sync();
    }

    pub(crate) fn add_reset_xy_check_flags(&mut self, value: u16) -> u16 {
        let flags = self.state.add_reset_xy_check_flags(value);
        self.sync();
        flags
    }

    pub(crate) fn set_adjacent_door_flags(&mut self, value: u16) {
        self.state.set_adjacent_door_flags(value);
        self.sync();
    }

    pub(crate) fn mark_adjacent_door_flag(&mut self, index: usize) -> u16 {
        let flags = self.state.mark_adjacent_door_flag(index);
        self.sync();
        flags
    }

    pub(crate) fn set_adjacent_door(&mut self, index: usize, value: u16) {
        self.state.set_adjacent_door(index, value);
        self.sync();
    }

    pub(crate) fn load_adjacent_doors_from_room_info(&mut self, door_info: &[u8]) {
        self.state.load_adjacent_doors_from_room_info(door_info);
        self.sync();
    }

    pub(crate) fn mark_no_adjacent_doors(&mut self) {
        self.state.mark_no_adjacent_doors();
        self.sync();
    }
}

pub(crate) struct NativeDungeonRoomRuntimeBridgeMut<'a> {
    state: &'a mut DungeonRoomRuntimeState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonRoomRuntimeBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonRoomRuntimeState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonRoomRuntimeState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_dungeon_music_type_flag(&mut self) {
        self.state.clear_dungeon_music_type_flag();
        self.sync();
    }

    pub(crate) fn set_dungeon_music_type_flag(&mut self, value: u8) {
        self.state.set_dungeon_music_type_flag(value);
        self.sync();
    }

    pub(crate) fn set_room_index_x3(&mut self, value: u16) {
        self.state.set_room_index_x3(value);
        self.sync();
    }

    pub(crate) fn clear_reserved_gfx_config(&mut self) {
        self.state.clear_reserved_gfx_config();
        self.sync();
    }

    pub(crate) fn skip_room_tags_once(&mut self) {
        self.state.skip_room_tags_once();
        self.sync();
    }

    pub(crate) fn clear_room_tag_skip(&mut self) {
        self.state.clear_room_tag_skip();
        self.sync();
    }

    pub(crate) fn set_landing_class(&mut self, value: u8) {
        self.state.set_landing_class(value);
        self.sync();
    }

    pub(crate) fn clear_landing_class(&mut self) {
        self.state.clear_landing_class();
        self.sync();
    }
}

pub(crate) struct NativeDungeonMovableBlockBridgeMut<'a> {
    state: &'a mut DungeonMovableBlockState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonMovableBlockBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonMovableBlockState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonMovableBlockState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_movable_block_record(&mut self, index: usize, room: u16, tilemap: u16) {
        self.state.set_movable_block_record(index, room, tilemap);
        self.sync();
    }

    pub(crate) fn copy_records_from_bytes(&mut self, data: &[u8]) {
        self.state.copy_records_from_bytes(data);
        self.sync();
    }
}

pub(crate) struct NativeDungeonStairMovementBridgeMut<'a> {
    state: &'a mut DungeonStairMovementState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonStairMovementBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonStairMovementState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonStairMovementState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_current_floor(&mut self, value: u8) {
        self.state.set_current_floor(value);
        self.sync();
    }

    pub(crate) fn decrement_current_floor(&mut self) -> u8 {
        let value = self.state.decrement_current_floor();
        self.sync();
        value
    }

    pub(crate) fn increment_current_floor(&mut self) -> u8 {
        let value = self.state.increment_current_floor();
        self.sync();
        value
    }

    pub(crate) fn cache_current_floor(&mut self) {
        self.state.cache_current_floor();
        self.sync();
    }

    pub(crate) fn restore_cached_floor(&mut self) {
        self.state.restore_cached_floor();
        self.sync();
    }

    pub(crate) fn set_staircase_tilemap_pos_x2(&mut self, value: u16) {
        self.state.set_staircase_tilemap_pos_x2(value);
        self.sync();
    }

    pub(crate) fn set_current_staircase_plane(&mut self, value: u8) {
        self.state.set_current_staircase_plane(value);
        self.sync();
    }

    pub(crate) fn set_staircase_lower_level_status(&mut self, value: u8) {
        self.state.set_staircase_lower_level_status(value);
        self.sync();
    }

    pub(crate) fn set_staircase_countdown(&mut self, value: u8) {
        self.state.set_staircase_countdown(value);
        self.sync();
    }

    pub(crate) fn decrement_staircase_countdown_clamped(&mut self) -> u8 {
        let value = self.state.decrement_staircase_countdown_clamped();
        self.sync();
        value
    }

    pub(crate) fn decrement_staircase_countdown_underflowed(&mut self) -> bool {
        let underflowed = self.state.decrement_staircase_countdown_underflowed();
        self.sync();
        underflowed
    }

    pub(crate) fn set_staircase_index(&mut self, value: u8) {
        self.state.set_staircase_index(value);
        self.sync();
    }

    pub(crate) fn set_staircase_index_high(&mut self, value: u8) {
        self.state.set_staircase_index_high(value);
        self.sync();
    }

    pub(crate) fn set_staircase_move_counter(&mut self, value: u8) {
        self.state.set_staircase_move_counter(value);
        self.sync();
    }

    pub(crate) fn decrement_staircase_move_counter(&mut self) -> u8 {
        let value = self.state.decrement_staircase_move_counter();
        self.sync();
        value
    }

    pub(crate) fn set_kind_of_in_room_staircase_word(&mut self, value: u16) {
        self.state.set_kind_of_in_room_staircase_word(value);
        self.sync();
    }
}

pub(crate) struct NativeDungeonTorchBridgeMut<'a> {
    torch: &'a mut DungeonTorchState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonTorchBridgeMut<'a> {
    pub(crate) fn new(torch: &'a mut DungeonTorchState, ram: &'a mut [u8]) -> Self {
        Self { torch, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.torch, DungeonTorchState::load_from_ram(self.ram));
    }

    pub(crate) fn copy_torch_init_to_movable_blocks(&mut self, torch_init: &[u8]) {
        self.ram[MOVABLE_BLOCK_DATAS + 99 * 4..MOVABLE_BLOCK_DATAS + 99 * 4 + 116]
            .copy_from_slice(&torch_init[..116]);
    }

    pub(crate) fn copy_torch_data_table(&mut self, torch_init: &[u8]) {
        self.ram[DUNGEON_TORCH_DATA..DUNGEON_TORCH_DATA + torch_init.len()]
            .copy_from_slice(torch_init);
        *self.torch = DungeonTorchState::load_from_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn copy_torch_junk(&mut self, torch_junk: &[u8]) {
        self.ram[DUNGEON_TORCH_DATA + 144 * 2..DUNGEON_TORCH_DATA + 144 * 2 + torch_junk.len()]
            .copy_from_slice(torch_junk);
        *self.torch = DungeonTorchState::load_from_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_timer(&mut self, index: usize) {
        self.torch.clear_timer(index);
        if index < DUNGEON_TORCH_TIMER_COUNT {
            self.ram[TORCH_TIMERS + index] = 0;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_timers(&mut self) {
        self.torch.clear_timers();
        self.ram[TORCH_TIMERS..TORCH_TIMERS + DUNGEON_TORCH_TIMER_COUNT].fill(0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_timer(&mut self, index: usize, value: u8) {
        self.torch.set_timer(index, value);
        if index < DUNGEON_TORCH_TIMER_COUNT {
            self.ram[TORCH_TIMERS + index] = value;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_lit_torches(&mut self) {
        self.torch.clear_lit_torches();
        self.ram[DUNG_NUM_LIT_TORCHES] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_lit_torches(&mut self, value: u8) {
        self.torch.set_lit_torches(value);
        self.ram[DUNG_NUM_LIT_TORCHES] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_lit_torches(&mut self) -> u8 {
        let value = self.torch.increment_lit_torches();
        self.ram[DUNG_NUM_LIT_TORCHES] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn decrement_lit_torches(&mut self) -> u8 {
        let value = self.torch.decrement_lit_torches();
        self.ram[DUNG_NUM_LIT_TORCHES] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_lights_out_request(&mut self, value: u8) {
        self.torch.set_lights_out_request(value);
        self.ram[DUNG_WANT_LIGHTS_OUT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_lights_out_request(&mut self) {
        self.torch.clear_lights_out_request();
        self.ram[DUNG_WANT_LIGHTS_OUT] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_lights_out_request_copy(&mut self, value: u8) {
        self.torch.set_lights_out_request_copy(value);
        self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn copy_lights_out_request(&mut self) {
        self.torch.copy_lights_out_request();
        self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = self.ram[DUNG_WANT_LIGHTS_OUT];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_lights_out_requests(&mut self) {
        self.torch.clear_lights_out_requests();
        self.ram[DUNG_WANT_LIGHTS_OUT] = 0;
        self.ram[DUNG_WANT_LIGHTS_OUT_COPY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_dungeon_dark_with_lantern(&mut self) {
        self.torch.set_dungeon_dark_with_lantern();
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_dungeon_dark_with_lantern_raw(&mut self, value: u8) {
        self.torch.set_dungeon_dark_with_lantern_raw(value);
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_dungeon_dark_with_lantern(&mut self) {
        self.torch.clear_dungeon_dark_with_lantern();
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_torch_index_range_start(&mut self, value: u16) {
        self.torch.refresh_object_data_positions_from_ram(self.ram);
        self.torch.set_torch_index_range_start(value);
        write_le_u16(self.ram, DUNG_INDEX_OF_TORCHES_START, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_torch_index(&mut self, value: u16) {
        self.torch.refresh_object_data_positions_from_ram(self.ram);
        self.torch.set_torch_index(value);
        write_le_u16(self.ram, DUNG_INDEX_OF_TORCHES, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_torch_data_word_index(&mut self, index: usize, value: u16) {
        self.torch.set_torch_data_word_index(index, value);
        write_le_u16(self.ram, DUNGEON_TORCH_DATA + index * 2, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_attr(&mut self, value: u8) {
        self.torch.set_attr(value);
        self.ram[DUNGEON_TORCH_ATTR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_ganon_torch_count(&mut self, value: u8) {
        self.torch.set_ganon_torch_count(value);
        self.ram[GANON_TORCH_COUNT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_attr(&mut self) {
        self.torch.clear_attr();
        self.ram[DUNGEON_TORCH_ATTR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn refresh_object_data_positions(&mut self) {
        *self.torch = DungeonTorchState::load_from_ram(self.ram);
    }
}

pub(crate) struct NativeDungeonHeaderBridgeMut<'a> {
    header: &'a mut DungeonHeaderState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonHeaderBridgeMut<'a> {
    pub(crate) fn new(header: &'a mut DungeonHeaderState, ram: &'a mut [u8]) -> Self {
        Self { header, ram }
    }

    fn sync(&mut self) {
        self.header.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.header, DungeonHeaderState::load_from_ram(self.ram));
    }

    pub(crate) fn set_hole_teleporter_planes(&mut self, packed: u8, extra: u8) {
        self.header.set_hole_teleporter_planes(packed, extra);
        self.sync();
    }

    pub(crate) fn set_header_tag(&mut self, index: usize, value: u8) {
        self.header.set_header_tag(index, value);
        self.sync();
    }

    pub(crate) fn copy_travel_destinations_from_header(&mut self, header: &[u8]) {
        self.header.copy_travel_destinations_from_header(header);
        self.sync();
    }

    pub(crate) fn clear_header_tag(&mut self, index: usize) {
        self.header.clear_header_tag(index);
        self.sync();
    }

    pub(crate) fn clear_header_tags(&mut self, count: usize) {
        self.header.clear_header_tags(count);
        self.sync();
    }
}
