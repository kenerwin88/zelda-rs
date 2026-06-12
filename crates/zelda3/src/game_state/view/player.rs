use super::*;

const FALL_HOLE_SCAN_INDEX_LOCAL: usize = 0x02c9;

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

    pub(crate) fn scratch_a(&self) -> u8 {
        byte(self.ram, SCRATCH_A)
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
        byte(self.ram, LINK_ITEM_ACTION_STEP)
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
        byte(self.ram, LINK_IS_ON_LOWER_LEVEL) | byte(self.ram, LINK_IS_ON_LOWER_LEVEL_MIRROR) != 0
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
            && (!self.is_lifting_or_carrying() || byte(self.ram, LINK_PICKING_THROW_STATE) & 1 == 0)
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

    pub(crate) fn cape_decrement_counter(&self) -> u8 {
        byte(self.ram, CAPE_DECREMENT_COUNTER)
    }

    pub(crate) fn index_of_dashing_sfx(&self) -> u8 {
        byte(self.ram, INDEX_OF_DASHING_SFX)
    }

    pub(crate) fn gravestone_push_timeout(&self) -> u8 {
        byte(self.ram, GRAVESTONE_PUSH_TIMEOUT)
    }

    pub(crate) fn moving_against_diag_deadlocked(&self) -> u8 {
        byte(self.ram, MOVING_AGAINST_DIAG_DEADLOCKED)
    }

    pub(crate) fn about_to_jump_off_ledge(&self) -> u8 {
        byte(self.ram, ABOUT_TO_JUMP_OFF_LEDGE)
    }

    pub(crate) fn item_pickup_in_progress(&self) -> bool {
        byte(self.ram, ITEM_PICKUP_IN_PROGRESS_FLAG) != 0
    }

    pub(crate) fn pit_correction_timer(&self) -> u8 {
        byte(self.ram, PIT_CORRECTION_TIMER)
    }

    pub(crate) fn pit_correction_active(&self) -> bool {
        byte(self.ram, PIT_CORRECTION_ACTIVE_FLAG) != 0
    }

    pub(crate) fn hookshot_bg_check_off_timer(&self) -> u8 {
        byte(self.ram, HOOKSHOT_BG_CHECK_OFF_TIMER)
    }

    /// `offset` is the byte offset (0 = y axis, 2 = x axis), matching the
    /// SwimAcceleration view convention.
    pub(crate) fn swim_stroke_frame_counter(&self, offset: usize) -> u16 {
        word(self.ram, SWIM_STROKE_FRAME_COUNTER + offset)
    }

    pub(crate) fn spin_attack_sound_latch(&self) -> u8 {
        byte(self.ram, SPIN_ATTACK_SOUND_LATCH)
    }

    pub(crate) fn flute_countdown(&self) -> u8 {
        byte(self.ram, FLUTE_COUNTDOWN)
    }

    pub(crate) fn tile_coll_flag(&self) -> u8 {
        byte(self.ram, TILE_COLL_FLAG)
    }

    pub(crate) fn tile_action_index(&self) -> u8 {
        byte(self.ram, TILE_ACTION_INDEX)
    }

    pub(crate) fn player_pose_draw_counter(&self) -> u8 {
        byte(self.ram, PLAYER_POSE_DRAW_COUNTER)
    }

    pub(crate) fn sleep_in_bed_state(&self) -> u8 {
        byte(self.ram, PLAYER_SLEEP_IN_BED_STATE)
    }

    pub(crate) fn moving_floor_x(&self) -> u16 {
        word(self.ram, RELATED_TO_MOVING_FLOOR_X)
    }

    pub(crate) fn moving_floor_y(&self) -> u16 {
        word(self.ram, RELATED_TO_MOVING_FLOOR_Y)
    }

    pub(crate) fn somaria_block_bg_check_flag(&self) -> u8 {
        byte(self.ram, SOMARIA_BLOCK_BG_CHECK_FLAG)
    }

    pub(crate) fn player_special_draw_flag(&self) -> u8 {
        byte(self.ram, PLAYER_SPECIAL_DRAW_FLAG)
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

    pub(crate) fn force_move_any_direction(&self) -> u16 {
        word(self.ram, FORCE_MOVE_ANY_DIRECTION)
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

    pub(crate) fn force_z_mirror_low_ff(&mut self) {
        let value = word(self.ram, LINK_Z_COORD_MIRROR) | 0x00ff;
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
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED];
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
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 2
        {
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
            self.ram[LINK_DIRECTION] |= if (self.ram[LINK_ACTUAL_Y_VELOCITY] as i8).is_negative() {
                8
            } else {
                4
            };
        }
        if self.ram[LINK_ACTUAL_X_VELOCITY] != 0 {
            self.ram[LINK_DIRECTION] |= if (self.ram[LINK_ACTUAL_X_VELOCITY] as i8).is_negative() {
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
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.ram[LINK_FRAME_CHANGE_COUNTER].wrapping_add(1);
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
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = self.ram[LINK_DISABLE_SPRITE_DAMAGE].wrapping_add(1);
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
        self.ram[LINK_ITEM_ACTION_STEP] = value;
    }

    pub(crate) fn set_throw_oam_state_index(&mut self, value: u8) {
        self.ram[LINK_THROW_OAM_STATE_INDEX] = value;
    }

    pub(crate) fn clear_item_action_step_var(&mut self) {
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
    }

    pub(crate) fn increment_item_action_step_var(&mut self) -> u8 {
        self.ram[LINK_ITEM_ACTION_STEP] = self.ram[LINK_ITEM_ACTION_STEP].wrapping_add(1);
        self.ram[LINK_ITEM_ACTION_STEP]
    }

    pub(crate) fn advance_item_action_step_var_wrapping_7_to_1(&mut self) -> u8 {
        self.ram[LINK_ITEM_ACTION_STEP] = if self.ram[LINK_ITEM_ACTION_STEP].wrapping_add(1) == 7 {
            1
        } else {
            self.ram[LINK_ITEM_ACTION_STEP].wrapping_add(1)
        };
        self.ram[LINK_ITEM_ACTION_STEP]
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
        let mut source_offset = read_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET).wrapping_add(0x400);
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
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
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
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.ram[LINK_FRAME_CHANGE_COUNTER].wrapping_add(1);
        if self.ram[LINK_FRAME_CHANGE_COUNTER] >= 16 {
            self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
            self.ram[SWIM_STROKE_ANIM_STEP] = 0;
            self.ram[LINK_ANIMATION_STEPS] = (self.ram[LINK_ANIMATION_STEPS] & 1) ^ 1;
        }
    }

    pub(crate) fn advance_active_swim_animation(&mut self, stroke_steps: &[u8; 4]) {
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.ram[LINK_FRAME_CHANGE_COUNTER].wrapping_add(1);
        if self.ram[LINK_FRAME_CHANGE_COUNTER] >= 8 {
            self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
            self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1) & 3;
            self.ram[SWIM_STROKE_ANIM_STEP] = stroke_steps[self.ram[LINK_ANIMATION_STEPS] as usize];
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
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = self.ram[LINK_TIMER_PUSH_GET_TIRED].wrapping_sub(1);
        self.ram[LINK_TIMER_PUSH_GET_TIRED]
    }

    pub(crate) fn set_push_fatigue_timer(&mut self, value: u8) {
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = value;
    }

    pub(crate) fn reset_push_fatigue_timer(&mut self) {
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
    }

    pub(crate) fn tick_cape_transform_timer(&mut self) -> u8 {
        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = self.ram[LINK_BUNNY_TRANSFORM_TIMER].wrapping_sub(1);
        self.ram[LINK_BUNNY_TRANSFORM_TIMER]
    }

    pub(crate) fn clear_cape_transform_timer(&mut self) {
        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 0;
    }

    pub(crate) fn set_current_item_y(&mut self, value: u8) {
        self.ram[LINK_CURRENT_ITEM_Y] = value;
    }

    pub(crate) fn increment_sleep_in_bed_state(&mut self) {
        self.ram[PLAYER_SLEEP_IN_BED_STATE] = self.ram[PLAYER_SLEEP_IN_BED_STATE].wrapping_add(1);
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
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
        self.ram[LINK_THROW_OAM_STATE_INDEX] = 0;
    }

    pub(crate) fn clear_lift_throw_scratch_state(&mut self) {
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
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
        self.ram[LINK_ITEM_ACTION_STEP] = 0;
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

    pub(crate) fn set_allow_scroll_z(&mut self, value: u8) {
        self.ram[ALLOW_SCROLL_Z] = value;
    }

    pub(crate) fn set_cape_decrement_counter(&mut self, value: u8) {
        self.ram[CAPE_DECREMENT_COUNTER] = value;
    }

    pub(crate) fn decrement_cape_decrement_counter(&mut self) {
        self.ram[CAPE_DECREMENT_COUNTER] = self.ram[CAPE_DECREMENT_COUNTER].wrapping_sub(1);
    }

    pub(crate) fn clear_index_of_dashing_sfx(&mut self) {
        self.ram[INDEX_OF_DASHING_SFX] = 0;
    }

    pub(crate) fn decrement_index_of_dashing_sfx(&mut self) {
        self.ram[INDEX_OF_DASHING_SFX] = self.ram[INDEX_OF_DASHING_SFX].wrapping_sub(1);
    }

    pub(crate) fn set_gravestone_push_timeout(&mut self, value: u8) {
        self.ram[GRAVESTONE_PUSH_TIMEOUT] = value;
    }

    pub(crate) fn decrement_gravestone_push_timeout(&mut self) {
        self.ram[GRAVESTONE_PUSH_TIMEOUT] = self.ram[GRAVESTONE_PUSH_TIMEOUT].wrapping_sub(1);
    }

    pub(crate) fn set_moving_against_diag_deadlocked(&mut self, value: u8) {
        self.ram[MOVING_AGAINST_DIAG_DEADLOCKED] = value;
    }

    pub(crate) fn clear_about_to_jump_off_ledge(&mut self) {
        self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 0;
    }

    pub(crate) fn increment_about_to_jump_off_ledge(&mut self) {
        self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = self.ram[ABOUT_TO_JUMP_OFF_LEDGE].wrapping_add(1);
    }

    pub(crate) fn set_item_pickup_in_progress(&mut self, value: u8) {
        self.ram[ITEM_PICKUP_IN_PROGRESS_FLAG] = value;
    }

    pub(crate) fn set_pit_correction_timer(&mut self, value: u8) {
        self.ram[PIT_CORRECTION_TIMER] = value;
    }

    pub(crate) fn increment_pit_correction_timer(&mut self) {
        self.ram[PIT_CORRECTION_TIMER] = self.ram[PIT_CORRECTION_TIMER].wrapping_add(1);
    }

    pub(crate) fn set_hookshot_bg_check_off_timer(&mut self, value: u8) {
        self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER] = value;
    }

    pub(crate) fn decrement_hookshot_bg_check_off_timer(&mut self) {
        self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER] =
            self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER].wrapping_sub(1);
    }

    /// `offset` is the byte offset (0 = y axis, 2 = x axis), matching the
    /// SwimAcceleration view convention.
    pub(crate) fn set_swim_stroke_frame_counter(&mut self, offset: usize, value: u16) {
        write_le_u16(self.ram, SWIM_STROKE_FRAME_COUNTER + offset, value);
    }

    pub(crate) fn set_spin_attack_sound_latch(&mut self, value: u8) {
        self.ram[SPIN_ATTACK_SOUND_LATCH] = value;
    }

    pub(crate) fn set_state_for_spin_attack(&mut self, value: u8) {
        self.ram[STATE_FOR_SPIN_ATTACK] = value;
    }

    pub(crate) fn set_current_item_active(&mut self, value: u8) {
        self.ram[LINK_CURRENT_ITEM_ACTIVE] = value;
    }

    pub(crate) fn set_selected_rod(&mut self, value: u8) {
        self.ram[EQ_SELECTED_ROD] = value;
    }

    pub(crate) fn set_pit_correction_active(&mut self) {
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 1;
    }

    pub(crate) fn set_flute_countdown(&mut self, value: u8) {
        self.ram[FLUTE_COUNTDOWN] = value;
    }

    pub(crate) fn decrement_flute_countdown(&mut self) {
        self.ram[FLUTE_COUNTDOWN] = self.ram[FLUTE_COUNTDOWN].wrapping_sub(1);
    }

    pub(crate) fn set_layer_collision_flags(&mut self, value: u8) {
        self.ram[PLAYER_LAYER_COLLISION_FLAGS] = value;
    }

    pub(crate) fn set_tile_coll_flag(&mut self, value: u8) {
        self.ram[TILE_COLL_FLAG] = value;
    }

    pub(crate) fn set_tile_action_index(&mut self, value: u8) {
        self.ram[TILE_ACTION_INDEX] = value;
    }

    pub(crate) fn set_cached_tile_action_index(&mut self, value: u8) {
        self.ram[CACHED_TILE_ACTION_INDEX] = value;
    }

    pub(crate) fn clear_swimming_countdown(&mut self) {
        self.ram[SWIMMING_COUNTDOWN] = 0;
    }

    pub(crate) fn set_force_move_any_direction(&mut self, value: u16) {
        write_le_u16(self.ram, FORCE_MOVE_ANY_DIRECTION, value);
    }

    pub(crate) fn set_custom_spell_animation_active(&mut self) {
        self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 1;
    }

    pub(crate) fn clear_spin_attack_sound_latch(&mut self) {
        self.ram[SPIN_ATTACK_SOUND_LATCH] = 0;
    }

    pub(crate) fn clear_state_for_spin_attack(&mut self) {
        self.ram[STATE_FOR_SPIN_ATTACK] = 0;
    }

    pub(crate) fn clear_magic_spell_player_lock(&mut self) {
        self.ram[MAGIC_SPELL_PLAYER_LOCK_FLAG] = 0;
    }

    pub(crate) fn clear_ancilla_interactive_reset_flag(&mut self) {
        self.ram[ANCILLA_INTERACTIVE_RESET_FLAG] = 0;
    }

    pub(crate) fn clear_flute_countdown(&mut self) {
        self.ram[FLUTE_COUNTDOWN] = 0;
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

    pub(crate) fn set_sprite_pickup_flag_cached(&mut self, value: u8) {
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED] = value;
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

    pub(crate) fn set_player_special_draw_flag(&mut self, value: u8) {
        self.ram[PLAYER_SPECIAL_DRAW_FLAG] = value;
    }
}

pub(crate) struct SpecialExitPositionRawView<'a> {
    ram: &'a [u8],
}

