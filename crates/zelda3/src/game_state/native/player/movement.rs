//! Position, velocity, floor, collision, and locomotion state.

use super::{swim_axis_index, FollowerLinkState, FOLLOWER_LAYER_BITS_BY_FLOOR, SWIM_AXIS_COUNT};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct PlayerMovementState {
    pub(super) x: u16,
    pub(super) y: u16,
    pub(super) z: u16,
    pub(super) z_mirror: u16,
    pub(super) x_subpixel: u8,
    pub(super) y_subpixel: u8,
    pub(super) x_velocity: u8,
    pub(super) y_velocity: u8,
    pub(super) actual_x_velocity: u8,
    pub(super) actual_y_velocity: u8,
    pub(super) z_velocity: u8,
    pub(super) z_velocity_copy: u8,
    pub(super) z_velocity_mirror: u8,
    pub(super) z_velocity_copy_mirror: u8,
    pub(super) recoil_z_velocity_for_dungeon_reset: u8,
    pub(super) recoil_timer: u8,
    pub(super) floor: u8,
    pub(super) lower_level_mirror_state: u8,
    pub(super) cached_lower_level_state: u8,
    pub(super) cached_lower_level_mirror_state: u8,
    pub(super) direction: u8,
    pub(super) direction_lock: u8,
    pub(super) direction_mask_a: u8,
    pub(super) direction_mask_b: u8,
    pub(super) last_direction: u8,
    pub(super) last_direction_moved_towards: u8,
    pub(super) moving_against_diag_tile: u8,
    pub(super) movement_flag: u8,
    pub(super) quadrant_x: u8,
    pub(super) quadrant_y: u8,
    pub(super) cached_quadrant_x: u8,
    pub(super) cached_quadrant_y: u8,
    pub(super) num_orthogonal_directions: u8,
    pub(super) swim_direction_flags: u8,
    pub(super) facing: u8,
    pub(super) facing_mirror: u8,
    pub(super) cached_facing: u8,
    pub(super) speed_setting: u8,
    pub(super) speed_modifier: u8,
    pub(super) dash_counter: u8,
    pub(super) dash_countdown: u8,
    pub(super) jump_ledge_timer: u8,
    pub(super) about_to_jump_off_ledge: u8,
    pub(super) push_fatigue_timer: u8,
    pub(super) gravestone_push_timeout: u8,
    pub(super) running: u8,
    pub(super) doorway_state: u8,
    pub(super) deep_water_state: u8,
    pub(super) swim_fast_state: u8,
    pub(super) hard_swim_stroke: u8,
    pub(super) swim_stroke_frame_counters: [u16; SWIM_AXIS_COUNT],
    pub(super) swim_stroke_anim_step: u8,
    pub(super) swimming_countdown: u8,
    pub(super) conveyor_belt_state: u8,
    pub(super) tile_below: u8,
    pub(super) tile_action_index: u8,
    pub(super) tile_collision_flag: u8,
    pub(super) whirlpool_trigger: u8,
    pub(super) prevent_movement: u8,
    pub(super) cached_tile_action_index: u8,
    pub(super) force_move_any_direction: u16,
    pub(super) position_mode: u8,
    pub(super) near_moveable_statue_flag: u8,
    pub(super) grabbing_wall: u8,
    pub(super) somaria_platform_state: u8,
    pub(super) near_pit_state: u8,
    pub(super) pit_data_index: u8,
    pub(super) pit_correction_timer: u8,
    pub(super) pit_correction_active: u8,
    pub(super) moving_against_diag_deadlocked: u8,
    pub(super) incapacitated_camera_timer: u8,
    pub(super) hop_origin_coord: u16,
    pub(super) cached_x: u16,
    pub(super) cached_y: u16,
    pub(super) copied_x: u16,
    pub(super) copied_y: u16,
    pub(super) previous_x: u16,
    pub(super) previous_y: u16,
    pub(super) safe_return_x: u16,
    pub(super) safe_return_y: u16,
    pub(super) bit9_of_xcoord: u16,
    pub(super) cheat_walk_through_walls: u8,
    pub(super) x_page_movement_delta: u8,
    pub(super) y_page_movement_delta: u8,
    pub(super) moving_floor_x: u16,
    pub(super) moving_floor_y: u16,
    pub(super) drag_player_x: u16,
    pub(super) drag_player_y: u16,
}

impl PlayerMovementState {
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

