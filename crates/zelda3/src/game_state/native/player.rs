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

    pub(crate) fn set_x(&mut self, value: u16) {
        self.x = value;
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.y = value;
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub(crate) fn offset_position(&mut self, x_delta: u16, y_delta: u16) {
        self.x = self.x.wrapping_add(x_delta);
        self.y = self.y.wrapping_add(y_delta);
    }

    pub(crate) fn store_from_player_ram(&mut self, ram: &[u8]) {
        self.x = u16::from(ram_byte(ram, LINK_X_COORD))
            | (u16::from(ram_byte(ram, LINK_X_COORD + 1)) << 8);
        self.y = u16::from(ram_byte(ram, LINK_Y_COORD))
            | (u16::from(ram_byte(ram, LINK_Y_COORD + 1)) << 8);
    }

    pub(crate) fn restore_player_position_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, LINK_X_COORD, self.x);
        write_le_u16(ram, LINK_Y_COORD, self.y);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlayerState {
    pub(crate) special_exit_position: SpecialExitPositionState,
    pub(crate) follower_link: FollowerLinkState,
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
            follower_link: FollowerLinkState::load_from_ram(ram),
            swim_acceleration: SwimAccelerationState::load_from_ram(ram),
            pushed_block: PushedBlockState::load_from_ram(ram),
            bg1_movement_accumulator: Bg1MovementAccumulatorState::load_from_ram(ram),
            tile_detection: TileDetectionState::load_from_ram(ram),
            tile_attributes: PlayerTileAttributeTableState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.special_exit_position.write_to_ram(ram);
        self.follower_link.write_to_ram(ram);
        self.swim_acceleration.write_to_ram(ram);
        self.pushed_block.write_to_ram(ram);
        self.bg1_movement_accumulator.write_to_ram(ram);
        self.tile_detection.write_to_ram(ram);
        self.tile_attributes.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlayerSnapshotState {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) z: u16,
    pub(crate) x_velocity: u8,
    pub(crate) y_velocity: u8,
    pub(crate) z_velocity: u8,
    pub(crate) direction: u8,
    pub(crate) last_direction: u8,
    pub(crate) facing: u8,
    pub(crate) handler_state: u8,
    pub(crate) auxiliary_state: u8,
    pub(crate) current_health: u8,
    pub(crate) magic_power: u8,
    pub(crate) equipped_item: u8,
    pub(crate) item_in_hand: u8,
    pub(crate) current_item_y: u8,
    pub(crate) current_item_active: u8,
}

impl PlayerSnapshotState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            x: read_le_u16(ram, LINK_X_COORD),
            y: read_le_u16(ram, LINK_Y_COORD),
            z: read_le_u16(ram, LINK_Z_COORD),
            x_velocity: ram_byte(ram, LINK_X_VELOCITY),
            y_velocity: ram_byte(ram, LINK_Y_VELOCITY),
            z_velocity: ram_byte(ram, LINK_Z_VELOCITY),
            direction: ram_byte(ram, LINK_DIRECTION),
            last_direction: ram_byte(ram, LINK_LAST_DIRECTION),
            facing: ram_byte(ram, LINK_FACING),
            handler_state: ram_byte(ram, LINK_HANDLER_STATE),
            auxiliary_state: ram_byte(ram, LINK_AUXILIARY_STATE),
            current_health: ram_byte(ram, LINK_CURRENT_HEALTH),
            magic_power: ram_byte(ram, LINK_MAGIC_POWER),
            equipped_item: ram_byte(ram, LINK_EQUIPPED_ITEM),
            item_in_hand: ram_byte(ram, LINK_ITEM_IN_HAND),
            current_item_y: ram_byte(ram, LINK_CURRENT_ITEM_Y),
            current_item_active: ram_byte(ram, LINK_CURRENT_ITEM_ACTIVE),
        }
    }
}

const PLAYER_HANDLER_STATE_GROUND: u8 = 0;
const PLAYER_HANDLER_STATE_SWIMMING: u8 = 4;
const PLAYER_HANDLER_STATE_RECOIL_OTHER: u8 = 6;
const PLAYER_HANDLER_STATE_ETHER: u8 = 8;
const PLAYER_HANDLER_STATE_BOMBOS: u8 = 9;
const PLAYER_HANDLER_STATE_QUAKE: u8 = 10;
const PLAYER_HANDLER_STATE_START_DASH: u8 = 17;
const PLAYER_HANDLER_STATE_HOOKSHOT: u8 = 19;

const FOLLOWER_LAYER_BITS_BY_FLOOR: [u8; 4] = [0x20, 0x10, 0x30, 0x20];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct FollowerLinkState {
    x: u16,
    y: u16,
    z: u16,
    x_velocity: u8,
    y_velocity: u8,
    floor: u8,
    facing: u8,
    speed_setting: u8,
    handler_state: u8,
    immobilized: u8,
    action_state_bits: u8,
    auxiliary_state: u8,
    running: u8,
    button_mask_b_y: u8,
    pull_action_state: u8,
    item_in_hand: u8,
    position_mode: u8,
    ancilla_pickup_flag: u8,
    sprite_pickup_flag: u8,
    grabbing_wall: u8,
    sprite_damage_disabled: u8,
}

