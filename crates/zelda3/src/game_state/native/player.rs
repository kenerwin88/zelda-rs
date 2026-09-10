// The facade retains read-only access and the existing mutation boundary.
// Forwarding carries no storage, import, or publication side effects.
macro_rules! forward_player_component {
    ($field:ident; $($vis:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)?;)+) => {
        $(forward_player_component!(@method $field; $vis fn $name($($args)*) $(-> $ret)?);)+
    };
    (@method $field:ident; $vis:vis fn $name:ident(&self $(, $arg:ident: $ty:ty)* $(,)?) $(-> $ret:ty)?) => {
        $vis fn $name(&self, $($arg: $ty),*) $(-> $ret)? { self.$field.$name($($arg),*) }
    };
    (@method $field:ident; $vis:vis fn $name:ident(&mut self $(, $arg:ident: $ty:ty)* $(,)?) $(-> $ret:ty)?) => {
        $vis fn $name(&mut self, $($arg: $ty),*) $(-> $ret)? { self.$field.$name($($arg),*) }
    };
}

mod collision;
pub(crate) use collision::{
    CollisionAxis, CollisionDirection, CollisionOrder, MovementProbe, MovementProbeKind,
    PlayerFootprint,
};
mod compatibility;
mod tile_behavior;
pub(crate) use tile_behavior::TileResult;
mod transitions;
pub(crate) use compatibility::NativeFollowerLinkBridgeMut;

mod input;
use crate::game_state::native::ram_target::RamTarget;
use input::PlayerInputState;
mod presentation;
use presentation::PlayerPresentationState;
mod motion;
pub(crate) use motion::PlayerAxis;
use motion::PlayerPosition;
mod movement;
use movement::PlayerMovementState;
mod actions;
use actions::PlayerActionsState;

use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const PUSHED_BLOCK_BANK_LEN: usize = 4;
const SWIM_AXIS_COUNT: usize = 2;
const FALL_HOLE_SCAN_INDEX_LOCAL: usize = 0x02c9;

fn swim_axis_index(offset: usize) -> Option<usize> {
    match offset {
        0 => Some(0),
        2 => Some(1),
        _ => None,
    }
}

// ATTRIBUTES_FOR_TILE (0xfe00) owns exactly 0x200 bytes and is solely owned by
// DungeonBg2AttributeState, which reads it straight out of RAM like C does
// (`attributes_for_tile[tile & 0x3ff]`). A second 0x400-wide native mirror lived here and
// bulk-projected 0xfe00..0x101ff every frame; because `player` projects after `dungeon`
// in GameState::write_to_ram it re-stamped a stale frame-start copy over 0x10000..0x101ff,
// which C uses for messaging_buf / blastwall_var* / skullwoodsfire_var*. Same overrun the
// dungeon-side copy was already sized down to 0x200 to fix; do not reintroduce a mirror.

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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(LINK_X_COORD_SPEXIT, self.x);
        ram.write_word(LINK_Y_COORD_SPEXIT, self.y);
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
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        self.special_exit_position.write_to_ram(ram);
        self.follower_link.write_to_ram(ram);
        self.swim_acceleration.write_to_ram(ram);
        self.pushed_block.write_to_ram(ram);
        self.bg1_movement_accumulator.write_to_ram(ram);
        self.tile_detection.write_to_ram(ram);
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
    input: PlayerInputState,
    presentation: PlayerPresentationState,
    movement: PlayerMovementState,
    actions: PlayerActionsState,
}

impl FollowerLinkState {
    pub(crate) fn oam_priority_for_floor_value(floor: u8) -> u8 {
        FOLLOWER_LAYER_BITS_BY_FLOOR[floor as usize]
    }

    pub(crate) fn has_item_or_position_mode(&self) -> bool {
        self.actions.item_in_hand | self.movement.position_mode != 0
    }

    pub(crate) fn is_ready_to_start_ground_movement(&self) -> bool {
        (self.movement.grabbing_wall & !2) == 0
            && !self.has_non_lift_action_state()
            && (!self.is_lifting_or_carrying() || self.actions.picking_throw_state & 1 == 0)
            && !self.has_item_or_position_mode()
    }

    pub(crate) fn button_b_frames_word(&self) -> u16 {
        // The ether/bombos cutscene reinterprets BUTTON_B_FRAMES (0x3c) and the
        // adjacent LINK_DELAY_TIMER_SPIN_ATTACK (0x3d) as one 16-bit counter.
        u16::from(self.input.button_b_frames)
            | (u16::from(self.actions.spin_attack_delay_timer) << 8)
    }

    pub(crate) fn can_open_follower_message(&self) -> bool {
        let blocked = (self.input.button_mask_b_y & 0x80)
            | self.actions.pull_action_state
            | self.actions.item_in_hand
            | self.movement.position_mode
            | self.actions.ancilla_pickup_flag
            | self.actions.sprite_pickup_flag
            | self.actions.action_state_bits
            | self.movement.grabbing_wall;
        self.is_ground_swim_or_dash_start() && blocked == 0
    }

    pub(crate) fn can_reacquire_old_man(&self) -> bool {
        !self.is_running() && !self.has_auxiliary_state() && !self.is_swimming()
    }

    fn reset_swim_subpixel_and_defense_state(&mut self) {
        self.movement.clear_movement_subpixels();
        self.movement.moving_against_diag_tile = 0;
        self.actions.defense_flags = 0;
    }

