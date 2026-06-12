use super::*;

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

    pub(crate) fn current_area_of_player(&self) -> u8 {
        byte(self.ram, CURRENT_AREA_OF_PLAYER)
    }

    pub(crate) fn milestone_item_gfx_swap_countdown(&self) -> u8 {
        byte(self.ram, MILESTONE_ITEM_GFX_SWAP_COUNTDOWN)
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

    pub(crate) fn overworld_map_state_word(&self) -> u16 {
        word(self.ram, OVERWORLD_MAP_STATE)
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

    pub(crate) fn horizontal_room_bounds_base_index(&self) -> usize {
        (byte(self.ram, QUADRANT_FULLSIZE_X) >> 1) as usize
    }

    pub(crate) fn vertical_room_bounds_base_index(&self) -> usize {
        (byte(self.ram, QUADRANT_FULLSIZE_Y) >> 1) as usize
    }

    pub(crate) fn quadrant_fullsize_x(&self) -> u8 {
        byte(self.ram, QUADRANT_FULLSIZE_X)
    }

    pub(crate) fn quadrant_fullsize_y(&self) -> u8 {
        byte(self.ram, QUADRANT_FULLSIZE_Y)
    }

    pub(crate) fn dungeon_quadrant_visit_index(
        &self,
        player_quadrant_y: u8,
        player_quadrant_x: u8,
    ) -> usize {
        ((byte(self.ram, QUADRANT_FULLSIZE_Y) as usize) << 2)
            + ((byte(self.ram, QUADRANT_FULLSIZE_X) as usize) << 1)
            + player_quadrant_y as usize
            + player_quadrant_x as usize
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

    pub(crate) fn scroll_x_start(&self) -> u16 {
        word(self.ram, OVERWORLD_SCROLL_X_START)
    }

    pub(crate) fn scroll_x_end(&self) -> u16 {
        word(self.ram, OVERWORLD_SCROLL_X_END)
    }

    pub(crate) fn scroll_y_end(&self) -> u16 {
        word(self.ram, OVERWORLD_SCROLL_Y_END)
    }

    pub(crate) fn exit_screen_index(&self) -> u16 {
        word(self.ram, OVERWORLD_SCREEN_INDEX_EXIT)
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

    pub(crate) fn dungeon_room(&self) -> u16 {
        word(self.ram, DUNGEON_ROOM)
    }

    pub(crate) fn overworld_screen_word(&self) -> u16 {
        word(self.ram, OVERWORLD_SCREEN_INDEX)
    }

    pub(crate) fn indoor_flag(&self) -> u8 {
        byte(self.ram, PLAYER_IS_INDOORS)
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

    pub(crate) fn set_overworld_map_state_word(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_MAP_STATE, value);
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

    pub(crate) fn decrement_milestone_item_gfx_swap_countdown(&mut self) {
        self.ram[MILESTONE_ITEM_GFX_SWAP_COUNTDOWN] =
            self.ram[MILESTONE_ITEM_GFX_SWAP_COUNTDOWN].wrapping_sub(1);
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

    pub(crate) fn set_mode7_zoom_step_counter(&mut self, value: u8) {
        self.ram[MODE7_ZOOM_STEP_COUNTER] = value;
    }

    pub(crate) fn set_timer_for_mode7_zoom(&mut self, value: u8) {
        self.ram[TIMER_FOR_MODE7_ZOOM] = value;
    }

    pub(crate) fn set_overworld_map_flags(&mut self, value: u8) {
        self.ram[OVERWORLD_MAP_FLAGS] = value;
    }

    pub(crate) fn set_aux_bg_subset(&mut self, index: usize, value: u8) {
        self.ram[AUX_BG_SUBSET_0 + index] = value;
    }

    pub(crate) fn set_overworld_palette_aux1_hi(&mut self, value: u8) {
        self.ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI] = value;
    }

    pub(crate) fn and_overworld_map_flags(&mut self, value: u8) {
        self.ram[OVERWORLD_MAP_FLAGS] &= value;
    }

    pub(crate) fn or_overworld_map_flags(&mut self, value: u8) {
        self.ram[OVERWORLD_MAP_FLAGS] |= value;
    }

    pub(crate) fn decrement_timer_for_mode7_zoom(&mut self) {
        self.ram[TIMER_FOR_MODE7_ZOOM] = self.ram[TIMER_FOR_MODE7_ZOOM].wrapping_sub(1);
    }

    pub(crate) fn and_birdtravel_status(&mut self, value: u8) {
        self.ram[BIRDTRAVEL_STATUS] &= value;
    }

    pub(crate) fn decrement_birdtravel_status(&mut self) {
        self.ram[BIRDTRAVEL_STATUS] = self.ram[BIRDTRAVEL_STATUS].wrapping_sub(1);
    }

    pub(crate) fn increment_birdtravel_status(&mut self) {
        self.ram[BIRDTRAVEL_STATUS] = self.ram[BIRDTRAVEL_STATUS].wrapping_add(1);
    }

    pub(crate) fn set_overworld_area_index(&mut self, value: u8) {
        self.ram[OVERWORLD_AREA_INDEX] = value;
    }

    pub(crate) fn set_overworld_area_index_word(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_AREA_INDEX, value);
    }

    pub(crate) fn set_current_area_of_player_word(&mut self, value: u16) {
        write_le_u16(self.ram, CURRENT_AREA_OF_PLAYER, value);
    }

    pub(crate) fn set_flag_overworld_area_changed(&mut self, value: u8) {
        self.ram[FLAG_OVERWORLD_AREA_CHANGED] = value;
    }

    pub(crate) fn set_ow_countdown_transition(&mut self, value: u8) {
        self.ram[OW_COUNTDOWN_TRANSITION] = value;
    }

    pub(crate) fn decrement_ow_countdown_transition(&mut self) -> u8 {
        self.ram[OW_COUNTDOWN_TRANSITION] = self.ram[OW_COUNTDOWN_TRANSITION].wrapping_sub(1);
        self.ram[OW_COUNTDOWN_TRANSITION]
    }

    pub(crate) fn ow_countdown_transition(&self) -> u8 {
        self.ram[OW_COUNTDOWN_TRANSITION]
    }

    pub(crate) fn set_transition_counter(&mut self, value: u8) {
        self.ram[TRANSITION_COUNTER] = value;
    }

    pub(crate) fn increment_transition_counter(&mut self) -> u8 {
        self.ram[TRANSITION_COUNTER] = self.ram[TRANSITION_COUNTER].wrapping_add(1);
        self.ram[TRANSITION_COUNTER]
    }

    pub(crate) fn transition_counter(&self) -> u8 {
        self.ram[TRANSITION_COUNTER]
    }

    pub(crate) fn set_transition_dir_enum(&mut self, value: u8) {
        self.ram[OVERWORLD_TRANSITION_DIR] = value;
    }

    pub(crate) fn transition_dir_enum(&self) -> u8 {
        self.ram[OVERWORLD_TRANSITION_DIR]
    }

    pub(crate) fn set_door_animation_step(&mut self, value: u8) {
        self.ram[DOOR_ANIMATION_STEP_INDICATOR] = value;
    }

    pub(crate) fn door_animation_step(&self) -> u16 {
        word(self.ram, DOOR_ANIMATION_STEP_INDICATOR)
    }

    pub(crate) fn set_door_animation_step_word(&mut self, value: u16) {
        write_le_u16(self.ram, DOOR_ANIMATION_STEP_INDICATOR, value);
    }

    pub(crate) fn increment_move_overlay_ctr(&mut self) -> u8 {
        self.ram[MOVE_OVERLAY_CTR] = self.ram[MOVE_OVERLAY_CTR].wrapping_add(1) & 3;
        self.ram[MOVE_OVERLAY_CTR]
    }

    pub(crate) fn set_overworld_screen_trans_dir_bits_word(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, value);
    }

    pub(crate) fn clear_overworld_screen_trans_dir_bits(&mut self) {
        self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS] = 0;
    }

    pub(crate) fn set_hud_palette(&mut self, value: u8) {
        self.ram[HUD_PALETTE] = value;
    }

    pub(crate) fn clear_hud_floor_changed_timer(&mut self) {
        self.ram[HUD_FLOOR_CHANGED_TIMER] = 0;
    }

    pub(crate) fn set_nmi_thread_active(&mut self, value: u8) {
        self.ram[IS_NMI_THREAD_ACTIVE] = value;
    }

    pub(crate) fn increment_nmi_thread_active(&mut self) -> u8 {
        self.ram[IS_NMI_THREAD_ACTIVE] = self.ram[IS_NMI_THREAD_ACTIVE].wrapping_add(1);
        self.ram[IS_NMI_THREAD_ACTIVE]
    }

    pub(crate) fn clear_nmi_flag_update_polyhedral(&mut self) {
        self.ram[NMI_FLAG_UPDATE_POLYHEDRAL] = 0;
    }

    pub(crate) fn set_is_standing_in_doorway_cached(&mut self, value: u8) {
        self.ram[IS_STANDING_IN_DOORWAY_CACHED] = value;
    }

    pub(crate) fn set_mapbak_tm(&mut self, value: u8) {
        self.ram[MAPBAK_TM] = value;
    }

    pub(crate) fn set_mapbak_ts(&mut self, value: u8) {
        self.ram[MAPBAK_TS] = value;
    }

    pub(crate) fn clear_set_when_damaging_enemies(&mut self) {
        self.ram[SET_WHEN_DAMAGING_ENEMIES] = 0;
    }

    pub(crate) fn set_which_entrance_byte(&mut self, value: u8) {
        self.ram[WHICH_ENTRANCE] = value;
    }

    pub(crate) fn set_overworld_hole_scan_step(&mut self, value: u8) {
        self.ram[OVERWORLD_HOLE_SCAN_STEP] = value;
    }

    pub(crate) fn set_quadrant_fullsize_x(&mut self, value: u8) {
        self.ram[QUADRANT_FULLSIZE_X] = value;
    }

    pub(crate) fn set_quadrant_fullsize_y(&mut self, value: u8) {
        self.ram[QUADRANT_FULLSIZE_Y] = value;
    }

    pub(crate) fn set_fullsize_overworld_quadrants(&mut self) {
        self.ram[QUADRANT_FULLSIZE_X] = 2;
        self.ram[QUADRANT_FULLSIZE_Y] = 2;
    }

    pub(crate) fn set_horizontal_room_fullsize_state(&mut self, value: u8) {
        self.ram[QUADRANT_FULLSIZE_X] = value;
    }

    pub(crate) fn set_vertical_room_fullsize_state(&mut self, value: u8) {
        self.ram[QUADRANT_FULLSIZE_Y] = value;
    }

    pub(crate) fn apply_dungeon_layout_quadrant_fullsize(
        &mut self,
        layout_flags: u8,
        horizontal_mask: u8,
        vertical_mask: u8,
        blast_wall_x_open: bool,
        blast_wall_y_open: bool,
    ) {
        self.ram[QUADRANT_FULLSIZE_X] = if blast_wall_x_open || layout_flags & horizontal_mask == 0
        {
            2
        } else {
            0
        };
        self.ram[QUADRANT_FULLSIZE_Y] = if blast_wall_y_open || layout_flags & vertical_mask == 0 {
            2
        } else {
            0
        };
    }

    pub(crate) fn apply_dungeon_layout_horizontal_fullsize(
        &mut self,
        layout_flags: u8,
        horizontal_mask: u8,
        blast_wall_x_open: bool,
    ) {
        self.ram[QUADRANT_FULLSIZE_X] = if blast_wall_x_open || layout_flags & horizontal_mask == 0
        {
            2
        } else {
            0
        };
    }

    pub(crate) fn apply_dungeon_layout_vertical_fullsize(
        &mut self,
        layout_flags: u8,
        vertical_mask: u8,
        blast_wall_y_open: bool,
    ) {
        self.ram[QUADRANT_FULLSIZE_Y] = if blast_wall_y_open || layout_flags & vertical_mask == 0 {
            2
        } else {
            0
        };
    }

    pub(crate) fn apply_reset_xy_quadrant_overrides(&mut self, reset_xy_flags: u16) {
        if reset_xy_flags as u8 != 0 {
            self.ram[QUADRANT_FULLSIZE_X] = reset_xy_flags as u8;
        }
        if (reset_xy_flags >> 8) as u8 != 0 {
            self.ram[QUADRANT_FULLSIZE_Y] = (reset_xy_flags >> 8) as u8;
        }
    }

    pub(crate) fn cache_quadrant_fullsize_state(&mut self) {
        let quadrant = word(self.ram, QUADRANT_FULLSIZE_X);
        write_le_u16(self.ram, QUADRANT_FULLSIZE_X_CACHED, quadrant);
    }

    pub(crate) fn force_horizontal_fullsize_for_blast_wall(&mut self) {
        self.ram[QUADRANT_FULLSIZE_X] = 2;
    }

    pub(crate) fn force_vertical_fullsize_for_blast_wall(&mut self) {
        self.ram[QUADRANT_FULLSIZE_Y] = 2;
    }

    pub(crate) fn set_overworld_tile_theme_index(&mut self, value: u8) {
        self.ram[OVERWORLD_TILE_THEME_INDEX] = value;
    }

    pub(crate) fn set_main_tile_theme_index(&mut self, value: u8) {
        self.ram[MAIN_TILE_THEME_INDEX] = value;
    }

    pub(crate) fn set_aux_tile_theme_index(&mut self, value: u8) {
        self.ram[AUX_TILE_THEME_INDEX] = value;
    }

    pub(crate) fn set_misc_sprites_graphics_index(&mut self, value: u8) {
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = value;
    }

    pub(crate) fn set_palette_sp6r_indoors(&mut self, value: u8) {
        self.ram[PALETTE_SP6R_INDOORS] = value;
    }

    pub(crate) fn set_big_key_door_message_triggered(&mut self, value: u16) {
        write_le_u16(self.ram, BIG_KEY_DOOR_MESSAGE_TRIGGERED, value);
    }

    pub(crate) fn big_key_door_message_triggered(&self) -> u16 {
        word(self.ram, BIG_KEY_DOOR_MESSAGE_TRIGGERED)
    }

    pub(crate) fn set_overworld_peg_puzzle_progress(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_PEG_PUZZLE_PROGRESS, value);
    }

    pub(crate) fn overworld_peg_puzzle_progress(&self) -> u16 {
        word(self.ram, OVERWORLD_PEG_PUZZLE_PROGRESS)
    }

    pub(crate) fn set_camera_y_coord_scroll_low(&mut self, value: u16) {
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_LOW, value);
    }

    pub(crate) fn camera_y_coord_scroll_low(&self) -> u16 {
        word(self.ram, CAMERA_Y_COORD_SCROLL_LOW)
    }

    pub(crate) fn set_camera_y_coord_scroll_hi(&mut self, value: u16) {
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_HI, value);
    }

    pub(crate) fn camera_y_coord_scroll_hi(&self) -> u16 {
        word(self.ram, CAMERA_Y_COORD_SCROLL_HI)
    }

    pub(crate) fn set_camera_x_coord_scroll_low(&mut self, value: u16) {
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_LOW, value);
    }

    pub(crate) fn camera_x_coord_scroll_low(&self) -> u16 {
        word(self.ram, CAMERA_X_COORD_SCROLL_LOW)
    }

    pub(crate) fn set_camera_x_coord_scroll_hi(&mut self, value: u16) {
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_HI, value);
    }

    pub(crate) fn camera_x_coord_scroll_hi(&self) -> u16 {
        word(self.ram, CAMERA_X_COORD_SCROLL_HI)
    }

    pub(crate) fn scroll_x_start(&self) -> u16 {
        word(self.ram, OVERWORLD_SCROLL_X_START)
    }

    pub(crate) fn scroll_x_end(&self) -> u16 {
        word(self.ram, OVERWORLD_SCROLL_X_END)
    }

    pub(crate) fn scroll_y_end(&self) -> u16 {
        word(self.ram, OVERWORLD_SCROLL_Y_END)
    }

    pub(crate) fn camera_scroll_low_for_axis(&self, horizontal: bool) -> u16 {
        if horizontal {
            word(self.ram, CAMERA_X_COORD_SCROLL_LOW)
        } else {
            word(self.ram, CAMERA_Y_COORD_SCROLL_LOW)
        }
    }

    pub(crate) fn camera_scroll_hi_for_axis(&self, horizontal: bool) -> u16 {
        if horizontal {
            word(self.ram, CAMERA_X_COORD_SCROLL_HI)
        } else {
            word(self.ram, CAMERA_Y_COORD_SCROLL_HI)
        }
    }

    pub(crate) fn add_camera_scroll_for_axis(&mut self, horizontal: bool, delta: i16) -> u16 {
        let hi_addr = if horizontal {
            CAMERA_X_COORD_SCROLL_HI
        } else {
            CAMERA_Y_COORD_SCROLL_HI
        };
        let low_addr = if horizontal {
            CAMERA_X_COORD_SCROLL_LOW
        } else {
            CAMERA_Y_COORD_SCROLL_LOW
        };
        let hi = word(self.ram, hi_addr).wrapping_add_signed(delta);
        write_le_u16(self.ram, hi_addr, hi);
        write_le_u16(self.ram, low_addr, hi.wrapping_add(2));
        hi
    }

    pub(crate) fn set_camera_scroll_from_link_for_axis(&mut self, horizontal: bool, value: u16) {
        if horizontal {
            self.set_camera_x_coord_scroll_hi(value);
            self.set_camera_x_coord_scroll_low(value.wrapping_add(2));
        } else {
            self.set_camera_y_coord_scroll_hi(value);
            self.set_camera_y_coord_scroll_low(value.wrapping_add(2));
        }
    }

    pub(crate) fn set_up_down_scroll_target(&mut self, value: u16) {
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET, value);
    }

    pub(crate) fn up_down_scroll_target(&self, index: usize) -> u16 {
        word(self.ram, UP_DOWN_SCROLL_TARGET + index * 2)
    }

    pub(crate) fn set_up_down_scroll_target_end(&mut self, value: u16) {
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET_END, value);
    }

    pub(crate) fn set_left_right_scroll_target(&mut self, value: u16) {
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET, value);
    }

    pub(crate) fn set_left_right_scroll_target_end(&mut self, value: u16) {
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET_END, value);
    }

    pub(crate) fn cache_scroll_targets(&mut self) {
        copy_word(
            self.ram,
            UP_DOWN_SCROLL_TARGET_CACHED,
            UP_DOWN_SCROLL_TARGET,
        );
        copy_word(
            self.ram,
            UP_DOWN_SCROLL_TARGET_END_CACHED,
            UP_DOWN_SCROLL_TARGET_END,
        );
        copy_word(
            self.ram,
            LEFT_RIGHT_SCROLL_TARGET_CACHED,
            LEFT_RIGHT_SCROLL_TARGET,
        );
        copy_word(
            self.ram,
            LEFT_RIGHT_SCROLL_TARGET_END_CACHED,
            LEFT_RIGHT_SCROLL_TARGET_END,
        );
    }

    pub(crate) fn set_overworld_scroll_up_counter(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCROLL_UP_COUNTER, value);
    }

    pub(crate) fn set_overworld_scroll_down_counter(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCROLL_DOWN_COUNTER, value);
    }

    pub(crate) fn set_overworld_scroll_left_counter(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCROLL_LEFT_COUNTER, value);
    }

    pub(crate) fn set_overworld_scroll_right_counter(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCROLL_RIGHT_COUNTER, value);
    }

    pub(crate) fn set_overworld_scroll_counter_for_axis(&mut self, ya: usize, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCROLL_UP_COUNTER + ya * 2, value);
    }

    pub(crate) fn overworld_scroll_counter_for_axis(&self, ya: usize) -> u16 {
        word(self.ram, OVERWORLD_SCROLL_UP_COUNTER + ya * 2)
    }

    pub(crate) fn clear_opposed_scroll_counters(&mut self, ya: usize) {
        self.set_overworld_scroll_counter_for_axis(ya, 0);
        self.set_overworld_scroll_counter_for_axis(ya ^ 1, 0);
    }

    pub(crate) fn set_opposed_scroll_counter_pair(&mut self, ya: usize, value: u16) {
        self.set_overworld_scroll_counter_for_axis(ya, value);
        self.set_overworld_scroll_counter_for_axis(ya ^ 1, (0u16).wrapping_sub(value));
    }

    pub(crate) fn set_savegame_has_master_sword_flags(&mut self, value: u16) {
        write_le_u16(self.ram, SAVEGAME_HAS_MASTER_SWORD_FLAGS, value);
    }

    pub(crate) fn savegame_has_master_sword_flags(&self) -> u16 {
        word(self.ram, SAVEGAME_HAS_MASTER_SWORD_FLAGS)
    }

    pub(crate) fn set_overworld_bomb_tile_sweep_x(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_BOMB_TILE_SWEEP_X, value);
    }

    pub(crate) fn set_overworld_bomb_tile_sweep_y_end(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_BOMB_TILE_SWEEP_Y_END, value);
    }

    pub(crate) fn set_overworld_hole_tilemap_pos(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_HOLE_TILEMAP_POS, value);
    }

    pub(crate) fn set_special_exit_room_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        write_le_u16(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_Y_START, y_start);
        write_le_u16(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_Y_END, y_end);
        write_le_u16(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_X_START, x_start);
        write_le_u16(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_X_END, x_end);
    }

    pub(crate) fn copy_spexit_scroll_targets(&mut self) {
        let up = word(self.ram, UP_DOWN_SCROLL_TARGET);
        let up_end = word(self.ram, UP_DOWN_SCROLL_TARGET_END);
        let lr = word(self.ram, LEFT_RIGHT_SCROLL_TARGET);
        let lr_end = word(self.ram, LEFT_RIGHT_SCROLL_TARGET_END);
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET_SPEXIT, up);
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET_END_SPEXIT, up_end);
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET_SPEXIT, lr);
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET_END_SPEXIT, lr_end);
    }

    pub(crate) fn copy_spexit_scroll_counters(&mut self) {
        let up = word(self.ram, OVERWORLD_SCROLL_UP_COUNTER);
        let down = word(self.ram, OVERWORLD_SCROLL_DOWN_COUNTER);
        let left = word(self.ram, OVERWORLD_SCROLL_LEFT_COUNTER);
        let right = word(self.ram, OVERWORLD_SCROLL_RIGHT_COUNTER);
        write_le_u16(self.ram, OVERWORLD_SCROLL_UP_COUNTER_SPEXIT, up);
        write_le_u16(self.ram, OVERWORLD_SCROLL_DOWN_COUNTER_SPEXIT, down);
        write_le_u16(self.ram, OVERWORLD_SCROLL_LEFT_COUNTER_SPEXIT, left);
        write_le_u16(self.ram, OVERWORLD_SCROLL_RIGHT_COUNTER_SPEXIT, right);
    }

    pub(crate) fn restore_spexit_scroll_targets(&mut self) {
        let up = word(self.ram, UP_DOWN_SCROLL_TARGET_SPEXIT);
        let up_end = word(self.ram, UP_DOWN_SCROLL_TARGET_END_SPEXIT);
        let lr = word(self.ram, LEFT_RIGHT_SCROLL_TARGET_SPEXIT);
        let lr_end = word(self.ram, LEFT_RIGHT_SCROLL_TARGET_END_SPEXIT);
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET, up);
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET_END, up_end);
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET, lr);
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET_END, lr_end);
    }

    pub(crate) fn restore_spexit_scroll_counters(&mut self) {
        let up = word(self.ram, OVERWORLD_SCROLL_UP_COUNTER_SPEXIT);
        let down = word(self.ram, OVERWORLD_SCROLL_DOWN_COUNTER_SPEXIT);
        let left = word(self.ram, OVERWORLD_SCROLL_LEFT_COUNTER_SPEXIT);
        let right = word(self.ram, OVERWORLD_SCROLL_RIGHT_COUNTER_SPEXIT);
        write_le_u16(self.ram, OVERWORLD_SCROLL_UP_COUNTER, up);
        write_le_u16(self.ram, OVERWORLD_SCROLL_DOWN_COUNTER, down);
        write_le_u16(self.ram, OVERWORLD_SCROLL_LEFT_COUNTER, left);
        write_le_u16(self.ram, OVERWORLD_SCROLL_RIGHT_COUNTER, right);
    }

    pub(crate) fn copy_exit_scroll_targets(&mut self) {
        let up = word(self.ram, UP_DOWN_SCROLL_TARGET);
        let up_end = word(self.ram, UP_DOWN_SCROLL_TARGET_END);
        let lr = word(self.ram, LEFT_RIGHT_SCROLL_TARGET);
        let lr_end = word(self.ram, LEFT_RIGHT_SCROLL_TARGET_END);
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET_EXIT, up);
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET_END_EXIT, up_end);
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET_EXIT, lr);
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET_END_EXIT, lr_end);
    }

    pub(crate) fn copy_exit_scroll_counters(&mut self) {
        let up = word(self.ram, OVERWORLD_SCROLL_UP_COUNTER);
        let down = word(self.ram, OVERWORLD_SCROLL_DOWN_COUNTER);
        let left = word(self.ram, OVERWORLD_SCROLL_LEFT_COUNTER);
        let right = word(self.ram, OVERWORLD_SCROLL_RIGHT_COUNTER);
        write_le_u16(self.ram, OVERWORLD_SCROLL_UP_COUNTER_EXIT, up);
        write_le_u16(self.ram, OVERWORLD_SCROLL_DOWN_COUNTER_EXIT, down);
        write_le_u16(self.ram, OVERWORLD_SCROLL_LEFT_COUNTER_EXIT, left);
        write_le_u16(self.ram, OVERWORLD_SCROLL_RIGHT_COUNTER_EXIT, right);
    }

    pub(crate) fn restore_exit_scroll_targets(&mut self) {
        let up = word(self.ram, UP_DOWN_SCROLL_TARGET_EXIT);
        let up_end = word(self.ram, UP_DOWN_SCROLL_TARGET_END_EXIT);
        let lr = word(self.ram, LEFT_RIGHT_SCROLL_TARGET_EXIT);
        let lr_end = word(self.ram, LEFT_RIGHT_SCROLL_TARGET_END_EXIT);
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET, up);
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET_END, up_end);
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET, lr);
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET_END, lr_end);
    }

    pub(crate) fn restore_exit_scroll_counters(&mut self) {
        let up = word(self.ram, OVERWORLD_SCROLL_UP_COUNTER_EXIT);
        let down = word(self.ram, OVERWORLD_SCROLL_DOWN_COUNTER_EXIT);
        let left = word(self.ram, OVERWORLD_SCROLL_LEFT_COUNTER_EXIT);
        let right = word(self.ram, OVERWORLD_SCROLL_RIGHT_COUNTER_EXIT);
        write_le_u16(self.ram, OVERWORLD_SCROLL_UP_COUNTER, up);
        write_le_u16(self.ram, OVERWORLD_SCROLL_DOWN_COUNTER, down);
        write_le_u16(self.ram, OVERWORLD_SCROLL_LEFT_COUNTER, left);
        write_le_u16(self.ram, OVERWORLD_SCROLL_RIGHT_COUNTER, right);
    }

    pub(crate) fn save_spexit_area_index(&mut self) {
        let v = word(self.ram, OVERWORLD_AREA_INDEX);
        write_le_u16(self.ram, OVERWORLD_AREA_INDEX_SPEXIT, v);
    }

    pub(crate) fn restore_spexit_area_index(&mut self) {
        let value = word(self.ram, OVERWORLD_AREA_INDEX_SPEXIT);
        write_le_u16(self.ram, OVERWORLD_AREA_INDEX, value);
    }

    pub(crate) fn spexit_area_index(&self) -> u16 {
        word(self.ram, OVERWORLD_AREA_INDEX_SPEXIT)
    }

    pub(crate) fn save_spexit_tm_copy(&mut self) {
        let v = word(self.ram, TM_COPY);
        write_le_u16(self.ram, TM_COPY_SPEXIT, v);
    }

    pub(crate) fn restore_spexit_layer_masks(&mut self) {
        let value = word(self.ram, TM_COPY_SPEXIT);
        write_le_u16(self.ram, TM_COPY, value);
    }

    pub(crate) fn set_spexit_screen_index(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCREEN_INDEX_SPEXIT, value);
    }

    pub(crate) fn spexit_screen_index(&self) -> u16 {
        word(self.ram, OVERWORLD_SCREEN_INDEX_SPEXIT)
    }

    pub(crate) fn set_spexit_map16_src_off(&mut self, value: u16) {
        write_le_u16(self.ram, MAP16_LOAD_SRC_OFF_SPEXIT, value);
    }

    pub(crate) fn save_spexit_camera_coords(&mut self) {
        let cy = word(self.ram, CAMERA_Y_COORD_SCROLL_LOW);
        let cx = word(self.ram, CAMERA_X_COORD_SCROLL_LOW);
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_LOW_SPEXIT, cy);
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_LOW_SPEXIT, cx);
    }

    pub(crate) fn spexit_camera_y_scroll_low(&self) -> u16 {
        word(self.ram, CAMERA_Y_COORD_SCROLL_LOW_SPEXIT)
    }

    pub(crate) fn spexit_camera_x_scroll_low(&self) -> u16 {
        word(self.ram, CAMERA_X_COORD_SCROLL_LOW_SPEXIT)
    }

    pub(crate) fn spexit_room_bound_y_start(&self) -> u16 {
        word(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_Y_START)
    }

    pub(crate) fn spexit_room_bound_y_end(&self) -> u16 {
        word(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_Y_END)
    }

    pub(crate) fn spexit_room_bound_x_start(&self) -> u16 {
        word(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_X_START)
    }

    pub(crate) fn spexit_room_bound_x_end(&self) -> u16 {
        word(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_X_END)
    }

    pub(crate) fn save_exit_area_index(&mut self) {
        let v = word(self.ram, OVERWORLD_AREA_INDEX);
        write_le_u16(self.ram, OVERWORLD_AREA_INDEX_EXIT, v);
    }

    pub(crate) fn restore_exit_area_index(&mut self) {
        let value = word(self.ram, OVERWORLD_AREA_INDEX_EXIT);
        write_le_u16(self.ram, OVERWORLD_AREA_INDEX, value);
    }

    pub(crate) fn exit_area_index(&self) -> u16 {
        word(self.ram, OVERWORLD_AREA_INDEX_EXIT)
    }

    pub(crate) fn save_exit_tm_copy(&mut self) {
        let v = word(self.ram, TM_COPY);
        write_le_u16(self.ram, TM_COPY_EXIT, v);
    }

    pub(crate) fn restore_exit_layer_masks(&mut self) {
        let value = word(self.ram, TM_COPY_EXIT);
        write_le_u16(self.ram, TM_COPY, value);
    }

    pub(crate) fn set_exit_screen_index(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCREEN_INDEX_EXIT, value);
    }

    pub(crate) fn exit_screen_index(&self) -> u16 {
        word(self.ram, OVERWORLD_SCREEN_INDEX_EXIT)
    }

    pub(crate) fn set_exit_map16_src_off(&mut self, value: u16) {
        write_le_u16(self.ram, MAP16_LOAD_SRC_OFF_EXIT, value);
    }

    pub(crate) fn save_exit_camera_coords(&mut self) {
        let cy = word(self.ram, CAMERA_Y_COORD_SCROLL_LOW);
        let cx = word(self.ram, CAMERA_X_COORD_SCROLL_LOW);
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_LOW_EXIT, cy);
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_LOW_EXIT, cx);
    }

    pub(crate) fn exit_camera_y_scroll_low(&self) -> u16 {
        word(self.ram, CAMERA_Y_COORD_SCROLL_LOW_EXIT)
    }

    pub(crate) fn exit_camera_x_scroll_low(&self) -> u16 {
        word(self.ram, CAMERA_X_COORD_SCROLL_LOW_EXIT)
    }

    pub(crate) fn restore_exit_camera_scroll(&mut self) {
        let camera_y = word(self.ram, CAMERA_Y_COORD_SCROLL_LOW_EXIT);
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_LOW, camera_y);
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_HI, camera_y.wrapping_sub(2));

        let camera_x = word(self.ram, CAMERA_X_COORD_SCROLL_LOW_EXIT);
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_LOW, camera_x);
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_HI, camera_x.wrapping_sub(2));
    }

    pub(crate) fn restore_special_exit_camera_scroll(&mut self) {
        let camera_y = word(self.ram, CAMERA_Y_COORD_SCROLL_LOW_SPEXIT);
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_LOW, camera_y);
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_HI, camera_y.wrapping_sub(2));

        let camera_x = word(self.ram, CAMERA_X_COORD_SCROLL_LOW_SPEXIT);
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_LOW, camera_x);
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_HI, camera_x.wrapping_sub(2));
    }

    pub(crate) fn restore_exit_tile_themes(&mut self) {
        self.ram[OVERWORLD_TILE_THEME_INDEX] = self.ram[OVERWORLD_TILE_THEME_INDEX_EXIT];
        self.ram[MAIN_TILE_THEME_INDEX] = self.ram[MAIN_TILE_THEME_INDEX_EXIT];
        self.ram[AUX_TILE_THEME_INDEX] = self.ram[AUX_TILE_THEME_INDEX_EXIT];
    }

    pub(crate) fn save_spexit_tile_themes(&mut self) {
        self.ram[OVERWORLD_SPECIAL_TILE_THEME_INDEX] = self.ram[OVERWORLD_TILE_THEME_INDEX];
        self.ram[MAIN_TILE_THEME_INDEX_SPEXIT] = self.ram[MAIN_TILE_THEME_INDEX];
        self.ram[AUX_TILE_THEME_INDEX_SPEXIT] = self.ram[AUX_TILE_THEME_INDEX];
    }

    pub(crate) fn restore_spexit_tile_themes(&mut self) {
        self.ram[OVERWORLD_TILE_THEME_INDEX] = self.ram[OVERWORLD_SPECIAL_TILE_THEME_INDEX];
        self.ram[MAIN_TILE_THEME_INDEX] = self.ram[MAIN_TILE_THEME_INDEX_SPEXIT];
        self.ram[AUX_TILE_THEME_INDEX] = self.ram[AUX_TILE_THEME_INDEX_SPEXIT];
    }

    pub(crate) fn save_prev_screen_trans_bits(&mut self) {
        let bits = self.ram[OVERWORLD_SCREEN_TRANS_DIR_BITS];
        let bits2 = word(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2);
        write_le_u16(
            self.ram,
            OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV,
            u16::from(bits),
        );
        write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV, bits2);
    }

    pub(crate) fn restore_prev_screen_trans_bits(&mut self) {
        let bits = word(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV);
        let bits2 = word(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV);
        write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, bits);
        write_le_u16(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, bits2);
    }

    pub(crate) fn set_prev_screen_transition(&mut self, value: u8) {
        self.ram[OVERWORLD_SCREEN_TRANSITION_PREV] = value;
    }

    pub(crate) fn prev_screen_transition(&self) -> u8 {
        self.ram[OVERWORLD_SCREEN_TRANSITION_PREV]
    }

    pub(crate) fn set_prev_screen_index_word(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCREEN_INDEX_PREV, value);
    }

    pub(crate) fn prev_screen_index_word(&self) -> u16 {
        word(self.ram, OVERWORLD_SCREEN_INDEX_PREV)
    }

    pub(crate) fn prev_screen_index_byte(&self) -> u8 {
        byte(self.ram, OVERWORLD_SCREEN_INDEX_PREV)
    }

    pub(crate) fn clear_overlay_index_word(&mut self) {
        write_le_u16(self.ram, OVERLAY_INDEX, 0);
    }

    pub(crate) fn set_overlay_index_word(&mut self, value: u16) {
        write_le_u16(self.ram, OVERLAY_INDEX, value);
    }

    pub(crate) fn overworld_screen_trans_dir_bits(&self) -> u8 {
        byte(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS)
    }

    pub(crate) fn allow_scroll_z(&self) -> u8 {
        self.ram[ALLOW_SCROLL_Z]
    }

    pub(crate) fn trigger_special_entrance(&self) -> u8 {
        self.ram[TRIGGER_SPECIAL_ENTRANCE]
    }

    pub(crate) fn clear_trigger_special_entrance(&mut self) {
        self.ram[TRIGGER_SPECIAL_ENTRANCE] = 0;
    }

    pub(crate) fn flag_custom_spell_anim_active(&self) -> u8 {
        self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE]
    }

    pub(crate) fn super_bomb_indicator_timer(&self) -> u8 {
        self.ram[SUPER_BOMB_INDICATOR_TIMER]
    }

    pub(crate) fn current_area_of_player_word(&self) -> u16 {
        word(self.ram, CURRENT_AREA_OF_PLAYER)
    }

    pub(crate) fn ow_entrance_value(&self) -> u16 {
        word(self.ram, OW_ENTRANCE_VALUE)
    }

    pub(crate) fn set_ow_entrance_value(&mut self, value: u16) {
        write_le_u16(self.ram, OW_ENTRANCE_VALUE, value);
    }

    pub(crate) fn set_map16_load_src(&mut self, value: u16) {
        write_le_u16(self.ram, MAP16_LOAD_SRC_OFF, value);
    }

    pub(crate) fn set_map16_load_dst(&mut self, value: u16) {
        write_le_u16(self.ram, MAP16_LOAD_DST_OFF, value);
    }

    pub(crate) fn set_map16_load_y_unit(&mut self, value: u16) {
        write_le_u16(self.ram, MAP16_LOAD_Y_UNIT, value);
    }

    pub(crate) fn set_prev_map16_load_state(&mut self, src_off: u16, dst_off: u16, y_unit: u16) {
        write_le_u16(self.ram, MAP16_LOAD_SRC_OFF_PREV, src_off);
        write_le_u16(self.ram, MAP16_LOAD_DST_OFF_PREV, dst_off);
        write_le_u16(self.ram, MAP16_LOAD_Y_UNIT_PREV, y_unit);
    }

    pub(crate) fn set_small_ow_scroll_backup(&mut self, src_off: u16, dst_off: u16, y_unit: u16) {
        write_le_u16(self.ram, ORANGE_BLUE_BARRIER_STATE, src_off);
        write_le_u16(self.ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF, dst_off);
        write_le_u16(self.ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT, y_unit);
    }

    pub(crate) fn dung_replacement_tile_state(&self, index: usize) -> u16 {
        word(self.ram, DUNG_REPLACEMENT_TILE_STATE + index * 2)
    }

    pub(crate) fn restore_scroll_targets_from_cached(&mut self) {
        let v0 = word(self.ram, UP_DOWN_SCROLL_TARGET_CACHED);
        let v1 = word(self.ram, UP_DOWN_SCROLL_TARGET_END_CACHED);
        let v2 = word(self.ram, LEFT_RIGHT_SCROLL_TARGET_CACHED);
        let v3 = word(self.ram, LEFT_RIGHT_SCROLL_TARGET_END_CACHED);
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET, v0);
        write_le_u16(self.ram, UP_DOWN_SCROLL_TARGET_END, v1);
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET, v2);
        write_le_u16(self.ram, LEFT_RIGHT_SCROLL_TARGET_END, v3);
    }

    pub(crate) fn restore_camera_y_from_cached_indoor(&mut self) {
        let cy = word(self.ram, CAMERA_Y_COORD_SCROLL_LOW_CACHED);
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_LOW, cy);
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_HI, cy.wrapping_add(2));
    }

    pub(crate) fn restore_camera_x_from_cached_indoor(&mut self) {
        let cx = word(self.ram, CAMERA_X_COORD_SCROLL_LOW_CACHED);
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_LOW, cx);
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_HI, cx.wrapping_add(2));
    }

    pub(crate) fn update_camera_hi_outdoor(&mut self) {
        let cy = word(self.ram, CAMERA_Y_COORD_SCROLL_LOW);
        write_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_HI, cy.wrapping_sub(2));
        let cx = word(self.ram, CAMERA_X_COORD_SCROLL_LOW);
        write_le_u16(self.ram, CAMERA_X_COORD_SCROLL_HI, cx.wrapping_sub(2));
    }

    pub(crate) fn restore_quadrant_fullsize_from_cached(&mut self) {
        self.ram[QUADRANT_FULLSIZE_X] = self.ram[QUADRANT_FULLSIZE_X_CACHED];
    }

    pub(crate) fn set_overworld_offset_base_y(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_OFFSET_BASE_Y, value);
    }

    pub(crate) fn set_overworld_offset_base_x(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_OFFSET_BASE_X, value);
    }

    pub(crate) fn set_overworld_offset_mask_y(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_OFFSET_MASK_Y, value);
    }

    pub(crate) fn set_overworld_offset_mask_x(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_OFFSET_MASK_X, value);
    }

    pub(crate) fn set_dung_replacement_tile_state(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNG_REPLACEMENT_TILE_STATE + index * 2, value);
    }

    pub(crate) fn birdtravel_status_word(&self) -> u16 {
        word(self.ram, BIRDTRAVEL_STATUS)
    }

    pub(crate) fn set_birdtravel_status_word(&mut self, value: u16) {
        write_le_u16(self.ram, BIRDTRAVEL_STATUS, value);
    }
}

