//! Player animation, OAM, graphics staging, and draw state.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct PlayerPresentationState {
    pub(super) oam_x_offset: u8,
    pub(super) oam_y_offset: u8,
    pub(super) visibility_status: u8,
    pub(super) blink_countdown: u8,
    pub(super) spin_animation_step_counter: u8,
    pub(super) animation_step: u8,
    pub(super) opening_pose: u8,
    pub(super) water_ripple_or_grass_state: u8,
    pub(super) primary_water_grass_timer: u8,
    pub(super) secondary_water_grass_timer: u8,
    pub(super) frame_change_counter: u8,
    pub(super) sprite_oam_state_timer: u8,
    pub(super) throw_oam_state_index: u8,
    pub(super) item_hold_pose: u8,
    pub(super) force_hold_sword_up: u8,
    pub(super) dash_noise_requested: u8,
    pub(super) faint_animation_active: u8,
    pub(super) index_of_dashing_sfx: u8,
    pub(super) spin_offsets: u8,
    pub(super) link_dma_graphics_index: u16,
    pub(super) link_dma_left_sprite_bank: u16,
    pub(super) link_dma_right_sprite_bank: u16,
    pub(super) sword_dma_graphics_index: u8,
    pub(super) shield_dma_graphics_index: u8,
    pub(super) link_dma_staging_index: u8,
    pub(super) palette_bits_of_oam: u16,
    pub(super) player_pose_draw_counter: u8,
    pub(super) player_special_draw_flag: u8,
    pub(super) sleep_in_bed_state: u8,
}

impl PlayerPresentationState {
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

    pub(crate) fn item_hold_pose(&self) -> u8 {
        self.item_hold_pose
    }

    pub(crate) fn force_hold_sword_up_state(&self) -> u8 {
        self.force_hold_sword_up
    }

    pub(crate) fn visibility_status(&self) -> u8 {
        self.visibility_status
    }

    pub(crate) fn link_dma_graphics_index_word(&self) -> u16 {
        self.link_dma_graphics_index
    }

    pub(crate) fn link_dma_staging_index(&self) -> u8 {
        self.link_dma_staging_index
    }

    pub(crate) fn blink_countdown(&self) -> u8 {
        self.blink_countdown
    }