    fn reset_incapacitated_camera_timer_from_incapacitated(&mut self) {
        self.movement.incapacitated_camera_timer = self.actions.incapacitated_timer >> 4;
    }

    fn set_button_b_frames_word(&mut self, value: u16) {
        // Word access spans the independently-owned spin-attack delay timer (0x3d).
        self.input.button_b_frames = value as u8;
        self.actions.spin_attack_delay_timer = (value >> 8) as u8;
    }

    fn decrement_button_b_frames_word(&mut self) -> u16 {
        let value = self.button_b_frames_word().wrapping_sub(1);
        self.set_button_b_frames_word(value);
        value
    }

    fn clear_action_scratch_state(&mut self) {
        self.actions.item_action_debug_value_2 = 0;
        self.actions.item_action_step = 0;
        self.presentation.throw_oam_state_index = 0;
    }

    fn clear_lift_throw_scratch_state(&mut self) {
        self.actions.item_action_step = 0;
        self.presentation.throw_oam_state_index = 0;
    }

    fn start_bunny_transform_poof(&mut self) {
        self.actions.sprite_damage_disabled = 1;
        self.actions.transform_poof_needed = 1;
        self.presentation.visibility_status = 12;
    }

    fn finish_bunny_transform_poof(&mut self) {
        self.actions.bunny_mirror = 1;
        self.actions.bunny_state = 1;
        self.presentation.visibility_status = 0;
        self.actions.sprite_damage_disabled = 0;
        self.actions.transform_poof_needed = 0;
    }

    fn enter_item_hold_pose(&mut self) {
        self.actions.action_state_bits = 0x80;
        self.actions.picking_throw_state = 0;
        self.movement.facing = 0;
        self.presentation.animation_step = 0;
    }

    fn clear_state_item_and_grab_flags(&mut self) {
        self.actions.action_state_bits = 0;
        self.actions.picking_throw_state = 0;
        self.movement.grabbing_wall = 0;
    }

    fn clear_swimming_action_state(&mut self) {
        self.input.button_mask_b_y = 0;
        self.input.clear_button_b_frames();
        self.actions.spin_attack_delay_timer = 0;
        self.actions.spin_attack_step_counter = 0;
        self.actions.action_state_bits = 0;
        self.actions.picking_throw_state = 0;
    }

    fn interrupt_swimming_for_auxiliary_state(&mut self) {
        self.actions.handler_state = 2;
        self.movement.z &= 0x00ff;
        self.movement.swim_fast_state = 0;
        self.movement.hard_swim_stroke = 0;
        self.movement.direction_lock &= !1;
    }

    fn reset_idle_swim_animation_if_out_of_water(&mut self) {
        if self.actions.handler_state != PLAYER_HANDLER_STATE_SWIMMING {
            self.presentation.animation_step = 0;
        }
    }

    fn setup_bed_pose(&mut self) {
        self.actions.handler_state = 0x16;
        self.presentation.sleep_in_bed_state = 0;
        self.presentation.opening_pose = 0;
        self.movement.dash_countdown = 3;
    }

    fn reset_after_damaging_pit(&mut self, handler_state: u8) {
        self.actions.handler_state = handler_state;
        self.movement.last_direction = self.movement.swim_direction_flags;
        self.movement.deep_water_state = 0;
        self.actions.sprite_damage_disabled = 0;
        self.movement.pit_data_index = 0;
        self.movement.near_pit_state = 0;
    }

    fn recache_bunny_state(&mut self, has_moon_pearl: bool) {
        self.actions.transform_poof_needed = 0;
        self.actions.temp_bunny_timer = 0;
        if has_moon_pearl {
            self.actions.bunny_state = 0;
            self.actions.auxiliary_state = 0;
        }
        self.presentation.animation_step = 0;
        self.actions.transforming = 0;
        self.movement.direction_lock = 0;
    }

    forward_player_component! { input;
        pub(crate) fn button_mask_b_y(&self) -> u8;
        pub(crate) fn filtered_joypad_h(&self) -> u8;
        pub(crate) fn filtered_joypad_l(&self) -> u8;
        pub(crate) fn joypad1h_last(&self) -> u8;
        pub(crate) fn joypad1l_last(&self) -> u8;
        pub(crate) fn joypad1h_last2(&self) -> u8;
        pub(crate) fn joypad1l_last2(&self) -> u8;
        pub(crate) fn button_b_frames(&self) -> u8;
        pub(crate) fn button_b_frames_index(&self) -> usize;
    }

