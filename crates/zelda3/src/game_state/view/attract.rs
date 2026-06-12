use super::*;

pub(crate) struct AttractStateView<'a> {
    ram: &'a [u8],
}

impl<'a> AttractStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn state(&self) -> u8 {
        byte(self.ram, ATTRACT_STATE)
    }

    pub(crate) fn state_word(&self) -> u16 {
        word(self.ram, ATTRACT_STATE)
    }

    pub(crate) fn sequence(&self) -> u8 {
        byte(self.ram, ATTRACT_SEQUENCE)
    }

    pub(crate) fn scene_timer(&self) -> u8 {
        byte(self.ram, ATTRACT_SCENE_TIMER)
    }

    pub(crate) fn scene_substep(&self) -> u8 {
        byte(self.ram, ATTRACT_SCENE_SUBSTEP)
    }

    pub(crate) fn x_base(&self) -> u8 {
        byte(self.ram, ATTRACT_X_BASE)
    }

    pub(crate) fn x_base_word(&self) -> u16 {
        word(self.ram, ATTRACT_X_BASE)
    }

    pub(crate) fn x_base_high(&self) -> u8 {
        byte(self.ram, ATTRACT_X_BASE_HI)
    }

    pub(crate) fn y_base(&self) -> u8 {
        byte(self.ram, ATTRACT_Y_BASE)
    }

    pub(crate) fn oam_index(&self) -> u8 {
        byte(self.ram, ATTRACT_OAM_IDX)
    }

    pub(crate) fn maiden_warp_step(&self) -> u8 {
        byte(self.ram, ATTRACT_MAIDEN_WARP_STEP)
    }

    pub(crate) fn intro_step_index(&self) -> u8 {
        byte(self.ram, INTRO_STEP_INDEX)
    }

    pub(crate) fn intro_step_timer(&self) -> u8 {
        byte(self.ram, INTRO_STEP_TIMER)
    }

    pub(crate) fn intro_frame_counter(&self) -> u8 {
        byte(self.ram, INTRO_FRAME_CTR)
    }

    pub(crate) fn intro_did_run_step(&self) -> u8 {
        byte(self.ram, INTRO_DID_RUN_STEP)
    }

    pub(crate) fn intro_palette_flash_count(&self) -> u8 {
        byte(self.ram, INTRO_TIMES_PAL_FLASH)
    }

    pub(crate) fn legend_flag(&self) -> u8 {
        byte(self.ram, ATTRACT_LEGEND_FLAG)
    }
    pub(crate) fn next_legend_gfx(&self) -> u8 {
        byte(self.ram, ATTRACT_NEXT_LEGEND_GFX)
    }
    pub(crate) fn next_legend_image(&self) -> u8 {
        byte(self.ram, ATTRACT_NEXT_LEGEND_GFX) >> 1
    }
    pub(crate) fn bg2_vofs_backup(&self) -> u16 {
        word(self.ram, ATTRACT_BG2_VOFS_BACKUP)
    }
    pub(crate) fn throne_fade_timer(&self) -> u8 {
        byte(self.ram, ATTRACT_THRONE_FADE_TIMER)
    }
    pub(crate) fn prison_zelda_y_base(&self) -> u8 {
        byte(self.ram, ATTRACT_PRISON_ZELDA_Y_BASE)
    }
    pub(crate) fn anim_step_counter(&self) -> u8 {
        byte(self.ram, ATTRACT_ANIM_STEP_COUNTER)
    }
    pub(crate) fn soldier_anim_step(&self) -> u8 {
        byte(self.ram, ATTRACT_SOLDIER_ANIM_STEP)
    }
    pub(crate) fn prison_soldier_x_lo(&self) -> u8 {
        byte(self.ram, ATTRACT_PRISON_SOLDIER_X_LO)
    }
    pub(crate) fn scene_frame_counter(&self) -> u8 {
        byte(self.ram, ATTRACT_SCENE_FRAME_COUNTER)
    }
    pub(crate) fn scene_done_flag(&self) -> u8 {
        byte(self.ram, ATTRACT_SCENE_DONE_FLAG)
    }
    pub(crate) fn legend_ctr(&self) -> u16 {
        word(self.ram, ATTRACT_LEGEND_CTR)
    }
    pub(crate) fn fade_in_complete_flag(&self) -> u8 {
        byte(self.ram, ATTRACT_FADE_IN_COMPLETE_FLAG)
    }
    pub(crate) fn fade_in_done_flag(&self) -> u8 {
        byte(self.ram, ATTRACT_FADE_IN_DONE_FLAG)
    }
    pub(crate) fn substep_delay_counter(&self) -> u8 {
        byte(self.ram, ATTRACT_SUBSTEP_DELAY_COUNTER)
    }
    pub(crate) fn maiden_warp_timer_a(&self) -> u8 {
        byte(self.ram, ATTRACT_MAIDEN_WARP_TIMER_A)
    }
    pub(crate) fn maiden_warp_timer_b(&self) -> u8 {
        byte(self.ram, ATTRACT_MAIDEN_WARP_TIMER_B)
    }
    pub(crate) fn mode7_zoom_timer(&self) -> u8 {
        byte(self.ram, TIMER_FOR_MODE7_ZOOM)
    }
}

