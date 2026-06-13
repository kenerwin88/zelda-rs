use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const PUSHED_BLOCK_BANK_LEN: usize = 4;
const PUSHED_BLOCK_SLOT_COUNT: usize = 2;
const SWIM_AXIS_COUNT: usize = 2;
const FALL_HOLE_SCAN_INDEX_LOCAL: usize = 0x02c9;
const PLAYER_TILE_ATTRIBUTE_COUNT: usize = 0x400;

fn swim_axis_index(offset: usize) -> Option<usize> {
    match offset {
        0 => Some(0),
        2 => Some(1),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlayerTileAttributeTableState {
    attributes: Vec<u8>,
}

impl Default for PlayerTileAttributeTableState {
    fn default() -> Self {
        Self {
            attributes: vec![0; PLAYER_TILE_ATTRIBUTE_COUNT],
        }
    }
}

impl PlayerTileAttributeTableState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut attributes = vec![0; PLAYER_TILE_ATTRIBUTE_COUNT];
        for (index, attribute) in attributes.iter_mut().enumerate() {
            *attribute = ram_byte(ram, ATTRIBUTES_FOR_TILE_PLAYER + index);
        }
        Self { attributes }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[ATTRIBUTES_FOR_TILE_PLAYER..ATTRIBUTES_FOR_TILE_PLAYER + PLAYER_TILE_ATTRIBUTE_COUNT]
            .copy_from_slice(&self.attributes);
    }

    pub(crate) fn attr_for_tile(&self, tile: usize) -> u8 {
        self.attributes[tile & (PLAYER_TILE_ATTRIBUTE_COUNT - 1)]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpecialExitPositionState {
    x: u16,
    y: u16,
}

impl SpecialExitPositionState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            x: if LINK_X_COORD_SPEXIT + 1 < ram.len() {
                read_le_u16(ram, LINK_X_COORD_SPEXIT)
            } else {
                0
            },
            y: if LINK_Y_COORD_SPEXIT + 1 < ram.len() {
                read_le_u16(ram, LINK_Y_COORD_SPEXIT)
            } else {
                0
            },
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, LINK_X_COORD_SPEXIT, self.x);
        write_le_u16(ram, LINK_Y_COORD_SPEXIT, self.y);
    }

    pub(crate) fn x(&self) -> u16 {
        self.x
    }

    pub(crate) fn y(&self) -> u16 {
        self.y
    }

    pub(crate) fn map_zoom_y(&self) -> u16 {
        ((self.y >> 4).wrapping_sub(0x48)) & !1
    }

    pub(crate) fn map_zoom_x_offset(&self) -> u16 {
        (self.x >> 4).wrapping_sub(0x80)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlayerState {
    pub(crate) special_exit_position: SpecialExitPositionState,
    pub(crate) swim_acceleration: SwimAccelerationState,
    pub(crate) pushed_block: PushedBlockState,
    pub(crate) bg1_movement_accumulator: Bg1MovementAccumulatorState,
    pub(crate) tile_detection: TileDetectionState,
    pub(crate) tile_attributes: PlayerTileAttributeTableState,
}

impl PlayerState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            special_exit_position: SpecialExitPositionState::load_from_ram(ram),
            swim_acceleration: SwimAccelerationState::load_from_ram(ram),
            pushed_block: PushedBlockState::load_from_ram(ram),
            bg1_movement_accumulator: Bg1MovementAccumulatorState::load_from_ram(ram),
            tile_detection: TileDetectionState::load_from_ram(ram),
            tile_attributes: PlayerTileAttributeTableState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.special_exit_position.write_to_ram(ram);
        self.swim_acceleration.write_to_ram(ram);
        self.pushed_block.write_to_ram(ram);
        self.bg1_movement_accumulator.write_to_ram(ram);
        self.tile_detection.write_to_ram(ram);
        self.tile_attributes.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TileDetectionState {
    probe_y: u16,
    probe_x: u16,
    tile_collision_bits_primary: u8,
    tile_collision_bits_secondary: u8,
    liftable_tile_index: u8,
    liftable_action_index_primary: u8,
    liftable_action_index_secondary: u8,
    interaction_scratch_y: u16,
    interaction_scratch_x: u16,
    location_calc_mask: u16,
    interacting_tile: u16,
    pit_tile: u16,
    deepwater: u16,
    normal_tiles: u16,
    misc_tiles: u16,
    thick_grass: u16,
    diagonal_tile: u16,
    stair_tile: u8,
    block_flags: u16,
    door_direction_flags: u16,
    diag_state: u16,
    moving_floor_tiles: u16,
    icy_floor: u16,
    water_staircase: u16,
    shallow_water: u16,
    destruction_aftermath: u16,
    read_something: u16,
    vertical_ledge: u8,
    horizontal_ledge: u8,
    ledges_down_leftright: u8,
    diagonal_ledge_tiles: u8,
    chest: u16,
    key_lock_gravestones: u16,
    tile_type: u16,
    spike_floor_and_triggers: u8,
    dashable_tiles: u8,
    staircase_cache: u8,
    slope_collision_bits: u16,
    collision_bits: u16,
    palette_bits_high: u8,
    inroom_staircase: u16,
    fall_hole_scan_index: u8,
}

impl TileDetectionState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            probe_y: read_le_u16(ram, TILEDETECT_WHICH_Y_POS),
            probe_x: read_le_u16(ram, TILEDETECT_WHICH_Y_POS + 2),
            tile_collision_bits_primary: ram_byte(ram, TILE_COLLISION_BITS_PRIMARY),
            tile_collision_bits_secondary: ram_byte(ram, TILE_COLLISION_BITS_SECONDARY),
            liftable_tile_index: ram_byte(ram, LIFTABLE_TILE_DETECTED_INDEX_DOUBLED),
            liftable_action_index_primary: ram_byte(ram, LIFTABLE_TILE_ACTION_INDEX_PRIMARY),
            liftable_action_index_secondary: ram_byte(ram, LIFTABLE_TILE_ACTION_INDEX_SECONDARY),
            interaction_scratch_y: read_le_u16(ram, SCRATCH_0),
            interaction_scratch_x: read_le_u16(ram, SCRATCH_1),
            location_calc_mask: read_le_u16(ram, TILEMAP_LOCATION_CALC_MASK),
            interacting_tile: read_le_u16(ram, INDEX_OF_INTERACTING_TILE),
            pit_tile: read_le_u16(ram, TILEDETECT_PIT_TILE),
            deepwater: read_le_u16(ram, TILEDETECT_DEEPWATER),
            normal_tiles: read_le_u16(ram, TILEDETECT_NORMAL_TILES),
            misc_tiles: read_le_u16(ram, TILEDETECT_MISC_TILES),
            thick_grass: read_le_u16(ram, TILEDETECT_THICK_GRASS),
            diagonal_tile: read_le_u16(ram, TILEDETECT_DIAGONAL_TILE),
            stair_tile: ram_byte(ram, TILEDETECT_STAIR_TILE),
            block_flags: read_le_u16(ram, TILEDETECT_BLOCK_FLAGS_LO),
            door_direction_flags: read_le_u16(ram, TILEDETECT_DOOR_DIRECTION_FLAGS),
            diag_state: read_le_u16(ram, TILEDETECT_DIAG_STATE),
            moving_floor_tiles: read_le_u16(ram, TILEDETECT_MOVING_FLOOR_TILES),
            icy_floor: read_le_u16(ram, TILEDETECT_ICY_FLOOR),
            water_staircase: read_le_u16(ram, TILEDETECT_WATER_STAIRCASE),
            shallow_water: read_le_u16(ram, TILEDETECT_SHALLOW_WATER),
            destruction_aftermath: read_le_u16(ram, TILEDETECT_DESTRUCTION_AFTERMATH),
            read_something: read_le_u16(ram, TILEDETECT_READ_SOMETHING),
            vertical_ledge: ram_byte(ram, TILEDETECT_VERTICAL_LEDGE),
            horizontal_ledge: ram_byte(ram, DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ),
            ledges_down_leftright: ram_byte(ram, TILEDETECT_LEDGES_DOWN_LEFTRIGHT),
            diagonal_ledge_tiles: ram_byte(ram, TILEDETECT_DIAGONAL_LEDGE_TILES),
            chest: read_le_u16(ram, TILEDETECT_CHEST),
            key_lock_gravestones: read_le_u16(ram, TILEDETECT_KEY_LOCK_GRAVESTONES),
            tile_type: read_le_u16(ram, TILEDETECT_TILE_TYPE),
            spike_floor_and_triggers: ram_byte(ram, TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS),
            dashable_tiles: ram_byte(ram, BITMASK_FOR_DASHABLE_TILES),
            staircase_cache: ram_byte(ram, TILEDETECT_STAIRCASE_CACHE),
            slope_collision_bits: read_le_u16(ram, TILEDETECT_SLOPE_COLLISION_BITS),
            collision_bits: read_le_u16(ram, TILEDETECT_COLLISION_BITS),
            palette_bits_high: ram_byte(ram, LINK_PALETTE_BITS_OF_OAM + 1),
            inroom_staircase: read_le_u16(ram, TILEDETECT_INROOM_STAIRCASE),
            fall_hole_scan_index: ram_byte(ram, FALL_HOLE_SCAN_INDEX_LOCAL),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, TILEDETECT_WHICH_Y_POS, self.probe_y);
        write_le_u16(ram, TILEDETECT_WHICH_Y_POS + 2, self.probe_x);
        ram[TILE_COLLISION_BITS_PRIMARY] = self.tile_collision_bits_primary;
        ram[TILE_COLLISION_BITS_SECONDARY] = self.tile_collision_bits_secondary;
        ram[LIFTABLE_TILE_DETECTED_INDEX_DOUBLED] = self.liftable_tile_index;
        ram[LIFTABLE_TILE_ACTION_INDEX_PRIMARY] = self.liftable_action_index_primary;
        ram[LIFTABLE_TILE_ACTION_INDEX_SECONDARY] = self.liftable_action_index_secondary;
        write_le_u16(ram, SCRATCH_0, self.interaction_scratch_y);
        write_le_u16(ram, SCRATCH_1, self.interaction_scratch_x);
        write_le_u16(ram, TILEMAP_LOCATION_CALC_MASK, self.location_calc_mask);
        write_le_u16(ram, INDEX_OF_INTERACTING_TILE, self.interacting_tile);
        write_le_u16(ram, TILEDETECT_PIT_TILE, self.pit_tile);
        write_le_u16(ram, TILEDETECT_DEEPWATER, self.deepwater);
        write_le_u16(ram, TILEDETECT_NORMAL_TILES, self.normal_tiles);
        write_le_u16(ram, TILEDETECT_MISC_TILES, self.misc_tiles);
        write_le_u16(ram, TILEDETECT_THICK_GRASS, self.thick_grass);
        write_le_u16(ram, TILEDETECT_DIAGONAL_TILE, self.diagonal_tile);
        ram[TILEDETECT_STAIR_TILE] = self.stair_tile;
        write_le_u16(ram, TILEDETECT_BLOCK_FLAGS_LO, self.block_flags);
        write_le_u16(
            ram,
            TILEDETECT_DOOR_DIRECTION_FLAGS,
            self.door_direction_flags,
        );
        write_le_u16(ram, TILEDETECT_DIAG_STATE, self.diag_state);
        write_le_u16(ram, TILEDETECT_MOVING_FLOOR_TILES, self.moving_floor_tiles);
        write_le_u16(ram, TILEDETECT_ICY_FLOOR, self.icy_floor);
        write_le_u16(ram, TILEDETECT_WATER_STAIRCASE, self.water_staircase);
        write_le_u16(ram, TILEDETECT_SHALLOW_WATER, self.shallow_water);
        write_le_u16(
            ram,
            TILEDETECT_DESTRUCTION_AFTERMATH,
            self.destruction_aftermath,
        );
        write_le_u16(ram, TILEDETECT_READ_SOMETHING, self.read_something);
        ram[TILEDETECT_VERTICAL_LEDGE] = self.vertical_ledge;
        ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] = self.horizontal_ledge;
        ram[TILEDETECT_LEDGES_DOWN_LEFTRIGHT] = self.ledges_down_leftright;
        ram[TILEDETECT_DIAGONAL_LEDGE_TILES] = self.diagonal_ledge_tiles;
        write_le_u16(ram, TILEDETECT_CHEST, self.chest);
        write_le_u16(
            ram,
            TILEDETECT_KEY_LOCK_GRAVESTONES,
            self.key_lock_gravestones,
        );
        write_le_u16(ram, TILEDETECT_TILE_TYPE, self.tile_type);
        ram[TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS] = self.spike_floor_and_triggers;
        ram[BITMASK_FOR_DASHABLE_TILES] = self.dashable_tiles;
        ram[TILEDETECT_STAIRCASE_CACHE] = self.staircase_cache;
        write_le_u16(
            ram,
            TILEDETECT_SLOPE_COLLISION_BITS,
            self.slope_collision_bits,
        );
        write_le_u16(ram, TILEDETECT_COLLISION_BITS, self.collision_bits);
        ram[LINK_PALETTE_BITS_OF_OAM + 1] = self.palette_bits_high;
        write_le_u16(ram, TILEDETECT_INROOM_STAIRCASE, self.inroom_staircase);
        ram[FALL_HOLE_SCAN_INDEX_LOCAL] = self.fall_hole_scan_index;
    }

    pub(crate) fn y_low_at(&self, offset: usize) -> u8 {
        match offset {
            0 => self.probe_y as u8,
            1 => (self.probe_y >> 8) as u8,
            2 => self.probe_x as u8,
            3 => (self.probe_x >> 8) as u8,
            _ => 0,
        }
    }

    pub(crate) fn tile_collision_bits_primary(&self) -> u8 {
        self.tile_collision_bits_primary
    }

    pub(crate) fn tile_collision_bits_secondary(&self) -> u8 {
        self.tile_collision_bits_secondary
    }

    pub(crate) fn liftable_tile_index(&self) -> u8 {
        self.liftable_tile_index
    }

    pub(crate) fn liftable_action_index_primary(&self) -> u8 {
        self.liftable_action_index_primary
    }

    pub(crate) fn interaction_scratch_y(&self) -> u16 {
        self.interaction_scratch_y
    }

    pub(crate) fn interaction_scratch_x(&self) -> u16 {
        self.interaction_scratch_x
    }

    pub(crate) fn y(&self) -> u16 {
        self.probe_y
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.probe_y as u8
    }

    pub(crate) fn x(&self) -> u16 {
        self.probe_x
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.probe_x as u8
    }

    pub(crate) fn location_calc_mask(&self) -> u16 {
        self.location_calc_mask
    }

    pub(crate) fn interacting_tile(&self) -> u16 {
        self.interacting_tile
    }

    pub(crate) fn interacting_tile_low(&self) -> u8 {
        self.interacting_tile as u8
    }

    pub(crate) fn pit_tile(&self) -> u8 {
        self.pit_tile as u8
    }

    pub(crate) fn pit_tile_word(&self) -> u16 {
        self.pit_tile
    }

    pub(crate) fn deepwater(&self) -> u16 {
        self.deepwater
    }

    pub(crate) fn deepwater_high(&self) -> u8 {
        (self.deepwater >> 8) as u8
    }

    pub(crate) fn normal_tiles(&self) -> u16 {
        self.normal_tiles
    }

    pub(crate) fn normal_tiles_high(&self) -> u8 {
        (self.normal_tiles >> 8) as u8
    }

    pub(crate) fn misc_tiles(&self) -> u16 {
        self.misc_tiles
    }

    pub(crate) fn thick_grass(&self) -> u16 {
        self.thick_grass
    }

    pub(crate) fn thick_grass_low(&self) -> u8 {
        self.thick_grass as u8
    }

    pub(crate) fn diagonal_tile(&self) -> u16 {
        self.diagonal_tile
    }

    pub(crate) fn stair_tile(&self) -> u8 {
        self.stair_tile
    }

    pub(crate) fn block_flags(&self) -> u16 {
        self.block_flags
    }

    pub(crate) fn door_direction_flags(&self) -> u16 {
        self.door_direction_flags
    }

    pub(crate) fn diag_state(&self) -> u16 {
        self.diag_state
    }

    pub(crate) fn moving_floor_tiles(&self) -> u16 {
        self.moving_floor_tiles
    }

    pub(crate) fn icy_floor(&self) -> u16 {
        self.icy_floor
    }

    pub(crate) fn water_staircase(&self) -> u16 {
        self.water_staircase
    }

    pub(crate) fn shallow_water(&self) -> u16 {
        self.shallow_water
    }

    pub(crate) fn shallow_water_low(&self) -> u8 {
        self.shallow_water as u8
    }

    pub(crate) fn destruction_aftermath(&self) -> u16 {
        self.destruction_aftermath
    }

    pub(crate) fn destruction_aftermath_low(&self) -> u8 {
        self.destruction_aftermath as u8
    }

    pub(crate) fn read_something(&self) -> u16 {
        self.read_something
    }

    pub(crate) fn vertical_ledge(&self) -> u8 {
        self.vertical_ledge
    }

    pub(crate) fn horizontal_ledge(&self) -> u8 {
        self.horizontal_ledge
    }

    pub(crate) fn ledge_mask(&self) -> u8 {
        self.vertical_ledge | self.horizontal_ledge
    }

    pub(crate) fn ledges_down_leftright(&self) -> u8 {
        self.ledges_down_leftright
    }

    pub(crate) fn diagonal_ledge_tiles(&self) -> u8 {
        self.diagonal_ledge_tiles
    }

    pub(crate) fn chest(&self) -> u16 {
        self.chest
    }

    pub(crate) fn key_lock_gravestones(&self) -> u16 {
        self.key_lock_gravestones
    }

    pub(crate) fn key_lock_gravestones_low(&self) -> u8 {
        self.key_lock_gravestones as u8
    }

    pub(crate) fn spike_cactus_tiles(&self) -> u8 {
        (self.key_lock_gravestones >> 8) as u8
    }

    pub(crate) fn tile_type(&self) -> u16 {
        self.tile_type
    }

    pub(crate) fn spike_floor_and_triggers(&self) -> u8 {
        self.spike_floor_and_triggers
    }

    pub(crate) fn dashable_tiles(&self) -> u8 {
        self.dashable_tiles
    }

    pub(crate) fn staircase_cache(&self) -> u8 {
        self.staircase_cache
    }

    pub(crate) fn slope_collision_bits(&self) -> u16 {
        self.slope_collision_bits
    }

    pub(crate) fn collision_bits(&self) -> u16 {
        self.collision_bits
    }

    pub(crate) fn collision_bits_low(&self) -> u8 {
        self.collision_bits as u8
    }

    pub(crate) fn bonk_bits_low(&self) -> u8 {
        self.slope_collision_bits as u8 | self.collision_bits as u8
    }

    pub(crate) fn has_collision_bits(&self, mask: u16) -> bool {
        self.collision_bits & mask != 0
    }

    pub(crate) fn has_slope_collision_bits(&self, mask: u16) -> bool {
        self.slope_collision_bits & mask != 0
    }

    pub(crate) fn palette_bits_high(&self) -> u8 {
        self.palette_bits_high
    }

    pub(crate) fn inroom_staircase(&self) -> u16 {
        self.inroom_staircase
    }
}