    forward_player_component! { presentation;
        pub(crate) fn oam_x_offset(&self) -> u8;
        pub(crate) fn oam_y_offset(&self) -> u8;
        pub(crate) fn oam_x_offset_signed(&self) -> i8;
        pub(crate) fn oam_y_offset_signed(&self) -> i8;
        pub(crate) fn has_disabled_oam_offsets(&self) -> bool;
        pub(crate) fn item_hold_pose(&self) -> u8;
        pub(crate) fn force_hold_sword_up_state(&self) -> u8;
        pub(crate) fn visibility_status(&self) -> u8;
        pub(crate) fn link_dma_graphics_index_word(&self) -> u16;
        pub(crate) fn link_dma_staging_index(&self) -> u8;
        pub(crate) fn blink_countdown(&self) -> u8;
        pub(crate) fn spin_animation_step_counter(&self) -> u8;
        pub(crate) fn animation_step(&self) -> u8;
        pub(crate) fn opening_pose(&self) -> u8;
        pub(crate) fn animation_step_index(&self) -> usize;
        pub(crate) fn water_ripple_or_grass_state(&self) -> u8;
        pub(crate) fn primary_water_grass_timer(&self) -> u8;
        pub(crate) fn secondary_water_grass_timer(&self) -> u8;
        pub(crate) fn sleep_in_bed_state(&self) -> u8;
        pub(crate) fn throw_oam_state_index(&self) -> u8;
        pub(crate) fn sword_dma_graphics_index(&self) -> u8;
        pub(crate) fn shield_dma_graphics_index(&self) -> u8;
        pub(crate) fn link_dma_left_sprite_bank_word(&self) -> u16;
        pub(crate) fn link_dma_right_sprite_bank_word(&self) -> u16;
        pub(crate) fn link_dma_staging_group(&self) -> u8;
        pub(crate) fn palette_bits_of_oam(&self) -> u8;
        pub(crate) fn palette_bits_of_oam_word(&self) -> u16;
        pub(crate) fn player_pose_draw_counter(&self) -> u8;
        pub(crate) fn player_special_draw_flag(&self) -> u8;
        pub(crate) fn spin_offsets(&self) -> u8;
        pub(crate) fn dash_noise_requested(&self) -> bool;
        pub(crate) fn faint_animation_active(&self) -> u8;
        pub(crate) fn index_of_dashing_sfx(&self) -> u8;
    }