impl<'a> WorldStateView<'a> {
    pub(crate) fn last_light_vs_dark_world(&self) -> u8 {
        byte(self.ram, LAST_LIGHT_VS_DARK_WORLD)
    }

    pub(crate) fn which_entrance(&self) -> u16 {
        word(self.ram, WHICH_ENTRANCE)
    }

    pub(crate) fn overworld_hole_tilemap_pos(&self) -> u8 {
        byte(self.ram, OVERWORLD_HOLE_TILEMAP_POS)
    }

    pub(crate) fn aux_bg_subset(&self, index: usize) -> u8 {
        byte(self.ram, AUX_BG_SUBSET_0 + index)
    }

    pub(crate) fn overworld_palette_aux1_hi(&self) -> u8 {
        byte(self.ram, OVERWORLD_PALETTE_AUX1_BP2TO4_HI)
    }

    pub(crate) fn overworld_palette_mode(&self) -> u8 {
        byte(self.ram, OVERWORLD_PALETTE_MODE)
    }

    pub(crate) fn palette_main_indoors(&self) -> u8 {
        byte(self.ram, PALETTE_MAIN_INDOORS)
    }

    pub(crate) fn palette_main_indoors_copy(&self) -> u8 {
        byte(self.ram, PALETTE_MAIN_INDOORS_COPY)
    }