pub(crate) struct NativeTileDetectionBridgeMut<'a> {
    state: &'a mut TileDetectionState,
    ram: &'a mut [u8],
}

impl<'a> NativeTileDetectionBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut TileDetectionState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, TileDetectionState::load_from_ram(self.ram));
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.state.probe_y = (self.state.probe_y & 0x00ff) | (u16::from(value) << 8);
        self.sync();
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.state.probe_y = value;
        self.sync();
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.state.probe_x = value;
        self.sync();
    }

    pub(crate) fn set_location_calc_mask(&mut self, value: u16) {
        self.state.location_calc_mask = value;
        self.sync();
    }

    pub(crate) fn set_interacting_tile(&mut self, value: u16) {
        self.state.interacting_tile = value;
        self.sync();
    }

    pub(crate) fn set_interacting_tile_low(&mut self, value: u8) {
        self.state.interacting_tile = (self.state.interacting_tile & 0xff00) | u16::from(value);
        self.sync();
    }

    pub(crate) fn set_fall_hole_scan_index(&mut self, value: u8) {
        self.state.fall_hole_scan_index = value;
        self.sync();
    }

    pub(crate) fn set_interaction_scratch_y(&mut self, value: u16) {
        self.state.interaction_scratch_y = value;
        self.sync();
    }

    pub(crate) fn set_interaction_scratch_x(&mut self, value: u16) {
        self.state.interaction_scratch_x = value;
        self.sync();
    }

    pub(crate) fn set_diagonal_tile(&mut self, value: u16) {
        self.state.diagonal_tile = value;
        self.sync();
    }

    pub(crate) fn clear_diagonal_tile(&mut self) {
        self.set_diagonal_tile(0);
    }

    pub(crate) fn or_diagonal_tile(&mut self, value: u16) -> u16 {
        self.state.diagonal_tile |= value;
        let next = self.state.diagonal_tile;
        self.sync();
        next
    }

    pub(crate) fn set_stair_tile(&mut self, value: u8) {
        self.state.stair_tile = value;
        self.sync();
    }

    pub(crate) fn clear_stair_tile(&mut self) {
        self.set_stair_tile(0);
    }

    pub(crate) fn or_stair_tile(&mut self, value: u8) {
        self.state.stair_tile |= value;
        self.sync();
    }

    pub(crate) fn set_block_flags(&mut self, value: u16) {
        self.state.block_flags = value;
        self.sync();
    }

    pub(crate) fn clear_block_flags(&mut self) {
        self.set_block_flags(0);
    }

    pub(crate) fn or_block_flags(&mut self, value: u16) -> u16 {
        self.state.block_flags |= value;
        let next = self.state.block_flags;
        self.sync();
        next
    }

    pub(crate) fn set_door_direction_flags(&mut self, value: u16) {
        self.state.door_direction_flags = value;
        self.sync();
    }

    pub(crate) fn clear_door_direction_flags(&mut self) {
        self.set_door_direction_flags(0);
    }

    pub(crate) fn set_diag_state(&mut self, value: u16) {
        self.state.diag_state = value;
        self.sync();
    }

    pub(crate) fn clear_diag_state(&mut self) {
        self.set_diag_state(0);
    }

    pub(crate) fn clear_pit_tile(&mut self) {
        self.state.pit_tile = 0;
        self.sync();
    }

    pub(crate) fn or_pit_tile(&mut self, value: u8) {
        self.state.pit_tile |= u16::from(value);
        self.sync();
    }

    pub(crate) fn set_deepwater(&mut self, value: u16) {
        self.state.deepwater = value;
        self.sync();
    }

    pub(crate) fn clear_deepwater(&mut self) {
        self.set_deepwater(0);
    }

    pub(crate) fn or_deepwater(&mut self, value: u16) -> u16 {
        self.state.deepwater |= value;
        let next = self.state.deepwater;
        self.sync();
        next
    }

    pub(crate) fn set_normal_tiles(&mut self, value: u16) {
        self.state.normal_tiles = value;
        self.sync();
    }

    pub(crate) fn clear_normal_tiles(&mut self) {
        self.set_normal_tiles(0);
    }

    pub(crate) fn or_normal_tiles(&mut self, value: u16) -> u16 {
        self.state.normal_tiles |= value;
        let next = self.state.normal_tiles;
        self.sync();
        next
    }

    pub(crate) fn set_misc_tiles(&mut self, value: u16) {
        self.state.misc_tiles = value;
        self.sync();
    }

    pub(crate) fn clear_misc_tiles(&mut self) {
        self.set_misc_tiles(0);
    }

    pub(crate) fn or_misc_tiles(&mut self, value: u16) -> u16 {
        self.state.misc_tiles |= value;
        let next = self.state.misc_tiles;
        self.sync();
        next
    }

    pub(crate) fn set_thick_grass(&mut self, value: u16) {
        self.state.thick_grass = value;
        self.sync();
    }

    pub(crate) fn clear_thick_grass(&mut self) {
        self.set_thick_grass(0);
    }

    pub(crate) fn or_thick_grass(&mut self, value: u16) -> u16 {
        self.state.thick_grass |= value;
        let next = self.state.thick_grass;
        self.sync();
        next
    }

    pub(crate) fn clear_vertical_ledge(&mut self) {
        self.state.vertical_ledge = 0;
        self.sync();
    }

    pub(crate) fn or_vertical_ledge(&mut self, value: u8) {
        self.state.vertical_ledge |= value;
        self.sync();
    }

    pub(crate) fn clear_horizontal_ledge(&mut self) {
        self.state.horizontal_ledge = 0;
        self.sync();
    }

    pub(crate) fn or_horizontal_ledge(&mut self, value: u8) {
        self.state.horizontal_ledge |= value;
        self.sync();
    }

    pub(crate) fn set_moving_floor_tiles(&mut self, value: u16) {
        self.state.moving_floor_tiles = value;
        self.sync();
    }

    pub(crate) fn clear_moving_floor_tiles(&mut self) {
        self.set_moving_floor_tiles(0);
    }

    pub(crate) fn or_moving_floor_tiles(&mut self, value: u16) -> u16 {
        self.state.moving_floor_tiles |= value;
        let next = self.state.moving_floor_tiles;
        self.sync();
        next
    }

    pub(crate) fn set_icy_floor(&mut self, value: u16) {
        self.state.icy_floor = value;
        self.sync();
    }

    pub(crate) fn clear_icy_floor(&mut self) {
        self.set_icy_floor(0);
    }

    pub(crate) fn or_icy_floor(&mut self, value: u16) -> u16 {
        self.state.icy_floor |= value;
        let next = self.state.icy_floor;
        self.sync();
        next
    }

    pub(crate) fn set_water_staircase(&mut self, value: u16) {
        self.state.water_staircase = value;
        self.sync();
    }

    pub(crate) fn clear_water_staircase(&mut self) {
        self.set_water_staircase(0);
    }

    pub(crate) fn or_water_staircase(&mut self, value: u16) -> u16 {
        self.state.water_staircase |= value;
        let next = self.state.water_staircase;
        self.sync();
        next
    }

    pub(crate) fn set_shallow_water(&mut self, value: u16) {
        self.state.shallow_water = value;
        self.sync();
    }

    pub(crate) fn clear_shallow_water(&mut self) {
        self.set_shallow_water(0);
    }

    pub(crate) fn or_shallow_water(&mut self, value: u16) -> u16 {
        self.state.shallow_water |= value;
        let next = self.state.shallow_water;
        self.sync();
        next
    }

    pub(crate) fn set_destruction_aftermath(&mut self, value: u16) {
        self.state.destruction_aftermath = value;
        self.sync();
    }

    pub(crate) fn clear_destruction_aftermath(&mut self) {
        self.set_destruction_aftermath(0);
    }

    pub(crate) fn or_destruction_aftermath(&mut self, value: u16) -> u16 {
        self.state.destruction_aftermath |= value;
        let next = self.state.destruction_aftermath;
        self.sync();
        next
    }

    pub(crate) fn set_read_something(&mut self, value: u16) {
        self.state.read_something = value;
        self.sync();
    }

    pub(crate) fn clear_read_something(&mut self) {
        self.set_read_something(0);
    }

    pub(crate) fn or_read_something(&mut self, value: u16) -> u16 {
        self.state.read_something |= value;
        let next = self.state.read_something;
        self.sync();
        next
    }

    pub(crate) fn set_ledges_down_leftright(&mut self, value: u8) {
        self.state.ledges_down_leftright = value;
        self.sync();
    }

    pub(crate) fn clear_ledges_down_leftright(&mut self) {
        self.set_ledges_down_leftright(0);
    }

    pub(crate) fn or_ledges_down_leftright(&mut self, value: u8) {
        self.state.ledges_down_leftright |= value;
        self.sync();
    }

    pub(crate) fn set_diagonal_ledge_tiles(&mut self, value: u8) {
        self.state.diagonal_ledge_tiles = value;
        self.sync();
    }

    pub(crate) fn clear_diagonal_ledge_tiles(&mut self) {
        self.set_diagonal_ledge_tiles(0);
    }

    pub(crate) fn or_diagonal_ledge_tiles(&mut self, value: u8) {
        self.state.diagonal_ledge_tiles |= value;
        self.sync();
    }

    pub(crate) fn set_chest(&mut self, value: u16) {
        self.state.chest = value;
        self.sync();
    }

    pub(crate) fn clear_chest(&mut self) {
        self.set_chest(0);
    }

    pub(crate) fn or_chest(&mut self, value: u16) -> u16 {
        self.state.chest |= value;
        let next = self.state.chest;
        self.sync();
        next
    }

    pub(crate) fn set_key_lock_gravestones(&mut self, value: u8) {
        self.state.key_lock_gravestones =
            (self.state.key_lock_gravestones & 0xff00) | u16::from(value);
        self.sync();
    }

    pub(crate) fn clear_key_lock_gravestones(&mut self) {
        self.set_key_lock_gravestones(0);
    }

    pub(crate) fn or_key_lock_gravestones(&mut self, value: u8) {
        self.state.key_lock_gravestones |= u16::from(value);
        self.sync();
    }

    pub(crate) fn set_spike_cactus_tiles(&mut self, value: u8) {
        self.state.key_lock_gravestones =
            (self.state.key_lock_gravestones & 0x00ff) | (u16::from(value) << 8);
        self.sync();
    }

    pub(crate) fn clear_spike_cactus_tiles(&mut self) {
        self.set_spike_cactus_tiles(0);
    }

    pub(crate) fn or_spike_cactus_tiles(&mut self, value: u8) {
        self.set_spike_cactus_tiles(self.state.spike_cactus_tiles() | value);
    }

    pub(crate) fn set_tile_type(&mut self, value: u16) {
        self.state.tile_type = value;
        self.sync();
    }

    pub(crate) fn clear_tile_type(&mut self) {
        self.set_tile_type(0);
    }

    pub(crate) fn set_spike_floor_and_triggers(&mut self, value: u8) {
        self.state.spike_floor_and_triggers = value;
        self.sync();
    }

    pub(crate) fn clear_spike_floor_and_triggers(&mut self) {
        self.set_spike_floor_and_triggers(0);
    }

    pub(crate) fn or_spike_floor_and_triggers(&mut self, value: u8) {
        self.state.spike_floor_and_triggers |= value;
        self.sync();
    }

    pub(crate) fn set_dashable_tiles(&mut self, value: u8) {
        self.state.dashable_tiles = value;
        self.sync();
    }

    pub(crate) fn clear_dashable_tiles(&mut self) {
        self.set_dashable_tiles(0);
    }

    pub(crate) fn or_dashable_tiles(&mut self, value: u8) {
        self.state.dashable_tiles |= value;
        self.sync();
    }

    pub(crate) fn set_staircase_cache(&mut self, value: u8) {
        self.state.staircase_cache = value;
        self.sync();
    }

    pub(crate) fn set_slope_collision_bits(&mut self, value: u16) {
        self.state.slope_collision_bits = value;
        self.sync();
    }

    pub(crate) fn clear_slope_collision_bits(&mut self) {
        self.set_slope_collision_bits(0);
    }

    pub(crate) fn or_slope_collision_bits(&mut self, value: u16) -> u16 {
        self.state.slope_collision_bits |= value;
        let next = self.state.slope_collision_bits;
        self.sync();
        next
    }

    pub(crate) fn set_collision_bits(&mut self, value: u16) {
        self.state.collision_bits = value;
        self.sync();
    }

    pub(crate) fn clear_collision_bits(&mut self) {
        self.set_collision_bits(0);
    }

    pub(crate) fn or_collision_bits(&mut self, value: u16) -> u16 {
        self.state.collision_bits |= value;
        let next = self.state.collision_bits;
        self.sync();
        next
    }

    pub(crate) fn set_tile_probe_anchor(&mut self, value: u16) {
        self.state.interaction_scratch_x = value;
        self.sync();
    }

    pub(crate) fn clear_inroom_staircase(&mut self) {
        self.state.inroom_staircase = 0;
        self.sync();
    }

    pub(crate) fn or_inroom_staircase(&mut self, bits: u16) -> u16 {
        self.state.inroom_staircase |= bits;
        let next = self.state.inroom_staircase;
        self.sync();
        next
    }

    pub(crate) fn set_liftable_tile_index(&mut self, value: u8) {
        self.state.liftable_tile_index = value;
        self.sync();
    }

    pub(crate) fn set_tile_collision_bits_primary(&mut self, value: u8) {
        self.state.tile_collision_bits_primary = value;
        self.sync();
    }

    pub(crate) fn set_liftable_action_index_primary(&mut self, value: u8) {
        self.state.liftable_action_index_primary = value;
        self.sync();
    }

    pub(crate) fn set_liftable_action_index_secondary(&mut self, value: u8) {
        self.state.liftable_action_index_secondary = value;
        self.sync();
    }

    pub(crate) fn clear_interaction_scratch_x_low(&mut self) {
        self.state.interaction_scratch_x &= 0xff00;
        self.sync();
    }

    pub(crate) fn set_interaction_scratch_y_bytes(&mut self, low: u8, high: u8) {
        self.state.interaction_scratch_y = u16::from(low) | (u16::from(high) << 8);
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Bg1MovementAccumulatorState {
    y_subpixel: u8,
    x_subpixel: u8,
}

impl Bg1MovementAccumulatorState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            y_subpixel: ram_byte(ram, BG1_MOVE_CALC_BUFFER),
            x_subpixel: ram_byte(ram, BG1_MOVE_CALC_BUFFER + 1),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[BG1_MOVE_CALC_BUFFER] = self.y_subpixel;
        ram[BG1_MOVE_CALC_BUFFER + 1] = self.x_subpixel;
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        self.x_subpixel
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        self.y_subpixel
    }

    pub(crate) fn set_buffer(&mut self, value: u16) {
        self.y_subpixel = value as u8;
        self.x_subpixel = (value >> 8) as u8;
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.y_subpixel = value;
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.x_subpixel = value;
    }

    pub(crate) fn advance_x_subpixel(&mut self, delta: u16) -> u16 {
        let next = u16::from(self.x_subpixel).wrapping_add(delta);
        self.set_x_subpixel(next as u8);
        next
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PushedBlockState {
    x_high: [u8; PUSHED_BLOCK_BANK_LEN],
    x_low: [u8; PUSHED_BLOCK_BANK_LEN],
    target: [u8; PUSHED_BLOCK_BANK_LEN],
    y_high: [u8; PUSHED_BLOCK_BANK_LEN],
    y_low: [u8; PUSHED_BLOCK_BANK_LEN],
    subpixel: [u8; PUSHED_BLOCK_BANK_LEN],
    facing_player: [u8; PUSHED_BLOCK_BANK_LEN],
    animation_mode: u8,
    animation_timer: u8,
    push_direction: u8,
}

impl PushedBlockState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            x_high: read_pushed_block_bank(ram, PUSHEDBLOCKS_X_HI),
            x_low: read_pushed_block_bank(ram, PUSHEDBLOCKS_X_LO),
            target: read_pushed_block_bank(ram, PUSHEDBLOCKS_TARGET),
            y_high: read_pushed_block_bank(ram, PUSHEDBLOCKS_Y_HI),
            y_low: read_pushed_block_bank(ram, PUSHEDBLOCKS_Y_LO),
            subpixel: read_pushed_block_bank(ram, PUSHEDBLOCKS_SUBPIXEL),
            facing_player: read_pushed_block_bank(ram, PUSHEDBLOCK_FACING_PLAYER),
            animation_mode: ram_byte(ram, PUSHED_BLOCK_MODE),
            animation_timer: ram_byte(ram, PUSHED_BLOCK_ANIMATION_TIMER),
            push_direction: ram_byte(ram, PUSH_BLOCK_DIRECTION),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_pushed_block_bank(ram, PUSHEDBLOCKS_X_HI, self.x_high);
        write_pushed_block_bank(ram, PUSHEDBLOCKS_X_LO, self.x_low);
        write_pushed_block_bank(ram, PUSHEDBLOCKS_TARGET, self.target);
        write_pushed_block_bank(ram, PUSHEDBLOCKS_Y_HI, self.y_high);
        write_pushed_block_bank(ram, PUSHEDBLOCKS_Y_LO, self.y_low);
        write_pushed_block_bank(ram, PUSHEDBLOCKS_SUBPIXEL, self.subpixel);
        write_pushed_block_bank(ram, PUSHEDBLOCK_FACING_PLAYER, self.facing_player);
        ram[PUSHED_BLOCK_MODE] = self.animation_mode;
        ram[PUSHED_BLOCK_ANIMATION_TIMER] = self.animation_timer;
        ram[PUSH_BLOCK_DIRECTION] = self.push_direction;
    }

    pub(crate) fn x(&self, slot: usize) -> u16 {
        u16::from(self.x_low(slot)) | (u16::from(self.bank_value(self.x_high, slot)) << 8)
    }

    pub(crate) fn y(&self, slot: usize) -> u16 {
        u16::from(self.y_low(slot)) | (u16::from(self.bank_value(self.y_high, slot)) << 8)
    }

    pub(crate) fn x_low(&self, slot: usize) -> u8 {
        self.bank_value(self.x_low, slot)
    }

    pub(crate) fn y_low(&self, slot: usize) -> u8 {
        self.bank_value(self.y_low, slot)
    }

    pub(crate) fn subpixel(&self, slot: usize) -> u8 {
        self.bank_value(self.subpixel, slot)
    }

    pub(crate) fn target_low(&self, slot: usize) -> u8 {
        self.bank_value(self.target, slot)
    }

    pub(crate) fn facing_player(&self, slot: usize) -> u8 {
        self.bank_value(self.facing_player, slot)
    }

    pub(crate) fn animation_mode(&self) -> u8 {
        self.animation_mode
    }

    pub(crate) fn animation_timer(&self) -> u8 {
        self.animation_timer
    }

    pub(crate) fn push_direction(&self) -> u8 {
        self.push_direction
    }

    pub(crate) fn push_direction_index(&self) -> usize {
        usize::from((self.push_direction >> 1) & 3)
    }

    pub(crate) fn x_fixed24(&self, slot: usize) -> u32 {
        u32::from(self.subpixel(slot))
            | (u32::from(self.x_low(slot)) << 8)
            | (u32::from(self.bank_value(self.x_high, slot)) << 16)
    }

    pub(crate) fn y_fixed24(&self, slot: usize) -> u32 {
        u32::from(self.subpixel(slot))
            | (u32::from(self.y_low(slot)) << 8)
            | (u32::from(self.bank_value(self.y_high, slot)) << 16)
    }

    fn bank_value(&self, bank: [u8; PUSHED_BLOCK_BANK_LEN], slot: usize) -> u8 {
        pushed_block_bank_offset(slot)
            .and_then(|offset| bank.get(offset).copied())
            .unwrap_or(0)
    }
}