    forward_player_component! { movement;
        pub(crate) fn x(&self) -> u16;
        pub(crate) fn y(&self) -> u16;
        pub(crate) fn x_low(&self) -> u8;
        pub(crate) fn y_low(&self) -> u8;
        pub(crate) fn safe_return_x_high(&self) -> u8;
        pub(crate) fn safe_return_y_high(&self) -> u8;
        pub(crate) fn safe_return_y_low(&self) -> u8;
        pub(crate) fn y_low_delta_from_safe_return(&self) -> u8;
        pub(crate) fn safe_return_x(&self) -> u16;
        pub(crate) fn safe_return_y(&self) -> u16;
        pub(crate) fn x_high(&self) -> u8;
        pub(crate) fn y_high(&self) -> u8;
        pub(crate) fn z(&self) -> u16;
        pub(crate) fn z_low(&self) -> u8;
        pub(crate) fn is_z_low_negative(&self) -> bool;
        pub(crate) fn z_mirror_delta_low(&self) -> u8;
        pub(crate) fn is_grounded_or_z_sentinel(&self) -> bool;
        pub(crate) fn is_landing_at_or_above_ground(&self) -> bool;
        pub(crate) fn is_low_z_landing_at_or_above_ground(&self) -> bool;
        pub(crate) fn is_recoil_landing_z_window(&self) -> bool;
        pub(crate) fn should_probe_recoil_landing_tile(&self) -> bool;
        pub(crate) fn z_for_follow(&self) -> u8;
        pub(crate) fn z_for_oam(&self) -> u8;
        pub(crate) fn is_moving(&self) -> bool;
        pub(crate) fn x_velocity(&self) -> u8;
        pub(crate) fn x_velocity_signed(&self) -> i8;
        pub(crate) fn y_velocity(&self) -> u8;
        pub(crate) fn y_velocity_signed(&self) -> i8;
        pub(crate) fn actual_z_velocity(&self) -> u8;
        pub(crate) fn recoil_z_velocity_for_dungeon_reset(&self) -> u8;
        pub(crate) fn actual_z_velocity_copy(&self) -> u8;
        pub(crate) fn actual_z_velocity_mirror(&self) -> u8;
        pub(crate) fn x_page_movement_delta(&self) -> u8;
        pub(crate) fn y_page_movement_delta(&self) -> u8;
        pub(crate) fn x_page_movement_delta_signed(&self) -> i8;
        pub(crate) fn y_page_movement_delta_signed(&self) -> i8;
        pub(crate) fn actual_x_velocity(&self) -> u8;
        pub(crate) fn actual_x_velocity_signed(&self) -> i8;
        pub(crate) fn actual_y_velocity(&self) -> u8;
        pub(crate) fn actual_y_velocity_signed(&self) -> i8;
        pub(crate) fn floor(&self) -> u8;
        pub(crate) fn is_on_lower_level(&self) -> bool;
        pub(crate) fn lower_level_tilemap_offset(&self) -> u16;
        pub(crate) fn has_lower_level_state_or_mirror(&self) -> bool;
        pub(crate) fn lower_level_state(&self) -> u8;
        pub(crate) fn lower_level_mirror_state(&self) -> u8;
        pub(crate) fn floor_layer_bits(&self) -> u8;
        pub(crate) fn oam_priority_for_floor(&self) -> u8;
        pub(crate) fn direction(&self) -> u8;
        pub(crate) fn direction_lock(&self) -> u8;
        pub(crate) fn direction_lock_has(&self, mask: u8) -> bool;
        pub(crate) fn moving_against_diag_tile(&self) -> u8;
        pub(crate) fn flag_moving(&self) -> u8;
        pub(crate) fn quadrant_x(&self) -> u8;
        pub(crate) fn quadrant_y(&self) -> u8;
        pub(crate) fn quadrant_x_mask(&self) -> u8;
        pub(crate) fn quadrant_y_mask(&self) -> u8;
        pub(crate) fn is_moving_against_diag_tile_on_both_axes(&self) -> bool;
        pub(crate) fn has_swim_axis_drag(&self) -> bool;
        pub(crate) fn num_orthogonal_directions(&self) -> u8;
        pub(crate) fn last_direction_moved_towards(&self) -> u8;
        pub(crate) fn last_direction_moved_towards_index(&self) -> usize;
        pub(crate) fn last_direction(&self) -> u8;
        pub(crate) fn facing(&self) -> u8;
        pub(crate) fn has_facing(&self) -> bool;
        pub(crate) fn facing_index(&self) -> usize;
        pub(crate) fn facing_mirror_index(&self) -> usize;
        pub(crate) fn facing_layer_bits(&self) -> u8;
        pub(crate) fn swim_direction_flags(&self) -> u8;
        pub(crate) fn speed_setting(&self) -> u8;
        pub(crate) fn speed_modifier(&self) -> u8;
        pub(crate) fn dash_counter(&self) -> u8;
        pub(crate) fn dash_countdown(&self) -> u8;
        pub(crate) fn about_to_jump_off_ledge(&self) -> u8;
        pub(crate) fn push_fatigue_timer(&self) -> u8;
        pub(crate) fn gravestone_push_timeout(&self) -> u8;
        pub(crate) fn is_running(&self) -> bool;
        pub(crate) fn running_state(&self) -> u8;
        pub(crate) fn has_position_mode(&self) -> bool;
        pub(crate) fn position_mode(&self) -> u8;
        pub(crate) fn position_mode_has(&self, mask: u8) -> bool;
        pub(crate) fn has_grabbing_wall_state(&self) -> bool;
        pub(crate) fn grabbing_wall(&self) -> u8;
        pub(crate) fn grabbing_wall_has(&self, mask: u8) -> bool;
        pub(crate) fn hop_origin_coord(&self) -> u16;
        pub(crate) fn drag_player_x(&self) -> u16;
        pub(crate) fn drag_player_y(&self) -> u16;
        pub(crate) fn on_somaria_platform(&self) -> u8;
        pub(crate) fn has_somaria_platform_state(&self) -> bool;
        pub(crate) fn near_pit_state(&self) -> u8;
        pub(crate) fn is_near_pit(&self) -> bool;
        pub(crate) fn near_pit_state_is(&self, value: u8) -> bool;
        pub(crate) fn near_pit_state_at_least(&self, value: u8) -> bool;
        pub(crate) fn pit_data_index(&self) -> u8;
        pub(crate) fn pit_correction_timer(&self) -> u8;
        pub(crate) fn pit_correction_active(&self) -> bool;
        pub(crate) fn moving_against_diag_deadlocked(&self) -> u8;
        pub(crate) fn doorway_state(&self) -> u8;
        pub(crate) fn deep_water_state(&self) -> u8;
        pub(crate) fn swim_fast_state(&self) -> u8;
        pub(crate) fn hard_swim_stroke(&self) -> u8;
        pub(crate) fn swim_stroke_frame_counter(&self, offset: usize) -> u16;
        pub(crate) fn swim_stroke_anim_step(&self) -> u8;
        pub(crate) fn is_in_deep_water(&self) -> bool;
        pub(crate) fn conveyor_belt_state(&self) -> u8;
        pub(crate) fn tile_below(&self) -> u8;
        pub(crate) fn tile_action_index(&self) -> u8;
        pub(crate) fn tile_coll_flag(&self) -> u8;
        pub(crate) fn whirlpool_triggered(&self) -> bool;
        pub(crate) fn is_prevented_from_moving(&self) -> bool;
        pub(crate) fn force_move_any_direction(&self) -> u16;
        pub(crate) fn cached_x(&self) -> u16;
        pub(crate) fn cached_y(&self) -> u16;
        pub(crate) fn copied_x(&self) -> u16;
        pub(crate) fn copied_y(&self) -> u16;
        pub(crate) fn bit9_of_xcoord(&self) -> u8;
        pub(crate) fn moving_floor_x(&self) -> u16;
        pub(crate) fn moving_floor_y(&self) -> u16;
        pub(crate) fn cheat_walk_through_walls(&self) -> u8;
        pub(crate) fn is_near_moveable_statue(&self) -> bool;
    }

