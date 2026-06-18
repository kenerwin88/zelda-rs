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

fn move_link_axis_by_velocity(
    ram: &mut [u8],
    subpixel_offset: usize,
    coord_offset: usize,
    velocity: u8,
) -> u16 {
    let pos = u32::from(ram[subpixel_offset]) | (u32::from(read_le_u16(ram, coord_offset)) << 8);
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
    let pos = u32::from(ram[subpixel_offset]) | (u32::from(read_le_u16(ram, coord_offset)) << 8);
    let moved = pos.wrapping_add(delta as i16 as i32 as u32);
    ram[subpixel_offset] = moved as u8;
    write_le_u16(ram, coord_offset, (moved >> 8) as u16);
    (moved >> 8) as u16
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

    pub(crate) fn sync_follower_link_from_ram(&mut self, ram: &[u8]) {
        self.follower_link = FollowerLinkState::load_from_ram(ram);
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
    z_mirror: u16,
    oam_x_offset: u8,
    oam_y_offset: u8,
    x_subpixel: u8,
    y_subpixel: u8,
    x_velocity: u8,
    y_velocity: u8,
    actual_x_velocity: u8,
    actual_y_velocity: u8,
    z_velocity: u8,
    z_velocity_copy: u8,
    z_velocity_mirror: u8,
    z_velocity_copy_mirror: u8,
    recoil_z_velocity_for_dungeon_reset: u8,
    recoil_timer: u8,
    floor: u8,
    lower_level_mirror_state: u8,
    cached_lower_level_state: u8,
    cached_lower_level_mirror_state: u8,
    direction: u8,
    direction_lock: u8,
    direction_mask_a: u8,
    direction_mask_b: u8,
    last_direction: u8,
    last_direction_moved_towards: u8,
    moving_against_diag_tile: u8,
    movement_flag: u8,
    quadrant_x: u8,
    quadrant_y: u8,
    cached_quadrant_x: u8,
    cached_quadrant_y: u8,
    num_orthogonal_directions: u8,
    swim_direction_flags: u8,
    facing: u8,
    facing_mirror: u8,
    cached_facing: u8,
    speed_setting: u8,
    speed_modifier: u8,
    dash_counter: u8,
    dash_countdown: u8,
    jump_ledge_timer: u8,
    about_to_jump_off_ledge: u8,
    push_fatigue_timer: u8,
    gravestone_push_timeout: u8,
    menu_block_flag: u8,
    handler_state: u8,
    immobilized: u8,
    action_state_bits: u8,
    auxiliary_state: u8,
    running: u8,
    picking_throw_state: u8,
    button_mask_b_y: u8,
    filtered_joypad_h: u8,
    filtered_joypad_l: u8,
    joypad1h_last: u8,
    joypad1l_last: u8,
    joypad1h_last2: u8,
    joypad1l_last2: u8,
    spin_attack_delay_timer: u8,
    spin_attack_step_counter: u8,
    spin_attack_state: u8,
    spin_attack_sound_latch: u8,
    incapacitated_timer: u8,
    visibility_status: u8,
    y_button_action_flags: u8,
    y_button_action_step: u8,
    y_button_action_timer: u8,
    defense_flags: u8,
    item_receipt_method: u8,
    action_handler_timer: u8,
    doorway_state: u8,
    blink_countdown: u8,
    bunny_transform_timer: u8,
    bunny_state: u8,
    bunny_mirror: u8,
    temp_bunny_timer: u16,
    transform_poof_needed: u8,
    spin_animation_step_counter: u8,
    button_b_frames: u8,
    animation_step: u8,
    opening_pose: u8,
    water_ripple_or_grass_state: u8,
    primary_water_grass_timer: u8,
    secondary_water_grass_timer: u8,
    deep_water_state: u8,
    swim_fast_state: u8,
    hard_swim_stroke: u8,
    swim_stroke_frame_counters: [u16; SWIM_AXIS_COUNT],
    swim_stroke_anim_step: u8,
    swimming_countdown: u8,
    conveyor_belt_state: u8,
    tile_below: u8,
    tile_action_index: u8,
    tile_collision_flag: u8,
    frame_change_counter: u8,
    sprite_oam_state_timer: u8,
    whirlpool_trigger: u8,
    prevent_movement: u8,
    magic_spell_player_lock: u8,
    item_holding_timer: u8,
    cached_tile_action_index: u8,
    ancilla_interactive_reset_flag: u8,
    force_move_any_direction: u16,
    item_action_step: u8,
    throw_oam_state_index: u8,
    item_action_debug_value_2: u8,
    item_debug_value_1: u8,
    given_damage: u8,
    pull_action_state: u8,
    current_item_y: u8,
    current_item_active: u8,
    receive_item_index: u8,
    item_in_hand: u8,
    item_pickup_in_progress: u8,
    position_mode: u8,
    selected_rod: u8,
    flippers: u8,
    moon_pearl: u8,
    magic_power: u8,
    magic_consumption: u8,
    ancilla_pickup_flag: u8,
    sprite_pickup_flag: u8,
    sprite_pickup_flag_cached: u8,
    pull_for_rupees_sprite_needed: u8,
    near_moveable_statue_flag: u8,
    hookshot_interlock: u8,
    grabbing_wall: u8,
    hookshot_grave_latch: u8,
    electrocute_on_touch: u8,
    cape_mode: u8,
    cape_decrement_counter: u8,
    item_hold_pose: u8,
    force_hold_sword_up: u8,
    sword_delay_timer: u8,
    dash_noise_requested: u8,
    faint_animation_active: u8,
    transforming: u8,
    flute_countdown: u8,
    hookshot_bg_check_off_timer: u8,
    index_of_dashing_sfx: u8,
    spin_offsets: u8,
    somaria_platform_state: u8,
    near_pit_state: u8,
    pit_data_index: u8,
    pit_correction_timer: u8,
    pit_correction_active: u8,
    moving_against_diag_deadlocked: u8,
    incapacitated_camera_timer: u8,
    sprite_damage_disabled: u8,
    link_dma_graphics_index: u16,
    link_dma_left_sprite_bank: u16,
    link_dma_right_sprite_bank: u16,
    sword_dma_graphics_index: u8,
    shield_dma_graphics_index: u8,
    link_dma_staging_index: u8,
    link_dma_source_offset: u16,
    link_dma_tile_offset: u16,
    link_dma_countdown: u16,
    palette_bits_of_oam: u16,
    link_sprite_index_scratch: u16,
    hop_origin_coord: u16,
    cached_x: u16,
    cached_y: u16,
    copied_x: u16,
    copied_y: u16,
    previous_x: u16,
    previous_y: u16,
    safe_return_x: u16,
    safe_return_y: u16,
    bit9_of_xcoord: u16,
    somaria_block_bg_check_flag: u8,
    player_pose_draw_counter: u8,
    player_special_draw_flag: u8,
    sleep_in_bed_state: u8,
    cheat_walk_through_walls: u8,
    x_page_movement_delta: u8,
    y_page_movement_delta: u8,
    moving_floor_x: u16,
    moving_floor_y: u16,
    drag_player_x: u16,
    drag_player_y: u16,
}

impl FollowerLinkState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            x: read_le_u16(ram, LINK_X_COORD),
            y: read_le_u16(ram, LINK_Y_COORD),
            z: read_le_u16(ram, LINK_Z_COORD),
            z_mirror: read_le_u16(ram, LINK_Z_COORD_MIRROR),
            oam_x_offset: ram_byte(ram, PLAYER_OAM_X_OFFSET),
            oam_y_offset: ram_byte(ram, PLAYER_OAM_Y_OFFSET),
            x_subpixel: ram_byte(ram, LINK_X_SUBPIXEL),
            y_subpixel: ram_byte(ram, LINK_Y_SUBPIXEL),
            x_velocity: ram_byte(ram, LINK_X_VELOCITY),
            y_velocity: ram_byte(ram, LINK_Y_VELOCITY),
            actual_x_velocity: ram_byte(ram, LINK_ACTUAL_X_VELOCITY),
            actual_y_velocity: ram_byte(ram, LINK_ACTUAL_Y_VELOCITY),
            z_velocity: ram_byte(ram, LINK_Z_VELOCITY),
            z_velocity_copy: ram_byte(ram, LINK_Z_VELOCITY_COPY),
            z_velocity_mirror: ram_byte(ram, LINK_Z_VELOCITY_MIRROR),
            z_velocity_copy_mirror: ram_byte(ram, LINK_Z_VELOCITY_COPY_MIRROR),
            recoil_z_velocity_for_dungeon_reset: ram_byte(ram, LINK_RECOIL_Z_VELOCITY_DUNGEON),
            recoil_timer: ram_byte(ram, LINK_RECOIL_TIMER),
            floor: ram_byte(ram, LINK_IS_ON_LOWER_LEVEL),
            lower_level_mirror_state: ram_byte(ram, LINK_IS_ON_LOWER_LEVEL_MIRROR),
            cached_lower_level_state: ram_byte(ram, LINK_IS_ON_LOWER_LEVEL_CACHED),
            cached_lower_level_mirror_state: ram_byte(ram, LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED),
            direction: ram_byte(ram, LINK_DIRECTION),
            direction_lock: ram_byte(ram, LINK_CANT_CHANGE_DIRECTION),
            direction_mask_a: ram_byte(ram, LINK_DIRECTION_MASK_A),
            direction_mask_b: ram_byte(ram, LINK_DIRECTION_MASK_B),
            last_direction: ram_byte(ram, LINK_LAST_DIRECTION),
            last_direction_moved_towards: ram_byte(ram, LINK_LAST_DIRECTION_MOVED_TOWARDS),
            moving_against_diag_tile: ram_byte(ram, LINK_MOVING_AGAINST_DIAG_TILE),
            movement_flag: ram_byte(ram, LINK_FLAG_MOVING),
            quadrant_x: ram_byte(ram, LINK_QUADRANT_X),
            quadrant_y: ram_byte(ram, LINK_QUADRANT_Y),
            cached_quadrant_x: ram_byte(ram, LINK_QUADRANT_X_CACHED),
            cached_quadrant_y: ram_byte(ram, LINK_QUADRANT_Y_CACHED),
            num_orthogonal_directions: ram_byte(ram, LINK_NUM_ORTHOGONAL_DIRECTIONS),
            swim_direction_flags: ram_byte(ram, SWIM_PLAYER_DIRECTION_FLAGS),
            facing: ram_byte(ram, LINK_FACING),
            facing_mirror: ram_byte(ram, LINK_FACING_MIRROR),
            cached_facing: ram_byte(ram, LINK_FACING_CACHED),
            speed_setting: ram_byte(ram, LINK_SPEED_SETTING),
            speed_modifier: ram_byte(ram, LINK_SPEED_MODIFIER),
            dash_counter: ram_byte(ram, LINK_DASH_COUNTER),
            dash_countdown: ram_byte(ram, LINK_COUNTDOWN_FOR_DASH),
            jump_ledge_timer: ram_byte(ram, LINK_TIMER_JUMP_LEDGE),
            about_to_jump_off_ledge: ram_byte(ram, ABOUT_TO_JUMP_OFF_LEDGE),
            push_fatigue_timer: ram_byte(ram, LINK_TIMER_PUSH_GET_TIRED),
            gravestone_push_timeout: ram_byte(ram, GRAVESTONE_PUSH_TIMEOUT),
            menu_block_flag: ram_byte(ram, FLAG_BLOCK_LINK_MENU),
            handler_state: ram_byte(ram, LINK_HANDLER_STATE),
            immobilized: ram_byte(ram, FLAG_IS_LINK_IMMOBILIZED),
            action_state_bits: ram_byte(ram, LINK_STATE_BITS),
            auxiliary_state: ram_byte(ram, LINK_AUXILIARY_STATE),
            running: ram_byte(ram, LINK_IS_RUNNING),
            picking_throw_state: ram_byte(ram, LINK_PICKING_THROW_STATE),
            button_mask_b_y: ram_byte(ram, BUTTON_MASK_B_Y),
            filtered_joypad_h: ram_byte(ram, FILTERED_JOYPAD_H),
            filtered_joypad_l: ram_byte(ram, FILTERED_JOYPAD_L),
            joypad1h_last: ram_byte(ram, JOYPAD1H_LAST),
            joypad1l_last: ram_byte(ram, JOYPAD1L_LAST),
            joypad1h_last2: ram_byte(ram, JOYPAD1H_LAST2),
            joypad1l_last2: ram_byte(ram, JOYPAD1L_LAST2),
            spin_attack_delay_timer: ram_byte(ram, LINK_DELAY_TIMER_SPIN_ATTACK),
            spin_attack_step_counter: ram_byte(ram, LINK_SPIN_ATTACK_STEP_COUNTER),
            spin_attack_state: ram_byte(ram, STATE_FOR_SPIN_ATTACK),
            spin_attack_sound_latch: ram_byte(ram, SPIN_ATTACK_SOUND_LATCH),
            incapacitated_timer: ram_byte(ram, LINK_INCAPACITATED_TIMER),
            visibility_status: ram_byte(ram, LINK_VISIBILITY_STATUS),
            y_button_action_flags: ram_byte(ram, Y_BUTTON_ACTION_FLAGS),
            y_button_action_step: ram_byte(ram, Y_BUTTON_ACTION_STEP),
            y_button_action_timer: ram_byte(ram, Y_BUTTON_ACTION_TIMER),
            defense_flags: ram_byte(ram, PLAYER_DEFENSE_FLAGS),
            item_receipt_method: ram_byte(ram, ITEM_RECEIPT_METHOD),
            action_handler_timer: ram_byte(ram, PLAYER_HANDLER_TIMER),
            doorway_state: ram_byte(ram, IS_STANDING_IN_DOORWAY),
            blink_countdown: ram_byte(ram, COUNTDOWN_FOR_BLINK),
            bunny_transform_timer: ram_byte(ram, LINK_BUNNY_TRANSFORM_TIMER),
            bunny_state: ram_byte(ram, LINK_IS_BUNNY),
            bunny_mirror: ram_byte(ram, LINK_IS_BUNNY_MIRROR),
            temp_bunny_timer: read_le_u16(ram, LINK_TIMER_TEMPBUNNY),
            transform_poof_needed: ram_byte(ram, LINK_NEED_FOR_POOF_FOR_TRANSFORM),
            spin_animation_step_counter: ram_byte(ram, STEP_COUNTER_FOR_SPIN_ATTACK),
            button_b_frames: ram_byte(ram, BUTTON_B_FRAMES),
            animation_step: ram_byte(ram, LINK_ANIMATION_STEPS),
            opening_pose: ram_byte(ram, LINK_POSE_DURING_OPENING),
            water_ripple_or_grass_state: ram_byte(ram, DRAW_WATER_RIPPLES_OR_GRASS),
            primary_water_grass_timer: ram_byte(ram, PRIMARY_WATER_GRASS_TIMER),
            secondary_water_grass_timer: ram_byte(ram, SECONDARY_WATER_GRASS_TIMER),
            deep_water_state: ram_byte(ram, LINK_IS_IN_DEEP_WATER),
            swim_fast_state: ram_byte(ram, LINK_MAYBE_SWIM_FASTER),
            hard_swim_stroke: ram_byte(ram, LINK_SWIM_HARD_STROKE),
            swim_stroke_frame_counters: [
                read_le_u16(ram, SWIM_STROKE_FRAME_COUNTER),
                read_le_u16(ram, SWIM_STROKE_FRAME_COUNTER + 2),
            ],
            swim_stroke_anim_step: ram_byte(ram, SWIM_STROKE_ANIM_STEP),
            swimming_countdown: ram_byte(ram, SWIMMING_COUNTDOWN),
            conveyor_belt_state: ram_byte(ram, LINK_ON_CONVEYOR_BELT),
            tile_below: ram_byte(ram, LINK_TILE_BELOW),
            tile_action_index: ram_byte(ram, TILE_ACTION_INDEX),
            tile_collision_flag: ram_byte(ram, TILE_COLL_FLAG),
            frame_change_counter: ram_byte(ram, LINK_FRAME_CHANGE_COUNTER),
            sprite_oam_state_timer: ram_byte(ram, LINK_SPRITE_OAM_STATE_TIMER),
            whirlpool_trigger: ram_byte(ram, LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE),
            prevent_movement: ram_byte(ram, LINK_PREVENT_FROM_MOVING),
            magic_spell_player_lock: ram_byte(ram, MAGIC_SPELL_PLAYER_LOCK_FLAG),
            item_holding_timer: ram_byte(ram, LINK_ITEM_HOLDING_TIMER),
            cached_tile_action_index: ram_byte(ram, CACHED_TILE_ACTION_INDEX),
            ancilla_interactive_reset_flag: ram_byte(ram, ANCILLA_INTERACTIVE_RESET_FLAG),
            force_move_any_direction: read_le_u16(ram, FORCE_MOVE_ANY_DIRECTION),
            item_action_step: ram_byte(ram, LINK_ITEM_ACTION_STEP),
            throw_oam_state_index: ram_byte(ram, LINK_THROW_OAM_STATE_INDEX),
            item_action_debug_value_2: ram_byte(ram, LINK_DEBUG_VALUE_2),
            item_debug_value_1: ram_byte(ram, LINK_DEBUG_VALUE_1),
            given_damage: ram_byte(ram, LINK_GIVE_DAMAGE),
            pull_action_state: ram_byte(ram, LINK_PULL_ACTION_STATE),
            current_item_y: ram_byte(ram, LINK_CURRENT_ITEM_Y),
            current_item_active: ram_byte(ram, LINK_CURRENT_ITEM_ACTIVE),
            receive_item_index: ram_byte(ram, LINK_RECEIVE_ITEM_INDEX),
            item_in_hand: ram_byte(ram, LINK_ITEM_IN_HAND),
            item_pickup_in_progress: ram_byte(ram, ITEM_PICKUP_IN_PROGRESS_FLAG),
            position_mode: ram_byte(ram, LINK_POSITION_MODE),
            selected_rod: ram_byte(ram, EQ_SELECTED_ROD),
            flippers: ram_byte(ram, LINK_ITEM_FLIPPERS),
            moon_pearl: ram_byte(ram, LINK_ITEM_MOON_PEARL),
            magic_power: ram_byte(ram, LINK_MAGIC_POWER),
            magic_consumption: ram_byte(ram, LINK_MAGIC_CONSUMPTION),
            ancilla_pickup_flag: ram_byte(ram, FLAG_IS_ANCILLA_TO_PICK_UP),
            sprite_pickup_flag: ram_byte(ram, FLAG_IS_SPRITE_TO_PICK_UP),
            sprite_pickup_flag_cached: ram_byte(ram, FLAG_IS_SPRITE_TO_PICK_UP_CACHED),
            pull_for_rupees_sprite_needed: ram_byte(ram, LINK_NEED_FOR_PULLFORRUPEES_SPRITE),
            near_moveable_statue_flag: ram_byte(ram, LINK_IS_NEAR_MOVEABLE_STATUE),
            hookshot_interlock: ram_byte(ram, RELATED_TO_HOOKSHOT),
            grabbing_wall: ram_byte(ram, LINK_GRABBING_WALL),
            hookshot_grave_latch: ram_byte(ram, LINK_SOMETHING_WITH_HOOKSHOT),
            electrocute_on_touch: ram_byte(ram, LINK_ELECTROCUTE_ON_TOUCH),
            cape_mode: ram_byte(ram, LINK_CAPE_MODE),
            cape_decrement_counter: ram_byte(ram, CAPE_DECREMENT_COUNTER),
            item_hold_pose: ram_byte(ram, LINK_POSE_FOR_ITEM),
            force_hold_sword_up: ram_byte(ram, LINK_FORCE_HOLD_SWORD_UP),
            sword_delay_timer: ram_byte(ram, LINK_SWORD_DELAY_TIMER),
            dash_noise_requested: ram_byte(ram, LINK_WANT_MAKE_NOISE_WHEN_DASHED),
            faint_animation_active: ram_byte(ram, LINK_FAINT_ANIMATION_ACTIVE),
            transforming: ram_byte(ram, LINK_IS_TRANSFORMING),
            flute_countdown: ram_byte(ram, FLUTE_COUNTDOWN),
            hookshot_bg_check_off_timer: ram_byte(ram, HOOKSHOT_BG_CHECK_OFF_TIMER),
            index_of_dashing_sfx: ram_byte(ram, INDEX_OF_DASHING_SFX),
            spin_offsets: ram_byte(ram, LINK_SPIN_OFFSETS),
            somaria_platform_state: ram_byte(ram, PLAYER_ON_SOMARIA_PLATFORM),
            near_pit_state: ram_byte(ram, PLAYER_NEAR_PIT_STATE),
            pit_data_index: ram_byte(ram, PLAYER_PIT_DATA_INDEX),
            pit_correction_timer: ram_byte(ram, PIT_CORRECTION_TIMER),
            pit_correction_active: ram_byte(ram, PIT_CORRECTION_ACTIVE_FLAG),
            moving_against_diag_deadlocked: ram_byte(ram, MOVING_AGAINST_DIAG_DEADLOCKED),
            incapacitated_camera_timer: ram_byte(ram, LINK_INCAPACITATED_CAMERA_TIMER),
            sprite_damage_disabled: ram_byte(ram, LINK_DISABLE_SPRITE_DAMAGE),
            link_dma_graphics_index: read_le_u16(ram, LINK_DMA_GRAPHICS_INDEX),
            link_dma_left_sprite_bank: read_le_u16(ram, LINK_DMA_LEFT_SPRITE_BANK_INDEX),
            link_dma_right_sprite_bank: read_le_u16(ram, LINK_DMA_RIGHT_SPRITE_BANK_INDEX),
            sword_dma_graphics_index: ram_byte(ram, LINK_DMA_SWORD_GRAPHICS_INDEX),
            shield_dma_graphics_index: ram_byte(ram, LINK_DMA_SHIELD_GRAPHICS_INDEX),
            link_dma_staging_index: ram_byte(ram, LINK_DMA_STAGING_INDEX),
            link_dma_source_offset: read_le_u16(ram, LINK_DMA_SOURCE_OFFSET),
            link_dma_tile_offset: read_le_u16(ram, LINK_DMA_TILE_OFFSET),
            link_dma_countdown: read_le_u16(ram, LINK_DMA_COUNTDOWN),
            palette_bits_of_oam: read_le_u16(ram, LINK_PALETTE_BITS_OF_OAM),
            link_sprite_index_scratch: read_le_u16(ram, SCRATCH_1),
            hop_origin_coord: read_le_u16(ram, LINK_Y_COORD_ORIGINAL),
            cached_x: read_le_u16(ram, LINK_X_COORD_CACHED),
            cached_y: read_le_u16(ram, LINK_Y_COORD_CACHED),
            copied_x: read_le_u16(ram, LINK_X_COORD_COPY),
            copied_y: read_le_u16(ram, LINK_Y_COORD_COPY),
            previous_x: read_le_u16(ram, LINK_X_COORD_PREV),
            previous_y: read_le_u16(ram, LINK_Y_COORD_PREV),
            safe_return_x: u16::from(ram_byte(ram, LINK_X_COORD_SAFE_RETURN_LO))
                | (u16::from(ram_byte(ram, LINK_X_COORD_SAFE_RETURN_HI)) << 8),
            safe_return_y: u16::from(ram_byte(ram, LINK_Y_COORD_SAFE_RETURN_LO))
                | (u16::from(ram_byte(ram, LINK_Y_COORD_SAFE_RETURN_HI)) << 8),
            bit9_of_xcoord: read_le_u16(ram, BIT9_OF_XCOORD),
            somaria_block_bg_check_flag: ram_byte(ram, SOMARIA_BLOCK_BG_CHECK_FLAG),
            player_pose_draw_counter: ram_byte(ram, PLAYER_POSE_DRAW_COUNTER),
            player_special_draw_flag: ram_byte(ram, PLAYER_SPECIAL_DRAW_FLAG),
            sleep_in_bed_state: ram_byte(ram, PLAYER_SLEEP_IN_BED_STATE),
            cheat_walk_through_walls: ram_byte(ram, CHEAT_WALK_THROUGH_WALLS),
            x_page_movement_delta: ram_byte(ram, LINK_X_PAGE_MOVEMENT_DELTA),
            y_page_movement_delta: ram_byte(ram, LINK_Y_PAGE_MOVEMENT_DELTA),
            moving_floor_x: read_le_u16(ram, RELATED_TO_MOVING_FLOOR_X),
            moving_floor_y: read_le_u16(ram, RELATED_TO_MOVING_FLOOR_Y),
            drag_player_x: read_le_u16(ram, DRAG_PLAYER_X),
            drag_player_y: read_le_u16(ram, DRAG_PLAYER_Y),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, LINK_X_COORD, self.x);
        write_le_u16(ram, LINK_Y_COORD, self.y);
        write_le_u16(ram, LINK_Z_COORD, self.z);
        write_le_u16(ram, LINK_Z_COORD_MIRROR, self.z_mirror);
        ram[PLAYER_OAM_X_OFFSET] = self.oam_x_offset;
        ram[PLAYER_OAM_Y_OFFSET] = self.oam_y_offset;
        ram[LINK_X_SUBPIXEL] = self.x_subpixel;
        ram[LINK_Y_SUBPIXEL] = self.y_subpixel;
        ram[LINK_X_VELOCITY] = self.x_velocity;
        ram[LINK_Y_VELOCITY] = self.y_velocity;
        ram[LINK_ACTUAL_X_VELOCITY] = self.actual_x_velocity;
        ram[LINK_ACTUAL_Y_VELOCITY] = self.actual_y_velocity;
        ram[LINK_Z_VELOCITY] = self.z_velocity;
        ram[LINK_Z_VELOCITY_COPY] = self.z_velocity_copy;
        ram[LINK_Z_VELOCITY_MIRROR] = self.z_velocity_mirror;
        ram[LINK_Z_VELOCITY_COPY_MIRROR] = self.z_velocity_copy_mirror;
        ram[LINK_RECOIL_Z_VELOCITY_DUNGEON] = self.recoil_z_velocity_for_dungeon_reset;
        ram[LINK_RECOIL_TIMER] = self.recoil_timer;
        ram[LINK_IS_ON_LOWER_LEVEL] = self.floor;
        ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = self.lower_level_mirror_state;
        ram[LINK_IS_ON_LOWER_LEVEL_CACHED] = self.cached_lower_level_state;
        ram[LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED] = self.cached_lower_level_mirror_state;
        ram[LINK_DIRECTION] = self.direction;
        ram[LINK_CANT_CHANGE_DIRECTION] = self.direction_lock;
        ram[LINK_DIRECTION_MASK_A] = self.direction_mask_a;
        ram[LINK_DIRECTION_MASK_B] = self.direction_mask_b;
        ram[LINK_LAST_DIRECTION] = self.last_direction;
        ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = self.last_direction_moved_towards;
        ram[LINK_MOVING_AGAINST_DIAG_TILE] = self.moving_against_diag_tile;
        ram[LINK_FLAG_MOVING] = self.movement_flag;
        ram[LINK_QUADRANT_X] = self.quadrant_x;
        ram[LINK_QUADRANT_Y] = self.quadrant_y;
        ram[LINK_QUADRANT_X_CACHED] = self.cached_quadrant_x;
        ram[LINK_QUADRANT_Y_CACHED] = self.cached_quadrant_y;
        ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = self.num_orthogonal_directions;
        ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.swim_direction_flags;
        ram[LINK_FACING] = self.facing;
        ram[LINK_FACING_MIRROR] = self.facing_mirror;
        ram[LINK_FACING_CACHED] = self.cached_facing;
        ram[LINK_SPEED_SETTING] = self.speed_setting;
        ram[LINK_SPEED_MODIFIER] = self.speed_modifier;
        ram[LINK_DASH_COUNTER] = self.dash_counter;
        ram[LINK_COUNTDOWN_FOR_DASH] = self.dash_countdown;
        ram[LINK_TIMER_JUMP_LEDGE] = self.jump_ledge_timer;
        ram[ABOUT_TO_JUMP_OFF_LEDGE] = self.about_to_jump_off_ledge;
        ram[LINK_TIMER_PUSH_GET_TIRED] = self.push_fatigue_timer;
        ram[GRAVESTONE_PUSH_TIMEOUT] = self.gravestone_push_timeout;
        ram[FLAG_BLOCK_LINK_MENU] = self.menu_block_flag;
        ram[LINK_HANDLER_STATE] = self.handler_state;
        ram[FLAG_IS_LINK_IMMOBILIZED] = self.immobilized;
        ram[LINK_STATE_BITS] = self.action_state_bits;
        ram[LINK_AUXILIARY_STATE] = self.auxiliary_state;
        ram[LINK_IS_RUNNING] = self.running;
        ram[LINK_PICKING_THROW_STATE] = self.picking_throw_state;
        ram[BUTTON_MASK_B_Y] = self.button_mask_b_y;
        ram[FILTERED_JOYPAD_H] = self.filtered_joypad_h;
        ram[FILTERED_JOYPAD_L] = self.filtered_joypad_l;
        ram[JOYPAD1H_LAST] = self.joypad1h_last;
        ram[JOYPAD1L_LAST] = self.joypad1l_last;
        ram[JOYPAD1H_LAST2] = self.joypad1h_last2;
        ram[JOYPAD1L_LAST2] = self.joypad1l_last2;
        ram[LINK_DELAY_TIMER_SPIN_ATTACK] = self.spin_attack_delay_timer;
        ram[LINK_SPIN_ATTACK_STEP_COUNTER] = self.spin_attack_step_counter;
        ram[STATE_FOR_SPIN_ATTACK] = self.spin_attack_state;
        ram[SPIN_ATTACK_SOUND_LATCH] = self.spin_attack_sound_latch;
        ram[LINK_INCAPACITATED_TIMER] = self.incapacitated_timer;
        ram[LINK_VISIBILITY_STATUS] = self.visibility_status;
        ram[Y_BUTTON_ACTION_FLAGS] = self.y_button_action_flags;
        ram[Y_BUTTON_ACTION_STEP] = self.y_button_action_step;
        ram[Y_BUTTON_ACTION_TIMER] = self.y_button_action_timer;
        ram[PLAYER_DEFENSE_FLAGS] = self.defense_flags;
        ram[ITEM_RECEIPT_METHOD] = self.item_receipt_method;
        ram[PLAYER_HANDLER_TIMER] = self.action_handler_timer;
        ram[IS_STANDING_IN_DOORWAY] = self.doorway_state;
        ram[COUNTDOWN_FOR_BLINK] = self.blink_countdown;
        ram[LINK_BUNNY_TRANSFORM_TIMER] = self.bunny_transform_timer;
        ram[LINK_IS_BUNNY] = self.bunny_state;
        ram[LINK_IS_BUNNY_MIRROR] = self.bunny_mirror;
        write_le_u16(ram, LINK_TIMER_TEMPBUNNY, self.temp_bunny_timer);
        ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = self.transform_poof_needed;
        ram[STEP_COUNTER_FOR_SPIN_ATTACK] = self.spin_animation_step_counter;
        ram[BUTTON_B_FRAMES] = self.button_b_frames;
        ram[LINK_ANIMATION_STEPS] = self.animation_step;
        ram[LINK_POSE_DURING_OPENING] = self.opening_pose;
        ram[DRAW_WATER_RIPPLES_OR_GRASS] = self.water_ripple_or_grass_state;
        ram[PRIMARY_WATER_GRASS_TIMER] = self.primary_water_grass_timer;
        ram[SECONDARY_WATER_GRASS_TIMER] = self.secondary_water_grass_timer;
        ram[LINK_IS_IN_DEEP_WATER] = self.deep_water_state;
        ram[LINK_MAYBE_SWIM_FASTER] = self.swim_fast_state;
        ram[LINK_SWIM_HARD_STROKE] = self.hard_swim_stroke;
        write_le_u16(
            ram,
            SWIM_STROKE_FRAME_COUNTER,
            self.swim_stroke_frame_counters[0],
        );
        write_le_u16(
            ram,
            SWIM_STROKE_FRAME_COUNTER + 2,
            self.swim_stroke_frame_counters[1],
        );
        ram[SWIM_STROKE_ANIM_STEP] = self.swim_stroke_anim_step;
        ram[SWIMMING_COUNTDOWN] = self.swimming_countdown;
        ram[LINK_ON_CONVEYOR_BELT] = self.conveyor_belt_state;
        ram[LINK_TILE_BELOW] = self.tile_below;
        ram[TILE_ACTION_INDEX] = self.tile_action_index;
        ram[TILE_COLL_FLAG] = self.tile_collision_flag;
        ram[LINK_FRAME_CHANGE_COUNTER] = self.frame_change_counter;
        ram[LINK_SPRITE_OAM_STATE_TIMER] = self.sprite_oam_state_timer;
        ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = self.whirlpool_trigger;
        ram[LINK_PREVENT_FROM_MOVING] = self.prevent_movement;
        ram[MAGIC_SPELL_PLAYER_LOCK_FLAG] = self.magic_spell_player_lock;
        ram[LINK_ITEM_HOLDING_TIMER] = self.item_holding_timer;
        ram[CACHED_TILE_ACTION_INDEX] = self.cached_tile_action_index;
        ram[ANCILLA_INTERACTIVE_RESET_FLAG] = self.ancilla_interactive_reset_flag;
        write_le_u16(ram, FORCE_MOVE_ANY_DIRECTION, self.force_move_any_direction);
        ram[LINK_ITEM_ACTION_STEP] = self.item_action_step;
        ram[LINK_THROW_OAM_STATE_INDEX] = self.throw_oam_state_index;
        ram[LINK_DEBUG_VALUE_2] = self.item_action_debug_value_2;
        ram[LINK_DEBUG_VALUE_1] = self.item_debug_value_1;
        ram[LINK_GIVE_DAMAGE] = self.given_damage;
        ram[LINK_PULL_ACTION_STATE] = self.pull_action_state;
        ram[LINK_CURRENT_ITEM_Y] = self.current_item_y;
        ram[LINK_CURRENT_ITEM_ACTIVE] = self.current_item_active;
        ram[LINK_RECEIVE_ITEM_INDEX] = self.receive_item_index;
        ram[LINK_ITEM_IN_HAND] = self.item_in_hand;
        ram[ITEM_PICKUP_IN_PROGRESS_FLAG] = self.item_pickup_in_progress;
        ram[LINK_POSITION_MODE] = self.position_mode;
        ram[EQ_SELECTED_ROD] = self.selected_rod;
        ram[LINK_ITEM_FLIPPERS] = self.flippers;
        ram[LINK_ITEM_MOON_PEARL] = self.moon_pearl;
        ram[LINK_MAGIC_POWER] = self.magic_power;
        ram[LINK_MAGIC_CONSUMPTION] = self.magic_consumption;
        ram[FLAG_IS_ANCILLA_TO_PICK_UP] = self.ancilla_pickup_flag;
        ram[FLAG_IS_SPRITE_TO_PICK_UP] = self.sprite_pickup_flag;
        ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED] = self.sprite_pickup_flag_cached;
        ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = self.pull_for_rupees_sprite_needed;
        ram[LINK_IS_NEAR_MOVEABLE_STATUE] = self.near_moveable_statue_flag;
        ram[RELATED_TO_HOOKSHOT] = self.hookshot_interlock;
        ram[LINK_GRABBING_WALL] = self.grabbing_wall;
        ram[LINK_SOMETHING_WITH_HOOKSHOT] = self.hookshot_grave_latch;
        ram[LINK_ELECTROCUTE_ON_TOUCH] = self.electrocute_on_touch;
        ram[LINK_CAPE_MODE] = self.cape_mode;
        ram[CAPE_DECREMENT_COUNTER] = self.cape_decrement_counter;
        ram[LINK_POSE_FOR_ITEM] = self.item_hold_pose;
        ram[LINK_FORCE_HOLD_SWORD_UP] = self.force_hold_sword_up;
        ram[LINK_SWORD_DELAY_TIMER] = self.sword_delay_timer;
        ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = self.dash_noise_requested;
        ram[LINK_FAINT_ANIMATION_ACTIVE] = self.faint_animation_active;
        ram[LINK_IS_TRANSFORMING] = self.transforming;
        ram[FLUTE_COUNTDOWN] = self.flute_countdown;
        ram[HOOKSHOT_BG_CHECK_OFF_TIMER] = self.hookshot_bg_check_off_timer;
        ram[INDEX_OF_DASHING_SFX] = self.index_of_dashing_sfx;
        ram[LINK_SPIN_OFFSETS] = self.spin_offsets;
        ram[PLAYER_ON_SOMARIA_PLATFORM] = self.somaria_platform_state;
        ram[PLAYER_NEAR_PIT_STATE] = self.near_pit_state;
        ram[PLAYER_PIT_DATA_INDEX] = self.pit_data_index;
        ram[PIT_CORRECTION_TIMER] = self.pit_correction_timer;
        ram[PIT_CORRECTION_ACTIVE_FLAG] = self.pit_correction_active;
        ram[MOVING_AGAINST_DIAG_DEADLOCKED] = self.moving_against_diag_deadlocked;
        ram[LINK_INCAPACITATED_CAMERA_TIMER] = self.incapacitated_camera_timer;
        ram[LINK_DISABLE_SPRITE_DAMAGE] = self.sprite_damage_disabled;
        write_le_u16(ram, LINK_DMA_GRAPHICS_INDEX, self.link_dma_graphics_index);
        write_le_u16(
            ram,
            LINK_DMA_LEFT_SPRITE_BANK_INDEX,
            self.link_dma_left_sprite_bank,
        );
        write_le_u16(
            ram,
            LINK_DMA_RIGHT_SPRITE_BANK_INDEX,
            self.link_dma_right_sprite_bank,
        );
        ram[LINK_DMA_SWORD_GRAPHICS_INDEX] = self.sword_dma_graphics_index;
        ram[LINK_DMA_SHIELD_GRAPHICS_INDEX] = self.shield_dma_graphics_index;
        ram[LINK_DMA_STAGING_INDEX] = self.link_dma_staging_index;
        write_le_u16(ram, LINK_DMA_SOURCE_OFFSET, self.link_dma_source_offset);
        write_le_u16(ram, LINK_DMA_TILE_OFFSET, self.link_dma_tile_offset);
        write_le_u16(ram, LINK_DMA_COUNTDOWN, self.link_dma_countdown);
        write_le_u16(ram, LINK_PALETTE_BITS_OF_OAM, self.palette_bits_of_oam);
        write_le_u16(ram, SCRATCH_1, self.link_sprite_index_scratch);
        write_le_u16(ram, LINK_Y_COORD_ORIGINAL, self.hop_origin_coord);
        write_le_u16(ram, LINK_X_COORD_CACHED, self.cached_x);
        write_le_u16(ram, LINK_Y_COORD_CACHED, self.cached_y);
        write_le_u16(ram, LINK_X_COORD_COPY, self.copied_x);
        write_le_u16(ram, LINK_Y_COORD_COPY, self.copied_y);
        write_le_u16(ram, LINK_X_COORD_PREV, self.previous_x);
        write_le_u16(ram, LINK_Y_COORD_PREV, self.previous_y);
        ram[LINK_X_COORD_SAFE_RETURN_LO] = self.safe_return_x as u8;
        ram[LINK_X_COORD_SAFE_RETURN_HI] = (self.safe_return_x >> 8) as u8;
        ram[LINK_Y_COORD_SAFE_RETURN_LO] = self.safe_return_y as u8;
        ram[LINK_Y_COORD_SAFE_RETURN_HI] = (self.safe_return_y >> 8) as u8;
        write_le_u16(ram, BIT9_OF_XCOORD, self.bit9_of_xcoord);
        ram[SOMARIA_BLOCK_BG_CHECK_FLAG] = self.somaria_block_bg_check_flag;
        ram[PLAYER_POSE_DRAW_COUNTER] = self.player_pose_draw_counter;
        ram[PLAYER_SPECIAL_DRAW_FLAG] = self.player_special_draw_flag;
        ram[PLAYER_SLEEP_IN_BED_STATE] = self.sleep_in_bed_state;
        ram[CHEAT_WALK_THROUGH_WALLS] = self.cheat_walk_through_walls;
        ram[LINK_X_PAGE_MOVEMENT_DELTA] = self.x_page_movement_delta;
        ram[LINK_Y_PAGE_MOVEMENT_DELTA] = self.y_page_movement_delta;
        write_le_u16(ram, RELATED_TO_MOVING_FLOOR_X, self.moving_floor_x);
        write_le_u16(ram, RELATED_TO_MOVING_FLOOR_Y, self.moving_floor_y);
        write_le_u16(ram, DRAG_PLAYER_X, self.drag_player_x);
        write_le_u16(ram, DRAG_PLAYER_Y, self.drag_player_y);
    }

    pub(crate) fn x(&self) -> u16 {
        self.x
    }

    pub(crate) fn y(&self) -> u16 {
        self.y
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.x as u8
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.y as u8
    }

    pub(crate) fn safe_return_x_high(&self) -> u8 {
        (self.safe_return_x >> 8) as u8
    }

    pub(crate) fn safe_return_y_high(&self) -> u8 {
        (self.safe_return_y >> 8) as u8
    }

    pub(crate) fn safe_return_y_low(&self) -> u8 {
        self.safe_return_y as u8
    }

    pub(crate) fn y_low_delta_from_safe_return(&self) -> u8 {
        self.y_low().wrapping_sub(self.safe_return_y_low())
    }

    pub(crate) fn safe_return_x(&self) -> u16 {
        self.safe_return_x
    }

    pub(crate) fn safe_return_y(&self) -> u16 {
        self.safe_return_y
    }

    pub(crate) fn x_high(&self) -> u8 {
        (self.x >> 8) as u8
    }

    pub(crate) fn y_high(&self) -> u8 {
        (self.y >> 8) as u8
    }

    pub(crate) fn z(&self) -> u16 {
        self.z
    }

    pub(crate) fn z_low(&self) -> u8 {
        self.z as u8
    }

    pub(crate) fn z_low_signed(&self) -> i8 {
        self.z_low() as i8
    }

    pub(crate) fn is_z_low_negative(&self) -> bool {
        self.z_low_signed().is_negative()
    }

    pub(crate) fn z_mirror(&self) -> u16 {
        self.z_mirror
    }

    pub(crate) fn z_mirror_low(&self) -> u8 {
        self.z_mirror as u8
    }

    pub(crate) fn z_mirror_delta_low(&self) -> u8 {
        self.z_mirror_low().wrapping_sub(self.z_low())
    }

    pub(crate) fn oam_x_offset(&self) -> u8 {
        self.oam_x_offset
    }

    pub(crate) fn oam_y_offset(&self) -> u8 {
        self.oam_y_offset
    }

    pub(crate) fn oam_x_offset_signed(&self) -> i8 {
        self.oam_x_offset as i8
    }

    pub(crate) fn oam_y_offset_signed(&self) -> i8 {
        self.oam_y_offset as i8
    }

    pub(crate) fn has_disabled_oam_offsets(&self) -> bool {
        self.oam_y_offset == 0x80
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        self.x_subpixel
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        self.y_subpixel
    }

    pub(crate) fn is_grounded_or_z_sentinel(&self) -> bool {
        self.z_low() == 0 || self.z_low() >= 0xe0
    }

    pub(crate) fn is_landing_at_or_above_ground(&self) -> bool {
        self.z >= 0xfff0
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

    pub(crate) fn x_velocity(&self) -> u8 {
        self.x_velocity
    }

    pub(crate) fn x_velocity_signed(&self) -> i8 {
        self.x_velocity as i8
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.y_velocity
    }

    pub(crate) fn y_velocity_signed(&self) -> i8 {
        self.y_velocity as i8
    }

    pub(crate) fn actual_z_velocity(&self) -> u8 {
        self.z_velocity
    }

    pub(crate) fn recoil_z_velocity_for_dungeon_reset(&self) -> u8 {
        self.recoil_z_velocity_for_dungeon_reset
    }

    pub(crate) fn actual_z_velocity_copy(&self) -> u8 {
        self.z_velocity_copy
    }

    pub(crate) fn actual_z_velocity_mirror(&self) -> u8 {
        self.z_velocity_mirror
    }

    pub(crate) fn x_page_movement_delta(&self) -> u8 {
        self.x_page_movement_delta
    }

    pub(crate) fn y_page_movement_delta(&self) -> u8 {
        self.y_page_movement_delta
    }

    pub(crate) fn x_page_movement_delta_signed(&self) -> i8 {
        self.x_page_movement_delta as i8
    }

    pub(crate) fn y_page_movement_delta_signed(&self) -> i8 {
        self.y_page_movement_delta as i8
    }

    pub(crate) fn recoil_timer(&self) -> u8 {
        self.recoil_timer
    }

    pub(crate) fn actual_x_velocity(&self) -> u8 {
        self.actual_x_velocity
    }

    pub(crate) fn actual_x_velocity_signed(&self) -> i8 {
        self.actual_x_velocity as i8
    }

    pub(crate) fn actual_y_velocity(&self) -> u8 {
        self.actual_y_velocity
    }

    pub(crate) fn actual_y_velocity_signed(&self) -> i8 {
        self.actual_y_velocity as i8
    }

    pub(crate) fn floor(&self) -> u8 {
        self.floor
    }

    pub(crate) fn is_on_lower_level(&self) -> bool {
        self.floor != 0
    }

    pub(crate) fn lower_level_tilemap_offset(&self) -> u16 {
        if self.is_on_lower_level() {
            0x1000
        } else {
            0
        }
    }

    pub(crate) fn has_lower_level_state_or_mirror(&self) -> bool {
        self.floor | self.lower_level_mirror_state != 0
    }

    pub(crate) fn lower_level_state(&self) -> u8 {
        self.floor
    }

    pub(crate) fn lower_level_mirror_state(&self) -> u8 {
        self.lower_level_mirror_state
    }

    pub(crate) fn cached_lower_level_state(&self) -> u8 {
        self.cached_lower_level_state
    }

    pub(crate) fn cached_lower_level_mirror_state(&self) -> u8 {
        self.cached_lower_level_mirror_state
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

    pub(crate) fn direction(&self) -> u8 {
        self.direction
    }

    pub(crate) fn direction_lock(&self) -> u8 {
        self.direction_lock
    }

    pub(crate) fn direction_lock_has(&self, mask: u8) -> bool {
        self.direction_lock & mask != 0
    }

    pub(crate) fn moving_against_diag_tile(&self) -> u8 {
        self.moving_against_diag_tile
    }

    pub(crate) fn flag_moving(&self) -> u8 {
        self.movement_flag
    }

    pub(crate) fn quadrant_x(&self) -> u8 {
        self.quadrant_x
    }

    pub(crate) fn quadrant_y(&self) -> u8 {
        self.quadrant_y
    }

    pub(crate) fn quadrant_visit_index(&self, fullsize_y: u8, fullsize_x: u8) -> usize {
        ((fullsize_y as usize) << 2)
            + ((fullsize_x as usize) << 1)
            + self.quadrant_y as usize
            + self.quadrant_x as usize
    }

    pub(crate) fn quadrant_x_mask(&self) -> u8 {
        if self.quadrant_x != 0 {
            2
        } else {
            1
        }
    }

    pub(crate) fn quadrant_y_mask(&self) -> u8 {
        if self.quadrant_y != 0 {
            8
        } else {
            4
        }
    }

    pub(crate) fn is_moving_against_diag_tile_on_both_axes(&self) -> bool {
        self.moving_against_diag_tile & 0x0c != 0 && self.moving_against_diag_tile & 3 != 0
    }

    pub(crate) fn has_swim_axis_drag(&self) -> bool {
        (self.num_orthogonal_directions | self.moving_against_diag_tile) != 0
    }

    pub(crate) fn num_orthogonal_directions(&self) -> u8 {
        self.num_orthogonal_directions
    }

    pub(crate) fn last_direction_moved_towards(&self) -> u8 {
        self.last_direction_moved_towards
    }

    pub(crate) fn last_direction_moved_towards_index(&self) -> usize {
        usize::from(self.last_direction_moved_towards)
    }

    pub(crate) fn last_direction(&self) -> u8 {
        self.last_direction
    }

    pub(crate) fn facing(&self) -> u8 {
        self.facing
    }

    pub(crate) fn has_facing(&self) -> bool {
        self.facing != 0
    }

    pub(crate) fn facing_index(&self) -> usize {
        usize::from(self.facing >> 1)
    }

    pub(crate) fn facing_mirror_index(&self) -> usize {
        usize::from(self.facing_mirror >> 1)
    }

    pub(crate) fn facing_layer_bits(&self) -> u8 {
        self.facing >> 1
    }

    pub(crate) fn swim_direction_flags(&self) -> u8 {
        self.swim_direction_flags
    }

    pub(crate) fn speed_setting(&self) -> u8 {
        self.speed_setting
    }

    pub(crate) fn speed_modifier(&self) -> u8 {
        self.speed_modifier
    }

    pub(crate) fn dash_counter(&self) -> u8 {
        self.dash_counter
    }

    pub(crate) fn dash_countdown(&self) -> u8 {
        self.dash_countdown
    }

    pub(crate) fn jump_ledge_timer(&self) -> u8 {
        self.jump_ledge_timer
    }

    pub(crate) fn about_to_jump_off_ledge(&self) -> u8 {
        self.about_to_jump_off_ledge
    }

    pub(crate) fn push_fatigue_timer(&self) -> u8 {
        self.push_fatigue_timer
    }

    pub(crate) fn gravestone_push_timeout(&self) -> u8 {
        self.gravestone_push_timeout
    }

    pub(crate) fn menu_block_flag(&self) -> u8 {
        self.menu_block_flag
    }

    pub(crate) fn is_menu_blocked(&self) -> bool {
        self.menu_block_flag != 0
    }

    pub(crate) fn has_menu_block_flag(&self, value: u8) -> bool {
        self.menu_block_flag == value
    }

    pub(crate) fn handler_state(&self) -> u8 {
        self.handler_state
    }

    pub(crate) fn is_edge_transition_blocked_by_handler_state(&self) -> bool {
        matches!(self.handler_state, 3 | 8 | 9 | 10)
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

    pub(crate) fn immobilized_flag(&self) -> u8 {
        self.immobilized
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

    pub(crate) fn state_bits(&self) -> u8 {
        self.action_state_bits
    }

    pub(crate) fn state_bits_has(&self, mask: u8) -> bool {
        self.action_state_bits & mask != 0
    }

    pub(crate) fn has_non_lift_action_state(&self) -> bool {
        self.action_state_bits & 0x7f != 0
    }

    pub(crate) fn is_lifting_or_carrying(&self) -> bool {
        self.action_state_bits & 0x80 != 0
    }

    pub(crate) fn auxiliary_state(&self) -> u8 {
        self.auxiliary_state
    }

    pub(crate) fn is_in_auxiliary_state(&self, value: u8) -> bool {
        self.auxiliary_state == value
    }

    pub(crate) fn has_auxiliary_state(&self) -> bool {
        self.auxiliary_state != 0
    }

    pub(crate) fn is_running(&self) -> bool {
        self.running != 0
    }

    pub(crate) fn running_state(&self) -> u8 {
        self.running
    }

    pub(crate) fn item_in_hand(&self) -> u8 {
        self.item_in_hand
    }

    pub(crate) fn item_hold_pose(&self) -> u8 {
        self.item_hold_pose
    }

    pub(crate) fn force_hold_sword_up_state(&self) -> u8 {
        self.force_hold_sword_up
    }

    pub(crate) fn has_item_in_hand(&self) -> bool {
        self.item_in_hand != 0
    }

    pub(crate) fn item_in_hand_has(&self, mask: u8) -> bool {
        self.item_in_hand & mask != 0
    }

    pub(crate) fn has_item_or_position_mode(&self) -> bool {
        self.item_in_hand | self.position_mode != 0
    }

    pub(crate) fn has_position_mode(&self) -> bool {
        self.position_mode != 0
    }

    pub(crate) fn position_mode(&self) -> u8 {
        self.position_mode
    }

    pub(crate) fn position_mode_has(&self, mask: u8) -> bool {
        self.position_mode & mask != 0
    }

    pub(crate) fn picking_throw_state(&self) -> u8 {
        self.picking_throw_state
    }

    pub(crate) fn picking_throw_state_has(&self, mask: u8) -> bool {
        self.picking_throw_state & mask != 0
    }

    pub(crate) fn has_picking_throw_state(&self) -> bool {
        self.picking_throw_state != 0
    }

    pub(crate) fn is_lift_throw_primed(&self) -> bool {
        self.picking_throw_state & 1 != 0
    }

    pub(crate) fn is_ready_to_start_ground_movement(&self) -> bool {
        (self.grabbing_wall & !2) == 0
            && !self.has_non_lift_action_state()
            && (!self.is_lifting_or_carrying() || self.picking_throw_state & 1 == 0)
            && !self.has_item_or_position_mode()
    }

    pub(crate) fn has_grabbing_wall_state(&self) -> bool {
        self.grabbing_wall != 0
    }

    pub(crate) fn grabbing_wall(&self) -> u8 {
        self.grabbing_wall
    }

    pub(crate) fn grabbing_wall_has(&self, mask: u8) -> bool {
        self.grabbing_wall & mask != 0
    }

    pub(crate) fn button_mask_b_y(&self) -> u8 {
        self.button_mask_b_y
    }

    pub(crate) fn filtered_joypad_h(&self) -> u8 {
        self.filtered_joypad_h
    }

    pub(crate) fn filtered_joypad_l(&self) -> u8 {
        self.filtered_joypad_l
    }

    pub(crate) fn joypad1h_last(&self) -> u8 {
        self.joypad1h_last
    }

    pub(crate) fn joypad1l_last(&self) -> u8 {
        self.joypad1l_last
    }

    pub(crate) fn joypad1h_last2(&self) -> u8 {
        self.joypad1h_last2
    }

    pub(crate) fn joypad1l_last2(&self) -> u8 {
        self.joypad1l_last2
    }

    pub(crate) fn spin_attack_delay_timer(&self) -> u8 {
        self.spin_attack_delay_timer
    }

    pub(crate) fn spin_attack_step_counter(&self) -> u8 {
        self.spin_attack_step_counter
    }

    pub(crate) fn incapacitated_timer(&self) -> u8 {
        self.incapacitated_timer
    }

    pub(crate) fn visibility_status(&self) -> u8 {
        self.visibility_status
    }

    pub(crate) fn y_button_action_flags(&self) -> u8 {
        self.y_button_action_flags
    }

    pub(crate) fn y_button_action_step(&self) -> u8 {
        self.y_button_action_step
    }

    pub(crate) fn y_button_action_timer(&self) -> u8 {
        self.y_button_action_timer
    }

    pub(crate) fn defense_flags(&self) -> u8 {
        self.defense_flags
    }

    pub(crate) fn electrocute_on_touch(&self) -> u8 {
        self.electrocute_on_touch
    }

    pub(crate) fn is_cape_active(&self) -> bool {
        self.cape_mode != 0
    }

    pub(crate) fn cape_decrement_counter(&self) -> u8 {
        self.cape_decrement_counter
    }

    pub(crate) fn sprite_damage_disable_timer(&self) -> u8 {
        self.sprite_damage_disabled
    }

    pub(crate) fn link_dma_graphics_index_word(&self) -> u16 {
        self.link_dma_graphics_index
    }

    pub(crate) fn link_dma_staging_index(&self) -> u8 {
        self.link_dma_staging_index
    }

    pub(crate) fn hop_origin_coord(&self) -> u16 {
        self.hop_origin_coord
    }

    pub(crate) fn drag_player_x(&self) -> u16 {
        self.drag_player_x
    }

    pub(crate) fn drag_player_y(&self) -> u16 {
        self.drag_player_y
    }

    pub(crate) fn on_somaria_platform(&self) -> u8 {
        self.somaria_platform_state
    }

    pub(crate) fn has_somaria_platform_state(&self) -> bool {
        self.somaria_platform_state != 0
    }

    pub(crate) fn near_pit_state(&self) -> u8 {
        self.near_pit_state
    }

    pub(crate) fn is_near_pit(&self) -> bool {
        self.near_pit_state != 0
    }

    pub(crate) fn near_pit_state_is(&self, value: u8) -> bool {
        self.near_pit_state == value
    }

    pub(crate) fn near_pit_state_at_least(&self, value: u8) -> bool {
        self.near_pit_state >= value
    }

    pub(crate) fn pit_data_index(&self) -> u8 {
        self.pit_data_index
    }

    pub(crate) fn pit_correction_timer(&self) -> u8 {
        self.pit_correction_timer
    }

    pub(crate) fn pit_correction_active(&self) -> bool {
        self.pit_correction_active != 0
    }

    pub(crate) fn moving_against_diag_deadlocked(&self) -> u8 {
        self.moving_against_diag_deadlocked
    }

    pub(crate) fn item_receipt_method(&self) -> u8 {
        self.item_receipt_method
    }

    pub(crate) fn action_handler_timer(&self) -> u8 {
        self.action_handler_timer
    }

    pub(crate) fn doorway_state(&self) -> u8 {
        self.doorway_state
    }

    pub(crate) fn blink_countdown(&self) -> u8 {
        self.blink_countdown
    }

    pub(crate) fn is_bunny(&self) -> bool {
        self.bunny_state != 0
    }

    pub(crate) fn is_bunny_mirror(&self) -> bool {
        self.bunny_mirror != 0
    }

    pub(crate) fn temp_bunny_timer(&self) -> u16 {
        self.temp_bunny_timer
    }

    pub(crate) fn needs_transform_poof(&self) -> bool {
        self.transform_poof_needed != 0
    }

    pub(crate) fn spin_animation_step_counter(&self) -> u8 {
        self.spin_animation_step_counter
    }

    pub(crate) fn state_for_spin_attack(&self) -> u8 {
        self.spin_attack_state
    }

    pub(crate) fn spin_attack_sound_latch(&self) -> u8 {
        self.spin_attack_sound_latch
    }

    pub(crate) fn button_b_frames(&self) -> u8 {
        self.button_b_frames
    }

    pub(crate) fn button_b_frames_word(&self) -> u16 {
        // The ether/bombos cutscene reinterprets BUTTON_B_FRAMES (0x3c) and the
        // adjacent LINK_DELAY_TIMER_SPIN_ATTACK (0x3d) as one 16-bit counter.
        u16::from(self.button_b_frames) | (u16::from(self.spin_attack_delay_timer) << 8)
    }

    pub(crate) fn button_b_frames_index(&self) -> usize {
        usize::from(self.button_b_frames())
    }

    pub(crate) fn animation_step(&self) -> u8 {
        self.animation_step
    }

    pub(crate) fn opening_pose(&self) -> u8 {
        self.opening_pose
    }

    pub(crate) fn animation_step_index(&self) -> usize {
        usize::from(self.animation_step)
    }

    pub(crate) fn water_ripple_or_grass_state(&self) -> u8 {
        self.water_ripple_or_grass_state
    }

    pub(crate) fn primary_water_grass_timer(&self) -> u8 {
        self.primary_water_grass_timer
    }

    pub(crate) fn secondary_water_grass_timer(&self) -> u8 {
        self.secondary_water_grass_timer
    }

    pub(crate) fn deep_water_state(&self) -> u8 {
        self.deep_water_state
    }

    pub(crate) fn swim_fast_state(&self) -> u8 {
        self.swim_fast_state
    }

    pub(crate) fn hard_swim_stroke(&self) -> u8 {
        self.hard_swim_stroke
    }

    pub(crate) fn swim_stroke_frame_counter(&self, offset: usize) -> u16 {
        let axis =
            swim_axis_index(offset).expect("swim stroke frame counter offset must be 0 or 2");
        self.swim_stroke_frame_counters[axis]
    }

    pub(crate) fn swim_stroke_anim_step(&self) -> u8 {
        self.swim_stroke_anim_step
    }

    pub(crate) fn sleep_in_bed_state(&self) -> u8 {
        self.sleep_in_bed_state
    }

    pub(crate) fn is_in_deep_water(&self) -> bool {
        self.deep_water_state != 0
    }

    pub(crate) fn conveyor_belt_state(&self) -> u8 {
        self.conveyor_belt_state
    }

    pub(crate) fn tile_below(&self) -> u8 {
        self.tile_below
    }

    pub(crate) fn tile_action_index(&self) -> u8 {
        self.tile_action_index
    }

    pub(crate) fn tile_coll_flag(&self) -> u8 {
        self.tile_collision_flag
    }

    pub(crate) fn sprite_oam_state_timer(&self) -> u8 {
        self.sprite_oam_state_timer
    }

    pub(crate) fn whirlpool_triggered(&self) -> bool {
        self.whirlpool_trigger != 0
    }

    pub(crate) fn is_prevented_from_moving(&self) -> bool {
        self.prevent_movement != 0
    }

    pub(crate) fn force_move_any_direction_lo(&self) -> u16 {
        u16::from(self.force_move_any_direction as u8)
    }

    pub(crate) fn force_move_any_direction(&self) -> u16 {
        self.force_move_any_direction
    }

    pub(crate) fn item_action_step_var(&self) -> u8 {
        self.item_action_step
    }

    pub(crate) fn throw_oam_state_index(&self) -> u8 {
        self.throw_oam_state_index
    }

    pub(crate) fn item_action_debug_value_2(&self) -> u8 {
        self.item_action_debug_value_2
    }

    pub(crate) fn item_debug_value_1(&self) -> u8 {
        self.item_debug_value_1
    }

    pub(crate) fn sword_dma_graphics_index(&self) -> u8 {
        self.sword_dma_graphics_index
    }

    pub(crate) fn shield_dma_graphics_index(&self) -> u8 {
        self.shield_dma_graphics_index
    }

    pub(crate) fn link_dma_left_sprite_bank_word(&self) -> u16 {
        self.link_dma_left_sprite_bank
    }

    pub(crate) fn link_dma_right_sprite_bank_word(&self) -> u16 {
        self.link_dma_right_sprite_bank
    }

    pub(crate) fn link_dma_staging_group(&self) -> u8 {
        self.link_dma_staging_index >> 3
    }

    pub(crate) fn palette_bits_of_oam(&self) -> u8 {
        self.palette_bits_of_oam as u8
    }

    pub(crate) fn palette_bits_of_oam_word(&self) -> u16 {
        self.palette_bits_of_oam
    }

    pub(crate) fn cached_x(&self) -> u16 {
        self.cached_x
    }

    pub(crate) fn cached_y(&self) -> u16 {
        self.cached_y
    }

    pub(crate) fn copied_x(&self) -> u16 {
        self.copied_x
    }

    pub(crate) fn copied_y(&self) -> u16 {
        self.copied_y
    }

    pub(crate) fn bit9_of_xcoord(&self) -> u8 {
        self.bit9_of_xcoord as u8
    }

    pub(crate) fn somaria_block_bg_check_flag(&self) -> u8 {
        self.somaria_block_bg_check_flag
    }

    pub(crate) fn player_pose_draw_counter(&self) -> u8 {
        self.player_pose_draw_counter
    }

    pub(crate) fn player_special_draw_flag(&self) -> u8 {
        self.player_special_draw_flag
    }

    pub(crate) fn moving_floor_x(&self) -> u16 {
        self.moving_floor_x
    }

    pub(crate) fn moving_floor_y(&self) -> u16 {
        self.moving_floor_y
    }

    pub(crate) fn sprite_pickup_flag_cached(&self) -> u8 {
        self.sprite_pickup_flag_cached
    }

    pub(crate) fn cheat_walk_through_walls(&self) -> u8 {
        self.cheat_walk_through_walls
    }

    pub(crate) fn sword_delay_timer(&self) -> u8 {
        self.sword_delay_timer
    }

    pub(crate) fn spin_offsets(&self) -> u8 {
        self.spin_offsets
    }

    pub(crate) fn dash_noise_requested(&self) -> bool {
        self.dash_noise_requested != 0
    }

    pub(crate) fn is_transforming(&self) -> bool {
        self.transforming != 0
    }

    pub(crate) fn needs_pull_for_rupees_sprite(&self) -> bool {
        self.pull_for_rupees_sprite_needed != 0
    }

    pub(crate) fn is_near_moveable_statue(&self) -> bool {
        self.near_moveable_statue_flag != 0
    }

    pub(crate) fn given_damage(&self) -> u8 {
        self.given_damage
    }

    pub(crate) fn has_pull_action_state(&self) -> bool {
        self.pull_action_state != 0
    }

    pub(crate) fn pull_action_state(&self) -> u8 {
        self.pull_action_state
    }

    pub(crate) fn current_item_y(&self) -> u8 {
        self.current_item_y
    }

    pub(crate) fn current_item_active(&self) -> u8 {
        self.current_item_active
    }

    pub(crate) fn receive_item_index(&self) -> u8 {
        self.receive_item_index
    }

    pub(crate) fn item_pickup_in_progress(&self) -> bool {
        self.item_pickup_in_progress != 0
    }

    pub(crate) fn selected_rod(&self) -> u8 {
        self.selected_rod
    }

    pub(crate) fn has_flippers(&self) -> bool {
        self.flippers != 0
    }

    pub(crate) fn flippers(&self) -> u8 {
        self.flippers
    }

    pub(crate) fn moon_pearl(&self) -> u8 {
        self.moon_pearl
    }

    pub(crate) fn has_moon_pearl(&self) -> bool {
        self.moon_pearl != 0
    }

    pub(crate) fn magic_power(&self) -> u8 {
        self.magic_power
    }

    pub(crate) fn magic_consumption_level(&self) -> u8 {
        self.magic_consumption
    }

    pub(crate) fn ancilla_pickup_flag(&self) -> u8 {
        self.ancilla_pickup_flag
    }

    pub(crate) fn sprite_pickup_flag(&self) -> u8 {
        self.sprite_pickup_flag
    }

    pub(crate) fn hookshot_interlock(&self) -> u8 {
        self.hookshot_interlock
    }

    pub(crate) fn has_hookshot_interlock(&self) -> bool {
        self.hookshot_interlock != 0
    }

    pub(crate) fn hookshot_grave_latch(&self) -> bool {
        self.hookshot_grave_latch != 0
    }

    pub(crate) fn faint_animation_active(&self) -> u8 {
        self.faint_animation_active
    }

    pub(crate) fn flute_countdown(&self) -> u8 {
        self.flute_countdown
    }

    pub(crate) fn hookshot_bg_check_off_timer(&self) -> u8 {
        self.hookshot_bg_check_off_timer
    }

    pub(crate) fn index_of_dashing_sfx(&self) -> u8 {
        self.index_of_dashing_sfx
    }

    pub(crate) fn hookshot_interlock_has(&self, mask: u8) -> bool {
        self.hookshot_interlock & mask != 0
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

    fn decrement_speed_setting(&mut self) -> u8 {
        self.speed_setting = self.speed_setting.wrapping_sub(1);
        self.speed_setting
    }

    fn clear_speed_modifier(&mut self) {
        self.speed_modifier = 0;
    }

    fn set_speed_modifier(&mut self, value: u8) {
        self.speed_modifier = value;
    }

    fn mark_lower_level(&mut self) {
        self.floor = 1;
    }

    fn mark_lower_level_mirror(&mut self) {
        self.lower_level_mirror_state = 1;
    }

    fn set_lower_level_state(&mut self, value: u8) {
        self.floor = value;
    }

    fn set_lower_level_mirror_state(&mut self, value: u8) {
        self.lower_level_mirror_state = value;
    }

    fn set_lower_level_states(&mut self, state: u8, mirror: u8) {
        self.floor = state;
        self.lower_level_mirror_state = mirror;
    }

    fn clear_lower_level(&mut self) {
        self.floor = 0;
    }

    fn clear_lower_level_states(&mut self) {
        self.floor = 0;
        self.lower_level_mirror_state = 0;
    }

    fn toggle_lower_level_state(&mut self) {
        self.floor ^= 1;
    }

    fn toggle_lower_level_mirror_state(&mut self) {
        self.lower_level_mirror_state ^= 1;
    }

    fn mirror_lower_level_state(&mut self) {
        self.lower_level_mirror_state = self.floor;
    }

    fn cache_lower_level_states(&mut self) {
        self.cached_lower_level_state = self.floor;
        self.cached_lower_level_mirror_state = self.lower_level_mirror_state;
    }

    fn restore_lower_level_state_from_cached(&mut self) {
        self.floor = self.cached_lower_level_state;
        self.lower_level_mirror_state = self.cached_lower_level_mirror_state;
    }

    fn arm_stair_speed_modifier(&mut self) {
        self.speed_setting = 2;
        self.speed_modifier = 1;
    }

    fn resolve_dash_speed_setting(&mut self) {
        if self.speed_setting == 2 {
            self.speed_setting = if self.running != 0 { 16 } else { 0 };
        }
    }

    fn promote_pending_speed_modifier(&mut self) {
        if self.speed_modifier == 1 {
            self.speed_modifier = 2;
        }
    }

    fn increase_near_pit_speed_modifier(&mut self) {
        self.speed_modifier = if self.speed_modifier < 48 {
            self.speed_modifier.wrapping_add(8)
        } else {
            32
        };
    }

    fn advance_dash_deceleration(&mut self) {
        self.speed_modifier = self.speed_modifier.wrapping_add(1);
    }

    fn set_dash_countdown(&mut self, value: u8) {
        self.dash_countdown = value;
    }

    fn increment_dash_countdown(&mut self) -> u8 {
        self.dash_countdown = self.dash_countdown.wrapping_add(1);
        self.dash_countdown
    }

    fn decrement_dash_countdown(&mut self) -> u8 {
        self.dash_countdown = self.dash_countdown.wrapping_sub(1);
        self.dash_countdown
    }

    fn set_dash_counter(&mut self, value: u8) {
        self.dash_counter = value;
    }

    fn prime_dash_counter(&mut self) {
        self.dash_counter = 64;
    }

    fn decrement_dash_counter_clamped_to_minimum(&mut self, minimum: u8) {
        self.dash_counter = self.dash_counter.wrapping_sub(1);
        if self.dash_counter < minimum {
            self.dash_counter = minimum;
        }
    }

    fn set_menu_block_flag(&mut self, value: u8) {
        self.menu_block_flag = value;
    }

    fn clear_menu_block(&mut self) {
        self.menu_block_flag = 0;
    }

    fn increment_menu_block_flag(&mut self) -> u8 {
        self.menu_block_flag = self.menu_block_flag.wrapping_add(1);
        self.menu_block_flag
    }

    fn set_handler_state(&mut self, value: u8) {
        self.handler_state = value;
    }

    fn clear_handler_state(&mut self) {
        self.handler_state = 0;
    }

    fn set_facing(&mut self, value: u8) {
        self.facing = value;
    }

    fn restore_facing_from_cached(&mut self) {
        self.facing = self.cached_facing;
    }

    fn set_facing_mirror(&mut self, value: u8) {
        self.facing_mirror = value;
    }

    fn cache_facing_to_mirror(&mut self) {
        self.facing_mirror = self.facing;
    }

    fn cache_facing(&mut self) {
        self.cached_facing = self.facing;
    }

    fn set_moving_against_diag_tile(&mut self, value: u8) {
        self.moving_against_diag_tile = value;
    }

    fn add_moving_against_diag_tile_flags(&mut self, value: u8) {
        self.moving_against_diag_tile |= value;
    }

    fn clear_moving_against_diag_tile(&mut self) {
        self.moving_against_diag_tile = 0;
    }

    fn set_flag_moving(&mut self, value: u8) {
        self.movement_flag = value;
    }

    fn clear_flag_moving(&mut self) {
        self.movement_flag = 0;
    }

    fn set_quadrants_from_packed_nibbles(&mut self, value: u8) {
        self.quadrant_x = value >> 4;
        self.quadrant_y = value & 0x0f;
    }

    fn set_quadrants(&mut self, x: u8, y: u8) {
        self.quadrant_x = x;
        self.quadrant_y = y;
    }

    fn toggle_quadrant_x(&mut self) -> u8 {
        self.quadrant_x ^= 1;
        self.quadrant_x
    }

    fn toggle_quadrant_y(&mut self) -> u8 {
        self.quadrant_y ^= 2;
        self.quadrant_y
    }

    fn reset_direction_limits(&mut self) {
        self.direction_mask_a = 0x0f;
        self.direction_mask_b = 0x0f;
        self.num_orthogonal_directions = 0;
    }

    fn reset_direction_masks(&mut self) {
        self.direction_mask_a = 0x0f;
        self.direction_mask_b = 0x0f;
    }

    fn increment_orthogonal_direction_count(&mut self) {
        self.num_orthogonal_directions = self.num_orthogonal_directions.wrapping_add(1);
    }

    fn clear_orthogonal_direction_count(&mut self) {
        self.num_orthogonal_directions = 0;
    }

    fn set_last_direction_moved_towards(&mut self, value: u8) {
        self.last_direction_moved_towards = value;
    }

    fn set_last_direction_from_current_direction(&mut self) {
        self.last_direction = self.direction;
    }

    fn set_last_direction(&mut self, value: u8) {
        self.last_direction = value;
    }

    fn mask_last_direction(&mut self, mask: u8) {
        self.last_direction &= mask;
    }

    fn set_last_direction_from_swim_flags(&mut self) {
        self.last_direction = self.swim_direction_flags;
    }

    fn set_swim_flags_from_last_direction(&mut self) {
        self.swim_direction_flags = self.last_direction;
    }

    fn set_direction(&mut self, value: u8) {
        self.direction = value;
    }

    fn set_direction_and_last_direction(&mut self, value: u8) {
        self.direction = value;
        self.last_direction = value;
    }

    fn set_direction_and_swim_flags(&mut self, value: u8) {
        self.direction = value;
        self.swim_direction_flags = value;
    }

    fn mask_direction(&mut self, mask: u8) {
        self.direction &= mask;
    }

    fn clear_cardinal_direction(&mut self) {
        self.direction &= !0x0f;
    }

    fn add_direction_flags(&mut self, flags: u8) {
        self.direction |= flags;
    }

    fn clear_direction_flags(&mut self, flags: u8) {
        self.direction &= !flags;
    }

    fn set_direction_lock(&mut self, value: u8) {
        self.direction_lock = value;
    }

    fn clear_direction_lock(&mut self) {
        self.direction_lock = 0;
    }

    fn set_direction_lock_bits(&mut self, mask: u8) {
        self.direction_lock |= mask;
    }

    fn clear_direction_lock_bits(&mut self, mask: u8) {
        self.direction_lock &= !mask;
    }

    fn set_direction_mask_a(&mut self, value: u8) {
        self.direction_mask_a = value;
    }

    fn set_direction_mask_b(&mut self, value: u8) {
        self.direction_mask_b = value;
    }

    fn apply_direction_masks(&mut self) {
        self.direction &= self.direction_mask_a & self.direction_mask_b;
    }

    fn force_direction_from_diag_tile_if_needed(&mut self) {
        if self.direction & 0x0f != 0 && self.moving_against_diag_tile & 0x0f != 0 {
            self.direction = self.moving_against_diag_tile & 0x0f;
        }
    }

    fn resolve_orthogonal_direction_count_from_facing(&mut self) {
        self.num_orthogonal_directions = if self.num_orthogonal_directions == 2 {
            if self.facing & 4 != 0 {
                2
            } else {
                1
            }
        } else {
            0
        };
    }

    fn mark_moving_floor_direction(&mut self, floor_y: u16, floor_x: u16) {
        if floor_y != 0 {
            self.direction |= if (floor_y as i16).is_negative() { 8 } else { 4 };
        }
        if floor_x != 0 {
            self.direction |= if (floor_x as i16).is_negative() { 2 } else { 1 };
        }
    }

    fn set_last_direction_moved_towards_from_facing(&mut self) {
        self.last_direction_moved_towards = self.facing >> 1;
    }

    fn set_swim_direction_flags(&mut self, direction: u8) {
        self.swim_direction_flags = direction;
    }

    fn set_y(&mut self, value: u16) {
        self.y = value;
    }

    fn set_x(&mut self, value: u16) {
        self.x = value;
    }

    fn set_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    fn store_safe_return_position(&mut self, x: u16, y: u16) {
        self.safe_return_x = x;
        self.safe_return_y = y;
    }

    fn store_safe_return_y(&mut self, y: u16) {
        self.safe_return_y = y;
    }

    fn set_safe_return_y_low(&mut self, value: u8) {
        self.safe_return_y = (self.safe_return_y & 0xff00) | u16::from(value);
    }

    fn store_safe_return_low_from_current(&mut self) {
        self.safe_return_y = (self.safe_return_y & 0xff00) | u16::from(self.y_low());
        self.safe_return_x = (self.safe_return_x & 0xff00) | u16::from(self.x_low());
    }

    fn cache_safe_return_position_from_current(&mut self) {
        self.store_safe_return_position(self.x, self.y);
    }

    fn cache_safe_return_high_from_current(&mut self) {
        self.safe_return_x = (self.safe_return_x & 0x00ff) | (self.x & 0xff00);
        self.safe_return_y = (self.safe_return_y & 0x00ff) | (self.y & 0xff00);
    }

    fn clear_page_movement_deltas(&mut self) {
        self.x_page_movement_delta = 0;
        self.y_page_movement_delta = 0;
    }

    fn set_page_movement_deltas(&mut self, y_delta: u8, x_delta: u8) {
        self.y_page_movement_delta = y_delta;
        self.x_page_movement_delta = x_delta;
    }

    fn set_y_page_movement_delta_from_high_position(&mut self, high: u8) {
        self.y_page_movement_delta = high.wrapping_sub(self.safe_return_y_high());
    }

    fn set_x_page_movement_delta_from_high_position(&mut self, high: u8) {
        self.x_page_movement_delta = high.wrapping_sub(self.safe_return_x_high());
    }

    fn set_position_with_subpixels(&mut self, x: u16, y: u16, x_subpixel: u8, y_subpixel: u8) {
        self.x = x;
        self.y = y;
        self.x_subpixel = x_subpixel;
        self.y_subpixel = y_subpixel;
    }

    fn set_oam_x_offset(&mut self, value: u8) {
        self.oam_x_offset = value;
    }

    fn set_oam_y_offset(&mut self, value: u8) {
        self.oam_y_offset = value;
    }

    fn set_oam_offset(&mut self, y: u8, x: u8) {
        self.oam_y_offset = y;
        self.oam_x_offset = x;
    }

    fn disable_oam_offsets(&mut self) {
        self.set_oam_offset(0x80, 0x80);
    }

    fn set_x_with_subpixel(&mut self, x: u16, x_subpixel: u8) {
        self.x = x;
        self.x_subpixel = x_subpixel;
    }

    fn set_y_with_subpixel(&mut self, y: u16, y_subpixel: u8) {
        self.y = y;
        self.y_subpixel = y_subpixel;
    }

    fn set_y_low(&mut self, value: u8) {
        self.y = (self.y & 0xff00) | u16::from(value);
    }

    fn set_x_low(&mut self, value: u8) {
        self.x = (self.x & 0xff00) | u16::from(value);
    }

    fn set_x_velocity(&mut self, value: u8) {
        self.x_velocity = value;
    }

    fn set_y_velocity(&mut self, value: u8) {
        self.y_velocity = value;
    }

    fn set_movement_velocity_from_delta(&mut self, x_delta: u16, y_delta: u16) {
        self.x_velocity = x_delta as u8;
        self.y_velocity = y_delta as u8;
    }

    fn subtract_axis_velocity_delta(&mut self, horizontal: bool, delta: u8) {
        if horizontal {
            self.x_velocity = self.x_velocity.wrapping_sub(delta);
        } else {
            self.y_velocity = self.y_velocity.wrapping_sub(delta);
        }
    }

    fn add_movement_velocity_delta(&mut self, x_delta: u16, y_delta: u16) {
        self.x_velocity = self.x_velocity.wrapping_add(x_delta as u8);
        self.y_velocity = self.y_velocity.wrapping_add(y_delta as u8);
    }

    fn add_y_velocity_delta(&mut self, y_delta: u8) {
        self.y_velocity = self.y_velocity.wrapping_add(y_delta);
    }

    fn clear_movement_velocity(&mut self) {
        self.x_velocity = 0;
        self.y_velocity = 0;
    }

    fn clear_movement_subpixels(&mut self) {
        self.x_subpixel = 0;
        self.y_subpixel = 0;
    }

    fn set_actual_x_velocity(&mut self, value: u8) {
        self.actual_x_velocity = value;
    }

    fn set_actual_y_velocity(&mut self, value: u8) {
        self.actual_y_velocity = value;
    }

    fn clear_actual_x_velocity(&mut self) {
        self.actual_x_velocity = 0;
    }

    fn clear_actual_y_velocity(&mut self) {
        self.actual_y_velocity = 0;
    }

    fn set_actual_velocity_xy(&mut self, x: u8, y: u8) {
        self.actual_x_velocity = x;
        self.actual_y_velocity = y;
    }

    fn clear_actual_velocity_xy(&mut self) {
        self.set_actual_velocity_xy(0, 0);
    }

    fn invert_actual_velocity_xy(&mut self) {
        self.actual_x_velocity = (-(self.actual_x_velocity as i8)) as u8;
        self.actual_y_velocity = (-(self.actual_y_velocity as i8)) as u8;
    }

    fn xor_actual_velocity_xy(&mut self, mask: u8) {
        self.actual_x_velocity ^= mask;
        self.actual_y_velocity ^= mask;
    }

    fn set_actual_velocity_from_direction(&mut self, direction: u8, velocity: u8) {
        self.actual_x_velocity = if direction & 0x03 != 0 {
            if direction & 0x02 != 0 {
                0u8.wrapping_sub(velocity)
            } else {
                velocity
            }
        } else {
            0
        };
        self.actual_y_velocity = if direction & 0x0c != 0 {
            if direction & 0x08 != 0 {
                0u8.wrapping_sub(velocity)
            } else {
                velocity
            }
        } else {
            0
        };
    }

    fn set_z(&mut self, value: u16) {
        self.z = value;
    }

    fn set_z_low(&mut self, value: u8) {
        self.z = (self.z & 0xff00) | u16::from(value);
    }

    fn clear_z_high(&mut self) {
        self.z &= 0x00ff;
    }

    fn set_z_mirror(&mut self, value: u16) {
        self.z_mirror = value;
    }

    fn restore_z_low_from_mirror(&mut self) {
        self.set_z_low(self.z_mirror_low());
    }

    fn restore_z_from_mirror(&mut self) {
        self.z = self.z_mirror;
    }

    fn cache_z_low_to_mirror(&mut self) {
        self.z_mirror = (self.z_mirror & 0xff00) | u16::from(self.z_low());
    }

    fn cache_z_to_mirror(&mut self) {
        self.z_mirror = self.z;
    }

    fn clear_z_mirror_low(&mut self) {
        self.z_mirror &= 0xff00;
    }

    fn clear_z_mirror_word_low(&mut self) {
        self.clear_z_mirror_low();
    }

    fn force_z_mirror_low_ff(&mut self) {
        self.z_mirror |= 0x00ff;
    }

    fn set_z_and_mirror(&mut self, value: u16) {
        self.z = value;
        self.z_mirror = value;
    }

    fn set_actual_z_velocity(&mut self, value: u8) {
        self.z_velocity = value;
    }

    fn set_actual_z_velocity_and_copy(&mut self, value: u8) {
        self.z_velocity = value;
        self.z_velocity_copy = value;
    }

    fn set_actual_z_velocity_mirror_and_copy(&mut self, value: u8) {
        self.z_velocity_mirror = value;
        self.z_velocity_copy_mirror = value;
    }

    fn restore_actual_z_velocity_from_mirror(&mut self) {
        self.z_velocity = self.z_velocity_mirror;
        self.z_velocity_copy = self.z_velocity_copy_mirror;
    }

    fn cache_actual_z_velocity_to_mirror(&mut self) {
        self.z_velocity_mirror = self.z_velocity;
        self.z_velocity_copy_mirror = self.z_velocity_copy;
    }

    fn prime_airborne_z_velocity(&mut self) {
        self.z_velocity = 0xff;
        self.z = 0xffff;
    }

    fn decrement_actual_z_velocity(&mut self, delta: u8) {
        self.z_velocity = self.z_velocity.wrapping_sub(delta);
    }

    fn set_ground_state(&mut self) {
        self.handler_state = PLAYER_HANDLER_STATE_GROUND;
    }

    fn clear_running(&mut self) {
        self.running = 0;
    }

    fn start_running(&mut self) {
        self.running = 1;
    }

    fn set_running_state(&mut self, value: u8) {
        self.running = value;
    }

    fn cancel_dash_state(&mut self) {
        self.dash_countdown = 0;
        self.speed_setting = 0;
        self.running = 0;
        self.direction_lock = 0;
    }

    fn immobilize(&mut self) {
        self.immobilized = 1;
    }

    fn clear_immobilized(&mut self) {
        self.immobilized = 0;
    }

    fn set_button_mask_b_y(&mut self, value: u8) {
        self.button_mask_b_y = value;
    }

    fn add_button_mask_b_y_bits(&mut self, bits: u8) {
        self.button_mask_b_y |= bits;
    }

    fn set_pull_action_state(&mut self, value: u8) {
        self.pull_action_state = value;
    }

    fn clear_button_mask_b_y_bits(&mut self, mask: u8) {
        self.button_mask_b_y &= !mask;
    }

    fn set_filtered_joypad_h(&mut self, value: u8) {
        self.filtered_joypad_h = value;
    }

    fn set_filtered_joypad_l(&mut self, value: u8) {
        self.filtered_joypad_l = value;
    }

    fn clear_filtered_joypad_l_bits(&mut self, bits: u8) {
        self.filtered_joypad_l &= !bits;
    }

    fn set_joypad1h_last(&mut self, value: u8) {
        self.joypad1h_last = value;
    }

    fn set_joypad1l_last(&mut self, value: u8) {
        self.joypad1l_last = value;
    }

    fn set_joypad1h_last2(&mut self, value: u8) {
        self.joypad1h_last2 = value;
    }

    fn set_joypad1l_last2(&mut self, value: u8) {
        self.joypad1l_last2 = value;
    }

    fn set_spin_attack_delay_timer(&mut self, value: u8) {
        self.spin_attack_delay_timer = value;
    }

    fn decrement_spin_attack_delay_timer(&mut self) -> u8 {
        self.spin_attack_delay_timer = self.spin_attack_delay_timer.wrapping_sub(1);
        self.spin_attack_delay_timer
    }

    fn set_incapacitated_timer(&mut self, value: u8) {
        self.incapacitated_timer = value;
    }

    fn decrement_incapacitated_timer(&mut self) -> u8 {
        self.incapacitated_timer = self.incapacitated_timer.wrapping_sub(1);
        self.incapacitated_timer
    }

    fn reset_elapsed_incapacitated_timer(&mut self) {
        if self.incapacitated_timer == 0 {
            self.incapacitated_timer = 1;
        }
    }

    fn set_recoil_timer(&mut self, value: u8) {
        self.recoil_timer = value;
    }

    fn increment_recoil_timer(&mut self) -> u8 {
        self.recoil_timer = self.recoil_timer.wrapping_add(1);
        self.recoil_timer
    }

    fn set_visibility_status(&mut self, value: u8) {
        self.visibility_status = value;
    }

    fn set_y_button_action_flags(&mut self, value: u8) {
        self.y_button_action_flags = value;
    }

    fn add_y_button_action_flag_bits(&mut self, bits: u8) {
        self.y_button_action_flags |= bits;
    }

    fn clear_y_button_action_flags(&mut self) {
        self.y_button_action_flags = 0;
    }

    fn set_y_button_action_step(&mut self, value: u8) {
        self.y_button_action_step = value;
    }

    fn clear_y_button_action_step(&mut self) {
        self.y_button_action_step = 0;
    }

    fn set_y_button_action_timer(&mut self, value: u8) {
        self.y_button_action_timer = value;
    }

    fn decrement_y_button_action_timer(&mut self) -> u8 {
        self.y_button_action_timer = self.y_button_action_timer.wrapping_sub(1);
        self.y_button_action_timer
    }

    fn clear_defense_flags(&mut self) {
        self.defense_flags = 0;
    }

    fn reset_swim_subpixel_and_defense_state(&mut self) {
        self.clear_movement_subpixels();
        self.moving_against_diag_tile = 0;
        self.defense_flags = 0;
    }

    fn set_defense_flags(&mut self, value: u8) {
        self.defense_flags = value;
    }

    fn or_defense_flags(&mut self, value: u8) {
        self.defense_flags |= value;
    }

    fn and_defense_flags(&mut self, value: u8) {
        self.defense_flags &= value;
    }

    fn set_item_receipt_method(&mut self, value: u8) {
        self.item_receipt_method = value;
    }

    fn set_tile_below(&mut self, value: u8) {
        self.tile_below = value;
    }

    fn set_tile_action_index(&mut self, value: u8) {
        self.tile_action_index = value;
    }

    fn set_tile_coll_flag(&mut self, value: u8) {
        self.tile_collision_flag = value;
    }

    fn clear_tile_coll_flag(&mut self) {
        self.tile_collision_flag = 0;
    }

    fn set_force_move_any_direction(&mut self, value: u16) {
        self.force_move_any_direction = value;
    }

    fn clear_conveyor_belt_state(&mut self) {
        self.conveyor_belt_state = 0;
    }

    fn set_conveyor_belt_state(&mut self, value: u8) {
        self.conveyor_belt_state = value;
    }

    fn clear_faint_animation_active(&mut self) {
        self.faint_animation_active = 0;
    }

    fn set_faint_animation_active(&mut self, value: u8) {
        self.faint_animation_active = value;
    }

    fn clear_item_debug_value_1(&mut self) {
        self.item_debug_value_1 = 0;
    }

    fn clear_hookshot_grave_latch(&mut self) {
        self.hookshot_grave_latch = 0;
    }

    fn set_hookshot_grave_latch(&mut self) {
        self.hookshot_grave_latch = 1;
    }

    fn set_dash_noise_request(&mut self) {
        self.dash_noise_requested = 1;
    }

    fn clear_dash_noise_request(&mut self) {
        self.dash_noise_requested = 0;
    }

    fn tick_jump_ledge_timer_or_reset(&mut self) -> bool {
        self.jump_ledge_timer = self.jump_ledge_timer.wrapping_sub(1);
        if (self.jump_ledge_timer as i8).is_negative() {
            self.jump_ledge_timer = 19;
            true
        } else {
            false
        }
    }

    fn reset_jump_ledge_timer(&mut self) {
        self.jump_ledge_timer = 19;
    }

    fn clear_about_to_jump_off_ledge(&mut self) {
        self.about_to_jump_off_ledge = 0;
    }

    fn increment_about_to_jump_off_ledge(&mut self) {
        self.about_to_jump_off_ledge = self.about_to_jump_off_ledge.wrapping_add(1);
    }

    fn decrement_push_fatigue_timer(&mut self) -> u8 {
        self.push_fatigue_timer = self.push_fatigue_timer.wrapping_sub(1);
        self.push_fatigue_timer
    }

    fn set_push_fatigue_timer(&mut self, value: u8) {
        self.push_fatigue_timer = value;
    }

    fn reset_push_fatigue_timer(&mut self) {
        self.push_fatigue_timer = 32;
    }

    fn clear_near_moveable_statue(&mut self) {
        self.near_moveable_statue_flag = 0;
    }

    fn mark_near_moveable_statue(&mut self) {
        self.near_moveable_statue_flag = 1;
    }

    fn clear_pull_for_rupees_sprite_need(&mut self) {
        self.pull_for_rupees_sprite_needed = 0;
    }

    fn set_pull_for_rupees_sprite_need(&mut self) {
        self.pull_for_rupees_sprite_needed = 1;
    }

    fn clear_pit_correction(&mut self) {
        self.pit_correction_active = 0;
    }

    fn set_pit_correction_active(&mut self) {
        self.pit_correction_active = 1;
    }

    fn set_pit_correction_timer(&mut self, value: u8) {
        self.pit_correction_timer = value;
    }

    fn increment_pit_correction_timer(&mut self) {
        self.pit_correction_timer = self.pit_correction_timer.wrapping_add(1);
    }

    fn set_moving_against_diag_deadlocked(&mut self, value: u8) {
        self.moving_against_diag_deadlocked = value;
    }

    fn clear_misc_bugfix_movement_state(&mut self) {
        self.clear_about_to_jump_off_ledge();
        self.clear_near_moveable_statue();
        self.clear_conveyor_belt_state();
        self.clear_flag_moving();
    }

    fn clear_electrocute_on_touch(&mut self) {
        self.electrocute_on_touch = 0;
    }

    fn set_electrocute_on_touch(&mut self, value: u8) {
        self.electrocute_on_touch = value;
    }

    fn clear_cape_mode(&mut self) {
        self.cape_mode = 0;
    }

    fn set_cape_mode(&mut self, value: u8) {
        self.cape_mode = value;
    }

    fn set_cape_decrement_counter(&mut self, value: u8) {
        self.cape_decrement_counter = value;
    }

    fn decrement_cape_decrement_counter(&mut self) {
        self.cape_decrement_counter = self.cape_decrement_counter.wrapping_sub(1);
    }

    fn clear_transforming(&mut self) {
        self.transforming = 0;
    }

    fn set_transforming(&mut self) {
        self.transforming = 1;
    }

    fn clear_sword_delay_timer(&mut self) {
        self.sword_delay_timer = 0;
    }

    fn set_sword_delay_timer(&mut self, value: u8) {
        self.sword_delay_timer = value;
    }

    fn decrement_sword_delay_timer(&mut self) -> u8 {
        self.sword_delay_timer = self.sword_delay_timer.wrapping_sub(1);
        self.sword_delay_timer
    }

    fn set_spin_offsets(&mut self, value: u8) {
        self.spin_offsets = value;
    }

    fn clear_somaria_platform_state(&mut self) {
        self.somaria_platform_state = 0;
    }

    fn set_somaria_platform_state(&mut self, value: u8) {
        self.somaria_platform_state = value;
    }

    fn clear_spin_attack_step_counter(&mut self) {
        self.spin_attack_step_counter = 0;
    }

    fn increment_spin_attack_step_counter(&mut self) -> u8 {
        self.spin_attack_step_counter = self.spin_attack_step_counter.wrapping_add(1);
        self.spin_attack_step_counter
    }

    fn set_spin_attack_sound_latch(&mut self, value: u8) {
        self.spin_attack_sound_latch = value;
    }

    fn clear_spin_attack_sound_latch(&mut self) {
        self.spin_attack_sound_latch = 0;
    }

    fn set_state_for_spin_attack(&mut self, value: u8) {
        self.spin_attack_state = value;
    }

    fn clear_state_for_spin_attack(&mut self) {
        self.spin_attack_state = 0;
    }

    fn increment_immobilized_flag(&mut self) -> u8 {
        self.immobilized = self.immobilized.wrapping_add(1);
        self.immobilized
    }

    fn set_immobilized_flag(&mut self, value: u8) {
        self.immobilized = value;
    }

    fn reset_incapacitated_camera_timer_from_incapacitated(&mut self) {
        self.incapacitated_camera_timer = self.incapacitated_timer >> 4;
    }

    fn clear_action_handler_timer(&mut self) {
        self.action_handler_timer = 0;
    }

    fn set_action_handler_timer(&mut self, value: u8) {
        self.action_handler_timer = value;
    }

    fn increment_action_handler_timer(&mut self) -> u8 {
        self.action_handler_timer = self.action_handler_timer.wrapping_add(1);
        self.action_handler_timer
    }

    fn clear_doorway_state(&mut self) {
        self.doorway_state = 0;
    }

    fn set_doorway_state(&mut self, value: u8) {
        self.doorway_state = value;
    }

    fn clear_blink_countdown(&mut self) {
        self.blink_countdown = 0;
    }

    fn set_blink_countdown(&mut self, value: u8) {
        self.blink_countdown = value;
    }

    fn decrement_blink_countdown(&mut self) -> u8 {
        self.blink_countdown = self.blink_countdown.wrapping_sub(1);
        self.blink_countdown
    }

    fn set_cape_transform_timer(&mut self, value: u8) {
        self.bunny_transform_timer = value;
    }

    fn tick_cape_transform_timer(&mut self) -> u8 {
        self.bunny_transform_timer = self.bunny_transform_timer.wrapping_sub(1);
        self.bunny_transform_timer
    }

    fn clear_cape_transform_timer(&mut self) {
        self.bunny_transform_timer = 0;
    }

    fn clear_bunny_mirror(&mut self) {
        self.bunny_mirror = 0;
    }

    fn clear_bunny_body_state(&mut self) {
        self.bunny_state = 0;
    }

    fn set_bunny_state(&mut self, value: u8) {
        self.bunny_state = value;
        self.bunny_mirror = value;
    }

    fn clear_bunny_transform_flags(&mut self) {
        self.transform_poof_needed = 0;
        self.bunny_state = 0;
        self.bunny_mirror = 0;
    }

    fn clear_bunny_transform_after_moon_pearl(&mut self) {
        self.clear_bunny_transform_flags();
        self.temp_bunny_timer &= 0xff00;
    }

    fn clear_transform_poof_need_and_temp_bunny_timer(&mut self) {
        self.transform_poof_needed = 0;
        self.temp_bunny_timer = 0;
    }

    fn clear_temp_bunny_timer(&mut self) {
        self.temp_bunny_timer = 0;
    }

    fn set_temp_bunny_timer(&mut self, value: u16) {
        self.temp_bunny_timer = value;
    }

    fn decrement_temp_bunny_timer(&mut self) -> u16 {
        self.temp_bunny_timer = self.temp_bunny_timer.wrapping_sub(1);
        self.temp_bunny_timer
    }

    fn set_spin_animation_step_counter(&mut self, value: u8) {
        self.spin_animation_step_counter = value;
    }

    fn increment_spin_animation_step_counter(&mut self) -> u8 {
        self.spin_animation_step_counter = self.spin_animation_step_counter.wrapping_add(1);
        self.spin_animation_step_counter
    }

    fn clear_spin_animation_step_counter(&mut self) {
        self.spin_animation_step_counter = 0;
    }

    fn clear_button_b_frames(&mut self) {
        self.button_b_frames = 0;
    }

    fn set_button_b_frames(&mut self, value: u8) {
        self.button_b_frames = value;
    }

    fn set_button_b_frames_word(&mut self, value: u16) {
        // Word access spans the independently-owned spin-attack delay timer (0x3d).
        self.button_b_frames = value as u8;
        self.spin_attack_delay_timer = (value >> 8) as u8;
    }

    fn increment_button_b_frames(&mut self) -> u8 {
        let value = self.button_b_frames().wrapping_add(1);
        self.set_button_b_frames(value);
        value
    }

    fn decrement_button_b_frames_word(&mut self) -> u16 {
        let value = self.button_b_frames_word().wrapping_sub(1);
        self.set_button_b_frames_word(value);
        value
    }

    fn clear_animation_step(&mut self) {
        self.animation_step = 0;
    }

    fn set_animation_step(&mut self, value: u8) {
        self.animation_step = value;
    }

    fn increment_opening_pose(&mut self) {
        self.opening_pose = self.opening_pose.wrapping_add(1);
    }

    fn advance_animation_step(&mut self, wrap_at: u8, wrap_to: u8) {
        self.animation_step = self.animation_step.wrapping_add(1);
        if self.animation_step == wrap_at {
            self.animation_step = wrap_to;
        }
    }

    fn advance_animation_step_at_least(&mut self, wrap_at: u8, wrap_to: u8) {
        self.animation_step = self.animation_step.wrapping_add(1);
        if self.animation_step >= wrap_at {
            self.animation_step = wrap_to;
        }
    }

    fn clear_animation_step_if_at_least(&mut self, threshold: u8) {
        if self.animation_step >= threshold {
            self.clear_animation_step();
        }
    }

    fn subtract_animation_step_if_at_least(&mut self, threshold: u8, delta: u8) {
        if self.animation_step >= threshold {
            self.animation_step = self.animation_step.wrapping_sub(delta);
        }
    }

    fn clear_water_ripple_or_grass_state(&mut self) {
        self.water_ripple_or_grass_state = 0;
    }

    fn set_water_ripple_or_grass_state(&mut self, value: u8) {
        self.water_ripple_or_grass_state = value;
    }

    fn set_secondary_water_grass_timer(&mut self, value: u8) {
        self.secondary_water_grass_timer = value;
    }

    fn clear_swim_fast_state(&mut self) {
        self.swim_fast_state = 0;
    }

    fn reset_swimming_state_fields(&mut self) {
        self.swimming_countdown = 0;
        self.hard_swim_stroke = 0;
        self.swim_fast_state = 0;
    }

    fn start_hard_swim_stroke(&mut self, hard_stroke: u8) {
        self.hard_swim_stroke = hard_stroke;
        self.swim_fast_state = 1;
        self.swimming_countdown = 7;
    }

    fn tick_hard_swim_stroke(&mut self, swimming_countdown: u8) {
        self.swimming_countdown = swimming_countdown;
        if (swimming_countdown as i8).is_negative() {
            self.swimming_countdown = 7;
            self.swim_fast_state = self.swim_fast_state.wrapping_add(1);
            if self.swim_fast_state == 5 {
                self.swim_fast_state = 0;
                self.hard_swim_stroke &= !0xc0;
            }
        }
    }

    fn increment_water_ripple_or_grass_state(&mut self) -> u8 {
        self.water_ripple_or_grass_state = self.water_ripple_or_grass_state.wrapping_add(1);
        self.water_ripple_or_grass_state
    }

    fn set_item_pickup_in_progress(&mut self, value: u8) {
        self.item_pickup_in_progress = value;
    }

    fn set_hookshot_bg_check_off_timer(&mut self, value: u8) {
        self.hookshot_bg_check_off_timer = value;
    }

    fn decrement_hookshot_bg_check_off_timer(&mut self) {
        self.hookshot_bg_check_off_timer = self.hookshot_bg_check_off_timer.wrapping_sub(1);
    }

    fn set_selected_rod(&mut self, value: u8) {
        self.selected_rod = value;
    }

    fn set_flute_countdown(&mut self, value: u8) {
        self.flute_countdown = value;
    }

    fn decrement_flute_countdown(&mut self) {
        self.flute_countdown = self.flute_countdown.wrapping_sub(1);
    }

    fn clear_flute_countdown(&mut self) {
        self.flute_countdown = 0;
    }

    fn clear_index_of_dashing_sfx(&mut self) {
        self.index_of_dashing_sfx = 0;
    }

    fn decrement_index_of_dashing_sfx(&mut self) {
        self.index_of_dashing_sfx = self.index_of_dashing_sfx.wrapping_sub(1);
    }

    fn set_deep_water_state(&mut self, value: u8) {
        self.deep_water_state = value;
    }

    fn enter_deep_water_state(&mut self) {
        self.deep_water_state = 1;
    }

    fn clear_deep_water_state(&mut self) {
        self.deep_water_state = 0;
    }

    fn clear_item_action_step_var(&mut self) {
        self.item_action_step = 0;
    }

    fn set_throw_oam_state_index(&mut self, value: u8) {
        self.throw_oam_state_index = value;
    }

    fn clear_throw_oam_state_index(&mut self) {
        self.throw_oam_state_index = 0;
    }

    fn increment_item_action_step_var(&mut self) -> u8 {
        self.item_action_step = self.item_action_step.wrapping_add(1);
        self.item_action_step
    }

    fn advance_item_action_step_var_wrapping_7_to_1(&mut self) -> u8 {
        self.item_action_step = if self.item_action_step.wrapping_add(1) == 7 {
            1
        } else {
            self.item_action_step.wrapping_add(1)
        };
        self.item_action_step
    }

    fn clear_given_damage(&mut self) {
        self.given_damage = 0;
    }

    fn set_given_damage(&mut self, value: u8) {
        self.given_damage = value;
    }

    fn set_item_in_hand(&mut self, value: u8) {
        self.item_in_hand = value;
    }

    fn clear_item_in_hand(&mut self) {
        self.item_in_hand = 0;
    }

    fn clear_item_in_hand_bits(&mut self, mask: u8) {
        self.item_in_hand &= !mask;
    }

    fn clear_position_mode(&mut self) {
        self.position_mode = 0;
    }

    fn set_position_mode(&mut self, value: u8) {
        self.position_mode = value;
    }

    fn set_position_mode_bits(&mut self, mask: u8) {
        self.position_mode |= mask;
    }

    fn clear_position_mode_bits(&mut self, mask: u8) {
        self.position_mode &= !mask;
    }

    fn set_item_action_step_var(&mut self, value: u8) {
        self.item_action_step = value;
    }

    fn set_item_action_debug_value_2(&mut self, value: u8) {
        self.item_action_debug_value_2 = value;
    }

    fn clear_item_action_debug_value_2(&mut self) {
        self.item_action_debug_value_2 = 0;
    }

    fn set_current_item_y(&mut self, value: u8) {
        self.current_item_y = value;
    }

    fn set_current_item_active(&mut self, value: u8) {
        self.current_item_active = value;
    }

    fn set_receive_item_index(&mut self, value: u8) {
        self.receive_item_index = value;
    }

    fn spend_magic(&mut self, cost: u8) -> bool {
        let new_magic = self.magic_power.wrapping_sub(cost);
        if self.magic_power != 0 && new_magic < 0x80 {
            self.magic_power = new_magic;
            true
        } else {
            false
        }
    }

    fn refund_magic(&mut self, cost: u8, clamp_full: bool) {
        let mut new_magic = self.magic_power as u16 + cost as u16;
        if clamp_full && new_magic >= 128 {
            new_magic = 128;
        }
        self.magic_power = new_magic as u8;
    }

    fn decrement_magic_power(&mut self) -> u8 {
        self.magic_power = self.magic_power.wrapping_sub(1);
        self.magic_power
    }

    fn set_magic_power(&mut self, value: u8) {
        self.magic_power = value;
    }

    fn increment_magic_power(&mut self) {
        self.magic_power = self.magic_power.wrapping_add(1);
    }

    fn clear_action_scratch_state(&mut self) {
        self.item_action_debug_value_2 = 0;
        self.item_action_step = 0;
        self.throw_oam_state_index = 0;
    }

    fn clear_lift_throw_scratch_state(&mut self) {
        self.item_action_step = 0;
        self.throw_oam_state_index = 0;
    }

    fn clear_ancilla_pickup_flag(&mut self) {
        self.ancilla_pickup_flag = 0;
    }

    fn set_ancilla_pickup_flag(&mut self, value: u8) {
        self.ancilla_pickup_flag = value;
    }

    fn clear_sprite_pickup_flag(&mut self) {
        self.sprite_pickup_flag = 0;
    }

    fn set_sprite_pickup_flag(&mut self, value: u8) {
        self.sprite_pickup_flag = value;
    }

    fn set_hookshot_interlock(&mut self, value: u8) {
        self.hookshot_interlock = value;
    }

    fn clear_hookshot_interlock(&mut self) {
        self.hookshot_interlock = 0;
    }

    fn xor_hookshot_interlock(&mut self, mask: u8) {
        self.hookshot_interlock ^= mask;
    }

    fn clear_grabbing_wall(&mut self) {
        self.grabbing_wall = 0;
    }

    fn set_grabbing_wall(&mut self, value: u8) {
        self.grabbing_wall = value;
    }

    fn enable_cutscene_immunity(&mut self) {
        self.sprite_damage_disabled = 1;
    }

    fn set_sprite_damage_disable_timer(&mut self, value: u8) {
        self.sprite_damage_disabled = value;
    }

    fn clear_sprite_damage_disable_timer(&mut self) {
        self.sprite_damage_disabled = 0;
    }

    fn increment_sprite_damage_disable_timer(&mut self) {
        self.sprite_damage_disabled = self.sprite_damage_disabled.wrapping_add(1);
    }

    fn set_item_hold_pose(&mut self, value: u8) {
        self.item_hold_pose = value;
    }

    fn clear_item_hold_pose(&mut self) {
        self.item_hold_pose = 0;
    }

    fn force_hold_sword_up(&mut self) {
        self.force_hold_sword_up = 1;
    }

    fn clear_force_hold_sword_up(&mut self) {
        self.force_hold_sword_up = 0;
    }

    fn set_near_pit_state(&mut self, value: u8) {
        self.near_pit_state = value;
    }

    fn clear_near_pit_state(&mut self) {
        self.near_pit_state = 0;
    }

    fn set_pit_data_index(&mut self, value: u8) {
        self.pit_data_index = value;
    }

    fn clear_pit_data_index(&mut self) {
        self.pit_data_index = 0;
    }

    fn advance_pit_data_index(&mut self) -> u8 {
        self.pit_data_index = self.pit_data_index.wrapping_add(1);
        self.pit_data_index
    }

    fn begin_pit_check(&mut self) {
        self.clear_pit_data_index();
        self.set_near_pit_state(1);
    }

    fn clear_pit_state(&mut self) {
        self.clear_pit_data_index();
        self.clear_near_pit_state();
    }

    fn start_bunny_transform_poof(&mut self) {
        self.sprite_damage_disabled = 1;
        self.transform_poof_needed = 1;
        self.visibility_status = 12;
    }

    fn finish_bunny_transform_poof(&mut self) {
        self.bunny_mirror = 1;
        self.bunny_state = 1;
        self.visibility_status = 0;
        self.sprite_damage_disabled = 0;
        self.transform_poof_needed = 0;
    }

    fn set_auxiliary_state(&mut self, value: u8) {
        self.auxiliary_state = value;
    }

    fn clear_auxiliary_state(&mut self) {
        self.auxiliary_state = 0;
    }

    fn set_state_bits(&mut self, value: u8) {
        self.action_state_bits = value;
    }

    fn clear_state_bits(&mut self) {
        self.action_state_bits = 0;
    }

    fn clear_lifting_or_carrying_state(&mut self) {
        self.action_state_bits &= !0x80;
    }

    fn keep_only_lifting_or_carrying_state(&mut self) {
        self.action_state_bits &= 0x80;
    }

    fn enter_item_hold_pose(&mut self) {
        self.action_state_bits = 0x80;
        self.picking_throw_state = 0;
        self.facing = 0;
        self.animation_step = 0;
    }

    fn clear_state_item_and_grab_flags(&mut self) {
        self.action_state_bits = 0;
        self.picking_throw_state = 0;
        self.grabbing_wall = 0;
    }

    fn clear_picking_throw_state(&mut self) {
        self.picking_throw_state = 0;
    }

    fn set_picking_throw_state(&mut self, value: u8) {
        self.picking_throw_state = value;
    }

    fn start_lift_throw_state(&mut self) {
        self.picking_throw_state = 1;
        self.action_state_bits = 0x80;
    }

    fn clear_swimming_action_state(&mut self) {
        self.button_mask_b_y = 0;
        self.clear_button_b_frames();
        self.spin_attack_delay_timer = 0;
        self.spin_attack_step_counter = 0;
        self.action_state_bits = 0;
        self.picking_throw_state = 0;
    }

    fn initialize_link_action_state(&mut self) {
        self.facing = 2;
        self.last_direction = 0;
        self.item_in_hand = 0;
        self.position_mode = 0;
        self.item_debug_value_1 = 0;
        self.item_action_debug_value_2 = 0;
        self.item_action_step = 0;
        self.throw_oam_state_index = 0;
        self.y_button_action_step = 0;
        self.transforming = 0;
        self.y_button_action_flags = 0;
        self.button_mask_b_y &= !0x40;
        self.action_state_bits = 0;
        self.picking_throw_state = 0;
        self.grabbing_wall = 0;
    }

    fn reset_properties_c_fields(&mut self) {
        self.tile_action_index = 0;
        self.spin_animation_step_counter = 0;
        self.spin_attack_state = 0;
        self.tile_collision_flag = 0;
        self.force_hold_sword_up = 0;
        self.sword_delay_timer = 0;
        self.item_in_hand = 0;
        self.position_mode = 0;
        self.item_debug_value_1 = 0;
        self.item_action_debug_value_2 = 0;
        self.item_action_step = 0;
        self.throw_oam_state_index = 0;
        self.y_button_action_step = 0;
        self.y_button_action_flags = 0;
        self.button_mask_b_y = 0;
        self.clear_button_b_frames();
        self.action_state_bits = 0;
        self.picking_throw_state = 0;
        self.grabbing_wall = 0;
        self.direction_lock = 0;
        self.auxiliary_state = 0;
        self.incapacitated_timer = 0;
        self.defense_flags = 0;
        self.action_handler_timer = 0;
        self.sprite_damage_disabled = 0;
        self.ancilla_pickup_flag = 0;
        self.sprite_pickup_flag = 0;
        self.clear_pull_for_rupees_sprite_need();
        self.clear_near_moveable_statue();
        self.spin_attack_step_counter = 0;
    }

    fn finish_link_action_state_initialization(&mut self) {
        self.direction_lock &= !1;
        self.z &= 0x00ff;
        self.auxiliary_state = 0;
        self.incapacitated_timer = 0;
        self.blink_countdown = 0;
        self.electrocute_on_touch = 0;
        self.item_hold_pose = 0;
        self.cape_mode = 0;
        self.sprite_damage_disabled = 0;
        self.action_handler_timer = 0;
        self.direction &= !0x0f;
        self.somaria_platform_state = 0;
        self.spin_attack_step_counter = 0;
    }

    fn reset_properties_a_fields(&mut self) {
        self.last_direction = 0;
        self.direction = 0;
        self.movement_flag = 0;
        self.blink_countdown = 0;
        self.transforming = 0;
        self.bunny_state = 0;
        self.bunny_mirror = 0;
        self.temp_bunny_timer = 0;
        self.transform_poof_needed = 0;
        self.clear_pull_for_rupees_sprite_need();
        self.hookshot_grave_latch = 0;
        self.given_damage = 0;
        self.spin_offsets = 0;
        self.dash_noise_requested = 0;
        self.item_receipt_method = 0;
        self.bit9_of_xcoord = 0;
        self.whirlpool_trigger = 0;
    }

    fn cache_current_quadrants(&mut self) {
        self.cached_quadrant_x = self.quadrant_x;
        self.cached_quadrant_y = self.quadrant_y;
    }

    fn restore_quadrants_from_cached(&mut self) {
        self.quadrant_x = self.cached_quadrant_x;
        self.quadrant_y = self.cached_quadrant_y;
    }

    fn advance_frame_change_counter(&mut self, delay: u8) -> bool {
        self.frame_change_counter = self.frame_change_counter.wrapping_add(1);
        if self.frame_change_counter >= delay {
            self.frame_change_counter = 0;
            true
        } else {
            false
        }
    }

    fn clear_frame_change_counter(&mut self) {
        self.frame_change_counter = 0;
    }

    fn set_sprite_oam_state_timer(&mut self, value: u8) {
        self.sprite_oam_state_timer = value;
    }

    fn set_recoil_z_velocity_for_dungeon_reset(&mut self, value: u8) {
        self.recoil_z_velocity_for_dungeon_reset = value;
    }

    fn set_recoil_z_velocity(&mut self, value: u8) {
        self.recoil_z_velocity_for_dungeon_reset = value;
        self.z_velocity = value;
    }

    fn decrement_sprite_oam_state_timer(&mut self) -> u8 {
        self.sprite_oam_state_timer = self.sprite_oam_state_timer.wrapping_sub(1);
        self.sprite_oam_state_timer
    }

    fn mark_pit_landing_oam_state(&mut self) {
        self.sprite_oam_state_timer = 9;
    }

    fn set_whirlpool_trigger(&mut self) {
        self.whirlpool_trigger = 1;
    }

    fn clear_whirlpool_trigger(&mut self) {
        self.whirlpool_trigger = 0;
    }

    fn prevent_movement(&mut self) {
        self.prevent_movement = 1;
    }

    fn clear_prevent_movement(&mut self) {
        self.prevent_movement = 0;
    }

    fn set_swim_stroke_frame_counter(&mut self, offset: usize, value: u16) {
        let axis =
            swim_axis_index(offset).expect("swim stroke frame counter offset must be 0 or 2");
        self.swim_stroke_frame_counters[axis] = value;
    }

    fn clear_magic_spell_player_lock(&mut self) {
        self.magic_spell_player_lock = 0;
    }

    fn clear_somaria_block_bg_check_flag(&mut self) {
        self.somaria_block_bg_check_flag = 0;
    }

    fn clear_player_pose_draw_counter(&mut self) {
        self.player_pose_draw_counter = 0;
    }

    fn increment_player_pose_draw_counter(&mut self) {
        self.player_pose_draw_counter = self.player_pose_draw_counter.wrapping_add(1);
    }

    fn clear_player_special_draw_flag(&mut self) {
        self.player_special_draw_flag = 0;
    }

    fn set_player_special_draw_flag(&mut self, value: u8) {
        self.player_special_draw_flag = value;
    }

    fn set_primary_water_grass_timer(&mut self, value: u8) {
        self.primary_water_grass_timer = value;
    }

    fn set_bit9_of_xcoord_word(&mut self, value: u16) {
        self.bit9_of_xcoord = value;
    }

    fn set_hop_origin_delta_from_y(&mut self, y: u16) -> u16 {
        self.hop_origin_coord = self.hop_origin_coord.wrapping_sub(y);
        self.hop_origin_coord
    }

    fn set_movement_velocity_from_position_delta(
        &mut self,
        x: u16,
        y: u16,
        old_x: u16,
        old_y: u16,
    ) {
        self.y_velocity = y.wrapping_sub(old_y) as u8;
        self.x_velocity = x.wrapping_sub(old_x) as u8;
    }

    fn clear_actual_velocity_and_page_movement_deltas(&mut self) {
        self.actual_x_velocity = 0;
        self.actual_y_velocity = 0;
        self.x_page_movement_delta = 0;
        self.y_page_movement_delta = 0;
    }

    fn cache_moving_floor_position(&mut self, x: u16, y: u16) {
        self.moving_floor_x = x;
        self.moving_floor_y = y;
    }

    fn decrement_incapacitated_camera_timer(&mut self) -> u8 {
        self.incapacitated_camera_timer = self.incapacitated_camera_timer.wrapping_sub(1);
        self.incapacitated_camera_timer
    }

    fn increment_pull_action_state(&mut self) {
        self.pull_action_state = self.pull_action_state.wrapping_add(1);
    }

    fn set_item_holding_timer(&mut self, value: u8) {
        self.item_holding_timer = value;
    }

    fn clear_swim_movement_velocity(&mut self) {
        self.y_velocity = 0;
        self.x_velocity = 0;
    }

    fn increment_sleep_in_bed_state(&mut self) {
        self.sleep_in_bed_state = self.sleep_in_bed_state.wrapping_add(1);
    }

    fn set_cached_tile_action_index(&mut self, value: u8) {
        self.cached_tile_action_index = value;
    }

    fn clear_swimming_countdown(&mut self) {
        self.swimming_countdown = 0;
    }

    fn clear_ancilla_interactive_reset_flag(&mut self) {
        self.ancilla_interactive_reset_flag = 0;
    }

    fn clear_force_move_high_byte(&mut self) {
        self.force_move_any_direction = u16::from(self.force_move_any_direction as u8);
    }

    fn set_sprite_pickup_flag_cached(&mut self, value: u8) {
        self.sprite_pickup_flag_cached = value;
    }

    fn set_link_dma_graphics_index_word(&mut self, value: u16) {
        self.link_dma_graphics_index = value;
    }

    fn set_link_dma_left_sprite_bank_word(&mut self, value: u16) {
        self.link_dma_left_sprite_bank = value;
    }

    fn set_link_dma_right_sprite_bank_word(&mut self, value: u16) {
        self.link_dma_right_sprite_bank = value;
    }

    fn clear_link_dma_sprite_banks(&mut self) {
        self.link_dma_left_sprite_bank = 0;
        self.link_dma_right_sprite_bank = 0;
    }

    fn set_palette_bits_of_oam_word(&mut self, value: u16) {
        self.palette_bits_of_oam = value;
    }

    fn advance_link_dma_source_offset(&mut self) -> u16 {
        self.link_dma_source_offset = self.link_dma_source_offset.wrapping_add(0x400);
        if self.link_dma_source_offset == 0x0c00 {
            self.link_dma_source_offset = 0;
        }
        self.link_dma_source_offset
    }

    fn advance_link_dma_tile_offset(&mut self) -> u16 {
        self.link_dma_tile_offset = self.link_dma_tile_offset.wrapping_add(2);
        if self.link_dma_tile_offset == 12 {
            self.link_dma_tile_offset = 0;
        }
        self.link_dma_tile_offset
    }

    fn set_link_dma_countdown(&mut self, value: u16) {
        self.link_dma_countdown = value;
    }

    fn decrement_link_dma_countdown(&mut self) -> u16 {
        self.link_dma_countdown = self.link_dma_countdown.wrapping_sub(1);
        self.link_dma_countdown
    }

    fn reset_link_dma_animation_cycle(&mut self, countdown: u16) {
        self.link_dma_countdown = countdown;
        self.link_dma_source_offset = 0;
        self.link_dma_tile_offset = 0;
    }

    fn set_sword_dma_graphics_index(&mut self, value: u8) {
        self.sword_dma_graphics_index = value;
    }

    fn set_shield_dma_graphics_index(&mut self, value: u8) {
        self.shield_dma_graphics_index = value;
    }

    fn set_link_dma_staging_index(&mut self, value: u8) {
        self.link_dma_staging_index = value;
    }

    fn set_link_sprite_index_scratch(&mut self, value: u16) {
        self.link_sprite_index_scratch = value;
    }

    fn set_hop_origin_coord(&mut self, value: u16) {
        self.hop_origin_coord = value;
    }

    fn set_previous_position(&mut self, x: u16, y: u16) {
        self.previous_x = x;
        self.previous_y = y;
    }

    fn cache_previous_position_from_current(&mut self) {
        self.previous_x = self.x;
        self.previous_y = self.y;
    }

    fn set_drag_player_x(&mut self, value: u16) {
        self.drag_player_x = value;
    }

    fn set_drag_player_y(&mut self, value: u16) {
        self.drag_player_y = value;
    }

    fn add_drag_player_x(&mut self, delta: u16) {
        self.drag_player_x = self.drag_player_x.wrapping_add(delta);
    }

    fn add_drag_player_y(&mut self, delta: u16) {
        self.drag_player_y = self.drag_player_y.wrapping_add(delta);
    }

    fn set_gravestone_push_timeout(&mut self, value: u8) {
        self.gravestone_push_timeout = value;
    }

    fn decrement_gravestone_push_timeout(&mut self) {
        self.gravestone_push_timeout = self.gravestone_push_timeout.wrapping_sub(1);
    }

    fn reset_properties_b_fields(&mut self) {
        self.somaria_platform_state = 0;
        self.spin_attack_step_counter = 0;
        self.defense_flags = 0;
        self.sprite_pickup_flag_cached = 0;
        self.clear_pit_correction();
        self.pit_data_index = 0;
        self.near_pit_state = 0;
    }

    fn land_after_splash_with_handler(&mut self, handler_state: u8) {
        self.handler_state = handler_state;
    }

    fn enter_water_hop_state(&mut self) {
        if self.auxiliary_state != 2 {
            self.auxiliary_state = 1;
            self.electrocute_on_touch = 0;
        }
        self.handler_state = 6;
    }

    fn interrupt_swimming_for_auxiliary_state(&mut self) {
        self.handler_state = 2;
        self.z &= 0x00ff;
        self.swim_fast_state = 0;
        self.hard_swim_stroke = 0;
        self.direction_lock &= !1;
    }

    fn reset_idle_swim_animation_if_out_of_water(&mut self) {
        if self.handler_state != PLAYER_HANDLER_STATE_SWIMMING {
            self.animation_step = 0;
        }
    }

    fn become_bunny_handler(&mut self) {
        self.handler_state = 23;
        self.bunny_state = 1;
        self.bunny_mirror = 1;
    }

    fn setup_bed_pose(&mut self) {
        self.handler_state = 0x16;
        self.sleep_in_bed_state = 0;
        self.opening_pose = 0;
        self.dash_countdown = 3;
    }

    fn reset_after_damaging_pit(&mut self, handler_state: u8) {
        self.handler_state = handler_state;
        self.last_direction = self.swim_direction_flags;
        self.deep_water_state = 0;
        self.sprite_damage_disabled = 0;
        self.pit_data_index = 0;
        self.near_pit_state = 0;
    }

    fn recache_bunny_state(&mut self, has_moon_pearl: bool) {
        self.transform_poof_needed = 0;
        self.temp_bunny_timer = 0;
        if has_moon_pearl {
            self.bunny_state = 0;
            self.auxiliary_state = 0;
        }
        self.animation_step = 0;
        self.transforming = 0;
        self.direction_lock = 0;
    }

    fn enter_deep_water(&mut self) {
        self.deep_water_state = 1;
        self.swim_direction_flags = self.last_direction;
        self.grabbing_wall = 0;
        self.speed_setting = 0;
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

    pub(crate) fn decrement_speed_setting(&mut self) -> u8 {
        let value = self.state.decrement_speed_setting();
        self.ram[LINK_SPEED_SETTING] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_speed_modifier(&mut self) {
        self.state.clear_speed_modifier();
        self.ram[LINK_SPEED_MODIFIER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_speed_modifier(&mut self, value: u8) {
        self.state.set_speed_modifier(value);
        self.ram[LINK_SPEED_MODIFIER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mark_lower_level(&mut self) {
        self.state.mark_lower_level();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mark_lower_level_mirror(&mut self) {
        self.state.mark_lower_level_mirror();
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_lower_level_state(&mut self, value: u8) {
        self.state.set_lower_level_state(value);
        self.ram[LINK_IS_ON_LOWER_LEVEL] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_lower_level_mirror_state(&mut self, value: u8) {
        self.state.set_lower_level_mirror_state(value);
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_lower_level_states(&mut self, state: u8, mirror: u8) {
        self.state.set_lower_level_states(state, mirror);
        self.ram[LINK_IS_ON_LOWER_LEVEL] = state;
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = mirror;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_lower_level(&mut self) {
        self.state.clear_lower_level();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_lower_level_states(&mut self) {
        self.state.clear_lower_level_states();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn toggle_lower_level_state(&mut self) {
        self.state.toggle_lower_level_state();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = self.state.lower_level_state();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn toggle_lower_level_mirror_state(&mut self) {
        self.state.toggle_lower_level_mirror_state();
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = self.state.lower_level_mirror_state();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mirror_lower_level_state(&mut self) {
        self.state.mirror_lower_level_state();
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = self.state.lower_level_state();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_lower_level_states(&mut self) {
        self.state.cache_lower_level_states();
        self.ram[LINK_IS_ON_LOWER_LEVEL_CACHED] = self.state.cached_lower_level_state();
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED] =
            self.state.cached_lower_level_mirror_state();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_lower_level_state_from_cached(&mut self) {
        self.state.restore_lower_level_state_from_cached();
        self.ram[LINK_IS_ON_LOWER_LEVEL] = self.state.lower_level_state();
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = self.state.lower_level_mirror_state();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn arm_stair_speed_modifier(&mut self) {
        self.state.arm_stair_speed_modifier();
        self.ram[LINK_SPEED_SETTING] = 2;
        self.ram[LINK_SPEED_MODIFIER] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn resolve_dash_speed_setting(&mut self) {
        self.state.resolve_dash_speed_setting();
        self.ram[LINK_SPEED_SETTING] = self.state.speed_setting();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn promote_pending_speed_modifier(&mut self) {
        self.state.promote_pending_speed_modifier();
        self.ram[LINK_SPEED_MODIFIER] = self.state.speed_modifier();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increase_near_pit_speed_modifier(&mut self) {
        self.state.increase_near_pit_speed_modifier();
        self.ram[LINK_SPEED_MODIFIER] = self.state.speed_modifier();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_dash_deceleration(&mut self) {
        self.state.advance_dash_deceleration();
        self.ram[LINK_SPEED_MODIFIER] = self.state.speed_modifier();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_handler_state(&mut self, value: u8) {
        self.state.set_handler_state(value);
        self.ram[LINK_HANDLER_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_handler_state(&mut self) {
        self.state.clear_handler_state();
        self.ram[LINK_HANDLER_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_facing(&mut self, value: u8) {
        self.state.set_facing(value);
        self.ram[LINK_FACING] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_facing_from_cached(&mut self) {
        self.state.restore_facing_from_cached();
        self.ram[LINK_FACING] = self.state.facing();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_facing_mirror(&mut self, value: u8) {
        self.state.set_facing_mirror(value);
        self.ram[LINK_FACING_MIRROR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_facing_to_mirror(&mut self) {
        self.state.cache_facing_to_mirror();
        self.ram[LINK_FACING_MIRROR] = self.state.facing();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_facing(&mut self) {
        self.state.cache_facing();
        self.ram[LINK_FACING_CACHED] = self.state.facing();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_moving_against_diag_tile(&mut self, value: u8) {
        self.state.set_moving_against_diag_tile(value);
        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_moving_against_diag_tile_flags(&mut self, value: u8) {
        self.state.add_moving_against_diag_tile_flags(value);
        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = self.state.moving_against_diag_tile();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_moving_against_diag_tile(&mut self) {
        self.state.clear_moving_against_diag_tile();
        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_flag_moving(&mut self, value: u8) {
        self.state.set_flag_moving(value);
        self.ram[LINK_FLAG_MOVING] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_flag_moving(&mut self) {
        self.state.clear_flag_moving();
        self.ram[LINK_FLAG_MOVING] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_quadrants_from_packed_nibbles(&mut self, value: u8) {
        self.state.set_quadrants_from_packed_nibbles(value);
        self.ram[LINK_QUADRANT_X] = self.state.quadrant_x();
        self.ram[LINK_QUADRANT_Y] = self.state.quadrant_y();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_quadrants(&mut self, x: u8, y: u8) {
        self.state.set_quadrants(x, y);
        self.ram[LINK_QUADRANT_X] = x;
        self.ram[LINK_QUADRANT_Y] = y;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn toggle_quadrant_x(&mut self) -> u8 {
        let value = self.state.toggle_quadrant_x();
        self.ram[LINK_QUADRANT_X] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn toggle_quadrant_y(&mut self) -> u8 {
        let value = self.state.toggle_quadrant_y();
        self.ram[LINK_QUADRANT_Y] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn reset_direction_limits(&mut self) {
        self.state.reset_direction_limits();
        self.ram[LINK_DIRECTION_MASK_A] = 0x0f;
        self.ram[LINK_DIRECTION_MASK_B] = 0x0f;
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_direction_masks(&mut self) {
        self.state.reset_direction_masks();
        self.ram[LINK_DIRECTION_MASK_A] = 0x0f;
        self.ram[LINK_DIRECTION_MASK_B] = 0x0f;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_orthogonal_direction_count(&mut self) {
        self.state.increment_orthogonal_direction_count();
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = self.state.num_orthogonal_directions();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_orthogonal_direction_count(&mut self) {
        self.state.clear_orthogonal_direction_count();
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_last_direction_moved_towards(&mut self, value: u8) {
        self.state.set_last_direction_moved_towards(value);
        self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_last_direction_from_current_direction(&mut self) {
        self.state.set_last_direction_from_current_direction();
        self.ram[LINK_LAST_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_last_direction(&mut self, value: u8) {
        self.state.set_last_direction(value);
        self.ram[LINK_LAST_DIRECTION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mask_last_direction(&mut self, mask: u8) {
        self.state.mask_last_direction(mask);
        self.ram[LINK_LAST_DIRECTION] = self.state.last_direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_last_direction_from_swim_flags(&mut self) {
        self.state.set_last_direction_from_swim_flags();
        self.ram[LINK_LAST_DIRECTION] = self.state.swim_direction_flags();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_swim_flags_from_last_direction(&mut self) {
        self.state.set_swim_flags_from_last_direction();
        self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.state.last_direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction(&mut self, value: u8) {
        self.state.set_direction(value);
        self.ram[LINK_DIRECTION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_and_last_direction(&mut self, value: u8) {
        self.state.set_direction_and_last_direction(value);
        self.ram[LINK_DIRECTION] = value;
        self.ram[LINK_LAST_DIRECTION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_and_swim_flags(&mut self, value: u8) {
        self.state.set_direction_and_swim_flags(value);
        self.ram[LINK_DIRECTION] = value;
        self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mask_direction(&mut self, mask: u8) {
        self.state.mask_direction(mask);
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_cardinal_direction(&mut self) {
        self.state.clear_cardinal_direction();
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_direction_flags(&mut self, flags: u8) {
        self.state.add_direction_flags(flags);
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_direction_flags(&mut self, flags: u8) {
        self.state.clear_direction_flags(flags);
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_lock(&mut self, value: u8) {
        self.state.set_direction_lock(value);
        self.ram[LINK_CANT_CHANGE_DIRECTION] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_direction_lock(&mut self) {
        self.state.clear_direction_lock();
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_lock_bits(&mut self, mask: u8) {
        self.state.set_direction_lock_bits(mask);
        self.ram[LINK_CANT_CHANGE_DIRECTION] = self.state.direction_lock();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_direction_lock_bits(&mut self, mask: u8) {
        self.state.clear_direction_lock_bits(mask);
        self.ram[LINK_CANT_CHANGE_DIRECTION] = self.state.direction_lock();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_mask_a(&mut self, value: u8) {
        self.state.set_direction_mask_a(value);
        self.ram[LINK_DIRECTION_MASK_A] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_direction_mask_b(&mut self, value: u8) {
        self.state.set_direction_mask_b(value);
        self.ram[LINK_DIRECTION_MASK_B] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn apply_direction_masks(&mut self) {
        self.state.apply_direction_masks();
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn force_direction_from_diag_tile_if_needed(&mut self) {
        self.state.force_direction_from_diag_tile_if_needed();
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn resolve_orthogonal_direction_count_from_facing(&mut self) {
        self.state.resolve_orthogonal_direction_count_from_facing();
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = self.state.num_orthogonal_directions();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mark_moving_floor_direction(&mut self, floor_y: u16, floor_x: u16) {
        self.state.mark_moving_floor_direction(floor_y, floor_x);
        self.ram[LINK_DIRECTION] = self.state.direction();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_last_direction_moved_towards_from_facing(&mut self) {
        self.state.set_last_direction_moved_towards_from_facing();
        self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = self.state.last_direction_moved_towards();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_swim_direction_flags(&mut self, direction: u8) {
        self.state.set_swim_direction_flags(direction);
        self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = direction;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.state.set_y(value);
        write_le_u16(self.ram, LINK_Y_COORD, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.state.set_x(value);
        write_le_u16(self.ram, LINK_X_COORD, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_position(x, y);
        write_le_u16(self.ram, LINK_X_COORD, x);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.state.set_y_low(value);
        self.ram[LINK_Y_COORD] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.state.set_x_low(value);
        self.ram[LINK_X_COORD] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_position_from_cached(&mut self) {
        let x = self.state.cached_x;
        let y = self.state.cached_y;
        self.state.set_position(x, y);
        write_le_u16(self.ram, LINK_X_COORD, x);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_current_position(&mut self) {
        self.state.cached_x = self.state.x();
        self.state.cached_y = self.state.y();
        write_le_u16(self.ram, LINK_Y_COORD_CACHED, self.state.y());
        write_le_u16(self.ram, LINK_X_COORD_CACHED, self.state.x());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_copied_position_from_current(&mut self) {
        self.state.copied_x = self.state.x();
        self.state.copied_y = self.state.y();
        write_le_u16(self.ram, LINK_Y_COORD_COPY, self.state.y());
        write_le_u16(self.ram, LINK_X_COORD_COPY, self.state.x());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_y_from_previous_position(&mut self) {
        let y = read_le_u16(self.ram, LINK_Y_COORD_PREV);
        self.state.set_y(y);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_position_from_previous(&mut self) {
        let x = read_le_u16(self.ram, LINK_X_COORD_PREV);
        let y = read_le_u16(self.ram, LINK_Y_COORD_PREV);
        self.state.set_position(x, y);
        write_le_u16(self.ram, LINK_X_COORD, x);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_previous_position_from_current(&mut self) {
        self.state.cache_previous_position_from_current();
        write_le_u16(self.ram, LINK_Y_COORD_PREV, self.state.y());
        write_le_u16(self.ram, LINK_X_COORD_PREV, self.state.x());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_previous_position_from_current_xy_order(&mut self) {
        self.state.cache_previous_position_from_current();
        write_le_u16(self.ram, LINK_X_COORD_PREV, self.state.x());
        write_le_u16(self.ram, LINK_Y_COORD_PREV, self.state.y());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_previous_position(&mut self, x: u16, y: u16) {
        self.state.set_previous_position(x, y);
        write_le_u16(self.ram, LINK_X_COORD_PREV, x);
        write_le_u16(self.ram, LINK_Y_COORD_PREV, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn move_x_by_velocity(&mut self, velocity: u8) -> u16 {
        let x = move_link_axis_by_velocity(self.ram, LINK_X_SUBPIXEL, LINK_X_COORD, velocity);
        self.state.set_x_with_subpixel(x, self.ram[LINK_X_SUBPIXEL]);
        self.debug_assert_matches_ram();
        x
    }

    pub(crate) fn move_y_by_velocity(&mut self, velocity: u8) -> u16 {
        let y = move_link_axis_by_velocity(self.ram, LINK_Y_SUBPIXEL, LINK_Y_COORD, velocity);
        self.state.set_y_with_subpixel(y, self.ram[LINK_Y_SUBPIXEL]);
        self.debug_assert_matches_ram();
        y
    }

    pub(crate) fn move_x_by_subpixel_delta(&mut self, delta: u16) -> u16 {
        let x = move_link_axis_by_subpixel_delta(self.ram, LINK_X_SUBPIXEL, LINK_X_COORD, delta);
        self.state.set_x_with_subpixel(x, self.ram[LINK_X_SUBPIXEL]);
        self.debug_assert_matches_ram();
        x
    }

    pub(crate) fn move_y_by_subpixel_delta(&mut self, delta: u16) -> u16 {
        let y = move_link_axis_by_subpixel_delta(self.ram, LINK_Y_SUBPIXEL, LINK_Y_COORD, delta);
        self.state.set_y_with_subpixel(y, self.ram[LINK_Y_SUBPIXEL]);
        self.debug_assert_matches_ram();
        y
    }

    pub(crate) fn store_overworld_exit_position_from_current(&mut self) {
        write_le_u16(self.ram, LINK_Y_COORD_EXIT_OVERWORLD, self.state.y());
        write_le_u16(self.ram, LINK_X_COORD_EXIT_OVERWORLD, self.state.x());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn store_overworld_exit_y_from_current(&mut self) {
        write_le_u16(self.ram, LINK_Y_COORD_EXIT_OVERWORLD, self.state.y());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_y_from_overworld_exit(&mut self) {
        let y = read_le_u16(self.ram, LINK_Y_COORD_EXIT_OVERWORLD);
        self.state.set_y(y);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_position_from_overworld_exit(&mut self) {
        let x = read_le_u16(self.ram, LINK_X_COORD_EXIT_OVERWORLD);
        let y = read_le_u16(self.ram, LINK_Y_COORD_EXIT_OVERWORLD);
        self.state.set_position(x, y);
        write_le_u16(self.ram, LINK_X_COORD, x);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_position_from_safe_return(&mut self) {
        let x = self.state.safe_return_x();
        let y = self.state.safe_return_y();
        self.state.set_position(x, y);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        write_le_u16(self.ram, LINK_X_COORD, x);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn store_safe_return_position(&mut self, x: u16, y: u16) {
        self.state.store_safe_return_position(x, y);
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_LO] = x as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_HI] = (x >> 8) as u8;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn store_safe_return_low_from_current(&mut self) {
        self.state.store_safe_return_low_from_current();
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = self.state.safe_return_y() as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_LO] = self.state.safe_return_x() as u8;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn store_safe_return_y(&mut self, y: u16) {
        self.state.store_safe_return_y(y);
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_safe_return_y_low(&mut self, value: u8) {
        self.state.set_safe_return_y_low(value);
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_safe_return_position_from_current(&mut self) {
        self.state.cache_safe_return_position_from_current();
        self.ram[LINK_X_COORD_SAFE_RETURN_LO] = self.state.safe_return_x() as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_HI] = self.state.safe_return_x_high();
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = self.state.safe_return_y_low();
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = self.state.safe_return_y_high();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_safe_return_high_from_current(&mut self) {
        self.state.cache_safe_return_high_from_current();
        self.ram[LINK_X_COORD_SAFE_RETURN_HI] = self.state.safe_return_x_high();
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = self.state.safe_return_y_high();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_page_movement_deltas(&mut self) {
        self.state.clear_page_movement_deltas();
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = 0;
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_page_movement_deltas(&mut self, y_delta: u8, x_delta: u8) {
        self.state.set_page_movement_deltas(y_delta, x_delta);
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = y_delta;
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = x_delta;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_page_movement_delta_from_high_position(&mut self, high: u8) {
        self.state
            .set_y_page_movement_delta_from_high_position(high);
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = self.state.y_page_movement_delta();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_x_page_movement_delta_from_high_position(&mut self, high: u8) {
        self.state
            .set_x_page_movement_delta_from_high_position(high);
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = self.state.x_page_movement_delta();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_y_from_hop_origin(&mut self) {
        let y = read_le_u16(self.ram, LINK_Y_COORD_ORIGINAL);
        self.state.set_y(y);
        write_le_u16(self.ram, LINK_Y_COORD, y);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_link_state_block_for_ending(&mut self) {
        self.ram[LINK_Y_COORD..LINK_Y_COORD + 0x70].fill(0);
        self.sync_from_ram();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_oam_x_offset(&mut self, value: u8) {
        self.state.set_oam_x_offset(value);
        self.ram[PLAYER_OAM_X_OFFSET] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_oam_y_offset(&mut self, value: u8) {
        self.state.set_oam_y_offset(value);
        self.ram[PLAYER_OAM_Y_OFFSET] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_oam_offset(&mut self, y: u8, x: u8) {
        self.state.set_oam_offset(y, x);
        self.ram[PLAYER_OAM_Y_OFFSET] = y;
        self.ram[PLAYER_OAM_X_OFFSET] = x;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn disable_oam_offsets(&mut self) {
        self.state.disable_oam_offsets();
        self.ram[PLAYER_OAM_Y_OFFSET] = 0x80;
        self.ram[PLAYER_OAM_X_OFFSET] = 0x80;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.state.set_x_velocity(value);
        self.ram[LINK_X_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.state.set_y_velocity(value);
        self.ram[LINK_Y_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_movement_velocity_from_delta(&mut self, x_delta: u16, y_delta: u16) {
        self.state
            .set_movement_velocity_from_delta(x_delta, y_delta);
        self.ram[LINK_X_VELOCITY] = x_delta as u8;
        self.ram[LINK_Y_VELOCITY] = y_delta as u8;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn subtract_axis_velocity_delta(&mut self, horizontal: bool, delta: u8) {
        self.state.subtract_axis_velocity_delta(horizontal, delta);
        if horizontal {
            self.ram[LINK_X_VELOCITY] = self.state.x_velocity();
        } else {
            self.ram[LINK_Y_VELOCITY] = self.state.y_velocity();
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_movement_velocity_delta(&mut self, x_delta: u16, y_delta: u16) {
        self.state.add_movement_velocity_delta(x_delta, y_delta);
        self.ram[LINK_X_VELOCITY] = self.state.x_velocity();
        self.ram[LINK_Y_VELOCITY] = self.state.y_velocity();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_y_velocity_delta(&mut self, y_delta: u8) {
        self.state.add_y_velocity_delta(y_delta);
        self.ram[LINK_Y_VELOCITY] = self.state.y_velocity();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_movement_velocity(&mut self) {
        self.state.clear_movement_velocity();
        self.ram[LINK_X_VELOCITY] = 0;
        self.ram[LINK_Y_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_movement_subpixels(&mut self) {
        self.state.clear_movement_subpixels();
        self.ram[LINK_X_SUBPIXEL] = 0;
        self.ram[LINK_Y_SUBPIXEL] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_movement_velocity_and_direction(&mut self) {
        self.clear_movement_velocity();
        self.state.set_direction(0);
        self.ram[LINK_DIRECTION] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_velocity_from_safe_return_delta_unless_ledge_hopping(&mut self) {
        if self.ram[LINK_HANDLER_STATE] != 11 {
            let value = self.state.y_low_delta_from_safe_return();
            self.state.set_y_velocity(value);
            self.ram[LINK_Y_VELOCITY] = value;
            self.debug_assert_matches_ram();
        }
    }

    pub(crate) fn set_x_velocity_from_safe_return_delta(&mut self) {
        let value = self
            .state
            .x_low()
            .wrapping_sub(self.state.safe_return_x() as u8);
        self.state.set_x_velocity(value);
        self.ram[LINK_X_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn update_vertical_direction_from_movement_velocity(&mut self) {
        if self.state.y_velocity() != 0 {
            let direction = (self.state.direction() & 3)
                | if self.state.y_velocity_signed().is_negative() {
                    8
                } else {
                    4
                };
            self.state.set_direction(direction);
            self.ram[LINK_DIRECTION] = direction;
            self.debug_assert_matches_ram();
        }
    }

    pub(crate) fn update_horizontal_direction_from_movement_velocity(&mut self) {
        if self.state.x_velocity() != 0 {
            let direction = (self.state.direction() & 0x0c)
                | if self.state.x_velocity_signed().is_negative() {
                    2
                } else {
                    1
                };
            self.state.set_direction(direction);
            self.ram[LINK_DIRECTION] = direction;
            self.debug_assert_matches_ram();
        }
    }

    pub(crate) fn refresh_direction_from_safe_return_delta(&mut self) {
        self.set_y_velocity_from_safe_return_delta_unless_ledge_hopping();
        self.update_vertical_direction_from_movement_velocity();
        self.set_x_velocity_from_safe_return_delta();
        self.update_horizontal_direction_from_movement_velocity();
    }

    pub(crate) fn set_actual_x_velocity(&mut self, value: u8) {
        self.state.set_actual_x_velocity(value);
        self.ram[LINK_ACTUAL_X_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_actual_y_velocity(&mut self, value: u8) {
        self.state.set_actual_y_velocity(value);
        self.ram[LINK_ACTUAL_Y_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_actual_x_velocity(&mut self) {
        self.state.clear_actual_x_velocity();
        self.ram[LINK_ACTUAL_X_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_actual_y_velocity(&mut self) {
        self.state.clear_actual_y_velocity();
        self.ram[LINK_ACTUAL_Y_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_actual_velocity_xy(&mut self, x: u8, y: u8) {
        self.state.set_actual_velocity_xy(x, y);
        self.ram[LINK_ACTUAL_X_VELOCITY] = x;
        self.ram[LINK_ACTUAL_Y_VELOCITY] = y;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_actual_velocity_xy(&mut self) {
        self.state.clear_actual_velocity_xy();
        self.ram[LINK_ACTUAL_X_VELOCITY] = 0;
        self.ram[LINK_ACTUAL_Y_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn invert_actual_velocity_xy(&mut self) {
        self.state.invert_actual_velocity_xy();
        self.ram[LINK_ACTUAL_X_VELOCITY] = self.state.actual_x_velocity();
        self.ram[LINK_ACTUAL_Y_VELOCITY] = self.state.actual_y_velocity();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn xor_actual_velocity_xy(&mut self, mask: u8) {
        self.state.xor_actual_velocity_xy(mask);
        self.ram[LINK_ACTUAL_X_VELOCITY] = self.state.actual_x_velocity();
        self.ram[LINK_ACTUAL_Y_VELOCITY] = self.state.actual_y_velocity();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn derive_direction_from_actual_velocity(&mut self) {
        let mut direction = 0;
        if self.state.actual_y_velocity() != 0 {
            direction |= if self.state.actual_y_velocity_signed().is_negative() {
                8
            } else {
                4
            };
        }
        if self.state.actual_x_velocity() != 0 {
            direction |= if self.state.actual_x_velocity_signed().is_negative() {
                2
            } else {
                1
            };
        }
        self.state.set_direction(direction);
        self.ram[LINK_DIRECTION] = direction;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_actual_velocity_from_direction(&mut self, direction: u8, velocity: u8) {
        self.state
            .set_actual_velocity_from_direction(direction, velocity);
        self.ram[LINK_ACTUAL_X_VELOCITY] = self.state.actual_x_velocity();
        self.ram[LINK_ACTUAL_Y_VELOCITY] = self.state.actual_y_velocity();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_z(&mut self, value: u16) {
        self.state.set_z(value);
        write_le_u16(self.ram, LINK_Z_COORD, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_z_low(&mut self, value: u8) {
        self.state.set_z_low(value);
        self.ram[LINK_Z_COORD] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_z_high(&mut self) {
        self.state.clear_z_high();
        self.ram[LINK_Z_COORD + 1] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_z_low_from_mirror(&mut self) {
        self.state.restore_z_low_from_mirror();
        self.ram[LINK_Z_COORD] = self.ram[LINK_Z_COORD_MIRROR];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_z_from_mirror(&mut self) {
        self.state.restore_z_from_mirror();
        let value = read_le_u16(self.ram, LINK_Z_COORD_MIRROR);
        write_le_u16(self.ram, LINK_Z_COORD, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_z_low_to_mirror(&mut self) {
        self.state.cache_z_low_to_mirror();
        self.ram[LINK_Z_COORD_MIRROR] = self.ram[LINK_Z_COORD];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_z_to_mirror(&mut self) {
        self.state.cache_z_to_mirror();
        write_le_u16(self.ram, LINK_Z_COORD_MIRROR, self.state.z());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_z_mirror(&mut self, value: u16) {
        self.state.set_z_mirror(value);
        write_le_u16(self.ram, LINK_Z_COORD_MIRROR, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_z_mirror_low(&mut self) {
        self.state.clear_z_mirror_low();
        self.ram[LINK_Z_COORD_MIRROR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_z_mirror_word_low(&mut self) {
        self.state.clear_z_mirror_word_low();
        let value = read_le_u16(self.ram, LINK_Z_COORD_MIRROR) & !0x00ff;
        write_le_u16(self.ram, LINK_Z_COORD_MIRROR, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn force_z_mirror_low_ff(&mut self) {
        self.state.force_z_mirror_low_ff();
        let value = read_le_u16(self.ram, LINK_Z_COORD_MIRROR) | 0x00ff;
        write_le_u16(self.ram, LINK_Z_COORD_MIRROR, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_z_and_mirror(&mut self, value: u16) {
        self.state.set_z_and_mirror(value);
        write_le_u16(self.ram, LINK_Z_COORD, value);
        write_le_u16(self.ram, LINK_Z_COORD_MIRROR, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn move_z_by_velocity(&mut self, velocity: u8) -> u16 {
        let z = move_link_axis_by_velocity(self.ram, LINK_Z_SUBPIXEL, LINK_Z_COORD, velocity);
        self.state.set_z(z);
        self.debug_assert_matches_ram();
        z
    }

    pub(crate) fn set_actual_z_velocity(&mut self, value: u8) {
        self.state.set_actual_z_velocity(value);
        self.ram[LINK_Z_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_actual_z_velocity_and_copy(&mut self, value: u8) {
        self.state.set_actual_z_velocity_and_copy(value);
        self.ram[LINK_Z_VELOCITY] = value;
        self.ram[LINK_Z_VELOCITY_COPY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_actual_z_velocity_mirror_and_copy(&mut self, value: u8) {
        self.state.set_actual_z_velocity_mirror_and_copy(value);
        self.ram[LINK_Z_VELOCITY_MIRROR] = value;
        self.ram[LINK_Z_VELOCITY_COPY_MIRROR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_actual_z_velocity_from_mirror(&mut self) {
        self.state.restore_actual_z_velocity_from_mirror();
        self.ram[LINK_Z_VELOCITY] = self.ram[LINK_Z_VELOCITY_MIRROR];
        self.ram[LINK_Z_VELOCITY_COPY] = self.ram[LINK_Z_VELOCITY_COPY_MIRROR];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_actual_z_velocity_to_mirror(&mut self) {
        self.state.cache_actual_z_velocity_to_mirror();
        self.ram[LINK_Z_VELOCITY_MIRROR] = self.ram[LINK_Z_VELOCITY];
        self.ram[LINK_Z_VELOCITY_COPY_MIRROR] = self.ram[LINK_Z_VELOCITY_COPY];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn prime_airborne_z_velocity(&mut self) {
        self.state.prime_airborne_z_velocity();
        self.ram[LINK_Z_VELOCITY] = 0xff;
        write_le_u16(self.ram, LINK_Z_COORD, 0xffff);
        self.ram[LINK_Z_SUBPIXEL] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_actual_z_velocity(&mut self, delta: u8) {
        self.state.decrement_actual_z_velocity(delta);
        self.ram[LINK_Z_VELOCITY] = self.ram[LINK_Z_VELOCITY].wrapping_sub(delta);
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

    pub(crate) fn start_running(&mut self) {
        self.state.start_running();
        self.ram[LINK_IS_RUNNING] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_running_state(&mut self, value: u8) {
        self.state.set_running_state(value);
        self.ram[LINK_IS_RUNNING] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_dash_countdown(&mut self, value: u8) {
        self.state.set_dash_countdown(value);
        self.ram[LINK_COUNTDOWN_FOR_DASH] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_dash_countdown(&mut self) -> u8 {
        let value = self.state.increment_dash_countdown();
        self.ram[LINK_COUNTDOWN_FOR_DASH] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn decrement_dash_countdown(&mut self) -> u8 {
        let value = self.state.decrement_dash_countdown();
        self.ram[LINK_COUNTDOWN_FOR_DASH] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_dash_counter(&mut self, value: u8) {
        self.state.set_dash_counter(value);
        self.ram[LINK_DASH_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn prime_dash_counter(&mut self) {
        self.state.prime_dash_counter();
        self.ram[LINK_DASH_COUNTER] = 64;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_dash_counter_clamped_to_minimum(&mut self, minimum: u8) {
        self.state
            .decrement_dash_counter_clamped_to_minimum(minimum);
        self.ram[LINK_DASH_COUNTER] = self.state.dash_counter();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cancel_dash_state(&mut self) {
        self.state.cancel_dash_state();
        self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[LINK_IS_RUNNING] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn immobilize(&mut self) {
        self.state.immobilize();
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_immobilized(&mut self) {
        self.state.clear_immobilized();
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_button_mask_b_y(&mut self, value: u8) {
        self.state.set_button_mask_b_y(value);
        self.ram[BUTTON_MASK_B_Y] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_menu_block_flag(&mut self, value: u8) {
        self.state.set_menu_block_flag(value);
        self.ram[FLAG_BLOCK_LINK_MENU] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_menu_block(&mut self) {
        self.state.clear_menu_block();
        self.ram[FLAG_BLOCK_LINK_MENU] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_menu_block_flag(&mut self) -> u8 {
        let value = self.state.increment_menu_block_flag();
        self.ram[FLAG_BLOCK_LINK_MENU] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn add_button_mask_b_y_bits(&mut self, bits: u8) {
        self.state.add_button_mask_b_y_bits(bits);
        self.ram[BUTTON_MASK_B_Y] |= bits;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_pull_action_state(&mut self, value: u8) {
        self.state.set_pull_action_state(value);
        self.ram[LINK_PULL_ACTION_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_button_mask_b_y_bits(&mut self, mask: u8) {
        self.state.clear_button_mask_b_y_bits(mask);
        self.ram[BUTTON_MASK_B_Y] &= !mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_filtered_joypad_h(&mut self, value: u8) {
        self.state.set_filtered_joypad_h(value);
        self.ram[FILTERED_JOYPAD_H] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_filtered_joypad_l(&mut self, value: u8) {
        self.state.set_filtered_joypad_l(value);
        self.ram[FILTERED_JOYPAD_L] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_filtered_joypad_l_bits(&mut self, bits: u8) {
        self.state.clear_filtered_joypad_l_bits(bits);
        self.ram[FILTERED_JOYPAD_L] &= !bits;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_joypad1h_last(&mut self, value: u8) {
        self.state.set_joypad1h_last(value);
        self.ram[JOYPAD1H_LAST] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_joypad1l_last(&mut self, value: u8) {
        self.state.set_joypad1l_last(value);
        self.ram[JOYPAD1L_LAST] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_joypad1h_last2(&mut self, value: u8) {
        self.state.set_joypad1h_last2(value);
        self.ram[JOYPAD1H_LAST2] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_joypad1l_last2(&mut self, value: u8) {
        self.state.set_joypad1l_last2(value);
        self.ram[JOYPAD1L_LAST2] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_spin_attack_delay_timer(&mut self, value: u8) {
        self.state.set_spin_attack_delay_timer(value);
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_spin_attack_delay_timer(&mut self) -> u8 {
        let value = self.state.decrement_spin_attack_delay_timer();
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_incapacitated_timer(&mut self, value: u8) {
        self.state.set_incapacitated_timer(value);
        self.ram[LINK_INCAPACITATED_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_incapacitated_timer(&mut self) -> u8 {
        let value = self.state.decrement_incapacitated_timer();
        self.ram[LINK_INCAPACITATED_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn reset_elapsed_incapacitated_timer(&mut self) {
        self.state.reset_elapsed_incapacitated_timer();
        self.ram[LINK_INCAPACITATED_TIMER] = self.state.incapacitated_timer();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_visibility_status(&mut self, value: u8) {
        self.state.set_visibility_status(value);
        self.ram[LINK_VISIBILITY_STATUS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_button_action_flags(&mut self, value: u8) {
        self.state.set_y_button_action_flags(value);
        self.ram[Y_BUTTON_ACTION_FLAGS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_y_button_action_flag_bits(&mut self, bits: u8) {
        self.state.add_y_button_action_flag_bits(bits);
        self.ram[Y_BUTTON_ACTION_FLAGS] |= bits;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_y_button_action_flags(&mut self) {
        self.state.clear_y_button_action_flags();
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_button_action_step(&mut self, value: u8) {
        self.state.set_y_button_action_step(value);
        self.ram[Y_BUTTON_ACTION_STEP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_y_button_action_step(&mut self) {
        self.state.clear_y_button_action_step();
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_y_button_action_timer(&mut self, value: u8) {
        self.state.set_y_button_action_timer(value);
        self.ram[Y_BUTTON_ACTION_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_y_button_action_timer(&mut self) -> u8 {
        let value = self.state.decrement_y_button_action_timer();
        self.ram[Y_BUTTON_ACTION_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_defense_flags(&mut self) {
        self.state.clear_defense_flags();
        self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_swim_subpixel_and_defense_state(&mut self) {
        self.state.reset_swim_subpixel_and_defense_state();
        self.ram[LINK_X_SUBPIXEL] = 0;
        self.ram[LINK_Y_SUBPIXEL] = 0;
        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_defense_flags(&mut self, value: u8) {
        self.state.set_defense_flags(value);
        self.ram[PLAYER_DEFENSE_FLAGS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn or_defense_flags(&mut self, value: u8) {
        self.state.or_defense_flags(value);
        self.ram[PLAYER_DEFENSE_FLAGS] |= value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn and_defense_flags(&mut self, value: u8) {
        self.state.and_defense_flags(value);
        self.ram[PLAYER_DEFENSE_FLAGS] &= value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_receipt_method(&mut self, value: u8) {
        self.state.set_item_receipt_method(value);
        self.ram[ITEM_RECEIPT_METHOD] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_tile_below(&mut self, value: u8) {
        self.state.set_tile_below(value);
        self.ram[LINK_TILE_BELOW] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_tile_action_index(&mut self, value: u8) {
        self.state.set_tile_action_index(value);
        self.ram[TILE_ACTION_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_tile_coll_flag(&mut self, value: u8) {
        self.state.set_tile_coll_flag(value);
        self.ram[TILE_COLL_FLAG] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_tile_coll_flag(&mut self) {
        self.state.clear_tile_coll_flag();
        self.ram[TILE_COLL_FLAG] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_force_move_any_direction(&mut self, value: u16) {
        self.state.set_force_move_any_direction(value);
        write_le_u16(self.ram, FORCE_MOVE_ANY_DIRECTION, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_recoil_timer(&mut self, value: u8) {
        self.state.set_recoil_timer(value);
        self.ram[LINK_RECOIL_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_recoil_timer(&mut self) -> u8 {
        let value = self.state.increment_recoil_timer();
        self.ram[LINK_RECOIL_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn tick_jump_ledge_timer_or_reset(&mut self) -> bool {
        let reset = self.state.tick_jump_ledge_timer_or_reset();
        self.ram[LINK_TIMER_JUMP_LEDGE] = self.state.jump_ledge_timer();
        self.debug_assert_matches_ram();
        reset
    }

    pub(crate) fn reset_jump_ledge_timer(&mut self) {
        self.state.reset_jump_ledge_timer();
        self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_about_to_jump_off_ledge(&mut self) {
        self.state.clear_about_to_jump_off_ledge();
        self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_about_to_jump_off_ledge(&mut self) {
        self.state.increment_about_to_jump_off_ledge();
        self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = self.state.about_to_jump_off_ledge();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_push_fatigue_timer(&mut self) -> u8 {
        let value = self.state.decrement_push_fatigue_timer();
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_push_fatigue_timer(&mut self, value: u8) {
        self.state.set_push_fatigue_timer(value);
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_push_fatigue_timer(&mut self) {
        self.state.reset_push_fatigue_timer();
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_near_moveable_statue(&mut self) {
        self.state.clear_near_moveable_statue();
        self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn mark_near_moveable_statue(&mut self) {
        self.state.mark_near_moveable_statue();
        self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_pull_for_rupees_sprite_need(&mut self) {
        self.state.clear_pull_for_rupees_sprite_need();
        self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_pull_for_rupees_sprite_need(&mut self) {
        self.state.set_pull_for_rupees_sprite_need();
        self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_pit_correction(&mut self) {
        self.state.clear_pit_correction();
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_pit_correction_active(&mut self) {
        self.state.set_pit_correction_active();
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_pit_correction_timer(&mut self, value: u8) {
        self.state.set_pit_correction_timer(value);
        self.ram[PIT_CORRECTION_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_pit_correction_timer(&mut self) {
        self.state.increment_pit_correction_timer();
        self.ram[PIT_CORRECTION_TIMER] = self.state.pit_correction_timer();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_moving_against_diag_deadlocked(&mut self, value: u8) {
        self.state.set_moving_against_diag_deadlocked(value);
        self.ram[MOVING_AGAINST_DIAG_DEADLOCKED] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_electrocute_on_touch(&mut self) {
        self.state.clear_electrocute_on_touch();
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_electrocute_on_touch(&mut self, value: u8) {
        self.state.set_electrocute_on_touch(value);
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_faint_animation_active(&mut self) {
        self.state.clear_faint_animation_active();
        self.ram[LINK_FAINT_ANIMATION_ACTIVE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_faint_animation_active(&mut self, value: u8) {
        self.state.set_faint_animation_active(value);
        self.ram[LINK_FAINT_ANIMATION_ACTIVE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_item_debug_value_1(&mut self) {
        self.state.clear_item_debug_value_1();
        self.ram[LINK_DEBUG_VALUE_1] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_hookshot_grave_latch(&mut self) {
        self.state.clear_hookshot_grave_latch();
        self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_hookshot_grave_latch(&mut self) {
        self.state.set_hookshot_grave_latch();
        self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_dash_noise_request(&mut self) {
        self.state.set_dash_noise_request();
        self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_dash_noise_request(&mut self) {
        self.state.clear_dash_noise_request();
        self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_cape_mode(&mut self) {
        self.state.clear_cape_mode();
        self.ram[LINK_CAPE_MODE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_cape_mode(&mut self, value: u8) {
        self.state.set_cape_mode(value);
        self.ram[LINK_CAPE_MODE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_cape_decrement_counter(&mut self, value: u8) {
        self.state.set_cape_decrement_counter(value);
        self.ram[CAPE_DECREMENT_COUNTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_cape_decrement_counter(&mut self) {
        self.state.decrement_cape_decrement_counter();
        self.ram[CAPE_DECREMENT_COUNTER] = self.state.cape_decrement_counter();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_transforming(&mut self) {
        self.state.clear_transforming();
        self.ram[LINK_IS_TRANSFORMING] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_transforming(&mut self) {
        self.state.set_transforming();
        self.ram[LINK_IS_TRANSFORMING] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_sword_delay_timer(&mut self) {
        self.state.clear_sword_delay_timer();
        self.ram[LINK_SWORD_DELAY_TIMER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sword_delay_timer(&mut self, value: u8) {
        self.state.set_sword_delay_timer(value);
        self.ram[LINK_SWORD_DELAY_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_sword_delay_timer(&mut self) -> u8 {
        let value = self.state.decrement_sword_delay_timer();
        self.ram[LINK_SWORD_DELAY_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_spin_offsets(&mut self, value: u8) {
        self.state.set_spin_offsets(value);
        self.ram[LINK_SPIN_OFFSETS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_somaria_platform_state(&mut self) {
        self.state.clear_somaria_platform_state();
        self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_somaria_platform_state(&mut self, value: u8) {
        self.state.set_somaria_platform_state(value);
        self.ram[PLAYER_ON_SOMARIA_PLATFORM] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_spin_attack_step_counter(&mut self) {
        self.state.clear_spin_attack_step_counter();
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_spin_attack_step_counter(&mut self) -> u8 {
        let value = self.state.increment_spin_attack_step_counter();
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_spin_attack_sound_latch(&mut self, value: u8) {
        self.state.set_spin_attack_sound_latch(value);
        self.ram[SPIN_ATTACK_SOUND_LATCH] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_spin_attack_sound_latch(&mut self) {
        self.state.clear_spin_attack_sound_latch();
        self.ram[SPIN_ATTACK_SOUND_LATCH] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_state_for_spin_attack(&mut self, value: u8) {
        self.state.set_state_for_spin_attack(value);
        self.ram[STATE_FOR_SPIN_ATTACK] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_state_for_spin_attack(&mut self) {
        self.state.clear_state_for_spin_attack();
        self.ram[STATE_FOR_SPIN_ATTACK] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_immobilized_flag(&mut self) -> u8 {
        let value = self.state.increment_immobilized_flag();
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_immobilized_flag(&mut self, value: u8) {
        self.state.set_immobilized_flag(value);
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_incapacitated_camera_timer_from_incapacitated(&mut self) {
        self.state
            .reset_incapacitated_camera_timer_from_incapacitated();
        self.ram[LINK_INCAPACITATED_CAMERA_TIMER] = self.state.incapacitated_timer() >> 4;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_action_handler_timer(&mut self) {
        self.state.clear_action_handler_timer();
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_action_handler_timer(&mut self, value: u8) {
        self.state.set_action_handler_timer(value);
        self.ram[PLAYER_HANDLER_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_action_handler_timer(&mut self) -> u8 {
        let value = self.state.increment_action_handler_timer();
        self.ram[PLAYER_HANDLER_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_doorway_state(&mut self) {
        self.state.clear_doorway_state();
        self.ram[IS_STANDING_IN_DOORWAY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_doorway_state(&mut self, value: u8) {
        self.state.set_doorway_state(value);
        self.ram[IS_STANDING_IN_DOORWAY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_blink_countdown(&mut self) {
        self.state.clear_blink_countdown();
        self.ram[COUNTDOWN_FOR_BLINK] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_blink_countdown(&mut self, value: u8) {
        self.state.set_blink_countdown(value);
        self.ram[COUNTDOWN_FOR_BLINK] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_blink_countdown(&mut self) -> u8 {
        let value = self.state.decrement_blink_countdown();
        self.ram[COUNTDOWN_FOR_BLINK] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_spin_animation_step_counter(&mut self, value: u8) {
        self.state.set_spin_animation_step_counter(value);
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_spin_animation_step_counter(&mut self) -> u8 {
        let value = self.state.increment_spin_animation_step_counter();
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_spin_animation_step_counter(&mut self) {
        self.state.clear_spin_animation_step_counter();
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_button_b_frames(&mut self) {
        self.state.clear_button_b_frames();
        self.ram[BUTTON_B_FRAMES] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_button_b_frames(&mut self, value: u8) {
        self.state.set_button_b_frames(value);
        self.ram[BUTTON_B_FRAMES] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_button_b_frames_word(&mut self, value: u16) {
        self.state.set_button_b_frames_word(value);
        write_le_u16(self.ram, BUTTON_B_FRAMES, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_button_b_frames(&mut self) -> u8 {
        let value = self.state.increment_button_b_frames();
        self.ram[BUTTON_B_FRAMES] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn decrement_button_b_frames_word(&mut self) -> u16 {
        let value = self.state.decrement_button_b_frames_word();
        write_le_u16(self.ram, BUTTON_B_FRAMES, value);
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_animation_step(&mut self) {
        self.state.clear_animation_step();
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_animation_step(&mut self, value: u8) {
        self.state.set_animation_step(value);
        self.ram[LINK_ANIMATION_STEPS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_opening_pose(&mut self) {
        self.state.increment_opening_pose();
        self.ram[LINK_POSE_DURING_OPENING] = self.state.opening_pose();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_animation_step(&mut self, wrap_at: u8, wrap_to: u8) {
        self.state.advance_animation_step(wrap_at, wrap_to);
        self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1);
        if self.ram[LINK_ANIMATION_STEPS] == wrap_at {
            self.ram[LINK_ANIMATION_STEPS] = wrap_to;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_animation_step_at_least(&mut self, wrap_at: u8, wrap_to: u8) {
        self.state.advance_animation_step_at_least(wrap_at, wrap_to);
        self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1);
        if self.ram[LINK_ANIMATION_STEPS] >= wrap_at {
            self.ram[LINK_ANIMATION_STEPS] = wrap_to;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_idle_swim_animation(&mut self) {
        self.state.animation_step &= 1;
        self.state.frame_change_counter = self.state.frame_change_counter.wrapping_add(1);
        if self.state.frame_change_counter >= 16 {
            self.state.frame_change_counter = 0;
            self.state.swim_stroke_anim_step = 0;
            self.state.animation_step = (self.state.animation_step & 1) ^ 1;
        }
        self.ram[LINK_ANIMATION_STEPS] = self.state.animation_step;
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.state.frame_change_counter;
        self.ram[SWIM_STROKE_ANIM_STEP] = self.state.swim_stroke_anim_step;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_active_swim_animation(&mut self, stroke_steps: &[u8; 4]) {
        self.state.frame_change_counter = self.state.frame_change_counter.wrapping_add(1);
        if self.state.frame_change_counter >= 8 {
            self.state.frame_change_counter = 0;
            self.state.animation_step = self.state.animation_step.wrapping_add(1) & 3;
            self.state.swim_stroke_anim_step = stroke_steps[self.state.animation_step as usize];
        }
        self.ram[LINK_ANIMATION_STEPS] = self.state.animation_step;
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.state.frame_change_counter;
        self.ram[SWIM_STROKE_ANIM_STEP] = self.state.swim_stroke_anim_step;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_animation_step_if_at_least(&mut self, threshold: u8) {
        self.state.clear_animation_step_if_at_least(threshold);
        self.ram[LINK_ANIMATION_STEPS] = self.state.animation_step();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn subtract_animation_step_if_at_least(&mut self, threshold: u8, delta: u8) {
        self.state
            .subtract_animation_step_if_at_least(threshold, delta);
        self.ram[LINK_ANIMATION_STEPS] = self.state.animation_step();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_water_ripple_or_grass_state(&mut self) {
        self.state.clear_water_ripple_or_grass_state();
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_water_ripple_or_grass_state(&mut self, value: u8) {
        self.state.set_water_ripple_or_grass_state(value);
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_secondary_water_grass_timer(&mut self, value: u8) {
        self.state.set_secondary_water_grass_timer(value);
        self.ram[SECONDARY_WATER_GRASS_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_swim_fast_state(&mut self) {
        self.state.clear_swim_fast_state();
        self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_swimming_state_fields(&mut self) {
        self.state.reset_swimming_state_fields();
        self.ram[SWIMMING_COUNTDOWN] = 0;
        self.ram[LINK_SWIM_HARD_STROKE] = 0;
        self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn start_hard_swim_stroke(&mut self, hard_stroke: u8) {
        self.state.start_hard_swim_stroke(hard_stroke);
        self.ram[LINK_SWIM_HARD_STROKE] = hard_stroke;
        self.ram[LINK_MAYBE_SWIM_FASTER] = 1;
        self.ram[SWIMMING_COUNTDOWN] = 7;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn tick_hard_swim_stroke(&mut self) {
        let countdown = self.ram[SWIMMING_COUNTDOWN].wrapping_sub(1);
        self.ram[SWIMMING_COUNTDOWN] = countdown;
        if (countdown as i8).is_negative() {
            self.ram[SWIMMING_COUNTDOWN] = 7;
        }
        self.state.tick_hard_swim_stroke(countdown);
        self.ram[LINK_MAYBE_SWIM_FASTER] = self.state.swim_fast_state();
        self.ram[LINK_SWIM_HARD_STROKE] = self.state.hard_swim_stroke();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_pickup_in_progress(&mut self, value: u8) {
        self.state.set_item_pickup_in_progress(value);
        self.ram[ITEM_PICKUP_IN_PROGRESS_FLAG] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_hookshot_bg_check_off_timer(&mut self, value: u8) {
        self.state.set_hookshot_bg_check_off_timer(value);
        self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_hookshot_bg_check_off_timer(&mut self) {
        self.state.decrement_hookshot_bg_check_off_timer();
        self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER] = self.state.hookshot_bg_check_off_timer();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_selected_rod(&mut self, value: u8) {
        self.state.set_selected_rod(value);
        self.ram[EQ_SELECTED_ROD] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_flute_countdown(&mut self, value: u8) {
        self.state.set_flute_countdown(value);
        self.ram[FLUTE_COUNTDOWN] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_flute_countdown(&mut self) {
        self.state.decrement_flute_countdown();
        self.ram[FLUTE_COUNTDOWN] = self.state.flute_countdown();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_flute_countdown(&mut self) {
        self.state.clear_flute_countdown();
        self.ram[FLUTE_COUNTDOWN] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_index_of_dashing_sfx(&mut self) {
        self.state.clear_index_of_dashing_sfx();
        self.ram[INDEX_OF_DASHING_SFX] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_index_of_dashing_sfx(&mut self) {
        self.state.decrement_index_of_dashing_sfx();
        self.ram[INDEX_OF_DASHING_SFX] = self.state.index_of_dashing_sfx();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_water_ripple_or_grass_state(&mut self) -> u8 {
        let value = self.state.increment_water_ripple_or_grass_state();
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_primary_water_grass_timer(&mut self, value: u8) {
        self.state.set_primary_water_grass_timer(value);
        self.ram[PRIMARY_WATER_GRASS_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_deep_water_state(&mut self, value: u8) {
        self.state.set_deep_water_state(value);
        self.ram[LINK_IS_IN_DEEP_WATER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn enter_deep_water_state(&mut self) {
        self.state.enter_deep_water_state();
        self.ram[LINK_IS_IN_DEEP_WATER] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_deep_water_state(&mut self) {
        self.state.clear_deep_water_state();
        self.ram[LINK_IS_IN_DEEP_WATER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_conveyor_belt_state(&mut self) {
        self.state.clear_conveyor_belt_state();
        self.ram[LINK_ON_CONVEYOR_BELT] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_conveyor_belt_state(&mut self, value: u8) {
        self.state.set_conveyor_belt_state(value);
        self.ram[LINK_ON_CONVEYOR_BELT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_misc_bugfix_movement_state(&mut self) {
        self.state.clear_misc_bugfix_movement_state();
        self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 0;
        self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 0;
        self.ram[LINK_ON_CONVEYOR_BELT] = 0;
        self.ram[LINK_FLAG_MOVING] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_item_action_step_var(&mut self) {
        self.state.clear_item_action_step_var();
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_current_quadrants(&mut self) {
        self.state.cache_current_quadrants();
        self.ram[LINK_QUADRANT_X_CACHED] = self.ram[LINK_QUADRANT_X];
        self.ram[LINK_QUADRANT_Y_CACHED] = self.ram[LINK_QUADRANT_Y];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn restore_quadrants_from_cached(&mut self) {
        self.state.restore_quadrants_from_cached();
        self.ram[LINK_QUADRANT_X] = self.ram[LINK_QUADRANT_X_CACHED];
        self.ram[LINK_QUADRANT_Y] = self.ram[LINK_QUADRANT_Y_CACHED];
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_frame_change_counter(&mut self, delay: u8) -> bool {
        let advanced = self.state.advance_frame_change_counter(delay);
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.state.frame_change_counter;
        self.debug_assert_matches_ram();
        advanced
    }

    pub(crate) fn clear_frame_change_counter(&mut self) {
        self.state.clear_frame_change_counter();
        self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sprite_oam_state_timer(&mut self, value: u8) {
        self.state.set_sprite_oam_state_timer(value);
        self.ram[LINK_SPRITE_OAM_STATE_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_recoil_z_velocity_for_dungeon_reset(&mut self, value: u8) {
        self.state.set_recoil_z_velocity_for_dungeon_reset(value);
        self.ram[LINK_RECOIL_Z_VELOCITY_DUNGEON] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_recoil_z_velocity(&mut self, value: u8) {
        self.state.set_recoil_z_velocity(value);
        self.ram[LINK_RECOIL_Z_VELOCITY_DUNGEON] = value;
        self.ram[LINK_Z_VELOCITY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_sprite_oam_state_timer(&mut self) -> u8 {
        let value = self.state.decrement_sprite_oam_state_timer();
        self.ram[LINK_SPRITE_OAM_STATE_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn mark_pit_landing_oam_state(&mut self) {
        self.state.mark_pit_landing_oam_state();
        self.ram[LINK_SPRITE_OAM_STATE_TIMER] = 9;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_whirlpool_trigger(&mut self) {
        self.state.set_whirlpool_trigger();
        self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_whirlpool_trigger(&mut self) {
        self.state.clear_whirlpool_trigger();
        self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn prevent_movement(&mut self) {
        self.state.prevent_movement();
        self.ram[LINK_PREVENT_FROM_MOVING] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_prevent_movement(&mut self) {
        self.state.clear_prevent_movement();
        self.ram[LINK_PREVENT_FROM_MOVING] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_hop_origin_delta_from_y(&mut self, y: u16) -> u16 {
        let value = self.state.set_hop_origin_delta_from_y(y);
        write_le_u16(self.ram, LINK_Y_COORD_ORIGINAL, value);
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_movement_velocity_from_position_delta(
        &mut self,
        x: u16,
        y: u16,
        old_x: u16,
        old_y: u16,
    ) {
        self.state
            .set_movement_velocity_from_position_delta(x, y, old_x, old_y);
        self.ram[LINK_Y_VELOCITY] = self.state.y_velocity;
        self.ram[LINK_X_VELOCITY] = self.state.x_velocity;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_actual_velocity_and_page_movement_deltas(&mut self) {
        self.state.clear_actual_velocity_and_page_movement_deltas();
        self.ram[LINK_ACTUAL_X_VELOCITY] = 0;
        self.ram[LINK_ACTUAL_Y_VELOCITY] = 0;
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = 0;
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn cache_moving_floor_position(&mut self, x: u16, y: u16) {
        self.state.cache_moving_floor_position(x, y);
        write_le_u16(self.ram, RELATED_TO_MOVING_FLOOR_Y, y);
        write_le_u16(self.ram, RELATED_TO_MOVING_FLOOR_X, x);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_incapacitated_camera_timer(&mut self) -> u8 {
        let value = self.state.decrement_incapacitated_camera_timer();
        self.ram[LINK_INCAPACITATED_CAMERA_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn increment_pull_action_state(&mut self) {
        self.state.increment_pull_action_state();
        self.ram[LINK_PULL_ACTION_STATE] = self.state.pull_action_state;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_holding_timer(&mut self, value: u8) {
        self.state.set_item_holding_timer(value);
        self.ram[LINK_ITEM_HOLDING_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_swim_movement_velocity(&mut self) {
        self.state.clear_swim_movement_velocity();
        self.ram[LINK_Y_VELOCITY] = 0;
        self.ram[LINK_X_VELOCITY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_sleep_in_bed_state(&mut self) {
        self.state.increment_sleep_in_bed_state();
        self.ram[PLAYER_SLEEP_IN_BED_STATE] = self.state.sleep_in_bed_state;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_cached_tile_action_index(&mut self, value: u8) {
        self.state.set_cached_tile_action_index(value);
        self.ram[CACHED_TILE_ACTION_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_swimming_countdown(&mut self) {
        self.state.clear_swimming_countdown();
        self.ram[SWIMMING_COUNTDOWN] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_ancilla_interactive_reset_flag(&mut self) {
        self.state.clear_ancilla_interactive_reset_flag();
        self.ram[ANCILLA_INTERACTIVE_RESET_FLAG] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_force_move_high_byte(&mut self) {
        self.state.clear_force_move_high_byte();
        write_le_u16(
            self.ram,
            FORCE_MOVE_ANY_DIRECTION,
            self.state.force_move_any_direction,
        );
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sprite_pickup_flag_cached(&mut self, value: u8) {
        self.state.set_sprite_pickup_flag_cached(value);
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_swim_stroke_frame_counter(&mut self, offset: usize, value: u16) {
        self.state.set_swim_stroke_frame_counter(offset, value);
        write_le_u16(self.ram, SWIM_STROKE_FRAME_COUNTER + offset, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_magic_spell_player_lock(&mut self) {
        self.state.clear_magic_spell_player_lock();
        self.ram[MAGIC_SPELL_PLAYER_LOCK_FLAG] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_somaria_block_bg_check_flag(&mut self) {
        self.state.clear_somaria_block_bg_check_flag();
        self.ram[SOMARIA_BLOCK_BG_CHECK_FLAG] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_player_pose_draw_counter(&mut self) {
        self.state.clear_player_pose_draw_counter();
        self.ram[PLAYER_POSE_DRAW_COUNTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_player_pose_draw_counter(&mut self) {
        self.state.increment_player_pose_draw_counter();
        self.ram[PLAYER_POSE_DRAW_COUNTER] = self.state.player_pose_draw_counter;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_player_special_draw_flag(&mut self) {
        self.state.clear_player_special_draw_flag();
        self.ram[PLAYER_SPECIAL_DRAW_FLAG] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_player_special_draw_flag(&mut self, value: u8) {
        self.state.set_player_special_draw_flag(value);
        self.ram[PLAYER_SPECIAL_DRAW_FLAG] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_bit9_of_xcoord_word(&mut self, value: u16) {
        self.state.set_bit9_of_xcoord_word(value);
        write_le_u16(self.ram, BIT9_OF_XCOORD, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_item_action_step_var(&mut self) -> u8 {
        let value = self.state.increment_item_action_step_var();
        self.ram[LINK_ITEM_ACTION_STEP] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn advance_item_action_step_var_wrapping_7_to_1(&mut self) -> u8 {
        let value = self.state.advance_item_action_step_var_wrapping_7_to_1();
        self.ram[LINK_ITEM_ACTION_STEP] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_given_damage(&mut self) {
        self.state.clear_given_damage();
        self.ram[LINK_GIVE_DAMAGE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_given_damage(&mut self, value: u8) {
        self.state.set_given_damage(value);
        self.ram[LINK_GIVE_DAMAGE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_in_hand(&mut self, value: u8) {
        self.state.set_item_in_hand(value);
        self.ram[LINK_ITEM_IN_HAND] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_item_in_hand(&mut self) {
        self.state.clear_item_in_hand();
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_item_in_hand_bits(&mut self, mask: u8) {
        self.state.clear_item_in_hand_bits(mask);
        self.ram[LINK_ITEM_IN_HAND] &= !mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_position_mode(&mut self) {
        self.state.clear_position_mode();
        self.ram[LINK_POSITION_MODE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_position_mode(&mut self, value: u8) {
        self.state.set_position_mode(value);
        self.ram[LINK_POSITION_MODE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_position_mode_bits(&mut self, mask: u8) {
        self.state.set_position_mode_bits(mask);
        self.ram[LINK_POSITION_MODE] |= mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_position_mode_bits(&mut self, mask: u8) {
        self.state.clear_position_mode_bits(mask);
        self.ram[LINK_POSITION_MODE] &= !mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_action_step_var(&mut self, value: u8) {
        self.state.set_item_action_step_var(value);
        self.ram[LINK_ITEM_ACTION_STEP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_throw_oam_state_index(&mut self, value: u8) {
        self.state.set_throw_oam_state_index(value);
        self.ram[LINK_THROW_OAM_STATE_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_action_debug_value_2(&mut self, value: u8) {
        self.state.set_item_action_debug_value_2(value);
        self.ram[LINK_DEBUG_VALUE_2] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_current_item_y(&mut self, value: u8) {
        self.state.set_current_item_y(value);
        self.ram[LINK_CURRENT_ITEM_Y] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_current_item_active(&mut self, value: u8) {
        self.state.set_current_item_active(value);
        self.ram[LINK_CURRENT_ITEM_ACTIVE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_receive_item_index(&mut self, value: u8) {
        self.state.set_receive_item_index(value);
        self.ram[LINK_RECEIVE_ITEM_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_link_dma_graphics_index_word(&mut self, value: u16) {
        self.state.set_link_dma_graphics_index_word(value);
        write_le_u16(self.ram, LINK_DMA_GRAPHICS_INDEX, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_link_dma_left_sprite_bank_word(&mut self, value: u16) {
        self.state.set_link_dma_left_sprite_bank_word(value);
        write_le_u16(self.ram, LINK_DMA_LEFT_SPRITE_BANK_INDEX, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_link_dma_right_sprite_bank_word(&mut self, value: u16) {
        self.state.set_link_dma_right_sprite_bank_word(value);
        write_le_u16(self.ram, LINK_DMA_RIGHT_SPRITE_BANK_INDEX, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_link_dma_sprite_banks(&mut self) {
        self.state.clear_link_dma_sprite_banks();
        write_le_u16(self.ram, LINK_DMA_LEFT_SPRITE_BANK_INDEX, 0);
        write_le_u16(self.ram, LINK_DMA_RIGHT_SPRITE_BANK_INDEX, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_palette_bits_of_oam_word(&mut self, value: u16) {
        self.state.set_palette_bits_of_oam_word(value);
        write_le_u16(self.ram, LINK_PALETTE_BITS_OF_OAM, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_link_dma_source_offset(&mut self) -> u16 {
        let value = self.state.advance_link_dma_source_offset();
        write_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET, value);
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn advance_link_dma_tile_offset(&mut self) -> u16 {
        let value = self.state.advance_link_dma_tile_offset();
        write_le_u16(self.ram, LINK_DMA_TILE_OFFSET, value);
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_link_dma_countdown(&mut self, value: u16) {
        self.state.set_link_dma_countdown(value);
        write_le_u16(self.ram, LINK_DMA_COUNTDOWN, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_link_dma_countdown(&mut self) -> u16 {
        let value = self.state.decrement_link_dma_countdown();
        write_le_u16(self.ram, LINK_DMA_COUNTDOWN, value);
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn reset_link_dma_animation_cycle(&mut self, countdown: u16) {
        self.state.reset_link_dma_animation_cycle(countdown);
        write_le_u16(self.ram, LINK_DMA_COUNTDOWN, countdown);
        write_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET, 0);
        write_le_u16(self.ram, LINK_DMA_TILE_OFFSET, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sword_dma_graphics_index(&mut self, value: u8) {
        self.state.set_sword_dma_graphics_index(value);
        self.ram[LINK_DMA_SWORD_GRAPHICS_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_shield_dma_graphics_index(&mut self, value: u8) {
        self.state.set_shield_dma_graphics_index(value);
        self.ram[LINK_DMA_SHIELD_GRAPHICS_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_link_dma_staging_index(&mut self, value: u8) {
        self.state.set_link_dma_staging_index(value);
        self.ram[LINK_DMA_STAGING_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_link_sprite_index_scratch(&mut self, value: u16) {
        self.state.set_link_sprite_index_scratch(value);
        write_le_u16(self.ram, SCRATCH_1, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_hop_origin_coord(&mut self, value: u16) {
        self.state.set_hop_origin_coord(value);
        write_le_u16(self.ram, LINK_Y_COORD_ORIGINAL, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn spend_magic(&mut self, cost: u8) -> bool {
        let spent = self.state.spend_magic(cost);
        if spent {
            self.ram[LINK_MAGIC_POWER] = self.state.magic_power();
        }
        self.debug_assert_matches_ram();
        spent
    }

    pub(crate) fn refund_magic(&mut self, cost: u8, clamp_full: bool) {
        self.state.refund_magic(cost, clamp_full);
        self.ram[LINK_MAGIC_POWER] = self.state.magic_power();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_magic_power(&mut self) -> u8 {
        let value = self.state.decrement_magic_power();
        self.ram[LINK_MAGIC_POWER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_magic_power(&mut self, value: u8) {
        self.state.set_magic_power(value);
        self.ram[LINK_MAGIC_POWER] = self.state.magic_power();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_magic_power(&mut self) {
        self.state.increment_magic_power();
        self.ram[LINK_MAGIC_POWER] = self.state.magic_power();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_action_scratch_state(&mut self) {
        self.state.clear_action_scratch_state();
        self.ram[LINK_DEBUG_VALUE_1] = 0;
        self.ram[LINK_DEBUG_VALUE_2] = 0;
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
        self.ram[LINK_THROW_OAM_STATE_INDEX] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_lift_throw_scratch_state(&mut self) {
        self.state.clear_lift_throw_scratch_state();
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
        self.ram[LINK_THROW_OAM_STATE_INDEX] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_ancilla_pickup_flag(&mut self) {
        self.state.clear_ancilla_pickup_flag();
        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_ancilla_pickup_flag(&mut self, value: u8) {
        self.state.set_ancilla_pickup_flag(value);
        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_sprite_pickup_flag(&mut self) {
        self.state.clear_sprite_pickup_flag();
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sprite_pickup_flag(&mut self, value: u8) {
        self.state.set_sprite_pickup_flag(value);
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_hookshot_interlock(&mut self, value: u8) {
        self.state.set_hookshot_interlock(value);
        self.ram[RELATED_TO_HOOKSHOT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_hookshot_interlock(&mut self) {
        self.state.clear_hookshot_interlock();
        self.ram[RELATED_TO_HOOKSHOT] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn xor_hookshot_interlock(&mut self, mask: u8) {
        self.state.xor_hookshot_interlock(mask);
        self.ram[RELATED_TO_HOOKSHOT] ^= mask;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_grabbing_wall(&mut self) {
        self.state.clear_grabbing_wall();
        self.ram[LINK_GRABBING_WALL] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_grabbing_wall(&mut self, value: u8) {
        self.state.set_grabbing_wall(value);
        self.ram[LINK_GRABBING_WALL] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn enable_cutscene_immunity(&mut self) {
        self.state.enable_cutscene_immunity();
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sprite_damage_disable_timer(&mut self, value: u8) {
        self.state.set_sprite_damage_disable_timer(value);
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_sprite_damage_disable_timer(&mut self) {
        self.state.clear_sprite_damage_disable_timer();
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_sprite_damage_disable_timer(&mut self) {
        self.state.increment_sprite_damage_disable_timer();
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = self.ram[LINK_DISABLE_SPRITE_DAMAGE].wrapping_add(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_item_hold_pose(&mut self, value: u8) {
        self.state.set_item_hold_pose(value);
        self.ram[LINK_POSE_FOR_ITEM] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_item_hold_pose(&mut self) {
        self.state.clear_item_hold_pose();
        self.ram[LINK_POSE_FOR_ITEM] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn force_hold_sword_up(&mut self) {
        self.state.force_hold_sword_up();
        self.ram[LINK_FORCE_HOLD_SWORD_UP] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_force_hold_sword_up(&mut self) {
        self.state.clear_force_hold_sword_up();
        self.ram[LINK_FORCE_HOLD_SWORD_UP] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_near_pit_state(&mut self, value: u8) {
        self.state.set_near_pit_state(value);
        self.ram[PLAYER_NEAR_PIT_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_near_pit_state(&mut self) {
        self.state.clear_near_pit_state();
        self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_pit_data_index(&mut self, value: u8) {
        self.state.set_pit_data_index(value);
        self.ram[PLAYER_PIT_DATA_INDEX] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_pit_data_index(&mut self) {
        self.state.clear_pit_data_index();
        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_pit_data_index(&mut self) -> u8 {
        let value = self.state.advance_pit_data_index();
        self.ram[PLAYER_PIT_DATA_INDEX] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn begin_pit_check(&mut self) {
        self.state.begin_pit_check();
        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        self.ram[PLAYER_NEAR_PIT_STATE] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_pit_state(&mut self) {
        self.state.clear_pit_state();
        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_cape_transform_timer(&mut self, value: u8) {
        self.state.set_cape_transform_timer(value);
        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn tick_cape_transform_timer(&mut self) -> u8 {
        let value = self.state.tick_cape_transform_timer();
        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn clear_cape_transform_timer(&mut self) {
        self.state.clear_cape_transform_timer();
        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_bunny_mirror(&mut self) {
        self.state.clear_bunny_mirror();
        self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_bunny_body_state(&mut self) {
        self.state.clear_bunny_body_state();
        self.ram[LINK_IS_BUNNY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_bunny_state(&mut self, value: u8) {
        self.state.set_bunny_state(value);
        self.ram[LINK_IS_BUNNY] = value;
        self.ram[LINK_IS_BUNNY_MIRROR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_bunny_transform_flags(&mut self) {
        self.state.clear_bunny_transform_flags();
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        self.ram[LINK_IS_BUNNY] = 0;
        self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_bunny_transform_after_moon_pearl(&mut self) {
        self.state.clear_bunny_transform_after_moon_pearl();
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        self.ram[LINK_IS_BUNNY] = 0;
        self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        self.ram[LINK_TIMER_TEMPBUNNY] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_transform_poof_need_and_temp_bunny_timer(&mut self) {
        self.state.clear_transform_poof_need_and_temp_bunny_timer();
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_temp_bunny_timer(&mut self) {
        self.state.clear_temp_bunny_timer();
        write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_temp_bunny_timer(&mut self, value: u16) {
        self.state.set_temp_bunny_timer(value);
        write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_temp_bunny_timer(&mut self) -> u16 {
        let value = self.state.decrement_temp_bunny_timer();
        write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, value);
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn start_bunny_transform_poof(&mut self) {
        self.state.start_bunny_transform_poof();
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 1;
        self.ram[LINK_VISIBILITY_STATUS] = 12;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn finish_bunny_transform_poof(&mut self) {
        self.state.finish_bunny_transform_poof();
        self.ram[LINK_IS_BUNNY_MIRROR] = 1;
        self.ram[LINK_IS_BUNNY] = 1;
        self.ram[LINK_VISIBILITY_STATUS] = 0;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_auxiliary_state(&mut self, value: u8) {
        self.state.set_auxiliary_state(value);
        self.ram[LINK_AUXILIARY_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_auxiliary_state(&mut self) {
        self.state.clear_auxiliary_state();
        self.ram[LINK_AUXILIARY_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_state_bits(&mut self, value: u8) {
        self.state.set_state_bits(value);
        self.ram[LINK_STATE_BITS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_state_bits(&mut self) {
        self.state.clear_state_bits();
        self.ram[LINK_STATE_BITS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_lifting_or_carrying_state(&mut self) {
        self.state.clear_lifting_or_carrying_state();
        self.ram[LINK_STATE_BITS] &= !0x80;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn keep_only_lifting_or_carrying_state(&mut self) {
        self.state.keep_only_lifting_or_carrying_state();
        self.ram[LINK_STATE_BITS] &= 0x80;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn enter_item_hold_pose(&mut self) {
        self.state.enter_item_hold_pose();
        self.ram[LINK_STATE_BITS] = 0x80;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_FACING] = 0;
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_state_item_and_grab_flags(&mut self) {
        self.state.clear_state_item_and_grab_flags();
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_GRABBING_WALL] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_picking_throw_state(&mut self) {
        self.state.clear_picking_throw_state();
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_picking_throw_state(&mut self, value: u8) {
        self.state.set_picking_throw_state(value);
        self.ram[LINK_PICKING_THROW_STATE] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn start_lift_throw_state(&mut self) {
        self.state.start_lift_throw_state();
        self.ram[LINK_PICKING_THROW_STATE] = 1;
        self.ram[LINK_STATE_BITS] = 0x80;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_swimming_action_state(&mut self) {
        self.state.clear_swimming_action_state();
        self.ram[BUTTON_MASK_B_Y] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn initialize_link_action_state(&mut self) {
        self.state.initialize_link_action_state();
        self.ram[LINK_FACING] = 2;
        self.ram[LINK_LAST_DIRECTION] = 0;
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.ram[LINK_POSITION_MODE] = 0;
        self.ram[LINK_DEBUG_VALUE_1] = 0;
        self.ram[LINK_DEBUG_VALUE_2] = 0;
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
        self.ram[LINK_THROW_OAM_STATE_INDEX] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[LINK_IS_TRANSFORMING] = 0;
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_GRABBING_WALL] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_properties_c_fields(&mut self) {
        self.state.reset_properties_c_fields();
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
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
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
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn finish_link_action_state_initialization(&mut self) {
        self.state.finish_link_action_state_initialization();
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        self.ram[LINK_Z_COORD + 1] = 0;
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
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_properties_a_fields(&mut self) {
        self.state.reset_properties_a_fields();
        self.ram[LINK_LAST_DIRECTION] = 0;
        self.ram[LINK_DIRECTION] = 0;
        self.ram[LINK_FLAG_MOVING] = 0;
        self.ram[LINK_IS_TRANSFORMING] = 0;
        self.ram[COUNTDOWN_FOR_BLINK] = 0;
        self.ram[PLAYER_RESET_ANCILLA_WORK_BYTE_24] = 0;
        self.ram[LINK_IS_BUNNY] = 0;
        self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, 0);
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
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_properties_b_fields(&mut self) {
        self.state.reset_properties_b_fields();
        self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED] = 0;
        self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_drag_player_x(&mut self, value: u16) {
        self.state.set_drag_player_x(value);
        write_le_u16(self.ram, DRAG_PLAYER_X, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_drag_player_y(&mut self, value: u16) {
        self.state.set_drag_player_y(value);
        write_le_u16(self.ram, DRAG_PLAYER_Y, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_drag_player_x(&mut self, delta: u16) {
        self.state.add_drag_player_x(delta);
        write_le_u16(self.ram, DRAG_PLAYER_X, self.state.drag_player_x());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn add_drag_player_y(&mut self, delta: u16) {
        self.state.add_drag_player_y(delta);
        write_le_u16(self.ram, DRAG_PLAYER_Y, self.state.drag_player_y());
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_gravestone_push_timeout(&mut self, value: u8) {
        self.state.set_gravestone_push_timeout(value);
        self.ram[GRAVESTONE_PUSH_TIMEOUT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_gravestone_push_timeout(&mut self) {
        self.state.decrement_gravestone_push_timeout();
        self.ram[GRAVESTONE_PUSH_TIMEOUT] = self.state.gravestone_push_timeout();
        self.debug_assert_matches_ram();
    }

    pub(crate) fn land_after_splash(&mut self) {
        let handler_state = if self.ram[LINK_IS_BUNNY_MIRROR] != 0 {
            if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
                3
            } else {
                23
            }
        } else if self.state.is_in_deep_water() {
            4
        } else {
            0
        };
        self.state.land_after_splash_with_handler(handler_state);
        self.ram[LINK_HANDLER_STATE] = handler_state;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn enter_water_hop_state(&mut self) {
        self.state.enter_water_hop_state();
        if self.ram[LINK_AUXILIARY_STATE] != 2 {
            self.ram[LINK_AUXILIARY_STATE] = 1;
            self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        }
        self.ram[LINK_HANDLER_STATE] = 6;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn interrupt_swimming_for_auxiliary_state(&mut self) {
        self.state.interrupt_swimming_for_auxiliary_state();
        self.ram[LINK_HANDLER_STATE] = 2;
        self.ram[LINK_Z_COORD + 1] = 0;
        self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
        self.ram[LINK_SWIM_HARD_STROKE] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_idle_swim_animation_if_out_of_water(&mut self) {
        self.state.reset_idle_swim_animation_if_out_of_water();
        if self.ram[LINK_HANDLER_STATE] != 4 {
            self.ram[LINK_ANIMATION_STEPS] = 0;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn become_bunny_handler(&mut self) {
        self.state.become_bunny_handler();
        self.ram[LINK_HANDLER_STATE] = 23;
        self.ram[LINK_IS_BUNNY] = 1;
        self.ram[LINK_IS_BUNNY_MIRROR] = 1;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn setup_bed_pose(&mut self) {
        self.state.setup_bed_pose();
        self.ram[LINK_HANDLER_STATE] = 0x16;
        self.ram[PLAYER_SLEEP_IN_BED_STATE] = 0;
        self.ram[LINK_POSE_DURING_OPENING] = 0;
        self.ram[LINK_COUNTDOWN_FOR_DASH] = 3;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_after_damaging_pit(&mut self) {
        let handler_state = if self.ram[LINK_IS_BUNNY] != 0 && self.ram[LINK_ITEM_MOON_PEARL] == 0 {
            23
        } else {
            0
        };
        self.state.reset_after_damaging_pit(handler_state);
        self.ram[LINK_HANDLER_STATE] = handler_state;
        self.ram[LINK_LAST_DIRECTION] = self.state.swim_direction_flags();
        self.ram[LINK_IS_IN_DEEP_WATER] = 0;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn recache_bunny_state(&mut self) {
        let has_moon_pearl = self.ram[LINK_ITEM_MOON_PEARL] != 0;
        self.state.recache_bunny_state(has_moon_pearl);
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        write_le_u16(self.ram, LINK_TIMER_TEMPBUNNY, 0);
        if has_moon_pearl {
            self.ram[LINK_IS_BUNNY] = 0;
            self.ram[LINK_AUXILIARY_STATE] = 0;
        }
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.ram[LINK_IS_TRANSFORMING] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn enter_deep_water(&mut self) {
        self.state.enter_deep_water();
        self.ram[LINK_IS_IN_DEEP_WATER] = 1;
        self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.state.last_direction();
        self.ram[LINK_GRABBING_WALL] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
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
            // tiledetect_pit_tile is a uint8 at 0x59 (C); 0x5a is
            // link_this_controls_sprite_oam (the overworld pit-fall counter). Read
            // and project only the byte so the u16 field never clobbers 0x5a.
            pit_tile: u16::from(ram[TILEDETECT_PIT_TILE]),
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
        ram[TILEDETECT_PIT_TILE] = self.pit_tile as u8;
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

    pub(crate) fn set_collision_bits_low_byte(&mut self, value: u8) {
        self.collision_bits = (self.collision_bits & 0xff00) | u16::from(value);
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

    /// Sets only the LOW byte of collision_bits (R14 @ 0x0e), preserving the high byte (0x0f =
    /// R15 / SPRITE_LAST_GARNISH_INDEX, a stale leftover). C's room-tag dispatcher writes
    /// `ram[R14] = k` as a BYTE; the full-u16 setter clobbers 0x0f. The u16 projection re-stamps
    /// 0x0f from the live native high byte (coherent with RAM), so it is preserved.
    pub(crate) fn set_collision_bits_low_byte(&mut self, value: u8) {
        self.state.set_collision_bits_low_byte(value);
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