    pub(crate) fn palette_swap_flag(&self) -> u8 {
        byte(self.ram, PALETTE_SWAP_FLAG)
    }

    pub(crate) fn palette_sp0l(&self) -> u8 {
        byte(self.ram, PALETTE_SP0L)
    }

    pub(crate) fn palette_sp5l(&self) -> u8 {
        byte(self.ram, PALETTE_SP5L)
    }

    pub(crate) fn palette_sp6l(&self) -> u8 {
        byte(self.ram, PALETTE_SP6L)
    }

    pub(crate) fn palette_sp6r_indoors(&self) -> u8 {
        byte(self.ram, PALETTE_SP6R_INDOORS)
    }

    pub(crate) fn hud_palette(&self) -> u8 {
        byte(self.ram, HUD_PALETTE)
    }

    pub(crate) fn overworld_palette_aux2_hi(&self) -> u8 {
        byte(self.ram, OVERWORLD_PALETTE_AUX2_BP5TO7_HI)
    }

    pub(crate) fn overworld_palette_aux3_lo(&self) -> u8 {
        byte(self.ram, OVERWORLD_PALETTE_AUX3_BP7_LO)
    }

    pub(crate) fn misc_sprites_graphics_index(&self) -> u8 {
        byte(self.ram, MISC_SPRITES_GRAPHICS_INDEX)
    }