    forward_player_component! { actions;
        pub(crate) fn menu_block_flag(&self) -> u8;
        pub(crate) fn is_menu_blocked(&self) -> bool;
        pub(crate) fn has_menu_block_flag(&self, value: u8) -> bool;
        pub(crate) fn handler_state(&self) -> u8;
        pub(crate) fn is_edge_transition_blocked_by_handler_state(&self) -> bool;
        pub(crate) fn is_ground_swim_or_dash_start(&self) -> bool;
        pub(crate) fn is_using_medallion(&self) -> bool;
        pub(crate) fn is_swimming(&self) -> bool;
        pub(crate) fn is_immobilized(&self) -> bool;
        pub(crate) fn immobilized_flag(&self) -> u8;
        pub(crate) fn is_hookshot(&self) -> bool;
        pub(crate) fn has_action_state(&self) -> bool;
        pub(crate) fn state_bits(&self) -> u8;
        pub(crate) fn state_bits_has(&self, mask: u8) -> bool;
        pub(crate) fn has_non_lift_action_state(&self) -> bool;
        pub(crate) fn is_lifting_or_carrying(&self) -> bool;
        pub(crate) fn auxiliary_state(&self) -> u8;
        pub(crate) fn is_in_auxiliary_state(&self, value: u8) -> bool;
        pub(crate) fn has_auxiliary_state(&self) -> bool;
        pub(crate) fn item_in_hand(&self) -> u8;
        pub(crate) fn has_item_in_hand(&self) -> bool;
        pub(crate) fn item_in_hand_has(&self, mask: u8) -> bool;
        pub(crate) fn picking_throw_state(&self) -> u8;
        pub(crate) fn picking_throw_state_has(&self, mask: u8) -> bool;
        pub(crate) fn has_picking_throw_state(&self) -> bool;
        pub(crate) fn is_lift_throw_primed(&self) -> bool;
        pub(crate) fn spin_attack_delay_timer(&self) -> u8;
        pub(crate) fn spin_attack_step_counter(&self) -> u8;
        pub(crate) fn incapacitated_timer(&self) -> u8;
        pub(crate) fn y_button_action_flags(&self) -> u8;
        pub(crate) fn y_button_action_step(&self) -> u8;
        pub(crate) fn y_button_action_timer(&self) -> u8;
        pub(crate) fn defense_flags(&self) -> u8;
        pub(crate) fn electrocute_on_touch(&self) -> u8;
        pub(crate) fn is_cape_active(&self) -> bool;
        pub(crate) fn cape_decrement_counter(&self) -> u8;
        pub(crate) fn sprite_damage_disable_timer(&self) -> u8;
        pub(crate) fn item_receipt_method(&self) -> u8;
        pub(crate) fn action_handler_timer(&self) -> u8;
        pub(crate) fn is_bunny(&self) -> bool;
        pub(crate) fn is_bunny_mirror(&self) -> bool;
        pub(crate) fn temp_bunny_timer(&self) -> u16;
        pub(crate) fn needs_transform_poof(&self) -> bool;
        pub(crate) fn state_for_spin_attack(&self) -> u8;
        pub(crate) fn spin_attack_sound_latch(&self) -> u8;
        pub(crate) fn item_action_step_var(&self) -> u8;
        pub(crate) fn item_debug_value_1(&self) -> u8;
        pub(crate) fn sprite_pickup_flag_cached(&self) -> u8;
        pub(crate) fn is_transforming(&self) -> bool;
        pub(crate) fn needs_pull_for_rupees_sprite(&self) -> bool;
        pub(crate) fn given_damage(&self) -> u8;
        pub(crate) fn has_pull_action_state(&self) -> bool;
        pub(crate) fn pull_action_state(&self) -> u8;
        pub(crate) fn current_item_y(&self) -> u8;
        pub(crate) fn current_item_active(&self) -> u8;
        pub(crate) fn receive_item_index(&self) -> u8;
        pub(crate) fn item_pickup_in_progress(&self) -> bool;
        pub(crate) fn selected_rod(&self) -> u8;
        pub(crate) fn ancilla_pickup_flag(&self) -> u8;
        pub(crate) fn sprite_pickup_flag(&self) -> u8;
        pub(crate) fn hookshot_interlock(&self) -> u8;
        pub(crate) fn has_hookshot_interlock(&self) -> bool;
        pub(crate) fn hookshot_grave_latch(&self) -> bool;
        pub(crate) fn flute_countdown(&self) -> u8;
        pub(crate) fn hookshot_bg_check_off_timer(&self) -> u8;
        pub(crate) fn hookshot_interlock_has(&self, mask: u8) -> bool;
        pub(crate) fn can_drop_follower(&self) -> bool;
        pub(crate) fn should_transform_old_man_from_recoil(&self) -> bool;
        pub(crate) fn should_transform_old_man_from_auxiliary_state(&self) -> bool;
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
    inroom_staircase: u16,
    fall_hole_scan_index: u8,
}

// R15 is the collision word's high byte; the sparkle garnish spawner writes it
// as raw scratch, so the collision word is that byte's one owner.
const _: () = assert!(
    crate::game_state::constants::SPRITE_LAST_GARNISH_INDEX == TILEDETECT_COLLISION_BITS + 1
);

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
            inroom_staircase: read_le_u16(ram, TILEDETECT_INROOM_STAIRCASE),
            fall_hole_scan_index: ram_byte(ram, FALL_HOLE_SCAN_INDEX_LOCAL),
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_word(TILEDETECT_WHICH_Y_POS, self.probe_y);
        ram.write_word(TILEDETECT_WHICH_Y_POS + 2, self.probe_x);
        ram.write_byte(
            TILE_COLLISION_BITS_PRIMARY,
            self.tile_collision_bits_primary,
        );
        ram.write_byte(
            TILE_COLLISION_BITS_SECONDARY,
            self.tile_collision_bits_secondary,
        );
        ram.write_byte(
            LIFTABLE_TILE_DETECTED_INDEX_DOUBLED,
            self.liftable_tile_index,
        );
        ram.write_byte(
            LIFTABLE_TILE_ACTION_INDEX_PRIMARY,
            self.liftable_action_index_primary,
        );
        ram.write_byte(
            LIFTABLE_TILE_ACTION_INDEX_SECONDARY,
            self.liftable_action_index_secondary,
        );
        ram.write_word(SCRATCH_0, self.interaction_scratch_y);
        ram.write_word(SCRATCH_1, self.interaction_scratch_x);
        ram.write_word(TILEMAP_LOCATION_CALC_MASK, self.location_calc_mask);
        ram.write_word(INDEX_OF_INTERACTING_TILE, self.interacting_tile);
        ram.write_byte(TILEDETECT_PIT_TILE, self.pit_tile as u8);
        ram.write_word(TILEDETECT_DEEPWATER, self.deepwater);
        ram.write_word(TILEDETECT_NORMAL_TILES, self.normal_tiles);
        ram.write_word(TILEDETECT_MISC_TILES, self.misc_tiles);
        ram.write_word(TILEDETECT_THICK_GRASS, self.thick_grass);
        ram.write_word(TILEDETECT_DIAGONAL_TILE, self.diagonal_tile);
        ram.write_byte(TILEDETECT_STAIR_TILE, self.stair_tile);
        ram.write_word(TILEDETECT_BLOCK_FLAGS_LO, self.block_flags);
        ram.write_word(TILEDETECT_DOOR_DIRECTION_FLAGS, self.door_direction_flags);
        ram.write_word(TILEDETECT_DIAG_STATE, self.diag_state);
        ram.write_word(TILEDETECT_MOVING_FLOOR_TILES, self.moving_floor_tiles);
        ram.write_word(TILEDETECT_ICY_FLOOR, self.icy_floor);
        ram.write_word(TILEDETECT_WATER_STAIRCASE, self.water_staircase);
        ram.write_word(TILEDETECT_SHALLOW_WATER, self.shallow_water);
        ram.write_word(TILEDETECT_DESTRUCTION_AFTERMATH, self.destruction_aftermath);
        ram.write_word(TILEDETECT_READ_SOMETHING, self.read_something);
        ram.write_byte(TILEDETECT_VERTICAL_LEDGE, self.vertical_ledge);
        ram.write_byte(
            DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ,
            self.horizontal_ledge,
        );
        ram.write_byte(TILEDETECT_LEDGES_DOWN_LEFTRIGHT, self.ledges_down_leftright);
        ram.write_byte(TILEDETECT_DIAGONAL_LEDGE_TILES, self.diagonal_ledge_tiles);
        ram.write_word(TILEDETECT_CHEST, self.chest);
        ram.write_word(TILEDETECT_KEY_LOCK_GRAVESTONES, self.key_lock_gravestones);
        ram.write_word(TILEDETECT_TILE_TYPE, self.tile_type);
        ram.write_byte(
            TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS,
            self.spike_floor_and_triggers,
        );
        ram.write_byte(BITMASK_FOR_DASHABLE_TILES, self.dashable_tiles);
        ram.write_byte(TILEDETECT_STAIRCASE_CACHE, self.staircase_cache);
        ram.write_word(TILEDETECT_SLOPE_COLLISION_BITS, self.slope_collision_bits);
        ram.write_word(TILEDETECT_COLLISION_BITS, self.collision_bits);
        ram.write_byte(PLAYER_LAYER_COLLISION_FLAGS, self.layer_collision_flags);
        ram.write_word(TILEDETECT_INROOM_STAIRCASE, self.inroom_staircase);
        ram.write_byte(FALL_HOLE_SCAN_INDEX_LOCAL, self.fall_hole_scan_index);
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

