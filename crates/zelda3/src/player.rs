// Methods ported from zelda3/src/player.c and included inside ZeldaState.

use super::sprite::SpriteSpawnInfo;
use super::*;
use crate::types::Point16U;

const DOOR_ANIMATION_STEP_INDICATOR_PLAYER: usize = 0x0690;
const PUSH_BLOCK_DIRECTION_PLAYER: usize = 0x0474;
const SPRITE_C_PLAYER: usize = 0x0db0;
const DASH_FOLLOWER_SLOWDOWN_INDICATORS: [u8; 15] =
    [0xff, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const DASH_FOLLOWER_RELEASE_INDICATORS: [u8; 15] = [0xff, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

fn player_memory_location_to_give_item_to(item: u8) -> usize {
    const MEMORY_LOCATIONS: [usize; 76] = [
        0xf359, 0xf359, 0xf359, 0xf359, 0xf35a, 0xf35a, 0xf35a, 0xf345, 0xf346, 0xf34b, 0xf342,
        0xf340, 0xf341, 0xf344, 0xf35c, 0xf347, 0xf348, 0xf349, 0xf34a, 0xf34c, 0xf34c, 0xf350,
        0xf35c, 0xf36b, 0xf351, 0xf352, 0xf353, 0xf354, 0xf354, 0xf34e, 0xf356, 0xf357, 0xf37a,
        0xf34d, 0xf35b, 0xf35b, 0xf36f, 0xf364, 0xf36c, 0xf375, 0xf375, 0xf344, 0xf341, 0xf35c,
        0xf35c, 0xf35c, 0xf36d, 0xf36e, 0xf36e, 0xf375, 0xf366, 0xf368, 0xf360, 0xf360, 0xf360,
        0xf374, 0xf374, 0xf374, 0xf340, 0xf340, 0xf35c, 0xf35c, 0xf36c, 0xf36c, 0xf360, 0xf360,
        0xf372, 0xf376, 0xf376, 0xf373, 0xf360, 0xf360, 0xf35c, 0xf359, 0xf34c, 0xf355,
    ];
    MEMORY_LOCATIONS.get(item as usize).copied().unwrap_or(0)
}

fn replay_trace_u16_env(name: &str) -> Option<u16> {
    let value = std::env::var(name).ok()?;
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16).ok()
    } else {
        value.parse::<u16>().ok()
    }
}

impl ZeldaState {
    pub(super) fn replay_trace_player_state(&self, label: &str) {
        if std::env::var_os("ZELDA3_REPLAY_TRACE_STATE").is_none() {
            return;
        }
        let room = self.world_location_state().dungeon_room;
        let x = self.player_state_view().x();
        let y = self.player_state_view().y();
        if let Some(frame) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_FRAME") {
            if self.frame_state().frame_counter as u16 != frame {
                return;
            }
        }
        if let Some(frame_min) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_FRAME_MIN") {
            if (self.frame_state().frame_counter as u16) < frame_min {
                return;
            }
        }
        if let Some(frame_max) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_FRAME_MAX") {
            if self.frame_state().frame_counter as u16 > frame_max {
                return;
            }
        }
        if let Some(expected_room) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_ROOM") {
            if room != expected_room {
                return;
            }
        }
        if let Some(expected_ow) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_OW") {
            if u16::from(self.world_location_state().overworld_screen_index()) != expected_ow {
                return;
            }
        }
        if let Some(x_min) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_X_MIN") {
            if x < x_min {
                return;
            }
        }
        if let Some(x_max) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_X_MAX") {
            if x > x_max {
                return;
            }
        }
        if let Some(y_min) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_Y_MIN") {
            if y < y_min {
                return;
            }
        }
        if let Some(y_max) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_Y_MAX") {
            if y > y_max {
                return;
            }
        }
        eprintln!(
            "state-trace frame={} {label} main={} sub={} state=0x{:02x} aux=0x{:02x} incap=0x{:02x} recoil=0x{:02x} scratch_a=0x{:02x} z=0x{:04x} vz=0x{:02x} vzcopy=0x{:02x} x=0x{x:04x} y=0x{y:04x} subpix=0x{:02x}/0x{:02x} vel=0x{:02x}/0x{:02x} dir=0x{:02x} last=0x{:02x} dlast=0x{:02x} pit=0x{:02x} below=0x{:02x} r14=0x{:04x} normal=0x{:04x} drag=0x{:02x} lspeed=0x{:02x}/0x{:02x}",
            self.frame_state().frame_counter,
            self.frame_state().main_module,
            self.frame_state().submodule,
            self.player_state_view().handler_state(),
            self.player_state_view().auxiliary_state(),
            self.player_state_view().incapacitated_timer(),
            self.player_state_view().recoil_timer(),
            self.player_state_view().scratch_a(),
            self.player_state_view().z(),
            self.player_state_view().actual_z_velocity(),
            self.player_state_view().actual_z_velocity_copy(),
            self.player_state_view().x_subpixel(),
            self.player_state_view().y_subpixel(),
            self.player_state_view().actual_x_velocity(),
            self.player_state_view().actual_y_velocity(),
            self.player_state_view().direction(),
            self.player_state_view().last_direction(),
            self.player_state_view().last_direction_moved_towards(),
            self.tile_detect_position_view().pit_tile(),
            self.player_state_view().tile_below(),
            self.tile_detect_position_view().collision_bits(),
            self.tile_detect_position_view().normal_tiles(),
            self.player_state_view().defense_flags(),
            self.player_state_view().speed_setting(),
            self.player_state_view().speed_modifier(),
        );
    }

    pub(super) fn replay_trace_drag_tail(&self, label: &str) {
        if std::env::var_os("ZELDA3_REPLAY_TRACE_SUB_FRAME").is_none() {
            return;
        }
        eprintln!(
            "drag-tail frame={} {label} r14=0x{:04x} tilecoll=0x{:02x} misc=0x{:04x} drag=0x{:02x} timer=0x{:02x} bframes=0x{:02x} lastmove=0x{:02x} face=0x{:02x}",
            self.frame_state().frame_counter,
            self.tile_detect_position_view().collision_bits(),
            self.player_state_view().tile_coll_flag(),
            self.tile_detect_position_view().misc_tiles(),
            self.player_state_view().defense_flags(),
            self.player_state_view().push_fatigue_timer(),
            self.player_state_view().button_b_frames(),
            self.player_state_view().last_direction_moved_towards(),
            self.player_state_view().facing(),
        );
    }

    pub(super) fn bit_sum4(value: u8) -> u8 {
        (value & 1) + ((value >> 1) & 1) + ((value >> 2) & 1) + ((value >> 3) & 1)
    }

    pub(super) fn dungeon_handle_layer_change(&mut self) {
        self.player_state_view_mut().mark_lower_level_mirror();
        if self.dungeon_state_view().kind_of_in_room_staircase() == 0 {
            self.increment_dungeon_room_index_by(16);
        }
        if self.dungeon_state_view().kind_of_in_room_staircase() != 2 {
            self.player_state_view_mut().mark_lower_level();
        }
        self.player_state_view_mut().clear_about_to_jump_off_ledge();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn check_ability_to_swim(&mut self) {
        self.replay_trace_submodule("check_ability_to_swim-entry");
        let player = self.player_state_view();
        if !player.is_bunny_mirror() && player.has_flippers() {
            return;
        }
        if self.player_state_view().has_moon_pearl() {
            self.player_state_view_mut().clear_bunny_mirror();
        }
        self.player_state_view_mut().set_visibility_status(0x0c);
        let submodule = if self.world_location_state().is_indoors() {
            20
        } else {
            42
        };
        self.set_submodule(submodule);
        self.replay_trace_submodule("check_ability_to_swim-exit");
    }

    pub(super) fn link_initialize(&mut self) {
        self.player_state_view_mut().initialize_link_action_state();
        self.link_reset_swimming_state();
        self.player_state_view_mut()
            .finish_link_action_state_initialization();
        self.link_force_unequip_cape_quietly();
        self.link_reset_sword_and_item_usage();

        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES) {
            self.player_state_view_mut()
                .clear_misc_bugfix_movement_state();
            self.world_state_view_mut().set_bg1_y_offset(0);
            self.world_state_view_mut().set_bg1_x_offset(0);

            let player = self.player_state_view();
            if !player.has_moon_pearl() && player.is_darkworld_save() {
                self.player_state_view_mut().become_bunny_handler();
                self.load_gear_palettes_bunny();
            }
        }
    }

    pub(super) fn link_reset_properties_a(&mut self) {
        self.player_state_view_mut().reset_properties_a_fields();
        self.link_reset_swimming_state();
        self.link_reset_properties_b();
    }

    pub(super) fn link_reset_properties_b(&mut self) {
        self.player_state_view_mut().reset_properties_b_fields();
        self.link_reset_properties_c();
    }

    pub(super) fn link_reset_properties_c(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES) {
            self.player_state_view_mut().clear_custom_spell_animation();
        }

        self.player_state_view_mut().reset_properties_c_fields();
        self.link_reset_sword_and_item_usage();
    }

    pub(super) fn link_tuck_into_bed(&mut self) {
        self.player_state_view_mut().set_y(0x215a);
        self.player_state_view_mut().set_x(0x0940);
        self.player_state_view_mut().setup_bed_pose();
        self.ancilla_add_blanket(0x20);
    }

    pub(super) fn link_reset_swimming_state(&mut self) {
        self.player_state_view_mut().reset_swimming_state_fields();
        self.reset_all_acceleration();
    }

    pub(super) fn link_reset_state_after_damaging_pit(&mut self) {
        self.link_reset_swimming_state();
        self.player_state_view_mut().reset_after_damaging_pit();
    }

    pub(super) fn link_state_bunny_recache(&mut self) {
        self.player_state_view_mut().recache_bunny_state();
        self.link_reset_swimming_state();
        self.player_state_view_mut().set_handler_state(6);
        if self.player_state_view().has_moon_pearl() {
            self.player_state_view_mut().clear_handler_state();
            self.load_actual_gear_palettes();
        }
    }

    pub(super) fn link_set_to_deep_water(&mut self) {
        self.player_state_view_mut().enter_deep_water();
        self.link_reset_swimming_state();
    }

    pub(super) fn link_splash_upon_landing(&mut self) {
        if self.player_state_view().is_bunny_mirror() {
            if self.player_state_view().is_in_deep_water() {
                self.ancilla_add_splash(21, 0);
                self.link_state_bunny_recache();
                return;
            }
            self.player_state_view_mut().land_after_splash();
        } else if self.player_state_view().is_in_deep_water() {
            if self.player_state_view().handler_state() != 2 {
                self.ancilla_add_splash(21, 0);
            }
            self.link_force_unequip_cape_quietly();
            self.player_state_view_mut().land_after_splash();
        } else {
            self.player_state_view_mut().land_after_splash();
        }
    }

    pub(super) fn link_handle_swim_accels(&mut self) {
        const SWIM_ACCELERATION_TARGETS: [u16; 9] = [128, 160, 192, 224, 256, 288, 320, 352, 384];

        let mut mask = 0x0c;
        for offset in [0, 2] {
            if self.player_state_view().joypad1h_last() & mask != 0 {
                let acceleration = self.swim_acceleration_view().acceleration(offset);
                let max_speed = self.swim_acceleration_view().max_speed(offset);
                if acceleration != 0 && max_speed >= 384 {
                    let target = SWIM_ACCELERATION_TARGETS
                        .iter()
                        .copied()
                        .find(|value| *value >= acceleration)
                        .unwrap_or(384);
                    self.swim_acceleration_view_mut()
                        .set_max_speed(offset, target);
                } else if max_speed != 0 {
                    let target = max_speed.wrapping_add(160).min(384);
                    self.swim_acceleration_view_mut()
                        .set_max_speed(offset, target);
                } else {
                    self.swim_acceleration_view_mut()
                        .set_acceleration(offset, 1);
                    self.swim_acceleration_view_mut().set_max_speed(offset, 240);
                }
            }
            mask >>= 2;
        }
    }

    pub(super) fn link_flag_max_accels(&mut self) {
        if self.player_state_view().flag_moving() == 0 {
            return;
        }

        for offset in [2, 0] {
            let acceleration = self.swim_acceleration_view().acceleration(offset);
            if acceleration != 0 {
                self.swim_acceleration_view_mut()
                    .set_max_speed(offset, acceleration);
                self.swim_acceleration_view_mut().set_mode(offset, 1);
            }
        }
    }

    pub(super) fn link_set_ice_max_accel(&mut self) {
        if self.player_state_view().flag_moving() == 0 {
            return;
        }
        self.swim_acceleration_view_mut().set_max_speed(0, 0x0180);
        self.swim_acceleration_view_mut().set_max_speed(2, 0x0180);
    }

    pub(super) fn link_set_momentum(&mut self) {
        const SWIM_STROKE_TIMERS_BY_MOVING_FLAG: [u8; 2] = [32, 8];

        let joy = self.player_state_view().joypad1h_last() & 0x0f;
        let mut mask = 0x0c;
        let mut bit = 0x08;
        for offset in [0, 2] {
            if joy & mask != 0 {
                let flag_moving = self.player_state_view().flag_moving();
                let stroke_timer = if flag_moving != 0 {
                    SWIM_STROKE_TIMERS_BY_MOVING_FLAG[(flag_moving - 1) as usize]
                } else {
                    32
                };
                self.player_state_view_mut()
                    .set_swim_stroke_frame_counter(offset, stroke_timer as u16);

                if (self.player_state_view().swim_direction_flags()
                    | self.player_state_view().direction())
                    & mask
                    == mask
                {
                    self.swim_acceleration_view_mut().set_mode(offset, 2);
                } else {
                    let direction = if joy & bit != 0 { 0 } else { 1 };
                    self.swim_acceleration_view_mut()
                        .set_acceleration_direction(offset, direction);
                    self.swim_acceleration_view_mut().set_mode(offset, 0);
                }

                if self.swim_acceleration_view().max_speed(offset) == 0 {
                    self.swim_acceleration_view_mut().set_max_speed(offset, 240);
                }
            }
            mask >>= 2;
            bit >>= 2;
        }
    }

    pub(super) fn player_handler_04_swimming(&mut self) {
        const ACTIVE_SWIM_ANIMATION_DELAYS: [u8; 4] = [2, 0, 1, 0];

        if self.player_state_view().auxiliary_state() != 0 {
            self.player_state_view_mut()
                .interrupt_swimming_for_auxiliary_state();
            self.reset_all_acceleration();
            self.link_state_recoil();
            return;
        }

        self.player_state_view_mut().clear_swimming_action_state();
        if !self.player_state_view().has_flippers() {
            return;
        }

        let has_swim_velocity = self.swim_acceleration_view().acceleration(0)
            | self.swim_acceleration_view().acceleration(2)
            != 0;
        if !has_swim_velocity {
            let swim = self.swim_acceleration_view();
            if swim.mode_low(0) != 2 && swim.mode_low(1) != 2 {
                self.reset_all_acceleration();
            }
            self.player_state_view_mut().advance_idle_swim_animation();
        } else {
            self.player_state_view_mut()
                .advance_active_swim_animation(&ACTIVE_SWIM_ANIMATION_DELAYS);
        }

        if self.player_state_view().hard_swim_stroke() == 0 {
            let hard_stroke = ((self.player_state_view().filtered_joypad_l() & 0x80)
                | self.player_state_view().filtered_joypad_h())
                & 0xc0;
            if !has_swim_velocity || hard_stroke == 0 {
                self.link_handle_swim_movements();
                return;
            }
            self.player_state_view_mut()
                .start_hard_swim_stroke(hard_stroke);
            self.ancilla_sfx2_near(37);
            self.link_handle_swim_accels();
        }

        self.player_state_view_mut().tick_hard_swim_stroke();
        self.link_handle_swim_movements();
    }

    fn advance_link_animation_step(&mut self, delay: u8, wrap_at: u8, wrap_to: u8) {
        if self
            .player_state_view_mut()
            .advance_frame_change_counter(delay)
        {
            self.player_state_view_mut()
                .advance_animation_step(wrap_at, wrap_to);
        }
    }

    fn advance_link_animation_step_at_least(&mut self, delay: u8, wrap_at: u8, wrap_to: u8) {
        if self
            .player_state_view_mut()
            .advance_frame_change_counter(delay)
        {
            self.player_state_view_mut()
                .advance_animation_step_at_least(wrap_at, wrap_to);
        }
    }

    pub(super) fn link_handle_swim_movements(&mut self) {
        let mut direction = (self.player_state_view().force_move_any_direction() as u8) & 0x0f;
        if direction == 0 {
            direction = self.player_state_view().joypad1h_last() & 0x0f;
        }

        if direction == 0 {
            self.player_state_view_mut().clear_swim_movement_velocity();
            self.link_flag_max_accels();
            if self.player_state_view().flag_moving() != 0 {
                if self.player_state_view().is_running() {
                    direction = self.player_state_view().swim_direction_flags();
                } else {
                    if self.swim_acceleration_view().acceleration(0)
                        | self.swim_acceleration_view().acceleration(2)
                        == 0
                    {
                        self.player_state_view_mut().clear_defense_flags();
                        self.link_reset_swimming_state();
                    }
                    self.finish_swim_movement_tail();
                    return;
                }
            } else {
                self.player_state_view_mut()
                    .reset_idle_swim_animation_if_out_of_water();
                self.finish_swim_movement_tail();
                return;
            }
        }

        if direction != self.player_state_view().swim_direction_flags() {
            self.player_state_view_mut()
                .set_swim_direction_flags(direction);
            self.player_state_view_mut()
                .reset_swim_subpixel_and_defense_state();
        }
        self.link_set_ice_max_accel();
        self.link_set_momentum();
        self.link_set_the_max_accel();
        self.finish_swim_movement_tail();
    }

    fn finish_swim_movement_tail(&mut self) {
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.player_state_view_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_set_the_max_accel(&mut self) {
        if self.player_state_view().flag_moving() != 0
            || self.player_state_view().hard_swim_stroke() != 0
        {
            return;
        }

        let mut mask = 0x0c;
        for offset in [0, 2] {
            let mode = self.swim_acceleration_view().mode(offset);
            if self.player_state_view().joypad1h_last() & mask != 0 && mode != 2 {
                let speed_active = self.swim_acceleration_view().speed_active_flag(offset);
                let acceleration = self.swim_acceleration_view().acceleration(offset);
                let max_speed = self.swim_acceleration_view().max_speed(offset);
                if speed_active != 0 || (acceleration >= 240 && acceleration >= max_speed) {
                    self.swim_acceleration_view_mut().set_mode(offset, 0);
                    if acceleration >= 240 {
                        self.swim_acceleration_view_mut()
                            .set_speed_active_flag(offset, 1);
                        self.swim_acceleration_view_mut().set_mode(offset, 1);
                    } else {
                        self.swim_acceleration_view_mut().set_max_speed(offset, 240);
                        self.swim_acceleration_view_mut()
                            .set_speed_active_flag(offset, 0);
                    }
                }
            } else {
                self.swim_acceleration_view_mut().set_max_speed(offset, 240);
                self.swim_acceleration_view_mut()
                    .set_speed_active_flag(offset, 0);
            }
            mask >>= 2;
        }
    }

    pub(super) fn link_handle_toss(&mut self) -> bool {
        if self.player_state_view().y_button_action_flags() & 0x80 == 0
            || self.player_state_view().filtered_joypad_l() & 0x80 == 0
            || self.player_state_view().is_lift_throw_primed()
        {
            return false;
        }

        self.player_state_view_mut()
            .clear_lift_throw_scratch_state();
        self.player_state_view_mut().set_y_button_action_step(0);
        self.player_state_view_mut().set_y_button_action_flags(0);
        self.player_state_view_mut().clear_direction_lock_bits(1);
        true
    }

    pub(super) fn link_cancel_dash(&mut self) {
        if !self.player_state_view().is_running() {
            return;
        }
        for i in (0..=4).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x1e {
                self.ancilla_slot_view_mut(i).set_ancilla_type(0);
            }
        }
        self.player_state_view_mut().cancel_dash_state();
        self.swim_acceleration_view_mut().clear_mode_low_axis();
    }

    pub(super) fn repel_dash(&mut self) {
        if self.player_state_view().is_running() && self.player_state_view().dash_counter() != 64 {
            self.link_reset_swimming_state();
            self.ancilla_add_dash_tremor(29, 1);
            self.prepare_apply_rumble_to_sprites();
            if self.system_signals_view().sound_effect_2() & 0x3f != 27
                && self.system_signals_view().sound_effect_2() & 0x3f != 50
            {
                self.ancilla_sfx3_near(3);
            }
            self.link_apply_tile_rebound();
        }
    }

    pub(super) fn sprite_repel_dash(&mut self) {
        self.player_state_view_mut()
            .set_last_direction_moved_towards_from_facing();
        self.repel_dash();
    }

    pub(super) fn link_apply_tile_rebound(&mut self) {
        const DASH_REBOUND_Y_VELOCITY_BY_DIRECTION: [u8; 4] = [24, (-24i8) as u8, 0, 0];
        const DASH_REBOUND_X_VELOCITY_BY_DIRECTION: [u8; 4] = [0, 0, 24, (-24i8) as u8];
        const DASH_REBOUND_SWIM_Y_DIRECTIONS: [u8; 4] = [1, 0, 0, 0];
        const DASH_REBOUND_SWIM_X_DIRECTIONS: [u8; 4] = [0, 0, 1, 0];
        const DASH_REBOUND_SWIM_Y_ACCELERATIONS: [u16; 8] = [384, 384, 0, 0, 256, 256, 0, 0];
        const DASH_REBOUND_SWIM_X_ACCELERATIONS: [u16; 8] = [0, 0, 384, 384, 0, 0, 256, 256];
        const DIRECTION_BITS_BY_FACING: [u8; 4] = [8, 4, 2, 1];

        let dir = self.player_state_view().last_direction_moved_towards() as usize;
        self.player_state_view_mut().set_actual_velocity_xy(
            DASH_REBOUND_X_VELOCITY_BY_DIRECTION[dir],
            DASH_REBOUND_Y_VELOCITY_BY_DIRECTION[dir],
        );
        self.player_state_view_mut().set_incapacitated_timer(24);
        self.player_state_view_mut()
            .set_actual_z_velocity_and_copy(36);
        if self.player_state_view().flag_moving() != 0 {
            self.player_state_view_mut()
                .set_direction_and_swim_flags(DIRECTION_BITS_BY_FACING[dir]);
            self.swim_acceleration_view_mut()
                .set_acceleration_direction(0, DASH_REBOUND_SWIM_Y_DIRECTIONS[dir] as u16);
            self.swim_acceleration_view_mut()
                .set_acceleration_direction(2, DASH_REBOUND_SWIM_X_DIRECTIONS[dir] as u16);
            let i = (self.player_state_view().flag_moving() - 1) as usize * 4 + dir;
            self.swim_acceleration_view_mut()
                .set_acceleration(0, DASH_REBOUND_SWIM_Y_ACCELERATIONS[i]);
            self.swim_acceleration_view_mut()
                .set_acceleration(2, DASH_REBOUND_SWIM_X_ACCELERATIONS[i]);
        }
        self.player_state_view_mut().set_auxiliary_state(1);
        self.player_state_view_mut().set_dash_noise_request();
        self.tile_detect_position_view_mut()
            .clear_interaction_scratch_x_low();
        self.player_state_view_mut().clear_electrocute_on_touch();
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().clear_direction_lock();
        self.player_state_view_mut()
            .clear_moving_against_diag_tile();
        if self.player_state_view().last_direction_moved_towards() & 2 != 0 {
            self.player_state_view_mut().set_y_velocity(0);
        } else {
            self.player_state_view_mut().set_x_velocity(0);
        }
    }

    pub(super) fn link_handle_moving_animation_full_long_entry(&mut self) {
        if self.player_state_view().handler_state() == 4 {
            self.link_handle_moving_animation_swimming();
            return;
        }

        const FACING_DIRECTION_BITS: [u8; 4] = [8, 4, 2, 1];
        let mut r0 = self.player_state_view().last_direction();
        if r0 == 0 {
            return;
        }
        if self.player_state_view().flag_moving() != 0 {
            r0 = self.player_state_view().swim_direction_flags();
        }
        if self.player_state_view().direction_lock() == 0 {
            let mut y;
            if self.player_state_view().num_orthogonal_directions() == 0 {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            } else if self.player_state_view().doorway_state() != 0 {
                y = self.player_state_view().doorway_state().wrapping_mul(2) & !3;
            } else if r0 & FACING_DIRECTION_BITS[self.player_state_view().facing_index()] != 0 {
                self.link_handle_moving_animation_start_with_dash();
                return;
            } else {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            }
            if y != 4 {
                y = y.wrapping_add(if r0 & 4 != 0 { 2 } else { 0 });
            } else {
                y = y.wrapping_add(if r0 & 1 != 0 { 2 } else { 0 });
            }
            self.player_state_view_mut().set_facing(y);
        }
        self.link_handle_moving_animation_start_with_dash();
    }

    pub(super) fn link_handle_moving_animation_start_with_dash(&mut self) {
        if self.player_state_view().is_running() {
            self.link_handle_moving_animation_dash();
            return;
        }
        let mut x = self.player_state_view().facing() >> 1;
        if self.player_state_view().speed_setting() == 6 {
            x = x.wrapping_add(4);
        } else if self.player_state_view().flag_moving() != 0 {
            if self.player_state_view().joypad1h_last() & 0x0f == 0 {
                self.player_state_view_mut().clear_animation_step();
                return;
            }
            x = x.wrapping_add(4);
        }

        const POSE_ANIMATION_DELAYS: [u8; 16] = [4, 4, 4, 4, 1, 1, 1, 1, 2, 2, 2, 2, 8, 8, 8, 8];
        const WALK_ANIMATION_DELAYS: [u8; 24] = [
            1, 2, 3, 2, 2, 2, 3, 2, 1, 1, 2, 1, 1, 1, 2, 1, 2, 2, 3, 2, 2, 2, 3, 2,
        ];
        if self.player_state_view().handler_state() == 23
            || (self.enhanced_features_view().has(4096)
                && self.player_state_view().handler_state() == 28)
        {
            if self.player_state_view().animation_step() < 4
                && self.player_state_view().on_somaria_platform() != 2
            {
                self.advance_link_animation_step(POSE_ANIMATION_DELAYS[x as usize], 4, 0);
            } else {
                self.player_state_view_mut().clear_animation_step();
            }
            return;
        }

        if self.frame_state().submodule == 18 || self.frame_state().submodule == 19 {
            x = 12;
        } else if self.frame_state().submodule != 14
            && !self.player_state_view().is_lifting_or_carrying()
        {
            if self.player_state_view().defense_flags() & 0x8d != 0 {
                x = 12;
            } else if self.player_state_view().water_ripple_or_grass_state() == 0
                && self.player_state_view().button_b_frames() == 0
            {
                let mut idx = self.player_state_view().animation_step();
                if self.player_state_view().speed_setting() == 6 {
                    idx = idx.wrapping_add(8);
                }
                if self.player_state_view().flag_moving() != 0 {
                    idx = idx.wrapping_add(8);
                }
                if self.player_state_view().on_somaria_platform() != 2 {
                    self.advance_link_animation_step(WALK_ANIMATION_DELAYS[idx as usize], 9, 1);
                }
                return;
            }
        }

        if self.player_state_view().animation_step() < 6
            && self.player_state_view().on_somaria_platform() != 2
        {
            self.advance_link_animation_step(POSE_ANIMATION_DELAYS[x as usize], 6, 0);
        } else {
            self.player_state_view_mut().clear_animation_step();
        }
    }

    pub(super) fn link_handle_moving_animation_swimming(&mut self) {
        const FACING_DIRECTION_BITS: [u8; 4] = [8, 4, 2, 1];
        let r0 = self.player_state_view().swim_direction_flags();
        if r0 == 0 || self.player_state_view().direction_lock() != 0 {
            return;
        }
        let mut y;
        if self.player_state_view().num_orthogonal_directions() != 0 {
            if self.player_state_view().doorway_state() != 0 {
                y = self.player_state_view().doorway_state().wrapping_mul(2) & !3;
            } else if r0 & FACING_DIRECTION_BITS[self.player_state_view().facing_index()] != 0 {
                return;
            } else {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            }
        } else {
            y = if r0 & 0x0c != 0 { 0 } else { 4 };
        }
        if y != 4 {
            y = y.wrapping_add(if r0 & 4 != 0 { 2 } else { 0 });
        } else {
            y = y.wrapping_add(if r0 & 1 != 0 { 2 } else { 0 });
        }
        self.player_state_view_mut().set_facing(y);
    }

    pub(super) fn link_handle_moving_animation_dash(&mut self) {
        const DASH_CHARGE_ANIM_THRESHOLDS: [u8; 7] = [48, 36, 24, 16, 12, 8, 4];
        const DASH_CHARGE_ANIM_DELAYS: [u8; 56] = [
            3, 3, 5, 3, 3, 3, 5, 3, 2, 2, 4, 2, 2, 2, 4, 2, 2, 2, 3, 2, 2, 2, 3, 2, 1, 1, 2, 1, 1,
            1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const DASH_CHARGE_MIN_ANIM_STEPS: [u8; 7] = [1, 2, 2, 2, 2, 2, 2];
        let mut t = 6usize;
        while self.player_state_view().dash_countdown() >= DASH_CHARGE_ANIM_THRESHOLDS[t] && t != 0
        {
            t -= 1;
        }
        if self.player_state_view().button_b_frames() < 9
            && self.player_state_view().water_ripple_or_grass_state() == 0
        {
            self.advance_link_animation_step(DASH_CHARGE_ANIM_DELAYS[t * 8], 9, 1);
        } else {
            self.advance_link_animation_step_at_least(DASH_CHARGE_MIN_ANIM_STEPS[t], 6, 0);
        }
    }

    pub(super) fn link_apply_moving_floor_velocity(&mut self) {
        self.player_state_view_mut()
            .clear_orthogonal_direction_count();
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(self.dungeon_state_view().floor_y_velocity());
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(self.dungeon_state_view().floor_x_velocity());
        self.player_state_view_mut().set_y(y);
        self.player_state_view_mut().set_x(x);
    }

    pub(super) fn link_apply_conveyor(&mut self) {
        const MOVE_POS_DIR_FLAG: [u8; 4] = [8, 4, 2, 1];
        const MOVING_BELT_Y: [i8; 4] = [-8, 8, 0, 0];
        const MOVING_BELT_X: [i8; 4] = [0, 0, -8, 8];
        if self.player_state_view().conveyor_belt_state() == 0 {
            return;
        }
        if !self.player_state_view().is_grounded_or_z_sentinel() {
            return;
        }
        if self.player_state_view().grabbing_wall_has(1)
            || self.player_state_view().handler_state() == 19
            || self.player_state_view().has_auxiliary_state()
        {
            return;
        }
        let j = self
            .player_state_view()
            .conveyor_belt_state()
            .wrapping_sub(1) as usize;
        if j >= MOVE_POS_DIR_FLAG.len() {
            return;
        }
        if self.player_state_view().is_running()
            && self.player_state_view().dash_counter() == 32
            && self.player_state_view().direction() & MOVE_POS_DIR_FLAG[j] != 0
        {
            return;
        }
        self.player_state_view_mut()
            .clear_orthogonal_direction_count();
        self.player_state_view_mut()
            .add_direction_flags(MOVE_POS_DIR_FLAG[j]);
        let y = ((self.player_state_view().y() as u32) << 8
            | self.bg1_move_calc_view().y_subpixel() as u32)
            .wrapping_add(((MOVING_BELT_Y[j] as i32) << 4) as u32);
        self.bg1_move_calc_view_mut().set_y_subpixel(y as u8);
        self.player_state_view_mut().set_y((y >> 8) as u16);
        let x = ((self.player_state_view().x() as u32) << 8
            | self.bg1_move_calc_view().x_subpixel() as u32)
            .wrapping_add(((MOVING_BELT_X[j] as i32) << 4) as u32);
        self.bg1_move_calc_view_mut().set_x_subpixel(x as u8);
        self.player_state_view_mut().set_x((x >> 8) as u16);
    }

    pub(super) fn flag67_with_directions(&mut self) {
        self.player_state_view_mut()
            .derive_direction_from_actual_velocity();
    }

    pub(super) fn link_add_in_velocity_y_falling(&mut self) {
        let adjust = i16::from(self.tile_detect_position_view().y_low() & 7)
            - if self.player_state_view().y_velocity_signed().is_negative() {
                8
            } else {
                0
            };
        let y = self.player_state_view().y().wrapping_sub(adjust as u16);
        self.player_state_view_mut().set_y(y);
    }

    pub(super) fn link_add_in_velocity_y(&mut self) {
        let y = self
            .player_state_view()
            .y()
            .wrapping_sub(self.player_state_view().y_velocity() as i8 as i16 as u16);
        self.player_state_view_mut().set_y(y);
    }

    pub(super) fn player_change_z(&mut self, z_delta: u8) {
        if (self.player_state_view().actual_z_velocity() as i8).is_negative() {
            if self.player_state_view().z_low() == 0 {
                return;
            }
            if (self.player_state_view().z_low() as i8).is_negative() {
                self.player_state_view_mut().set_z(0xffff);
                self.player_state_view_mut().set_actual_z_velocity(0xff);
                return;
            }
        }
        self.player_state_view_mut()
            .decrement_actual_z_velocity(z_delta);
    }

    pub(super) fn link_move_position(&mut self) {
        let x = self.player_state_view().x();
        let y = self.player_state_view().y();
        self.player_state_view_mut()
            .store_safe_return_position(x, y);

        if self.player_state_view().handler_state() != 10
            && self.player_state_view().on_somaria_platform() == 2
        {
            self.link_handle_velocity_and_sand_drag(x, y);
            return;
        }

        let actual_x_velocity = self.player_state_view().actual_x_velocity();
        let actual_y_velocity = self.player_state_view().actual_y_velocity();
        self.player_state_view_mut()
            .move_x_by_velocity(actual_x_velocity);
        self.player_state_view_mut()
            .move_y_by_velocity(actual_y_velocity);
        if self.player_state_view().auxiliary_state() != 0 {
            let actual_z_velocity = self.player_state_view().actual_z_velocity();
            self.player_state_view_mut()
                .move_z_by_velocity(actual_z_velocity);
        }

        self.link_handle_moving_floor();
        self.link_apply_conveyor();
        self.link_handle_velocity_and_sand_drag(x, y);
    }

    pub(super) fn link_handle_velocity_and_sand_drag(&mut self, old_x: u16, old_y: u16) {
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(self.player_state_view().drag_player_y());
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(self.player_state_view().drag_player_x());
        self.player_state_view_mut().set_y(y);
        self.player_state_view_mut().set_x(x);
        self.player_state_view_mut()
            .set_movement_velocity_from_position_delta(x, y, old_x, old_y);
    }

    pub(super) fn link_handle_moving_floor(&mut self) {
        if self.dungeon_state_view().header_collision() == 0 {
            return;
        }
        let z = self.player_state_view().z_low();
        if z != 0 && z != 0xff {
            return;
        }
        if !self
            .has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH)
        {
            return;
        }
        if self.player_state_view().handler_state() == 19 {
            return;
        }

        let floor_y = self.dungeon_state_view().floor_y_velocity();
        let floor_x = self.dungeon_state_view().floor_x_velocity();
        self.player_state_view_mut()
            .mark_moving_floor_direction(floor_y, floor_x);

        self.link_apply_moving_floor_velocity();
    }

    pub(super) fn check_if_room_needs_double_layer_check(&mut self) -> bool {
        if self.dungeon_state_view().header_collision() == 0
            || self.dungeon_state_view().header_collision() == 4
        {
            return false;
        }

        if self.dungeon_state_view().header_collision() >= 2 {
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(self.world_state_view().bg1_y())
                .wrapping_sub(self.world_state_view().bg2_y());
            self.player_state_view_mut().set_y(y);

            let x = self
                .player_state_view()
                .x()
                .wrapping_add(self.world_state_view().bg1_x())
                .wrapping_sub(self.world_state_view().bg2_x());
            self.player_state_view_mut().set_x(x);
            self.player_state_view_mut()
                .cache_moving_floor_position(x, y);
        }
        self.player_state_view_mut().mark_lower_level();
        true
    }

    pub(super) fn create_velocity_from_moving_background(&mut self) {
        if self.dungeon_state_view().header_collision() != 1 {
            let x = self
                .player_state_view()
                .x()
                .wrapping_sub(self.player_state_view().moving_floor_x());
            let y = self
                .player_state_view()
                .y()
                .wrapping_sub(self.player_state_view().moving_floor_y());
            let new_y = self
                .player_state_view()
                .y()
                .wrapping_add(self.world_state_view().bg2_y())
                .wrapping_sub(self.world_state_view().bg1_y());
            let new_x = self
                .player_state_view()
                .x()
                .wrapping_add(self.world_state_view().bg2_x())
                .wrapping_sub(self.world_state_view().bg1_x());
            self.player_state_view_mut().set_y(new_y);
            self.player_state_view_mut().set_x(new_x);
            if self.player_state_view().direction() != 0 {
                self.player_state_view_mut()
                    .add_movement_velocity_delta(x, y);
            }
        }
        self.player_state_view_mut().clear_lower_level();
    }

    pub(super) fn calculate_snap_scratch_y(&mut self) {
        let mut y_vel = self.player_state_view().y_velocity() as i8;
        if self.tile_detect_position_view().collision_bits() & 4 != 0 {
            if y_vel >= 0 {
                y_vel = y_vel.wrapping_neg();
            }
        } else if y_vel < 0 {
            y_vel = y_vel.wrapping_neg();
        }

        let x = self.player_state_view().x();
        let delta = if y_vel < 0 { -1i16 } else { 1i16 };
        self.player_state_view_mut()
            .set_x(x.wrapping_add(delta as u16));
    }

    pub(super) fn change_axis_of_perpendicular_door_movement_y(&mut self) {
        self.player_state_view_mut().set_direction_lock_bits(2);
        let r14 = self.tile_detect_position_view().collision_bits();
        let t = ((r14 | (r14 >> 4)) & 0x0f) as u8;
        if t & 7 == 0 {
            self.player_state_view_mut().clear_doorway_state();
            return;
        }

        let mut t = self.player_state_view().y_velocity();
        let dir = if self.player_state_view().x_low() >= 0x80 {
            if t & 0x80 == 0 {
                t = t.wrapping_neg();
            }
            4
        } else {
            if t & 0x80 != 0 {
                t = t.wrapping_neg();
            }
            6
        };
        let vel = if t & 0x80 != 0 { -1i16 } else { 1i16 };
        if !self.player_state_view().direction_lock_has(1) {
            self.player_state_view_mut().set_facing(dir);
        }
        let x = self.player_state_view().x();
        self.player_state_view_mut()
            .set_x(x.wrapping_add(vel as u16));
    }

    pub(super) fn snap_on_x(&mut self) {
        let x = self.player_state_view().x();
        let adjust = (x & 7).wrapping_sub(
            if (self.player_state_view().x_velocity() as i8).is_negative() {
                8
            } else {
                0
            },
        );
        self.player_state_view_mut().set_x(x.wrapping_sub(adjust));
    }

    pub(super) fn calculate_snap_scratch_x(&mut self) {
        let mut x_vel = self.player_state_view().x_velocity() as i8;
        if self.tile_detect_position_view().collision_bits() & 4 != 0 {
            if x_vel >= 0 {
                x_vel = x_vel.wrapping_neg();
            }
        } else if x_vel < 0 {
            x_vel = x_vel.wrapping_neg();
        }

        let y = self.player_state_view().y();
        let delta = if x_vel < 0 { -1i16 } else { 1i16 };
        self.player_state_view_mut()
            .set_y(y.wrapping_add(delta as u16));
    }

    pub(super) fn change_axis_of_perpendicular_door_movement_x(&mut self) -> i8 {
        self.player_state_view_mut().set_direction_lock_bits(2);
        let r14 = self.tile_detect_position_view().collision_bits();
        let r0 = ((r14 | (r14 >> 4)) & 0x0f) as u8;
        if r0 & 7 == 0 {
            self.player_state_view_mut().clear_doorway_state();
            return r0 as i8;
        }

        let mut x_vel = self.player_state_view().x_velocity_signed();
        let dir = if self.player_state_view().y_low() >= 0x80 {
            if x_vel >= 0 {
                x_vel = x_vel.wrapping_neg();
            }
            0
        } else {
            if x_vel < 0 {
                x_vel = x_vel.wrapping_neg();
            }
            2
        };
        if !self.player_state_view().direction_lock_has(1) {
            self.player_state_view_mut().set_facing(dir);
        }
        let y = self.player_state_view().y();
        self.player_state_view_mut()
            .set_y(y.wrapping_add(x_vel as i16 as u16));
        x_vel
    }

    pub(super) fn tile_behavior_handle_item_and_execute(&mut self, x: u16, y: u16) {
        let tile = self.handle_item_tile_action_overworld(x, y);
        self.tile_detect_execute_inner(tile, 0, 1, false);
    }

    pub(super) fn push_block_get_target_tile_flag(&self, x: u16, y: u16) -> u8 {
        let offset = ((y & !7) as usize) * 8
            + (x & 0x3f) as usize
            + if self.player_state_view().is_on_lower_level() {
                0x1000
            } else {
                0
            };
        self.dungeon_state_view().bg2_attr(offset)
    }

    pub(super) fn link_handle_change_in_z_velocity(&mut self) {
        self.player_change_z(if self.player_state_view().handler_state() == 19 {
            1
        } else {
            2
        });
    }

    pub(super) fn run_slope_collision_checks_vertical_first(&mut self) {
        if self.player_state_view().moving_against_diag_tile() & 0x20 == 0 {
            self.start_movement_collision_checks_y();
        }
        if self.player_state_view().moving_against_diag_tile() & 0x10 == 0 {
            self.start_movement_collision_checks_x();
        }
    }

    pub(super) fn run_slope_collision_checks_horizontal_first(&mut self) {
        if self.player_state_view().moving_against_diag_tile() & 0x10 == 0 {
            self.start_movement_collision_checks_x();
        }
        if self.player_state_view().moving_against_diag_tile() & 0x20 == 0 {
            self.start_movement_collision_checks_y();
        }
    }

    pub(super) fn link_hop_in_or_out_of_water_y(&mut self) {
        const RECOIL_VEL_Y: [u8; 3] = [24, 16, 16];
        const RECOIL_VEL_Z: [u8; 3] = [36, 24, 24];
        let ts = if self.world_location_state().is_outdoors() {
            2
        } else if self.player_state_view().about_to_jump_off_ledge() != 0 {
            0
        } else {
            self.display_state().sub_screen_layers
        };

        let mut vel = RECOIL_VEL_Y[ts as usize];
        if self.player_state_view().last_direction_moved_towards() == 0 {
            vel = 0u8.wrapping_sub(vel);
        }

        self.player_state_view_mut().set_actual_velocity_xy(0, vel);
        self.player_state_view_mut()
            .set_actual_z_velocity_and_copy(RECOIL_VEL_Z[ts as usize]);
        self.player_state_view_mut().set_z(0);
        self.player_state_view_mut().set_incapacitated_timer(16);
        self.player_state_view_mut().enter_water_hop_state();
    }

    pub(super) fn link_hop_in_or_out_of_water_x(&mut self) {
        const RECOIL_VEL_X: [u8; 3] = [28, 24, 16];
        const RECOIL_VEL_Z: [u8; 3] = [32, 24, 24];
        let ts = if self.world_location_state().is_outdoors() {
            2
        } else if self.player_state_view().about_to_jump_off_ledge() != 0 {
            0
        } else {
            self.display_state().sub_screen_layers
        };

        let mut vel = RECOIL_VEL_X[ts as usize];
        if self.player_state_view().last_direction_moved_towards() & 1 == 0 {
            vel = 0u8.wrapping_sub(vel);
        }
        self.player_state_view_mut().set_actual_velocity_xy(vel, 0);
        self.player_state_view_mut()
            .set_actual_z_velocity_and_copy(RECOIL_VEL_Z[ts as usize]);
        self.player_state_view_mut().set_incapacitated_timer(16);
        self.player_state_view_mut().enter_water_hop_state();
    }

    pub(super) fn run_ledge_hop_timer(&mut self) -> bool {
        let mut rv = false;
        if !self.player_state_view().is_in_auxiliary_state(1) {
            if !self.player_state_view().is_running() {
                if self
                    .player_state_view_mut()
                    .tick_jump_ledge_timer_or_reset()
                {
                    return true;
                }
            } else {
                rv = true;
            }
        }
        self.player_state_view_mut()
            .restore_position_from_previous();
        self.player_state_view_mut().clear_movement_subpixels();
        rv
    }

    pub(super) fn flag_moving_into_slopes_y(&mut self) {
        const AVOID_JUDDER: [i8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4,
            5, 6, 7,
        ];
        let diag_state = self.tile_detect_position_view().diag_state() as usize;
        let x = self.player_state_view().x().wrapping_sub(
            if self.tile_detect_position_view().slope_collision_bits() & 4 != 0 {
                1
            } else {
                0
            },
        );
        let o = diag_state * 4 + (x & 7) as usize;
        let mut y = (self.tile_detect_position_view().y_low() & 7) as i8;

        if self.tile_detect_position_view().diagonal_tile() & 5 != 0 {
            let mut ym = (self.tile_detect_position_view().y_low() & 7) as i8;
            if self.enhanced_features_view().has(4096) {
                if diag_state & 2 != 0 {
                    ym = ym.wrapping_neg();
                } else {
                    ym = AVOID_JUDDER[o].wrapping_sub(8 - ym);
                }
            } else {
                if diag_state & 2 == 0 {
                    ym = 8 - ym;
                } else {
                    ym += 8;
                }
                ym = AVOID_JUDDER[o].wrapping_sub(ym);
            }

            let y_velocity = self.player_state_view().y_velocity_signed();
            if y_velocity == 0 {
                return;
            }
            if y_velocity.is_negative() {
                ym = ym.wrapping_neg();
            }
            y = ym;
        } else {
            y = AVOID_JUDDER[o].wrapping_sub(y);
        }

        if self.player_state_view().y_velocity_signed().is_negative() {
            if y <= 0 {
                return;
            }
            let coord = self.player_state_view().y().wrapping_add(y as i16 as u16);
            self.player_state_view_mut().set_y(coord);
            self.player_state_view_mut().set_moving_against_diag_tile(8);
        } else {
            if y >= 0 {
                return;
            }
            let coord = self.player_state_view().y().wrapping_add(y as i16 as u16);
            self.player_state_view_mut().set_y(coord);
            self.player_state_view_mut().set_moving_against_diag_tile(4);
        }

        let diag_flags = if self.tile_detect_position_view().slope_collision_bits() & 4 != 0 {
            0x12
        } else {
            0x11
        };
        self.player_state_view_mut()
            .add_moving_against_diag_tile_flags(diag_flags);
    }

    pub(super) fn flag_moving_into_slopes_x(&mut self) {
        const AVOID_JUDDER: [i8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4,
            5, 6, 7,
        ];
        let diag_state = self.tile_detect_position_view().diag_state();
        let mut x = (self
            .player_state_view()
            .x()
            .wrapping_sub(if diag_state == 6 { 1 } else { 0 })
            & 7) as i8;
        let which_y_offset = if self.tile_detect_position_view().slope_collision_bits() & 4 != 0 {
            2
        } else {
            0
        };
        let o = diag_state as usize * 4
            + (self.tile_detect_position_view().y_low_at(which_y_offset) & 7) as usize;

        if self.tile_detect_position_view().diagonal_tile() & 5 != 0 {
            let mut xm = (self.player_state_view().x() & 7) as i8;
            if diag_state != 4 && diag_state != 6 {
                xm = xm.wrapping_neg();
            } else {
                xm = AVOID_JUDDER[o].wrapping_sub(8 - xm);
            }
            let x_velocity = self.player_state_view().x_velocity_signed();
            if x_velocity == 0 {
                return;
            }
            if x_velocity.is_negative() {
                xm = xm.wrapping_neg();
            }
            x = xm;
        } else {
            x = AVOID_JUDDER[o].wrapping_sub(x);
        }

        if self.player_state_view().x_velocity_signed().is_negative() {
            if x <= 0 {
                return;
            }
            let coord = self.player_state_view().x().wrapping_add(x as i16 as u16);
            self.player_state_view_mut().set_x(coord);
            self.player_state_view_mut().set_moving_against_diag_tile(2);
        } else {
            if x >= 0 {
                return;
            }
            let coord = self.player_state_view().x().wrapping_add(x as i16 as u16);
            self.player_state_view_mut().set_x(coord);
            self.player_state_view_mut().set_moving_against_diag_tile(1);
        }

        self.player_state_view_mut()
            .add_moving_against_diag_tile_flags(if diag_state & 2 != 0 { 0x28 } else { 0x24 });
    }

    pub(super) fn player_something_with_velocity_tired_or_swim(&mut self, xvel: u16, yvel: u16) {
        let old_x = self.player_state_view().x();
        let old_y = self.player_state_view().y();
        self.player_state_view_mut()
            .store_safe_return_position(old_x, old_y);

        self.player_state_view_mut().move_x_by_subpixel_delta(xvel);
        let u = (xvel >> 8) as u8;
        let actual_x_velocity = (if (u as i8).is_negative() {
            0u8.wrapping_sub(u)
        } else {
            u
        } << 4)
            | ((xvel as u8) >> 4);

        self.player_state_view_mut().move_y_by_subpixel_delta(yvel);
        let u = (yvel >> 8) as u8;
        let actual_y_velocity = (if (u as i8).is_negative() {
            0u8.wrapping_sub(u)
        } else {
            u
        } << 4)
            | ((yvel as u8) >> 4);
        self.player_state_view_mut()
            .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);

        if self.dungeon_state_view().header_collision() == 4 {
            self.link_apply_moving_floor_velocity();
        }
        self.player_state_view_mut().clear_page_movement_deltas();
        self.link_handle_velocity_and_sand_drag(old_x, old_y);
    }

    pub(super) fn link_check_for_edge_screen_transition(&mut self) -> bool {
        if self
            .player_state_view()
            .is_edge_transition_blocked_by_handler_state()
            || self.player_state_view().incapacitated_timer() == 0
        {
            return false;
        }
        self.player_state_view_mut().clear_actual_velocity_xy();
        self.player_state_view_mut().set_recoil_timer(3);
        self.player_state_view_mut()
            .restore_position_from_previous();
        true
    }

    pub(super) fn player_limit_directions_inner(&mut self) {
        self.player_state_view_mut().reset_direction_limits();

        const MASKS: [u8; 4] = [0x07, 0x0b, 0x0d, 0x0e];

        let direction = self.player_state_view().direction();
        if direction & 0x0c != 0 {
            self.player_state_view_mut()
                .increment_orthogonal_direction_count();
            let last_direction_moved_towards = if direction & 8 != 0 { 0 } else { 1 };
            self.player_state_view_mut()
                .set_last_direction_moved_towards(last_direction_moved_towards);
            self.tile_detect_movement_vertical_slopes(last_direction_moved_towards as u16);

            let r14 = self.tile_detect_position_view().collision_bits();
            if r14 & 0x30 != 0
                && self.tile_detect_position_view().door_direction_flags() as u8 & 2 == 0
                && ((((r14 & 0x30) >> 4) as u8) & self.player_state_view().direction()) == 0
                && self.player_state_view().direction() & 3 != 0
            {
                let mask = MASKS[if self.player_state_view().direction() & 2 != 0 {
                    2
                } else {
                    3
                }];
                self.player_state_view_mut().set_direction_mask_a(mask);
            } else {
                let mut set_thingy = false;
                if self.dungeon_state_view().header_collision() == 0
                    && self.player_state_view().has_auxiliary_state()
                    && self.tile_detect_position_view().slope_collision_bits() & 3 != 0
                {
                    set_thingy = true;
                }

                if r14 & 3 != 0 {
                    self.player_state_view_mut()
                        .clear_moving_against_diag_tile();
                    if self.player_state_view().flag_moving() != 0
                        && self.tile_detect_position_view().spike_cactus_tiles() & 3 == 0
                        && self.player_state_view().direction() & 3 != 0
                    {
                        self.swim_acceleration_view_mut()
                            .set_speed_active_flag(0, 0);
                        self.swim_acceleration_view_mut().set_mode(0, 0);
                        self.swim_acceleration_view_mut().set_acceleration(0, 0);
                        self.swim_acceleration_view_mut().set_max_speed(0, 0);
                    }
                    set_thingy = true;
                }

                if set_thingy {
                    self.player_state_view_mut().set_pit_correction_active();
                    let mask =
                        MASKS[self.player_state_view().last_direction_moved_towards() as usize];
                    self.player_state_view_mut().set_direction_mask_a(mask);
                }
            }
        }

        let direction = self.player_state_view().direction();
        if direction & 0x0c != 0 && direction & 3 != 0 {
            self.player_state_view_mut()
                .increment_orthogonal_direction_count();
            let last_direction_moved_towards = if direction & 2 != 0 { 2 } else { 3 };
            self.player_state_view_mut()
                .set_last_direction_moved_towards(last_direction_moved_towards);
            self.tile_detect_movement_horizontal_slopes(last_direction_moved_towards as u16);

            let r14 = self.tile_detect_position_view().collision_bits();
            if r14 & 0x30 != 0
                && self.tile_detect_position_view().door_direction_flags() as u8 & 2 != 0
                && ((((r14 & 0x30) >> 2) as u8) & self.player_state_view().direction()) == 0
                && self.player_state_view().direction() & 0x0c != 0
            {
                let mask = MASKS[if self.player_state_view().direction() & 8 != 0 {
                    0
                } else {
                    1
                }];
                self.player_state_view_mut().set_direction_mask_b(mask);
            } else {
                let mut set_thingy_b = false;
                if self.dungeon_state_view().header_collision() == 0
                    && self.player_state_view().has_auxiliary_state()
                    && self.tile_detect_position_view().slope_collision_bits() & 3 != 0
                {
                    set_thingy_b = true;
                }

                if r14 & 3 != 0 {
                    self.player_state_view_mut()
                        .clear_moving_against_diag_tile();
                    if self.player_state_view().flag_moving() != 0
                        && self.tile_detect_position_view().spike_cactus_tiles() & 3 == 0
                        && self.player_state_view().direction() & 0x0c != 0
                    {
                        self.swim_acceleration_view_mut()
                            .set_speed_active_flag(2, 0);
                        self.swim_acceleration_view_mut().set_mode(2, 0);
                        self.swim_acceleration_view_mut().set_acceleration(2, 0);
                        self.swim_acceleration_view_mut().set_max_speed(2, 0);
                    }
                    set_thingy_b = true;
                }

                if set_thingy_b {
                    self.player_state_view_mut().set_pit_correction_active();
                    let mask =
                        MASKS[self.player_state_view().last_direction_moved_towards() as usize];
                    self.player_state_view_mut().set_direction_mask_b(mask);
                }
            }

            self.player_state_view_mut().apply_direction_masks();
        }

        self.player_state_view_mut()
            .force_direction_from_diag_tile_if_needed();
        self.player_state_view_mut()
            .resolve_orthogonal_direction_count_from_facing();
    }

    pub(super) fn link_handle_velocity(&mut self) {
        let old_x = self.player_state_view().x();
        let old_y = self.player_state_view().y();

        if (self.frame_state().submodule == 2 && self.frame_state().main_module == 14)
            || self.player_state_view().is_prevented_from_moving()
        {
            self.store_link_safe_return_position(old_x, old_y);
            self.link_handle_velocity_and_sand_drag(old_x, old_y);
            return;
        }

        if self.player_state_view().handler_state() == 4 {
            self.handle_swim_stroke_and_subpixels();
            return;
        }

        let mut speed_index = if self.player_state_view().flag_moving() != 0 {
            if !self.player_state_view().is_running() {
                self.handle_swim_stroke_and_subpixels();
                return;
            }
            24
        } else {
            if self.player_state_view().is_running() {
                self.player_state_view_mut().clear_speed_modifier();
                assert!(self.player_state_view().dash_counter() >= 32);
            }
            if (self
                .tile_detect_position_view()
                .tile_collision_bits_primary()
                | self
                    .tile_detect_position_view()
                    .tile_collision_bits_secondary())
                == 0x0f
            {
                return;
            }
            if self.player_state_view().water_ripple_or_grass_state() != 0 {
                match self.player_state_view().speed_setting() {
                    16 => 22,
                    12 => 14,
                    _ => 12,
                }
            } else {
                self.player_state_view().speed_setting()
            }
        };

        self.player_state_view_mut()
            .clear_actual_velocity_and_page_movement_deltas();

        let direction = self.player_state_view().direction();
        if (direction & 0x0c) != 0 && (direction & 0x03) != 0 {
            speed_index = speed_index.wrapping_add(1);
        }

        if self.player_state_view().is_near_pit() {
            if self.player_state_view().near_pit_state_is(3) {
                self.player_state_view_mut()
                    .increase_near_pit_speed_modifier();
            }
        } else if self.player_state_view().speed_modifier() != 0 {
            speed_index = if self.frame_state().submodule == 8 || self.frame_state().submodule == 16
            {
                10
            } else {
                2
            };
            let speed_modifier = self.player_state_view().speed_modifier();
            if speed_modifier != 1 && speed_modifier < 16 {
                self.player_state_view_mut().advance_dash_deceleration();
                speed_index = 26;
            } else if speed_modifier != 1 {
                self.player_state_view_mut().clear_speed_modifier();
                self.player_state_view_mut().set_speed_setting(0);
            }
        }

        const SPEED_MOD: [u8; 27] = [
            24, 16, 10, 24, 16, 8, 8, 4, 12, 16, 9, 25, 20, 13, 16, 8, 64, 42, 16, 8, 4, 2, 48, 24,
            32, 21, 0,
        ];
        let vel = self
            .player_state_view()
            .speed_modifier()
            .wrapping_add(SPEED_MOD[speed_index as usize]);
        let direction = self.player_state_view().direction();
        self.player_state_view_mut()
            .set_actual_velocity_from_direction(direction, vel);
        self.player_state_view_mut().prime_airborne_z_velocity();
        self.link_move_position();
    }

    pub(super) fn handle_swim_stroke_and_subpixels(&mut self) {
        self.player_state_view_mut().clear_actual_velocity_xy();

        const SWIM_ACCELERATION_DELTAS: [i8; 12] =
            [8, -12, -8, -16, 4, -6, -12, -6, 10, -16, -12, -6];
        const SWIM_AXIS_DIRECTION_CLEAR_MASKS: [u8; 2] = [!0x0c, !0x03];
        const SWIM_DIRECTION_BITS_BY_AXIS: [u8; 4] = [8, 4, 2, 1];
        let mut stroke = [0u16; 2];

        for i in (0..=1).rev() {
            let offset = i * 2;
            let stroke_timer = self
                .player_state_view()
                .swim_stroke_frame_counter(offset)
                .wrapping_sub(1);
            self.player_state_view_mut()
                .set_swim_stroke_frame_counter(offset, stroke_timer);
            if (stroke_timer as i16) < 0 {
                self.player_state_view_mut()
                    .set_swim_stroke_frame_counter(offset, 0);
                self.swim_acceleration_view_mut().set_mode(offset, 1);
            }

            let mut table_index = self.swim_acceleration_view().mode(offset);
            if self.player_state_view().flag_moving() != 0 {
                table_index =
                    table_index.wrapping_add(u16::from(self.player_state_view().flag_moving()) * 4);
            }

            let delta = SWIM_ACCELERATION_DELTAS[table_index as usize] as i16 as u16;
            let mut sum = self
                .swim_acceleration_view()
                .acceleration(offset)
                .wrapping_add(delta);
            if (sum as i16) <= 0 {
                self.player_state_view_mut()
                    .mask_direction(SWIM_AXIS_DIRECTION_CLEAR_MASKS[i]);
                self.player_state_view_mut()
                    .set_last_direction_from_current_direction();
                if self.swim_acceleration_view().mode(offset) == 2 {
                    self.swim_acceleration_view_mut().set_mode(offset, 0);
                    self.swim_acceleration_view_mut().set_max_speed(offset, 240);
                    self.swim_acceleration_view_mut()
                        .set_acceleration(offset, 2);
                } else {
                    self.swim_acceleration_view_mut().set_mode(offset, 0);
                    self.swim_acceleration_view_mut().set_max_speed(offset, 0);
                    self.swim_acceleration_view_mut()
                        .set_acceleration(offset, 0);
                }
            } else {
                let dir_index =
                    self.swim_acceleration_view().acceleration_direction(offset) as usize + i * 2;
                self.player_state_view_mut()
                    .add_direction_flags(SWIM_DIRECTION_BITS_BY_AXIS[dir_index]);
                let max_sum = self.swim_acceleration_view().max_speed(offset);
                if sum >= max_sum {
                    sum = max_sum;
                }
                self.swim_acceleration_view_mut()
                    .set_acceleration(offset, sum);
            }

            stroke[i] = self.swim_acceleration_view().acceleration(offset);
            if self.player_state_view().has_swim_axis_drag() {
                stroke[i] = stroke[i].wrapping_sub(stroke[i] >> 2);
            }
            if self.swim_acceleration_view().acceleration_direction(offset) == 0 {
                stroke[i] = 0u16.wrapping_sub(stroke[i]);
            }
        }

        self.player_something_with_velocity_tired_or_swim(stroke[1], stroke[0]);
    }

    pub(super) fn link_receive_item(&mut self, item: u8, chest_position: u16) {
        if self.player_state_view().has_auxiliary_state() {
            self.player_state_view_mut().clear_auxiliary_state();
            self.player_state_view_mut().set_incapacitated_timer(0);
            self.player_state_view_mut().clear_blink_countdown();
            self.player_state_view_mut().clear_state_bits();
        }
        self.player_state_view_mut().set_receive_item_index(item);
        if item == 0x3e {
            self.ancilla_sfx3_near(0x2e);
        }
        self.player_state_view_mut().set_item_holding_timer(0x60);
        if self.player_state_view().item_receipt_method() == 0
            || self.player_state_view().item_receipt_method() == 3
        {
            self.player_state_view_mut().clear_state_bits();
            self.player_state_view_mut().set_button_mask_b_y(0);
            self.player_state_view_mut().set_y_button_action_flags(0);
            self.player_state_view_mut().set_button_b_frames(0);
            self.player_state_view_mut().set_speed_setting(0);
            self.player_state_view_mut().clear_direction_lock();
            self.player_state_view_mut().clear_item_in_hand();
            self.player_state_view_mut().clear_position_mode();
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_handler_state(21);
            self.player_state_view_mut().set_item_hold_pose(1);
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            if item == 0x20 {
                self.player_state_view_mut().set_item_hold_pose(2);
            }
        }
        self.ancilla_add_item_receipt(0x22, 4, chest_position);
        if item != 0x20 && item != 0x37 && item != 0x38 && item != 0x39 {
            self.hud_refresh_icon();
        }
        self.link_cancel_dash();
    }

    pub(super) fn handle_nudging(&mut self, arg_r0: i8) {
        const NUDGE_PROBE_Y0_OFFSETS: [u8; 8] = [8, 8, 23, 23, 8, 23, 8, 23];
        const NUDGE_PROBE_X0_OFFSETS: [u8; 8] = [0, 15, 0, 15, 0, 0, 15, 15];
        const NUDGE_PROBE_Y1_OFFSETS: [u8; 8] = [23, 23, 8, 8, 8, 23, 8, 23];
        const NUDGE_PROBE_X1_OFFSETS: [u8; 8] = [0, 15, 0, 15, 15, 15, 0, 0];

        let last_direction_moved_towards = self.player_state_view().last_direction_moved_towards();
        let p = if last_direction_moved_towards & 2 == 0 {
            if last_direction_moved_towards & 1 != 0 {
                4
            } else {
                0
            }
        } else if last_direction_moved_towards & 1 != 0 {
            12
        } else {
            8
        };
        let o = (((if self.tile_detect_position_view().collision_bits() & 4 != 0 {
            0
        } else {
            2
        }) + p)
            >> 1) as usize;

        self.tile_detect_position_view_mut().clear_pit_tile();
        self.tile_detect_reset_state();

        let mask = self.tile_detect_position_view().location_calc_mask();
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let y0 = link_y.wrapping_add(NUDGE_PROBE_Y0_OFFSETS[o] as u16) & mask;
        let x0 = (link_x.wrapping_add(NUDGE_PROBE_X0_OFFSETS[o] as u16) & mask) >> 3;
        let y1 = link_y.wrapping_add(NUDGE_PROBE_Y1_OFFSETS[o] as u16) & mask;
        let x1 = (link_x.wrapping_add(NUDGE_PROBE_X1_OFFSETS[o] as u16) & mask) >> 3;

        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);

        let blocked = (self.tile_detect_position_view().collision_bits()
            | self.tile_detect_position_view().horizontal_ledge() as u16)
            & 3
            != 0
            || (self.tile_detect_position_view().vertical_ledge()
                | self.tile_detect_position_view().diagonal_ledge_tiles())
                & 0x33
                != 0;
        if blocked {
            if self.player_state_view().last_direction_moved_towards() & 2 != 0 {
                let y = self.player_state_view().y();
                self.player_state_view_mut()
                    .set_y(y.wrapping_sub(arg_r0 as i16 as u16));
            } else {
                let x = self.player_state_view().x();
                self.player_state_view_mut()
                    .set_x(x.wrapping_sub(arg_r0 as i16 as u16));
            }
        }
    }

    pub(super) fn handle_pushing_bonking_snaps_y(&mut self) {
        let r14 = self.tile_detect_position_view().collision_bits();
        if r14 & 7 == 0 {
            if self.player_state_view().is_on_lower_level() {
                return;
            }
            self.player_state_view_mut().and_defense_flags(!9);
            self.handle_pushing_bonking_snaps_return();
            return;
        }

        let mut used_swim_axis_reprobe = false;
        if self.player_state_view().handler_state() == 4 {
            if self.dungeon_state_view().floor_y_velocity_low() == 0 {
                self.reset_all_acceleration();
            }
            if self.player_state_view().num_orthogonal_directions() != 0 {
                self.link_add_in_velocity_y_falling();
                used_swim_axis_reprobe = true;
            }
        }

        if r14 & 2 != 0 || (r14 & 5) == 5 {
            self.replay_trace_drag_tail("snaps-y-before-first-bonk");
            let bak = r14;
            self.link_bonk_and_smash();
            self.repel_dash();
            self.tile_detect_position_view_mut().set_collision_bits(bak);
            self.replay_trace_drag_tail("snaps-y-after-first-bonk");
        }

        self.player_state_view_mut().set_pit_correction_active();

        if !used_swim_axis_reprobe {
            if r14 & 2 == 2 {
                self.replay_trace_drag_tail("snaps-y-before-add-vel");
                self.link_add_in_velocity_y_falling();
                self.replay_trace_drag_tail("snaps-y-after-add-vel");
            } else {
                if self.player_state_view().num_orthogonal_directions() == 1 {
                    self.handle_pushing_bonking_snaps_return();
                    return;
                }
                self.link_add_in_velocity_y_falling();
                if self.player_state_view().num_orthogonal_directions() == 2 {
                    self.handle_pushing_bonking_snaps_return();
                    return;
                }
            }
        }

        if (r14 & 5) == 5 {
            self.replay_trace_drag_tail("snaps-y-before-second-bonk");
            self.link_bonk_and_smash();
            self.repel_dash();
            self.replay_trace_drag_tail("snaps-y-after-second-bonk");
        } else if r14 & 2 == 0 {
            let y_vel = self.player_state_view().y_velocity();
            let tt = if r14 & 4 != 0 {
                if (y_vel as i8).is_negative() {
                    y_vel
                } else {
                    0u8.wrapping_sub(y_vel)
                }
            } else if (y_vel as i8).is_negative() {
                0u8.wrapping_sub(y_vel)
            } else {
                y_vel
            };
            let delta = if (tt as i8).is_negative() {
                -1i16
            } else {
                1i16
            };
            if self.player_state_view().x() & 7 != 0 {
                let x = self.player_state_view().x();
                self.player_state_view_mut()
                    .set_x(x.wrapping_add(delta as u16));
                self.handle_nudging(delta as i8);
                return;
            }
            self.link_bonk_and_smash();
            self.repel_dash();
        }

        if self.handle_pushing_bonking_dragstate_tail() {
            return;
        }
        self.handle_pushing_bonking_snaps_return();
    }

    pub(super) fn handle_pushing_bonking_snaps_x(&mut self) {
        let r14 = self.tile_detect_position_view().collision_bits();
        if r14 & 7 == 0 {
            if self.player_state_view().is_on_lower_level() {
                return;
            }
            self.player_state_view_mut().and_defense_flags(!9);
            self.handle_pushing_bonking_snaps_return();
            return;
        }

        if self.player_state_view().handler_state() == 4
            && self.dungeon_state_view().floor_x_velocity_low() == 0
        {
            self.reset_all_acceleration();
        }

        if r14 & 2 != 0 {
            let bak = r14;
            self.link_bonk_and_smash();
            self.repel_dash();
            self.tile_detect_position_view_mut().set_collision_bits(bak);
        }

        self.player_state_view_mut().set_pit_correction_active();

        if r14 & 7 == 7 {
            self.snap_on_x();
        } else {
            if self.player_state_view().num_orthogonal_directions() == 2 {
                self.handle_pushing_bonking_snaps_return();
                return;
            }
            self.snap_on_x();
            if self.player_state_view().num_orthogonal_directions() == 1 {
                self.handle_pushing_bonking_snaps_return();
                return;
            }
        }

        if (r14 & 5) == 5 {
            self.link_bonk_and_smash();
            self.repel_dash();
        } else if r14 & 2 == 0 {
            let x_vel = self.player_state_view().x_velocity();
            let tt = if r14 & 4 != 0 {
                if (x_vel as i8).is_negative() {
                    x_vel
                } else {
                    0u8.wrapping_sub(x_vel)
                }
            } else if (x_vel as i8).is_negative() {
                0u8.wrapping_sub(x_vel)
            } else {
                x_vel
            };
            let delta = if (tt as i8).is_negative() {
                -1i16
            } else {
                1i16
            };
            if self.player_state_view().y() & 7 != 0 {
                let y = self.player_state_view().y();
                self.player_state_view_mut()
                    .set_y(y.wrapping_add(delta as u16));
                self.handle_nudging(delta as i8);
                return;
            }
            self.link_bonk_and_smash();
            self.repel_dash();
        }

        if self.handle_pushing_bonking_dragstate_tail() {
            return;
        }
        self.handle_pushing_bonking_snaps_return();
    }

    fn handle_pushing_bonking_dragstate_tail(&mut self) -> bool {
        if self
            .player_state_view()
            .last_direction_moved_towards()
            .wrapping_mul(2)
            != self.player_state_view().facing()
        {
            return false;
        }

        self.replay_trace_drag_tail("tail-match-entry");
        let drag_bits = (self.player_state_view().tile_coll_flag() & 1) << 1;
        self.player_state_view_mut().or_defense_flags(drag_bits);
        self.replay_trace_drag_tail("tail-after-lowbit");
        let push_fatigue_timer = self.player_state_view_mut().decrement_push_fatigue_timer();
        if self.player_state_view().button_b_frames() == 0
            && !(push_fatigue_timer as i8).is_negative()
        {
            self.replay_trace_drag_tail("tail-return-timer");
            return true;
        }

        let tile_coll = self.player_state_view().tile_coll_flag();
        let drag_bits = if self.tile_detect_position_view().misc_tiles() & 0x20 != 0 {
            tile_coll << 3
        } else {
            tile_coll
        };
        self.player_state_view_mut().or_defense_flags(drag_bits);
        self.replay_trace_drag_tail("tail-after-fullbits");
        false
    }

    fn handle_pushing_bonking_snaps_return(&mut self) {
        self.player_state_view_mut().reset_push_fatigue_timer();
        self.player_state_view_mut().and_defense_flags(!2);
    }

    pub(super) fn push_block_attempt_to_push_the_block(&self, what: u8, x: u16, y: u16) -> bool {
        const Y0: [i8; 4] = [-4, 20, 4, 4];
        const Y1: [i8; 4] = [-4, 20, 12, 12];
        const X0: [i8; 4] = [4, 4, -4, 20];
        const X1: [i8; 4] = [12, 12, -4, 20];

        let idx =
            what as usize * 4 + self.player_state_view().last_direction_moved_towards() as usize;
        let mask = self.tile_detect_position_view().location_calc_mask();

        let x0 = (x.wrapping_add(X0[idx] as i16 as u16) & mask) >> 3;
        let y0 = y.wrapping_add(Y0[idx] as i16 as u16) & mask;
        let xt = self.push_block_get_target_tile_flag(x0, y0);
        if push_block_target_is_blocked(xt) {
            return true;
        }

        let x1 = (x.wrapping_add(X1[idx] as i16 as u16) & mask) >> 3;
        let y1 = y.wrapping_add(Y1[idx] as i16 as u16) & mask;
        push_block_target_is_blocked(self.push_block_get_target_tile_flag(x1, y1))
    }

    pub(super) fn link_find_valid_landing_tile_north(&mut self) {
        const DY: [u8; 32] = [
            16, 16, 20, 20, 24, 24, 28, 28, 32, 32, 36, 36, 40, 40, 44, 44, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        ];
        const DZ: [u8; 32] = [
            24, 24, 24, 24, 28, 28, 28, 28, 32, 32, 32, 32, 36, 36, 36, 36, 40, 40, 40, 40, 44, 44,
            44, 44, 48, 48, 48, 48, 52, 52, 52, 52,
        ];
        const TIMER: [u8; 32] = [
            16, 16, 20, 20, 24, 24, 28, 28, 32, 32, 36, 36, 40, 40, 44, 44, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        ];

        let y_coord_bak = self.player_state_view().y();
        self.player_state_view_mut()
            .set_hop_origin_coord(y_coord_bak);
        loop {
            let y = self.player_state_view().y().wrapping_sub(16);
            self.player_state_view_mut().set_y(y);
            self.tile_detect_movement_y(
                self.player_state_view().last_direction_moved_towards() as u16
            );
            let terrain = self.tile_detect_position_view().normal_tiles()
                | self.tile_detect_position_view().destruction_aftermath()
                | self.tile_detect_position_view().thick_grass()
                | self.tile_detect_position_view().deepwater();
            if terrain & 7 == 7 {
                break;
            }
        }

        if self.tile_detect_position_view().deepwater() & 7 != 0 {
            self.player_state_view_mut().set_auxiliary_state(1);
            self.player_state_view_mut().clear_electrocute_on_touch();
            self.player_state_view_mut().enter_deep_water_state();
            self.player_state_view_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.player_state_view_mut().clear_grabbing_wall();
            self.player_state_view_mut().set_speed_setting(0);
        }

        let y = self.player_state_view().y().wrapping_sub(16);
        self.player_state_view_mut().set_y(y);
        let diff = self.player_state_view_mut().set_hop_origin_delta_from_y(y);
        self.player_state_view_mut().set_y(y_coord_bak);
        let o = ((diff as u8) >> 3) as usize;
        let dy = DY[o];
        let actual_y_velocity = if self.player_state_view().last_direction_moved_towards() != 0 {
            dy
        } else {
            0u8.wrapping_sub(dy)
        };
        self.player_state_view_mut()
            .set_actual_velocity_xy(0, actual_y_velocity);
        self.player_state_view_mut()
            .set_actual_z_velocity_and_copy(DZ[o]);
        self.player_state_view_mut().set_z(0);
        self.player_state_view_mut()
            .set_incapacitated_timer(TIMER[o]);
        self.player_state_view_mut().set_auxiliary_state(2);
        self.player_state_view_mut().clear_electrocute_on_touch();
        self.player_state_view_mut().set_handler_state(6);
    }

    pub(super) fn link_find_valid_landing_tile_diagonal_north(&mut self) {
        const DX: [u8; 32] = [
            8, 8, 8, 8, 16, 16, 16, 16, 24, 24, 24, 24, 16, 16, 16, 16, 8, 20, 20, 20, 24, 24, 24,
            24, 28, 28, 28, 28, 32, 32, 32, 32,
        ];
        const DY: [u8; 32] = [
            8, 8, 8, 8, 16, 16, 20, 20, 24, 24, 24, 24, 32, 32, 32, 32, 8, 20, 20, 20, 24, 24, 24,
            24, 28, 28, 28, 28, 32, 32, 32, 32,
        ];
        const DZ: [u8; 32] = [
            32, 32, 32, 32, 32, 32, 32, 32, 36, 36, 36, 36, 40, 40, 40, 40, 32, 40, 40, 40, 44, 44,
            44, 44, 48, 48, 48, 48, 52, 52, 52, 52,
        ];

        let y_safe = self.player_state_view().safe_return_y_low();
        let x_bak = self.player_state_view().x();
        let dir = self.player_state_view().last_direction_moved_towards();

        let actual_x_velocity = if self.player_state_view().last_direction_moved_towards() != 2 {
            1
        } else {
            0xff
        };
        self.player_state_view_mut()
            .set_actual_x_velocity(actual_x_velocity);
        self.player_state_view_mut()
            .set_last_direction_moved_towards(0);
        self.link_hop_find_landing_spot_diagonally_down();

        self.player_state_view_mut().set_x(x_bak);
        self.player_state_view_mut().set_safe_return_y_low(y_safe);

        let diff = self
            .player_state_view()
            .hop_origin_coord()
            .wrapping_sub(self.player_state_view().y());
        let o = (diff >> 3) as usize;
        self.player_state_view_mut().restore_y_from_hop_origin();

        let actual_y_velocity = 0u8.wrapping_sub(DY[o]);
        let actual_x_velocity = if dir != 2 {
            DX[o]
        } else {
            0u8.wrapping_sub(DX[o])
        };
        self.player_state_view_mut()
            .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);
        self.player_state_view_mut()
            .set_actual_z_velocity_and_copy(DZ[o]);
        self.player_state_view_mut().set_z(0);
        self.player_state_view_mut().clear_z_mirror_word_low();
        self.player_state_view_mut().set_auxiliary_state(2);
        self.player_state_view_mut().clear_electrocute_on_touch();
        self.player_state_view_mut().set_handler_state(13);
    }

    pub(super) fn tile_detect_main_handler(&mut self, item: u8) {
        self.tile_detect_position_view_mut().clear_pit_tile();
        self.tile_detect_reset_state();

        let probe_base = if item == 8 {
            let spin = self
                .player_state_view()
                .state_for_spin_attack()
                .wrapping_sub(2);
            if spin >= 8 {
                return;
            }
            const SPIN_OFFSETS: [u8; 8] = [10, 6, 14, 2, 12, 4, 8, 0];
            SPIN_OFFSETS[spin as usize] as u16 + 0x40
        } else {
            item as u16 * 8 + self.player_state_view().facing() as u16
        };
        let offset = probe_base >> 1;
        const X: [i8; 40] = [
            8, 8, 8, 8, 6, 8, -1, 22, 19, 19, 0, 19, 6, 8, -1, 22, 8, 8, 8, 8, 8, 8, 0, 15, 6, 8,
            -10, 29, 6, 8, -6, 22, 6, 8, -4, 22, -4, 22, -4, 22,
        ];
        const Y: [i8; 40] = [
            20, 20, 20, 20, 4, 28, 16, 16, 22, 22, 22, 22, 4, 24, 16, 16, 16, 16, 16, 16, 20, 20,
            23, 23, -4, 36, 16, 16, 4, 28, 16, 16, 4, 28, 16, 16, 4, 4, 28, 28,
        ];
        let mask = self.tile_detect_position_view().location_calc_mask();
        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        let x = (link_x.wrapping_add(X[offset as usize] as i16 as u16) & mask) >> 3;
        let y = link_y.wrapping_add(Y[offset as usize] as i16 as u16) & mask;

        if matches!(item, 1 | 2 | 3 | 6 | 7 | 8) {
            self.tile_behavior_handle_item_and_execute(x, y);
            return;
        }

        self.tile_detection_execute(x, y, 1);
        if item == 5 {
            return;
        }

        if self.tile_detect_position_view().thick_grass() & 0x10 != 0 {
            let tx = self.player_state_view().x() & 0x0f;
            let ty = self.player_state_view().y().wrapping_add(8) & 0x0f;
            if !(4..11).contains(&ty)
                && !(4..12).contains(&tx)
                && self.player_state_view().blink_countdown() == 0
                && !self.player_state_view().has_auxiliary_state()
            {
                if self.world_location_state().is_indoors() {
                    self.Dungeon_FlagRoomData_Quadrants();
                    self.ancilla_sfx2_near(0x33);
                    self.player_state_view_mut().set_speed_setting(0);
                    self.set_submodule(21);
                    let prev_room = self.world_location_state().dungeon_room_index();
                    self.dungeon_state_view_mut().set_room_index_prev(prev_room);
                    let room = self.dungeon_header_view().travel_destination(0);
                    self.set_dungeon_room_index(room);
                    self.handle_layer_of_destination();
                } else if !self.player_state_view_mut().whirlpool_triggered() {
                    self.do_sword_interaction_with_tiles_mirror();
                }
            }
        } else {
            self.player_state_view_mut().clear_whirlpool_trigger();
            if self.tile_detect_position_view().thick_grass() & 1 != 0 {
                self.player_state_view_mut()
                    .set_water_ripple_or_grass_state(2);
                if !self.link_permission_for_slosh_sounds()
                    && !self.player_state_view().has_auxiliary_state()
                {
                    self.ancilla_sfx2_near(26);
                }
                return;
            }

            if self.tile_detect_position_view().shallow_water() & 1 != 0 {
                self.player_state_view_mut()
                    .set_water_ripple_or_grass_state(1);
                if self.world_location_state().is_outdoors()
                    && self.player_state_view().is_in_deep_water()
                    && !self.player_state_view().is_bunny_mirror()
                {
                    if self.player_state_view().has_flippers() {
                        self.player_state_view_mut().clear_deep_water_state();
                        self.player_state_view_mut()
                            .set_last_direction_from_swim_flags();
                        self.player_state_view_mut().clear_handler_state();
                    }
                } else if !self.link_permission_for_slosh_sounds() {
                    if self.world_location_state().overworld_screen_index() == 0x70 {
                        self.ancilla_sfx2_near(27);
                    } else if !self.player_state_view().has_auxiliary_state() {
                        self.ancilla_sfx2_near(28);
                    }
                }
                return;
            }

            if self.world_location_state().is_outdoors()
                && !self.player_state_view().is_in_deep_water()
                && self.tile_detect_position_view().deepwater() & 1 != 0
            {
                self.player_state_view_mut()
                    .set_water_ripple_or_grass_state(1);
                if !self.link_permission_for_slosh_sounds() {
                    if self.world_location_state().overworld_screen_index() == 0x70 {
                        self.ancilla_sfx2_near(27);
                    } else if !self.player_state_view().has_auxiliary_state() {
                        self.ancilla_sfx2_near(28);
                    }
                }
                return;
            }
        }

        self.player_state_view_mut()
            .clear_water_ripple_or_grass_state();
        if self.tile_detect_position_view().spike_floor_and_triggers() & 1 != 0 {
            self.player_state_view_mut().set_item_pickup_in_progress(1);
            return;
        }
        self.player_state_view_mut().set_item_pickup_in_progress(0);

        if self.tile_detect_position_view().spike_floor_and_triggers() & 0x10 != 0 {
            self.player_state_view_mut().clear_given_damage();
            if !self.player_state_view().is_cape_active()
                && !self.search_for_byrna_spark()
                && self.player_state_view().blink_countdown() == 0
            {
                self.player_state_view_mut()
                    .clear_transform_poof_need_and_temp_bunny_timer();
                if self.player_state_view().has_moon_pearl() {
                    self.player_state_view_mut()
                        .clear_bunny_transform_after_moon_pearl();
                }
                self.player_state_view_mut().set_given_damage(8);
                self.link_cancel_dash();
                return;
            }
        }

        if self.tile_detect_position_view().icy_floor() & 0x11 != 0 {
            if self.player_state_view().flag_moving() != 0 {
                if self.player_state_view().num_orthogonal_directions() != 0 {
                    self.player_state_view_mut()
                        .set_last_direction_from_swim_flags();
                }
            } else {
                if self.player_state_view().direction() & 0x0c != 0 {
                    self.swim_acceleration_view_mut()
                        .set_acceleration(0, 0x0180);
                }
                if self.player_state_view().direction() & 3 != 0 {
                    self.swim_acceleration_view_mut()
                        .set_acceleration(0, 0x0180);
                }
                let flag_moving = if self.tile_detect_position_view().icy_floor() & 1 != 0 {
                    1
                } else {
                    2
                };
                self.player_state_view_mut().set_flag_moving(flag_moving);
                self.player_state_view_mut()
                    .set_swim_flags_from_last_direction();
                self.link_reset_swimming_state();
            }
        } else {
            if self.player_state_view().handler_state() != 4 {
                if self.player_state_view().flag_moving() != 0 {
                    self.player_state_view_mut()
                        .set_last_direction_from_swim_flags();
                }
                self.link_reset_swimming_state();
            }
            self.player_state_view_mut().set_flag_moving(0);
        }

        if self.tile_detect_position_view().spike_cactus_tiles() & 0x10 != 0
            && self.player_state_view().blink_countdown() == 0
        {
            self.player_state_view_mut().set_blink_countdown(58);
        }
    }

    pub(super) fn start_movement_collision_checks_y(&mut self) {
        self.replay_trace_submodule("start-y-entry");
        if self.player_state_view().y_velocity() == 0 {
            return;
        }
        let last_direction_moved_towards = if self.player_state_view().doorway_state() == 1 {
            if self.player_state_view().y_low() < 0x80 {
                0
            } else {
                1
            }
        } else if self.player_state_view().y_velocity_signed().is_negative() {
            0
        } else {
            1
        };
        self.player_state_view_mut()
            .set_last_direction_moved_towards(last_direction_moved_towards);
        self.tile_detect_movement_y(last_direction_moved_towards as u16);
        self.replay_trace_submodule("start-y-after-tiledetect");
        if self.world_location_state().is_indoors() {
            self.start_movement_collision_checks_y_handle_indoors();
        } else {
            self.start_movement_collision_checks_y_handle_outdoors();
        }
    }

    pub(super) fn start_movement_collision_checks_x(&mut self) {
        if self.player_state_view().x_velocity() == 0 {
            return;
        }
        let last_direction_moved_towards = if self.player_state_view().doorway_state() == 2 {
            if self.player_state_view().x_low() < 0x80 {
                2
            } else {
                3
            }
        } else if self.player_state_view().x_velocity_signed().is_negative() {
            2
        } else {
            3
        };
        self.player_state_view_mut()
            .set_last_direction_moved_towards(last_direction_moved_towards);
        self.tile_detect_movement_x(last_direction_moved_towards as u16);
        if self.world_location_state().is_indoors() {
            self.start_movement_collision_checks_x_handle_indoors();
        } else {
            self.start_movement_collision_checks_x_handle_outdoors();
        }
    }

    pub(super) fn start_movement_collision_checks_y_handle_indoors(&mut self) {
        let mut r14 = self.tile_detect_position_view().collision_bits();
        if self.player_state_view().is_lifting_or_carrying()
            || self.player_state_view().incapacitated_timer() != 0
        {
            r14 |= r14 >> 4;
            self.tile_detect_position_view_mut().set_collision_bits(r14);
        } else {
            if self.player_state_view().doorway_state() == 2 {
                if self.player_state_view().num_orthogonal_directions() == 0 {
                    if self.dungeon_state_view().header_collision() != 3
                        || !self.player_state_view().is_on_lower_level()
                    {
                        self.link_add_in_velocity_y();
                        self.change_axis_of_perpendicular_door_movement_y();
                        return;
                    }
                } else if self.tile_detect_position_view().door_direction_flags() != 0 {
                    self.link_add_in_velocity_y();
                    self.finish_indoor_y_collision();
                    return;
                }
            }

            if r14 & 0x70 != 0 {
                if (r14 >> 8) & 7 != 0 {
                    let force_move = if self.player_state_view().y_velocity_signed().is_negative() {
                        8
                    } else {
                        4
                    };
                    self.player_state_view_mut()
                        .set_force_move_any_direction(force_move);
                }

                self.player_state_view_mut().set_doorway_state(1);
                self.player_state_view_mut().clear_conveyor_belt_state();
                if r14 & 0x70 != 0x70 {
                    if r14 & 5 != 0 {
                        self.player_state_view_mut()
                            .clear_moving_against_diag_tile();
                        self.link_add_in_velocity_y_falling();
                        self.calculate_snap_scratch_y();
                        self.player_state_view_mut().clear_doorway_state();
                        if r14 & 0x20 != 0 && r14 & 1 == 0 && self.player_state_view().x() & 7 == 1
                        {
                            let x = self.player_state_view().x() & !7;
                            self.player_state_view_mut().set_x(x);
                        }
                        if self.player_state_view().tile_coll_flag() & 2 == 0 {
                            self.player_state_view_mut().clear_direction_lock_bits(2);
                        }
                        return;
                    }
                    if r14 & 0x20 != 0 {
                        if self.player_state_view().tile_coll_flag() & 2 == 0 {
                            self.player_state_view_mut().clear_direction_lock_bits(2);
                        }
                        return;
                    }
                } else {
                    if self.player_state_view().tile_coll_flag() & 2 == 0 {
                        self.player_state_view_mut().clear_direction_lock_bits(2);
                    }
                    return;
                }
            }
        }

        if self.player_state_view().tile_coll_flag() & 2 == 0 {
            self.player_state_view_mut().clear_doorway_state();
        }
        self.finish_indoor_y_collision();
    }

    pub(super) fn start_movement_collision_checks_x_handle_indoors(&mut self) {
        let mut r14 = self.tile_detect_position_view().collision_bits();
        if self.player_state_view().is_lifting_or_carrying()
            || self.player_state_view().incapacitated_timer() != 0
        {
            r14 |= r14 >> 4;
            self.tile_detect_position_view_mut().set_collision_bits(r14);
        } else {
            if self.player_state_view().num_orthogonal_directions() == 0 {
                self.player_state_view_mut().clear_speed_modifier();
            }

            if self.player_state_view().doorway_state() == 1
                && self.player_state_view().num_orthogonal_directions() == 0
                && (self.dungeon_state_view().header_collision() != 3
                    || !self.player_state_view().is_on_lower_level())
            {
                self.snap_on_x();
                let spd = self.change_axis_of_perpendicular_door_movement_x();
                self.handle_nudging_in_a_door(spd);
                return;
            }

            if r14 & 0x70 != 0 {
                if (r14 >> 8) & 7 != 0 {
                    let force_move = if self.player_state_view().x_velocity_signed().is_negative() {
                        2
                    } else {
                        1
                    };
                    self.player_state_view_mut()
                        .set_force_move_any_direction(force_move);
                }

                self.player_state_view_mut().set_doorway_state(2);
                self.player_state_view_mut().clear_conveyor_belt_state();
                if r14 & 0x70 != 0x70 {
                    if r14 & 7 != 0 {
                        self.player_state_view_mut()
                            .clear_moving_against_diag_tile();
                        self.player_state_view_mut().clear_doorway_state();
                        self.snap_on_x();
                        self.calculate_snap_scratch_x();
                        return;
                    }
                    self.player_state_view_mut().clear_direction_lock_bits(2);
                    return;
                }

                self.player_state_view_mut().clear_direction_lock_bits(2);
                return;
            }
        }

        if self.player_state_view().tile_coll_flag() & 2 == 0 {
            self.player_state_view_mut().clear_direction_lock_bits(2);
            self.player_state_view_mut().clear_doorway_state();
            self.world_state_view_mut().set_room_transitioning_flags(0);
            self.player_state_view_mut().set_force_move_any_direction(0);
        }

        if self.tile_detect_position_view().collision_bits() & 2 == 0
            && self.tile_detect_position_view().slope_collision_bits() & 5 != 0
        {
            self.player_state_view_mut().clear_conveyor_belt_state();
            self.flag_moving_into_slopes_x();
            if self.player_state_view().moving_against_diag_tile() & 0x0f != 0 {
                return;
            }
        }

        self.player_state_view_mut()
            .clear_moving_against_diag_tile();
        self.finish_indoor_collision_common(false);
    }

    pub(super) fn start_movement_collision_checks_y_handle_outdoors(&mut self) {
        self.replay_trace_submodule("outdoor-y-entry");
        self.replay_trace_drag_tail("outdoor-y-drag-entry");
        self.player_state_view_mut().resolve_dash_speed_setting();
        self.replay_trace_drag_tail("outdoor-y-after-speed-setting");

        if self.tile_detect_position_view().pit_tile() & 5 != 0
            && self.tile_detect_position_view().collision_bits() & 2 == 0
        {
            self.start_falling_into_hole();
            return;
        }

        let liftable_primary = if self.tile_detect_position_view().read_something() & 2 != 0 {
            self.tile_detect_position_view().liftable_tile_index() >> 1
        } else {
            0
        };
        self.tile_detect_position_view_mut()
            .set_liftable_action_index_primary(liftable_primary);

        if self.tile_detect_position_view().deepwater() & 2 != 0
            && !self.player_state_view().is_in_deep_water()
            && !self.player_state_view().has_auxiliary_state()
        {
            self.link_reset_sword_and_item_usage();
            self.link_cancel_dash();
            self.player_state_view_mut().enter_deep_water_state();
            self.player_state_view_mut()
                .set_swim_flags_from_last_direction();
            self.player_state_view_mut().clear_grabbing_wall();
            self.player_state_view_mut().set_speed_setting(0);
            self.link_reset_swimming_state();
            if self.player_state_view().water_ripple_or_grass_state() == 1 && {
                self.link_force_unequip_cape_quietly();
                self.player_state_view().has_flippers()
            } {
                if !self.player_state_view().is_bunny_mirror() {
                    self.player_state_view_mut().set_handler_state(4);
                }
            } else {
                self.ancilla_sfx2_near(0x20);
                self.restore_link_safe_return_position();
                self.player_state_view_mut()
                    .set_sprite_damage_disable_timer(1);
                self.link_hop_in_or_out_of_water_y();
            }
        }

        if self.player_state_view().is_in_deep_water() {
            if self.tile_detect_position_view().vertical_ledge() & 7 != 0 {
                let r14 = (self.tile_detect_position_view().vertical_ledge() & 7) as u16;
                self.tile_detect_position_view_mut().set_collision_bits(r14);
                self.handle_pushing_bonking_snaps_y();
                return;
            }
            if (self.tile_detect_position_view().stair_tile() & 7) == 7
                || self.tile_detect_position_view().normal_tiles() & 7 == 7
            {
                self.link_cancel_dash();
                self.player_state_view_mut().clear_deep_water_state();
                if self.player_state_view().auxiliary_state() == 0 {
                    self.player_state_view_mut()
                        .set_last_direction_from_swim_flags();
                    self.player_state_view_mut()
                        .set_sprite_damage_disable_timer(1);
                    self.ancilla_add_splash(0x15, 0);
                    self.link_hop_in_or_out_of_water_y();
                    return;
                }
            }
        }

        if self.tile_detect_position_view().horizontal_ledge() & 2 != 0
            || self.tile_detect_position_view().diagonal_ledge_tiles() & 0x22 != 0
        {
            self.tile_detect_position_view_mut().set_collision_bits(7);
            self.handle_pushing_bonking_snaps_y();
            return;
        }

        if self.tile_detect_position_view().vertical_ledge() & 0x70 != 0
            && self.run_ledge_hop_timer()
        {
            self.link_cancel_dash();
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.player_state_view_mut().set_allow_scroll_z(1);
            self.player_state_view_mut().set_handler_state(11);
            self.player_state_view_mut().set_incapacitated_timer(0);
            self.player_state_view_mut().set_z_mirror(0xffff);
            self.player_state_view_mut().clear_defense_flags();
            self.player_state_view_mut().set_speed_setting(0);
            let zvel = if self.player_state_view().is_in_deep_water() {
                14
            } else {
                20
            };
            self.player_state_view_mut()
                .set_actual_z_velocity_mirror_and_copy(zvel);
            let auxiliary_state = if self.player_state_view().is_in_deep_water() {
                4
            } else {
                2
            };
            self.player_state_view_mut()
                .set_auxiliary_state(auxiliary_state);
            return;
        }

        if self.tile_detect_position_view().vertical_ledge() & 7 != 0 && self.run_ledge_hop_timer()
        {
            self.ancilla_sfx2_near(0x20);
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_cancel_dash();
            self.player_state_view_mut().clear_defense_flags();
            self.player_state_view_mut().set_speed_setting(0);
            self.link_find_valid_landing_tile_north();
            return;
        }

        if !self.player_state_view().is_in_deep_water() {
            if self.tile_detect_position_view().ledges_down_leftright() & 7 != 0
                && self.tile_detect_position_view().vertical_ledge() & 0x77 == 0
            {
                let xand = if self.tile_detect_position_view().interacting_tile() == 0x2f {
                    4
                } else {
                    1
                };
                if self.tile_detect_position_view().ledges_down_leftright() & xand != 0
                    && self.run_ledge_hop_timer()
                {
                    self.link_cancel_dash();
                    let actual_x_velocity =
                        if self.tile_detect_position_view().ledges_down_leftright() & 4 != 0 {
                            16
                        } else {
                            0u8.wrapping_sub(16)
                        };
                    self.player_state_view_mut()
                        .set_actual_x_velocity(actual_x_velocity);
                    self.setup_horizontal_ledge_hop(14);
                    return;
                }
            }

            if self.tile_detect_position_view().horizontal_ledge() & 0x70 != 0
                && self.tile_detect_position_view().vertical_ledge() & 0x77 == 0
                && self.run_ledge_hop_timer()
            {
                self.link_cancel_dash();
                self.ancilla_sfx2_near(0x20);
                let last_direction_moved_towards =
                    if self.tile_detect_position_view().horizontal_ledge() & 0x40 != 0 {
                        3
                    } else {
                        2
                    };
                self.player_state_view_mut()
                    .set_last_direction_moved_towards(last_direction_moved_towards);
                self.player_state_view_mut()
                    .set_sprite_damage_disable_timer(1);
                self.player_state_view_mut().clear_defense_flags();
                self.player_state_view_mut().set_speed_setting(0);
                self.link_find_valid_landing_tile_diagonal_north();
                return;
            }
        }

        if (self.tile_detect_position_view().stair_tile() & 7) == 7 {
            if self.player_state_view().incapacitated_timer() != 0 {
                let r14 = (self.tile_detect_position_view().stair_tile() & 7) as u16;
                self.tile_detect_position_view_mut().set_collision_bits(r14);
                self.handle_pushing_bonking_snaps_y();
                return;
            } else if self.player_state_view().last_direction_moved_towards() & 2 == 0 {
                self.player_state_view_mut().arm_stair_speed_modifier();
                return;
            }
        }

        self.player_state_view_mut().resolve_dash_speed_setting();
        self.replay_trace_drag_tail("outdoor-y-after-late-speed-setting");
        if self.player_state_view().speed_modifier() == 1 {
            self.player_state_view_mut()
                .promote_pending_speed_modifier();
        }
        self.replay_trace_drag_tail("outdoor-y-after-speed-modifier");

        if self.tile_detect_position_view().collision_bits() & 7 == 0
            && self.tile_detect_position_view().slope_collision_bits() & 5 != 0
        {
            self.replay_trace_drag_tail("outdoor-y-before-slopes");
            self.flag_moving_into_slopes_y();
            self.replay_trace_drag_tail("outdoor-y-after-slopes");
            if self.player_state_view().moving_against_diag_tile() & 0x0f != 0 {
                return;
            }
        }

        self.player_state_view_mut()
            .clear_moving_against_diag_tile();
        self.replay_trace_drag_tail("outdoor-y-after-clear-diag");
        if self.tile_detect_position_view().key_lock_gravestones_low() & 2 != 0
            && self.player_state_view().last_direction_moved_towards() == 0
        {
            let timeout = self
                .player_state_view()
                .gravestone_push_timeout()
                .wrapping_sub(1);
            self.player_state_view_mut()
                .set_gravestone_push_timeout(timeout);
            if self.player_state_view().is_running() || (timeout as i8).is_negative() {
                let bak = self.tile_detect_position_view().collision_bits();
                self.ancilla_add_grave_stone(0x24, 4);
                self.tile_detect_position_view_mut().set_collision_bits(bak);
                self.player_state_view_mut().set_gravestone_push_timeout(52);
            }
        } else {
            self.player_state_view_mut().set_gravestone_push_timeout(52);
        }
        self.replay_trace_drag_tail("outdoor-y-after-gravestone");

        if self.tile_detect_position_view().spike_cactus_tiles() & 7 != 0 {
            if (self.player_state_view().incapacitated_timer()
                | self.player_state_view().blink_countdown()
                | self.player_state_view().is_cape_active() as u8)
                == 0
            {
                let should_damage = if self.player_state_view().last_direction_moved_towards() == 0
                {
                    self.player_state_view().y() & 4 == 0
                } else {
                    self.player_state_view().y() & 4 != 0
                };
                if should_damage {
                    self.player_state_view_mut().set_given_damage(8);
                    self.link_cancel_dash();
                    self.link_force_unequip_cape_quietly();
                    self.link_apply_tile_rebound();
                    return;
                }
            } else {
                let r14 = (self.tile_detect_position_view().spike_cactus_tiles() & 7) as u16;
                self.tile_detect_position_view_mut().set_collision_bits(r14);
            }
        }
        self.replay_trace_drag_tail("outdoor-y-before-snaps");
        self.handle_pushing_bonking_snaps_y();
        self.replay_trace_submodule("outdoor-y-after-snaps");
    }

    pub(super) fn start_movement_collision_checks_x_handle_outdoors(&mut self) {
        if self.player_state_view().num_orthogonal_directions() == 0 {
            self.player_state_view_mut().clear_speed_modifier();
            if self.player_state_view().speed_setting() == 2 {
                self.player_state_view_mut().set_speed_setting(0);
            }
        }

        if self.tile_detect_position_view().pit_tile() & 5 != 0
            && self.tile_detect_position_view().collision_bits() & 2 == 0
        {
            self.start_falling_into_hole();
            return;
        }

        let liftable_secondary = if self.tile_detect_position_view().read_something() & 2 != 0 {
            self.tile_detect_position_view().liftable_tile_index() >> 1
        } else {
            0
        };
        self.tile_detect_position_view_mut()
            .set_liftable_action_index_secondary(liftable_secondary);

        if self.tile_detect_position_view().deepwater() & 4 != 0
            && !self.player_state_view().is_in_deep_water()
            && !self.player_state_view().has_auxiliary_state()
        {
            self.link_cancel_dash();
            self.link_reset_sword_and_item_usage();
            self.player_state_view_mut().enter_deep_water_state();
            self.player_state_view_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.player_state_view_mut().clear_grabbing_wall();
            self.player_state_view_mut().set_speed_setting(0);
            if self.player_state_view().water_ripple_or_grass_state() == 1 && {
                self.link_force_unequip_cape_quietly();
                self.player_state_view().has_flippers()
            } {
                if !self.player_state_view().is_bunny_mirror() {
                    self.player_state_view_mut().set_handler_state(4);
                }
            } else {
                self.restore_link_safe_return_position();
                self.player_state_view_mut()
                    .set_sprite_damage_disable_timer(1);
                self.link_hop_in_or_out_of_water_x();
                self.ancilla_sfx2_near(0x20);
            }
        }

        if if self.player_state_view().is_in_deep_water() {
            self.tile_detect_position_view().horizontal_ledge() & 7 == 7
        } else {
            self.tile_detect_position_view().vertical_ledge() & 0x42 != 0
        } {
            self.tile_detect_position_view_mut().set_collision_bits(7);
            self.handle_pushing_bonking_snaps_x();
            return;
        }

        if self.tile_detect_position_view().normal_tiles() & 7 == 7
            && self.player_state_view().is_in_deep_water()
        {
            self.link_cancel_dash();
            if self.player_state_view().auxiliary_state() == 0 {
                self.player_state_view_mut()
                    .set_last_direction_from_swim_flags();
                self.player_state_view_mut().clear_deep_water_state();
                self.player_state_view_mut()
                    .set_sprite_damage_disable_timer(1);
                self.ancilla_add_splash(0x15, 0);
                self.link_hop_in_or_out_of_water_x();
                return;
            }
        }

        if self.tile_detect_position_view().horizontal_ledge() & 7 != 0
            && self.run_ledge_hop_timer()
        {
            self.ancilla_sfx2_near(0x20);
            let actual_x_velocity =
                if self.player_state_view().last_direction_moved_towards() & 1 != 0 {
                    0x10
                } else {
                    0u8.wrapping_sub(0x10)
                };
            self.player_state_view_mut()
                .set_actual_x_velocity(actual_x_velocity);
            self.link_cancel_dash();
            self.setup_horizontal_ledge_hop(12);
            if self.world_location_state().is_outdoors() {
                self.player_state_view_mut().set_lower_level_state(2);
            }
            let x_bak = self.player_state_view().x();
            let rv = self.link_hopping_horizontally_find_tile_x(
                (self.player_state_view().last_direction_moved_towards() & !2) * 2,
            );
            self.player_state_view_mut()
                .set_last_direction_moved_towards(1);
            if rv != 0xff {
                self.link_hopping_horizontally_find_tile_y();
            } else {
                self.link_hop_find_tile_to_land_on_south();
            }
            self.player_state_view_mut().set_x(x_bak);
            return;
        }

        if self.tile_detect_position_view().diagonal_ledge_tiles() & 0x77 != 0
            && self.run_ledge_hop_timer()
        {
            self.ancilla_sfx2_near(0x20);
            let handler_state = if self.system_signals_view().sound_effect_1() & 7 == 0 {
                16
            } else {
                15
            };
            self.player_state_view_mut()
                .set_handler_state(handler_state);
            let actual_x_velocity =
                if self.player_state_view().last_direction_moved_towards() & 1 != 0 {
                    0x10
                } else {
                    0u8.wrapping_sub(0x10)
                };
            self.player_state_view_mut()
                .set_actual_x_velocity(actual_x_velocity);
            self.link_cancel_dash();
            self.player_state_view_mut().set_auxiliary_state(2);
            self.player_state_view_mut()
                .set_actual_z_velocity_mirror_and_copy(20);
            self.set_link_z_coord_mirror_low_ff();
            self.player_state_view_mut().set_incapacitated_timer(0);
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.player_state_view_mut().set_allow_scroll_z(1);
            self.player_state_view_mut().clear_defense_flags();
            self.player_state_view_mut().set_speed_setting(0);
            return;
        }

        if self.tile_detect_position_view().horizontal_ledge() & 0x70 != 0
            && self.tile_detect_position_view().horizontal_ledge() & 7 == 0
            && self.tile_detect_position_view().diagonal_ledge_tiles() & 0x77 == 0
            && self.player_state_view().handler_state() != 13
            && self.run_ledge_hop_timer()
        {
            self.ancilla_sfx2_near(0x20);
            self.link_cancel_dash();
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.player_state_view_mut().clear_defense_flags();
            self.player_state_view_mut().set_speed_setting(0);
            self.link_find_valid_landing_tile_diagonal_north();
            return;
        }

        if self.tile_detect_position_view().ledges_down_leftright() & 7 != 0
            && self.tile_detect_position_view().horizontal_ledge() & 7 == 0
            && self.tile_detect_position_view().diagonal_ledge_tiles() & 0x77 == 0
            && self.run_ledge_hop_timer()
        {
            let actual_x_velocity =
                if self.player_state_view().last_direction_moved_towards() & 1 != 0 {
                    0x10
                } else {
                    0u8.wrapping_sub(0x10)
                };
            self.player_state_view_mut()
                .set_actual_x_velocity(actual_x_velocity);
            self.link_cancel_dash();
            self.setup_horizontal_ledge_hop(14);
            return;
        }

        if self.tile_detect_position_view().collision_bits() & 2 == 0
            && self.tile_detect_position_view().slope_collision_bits() & 5 != 0
        {
            let skip_check =
                self.player_state_view().is_running() && self.player_state_view().facing() & 4 == 0;
            const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
            if !skip_check || self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES) {
                self.flag_moving_into_slopes_x();
                if self.player_state_view().moving_against_diag_tile() & 0x0f != 0 {
                    return;
                }
            }
        }

        self.player_state_view_mut()
            .clear_moving_against_diag_tile();
        if self.tile_detect_position_view().spike_cactus_tiles() & 7 != 0 {
            if (self.player_state_view().incapacitated_timer()
                | self.player_state_view().blink_countdown()
                | self.player_state_view().is_cape_active() as u8)
                == 0
            {
                let should_damage = if self.player_state_view().last_direction_moved_towards() == 2
                {
                    self.player_state_view().x() & 4 == 0
                } else {
                    self.player_state_view().x() & 4 != 0
                };
                if should_damage {
                    self.player_state_view_mut().set_given_damage(8);
                    self.link_cancel_dash();
                    self.link_apply_tile_rebound();
                    return;
                }
            } else {
                let r14 = (self.tile_detect_position_view().spike_cactus_tiles() & 7) as u16;
                self.tile_detect_position_view_mut().set_collision_bits(r14);
            }
        }
        self.handle_pushing_bonking_snaps_x();
    }

    fn start_falling_into_hole(&mut self) {
        if self.player_state_view().handler_state() != 5
            && self.player_state_view().handler_state() != 2
        {
            self.player_state_view_mut().set_sprite_oam_state_timer(9);
            self.player_state_view_mut().begin_pit_check();
            self.player_state_view_mut().set_handler_state(1);
        }
    }

    fn setup_horizontal_ledge_hop(&mut self, player_state: u8) {
        self.player_state_view_mut()
            .set_sprite_damage_disable_timer(1);
        self.player_state_view_mut().clear_defense_flags();
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().set_allow_scroll_z(1);
        self.player_state_view_mut().set_auxiliary_state(2);
        self.player_state_view_mut()
            .set_actual_z_velocity_mirror_and_copy(20);
        self.set_link_z_coord_mirror_low_ff();
        self.player_state_view_mut().set_incapacitated_timer(0);
        self.player_state_view_mut().set_handler_state(player_state);
    }

    fn finish_indoor_y_collision(&mut self) {
        if self.player_state_view().tile_coll_flag() & 2 == 0 {
            self.player_state_view_mut().clear_doorway_state();
            self.player_state_view_mut().clear_direction_lock_bits(2);
            self.world_state_view_mut().set_room_transitioning_flags(0);
            self.player_state_view_mut().set_force_move_any_direction(0);
        }

        if self.tile_detect_position_view().collision_bits() & 7 == 0
            && self.tile_detect_position_view().slope_collision_bits() & 5 != 0
        {
            self.player_state_view_mut().clear_conveyor_belt_state();
            self.flag_moving_into_slopes_y();
            if self.player_state_view().moving_against_diag_tile() & 0x0f != 0 {
                return;
            }
        }

        self.player_state_view_mut()
            .clear_moving_against_diag_tile();
        if self.tile_detect_position_view().key_lock_gravestones() & 0x20 != 0 {
            let bak = self.tile_detect_position_view().collision_bits();
            let mut chest_position = 0;
            let _ = self.OpenChestForItem(
                self.tile_detect_position_view().tile_type() as u8,
                &mut chest_position,
            );
            self.tile_detect_position_view_mut().clear_tile_type();
            self.tile_detect_position_view_mut().set_collision_bits(bak);
        }

        self.finish_indoor_collision_common(true);
    }

    fn finish_indoor_collision_common(&mut self, y_axis: bool) {
        let r14 = self.tile_detect_position_view().collision_bits();
        if !self.player_state_view().is_on_lower_level() {
            if self.tile_detect_position_view().water_staircase() & 7 != 0 {
                self.set_player_layer_collision(
                    crate::game_state::constants::player::LAYER_COLLISION_BG1,
                    true,
                );
            } else if self.tile_detect_position_view().spike_cactus_tiles() & 7 == 0 && r14 & 2 == 0
            {
                self.set_player_layer_collision(
                    crate::game_state::constants::player::LAYER_COLLISION_BG1,
                    false,
                );
            }
        } else if self.tile_detect_position_view().moving_floor_tiles() & 7 != 0 {
            self.set_player_layer_collision(
                crate::game_state::constants::player::LAYER_COLLISION_BG2,
                true,
            );
        } else {
            self.set_player_layer_collision(
                crate::game_state::constants::player::LAYER_COLLISION_BG2,
                false,
            );
        }

        if self.tile_detect_position_view().misc_tiles() & 0x2200 != 0 {
            const DX: [u8; 4] = [8, 8, 0, 15];
            const DY: [u8; 4] = [8, 24, 16, 16];
            let dy = if self.tile_detect_position_view().misc_tiles() & 0x2000 != 0 {
                8
            } else {
                0
            };
            let dir = self.player_state_view().last_direction_moved_towards() as usize;
            let rupees = self.player_resources_view().rupees_goal().wrapping_add(5);
            self.player_resources_view_mut().set_rupees_goal(rupees);
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(DY[dir] as u16)
                .wrapping_sub(dy);
            let x = self.player_state_view().x().wrapping_add(DX[dir] as u16);
            self.dungeon_delete_rupee_tile_for_player(x, y);
            self.ancilla_sfx3_near(10);
        }

        let moving_floor_flags = self.dungeon_state_view().moving_floor_check_flags();
        if moving_floor_flags & 0x22 != 0 {
            self.player_state_view_mut()
                .set_conveyor_belt_state(if moving_floor_flags & 0x20 != 0 { 2 } else { 1 });
        } else if moving_floor_flags & 0x2200 != 0 {
            self.player_state_view_mut().set_conveyor_belt_state(
                if moving_floor_flags & 0x2000 != 0 {
                    4
                } else {
                    3
                },
            );
        } else if self.tile_detect_position_view().spike_cactus_tiles() & 7 == 0 && r14 & 2 == 0 {
            self.player_state_view_mut().clear_conveyor_belt_state();
        }

        if y_axis {
            self.finish_indoor_y_collision_tail();
        } else {
            self.finish_indoor_x_collision_tail();
        }
    }

    fn finish_indoor_y_collision_tail(&mut self) {
        if (self.tile_detect_position_view().vertical_ledge() & 7) == 7
            && self.run_ledge_hop_timer()
        {
            self.link_cancel_dash();
            self.player_state_view_mut()
                .increment_about_to_jump_off_ledge();
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.player_state_view_mut().set_auxiliary_state(2);
            self.ancilla_sfx2_near(0x20);
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_hop_in_or_out_of_water_y();
        } else if (self.tile_detect_position_view().deepwater() & 7) == 7
            && !self.player_state_view().is_in_deep_water()
        {
            self.link_cancel_dash();
            if self.display_state().sub_screen_layers == 0 {
                self.dungeon_handle_layer_change();
            } else {
                self.player_state_view_mut().enter_deep_water_state();
                self.player_state_view_mut()
                    .set_swim_flags_from_last_direction();
                self.player_state_view_mut()
                    .clear_state_item_and_grab_flags();
                self.player_state_view_mut().set_speed_setting(0);
                self.link_reset_swimming_state();
                self.ancilla_sfx2_near(0x20);
            }
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_hop_in_or_out_of_water_y();
        } else if self.tile_detect_position_view().normal_tiles() & 2 != 0
            && self.player_state_view().is_in_deep_water()
        {
            if self.player_state_view().has_auxiliary_state() {
                self.tile_detect_position_view_mut().set_collision_bits(7);
            } else {
                self.link_cancel_dash();
                self.player_state_view_mut()
                    .set_last_direction_from_swim_flags();
                self.player_state_view_mut().clear_deep_water_state();
                if self.ancilla_add_splash(0x15, 0) {
                    self.player_state_view_mut().enter_deep_water_state();
                    self.tile_detect_position_view_mut().set_collision_bits(7);
                } else {
                    self.player_state_view_mut()
                        .set_sprite_damage_disable_timer(1);
                    self.link_hop_in_or_out_of_water_y();
                }
            }
        }

        if (self.tile_detect_position_view().stair_tile() & 7) == 7 {
            if self.player_state_view().incapacitated_timer() != 0 {
                let stair_bits = (self.tile_detect_position_view().stair_tile() & 7) as u16;
                self.tile_detect_position_view_mut()
                    .set_collision_bits(stair_bits);
                self.handle_pushing_bonking_snaps_y();
                return;
            }
            let stairs = self.tile_detect_position_view().inroom_staircase();
            if stairs & 0x77 != 0 {
                let submodule = if stairs & 0x70 != 0 { 16 } else { 8 };
                self.set_submodule(submodule);
                self.set_main_module(7);
                self.link_cancel_dash();
            } else {
                const FEATURES0_TURN_WHILE_DASHING: u32 = 4;
                if self
                    .enhanced_features_view()
                    .has(FEATURES0_TURN_WHILE_DASHING)
                {
                    self.link_cancel_dash();
                }
            }
            if self.player_state_view().last_direction_moved_towards() & 2 == 0 {
                self.player_state_view_mut().arm_stair_speed_modifier();
                return;
            }
        }

        if self.finish_indoor_collision_shared_tail(true) {
            return;
        }
        self.handle_pushing_bonking_snaps_y();
    }

    fn finish_indoor_x_collision_tail(&mut self) {
        if self.tile_detect_position_view().horizontal_ledge() & 7 == 7
            && self.run_ledge_hop_timer()
        {
            self.link_cancel_dash();
            self.player_state_view_mut()
                .increment_about_to_jump_off_ledge();
            self.player_state_view_mut().set_auxiliary_state(2);
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_hop_in_or_out_of_water_x();
            self.ancilla_sfx2_near(0x20);
            return;
        }

        if self.finish_indoor_collision_shared_tail(false) {
            return;
        }
        if self.player_state_view().num_orthogonal_directions() == 0 {
            self.player_state_view_mut().clear_speed_modifier();
            if self.player_state_view().speed_setting() == 2 {
                self.player_state_view_mut().set_speed_setting(0);
            }
        }
        self.handle_pushing_bonking_snaps_x();
    }

    fn finish_indoor_collision_shared_tail(&mut self, y_axis: bool) -> bool {
        if y_axis {
            self.player_state_view_mut().resolve_dash_speed_setting();
            self.player_state_view_mut()
                .promote_pending_speed_modifier();
        }

        let r14 = self.tile_detect_position_view().collision_bits();
        if self.tile_detect_position_view().pit_tile() & 5 != 0 && r14 & 2 == 0 {
            self.start_falling_into_hole();
            return true;
        }

        if y_axis {
            self.player_state_view_mut().clear_pit_data_index();
        } else {
            self.player_state_view_mut().clear_near_pit_state();
        }
        if self.tile_detect_position_view().spike_cactus_tiles() & 7 != 0 {
            if (self.player_state_view().incapacitated_timer()
                | self.player_state_view().blink_countdown()
                | self.player_state_view().is_cape_active() as u8)
                == 0
            {
                let coord = if self.player_state_view().last_direction_moved_towards() & 2 == 0 {
                    self.player_state_view().y()
                } else {
                    self.player_state_view().x()
                };
                let low_phase = coord & 4 == 0;
                let damage = if self.player_state_view().last_direction_moved_towards() & 1 == 0 {
                    low_phase
                } else {
                    !low_phase
                };
                if damage {
                    self.player_state_view_mut().set_given_damage(8);
                    self.link_cancel_dash();
                    self.link_force_unequip_cape_quietly();
                    self.link_apply_tile_rebound();
                    return true;
                }
            } else {
                let spike_bits = (self.tile_detect_position_view().spike_cactus_tiles() & 7) as u16;
                self.tile_detect_position_view_mut()
                    .set_collision_bits(spike_bits);
            }
        }

        if self.dungeon_state_view().header_collision() == 0
            || self.dungeon_state_view().header_collision() == 4
            || !self.player_state_view().is_on_lower_level()
        {
            self.handle_indoor_pushblock_timeout(y_axis);
        }
        false
    }

    fn handle_indoor_pushblock_timeout(&mut self, y_axis: bool) {
        let block_flags = self.tile_detect_position_view().block_flags();
        if block_flags != 0 && self.player_state_view().num_orthogonal_directions() == 0 {
            self.tile_detect_position_view_mut()
                .set_staircase_cache(block_flags as u8);
            self.player_state_view_mut()
                .decrement_gravestone_push_timeout();
            if !(self.player_state_view().gravestone_push_timeout() as i8).is_negative() {
                return;
            }
            let mut bits = block_flags;
            for i in (0..=15).rev() {
                if bits & 0x8000 != 0 {
                    let idx = self.find_free_moving_block_slot(i);
                    if idx != 0xff {
                        let slot = idx as usize;
                        self.tile_detect_position_view_mut()
                            .set_collision_bits(idx as u16);
                        if !self.initialize_push_block(idx, (i * 2) as u8) {
                            self.sprite_dungeon_draw_single_push_block(slot * 2);
                            self.tile_detect_position_view_mut().set_collision_bits(4);
                            let facing =
                                self.player_state_view().last_direction_moved_towards() * 2;
                            self.pushed_block_view_mut().set_facing_player(slot, facing);
                            self.pushed_block_view_mut().set_push_direction(facing);
                            let target = if y_axis {
                                let y_lo = self.pushed_block_view().y_low(slot);
                                y_lo.wrapping_sub(u8::from(
                                    self.player_state_view().last_direction_moved_towards() == 1,
                                ))
                            } else {
                                let x_lo = self.pushed_block_view().x_low(slot);
                                x_lo.wrapping_sub(u8::from(
                                    self.player_state_view().last_direction_moved_towards() != 2,
                                ))
                            } & 0x0f;
                            self.pushed_block_view_mut().set_target_low(slot, target);
                        }
                    }
                }
                bits <<= 1;
            }
        }
        self.player_state_view_mut().set_gravestone_push_timeout(21);
    }

    fn dungeon_delete_rupee_tile_for_player(&mut self, x: u16, y: u16) {
        let pos = ((y & 0x01f8) * 8) | ((x & 0x01f8) >> 3);
        let dst = self.display_state().current_vram_upload_data_address();
        self.write_vram_upload_absolute_word(dst + 4, 0x190f);
        self.write_vram_upload_absolute_word(dst + 10, 0x190f);
        self.dungeon_state_view_mut()
            .set_bg2_tile(pos as usize, 0x190f);
        self.dungeon_state_view_mut()
            .set_bg2_tile((pos + 64) as usize, 0x190f);
        let attr = u16::from(self.player_tile_attributes().attr_for_tile(0x190f)) * 0x0101;
        let vram0 = self.Dungeon_MapVramAddr(pos);
        let vram1 = self.Dungeon_MapVramAddr(pos + 64);
        self.dungeon_state_view_mut()
            .set_bg2_attr_word(pos as usize, attr);
        self.dungeon_state_view_mut()
            .set_bg2_attr_word((pos + 64) as usize, attr);
        self.write_vram_upload_absolute_word(dst, vram0);
        self.write_vram_upload_absolute_word(dst + 6, vram1);
        self.write_vram_upload_absolute_word(dst + 2, 0x0100);
        self.write_vram_upload_absolute_word(dst + 8, 0x0100);
        self.write_vram_upload_absolute_word(dst + 12, 0xffff);
        self.advance_vram_upload_cursor_by(24);
        self.dungeon_state_view_mut()
            .set_savegame_state_high_bits(0x10);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn link_handle_liftables(&mut self) -> u8 {
        const ACTION_FOR_GLOVES: [u8; 7] = [0, 1, 0, 0, 2, 1, 2];
        const ACTION_FOR_TILE: [u8; 7] = [2, 3, 1, 4, 0, 5, 6];
        const ACTION_X: [i8; 4] = [7, 7, -3, 16];
        const ACTION_Y: [i8; 4] = [6, 24, 12, 12];

        self.tile_detect_position_view_mut().clear_pit_tile();
        self.tile_detect_reset_state();

        let facing = self.player_state_view().facing_index();
        let mask = self.tile_detect_position_view().location_calc_mask();
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let y0 = link_y.wrapping_add(ACTION_Y[facing] as i16 as u16) & mask;
        let y1 = link_y.wrapping_add(20) & mask;
        let x0 = (link_x.wrapping_add(ACTION_X[facing] as i16 as u16) & mask) >> 3;
        let x1 = (link_x.wrapping_add(8) & mask) >> 3;

        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);

        let mut action = if (self.tile_detect_position_view().collision_bits()
            | self.tile_detect_position_view().vertical_ledge() as u16)
            & 1
            != 0
        {
            3
        } else {
            2
        };

        if self.world_location_state().is_indoors() {
            let liftable = self.Dungeon_CheckForAndIDLiftableTile();
            if liftable != 0xffff {
                self.tile_detect_position_view_mut()
                    .set_liftable_action_index_primary(ACTION_FOR_TILE[(liftable & 0x0f) as usize]);
            } else {
                if self.tile_detect_position_view().read_something() & 1 != 0
                    && self.player_state_view().facing() == 0
                    && self.tile_detect_position_view().liftable_tile_index() == 0
                {
                    action = 4;
                }
                if self.tile_detect_position_view().chest() & 1 != 0 {
                    action = 5;
                }
                return action;
            }
        } else {
            if self.tile_detect_position_view().read_something() & 1 == 0 {
                if self.tile_detect_position_view().chest() & 1 != 0 {
                    action = 5;
                }
                return action;
            }
            if self.player_state_view().facing() == 0
                && self.tile_detect_position_view().liftable_tile_index() == 0
            {
                action = 4;
                if self.tile_detect_position_view().chest() & 1 != 0 {
                    action = 5;
                }
                return action;
            }
            let liftable_primary = self.tile_detect_position_view().liftable_tile_index() >> 1;
            self.tile_detect_position_view_mut()
                .set_liftable_action_index_primary(liftable_primary);
        }

        let liftable_index = self
            .tile_detect_position_view()
            .liftable_action_index_primary() as usize;
        if self.inventory_items().gloves() >= ACTION_FOR_GLOVES[liftable_index] {
            action = 1;
        }

        if self.tile_detect_position_view().chest() & 1 != 0 {
            action = 5;
        }
        action
    }

    pub(super) fn link_bonk_and_smash(&mut self) {
        if !self.player_state_view().is_running()
            || self.player_state_view().dash_counter() == 64
            || self.tile_detect_position_view().dashable_tiles() & 0x70 == 0
        {
            return;
        }
        const LIFTABLE_TILE_ATTR_TO_TERRAIN_TYPE: [u8; 9] =
            [0x54, 0x52, 0x50, 0xff, 0x51, 0x53, 0x55, 0x56, 0x57];
        for i in 0..2 {
            if let Some((j, x, y)) = self.overworld_smash_rock_pile_result(i != 0) {
                if let Some(k) = LIFTABLE_TILE_ATTR_TO_TERRAIN_TYPE
                    .iter()
                    .position(|&v| v == j)
                {
                    if k == 2 || k == 4 {
                        self.ancilla_sfx3_near(0x32);
                    }
                    self.sprite_spawn_immediately_smashed_terrain(k as u8, x, y);
                }
            }
        }
    }

    pub(super) fn overworld_smash_rock_pile_result(
        &mut self,
        down_one_tile: bool,
    ) -> Option<(u8, u16, u16)> {
        let bak = self.player_state_view().y();
        if down_one_tile {
            self.player_state_view_mut().set_y(bak.wrapping_add(8));
        }
        let (pos, x, y) = self.overworld_get_link_map16_coords_result();
        self.player_state_view_mut().set_y(bak);
        let a = self.dungeon_state_view().bg2_tile_by_byte_pos(pos);
        match a {
            0x226 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 0, x, y), x, y)),
            0x227 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 1, x, y), x, y)),
            0x228 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 2, x, y), x, y)),
            0x229 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 3, x, y), x, y)),
            0x36 => Some((
                self.overworld_lifting_small_obj_impl(a, pos, 0x0dc7, x, y),
                x,
                y,
            )),
            _ => None,
        }
    }

    pub(super) fn overworld_get_link_map16_coords_result(&self) -> (u16, u16, u16) {
        const X: [i16; 4] = [7, 7, -3, 16];
        const Y: [i16; 4] = [6, 24, 12, 12];
        let dir = self.player_state_view().facing_index();
        let x = self.player_state_view().x().wrapping_add(X[dir] as u16) & !0x0f;
        let y = self.player_state_view().y().wrapping_add(Y[dir] as u16) & !0x0f;
        let pos = ((y.wrapping_sub(self.world_state_view().overworld_offset_base_y())
            & self.world_state_view().overworld_offset_mask_y())
            << 3)
            .wrapping_add(
                ((x >> 3).wrapping_sub(self.world_state_view().overworld_offset_base_x()))
                    & self.world_state_view().overworld_offset_mask_x(),
            );
        (pos, x, y)
    }

    pub(super) fn overworld_lifting_small_obj_impl(
        &mut self,
        a: u16,
        pos: u16,
        mut tile: u16,
        x: u16,
        y: u16,
    ) -> u8 {
        let secret = self.overworld_reveal_secret_for_smash(pos);
        if secret != 0 {
            tile = secret;
        }
        self.dungeon_state_view_mut()
            .set_bg2_tile_by_byte_pos(pos, tile);
        self.overworld_memorize_map16_change_for_smash(pos, tile);
        self.overworld_draw_map16_for_smash(pos, tile);
        self.set_bg_vram_load_mode(1);
        self.map16_quadrant_attr(a, x, y)
    }

    pub(super) fn smash_rock_pile_from_lift_impl(
        &mut self,
        a: u16,
        pos: u16,
        quadrant: usize,
        x: u16,
        y: u16,
    ) -> u8 {
        const BIG_ROCK_MAP16_QUADRANT_OFFSETS: [i16; 4] = [0, -1, -64, -65];
        const BIG_ROCK_QUADRANT_Y_OFFSETS: [i16; 4] = [0, 0, -64, -64];
        const BIG_ROCK_QUADRANT_X_OFFSETS: [i16; 4] = [0, -1, 0, -1];
        let pos = 2 * ((pos >> 1).wrapping_add(BIG_ROCK_MAP16_QUADRANT_OFFSETS[quadrant] as u16));
        self.dungeon_state_view_mut()
            .set_big_rock_starting_address(pos);
        self.dungeon_state_view_mut().set_door_open_counter(40);
        let secret = self.overworld_reveal_secret_for_smash(pos);
        if secret == 0xffff {
            let screen = u16::from(self.world_location_state().overworld_screen_index()) as usize;
            self.overworld_event_info_view_mut()
                .set_event_bits(screen, 0x20);
            self.system_signals_view_mut().set_sound_effect_2(27);
            self.dungeon_state_view_mut().set_door_open_counter(80);
        }
        let x = x.wrapping_add((BIG_ROCK_QUADRANT_X_OFFSETS[quadrant] * 2) as u16);
        let y = y.wrapping_add((BIG_ROCK_QUADRANT_Y_OFFSETS[quadrant] * 2) as u16);
        self.overworld_do_map_update32x32_b_for_smash();
        self.map16_quadrant_attr(a, x, y)
    }

    pub(super) fn overworld_reveal_secret_for_smash(&mut self, pos: u16) -> u16 {
        self.dungeon_secret_scratch_view_mut().clear_pending_kind();

        let screen = u16::from(self.world_location_state().overworld_screen_index()) as usize;
        if screen >= 0x80 {
            self.adjust_secret_for_powder_for_smash();
            return 0;
        }

        let secret_offsets = self
            .asset_raw(157)
            .expect("overworld_reveal_secret_for_smash missing kOverworldSecrets_Offs asset")
            .to_vec();
        let secrets = self
            .asset_raw(158)
            .expect("overworld_reveal_secret_for_smash missing kOverworldSecrets asset")
            .to_vec();
        let ptr = u16::from(secret_offsets[screen * 2])
            | (u16::from(secret_offsets[screen * 2 + 1]) << 8);
        let mut ptr = ptr as usize;
        loop {
            let x = u16::from(secrets[ptr]) | (u16::from(secrets[ptr + 1]) << 8);
            if x == 0xffff {
                self.adjust_secret_for_powder_for_smash();
                return 0;
            }
            if x & 0x7fff == pos {
                break;
            }
            ptr += 3;
        }

        let data = secrets[ptr + 2];
        if data != 0 && data < 0x80 {
            self.dungeon_secret_scratch_view_mut().or_pending_kind(data);
        }
        if data < 0x80 {
            self.adjust_secret_for_powder_for_smash();
            return 0;
        }

        self.dungeon_secret_scratch_view_mut()
            .set_pending_kind(0xff);
        if data != 0x84 && self.overworld_event_info_view().event_info(screen) & 2 == 0 {
            if screen == 0x5b && self.follower_state_view().indicator() != 13 {
                self.adjust_secret_for_powder_for_smash();
                return 0;
            }
            self.system_signals_view_mut().set_sound_effect_2(0x1b);
        } else if data == 0x82 && self.enhanced_features_view().has(4096) {
            self.system_signals_view_mut().set_sound_effect_2(0x1b);
        }

        const TILE_BELOW: [u16; 4] = [0x0dcc, 0x0212, 0xffff, 0x0db4];
        self.adjust_secret_for_powder_for_smash();
        TILE_BELOW[((data & 0x0f) >> 1) as usize]
    }

    fn adjust_secret_for_powder_for_smash(&mut self) {
        if self.player_state_view().item_in_hand_has(0x40) {
            self.dungeon_secret_scratch_view_mut()
                .set_powder_pending_kind();
        }
    }

    pub(super) fn overworld_memorize_map16_change_for_smash(&mut self, pos: u16, value: u16) {
        if value == 0x0dc5 || value == 0x0dc9 {
            return;
        }
        let x = self.memorized_tile_view().count() as usize;
        self.memorized_tile_view_mut().set_entry_value(x, value);
        self.memorized_tile_view_mut().set_entry_addr(x, pos);
        self.memorized_tile_view_mut().set_count((x + 2) as u16);
    }

    pub(super) fn overworld_draw_map16_for_smash(&mut self, pos: u16, value: u16) {
        let vram_pos = self.overworld_find_map16_vram_address_for_smash(pos);
        let dst = self.display_state().current_vram_upload_data_address();
        let src = value as usize * 4;
        let map8 = self
            .asset_raw(70)
            .expect("overworld_draw_map16_for_smash missing kMap16ToMap8 asset");
        let tile0 = u16::from(map8[src * 2]) | (u16::from(map8[src * 2 + 1]) << 8);
        let tile1 = u16::from(map8[(src + 1) * 2]) | (u16::from(map8[(src + 1) * 2 + 1]) << 8);
        let tile2 = u16::from(map8[(src + 2) * 2]) | (u16::from(map8[(src + 2) * 2 + 1]) << 8);
        let tile3 = u16::from(map8[(src + 3) * 2]) | (u16::from(map8[(src + 3) * 2 + 1]) << 8);
        self.write_vram_upload_map16_update_packet(dst, vram_pos, [tile0, tile1, tile2, tile3]);
        self.advance_vram_upload_cursor_by(16);
    }

    fn overworld_find_map16_vram_address_for_smash(&self, addr: u16) -> u16 {
        (if addr & 0x3f >= 0x20 { 0x0400 } else { 0 })
            + (if addr & 0x0fff >= 0x0800 { 0x0800 } else { 0 })
            + (addr & 0x001f)
            + ((addr & 0x0780) >> 1)
    }

    fn overworld_do_map_update32x32_b_for_smash(&mut self) {
        self.overworld_do_map_update32x32_for_smash();
        self.dungeon_state_view_mut().clear_door_open_counter_low();
    }

    fn overworld_do_map_update32x32_for_smash(&mut self) {
        const DOOR_ANIM_TILES: [u16; 56] = [
            0x0da8, 0x0da9, 0x0daa, 0x0dab, 0x0dac, 0x0dad, 0x0dae, 0x0daf, 0x0db0, 0x0db1, 0x0db2,
            0x0db3, 0x0db6, 0x0db7, 0x0db8, 0x0db9, 0x0dba, 0x0dbb, 0x0dbc, 0x0dbd, 0x0dcd, 0x0dce,
            0x0dcf, 0x0dd0, 0x0dd3, 0x0dd4, 0x0dd5, 0x0dd6, 0x0dd7, 0x0dd8, 0x0dd9, 0x0dda, 0x0dd1,
            0x0dd2, 0x0dd3, 0x0dd4, 0x0dd1, 0x0dd2, 0x0dd7, 0x0dd8, 0x0918, 0x0919, 0x091a, 0x091b,
            0x0ddb, 0x0ddc, 0x0ddd, 0x0dde, 0x0dd1, 0x0dd2, 0x0ddb, 0x0ddc, 0x0e21, 0x0e22, 0x0e23,
            0x0e24,
        ];
        let i = self.memorized_tile_view().count() as usize;
        let j = (self.dungeon_state_view().door_open_counter() >> 1) as usize;
        let base = self.dungeon_state_view().big_rock_starting_address();
        let entries = [
            (base, DOOR_ANIM_TILES[j]),
            (base.wrapping_add(2), DOOR_ANIM_TILES[j + 1]),
            (base.wrapping_add(0x80), DOOR_ANIM_TILES[j + 2]),
            (base.wrapping_add(0x82), DOOR_ANIM_TILES[j + 3]),
        ];
        for (n, (pos, tile)) in entries.into_iter().enumerate() {
            self.memorized_tile_view_mut()
                .set_entry_addr(i + n * 2, pos);
            self.memorized_tile_view_mut()
                .set_entry_value(i + n * 2, tile);
            self.overworld_draw_map16_persist_for_smash(pos, tile);
        }
        let upload = self.display_state().vram_upload_cursor_usize();
        self.write_vram_upload_buffer_word(upload, 0xffff);
        self.memorized_tile_view_mut().set_count((i + 8) as u16);
        let step = self
            .dungeon_state_view()
            .door_animation_step()
            .wrapping_add(if self.dungeon_state_view().door_open_counter() == 32 {
                2
            } else {
                1
            });
        self.dungeon_state_view_mut().set_door_animation_step(step);
        self.set_bg_vram_load_mode(1);
        self.dungeon_state_view_mut()
            .increment_door_open_counter_low();
    }

    fn overworld_draw_map16_persist_for_smash(&mut self, pos: u16, value: u16) {
        self.dungeon_state_view_mut()
            .set_bg2_tile_by_byte_pos(pos, value);
        self.overworld_draw_map16_for_smash(pos, value);
    }

    fn map16_quadrant_attr(&self, map16: u16, x: u16, y: u16) -> u8 {
        let index = map16 as usize * 4 + usize::from(x & 8 != 0) * 2 + usize::from(y & 8 != 0);
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("map16_quadrant_attr missing kMap16ToMap8 asset");
        let tile_attrs = self
            .asset_raw(163)
            .expect("map16_quadrant_attr missing kMap8DataToTileAttr asset");
        let map8 = read_word_from_slice(map16_to_map8, index * 2);
        tile_attrs[(map8 & 0x01ff) as usize]
    }

    pub(super) fn link_handle_diagonal_collision(&mut self) {
        if self.check_if_room_needs_double_layer_check() {
            self.player_limit_directions_inner();
            self.create_velocity_from_moving_background();
        }
        self.player_state_view_mut().mask_direction(0x0f);
        self.player_limit_directions_inner();
    }

    pub(super) fn link_handle_diagonal_kickback(&mut self) {
        if self.player_state_view().x_velocity() == 0 || self.player_state_view().y_velocity() == 0
        {
            self.player_state_view_mut()
                .set_moving_against_diag_deadlocked(0);
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
            return;
        }

        self.player_state_view_mut()
            .cache_copied_position_from_current();

        self.tile_detect_movement_x(
            if self.player_state_view().x_velocity_signed().is_negative() {
                2
            } else {
                3
            },
        );
        if self.tile_detect_position_view().slope_collision_bits() & 5 == 0 {
            self.player_state_view_mut()
                .set_moving_against_diag_deadlocked(0);
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
            return;
        }
        self.flag_moving_into_slopes_x();
        if self.player_state_view().moving_against_diag_tile() & 0x0f == 0 {
            self.player_state_view_mut()
                .set_moving_against_diag_deadlocked(0);
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
            return;
        }

        let xd = self
            .player_state_view()
            .x()
            .wrapping_sub(self.player_state_view().copied_x()) as u8;
        let copied_x = self.player_state_view().copied_x();
        self.player_state_view_mut().set_x(copied_x);
        self.player_state_view_mut().set_x_velocity(xd);

        self.tile_detect_movement_y(
            if self.player_state_view().y_velocity_signed().is_negative() {
                0
            } else {
                1
            },
        );
        if self.tile_detect_position_view().slope_collision_bits() & 5 == 0 {
            self.player_state_view_mut()
                .set_moving_against_diag_deadlocked(0);
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
            return;
        }
        self.flag_moving_into_slopes_y();
        if self.player_state_view().moving_against_diag_tile() & 0x0f == 0 {
            self.player_state_view_mut()
                .set_moving_against_diag_deadlocked(0);
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
            return;
        }

        let diag_tile = self.player_state_view().moving_against_diag_tile();
        self.player_state_view_mut()
            .set_moving_against_diag_deadlocked(diag_tile);
        let yd = self
            .player_state_view()
            .y()
            .wrapping_sub(self.player_state_view().copied_y()) as u8;
        self.player_state_view_mut().set_y_velocity(yd);

        const X0: [i8; 10] = [0, 1, 1, 1, 2, 2, 2, 3, 3, 3];
        const X1: [i8; 10] = [0, -1, -1, -1, -2, -2, -2, -3, -3, -3];
        let x_vel = self.player_state_view().x_velocity_signed();
        let x_idx = x_vel.unsigned_abs() as usize;
        let x_delta = if x_vel < 0 { X1[x_idx] } else { X0[x_idx] };
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(x_delta as i16 as u16);
        self.player_state_view_mut().set_x(x);

        const Y0: [i8; 10] = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3];
        const Y1: [i8; 16] = [0, 1, 1, 2, 2, 2, 3, 3, 3, 3, -91, 48, -16, 4, -91, 49];
        let y_vel = self.player_state_view().y_velocity_signed();
        let y_idx = y_vel.unsigned_abs() as usize;
        let y_delta = if y_vel < 0 { Y1[y_idx] } else { Y0[y_idx] };
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(y_delta as i16 as u16);
        self.player_state_view_mut().set_y(y);

        self.player_state_view_mut()
            .clear_moving_against_diag_tile();
    }

    pub(super) fn link_handle_cardinal_collision(&mut self) {
        self.replay_trace_submodule("cardinal-entry");
        self.tile_detect_position_view_mut().clear_diag_state();
        self.tile_detect_position_view_mut().clear_diagonal_tile();

        let can_double_layer = if self.player_state_view().moving_against_diag_tile() & 0x30 != 0 {
            true
        } else {
            self.link_handle_diagonal_kickback();
            self.player_state_view().moving_against_diag_deadlocked() == 0
        };

        if can_double_layer && self.check_if_room_needs_double_layer_check() {
            if self.dungeon_state_view().header_collision() >= 2
                && self.dungeon_state_view().header_collision() != 3
            {
                self.player_state_view_mut().set_tile_coll_flag(2);
                self.player_tile_detect_nearby();
                let collision_bits = self.tile_detect_position_view().collision_bits() as u8;
                self.tile_detect_position_view_mut()
                    .set_tile_collision_bits_primary(collision_bits);
                if self
                    .tile_detect_position_view()
                    .tile_collision_bits_primary()
                    != 0
                {
                    let floor_x_velocity = self.dungeon_state_view().floor_x_velocity_low() as u16;
                    let floor_y_velocity = self.dungeon_state_view().floor_y_velocity_low() as u16;
                    self.player_state_view_mut()
                        .add_movement_velocity_delta(floor_x_velocity, floor_y_velocity);

                    let a = self.tile_detect_position_view().collision_bits() as u8;
                    let horizontal_first = if a == 12 || a == 3 {
                        false
                    } else if a == 10 || a == 5 {
                        true
                    } else if (a & 0x0c) == 0 && (a & 3) == 0 {
                        false
                    } else if self.player_state_view().y_velocity() != 0 {
                        true
                    } else if self.player_state_view().x_velocity() == 0 {
                        false
                    } else {
                        (self.dungeon_state_view().floor_y_velocity_low() as i8) >= 0
                    };
                    if horizontal_first {
                        self.run_slope_collision_checks_horizontal_first();
                    } else {
                        self.run_slope_collision_checks_vertical_first();
                    }
                } else {
                    self.run_slope_collision_checks_vertical_first();
                }
            } else {
                self.run_slope_collision_checks_vertical_first();
            }
            self.create_velocity_from_moving_background();
        }

        let collision = self.dungeon_state_view().header_collision();
        let moved =
            (self.player_state_view().x_velocity() | self.player_state_view().y_velocity()) != 0;
        if collision == 2 {
            self.player_tile_detect_nearby();
            if (self.tile_detect_position_view().collision_bits() as u8
                | self
                    .tile_detect_position_view()
                    .tile_collision_bits_primary())
                == 0x0f
            {
                if self.player_state_view().blink_countdown() == 0 {
                    self.player_state_view_mut().set_blink_countdown(58);
                }
                if self.player_state_view().direction() == 0 {
                    if self.dungeon_state_view().floor_y_velocity_low() != 0 {
                        let y_velocity = self.player_state_view().y_velocity();
                        self.player_state_view_mut()
                            .set_y_velocity((0u8).wrapping_sub(y_velocity));
                    }
                    if self.dungeon_state_view().floor_x_velocity_low() != 0 {
                        let x_velocity = self.player_state_view().x_velocity();
                        self.player_state_view_mut()
                            .set_x_velocity((0u8).wrapping_sub(x_velocity));
                    }
                }
            }
            self.player_state_view_mut().set_tile_coll_flag(1);
            self.run_slope_collision_checks_vertical_first();
        } else if collision == 3 {
            self.player_state_view_mut().set_tile_coll_flag(1);
            self.run_slope_collision_checks_horizontal_first();
        } else if collision == 4 || moved {
            self.player_state_view_mut().set_tile_coll_flag(1);
            self.run_slope_collision_checks_vertical_first();
        } else if !self
            .player_state_view()
            .is_edge_transition_blocked_by_handler_state()
            && self.player_state_view().handler_state() != 19
        {
            self.player_tile_detect_nearby();
            if self.tile_detect_position_view().pit_tile() & 0x0f != 0 {
                self.player_state_view_mut().set_handler_state(1);
                if !self.player_state_view().is_running() {
                    self.player_state_view_mut().set_speed_setting(4);
                }
            }
        }

        self.tile_detect_main_handler(0);
        self.replay_trace_submodule("cardinal-after-tile-main");
        if self.player_state_view().num_orthogonal_directions() != 0 {
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
        }

        self.player_state_view_mut()
            .refresh_direction_from_safe_return_delta();
        self.replay_trace_submodule("cardinal-after-dir-vel");

        if self.world_location_state().is_outdoors()
            || self.dungeon_state_view().header_collision() != 4
            || self.player_state_view().handler_state() != 4
        {
            return;
        }

        if self.dungeon_state_view().floor_y_velocity_low() != 0
            && self
                .player_state_view()
                .y_velocity()
                .wrapping_sub(self.dungeon_state_view().floor_y_velocity_low())
                == 0
        {
            if (self.dungeon_state_view().floor_y_velocity_low() as i8).is_negative() {
                self.player_state_view_mut().clear_direction_flags(8);
            } else {
                self.player_state_view_mut().clear_direction_flags(4);
            }
        }
        if self.dungeon_state_view().floor_x_velocity_low() != 0
            && self
                .player_state_view()
                .x_velocity()
                .wrapping_sub(self.dungeon_state_view().floor_x_velocity_low())
                == 0
        {
            if (self.dungeon_state_view().floor_x_velocity_low() as i8).is_negative() {
                self.player_state_view_mut().clear_direction_flags(2);
            } else {
                self.player_state_view_mut().clear_direction_flags(1);
            }
        }
    }

    pub(super) fn link_state_recoil(&mut self) {
        self.replay_trace_player_state("recoil-entry");
        let old_x = self.player_state_view().x();
        let old_y = self.player_state_view().y();
        self.store_link_safe_return_position(old_x, old_y);

        self.link_handle_change_in_z_velocity();
        self.player_state_view_mut().clear_direction_lock();
        self.player_state_view_mut()
            .clear_water_ripple_or_grass_state();

        if self.player_state_view().is_z_low_negative()
            && (self.player_state_view().actual_z_velocity() as i8).is_negative()
        {
            self.tile_detect_main_handler(5);
            if self.tile_detect_position_view().deepwater() & 1 != 0 {
                self.player_state_view_mut().set_handler_state(4);
                self.link_set_to_deep_water();
                self.link_reset_sword_and_item_usage();
                self.ancilla_add_splash(21, 0);
                self.link_handle_recoil_and_timer(true);
            } else {
                let recoil_timer = self.player_state_view_mut().increment_recoil_timer();
                if recoil_timer != 4 {
                    let mut z = self.player_state_view().actual_z_velocity_copy();
                    let mut s = recoil_timer;
                    loop {
                        z >>= 1;
                        s = s.wrapping_sub(1);
                        if s != 0 {
                            break;
                        }
                    }
                    self.player_state_view_mut().set_actual_z_velocity(z);
                } else {
                    self.player_state_view_mut().set_recoil_timer(3);
                }
                self.link_handle_recoil_and_timer(false);
            }
        } else {
            self.link_handle_recoil_and_timer(false);
        }
        self.player_state_view_mut().clear_z_high();
        self.replay_trace_player_state("recoil-exit");
    }

    pub(super) fn link_state_sleeping(&mut self) {
        match self.player_state_view().sleep_in_bed_state() {
            0 => {
                if self.frame_state().frame_counter & 0x1f == 0 {
                    self.ancilla_add_snoring(0x21, 1);
                }
            }
            1 => {
                if self.frame_state().submodule == 0 {
                    if (self.player_state_view_mut().decrement_dash_countdown() as i8).is_negative()
                    {
                        self.player_state_view_mut().set_dash_countdown(0);
                        let input = (self.player_state_view().filtered_joypad_h() & 0xe0)
                            | (self.player_state_view().filtered_joypad_h() << 4)
                            | self.player_state_view().filtered_joypad_l();
                        if input & 0xf0 != 0 {
                            self.player_state_view_mut().increment_opening_pose();
                            self.player_state_view_mut().set_facing(6);
                            self.player_state_view_mut().increment_sleep_in_bed_state();
                            self.player_state_view_mut().set_dash_countdown(4);
                        }
                    }
                }
            }
            2 => {
                if (self.player_state_view_mut().decrement_dash_countdown() as i8).is_negative() {
                    self.player_state_view_mut().set_actual_velocity_xy(21, 4);
                    self.player_state_view_mut()
                        .set_actual_z_velocity_and_copy(24);
                    self.player_state_view_mut().set_incapacitated_timer(16);
                    self.player_state_view_mut().set_auxiliary_state(2);
                    self.player_state_view_mut().set_handler_state(6);
                }
            }
            _ => {}
        }
    }

    pub(super) fn link_state_zapped(&mut self) {
        self.cache_camera_properties_if_outdoors();
        self.LinkZap_HandleMosaic();

        let delay = self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer();
        if !(delay as i8).is_negative() {
            return;
        }

        self.player_state_view_mut().set_spin_attack_delay_timer(2);
        self.player_state_view_mut()
            .increment_action_handler_timer();
        if self.player_state_view().action_handler_timer() & 1 != 0 {
            self.palette_electro_themed_gear();
        } else {
            self.load_actual_gear_palettes();
        }
        if self.player_state_view().action_handler_timer() == 8 {
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().clear_handler_state();
            self.player_state_view_mut()
                .clear_sprite_damage_disable_timer();
            self.player_state_view_mut().clear_electrocute_on_touch();
            self.player_state_view_mut().clear_auxiliary_state();
            self.Player_SetCustomMosaicLevel(0);
        }
    }

    pub(super) fn link_state_exiting_dash(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.player_state_view().joypad1h_last() & 0x0f != 0
            || self.player_state_view().dash_countdown() >= 16
        {
            self.player_state_view_mut().set_dash_countdown(0);
            self.player_state_view_mut().set_speed_setting(0);
            self.player_state_view_mut().clear_handler_state();
            self.player_state_view_mut().clear_running();
            self.swim_acceleration_view_mut().set_mode(0, 0);
            if self.player_state_view().button_b_frames() < 9 {
                self.player_state_view_mut().clear_direction_lock();
            }
        } else {
            self.player_state_view_mut().increment_dash_countdown();
        }
        self.link_handle_moving_animation_full_long_entry();
    }

    pub(super) fn reset_all_acceleration(&mut self) {
        self.swim_acceleration_view_mut().clear_axis_motion(0);
        self.swim_acceleration_view_mut().clear_axis_motion(2);
        for offset in [0, 2] {
            self.player_state_view_mut()
                .set_swim_stroke_frame_counter(offset, 0);
        }
    }

    pub(super) fn link_force_unequip_cape_quietly(&mut self) {
        self.player_state_view_mut().set_cape_transform_timer(32);
        self.player_state_view_mut()
            .clear_sprite_damage_disable_timer();
        self.player_state_view_mut().set_cape_mode(0);
        self.player_state_view_mut().clear_electrocute_on_touch();
    }

    pub(super) fn link_force_unequip_cape(&mut self) {
        self.ancilla_add_cape_poof(35, 4);
        self.ancilla_sfx2_near(21);
        self.link_force_unequip_cape_quietly();
    }

    pub(super) fn halt_link_when_using_items(&mut self) {
        if self.dungeon_state_view().header_collision_2() == 2
            && self.has_player_layer_collision(
                crate::game_state::constants::player::LAYER_COLLISION_BOTH,
            )
        {
            self.player_state_view_mut()
                .clear_movement_velocity_and_direction();
            self.player_state_view_mut().clear_movement_subpixels();
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
        }
        if self.player_state_view().has_somaria_platform_state() {
            self.player_state_view_mut().set_direction(0);
        }
    }

    pub(super) fn link_handle_cape_passive_lift_check(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.player_state_view().is_lifting_or_carrying()
            || (self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES)
                && self.player_state_view().has_grabbing_wall_state())
        {
            self.player_check_handle_cape_stuff();
        }
    }

    pub(super) fn player_check_handle_cape_stuff(&mut self) {
        const CAPE_DEPLETION_TIMERS: [u8; 3] = [4, 8, 8];
        if !self.player_state_view().is_cape_active()
            || self.player_state_view().current_item_active() != 19
        {
            return;
        }
        if self.player_state_view().current_item_active()
            == self.player_state_view().current_item_y()
        {
            self.player_state_view_mut()
                .decrement_cape_decrement_counter();
            if self.player_state_view().cape_decrement_counter() != 0 {
                return;
            }
            let cape_timer =
                CAPE_DEPLETION_TIMERS[self.player_state_view().magic_consumption_level() as usize];
            self.player_state_view_mut()
                .set_cape_decrement_counter(cape_timer);
            if self.player_state_view().magic_power() == 0 {
                return;
            }
            if self.player_state_view_mut().decrement_magic_power() != 0 {
                return;
            }
        }
        self.link_force_unequip_cape();
    }

    pub(super) fn check_y_button_press(&mut self) -> bool {
        if self.player_state_view().button_mask_b_y() & 0x40 != 0
            || self.player_state_view().incapacitated_timer() != 0
            || self.player_state_view().filtered_joypad_h() & 0x40 == 0
        {
            return false;
        }
        self.player_state_view_mut().add_button_mask_b_y_bits(0x40);
        true
    }

    pub(super) fn link_check_magic_cost(&mut self, item: u8) -> bool {
        const LINK_ITEM_MAGIC_COSTS: [u8; 27] = [
            16, 8, 4, 32, 16, 8, 8, 4, 2, 8, 4, 2, 8, 4, 2, 16, 8, 4, 4, 2, 2, 8, 4, 2, 16, 8, 4,
        ];
        let idx = item as usize * 3 + self.player_state_view().magic_consumption_level() as usize;
        let cost = LINK_ITEM_MAGIC_COSTS[idx];
        if self.player_state_view_mut().spend_magic(cost) {
            return true;
        }
        if item != 3 {
            self.ancilla_sfx2_near(60);
            self.dialogue_message_index_view_mut().set_value(123);
            self.main_show_text_message();
        }
        false
    }

    pub(super) fn refund_magic(&mut self, item: u8) {
        const LINK_ITEM_MAGIC_COSTS: [u8; 27] = [
            16, 8, 4, 32, 16, 8, 8, 4, 2, 8, 4, 2, 8, 4, 2, 16, 8, 4, 4, 2, 2, 8, 4, 2, 16, 8, 4,
        ];
        let idx = item as usize * 3 + self.player_state_view().magic_consumption_level() as usize;
        let cost = LINK_ITEM_MAGIC_COSTS[idx];
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        let clamp_full = self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES);
        self.player_state_view_mut().refund_magic(cost, clamp_full);
    }

    pub(super) fn link_item_reset_from_overworld_things(&mut self) {
        self.player_state_view_mut().set_y_button_action_step(0);
        self.player_state_view_mut().set_y_button_action_flags(0);
        self.player_state_view_mut()
            .clear_state_item_and_grab_flags();
        self.player_state_view_mut().clear_direction_lock_bits(1);
    }

    pub(super) fn link_item_cape(&mut self) {
        const CAPE_DEPLETION_TIMERS: [u8; 3] = [4, 8, 8];
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        if !self.player_state_view().is_cape_active() {
            if (self.player_state_view_mut().tick_cape_transform_timer() as i8) >= 0 {
                self.player_state_view_mut().clear_direction_flags(0x0f);
                self.halt_link_when_using_items();
                return;
            }

            self.player_state_view_mut().clear_cape_transform_timer();
            if self.player_state_view().doorway_state() != 0 || !self.check_y_button_press() {
                return;
            }
            self.player_state_view_mut()
                .clear_button_mask_b_y_bits(0x40);
            if self.player_state_view().magic_power() == 0 {
                self.ancilla_sfx2_near(60);
                self.dialogue_message_index_view_mut().set_value(123);
                self.main_show_text_message();
                return;
            }

            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_cape_mode(1);
            let cape_timer =
                CAPE_DEPLETION_TIMERS[self.player_state_view().magic_consumption_level() as usize];
            self.player_state_view_mut()
                .set_cape_decrement_counter(cape_timer);
            self.player_state_view_mut().set_cape_transform_timer(20);
            self.ancilla_add_cape_poof(35, 4);
            self.ancilla_sfx2_near(20);
            return;
        }

        self.player_state_view_mut()
            .set_sprite_damage_disable_timer(1);
        self.halt_link_when_using_items();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        self.player_state_view_mut()
            .decrement_cape_decrement_counter();
        if self.player_state_view().cape_decrement_counter() == 0 {
            let cape_timer =
                CAPE_DEPLETION_TIMERS[self.player_state_view().magic_consumption_level() as usize];
            self.player_state_view_mut()
                .set_cape_decrement_counter(cape_timer);
            if self.player_state_view().magic_power() == 0
                && self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES)
            {
                self.link_force_unequip_cape();
                return;
            }
            if self.player_state_view_mut().decrement_magic_power() == 0 {
                self.link_force_unequip_cape();
                return;
            }
        }

        if (self.player_state_view_mut().tick_cape_transform_timer() as i8) < 0 {
            self.player_state_view_mut().clear_cape_transform_timer();
            if self.player_state_view().filtered_joypad_h() & 0x40 != 0 {
                self.link_force_unequip_cape();
            }
        }
    }

    pub(super) fn link_item_rod(&mut self) {
        const ROD_ANIM_DELAYS: [u8; 3] = [3, 3, 5];
        if self.player_state_view().button_mask_b_y() & 0x40 == 0 {
            if self.player_state_view().doorway_state() != 0 || !self.check_y_button_press() {
                return;
            }
            if !self.link_check_magic_cost(0) {
                self.player_state_view_mut()
                    .clear_button_mask_b_y_bits(0x40);
                return;
            }
            self.player_state_view_mut()
                .set_item_action_debug_value_2(1);
            if self.player_state_view().selected_rod() == 1 {
                self.ancilla_add_fire_rod_shot(2, 1);
            } else {
                self.ancilla_add_ice_rod_shot(11, 1);
            }
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(ROD_ANIM_DELAYS[0]);
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_item_in_hand(1);
        }
        self.halt_link_when_using_items();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        self.player_state_view_mut()
            .increment_action_handler_timer();
        let step = self.player_state_view().action_handler_timer() as usize;
        if step < ROD_ANIM_DELAYS.len() {
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(ROD_ANIM_DELAYS[step]);
            return;
        }
        self.player_state_view_mut()
            .set_item_action_debug_value_2(0);
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().clear_action_handler_timer();
        self.player_state_view_mut().set_spin_attack_delay_timer(0);
        self.player_state_view_mut().clear_item_in_hand_bits(1);
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
    }

    pub(super) fn link_item_hammer(&mut self) {
        const HAMMER_ANIM_DELAYS: [u8; 3] = [3, 3, 16];
        if self.player_state_view().item_in_hand_has(0x10) {
            return;
        }
        if self.player_state_view().button_mask_b_y() & 0x40 == 0 {
            if self.player_state_view().doorway_state() != 0
                || self.player_state_view().filtered_joypad_h() & 0x40 == 0
            {
                return;
            }
            self.player_state_view_mut().add_button_mask_b_y_bits(0x40);
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(HAMMER_ANIM_DELAYS[0]);
            self.player_state_view_mut().set_direction_lock_bits(1);
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_item_in_hand(2);
        }
        self.halt_link_when_using_items();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        self.player_state_view_mut()
            .increment_action_handler_timer();
        let step = self.player_state_view().action_handler_timer() as usize;
        self.player_state_view_mut().set_spin_attack_delay_timer(
            HAMMER_ANIM_DELAYS[step.min(HAMMER_ANIM_DELAYS.len() - 1)],
        );
        if self.player_state_view().action_handler_timer() == 1 {
            self.tile_detect_main_handler(3);
            self.ancilla_add_hit_stars(22, 0);
            if self.system_signals_view().sound_effect_1() == 0 {
                self.ancilla_sfx2_near(16);
                self.spawn_hammer_water_splash();
            }
        } else if self.player_state_view().action_handler_timer() == 3 {
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_spin_attack_delay_timer(0);
            self.player_state_view_mut()
                .clear_button_mask_b_y_bits(0x40);
            self.player_state_view_mut().clear_direction_lock_bits(1);
            self.player_state_view_mut().clear_item_in_hand_bits(2);
        }
    }

    pub(super) fn link_item_bow(&mut self) {
        const BOW_DELAYS: [u8; 3] = [3, 3, 8];
        if self.player_state_view().button_mask_b_y() & 0x40 == 0 {
            if self.player_state_view().doorway_state() != 0 || !self.check_y_button_press() {
                return;
            }
            self.player_state_view_mut().set_direction_lock_bits(1);
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(BOW_DELAYS[0]);
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_item_in_hand(16);
        }
        self.halt_link_when_using_items();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        self.player_state_view_mut()
            .increment_action_handler_timer();
        let step = self.player_state_view().action_handler_timer() as usize;
        if step < BOW_DELAYS.len() {
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(BOW_DELAYS[step]);
            return;
        }

        let k = self.ancilla_add_arrow(
            9,
            self.player_state_view().facing(),
            2,
            self.player_state_view().x(),
            self.player_state_view().y(),
        );
        if k >= 0 {
            let k = k as usize;
            if self.archery_game_view().arrows_left() != 0 {
                self.archery_game_view_mut().decrement_arrows_left();
                self.player_resources_view_mut().increment_arrows_by(2);
            }
            if self.archery_game_view().out_of_arrows() == 0
                && self.player_resources_view().arrows() != 0
            {
                if self.player_resources_view_mut().decrement_arrows() == 0 {
                    self.hud_refresh_icon();
                }
            } else {
                self.ancilla_slot_view_mut(k).set_ancilla_type(0);
                self.ancilla_sfx2_near(60);
            }
        }

        self.player_state_view_mut().clear_action_handler_timer();
        self.player_state_view_mut().set_spin_attack_delay_timer(0);
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.player_state_view_mut().clear_direction_lock_bits(1);
        self.player_state_view_mut().clear_item_in_hand_bits(0x10);
        if self.player_state_view().button_b_frames() >= 9 {
            self.player_state_view_mut().set_button_b_frames(9);
        }
    }

    pub(super) fn link_item_boomerang(&mut self) {
        if self.player_state_view().button_mask_b_y() & 0x40 == 0 {
            if self.player_state_view().doorway_state() != 0
                || !self.check_y_button_press()
                || self.minigame_state_view().flag_boomerang_in_place() != 0
            {
                return;
            }
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().set_item_in_hand(0x80);
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_spin_attack_delay_timer(7);
            let s0 = self.ancilla_add_boomerang(5, 0);
            if self.player_state_view().button_b_frames() >= 9 {
                self.link_reset_boomerang_y_stuff();
                return;
            }
            if s0 == 0 {
                let last_direction = self.player_state_view().joypad1h_last() & 0x0f;
                self.player_state_view_mut()
                    .set_last_direction(last_direction);
            } else {
                self.player_state_view_mut().set_direction_lock_bits(1);
            }
        } else {
            self.player_state_view_mut().set_direction_lock_bits(1);
        }

        if self.player_state_view().has_item_in_hand() {
            self.halt_link_when_using_items();
            self.player_state_view_mut().clear_direction_flags(0x0f);
            if (self
                .player_state_view_mut()
                .decrement_spin_attack_delay_timer() as i8)
                >= 0
            {
                return;
            }
            self.player_state_view_mut().set_spin_attack_delay_timer(5);
            self.player_state_view_mut()
                .increment_action_handler_timer();
            if self.player_state_view().action_handler_timer() != 2 {
                return;
            }
        }
        self.link_reset_boomerang_y_stuff();
    }

    pub(super) fn link_reset_boomerang_y_stuff(&mut self) {
        self.player_state_view_mut().clear_item_in_hand();
        self.player_state_view_mut().clear_action_handler_timer();
        self.player_state_view_mut().set_spin_attack_delay_timer(0);
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
        if self.player_state_view().button_mask_b_y() & 0x80 == 0 {
            self.player_state_view_mut().clear_direction_lock_bits(1);
        }
    }

    pub(super) fn link_handle_a_press(&mut self) {
        self.player_state_view_mut()
            .set_sprite_pickup_flag_cached(0);
        if self.player_state_view().has_item_in_hand()
            || self.player_state_view().position_mode_has(0x1f)
            || self.player_state_view().player_pose_draw_counter() != 0
        {
            return;
        }
        if self.player_state_view().button_b_frames() < 9
            && (self.player_state_view().button_mask_b_y() & 0x80) != 0
        {
            return;
        }

        let mut action = self.player_state_view().tile_action_index();
        if !self.player_state_view().has_action_state()
            && !self.player_state_view().has_grabbing_wall_state()
        {
            if !self.link_check_new_a_press() {
                self.player_state_view_mut().set_y_button_action_flags(0);
                return;
            }
            if self.player_state_view().needs_pull_for_rupees_sprite()
                && self.player_state_view().facing() == 0
            {
                action = 7;
            } else if self.player_state_view().is_near_moveable_statue() {
                action = 6;
            } else {
                let mut attempt_action = false;
                if self.player_state_view().ancilla_pickup_flag() == 0 {
                    if self.player_state_view().sprite_pickup_flag() == 0 {
                        action = self.link_handle_liftables();
                        attempt_action = true;
                    } else {
                        let pickup_flag = self.player_state_view().sprite_pickup_flag();
                        self.player_state_view_mut()
                            .set_sprite_pickup_flag_cached(pickup_flag);
                    }
                }
                if !attempt_action {
                    if self.player_state_view().button_b_frames() != 0 {
                        self.link_reset_sword_and_item_usage();
                    }
                    if self.player_state_view().has_item_or_position_mode() {
                        self.player_state_view_mut().clear_item_in_hand();
                        self.player_state_view_mut().clear_position_mode();
                        self.link_reset_boomerang_y_stuff();
                        self.minigame_state_view_mut()
                            .clear_flag_boomerang_in_place();
                        if self.ancilla_slot_view(0).ancilla_type() == 5 {
                            self.ancilla_slot_view_mut(0).set_ancilla_type(0);
                        }
                    }
                    action = 1;
                }
            }

            const ABILITY_BITMASKS: [u8; 8] = [0xe0, 0x40, 4, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0];
            if action as usize >= ABILITY_BITMASKS.len()
                || (ABILITY_BITMASKS[action as usize]
                    & self.player_resources_view().ability_flags())
                    == 0
            {
                self.player_state_view_mut().set_y_button_action_flags(0);
                return;
            }
            self.player_state_view_mut().set_tile_action_index(action);
            self.link_a_press_perform_basic(action.wrapping_mul(2));
        }

        let action_index = self.player_state_view().tile_action_index();
        self.player_state_view_mut()
            .set_cached_tile_action_index(action_index);
        match self.player_state_view().tile_action_index() {
            1 => self.link_a_press_lift_carry_throw(),
            3 => self.link_a_press_pull_object(),
            6 => self.link_a_press_statue_drag(),
            _ => {}
        }
    }

    pub(super) fn link_a_press_perform_basic(&mut self, action_x2: u8) {
        match action_x2 >> 1 {
            0 => self.link_perform_desert_prayer(),
            1 => self.link_perform_throw(),
            2 => self.link_perform_dash(),
            3 => self.link_perform_grab(),
            4 => self.link_perform_read(),
            5 => self.link_perform_open_chest(),
            6 => self.link_perform_statue_drag(),
            7 => self.link_perform_rupee_pull(),
            // C Link_APress_PerformBasic asserts for action slots outside 0..=7.
            _ => panic!("Link_APress_PerformBasic action {}", action_x2 >> 1),
        }
    }

    pub(super) fn link_check_new_a_press(&mut self) -> bool {
        if self.player_state_view().y_button_action_flags() & 0x80 != 0
            || self.player_state_view().incapacitated_timer() != 0
            || self.player_state_view().filtered_joypad_l() & 0x80 == 0
        {
            return false;
        }
        self.player_state_view_mut()
            .add_y_button_action_flag_bits(0x80);
        true
    }

    pub(super) fn link_perform_dash(&mut self) {
        if self.player_state_view().has_somaria_platform_state()
            || (self.player_state_view().sprite_pickup_flag()
                | self.player_state_view().ancilla_pickup_flag())
                != 0
            || self.player_state_view().is_lifting_or_carrying()
        {
            return;
        }
        self.player_state_view_mut().set_y_button_action_flags(0);
        self.player_state_view_mut().set_dash_countdown(29);
        self.player_state_view_mut().set_dash_counter(64);
        self.player_state_view_mut().set_handler_state(17);
        self.player_state_view_mut().start_running();
        let button_mask_b_y = self.player_state_view().button_mask_b_y() & 0x80;
        self.player_state_view_mut()
            .set_button_mask_b_y(button_mask_b_y);
        self.player_state_view_mut().clear_state_bits();
        self.player_state_view_mut().clear_item_in_hand();
        self.player_state_view_mut().clear_defense_flags();
        self.player_state_view_mut()
            .clear_moving_against_diag_tile();

        let follower = self.follower_state_view().indicator() as usize;
        if self.follower_state_view().indicator() == DASH_FOLLOWER_SLOWDOWN_INDICATORS[follower] {
            self.player_state_view_mut().set_speed_setting(0);
            self.follower_state_view_mut().set_reacquire_timer(64);
        }
    }

    pub(super) fn link_perform_grab(&mut self) {
        if self.player_state_view().button_mask_b_y() & 0x80 != 0
            && self.player_state_view().button_b_frames() >= 9
        {
            return;
        }
        self.player_state_view_mut().set_grabbing_wall(1);
        self.player_state_view_mut().set_direction_lock_bits(1);
        self.player_state_view_mut().clear_animation_step();
        self.player_state_view_mut().set_y_button_action_step(0);
        self.player_state_view_mut().set_y_button_action_timer(0);
        self.player_state_view_mut().clear_item_action_step_var();
    }

    pub(super) fn link_perform_read(&mut self) {
        let message = if self.world_location_state().is_indoors() {
            self.Dungeon_GetTeleMsg(self.world_location_state().dungeon_room as usize)
        } else if self.save_progress_view().progress_indicator() < 2 {
            0x003a
        } else {
            self.asset_u16(
                110,
                u16::from(self.world_location_state().overworld_screen_index()) as usize,
            )
        };
        self.dialogue_message_index_view_mut().set_value(message);
        self.main_show_text_message();
        self.player_state_view_mut().set_y_button_action_flags(0);
    }

    pub(super) fn link_perform_open_chest(&mut self) {
        if self.player_state_view().facing() != 0
            || self.player_state_view().item_receipt_method() != 0
            || self.player_state_view().has_auxiliary_state()
        {
            return;
        }

        self.player_state_view_mut().set_y_button_action_flags(0);
        let Some((mut item, chest_position)) =
            self.OpenChestForItemResult(self.tile_detect_position_view().interacting_tile() as u8)
        else {
            self.player_state_view_mut().set_item_receipt_method(0);
            return;
        };

        self.player_state_view_mut().set_item_receipt_method(1);
        const RECEIVE_ITEM_ALTERNATES: [u8; 76] = [
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 68, 255, 255, 255, 255,
            255, 53, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 70, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255,
        ];
        if let Some(&alternate) = RECEIVE_ITEM_ALTERNATES.get(item as usize) {
            if alternate != 0xff {
                let ram_addr = player_memory_location_to_give_item_to(item);
                if self.item_memory_value(ram_addr) != 0 {
                    item = alternate;
                }
            }
        }

        self.link_receive_item(item, chest_position);
    }

    pub(super) fn link_perform_statue_drag(&mut self) {
        self.player_state_view_mut().set_grabbing_wall(2);
        self.player_state_view_mut().set_direction_lock_bits(1);
        self.player_state_view_mut().clear_animation_step();
        self.player_state_view_mut().set_y_button_action_step(0);
        self.player_state_view_mut().set_y_button_action_timer(0);
        self.player_state_view_mut().clear_item_action_step_var();
    }

    pub(super) fn link_perform_rupee_pull(&mut self) {
        if self.player_state_view().facing() != 0 {
            return;
        }
        self.link_reset_properties_a();
        self.player_state_view_mut().set_grabbing_wall(2);
        self.player_state_view_mut().set_direction_lock_bits(2);
        self.player_state_view_mut().clear_animation_step();
        self.player_state_view_mut().set_y_button_action_step(0);
        self.player_state_view_mut().set_y_button_action_timer(0);
        self.player_state_view_mut().clear_item_action_step_var();
        self.player_state_view_mut().set_handler_state(29);
        self.player_state_view_mut().clear_actual_velocity_xy();
        self.player_state_view_mut().set_button_mask_b_y(0);
    }

    pub(super) fn search_for_byrna_spark(&self) -> bool {
        if self.player_state_view().position_mode_has(8) {
            return false;
        }
        (0..=4)
            .rev()
            .any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x31)
    }

    pub(super) fn link_permission_for_slosh_sounds(&self) -> bool {
        if self.player_state_view().direction() & 0x0f == 0 {
            return true;
        }
        if self.player_state_view().handler_state() != 17 {
            self.frame_state().frame_counter & 0x0f != 0
        } else {
            self.frame_state().frame_counter & 0x07 != 0
        }
    }

    pub(super) fn link_a_press_lift_carry_throw(&mut self) {
        const LIFT_THROW_ACTION_TIMERS: [u8; 10] = [8, 24, 8, 24, 8, 32, 6, 8, 13, 13];
        const LIFT_THROW_ACTION_STEPS: [u8; 10] = [0, 1, 0, 1, 0, 1, 0, 1, 2, 3];
        const LIFT_THROW_SEQUENCE_TIMERS: [u8; 29] = [
            6, 7, 7, 5, 10, 0, 23, 0, 18, 0, 18, 0, 8, 0, 8, 0, 254, 255, 17, 0, 0x54, 0x52, 0x50,
            0xff, 0x51, 0x53, 0x55, 0x56, 0x57,
        ];
        if !self.player_state_view().has_action_state() {
            return;
        }
        if self.player_state_view().picking_throw_state_has(2)
            && self.player_state_view().y_button_action_timer() >= 5
        {
            self.player_state_view_mut().set_y_button_action_timer(5);
        }
        if self.player_state_view().has_picking_throw_state() {
            self.halt_link_when_using_items();
        }
        if self.player_state_view().is_lift_throw_primed() {
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().clear_frame_change_counter();
            self.player_state_view_mut().clear_direction_flags(0x0f);
        }
        self.player_state_view_mut()
            .decrement_y_button_action_timer();
        if self.player_state_view().y_button_action_timer() != 0 {
            return;
        }
        if self.player_state_view().picking_throw_state_has(2) {
            self.player_state_view_mut().clear_state_bits();
            self.player_state_view_mut().clear_defense_flags();
            self.player_state_view_mut().set_speed_setting(0);
            if self.player_state_view().handler_state() == 24 {
                self.player_state_view_mut().clear_handler_state();
            }
        } else if self.player_state_view().action_handler_timer() != 0 {
            if self
                .player_state_view()
                .action_handler_timer()
                .wrapping_add(1)
                != 9
            {
                self.player_state_view_mut()
                    .increment_action_handler_timer();
                let timer = self.player_state_view().action_handler_timer() as usize;
                self.player_state_view_mut()
                    .set_y_button_action_timer(LIFT_THROW_ACTION_TIMERS[timer]);
                self.player_state_view_mut()
                    .set_y_button_action_step(LIFT_THROW_ACTION_STEPS[timer]);
                if self.player_state_view().action_handler_timer() == 6 {
                    self.dungeon_secret_scratch_view_mut().clear_pending_kind();
                    let (what, x, y) = if self.world_location_state().is_indoors() {
                        let mut pt = Point16U { x: 0, y: 0 };
                        let what = self.Dungeon_LiftAndReplaceLiftable(&mut pt);
                        (what, pt.x, pt.y)
                    } else {
                        let mut pt = Point16U { x: 0, y: 0 };
                        let what = self.Overworld_HandleLiftableTiles(&mut pt);
                        (what, pt.x, pt.y)
                    };
                    self.player_state_view_mut().set_handler_state(24);
                    self.player_state_view_mut().set_sprite_pickup_flag(1);
                    self.sprite_spawn_throwable_terrain((what & 0x0f).wrapping_add(1), x, y);
                    self.player_state_view_mut()
                        .clear_filtered_joypad_l_bits(0x80);
                }
                return;
            }
        } else {
            if self.player_state_view().y_button_action_step() as usize
                >= LIFT_THROW_SEQUENCE_TIMERS.len() - 1
            {
                return;
            }
            let y_button_action_step = self
                .player_state_view()
                .y_button_action_step()
                .wrapping_add(1);
            self.player_state_view_mut()
                .set_y_button_action_step(y_button_action_step);
            self.player_state_view_mut().set_y_button_action_timer(
                LIFT_THROW_SEQUENCE_TIMERS[y_button_action_step as usize],
            );
            if self.player_state_view().y_button_action_step() != 3 {
                return;
            }
        }
        self.player_state_view_mut().clear_picking_throw_state();
        self.player_state_view_mut().clear_direction_lock_bits(1);
    }

    pub(super) fn link_a_press_pull_object(&mut self) {
        const GRAB_WALL_DIRS: [u8; 4] = [4, 8, 1, 2];
        const GRAB_WALL_ANIM_STEPS: [u8; 7] = [0, 1, 2, 3, 1, 2, 3];
        const GRAB_WALL_ANIM_TIMER: [u8; 7] = [0, 5, 5, 12, 5, 5, 12];
        self.player_state_view_mut().clear_direction_flags(0x0f);
        let facing = self.player_state_view().facing_index();
        if GRAB_WALL_DIRS[facing] & self.player_state_view().joypad1h_last() == 0 {
            self.player_state_view_mut().clear_item_action_step_var();
            let step = self.player_state_view().item_action_step_var() as usize;
            self.player_state_view_mut()
                .set_y_button_action_step(GRAB_WALL_ANIM_STEPS[step]);
            self.player_state_view_mut()
                .set_y_button_action_timer(GRAB_WALL_ANIM_TIMER[step]);
        } else {
            self.player_state_view_mut()
                .decrement_y_button_action_timer();
            if (self.player_state_view().y_button_action_timer() as i8) < 0 {
                let step = self
                    .player_state_view_mut()
                    .advance_item_action_step_var_wrapping_7_to_1()
                    as usize;
                self.player_state_view_mut()
                    .set_y_button_action_step(GRAB_WALL_ANIM_STEPS[step]);
                self.player_state_view_mut()
                    .set_y_button_action_timer(GRAB_WALL_ANIM_TIMER[step]);
            }
        }
        if self.player_state_view().joypad1l_last() & 0x80 == 0 {
            self.player_state_view_mut().clear_item_action_step_var();
            self.player_state_view_mut().set_y_button_action_step(0);
            self.player_state_view_mut().clear_grabbing_wall();
            self.player_state_view_mut().set_y_button_action_flags(0);
            self.player_state_view_mut().clear_direction_lock_bits(1);
        }
    }

    pub(super) fn link_a_press_statue_drag(&mut self) {
        const GRAB_WALL_DIRS: [u8; 4] = [4, 8, 1, 2];
        const GRAB_WALL_ANIM_STEPS: [u8; 7] = [0, 1, 2, 3, 1, 2, 3];
        const GRAB_WALL_ANIM_TIMER: [u8; 7] = [0, 5, 5, 12, 5, 5, 12];
        self.player_state_view_mut().set_speed_setting(20);
        let j = self.player_state_view().joypad1h_last()
            & GRAB_WALL_DIRS[self.player_state_view().facing_index()];
        if j == 0 {
            self.player_state_view_mut()
                .clear_movement_velocity_and_direction();
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().clear_item_action_step_var();
        } else {
            self.player_state_view_mut().set_direction(j);
            self.player_state_view_mut()
                .decrement_y_button_action_timer();
            if (self.player_state_view().y_button_action_timer() as i8) >= 0 {
                if self.player_state_view().joypad1l_last() & 0x80 == 0 {
                    self.link_a_press_statue_drag_release();
                }
                return;
            }
            self.player_state_view_mut()
                .advance_item_action_step_var_wrapping_7_to_1();
        }
        let step = self.player_state_view().item_action_step_var() as usize;
        self.player_state_view_mut()
            .set_y_button_action_step(GRAB_WALL_ANIM_STEPS[step]);
        self.player_state_view_mut()
            .set_y_button_action_timer(GRAB_WALL_ANIM_TIMER[step]);
        if self.player_state_view().joypad1l_last() & 0x80 == 0 {
            self.link_a_press_statue_drag_release();
        }
    }

    fn link_a_press_statue_drag_release(&mut self) {
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().clear_near_moveable_statue();
        self.player_state_view_mut().clear_item_action_step_var();
        self.player_state_view_mut().set_y_button_action_step(0);
        self.player_state_view_mut().clear_grabbing_wall();
        self.player_state_view_mut().set_y_button_action_flags(0);
        self.player_state_view_mut().clear_direction_lock_bits(1);
    }

    pub(super) fn link_item_bombs(&mut self) {
        const FEATURES0_MORE_ACTIVE_BOMBS: u32 = 1 << 2;
        if self.player_state_view().doorway_state() != 0
            || self.follower_state_view().indicator() == 13
            || !self.check_y_button_press()
        {
            return;
        }
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
        let limit = if self
            .enhanced_features_view()
            .has(FEATURES0_MORE_ACTIVE_BOMBS)
        {
            3
        } else {
            1
        };
        self.ancilla_add_bomb(7, limit);
        self.player_state_view_mut().clear_item_in_hand();
    }

    pub(super) fn link_item_book(&mut self) {
        if self.player_state_view().button_mask_b_y() & 0x40 != 0
            || self.player_state_view().doorway_state() != 0
            || !self.check_y_button_press()
        {
            return;
        }
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
        if self.player_state_view().item_pickup_in_progress() {
            self.link_perform_desert_prayer();
        } else {
            self.ancilla_sfx2_near(60);
        }
    }

    pub(super) fn link_item_bottle(&mut self) {
        if !self.check_y_button_press() {
            return;
        }
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
        let btidx = self
            .player_resources_view()
            .equipped_bottle_index()
            .wrapping_sub(1) as usize;
        if btidx >= 4 {
            return;
        }
        let bottle = self.inventory_items().bottle(btidx);
        if bottle == 0 {
            return;
        }
        if bottle < 3 {
            self.ancilla_sfx2_near(60);
        } else if bottle == 3 {
            if self.player_resources_view().health_capacity()
                == self.player_resources_view().current_health()
            {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.player_state_view_mut().clear_item_in_hand();
            let main_module = self.frame_state().main_module;
            self.set_submodule(4);
            self.set_saved_module_for_menu(main_module);
            self.set_main_module(14);
            self.hud_state_view_mut().set_heart_refill_countdown(7);
            self.hud_rebuild();
        } else if bottle == 4 {
            if self.player_state_view().magic_power() == 128 {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.player_state_view_mut().clear_item_in_hand();
            let main_module = self.frame_state().main_module;
            self.set_submodule(8);
            self.set_saved_module_for_menu(main_module);
            self.set_main_module(14);
            self.hud_state_view_mut().set_heart_refill_countdown(7);
            self.hud_rebuild();
        } else if bottle == 5 {
            if self.player_resources_view().health_capacity()
                == self.player_resources_view().current_health()
                && self.player_state_view().magic_power() == 128
            {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.player_state_view_mut().clear_item_in_hand();
            let main_module = self.frame_state().main_module;
            self.set_submodule(9);
            self.set_saved_module_for_menu(main_module);
            self.set_main_module(14);
            self.hud_state_view_mut().set_heart_refill_countdown(7);
            self.hud_rebuild();
        } else if bottle == 6 {
            self.player_state_view_mut().clear_item_in_hand();
            if self.release_fairy() < 0 {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.hud_rebuild();
        } else if bottle == 7 || bottle == 8 {
            if self.release_bee_from_bottle(btidx) == 0 {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.hud_rebuild();
        }
    }

    pub(super) fn link_perform_desert_prayer(&mut self) {
        let main_module = self.frame_state().main_module;
        self.set_submodule(5);
        self.set_saved_module_for_menu(main_module);
        self.set_main_module(14);
        self.set_modal_pause_flag(1);
        self.player_state_view_mut().set_y_button_action_timer(22);
        self.player_state_view_mut().set_y_button_action_step(0);
        self.player_state_view_mut().set_state_bits(2);
        self.player_state_view_mut().set_direction_lock_bits(1);
        self.player_state_view_mut().clear_animation_step();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        self.system_signals_view_mut().set_ambient_sound_effect(17);
        self.system_signals_view_mut().set_music_control(242);
    }

    pub(super) fn link_item_lamp(&mut self) {
        if self.player_state_view().doorway_state() != 0 || !self.check_y_button_press() {
            return;
        }
        if self.inventory_items().torch() != 0 && self.link_check_magic_cost(6) {
            self.ancilla_add_magic_powder(0x1a, 0);
            self.dungeon_light_torch();
            self.ancilla_add_lamp_flame(0x2f, 2);
        }
        self.player_state_view_mut().clear_item_in_hand();
        self.player_state_view_mut().set_button_mask_b_y(0);
        self.player_state_view_mut().set_button_b_frames(0);
        self.player_state_view_mut().clear_direction_lock();
    }

    pub(super) fn link_item_powder(&mut self) {
        const MUSHROOM_TIMER: [u8; 10] = [2, 1, 1, 3, 2, 2, 2, 2, 6, 0];

        if self.player_state_view().button_mask_b_y() & 0x40 == 0 {
            if self.player_state_view().doorway_state() != 0 || !self.check_y_button_press() {
                return;
            }
            if self.inventory_items().mushroom() != 2 {
                self.ancilla_sfx2_near(60);
                self.finish_powder_item();
                return;
            }
            if !self.link_check_magic_cost(2) {
                self.finish_powder_item();
                return;
            }
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(MUSHROOM_TIMER[0]);
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().clear_direction_flags(0x0f);
            self.player_state_view_mut().set_item_in_hand(0x40);
        }

        self.player_state_view_mut()
            .clear_movement_velocity_and_direction();
        self.player_state_view_mut().clear_movement_subpixels();
        self.player_state_view_mut()
            .clear_moving_against_diag_tile();
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.player_state_view_mut()
            .increment_action_handler_timer();
        let step = self.player_state_view().action_handler_timer() as usize;
        self.player_state_view_mut()
            .set_spin_attack_delay_timer(MUSHROOM_TIMER[step]);
        if self.player_state_view().action_handler_timer() == 4 {
            self.ancilla_add_magic_powder(26, 0);
        }
        if self.player_state_view().action_handler_timer() == 9 {
            if self.frame_state().submodule == 0 {
                self.tile_detect_main_handler(1);
            }
            self.finish_powder_item();
        }
    }

    pub(super) fn link_item_shovel_and_flute(&mut self) {
        if self.inventory_items().flute() == 1 {
            self.link_item_shovel();
        } else if self.inventory_items().flute() != 0 {
            self.link_item_flute();
        }
    }

    pub(super) fn link_item_shovel(&mut self) {
        const SHOVEL_ANIM_DELAY: [u8; 6] = [7, 18, 16, 7, 18, 16];
        const SHOVEL_ANIM_DELAY2: [u8; 6] = [0, 1, 2, 0, 1, 2];
        if self.player_state_view().button_mask_b_y() & 0x40 == 0 {
            if self.player_state_view().doorway_state() != 0 || !self.check_y_button_press() {
                return;
            }
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(SHOVEL_ANIM_DELAY[0]);
            self.player_state_view_mut().clear_item_action_step_var();
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_position_mode(1);
            self.player_state_view_mut().set_direction_lock_bits(1);
            self.player_state_view_mut().clear_animation_step();
        }
        self.halt_link_when_using_items();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        let step = self
            .player_state_view_mut()
            .increment_item_action_step_var() as usize;
        self.player_state_view_mut()
            .set_spin_attack_delay_timer(SHOVEL_ANIM_DELAY[step]);
        self.player_state_view_mut()
            .set_action_handler_timer(SHOVEL_ANIM_DELAY2[step]);

        if self.player_state_view().action_handler_timer() == 1 {
            self.tile_detect_main_handler(2);
            if self.world_state_view().overworld_hole_tilemap_pos() != 0 {
                self.ancilla_sfx3_near(27);
                self.ancilla_add_dug_up_flute(54, 0);
            }
            if (self.tile_detect_position_view().thick_grass()
                | self.tile_detect_position_view().destruction_aftermath())
                & 1
                == 0
            {
                self.ancilla_add_hit_stars(22, 0);
                self.ancilla_sfx2_near(5);
            } else {
                self.ancilla_add_shovel_dirt(23, 0);
                if self.minigame_state_view().is_archer_or_shovel_game() != 0 {
                    self.digging_game_guy_attempt_prize_spawn();
                }
                self.ancilla_sfx2_near(18);
            }
        }

        if self.player_state_view().item_action_step_var() == 3 {
            self.player_state_view_mut().clear_item_action_step_var();
            self.player_state_view_mut().clear_action_handler_timer();
            let button_mask_b_y = self.player_state_view().button_mask_b_y() & 0x80;
            self.player_state_view_mut()
                .set_button_mask_b_y(button_mask_b_y);
            self.player_state_view_mut().clear_position_mode();
            self.player_state_view_mut().clear_direction_lock_bits(1);
        }
    }

    pub(super) fn link_item_flute(&mut self) {
        if self.player_state_view().button_mask_b_y() & 0x40 != 0 {
            self.player_state_view_mut().decrement_flute_countdown();
            if self.player_state_view().flute_countdown() != 0 {
                return;
            }
            self.player_state_view_mut()
                .clear_button_mask_b_y_bits(0x40);
        }
        if !self.check_y_button_press() {
            return;
        }
        self.player_state_view_mut().set_flute_countdown(128);
        self.ancilla_sfx2_near(19);
        if self.world_location_state().is_indoors()
            || u16::from(self.world_location_state().overworld_screen_index()) & 0x40 != 0
            || self.frame_state().main_module == 11
        {
            return;
        }
        if (0..5).any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x27) {
            return;
        }
        if self.inventory_items().flute() == 2 {
            let screen = u16::from(self.world_location_state().overworld_screen_index());
            let y = self.player_state_view().y();
            let x = self.player_state_view().x();
            if screen == 0x18 && (0x760..0x7e0).contains(&y) && (0x1cf..0x230).contains(&x) {
                self.set_submodule(45);
                self.ancilla_add_exploding_weather_vane(55, 0);
            }
        } else {
            self.ancilla_add_duck_take_off(39, 4);
            self.player_state_view_mut()
                .clear_pull_for_rupees_sprite_need();
        }
    }

    pub(super) fn link_handle_y_item(&mut self) {
        if self.player_state_view().button_b_frames() != 0
            && self.player_state_view().button_b_frames() < 9
        {
            return;
        }

        let mut item = self.player_state_view().current_item_y();
        if self.player_state_view().is_bunny_mirror() && item != 11 && item != 20 {
            return;
        }

        if self.minigame_state_view().is_archer_or_shovel_game() != 0
            && !self.player_state_view().is_bunny_mirror()
        {
            if self.minigame_state_view().is_archer_or_shovel_game() == 2 {
                self.link_item_bow();
            } else {
                self.link_item_shovel();
            }
            return;
        }

        let old_down = self.player_state_view().joypad1h_last();
        let old_pressed = self.player_state_view().filtered_joypad_h();
        let old_bottle = self.player_resources_view().equipped_bottle_index();
        if !self.player_state_view().has_item_in_hand()
            && !self.player_state_view().has_position_mode()
            && old_down & 0x40 == 0
        {
            let btn_index = self.get_current_item_button_index();
            if btn_index != 0 {
                let hud_item = self.save_progress_view().hud_current_item_slot(btn_index);
                if hud_item != 0 {
                    if hud_item >= 21 {
                        self.player_resources_view_mut()
                            .set_equipped_bottle_index(hud_item - 20);
                    }
                    item = self.hud_lookup_inventory_item(hud_item);
                    self.player_state_view_mut()
                        .set_joypad1h_last(old_down | 0x40);
                    const BUTTON_INDEX_KEYS: [u8; 4] = [0, 0x40, 0x20, 0x10];
                    if self.player_state_view().filtered_joypad_l() & BUTTON_INDEX_KEYS[btn_index]
                        != 0
                    {
                        self.player_state_view_mut()
                            .set_filtered_joypad_h(old_pressed | 0x40);
                    }
                }
            }
        }

        if item != self.player_state_view().current_item_active() {
            if self.player_state_view().current_item_active() == 8
                && self.inventory_items().flute() & 2 != 0
            {
                self.player_state_view_mut()
                    .clear_button_mask_b_y_bits(0x40);
            }
            if self.player_state_view().current_item_active() == 19
                && self.player_state_view().is_cape_active()
            {
                self.link_force_unequip_cape();
            }
        }

        if !self.player_state_view().has_item_in_hand()
            && !self.player_state_view().has_position_mode()
        {
            self.player_state_view_mut().set_current_item_active(item);
        }
        if matches!(self.player_state_view().current_item_active(), 5 | 6) {
            let rod = self.player_state_view().current_item_active() - 4;
            self.player_state_view_mut().set_selected_rod(rod);
        }

        match self.player_state_view().current_item_active() {
            0 => {}
            1 => self.link_item_bombs(),
            2 => self.link_item_boomerang(),
            3 => self.link_item_bow(),
            4 => self.link_item_hammer(),
            5 | 6 => self.link_item_rod(),
            7 => self.link_item_net(),
            8 => self.link_item_shovel_and_flute(),
            9 => self.link_item_lamp(),
            10 => self.link_item_powder(),
            11 => self.link_item_bottle(),
            12 => self.link_item_book(),
            13 => self.link_item_cane_of_byrna(),
            14 => self.link_item_hookshot(),
            15 => self.link_item_bombos(),
            16 => self.link_item_ether(),
            17 => self.link_item_quake(),
            18 => self.link_item_cane_of_somaria(),
            19 => self.link_item_cape(),
            20 => self.link_item_mirror(),
            21 => self.link_item_shovel(),
            // C Link_HandleYItem asserts outside item slots 0..=21.
            _ => panic!(
                "Link_HandleYItem current_item_active {}",
                self.player_state_view().current_item_active()
            ),
        }

        self.player_state_view_mut().set_joypad1h_last(old_down);
        self.player_state_view_mut()
            .set_filtered_joypad_h(old_pressed);
        self.player_resources_view_mut()
            .set_equipped_bottle_index(old_bottle);
    }

    pub(super) fn link_item_ether(&mut self) {
        const ETHER_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 3, 3];
        const ETHER_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 6, 7];
        self.start_medallion_item(8, ETHER_ANIM_DELAYS[0], ETHER_ANIM_STATES[0], None);
    }

    pub(super) fn link_item_bombos(&mut self) {
        const BOMBOS_ANIM_DELAYS: [u8; 20] =
            [5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3, 3, 7, 1, 1, 1, 1, 1, 13];
        const BOMBOS_ANIM_STATES: [u8; 20] = [
            0, 1, 2, 3, 0, 1, 2, 3, 8, 9, 10, 11, 12, 10, 8, 13, 14, 15, 16, 17,
        ];
        self.start_medallion_item(9, BOMBOS_ANIM_DELAYS[0], BOMBOS_ANIM_STATES[0], None);
    }

    pub(super) fn link_item_quake(&mut self) {
        const QUAKE_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 19];
        const QUAKE_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 18, 19, 20, 22];
        self.start_medallion_item(10, QUAKE_ANIM_DELAYS[0], QUAKE_ANIM_STATES[0], Some(()));
    }

    pub(super) fn link_state_using_ether(&mut self) {
        const ETHER_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 3, 3];
        const ETHER_ANIM_DELAYS_NO_FLASH: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 24, 24];
        const ETHER_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 6, 7];
        const FEATURES0_DIM_FLASHES: u32 = 65536;

        self.increment_modal_pause_flag();
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.player_state_view_mut()
            .increment_spin_animation_step_counter();
        if self.player_state_view().spin_animation_step_counter() == 4 {
            self.ancilla_sfx3_near(35);
        } else if self.player_state_view().spin_animation_step_counter() == 9 {
            self.ancilla_sfx2_near(44);
        } else if self.player_state_view().spin_animation_step_counter() == 12 {
            self.player_state_view_mut()
                .set_spin_animation_step_counter(10);
        }

        let step = self.player_state_view().spin_animation_step_counter() as usize;
        let delays = if self.enhanced_features_view().has(FEATURES0_DIM_FLASHES) {
            ETHER_ANIM_DELAYS_NO_FLASH
        } else {
            ETHER_ANIM_DELAYS
        };
        self.player_state_view_mut()
            .set_spin_attack_delay_timer(delays[step]);
        self.player_state_view_mut()
            .set_state_for_spin_attack(ETHER_ANIM_STATES[step]);
        if self.player_state_view().spin_attack_sound_latch() == 0
            && self.player_state_view().spin_animation_step_counter() == 10
        {
            self.player_state_view_mut().set_spin_attack_sound_latch(1);
            self.ancilla_add_ether_spell(24, 0);
            self.player_state_view_mut().clear_auxiliary_state();
            self.player_state_view_mut().set_incapacitated_timer(0);
        }
    }

    pub(super) fn link_state_using_bombos(&mut self) {
        const BOMBOS_ANIM_DELAYS: [u8; 20] =
            [5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3, 3, 7, 1, 1, 1, 1, 1, 13];
        const BOMBOS_ANIM_STATES: [u8; 20] = [
            0, 1, 2, 3, 0, 1, 2, 3, 8, 9, 10, 11, 12, 10, 8, 13, 14, 15, 16, 17,
        ];

        self.increment_modal_pause_flag();
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.player_state_view_mut()
            .increment_spin_animation_step_counter();
        if self.player_state_view().spin_animation_step_counter() == 4 {
            self.ancilla_sfx3_near(35);
        } else if self.player_state_view().spin_animation_step_counter() == 10 {
            self.ancilla_sfx2_near(44);
        } else if self.player_state_view().spin_animation_step_counter() == 20 {
            self.player_state_view_mut()
                .set_spin_animation_step_counter(19);
        }
        let step = self.player_state_view().spin_animation_step_counter() as usize;
        self.player_state_view_mut()
            .set_spin_attack_delay_timer(BOMBOS_ANIM_DELAYS[step]);
        self.player_state_view_mut()
            .set_state_for_spin_attack(BOMBOS_ANIM_STATES[step]);
        if self.player_state_view().spin_attack_sound_latch() == 0
            && self.player_state_view().spin_animation_step_counter() == 19
        {
            self.player_state_view_mut().set_spin_attack_sound_latch(1);
            self.ancilla_add_bombos_spell(25, 0);
            self.player_state_view_mut().clear_auxiliary_state();
            self.player_state_view_mut().set_incapacitated_timer(0);
        }
    }

    pub(super) fn link_state_using_quake(&mut self) {
        const QUAKE_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 19];
        const QUAKE_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 18, 19, 20, 22];
        self.increment_modal_pause_flag();
        self.player_state_view_mut().clear_actual_velocity_xy();

        if self.player_state_view().spin_animation_step_counter() == 10 {
            self.player_state_view_mut()
                .restore_actual_z_velocity_from_mirror();
            self.player_state_view_mut().restore_z_low_from_mirror();
            self.player_state_view_mut().set_auxiliary_state(2);
            self.player_change_z(2);
            self.link_move_position();
            self.player_state_view_mut()
                .cache_actual_z_velocity_to_mirror();
            self.player_state_view_mut().cache_z_low_to_mirror();
            if !self.player_state_view().is_z_low_negative() {
                let spin_state = if (self.player_state_view().actual_z_velocity() as i8) < 0 {
                    21
                } else {
                    20
                };
                self.player_state_view_mut()
                    .set_state_for_spin_attack(spin_state);
                return;
            }
        } else {
            if (self
                .player_state_view_mut()
                .decrement_spin_attack_delay_timer() as i8)
                >= 0
            {
                return;
            }
        }

        self.player_state_view_mut()
            .increment_spin_animation_step_counter();
        if self.player_state_view().spin_animation_step_counter() == 4 {
            self.ancilla_sfx3_near(35);
        } else if self.player_state_view().spin_animation_step_counter() == 10 {
            self.ancilla_sfx2_near(44);
        } else if self.player_state_view().spin_animation_step_counter() == 11 {
            self.ancilla_sfx2_near(12);
        } else if self.player_state_view().spin_animation_step_counter() == 12 {
            self.player_state_view_mut()
                .set_spin_animation_step_counter(11);
        }
        let step = self.player_state_view().spin_animation_step_counter() as usize;
        self.player_state_view_mut()
            .set_spin_attack_delay_timer(QUAKE_ANIM_DELAYS[step]);
        self.player_state_view_mut()
            .set_state_for_spin_attack(QUAKE_ANIM_STATES[step]);
        if self.player_state_view().spin_attack_sound_latch() == 0
            && self.player_state_view().spin_animation_step_counter() == 11
        {
            self.player_state_view_mut().set_spin_attack_sound_latch(1);
            self.ancilla_add_quake_spell(28, 0);
            self.player_state_view_mut().clear_auxiliary_state();
            self.player_state_view_mut().set_incapacitated_timer(0);
        }
    }

    pub(super) fn link_item_mirror(&mut self) {
        if self.player_state_view().button_mask_b_y() & 0x40 == 0 {
            if !self.check_y_button_press() {
                return;
            }
            if self.follower_state_view().indicator() == 10 {
                self.dialogue_message_index_view_mut().set_value(289);
                self.main_show_text_message();
                return;
            }
        }
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);

        const FEATURES0_MIRROR_TO_DARKWORLD: u32 = 8;
        if self.player_state_view().doorway_state() != 0
            || (self.player_state_view().cheat_walk_through_walls() == 0
                && !self
                    .enhanced_features_view()
                    .has(FEATURES0_MIRROR_TO_DARKWORLD)
                && self.world_location_state().is_outdoors()
                && u16::from(self.world_location_state().overworld_screen_index()) & 0x40 == 0)
        {
            self.ancilla_sfx2_near(60);
            return;
        }

        self.do_sword_interaction_with_tiles_mirror();
    }

    pub(super) fn do_sword_interaction_with_tiles_mirror(&mut self) {
        if self.world_location_state().is_indoors() {
            if self.player_state_view().is_menu_blocked() {
                return;
            }
            self.Mirror_SaveRoomData();
            if self.system_signals_view().sound_effect_1() != 60 {
                self.dungeon_state_view_mut()
                    .clear_changeable_object_index(0);
                self.dungeon_state_view_mut()
                    .clear_changeable_object_index(1);
            }
            return;
        }
        if self.frame_state().main_module == 11 {
            return;
        }
        let screen = u16::from(self.world_location_state().overworld_screen_index());
        self.world_state_view_mut()
            .set_last_light_vs_dark_world((screen & 0x40) as u8);
        if self.world_state_view().last_light_vs_dark_world() != 0 {
            let y = self.player_state_view().y();
            let x = self.player_state_view().x();
            self.set_bird_travel_destination(15, x, y);
        }
        self.set_submodule(35);
        self.player_state_view_mut()
            .clear_pull_for_rupees_sprite_need();
        self.player_state_view_mut().set_whirlpool_trigger();
        self.set_subsubmodule(0);
        self.player_state_view_mut().clear_actual_velocity_xy();
        self.player_state_view_mut().set_handler_state(20);
    }

    pub(super) fn link_state_crossing_worlds(&mut self) {
        self.link_reset_properties_b();
        self.tile_check_for_mirror_bonk();
        let world_changed = (u16::from(self.world_location_state().overworld_screen_index()) as u8
            & 0x40)
            != self.world_state_view().last_light_vs_dark_world();
        let bonk_bits = self.tile_detect_position_view().bonk_bits_low();
        if world_changed && bonk_bits & 0x0c != 0 && Self::bit_sum4(bonk_bits) >= 2 {
            self.start_mirror_transition(44);
            return;
        }

        if Self::bit_sum4(self.tile_detect_position_view().deepwater() as u8) >= 2 {
            if self.player_state_view().has_flippers() {
                self.link_set_to_deep_water();
                self.player_state_view_mut().set_handler_state(4);
                self.link_force_unequip_cape_quietly();
                return;
            }
            if world_changed {
                self.start_mirror_transition(44);
                return;
            }
            self.check_ability_to_swim();
        }

        if self.player_state_view().is_in_deep_water() {
            self.player_state_view_mut().clear_deep_water_state();
            self.player_state_view_mut()
                .set_last_direction_from_swim_flags();
        }
        self.player_state_view_mut().set_dash_countdown(0);
        self.player_state_view_mut().clear_running();
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().set_button_mask_b_y(0);
        self.player_state_view_mut().set_button_b_frames(0);
        self.player_state_view_mut().clear_direction_lock();
        self.swim_acceleration_view_mut().set_mode(0, 0);
        self.player_state_view_mut().set_actual_y_velocity(0);
        if world_changed {
            self.memorized_tile_view_mut().set_count(0);
        }
        let handler_state = if self.player_state_view().has_moon_pearl()
            || u16::from(self.world_location_state().overworld_screen_index()) & 0x40 == 0
        {
            0
        } else {
            23
        };
        self.player_state_view_mut()
            .set_handler_state(handler_state);
    }

    pub(super) fn handle_followers_after_mirroring(&mut self) {
        self.tile_detect_main_handler(0);
        self.player_state_view_mut().clear_animation_step();
        match self.follower_state_view().indicator() {
            12 | 13 => {
                if self.follower_state_view().indicator() == 13 {
                    self.hud_state_view_mut()
                        .set_super_bomb_indicator_timer(0xfe);
                    self.hud_state_view_mut()
                        .set_super_bomb_indicator_counter(0);
                }
                if self.follower_state_view().dropped() != 0 {
                    self.follower_state_view_mut().set_dropped(0);
                    self.follower_state_view_mut().set_indicator(0);
                }
            }
            9 | 10 => self.follower_state_view_mut().set_indicator(0),
            7 | 8 => {
                self.follower_state_view_mut().xor_indicator(7 ^ 8);
                self.load_follower_graphics();
                self.ancilla_add_dwarf_poof(0x40, 4);
            }
            _ => {}
        }

        if !self.player_state_view().has_moon_pearl() {
            self.ancilla_add_bunny_poof(0x23, 4);
            self.link_force_unequip_cape_quietly();
            self.player_state_view_mut().clear_cape_transform_timer();
        } else if self.player_state_view().is_cape_active() {
            self.link_force_unequip_cape();
            self.player_state_view_mut().clear_cape_transform_timer();
        }
    }

    pub(super) fn link_item_hookshot(&mut self) {
        if self.player_state_view().button_mask_b_y() & 0x40 != 0
            || self.player_state_view().doorway_state() != 0
            || self.player_state_view().defense_flags() & 2 != 0
            || !self.check_y_button_press()
        {
            return;
        }

        self.reset_all_acceleration();
        self.player_state_view_mut().clear_action_handler_timer();
        self.player_state_view_mut().set_direction_lock_bits(1);
        self.player_state_view_mut().set_spin_attack_delay_timer(7);
        self.player_state_view_mut().clear_animation_step();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        self.player_state_view_mut().set_position_mode(4);
        self.player_state_view_mut().set_handler_state(19);
        self.player_state_view_mut()
            .set_sprite_damage_disable_timer(1);
        self.ancilla_add_hookshot(0x1f, 3);
    }

    pub(super) fn link_state_hookshotting(&mut self) {
        const HOOKSHOT_TARGET_Y_OFFSETS: [i8; 4] = [-8, -16, 0, 0];
        const HOOKSHOT_TARGET_X_OFFSETS: [i8; 4] = [0, 0, 4, -12];
        const HOOKSHOT_PULL_Y_VELOCITIES: [u8; 4] = [0xc0, 0x40, 0, 0];
        const HOOKSHOT_PULL_X_VELOCITIES: [u8; 4] = [0, 0, 0xc0, 0x40];

        self.player_state_view_mut().clear_given_damage();
        self.player_state_view_mut().clear_auxiliary_state();
        self.player_state_view_mut().set_incapacitated_timer(0);
        let hookshot = (0..=4)
            .rev()
            .find(|&i| self.ancilla_slot_view(i).ancilla_type() == 0x1f);
        let Some(_k) = hookshot else {
            if (self
                .player_state_view_mut()
                .decrement_spin_attack_delay_timer() as i8)
                >= 0
            {
                return;
            }
            self.finish_hookshot_state();
            return;
        };

        if self.player_state_view().spin_attack_delay_timer() != 0 {
            if (self
                .player_state_view_mut()
                .decrement_spin_attack_delay_timer() as i8)
                < 0
            {
                self.player_state_view_mut().set_spin_attack_delay_timer(0);
            }
        }

        if !self.player_state_view().has_hookshot_interlock() {
            self.player_state_view_mut()
                .store_safe_return_low_from_current();
            self.player_state_view_mut().set_y_velocity(0);
            self.player_state_view_mut().set_x_velocity(0);
            self.link_handle_cardinal_collision();
            return;
        }

        self.player_state_view_mut().clear_somaria_platform_state();

        let hei = self.messaging_state_view().effect_index() as usize;
        let item_to_link = self.ancilla_slot_view(hei).item_to_link().wrapping_sub(1);
        self.ancilla_slot_view_mut(hei)
            .set_item_to_link(item_to_link);
        if (item_to_link as i8) < 0 {
            self.ancilla_slot_view_mut(hei).set_item_to_link(0);
        } else {
            let hookshot = self.ancilla_slot_view(hei);
            let dir = hookshot.direction() as usize;
            let x = hookshot.x();
            let y = hookshot.y();
            let target_y = y.wrapping_add(HOOKSHOT_TARGET_Y_OFFSETS[dir] as i16 as u16);
            let target_x = x.wrapping_add(HOOKSHOT_TARGET_X_OFFSETS[dir] as i16 as u16);
            let yd = target_y.wrapping_sub(self.player_state_view().y()) as i16;
            let mut actual_y_velocity = 0;
            if yd.wrapping_abs() >= 2 {
                actual_y_velocity = HOOKSHOT_PULL_Y_VELOCITIES[dir];
            }
            let xd = target_x.wrapping_sub(self.player_state_view().x()) as i16;
            let mut actual_x_velocity = 0;
            if xd.wrapping_abs() >= 2 {
                actual_x_velocity = HOOKSHOT_PULL_X_VELOCITIES[dir];
            }
            self.player_state_view_mut()
                .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);
            if (actual_x_velocity | actual_y_velocity) != 0 {
                self.continue_hookshot_drag();
                return;
            }
        }

        self.ancilla_slot_view_mut(hei).set_ancilla_type(0);
        self.follower_state_view_mut()
            .set_hookshot_release_tail_index_from_tail_write_index();
        self.finish_hookshot_state_without_button_clamp();

        if self.ancilla_slot_view(hei).work_byte_1() != 0 {
            self.player_state_view_mut()
                .toggle_lower_level_mirror_state();
            self.dungeon_state_view_mut().decrement_current_floor();
            if self.dungeon_state_view().kind_of_in_room_staircase() == 0 {
                let dungeon_room_index = self.world_location_state().dungeon_room_index();
                self.dungeon_state_view_mut()
                    .set_room_index2(dungeon_room_index);
                self.increment_dungeon_room_index_by(0x10);
            }
            if self.dungeon_state_view().kind_of_in_room_staircase() != 2 {
                self.player_state_view_mut().toggle_lower_level_state();
            }
            self.Dungeon_FlagRoomData_Quadrants();
        }

        self.player_tile_detect_nearby();
        if self.tile_detect_position_view().deepwater() & 0x0f != 0
            && !self.player_state_view().is_in_deep_water()
        {
            self.link_set_to_deep_water();
            self.ancilla_add_splash(21, 0);
            self.player_state_view_mut().set_handler_state(4);
            self.link_force_unequip_cape_quietly();
            self.player_state_view_mut()
                .clear_state_item_and_grab_flags();
            self.player_state_view_mut().set_speed_setting(0);
            if self.world_location_state().is_indoors() {
                self.player_state_view_mut().mark_lower_level();
            }
            if self.player_state_view().button_b_frames() >= 9 {
                self.player_state_view_mut().set_button_b_frames(9);
            }
        } else if self.tile_detect_position_view().pit_tile() & 0x0f != 0 {
            self.player_state_view_mut().set_sprite_oam_state_timer(9);
            self.player_state_view_mut().begin_pit_check();
            self.player_state_view_mut().set_handler_state(1);
            if self.player_state_view().button_b_frames() >= 9 {
                self.player_state_view_mut().set_button_b_frames(9);
            }
        } else {
            let y = self.player_state_view().y();
            let x = self.player_state_view().x();
            self.player_state_view_mut()
                .store_safe_return_position(x, y);
            self.link_handle_cardinal_collision();
            self.handle_indoor_camera_and_doors();
        }
    }

    fn continue_hookshot_drag(&mut self) {
        self.link_move_position();
        self.tile_detect_main_handler(5);
        if self.world_location_state().is_indoors() {
            let x = (self.tile_detect_position_view().vertical_ledge() >> 4)
                | self.tile_detect_position_view().vertical_ledge()
                | self.tile_detect_position_view().horizontal_ledge();
            if x & 1 != 0 {
                self.player_state_view_mut()
                    .decrement_hookshot_bg_check_off_timer();
                if (self.player_state_view().hookshot_bg_check_off_timer() as i8) < 0 {
                    self.player_state_view_mut()
                        .set_hookshot_bg_check_off_timer(3);
                    self.player_state_view_mut().xor_hookshot_interlock(2);
                }
            }
        }
        self.player_state_view_mut()
            .clear_water_ripple_or_grass_state();
        if !self.player_state_view().hookshot_interlock_has(2) {
            if self.tile_detect_position_view().thick_grass_low() & 1 != 0 {
                self.player_state_view_mut()
                    .set_water_ripple_or_grass_state(2);
                if !self.link_permission_for_slosh_sounds() {
                    self.ancilla_sfx2_near(26);
                }
            } else if (self.tile_detect_position_view().shallow_water_low()
                | self.tile_detect_position_view().deepwater() as u8)
                & 1
                != 0
            {
                self.player_state_view_mut()
                    .increment_water_ripple_or_grass_state();
                self.ancilla_sfx2_near(
                    if self.world_location_state().overworld_screen_index() == 0x70 {
                        27
                    } else {
                        28
                    },
                );
            }
        }
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_item_cane_of_somaria(&mut self) {
        const ROD_ANIM_DELAYS: [u8; 3] = [3, 3, 5];
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        if self.player_state_view().button_mask_b_y() & 0x40 == 0 {
            if self.player_state_view().has_somaria_platform_state()
                || self.player_state_view().doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }

            let mut did_charge_magic = false;
            if !(0..5).any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x2c) {
                if !self.link_check_magic_cost(4) {
                    if self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES) {
                        self.player_state_view_mut()
                            .clear_button_mask_b_y_bits(0x40);
                    }
                    return;
                }
                did_charge_magic = true;
            }

            self.player_state_view_mut()
                .set_item_action_debug_value_2(1);
            if self.ancilla_add_somaria_block(0x2c, 1).is_none() {
                if did_charge_magic || !self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES)
                {
                    self.refund_magic(4);
                }
            }
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(ROD_ANIM_DELAYS[0]);
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().clear_item_in_hand();
            self.player_state_view_mut().set_position_mode_bits(8);
        }

        self.halt_link_when_using_items();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.player_state_view_mut()
            .increment_action_handler_timer();
        let step = self.player_state_view().action_handler_timer() as usize;
        if step < ROD_ANIM_DELAYS.len() {
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(ROD_ANIM_DELAYS[step]);
            return;
        }
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().clear_action_handler_timer();
        self.player_state_view_mut().set_spin_attack_delay_timer(0);
        self.player_state_view_mut()
            .set_item_action_debug_value_2(0);
        self.player_state_view_mut().clear_position_mode_bits(8);
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
    }

    pub(super) fn link_item_cane_of_byrna(&mut self) {
        const BYRNA_DELAYS: [u8; 4] = [19, 7, 13, 32];
        if self.search_for_byrna_spark() {
            return;
        }
        if self.player_state_view().button_mask_b_y() & 0x40 == 0 {
            if self.player_state_view().doorway_state() != 0 || !self.check_y_button_press() {
                return;
            }
            if !self.link_check_magic_cost(8) {
                self.finish_byrna_item();
                return;
            }
            self.ancilla_add_cane_of_byrna_init_spark(0x30, 0);
            self.player_state_view_mut()
                .clear_spin_attack_step_counter();
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(BYRNA_DELAYS[0]);
            self.player_state_view_mut().clear_item_action_step_var();
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_position_mode(8);
            self.player_state_view_mut().set_direction_lock_bits(1);
            self.player_state_view_mut().clear_animation_step();
        }

        self.halt_link_when_using_items();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.player_state_view_mut()
            .increment_action_handler_timer();
        let step = self.player_state_view().action_handler_timer() as usize;
        self.player_state_view_mut()
            .set_spin_attack_delay_timer(BYRNA_DELAYS[step]);
        if self.player_state_view().action_handler_timer() == 1 {
            self.ancilla_sfx3_near(42);
        } else if self.player_state_view().action_handler_timer() == 3 {
            self.finish_byrna_item();
        }
    }

    pub(super) fn link_item_net(&mut self) {
        const BUG_NET_TIMERS: [u8; 40] = [
            11, 6, 7, 8, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 9, 4, 5, 6, 7, 8, 1, 2, 3,
            4, 10, 8, 1, 2, 3, 4, 5, 6, 7, 8,
        ];
        let base = self.player_state_view().facing_index() * 10;
        if self.player_state_view().button_mask_b_y() & 0x40 == 0 {
            if self.player_state_view().doorway_state() != 0 || !self.check_y_button_press() {
                return;
            }
            self.player_state_view_mut()
                .set_action_handler_timer(BUG_NET_TIMERS[base]);
            self.player_state_view_mut().set_spin_attack_delay_timer(3);
            self.player_state_view_mut().clear_item_action_step_var();
            self.player_state_view_mut().set_position_mode(16);
            self.player_state_view_mut().set_direction_lock_bits(1);
            self.player_state_view_mut().clear_animation_step();
            self.ancilla_sfx2_near(50);
        }

        self.halt_link_when_using_items();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.player_state_view_mut()
            .increment_item_action_step_var();
        self.player_state_view_mut().set_spin_attack_delay_timer(3);
        if self.player_state_view().item_action_step_var() == 10 {
            self.player_state_view_mut().clear_item_action_step_var();
            self.player_state_view_mut().clear_action_handler_timer();
            let button_mask_b_y = self.player_state_view().button_mask_b_y() & 0x80;
            self.player_state_view_mut()
                .set_button_mask_b_y(button_mask_b_y);
            self.player_state_view_mut().clear_position_mode();
            self.player_state_view_mut().clear_direction_lock_bits(1);
            self.player_state_view_mut().disable_oam_offsets();
            return;
        }

        let index = base + self.player_state_view().item_action_step_var() as usize;
        self.player_state_view_mut()
            .set_action_handler_timer(BUG_NET_TIMERS[index]);
    }

    pub(super) fn ancilla_add_dug_up_flute(&mut self, ty: u8, limit: u8) {
        let Some(k) = self.ancilla_add_simple(ty, limit) else {
            return;
        };
        let x_velocity = if self.player_state_view().facing() == 4 {
            (-8i8) as u8
        } else {
            8
        };
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_z(0);
            ancilla.set_z_velocity(24);
            ancilla.set_x_velocity(x_velocity);
            ancilla.set_step(0);
        }
        self.DecodeAnimatedSpriteTile_variable(12);
        self.ancilla_set_xy(k, 0x0490, 0x0a8a);
    }

    pub(super) fn ancilla_add_cane_of_byrna_init_spark(&mut self, ty: u8, limit: u8) {
        for k in (0..5).rev() {
            if self.ancilla_slot_view(k).ancilla_type() == 0x31 {
                self.ancilla_slot_view_mut(k).set_ancilla_type(0);
            }
        }
        if let Some(k) = self.ancilla_add_simple(ty, limit) {
            let mut spark = self.ancilla_slot_view_mut(k);
            spark.set_item_to_link(0);
            spark.set_aux_timer(9);
            spark.set_work_byte_3(2);
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
        }
    }

    pub(super) fn ancilla_add_shovel_dirt(&mut self, ty: u8, limit: u8) {
        if let Some(k) = self.ancilla_add_simple(ty, limit) {
            let mut dirt = self.ancilla_slot_view_mut(k);
            dirt.set_item_to_link(0);
            dirt.set_timer(20);
            self.ancilla_set_xy(
                k,
                self.player_state_view().x(),
                self.player_state_view().y(),
            );
        }
    }

    fn start_medallion_item(
        &mut self,
        player_state: u8,
        delay: u8,
        spin_state: u8,
        quake: Option<()>,
    ) {
        if !self.check_y_button_press() {
            return;
        }
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);

        let sword_ok = self.inventory_items().sword_type().wrapping_add(1) & !1 != 0;
        let blocked = self.player_state_view().doorway_state() != 0
            || self.player_state_view().is_menu_blocked()
            || self.dungeon_state_view().savegame_state_bits() & 0x8000 != 0
            || !sword_ok
            || (self.follower_state_view().dropped() != 0
                && self.follower_state_view().indicator() == 13);
        if blocked {
            self.ancilla_sfx2_near(60);
            return;
        }

        if (0..3).any(|i| self.ancilla_slot_view(i).ancilla_type() != 0) {
            return;
        }
        if !self.link_check_magic_cost(1) {
            return;
        }

        self.player_state_view_mut().set_handler_state(player_state);
        self.player_state_view_mut().set_direction_lock_bits(1);
        self.player_state_view_mut()
            .set_spin_attack_delay_timer(delay);
        self.player_state_view_mut()
            .set_state_for_spin_attack(spin_state);
        self.player_state_view_mut()
            .clear_spin_animation_step_counter();
        self.player_state_view_mut().set_spin_attack_sound_latch(0);
        if quake.is_some() {
            self.player_state_view_mut()
                .set_actual_z_velocity_mirror_and_copy(40);
            self.player_state_view_mut().clear_z_mirror_low();
        }
        self.ancilla_sfx3_near(35);
    }

    fn start_mirror_transition(&mut self, submodule: u8) {
        self.set_submodule(submodule);
        self.player_state_view_mut()
            .clear_pull_for_rupees_sprite_need();
        self.player_state_view_mut().set_whirlpool_trigger();
        self.set_subsubmodule(0);
        self.player_state_view_mut().clear_actual_velocity_xy();
        self.player_state_view_mut().set_handler_state(20);
    }

    pub(super) fn ancilla_add_hookshot(&mut self, a: u8, y: u8) {
        self.ancilla_add_hookshot_inner(a, y);
    }

    fn ancilla_add_hookshot_inner(&mut self, a: u8, y: u8) -> Option<usize> {
        const HOOKSHOT_Y_VEL: [u8; 4] = [0xc0, 0x40, 0, 0];
        const HOOKSHOT_X_VEL: [u8; 4] = [0, 0, 0xc0, 0x40];
        const HOOKSHOT_Y_DELTA: [i16; 4] = [4, 20, 8, 8];
        const HOOKSHOT_X_DELTA: [i16; 4] = [0, 0, -4, 11];

        let k = self.ancilla_add_simple(a, y)?;
        self.player_state_view_mut().clear_hookshot_interlock();
        self.messaging_state_view_mut().set_effect_index(k as u8);
        let dir = self.player_state_view().facing() >> 1;
        {
            let mut hookshot = self.ancilla_slot_view_mut(k);
            hookshot.set_aux_timer(3);
            hookshot.set_step(0);
            hookshot.set_l(0);
            hookshot.set_k(0);
            hookshot.set_g(0xff);
            hookshot.set_work_byte_1(0);
            hookshot.set_item_to_link(0);
            hookshot.set_timer(0);
            hookshot.set_direction(dir);
            hookshot.set_x_velocity(HOOKSHOT_X_VEL[dir as usize]);
            hookshot.set_y_velocity(HOOKSHOT_Y_VEL[dir as usize]);
        }
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(HOOKSHOT_X_DELTA[dir as usize] as u16);
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(HOOKSHOT_Y_DELTA[dir as usize] as u16);
        self.ancilla_set_xy(k, x, y);
        Some(k)
    }

    fn finish_hookshot_state(&mut self) {
        self.finish_hookshot_state_without_button_clamp();
        if self.player_state_view().button_b_frames() >= 9 {
            self.player_state_view_mut().set_button_b_frames(9);
        }
    }

    fn finish_hookshot_state_without_button_clamp(&mut self) {
        self.player_state_view_mut().clear_handler_state();
        self.player_state_view_mut().clear_action_handler_timer();
        self.player_state_view_mut().set_spin_attack_delay_timer(0);
        self.player_state_view_mut().clear_hookshot_interlock();
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.player_state_view_mut().clear_direction_lock_bits(1);
        self.player_state_view_mut().clear_position_mode_bits(4);
        self.player_state_view_mut()
            .clear_sprite_damage_disable_timer();
    }

    fn finish_byrna_item(&mut self) {
        self.player_state_view_mut().clear_item_action_step_var();
        self.player_state_view_mut().clear_action_handler_timer();
        let button_mask_b_y = self.player_state_view().button_mask_b_y() & 0x80;
        self.player_state_view_mut()
            .set_button_mask_b_y(button_mask_b_y);
        self.player_state_view_mut().clear_position_mode();
        self.player_state_view_mut().clear_direction_lock_bits(1);
    }

    fn finish_powder_item(&mut self) {
        self.player_state_view_mut().clear_item_in_hand();
        self.player_state_view_mut().clear_action_handler_timer();
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
    }

    pub(super) fn link_reset_sword_and_item_usage(&mut self) {
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().and_defense_flags(!9);
        self.player_state_view_mut().set_spin_attack_delay_timer(0);
        self.player_state_view_mut().set_button_b_frames(0);
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x81);
        self.player_state_view_mut().clear_direction_lock_bits(1);
    }
}

impl ZeldaState {
    pub(super) fn cache_camera_properties(&mut self) {
        self.ppu_scroll_copy_view_mut().cache_bg2_live_scroll();
        self.player_state_view_mut().cache_current_position();
        let y_start = self.room_bounds_view().y_bound(0);
        let y_end = self.room_bounds_view().y_bound(2);
        let x_start = self.room_bounds_view().x_bound(0);
        let x_end = self.room_bounds_view().x_bound(2);
        self.dungeon_state_view_mut()
            .set_cached_room_bounds(y_start, y_end, x_start, x_end);
        copy_le_u16(
            &mut self.ram,
            UP_DOWN_SCROLL_TARGET_CACHED,
            UP_DOWN_SCROLL_TARGET,
        );
        copy_le_u16(
            &mut self.ram,
            UP_DOWN_SCROLL_TARGET_END_CACHED,
            UP_DOWN_SCROLL_TARGET_END,
        );
        copy_le_u16(
            &mut self.ram,
            LEFT_RIGHT_SCROLL_TARGET_CACHED,
            LEFT_RIGHT_SCROLL_TARGET,
        );
        copy_le_u16(
            &mut self.ram,
            LEFT_RIGHT_SCROLL_TARGET_END_CACHED,
            LEFT_RIGHT_SCROLL_TARGET_END,
        );
        copy_le_u16(
            &mut self.ram,
            CAMERA_Y_COORD_SCROLL_LOW_CACHED,
            CAMERA_Y_COORD_SCROLL_LOW,
        );
        copy_le_u16(
            &mut self.ram,
            CAMERA_X_COORD_SCROLL_LOW_CACHED,
            CAMERA_X_COORD_SCROLL_LOW,
        );
        copy_le_u16(
            &mut self.ram,
            QUADRANT_FULLSIZE_X_CACHED,
            QUADRANT_FULLSIZE_X,
        );
        self.player_state_view_mut().cache_current_quadrants();
        self.player_state_view_mut().cache_facing();
        self.player_state_view_mut().cache_lower_level_states();
        let doorway_state = self.player_state_view().doorway_state();
        self.dungeon_state_view_mut()
            .set_standing_in_doorway_cached(doorway_state);
        self.dungeon_state_view_mut().cache_current_floor();
    }

    pub(super) fn link_main(&mut self) {
        self.player_state_view_mut()
            .cache_previous_position_from_current_xy_order();
        self.clear_modal_pause_flag();
        if !self.player_state_view().is_immobilized() {
            self.link_control_handler();
        }
        self.handle_somaria_and_graves();
    }

    pub(super) fn link_control_handler(&mut self) {
        if self.player_state_view().given_damage() != 0 {
            if self.player_state_view().is_cape_active() {
                self.player_state_view_mut().clear_given_damage();
                self.player_state_view_mut().clear_auxiliary_state();
                self.player_state_view_mut().set_incapacitated_timer(0);
            } else if self.player_state_view().sprite_damage_disable_timer() == 0 {
                let dmg = self.player_state_view().given_damage();
                self.player_state_view_mut().clear_given_damage();
                if self.ancilla_slot_view(0).ancilla_type() == 5
                    && self.player_state_view().action_handler_timer() == 0
                    && self.player_state_view().spin_attack_delay_timer() != 0
                {
                    self.ancilla_slot_view_mut(0).set_ancilla_type(0);
                    self.minigame_state_view_mut()
                        .clear_flag_boomerang_in_place();
                }
                if self.player_state_view().blink_countdown() == 0 {
                    self.player_state_view_mut().set_blink_countdown(58);
                }
                self.ancilla_sfx2_near(38);
                self.sprite_battle_view_mut()
                    .increment_times_hurt_by_sprites();
                let new_dmg = self
                    .player_resources_view()
                    .current_health()
                    .wrapping_sub(dmg);
                let new_dmg = if new_dmg == 0 || new_dmg >= 0xa8 {
                    let main_layers = self.display_state().main_screen_layers;
                    let sub_layers = self.display_state().sub_screen_layers;
                    self.ppu_scroll_copy_view_mut().set_mapbak_tm(main_layers);
                    self.ppu_scroll_copy_view_mut().set_mapbak_ts(sub_layers);
                    let main_module = self.frame_state().main_module;
                    self.set_saved_module_for_menu(main_module);
                    self.set_main_module(18);
                    self.set_submodule(1);
                    self.player_state_view_mut().clear_blink_countdown();
                    self.player_resources_view_mut().set_heart_filler(0);
                    0
                } else {
                    new_dmg
                };
                self.player_resources_view_mut().set_current_health(new_dmg);
            }
        }
        if self.player_state_view().handler_state() != 0 {
            self.player_check_handle_cape_stuff();
        }
        match self.player_state_view().handler_state() {
            0 => self.link_state_default(),
            1 => self.link_state_pits(),
            2 | 6 => self.link_state_recoil(),
            3 | 30 => self.link_state_spin_attack(),
            4 => self.player_handler_04_swimming(),
            5 => self.link_state_on_ice(),
            7 => self.link_state_zapped(),
            8 => self.link_state_using_ether(),
            9 => self.link_state_using_bombos(),
            10 => self.link_state_using_quake(),
            11 => self.link_hop_hopping_south_ow(),
            12 => self.link_state_hopping_horizontally_ow(),
            13 => self.link_state_hopping_diagonally_up_ow(),
            14 => self.link_state_hopping_diagonally_down_ow(),
            15 | 16 => self.link_state_0_f(),
            17 => self.link_state_dashing(),
            18 => self.link_state_exiting_dash(),
            19 => self.link_state_hookshotting(),
            20 => self.link_state_crossing_worlds(),
            21 => self.player_handler_15_hold_item(),
            22 => self.link_state_sleeping(),
            23 => self.player_handler_17_bunny(),
            24 => self.link_state_holding_big_rock(),
            25 => self.link_state_receiving_ether(),
            26 => self.link_state_receiving_bombos(),
            27 => self.link_state_reading_desert_tablet(),
            28 => self.link_state_temporary_bunny(),
            29 => self.link_state_tree_pull(),
            _ => {}
        }
    }

    pub(super) fn link_state_default(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.link_handle_bunny_transformation() {
            if self.player_state_view().handler_state() == 23 {
                self.player_handler_17_bunny();
            }
            return;
        }
        self.player_state_view_mut().set_pit_correction_timer(0);
        if self.player_state_view().has_auxiliary_state() {
            self.handle_link_from1_d();
        } else {
            self.player_handler_00_ground_3();
        }
    }

    pub(super) fn handle_link_from1_d(&mut self) {
        self.player_state_view_mut().clear_item_in_hand();
        self.player_state_view_mut().clear_position_mode();
        self.player_state_view_mut().clear_action_scratch_state();
        self.player_state_view_mut().set_y_button_action_step(0);
        self.player_state_view_mut().set_y_button_action_flags(0);
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.player_state_view_mut()
            .clear_state_item_and_grab_flags();
        self.player_state_view_mut().clear_defense_flags();
        self.link_reset_swimming_state();
        self.player_state_view_mut().clear_direction_lock_bits(1);
        self.player_state_view_mut().clear_z_high();
        if self.player_state_view().electrocute_on_touch() != 0 {
            if self.player_state_view().is_cape_active() {
                self.link_force_unequip_cape_quietly();
            }
            self.link_reset_sword_and_item_usage();
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_spin_attack_delay_timer(2);
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().clear_direction_flags(0x0f);
            self.ancilla_sfx3_near(43);
            self.player_state_view_mut().set_handler_state(7);
            self.link_state_zapped();
        } else {
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
            self.player_state_view_mut().set_handler_state(2);
            self.link_state_recoil();
        }
    }

    pub(super) fn link_state_0_f(&mut self) {
        // LinkState_0F is an assert-only unreachable state in the C port.
        panic!("LinkState_0F reached");
    }

    pub(super) fn player_handler_15_hold_item(&mut self) {}

    pub(super) fn link_handle_bunny_transformation(&mut self) -> bool {
        if self.player_state_view().temp_bunny_timer() == 0 {
            return false;
        }

        if !self.player_state_view().needs_transform_poof() {
            if matches!(self.player_state_view().handler_state(), 23 | 28) {
                self.player_state_view_mut().clear_temp_bunny_timer();
                return false;
            }
            if self.player_state_view().picking_throw_state_has(2) {
                self.player_state_view_mut().clear_state_bits();
            }
            let preserved_lift_bit = self.player_state_view().state_bits() & 0x80;
            self.link_reset_properties_a();
            self.player_state_view_mut()
                .set_state_bits(preserved_lift_bit);

            for i in 0..5 {
                if matches!(self.ancilla_slot_view(i).ancilla_type(), 0x30 | 0x31) {
                    self.ancilla_slot_view_mut(i).set_ancilla_type(0);
                }
            }
            self.link_cancel_dash();
            self.ancilla_add_cape_poof(0x23, 4);
            self.ancilla_sfx2_near(0x14);
            self.player_state_view_mut().set_cape_transform_timer(20);
            self.player_state_view_mut().start_bunny_transform_poof();
        }

        if (self.player_state_view_mut().tick_cape_transform_timer() as i8).is_negative() {
            self.player_state_view_mut().set_handler_state(28);
            self.player_state_view_mut().finish_bunny_transform_poof();
            self.load_gear_palettes_bunny();
        }
        true
    }

    pub(super) fn link_state_temporary_bunny(&mut self) {
        let timer = self.player_state_view().temp_bunny_timer();
        if timer == 0 {
            self.ancilla_add_cape_poof(0x23, 4);
            self.ancilla_sfx2_near(0x15);
            self.player_state_view_mut().set_cape_transform_timer(32);
            self.player_state_view_mut().clear_handler_state();
            self.link_reset_properties_c();
            self.player_state_view_mut().clear_bunny_transform_flags();
            self.load_actual_gear_palettes();
            self.link_state_default();
        } else {
            self.player_state_view_mut().decrement_temp_bunny_timer();
            self.player_handler_17_bunny();
        }
    }

    pub(super) fn player_handler_17_bunny(&mut self) {
        self.cache_camera_properties_if_outdoors();
        self.player_state_view_mut().set_pit_correction_timer(0);
        if !self.player_state_view().is_in_deep_water() {
            if !self.player_state_view().has_auxiliary_state() {
                self.link_temp_bunny_func2();
                return;
            }
            if self.player_state_view().has_moon_pearl() {
                self.player_state_view_mut().clear_bunny_mirror();
            }
        }
        self.link_state_bunny_recache();
    }

    pub(super) fn link_temp_bunny_func2(&mut self) {
        if self.player_state_view().incapacitated_timer() != 0 {
            self.link_handle_recoil_and_timer(false);
            return;
        }
        self.player_state_view_mut().set_z(0xffff);
        self.player_state_view_mut().set_actual_z_velocity(0xff);
        self.player_state_view_mut().set_recoil_timer(0);
        if self.player_state_view().flag_moving() != 0 {
            self.swim_acceleration_view_mut().set_max_speed(0, 0x0180);
            self.swim_acceleration_view_mut().set_max_speed(2, 0x0180);
            self.link_handle_swim_movements();
            return;
        }

        self.reset_all_acceleration();
        self.link_handle_y_item();
        let mut dir = (self.player_state_view().force_move_any_direction() as u8) & 0x0f;
        if dir == 0 {
            dir = self.player_state_view().joypad1h_last() & 0x0f;
        }
        if dir == 0 {
            self.player_state_view_mut()
                .clear_movement_velocity_and_direction();
            self.player_state_view_mut().set_last_direction(0);
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().and_defense_flags(!9);
            self.player_state_view_mut().reset_push_fatigue_timer();
            self.player_state_view_mut().reset_jump_ledge_timer();
        } else {
            self.player_state_view_mut().set_direction(dir);
            if dir != self.player_state_view().last_direction() {
                self.player_state_view_mut().set_last_direction(dir);
                self.player_state_view_mut().clear_movement_subpixels();
                self.player_state_view_mut()
                    .clear_moving_against_diag_tile();
                self.player_state_view_mut().clear_defense_flags();
                self.player_state_view_mut().reset_push_fatigue_timer();
                self.player_state_view_mut().reset_jump_ledge_timer();
            }
        }
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.player_state_view_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_state_holding_big_rock(&mut self) {
        if self.player_state_view().has_auxiliary_state() {
            self.player_state_view_mut().clear_item_in_hand();
            self.player_state_view_mut().clear_position_mode();
            self.player_state_view_mut().clear_action_scratch_state();
            self.player_state_view_mut().set_y_button_action_step(0);
            self.player_state_view_mut().set_y_button_action_flags(0);
            self.player_state_view_mut()
                .clear_state_item_and_grab_flags();
            self.player_state_view_mut().clear_defense_flags();
            self.player_state_view_mut().clear_direction_lock_bits(1);
            self.player_state_view_mut().set_z_low(0);
            if self.player_state_view().electrocute_on_touch() != 0 {
                self.link_reset_sword_and_item_usage();
                self.player_state_view_mut()
                    .set_sprite_damage_disable_timer(1);
                self.player_state_view_mut().clear_action_handler_timer();
                self.player_state_view_mut().set_spin_attack_delay_timer(2);
                self.player_state_view_mut().clear_animation_step();
                self.player_state_view_mut().clear_direction_flags(0x0f);
                self.ancilla_sfx3_near(43);
                self.player_state_view_mut().set_handler_state(7);
                self.link_state_zapped();
            } else {
                self.player_state_view_mut().set_handler_state(2);
                self.link_state_recoil();
            }
            return;
        }

        self.player_state_view_mut().set_z(0xffff);
        self.player_state_view_mut().set_actual_z_velocity(0xff);
        self.player_state_view_mut().set_recoil_timer(0);
        if self.player_state_view().incapacitated_timer() != 0 {
            self.player_state_view_mut()
                .clear_lift_throw_scratch_state();
            self.player_state_view_mut().set_y_button_action_step(0);
            self.player_state_view_mut().set_y_button_action_flags(0);
            self.player_state_view_mut()
                .clear_state_item_and_grab_flags();
            if self.player_state_view().button_mask_b_y() & 0x80 == 0 {
                self.player_state_view_mut().clear_direction_lock_bits(1);
            }
            self.link_handle_recoil_and_timer(false);
            return;
        }

        self.link_handle_a_press();
        let dir = self.player_state_view().joypad1h_last() & 0x0f;
        if dir == 0 {
            self.player_state_view_mut()
                .clear_movement_velocity_and_direction();
            self.player_state_view_mut().set_last_direction(0);
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().and_defense_flags(!9);
            self.player_state_view_mut().reset_push_fatigue_timer();
            self.player_state_view_mut().reset_jump_ledge_timer();
        } else {
            self.player_state_view_mut().set_direction(dir);
            if dir != self.player_state_view().last_direction() {
                self.player_state_view_mut().set_last_direction(dir);
                self.player_state_view_mut().clear_movement_subpixels();
                self.player_state_view_mut()
                    .clear_moving_against_diag_tile();
                self.player_state_view_mut().clear_defense_flags();
                self.player_state_view_mut().reset_push_fatigue_timer();
                self.player_state_view_mut().reset_jump_ledge_timer();
            }
        }
        self.link_handle_moving_animation_full_long_entry();
        self.player_state_view_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn handle_somaria_and_graves(&mut self) {
        if self.world_location_state().is_outdoors()
            && self.player_state_view().hookshot_grave_latch()
        {
            for i in (0..5).rev() {
                if self.ancilla_slot_view(i).ancilla_type() == 0x24 {
                    self.gravestone_move(i);
                }
            }
        }
        for i in (0..5).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x2c {
                self.somaria_block_handle_player_interaction(i);
                return;
            }
        }
    }

    pub(super) fn link_state_on_ice(&mut self) {
        // LinkState_OnIce is an assert-only unreachable state in the C port.
        panic!("LinkState_OnIce reached");
    }

    pub(super) fn link_state_receiving_ether(&mut self) {
        self.player_state_view_mut().clear_auxiliary_state();
        self.player_state_view_mut().set_incapacitated_timer(0);
        self.player_state_view_mut().clear_given_damage();
        let frames = self
            .player_state_view_mut()
            .decrement_button_b_frames_word();
        if (frames as i16).is_negative() {
            self.player_state_view_mut().set_button_b_frames_word(0);
            self.player_state_view_mut().set_spin_attack_delay_timer(0);
        } else if frames == 0xbf {
            self.player_state_view_mut().force_hold_sword_up();
        } else if frames == 160 {
            let x = self.player_state_view().x();
            let y = self.player_state_view().y();
            self.player_state_view_mut().set_x(0x06b0);
            self.player_state_view_mut().set_y(0x0037);
            self.ancilla_add_ether_spell(0x18, 0);
            self.player_state_view_mut().set_x(x);
            self.player_state_view_mut().set_y(y);
        } else if frames == 0 {
            self.ancilla_add_falling_prize(0x29, 0, 4);
            self.player_state_view_mut().immobilize();
            self.player_state_view_mut().clear_menu_block();
        }
    }

    pub(super) fn link_state_receiving_bombos(&mut self) {
        self.player_state_view_mut().clear_auxiliary_state();
        self.player_state_view_mut().set_incapacitated_timer(0);
        self.player_state_view_mut().clear_given_damage();
        let frames = self
            .player_state_view_mut()
            .decrement_button_b_frames_word();
        if (frames as i16).is_negative() {
            self.player_state_view_mut().set_button_b_frames_word(0);
            self.player_state_view_mut().set_spin_attack_delay_timer(0);
        } else if frames == 223 {
            self.player_state_view_mut().force_hold_sword_up();
        } else if frames == 160 {
            let x = self.player_state_view().x();
            let y = self.player_state_view().y();
            self.player_state_view_mut().set_x(0x0378);
            self.player_state_view_mut().set_y(0x0eb0);
            self.ancilla_add_bombos_spell(0x19, 0);
            self.player_state_view_mut().set_x(x);
            self.player_state_view_mut().set_y(y);
        } else if frames == 0 {
            self.ancilla_add_falling_prize(0x29, 5, 4);
            self.player_state_view_mut().immobilize();
        }
    }

    pub(super) fn ether_tablet_start_cutscene(&mut self) {
        self.player_state_view_mut()
            .set_button_b_frames_word(0x00c0);
        self.player_state_view_mut().set_spin_attack_delay_timer(0);
        self.player_state_view_mut().set_handler_state(25);
        self.player_state_view_mut()
            .set_sprite_damage_disable_timer(1);
        self.player_state_view_mut().set_menu_block_flag(1);
    }

    pub(super) fn bombos_tablet_start_cutscene(&mut self) {
        self.player_state_view_mut()
            .set_button_b_frames_word(0x00e0);
        self.player_state_view_mut().set_spin_attack_delay_timer(0);
        self.player_state_view_mut().set_handler_state(26);
        self.player_state_view_mut()
            .set_sprite_damage_disable_timer(1);
        self.player_state_view_mut()
            .set_custom_spell_animation_active();
    }

    pub(super) fn link_state_reading_desert_tablet(&mut self) {
        let button_b_frames = self.player_state_view().button_b_frames().wrapping_sub(1);
        self.player_state_view_mut()
            .set_button_b_frames(button_b_frames);
        if self.player_state_view().button_b_frames() == 0 {
            self.player_state_view_mut().clear_handler_state();
            self.link_perform_desert_prayer();
        }
    }

    pub(super) fn link_state_pits(&mut self) {
        self.player_state_view_mut().set_direction(0);
        if self.player_state_view().pit_correction_active() && {
            self.player_state_view_mut()
                .increment_pit_correction_timer();
            self.player_state_view().pit_correction_timer() == 0x20
        } {
            self.player_state_view_mut().set_pit_correction_timer(31);
        } else {
            if !self.player_state_view().is_running() {
                if !self.player_state_view().is_in_auxiliary_state(1) {
                    let direction = self.player_state_view().joypad1h_last() & 0x0f;
                    self.player_state_view_mut().set_direction(direction);
                }
                self.link_state_pits_after_aux_state();
                return;
            }
            const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
            if self.player_state_view().dash_countdown() != 0
                && (!self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES)
                    || self.player_state_view().joypad1l_last() & 0x80 != 0)
            {
                self.link_state_dashing();
                return;
            }
            if self.player_state_view().joypad1h_last() & 0x0f != 0
                && (self.player_state_view().joypad1h_last()
                    & 0x0f
                    & self.player_state_view().direction())
                    == 0
            {
                self.link_cancel_dash();
                if !self.player_state_view().is_in_auxiliary_state(1) {
                    let direction = self.player_state_view().joypad1h_last() & 0x0f;
                    self.player_state_view_mut().set_direction(direction);
                }
            }
        }

        self.link_state_pits_after_aux_state();
    }

    pub(super) fn handle_dungeon_landing_from_pit(&mut self) {
        self.link_oam_main();
        self.player_state_view_mut()
            .cache_previous_position_from_current_xy_order();
        if self.frame_state().submodule == 7 {
            self.player_state_view_mut().set_visibility_status(0);
        }
        if self.frame_state().frame_counter & 3 == 0 {
            self.player_state_view_mut().advance_pit_data_index();
            if self.player_state_view().pit_data_index() == 10 {
                self.player_state_view_mut().set_pit_data_index(6);
            }
        }
        self.player_state_view_mut().set_direction(4);
        self.link_handle_velocity();

        let link_y = self.player_state_view().y();
        let target_y = self.tile_detect_position_view().y();
        if (link_y as i16).is_negative() && !(target_y as i16).is_negative() {
            if (!link_y).wrapping_add(1).wrapping_add(target_y) < 0x8000 {
                return;
            }
        } else if target_y >= link_y {
            return;
        }

        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES) {
            self.player_state_view_mut().clear_about_to_jump_off_ledge();
        }
        self.player_state_view_mut().set_y(target_y);
        self.player_state_view_mut().clear_animation_step();
        self.player_state_view_mut().clear_speed_modifier();
        self.player_state_view_mut().clear_pit_data_index();
        self.player_state_view_mut().clear_near_pit_state();
        self.player_state_view_mut().set_speed_setting(0);
        self.set_subsubmodule(0);
        self.set_submodule(0);
        self.player_state_view_mut()
            .clear_sprite_damage_disable_timer();
        if self.follower_state_view().indicator() != 0
            && self.follower_state_view().indicator() != 3
        {
            self.follower_state_view_mut().set_appearance_none_flag(0);
            if self.follower_state_view().indicator() == 13 {
                self.follower_state_view_mut().set_indicator(0);
                self.hud_state_view_mut().set_super_bomb_indicator_timer(0);
                self.hud_state_view_mut()
                    .set_super_bomb_indicator_counter(0);
                self.follower_state_view_mut().set_dropped(0);
            } else {
                self.follower_initialize();
            }
        }
        self.tile_detect_main_handler(0);
        if self.tile_detect_position_view().shallow_water() & 1 != 0 {
            self.ancilla_sfx2_near(0x24);
        }
        self.player_tile_detect_nearby();
        if self.system_signals_view().sound_effect_1() & 0x3f != 0x24 {
            self.ancilla_sfx2_near(0x21);
        }

        if self.dungeon_state_view().header_collision_2() == 2
            && self.tile_detect_position_view().water_staircase() & 0x0f != 0
        {
            self.player_state_view_mut().set_layer_collision_flags(
                crate::game_state::constants::player::LAYER_COLLISION_BOTH,
            );
        }
        if self.tile_detect_position_view().deepwater() & 0x0f == 0x0f {
            self.player_state_view_mut().enter_deep_water_state();
            self.player_state_view_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.player_state_view_mut().mark_lower_level();
            self.ancilla_add_splash(0x15, 1);
            self.player_state_view_mut().set_handler_state(4);
            self.link_force_unequip_cape_quietly();
            self.player_state_view_mut()
                .clear_state_item_and_grab_flags();
            self.player_state_view_mut().set_speed_setting(0);
        } else {
            let handler_state = if self.tile_detect_position_view().pit_tile() & 0x0f != 0 {
                1
            } else {
                0
            };
            self.player_state_view_mut()
                .set_handler_state(handler_state);
        }
    }

    pub(super) fn link_state_spin_attack(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.player_state_view().has_auxiliary_state() {
            for i in (0..5).rev() {
                if matches!(self.ancilla_slot_view(i).ancilla_type(), 0x2a | 0x2b) {
                    self.ancilla_slot_view_mut(i).set_ancilla_type(0);
                }
            }
            self.player_state_view_mut().clear_z_high();
            self.player_state_view_mut().clear_direction_lock_bits(1);
            self.player_state_view_mut().set_spin_attack_delay_timer(0);
            self.player_state_view_mut().set_button_b_frames(0);
            self.player_state_view_mut().set_button_mask_b_y(0);
            self.player_state_view_mut().set_y_button_action_flags(0);
            self.player_state_view_mut().set_state_for_spin_attack(0);
            self.player_state_view_mut()
                .clear_spin_animation_step_counter();
            self.player_state_view_mut().set_speed_setting(0);
            if self.player_state_view().electrocute_on_touch() != 0 {
                if self.player_state_view().is_cape_active() {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_reset_sword_and_item_usage();
                self.player_state_view_mut()
                    .set_sprite_damage_disable_timer(1);
                self.player_state_view_mut().clear_action_handler_timer();
                self.player_state_view_mut().set_spin_attack_delay_timer(2);
                self.player_state_view_mut().clear_animation_step();
                self.player_state_view_mut().clear_direction_flags(0x0f);
                self.ancilla_sfx3_near(43);
                self.player_state_view_mut().set_handler_state(7);
                self.link_state_zapped();
            } else {
                self.player_state_view_mut().set_handler_state(2);
                self.link_state_recoil();
            }
            return;
        }

        if self.player_state_view().incapacitated_timer() != 0 {
            self.link_handle_recoil_and_timer(false);
        } else {
            self.player_state_view_mut().set_direction(0);
            self.link_handle_velocity();
            self.link_handle_cardinal_collision();
            self.player_state_view_mut().set_handler_state(3);
            self.player_state_view_mut().clear_pit_correction();
            self.handle_indoor_camera_and_doors();
        }

        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.player_state_view_mut()
            .increment_spin_animation_step_counter();
        if self.player_state_view().spin_animation_step_counter() == 2 {
            self.ancilla_sfx3_near(35);
        }
        if self.player_state_view().spin_animation_step_counter() == 12 {
            self.player_state_view_mut().clear_direction_lock_bits(1);
            self.player_state_view_mut().set_spin_attack_delay_timer(0);
            self.player_state_view_mut().set_button_b_frames(0);
            self.player_state_view_mut().set_state_for_spin_attack(0);
            self.player_state_view_mut()
                .clear_spin_animation_step_counter();
            if self.player_state_view().handler_state() != 30 {
                let button_mask_b_y = if self.player_state_view().button_b_frames() != 0 {
                    self.player_state_view().joypad1h_last() & 0x80
                } else {
                    0
                };
                self.player_state_view_mut()
                    .set_button_mask_b_y(button_mask_b_y);
            }
            self.player_state_view_mut().clear_handler_state();
        } else {
            let idx =
                self.player_state_view()
                    .spin_animation_step_counter()
                    .wrapping_add(self.player_state_view().spin_offsets()) as usize;
            self.player_state_view_mut()
                .set_state_for_spin_attack(LINK_SPIN_GRAPHICS_BY_DIR[idx]);
            let delay =
                LINK_SPIN_DELAYS[self.player_state_view().spin_animation_step_counter() as usize];
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(delay);
            self.tile_detect_main_handler(8);
        }
    }

    pub(super) fn link_hop_hopping_south_ow(&mut self) {
        self.player_state_view_mut()
            .set_last_direction_moved_towards(1);
        self.player_state_view_mut().clear_direction_lock();
        self.player_state_view_mut().clear_actual_velocity_xy();
        self.player_state_view_mut()
            .clear_water_ripple_or_grass_state();
        if self.player_state_view().incapacitated_timer() == 0
            && self.player_state_view().actual_z_velocity_mirror() == 0
        {
            self.ancilla_sfx2_near(32);
            self.link_hop_find_tile_to_land_on_south();
            if self.world_location_state().is_outdoors() {
                self.player_state_view_mut().set_lower_level_state(2);
            }
        }

        self.player_state_view_mut().restore_z_from_mirror();
        self.player_state_view_mut()
            .restore_actual_z_velocity_from_mirror();
        self.player_state_view_mut().decrement_actual_z_velocity(2);
        self.link_move_position();

        if (self.player_state_view().actual_z_velocity() as i8).is_negative() {
            if self.player_state_view().actual_z_velocity() < 0xa0 {
                self.player_state_view_mut().set_actual_z_velocity(0xa0);
            }
            if self.player_state_view().is_landing_at_or_above_ground() {
                self.player_state_view_mut().set_z(0);
                self.link_splash_upon_landing();
                if self.player_state_view().is_near_pit() {
                    self.player_state_view_mut().set_handler_state(1);
                }
                if self.player_state_view().handler_state() != 4
                    && self.player_state_view().handler_state() != 1
                    && !self.player_state_view().is_in_deep_water()
                {
                    self.ancilla_sfx2_near(33);
                }
                self.player_state_view_mut()
                    .clear_sprite_damage_disable_timer();
                self.player_state_view_mut().set_allow_scroll_z(0);
                self.player_state_view_mut().clear_auxiliary_state();
                self.player_state_view_mut().set_actual_z_velocity(0xff);
                self.player_state_view_mut().set_z(0xffff);
                self.player_state_view_mut().set_incapacitated_timer(0);
                if self.world_location_state().is_outdoors() {
                    self.player_state_view_mut().clear_lower_level();
                }
            } else {
                let y_velocity = self.player_state_view().z_mirror_delta_low();
                self.player_state_view_mut().set_y_velocity(y_velocity);
            }
        } else {
            let y_velocity = self.player_state_view().z_mirror_delta_low();
            self.player_state_view_mut().set_y_velocity(y_velocity);
        }
        self.player_state_view_mut()
            .cache_actual_z_velocity_to_mirror();
        self.player_state_view_mut().cache_z_to_mirror();
    }

    pub(super) fn link_hop_find_tile_to_land_on_south(&mut self) {
        let original_y = self.player_state_view().y();
        self.player_state_view_mut()
            .set_hop_origin_coord(original_y);
        let y_velocity = self.player_state_view().y_low_delta_from_safe_return();
        self.player_state_view_mut().set_y_velocity(y_velocity);
        loop {
            let dir = self
                .player_state_view()
                .last_direction_moved_towards_index();
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(HOP_SOUTH_Y[dir] as i16 as u16);
            self.player_state_view_mut().set_y(y);
            self.tile_detect_movement_y(
                self.player_state_view()
                    .last_direction_moved_towards()
                    .into(),
            );
            let terrain = self.tile_detect_position_view().normal_tiles()
                | self.tile_detect_position_view().pit_tile_word()
                | self.tile_detect_position_view().destruction_aftermath()
                | self.tile_detect_position_view().thick_grass()
                | self.tile_detect_position_view().deepwater();
            if terrain & 7 == 7 {
                break;
            }
        }
        if self.tile_detect_position_view().deepwater() & 7 != 0 {
            self.player_state_view_mut().enter_deep_water_state();
            if !self.player_state_view().is_in_auxiliary_state(4) {
                self.player_state_view_mut().set_auxiliary_state(2);
            }
            self.player_state_view_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.player_state_view_mut().clear_grabbing_wall();
            self.player_state_view_mut().set_speed_setting(0);
        }
        if self.tile_detect_position_view().pit_tile_word() & 7 != 0 {
            self.player_state_view_mut().mark_pit_landing_oam_state();
            self.player_state_view_mut().begin_pit_check();
        }
        let dir = self
            .player_state_view()
            .last_direction_moved_towards_index();
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(HOP_SOUTH_Y2[dir] as i16 as u16);
        self.player_state_view_mut().set_y(y);
        self.player_state_view_mut().store_safe_return_y(y);
        self.player_state_view_mut().set_incapacitated_timer(1);
        let mut z = self.player_state_view().z_low();
        if z >= 0xf0 {
            z = 0;
        }
        let z = y.wrapping_sub(original_y).wrapping_add(z as u16);
        self.player_state_view_mut().set_z_and_mirror(z);
    }

    pub(super) fn link_state_hopping_horizontally_ow(&mut self) {
        let direction = if self
            .player_state_view()
            .actual_x_velocity_signed()
            .is_negative()
        {
            6
        } else {
            5
        };
        self.player_state_view_mut().set_direction(direction);
        self.player_state_view_mut().clear_direction_lock();
        self.player_state_view_mut().set_actual_y_velocity(0);
        self.player_state_view_mut()
            .clear_water_ripple_or_grass_state();
        self.link_state_handling_jump();
    }

    pub(super) fn link_hopping_horizontally_find_tile_y(&mut self) {
        let original_y = self.player_state_view().y();
        self.player_state_view_mut()
            .set_hop_origin_coord(original_y);
        let y_velocity = self.player_state_view().y_low_delta_from_safe_return();
        self.player_state_view_mut().set_y_velocity(y_velocity);

        let dir = self
            .player_state_view()
            .last_direction_moved_towards_index();
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(HOP_SOUTH_Y[dir] as i16 as u16);
        self.player_state_view_mut().set_y(y);
        self.tile_detect_movement_y(
            self.player_state_view()
                .last_direction_moved_towards()
                .into(),
        );

        let terrain = self.tile_detect_position_view().normal_tiles()
            | self.tile_detect_position_view().destruction_aftermath()
            | self.tile_detect_position_view().thick_grass()
            | self.tile_detect_position_view().deepwater();

        if terrain & 7 != 7 {
            self.player_state_view_mut().set_y(original_y);
            self.player_state_view_mut().set_incapacitated_timer(1);

            let org_velx = self.player_state_view().actual_x_velocity();
            let mut velx = org_velx as i8;
            if velx < 0 {
                velx = velx.wrapping_neg();
            }
            let idx = ((velx as u8) >> 4) as usize;
            self.player_state_view_mut()
                .set_actual_z_velocity_mirror_and_copy(HOP_HORIZ_VEL_Z[idx]);
            let mut xt = HOP_HORIZ_VEL_X[idx];
            if (org_velx as i8) < 0 {
                xt = 0u8.wrapping_sub(xt);
            }
            self.player_state_view_mut().set_actual_x_velocity(xt);
        } else {
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(HOP_SOUTH_Y2[dir] as i16 as u16);
            self.player_state_view_mut().set_y(y);
            self.player_state_view_mut().store_safe_return_y(y);
            self.player_state_view_mut().set_incapacitated_timer(1);
            let mut z = self.player_state_view().z_low();
            if z == 255 {
                z = 0;
            }
            let z = y.wrapping_sub(original_y).wrapping_add(z as u16);
            self.player_state_view_mut().set_z_and_mirror(z);
        }

        if self.tile_detect_position_view().deepwater() & 7 != 0 {
            self.player_state_view_mut().set_auxiliary_state(2);
            self.link_set_to_deep_water();
        }
    }

    pub(super) fn link_hopping_horizontally_find_tile_x(&mut self, o: u8) -> u8 {
        assert!(o == 0 || o == 2);
        let original_x = self.player_state_view().x();
        self.player_state_view_mut()
            .set_hop_origin_coord(original_x);
        let table_idx = (o >> 1) as usize;
        let mut i: i16 = 7;
        loop {
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(HOP_HORIZ_X_STEP[table_idx] as i16 as u16);
            self.player_state_view_mut().set_x(x);
            self.tile_detect_movement_x(
                self.player_state_view()
                    .last_direction_moved_towards()
                    .into(),
            );

            let terrain = self.tile_detect_position_view().normal_tiles()
                | self.tile_detect_position_view().destruction_aftermath()
                | self.tile_detect_position_view().thick_grass()
                | self.tile_detect_position_view().deepwater()
                | self.tile_detect_position_view().pit_tile_word();

            if terrain & 7 == 7 {
                if self.tile_detect_position_view().deepwater() & 7 == 7 {
                    self.player_state_view_mut().enter_deep_water_state();
                    self.player_state_view_mut().set_auxiliary_state(2);
                    self.player_state_view_mut()
                        .set_swim_flags_from_last_direction();
                    self.player_state_view_mut().clear_swimming_countdown();
                    self.player_state_view_mut().set_speed_setting(0);
                    self.player_state_view_mut().clear_grabbing_wall();
                    self.reset_all_acceleration();
                }
                break;
            }
            i -= 1;
            if i < 0 {
                let x = original_x.wrapping_add(HOP_HORIZ_X_FALLBACK[table_idx] as i16 as u16);
                self.player_state_view_mut().set_x(x);
                break;
            }
        }

        let x = self
            .player_state_view()
            .x()
            .wrapping_add(HOP_HORIZ_X_FINAL[table_idx] as i16 as u16);
        self.player_state_view_mut().set_x(x);
        let distance = original_x.wrapping_sub(x) as i16;
        let distance = if distance < 0 { -distance } else { distance };
        let idx = (distance as u16 >> 3) as usize;
        let mut velx = HOP_HORIZ_X_VEL[idx];
        if o != 2 {
            velx = 0u8.wrapping_sub(velx);
        }
        self.player_state_view_mut().set_actual_x_velocity(velx);
        self.player_state_view_mut()
            .set_actual_z_velocity_mirror_and_copy(HOP_HORIZ_Z_VEL[idx]);
        i as u8
    }

    pub(super) fn link_state_hopping_diagonally_up_ow(&mut self) {
        self.player_state_view_mut()
            .clear_water_ripple_or_grass_state();
        self.player_change_z(2);
        self.link_move_position();
        if self.player_state_view().is_z_low_negative() {
            self.link_splash_upon_landing();
            if self.player_state_view().handler_state() != 4
                && !self.player_state_view().is_in_deep_water()
            {
                self.ancilla_sfx2_near(33);
            }
            self.player_state_view_mut()
                .clear_sprite_damage_disable_timer();
            self.player_state_view_mut().clear_auxiliary_state();
            self.player_state_view_mut().set_actual_z_velocity(0xff);
            self.player_state_view_mut().set_z(0xffff);
            self.player_state_view_mut().set_incapacitated_timer(0);
            self.player_state_view_mut().clear_direction_lock();
        }
    }

    pub(super) fn link_state_hopping_diagonally_down_ow(&mut self) {
        let dir = if self
            .player_state_view()
            .actual_x_velocity_signed()
            .is_negative()
        {
            2
        } else {
            3
        };
        self.player_state_view_mut()
            .set_last_direction_moved_towards(dir);
        self.player_state_view_mut().clear_direction_lock();
        self.player_state_view_mut().set_actual_y_velocity(0);
        self.player_state_view_mut()
            .clear_water_ripple_or_grass_state();
        if self.player_state_view().incapacitated_timer() == 0
            && self.player_state_view().actual_z_velocity_mirror() == 0
        {
            self.player_state_view_mut()
                .set_last_direction_moved_towards(1);
            let old_x = self.player_state_view().x();
            self.ancilla_sfx2_near(32);
            self.link_hop_find_landing_spot_diagonally_down();
            self.player_state_view_mut().set_x(old_x);

            let distance = self
                .player_state_view()
                .y()
                .wrapping_sub(self.player_state_view().hop_origin_coord());
            let idx = ((distance >> 3) as usize).min(23);
            let mut velx = LEDGE_DOWN_X_VEL[idx];
            if dir == 2 {
                velx = 0u8.wrapping_sub(velx);
            }
            self.player_state_view_mut().set_actual_x_velocity(velx);
            if self.world_location_state().is_outdoors() {
                self.player_state_view_mut().set_lower_level_state(2);
            }
        }
        self.link_state_handling_jump();
    }

    pub(super) fn link_hop_find_landing_spot_diagonally_down(&mut self) {
        let original_y = self.player_state_view().y();
        self.player_state_view_mut()
            .set_hop_origin_coord(original_y);
        let y_velocity = self.player_state_view().y_low_delta_from_safe_return();
        self.player_state_view_mut().set_y_velocity(y_velocity);

        let scratch = loop {
            let o = if self
                .player_state_view()
                .actual_x_velocity_signed()
                .is_negative()
            {
                0
            } else {
                1
            };
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(LEDGE_DIAG_DX[o] as i16 as u16);
            self.player_state_view_mut().set_x(x);
            let dir = self
                .player_state_view()
                .last_direction_moved_towards_index();
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(LEDGE_DIAG_DY[dir] as i16 as u16);
            self.player_state_view_mut().set_y(y);
            self.tile_detect_movement_y(
                self.player_state_view()
                    .last_direction_moved_towards()
                    .into(),
            );
            let scratch = LEDGE_DIAG_BITS[o];
            let terrain = self.tile_detect_position_view().normal_tiles()
                | self.tile_detect_position_view().destruction_aftermath_low() as u16
                | self.tile_detect_position_view().thick_grass_low() as u16
                | self.tile_detect_position_view().deepwater();
            if terrain & scratch as u16 == scratch as u16 {
                break scratch;
            }
        };

        if self.tile_detect_position_view().deepwater() & scratch as u16 != 0 {
            self.player_state_view_mut().enter_deep_water_state();
            self.player_state_view_mut().set_auxiliary_state(2);
            self.player_state_view_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.player_state_view_mut().set_speed_setting(0);
            self.player_state_view_mut().clear_grabbing_wall();
        }

        let dir = self
            .player_state_view()
            .last_direction_moved_towards_index();
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(LEDGE_DIAG_DY2[dir] as i16 as u16);
        self.player_state_view_mut().set_y(y);
        self.player_state_view_mut().store_safe_return_y(y);
        self.player_state_view_mut().set_incapacitated_timer(1);
        let z = y
            .wrapping_sub(original_y)
            .wrapping_add(self.player_state_view().z_low() as u16);
        self.player_state_view_mut().set_z_and_mirror(z);
    }

    pub(super) fn link_state_handling_jump(&mut self) {
        self.player_state_view_mut()
            .restore_actual_z_velocity_from_mirror();
        self.player_state_view_mut().restore_z_low_from_mirror();
        self.player_state_view_mut().decrement_actual_z_velocity(2);
        self.link_move_position();
        if (self.player_state_view().actual_z_velocity() as i8).is_negative() {
            if self.player_state_view().actual_z_velocity() < 0xa0 {
                self.player_state_view_mut().set_actual_z_velocity(0xa0);
            }
            if self
                .player_state_view()
                .is_low_z_landing_at_or_above_ground()
            {
                self.player_state_view_mut().set_z(0);
                let mut falling_into_pit = false;
                if matches!(self.player_state_view().handler_state(), 12 | 14) {
                    self.tile_detect_main_handler(0);
                    if self.tile_detect_position_view().deepwater() as u8 & 1 != 0 {
                        self.player_state_view_mut().set_handler_state(4);
                        self.link_set_to_deep_water();
                        self.link_reset_sword_and_item_usage();
                        self.ancilla_add_splash(21, 0);
                    } else if self.tile_detect_position_view().pit_tile() & 1 != 0 {
                        self.player_state_view_mut().mark_pit_landing_oam_state();
                        self.player_state_view_mut().begin_pit_check();
                        self.player_state_view_mut().set_handler_state(1);
                        falling_into_pit = true;
                    }
                }
                if !falling_into_pit {
                    self.link_splash_upon_landing();
                    if self.player_state_view().handler_state() != 4
                        && !self.player_state_view().is_in_deep_water()
                    {
                        self.ancilla_sfx2_near(33);
                    }
                }
                if self.player_state_view().handler_state() != 4
                    || !self.player_state_view().is_bunny_mirror()
                {
                    self.player_state_view_mut()
                        .clear_sprite_damage_disable_timer();
                }
                self.player_state_view_mut().set_allow_scroll_z(0);
                self.player_state_view_mut().clear_auxiliary_state();
                self.player_state_view_mut().set_actual_z_velocity(0xff);
                self.player_state_view_mut().set_z(0xffff);
                self.player_state_view_mut().set_incapacitated_timer(0);
                if self.world_location_state().is_outdoors() {
                    self.player_state_view_mut().clear_lower_level();
                }
            } else {
                let y_velocity = self.player_state_view().z_mirror_delta_low();
                self.player_state_view_mut().set_y_velocity(y_velocity);
            }
        } else {
            let y_velocity = self.player_state_view().z_mirror_delta_low();
            self.player_state_view_mut().set_y_velocity(y_velocity);
        }
        self.player_state_view_mut()
            .cache_actual_z_velocity_to_mirror();
        self.player_state_view_mut().cache_z_low_to_mirror();
    }

    pub(super) fn link_state_dashing(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.link_handle_bunny_transformation() {
            if self.player_state_view().handler_state() == 23 {
                self.player_handler_17_bunny();
            }
            return;
        }
        if !self.player_state_view().is_running() {
            self.player_state_view_mut()
                .clear_sprite_damage_disable_timer();
            self.player_state_view_mut().set_dash_countdown(0);
            self.player_state_view_mut().set_speed_setting(0);
            self.player_state_view_mut().clear_handler_state();
            self.player_state_view_mut().clear_direction_lock();
            return;
        }
        if self.player_state_view().button_mask_b_y() & 0x80 != 0
            && self.player_state_view().button_b_frames() >= 9
        {
            self.player_state_view_mut().set_button_b_frames(9);
        }
        self.player_state_view_mut().set_pit_correction_timer(0);

        if self.player_state_view().has_auxiliary_state() {
            self.player_state_view_mut()
                .clear_sprite_damage_disable_timer();
            self.player_state_view_mut().set_dash_countdown(0);
            self.player_state_view_mut().set_speed_setting(0);
            self.player_state_view_mut().clear_direction_lock();
            self.player_state_view_mut().clear_running();
            self.player_state_view_mut().clear_defense_flags();
            if self.player_state_view().electrocute_on_touch() != 0 {
                if self.player_state_view().is_cape_active() {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_reset_sword_and_item_usage();
                self.player_state_view_mut()
                    .set_sprite_damage_disable_timer(1);
                self.player_state_view_mut().clear_action_handler_timer();
                self.player_state_view_mut().set_spin_attack_delay_timer(2);
                self.player_state_view_mut().clear_animation_step();
                self.player_state_view_mut().clear_direction_flags(0x0f);
                self.ancilla_sfx3_near(43);
                self.player_state_view_mut().set_handler_state(7);
                self.link_state_zapped();
            } else {
                self.player_state_view_mut().set_handler_state(2);
                self.link_state_recoil();
            }
            return;
        }

        const DASH_SFX_TRIGGER_MASKS: [u8; 3] = [7, 15, 15];
        const DASH_DIRECTION_BITS_BY_FACING: [u8; 4] = [8, 4, 2, 1];
        let mut a = self.player_state_view().dash_countdown();
        if a == 0 {
            a = self.player_state_view().index_of_dashing_sfx();
            self.player_state_view_mut()
                .decrement_index_of_dashing_sfx();
        }
        if DASH_SFX_TRIGGER_MASKS[(self.player_state_view().dash_countdown() >> 4) as usize] & a
            == 0
        {
            self.ancilla_sfx2_near(35);
        }
        if (self.player_state_view_mut().decrement_dash_countdown() as i8).is_negative() {
            self.player_state_view_mut().set_dash_countdown(0);
            let follower = self.follower_state_view().indicator() as usize;
            if self.follower_state_view().indicator() == DASH_FOLLOWER_SLOWDOWN_INDICATORS[follower]
            {
                self.follower_state_view_mut()
                    .set_indicator(DASH_FOLLOWER_RELEASE_INDICATORS[follower]);
            }
        } else {
            self.player_state_view_mut().clear_index_of_dashing_sfx();
            if self.player_state_view().joypad1l_last() & 0x80 == 0 {
                self.player_state_view_mut().clear_animation_step();
                self.player_state_view_mut().set_dash_countdown(0);
                self.player_state_view_mut().set_speed_setting(0);
                self.player_state_view_mut().clear_handler_state();
                self.player_state_view_mut().clear_running();
                if self.player_state_view().button_mask_b_y() & 0x80 == 0 {
                    self.player_state_view_mut().clear_direction_lock();
                }
                return;
            }
            self.ancilla_add_dash_dust_charging(30, 0);
            self.player_state_view_mut().clear_movement_velocity();
            self.player_state_view_mut().prime_dash_counter();
            self.player_state_view_mut().set_speed_setting(16);
            let mut dir = self.player_state_view().joypad1h_last() & 0x0f;
            if self.player_state_view().button_mask_b_y() & 0x80 != 0
                || self.player_state_view().doorway_state() != 0
                || dir == 0
            {
                dir = DASH_DIRECTION_BITS_BY_FACING[self.player_state_view().facing_index()];
            }
            self.player_state_view_mut()
                .set_direction_and_last_direction(dir);
            self.player_state_view_mut()
                .set_swim_flags_from_last_direction();
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
            self.link_handle_moving_animation_full_long_entry();
            let org_x = self.player_state_view().x();
            let org_y = self.player_state_view().y();
            self.store_link_safe_return_position(org_x, org_y);
            self.link_handle_moving_floor();
            self.link_apply_conveyor();
            if self.player_state_view().has_somaria_platform_state() {
                self.link_handle_velocity_and_sand_drag(org_x, org_y);
            }
            let x = self.player_state_view().x();
            let y = self.player_state_view().y();
            self.player_state_view_mut()
                .set_movement_velocity_from_position_delta(x, y, org_x, org_y);
            self.link_handle_cardinal_collision();
            self.handle_indoor_camera_and_doors();
            return;
        }

        self.player_state_view_mut()
            .clear_animation_step_if_at_least(6);
        self.player_state_view_mut()
            .decrement_dash_counter_clamped_to_minimum(32);
        self.ancilla_add_dash_dust(30, 0);
        self.player_state_view_mut()
            .clear_spin_attack_step_counter();
        if self.inventory_items().sword_type().wrapping_add(1) & 0xfe != 0 {
            self.tile_detect_main_handler(7);
        }
        if self.save_progress_view().progress_indicator() != 0 {
            self.player_state_view_mut().add_button_mask_b_y_bits(0x80);
            self.player_state_view_mut().set_button_b_frames(9);
        }
        self.player_state_view_mut().set_incapacitated_timer(0);

        let mut want_stop_dash = false;
        const FEATURES0_TURN_WHILE_DASHING: u32 = 4;
        if self
            .enhanced_features_view()
            .has(FEATURES0_TURN_WHILE_DASHING)
        {
            if self.player_state_view().joypad1l_last() & 0x80 == 0 {
                self.player_state_view_mut().set_dash_countdown(0x11);
                want_stop_dash = true;
            } else {
                const DASH_CTRLS_TO_DIR: [u8; 16] =
                    [0, 1, 2, 0, 4, 4, 4, 0, 8, 8, 8, 0, 0, 0, 0, 0];
                let t =
                    DASH_CTRLS_TO_DIR[(self.player_state_view().joypad1h_last() & 0x0f) as usize];
                if t != 0 && t != self.player_state_view().last_direction() {
                    self.player_state_view_mut()
                        .set_direction_and_last_direction(t);
                    self.player_state_view_mut()
                        .set_swim_flags_from_last_direction();
                    self.link_handle_moving_animation_full_long_entry();
                }
            }
        } else {
            let dir = self.player_state_view().joypad1h_last() & 0x0f;
            want_stop_dash = dir != 0
                && dir != DASH_DIRECTION_BITS_BY_FACING[self.player_state_view().facing_index()];
        }
        if want_stop_dash {
            self.player_state_view_mut().set_handler_state(18);
            self.player_state_view_mut()
                .clear_button_mask_b_y_bits(0x80);
            self.player_state_view_mut().set_button_b_frames(0);
            self.player_state_view_mut().set_spin_attack_delay_timer(0);
            self.link_state_exiting_dash();
            return;
        }

        if self.player_state_view().speed_setting() == 0
            && self
                .enhanced_features_view()
                .has(FEATURES0_TURN_WHILE_DASHING)
        {
            self.player_state_view_mut().set_speed_setting(16);
        }
        let mut dir = (self.player_state_view().force_move_any_direction() as u8) & 0x0f;
        if dir == 0 {
            dir = DASH_DIRECTION_BITS_BY_FACING[self.player_state_view().facing_index()];
        }
        self.player_state_view_mut()
            .set_direction_and_last_direction(dir);
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.player_state_view_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_handle_sword_cooldown(&mut self) {
        if (self.player_state_view_mut().decrement_sword_delay_timer() as i8) >= 0 {
            return;
        }
        self.player_state_view_mut().clear_sword_delay_timer();
        if self.player_state_view().has_item_or_position_mode() {
            return;
        }
        if self.player_state_view().button_b_frames() < 9 {
            if !self.player_state_view().is_running() {
                self.link_check_for_sword_swing();
            }
        } else {
            self.handle_sword_controls();
        }
    }

    pub(super) fn handle_sword_sfx_and_beam(&mut self) {
        self.player_state_view_mut().clear_direction_flags(0x0f);
        self.player_state_view_mut().clear_button_b_frames();
        self.player_state_view_mut()
            .clear_spin_attack_step_counter();

        let health = self
            .player_resources_view()
            .health_capacity()
            .wrapping_sub(4);
        let sword = self.inventory_items().sword_type();
        if health < self.player_resources_view().current_health()
            && sword.wrapping_add(1) & 0xfe != 0
            && sword >= 2
            && !(0..5)
                .rev()
                .any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x31)
        {
            self.add_sword_beam(0);
        }
        let sword = sword.wrapping_sub(1);
        if sword != 0xfe && sword != 0xff {
            self.set_sound_effect_1_with_link_pan(FIRE_BEAM_SOUNDS[sword as usize]);
        }
        self.player_state_view_mut().set_spin_attack_delay_timer(1);
    }

    pub(super) fn link_check_for_sword_swing(&mut self) {
        if self.player_state_view().y_button_action_flags() & 0x10 != 0 {
            return;
        }
        if self.player_state_view().button_mask_b_y() & 0x80 == 0 {
            if self.player_state_view().filtered_joypad_h() & 0x80 == 0 {
                return;
            }
            if self.player_state_view().doorway_state() != 0 {
                self.tile_detect_sword_swing_deep_in_door(self.player_state_view().doorway_state());
                if self.tile_detect_position_view().collision_bits_low() & 0x30 == 0x30 {
                    return;
                }
            }
            self.player_state_view_mut().add_button_mask_b_y_bits(0x80);
            self.handle_sword_sfx_and_beam();
            self.player_state_view_mut().set_direction_lock_bits(1);
            self.player_state_view_mut().clear_animation_step();
        }

        if self.player_state_view().joypad1h_last() & 0x80 == 0 {
            self.player_state_view_mut().add_button_mask_b_y_bits(1);
        }
        self.halt_link_when_using_items();
        self.player_state_view_mut().clear_direction_flags(0x0f);
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            .is_negative()
        {
            let frames = self.player_state_view_mut().increment_button_b_frames();
            if frames >= 9 {
                self.handle_sword_controls();
                return;
            }
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(SPIN_ATTACK_DELAYS[frames as usize]);
            let sword = self.inventory_items().sword_type();
            if frames == 5 {
                if sword != 0 && sword != 1 && sword != 0xff {
                    self.ancilla_add_sword_swing_sparkle(0x26, 4);
                }
                if sword != 0 && sword != 0xff {
                    self.tile_detect_main_handler(if sword == 1 { 1 } else { 6 });
                }
            } else if frames >= 4
                && self.player_state_view().button_mask_b_y() & 1 != 0
                && self.player_state_view().joypad1h_last() & 0x80 != 0
            {
                self.player_state_view_mut().clear_button_mask_b_y_bits(1);
                self.handle_sword_sfx_and_beam();
                return;
            }
        }
        self.calculate_sword_hit_box();
    }

    pub(super) fn handle_sword_controls(&mut self) {
        if self.player_state_view().joypad1h_last() & 0x80 != 0 {
            self.player_sword_spin_attack_jerks_hold_down();
        } else if self.player_state_view().spin_attack_step_counter() < 48 {
            self.link_reset_sword_and_item_usage();
        } else {
            self.link_reset_sword_and_item_usage();
            self.player_state_view_mut()
                .clear_spin_attack_step_counter();
            self.link_activate_spin_attack();
        }
    }

    pub(super) fn player_sword_spin_attack_jerks_hold_down(&mut self) {
        if self.player_state_view().defense_flags() & 0x80 != 0
            || self.player_state_view().defense_flags() & 9 == 0
        {
            if self.sprite_battle_view().damaging_enemies_timer() == 0 {
                self.player_state_view_mut().set_button_b_frames(9);
                self.player_state_view_mut().set_direction_lock_bits(1);
                self.player_state_view_mut().set_spin_attack_delay_timer(0);
                if self.player_state_view().speed_setting() != 4
                    && self.player_state_view().speed_setting() != 16
                {
                    self.player_state_view_mut().set_speed_setting(12);
                    if self.inventory_items().sword_type().wrapping_add(1) & !1 == 0 {
                        return;
                    }
                    if (0..5)
                        .rev()
                        .any(|i| matches!(self.ancilla_slot_view(i).ancilla_type(), 0x30 | 0x31))
                    {
                        return;
                    }
                    if self.player_state_view().spin_attack_step_counter() >= 6
                        && self.frame_state().frame_counter & 3 == 0
                    {
                        self.ancilla_spawn_sword_charge_sparkle();
                    }
                    if self.player_state_view().spin_attack_step_counter() < 64 {
                        if self
                            .player_state_view_mut()
                            .increment_spin_attack_step_counter()
                            == 48
                        {
                            self.ancilla_sfx2_near(55);
                            self.ancilla_add_charged_spin_attack_sparkle();
                        }
                    }
                } else {
                    self.calculate_sword_hit_box();
                }
                return;
            } else if self.sprite_battle_view().damaging_enemies_timer() == 1 {
                self.link_reset_sword_and_item_usage();
                return;
            }
        }
        if self.player_state_view().button_b_frames() == 9 {
            self.player_state_view_mut().set_button_b_frames(10);
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(SPIN_ATTACK_DELAYS[10]);
        }
        if (self
            .player_state_view_mut()
            .decrement_spin_attack_delay_timer() as i8)
            .is_negative()
        {
            let mut frames = self.player_state_view().button_b_frames().wrapping_add(1);
            if frames == 13 {
                if self.inventory_items().sword_type().wrapping_add(1) & !1 != 0
                    && self.player_state_view().defense_flags() & 9 != 0
                {
                    self.ancilla_add_wall_tap_spark(27, 1);
                    self.ancilla_sfx2_near(if self.player_state_view().defense_flags() & 8 != 0 {
                        6
                    } else {
                        5
                    });
                    self.tile_detect_main_handler(1);
                }
                frames = 10;
            }
            self.player_state_view_mut().set_button_b_frames(frames);
            self.player_state_view_mut()
                .set_spin_attack_delay_timer(SPIN_ATTACK_DELAYS[frames as usize]);
        }
        self.calculate_sword_hit_box();
    }

    pub(super) fn link_activate_spin_attack(&mut self) {
        self.ancilla_add_spin_attack_init_spark(42, 0, 0);
        self.link_animate_victory_spin();
    }

    pub(super) fn link_animate_victory_spin(&mut self) {
        self.player_state_view_mut().set_handler_state(3);
        let spin_offsets = (self.player_state_view().facing() >> 1) * 12;
        self.player_state_view_mut().set_spin_offsets(spin_offsets);
        self.player_state_view_mut().set_spin_attack_delay_timer(3);
        let spin_state =
            LINK_SPIN_GRAPHICS_BY_DIR[self.player_state_view().spin_offsets() as usize];
        self.player_state_view_mut()
            .set_state_for_spin_attack(spin_state);
        self.player_state_view_mut()
            .clear_spin_animation_step_counter();
        self.player_state_view_mut().set_button_b_frames(144);
        self.player_state_view_mut().set_direction_lock_bits(1);
        self.player_state_view_mut().set_button_mask_b_y(0x80);
        self.link_state_spin_attack();
    }

    pub(super) fn link_state_tree_pull(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.player_state_view().has_auxiliary_state() {
            self.handle_link_from1_d();
            return;
        }

        if self.player_state_view().has_grabbing_wall_state() {
            if self.player_state_view().button_mask_b_y() == 0 {
                if self.player_state_view().joypad1l_last() & 0x80 == 0 {
                    self.player_state_view_mut().clear_grabbing_wall();
                    self.player_state_view_mut().clear_item_action_step_var();
                    self.player_state_view_mut().set_y_button_action_timer(2);
                    self.player_state_view_mut().set_y_button_action_step(0);
                    self.player_state_view_mut().clear_direction_lock();
                    self.player_state_view_mut().clear_handler_state();
                    self.link_state_default();
                    return;
                }
                if self.player_state_view().joypad1h_last() & 4 == 0 {
                    self.link_state_tree_pull_tail();
                    return;
                }
                self.player_state_view_mut().set_button_mask_b_y(4);
                self.ancilla_sfx2_near(0x22);
            }

            self.player_state_view_mut()
                .decrement_y_button_action_timer();
            if (self.player_state_view().y_button_action_timer() as i8) >= 0 {
                self.link_state_tree_pull_tail();
                return;
            }
            let j = self
                .player_state_view_mut()
                .increment_item_action_step_var() as usize;
            self.player_state_view_mut()
                .set_y_button_action_step(*GRAB_WALL_ANIM_STEPS.get(j).unwrap_or(&0));
            self.player_state_view_mut()
                .set_y_button_action_timer(*GRAB_WALL_ANIM_TIMER.get(j).unwrap_or(&0));
            if j != 7 {
                self.link_state_tree_pull_tail();
                return;
            }

            self.player_state_view_mut().clear_grabbing_wall();
            self.player_state_view_mut().clear_item_action_step_var();
            self.player_state_view_mut().set_y_button_action_timer(2);
            self.player_state_view_mut().set_y_button_action_step(0);
            self.player_state_view_mut().set_state_bits(1);
            self.player_state_view_mut().clear_picking_throw_state();
        }

        if self.player_state_view().defense_flags() & 9 != 0 {
            self.link_state_tree_pull_reset_to_normal();
            return;
        }
        if self.player_state_view().item_action_step_var() == 9 {
            if self.player_state_view().filtered_joypad_h() & 0x0f == 0 {
                self.link_handle_cardinal_collision();
                self.handle_indoor_camera_and_doors();
                return;
            }
            self.player_state_view_mut().clear_handler_state();
            self.link_state_default();
            return;
        }
        self.ancilla_add_dash_dust_charging(0x1e, 0);
        self.player_state_view_mut()
            .decrement_y_button_action_timer();
        if (self.player_state_view().y_button_action_timer() as i8) < 0 {
            let j = self
                .player_state_view_mut()
                .increment_item_action_step_var() as usize;
            self.player_state_view_mut()
                .set_y_button_action_step(GRAB_WALL_ANIM_STEPS2[j]);
            self.player_state_view_mut().set_y_button_action_timer(2);
            self.player_state_view_mut().set_actual_y_velocity(48);
            if j == 9 {
                self.link_state_tree_pull_reset_to_normal();
                return;
            }
        }
        self.flag67_with_directions();
        if self.player_state_view().direction() & 3 == 0 {
            self.player_state_view_mut().clear_actual_x_velocity();
        }
        if self.player_state_view().direction() & 0x0c == 0 {
            self.player_state_view_mut().clear_actual_y_velocity();
        }
        self.link_state_tree_pull_tail();
    }

    pub(super) fn link_handle_recoil_and_timer(&mut self, jump_into_middle: bool) {
        self.replay_trace_player_state(if jump_into_middle {
            "recoil-timer-entry-jump"
        } else {
            "recoil-timer-entry"
        });
        if !jump_into_middle {
            self.player_state_view_mut().clear_page_movement_deltas();
            self.player_state_view_mut()
                .clear_orthogonal_direction_count();
            self.link_handle_recoiling();
            if self.player_state_view_mut().decrement_incapacitated_timer() == 0 {
                self.player_state_view_mut()
                    .reset_elapsed_incapacitated_timer();
                if self.player_state_view().is_recoil_landing_z_window()
                    && (self.player_state_view().actual_z_velocity() as i8) < 0
                {
                    if self.player_state_view().has_auxiliary_state() {
                        self.player_state_view_mut()
                            .clear_sprite_damage_disable_timer();
                        let old_state = self.player_state_view().handler_state();
                        self.tile_detect_position_view_mut()
                            .set_interaction_scratch_y(old_state as u16);
                        if self.player_state_view().handler_state() != 6 {
                            self.player_state_view_mut().clear_button_b_frames();
                            self.player_state_view_mut().set_button_mask_b_y(0);
                            self.player_state_view_mut().set_spin_attack_delay_timer(0);
                            self.player_state_view_mut()
                                .clear_spin_attack_step_counter();
                        }
                        self.link_splash_upon_landing();
                        if !self.player_state_view().is_bunny_mirror()
                            || !self.player_state_view().is_in_deep_water()
                        {
                            if self.player_state_view().dash_noise_requested() {
                                self.player_state_view_mut().clear_dash_noise_request();
                                self.ancilla_sfx2_near(33);
                            } else if old_state != 2
                                && self.player_state_view().handler_state() != 4
                            {
                                self.ancilla_sfx2_near(33);
                            }
                            if self.player_state_view().handler_state() == 4 {
                                self.link_force_unequip_cape_quietly();
                                if self.world_location_state().is_indoors()
                                    && old_state != 2
                                    && self.player_state_view().has_flippers()
                                {
                                    self.player_state_view_mut().mark_lower_level();
                                }
                                self.ancilla_add_splash(21, 0);
                            }
                            self.tile_detect_main_handler(0);
                            if self.tile_detect_position_view().thick_grass_low() & 1 != 0 {
                                self.ancilla_sfx2_near(26);
                            }
                            if self.tile_detect_position_view().shallow_water_low() & 1 != 0
                                && self.system_signals_view().sound_effect_1() != 36
                            {
                                self.ancilla_sfx2_near(28);
                            }
                            if self.tile_detect_position_view().deepwater() & 1 != 0 {
                                self.player_state_view_mut().set_handler_state(4);
                                self.link_set_to_deep_water();
                                self.link_reset_sword_and_item_usage();
                                self.ancilla_add_splash(21, 0);
                            }
                        }
                        self.finish_recoil_landing();
                    }
                    self.player_state_view_mut().clear_animation_step();
                    self.player_state_view_mut().set_incapacitated_timer(0);
                }
            }
        } else {
            self.finish_recoil_landing();
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().set_incapacitated_timer(0);
        }

        if self.player_state_view().handler_state() != 5
            && self.player_state_view().incapacitated_timer() >= 33
        {
            if (self
                .player_state_view_mut()
                .decrement_incapacitated_camera_timer() as i8)
                >= 0
            {
                self.handle_indoor_camera_and_doors();
                self.player_state_view_mut().clear_z_high();
                return;
            }
            self.player_state_view_mut()
                .reset_incapacitated_camera_timer_from_incapacitated();
        }

        self.flag67_with_directions();
        if self.player_state_view().handler_state() != 6 {
            self.link_handle_diagonal_collision();
            if self.player_state_view().direction() & 3 == 0 {
                self.player_state_view_mut().clear_actual_x_velocity();
            }
            if self.player_state_view().direction() & 0x0c == 0 {
                self.player_state_view_mut().clear_actual_y_velocity();
            }
        }
        self.link_move_position();

        if self.player_state_view().handler_state() != 6 {
            self.link_handle_cardinal_collision();
            self.player_state_view_mut().clear_pit_correction();
        }
        self.handle_indoor_camera_and_doors();
        if self.player_state_view().should_probe_recoil_landing_tile() {
            self.player_tile_detect_nearby();
            self.replay_trace_player_state("recoil-timer-after-nearby");
            if self.tile_detect_position_view().pit_tile() & 0x0f == 0x0f {
                self.player_state_view_mut().set_handler_state(1);
                self.player_state_view_mut().set_speed_setting(4);
                self.replay_trace_player_state("recoil-timer-set-pit");
            }
        }
        self.player_state_view_mut().clear_z_high();
        self.replay_trace_player_state("recoil-timer-exit");
    }

    pub(super) fn gravestone_move(&mut self, k: usize) {
        if self.frame_state().submodule != 0 {
            return;
        }
        self.ancilla_slot_view_mut(k).set_y_velocity((-8i8) as u8);
        self.ancilla_move_y(k);

        self.gravestone_act_as_barrier(k);
        let y_target = self.ancilla_slot_view(k).ab_word();
        let y_cur = self.ancilla_get_y(k);
        if y_cur >= y_target {
            return;
        }

        self.ancilla_slot_view_mut(k).set_ancilla_type(0);
        self.player_state_view_mut().clear_hookshot_grave_latch();
        self.player_state_view_mut().and_defense_flags(!4);
        let debris_y = self.door_debris_view().y(k);
        let debris_x = self.door_debris_view().x(k);
        self.tile_detect_position_view_mut()
            .set_interaction_scratch_y_bytes(debris_y, debris_x);
        let big_rock = self.tile_detect_position_view().interaction_scratch_y();
        self.dungeon_state_view_mut()
            .set_big_rock_starting_address(big_rock);
        let counter = match big_rock {
            0x0532 => 0x48,
            0x0488 => 0x60,
            _ => 0x40,
        };
        self.dungeon_state_view_mut().set_door_open_counter(counter);
        self.overworld_do_map_update32x32_b_for_smash();
    }

    pub(super) fn somaria_block_handle_player_interaction(&mut self, k: usize) {
        self.sprite_system_view_mut().set_cur_object_index(k as u8);
        if self.ancilla_slot_view(k).g() != 0 {
            return;
        }

        if self.ancilla_slot_view(k).h() == 0 {
            if self.player_state_view().has_auxiliary_state()
                || self.player_state_view().state_bits_has(1)
                || {
                    let z = self.ancilla_slot_view(k).z();
                    z != 0 && z != 0xff
                }
                || self.ancilla_slot_view(k).k() != 0
                || self.ancilla_slot_view(k).l() != 0
            {
                return;
            }
            if self.player_state_view().joypad1h_last() & 0x0f == 0 {
                self.ancilla_slot_view_mut(k).set_work_byte_3(0);
                self.player_state_view_mut().clear_defense_flags();
                self.ancilla_slot_view_mut(k).set_a(255);
                if !self.player_state_view().is_running() {
                    self.player_state_view_mut().set_speed_setting(0);
                    return;
                }
            } else if self.player_state_view().joypad1h_last() & 0x0f
                == self.ancilla_slot_view(k).work_byte_3()
            {
                if self.player_state_view().speed_setting() == 18 {
                    self.player_state_view_mut().or_defense_flags(0x81);
                }
            } else {
                let last_direction = self.player_state_view().joypad1h_last() & 0x0f;
                self.ancilla_slot_view_mut(k)
                    .set_work_byte_3(last_direction);
                self.player_state_view_mut().set_speed_setting(0);
            }

            if !self.ancilla_check_link_collision(k, 4)
                || self.ancilla_slot_view(k).floor() != self.player_state_view().lower_level_state()
            {
                return;
            }

            if !self.player_state_view().is_running()
                || self.player_state_view().dash_counter() == 64
            {
                let t = self.player_state_view().joypad1h_last() & 0x0f;
                self.ancilla_slot_view_mut(k).set_work_byte_3(t);
                if t & 3 != 0 {
                    let mut ancilla = self.ancilla_slot_view_mut(k);
                    ancilla.set_y_velocity(0);
                    ancilla.set_x_velocity(if t & 1 != 0 { 16 } else { (-16i8) as u8 });
                    ancilla.set_direction(if t & 1 != 0 { 3 } else { 2 });
                } else {
                    let mut ancilla = self.ancilla_slot_view_mut(k);
                    ancilla.set_x_velocity(0);
                    ancilla.set_y_velocity(if t & 8 != 0 { (-16i8) as u8 } else { 16 });
                    ancilla.set_direction(if t & 8 != 0 { 0 } else { 1 });
                }
                if self.player_state_view().actual_y_velocity() == 0
                    || self.player_state_view().actual_x_velocity() == 0
                {
                    if !self.ancilla_check_tile_collision_class2(k) {
                        self.ancilla_move_y(k);
                        self.ancilla_move_x(k);
                        let movement_ticks = self.ancilla_slot_view_mut(k).advance_a();
                        if !self.player_state_view().is_lifting_or_carrying()
                            && movement_ticks & 7 == 0
                        {
                            self.ancilla_sfx2_pan(k, 0x22);
                        }
                    }
                    self.player_state_view_mut().set_defense_flags(0x81);
                    self.player_state_view_mut().set_speed_setting(0x12);
                }
                self.sprite_nullify_hookshot_drag();
                return;
            }

            const SOMARIA_BLOCK_YVEL: [u8; 4] = [(-40i8) as u8, 40, 0, 0];
            const SOMARIA_BLOCK_XVEL: [u8; 4] = [0, 0, (-40i8) as u8, 40];
            if self.player_state_view().ancilla_pickup_flag() == k as u8 + 1 {
                self.player_state_view_mut().clear_ancilla_pickup_flag();
            }
            self.link_cancel_dash();
            self.ancilla_sfx3_pan(k, 0x32);
            let j = self.player_state_view().facing_index();
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_direction(j as u8);
                ancilla.set_y_velocity(SOMARIA_BLOCK_YVEL[j]);
                ancilla.set_x_velocity(SOMARIA_BLOCK_XVEL[j]);
                ancilla.set_z_velocity(48);
                ancilla.set_z(0);
            }
            self.ancilla_slot_view_mut(k).set_h(1);
        }

        self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
        self.ancilla_move_z(k);
        let z = self.ancilla_slot_view(k).z();
        if z != 0 && z < 252 {
            return;
        }

        self.ancilla_sfx2_pan(k, 0x21);
        self.ancilla_slot_view_mut(k).set_z(0);
        let j = self.ancilla_slot_view(k).h();
        self.ancilla_slot_view_mut(k).set_h(j.wrapping_add(1));
        if j == 3 {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_work_byte_4(0);
            ancilla.set_h(0);
        } else {
            const SOMARIA_BLOCK_ZVEL: [u8; 4] = [48, 24, 16, 8];
            let mut ancilla = self.ancilla_slot_view_mut(k);
            let y_velocity = ((ancilla.y_velocity() as i8) / 2) as u8;
            let x_velocity = ((ancilla.x_velocity() as i8) / 2) as u8;
            ancilla.set_z_velocity(SOMARIA_BLOCK_ZVEL[j.wrapping_sub(1) as usize]);
            ancilla.set_y_velocity(y_velocity);
            ancilla.set_x_velocity(x_velocity);
        }
    }

    pub(super) fn gravestone_act_as_barrier(&mut self, k: usize) {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        let r4 = y.wrapping_add(0x18);
        let r6 = x.wrapping_add(0x20);
        let lx = self.player_state_view().x().wrapping_add(8);
        let ly = self.player_state_view().y().wrapping_add(8);
        if ly >= y && ly < r4 && lx >= x && lx < r6 {
            let r10 = ly.abs_diff(r4);
            let new_y = self.player_state_view().y().wrapping_add(r10);
            self.player_state_view_mut().set_y(new_y);
            self.player_state_view_mut().add_y_velocity_delta(r10 as u8);
            self.player_state_view_mut().or_defense_flags(4);
        }
        if self.player_state_view().has_facing() {
            let facing = self.player_state_view().facing() & !4;
            self.player_state_view_mut().set_facing(facing);
        }
    }

    pub(super) fn link_handle_recoiling(&mut self) {
        self.player_state_view_mut().set_direction(0);
        if self.player_state_view().actual_y_velocity() != 0 {
            let direction = if self
                .player_state_view()
                .actual_y_velocity_signed()
                .is_negative()
            {
                8
            } else {
                4
            };
            self.player_state_view_mut().add_direction_flags(direction);
            self.player_state_view_mut()
                .set_last_direction_from_current_direction();
            self.player_handle_incapacitated_inner2();
        }
        if self.player_state_view().actual_x_velocity() != 0 {
            let direction = if self
                .player_state_view()
                .actual_x_velocity_signed()
                .is_negative()
            {
                2
            } else {
                1
            };
            self.player_state_view_mut().add_direction_flags(direction);
            self.player_state_view_mut()
                .set_last_direction_from_current_direction();
        }
        self.player_handle_incapacitated_inner2();
    }

    pub(super) fn player_handle_incapacitated_inner2(&mut self) {
        if self
            .player_state_view()
            .is_moving_against_diag_tile_on_both_axes()
            && self.player_state_view().handler_state() == 2
        {
            self.player_state_view_mut().invert_actual_velocity_xy();
        }
        if self.player_state_view().doorway_state() == 1 {
            self.player_state_view_mut().mask_last_direction(0x0c);
            self.player_state_view_mut().mask_direction(0x0c);
            self.player_state_view_mut().clear_actual_x_velocity();
        } else if self.player_state_view().doorway_state() == 2 {
            self.player_state_view_mut().mask_last_direction(3);
            self.player_state_view_mut().mask_direction(3);
            self.player_state_view_mut().clear_actual_y_velocity();
        }
    }

    pub(super) fn find_free_moving_block_slot(&mut self, x: u8) -> u8 {
        if self.dungeon_state_view().changeable_object_index(1) == 0 {
            self.dungeon_state_view_mut()
                .set_changeable_object_index(1, x.wrapping_add(1));
            return 1;
        }
        if self.dungeon_state_view().changeable_object_index(0) == 0 {
            self.dungeon_state_view_mut()
                .set_changeable_object_index(0, x.wrapping_add(1));
            return 0;
        }
        0xff
    }

    pub(super) fn initialize_push_block(&mut self, r14: u8, idx: u8) -> bool {
        let slot = r14 as usize;
        let idx_word = (idx >> 1) as usize;
        let pos = self.dungeon_state_view().object_tilemap_pos(idx_word);
        let mut x = (pos & 0x007e) << 2;
        let mut y = (pos & 0x1f80) >> 4;
        x = x.wrapping_add(self.dungeon_state_view().loading_bg_offset_h() & 0xff00);
        y = y.wrapping_add(self.dungeon_state_view().loading_bg_offset_v() & 0xff00);

        self.pushed_block_view_mut().init_slot(slot, x, y);

        if self.dungeon_state_view().primary_header_tag() != 38
            && self.dungeon_state_view().replacement_tile_state(idx_word) == 0
        {
            if !self.push_block_attempt_to_push_the_block(0, x, y) {
                self.ancilla_sfx2_near(0x22);
                self.dungeon_state_view_mut()
                    .set_replacement_tile_state(idx_word, 1);
                return false;
            }
        }

        self.dungeon_state_view_mut()
            .clear_changeable_object_index(slot);
        true
    }

    pub(super) fn sprite_dungeon_draw_single_push_block(&mut self, mut j: usize) {
        const PUSH_BLOCK_CHAR_INDEX_BY_MODE: [usize; 9] = [0, 1, 2, 3, 4, 0, 0, 0, 0];
        const CHAR: [u8; 4] = [0x0c, 0x0c, 0x0c, 0xff];
        j >>= 1;
        self.oam_allocate_from_region_b(4);
        let y = self
            .pushed_block_view()
            .y(j)
            .wrapping_sub(self.world_state_view().bg2_y())
            .wrapping_sub(1);
        let x = self
            .pushed_block_view()
            .x(j)
            .wrapping_sub(self.world_state_view().bg2_x());
        let ch = CHAR[PUSH_BLOCK_CHAR_INDEX_BY_MODE
            [self.pushed_block_view().animation_mode() as usize]
            .min(CHAR.len() - 1)];
        if ch != 0xff {
            let oam = self.oam_state_view().current_pointer_usize();
            self.oam_state_view_mut()
                .write_entry(oam, x as u8, y as u8, ch, 0x20);
            let ext = self.oam_state_view().current_extended_pointer_usize();
            self.oam_state_view_mut().set_extended_byte_at(ext, 2);
        }
    }

    pub(super) fn handle_layer_of_destination(&mut self) {
        let hole_teleporter_plane = self.dungeon_header_view().hole_teleporter_plane(0);
        self.player_state_view_mut().set_lower_level_states(
            u8::from(hole_teleporter_plane >= 2),
            u8::from(hole_teleporter_plane >= 1),
        );
    }

    pub(super) fn dungeon_pit_do_damage(&mut self) {
        self.set_submodule(20);
        if self
            .player_resources_view_mut()
            .decrement_current_health_by(8)
            >= 0xa8
        {
            self.player_resources_view_mut().set_current_health(0);
        }
    }

    pub(super) fn reset_some_things_after_death(&mut self, speed_setting: u8) {
        self.player_state_view_mut().clear_deep_water_state();
        self.player_state_view_mut()
            .set_speed_setting(speed_setting);
        self.player_state_view_mut().clear_conveyor_belt_state();
        self.player_state_view_mut().set_layer_collision_flags(0);
        self.player_state_view_mut().clear_immobilized();
        self.follower_state_view_mut().clear_palette_swap_flag();
        self.player_state_view_mut().clear_faint_animation_active();
        self.player_state_view_mut().clear_given_damage();
        self.player_state_view_mut().clear_actual_velocity_xy();
        self.player_state_view_mut().set_actual_z_velocity(0);
        self.player_state_view_mut().set_z(0);
        self.player_state_view_mut()
            .clear_water_ripple_or_grass_state();
        self.tile_detect_position_view_mut()
            .set_tile_collision_bits_primary(0);
        self.player_state_view_mut().clear_blink_countdown();
        self.player_state_view_mut().clear_handler_state();
        self.player_state_view_mut().set_visibility_status(0);
        self.ancilla_terminate_select_interactives(0);
        self.link_reset_properties_a();
    }

    pub(super) fn player_handler_00_ground_3(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        self.apply_links_movement_to_camera_called = false;
        self.player_state_view_mut().set_z(0xffff);
        self.player_state_view_mut().set_actual_z_velocity(0xff);
        self.player_state_view_mut().set_recoil_timer(0);

        let mut clear_vel_after = false;
        if !self.link_handle_toss() {
            self.link_handle_a_press();
            if !self.player_state_view().has_action_state()
                && !self.player_state_view().has_grabbing_wall_state()
                && !self.player_state_view().has_pull_action_state()
                && self.player_state_view().handler_state() != 17
            {
                self.link_handle_y_item();
                if self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES)
                    && ((self.frame_state().main_module == 14 && self.frame_state().submodule != 2)
                        || matches!(self.player_state_view().handler_state(), 8 | 9 | 10))
                {
                    self.finish_ground_movement_clear_vel_tail();
                    return;
                }
                if self.save_progress_view().progress_indicator() != 0 {
                    self.link_handle_sword_cooldown();
                    if self.player_state_view().handler_state() == 3 {
                        self.finish_ground_movement_clear_vel_tail();
                        return;
                    }
                }
            }
        }

        self.link_handle_cape_passive_lift_check();
        if self.player_state_view().incapacitated_timer() != 0 {
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
            self.player_state_view_mut()
                .clear_lift_throw_scratch_state();
            self.player_state_view_mut().set_y_button_action_step(0);
            self.player_state_view_mut().set_y_button_action_flags(0);
            self.player_state_view_mut()
                .clear_state_item_and_grab_flags();
            if self.player_state_view().button_mask_b_y() & 0x80 == 0 {
                self.player_state_view_mut().clear_direction_lock_bits(1);
            }
            self.link_handle_recoil_and_timer(false);
            return;
        }

        if self.player_state_view().has_pull_action_state() {
            self.player_state_view_mut().set_direction(0);
            clear_vel_after = true;
        } else if !self.player_state_view().is_transforming()
            && self.player_state_view().is_ready_to_start_ground_movement()
            && (self.player_state_view().button_b_frames() >= 9
                || (self.player_state_view().button_mask_b_y() & 0x20) != 0
                || (self.player_state_view().button_mask_b_y() & 0x80) == 0)
        {
            if self.player_state_view().flag_moving() != 0 {
                self.swim_acceleration_view_mut().set_max_speed(0, 0x0180);
                self.swim_acceleration_view_mut().set_max_speed(2, 0x0180);
                self.link_handle_swim_movements();
                return;
            }

            self.reset_all_acceleration();
            let mut dir = (self.player_state_view().force_move_any_direction() as u8) & 0x0f;
            if dir == 0 {
                if self.player_state_view().grabbing_wall_has(2) {
                    self.finish_ground_movement_tail(clear_vel_after);
                    return;
                }
                dir = self.player_state_view().joypad1h_last() & 0x0f;
            }
            if dir == 0 {
                self.player_state_view_mut()
                    .clear_movement_velocity_and_direction();
                self.player_state_view_mut().set_last_direction(0);
                self.player_state_view_mut().clear_animation_step();
                self.player_state_view_mut().and_defense_flags(!0x0f);
                self.player_state_view_mut().reset_push_fatigue_timer();
                self.player_state_view_mut().reset_jump_ledge_timer();
            } else {
                self.player_state_view_mut().set_direction(dir);
                if dir != self.player_state_view().last_direction() {
                    self.player_state_view_mut().set_last_direction(dir);
                    self.player_state_view_mut().clear_movement_subpixels();
                    self.player_state_view_mut()
                        .clear_moving_against_diag_tile();
                    self.player_state_view_mut().clear_defense_flags();
                    self.player_state_view_mut().reset_push_fatigue_timer();
                    self.player_state_view_mut().reset_jump_ledge_timer();
                }
            }
        }

        self.finish_ground_movement_tail(clear_vel_after);
    }

    pub(super) fn link_perform_throw(&mut self) {
        const LIFTABLE_TILE_ATTR_TO_TERRAIN_TYPE: [u8; 9] =
            [0x54, 0x52, 0x50, 0xff, 0x51, 0x53, 0x55, 0x56, 0x57];
        if (self.player_state_view().sprite_pickup_flag()
            | self.player_state_view().ancilla_pickup_flag())
            == 0
        {
            self.link_reset_sword_and_item_usage();
            self.player_state_view_mut().set_y_button_action_flags(0);
            let mut i = 15i8;
            while self.sprite_slot_view(i as usize).state() != 0 {
                i -= 1;
                if i < 0 {
                    return;
                }
            }

            if matches!(
                self.tile_detect_position_view()
                    .liftable_action_index_primary(),
                5 | 6
            ) {
                self.player_state_view_mut().set_action_handler_timer(1);
            } else {
                let (attr, x, y) = if self.world_location_state().is_indoors() {
                    let mut pt = Point16U { x: 0, y: 0 };
                    let attr = self.Dungeon_LiftAndReplaceLiftable(&mut pt);
                    (attr, pt.x, pt.y)
                } else {
                    let mut pt = Point16U { x: 0, y: 0 };
                    let attr = self.Overworld_HandleLiftableTiles(&mut pt);
                    (attr, pt.x, pt.y)
                };
                let Some(idx) = LIFTABLE_TILE_ATTR_TO_TERRAIN_TYPE
                    .iter()
                    .rposition(|&value| value == attr)
                else {
                    return;
                };
                self.player_state_view_mut().set_sprite_pickup_flag(1);
                self.sprite_spawn_throwable_terrain(idx as u8, x, y);
                self.player_state_view_mut()
                    .clear_filtered_joypad_l_bits(0x80);
                self.player_state_view_mut().clear_action_handler_timer();
            }
        } else {
            self.player_state_view_mut().clear_action_handler_timer();
        }

        self.player_state_view_mut().set_button_mask_b_y(0);
        self.player_state_view_mut().set_y_button_action_timer(6);
        self.player_state_view_mut().start_lift_throw_state();
        self.player_state_view_mut().set_y_button_action_step(0);
        self.player_state_view_mut().set_speed_setting(12);
        self.player_state_view_mut().clear_animation_step();
        self.player_state_view_mut().mask_direction(0xf0);
        self.player_state_view_mut().set_direction_lock_bits(1);
    }

    pub(super) fn spawn_hammer_water_splash(&mut self) {
        const HAMMER_WATER_X: [i8; 4] = [0, 12, -8, 24];
        const HAMMER_WATER_Y: [i8; 4] = [8, 32, 24, 24];
        if (self.frame_state().submodule
            | self.player_state_view().immobilized_flag()
            | self.frame_state().modal_pause_flag)
            != 0
        {
            return;
        }
        let i = self.player_state_view().facing_index();
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(HAMMER_WATER_X[i] as i16 as u16);
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(HAMMER_WATER_Y[i] as i16 as u16);
        let tiletype = if self.world_location_state().is_indoors() {
            let mut t = if self.player_state_view().lower_level_state() >= 1 {
                0x1000
            } else {
                0
            };
            t += ((x & 0x01f8) >> 3) as usize;
            t += ((y & 0x01f8) << 3) as usize;
            self.dungeon_state_view().bg2_attr(t)
        } else {
            self.overworld_get_tile_attribute_at_location(x >> 3, y)
        };

        if matches!(tiletype, 8 | 9) {
            let j = self.sprite_spawn_small_splash(0);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_x(j, x.wrapping_sub(8));
                self.sprite_set_y(j, y.wrapping_sub(16));
                let floor = self.player_state_view().lower_level_state();
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_floor(floor);
                sprite.set_z(0);
            }
        }
    }

    pub(super) fn digging_game_guy_attempt_prize_spawn(&mut self) {
        const DIGGING_GAME_XVEL: [u8; 2] = [(-16i8) as u8, 16];
        const DIGGING_GAME_X: [i8; 2] = [0, 19];
        const DIGGING_GAME_ITEMS: [u8; 4] = [0xdb, 0xda, 0xd9, 0xdf];

        self.digging_game_prize_view_mut().increment_attempts();
        if self.player_state_view().y() >= 0x0b18 {
            return;
        }
        let j = self.get_random_number() & 7;
        let item_to_spawn = match j {
            0..=3 => DIGGING_GAME_ITEMS[j as usize],
            4 => {
                if self.digging_game_prize_view().attempts() < 25
                    || self.digging_game_prize_view().spawned_marker() != 0
                    || self.get_random_number() & 3 != 0
                {
                    return;
                }
                self.digging_game_prize_view_mut().mark_spawned();
                0xeb
            }
            _ => return,
        };

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(4, item_to_spawn, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = usize::from(self.player_state_view().facing() != 4);
            {
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_x_velocity(DIGGING_GAME_XVEL[i]);
                sprite.set_y_velocity(0);
                sprite.set_z_velocity(24);
                sprite.set_stunned(255);
                sprite.set_delay_aux4(48);
            }
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(DIGGING_GAME_X[i] as i16 as u16)
                & !0x0f;
            let y = self.player_state_view().y().wrapping_add(22) & !0x0f;
            self.sprite_set_x(j, x);
            self.sprite_set_y(j, y);
            self.sprite_slot_view_mut(j).set_floor(0);
            self.sprite_sfx_queue_sfx3_with_pan(j, 0x30);
        }
    }

    fn sprite_spawn_small_splash_for_player(&mut self) -> Option<usize> {
        self.sprite_spawn_dynamically_for_player(0, 0xec)
    }

    fn sprite_spawn_dynamically_for_player(&mut self, _k: u8, what: u8) -> Option<usize> {
        let j = (0..16)
            .rev()
            .find(|&j| self.sprite_slot_view(j).state() == 0)?;
        {
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_state(9);
            sprite.set_sprite_type(what);
        }
        Some(j)
    }

    pub(super) fn handle_indoor_camera_and_doors(&mut self) {
        if self.world_location_state().is_outdoors() {
            return;
        }
        if self.player_state_view().doorway_state() != 0 {
            self.handle_door_transitions();
        } else {
            self.apply_links_movement_to_camera();
        }
    }

    pub(super) fn cache_camera_properties_if_outdoors(&mut self) {
        if self.world_location_state().is_outdoors() {
            self.cache_camera_properties_for_player();
        }
    }

    pub(super) fn handle_door_transitions(&mut self) {
        self.player_state_view_mut().clear_page_movement_deltas();

        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES)
            && !(self.frame_state().main_module == 7 && self.frame_state().submodule == 0)
        {
            return;
        }

        if self.player_state_view().last_direction() & 0x0c != 0
            && self.player_state_view().doorway_state() == 1
        {
            if self.player_state_view().last_direction() & 4 != 0 {
                let t = self.player_state_view().y().wrapping_add(28);
                if t & 0x00fc == 0 {
                    self.player_state_view_mut()
                        .set_y_page_movement_delta_from_high_position((t >> 8) as u8);
                }
            } else {
                let t = self.player_state_view().y().wrapping_sub(18);
                self.player_state_view_mut()
                    .set_y_page_movement_delta_from_high_position((t >> 8) as u8);
            }
        }

        if self.player_state_view().last_direction() & 3 != 0
            && self.player_state_view().doorway_state() == 2
        {
            if self.player_state_view().last_direction() & 1 != 0 {
                let t = self.player_state_view().x().wrapping_add(21);
                if t & 0x00fc == 0 {
                    self.player_state_view_mut()
                        .set_x_page_movement_delta_from_high_position((t >> 8) as u8);
                }
            } else {
                let t = self.player_state_view().x().wrapping_sub(8);
                self.player_state_view_mut()
                    .set_x_page_movement_delta_from_high_position((t >> 8) as u8);
            }
        }

        if self.player_state_view().x_page_movement_delta() != 0 {
            self.player_state_view_mut().set_y_button_action_timer(0);
            self.player_state_view_mut()
                .clear_state_item_and_grab_flags();
            if self
                .player_state_view()
                .x_page_movement_delta_signed()
                .is_negative()
            {
                self.Dung_StartInterRoomTrans_Left_Plus();
            } else {
                self.HandleEdgeTransitionMovementEast_RightBy8();
            }
        } else if self.player_state_view().y_page_movement_delta() != 0 {
            self.player_state_view_mut().set_y_button_action_timer(0);
            self.player_state_view_mut()
                .clear_state_item_and_grab_flags();
            if self
                .player_state_view()
                .y_page_movement_delta_signed()
                .is_negative()
            {
                self.Dungeon_StartInterRoomTrans_Up();
            } else {
                self.HandleEdgeTransitionMovementSouth_DownBy16();
            }
        }
    }

    pub(super) fn apply_links_movement_to_camera(&mut self) {
        self.apply_links_movement_to_camera_called = true;
        let (y_delta, x_delta) = {
            let player = self.player_state_view();
            (
                player.y_high().wrapping_sub(player.safe_return_y_high()),
                player.x_high().wrapping_sub(player.safe_return_x_high()),
            )
        };
        self.player_state_view_mut()
            .set_page_movement_deltas(y_delta, x_delta);

        if self.player_state_view().x_page_movement_delta() != 0 {
            if self
                .player_state_view()
                .x_page_movement_delta_signed()
                .is_negative()
            {
                self.AdjustQuadrantAndCamera_left();
            } else {
                self.AdjustQuadrantAndCamera_right();
            }
        }
        if self.player_state_view().y_page_movement_delta() != 0 {
            if self
                .player_state_view()
                .y_page_movement_delta_signed()
                .is_negative()
            {
                self.AdjustQuadrantAndCamera_up();
            } else {
                self.AdjustQuadrantAndCamera_down();
            }
        }
    }
}