fn pushed_block_bank_offset(slot: usize) -> Option<usize> {
    let offset = slot.checked_mul(2)?;
    (offset < PUSHED_BLOCK_BANK_LEN).then_some(offset)
}

fn read_pushed_block_bank(ram: &[u8], base: usize) -> [u8; PUSHED_BLOCK_BANK_LEN] {
    let mut bank = [0; PUSHED_BLOCK_BANK_LEN];
    for (offset, value) in bank.iter_mut().enumerate() {
        *value = ram.get(base + offset).copied().unwrap_or(0);
    }
    bank
}

fn write_pushed_block_bank(ram: &mut [u8], base: usize, bank: [u8; PUSHED_BLOCK_BANK_LEN]) {
    for (offset, value) in bank.iter().copied().enumerate() {
        ram[base + offset] = value;
    }
}

fn write_pushed_block_bank_word(bank: &mut [u8; PUSHED_BLOCK_BANK_LEN], slot: usize, value: u16) {
    if let Some(offset) = pushed_block_bank_offset(slot) {
        if offset + 1 < bank.len() {
            write_le_u16(bank, offset, value);
        }
    }
}

pub(crate) struct NativeBg1MovementAccumulatorBridgeMut<'a> {
    state: &'a mut Bg1MovementAccumulatorState,
    ram: &'a mut [u8],
}