    #[cfg(test)]
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

    pub(crate) fn has_layer_collision(&self, mask: u8) -> bool {
        self.layer_collision_flags & mask == mask
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

    #[cfg(test)]
    pub(crate) fn set_diagonal_tile(&mut self, value: u16) {
        self.diagonal_tile = value;
    }

    pub(crate) fn clear_diagonal_tile(&mut self) {
        self.diagonal_tile = 0;
    }

    #[cfg(test)]
    pub(crate) fn or_diagonal_tile(&mut self, value: u16) -> u16 {
        self.diagonal_tile |= value;
        self.diagonal_tile
    }

    #[cfg(test)]
    pub(crate) fn set_stair_tile(&mut self, value: u8) {
        self.stair_tile = value;
    }

    pub(crate) fn clear_stair_tile(&mut self) {
        self.stair_tile = 0;
    }

    #[cfg(test)]
    pub(crate) fn or_stair_tile(&mut self, value: u8) {
        self.stair_tile |= value;
    }

    pub(crate) fn clear_block_flags(&mut self) {
        self.block_flags = 0;
    }

    #[cfg(test)]
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

    #[cfg(test)]
    pub(crate) fn or_deepwater(&mut self, value: u16) -> u16 {
        self.deepwater |= value;
        self.deepwater
    }

    pub(crate) fn clear_normal_tiles(&mut self) {
        self.normal_tiles = 0;
    }

    #[cfg(test)]
    pub(crate) fn or_normal_tiles(&mut self, value: u16) -> u16 {
        self.normal_tiles |= value;
        self.normal_tiles
    }

    pub(crate) fn clear_misc_tiles(&mut self) {
        self.misc_tiles = 0;
    }

    #[cfg(test)]
    pub(crate) fn or_misc_tiles(&mut self, value: u16) -> u16 {
        self.misc_tiles |= value;
        self.misc_tiles
    }

    pub(crate) fn clear_thick_grass(&mut self) {
        self.thick_grass = 0;
    }

    #[cfg(test)]
    pub(crate) fn or_thick_grass(&mut self, value: u16) -> u16 {
        self.thick_grass |= value;
        self.thick_grass
    }

    pub(crate) fn clear_vertical_ledge(&mut self) {
        self.vertical_ledge = 0;
    }

    #[cfg(test)]
    pub(crate) fn or_vertical_ledge(&mut self, value: u8) {
        self.vertical_ledge |= value;
    }

    pub(crate) fn clear_horizontal_ledge(&mut self) {
        self.horizontal_ledge = 0;
    }

    #[cfg(test)]
    pub(crate) fn or_horizontal_ledge(&mut self, value: u8) {
        self.horizontal_ledge |= value;
    }

    pub(crate) fn clear_moving_floor_tiles(&mut self) {
        self.moving_floor_tiles = 0;
    }

    pub(crate) fn clear_icy_floor(&mut self) {
        self.icy_floor = 0;
    }

    pub(crate) fn clear_water_staircase(&mut self) {
        self.water_staircase = 0;
    }

    pub(crate) fn clear_shallow_water(&mut self) {
        self.shallow_water = 0;
    }

    pub(crate) fn clear_destruction_aftermath(&mut self) {
        self.destruction_aftermath = 0;
    }

    pub(crate) fn clear_read_something(&mut self) {
        self.read_something = 0;
    }

    pub(crate) fn clear_ledges_down_leftright(&mut self) {
        self.ledges_down_leftright = 0;
    }

    pub(crate) fn clear_diagonal_ledge_tiles(&mut self) {
        self.diagonal_ledge_tiles = 0;
    }

    pub(crate) fn clear_chest(&mut self) {
        self.chest = 0;
    }

    #[cfg(test)]
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

    /// C's Link_ResetProperties_A does `BYTE(tiledetect_tile_type) = 0` — a
    /// single-byte store that preserves the high byte.
    pub(crate) fn clear_tile_type_low(&mut self) {
        self.tile_type &= 0xff00;
    }

    pub(crate) fn clear_spike_floor_and_triggers(&mut self) {
        self.spike_floor_and_triggers = 0;
    }

    #[cfg(test)]
    pub(crate) fn or_spike_floor_and_triggers(&mut self, value: u8) {
        self.spike_floor_and_triggers |= value;
    }

    pub(crate) fn clear_dashable_tiles(&mut self) {
        self.dashable_tiles = 0;
    }

    #[cfg(test)]
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

    #[cfg(test)]
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

    /// The sparkle garnish spawner records the slot it used in R15, which is
    /// the high byte of the collision word; the low byte keeps its value.
    pub(crate) fn set_last_garnish_index(&mut self, index: u8) {
        self.collision_bits = (self.collision_bits & 0x00ff) | (u16::from(index) << 8);
    }

    #[cfg(test)]
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

    #[cfg(test)]
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

adopting_bridge!(NativeTileDetectionBridgeMut, state: TileDetectionState);

impl<'a> NativeTileDetectionBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_y_high(value: u8);
        fn set_y(value: u16);
        fn set_x(value: u16);
        fn set_location_calc_mask(value: u16);
        fn set_interacting_tile(value: u16);
        fn set_interacting_tile_low(value: u8);
        fn set_fall_hole_scan_index(value: u8);
        fn set_interaction_scratch_y(value: u16);
        fn set_interaction_scratch_x(value: u16);
        fn clear_diagonal_tile();
        fn clear_stair_tile();
        fn clear_block_flags();
        fn set_door_direction_flags(value: u16);
        fn clear_door_direction_flags();
        fn set_diag_state(value: u16);
        fn clear_diag_state();
        fn clear_pit_tile();
        fn or_pit_tile(value: u8);
        fn set_deepwater(value: u16);
        fn clear_deepwater();
        fn clear_normal_tiles();
        fn clear_misc_tiles();
        fn clear_thick_grass();
        fn clear_vertical_ledge();
        fn clear_horizontal_ledge();
        fn clear_moving_floor_tiles();
        fn clear_icy_floor();
        fn clear_water_staircase();
        fn clear_shallow_water();
        fn clear_destruction_aftermath();
        fn clear_read_something();
        fn clear_ledges_down_leftright();
        fn clear_diagonal_ledge_tiles();
        fn clear_chest();
        fn clear_key_lock_gravestones();
        fn set_spike_cactus_tiles(value: u8);
        fn clear_spike_cactus_tiles();
        fn set_tile_type(value: u16);
        fn clear_tile_type();
    }