pub(crate) struct AttractStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> AttractStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_state(&mut self, value: u8) {
        self.ram[ATTRACT_STATE] = value;
    }

    pub(crate) fn set_state_word(&mut self, value: u16) {
        write_le_u16(self.ram, ATTRACT_STATE, value);
    }

    pub(crate) fn increment_state(&mut self) -> u8 {
        self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(1);
        self.ram[ATTRACT_STATE]
    }

    pub(crate) fn add_state(&mut self, value: u8) -> u8 {
        self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_add(value);
        self.ram[ATTRACT_STATE]
    }

    pub(crate) fn subtract_state(&mut self, value: u8) -> u8 {
        self.ram[ATTRACT_STATE] = self.ram[ATTRACT_STATE].wrapping_sub(value);
        self.ram[ATTRACT_STATE]
    }

    pub(crate) fn set_sequence(&mut self, value: u8) {
        self.ram[ATTRACT_SEQUENCE] = value;
    }

    pub(crate) fn increment_sequence(&mut self) -> u8 {
        self.ram[ATTRACT_SEQUENCE] = self.ram[ATTRACT_SEQUENCE].wrapping_add(1);
        self.ram[ATTRACT_SEQUENCE]
    }

    pub(crate) fn set_scene_timer(&mut self, value: u8) {
        self.ram[ATTRACT_SCENE_TIMER] = value;
    }

    pub(crate) fn decrement_scene_timer(&mut self) -> u8 {
        self.ram[ATTRACT_SCENE_TIMER] = self.ram[ATTRACT_SCENE_TIMER].wrapping_sub(1);
        self.ram[ATTRACT_SCENE_TIMER]
    }

    pub(crate) fn set_scene_substep(&mut self, value: u8) {
        self.ram[ATTRACT_SCENE_SUBSTEP] = value;
    }

    pub(crate) fn increment_scene_substep(&mut self) -> u8 {
        self.ram[ATTRACT_SCENE_SUBSTEP] = self.ram[ATTRACT_SCENE_SUBSTEP].wrapping_add(1);
        self.ram[ATTRACT_SCENE_SUBSTEP]
    }

    pub(crate) fn set_x_base(&mut self, value: u8) {
        self.ram[ATTRACT_X_BASE] = value;
    }

    pub(crate) fn set_x_base_high(&mut self, value: u8) {
        self.ram[ATTRACT_X_BASE_HI] = value;
    }

    pub(crate) fn set_y_base(&mut self, value: u8) {
        self.ram[ATTRACT_Y_BASE] = value;
    }

    pub(crate) fn set_story_text_pointer(&mut self, value: u16) {
        write_le_u16(self.ram, ATTRACT_STORY_TEXT_POINTER, value);
    }

    pub(crate) fn set_oam_index(&mut self, value: u8) {
        self.ram[ATTRACT_OAM_IDX] = value;
    }

    pub(crate) fn advance_oam_index_by(&mut self, value: u8) -> u8 {
        self.ram[ATTRACT_OAM_IDX] = self.ram[ATTRACT_OAM_IDX].wrapping_add(value);
        self.ram[ATTRACT_OAM_IDX]
    }

    pub(crate) fn set_maiden_warp_step(&mut self, value: u8) {
        self.ram[ATTRACT_MAIDEN_WARP_STEP] = value;
    }

    pub(crate) fn increment_maiden_warp_step(&mut self) -> u8 {
        self.ram[ATTRACT_MAIDEN_WARP_STEP] = self.ram[ATTRACT_MAIDEN_WARP_STEP].wrapping_add(1);
        self.ram[ATTRACT_MAIDEN_WARP_STEP]
    }

    pub(crate) fn decrement_maiden_warp_step(&mut self) -> u8 {
        self.ram[ATTRACT_MAIDEN_WARP_STEP] = self.ram[ATTRACT_MAIDEN_WARP_STEP].wrapping_sub(1);
        self.ram[ATTRACT_MAIDEN_WARP_STEP]
    }

    pub(crate) fn set_intro_step_index(&mut self, value: u8) {
        self.ram[INTRO_STEP_INDEX] = value;
    }

    pub(crate) fn clear_intro_step_state_block(&mut self) {
        self.ram[INTRO_STEP_INDEX..INTRO_STEP_INDEX + 7 * 16].fill(0);
    }

    pub(crate) fn increment_intro_step_index(&mut self) -> u8 {
        self.ram[INTRO_STEP_INDEX] = self.ram[INTRO_STEP_INDEX].wrapping_add(1);
        self.ram[INTRO_STEP_INDEX]
    }

    pub(crate) fn set_intro_step_timer(&mut self, value: u8) {
        self.ram[INTRO_STEP_TIMER] = value;
    }

    pub(crate) fn increment_intro_step_timer(&mut self) -> u8 {
        self.ram[INTRO_STEP_TIMER] = self.ram[INTRO_STEP_TIMER].wrapping_add(1);
        self.ram[INTRO_STEP_TIMER]
    }

    pub(crate) fn decrement_intro_step_timer(&mut self) -> u8 {
        self.ram[INTRO_STEP_TIMER] = self.ram[INTRO_STEP_TIMER].wrapping_sub(1);
        self.ram[INTRO_STEP_TIMER]
    }

    pub(crate) fn increment_intro_frame_counter(&mut self) -> u8 {
        self.ram[INTRO_FRAME_CTR] = self.ram[INTRO_FRAME_CTR].wrapping_add(1);
        self.ram[INTRO_FRAME_CTR]
    }

    pub(crate) fn set_intro_did_run_step(&mut self, value: u8) {
        self.ram[INTRO_DID_RUN_STEP] = value;
    }

    pub(crate) fn clear_intro_did_run_step(&mut self) {
        self.ram[INTRO_DID_RUN_STEP] = 0;
    }

    pub(crate) fn mark_intro_did_run_step(&mut self) {
        self.ram[INTRO_DID_RUN_STEP] = 1;
    }

    pub(crate) fn set_intro_palette_flash_count(&mut self, value: u8) {
        self.ram[INTRO_TIMES_PAL_FLASH] = value;
    }

    pub(crate) fn clear_intro_palette_flash_count(&mut self) {
        self.ram[INTRO_TIMES_PAL_FLASH] = 0;
    }

    pub(crate) fn decrement_intro_palette_flash_count(&mut self) -> u8 {
        self.ram[INTRO_TIMES_PAL_FLASH] = self.ram[INTRO_TIMES_PAL_FLASH].wrapping_sub(1);
        self.ram[INTRO_TIMES_PAL_FLASH]
    }

    pub(crate) fn increment_legend_flag(&mut self) {
        self.ram[ATTRACT_LEGEND_FLAG] = self.ram[ATTRACT_LEGEND_FLAG].wrapping_add(1);
    }
    pub(crate) fn clear_legend_flag(&mut self) {
        self.ram[ATTRACT_LEGEND_FLAG] = 0;
    }
    pub(crate) fn clear_next_legend_gfx(&mut self) {
        self.ram[ATTRACT_NEXT_LEGEND_GFX] = 0;
    }
    pub(crate) fn advance_next_legend_gfx(&mut self) {
        self.ram[ATTRACT_NEXT_LEGEND_GFX] = self.ram[ATTRACT_NEXT_LEGEND_GFX].wrapping_add(2);
    }
    pub(crate) fn set_bg2_vofs_backup(&mut self, value: u16) {
        write_le_u16(self.ram, ATTRACT_BG2_VOFS_BACKUP, value);
    }
    pub(crate) fn set_throne_fade_timer(&mut self, value: u8) {
        self.ram[ATTRACT_THRONE_FADE_TIMER] = value;
    }
    pub(crate) fn decrement_throne_fade_timer(&mut self) -> u8 {
        self.ram[ATTRACT_THRONE_FADE_TIMER] = self.ram[ATTRACT_THRONE_FADE_TIMER].wrapping_sub(1);
        self.ram[ATTRACT_THRONE_FADE_TIMER]
    }
    pub(crate) fn set_prison_zelda_y_base(&mut self, value: u8) {
        self.ram[ATTRACT_PRISON_ZELDA_Y_BASE] = value;
    }
    pub(crate) fn decrement_prison_zelda_y_base(&mut self) {
        self.ram[ATTRACT_PRISON_ZELDA_Y_BASE] =
            self.ram[ATTRACT_PRISON_ZELDA_Y_BASE].wrapping_sub(1);
    }
    pub(crate) fn set_anim_step_counter(&mut self, value: u8) {
        self.ram[ATTRACT_ANIM_STEP_COUNTER] = value;
    }
    pub(crate) fn decrement_anim_step_counter(&mut self) -> u8 {
        self.ram[ATTRACT_ANIM_STEP_COUNTER] = self.ram[ATTRACT_ANIM_STEP_COUNTER].wrapping_sub(1);
        self.ram[ATTRACT_ANIM_STEP_COUNTER]
    }
    pub(crate) fn increment_anim_step_counter(&mut self) -> u8 {
        self.ram[ATTRACT_ANIM_STEP_COUNTER] = self.ram[ATTRACT_ANIM_STEP_COUNTER].wrapping_add(1);
        self.ram[ATTRACT_ANIM_STEP_COUNTER]
    }
    pub(crate) fn set_soldier_anim_step(&mut self, value: u8) {
        self.ram[ATTRACT_SOLDIER_ANIM_STEP] = value;
    }
    pub(crate) fn increment_soldier_anim_step(&mut self) {
        self.ram[ATTRACT_SOLDIER_ANIM_STEP] = self.ram[ATTRACT_SOLDIER_ANIM_STEP].wrapping_add(1);
    }
    pub(crate) fn set_prison_soldier_x_lo(&mut self, value: u8) {
        self.ram[ATTRACT_PRISON_SOLDIER_X_LO] = value;
    }
    pub(crate) fn set_scene_frame_counter(&mut self, value: u8) {
        self.ram[ATTRACT_SCENE_FRAME_COUNTER] = value;
    }
    pub(crate) fn increment_scene_frame_counter(&mut self) -> u8 {
        self.ram[ATTRACT_SCENE_FRAME_COUNTER] =
            self.ram[ATTRACT_SCENE_FRAME_COUNTER].wrapping_add(1);
        self.ram[ATTRACT_SCENE_FRAME_COUNTER]
    }
    pub(crate) fn decrement_scene_frame_counter(&mut self) -> u8 {
        self.ram[ATTRACT_SCENE_FRAME_COUNTER] =
            self.ram[ATTRACT_SCENE_FRAME_COUNTER].wrapping_sub(1);
        self.ram[ATTRACT_SCENE_FRAME_COUNTER]
    }
    pub(crate) fn increment_scene_done_flag(&mut self) {
        self.ram[ATTRACT_SCENE_DONE_FLAG] = self.ram[ATTRACT_SCENE_DONE_FLAG].wrapping_add(1);
    }
    pub(crate) fn set_legend_ctr(&mut self, value: u16) {
        write_le_u16(self.ram, ATTRACT_LEGEND_CTR, value);
    }
    pub(crate) fn decrement_legend_ctr(&mut self) -> u16 {
        let v = read_le_u16(self.ram, ATTRACT_LEGEND_CTR).wrapping_sub(1);
        write_le_u16(self.ram, ATTRACT_LEGEND_CTR, v);
        v
    }
    pub(crate) fn set_fade_in_complete_flag(&mut self, value: u8) {
        self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG] = value;
    }
    pub(crate) fn increment_fade_in_complete_flag(&mut self) {
        self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG] =
            self.ram[ATTRACT_FADE_IN_COMPLETE_FLAG].wrapping_add(1);
    }
    pub(crate) fn clear_fade_in_done_flag(&mut self) {
        self.ram[ATTRACT_FADE_IN_DONE_FLAG] = 0;
    }
    pub(crate) fn increment_fade_in_done_flag(&mut self) {
        self.ram[ATTRACT_FADE_IN_DONE_FLAG] = self.ram[ATTRACT_FADE_IN_DONE_FLAG].wrapping_add(1);
    }
    pub(crate) fn clear_substep_delay_counter(&mut self) {
        self.ram[ATTRACT_SUBSTEP_DELAY_COUNTER] = 0;
    }
    pub(crate) fn increment_substep_delay_counter(&mut self) {
        self.ram[ATTRACT_SUBSTEP_DELAY_COUNTER] =
            self.ram[ATTRACT_SUBSTEP_DELAY_COUNTER].wrapping_add(1);
    }
    pub(crate) fn set_maiden_warp_timer_a(&mut self, value: u8) {
        self.ram[ATTRACT_MAIDEN_WARP_TIMER_A] = value;
    }
    pub(crate) fn decrement_maiden_warp_timer_a(&mut self) -> u8 {
        self.ram[ATTRACT_MAIDEN_WARP_TIMER_A] =
            self.ram[ATTRACT_MAIDEN_WARP_TIMER_A].wrapping_sub(1);
        self.ram[ATTRACT_MAIDEN_WARP_TIMER_A]
    }
    pub(crate) fn set_maiden_warp_timer_b(&mut self, value: u8) {
        self.ram[ATTRACT_MAIDEN_WARP_TIMER_B] = value;
    }
    pub(crate) fn decrement_maiden_warp_timer_b(&mut self) -> u8 {
        self.ram[ATTRACT_MAIDEN_WARP_TIMER_B] =
            self.ram[ATTRACT_MAIDEN_WARP_TIMER_B].wrapping_sub(1);
        self.ram[ATTRACT_MAIDEN_WARP_TIMER_B]
    }
    pub(crate) fn set_mode7_zoom_timer(&mut self, value: u8) {
        self.ram[TIMER_FOR_MODE7_ZOOM] = value;
    }
    pub(crate) fn decrement_mode7_zoom_timer(&mut self) {
        self.ram[TIMER_FOR_MODE7_ZOOM] = self.ram[TIMER_FOR_MODE7_ZOOM].wrapping_sub(1);
    }
}