impl<'a> NativeBg1MovementAccumulatorBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut Bg1MovementAccumulatorState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            Bg1MovementAccumulatorState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_buffer(&mut self, value: u16) {
        self.state.set_buffer(value);
        self.sync();
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.state.set_y_subpixel(value);
        self.sync();
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.state.set_x_subpixel(value);
        self.sync();
    }

    pub(crate) fn advance_x_subpixel(&mut self, delta: u16) -> u16 {
        let next = self.state.advance_x_subpixel(delta);
        self.sync();
        next
    }
}

pub(crate) struct NativePushedBlockBridgeMut<'a> {
    state: &'a mut PushedBlockState,
    ram: &'a mut [u8],
}

impl<'a> NativePushedBlockBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut PushedBlockState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    pub(crate) fn set_facing_player(&mut self, slot: usize, value: u8) {
        if let Some(offset) = pushed_block_bank_offset(slot) {
            self.state.facing_player[offset] = value;
            self.sync();
        }
    }

    pub(crate) fn set_target_low(&mut self, slot: usize, value: u8) {
        if let Some(offset) = pushed_block_bank_offset(slot) {
            self.state.target[offset] = value;
            self.sync();
        }
    }

    pub(crate) fn set_animation_mode(&mut self, value: u8) {
        self.state.animation_mode = value;
        self.sync();
    }

    pub(crate) fn reset_animation_timer(&mut self) {
        self.state.animation_timer = 9;
        self.sync();
    }

    pub(crate) fn decrement_animation_timer(&mut self) -> u8 {
        self.state.animation_timer = self.state.animation_timer.wrapping_sub(1);
        self.sync();
        self.state.animation_timer
    }

    pub(crate) fn advance_animation_mode(&mut self) -> u8 {
        self.state.animation_timer = 9;
        self.state.animation_mode = self.state.animation_mode.wrapping_add(1);
        self.sync();
        self.state.animation_mode
    }

    pub(crate) fn init_slot(&mut self, slot: usize, x: u16, y: u16) {
        write_pushed_block_bank_word(&mut self.state.x_low, slot, x & 0x00ff);
        write_pushed_block_bank_word(&mut self.state.x_high, slot, x >> 8);
        write_pushed_block_bank_word(&mut self.state.y_low, slot, y & 0x00ff);
        write_pushed_block_bank_word(&mut self.state.y_high, slot, y >> 8);
        write_pushed_block_bank_word(&mut self.state.target, slot, 0);
        write_pushed_block_bank_word(&mut self.state.subpixel, slot, 0);
        self.sync();
    }

    pub(crate) fn set_push_direction(&mut self, value: u8) {
        self.state.push_direction = value;
        self.sync();
    }

    pub(crate) fn set_x_fixed24(&mut self, slot: usize, value: u32) {
        if let Some(offset) = pushed_block_bank_offset(slot) {
            self.state.subpixel[offset] = value as u8;
            self.state.x_low[offset] = (value >> 8) as u8;
            self.state.x_high[offset] = (value >> 16) as u8;
            self.sync();
        }
    }

    pub(crate) fn set_y_fixed24(&mut self, slot: usize, value: u32) {
        if let Some(offset) = pushed_block_bank_offset(slot) {
            self.state.subpixel[offset] = value as u8;
            self.state.y_low[offset] = (value >> 8) as u8;
            self.state.y_high[offset] = (value >> 16) as u8;
            self.sync();
        }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, PushedBlockState::load_from_ram(self.ram));
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SwimAccelerationState {
    mode: [u16; SWIM_AXIS_COUNT],
    speed_active_flag: [u16; SWIM_AXIS_COUNT],
    max_speed: [u16; SWIM_AXIS_COUNT],
    acceleration_direction: [u16; SWIM_AXIS_COUNT],
    acceleration: [u16; SWIM_AXIS_COUNT],
}