impl FollowerLinkState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            x: read_le_u16(ram, LINK_X_COORD),
            y: read_le_u16(ram, LINK_Y_COORD),
            z: read_le_u16(ram, LINK_Z_COORD),
            x_velocity: ram_byte(ram, LINK_X_VELOCITY),
            y_velocity: ram_byte(ram, LINK_Y_VELOCITY),
            floor: ram_byte(ram, LINK_IS_ON_LOWER_LEVEL),
            facing: ram_byte(ram, LINK_FACING),
            speed_setting: ram_byte(ram, LINK_SPEED_SETTING),
            handler_state: ram_byte(ram, LINK_HANDLER_STATE),
            immobilized: ram_byte(ram, FLAG_IS_LINK_IMMOBILIZED),
            action_state_bits: ram_byte(ram, LINK_STATE_BITS),
            auxiliary_state: ram_byte(ram, LINK_AUXILIARY_STATE),
            running: ram_byte(ram, LINK_IS_RUNNING),
            button_mask_b_y: ram_byte(ram, BUTTON_MASK_B_Y),
            pull_action_state: ram_byte(ram, LINK_PULL_ACTION_STATE),
            item_in_hand: ram_byte(ram, LINK_ITEM_IN_HAND),
            position_mode: ram_byte(ram, LINK_POSITION_MODE),
            ancilla_pickup_flag: ram_byte(ram, FLAG_IS_ANCILLA_TO_PICK_UP),
            sprite_pickup_flag: ram_byte(ram, FLAG_IS_SPRITE_TO_PICK_UP),
            grabbing_wall: ram_byte(ram, LINK_GRABBING_WALL),
            sprite_damage_disabled: ram_byte(ram, LINK_DISABLE_SPRITE_DAMAGE),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, LINK_X_COORD, self.x);
        write_le_u16(ram, LINK_Y_COORD, self.y);
        write_le_u16(ram, LINK_Z_COORD, self.z);
        ram[LINK_X_VELOCITY] = self.x_velocity;
        ram[LINK_Y_VELOCITY] = self.y_velocity;
        ram[LINK_IS_ON_LOWER_LEVEL] = self.floor;
        ram[LINK_FACING] = self.facing;
        ram[LINK_SPEED_SETTING] = self.speed_setting;
        ram[LINK_HANDLER_STATE] = self.handler_state;
        ram[FLAG_IS_LINK_IMMOBILIZED] = self.immobilized;
        ram[LINK_STATE_BITS] = self.action_state_bits;
        ram[LINK_AUXILIARY_STATE] = self.auxiliary_state;
        ram[LINK_IS_RUNNING] = self.running;
        ram[BUTTON_MASK_B_Y] = self.button_mask_b_y;
        ram[LINK_PULL_ACTION_STATE] = self.pull_action_state;
        ram[LINK_ITEM_IN_HAND] = self.item_in_hand;
        ram[LINK_POSITION_MODE] = self.position_mode;
        ram[FLAG_IS_ANCILLA_TO_PICK_UP] = self.ancilla_pickup_flag;
        ram[FLAG_IS_SPRITE_TO_PICK_UP] = self.sprite_pickup_flag;
        ram[LINK_GRABBING_WALL] = self.grabbing_wall;
        ram[LINK_DISABLE_SPRITE_DAMAGE] = self.sprite_damage_disabled;
    }

    pub(crate) fn x(&self) -> u16 {
        self.x
    }

    pub(crate) fn y(&self) -> u16 {
        self.y
    }

    pub(crate) fn z_for_follow(&self) -> u8 {
        let z = self.z as u8;
        if z >= 0xf0 {
            0
        } else {
            z
        }
    }

    pub(crate) fn z_for_oam(&self) -> u8 {
        let z = self.z as u8;
        if self.z < 0x8000 || z < 0xf0 {
            z
        } else {
            0
        }
    }

    pub(crate) fn is_moving(&self) -> bool {
        (self.x_velocity | self.y_velocity) != 0
    }

    pub(crate) fn floor(&self) -> u8 {
        self.floor
    }

    pub(crate) fn floor_layer_bits(&self) -> u8 {
        FOLLOWER_LAYER_BITS_BY_FLOOR[self.floor as usize] >> 2
    }

    pub(crate) fn oam_priority_for_floor(&self) -> u8 {
        Self::oam_priority_for_floor_value(self.floor)
    }

    pub(crate) fn oam_priority_for_floor_value(floor: u8) -> u8 {
        FOLLOWER_LAYER_BITS_BY_FLOOR[floor as usize]
    }

    pub(crate) fn facing_layer_bits(&self) -> u8 {
        self.facing >> 1
    }

    pub(crate) fn speed_setting(&self) -> u8 {
        self.speed_setting
    }

    pub(crate) fn is_ground_swim_or_dash_start(&self) -> bool {
        matches!(
            self.handler_state,
            PLAYER_HANDLER_STATE_GROUND
                | PLAYER_HANDLER_STATE_SWIMMING
                | PLAYER_HANDLER_STATE_START_DASH
        )
    }

    pub(crate) fn is_using_medallion(&self) -> bool {
        matches!(
            self.handler_state,
            PLAYER_HANDLER_STATE_ETHER | PLAYER_HANDLER_STATE_BOMBOS | PLAYER_HANDLER_STATE_QUAKE
        )
    }

    pub(crate) fn is_swimming(&self) -> bool {
        self.handler_state == PLAYER_HANDLER_STATE_SWIMMING
    }

    pub(crate) fn is_immobilized(&self) -> bool {
        self.immobilized != 0
    }

    pub(crate) fn is_hookshot(&self) -> bool {
        self.handler_state == PLAYER_HANDLER_STATE_HOOKSHOT
    }

    pub(crate) fn is_recoiling_from_other_source(&self) -> bool {
        self.handler_state == PLAYER_HANDLER_STATE_RECOIL_OTHER
    }

    pub(crate) fn has_action_state(&self) -> bool {
        self.action_state_bits != 0
    }

    pub(crate) fn is_lifting_or_carrying(&self) -> bool {
        self.action_state_bits & 0x80 != 0
    }

    pub(crate) fn auxiliary_state(&self) -> u8 {
        self.auxiliary_state
    }

    pub(crate) fn has_auxiliary_state(&self) -> bool {
        self.auxiliary_state != 0
    }

    pub(crate) fn is_running(&self) -> bool {
        self.running != 0
    }

    pub(crate) fn can_open_follower_message(&self) -> bool {
        let blocked = (self.button_mask_b_y & 0x80)
            | self.pull_action_state
            | self.item_in_hand
            | self.position_mode
            | self.ancilla_pickup_flag
            | self.sprite_pickup_flag
            | self.action_state_bits
            | self.grabbing_wall;
        self.is_ground_swim_or_dash_start() && blocked == 0
    }

    pub(crate) fn can_drop_follower(&self) -> bool {
        self.auxiliary_state != 1 && !self.is_lifting_or_carrying()
    }

    pub(crate) fn should_transform_old_man_from_recoil(&self) -> bool {
        (self.auxiliary_state & 1) != 0 && self.is_recoiling_from_other_source()
    }

    pub(crate) fn should_transform_old_man_from_auxiliary_state(&self) -> bool {
        self.auxiliary_state & 2 != 0
    }

    pub(crate) fn can_reacquire_old_man(&self) -> bool {
        !self.is_running() && !self.has_auxiliary_state() && !self.is_swimming()
    }

    fn set_speed_setting(&mut self, value: u8) {
        self.speed_setting = value;
    }

    fn set_ground_state(&mut self) {
        self.handler_state = PLAYER_HANDLER_STATE_GROUND;
    }

    fn clear_running(&mut self) {
        self.running = 0;
    }

    fn immobilize(&mut self) {
        self.immobilized = 1;
    }

    fn enable_cutscene_immunity(&mut self) {
        self.sprite_damage_disabled = 1;
    }
}