    pub(crate) fn spin_animation_step_counter(&self) -> u8 {
        self.spin_animation_step_counter
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

    pub(crate) fn sleep_in_bed_state(&self) -> u8 {
        self.sleep_in_bed_state
    }

    pub(crate) fn throw_oam_state_index(&self) -> u8 {
        self.throw_oam_state_index
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

    pub(crate) fn player_pose_draw_counter(&self) -> u8 {
        self.player_pose_draw_counter
    }

    pub(crate) fn player_special_draw_flag(&self) -> u8 {
        self.player_special_draw_flag
    }

    pub(crate) fn spin_offsets(&self) -> u8 {
        self.spin_offsets
    }

    pub(crate) fn dash_noise_requested(&self) -> bool {
        self.dash_noise_requested != 0
    }

    pub(crate) fn faint_animation_active(&self) -> u8 {
        self.faint_animation_active
    }

    pub(crate) fn index_of_dashing_sfx(&self) -> u8 {
        self.index_of_dashing_sfx
    }

    pub(super) fn set_oam_x_offset(&mut self, value: u8) {
        self.oam_x_offset = value;
    }

    pub(super) fn set_oam_y_offset(&mut self, value: u8) {
        self.oam_y_offset = value;
    }

    pub(super) fn set_oam_offset(&mut self, y: u8, x: u8) {
        self.oam_y_offset = y;
        self.oam_x_offset = x;
    }

    pub(super) fn disable_oam_offsets(&mut self) {
        self.set_oam_offset(0x80, 0x80);
    }

    pub(super) fn set_visibility_status(&mut self, value: u8) {
        self.visibility_status = value;
    }

    pub(super) fn clear_faint_animation_active(&mut self) {
        self.faint_animation_active = 0;
    }

    pub(super) fn set_faint_animation_active(&mut self, value: u8) {
        self.faint_animation_active = value;
    }

    pub(super) fn set_dash_noise_request(&mut self) {
        self.dash_noise_requested = 1;
    }

    pub(super) fn clear_dash_noise_request(&mut self) {
        self.dash_noise_requested = 0;
    }

    pub(super) fn set_spin_offsets(&mut self, value: u8) {
        self.spin_offsets = value;
    }

    pub(super) fn clear_blink_countdown(&mut self) {
        self.blink_countdown = 0;
    }

    pub(super) fn set_blink_countdown(&mut self, value: u8) {
        self.blink_countdown = value;
    }

    pub(super) fn decrement_blink_countdown(&mut self) -> u8 {
        self.blink_countdown = self.blink_countdown.wrapping_sub(1);
        self.blink_countdown
    }

    pub(super) fn set_spin_animation_step_counter(&mut self, value: u8) {
        self.spin_animation_step_counter = value;
    }

    pub(super) fn increment_spin_animation_step_counter(&mut self) -> u8 {
        self.spin_animation_step_counter = self.spin_animation_step_counter.wrapping_add(1);
        self.spin_animation_step_counter
    }

    pub(super) fn clear_spin_animation_step_counter(&mut self) {
        self.spin_animation_step_counter = 0;
    }

    pub(super) fn clear_animation_step(&mut self) {
        self.animation_step = 0;
    }

    pub(super) fn set_animation_step(&mut self, value: u8) {
        self.animation_step = value;
    }

    pub(super) fn increment_opening_pose(&mut self) {
        self.opening_pose = self.opening_pose.wrapping_add(1);
    }

    pub(super) fn advance_animation_step(&mut self, wrap_at: u8, wrap_to: u8) {
        self.animation_step = self.animation_step.wrapping_add(1);
        if self.animation_step == wrap_at {
            self.animation_step = wrap_to;
        }
    }

    pub(super) fn advance_animation_step_at_least(&mut self, wrap_at: u8, wrap_to: u8) {
        self.animation_step = self.animation_step.wrapping_add(1);
        if self.animation_step >= wrap_at {
            self.animation_step = wrap_to;
        }
    }

    pub(super) fn clear_animation_step_if_at_least(&mut self, threshold: u8) {
        if self.animation_step >= threshold {
            self.clear_animation_step();
        }
    }

    pub(super) fn subtract_animation_step_if_at_least(&mut self, threshold: u8, delta: u8) {
        if self.animation_step >= threshold {
            self.animation_step = self.animation_step.wrapping_sub(delta);
        }
    }

    pub(super) fn clear_water_ripple_or_grass_state(&mut self) {
        self.water_ripple_or_grass_state = 0;
    }

    pub(super) fn set_water_ripple_or_grass_state(&mut self, value: u8) {
        self.water_ripple_or_grass_state = value;
    }

    pub(super) fn set_secondary_water_grass_timer(&mut self, value: u8) {
        self.secondary_water_grass_timer = value;
    }

    pub(super) fn increment_water_ripple_or_grass_state(&mut self) -> u8 {
        self.water_ripple_or_grass_state = self.water_ripple_or_grass_state.wrapping_add(1);
        self.water_ripple_or_grass_state
    }

    pub(super) fn clear_index_of_dashing_sfx(&mut self) {
        self.index_of_dashing_sfx = 0;
    }

    pub(super) fn decrement_index_of_dashing_sfx(&mut self) {
        self.index_of_dashing_sfx = self.index_of_dashing_sfx.wrapping_sub(1);
    }

    pub(super) fn set_throw_oam_state_index(&mut self, value: u8) {
        self.throw_oam_state_index = value;
    }

    pub(super) fn set_item_hold_pose(&mut self, value: u8) {
        self.item_hold_pose = value;
    }

    pub(super) fn clear_item_hold_pose(&mut self) {
        self.item_hold_pose = 0;
    }

    pub(super) fn force_hold_sword_up(&mut self) {
        self.force_hold_sword_up = 1;
    }

    pub(super) fn clear_force_hold_sword_up(&mut self) {
        self.force_hold_sword_up = 0;
    }

    pub(super) fn advance_frame_change_counter(&mut self, delay: u8) -> bool {
        self.frame_change_counter = self.frame_change_counter.wrapping_add(1);
        if self.frame_change_counter >= delay {
            self.frame_change_counter = 0;
            true
        } else {
            false
        }
    }

    pub(super) fn clear_frame_change_counter(&mut self) {
        self.frame_change_counter = 0;
    }

    pub(super) fn set_sprite_oam_state_timer(&mut self, value: u8) {
        self.sprite_oam_state_timer = value;
    }

    pub(super) fn decrement_sprite_oam_state_timer(&mut self) -> u8 {
        self.sprite_oam_state_timer = self.sprite_oam_state_timer.wrapping_sub(1);
        self.sprite_oam_state_timer
    }

    pub(super) fn mark_pit_landing_oam_state(&mut self) {
        self.sprite_oam_state_timer = 9;
    }

    pub(super) fn clear_player_pose_draw_counter(&mut self) {
        self.player_pose_draw_counter = 0;
    }

    pub(super) fn increment_player_pose_draw_counter(&mut self) {
        self.player_pose_draw_counter = self.player_pose_draw_counter.wrapping_add(1);
    }

    pub(super) fn clear_player_special_draw_flag(&mut self) {
        self.player_special_draw_flag = 0;
    }

    pub(super) fn set_player_special_draw_flag(&mut self, value: u8) {
        self.player_special_draw_flag = value;
    }

    pub(super) fn set_primary_water_grass_timer(&mut self, value: u8) {
        self.primary_water_grass_timer = value;
    }

    pub(super) fn increment_sleep_in_bed_state(&mut self) {
        self.sleep_in_bed_state = self.sleep_in_bed_state.wrapping_add(1);
    }

    pub(super) fn set_link_dma_graphics_index_word(&mut self, value: u16) {
        self.link_dma_graphics_index = value;
    }

    pub(super) fn set_link_dma_left_sprite_bank_word(&mut self, value: u16) {
        self.link_dma_left_sprite_bank = value;
    }

    pub(super) fn set_link_dma_right_sprite_bank_word(&mut self, value: u16) {
        self.link_dma_right_sprite_bank = value;
    }

    pub(super) fn clear_link_dma_sprite_banks(&mut self) {
        self.link_dma_left_sprite_bank = 0;
        self.link_dma_right_sprite_bank = 0;
    }

    pub(super) fn set_palette_bits_of_oam_word(&mut self, value: u16) {
        self.palette_bits_of_oam = value;
    }

    pub(super) fn set_sword_dma_graphics_index(&mut self, value: u8) {
        self.sword_dma_graphics_index = value;
    }

    pub(super) fn set_shield_dma_graphics_index(&mut self, value: u8) {
        self.shield_dma_graphics_index = value;
    }

    pub(super) fn set_link_dma_staging_index(&mut self, value: u8) {
        self.link_dma_staging_index = value;
    }
}
