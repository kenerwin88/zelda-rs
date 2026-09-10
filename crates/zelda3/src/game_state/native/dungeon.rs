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
    FLOOR_1_FILLER_TILES, FLOOR_2_FILLER_TILES, GANON_TORCH_COUNT, HDR_DUNGEON_DARK_WITH_LANTERN,
    INVISIBLE_DOOR_DIR_AND_INDEX_X2, MAIN_TILE_THEME_INDEX, MOVABLE_BLOCK_DATAS,
    MOVING_FLOOR_BG_CHECK_FLAGS, MOVING_WALL_DOT_POINTER, MOVING_WALL_REPLACEMENT_BUFFER,
    MOVING_WALL_WRITE_POINT, ORANGE_BLUE_BARRIER_STATE, STAR_TILE_PHASE,
    OVERWORLD_EXIT_TILE_THEME_INDEX, OVERWORLD_FIXED_COLOR_PLUSMINUS,
    OVERWORLD_TILE_THEME_INDEX, REPLACEMENT_TILEMAP_LL, REPLACEMENT_TILEMAP_LR,
    REPLACEMENT_TILEMAP_UL, REPLACEMENT_TILEMAP_UR, RESERVED_GFX_CONFIG_WORD, RESET_XY_CHECK_FLAGS,
    SOMARIA_BLOCK_BG_CHECK_FLAG, SPRITE_GRAPHICS_INDEX, TORCH_TIMERS,
    WATER_SIDE_STEP_SWITCH,
};
use crate::game_state::constants::{
    COUNTDOWN_TIMER_FOR_STAIRCASES, CUR_STAIRCASE_PLANE, KIND_OF_IN_ROOM_STAIRCASE,
    STAIRCASE_LOWER_LEVEL_STATUS, STAIRCASE_MOVE_COUNTER, STAIRCASE_TILEMAP_POS_X2,
    WHICH_STAIRCASE_INDEX,
};
use crate::game_state::constants::{
    DUNGEON_ROOM_HISTORY, DUNGEON_ROOM_INDEX2, DUNGEON_ROOM_INDEX_PREV,
};
use crate::game_state::native::ram_target::RamTarget;
use crate::tile_definition::{DungeonRole, NativeTile};
use crate::types::{read_le_u16, write_le_u16};

const DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT: usize = 5;
const DUNGEON_HEADER_PLANE_SCRATCH_COUNT: usize = 5;
const DUNGEON_HEADER_TAG_COUNT: usize = 2;
const DUNGEON_TORCH_TIMER_COUNT: usize = 16;
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
const DUNGEON_ADJACENT_DOOR_COUNT: usize = 9;
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
const DUNGEON_STAIR_TABLE_2_WORDS: usize = (DUNG_STAIRS_TABLE_2_END - DUNG_STAIRS_TABLE_2) / 2;

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
/// End of dung_stairs_table_2 in the C port's layout (where it placed
/// door_debris_x; the ROM keeps door debris at $03B6). Only a table bound.
const DUNG_STAIRS_TABLE_2_END: usize = 0x0728;

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
        // scratch_word (R16/R18) is write-through and shares its SNES bytes with code
        // that writes RAM directly (gfx converter, select-file), so the persisted native
        // legitimately lags RAM between bridge setters — not a coherence bug.
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        self.header.write_to_ram(ram);
        // NOTE: scratch_word (R16/R18, 0xc8-0xcb) is intentionally NOT bulk-projected
        // here. Those bytes are written directly by code that shares them (the gfx
        // converter's DUNG_LINE_PTRS_ROW0 scratch, select-file R17), so re-stamping the
        // native every frame would clobber that live value (e.g. the ending credits at
        // frame 1025036 and select-file at frame 1003). The scratch-word bridge is
        // write-through: each setter re-reads RAM, mutates, and writes only its own
        // bytes back (see NativeDungeonScratchWordBridgeMut).
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        self.write_tilemaps_to_ram(ram);
        self.write_line_pointers_to_ram(ram);
    }

    /// Project only the BG1/BG2 tilemaps. The line-pointer buffer
    /// (`DUNG_LINE_PTRS_ROW0` = 0xbf, 33 bytes → 0xbf-0xdf) overlaps the dungeon
    /// work registers R16/R18 (0xc8-0xcb) and the intro-sword bytes by SNES byte
    /// reuse, so it must NOT be re-stamped on every tile write — that would
    /// clobber the live scratch a lifted-tile/probe just set.
    pub(crate) fn write_tilemaps_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        for (index, tile) in self.bg1_tiles.iter().enumerate() {
            ram.write_word(DUNG_BG1 + index * 2, *tile);
        }
        for (index, tile) in self.bg2_tiles.iter().enumerate() {
            ram.write_word(DUNG_BG2 + index * 2, *tile);
        }
    }

    /// Project the line-pointer buffer (0xbf-0xdf). C writes these bytes only
    /// when room draw actually computes line pointers, so only the dedicated
    /// line-pointer setters call this — matching the C site that overwrites the
    /// overlapping work registers at exactly that moment.
    pub(crate) fn write_line_pointers_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        for (index, byte) in self.line_pointer_bytes.iter().enumerate() {
            ram.write_byte(DUNG_LINE_PTRS_ROW0 + index, *byte);
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_byte(
            DUNG_FLAG_STATECHANGE_WATERPUZZLE,
            self.water_puzzle_state_changed,
        );
        ram.write_word(DUNG_FLAG_TRAPDOORS_DOWN, self.trapdoors_down);
        ram.write_byte(
            DUNG_FLAG_SOMARIA_BLOCK_SWITCH,
            self.somaria_block_switch_counter,
        );
        ram.write_byte(
            SOMARIA_BLOCK_BG_CHECK_FLAG,
            self.somaria_block_bg_check_flag,
        );
        ram.write_byte(ORANGE_BLUE_BARRIER_STATE, self.orange_blue_barrier_state);
        ram.write_word(MOVING_FLOOR_BG_CHECK_FLAGS, self.moving_floor_check_flags);
        ram.write_byte(
            DUNG_FLAG_MOVABLE_BLOCK_WAS_PUSHED,
            self.movable_block_was_pushed,
        );
        ram.write_word(BLOCK_TRAP_CHECK_FLAG, self.block_trap_related_tile);
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

    pub(crate) fn somaria_block_bg_check_flag(&self) -> u8 {
        self.somaria_block_bg_check_flag
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

    fn clear_somaria_block_bg_check_flag(&mut self) {
        self.somaria_block_bg_check_flag = 0;
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_byte(DUNG_HDR_COLLISION, self.header_collision);
        ram.write_byte(DUNG_HDR_COLLISION_2, self.header_collision_2);
        ram.write_word(
            DUNGEON_HEADER_COLLISION_2_MIRROR,
            self.header_collision_2_mirror,
        );
        ram.write_byte(DUNG_HDR_BG2_PROPERTIES, self.bg2_properties);
        ram.write_byte(DUNG_HDR_BG2_PROPERTIES_BACKUP, self.bg2_properties_backup);
        ram.write_byte(COMPOSITE_OF_LAYOUT_AND_QUADRANT, self.layout_quadrant_key);
        ram.write_word(DUNG_QUADRANTS_VISITED, self.quadrants_visited);
        ram.write_byte(DUNG_CUR_QUADRANT_UPLOAD, self.quadrant_upload_index);
        ram.write_word(DUNG_DRAW_WIDTH_INDICATOR, self.draw_width_indicator);
        ram.write_word(DUNG_DRAW_HEIGHT_INDICATOR, self.draw_height_indicator);
        ram.write_byte(DUNG_OVERLAY_TO_LOAD, self.overlay_to_load);
        ram.write_word(DUNG_WHICH_KEY_X2_DUNGEON, self.selected_key_door_x2);
        ram.write_word(DUNG_LOAD_PTR_OFFS, self.load_ptr_offset);
        ram.write_word(DUNG_LOADE_BGOFFS_H_COPY, self.loading_bg_offset_h);
        ram.write_word(DUNG_LOADE_BGOFFS_V_COPY, self.loading_bg_offset_v);
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

    fn set_selected_key_door(&mut self, door: usize) {
        self.selected_key_door_x2 = (door * 2) as u16;
    }

    fn set_load_ptr_offset(&mut self, value: u16) {
        self.load_ptr_offset = value;
    }

    fn set_loading_bg_offsets(&mut self, horizontal: u16, vertical: u16) {
        self.loading_bg_offset_h = horizontal;
        self.loading_bg_offset_v = vertical;
    }
}

/// One dungeon object's replacement record. The room draw registers the
/// object's kind (a liftable kind, a segment of a big rock or bombable
/// floor, a hammer peg); a push block instead counts through named phases.
/// The word layout is the original's, so a record survives WRAM and
/// checkpoints unchanged; the queries keep the original's masks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ObjectRecord(u16);

impl ObjectRecord {
    pub(crate) const IDLE_PUSH_BLOCK: Self = Self(0);
    pub(crate) const PUSHED: Self = Self(1);
    pub(crate) const SLIDING: Self = Self(2);
    pub(crate) const ARRIVED: Self = Self(3);
    pub(crate) const FALLING: Self = Self(4);
    pub(crate) const RESTING_ON_PLATE: Self = Self(5);
    /// A block that dropped through a hole; the handler's next advance wraps
    /// it back to idle, as the original's increment did.
    pub(crate) const VANISHED: Self = Self(0xffff);
    pub(crate) const HAMMER_PEG: Self = Self(0x4040);
    pub(crate) const POT: Self = Self::liftable(1);

    pub(crate) const fn liftable(kind: usize) -> Self {
        Self(0x1010 + (kind as u16 & 0x0f) * 0x0101)
    }

    pub(crate) const fn big_rock_segment(segment: usize) -> Self {
        Self(0x2020 + (segment as u16 & 0x0f) * 0x0101)
    }

    pub(crate) const fn bombable_floor_segment(segment: usize) -> Self {
        Self(0x3030 + (segment as u16 & 0x0f) * 0x0101)
    }

    pub(crate) const fn from_word(word: u16) -> Self {
        Self(word)
    }

    pub(crate) const fn word(self) -> u16 {
        self.0
    }

    pub(crate) fn liftable_kind(self) -> Option<usize> {
        (self.0 & 0xf0f0 == 0x1010).then_some(self.kind_index())
    }

    pub(crate) fn big_rock_segment_index(self) -> Option<usize> {
        (self.0 & 0xf0f0 == 0x2020).then_some(self.kind_index())
    }

    pub(crate) fn is_hammer_peg(self) -> bool {
        self.0 & 0xf0f0 == 0x4040
    }

    /// The kind or segment nibble, read without a family check where the
    /// original did not check one.
    pub(crate) const fn kind_index(self) -> usize {
        (self.0 & 0x0f) as usize
    }

    /// Room attribute loading skips bombable floor segments; the original
    /// tests only the low byte here.
    pub(crate) fn skips_room_attribute(self) -> bool {
        self.0 & 0x00f0 == 0x0030
    }

    pub(crate) fn is_idle(self) -> bool {
        self.0 == 0
    }

    pub(crate) fn advanced(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// The falling animation's end clears only the low byte.
    pub(crate) fn settled(self) -> Self {
        Self(self.0 & 0xff00)
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(DUNG_MISC_OBJS_INDEX, self.misc_object_index);
        for (index, state) in self.replacement_tile_states.iter().enumerate() {
            ram.write_word(DUNGEON_REPLACEMENT_TILE_STATE + index * 2, *state);
        }
        for (index, position) in self.object_data_positions.iter().enumerate() {
            ram.write_word(DUNG_OBJECT_POS_IN_OBJDATA + index * 2, *position);
        }
        for (index, position) in self.object_tilemap_positions.iter().enumerate() {
            ram.write_word(DUNG_OBJECT_TILEMAP_POS + index * 2, *position);
        }
        for (index, object_index) in self.changeable_object_indices.iter().enumerate() {
            ram.write_byte(CHANGEABLE_DUNGEON_OBJECT_INDEX + index, *object_index);
        }
        ram.write_word(BIG_ROCK_STARTING_ADDRESS, self.big_rock_starting_address);
    }

    pub(crate) fn misc_object_index(&self) -> u16 {
        self.misc_object_index
    }

    pub(crate) fn object_record(&self, index: usize) -> ObjectRecord {
        ObjectRecord::from_word(
            self.replacement_tile_states
                .get(index)
                .copied()
                .unwrap_or(0),
        )
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

    fn clear_replacement_tile_states(&mut self) {
        self.replacement_tile_states.fill(0);
    }

    fn clear_object_data_positions(&mut self) {
        self.object_data_positions.fill(0);
    }

    fn set_object_record(&mut self, index: usize, record: ObjectRecord) {
        if let Some(state) = self.replacement_tile_states.get_mut(index) {
            *state = record.word();
        }
    }

    fn advance_object_record(&mut self, index: usize) {
        let record = self.object_record(index).advanced();
        self.set_object_record(index, record);
    }

    fn settle_object_record(&mut self, index: usize) {
        let record = self.object_record(index).settled();
        self.set_object_record(index, record);
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(DUNG_DOOR_OPENED, self.opened_doors);
        ram.write_word(
            DUNG_DOOR_OPENED_INCL_ADJACENT,
            self.opened_doors_including_adjacent,
        );
        ram.write_word(DUNG_CUR_DOOR_IDX, self.current_door_index);
        ram.write_word(DUNG_CUR_DOOR_POS_DUNGEON, self.current_door_pos);
        ram.write_word(DOOR_ANIMATION_STEP_INDICATOR_DUNGEON, self.animation_step);
        ram.write_word(DOOR_OPEN_CLOSED_COUNTER, self.open_counter);
        for (door, address) in self.door_tilemap_addresses.iter().enumerate() {
            ram.write_word(DUNG_DOOR_TILEMAP_ADDRESS + door * 2, *address);
        }
        for (door, door_type) in self.door_types.iter().enumerate() {
            ram.write_word(DOOR_TYPE_AND_SLOT + door * 2, *door_type);
        }
        for (door, direction) in self.door_directions.iter().enumerate() {
            ram.write_word(DUNGEON_DOOR_DIRECTION + door * 2, *direction);
        }
        ram.write_byte(DUNG_DOOR_SWITCH_TRIGGERED, self.switch_triggered);
        ram.write_word(
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

    fn set_current_door_index(&mut self, value: u16) {
        self.current_door_index = value;
    }

    fn set_current_door_index_for_slot(&mut self, door: usize) {
        self.current_door_index = (door * 2) as u16;
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

    fn set_door_animation_step_low(&mut self, value: u8) {
        self.animation_step = (self.animation_step & 0xff00) | u16::from(value);
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(DUNGEON_ROOM_INDEX2, self.room_index2);
        ram.write_word(DUNGEON_ROOM_INDEX_PREV, self.previous_room_index);
        for (index, entry) in self.history.iter().enumerate() {
            ram.write_word(DUNGEON_ROOM_HISTORY + index * 2, *entry);
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(DUNGEON_FLOOR_Y_VELOCITY, self.y_velocity);
        ram.write_word(DUNGEON_FLOOR_X_VELOCITY, self.x_velocity);
        ram.write_word(DUNG_FLOOR_X_OFFS, self.x_offset);
        ram.write_word(DUNG_FLOOR_Y_OFFS, self.y_offset);
        ram.write_word(DUNG_FLOOR_MOVE_FLAGS, self.move_flags);
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

    /// `turn_on_off_water_ctr` in the original: the water-toggle rooms count
    /// their transition in the low byte of the floor offset word, which no
    /// moving floor uses in those rooms.
    pub(crate) fn water_transition_counter(&self) -> u8 {
        self.y_offset as u8
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

    fn set_floor_x_offset(&mut self, value: u16) {
        self.x_offset = value;
    }

    fn set_floor_y_offset(&mut self, value: u16) {
        self.y_offset = value;
    }

    fn set_floor_y_offset_low(&mut self, value: u8) {
        self.y_offset = (self.y_offset & 0xff00) | u16::from(value);
    }

    fn set_water_transition_counter(&mut self, value: u8) {
        self.set_floor_y_offset_low(value);
    }

    fn increment_water_transition_counter(&mut self) -> u8 {
        let value = self.water_transition_counter().wrapping_add(1);
        self.set_floor_y_offset_low(value);
        value
    }

    fn decrement_water_transition_counter(&mut self) -> u8 {
        let value = self.water_transition_counter().wrapping_sub(1);
        self.set_floor_y_offset_low(value);
        value
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(DUNG_CUR_FLOOR, self.current_floor_word);
        ram.write_byte(DUNG_CUR_FLOOR_CACHED, self.cached_floor);
        ram.write_word(WHICH_STAIRCASE_INDEX, self.staircase_index);
        ram.write_byte(STAIRCASE_MOVE_COUNTER, self.move_counter);
        ram.write_byte(CUR_STAIRCASE_PLANE, self.current_plane);
        ram.write_byte(STAIRCASE_LOWER_LEVEL_STATUS, self.lower_level_status);
        ram.write_word(STAIRCASE_TILEMAP_POS_X2, self.tilemap_pos_x2);
        ram.write_word(KIND_OF_IN_ROOM_STAIRCASE, self.in_room_kind);
        ram.write_byte(COUNTDOWN_TIMER_FOR_STAIRCASES, self.countdown);
    }

    pub(crate) fn current_floor(&self) -> u8 {
        self.current_floor_word as u8
    }

    pub(crate) fn current_floor_word(&self) -> u16 {
        self.current_floor_word
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

    pub(crate) fn staircase_move_counter(&self) -> u8 {
        self.move_counter
    }

    pub(crate) fn kind_of_in_room_staircase(&self) -> u8 {
        self.in_room_kind as u8
    }

    pub(crate) fn staircase_tilemap_pos_x2(&self) -> u16 {
        self.tilemap_pos_x2
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        for (index, record) in self.records.iter().enumerate() {
            let base = MOVABLE_BLOCK_DATAS + index * 4;
            ram.write_word(base, record[0]);
            ram.write_word(base + 2, record[1]);
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
    room_tag_skip_count: u8,
    landing_class: u8,
    room_index_x3: u16,
    reserved_gfx_config_word: u16,
}

impl DungeonRoomRuntimeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            room_tag_skip_count: ram.get(FLAG_SKIP_CALL_TAG_ROUTINES).copied().unwrap_or(0),
            landing_class: ram.get(DUNG_TRANSITION_LANDING_CLASS).copied().unwrap_or(0),
            room_index_x3: read_le_u16(ram, DUNG_INDEX_X3),
            reserved_gfx_config_word: read_le_u16(ram, RESERVED_GFX_CONFIG_WORD),
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_byte(FLAG_SKIP_CALL_TAG_ROUTINES, self.room_tag_skip_count);
        ram.write_byte(DUNG_TRANSITION_LANDING_CLASS, self.landing_class);
        ram.write_word(DUNG_INDEX_X3, self.room_index_x3);
        ram.write_word(RESERVED_GFX_CONFIG_WORD, self.reserved_gfx_config_word);
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(ADJACENT_DOORS_FLAGS, self.adjacent_door_flags);
        for (index, &door) in self.adjacent_doors.iter().enumerate() {
            ram.write_word(ADJACENT_DOORS + index * 2, door);
        }
        ram.write_word(DUNG_EXIT_DOOR_COUNT, self.exit_door_count_x2);
        for (index, &address) in self.exit_door_addresses.iter().enumerate() {
            ram.write_word(DUNG_EXIT_DOOR_ADDRESSES + index * 2, address);
        }
        ram.write_word(INVISIBLE_DOOR_DIR_AND_INDEX_X2, self.invisible_door_marker);
        ram.write_word(DUNG_LOAD_PTR, self.active_room_load_ptr);
        ram.write_byte(DUNG_LOAD_PTR_BANK, self.active_room_load_ptr_bank);
        ram.write_word(DUNG_WIDTH_ROAD_ADDRESS, self.width_road_address);
        ram.write_word(RESET_XY_CHECK_FLAGS, self.reset_xy_check_flags);
    }

    pub(crate) fn adjacent_door_flags(&self) -> u16 {
        self.adjacent_door_flags
    }

    pub(crate) fn adjacent_door(&self, index: usize) -> u16 {
        self.adjacent_doors.get(index).copied().unwrap_or(0xffff)
    }

    pub(crate) fn has_exit_door_address(&self, address: u16) -> bool {
        self.exit_door_addresses.contains(&address)
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

    fn clear_reset_xy_check_flags(&mut self) {
        self.reset_xy_check_flags = 0;
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
    tile_attributes: Vec<NativeTile>,
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
            tile_attributes: vec![NativeTile::GROUND; 0x200],
        }
    }
}

impl DungeonRoomParserState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut star_switch_tilemap_positions = [0; DUNGEON_ROOM_PARSER_SMALL_TABLE_COUNT];
        for (index, pos) in star_switch_tilemap_positions.iter_mut().enumerate() {
            *pos = read_le_u16(ram, STAR_SHAPED_SWITCHES_TILE_LOCAL + index * 2);
        }

        // DUNG_TOGGLE_FLOOR_POS (0x6c0) / DUNG_TOGGLE_PALACE_POS (0x6d0) are mode-reused with the
        // dungeon stair tables (DUNG_STAIRS_TABLE_1/2, owned by DungeonStairListsState). Only the
        // first `count` slots are live toggle positions; the rest are stair-table bytes. Load only
        // the live slots so projecting them back can't re-stamp a stale toggle position over the
        // stair table (f633081).
        let toggle_floor_count_x2 = read_le_u16(ram, DUNG_NUM_TOGGLE_FLOOR);
        let toggle_palace_count_x2 = read_le_u16(ram, DUNG_NUM_TOGGLE_PALACE);
        let mut toggle_floor_positions = [0; DUNGEON_ROOM_TOGGLE_SLOT_COUNT];
        let mut toggle_palace_positions = [0; DUNGEON_ROOM_TOGGLE_SLOT_COUNT];
        let toggle_floor_count = usize::from(toggle_floor_count_x2 / 2);
        for (index, position) in toggle_floor_positions
            .iter_mut()
            .enumerate()
            .take(toggle_floor_count)
        {
            *position = read_le_u16(ram, DUNG_TOGGLE_FLOOR_POS + index * 2);
        }
        let toggle_palace_count = usize::from(toggle_palace_count_x2 / 2);
        for (index, position) in toggle_palace_positions
            .iter_mut()
            .enumerate()
            .take(toggle_palace_count)
        {
            *position = read_le_u16(ram, DUNG_TOGGLE_PALACE_POS + index * 2);
        }

        let mut pot_reveal_masks = vec![0; DUNGEON_POT_REVEAL_ROOM_COUNT];
        for (room, mask) in pot_reveal_masks.iter_mut().enumerate() {
            *mask = read_le_u16(ram, POTS_REVEALED_IN_ROOM_DUNGEON_LOCAL + room * 2);
        }

        // ATTRIBUTES_FOR_TILE owns exactly 0x200 bytes (0xfe00..0x10000). A 0x400 array
        // overran the load/projection into 0x10000-0x101ff (BG_CHAR_BUFFER), re-stamping a
        // stale copy over the star-tile graphics written by Dungeon_RestoreStarTileChr.
        let mut tile_attributes = vec![NativeTile::GROUND; 0x200];
        let available = ram.len().saturating_sub(ATTRIBUTES_FOR_TILE_PLAYER);
        let len = tile_attributes.len().min(available);
        NativeTile::import_slice(
            &mut tile_attributes[..len],
            &ram[ATTRIBUTES_FOR_TILE_PLAYER..ATTRIBUTES_FOR_TILE_PLAYER + len],
        );

        Self {
            star_switch_count_x2: read_le_u16(ram, DUNG_NUM_STAR_SHAPED_SWITCHES_LOCAL),
            star_switch_tilemap_positions,
            toggle_floor_count_x2,
            toggle_palace_count_x2,
            toggle_floor_positions,
            toggle_palace_positions,
            floor_1_filler_tiles: read_le_u16(ram, FLOOR_1_FILLER_TILES),
            floor_2_filler_tiles: read_le_u16(ram, FLOOR_2_FILLER_TILES),
            room_layout_and_starting_quadrant: read_le_u16(ram, DUNG_LAYOUT_AND_STARTING_QUADRANT),
            pot_reveal_masks,
            tile_attributes,
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(
            DUNG_NUM_STAR_SHAPED_SWITCHES_LOCAL,
            self.star_switch_count_x2,
        );
        for (index, &pos) in self.star_switch_tilemap_positions.iter().enumerate() {
            ram.write_word(STAR_SHAPED_SWITCHES_TILE_LOCAL + index * 2, pos);
        }
        ram.write_word(DUNG_NUM_TOGGLE_FLOOR, self.toggle_floor_count_x2);
        ram.write_word(DUNG_NUM_TOGGLE_PALACE, self.toggle_palace_count_x2);
        // Project only the live toggle slots — the rest of 0x6c0/0x6d0 is the mode-reused stair
        // table owned by DungeonStairListsState; re-stamping stale toggle positions there clobbered
        // a fresh stair-table entry (f633081).
        for index in
            0..usize::from(self.toggle_floor_count_x2 / 2).min(DUNGEON_ROOM_TOGGLE_SLOT_COUNT)
        {
            ram.write_word(
                DUNG_TOGGLE_FLOOR_POS + index * 2,
                self.toggle_floor_positions[index],
            );
        }
        for index in
            0..usize::from(self.toggle_palace_count_x2 / 2).min(DUNGEON_ROOM_TOGGLE_SLOT_COUNT)
        {
            ram.write_word(
                DUNG_TOGGLE_PALACE_POS + index * 2,
                self.toggle_palace_positions[index],
            );
        }
        ram.write_word(FLOOR_1_FILLER_TILES, self.floor_1_filler_tiles);
        ram.write_word(FLOOR_2_FILLER_TILES, self.floor_2_filler_tiles);
        ram.write_word(
            DUNG_LAYOUT_AND_STARTING_QUADRANT,
            self.room_layout_and_starting_quadrant,
        );
        for (room, &mask) in self.pot_reveal_masks.iter().enumerate() {
            ram.write_word(POTS_REVEALED_IN_ROOM_DUNGEON_LOCAL + room * 2, mask);
        }
        let len = self
            .tile_attributes
            .len()
            .min(ram.len().saturating_sub(ATTRIBUTES_FOR_TILE_PLAYER));
        let mut bytes = vec![0u8; len];
        NativeTile::export_slice(&self.tile_attributes[..len], &mut bytes);
        ram.write_range(
            ATTRIBUTES_FOR_TILE_PLAYER..ATTRIBUTES_FOR_TILE_PLAYER + len,
            &bytes,
        );
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

    pub(crate) fn tile_definition(&self, ram: &[u8], tile: usize) -> NativeTile {
        let index = tile & 0x03ff;
        self.tile_attributes.get(index).copied().unwrap_or_else(|| {
            // The upper half aliases live BG character memory, not this catalog.
            NativeTile::from_cartridge(
                ram.get(ATTRIBUTES_FOR_TILE_PLAYER + index)
                    .copied()
                    .unwrap_or(0),
            )
        })
    }

    fn copy_custom_tile_attrs(&mut self, attrs: &[u8]) {
        NativeTile::import_slice(&mut self.tile_attributes[0x140..0x1c0], attrs);
    }

    fn copy_default_tile_attrs_tail(&mut self, attrs: &[u8]) {
        NativeTile::import_slice(&mut self.tile_attributes[0x1c0..0x200], attrs);
    }

    fn copy_default_tile_attrs_head(&mut self, data: &[u8]) {
        NativeTile::import_slice(&mut self.tile_attributes[..0x140], &data[..0x140]);
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
    /// `SWYKPT` in the original: the star-switch floor phase, toggled by a
    /// pressed star tile and read to pick which half of the star CHR to restore.
    star_tile_phase: u8,
    fixed_color_plusminus: u8,
    trap_trigger_latch: u8,
    bomb_trap_activation: u8,
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
            star_tile_phase: 0,
            fixed_color_plusminus: 0,
            trap_trigger_latch: 0,
            bomb_trap_activation: 0,
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
            star_tile_phase: ram.get(STAR_TILE_PHASE).copied().unwrap_or(0),
            fixed_color_plusminus: ram
                .get(OVERWORLD_FIXED_COLOR_PLUSMINUS)
                .copied()
                .unwrap_or(0),
            trap_trigger_latch: ram.get(DUNGEON_TRAP_TRIGGER_LATCH).copied().unwrap_or(0),
            bomb_trap_activation: ram.get(ACTIVATE_BOMB_TRAP_OVERLORD).copied().unwrap_or(0),
            moving_wall_replacement_buffer,
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_byte(DUNG_BLASTWALL_FLAG_X, self.blast_wall_x_open);
        ram.write_byte(DUNG_BLASTWALL_FLAG_Y, self.blast_wall_y_open);
        ram.write_word(CRUSH_WALL_PROGRESS, self.crush_wall_progress);
        ram.write_word(CRUSH_WALL_DOOR_INDEX_X2, self.blast_wall_door_index_x2);
        ram.write_byte(MOVING_WALL_DOT_POINTER, self.moving_wall_dot_pointer);
        ram.write_word(MOVING_WALL_WRITE_POINT, self.moving_wall_write_point);
        ram.write_byte(STAR_TILE_PHASE, self.star_tile_phase);
        ram.write_byte(OVERWORLD_FIXED_COLOR_PLUSMINUS, self.fixed_color_plusminus);
        ram.write_byte(DUNGEON_TRAP_TRIGGER_LATCH, self.trap_trigger_latch);
        ram.write_byte(ACTIVATE_BOMB_TRAP_OVERLORD, self.bomb_trap_activation);
        for (index, &value) in self.moving_wall_replacement_buffer.iter().enumerate() {
            ram.write_word(MOVING_WALL_REPLACEMENT_BUFFER + index * 2, value);
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

    pub(crate) fn star_tile_phase(&self) -> u8 {
        self.star_tile_phase
    }

    fn toggle_star_tile_phase(&mut self) {
        self.star_tile_phase ^= 1;
    }

    fn clear_star_tile_phase(&mut self) {
        self.star_tile_phase = 0;
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

    fn clear_blast_wall_state(&mut self) {
        self.blast_wall_x_open = 0;
        self.blast_wall_y_open = 0;
        self.crush_wall_progress = 0;
        self.blast_wall_door_index_x2 = 0;
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

    fn set_activate_bomb_trap_overlord(&mut self, value: u8) {
        self.bomb_trap_activation = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonRoomItemState {
    num_chests_x2: u16,
    num_big_key_locks_x2: u16,
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
        for (index, quad) in replacement_tilemap_quads.iter_mut().enumerate() {
            *quad = [
                read_le_u16(ram, REPLACEMENT_TILEMAP_UL + index * 2),
                read_le_u16(ram, REPLACEMENT_TILEMAP_LL + index * 2),
                read_le_u16(ram, REPLACEMENT_TILEMAP_UR + index * 2),
                read_le_u16(ram, REPLACEMENT_TILEMAP_LR + index * 2),
            ];
        }
        Self {
            num_chests_x2: read_le_u16(ram, DUNG_NUM_CHESTS_X2),
            num_big_key_locks_x2: read_le_u16(ram, DUNG_NUM_BIGKEY_LOCKS_X2),
            replacement_tile_destination_x2: read_le_u16(ram, DUNG_REPLACEMENT_TILE_DST_POS_X2),
            replacement_tile_source_x2: read_le_u16(ram, DUNG_REPLACEMENT_TILE_SRC_POS_X2),
            chest_locations,
            replacement_tilemap_quads,
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(DUNG_NUM_CHESTS_X2, self.num_chests_x2);
        ram.write_word(DUNG_NUM_BIGKEY_LOCKS_X2, self.num_big_key_locks_x2);
        ram.write_word(
            DUNG_REPLACEMENT_TILE_DST_POS_X2,
            self.replacement_tile_destination_x2,
        );
        ram.write_word(
            DUNG_REPLACEMENT_TILE_SRC_POS_X2,
            self.replacement_tile_source_x2,
        );
        for (index, &location) in self.chest_locations.iter().enumerate() {
            ram.write_word(DUNG_CHEST_LOCATIONS + index * 2, location);
        }
        for (index, quad) in self.replacement_tilemap_quads.iter().enumerate() {
            ram.write_word(REPLACEMENT_TILEMAP_UL + index * 2, quad[0]);
            ram.write_word(REPLACEMENT_TILEMAP_LL + index * 2, quad[1]);
            ram.write_word(REPLACEMENT_TILEMAP_UR + index * 2, quad[2]);
            ram.write_word(REPLACEMENT_TILEMAP_LR + index * 2, quad[3]);
        }
    }

    pub(crate) fn num_chests_x2(&self) -> u16 {
        self.num_chests_x2
    }

    pub(crate) fn num_big_key_locks_x2(&self) -> u16 {
        self.num_big_key_locks_x2
    }

    pub(crate) fn chest_reveal_cursor_reached_end(&self, cursor_x2: u16) -> bool {
        cursor_x2 == self.num_chests_x2
    }

    pub(crate) fn replacement_tile_destination_x2(&self) -> u16 {
        self.replacement_tile_destination_x2
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        for list in ALL_DUNGEON_STAIR_LISTS {
            ram.write_word(
                stair_list_counter_address(list),
                self.stair_list_count(list),
            );
        }

        for (index, position) in self.inter_staircases.iter().enumerate() {
            ram.write_word(DUNG_INTER_STAIRCASES + index * 2, *position);
        }

        for (index, position) in self.stairs_table_1.iter().enumerate() {
            ram.write_word(DUNG_STAIRS_TABLE_1 + index * 2, *position);
        }

        for (index, position) in self.stairs_table_2.iter().enumerate() {
            ram.write_word(DUNG_STAIRS_TABLE_2 + index * 2, *position);
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
    attrs: Vec<NativeTile>,
}

impl Default for DungeonBg2AttributeState {
    fn default() -> Self {
        Self {
            attrs: vec![NativeTile::GROUND; DUNGEON_BG2_ATTR_BUFFER_LEN],
        }
    }
}

impl DungeonBg2AttributeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut attrs = vec![NativeTile::GROUND; DUNGEON_BG2_ATTR_BUFFER_LEN];
        let available = ram.len().saturating_sub(DUNGEON_BG2_ATTR_TABLE);
        let len = attrs.len().min(available);
        NativeTile::import_slice(
            &mut attrs[..len],
            &ram[DUNGEON_BG2_ATTR_TABLE..DUNGEON_BG2_ATTR_TABLE + len],
        );
        Self { attrs }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        let available = ram.len().saturating_sub(DUNGEON_BG2_ATTR_TABLE);
        let len = self.attrs.len().min(available);
        let mut bytes = vec![0u8; len];
        NativeTile::export_slice(&self.attrs[..len], &mut bytes);
        ram.write_range(DUNGEON_BG2_ATTR_TABLE..DUNGEON_BG2_ATTR_TABLE + len, &bytes);
    }

    pub(crate) fn bg2_attr(&self, offset: usize) -> u8 {
        self.bg2_tile(offset).cartridge_attribute()
    }

    pub(crate) fn bg2_tiles(&self, offset: usize) -> crate::tile_definition::TilePair {
        crate::tile_definition::TilePair([self.bg2_tile(offset), self.bg2_tile(offset + 1)])
    }

    pub(crate) fn bg1_tiles(&self, offset: usize) -> crate::tile_definition::TilePair {
        crate::tile_definition::TilePair([self.bg1_tile(offset), self.bg1_tile(offset + 1)])
    }

    pub(crate) fn bg2_attr_word(&self, offset: usize) -> u16 {
        u16::from(self.bg2_attr(offset)) | (u16::from(self.bg2_attr(offset + 1)) << 8)
    }

    pub(crate) fn bg2_tile(&self, offset: usize) -> NativeTile {
        self.attrs.get(offset).copied().unwrap_or_default()
    }

    pub(crate) fn bg1_tile(&self, offset: usize) -> NativeTile {
        self.bg2_tile(DUNGEON_BG1_ATTR_BUFFER_OFFSET + offset)
    }

    pub(crate) fn bg1_attr(&self, offset: usize) -> u8 {
        self.bg1_tile(offset).cartridge_attribute()
    }

    pub(crate) fn bg2_attr_pair(&self, offset: usize) -> Option<(u8, u8)> {
        Some((
            self.attrs.get(offset)?.cartridge_attribute(),
            self.attrs
                .get(offset.wrapping_add(1))?
                .cartridge_attribute(),
        ))
    }

    #[cfg(test)]
    pub(crate) fn bg2_attr_slice(&self, start: usize, len: usize) -> Vec<u8> {
        self.attrs[start..start + len]
            .iter()
            .map(|tile| tile.cartridge_attribute())
            .collect()
    }

    fn set_bg2_tile(&mut self, offset: usize, tile: NativeTile) {
        self.attrs[offset] = tile;
    }

    fn set_bg2_tiles(&mut self, offset: usize, tiles: [NativeTile; 2]) {
        self.attrs[offset..offset + 2].copy_from_slice(&tiles);
    }

    fn set_bg1_tiles(&mut self, offset: usize, tiles: [NativeTile; 2]) {
        self.set_bg2_tiles(DUNGEON_BG1_ATTR_BUFFER_OFFSET + offset, tiles);
    }

    #[cfg(test)]
    fn set_bg2_attr(&mut self, offset: usize, value: u8) {
        self.set_bg2_tile(offset, NativeTile::from_cartridge(value));
    }

    // Overworld transitions reuse this bank for packed map graphics. Preserve
    // both byte identities without treating this as a gameplay tile edit.
    fn import_aliased_map8_word(&mut self, offset: usize, value: u16) {
        self.set_bg2_tiles(offset, NativeTile::import_pair(value));
    }

    #[cfg(test)]
    fn set_bg1_attr_word(&mut self, offset: usize, value: u16) {
        self.set_bg1_tiles(offset, NativeTile::import_pair(value));
    }

    #[cfg(test)]
    fn fill_bg2_attr_range(&mut self, start: usize, len: usize, value: u8) {
        self.attrs[start..start + len].fill(NativeTile::from_cartridge(value));
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(DUNG_SAVEGAME_STATE_BITS, self.state_bits);
    }

    pub(crate) fn savegame_state_bits(&self) -> u16 {
        self.state_bits
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
    target: NativeTile,
    lit_torches: u8,
    lights_out_request: u8,
    lights_out_request_copy: u8,
    dark_with_lantern: u8,
    ganon_torch_count: u8,
    torches_start_index: u16,
    torch_index: u16,
    torch_data_words: Vec<u16>,
}

impl DungeonTorchState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut timers = [0; DUNGEON_TORCH_TIMER_COUNT];
        for (index, timer) in timers.iter_mut().enumerate() {
            *timer = ram.get(TORCH_TIMERS + index).copied().unwrap_or(0);
        }

        let mut torch_data_words = vec![0; DUNGEON_TORCH_DATA_SCAN_WORDS];
        for (index, word) in torch_data_words.iter_mut().enumerate() {
            *word = read_le_u16(ram, DUNGEON_TORCH_DATA + index * 2);
        }

        Self {
            timers,
            target: NativeTile::from_cartridge(ram.get(DUNGEON_TORCH_ATTR).copied().unwrap_or(0)),
            lit_torches: ram.get(DUNG_NUM_LIT_TORCHES).copied().unwrap_or(0),
            lights_out_request: ram.get(DUNG_WANT_LIGHTS_OUT).copied().unwrap_or(0),
            lights_out_request_copy: ram.get(DUNG_WANT_LIGHTS_OUT_COPY).copied().unwrap_or(0),
            dark_with_lantern: ram.get(HDR_DUNGEON_DARK_WITH_LANTERN).copied().unwrap_or(0),
            ganon_torch_count: ram.get(GANON_TORCH_COUNT).copied().unwrap_or(0),
            torches_start_index: read_le_u16(ram, DUNG_INDEX_OF_TORCHES_START),
            torch_index: read_le_u16(ram, DUNG_INDEX_OF_TORCHES),
            torch_data_words,
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_range(
            TORCH_TIMERS..TORCH_TIMERS + DUNGEON_TORCH_TIMER_COUNT,
            &self.timers,
        );
        ram.write_byte(DUNGEON_TORCH_ATTR, self.target.cartridge_attribute());
        ram.write_byte(DUNG_NUM_LIT_TORCHES, self.lit_torches);
        ram.write_byte(DUNG_WANT_LIGHTS_OUT, self.lights_out_request);
        ram.write_byte(DUNG_WANT_LIGHTS_OUT_COPY, self.lights_out_request_copy);
        ram.write_byte(HDR_DUNGEON_DARK_WITH_LANTERN, self.dark_with_lantern);
        ram.write_byte(GANON_TORCH_COUNT, self.ganon_torch_count);
        ram.write_word(DUNG_INDEX_OF_TORCHES_START, self.torches_start_index);
        ram.write_word(DUNG_INDEX_OF_TORCHES, self.torch_index);
        for (index, word) in self.torch_data_words.iter().enumerate() {
            ram.write_word(DUNGEON_TORCH_DATA + index * 2, *word);
        }
    }

    pub(crate) fn timer(&self, index: usize) -> u8 {
        self.timers.get(index).copied().unwrap_or(0)
    }

    /// Slot nibble of the targeted identity, as the original masked it.
    pub(crate) fn attr_index(&self) -> usize {
        self.target.object_slot()
    }

    /// The tile the torch logic was last pointed at; only a torch identity
    /// lights or extinguishes.
    pub(crate) fn target(&self) -> NativeTile {
        self.target
    }

    pub(crate) fn targets_torch(&self) -> bool {
        matches!(self.target.dungeon_role(), DungeonRole::Torch { .. })
    }

    pub(crate) fn lit_torches(&self) -> u8 {
        self.lit_torches
    }

    pub(crate) fn wants_lights_out(&self) -> u8 {
        self.lights_out_request
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

    fn clear_timer(&mut self, index: usize) {
        if let Some(timer) = self.timers.get_mut(index) {
            *timer = 0;
        }
    }

    fn clear_timers(&mut self) {
        self.timers.fill(0);
    }

    fn clear_torch_indices(&mut self) {
        self.torches_start_index = 0;
        self.torch_index = 0;
    }

    fn set_timer(&mut self, index: usize, value: u8) {
        if let Some(timer) = self.timers.get_mut(index) {
            *timer = value;
        }
    }

    fn clear_lit_torches(&mut self) {
        self.lit_torches = 0;
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

    fn set_target(&mut self, value: NativeTile) {
        self.target = value;
    }

    fn clear_target(&mut self) {
        self.target = NativeTile::GROUND;
    }

    fn set_ganon_torch_count(&mut self, value: u8) {
        self.ganon_torch_count = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonEntranceBackupState {
    exit_tile_themes: [u8; 4],
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
        Self { exit_tile_themes }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_range(
            OVERWORLD_EXIT_TILE_THEME_INDEX..OVERWORLD_EXIT_TILE_THEME_INDEX + 4,
            &self.exit_tile_themes,
        );
    }

    pub(crate) fn exit_tile_theme(&self, index: usize) -> u8 {
        self.exit_tile_themes.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn cache_exit_tile_themes(&mut self, overworld: u8, main: u8, aux: u8, sprite: u8) {
        self.exit_tile_themes = [overworld, main, aux, sprite];
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_range(
            DUNGEON_HEADER_TAG..DUNGEON_HEADER_TAG + DUNGEON_HEADER_TAG_COUNT,
            &self.tags,
        );
        ram.write_range(
            DUNGEON_HEADER_TRAVEL_DESTINATIONS
                ..DUNGEON_HEADER_TRAVEL_DESTINATIONS + DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT,
            &self.travel_destinations,
        );
        ram.write_range(
            DUNGEON_HEADER_HOLE_TELEPORTER_PLANE
                ..DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + DUNGEON_HEADER_PLANE_SCRATCH_COUNT,
            &self.plane_scratch,
        );
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(DUNGEON_WORK_R16, self.r16);
        ram.write_word(DUNGEON_WORK_R18, self.r18);
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
        self.r16 &= 0xff00;
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

adopting_bridge!(NativeDungeonEntranceBackupBridgeMut, state: DungeonEntranceBackupState);

impl<'a> NativeDungeonEntranceBackupBridgeMut<'a> {
    pub(crate) fn cache_exit_tile_themes(&mut self) {
        self.state.cache_exit_tile_themes(
            self.ram[OVERWORLD_TILE_THEME_INDEX],
            self.ram[MAIN_TILE_THEME_INDEX],
            self.ram[AUX_TILE_THEME_INDEX],
            self.ram[SPRITE_GRAPHICS_INDEX],
        );
        self.sync();
    }
}

// R16/R18 (0xc8-0xcb) are shared SNES bytes that other code writes directly —
// notably the 3bpp->4bpp gfx converter, whose DUNG_LINE_PTRS_ROW0 (0xbf) scratch
// buffer overlaps them, and the select-file R17 clear. This native is NOT
// bulk-projected every frame (see DungeonState::write_to_ram), so re-read it from
// RAM before any mutation: byte/half-word setters then preserve the live RAM half
// (like C's `ram[R16]=v` byte store) instead of re-stamping a stale frame-start word.
adopting_bridge!(NativeDungeonScratchWordBridgeMut, scratch: DungeonScratchWordState);

impl<'a> NativeDungeonScratchWordBridgeMut<'a> {
    forward_synced! {
        scratch;
        fn decrement_high() -> u8;
        fn set_word(value: u16);
        fn set_liftable_tile_probe_position(y: u16, x: u16);
        fn set_ganon_door_bounce_countdown(value: u16);
        fn decrement_ganon_door_bounce_low() -> u8;
        fn clear_module_transition_counter();
        fn set_minigame_previous_chest_choice(value: u8);
        fn set_primary_word(value: u16);
        fn set_secondary_word(value: u16);
        fn clear_primary_word();
        fn set_primary_low(value: u8);
        fn decrement_primary_low() -> u8;
        fn increment_secondary_low() -> u8;
    }
}

adopting_bridge!(NativeDungeonSavegameBridgeMut, state: DungeonSavegameState);

impl<'a> NativeDungeonSavegameBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_savegame_state_bits(value: u16);
        fn clear_savegame_state_bits();
        fn clear_savegame_state_high();
        fn clear_savegame_state_low();
        fn set_savegame_state_high_bits(mask: u8);
        fn or_savegame_state_bits(mask: u16) -> u16;
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

    /// Encode the tiles at `offset..offset + count` and write exactly those bytes; this
    /// bridge is on the per-tile room draw path, so it neither adopts nor re-projects
    /// the whole table.
    fn publish(&mut self, offset: usize, count: usize) {
        let end = (offset + count).min(self.state.attrs.len());
        if offset >= end {
            return;
        }
        let mut bytes = vec![0u8; end - offset];
        NativeTile::export_slice(&self.state.attrs[offset..end], &mut bytes);
        self.ram[DUNGEON_BG2_ATTR_TABLE + offset..DUNGEON_BG2_ATTR_TABLE + end]
            .copy_from_slice(&bytes);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonBg2AttributeState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_bg2_tile(&mut self, offset: usize, tile: NativeTile) {
        self.state.set_bg2_tile(offset, tile);
        self.publish(offset, 1);
    }

    pub(crate) fn set_bg2_tiles(&mut self, offset: usize, tiles: [NativeTile; 2]) {
        self.state.set_bg2_tiles(offset, tiles);
        self.publish(offset, 2);
    }

    pub(crate) fn set_bg1_tiles(&mut self, offset: usize, tiles: [NativeTile; 2]) {
        self.state.set_bg1_tiles(offset, tiles);
        self.publish(DUNGEON_BG1_ATTR_BUFFER_OFFSET + offset, 2);
    }

    pub(crate) fn import_aliased_map8_word(&mut self, offset: usize, value: u16) {
        self.state.import_aliased_map8_word(offset, value);
        self.publish(offset, 2);
    }

    #[cfg(test)]
    pub(crate) fn set_bg2_attr(&mut self, offset: usize, value: u8) {
        self.state.set_bg2_attr(offset, value);
        self.publish(offset, 1);
    }

    #[cfg(test)]
    pub(crate) fn set_bg1_attr_word(&mut self, offset: usize, value: u16) {
        self.state.set_bg1_attr_word(offset, value);
        self.publish(DUNGEON_BG1_ATTR_BUFFER_OFFSET + offset, 2);
    }

    #[cfg(test)]
    pub(crate) fn fill_bg2_attr_range(&mut self, start: usize, len: usize, value: u8) {
        self.state.fill_bg2_attr_range(start, len, value);
        self.publish(start, len);
    }
}

adopting_bridge!(NativeDungeonStairListsBridgeMut, state: DungeonStairListsState);

impl<'a> NativeDungeonStairListsBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_stair_list_count(list: DungeonStairList, value: u16);
        fn clear_all_counts();
        fn sync_stair_list_counts(lists: &[DungeonStairList], value: u16);
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

    forward_synced! {
        state;
        fn promote_water_stairs_to_active();
        fn set_inter_staircase_pos(index: usize, value: u16);
    }
}

adopting_bridge!(NativeDungeonMovingFloorBridgeMut, state: DungeonMovingFloorState);

impl<'a> NativeDungeonMovingFloorBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_floor_y_velocity_high(value: u8);
        fn set_floor_y_velocity(value: u16);
        fn set_floor_x_velocity(value: u16);
        fn set_floor_x_offset(value: u16);
        fn set_floor_y_offset(value: u16);
        fn set_water_transition_counter(value: u8);
        fn increment_water_transition_counter() -> u8;
        fn decrement_water_transition_counter() -> u8;
        fn set_floor_offsets(x: u16, y: u16);
        fn add_floor_x_offset(delta: u16) -> u16;
        fn sub_floor_x_offset(delta: u16) -> u16;
        fn clear_floor_offsets();
        fn clear_floor_move_flags();
        fn set_floor_move_flags(value: u16);
        fn increment_floor_move_flags();
    }
}

adopting_bridge!(NativeDungeonRoomTrackingBridgeMut, state: DungeonRoomTrackingState);

impl<'a> NativeDungeonRoomTrackingBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_room_index2(value: u8);
        fn set_room_index2_word(value: u16);
        fn set_room_index_prev(value: u8);
        fn set_previous_room_index_word(value: u16);
        fn set_room_history_entry(index: usize, value: u16);
        fn reset_room_history();
    }
}

adopting_bridge!(NativeDungeonObjectTrackingBridgeMut, state: DungeonObjectTrackingState);

impl<'a> NativeDungeonObjectTrackingBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_big_rock_starting_address(value: u16);
        fn clear_replacement_tile_states();
        fn clear_object_data_positions();
        fn set_object_record(index: usize, record: ObjectRecord);
        fn advance_object_record(index: usize);
        fn settle_object_record(index: usize);
        fn set_object_data_pos(index: usize, value: u16);
        fn set_object_tilemap_pos(index: usize, value: u16);
        fn set_misc_object_index(value: u16);
        fn clear_misc_object_index();
        fn set_changeable_object_index(index: usize, value: u8);
        fn clear_changeable_object_index(index: usize);
    }
}

adopting_bridge!(NativeDungeonDoorBridgeMut, state: DungeonDoorState);

impl<'a> NativeDungeonDoorBridgeMut<'a> {
    pub(crate) fn or_opened_doors(&mut self, mask: u16) -> u16 {
        let opened = self.state.or_opened_doors(mask);
        self.sync();
        opened
    }

    pub(crate) fn mark_door_opened(&mut self, door: usize) -> u16 {
        let opened = self.state.mark_door_opened(door);
        self.sync();
        opened
    }

    /// Clears only the LOW byte of DUNG_CUR_DOOR_POS (0x68e), preserving the high byte (0x68f).
    /// C's InterRoomTrans_State15 writes `ram[DUNG_CUR_DOOR_POS_DUNGEON] = 0` as a BYTE, leaving
    /// the high byte (a stale leftover) intact; a full word-clear here diverges scratch (0x68f).
    pub(crate) fn clear_current_door_pos_low_byte(&mut self) {
        self.state.clear_current_door_pos_low_byte();
        self.sync();
    }

    forward_synced! {
        state;
        fn clear_door_animation_step();
        fn set_door_animation_step(value: u16);
        fn set_door_animation_step_low(value: u8);
    }

    forward_synced! {
        state;
        fn set_opened_doors(value: u16);
        fn set_opened_doors_including_adjacent(value: u16);
        fn clear_door_tilemap_addresses();
        fn set_door_tilemap_address(door: usize, value: u16);
        fn load_room_door_tilemap_addresses_from_info(door_info: &[u8]);
        fn clear_door_tables();
        fn set_door_type_word(door: usize, value: u16);
        fn set_door_direction_word(door: usize, value: u16);
        fn set_current_door_index(value: u16);
        fn set_current_door_index_for_slot(door: usize);
        fn set_current_door_pos(value: u16);
        fn clear_current_door_pos();
        fn set_door_open_counter(value: u16);
        fn set_door_open_counter_low(value: u8);
        fn clear_door_open_counter_low();
        fn increment_door_open_counter_low() -> u8;
        fn mark_door_switch_triggered();
        fn clear_door_switch_triggered();
        fn clear_door_barrier_or_switch_flag();
    }
}

adopting_bridge!(NativeDungeonRoomLoadBridgeMut, state: DungeonRoomLoadState);

impl<'a> NativeDungeonRoomLoadBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_header_collision(value: u8);
        fn set_header_collision_2(value: u8);
        fn clear_header_collision_2();
        fn set_header_collision_2_mirror(value: u8);
        fn set_header_collision_2_mirror_high(value: u8);
        fn increment_header_collision_2_mirror() -> u8;
        fn copy_header_collision_2_to_mirror();
        fn set_bg2_properties(value: u8);
        fn clear_bg2_properties();
        fn set_bg2_properties_backup(value: u8);
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

    forward_synced! {
        state;
        fn set_quadrants_visited(value: u16);
        fn or_quadrants_visited(value: u16) -> u16;
        fn clear_quadrant_upload_index();
        fn advance_quadrant_upload_index_by(value: u8) -> u8;
        fn set_draw_width_indicator(value: u8);
        fn set_draw_width_indicator_word(value: u16);
        fn set_draw_height_indicator(value: u8);
        fn set_draw_height_indicator_word(value: u16);
        fn set_draw_dimensions_words(width: u16, height: u16);
        fn set_overlay_to_load(value: u8);
        fn set_overlay_to_load_if_empty(value: u8);
        fn clear_overlay_to_load();
        fn set_selected_key_door(door: usize);
        fn set_load_ptr_offset(value: u16);
        fn set_loading_bg_offsets(horizontal: u16, vertical: u16);
    }
}

adopting_bridge!(NativeDungeonEnvironmentBridgeMut, state: DungeonEnvironmentState);

impl<'a> NativeDungeonEnvironmentBridgeMut<'a> {
    forward_synced! {
        state;
        fn clear_water_puzzle_state_changed();
        fn set_water_puzzle_state_changed(value: u8);
        fn increment_water_puzzle_state_changed() -> u8;
        fn set_trapdoors_down(value: u16);
        fn clear_trapdoors_down();
        fn set_trapdoors_down_low(value: u8);
        fn increment_trapdoors_down_low() -> u8;
        fn clear_somaria_block_switch_counter();
        fn increment_somaria_block_switch_counter();
        fn increment_somaria_block_bg_check_flag();
        fn clear_somaria_block_bg_check_flag();
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

    forward_synced! {
        state;
        fn toggle_orange_blue_barrier_state();
        fn clear_moving_floor_check_flags();
        fn or_moving_floor_check_flags(bits: u16) -> u16;
        fn clear_movable_block_was_pushed();
        fn toggle_movable_block_was_pushed();
        fn set_block_trap_related_tile(value: u16);
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

    forward_synced! {
        state;
        fn set_bg1_tile(index: usize, value: u16);
        fn set_bg2_tile(index: usize, value: u16);
        fn set_bg2_tile_by_byte_pos(pos: u16, value: u16);
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

adopting_bridge!(NativeDungeonRoomItemBridgeMut, state: DungeonRoomItemState);

impl<'a> NativeDungeonRoomItemBridgeMut<'a> {
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

    forward_synced! {
        state;
        fn advance_chest_and_big_key_counts() -> usize;
        fn advance_big_key_lock_count() -> usize;
        fn set_chest_location(index: usize, value: u16);
    }

    forward_synced! {
        state;
        fn set_replacement_tile_destination_x2(value: u16);
        fn set_replacement_tile_source_x2(value: u16);
        fn clear_replacement_tile_destination();
        fn clear_chest_location(index: usize);
        fn set_replacement_tilemap_quad(index: usize, quad: [u16; 4]);
    }

    forward_synced! {
        state;
        fn set_chest_location_for_offset_x2(offset_x2: usize, value: u16);
    }
}

adopting_bridge!(NativeDungeonRoomEffectsBridgeMut, state: DungeonRoomEffectsState);

impl<'a> NativeDungeonRoomEffectsBridgeMut<'a> {
    forward_synced! {
        state;
        fn fill_moving_wall_replacement_buffer(value: u16);
        fn set_moving_wall_write_point(value: u16);
        fn set_moving_wall_dot_pointer(value: u8);
        fn set_fixed_color_plusminus(value: u8);
    }

    /// Write the owned fixed-color byte through without re-projecting unrelated room-effect
    /// scratch that is reused by messaging and blast-wall code in other modes.
    pub(crate) fn set_fixed_color_plusminus_write_through(&mut self, value: u8) {
        self.state.set_fixed_color_plusminus(value);
        self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = value;
    }

    forward_synced! {
        state;
        fn toggle_star_tile_phase();
        fn clear_star_tile_phase();
        fn set_blast_wall_door_index(door: usize);
        fn clear_blast_wall_door_index();
        fn mark_blast_wall_x_open();
        fn mark_blast_wall_y_open();
        fn clear_blast_wall_state();
        fn set_crush_wall_progress(value: u16);
        fn set_crush_wall_progress_low(value: u8);
        fn increment_trap_trigger_latch();
        fn mark_trap_trigger_latched();
        fn clear_trap_trigger_latch();
        fn set_activate_bomb_trap_overlord(value: u8);
    }
}

adopting_bridge!(NativeDungeonRoomParserBridgeMut, state: DungeonRoomParserState);

impl<'a> NativeDungeonRoomParserBridgeMut<'a> {
    forward_synced! { state; fn append_star_switch_tile(tilemap_pos: u16) -> usize; }

    pub(crate) fn clear_room_parser_words(&mut self, offsets: &[usize]) {
        for &offset in offsets {
            write_le_u16(self.ram, offset, 0);
        }
        *self.state = DungeonRoomParserState::load_from_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    forward_synced! {
        state;
        fn mark_pot_revealed_in_room(room: usize, mask: u16) -> u16;
        fn clear_pot_reveal_masks();
        fn append_toggle_palace_pos(pos: u16) -> usize;
        fn append_toggle_floor_pos(pos: u16) -> usize;
        fn copy_custom_tile_attrs(attrs: &[u8]);
        fn copy_default_tile_attrs_tail(attrs: &[u8]);
        fn copy_default_tile_attrs_head(data: &[u8]);
        fn set_floor_1_filler_high(value: u8);
        fn set_floor_2_filler_high(value: u8);
        fn set_floor_1_filler_low(value: u8);
        fn set_floor_2_filler_low(value: u8);
        fn set_room_layout_and_starting_quadrant(value: u16);
    }
}

adopting_bridge!(NativeDungeonRoomDoorSetupBridgeMut, state: DungeonRoomDoorSetupState);

impl<'a> NativeDungeonRoomDoorSetupBridgeMut<'a> {
    forward_synced! {
        state;
        fn clear_invisible_door_marker();
        fn set_invisible_door_marker(slot: usize, direction: u16);
        fn clear_exit_door_count_and_flags();
        fn append_exit_door_address(address: u16) -> usize;
        fn set_active_room_load_ptr(value: u16);
        fn set_active_room_load_ptr_bank(value: u8);
        fn add_reset_xy_check_flags(value: u16) -> u16;
    }

    /// Room load zeroes `dung_unk2` (C dungeon.c `Dungeon_LoadRoom`); keep the
    /// native owner aligned so its projection does not re-stamp a previous
    /// room's exploding-wall quadrant override (route host 619103, room $49).
    pub(crate) fn clear_reset_xy_check_flags(&mut self) {
        self.state.clear_reset_xy_check_flags();
        self.sync();
    }

    forward_synced! {
        state;
        fn set_adjacent_door_flags(value: u16);
        fn load_adjacent_doors_from_room_info(door_info: &[u8]);
        fn mark_no_adjacent_doors();
    }
}

adopting_bridge!(NativeDungeonRoomRuntimeBridgeMut, state: DungeonRoomRuntimeState);

impl<'a> NativeDungeonRoomRuntimeBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_room_index_x3(value: u16);
        fn clear_reserved_gfx_config();
        fn skip_room_tags_once();
        fn clear_room_tag_skip();
        fn set_landing_class(value: u8);
        fn clear_landing_class();
    }
}

adopting_bridge!(NativeDungeonMovableBlockBridgeMut, state: DungeonMovableBlockState);

impl<'a> NativeDungeonMovableBlockBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_movable_block_record(index: usize, room: u16, tilemap: u16);
        fn copy_records_from_bytes(data: &[u8]);
    }
}

adopting_bridge!(NativeDungeonStairMovementBridgeMut, state: DungeonStairMovementState);

impl<'a> NativeDungeonStairMovementBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_current_floor(value: u8);
        fn decrement_current_floor() -> u8;
        fn increment_current_floor() -> u8;
        fn cache_current_floor();
        fn restore_cached_floor();
        fn set_staircase_tilemap_pos_x2(value: u16);
        fn set_current_staircase_plane(value: u8);
        fn set_staircase_lower_level_status(value: u8);
        fn set_staircase_countdown(value: u8);
        fn decrement_staircase_countdown_underflowed() -> bool;
        fn set_staircase_index(value: u8);
        fn set_staircase_index_high(value: u8);
        fn set_staircase_move_counter(value: u8);
        fn decrement_staircase_move_counter() -> u8;
        fn set_kind_of_in_room_staircase_word(value: u16);
    }
}

pub(crate) struct NativeDungeonTorchBridgeMut<'a> {
    torch: &'a mut DungeonTorchState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonTorchBridgeMut<'a> {
    pub(crate) fn new(torch: &'a mut DungeonTorchState, ram: &'a mut [u8]) -> Self {
        *torch = DungeonTorchState::load_from_ram(&*ram);
        Self { torch, ram }
    }

    fn debug_assert_matches_ram(&mut self) {
        debug_assert_eq!(*self.torch, DungeonTorchState::load_from_ram(self.ram));
    }

    fn sync(&mut self) {
        self.torch
            .write_to_ram(&mut crate::game_state::native::ram_target::DiffTarget::new(
                self.ram,
            ));
        self.debug_assert_matches_ram();
    }

    pub(crate) fn copy_torch_init_to_movable_blocks(&mut self, torch_init: &[u8]) {
        self.ram[MOVABLE_BLOCK_DATAS + 99 * 4..MOVABLE_BLOCK_DATAS + 99 * 4 + 116]
            .copy_from_slice(&torch_init[..116]);
    }

    /// Imports the cartridge torch table through WRAM: the copy lands in the
    /// bytes the state scans and in the bytes past them alike, then the state
    /// adopts the result.
    pub(crate) fn copy_torch_data_table(&mut self, torch_init: &[u8]) {
        self.ram[DUNGEON_TORCH_DATA..DUNGEON_TORCH_DATA + torch_init.len()]
            .copy_from_slice(torch_init);
        *self.torch = DungeonTorchState::load_from_ram(self.ram);
    }

    /// The bytes past the scanned torch table are not modelled; the copy
    /// lands in WRAM and the state re-adopts.
    pub(crate) fn copy_torch_junk(&mut self, torch_junk: &[u8]) {
        self.ram[DUNGEON_TORCH_DATA + 144 * 2..DUNGEON_TORCH_DATA + 144 * 2 + torch_junk.len()]
            .copy_from_slice(torch_junk);
        *self.torch = DungeonTorchState::load_from_ram(self.ram);
    }

    /// Reload the torch model from RAM after the room object parser has run
    /// (`RoomDraw_DrawAllObjectsCurrentRoom` writes torch-owned bytes directly,
    /// outside this bridge, so the native model must be resynced).
    pub(crate) fn resync_from_ram(&mut self) {
        *self.torch = DungeonTorchState::load_from_ram(self.ram);
    }

    forward_synced! {
        torch;
        fn clear_timer(index: usize);
        fn clear_timers();
        fn clear_torch_indices();
        fn set_timer(index: usize, value: u8);
        fn clear_lit_torches();
        fn increment_lit_torches() -> u8;
        fn decrement_lit_torches() -> u8;
        fn set_lights_out_request(value: u8);
        fn clear_lights_out_request();
        fn copy_lights_out_request();
        fn clear_lights_out_requests();
        fn set_dungeon_dark_with_lantern();
        fn set_dungeon_dark_with_lantern_raw(value: u8);
        fn clear_dungeon_dark_with_lantern();
        fn set_torch_index_range_start(value: u16);
        fn set_torch_index(value: u16);
        fn set_torch_data_word_index(index: usize, value: u16);
        fn set_target(value: NativeTile);
        fn set_ganon_torch_count(value: u8);
        fn clear_target();
    }
}

adopting_bridge!(NativeDungeonHeaderBridgeMut, header: DungeonHeaderState);

impl<'a> NativeDungeonHeaderBridgeMut<'a> {
    forward_synced! {
        header;
        fn set_hole_teleporter_planes(packed: u8, extra: u8);
        fn set_header_tag(index: usize, value: u8);
        fn copy_travel_destinations_from_header(header: &[u8]);
        fn clear_header_tag(index: usize);
    }
}