pub(crate) struct NativeFollowerLinkBridgeMut<'a> {
    state: &'a mut FollowerLinkState,
    ram: &'a mut [u8],
}

impl<'a> NativeFollowerLinkBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut FollowerLinkState, ram: &'a mut [u8]) -> Self {
        let mut bridge = Self { state, ram };
        bridge.sync_from_ram();
        bridge
    }

    fn sync_from_ram(&mut self) {
        *self.state = FollowerLinkState::load_from_ram(self.ram);
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, FollowerLinkState::load_from_ram(self.ram));
    }

    pub(crate) fn set_speed_setting(&mut self, value: u8) {
        self.state.set_speed_setting(value);
        self.ram[LINK_SPEED_SETTING] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_ground_state(&mut self) {
        self.state.set_ground_state();
        self.ram[LINK_HANDLER_STATE] = PLAYER_HANDLER_STATE_GROUND;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_running(&mut self) {
        self.state.clear_running();
        self.ram[LINK_IS_RUNNING] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn immobilize(&mut self) {
        self.state.immobilize();
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn enable_cutscene_immunity(&mut self) {
        self.state.enable_cutscene_immunity();
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        self.debug_assert_matches_ram();
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
    layer_collision_flags: u8,
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
            layer_collision_flags: ram_byte(ram, PLAYER_LAYER_COLLISION_FLAGS),
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
        ram[PLAYER_LAYER_COLLISION_FLAGS] = self.layer_collision_flags;
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

    pub(crate) fn has_layer_collision(&self, mask: u8) -> bool {
        self.layer_collision_flags & mask == mask
    }

    pub(crate) fn palette_bits_high(&self) -> u8 {
        self.palette_bits_high
    }

    pub(crate) fn inroom_staircase(&self) -> u16 {
        self.inroom_staircase
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.probe_y = (self.probe_y & 0x00ff) | (u16::from(value) << 8);
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.probe_y = value;
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.probe_x = value;
    }

    pub(crate) fn set_location_calc_mask(&mut self, value: u16) {
        self.location_calc_mask = value;
    }

    pub(crate) fn set_interacting_tile(&mut self, value: u16) {
        self.interacting_tile = value;
    }

    pub(crate) fn set_interacting_tile_low(&mut self, value: u8) {
        self.interacting_tile = (self.interacting_tile & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_fall_hole_scan_index(&mut self, value: u8) {
        self.fall_hole_scan_index = value;
    }

    pub(crate) fn set_interaction_scratch_y(&mut self, value: u16) {
        self.interaction_scratch_y = value;
    }

    pub(crate) fn set_interaction_scratch_x(&mut self, value: u16) {
        self.interaction_scratch_x = value;
    }

    pub(crate) fn set_diagonal_tile(&mut self, value: u16) {
        self.diagonal_tile = value;
    }

    pub(crate) fn clear_diagonal_tile(&mut self) {
        self.diagonal_tile = 0;
    }

    pub(crate) fn or_diagonal_tile(&mut self, value: u16) -> u16 {
        self.diagonal_tile |= value;
        self.diagonal_tile
    }

    pub(crate) fn set_stair_tile(&mut self, value: u8) {
        self.stair_tile = value;
    }

    pub(crate) fn clear_stair_tile(&mut self) {
        self.stair_tile = 0;
    }

    pub(crate) fn or_stair_tile(&mut self, value: u8) {
        self.stair_tile |= value;
    }

    pub(crate) fn set_block_flags(&mut self, value: u16) {
        self.block_flags = value;
    }

    pub(crate) fn clear_block_flags(&mut self) {
        self.block_flags = 0;
    }

    pub(crate) fn or_block_flags(&mut self, value: u16) -> u16 {
        self.block_flags |= value;
        self.block_flags
    }

    pub(crate) fn set_door_direction_flags(&mut self, value: u16) {
        self.door_direction_flags = value;
    }

    pub(crate) fn clear_door_direction_flags(&mut self) {
        self.door_direction_flags = 0;
    }

    pub(crate) fn set_diag_state(&mut self, value: u16) {
        self.diag_state = value;
    }

    pub(crate) fn clear_diag_state(&mut self) {
        self.diag_state = 0;
    }

    pub(crate) fn clear_pit_tile(&mut self) {
        self.pit_tile = 0;
    }

    pub(crate) fn or_pit_tile(&mut self, value: u8) {
        self.pit_tile |= u16::from(value);
    }

    pub(crate) fn set_deepwater(&mut self, value: u16) {
        self.deepwater = value;
    }

    pub(crate) fn clear_deepwater(&mut self) {
        self.deepwater = 0;
    }

    pub(crate) fn or_deepwater(&mut self, value: u16) -> u16 {
        self.deepwater |= value;
        self.deepwater
    }

    pub(crate) fn set_normal_tiles(&mut self, value: u16) {
        self.normal_tiles = value;
    }

    pub(crate) fn clear_normal_tiles(&mut self) {
        self.normal_tiles = 0;
    }

    pub(crate) fn or_normal_tiles(&mut self, value: u16) -> u16 {
        self.normal_tiles |= value;
        self.normal_tiles
    }

    pub(crate) fn set_misc_tiles(&mut self, value: u16) {
        self.misc_tiles = value;
    }

    pub(crate) fn clear_misc_tiles(&mut self) {
        self.misc_tiles = 0;
    }

    pub(crate) fn or_misc_tiles(&mut self, value: u16) -> u16 {
        self.misc_tiles |= value;
        self.misc_tiles
    }

    pub(crate) fn set_thick_grass(&mut self, value: u16) {
        self.thick_grass = value;
    }

    pub(crate) fn clear_thick_grass(&mut self) {
        self.thick_grass = 0;
    }

    pub(crate) fn or_thick_grass(&mut self, value: u16) -> u16 {
        self.thick_grass |= value;
        self.thick_grass
    }

    pub(crate) fn clear_vertical_ledge(&mut self) {
        self.vertical_ledge = 0;
    }

    pub(crate) fn or_vertical_ledge(&mut self, value: u8) {
        self.vertical_ledge |= value;
    }

    pub(crate) fn clear_horizontal_ledge(&mut self) {
        self.horizontal_ledge = 0;
    }

    pub(crate) fn or_horizontal_ledge(&mut self, value: u8) {
        self.horizontal_ledge |= value;
    }

    pub(crate) fn set_moving_floor_tiles(&mut self, value: u16) {
        self.moving_floor_tiles = value;
    }

    pub(crate) fn clear_moving_floor_tiles(&mut self) {
        self.moving_floor_tiles = 0;
    }

    pub(crate) fn or_moving_floor_tiles(&mut self, value: u16) -> u16 {
        self.moving_floor_tiles |= value;
        self.moving_floor_tiles
    }

    pub(crate) fn set_icy_floor(&mut self, value: u16) {
        self.icy_floor = value;
    }

    pub(crate) fn clear_icy_floor(&mut self) {
        self.icy_floor = 0;
    }

    pub(crate) fn or_icy_floor(&mut self, value: u16) -> u16 {
        self.icy_floor |= value;
        self.icy_floor
    }

    pub(crate) fn set_water_staircase(&mut self, value: u16) {
        self.water_staircase = value;
    }

    pub(crate) fn clear_water_staircase(&mut self) {
        self.water_staircase = 0;
    }

    pub(crate) fn or_water_staircase(&mut self, value: u16) -> u16 {
        self.water_staircase |= value;
        self.water_staircase
    }

    pub(crate) fn set_shallow_water(&mut self, value: u16) {
        self.shallow_water = value;
    }

    pub(crate) fn clear_shallow_water(&mut self) {
        self.shallow_water = 0;
    }

    pub(crate) fn or_shallow_water(&mut self, value: u16) -> u16 {
        self.shallow_water |= value;
        self.shallow_water
    }

    pub(crate) fn set_destruction_aftermath(&mut self, value: u16) {
        self.destruction_aftermath = value;
    }

    pub(crate) fn clear_destruction_aftermath(&mut self) {
        self.destruction_aftermath = 0;
    }

    pub(crate) fn or_destruction_aftermath(&mut self, value: u16) -> u16 {
        self.destruction_aftermath |= value;
        self.destruction_aftermath
    }

    pub(crate) fn set_read_something(&mut self, value: u16) {
        self.read_something = value;
    }

    pub(crate) fn clear_read_something(&mut self) {
        self.read_something = 0;
    }

    pub(crate) fn or_read_something(&mut self, value: u16) -> u16 {
        self.read_something |= value;
        self.read_something
    }

    pub(crate) fn set_ledges_down_leftright(&mut self, value: u8) {
        self.ledges_down_leftright = value;
    }

    pub(crate) fn clear_ledges_down_leftright(&mut self) {
        self.ledges_down_leftright = 0;
    }

    pub(crate) fn or_ledges_down_leftright(&mut self, value: u8) {
        self.ledges_down_leftright |= value;
    }

    pub(crate) fn set_diagonal_ledge_tiles(&mut self, value: u8) {
        self.diagonal_ledge_tiles = value;
    }

    pub(crate) fn clear_diagonal_ledge_tiles(&mut self) {
        self.diagonal_ledge_tiles = 0;
    }

    pub(crate) fn or_diagonal_ledge_tiles(&mut self, value: u8) {
        self.diagonal_ledge_tiles |= value;
    }

    pub(crate) fn set_chest(&mut self, value: u16) {
        self.chest = value;
    }

    pub(crate) fn clear_chest(&mut self) {
        self.chest = 0;
    }

    pub(crate) fn or_chest(&mut self, value: u16) -> u16 {
        self.chest |= value;
        self.chest
    }

    pub(crate) fn set_key_lock_gravestones(&mut self, value: u8) {
        self.key_lock_gravestones = (self.key_lock_gravestones & 0xff00) | u16::from(value);
    }

    pub(crate) fn clear_key_lock_gravestones(&mut self) {
        self.set_key_lock_gravestones(0);
    }

    pub(crate) fn or_key_lock_gravestones(&mut self, value: u8) {
        self.key_lock_gravestones |= u16::from(value);
    }

    pub(crate) fn set_spike_cactus_tiles(&mut self, value: u8) {
        self.key_lock_gravestones = (self.key_lock_gravestones & 0x00ff) | (u16::from(value) << 8);
    }

    pub(crate) fn clear_spike_cactus_tiles(&mut self) {
        self.set_spike_cactus_tiles(0);
    }

    pub(crate) fn or_spike_cactus_tiles(&mut self, value: u8) {
        self.set_spike_cactus_tiles(self.spike_cactus_tiles() | value);
    }

    pub(crate) fn set_tile_type(&mut self, value: u16) {
        self.tile_type = value;
    }

    pub(crate) fn clear_tile_type(&mut self) {
        self.tile_type = 0;
    }

    pub(crate) fn set_spike_floor_and_triggers(&mut self, value: u8) {
        self.spike_floor_and_triggers = value;
    }

    pub(crate) fn clear_spike_floor_and_triggers(&mut self) {
        self.spike_floor_and_triggers = 0;
    }

    pub(crate) fn or_spike_floor_and_triggers(&mut self, value: u8) {
        self.spike_floor_and_triggers |= value;
    }

    pub(crate) fn set_dashable_tiles(&mut self, value: u8) {
        self.dashable_tiles = value;
    }

    pub(crate) fn clear_dashable_tiles(&mut self) {
        self.dashable_tiles = 0;
    }

    pub(crate) fn or_dashable_tiles(&mut self, value: u8) {
        self.dashable_tiles |= value;
    }

    pub(crate) fn set_staircase_cache(&mut self, value: u8) {
        self.staircase_cache = value;
    }

    pub(crate) fn set_slope_collision_bits(&mut self, value: u16) {
        self.slope_collision_bits = value;
    }

    pub(crate) fn clear_slope_collision_bits(&mut self) {
        self.slope_collision_bits = 0;
    }

    pub(crate) fn or_slope_collision_bits(&mut self, value: u16) -> u16 {
        self.slope_collision_bits |= value;
        self.slope_collision_bits
    }

    pub(crate) fn set_collision_bits(&mut self, value: u16) {
        self.collision_bits = value;
    }

    pub(crate) fn clear_collision_bits(&mut self) {
        self.collision_bits = 0;
    }

    pub(crate) fn or_collision_bits(&mut self, value: u16) -> u16 {
        self.collision_bits |= value;
        self.collision_bits
    }

    pub(crate) fn set_layer_collision(&mut self, mask: u8, enabled: bool) {
        self.layer_collision_flags = if enabled {
            self.layer_collision_flags | mask
        } else {
            self.layer_collision_flags & !mask
        };
    }

    pub(crate) fn set_layer_collision_flags(&mut self, value: u8) {
        self.layer_collision_flags = value;
    }

    pub(crate) fn set_tile_probe_anchor(&mut self, value: u16) {
        self.interaction_scratch_x = value;
    }

    pub(crate) fn clear_inroom_staircase(&mut self) {
        self.inroom_staircase = 0;
    }

    pub(crate) fn or_inroom_staircase(&mut self, bits: u16) -> u16 {
        self.inroom_staircase |= bits;
        self.inroom_staircase
    }

    pub(crate) fn set_liftable_tile_index(&mut self, value: u8) {
        self.liftable_tile_index = value;
    }

    pub(crate) fn set_tile_collision_bits_primary(&mut self, value: u8) {
        self.tile_collision_bits_primary = value;
    }

    pub(crate) fn set_liftable_action_index_primary(&mut self, value: u8) {
        self.liftable_action_index_primary = value;
    }

    pub(crate) fn set_liftable_action_index_secondary(&mut self, value: u8) {
        self.liftable_action_index_secondary = value;
    }

    pub(crate) fn clear_interaction_scratch_x_low(&mut self) {
        self.interaction_scratch_x &= 0xff00;
    }

    pub(crate) fn set_interaction_scratch_y_bytes(&mut self, low: u8, high: u8) {
        self.interaction_scratch_y = u16::from(low) | (u16::from(high) << 8);
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
        self.state.set_y_high(value);
        self.sync();
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.state.set_y(value);
        self.sync();
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.state.set_x(value);
        self.sync();
    }

    pub(crate) fn set_location_calc_mask(&mut self, value: u16) {
        self.state.set_location_calc_mask(value);
        self.sync();
    }

    pub(crate) fn set_interacting_tile(&mut self, value: u16) {
        self.state.set_interacting_tile(value);
        self.sync();
    }

    pub(crate) fn set_interacting_tile_low(&mut self, value: u8) {
        self.state.set_interacting_tile_low(value);
        self.sync();
    }

    pub(crate) fn set_fall_hole_scan_index(&mut self, value: u8) {
        self.state.set_fall_hole_scan_index(value);
        self.sync();
    }

    pub(crate) fn set_interaction_scratch_y(&mut self, value: u16) {
        self.state.set_interaction_scratch_y(value);
        self.sync();
    }

    pub(crate) fn set_interaction_scratch_x(&mut self, value: u16) {
        self.state.set_interaction_scratch_x(value);
        self.sync();
    }

    pub(crate) fn set_diagonal_tile(&mut self, value: u16) {
        self.state.set_diagonal_tile(value);
        self.sync();
    }

    pub(crate) fn clear_diagonal_tile(&mut self) {
        self.state.clear_diagonal_tile();
        self.sync();
    }

    pub(crate) fn or_diagonal_tile(&mut self, value: u16) -> u16 {
        let next = self.state.or_diagonal_tile(value);
        self.sync();
        next
    }

    pub(crate) fn set_stair_tile(&mut self, value: u8) {
        self.state.set_stair_tile(value);
        self.sync();
    }

    pub(crate) fn clear_stair_tile(&mut self) {
        self.state.clear_stair_tile();
        self.sync();
    }

    pub(crate) fn or_stair_tile(&mut self, value: u8) {
        self.state.or_stair_tile(value);
        self.sync();
    }

    pub(crate) fn set_block_flags(&mut self, value: u16) {
        self.state.set_block_flags(value);
        self.sync();
    }

    pub(crate) fn clear_block_flags(&mut self) {
        self.state.clear_block_flags();
        self.sync();
    }

    pub(crate) fn or_block_flags(&mut self, value: u16) -> u16 {
        let next = self.state.or_block_flags(value);
        self.sync();
        next
    }

    pub(crate) fn set_door_direction_flags(&mut self, value: u16) {
        self.state.set_door_direction_flags(value);
        self.sync();
    }

    pub(crate) fn clear_door_direction_flags(&mut self) {
        self.state.clear_door_direction_flags();
        self.sync();
    }

    pub(crate) fn set_diag_state(&mut self, value: u16) {
        self.state.set_diag_state(value);
        self.sync();
    }

    pub(crate) fn clear_diag_state(&mut self) {
        self.state.clear_diag_state();
        self.sync();
    }

    pub(crate) fn clear_pit_tile(&mut self) {
        self.state.clear_pit_tile();
        self.sync();
    }

    pub(crate) fn or_pit_tile(&mut self, value: u8) {
        self.state.or_pit_tile(value);
        self.sync();
    }

    pub(crate) fn set_deepwater(&mut self, value: u16) {
        self.state.set_deepwater(value);
        self.sync();
    }

    pub(crate) fn clear_deepwater(&mut self) {
        self.state.clear_deepwater();
        self.sync();
    }

    pub(crate) fn or_deepwater(&mut self, value: u16) -> u16 {
        let next = self.state.or_deepwater(value);
        self.sync();
        next
    }

    pub(crate) fn set_normal_tiles(&mut self, value: u16) {
        self.state.set_normal_tiles(value);
        self.sync();
    }

    pub(crate) fn clear_normal_tiles(&mut self) {
        self.state.clear_normal_tiles();
        self.sync();
    }

    pub(crate) fn or_normal_tiles(&mut self, value: u16) -> u16 {
        let next = self.state.or_normal_tiles(value);
        self.sync();
        next
    }

    pub(crate) fn set_misc_tiles(&mut self, value: u16) {
        self.state.set_misc_tiles(value);
        self.sync();
    }

    pub(crate) fn clear_misc_tiles(&mut self) {
        self.state.clear_misc_tiles();
        self.sync();
    }

    pub(crate) fn or_misc_tiles(&mut self, value: u16) -> u16 {
        let next = self.state.or_misc_tiles(value);
        self.sync();
        next
    }

    pub(crate) fn set_thick_grass(&mut self, value: u16) {
        self.state.set_thick_grass(value);
        self.sync();
    }

    pub(crate) fn clear_thick_grass(&mut self) {
        self.state.clear_thick_grass();
        self.sync();
    }

    pub(crate) fn or_thick_grass(&mut self, value: u16) -> u16 {
        let next = self.state.or_thick_grass(value);
        self.sync();
        next
    }

    pub(crate) fn clear_vertical_ledge(&mut self) {
        self.state.clear_vertical_ledge();
        self.sync();
    }

    pub(crate) fn or_vertical_ledge(&mut self, value: u8) {
        self.state.or_vertical_ledge(value);
        self.sync();
    }

    pub(crate) fn clear_horizontal_ledge(&mut self) {
        self.state.clear_horizontal_ledge();
        self.sync();
    }

    pub(crate) fn or_horizontal_ledge(&mut self, value: u8) {
        self.state.or_horizontal_ledge(value);
        self.sync();
    }

    pub(crate) fn set_moving_floor_tiles(&mut self, value: u16) {
        self.state.set_moving_floor_tiles(value);
        self.sync();
    }

    pub(crate) fn clear_moving_floor_tiles(&mut self) {
        self.state.clear_moving_floor_tiles();
        self.sync();
    }

    pub(crate) fn or_moving_floor_tiles(&mut self, value: u16) -> u16 {
        let next = self.state.or_moving_floor_tiles(value);
        self.sync();
        next
    }

    pub(crate) fn set_icy_floor(&mut self, value: u16) {
        self.state.set_icy_floor(value);
        self.sync();
    }

    pub(crate) fn clear_icy_floor(&mut self) {
        self.state.clear_icy_floor();
        self.sync();
    }

    pub(crate) fn or_icy_floor(&mut self, value: u16) -> u16 {
        let next = self.state.or_icy_floor(value);
        self.sync();
        next
    }

    pub(crate) fn set_water_staircase(&mut self, value: u16) {
        self.state.set_water_staircase(value);
        self.sync();
    }

    pub(crate) fn clear_water_staircase(&mut self) {
        self.state.clear_water_staircase();
        self.sync();
    }

    pub(crate) fn or_water_staircase(&mut self, value: u16) -> u16 {
        let next = self.state.or_water_staircase(value);
        self.sync();
        next
    }

    pub(crate) fn set_shallow_water(&mut self, value: u16) {
        self.state.set_shallow_water(value);
        self.sync();
    }

    pub(crate) fn clear_shallow_water(&mut self) {
        self.state.clear_shallow_water();
        self.sync();
    }

    pub(crate) fn or_shallow_water(&mut self, value: u16) -> u16 {
        let next = self.state.or_shallow_water(value);
        self.sync();
        next
    }

    pub(crate) fn set_destruction_aftermath(&mut self, value: u16) {
        self.state.set_destruction_aftermath(value);
        self.sync();
    }

    pub(crate) fn clear_destruction_aftermath(&mut self) {
        self.state.clear_destruction_aftermath();
        self.sync();
    }

    pub(crate) fn or_destruction_aftermath(&mut self, value: u16) -> u16 {
        let next = self.state.or_destruction_aftermath(value);
        self.sync();
        next
    }

    pub(crate) fn set_read_something(&mut self, value: u16) {
        self.state.set_read_something(value);
        self.sync();
    }

    pub(crate) fn clear_read_something(&mut self) {
        self.state.clear_read_something();
        self.sync();
    }

    pub(crate) fn or_read_something(&mut self, value: u16) -> u16 {
        let next = self.state.or_read_something(value);
        self.sync();
        next
    }

    pub(crate) fn set_ledges_down_leftright(&mut self, value: u8) {
        self.state.set_ledges_down_leftright(value);
        self.sync();
    }

    pub(crate) fn clear_ledges_down_leftright(&mut self) {
        self.state.clear_ledges_down_leftright();
        self.sync();
    }

    pub(crate) fn or_ledges_down_leftright(&mut self, value: u8) {
        self.state.or_ledges_down_leftright(value);
        self.sync();
    }

    pub(crate) fn set_diagonal_ledge_tiles(&mut self, value: u8) {
        self.state.set_diagonal_ledge_tiles(value);
        self.sync();
    }

    pub(crate) fn clear_diagonal_ledge_tiles(&mut self) {
        self.state.clear_diagonal_ledge_tiles();
        self.sync();
    }

    pub(crate) fn or_diagonal_ledge_tiles(&mut self, value: u8) {
        self.state.or_diagonal_ledge_tiles(value);
        self.sync();
    }

    pub(crate) fn set_chest(&mut self, value: u16) {
        self.state.set_chest(value);
        self.sync();
    }

    pub(crate) fn clear_chest(&mut self) {
        self.state.clear_chest();
        self.sync();
    }

    pub(crate) fn or_chest(&mut self, value: u16) -> u16 {
        let next = self.state.or_chest(value);
        self.sync();
        next
    }

    pub(crate) fn set_key_lock_gravestones(&mut self, value: u8) {
        self.state.set_key_lock_gravestones(value);
        self.sync();
    }

    pub(crate) fn clear_key_lock_gravestones(&mut self) {
        self.state.clear_key_lock_gravestones();
        self.sync();
    }

    pub(crate) fn or_key_lock_gravestones(&mut self, value: u8) {
        self.state.or_key_lock_gravestones(value);
        self.sync();
    }

    pub(crate) fn set_spike_cactus_tiles(&mut self, value: u8) {
        self.state.set_spike_cactus_tiles(value);
        self.sync();
    }

    pub(crate) fn clear_spike_cactus_tiles(&mut self) {
        self.state.clear_spike_cactus_tiles();
        self.sync();
    }

    pub(crate) fn or_spike_cactus_tiles(&mut self, value: u8) {
        self.state.or_spike_cactus_tiles(value);
        self.sync();
    }

    pub(crate) fn set_tile_type(&mut self, value: u16) {
        self.state.set_tile_type(value);
        self.sync();
    }

    pub(crate) fn clear_tile_type(&mut self) {
        self.state.clear_tile_type();
        self.sync();
    }

    pub(crate) fn set_spike_floor_and_triggers(&mut self, value: u8) {
        self.state.set_spike_floor_and_triggers(value);
        self.sync();
    }

    pub(crate) fn clear_spike_floor_and_triggers(&mut self) {
        self.state.clear_spike_floor_and_triggers();
        self.sync();
    }

    pub(crate) fn or_spike_floor_and_triggers(&mut self, value: u8) {
        self.state.or_spike_floor_and_triggers(value);
        self.sync();
    }

    pub(crate) fn set_dashable_tiles(&mut self, value: u8) {
        self.state.set_dashable_tiles(value);
        self.sync();
    }

    pub(crate) fn clear_dashable_tiles(&mut self) {
        self.state.clear_dashable_tiles();
        self.sync();
    }

    pub(crate) fn or_dashable_tiles(&mut self, value: u8) {
        self.state.or_dashable_tiles(value);
        self.sync();
    }

    pub(crate) fn set_staircase_cache(&mut self, value: u8) {
        self.state.set_staircase_cache(value);
        self.sync();
    }

    pub(crate) fn set_slope_collision_bits(&mut self, value: u16) {
        self.state.set_slope_collision_bits(value);
        self.sync();
    }

    pub(crate) fn clear_slope_collision_bits(&mut self) {
        self.state.clear_slope_collision_bits();
        self.sync();
    }

    pub(crate) fn or_slope_collision_bits(&mut self, value: u16) -> u16 {
        let next = self.state.or_slope_collision_bits(value);
        self.sync();
        next
    }

    pub(crate) fn set_collision_bits(&mut self, value: u16) {
        self.state.set_collision_bits(value);
        self.sync();
    }

    pub(crate) fn clear_collision_bits(&mut self) {
        self.state.clear_collision_bits();
        self.sync();
    }

    pub(crate) fn or_collision_bits(&mut self, value: u16) -> u16 {
        let next = self.state.or_collision_bits(value);
        self.sync();
        next
    }

    pub(crate) fn set_layer_collision(&mut self, mask: u8, enabled: bool) {
        self.state.set_layer_collision(mask, enabled);
        self.sync();
    }

    pub(crate) fn set_layer_collision_flags(&mut self, value: u8) {
        self.state.set_layer_collision_flags(value);
        self.sync();
    }

    pub(crate) fn set_tile_probe_anchor(&mut self, value: u16) {
        self.state.set_tile_probe_anchor(value);
        self.sync();
    }

    pub(crate) fn clear_inroom_staircase(&mut self) {
        self.state.clear_inroom_staircase();
        self.sync();
    }

    pub(crate) fn or_inroom_staircase(&mut self, bits: u16) -> u16 {
        let next = self.state.or_inroom_staircase(bits);
        self.sync();
        next
    }

    pub(crate) fn set_liftable_tile_index(&mut self, value: u8) {
        self.state.set_liftable_tile_index(value);
        self.sync();
    }

    pub(crate) fn set_tile_collision_bits_primary(&mut self, value: u8) {
        self.state.set_tile_collision_bits_primary(value);
        self.sync();
    }

    pub(crate) fn set_liftable_action_index_primary(&mut self, value: u8) {
        self.state.set_liftable_action_index_primary(value);
        self.sync();
    }

    pub(crate) fn set_liftable_action_index_secondary(&mut self, value: u8) {
        self.state.set_liftable_action_index_secondary(value);
        self.sync();
    }

    pub(crate) fn clear_interaction_scratch_x_low(&mut self) {
        self.state.clear_interaction_scratch_x_low();
        self.sync();
    }

    pub(crate) fn set_interaction_scratch_y_bytes(&mut self, low: u8, high: u8) {
        self.state.set_interaction_scratch_y_bytes(low, high);
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

    pub(crate) fn set_facing_player(&mut self, slot: usize, value: u8) -> bool {
        let Some(offset) = pushed_block_bank_offset(slot) else {
            return false;
        };
        self.facing_player[offset] = value;
        true
    }

    pub(crate) fn set_target_low(&mut self, slot: usize, value: u8) -> bool {
        let Some(offset) = pushed_block_bank_offset(slot) else {
            return false;
        };
        self.target[offset] = value;
        true
    }

    pub(crate) fn set_animation_mode(&mut self, value: u8) {
        self.animation_mode = value;
    }

    pub(crate) fn reset_animation_timer(&mut self) {
        self.animation_timer = 9;
    }

    pub(crate) fn decrement_animation_timer(&mut self) -> u8 {
        self.animation_timer = self.animation_timer.wrapping_sub(1);
        self.animation_timer
    }

    pub(crate) fn advance_animation_mode(&mut self) -> u8 {
        self.animation_timer = 9;
        self.animation_mode = self.animation_mode.wrapping_add(1);
        self.animation_mode
    }

    pub(crate) fn init_slot(&mut self, slot: usize, x: u16, y: u16) {
        write_pushed_block_bank_word(&mut self.x_low, slot, x & 0x00ff);
        write_pushed_block_bank_word(&mut self.x_high, slot, x >> 8);
        write_pushed_block_bank_word(&mut self.y_low, slot, y & 0x00ff);
        write_pushed_block_bank_word(&mut self.y_high, slot, y >> 8);
        write_pushed_block_bank_word(&mut self.target, slot, 0);
        write_pushed_block_bank_word(&mut self.subpixel, slot, 0);
    }

    pub(crate) fn set_push_direction(&mut self, value: u8) {
        self.push_direction = value;
    }

    pub(crate) fn set_x_fixed24(&mut self, slot: usize, value: u32) -> bool {
        let Some(offset) = pushed_block_bank_offset(slot) else {
            return false;
        };
        self.subpixel[offset] = value as u8;
        self.x_low[offset] = (value >> 8) as u8;
        self.x_high[offset] = (value >> 16) as u8;
        true
    }

    pub(crate) fn set_y_fixed24(&mut self, slot: usize, value: u32) -> bool {
        let Some(offset) = pushed_block_bank_offset(slot) else {
            return false;
        };
        self.subpixel[offset] = value as u8;
        self.y_low[offset] = (value >> 8) as u8;
        self.y_high[offset] = (value >> 16) as u8;
        true
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
        if self.state.set_facing_player(slot, value) {
            self.sync();
        }
    }

    pub(crate) fn set_target_low(&mut self, slot: usize, value: u8) {
        if self.state.set_target_low(slot, value) {
            self.sync();
        }
    }

    pub(crate) fn set_animation_mode(&mut self, value: u8) {
        self.state.set_animation_mode(value);
        self.sync();
    }

    pub(crate) fn reset_animation_timer(&mut self) {
        self.state.reset_animation_timer();
        self.sync();
    }

    pub(crate) fn decrement_animation_timer(&mut self) -> u8 {
        let timer = self.state.decrement_animation_timer();
        self.sync();
        timer
    }

    pub(crate) fn advance_animation_mode(&mut self) -> u8 {
        let mode = self.state.advance_animation_mode();
        self.sync();
        mode
    }

    pub(crate) fn init_slot(&mut self, slot: usize, x: u16, y: u16) {
        self.state.init_slot(slot, x, y);
        self.sync();
    }

    pub(crate) fn set_push_direction(&mut self, value: u8) {
        self.state.set_push_direction(value);
        self.sync();
    }

    pub(crate) fn set_x_fixed24(&mut self, slot: usize, value: u32) {
        if self.state.set_x_fixed24(slot, value) {
            self.sync();
        }
    }

    pub(crate) fn set_y_fixed24(&mut self, slot: usize, value: u32) {
        if self.state.set_y_fixed24(slot, value) {
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

    pub(crate) fn set_mode(&mut self, offset: usize, value: u16) -> bool {
        let Some(axis) = swim_axis_index(offset) else {
            return false;
        };
        self.mode[axis] = value;
        true
    }

    pub(crate) fn clear_mode_low_axis(&mut self) {
        self.mode[0] = 0;
    }

    pub(crate) fn set_speed_active_flag(&mut self, offset: usize, value: u16) -> bool {
        let Some(axis) = swim_axis_index(offset) else {
            return false;
        };
        self.speed_active_flag[axis] = value;
        true
    }

    pub(crate) fn set_max_speed(&mut self, offset: usize, value: u16) -> bool {
        let Some(axis) = swim_axis_index(offset) else {
            return false;
        };
        self.max_speed[axis] = value;
        true
    }

    pub(crate) fn set_max_speed_both_axes(&mut self, value: u16) {
        self.max_speed = [value; SWIM_AXIS_COUNT];
    }

    pub(crate) fn set_acceleration_direction(&mut self, offset: usize, value: u16) -> bool {
        let Some(axis) = swim_axis_index(offset) else {
            return false;
        };
        self.acceleration_direction[axis] = value;
        true
    }

    pub(crate) fn set_acceleration(&mut self, offset: usize, value: u16) -> bool {
        let Some(axis) = swim_axis_index(offset) else {
            return false;
        };
        self.acceleration[axis] = value;
        true
    }

    pub(crate) fn clear_axis_motion(&mut self, offset: usize) -> bool {
        let Some(axis) = swim_axis_index(offset) else {
            return false;
        };
        self.speed_active_flag[axis] = 0;
        self.mode[axis] = 0;
        self.acceleration[axis] = 0;
        self.max_speed[axis] = 0;
        true
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
        if self.state.set_mode(offset, value) {
            self.sync();
        }
    }

    pub(crate) fn clear_mode_low_axis(&mut self) {
        self.state.clear_mode_low_axis();
        self.sync();
    }

    pub(crate) fn set_speed_active_flag(&mut self, offset: usize, value: u16) {
        if self.state.set_speed_active_flag(offset, value) {
            self.sync();
        }
    }

    pub(crate) fn set_max_speed(&mut self, offset: usize, value: u16) {
        if self.state.set_max_speed(offset, value) {
            self.sync();
        }
    }

    pub(crate) fn set_max_speed_both_axes(&mut self, value: u16) {
        self.state.set_max_speed_both_axes(value);
        self.sync();
    }

    pub(crate) fn set_acceleration_direction(&mut self, offset: usize, value: u16) {
        if self.state.set_acceleration_direction(offset, value) {
            self.sync();
        }
    }

    pub(crate) fn set_acceleration(&mut self, offset: usize, value: u16) {
        if self.state.set_acceleration(offset, value) {
            self.sync();
        }
    }

    pub(crate) fn clear_axis_motion(&mut self, offset: usize) {
        if self.state.clear_axis_motion(offset) {
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
        self.state.set_x(value);
        self.sync();
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.state.set_y(value);
        self.sync();
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_position(x, y);
        self.sync();
    }

    pub(crate) fn offset_position(&mut self, x_delta: u16, y_delta: u16) {
        self.state.offset_position(x_delta, y_delta);
        self.sync();
    }

    pub(crate) fn store_from_player(&mut self) {
        self.state.store_from_player_ram(self.ram);
        self.sync();
    }

    pub(crate) fn restore_player_position(&mut self) {
        self.state.restore_player_position_to_ram(self.ram);
    }
}