impl<'a> SpecialExitPositionRawView<'a> {
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

pub(crate) struct SpecialExitPositionRawViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> SpecialExitPositionRawViewMut<'a> {
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

pub(crate) struct SwimAccelerationRawView<'a> {
    ram: &'a [u8],
}

impl<'a> SwimAccelerationRawView<'a> {
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

pub(crate) struct SwimAccelerationRawViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> SwimAccelerationRawViewMut<'a> {
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

    pub(crate) fn y_subpixel(&self) -> u8 {
        byte(self.ram, BG1_MOVE_CALC_BUFFER)
    }
}

pub(crate) struct Bg1MoveCalcViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> Bg1MoveCalcViewMut<'a> {
    pub(crate) fn set_buffer(&mut self, value: u16) {
        write_le_u16(self.ram, BG1_MOVE_CALC_BUFFER, value);
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.ram[BG1_MOVE_CALC_BUFFER] = value;
    }

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

    pub(crate) fn tile_collision_bits_primary(&self) -> u8 {
        byte(self.ram, TILE_COLLISION_BITS_PRIMARY)
    }

    pub(crate) fn tile_collision_bits_secondary(&self) -> u8 {
        byte(self.ram, TILE_COLLISION_BITS_SECONDARY)
    }

    pub(crate) fn liftable_tile_index(&self) -> u8 {
        byte(self.ram, LIFTABLE_TILE_DETECTED_INDEX_DOUBLED)
    }

    pub(crate) fn liftable_action_index_primary(&self) -> u8 {
        byte(self.ram, LIFTABLE_TILE_ACTION_INDEX_PRIMARY)
    }

    pub(crate) fn interaction_scratch_y(&self) -> u16 {
        word(self.ram, SCRATCH_0)
    }

    pub(crate) fn interaction_scratch_x(&self) -> u16 {
        word(self.ram, SCRATCH_1)
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

    pub(crate) fn interacting_tile_low(&self) -> u8 {
        byte(self.ram, INDEX_OF_INTERACTING_TILE)
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
        byte(self.ram, TILEDETECT_SLOPE_COLLISION_BITS) | byte(self.ram, TILEDETECT_COLLISION_BITS)
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

    pub(crate) fn set_interacting_tile_low(&mut self, value: u8) {
        self.ram[INDEX_OF_INTERACTING_TILE] = value;
    }

    pub(crate) fn set_fall_hole_scan_index(&mut self, value: u8) {
        self.ram[FALL_HOLE_SCAN_INDEX_LOCAL] = value;
    }

    /// Y coordinate scratch word at 0x72 shared with tile interaction
    /// routines.
    pub(crate) fn set_interaction_scratch_y(&mut self, value: u16) {
        write_le_u16(self.ram, SCRATCH_0, value);
    }

    /// X coordinate scratch word at 0x74 shared with tile interaction
    /// routines.
    pub(crate) fn set_interaction_scratch_x(&mut self, value: u16) {
        write_le_u16(self.ram, SCRATCH_1, value);
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

    pub(crate) fn set_tile_collision_bits_primary(&mut self, value: u8) {
        self.ram[TILE_COLLISION_BITS_PRIMARY] = value;
    }

    pub(crate) fn set_liftable_action_index_primary(&mut self, value: u8) {
        self.ram[LIFTABLE_TILE_ACTION_INDEX_PRIMARY] = value;
    }

    pub(crate) fn set_liftable_action_index_secondary(&mut self, value: u8) {
        self.ram[LIFTABLE_TILE_ACTION_INDEX_SECONDARY] = value;
    }

    pub(crate) fn clear_interaction_scratch_x_low(&mut self) {
        self.ram[SCRATCH_1] = 0;
    }

    /// Writes the two scratch-Y bytes separately (low, then high), as the
    /// door-debris smash path packs y/x coordinates into the word.
    pub(crate) fn set_interaction_scratch_y_bytes(&mut self, low: u8, high: u8) {
        self.ram[SCRATCH_0] = low;
        self.ram[SCRATCH_0 + 1] = high;
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

    pub(crate) fn animation_mode(&self) -> u8 {
        byte(self.ram, PUSHED_BLOCK_MODE)
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

    pub(crate) fn set_animation_mode(&mut self, value: u8) {
        self.ram[PUSHED_BLOCK_MODE] = value;
    }

    pub(crate) fn reset_animation_timer(&mut self) {
        self.ram[PUSHED_BLOCK_ANIMATION_TIMER] = 9;
    }

    pub(crate) fn decrement_animation_timer(&mut self) -> u8 {
        self.ram[PUSHED_BLOCK_ANIMATION_TIMER] =
            self.ram[PUSHED_BLOCK_ANIMATION_TIMER].wrapping_sub(1);
        self.ram[PUSHED_BLOCK_ANIMATION_TIMER]
    }

    pub(crate) fn advance_animation_mode(&mut self) -> u8 {
        self.reset_animation_timer();
        self.ram[PUSHED_BLOCK_MODE] = self.ram[PUSHED_BLOCK_MODE].wrapping_add(1);
        self.ram[PUSHED_BLOCK_MODE]
    }

    /// Initializes a pushed-block slot: split x/y words, zero target and
    /// subpixel. Matches the original write order.
    pub(crate) fn init_slot(&mut self, slot: usize, x: u16, y: u16) {
        write_le_u16(self.ram, PUSHEDBLOCKS_X_LO + slot * 2, x & 0x00ff);
        write_le_u16(self.ram, PUSHEDBLOCKS_X_HI + slot * 2, x >> 8);
        write_le_u16(self.ram, PUSHEDBLOCKS_Y_LO + slot * 2, y & 0x00ff);
        write_le_u16(self.ram, PUSHEDBLOCKS_Y_HI + slot * 2, y >> 8);
        write_le_u16(self.ram, PUSHEDBLOCKS_TARGET + slot * 2, 0);
        write_le_u16(self.ram, PUSHEDBLOCKS_SUBPIXEL + slot * 2, 0);
    }

    pub(crate) fn set_push_direction(&mut self, value: u8) {
        self.ram[PUSH_BLOCK_DIRECTION] = value;
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