    pub(crate) fn main_tile_theme_index(&self) -> u8 {
        byte(self.ram, MAIN_TILE_THEME_INDEX)
    }

    pub(crate) fn aux_tile_theme_index(&self) -> u8 {
        byte(self.ram, AUX_TILE_THEME_INDEX)
    }

    pub(crate) fn overworld_map_flags(&self) -> u8 {
        byte(self.ram, OVERWORLD_MAP_FLAGS)
    }

    pub(crate) fn timer_for_mode7_zoom(&self) -> u8 {
        byte(self.ram, TIMER_FOR_MODE7_ZOOM)
    }

    pub(crate) fn birdtravel_status(&self) -> u8 {
        byte(self.ram, BIRDTRAVEL_STATUS)
    }

    pub(crate) fn birdtravel_status_word(&self) -> u16 {
        word(self.ram, BIRDTRAVEL_STATUS)
    }

    pub(crate) fn hud_cur_item_x(&self) -> u8 {
        byte(self.ram, HUD_CUR_ITEM_X)
    }

    pub(crate) fn overworld_screen_trans_dir_bits(&self) -> u8 {
        byte(self.ram, OVERWORLD_SCREEN_TRANS_DIR_BITS)
    }

    pub(crate) fn ow_countdown_transition(&self) -> u8 {
        self.ram[OW_COUNTDOWN_TRANSITION]
    }