    pub(crate) fn clear_tile_type_low(&mut self) {
        self.state.clear_tile_type_low();
        // Write-through single-byte store, matching C's `BYTE(tiledetect_tile_type)
        // = 0`. Link_ResetProperties_A also runs during the ending sequence, where a
        // full sync would re-stamp the attract-aliased tiledetect scratch words
        // (0x51/0x5f/0x62) over the live scene state.
        self.ram[TILEDETECT_TILE_TYPE] = 0;
    }

    forward_synced! {
        state;
        fn clear_spike_floor_and_triggers();
        fn clear_dashable_tiles();
        fn set_staircase_cache(value: u8);
        fn set_slope_collision_bits(value: u16);
        fn clear_slope_collision_bits();
        fn set_collision_bits(value: u16);
        fn clear_collision_bits();
    }

    /// Sets only the LOW byte of collision_bits (R14 @ 0x0e), preserving the high byte (0x0f =
    /// R15 / SPRITE_LAST_GARNISH_INDEX, a stale leftover). C's room-tag dispatcher writes
    /// `ram[R14] = k` as a BYTE; the full-u16 setter clobbers 0x0f. The u16 projection re-stamps
    /// 0x0f from the live native high byte (coherent with RAM), so it is preserved.
    pub(crate) fn set_collision_bits_low_byte(&mut self, value: u8) {
        self.state.set_collision_bits_low_byte(value);
        self.sync();
    }