impl SwimAccelerationState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            mode: read_axis_words(ram, SWIM_ACCELERATION_MODE),
            speed_active_flag: read_axis_words(ram, SWIM_SPEED_ACTIVE_FLAG),
            max_speed: read_axis_words(ram, SWIM_MAX_SPEED),
            acceleration_direction: read_axis_words(ram, SWIM_ACCELERATION_DIRECTION),
            acceleration: read_axis_words(ram, SWIM_ACCELERATION),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_axis_words(ram, SWIM_ACCELERATION_MODE, self.mode);
        write_axis_words(ram, SWIM_SPEED_ACTIVE_FLAG, self.speed_active_flag);
        write_axis_words(ram, SWIM_MAX_SPEED, self.max_speed);
        write_axis_words(
            ram,
            SWIM_ACCELERATION_DIRECTION,
            self.acceleration_direction,
        );
        write_axis_words(ram, SWIM_ACCELERATION, self.acceleration);
    }

    pub(crate) fn mode(&self, offset: usize) -> u16 {
        axis_word(self.mode, offset)
    }

    pub(crate) fn mode_low(&self, axis: usize) -> u8 {
        self.mode.get(axis).copied().unwrap_or(0) as u8
    }

    pub(crate) fn speed_active_flag(&self, offset: usize) -> u16 {
        axis_word(self.speed_active_flag, offset)
    }

    pub(crate) fn max_speed(&self, offset: usize) -> u16 {
        axis_word(self.max_speed, offset)
    }

    pub(crate) fn acceleration_direction(&self, offset: usize) -> u16 {
        axis_word(self.acceleration_direction, offset)
    }

    pub(crate) fn acceleration(&self, offset: usize) -> u16 {
        axis_word(self.acceleration, offset)
    }

    pub(crate) fn has_any_acceleration(&self) -> bool {
        self.acceleration[0] | self.acceleration[1] != 0
    }
}