    pub(crate) fn transition_counter(&self) -> u8 {
        self.ram[TRANSITION_COUNTER]
    }

    pub(crate) fn transition_dir_enum(&self) -> u8 {
        self.ram[OVERWORLD_TRANSITION_DIR]
    }

    pub(crate) fn door_animation_step(&self) -> u16 {
        word(self.ram, DOOR_ANIMATION_STEP_INDICATOR)
    }

    pub(crate) fn big_key_door_message_triggered(&self) -> u16 {
        word(self.ram, BIG_KEY_DOOR_MESSAGE_TRIGGERED)
    }

    pub(crate) fn overworld_peg_puzzle_progress(&self) -> u16 {
        word(self.ram, OVERWORLD_PEG_PUZZLE_PROGRESS)
    }

    pub(crate) fn camera_y_coord_scroll_low(&self) -> u16 {
        word(self.ram, CAMERA_Y_COORD_SCROLL_LOW)
    }

    pub(crate) fn camera_y_coord_scroll_hi(&self) -> u16 {
        word(self.ram, CAMERA_Y_COORD_SCROLL_HI)
    }

    pub(crate) fn camera_x_coord_scroll_low(&self) -> u16 {
        word(self.ram, CAMERA_X_COORD_SCROLL_LOW)
    }