    pub(crate) fn z_mirror_low(&self) -> u8 {
        self.z_mirror as u8
    }

    pub(crate) fn z_mirror_delta_low(&self) -> u8 {
        self.z_mirror_low().wrapping_sub(self.z_low())
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
        FollowerLinkState::oam_priority_for_floor_value(self.floor)
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

    pub(crate) fn is_running(&self) -> bool {
        self.running != 0
    }

    pub(crate) fn running_state(&self) -> u8 {
        self.running
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

    pub(crate) fn has_grabbing_wall_state(&self) -> bool {
        self.grabbing_wall != 0
    }

    pub(crate) fn grabbing_wall(&self) -> u8 {
        self.grabbing_wall
    }

    pub(crate) fn grabbing_wall_has(&self, mask: u8) -> bool {
        self.grabbing_wall & mask != 0
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

    pub(crate) fn doorway_state(&self) -> u8 {
        self.doorway_state
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

    pub(crate) fn whirlpool_triggered(&self) -> bool {
        self.whirlpool_trigger != 0
    }

    pub(crate) fn is_prevented_from_moving(&self) -> bool {
        self.prevent_movement != 0
    }

    pub(crate) fn force_move_any_direction(&self) -> u16 {
        self.force_move_any_direction
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

    pub(crate) fn moving_floor_x(&self) -> u16 {
        self.moving_floor_x
    }

    pub(crate) fn moving_floor_y(&self) -> u16 {
        self.moving_floor_y
    }

    pub(crate) fn cheat_walk_through_walls(&self) -> u8 {
        self.cheat_walk_through_walls
    }

    pub(crate) fn is_near_moveable_statue(&self) -> bool {
        self.near_moveable_statue_flag != 0
    }

    pub(super) fn set_speed_setting(&mut self, value: u8) {
        self.speed_setting = value;
    }

    pub(super) fn decrement_speed_setting(&mut self) -> u8 {
        self.speed_setting = self.speed_setting.wrapping_sub(1);
        self.speed_setting
    }

    pub(super) fn clear_speed_modifier(&mut self) {
        self.speed_modifier = 0;
    }

    pub(super) fn set_speed_modifier(&mut self, value: u8) {
        self.speed_modifier = value;
    }

    pub(super) fn mark_lower_level(&mut self) {
        self.floor = 1;
    }

    pub(super) fn mark_lower_level_mirror(&mut self) {
        self.lower_level_mirror_state = 1;
    }

    pub(super) fn set_lower_level_state(&mut self, value: u8) {
        self.floor = value;
    }

    pub(super) fn set_lower_level_mirror_state(&mut self, value: u8) {
        self.lower_level_mirror_state = value;
    }

    pub(super) fn set_lower_level_states(&mut self, state: u8, mirror: u8) {
        self.floor = state;
        self.lower_level_mirror_state = mirror;
    }

    pub(super) fn clear_lower_level(&mut self) {
        self.floor = 0;
    }

    pub(super) fn clear_lower_level_states(&mut self) {
        self.floor = 0;
        self.lower_level_mirror_state = 0;
    }

    pub(super) fn toggle_lower_level_state(&mut self) {
        self.floor ^= 1;
    }

    pub(super) fn toggle_lower_level_mirror_state(&mut self) {
        self.lower_level_mirror_state ^= 1;
    }

    pub(super) fn mirror_lower_level_state(&mut self) {
        self.lower_level_mirror_state = self.floor;
    }

    pub(super) fn cache_lower_level_states(&mut self) {
        self.cached_lower_level_state = self.floor;
        self.cached_lower_level_mirror_state = self.lower_level_mirror_state;
    }

    pub(super) fn restore_lower_level_state_from_cached(&mut self) {
        self.floor = self.cached_lower_level_state;
        self.lower_level_mirror_state = self.cached_lower_level_mirror_state;
    }

    pub(super) fn arm_stair_speed_modifier(&mut self) {
        self.speed_setting = 2;
        self.speed_modifier = 1;
    }

    pub(super) fn resolve_dash_speed_setting(&mut self) {
        if self.speed_setting == 2 {
            self.speed_setting = if self.running != 0 { 16 } else { 0 };
        }
    }

    pub(super) fn promote_pending_speed_modifier(&mut self) {
        if self.speed_modifier == 1 {
            self.speed_modifier = 2;
        }
    }

    pub(super) fn increase_near_pit_speed_modifier(&mut self) {
        self.speed_modifier = if self.speed_modifier < 48 {
            self.speed_modifier.wrapping_add(8)
        } else {
            32
        };
    }

    pub(super) fn advance_dash_deceleration(&mut self) {
        self.speed_modifier = self.speed_modifier.wrapping_add(1);
    }

    pub(super) fn set_dash_countdown(&mut self, value: u8) {
        self.dash_countdown = value;
    }

    pub(super) fn increment_dash_countdown(&mut self) -> u8 {
        self.dash_countdown = self.dash_countdown.wrapping_add(1);
        self.dash_countdown
    }

    pub(super) fn decrement_dash_countdown(&mut self) -> u8 {
        self.dash_countdown = self.dash_countdown.wrapping_sub(1);
        self.dash_countdown
    }

    pub(super) fn set_dash_counter(&mut self, value: u8) {
        self.dash_counter = value;
    }

    pub(super) fn prime_dash_counter(&mut self) {
        self.dash_counter = 64;
    }

    pub(super) fn decrement_dash_counter_clamped_to_minimum(&mut self, minimum: u8) {
        self.dash_counter = self.dash_counter.wrapping_sub(1);
        if self.dash_counter < minimum {
            self.dash_counter = minimum;
        }
    }

    pub(super) fn set_facing(&mut self, value: u8) {
        self.facing = value;
    }

    pub(super) fn restore_facing_from_cached(&mut self) {
        self.facing = self.cached_facing;
    }

    pub(super) fn set_facing_mirror(&mut self, value: u8) {
        self.facing_mirror = value;
    }

    pub(super) fn cache_facing_to_mirror(&mut self) {
        self.facing_mirror = self.facing;
    }

    pub(super) fn cache_facing(&mut self) {
        self.cached_facing = self.facing;
    }

    pub(super) fn set_moving_against_diag_tile(&mut self, value: u8) {
        self.moving_against_diag_tile = value;
    }

    pub(super) fn add_moving_against_diag_tile_flags(&mut self, value: u8) {
        self.moving_against_diag_tile |= value;
    }

    pub(super) fn clear_moving_against_diag_tile(&mut self) {
        self.moving_against_diag_tile = 0;
    }

    pub(super) fn set_flag_moving(&mut self, value: u8) {
        self.movement_flag = value;
    }

    pub(super) fn clear_flag_moving(&mut self) {
        self.movement_flag = 0;
    }

    pub(super) fn set_quadrants_from_packed_nibbles(&mut self, value: u8) {
        self.quadrant_x = value >> 4;
        self.quadrant_y = value & 0x0f;
    }

    pub(super) fn set_quadrants(&mut self, x: u8, y: u8) {
        self.quadrant_x = x;
        self.quadrant_y = y;
    }

    pub(super) fn toggle_quadrant_x(&mut self) -> u8 {
        self.quadrant_x ^= 1;
        self.quadrant_x
    }

    pub(super) fn toggle_quadrant_y(&mut self) -> u8 {
        self.quadrant_y ^= 2;
        self.quadrant_y
    }

    pub(super) fn reset_direction_limits(&mut self) {
        self.direction_mask_a = 0x0f;
        self.direction_mask_b = 0x0f;
        self.num_orthogonal_directions = 0;
    }

    pub(super) fn reset_direction_masks(&mut self) {
        self.direction_mask_a = 0x0f;
        self.direction_mask_b = 0x0f;
    }

    pub(super) fn increment_orthogonal_direction_count(&mut self) {
        self.num_orthogonal_directions = self.num_orthogonal_directions.wrapping_add(1);
    }

    pub(super) fn clear_orthogonal_direction_count(&mut self) {
        self.num_orthogonal_directions = 0;
    }

    pub(super) fn set_last_direction_moved_towards(&mut self, value: u8) {
        self.last_direction_moved_towards = value;
    }

    pub(super) fn set_last_direction_from_current_direction(&mut self) {
        self.last_direction = self.direction;
    }

    pub(super) fn set_last_direction(&mut self, value: u8) {
        self.last_direction = value;
    }

    pub(super) fn mask_last_direction(&mut self, mask: u8) {
        self.last_direction &= mask;
    }

    pub(super) fn set_last_direction_from_swim_flags(&mut self) {
        self.last_direction = self.swim_direction_flags;
    }

    pub(super) fn set_swim_flags_from_last_direction(&mut self) {
        self.swim_direction_flags = self.last_direction;
    }

    pub(super) fn set_direction(&mut self, value: u8) {
        self.direction = value;
    }

    pub(super) fn set_direction_and_last_direction(&mut self, value: u8) {
        self.direction = value;
        self.last_direction = value;
    }

    pub(super) fn set_direction_and_swim_flags(&mut self, value: u8) {
        self.direction = value;
        self.swim_direction_flags = value;
    }

    pub(super) fn mask_direction(&mut self, mask: u8) {
        self.direction &= mask;
    }

    pub(super) fn add_direction_flags(&mut self, flags: u8) {
        self.direction |= flags;
    }

    pub(super) fn clear_direction_flags(&mut self, flags: u8) {
        self.direction &= !flags;
    }

    pub(super) fn clear_direction_lock(&mut self) {
        self.direction_lock = 0;
    }

    pub(super) fn set_direction_lock_bits(&mut self, mask: u8) {
        self.direction_lock |= mask;
    }

    pub(super) fn clear_direction_lock_bits(&mut self, mask: u8) {
        self.direction_lock &= !mask;
    }

    pub(super) fn set_direction_mask_a(&mut self, value: u8) {
        self.direction_mask_a = value;
    }

    pub(super) fn set_direction_mask_b(&mut self, value: u8) {
        self.direction_mask_b = value;
    }

    pub(super) fn apply_direction_masks(&mut self) {
        self.direction &= self.direction_mask_a & self.direction_mask_b;
    }

    pub(super) fn force_direction_from_diag_tile_if_needed(&mut self) {
        if self.direction & 0x0f != 0 && self.moving_against_diag_tile & 0x0f != 0 {
            self.direction = self.moving_against_diag_tile & 0x0f;
        }
    }

    pub(super) fn resolve_orthogonal_direction_count_from_facing(&mut self) {
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

    pub(super) fn mark_moving_floor_direction(&mut self, floor_y: u16, floor_x: u16) {
        if floor_y != 0 {
            self.direction |= if (floor_y as i16).is_negative() { 8 } else { 4 };
        }
        if floor_x != 0 {
            self.direction |= if (floor_x as i16).is_negative() { 2 } else { 1 };
        }
    }

    pub(super) fn set_last_direction_moved_towards_from_facing(&mut self) {
        self.last_direction_moved_towards = self.facing >> 1;
    }

    pub(super) fn set_swim_direction_flags(&mut self, direction: u8) {
        self.swim_direction_flags = direction;
    }

    pub(super) fn set_y(&mut self, value: u16) {
        self.y = value;
    }

    pub(super) fn set_x(&mut self, value: u16) {
        self.x = value;
    }

    pub(super) fn set_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub(super) fn store_safe_return_position(&mut self, x: u16, y: u16) {
        self.safe_return_x = x;
        self.safe_return_y = y;
    }

    pub(super) fn store_safe_return_y(&mut self, y: u16) {
        self.safe_return_y = y;
    }

    pub(super) fn set_safe_return_y_low(&mut self, value: u8) {
        self.safe_return_y = (self.safe_return_y & 0xff00) | u16::from(value);
    }

    pub(super) fn store_safe_return_low_from_current(&mut self) {
        self.safe_return_y = (self.safe_return_y & 0xff00) | u16::from(self.y_low());
        self.safe_return_x = (self.safe_return_x & 0xff00) | u16::from(self.x_low());
    }

    pub(super) fn cache_safe_return_position_from_current(&mut self) {
        self.store_safe_return_position(self.x, self.y);
    }

    pub(super) fn cache_safe_return_high_from_current(&mut self) {
        self.safe_return_x = (self.safe_return_x & 0x00ff) | (self.x & 0xff00);
        self.safe_return_y = (self.safe_return_y & 0x00ff) | (self.y & 0xff00);
    }

    pub(super) fn clear_page_movement_deltas(&mut self) {
        self.x_page_movement_delta = 0;
        self.y_page_movement_delta = 0;
    }

    pub(super) fn set_page_movement_deltas(&mut self, y_delta: u8, x_delta: u8) {
        self.y_page_movement_delta = y_delta;
        self.x_page_movement_delta = x_delta;
    }

    pub(super) fn set_y_page_movement_delta_from_high_position(&mut self, high: u8) {
        self.y_page_movement_delta = high.wrapping_sub(self.safe_return_y_high());
    }

    pub(super) fn set_x_page_movement_delta_from_high_position(&mut self, high: u8) {
        self.x_page_movement_delta = high.wrapping_sub(self.safe_return_x_high());
    }

    pub(super) fn set_x_with_subpixel(&mut self, x: u16, x_subpixel: u8) {
        self.x = x;
        self.x_subpixel = x_subpixel;
    }

    pub(super) fn set_y_with_subpixel(&mut self, y: u16, y_subpixel: u8) {
        self.y = y;
        self.y_subpixel = y_subpixel;
    }

    pub(super) fn set_y_low(&mut self, value: u8) {
        self.y = (self.y & 0xff00) | u16::from(value);
    }

    pub(super) fn set_x_low(&mut self, value: u8) {
        self.x = (self.x & 0xff00) | u16::from(value);
    }

    pub(super) fn set_x_velocity(&mut self, value: u8) {
        self.x_velocity = value;
    }

    pub(super) fn set_y_velocity(&mut self, value: u8) {
        self.y_velocity = value;
    }

    pub(super) fn set_movement_velocity_from_delta(&mut self, x_delta: u16, y_delta: u16) {
        self.x_velocity = x_delta as u8;
        self.y_velocity = y_delta as u8;
    }

    pub(super) fn subtract_axis_velocity_delta(&mut self, horizontal: bool, delta: u8) {
        if horizontal {
            self.x_velocity = self.x_velocity.wrapping_sub(delta);
        } else {
            self.y_velocity = self.y_velocity.wrapping_sub(delta);
        }
    }

    pub(super) fn add_movement_velocity_delta(&mut self, x_delta: u16, y_delta: u16) {
        self.x_velocity = self.x_velocity.wrapping_add(x_delta as u8);
        self.y_velocity = self.y_velocity.wrapping_add(y_delta as u8);
    }

    pub(super) fn add_y_velocity_delta(&mut self, y_delta: u8) {
        self.y_velocity = self.y_velocity.wrapping_add(y_delta);
    }

    pub(super) fn clear_movement_velocity(&mut self) {
        self.x_velocity = 0;
        self.y_velocity = 0;
    }

    pub(super) fn clear_movement_subpixels(&mut self) {
        self.x_subpixel = 0;
        self.y_subpixel = 0;
    }

    pub(super) fn set_actual_x_velocity(&mut self, value: u8) {
        self.actual_x_velocity = value;
    }

    pub(super) fn set_actual_y_velocity(&mut self, value: u8) {
        self.actual_y_velocity = value;
    }

    pub(super) fn clear_actual_x_velocity(&mut self) {
        self.actual_x_velocity = 0;
    }

    pub(super) fn clear_actual_y_velocity(&mut self) {
        self.actual_y_velocity = 0;
    }

    pub(super) fn set_actual_velocity_xy(&mut self, x: u8, y: u8) {
        self.actual_x_velocity = x;
        self.actual_y_velocity = y;
    }

    pub(super) fn clear_actual_velocity_xy(&mut self) {
        self.set_actual_velocity_xy(0, 0);
    }

    pub(super) fn invert_actual_velocity_xy(&mut self) {
        self.actual_x_velocity = (-(self.actual_x_velocity as i8)) as u8;
        self.actual_y_velocity = (-(self.actual_y_velocity as i8)) as u8;
    }

    pub(super) fn xor_actual_velocity_xy(&mut self, mask: u8) {
        self.actual_x_velocity ^= mask;
        self.actual_y_velocity ^= mask;
    }

    pub(super) fn set_actual_velocity_from_direction(&mut self, direction: u8, velocity: u8) {
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

    pub(super) fn set_z(&mut self, value: u16) {
        self.z = value;
    }

    pub(super) fn set_z_low(&mut self, value: u8) {
        self.z = (self.z & 0xff00) | u16::from(value);
    }

    pub(super) fn clear_z_high(&mut self) {
        self.z &= 0x00ff;
    }

    pub(super) fn set_z_mirror(&mut self, value: u16) {
        self.z_mirror = value;
    }

    pub(super) fn restore_z_low_from_mirror(&mut self) {
        self.set_z_low(self.z_mirror_low());
    }

    pub(super) fn restore_z_from_mirror(&mut self) {
        self.z = self.z_mirror;
    }

    pub(super) fn cache_z_low_to_mirror(&mut self) {
        self.z_mirror = (self.z_mirror & 0xff00) | u16::from(self.z_low());
    }

    pub(super) fn cache_z_to_mirror(&mut self) {
        self.z_mirror = self.z;
    }

    pub(super) fn clear_z_mirror_low(&mut self) {
        self.z_mirror &= 0xff00;
    }

    pub(super) fn clear_z_mirror_word_low(&mut self) {
        self.clear_z_mirror_low();
    }

    pub(super) fn force_z_mirror_low_ff(&mut self) {
        self.z_mirror |= 0x00ff;
    }

    pub(super) fn set_z_and_mirror(&mut self, value: u16) {
        self.z = value;
        self.z_mirror = value;
    }

    pub(super) fn set_actual_z_velocity(&mut self, value: u8) {
        self.z_velocity = value;
    }

    pub(super) fn set_actual_z_velocity_and_copy(&mut self, value: u8) {
        self.z_velocity = value;
        self.z_velocity_copy = value;
    }

    pub(super) fn set_actual_z_velocity_mirror_and_copy(&mut self, value: u8) {
        self.z_velocity_mirror = value;
        self.z_velocity_copy_mirror = value;
    }

    pub(super) fn restore_actual_z_velocity_from_mirror(&mut self) {
        self.z_velocity = self.z_velocity_mirror;
        self.z_velocity_copy = self.z_velocity_copy_mirror;
    }

    pub(super) fn cache_actual_z_velocity_to_mirror(&mut self) {
        self.z_velocity_mirror = self.z_velocity;
        self.z_velocity_copy_mirror = self.z_velocity_copy;
    }

    pub(super) fn prime_airborne_z_velocity(&mut self) {
        self.z_velocity = 0xff;
        self.z = 0xffff;
    }

    pub(super) fn decrement_actual_z_velocity(&mut self, delta: u8) {
        self.z_velocity = self.z_velocity.wrapping_sub(delta);
    }

    pub(super) fn clear_running(&mut self) {
        self.running = 0;
    }

    pub(super) fn start_running(&mut self) {
        self.running = 1;
    }

    pub(super) fn set_running_state(&mut self, value: u8) {
        self.running = value;
    }

    pub(super) fn cancel_dash_state(&mut self) {
        self.dash_countdown = 0;
        self.speed_setting = 0;
        self.running = 0;
        self.direction_lock = 0;
    }

    pub(super) fn set_recoil_timer(&mut self, value: u8) {
        self.recoil_timer = value;
    }

    pub(super) fn increment_recoil_timer(&mut self) -> u8 {
        self.recoil_timer = self.recoil_timer.wrapping_add(1);
        self.recoil_timer
    }

    pub(super) fn set_tile_below(&mut self, value: u8) {
        self.tile_below = value;
    }

    pub(super) fn set_tile_action_index(&mut self, value: u8) {
        self.tile_action_index = value;
    }

    pub(super) fn set_tile_coll_flag(&mut self, value: u8) {
        self.tile_collision_flag = value;
    }

    pub(super) fn set_force_move_any_direction(&mut self, value: u16) {
        self.force_move_any_direction = value;
    }

    pub(super) fn clear_conveyor_belt_state(&mut self) {
        self.conveyor_belt_state = 0;
    }

    pub(super) fn set_conveyor_belt_state(&mut self, value: u8) {
        self.conveyor_belt_state = value;
    }

    pub(super) fn tick_jump_ledge_timer_or_reset(&mut self) -> bool {
        self.jump_ledge_timer = self.jump_ledge_timer.wrapping_sub(1);
        if (self.jump_ledge_timer as i8).is_negative() {
            self.jump_ledge_timer = 19;
            true
        } else {
            false
        }
    }

    pub(super) fn reset_jump_ledge_timer(&mut self) {
        self.jump_ledge_timer = 19;
    }

    pub(super) fn clear_about_to_jump_off_ledge(&mut self) {
        self.about_to_jump_off_ledge = 0;
    }

    pub(super) fn increment_about_to_jump_off_ledge(&mut self) {
        self.about_to_jump_off_ledge = self.about_to_jump_off_ledge.wrapping_add(1);
    }

    pub(super) fn decrement_push_fatigue_timer(&mut self) -> u8 {
        self.push_fatigue_timer = self.push_fatigue_timer.wrapping_sub(1);
        self.push_fatigue_timer
    }

    pub(super) fn set_push_fatigue_timer(&mut self, value: u8) {
        self.push_fatigue_timer = value;
    }

    pub(super) fn reset_push_fatigue_timer(&mut self) {
        self.push_fatigue_timer = 32;
    }

    pub(super) fn clear_near_moveable_statue(&mut self) {
        self.near_moveable_statue_flag = 0;
    }

    pub(super) fn mark_near_moveable_statue(&mut self) {
        self.near_moveable_statue_flag = 1;
    }

    pub(super) fn clear_pit_correction(&mut self) {
        self.pit_correction_active = 0;
    }

    pub(super) fn set_pit_correction_active(&mut self) {
        self.pit_correction_active = 1;
    }

    pub(super) fn set_pit_correction_timer(&mut self, value: u8) {
        self.pit_correction_timer = value;
    }

    pub(super) fn increment_pit_correction_timer(&mut self) {
        self.pit_correction_timer = self.pit_correction_timer.wrapping_add(1);
    }

    pub(super) fn set_moving_against_diag_deadlocked(&mut self, value: u8) {
        self.moving_against_diag_deadlocked = value;
    }

    pub(super) fn clear_misc_bugfix_movement_state(&mut self) {
        self.clear_about_to_jump_off_ledge();
        self.clear_near_moveable_statue();
        self.clear_conveyor_belt_state();
        self.clear_flag_moving();
    }

    pub(super) fn clear_somaria_platform_state(&mut self) {
        self.somaria_platform_state = 0;
    }

    pub(super) fn set_somaria_platform_state(&mut self, value: u8) {
        self.somaria_platform_state = value;
    }

    pub(super) fn clear_doorway_state(&mut self) {
        self.doorway_state = 0;
    }

    pub(super) fn set_doorway_state(&mut self, value: u8) {
        self.doorway_state = value;
    }

    pub(super) fn clear_swim_fast_state(&mut self) {
        self.swim_fast_state = 0;
    }

    pub(super) fn reset_swimming_state_fields(&mut self) {
        self.swimming_countdown = 0;
        self.hard_swim_stroke = 0;
        self.swim_fast_state = 0;
    }

    pub(super) fn start_hard_swim_stroke(&mut self, hard_stroke: u8) {
        self.hard_swim_stroke = hard_stroke;
        self.swim_fast_state = 1;
        self.swimming_countdown = 7;
    }

    pub(super) fn tick_hard_swim_stroke(&mut self, swimming_countdown: u8) {
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

    pub(super) fn enter_deep_water_state(&mut self) {
        self.deep_water_state = 1;
    }

    pub(super) fn clear_deep_water_state(&mut self) {
        self.deep_water_state = 0;
    }

    pub(super) fn clear_position_mode(&mut self) {
        self.position_mode = 0;
    }

    pub(super) fn set_position_mode(&mut self, value: u8) {
        self.position_mode = value;
    }

    pub(super) fn set_position_mode_bits(&mut self, mask: u8) {
        self.position_mode |= mask;
    }

    pub(super) fn clear_position_mode_bits(&mut self, mask: u8) {
        self.position_mode &= !mask;
    }

    pub(super) fn clear_grabbing_wall(&mut self) {
        self.grabbing_wall = 0;
    }

    pub(super) fn set_grabbing_wall(&mut self, value: u8) {
        self.grabbing_wall = value;
    }

    pub(super) fn set_near_pit_state(&mut self, value: u8) {
        self.near_pit_state = value;
    }

    pub(super) fn clear_near_pit_state(&mut self) {
        self.near_pit_state = 0;
    }

    pub(super) fn set_pit_data_index(&mut self, value: u8) {
        self.pit_data_index = value;
    }

    pub(super) fn clear_pit_data_index(&mut self) {
        self.pit_data_index = 0;
    }

    pub(super) fn advance_pit_data_index(&mut self) -> u8 {
        self.pit_data_index = self.pit_data_index.wrapping_add(1);
        self.pit_data_index
    }

    pub(super) fn begin_pit_check(&mut self) {
        self.clear_pit_data_index();
        self.set_near_pit_state(1);
    }

    pub(super) fn cache_current_quadrants(&mut self) {
        self.cached_quadrant_x = self.quadrant_x;
        self.cached_quadrant_y = self.quadrant_y;
    }

    pub(super) fn restore_quadrants_from_cached(&mut self) {
        self.quadrant_x = self.cached_quadrant_x;
        self.quadrant_y = self.cached_quadrant_y;
    }

    pub(super) fn set_recoil_z_velocity_for_dungeon_reset(&mut self, value: u8) {
        self.recoil_z_velocity_for_dungeon_reset = value;
    }

    pub(super) fn set_recoil_z_velocity(&mut self, value: u8) {
        self.recoil_z_velocity_for_dungeon_reset = value;
        self.z_velocity = value;
    }

    pub(super) fn set_whirlpool_trigger(&mut self) {
        self.whirlpool_trigger = 1;
    }

    pub(super) fn clear_whirlpool_trigger(&mut self) {
        self.whirlpool_trigger = 0;
    }

    pub(super) fn prevent_movement(&mut self) {
        self.prevent_movement = 1;
    }

    pub(super) fn clear_prevent_movement(&mut self) {
        self.prevent_movement = 0;
    }

    pub(super) fn set_swim_stroke_frame_counter(&mut self, offset: usize, value: u16) {
        let axis =
            swim_axis_index(offset).expect("swim stroke frame counter offset must be 0 or 2");
        self.swim_stroke_frame_counters[axis] = value;
    }

    pub(super) fn set_bit9_of_xcoord_word(&mut self, value: u16) {
        self.bit9_of_xcoord = value;
    }

    pub(super) fn set_hop_origin_delta_from_y(&mut self, y: u16) -> u16 {
        self.hop_origin_coord = self.hop_origin_coord.wrapping_sub(y);
        self.hop_origin_coord
    }

    pub(super) fn set_movement_velocity_from_position_delta(
        &mut self,
        x: u16,
        y: u16,
        old_x: u16,
        old_y: u16,
    ) {
        self.y_velocity = y.wrapping_sub(old_y) as u8;
        self.x_velocity = x.wrapping_sub(old_x) as u8;
    }

    pub(super) fn clear_actual_velocity_and_page_movement_deltas(&mut self) {
        self.actual_x_velocity = 0;
        self.actual_y_velocity = 0;
        self.x_page_movement_delta = 0;
        self.y_page_movement_delta = 0;
    }

    pub(super) fn cache_moving_floor_position(&mut self, x: u16, y: u16) {
        self.moving_floor_x = x;
        self.moving_floor_y = y;
    }

    pub(super) fn decrement_incapacitated_camera_timer(&mut self) -> u8 {
        self.incapacitated_camera_timer = self.incapacitated_camera_timer.wrapping_sub(1);
        self.incapacitated_camera_timer
    }

    pub(super) fn clear_swim_movement_velocity(&mut self) {
        self.y_velocity = 0;
        self.x_velocity = 0;
    }

    pub(super) fn set_cached_tile_action_index(&mut self, value: u8) {
        self.cached_tile_action_index = value;
    }

    pub(super) fn clear_swimming_countdown(&mut self) {
        self.swimming_countdown = 0;
    }

    pub(super) fn clear_force_move_high_byte(&mut self) {
        self.force_move_any_direction = u16::from(self.force_move_any_direction as u8);
    }

    pub(super) fn set_hop_origin_coord(&mut self, value: u16) {
        self.hop_origin_coord = value;
    }

    pub(super) fn set_previous_position(&mut self, x: u16, y: u16) {
        self.previous_x = x;
        self.previous_y = y;
    }

    pub(super) fn cache_previous_position_from_current(&mut self) {
        self.previous_x = self.x;
        self.previous_y = self.y;
    }

    pub(super) fn set_drag_player_x(&mut self, value: u16) {
        self.drag_player_x = value;
    }

    pub(super) fn set_drag_player_y(&mut self, value: u16) {
        self.drag_player_y = value;
    }

    pub(super) fn add_drag_player_x(&mut self, delta: u16) {
        self.drag_player_x = self.drag_player_x.wrapping_add(delta);
    }

    pub(super) fn add_drag_player_y(&mut self, delta: u16) {
        self.drag_player_y = self.drag_player_y.wrapping_add(delta);
    }

    pub(super) fn set_gravestone_push_timeout(&mut self, value: u8) {
        self.gravestone_push_timeout = value;
    }

    pub(super) fn decrement_gravestone_push_timeout(&mut self) {
        self.gravestone_push_timeout = self.gravestone_push_timeout.wrapping_sub(1);
    }

    pub(super) fn enter_deep_water(&mut self) {
        self.deep_water_state = 1;
        self.swim_direction_flags = self.last_direction;
        self.grabbing_wall = 0;
        self.speed_setting = 0;
    }
}