    forward_synced! { state; fn set_last_garnish_index(index: u8); }

    forward_synced! {
        state;
        fn set_layer_collision(mask: u8, enabled: bool);
        fn set_layer_collision_flags(value: u8);
        fn set_tile_probe_anchor(value: u16);
        fn clear_inroom_staircase();
        fn set_liftable_tile_index(value: u8);
        fn set_tile_collision_bits_primary(value: u8);
        fn set_liftable_action_index_primary(value: u8);
        fn set_liftable_action_index_secondary(value: u8);
        fn clear_interaction_scratch_x_low();
        fn set_interaction_scratch_y_bytes(low: u8, high: u8);
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_byte(BG1_MOVE_CALC_BUFFER, self.y_subpixel);
        ram.write_byte(BG1_MOVE_CALC_BUFFER + 1, self.x_subpixel);
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        write_pushed_block_bank(ram, PUSHEDBLOCKS_X_HI, self.x_high);
        write_pushed_block_bank(ram, PUSHEDBLOCKS_X_LO, self.x_low);
        write_pushed_block_bank(ram, PUSHEDBLOCKS_TARGET, self.target);
        write_pushed_block_bank(ram, PUSHEDBLOCKS_Y_HI, self.y_high);
        write_pushed_block_bank(ram, PUSHEDBLOCKS_Y_LO, self.y_low);
        write_pushed_block_bank(ram, PUSHEDBLOCKS_SUBPIXEL, self.subpixel);
        write_pushed_block_bank(ram, PUSHEDBLOCK_FACING_PLAYER, self.facing_player);
        ram.write_byte(PUSHED_BLOCK_MODE, self.animation_mode);
        ram.write_byte(PUSHED_BLOCK_ANIMATION_TIMER, self.animation_timer);
        ram.write_byte(PUSH_BLOCK_DIRECTION, self.push_direction);
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

fn write_pushed_block_bank<R: RamTarget + ?Sized>(
    ram: &mut R,
    base: usize,
    bank: [u8; PUSHED_BLOCK_BANK_LEN],
) {
    for (offset, value) in bank.iter().copied().enumerate() {
        ram.write_byte(base + offset, value);
    }
}

fn write_pushed_block_bank_word(bank: &mut [u8; PUSHED_BLOCK_BANK_LEN], slot: usize, value: u16) {
    if let Some(offset) = pushed_block_bank_offset(slot) {
        if offset + 1 < bank.len() {
            write_le_u16(bank, offset, value);
        }
    }
}

adopting_bridge!(NativeBg1MovementAccumulatorBridgeMut, state: Bg1MovementAccumulatorState);

impl<'a> NativeBg1MovementAccumulatorBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_buffer(value: u16);
        fn set_y_subpixel(value: u8);
        fn set_x_subpixel(value: u8);
        fn advance_x_subpixel(delta: u16) -> u16;
    }
}

adopting_bridge!(NativePushedBlockBridgeMut, state: PushedBlockState);

impl<'a> NativePushedBlockBridgeMut<'a> {
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

    forward_synced! {
        state;
        fn set_animation_mode(value: u8);
        fn reset_animation_timer();
        fn decrement_animation_timer() -> u8;
        fn advance_animation_mode() -> u8;
        fn init_slot(slot: usize, x: u16, y: u16);
        fn set_push_direction(value: u8);
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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
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

    #[cfg(test)]
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

fn write_axis_words<R: RamTarget + ?Sized>(
    ram: &mut R,
    base: usize,
    values: [u16; SWIM_AXIS_COUNT],
) {
    ram.write_word(base, values[0]);
    ram.write_word(base + 2, values[1]);
}

fn axis_word(values: [u16; SWIM_AXIS_COUNT], offset: usize) -> u16 {
    swim_axis_index(offset)
        .and_then(|axis| values.get(axis).copied())
        .unwrap_or(0)
}

adopting_bridge!(NativeSwimAccelerationBridgeMut, state: SwimAccelerationState);

impl<'a> NativeSwimAccelerationBridgeMut<'a> {
    pub(crate) fn set_mode(&mut self, offset: usize, value: u16) {
        if self.state.set_mode(offset, value) {
            self.sync();
        }
    }

    forward_synced! { state; fn clear_mode_low_axis(); }

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

    forward_synced! { state; fn set_max_speed_both_axes(value: u16); }

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

adopting_bridge!(NativeSpecialExitPositionBridgeMut, state: SpecialExitPositionState);

impl<'a> NativeSpecialExitPositionBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_x(value: u16);
        fn set_y(value: u16);
        fn set_position(x: u16, y: u16);
        fn offset_position(x_delta: u16, y_delta: u16);
    }

    pub(crate) fn store_from_player(&mut self) {
        self.state.store_from_player_ram(self.ram);
        self.sync();
    }

    pub(crate) fn restore_player_position(&mut self) {
        self.state.restore_player_position_to_ram(self.ram);
    }
}