    pub(crate) fn camera_x_coord_scroll_hi(&self) -> u16 {
        word(self.ram, CAMERA_X_COORD_SCROLL_HI)
    }

    pub(crate) fn up_down_scroll_target(&self, index: usize) -> u16 {
        word(self.ram, UP_DOWN_SCROLL_TARGET + index * 2)
    }

    pub(crate) fn overworld_scroll_counter_for_axis(&self, ya: usize) -> u16 {
        word(self.ram, OVERWORLD_SCROLL_UP_COUNTER + ya * 2)
    }

    pub(crate) fn savegame_has_master_sword_flags(&self) -> u16 {
        word(self.ram, SAVEGAME_HAS_MASTER_SWORD_FLAGS)
    }

    pub(crate) fn allow_scroll_z(&self) -> u8 {
        self.ram[ALLOW_SCROLL_Z]
    }

    pub(crate) fn trigger_special_entrance(&self) -> u8 {
        self.ram[TRIGGER_SPECIAL_ENTRANCE]
    }

    pub(crate) fn flag_custom_spell_anim_active(&self) -> u8 {
        self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE]
    }

    pub(crate) fn super_bomb_indicator_timer(&self) -> u8 {
        self.ram[SUPER_BOMB_INDICATOR_TIMER]
    }

    pub(crate) fn current_area_of_player_word(&self) -> u16 {
        word(self.ram, CURRENT_AREA_OF_PLAYER)
    }

    pub(crate) fn overworld_area_index_word(&self) -> u16 {
        word(self.ram, OVERWORLD_AREA_INDEX)
    }

    pub(crate) fn spexit_area_index(&self) -> u16 {
        word(self.ram, OVERWORLD_AREA_INDEX_SPEXIT)
    }

    pub(crate) fn spexit_screen_index(&self) -> u16 {
        word(self.ram, OVERWORLD_SCREEN_INDEX_SPEXIT)
    }

    pub(crate) fn spexit_camera_y_scroll_low(&self) -> u16 {
        word(self.ram, CAMERA_Y_COORD_SCROLL_LOW_SPEXIT)
    }

    pub(crate) fn spexit_camera_x_scroll_low(&self) -> u16 {
        word(self.ram, CAMERA_X_COORD_SCROLL_LOW_SPEXIT)
    }

    pub(crate) fn spexit_room_bound_y_start(&self) -> u16 {
        word(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_Y_START)
    }

    pub(crate) fn spexit_room_bound_y_end(&self) -> u16 {
        word(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_Y_END)
    }

    pub(crate) fn spexit_room_bound_x_start(&self) -> u16 {
        word(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_X_START)
    }

    pub(crate) fn spexit_room_bound_x_end(&self) -> u16 {
        word(self.ram, SPECIAL_EXIT_ROOM_BOUNDS_X_END)
    }

    pub(crate) fn prev_screen_index_byte(&self) -> u8 {
        byte(self.ram, OVERWORLD_SCREEN_INDEX_PREV)
    }

    pub(crate) fn prev_screen_index_word(&self) -> u16 {
        word(self.ram, OVERWORLD_SCREEN_INDEX_PREV)
    }

    pub(crate) fn prev_screen_transition(&self) -> u8 {
        self.ram[OVERWORLD_SCREEN_TRANSITION_PREV]
    }

    pub(crate) fn ow_entrance_value(&self) -> u16 {
        word(self.ram, OW_ENTRANCE_VALUE)
    }

    pub(crate) fn dung_replacement_tile_state(&self, index: usize) -> u16 {
        word(self.ram, DUNG_REPLACEMENT_TILE_STATE + index * 2)
    }

    pub(crate) fn is_standing_in_doorway_cached(&self) -> u8 {
        self.ram[IS_STANDING_IN_DOORWAY_CACHED]
    }
}