fn read_axis_words(ram: &[u8], base: usize) -> [u16; SWIM_AXIS_COUNT] {
    [
        if base + 1 < ram.len() {
            read_le_u16(ram, base)
        } else {
            0
        },
        if base + 3 < ram.len() {
            read_le_u16(ram, base + 2)
        } else {
            0
        },
    ]
}

fn write_axis_words(ram: &mut [u8], base: usize, values: [u16; SWIM_AXIS_COUNT]) {
    write_le_u16(ram, base, values[0]);
    write_le_u16(ram, base + 2, values[1]);
}

fn axis_word(values: [u16; SWIM_AXIS_COUNT], offset: usize) -> u16 {
    swim_axis_index(offset)
        .and_then(|axis| values.get(axis).copied())
        .unwrap_or(0)
}

pub(crate) struct NativeSwimAccelerationBridgeMut<'a> {
    state: &'a mut SwimAccelerationState,
    ram: &'a mut [u8],
}

impl<'a> NativeSwimAccelerationBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SwimAccelerationState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, SwimAccelerationState::load_from_ram(self.ram));
    }

    pub(crate) fn set_mode(&mut self, offset: usize, value: u16) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.mode[axis] = value;
            self.sync();
        }
    }

    pub(crate) fn clear_mode_low_axis(&mut self) {
        self.state.mode[0] = 0;
        self.sync();
    }

    pub(crate) fn set_speed_active_flag(&mut self, offset: usize, value: u16) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.speed_active_flag[axis] = value;
            self.sync();
        }
    }

    pub(crate) fn set_max_speed(&mut self, offset: usize, value: u16) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.max_speed[axis] = value;
            self.sync();
        }
    }

    pub(crate) fn set_max_speed_both_axes(&mut self, value: u16) {
        self.state.max_speed = [value; SWIM_AXIS_COUNT];
        self.sync();
    }

    pub(crate) fn set_acceleration_direction(&mut self, offset: usize, value: u16) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.acceleration_direction[axis] = value;
            self.sync();
        }
    }

    pub(crate) fn set_acceleration(&mut self, offset: usize, value: u16) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.acceleration[axis] = value;
            self.sync();
        }
    }

    pub(crate) fn clear_axis_motion(&mut self, offset: usize) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.speed_active_flag[axis] = 0;
            self.state.mode[axis] = 0;
            self.state.acceleration[axis] = 0;
            self.state.max_speed[axis] = 0;
            self.sync();
        }
    }
}

pub(crate) struct NativeSpecialExitPositionBridgeMut<'a> {
    state: &'a mut SpecialExitPositionState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpecialExitPositionBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SpecialExitPositionState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            SpecialExitPositionState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.state.x = value;
        self.sync();
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.state.y = value;
        self.sync();
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.x = x;
        self.state.y = y;
        self.sync();
    }

    pub(crate) fn offset_position(&mut self, x_delta: u16, y_delta: u16) {
        self.state.x = self.state.x.wrapping_add(x_delta);
        self.state.y = self.state.y.wrapping_add(y_delta);
        self.sync();
    }

    pub(crate) fn store_from_player(&mut self) {
        self.state.x = u16::from(ram_byte(self.ram, LINK_X_COORD))
            | (u16::from(ram_byte(self.ram, LINK_X_COORD + 1)) << 8);
        self.state.y = u16::from(ram_byte(self.ram, LINK_Y_COORD))
            | (u16::from(ram_byte(self.ram, LINK_Y_COORD + 1)) << 8);
        self.sync();
    }

    pub(crate) fn restore_player_position(&mut self) {
        write_le_u16(self.ram, LINK_X_COORD, self.state.x);
        write_le_u16(self.ram, LINK_Y_COORD, self.state.y);
    }
}
